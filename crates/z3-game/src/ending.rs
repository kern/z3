//! Opening, Triforce-room, ending, and credits state machines.
//!
//! This is a mechanical port of `src/ending.c`.  WRAM addresses deliberately
//! retain the aliases used by the original program and every arithmetic
//! operation that was byte/word sized in C is explicitly wrapping here.

use crate::{oam, ram::Ram};

const MAIN_MODULE: usize = 0x10;
const SUBMODULE: usize = 0x11;
const INIDISP: usize = 0x13;
const NMI_LOAD_BG: usize = 0x14;
const UPDATE_CGRAM: usize = 0x15;
const FRAME_COUNTER: usize = 0x1a;
const TM: usize = 0x1c;
const TS: usize = 0x1d;
const LINK_Y_VEL: usize = 0x30;
const LINK_X_VEL: usize = 0x31;
const OVERLAY_INDEX: usize = 0x8c;
const BGMODE: usize = 0x94;
const MOSAIC: usize = 0x95;
const CGWSEL: usize = 0x99;
const CGADSUB: usize = 0x9a;
const HDMAEN: usize = 0x9b;
const COLDATA0: usize = 0x9c;
const DUNGEON_ROOM_INDEX: usize = 0xa0;
const SUBSUBMODULE: usize = 0xb0;
const R16: usize = 0xc8;
const R18: usize = 0xca;
const INTRO_SWORD_20: usize = 0xcc;
const INTRO_SWORD_21: usize = 0xcd;
const INTRO_SWORD_24: usize = 0xd0;
const BG1_HOFS2: usize = 0xe0;
const BG2_HOFS2: usize = 0xe2;
const BG3_HOFS2: usize = 0xe4;
const BG1_VOFS2: usize = 0xe6;
const BG2_VOFS2: usize = 0xe8;
const BG3_VOFS2: usize = 0xea;
const FILTERED_JOYPAD_H: usize = 0xf4;
const FILTERED_JOYPAD_L: usize = 0xf6;
const VIRQ_TRIGGER: usize = 0xff;
const LINK_DMA_GRAPHICS_INDEX: usize = 0x100;
const BG2_HOFS: usize = 0x11e;
const BG1_HOFS: usize = 0x120;
const BG2_VOFS: usize = 0x122;
const BG1_VOFS: usize = 0x124;
const IRQ_FLAG: usize = 0x128;
const NMI_THREAD_ACTIVE: usize = 0x12a;
const MUSIC_CONTROL: usize = 0x12c;
const SOUND_EFFECT_1: usize = 0x12e;
const SOUND_EFFECT_2: usize = 0x12f;
const OVERWORLD_AREA_INDEX: usize = 0x40a;
const OW_TRANS_DIR: usize = 0x416;
const DUNGEON_ROOM_INDEX2: usize = 0x48e;
const BG1_HOFS_SUB: usize = 0x620;
const BG1_VOFS_SUB: usize = 0x622;
const OW_UNK1: usize = 0x624;
const OW_UNK1_NEG: usize = 0x626;
const OW_UNK3: usize = 0x628;
const OW_UNK3_NEG: usize = 0x62a;
const NMI_DISABLE_CORE: usize = 0x710;
const SPRITE_Y_LO: usize = 0xd00;
const SPRITE_X_LO: usize = 0xd10;
const SPRITE_Y_HI: usize = 0xd20;
const SPRITE_X_HI: usize = 0xd30;
const SPRITE_Y_VEL: usize = 0xd40;
const SPRITE_X_VEL: usize = 0xd50;
const SPRITE_AI_STATE: usize = 0xd80;
const SPRITE_A: usize = 0xd90;
const SPRITE_B: usize = 0xda0;
const SPRITE_C: usize = 0xdb0;
const SPRITE_GRAPHICS: usize = 0xdc0;
const SPRITE_STATE: usize = 0xdd0;
const SPRITE_D: usize = 0xde0;
const SPRITE_DELAY_MAIN: usize = 0xdf0;
const SPRITE_DELAY_AUX1: usize = 0xe00;
const SPRITE_TYPE: usize = 0xe20;
const SPRITE_FLAGS2: usize = 0xe40;
const SPRITE_FLAGS3: usize = 0xe60;
const SPRITE_E: usize = 0xe90;
const SPRITE_HEAD_DIR: usize = 0xeb0;
const SPRITE_PAUSE: usize = 0xf00;
const SPRITE_OAM_FLAGS: usize = 0xf50;
const SPRITE_Z: usize = 0xf70;
const SPRITE_FLAGS5: usize = 0xbe0;
const SPRITE_DEFL_BITS: usize = 0xcaa;
const NMI_FLAG_UPDATE_POLY: usize = 0x1f0c;
const VRAM_UPLOAD_OFFSET: usize = 0x1000;
const VRAM_UPLOAD_DATA: usize = 0x1002;
const INTRO_STEP_INDEX: usize = 0x1e00;
const INTRO_STEP_TIMER: usize = 0x1e01;
const INTRO_WANT_DOUBLE_RET: usize = 0x1e02;
const INTRO_SPRITE_ALLOC: usize = 0x1e08;
const INTRO_FRAME_CTR: usize = 0x1e0a;
const TRIFORCE_CTR: usize = 0x1e0c;
const INTRO_INITED: usize = 0x1e10;
const INTRO_SUBTYPE: usize = 0x1e18;
const INTRO_STATE: usize = 0x1e20;
const INTRO_X_SUB: usize = 0x1e28;
const INTRO_X_LO: usize = 0x1e30;
const INTRO_X_HI: usize = 0x1e38;
const INTRO_Y_SUB: usize = 0x1e40;
const INTRO_Y_LO: usize = 0x1e48;
const INTRO_Y_HI: usize = 0x1e50;
const INTRO_X_VEL: usize = 0x1e58;
const INTRO_Y_VEL: usize = 0x1e60;
const INTRO_DID_RUN_STEP: usize = 0x1f00;
const POLY_COLOR_MODE: usize = 0x1f01;
const POLY_CONFIG1: usize = 0x1f02;
const POLY_MODEL: usize = 0x1f03;
const POLY_A: usize = 0x1f04;
const POLY_B: usize = 0x1f05;
const POLY_BASE_X: usize = 0x1f06;
const POLY_BASE_Y: usize = 0x1f07;
const POLY_VAR1: usize = 0x1f08;
const THREAD_OTHER_STACK: usize = 0x1f0a;
const PALETTE_FILTER_COUNTDOWN: usize = 0xc007;
const DARKENING: usize = 0xc009;
const MOSAIC_TARGET: usize = 0xc00b;
const AUX_PALETTE: usize = 0xc300;
const MAIN_PALETTE: usize = 0xc500;
const INTRO_FLASHES: usize = 0xff9;

