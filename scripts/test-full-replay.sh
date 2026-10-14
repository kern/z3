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

cargo build -q --release -p z3-verify --bin check_full_replay

chapter=1
while [ "$chapter" -le 13 ]; do
  target/release/check_full_replay "$oracle" "$assets" "$chapter"
  chapter=$((chapter + 1))
done
