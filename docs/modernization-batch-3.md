# Modernization batch 3: sprite operations, maps and graphics borrowing

Parent baseline: `c224740` (batch 2, pushed to `kern/main`). This is an
incremental receipt, not completion of the M01–M24 backlog.

## Changes

- **Sprite arithmetic and operations (M02/M04/M09):** `VelocityByte` names
  wrapping byte negation, arithmetic halving and approach-by-wrapped-difference.
  Twelve signed-byte unary negations no longer panic on `0x80` in debug builds.
  The logically shifted Z rebound remains distinct. Four ordered countdowns,
  40 transient reset fields, nine initial properties and the 24-field transition
  cache now use the central sprite schema. The cache is deliberately partial:
  callbacks retain changes outside those 24 fields. Raw replay patches remain
  immediately visible. Movement layout knowledge moved into the sprite view.
- **Tile maps (M02/M10):** word indices and byte offsets are distinct types.
  Live dungeon BG1/BG2, overworld map16 and collision-attribute views replace
  selected address arithmetic in dungeon, overworld and collision callers.
  BG2/map16 retain their shared storage, odd byte reads remain unaligned, and
  existing caller-side wrapping and mutation order are preserved.
- **Graphics borrowing (M14/M15):** CPU-side graphics scripts and callbacks use
  `GraphicsCpu`, which cannot borrow VRAM; hook-free transfers use
  `GraphicsUpload`. The original `LoadGfx` entrypoint facade delegates to them.
  All three native graphics helper paths stop allocating zeroed 64-KiB VRAM
  replacements; messaging reuses the shared CPU/upload helpers.
  Normal planar conversion uses disjoint borrowed source/destination slices and
  direct word output instead of source/conversion vectors. Explicit overlap-only
  snapshots retain the old compatibility API behavior. Low-WRAM planar scratch
  residue, callback order and mirror publication remain observable.

The graphics snapshot/diff/flush bridge has **not** been removed. Tests exercise
nested callbacks, mirror values 0/8/13, full scratch/sentinel images, overlapping
conversion, erase-then-rewrite and chained host-action flush visibility. These
tests protect the next live-VRAM ownership step; they do not claim it is done.

## Focused C evidence

New standalone harnesses link the existing C oracle without modifying C gameplay
or device code. Both checkers are mandatory in `scripts/test-parity.sh`.

- `check_sprite_state`: 14,240 complete-WRAM cases for inversion, halving,
  wall masks, reset/properties, transition caching, location projection and
  defined angle-table inputs. In addition, all 65,536 current/target approach
  pairs are checked on both axes at slots 0 and 15; the full WRAM image is
  compared after each exhaustive stream, not after each individual pair.
- `check_tile_maps`: all 131,072 address-conversion results and 128 whole-WRAM
  dungeon 2×2 update cases. Update vectors use aligned, disjoint normal upload
  buffers and valid map positions; this is not a claim about arbitrary overlap
  or odd upload pointers. Existing Rust update ordering is retained, including
  its per-tile interleaving rather than changing it to C's grouped writes.

Unit tests additionally check all byte negations/halves and wrapped approach
pairs, timer neighbors, live cache patches, partial restoration, shared map
aliases, unaligned reads and full-image sentinels. Source audits normalize the
map accessor substitutions against the parent baseline; C provenance remains
accounted for by the existing entrypoint audit.

The graphics inverse audit against `c224740` checks 146 CPU method bodies
(137 unchanged after whitespace normalization, nine reviewed borrowing changes),
14 upload methods (six unchanged, seven reviewed conversion changes, one real
free upload kernel), and all 144 original public facade methods (138 exact
delegates with original argument order, six unchanged read-only/static bodies).
The host-action drain loop is byte-identical after extraction. The messaging
helper migration preserves call order and sends only its two upload operations
through the hook-free context; all other operations use the CPU context.

## Acceptance status

Both frozen-source commands below exited successfully. Each sweep covered all
13 recordings and all 1,073,092 frames. No previous batch's recording run is
counted as evidence for these changes. No comparisons or masks were relaxed.

| Gate | Result |
| --- | --- |
| C source entrypoint audit | 3,366 / 3,366 accounted for |
| Workspace unit tests | 580 passed, including 462 gameplay tests; GPU opt-in test remains ignored in the normal suite |
| Workspace formatting and strict all-targets Clippy | Passed |
| New sprite C differential | 14,240 full-WRAM cases and exhaustive approach streams passed |
| New tile-map C differential | 131,072 conversions and 128 full-WRAM cases passed |
| Existing feature/component C checks | All passed, including aspect ratios and 4× Mode 7 map pixels |
| Complete state/pixel/PCM sweep | All 13 recordings passed, every frame/pixel/PCM value exact |
| Separate queue/SPC/DSP audio-state sweep | All 13 recordings passed, every frame exact |
| Native software smoke | 600 frames passed |

```sh
Z3_REPLAY_JOBS=3 scripts/test-parity.sh
Z3_VERIFY_AUDIO_STATE=1 scripts/test-full-audio-replay.sh
```

GPU presentation code is unchanged in this batch. The earlier batch's Metal
readback result is not presented as a new GPU run here. The main pixel sweep
checks the actual PPU frames before host post-processing.

## Remaining scope

Global OAM allocation and other draw callers, object raster cursors, map stream
decoding, collision scratch/results, additional pools/state IDs, cross-system
views, live graphics ownership, snapshot/SPC codecs, audio/host/device boundaries,
renderer surface/history ownership and gameplay organization remain in the full
[modernization plan](rust-modernization-plan.md).
