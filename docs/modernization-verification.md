# Modernization batch 1 verification

Date: 2026-10-14. Pre-change source baseline: `e424757`.
Scope and remaining work: [modernization inventory](rust-modernization-plan.md#implementation-progress).

## Reference configuration

- C gameplay and SNES device sources are unchanged. The new movement harness
  calls the existing compiled `Ancilla_MoveX/Y/Z` and `Sprite_MoveX/Y/Z` functions;
  it does not supply alternate movement math.
- Apple Clang 21.0.0 (`clang-2100.1.1.101`), `arm64-apple-darwin25.5.0`.
- C compilation and dylib link order remain defined by
  [`build-c-reference.sh`](../scripts/build-c-reference.sh) and
  [`build-c-oracle.sh`](../scripts/build-c-oracle.sh): `-O2 -Werror`, the existing
  prototype warning suppression, SDL includes/defines and system-volume policy.
  The oracle wrapper additionally uses `-fPIC`; SDL2 and libm remain linked.
- The existing 165-entry asset schema/signature, ROM identity check and all 13
  reference recordings are unchanged. Assets, ROMs, saves and generated traces
  remain local/ignored, not documentation fixtures.
- Combined replay uses the production integrated runtime and existing default
  `RENDER_NEW | RENDER_MODE7_4X` pixel comparison, including dynamic frame size.
  State bytes, complete rendered buffers and stereo PCM values are compared
  exactly at every recorded frame. No verifier masks or tolerances were added.

## Results

| Gate | Result |
| --- | --- |
| C build and repeated deterministic smoke | Passed |
| Formatting / strict workspace Clippy / whitespace check | Passed |
| Workspace tests | 542 passed, including 429 gameplay tests |
| C entrypoint coverage audit | 3,366 / 3,366 accounted for |
| Integrated startup differential check | 2,000 exact frames (additional smoke, not the recording gate) |
| Feature/aspect-ratio matrix | All existing scenarios passed, including 4× Mode 7 map pixels |
| Device and subsystem C differentials | CPU, SPC, music player, poly, player OAM helpers/main, tiles, overlords, followers, file selection, HUD, misc and overworld passed |
| New movement differential | 3,492 complete-WRAM comparisons passed |
| Full recordings, state + pixels + PCM | All 13 recordings passed: 1,073,092 frames, every pixel and sample exact |
| Full recordings, audio queue + mapped SPC/DSP diagnostics | All 13 recordings passed: 1,073,092 frames |
| Native software host smoke | 600 frames passed with dummy SDL video/audio |

The movement vectors cover both entity families, all three axes, first/last
slots, every signed 8-bit velocity bit pattern, and explicit boundary/carry/
borrow combinations. Nonuniform initial WRAM makes unrelated writes visible.
The harness streams one initial/final image at a time and needs no game assets.

## Commands and execution notes

`scripts/test-parity.sh` passed its build, audit, workspace, lint, feature and
component stages through ROM inspection. Its serial recording stage was stopped
deliberately and restarted as the following **disjoint, complete** ranges using
the same release build/configuration:

```sh
scripts/test-full-replay.sh 1 4
scripts/test-full-replay.sh 5 9
scripts/test-full-replay.sh 10 13
Z3_VERIFY_AUDIO_STATE=1 scripts/test-full-audio-replay.sh
```

The final host stage was run separately:

```sh
cargo build -q -p z3-native --bin z3
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  target/debug/z3 --config config/parity.ini --frames 600 zelda3.sfc
```

The new movement gate is wired into `scripts/test-parity.sh` for future runs:

```sh
cargo run -q -p z3-verify --bin check_positions -- build/c-oracle/position_oracle
```

The audio-state diagnostic exports mapped SPC variables to RAM, as before. It
is supplementary; it does not replace the combined sweep without those exports.
The GPU renderer/shader code is unchanged in this batch. These checks certify
the existing pre-presentation PPU pixel contract, not a new post-shader proof.
