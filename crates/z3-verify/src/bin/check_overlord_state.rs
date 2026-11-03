//! Asset-backed, original-entrypoint Overlord differential, complete WRAM.
use std::{
    io::{Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
use z3_core::WRAM_SIZE;
use z3_game::{
    assets::RuntimeAssets, integration::NativeGameServices, overlord, ram::Ram, sprite::Sprite,
    sprite_main_bank1::SpriteMainBank1, sprite_main_bank2a::SpriteMainBank2a,
    sprite_main_bank2b::SpriteMainBank2b,
};
type Error = Box<dyn std::error::Error>;
#[derive(Clone, Copy, Debug)]
struct Case {
    operation: u8,
    slot: u8,
    value: u16,
    source: [u8; 3],
}
impl Case {
    fn op(operation: u8) -> Self {
        Self {
            operation,
            slot: 0,
            value: 0,
            source: [0; 3],
        }
    }
}
struct Checker<'a> {
    assets: &'a RuntimeAssets<'a>,
    input: ChildStdin,
    output: ChildStdout,
    expected: Box<[u8; WRAM_SIZE]>,
    actual: Box<[u8; WRAM_SIZE]>,
    count: usize,
}
impl Checker<'_> {
    fn check(&mut self, initial: &[u8; WRAM_SIZE], case: Case, context: &str) -> Result<(), Error> {
        self.input.write_all(&[case.operation, case.slot])?;
        self.input.write_all(&case.value.to_le_bytes())?;
        self.input.write_all(&case.source)?;
        self.input.write_all(&[0])?;
        self.input.write_all(initial)?;
        self.input.flush()?;
        // Observe the whole actual C response before invoking native Rust.
        let mut returned = [0; 4];
        self.output.read_exact(&mut returned)?;
        self.output.read_exact(self.expected.as_mut_slice())?;
        self.actual.copy_from_slice(initial);
        let mut hooks = NativeGameServices::new(self.assets);
        let mut ram = Ram::new(&mut self.actual);
        let slot = usize::from(case.slot);
        let result = match case.operation {
            0 => {
                overlord::set_x(&mut ram, slot, case.value);
                u32::from(overlord::get_x(&ram, slot))
            }
            1 => {
                overlord::set_y(&mut ram, slot, case.value);
                u32::from(overlord::get_y(&ram, slot))
            }
            2 => {
                u32::from(overlord::get_x(&ram, slot))
                    | u32::from(overlord::get_y(&ram, slot)) << 16
            }
            3 => {
                overlord::check_if_active(&mut ram, slot);
                0
            }
            4 => {
                overlord::execute_single(&mut ram, &mut hooks, slot);
                0
            }
            5 => {
                overlord::execute_all(&mut ram, &mut hooks);
                0
            }
            6 => {
                overlord::main(&mut ram, &mut hooks);
                0
            }
            7 => u32::from(
                Sprite::new(ram.bytes_mut(), &mut hooks).sprite_check_if_overlords_clear(),
            ),
            8 => {
                Sprite::new(ram.bytes_mut(), &mut hooks).sprite_initialize_slots();
                0
            }
            9 => {
                Sprite::new(ram.bytes_mut(), &mut hooks).sprite_disable_all();
                0
            }
            10 => {
                Sprite::new(ram.bytes_mut(), &mut hooks).dungeon_load_single_overlord(&case.source);
                0
            }
            11 => {
                overlord::armos_coordinator_rotate(&mut ram, &hooks, slot);
                0
            }
            12 => {
                overlord::armos_coordinator_rotate_knights(&mut ram, &hooks, slot);
                0
            }
            13 => u32::from(overlord::armos_coordinator_check_knights(&ram)),
            14 => {
                overlord::armos_coordinator_disable_coercion(&mut ram);
                0
            }
            15 => {
                overlord::armos_coordinator_bounce(&mut ram, &mut hooks, slot);
                0
            }
            16 => overlord::tile_room_spawn_tile(&mut ram, &mut hooks, slot)
                .map_or(u32::MAX, |s| s as u32),
            17 => {
                overlord::spawn_falling_tile(&mut ram, &mut hooks, slot);
                0
            }
            18 => {
                overlord::spawn_cannon_ball(&mut ram, &mut hooks, slot, case.value as i16);
                0
            }
            19 => {
                overlord::spawn_boulder(&mut ram, &mut hooks);
                0
            }
            20 => {
                SpriteMainBank2a {
                    ram: &mut ram,
                    runtime: &mut hooks,
                }
                .ganon_handle_fire_bat_circle(slot);
                0
            }
            21 => {
                SpriteMainBank2b {
                    ram: &mut ram,
                    runtime: &mut hooks,
                }
                .arrghus_handle_puffs(slot);
                0
            }
            22 => {
                SpriteMainBank1 {
                    ram: &mut ram,
                    runtime: &mut hooks,
                }
                .sprite_prep_arrghi(slot);
                0
            }
            _ => unreachable!(),
        };
        if result != u32::from_le_bytes(returned) {
            return Err(format!(
                "{context} case{} {case:?}: return C={:#x} Rust={result:#x}",
                self.count,
                u32::from_le_bytes(returned)
            )
            .into());
        }
        if self.expected[..] != self.actual[..] {
            let at = self
                .expected
                .iter()
                .zip(self.actual.iter())
                .position(|(a, b)| a != b)
                .unwrap();
            return Err(format!(
                "{context} case{} {case:?}: WRAM{at:05x} C={:02x} Rust={:02x}",
                self.count, self.expected[at], self.actual[at]
            )
            .into());
        }
        self.count += 1;
        Ok(())
    }
}
fn word(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn xy(ram: &mut [u8], slot: usize, x: u16, y: u16) {
    ram[0xb08 + slot] = x as u8;
    ram[0xb10 + slot] = (x >> 8) as u8;
    ram[0xb18 + slot] = y as u8;
    ram[0xb20 + slot] = (y >> 8) as u8;
}
fn base(profile: u8) -> Box<[u8; WRAM_SIZE]> {
    let mut ram = Box::new([0; WRAM_SIZE]);
    // Unrelated patterned tail detects accidental clearing without inventing
    // malformed table selectors in the actual dynamic sprite/tile hooks.
    for at in 0x18000..WRAM_SIZE {
        ram[at] = (at as u8).wrapping_mul(37).wrapping_add((at >> 8) as u8);
    }
    ram[0x1b] = 1;
    word(&mut ram[..], 0x22, 0x100);
    word(&mut ram[..], 0x20, 0x100);
    word(&mut ram[..], 0xe2, 0x100);
    word(&mut ram[..], 0xe8, 0x100);
    ram[0xfb0] = 1;
    ram[0xfb1] = 1;
    ram[0x2f] = 0;
    ram[0x12000..0x14000].fill(0x82);
    for slot in 0..8 {
        xy(&mut ram[..], slot, 0x100, 0x100);
        word(&mut ram[..], 0xb48 + slot * 2, 0xffff);
    }
    for slot in 0..16 {
        ram[0xdd0 + slot] = match profile {
            0 => 0,
            1 => 9,
            2 => u8::from(slot != 7) * 9,
            _ => u8::from(slot < 5) * 9,
        };
        ram[0xe20 + slot] = if profile == 4 { 0x10 } else { 0x8f };
    }
    if profile == 1 {
        ram[0x1f800..0x1f81e].fill(1);
    }
    ram
}
fn initialization(checker: &mut Checker<'_>) -> Result<(), Error> {
    // Initialize/disable inspect raw kind bytes but never dispatch them.
    // Include unknown kinds without redefining the controller enum/domain.
    for kind in 0..=255u8 {
        for room in [0x34, 0x35] {
            let mut ram = base(0);
            ram[0x40a] = 0x34;
            for slot in 0..8 {
                ram[0xb00 + slot] = kind;
                ram[0xcca + slot] = room;
            }
            checker.check(&ram, Case::op(8), "initialize all raw controller kinds")?;
            checker.check(&ram, Case::op(9), "disable all raw controller kinds")?;
        }
    }
    // A state10 EC/D2 sprite bypasses room filtering in the original nested
    // branch. Prove every type and every physical slot, not just one red case.
    for slot in 0..16 {
        for kind in 0..=255u8 {
            for room in [0x34, 0x35] {
                let mut ram = base(0);
                ram[0x40a] = 0x34;
                ram[0xdd0 + slot] = 10;
                ram[0xe20 + slot] = kind;
                ram[0xc9a + slot] = room;
                checker.check(&ram, Case::op(8), "state10 all types/slots/room branches")?;
            }
        }
    }
    for state in [0, 1, 9, 10, 255] {
        for kind in [0, 0x6c, 0xec, 0xd2, 0xff] {
            for room in [0x34, 0x35] {
                let mut ram = base(0);
                ram[0x40a] = 0x34;
                for slot in 0..16 {
                    ram[0xdd0 + slot] = state;
                    ram[0xe20 + slot] = kind;
                    ram[0xc9a + slot] = room;
                }
                for slot in 0..8 {
                    ram[0xb00 + slot] = slot as u8;
                    ram[0xcca + slot] = room;
                }
                checker.check(&ram, Case::op(8), "initialize slots")?;
                for indoors in [0, 1] {
                    ram[0x1b] = indoors;
                    checker.check(&ram, Case::op(9), "disable all")?;
                }
            }
        }
    }
    Ok(())
}
fn run(checker: &mut Checker<'_>) -> Result<(), Error> {
    initialization(checker)?;
    let mut ram = base(0);
    // All raw byte indices, including physical aliases beyond nominal8 slots.
    for (at, byte) in ram.iter_mut().enumerate() {
        *byte = (at as u8).wrapping_mul(37).wrapping_add((at >> 8) as u8);
    }
    for slot in 0..=255u8 {
        for value in [0, 0xff, 0x100, 0x7fff, 0x8000, 0xffff] {
            for operation in [0, 1] {
                checker.check(
                    &ram,
                    Case {
                        slot,
                        value,
                        ..Case::op(operation)
                    },
                    "raw coordinates",
                )?;
            }
        }
        checker.check(
            &ram,
            Case {
                slot,
                ..Case::op(2)
            },
            "raw coordinate read",
        )?;
    }
    for slot in [0, 7, 8, 15, 127, 255u8] {
        for frame in [0, 1] {
            for indoors in [0, 1] {
                for scroll in [0u16, 0x100, 0xffff] {
                    for (dx, dy) in [
                        (-65i16, -65i16),
                        (-64, -64),
                        (-1, -1),
                        (0, 0),
                        (255, 255),
                        (256, 256),
                        (303, 303),
                        (304, 304),
                        (305, 305),
                        (-65, 0),
                        (-64, 0),
                        (303, 0),
                        (304, 0),
                        (305, 0),
                        (0, -65),
                        (0, -64),
                        (0, 303),
                        (0, 304),
                        (0, 305),
                    ] {
                        for block in [0xffff, 0, 7, 8, 0xfff, 0x83ff] {
                            let mut ram = base(0);
                            ram[0x1b] = indoors;
                            ram[0x1a] = frame;
                            ram[0xb00 + usize::from(slot)] = 23;
                            word(&mut ram[..], 0xe2, scroll);
                            word(&mut ram[..], 0xe8, scroll);
                            xy(
                                &mut ram[..],
                                usize::from(slot),
                                scroll.wrapping_add(dx as u16),
                                scroll.wrapping_add(dy as u16),
                            );
                            word(&mut ram[..], 0xb48 + usize::from(slot) * 2, block);
                            if block != 0xffff {
                                ram[0x1ef80 + usize::from(block >> 3)] = 0xff;
                            }
                            checker.check(
                                &ram,
                                Case {
                                    slot,
                                    ..Case::op(3)
                                },
                                "active scroll/loaded-bit",
                            )?;
                        }
                    }
                }
            }
        }
    }
    for slot in 0..8u8 {
        for kind in [1, 7, 23] {
            let mut ram = base(0);
            ram[0x1b] = 0;
            ram[0xb00 + usize::from(slot)] = kind;
            word(&mut ram[..], 0xe2, 0);
            word(&mut ram[..], 0xe8, 0);
            word(&mut ram[..], 0x22, 0x7000);
            word(&mut ram[..], 0x20, 0x7000);
            xy(&mut ram[..], usize::from(slot), 0x7000, 0x7000);
            ram[0xdd0] = 4;
            word(&mut ram[..], 0xb48 + usize::from(slot) * 2, 8);
            checker.check(
                &ram,
                Case {
                    slot,
                    ..Case::op(4)
                },
                "captured kind after active clear",
            )?;
        }
    }
    for kind in 0..=255u8 {
        for slot in 0..8 {
            let mut ram = base(0);
            ram[0xb00 + slot] = kind;
            checker.check(&ram, Case::op(7), "all raw room-clear kinds")?;
        }
    }
    for profile in 0..6 {
        for kind in 0..=255u8 {
            for source in [
                [0, 0, kind],
                [0x7f, 0x80, kind],
                [0x80, 0x1f, kind],
                [0xff, 0xff, kind],
            ] {
                let mut ram = base(0);
                ram[0xfb0] = kind;
                ram[0xfb1] = kind.wrapping_neg();
                ram[0x40a] = 0x73;
                for slot in 0..8 {
                    ram[0xb00 + slot] = match profile {
                        0 => 0,
                        1 => 1,
                        2 => u8::from(slot != 0),
                        3 => u8::from(slot != 3),
                        4 => u8::from(slot != 7),
                        _ => (slot & 1) as u8,
                    };
                    ram[0xb28 + slot] = 0xa5;
                    ram[0xb30 + slot] = 0x3c;
                    ram[0xb38 + slot] = 0xc7;
                }
                checker.check(
                    &ram,
                    Case {
                        source,
                        ..Case::op(10)
                    },
                    "load all raw types/reverse allocation",
                )?;
            }
        }
    }
    println!(
        "Overlord coordinates/active/initialize/load: {} cases",
        checker.count
    );
    // Every timer byte, every valid dispatch kind; controller slot cycles over
    // all8, and real allocators see empty/full/one-free/count-limited pools.
    for kind in 1..=26u8 {
        if kind == 4 {
            continue;
        }
        for timer in 0..=255u8 {
            for profile in 0..5 {
                let slot = timer & 7;
                let index = usize::from(slot);
                let mut ram = base(profile);
                ram[0xb00 + index] = kind;
                ram[0xb30 + index] = timer;
                ram[0xb28 + index] = if matches!(kind, 5 | 6 | 26) { timer } else { 0 };
                ram[0xb38 + index] = timer & 1;
                ram[0x1a] = timer;
                ram[0xb9e] = 1;
                ram[0xcf4] = 1;
                checker.check(
                    &ram,
                    Case {
                        slot,
                        ..Case::op(4)
                    },
                    "valid kind/all timers/real spawn",
                )?;
            }
        }
    }
    // Counter-selected slots in the broad timer sweep cannot independently
    // hit these slot-dependent triggers. Prove each real spawn/failure path.
    for kind in [5, 6, 26u8] {
        for slot in 0..8u8 {
            let trigger = if kind == 5 {
                [255u8, 224, 192, 160, 128, 96, 64, 32][usize::from(slot)]
            } else {
                [0x20u8, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90][usize::from(slot)]
            };
            for counter in [0, trigger.wrapping_sub(1), trigger, trigger.wrapping_add(1)] {
                for profile in 0..4 {
                    let mut ram = base(profile);
                    let index = usize::from(slot);
                    ram[0xb00 + index] = kind;
                    ram[0xb28 + index] = counter;
                    ram[0xb9e] = 1;
                    ram[0xcf4] = 1;
                    ram[0xb40 + index] = [0, 1, 255][index % 3];
                    checker.check(
                        &ram,
                        Case {
                            slot,
                            ..Case::op(4)
                        },
                        "slot-specific Stalfos/snake/bomb spawn triggers",
                    )?;
                }
            }
        }
    }
    for kind in 10..=15u8 {
        let offsets = [0u8, 25, 66, 77, 87, 98, 108];
        let path = usize::from(kind - 10);
        for step in 0..=offsets[path + 1] - offsets[path] {
            for profile in [0, 1] {
                let mut ram = base(profile);
                ram[0xb07] = kind;
                ram[0xb2f] = step;
                checker.check(
                    &ram,
                    Case {
                        slot: 7,
                        ..Case::op(4)
                    },
                    "bounded falling path including final entry",
                )?;
            }
        }
    }
    for step in 0..22 {
        for slot in 0..8u8 {
            for profile in 0..4 {
                let mut ram = base(profile);
                ram[0xb28 + usize::from(slot)] = step;
                checker.check(
                    &ram,
                    Case {
                        slot,
                        ..Case::op(16)
                    },
                    "all valid flying-tile rows",
                )?;
                ram[0xb00 + usize::from(slot)] = 20;
                ram[0xb30 + usize::from(slot)] = 129;
                checker.check(
                    &ram,
                    Case {
                        slot,
                        ..Case::op(4)
                    },
                    "tile room increment/expiry/failure policy",
                )?;
            }
        }
    }
    for kind in [3, 7] {
        for state in 0..=255u8 {
            for timer in [0, 1, 32, 128, 160, 255] {
                for profile in [0, 1] {
                    let slot = state & 7;
                    let mut ram = base(profile);
                    ram[0xb00 + usize::from(slot)] = kind;
                    ram[0xb28 + usize::from(slot)] = state;
                    ram[0xb30 + usize::from(slot)] = timer;
                    checker.check(
                        &ram,
                        Case {
                            slot,
                            ..Case::op(4)
                        },
                        "all cannon/floor counter bytes",
                    )?;
                }
            }
        }
    }
    for slot in 0..8u8 {
        for profile in 0..4 {
            let mut ram = base(profile);
            ram[0xb40 + usize::from(slot)] = [0, 1, 255][usize::from(slot) % 3];
            checker.check(
                &ram,
                Case {
                    slot,
                    ..Case::op(17)
                },
                "falling-tile actual allocation/pan",
            )?;
            for direction in 0..4 {
                for offset in [0, 1, 8, 0xfff8, 0x7fff, 0x8000] {
                    for special in [0, 160] {
                        ram[0xfb5] = direction;
                        ram[0xfb6] = special;
                        checker.check(
                            &ram,
                            Case {
                                slot,
                                value: offset,
                                ..Case::op(18)
                            },
                            "cannon direction/signed offset/special/occupancy",
                        )?;
                    }
                }
            }
        }
    }
    for profile in 0..4 {
        for pause in [0, 1, 0xff] {
            for flag in [0, 1] {
                let mut ram = base(profile);
                ram[0x11] = pause;
                ram[0xfc1] = flag;
                for slot in 0..8 {
                    ram[0xb00 + slot] = [1, 7, 23, 5, 6, 8, 9, 21][slot];
                    ram[0xb30 + slot] = 128;
                }
                checker.check(&ram, Case::op(5), "reverse/paused execute all")?;
                ram[0xffd] = 1;
                ram[0xffe] = 63;
                checker.check(&ram, Case::op(6), "main includes boulder guard")?;
            }
        }
    }
    println!(
        "Overlord dispatcher/spawns: {} cumulative cases",
        checker.count
    );
    for selected in 0..6 {
        for state in [0, 1, 255] {
            for ai in 0..=255u8 {
                let mut ram = base(1);
                ram[0xd80..0xd90].fill(1);
                ram[0xdd0 + selected] = state;
                ram[0xd80 + selected] = ai;
                checker.check(&ram, Case::op(13), "knight active/AI return guards")?;
                checker.check(&ram, Case::op(14), "knight six-slot-only coercion clear")?;
            }
        }
    }
    // Actual asset-backed sine values, signed angular increments and live
    // source/output overlap for inside and outside six-knight alias slots.
    for slot in [0, 5, 7u8] {
        for phase in [0, 1, 0xff, 0x100, 0x1ff, 0x200, 0x7fff, 0x8000, 0xffff] {
            for radius in [0, 1, 64, 255] {
                for increment in 0..=255u8 {
                    for operation in [11, 12] {
                        let mut ram = base(0);
                        xy(&mut ram[..], usize::from(slot), 0x120, 0x170);
                        word(&mut ram[..], 0xb08, phase);
                        ram[0xb0a] = radius;
                        ram[0xb40 + usize::from(slot)] = increment;
                        ram[0xb30 + usize::from(slot)] = u8::from(increment & 1 != 0);
                        checker.check(
                            &ram,
                            Case {
                                slot,
                                ..Case::op(operation)
                            },
                            "Armos real sine/live orbit aliases",
                        )?;
                    }
                }
            }
        }
    }
    for state in 0..=255u8 {
        for timer in [0, 1, 2, 63, 64, 127, 128, 192, 254, 255] {
            for radius in [0, 32, 33, 63, 64, 255] {
                let mut ram = base(0);
                ram[0xb07] = 25;
                ram[0xb2f] = state;
                ram[0xb37] = timer;
                ram[0xb0a] = radius;
                ram[0xd90] = 1;
                checker.check(
                    &ram,
                    Case {
                        slot: 7,
                        ..Case::op(15)
                    },
                    "Armos coordinator states/timer boundaries",
                )?;
            }
        }
    }
    for profile in 0..4 {
        for timer in 0..=255u8 {
            let mut ram = base(profile);
            ram[0x1b] = 0;
            ram[0xffd] = 1;
            ram[0xffe] = timer;
            word(&mut ram[..], 0xe8, 0x400);
            word(&mut ram[..], 0xfbe, 0x100);
            checker.check(&ram, Case::op(19), "boulder real spawn")?;
        }
    }
    println!("Overlord + Armos: {} cumulative cases", checker.count);
    for operation in [20, 21] {
        for phase in [
            0, 1, 0x7f, 0xff, 0x100, 0x1ff, 0x200, 0x7fff, 0x8000, 0xffff,
        ] {
            for radius in [0, 1, 64, 127, 128, 255] {
                for speed in 0..=255u8 {
                    let mut ram = base(0);
                    word(&mut ram[..], 0xb08, phase);
                    ram[0xb0a] = radius;
                    ram[0xb0c] = speed;
                    ram[0xb0d] = speed ^ 0xa5;
                    ram[0x1a] = speed;
                    ram[0xd90] = speed % 13;
                    ram[0xda0] = 12 - speed % 13;
                    for sprite in 0..16 {
                        ram[0xd00 + sprite] = (0xf8 + sprite) as u8;
                        ram[0xd10 + sprite] = (0xfe + sprite) as u8;
                        ram[0xd20 + sprite] = 0xff;
                        ram[0xd30 + sprite] = 0xff;
                        ram[0xd80 + sprite] = if sprite % 2 == 0 { 2 } else { 1 };
                    }
                    checker.check(
                        &ram,
                        Case {
                            slot: 0,
                            ..Case::op(operation)
                        },
                        "boss live orbit/speed byte/high sentinel",
                    )?;
                }
            }
        }
    }
    for slot in 1..=13u8 {
        for phase in [0, 0xff, 0x1ff, 0xffff] {
            for frame in [0, 1, 3, 4, 7, 8, 255] {
                for finished in [false, true] {
                    let mut ram = base(0);
                    for at in 0xb08..0xb58 {
                        ram[at] = (at as u8).wrapping_mul(17);
                    }
                    word(&mut ram[..], 0xb08, phase);
                    ram[0x1a] = frame;
                    ram[0xd90] = 0;
                    ram[0xda0] = 12;
                    ram[0x40c] = 0;
                    word(&mut ram[..], 0x48e, 0);
                    word(&mut ram[..], 0x402, if finished { 0x8000 } else { 0 });
                    ram[0xdd0 + usize::from(slot)] = 9;
                    ram[0xd10 + usize::from(slot)] = 0x3c;
                    ram[0xd30 + usize::from(slot)] = 0x6d;
                    ram[0xd00 + usize::from(slot)] = 0x8e;
                    ram[0xd20 + usize::from(slot)] = 0xa5;
                    checker.check(
                        &ram,
                        Case {
                            slot,
                            ..Case::op(22)
                        },
                        "Arrghi prep k+7 cross-lane aliases",
                    )?;
                }
            }
        }
    }
    Ok(())
}
fn finish(mut child: Child, result: Result<(), Error>) -> Result<(), Error> {
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait()?;
    result?;
    if !status.success() {
        return Err(format!("Overlord C process exited {status}").into());
    }
    Ok(())
}
fn main() -> Result<(), Error> {
    let args: Vec<_> = std::env::args_os().collect();
    let executable = args.get(1).map_or(
        std::ffi::OsStr::new("build/c-oracle/overlord_state_oracle"),
        |s| s.as_os_str(),
    );
    let asset_path = args
        .get(2)
        .map_or(std::ffi::OsStr::new("zelda3_assets.dat"), |s| s.as_os_str());
    let bytes = std::fs::read(asset_path)?;
    let assets = RuntimeAssets::parse(&bytes)?;
    let mut child = Command::new(executable)
        .arg(asset_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut checker = Checker {
        assets: &assets,
        input: child.stdin.take().ok_or("missing C stdin")?,
        output: child.stdout.take().ok_or("missing C stdout")?,
        expected: Box::new([0; WRAM_SIZE]),
        actual: Box::new([0; WRAM_SIZE]),
        count: 0,
    };
    let result = run(&mut checker);
    let count = checker.count;
    drop(checker);
    finish(child, result)?;
    println!(
        "Overlord actual C/native services: {count} cases, {} complete return+WRAM bytes; no masks",
        count * (WRAM_SIZE + 4)
    );
    Ok(())
}
