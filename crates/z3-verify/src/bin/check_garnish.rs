//! Asset-free, unmasked whole-WRAM Garnish differential against original C.
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use z3_core::WRAM_SIZE;
use z3_game::{
    ram::Ram,
    sprite::{Sprite, SpriteHooks},
    sprite_main_bank0::{SpriteMainBank0, SpriteMainBank0Runtime},
    sprite_main_bank1::SpriteMainBank1Runtime,
    sprite_main_bank2a::SpriteMainBank2aRuntime,
    sprite_main_bank2b::SpriteMainBank2bRuntime,
};

type Error = Box<dyn std::error::Error>;
struct Hooks;
impl SpriteHooks for Hooks {}
impl SpriteMainBank0Runtime for Hooks {}
impl SpriteMainBank1Runtime for Hooks {}
impl SpriteMainBank2aRuntime for Hooks {}
impl SpriteMainBank2bRuntime for Hooks {}

#[derive(Clone, Copy, Debug)]
struct Case {
    op: u8,
    slot: u8,
    value: u16,
    limit: i32,
}
impl Case {
    fn op(op: u8) -> Self {
        Self {
            op,
            slot: 0,
            value: 0,
            limit: 29,
        }
    }
}
struct Checker {
    input: ChildStdin,
    output: ChildStdout,
    expected: Box<[u8; WRAM_SIZE]>,
    actual: Box<[u8; WRAM_SIZE]>,
    count: usize,
}
impl Checker {
    fn compare(&mut self, initial: &[u8; WRAM_SIZE], case: Case, red: bool) -> Result<(), Error> {
        self.input.write_all(&[case.op, case.slot])?;
        self.input.write_all(&case.value.to_le_bytes())?;
        self.input.write_all(&case.limit.to_le_bytes())?;
        self.input.write_all(initial)?;
        self.input.flush()?;
        let mut result = [0; 4];
        self.output.read_exact(&mut result)?;
        self.output.read_exact(self.expected.as_mut_slice())?;
        let expected_result = i32::from_le_bytes(result);
        self.actual.copy_from_slice(initial);
        let mut hooks = Hooks;
        let mut ram = Ram::new(&mut self.actual);
        let actual_result = if case.op < 8 {
            let mut bank = SpriteMainBank0 {
                ram: &mut ram,
                runtime: &mut hooks,
            };
            match case.op {
                0 => bank.garnish_alloc_force() as i32,
                1 => bank.garnish_alloc(),
                2 => bank.garnish_alloc_low(),
                3 => bank.garnish_alloc_limit(case.limit),
                4 => bank.garnish_alloc_overwrite_old_low() as i32,
                5 => bank.garnish_alloc_overwrite_old() as i32,
                6 => {
                    bank.garnish_set_x(usize::from(case.slot), case.value);
                    0
                }
                7 => {
                    bank.garnish_set_y(usize::from(case.slot), case.value);
                    0
                }
                _ => unreachable!(),
            }
        } else {
            let mut sprite = Sprite::new(&mut self.actual, &mut hooks);
            match case.op {
                8 => sprite.garnish_execute_single(usize::from(case.slot)),
                9 => sprite.garnish_execute_upper_slots(),
                10 => sprite.garnish_execute_lower_slots(),
                11 => sprite.garnish13_pyramid_debris(usize::from(case.slot)),
                _ => unreachable!(),
            }
            0
        };
        if self.expected[..] != self.actual[..] {
            let at = self
                .expected
                .iter()
                .zip(self.actual.iter())
                .position(|(a, b)| a != b)
                .unwrap();
            return Err(format!(
                "case {} {case:?}: WRAM {at:05x}: C={:02x}, Rust={:02x}",
                self.count, self.expected[at], self.actual[at]
            )
            .into());
        }
        if red {
            if self.actual[..] != initial[..] {
                return Err("negative-limit RED unexpectedly mutated WRAM".into());
            }
            if expected_result != -4 || actual_result != -1 {
                return Err(format!("negative-limit RED expected C=-4 Rust=-1, got C={expected_result} Rust={actual_result}").into());
            }
            println!("confirmed defined negative-limit RED: C=-4 Rust=-1; complete WRAM unchanged");
        } else if expected_result != actual_result {
            return Err(format!(
                "case {} {case:?}: result C={expected_result}, Rust={actual_result}",
                self.count
            )
            .into());
        }
        self.count += 1;
        Ok(())
    }
}

