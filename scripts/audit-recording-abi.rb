#!/usr/bin/env ruby
# The C recorder type is private to zelda_rtl.c. The test harness calls the
# original exported functions with an identical declaration, not Rust layout.
# Reject any source drift in that mirror, including field types and order.

def recorder_declaration(source)
  body = source[/typedef\s+struct\s+StateRecorder\s*\{(.*?)\}\s*StateRecorder\s*;/m, 1]
  raise 'missing StateRecorder declaration' unless body

  body.gsub(%r{/\*.*?\*/}m, '').gsub(%r{//[^\n]*}, '').gsub(/\s+/, '')
end

root = File.expand_path('..', __dir__)
reference = File.read(File.join(root, 'src/zelda_rtl.c'))
expected = recorder_declaration(reference)
%w[recorder_file_oracle.c recorder_replay_oracle.c].each do |name|
  mirror = File.read(File.join(root, 'oracle', name))
  abort "C recorder harness ABI declaration differs from original: #{name}" unless recorder_declaration(mirror) == expected
end

# Exercise the actual normalizer against changes that must not be overlooked.
probe = 'typedef struct StateRecorder { uint16 last_inputs; uint32 total_frames; } StateRecorder;'
same = probe.sub('uint16 last_inputs;', "uint16 /* retained */\nlast_inputs;")
raise 'comment/whitespace normalization failed' unless recorder_declaration(probe) == recorder_declaration(same)
[
  probe.sub('uint16', 'uint32'),
  probe.sub('last_inputs', 'other_inputs'),
  probe.sub('uint16 last_inputs; uint32 total_frames;', 'uint32 total_frames; uint16 last_inputs;'),
  probe.sub('uint32 total_frames;', ''),
  probe.sub('uint32 total_frames;', 'uint32 total_frames; uint8 extra;'),
].each do |changed|
  raise 'normalizer concealed a changed field/type/order' if recorder_declaration(changed) == recorder_declaration(probe)
end

puts 'C recorder harness declaration matches original field types, names and order; drift probes passed'