pub const POLYHEDRAL_PALETTE: [u16; 8] = [0, 0x14d, 0x1b0, 0x1f3, 0x256, 0x279, 0x2fd, 0x35f];
pub const ENDING_TAB1: [u16; 16] = [
    0x1000, 2, 0x1002, 0x1012, 0x1004, 0x1006, 0x1010, 0x1014, 0x100a, 0x1016, 0x5d, 0x64, 0x100e,
    0x1008, 0x1018, 0x180,
];
pub const ENDING_SPRITE_PACK: [u8; 17] = [
    0x28, 0x46, 0x27, 0x2e, 0x2b, 0x2b, 0x0e, 0x2c, 0x1a, 0x29, 0x47, 0x28, 0x27, 0x28, 0x2a, 0x28,
    0x2d,
];
pub const ENDING_SPRITE_PAL: [u8; 17] = [
    1, 0x40, 1, 4, 1, 1, 1, 0x11, 1, 1, 0x47, 0x40, 1, 1, 1, 1, 1,
];
pub const ENDING_TARGET_Y: [u16; 16] = [
    0x6f2, 0x210, 0x72c, 0xc00, 0x10c, 0xa9b, 0x10, 0x510, 0x89, 0xa8e, 0x222c, 0x2510, 0x826,
    0x5c, 0x20a, 0x30,
];
pub const ENDING_TARGET_X: [u16; 16] = [
    0x77f, 0x480, 0x193, 0xaa, 0x878, 0x847, 0x4fd, 0xc57, 0x40f, 0x478, 0xa00, 0x200, 0x201,
    0xaa1, 0x26f, 0,
];
pub const ENDING_Y_VEL: [i8; 16] = [-1, -1, 1, -1, 1, 1, 0, 1, 0, -1, -1, 0, 0, 0, 1, -1];
pub const ENDING_X_VEL: [i8; 16] = [0, 0, -1, 0, 0, -1, 1, 0, -1, 0, 0, 0, 1, -1, 1, 0];
pub const ENDING_SPRITES_IDX: [u8; 17] = [
    0, 12, 14, 21, 28, 31, 35, 38, 40, 41, 52, 58, 64, 71, 72, 79, 85,
];
pub const ENDING_SPRITES_X: [u16; 85] = [
    0x1e0, 0x200, 0x1ed, 0x203, 0x1da, 0x216, 0x1c8, 0x228, 0x1c0, 0x1e0, 0x208, 0x228, 0xf8, 0xf0,
    0x278, 0x298, 0x1e0, 0x200, 0x220, 0x288, 0x1e2, 0xe0, 0x150, 0xe8, 0x168, 0x128, 0x170, 0x170,
    0x335, 0x335, 0x300, 0xb8, 0xce, 0xac, 0xc4, 0x3b0, 0x390, 0x3d0, 0xf8, 0xc8, 0x80, 0xf8, 0xf8,
    0xf8, 0xf8, 0xf8, 0xe8, 0xf8, 0xd8, 0xf8, 0xc8, 0x108, 0x70, 0x70, 0x70, 0x68, 0x88, 0x70,
    0x40, 0x70, 0x4f, 0x61, 0x37, 0x79, 0xc8, 0x278, 0x258, 0x1d8, 0x1c8, 0x188, 0x270, 0x180,
    0x2e8, 0x270, 0x270, 0x2a0, 0x2a0, 0x2a4, 0x2fc, 0x76, 0x73, 0x76, 0, 0xd0, 0x80,
];
pub const ENDING_SPRITES_Y: [u16; 85] = [
    0x158, 0x158, 0x138, 0x138, 0x140, 0x140, 0x150, 0x150, 0x120, 0x120, 0x120, 0x120, 0x60, 0x37,
    0xc2, 0xc2, 0x16b, 0x16c, 0x16b, 0xb8, 0x16b, 0x80, 0x60, 0x146, 0x146, 0x1c6, 0x70, 0x70,
    0x128, 0x128, 0x16f, 0xf5, 0xfc, 0x10d, 0x10d, 0x40, 0x40, 0x40, 0x150, 0x158, 0xf4, 0x120,
    0x120, 0x120, 0x120, 0x120, 0x108, 0x100, 0xd8, 0xd8, 0xf0, 0xf0, 0x3c, 0x3c, 0x3c, 0x90, 0x80,
    0x3c, 0x16c, 0x16c, 0x174, 0x174, 0x175, 0x175, 0x250, 0x2b0, 0x2b0, 0x2a0, 0x2b0, 0x2b0,
    0x2b8, 0xd8, 0x24b, 0x1b0, 0x1c8, 0x1c8, 0x1b0, 0x230, 0x230, 0x8b, 0x83, 0x85, 0x2c, 0xf8,
    0x100,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntroSpriteEnt {
    pub x: i8,
    pub y: i8,
    pub charnum: u8,
    pub flags: u8,
    pub ext: u8,
}

/// Only calls whose implementations live in another C translation unit.
pub trait EndingRuntime {
    fn call(&mut self, _name: &'static str, _ram: &mut Ram<'_>) {}
    fn ppu_write(&mut self, _register: u16, _value: u8) {}
    fn sprite_reset(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_set_x(&mut self, ram: &mut Ram<'_>, k: usize, x: u16) {
        ram.write_u8(SPRITE_X_LO + k, x as u8);
        ram.write_u8(SPRITE_X_HI + k, (x >> 8) as u8);
    }
    fn sprite_set_y(&mut self, ram: &mut Ram<'_>, k: usize, y: u16) {
        ram.write_u8(SPRITE_Y_LO + k, y as u8);
        ram.write_u8(SPRITE_Y_HI + k, (y >> 8) as u8);
    }
    fn sprite_move_xy(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_active_main(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_draw_shadow(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn oam_allocate_region_a(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn sprite_get_16_bit_coords(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_draw_multiple(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _entries: &[DrawMultipleData],
    ) {
    }
    fn ending_map_data(&self) -> &[u16] {
        &[]
    }
    fn ending_credits_text(&self) -> &[u8] {
        &[]
    }
    fn ending_credits_offsets(&self) -> &[u16] {
        &[]
    }
    fn ending_scene_text(&self) -> &[u8] {
        &[]
    }
    fn ending_scene_offsets(&self) -> &[u16] {
        &[]
    }
    fn random_number(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
    fn generated_end_sequence_15(&self, _frame: u8) -> u8 {
        0
    }
    fn decompress_animated_overworld_tiles(&mut self, _ram: &mut Ram<'_>, _set: u8) {}
    fn get_overworld_bg_palette(&mut self, _screen: u8) -> u8 {
        0
    }
    fn overworld_load_palettes(&mut self, _ram: &mut Ram<'_>, _bg: u8, _sprites: u8) {}
    fn decompress_animated_dungeon_tiles(&mut self, _ram: &mut Ram<'_>, _set: u8) {}
    fn dungeon_palette_info(&mut self, _index: u8) -> (u8, u8) {
        (0, 0)
    }
    fn health_after_death(&self, _capacity_index: usize) -> u8 {
        0
    }
    fn save_game_file(&mut self, _ram: &mut Ram<'_>) {}
    fn credits_hdma_setup(&mut self, _ram: &mut Ram<'_>) {}
    fn vram_write_word(&mut self, _word_address: usize, _value: u16) {}
    fn dungeon_animated_tile_for_theme(&self, _theme: u8) -> u8 {
        0
    }
    fn overworld_load_area_palettes_ex(&mut self, _ram: &mut Ram<'_>, _index: u8) {}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawMultipleData {
    pub x: i8,
    pub y: i8,
    pub tile: u16,
    pub ext: u8,
}

pub struct Ending<'a, 'r, R: EndingRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
}

impl<R: EndingRuntime> Ending<'_, '_, R> {
    fn inc8(&mut self, a: usize) -> u8 {
        let v = self.ram.read_u8(a).wrapping_add(1);
        self.ram.write_u8(a, v);
        v
    }
    fn dec8(&mut self, a: usize) -> u8 {
        let v = self.ram.read_u8(a).wrapping_sub(1);
        self.ram.write_u8(a, v);
        v
    }
    fn add8(&mut self, a: usize, v: u8) {
        self.ram.write_u8(a, self.ram.read_u8(a).wrapping_add(v));
    }
    fn copy_scroll(&mut self) {
        for (a, b) in [
            (BG2_HOFS, BG2_HOFS2),
            (BG2_VOFS, BG2_VOFS2),
            (BG1_HOFS, BG1_HOFS2),
            (BG1_VOFS, BG1_VOFS2),
        ] {
            self.ram.write_u16(a, self.ram.read_u16(b));
        }
    }
    fn joy_skip(&self) -> bool {
        (((self.ram.read_u8(FILTERED_JOYPAD_L) & 0xc0) | self.ram.read_u8(FILTERED_JOYPAD_H))
            & 0xd0)
            != 0
    }
    fn external(&mut self, n: &'static str) {
        self.runtime.call(n, self.ram);
    }

    pub fn credits_load_scene_overworld_prep_gfx(&mut self) {
        self.external("EnableForceBlank");
        self.external("EraseTileMaps_normal");
        self.ram.write_u8(CGWSEL, 0x82);
        let k = (self.ram.read_u8(SUBMODULE) >> 1) as usize;
        self.ram.write_u16(DUNGEON_ROOM_INDEX, ENDING_TAB1[k]);
        self.external(if k == 6 || k == 15 {
            "Overworld_EnterSpecialArea"
        } else {
            "LoadOverworldFromDungeon"
        });
        self.ram.write_u8(MUSIC_CONTROL, 0);
        self.ram.write_u8(0x12d, 0);
        let screen = self.ram.read_u8(0x8a) & !0x40;
        self.runtime.decompress_animated_overworld_tiles(
            self.ram,
            if screen == 3 || screen == 5 || screen == 7 {
                0x58
            } else {
                0x5a
            },
        );
        self.ram.write_u8(0xaa3, ENDING_SPRITE_PACK[k]);
        self.external("InitializeTilesets");
        self.external("OverworldLoadScreensPaletteSet");
        let bg = self
            .runtime
            .get_overworld_bg_palette(self.ram.read_u8(0x8a));
        self.runtime
            .overworld_load_palettes(self.ram, bg, ENDING_SPRITE_PAL[k]);
        self.ram.write_u8(0xab2, 1);
        self.external("Palette_Load_HUD");
        if self.ram.read_u8(SUBMODULE) == 0 {
            self.external("TransferFontToVRAM")
        }
        self.external("Overworld_LoadPalettesInner");
        self.external("Overworld_SetFixedColAndScroll");
        if self.ram.read_u8(0x8a) >= 128 {
            self.external("Palette_SetOwBgColor")
        }
        self.ram.write_u8(BGMODE, 9);
        self.inc8(SUBSUBMODULE);
    }

    pub fn credits_load_scene_overworld_overlay(&mut self) {
        self.external("Overworld_LoadOverlays2");
        self.ram.write_u8(MUSIC_CONTROL, 0);
        self.dec8(SUBMODULE);
        self.inc8(SUBSUBMODULE);
    }

    pub fn credits_load_scene_overworld_load_map(&mut self) {
        self.external("Overworld_LoadAndBuildScreen");
        self.credits_prep_and_load_sprites();
        self.ram.write_u16(R16, 0);
        self.ram.write_u8(SUBSUBMODULE, 0);
    }

    pub fn credits_load_cool_background(&mut self) {
        self.ram.write_u8(0xaa1, 33);
        self.ram.write_u8(0xaa2, 59);
        self.ram.write_u8(0xaa3, 45);
        self.external("InitializeTilesets");
        self.ram.write_u8(0x8a, 0x5b);
        let bg = self.runtime.get_overworld_bg_palette(0x5b);
        self.runtime.overworld_load_palettes(self.ram, bg, 0x13);
        self.ram.write_u8(0xab5, 3);
        self.external("Palette_Load_OWBG2");
        self.external("Overworld_CopyPalettesToCache");
        self.external("Overworld_LoadOverlays2");
        self.ram.write_u16(BG1_VOFS2, 0);
        self.ram.write_u16(BG1_HOFS2, 0);
        self.dec8(SUBMODULE);
    }

    pub fn credits_load_scene_dungeon(&mut self) {
        self.external("EnableForceBlank");
        self.external("EraseTileMaps_normal");
        let k = (self.ram.read_u8(SUBMODULE) >> 1) as usize;
        self.ram.write_u16(0x106, ENDING_TAB1[k]);
        self.external("Dungeon_LoadEntrance");
        self.ram.write_u8(0x45a, 0);
        self.ram.write_u8(0x458, 0);
        self.external("Dungeon_LoadAndDrawRoom");
        let anim = self
            .runtime
            .dungeon_animated_tile_for_theme(self.ram.read_u8(0xaa1));
        self.runtime
            .decompress_animated_dungeon_tiles(self.ram, anim);
        self.ram.write_u8(0xaa3, ENDING_SPRITE_PACK[k]);
        let (p5, p6) = self
            .runtime
            .dungeon_palette_info(ENDING_SPRITE_PAL[k] & 0x3f);
        self.ram.write_u8(0xaad, p5);
        self.ram.write_u8(0xaae, p6);
        self.ram.write_u8(0xaa4, 10);
        self.external("InitializeTilesets");
        self.ram.write_u8(0xab1, 10);
        self.external("Dungeon_LoadPalettes");
        self.ram.write_u8(BGMODE, 9);
        self.ram.write_u16(R16, 0);
        self.ram.write_u8(INIDISP, 0);
        self.inc8(SUBMODULE);
        self.credits_prep_and_load_sprites();
    }

    pub fn credits_load_next_scene_overworld(&mut self) {
        match self.ram.read_u8(SUBSUBMODULE) {
            0 => self.credits_load_scene_overworld_prep_gfx(),
            1 => self.credits_load_scene_overworld_overlay(),
            2 => self.credits_load_scene_overworld_load_map(),
            _ => {}
        }
        self.credits_add_ending_sequence_text();
    }

    pub fn credits_load_next_scene_dungeon(&mut self) {
        self.credits_load_scene_dungeon();
        self.credits_add_ending_sequence_text();
    }

    pub fn module1a_credits(&mut self) {
        self.ram.write_u16(0xfe0, 0x30);
        self.ram.write_u16(0xfe2, 0x1d0);
        self.ram.write_u16(0xfe4, 0);
        match self.ram.read_u8(SUBMODULE) {
            0 | 4 | 6 | 8 | 10 | 12 | 14 | 16 | 18 | 24 | 26 | 28 | 30 => {
                self.credits_load_next_scene_overworld()
            }
            2 | 20 | 22 => self.credits_load_next_scene_dungeon(),
            1 | 5 | 7 | 9 | 11 | 13 | 15 | 17 | 19 | 25 | 27 | 29 | 31 => {
                self.credits_scroll_scene_overworld()
            }
            3 | 21 | 23 => self.credits_scroll_scene_dungeon(),
            32 => self.end_sequence_32(),
            33 => self.credits_brighten_triangles(),
            34 => self.credits_fade_color_and_begin_animating(),
            35 => self.credits_stop_credits_scroll(),
            36 => self.credits_fade_and_disperse_triangles(),
            37 => self.credits_fade_in_the_end(),
            38 => self.credits_hang_forever(),
            _ => {}
        }
    }

    pub fn module18_ganon_emerges(&mut self) {
        let saved = [
            self.ram.read_u16(BG2_HOFS2),
            self.ram.read_u16(BG2_VOFS2),
            self.ram.read_u16(BG1_HOFS2),
            self.ram.read_u16(BG1_VOFS2),
        ];
        let xo = self.ram.read_u16(0x11a);
        let yo = self.ram.read_u16(0x11c);
        self.ram.write_u16(BG2_HOFS2, saved[0].wrapping_add(xo));
        self.ram.write_u16(BG2_HOFS, self.ram.read_u16(BG2_HOFS2));
        self.ram.write_u16(BG2_VOFS2, saved[1].wrapping_add(yo));
        self.ram.write_u16(BG2_VOFS, self.ram.read_u16(BG2_VOFS2));
        self.ram.write_u16(BG1_HOFS2, saved[2].wrapping_add(xo));
        self.ram.write_u16(BG1_HOFS, self.ram.read_u16(BG1_HOFS2));
        self.ram.write_u16(BG1_VOFS2, saved[3].wrapping_add(yo));
        self.ram.write_u16(BG1_VOFS, self.ram.read_u16(BG1_VOFS2));
        self.external("Sprite_Main");
        for (a, v) in [
            (BG2_HOFS2, saved[0]),
            (BG2_VOFS2, saved[1]),
            (BG1_HOFS2, saved[2]),
            (BG1_VOFS2, saved[3]),
        ] {
            self.ram.write_u16(a, v)
        }
        match self.ram.read_u8(0x200) {
            0 => {
                self.external("Dungeon_HandleLayerEffect");
                self.external("CallForDuckIndoors");
                self.external("SaveDungeonKeys");
                self.inc8(0x200);
                self.inc8(0x2f);
            }
            1 => {
                self.external("Dungeon_HandleLayerEffect");
                if self.ram.read_u8(SUBMODULE) == 10 {
                    self.ram.write_u16(0x8a, 91);
                    self.ram.write_u8(0x1b, 0);
                    self.ram.write_u8(MAIN_MODULE, 24);
                    self.ram.write_u8(SUBMODULE, 0);
                    self.ram.write_u8(0x200, 2)
                }
            }
            2 => {
                self.external("Dungeon_HandleLayerEffect");
                if self.dec8(INIDISP) == 0 {
                    self.external("EnableForceBlank");
                    self.inc8(0x200);
                    self.external("Hud_RebuildIndoor");
                    self.ram.write_u8(LINK_X_VEL, 0);
                    self.ram.write_u8(LINK_Y_VEL, 0)
                }
            }
            3 => {
                self.ram.write_u8(0x1af0, 8);
                self.ram.write_u8(0x1af1, 0);
                self.external("FluteMenu_LoadSelectedScreen");
                self.external("LoadOWMusicIfNeeded");
                self.ram.write_u8(MUSIC_CONTROL, 9)
            }
            4 => {
                self.external("Overworld_LoadOverlayAndMap");
                self.ram.write_u8(SUBSUBMODULE, 0)
            }
            5 => {
                if self.inc8(INIDISP) == 15 {
                    self.ram.write_u16(0x402, 0);
                    self.ram.write_u8(0xfc1, 0);
                    self.external("Sprite_SpawnBatCrashCutscene");
                    self.ram.write_u8(0x2f, 2);
                    self.ram.write_u8(0x10c, 9);
                    self.ram.write_u8(0x1b, 0);
                    self.inc8(0x200);
                    self.ram.write_u8(SUBSUBMODULE, 128);
                    self.ram.write_u8(0x40c, 255)
                }
            }
            7 => {
                if self.dec8(SUBSUBMODULE) == 0 {
                    self.inc8(0x200);
                }
            }
            8 => self.external("BirdTravel_Finish_Doit"),
            _ => {}
        }
        self.external("LinkOam_Main");
    }

    pub fn module19_triforce_room(&mut self) {
        match self.ram.read_u8(SUBSUBMODULE) {
            0 => {
                self.external("Link_ResetProperties_A");
                self.ram.write_u8(0x66, 0);
                self.ram.write_u8(MUSIC_CONTROL, 0xf1);
                self.external("ResetTransitionPropsAndAdvance_ResetInterface")
            }
            1 => {
                self.external("ConditionalMosaicControl");
                self.external("ApplyPaletteFilter_bounce")
            }
            2 => {
                self.external("EnableForceBlank");
                self.external("LoadCreditsSongs");
                self.ram.write_u16(DUNGEON_ROOM_INDEX, 0x189);
                self.external("EraseTileMaps_normal");
                self.external("Palette_RevertTranslucencySwap");
                self.external("Overworld_EnterSpecialArea");
                self.external("Overworld_LoadOverlays2");
                self.inc8(SUBSUBMODULE);
                self.ram.write_u8(MAIN_MODULE, 25);
                self.ram.write_u8(SUBMODULE, 0)
            }
            3 => {
                self.ram.write_u8(0xaa1, 36);
                self.ram.write_u8(0xaa3, 125);
                self.ram.write_u8(0xaa2, 81);
                self.external("InitializeTilesets");
                self.runtime.overworld_load_area_palettes_ex(self.ram, 4);
                self.runtime.overworld_load_palettes(self.ram, 14, 0);
                self.external("SpecialOverworld_CopyPalettesToCache");
                self.inc8(SUBSUBMODULE);
            }
            4 => {
                let s = self.ram.read_u8(SUBSUBMODULE);
                self.external("Module08_02_LoadAndAdvance");
                self.ram.write_u8(SUBSUBMODULE, s + 1);
                self.ram.write_u8(INIDISP, 15);
                self.ram.write_u16(PALETTE_FILTER_COUNTDOWN, 31);
                self.ram.write_u8(MOSAIC_TARGET, 0);
                self.ram.write_u8(CGWSEL, 2);
                self.ram.write_u8(CGADSUB, 50);
                self.ram.write_u8(MOSAIC, 240);
                self.ram.write_u8(BG1_HOFS2 + 1, 1);
                self.ram.write_u8(0x20, 236);
                self.ram.write_u8(0x22, 120);
                self.ram.write_u8(0xee, 2);
                self.ram.write_u8(MUSIC_CONTROL, 32);
                self.ram.write_u8(MAIN_MODULE, 25);
                self.ram.write_u8(SUBMODULE, 0)
            }
            5 => {
                self.ram.write_u8(0x67, 8);
                self.ram.write_u8(0x26, 8);
                self.ram.write_u8(0x2f, 0);
                if self.ram.read_u8(0x20) < 192 {
                    self.ram.write_u8(0x67, 0);
                    self.ram.write_u8(0x26, 0);
                    self.ram.write_u8(0x2e, 0);
                    self.inc8(SUBSUBMODULE);
                }
            }
            6 => {
                if self.ram.read_u8(PALETTE_FILTER_COUNTDOWN) & 1 == 0
                    && self.ram.read_u8(MOSAIC) != 0
                {
                    self.add8(MOSAIC, 0xf0)
                }
                self.ram.write_u8(BGMODE, 9);
                self.external("ApplyPaletteFilter_bounce")
            }
            7 => {
                self.triforce_room_prep_gfx_slot_for_poly();
                self.ram.write_u16(0x1cf0, 0x173);
                self.external("Main_ShowTextMessage");
                self.external("RenderText");
                self.ram.write_u8(R16, 0x80);
                self.ram.write_u8(MAIN_MODULE, 25);
                self.inc8(SUBSUBMODULE);
            }
            8 | 10 => {
                self.advance_polyhedral();
                if self.ram.read_u8(SUBSUBMODULE) == 11 {
                    self.ram.write_u8(MUSIC_CONTROL, 33);
                    self.ram.write_u8(MAIN_MODULE, 25);
                    self.ram.write_u8(0x67, 0);
                    self.ram.write_u8(0x26, 0);
                    self.inc8(SUBMODULE);
                }
            }
            9 => {
                self.advance_polyhedral();
                self.external("RenderText");
                if self.ram.read_u8(SUBMODULE) == 0 {
                    self.ram.write_u8(0x200, 0);
                    self.ram.write_u8(MAIN_MODULE, 25);
                    self.inc8(SUBSUBMODULE);
                }
            }
            11 => {
                self.advance_polyhedral();
                self.external("TriforceRoom_LinkApproachTriforce");
                if self.ram.read_u8(SUBSUBMODULE) == 12 {
                    self.ram.write_u8(0x67, 0);
                    self.ram.write_u8(0x26, 0)
                }
            }
            12 => {
                self.advance_polyhedral();
                if self.dec8(R16) == 0 {
                    self.external("Palette_AnimGetMasterSword2");
                    self.inc8(SUBMODULE);
                }
            }
            13 => {
                self.advance_polyhedral();
                self.external("PaletteFilter_BlindingWhiteTriforce");
                if self.ram.read_u8(DARKENING) == 255 {
                    self.inc8(SUBSUBMODULE);
                }
            }
            14 => {
                if self.dec8(INIDISP) == 0 {
                    self.ram.write_u8(MAIN_MODULE, 26);
                    self.ram.write_u8(SUBMODULE, 0);
                    self.ram.write_u8(SUBSUBMODULE, 0);
                    self.ram.write_u8(IRQ_FLAG, 255);
                    self.ram.write_u8(NMI_THREAD_ACTIVE, 0);
                    self.ram.write_u8(NMI_FLAG_UPDATE_POLY, 0);
                    self.ram.write_u8(0xf3ca, 0)
                }
            }
            _ => {}
        }
        self.copy_scroll();
        let s = self.ram.read_u8(SUBSUBMODULE);
        if !(7..11).contains(&s) {
            self.external("Link_HandleVelocity");
            self.external("Link_HandleMovingAnimation_FullLongEntry")
        }
        self.external("LinkOam_Main");
    }

    pub fn intro_initialize_background_settings(&mut self) {
        self.ram.write_u8(BGMODE, 9);
        self.ram.write_u8(MOSAIC, 0);
        self.runtime.ppu_write(0x07, 0x13);
        self.runtime.ppu_write(0x08, 3);
        self.runtime.ppu_write(0x09, 0x63);
        self.ram.write_u8(CGADSUB, 32);
        self.ram.write_u8(COLDATA0, 32);
        self.ram.write_u8(COLDATA0 + 1, 64);
        self.ram.write_u8(COLDATA0 + 2, 128);
    }
    pub fn polyhedral_initialize_thread(&mut self) {
        self.ram.slice_mut(0x1f00, 256).fill(0);
        self.ram.write_u16(THREAD_OTHER_STACK, 0x1f31);
        self.ram
            .slice_mut(0x1f32, 13)
            .copy_from_slice(&[9, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0x30, 0x1d, 0xf8, 9]);
    }
    pub fn load_triforce_sprite_palette(&mut self) {
        for (i, &v) in POLYHEDRAL_PALETTE.iter().enumerate() {
            self.ram.write_u16(MAIN_PALETTE + 0x1a0 + i * 2, v)
        }
        self.inc8(UPDATE_CGRAM);
    }
    pub fn intro_init_gfx_helper(&mut self) {
        self.polyhedral_initialize_thread();
        self.load_triforce_sprite_palette();
        self.ram.write_u8(VIRQ_TRIGGER, 0x90);
        self.ram.write_u8(POLY_CONFIG1, 255);
        self.ram.write_u8(POLY_BASE_X, 32);
        self.ram.write_u8(POLY_BASE_Y, 32);
        self.ram.write_u8(POLY_VAR1, 32);
        self.ram.write_u8(POLY_A, 0xa0);
        self.ram.write_u8(POLY_B, 0x60);
        self.ram.write_u8(POLY_COLOR_MODE, 1);
        self.ram.write_u8(POLY_MODEL, 1);
        self.ram.write_u8(NMI_THREAD_ACTIVE, 1);
        self.ram.write_u8(INTRO_DID_RUN_STEP, 1);
        self.ram.slice_mut(INTRO_STEP_INDEX, 112).fill(0);
    }
    pub fn intro_setup_screen(&mut self) {
        self.ram.write_u8(NMI_DISABLE_CORE, 0x80);
        self.external("EnableForceBlank");
        self.ram.write_u8(TM, 16);
        self.ram.write_u8(TS, 0);
        self.intro_initialize_background_settings();
        self.ram.write_u8(CGWSEL, 0x20);
        self.ram.write_u8(0xaaa, 20);
        self.external("Graphics_LoadChrHalfSlot");
        self.ram.write_u8(0xaaa, 0);
        self.external("LoadOWMusicIfNeeded");
        for i in 0..17 {
            self.ram.write_u16(MAIN_PALETTE + (144 + i) * 2, 0x7fff);
        }
        for i in 0..17 {
            self.runtime.vram_write_word(0x27f0 + i, 0);
        }
        self.ram.write_u16(R16, 0x1ffe);
        self.ram.write_u16(R18, 0x1bfe);
    }
    pub fn intro_load_text_pointers_and_palettes(&mut self) {
        self.external("Text_GenerateMessagePointers");
        self.external("Overworld_LoadAllPalettes");
    }
    pub fn module00_intro(&mut self, enhanced_features0: u32) {
        let skip = if enhanced_features0 & 128 != 0 { 4 } else { 8 };
        let sm = self.ram.read_u8(SUBMODULE);
        if sm >= skip && self.joy_skip() {
            self.fade_music_and_reset_sram_mirror();
            return;
        }
        match sm {
            0 => self.intro_init(),
            1 => self.intro_init_continue(),
            2 | 10 => self.intro_initialize_triforce_poly_thread(),
            3 | 4 | 9 | 11 => self.intro_handle_all_triforce_animations(),
            5 => self.intro_zelda_fadein(),
            6 => self.intro_sword_coming_down(0),
            7 => self.intro_fade_in_bg(enhanced_features0),
            8 => self.intro_wait_player(enhanced_features0),
            _ => {}
        }
    }
    pub fn intro_init(&mut self) {
        self.intro_setup_screen();
        self.ram.write_u8(INIDISP, 15);
        self.ram.write_u8(SUBSUBMODULE, 0);
        self.inc8(UPDATE_CGRAM);
        self.inc8(SUBMODULE);
        self.ram.write_u8(SOUND_EFFECT_2, 10);
        self.intro_init_continue();
    }
    pub fn intro_init_continue(&mut self) {
        self.intro_display_logo();
        let t = self.ram.read_u8(SUBSUBMODULE);
        self.inc8(SUBSUBMODULE);
        if t >= 11 {
            if self.dec8(INIDISP) == 0 {
                self.intro_initialize_memory_darken();
            }
            return;
        }
        match t {
            0..=7 => self.intro_clear_1kb_blocks_of_wram(),
            8 => self.intro_load_text_pointers_and_palettes(),
            9 => self.external("LoadItemGFXIntoWRAM4BPPBuffer"),
            10 => self.external("LoadFollowerGraphics"),
            _ => {}
        }
    }
    pub fn intro_clear_1kb_blocks_of_wram(&mut self) {
        let mut i = self.ram.read_u16(R16);
        let stop = self.ram.read_u16(R18);
        loop {
            for j in 0..15 {
                self.ram.write_u16(0x2000 + i as usize + j * 0x2000, 0);
            }
            i = i.wrapping_sub(2);
            if i == stop {
                break;
            }
        }
        self.ram.write_u16(R16, i);
        self.ram.write_u16(R18, i.wrapping_sub(0x400));
    }
    pub fn intro_initialize_memory_darken(&mut self) {
        self.external("EnableForceBlank");
        self.external("EraseTileMaps_normal");
        self.ram.write_u8(0xaa1, 35);
        self.ram.write_u8(0xaa3, 125);
        self.ram.write_u8(0xaa2, 81);
        self.ram.write_u8(0xaa4, 8);
        self.external("LoadDefaultGraphics");
        self.external("InitializeTilesets");
        self.runtime
            .decompress_animated_dungeon_tiles(self.ram, 0x5d);
        self.ram.write_u16(0xc00d, 2);
        self.ram.write_u8(0x8a, 0);
        self.ram.write_u8(0xab6, 0);
        self.ram.write_u8(0xab8, 0);
        self.ram.write_u16(R16, 0);
        self.ram.write_u16(R18, 0);
        self.ram.write_u16(DARKENING, 2);
        self.ram.write_u16(PALETTE_FILTER_COUNTDOWN, 31);
        self.ram.write_u8(MOSAIC_TARGET, 0);
        self.inc8(SUBMODULE);
    }
    pub fn intro_zelda_fadein(&mut self) {
        self.intro_handle_all_triforce_animations();
        if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
            return;
        }
        self.external("Palette_FadeIntroOneStep");
        let p = self.ram.read_u8(PALETTE_FILTER_COUNTDOWN);
        if p == 0 {
            self.ram.write_u8(SUBSUBMODULE, 42);
            self.inc8(SUBMODULE);
            self.intro_setup_sword_and_intro_flash();
        } else if p == 13 {
            self.ram.write_u8(TM, 0x15);
            self.ram.write_u8(TS, 0)
        }
    }
    pub fn intro_fade_in_bg(&mut self, features: u32) {
        self.intro_periodic_sword_and_intro_flash(features);
        self.intro_handle_all_triforce_animations();
        if self.ram.read_u8(PALETTE_FILTER_COUNTDOWN) != 0 {
            if self.ram.read_u8(FRAME_COUNTER) & 1 != 0 {
                self.external("Palette_FadeIntro2")
            }
        } else if self.joy_skip() {
            self.fade_music_and_reset_sram_mirror()
        } else if self.dec8(SUBSUBMODULE) == 0 {
            self.inc8(SUBMODULE);
        }
    }
    pub fn intro_sword_coming_down(&mut self, features: u32) {
        self.intro_handle_all_triforce_animations();
        self.ram.write_u8(INTRO_DID_RUN_STEP, 0);
        self.ram.write_u8(NMI_THREAD_ACTIVE, 0);
        self.intro_periodic_sword_and_intro_flash(features);
        if self.dec8(SUBSUBMODULE) == 0 {
            self.inc8(SUBMODULE);
            self.ram.write_u8(CGWSEL, 2);
            self.ram.write_u8(CGADSUB, 0x22);
            self.ram.write_u16(PALETTE_FILTER_COUNTDOWN, 31);
            self.ram.write_u8(TS, 2)
        }
    }
    pub fn intro_wait_player(&mut self, features: u32) {
        self.intro_handle_all_triforce_animations();
        self.ram.write_u8(INTRO_DID_RUN_STEP, 0);
        self.ram.write_u8(NMI_THREAD_ACTIVE, 0);
        self.intro_periodic_sword_and_intro_flash(features);
        if self.dec8(SUBSUBMODULE) == 0 {
            self.inc8(SUBMODULE);
            self.ram.write_u8(MAIN_MODULE, 20);
            self.ram.write_u8(SUBMODULE, 0);
            self.ram.write_u8(0x22, 0)
        }
    }
    pub fn fade_music_and_reset_sram_mirror(&mut self) {
        self.ram.write_u8(IRQ_FLAG, 255);
        self.ram.write_u8(TM, 0x15);
        self.ram.write_u8(TS, 0);
        self.ram.write_u8(MUSIC_CONTROL, 0xf1);
        self.external("SetBackdropcolorBlack");
        self.ram.slice_mut(0x20, 0x70).fill(0);
        self.ram.slice_mut(0xf000, 1280).fill(0);
        self.ram.write_u8(MAIN_MODULE, 1);
        self.ram.write_u8(0x3f4, 1);
        self.ram.write_u8(SUBMODULE, 0);
    }
    pub fn intro_initialize_triforce_poly_thread(&mut self) {
        self.external("LoadCommonSprites");
        self.intro_init_gfx_helper();
        for k in 0..3 {
            self.ram.write_u8(INTRO_INITED + k, 1);
            self.ram.write_u8(INTRO_SUBTYPE + k, 0)
        }
        self.ram.write_u8(INTRO_INITED + 4, 1);
        self.ram.write_u8(INTRO_SUBTYPE + 4, 2);
        self.ram.write_u8(INIDISP, 15);
        self.inc8(SUBMODULE);
    }
    pub fn intro_handle_all_triforce_animations(&mut self) {
        self.inc8(INTRO_FRAME_CTR);
        self.intro_animate_triforce();
        self.scene_animate_every_sprite();
    }
    pub fn scene_animate_every_sprite(&mut self) {
        self.ram.write_u16(INTRO_SPRITE_ALLOC, 0x800);
        for k in (0..8).rev() {
            self.intro_anim_one_obj(k)
        }
    }
    pub fn intro_animate_triforce(&mut self) {
        self.ram.write_u8(NMI_THREAD_ACTIVE, 1);
        if self.ram.read_u8(INTRO_DID_RUN_STEP) == 0 {
            self.intro_run_step();
            self.ram.write_u8(INTRO_DID_RUN_STEP, 1)
        }
    }
    pub fn intro_run_step(&mut self) {
        match self.ram.read_u8(INTRO_STEP_INDEX) {
            0 => {
                if self.inc8(INTRO_STEP_TIMER) == 64 {
                    self.inc8(INTRO_STEP_INDEX);
                }
                self.add8(POLY_B, 5);
                self.add8(POLY_A, 3)
            }
            1 => {
                if self.ram.read_u8(POLY_CONFIG1) < 2 {
                    self.ram.write_u8(POLY_CONFIG1, 0);
                    self.inc8(INTRO_STEP_INDEX);
                    self.ram.write_u8(INTRO_STEP_TIMER, 64);
                    return;
                }
                self.add8(POLY_CONFIG1, 254);
                self.add8(POLY_B, 5);
                self.add8(POLY_A, 3);
                let p = self.ram.read_u8(POLY_CONFIG1);
                if p < 225 {
                    self.ram.write_u8(SUBMODULE, 4)
                }
                if p == 113 {
                    self.ram.write_u8(MUSIC_CONTROL, 1)
                }
            }
            2 => {
                if self.dec8(INTRO_STEP_TIMER) == 0 {
                    self.inc8(INTRO_STEP_INDEX);
                } else {
                    self.add8(POLY_B, 5);
                    self.add8(POLY_A, 3)
                }
            }
            3 => {
                if self.ram.read_u8(POLY_B) >= 250 && self.ram.read_u8(POLY_A) >= 252 {
                    self.inc8(INTRO_STEP_INDEX);
                    self.ram.write_u8(INTRO_STEP_TIMER, 32)
                } else {
                    self.add8(POLY_B, 5);
                    self.add8(POLY_A, 3)
                }
            }
            4 => {
                self.ram.write_u8(POLY_B, 0);
                self.ram.write_u8(POLY_A, 0);
                if self.dec8(INTRO_STEP_TIMER) == 0 {
                    self.inc8(INTRO_STEP_INDEX);
                    self.ram.write_u8(INTRO_INITED + 5, 1);
                    self.ram.write_u8(INTRO_SUBTYPE + 5, 3);
                    self.ram.write_u8(TM, 0x10);
                    self.ram.write_u8(TS, 5);
                    self.ram.write_u8(CGWSEL, 2);
                    self.ram.write_u8(CGADSUB, 0x31);
                    self.ram.write_u8(SUBSUBMODULE, 0);
                    self.inc8(UPDATE_CGRAM);
                    self.ram.write_u8(NMI_LOAD_BG, 3);
                    self.inc8(SUBMODULE);
                }
            }
            _ => {}
        }
    }
    pub fn intro_anim_one_obj(&mut self, k: usize) {
        match (
            self.ram.read_u8(INTRO_INITED + k),
            self.ram.read_u8(INTRO_SUBTYPE + k),
        ) {
            (1, 0) => self.intro_sprite_type_a_0(k),
            (1, 2) => self.initialize_scene_sprite_copyright(k),
            (1, 3) => self.initialize_scene_sprite_sparkle(k),
            (1, 4..=6) => self.initialize_scene_sprite_triforce_room_triangle(k),
            (1, 7) => self.initialize_scene_sprite_credits_triangle(k),
            (2, 0) => self.intro_sprite_type_b_0(k),
            (2, 2) => self.animate_scene_sprite_copyright(k),
            (2, 3) => self.animate_scene_sprite_sparkle(k),
            (2, 4..=6) => self.intro_sprite_type_b_456(k),
            (2, 7) => self.animate_scene_sprite_credits_triangle(k),
            _ => {}
        }
    }
    pub fn intro_sprite_type_a_0(&mut self, k: usize) {
        const X: [i16; 3] = [-38, 95, 230];
        const Y: [i16; 3] = [200, -67, 200];
        const XV: [i8; 3] = [1, 0, -1];
        const YV: [i8; 3] = [-1, 1, -1];
        self.ram.write_u8(INTRO_X_LO + k, X[k] as u8);
        self.ram.write_u8(INTRO_X_HI + k, (X[k] >> 8) as u8);
        self.ram.write_u8(INTRO_Y_LO + k, Y[k] as u8);
        self.ram.write_u8(INTRO_Y_HI + k, (Y[k] >> 8) as u8);
        self.ram.write_i8(INTRO_X_VEL + k, XV[k]);
        self.ram.write_i8(INTRO_Y_VEL + k, YV[k]);
        self.inc8(INTRO_INITED + k);
    }
    pub fn intro_sprite_type_b_0(&mut self, k: usize) {
        const XL: [u8; 3] = [75, 95, 117];
        const YL: [u8; 3] = [88, 48, 88];
        self.animate_scene_sprite_draw_triangle(k);
        self.animate_scene_sprite_move_triangle(k);
        if self.ram.read_u8(INTRO_STEP_INDEX) != 5 {
            if self.ram.read_u8(INTRO_FRAME_CTR) & 31 == 0 {
                self.add8(INTRO_X_VEL + k, [1, 0, 255][k]);
                self.add8(INTRO_Y_VEL + k, [255, 1, 255][k])
            }
            if self.ram.read_u8(INTRO_X_LO + k) == XL[k] {
                self.ram.write_u8(INTRO_X_VEL + k, 0)
            }
            if self.ram.read_u8(INTRO_Y_LO + k) == YL[k] {
                self.ram.write_u8(INTRO_Y_VEL + k, 0)
            }
        } else {
            self.ram.write_u8(INTRO_X_VEL + k, 0);
            self.ram.write_u8(INTRO_Y_VEL + k, 0)
        }
    }
    pub fn animate_scene_sprite_draw_triangle(&mut self, k: usize) {
        self.draw_triangle(k, false)
    }
    pub fn animate_scene_sprite_add_objects_to_oam_buffer(
        &mut self,
        k: usize,
        entries: &[IntroSpriteEnt],
    ) {
        let x = u16::from(self.ram.read_u8(INTRO_X_LO + k))
            | u16::from(self.ram.read_u8(INTRO_X_HI + k)) << 8;
        let y = u16::from(self.ram.read_u8(INTRO_Y_LO + k))
            | u16::from(self.ram.read_u8(INTRO_Y_HI + k)) << 8;
        let allocation = self.ram.read_u16(INTRO_SPRITE_ALLOC) as usize;
        let base = (allocation - oam::OAM_BUFFER) / 4;
        for (i, e) in entries.iter().enumerate() {
            oam::set_helper0(
                self.ram,
                base + i,
                x.wrapping_add_signed(e.x as i16),
                y.wrapping_add_signed(e.y as i16),
                e.charnum,
                e.flags,
                e.ext,
            );
        }
        self.ram
            .write_u16(INTRO_SPRITE_ALLOC, (allocation + entries.len() * 4) as u16);
    }
    pub fn exit_0cca90(&mut self, _k: usize) {}
    pub fn intro_copy_sprite_type4_to_oam(&mut self, k: usize) {
        self.draw_triangle(k, true)
    }
    fn draw_triangle(&mut self, k: usize, pal4: bool) {
        let base = self.ram.read_u16(INTRO_SPRITE_ALLOC) as usize;
        let x = u16::from(self.ram.read_u8(INTRO_X_LO + k))
            | u16::from(self.ram.read_u8(INTRO_X_HI + k)) << 8;
        let y = u16::from(self.ram.read_u8(INTRO_Y_LO + k))
            | u16::from(self.ram.read_u8(INTRO_Y_HI + k)) << 8;
        for n in 0..16 {
            let col = if k == 2 { 3 - n % 4 } else { n % 4 };
            let row = n / 4;
            let chr = [
                0x80, 0x82, 0x84, 0x86, 0xa0, 0xa2, 0xa4, 0xa6, 0x88, 0x8a, 0x8c, 0x8e, 0xa8, 0xaa,
                0xac, 0xae,
            ][n];
            let flags = if k == 2 {
                if pal4 { 0x6b } else { 0x5b }
            } else if pal4 {
                0x2b
            } else {
                0x1b
            };
            oam::set_helper0(
                self.ram,
                (base - oam::OAM_BUFFER) / 4 + n,
                x.wrapping_add((col * 16) as u16),
                y.wrapping_add((row * 16) as u16),
                chr,
                flags,
                2,
            )
        }
        self.ram.write_u16(INTRO_SPRITE_ALLOC, (base + 64) as u16)
    }
    pub fn initialize_scene_sprite_copyright(&mut self, k: usize) {
        self.ram.write_u8(INTRO_X_LO + k, 76);
        self.ram.write_u8(INTRO_X_HI + k, 0);
        self.ram.write_u8(INTRO_Y_LO + k, 184);
        self.ram.write_u8(INTRO_Y_HI + k, 0);
        self.inc8(INTRO_INITED + k);
    }
    pub fn animate_scene_sprite_copyright(&mut self, k: usize) {
        const C: [u8; 13] = [
            0x40, 0x41, 0x42, 0x68, 0x41, 0x42, 0x43, 0x44, 0x50, 0x51, 0x52, 0x53, 0x54,
        ];
        let x = self.ram.read_u8(INTRO_X_LO + k);
        let y = self.ram.read_u8(INTRO_Y_LO + k);
        let allocation = self.ram.read_u16(INTRO_SPRITE_ALLOC) as usize;
        let base = (allocation - oam::OAM_BUFFER) / 4;
        for (n, &c) in C.iter().enumerate() {
            oam::set_helper0(
                self.ram,
                base + n,
                x.wrapping_add((n * 8) as u8) as u16,
                y as u16,
                c,
                0x0a,
                0,
            )
        }
        self.ram
            .write_u16(INTRO_SPRITE_ALLOC, (allocation + 13 * 4) as u16)
    }
    pub fn initialize_scene_sprite_sparkle(&mut self, k: usize) {
        const X: [u8; 4] = [0xc2, 0x98, 0x6f, 0x34];
        const Y: [u8; 4] = [0x7c, 0x54, 0x7c, 0x57];
        let j = (self.ram.read_u8(INTRO_FRAME_CTR) >> 5 & 3) as usize;
        self.ram.write_u8(INTRO_X_LO + k, X[j]);
        self.ram.write_u8(INTRO_X_HI + k, 0);
        self.ram.write_u8(INTRO_Y_LO + k, Y[j]);
        self.ram.write_u8(INTRO_Y_HI + k, 0);
        self.inc8(INTRO_INITED + k);
    }
    pub fn animate_scene_sprite_sparkle(&mut self, k: usize) {
        const ST: [u8; 8] = [0, 1, 2, 3, 2, 1, 255, 255];
        let s = self.ram.read_u8(INTRO_STATE + k);
        if s < 4 {
            let d = [
                (0, 0, 0x80, 0x34, 0),
                (0, 0, 0xb7, 0x34, 0),
                (-4, -3, 0x64, 0x38, 2),
                (-4, -3, 0x62, 0x34, 2),
            ][s as usize];
            let allocation = self.ram.read_u16(INTRO_SPRITE_ALLOC) as usize;
            let i = (allocation - oam::OAM_BUFFER) / 4;
            oam::set_helper0(
                self.ram,
                i,
                self.ram.read_u8(INTRO_X_LO + k).wrapping_add(d.0 as u8) as u16,
                self.ram.read_u8(INTRO_Y_LO + k).wrapping_add(d.1 as u8) as u16,
                d.2,
                d.3,
                d.4,
            );
            self.ram
                .write_u16(INTRO_SPRITE_ALLOC, (allocation + 4) as u16)
        }
        let f = self.ram.read_u8(INTRO_FRAME_CTR);
        self.ram
            .write_u8(INTRO_STATE + k, ST[(f >> 2 & 7) as usize]);
        let j = (f >> 5 & 3) as usize;
        self.ram
            .write_u8(INTRO_X_LO + k, [0xc2, 0x98, 0x6f, 0x34][j]);
        self.ram
            .write_u8(INTRO_Y_LO + k, [0x7c, 0x54, 0x7c, 0x57][j]);
    }
    pub fn animate_scene_sprite_move_triangle(&mut self, k: usize) {
        for (s, l, h, v) in [
            (INTRO_X_SUB, INTRO_X_LO, INTRO_X_HI, INTRO_X_VEL),
            (INTRO_Y_SUB, INTRO_Y_LO, INTRO_Y_HI, INTRO_Y_VEL),
        ] {
            let vel = self.ram.read_i8(v + k);
            if vel != 0 {
                let old = u32::from(self.ram.read_u8(s + k))
                    | (u32::from(self.ram.read_u8(l + k)) << 8)
                    | (u32::from(self.ram.read_u8(h + k)) << 16);
                let t = old.wrapping_add(((vel as i32) << 4) as u32);
                self.ram.write_u8(s + k, t as u8);
                self.ram.write_u8(l + k, (t >> 8) as u8);
                self.ram.write_u8(h + k, (t >> 16) as u8)
            }
        }
    }
    pub fn initialize_scene_sprite_triforce_room_triangle(&mut self, k: usize) {
        self.ram.write_u8(INTRO_X_LO + k, [0x4e, 0x5f, 0x72][k]);
        self.ram.write_u8(INTRO_X_HI + k, 0);
        self.ram.write_u8(INTRO_Y_LO + k, 0x9c);
        self.ram.write_u8(INTRO_Y_HI + k, 0);
        self.ram.write_i8(INTRO_X_VEL + k, [-2, 0, 2][k]);
        self.ram.write_i8(INTRO_Y_VEL + k, [4, -4, 4][k]);
        self.inc8(INTRO_INITED + k);
    }
    pub fn animate_triforce_room_triangle_handle_contracting(&mut self, k: usize) {
        for (v, p, t) in [
            (INTRO_X_VEL, INTRO_X_LO, [0x59, 0x5f, 0x67][k]),
            (INTRO_Y_VEL, INTRO_Y_LO, [0x74, 0x68, 0x74][k]),
        ] {
            let nv = self
                .ram
                .read_u8(v + k)
                .wrapping_add(if self.ram.read_u8(p + k) <= t { 1 } else { 255 });
            self.ram.write_u8(
                v + k,
                if nv == 0x11 {
                    0x10
                } else if nv == 0xef {
                    0xf0
                } else {
                    nv
                },
            )
        }
    }
    pub fn intro_sprite_type_b_456(&mut self, k: usize) {
        self.intro_copy_sprite_type4_to_oam(k);
        if self.ram.read_u8(INTRO_WANT_DOUBLE_RET) != 0 {
            return;
        }
        self.animate_scene_sprite_move_triangle(k);
        match self.ram.read_u8(INTRO_STEP_INDEX) {
            0 => {
                if self.ram.read_u8(INTRO_FRAME_CTR) & 7 == 0 {
                    self.add8(INTRO_X_VEL + k, [255, 0, 1][k])
                }
                if self.ram.read_u8(INTRO_FRAME_CTR) & 3 == 0 {
                    self.add8(INTRO_Y_VEL + k, 255)
                }
            }
            1 => {
                self.ram.write_u8(INTRO_X_VEL + k, 0);
                self.ram.write_u8(INTRO_Y_VEL + k, 0)
            }
            2 => {
                if self.ram.read_u8(INTRO_FRAME_CTR) & 3 == 0 {
                    self.animate_triforce_room_triangle_handle_contracting(k)
                }
                if self.ram.read_u8(INTRO_X_LO + k) == [0x59, 0x5f, 0x67][k] {
                    self.ram.write_u8(INTRO_X_VEL + k, 0)
                }
                if self.ram.read_u8(INTRO_Y_LO + k) == [0x74, 0x68, 0x74][k] {
                    self.ram.write_u8(INTRO_Y_VEL + k, 0)
                }
            }
            3 | 4 => {
                let t = self.ram.read_u16(TRIFORCE_CTR);
                if t == 0 {
                    self.ram.write_u8(INTRO_Y_LO + k, [0x72, 0x66, 0x72][k])
                } else {
                    self.ram.write_u16(TRIFORCE_CTR, t - 1)
                }
            }
            _ => {}
        }
    }
    pub fn initialize_scene_sprite_credits_triangle(&mut self, k: usize) {
        self.ram.write_u8(INTRO_X_LO + k, [0x29, 0x5f, 0x97][k]);
        self.ram.write_u8(INTRO_X_HI + k, 0);
        self.ram.write_u8(INTRO_Y_LO + k, [0x70, 0x20, 0x70][k]);
        self.ram.write_u8(INTRO_Y_HI + k, 0);
        self.inc8(INTRO_INITED + k);
    }
    pub fn animate_scene_sprite_credits_triangle(&mut self, k: usize) {
        self.load_triforce_sprite_palette();
        self.intro_copy_sprite_type4_to_oam(k);
        self.animate_scene_sprite_move_triangle(k);
        if self.ram.read_u8(SUBMODULE) != 36 {
            self.ram.write_u8(INTRO_STATE + k, 0)
        } else if self.ram.read_u8(INTRO_STATE + k) != 80 {
            self.inc8(INTRO_STATE + k);
            self.add8(INTRO_X_VEL + k, [255, 0, 1][k]);
            self.add8(INTRO_Y_VEL + k, [1, 255, 1][k])
        }
    }
    pub fn triforce_room_prep_gfx_slot_for_poly(&mut self) {
        self.external("LoadCommonSprites");
        self.intro_init_gfx_helper();
        for k in 0..3 {
            self.ram.write_u8(INTRO_INITED + k, 1);
            self.ram.write_u8(INTRO_SUBTYPE + k, (4 + k) as u8)
        }
        self.ram.write_u8(INIDISP, 15);
        self.inc8(SUBMODULE);
    }
    pub fn credits_initialize_polyhedral(&mut self) {
        self.external("LoadCommonSprites");
        self.intro_init_gfx_helper();
        self.ram.write_u8(POLY_CONFIG1, 0);
        for k in 0..3 {
            self.ram.write_u8(INTRO_INITED + k, 1);
            self.ram.write_u8(INTRO_SUBTYPE + k, 7)
        }
        self.ram.write_u8(INIDISP, 15);
        self.inc8(SUBMODULE);
    }
    pub fn advance_polyhedral(&mut self) {
        self.triforce_room_handle_poly();
        self.scene_animate_every_sprite()
    }
    pub fn triforce_room_handle_poly(&mut self) {
        self.ram.write_u8(NMI_THREAD_ACTIVE, 1);
        self.ram.write_u8(INTRO_WANT_DOUBLE_RET, 1);
        if self.ram.read_u8(INTRO_DID_RUN_STEP) != 0 {
            return;
        }
        match self.ram.read_u8(INTRO_STEP_INDEX) {
            0 => {
                self.add8(POLY_CONFIG1, 254);
                if self.ram.read_u8(POLY_CONFIG1) < 2 {
                    self.ram.write_u8(POLY_CONFIG1, 0);
                    self.inc8(INTRO_STEP_INDEX);
                    self.inc8(SUBSUBMODULE);
                }
                self.add8(POLY_B, 2);
                self.add8(POLY_A, 1)
            }
            1 => {
                if self.ram.read_u8(SUBSUBMODULE) >= 10 {
                    self.inc8(INTRO_STEP_INDEX);
                    self.ram.write_u8(INTRO_Y_VEL + 1, 5)
                }
                self.add8(POLY_B, 2);
                self.add8(POLY_A, 1)
            }
            2 => {
                self.ram.write_u16(TRIFORCE_CTR, 0x1c0);
                if self.ram.read_u8(POLY_CONFIG1) < 128 {
                    self.inc8(POLY_CONFIG1);
                } else if self.ram.read_u8(POLY_B).wrapping_sub(10) & 0x7f >= 92
                    && self.ram.read_u8(POLY_A).wrapping_sub(11) >= 220
                {
                    self.ram.write_u8(POLY_A, 0);
                    self.ram.write_u8(POLY_B, 0);
                    self.inc8(SUBSUBMODULE);
                    self.inc8(INTRO_STEP_INDEX);
                    self.ram.write_u8(SOUND_EFFECT_1, 44);
                    self.ram.write_u16(MAIN_PALETTE + 0x1ae, 0x7fff);
                    self.inc8(UPDATE_CGRAM);
                    self.ram.write_u8(INTRO_STEP_TIMER, 6);
                    return;
                }
                self.add8(POLY_B, 5);
                self.add8(POLY_A, 3)
            }
            3 => {
                if self.dec8(INTRO_STEP_TIMER) == 0 {
                    self.ram
                        .write_u16(MAIN_PALETTE + 0x1ae, POLYHEDRAL_PALETTE[7]);
                    self.inc8(UPDATE_CGRAM);
                    self.inc8(INTRO_STEP_INDEX);
                }
            }
            _ => {}
        }
        self.ram.write_u8(INTRO_DID_RUN_STEP, 1);
        self.ram.write_u8(INTRO_WANT_DOUBLE_RET, 0);
        self.inc8(INTRO_FRAME_CTR);
    }
    pub fn credits_animate_the_triangles(&mut self) {
        self.inc8(INTRO_FRAME_CTR);
        self.ram.write_u8(NMI_THREAD_ACTIVE, 1);
        if self.ram.read_u8(INTRO_DID_RUN_STEP) == 0 {
            self.add8(POLY_B, 3);
            self.add8(POLY_A, 1);
            self.ram.write_u8(INTRO_DID_RUN_STEP, 1)
        }
        self.scene_animate_every_sprite()
    }
    pub fn intro_display_logo(&mut self) {
        for (i, (&x, &c)) in [0x60, 0x70, 0x80, 0x88]
            .iter()
            .zip([0x69, 0x6b, 0x6d, 0x6e].iter())
            .enumerate()
        {
            oam::set_plain(self.ram, i, x, 0x68, c, 0x32, 2)
        }
    }
    pub fn intro_setup_sword_and_intro_flash(&mut self) {
        self.ram.write_u8(R18 + 1, 7);
        self.ram.write_u8(INTRO_SWORD_20, 0);
        self.ram.write_u8(INTRO_SWORD_21, 0);
        self.ram.write_u16(R16, (-130i16) as u16);
        self.intro_periodic_sword_and_intro_flash(0)
    }
    pub fn intro_periodic_sword_and_intro_flash(&mut self, features: u32) {
        if self.ram.read_u8(R18) != 0 {
            self.dec8(R18);
        }
        self.external("SetBackdropcolorBlack");
        let flashes = self.ram.read_u8(INTRO_FLASHES);
        if flashes != 0 {
            if flashes & 3 != 0 {
                let i = self.ram.read_u8(INTRO_SWORD_24) as usize;
                self.ram.write_u8(
                    COLDATA0 + i,
                    self.ram.read_u8(COLDATA0 + i) | if features & 65536 != 0 { 5 } else { 0x1f },
                );
                self.ram
                    .write_u8(INTRO_SWORD_24, if i == 2 { 0 } else { i as u8 + 1 })
            }
            self.dec8(INTRO_FLASHES);
        }
        let yp = self.ram.read_u16(R16);
        const CH: [u8; 10] = [0, 2, 0x20, 0x22, 4, 6, 8, 0xa, 0xc, 0xe];
        const X: [u8; 10] = [0x40, 0x40, 0x30, 0x50, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40];
        const Y: [u16; 10] = [0x10, 0x20, 0x28, 0x28, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80];
        for j in (0..10).rev() {
            let y = yp.wrapping_add(Y[j]);
            oam::set_plain(
                self.ram,
                0x52 + j,
                X[j],
                (if y & 0xff00 != 0 { 0xf8 } else { y as u8 }).wrapping_sub(8),
                CH[j],
                0x21,
                2,
            )
        }
        if yp != 30 {
            if yp == 0xffbe {
                self.ram.write_u8(SOUND_EFFECT_1, 1)
            } else if yp == 14 {
                self.ram.write_u16(INTRO_SWORD_24, 0);
                self.ram.write_u8(INTRO_FLASHES, 0x20);
                self.ram.write_u8(SOUND_EFFECT_1, 0x2c)
            }
            self.ram.write_u16(R16, yp.wrapping_add(16))
        }
        match self.ram.read_u8(INTRO_SWORD_20) >> 1 {
            0 => {
                if self.ram.read_u8(INTRO_FLASHES) == 0 && self.ram.read_u16(R16) == 30 {
                    self.add8(INTRO_SWORD_20, 2);
                }
            }
            1 => {
                const WAIT: [u8; 7] = [4, 4, 6, 6, 6, 4, 4];
                const TILE: [u8; 7] = [0x28, 0x37, 0x27, 0x36, 0x27, 0x37, 0x28];
                if self.ram.read_u8(R18) == 0 {
                    let k = self.ram.read_u8(R18 + 1).wrapping_sub(1);
                    self.ram.write_u8(R18 + 1, k);
                    if (k as i8) < 0 {
                        self.ram.write_u8(R18 + 1, 0);
                        self.ram.write_u8(R18, 2);
                        self.add8(INTRO_SWORD_20, 2);
                        return;
                    }
                    self.ram.write_u8(R18, WAIT[k as usize])
                }
                let k = self.ram.read_u8(R18 + 1) as usize;
                oam::set_plain(self.ram, 0x50, 0x44, 0x43, TILE[k], 0x25, 0)
            }
            2 => {
                const TILE: [u8; 8] = [0x26, 0x20, 0x24, 0x34, 0x25, 0x20, 0x35, 0x20];
                let k = self.ram.read_u8(R18 + 1) as usize;
                if k >= 7 {
                    return;
                }
                let y = self
                    .ram
                    .read_u8(INTRO_SWORD_21)
                    .min(0x4f)
                    .wrapping_add(self.ram.read_u16(R16) as u8)
                    .wrapping_add(0x31);
                oam::set_plain(self.ram, 0x50, 0x42, y, TILE[k], 0x23, 0);
                oam::set_plain(
                    self.ram,
                    0x51,
                    0x42,
                    y.wrapping_add(8),
                    TILE[k + 1],
                    0x23,
                    0,
                );
                if self.ram.read_u8(R18) == 0 {
                    let p = self.ram.read_u8(INTRO_SWORD_21).wrapping_add(4);
                    self.ram.write_u8(INTRO_SWORD_21, p);
                    if matches!(p, 4 | 0x48 | 0x4c | 0x58) {
                        self.add8(R18 + 1, 2);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn credits_prep_and_load_sprites(&mut self) {
        for k in (0..16).rev() {
            self.runtime.sprite_reset(self.ram, k);
            self.ram.write_u8(SPRITE_STATE + k, 0);
            self.ram.write_u8(SPRITE_FLAGS5 + k, 0);
            self.ram.write_u8(SPRITE_DEFL_BITS + k, 0)
        }
        let scene = (self.ram.read_u8(SUBMODULE) >> 1) as usize;
        let idx = ENDING_SPRITES_IDX[scene] as usize;
        let num = ENDING_SPRITES_IDX[scene + 1] as usize - idx;
        let dungeon = matches!(scene, 1 | 10 | 11);
        for k in (0..num).rev() {
            let (x, y) = if dungeon {
                let ri = self.ram.read_u16(DUNGEON_ROOM_INDEX2);
                (
                    ((ri & 15) << 9).wrapping_add(ENDING_SPRITES_X[idx + k]),
                    ((ri >> 3 & 254) << 8).wrapping_add(ENDING_SPRITES_Y[idx + k]),
                )
            } else {
                let a = self.ram.read_u16(OVERWORLD_AREA_INDEX);
                (
                    ((a << 1).swap_bytes() & 0xf00).wrapping_add(ENDING_SPRITES_X[idx + k]),
                    ((a >> 2).swap_bytes() & 0xe00).wrapping_add(ENDING_SPRITES_Y[idx + k]),
                )
            };
            self.runtime.sprite_set_x(self.ram, k, x);
            self.runtime.sprite_set_y(self.ram, k, y)
        }
        match scene {
            2 => self.ram.write_i8(SPRITE_Y_VEL + 6, -16),
            3 => {
                self.ram.write_u8(SPRITE_A + 5, 22);
                self.ram.write_i8(SPRITE_Y_VEL, -16);
                self.ram.write_i8(SPRITE_Y_VEL + 1, 16);
                self.ram.write_u8(SPRITE_HEAD_DIR + 1, 1);
                for j in 0..3 {
                    self.ram.write_u8(SPRITE_TYPE + 2 + j, 0x57);
                    self.ram.write_u8(SPRITE_OAM_FLAGS + 2 + j, 0x31)
                }
            }
            6 => {
                for k in 0..3 {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255)
                }
            }
            7 => self.ram.write_u8(SPRITE_DELAY_MAIN + 1, 255),
            9 => {
                for j in 0..5 {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + j, (j * 19) as u8);
                    self.ram.write_u8(SPRITE_STATE + j, 0)
                }
                self.ram.write_u8(SPRITE_TYPE + 5, 0x2e);
                for j in 0..2 {
                    self.ram.write_u8(SPRITE_TYPE + 7 + j, 0x9f);
                    self.ram.write_u8(SPRITE_TYPE + 9 + j, 0xa0);
                    self.ram.write_u8(SPRITE_FLAGS2 + 7 + j, 1);
                    self.ram.write_u8(SPRITE_FLAGS2 + 9 + j, 2);
                    self.ram.write_u8(SPRITE_FLAGS3 + 7 + j, 0x10);
                    self.ram.write_u8(SPRITE_FLAGS3 + 9 + j, 0x10)
                }
            }
            10 => {
                self.ram.write_u8(SPRITE_DELAY_MAIN + 1, 0x10);
                self.ram.write_u8(SPRITE_DELAY_MAIN + 2, 0x20);
                self.ram.write_u8(SPRITE_OAM_FLAGS + 3, 8);
                self.ram.write_u8(SPRITE_OAM_FLAGS + 4, 8)
            }
            11 => {
                self.ram.write_u8(SPRITE_OAM_FLAGS + 4, 0x79);
                self.ram.write_u8(SPRITE_OAM_FLAGS + 5, 0x39);
                self.ram.write_u8(SPRITE_D + 1, 1);
                self.ram.write_u8(SPRITE_A + 1, 4)
            }
            12 => {
                for j in 0..2 {
                    self.ram.write_u8(SPRITE_OAM_FLAGS + 3 + j, 0x39);
                    self.ram.write_u8(SPRITE_TYPE + 3 + j, 0xb);
                    self.ram.write_u8(SPRITE_FLAGS3 + 3 + j, 0x10);
                    self.ram.write_u8(SPRITE_FLAGS2 + 3 + j, 1)
                }
                self.ram.write_u8(SPRITE_TYPE + 5, 0x2a);
                self.ram.write_u8(SPRITE_TYPE + 6, 0x79);
                self.ram.write_u8(SPRITE_AI_STATE + 6, 1);
                self.ram.write_u8(SPRITE_Z + 6, 5)
            }
            14 => {
                self.ram.write_i8(SPRITE_Y_VEL + 5, -16);
                self.ram.write_i8(SPRITE_Y_VEL + 6, 16);
                self.ram.write_u8(SPRITE_HEAD_DIR + 6, 1);
                self.ram.write_u8(SPRITE_A, 8);
                for j in 0..4 {
                    self.ram.write_u8(SPRITE_Y_VEL + 1 + j, 4)
                }
            }
            15 => {
                self.ram.write_u8(SPRITE_C + 4, 2);
                self.ram.write_u8(SPRITE_Y_VEL + 5, 8);
                self.ram.write_u8(SPRITE_DELAY_MAIN + 1, 0x13);
                self.ram.write_u8(SPRITE_DELAY_MAIN + 4, 0x40)
            }
            _ => {}
        }
    }
    pub fn credits_scroll_scene_overworld(&mut self) {
        for k in (0..16).rev() {
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
                self.dec8(SPRITE_DELAY_MAIN + k);
            }
        }
        let i = (self.ram.read_u8(SUBMODULE) >> 1) as usize;
        self.ram.write_u8(LINK_X_VEL, 0);
        self.ram.write_u8(LINK_Y_VEL, 0);
        let r = self.ram.read_u16(R16);
        if r >= 0x40 && r & 1 == 0 {
            if self.ram.read_u16(BG2_VOFS2) != ENDING_TARGET_Y[i] {
                self.ram.write_i8(LINK_Y_VEL, ENDING_Y_VEL[i])
            }
            if self.ram.read_u16(BG2_HOFS2) != ENDING_TARGET_X[i] {
                self.ram.write_i8(LINK_X_VEL, ENDING_X_VEL[i])
            }
        }
        self.credits_operate_scrolling_and_tile_map();
        self.credits_handle_scene_fade()
    }
    pub fn credits_scroll_scene_dungeon(&mut self) {
        for k in (0..16).rev() {
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
                self.dec8(SPRITE_DELAY_MAIN + k);
            }
        }
        let i = (self.ram.read_u8(SUBMODULE) >> 1) as usize;
        let r = self.ram.read_u16(R16);
        if r >= 0x40 && r & 1 == 0 {
            if self.ram.read_u16(BG2_VOFS2) != ENDING_TARGET_Y[i] {
                self.ram.write_u16(
                    BG2_VOFS2,
                    self.ram
                        .read_u16(BG2_VOFS2)
                        .wrapping_add_signed(ENDING_Y_VEL[i] as i16),
                )
            }
            if self.ram.read_u16(BG2_HOFS2) != ENDING_TARGET_X[i] {
                self.ram.write_u16(
                    BG2_HOFS2,
                    self.ram
                        .read_u16(BG2_HOFS2)
                        .wrapping_add_signed(ENDING_X_VEL[i] as i16),
                )
            }
        }
        self.credits_handle_scene_fade()
    }
    pub fn credits_handle_scene_fade(&mut self) {
        const LIMIT: [u16; 16] = [
            0x300, 0x280, 0x250, 0x2e0, 0x280, 0x250, 0x2c0, 0x2c0, 0x250, 0x250, 0x280, 0x250,
            0x480, 0x400, 0x250, 0x500,
        ];
        self.credits_draw_scene_sprites();
        let k = (self.ram.read_u8(SUBMODULE) >> 1) as usize;
        let r = self.ram.read_u16(R16);
        if r >= LIMIT[k] {
            if r & 1 == 0 && self.dec8(INIDISP) == 0 {
                self.inc8(SUBMODULE);
            } else {
                self.ram.write_u16(R16, r.wrapping_add(1))
            }
        } else {
            if r & 1 == 0 && self.ram.read_u8(INIDISP) != 15 {
                self.inc8(INIDISP);
            }
            self.ram.write_u16(R16, r.wrapping_add(1))
        }
        self.copy_scroll()
    }
    fn credits_draw_scene_sprites(&mut self) {
        let scene = (self.ram.read_u8(SUBMODULE) >> 1) as usize;
        let frame = self.ram.read_u8(FRAME_COUNTER);
        let r = self.ram.read_u16(R16);
        match scene {
            0 => {
                const F: [u8; 12] = [
                    0x3b, 0x31, 0x3d, 0x3f, 0x39, 0x3b, 0x37, 0x3d, 0x39, 0x37, 0x37, 0x39,
                ];
                const N: [u8; 12] = [6, 3, 2, 2, 2, 2, 2, 2, 6, 6, 6, 6];
                const J: [u8; 12] = [
                    0x1e, 0x20, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x16, 0x16, 0x16, 0x16,
                ];
                for k in (0..12).rev() {
                    self.ram.write_u8(
                        SPRITE_OAM_FLAGS + k,
                        F[k] | if (2..=7).contains(&k) {
                            frame << 2 & 0x40
                        } else {
                            0
                        },
                    );
                    self.credits_sprite_draw_single(k, N[k], J[k]);
                }
            }
            1 => {
                self.credits_sprite_draw_single(0, 3, 12);
                self.credits_sprite_draw_draw_shadow(0);
                self.ram.write_u8(SPRITE_TYPE + 1, 0x73);
                self.ram.write_u8(SPRITE_OAM_FLAGS + 1, 0x27);
                self.ram.write_u8(SPRITE_E + 1, 2);
                self.credits_sprite_draw_preexisting_sprite_draw(1, 16)
            }
            2 => {
                self.ram
                    .write_u8(0x3ca, [0x20, 0x40][(frame >> 2 & 1) as usize]);
                let k = 6;
                let j = (self.ram.read_u8(SPRITE_X_VEL + k) >> 7) as usize;
                self.ram.write_u8(
                    SPRITE_OAM_FLAGS + k,
                    (self
                        .ram
                        .read_u8(SPRITE_X_VEL + k)
                        .wrapping_add([16u8, 240][j])
                        >> 1
                        & 0x40)
                        | 0x32,
                );
                self.credits_sprite_draw_single(k, 2, 0x24);
                self.credits_sprite_draw_circling_birds(k);
                let k = 5;
                self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x31);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    let j = self.ram.read_u8(SPRITE_A + k);
                    self.ram.write_u8(SPRITE_A + k, j ^ 1);
                    self.ram
                        .write_u8(SPRITE_DELAY_MAIN + k, [0x30, 0x10][j as usize]);
                    self.ram.write_u8(
                        SPRITE_GRAPHICS + k,
                        self.ram.read_u8(SPRITE_GRAPHICS + k).wrapping_add(1) & 3,
                    )
                }
                self.credits_sprite_draw_single(k, 2, 0x26);
                for k in (0..5).rev() {
                    if frame & 15 == 0 {
                        self.ram.write_u8(
                            SPRITE_GRAPHICS + k,
                            self.ram.read_u8(SPRITE_GRAPHICS + k) ^ 1,
                        )
                    }
                    self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x31);
                    self.credits_sprite_draw_single(k, 3, [0x28, 0x2a, 0x2c, 0x2e, 0x2c][k]);
                    self.end_sequence_draw_shadow2(k)
                }
            }
            3 => {
                const G: [u8; 4] = [1, 2, 3, 2];
                for k in 0..5 {
                    if k < 2 {
                        self.ram.write_u8(SPRITE_TYPE + k, 1);
                        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0xb);
                        self.credits_sprite_draw_set_shadow_prop(k, 2);
                        self.ram.write_u8(SPRITE_Z + k, 48);
                        self.ram.write_u8(
                            SPRITE_GRAPHICS + k,
                            G[((frame.wrapping_add(if k != 0 { 0x5f } else { 0x7d })) >> 2 & 3)
                                as usize],
                        );
                        self.credits_sprite_draw_circling_birds(k);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 12)
                    } else {
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 16)
                    }
                }
                self.credits_sprite_draw_single(5, 2, 0x38);
                self.ending_func2(5, 0x30);
                self.credits_sprite_draw_single(6, 3, 0x3a)
            }
            4 => {
                const V: [i8; 10] = [0, -12, -16, -12, 0, 12, 16, 12, 0, -12];
                const DV: [u8; 24] = [
                    0x3b, 0x14, 0x1e, 0x1d, 0x2c, 0x2b, 0x42, 0x20, 0x27, 0x28, 0x2e, 0x38, 0x3a,
                    0x4c, 0x32, 0x44, 0x2e, 0x2f, 0x1e, 0x28, 0x47, 0x35, 0x32, 0x30,
                ];
                self.ram.write_u8(SPRITE_OAM_FLAGS + 2, 0x35);
                self.credits_sprite_draw_single(2, 1, 0x3c);
                for k in (0..2).rev() {
                    self.ram.write_u8(
                        SPRITE_OAM_FLAGS + k,
                        (self.ram.read_u8(SPRITE_X_VEL + k).wrapping_sub(1) >> 1 & 0x40) ^ 0x71,
                    );
                    self.ram.write_u8(SPRITE_GRAPHICS + k, frame >> 3 & 1);
                    if r >= [0x20, 0][k] && self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                        let a = DV[self.ram.read_u8(SPRITE_A + k) as usize];
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, a & 0xf8);
                        self.ram.write_i8(SPRITE_Y_VEL + k, V[(a & 7) as usize + 2]);
                        self.ram.write_i8(SPRITE_X_VEL + k, V[(a & 7) as usize]);
                        self.inc8(SPRITE_A + k);
                    }
                    self.credits_sprite_draw_single(k, 2, [0x30, 0x32][k]);
                    self.end_sequence_draw_shadow2(k);
                    self.runtime.sprite_move_xy(self.ram, k)
                }
            }
            5 => {
                if r == 0x200 {
                    self.ram.write_u8(SOUND_EFFECT_1, 1)
                } else if r == 0x208 {
                    self.ram.write_u8(SOUND_EFFECT_1, 0x2c)
                }
                if r.wrapping_sub(0x208) < 0x30 {
                    self.credits_sprite_draw_add_sparkle(2, 10, r.wrapping_sub(0x208) as u8)
                }
                let j = if r >= 0x200 {
                    self.ram.write_u8(SPRITE_GRAPHICS + 3, 1);
                    1
                } else {
                    self.ram.read_u8(SPRITE_GRAPHICS + 3)
                };
                self.ram.write_u8(SPRITE_OAM_FLAGS + 3, 0x31);
                self.credits_sprite_draw_single(3, 4, 8);
                self.end_sequence_draw_shadow2(3);
                self.ram.write_u8(SPRITE_GRAPHICS + 2, j);
                self.ram.write_u8(SPRITE_OAM_FLAGS + 2, 0x30);
                self.ram
                    .write_u16(LINK_DMA_GRAPHICS_INDEX, [0xa, 0x224][j as usize]);
                self.credits_sprite_draw_single(2, 5, [10, 14][j as usize]);
                self.end_sequence_draw_shadow2(2)
            }
            6 => {
                let idx = ENDING_SPRITES_IDX[scene] as usize;
                let n = ENDING_SPRITES_IDX[scene + 1] as usize - idx;
                for k in (0..n).rev() {
                    self.ram.write_u8(SPRITE_TYPE + k, [0x52, 0x55, 0x55][k]);
                    self.runtime
                        .oam_allocate_region_a(self.ram, [0x20, 8, 8][k]);
                    self.ram.write_u8(SPRITE_AI_STATE + k, [3, 1, 1][k]);
                    let j = if r >= 0x26f { k + 3 } else { k };
                    if r == 0x26f {
                        self.ram.write_u8(SOUND_EFFECT_2, 0x21)
                    }
                    self.ram
                        .write_u8(SPRITE_GRAPHICS + k, [0, 5, 5, 1, 6, 6][j]);
                    self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x33);
                    self.runtime.sprite_get_16_bit_coords(self.ram, k);
                    self.runtime.sprite_active_main(self.ram, k)
                }
            }
            7 => {
                let k = 1;
                self.credits_sprite_draw_set_shadow_prop(k, 2);
                self.ram.write_u8(SPRITE_TYPE + k, 0xe9);
                self.runtime.oam_allocate_region_a(self.ram, 12);
                self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x37);
                if frame & 15 == 0 {
                    self.ram.write_u8(
                        SPRITE_GRAPHICS + k,
                        self.ram.read_u8(SPRITE_GRAPHICS + k) ^ 1,
                    )
                }
                self.runtime.sprite_active_main(self.ram, k);
                if r >= 0x180 {
                    self.ram.write_u8(SPRITE_Y_VEL + k, 4);
                    if self.ram.read_u8(SPRITE_Y_LO + k) != 0x7c {
                        self.runtime.sprite_move_xy(self.ram, k)
                    }
                }
                let k = 0;
                self.ram.write_u8(SPRITE_TYPE + k, 0x36);
                self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x39);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 4);
                    self.ram.write_u8(
                        SPRITE_GRAPHICS + k,
                        self.ram
                            .read_u8(SPRITE_GRAPHICS + k)
                            .wrapping_add(if r >> 9 & 1 != 0 { 255 } else { 1 })
                            & 7,
                    )
                }
                self.runtime.sprite_active_main(self.ram, k)
            }
            8 => {
                self.ram.write_u8(SPRITE_TYPE, 0x2c);
                self.runtime.oam_allocate_region_a(self.ram, 0x2c);
                self.ram.write_u8(SPRITE_OAM_FLAGS, 0x3b);
                self.ram.write_u8(
                    SPRITE_GRAPHICS,
                    if r < 0x1c0 { (r >> 5 & 1) as u8 } else { 2 },
                );
                self.runtime.sprite_active_main(self.ram, 0)
            }
            9 => {
                for k in 0..5 {
                    if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 96);
                        self.ram.write_u8(SPRITE_STATE + k, 96);
                        self.ram.write_u8(SPRITE_X_VEL + k, 0);
                        self.ram.write_u8(SPRITE_X_LO + k, 238);
                        self.ram.write_u8(SPRITE_X_HI + k, 4);
                        self.ram.write_u8(SPRITE_Y_LO + k, 24);
                        self.ram.write_u8(SPRITE_Y_HI + k, 11)
                    }
                    if self.ram.read_u8(SPRITE_STATE + k) != 0 {
                        self.ram.write_i8(SPRITE_Y_VEL + k, -8);
                        self.runtime.sprite_move_xy(self.ram, k);
                        if frame & 1 == 0 {
                            self.add8(
                                SPRITE_X_VEL + k,
                                if ((frame >> 5) ^ k as u8) & 1 != 0 {
                                    255
                                } else {
                                    1
                                },
                            )
                        }
                        self.credits_sprite_draw_single(k, 1, 0x10)
                    }
                }
                for k in 5..7 {
                    if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                        let a = self.ram.read_u8(SPRITE_A + k) as usize;
                        self.ram.write_u8(
                            SPRITE_DELAY_MAIN + k,
                            if k == 5 {
                                [16, 14, 16, 18][a]
                            } else {
                                [20, 48, 20, 20][a]
                            },
                        );
                        self.ram.write_u8(SPRITE_A + k, (a as u8 + 1) & 3);
                        self.ram.write_u8(
                            SPRITE_GRAPHICS + k,
                            self.ram.read_u8(SPRITE_GRAPHICS + k) ^ 1,
                        )
                    }
                    if k == 5 {
                        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x31);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 16)
                    } else {
                        self.credits_sprite_draw_single(k, 2, 0x12)
                    }
                }
                for k in 7..11 {
                    self.ram
                        .write_u8(SPRITE_OAM_FLAGS + k, [55, 55, 59, 61][k - 7]);
                    self.ram.write_u8(SPRITE_D + k, [0, 1, 0, 1][k - 7]);
                    self.credits_sprite_draw_activate_and_run_sprite(k, [8, 8, 12, 12][k - 7])
                }
            }
            10 => {
                let k = 5;
                self.runtime.sprite_get_16_bit_coords(self.ram, k);
                if self.ram.read_u8(SPRITE_PAUSE + k) == 0 {
                    let rx = (self.runtime.random_number(self.ram) & 7) as usize;
                    let ry = (self.runtime.random_number(self.ram) & 7) as usize;
                    let xb = self
                        .ram
                        .read_u8(SPRITE_X_LO + k)
                        .wrapping_add([0, 4, 8, 12, 16, 20, 24, 0][rx]);
                    let yb = self
                        .ram
                        .read_u8(SPRITE_Y_LO + k)
                        .wrapping_add([0, 8, 16, 24, 32, 40, 4, 36][ry]);
                    self.credits_sprite_draw_add_sparkle(3, xb, yb)
                }
                for k in 3..5 {
                    if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) != 0 {
                        self.dec8(SPRITE_DELAY_AUX1 + k);
                    }
                    self.ram.write_u8(SPRITE_TYPE + k, 0xe3);
                    self.credits_sprite_draw_set_shadow_prop(k, 1);
                    self.credits_sprite_draw_activate_and_run_sprite(k, 8)
                }
                self.ram.write_u8(SPRITE_TYPE + 5, 0x72);
                self.ram.write_u8(SPRITE_OAM_FLAGS + 5, 0x3b);
                self.ram.write_u8(SPRITE_STATE + 5, 9);
                self.ram.write_u8(SPRITE_B + 5, 9);
                self.credits_sprite_draw_preexisting_sprite_draw(5, 0x30)
            }
            11 => {
                if r >= 0x170 {
                    for k in 4..6 {
                        self.credits_sprite_draw_single(k, 1, 0x3e)
                    }
                    self.ram.write_u8(SPRITE_OAM_FLAGS, 0x39);
                    if r < 0x1c0 {
                        self.ram.write_u8(SPRITE_GRAPHICS, 2)
                    } else if self.ram.read_u8(SPRITE_DELAY_MAIN) == 0 {
                        self.ram.write_u8(SPRITE_DELAY_MAIN, 0x20);
                        self.ram
                            .write_u8(SPRITE_GRAPHICS, (self.ram.read_u8(SPRITE_GRAPHICS) ^ 1) & 1)
                    }
                    self.credits_sprite_draw_single(0, 4, 6)
                } else {
                    const G: [u8; 16] = [1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0];
                    for k in 0..2 {
                        self.ram.write_u8(SPRITE_TYPE + k, 0x1a);
                        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x39);
                        self.credits_sprite_draw_set_shadow_prop(k, 2);
                        self.credits_sprite_draw_activate_and_run_sprite(k, 12);
                        if self.ram.read_u8(SPRITE_B + k) == 15
                            && self.ram.read_u8(SPRITE_A + k) == 4
                        {
                            self.ram.write_u8(SPRITE_DELAY_MAIN + k + 2, 15)
                        }
                        let j = self.ram.read_u8(SPRITE_DELAY_MAIN + k + 2);
                        if j != 0 {
                            self.ram.write_u8(SPRITE_OAM_FLAGS + k + 2, 2);
                            self.ram.write_u8(SPRITE_GRAPHICS + k + 2, G[j as usize]);
                            self.credits_sprite_draw_single(k + 2, 2, 0x36)
                        }
                    }
                }
            }
            12 => {
                let k = 6;
                self.ram.write_u8(SPRITE_GRAPHICS + k, frame & 1);
                if frame & 1 == 0 {
                    self.add8(
                        SPRITE_X_VEL + k,
                        if (self.ram.read_u8(SPRITE_X_LO + k).wrapping_sub(0x80) as i8) < 0 {
                            1
                        } else {
                            255
                        },
                    );
                    self.add8(
                        SPRITE_Y_VEL + k,
                        if (self.ram.read_u8(SPRITE_Y_LO + k).wrapping_sub(0xb0) as i8) < 0 {
                            1
                        } else {
                            255
                        },
                    );
                    self.runtime.sprite_move_xy(self.ram, k)
                }
                self.ram.write_u8(
                    SPRITE_OAM_FLAGS + k,
                    (self.ram.read_u8(SPRITE_X_VEL + k) >> 1 & 0x40) ^ 0x7e,
                );
                self.credits_sprite_draw_set_shadow_prop(k, 1);
                self.ram.write_u8(SPRITE_Z + k, 16);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 8);
                for (k, a) in [(5, 12), (4, 8), (3, 8)] {
                    self.credits_sprite_draw_activate_and_run_sprite(k, a)
                }
                for k in (0..3).rev() {
                    self.credits_sprite_draw_single(k, [3, 3, 8][k], (k * 2) as u8);
                    if k == 0 {
                        self.ending_func2(k, 0x30)
                    } else if k == 2 {
                        self.ram.write_u8(SPRITE_GRAPHICS + k, frame >> 3 & 1)
                    } else {
                        const Z: [u8; 15] = [2, 4, 5, 6, 6, 7, 7, 7, 7, 6, 6, 5, 4, 2, 0];
                        let j = (frame & 0x1f) as usize;
                        if j < 15 {
                            self.ram.write_u8(SPRITE_Z + k, Z[j])
                        }
                        self.ram
                            .write_u8(SPRITE_GRAPHICS + k, if j < 15 { 1 } else { 0 });
                        self.credits_sprite_draw_draw_shadow(k)
                    }
                }
            }
            13 => {
                if r == 0x200 {
                    self.ram.write_i8(SPRITE_X_VEL, -4)
                }
                self.ram.write_u8(SPRITE_GRAPHICS, frame >> 4 & 1);
                if self.ram.read_u8(SPRITE_X_LO) == 56 {
                    self.ram.write_u8(SPRITE_X_VEL, 0);
                    self.add8(SPRITE_GRAPHICS, 2)
                }
                self.credits_sprite_draw_single(0, 3, 0x34);
                self.runtime.sprite_move_xy(self.ram, 0)
            }
            14 => {
                for k in (1..7).rev() {
                    if k >= 5 {
                        self.ram.write_u8(SPRITE_TYPE + k, 0);
                        self.credits_sprite_draw_set_shadow_prop(k, 1);
                        self.ram
                            .write_u8(SPRITE_GRAPHICS + k, (frame.wrapping_add(0x4a) & 8) >> 3);
                        self.ram.write_u8(SPRITE_Z + k, 32);
                        self.credits_sprite_draw_circling_birds(k);
                        self.ram.write_u8(
                            SPRITE_OAM_FLAGS + k,
                            (self.ram.read_u8(SPRITE_X_VEL + k) >> 1 & 0x40) ^ 0xf,
                        );
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 8)
                    } else {
                        self.ram.write_u8(SPRITE_TYPE + k, 0xd);
                        if k == 1 {
                            self.ram.write_u8(SPRITE_HEAD_DIR + k, 0xd)
                        }
                        self.credits_sprite_draw_set_shadow_prop(k, 3);
                        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x2b);
                        let mut a = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                        if a == 0 {
                            a = 0xc0;
                            self.ram.write_u8(SPRITE_DELAY_MAIN + k, a)
                        }
                        a >>= 1;
                        if a == 0 {
                            self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                            self.ram.write_u8(SPRITE_X_VEL + k, 0)
                        } else if a < [2, 8, 32, 32, 8][k] && frame & 3 == 0 {
                            let mut v = self.ram.read_u8(SPRITE_Y_VEL + k);
                            if v != 0 {
                                v = v.wrapping_sub(1);
                                self.ram.write_u8(SPRITE_Y_VEL + k, v);
                                let mut x = v.wrapping_sub(4);
                                if k < 3 {
                                    x = 0u8.wrapping_sub(x)
                                }
                                self.ram.write_u8(SPRITE_X_VEL + k, x)
                            }
                        }
                        self.runtime.sprite_move_xy(self.ram, k);
                        self.ram
                            .write_u8(SPRITE_GRAPHICS + k, [0, 1, 0, 2][(frame >> 3 & 3) as usize]);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 16)
                    }
                }
                self.credits_sprite_draw_single(0, 3, 0x18);
                self.ending_func2(0, 0x20)
            }
            15 => {
                let j = (self.runtime.generated_end_sequence_15(frame) & 3) as usize;
                self.credits_sprite_draw_add_sparkle(
                    2,
                    [0x76, 0x73, 0x71, 0x78][j],
                    [0x8b, 0x83, 0x8d, 0x85][j],
                );
                self.ram.write_u8(SPRITE_TYPE + 2, 0x62);
                self.ram.write_u8(SPRITE_OAM_FLAGS + 2, 0x39);
                self.credits_sprite_draw_preexisting_sprite_draw(2, 0x18);
                for (j, k) in [(1usize, 3usize), (0, 4)] {
                    if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) != 0 {
                        self.dec8(SPRITE_DELAY_AUX1 + k);
                    }
                    self.ram.write_u8(
                        SPRITE_OAM_FLAGS + k,
                        (self.ram.read_u8(SPRITE_X_VEL + k) >> 1 & 0x40)
                            ^ [0x61, 0x61, 0x3b, 0x39][j],
                    );
                    if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
                        self.ram.write_u8(SPRITE_A + k, 0)
                    }
                    if self.ram.read_u8(SPRITE_A + k) == 0 {
                        self.ram.write_u8(SPRITE_GRAPHICS + k, (frame >> 2 & 1) + 2);
                        self.credits_sprite_draw_move_squirrel(k)
                    } else if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) == 0 {
                        if self.ram.read_u8(SPRITE_B + k) == 8 {
                            self.ram.write_u8(SPRITE_B + k, 0)
                        }
                        self.ram.write_u8(
                            SPRITE_DELAY_AUX1 + k,
                            [6, 6, 6, 6, 6, 6, 10, 8]
                                [(self.ram.read_u8(SPRITE_B + k) & 7) as usize],
                        );
                        self.ram.write_u8(
                            SPRITE_GRAPHICS + k,
                            (self.ram.read_u8(SPRITE_GRAPHICS + k) & 1) ^ 1,
                        );
                        self.inc8(SPRITE_B + k);
                    }
                    self.credits_sprite_draw_single(k, 1, 20);
                    self.end_sequence_draw_shadow2(k)
                }
                self.credits_sprite_draw_walk_link_away_from_pedestal(5)
            }
            _ => {}
        }
    }
    pub fn credits_operate_scrolling_and_tile_map(&mut self) {
        self.credits_handle_camera_scroll_control();
        if self.ram.read_u8(OW_TRANS_DIR) != 0 {
            self.external("OverworldHandleMapScroll")
        }
    }
    pub fn credits_handle_camera_scroll_control(&mut self) {
        self.scroll_axis(
            LINK_Y_VEL,
            BG2_VOFS2,
            OW_UNK1,
            OW_UNK1_NEG,
            BG1_VOFS2,
            BG1_VOFS_SUB,
            8,
            4,
            true,
        );
        self.scroll_axis(
            LINK_X_VEL,
            BG2_HOFS2,
            OW_UNK3,
            OW_UNK3_NEG,
            BG1_HOFS2,
            BG1_HOFS_SUB,
            2,
            1,
            false,
        );
        let oi = self.ram.read_u8(OVERLAY_INDEX);
        if oi == 0x9c {
            let t = (u32::from(self.ram.read_u16(BG1_VOFS2)) << 16
                | u32::from(self.ram.read_u16(BG1_VOFS_SUB)))
            .wrapping_sub(0x2000);
            self.ram.write_u16(BG1_VOFS_SUB, t as u16);
            self.ram.write_u16(BG1_VOFS2, (t >> 16) as u16);
            self.ram.write_u16(BG1_HOFS2, self.ram.read_u16(BG2_HOFS2))
        } else if oi == 0x97 || oi == 0x9d {
            for (a, s) in [(BG1_VOFS2, BG1_VOFS_SUB), (BG1_HOFS2, BG1_HOFS_SUB)] {
                let t = (u32::from(self.ram.read_u16(a)) << 16 | u32::from(self.ram.read_u16(s)))
                    .wrapping_add(0x2000);
                self.ram.write_u16(s, t as u16);
                self.ram.write_u16(a, (t >> 16) as u16)
            }
        }
        if self.ram.read_u16(DUNGEON_ROOM_INDEX) == 0x181 {
            self.ram
                .write_u16(BG1_VOFS2, self.ram.read_u16(BG2_VOFS2) | 0x100);
            self.ram.write_u16(BG1_HOFS2, self.ram.read_u16(BG2_HOFS2))
        }
    }
    // Kept scalar to mirror the two C branches and make each WRAM alias auditable.
    #[allow(clippy::too_many_arguments)]
    fn scroll_axis(
        &mut self,
        vel_a: usize,
        bg2: usize,
        pos: usize,
        neg: usize,
        bg1: usize,
        sub: usize,
        negbit: u16,
        posbit: u16,
        vertical: bool,
    ) {
        let vel = self.ram.read_i8(vel_a);
        if vel == 0 {
            return;
        }
        self.ram
            .write_u16(bg2, self.ram.read_u16(bg2).wrapping_add_signed(vel as i16));
        let (w, other) = if vel < 0 { (pos, neg) } else { (neg, pos) };
        let n = self.ram.read_u16(w).wrapping_add(vel.unsigned_abs() as u16);
        let n2 = if n >= 0x10 {
            self.ram.write_u16(
                OW_TRANS_DIR,
                self.ram.read_u16(OW_TRANS_DIR) | (if vel < 0 { negbit } else { posbit }),
            );
            n - 0x10
        } else {
            n
        };
        self.ram.write_u16(w, n2);
        self.ram.write_u16(other, 0u16.wrapping_sub(n2));
        let oi = self.ram.read_u8(OVERLAY_INDEX);
        if oi == 0x97 || oi == 0x9d {
            return;
        }
        let quarter = if vertical {
            oi == 0xb5 || oi == 0xbe
        } else {
            oi == 0x95 || oi == 0x9e
        };
        let rv = vel as i16 as u16;
        let (sh, sp) = if quarter {
            (2, (rv & 3) << 14)
        } else {
            (1, (rv & 1) << 15)
        };
        let hi = ((rv >> sh) | if rv >> (sh + 12) != 0 { 0xf000 } else { 0 }) as u32;
        let t = (u32::from(self.ram.read_u16(bg1)) << 16 | u32::from(self.ram.read_u16(sub)))
            .wrapping_add((hi << 16) | u32::from(sp));
        self.ram.write_u16(sub, t as u16);
        self.ram.write_u16(bg1, (t >> 16) as u16)
    }
    pub fn ending_func2(&mut self, k: usize, ain: u8) {
        const D: [u8; 27] = [
            10, 10, 10, 10, 20, 8, 8, 0, 255, 12, 12, 12, 12, 12, 12, 30, 8, 4, 4, 4, 0, 0, 255,
            255, 144, 4, 0,
        ];
        const G: [i8; 28] = [
            0, 0, 1, 0, 1, 0, 2, 3, 0, 2, 0, 1, 0, 1, 0, 1, 2, 3, 4, 5, 6, 3, 0, -1, -1, -1, 2, 3,
        ];
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, ain);
        self.end_sequence_draw_shadow2(k);
        let mut j = self.ram.read_u8(SPRITE_A + k);
        if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
            j = j.wrapping_add(1);
            if j == 8 {
                j = 6
            } else if j == 22 {
                j = 21
            } else if j == 28 {
                j = 27
            }
            self.ram.write_u8(SPRITE_A + k, j);
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, D[j as usize - 1])
        }
        let a = G[j as usize] as u8;
        self.ram.write_u8(
            SPRITE_GRAPHICS + k,
            if a == 255 {
                self.ram.read_u8(FRAME_COUNTER) >> 3 & 1
            } else {
                a
            },
        );
        if (j < 5 || (10..15).contains(&j)) && self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
            self.inc8(SPRITE_Y_LO + k);
        }
    }
    pub fn credits_sprite_draw_set_shadow_prop(&mut self, k: usize, a: u8) {
        self.ram.write_u8(SPRITE_FLAGS2 + k, a);
        self.ram.write_u8(SPRITE_FLAGS3 + k, 16)
    }
    pub fn credits_sprite_draw_draw_shadow(&mut self, k: usize) {
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0x30);
        self.credits_sprite_draw_set_shadow_prop(k, 0);
        self.runtime.oam_allocate_region_a(self.ram, 4);
        self.runtime.sprite_draw_shadow(self.ram, k)
    }
    pub fn end_sequence_draw_shadow2(&mut self, k: usize) {
        self.credits_sprite_draw_set_shadow_prop(k, 0);
        self.runtime.oam_allocate_region_a(self.ram, 4);
        self.runtime.sprite_draw_shadow(self.ram, k)
    }
    pub fn credits_sprite_draw_add_sparkle(&mut self, count: usize, xb: u8, yb: u8) {
        const D: [u8; 6] = [32, 4, 4, 4, 5, 6];
        self.ram.write_u8(SPRITE_C, count as u8);
        for k in 0..count {
            let mut j = self.ram.read_u8(SPRITE_GRAPHICS + k);
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                j = j.wrapping_add(1);
                if j >= 6 {
                    self.ram.write_u8(SPRITE_X_LO + k, xb);
                    self.ram.write_u8(SPRITE_Y_LO + k, yb);
                    j = 0
                }
                self.ram.write_u8(SPRITE_GRAPHICS + k, j);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, D[j as usize])
            }
            if j != 0 {
                self.credits_sprite_draw_single(k, 1, 0x1c)
            }
        }
    }
    pub fn credits_sprite_draw_single(&mut self, k: usize, a: u8, j: u8) {
        self.runtime
            .oam_allocate_region_a(self.ram, a.wrapping_mul(4));
        self.runtime.sprite_get_16_bit_coords(self.ram, k);
        self.runtime.sprite_draw_multiple(
            self.ram,
            k,
            draw_multiple_data(j >> 1, a, self.ram.read_u8(SPRITE_GRAPHICS + k)),
        )
    }
    pub fn credits_sprite_draw_move_squirrel(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_DELAY_MAIN + k) < 64 {
            self.ram.write_u8(
                SPRITE_C + k,
                self.ram.read_u8(SPRITE_C + k).wrapping_add(1) & 3,
            );
            self.inc8(SPRITE_A + k);
        } else {
            let j = (self.ram.read_u8(SPRITE_C + k) & 3) as usize;
            self.ram.write_i8(SPRITE_X_VEL + k, [32, 24, -32, -24][j]);
            self.ram.write_i8(SPRITE_Y_VEL + k, [8, -8, -8, 8][j]);
            self.runtime.sprite_move_xy(self.ram, k)
        }
    }
    pub fn credits_sprite_draw_circling_birds(&mut self, k: usize) {
        let j = (self.ram.read_u8(SPRITE_D + k) & 1) as usize;
        self.add8(SPRITE_X_VEL + k, if j != 0 { 255 } else { 1 });
        if self.ram.read_u8(SPRITE_X_VEL + k) == [0x20, 0xe0][j] {
            self.inc8(SPRITE_D + k);
        }
        if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
            let j = (self.ram.read_u8(SPRITE_HEAD_DIR + k) & 1) as usize;
            self.add8(SPRITE_Y_VEL + k, if j != 0 { 255 } else { 1 });
            if self.ram.read_u8(SPRITE_Y_VEL + k) == [0x10, 0xf0][j] {
                self.inc8(SPRITE_HEAD_DIR + k);
            }
        }
        self.runtime.sprite_move_xy(self.ram, k)
    }
    pub fn credits_sprite_draw_activate_and_run_sprite(&mut self, k: usize, bytes: u8) {
        self.runtime.oam_allocate_region_a(self.ram, bytes);
        self.runtime.sprite_get_16_bit_coords(self.ram, k);
        let saved = self.ram.read_u8(SUBMODULE);
        self.ram.write_u8(SUBMODULE, 0);
        self.ram.write_u8(SPRITE_STATE + k, 9);
        self.runtime.sprite_active_main(self.ram, k);
        self.ram.write_u8(SUBMODULE, saved);
    }
    pub fn credits_sprite_draw_preexisting_sprite_draw(&mut self, k: usize, bytes: u8) {
        self.runtime.oam_allocate_region_a(self.ram, bytes);
        self.runtime.sprite_get_16_bit_coords(self.ram, k);
        self.runtime.sprite_active_main(self.ram, k);
    }
    pub fn credits_sprite_draw_walk_link_away_from_pedestal(&mut self, k: usize) {
        const DMA: [u16; 8] = [0x16c, 0x16e, 0x170, 0x172, 0x16c, 0x174, 0x176, 0x178];
        if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
            self.ram.write_u8(
                SPRITE_GRAPHICS + k,
                self.ram.read_u8(SPRITE_GRAPHICS + k).wrapping_add(1) & 7,
            );
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 4);
        }
        self.ram.write_u16(
            LINK_DMA_GRAPHICS_INDEX,
            DMA[self.ram.read_u8(SPRITE_GRAPHICS + k) as usize],
        );
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 32);
        self.credits_sprite_draw_single(k, 2, 26);
        self.end_sequence_draw_shadow2(k);
        self.runtime.sprite_move_xy(self.ram, k);
    }
    pub fn end_sequence_32(&mut self) {
        self.external("EnableForceBlank");
        self.external("EraseTileMaps_triforce");
        self.external("TransferFontToVRAM");
        self.credits_load_cool_background();
        self.credits_initialize_polyhedral();
        self.ram.write_u8(INIDISP, 128);
        self.ram.write_u16(0xaa8, 0x200);
        self.ram.write_u8(0xab2, 1);
        self.external("Palette_Load_HUD");
        self.inc8(UPDATE_CGRAM);
        self.ram.write_u16(0xf3e7 + 4 * 2, 0);
        let ds = self.ram.read_u16(0xf403);
        self.ram.write_u16(
            0xf3e7 + 13 * 2,
            self.ram.read_u16(0xf3e7 + 13 * 2).wrapping_add(ds),
        );
        let mut sum = 0u16;
        for i in 0..14 {
            sum = sum.wrapping_add(self.ram.read_u16(0xf3e7 + i * 2));
        }
        self.ram.write_u16(0xf405, sum);
        self.ram.write_u16(0xf403, 0);
        let hp = self
            .runtime
            .health_after_death((self.ram.read_u8(0xf36c) >> 3) as usize);
        self.ram.write_u8(0xf36d, hp);
        self.ram.write_u8(0xf3ca, 0x40);
        self.runtime.save_game_file(self.ram);
        for a in [
            AUX_PALETTE + 76,
            MAIN_PALETTE + 76,
            AUX_PALETTE,
            MAIN_PALETTE,
        ] {
            self.ram.write_u16(a, 0);
        }
        self.ram.write_u8(TM, 0x16);
        self.ram.write_u8(TS, 0);
        self.ram.write_u16(R16, 0x6800);
        self.ram.write_u16(R18, 0);
        self.ram.write_u16(0xcc, 0);
        self.ram.write_u16(BG2_VOFS2, (-0x48i16) as u16);
        self.ram.write_u16(BG2_HOFS2, 0x90);
        self.ram.write_u16(BG3_VOFS2, 0);
        self.ram.write_u16(BG3_HOFS2, 0);
        self.credits_add_next_attribution();
        self.ram.write_u8(MUSIC_CONTROL, 0x22);
        self.ram.write_u8(CGWSEL, 0);
        self.ram.write_u8(CGADSUB, 162);
        self.runtime.ppu_write(0x08, 0x13);
        self.ram.write_u8(COLDATA0, 0x3f);
        self.ram.write_u8(COLDATA0 + 1, 0x5f);
        self.ram.write_u8(COLDATA0 + 2, 0x9f);
        self.ram.write_u8(SUBSUBMODULE, 64);
        self.ram.write_u8(INIDISP, 0);
        self.runtime.credits_hdma_setup(self.ram);
        self.ram.write_u8(HDMAEN, 0x80);
        self.copy_scroll();
    }
    pub fn credits_fade_color_and_begin_animating(&mut self) {
        self.credits_fade_out_fixed_col();
        self.ram.write_u8(NMI_DISABLE_CORE, 1);
        self.credits_animate_the_triangles();
        if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 {
            let x = self.ram.read_u16(BG2_HOFS2).wrapping_add(1);
            self.ram.write_u16(BG2_HOFS2, x);
            if x == 0xc00 {
                self.runtime.ppu_write(0x08, 0x13);
            }
            let a1 = x >> 1;
            let a0 = a1.wrapping_add(x);
            self.ram.write_u16(0x604, a1);
            self.ram.write_u16(0x600, a0);
            self.ram.write_u16(0x602, a0 >> 1);
            self.ram.write_u16(0x606, a1 >> 1);
            let y = self.ram.read_u16(BG3_VOFS2);
            if y == 3288 {
                self.ram.write_u16(R16, 0x80);
                self.inc8(SUBMODULE);
            } else {
                let y = y.wrapping_add(1);
                self.ram.write_u16(BG3_VOFS2, y);
                if y & 7 == 0 {
                    self.ram.write_u16(R18, y >> 3);
                    self.credits_add_next_attribution();
                }
            }
        }
        self.copy_scroll();
    }
    pub fn credits_add_next_attribution(&mut self) {
        const DUNGEON: [usize; 14] = [1, 0, 2, 3, 10, 6, 5, 8, 11, 9, 7, 12, 13, 15];
        const SCROLL: [u16; 14] = [
            0x290, 0x298, 0x2a0, 0x2a8, 0x2b0, 0x2ba, 0x2c2, 0x2ca, 0x2d2, 0x2da, 0x2e2, 0x2ea,
            0x2f2, 0x310,
        ];
        let map = self.runtime.ending_map_data().to_vec();
        let text = self.runtime.ending_credits_text().to_vec();
        let offs = self.runtime.ending_credits_offsets().to_vec();
        if map.len() <= 159 {
            return;
        }
        let mut d = VRAM_UPLOAD_DATA + (self.ram.read_u16(VRAM_UPLOAD_OFFSET) as usize);
        let r16 = self.ram.read_u16(R16);
        let r18 = self.ram.read_u16(R18) as usize;
        self.ram.write_u16(d, r16.swap_bytes());
        self.ram.write_u16(d + 2, 0x3e40);
        self.ram.write_u16(d + 4, map[159]);
        d += 6;
        if r18 < 394 && r18 < offs.len() {
            let mut s = offs[r18] as usize;
            if s < text.len() && text[s] != 0xff {
                self.ram
                    .write_u16(d, r16.wrapping_add(text[s] as u16).swap_bytes());
                s += 1;
                let n = text[s] as usize;
                self.ram.write_u16(d + 2, (n as u16).swap_bytes());
                s += 1;
                d += 4;
                for _ in 0..((n + 1) >> 1) {
                    self.ram.write_u16(d, map[text[s] as usize]);
                    d += 2;
                    s += 1;
                }
            }
            let which = self.ram.read_u16(0xcc) as usize;
            if which & 1 != 0 || which / 2 < SCROLL.len() && r18 as u16 * 2 == SCROLL[which / 2] {
                let t = [0x3ce6, 0x3cf6][which & 1];
                self.ram.write_u16(0xce, t);
                self.ram.write_u16(d, r16.wrapping_add(0x19).swap_bytes());
                self.ram.write_u16(d + 2, 0x500);
                let mut deaths = self.ram.read_u16(0xf3e7 + DUNGEON[which / 2] * 2).min(999);
                self.ram.write_u16(d + 8, t + deaths % 10);
                deaths /= 10;
                self.ram.write_u16(d + 6, t + deaths % 10);
                deaths /= 10;
                self.ram.write_u16(d + 4, t + deaths);
                d += 10;
                self.ram.write_u16(0xcc, (which + 1) as u16);
            }
        }
        let mut next = r16.wrapping_add(0x20);
        if next & 0x3ff == 0 {
            next = (next & 0x6800) ^ 0x800
        }
        self.ram.write_u16(R16, next);
        self.ram
            .write_u16(VRAM_UPLOAD_OFFSET, (d - VRAM_UPLOAD_DATA) as u16);
        self.ram.write_u8(d, 0xff);
        self.ram.write_u8(NMI_LOAD_BG, 1);
    }
    pub fn credits_add_ending_sequence_text(&mut self) {
        let map = self.runtime.ending_map_data().to_vec();
        let text = self.runtime.ending_scene_text().to_vec();
        let offs = self.runtime.ending_scene_offsets().to_vec();
        let scene = (self.ram.read_u8(SUBMODULE) >> 1) as usize;
        if map.len() <= 159 || scene + 1 >= offs.len() {
            return;
        }
        let mut d = VRAM_UPLOAD_DATA;
        self.ram.write_u16(d, 0x60);
        self.ram.write_u16(d + 2, 0xfe47);
        self.ram.write_u16(d + 4, map[159]);
        d += 6;
        let mut s = offs[scene] as usize;
        let end = offs[scene + 1] as usize;
        while s < end {
            self.ram
                .write_u16(d, u16::from_le_bytes([text[s], text[s + 1]]));
            self.ram
                .write_u16(d + 2, u16::from_le_bytes([text[s + 2], text[s + 3]]));
            let n = ((self.ram.read_u16(d + 2) >> 9) & 0x7f) as usize + 1;
            d += 4;
            s += 4;
            for _ in 0..n {
                self.ram.write_u16(d, map[text[s] as usize]);
                d += 2;
                s += 1;
            }
        }
        self.ram
            .write_u16(VRAM_UPLOAD_OFFSET, (d - VRAM_UPLOAD_DATA) as u16);
        self.ram.write_u8(d, 0xff);
        self.ram.write_u8(NMI_LOAD_BG, 1);
    }
    pub fn credits_fade_out_fixed_col(&mut self) {
        if self.dec8(SUBSUBMODULE) == 0 {
            self.ram.write_u8(SUBSUBMODULE, 16);
            for (i, target) in [32, 64, 128].iter().enumerate() {
                if self.ram.read_u8(COLDATA0 + i) != *target {
                    self.dec8(COLDATA0 + i);
                    break;
                }
            }
        }
    }
    pub fn credits_brighten_triangles(&mut self) {
        if self.ram.read_u8(FRAME_COUNTER) & 15 == 0 && self.inc8(INIDISP) == 15 {
            self.inc8(SUBMODULE);
        }
        self.credits_animate_the_triangles()
    }
    pub fn credits_stop_credits_scroll(&mut self) {
        if self.dec8(R16) == 0 {
            self.ram.write_u16(DARKENING, 0);
            self.ram.write_u16(PALETTE_FILTER_COUNTDOWN, 0);
            self.ram.write_u16(MOSAIC_TARGET, 0x1f);
            self.inc8(SUBMODULE);
            self.ram.write_u16(R16, 0xc0);
            self.ram.write_u16(R18, 0)
        }
        self.credits_animate_the_triangles()
    }
    pub fn credits_fade_and_disperse_triangles(&mut self) {
        self.dec8(R16);
        if self.ram.read_u8(R18) == 0 {
            self.external("ApplyPaletteFilter_bounce");
            if self.ram.read_u8(PALETTE_FILTER_COUNTDOWN) != 0 {
                self.credits_animate_the_triangles();
                return;
            }
            self.inc8(R18);
        }
        if self.ram.read_u8(R16) != 0 {
            self.credits_animate_the_triangles();
            return;
        }
        self.inc8(SUBMODULE);
        self.external("PaletteFilter_WishPonds_Inner")
    }
    pub fn credits_fade_in_the_end(&mut self) {
        if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
            self.external("PaletteFilter_SP5F");
            if self.ram.read_u8(PALETTE_FILTER_COUNTDOWN) == 0 {
                self.inc8(SUBMODULE);
            }
        }
        self.credits_hang_forever()
    }
    pub fn credits_hang_forever(&mut self) {
        for (i, &c) in [0, 2, 4, 6].iter().enumerate() {
            oam::set_plain(
                self.ram,
                i,
                (-96i8).wrapping_add((i * 16) as i8) as u8,
                -72i8 as u8,
                c,
                0x3b,
                2,
            )
        }
    }
    pub fn crystal_cutscene_initialize_polyhedral(&mut self) {
        self.ram.write_u8(POLY_CONFIG1, 156);
        self.ram.write_u8(POLY_COLOR_MODE, 1);
        self.ram.write_u8(NMI_THREAD_ACTIVE, 1);
        self.ram.write_u8(INTRO_DID_RUN_STEP, 1);
        self.ram.write_u8(POLY_BASE_X, 32);
        self.ram.write_u8(POLY_BASE_Y, 32);
        self.ram.write_u8(POLY_VAR1, 32);
        self.ram.write_u8(POLY_MODEL, 0);
        self.ram.write_u8(POLY_A, 16);
        self.ram.write_u8(TS, 0);
        self.ram.write_u8(TM, 0x16)
    }
}

