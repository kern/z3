#!/usr/bin/env ruby
# Independently compare the live Rust swimming schema with the unchanged C
# pointer aliases. Behavioral tests separately cover stride, byte aliases and
# reset/publication order; a schema audit alone cannot establish those.
require 'digest'

root = File.expand_path('..', __dir__)
c_source = File.read(File.join(root, 'src/variables.h'))
rust_source = File.read(File.join(root, 'crates/z3-game/src/swim_state.rs'))

FAMILIES = {
  'acceleration_latch' => 'swimcoll_var1',
  'hold_timer' => 'swimcoll_var3',
  'acceleration_mode' => 'swimcoll_var5',
  'speed' => 'swimcoll_var7',
  'speed_limit' => 'swimcoll_var9',
  'direction_selector' => 'swimcoll_var11'
}.freeze

def compare_schema(c_source, rust_source)
  declarations = c_source.lines.select { |line| line.match?(/^#define swimcoll_var\d+\b/) }
  c_rows = declarations.map do |line|
    match = /^#define (swimcoll_var\d+) \(\(uint16\*\)\(g_ram\+(0x[0-9a-fA-F]+)\)\)\s*$/.match(line)
    raise "unparsed C swimming alias: #{line.inspect}" unless match

    [match[1], match[2].to_i(16)]
  end
  raise 'C swimming aliases changed or duplicated' unless c_rows.map(&:first).sort == FAMILIES.values.sort

  body = rust_source[/\bfields!\s*\{(.*?)\}/m, 1]
  raise 'missing Rust swimming schema' unless body

  rust_rows = body.lines.reject { |line| line.strip.empty? }.map do |line|
    match = /^\s*(\w+),\s*(\w+),\s*(\w+)\s*=>\s*(0x[0-9a-fA-F]+);\s*$/.match(line)
    raise "unparsed Rust swimming row: #{line.inspect}" unless match

    [match[1], match[4].to_i(16)]
  end
  raise 'Rust swimming fields changed or duplicated' unless rust_rows.map(&:first).sort == FAMILIES.keys.sort

  c_addresses = c_rows.to_h
  rust_rows.each do |field, address|
    expected = c_addresses.fetch(FAMILIES.fetch(field))
    raise "#{field}: C=#{expected.to_s(16)}, Rust=#{address.to_s(16)}" unless address == expected
  end
end

begin
  compare_schema(c_source, rust_source)
  # Exercise rejection paths against the actual schemas, not alternate fixtures
  # that could accidentally validate an obsolete layout.
  mutations = [
    [c_source.sub('swimcoll_var1 ((uint16*', 'swimcoll_var1 ((uint8*'), rust_source],
    [c_source.sub('g_ram+0x32F', 'g_ram+0x330'), rust_source],
    [c_source + "\n#define swimcoll_var13 ((uint16*)(g_ram+0x340))\n", rust_source],
    [c_source, rust_source.sub(/acceleration_latch,([^\n]*=>\s*)0x32f/i, 'acceleration_latch,\1 0x330')],
    [c_source, rust_source.sub(/^\s*hold_timer,[^\n]*\n/, '')]
  ]
  mutations.each_with_index do |(changed_c, changed_rust), index|
    raise "mutation #{index} did not change the source" if changed_c == c_source && changed_rust == rust_source

    rejected = false
    begin
      compare_schema(changed_c, changed_rust)
    rescue StandardError
      rejected = true
    end
    raise "schema mutation #{index} escaped detection" unless rejected
  end
rescue StandardError => error
  abort error.message
end

puts 'Swimming schema matches all six original C uint16 pointer aliases; drift probes rejected'

# Root independently compared all twelve methods with player.rs at 2dc8f79.
# Pin the test reference without depending on Git history at test time. These
# particular methods contain no brace-bearing strings; only comments and
# whitespace are normalized, never expressions, types or function names.
frozen = File.read(File.join(root, 'crates/z3-game/src/swim_state/frozen.rs'))
names = %w[
  link_flag_max_accels link_set_ice_max_accel link_set_momentum
  link_set_the_max_accel link_handle_swim_accels handle_swim_stroke_and_subpixels
  link_reset_swimming_state reset_all_acceleration
  player_something_with_velocity_tired_or_swim save_safe_return_coordinates
  link_handle_velocity_and_sand_drag link_apply_moving_floor_velocity
]
abort 'frozen swimming method set/order changed' unless frozen.scan(/^    (?:pub )?fn (\w+)/).flatten == names

methods = names.map do |name|
  start = frozen.match(/^    (?:pub )?fn #{Regexp.escape(name)}\b/).begin(0)
  opening = frozen.index('{', start)
  depth = 1
  ending = opening + 1
  while depth.positive?
    character = frozen[ending]
    abort "unclosed frozen method #{name}" unless character

    depth += 1 if character == '{'
    depth -= 1 if character == '}'
    ending += 1
  end
  frozen[start...ending].gsub(%r{//[^\n]*}, '').gsub(/\s+/, '')
end
expected = '41fde8d33e453128f1c6ba4f76986cb5da6185f48fff50c951d31cc23b84a7f3'
abort 'frozen swimming methods differ from independently audited 2dc8f79 reference' unless Digest::SHA256.hexdigest(methods.join) == expected
abort 'frozen swimming table must not use the production constant' if frozen.match?(/use[^;]*SWIMMING_TAB2/m)
abort 'frozen swimming table changed' unless frozen.match?(/const SWIMMING_TAB2:\s*\[u8;\s*2\]\s*=\s*\[32,\s*8\];/)

puts 'Frozen swimming methods match independently audited 2dc8f79 reference'
