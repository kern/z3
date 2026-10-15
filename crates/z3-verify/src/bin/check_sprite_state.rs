//! Whole-WRAM sprite state vectors, plus exhaustive byte approach outputs.
use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use z3_core::WRAM_SIZE;
use z3_game::{
    ram::Ram,
    sprite::{Sprite, SpriteHooks},
    sprite_main_bank0::SpriteMainBank0Runtime,
    sprite_main_bank1::SpriteMainBank1Runtime,
    sprite_main_bank2a::SpriteMainBank2aRuntime,
    sprite_main_bank2b::SpriteMainBank2bRuntime,
};

struct Hooks;
impl SpriteHooks for Hooks {}
impl SpriteMainBank0Runtime for Hooks {}
impl SpriteMainBank1Runtime for Hooks {}
impl SpriteMainBank2aRuntime for Hooks {}
impl SpriteMainBank2bRuntime for Hooks {}

fn requests() -> Vec<[u8; 8]> {
    let mut requests = Vec::new();
    for slot in [0, 15] {
        for value in 0..=u8::MAX {
            for operation in 0..=4 {
                requests.push([operation, slot, value, value ^ 0x80, 0, 0, 0, 0]);
            }
        }
        for kind in 0..=242 {
            requests.push([5, slot, kind, 0, 0, 0, 0, 0]);
        }
        for value in 0..=u8::MAX {
            for wall in 0..16 {
                requests.push([2, slot, value, 0, 0, 0, 0, wall]);
            }
        }
        requests.push([7, slot, 0, 0, 0, 0, 0, 0]);
    }
    for seed in 0..32 {
        requests.push([6, 0, seed, 0, 0, 0, 0, 0]);
    }
    // Every speed in every sign quadrant, including fractional projection and
    // wrap boundaries; location projection has no angle lookup table bounds.
    for speed in 0..=u8::MAX {
        for (x, y) in [
            (0x120u16, 0x110u16),
            (0xe0, 0x110),
            (0x120, 0xf0),
            (0xe0, 0xf0),
            (0x180, 0x101),
            (0x100, 0x100),
            (0x180, 0x180),
            (0xffff, 0),
            (0, 0xffff),
        ] {
            let [xl, xh] = x.to_le_bytes();
            let [yl, yh] = y.to_le_bytes();
            requests.push([8, 15, xl, xh, yl, yh, speed, 0]);
        }
    }
    for value in 0..=u8::MAX {
        requests.push([9, 0, value, 0, 0, 0, 0, 0]);
        requests.push([9, 0, 0, value, 0, 0, 0, 0]);
    }
    for x in [0u8, 1, 3, 4, 31, 32, 63, 64, 127, 128, 129, 192, 255] {
        for y in [0u8, 1, 3, 4, 31, 32, 63, 64, 127, 128, 129, 192, 255] {
            let ax = (x as i8).unsigned_abs();
            let ay = (y as i8).unsigned_abs();
            let sign = usize::from((y >> 7) + (x >> 7) * 2) * 8;
            if usize::from(ax.min(ay) >> 2) + sign < 32 {
                requests.push([9, 0, x, y, 0, 0, 0, 0]);
            }
        }
    }
    requests
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .ok_or("usage: check_sprite_state SPRITE_STATE_ORACLE")?;
    let mut child = Command::new(oracle)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let result = (|| -> Result<usize, Box<dyn std::error::Error>> {
        let mut input = child.stdin.take().ok_or("oracle stdin")?;
        let mut output = BufReader::new(child.stdout.take().ok_or("oracle stdout")?);
        let mut actual = Box::new([0; WRAM_SIZE]);
        let mut expected = Box::new([0; WRAM_SIZE]);
        let mut hooks = Hooks;
        let requests = requests();
        for (case, request) in requests.iter().enumerate() {
            for (index, byte) in actual.iter_mut().enumerate() {
                *byte = (index as u8)
                    .wrapping_mul(37)
                    .wrapping_add((index >> 8) as u8)
                    .wrapping_add(case as u8);
            }
            let slot = usize::from(request[1]);
            {
                let mut ram = Ram::new(&mut actual);
                ram.set_sprite_velocity_x(slot, request[2]);
                ram.set_sprite_velocity_y(slot, request[2].wrapping_neg());
                ram.set_sprite_wall_collision(slot, request[7]);
                ram.set_sprite_kind(slot, request[2].min(242));
                ram.set_sprite_x(slot, 0x100);
                ram.set_sprite_y(slot, 0x100);
                ram.write_u8(0x1b, (case & 1) as u8);
                if request[0] == 6 {
                    for index in 0..16 {
                        ram.set_sprite_state(index, [0, 4, 9, 10, 255][(index + case) % 5]);
                        ram.set_sprite_pause(index, ((index + case) % 3) as u8);
                    }
                }
            }
            input.write_all(request)?;
            input.write_all(actual.as_slice())?;
            input.flush()?;
            let mut sprite = Sprite::new(&mut actual, &mut hooks);
            match request[0] {
                0 => sprite.sprite_invert_xy_speeds(slot),
                1 => sprite.sprite_halve_speed_xy(slot),
                2 => sprite.sprite_bounce_off_wall(slot),
                3 => sprite.sprite_approach_target_speed(slot, request[2], request[3]),
                4 => sprite.sprite_prep_reset_properties(slot),
                5 => sprite.sprite_prep_load_properties(slot),
                6 => sprite.dungeon_cache_trans_sprites(),
                7 => {
                    let mut values = vec![0; 256 * 256 * 2];
                    output.read_exact(&mut values)?;
                    let mut index = 0;
                    for current in 0..=u8::MAX {
                        for target in 0..=u8::MAX {
                            sprite.ram.set_sprite_velocity_x(slot, current);
                            sprite.ram.set_sprite_velocity_y(slot, target);
                            sprite.sprite_approach_target_speed(slot, target, current);
                            let pair = [
                                sprite.ram.sprite(slot).velocity_x(),
                                sprite.ram.sprite(slot).velocity_y(),
                            ];
                            if pair != values[index..index + 2] {
                                return Err(format!(
                                    "approach current={current} target={target} slot={slot}"
                                )
                                .into());
                            }
                            index += 2;
                        }
                    }
                }
                8 => {
                    let result = sprite.sprite_project_speed_towards_location(
                        slot,
                        u16::from_le_bytes([request[2], request[3]]),
                        u16::from_le_bytes([request[4], request[5]]),
                        request[6],
                    );
                    let mut reference = [0; 4];
                    output.read_exact(&mut reference)?;
                    if reference != [result.x, result.y, result.xdiff, result.ydiff] {
                        return Err(format!("projection case {case}").into());
                    }
                }
                9 => {
                    let result =
                        Sprite::<Hooks>::sprite_convert_velocity_to_angle(request[2], request[3]);
                    let mut reference = [0];
                    output.read_exact(&mut reference)?;
                    if reference[0] != result {
                        return Err(format!("angle case {case}").into());
                    }
                }
                _ => unreachable!(),
            }
            output.read_exact(expected.as_mut_slice())?;
            if let Some(offset) = expected.iter().zip(actual.iter()).position(|(c, r)| c != r) {
                return Err(format!("sprite case {case} request={request:?}, WRAM {offset:#07x}: C={:02x} Rust={:02x}", expected[offset], actual[offset]).into());
            }
        }
        Ok(requests.len())
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait()?;
    let count = result?;
    if !status.success() {
        return Err(format!("sprite oracle exited {status}").into());
    }
    println!(
        "sprite arithmetic/state matches compiled C: {count} whole-WRAM cases, exhaustive 65,536 approach pairs at both slot extremes, defined angle/projection vectors"
    );
    Ok(())
}
