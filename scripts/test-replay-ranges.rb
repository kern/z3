#!/usr/bin/env ruby
# Exercise actual shell sources, mocking only build/verifier commands.
require 'open3'

BUILD = 'cargo build -q --release -p z3-verify --bin check_full_replay'
def runner_source(name, counter)
  runner = File.read(File.join(__dir__, name))
  raise 'runner build seam changed' unless runner.scan(BUILD).length == 1
  check = '"$executable" "$oracle" "$assets" "$' + counter + '"'
  raise 'runner checker seam changed' unless runner.scan(check).length == 1
  runner = runner.sub(BUILD, 'printf "build\n"')
  runner = runner.sub('executable=target/release/check_full_replay', 'executable=/usr/bin/true')
  runner.sub(check, <<~SH.strip)
    /bin/sh -c 'printf "chapter:%s executable:%s audio:%s passive:%s legacy:%s\\n" "$1" "$2" "${Z3_VERIFY_AUDIO_ONLY-}" "${Z3_VERIFY_PASSIVE_AUDIO_STATE-}" "${Z3_VERIFY_AUDIO_STATE-}"; test "$1" -ne "${Z3_TEST_FAIL_CHAPTER:-0}"' verifier "$#{counter}" "$executable"
  SH
end

def run(runner, jobs, args, fail_chapter = nil, executable = nil)
  env = {
    'Z3_REPLAY_JOBS' => jobs&.to_s, 'Z3_TEST_FAIL_CHAPTER' => fail_chapter&.to_s,
    'STREAM_ORACLE' => '/usr/bin/true', 'ZELDA3_ASSETS' => __FILE__,
    'Z3_REPLAY_EXECUTABLE' => executable,
    'Z3_VERIFY_AUDIO_ONLY' => nil, 'Z3_VERIFY_PASSIVE_AUDIO_STATE' => nil,
    'Z3_VERIFY_AUDIO_STATE' => '1'
  }
  output, error, status = Open3.capture3(env, 'sh', '-s', '--', *args.map(&:to_s), stdin_data: runner)
  [output, error, status]
end

runner = runner_source('test-full-replay.sh', 'current')
coverage = [[nil, [], (1..13).to_a], [3, [], (1..13).to_a], [13, [], (1..13).to_a],
            [3, [8, 11], (8..11).to_a], [13, [2, 4], (2..4).to_a], [13, [6, 6], [6]]]
[nil, '/usr/bin/false'].each do |executable|
  coverage.each do |jobs, args, expected|
    output, error, status = run(runner, jobs, args, nil, executable)
    chapters = output.scan(/^chapter:(\d+)/).flatten.map(&:to_i).sort
    raise "coverage jobs=#{jobs} args=#{args}: #{chapters}, #{error}" unless status.success? && chapters == expected
    raise 'wrong build choice' unless output.scan(/^build$/).length == (executable.nil? ? 1 : 0)
    raise 'wrong verifier selected' unless output.scan(/executable:(\S+)/).flatten.uniq == [executable || '/usr/bin/true']
  end
  output, error, status = run(runner, 3, [], 2, executable)
  chapters = output.scan(/^chapter:(\d+)/).flatten.map(&:to_i).sort
  raise "failed worker hidden/abandoned: #{chapters}, #{error}" unless status.exitstatus == 1 && chapters == [1, 2, *(5..13)]
end

[[0, []], [14, []], ['no', []], [3, [8, 2]], [3, [0, 13]], [3, [1, 14]], [3, [1, 2, 3]]].each do |jobs, args|
  output, _error, status = run(runner, jobs, args)
  raise "invalid input accepted #{jobs} #{args}" unless status.exitstatus == 2 && output.empty?
end

%w[test-full-replay.sh test-full-audio-replay.sh test-full-passive-audio-replay.sh].each do |name|
  source = runner_source(name, name == 'test-full-replay.sh' ? 'current' : 'chapter')
  ['', '/missing/prebuilt-verifier'].each do |bad_override|
    output, _error, status = run(source, 1, [], nil, bad_override)
    raise "bad override built/launched #{name}" unless status.exitstatus == 1 && output.empty?
  end
  next if name == 'test-full-replay.sh'
  [nil, '/usr/bin/false'].each do |executable|
    output, error, status = run(source, nil, [], nil, executable)
    raise "audio sweep failed #{name}: #{error}" unless status.success?
    raise 'audio coverage changed' unless output.scan(/^chapter:(\d+)/).flatten.map(&:to_i) == (1..13).to_a
    raise 'audio build choice changed' unless output.scan(/^build$/).length == (executable.nil? ? 1 : 0)
    raise 'audio override ignored' unless output.scan(/executable:(\S+)/).flatten.uniq == [executable || '/usr/bin/true']
    expected_passive = name.include?('passive') ? '1' : ''
    raise 'audio modes changed' unless output.scan(/audio:(\S*) passive:(\S*) legacy:(\S*)/).uniq == [['1', expected_passive, '1']]
  end
end
puts 'replay runners: standalone/prebuilt choice, all modes, serial/parallel coverage, invalid overrides and worker waits pass'
