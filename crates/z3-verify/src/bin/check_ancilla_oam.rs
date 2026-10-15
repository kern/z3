//! Full-image comparison of ancilla local/global OAM cursor boundaries.

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

use z3_core::WRAM_SIZE;
use z3_game::{
    ancilla::{Ancilla, AncillaRuntime},
    ram::Ram,
};

type Error = Box<dyn std::error::Error>;
struct Runtime;
impl AncillaRuntime for Runtime {}

#[derive(Clone, Copy, Debug)]
struct Case {
    operation: u8,
    sort: u8,
    index: u16,
    slot: u8,
    frame: u8,
    coordinate: u16,
}

fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for operation in 0..2 {
        for sort in [0, 1, 2, 0xff] {
            for index in 0..=128 {
                cases.push(Case {
                    operation,
                    sort,
                    index,
                    slot: 0,
                    frame: 0,
                    coordinate: 0,
                });
            }
        }
    }
    for sort in [0, 1] {
        for index in [0, 55, 63, 99, 115, 126] {
            for slot in 0..5 {
                for frame in [0, 7] {
                    for coordinate in [0, 0x80, 0x100] {
                        cases.push(Case {
                            operation: 2,
                            sort,
                            index,
                            slot,
                            frame,
                            coordinate,
                        });
                    }
                }
            }
            for frame in 0..55 {
                cases.push(Case {
                    operation: 3,
                    sort,
                    index,
                    slot: 2,
                    frame,
                    coordinate: 0,
                });
            }
        }
    }
    cases
}

fn check(child: &mut Child) -> Result<usize, Error> {
    let mut input = child
        .stdin
        .take()
        .ok_or("missing ancilla OAM oracle stdin")?;
    let mut output = child
        .stdout
        .take()
        .ok_or("missing ancilla OAM oracle stdout")?;
    let mut actual = Box::new([0; WRAM_SIZE]);
    let mut reference = Box::new([0; WRAM_SIZE]);
    let cases = cases();
    for (count, case) in cases.iter().copied().enumerate() {
        for (address, byte) in actual.iter_mut().enumerate() {
            *byte = (address as u8)
                .wrapping_mul(37)
                .wrapping_add((address >> 8) as u8)
                .wrapping_add(count as u8);
        }
        actual[0xfb3] = case.sort;
        // Early returns must retain this unrelated primary/ext pair.
        actual[0x90..0x94].copy_from_slice(&[0xfc, 0x9, 0x21, 0xa]);
        if case.operation >= 2 {
            let primary = 0x800 + case.index * 4;
            actual[0x90..0x92].copy_from_slice(&primary.to_le_bytes());
            actual[0x92..0x94].copy_from_slice(&0xffff_u16.to_le_bytes());
            actual[0x15805..0x1580a].fill(0);
            actual[0x15805 + usize::from(case.slot)] = case.frame;
            actual[0x1580b..0x1580d].copy_from_slice(&case.coordinate.to_le_bytes());
            actual[0x1580d..0x1580f].copy_from_slice(&case.coordinate.to_le_bytes());
            actual[0xe2..0xe4].fill(0);
            actual[0xe8..0xea].fill(0);
            let slot = usize::from(case.slot);
            actual[0xc54 + slot] = 1;
            actual[0xc68 + slot] = 1;
            actual[0xc5e + slot] = case.frame;
        }
        let [low, high] = case.index.to_le_bytes();
        input.write_all(&[case.operation, low, high, case.slot])?;
        input.write_all(actual.as_slice())?;
        input.flush()?;
        let mut returned = [0; 2];
        output.read_exact(&mut returned)?;
        output.read_exact(reference.as_mut_slice())?;
        let expected = usize::from(u16::from_le_bytes(returned));
        let mut ram = Ram::new(&mut actual);
        let mut ancilla = Ancilla {
            ram: &mut ram,
            runtime: &mut Runtime,
        };
        let result = match case.operation {
            0 => ancilla.allocate_oam_from_custom_region(usize::from(case.index)),
            1 => ancilla.hit_stars_update_oam_buffer_position(usize::from(case.index)),
            2 => {
                ancilla.draw_quake_initial_bolts(usize::from(case.slot));
                0
            }
            3 => {
                ancilla.quake_spell_spread_bolts(usize::from(case.slot));
                0
            }
            _ => unreachable!(),
        };
        if result != expected {
            return Err(format!(
                "ancilla OAM return mismatch: {case:?}, C={expected}, Rust={result}"
            )
            .into());
        }
        if let Some(address) = actual
            .iter()
            .zip(reference.iter())
            .position(|(r, c)| r != c)
        {
            return Err(format!(
                "ancilla OAM state mismatch: {case:?}, WRAM {address:#07x}, C={:02x}, Rust={:02x}",
                reference[address], actual[address]
            )
            .into());
        }
    }
    Ok(cases.len())
}

fn main() -> Result<(), Error> {
    let oracle = std::env::args_os()
        .nth(1)
        .ok_or("usage: check_ancilla_oam ANCILLA_OAM_ORACLE")?;
    let mut child = Command::new(oracle)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let result = check(&mut child);
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait()?;
    let count = result?;
    if !status.success() {
        return Err(format!("ancilla OAM oracle exited with {status}").into());
    }
    println!(
        "ancilla OAM cursor policies and Quake drawing match compiled C across {count} complete WRAM images (local entries including one-past, both wrap policies, all spread frames, hidden X bytes, independent extended-pointer advancement)"
    );
    Ok(())
}
