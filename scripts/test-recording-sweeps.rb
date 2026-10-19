#!/usr/bin/env ruby
# Exercise real shell orchestration and scoped cleanup; mock only external
# build/copy/signature checks and sweep processes.
require 'open3'
require 'rbconfig'
require 'shellwords'
require 'tmpdir'
require 'timeout'

source = File.read(File.join(__dir__, 'test-recording-sweeps.sh'))
stage = <<~'RUBY'
  require 'fileutils'
  name, failure, directory, *arguments = ARGV
  File.open(File.join(directory, 'events'), 'a') { |f| f.puts(name) }
  exit 7 if failure == name
  case name
  when 'build'
    FileUtils.mkdir_p('target/release')
    File.write('target/release/check_full_replay', 'immutable verifier')
    File.chmod(0755, 'target/release/check_full_replay')
  when 'copy'
    FileUtils.cp('target/release/check_full_replay', arguments.fetch(0))
    exit 7 if failure == 'copy-partial'
  when 'signature-built', 'signature-copy'
    abort 'bad signature arguments' unless arguments[0, 2] == ['--verify', '--strict']
    abort 'signature checked missing binary' unless File.executable?(arguments.fetch(2))
  end
RUBY
worker = <<~'RUBY'
  require 'timeout'
  role, failure, directory = ARGV
  expected = role == 'legacy' ? '1' : nil
  abort 'incorrect legacy observer environment' unless ENV['Z3_VERIFY_AUDIO_STATE'] == expected
  abort 'passive environment leaked' unless ENV['Z3_VERIFY_PASSIVE_AUDIO_STATE'].nil?
  abort 'audio-only environment leaked' unless ENV['Z3_VERIFY_AUDIO_ONLY'].nil?
  events = File.readlines(File.join(directory, 'events'), chomp: true)
  expected_events = failure == 'linux' ? %w[build copy] : %w[build signature-built copy signature-copy]
  abort 'worker launched before complete preflight' unless events == expected_events
  executable = ENV.fetch('Z3_REPLAY_EXECUTABLE')
  abort 'ambient executable override leaked' if executable == '/poison/ambient-verifier'
  abort 'worker did not get private snapshot' unless executable.start_with?(directory + '/z3-replay-sweeps.')
  abort 'snapshot contents changed' unless File.binread(executable) == 'immutable verifier'
  abort 'snapshot not executable' unless File.executable?(executable)
  File.write(File.join(directory, "#{role}.ready"), executable)
  Timeout.timeout(3) do
    sleep 0.005 until %w[normal legacy passive].all? { |name| File.exist?(File.join(directory, "#{name}.ready")) }
  end
  sleep(failure == 'interrupt' ? 0.3 : role == 'normal' ? 0 : 0.05)
  abort 'snapshot removed before all children finished' unless File.exist?(executable)
  File.write(File.join(directory, "#{role}.done"), '')
  exit(role == failure ? 7 : 0)
RUBY

def replace_once(source, seam, replacement)
  raise "expected one seam: #{seam}" unless source.scan(seam).length == 1
  source.sub(seam) { replacement }
end

cases = %w[none linux interrupt normal legacy passive build copy copy-partial signature-built signature-copy]
cases.each do |failure|
  Dir.mktmpdir('z3 sweep test ') do |directory|
    command = ->(name) { Shellwords.join([RbConfig.ruby, '-e', stage, name, failure, directory]) }
    script = replace_once(source, 'cargo build -q --release -p z3-verify --bin check_full_replay', command.call('build'))
    script = script.gsub('$(uname -s)', failure == 'linux' ? 'Linux' : 'Darwin')
    script = replace_once(script, 'codesign --verify --strict target/release/check_full_replay', command.call('signature-built') + ' --verify --strict target/release/check_full_replay')
    script = replace_once(script, 'cp target/release/check_full_replay "$snapshot_executable"', command.call('copy') + ' "$snapshot_executable"')
    script = replace_once(script, 'codesign --verify --strict "$snapshot_executable"', command.call('signature-copy') + ' --verify --strict "$snapshot_executable"')
    %w[normal legacy passive].zip(%w[test-full-replay.sh test-full-audio-replay.sh test-full-passive-audio-replay.sh]).each do |role, name|
      script = replace_once(script, "scripts/#{name}", Shellwords.join([RbConfig.ruby, '-e', worker, role, failure, directory]))
    end
    env = {
      'Z3_VERIFY_AUDIO_ONLY' => '1', 'Z3_VERIFY_AUDIO_STATE' => '1',
      'Z3_VERIFY_PASSIVE_AUDIO_STATE' => '1', 'Z3_REPLAY_EXECUTABLE' => '/poison/ambient-verifier',
      'TMPDIR' => directory
    }
    output, status = if failure == 'interrupt'
      Open3.popen2e(env, '/bin/sh', '-c', script, chdir: directory) do |input, result, waiter|
        input.close
        Timeout.timeout(3) do
          sleep 0.005 until %w[normal legacy passive].all? { |role| File.exist?(File.join(directory, "#{role}.ready")) }
        end
        Process.kill('TERM', waiter.pid)
        [result.read, waiter.value]
      end
    else
      Open3.capture2e(env, '/bin/sh', '-c', script, chdir: directory)
    end
    raise "sweep status #{failure}: #{status.exitstatus}\n#{output}" unless status.success? == %w[none linux].include?(failure)
    launched = %w[none linux interrupt normal legacy passive].include?(failure)
    %w[normal legacy passive].each do |role|
      raise "child wait/abort #{role}, #{failure}\n#{output}" unless File.exist?(File.join(directory, "#{role}.done")) == launched
    end
    if launched
      paths = %w[normal legacy passive].map { |role| File.read(File.join(directory, "#{role}.ready")) }
      raise 'sweeps did not share one executable' unless paths.uniq.length == 1
      raise 'snapshot executable was not cleaned up' if File.exist?(paths.first)
    end
    raise 'snapshot directory was not cleaned up' unless Dir.glob(File.join(directory, 'z3-replay-sweeps.*')).empty?
    events = File.readlines(File.join(directory, 'events'), chomp: true)
    raise 'more than one build' unless events.count('build') == 1
    raise 'source executable was removed' if failure != 'build' && !File.exist?(File.join(directory, 'target/release/check_full_replay'))
  end
end
puts 'recording sweeps: single immutable build, preflight failures, isolated concurrent modes, all waits and scoped cleanup pass'
