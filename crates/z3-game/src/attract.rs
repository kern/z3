//! Attract-mode story sequence, ported from `src/attract.c`.
//!
//! The deliberately overlapping addresses below are the original WRAM aliases.
//! Arithmetic uses wrapping operations wherever the C code relied on 8/16-bit
//! machine overflow.

use crate::{messaging::MessagingAssets, oam, ram::Ram};
use z3_snes::{dma::Dma, ppu::Ppu};

const MAIN_MODULE: usize = 0x10;
const SUBMODULE: usize = 0x11;
const INIDISP: usize = 0x13;
const NMI_LOAD_BG: usize = 0x14;
const UPDATE_CGRAM: usize = 0x15;
const FRAME_COUNTER: usize = 0x1a;
const TM: usize = 0x1c;
const TS: usize = 0x1d;
const TMW: usize = 0x1e;
const TSW: usize = 0x1f;
const ATTRACT_VAR12: usize = 0x20;
const ATTRACT_STATE: usize = 0x22;
const ATTRACT_SEQUENCE: usize = 0x23;
const ATTRACT_VAR10: usize = 0x25;
const ATTRACT_NEXT_LEGEND_GFX: usize = 0x26;
const ATTRACT_LEGEND_FLAG: usize = 0x27;
const ATTRACT_X_BASE: usize = 0x28;
const ATTRACT_Y_BASE: usize = 0x29;
const ATTRACT_OAM_IDX: usize = 0x2a;
const ATTRACT_VAR9: usize = 0x2b;
const ATTRACT_VAR13: usize = 0x2c;
const ATTRACT_VAR7: usize = 0x2d;
const ATTRACT_VRAM_DST: usize = 0x30;
const ATTRACT_VAR1: usize = 0x32;
const ATTRACT_VAR3: usize = 0x33;
const ATTRACT_VAR4: usize = 0x34;
const ATTRACT_X_BASE_HI: usize = 0x40;
const ATTRACT_VAR17: usize = 0x50;
const ATTRACT_VAR21: usize = 0x51;
const ATTRACT_VAR15: usize = 0x52;
const ATTRACT_VAR22: usize = 0x5d;
const LINK_SPEED_SETTING: usize = 0x5e;
const ATTRACT_VAR18: usize = 0x5f;
const ATTRACT_VAR5: usize = 0x60;
const ATTRACT_VAR11: usize = 0x61;
const ATTRACT_VAR19: usize = 0x62;
const ATTRACT_VAR20: usize = 0x63;
const OAM_PRIORITY: usize = 0x64;
const OAM_CUR_PTR: usize = 0x90;
const OAM_EXT_CUR_PTR: usize = 0x92;
const BGMODE: usize = 0x94;
const W12SEL: usize = 0x96;
const W34SEL: usize = 0x97;
const WOBJSEL: usize = 0x98;
const CGWSEL: usize = 0x99;
const CGADSUB: usize = 0x9a;
const HDMAEN: usize = 0x9b;
const COLDATA0: usize = 0x9c;
const COLDATA1: usize = 0x9d;
const COLDATA2: usize = 0x9e;
const SUBSUBMODULE: usize = 0xb0;
const BG2_HOFS2: usize = 0xe2;
const BG3_VOFS2: usize = 0xea;
const BG2_VOFS2: usize = 0xe8;
const JOYPAD1_L_LAST: usize = 0xf2;
const FILTERED_JOYPAD_H: usize = 0xf4;
const FILTERED_JOYPAD_L: usize = 0xf6;
const ATTRACT_ROOM_INDEX: usize = 0x10e;
const BG2_HOFS: usize = 0x11e;
const BG1_HOFS: usize = 0x120;
const BG2_VOFS: usize = 0x122;
const BG1_VOFS: usize = 0x124;
const IRQ_FLAG: usize = 0x128;
const NMI_THREAD_ACTIVE: usize = 0x12a;
const MUSIC_CONTROL: usize = 0x12c;
const SOUND_EFFECT_1: usize = 0x12e;
const SOUND_EFFECT_2: usize = 0x12f;
const ATTRACT_LEGEND_CTR: usize = 0x200;
const HDR_DARK_WITH_LANTERN: usize = 0x458;
const DUNG_NUM_LIT_TORCHES: usize = 0x45a;
const TIMER_MODE7_ZOOM: usize = 0x637;
const M7X: usize = 0x638;
const M7Y: usize = 0x63a;
const NMI_DISABLE_CORE: usize = 0x710;
const MAIN_TILE_THEME: usize = 0xaa1;
const AUX_TILE_THEME: usize = 0xaa2;
const SPRITE_GRAPHICS_INDEX: usize = 0xaa3;
const MISC_SPRITE_GRAPHICS_INDEX: usize = 0xaa4;
const PALETTE_AUX_OR_MAIN: usize = 0xaa8;
const PALETTE_SP0L: usize = 0xaac;
const PALETTE_SP5L: usize = 0xaad;
const PALETTE_SP6L: usize = 0xaae;
const HUD_PALETTE: usize = 0xab2;
const OVERWORLD_PALETTE_MODE: usize = 0xab3;
const PALETTE_MAIN_INDOORS: usize = 0xab6;
const INTRO_TIMES_PAL_FLASH: usize = 0xff9;
const MESSAGING_MODULE: usize = 0x1cd8;
const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;
const INTRO_DID_RUN_STEP: usize = 0x1f00;
const NMI_FLAG_UPDATE_POLY: usize = 0x1f0c;
const VRAM_UPLOAD_DATA: usize = 0x1002;
const SPRITE_OBJ_PRIO: usize = 0xb89;
const SPRITE_Y_LO: usize = 0xd00;
const SPRITE_X_LO: usize = 0xd10;
const SPRITE_Y_HI: usize = 0xd20;
const SPRITE_X_HI: usize = 0xd30;
const SPRITE_GRAPHICS: usize = 0xdc0;
const SPRITE_D: usize = 0xde0;
const SPRITE_TYPE: usize = 0xe20;
const SPRITE_FLAGS2: usize = 0xe40;
const SPRITE_FLAGS3: usize = 0xe60;
const SPRITE_HEAD_DIR: usize = 0xeb0;
const SPRITE_OAM_FLAGS: usize = 0xf50;
const SPRITE_Z: usize = 0xf70;
const CUR_SPRITE_X: usize = 0xfd8;
const CUR_SPRITE_Y: usize = 0xfda;
const MAIN_PALETTE: usize = 0xc500;
const AUX_PALETTE: usize = 0xc300;
const HDMA_TABLE_DYNAMIC: usize = 0x1dba0;

