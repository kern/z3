# Rust modernization inventory and parity-preserving plan

Inventory date: 2026-10-14. Source baseline: `e424757`.

This document tracks the plan and its implementation. It covers the implementation of
the full Rust port, including direct memory access, pointer/offset arithmetic,
data representation, algorithms, control flow, ownership, and host APIs. The
goal is ordinary, understandable Rust gameplay code with explicit compatibility
boundaries, while retaining the existing C parity contract.

## Implementation progress

Implementation batches replace layout knowledge at selected boundaries; they do
**not** yet complete the entire M01–M24 backlog or introduce owned gameplay
state. All converted views still access the single live compatibility image.

| Area | Implemented scope | Remaining scope |
| --- | --- | --- |
| M01 player views | `PlayerRef`/`PlayerMut`, explicit overlapping byte/word fields, position/velocity/action/equipment/inventory; 411 reads and 373 writes migrated in `player.rs` | Other player fields/callers, complete region reader/writer and reset maps |
| M02/M09 sprite/effect pools | 59 sprite fields and 41 effect fields behind live views; 8,782 sprite byte accesses plus 5 word aliases and over 1,000 effect expressions migrated; typed physical slots, reverse search, ordered countdown/reset/property operations and a live partial transition cache | Boss history storage, extended effect workspaces, cross-system callers and other pools |
| M03 movement | Shared wrapping 16.8 world position and 8.8 height types; ancilla/sprite XYZ helpers; integer split-coordinate views for controllers/follower history | Player-specific movement and remaining coordinate consumers; no invented fractional controller/follower fields |
| M04 byte velocity | Explicit wrapping negation, arithmetic halving and wrapped-difference approach; sprite helper and selected bank callers migrated; exhaustive byte/pair tests and C vectors | Remaining gameplay arithmetic, promoted intermediates and carry/borrow operations |
| M06 asset ranges | Validated file ranges and explicit suffix view; removed palette pointer-to-integer subtraction | Typed asset IDs and stream/table cursors |
| M07 sound | Named ambient/effect latches, lossless requests, lazy first-writer emitter resolution, player/world/tile pan constructors; migrated regression-sensitive effects, sprite service helpers and NMI | Remaining raw sound call sites and richer effect IDs |
| M10 tile maps | Distinct word indices/byte offsets, live aliased dungeon/overworld maps and collision attributes; selected map callers migrated | Object raster cursors, map stream decoding, remaining map arithmetic and collision scratch/results |
| M11 graphics uploads | Typed word destinations, immediate transfer operations, explicit clipping/strict policies; separate stripe, UVRAM and scroll decoders; borrowed NMI sources | Packet generation, gameplay-facing upload staging, palette/HDMA views and remaining address-based transfer profiles |
| M12 OAM | Entry-unit local cursor and player drawing migration; preserves hidden bytes, global allocation cursor, paired extended writes and one-past results | Other entity draw paths and global allocator API |
| M14 ending routing | 58 explicit `EndingCall` variants through immediate/deferred production dispatch; string routes removed | Other broad capability traits, default no-ops and borrow scaffolding |
| M14/M15 graphics contexts | CPU scripts/callbacks cannot borrow VRAM; hook-free uploads borrow disjoint graphics storage; normal planar source/conversion temporaries removed | Snapshot/diff/flush bridge, remaining mixed service helpers and transient live-VRAM routing |
| M20 host commands | Lossless `HostCommand` including unknown values, typed dispatch and runtime patch keys | Host scheduling, audio ownership, RAII and application decomposition |
| M23 renderer | Separate preset/source/plan/texture-mipmap modules; distinct sampling versus render-attachment views; synthetic Metal readback regression | Surface/history ownership, presentation policy decomposition and broader post-shader preset coverage |

Focused tests cover complete-memory equivalence, live aliases, signed movement
boundaries, OAM visibility/adjacent bytes, sound latch priority and hook timing,
asset padding/credits lookup, command round trips, and ending call ordering.
See [batch 1](modernization-verification.md), [batch 2](modernization-batch-2.md)
and [batch 3](modernization-batch-3.md)
for exact commands, completed gates and recording-sweep status. Counts describe
source accesses, including migrated existing tests, not distinct state fields.

