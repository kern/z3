# Modernization batch 6: owned music phrase cursor

Parent baseline: `8b54fe7` (batch 5, pushed to `kern/main`). This is another
bounded M18 increment, not completion of the M01–M24 modernization backlog.

## Persistent cursor ownership

The player's `music_ptr_toplevel` is an owned `PhraseCursor`, not a raw address
temporarily copied into a wrapper. Song selection, control-word consumption,
repeat-destination consumption and jumps use its operations. Its private
position remains a lossless 16-bit value; the legacy schema still encodes the
same little-endian word at `0x40`. The independent C-table audit recognizes
this specific two-byte type and continues checking every field, address,
width and ordering entry.

Control decoding checks the high byte first: all 65,280 nonzero-high-byte words
remain pattern-table addresses. Among low-byte words, zero stops, `0x80` and
`0x81` toggle fast-forward, and all other 253 values remain repeat counts.
Reading a control word advances only that word. The player updates its repeat
counter before separately reading the destination, including when the jump is
not taken. Eight channel pointers are still read and assigned individually.
No RAM contents are cached or borrowed across operations.

The cursor retains the old Rust wrapping read at `0xffff`. This is not claimed
as C parity: the reference word read there is out of bounds. C differential
programs use aligned words at most `0xfffe` and eight-word tables at most
`0xfff0`. Rust-only exhaustive address tests cover the stronger wrap behavior.

## Focused independent checks

Unit tests exhaust control-word classification and cursor positions, check a
fresh destination read after modifying live RAM, and check lossless snapshot
encoding. The focused C harness includes the unchanged `src/spc_player.c` and
calls its actual static music handler; it does not reimplement that handler.
Requests supply separate raw and mapped RAM images. Responses compare complete
raw RAM, passive mapped RAM, the DSP save block and register-write history,
without new masks. Synthetic programs are bounded and do not generate audio.
Full recorded PCM and pixel checks remain separate mandatory gates.

The focused release test passes 65,635 cases, including all 253 repeat codes
crossed with all 256 prior counter values (64,768 cases). It compares 66,135
complete 134,612-byte response snapshots: 8,902,564,620 bytes in total. Extra
vectors cover song selection/reset/loading across three calls, control chains,
pause guards, all channel activity masks with two instrument states,
mid-channel phrase restarts and both taken/fallthrough destination wrapping.

## Acceptance status

The frozen-source gate exited successfully. All three sweeps completed all 13
recordings and 1,073,092 frames each; retained output contains all 13 pixel/PCM
completion lines, 26 audio completion lines and 13 passive confirmation lines.
The final native software smoke also passed. Batch 5's successful results do
not certify these changes; these are fresh results for this batch.

| Gate | Result |
| --- | --- |
| C entrypoint audit | 3,366 / 3,366 accounted for |
| Independent SPC table audit | Same 598 bytes, exact field/address/width/order |
| Workspace tests | 605 passed, including 481 gameplay tests |
| Explicit C phrase differential | 65,635 cases / 66,135 complete snapshots passed |
| Explicit passive protocol and failure-path tests | Both passed |
| Formatting / strict all-target Clippy | Passed |
| Existing C purity, component, feature and snapshot checks | All passed |
| All 13 unobserved state/pixel/PCM recordings | Every frame/pixel/PCM value exact |
| All 13 legacy queue/SPC/DSP recordings | Every recorded frame exact |
| All 13 passive queue/SPC/DSP recordings | Every recorded frame exact |
| Native software smoke | 600 frames passed |

```sh
Z3_REPLAY_JOBS=3 scripts/test-parity.sh
```

The gate explicitly executes the three build/asset-dependent tests ignored by
the ordinary workspace suite. The unchanged GPU readback test remains ignored
there; batch 4 records the latest real Metal/native WGPU checks. Recorded pixel
parity is for the game framebuffer before host post-processing. C gameplay and
device sources are unchanged, and no comparison masks were added or relaxed.

## Remaining scope

Channel pattern cursors, subroutine return/start cursors, and SFX cursors still
use their prior representation. Their next-unread/current-token and lookahead
semantics differ; they must not be folded into this cursor indiscriminately.
Voice/command semantics and broader device/codec separation remain open.
