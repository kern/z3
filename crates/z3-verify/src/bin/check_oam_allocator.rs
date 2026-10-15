//! Asset-free, complete-WRAM comparison of OAM allocation against compiled C.

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

use z3_core::WRAM_SIZE;
use z3_game::{
    oam::{REGION_INITIAL_BASES, REGION_LIMITS},
    sprite::{Sprite, SpriteHooks},
    sprite_main_bank0::SpriteMainBank0Runtime,
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
    operation: u8,
    selector: u8,
    count: u8,
    base: u16,
    counter: u16,
}

fn cases() -> Vec<Case> {
    let mut result = Vec::new();
    for region in 0..6 {
        let limit = REGION_LIMITS[region];
        let bases = [
            0,
            REGION_INITIAL_BASES[region],
            limit - 1,
            limit,
            limit + 1,
            0xff00,
            0xffff,
        ];
        for count in 0..=u8::MAX {
            result.push(Case {
                operation: 0,
                selector: (region * 2) as u8 + (count & 1),
                count,
                base: bases[usize::from(count) % bases.len()],
                counter: u16::from(count) * 257,
            });
        }
        for base in bases {
            for counter in [0, 1, 7, 8, 0xfffe, 0xffff] {
                for count in [0, 1, 4, 255] {
                    result.push(Case {
                        operation: 0,
                        selector: (region * 2) as u8,
                        count,
                        base,
                        counter,
                    });
                }
            }
            for count in [0, 1, 4, 255] {
                result.push(Case {
                    operation: 1,
                    selector: region as u8,
                    count,
                    base,
                    counter: 7,
                });
            }
        }
    }
    for counter in [0, 7, 0xfffe, 0xffff] {
        result.push(Case {
            operation: 2,
            selector: 0,
            count: 0,
            base: 0xffff,
            counter,
        });
    }
    result
}

fn check(child: &mut Child) -> Result<usize, Error> {
    let mut input = child
        .stdin
        .take()
        .ok_or("missing OAM allocator oracle stdin")?;
    let mut output = child
        .stdout
        .take()
        .ok_or("missing OAM allocator oracle stdout")?;
    let mut initial = Box::new([0; WRAM_SIZE]);
    let mut actual = Box::new([0; WRAM_SIZE]);
    let mut reference = Box::new([0; WRAM_SIZE]);
    let cases = cases();
    for (index, case) in cases.iter().copied().enumerate() {
        for (address, byte) in initial.iter_mut().enumerate() {
            *byte = (address as u8)
                .wrapping_mul(37)
                .wrapping_add((address >> 8) as u8)
                .wrapping_add(index as u8);
        }
        let region = usize::from(if case.operation == 1 {
            case.selector
        } else {
            case.selector >> 1
        });
        initial[0xfe0 + region * 2..0xfe2 + region * 2].copy_from_slice(&case.base.to_le_bytes());
        initial[0xfec + region * 2..0xfee + region * 2]
            .copy_from_slice(&case.counter.to_le_bytes());
        // These intentionally differ from an ordinary primary/extended pair.
        // Reset must preserve them; allocation must publish its own two values.
        initial[0x90..0x94].copy_from_slice(&[0xfc, 0xff, 0x34, 0x12]);
        actual.copy_from_slice(initial.as_slice());
        input.write_all(&[case.operation, case.selector, case.count])?;
        input.write_all(initial.as_slice())?;
        input.flush()?;
        let mut expected_result = [0];
        output.read_exact(&mut expected_result)?;
        output.read_exact(reference.as_mut_slice())?;
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut actual, &mut hooks);
        let result = match case.operation {
            0 => sprites.oam_get_buffer_position(case.count, case.selector),
            1 => match case.selector {
                0 => sprites.oam_allocate_from_region_a(case.count),
                1 => sprites.oam_allocate_from_region_b(case.count),
                2 => sprites.oam_allocate_from_region_c(case.count),
                3 => sprites.oam_allocate_from_region_d(case.count),
                4 => sprites.oam_allocate_from_region_e(case.count),
                5 => sprites.oam_allocate_from_region_f(case.count),
                _ => unreachable!(),
            },
            2 => {
                sprites.oam_reset_region_bases();
                0
            }
            _ => unreachable!(),
        };
        if result != expected_result[0] {
            return Err(format!(
                "OAM allocation return mismatch case {index} {case:?}: C={:02x}, Rust={result:02x}",
                expected_result[0]
            )
            .into());
        }
        if let Some(address) = actual
            .iter()
            .zip(reference.iter())
            .position(|(r, c)| r != c)
        {
            return Err(format!("OAM allocation state mismatch case {index} {case:?} at WRAM {address:#07x}: C={:02x}, Rust={:02x}", reference[address], actual[address]).into());
        }
    }
    Ok(cases.len())
}

fn main() -> Result<(), Error> {
    let oracle = std::env::args_os()
        .nth(1)
        .ok_or("usage: check_oam_allocator OAM_ALLOCATOR_ORACLE")?;
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
        return Err(format!("OAM allocator oracle exited with {status}").into());
    }
    println!(
        "OAM allocation/reset matches compiled C across {count} complete WRAM images (all byte counts, six regions, paired selectors, thresholds and counter/base wrap)"
    );
    Ok(())
}