## Non-negotiable behavior

The baseline passes all 13 complete reference recordings: 1,073,092 frames of
exact canonical state, rendered pixels, and PCM output. The separate audio
diagnostic sweep also compares the APU queue and mapped SPC/DSP state.
See [the parity baseline](parity-port-plan.md).

Modernization must preserve:

- Every compared WRAM, SRAM, and VRAM byte, including scratch and apparently
  unused bytes. Do not mask new differences or zero inconvenient regions.
- Every rendered pixel, including high-resolution Mode 7 and hidden OAM effects.
- Every PCM value at the same frame/audio cadence, and sound-port latch ordering.
- Legacy save import/export, replay patches to arbitrary WRAM addresses, reset
  behavior, and opaque snapshot bytes retained by the existing snapshot template.
- Fixed-width arithmetic, truncation points, signed promotion, carry/borrow,
  timer underflow, RNG calls, fixed slot allocation, and processing order.
- Existing optional gameplay, aspect-ratio, rendering, input, shader, and audio
  settings. Preserve the old configuration aliases and file formats.

Passing the recordings is necessary, not a proof of equivalence on every possible
input or feature combination. Add focused C vectors for each changed behavior,
especially branches absent from the recordings. Keep the C oracle unchanged;
do not redefine the expected output to accommodate a refactor.

## Inventory scope and measurements

The inventory reviews crate boundaries, module responsibilities, storage and
call interfaces, and representative implementations throughout each subsystem.
It is not a claim that every individual expression has been formally audited.
Counts below include tables and inline tests, exclude dependencies/build output,
and were measured from `crates/*/src/**/*.rs` at the baseline.

| Crate | Rust files | Lines | Main modernization opportunity |
| --- | ---: | ---: | --- |
| `z3-game` | 35 | 120,950 | Address-based state and bank-based gameplay organization |
| `z3-core` | 6 | 2,415 | Legacy memory/codecs, C helper types, numeric config/asset IDs |
| `z3-snes` | 12 | 6,712 | Isolate accurate devices from gameplay-facing operations |
| `z3-native` | 4 | 1,264 | Typed host commands, scheduling, explicit resource ownership |
| `z3-sdl` | 2 | 3,142 | Separate SDL FFI, GPU presentation, shader/preset management |
| `z3-opus` | 1 | 73 | Retain narrow RAII FFI; improve errors and sample types |
| `z3-verify` | 25 | 4,559 | Reusable runners, read-only diagnostics, semantic source mapping |
| **Total** | **85** | **139,115** | |

A syntactic scan of `z3-game` found 12,756 calls starting with a literal hex
address across its memory accessor spellings, 4,873 `.wrapping_*` calls, and 31
`RefCell::new` expressions. These are indicators, not counts of defects: named
address arithmetic is additional, and much of the wrapping behavior is required.

The gameplay crate already avoids unsafe pointer dereferencing. That does not
make its representation ergonomic: safe indexing such as `base + slot`, manual
byte assembly, and address-derived table cursors still encode C/SNES pointer
semantics. At baseline, one actual address-to-integer subtraction remained in
`RuntimeAssets::background_palettes`; it reconstructs a file offset from two
slice pointers (removed in the first implementation batch). SDL and Opus have
legitimate raw FFI pointers, a separate concern.

Key source entry points are the [RAM accessor](../crates/z3-game/src/ram.rs),
[asset container](../crates/z3-core/src/assets.rs),
[runtime asset adapter](../crates/z3-game/src/assets.rs),
[entity movement](../crates/z3-game/src/ancilla.rs),
[production composition](../crates/z3-game/src/integration.rs),
[graphics service borrowing](../crates/z3-game/src/native_services.rs), and
[snapshot/replay runtime](../crates/z3-game/src/runtime.rs).

## Architecture to work toward

Gameplay should operate on named state views and domain values: player position,
sprite slots, tile coordinates, palettes, sound requests, and scene stages.
Compatibility code should own byte layout, historical aliases, packed codecs,
legacy addresses, device registers, and documented C quirks.

