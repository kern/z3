//! HUD counters, inventory selection, and pause-menu state machine.
//!
//! State addresses deliberately remain identical to `variables.h`. Pause-menu
//! and in-game HUD tilemaps are produced here; [`HudRuntime`] only bridges
//! emulator VRAM glyph uploads and the external save/select menu.

use crate::ram::Ram;

pub const HUD_ITEM_COUNT: usize = 20;
pub const MAX_BOMBS_FOR_LEVEL: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
pub const MAX_ARROWS_FOR_LEVEL: [u8; 8] = [30, 35, 40, 45, 50, 55, 60, 70];
const MAX_HEALTH_FOR_LEVEL: [u8; 21] = [
    9, 9, 9, 9, 9, 9, 9, 9, 17, 17, 17, 17, 17, 17, 17, 25, 25, 25, 25, 25, 25,
];

const MAIN_MODULE: usize = 0x10;
const SUBMODULE: usize = 0x11;
const FLAG_UPDATE_HUD: usize = 0x16;
const NMI_SUBROUTINE: usize = 0x17;
const TS_COPY: usize = 0x1d;
const FRAME_COUNTER: usize = 0x1a;
const DUNGEON_ROOM: usize = 0xa0;
const DUNGEON_FLOOR: usize = 0xa4;
const HUD_TMP1: usize = 0xbd;
const BG3VOFS_COPY2: usize = 0xea;
const JOYPAD_H_LAST: usize = 0xf0;
const JOYPAD_L_LAST: usize = 0xf2;
const FILTERED_JOYPAD_H: usize = 0xf4;
const FILTERED_JOYPAD_L: usize = 0xf6;
const SAVED_MODULE_FOR_MENU: usize = 0x10c;
const NMI_LOAD_TARGET: usize = 0x116;
const SOUND_AMBIENT: usize = 0x12d;
const SOUND_1: usize = 0x12e;
const SOUND_2: usize = 0x12f;
const MAP_STATE: usize = 0x200;
const HUD_CUR_ITEM: usize = 0x202;
const HUD_VAR1: usize = 0x204;
const BOTTLE_EXPAND_ROW: usize = 0x205;
const HUD_FRAME: usize = 0x206;
const FLASH_TIMER: usize = 0x207;
const HEART_ANIM_TIMER: usize = 0x208;
const HEART_ANIM_SUBPOS: usize = 0x209;
const HEART_ANIMATING: usize = 0x20a;
const LINK_DEBUG_VALUE_1: usize = 0x20b;
const INVENTORY_ORDER: usize = 0x225;
const LINK_IMMOBILIZED: usize = 0x2e4;
const CURRENT_ITEM_Y: usize = 0x303;
const EQ_DEBUG_VARIABLE: usize = 0x34b;
const CUR_PALACE_X2: usize = 0x40c;
const DUNGEON_BG2_PROPERTIES: usize = 0x414;
const DARK_WITH_LANTERN: usize = 0x458;
const NUM_LIT_TORCHES: usize = 0x45a;
const FLOOR_TIMER: usize = 0x4a0;
const SUPER_BOMB_VALUE: usize = 0x4b4;
const SUPER_BOMB_TIMER: usize = 0x4b5;
const LOW_HEALTH_TIMER: usize = 0x4ca;
const RUPEE_SOUND_DELAY: usize = 0xcfd;
const FEATURES0: usize = 0x64c;
const HUD_CUR_ITEM_X: usize = 0x656;
const HUD_CUR_ITEM_L: usize = 0x657;
const HUD_CUR_ITEM_R: usize = 0x658;
const HUD_TILEMAP: usize = 0xc700;
const DUNGEON_WANTS_LIGHTS_OUT: usize = 0xc005;
const FIXED_COLOR: usize = 0xc017;
const LINK_ITEMS: usize = 0xf340;
const LINK_BOW: usize = 0xf340;
const LINK_BOMBS: usize = 0xf343;
const LINK_TORCH: usize = 0xf34a;
const LINK_BOTTLE_INDEX: usize = 0xf34f;
const LINK_GLOVES: usize = 0xf354;
const LINK_BOOTS: usize = 0xf355;
const LINK_FLIPPERS: usize = 0xf356;
const LINK_MOON_PEARL: usize = 0xf357;
const LINK_SWORD: usize = 0xf359;
const LINK_SHIELD: usize = 0xf35a;
const LINK_ARMOR: usize = 0xf35b;
const BOTTLES: usize = 0xf35c;
const RUPEES_GOAL: usize = 0xf360;
const RUPEES_ACTUAL: usize = 0xf362;
const HEALTH_CAPACITY: usize = 0xf36c;
const HEALTH_CURRENT: usize = 0xf36d;
const MAGIC_POWER: usize = 0xf36e;
const NUM_KEYS: usize = 0xf36f;
const LINK_COMPASS: usize = 0xf364;
const LINK_BIG_KEY: usize = 0xf366;
const LINK_DUNGEON_MAP: usize = 0xf368;
const HEART_PIECES: usize = 0xf36b;
const BOMB_UPGRADES: usize = 0xf370;
const ARROW_UPGRADES: usize = 0xf371;
const HEARTS_FILLER: usize = 0xf372;
const PENDANTS: usize = 0xf374;
const MAGIC_FILLER: usize = 0xf373;
const BOMB_FILLER: usize = 0xf375;
const ARROW_FILLER: usize = 0xf376;
const NUM_ARROWS: usize = 0xf377;
const ABILITY_FLAGS: usize = 0xf379;
const CRYSTALS: usize = 0xf37a;
const MAGIC_CONSUMPTION: usize = 0xf37b;
const SRAM_PROGRESS: usize = 0xf3c5;

const FEATURE_SWITCH_LR: u32 = 2;
const FEATURE_DISABLE_LOW_HEALTH_BEEP: u32 = 64;
const FEATURE_SHOW_MAX_YELLOW: u32 = 256;
const FEATURE_CARRY_MORE_RUPEES: u32 = 2048;
const FEATURE_SWITCH_LR_LIMIT: u32 = 32768;

const JOY_UP: u8 = 8;
const JOY_DOWN: u8 = 4;
const JOY_LEFT: u8 = 2;
const JOY_RIGHT: u8 = 1;
const JOY_START: u8 = 0x10;
const JOY_SELECT: u8 = 0x20;
const JOY_Y: u8 = 0x40;
const JOY_X: u8 = 0x40;
const JOY_L: u8 = 0x20;
const JOY_R: u8 = 0x10;

const fn hud_xy(x: usize, y: usize) -> usize {
    x + y * 32
}

const ITEM_POSITIONS: [usize; 20] = [
    hud_xy(4, 7),
    hud_xy(7, 7),
    hud_xy(10, 7),
    hud_xy(13, 7),
    hud_xy(16, 7),
    hud_xy(4, 10),
    hud_xy(7, 10),
    hud_xy(10, 10),
    hud_xy(13, 10),
    hud_xy(16, 10),
    hud_xy(4, 13),
    hud_xy(7, 13),
    hud_xy(10, 13),
    hud_xy(13, 13),
    hud_xy(16, 13),
    hud_xy(4, 16),
    hud_xy(7, 16),
    hud_xy(10, 16),
    hud_xy(13, 16),
    hud_xy(16, 16),
];

const EMPTY_ICON: [u16; 4] = [0x20f5; 4];
const BOTTLE_ICONS: [[u16; 4]; 9] = [
    EMPTY_ICON,
    [0x2044, 0x2045, 0x2046, 0x2047],
    [0x2837, 0x2838, 0x2cc3, 0x2cd3],
    [0x24d2, 0x64d2, 0x24e2, 0x24e3],
    [0x3cd2, 0x7cd2, 0x3ce2, 0x3ce3],
    [0x2cd2, 0x6cd2, 0x2ce2, 0x2ce3],
    [0x2855, 0x6855, 0x2c57, 0x2c5a],
    [0x2837, 0x2838, 0x2839, 0x283a],
    [0x2837, 0x2838, 0x2839, 0x283a],
];

