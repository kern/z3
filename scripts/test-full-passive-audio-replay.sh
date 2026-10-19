#!/bin/sh
set -eu

oracle=${STREAM_ORACLE:-build/c-oracle/stream_oracle}
assets=${ZELDA3_ASSETS:-zelda3_assets.dat}

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

chapter=1
while [ "$chapter" -le 13 ]; do
  Z3_VERIFY_AUDIO_ONLY=1 Z3_VERIFY_PASSIVE_AUDIO_STATE=1 \
    "$executable" "$oracle" "$assets" "$chapter"
  chapter=$((chapter + 1))
done
