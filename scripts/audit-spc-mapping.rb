#!/usr/bin/env ruby
# Compare Rust's sole ordered mapping schemas with the unchanged C tables.
# A shared mistake in both Rust copy directions can pass a round-trip test;
# this independent source audit checks field identity, address, width and order.

root = File.expand_path('..', __dir__)
c_source = File.read(File.join(root, 'src/spc_player.c'))
rust_source = File.read(File.join(root, 'crates/z3-game/src/spc_player/ram_mapping.rs'))

def c_rows(source, table, owner, sized:)
  body = source[/static const \w+ #{table}\[\] = \{(.*?)\};/m, 1]
  abort "missing C table #{table}" unless body

  pattern = if sized
              /\{offsetof\(#{owner},\s*(\w+)\),\s*(0x[0-9a-fA-F]+),\s*(\d+)\}/
            else
              /\{offsetof\(#{owner},\s*(\w+)\),\s*(0x[0-9a-fA-F]+)\}/
            end
  rows = body.scan(pattern)
  abort "unparsed entries in C table #{table}" unless rows.length == body.scan(/offsetof\(/).length

  rows.map do |field, encoded, size|
    address = encoded.to_i(16)
    width = sized ? size.to_i : (address & 0x8000 == 0 ? 1 : 2)
    address &= 0x7fff unless sized
    [field.sub(/^_/, '').downcase, address, width]
  end
end

def rust_rows(source, schema)
  body = source[/#{schema}! \{(.*?)\n\}/m, 1]
  abort "missing Rust schema #{schema}" unless body

  rows = body.lines.reject { |line| line.strip.empty? }.map do |line|
    match = /^\s*(\w+):\s*(u8|u16|PhraseCursor|PatternCursor|PatternAddress|SfxCursor|SfxPosition|\[u8;\s*4\])\s*=>\s*(0x[0-9a-f]+);\s*$/.match(line)
    abort "unparsed Rust schema row #{line.inspect}" unless match

    field, type, address = match.captures
    width = { 'u8' => 1, 'u16' => 2, 'PhraseCursor' => 2, 'PatternCursor' => 2, 'PatternAddress' => 2, 'SfxCursor' => 2, 'SfxPosition' => 2, '[u8;4]' => 4 }.fetch(type.delete(' '))
    [field, address.to_i(16), width]
  end
  rows
end

channel_c = c_rows(c_source, 'kChannel_Maps', 'Channel', sized: false)
player_c = c_rows(c_source, 'kSpcPlayer_Maps', 'SpcPlayer', sized: true)
channel_rust = rust_rows(rust_source, 'channel_mapping')
player_rust = rust_rows(rust_source, 'player_mapping')

[['channel', channel_c, channel_rust], ['player', player_c, player_rust]].each do |name, expected, actual|
  next if expected == actual

  mismatch = (0...[expected.length, actual.length].max).find { |i| expected[i] != actual[i] }
  abort "#{name} mapping row #{mismatch}: C=#{expected[mismatch].inspect}, Rust=#{actual[mismatch].inspect}"
end

mapped_bytes = channel_c.sum { |_, _, width| width } * 8 + player_c.sum { |_, _, width| width }
puts "SPC mapping matches C: #{channel_c.length} channel fields × 8, #{player_c.length} player fields, #{mapped_bytes} RAM bytes"