Use two migration stages, with one source of truth at each stage:

1. **Typed views over existing storage.** Introduce `PlayerView`,
   `SpritePoolView`, `DungeonMapView`, etc. Their methods read/write the current
   RAM image immediately. Move address arithmetic into those implementations.
   This changes how callers express behavior without changing where state lives.
2. **Owned domain state where justified.** After every reader/writer of a region
   is known and converted, move that region into ordinary Rust fields/arrays.
   Provide an exact legacy codec and a defined route for raw replay patches and
   remaining compatibility accesses. Retain opaque bytes and live aliases.

Do not introduce a writable `GameState` cache beside writable WRAM and synchronize
it only at the end of a frame. Callbacks, overlapping accesses, and replay patches
would see stale values. Do not fabricate a canonical image merely to satisfy the
verifier: it must encode the state and side effects of the runtime actually used
by the interactive host. A region that cannot yet meet those rules stays backed
by compatibility memory, hidden behind its domain API.

The final ergonomic boundary need not eliminate every byte array. Tile data,
audio samples, device RAM, and snapshot codecs naturally contain bytes. The goal
is to remove layout knowledge from ordinary gameplay logic.

## Implementation-level opportunities

Priority: **P1** is the first practical wave, **P2** follows typed boundaries,
and **P3** is a later structural migration. Risk describes behavioral risk,
not implementation effort. Names of proposed types below are illustrative.