const fn dmd(x: i8, y: i8, tile: u16, ext: u8) -> DrawMultipleData {
    DrawMultipleData { x, y, tile, ext }
}

const DMD_OFFSETS: [usize; 33] = [
    0, 12, 18, 34, 46, 54, 64, 67, 77, 78, 82, 86, 92, 113, 129, 135, 141, 144, 146, 148, 156, 162,
    168, 174, 180, 184, 188, 200, 206, 214, 217, 218, 219,
];
const DMD: [DrawMultipleData; 219] = [
    dmd(0, -8, 0x072a, 2),
    dmd(0, -8, 0x072a, 2),
    dmd(0, 0, 0x4fca, 2),
    dmd(0, -8, 0x072a, 2),
    dmd(0, -8, 0x072a, 2),
    dmd(0, 0, 0x0fca, 2),
    dmd(-2, 0, 0x0f77, 0),
    dmd(0, -8, 0x072a, 2),
    dmd(0, 0, 0x4fca, 2),
    dmd(-3, 0, 0x0f66, 0),
    dmd(0, -8, 0x072a, 2),
    dmd(0, 0, 0x4fca, 2),
    dmd(14, -7, 0x0d48, 2),
    dmd(0, -6, 0x0944, 2),
    dmd(0, 0, 0x094e, 2),
    dmd(13, -14, 0x0d48, 2),
    dmd(0, -8, 0x0944, 2),
    dmd(0, 0, 0x0946, 2),
    dmd(-2, -16, 0x3d78, 0),
    dmd(0, -24, 0x3d24, 2),
    dmd(0, -16, 0x3dc2, 2),
    dmd(61, -16, 0x3777, 0),
    dmd(64, -24, 0x37c4, 2),
    dmd(64, -16, 0x77ca, 2),
    dmd(0, -6, 0x326c, 2),
    dmd(64, -6, 0x326c, 2),
    dmd(-2, -16, 0x3d68, 0),
    dmd(0, -24, 0x3d24, 2),
    dmd(0, -16, 0x3dc2, 2),
    dmd(61, -16, 0x3766, 0),
    dmd(64, -24, 0x37c4, 2),
    dmd(64, -16, 0x77ca, 2),
    dmd(0, -6, 0x326c, 2),
    dmd(64, -6, 0x326c, 2),
    dmd(0, 0, 0x0022, 2),
    dmd(48, 0, 0x0064, 2),
    dmd(0, 10, 0x016c, 2),
    dmd(48, 10, 0x016c, 2),
    dmd(0, 0, 0x0064, 2),
    dmd(48, 0, 0x0022, 2),
    dmd(0, 10, 0x016c, 2),
    dmd(48, 10, 0x016c, 2),
    dmd(0, 0, 0x0064, 2),
    dmd(48, 0, 0x0064, 2),
    dmd(0, 10, 0x016c, 2),
    dmd(48, 10, 0x016c, 2),
    dmd(10, 8, 0x8a32, 0),
    dmd(10, 16, 0x8a22, 0),
    dmd(0, -10, 0x0800, 2),
    dmd(0, 0, 0x082c, 2),
    dmd(10, -14, 0x0a22, 0),
    dmd(10, -6, 0x0a32, 0),
    dmd(0, -10, 0x082a, 2),
    dmd(0, 0, 0x0828, 2),
    dmd(10, 16, 0x8a05, 0),
    dmd(10, 8, 0x8a15, 0),
    dmd(-4, 2, 0x0a07, 2),
    dmd(0, -7, 0x0e00, 2),
    dmd(0, 1, 0x0e02, 2),
    dmd(10, -20, 0x0a05, 0),
    dmd(10, -12, 0x0a15, 0),
    dmd(-7, 1, 0x4a07, 2),
    dmd(0, -7, 0x0e00, 2),
    dmd(0, 1, 0x0e02, 2),
    dmd(-6, -2, 0x0706, 2),
    dmd(0, -9, 0x090e, 2),
    dmd(0, -1, 0x0908, 2),
    dmd(0, -10, 0x082a, 2),
    dmd(0, 0, 0x0828, 2),
    dmd(10, 16, 0x8a05, 0),
    dmd(10, 8, 0x8a15, 0),
    dmd(-4, 2, 0x0a07, 2),
    dmd(0, -7, 0x0e00, 2),
    dmd(0, 1, 0x0e02, 2),
    dmd(10, -20, 0x0a05, 0),
    dmd(10, -12, 0x0a15, 0),
    dmd(-7, 1, 0x4a07, 2),
    dmd(0, -19, 0x39af, 0),
    dmd(-16, -24, 0x3704, 2),
    dmd(-16, -16, 0x3764, 2),
    dmd(-16, -24, 0x3762, 2),
    dmd(-16, -16, 0x3764, 2),
    dmd(0, 0, 0x0c0c, 2),
    dmd(0, 0, 0x0c0a, 2),
    dmd(0, 0, 0x0cc5, 2),
    dmd(0, 0, 0x0ce1, 2),
    dmd(1, 4, 0x002a, 0),
    dmd(1, 12, 0x003a, 0),
    dmd(4, 0, 0x0026, 2),
    dmd(0, 9, 0x0024, 2),
    dmd(8, 9, 0x4024, 2),
    dmd(4, 20, 0x016c, 2),
    dmd(0, -7, 0x0d00, 2),
    dmd(0, -7, 0x0d00, 2),
    dmd(0, 0, 0x0d06, 2),
    dmd(0, -7, 0x0d00, 2),
    dmd(0, -7, 0x0d00, 2),
    dmd(0, 0, 0x4d06, 2),
    dmd(0, -8, 0x0d00, 2),
    dmd(0, -8, 0x0d00, 2),
    dmd(0, 0, 0x0d20, 2),
    dmd(0, -8, 0x0d02, 2),
    dmd(0, -8, 0x0d02, 2),
    dmd(0, 0, 0x0d2c, 2),
    dmd(-3, 0, 0x0d2f, 0),
    dmd(0, -7, 0x0d02, 2),
    dmd(0, 0, 0x0d2c, 2),
    dmd(-5, 2, 0x0d2f, 0),
    dmd(0, -8, 0x0d02, 2),
    dmd(0, 0, 0x0d2c, 2),
    dmd(-5, 2, 0x0d3f, 0),
    dmd(0, -8, 0x0d02, 2),
    dmd(0, 0, 0x0d2c, 2),
    dmd(0, -7, 0x0e00, 2),
    dmd(0, 1, 0x4e02, 2),
    dmd(0, -8, 0x0e00, 2),
    dmd(0, 1, 0x0e02, 2),
    dmd(0, -9, 0x0e00, 2),
    dmd(0, 1, 0x0e02, 2),
    dmd(0, -7, 0x0e00, 2),
    dmd(0, 1, 0x0e02, 2),
    dmd(0, -7, 0x0e00, 2),
    dmd(0, 1, 0x4e02, 2),
    dmd(0, -8, 0x0e00, 2),
    dmd(0, 1, 0x4e02, 2),
    dmd(0, -9, 0x0e00, 2),
    dmd(0, 1, 0x4e02, 2),
    dmd(0, -7, 0x0e00, 2),
    dmd(0, 1, 0x4e02, 2),
    dmd(0, 0, 0, 0),
    dmd(0, 0, 0x34c7, 0),
    dmd(0, 0, 0x3480, 0),
    dmd(0, 0, 0x34b6, 0),
    dmd(0, 0, 0x34b7, 0),
    dmd(0, 0, 0x34a6, 0),
    dmd(-3, 17, 0x002b, 0),
    dmd(-3, 25, 0x003b, 0),
    dmd(0, 0, 0x000e, 2),
    dmd(16, 0, 0x400e, 2),
    dmd(0, 16, 0x002e, 2),
    dmd(16, 16, 0x402e, 2),
    dmd(8, 5, 0x0a04, 2),
    dmd(0, 16, 0x0806, 2),
    dmd(16, 16, 0x4806, 2),
    dmd(0, 0, 0x0000, 2),
    dmd(0, 11, 0x0002, 2),
    dmd(0, 0, 0x000e, 2),
    dmd(0, 64, 0x006c, 2),
    dmd(0, 0, 0x0882, 2),
    dmd(0, 7, 0x0a4e, 2),
    dmd(0, 0, 0x4880, 2),
    dmd(0, 7, 0x0a4e, 2),
    dmd(0, 0, 0x0882, 2),
    dmd(0, 7, 0x0a4e, 2),
    dmd(0, 0, 0x0880, 2),
    dmd(0, 7, 0x0a4e, 2),
    dmd(-4, 1, 0x0c68, 0),
    dmd(0, -8, 0x0c40, 2),
    dmd(0, 1, 0x0c42, 2),
    dmd(-4, 1, 0x0c78, 0),
    dmd(0, -8, 0x0c40, 2),
    dmd(0, 1, 0x0c42, 2),
    dmd(8, 5, 0x0679, 0),
    dmd(0, -10, 0x088e, 2),
    dmd(0, 0, 0x066e, 2),
    dmd(0, -10, 0x088e, 2),
    dmd(0, -10, 0x088e, 2),
    dmd(0, 0, 0x066e, 2),
    dmd(11, -3, 0x0869, 0),
    dmd(0, -12, 0x0804, 2),
    dmd(0, 0, 0x0860, 2),
    dmd(10, -3, 0x0867, 0),
    dmd(0, -12, 0x0804, 2),
    dmd(0, 0, 0x0860, 2),
    dmd(-2, 1, 0x0868, 0),
    dmd(0, -8, 0x08c0, 2),
    dmd(0, 0, 0x08c2, 2),
    dmd(-3, 1, 0x0878, 0),
    dmd(0, -8, 0x08c0, 2),
    dmd(0, 0, 0x08c2, 2),
    dmd(0, -10, 0x084c, 2),
    dmd(0, 0, 0x0a6c, 2),
    dmd(0, -9, 0x084c, 2),
    dmd(0, 0, 0x0aa8, 2),
    dmd(0, -7, 0x084a, 2),
    dmd(0, 0, 0x0c6a, 2),
    dmd(0, -7, 0x084a, 2),
    dmd(0, 0, 0x0ca6, 2),
    dmd(-18, -24, 0x39a4, 2),
    dmd(-16, -16, 0x39a8, 2),
    dmd(-18, -24, 0x39a4, 2),
    dmd(-18, -24, 0x39a4, 2),
    dmd(-16, -16, 0x39a6, 2),
    dmd(-18, -24, 0x39a4, 2),
    dmd(-6, -17, 0x392d, 0),
    dmd(-16, -24, 0x39a0, 2),
    dmd(-16, -16, 0x39aa, 2),
    dmd(-5, -17, 0x392c, 0),
    dmd(-16, -24, 0x39a0, 2),
    dmd(-16, -16, 0x39aa, 2),
    dmd(0, -4, 0x30aa, 2),
    dmd(0, -4, 0x30aa, 2),
    dmd(-4, -8, 0x3090, 0),
    dmd(12, -8, 0x7090, 0),
    dmd(-6, -10, 0x3091, 0),
    dmd(14, -10, 0x7091, 0),
    dmd(0, 0, 0x0722, 2),
    dmd(0, -8, 0x09c2, 2),
    dmd(0, 0, 0x4722, 2),
    dmd(0, -8, 0x09c2, 2),
    dmd(0, -9, 0x09c4, 2),
    dmd(0, 0, 0x0722, 2),
    dmd(0, -9, 0x0924, 2),
    dmd(0, 0, 0x0722, 2),
    dmd(-16, -12, 0x3f08, 2),
    dmd(0, -12, 0x3f20, 2),
    dmd(16, -12, 0x3f20, 2),
    dmd(0, 0, 0x0086, 2),
    dmd(0, 0, 0x8060, 2),
];

