# Modernization batch 12: ending/credits sprite views

Parent baseline: `951fc09` (batch 11, accepted and pushed to `kern/main`).

## Scope

`ending.rs` declared 25 private `SPRITE_*` address constants and used them in
about 250 raw `read_u8`/`write_u8`/`read_i8`/`write_i8` accesses plus the local
`inc8`/`dec8`/`add8` helpers. They bypassed the live sprite views that every
other sprite caller already uses. All of those accesses now go through the
named `SpriteField` getters/setters; the constants and nothing else are
removed. Slot indices are still unclamped raw offsets (for example `+ 7 + j`),
which the views index the same way, so addressing and byte widths are
unchanged. Signed writes use `set_signed_field`.

`INTRO_SPRITE_ALLOC` (the OAM allocation word, not sprite-pool storage) and all
non-sprite addresses are unchanged. No gameplay behavior, ordering or panic
condition changes in this batch.

## Verification

`ending/frozen.rs` is a verbatim copy of the parent `ending.rs` (tests module
omitted). `ending/frozen_tests.rs` runs all 84 public entry points of both
implementations from identical whole-WRAM images with an identical logging
runtime (non-empty map/text/offset tables, deterministic random and palette
callbacks) and requires equal WRAM, runtime logs (calls, PPU writes, sprite
service calls, draw requests) and panic outcomes/messages. Argument-free entry
points sweep module, submodule and sub-submodule bytes (3 random images x 40 x
24); entry points with arguments sweep slot/value arguments over 12 random
images. Two deliberate mutations (wrong field, wrong slot offset) were confirmed
to fail the test.

This test is a differential against the previous Rust, not against C; the
ending has no C oracle. The unchanged recording sweeps remain the C-grounded
integration check.

## Acceptance

`Z3_REPLAY_JOBS=3 scripts/test-parity.sh` passed on the frozen source, with the
same documented deviation as [batch 11](modernization-batch-11.md#combined-acceptance):
clippy ran with `explicit_counter_loop`, `collapsible_match` and
`manual_checked_ops` allowed (they also fail the unchanged parent under
rustc 1.95) and `-D warnings` otherwise. All 13 recordings matched C state,
pixels and PCM in each of the three sweeps (39 results over 3 x 1,073,092
frames); all oracle checkers passed. GPU code is unchanged.
