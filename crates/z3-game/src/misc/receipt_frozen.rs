//! Frozen 412aa43 receipt functions and constant tables; no target-domain API.
use super::*;

pub const ITEM_TARGETS: [usize; 76] = [
    0xf359, 0xf359, 0xf359, 0xf359, 0xf35a, 0xf35a, 0xf35a, 0xf345, 0xf346, 0xf34b, 0xf342, 0xf340,
    0xf341, 0xf344, 0xf35c, 0xf347, 0xf348, 0xf349, 0xf34a, 0xf34c, 0xf34c, 0xf350, 0xf35c, 0xf36b,
    0xf351, 0xf352, 0xf353, 0xf354, 0xf354, 0xf34e, 0xf356, 0xf357, 0xf37a, 0xf34d, 0xf35b, 0xf35b,
    0xf36f, 0xf364, 0xf36c, 0xf375, 0xf375, 0xf344, 0xf341, 0xf35c, 0xf35c, 0xf35c, 0xf36d, 0xf36e,
    0xf36e, 0xf375, 0xf366, 0xf368, 0xf360, 0xf360, 0xf360, 0xf374, 0xf374, 0xf374, 0xf340, 0xf340,
    0xf35c, 0xf35c, 0xf36c, 0xf36c, 0xf360, 0xf360, 0xf372, 0xf376, 0xf376, 0xf373, 0xf360, 0xf360,
    0xf35c, 0xf359, 0xf34c, 0xf355,
];

const ITEM_VALUES: [i8; 76] = [
    1, 2, 3, 4, 1, 2, 3, 1, 1, 1, 1, 1, 1, 2, -1, 1, 1, 1, 1, 1, 2, 1, -1, -1, 1, 1, 2, 1, 2, 1, 1,
    1, -1, 1, -1, 2, -1, -1, -1, -1, -1, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -5, -20,
    -1, -1, -1, 1, 3, -1, -1, -1, -1, -100, -50, -1, 1, 10, -1, -1, -1, -1, 1, 3, 1,
];

const BOTTLE_LIST: [u8; 7] = [0x16, 0x2b, 0x2c, 0x2d, 0x3d, 0x3c, 0x48];

const POTION_LIST: [u8; 5] = [0x2e, 0x2f, 0x30, 0xff, 0xe];

pub const RECEIVE_ITEM_TAB1: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];

const RECEIVE_ITEM_TAB2: [i8; 76] = [
    -5, -5, -5, -5, -5, -4, -4, -5, -5, -4, -4, -4, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4,
    -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -5, -4, -4, -4, -4, -4, -4, -2, -4, -4, -4, -4, -4,
    -4, -4, -4, -4, -2, -2, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -2, -2, -4, -2, -4, -4,
    -4, -5, -4, -4,
];

const RECEIVE_ITEM_TAB3: [u8; 76] = [
    4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
];

pub const RECEIVE_ITEM_GFX: [u8; 76] = [
    6, 0x18, 0x18, 0x18, 0x2d, 0x20, 0x2e, 9, 9, 0xa, 8, 5, 0x10, 0xb, 0x2c, 0x1b, 0x1a, 0x1c,
    0x14, 0x19, 0xc, 7, 0x1d, 0x2f, 7, 0x15, 0x12, 0xd, 0xd, 0xe, 0x11, 0x17, 0x28, 0x27, 4, 4,
    0xf, 0x16, 3, 0x13, 1, 0x1e, 0x10, 0, 0, 0, 0, 0, 0, 0x30, 0x22, 0x21, 0x24, 0x24, 0x24, 0x23,
    0x23, 0x23, 0x29, 0x2a, 0x2c, 0x2b, 3, 3, 0x34, 0x35, 0x31, 0x33, 2, 0x32, 0x36, 0x37, 0x2c, 6,
    0xc, 0x38,
];

