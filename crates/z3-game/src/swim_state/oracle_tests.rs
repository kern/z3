//! Bounded, table-index-safe whole-WRAM comparisons against unchanged C.
//! Original C word aliases rely on the pinned target's unaligned accesses.
//! Negative downstream velocity display shifts and invalid table indices are
//! intentionally covered by frozen Rust tests, not claimed as portable C.

use crate::{
    player::{Player, PlayerRuntime},
    ram::Ram,
    swim_state::SwimAxis,
};
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
use z3_core::WRAM_SIZE;

struct Runtime;
impl PlayerRuntime for Runtime {}

fn execute(operation: u8, ram: &mut Ram<'_>) {
    if operation == 0 {
        ram.reset_swim_acceleration();
        return;
    }
    let mut runtime = Runtime;
    let mut player = Player {
        ram,
        runtime: &mut runtime,
        applied_movement_to_camera: false,
    };
    match operation {
        1 => player.link_reset_swimming_state(),
        2 => player.link_flag_max_accels(),
        3 => player.link_set_ice_max_accel(),
        4 => player.link_set_momentum(),
        5 => player.link_set_the_max_accel(),
        6 => player.link_handle_swim_accels(),
        7 => player.handle_swim_stroke_and_subpixels(),
        _ => unreachable!(),
    }
}

struct Oracle {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
    cases: usize,
}

impl Oracle {
    fn new() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut child = Command::new(root.join("build/c-oracle/swim_state_oracle"))
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("build original C swim_state_oracle first");
        Self {
            input: BufWriter::new(child.stdin.take().unwrap()),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            cases: 0,
        }
    }

    fn compare(&mut self, operation: u8, mut bytes: Box<[u8; WRAM_SIZE]>) {
        self.input.write_all(&[operation]).unwrap();
        self.input.write_all(bytes.as_slice()).unwrap();
        self.input.flush().unwrap();
        let mut expected = Box::new([0; WRAM_SIZE]);
        self.output.read_exact(expected.as_mut_slice()).unwrap();
        execute(operation, &mut Ram::new(&mut bytes));
        if let Some(address) = bytes.iter().zip(expected.iter()).position(|(a, b)| a != b) {
            panic!(
                "C swim case{} operation{} WRAM{address:#x}: Rust{:02x}, C{:02x}",
                self.cases, operation, bytes[address], expected[address]
            );
        }
        self.cases += 1;
    }

    fn finish(mut self) {
        drop(self.input);
        assert!(self.child.wait().unwrap().success());
        eprintln!(
            "original C swim axes: {} cases, {} compared WRAM bytes",
            self.cases,
            self.cases * WRAM_SIZE
        );
    }
}

fn image(seed: u8) -> Box<[u8; WRAM_SIZE]> {
    let mut bytes = Box::new([0; WRAM_SIZE]);
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (index.wrapping_mul(17) ^ (index >> 8)) as u8 ^ seed;
    }
    bytes
}

#[test]
fn frozen_rust_swim_steps_preserve_negative_velocity_and_invalid_index_panics() {
    // These include deliberately invalid table indices and downstream signed
    // shifts whose C behavior is not claimed. Compare Rust panic timing and
    // every byte already written, rather than masking or validating values.
    for mode in [0, 1, 2, 3, 0x100, 0xffff] {
        for moving in [0, 1, 2, 3, 255] {
            for selector in [0, 1, 2, 3, 0xffff] {
                for timer in [0, 1, 0x8000, 0xffff] {
                    let mut original = image(moving);
                    let mut ram = Ram::new(&mut original);
                    ram.write_u8(0x34a, moving);
                    ram.write_u8(0x6a, 1);
                    for axis in SwimAxis::FORWARD {
                        ram.set_swim_acceleration_mode(axis, mode);
                        ram.set_swim_hold_timer(axis, timer);
                        ram.set_swim_direction_selector(axis, selector);
                        ram.set_swim_speed(axis, 0x180);
                        ram.set_swim_speed_limit(axis, 0x300);
                    }
                    let mut expected = original.clone();
                    let mut actual = original;
                    let old = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        super::frozen::execute(7, &mut Ram::new(&mut expected))
                    }));
                    let new = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute(7, &mut Ram::new(&mut actual))
                    }));
                    assert_eq!(
                        new.is_err(),
                        old.is_err(),
                        "mode={mode} moving={moving} selector={selector} timer={timer}"
                    );
                    assert_eq!(
                        actual, expected,
                        "mode={mode} moving={moving} selector={selector} timer={timer}"
                    );
                }
            }
        }
    }
    for operation in 0..=6 {
        for seed in 0..32 {
            let mut actual = image(seed);
            let mut expected = actual.clone();
            let old = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                super::frozen::execute(operation, &mut Ram::new(&mut expected))
            }));
            let new = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                execute(operation, &mut Ram::new(&mut actual))
            }));
            assert_eq!(new.is_err(), old.is_err());
            assert_eq!(actual, expected);
        }
    }
    // Invalid Y must panic only AFTER the valid X step has committed; invalid
    // X must panic before Y is touched. This proves the reverse-axis boundary.
    for invalid_axis in SwimAxis::FORWARD {
        let mut actual = image(7);
        let mut ram = Ram::new(&mut actual);
        ram.write_u8(0x34a, 0);
        for axis in SwimAxis::FORWARD {
            ram.set_swim_acceleration_mode(axis, 0);
            ram.set_swim_hold_timer(axis, 1);
            ram.set_swim_direction_selector(axis, 1);
            ram.set_swim_speed(axis, 384);
            ram.set_swim_speed_limit(axis, 768);
        }
        ram.set_swim_acceleration_mode(invalid_axis, 0x100);
        let mut expected = actual.clone();
        let old = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::frozen::execute(7, &mut Ram::new(&mut expected))
        }));
        let new = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute(7, &mut Ram::new(&mut actual))
        }));
        assert!(old.is_err() && new.is_err());
        assert_eq!(actual, expected);
    }
}

