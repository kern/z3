#!/usr/bin/env ruby
# Exercise the real orchestrator; replace only its three external sweep calls.
require 'open3'
require 'rbconfig'
require 'shellwords'
require 'tmpdir'

source = File.read(File.join(__dir__, 'test-recording-sweeps.sh'))
worker = <<~'RUBY'
  require 'timeout'
  role, failure, directory = ARGV
  expected = role == 'legacy' ? '1' : nil
  abort 'incorrect legacy observer environment' unless ENV['Z3_VERIFY_AUDIO_STATE'] == expected
  abort 'passive environment leaked' unless ENV['Z3_VERIFY_PASSIVE_AUDIO_STATE'].nil?
  abort 'audio-only environment leaked into pixel run' if role == 'normal' && ENV['Z3_VERIFY_AUDIO_ONLY']
  File.write(File.join(directory, "#{role}.ready"), '')
  Timeout.timeout(3) do
    sleep 0.005 until %w[normal legacy passive].all? { |name| File.exist?(File.join(directory, "#{name}.ready")) }
  end
  # Make the first wait fail before the other workers finish, testing that a
  # failure cannot abandon children or be hidden by a later successful wait.
  sleep(role == 'normal' ? 0 : 0.05)
  File.write(File.join(directory, "#{role}.done"), '')
  exit(role == failure ? 7 : 0)
RUBY

['none', 'normal', 'legacy', 'passive'].each do |failure|
  Dir.mktmpdir('z3-sweep-test-') do |directory|
    script = source.dup
    %w[normal legacy passive].zip(%w[test-full-replay.sh test-full-audio-replay.sh test-full-passive-audio-replay.sh]).each do |role, name|
      seam = "scripts/#{name}"
      abort "expected exactly one #{seam}" unless script.scan(seam).length == 1
      command = Shellwords.join([RbConfig.ruby, '-e', worker, role, failure, directory])
      script = script.sub(seam) { command }
    end
    # Deliberately poison ambient modes: the orchestrator must isolate them.
    env = { 'Z3_VERIFY_AUDIO_ONLY' => '1', 'Z3_VERIFY_AUDIO_STATE' => '1', 'Z3_VERIFY_PASSIVE_AUDIO_STATE' => '1' }
    output, status = Open3.capture2e(env, '/bin/sh', '-c', script)
    expected = failure == 'none'
    abort "sweep status for #{failure}: #{status.exitstatus}\n#{output}" unless status.success? == expected
    %w[normal legacy passive].each do |role|
      abort "#{role} was not waited for after #{failure}\n#{output}" unless File.exist?(File.join(directory, "#{role}.done"))
    end
  end
end
puts 'recording sweeps: concurrent launch, isolated modes, all failures and child waits passed'
