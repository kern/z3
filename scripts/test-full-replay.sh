#!/bin/sh
set -eu

oracle=${STREAM_ORACLE:-build/c-oracle/stream_oracle}
assets=${ZELDA3_ASSETS:-zelda3_assets.dat}
chapter=${1:-1}
last_chapter=${2:-13}

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

if [ ! -x "$oracle" ]; then
  echo "missing C stream oracle: $oracle" >&2
  exit 1
fi
if [ ! -f "$assets" ]; then
  echo "missing compiled assets: $assets" >&2
  exit 1
fi

cargo build -q --release -p z3-verify --bin check_full_replay

while [ "$chapter" -le "$last_chapter" ]; do
  target/release/check_full_replay "$oracle" "$assets" "$chapter"
  chapter=$((chapter + 1))
done
