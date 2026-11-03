#!/usr/bin/env ruby
# Independent immutable-C constants and frozen malformed-upload reference.
# This is complementary to full-state C execution, not an equivalence proof
# for arbitrary live table inputs. No ROM or generated assets are read here.
require 'digest'

ROOT = File.expand_path('..', __dir__)
def source(path)
  File.read(File.join(ROOT, path))
end
def numbers(body)
  body.gsub(%r{//[^\n]*}, '').split(',').map(&:strip).reject(&:empty?).map do |value|
    raise "non-numeric array token #{value}" unless value.match?(/\A-?(?:0x[\da-fA-F]+|\d+)\z/)
    Integer(value)
  end
end
ARRAYS = [
  ['kEffectByteLength', 'EFFECT_BYTE_LENGTH', 27],
  ['kVolumeTable', 'PAN_CURVE', 22],
  ['kBaseNoteFreqs', 'BASE_NOTE_FREQS', 13],
  ['kEchoFirParameters', 'FIR', 32],
  ['kNoteVol', 'NOTE_VOLUME', 16],
  ['kNoteGateOffPct', 'NOTE_GATE', 8]
].freeze
def compare_arrays(c, rust)
  ARRAYS.each do |original, migrated, count|
    c_match = c.match(/static\s+const\s+\w+\s+#{original}\[(\d*)\]\s*=\s*\{(.*?)\};/m)
    c_body = c_match && c_match[2]
    r_body = rust[/const\s+#{migrated}:\s*\[\w+;\s*#{count}\]\s*=\s*\[(.*?)\];/m, 1]
    raise "missing static array #{original}/#{migrated}" unless c_body && r_body
    raise "C static dimension differs #{original}" unless c_match[1].empty? || c_match[1].to_i == count
    expected = numbers(c_body)
    raise "static array differs #{migrated}" unless expected.length == count && expected == numbers(r_body)
  end
end

c = source('src/spc_player.c')
data = source('crates/z3-game/src/spc_player/data.rs')
begin
  compare_arrays(c, data)
  [[c.sub('kVolumeTable[22]', 'kVolumeTable[21]'), data],
   [c, data.sub('PAN_CURVE: [u8; 22]', 'PAN_CURVE: [u8; 21]')],
   [c, data.sub('2143, 2270', '2144, 2270')]].each_with_index do |(original, migrated), index|
    raise "inert drift probe #{index}" if original == c && migrated == data
    rejected = false
    begin
      compare_arrays(original, migrated)
    rescue StandardError
      rejected = true
    end
    raise "escaped drift probe #{index}" unless rejected
  end
rescue StandardError => error
  abort error.message
end
puts 'SPC data: all six static arrays match unchanged C; drift probes rejected'

# All moved upload error handling is checked against the accepted parent,
# independently read and token-compared before pinning this function. Do not
# replace this reference with a call into production UploadBlocks.
frozen = source('crates/z3-game/src/spc_player/upload_baseline_tests.rs')
body = frozen[/    fn frozen_upload\([^;{]+\{.*?\n    \}/m]
abort 'missing frozen upload function' unless body
abort 'frozen upload imports production decoder' if body.match?(/UploadBlocks|next_block|\.apply\(/)
normalized = body.gsub(%r{//[^\n]*}, '').gsub(/\s+/, '')
expected = '199923db7c73432dbe19cd6366bd9de2b4c3a28fe2bda02a944eea43b4b6dae4'
abort 'frozen upload differs from independently reviewed dc325d5' unless Digest::SHA256.hexdigest(normalized) == expected
puts 'SPC malformed-upload reference matches independently audited dc325d5 function'

# Canonical table storage must remain live and private to the data/cursor
# owners. This catches a caller bypass, rather than accepting a parallel cache.
%w[spc_player.rs spc_player/sfx.rs].each do |path|
  production = source("crates/z3-game/src/#{path}").split("#[cfg(test)]\nmod tests", 2).first
  abort "raw SPC table access reintroduced in #{path}" if production.match?(/self\.ram\s*\[|\bread16\s*\(/)
end
abort 'SPC data owner must borrow canonical RAM' unless data.include?("ram: &'a [u8; 65536]")
puts 'SPC interpreter callers use live data/cursor owners, without direct RAM indexing'