| ID / priority / risk | Current evidence | Proposed replacement and parity constraint |
| --- | --- | --- |
| M01 / P1 / medium | `ram.rs`; repeated `r8/w8/r16/w16`, literal addresses throughout `player.rs`, `dungeon.rs`, sprite banks | Central layout declarations and named borrowed views. Preserve unaligned word access, byte aliases, and immediate visibility. Merely naming constants is an intermediate step, not the endpoint. |
| M02 / P1 / medium | `base + k`, `index * 2`, `OAM_BUFFER + index * 4`; VRAM words and WRAM bytes both represented as integers | Distinct `SpriteSlot`, `AncillaSlot`, `WramAddr`, `VramWordAddr`, `ByteOffset`, `TileIndex`, and banked-address types. Make unit conversions explicit; retain intentional wrap separately from checked host offsets. |
| M03 / P1 / medium | Split X/Y arrays in `ancilla.rs`, `sprite.rs`, `overlord.rs`, `tagalong.rs`; `move_axis` assembles a 24-bit value | Position/subpixel views and a tested fixed-point value type. Preserve the 24-bit mask, signed 8-bit velocity scaled by 16, and distinct 16-bit Z arithmetic. Do not replace movement with floats or wider unbounded accumulation. |
| M04 / P1 / medium | Thousands of casts/wrapping calls; C macro equivalents in `c_types.rs`; recoil/collision code in `player.rs` | Name repeated arithmetic operations by their domain meaning. Specify promotion and truncation in the helper, including signed shifts and borrow/carry. Do not mechanically replace wrapping operations with normal arithmetic or saturation. |
| M05 / P1 / high | Shared scratch addresses, register-style `r0/r2` results, `cur_object_index`, poly/ending scratch near `0x1fb0` | Introduce named result structs and scratch views. Convert scratch to local variables only after a read/write and callback-liveness audit; preserve observable scratch writes until proven irrelevant to the contract. |
| M06 / P1 / medium | `RuntimeAssets` numeric indices and four decoded views; `background_palettes` subtracts pointer values | Store explicit asset file ranges in `AssetFile`; expose typed asset IDs and validated typed tables/cursors. Preserve the packed-file suffix used by the credits lookup, odd-aligned room streams, and accepted trailing bytes. |
| M07 / P1 / medium | Raw writes to `0x12d..0x12f`, sound ID/pan ORs; recent sword-beam/Ether and enemy-pan bugs | Typed sound ports, pan values, and explicit player/sprite/tile emitters. Preserve overwrite-versus-if-empty latch policies, the separate eight-zone tile pan, and NMI clear timing. A queue of all requests is not equivalent to the current latches. |
| M08 / P1 / medium | Sentinel `-1`/`0xff`, raw kind/state/direction IDs, packed flag masks | Use domain IDs, `Option<Slot>` at allocation APIs, and enums/flag wrappers with lossless raw encoding. Preserve unknown/reserved byte values at compatibility boundaries rather than inventing defaults or rejecting previously supported states. |
| M09 / P2 / high | Entity SoA in WRAM, reverse scans, fixed sprite/effect pools and auxiliary A/B/C fields | Fixed-capacity typed pool views first; later owned pool records where aliases allow it. Preserve physical slot identity, allocation scan direction, same-frame spawn visibility, inactive-slot contents, and per-kind reuse of auxiliary fields. SoA itself can remain; a dynamic ECS is not a prerequisite. |
| M10 / P2 / high | Packed dungeon/overworld map formulas, shifts/masks, object source/destination arithmetic | Typed map/grid/room views and explicit byte/word cursors. Keep original stride, layer selection, wrapping edges, and mutation order. Use overlap-aware decompression/back-reference operations, not an indiscriminate slice-copy rewrite. |
| M11 / P2 / high | NMI DMA packet bytes, `uvram` offsets, palette staging, bank/register tuples | Named upload operations plus a compatibility encoder/interpreter. Keep hardware-visible transfer order, odd-length truncation, packet headers/padding, HDMA scanline behavior, and the exact point where staged data becomes visible. |
| M12 / P2 / high | OAM pointer in WRAM, local/global OAM advancement, `PrepOamCoordsRet`, high-X/size packing | `OamCursor` and explicit local/global writer operations; named sprite draw records. Retain OAM slot order, clipping differences, untouched hidden-entry bytes, and last-coordinate caches used by shadows. Sorting or rebuilding a clean sprite list changes behavior. |
| M13 / P2 / medium | Numeric module/submodule routing, translated loop/goto shapes, booleans used as status codes | Extend the existing `MainModule` pattern with subsystem-specific stages and explicit transition results. Refactor one control-flow region at a time, preserving fallthrough, decrement-before-test, skipped calls, and exact transition frames. |
| M14 / P2 / high | Broad runtime traits, default no-op hooks, string calls, many reborrows/`RefCell`s | Smaller required capabilities, typed call enums, and a narrow frame context. Follow existing typed `NativeHostAction`/`MiscCall` examples. Preserve immediate versus deferred calls and every re-entry observation point; test doubles may opt into no-ops explicitly. |
| M15 / P2 / high | `NativeGameServices` scratch VRAM plus snapshot; full-array diffs; `with_load_gfx` allocates replacement storage | Borrow the real graphics storage through disjoint context fields, or use explicit tracked writes. Remove copies only after nested-call tests prove identical visibility. An erase followed by a write equal to the old value must still reach live VRAM. |
| M16 / P2 / medium | 35 gameplay source files largely mirror C units/ROM banks; repeated movement, OAM, pan, and spawn adapters | Organize by domain and named entity behavior after storage APIs settle. Extract shared behavior only when exact side effects match. Retain C-entrypoint-to-Rust-symbol provenance rather than duplicating implementations to satisfy the name audit. |
| M17 / P2 / high | Runtime save offsets, `legacy_snapshot_template`, recorder WRAM patches; parallel verifier decoders | A compatibility snapshot/replay module with explicit region schemas and typed readers/writers. Preserve opaque bytes, legacy checksums, patch wrapping, reset/music restoration, and byte-for-byte round trips. Keep independent verifier checks so a shared codec bug cannot validate itself. |
| M18 / P2 / medium | SPC field-to-RAM macros, pointer-like pattern cursors and duplicate mapping directions in `spc_player.rs` | Typed pattern cursors, named voice/state records, and one reviewed mapping specification generating both codecs. Preserve live SPC RAM/echo interactions, mapped field offsets, register-write order, and all integer/sample arithmetic. |
| M19 / P2 / medium | `ApuWriteQueue` masks/counts; interleaved buffers; MSU integer tags/options in `audio.rs` | Typed port snapshots, stereo sample/frame views, and explicit playback states. Keep the 16-entry overwrite policy and wrapping counters. Any enum must retain necessary resume/raw states; do not merge PCM/OPUZ paths if their semantics differ. |
| M20 / P2 / medium | Host numeric command constants, key ranges, SDL flags, manual lock/unlock names, whole-runtime audio mutex | Typed host commands and RAII guards; isolate device callbacks from host policy. Split audio ownership only after deterministic command delivery is defined. Preserve normalization, pause/turbo behavior, callback chunking, volume math, and save/reset locking order. |
| M21 / P3 / high | Gameplay reaches PPU/DMA/register details through NMI/load-gfx adapters | A gameplay graphics service above an accurate compatibility renderer. Keep the PPU/DMA implementation as the reference backend. Replacing it with a scene renderer is a separate project requiring new pixel/scanline proofs. |
| M22 / P3 / medium | `Snes` owns CPU/APU/bus/cart/input; native runtime directly owns PPU/DMA/native music player | Separate native-required devices from optional full-emulator machinery by dependency analysis. Keep CPU/SPC opcode oracles and legacy codecs. Do not assume the native game needs a CPU interpreter, or that all of `z3-snes` can be removed. |
| M23 / P2 / medium | Large `wgpu_renderer.rs` mixes presets, resource history, GPU setup and presentation | Separate preset parsing, shader resources/history, surface ownership, and presentation. Preserve pass order, history/frame counts, formats, filter/scaling, LUTs, sRGB/float targets, and WGSL bindings. Keep unsafe surface handles confined to the host boundary. |
| M24 / P1 / low | `c_types.rs` compatibility wrappers, `util.rs`, config numeric masks/legacy output aliases | Prefer standard slices, domain-specific result types, and typed config flags where lossless. Keep meaningful fixed-width types and wire codecs. Remove redundant wrappers only with reviewed call-site/provenance updates. |

