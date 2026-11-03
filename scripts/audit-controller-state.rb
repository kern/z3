#!/usr/bin/env ruby
# Closed live-schema comparison against unchanged C WRAM aliases. Boss rows
# are explicit base+index interpretations, not independent storage regions.
require 'digest'
ROOT = File.expand_path('..', __dir__)
CONTROLLERS = {
  'kind' => 'overlord_type', 'x_low' => 'overlord_x_lo',
  'x_high' => 'overlord_x_hi', 'y_low' => 'overlord_y_lo',
  'y_high' => 'overlord_y_hi', 'scratch_a' => 'overlord_gen1',
  'timer' => 'overlord_gen2', 'scratch_c' => 'overlord_gen3',
  'floor' => 'overlord_floor', 'spawned_area' => 'overlord_spawned_in_area'
}.freeze
INDEXED = {
  'orbit_x_low' => ['overlord_x_hi', 0], 'orbit_x_high' => ['overlord_y_hi', 0],
  'orbit_y_low' => ['overlord_gen2', 0], 'orbit_y_high' => ['overlord_floor', 0],
  'arrghi_x_low' => ['overlord_x_lo', 0], 'arrghi_x_high' => ['overlord_y_lo', 0],
  'arrghi_y_low' => ['overlord_gen1', 0], 'arrghi_y_high' => ['overlord_gen3', 0],
  'helmasaur_leg_frame' => ['overlord_x_lo', 0],
  'helmasaur_tail_x' => ['overlord_x_lo', 5], 'helmasaur_tail_y' => ['overlord_y_lo', 5],
  'trinexx_control' => ['overlord_x_lo', 0]
}.freeze
BYTES = {
  'orbit_phase_low' => ['overlord_x_lo', 0], 'orbit_phase_high' => ['overlord_x_lo', 1],
  'orbit_radius' => ['overlord_x_lo', 2], 'orbit_count' => ['overlord_x_lo', 3],
  'orbit_speed' => ['overlord_x_lo', 4], 'helmasaur_animation_clock' => ['overlord_x_lo', 4],
  'helmasaur_phase_low' => ['overlord_gen1', 5], 'helmasaur_turn' => ['overlord_gen1', 6],
  'helmasaur_radius' => ['overlord_gen1', 7], 'helmasaur_swing_state' => ['overlord_gen2', 0],
  'helmasaur_velocity_low' => ['overlord_gen2', 1], 'helmasaur_velocity_high' => ['overlord_gen2', 2],
  'helmasaur_next_tail' => ['overlord_gen2', 3], 'trinexx_target_x_low' => ['overlord_x_lo', 0],
  'trinexx_target_y_low' => ['overlord_x_lo', 1], 'trinexx_bend' => ['overlord_x_lo', 2],
  'trinexx_direction' => ['overlord_x_lo', 3], 'trinexx_clock' => ['overlord_x_lo', 4],
  'trinexx_delay' => ['overlord_x_lo', 5], 'trinexx_startup_delay' => ['overlord_x_lo', 6],
  'trinexx_tail_phase_low' => ['overlord_x_lo', 7], 'trinexx_ready' => ['overlord_x_hi', 0]
}.freeze
WORDS = {
  'orbit_phase' => ['overlord_x_lo', 0], 'helmasaur_phase' => ['overlord_gen1', 5],
  'helmasaur_velocity' => ['overlord_gen2', 1], 'trinexx_tail_phase' => ['overlord_x_lo', 7]
}.freeze