pub fn item_receipt_give_bottled_item(r: &mut Ram<'_>, item: u8) {
    if let Some(j) = BOTTLE_LIST.iter().position(|&v| v == item) {
        for i in 0..4 {
            if r.read_u8(BOTTLES + i) < 2 {
                r.write_u8(BOTTLES + i, j as u8 + 2);
                return;
            }
        }
    }
    if let Some(j) = POTION_LIST.iter().position(|&v| v == item) {
        for i in 0..4 {
            if r.read_u8(BOTTLES + i) == 2 {
                r.write_u8(BOTTLES + i, j as u8 + 3);
                return;
            }
        }
    }
}

pub fn ancilla_add_item_receipt<H: MiscHooks>(
    r: &mut Ram<'_>,
    hooks: &mut H,
    ain: u8,
    yin: u8,
    chest_pos: i32,
) {
    let ancilla = hooks.add_ancilla(r, ain, yin);
    if ancilla < 0 {
        return;
    }
    let ancilla = ancilla as usize;
    let j = usize::from(r.read_u8(RECEIVE_ITEM));
    r.write_u8(IMMOBILIZED, if j == 0x20 { 2 } else { 1 });
    if j == 0 {
        r.write_u8(ITEM_TARGETS[4], ITEM_VALUES[0] as u8);
    }
    let p = ITEM_TARGETS[j];
    let value = ITEM_VALUES[j];
    if value >= 0 {
        r.write_u8(p, value as u8);
    }

    if j == 0x1f {
        r.write_u8(IS_BUNNY, 0);
    } else if matches!(j, 0x4b | 0x1e) {
        r.write_u8(
            ABILITY_FLAGS,
            r.read_u8(ABILITY_FLAGS) | if j == 0x4b { 4 } else { 2 },
        );
    }
    if matches!(j, 0x1b | 0x1c) {
        hooks.call(r, MiscCall::PaletteUpdateGloves);
    } else if let Some(t) = match j {
        0x37 => Some(4),
        0x38 => Some(1),
        0x39 => Some(2),
        _ => None,
    } {
        let v = r.read_u8(p) | t;
        r.write_u8(p, v);
        if v & 7 == 7 {
            r.write_u8(MAP_ICONS, 4);
        }
        inc8(r, MAP_STATE);
    } else if j == 0x22 {
        if r.read_u8(p) == 0 {
            r.write_u8(p, 1);
        }
    } else if matches!(j, 0x25 | 0x32 | 0x33) {
        let bit = 0x8000u16 >> (r.read_u8(PALACE_X2) >> 1);
        r.write_u16(p, r.read_u16(p) | bit);
    } else if j == 0x3e {
        if r.read_u8(LINK_STATE_BITS) & 0x80 != 0 {
            r.write_u8(PICK_THROW, 2);
        }
    } else if j == 0x20 {
        inc8(r, MAP_STATE);
        for i in (0..5).rev() {
            if matches!(r.read_u8(ANC_TYPE + i), 7 | 0x2c) {
                r.write_u8(ANC_TYPE + i, 0);
                r.write_u8(LINK_STATE_BITS, 0);
                r.write_u8(PICK_THROW, 0);
            }
        }
        if r.read_u8(CAPE_MODE) != 0 {
            r.write_u8(BUNNY_TIMER, 32);
            r.write_u8(DISABLE_DAMAGE, 0);
            r.write_u8(CAPE_MODE, 0);
            hooks.call(r, MiscCall::CapePoof(0x23, 4));
            r.write_u8(SFX1, 0x15 | link_calculate_sfx_pan(r));
        }
    } else if j == 0x29 {
        if r.read_u8(MUSHROOM) != 2 {
            r.write_u8(p, 1);
            hooks.call(r, MiscCall::RefreshIcon);
        }
    } else {
        let increment = if j == 0x24 {
            Some(1)
        } else if r.read_u8(RECEIPT_METHOD) != 2 {
            match j {
                0x27 => Some(1),
                0x28 => Some(3),
                0x31 => Some(10),
                _ => None,
            }
        } else {
            None
        };
        if let Some(t) = increment {
            let v = r.read_u8(p).wrapping_add(t).min(99);
            r.write_u8(p, v);
            hooks.call(r, MiscCall::RefreshIcon);
        } else if j == 0x17 {
            r.write_u8(p, r.read_u8(p).wrapping_add(1) & 3);
            r.write_u8(SFX2, 0x2d | link_calculate_sfx_pan(r));
        } else if j == 1 {
            hooks.call(r, MiscCall::OverworldSetSongList);
        } else {
            item_receipt_give_bottled_item(r, j as u8);
        }
    }

    let mut gfx = RECEIVE_ITEM_GFX[j];
    if gfx == 0xff {
        gfx = 0;
    } else if matches!(gfx, 0x20 | 0x2d | 0x2e) {
        hooks.call(r, MiscCall::DecompressShield);
        hooks.call(r, MiscCall::PaletteLoadShield);
    }
    hooks.call(r, MiscCall::DecodeAnimatedTile(gfx));
    if matches!(gfx, 6 | 0x18) && j != 0 {
        hooks.call(r, MiscCall::DecompressSword);
        hooks.call(r, MiscCall::PaletteLoadSword);
    }
    r.write_u8(ANC_ITEM + ancilla, j as u8);
    r.write_u8(ANC_ARR1 + ancilla, 0);
    if j == 1 && r.read_u8(RECEIPT_METHOD) != 2 {
        r.write_u8(ANC_TIMER + ancilla, 160);
        r.write_u8(SUB, 43);
        r.write_u8(PALETTE_COUNTDOWN, 0);
        hooks.call(r, MiscCall::MsCutscene(0x35, 4));
        r.write_u8(ANC_ARR3 + ancilla, 2);
    } else {
        r.write_u8(ANC_ARR3 + ancilla, 9);
    }
    r.write_u8(ANC_ARR4 + ancilla, 5);
    r.write_u8(ANC_STEP + ancilla, r.read_u8(RECEIPT_METHOD));
    let aux = if matches!(j, 0x20 | 0x37 | 0x38 | 0x39) {
        0x68
    } else if j == 0x26 {
        2
    } else if r.read_u8(RECEIPT_METHOD) != 0 {
        0x38
    } else {
        0x60
    };
    r.write_u8(ANC_AUX_TIMER + ancilla, aux);

    let method = r.read_u8(RECEIPT_METHOD);
    let (x, y) = if method == 1 {
        let y = ((chest_pos & 0x1f80) >> 4)
            + i32::from(r.read_u16(DUNG_BG_V) & !0xff)
            + i32::from(RECEIVE_ITEM_TAB2[j]);
        let x = ((chest_pos & 0x7e) << 2)
            + i32::from(r.read_u16(DUNG_BG_H) & !0xff)
            + i32::from(RECEIVE_ITEM_TAB3[j]);
        (x, y)
    } else {
        if r.read_u8(ANC_STEP + ancilla) == 0 && j == 1 {
            r.write_u8(SFX1, link_calculate_sfx_pan(r) | 0x2c);
        } else if matches!(j, 0x20 | 0x37 | 0x38 | 0x39) {
            r.write_u8(MUSIC, link_calculate_sfx_pan(r) | 0x13);
        } else if !matches!(j, 0x3e | 0x17) {
            r.write_u8(SFX2, link_calculate_sfx_pan(r) | 0xf);
        }
        let effective_method = if method == 3 { 0 } else { method };
        let xoff = if effective_method != 0 {
            RECEIVE_ITEM_TAB3[j]
        } else if RECEIVE_ITEM_TAB1[j] == 0 {
            10
        } else if j == 0x20 {
            0
        } else {
            6
        };
        let x = i32::from(r.read_u16(LINK_X)) + i32::from(xoff);
        let yoff = if effective_method != 0 {
            i32::from(RECEIVE_ITEM_TAB2[j])
        } else {
            -14
        };
        let y = i32::from(r.read_u16(LINK_Y)) + yoff + if effective_method == 2 { -8 } else { 0 };
        (x, y)
    };
    hooks.set_ancilla_xy(r, ancilla, x as u16, y as u16);
}
