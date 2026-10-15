# Modernization batch 2: pools, transfers and renderer boundaries

Date: 2026-10-14. Parent baseline: `0be0ff2` (batch 1, pushed to `kern/main`).
The full M01–M24 backlog remains the objective; this is an incremental receipt,
not a declaration that the remaining map, state, ownership or device work is done.

## Changes

- **Sprite storage (M01/M02/M09):** one reviewed declaration for 59 byte-array
  fields; live pool/slot views, validated physical `SpriteSlot` iteration and
  reverse free-slot search. Migrated 3,598 unsigned reads, 4,733 unsigned writes,
  123 signed reads and 328 signed writes across sprite logic and all four banks.
  Signed setters accept `i8`, preserving the original RHS inference rather than
  repairing expressions with unsigned casts. Explicit legacy-index access keeps
  sentinel `0xff` reads and neighboring aliases; it does not clamp or compact.
- **Effect storage (M01/M09):** 41 named live byte fields and a named carried-Y
  word alias, including short five-byte arrays whose upper effect slots alias
  adjacent fields. Over 1,000 handler expressions and ordered constructor writes
  use these views. Inactive bytes, callbacks and allocation order are retained.
- **Integer coordinate storage (M03):** controller and follower/history XY use
  `SplitCoordinate`; no nonexistent subpixel storage was introduced. Movement's
  existing fixed-point types and compatibility write ordering are unchanged.
- **Graphics uploads (M02/M11):** typed VRAM-word destinations, explicit packed/
  column/low-byte operations, and distinct strict versus off-buffer clipping
  policies. Big-endian stripes, little-endian UVRAM packets and overworld scroll
  packets have separate lazy decoders. NMI orchestrates immediate transfers;
  codec arithmetic stays at the compatibility boundary. Read-only NMI sources
  now borrow RAM instead of copying it into temporary vectors. Odd copy/fill
  lengths, padding, marker timing, partial-error writes and clipping are retained.
- **Presentation structure (M23):** private preset, shader-source, pass-plan,
  texture/mipmap and test modules replace the monolithic renderer layout.
  Shader text and production draw/order/binding logic were mechanically retained
  except for the explicitly described attachment-role correction below.
- **Verification runner (M24):** `Z3_REPLAY_JOBS=1..13` partitions any requested
  chapter range without overlap/gaps. All workers are waited, and any failed
  worker fails the script; the default remains serial. No comparison is skipped.

The source inverse-audits reproduce all five prior sprite files after normalizing
the named accessor substitutions and rustfmt, with explicitly reviewed exceptions
for typed physical-slot iteration/search and the N-word alias. All 59 sprite
array bases were independently checked against `variables.h`.

## GPU correction and evidence

A mipmapped pass target previously shared one full-chain texture view for both
sampling and rendering. Metal/wgpu validation rejected a four-level color
attachment: render attachments must identify a single mip level. `PassTarget`
now owns distinct `sampled` and `attachment` views. Sampling still sees all mip
levels; rendering targets level zero. Texture formats, shader text, uniforms,
parameters, filters, wrap modes, transfer order and mipmap generation stay intact.
The ignored GPU regression uses the production attachment field (not a handmade
test-only bypass), exact synthetic RGBA expectations, and real Metal readback for
nearest blitting plus multipass/history uniform/parameter bindings.

This is a concrete pre-existing validation failure fixed inside the touched
resource boundary. It does not change the C simulation, pre-presentation PPU
pixel output or audio. The GPU test is not a proof for every preset/surface mode.

## Acceptance

The C gameplay/device implementation, existing masks, output comparisons, asset
schema, recordings and configuration contracts are unchanged. Compiler/link
setup is the same as [batch 1](modernization-verification.md).

