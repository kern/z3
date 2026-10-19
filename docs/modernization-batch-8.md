# Modernization batch 8: owned replay tape and music patterns

Parent baseline: `02d83bb` (batch 7, pushed to `kern/main`). This batch advances
M17 and M18 in parallel; the full M01–M24 backlog remains open.

## Replay tape and scheduler ownership

`runtime/recorder.rs` owns recording and playback. `StateRecorder` holds the
opaque base snapshot and one `RecordedInputs` owner containing log bytes,
input/counter state and the complete six-field `ReplayState`. Old parallel
fields are removed, not copied into temporary wrappers. Callers use immutable
accessors. The outer file codec now lives beneath this owner; public runtime
type/error paths and file methods remain available through reexports.

`replay_log.rs` owns typed positions, lossless raw command bytes, pending
headers, streaming patch metadata and the writer. Header decoding and payload
application stay separate. Invalid raw opcodes are retained until due; a
truncated patch keeps the already-written wrapped WRAM prefix. The scheduler
retains separate pending and last-complete cursor commits. A later header error
may leave a previous patch pending for reapplication on retry. There is no
transactional rollback, eager whole-log validation or second writable cache.

Clearing history retains the original raw-command OR behavior, stale EOF
cursors and ordering of mutations before a possible panic. Replay patches wrap
at 17 bits, while MultiPatch's contiguous-run grouping does not. Existing
unsigned Rust counter/length behavior is retained even where signed C arithmetic
or malformed C reads cannot provide a defined oracle.

The test-only frozen implementation is independently compared with the parent
source and pinned by a normalized SHA-256 audit. Production does not call it.
Differentials compare results, every recorder field, complete log/base bytes
and full WRAM after calls and retries, including malformed inputs. Source
coverage follows the real owner and codec implementations, not this reference.

## Music pattern ownership

Live channel pattern positions use `PatternCursor`; saved return and loop-start
positions use lossless `PatternAddress` bookmarks. All behavioral consumers
move together, including phrase setup, subroutine calls/returns, pitch-slide
checks and activity guards. Nonzero high-byte activity is deliberately not
replaced with nonzero-address semantics. Snapshot fields retain their exact
little-endian words, addresses and ordering in the independently audited schema.

KOF lookahead uses an independent probe. It observes live RAM without publishing
a cursor or changing loop/return state, and follows only a call's destination,
not the live call's full operand sequence. Reads retain wrapping and publication
order. SFX current-token cursors are a different domain and remain unchanged.

## Focused evidence and limits

The scheduler oracle calls the original exported C record/patch/replay/stop/
clear functions. Its 993 cases compare 4,326 operations, including 42 clears:
all nine scalar fields, the raw pending command, returned inputs, complete log
and base bytes, and all 128 KiB of WRAM after every operation. C source audits
cover both harnesses' private recorder declarations. For this scheduler-only
test, an explicit original-C save first canonicalizes the snapshot boundary;
every tested clear then asserts whole WRAM and SPC RAM are unchanged by that
boundary and supplies its actual C base bytes to Rust. No serializer effects
are silently masked. Noncanonical save-boundary behavior is not newly certified
by this focused test.

Malformed logs, signed-C overflow domains and unsafe EOF-clear/stop sequences
are not claimed as defined C proofs. The frozen Rust differential retains
their previous accepted behavior instead. Seven new reference tests compare
full state/WRAM across all raw command bytes, prefixes and retries, timing and
length boundaries, panic mutations, EOF/counter wrapping, and a variable-length
sum wrapping `u32`. The frozen body matches `02d83bb` independently after only
type-name/test-visibility changes and comment/whitespace normalization.

The pattern oracle includes unchanged `src/spc_player.c` and calls its actual
static handlers. All 576,008 cases compare a return byte, full raw and mapped
64-KiB RAM images, complete DSP save block and complete register history:
77,538,164,904 bytes total, without masks. This includes all 65,536 pitch-read
positions across eight voices, subroutine counts/returns, supported effects,
guards, lookahead and byte-wrap edges. C lookahead's `int` cursor must not scan
beyond RAM; Rust-only tests preserve the older Rust wrap behavior there without
claiming C definedness. Existing full phrase vectors also pass unchanged apart
from typed test constructors. Independent inverse reviews reconstruct all
affected SPC method bodies and retain the untouched SFX implementation.

## Acceptance status

Implementation, focused validation and review are complete. The first full
gate exited with failure: macOS killed one normal-replay worker before its first
progress line. The system log explicitly identifies that PID's termination as
a Gatekeeper/AMFI signature rejection, not a parity error. Both audio sweeps and
the surviving normal ranges completed, but this incomplete gate does **not**
certify the batch or its missing normal range.

After runner hardening, a fresh `Z3_REPLAY_JOBS=3 scripts/test-parity.sh`
completed with exit code 0. Game source stayed frozen throughout that run.
Original C gameplay/device sources and comparison masks remain unchanged;
macOS security settings were not relaxed.

The hardened orchestrator builds the verifier once before launching consumers,
checks its signature on Darwin, copies it into a private temporary directory
and checks that copy before all three sweeps use the same executable. Child
runners do not rebuild or replace it. Standalone runners still build by default;
an explicit prebuilt override is validated, and the orchestrator overwrites any
ambient override with its own copy. All modes still execute every frame of all
13 recordings and propagate every worker failure. Cleanup waits for children
and removes only the generated copy/directory, never the Cargo source artifact.
Mock tests exercise the actual shell source, build-before-launch, shared image,
mode isolation, preflight failures, complete child waits and scoped cleanup.
They do not replace a fresh real macOS execution of the full gate.

The fresh acceptance run completed:

- All 622 ordinary workspace tests, formatting and warnings-as-errors Clippy.
  All five explicitly selected ignored C/process tests also passed. The ignored
  GPU test was not run; this batch does not change GPU code or claim a fresh
  Metal/WGPU smoke test.
- All source, SPC mapping, recording ABI and frozen-reference audits, including
  3,366/3,366 C function coverage entries; both runner regression suites.
- The focused scheduler and pattern C differentials described above, the
  existing phrase differential, independent snapshot/file codecs, feature and
  component C checks.
- All 13 complete unobserved recordings: 1,073,092 frames of exact canonical
  state, every rendered pixel and PCM output.
- All 13 recordings separately under each legacy and passive audio observer:
  2,146,184 additional recorded frames, including queue, mapped SPC and DSP
  comparisons. Passive observation never exports into live RAM.
- The final 600-frame native dummy-video/dummy-audio software smoke test.

Independent read-only reviews found no concrete recorder or pattern-cursor
issues. These results certify the implemented batch, not the full backlog or
every possible input. The earlier incomplete run is not counted as acceptance.

## Follow-up work

The SFX audit found a pre-existing in-bounds C/Rust lookup discrepancy: C widens
a sound byte before subtracting one, whereas several Rust sites subtract with
byte wrapping first. Packed port-2/3 commands can install ID zero through the
ordinary audio API. A separately identified correction must compare those
defined C cases without changing positive-ID lookup behavior. This batch leaves
SFX unchanged and does not claim those cases are already equivalent.

Other remaining work includes SFX cursor ownership, live table/upload codecs,
shared voice-state organization, broader player/pool view adoption and the
remaining OAM draw families. The full modernization backlog is not complete.