## Coverage by gameplay subsystem

This table maps every `z3-game` source module into the backlog. It prevents a
plan centered on rendering interfaces from overlooking the bulk of the port.
Paths here are relative to `crates/z3-game/src/`.

| Modules | Specific work beyond the shared memory views |
| --- | --- |
| `lib.rs`, `ram.rs`, `runtime.rs` | Module boundaries, schema-backed state access, frame scheduling, reset and snapshot ownership; M01–05, M13, M17 |
| `integration.rs`, `native_services.rs`, `native_gameplay_services.rs`, `native_messaging_services.rs` | Capability boundaries, typed routing, callback order and VRAM ownership; M14–16 |
| `player.rs` | Player/inventory/action views; signed collision and recoil operations; state transitions and item APIs; M01–05, M08, M13 |
| `sprite.rs`, `sprite_main_bank0.rs`, `sprite_main_bank1.rs`, `sprite_main_bank2a.rs`, `sprite_main_bank2b.rs` | Fixed pool views, per-enemy behaviors, spawn/interaction results, typed hitboxes, exact damage and slot iteration; M02–05, M07–09, M12–16 |
| `ancilla.rs`, `ancilla_handlers_0.rs` | Projectile/effect pools, fixed-point movement, typed collision results, item-specific state and sound latches; M02–09, M12 |
| `overlord.rs`, `tagalong.rs` | Spawner slot views, follower history rings, spawn results and explicit lifecycle stages; M03, M05, M08–09, M13 |
| `dungeon.rs` | Room/door/map views, object stream cursors, palette/tile offsets, door transitions and tile pan; M01–05, M10–11, M13 |
| `overworld.rs` | Map16/map8 views, camera/scroll units, decompression cursors, event overlays and transitions; M02–05, M10–11, M13 |
| `tile_detect.rs` | Typed collision probes, bitfield results and layer-aware map coordinates, without dropping scratch writes; M02–05, M10 |
| `load_gfx.rs`, `nmi.rs` | Asset/VRAM address units, palette views, upload codecs and logical commit phases; M06–07, M10–11, M15, M21 |
| `oam.rs`, `player_oam.rs`, `poly.rs` | OAM cursors and draw data, player animation descriptors, fixed-width poly raster state; M04–06, M12, M21 |
| `hud.rs`, `messaging.rs`, `select_file.rs` | Typed tile/stripe writers, dialogue token cursors, menu/name-entry states, inventory and save-slot views; M01–02, M06, M08, M10–13, M17 |
| `attract.rs`, `ending.rs` | Scene-specific stages and data, explicit draw caches and scratch lifetimes; M05, M12–14 |
| `misc.rs` | Split helpers by ownership/domain; keep pan variants, RNG, item and transition side effects explicit; M04, M07, M14, M16 |
| `assets.rs` | Validated asset directory/ranges and named typed access; M06 |
| `audio.rs`, `spc_player.rs` | Port latches, sample/voice/pattern types and exact mapping codecs; M07, M18–20 |
| `host.rs` | Save-slot domains, typed host commands, input bitsets and configuration normalization; M08, M17, M20, M24 |