#[test]
#[ignore = "requires original C swim_state_oracle"]
fn swim_axes_match_original_c_whole_wram() {
    let mut oracle = Oracle::new();
    // Retention, guard and asymmetric input cases for every operation.
    for operation in 0..=6 {
        for seed in 0..16 {
            let mut bytes = image(seed);
            bytes[0x34a] = seed % 3;
            bytes[0x34f] = seed % 2;
            bytes[0xf0] = seed;
            oracle.compare(operation, bytes);
        }
    }
    // Every joypad combination, both axes, moving-table selection and every
    // overlapping-direction condition. Initial word high bits remain arbitrary.
    for joy in 0..16 {
        for moving in 0..=2 {
            for direction in [0, 3, 12, 15] {
                for previous in [0, 3, 12, 15] {
                    let mut bytes = image(joy);
                    bytes[0xf0] = joy;
                    bytes[0x34a] = moving;
                    bytes[0x67] = direction;
                    bytes[0x340] = previous;
                    oracle.compare(4, bytes);
                }
            }
        }
    }
    for mode in [0, 1, 2, 3, 0x100, 0xffff] {
        for latch in [0, 1, 0xffff] {
            for speed in [0, 239, 240, 0xffff] {
                for limit in [0, 239, 240, 0xffff] {
                    for joy in [0, 1, 2, 3, 4, 8, 12, 15] {
                        let mut bytes = image(joy);
                        let mut ram = Ram::new(&mut bytes);
                        ram.write_u8(0x34a, 0);
                        ram.write_u8(0x34f, 0);
                        ram.write_u8(0xf0, joy);
                        for axis in SwimAxis::FORWARD {
                            ram.set_swim_acceleration_mode(axis, mode);
                            ram.set_swim_acceleration_latch(axis, latch);
                            ram.set_swim_speed(axis, speed);
                            ram.set_swim_speed_limit(axis, limit);
                        }
                        oracle.compare(5, bytes);
                    }
                }
            }
        }
    }
    let edges = [
        0u16, 1, 127, 128, 239, 240, 383, 384, 0x7fff, 0xff60, 0xffff,
    ];
    for speed in edges {
        for limit in edges {
            for joy in 0..16 {
                let mut bytes = image(joy);
                let mut ram = Ram::new(&mut bytes);
                ram.write_u8(0xf0, joy);
                ram.set_swim_speed(SwimAxis::Vertical, speed);
                ram.set_swim_speed(SwimAxis::Horizontal, limit);
                ram.set_swim_speed_limit(SwimAxis::Vertical, limit);
                ram.set_swim_speed_limit(SwimAxis::Horizontal, speed);
                oracle.compare(6, bytes);
            }
        }
    }
    // Reverse-axis stroke step, signed countdown/sum branches, u16 cap and
    // diagonal slowdown. Selectors deliberately preserve non-boolean values
    // 2/3 on Y while staying within C's actual four-entry lookup table.
    for mode in 0..=3 {
        for moving in 0..=2 {
            for timer in [0, 1, 0x7fff, 0x8000, 0xffff] {
                for speed in [0, 1, 7, 8, 12, 239, 240, 383, 384, 0x7fff, 0x8000, 0xffff] {
                    for limit in [0, 1, 240, 384, 0xffff] {
                        for diagonal in [0, 1] {
                            let mut bytes = image(mode as u8);
                            let mut ram = Ram::new(&mut bytes);
                            ram.write_u8(0x34a, moving);
                            ram.write_u8(0x6a, diagonal);
                            ram.write_u8(0x6b, 0);
                            for axis in SwimAxis::FORWARD {
                                ram.set_swim_acceleration_mode(axis, mode);
                                ram.set_swim_hold_timer(axis, timer);
                                ram.set_swim_speed(axis, speed);
                                ram.set_swim_speed_limit(axis, limit);
                            }
                            // Asymmetric axes force mixed signed-sum/countdown
                            // branches and expose shared-direction write order.
                            ram.set_swim_acceleration_mode(SwimAxis::Horizontal, 3 - mode);
                            ram.set_swim_hold_timer(SwimAxis::Horizontal, timer.wrapping_add(1));
                            ram.set_swim_speed(SwimAxis::Horizontal, speed.wrapping_add(17));
                            ram.set_swim_speed_limit(SwimAxis::Horizontal, limit ^ 0x100);
                            ram.write_u8(0x46c, if diagonal == 0 { 0 } else { 4 });
                            ram.set_swim_direction_selector(SwimAxis::Vertical, 1 + mode % 3);
                            ram.set_swim_direction_selector(SwimAxis::Horizontal, 1);
                            oracle.compare(7, bytes);
                        }
                    }
                }
            }
        }
    }
    oracle.finish();
}
