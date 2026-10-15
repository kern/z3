//! Compares complete WRAM after native Rust and compiled-C movement operations.
//! The oracle accepts an initial image per request and flushes each reply, so
//! verification needs bounded memory and no assets or intermediate trace files.

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

use z3_core::WRAM_SIZE;
use z3_game::{
    ancilla::{Ancilla, AncillaRuntime},
    ram::Ram,
    sprite::{Sprite, SpriteHooks},
    sprite_main_bank0::SpriteMainBank0Runtime,
    sprite_main_bank1::SpriteMainBank1Runtime,
    sprite_main_bank2a::SpriteMainBank2aRuntime,
    sprite_main_bank2b::SpriteMainBank2bRuntime,
};

type Error = Box<dyn std::error::Error>;

struct Hooks;
impl AncillaRuntime for Hooks {}
impl SpriteHooks for Hooks {}
impl SpriteMainBank0Runtime for Hooks {}
impl SpriteMainBank1Runtime for Hooks {}
impl SpriteMainBank2aRuntime for Hooks {}
impl SpriteMainBank2bRuntime for Hooks {}

#[derive(Clone, Copy, Debug)]
struct Case {
    operation: u8,
    slot: u8,
    pixels: u16,
    fraction: u8,
    velocity: i8,
}

impl Case {
    fn initialize(self, bytes: &mut [u8; WRAM_SIZE], seed: u8) {
        // A nonuniform background catches neighboring writes and scratch-byte
        // changes even outside the coordinate arrays selected by the request.
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = (index as u8)
                .wrapping_mul(37)
                .wrapping_add((index >> 8) as u8)
                .wrapping_add(seed);
        }
        let (fraction, low, high, velocity) = match self.operation {
            0 => (0xc40, 0xc04, Some(0xc18), 0xc2c),
            1 => (0xc36, 0xbfa, Some(0xc0e), 0xc22),
            2 => (0x2a8, 0x29e, None, 0x294),
            3 => (0xd70, 0xd10, Some(0xd30), 0xd50),
            4 => (0xd60, 0xd00, Some(0xd20), 0xd40),
            5 => (0xf90, 0xf70, None, 0xf80),
            _ => unreachable!("six movement operations"),
        };
        let slot = usize::from(self.slot);
        bytes[fraction + slot] = self.fraction;
        bytes[low + slot] = self.pixels as u8;
        if let Some(high) = high {
            bytes[high + slot] = (self.pixels >> 8) as u8;
        }
        bytes[velocity + slot] = self.velocity as u8;
    }

    fn run_rust(self, bytes: &mut [u8; WRAM_SIZE]) {
        let slot = usize::from(self.slot);
        let mut hooks = Hooks;
        if self.operation < 3 {
            let mut ram = Ram::new(bytes);
            let mut ancilla = Ancilla {
                ram: &mut ram,
                runtime: &mut hooks,
            };
            match self.operation {
                0 => ancilla.move_x(slot),
                1 => ancilla.move_y(slot),
                2 => ancilla.move_z(slot),
                _ => unreachable!(),
            }
        } else {
            let mut sprites = Sprite::new(bytes, &mut hooks);
            match self.operation {
                3 => sprites.sprite_move_x(slot),
                4 => sprites.sprite_move_y(slot),
                5 => sprites.sprite_move_z(slot),
                _ => unreachable!(),
            }
        }
    }
}

fn cases() -> Vec<Case> {
    let boundaries = [
        (0, 0),
        (0, 15),
        (0xff, 0xff),
        (0x100, 0),
        (0x7fff, 0xf8),
        (0xffff, 0xff),
        (0xffff, 0),
    ];
    let mut cases = Vec::new();
    for operation in 0..6 {
        for slot in [0, if operation < 3 { 9 } else { 15 }] {
            // Every velocity on each axis/family/slot extreme. Cycle boundary
            // positions to exercise fractional carries as well as page wraps.
            for velocity in 0..=u8::MAX {
                let (pixels, fraction) = boundaries[usize::from(velocity) % boundaries.len()];
                cases.push(Case {
                    operation,
                    slot,
                    pixels,
                    fraction,
                    velocity: velocity as i8,
                });
            }
            // At every boundary, explicitly cross zero/minimum/maximum signed
            // velocity with carries/borrows, rather than relying on cycling.
            for (pixels, fraction) in boundaries {
                for velocity in [-128, -1, 0, 1, 127] {
                    cases.push(Case {
                        operation,
                        slot,
                        pixels,
                        fraction,
                        velocity,
                    });
                }
            }
        }
    }
    cases
}

fn check(child: &mut Child) -> Result<usize, Error> {
    let mut input = child.stdin.take().ok_or("missing position oracle stdin")?;
    let mut output = child
        .stdout
        .take()
        .ok_or("missing position oracle stdout")?;
    let mut initial = Box::new([0; WRAM_SIZE]);
    let mut actual = Box::new([0; WRAM_SIZE]);
    let mut reference = Box::new([0; WRAM_SIZE]);
    let cases = cases();
    for (index, case) in cases.iter().copied().enumerate() {
        case.initialize(&mut initial, index as u8);
        actual.copy_from_slice(initial.as_slice());
        input.write_all(&[case.operation, case.slot])?;
        input.write_all(initial.as_slice())?;
        input.flush()?;
        output.read_exact(reference.as_mut_slice())?;
        case.run_rust(&mut actual);
        if let Some(address) = actual
            .iter()
            .zip(reference.iter())
            .position(|(r, c)| r != c)
        {
            return Err(format!(
                "position mismatch in case {index} {case:?} at WRAM {address:#07x}: initial={:02x}, C={:02x}, Rust={:02x}",
                initial[address], reference[address], actual[address],
            ).into());
        }
    }
    Ok(cases.len())
}

fn main() -> Result<(), Error> {
    let oracle = std::env::args_os()
        .nth(1)
        .ok_or("usage: check_positions POSITION_ORACLE")?;
    let mut child = Command::new(oracle)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let result = check(&mut child);
    if result.is_err() {
        // Close/collect the helper on diagnostic or I/O failures as well.
        let _ = child.kill();
    }
    let status = child.wait()?;
    let count = result?;
    if !status.success() {
        return Err(format!("position oracle exited with {status}").into());
    }
    println!(
        "ancilla/sprite XYZ movement matches compiled C across {count} complete WRAM images (all 256 velocities, boundary positions, and both slot extremes)"
    );
    Ok(())
}
