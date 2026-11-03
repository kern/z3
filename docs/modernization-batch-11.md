# Modernization batch 11: live SPC data and shared controller workspaces

Parent baseline: `dc325d5`, pushed to `kern/main` after its complete acceptance
gate. Combined acceptance passed (see below). This is not a claim that all
M01–M24 work is complete.

## Live audio data

Named instrument, DSP-preset, note, pan and song/SFX-table operations now own
SPC table layout. They borrow canonical RAM on demand. Six static arrays are
compared independently with unchanged C; instruments and presets retain their
overlapping live storage. No decoded table cache or parallel audio state is
introduced. Both pan sides read afresh, including the wrapped right-pan value
and the original live-RAM fallback beyond the static curve.

A lossless channel-pattern-table descriptor replaces the bare table address.
The caller still reads and immediately publishes each of eight voices in
sequence. No prefetch across voice initialization is introduced. Independent
Rust tests cover all 65,536 table addresses and eight sequential reads,
including wrapping words and live mutation.

An incremental upload decoder separates framing from ordered RAM commits.
The three DSP mute/key-off writes still occur before any parse error. Earlier
complete blocks survive a malformed later block; an incomplete current block
writes nothing. A two-byte zero length terminates immediately, ignoring the
tail; successful state resets remain after termination. Destination writes
wrap at sixteen bits. Malformed and odd-length input is compared against the
verbatim accepted Rust algorithm, not C's pointer-only unbounded API.

The independent baseline compares the entire player for 1,238 upload inputs,
not just returned errors or RAM. Root independently compared and pinned the
frozen function against `dc325d5`. Full-production inverse review reconstructs
all three changed interpreter files, verifies all six moved constants, and
enumerates the descriptor/read timing and decoder-private-position changes.

### Explicit C-grounded pan correction

An actual C-first regression completed the original C operation before Rust
panicked in debug: pan `0x15ff`, fallback bytes 0 and 255, final volume 255.
Original C promotes its byte subtraction and product to `int`; 255 × 255 fits
that type but not the former Rust `i16` product. The explicit correction uses
`i32` intermediates, retaining the byte-narrowing point and DSP write order.
The old release-mode truncation happened to yield the same final byte; this
fix removes an accepted-input debug panic, not a deliberate PCM change.
Negative right shifts follow the pinned target compiler's arithmetic behavior.

The focused original-C data suite compares raw RAM, passive mapped RAM, all
DSP bytes, complete register-history arrays/count, nonmapped scalar/index
fields, return bytes and public PCM after each operation. It covers all
instrument and preset IDs, ownership/percussion/voice profiles, every pan
word, phase/volume boundaries, overlapping live mutations, valid aligned
uploads, wrapped destinations and maximum 65,534-byte aligned payloads.
Four independently audible uploaded-BRR fixtures each generate three frames.
C upload fixtures are complete, source-aligned and nonaliased; malformed
source lengths and source aliasing are not claimed as C coverage.

Focused post-correction debug and release runs both passed: 89,945 cases,
90,041 complete snapshots, 12,314,277,283 compared response bytes per run.
The minimal regression also passed in both profiles. The finalized protocol
requires a successful child-process exit as well as every full response.

## Controller and boss workspaces

The nominal eight-slot controller pool and explicitly aliased boss workspaces
use canonical WRAM. Raw storage indices remain lossless;
an eight-slot gameplay traversal is not an eight-slot storage clamp. Controller
coordinate lanes share storage with Armos/Ganon/Arrghus orbit output, compact
Arrghi positions, odd-aligned Helmasaur words and Trinexx phase/activation bytes.
Select-file's separate phase aliases remain independent live users.

The closed schema covers ten byte lanes (including spawned area), two-byte
source-position entries and 38 explicitly overlapping boss interpretations.
It replaces 327 identified expressions in seven storage callers. A separate
native gameplay bridge is unchanged. Coordinates, reverse free-slot search,
kind-only clearing and room-clear queries now delegate domain operations;
finding a free slot does not reserve or initialize it. Room clearing still
blocks only types 20 and 24, not all nonzero controller types.

The full-production inverse review reconstructs all eight reviewed files
from the accepted parent, including the unchanged bridge, allowing only the
enumerated owner conversions, cohesive operation folds, dead declarations
and the separately documented sprite initialization correction. Root read
and ran that review. An independent closed C/schema audit checks widths,
addresses, order, aliases, traversal bounds and drift rejection. Four raw
boss-reference functions and five constants were independently compared with
the accepted parent and pinned; they do not call the production workspace.

Expanded original-C whole-WRAM stream verification runs alongside the
existing six-case Overlord oracle. It passed 183,242 C-first cases:
24,018,628,392 complete return-plus-WRAM response bytes, without masks.
Real local assets initialize the original C engine; native Rust uses the
production services. Sine, spawning, tile, random and sound hooks are not
stubbed. The child must exit successfully after all responses compare.

Coverage includes raw coordinates through index 255; independent X/Y activity
edges and loaded-bit sentinel/boundaries; captured-kind dispatch after live
kind clearing; all 256 raw kinds in non-dispatch queries, initialization and
loads; reverse/full allocation and stale payloads; every valid dispatcher kind
and all timer bytes; explicit per-slot Stalfos/snake/bomb triggers; all valid
falling paths and flying-tile rows; real spawn success/failure/count limits;
paused/reverse execution; knight guards and clear order; signed Armos rotation
with source/output overlap; Ganon/Arrghus orbit helpers and nested Arrghi prep.

Fixtures never dispatch unused kind 4, zero or unknown kinds. Table selectors
stay within actual C backing arrays; the last falling-path command is index
108 and flying-tile rows are 0–21. Loaded-block tests reach the final backing
WRAM byte but never beyond it. Zoro uses real indoor tile lookup; outdoor
tile lookup is not claimed by this focused checker. Arrghus selectors stay in
their defined table domain; a distinct adjacent-byte sentinel checks the
speed-byte interpretation. Raw invalid C table selectors, all device state
and framebuffer output are not claimed by these helper cases. The mandatory
recording gate supplies the integration state/pixel/PCM coverage.

Original gameplay/device C, masks and
expected recording results remain unchanged. No ROM or derived assets belong
in this batch's source commit.

### Explicit C-grounded sprite initialization correction

The expanded C-first stream reproduced a pre-existing branch mismatch: a
state-10 sprite of type `0xec` in a different room remained active in C but
was cleared by Rust. C's nested state-10 branch exempts types `0xec` and
`0xd2` from the subsequent room filter; the flat Rust branch accidentally
fell through into it. The explicitly authorized correction gates the room
branch with `state != 10`. Other states and the already-published state-10
clears remain ordered as before. Focused C fixtures cover all sixteen slots,
all 256 raw types and matching/mismatching rooms.

## Combined acceptance

`Z3_REPLAY_JOBS=3 scripts/test-parity.sh` passed on the frozen source with one
documented deviation. Rust 1.95 clippy rejects the unchanged parent `dc325d5`
with 103 errors in three style lints (`explicit_counter_loop`,
`collapsible_match`, `manual_checked_ops`), none in code introduced here. The
gate was run with exactly those three lints allowed and `-D warnings` otherwise
kept; workspace clippy is clean under that allow-list. `scripts/test-parity.sh`
itself is unchanged; the lint drift is a separate follow-up.

All thirteen complete recordings matched C state, pixels and PCM in each of the
three sweeps (unobserved playback, legacy and passive audio observers): 39
passing results over 3 × 1,073,092 frames. All oracle checkers, including
`check_overlord_state`, passed. GPU code is unchanged; a new GPU run is not
claimed.
