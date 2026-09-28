#!/bin/sh
set -eu

frames=${1:-2000}
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

cargo build -q -p z3-verify --bin check_integrated_frame

found=0
for save in saves/ref/*.sav; do
  if [ ! -f "$save" ]; then
    continue
  fi
  found=$((found + 1))
  target/debug/check_integrated_frame "$oracle" "$assets" --save "$save" "$frames" 1
done

if [ "$found" -ne 13 ]; then
  echo "expected 13 reference saves, found $found" >&2
  exit 1
fi

echo "all 13 reference saves match the C oracle through $frames frame(s)"