| Gate | Result |
| --- | --- |
| Complete C entrypoint audit | 3,366 / 3,366 accounted for |
| Workspace tests, lint and formatting | Corrected batch: 563 tests passed (445 gameplay); strict workspace Clippy and formatting passed |
| Movement differential | 3,492 complete-WRAM vectors passed |
| Stripe differential (new mandatory gate) | 276 complete-VRAM vectors passed |
| Metal readback (opt-in, outside sandbox) | Passed |
| Features/subsystems/native smoke | All C feature/subsystem checks and the 600-frame native software smoke passed |
| All 13 complete state/pixel/PCM recordings | Corrected batch passed all 1,073,092 frames, every pixel and PCM value exact |
| All 13 queue/SPC/DSP diagnostic recordings | Corrected batch passed all 1,073,092 frames |

```sh
Z3_REPLAY_JOBS=3 scripts/test-parity.sh
Z3_VERIFY_AUDIO_STATE=1 scripts/test-full-audio-replay.sh
cargo test -p z3-sdl \
  gpu_blit_and_multipass_history_readback_matches_generated_pixels \
  -- --ignored --nocapture
```

The new C stripe harness needs no game assets and calls the unmodified C
`HandleStripes14`; it is wired into the normal build/gate. Inputs stay within
defined C VRAM bounds, including the final word. Rust-only tests additionally
cover strict failure visibility, odd lengths, and explicitly clipped transfers.

The first audio-state sweep correctly rejected the batch at chapter 5, frame
19,171 (OAM X at WRAM `0x830`). A complete effect-handler inverse audit found
two linked misplaced substitutions: an Ether spoke workspace write had been
routed to the velocity setter, while the intended slot velocity update still
used a raw address. Both were corrected; dedicated step-4/step-5 regressions now
check exact workspace updates, velocity limits, and untouched low/remote WRAM.
The complete handler inverse audit then passed. The earlier run is not counted
as acceptance; all gates and complete recording sweeps are restarted below.

## Next remaining work

Cross-system player/sprite accessors, extended spell/debris workspaces, follower
Z/layer history and controller globals still need domain views. OAM allocation,
tile/map/VRAM address arithmetic, packet generation, scratch liveness and owned
state are not complete. Renderer surface/history ownership and broader GPU
preset/readback coverage remain. Other M01–M24 entries retain their full scope.
Arithmetic follow-up must include explicit wrapping negation for signed byte
velocities (`sprite_invert_xy_speeds` / wall bounces), with all-byte C vectors;
merely relying on release overflow behavior is not an ergonomic contract.

Prepared next increments from read-only reconnaissance:

- **M04/M09:** twelve unsafe signed-byte unary negations in sprite helpers;
  preserve the C promoted-integer-then-byte result with explicit wrapping.
  Model arithmetic halving and wrapped-difference approach separately. Generic
  sprite operations still comprise four countdowns, 40 reset fields, nine
  property writes, 24 cached fields and five velocity-axis loops. Use the field
  schema, but preserve each loop's exact source order and alias behavior. Angle
  tests must avoid undefined C table indices; not every velocity pair is valid.
- **M10/M12:** dungeon BG2 words and overworld map16 share `0x2000`; byte-offset
  reads can be unaligned and must not be silently aligned. A dungeon 2×2 write
  emits upload, tile and attribute effects per tile in order. Global OAM primary
  and extended pointers are independent live fields; sprite and effect code
  publish them in different orders. Separate local cursors from global region
  allocation rather than rederiving an extended pointer from a primary pointer.
- **M14/M15:** split WRAM/assets/hook computation from hook-free VRAM upload
  kernels before removing the scratch bridge. There are eight VRAM write sites
  in `load_gfx.rs`; most callback-bearing methods do not need a VRAM borrow.
  This can remove zeroed 64-KiB replacement allocations in both graphics helpers
  and conversion/source copies using disjoint slices. Keep low-WRAM conversion
  residue observable. Removing the remaining snapshot/diff bridge requires a
  transient service borrowing live VRAM at routing boundaries, with tests for
  nested callbacks, erase/rewrite, direct PPU writes and chained flush visibility.
