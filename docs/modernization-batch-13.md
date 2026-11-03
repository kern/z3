# Modernization batch 13: attract-mode sprite views

Parent baseline: `3d37ab3` (batch 12, accepted and pushed to `kern/main`).

## Scope

`attract.rs` declared 13 private sprite-pool address constants (Y/X low and
high, graphics, scratch D, kind, flags 2/3, head direction, OAM flags, Z and
object priority). Seventeen raw accesses in the `AttractRuntime` default
coordinate helpers and `sprite_simulate_soldier` now use the named sprite
views, and those constants are removed. `SPRITE_GRAPHICS_INDEX` (`0xaa3`) is
not a pool field and is unchanged, as are all other addresses.

## Verification

`simulated_soldier_sprite_views_match_pre_migration_raw_writes` keeps the exact
pre-migration raw-address implementation as an in-test reference and compares
complete 128 KiB WRAM images after `sprite_simulate_soldier` (which also runs
the migrated default `sprite_set_x`/`sprite_set_y`/`sprite_get_16_bit_coords`)
over six slot indices including out-of-pool offsets 17 and 40, all four
directions, eight flag values around the `== 9` kind selection, and extreme
coordinates/graphics, each on a randomized low-WRAM image. A deliberate
wrong-field mutation was confirmed to fail it. This is a differential against
the previous Rust; the unchanged recording sweeps are the C-grounded check.

The graphics argument stops at 244: both the old and migrated code compute
`[11, 4, 0, 7][dir] + gfx` as a `u8`, which overflows (and panics in debug)
above that. This pre-existing arithmetic is unchanged and deliberately not
exercised; the callers pass 0 or the small `ATTRACT_VAR3` scene counter.

## Acceptance

`Z3_REPLAY_JOBS=3 scripts/test-parity.sh` passed on the frozen source, with the
same documented clippy allow-list as
[batch 11](modernization-batch-11.md#combined-acceptance). A first run failed
only on the test-reference overflow described above (before any recording
ran); the corrected run passed. All 13 recordings matched C state, pixels and
PCM in each of the three sweeps (39 results over 3 x 1,073,092 frames); all
oracle checkers passed. GPU code is unchanged.
