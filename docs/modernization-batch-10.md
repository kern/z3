# Modernization batch 10: garnish pools and receipt targets

Parent baseline: `412aa43`, pushed to `kern/main` after all three complete
recording sweeps. This batch passed its complete acceptance gate. It advances
the full M01–M24 plan; that backlog remains open.

## Garnish ownership and caller migration

All fourteen byte-array fields now live behind one canonical-WRAM schema.
Seven caller files replace 218 identified direct field/scalar expressions;
coordinate helpers, reverse allocation and lifetime expiration use cohesive
domain operations. No copied pool state, compaction, implicit reservation or
aggregate spawn initialization is introduced.

Nominal traversal is thirty physical slots split into reverse upper/lower
passes. Raw storage indices remain lossless: a shared overwrite cursor `0x80`
decrements to `0x7f`, not slot 29. The `0xfb4` marker remains an arbitrary byte,
not a derived occupancy count; `0xff8` remains a shared live WRAM alias. Only
the kind byte is cleared on expiration, and zero countdown stays live. Paused
kind 5 retains its special ticking behavior. Boss/Overlord storage is not
claimed as migrated by this Garnish work.

Coordinate writes preserve byte order and hooks observe partially initialized
spawns. The falling-tile pan-hook regression mutates the spawn and controller
coordinates before later reads; they remain live. Lanmolas's low-Y array use
is scratch storage, not a world-coordinate abstraction. Pyramid debris retains
its low-pixel/subpixel arithmetic, untouched high bytes and partial OAM-X exit.

An independent C schema audit covers every field's address/byte width, both
shared scalar aliases and the nominal traversal partition. Drift probes reject
layout changes. The whole-production inverse review reconstructs all seven
caller files from the accepted parent with explicitly enumerated field,
coordinate, countdown, allocation and scalar-alias conversions. The inventory
callback change in the shared native bridge is reviewed separately, not hidden
by excluding that whole file.

## Explicit Garnish C correction

The original `GarnishAllocLimit` reads before checking its signed limit and
returns a negative limit unchanged. The prior Rust empty range returned `-1`.
Actual C vectors with backing bytes zero and 255 reproduced `C=-4, Rust=-1`
without WRAM changes before the correction.

The new guard returns the supplied negative value only for the C backing-WRAM
domain `-0x1f800..=-1`. Below it, prior Rust's `-1` fallback remains; positive
out-of-WRAM indexing retains its bounds panic. Positive allocation paths,
ordinary 14/29 callers and scan/write order remain unchanged. This is an
explicit C-grounded correction, not an unexplained refactor side effect.

The focused suite passed 47,359 original-C cases: 1,746 allocation/coordinate,
37,549 lifecycle/handler (including 2,400 collision) and 8,064 independent-axis
Pyramid cases. Signed returns and all WRAM compare without masks:
6,207,628,284 response bytes. Collision vectors assert hit/miss damage,
incapacitation and both velocity inversions, in addition to whole-WRAM equality.
C fixtures avoid null kind-13 dispatch except expiry and unsafe table states.
Upper-pass flash is inert in these C fixtures; mutable hooks are independently
tested in Rust. Negative-velocity signed-left-shift behavior is retained through
the Rust reference rather than described as portable defined C.

## Receipt inventory targets

The complete domain replaces the 76-row address table with typed
receipt/target/bottle operations and removes the pointer-returning player hook.
There are 41 distinct targets, including intentional aliases. It preserves all
initial values, row order, byte versus word operations, bottle search order,
hook/default behavior, late invalid-item panics and already-published writes.
Default player callbacks historically read WRAM byte zero; using the native
inventory lookup for every runtime would incorrectly change that behavior.
The default semantic query still reads byte zero, while the native query uses
the typed target. Custom query control and short-circuit order have direct
tests. The native chest query is source/schema-reviewed; the full playback gate
provides its integration coverage rather than the custom-runtime unit test.

Eight tables and two pre-batch receipt functions were independently compared
with the parent and pinned as the test-only reference. They do not call the
new target domain. Actual unchanged-C receipt/bottle operations use local
assets for focused tests; those assets are not included in source commits.

The whole-WRAM original-C suite passed 8,924 cases (1,169,686,528 bytes): 912
receipts, 192 dungeon masks, 76 allocation failures and 7,744 bottle profiles.
The Rust reference compares 3,974 receipt/hook cases, 160,000 bottle profiles,
768 default chest cases and 152 custom query cases. Hook comparisons include
full-memory snapshots, not just final return values. Unknown raw IDs retain
their allocation/immobilization prefix and late lookup panic.

## Explicit receipt shift correction

Actual unchanged C validated palace bytes 0–63. Its promoted 32-bit mask shifts
16–31 yield zero; the previous Rust word-width shift panicked first at palace
32 in the debug red test. The explicit correction computes the promoted mask
for counts below 32, while retaining the old Rust expression for counts 32 and
above, where the C expression is not portable defined behavior. Keep target
word read/OR/write after mask computation. The retained Rust domain passed 576
frozen-reference cases in each of debug and release; the release check is also
registered in the permanent full gate.

Both domain implementations, focused reviews and combined full acceptance are
complete. The focused results above are supplemented by the recording gate.

## Acceptance receipt

With source frozen, `Z3_REPLAY_JOBS=3 scripts/test-parity.sh` exited 0:

- All 650 ordinary workspace tests and ten explicitly selected ignored
  C/process tests passed, plus the additional release-mode malformed-palace
  reference check. Formatting and warnings-as-errors Clippy passed.
- Source coverage remains 3,366/3,366. The new independent Garnish and receipt
  schema/drift/reference audits passed alongside all existing audits.
- Independent snapshot/file codecs, all focused C checks, existing feature and
  component checks passed, including aspect ratios and enhanced Mode 7 map.
- All thirteen complete unobserved recordings passed: 1,073,092 frames of exact
  canonical state, every rendered pixel and PCM output.
- Both complete audio-observer contracts passed all thirteen recordings each:
  2,146,184 additional frames comparing queue, mapped SPC and DSP state.
- The final native dummy-video/dummy-audio software smoke passed 600 frames.

Original C gameplay/device source and comparison masks are unchanged. GPU code
is unchanged; the ignored GPU smoke was not rerun. No ROM, assets, saves, traces
or build products are included. Independent reviews found no remaining concrete
issues in the implemented domains. This acceptance does not complete the full
modernization backlog or prove every possible input/feature combination.
