#!/usr/bin/env ruby
# Exercise the real POSIX runner's partitioning and failure propagation, without
# assets, compilation or game execution. Only its two external commands are
# replaced; argument validation, worker launch, loops and waits stay unchanged.
require 'open3'

runner = File.read(File.join(__dir__, 'test-full-replay.sh'))
build = 'cargo build -q --release -p z3-verify --bin check_full_replay'
check = 'target/release/check_full_replay "$oracle" "$assets" "$current"'
raise 'runner build seam changed' unless runner.scan(build).length == 1
raise 'runner checker seam changed' unless runner.scan(check).length == 1
runner = runner.sub(build, ':')
runner = runner.sub(check, <<~'SH'.strip)
  printf 'chapter:%s\n' "$current"
  test "$current" -ne "${Z3_TEST_FAIL_CHAPTER:-0}"
SH

def run(runner, jobs, args, fail_chapter = nil)
  env = {
    'Z3_REPLAY_JOBS' => jobs&.to_s,
    'Z3_TEST_FAIL_CHAPTER' => fail_chapter&.to_s,
    'STREAM_ORACLE' => '/usr/bin/true',
    'ZELDA3_ASSETS' => __FILE__
  }
  output, error, status = Open3.capture3(env, 'sh', '-s', '--', *args.map(&:to_s), stdin_data: runner)
  [output.scan(/^chapter:(\d+)$/).flatten.map(&:to_i).sort, error, status]
end

[
  [nil, [], (1..13).to_a],
  [3, [], (1..13).to_a],
  [13, [], (1..13).to_a],
  [3, [8, 11], (8..11).to_a],
  [13, [2, 4], (2..4).to_a],
  [13, [6, 6], [6]]
].each do |jobs, args, expected|
  chapters, error, status = run(runner, jobs, args)
  raise "coverage jobs=#{jobs} args=#{args}: #{chapters}, #{error}" unless status.success? && chapters == expected
end

chapters, error, status = run(runner, 3, [], 2)
expected = [1, 2, *(5..13)]
raise "failed worker hidden or other workers abandoned: #{chapters}, #{error}" unless status.exitstatus == 1 && chapters == expected

[[0, []], [14, []], ['no', []], [3, [8, 2]], [3, [0, 13]], [3, [1, 14]], [3, [1, 2, 3]]].each do |jobs, args|
  chapters, _error, status = run(runner, jobs, args)
  raise "invalid input accepted jobs=#{jobs} args=#{args}" unless status.exitstatus == 2 && chapters.empty?
end

puts 'replay runner: serial/parallel coverage, range validation and worker failure propagation pass'