For the remaining crates:

- **`z3-core`:** `legacy_memory.rs` is a useful explicit compatibility model;
  keep its little-endian canonical codec. `assets.rs` should own file ranges.
  `config.rs` can separate parsing, bindings, feature flags, and normalized host
  settings. `c_types.rs`/`util.rs` need selective consolidation, not blanket removal.
- **`z3-snes`:** keep `cpu.rs`, `spc.rs`, `bus.rs`, `system.rs`, `apu.rs`,
  `cart.rs`, and `input.rs` accurate behind their device/emulator APIs.
  Improve grouping and register/address types in `ppu.rs`, `dma.rs`, `dsp.rs`;
  retain explicit layout codecs in `saveload.rs`. Accurate device internals are
  an appropriate place for hardware-shaped arithmetic.
- **`z3-native`:** separate the application loop, host command execution, frame
  policy, audio callback, persistence, and rendering policy currently in
  `lib.rs`/`native_app.rs`; keep `z3` and `sdl_smoke` as thin entrypoints.
- **`z3-sdl`:** existing owning wrappers and `TextureLock` are useful foundations.
  Tighten lifetime/error contracts around events, callbacks and surfaces; keep
  raw pointers inside this crate. Decompose GPU/preset code as in M23.
- **`z3-opus`:** the small owning `Decoder` already has the right basic shape.
  Improve error names and buffer/frame contracts without changing decoder or
  OPUZ behavior. Replacing libopus is not needed for gameplay ergonomics.
- **`z3-verify`:** retain `c_oracle.rs` as an independent process boundary and
  `schema.rs` as an explicit wire format. Consolidate repeated runner/reporting
  infrastructure across `bin/`, while retaining independent legacy decoder
  validation in `legacy_snapshot.rs`/`legacy_replay.rs`. Add semantic field
  names to raw-offset diagnostics; never suppress the raw byte difference.

## Hard compatibility cases to isolate rather than erase

1. **Packed-file and static-table overreads.** `background_palettes` intentionally
   exposes bytes after an asset's declared payload. `overworld_load_palettes`
   explicitly handles credits palette 167 using observed C bytes `[0, -49, -67]`.
   `buzz_blob_drop_preserves_adjacent_table_overread_value` documents another
   adjacent-table case. Put these in a named compatibility-quirk layer with
   fixtures; normal typed table APIs must not silently clamp or return zero.
   The static-data case is build-sensitive: record the oracle compiler/link
   configuration before attempting a cross-platform interpretation of it.
2. **Overlapping byte/word fields.** A word store can intentionally change two
   separately named byte fields. Views need a documented alias map. `repr(C)`
   and transmuting WRAM into structs do not solve overlap, alignment or Rust
   borrowing requirements.
3. **Scratch values and nested calls.** Credits, poly rendering, sprite drawing,
   collision probes and spawn-return caches share implicit state. A caller can
   depend on the bytes left by a callee even when its return value looks sufficient.
4. **Transient output.** NMI clears sound-effect bytes before canonical export.
   The recent sound-port/pan fixes demonstrate why end-of-frame memory alone is
   insufficient. Preserve output writes and their timing throughout refactors.