fn patterned() -> Box<[u8; WRAM_SIZE]> {
    let mut ram = Box::new([0; WRAM_SIZE]);
    for (address, byte) in ram.iter_mut().enumerate() {
        *byte = (address as u8)
            .wrapping_mul(37)
            .wrapping_add((address >> 8) as u8);
    }
    ram
}
fn word(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn lifecycle() -> Box<[u8; WRAM_SIZE]> {
    let mut ram = patterned();
    // Patterned untouched bytes expose accidental clearing. Only fixture inputs
    // are initialized: no assets, collisions or external tile-update effects.
    ram[0x1f800..0x1f81e].fill(0);
    ram[0xfb4] = 31;
    ram[0xff9] = 0; // Original HandleScreenFlash is a no-op for this C suite.
    ram[0x11] = 0;
    ram[0xfc1] = 0;
    ram[0xfb3] = 0;
    ram[0xfc6] = 0;
    ram[0x1b] = 0;
    ram[0x1a] = 0;
    ram[0x31f] = 1; // countdown_for_blink: no player collision side effects.
    word(&mut ram[..], 0xe2, 0);
    word(&mut ram[..], 0xe8, 0);
    word(&mut ram[..], 0x90, 0x830);
    word(&mut ram[..], 0x92, 0xa2c);
    for (i, base) in [0x30u16, 0x1d0, 0, 0x30, 0x120, 0x140].iter().enumerate() {
        word(&mut ram[..], 0xfe0 + i * 2, *base);
        word(&mut ram[..], 0xfec + i * 2, 0);
    }
    for slot in 0..30 {
        for base in [
            0x1f896, 0x1f8b4, 0x1f8d2, 0x1f8f0, 0x1f92c, 0x1f94a, 0x1f968, 0x1f9fe, 0x1f85a,
            0x1f878,
        ] {
            ram[base + slot] = 0;
        }
        ram[0x1f81e + slot] = 80;
        ram[0x1f83c + slot] = 64;
        ram[0x1f90e + slot] = 10;
    }
    ram
}

fn run(checker: &mut Checker, red: bool) -> Result<(), Error> {
    if red {
        let mut ram = patterned();
        for backing in [0, 255] {
            ram[0x1f800 - 4] = backing;
            checker.compare(
                &ram,
                Case {
                    limit: -4,
                    ..Case::op(3)
                },
                true,
            )?;
        }
        return Ok(());
    }
    let mut ram = patterned();
    // The left side of C's while condition reads these bytes even for negative
    // limits; either byte value must return the unchanged signed input.
    for limit in [-0x1f800i32, -0x1f7ff, -4, -1] {
        for backing in [0, 255] {
            ram[(0x1f800 + limit) as usize] = backing;
            checker.compare(
                &ram,
                Case {
                    limit,
                    ..Case::op(3)
                },
                false,
            )?;
        }
    }
    // Allocation: full/empty, every free slot, alternating pools, raw limits.
    for mask in 0..35 {
        for slot in 0..128 {
            ram[0x1f800 + slot] = match mask {
                0 => 1,
                1 => 0,
                2 => (slot & 1) as u8,
                3 => ((slot + 1) & 1) as u8,
                _ => u8::from(slot != mask - 4),
            };
        }
        for op in [0, 1, 2, 4, 5] {
            checker.compare(&ram, Case::op(op), false)?;
        }
        for limit in [
            -0x1f800, -0x1f7ff, -4, -1, 0, 1, 14, 15, 28, 29, 30, 127, 0x7ff,
        ] {
            checker.compare(
                &ram,
                Case {
                    limit,
                    ..Case::op(3)
                },
                false,
            )?;
        }
    }
    for full in [true, false] {
        ram[0x1f800..0x1f81e].fill(1);
        if !full {
            ram[0x1f800 + 7] = 0;
        }
        for seed in 0..=255u8 {
            ram[0xff8] = seed;
            for op in [4, 5] {
                checker.compare(&ram, Case::op(op), false)?;
            }
        }
    }
    for slot in [0, 14, 15, 29, 30, 127, 255] {
        for value in [0, 0xff, 0x100, 0x7fff, 0x8000, 0xffff] {
            for op in [6, 7] {
                checker.compare(
                    &ram,
                    Case {
                        slot,
                        value,
                        ..Case::op(op)
                    },
                    false,
                )?;
            }
        }
    }
    println!(
        "Garnish allocation/coordinates: {} full-WRAM cases",
        checker.count
    );
    // Every raw slot is legal for the early empty path; no 30-slot validation.
    for slot in [0, 14, 15, 29, 30, 127, 255] {
        let mut ram = lifecycle();
        ram[0x1f800 + usize::from(slot)] = 0;
        checker.compare(
            &ram,
            Case {
                slot,
                ..Case::op(8)
            },
            false,
        )?;
    }
    // Exercise every safe dispatcher entry and valid table range, all nominal
    // slots and A/D/F. Decimal13 is NULL C: only expiry is called below.
    for kind in 1..=22u8 {
        for pause in [0, 1] {
            for count in [0, 1, 2, 8, 9, 10, 15, 16, 24, 31] {
                let effective = if (pause == 0 || kind == 5) && count != 0 {
                    count - 1
                } else {
                    count
                };
                let expires = effective == 0 && count != 0 && (pause == 0 || kind == 5);
                let safe = match kind {
                    3 => effective <= 39 && effective != 30,
                    5 | 7 | 14 => effective <= 31,
                    8 | 20 => effective <= 11,
                    10 | 18 => effective <= 15,
                    11 => effective <= 23,
                    12 => effective != 80 && effective <= 191,
                    13 => false,
                    16 => effective >= 8,
                    22 => effective <= 31,
                    _ => true,
                };
                if !safe && !expires {
                    continue;
                }
                for slot in 0..30u8 {
                    for region in 0..3 {
                        let mut ram = lifecycle();
                        ram[0x1f800 + usize::from(slot)] = kind;
                        ram[0x1f90e + usize::from(slot)] = count;
                        ram[0x1f9fe + usize::from(slot)] = if kind == 15 { 7 } else { 0 };
                        ram[0x11] = pause;
                        ram[0xfb3] = u8::from(region != 0);
                        ram[0x1f968 + usize::from(slot)] = u8::from(region == 2);
                        checker.compare(
                            &ram,
                            Case {
                                slot,
                                ..Case::op(8)
                            },
                            false,
                        )?;
                    }
                }
            }
        }
    }
    // Whole-pass gate/order/FA0/expiry outcomes: all slots share valid trail
    // inputs but distinct low coordinates and countdowns; no filtered traversal.
    for op in [9, 10] {
        for marker in [0, 1, 5, 19, 31, 255] {
            for pause in [0, 1] {
                for region in 0..3 {
                    let mut ram = lifecycle();
                    ram[0xfb4] = marker;
                    ram[0x11] = pause;
                    ram[0xfb3] = u8::from(region != 0);
                    for slot in 0..30 {
                        ram[0x1f800 + slot] = [0, 1, 5, 13][slot % 4];
                        // NULL13 must expire before dispatch, even when paused.
                        if slot % 4 == 3 && pause != 0 {
                            ram[0x1f800 + slot] = 0;
                        }
                        ram[0x1f90e + slot] = if slot % 4 == 3 {
                            1
                        } else {
                            (slot % 3 + 1) as u8
                        };
                        ram[0x1f83c + slot] = (slot * 7) as u8;
                        ram[0x1f968 + slot] = u8::from(region == 2);
                    }
                    checker.compare(&ram, Case::op(op), false)?;
                }
            }
        }
    }
    // Table selectors, countdown transition branches and offscreen preparation
    // at independently varied coordinates. Avoid external tile-update trigger
    // countdown30 for kind3 and80 for kind12; live-hook tests cover that seam.
    for kind in 1..=22u8 {
        if kind == 13 {
            continue;
        }
        for variant in 0..4u8 {
            for (x, y) in [(64, 80), (255, 255), (256, 80), (64, 256), (0xffff, 0)] {
                let mut ram = lifecycle();
                ram[0x1f800] = kind;
                ram[0x11] = 1; // no decrement; precise valid table selectors.
                ram[0x1f90e] = match kind {
                    5 | 7 | 14 => variant * 8,
                    8 | 20 => (variant % 3) * 4,
                    10 => (variant % 2) * 8,
                    11 => (variant % 3) * 8,
                    16 => 8 + variant * 8,
                    18 => variant * 4,
                    22 => 16 + variant * 4,
                    _ => 8,
                };
                ram[0x1f9fe] = match kind {
                    4 => variant % 2,
                    15 => 7 + variant,
                    _ => variant,
                };
                ram[0x1f92c] = if kind == 22 { variant } else { variant % 4 };
                word(&mut ram[..], 0xe2, 0);
                word(&mut ram[..], 0xe8, 0);
                ram[0x1f83c] = x as u8;
                ram[0x1f878] = (x >> 8) as u8;
                ram[0x1f81e] = y as u8;
                ram[0x1f85a] = (y >> 8) as u8;
                checker.compare(&ram, Case::op(8), false)?;
            }
        }
    }
    // Actual LightningTrail -> Garnish_CheckPlayerCollision path. All writes
    // are canonical WRAM, so exercise hitbox edges and every early guard with
    // the unchanged C handler, rather than replacing the collision callback.
    for slot in [0, 7, 29u8] {
        for dx in [-13i16, -12, 0, 11, 12] {
            for dy in [-23i16, -22, 0, 5, 6] {
                for guard in 0..4 {
                    for source in 0..8 {
                        let mut ram = lifecycle();
                        let index = usize::from(slot);
                        ram[0x1f800 + index] = 9;
                        ram[0x1f90e + index] = 8;
                        ram[0x1f92c + index] = source;
                        ram[0x31f] = u8::from(guard == 2);
                        ram[0x37b] = u8::from(guard == 3);
                        ram[0x1a] = if guard == 1 { slot ^ 1 } else { slot };
                        word(&mut ram[..], 0x22, (64i16 + dx) as u16);
                        word(&mut ram[..], 0x20, (64i16 + dy) as u16);
                        ram[0x4d] = 0;
                        ram[0x46] = 0;
                        ram[0x373] = 0;
                        ram[0x27] = 0x35;
                        ram[0x28] = 0xc7;
                        checker.compare(
                            &ram,
                            Case {
                                slot,
                                ..Case::op(8)
                            },
                            false,
                        )?;
                        let hit = guard == 0 && (-12..12).contains(&dx) && (-22..6).contains(&dy);
                        let expected = if hit {
                            [1, 16, 16, 0xca, 0x38]
                        } else {
                            [0, 0, 0, 0x35, 0xc7]
                        };
                        let observed =
                            [0x4d, 0x46, 0x373, 0x27, 0x28].map(|at| checker.expected[at]);
                        if observed != expected {
                            return Err(format!("Lightning collision fixture slot={slot} dx={dx} dy={dy} guard={guard}: expected {expected:?}, C returned {observed:?}").into());
                        }
                    }
                }
            }
        }
    }
    let before_pyramid = checker.count;
    // Every positive signed velocity (C-defined shift), fraction carry and both
    // clip thresholds. Direct handler preserves raw high bytes and partial X.
    for velocity in 0..=127u8 {
        for (x, y) in [
            (64, 80),
            (247, 80),
            (248, 80),
            (255, 80),
            (64, 239),
            (64, 240),
            (64, 255),
        ] {
            for fraction in [0, 15, 255] {
                for (vx, vy) in [(velocity, 127 - velocity), (velocity, 0), (0, velocity)] {
                    let mut ram = lifecycle();
                    ram[0x1f800] = 19;
                    ram[0x1f83c] = x;
                    ram[0x1f81e] = y;
                    ram[0x1f896] = vy;
                    ram[0x1f8b4] = vx;
                    ram[0x1f8d2] = fraction;
                    ram[0x1f8f0] = 255 - fraction;
                    ram[0x1f85a] = 0x91;
                    ram[0x1f878] = 0xe3;
                    checker.compare(&ram, Case::op(11), false)?;
                }
            }
        }
    }
    println!(
        "Garnish lifecycle/handlers: {before_pyramid} cumulative cases; Pyramid: {} independent-axis cases",
        checker.count - before_pyramid
    );
    Ok(())
}

fn finish(mut child: Child, result: Result<(), Error>) -> Result<(), Error> {
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait()?;
    result?;
    if !status.success() {
        return Err(format!("Garnish oracle exited {status}").into());
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let red = std::env::args().any(|arg| arg == "--expect-negative-red");
    let mut child = Command::new("build/c-oracle/garnish_oracle")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut checker = Checker {
        input: child.stdin.take().ok_or("missing oracle stdin")?,
        output: child.stdout.take().ok_or("missing oracle stdout")?,
        expected: Box::new([0; WRAM_SIZE]),
        actual: Box::new([0; WRAM_SIZE]),
        count: 0,
    };
    let result = run(&mut checker, red);
    let count = checker.count;
    drop(checker);
    finish(child, result)?;
    println!(
        "Garnish original-C comparison passed: {count} cases; full WRAM + signed return, no masks"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undefined_c_negative_limits_retain_prior_rust_fallback() {
        let mut bytes = patterned();
        let original = bytes.clone();
        let mut ram = Ram::new(&mut bytes);
        let mut hooks = Hooks;
        let bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        for limit in [i32::MIN, -0x1f801] {
            assert_eq!(bank.garnish_alloc_limit(limit), -1);
        }
        assert_eq!(bytes, original);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn undefined_c_positive_limit_retains_rust_bounds_panic() {
        let mut bytes = patterned();
        let mut ram = Ram::new(&mut bytes);
        let mut hooks = Hooks;
        let _ = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        }
        .garnish_alloc_limit(0x800);
    }
}
