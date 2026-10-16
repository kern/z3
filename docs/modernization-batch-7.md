# Modernization batch 7: recording container ownership

Parent baseline: `fda6eab` (batch 6, pushed to `kern/main`). This bounded M17
increment does not complete the full M01–M24 backlog.

## Outer file codec

`runtime/recording_file.rs` owns the version-1 recorder container: a named
eight-word header, borrowed log/base/current sections and an explicit load mode.
Snapshot payloads remain opaque. Recorder encode/decode methods retain their
public signatures and error path through `runtime::StateFileError`; replay
scheduling, log commands and machine/device snapshot codecs are unchanged.

The compatibility parser remains deliberately more permissive than the
independent verifier. It accepts arbitrary snapshot lengths, raw input words,
positions outside the log and frame counters beyond the recorded total. It
requires the full current payload even when replaying from the base, reports
short headers before unsupported versions, checks length arithmetic in the
same order and ignores trailing bytes at the recorder adapter. Its borrowed
view retains the unconsumed tail without interpreting it.

Raw header inputs narrow only when installed in the recorder. Resume mode
depends on the saved replay frame counter, not on packed position bits. Saving
uses the last completed log position and the original truncating `u32` shift;
it does not save the pending read position. Noncanonical high input bits,
trailing bytes and otherwise inactive cursor bits are still canonicalized or
discarded by recorder decode/encode as before, not newly promised as lossless.

## Independent C grounding

The focused harness calls the unchanged exported C `StateRecorder_Save` and
`StateRecorder_Load` with real initialized devices and `tmpfile()` streams.
It does not replace the C implementations. Because their type is private,
the harness mirrors its declaration; independent source audits compare field
types, names and order against the original and check the shared `ByteArray`
layout. Audit drift probes reject type, name, order, omission and extra-field
changes.

C Save produces the complete file, including its actual machine snapshot.
Rust encoding with that same opaque snapshot must reproduce the entire file.
C Load/Save comparisons cover recorder metadata and payload framing; returned
replay mode and distinct WRAM markers check current/base/reset selection.
This does not claim a new complete machine-import differential or execution of
the synthetic logs. Invalid/truncated acceptance is tested against the frozen
Rust behavior, not against C asserts or undefined reads.

Focused checks pass 321 original-C Save/Load comparisons and 26 independent
real-file comparisons (all 13 reference recordings in both load modes). The
unchanged verifier parses those files independently; production does not use
its stricter decoder. Six new unit tests cover borrowed sections, every prefix
of a small complete container, error precedence/overflow policy, arbitrary
opaque lengths, input narrowing/tail discard, zero-sized base canonicalization,
resume flags and last-complete position encoding.

Two independent inverse reviews confirm that removing only the old codec/error
definitions and new module/reexport declarations leaves the remaining runtime
identical to the parent after whitespace normalization. Scheduler, replay-log
decoder and snapshot/device execution have not moved or changed.

## Acceptance status

The frozen-source full gate exited successfully, including all three complete
recording sweeps and the final native software smoke. Each sweep covered all
13 recordings and 1,073,092 frames. Retained output contains all 13 pixel/PCM
completion lines, 26 audio completion lines and 13 passive confirmation lines.
These are fresh results for this batch, not inherited from its parent.

| Gate | Result |
| --- | --- |
| C entrypoint audit | 3,366 / 3,366 accounted for |
| C recorder ABI source audit / drift probes | Passed |
| C recording-file differential | 321 complete valid-file Save/Load comparisons passed |
| Independent reference-file cross-check | All 13 files in both modes, 26 comparisons passed |
| Workspace tests | 611 passed, including 487 gameplay tests |
| Explicit protocol / failure-path / phrase tests | All passed |
| Formatting / strict all-target Clippy | Passed |
| Existing C purity, SPC schema, component, feature and snapshot checks | All passed |
| All 13 unobserved state/pixel/PCM recordings | Every frame/pixel/PCM value exact |
| All 13 legacy queue/SPC/DSP recordings | Every recorded frame exact |
| All 13 passive queue/SPC/DSP recordings | Every recorded frame exact |
| Native software smoke | 600 frames passed |

```sh
Z3_REPLAY_JOBS=3 scripts/test-parity.sh
```

The gate explicitly runs the three build/asset-dependent tests ignored by the
ordinary workspace suite. GPU code is unchanged; batch 4 records the latest
real Metal/native WGPU checks. Recorded pixel parity covers the game framebuffer
before host post-processing. Original C gameplay/device sources and comparison
masks remain unchanged.

## Remaining M17 scope

The replay command codec and scheduler retain their existing implementation.
Their pending-command timing, partial writes before malformed-log errors,
variable-length wrapping and sequential 17-bit patch writes need a separate
ownership migration. This file extraction does not declare them complete.