5. **Opaque save contents.** `legacy_snapshot_template` deliberately retains
   historical bytes not modeled as live fields. A clean-looking new serializer
   must not discard them or depend on Rust's in-memory struct layout.
6. **ABI-independent arithmetic.** Increasing integer width, using ordinary
   `abs`, reordering clipping, replacing signed shifts, or using f64 for a
   current f32 audio calculation can change exact results.

## Migration sequence and deliverables

Each step is a small series of reviewable changes, not one rewrite. Keep a
working native host and the same production runtime in the verifier throughout.

### 0. Freeze evidence and improve observability

- Record the C revision, compiler/flags/link setup, asset schema identity,
  runtime configuration and test commands alongside the source revision.
- Add opt-in, read-only boundary diagnostics for memory writes, sound latches,
  device commands, slot allocation and RNG consumption when a refactor needs them.
  Logs must preserve order and not alter simulation state.
- The current audio-state diagnostic calls `copy_variables_to_ram()` on both
  players. That writes mapped SPC RAM. Retain the normal combined replay gate
  without those per-frame diagnostic writes; design a passive codec/export path
  before treating diagnostics as a substitute for production execution.
- Evolve `audit-rust-port.rb` toward reviewed C-to-Rust symbol/behavior mappings.
  Function renaming, consolidation and file movement should update provenance,
  not remove coverage checks or expand a blanket allow-list. Likewise replace
  brittle textual seam checks with explicit capability checks as interfaces move.

Exit: reproducible baseline and diagnostics adequate to explain the first
changed byte, sample, pixel, or side effect.

### 1. Remove ambiguous address and value handling

Start with asset file ranges (the concrete pointer subtraction), typed address
units, typed sound requests, and fixed-width movement helpers. Keep the same
RAM, tables, state ownership and update loops. Extract compatibility quirks into
explicit helpers with evidence, while keeping their exact results.

Exit: selected callers contain domain operations instead of pointer/offset math;
all existing bytes and outputs remain exact.

### 2. Introduce state and entity views

Convert one cohesive field family at a time: player coordinates, sprite
coordinates, ancilla coordinates, then inventory/flags, pool metadata and
collision/map views. Centralize field metadata: width, units, aliases, reset
behavior, serialization, and every known reader/writer. Generate repetitive
accessors/codecs only from that reviewed specification, not guessed C names.

Exit: converted gameplay modules have no raw layout knowledge outside narrowly
named compatibility helpers. Slot order and overlapping writes remain explicit.

### 3. Refactor behavior and organization

Replace numeric states/control-flow fragments with typed state views and explicit
transition results. Move bank-based implementations into per-domain/per-enemy
modules incrementally. Extract common movement, collision, drawing and item
operations only after checking side effects, not just matching return values.

Exit: behavior reads as Rust domain logic, with provenance for merged/renamed
C routines and no changes to state transition timing or incidental writes.

### 4. Simplify ownership and cross-module calls

Split assets, state, graphics effects and audio effects into disjoint context
capabilities. Replace string calls and required no-op defaults. Reduce broad
traits and `RefCell` re-entry scaffolding. Then eliminate scratch VRAM swapping
and diff copies, preserving every existing flush/re-entry boundary.

Exit: mutation and immediate/deferred execution are evident in signatures;
neither host nor verifier has an alternative implementation of gameplay services.

### 5. Move eligible state into owned Rust records

Choose regions with complete access/alias ownership. Switch their authority
once, keeping exact codecs, preserved opaque storage, and a working raw replay
patch route. Start with isolated state, not global scratch or the entire player
and sprite memory image at once. Test save/load and nested callbacks during the
transition, not merely canonical export after a frame.

Exit: domain state owns the converted fields, with lossless legacy interchange
and no stale second copy. Regions still requiring aliases retain typed views.

### 6. Isolate devices and polish the host

Expose graphics/audio services at the game boundary; keep accurate devices as
the compatibility backend. Separate host commands, presentation, shader presets,
and audio ownership. Consider making full-emulator components optional only
after tracing native dependencies and keeping their oracle tests available.

Exit: normal game code does not know WRAM offsets, banked pointers, PPU registers,
or SDL handles. Those details live in clearly named compatibility/device/host
modules, and the same end-to-end parity gates remain mandatory.

