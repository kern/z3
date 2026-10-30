#!/usr/bin/env ruby
# Check the sole live Garnish schema against unchanged C aliases, and nominal
# traversal constants against C loop bounds. Raw indices remain unrestricted;
# nominal counts here must never become new validation at the storage boundary.
root = File.expand_path('..', __dir__)
variables = File.read(File.join(root, 'src/variables.h'))
c_sprite = File.read(File.join(root, 'src/sprite.c'))
rust = File.read(File.join(root, 'crates/z3-game/src/garnish_state.rs'))

FIELDS = {
  'kind' => 'garnish_type', 'y_low' => 'garnish_y_lo',
  'x_low' => 'garnish_x_lo', 'y_high' => 'garnish_y_hi',
  'x_high' => 'garnish_x_hi', 'y_velocity' => 'garnish_y_vel',
  'x_velocity' => 'garnish_x_vel', 'y_fraction' => 'garnish_y_subpixel',
  'x_fraction' => 'garnish_x_subpixel', 'countdown' => 'garnish_countdown',
  'source_sprite' => 'garnish_sprite', 'animation5' => 'garnish_anim_5',
  'floor' => 'garnish_floor', 'oam_flags' => 'garnish_oam_flags'
}.freeze
SCALARS = {
  'ACTIVE_MARKER_ADDRESS' => 'garnish_active',
  'SHARED_OVERWRITE_CURSOR_ADDRESS' => 'byte_7E0FF8'
}.freeze

def rust_constant(source, name)
  row = source.match(/^pub const #{Regexp.escape(name)}:\s*usize\s*=\s*(0x[0-9a-fA-F]+|\d+);$/)
  raise "missing/unparsed Rust #{name}" unless row

  Integer(row[1])
end

def compare_schema(variables, c_sprite, rust)
  c_names = variables.scan(/^#define (garnish_\w+) \(\(uint\d+\*/).flatten
  raise 'C Garnish pointer-field set/order changed' unless c_names == FIELDS.values

  body = rust[/\bbyte_fields!\s*\{(.*?)\}/m, 1]
  raise 'missing Rust Garnish schema' unless body

  rows = body.lines.reject { |line| line.strip.empty? }.map do |line|
    row = /^\s*(\w+),\s*(\w+)\s*=>\s*(0x[0-9a-fA-F]+);\s*$/.match(line)
    raise "unparsed Garnish schema row #{line.inspect}" unless row

    [row[1], row[3].to_i(16)]
  end
  raise 'Garnish field set/order/duplicates changed' unless rows.map(&:first) == FIELDS.keys

  rows.each do |field, address|
    name = FIELDS.fetch(field)
    row = variables.match(/^#define #{name} \(\(uint8\*\)\(g_ram\+(0x[0-9a-fA-F]+)\)\)\s*$/)
    raise "missing/unparsed C byte alias #{name}" unless row
    raise "Garnish address mismatch #{field}" unless address == row[1].to_i(16)
  end
  SCALARS.each do |constant, name|
    row = variables.match(/^#define #{name} \(\*\(uint8\*\)\(g_ram\+(0x[0-9a-fA-F]+)\)\)\s*$/)
    raise "missing/unparsed C scalar #{name}" unless row
    raise "Garnish scalar mismatch #{name}" unless rust_constant(rust, constant) == row[1].to_i(16)
  end
  upper = c_sprite.match(/void Garnish_ExecuteUpperSlots\(\).*?for \(int i = (\d+); i >= (\d+); i--\)/m)
  lower = c_sprite.match(/void Garnish_ExecuteLowerSlots\(\).*?for \(int i = (\d+); i >= (\d+); i--\)/m)
  raise 'unparsed C Garnish reverse traversals' unless upper && lower
  raise 'C Garnish traversal partition changed' unless lower[2].to_i.zero? && upper[2].to_i == lower[1].to_i + 1
  raise 'Garnish nominal slot count mismatch' unless rust_constant(rust, 'SLOT_COUNT') == upper[1].to_i + 1
  raise 'Garnish lower slot count mismatch' unless rust_constant(rust, 'LOWER_SLOT_COUNT') == lower[1].to_i + 1
end

begin
  compare_schema(variables, c_sprite, rust)
  mutations = [
    [variables.sub('garnish_type ((uint8*', 'garnish_type ((uint16*'), c_sprite, rust],
    [variables.sub('g_ram+0x1F800', 'g_ram+0x1F801'), c_sprite, rust],
    [variables.sub('garnish_active (*(uint8*', 'garnish_active (*(uint16*'), c_sprite, rust],
    [variables, c_sprite, rust.sub(/^\s*animation5,[^\n]*\n/, '')],
    [variables, c_sprite, rust.sub('ACTIVE_MARKER_ADDRESS: usize = 0xfb4', 'ACTIVE_MARKER_ADDRESS: usize = 0xfb5')],
    [variables, c_sprite, rust.sub('SLOT_COUNT: usize = 30', 'SLOT_COUNT: usize = 29')]
  ]
  mutations.each_with_index do |sources, index|
    raise "drift probe #{index} did not alter source" if sources == [variables, c_sprite, rust]
    rejected = false
    begin
      compare_schema(*sources)
    rescue StandardError
      rejected = true
    end
    raise "Garnish drift probe #{index} escaped detection" unless rejected
  end
rescue StandardError => error
  abort error.message
end
puts 'Garnish: all fourteen byte fields, shared scalars and C traversal bounds match; drift probes rejected'
