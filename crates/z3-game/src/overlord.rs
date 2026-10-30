//! Overlord (room-controller) logic, faithfully ported from `src/overlord.c`.
//!
//! The original WRAM addresses remain explicit during the parity phase. Calls
//! into the sprite allocator, tile system, random source, garnish allocator,
//! and sound system are represented by [`OverlordHooks`].

use crate::{position::SplitCoordinate, ram::Ram};

const OVERLORD_TYPE: usize = 0x0b00;
const OVERLORD_X_LO: usize = 0x0b08;
const OVERLORD_X_HI: usize = 0x0b10;
const OVERLORD_Y_LO: usize = 0x0b18;
const OVERLORD_Y_HI: usize = 0x0b20;
const OVERLORD_GEN1: usize = 0x0b28;
const OVERLORD_GEN2: usize = 0x0b30;
const OVERLORD_GEN3: usize = 0x0b38;
const OVERLORD_FLOOR: usize = 0x0b40;
const OVERLORD_OFFSET_SPRITE_POS: usize = 0x0b48;
const CONTROLLER_X: SplitCoordinate = SplitCoordinate::new(OVERLORD_X_LO, OVERLORD_X_HI);
const CONTROLLER_Y: SplitCoordinate = SplitCoordinate::new(OVERLORD_Y_LO, OVERLORD_Y_HI);

const SPRITE_IGNORE_PROJECTILE: usize = 0x0ba0;
const SPRITE_FLAGS5: usize = 0x0be0;
const SPRITE_BUMP_DAMAGE: usize = 0x0cd2;
const SPRITE_Y_LO: usize = 0x0d00;
const SPRITE_X_LO: usize = 0x0d10;
const SPRITE_Y_HI: usize = 0x0d20;
const SPRITE_X_HI: usize = 0x0d30;
const SPRITE_Y_VEL: usize = 0x0d40;
const SPRITE_X_VEL: usize = 0x0d50;
const SPRITE_AI_STATE: usize = 0x0d80;
const SPRITE_A: usize = 0x0d90;
const SPRITE_B: usize = 0x0da0;
const SPRITE_C: usize = 0x0db0;
const SPRITE_STATE: usize = 0x0dd0;
const SPRITE_D: usize = 0x0de0;
const SPRITE_DELAY_MAIN: usize = 0x0df0;
const SPRITE_DELAY_AUX1: usize = 0x0e00;
const SPRITE_DELAY_AUX2: usize = 0x0e10;
const SPRITE_TYPE: usize = 0x0e20;
const SPRITE_FLAGS2: usize = 0x0e40;
const SPRITE_HEALTH: usize = 0x0e50;
const SPRITE_FLAGS3: usize = 0x0e60;
const SPRITE_SUBTYPE2: usize = 0x0e80;
const SPRITE_E: usize = 0x0e90;
const SPRITE_HEAD_DIR: usize = 0x0eb0;
const SPRITE_FLOOR: usize = 0x0f20;
const SPRITE_OAM_FLAGS: usize = 0x0f50;
const SPRITE_FLAGS4: usize = 0x0f60;
const SPRITE_Z: usize = 0x0f70;

const OVERWORLD_SPRITE_WAS_LOADED: usize = 0x1ef80;

const K_ARMOS_COORDINATOR_BACK_WALL_X: [u8; 6] = [49, 77, 105, 131, 159, 187];
const K_RED_STALFOS_TRAP_X: [i8; 4] = [0, 0, -48, 48];
const K_RED_STALFOS_TRAP_Y: [i8; 4] = [-40, 56, 8, 8];
const K_RED_STALFOS_TRAP_DELAY: [u8; 4] = [0x30, 0x50, 0x70, 0x90];
const K_OVERLORD_ZORO_FACTORY_X: [i8; 8] = [-4, -2, 0, 2, 4, 6, 8, 12];
const K_OVERLORD_WIZZROBE_X: [i8; 4] = [48, -48, 0, 0];
const K_OVERLORD_WIZZROBE_Y: [i8; 4] = [16, 16, 64, -32];
const K_OVERLORD_WIZZROBE_DELAY: [u8; 4] = [0, 16, 32, 48];
const K_SPAWN_FLYING_TILE_X: [u8; 22] = [
    0x70, 0x80, 0x60, 0x90, 0x90, 0x60, 0x70, 0x80, 0x80, 0x70, 0x50, 0xa0, 0xa0, 0x50, 0x50, 0xa0,
    0xa0, 0x50, 0x70, 0x80, 0x80, 0x70,
];
const K_SPAWN_FLYING_TILE_Y: [u8; 22] = [
    0x80, 0x80, 0x70, 0x90, 0x70, 0x90, 0x60, 0xa0, 0x60, 0xa0, 0x60, 0xb0, 0x60, 0xb0, 0x80, 0x90,
    0x80, 0x90, 0x70, 0x90, 0x70, 0x90,
];
const K_OVERLORD_PIROGUSU_A: [u8; 4] = [2, 3, 0, 1];
const K_CRUMBLE_TILE_PATH_DATA: [u8; 109] = [
    2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 3, 1, 3, 0, 3, 1,
    3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1,
    3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0xff,
];
const K_CRUMBLE_TILE_PATH_OFFS: [u8; 7] = [0, 25, 66, 77, 87, 98, 108];
const K_CRUMBLE_TILE_PATH_X: [i8; 4] = [16, -16, 0, 0];
const K_CRUMBLE_TILE_PATH_Y: [i8; 4] = [0, 0, 16, -16];
const K_OVERLORD_ZOL_X: [i8; 4] = [0, 0, -48, 48];
const K_OVERLORD_ZOL_Y: [i8; 4] = [-40, 56, 8, 8];
const K_STALFOS_TRAP_TRIGGER: [u8; 8] = [255, 224, 192, 160, 128, 96, 64, 32];
const K_SNAKE_TRAP_OVERLORD_TAB1: [u8; 8] = [0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90];
const K_ALL_DIRECTION_METAL_BALL_FACTORY_IDX: [u8; 16] =
    [2, 2, 2, 2, 1, 1, 1, 1, 3, 3, 3, 3, 0, 0, 0, 0];
