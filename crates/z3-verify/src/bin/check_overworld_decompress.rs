use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::WRAM_SIZE;
use z3_game::{overworld, ram::Ram};

struct MapAssets {
    map16: Vec<u16>,
    secrets: Vec<u8>,
    secret_offsets: Vec<u16>,
    map_is_small: Vec<u8>,
    aux_themes: Vec<u8>,
    entrance_areas: Vec<u16>,
    entrance_positions: Vec<u16>,
    entrance_ids: Vec<u8>,
}

impl overworld::OverworldAssets for MapAssets {
    fn entrance_areas(&self) -> &[u16] {
        &self.entrance_areas
    }
    fn entrance_positions(&self) -> &[u16] {
        &self.entrance_positions
    }
    fn sign_text(&self) -> &[u16] {
        &[]
    }
    fn background_palettes(&self) -> &[u8] {
        &[]
    }
    fn sprite_graphics(&self) -> &[u8] {
        &[]
    }
    fn sprite_palettes(&self) -> &[u8] {
        &[]
    }
    fn sprite_offsets(&self) -> &[u16] {
        &[]
    }
    fn sprites(&self) -> &[u8] {
        &[]
    }
    fn map16_to_map8(&self) -> &[u16] {
        &self.map16
    }
    fn map8_tile_attributes(&self) -> &[u8] {
        &[]
    }
    fn enemy_damage_data(&self) -> &[u8] {
        &[]
    }
    fn overworld_music_sets(&self) -> &[u8] {
        &[]
    }
    fn overworld_music_sets2(&self) -> &[u8] {
        &[]
    }
    fn fall_hole_areas(&self) -> &[u16] {
        &[]
    }
    fn fall_hole_positions(&self) -> &[u16] {
        &[]
    }
    fn fall_hole_entrances(&self) -> &[u8] {
        &[]
    }
    fn whirlpool_areas(&self) -> &[u16] {
        &[]
    }
    fn overworld_secrets(&self) -> &[u8] {
        &self.secrets
    }
    fn overworld_secret_offsets(&self) -> &[u16] {
        &self.secret_offsets
    }
    fn overworld_map_is_small(&self) -> &[u8] {
        &self.map_is_small
    }
    fn overworld_aux_tile_themes(&self) -> &[u8] {
        &self.aux_themes
    }
    fn entrance_ids(&self) -> &[u8] {
        &self.entrance_ids
    }
}

fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn emit_short(source: &mut Vec<u8>, kind: u8, length: usize) {
    source.push(kind << 5 | (length - 1) as u8);
}

