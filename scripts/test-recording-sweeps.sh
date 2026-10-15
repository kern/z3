#!/bin/sh
set -eu

# Keep the original unobserved frame/pixel/PCM run and both observer contracts.
# Each sweep gets a distinct process/environment; wait for all of them even if
# an earlier one fails. Observation flags inherited from a caller must never
# silently turn the pixel run into an audio-only or live-export run.
env -u Z3_VERIFY_AUDIO_ONLY -u Z3_VERIFY_AUDIO_STATE -u Z3_VERIFY_PASSIVE_AUDIO_STATE \
  scripts/test-full-replay.sh &
normal_pid=$!
env -u Z3_VERIFY_AUDIO_ONLY -u Z3_VERIFY_PASSIVE_AUDIO_STATE Z3_VERIFY_AUDIO_STATE=1 \
  scripts/test-full-audio-replay.sh &
legacy_pid=$!
env -u Z3_VERIFY_AUDIO_ONLY -u Z3_VERIFY_AUDIO_STATE -u Z3_VERIFY_PASSIVE_AUDIO_STATE \
  scripts/test-full-passive-audio-replay.sh &
passive_pid=$!

failed=0
for pid in "$normal_pid" "$legacy_pid" "$passive_pid"; do
  if wait "$pid"; then :; else failed=1; fi
done
exit "$failed"
