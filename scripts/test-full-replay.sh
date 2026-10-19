#!/bin/sh
set -eu

oracle=${STREAM_ORACLE:-build/c-oracle/stream_oracle}
assets=${ZELDA3_ASSETS:-zelda3_assets.dat}
chapter=${1:-1}
last_chapter=${2:-13}
jobs=${Z3_REPLAY_JOBS:-1}

for endpoint in "$chapter" "$last_chapter"; do
  case "$endpoint" in
    1|2|3|4|5|6|7|8|9|10|11|12|13) ;;
    *) echo "usage: $0 [FIRST_CHAPTER [LAST_CHAPTER]] (1-13)" >&2; exit 2 ;;
  esac
done
if [ "$#" -gt 2 ] || [ "$chapter" -gt "$last_chapter" ]; then
  echo "usage: $0 [FIRST_CHAPTER [LAST_CHAPTER]] (1-13)" >&2
  exit 2
fi
case "$jobs" in
  1|2|3|4|5|6|7|8|9|10|11|12|13) ;;
  *) echo 'Z3_REPLAY_JOBS must be an integer from 1 to 13' >&2; exit 2 ;;
esac

if [ ! -x "$oracle" ]; then
  echo "missing C stream oracle: $oracle" >&2
  exit 1
fi
if [ ! -f "$assets" ]; then
  echo "missing compiled assets: $assets" >&2
  exit 1
fi

if [ "${Z3_REPLAY_EXECUTABLE+x}" = x ]; then
  executable=$Z3_REPLAY_EXECUTABLE
else
  cargo build -q --release -p z3-verify --bin check_full_replay
  executable=target/release/check_full_replay
fi
if [ ! -x "$executable" ]; then
  echo "missing replay executable: $executable" >&2
  exit 1
fi

run_range() (
  current=$1
  last=$2
  while [ "$current" -le "$last" ]; do
    "$executable" "$oracle" "$assets" "$current"
    current=$((current + 1))
  done
)

count=$((last_chapter - chapter + 1))
if [ "$jobs" -gt "$count" ]; then jobs=$count; fi
if [ "$jobs" -eq 1 ]; then
  run_range "$chapter" "$last_chapter"
else
  # Integer partitioning covers each chapter exactly once, including uneven
  # ranges. Wait for every worker and fail if any range fails; a later success
  # must never hide an earlier mismatch.
  worker=0
  workers=
  while [ "$worker" -lt "$jobs" ]; do
    first=$((chapter + worker * count / jobs))
    last=$((chapter + (worker + 1) * count / jobs - 1))
    run_range "$first" "$last" &
    workers="$workers $!"
    worker=$((worker + 1))
  done
  failed=0
  for pid in $workers; do
    if wait "$pid"; then :; else failed=1; fi
  done
  exit "$failed"
fi