const fn two(empty: [u16; 4], full: [u16; 4], value: u8) -> [u16; 4] {
    if value == 0 { empty } else { full }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HudRenderOp {
    /// Upload the replacement L/R letter glyphs to emulator VRAM.
    SwitchLetterTiles(u8),
}

/// Required integration with the pause-menu tilemap/VRAM and messaging code.
pub trait HudRuntime {
    fn render(&mut self, operation: HudRenderOp, ram: &mut Ram<'_>);
    fn display_select_menu(&mut self, ram: &mut Ram<'_>);
}

pub struct Hud<'a, 'r, R: HudRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
}

impl<R: HudRuntime> Hud<'_, '_, R> {
    fn feature(&self, flag: u32) -> bool {
        let b = self.ram.slice(FEATURES0, 4);
        u32::from_le_bytes([b[0], b[1], b[2], b[3]]) & flag != 0
    }

    fn increment(&mut self, address: usize) {
        self.ram
            .write_u8(address, self.ram.read_u8(address).wrapping_add(1));
    }

    fn tile(&self, index: usize) -> u16 {
        self.ram.read_u16(HUD_TILEMAP + index * 2)
    }

    fn menu_set(&mut self, index: usize, value: u16) {
        self.ram.write_u16(0x1000 + index * 2, value);
    }

    fn menu_get(&self, index: usize) -> u16 {
        self.ram.read_u16(0x1000 + index * 2)
    }

    fn draw_item_at(&mut self, index: usize, icon: [u16; 4]) {
        self.menu_set(index, icon[0]);
        self.menu_set(index + 1, icon[1]);
        self.menu_set(index + 32, icon[2]);
        self.menu_set(index + 33, icon[3]);
    }

    fn draw_nxn(&mut self, index: usize, src: &[u16], width: usize, height: usize) {
        for y in 0..height {
            for x in 0..width {
                self.menu_set(index + hud_xy(x, y), src[y * width + x]);
            }
        }
    }

    fn copy_2x2(&mut self, dst: usize, src: usize) {
        let tiles = [
            self.menu_get(src),
            self.menu_get(src + 1),
            self.menu_get(src + 32),
            self.menu_get(src + 33),
        ];
        self.draw_item_at(dst, tiles);
    }

    fn draw_box(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, palette: u16) {
        let corner = 0x20fb | palette << 10;
        self.menu_set(hud_xy(x1, y1), corner);
        self.menu_set(hud_xy(x2, y1), corner + 0x4000);
        self.menu_set(hud_xy(x1, y2), corner + 0x8000);
        self.menu_set(hud_xy(x2, y2), corner + 0xc000);
        let side = 0x20fc | palette << 10;
        for y in y1 + 1..y2 {
            self.menu_set(hud_xy(x1, y), side);
            self.menu_set(hud_xy(x2, y), side + 0x4000);
        }
        let edge = 0x20f9 | palette << 10;
        for x in x1 + 1..x2 {
            self.menu_set(hud_xy(x, y1), edge);
            self.menu_set(hud_xy(x, y2), edge + 0x8000);
        }
        for y in y1 + 1..y2 {
            for x in x1 + 1..x2 {
                self.menu_set(hud_xy(x, y), 0x24f5);
            }
        }
    }

    fn draw_flashing_circle(&mut self, p: usize, palette: u16) {
        let pp = palette << 10;
        for (offset, tile) in [
            (-32, 0x2061),
            (-31, 0x6061),
            (-1, 0x2070),
            (2, 0x6070),
            (31, 0xa070),
            (34, 0xe070),
            (64, 0xa061),
            (65, 0xe061),
            (-33, 0x2060),
            (-30, 0x6060),
            (66, 0xe060),
            (63, 0xa060),
        ] {
            self.menu_set((p as isize + offset) as usize, pp | tile);
        }
    }

    fn icon_for(&self, item: u8) -> [u16; 4] {
        if item == 0 {
            return EMPTY_ICON;
        }
        let mut value = self.item(item);
        if item == 4 {
            value = u8::from(value != 0);
        } else if item == 16 {
            value = if value == 0 {
                0
            } else {
                self.ram.read_u8(BOTTLES + usize::from(value - 1))
            };
        }
        Self::item_box_icon(item - 1, value)
    }

    /// Value-selected equivalent of C `Hud_GetItemBoxPtr(item)[value]`.
    /// `item_index` is the C table's zero-based item index.
    pub fn item_box_icon(item_index: u8, value: u8) -> [u16; 4] {
        match item_index {
            0 => [
                [0x20f5; 4],
                [0x28ba, 0x28e9, 0x28e8, 0x28cb],
                [0x28ba, 0x284a, 0x2849, 0x28cb],
                [0x28ba, 0x28e9, 0x28e8, 0x28cb],
                [0x28ba, 0x28bb, 0x24ca, 0x28cb],
            ][usize::from(value.min(4))],
            1 => [
                [0x20f5; 4],
                [0x2cb8, 0x2cb9, 0x2cf5, 0x2cc9],
                [0x24b8, 0x24b9, 0x24f5, 0x24c9],
            ][usize::from(value.min(2))],
            2 => two(EMPTY_ICON, [0x24f5, 0x24f6, 0x24c0, 0x24f5], value),
            3 => two(EMPTY_ICON, [0x2cb2, 0x2cb3, 0x2cc2, 0x6cc2], value),
            4 => [
                [0x20f5; 4],
                [0x2444, 0x2445, 0x2446, 0x2447],
                [0x203b, 0x203c, 0x203d, 0x203e],
            ][usize::from(value.min(2))],
            5 => two(EMPTY_ICON, [0x24b0, 0x24b1, 0x24c0, 0x24c1], value),
            6 => two(EMPTY_ICON, [0x2cb0, 0x2cbe, 0x2cc0, 0x2cc1], value),
            7 => two(EMPTY_ICON, [0x287d, 0x287e, 0xe87e, 0xe87d], value),
            8 => two(EMPTY_ICON, [0x2876, 0x2877, 0xe877, 0xe876], value),
            9 => two(EMPTY_ICON, [0x2866, 0x2867, 0xe867, 0xe866], value),
            10 => two(EMPTY_ICON, [0x24bc, 0x24bd, 0x24cc, 0x24cd], value),
            11 => two(EMPTY_ICON, [0x20b6, 0x20b7, 0x20c6, 0x20c7], value),
            12 => [
                [0x20f5; 4],
                [0x20d0, 0x20d1, 0x20e0, 0x20e1],
                [0x2cd4, 0x2cd5, 0x2ce4, 0x2ce5],
                [0x2cd4, 0x2cd5, 0x2ce4, 0x2ce5],
            ][usize::from(value.min(3))],
            13 => two(EMPTY_ICON, [0x3c40, 0x3c41, 0x2842, 0x3c43], value),
            14 => two(EMPTY_ICON, [0x3ca5, 0x3ca6, 0x3cd8, 0x3cd9], value),
            15 | 28..=31 => BOTTLE_ICONS[usize::from(value.min(8))],
            16 => two(EMPTY_ICON, [0x24dc, 0x24dd, 0x24ec, 0x24ed], value),
            17 => two(EMPTY_ICON, [0x2cdc, 0x2cdd, 0x2cec, 0x2ced], value),
            18 => two(EMPTY_ICON, [0x24b4, 0x24b5, 0x24c4, 0x24c5], value),
            19 => [
                [0x20f5; 4],
                [0x28de, 0x28df, 0x28ee, 0x28ef],
                [0x2c62, 0x2c63, 0x2c72, 0x2c73],
                [0x2886, 0x2887, 0x2888, 0x2889],
            ][usize::from(value.min(3))],
            20 => [
                EMPTY_ICON,
                [0x2130, 0x2131, 0x2140, 0x2141],
                [0x28da, 0x28db, 0x28ea, 0x28eb],
            ][usize::from(value.min(2))],
            21 => two(EMPTY_ICON, [0x3429, 0x342a, 0x342b, 0x342c], value),
            22 => two(EMPTY_ICON, [0x2c9a, 0x2c9b, 0x2c9d, 0x2c9e], value),
            23 => two(EMPTY_ICON, [0x2433, 0x2434, 0x2435, 0x2436], value),
            24 => EMPTY_ICON,
            25 => [
                EMPTY_ICON,
                [0x2c64, 0x2cce, 0x2c75, 0x3d25],
                [0x2c8a, 0x2c65, 0x2474, 0x3d26],
                [0x248a, 0x2465, 0x3c74, 0x2d48],
                [0x288a, 0x2865, 0x2c74, 0x2d39],
            ][usize::from(value.min(4))],
            26 => [
                [0x24f5; 4],
                [0x2cfd, 0x6cfd, 0x2cfe, 0x6cfe],
                [0x34ff, 0x74ff, 0x349f, 0x749f],
                [0x2880, 0x2881, 0x288d, 0x288e],
            ][usize::from(value.min(3))],
            27 => [
                [0x3c68, 0x7c68, 0x3c78, 0x7c78],
                [0x2c68, 0x6c68, 0x2c78, 0x6c78],
                [0x2468, 0x6468, 0x2478, 0x6478],
                [0; 4],
                [0; 4],
            ][usize::from(value.min(4))],
            _ => EMPTY_ICON,
        }
    }

    fn set_tile(&mut self, index: usize, value: u16) {
        self.ram.write_u16(HUD_TILEMAP + index * 2, value);
    }

    fn item(&self, item: u8) -> u8 {
        self.ram.read_u8(LINK_ITEMS + usize::from(item) - 1)
    }

    fn max_rupees(&self) -> u16 {
        if self.feature(FEATURE_CARRY_MORE_RUPEES) {
            9999
        } else {
            999
        }
    }

    pub fn draw_y_button_items(&mut self) {
        const LETTERS: [[u16; 2]; 4] = [
            [0x3cf0, 0x3cf1],
            [0x2cf0, 0xacf0],
            [0x300e, 0x300f],
            [0x300c, 0x300d],
        ];
        const PALETTES: [u16; 4] = [7, 3, 4, 4];
        let button = self.button_index();
        if button >= 2 {
            self.runtime
                .render(HudRenderOp::SwitchLetterTiles(button as u8), self.ram);
        }
        self.draw_box(1, 5, 19, 19, PALETTES[button]);
        self.menu_set(hud_xy(2, 6), LETTERS[button][0]);
        self.menu_set(hud_xy(2, 7), LETTERS[button][1]);
        self.menu_set(hud_xy(3, 5), 0x246e);
        self.menu_set(hud_xy(4, 5), 0x246f);
        for (i, &position) in ITEM_POSITIONS.iter().enumerate() {
            let ordered = self.ram.read_u8(INVENTORY_ORDER + i);
            self.draw_item_at(
                position,
                self.icon_for(if ordered == 0 { i as u8 + 1 } else { ordered }),
            );
        }
    }

    pub fn draw_ability_box(&mut self) {
        const TEXT: [u16; 80] = [
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d27, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d61, 0x2d54, 0x2d50, 0x2d53, 0x2cf5, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d63, 0x2d50, 0x2d5b, 0x2d5a, 0x207f, 0x207f, 0x207f,
            0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x2cf5, 0x2cf5, 0x2c2e, 0x2cf5,
            0x2cf5, 0x2d5f, 0x2d64, 0x2d5b, 0x2d5b, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5,
            0x2cf5, 0x2d61, 0x2d64, 0x2d5d, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5,
            0x2d62, 0x2d66, 0x2d58, 0x2d5c, 0x2cf5, 0x2cf5, 0x2cf5, 0x207f, 0x207f, 0x2c01, 0x2c18,
            0x2c28, 0x207f, 0x207f,
        ];
        const GLOVE_TEXT: [u16; 20] = [
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d28, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d29,
        ];
        self.draw_box(1, 21, 19, 29, 1);
        let mut flags = self.ram.read_u8(ABILITY_FLAGS);
        for row in 0..2 {
            for col in 0..3 {
                if flags & 0x80 != 0 {
                    let off = row * 40 + col * 10;
                    self.draw_nxn(
                        hud_xy(4 + col * 5, 22 + row * 2),
                        &TEXT[off..off + 10],
                        5,
                        2,
                    );
                }
                flags <<= 1;
            }
            flags <<= 1;
        }
        self.menu_set(hud_xy(2, 22), 0xa4f0);
        self.menu_set(hud_xy(2, 23), 0x24f2);
        self.menu_set(hud_xy(3, 21), 0x2482);
        self.menu_set(hud_xy(4, 21), 0x2483);
        const GLOVES: [[u16; 4]; 3] = [
            EMPTY_ICON,
            [0x2130, 0x2131, 0x2140, 0x2141],
            [0x28da, 0x28db, 0x28ea, 0x28eb],
        ];
        const BOOTS: [[u16; 4]; 2] = [EMPTY_ICON, [0x3429, 0x342a, 0x342b, 0x342c]];
        const FLIPPERS: [[u16; 4]; 2] = [EMPTY_ICON, [0x2c9a, 0x2c9b, 0x2c9d, 0x2c9e]];
        const PEARL: [[u16; 4]; 2] = [EMPTY_ICON, [0x2433, 0x2434, 0x2435, 0x2436]];
        let gloves = usize::from(self.ram.read_u8(LINK_GLOVES).min(2));
        self.draw_item_at(hud_xy(8, 27), GLOVES[gloves]);
        self.draw_item_at(
            hud_xy(4, 27),
            BOOTS[usize::from(self.ram.read_u8(LINK_BOOTS).min(1))],
        );
        self.draw_item_at(
            hud_xy(12, 27),
            FLIPPERS[usize::from(self.ram.read_u8(LINK_FLIPPERS).min(1))],
        );
        self.draw_item_at(
            hud_xy(16, 27),
            PEARL[usize::from(self.ram.read_u8(LINK_MOON_PEARL).min(1))],
        );
        if gloves != 0 {
            let off = usize::from(gloves != 1) * 10;
            self.draw_nxn(hud_xy(4, 22), &GLOVE_TEXT[off..off + 10], 5, 2);
        }
    }

    pub fn draw_progress_icons(&mut self) {
        if self.ram.read_u8(SRAM_PROGRESS) < 3 {
            self.draw_progress_icons_pendants();
        } else {
            self.draw_progress_icons_crystals();
        }
    }

    pub fn draw_progress_icons_pendants(&mut self) {
        const BG: [u16; 90] = [
            0x28fb, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x68fb, 0x28fc,
            0x2521, 0x2522, 0x2523, 0x2524, 0x253f, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5,
            0x24f5, 0x213b, 0x213c, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x24f5,
            0x213d, 0x213e, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x213b, 0x213c, 0x24f5, 0x24f5,
            0x213b, 0x213c, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x213d, 0x213e, 0x24f5, 0x24f5, 0x213d,
            0x213e, 0x24f5, 0x68fc, 0xa8fb, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9,
            0xa8f9, 0xe8fb,
        ];
        let dst = hud_xy(21, 11);
        self.draw_nxn(dst, &BG, 10, 9);
        let flags = self.ram.read_u8(PENDANTS);
        let icons = [
            [
                [0x313b, 0x313c, 0x313d, 0x313e],
                [0x252b, 0x252c, 0x252d, 0x252e],
            ],
            [
                [0x313b, 0x313c, 0x313d, 0x313e],
                [0x2d2b, 0x2d2c, 0x2d2d, 0x2d2e],
            ],
            [
                [0x313b, 0x313c, 0x313d, 0x313e],
                [0x3d2b, 0x3d2c, 0x3d2d, 0x3d2e],
            ],
        ];
        self.draw_item_at(dst + hud_xy(4, 3), icons[0][usize::from(flags & 1)]);
        self.draw_item_at(dst + hud_xy(2, 6), icons[1][usize::from(flags >> 1 & 1)]);
        self.draw_item_at(dst + hud_xy(6, 6), icons[2][usize::from(flags >> 2 & 1)]);
    }

    pub fn draw_progress_icons_crystals(&mut self) {
        const BG: [u16; 90] = [
            0x28fb, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x68fb, 0x28fc,
            0x252f, 0x2534, 0x2535, 0x2536, 0x2537, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5,
            0x3146, 0x3147, 0x3146, 0x3147, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x3146, 0x3147, 0x3146,
            0x3147, 0x3146, 0x3147, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x3146, 0x3147, 0x3146, 0x3147,
            0x24f5, 0x24f5, 0x68fc, 0xa8fb, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9,
            0xa8f9, 0xe8fb,
        ];
        let dst = hud_xy(21, 11);
        self.draw_nxn(dst, &BG, 10, 9);
        let flags = self.ram.read_u8(CRYSTALS);
        for (bit, x, y) in [
            (1, 3, 3),
            (2, 5, 3),
            (4, 2, 5),
            (8, 4, 5),
            (16, 6, 5),
            (32, 3, 7),
            (64, 5, 7),
        ] {
            if flags & bit != 0 {
                self.menu_set(dst + hud_xy(x, y), 0x2d44);
                self.menu_set(dst + hud_xy(x + 1, y), 0x2d45);
            }
        }
    }

    pub fn draw_equipment_box(&mut self) {
        const LABELS: [u16; 16] = [
            0x2479, 0x247a, 0x247b, 0x247c, 0x248c, 0x24f5, 0x24f5, 0x24f5, 0x2469, 0x246a, 0x246b,
            0x246c, 0x246d, 0x246e, 0x246f, 0x24f5,
        ];
        const HEART_PIECE: [[u16; 4]; 4] = [
            [0x2484, 0x6484, 0x2485, 0x6485],
            [0x24ad, 0x6484, 0x2485, 0x6485],
            [0x24ad, 0x6484, 0x24ae, 0x6485],
            [0x24ad, 0x64ad, 0x24ae, 0x6485],
        ];
        const SWORD: [[u16; 4]; 5] = [
            EMPTY_ICON,
            [0x2c64, 0x2cce, 0x2c75, 0x3d25],
            [0x2c8a, 0x2c65, 0x2474, 0x3d26],
            [0x248a, 0x2465, 0x3c74, 0x2d48],
            [0x288a, 0x2865, 0x2c74, 0x2d39],
        ];
        const SHIELD: [[u16; 4]; 4] = [
            [0x24f5; 4],
            [0x2cfd, 0x6cfd, 0x2cfe, 0x6cfe],
            [0x34ff, 0x74ff, 0x349f, 0x749f],
            [0x2880, 0x2881, 0x288d, 0x288e],
        ];
        const ARMOR: [[u16; 4]; 3] = [
            [0x3c68, 0x7c68, 0x3c78, 0x7c78],
            [0x2c68, 0x6c68, 0x2c78, 0x6c78],
            [0x2468, 0x6468, 0x2478, 0x6478],
        ];
        self.draw_box(21, 21, 30, 29, 2);
        for x in 22..30 {
            self.menu_set(hud_xy(x, 25), 0x28d7);
        }
        self.draw_nxn(hud_xy(22, 22), &LABELS[..8], 8, 1);
        self.draw_nxn(hud_xy(22, 26), &LABELS[8..], 8, 1);
        let palace = self.ram.read_u16(CUR_PALACE_X2);
        if palace == 0xff {
            for x in 22..30 {
                self.menu_set(hud_xy(x, 26), 0x24f5);
            }
            self.draw_item_at(
                hud_xy(25, 27),
                HEART_PIECE[usize::from(self.ram.read_u8(HEART_PIECES).min(3))],
            );
        }
        let sword = self.ram.read_u8(LINK_SWORD);
        self.draw_item_at(
            hud_xy(22, 23),
            SWORD[if sword == 0xff {
                0
            } else {
                usize::from(sword.min(4))
            }],
        );
        self.draw_item_at(
            hud_xy(25, 23),
            SHIELD[usize::from(self.ram.read_u8(LINK_SHIELD).min(3))],
        );
        self.draw_item_at(
            hud_xy(28, 23),
            ARMOR[usize::from(self.ram.read_u8(LINK_ARMOR).min(2))],
        );
        if palace != 0xff {
            let shift = palace >> 1;
            if self
                .ram
                .read_u16(LINK_BIG_KEY)
                .wrapping_shl(u32::from(shift))
                & 0x8000
                != 0
            {
                let icon = if self.check_palace_item_possession() {
                    [0x354b, 0x354c, 0x354d, 0x354e]
                } else {
                    [0x28d6, 0x68d6, 0x28e6, 0x28e7]
                };
                self.draw_item_at(hud_xy(28, 27), icon);
            }
            if self
                .ram
                .read_u16(LINK_DUNGEON_MAP)
                .wrapping_shl(u32::from(shift))
                & 0x8000
                != 0
            {
                self.draw_item_at(hud_xy(22, 27), [0x28de, 0x28df, 0x28ee, 0x28ef]);
            }
            if self
                .ram
                .read_u16(LINK_COMPASS)
                .wrapping_shl(u32::from(shift))
                & 0x8000
                != 0
            {
                self.draw_item_at(hud_xy(25, 27), [0x24bf, 0x64bf, 0x2ccf, 0x6ccf]);
            }
        }
    }

    pub fn draw_selected_y_button_item(&mut self) {
        const BOTTLE_TEXT: [u16; 128] = [
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2564, 0x2562,
            0x2557, 0x2561, 0x255e, 0x255e, 0x255c, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x2551, 0x255e, 0x2563, 0x2563, 0x255b, 0x2554, 0x24f5, 0x24f5, 0x255b,
            0x2558, 0x2555, 0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2554, 0x2553, 0x2558,
            0x2552, 0x2558, 0x255d, 0x2554, 0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5,
            0x24f5, 0x255c, 0x2554, 0x2553, 0x2558, 0x2552, 0x2558, 0x255d, 0x2554, 0x2552, 0x2564,
            0x2561, 0x2554, 0x256a, 0x2550, 0x255b, 0x255b, 0x255c, 0x2554, 0x2553, 0x2558, 0x2552,
            0x2558, 0x255d, 0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x2555, 0x2550, 0x2554, 0x2561, 0x2558, 0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2551, 0x2554, 0x2554, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2556,
            0x255e, 0x255e, 0x2553, 0x24f5, 0x2551, 0x2554, 0x2554,
        ];
        const MUSHROOM_TEXT: [u16; 16] = [
            0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255f, 0x255e,
            0x2566, 0x2553, 0x2554, 0x2561, 0x24f5,
        ];
        const FLUTE_TEXT: [u16; 32] = [
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2555, 0x255b, 0x2564,
            0x2563, 0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x2555, 0x255b, 0x2564, 0x2563, 0x2554, 0x24f5, 0x24f5, 0x24f5,
        ];
        const MIRROR_TEXT: [u16; 16] = [
            0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c,
            0x2558, 0x2561, 0x2561, 0x255e, 0x2561,
        ];
        const BOW_TEXT: [u16; 48] = [
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x256b, 0x256c, 0x256e,
            0x256f, 0x257c, 0x257d, 0x257e, 0x257f, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x256b, 0x256c, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x256b,
            0x256c, 0x24f5, 0x256e, 0x256f, 0x24f5, 0x24f5, 0x24f5, 0x2578, 0x2579, 0x257a, 0x257b,
            0x257c, 0x257d, 0x257e, 0x257f,
        ];
        const ITEM_TEXT: [u16; 320] = [
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x256b, 0x256c, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x2570, 0x2571, 0x2572, 0x2573, 0x2574, 0x2575, 0x2576, 0x2577, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2557, 0x255e, 0x255e, 0x255a,
            0x2562, 0x2557, 0x255e, 0x2563, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x2551, 0x255e, 0x255c, 0x2551, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2564, 0x2562, 0x2557, 0x2561,
            0x255e, 0x255e, 0x255c, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x2555, 0x2558, 0x2561, 0x2554, 0x2561, 0x255e, 0x2553, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2558, 0x2552, 0x2554, 0x2561, 0x255e, 0x2553,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2551,
            0x255e, 0x255c, 0x2551, 0x255e, 0x2562, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2554, 0x2563, 0x2557, 0x2554, 0x2561, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2560, 0x2564,
            0x2550, 0x255a, 0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x255b, 0x2550, 0x255c, 0x255f, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2557,
            0x2550, 0x255c, 0x255c, 0x2554, 0x2561, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x2562, 0x2557, 0x255e, 0x2565, 0x2554, 0x255b, 0x24f5, 0x24f5, 0x2400,
            0x2401, 0x2402, 0x2403, 0x2404, 0x2405, 0x2406, 0x2407, 0x2408, 0x2409, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2551, 0x255e, 0x255e, 0x255a, 0x24f5, 0x255e, 0x2555,
            0x24f5, 0x255c, 0x2564, 0x2553, 0x255e, 0x2561, 0x2550, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2564, 0x2562, 0x2557, 0x2561,
            0x255e, 0x255e, 0x255c, 0x2552, 0x2550, 0x255d, 0x2554, 0x24f5, 0x255e, 0x2555, 0x24f5,
            0x24f5, 0x2562, 0x255e, 0x255c, 0x2550, 0x2561, 0x2558, 0x2550, 0x2552, 0x2550, 0x255d,
            0x2554, 0x24f5, 0x255e, 0x2555, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2551, 0x2568, 0x2561,
            0x255d, 0x2550, 0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x2552, 0x2550, 0x255f, 0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5,
        ];
        const PALETTES: [u16; 4] = [7, 3, 4, 4];
        const NOT_ASSIGNED: [u16; 16] = [
            0x255d, 0x255e, 0x2563, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2550, 0x2562, 0x2562,
            0x2558, 0x2556, 0x255d, 0x2554, 0x2553,
        ];
        let button = self.button_index();
        let item = self.ram.read_u8(Self::button_address(button));
        self.draw_box(21, 5, 30, 10, PALETTES[button]);
        if item != 0 {
            let pos = ITEM_POSITIONS[self.current_item_position(item).max(0) as usize];
            self.copy_2x2(hud_xy(25, 6), pos);
            if self.ram.read_u8(FLASH_TIMER) & 0x10 != 0 {
                self.draw_flashing_circle(pos, PALETTES[button]);
            }
        }
        let (src, off): (&[u16], usize) = if item == 16 && self.ram.read_u8(LINK_BOTTLE_INDEX) != 0
        {
            let slot = usize::from((self.ram.read_u8(LINK_BOTTLE_INDEX) - 1).min(3));
            (
                &BOTTLE_TEXT,
                usize::from(self.ram.read_u8(BOTTLES + slot).saturating_sub(1).min(7)) * 16,
            )
        } else if item == 5 && self.item(5) != 1 {
            (&MUSHROOM_TEXT, 0)
        } else if item == 20 && self.item(20) != 1 {
            (&MIRROR_TEXT, 0)
        } else if item == 13 && self.item(13) != 1 {
            (
                &FLUTE_TEXT,
                usize::from(self.item(13).saturating_sub(2).min(1)) * 16,
            )
        } else if item == 1 && self.item(1) != 1 {
            (
                &BOW_TEXT,
                usize::from(self.item(1).saturating_sub(2).min(2)) * 16,
            )
        } else if item == 0 && button != 0 {
            (&NOT_ASSIGNED, 0)
        } else {
            let index = if item == 0 {
                19
            } else {
                usize::from(item.saturating_sub(1).min(19))
            };
            (&ITEM_TEXT, index * 16)
        };
        self.draw_nxn(hud_xy(22, 8), &src[off..off + 16], 8, 2);
    }

    pub fn draw_bottle_menu(&mut self) {
        self.draw_box(21, 11, 30, 29, 2);
        for i in 0..4 {
            let kind = usize::from(self.ram.read_u8(BOTTLES + i).min(8));
            self.draw_item_at(hud_xy(25, 13 + i * 4), BOTTLE_ICONS[kind]);
        }
        let selected = self.ram.read_u8(LINK_BOTTLE_INDEX);
        if selected != 0 {
            let slot = usize::from((selected - 1).min(3));
            let kind = usize::from(self.ram.read_u8(BOTTLES + slot).min(8));
            self.draw_item_at(ITEM_POSITIONS[15], BOTTLE_ICONS[kind]);
            if self.ram.read_u8(FLASH_TIMER) & 0x10 != 0 {
                self.draw_flashing_circle(hud_xy(25, 13 + slot * 4), 7);
            }
        }
    }

    pub fn draw_bottle_menu_update(&mut self) {
        self.draw_bottle_menu();
        self.draw_selected_y_button_item();
        self.ram.write_u8(NMI_SUBROUTINE, 1);
        self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
    }

    pub fn current_item_position(&self, item: u8) -> i32 {
        if item == 0 {
            return -1;
        }
        if self.ram.read_u8(INVENTORY_ORDER) == 0 {
            return i32::from(item - 1);
        }
        (0..HUD_ITEM_COUNT)
            .find(|&i| self.ram.read_u8(INVENTORY_ORDER + i) == item)
            .unwrap_or(HUD_ITEM_COUNT - 1) as i32
    }

    fn goto_previous(&mut self, item: &mut u8, first_item: bool) {
        if self.ram.read_u8(INVENTORY_ORDER) != 0 {
            let pos = self.current_item_position(*item);
            *item = if pos == 0 && !first_item {
                0
            } else {
                let pos = if pos <= 0 {
                    HUD_ITEM_COUNT
                } else {
                    pos as usize
                };
                self.ram.read_u8(INVENTORY_ORDER + pos - 1)
            };
        } else {
            *item = if usize::from(*item) > usize::from(first_item) {
                item.wrapping_sub(1)
            } else {
                HUD_ITEM_COUNT as u8
            };
        }
    }

    fn goto_next(&mut self, item: &mut u8, first_item: bool) {
        if self.ram.read_u8(INVENTORY_ORDER) != 0 {
            let pos = self.current_item_position(*item);
            let next = if pos < 0 || pos as usize >= HUD_ITEM_COUNT - 1 {
                0
            } else {
                pos as usize + 1
            };
            *item = self.ram.read_u8(INVENTORY_ORDER + next);
        } else {
            *item = if usize::from(*item) < HUD_ITEM_COUNT {
                item.wrapping_add(1)
            } else {
                u8::from(first_item)
            };
        }
    }

    pub fn have_item(&self, item: u8) -> bool {
        if item == 0 {
            return true;
        }
        self.item(item) != 0
    }

    fn equip_previous(&mut self, item: &mut u8, first_item: bool) {
        loop {
            self.goto_previous(item, first_item);
            if self.have_item(*item) {
                break;
            }
        }
    }

    fn equip_next(&mut self, item: &mut u8, first_item: bool) {
        loop {
            self.goto_next(item, first_item);
            if self.have_item(*item) {
                break;
            }
        }
    }

    fn equip_vertical(&mut self, item: &mut u8, down: bool) {
        let count = if *item == 0 && down { 1 } else { 5 };
        loop {
            for _ in 0..count {
                if down {
                    self.goto_next(item, true);
                } else {
                    self.goto_previous(item, true);
                }
            }
            if self.have_item(*item) {
                break;
            }
        }
    }

    pub fn have_any_items(&self) -> bool {
        self.ram.slice(LINK_ITEMS, 20).iter().any(|&v| v != 0)
    }

    pub fn search_for_equipped_item(&mut self) {
        if !self.have_any_items() {
            self.ram.write_u8(HUD_CUR_ITEM, 0);
            self.ram.write_u8(HUD_VAR1, 0);
            return;
        }
        let mut item = self.ram.read_u8(HUD_CUR_ITEM);
        if item == 0 {
            item = 1;
        }
        if !self.have_item(item) {
            self.equip_next(&mut item, true);
        }
        self.ram.write_u8(HUD_CUR_ITEM, item);
    }

    pub fn check_palace_item_possession(&self) -> bool {
        match self.ram.read_u16(CUR_PALACE_X2) >> 1 {
            2 => self.ram.read_u8(LINK_BOW) != 0,
            3 => self.ram.read_u8(LINK_GLOVES) != 0,
            5 => self.ram.read_u8(LINK_ITEMS + 2) != 0,
            6 => self.ram.read_u8(LINK_ITEMS + 11) != 0,
            7 => self.ram.read_u8(LINK_ITEMS + 16) != 0,
            8 => self.ram.read_u8(LINK_ITEMS + 5) != 0,
            9 => self.ram.read_u8(LINK_ARMOR) != 0,
            10 => self.ram.read_u8(LINK_ITEMS + 23) != 0,
            11 => self.ram.read_u8(LINK_GLOVES) != 1,
            12 => self.ram.read_u8(LINK_SHIELD) == 3,
            13 => self.ram.read_u8(LINK_ARMOR) == 2,
            _ => false,
        }
    }

    pub fn remove_super_bomb_indicator(&mut self) {
        for i in [0xf2 / 2, 0x132 / 2, 0xf4 / 2, 0x134 / 2] {
            self.set_tile(i, 0x7f);
        }
    }

    pub fn floor_indicator(&mut self) {
        const TOP: [u16; 11] = [
            0x2508, 0x2509, 0x2509, 0x250a, 0x250b, 0x250c, 0x250d, 0x251d, 0xe51c, 0x250e, 0x7f,
        ];
        const BOTTOM: [u16; 11] = [
            0x2518, 0x2519, 0xa509, 0x251a, 0x251b, 0x251c, 0x2518, 0xa51d, 0xe50c, 0xa50e, 0x7f,
        ];
        let mut timer = self.ram.read_u16(FLOOR_TIMER);
        if timer == 0 {
            self.remove_super_bomb_indicator();
            return;
        }
        timer = timer.wrapping_add(1);
        if timer == 0xc0 {
            timer = 0;
        }
        self.ram.write_u16(FLOOR_TIMER, timer);
        self.set_tile(0xf2 / 2, 0x251e);
        self.set_tile(0x134 / 2, 0x251f);
        self.set_tile(0x132 / 2, 0x2520);
        self.set_tile(0xf4 / 2, 0x250f);
        let floor = self.ram.read_u8(DUNGEON_FLOOR);
        let (k, j) = if (floor as i8) < 0 {
            self.ram.write_u8(SOUND_AMBIENT, 5);
            (1, usize::from(floor ^ 0xff))
        } else {
            if self.ram.read_u16(DUNGEON_FLOOR) == 0
                && self.ram.read_u16(DUNGEON_ROOM) != 2
                && self.ram.read_u8(SRAM_PROGRESS) < 2
            {
                self.ram.write_u8(SOUND_AMBIENT, 3);
            }
            (0, usize::from(floor))
        };
        self.set_tile(k + 0xf2 / 2, TOP[j]);
        self.set_tile(k + 0x132 / 2, BOTTOM[j]);
        self.increment(FLAG_UPDATE_HUD);
    }

    pub fn super_bomb_indicator(&mut self) {
        if self.ram.read_u8(SUPER_BOMB_TIMER) == 0 {
            if self.ram.read_i8(SUPER_BOMB_VALUE) < 0 {
                self.ram.write_u8(SUPER_BOMB_VALUE, 0xff);
                self.remove_super_bomb_indicator();
                return;
            }
            self.ram.write_u8(
                SUPER_BOMB_VALUE,
                self.ram.read_u8(SUPER_BOMB_VALUE).wrapping_sub(1),
            );
            self.ram.write_u8(SUPER_BOMB_TIMER, 62);
        }
        self.ram.write_u8(
            SUPER_BOMB_TIMER,
            self.ram.read_u8(SUPER_BOMB_TIMER).wrapping_sub(1),
        );
        let value = self.ram.read_u8(SUPER_BOMB_VALUE);
        if (value as i8) < 0 {
            self.ram.write_u8(SUPER_BOMB_VALUE, 0xff);
            self.remove_super_bomb_indicator();
            return;
        }
        const TOP: [u16; 11] = [
            0x2508, 0x2509, 0x2509, 0x250a, 0x250b, 0x250c, 0x250d, 0x251d, 0xe51c, 0x250e, 0x7f,
        ];
        const BOTTOM: [u16; 11] = [
            0x2518, 0x2519, 0xa509, 0x251a, 0x251b, 0x251c, 0x2518, 0xa51d, 0xe50c, 0xa50e, 0x7f,
        ];
        let ones = value % 10;
        let tens = value / 10;
        let oi = if ones == 0 { 9 } else { usize::from(ones - 1) };
        let ti = if tens == 0 { 10 } else { usize::from(tens - 1) };
        self.set_tile(0xf4 / 2, TOP[oi]);
        self.set_tile(0x134 / 2, BOTTOM[oi]);
        self.set_tile(0xf2 / 2, TOP[ti]);
        self.set_tile(0x132 / 2, BOTTOM[ti]);
    }

    pub fn refill_logic(&mut self) {
        if self.ram.read_u8(MAP_STATE) != 0 {
            return;
        }
        self.refill_magic_counter();
        self.adjust_rupees();
        self.adjust_ammunition();
        self.low_health_beep();
        if self.ram.read_u8(HEART_ANIMATING) != 0 {
            self.update_magic();
            self.update_inventory();
            self.animate_heart_refill();
            self.increment(FLAG_UPDATE_HUD);
            return;
        }
        let filler = self.ram.read_u8(HEARTS_FILLER);
        if filler != 0 && self.ram.read_u8(HEALTH_CURRENT) < self.ram.read_u8(HEALTH_CAPACITY) {
            let health = self
                .ram
                .read_u8(HEALTH_CURRENT)
                .saturating_add(8)
                .min(self.ram.read_u8(HEALTH_CAPACITY));
            self.ram.write_u8(HEALTH_CURRENT, health);
            if self.ram.read_u8(SOUND_2) == 0 {
                self.ram.write_u8(SOUND_2, 13);
            }
            self.ram.write_u8(HEARTS_FILLER, filler.wrapping_sub(8));
            self.increment(HEART_ANIMATING);
            self.ram.write_u8(HEART_ANIM_TIMER, 7);
            self.update_magic();
            self.update_inventory();
            self.animate_heart_refill();
        } else {
            if filler != 0 {
                self.ram
                    .write_u8(HEALTH_CURRENT, self.ram.read_u8(HEALTH_CAPACITY));
                self.ram.write_u8(HEARTS_FILLER, 0);
            }
            self.update_hearts();
            self.update_magic();
            self.update_inventory();
        }
        self.increment(FLAG_UPDATE_HUD);
    }

    fn refill_magic_counter(&mut self) {
        let filler = self.ram.read_u8(MAGIC_FILLER);
        if filler == 0 {
            return;
        }
        if self.ram.read_u8(MAGIC_POWER) >= 128 {
            self.ram.write_u8(MAGIC_POWER, 128);
            self.ram.write_u8(MAGIC_FILLER, 0);
        } else {
            self.ram.write_u8(MAGIC_FILLER, filler - 1);
            self.increment(MAGIC_POWER);
            if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 && self.ram.read_u8(SOUND_1) == 0 {
                self.ram.write_u8(SOUND_1, 45);
            }
        }
    }

    fn adjust_rupees(&mut self) {
        let mut actual = self.ram.read_u16(RUPEES_ACTUAL);
        let mut goal = self.ram.read_u16(RUPEES_GOAL);
        if actual == goal {
            self.ram.write_u8(RUPEE_SOUND_DELAY, 0);
            return;
        }
        if actual >= goal {
            actual = actual.wrapping_sub(1);
            if (actual as i16) < 0 {
                actual = 0;
                goal = 0;
            }
        } else {
            actual = actual.wrapping_add(1);
            if actual > self.max_rupees() {
                actual = self.max_rupees();
                goal = actual;
            }
        }
        self.ram.write_u16(RUPEES_ACTUAL, actual);
        self.ram.write_u16(RUPEES_GOAL, goal);
        if self.ram.read_u8(SOUND_1) == 0 {
            let delay = self.ram.read_u8(RUPEE_SOUND_DELAY);
            self.ram.write_u8(RUPEE_SOUND_DELAY, delay.wrapping_add(1));
            if delay & 7 == 0 {
                self.ram.write_u8(SOUND_1, 41);
            }
        } else {
            self.ram.write_u8(RUPEE_SOUND_DELAY, 0);
        }
    }

    fn adjust_ammunition(&mut self) {
        let bombs = self.ram.read_u8(BOMB_FILLER);
        if bombs != 0 {
            self.ram.write_u8(BOMB_FILLER, bombs - 1);
            if self.ram.read_u8(LINK_BOMBS)
                != MAX_BOMBS_FOR_LEVEL[usize::from(self.ram.read_u8(BOMB_UPGRADES))]
            {
                self.increment(LINK_BOMBS);
            }
        }
        let arrows = self.ram.read_u8(ARROW_FILLER);
        if arrows != 0 {
            self.ram.write_u8(ARROW_FILLER, arrows - 1);
            if self.ram.read_u8(NUM_ARROWS)
                != MAX_ARROWS_FOR_LEVEL[usize::from(self.ram.read_u8(ARROW_UPGRADES))]
            {
                self.increment(NUM_ARROWS);
            }
            let bow = self.ram.read_u8(LINK_BOW);
            if bow != 0 && bow & 1 != 0 {
                self.ram.write_u8(LINK_BOW, bow + 1);
                self.refresh_icon();
            }
        }
    }

    fn low_health_beep(&mut self) {
        let max = MAX_HEALTH_FOR_LEVEL[usize::from(self.ram.read_u8(HEALTH_CAPACITY) >> 3)];
        if self.ram.read_u8(LINK_IMMOBILIZED) != 0
            || self.ram.read_u8(HEARTS_FILLER) != 0
            || self.ram.read_u8(HEALTH_CURRENT) >= max
        {
            return;
        }
        let timer = self.ram.read_u8(LOW_HEALTH_TIMER);
        if timer != 0 {
            self.ram.write_u8(LOW_HEALTH_TIMER, timer - 1);
        } else if self.ram.read_u8(SOUND_1) == 0 {
            if !self.feature(FEATURE_DISABLE_LOW_HEALTH_BEEP) {
                self.ram.write_u8(SOUND_1, 43);
            }
            self.ram.write_u8(LOW_HEALTH_TIMER, 31);
        }
    }

    pub fn refresh_icon(&mut self) {
        self.search_for_equipped_item();
        self.update_hud();
        self.rebuild();
        self.ram.write_u8(MAP_STATE, 0);
    }

    pub fn run_module(&mut self) {
        self.increment(HUD_FRAME);
        match self.ram.read_u8(MAP_STATE) {
            0 => self.clear_tilemap(),
            1 => self.init_menu(),
            2 => self.bring_menu_down(),
            3 => self.choose_next_mode(),
            4 => self.normal_menu(),
            5 => self.update_hud(),
            6 => self.close_menu(),
            7 => self.goto_bottle_menu(),
            8 => self.init_bottle_menu(),
            9 => self.expand_bottle_menu(),
            10 => self.bottle_menu(),
            11 => self.erase_bottle_menu(),
            12 => self.restore_normal_menu(),
            _ => panic!("invalid HUD map state"),
        }
    }

    pub fn clear_tilemap(&mut self) {
        for i in 0..1024 {
            self.ram.write_u16(0x1000 + i * 2, 0x207f);
        }
        self.ram.write_u8(SOUND_2, 17);
        self.ram.write_u8(NMI_SUBROUTINE, 1);
        self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
        self.increment(MAP_STATE);
    }

    pub fn init_menu(&mut self) {
        self.search_for_equipped_item();
        self.draw_y_button_items();
        self.draw_ability_box();
        self.draw_progress_icons();
        self.draw_equipment_box();
        self.draw_selected_y_button_item();
        if self.have_any_items() {
            let first = (0..4).find(|&i| self.ram.read_u8(BOTTLES + i) != 0);
            match first {
                None => self.ram.write_u8(LINK_BOTTLE_INDEX, 0),
                Some(slot) if self.ram.read_u8(LINK_BOTTLE_INDEX) == 0 => {
                    self.ram.write_u8(LINK_BOTTLE_INDEX, slot as u8 + 1);
                }
                Some(_) => {}
            }
            if self.ram.read_u8(HUD_CUR_ITEM) == 16 {
                self.ram.write_u8(FLASH_TIMER, 16);
                self.draw_bottle_menu();
            }
        }
        self.ram.write_u8(FLASH_TIMER, 16);
        self.ram.write_u8(NMI_SUBROUTINE, 1);
        self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
        self.increment(MAP_STATE);
    }

    pub fn bring_menu_down(&mut self) {
        let y = self.ram.read_u16(BG3VOFS_COPY2).wrapping_sub(8);
        self.ram.write_u16(BG3VOFS_COPY2, y);
        if y == 0xff18 {
            self.increment(MAP_STATE);
        }
    }

    pub fn choose_next_mode(&mut self) {
        if self.have_any_items() {
            self.ram.write_u8(NMI_SUBROUTINE, 1);
            self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
            self.draw_selected_y_button_item();
            self.ram.write_u8(
                MAP_STATE,
                if self.ram.read_u8(HUD_CUR_ITEM) == 16 {
                    10
                } else {
                    4
                },
            );
        } else if self.ram.read_u8(FILTERED_JOYPAD_H) != 0 {
            self.ram.write_u8(MAP_STATE, 5);
        }
    }

    pub fn button_index(&self) -> usize {
        if !self.feature(FEATURE_SWITCH_LR) {
            return 0;
        }
        let joy = self.ram.read_u8(JOYPAD_L_LAST);
        if joy & JOY_X != 0 {
            1
        } else if joy & JOY_L != 0 {
            2
        } else if joy & JOY_R != 0 {
            3
        } else {
            0
        }
    }

    pub fn button_address(index: usize) -> usize {
        [HUD_CUR_ITEM, HUD_CUR_ITEM_X, HUD_CUR_ITEM_L, HUD_CUR_ITEM_R][index]
    }

    pub fn normal_menu(&mut self) {
        self.increment(FLASH_TIMER);
        if self.ram.read_u8(JOYPAD_H_LAST) == 0 {
            self.ram.write_u8(HUD_TMP1, 0);
        }
        let filtered = self.ram.read_u8(FILTERED_JOYPAD_H);
        if filtered & JOY_START != 0 {
            self.ram.write_u8(MAP_STATE, 5);
            self.ram.write_u8(SOUND_2, 18);
            return;
        }
        if self.ram.read_u8(JOYPAD_H_LAST) & JOY_SELECT != 0 && self.ram.read_u8(SRAM_PROGRESS) != 0
        {
            self.ram.write_u16(BG3VOFS_COPY2, 0xfff8);
            self.close_menu();
            self.runtime.display_select_menu(self.ram);
            return;
        }
        if self.ram.read_u8(JOYPAD_H_LAST) & JOY_Y != 0
            && self.ram.read_u8(JOYPAD_L_LAST) & JOY_X == 0
            && self.feature(FEATURE_SWITCH_LR)
        {
            if filtered & JOY_UP != 0 {
                self.reorder_item(-5);
            } else if filtered & JOY_DOWN != 0 {
                self.reorder_item(5);
            } else if filtered & JOY_LEFT != 0 {
                self.reorder_item(-1);
            } else if filtered & JOY_RIGHT != 0 {
                self.reorder_item(1);
            }
        } else if self.ram.read_u8(HUD_TMP1) == 0 {
            let index = self.button_index();
            let address = Self::button_address(index);
            let mut item = self.ram.read_u8(address);
            let old = item;
            if filtered & JOY_UP != 0 {
                self.equip_vertical(&mut item, false);
            } else if filtered & JOY_DOWN != 0 {
                self.equip_vertical(&mut item, true);
            } else if filtered & JOY_LEFT != 0 {
                self.equip_previous(&mut item, address == HUD_CUR_ITEM);
            } else if filtered & JOY_RIGHT != 0 {
                self.equip_next(&mut item, address == HUD_CUR_ITEM);
            }
            self.ram.write_u8(address, item);
            self.ram.write_u8(HUD_TMP1, filtered);
            if item != old {
                self.ram.write_u8(FLASH_TIMER, 16);
                self.ram.write_u8(SOUND_2, 32);
            }
        }
        self.draw_y_button_items();
        self.draw_selected_y_button_item();
        if self.ram.read_u8(HUD_CUR_ITEM) == 16 {
            self.ram.write_u8(MAP_STATE, 7);
        }
        self.ram.write_u8(NMI_SUBROUTINE, 1);
        self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
    }

    pub fn update_hud(&mut self) {
        self.increment(MAP_STATE);
        self.rebuild();
        self.update_equipped_item();
    }

    pub fn lookup_inventory_item(item: u8) -> u8 {
        const MAP: [u8; 21] = [
            0, 3, 2, 14, 1, 10, 5, 6, 15, 16, 17, 9, 4, 8, 7, 12, 11, 18, 13, 19, 20,
        ];
        MAP[usize::from(item)]
    }

    pub fn update_equipped_item(&mut self) {
        let item = self.ram.read_u8(HUD_CUR_ITEM);
        if item >= 21 {
            self.ram.write_u8(LINK_BOTTLE_INDEX, item - 21 + 1);
        }
        self.ram
            .write_u8(CURRENT_ITEM_Y, Self::lookup_inventory_item(item));
    }

    pub fn close_menu(&mut self) {
        let y = self.ram.read_u16(BG3VOFS_COPY2).wrapping_add(8);
        self.ram.write_u16(BG3VOFS_COPY2, y);
        if y != 0 {
            return;
        }
        self.rebuild();
        self.ram.write_u8(MAP_STATE, 0);
        self.ram.write_u8(SUBMODULE, 0);
        self.ram
            .write_u8(MAIN_MODULE, self.ram.read_u8(SAVED_MODULE_FOR_MENU));
        // The C source tests `submodule_index` immediately after clearing it,
        // so its torch-restoration call is intentionally unreachable here.
        if !matches!(self.ram.read_u8(CURRENT_ITEM_Y), 5 | 6) {
            self.ram.write_u8(EQ_DEBUG_VARIABLE, 2);
            self.ram.write_u8(LINK_DEBUG_VALUE_1, 0);
        } else {
            debug_assert_eq!(self.ram.read_u8(LINK_DEBUG_VALUE_1), 0);
            self.ram.write_u8(EQ_DEBUG_VARIABLE, 0);
        }
    }

    pub fn goto_bottle_menu(&mut self) {
        self.ram.write_u8(BOTTLE_EXPAND_ROW, 0);
        self.increment(MAP_STATE);
    }

    pub fn init_bottle_menu(&mut self) {
        let row = self.ram.read_u8(BOTTLE_EXPAND_ROW);
        for x in 21..=30 {
            self.menu_set(hud_xy(x, 11 + usize::from(row)), 0x207f);
        }
        let next = row + 1;
        self.ram.write_u8(BOTTLE_EXPAND_ROW, next);
        if next == 19 {
            self.increment(MAP_STATE);
            self.ram.write_u8(BOTTLE_EXPAND_ROW, 17);
        }
        self.ram.write_u8(NMI_SUBROUTINE, 1);
        self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
    }

    pub fn expand_bottle_menu(&mut self) {
        const TOP: [u16; 10] = [
            0x28fb, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x68fb,
        ];
        const MID: [u16; 10] = [
            0x28fc, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc,
        ];
        const BOTTOM: [u16; 10] = [
            0xa8fb, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xe8fb,
        ];
        let current = usize::from(self.ram.read_u8(BOTTLE_EXPAND_ROW));
        self.draw_nxn(hud_xy(21, 11 + current), &TOP, 10, 1);
        self.draw_nxn(hud_xy(21, 12 + current), &MID, 10, 1);
        self.draw_nxn(hud_xy(21, 29), &BOTTOM, 10, 1);
        let row = self.ram.read_u8(BOTTLE_EXPAND_ROW).wrapping_sub(1);
        self.ram.write_u8(BOTTLE_EXPAND_ROW, row);
        if (row as i8) < 0 {
            self.increment(MAP_STATE);
        }
        self.ram.write_u8(NMI_SUBROUTINE, 1);
        self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
    }

    pub fn bottle_menu(&mut self) {
        self.increment(FLASH_TIMER);
        let input = self.ram.read_u8(FILTERED_JOYPAD_H);
        if input & JOY_START != 0 {
            self.ram.write_u8(SOUND_2, 18);
            self.ram.write_u8(MAP_STATE, 5);
        } else if input & (JOY_LEFT | JOY_RIGHT) != 0 {
            let mut item = self.ram.read_u8(HUD_CUR_ITEM);
            if input & JOY_LEFT != 0 {
                self.equip_previous(&mut item, true);
            } else {
                self.equip_next(&mut item, true);
            }
            self.ram.write_u8(HUD_CUR_ITEM, item);
            self.ram.write_u8(FLASH_TIMER, 16);
            self.ram.write_u8(SOUND_2, 32);
            self.draw_y_button_items();
            self.draw_selected_y_button_item();
            self.increment(MAP_STATE);
            self.ram.write_u8(BOTTLE_EXPAND_ROW, 0);
            return;
        }
        self.draw_bottle_menu_update();
        if input & (JOY_DOWN | JOY_UP) != 0 {
            let old = self.ram.read_u8(LINK_BOTTLE_INDEX) - 1;
            let mut value = old;
            loop {
                value = if input & JOY_UP != 0 {
                    value.wrapping_sub(1) & 3
                } else {
                    value.wrapping_add(1) & 3
                };
                if self.ram.read_u8(BOTTLES + usize::from(value)) != 0 {
                    break;
                }
            }
            if old != value {
                self.ram.write_u8(LINK_BOTTLE_INDEX, value + 1);
                self.ram.write_u8(FLASH_TIMER, 16);
                self.ram.write_u8(SOUND_2, 32);
            }
        }
    }

    pub fn erase_bottle_menu(&mut self) {
        let row = self.ram.read_u8(BOTTLE_EXPAND_ROW);
        for x in 21..=30 {
            self.menu_set(hud_xy(x, 11 + usize::from(row)), 0x207f);
        }
        self.ram.write_u8(BOTTLE_EXPAND_ROW, row + 1);
        if row + 1 == 19 {
            self.increment(MAP_STATE);
        }
        self.ram.write_u8(NMI_SUBROUTINE, 1);
        self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
    }

    pub fn restore_normal_menu(&mut self) {
        self.draw_progress_icons();
        self.draw_equipment_box();
        self.ram.write_u8(MAP_STATE, 4);
        self.ram.write_u8(NMI_SUBROUTINE, 1);
        self.ram.write_u8(NMI_LOAD_TARGET, 0x22);
    }

    pub fn refill_health(&mut self) -> bool {
        if self.ram.read_u8(HEALTH_CURRENT) >= self.ram.read_u8(HEALTH_CAPACITY) {
            self.ram
                .write_u8(HEALTH_CURRENT, self.ram.read_u8(HEALTH_CAPACITY));
            self.ram.write_u8(HEARTS_FILLER, 0);
            return self.ram.read_u8(HEART_ANIMATING) == 0;
        }
        self.ram.write_u8(HEARTS_FILLER, 160);
        false
    }

    pub fn refill_magic_power(&mut self) -> bool {
        if self.ram.read_u8(MAGIC_POWER) >= 0x80 {
            true
        } else {
            self.ram.write_u8(MAGIC_FILLER, 0x80);
            false
        }
    }

    pub fn animate_heart_refill(&mut self) {
        let timer = self.ram.read_u8(HEART_ANIM_TIMER).wrapping_sub(1);
        self.ram.write_u8(HEART_ANIM_TIMER, timer);
        if timer != 0 {
            return;
        }
        let mut n = u16::from((self.ram.read_u8(HEALTH_CURRENT) & !7).wrapping_sub(1) >> 3) << 1;
        let mut index = hud_xy(20, 1);
        if n >= 20 {
            n -= 20;
            index += 0x20;
        }
        self.ram.write_u8(HEART_ANIM_TIMER, 1);
        const PARTIAL: [u16; 4] = [0x24a3, 0x24a4, 0x24a3, 0x24a0];
        self.set_tile(
            index + usize::from(n >> 1),
            PARTIAL[usize::from(self.ram.read_u8(HEART_ANIM_SUBPOS))],
        );
        let sub = (self.ram.read_u8(HEART_ANIM_SUBPOS) + 1) & 3;
        self.ram.write_u8(HEART_ANIM_SUBPOS, sub);
        if sub == 0 {
            self.rebuild();
            self.ram.write_u8(HEART_ANIMATING, 0);
        }
    }

    pub fn restore_torch_background(&mut self) {
        if self.ram.read_u8(LINK_TORCH) == 0
            || self.ram.read_u8(DUNGEON_WANTS_LIGHTS_OUT) == 0
            || self.ram.read_u8(DARK_WITH_LANTERN) != 0
            || self.ram.read_u8(NUM_LIT_TORCHES) != 0
        {
            return;
        }
        self.ram.write_u8(DARK_WITH_LANTERN, 1);
        if self.ram.read_u8(DUNGEON_BG2_PROPERTIES) != 2 {
            self.ram.write_u8(TS_COPY, 1);
        }
    }

    pub fn rebuild_indoor(&mut self) {
        self.ram.write_u8(FIXED_COLOR, 0);
        self.ram.write_u8(NUM_KEYS, 0xff);
        self.rebuild();
    }

    fn update_hearts_inner(&mut self, src: [u16; 3], mut n: i32) {
        let mut x = 0;
        let mut row = 0;
        while n > 0 {
            if x >= 10 {
                row += 1;
                x = 0;
            }
            self.set_tile(hud_xy(20 + x, 1 + row), src[if n >= 5 { 2 } else { 1 }]);
            x += 1;
            n -= 8;
        }
    }

    fn update_hearts(&mut self) {
        self.update_hearts_inner([0x24a2; 3], i32::from(self.ram.read_u8(HEALTH_CAPACITY)));
        self.update_hearts_inner(
            [0x24a2, 0x24a1, 0x24a0],
            i32::from(self.ram.read_u8(HEALTH_CURRENT).wrapping_add(3) & !3),
        );
    }

    fn update_magic(&mut self) {
        const MAGIC: [[u16; 4]; 17] = [
            [0x3cf5, 0x3cf5, 0x3cf5, 0x3cf5],
            [0x3cf5, 0x3cf5, 0x3cf5, 0x3c5f],
            [0x3cf5, 0x3cf5, 0x3cf5, 0x3c4c],
            [0x3cf5, 0x3cf5, 0x3cf5, 0x3c4d],
            [0x3cf5, 0x3cf5, 0x3cf5, 0x3c4e],
            [0x3cf5, 0x3cf5, 0x3c5f, 0x3c5e],
            [0x3cf5, 0x3cf5, 0x3c4c, 0x3c5e],
            [0x3cf5, 0x3cf5, 0x3c4d, 0x3c5e],
            [0x3cf5, 0x3cf5, 0x3c4e, 0x3c5e],
            [0x3cf5, 0x3c5f, 0x3c5e, 0x3c5e],
            [0x3cf5, 0x3c4c, 0x3c5e, 0x3c5e],
            [0x3cf5, 0x3c4d, 0x3c5e, 0x3c5e],
            [0x3cf5, 0x3c4e, 0x3c5e, 0x3c5e],
            [0x3c5f, 0x3c5e, 0x3c5e, 0x3c5e],
            [0x3c4c, 0x3c5e, 0x3c5e, 0x3c5e],
            [0x3c4d, 0x3c5e, 0x3c5e, 0x3c5e],
            [0x3c4e, 0x3c5e, 0x3c5e, 0x3c5e],
        ];
        if self.ram.read_u8(MAGIC_CONSUMPTION) >= 1 {
            self.set_tile(hud_xy(2, 0), 0x28f7);
            self.set_tile(hud_xy(3, 0), 0x2851);
            self.set_tile(hud_xy(4, 0), 0x28fa);
        }
        let src = MAGIC[usize::from(self.ram.read_u8(MAGIC_POWER).wrapping_add(7) >> 3)];
        for (i, value) in src.into_iter().enumerate() {
            self.set_tile(hud_xy(3, 1 + i), value);
        }
    }

    fn decimal(mut number: u16) -> [u8; 4] {
        let thousands = number / 1000;
        number %= 1000;
        let hundreds = number / 100;
        number %= 100;
        [
            thousands as u8 + 0x90,
            hundreds as u8 + 0x90,
            (number / 10) as u8 + 0x90,
            (number % 10) as u8 + 0x90,
        ]
    }

    fn update_inventory(&mut self) {
        let rupees = self.ram.read_u16(RUPEES_ACTUAL);
        let d = Self::decimal(rupees);
        const BACKGROUND: [u16; 26] = [
            0x207f, 0x207f, 0x3ca8, 0x207f, 0x207f, 0x2c88, 0x2c89, 0x207f, 0x20a7, 0x20a9, 0x207f,
            0x2871, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f,
            0x207f, 0x207f, 0x207f, 0x207f,
        ];
        let background_offset = usize::from(d[0] == 0x90);
        for x in 0..12 {
            self.set_tile(hud_xy(8 + x, 0), BACKGROUND[background_offset + x]);
            self.set_tile(hud_xy(8 + x, 1), BACKGROUND[13 + background_offset + x]);
        }
        let bow = self.ram.read_u8(LINK_BOW);
        if bow != 0 {
            let arrows = self.ram.read_u8(NUM_ARROWS);
            if bow >= 3 {
                self.set_tile(hud_xy(15, 0), 0x2486);
                self.set_tile(hud_xy(16, 0), 0x2487);
                self.ram.write_u8(LINK_BOW, if arrows != 0 { 4 } else { 3 });
            } else {
                self.ram.write_u8(LINK_BOW, if arrows != 0 { 2 } else { 1 });
            }
        }
        let yellow = self.feature(FEATURE_SHOW_MAX_YELLOW);
        let base = if rupees == self.max_rupees() && yellow {
            0x3400
        } else {
            0x2400
        };
        let base_x = 8 + usize::from(d[0] != 0x90);
        if d[0] != 0x90 {
            self.set_tile(hud_xy(base_x - 1, 1), base | u16::from(d[0]));
        }
        self.set_tile(hud_xy(base_x, 1), base | u16::from(d[1]));
        self.set_tile(hud_xy(base_x + 1, 1), base | u16::from(d[2]));
        self.set_tile(hud_xy(base_x + 2, 1), base | u16::from(d[3]));
        let bombs = self.ram.read_u8(LINK_BOMBS);
        let d = Self::decimal(u16::from(bombs));
        let max = bombs == MAX_BOMBS_FOR_LEVEL[usize::from(self.ram.read_u8(BOMB_UPGRADES))];
        let base = if max && yellow { 0x3400 } else { 0x2400 };
        self.set_tile(hud_xy(base_x + 4, 1), base | u16::from(d[2]));
        self.set_tile(hud_xy(base_x + 5, 1), base | u16::from(d[3]));
        let arrows = self.ram.read_u8(NUM_ARROWS);
        let d = Self::decimal(u16::from(arrows));
        let max = arrows == MAX_ARROWS_FOR_LEVEL[usize::from(self.ram.read_u8(ARROW_UPGRADES))];
        let base = if max && yellow { 0x3400 } else { 0x2400 };
        self.set_tile(hud_xy(base_x + 7, 1), base | u16::from(d[2]));
        self.set_tile(hud_xy(base_x + 8, 1), base | u16::from(d[3]));
        let key = if self.ram.read_u8(NUM_KEYS) == 0xff {
            0x7f
        } else {
            Self::decimal(u16::from(self.ram.read_u8(NUM_KEYS)))[3]
        };
        self.set_tile(hud_xy(base_x + 10, 1), 0x2400 | u16::from(key));
        if key == 0x7f {
            self.set_tile(hud_xy(base_x + 10, 0), 0x247f);
        }
    }

    /// `Hud_UpdateItemBox`: refresh only the equipped-item icon in the live
    /// HUD tilemap. This is intentionally narrower than `rebuild` because the
    /// L/R quick-switch path updates the item box in-place.
    fn update_item_box(&mut self) {
        let item = self.ram.read_u8(HUD_CUR_ITEM);
        if item != 0 {
            let icon = self.icon_for(item);
            let index = hud_xy(5, 1);
            self.set_tile(index, icon[0]);
            self.set_tile(index + 1, icon[1]);
            self.set_tile(index + 32, icon[2]);
            self.set_tile(index + 33, icon[3]);
        }
    }

    pub fn rebuild(&mut self) {
        if self.tile(hud_xy(8, 2)) == 0 {
            for i in 0..165 {
                self.set_tile(i, 0x207f);
            }
        }
        const LEFT: [u16; 48] = [
            0x207f, 0x207f, 0x2850, 0xa856, 0x2852, 0x285b, 0x285b, 0x285c, 0x207f, 0x207f, 0x2854,
            0x2871, 0x2858, 0x207f, 0x207f, 0x285d, 0x207f, 0x207f, 0x2854, 0x304e, 0x2858, 0x207f,
            0x207f, 0x285d, 0x207f, 0x207f, 0x2854, 0x305e, 0x2859, 0xa85b, 0xa85b, 0xa85c, 0x207f,
            0x207f, 0x2854, 0x305e, 0x6854, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0xa850, 0x2856,
            0xe850, 0, 0, 0,
        ];
        for y in 0..6 {
            for x in 0..8 {
                self.set_tile(hud_xy(x, y), LEFT[y * 8 + x]);
            }
        }
        const RIGHT_ROW0: [u16; 12] = [
            0x207f, 0x207f, 0x288b, 0x288f, 0x24ab, 0x24ac, 0x688f, 0x688b, 0x207f, 0x207f, 0x207f,
            0x207f,
        ];
        for y in 0..5 {
            for (x, &top) in RIGHT_ROW0.iter().enumerate() {
                self.set_tile(hud_xy(20 + x, y), if y == 0 { top } else { 0x207f });
            }
        }
        self.update_hearts();
        self.update_magic();
        self.update_inventory();
        self.update_item_box();
        self.increment(FLAG_UPDATE_HUD);
    }

    pub fn handle_item_switch_inputs(&mut self) {
        if !self.feature(FEATURE_SWITCH_LR) {
            return;
        }
        let input = self.ram.read_u8(FILTERED_JOYPAD_L);
        let direction = if input & JOY_L != 0 && self.ram.read_u8(HUD_CUR_ITEM_L) == 0 {
            self.ram.read_u8(HUD_CUR_ITEM_R) != 0
        } else if input & JOY_R != 0 && self.ram.read_u8(HUD_CUR_ITEM_R) == 0 {
            true
        } else {
            return;
        };
        let mut item = self.ram.read_u8(HUD_CUR_ITEM);
        for _ in 0..HUD_ITEM_COUNT {
            if direction {
                self.goto_next(&mut item, true);
            } else {
                self.goto_previous(&mut item, true);
            }
            if self.have_item(item)
                && (!self.feature(FEATURE_SWITCH_LR_LIMIT) || self.current_item_position(item) <= 3)
            {
                if item != self.ram.read_u8(HUD_CUR_ITEM) {
                    self.ram.write_u8(HUD_CUR_ITEM, item);
                    self.ram.write_u8(SOUND_2, 32);
                    self.update_equipped_item();
                    self.update_item_box();
                    self.increment(FLAG_UPDATE_HUD);
                }
                break;
            }
        }
    }

    pub fn reorder_item(&mut self, direction: i32) {
        if self.ram.read_u8(INVENTORY_ORDER) == 0 {
            for i in 0..24 {
                self.ram.write_u8(INVENTORY_ORDER + i, i as u8 + 1);
            }
        }
        let old = self.current_item_position(self.ram.read_u8(HUD_CUR_ITEM)) as usize;
        let new = (old as i32 + direction).rem_euclid(HUD_ITEM_COUNT as i32) as usize;
        let a = self.ram.read_u8(INVENTORY_ORDER + old);
        let b = self.ram.read_u8(INVENTORY_ORDER + new);
        self.ram.write_u8(INVENTORY_ORDER + old, b);
        self.ram.write_u8(INVENTORY_ORDER + new, a);
        self.draw_y_button_items();
        self.ram.write_u8(SOUND_2, 32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[derive(Default)]
    struct TestRuntime(Vec<HudRenderOp>);
    impl HudRuntime for TestRuntime {
        fn render(&mut self, operation: HudRenderOp, _ram: &mut Ram<'_>) {
            self.0.push(operation);
        }
        fn display_select_menu(&mut self, _ram: &mut Ram<'_>) {}
    }

    #[test]
    fn item_selection_skips_missing_items_and_wraps() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = TestRuntime::default();
        ram.write_u8(LINK_ITEMS + 2, 1);
        ram.write_u8(HUD_CUR_ITEM, 1);
        let mut hud = Hud {
            ram: &mut ram,
            runtime: &mut runtime,
        };
        let mut item = 1;
        hud.equip_next(&mut item, true);
        assert_eq!(item, 3);
        hud.equip_previous(&mut item, true);
        assert_eq!(item, 3);
    }

    #[test]
    fn counters_advance_toward_goals_with_original_sfx_cadence() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = TestRuntime::default();
        ram.write_u16(RUPEES_GOAL, 2);
        ram.write_u8(BOMB_FILLER, 1);
        ram.write_u8(ARROW_FILLER, 1);
        let mut hud = Hud {
            ram: &mut ram,
            runtime: &mut runtime,
        };
        hud.refill_logic();
        assert_eq!(hud.ram.read_u16(RUPEES_ACTUAL), 1);
        assert_eq!(hud.ram.read_u8(LINK_BOMBS), 1);
        assert_eq!(hud.ram.read_u8(NUM_ARROWS), 1);
        assert_eq!(hud.ram.read_u8(SOUND_1), 41);
    }

    #[test]
    fn rebuild_writes_representative_magic_inventory_and_heart_tiles() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = TestRuntime::default();
        ram.write_u8(MAGIC_POWER, 8);
        ram.write_u8(HEALTH_CAPACITY, 24);
        ram.write_u8(HEALTH_CURRENT, 12);
        ram.write_u16(RUPEES_ACTUAL, 123);
        ram.write_u8(NUM_KEYS, 0xff);
        let mut hud = Hud {
            ram: &mut ram,
            runtime: &mut runtime,
        };
        hud.rebuild();
        assert_eq!(hud.tile(hud_xy(3, 4)), 0x3c5f);
        assert_eq!(hud.tile(hud_xy(8, 1)) & 0xff, 0x91);
        assert_eq!(hud.tile(hud_xy(18, 1)), 0x247f);
        assert_ne!(hud.tile(hud_xy(20, 1)), 0x207f);
    }

    #[test]
    fn pause_menu_renderers_write_tilemap_without_local_runtime_hooks() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = TestRuntime::default();
        ram.write_u8(LINK_ITEMS, 1);
        ram.write_u8(LINK_GLOVES, 2);
        ram.write_u8(LINK_BOOTS, 1);
        ram.write_u8(ABILITY_FLAGS, 0x88);
        ram.write_u8(PENDANTS, 5);
        ram.write_u8(HUD_CUR_ITEM, 1);
        ram.write_u8(FLASH_TIMER, 0x10);
        let mut hud = Hud {
            ram: &mut ram,
            runtime: &mut runtime,
        };

        hud.draw_y_button_items();
        hud.draw_ability_box();
        hud.draw_progress_icons_pendants();
        hud.draw_equipment_box();
        hud.draw_selected_y_button_item();

        assert_eq!(hud.menu_get(ITEM_POSITIONS[0]), 0x28ba);
        assert_eq!(hud.menu_get(hud_xy(4, 22)), 0x2cf5);
        assert_eq!(hud.menu_get(hud_xy(25, 14)), 0x252b);
        assert_eq!(hud.menu_get(hud_xy(22, 22)), 0x2479);
        assert_eq!(hud.menu_get(hud_xy(25, 6)), 0x28ba);
        assert!(hud.runtime.0.is_empty());
    }

    #[test]
    fn bottle_menu_and_row_animation_match_c_layout() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = TestRuntime::default();
        ram.write_u8(BOTTLES, 3);
        ram.write_u8(BOTTLES + 1, 6);
        ram.write_u8(LINK_BOTTLE_INDEX, 2);
        ram.write_u8(FLASH_TIMER, 0x10);
        ram.write_u8(BOTTLE_EXPAND_ROW, 17);
        let mut hud = Hud {
            ram: &mut ram,
            runtime: &mut runtime,
        };

        hud.draw_bottle_menu();
        assert_eq!(hud.menu_get(hud_xy(25, 13)), 0x24d2);
        assert_eq!(hud.menu_get(hud_xy(25, 17)), 0x2855);
        assert_eq!(hud.menu_get(ITEM_POSITIONS[15]), 0x2855);
        hud.expand_bottle_menu();
        assert_eq!(hud.menu_get(hud_xy(21, 28)), 0x28fb);
        assert_eq!(hud.menu_get(hud_xy(21, 29)), 0xa8fb);
        assert_eq!(hud.ram.read_u8(BOTTLE_EXPAND_ROW), 16);
        assert!(hud.runtime.0.is_empty());
    }

    #[test]
    fn quick_switch_refreshes_only_the_live_item_box() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = TestRuntime::default();
        ram.slice_mut(FEATURES0, 4)
            .copy_from_slice(&FEATURE_SWITCH_LR.to_le_bytes());
        ram.write_u8(HUD_CUR_ITEM, 1);
        ram.write_u8(LINK_ITEMS, 1);
        ram.write_u8(LINK_ITEMS + 2, 1);
        ram.write_u8(FILTERED_JOYPAD_L, JOY_R);
        let mut hud = Hud {
            ram: &mut ram,
            runtime: &mut runtime,
        };

        hud.handle_item_switch_inputs();

        assert_eq!(hud.ram.read_u8(HUD_CUR_ITEM), 3);
        assert_eq!(hud.ram.read_u8(CURRENT_ITEM_Y), 14);
        assert_eq!(hud.tile(hud_xy(6, 1)), 0x24f6);
        assert_eq!(hud.ram.read_u8(SOUND_2), 32);
        assert_eq!(hud.ram.read_u8(FLAG_UPDATE_HUD), 1);
    }

    #[test]
    fn raw_item_box_lookup_covers_message_only_equipment_entries() {
        assert_eq!(
            Hud::<TestRuntime>::item_box_icon(25, 4),
            [0x288a, 0x2865, 0x2c74, 0x2d39]
        );
        assert_eq!(
            Hud::<TestRuntime>::item_box_icon(26, 3),
            [0x2880, 0x2881, 0x288d, 0x288e]
        );
        assert_eq!(
            Hud::<TestRuntime>::item_box_icon(28, 2),
            [0x2837, 0x2838, 0x2cc3, 0x2cd3]
        );
    }
}
