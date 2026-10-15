# Modernization batch 4: allocation, snapshot and presentation ownership

Parent baseline: `2904d88` (batch 3, pushed to `kern/main`). This receipt tracks
another bounded increment; the full M01–M24 backlog remains open.

## Global OAM allocation (M02/M12)

`OamRegion`, `OamAllocator`, `OamAllocation` and independent `OamPointers` move
region tables, allocation arithmetic and pointer publication into the OAM
boundary. Sprite compatibility wrappers and reset/current-entry helpers delegate;
ancilla's current-entry validation shares the same cursor operation. Two sprite
draw paths advance both live pointers without deriving either from the other.

Byte counts may be unaligned. Odd legacy region selectors retain their paired
aliases. Allocation wraps the 16-bit end before comparing the threshold;
fallback retains the base, uses the old counter's low three bits and increments
the full counter with wrapping. Allocation publishes extended before primary;
ordinary advancement writes primary before reading/updating extended. Reset
touches only six bases, not counters, pointers or hidden staging bytes.
The primary address addition is now explicitly wrapping, matching C/release for
raw wrapping inputs instead of panicking in checked Rust builds.

The asset-free C harness checks 2,716 complete-WRAM images and return bytes:
all 256 byte counts, thresholds, unaligned starts, odd selectors, all fallback
positions, base/counter overflow, six wrappers and reset preservation. The
safe pointer-wrap vectors do not dereference OAM. Source inverse audits retain
all untouched sprite/ancilla bodies and every moved table element. Custom
ancilla wrapping and handlers/native/bank pointer mutations remain outstanding.

## Snapshot boundary (M17)

`snapshot.rs` owns the fixed legacy region schema, checked immutable/mutable
views and the opaque historical template. Runtime code uses named regions and
separate load/write operations; the C-shaped boolean entrypoint remains as a
compatibility facade. Immutable loading no longer clones the full 275,465-byte
image. Live storage still belongs to the runtime/devices, not to the template.

Length acceptance, error text, device transfer order, template capture after
successful import, HDMA relocation and music restoration are unchanged. Unknown
outer and device bytes keep the existing codec behavior. The independent
verifier schema is deliberately not shared with production.

The mandatory `check_runtime_snapshots` cross-checks all 11 regions with that
independent parser on 25 reference images (13 current and 12 replay bases),
then verifies complete round trips, immutable import, live WRAM/SRAM/VRAM/SPC
mutations and untouched bytes. Each image is also checked after nonuniform
patches to every outer opaque gap. Unit tests cover exact region coverage,
immediate local mutation, template preservation and wrong-length rejection.
The recording container/header and replay-log codec are not yet extracted.

## Host asset offsets (M06/M24)

The native host's custom-Link/dim-flash patch adapter still reconstructed file
offsets by casting two slice pointers to integers and subtracting them. It now
uses the parser's validated source range directly. A regression checks every
one of the 165 payloads and retains the out-of-range error text. This completes
that host call site, not the broader typed-asset-ID and stream-cursor backlog.

## Renderer ownership (M23)

`SurfaceOwner` encapsulates configuration, resize/reacquisition and viewport
policy. Platform window-handle FFI lives in `surface.rs`; SDL still owns the
window. The renderer drops its surface before GPU resources and retains the
Metal view as its final owning field. The constructor now declares that view
before the surface, so early adapter/device/preset failures also drop a raw or
configured surface before destroying its Metal layer. This corrects the former
tuple-binding error-path drop order without changing successful presentation.

`FrameHistory` owns prior-frame textures and the shader presentation clock.
Binding fallback, ring wrap and resize reuse are preserved. Only a successful
present records history and advances the wrapping clock; timeout/occlusion/error
returns do neither. Shader text, parameter values, formats, pass order and the
original floating-point viewport arithmetic are retained.

The real Metal readback regression passes with the production history owner,
covering generated pixels, multipass/history binding, ring reuse and resize.
A real native 600-frame WGPU run with the bundled CRT preset also exits cleanly,
using a temporary working/save directory and read-only asset symlink, not the
user's save directory. CPU tests cover ring/clock wrapping, extent normalization
and aspect/stretch viewport behavior. These checks do not certify all preset,
surface-error or platform combinations; Windows/Linux runtime execution remains
outside this macOS receipt.

