#!/bin/sh
set -eu

scripts/build-c-oracle.sh

first=$(mktemp "${TMPDIR:-/tmp}/zrust-oracle-first.XXXXXX")
second=$(mktemp "${TMPDIR:-/tmp}/zrust-oracle-second.XXXXXX")
trap 'rm -f "$first" "$second"' EXIT

build/c-oracle/smoke_oracle zelda3_assets.dat >"$first"
build/c-oracle/smoke_oracle zelda3_assets.dat >"$second"
cmp "$first" "$second"
cat "$first"
