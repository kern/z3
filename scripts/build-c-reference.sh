#!/bin/sh
set -eu

expected_rom_sha256=66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb
rom_path=${1:-zelda3.sfc}

if [ ! -f "$rom_path" ]; then
  echo "Missing ROM: $rom_path" >&2
  exit 1
fi

actual_rom_sha256=$(shasum -a 256 "$rom_path" | awk '{print $1}')
if [ "$actual_rom_sha256" != "$expected_rom_sha256" ]; then
  echo "ROM hash mismatch" >&2
  echo "expected: $expected_rom_sha256" >&2
  echo "actual:   $actual_rom_sha256" >&2
  exit 1
fi

if [ ! -x .venv/bin/python ]; then
  echo "Missing .venv. Run: python3 -m venv .venv" >&2
  echo "Then run: .venv/bin/python -m pip install -r requirements.txt" >&2
  exit 1
fi

make -j8 \
  PYTHON=.venv/bin/python \
  CC=clang \
  CFLAGS='-O2 -Werror -Wno-deprecated-non-prototype -I . -I/opt/homebrew/include/SDL2 -D_THREAD_SAFE -DSYSTEM_VOLUME_MIXER_AVAILABLE=0'
