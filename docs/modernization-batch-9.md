# Modernization batch 9: swimming axes and SFX ownership

Parent baseline: `2dc8f79`, pushed to `kern/main` after the complete batch-8
parity gate passed. This batch passed its complete acceptance gate. The full
M01–M24 modernization backlog remains open.

## Swimming state

Replace direct two-axis address arithmetic with live, named swimming/momentum
views. Both axes retain six little-endian word families, including odd-aligned
words and overlapping low-byte access. Direction selectors, modes and latches
remain lossless words, not booleans or closed enums. The vertical selector can
legally select additional entries of the original direction table.

All reads and writes use canonical WRAM, not a copied `SwimState`. Forward and
reverse scans remain explicit. The ten-word acceleration reset and four-word
blocked-axis reset are different operations with their original write order.
Direction words survive acceleration resets. `0x32e` is the high byte of the
horizontal mode word, not padding; `0x333` is outside these families. Dungeon's
low-byte mode clear must not become a whole-word clear.

An independent schema audit checks all six pointer aliases against unchanged
`src/variables.h` and exercises rejection of width, address and field-set drift.
Focused original-C tests and whole-WRAM alias/reset tests are required before
acceptance. Undefined C table accesses and signed-negative left shifts cannot
be used as portable C proofs. The original C also casts odd addresses to word
pointers: bounded fixture comparisons establish behavior of the pinned native
oracle build, not portable ISO C alignment guarantees.

Focused swimming validation has passed 12,320 original-C cases, comparing
1,614,807,040 WRAM bytes without masks. It exercises seven reset/acceleration/
momentum operations and bounded stroke processing. Stroke fixtures retain
table-safe nonzero selectors; negative-output and invalid-index behavior is
instead compared against twelve frozen pre-batch Rust methods, including panic
status and every already-mutated byte. Their independently reviewed bodies and
local table constant are pinned by the schema/reference audit.

The inverse review reconstructs all five production caller files at `2dc8f79`
after reversing named field accesses, explicit axis scans and the four ordered
reset helpers. Only whitespace and Rustfmt's trailing call commas are ignored.
The identified 76 direct access sites and four bulk loops are migrated; this
does not complete player modernization outside this state family.

## SFX: explicit parity repair before refactoring

An independent temporary probe used the actual public C and Rust audio APIs:
initialization, commands `0x40`, `0x80`, `0xc0` on ports 2 and 3, and twelve
sample-generation frames per case. Both received identical bounded synthetic
RAM. In C, sound bytes are promoted before subtracting one for several table
lookups. The previous Rust code wrapped subtraction in a byte first, choosing
a different in-bounds table entry for sound ID zero.

All six public cases started with identical state. Port 2 first diverged at
frame 2 in mapped echo state (`0x3e2`: C `01`, Rust `00`). By frame 3, all six
cases had C current/saved pointers `0x4000` versus Rust `0x5000`. Both synthetic
targets terminate silently, so their DSP blocks matched: this evidence proves
reachable **state divergence**, not an observed PCM defect.

Before extraction, the permanent public-API regression reproduced the same
port-2 frame-2 failure. After only the explicit lookup correction, all six
twelve-frame cases passed raw/mapped RAM, DSP, register-history and PCM
comparisons. The expanded final harness additionally checks interpreter fields
outside the RAM mapping. The combined refactor also passed the final gate.

The correction must explicitly widen indexing before subtraction. Positive
IDs retain their prior lookup addresses. This is a C-grounded bug fix, not an
unexplained behavior change hidden in a refactor. Other suspected arithmetic
differences are not authorized by this proof and must not be folded in without
separate evidence.

The six Rust expressions cover eight original C sites (shared port-2/3 helpers
account for the difference). Their corrected addresses are:

| Lookup | Address for byte ID `s` |
| --- | --- |
| Port-1 loop/restart word | `0x17be + 2 * u16(s)` |
| Port-1 chained byte | `0x17ff + usize(s)` |
| Port-2 start word | `0x181e + 2 * usize(s)` |
| Port-3 start word | `0x191a + 2 * usize(s)` |
| Port-2 echo flag | `0x18dc + usize(command & 0x3f)` |
| Port-2 chained byte | `0x189d + usize(s)` |
| Port-3 chained byte | `0x1999 + usize(s)` |

For positive IDs these equal the old addresses algebraically; for zero they
select the C predecessor entry rather than the old wrapped-byte entry. Port-3
echo indexing has no subtraction and is unchanged. Channel-allocation matching
and sample arithmetic are not part of this correction.

The persistent SFX current-token cursor and saved positions now have their own
types, and eleven cohesive interpreter/allocation methods live in `sfx.rs`.
Music patterns' next-unread convention is not reused. Reads remain live, each
prefix increment publishes immediately, End-token early returns do not publish
a channel bookmark, and all raw word representations retain their save mapping.
An independent inverse audit reconstructs all eleven methods against the
parent plus only the six documented indexing corrections. Every other
production token in the parent module is unchanged after reversing module/type/
default declarations. This separates the bug fix from mechanical extraction.

The final static C differential passed 569,136 cases, including 524,288
pitch-slide operand cases across all 65,536 cursor positions and eight voices.
Each 136,763-byte response compares return value, full raw/mapped RAM, complete
DSP save block, history, fourteen nonmapped semantic bytes and PCM: exactly
77,836,746,768 compared bytes without masks. Other vectors exercise continuation,
duration/volume grammar, preset/loop chains, positive and zero sound IDs,
allocation/echo, all port-1 commands and countdown/recovery branches. Public
sample generation separately passed all six commands over 72 complete frames.
All-position coverage is specifically the bounded pitch-slide program, not
every possible SFX program. Silent synthetic programs do not prove an audible
defect in the old implementation.

## Acceptance receipt

With source frozen, `Z3_REPLAY_JOBS=3 scripts/test-parity.sh` exited 0:

- All 630 ordinary workspace tests and eight explicitly selected ignored
  C/process tests passed, with formatting and warnings-as-errors Clippy.
- Source coverage remains 3,366/3,366; SPC schema, swimming schema/frozen
  reference, recorder ABI/frozen reference and runner regression audits passed.
- Independent snapshot/file codecs, all focused C checks, existing component
  checks and enhanced-feature scenarios passed, including aspect ratios and
  the 4× enhanced Mode 7 map.
- All thirteen complete unobserved recordings passed: 1,073,092 frames of
  exact canonical state, every rendered pixel and PCM output.
- Both full audio-observer contracts passed all thirteen recordings separately:
  2,146,184 additional frames, including queue, mapped SPC and DSP checks.
- The final native dummy-video/dummy-audio software smoke test passed 600 frames.

Original C gameplay/device source and comparison masks are unchanged. GPU code
is unchanged; the ignored GPU smoke test was not rerun in this batch. Independent
reviews found no remaining concrete swimming/SFX issues. The runner's executable
mode is restored; no ROM, assets, saves, traces or build products are included.
These results certify this batch, not completion of the full modernization plan.