Normalized inverse comparisons against `2904d88` match platform ABI/FFI blocks,
pipeline/preset/resource planning, source uploads before acquisition, draw and
overlay encoding, source allocation/reuse and surface retry/error branches.
Reviewed substitutions are surface-format/clock getters, extracted history
binding and the `Option` return for skipped acquisition. History's label,
binding/sampler selection, fallback and post-present ordering are unchanged.
The constructor cleanup ordering above is the intentional behavior change.
Surface-error injection and interactive resize/minimize still need dedicated
coverage; headless readback alone is not evidence for them.

## Acceptance status

Both frozen-source sweep commands exited successfully. Each covered all 13
recordings and 1,073,092 frames; the main gate also completed its final native
software smoke. No previous batch's replay result certifies this batch, and no
comparison or mask was relaxed.

| Gate | Result |
| --- | --- |
| C entrypoint audit | 3,366 / 3,366 accounted for |
| Workspace unit tests | 593 passed, including 469 gameplay tests; one GPU test ignored in the ordinary suite |
| Workspace formatting / strict all-target Clippy | Passed |
| OAM C differential | 2,716 full-WRAM / return-byte cases passed |
| Independent snapshot cross-check | 25 reference images plus opaque-gap variants and live mutations passed |
| Existing C feature/component checks | All passed, including aspect ratios and 4× Mode 7 map pixels |
| All 13 complete state/pixel/PCM recordings | Passed, every frame/pixel/PCM value exact |
| All 13 queue/SPC/DSP diagnostic recordings | Passed, 1,073,092 frames exact |
| Real Metal synthetic readback | Passed |
| Native WGPU + CRT surface smoke | 600 frames passed, isolated working/save directory |
| Native software smoke | 600 frames passed |

```sh
Z3_REPLAY_JOBS=3 scripts/test-parity.sh
Z3_VERIFY_AUDIO_STATE=1 scripts/test-full-audio-replay.sh
cargo test -p z3-sdl \
  gpu_blit_and_multipass_history_readback_matches_generated_pixels \
  -- --ignored --nocapture
```

The main sweep compares every pixel before host post-processing. Metal readback
uses generated expectations and the actual production history/resource helpers;
the native CRT run checks surface execution, not exact C post-shader pixels.

## Remaining graphics ownership work

The M15 staging bridge remains: main-route entry, each drained host action and
the ending erase adapter rebase the scratch view. Device drain order remains
mirror HDMA, VRAM spans, PPU register writes, then graphics HDMA; newly queued
host actions run in later rounds. A live-VRAM service cannot simply be stored
beside adapters holding `&mut Ppu` (including Attract's direct tile draw).
The next step needs per-operation reborrowing or a real PPU storage split, not
another writable mirror or a skipped flush. Existing nested/erase-rewrite/chained
flush regressions must continue to cover those observation points.

## Prepared next audio-codec increment (M18)

Read-only reconnaissance found 52 ordered channel mappings (63 bytes each,
eight channels with stride two), 67 player scalar mappings and three four-byte
port arrays: 598 distinct mapped bytes, no overlaps, highest address `0x3ff`.
A single ordered typed schema can generate both codecs and a passive snapshot
overlay, retaining explicit little-endian words and unmapped state/RAM.
The C table's high address bit encodes field width, not an SPC address.

Both current diagnostic exporters actively copy variables into live SPC RAM.
A passive observer must therefore be introduced on both C and Rust sides,
without silently replacing the existing mandatory diagnostic contract. Retain
the legacy live-export regression and explicit save/load behavior; add a distinct
passive C export/check path and prove repeated observation leaves future PCM,
DSP, ports, echo RAM and history unchanged. Independently audit every generated
field/address/width/order against the original C tables; a codec round trip alone
can conceal matching mistakes in both directions. This is prepared work only,
not implemented or counted as accepted in this batch.