## Recommended first changes

1. **Asset ranges and cursors.** Change `z3-core::AssetFile` to retain source
   ranges and let `RuntimeAssets::background_palettes` request a file suffix
   without integer pointer subtraction. Preserve `get()`/packed asset behavior;
   cover odd starts, following payloads/padding, and the credits overread.
2. **Sound port and emitter types.** Replace the touched sword-beam, Ether,
   enemy-hit/stun/shield, and dungeon-tile sound call sites first. Their existing
   regressions are concrete acceptance cases. Expand through remaining raw
   sound writes only after proving overwrite/priority behavior.
3. **Typed position views and fixed-point operations.** Convert a single sprite
   or ancilla movement path without changing pool storage or iteration. Use
   boundary/carry vectors plus the relevant focused C oracle before expanding.
4. **Tile/VRAM unit wrappers.** Migrate one dungeon upload or collision path;
   keep packed bytes inside its encoder and expose domain coordinates to callers.

These changes deliver substantive reductions in memory arithmetic before
attempting large entity, state-machine, or renderer ownership changes.

## Acceptance and verification for every implementation batch

During development, run focused C vectors and the affected full recordings to
locate failures quickly. Before accepting a behavior/storage refactor, require
the full gate, not sampled frames or only the previously failing chapter:

```sh
scripts/test-parity.sh
Z3_VERIFY_AUDIO_STATE=1 scripts/test-full-audio-replay.sh
```

The first command builds the C oracle, checks formatting/source coverage/tests/
lint, runs the device and feature matrices, checks all 13 recordings with exact
state/pixels/PCM, and runs the software-host smoke check. Chapter ranges in
`scripts/test-full-replay.sh` may run in parallel, provided they cover 1–13 in
full using the same build and configuration and every range succeeds.

Additional checks depend on the boundary being changed:

| Change | Additional acceptance evidence |
| --- | --- |
| RAM views, fixed point, field ownership | C input/output vectors; byte/word overlap and immediate visibility tests; boundary values `0`, `0x7f/0x80`, `0xff`, `0x7fff/0x8000`, `0xffff`; preserved scratch/carry effects |
| Slots and entity organization | Allocation exhaustion/reuse, reverse processing order, spawn/despawn during iteration, inactive-slot contents, deterministic RNG consumption |
| Saves/replay/codecs | Existing `.sav` import/export and opaque-byte round trips; patches across WRAM wrap and field aliases; load/reset/music restoration; independent decoder agreement |
| NMI, VRAM, OAM, service ownership | Re-entry and erase-then-rewrite vectors, hidden/off-screen OAM bytes, local/global cursor advancement, Mode 7 and aspect-ratio feature matrix |
| Sound, SPC/DSP, audio ownership | Ordered port/register writes, queue overflow/counter wrap, stereo PCM, mono/resampling cases, callback chunking, and relevant MSU/OPUZ resume tests |
| GPU/shader or FFI lifetime work | Existing host smoke plus representative GPU readback/visual tests for presets, history, formats, resize and filtering; the C replay checker compares PPU pixels before wgpu presentation, so it does not alone certify post-shader output |

New differential generators should use valid domain states or deliberately
specified edge cases, avoiding undefined C accesses and unbounded state-machine
loops. Do not treat a successful hash as a substitute for the existing exact
byte/sample/pixel comparison.

Every batch should state which raw accesses moved behind domain APIs, which
aliases remain, which C behaviors were preserved, and which gates ran. Measure
less layout knowledge and clearer ownership, rather than fewer integer operations
or fewer lines at any cost. Do not commit ROMs, compiled assets, saves, or
ROM-derived fixtures while adding tests.

## Explicit exclusions from this plan

Variable timestep simulation, floating-point physics, changed RNG/order, dynamic
entity compaction, a new sound engine, a different game renderer, gameplay bug
fixes, a new asset authoring format, and a new save/replay format are separate
behavior changes. They may be useful later, but they do not satisfy this exact
C compatibility phase merely by producing a similar-looking game.
