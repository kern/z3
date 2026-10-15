# Modernization batch 5: SPC schema and passive observation

Parent baseline: `d8f1910` (batch 4, pushed to `kern/main`). This receipt tracks
another increment of the full M01–M24 backlog, not its completion.

## SPC RAM mapping (M18)

One ordered typed schema per channel/player now generates live import, live
export and passive mapped-RAM snapshots. The channel schema has 52 fields,
repeated for eight voices with a two-byte stride; the player schema has 67
scalars and three four-byte port arrays. Together they map 598 distinct bytes.
Words use explicit little-endian encoding and arrays remain arrays. Channel
indices, input ports, timer cycles, DSP/history state, last-written EDL and every
unmapped RAM byte retain their independent live values.

The independent `audit-spc-mapping.rb` checks field identity, address, width and
order against the original C tables, including the high-bit width encoding in
the channel table. Generated descriptors exercise region coverage/aliases in
unit tests without introducing a second production field list. The existing
source-entrypoint audit follows the relocated implementations into the child
module; no placeholder methods or broad audit exclusions were added.

The original live operations retain their public signatures and save/load use.
`mapped_ram_snapshot(&self)` clones the existing 64-KiB image and overlays only
mapped fields, never modifying the player. Tests compare the entire player
before/after, the snapshot against live export on a clone, explicit byte/word/
array/channel vectors, all unmapped bytes, import preservation and future
PCM/DSP/echo/history equality under repeated observation. Export/import row and
ordering inverse audits match the parent Rust implementation.

## Passive C oracle diagnostics (M18/M24)

The legacy live-RAM export remains byte-for-byte unchanged. A distinct C API and
stream opcode `0xf3` map a temporary `SpcPlayer` copy and serialize its RAM plus
the unchanged live DSP block. Inline channels/RAM are copied; shallow DSP/history
pointers are never used to generate audio or mutate state. The C purity harness
checks the complete live player, DSP and 75-byte queue before/after repeated and
null exports, verifies mapped values, and confirms legacy export still mutates
RAM while emitting the same mapped payload. A real-process Rust protocol test
checks payload framing, repeated snapshots, queue/canonical-state preservation
and the reserved opcode guard.

`Z3_VERIFY_PASSIVE_AUDIO_STATE=1` adds a distinct post-PCM queue/SPC/DSP comparison
whose Rust observer takes `&NativeRuntime`. It does not replace the legacy flag;
selecting both observation modes is rejected. Normal unobserved state/pixel/PCM
replay and the legacy diagnostic sweep remain required. The new
`test-full-passive-audio-replay.sh` runs every frame of all 13 recordings and is
also mandatory in `test-parity.sh`. `test-recording-sweeps.sh` launches all three
sweeps with isolated observation environments, waits for every child and fails
if any sweep fails. Its regression executes the actual shell source with only
external sweep calls mocked, testing concurrent launch, poisoned ambient modes,
failure of each worker and complete child waits. The comparison still includes the entire
64-KiB mapped image and complete DSP block, with no new masks or sampling.

Review caught a failure-only path still using the legacy mutating detail export.
Both PCM-error branches now select the passive detail operation in passive mode,
while default/legacy mode retains the old behavior. A real-process negative-path
regression checks complete Rust player preservation and C canonical/queue state,
then proves the same helper still maps live RAM in legacy mode. The configured
oracle-process constructor retains the ordinary spawn behavior and permits a
test-specific child working directory without changing the process-global cwd.
The incomplete first gate was explicitly stopped before making these fixes;
its partial recordings and earlier standalone legacy sweep do not certify the
corrected frozen-source gate.

## Effect OAM cursor policies (M12)

Effect-specific sorted/unsorted wrapping policies move into the OAM boundary.
Local cursors stay distinct from the independent global primary/extended words;
early returns leave both globals untouched. New selections retain each caller's
publication order. The two Quake loops use typed local cursors and explicit
global advancement, retaining hidden-entry bytes and wrap timing.

The new C differential passes 2,052 complete-WRAM/return-value cases: 1,032
wrap-policy cases across every local index 0..128 and sort values 0/1/2/255,
360 initial Quake draws and 660 spread draws (all 55 frames, both layouts and
six starts). Vectors cover hidden X retention, last-entry/one-past transitions
and an independent extended pointer of `0xffff`. Two whole-image Quake unit
regressions cover hidden bytes and exact global advancement. The entire
production handler token audit reconstructs the parent after the reviewed
cursor/publication inversions; extracted policies are separately checked against
C. Existing OAM module code is retained, with additions only.

Other effect loops still use local `usize` indices and compatibility drawing
helpers. ASCII region tags, native emitters, sprite-bank pointer mutations and
broader allocation ownership remain open; M12 is not complete.

## Acceptance status

The corrected frozen-source gate exited successfully, including all three
complete recording sweeps and the final native software smoke. Each sweep
covered all 13 recordings and 1,073,092 frames. No earlier batch's sweep
certifies this batch. C gameplay/device sources and existing comparison masks
are unchanged.

| Gate | Result |
| --- | --- |
| C entrypoint audit | 3,366 / 3,366 accounted for |
| Independent SPC table audit | 52 channel fields × 8, 70 player entries, 598 mapped bytes exact |
| Workspace tests | 601 passed, including 477 gameplay tests |
| Explicit real-process protocol / failure-path tests | Both passed; passive observation preserves live state |
| C passive snapshot purity harness | Passed, including full player/DSP/queue preservation |
| Workspace formatting / strict all-target Clippy | Passed |
| Ancilla OAM C differential | 2,052 complete-WRAM / return-value cases passed |
| Existing C feature/component and snapshot checks | All passed |
| All 13 unobserved state/pixel/PCM recordings | Every frame/pixel/PCM value exact |
| All 13 legacy queue/SPC/DSP recordings | Every recorded frame exact |
| All 13 passive queue/SPC/DSP recordings | Every recorded frame exact, without live RAM export |
| Sweep orchestrator regression | Concurrent launch, environment isolation and each worker failure passed |
| Native software smoke | 600 frames passed |

```sh
Z3_REPLAY_JOBS=3 scripts/test-parity.sh
```

The ordinary workspace suite ignores the two asset-dependent process tests;
the gate explicitly runs both. The existing GPU readback test remains ignored
in this suite. GPU implementation and shaders are unchanged in this batch;
batch 4 records their most recent real Metal and native WGPU smoke checks.
Pixel parity above covers the game framebuffer before host post-processing.

## Prepared next cursor increment

The complete behavioral consumers of `music_ptr_toplevel` are song-table
initialization and phrase loading, plus its default/schema encoding at `0x40`.
It is a suitable first owned `PhraseCursor` field, retaining all 65,536 raw
positions losslessly rather than copying a cursor into a local and committing
later. Decode one control word at a time: repeat-count updates precede the
destination-word read, and eight channel pointers are assigned individually.
Preserve every accepted repeat-code byte, not only a selected enum subset.

Other cursors have different meanings: pattern state points to the next unread
byte, SFX scratch points to the current token, and lookahead forks state without
mutating live cursors. High-byte-zero inactivity is not `NonZeroU16` semantics.
No cursor may borrow/cache its RAM source across uploads or DSP/echo writes.
Focused C vectors must distinguish defined wrapping from the original C word
overread at `0xffff`; Rust-only boundary vectors can preserve the current
wrapping behavior without claiming that undefined C access is an oracle proof.
No cursor implementation is counted in this batch.
