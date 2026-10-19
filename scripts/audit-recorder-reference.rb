#!/usr/bin/env ruby
# Keep the malformed-input oracle independent of the implementation under test.
# Root independently compared this body with 02d83bb's runtime.rs recorder.
# Only the test type's name/visibility, comments and whitespace differ.
require 'digest'

root = File.expand_path('..', __dir__)
source = File.read(File.join(root, 'crates/z3-game/src/runtime/recorder/frozen.rs'))
marker = '#[derive(Clone, Debug, Default, PartialEq, Eq)]'
start = source.index(marker)
abort 'missing frozen recorder body' unless start

body = source[start..].gsub('FrozenRecorder', 'StateRecorder')
  .gsub('pub(super)', '').gsub(%r{//[^\n]*}, '').gsub(/\s+/, '')
expected = '5fe4c017df242b5052a8f25cf4650afc07a5cbf5c1e8b89d16a2af1a7ff89b88'
abort 'frozen recorder differs from independently audited 02d83bb implementation' unless Digest::SHA256.hexdigest(body) == expected

puts 'Frozen recorder body matches independently audited 02d83bb reference'
