#!/usr/bin/env ruby
# The invalid-input Rust reference must remain independent of typed receipt
# targets. Root compared these exact blocks with misc.rs at 412aa43; Git is not
# required when running this audit from a source archive.
require 'digest'

root = File.expand_path('..', __dir__)
frozen = File.read(File.join(root, 'crates/z3-game/src/misc/receipt_frozen.rs'))
tables = %w[
  ITEM_TARGETS ITEM_VALUES BOTTLE_LIST POTION_LIST RECEIVE_ITEM_TAB1
  RECEIVE_ITEM_TAB2 RECEIVE_ITEM_TAB3 RECEIVE_ITEM_GFX
]
functions = %w[item_receipt_give_bottled_item ancilla_add_item_receipt]
abort 'receipt reference must not use the production target domain' if frozen.match?(/inventory_receipt|ReceiptItem|ReceiptTarget|BottleSlots/)
abort 'frozen receipt constant set/order changed' unless frozen.scan(/^(?:pub )?const (\w+)/).flatten == tables
abort 'frozen receipt function set/order changed' unless frozen.scan(/^(?:pub )?fn (\w+)/).flatten == functions

def reference_block(source, name, kind)
  match = source.match(/^(?:pub )?#{kind} #{Regexp.escape(name)}\b/)
  abort "missing frozen #{kind} #{name}" unless match
  first = match.begin(0)
  if kind == 'const'
    last = source.index(';', source.index('=', first)) + 1
  else
    last = source.index('{', first) + 1
    depth = 1
    while depth.positive?
      character = source[last]
      abort "unclosed frozen #{name}" unless character
      depth += 1 if character == '{'
      depth -= 1 if character == '}'
      last += 1
    end
  end
  source[first...last].gsub(%r{//[^\n]*}, '').gsub(/\s+/, '')
end

blocks = tables.map { |name| reference_block(frozen, name, 'const') } +
         functions.map { |name| reference_block(frozen, name, 'fn') }
expected = '88b3eedce55e99b77a630dcc04311c83f056d75193554cdd5ad2fc9facb4956a'
abort 'frozen receipt differs from independently audited 412aa43 reference' unless Digest::SHA256.hexdigest(blocks.join) == expected
puts 'Frozen receipt: all eight tables and two functions match independently audited 412aa43 reference'

# Resolve the production target identities independently through the unchanged
# C tables, not through the frozen Rust reference or a Rust round-trip codec.
c_source = File.read(File.join(root, 'src/misc.c'))
schema = File.read(File.join(root, 'crates/z3-game/src/inventory_receipt/target_schema.rs'))
domain = File.read(File.join(root, 'crates/z3-game/src/inventory_receipt.rs'))

def numeric_array(source, type, name, count)
  body = source[/(?:static\s+)?const\s+#{type}\s+#{name}\[#{count}\]\s*=\s*\{(.*?)\};/m, 1]
  raise "missing C array #{name}" unless body
  rows = body.gsub(%r{//[^\n]*}, '').split(',').map(&:strip).reject(&:empty?)
  raise "C array length changed #{name}" unless rows.length == count
  rows.map do |token|
    raise "unparsed C token #{token.inspect}" unless token.match?(/\A-?(?:0x[0-9a-fA-F]+|\d+)\z/)
    Integer(token)
  end
end

def compare_targets(c_source, schema, domain)
  addresses = numeric_array(c_source, 'uint16', 'kMemoryLocationToGiveItemTo', 76)
  values = numeric_array(c_source, 'int8', 'kValueToGiveItemTo', 76)
  body = schema[/\btarget_fields!\s*\{(.*?)\}/m, 1]
  raise 'missing Rust target schema' unless body
  targets = body.lines.reject { |line| line.strip.empty? }.map do |line|
    row = /^\s*(\w+)\s*=>\s*(0x[0-9a-fA-F]+);\s*$/.match(line)
    raise "unparsed Rust target #{line.inspect}" unless row
    [row[1], row[2].to_i(16)]
  end
  raise 'duplicate Rust target names' unless targets.map(&:first).uniq.length == targets.length
  raise 'receipt target domain cardinality changed' unless targets.length == 41
  raise 'Rust target addresses/order differ from distinct original C targets' unless targets.map(&:last) == addresses.uniq

  body = schema[/const RECEIPTS:\s*\[ReceiptSpec;\s*76\]\s*=\s*\[(.*?)\n\];/m, 1]
  raise 'missing ordered receipt schema' unless body
  body = body.gsub(%r{//[^\n]*}, '')
  pattern = /ReceiptSpec\s*\{\s*target:\s*InventoryTarget::(\w+),\s*initial_value:\s*(-?\d+),\s*\}/
  rows = body.scan(pattern)
  raise 'unparsed ordered receipt rows' unless body.gsub(pattern, '').match?(/\A[\s,]*\z/)
  raise 'receipt row count changed' unless rows.length == 76
  target_map = targets.to_h
  actual = rows.map { |name, value| [target_map.fetch(name), Integer(value)] }
  raise 'ordered receipt target/initial values differ from C' unless actual == addresses.zip(values)

  [['BOTTLE_ITEMS', 'kBottleList', 7], ['POTION_ITEMS', 'kPotionList', 5]].each do |rust_name, c_name, count|
    rust_body = domain[/const #{rust_name}:\s*\[u8;\s*#{count}\]\s*=\s*\[([^\]]*)\];/, 1]
    raise "missing Rust #{rust_name}" unless rust_body
    actual_items = rust_body.split(',').map { |value| Integer(value.strip) }
    raise "bottle/potion mapping differs #{rust_name}" unless actual_items == numeric_array(c_source, 'uint8', c_name, count)
  end
end

begin
  compare_targets(c_source, schema, domain)
  probes = [
    [c_source.sub('kMemoryLocationToGiveItemTo[76]', 'kMemoryLocationToGiveItemTo[75]'), schema, domain],
    [c_source, schema.sub('Sword => 0xf359', 'Sword => 0xf358'), domain],
    [c_source, schema.sub('initial_value: 1,', 'initial_value: 2,'), domain],
    [c_source, schema.sub(/ReceiptSpec\s*\{\s*target:\s*InventoryTarget::Sword,\s*initial_value:\s*1,\s*\},/, ''), domain],
    [c_source, schema, domain.sub('0x2e, 0x2f, 0x30, 0xff', '0x2e, 0x2f, 0x30, 0xfe')]
  ]
  probes.each_with_index do |sources, index|
    raise "inventory probe #{index} did not alter source" if sources == [c_source, schema, domain]
    rejected = false
    begin
      compare_targets(*sources)
    rescue StandardError
      rejected = true
    end
    raise "inventory drift probe #{index} escaped detection" unless rejected
  end
rescue StandardError => error
  abort error.message
end
puts 'Receipt targets: all 76 ordered C grants, 41 live identities and bottle/potion mappings match; drift probes rejected'