def c_address(variables, name, type)
  row = variables.match(/^#define #{name} \(\(#{type}\*\)\(g_ram\+(0x[\da-fA-F]+)\)\)\s*$/)
  raise "missing/wrong-width C alias #{name}" unless row
  Integer(row[1])
end
def schema(source, macro_name, expected, prefix)
  body = source[/^#{macro_name}!\s*\{(.*?)\n\}/m, 1]
  raise "missing schema #{macro_name}" unless body
  rows = body.lines.reject { |line| line.strip.empty? }.map do |line|
    row = /^\s*(\w+),\s*(\w+),\s*(\w+),\s*(\w+)\s*=>\s*(0x[\da-fA-F]+);\s*$/.match(line)
    raise "unparsed schema row #{line.inspect}" unless row
    name, setter, forward, forward_setter, address = row.captures
    raise "schema forwarding changed #{name}" unless [setter, forward, forward_setter] == ["set_#{name}", "#{prefix}_#{name}", "set_#{prefix}_#{name}"]
    [name, Integer(address)]
  end
  raise "schema field set/order changed #{macro_name}" unless rows.map(&:first) == expected.keys
  raise "schema address differs #{macro_name}" unless rows == expected.to_a
end
def compare(variables, c_overlord, c_sprite, c_boss, rust)
  c_names = variables.scan(/^#define (overlord_\w+) \(\(uint\d+\*/).flatten
  expected_names = CONTROLLERS.values.dup.insert(9, 'overlord_offset_sprite_pos')
  raise 'closed C controller alias set changed' unless c_names == expected_names
  schema(rust, 'controller_fields', CONTROLLERS.transform_values { |name| c_address(variables, name, 'uint8') }, 'controller')
  [['boss_indexed_fields', INDEXED], ['boss_byte_fields', BYTES], ['boss_word_fields', WORDS]].each do |macro_name, fields|
    addresses = fields.transform_values { |name, index| c_address(variables, name, 'uint8') + index }
    schema(rust, macro_name, addresses, 'boss')
  end
  WORDS.each_value do |name, index|
    raise "missing original word alias #{name}[#{index}]" unless (c_overlord + c_boss).include?("WORD(#{name}[#{index}])")
  end
  address = c_address(variables, 'overlord_offset_sprite_pos', 'uint16')
  source_words = rust.scan(/let address = (0x[\da-fA-F]+) \+ self\.index \* (\d+);/)
  raise 'controller source-position address/word stride changed' unless source_words == [[format('0x%x', address), '2']] * 2
  bound = c_overlord[/void Overlord_ExecuteAll\(\).*?for \(int i = (\d+); i >= 0; i--\)/m, 1]
  slots = rust[/pub const SLOT_COUNT: usize = (\d+);/, 1]
  raise 'controller nominal traversal count changed' unless bound && slots && bound.to_i + 1 == slots.to_i
  room = c_sprite[/bool Sprite_CheckIfOverlordsClear\(\).*?\n\}/m]
  raise 'original room-clear predicate changed' unless room && room.include?('overlord_type[i] == 0x14 || overlord_type[i] == 0x18')
  raise 'Rust room-clear predicate changed' unless rust.include?('!matches!(self.slot(i).kind(), 20 | 24)')
end

sources = ['src/variables.h', 'src/overlord.c', 'src/sprite.c', 'src/sprite_main.c',
           'crates/z3-game/src/overlord_state.rs'].map { |path| File.read(File.join(ROOT, path)) }
begin
  compare(*sources)
  probes = [
    [0, 'overlord_type ((uint8*', 'overlord_type ((uint16*'],
    [4, '=> 0xb00;', '=> 0xb01;'],
    [4, 'SLOT_COUNT: usize = 8', 'SLOT_COUNT: usize = 16'],
    [4, '0xb48 + self.index * 2', '0xb48 + self.index * 1'],
    [4, 'set_boss_helmasaur_phase => 0xb2d', 'set_boss_helmasaur_phase => 0xb2e'],
    [4, '20 | 24', '20 | 23']
  ]
  probes.each_with_index do |(which, before, after), index|
    mutated = sources.dup
    mutated[which] = mutated[which].sub(before, after)
    raise "inert controller drift probe #{index}" if mutated == sources
    rejected = false
    begin
      compare(*mutated)
    rescue StandardError
      rejected = true
    end
    raise "escaped controller drift probe #{index}" unless rejected
  end
rescue StandardError => error
  abort error.message
end
puts 'Controller/boss schemas: 10 controller lanes, source words, 38 live boss aliases and nominal C traversal/room predicates match; drift probes rejected'

# Independently compared verbatim with the accepted dc325d5 bank2a functions,
# including all static local lookup bytes. Keep the reference raw and separate
# from the new domain, including when mutable callback fixtures use it.
frozen = File.read(File.join(ROOT, 'crates/z3-game/src/overlord_state/frozen_boss.rs'))
names = %w[helmasaur_king_initialize helmasaur_king_reinitialize helmasaur_king_swing_tail trinexx_wag_tail]
abort 'frozen boss helper set/order changed' unless frozen.scan(/pub fn (\w+)\(/).flatten == names
abort 'frozen boss imports new workspace' if frozen.match?(/boss_workspace|boss_helmasaur|boss_trinexx|controller_mut/)
blocks = names.map do |name|
  match = frozen.match(/pub fn #{name}\(/)
  start = match.begin(0)
  finish = frozen.index('{', match.end(0)) + 1
  depth = 1
  while depth.positive?
    character = frozen[finish]
    abort "unterminated frozen helper #{name}" unless character
    depth += 1 if character == '{'
    depth -= 1 if character == '}'
    finish += 1
  end
  frozen[start...finish].gsub(%r{//[^\n]*}, '').gsub(/\s+/, '')
end
expected = '4f58eb7f3e25a6bf1bd825a1e00613d3a02c3bae24cf4d459693c8e46a99328e'
abort 'frozen boss helpers differ from independently audited dc325d5' unless Digest::SHA256.hexdigest(blocks.join) == expected
constants = frozen.scan(/^const (\w+):\s*usize\s*=\s*(0x[\da-fA-F]+);/).map { |name, address| [name, Integer(address)] }
abort 'frozen boss reference constants changed' unless constants == [
  ['OVERLORD_X_LO', 0xb08], ['OVERLORD_GEN1', 0xb28], ['OVERLORD_GEN2', 0xb30],
  ['FRAME_COUNTER', 0x1a], ['SOUND_EFFECT_2', 0x12f]
]
puts 'Four frozen boss helpers and five constants match independently audited dc325d5'