pub const MAP_MODE_ZOOMS1: [u16; 240] = [
    375, 374, 373, 373, 372, 371, 371, 370, 369, 369, 368, 367, 367, 366, 365, 365, 364, 363, 363,
    361, 361, 360, 359, 359, 358, 357, 357, 356, 355, 355, 354, 354, 353, 352, 352, 351, 351, 350,
    349, 349, 348, 348, 347, 346, 346, 345, 345, 344, 343, 343, 342, 342, 341, 341, 340, 339, 339,
    338, 338, 337, 337, 336, 335, 335, 334, 334, 333, 333, 332, 332, 331, 331, 330, 330, 328, 327,
    327, 326, 326, 325, 325, 324, 324, 323, 323, 322, 322, 321, 321, 320, 320, 319, 319, 318, 318,
    317, 317, 316, 316, 315, 315, 314, 314, 313, 313, 312, 312, 311, 311, 310, 310, 309, 309, 309,
    308, 308, 307, 307, 306, 306, 305, 305, 304, 304, 303, 303, 303, 302, 302, 301, 301, 300, 300,
    299, 299, 299, 298, 298, 297, 297, 295, 295, 294, 294, 294, 293, 293, 292, 292, 292, 291, 291,
    290, 290, 289, 289, 289, 288, 288, 287, 287, 287, 286, 286, 285, 285, 285, 284, 284, 283, 283,
    283, 282, 282, 281, 281, 281, 280, 280, 279, 279, 279, 278, 278, 278, 277, 277, 276, 276, 276,
    275, 275, 275, 274, 274, 273, 273, 273, 272, 272, 272, 271, 271, 271, 270, 270, 269, 269, 269,
    268, 268, 268, 267, 267, 267, 266, 266, 266, 265, 265, 265, 264, 264, 264, 263, 263, 262, 262,
    262, 261, 261, 261, 260, 260, 260, 259, 259, 259, 258, 258,
];
pub const MAP_MODE_ZOOMS2: [u16; 240] = [
    136, 136, 135, 135, 135, 135, 135, 134, 134, 134, 133, 133, 133, 133, 132, 132, 132, 132, 132,
    131, 131, 131, 130, 130, 130, 130, 130, 129, 129, 129, 129, 129, 128, 128, 128, 127, 127, 127,
    127, 127, 126, 126, 126, 126, 126, 125, 125, 125, 124, 124, 124, 124, 124, 124, 123, 123, 123,
    123, 123, 122, 122, 122, 121, 121, 121, 121, 121, 121, 120, 120, 120, 120, 120, 120, 119, 119,
    119, 118, 118, 118, 118, 118, 118, 117, 117, 117, 117, 117, 117, 116, 116, 116, 116, 115, 115,
    115, 115, 115, 115, 114, 114, 114, 114, 114, 114, 113, 113, 113, 113, 112, 112, 112, 112, 112,
    112, 112, 111, 111, 111, 111, 111, 111, 110, 110, 110, 110, 110, 109, 109, 109, 109, 109, 109,
    108, 108, 108, 108, 108, 108, 108, 107, 107, 107, 107, 107, 106, 106, 106, 106, 106, 106, 106,
    105, 105, 105, 105, 105, 105, 105, 104, 104, 104, 104, 104, 103, 103, 103, 103, 103, 103, 103,
    103, 102, 102, 102, 102, 102, 102, 102, 101, 101, 101, 101, 101, 101, 100, 100, 100, 100, 100,
    100, 100, 100, 99, 99, 99, 99, 99, 99, 99, 99, 98, 98, 98, 98, 98, 97, 97, 97, 97, 97, 97, 97,
    97, 97, 96, 96, 96, 96, 96, 96, 96, 96, 96, 95, 95, 95, 95, 95, 95, 95, 94, 94, 94, 94, 94, 94,
    94, 94, 94,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttractOamInfo {
    pub x: i8,
    pub y: i8,
    pub chr: u8,
    pub flags: u8,
    pub ext: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HdmaSetupArgs {
    pub table: u32,
    pub indirect: u32,
    pub mode: u8,
    pub first_register: u8,
    pub second_register: u8,
    pub flags: u8,
}
const fn oi(x: i8, y: i8, chr: u8, flags: u8, ext: u8) -> AttractOamInfo {
    AttractOamInfo {
        x,
        y,
        chr,
        flags,
        ext,
    }
}

/// Calls that genuinely belong to other engine modules in the C implementation.
pub trait AttractRuntime {
    fn intro_handle_all_triforce_animations(&mut self, _ram: &mut Ram<'_>) {}
    fn intro_periodic_sword_and_intro_flash(&mut self, _ram: &mut Ram<'_>) {}
    fn force_blank(&mut self, _ram: &mut Ram<'_>) {}
    fn erase_tile_maps(&mut self, _ram: &mut Ram<'_>) {}
    fn load_bg3_gfx(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_hud(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_ow_bg_main(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_link_armor_and_gloves(&mut self, _ram: &mut Ram<'_>) {}
    fn hdma_setup(&mut self, _dma: &mut Dma, _ram: &mut Ram<'_>, _args: HdmaSetupArgs) {}
    fn world_map_load_light_world_map(
        &mut self,
        _dma: &mut Dma,
        _ppu: &mut Ppu,
        _ram: &mut Ram<'_>,
    ) {
    }
    fn load_common_sprites(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_load_entrance(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_load_and_draw_room(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_reset_torch_background_and_player(&mut self, _ram: &mut Ram<'_>) {}
    fn initialize_tilesets(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_bg_and_fixed_color_black(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_sp0l(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_sprite_main(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_sp5l(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_sp6l(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_sprite_environment_dungeon(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_dungeon_set(&mut self, _ram: &mut Ram<'_>) {}
    fn setup_conclusion_hdma(&mut self, _dma: &mut Dma, _ram: &mut Ram<'_>) {}
    fn intro_initialize_memory_darken(&mut self, _ram: &mut Ram<'_>) {}
    fn overworld_load_all_palettes(&mut self, _ram: &mut Ram<'_>) {}
    fn render_text(&mut self, _ram: &mut Ram<'_>, _sram: &mut [u8], _assets: &MessagingAssets<'_>) {
    }
    fn handle_screen_flash(&mut self, _ram: &mut Ram<'_>) {}
    fn fade_music_and_reset_sram_mirror(&mut self, _ram: &mut Ram<'_>) {}
    fn sprite_prep_reset_properties(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_set_x(&mut self, ram: &mut Ram<'_>, k: usize, x: u16) {
        ram.write_u8(SPRITE_X_LO + k, x as u8);
        ram.write_u8(SPRITE_X_HI + k, (x >> 8) as u8);
    }
    fn sprite_set_y(&mut self, ram: &mut Ram<'_>, k: usize, y: u16) {
        ram.write_u8(SPRITE_Y_LO + k, y as u8);
        ram.write_u8(SPRITE_Y_HI + k, (y >> 8) as u8);
    }
    fn sprite_get_16_bit_coords(&mut self, ram: &mut Ram<'_>, k: usize) {
        ram.write_u16(
            CUR_SPRITE_X,
            u16::from(ram.read_u8(SPRITE_X_LO + k)) | u16::from(ram.read_u8(SPRITE_X_HI + k)) << 8,
        );
        ram.write_u16(
            CUR_SPRITE_Y,
            u16::from(ram.read_u8(SPRITE_Y_LO + k)) | u16::from(ram.read_u8(SPRITE_Y_HI + k)) << 8,
        );
    }
    fn guard_handle_all_animation(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
}

pub struct Attract<'a, 'r, 'p, 'm, R: AttractRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub sram: &'a mut [u8],
    pub dma: &'p mut Dma,
    pub ppu: &'p mut Ppu,
    pub messaging_assets: &'m MessagingAssets<'m>,
    pub runtime: &'a mut R,
}

impl<R: AttractRuntime> Attract<'_, '_, '_, '_, R> {
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
    fn set_palette(&mut self, base: usize, i: usize, v: u16) {
        self.ram.write_u16(base + i * 2, v);
    }
    pub fn draw_sprite_set2(&mut self, p: &[AttractOamInfo]) {
        let mut index = 64 + usize::from(self.ram.read_u8(ATTRACT_OAM_IDX));
        self.ram.write_u8(
            ATTRACT_OAM_IDX,
            self.ram
                .read_u8(ATTRACT_OAM_IDX)
                .wrapping_add(p.len() as u8),
        );
        let xb = self.ram.read_u8(ATTRACT_X_BASE);
        let yb = self.ram.read_u8(ATTRACT_Y_BASE);
        for e in p.iter().rev() {
            oam::set_plain(
                self.ram,
                index,
                xb.wrapping_add(e.x as u8),
                yb.wrapping_add(e.y as u8),
                e.chr,
                e.flags,
                e.ext,
            );
            index += 1;
        }
    }

    pub fn module14_attract(&mut self) {
        let mut st = self.ram.read_u8(ATTRACT_STATE);
        if self.ram.read_u8(INIDISP) != 0
            && self.ram.read_u8(INIDISP) != 128
            && st != 0
            && st != 2
            && st != 6
            && self.ram.read_u8(FILTERED_JOYPAD_H) & 0x90 != 0
        {
            st = 9;
            self.ram.write_u8(ATTRACT_STATE, 9);
        }
        match st {
            0 => self.fade(),
            1 => self.init_graphics(),
            2 | 6 => self.fade_out_sequence(),
            3 | 7 => self.load_new_scene(),
            4 => self.fade_in_sequence(),
            5 | 8 => self.enact_story(),
            9 => self.skip_to_file_select(),
            _ => {}
        }
    }
    pub fn fade(&mut self) {
        self.runtime.intro_handle_all_triforce_animations(self.ram);
        self.ram.write_u8(INTRO_DID_RUN_STEP, 0);
        self.ram.write_u8(NMI_THREAD_ACTIVE, 0);
        self.runtime.intro_periodic_sword_and_intro_flash(self.ram);
        if self.ram.read_u8(INIDISP) != 0 {
            self.dec8(INIDISP);
            return;
        }
        self.runtime.force_blank(self.ram);
        self.ram.write_u8(IRQ_FLAG, 255);
        self.ram.write_u8(NMI_THREAD_ACTIVE, 0);
        self.ram.write_u8(NMI_FLAG_UPDATE_POLY, 0);
        self.inc8(ATTRACT_STATE);
    }
    pub fn init_graphics(&mut self) {
        self.ram.slice_mut(ATTRACT_VAR12, 0x51).fill(0);
        self.runtime.erase_tile_maps(self.ram);
        self.runtime.load_bg3_gfx(self.ram);
        self.ram.write_u8(OVERWORLD_PALETTE_MODE, 4);
        self.ram.write_u8(HUD_PALETTE, 1);
        self.ram.write_u16(PALETTE_AUX_OR_MAIN, 0);
        self.runtime.palette_load_hud(self.ram);
        self.ram.write_u16(PALETTE_AUX_OR_MAIN, 0x200);
        self.runtime.palette_load_ow_bg_main(self.ram);
        self.runtime.palette_load_hud(self.ram);
        self.runtime.palette_load_link_armor_and_gloves(self.ram);
        self.set_palette(MAIN_PALETTE, 0x1d, 0x3800);
        self.inc8(UPDATE_CGRAM);
        self.ram.write_u8(BG3_VOFS2, 20);
        self.build_backgrounds();
        self.ram.write_u8(MESSAGING_MODULE, 0);
        self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x112);
        self.ram.write_u16(BG2_VOFS2, 0);
        self.ram.write_u16(ATTRACT_LEGEND_CTR, 0x1010);
        self.ram.write_u8(
            ATTRACT_STATE,
            self.ram.read_u8(ATTRACT_STATE).wrapping_add(3),
        );
        self.runtime.hdma_setup(
            self.dma,
            self.ram,
            HdmaSetupArgs {
                table: 0xcfa87,
                indirect: 0xcfa94,
                mode: 1,
                first_register: 0x26,
                second_register: 0x28,
                flags: 0,
            },
        );
        self.ram.write_u8(HDMAEN, 0xc0);
        self.ram.write_u8(W12SEL, 0);
        self.ram.write_u8(W34SEL, 0);
        self.ram.write_u8(WOBJSEL, 0xb0);
        self.ram.write_u8(TMW, 3);
        self.ram.write_u8(TSW, 0);
        self.ram.write_u8(COLDATA0, 0x25);
        self.ram.write_u8(COLDATA1, 0x45);
        self.ram.write_u8(COLDATA2, 0x85);
        self.ram.write_u8(CGWSEL, 0x10);
        self.ram.write_u8(CGADSUB, 0xa3);
        self.ram.write_u8(MUSIC_CONTROL, 6);
        self.inc8(ATTRACT_LEGEND_FLAG);
    }
    pub fn fade_in_step(&mut self) {
        if self.ram.read_u8(INIDISP) != 15 {
            if (self.dec8(LINK_SPEED_SETTING) as i8) < 0 {
                self.inc8(INIDISP);
                self.ram.write_u8(LINK_SPEED_SETTING, 1);
            }
        } else {
            self.inc8(ATTRACT_VAR18);
        }
    }
    pub fn fade_in_sequence(&mut self) {
        if self.ram.read_u8(INIDISP) != 15 {
            if (self.dec8(LINK_SPEED_SETTING) as i8) < 0 {
                self.inc8(INIDISP);
                self.ram.write_u8(LINK_SPEED_SETTING, 1);
            }
        } else {
            self.inc8(ATTRACT_STATE);
        }
    }
    pub fn fade_out_sequence(&mut self) {
        if self.ram.read_u8(INIDISP) != 0 {
            if (self.dec8(LINK_SPEED_SETTING) as i8) < 0 {
                self.dec8(INIDISP);
                self.ram.write_u8(LINK_SPEED_SETTING, 1);
            }
        } else {
            self.runtime.force_blank(self.ram);
            self.runtime.erase_tile_maps(self.ram);
            self.inc8(ATTRACT_STATE);
        }
    }
    pub fn load_new_scene(&mut self) {
        match self.ram.read_u8(ATTRACT_SEQUENCE) {
            0 => self.scene_polka_dots(),
            1 => self.scene_world_map(),
            2 => self.scene_throne_room(),
            3 => self.prep_zelda_prison(),
            4 => self.prep_maiden_warp(),
            5 => self.scene_end_of_story(),
            _ => {}
        }
    }
    pub fn scene_polka_dots(&mut self) {
        self.ram.write_u8(ATTRACT_NEXT_LEGEND_GFX, 0);
        self.inc8(ATTRACT_STATE);
        self.ram.write_u8(INIDISP, 0);
    }
    pub fn scene_world_map(&mut self) {
        self.ppu.write(0x07, 0x13);
        self.ppu.write(0x08, 3);
        self.ram.write_u8(CGWSEL, 0x80);
        self.ram.write_u8(CGADSUB, 0x21);
        self.ram.write_u8(BGMODE, 7);
        self.runtime
            .world_map_load_light_world_map(self.dma, self.ppu, self.ram);
        self.ram.write_u16(M7Y, 0xed);
        self.ram.write_u16(M7X, 0x100);
        self.ram.write_u16(BG1_HOFS, 0x80);
        self.ram.write_u16(BG1_VOFS, 0xc0);
        self.ram.write_u8(TIMER_MODE7_ZOOM, 255);
        self.control_map_zoom();
        self.ram.write_u8(ATTRACT_VAR10, 1);
        self.inc8(ATTRACT_STATE);
        self.ram.write_u8(INIDISP, 0);
    }
    pub fn dungeon_load_and_draw_entrance_room(&mut self, a: u8) {
        self.ram.write_u8(ATTRACT_ROOM_INDEX, a);
        self.runtime.dungeon_load_entrance(self.ram);
        self.ram.write_u8(DUNG_NUM_LIT_TORCHES, 0);
        self.ram.write_u8(HDR_DARK_WITH_LANTERN, 0);
        self.runtime.dungeon_load_and_draw_room(self.ram);
        self.runtime
            .dungeon_reset_torch_background_and_player(self.ram);
    }
    pub fn dungeon_save_and_load_all_palettes(&mut self, a: u8, k: u8) {
        self.ram.write_u8(SPRITE_GRAPHICS_INDEX, k);
        self.ram.write_u8(MAIN_TILE_THEME, a);
        self.ram.write_u8(AUX_TILE_THEME, a);
        self.runtime.initialize_tilesets(self.ram);
        self.ram.write_u16(PALETTE_AUX_OR_MAIN, 0x200);
        self.inc8(UPDATE_CGRAM);
        self.runtime.palette_bg_and_fixed_color_black(self.ram);
        self.palette_load_dungeon_bundle();
    }
    fn palette_load_dungeon_bundle(&mut self) {
        self.runtime.palette_load_sp0l(self.ram);
        self.runtime.palette_load_sprite_main(self.ram);
        self.runtime.palette_load_sp5l(self.ram);
        self.runtime.palette_load_sp6l(self.ram);
        self.runtime
            .palette_load_sprite_environment_dungeon(self.ram);
        self.runtime.palette_load_hud(self.ram);
        self.runtime.palette_load_dungeon_set(self.ram);
    }
    pub fn scene_throne_room(&mut self) {
        self.ram.write_u8(HDMAEN, 0);
        self.ram.write_u8(CGWSEL, 2);
        self.ram.write_u8(CGADSUB, 0x20);
        self.ram.write_u8(MISC_SPRITE_GRAPHICS_INDEX, 10);
        self.runtime.load_common_sprites(self.ram);
        let b0 = self.ram.read_u16(ATTRACT_VAR12);
        let b1 = self.ram.read_u16(ATTRACT_STATE);
        self.dungeon_load_and_draw_entrance_room(0x74);
        self.ram.write_u16(ATTRACT_STATE, b1);
        self.ram.write_u16(ATTRACT_VAR12, b0);
        self.ram.write_u8(PALETTE_MAIN_INDOORS, 0);
        self.ram.write_u8(PALETTE_SP0L, 0);
        self.ram.write_u8(PALETTE_SP5L, 14);
        self.ram.write_u8(PALETTE_SP6L, 3);
        self.dungeon_save_and_load_all_palettes(0, 0x7e);
        self.set_palette(MAIN_PALETTE, 0x1d, 0x3800);
        self.ram.write_u8(MESSAGING_MODULE, 0);
        self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x113);
        self.ram.write_u8(ATTRACT_VAR10, 2);
        self.ram.write_u8(ATTRACT_VAR13, 0xe0);
        self.ram.write_u16(OAM_PRIORITY, 0x210);
        self.prep_finish();
    }
    pub fn prep_finish(&mut self) {
        self.inc8(ATTRACT_STATE);
        self.ram.write_u8(INIDISP, 0);
        self.ram.write_u8(BG3_VOFS2, 0);
        for a in [BG2_HOFS, BG2_VOFS, BG2_HOFS2, BG2_VOFS2] {
            self.ram.write_u16(a, self.ram.read_u16(a) & 0x1ff);
        }
    }
    pub fn prep_zelda_prison(&mut self) {
        self.ram.write_u8(CGWSEL, 0);
        self.ram.write_u8(CGADSUB, 0);
        let b0 = self.ram.read_u16(ATTRACT_VAR12);
        let b1 = self.ram.read_u16(ATTRACT_STATE);
        self.dungeon_load_and_draw_entrance_room(0x73);
        self.ram.write_u16(ATTRACT_STATE, b1);
        self.ram.write_u16(ATTRACT_VAR12, b0);
        self.ram.write_u8(PALETTE_MAIN_INDOORS, 2);
        self.ram.write_u8(PALETTE_SP0L, 0);
        self.ram.write_u8(PALETTE_SP5L, 14);
        self.ram.write_u8(PALETTE_SP6L, 3);
        self.dungeon_save_and_load_all_palettes(1, 0x7f);
        self.set_palette(MAIN_PALETTE, 0x1d, 0x3800);
        self.ram.write_u8(MESSAGING_MODULE, 0);
        self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x114);
        for (a, v) in [
            (ATTRACT_VAR9, 148),
            (ATTRACT_VAR1, 0),
            (ATTRACT_VAR3, 0),
            (ATTRACT_X_BASE_HI, 0),
            (ATTRACT_VAR17, 0),
            (ATTRACT_VAR18, 0),
            (ATTRACT_VAR10, 255),
        ] {
            self.ram.write_u8(a, v);
        }
        self.ram.write_u16(ATTRACT_VRAM_DST, 0x68);
        self.ram.write_u16(OAM_PRIORITY, 0x240);
        self.prep_finish();
    }
    pub fn prep_maiden_warp(&mut self) {
        let b0 = self.ram.read_u16(ATTRACT_VAR12);
        let b1 = self.ram.read_u16(ATTRACT_STATE);
        self.dungeon_load_and_draw_entrance_room(0x75);
        self.ram.write_u16(ATTRACT_STATE, b1);
        self.ram.write_u16(ATTRACT_VAR12, b0);
        self.ram.write_u8(PALETTE_MAIN_INDOORS, 0);
        self.ram.write_u8(PALETTE_SP0L, 0);
        self.ram.write_u8(PALETTE_SP5L, 14);
        self.ram.write_u8(PALETTE_SP6L, 3);
        self.ram.write_u16(PALETTE_AUX_OR_MAIN, 0);
        self.palette_load_dungeon_bundle();
        self.dungeon_save_and_load_all_palettes(2, 0x7f);
        self.set_palette(AUX_PALETTE, 0x1d, 0x3800);
        self.set_palette(MAIN_PALETTE, 0x1d, 0x3800);
        self.ram.write_u8(MESSAGING_MODULE, 0);
        self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x115);
        self.ram.write_u8(ATTRACT_VAR10, 255);
        self.ram.write_u8(ATTRACT_VRAM_DST, 112);
        for (a, v) in [
            (ATTRACT_VAR19, 112),
            (ATTRACT_VAR20, 112),
            (ATTRACT_VAR1, 8),
            (ATTRACT_VAR17, 0),
            (ATTRACT_VAR21, 0),
            (ATTRACT_VAR15, 0),
            (ATTRACT_VAR18, 0),
            (ATTRACT_VAR5, 0),
            (ATTRACT_VAR11, 0),
        ] {
            self.ram.write_u8(a, v);
        }
        self.ram.write_u16(OAM_PRIORITY, 0xc0);
        self.prep_finish();
    }
    pub fn scene_end_of_story(&mut self) {
        self.runtime.setup_conclusion_hdma(self.dma, self.ram);
        self.death_func31();
    }
    pub fn death_func31(&mut self) {
        self.inc8(NMI_DISABLE_CORE);
        self.runtime.intro_initialize_memory_darken(self.ram);
        self.runtime.overworld_load_all_palettes(self.ram);
        self.ram.write_u8(BG3_VOFS2, 0);
        for a in [M7Y, M7X, BG1_HOFS, BG1_VOFS, BG2_HOFS, BG2_VOFS] {
            self.ram.write_u16(a, 0);
        }
        self.ram.write_u8(MUSIC_CONTROL, 0xf1);
        self.ram.write_u8(ATTRACT_SEQUENCE, 0);
        self.ram.write_u8(MAIN_MODULE, 0);
        self.ram.write_u8(SUBMODULE, 10);
        self.ram.write_u8(SUBSUBMODULE, 10);
    }
    pub fn enact_story(&mut self) {
        match self.ram.read_u8(ATTRACT_SEQUENCE) {
            0 => self.dramatize_polka_dots(),
            1 => self.dramatize_world_map(),
            2 => self.throne_room(),
            3 => self.dramatize_prison(),
            4 => self.dramatize_agahnim_altar(),
            _ => {}
        }
    }
    pub fn zelda_prison_case0(&mut self) {
        const A: [AttractOamInfo; 6] = [
            oi(5, 25, 0x6c, 0x38, 2),
            oi(11, 25, 0x6c, 0x38, 2),
            oi(0, 0, 0x84, 0x3b, 2),
            oi(16, 0, 0x84, 0x7b, 2),
            oi(0, 16, 0xa4, 0x3b, 2),
            oi(16, 16, 0xa4, 0x7b, 2),
        ];
        if self.ram.read_u8(ATTRACT_VAR4) == 0 {
            self.inc8(ATTRACT_VAR5);
        }
        if self.ram.read_u8(FRAME_COUNTER) & 1 != 0 {
            let v = self.ram.read_u16(ATTRACT_VRAM_DST).wrapping_sub(1);
            self.ram.write_u16(ATTRACT_VRAM_DST, v);
        }
        self.ram.write_u8(ATTRACT_X_BASE, 0x58);
        self.ram
            .write_u8(ATTRACT_Y_BASE, self.ram.read_u8(ATTRACT_VAR9));
        self.draw_sprite_set2(&A);
        self.ram.write_u16(ATTRACT_VAR7, 0xf8d9);
    }
    pub fn zelda_prison_case1(&mut self) {
        const A: [AttractOamInfo; 30] = [
            oi(5, 25, 0x6c, 0x38, 2),
            oi(11, 25, 0x6c, 0x38, 2),
            oi(0, 0, 0x84, 0x3b, 2),
            oi(16, 0, 0x84, 0x7b, 2),
            oi(0, 16, 0xa4, 0x3b, 2),
            oi(16, 16, 0xa4, 0x7b, 2),
            oi(5, 25, 0x6c, 0x38, 2),
            oi(11, 25, 0x6c, 0x38, 2),
            oi(0, 0, 0xc4, 0x3b, 2),
            oi(16, 0, 0xc2, 0x3b, 2),
            oi(0, 16, 0xe4, 0x3b, 2),
            oi(16, 16, 0xe6, 0x3b, 2),
            oi(5, 25, 0x6c, 0x38, 2),
            oi(11, 25, 0x6c, 0x38, 2),
            oi(0, 0, 0x88, 0x3b, 2),
            oi(16, 0, 0x8a, 0x3b, 2),
            oi(0, 16, 0xa8, 0x3b, 2),
            oi(16, 16, 0xaa, 0x3b, 2),
            oi(5, 25, 0x6c, 0x38, 2),
            oi(11, 25, 0x6c, 0x38, 2),
            oi(0, 0, 0x82, 0x3b, 2),
            oi(16, 0, 0x82, 0x7b, 2),
            oi(0, 16, 0xa2, 0x3b, 2),
            oi(16, 16, 0xa2, 0x7b, 2),
            oi(5, 25, 0x6c, 0x38, 2),
            oi(11, 25, 0x6c, 0x38, 2),
            oi(0, 0, 0x80, 0x3b, 2),
            oi(16, 0, 0x80, 0x7b, 2),
            oi(0, 16, 0xa0, 0x3b, 2),
            oi(16, 16, 0xa0, 0x7b, 2),
        ];
        let k;
        if self.ram.read_u8(ATTRACT_VAR10) < 0x80 {
            self.show_timed_text_message();
            if self.ram.read_u16(OAM_PRIORITY) != 0 {
                k = 4;
            } else if self.ram.read_u8(ATTRACT_VAR9) != 0x6e {
                self.dec8(ATTRACT_VAR9);
                k = 0;
            } else {
                k = self.prison_case1_countdown();
                if k == usize::MAX {
                    return;
                }
            }
        } else if self.ram.read_u8(ATTRACT_VAR9) != 0x6e {
            self.dec8(ATTRACT_VAR9);
            k = 0;
        } else {
            k = self.prison_case1_countdown();
            if k == usize::MAX {
                return;
            }
        }
        if self.ram.read_u8(FRAME_COUNTER) & 1 != 0 {
            self.ram.write_u16(
                ATTRACT_VRAM_DST,
                self.ram.read_u16(ATTRACT_VRAM_DST).wrapping_sub(1),
            );
        }
        self.ram.write_u8(ATTRACT_X_BASE, 0x58);
        self.ram
            .write_u8(ATTRACT_Y_BASE, self.ram.read_u8(ATTRACT_VAR9));
        self.draw_sprite_set2(&A[k * 6..k * 6 + 6]);
    }
    fn prison_case1_countdown(&mut self) -> usize {
        let v = self.ram.read_u8(ATTRACT_VAR10);
        if v < 31 && v & 1 == 0 {
            self.dec8(INIDISP);
        }
        let n = self.dec8(ATTRACT_VAR10);
        if n == 0 {
            self.inc8(ATTRACT_SEQUENCE);
            self.ram.write_u8(
                ATTRACT_STATE,
                self.ram.read_u8(ATTRACT_STATE).wrapping_sub(2),
            );
            return usize::MAX;
        }
        if n >= 0xc0 {
            0
        } else if n >= 0xb8 {
            1
        } else if n >= 0xb0 {
            2
        } else if n >= 0xa0 {
            3
        } else {
            4
        }
    }
    pub fn zelda_prison_draw_a(&mut self) {
        let i = 64 + usize::from(self.ram.read_u8(ATTRACT_OAM_IDX));
        let ext = if self.ram.read_u8(ATTRACT_X_BASE_HI) != 0 {
            3
        } else {
            2
        };
        let j = (self.ram.read_u8(ATTRACT_VAR1) >> 3) & 1;
        let x = self.ram.read_u8(ATTRACT_X_BASE);
        let y = self.ram.read_u8(ATTRACT_Y_BASE);
        oam::set_plain(self.ram, i, x, y.wrapping_add(j), 6, 0x3d, ext);
        oam::set_plain(
            self.ram,
            i + 1,
            x,
            y.wrapping_add(10),
            if j != 0 { 10 } else { 8 },
            0x3d,
            ext,
        );
        self.ram.write_u8(
            ATTRACT_OAM_IDX,
            self.ram.read_u8(ATTRACT_OAM_IDX).wrapping_add(2),
        );
    }
    pub fn maiden_warp_case0(&mut self) {
        if self.ram.read_u8(ATTRACT_VAR11) != 0 {
            self.inc8(ATTRACT_VAR5);
        }
    }
    pub fn maiden_warp_case1(&mut self) {
        const A: [AttractOamInfo; 28] = WARP_OAM;
        const N: [usize; 8] = [2, 2, 2, 6, 6, 10, 10, 14];
        let k = usize::from(self.ram.read_u8(FRAME_COUNTER) >> 2 & 1);
        let n = N[usize::from(self.ram.read_u8(ATTRACT_VAR21) >> 1 & 7)];
        self.ram.write_u8(ATTRACT_X_BASE, 110);
        self.ram.write_u8(ATTRACT_Y_BASE, 72);
        self.draw_sprite_set2(&A[k * 14..k * 14 + n]);
        if self.ram.read_u8(ATTRACT_VAR21) == 0 && self.ram.read_u8(ATTRACT_VAR20) == 0x70 {
            self.ram.write_u8(SOUND_EFFECT_2, 0x27);
        }
        if self.ram.read_u8(ATTRACT_VAR21) == 15 {
            self.inc8(ATTRACT_VAR5);
        } else {
            if self.ram.read_u8(ATTRACT_VAR21) == 6 {
                self.ram.write_u8(INTRO_TIMES_PAL_FLASH, 0x90);
                self.ram.write_u8(SOUND_EFFECT_2, 0x2b);
            }
            // Preserve the duplicated nested condition from the C source. The
            // inner `else` is unreachable whenever the outer condition runs.
            if self.ram.read_u8(ATTRACT_VAR20) != 0 && self.ram.read_u8(ATTRACT_VAR20) != 0 {
                self.dec8(ATTRACT_VAR20);
            }
        }
    }
    pub fn maiden_warp_case2(&mut self) {
        const N: [usize; 8] = [4, 4, 8, 8, 12, 12, 14, 14];
        self.ram.write_u8(ATTRACT_X_BASE, 110);
        self.ram.write_u8(ATTRACT_Y_BASE, 72);
        let k = usize::from(self.ram.read_u8(FRAME_COUNTER) >> 2 & 1);
        let n = N[usize::from(self.ram.read_u8(ATTRACT_VAR21) >> 1 & 7)];
        self.draw_sprite_set2(&WARP_OAM[k * 14 + 14 - n..k * 14 + 14]);
        if self.ram.read_u8(ATTRACT_VAR21) == 0 {
            if self.dec8(ATTRACT_VAR19) == 0 {
                self.inc8(ATTRACT_VAR5);
            }
        } else {
            self.dec8(ATTRACT_VAR21);
        }
    }
    pub fn maiden_warp_case3(&mut self) {
        const A: [AttractOamInfo; 3] = [
            oi(0, 0, 0xc6, 0x3d, 2),
            oi(0, 0, 0x24, 0x35, 2),
            oi(16, 0, 0x24, 0x75, 2),
        ];
        let v = self.ram.read_u8(ATTRACT_VAR21);
        if v == 6 {
            self.inc8(ATTRACT_VAR15);
            self.ram.write_u8(SOUND_EFFECT_1, 51);
        } else if v == 0x40 {
            self.ram.write_u8(ATTRACT_VAR21, 224);
            self.inc8(ATTRACT_VAR5);
        } else if v < 0xf {
            let k = usize::from(v >> 3 & 1);
            self.ram.write_u8(ATTRACT_X_BASE, [0x78, 0x70][k]);
            self.ram.write_u8(ATTRACT_Y_BASE, 0x60);
            self.draw_sprite_set2(if k != 0 { &A[1..3] } else { &A[0..1] });
        }
        self.inc8(ATTRACT_VAR21);
    }
    pub fn maiden_warp_case4(&mut self) {
        self.show_timed_text_message();
        if self.ram.read_u16(OAM_PRIORITY) == 0 {
            let v = self.ram.read_u8(ATTRACT_VAR21);
            if v < 31 && v & 1 == 0 {
                self.dec8(INIDISP);
            }
            if self.dec8(ATTRACT_VAR21) == 0 {
                self.inc8(ATTRACT_VAR22);
            }
        }
    }
    pub fn dramatize_polka_dots(&mut self) {
        if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 {
            for a in [BG1_VOFS, BG1_HOFS, BG2_VOFS] {
                let v = self.ram.read_u8(a).wrapping_add(1);
                self.ram.write_u8(a, v);
            }
            self.ram
                .write_u8(BG2_HOFS, self.ram.read_u8(BG2_HOFS).wrapping_sub(1));
        }
        if self.ram.read_u8(ATTRACT_LEGEND_FLAG) != 0 {
            self.build_next_image_tile_map();
            self.ram.write_u8(ATTRACT_LEGEND_FLAG, 0);
            self.ram.write_u8(
                ATTRACT_NEXT_LEGEND_GFX,
                self.ram.read_u8(ATTRACT_NEXT_LEGEND_GFX).wrapping_add(2),
            );
        }
        self.ram.write_u8(JOYPAD1_L_LAST, 0);
        self.ram.write_u8(FILTERED_JOYPAD_L, 0);
        self.ram.write_u8(FILTERED_JOYPAD_H, 0);
        self.runtime
            .render_text(self.ram, self.sram, self.messaging_assets);
        let ctr = self.ram.read_u16(ATTRACT_LEGEND_CTR).wrapping_sub(1);
        self.ram.write_u16(ATTRACT_LEGEND_CTR, ctr);
        if ctr == 0 {
            self.inc8(ATTRACT_SEQUENCE);
            self.ram.write_u8(
                ATTRACT_STATE,
                self.ram.read_u8(ATTRACT_STATE).wrapping_sub(3),
            );
        } else if ctr < 0x18 && ctr & 1 != 0 {
            self.dec8(INIDISP);
        }
    }
    pub fn dramatize_world_map(&mut self) {
        if self.ram.read_u8(TIMER_MODE7_ZOOM) != 0 {
            if self.ram.read_u8(TIMER_MODE7_ZOOM) < 15 {
                self.dec8(INIDISP);
            }
            if self.dec8(ATTRACT_VAR10) == 0 {
                self.ram.write_u8(ATTRACT_VAR10, 1);
                self.dec8(TIMER_MODE7_ZOOM);
                self.control_map_zoom();
            }
        } else {
            self.runtime.force_blank(self.ram);
            self.ram.write_u8(BGMODE, 9);
            self.runtime.erase_tile_maps(self.ram);
            self.inc8(ATTRACT_SEQUENCE);
            self.ram.write_u8(
                ATTRACT_STATE,
                self.ram.read_u8(ATTRACT_STATE).wrapping_sub(2),
            );
        }
    }
    pub fn throne_room(&mut self) {
        const A: [AttractOamInfo; 10] = [
            oi(16, 16, 0x2a, 0x7b, 2),
            oi(0, 16, 0x2a, 0x3b, 2),
            oi(16, 0, 0x0a, 0x7b, 2),
            oi(0, 0, 0x0a, 0x3b, 2),
            oi(0, 0, 0x0c, 0x31, 2),
            oi(16, 0, 0x0e, 0x31, 2),
            oi(32, 0, 0x0c, 0x71, 2),
            oi(0, 16, 0x2c, 0x31, 2),
            oi(16, 16, 0x2e, 0x31, 2),
            oi(32, 16, 0x2c, 0x71, 2),
        ];
        const OFF: [usize; 3] = [0, 4, 10];
        self.ram.write_u8(ATTRACT_OAM_IDX, 0);
        if self.ram.read_u8(ATTRACT_VAR15) == 0 {
            if self.ram.read_u8(INIDISP) != 15 {
                self.inc8(INIDISP);
            } else {
                self.inc8(ATTRACT_VAR15);
            }
        }
        if self.ram.read_u16(BG2_VOFS) == 0 {
            self.show_timed_text_message();
            if self.ram.read_u16(OAM_PRIORITY) == 0 {
                let v = self.ram.read_u8(ATTRACT_VAR13);
                if v < 31 && v & 1 == 0 {
                    self.dec8(INIDISP);
                }
                if self.dec8(ATTRACT_VAR13) == 0 {
                    self.inc8(ATTRACT_SEQUENCE);
                    self.inc8(ATTRACT_STATE);
                    return;
                }
            }
        } else {
            self.ram
                .write_u16(BG2_VOFS, self.ram.read_u16(BG2_VOFS).wrapping_sub(1));
            self.ram
                .write_u16(BG1_VOFS, self.ram.read_u16(BG1_VOFS).wrapping_sub(1));
        }
        for i in (0..=1).rev() {
            let y = ([88_u16, 32][i]).wrapping_sub(self.ram.read_u16(BG2_VOFS));
            if (y.wrapping_add(32) as i16) >= 0 {
                self.ram.write_u8(ATTRACT_X_BASE, [80, 104][i]);
                self.ram.write_u8(ATTRACT_Y_BASE, y as u8);
                self.draw_sprite_set2(&A[OFF[i]..OFF[i + 1]]);
            }
        }
        self.ram.write_u16(ATTRACT_VAR7, 0xf8a7);
    }
    pub fn dramatize_prison(&mut self) {
        const TAB: [u8; 16] = [0, 1, 2, 3, 4, 5, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1];
        self.ram.write_u8(ATTRACT_OAM_IDX, 0);
        if self.ram.read_u8(ATTRACT_VAR18) == 0 {
            self.fade_in_step();
        }
        self.ram.write_u8(ATTRACT_X_BASE, 56);
        self.draw_zelda();
        if self.ram.read_u8(ATTRACT_VAR10) >= 192 {
            self.ram.write_u8(ATTRACT_Y_BASE, 112);
            if (self.dec8(ATTRACT_VAR17) as i8) < 0 {
                self.ram.write_u8(ATTRACT_VAR17, 0xf);
            }
            let t = self
                .ram
                .read_u16(ATTRACT_VRAM_DST)
                .wrapping_add(u16::from(TAB[usize::from(self.ram.read_u8(ATTRACT_VAR17))]));
            self.ram.write_u8(ATTRACT_X_BASE_HI, (t >> 8) as u8);
            self.ram.write_u8(ATTRACT_X_BASE, t as u8);
            self.zelda_prison_draw_a();
            for k in (0..=1).rev() {
                self.runtime.sprite_prep_reset_properties(self.ram, k * 2);
                let x = ([32_i16, -12][k] as u16)
                    .wrapping_add(self.ram.read_u16(ATTRACT_VRAM_DST))
                    .wrapping_add(0x100);
                self.ram.write_u8(ATTRACT_VAR4, x as u8);
                self.sprite_simulate_soldier(
                    k * 2,
                    x,
                    112 + 24,
                    1,
                    [9, 7][k],
                    self.ram.read_u8(ATTRACT_VAR3),
                );
            }
            let a = self.inc8(ATTRACT_VAR1);
            if a & 7 == 0 {
                if self.ram.read_u8(ATTRACT_VAR3) == 2 {
                    self.ram.write_u8(ATTRACT_VAR3, 0xff);
                    if self.ram.read_u8(ATTRACT_VRAM_DST + 1) == 0 && a & 8 != 0 {
                        self.ram.write_u8(SOUND_EFFECT_2, 4);
                    }
                }
                self.inc8(ATTRACT_VAR3);
            }
        }
        match self.ram.read_u8(ATTRACT_VAR5) {
            0 => self.zelda_prison_case0(),
            1 => self.zelda_prison_case1(),
            _ => {}
        }
    }
    pub fn dramatize_agahnim_altar(&mut self) {
        if self.ram.read_u8(ATTRACT_VAR22) != 0 {
            self.inc8(ATTRACT_SEQUENCE);
            self.ram.write_u8(
                ATTRACT_STATE,
                self.ram.read_u8(ATTRACT_STATE).wrapping_sub(2),
            );
            return;
        }
        self.ram.write_u8(ATTRACT_OAM_IDX, 0);
        self.runtime.handle_screen_flash(self.ram);
        if self.ram.read_u8(ATTRACT_VAR18) == 0 {
            self.fade_in_step();
        }
        if self.ram.read_u8(ATTRACT_VAR17) != 255 {
            self.inc8(ATTRACT_VAR17);
        }
        if self.ram.read_u8(INTRO_TIMES_PAL_FLASH) & 4 != 0 {
            self.ram.write_u8(SOUND_EFFECT_2, 0x2b);
        }
        match self.ram.read_u8(ATTRACT_VAR5) {
            0 => self.maiden_warp_case0(),
            1 => self.maiden_warp_case1(),
            2 => self.maiden_warp_case2(),
            3 => self.maiden_warp_case3(),
            4 => self.maiden_warp_case4(),
            _ => {}
        }
        const X: [u16; 6] = [48, 192, 48, 192, 80, 160];
        const Y: [u16; 6] = [112, 112, 152, 152, 192, 192];
        const D: [u8; 6] = [0, 1, 0, 1, 3, 3];
        const F: [u8; 6] = [9, 9, 9, 9, 7, 9];
        for k in (0..6).rev() {
            self.runtime.sprite_prep_reset_properties(self.ram, k);
            self.sprite_simulate_soldier(k, X[k], Y[k], D[k], F[k], 0);
        }
        if self.ram.read_u8(ATTRACT_VAR17) >= 0xa0 {
            if self.ram.read_u8(ATTRACT_VRAM_DST) != 0x60 {
                if self.dec8(ATTRACT_VAR1) == 0 {
                    self.dec8(ATTRACT_VRAM_DST);
                    self.ram.write_u8(ATTRACT_VAR1, 8);
                }
            } else {
                self.inc8(ATTRACT_VAR11);
            }
        }
        if self.ram.read_u8(ATTRACT_VAR15) == 0 {
            const Z0: [AttractOamInfo; 4] = [
                oi(0, 0, 3, 0x3d, 2),
                oi(8, 0, 4, 0x3d, 2),
                oi(0, 0, 0, 0x3d, 2),
                oi(8, 0, 1, 0x3d, 2),
            ];
            self.ram.write_u8(ATTRACT_X_BASE, 116);
            self.ram
                .write_u8(ATTRACT_Y_BASE, self.ram.read_u8(ATTRACT_VRAM_DST));
            let off = if self.ram.read_u8(ATTRACT_VRAM_DST) == 0x70 {
                0
            } else {
                2
            };
            self.draw_sprite_set2(&Z0[off..off + 2]);
            let mut k = 7usize;
            if self.ram.read_u8(ATTRACT_VRAM_DST) < 0x68 {
                k = usize::from(self.ram.read_u8(ATTRACT_VRAM_DST).wrapping_sub(0x68) & 7);
            }
            self.ram
                .write_u8(ATTRACT_X_BASE, 0x74 + [4, 4, 3, 3, 2, 2, 1, 0][k]);
            self.ram.write_u8(ATTRACT_Y_BASE, 0x76);
            self.draw_sprite_set2(&WARP_SPARK[k * 2..k * 2 + 2]);
        }
        let k = usize::from(self.ram.read_u8(ATTRACT_VAR17) >> 5 & 7);
        self.ram.write_u8(ATTRACT_X_BASE, 112);
        self.ram.write_u8(ATTRACT_Y_BASE, 70);
        self.draw_sprite_set2(&MAIDEN[k * 6..k * 6 + 6]);
    }
    pub fn skip_to_file_select(&mut self) {
        if self.dec8(INIDISP) != 0 {
            return;
        }
        self.runtime.force_blank(self.ram);
        self.ppu.write(0x07, 0x13);
        self.ppu.write(0x08, 3);
        self.runtime.setup_conclusion_hdma(self.dma, self.ram);
        for a in [M7Y, M7X, BG1_HOFS, BG1_VOFS, BG3_VOFS2] {
            self.ram.write_u16(a, 0);
        }
        self.runtime.fade_music_and_reset_sram_mirror(self.ram);
    }
    pub fn show_timed_text_message(&mut self) {
        self.ram
            .write_u16(ATTRACT_VAR12, self.ram.read_u16(BG2_VOFS2));
        self.ram.write_u8(JOYPAD1_L_LAST, 0);
        self.ram.write_u8(FILTERED_JOYPAD_L, 0);
        self.ram.write_u8(FILTERED_JOYPAD_H, 0);
        self.runtime
            .render_text(self.ram, self.sram, self.messaging_assets);
        if self.ram.read_u16(OAM_PRIORITY) != 0 {
            self.ram
                .write_u16(OAM_PRIORITY, self.ram.read_u16(OAM_PRIORITY) - 1);
        }
    }
    pub fn control_map_zoom(&mut self) {
        let z = u32::from(self.ram.read_u8(TIMER_MODE7_ZOOM));
        for (i, &v) in MAP_MODE_ZOOMS1.iter().enumerate() {
            self.ram
                .write_u16(HDMA_TABLE_DYNAMIC + i * 2, ((u32::from(v) * z) >> 8) as u16);
        }
    }
    pub fn build_backgrounds(&mut self) {
        const T0: [u16; 16] = [
            0x1a0, 0x9a6, 0x89a5, 0x1a0, 0x9a5, 0x1a0, 0x1a0, 0x89a6, 0x49a5, 0x1a0, 0x1a0, 0x49a5,
            0x1a0, 0x89a5, 0xc9a5, 0x1a0,
        ];
        const T1: [u16; 4] = [0x9a1, 0x9a2, 0x9a3, 0x9a4];
        self.ram.write_u8(BGMODE, 9);
        self.ram.write_u8(TM, 0x17);
        self.ram.write_u8(TS, 0);
        self.ppu.write(0x07, 0x10);
        self.ppu.write(0x08, 0);
        for k in 0..128 {
            self.ram.write_u16(0x1006 + k * 2, T0[(k / 32) * 4 + k % 4]);
        }
        self.trigger_bg_dma(0x1000);
        for k in 0..128 {
            self.ram
                .write_u16(0x1006 + k * 2, T1[((k & 0x20) >> 4) + (k & 1)]);
        }
        self.trigger_bg_dma(0);
        self.ram.write_u16(ATTRACT_VRAM_DST, 0);
    }
    pub fn trigger_bg_dma(&mut self, dst: usize) {
        for block in 0..8 {
            for i in 0..128 {
                self.ppu.vram[dst + block * 128 + i] = self.ram.read_u16(0x1006 + i * 2);
            }
        }
    }
    pub fn draw_preloaded_sprite(
        &mut self,
        xp: &[u8],
        yp: &[u8],
        cp: &[u8],
        fp: &[u8],
        ep: &[u8],
        n: usize,
    ) {
        let count = n + 1;
        let mut oi = 64 + usize::from(self.ram.read_u8(ATTRACT_OAM_IDX));
        self.ram.write_u8(
            ATTRACT_OAM_IDX,
            self.ram.read_u8(ATTRACT_OAM_IDX).wrapping_add(count as u8),
        );
        for j in (0..=n).rev() {
            oam::set_plain(
                self.ram,
                oi,
                self.ram.read_u8(ATTRACT_X_BASE).wrapping_add(xp[j]),
                self.ram.read_u8(ATTRACT_Y_BASE).wrapping_add(yp[j]),
                cp[j],
                fp[j],
                ep[j],
            );
            oi += 1;
        }
    }
    pub fn draw_zelda(&mut self) {
        let i = 64 + usize::from(self.ram.read_u8(ATTRACT_OAM_IDX));
        let y = self.ram.read_u8(ATTRACT_X_BASE);
        oam::set_plain(self.ram, i, 0x60, y, 0x28, 0x29, 2);
        oam::set_plain(self.ram, i + 1, 0x60, y.wrapping_add(10), 0x2a, 0x29, 2);
        self.ram.write_u8(
            ATTRACT_OAM_IDX,
            self.ram.read_u8(ATTRACT_OAM_IDX).wrapping_add(2),
        );
    }
    pub fn sprite_simulate_soldier(
        &mut self,
        k: usize,
        x: u16,
        y: u16,
        dir: u8,
        flags: u8,
        gfx: u8,
    ) {
        self.runtime.sprite_set_x(self.ram, k, x);
        self.runtime.sprite_set_y(self.ram, k, y);
        self.ram.write_u8(SPRITE_Z + k, 0);
        self.runtime.sprite_get_16_bit_coords(self.ram, k);
        self.ram.write_u8(SPRITE_D + k, dir);
        self.ram.write_u8(SPRITE_HEAD_DIR + k, dir);
        self.ram
            .write_u8(SPRITE_GRAPHICS + k, [11, 4, 0, 7][usize::from(dir)] + gfx);
        self.ram.write_u8(SPRITE_FLAGS3 + k, 16);
        self.ram.write_u8(SPRITE_OBJ_PRIO + k, 0);
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, flags | 0x30);
        self.ram
            .write_u8(SPRITE_TYPE + k, if flags == 9 { 0x41 } else { 0x43 });
        self.ram.write_u8(SPRITE_FLAGS2 + k, 7);
        let i = k * 8;
        self.ram.write_u16(OAM_CUR_PTR, (0x800 + i * 4) as u16);
        self.ram.write_u16(OAM_EXT_CUR_PTR, (0xa20 + i) as u16);
        self.runtime.guard_handle_all_animation(self.ram, k);
    }
    pub fn build_next_image_tile_map(&mut self) {
        let legends: [&[u8]; 4] = [&LEGEND_0, &LEGEND_1, &LEGEND_2, &LEGEND_3];
        let src = legends[usize::from(self.ram.read_u8(ATTRACT_NEXT_LEGEND_GFX) >> 1)];
        self.ram
            .slice_mut(VRAM_UPLOAD_DATA, src.len())
            .copy_from_slice(src);
        self.ram.write_u8(NMI_LOAD_BG, 1);
    }
}

const WARP_OAM: [AttractOamInfo; 28] = [
    oi(0, 0, 0xce, 0x35, 0),
    oi(28, 0, 0xce, 0x35, 0),
    oi(-2, 3, 0x26, 0x75, 0),
    oi(30, 3, 0x26, 0x35, 0),
    oi(-2, 11, 0x36, 0x75, 0),
    oi(30, 11, 0x36, 0x35, 0),
    oi(0, 16, 0x26, 0x75, 0),
    oi(28, 16, 0x26, 0x35, 0),
    oi(0, 24, 0x36, 0x75, 0),
    oi(28, 24, 0x36, 0x35, 0),
    oi(2, 16, 0x20, 0x35, 2),
    oi(18, 16, 0x20, 0x75, 2),
    oi(2, 32, 0x20, 0xb5, 2),
    oi(18, 32, 0x20, 0xf5, 2),
    oi(0, 0, 0xce, 0x37, 0),
    oi(28, 0, 0xce, 0x37, 0),
    oi(-2, 3, 0x26, 0x77, 0),
    oi(30, 3, 0x26, 0x37, 0),
    oi(-2, 11, 0x36, 0x77, 0),
    oi(30, 11, 0x36, 0x37, 0),
    oi(0, 16, 0x26, 0x77, 0),
    oi(28, 16, 0x26, 0x37, 0),
    oi(0, 24, 0x36, 0x77, 0),
    oi(28, 24, 0x36, 0x37, 0),
    oi(2, 16, 0x22, 0x37, 2),
    oi(18, 16, 0x22, 0x77, 2),
    oi(2, 32, 0x22, 0xb7, 2),
    oi(18, 32, 0x22, 0xf7, 2),
];
const WARP_SPARK: [AttractOamInfo; 16] = [
    oi(0, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(2, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(2, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(4, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(4, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(6, 0, 0x6c, 0x38, 2),
    oi(0, 0, 0x6c, 0x38, 2),
    oi(8, 0, 0x6c, 0x38, 2),
];
const MAIDEN: [AttractOamInfo; 48] = [
    oi(5, 25, 0x6c, 0x38, 2),
    oi(11, 25, 0x6c, 0x38, 2),
    oi(0, 0, 0x82, 0x3b, 2),
    oi(16, 0, 0x82, 0x7b, 2),
    oi(0, 16, 0xa2, 0x3b, 2),
    oi(16, 16, 0xa2, 0x7b, 2),
    oi(5, 25, 0x6c, 0x38, 2),
    oi(11, 25, 0x6c, 0x38, 2),
    oi(0, 0, 0x80, 0x3b, 2),
    oi(16, 0, 0x82, 0x7b, 2),
    oi(0, 16, 0xa0, 0x3b, 2),
    oi(16, 16, 0xa2, 0x7b, 2),
    oi(5, 25, 0x6c, 0x38, 2),
    oi(11, 25, 0x6c, 0x38, 2),
    oi(0, 0, 0x82, 0x3b, 2),
    oi(16, 0, 0x82, 0x7b, 2),
    oi(0, 16, 0xa2, 0x3b, 2),
    oi(16, 16, 0xa2, 0x7b, 2),
    oi(5, 25, 0x6c, 0x38, 2),
    oi(11, 25, 0x6c, 0x38, 2),
    oi(0, 0, 0x82, 0x3b, 2),
    oi(16, 0, 0x80, 0x7b, 2),
    oi(0, 16, 0xa2, 0x3b, 2),
    oi(16, 16, 0xa0, 0x7b, 2),
    oi(5, 25, 0x6c, 0x38, 2),
    oi(11, 25, 0x6c, 0x38, 2),
    oi(0, 0, 0x82, 0x3b, 2),
    oi(16, 0, 0x82, 0x7b, 2),
    oi(0, 16, 0xa2, 0x3b, 2),
    oi(16, 16, 0xa2, 0x7b, 2),
    oi(5, 25, 0x6c, 0x38, 2),
    oi(11, 25, 0x6c, 0x38, 2),
    oi(0, 0, 0x80, 0x3b, 2),
    oi(16, 0, 0x82, 0x7b, 2),
    oi(0, 16, 0xa0, 0x3b, 2),
    oi(16, 16, 0xa2, 0x7b, 2),
    oi(5, 25, 0x6c, 0x38, 2),
    oi(11, 25, 0x6c, 0x38, 2),
    oi(0, 0, 0x82, 0x3b, 2),
    oi(16, 0, 0x82, 0x7b, 2),
    oi(0, 16, 0xa2, 0x3b, 2),
    oi(16, 16, 0xa2, 0x7b, 2),
    oi(5, 25, 0x6c, 0x38, 2),
    oi(11, 25, 0x6c, 0x38, 2),
    oi(0, 0, 0x80, 0x3b, 2),
    oi(16, 0, 0x80, 0x7b, 2),
    oi(0, 16, 0xa0, 0x3b, 2),
    oi(16, 16, 0xa0, 0x7b, 2),
];

const LEGEND_0: [u8; 158] = [
    0x61, 0x65, 0x40, 0x28, 0, 0x35, 0x61, 0x85, 0x40, 0x28, 0x10, 0x35, 0x61, 0xa5, 0, 0x29, 1,
    0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 3, 0x31, 3, 0x71,
    2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 0x61,
    0xc5, 0, 0x29, 0x11, 0x35, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11,
    0x35, 0x12, 0x35, 0x11, 0x35, 0x13, 0x35, 0x13, 0x75, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11,
    0x35, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11, 0x35, 0x61, 0xe5, 0,
    0x29, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21,
    0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21,
    0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x62, 5, 0x40, 0x28, 0, 0xb5,
    0xff, 0x61,
];
const LEGEND_1: [u8; 238] = [
    0x61, 0x65, 0x40, 0x28, 0, 0x35, 0x61, 0x85, 0, 0x13, 0x10, 0x35, 0x4e, 0x75, 0x6e, 0x35, 0x10,
    0x35, 0x4e, 0x35, 0x10, 0x35, 0x4c, 0x35, 0x10, 0x35, 0x4e, 0x75, 0x49, 0x35, 0x61, 0x8f, 0x40,
    8, 0x10, 0x35, 0x61, 0x94, 0, 0xb, 0x4e, 0x75, 0x6e, 0x35, 0x10, 0x35, 0x4e, 0x35, 0x10, 0x35,
    0x4c, 0x35, 0x61, 0xa5, 0, 0x29, 0x5f, 0x75, 0x5e, 0x75, 0x7e, 0x35, 0x7f, 0x35, 0x5e, 0x35,
    0x5f, 0x35, 0x4d, 0x35, 0x5f, 0x75, 0x5e, 0x75, 0x4a, 0x35, 0x4b, 0x35, 0x10, 0x35, 0x49, 0x75,
    0x10, 0x35, 0x5f, 0x75, 0x5e, 0x75, 0x7e, 0x35, 0x7f, 0x35, 0x5e, 0x35, 0x5f, 0x35, 0x4d, 0x35,
    0x61, 0xc5, 0, 0x29, 0x50, 0x35, 0x51, 0x35, 0x52, 0x35, 0x53, 0x35, 0x54, 0x35, 0x55, 0x35,
    0x56, 0x35, 0x57, 0x35, 0x58, 0x35, 0x59, 0x35, 0x5a, 0x35, 0x5b, 0x35, 0x5c, 0x35, 0x5d, 0x35,
    0x50, 0x35, 0x51, 0x35, 0x52, 0x35, 0x53, 0x35, 0x54, 0x35, 0x55, 0x35, 0x56, 0x35, 0x61, 0xe5,
    0, 0x29, 0x60, 0x35, 0x61, 0x35, 0x62, 0x35, 0x63, 0x35, 0x64, 0x35, 0x65, 0x35, 0x66, 0x35,
    0x67, 0x35, 0x68, 0x35, 0x69, 0x35, 0x6a, 0x35, 0x6b, 0x35, 0x6c, 0x35, 0x6d, 0x35, 0x60, 0x35,
    0x61, 0x35, 0x62, 0x35, 0x63, 0x35, 0x64, 0x35, 0x65, 0x35, 0x66, 0x35, 0x62, 5, 0, 0x29, 0x70,
    0x35, 0x71, 0x35, 0x72, 0x35, 0x73, 0x35, 0x74, 0x35, 0x75, 0x35, 0x76, 0x35, 0x77, 0x35, 0x78,
    0x35, 0x79, 0x35, 0x7a, 0x35, 0x7b, 0x35, 0x7c, 0x35, 0x7d, 0x35, 0x70, 0x35, 0x71, 0x35, 0x72,
    0x35, 0x73, 0x35, 0x74, 0x35, 0x75, 0x35, 0x76, 0x35, 0xff, 0x61,
];
const LEGEND_2: [u8; 200] = [
    0x61, 0x65, 0x40, 0x28, 0, 0x35, 0x61, 0x85, 0x40, 0x28, 0x10, 0x35, 0x61, 0xa5, 0, 0x1d, 0x22,
    0x35, 0x23, 0x35, 0x10, 0x35, 0x22, 0x35, 0x23, 0x35, 0x10, 0x35, 0x22, 0x35, 0x23, 0x35, 0x10,
    0x35, 0x22, 0x35, 0x23, 0x35, 0x10, 0x35, 0x10, 0x75, 0x23, 0x75, 0x22, 0x75, 0x61, 0xb4, 0x40,
    6, 0x10, 0x35, 0x61, 0xb8, 0, 3, 0x23, 0x75, 0x22, 0x75, 0x61, 0xc5, 0, 0x29, 4, 0x35, 5, 0x35,
    6, 0x35, 4, 0x35, 5, 0x35, 6, 0x35, 4, 0x35, 5, 0x35, 6, 0x35, 4, 0x35, 5, 0x35, 6, 0x35, 6,
    0x75, 5, 0x75, 4, 0x75, 0x10, 0x75, 0x23, 0x75, 0x22, 0x75, 6, 0x75, 5, 0x75, 4, 0x75, 0x61,
    0xe5, 0, 0x29, 0x14, 0x35, 0x15, 0x35, 0x16, 0x35, 0x14, 0x35, 0x15, 0x35, 0x16, 0x35, 0x14,
    0x35, 0x15, 0x35, 0x16, 0x35, 0x14, 0x35, 0x15, 0x35, 0x16, 0x35, 0x16, 0x75, 0x15, 0x75, 0x14,
    0x75, 6, 0x75, 5, 0x75, 4, 0x75, 0x16, 0x75, 0x15, 0x75, 0x14, 0x75, 0x62, 5, 0, 0x29, 0x24,
    0x35, 0x25, 0x35, 0x26, 0x35, 0x24, 0x35, 0x25, 0x35, 0x26, 0x35, 0x24, 0x35, 0x25, 0x35, 0x26,
    0x35, 0x24, 0x35, 0x25, 0x35, 0x26, 0x35, 0x26, 0x75, 0x25, 0x75, 0x24, 0x75, 0x26, 0x75, 0x25,
    0x75, 0x24, 0x75, 0x26, 0x75, 0x25, 0x75, 0x24, 0x75, 0xff, 0x61,
];
const LEGEND_3: [u8; 266] = [
    0x61, 0x65, 0, 0x29, 0, 0x35, 0, 0x35, 0x1b, 0x35, 0x30, 0x35, 0x31, 0x35, 0x32, 0x35, 0, 0x35,
    0, 0x35, 0, 0x35, 0x33, 0x35, 0x41, 0x35, 0x41, 0x75, 0x33, 0x75, 0, 0x75, 0, 0x75, 0, 0x75,
    0x32, 0x75, 0x31, 0x75, 0x30, 0x75, 0x1b, 0x75, 0, 0x75, 0x61, 0x85, 0x40, 0x1e, 0x10, 0x35,
    0x61, 0x86, 0, 9, 0x34, 0x35, 0xb, 0x35, 0x40, 0x35, 0x41, 0x35, 0x42, 0x35, 0x61, 0x95, 0, 9,
    0x42, 0x75, 0x41, 0x75, 0x40, 0x75, 0xb, 0x75, 0x34, 0x75, 0x61, 0xa5, 0, 0x29, 0x43, 0x35,
    0x44, 0x35, 7, 0x35, 8, 0x35, 9, 0x35, 0xa, 0x35, 0x10, 0x35, 0xc, 0x35, 0xd, 0x35, 0xe, 0x35,
    0xf, 0x35, 0xf, 0x75, 0xe, 0x75, 0xd, 0x75, 0xc, 0x75, 0x10, 0x75, 0xa, 0x75, 9, 0x75, 8, 0x75,
    7, 0x75, 0x44, 0x75, 0x61, 0xc5, 0, 0x29, 0x35, 0x35, 0x36, 0x35, 0x17, 0x35, 0x18, 0x35, 0x19,
    0x35, 0x1a, 0x35, 0x10, 0x35, 0x1c, 0x35, 0x1d, 0x35, 0x1e, 0x35, 0x1f, 0x35, 0x1f, 0x75, 0x1e,
    0x75, 0x1d, 0x75, 0x1c, 0x75, 0x10, 0x75, 0x1a, 0x75, 0x19, 0x75, 0x18, 0x75, 0x17, 0x75, 0x36,
    0x75, 0x61, 0xe5, 0, 0x29, 0x45, 0x35, 0x46, 0x35, 0x27, 0x35, 0x28, 0x35, 0x29, 0x35, 0x2a,
    0x35, 0x2b, 0x35, 0x2c, 0x35, 0x2d, 0x35, 0x2e, 0x35, 0x2f, 0x35, 0x2f, 0x75, 0x2e, 0x75, 0x2d,
    0x75, 0x2c, 0x75, 0x2b, 0x75, 0x2a, 0x75, 0x29, 0x75, 0x28, 0x75, 0x27, 0x75, 0x46, 0x75, 0x62,
    5, 0, 0x29, 0x47, 0x35, 0x48, 0x35, 0x37, 0x35, 0x38, 0x35, 0x39, 0x35, 0x3a, 0x35, 0x3b, 0x35,
    0x3c, 0x35, 0x3d, 0x35, 0x3e, 0x35, 0x3f, 0x35, 0x3f, 0x75, 0x3e, 0x75, 0x3d, 0x75, 0x3c, 0x75,
    0x3b, 0x75, 0x3a, 0x75, 0x39, 0x75, 0x38, 0x75, 0x37, 0x75, 0x48, 0x75, 0xff, 0,
];

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;
    struct Rt;
    impl AttractRuntime for Rt {}
    fn with_attract(f: impl FnOnce(&mut Attract<'_, '_, '_, '_, Rt>)) {
        let mut bytes = [0; WRAM_SIZE];
        let mut ppu = Ppu::new();
        let mut dma = Dma::new();
        let mut rt = Rt;
        let mut sram = [0; 0x2000];
        let messages: [&[u8]; 0] = [];
        let dictionary: [&[u8]; 0] = [];
        let messaging_assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dictionary,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let mut ram = Ram::new(&mut bytes);
        let mut a = Attract {
            ram: &mut ram,
            sram: &mut sram,
            dma: &mut dma,
            ppu: &mut ppu,
            messaging_assets: &messaging_assets,
            runtime: &mut rt,
        };
        f(&mut a);
    }
    #[test]
    fn zoom_uses_exact_fixed_point_product() {
        with_attract(|a| {
            a.ram.write_u8(TIMER_MODE7_ZOOM, 255);
            a.control_map_zoom();
            assert_eq!(a.ram.read_u16(HDMA_TABLE_DYNAMIC), 373);
            assert_eq!(a.ram.read_u16(HDMA_TABLE_DYNAMIC + 478), 256);
        });
    }
    #[test]
    fn draw_set_is_reverse_order_like_c_loop() {
        with_attract(|a| {
            a.ram.write_u8(ATTRACT_X_BASE, 10);
            a.ram.write_u8(ATTRACT_Y_BASE, 20);
            a.draw_sprite_set2(&[oi(1, 2, 3, 4, 0), oi(5, 6, 7, 8, 2)]);
            assert_eq!(a.ram.slice(0x900, 8), &[15, 26, 7, 8, 11, 22, 3, 4]);
            assert_eq!(a.ram.read_u8(ATTRACT_OAM_IDX), 2);
        });
    }
    #[test]
    fn legend_copies_exact_size_and_sets_upload_flag() {
        with_attract(|a| {
            a.ram.write_u8(ATTRACT_NEXT_LEGEND_GFX, 6);
            a.build_next_image_tile_map();
            assert_eq!(a.ram.slice(VRAM_UPLOAD_DATA, LEGEND_3.len()), &LEGEND_3);
            assert_eq!(a.ram.read_u8(NMI_LOAD_BG), 1);
        });
    }
    #[test]
    fn fade_cadence_preserves_wrapping_signed_test() {
        with_attract(|a| {
            a.ram.write_u8(LINK_SPEED_SETTING, 0);
            a.fade_in_step();
            assert_eq!(a.ram.read_u8(INIDISP), 1);
            assert_eq!(a.ram.read_u8(LINK_SPEED_SETTING), 1);
            a.ram.write_u8(INIDISP, 15);
            a.fade_in_step();
            assert_eq!(a.ram.read_u8(ATTRACT_VAR18), 1);
        });
    }

    #[test]
    fn maiden_case1_preserves_duplicated_conditional_quirk() {
        with_attract(|a| {
            a.ram.write_u8(ATTRACT_VAR20, 0);
            a.ram.write_u8(ATTRACT_VAR21, 2);
            a.maiden_warp_case1();
            assert_eq!(a.ram.read_u8(ATTRACT_VAR21), 2);
            a.ram.write_u8(ATTRACT_VAR20, 1);
            a.maiden_warp_case1();
            assert_eq!(a.ram.read_u8(ATTRACT_VAR20), 0);
            assert_eq!(a.ram.read_u8(ATTRACT_VAR21), 2);
        });
    }
}