fn draw_multiple_data(set: u8, count: u8, frame: u8) -> &'static [DrawMultipleData] {
    let set = set as usize;
    let start = DMD_OFFSETS[set] + count as usize * frame as usize;
    &DMD[start..start + count as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;
    struct Rt;
    impl EndingRuntime for Rt {}
    #[test]
    fn triangle_motion_sign_extends_nibbles() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(INTRO_X_LO, 0);
        ram.write_u8(INTRO_X_VEL, 0xff);
        {
            let mut e = Ending {
                ram: &mut ram,
                runtime: &mut rt,
            };
            e.animate_scene_sprite_move_triangle(0)
        }
        assert_eq!(ram.read_u8(INTRO_X_LO), 0xff);
        assert_eq!(ram.read_u8(INTRO_X_SUB), 0xf0);
        assert_eq!(ram.read_u8(INTRO_X_HI), 0xff)
    }
    #[test]
    fn contraction_clamps_at_signed_nibble_limits() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(INTRO_X_LO, 0);
        ram.write_u8(INTRO_X_VEL, 0x10);
        {
            let mut e = Ending {
                ram: &mut ram,
                runtime: &mut rt,
            };
            e.animate_triforce_room_triangle_handle_contracting(0)
        }
        assert_eq!(ram.read_u8(INTRO_X_VEL), 0x10);
        ram.write_u8(INTRO_X_LO, 0xff);
        ram.write_u8(INTRO_X_VEL, 0xf0);
        {
            let mut e = Ending {
                ram: &mut ram,
                runtime: &mut rt,
            };
            e.animate_triforce_room_triangle_handle_contracting(0)
        }
        assert_eq!(ram.read_u8(INTRO_X_VEL), 0xf0)
    }
    #[test]
    fn poly_thread_init_matches_wram_image() {
        let mut b = [0xaa; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        {
            let mut e = Ending {
                ram: &mut ram,
                runtime: &mut rt,
            };
            e.polyhedral_initialize_thread()
        }
        assert_eq!(ram.read_u16(THREAD_OTHER_STACK), 0x1f31);
        assert_eq!(
            ram.slice(0x1f32, 13),
            &[9, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0x30, 0x1d, 0xf8, 9]
        );
        assert!(ram.slice(0x1f00, 10).iter().all(|&x| x == 0))
    }
    #[test]
    fn fade_uses_wrapping_countdown() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(SUBSUBMODULE, 1);
        ram.write_u8(COLDATA0, 0x3f);
        {
            let mut e = Ending {
                ram: &mut ram,
                runtime: &mut rt,
            };
            e.credits_fade_out_fixed_col()
        }
        assert_eq!(ram.read_u8(SUBSUBMODULE), 16);
        assert_eq!(ram.read_u8(COLDATA0), 0x3e)
    }
    #[test]
    fn all_credit_draw_tables_have_exact_c_lengths() {
        let expected = [
            12, 6, 16, 12, 8, 10, 3, 10, 1, 4, 4, 6, 21, 16, 6, 6, 3, 2, 2, 8, 6, 6, 6, 6, 4, 4,
            12, 6, 8, 3, 1, 1,
        ];
        for (i, &len) in expected.iter().enumerate() {
            assert_eq!(DMD_OFFSETS[i + 1] - DMD_OFFSETS[i], len, "table {i}");
        }
        assert_eq!(DMD.len(), 219);
        assert_eq!(DMD[0], dmd(0, -8, 0x072a, 2));
        assert_eq!(DMD[218], dmd(0, 0, 0x8060, 2));
    }
    #[test]
    fn draw_table_frame_selection_matches_pointer_arithmetic() {
        assert_eq!(draw_multiple_data(1, 3, 1), &DMD[15..18]);
        assert_eq!(draw_multiple_data(31, 1, 0), &DMD[218..219]);
    }
}
