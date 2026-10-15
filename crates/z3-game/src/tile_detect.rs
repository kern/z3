//! Tile collision detection ported from `src/tile_detect.c`.

use crate::ram::Ram;
use crate::tile_map::{DungeonAttributes, OVERWORLD_MAP16, TileByteOffset, TileWordIndex};

/// ROM data and object coordinates owned by modules outside this port.
pub trait TileDetectHooks {
    fn map16_to_map8(&self, index: usize) -> u16;
    fn map8_to_tile_attr(&self, index: usize) -> u8;
    fn ancilla_x(&self, ram: &Ram<'_>, index: usize) -> u16;
    fn ancilla_y(&self, ram: &Ram<'_>, index: usize) -> u16;
}

const R12: usize = 0x0c;
const R14: usize = 0x0e;
const PLAYER_IS_INDOORS: usize = 0x1b;
const LINK_Y: usize = 0x20;
const LINK_X: usize = 0x22;
const TILE_DIAGONAL: usize = 0x38;
const FORCE_MOVE: usize = 0x49;
const TILE_WHICH_Y: usize = 0x51;
const TILE_STAIR: usize = 0x58;
const TILE_PIT: usize = 0x59;
const TILE_VAR2: usize = 0x5f;
const TILE_VAR1: usize = 0x62;
const LINK_LAST_DIRECTION: usize = 0x66;
const TILE_DIAG_STATE: usize = 0x6e;
const SCRATCH_1: usize = 0x74;
const INTERACTING_TILE: usize = 0x76;
const DUNGEON_ROOM: usize = 0xa0;
const BG1_HOFS: usize = 0xe0;
const BG2_HOFS: usize = 0xe2;
const BG1_VOFS: usize = 0xe6;
const BG2_VOFS: usize = 0xe8;
const TILEMAP_MASK: usize = 0xec;
const LOWER_LEVEL: usize = 0xee;
const ROOM_TRANSITION_FLAGS: usize = 0xef;
const LINK_TILE_BELOW: usize = 0x114;
const TILE_INROOM_STAIRS: usize = 0x2c0;
const TILE_CHEST: usize = 0x2e5;
const TILE_KEY_LOCK_GRAVESTONE: usize = 0x2e7;
const TILE_SPIKE_CACTUS: usize = 0x2e8;
const TILE_TYPE: usize = 0x2ea;
const TILE_SPIKE_FLOOR_TRIGGER: usize = 0x2ee;
const TILE_DASHABLE: usize = 0x2ef;
const PLAYER_ON_SOMARIA: usize = 0x2f5;
const TILE_MISC: usize = 0x2f6;
const TILE_MOVING_FLOOR: usize = 0x320;
const TILE_DEEPWATER: usize = 0x341;
const TILE_NORMAL: usize = 0x343;
const TILE_ICY: usize = 0x348;
const TILE_WATER_STAIRS: usize = 0x34c;
const TILE_THICK_GRASS: usize = 0x357;
const TILE_SHALLOW_WATER: usize = 0x359;
const TILE_DESTRUCTION: usize = 0x35b;
const TILE_READ: usize = 0x366;
const LIFTABLE_X2: usize = 0x36a;
const TILE_VERTICAL_LEDGE: usize = 0x36d;
const TILE_HORIZ_LEDGE: usize = 0x36e;
const TILE_DOWN_LEDGE: usize = 0x36f;
const TILE_UNKNOWN: usize = 0x370;
const CHEAT_WALK: usize = 0x37f;
const ANCILLA_ARR1: usize = 0x3a4;
const TILE_VAR4: usize = 0x3f1;
const DUNGEON_STATE_BITS: usize = 0x402;
const INROOM_STAIR_KIND: usize = 0x44a;
const DUNGEON_COLLISION: usize = 0x46c;
const CHEST_LOCATIONS: usize = 0x6e0;
const OW_BASE_Y: usize = 0x708;
const OW_MASK_Y: usize = 0x70a;
const OW_BASE_X: usize = 0x70c;
const OW_MASK_X: usize = 0x70e;
const ANCILLA_DIR: usize = 0xc72;
const BLOCK_LINK_MENU: usize = 0xffc;
#[cfg(test)]
const DUNGEON_ATTR: usize = 0x12000;

fn or8(ram: &mut Ram<'_>, address: usize, bits: u8) {
    ram.write_u8(address, ram.read_u8(address) | bits);
}

fn or16(ram: &mut Ram<'_>, address: usize, bits: u16) {
    ram.write_u16(address, ram.read_u16(address) | bits);
}

#[must_use]
pub fn overworld_tile_attribute(ram: &Ram<'_>, hooks: &impl TileDetectHooks, x: u16, y: u16) -> u8 {
    let mut t = y.wrapping_sub(ram.read_u16(OW_BASE_Y)) & ram.read_u16(OW_MASK_Y);
    t = t.wrapping_mul(8);
    t |= x.wrapping_sub(ram.read_u16(OW_BASE_X)) & ram.read_u16(OW_MASK_X);
    t = OVERWORLD_MAP16
        .read_at_byte(ram, TileByteOffset::new(usize::from(t)).aligned())
        .wrapping_mul(4);
    t |= (y & 8) >> 2;
    t |= x & 1;
    let map8 = hooks.map16_to_map8(usize::from(t));
    let mut result = hooks.map8_to_tile_attr(usize::from(map8 & 0x1ff));
    if (0x10..0x1c).contains(&result) {
        result |= ((map8 >> 14) & 1) as u8;
    }
    result
}

pub fn movement_y(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks, direction: usize) {
    const Y: [u16; 4] = [8, 24, 0, 15];
    const X0: [u16; 4] = [0, 0, 8, 8];
    const X1: [u16; 4] = [8, 8, 16, 16];
    const X2: [u16; 4] = [15, 15, 23, 23];
    assert!(direction < 4);
    reset_state(ram);
    let mask = ram.read_u16(TILEMAP_MASK);
    let x = ram.read_u16(LINK_X);
    let y_pos = ram.read_u16(LINK_Y).wrapping_add(Y[direction]);
    ram.write_u16(TILE_WHICH_Y, y_pos);
    let y = y_pos & mask;
    let x0 = (x.wrapping_add(X0[direction]) & mask) >> 3;
    let x1 = (x.wrapping_add(X1[direction]) & mask) >> 3;
    let x2 = (x.wrapping_add(X2[direction]) & mask) >> 3;
    ram.write_u16(SCRATCH_1, x2);
    execute(ram, hooks, x0, y, 1);
    execute(ram, hooks, x1, y, 2);
    execute(ram, hooks, x2, y, 4);
}

pub fn movement_x(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks, direction: usize) {
    const X: [u16; 4] = [8, 24, 0, 15];
    const Y0: [u16; 4] = [0, 0, 8, 8];
    const Y1: [u16; 4] = [8, 8, 16, 16];
    const Y2: [u16; 4] = [15, 15, 23, 23];
    assert!(direction < 4);
    reset_state(ram);
    let mask = ram.read_u16(TILEMAP_MASK);
    let x = (ram.read_u16(LINK_X).wrapping_add(X[direction]) & mask) >> 3;
    let y = ram.read_u16(LINK_Y);
    let y0 = y.wrapping_add(Y0[direction]) & mask;
    let y1 = y.wrapping_add(Y1[direction]);
    let y2 = y.wrapping_add(Y2[direction]);
    ram.write_u16(TILE_WHICH_Y, y1);
    ram.write_u16(TILE_WHICH_Y + 2, y2);
    execute(ram, hooks, x, y0, 1);
    execute(ram, hooks, x, y1 & mask, 2);
    execute(ram, hooks, x, y2 & mask, 4);
}

pub fn movement_vertical_slopes(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks, direction: usize) {
    const POS: [i8; 4] = [7, 24, -1, 16];
    const X0: [u16; 4] = [0, 0, 8, 8];
    const X1: [u16; 4] = [15, 15, 23, 23];
    assert!(direction < 4);
    reset_state(ram);
    let mask = ram.read_u16(TILEMAP_MASK);
    let y = ram
        .read_u16(LINK_Y)
        .wrapping_add_signed(i16::from(POS[direction]))
        & mask;
    let x = ram.read_u16(LINK_X);
    execute(
        ram,
        hooks,
        (x.wrapping_add(X0[direction]) & mask) >> 3,
        y,
        1,
    );
    execute(
        ram,
        hooks,
        (x.wrapping_add(X1[direction]) & mask) >> 3,
        y,
        2,
    );
}

pub fn movement_horizontal_slopes(
    ram: &mut Ram<'_>,
    hooks: &impl TileDetectHooks,
    direction: usize,
) {
    const POS: [i8; 4] = [7, 24, -1, 16];
    const Y0: [u16; 4] = [0, 0, 8, 8];
    const Y1: [u16; 4] = [15, 15, 23, 23];
    assert!(direction < 4);
    reset_state(ram);
    let mask = ram.read_u16(TILEMAP_MASK);
    let x = (ram
        .read_u16(LINK_X)
        .wrapping_add_signed(i16::from(POS[direction]))
        & mask)
        >> 3;
    let y = ram.read_u16(LINK_Y);
    execute(ram, hooks, x, y.wrapping_add(Y0[direction]) & mask, 1);
    execute(ram, hooks, x, y.wrapping_add(Y1[direction]) & mask, 2);
}

pub fn detect_nearby(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks) {
    detect_box(ram, hooks, 0, 15, 8, 23);
}
pub fn check_mirror_bonk(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks) {
    detect_box(ram, hooks, 2, 13, 10, 21);
}
fn detect_box(
    ram: &mut Ram<'_>,
    hooks: &impl TileDetectHooks,
    xo0: u16,
    xo1: u16,
    yo0: u16,
    yo1: u16,
) {
    reset_state(ram);
    let mask = ram.read_u16(TILEMAP_MASK);
    let x = ram.read_u16(LINK_X);
    let y = ram.read_u16(LINK_Y);
    let x0 = (x.wrapping_add(xo0) & mask) >> 3;
    let x1 = (x.wrapping_add(xo1) & mask) >> 3;
    let y0 = y.wrapping_add(yo0) & mask;
    let y1 = y.wrapping_add(yo1) & mask;
    ram.write_u16(SCRATCH_1, y0);
    execute(ram, hooks, x0, y0, 8);
    execute(ram, hooks, x0, y1, 2);
    execute(ram, hooks, x1, y0, 4);
    execute(ram, hooks, x1, y1, 1);
}

pub fn hookshot_collision(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks, index: usize) {
    let room = ram.read_u8(DUNGEON_ROOM);
    let lower = ram.read_u8(LOWER_LEVEL);
    if ram.read_u8(ANCILLA_ARR1 + index) != 0 {
        if ram.read_u8(INROOM_STAIR_KIND) == 0 {
            ram.write_u8(DUNGEON_ROOM, room.wrapping_add(0x10));
        }
        ram.write_u8(LOWER_LEVEL, lower ^ 1);
    }
    let x = hooks.ancilla_x(ram, index);
    let y = hooks.ancilla_y(ram, index);
    let direction = usize::from(ram.read_u8(ANCILLA_DIR + index));
    reset_state(ram);
    if ram.read_u8(DUNGEON_COLLISION) == 2 {
        ram.write_u8(LOWER_LEVEL, 1);
        let x2 = x
            .wrapping_add(ram.read_u16(BG1_HOFS))
            .wrapping_sub(ram.read_u16(BG2_HOFS));
        let y2 = y
            .wrapping_add(ram.read_u16(BG1_VOFS))
            .wrapping_sub(ram.read_u16(BG2_VOFS));
        hookshot_single_layer(ram, hooks, x2, y2, direction);
        ram.write_u8(LOWER_LEVEL, 0);
    }
    hookshot_single_layer(ram, hooks, x, y, direction);
    ram.write_u8(LOWER_LEVEL, lower);
    ram.write_u8(DUNGEON_ROOM, room);
}

pub fn hookshot_single_layer(
    ram: &mut Ram<'_>,
    hooks: &impl TileDetectHooks,
    x: u16,
    y: u16,
    direction: usize,
) {
    const X: [u16; 8] = [0, 15, 0, 15, 0, 0, 8, 8];
    const Y: [u16; 8] = [0, 0, 7, 7, 0, 15, 0, 15];
    assert!(direction < 4);
    let mask = ram.read_u16(TILEMAP_MASK);
    let i = direction * 2;
    execute(
        ram,
        hooks,
        (x.wrapping_add(X[i]) & mask) >> 3,
        y.wrapping_add(Y[i]) & mask,
        1,
    );
    execute(
        ram,
        hooks,
        (x.wrapping_add(X[i + 1]) & mask) >> 3,
        y.wrapping_add(Y[i + 1]) & mask,
        2,
    );
}

pub fn nudge_in_door(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks, speed: i8) {
    const YO: [i8; 4] = [8, 23, 16, 16];
    const XO: [i8; 4] = [8, 8, 0, 15];
    let vertical = ram.read_u8(LINK_LAST_DIRECTION) & 2 != 0;
    let which = if vertical {
        if (ram.read_u16(LINK_Y) as u8) < 0x80 {
            1
        } else {
            0
        }
    } else if (ram.read_u16(LINK_X) as u8) < 0x80 {
        3
    } else {
        2
    };
    reset_state(ram);
    let mask = ram.read_u16(TILEMAP_MASK);
    let x = (ram
        .read_u16(LINK_X)
        .wrapping_add_signed(i16::from(XO[which]))
        & mask)
        >> 3;
    let y = ram
        .read_u16(LINK_Y)
        .wrapping_add_signed(i16::from(YO[which]))
        & mask;
    execute(ram, hooks, x, y, 1);
    let first = (ram.read_u16(R14) | u16::from(ram.read_u8(TILE_HORIZ_LEDGE))) & 3;
    let second = (ram.read_u8(TILE_VERTICAL_LEDGE) | ram.read_u8(TILE_UNKNOWN)) & 0x33;
    if first == 0 && second == 0 {
        return;
    }
    let address = if vertical { LINK_Y } else { LINK_X };
    ram.write_u16(
        address,
        ram.read_u16(address).wrapping_sub(speed as i16 as u16),
    );
}

pub fn sword_swing_deep_in_door(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks, doorway: u8) {
    const X: [i8; 4] = [8, 8, -1, 16];
    const Y: [i8; 4] = [-1, 24, 16, 16];
    reset_state(ram);
    let i = usize::from(doorway.wrapping_sub(1).wrapping_mul(2));
    assert!(i + 1 < X.len());
    let mask = ram.read_u16(TILEMAP_MASK);
    let lx = ram.read_u16(LINK_X);
    let ly = ram.read_u16(LINK_Y);
    let x0 = (lx.wrapping_add_signed(i16::from(X[i])) & mask) >> 3;
    let x1 = (lx.wrapping_add_signed(i16::from(X[i + 1])) & mask) >> 3;
    let y0 = ly.wrapping_add_signed(i16::from(Y[i])) & mask;
    let y1 = ly.wrapping_add_signed(i16::from(Y[i + 1])) & mask;
    execute(ram, hooks, x0, y0, 1);
    execute(ram, hooks, x1, y1, 2);
}

pub fn reset_state(ram: &mut Ram<'_>) {
    for address in [
        R12,
        R14,
        TILE_DIAGONAL,
        TILE_VAR2,
        TILE_VAR1,
        TILE_INROOM_STAIRS,
        TILE_CHEST,
        TILE_MISC,
        TILE_MOVING_FLOOR,
        TILE_DEEPWATER,
        TILE_NORMAL,
        TILE_ICY,
        TILE_WATER_STAIRS,
        TILE_THICK_GRASS,
        TILE_SHALLOW_WATER,
        TILE_DESTRUCTION,
        TILE_READ,
        TILE_VAR4,
    ] {
        ram.write_u16(address, 0);
    }
    for address in [
        TILE_STAIR,
        TILE_PIT,
        TILE_KEY_LOCK_GRAVESTONE,
        TILE_SPIKE_CACTUS,
        TILE_SPIKE_FLOOR_TRIGGER,
        TILE_DASHABLE,
        TILE_VERTICAL_LEDGE,
        TILE_HORIZ_LEDGE,
        TILE_DOWN_LEDGE,
        TILE_UNKNOWN,
    ] {
        ram.write_u8(address, 0);
    }
}

pub fn execute(ram: &mut Ram<'_>, hooks: &impl TileDetectHooks, x: u16, y: u16, bits: u16) {
    let indoors = ram.read_u8(PLAYER_IS_INDOORS) != 0;
    let (mut tile, offset) = if indoors {
        ram.write_u16(FORCE_MOVE, ram.read_u16(FORCE_MOVE) & 0xff);
        let offset = (y & !7).wrapping_mul(8).wrapping_add(x & 63).wrapping_add(
            if ram.read_u8(LOWER_LEVEL) != 0 {
                0x1000
            } else {
                0
            },
        );
        let offset = usize::from(offset);
        let tile = DungeonAttributes::read(ram, TileWordIndex::new(offset));
        ram.write_u8(LINK_TILE_BELOW, tile);
        (tile, offset)
    } else {
        (overworld_tile_attribute(ram, hooks, x, y), 0)
    };
    if ram.read_u8(CHEAT_WALK) != 0 {
        tile = 0;
    }
    execute_inner(ram, tile, offset, bits, indoors);
}

pub fn execute_inner(ram: &mut Ram<'_>, mut tile: u8, offset: usize, bits: u16, indoors: bool) {
    if ram.read_u8(CHEAT_WALK) != 0 {
        tile = 0;
    }
    match tile {
        0x00
        | 0x05..=0x07
        | 0x14..=0x17
        | 0x21
        | 0x23..=0x25
        | 0x38..=0x3c
        | 0x41
        | 0x45
        | 0x47
        | 0x49
        | 0x5e..=0x5f
        | 0x61..=0x62
        | 0x64..=0x66
        | 0xa6..=0xa7
        | 0xbe..=0xbf
        | 0xd0..=0xef => {
            if !indoors {
                or16(ram, TILE_NORMAL, bits);
            }
        }
        0x01..=0x03 | 0x26 | 0x43 => or16(ram, R14, bits),
        0x6c..=0x6f => {
            if indoors {
                or16(ram, R14, bits);
            } else {
                or16(ram, TILE_NORMAL, bits);
            }
        }
        0x04 => {
            if indoors {
                or16(ram, R14, bits);
            } else {
                or16(ram, TILE_THICK_GRASS, bits);
            }
        }
        0x0b => {
            if indoors {
                or16(ram, R14, bits);
            } else {
                ram.write_u16(INTERACTING_TILE, u16::from(tile));
                or16(ram, TILE_DEEPWATER, bits << 4);
            }
        }
        0x08 => or16(ram, TILE_DEEPWATER, bits),
        0x09 => or16(ram, TILE_SHALLOW_WATER, bits),
        0x0a => or16(ram, TILE_NORMAL, bits),
        0x0c => or16(ram, TILE_MOVING_FLOOR, bits),
        0x0d => {
            if ram.read_u8(BLOCK_LINK_MENU) == 0 && ram.read_u16(DUNGEON_STATE_BITS) & 0x8000 == 0 {
                or8(ram, TILE_SPIKE_FLOOR_TRIGGER, (bits << 4) as u8);
            }
        }
        0x0e => or16(ram, TILE_ICY, bits),
        0x0f => or16(ram, TILE_ICY, bits << 4),
        0x10..=0x13 => {
            or16(ram, R12, bits);
            ram.write_u16(TILE_DIAG_STATE, [4, 0, 6, 2][usize::from(tile & 3)]);
        }
        0x18..=0x1b => {
            or16(ram, TILE_DIAGONAL, bits);
            or16(ram, R12, bits);
            ram.write_u16(TILE_DIAG_STATE, [4, 0, 6, 2][usize::from(tile & 3)]);
        }
        0x1c => or16(ram, TILE_WATER_STAIRS, bits),
        0x1d..=0x1f => {
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            or16(ram, TILE_INROOM_STAIRS, bits);
            or8(ram, TILE_STAIR, bits as u8);
        }
        0x20 | 0xb0..=0xbd => {
            if ram.read_u8(PLAYER_ON_SOMARIA) == 0 {
                or8(ram, TILE_PIT, bits as u8);
            }
        }
        0x22 | 0x30..=0x37 => or8(ram, TILE_STAIR, bits as u8),
        0x27 => {
            or16(ram, R14, bits);
            or16(ram, TILE_MISC, bits);
        }
        0x28 => {
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            or8(ram, TILE_VERTICAL_LEDGE, bits as u8);
        }
        0x29 => {
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            or8(ram, TILE_VERTICAL_LEDGE, (bits << 4) as u8);
        }
        0x2a..=0x2b => {
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            or8(ram, TILE_HORIZ_LEDGE, bits as u8);
        }
        0x2c | 0x2e => {
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            or8(ram, TILE_HORIZ_LEDGE, (bits << 4) as u8);
        }
        0x2d | 0x2f => {
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            or8(ram, TILE_DOWN_LEDGE, bits as u8);
        }
        0x3d..=0x3f => {
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            or16(ram, TILE_INROOM_STAIRS, bits << 4);
            or8(ram, TILE_STAIR, bits as u8);
        }
        0x40 => or16(ram, TILE_THICK_GRASS, bits),
        0x44 => {
            if ram.read_u8(BLOCK_LINK_MENU) == 0 && ram.read_u16(DUNGEON_STATE_BITS) & 0x8000 == 0 {
                or8(ram, TILE_SPIKE_CACTUS, bits as u8);
            } else {
                or16(ram, R14, bits);
            }
        }
        0x46 => {
            or8(ram, TILE_SPIKE_FLOOR_TRIGGER, bits as u8);
            or16(ram, R14, bits);
        }
        0x48 | 0x4a => {
            or16(ram, TILE_DESTRUCTION, bits);
            or16(ram, TILE_NORMAL, bits);
        }
        0x4b => or16(ram, TILE_THICK_GRASS, bits << 4),
        0x50..=0x56 => {
            const ORDER: [u8; 7] = [0x54, 0x52, 0x50, 0x51, 0x53, 0x55, 0x56];
            let i = ORDER
                .iter()
                .rposition(|&v| v == tile)
                .expect("liftable tile");
            if matches!(tile, 0x50 | 0x51) {
                or8(ram, TILE_DASHABLE, (bits << 4) as u8);
            }
            or16(ram, TILE_READ, bits);
            ram.write_u8(LIFTABLE_X2, (i * 2) as u8);
            or16(ram, R14, bits);
            or16(ram, TILE_MISC, bits);
        }
        0x57 => {
            or16(ram, R14, bits);
            or8(ram, TILE_DASHABLE, (bits << 4) as u8);
        }
        0x58..=0x5d => {
            or16(ram, TILE_MISC, bits);
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            let loc = ram.read_u16(CHEST_LOCATIONS + usize::from(tile - 0x58) * 2);
            if loc >= 0x8000 {
                or16(ram, R14, bits);
                or8(ram, TILE_KEY_LOCK_GRAVESTONE, (bits << 4) as u8);
                if bits & 2 != 0 {
                    ram.write_u16(TILE_TYPE, u16::from(tile));
                }
            } else {
                or16(ram, TILE_CHEST, bits);
                or16(ram, R14, bits);
            }
        }
        0x60 => {
            if indoors {
                or16(
                    ram,
                    TILE_MISC,
                    if DungeonAttributes::read(ram, TileWordIndex::new(offset + 64)) == 0x60 {
                        bits << 8
                    } else {
                        bits << 12
                    },
                );
            } else {
                or16(ram, TILE_NORMAL, bits);
            }
        }
        0x63 => {
            or16(ram, TILE_MISC, bits);
            ram.write_u16(INTERACTING_TILE, u16::from(tile));
            or16(ram, TILE_CHEST, bits);
            or16(ram, R14, bits);
        }
        0x67 => {
            or16(ram, R14, bits);
            or16(ram, TILE_MISC, bits);
            or8(ram, TILE_SPIKE_CACTUS, (bits << 4) as u8);
        }
        0x68 => or16(ram, TILE_VAR4, bits),
        0x69 => or16(ram, TILE_VAR4, bits << 4),
        0x6a => or16(ram, TILE_VAR4, bits << 8),
        0x6b => or16(ram, TILE_VAR4, bits << 12),
        0x70..=0x7f => {
            if bits & 2 != 0 {
                or16(ram, TILE_VAR2, 1 << (tile & 0xf));
            }
            or16(ram, R14, bits);
            or16(ram, TILE_MISC, bits);
        }
        0x80..=0x81 | 0x84..=0x8d => {
            or16(ram, R14, bits << 4);
            ram.write_u16(TILE_VAR1, u16::from(2 * (tile & 1)));
        }
        0x82..=0x83 => {
            or16(ram, R14, (bits << 4) | (bits << 8));
            ram.write_u16(TILE_VAR1, u16::from(2 * (tile & 1)));
        }
        0x8e..=0x8f => {
            or16(ram, R14, bits << 4);
            or8(ram, TILE_DASHABLE, bits as u8);
            ram.write_u16(TILE_VAR1, 0);
        }
        0x90..=0x97 => door(ram, tile, bits, 1, true),
        0x98..=0x9f | 0xa8..=0xaf => door(ram, tile, bits, 3, true),
        0xa0..=0xa1 | 0xa4..=0xa5 => door(ram, tile, bits, 2, false),
        0xa2..=0xa3 => door(ram, tile, bits, 2, true),
        0xc0..=0xcf => {
            or16(ram, R14, bits);
            or16(ram, TILE_MISC, bits);
        }
        0xf0..=0xff => {
            or16(ram, R14, bits);
            or16(ram, TILE_MISC, bits << 4);
        }
        0x42 => {
            if !indoors {
                or8(ram, TILE_KEY_LOCK_GRAVESTONE, bits as u8);
                or16(ram, R14, bits);
            }
        }
        0x4c..=0x4d => {
            if !indoors {
                ram.write_u16(INTERACTING_TILE, u16::from(tile));
                or8(ram, TILE_UNKNOWN, bits as u8);
            }
        }
        0x4e..=0x4f => {
            if !indoors {
                ram.write_u16(INTERACTING_TILE, u16::from(tile));
                or8(ram, TILE_UNKNOWN, (bits << 4) as u8);
            }
        }
    }
}

fn door(ram: &mut Ram<'_>, tile: u8, bits: u16, flags: u8, shutter: bool) {
    ram.write_u8(ROOM_TRANSITION_FLAGS, flags);
    or16(ram, R14, (bits << 4) | if shutter { bits << 8 } else { 0 });
    ram.write_u16(TILE_VAR1, u16::from(2 * (tile & 1)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;
    struct Hooks {
        map8: u16,
        attr: u8,
    }
    impl TileDetectHooks for Hooks {
        fn map16_to_map8(&self, _: usize) -> u16 {
            self.map8
        }
        fn map8_to_tile_attr(&self, _: usize) -> u8 {
            self.attr
        }
        fn ancilla_x(&self, _: &Ram<'_>, _: usize) -> u16 {
            0
        }
        fn ancilla_y(&self, _: &Ram<'_>, _: usize) -> u16 {
            0
        }
    }
    #[test]
    fn representative_tile_outputs_match_c() {
        let cases = [
            (0x01, false, R14, 2),
            (0x08, true, TILE_DEEPWATER, 2),
            (0x0f, true, TILE_ICY, 0x20),
            (0x18, true, TILE_DIAGONAL, 2),
            (0x29, true, TILE_VERTICAL_LEDGE, 0x20),
            (0x4b, false, TILE_THICK_GRASS, 0x20),
            (0x69, true, TILE_VAR4, 0x20),
            (0x6a, true, TILE_VAR4, 0x200),
            (0x6b, true, TILE_VAR4, 0x2000),
            (0xf0, true, TILE_MISC, 0x20),
        ];
        for (tile, indoors, address, expected) in cases {
            let mut bytes = [0; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            execute_inner(&mut ram, tile, 0, 2, indoors);
            assert_eq!(ram.read_u16(address), expected, "tile {tile:#04x}");
        }
    }
    #[test]
    fn slope_and_door_quirks_are_preserved() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        execute_inner(&mut ram, 0x11, 0, 4, true);
        assert_eq!(ram.read_u16(R12), 4);
        assert_eq!(ram.read_u16(TILE_DIAG_STATE), 0);
        execute_inner(&mut ram, 0x99, 0, 2, true);
        assert_eq!(ram.read_u8(ROOM_TRANSITION_FLAGS), 3);
        assert_eq!(ram.read_u16(R14), 0x220);
        assert_eq!(ram.read_u16(TILE_VAR1), 2);
    }
    #[test]
    fn overworld_slope_attribute_inherits_map8_flip_bit() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OW_MASK_X, 0);
        ram.write_u16(OW_MASK_Y, 0);
        assert_eq!(
            overworld_tile_attribute(
                &ram,
                &Hooks {
                    map8: 0x4000,
                    attr: 0x10
                },
                0,
                0
            ),
            0x11
        );
    }

    #[test]
    fn indoor_layer_selection_and_signed_door_nudge_match_c() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let hooks = Hooks { map8: 0, attr: 1 };
        ram.write_u8(PLAYER_IS_INDOORS, 1);
        ram.write_u8(DUNGEON_ATTR, 0x08);
        ram.write_u8(DUNGEON_ATTR + 0x1000, 0x09);
        ram.write_u8(LOWER_LEVEL, 1);
        execute(&mut ram, &hooks, 0, 0, 1);
        assert_eq!(ram.read_u8(LINK_TILE_BELOW), 0x09);
        assert_eq!(ram.read_u16(TILE_SHALLOW_WATER), 1);

        ram.write_u8(PLAYER_IS_INDOORS, 0);
        ram.write_u16(TILEMAP_MASK, 0xffff);
        ram.write_u16(LINK_X, 0x0080);
        ram.write_u16(LINK_Y, 0x0080);
        nudge_in_door(&mut ram, &hooks, -1);
        assert_eq!(ram.read_u16(LINK_X), 0x0081);
    }
}
