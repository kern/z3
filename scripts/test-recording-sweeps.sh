#!/bin/sh
set -eu

# Build before launching any consumers, then run an immutable private copy.
# In particular, no child cargo invocation may replace the executable while
# another sweep is using it. Never trust an ambient executable override here.
cargo build -q --release -p z3-verify --bin check_full_replay
if [ "$(uname -s)" = Darwin ]; then
  codesign --verify --strict target/release/check_full_replay
fi
snapshot_directory=$(mktemp -d "${TMPDIR:-/tmp}/z3-replay-sweeps.XXXXXX")
snapshot_executable=$snapshot_directory/check_full_replay
cleanup_snapshot() {
  rm -f "$snapshot_executable"
  rmdir "$snapshot_directory"
}
finish_interrupted() {
  # Do not unlink a running child's verifier. Cancellation waits for already
  # launched sweeps rather than leaving orphaned workers using a removed path.
  trap '' 1 2 15
  for pid in ${normal_pid-} ${legacy_pid-} ${passive_pid-}; do
    if wait "$pid"; then :; else :; fi
  done
  exit 1
}
trap cleanup_snapshot 0
trap finish_interrupted 1 2 15
cp target/release/check_full_replay "$snapshot_executable"
if [ ! -x "$snapshot_executable" ]; then
  echo "missing executable snapshot: $snapshot_executable" >&2
  exit 1
fi
if [ "$(uname -s)" = Darwin ]; then
  codesign --verify --strict "$snapshot_executable"
fi

# Keep the original unobserved frame/pixel/PCM run and both observer contracts.
# Each sweep gets a distinct process/environment; wait for all of them even if
# an earlier one fails. Observation flags inherited from a caller must never
# silently turn the pixel run into an audio-only or live-export run.
env -u Z3_VERIFY_AUDIO_ONLY -u Z3_VERIFY_AUDIO_STATE -u Z3_VERIFY_PASSIVE_AUDIO_STATE \
  Z3_REPLAY_EXECUTABLE="$snapshot_executable" \
  scripts/test-full-replay.sh &
normal_pid=$!
env -u Z3_VERIFY_AUDIO_ONLY -u Z3_VERIFY_PASSIVE_AUDIO_STATE Z3_VERIFY_AUDIO_STATE=1 \
  Z3_REPLAY_EXECUTABLE="$snapshot_executable" \
  scripts/test-full-audio-replay.sh &
legacy_pid=$!
env -u Z3_VERIFY_AUDIO_ONLY -u Z3_VERIFY_AUDIO_STATE -u Z3_VERIFY_PASSIVE_AUDIO_STATE \
  Z3_REPLAY_EXECUTABLE="$snapshot_executable" \
  scripts/test-full-passive-audio-replay.sh &
passive_pid=$!

failed=0
for pid in "$normal_pid" "$legacy_pid" "$passive_pid"; do
  if wait "$pid"; then :; else failed=1; fi
done
exit "$failed"