const K_ALL_DIRECTION_METAL_BALL_FACTORY_X: [u8; 16] = [
    64, 96, 144, 176, 240, 240, 240, 240, 176, 144, 96, 64, 0, 0, 0, 0,
];
const K_ALL_DIRECTION_METAL_BALL_FACTORY_Y: [u8; 16] = [
    16, 16, 16, 16, 64, 96, 160, 192, 240, 240, 240, 240, 192, 160, 96, 64,
];
const K_OVERLORD_SPAWN_BALL_X_VEL: [i8; 4] = [24, -24, 0, 0];
const K_OVERLORD_SPAWN_BALL_Y_VEL: [i8; 4] = [0, 0, 24, -24];
const K_OVERLORD_IN_RANGE_OFFS: [i16; 2] = [0x130, -0x40];
const K_ARMOS_COORDINATOR_TAB0: [u16; 6] = [0, 425, 340, 255, 170, 85];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SpawnInfo {
    pub overlord_x: u16,
    pub overlord_y: u16,
}

pub trait OverlordHooks {
    fn random_number(&mut self, ram: &mut Ram<'_>) -> u8;

    fn spawn_sprite(
        &mut self,
        ram: &mut Ram<'_>,
        source: usize,
        sprite_type: u8,
        highest_slot: usize,
    ) -> Option<(usize, SpawnInfo)>;

    fn tile_attribute(&mut self, ram: &mut Ram<'_>, floor: u8, x: &mut u16, y: u16) -> u8;
    fn garnish_alloc(&mut self, ram: &mut Ram<'_>) -> Option<usize>;
    fn calculate_sfx_pan(&mut self, ram: &mut Ram<'_>, x: u8) -> u8;
    fn queue_sfx2_with_pan(&mut self, ram: &mut Ram<'_>, sprite: usize, sfx: u8);
    fn queue_sfx3_with_pan(&mut self, ram: &mut Ram<'_>, sprite: usize, sfx: u8);
    fn transmute_to_bomb(&mut self, ram: &mut Ram<'_>, sprite: usize);
    fn sinus_lookup(&self, angle: u8) -> u16;
}

#[inline]
fn read_index(ram: &Ram<'_>, base: usize, index: usize) -> u8 {
    ram.read_u8(base + index)
}

#[inline]
fn write_index(ram: &mut Ram<'_>, base: usize, index: usize, value: u8) {
    ram.write_u8(base + index, value);
}

#[inline]
fn add_i8(value: u16, offset: i8) -> u16 {
    value.wrapping_add(offset as i16 as u16)
}

#[must_use]
pub fn get_x(ram: &Ram<'_>, k: usize) -> u16 {
    CONTROLLER_X.pixels(ram, k)
}

#[must_use]
pub fn get_y(ram: &Ram<'_>, k: usize) -> u16 {
    CONTROLLER_Y.pixels(ram, k)
}

pub fn set_x(ram: &mut Ram<'_>, k: usize, value: u16) {
    CONTROLLER_X.set_pixels(ram, k, value);
}

pub fn set_y(ram: &mut Ram<'_>, k: usize, value: u16) {
    CONTROLLER_Y.set_pixels(ram, k, value);
}

#[must_use]
pub fn armos_mult(a: u16, b: u8) -> u8 {
    if a >= 256 {
        b
    } else {
        let product = u32::from(a) * u32::from(b);
        ((product >> 8) + ((product >> 7) & 1)) as u8
    }
}

#[must_use]
pub fn armos_sin(hooks: &impl OverlordHooks, angle: u16, radius: u8) -> i8 {
    let value = armos_mult(hooks.sinus_lookup(angle as u8), radius);
    if angle & 0x100 != 0 {
        value.wrapping_neg() as i8
    } else {
        value as i8
    }
}

fn sprite_set_x(ram: &mut Ram<'_>, k: usize, value: u16) {
    SplitCoordinate::new(SPRITE_X_LO, SPRITE_X_HI).set_pixels(ram, k, value);
}

fn sprite_set_y(ram: &mut Ram<'_>, k: usize, value: u16) {
    SplitCoordinate::new(SPRITE_Y_LO, SPRITE_Y_HI).set_pixels(ram, k, value);
}

pub fn main(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks) {
    execute_all(ram, hooks);
    spawn_boulder(ram, hooks);
}

pub fn execute_all(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks) {
    if ram.read_u8(0x11) | ram.read_u8(0xfc1) != 0 {
        return;
    }
    for k in (0..8).rev() {
        if read_index(ram, OVERLORD_TYPE, k) != 0 {
            execute_single(ram, hooks, k);
        }
    }
}

pub fn execute_single(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let kind = read_index(ram, OVERLORD_TYPE, k);
    check_if_active(ram, k);
    match kind {
        1 => position_target(ram, k),
        2 => full_room_cannons(ram, hooks, k),
        3 => vertical_cannon(ram, hooks, k),
        4 => stalfos_factory(k),
        5 => falling_stalfos(ram, hooks, k),
        6 | 26 => bad_switch_snake(ram, hooks, k),
        7 => moving_floor(ram, hooks, k),
        8 => blob_spawner(ram, hooks, k),
        9 => wallmaster_spawner(ram, hooks, k),
        10..=15 => falling_square(ram, hooks, k),
        16..=19 => pirogusu_spawner_left(ram, hooks, k),
        20 => tile_room(ram, hooks, k),
        21 => wizzrobe_spawner(ram, hooks, k),
        22 => zoro_spawner(ram, hooks, k),
        23 => pot_trap(ram, k),
        24 => invisible_stalfos(ram, hooks, k),
        25 => armos_coordinator_bounce(ram, hooks, k),
        _ => unreachable!("invalid overlord type {kind}"),
    }
}

fn stalfos_factory(k: usize) -> ! {
    panic!("unused Overlord_StalfosFactory invoked for slot {k}")
}

pub fn spawn_boulder(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks) {
    if ram.read_u8(0x1b) != 0
        || ram.read_u8(0xffd) == 0
        || ram.read_u8(0x11) | ram.read_u8(0xfc1) != 0
    {
        return;
    }
    let counter = ram.read_u8(0xffe).wrapping_add(1);
    ram.write_u8(0xffe, counter);
    if counter & 63 != 0 {
        return;
    }
    let vertical_delta = ((ram.read_u16(0xe8) >> 8) as u8)
        .wrapping_sub((ram.read_u16(0xfbe) >> 8) as u8)
        .wrapping_sub(2);
    if (vertical_delta as i8) < 0 {
        return;
    }
    if let Some((j, _)) = hooks.spawn_sprite(ram, 0, 0xc2, 15) {
        let x = ram
            .read_u16(0xe2)
            .wrapping_add(u16::from(hooks.random_number(ram) & 127))
            .wrapping_add(64);
        sprite_set_x(ram, j, x);
        sprite_set_y(ram, j, ram.read_u16(0xe8).wrapping_sub(0x30));
        write_index(ram, SPRITE_FLOOR, j, 0);
        write_index(ram, SPRITE_D, j, 0);
        write_index(ram, SPRITE_Z, j, 0);
    }
}

pub fn armos_coordinator_bounce(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let timer = read_index(ram, OVERLORD_GEN2, k);
    if timer != 0 {
        write_index(ram, OVERLORD_GEN2, k, timer - 1);
    }
    match read_index(ram, OVERLORD_GEN1, k) {
        0 => {
            if read_index(ram, SPRITE_A, 0) != 0 {
                write_index(ram, OVERLORD_X_LO, k, 120);
                write_index(ram, OVERLORD_FLOOR, k, 255);
                write_index(ram, OVERLORD_X_LO, 2, 64);
                write_index(ram, OVERLORD_X_LO, 0, 192);
                write_index(ram, OVERLORD_X_LO, 1, 1);
                armos_coordinator_rotate_knights(ram, hooks, k);
            }
        }
        1 => {
            if armos_coordinator_check_knights(ram) {
                let value = read_index(ram, OVERLORD_GEN1, k).wrapping_add(1);
                write_index(ram, OVERLORD_GEN1, k, value);
                write_index(ram, OVERLORD_GEN2, k, 0xff);
            }
        }
        2 | 4 => armos_coordinator_rotate_knights(ram, hooks, k),
        3 => {
            let radius = read_index(ram, OVERLORD_X_LO, 2).wrapping_sub(1);
            write_index(ram, OVERLORD_X_LO, 2, radius);
            if radius == 32 {
                let state = read_index(ram, OVERLORD_GEN1, k).wrapping_add(1);
                write_index(ram, OVERLORD_GEN1, k, state);
                write_index(ram, OVERLORD_GEN2, k, 64);
            }
            armos_coordinator_rotate(ram, hooks, k);
        }
        5 => {
            let radius = read_index(ram, OVERLORD_X_LO, 2).wrapping_add(1);
            write_index(ram, OVERLORD_X_LO, 2, radius);
            if radius == 64 {
                let state = read_index(ram, OVERLORD_GEN1, k).wrapping_add(1);
                write_index(ram, OVERLORD_GEN1, k, state);
                write_index(ram, OVERLORD_GEN2, k, 64);
            }
            armos_coordinator_rotate(ram, hooks, k);
        }
        6 => {
            if read_index(ram, OVERLORD_GEN2, k) != 0 {
                return;
            }
            armos_coordinator_disable_coercion(ram);
            for j in (0..6).rev() {
                write_index(ram, OVERLORD_X_HI, j, K_ARMOS_COORDINATOR_BACK_WALL_X[j]);
                write_index(ram, OVERLORD_GEN2, j, 48);
            }
            let state = read_index(ram, OVERLORD_GEN1, k).wrapping_add(1);
            write_index(ram, OVERLORD_GEN1, k, state);
            write_index(ram, OVERLORD_GEN2, k, 255);
        }
        7 => {
            if read_index(ram, OVERLORD_GEN2, k) != 0 {
                return;
            }
            for j in (0..6).rev() {
                let value = read_index(ram, OVERLORD_GEN2, j).wrapping_add(1);
                write_index(ram, OVERLORD_GEN2, j, value);
                if value == 192 {
                    write_index(ram, OVERLORD_GEN1, k, 1);
                    let direction = read_index(ram, OVERLORD_FLOOR, k).wrapping_neg();
                    write_index(ram, OVERLORD_FLOOR, k, direction);
                    armos_coordinator_disable_coercion(ram);
                    armos_coordinator_rotate(ram, hooks, k);
                    return;
                }
            }
        }
        _ => {}
    }
}

pub fn invisible_stalfos(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    if get_x(ram, k)
        .wrapping_sub(ram.read_u16(0x22))
        .wrapping_add(24)
        >= 48
        || get_y(ram, k)
            .wrapping_sub(ram.read_u16(0x20))
            .wrapping_add(24)
            >= 48
    {
        return;
    }
    write_index(ram, OVERLORD_TYPE, k, 0);
    ram.write_u8(0xfb5, 3);
    loop {
        let i = usize::from(ram.read_u8(0xfb5));
        let Some((j, _)) = hooks.spawn_sprite(ram, k, 0xa7, 12) else {
            return;
        };
        sprite_set_x(ram, j, add_i8(ram.read_u16(0x22), K_RED_STALFOS_TRAP_X[i]));
        sprite_set_y(ram, j, add_i8(ram.read_u16(0x20), K_RED_STALFOS_TRAP_Y[i]));
        write_index(ram, SPRITE_DELAY_MAIN, j, K_RED_STALFOS_TRAP_DELAY[i]);
        write_index(ram, SPRITE_FLOOR, j, read_index(ram, OVERLORD_FLOOR, k));
        write_index(ram, SPRITE_E, j, 1);
        write_index(ram, SPRITE_FLAGS2, j, 3);
        write_index(ram, SPRITE_D, j, 2);
        let next = (i as u8).wrapping_sub(1);
        ram.write_u8(0xfb5, next);
        if (next as i8) < 0 {
            break;
        }
    }
}

pub fn pot_trap(ram: &mut Ram<'_>, k: usize) {
    if get_x(ram, k)
        .wrapping_sub(ram.read_u16(0x22))
        .wrapping_add(32)
        < 64
        && get_y(ram, k)
            .wrapping_sub(ram.read_u16(0x20))
            .wrapping_add(32)
            < 64
    {
        write_index(ram, OVERLORD_TYPE, k, 0);
        ram.write_u8(0xb9e, ram.read_u8(0xb9e).wrapping_add(1));
    }
}

pub fn zoro_spawner(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let timer = read_index(ram, OVERLORD_GEN2, k).wrapping_sub(1);
    write_index(ram, OVERLORD_GEN2, k, timer);
    let mut x = get_x(ram, k).wrapping_add(8);
    let y = get_y(ram, k).wrapping_add(8);
    if hooks.tile_attribute(ram, read_index(ram, OVERLORD_FLOOR, k), &mut x, y) != 0x82
        || timer >= 0x18
        || timer & 3 != 0
    {
        return;
    }
    if let Some((j, info)) = hooks.spawn_sprite(ram, k, 0x9c, 12) {
        let offset = K_OVERLORD_ZORO_FACTORY_X[usize::from(hooks.random_number(ram) & 7)];
        sprite_set_x(ram, j, add_i8(info.overlord_x, offset).wrapping_add(8));
        write_index(ram, SPRITE_Y_LO, j, (info.overlord_y as u8).wrapping_add(8));
        write_index(ram, SPRITE_Y_HI, j, (info.overlord_y >> 8) as u8);
        write_index(ram, SPRITE_FLOOR, j, read_index(ram, OVERLORD_FLOOR, k));
        write_index(ram, SPRITE_FLAGS4, j, 1);
        write_index(ram, SPRITE_E, j, 1);
        write_index(ram, SPRITE_IGNORE_PROJECTILE, j, 1);
        write_index(ram, SPRITE_Y_VEL, j, 16);
        write_index(ram, SPRITE_FLAGS2, j, 32);
        write_index(ram, SPRITE_OAM_FLAGS, j, 13);
        let subtype = hooks.random_number(ram);
        write_index(ram, SPRITE_SUBTYPE2, j, subtype);
        write_index(ram, SPRITE_DELAY_MAIN, j, 48);
        write_index(ram, SPRITE_BUMP_DAMAGE, j, 3);
    }
}

pub fn wizzrobe_spawner(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let timer = read_index(ram, OVERLORD_GEN2, k);
    if timer != 128 {
        if ram.read_u8(0x1a) & 1 != 0 {
            write_index(ram, OVERLORD_GEN2, k, timer.wrapping_sub(1));
        }
        return;
    }
    write_index(ram, OVERLORD_GEN2, k, 127);
    for i in (0..4).rev() {
        if let Some((j, _)) = hooks.spawn_sprite(ram, k, 0x9b, 12) {
            sprite_set_x(ram, j, add_i8(ram.read_u16(0x22), K_OVERLORD_WIZZROBE_X[i]));
            sprite_set_y(ram, j, add_i8(ram.read_u16(0x20), K_OVERLORD_WIZZROBE_Y[i]));
            write_index(ram, SPRITE_DELAY_MAIN, j, K_OVERLORD_WIZZROBE_DELAY[i]);
            write_index(ram, SPRITE_FLOOR, j, read_index(ram, OVERLORD_FLOOR, k));
            write_index(ram, SPRITE_B, j, 1);
        }
    }
    ram.write_u8(0xfb5, 0xff);
}

pub fn tile_room(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let x = get_x(ram, k).wrapping_sub(ram.read_u16(0xe2));
    let y = get_y(ram, k).wrapping_sub(ram.read_u16(0xe8));
    if x & 0xff00 != 0 || y & 0xff00 != 0 {
        return;
    }
    let timer = read_index(ram, OVERLORD_GEN2, k).wrapping_sub(1);
    write_index(ram, OVERLORD_GEN2, k, timer);
    if timer != 0x80 {
        return;
    }
    if tile_room_spawn_tile(ram, hooks, k).is_none() {
        write_index(ram, OVERLORD_GEN2, k, 0x81);
        return;
    }
    let count = read_index(ram, OVERLORD_GEN1, k).wrapping_add(1);
    write_index(ram, OVERLORD_GEN1, k, count);
    if count != 22 {
        write_index(ram, OVERLORD_GEN2, k, 0xe0);
    } else {
        write_index(ram, OVERLORD_TYPE, k, 0);
    }
}

pub fn tile_room_spawn_tile(
    ram: &mut Ram<'_>,
    hooks: &mut impl OverlordHooks,
    k: usize,
) -> Option<usize> {
    let (j, _) = hooks.spawn_sprite(ram, k, 0x94, 15)?;
    write_index(ram, SPRITE_E, j, 1);
    let i = usize::from(read_index(ram, OVERLORD_GEN1, k));
    write_index(ram, SPRITE_X_LO, j, K_SPAWN_FLYING_TILE_X[i]);
    write_index(
        ram,
        SPRITE_Y_LO,
        j,
        K_SPAWN_FLYING_TILE_Y[i].wrapping_sub(8),
    );
    write_index(ram, SPRITE_Y_HI, j, read_index(ram, OVERLORD_Y_HI, k));
    write_index(ram, SPRITE_X_HI, j, read_index(ram, OVERLORD_X_HI, k));
    write_index(ram, SPRITE_FLOOR, j, read_index(ram, OVERLORD_FLOOR, k));
    write_index(ram, SPRITE_HEALTH, j, 4);
    write_index(ram, SPRITE_FLAGS5, j, 0);
    write_index(ram, SPRITE_HEALTH, j, 0);
    ram.write_u8(0xcaa + j, 8);
    write_index(ram, SPRITE_FLAGS2, j, 4);
    write_index(ram, SPRITE_OAM_FLAGS, j, 1);
    write_index(ram, SPRITE_BUMP_DAMAGE, j, 4);
    Some(j)
}

pub fn pirogusu_spawner_left(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let subtype = read_index(ram, OVERLORD_TYPE, k).wrapping_sub(16);
    ram.write_u8(0xfb5, subtype);
    let timer = read_index(ram, OVERLORD_GEN2, k);
    if timer != 128 {
        write_index(ram, OVERLORD_GEN2, k, timer.wrapping_sub(1));
        return;
    }
    let delay = (hooks.random_number(ram) & 31).wrapping_add(96);
    write_index(ram, OVERLORD_GEN2, k, delay);
    let count = (0..16)
        .filter(|&i| {
            read_index(ram, SPRITE_STATE, i) != 0 && read_index(ram, SPRITE_TYPE, i) == 0x10
        })
        .count();
    if count >= 5 {
        return;
    }
    if let Some((j, info)) = hooks.spawn_sprite(ram, k, 0x94, 12) {
        sprite_set_x(ram, j, info.overlord_x);
        sprite_set_y(ram, j, info.overlord_y);
        write_index(ram, SPRITE_FLOOR, j, read_index(ram, OVERLORD_FLOOR, k));
        write_index(ram, SPRITE_DELAY_MAIN, j, 32);
        write_index(ram, SPRITE_D, j, subtype);
        write_index(
            ram,
            SPRITE_A,
            j,
            K_OVERLORD_PIROGUSU_A[usize::from(subtype)],
        );
    }
}

pub fn falling_square(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let timer = read_index(ram, OVERLORD_GEN2, k);
    if timer != 0 {
        if read_index(ram, OVERLORD_GEN3, k) != 0 {
            write_index(ram, OVERLORD_GEN2, k, timer.wrapping_sub(1));
            return;
        }
        let x = get_x(ram, k).wrapping_sub(ram.read_u16(0xe2));
        let y = get_y(ram, k).wrapping_sub(ram.read_u16(0xe8));
        if x & 0xff00 == 0 && y & 0xff00 == 0 {
            let value = read_index(ram, OVERLORD_GEN3, k).wrapping_add(1);
            write_index(ram, OVERLORD_GEN3, k, value);
        }
        return;
    }

    write_index(ram, OVERLORD_GEN2, k, 16);
    spawn_falling_tile(ram, hooks, k);
    let path = usize::from(read_index(ram, OVERLORD_TYPE, k).wrapping_sub(10));
    let step = read_index(ram, OVERLORD_GEN1, k);
    write_index(ram, OVERLORD_GEN1, k, step.wrapping_add(1));
    let length = K_CRUMBLE_TILE_PATH_OFFS[path + 1] - K_CRUMBLE_TILE_PATH_OFFS[path];
    if step == length {
        write_index(ram, OVERLORD_TYPE, k, 0);
    }
    let command =
        K_CRUMBLE_TILE_PATH_DATA[usize::from(K_CRUMBLE_TILE_PATH_OFFS[path].wrapping_add(step))];
    if command == 0xff {
        set_x(ram, k, get_x(ram, k).wrapping_add(0x0c1a));
        set_y(ram, k, get_y(ram, k).wrapping_add(0xbb66));
    } else {
        set_x(
            ram,
            k,
            add_i8(get_x(ram, k), K_CRUMBLE_TILE_PATH_X[usize::from(command)]),
        );
        set_y(
            ram,
            k,
            add_i8(get_y(ram, k), K_CRUMBLE_TILE_PATH_Y[usize::from(command)]),
        );
    }
}

pub fn spawn_falling_tile(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let Some(j) = hooks.garnish_alloc(ram) else {
        return;
    };
    ram.set_garnish_kind(j, 3);
    ram.set_garnish_x_high(j, read_index(ram, OVERLORD_X_HI, k));
    ram.set_garnish_x_low(j, read_index(ram, OVERLORD_X_LO, k));
    let pan = hooks.calculate_sfx_pan(ram, ram.garnish_x_low(j));
    ram.write_u8(0x12e, pan | 0x1f);
    let y = get_y(ram, k).wrapping_add(16);
    ram.set_garnish_y_low(j, y as u8);
    ram.set_garnish_y_high(j, (y >> 8) as u8);
    ram.set_garnish_countdown(j, 31);
    ram.set_garnish_active_marker(31);
}

pub fn wallmaster_spawner(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let timer = read_index(ram, OVERLORD_GEN2, k);
    if timer != 128 {
        if ram.read_u8(0x1a) & 1 == 0 {
            write_index(ram, OVERLORD_GEN2, k, timer.wrapping_sub(1));
        }
        return;
    }
    write_index(ram, OVERLORD_GEN2, k, 127);
    let Some((j, _)) = hooks.spawn_sprite(ram, k, 0x90, 12) else {
        return;
    };
    sprite_set_x(ram, j, ram.read_u16(0x22));
    sprite_set_y(ram, j, ram.read_u16(0x20));
    write_index(ram, SPRITE_Z, j, 208);
    hooks.queue_sfx2_with_pan(ram, j, 0x20);
    write_index(ram, SPRITE_FLOOR, j, ram.read_u8(0xee));
}

pub fn blob_spawner(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let timer = read_index(ram, OVERLORD_GEN2, k);
    if timer != 0 {
        write_index(ram, OVERLORD_GEN2, k, timer.wrapping_sub(1));
        return;
    }
    write_index(ram, OVERLORD_GEN2, k, 0xa0);
    let count = (0..16)
        .filter(|&i| {
            read_index(ram, SPRITE_STATE, i) != 0 && read_index(ram, SPRITE_TYPE, i) == 0x8f
        })
        .count();
    if count >= 5 {
        return;
    }
    if let Some((j, _)) = hooks.spawn_sprite(ram, k, 0x8f, 12) {
        let direction = usize::from(ram.read_u8(0x2f) >> 1);
        sprite_set_x(
            ram,
            j,
            add_i8(ram.read_u16(0x22), K_OVERLORD_ZOL_X[direction]),
        );
        sprite_set_y(
            ram,
            j,
            add_i8(ram.read_u16(0x20), K_OVERLORD_ZOL_Y[direction]),
        );
        write_index(ram, SPRITE_Z, j, 192);
        write_index(ram, SPRITE_FLOOR, j, ram.read_u8(0xee));
        write_index(ram, SPRITE_AI_STATE, j, 2);
        write_index(ram, SPRITE_E, j, 2);
        write_index(ram, SPRITE_C, j, 2);
        let heading = (hooks.random_number(ram) & 31) | 16;
        write_index(ram, SPRITE_HEAD_DIR, j, heading);
    }
}

pub fn moving_floor(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    if read_index(ram, SPRITE_STATE, 0) == 4 {
        write_index(ram, OVERLORD_TYPE, k, 0);
        ram.write_u8(0x41a, 1);
        return;
    }
    if read_index(ram, OVERLORD_GEN1, k) == 0 {
        let counter = read_index(ram, OVERLORD_GEN2, k).wrapping_add(1);
        write_index(ram, OVERLORD_GEN2, k, counter);
        if counter == 32 {
            write_index(ram, OVERLORD_GEN2, k, 0);
            let mask = if read_index(ram, OVERLORD_X_LO, k) != 0 {
                3
            } else {
                1
            };
            let direction = (hooks.random_number(ram) & mask) * 2;
            ram.write_u8(0x41a, direction);
            let delay = (hooks.random_number(ram) & 127).wrapping_add(128);
            write_index(ram, OVERLORD_GEN2, k, delay);
            write_index(ram, OVERLORD_GEN1, k, 1);
        } else {
            ram.write_u8(0x41a, 1);
        }
    } else {
        let timer = read_index(ram, OVERLORD_GEN2, k).wrapping_sub(1);
        write_index(ram, OVERLORD_GEN2, k, timer);
        if timer == 0 {
            write_index(ram, OVERLORD_GEN1, k, 0);
        }
    }
}

pub fn play_falling_sfx(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    hooks.queue_sfx2_with_pan(ram, k, 0x20);
}

pub fn falling_stalfos(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let x = get_x(ram, k).wrapping_sub(ram.read_u16(0xe2));
    let y = get_y(ram, k).wrapping_sub(ram.read_u16(0xe8));
    if x & 0xff00 != 0 || y & 0xff00 != 0 {
        return;
    }
    let state = read_index(ram, OVERLORD_GEN1, k);
    if state == 0 {
        if ram.read_u8(0xb9e) != 0 {
            write_index(ram, OVERLORD_GEN1, k, 1);
        }
        return;
    }
    write_index(ram, OVERLORD_GEN1, k, state.wrapping_add(1));
    if state != K_STALFOS_TRAP_TRIGGER[k] {
        return;
    }
    write_index(ram, OVERLORD_TYPE, k, 0);
    let Some((j, info)) = hooks.spawn_sprite(ram, k, 0x85, 12) else {
        return;
    };
    sprite_set_x(ram, j, info.overlord_x);
    sprite_set_y(ram, j, info.overlord_y);
    write_index(ram, SPRITE_Z, j, 224);
    write_index(ram, SPRITE_FLOOR, j, read_index(ram, OVERLORD_FLOOR, k));
    write_index(ram, SPRITE_D, j, 0);
    play_falling_sfx(ram, hooks, j);
}

pub fn bad_switch_snake(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let counter = read_index(ram, OVERLORD_GEN1, k);
    if counter == 0 {
        if ram.read_u8(0xcf4) != 0 {
            write_index(ram, OVERLORD_GEN1, k, 1);
        }
        return;
    }
    write_index(ram, OVERLORD_GEN1, k, counter.wrapping_add(1));
    if counter != K_SNAKE_TRAP_OVERLORD_TAB1[k] {
        return;
    }
    let Some((j, info)) = hooks.spawn_sprite(ram, k, 0x6e, 15) else {
        return;
    };
    sprite_set_x(ram, j, info.overlord_x);
    sprite_set_y(ram, j, info.overlord_y);
    write_index(ram, SPRITE_Z, j, 192);
    write_index(ram, SPRITE_E, j, 192);
    let flags = read_index(ram, SPRITE_FLAGS3, j) | 0x10;
    write_index(ram, SPRITE_FLAGS3, j, flags);
    write_index(ram, SPRITE_FLOOR, j, read_index(ram, OVERLORD_FLOOR, k));
    hooks.queue_sfx2_with_pan(ram, j, 0x20);
    let kind = read_index(ram, OVERLORD_TYPE, k);
    write_index(ram, OVERLORD_TYPE, k, 0);
    if kind == 26 {
        write_index(ram, SPRITE_TYPE, j, 74);
        hooks.transmute_to_bomb(ram, j);
        write_index(ram, SPRITE_DELAY_AUX1, j, 112);
    }
}

pub fn full_room_cannons(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let x = get_x(ram, k).wrapping_sub(ram.read_u16(0xe2));
    let y = get_y(ram, k).wrapping_sub(ram.read_u16(0xe8));
    if (x | y) & 0xff00 != 0 || ram.read_u8(0x1a) & 0xf != 0 {
        return;
    }
    ram.write_u8(0xfb6, 0);
    let choice = usize::from(hooks.random_number(ram) & 15);
    ram.write_u8(0xfb5, K_ALL_DIRECTION_METAL_BALL_FACTORY_IDX[choice]);
    write_index(
        ram,
        OVERLORD_X_LO,
        k,
        K_ALL_DIRECTION_METAL_BALL_FACTORY_X[choice],
    );
    write_index(ram, OVERLORD_X_HI, k, ram.read_u8(0xfb0));
    write_index(
        ram,
        OVERLORD_Y_LO,
        k,
        K_ALL_DIRECTION_METAL_BALL_FACTORY_Y[choice],
    );
    write_index(ram, OVERLORD_Y_HI, k, ram.read_u8(0xfb1).wrapping_add(1));
    spawn_cannon_ball(ram, hooks, k, 0);
}

pub fn vertical_cannon(ram: &mut Ram<'_>, hooks: &mut impl OverlordHooks, k: usize) {
    let x = get_x(ram, k).wrapping_sub(ram.read_u16(0xe2));
    if x & 0xff00 != 0 {
        write_index(ram, OVERLORD_GEN2, k, 255);
        return;
    }
    let timer = read_index(ram, OVERLORD_GEN2, k);
    if ram.read_u8(0x1a) & 1 == 0 && timer != 0 {
        write_index(ram, OVERLORD_GEN2, k, timer - 1);
    }
    ram.write_u8(0xfb5, 2);
    ram.write_u8(0xfb6, 0);
    let counter = read_index(ram, OVERLORD_GEN1, k).wrapping_sub(1);
    write_index(ram, OVERLORD_GEN1, k, counter);
    if counter as i8 >= 0 {
        return;
    }
    write_index(ram, OVERLORD_GEN1, k, 56);
    let x_offset = if read_index(ram, OVERLORD_GEN2, k) == 0 {
        write_index(ram, OVERLORD_GEN2, k, 160);
        ram.write_u8(0xfb6, 160);
        8
    } else {
        i16::from((hooks.random_number(ram) & 2) * 8)
    };
    spawn_cannon_ball(ram, hooks, k, x_offset);
}

pub fn spawn_cannon_ball(
    ram: &mut Ram<'_>,
    hooks: &mut impl OverlordHooks,
    k: usize,
    x_offset: i16,
) {
    let Some((j, info)) = hooks.spawn_sprite(ram, k, 0x50, 15) else {
        return;
    };
    sprite_set_x(ram, j, info.overlord_x.wrapping_add(x_offset as u16));
    sprite_set_y(ram, j, info.overlord_y.wrapping_sub(1));
    let direction = usize::from(ram.read_u8(0xfb5));
    write_index(
        ram,
        SPRITE_X_VEL,
        j,
        K_OVERLORD_SPAWN_BALL_X_VEL[direction] as u8,
    );
    write_index(
        ram,
        SPRITE_Y_VEL,
        j,
        K_OVERLORD_SPAWN_BALL_Y_VEL[direction] as u8,
    );
    write_index(ram, SPRITE_FLOOR, j, read_index(ram, OVERLORD_FLOOR, k));
    let special = ram.read_u8(0xfb6);
    if special != 0 {
        write_index(ram, SPRITE_AI_STATE, j, special);
        let y = read_index(ram, SPRITE_Y_LO, j).wrapping_add(8);
        write_index(ram, SPRITE_Y_LO, j, y);
        write_index(ram, SPRITE_FLAGS2, j, 3);
        write_index(ram, SPRITE_FLAGS4, j, 9);
    }
    write_index(ram, SPRITE_DELAY_AUX2, j, 64);
    hooks.queue_sfx3_with_pan(ram, j, 7);
}

pub fn position_target(ram: &mut Ram<'_>, k: usize) {
    ram.write_u8(0xfde, k as u8);
}

pub fn check_if_active(ram: &mut Ram<'_>, k: usize) {
    if ram.read_u8(0x1b) != 0 {
        return;
    }
    let j = usize::from(ram.read_u8(0x1a) & 1);
    let x = ram
        .read_u16(0xe2)
        .wrapping_add(K_OVERLORD_IN_RANGE_OFFS[j] as u16)
        .wrapping_sub(get_x(ram, k));
    let y = ram
        .read_u16(0xe8)
        .wrapping_add(K_OVERLORD_IN_RANGE_OFFS[j] as u16)
        .wrapping_sub(get_y(ram, k));
    if usize::from(x >> 15) == j && usize::from(y >> 15) == j {
        return;
    }
    write_index(ram, OVERLORD_TYPE, k, 0);
    let block = ram.read_u16(OVERLORD_OFFSET_SPRITE_POS + k * 2);
    if block != 0xffff {
        let mask = 0x80 >> (block & 7);
        let address = OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3);
        ram.write_u8(address, ram.read_u8(address) & !(mask as u8));
    }
}

pub fn armos_coordinator_rotate_knights(ram: &mut Ram<'_>, hooks: &impl OverlordHooks, k: usize) {
    if read_index(ram, OVERLORD_GEN2, k) == 0 {
        let state = read_index(ram, OVERLORD_GEN1, k).wrapping_add(1);
        write_index(ram, OVERLORD_GEN1, k, state);
    }
    armos_coordinator_rotate(ram, hooks, k);
}

pub fn armos_coordinator_rotate(ram: &mut Ram<'_>, hooks: &impl OverlordHooks, k: usize) {
    let angle = ram
        .read_u16(OVERLORD_X_LO)
        .wrapping_add(read_index(ram, OVERLORD_FLOOR, k) as i8 as i16 as u16);
    ram.write_u16(OVERLORD_X_LO, angle);
    for (i, phase) in K_ARMOS_COORDINATOR_TAB0.iter().copied().enumerate() {
        let theta = angle.wrapping_add(phase);
        let radius = read_index(ram, OVERLORD_X_LO, 2);
        let x = add_i8(get_x(ram, k), armos_sin(hooks, theta, radius));
        write_index(ram, OVERLORD_X_HI, i, x as u8);
        write_index(ram, OVERLORD_Y_HI, i, (x >> 8) as u8);
        let y = add_i8(
            get_y(ram, k),
            armos_sin(hooks, theta.wrapping_add(0x80), radius),
        );
        write_index(ram, OVERLORD_GEN2, i, y as u8);
        write_index(ram, OVERLORD_FLOOR, i, (y >> 8) as u8);
    }
    ram.write_u8(0xfb5, 6);
}

#[must_use]
pub fn armos_coordinator_check_knights(ram: &Ram<'_>) -> bool {
    (0..6)
        .rev()
        .all(|j| read_index(ram, SPRITE_STATE, j) == 0 || read_index(ram, SPRITE_AI_STATE, j) != 0)
}

pub fn armos_coordinator_disable_coercion(ram: &mut Ram<'_>) {
    for j in (0..6).rev() {
        write_index(ram, SPRITE_AI_STATE, j, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[test]
    fn split_controller_coordinates_touch_only_selected_low_and_high_bytes() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for slot in [0, 7] {
            for value in [0, 0xff, 0x100, 0xffff] {
                let mut expected = ram.slice(0, WRAM_SIZE).to_vec();
                set_x(&mut ram, slot, value);
                set_y(&mut ram, slot, value.wrapping_add(1));
                let [x_low, x_high] = value.to_le_bytes();
                let [y_low, y_high] = value.wrapping_add(1).to_le_bytes();
                expected[0xb08 + slot] = x_low;
                expected[0xb10 + slot] = x_high;
                expected[0xb18 + slot] = y_low;
                expected[0xb20 + slot] = y_high;
                assert_eq!(get_x(&ram, slot), value);
                assert_eq!(get_y(&ram, slot), value.wrapping_add(1));
                assert_eq!(ram.slice(0, WRAM_SIZE), expected);
            }
        }
    }

    #[derive(Default)]
    struct Hooks {
        random: Vec<u8>,
        random_pos: usize,
        spawn_slots: Vec<usize>,
        spawn_pos: usize,
        tile: u8,
        garnish: Option<usize>,
        garnish_pan_mutation: bool,
        garnish_pan_inputs: Vec<u8>,
        sfx2: Vec<(usize, u8)>,
        sfx3: Vec<(usize, u8)>,
        bombs: Vec<usize>,
    }

    impl OverlordHooks for Hooks {
        fn random_number(&mut self, _ram: &mut Ram<'_>) -> u8 {
            let result = self.random.get(self.random_pos).copied().unwrap_or(0);
            self.random_pos += 1;
            result
        }

        fn spawn_sprite(
            &mut self,
            ram: &mut Ram<'_>,
            source: usize,
            sprite_type: u8,
            _highest_slot: usize,
        ) -> Option<(usize, SpawnInfo)> {
            let slot = self.spawn_slots.get(self.spawn_pos).copied()?;
            self.spawn_pos += 1;
            write_index(ram, SPRITE_STATE, slot, 9);
            write_index(ram, SPRITE_TYPE, slot, sprite_type);
            Some((
                slot,
                SpawnInfo {
                    overlord_x: get_x(ram, source),
                    overlord_y: get_y(ram, source),
                },
            ))
        }

        fn tile_attribute(&mut self, _ram: &mut Ram<'_>, _floor: u8, _x: &mut u16, _y: u16) -> u8 {
            self.tile
        }

        fn garnish_alloc(&mut self, _ram: &mut Ram<'_>) -> Option<usize> {
            self.garnish.take()
        }

        fn calculate_sfx_pan(&mut self, ram: &mut Ram<'_>, x: u8) -> u8 {
            if self.garnish_pan_mutation {
                self.garnish_pan_inputs.push(x);
                ram.set_garnish_kind(2, 9);
                ram.set_garnish_x_low(2, 0x55);
                ram.set_garnish_countdown(2, 0xbb);
                ram.set_garnish_active_marker(0xaa);
                set_y(ram, 0, 0x3344);
            }
            x & 0xc0
        }

        fn queue_sfx2_with_pan(&mut self, _ram: &mut Ram<'_>, sprite: usize, sfx: u8) {
            self.sfx2.push((sprite, sfx));
        }

        fn queue_sfx3_with_pan(&mut self, _ram: &mut Ram<'_>, sprite: usize, sfx: u8) {
            self.sfx3.push((sprite, sfx));
        }

        fn transmute_to_bomb(&mut self, _ram: &mut Ram<'_>, sprite: usize) {
            self.bombs.push(sprite);
        }

        fn sinus_lookup(&self, angle: u8) -> u16 {
            match angle {
                0 => 0,
                64 => 181,
                128 => 256,
                192 => 181,
                _ => 0,
            }
        }
    }

    #[test]
    fn coordinates_and_armos_rounding_preserve_word_layout() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        set_x(&mut ram, 3, 0xabcd);
        set_y(&mut ram, 3, 0x1234);
        assert_eq!(get_x(&ram, 3), 0xabcd);
        assert_eq!(get_y(&ram, 3), 0x1234);
        assert_eq!(armos_mult(256, 19), 19);
        assert_eq!(armos_mult(128, 255), 128);
        assert_eq!(armos_mult(1, 127), 0);
        assert_eq!(armos_mult(1, 128), 1);
    }

    #[test]
    fn execute_single_dispatches_using_type_captured_before_deactivation() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut hooks = Hooks::default();
        write_index(&mut ram, OVERLORD_TYPE, 2, 1);
        set_x(&mut ram, 2, 0x5000);
        set_y(&mut ram, 2, 0x5000);
        ram.write_u16(OVERLORD_OFFSET_SPRITE_POS + 4, 0xffff);
        execute_single(&mut ram, &mut hooks, 2);
        assert_eq!(read_index(&ram, OVERLORD_TYPE, 2), 0);
        assert_eq!(ram.read_u8(0xfde), 2);
    }

    #[test]
    fn pot_trap_uses_wrapping_box_collision_and_arms_stalfos() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        set_x(&mut ram, 0, 4);
        set_y(&mut ram, 0, 7);
        ram.write_u16(0x22, 0xfff8);
        ram.write_u16(0x20, 0xfffb);
        write_index(&mut ram, OVERLORD_TYPE, 0, 23);
        pot_trap(&mut ram, 0);
        assert_eq!(read_index(&ram, OVERLORD_TYPE, 0), 0);
        assert_eq!(ram.read_u8(0xb9e), 1);
    }

    #[test]
    fn moving_floor_transition_consumes_random_values_in_c_order() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut hooks = Hooks {
            random: vec![3, 5],
            ..Hooks::default()
        };
        write_index(&mut ram, OVERLORD_GEN2, 1, 31);
        write_index(&mut ram, OVERLORD_X_LO, 1, 1);
        moving_floor(&mut ram, &mut hooks, 1);
        assert_eq!(ram.read_u8(0x41a), 6);
        assert_eq!(read_index(&ram, OVERLORD_GEN2, 1), 133);
        assert_eq!(read_index(&ram, OVERLORD_GEN1, 1), 1);
    }

    #[test]
    fn bomb_snake_preserves_spawn_then_transmute_sequence() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut hooks = Hooks {
            spawn_slots: vec![4],
            ..Hooks::default()
        };
        write_index(&mut ram, OVERLORD_TYPE, 0, 26);
        write_index(&mut ram, OVERLORD_GEN1, 0, 0x20);
        set_x(&mut ram, 0, 0x2345);
        set_y(&mut ram, 0, 0x3456);
        bad_switch_snake(&mut ram, &mut hooks, 0);
        assert_eq!(read_index(&ram, SPRITE_TYPE, 4), 74);
        assert_eq!(read_index(&ram, SPRITE_DELAY_AUX1, 4), 112);
        assert_eq!(hooks.bombs, vec![4]);
        assert_eq!(hooks.sfx2, vec![(4, 0x20)]);
    }

    #[test]
    fn falling_square_executes_first_path_step_and_spawns_garnish() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut hooks = Hooks {
            garnish: Some(2),
            ..Hooks::default()
        };
        write_index(&mut ram, OVERLORD_TYPE, 0, 10);
        set_x(&mut ram, 0, 0x0180);
        set_y(&mut ram, 0, 0x0220);
        falling_square(&mut ram, &mut hooks, 0);
        assert_eq!(get_x(&ram, 0), 0x0180);
        assert_eq!(get_y(&ram, 0), 0x0230);
        assert_eq!(read_index(&ram, OVERLORD_GEN2, 0), 16);
        assert_eq!(ram.garnish_kind(2), 3);
        assert_eq!(ram.garnish_countdown(2), 31);
    }

    #[test]
    fn falling_tile_pan_hook_observes_partial_spawn_and_later_reads_stay_live() {
        let mut bytes = Box::new(std::array::from_fn(|i| (i * 19) as u8));
        let mut ram = Ram::new(&mut bytes);
        set_x(&mut ram, 0, 0x1280);
        set_y(&mut ram, 0, 0x2233);
        let mut expected = Box::new(*ram.bytes_mut());
        expected[0x1f800 + 2] = 9;
        expected[0x1f878 + 2] = 0x12;
        expected[0x1f83c + 2] = 0x55;
        expected[OVERLORD_Y_LO] = 0x44;
        expected[OVERLORD_Y_HI] = 0x33;
        expected[0x12e] = 0x9f;
        expected[0x1f81e + 2] = 0x54;
        expected[0x1f85a + 2] = 0x33;
        expected[0x1f90e + 2] = 31;
        expected[0xfb4] = 31;
        let mut hooks = Hooks {
            garnish: Some(2),
            garnish_pan_mutation: true,
            ..Hooks::default()
        };
        spawn_falling_tile(&mut ram, &mut hooks, 0);
        assert_eq!(hooks.garnish_pan_inputs, [0x80]);
        assert_eq!(ram.bytes_mut(), &mut *expected);
    }
}