fn emit_long(source: &mut Vec<u8>, kind: u8, length: usize) {
    let encoded = length - 1;
    source.push(0xe0 | kind << 2 | (encoded >> 8) as u8);
    source.push(encoded as u8);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_overworld_decompress ORACLE")?;
    let mut child = Command::new(oracle).stdout(Stdio::piped()).spawn()?;
    let mut reference = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing overworld oracle stdout")?
        .read_to_end(&mut reference)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("overworld decompressor oracle exited with {status}").into());
    }

    let mut actual = Vec::with_capacity(256 * 8 + 65_536 * 2 + 2 * 64 * 0x18 + 2 * 256 * 3 + 14);
    for scenario in 0..=u8::MAX {
        let mut source = Vec::with_capacity(2048);
        emit_short(&mut source, 0, 32);
        for index in 0..32u8 {
            source.push(scenario.wrapping_add(index.wrapping_mul(17)));
        }

        let lengths = [
            1 + usize::from(scenario & 31),
            1 + usize::from(scenario.wrapping_mul(3) & 31),
            1 + usize::from(scenario.wrapping_mul(5) & 31),
            1 + usize::from(scenario.wrapping_mul(7) & 31),
        ];
        emit_short(&mut source, 1, lengths[0]);
        source.push(scenario ^ 0xa5);
        emit_short(&mut source, 2, lengths[1]);
        source.extend_from_slice(&[scenario, !scenario]);
        emit_short(&mut source, 3, lengths[2]);
        source.push(scenario.wrapping_mul(11));
        emit_short(&mut source, 4, lengths[3]);
        source.extend_from_slice(&[0, scenario & 15]);

        let kind = scenario % 5;
        let length = 40 + usize::from(scenario) % 240 * 4;
        emit_long(&mut source, kind, length);
        match kind {
            0 => {
                for index in 0..length {
                    source.push(
                        scenario
                            .wrapping_mul(29)
                            .wrapping_add((index as u8).wrapping_mul(13)),
                    );
                }
            }
            1 | 3 => source.push(scenario.wrapping_mul(19).wrapping_add(3)),
            2 => source.extend_from_slice(&[scenario.wrapping_add(7), scenario.wrapping_sub(9)]),
            4 => source.extend_from_slice(&[0, scenario & 31]),
            _ => unreachable!(),
        }
        source.push(0xff);

        let mut output = [0u8; 4096];
        let written = overworld::decompress_bank02(&mut output, &source);
        let c_length = i32::try_from(written)?.to_ne_bytes();
        actual.extend_from_slice(&(hash(&c_length) ^ hash(&output[..written])).to_le_bytes());
    }
    for address in 0..=u16::MAX {
        actual.extend_from_slice(&overworld::find_map16_vram_address(address).to_le_bytes());
    }
    let mut wram = [0u8; WRAM_SIZE];
    for big in [false, true] {
        for area in 0..64 {
            wram[0x600..0x618].fill(0);
            let mut ram = Ram::new(&mut wram);
            overworld::set_camera_boundaries(&mut ram, big, area);
            actual.extend_from_slice(ram.slice(0x600, 0x18));
        }
    }
    for indoors in [0u8, 1] {
        for health in 0..=u8::MAX {
            wram[..0x10000].fill(0);
            let mut ram = Ram::new(&mut wram);
            ram.write_u8(0x1b, indoors);
            ram.write_u8(0xf36d, health);
            overworld::take_damage_from_pit(&mut ram);
            actual.extend_from_slice(&[ram.read_u8(0x4b), ram.read_u8(0x11), ram.read_u8(0xf36d)]);
        }
    }
    for feature in [0u8, 4] {
        wram[..0x10000].fill(0xa5);
        let mut ram = Ram::new(&mut wram);
        ram.write_u8(0x64c, feature);
        overworld::force_nonbunny_status(&mut ram);
        for offset in [0x5d, 0x3f5, 0x3f6, 0x3f7, 0x56, 0x2e0, 0x372] {
            actual.push(ram.read_u8(offset));
        }
    }
    for completed in [false, true] {
        for progress in [0u16, 2, 4, 0xffff] {
            for position in [0x826u16, 0x5a0, 0x81a, 0x1234] {
                wram[..0x10000].fill(0);
                let mut ram = Ram::new(&mut wram);
                ram.write_u16(0x8a, 7);
                ram.write_u16(0x4c8, progress);
                ram.write_u8(0xf287, u8::from(completed) * 0x20);
                let update = overworld::handle_peg_puzzles(&mut ram, position);
                assert!(!update);
                actual.extend_from_slice(&ram.read_u16(0x4c8).to_le_bytes());
                actual.extend_from_slice(&ram.read_u16(0x12e).to_le_bytes());
                actual.extend_from_slice(&[ram.read_u8(0xf287), ram.read_u8(0x11)]);
            }
        }
    }
    for progress in 0u16..=30 {
        if progress == 21 {
            continue;
        }
        wram[..0x10000].fill(0);
        let mut ram = Ram::new(&mut wram);
        ram.write_u16(0x8a, 98);
        ram.write_u16(0x4c8, progress);
        assert!(!overworld::handle_peg_puzzles(&mut ram, 0));
        actual.extend_from_slice(&ram.read_u16(0x4c8).to_le_bytes());
        actual.extend_from_slice(&ram.read_u16(0x692).to_le_bytes());
        actual.extend_from_slice(&ram.read_u16(0x698).to_le_bytes());
        actual.extend_from_slice(&[ram.read_u8(0xf2e2), ram.read_u8(0x12f)]);
    }
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    for index in 0..256 {
        ram.write_u16(0xc300 + index * 2, index as u16 ^ 0x4210);
    }
    ram.write_u16(0xc500, 0x1234);
    ram.write_u16(0xc540, 0xabcd);
    overworld::palette_anim_get_master_sword2(&mut ram);
    actual.extend_from_slice(&hash(ram.slice(0xc300, 0x400)).to_le_bytes());
    actual.extend_from_slice(&hash(ram.slice(0x1dd80, 0x200)).to_le_bytes());
    actual.extend_from_slice(&ram.read_u16(0xc540).to_le_bytes());
    actual.extend_from_slice(&ram.read_u16(0xc007).to_le_bytes());
    actual.extend_from_slice(&ram.read_u16(0xc009).to_le_bytes());
    actual.push(ram.read_u8(0xb0));
    ram.write_u16(0xc009, 0);
    ram.write_u16(0xc007, 31);
    ram.write_u8(0x1d, 0xa5);
    overworld::palette_anim_get_master_sword3(&mut ram);
    actual.extend_from_slice(&hash(ram.slice(0xc300, 0x200)).to_le_bytes());
    actual.push(ram.read_u8(0x1d));
    for screen in [0x42u16, 0x43, 0x45, 0x47] {
        for frame in [3u8, 5, 7, 36, 44, 88, 90] {
            for event in [0u8, 0x20] {
                for trigger in [0u8, 1] {
                    wram[..0x20000].fill(0);
                    let mut ram = Ram::new(&mut wram);
                    ram.write_u16(0x8a, screen);
                    ram.write_u8(0x1a, frame);
                    ram.write_u8(0x4c6, trigger);
                    ram.write_u8(0xf2c3, event);
                    for index in 0..256 {
                        ram.write_u16(0xc300 + index * 2, (index * 73 + 11) as u16);
                    }
                    overworld::dw_death_mountain_palette_animation(&mut ram);
                    actual.extend_from_slice(&hash(ram.slice(0xc500, 0x200)).to_le_bytes());
                    actual.extend_from_slice(&[ram.read_u8(0x15), ram.read_u8(0x12e)]);
                }
            }
        }
    }
    let map_assets = MapAssets {
        map16: (0..32u16)
            .flat_map(|tile| [tile * 4, tile * 4 + 1, tile * 4 + 2, tile * 4 + 3])
            .collect(),
        secrets: Vec::new(),
        secret_offsets: Vec::new(),
        map_is_small: Vec::new(),
        aux_themes: Vec::new(),
        entrance_areas: Vec::new(),
        entrance_positions: Vec::new(),
        entrance_ids: Vec::new(),
    };
    let mut map_source = vec![0u8; 0x2000];
    for tile in 0..32u16 {
        map_source[usize::from(tile) * 2..usize::from(tile) * 2 + 2]
            .copy_from_slice(&tile.to_le_bytes());
    }
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    ram.write_u16(0x84, 0x410);
    ram.write_u16(0x86, 31);
    ram.write_u16(0x88, 0x12);
    overworld::copy_map16_to_buffer(&mut ram, &map_assets, &map_source, 0x1000, 0, 0x14000);
    actual.extend_from_slice(&hash(ram.slice(0x500, 0x40)).to_le_bytes());
    actual.extend_from_slice(&hash(ram.slice(0x12000, 0x100)).to_le_bytes());
    actual.extend_from_slice(&hash(ram.slice(0x14000, 4)).to_le_bytes());
    for screen in 0u16..128 {
        wram[..0x10000].fill(0);
        let mut ram = Ram::new(&mut wram);
        ram.write_u16(0x8a, screen);
        ram.write_u8(0xf280 + usize::from(screen), 2);
        overworld::handle_overlays_and_bomb_doors(&mut ram, |_| {
            unreachable!("event-overlay bit is clear")
        });
        actual.extend_from_slice(&hash(ram.slice(0x2000, 0x2000)).to_le_bytes());
    }
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    for index in 0..32 {
        ram.write_u16(0x2000 + index * 128, index as u16);
    }
    ram.write_u8(0x416, 1);
    ram.write_u16(0x84, 0x3d0);
    let stripe_end = overworld::buffer_and_build_map16_stripes_x(&mut ram, &map_assets, 0x1100);
    actual.extend_from_slice(&(((stripe_end - 0x1100) / 2) as u16).to_le_bytes());
    actual.extend_from_slice(&hash(ram.slice(0x1100, 264)).to_le_bytes());
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    for index in 0..32 {
        ram.write_u16(0x2000 + index * 2, index as u16);
    }
    ram.write_u8(0x416, 4);
    ram.write_u16(0x84, 0xf410);
    let stripe_end = overworld::buffer_and_build_map16_stripes_y(&mut ram, &map_assets, 0x1100);
    actual.extend_from_slice(&(((stripe_end - 0x1100) / 2) as u16).to_le_bytes());
    actual.extend_from_slice(&hash(ram.slice(0x1100, 260)).to_le_bytes());
    for axis in 0..4usize {
        for velocity in [1i16, -1] {
            for accumulator in [0u16, 15, 0xffff] {
                for at_boundary in [false, true] {
                    wram[..0x10000].fill(0);
                    let mut ram = Ram::new(&mut wram);
                    let xa = if axis < 2 { 6 } else { 0 };
                    let ya = axis * 2;
                    let r8 = if axis < 2 { 0 } else { 4 };
                    let background = if xa != 0 { 0xe8 } else { 0xe2 };
                    ram.write_u16(background, 0x345);
                    ram.write_u16(0x600 + axis * 2, if at_boundary { 0x345 } else { 0x567 });
                    ram.write_u16(0x624 + axis * 2, accumulator);
                    ram.write_u16(0x61a + (r8 >> 1) * 2, 0x234);
                    let result = overworld::camera_boundary_check(&mut ram, xa, ya, velocity, r8);
                    actual.extend_from_slice(&(result as u16).to_le_bytes());
                    actual.extend_from_slice(&hash(ram.slice(0xe0, 10)).to_le_bytes());
                    actual.extend_from_slice(&hash(ram.slice(0x600, 0x2c)).to_le_bytes());
                    actual.extend_from_slice(&hash(ram.slice(0x416, 4)).to_le_bytes());
                }
            }
        }
    }
    for transition in 0..4usize {
        for near_threshold in [false, true] {
            wram[..0x10000].fill(0);
            let mut ram = Ram::new(&mut wram);
            ram.write_u16(0x418, transition as u16);
            ram.write_u8(
                0x126,
                if near_threshold {
                    [27u8, 27, 30, 30][transition] - 1
                } else {
                    0
                },
            );
            ram.write_u16(0xe8, 0x1000);
            ram.write_u16(0xe2, 0x1000);
            ram.write_u16(0x20, 0x234);
            ram.write_u16(0x22, 0x456);
            ram.write_u16(0x610 + transition * 2, 0xffff);
            let result = overworld::scroll_transition(&mut ram, |_| {
                unreachable!("the test target cannot complete")
            });
            actual.extend_from_slice(&result.to_le_bytes());
            actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
        }
    }
    for overlay in [0u8, 0x95, 0x97, 0x9c, 0xb5] {
        for y_velocity in [0u8, 1, 2, 0xff] {
            for x_velocity in [0u8, 1, 2, 0xff] {
                wram[..0x10000].fill(0);
                let mut ram = Ram::new(&mut wram);
                ram.write_u16(0x8c, u16::from(overlay));
                ram.write_u16(0x8a, 5);
                ram.write_u16(0x20, 0x220);
                ram.write_u16(0x22, 0x330);
                ram.write_u8(0x30, y_velocity);
                ram.write_u8(0x31, x_velocity);
                ram.write_u16(0x24, 4);
                ram.write_u8(0x78, 1);
                ram.write_u16(0x618, 0x300);
                ram.write_u16(0x61a, 0x100);
                ram.write_u16(0x61c, 0x400);
                ram.write_u16(0x61e, 0x100);
                ram.write_u16(0xe8, 0x345);
                ram.write_u16(0xe2, 0x456);
                ram.write_u16(0x600, 0x111);
                ram.write_u16(0x602, 0x222);
                ram.write_u16(0x604, 0x333);
                ram.write_u16(0x606, 0x444);
                ram.write_u16(0x622, 0x1234);
                ram.write_u16(0xe6, 0x567);
                ram.write_u16(0x620, 0x4321);
                ram.write_u16(0xe0, 0x765);
                overworld::operate_camera_scroll(&mut ram);
                actual.extend_from_slice(&hash(ram.slice(0, 0x700)).to_le_bytes());
            }
        }
    }
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    ram.write_u16(0xc140, 0x23);
    ram.write_u16(0xc142, 0x1514);
    ram.write_u16(0xc144, 0x222);
    ram.write_u16(0xc146, 0x444);
    ram.write_u16(0xc148, 0x666);
    ram.write_u16(0xc14a, 0x888);
    ram.write_u16(0xc14c, 0x2a);
    ram.write_u16(0xc14e, 0x790);
    ram.write_u16(0xc150, 0x321);
    ram.write_u16(0xc152, 0x543);
    for index in 0..4 {
        ram.write_u16(0xc154 + index * 2, 0x1000 + index as u16 * 2);
        ram.write_u16(0xc15c + index * 2, 0x2000 + index as u16 * 2);
        ram.write_u16(0xc16a + index * 2, 0x3000 + index as u16 * 2);
    }
    ram.write_u8(0xc164, 0x51);
    ram.write_u8(0xc165, 0x52);
    ram.write_u8(0xc166, 0x53);
    ram.write_u8(0xc167, 0x54);
    ram.write_u16(0xa0, 0x130);
    overworld::load_cached_entrance_properties(&mut ram);
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    wram[..0x1000].fill(0xa5);
    let mut ram = Ram::new(&mut wram);
    overworld::entrance_play_jingle(&mut ram);
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    wram[..0x1000].fill(0xa5);
    let mut ram = Ram::new(&mut wram);
    overworld::entrance_advance_and_boom(&mut ram);
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    wram[..0x1000].fill(0xa5);
    let mut ram = Ram::new(&mut wram);
    ram.write_u8(0x13, 14);
    overworld::overworld_func22(&mut ram);
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    wram[..0x1000].fill(0xa5);
    let mut ram = Ram::new(&mut wram);
    ram.write_u8(0xb0, 2);
    ram.write_u16(0x8a, 0x40);
    ram.write_u16(0xc007, 1);
    ram.write_u8(0xc011, 0x30);
    overworld::overworld_func1c(
        &mut ram,
        |_| unreachable!(),
        |_| unreachable!(),
        |_| unreachable!(),
    );
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    for screen in [0u16, 3, 0x43, 0x5b, 0x70, 0x80, 0x81, 0x1b] {
        for submodule in [0u8, 4] {
            for direction in [0u8, 8] {
                for room in [0u16, 0x181] {
                    wram[..0x20000].fill(0);
                    let mut ram = Ram::new(&mut wram);
                    ram.write_u16(0x8a, screen);
                    ram.write_u8(0x11, submodule);
                    ram.write_u8(0x410, direction);
                    ram.write_u16(0xa0, room);
                    ram.write_u16(0xe8, 0x712);
                    ram.write_u16(0xe2, 0x756);
                    ram.write_u8(0x15, 0xfe);
                    overworld::set_fixed_color_and_scroll(&mut ram);
                    actual.extend_from_slice(&hash(ram.slice(0, 0x20000)).to_le_bytes());
                }
            }
        }
    }
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    ram.write_u8(0xb0, 2);
    ram.write_u16(0x8a, 0x80);
    ram.write_u8(0x10, 11);
    ram.write_u16(0xc007, 1);
    ram.write_u8(0xc011, 0x30);
    ram.write_u8(0x130, 0x27);
    overworld::fade_back_in_from_mosaic(
        &mut ram,
        &map_assets,
        |_, _, _| unreachable!(),
        |_| unreachable!(),
        |_| unreachable!(),
        |_| unreachable!(),
        |_| unreachable!(),
    );
    actual.extend_from_slice(&hash(ram.slice(0, 0x20000)).to_le_bytes());
    for room in [0x21u16, 0x20] {
        for facing in [0u8, 2] {
            for rock in [0u16, 0x1234, 0x9234] {
                for screen in [3u16, 0x43] {
                    wram[..0x1000].fill(0xa5);
                    let mut ram = Ram::new(&mut wram);
                    ram.write_u8(0x10, 9);
                    ram.write_u16(0xa0, room);
                    ram.write_u8(0x2f, facing);
                    ram.write_u16(0x698, rock);
                    ram.write_u16(0x696, u16::from(rock != 0));
                    ram.write_u16(0x8a, screen);
                    overworld::open_spotlight_next(
                        &mut ram,
                        |_| unreachable!(),
                        |_| unreachable!(),
                    );
                    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
                }
            }
        }
    }
    wram[..0x1000].fill(0xa5);
    let mut ram = Ram::new(&mut wram);
    overworld::finish_transition_gfx(&mut ram);
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    wram[..0x1000].fill(0xa5);
    let mut ram = Ram::new(&mut wram);
    ram.write_u8(0x690, 3);
    overworld::open_big_door_from_exit(&mut ram, |_| unreachable!());
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    wram[..0x1000].fill(0xa5);
    let mut ram = Ram::new(&mut wram);
    ram.write_u8(0x690, 3);
    overworld::open_big_door(&mut ram, |_| unreachable!());
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    wram[..0x1000].fill(0xa5);
    let mut ram = Ram::new(&mut wram);
    ram.write_u16(0x692, 5);
    overworld::update_map_32x32_conditional(&mut ram, |_| unreachable!());
    actual.extend_from_slice(&hash(ram.slice(0, 0x1000)).to_le_bytes());
    let door_assets = MapAssets {
        map16: (0..0x4000u16).collect(),
        secrets: Vec::new(),
        secret_offsets: Vec::new(),
        map_is_small: Vec::new(),
        aux_themes: Vec::new(),
        entrance_areas: Vec::new(),
        entrance_positions: Vec::new(),
        entrance_ids: Vec::new(),
    };
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    ram.write_u16(0x4ac, 4);
    ram.write_u16(0x698, 0x420);
    ram.write_u16(0x692, 8);
    overworld::update_map_32x32(&mut ram, &door_assets);
    actual.extend_from_slice(&hash(ram.slice(0, 0x20000)).to_le_bytes());
    for room in [0x180u16, 0x182, 0x183, 0x185] {
        wram[..0x20000].fill(0);
        let mut ram = Ram::new(&mut wram);
        ram.write_u16(0x8a, 0x80);
        ram.write_u16(0xa0, room);
        ram.write_u8(0xf300, 0x40);
        ram.write_u16(0x84, 0x1234);
        ram.write_u16(0x88, 0x5678);
        ram.write_u16(0x86, 0x9abc);
        ram.write_u16(0x418, 3);
        ram.write_u16(0x410, 4);
        ram.write_u16(0x416, 8);
        overworld::load_overlays_inner(&mut ram, |_| unreachable!());
        actual.extend_from_slice(&hash(ram.slice(0, 0x20000)).to_le_bytes());
    }
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    for index in 0..256 {
        ram.write_u16(0xc300 + index * 2, (index * 17 + 3) as u16);
    }
    overworld::ganon_tower_entrance_flash(&mut ram, |_| unreachable!());
    actual.extend_from_slice(&hash(ram.slice(0, 0x20000)).to_le_bytes());
    for scenario in 0..3 {
        let secret_assets = MapAssets {
            map16: Vec::new(),
            secrets: vec![0x34, 0x12, if scenario == 2 { 5 } else { 0x82 }, 0xff, 0xff],
            secret_offsets: vec![0; 128],
            map_is_small: Vec::new(),
            aux_themes: Vec::new(),
            entrance_areas: Vec::new(),
            entrance_positions: Vec::new(),
            entrance_ids: Vec::new(),
        };
        wram[..0x20000].fill(0);
        let mut ram = Ram::new(&mut wram);
        ram.write_u16(0x8a, 2);
        if scenario == 1 {
            ram.write_u8(0xf282, 2);
            ram.write_u16(0x64c, 0x1000);
            ram.write_u8(0x301, 0x40);
        }
        actual.extend_from_slice(
            &overworld::reveal_secret(&mut ram, &secret_assets, 0x1234).to_le_bytes(),
        );
        actual.extend_from_slice(&hash(ram.slice(0, 0x20000)).to_le_bytes());
    }
    let gfx_assets = MapAssets {
        map16: Vec::new(),
        secrets: Vec::new(),
        secret_offsets: Vec::new(),
        map_is_small: (0..128).map(|index| u8::from(index % 3 == 0)).collect(),
        aux_themes: (0..128)
            .map(|index| (index as u8).wrapping_mul(7).wrapping_add(3))
            .collect(),
        entrance_areas: Vec::new(),
        entrance_positions: Vec::new(),
        entrance_ids: Vec::new(),
    };
    for (index, screen) in [5u8, 0x45].into_iter().enumerate() {
        wram[..0x20000].fill(0xa5);
        let mut ram = Ram::new(&mut wram);
        ram.write_u16(0x8a, u16::from(screen));
        ram.write_u8(0xfcc0 + usize::from(screen), 0x31 + index as u8);
        ram.write_u16(0x712, 0x20a5);
        overworld::load_gfx_and_screen_size(&mut ram, &gfx_assets);
        actual.extend_from_slice(&hash(ram.slice(0, 0x20000)).to_le_bytes());
    }
    let mut entrance_ids = vec![0; 129];
    entrance_ids[128] = 9;
    let entrance_assets = MapAssets {
        map16: vec![0, 0, 0xfe, 0x14a],
        secrets: Vec::new(),
        secret_offsets: Vec::new(),
        map_is_small: Vec::new(),
        aux_themes: Vec::new(),
        entrance_areas: vec![0; 129],
        entrance_positions: vec![0; 129],
        entrance_ids,
    };
    wram[..0x20000].fill(0);
    let mut ram = Ram::new(&mut wram);
    ram.write_u8(0x2f, 2);
    overworld::use_entrance(&mut ram, &entrance_assets, |_| unreachable!());
    actual.extend_from_slice(&hash(ram.slice(0, 0x20000)).to_le_bytes());

    if reference != actual {
        let offset = reference
            .iter()
            .zip(&actual)
            .position(|(c, rust)| c != rust)
            .unwrap_or(reference.len().min(actual.len()));
        return Err(format!(
            "overworld helper mismatch at result byte {offset:#x}: C={:02x?}, Rust={:02x?}",
            reference.get(offset),
            actual.get(offset)
        )
        .into());
    }
    println!(
        "overworld decompressor, map16 VRAM addresses, camera bounds/steps/full scrolling, pit damage, bunny reset, peg puzzles, palette animation, graphics/screen sizing, entrance selection, map16 conversion/scroll stripes, and bomb-door overlays match compiled C"
    );
    Ok(())
}
