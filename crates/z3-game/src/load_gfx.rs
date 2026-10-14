//! Graphics decompression, tileset loading, palette filtering, and HDMA-window logic.
//!
//! This is a direct, bounds-checked Rust port of `src/load_gfx.c`. WRAM offsets and
//! VRAM word addresses use the same values as the C implementation.

use z3_core::assets::{AssetFile, packed_array_entry};

pub const GLOVES_COLOR: [u16; 2] = [0x52f6, 0x0376];

const FLAG_UPDATE_CGRAM: usize = 0x15;
const INIDISP_COPY: usize = 0x13;
const HDMAEN_COPY: usize = 0x9b;
const BGMODE_COPY: usize = 0x94;
const MOSAIC_COPY: usize = 0x95;
const SUBSUBMODULE: usize = 0xb0;
const PALETTE_FILTER_COUNTDOWN: usize = 0xc007;
const DARKENING_OR_LIGHTENING: usize = 0xc009;
const MOSAIC_TARGET_LEVEL: usize = 0xc00b;
const MOSAIC_LEVEL: usize = 0xc011;
const AUX_PALETTE: usize = 0xc300;
const MAIN_PALETTE: usize = 0xc500;
const HDMA_TABLE_DYNAMIC: usize = 0x1dba0;
const OVERWORLD_SCREEN: usize = 0x8a;
const DUNGEON_ROOM: usize = 0xa0;
const PALETTE_SWAP_FLAG: usize = 0xabd;
const FEATURES0: usize = 0x64c;
const VARIOUS_PACKS: [u8; 16] = [
    0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x5b, 0x01, 0x5a, 0x42, 0x43, 0x44, 0x45, 0x3f, 0x59, 0x0b, 0x5a,
];
const INTRO_LOAD_GFX: [usize; 10] = [0, 11, 8, 38, 42, 45, 34, 3, 33, 46];
const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];

const ASSET_PAL_DUNG_BG_MAIN: usize = 79;
const ASSET_PAL_MAIN_SPR: usize = 80;
const ASSET_PAL_ARMOR_GLOVES: usize = 81;
const ASSET_PAL_SWORD: usize = 82;
const ASSET_PAL_SHIELD: usize = 83;
const ASSET_PAL_SPR_AUX3: usize = 84;
const ASSET_PAL_MISC_SPR: usize = 85;
const ASSET_PAL_SPR_AUX1: usize = 86;
const ASSET_PAL_OW_MAIN: usize = 87;
const ASSET_PAL_OW_AUX12: usize = 88;
const ASSET_PAL_OW_AUX3: usize = 89;
const ASSET_PAL_MAP_BG: usize = 90;
const ASSET_PAL_MAP_SPR: usize = 91;
const ASSET_HUD_PAL: usize = 92;
const ASSET_OW_MAP_PAL: usize = 93;

pub const PALETTE_FILTERING_BITS: [u16; 64] = [
    0xffff, 0xffff, 0xfffe, 0xffff, 0x7fff, 0x7fff, 0x7fdf, 0xfbff, 0x7f7f, 0x7f7f, 0x7df7, 0xefbf,
    0x7bdf, 0x7bdf, 0x77bb, 0xddef, 0x7777, 0x7777, 0x6edd, 0xbb77, 0x6db7, 0x6db7, 0x5b6d, 0xb6db,
    0x5b5b, 0x5b5b, 0x56b6, 0xad6b, 0x5555, 0xad6b, 0x5555, 0xaaab, 0x5555, 0x5555, 0x2a55, 0x5555,
    0x2a55, 0x2a55, 0x294a, 0x5295, 0x2525, 0x2525, 0x2492, 0x4925, 0x1249, 0x1249, 0x1122, 0x4489,
    0x1111, 0x1111, 0x0844, 0x2211, 0x0421, 0x0421, 0x0208, 0x1041, 0x0101, 0x0101, 0x0020, 0x0401,
    0x0001, 0x0001, 0x0000, 0x0001,
];
pub const PALETTE_FILTER_AGAHNIM: [u16; 3] = [0x160, 0x180, 0x1a0];

pub const TAGALONG_WHICH: [u16; 14] = [
    0, 0x600, 0x300, 0x300, 0x300, 0, 0, 0x900, 0x600, 0x600, 0x900, 0x900, 0x600, 0x900,
];
pub const SWORD_TYPE_TO_GFX_OFFS: [u16; 5] = [0, 0, 0x120, 0x120, 0x120];
pub const SHIELD_TYPE_TO_GFX_OFFS: [u16; 4] = [0x660, 0x660, 0x6f0, 0x900];
pub const GRAPHICS_HALF_SLOT_PACKS: [u8; 20] =
    [1, 1, 8, 8, 9, 9, 2, 2, 2, 2, 3, 3, 4, 4, 5, 5, 8, 8, 8, 8];
pub const GRAPHICS_LOAD_SP6: [i8; 20] = [
    10, -1, 3, -1, 0, -1, -1, -1, 1, -1, 2, -1, 0, -1, -1, -1, -1, -1, -1, -1,
];
pub const MIRROR_WARP_NMI_LOAD: [u8; 15] = [0, 14, 15, 16, 17, 0, 0, 0, 0, 0, 0, 18, 19, 20, 0];
pub const SPOTLIGHT_DELTA_SIZE: [i8; 4] = [-7, 7, 7, 7];
pub const SPOTLIGHT_GOAL: [u8; 4] = [0, 126, 35, 126];
pub const SPOTLIGHT_CIRCLE: [u8; 129] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xfe, 0xfe,
    0xfd, 0xfd, 0xfd, 0xfd, 0xfc, 0xfc, 0xfc, 0xfb, 0xfb, 0xfb, 0xfa, 0xfa, 0xf9, 0xf9, 0xf8, 0xf8,
    0xf7, 0xf7, 0xf6, 0xf6, 0xf5, 0xf5, 0xf4, 0xf3, 0xf3, 0xf2, 0xf1, 0xf1, 0xf0, 0xef, 0xee, 0xee,
    0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe9, 0xe8, 0xe7, 0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xdf, 0xde,
    0xdd, 0xdc, 0xdb, 0xda, 0xd8, 0xd7, 0xd6, 0xd5, 0xd3, 0xd2, 0xd0, 0xcf, 0xcd, 0xcc, 0xca, 0xc9,
    0xc7, 0xc6, 0xc4, 0xc2, 0xc1, 0xbf, 0xbd, 0xbb, 0xb9, 0xb7, 0xb6, 0xb4, 0xb1, 0xaf, 0xad, 0xab,
    0xa9, 0xa7, 0xa4, 0xa2, 0x9f, 0x9d, 0x9a, 0x97, 0x95, 0x92, 0x8f, 0x8c, 0x89, 0x86, 0x82, 0x7f,
    0x7b, 0x78, 0x74, 0x70, 0x6c, 0x67, 0x63, 0x5e, 0x59, 0x53, 0x4d, 0x46, 0x3f, 0x37, 0x2d, 0x1f,
    0,
];

pub const MAIN_TILESETS: [[u8; 8]; 37] = [
    [0, 1, 16, 6, 14, 31, 24, 15],
    [0, 1, 16, 8, 14, 34, 27, 15],
    [0, 1, 16, 6, 14, 31, 24, 15],
    [0, 1, 19, 7, 14, 35, 28, 15],
    [0, 1, 16, 7, 14, 33, 24, 15],
    [0, 1, 16, 9, 14, 32, 25, 15],
    [2, 3, 18, 11, 14, 33, 26, 15],
    [0, 1, 17, 12, 14, 36, 27, 15],
    [0, 1, 17, 8, 14, 34, 27, 15],
    [0, 1, 17, 12, 14, 37, 26, 15],
    [0, 1, 17, 12, 14, 38, 27, 15],
    [0, 1, 20, 10, 14, 39, 29, 15],
    [0, 1, 17, 10, 14, 40, 30, 15],
    [2, 3, 18, 11, 14, 41, 22, 15],
    [0, 1, 21, 13, 14, 42, 24, 15],
    [0, 1, 16, 7, 14, 35, 28, 15],
    [0, 1, 19, 7, 14, 4, 5, 15],
    [0, 1, 19, 7, 14, 4, 5, 15],
    [0, 1, 16, 9, 14, 32, 27, 15],
    [0, 1, 16, 9, 14, 42, 23, 15],
    [2, 3, 18, 11, 14, 33, 28, 15],
    [0, 8, 17, 27, 34, 46, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [58, 59, 60, 61, 83, 77, 62, 91],
    [66, 67, 68, 69, 32, 43, 63, 93],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [113, 114, 113, 114, 32, 43, 93, 91],
    [58, 59, 60, 61, 83, 77, 62, 91],
    [66, 67, 68, 69, 32, 43, 63, 89],
    [0, 114, 113, 114, 32, 43, 93, 15],
    [22, 57, 29, 23, 64, 65, 57, 30],
    [0, 70, 57, 114, 64, 65, 57, 15],
];
pub const SPRITE_TILESETS: [[u8; 4]; 144] = [
    [0, 73, 0, 0],
    [70, 73, 12, 29],
    [72, 73, 19, 29],
    [70, 73, 19, 14],
    [72, 73, 12, 17],
    [72, 73, 12, 16],
    [79, 73, 74, 80],
    [14, 73, 74, 17],
    [70, 73, 18, 0],
    [0, 73, 0, 80],
    [0, 73, 0, 17],
    [72, 73, 12, 0],
    [0, 0, 55, 54],
    [72, 73, 76, 17],
    [93, 44, 12, 68],
    [0, 0, 78, 0],
    [15, 0, 18, 16],
    [0, 0, 0, 76],
    [0, 13, 23, 0],
    [22, 13, 23, 27],
    [22, 13, 23, 20],
    [21, 13, 23, 21],
    [22, 13, 24, 25],
    [22, 13, 23, 25],
    [22, 13, 0, 0],
    [22, 13, 24, 27],
    [15, 73, 74, 17],
    [75, 42, 92, 21],
    [22, 73, 23, 29],
    [0, 0, 0, 21],
    [22, 13, 23, 16],
    [22, 73, 18, 0],
    [22, 73, 12, 17],
    [0, 0, 18, 16],
    [22, 13, 0, 17],
    [22, 73, 12, 0],
    [22, 13, 76, 17],
    [14, 13, 74, 17],
    [22, 26, 23, 27],
    [79, 52, 74, 80],
    [53, 77, 101, 54],
    [74, 52, 78, 0],
    [14, 52, 74, 17],
    [81, 52, 93, 89],
    [75, 73, 76, 17],
    [45, 0, 0, 0],
    [93, 0, 18, 89],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [71, 73, 43, 45],
    [70, 73, 28, 82],
    [0, 73, 28, 82],
    [93, 73, 0, 82],
    [70, 73, 19, 82],
    [75, 77, 74, 90],
    [71, 73, 28, 82],
    [75, 77, 57, 54],
    [31, 44, 46, 82],
    [31, 44, 46, 29],
    [47, 44, 46, 82],
    [47, 44, 46, 49],
    [31, 30, 48, 82],
    [81, 73, 19, 0],
    [79, 73, 19, 80],
    [79, 77, 74, 80],
    [75, 73, 76, 43],
    [31, 32, 34, 83],
    [85, 61, 66, 67],
    [31, 30, 35, 82],
    [31, 30, 57, 58],
    [31, 30, 58, 62],
    [31, 30, 60, 61],
    [64, 30, 39, 63],
    [85, 26, 66, 67],
    [31, 30, 42, 82],
    [31, 30, 56, 82],
    [31, 32, 40, 82],
    [31, 32, 38, 82],
    [31, 44, 37, 82],
    [31, 32, 39, 82],
    [31, 30, 41, 82],
    [31, 44, 59, 82],
    [70, 73, 36, 82],
    [33, 65, 69, 51],
    [31, 44, 40, 49],
    [31, 13, 41, 82],
    [31, 30, 39, 82],
    [31, 32, 39, 83],
    [72, 73, 19, 82],
    [14, 30, 74, 80],
    [31, 32, 38, 83],
    [21, 0, 0, 0],
    [31, 0, 42, 82],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [50, 0, 0, 8],
    [93, 73, 0, 82],
    [85, 73, 66, 67],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 86, 87, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 86, 87, 80],
    [97, 86, 99, 80],
    [97, 86, 87, 80],
    [97, 86, 51, 80],
    [97, 86, 87, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
];
pub const AUX_TILESETS: [[u8; 4]; 82] = [
    [6, 0, 31, 24],
    [8, 0, 34, 27],
    [6, 0, 31, 24],
    [7, 0, 35, 28],
    [7, 0, 33, 24],
    [9, 0, 32, 25],
    [11, 0, 33, 26],
    [12, 0, 36, 25],
    [8, 0, 34, 27],
    [12, 0, 37, 27],
    [12, 0, 38, 27],
    [10, 0, 39, 29],
    [10, 0, 40, 30],
    [11, 0, 41, 22],
    [13, 0, 42, 24],
    [7, 0, 35, 28],
    [7, 0, 4, 5],
    [7, 0, 4, 5],
    [9, 0, 32, 27],
    [9, 0, 42, 23],
    [11, 0, 33, 28],
    [9, 0, 32, 25],
    [11, 0, 33, 26],
    [9, 0, 36, 27],
    [8, 0, 34, 27],
    [9, 0, 37, 27],
    [9, 0, 38, 27],
    [10, 0, 39, 29],
    [9, 0, 40, 30],
    [12, 0, 41, 22],
    [13, 0, 42, 23],
    [114, 0, 43, 93],
    [0, 0, 0, 0],
    [0, 87, 76, 0],
    [0, 86, 79, 0],
    [0, 83, 77, 0],
    [0, 82, 73, 0],
    [0, 85, 74, 0],
    [0, 83, 84, 0],
    [0, 81, 78, 0],
    [0, 0, 0, 0],
    [0, 80, 75, 0],
    [0, 83, 77, 0],
    [0, 85, 84, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 71, 72, 0],
    [0, 0, 0, 0],
    [0, 87, 76, 0],
    [0, 86, 79, 0],
    [0, 83, 77, 0],
    [0, 82, 73, 0],
    [0, 85, 74, 0],
    [0, 83, 84, 0],
    [0, 81, 78, 0],
    [0, 0, 0, 0],
    [0, 80, 75, 0],
    [0, 83, 0, 0],
    [0, 53, 54, 0],
    [0, 96, 52, 0],
    [0, 43, 44, 0],
    [0, 45, 46, 0],
    [0, 47, 48, 0],
    [0, 55, 56, 0],
    [0, 51, 52, 0],
    [0, 49, 50, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [114, 113, 114, 113],
    [23, 64, 65, 57],
];
pub const DECODE_ANIMATED_SPRITE_TILE: [u16; 57] = [
    0x9c0, 0x30, 0x60, 0x90, 0xc0, 0x300, 0x318, 0x330, 0x348, 0x360, 0x378, 0x390, 0x930, 0x3f0,
    0x420, 0x450, 0x468, 0x600, 0x630, 0x660, 0x690, 0x6c0, 0x6f0, 0x720, 0x750, 0x768, 0x900,
    0x930, 0x960, 0x990, 0x9f0, 0, 0xf0, 0xa20, 0xa50, 0x660, 0x600, 0x618, 0x630, 0x648, 0x678,
    0x6d8, 0x6a8, 0x708, 0x738, 0x768, 0x960, 0x900, 0x3c0, 0x990, 0x9a8, 0x9c0, 0x9d8, 0xa08,
    0xa38, 0x600, 0x630,
];
pub const OW_BG_PAL_INFO: [i8; 93] = [
    0, -1, 7, 0, 1, 7, 0, 2, 7, 0, 3, 7, 0, 4, 7, 0, 5, 7, 0, 6, 7, 7, 6, 5, 0, 8, 7, 0, 9, 7, 0,
    10, 7, 0, 11, 7, 0, -1, 7, 0, -1, 7, 3, 4, 7, 4, 4, 3, 16, -1, 6, 16, 1, 6, 16, 17, 6, 16, 3,
    6, 16, 4, 6, 16, 5, 6, 16, 6, 6, 18, 19, 4, 18, 5, 4, 16, 9, 6, 16, 11, 6, 16, 12, 6, 16, 13,
    6, 16, 14, 6, 16, 15, 6,
];
pub const OW_SPR_PAL_INFO: [i8; 40] = [
    -1, -1, 3, 10, 3, 6, 3, 1, 0, 2, 3, 14, 3, 2, 19, 1, 11, 12, 17, 1, 7, 5, 17, 0, 9, 11, 15, 5,
    3, 5, 3, 7, 15, 2, 10, 2, 5, 1, 12, 14,
];

/// Asset access used by the graphics loader. Sprite and background sheets are
/// packed-array entries 64 and 65 in `zelda3_assets.dat`.
pub trait GfxAssets {
    fn sprite(&self, index: usize) -> &[u8];
    fn background(&self, index: usize) -> &[u8];
    fn raw(&self, index: usize) -> &[u8];
    fn dialogue_font(&self) -> &[u8];
    fn gloves_color(&self, index: usize) -> u16 {
        GLOVES_COLOR[index]
    }
}

impl GfxAssets for AssetFile<'_> {
    fn sprite(&self, index: usize) -> &[u8] {
        packed_array_entry(self.get(64).expect("sprite graphics asset"), index)
            .expect("sprite graphics pack")
    }

    fn background(&self, index: usize) -> &[u8] {
        packed_array_entry(self.get(65).expect("background graphics asset"), index)
            .expect("background graphics pack")
    }

    fn raw(&self, index: usize) -> &[u8] {
        self.get(index).expect("graphics asset")
    }

    fn dialogue_font(&self) -> &[u8] {
        let language = packed_array_entry(self.get(95).expect("dialogue font asset"), 0)
            .expect("dialogue font language");
        packed_array_entry(language, 0).expect("dialogue font graphics")
    }
}

/// Calls that belong to other source modules or to the emulated PPU.
pub trait LoadGfxHooks {
    fn ppu_write(&mut self, _register: u8, _value: u8) {}
    fn setup_hdma(&mut self, _table: u16, _table_bank: u8, _bbad: u8) {}
    fn set_target_overworld_warp_to_pyramid(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {}
    fn pre_overworld_load_overlays(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {}
    fn overworld_draw_mirror_position(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {}
    fn mirror_warp_load_sprites_and_colors(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {}
    fn handle_followers_after_mirroring(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {}
    fn sprite_reset_all(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {}
    fn file_select_gear(&mut self, _slot: usize) -> (u8, u8, u8, u8) {
        (0, 0, 0, 0)
    }
}

pub struct LoadGfx<'a, A: GfxAssets, H: LoadGfxHooks> {
    pub wram: &'a mut [u8; z3_core::WRAM_SIZE],
    pub vram: &'a mut [u16],
    pub assets: &'a A,
    pub hooks: &'a mut H,
}

impl<A: GfxAssets, H: LoadGfxHooks> LoadGfx<'_, A, H> {
    fn read8(&self, address: usize) -> u8 {
        self.wram[address]
    }
    fn write8(&mut self, address: usize, value: u8) {
        self.wram[address] = value;
    }
    fn read16(&self, address: usize) -> u16 {
        u16::from_le_bytes([self.wram[address], self.wram[address + 1]])
    }
    fn write16(&mut self, address: usize, value: u16) {
        self.wram[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn increment8(&mut self, address: usize) {
        self.write8(address, self.read8(address).wrapping_add(1));
    }
    fn palette(&self, base: usize, index: usize) -> u16 {
        self.read16(base + index * 2)
    }
    fn set_palette(&mut self, base: usize, index: usize, value: u16) {
        self.write16(base + index * 2, value);
    }
    fn asset_words(&self, asset: usize, offset: usize, count: usize) -> Vec<u16> {
        self.assets.raw(asset)[offset * 2..(offset + count) * 2]
            .chunks_exact(2)
            .map(|x| u16::from_le_bytes([x[0], x[1]]))
            .collect()
    }

    fn decomp_and_upload_2bpp(&mut self, vram_offset: usize, pack: usize) {
        self.decomp_spr_to_wram(0x14000, pack);
        for i in 0..1024 {
            self.vram[vram_offset + i] = self.read16(0x14000 + i * 2);
        }
    }

    pub fn snes_divide(dividend: u16, divisor: u8) -> u16 {
        if divisor == 0 {
            0xffff
        } else {
            dividend / u16::from(divisor)
        }
    }

    pub fn erase_tile_maps(&mut self, high: u16, low: u16) {
        self.vram[..0x2000].fill(low);
        self.vram[0x6000..0x6800].fill(high);
    }

    pub fn erase_tile_maps_normal(&mut self) {
        self.erase_tile_maps(0x7f, 0x1ec);
    }
    pub fn erase_tile_maps_triforce(&mut self) {
        self.erase_tile_maps(0xa9, 0x7f);
    }
    pub fn erase_tile_maps_dungeonmap(&mut self) {
        self.erase_tile_maps(0x7f, 0x300);
    }

    pub fn enable_force_blank(&mut self) {
        self.write8(INIDISP_COPY, 0x80);
        self.write8(HDMAEN_COPY, 0);
    }

    pub fn load_overworld_map_palette(&mut self) {
        let offset = if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            0x80
        } else {
            0
        };
        let colors = self.asset_words(ASSET_OW_MAP_PAL, offset, 128);
        for (i, color) in colors.into_iter().enumerate() {
            self.set_palette(MAIN_PALETTE, i, color);
        }
    }

    pub fn load_item_animation_gfx_one(
        &mut self,
        destination: usize,
        count: usize,
        table_index: usize,
        from_temp: bool,
    ) -> usize {
        let base = if from_temp { 0x14000 } else { 0 };
        let source = if from_temp {
            self.wram[0x14000..0x14600].to_vec()
        } else {
            self.assets.sprite(0).to_vec()
        };
        let offset = INTRO_LOAD_GFX[table_index] * 24;
        expand_3_to_4_high(
            &mut self.wram[destination..],
            &source[offset..],
            base + offset,
            count,
        );
        expand_3_to_4_high(
            &mut self.wram[destination + count * 0x20..],
            &source[offset + 0x180..],
            base + offset + 0x180,
            count,
        );
        destination + count * 0x40
    }

    pub fn load_item_gfx_into_wram_4bpp_buffer(&mut self) {
        let mut dst = 0x9480;
        for (count, index) in [(7, 0), (7, 1), (3, 2)] {
            dst = self.load_item_animation_gfx_one(dst, count, index, false);
        }
        self.decomp_spr_to_wram(0x14000, 95);
        for (count, index, temp) in [(4, 3, true), (3, 4, true), (1, 5, true), (4, 6, false)] {
            dst = self.load_item_animation_gfx_one(dst, count, index, temp);
        }
        self.decomp_spr_to_wram(0x14000, 96);
        for (count, index) in [(14, 7), (7, 8)] {
            dst = self.load_item_animation_gfx_one(dst, count, index, true);
        }
        self.decomp_spr_to_wram(0x14000, 95);
        self.load_item_animation_gfx_one(dst, 2, 9, true);
        self.decomp_spr_to_wram(0x14000, 84);
        let source = self.wram[0x14000..0x14600].to_vec();
        expand_3_to_4_high(&mut self.wram[0xa480..], &source, 0x14000, 8);
        expand_3_to_4_high(&mut self.wram[0xa580..], &source[0x180..], 0x14180, 8);
        self.decomp_spr_to_wram(0x14000, 96);
        let source = self.wram[0x14000..0x14600].to_vec();
        expand_3_to_4_high(&mut self.wram[0xb280..], &source, 0x14000, 3);
        expand_3_to_4_high(&mut self.wram[0xb2e0..], &source[0x180..], 0x14180, 3);
        self.load_item_gfx_auxiliary();
    }

    pub fn decompress_sword_graphics(&mut self) {
        self.decomp_spr_to_wram(0x14600, 0x5f);
        self.decomp_spr_to_wram(0x14000, 0x5e);
        let offset = usize::from(SWORD_TYPE_TO_GFX_OFFS[usize::from(self.read8(0xf359))]);
        let source = self.wram[0x14000 + offset..0x15200].to_vec();
        expand_3_to_4_high(&mut self.wram[0x9000..], &source, 0x14000 + offset, 12);
        expand_3_to_4_high(
            &mut self.wram[0x9180..],
            &source[0x180..],
            0x14180 + offset,
            12,
        );
    }

    pub fn decompress_shield_graphics(&mut self) {
        self.decomp_spr_to_wram(0x14600, 0x5f);
        self.decomp_spr_to_wram(0x14000, 0x5e);
        let offset = usize::from(SHIELD_TYPE_TO_GFX_OFFS[usize::from(self.read8(0xf35a))]);
        let source = self.wram[0x14000 + offset..0x15200].to_vec();
        expand_3_to_4_high(&mut self.wram[0x9300..], &source, 0x14000 + offset, 6);
        expand_3_to_4_high(
            &mut self.wram[0x93c0..],
            &source[0x180..],
            0x14180 + offset,
            6,
        );
    }

    pub fn decompress_animated_dungeon_tiles(&mut self, pack: u8) {
        self.decomp_bg_to_wram(0x14000, usize::from(pack));
        self.convert_wram(0xa680, 0x14000, 48, false);
        self.decomp_bg_to_wram(0x14000, 0x5c);
        self.convert_wram(0xac80, 0x14000, 48, false);
        for i in (0..512).step_by(2) {
            let a = self.read16(0xa880 + i);
            let b = self.read16(0xac80 + i);
            let c = self.read16(0xae80 + i);
            let d = self.read16(0xaa80 + i);
            self.write16(0xa880 + i, b);
            self.write16(0xac80 + i, c);
            self.write16(0xae80 + i, d);
            self.write16(0xaa80 + i, a);
        }
        self.write16(0x134, 0x3b00);
    }

    pub fn decompress_animated_overworld_tiles(&mut self, pack: u8) {
        self.decomp_bg_to_wram(0x14000, usize::from(pack));
        self.convert_wram(0xa680, 0x14000, 64, false);
        self.decomp_bg_to_wram(0x14000, usize::from(pack.wrapping_add(1)));
        self.convert_wram(0xae80, 0x14000, 32, false);
        self.write16(0x134, 0x3c00);
    }

    pub fn load_item_gfx_auxiliary(&mut self) {
        self.decomp_bg_to_wram(0x14000, 0xf);
        self.convert_wram(0xb340, 0x14000, 16, false);
        self.decomp_spr_to_wram(0x14000, 0x58);
        self.convert_wram(0xb540, 0x14000, 32, false);
        self.decomp_bg_to_wram(0x14000, 5);
        self.convert_wram(0xbdc0, 0x14480, 2, false);
    }

    pub fn load_follower_graphics(&mut self) {
        let follower = usize::from(self.read8(0xf3cc));
        let pack = if follower == 1 {
            0x64
        } else if follower >= 12 {
            0x58
        } else if follower >= 9 {
            0x59
        } else {
            0x66
        };
        self.decomp_spr_to_wram(0x14600, pack);
        self.decomp_spr_to_wram(0x14000, 0x65);
        let source = 0x14000 + usize::from(TAGALONG_WHICH[follower]);
        self.convert_wram(0xb940, source, 0x20, false);
    }

    pub fn write_to_4bpp_buffer_at_7f4000(&mut self, index: u8) {
        let offset = usize::from(DECODE_ANIMATED_SPRITE_TILE[usize::from(index)]);
        let source = self.wram[0x14000 + offset..0x15200].to_vec();
        expand_3_to_4_high(&mut self.wram[0xbd40..], &source, 0x14000 + offset, 2);
        expand_3_to_4_high(
            &mut self.wram[0xbd80..],
            &source[0x180..],
            0x14180 + offset,
            2,
        );
    }

    pub fn decode_animated_sprite_tile_variable(&mut self, index: u8) {
        let pack = if index == 0x23 || index >= 0x37 {
            0x5d
        } else if index == 0xc || index >= 0x24 {
            0x5c
        } else {
            0x5b
        };
        self.decomp_spr_to_wram(0x14600, pack);
        self.decomp_spr_to_wram(0x14000, 0x5a);
        self.write_to_4bpp_buffer_at_7f4000(index);
    }

    pub fn decomp_spr(&self, dst: &mut [u8], mut gfx: usize) -> usize {
        if gfx < 12 {
            gfx = 12;
        }
        let src = self.assets.sprite(gfx);
        if gfx >= 103 || src.len() != 0x600 {
            decompress(dst, src)
        } else {
            dst[..0x600].copy_from_slice(src);
            0x600
        }
    }

    pub fn decomp_bg(&self, dst: &mut [u8], gfx: usize) -> usize {
        decompress(dst, self.assets.background(gfx))
    }

    pub fn decomp_spr_to_wram(&mut self, offset: usize, mut gfx: usize) -> usize {
        if gfx < 12 {
            gfx = 12;
        }
        let source = self.assets.sprite(gfx);
        let destination = &mut self.wram[offset..];
        if gfx >= 103 || source.len() != 0x600 {
            decompress(destination, source)
        } else {
            destination[..0x600].copy_from_slice(source);
            0x600
        }
    }

    pub fn decomp_bg_to_wram(&mut self, offset: usize, gfx: usize) -> usize {
        decompress(&mut self.wram[offset..], self.assets.background(gfx))
    }

    pub fn animate_mirror_warp_decompress_new_tile_sets(&mut self) {
        let main = MAIN_TILESETS[usize::from(self.read8(0xaa1))];
        let aux = AUX_TILESETS[usize::from(self.read8(0xaa2))];
        for i in 0..4 {
            self.write8(0xc2f8 + i, if aux[i] != 0 { aux[i] } else { main[i + 3] });
        }
        let sprites = SPRITE_TILESETS[usize::from(self.read8(0xaa3))];
        for (i, value) in sprites.into_iter().enumerate() {
            if value != 0 {
                self.write8(0xc2fc + i, value);
            }
        }
    }

    pub fn gfx_load_sprites_inner(&mut self, destination: usize) {
        let packs = SPRITE_TILESETS[usize::from(self.read8(0xaa3))];
        for (i, selected) in packs.into_iter().enumerate() {
            if selected != 0 {
                self.write8(0xc2fc + i, selected);
            }
            let pack = usize::from(self.read8(0xc2fc + i));
            let length = self.decomp_spr_to_wram(destination + i * 0x600, pack);
            debug_assert_eq!(length, 0x600);
        }
        self.write8(0x412, 0);
    }

    pub fn load_trans_aux_gfx(&mut self) {
        let packs = AUX_TILESETS[usize::from(self.read8(0xaa2))];
        for (i, pack) in packs.into_iter().enumerate() {
            if pack != 0 {
                self.write8(0xc2f8 + i, pack);
                let length = self.decomp_bg_to_wram(0x6000 + i * 0x600, usize::from(pack));
                debug_assert_eq!(length, 0x600);
            }
        }
        self.gfx_load_sprites_inner(0x7800);
    }

    pub fn load_trans_aux_gfx_sprite(&mut self) {
        self.gfx_load_sprites_inner(0x7800);
    }

    pub fn reload_previously_loaded_sheets(&mut self) {
        for i in 0..4 {
            let pack = usize::from(self.read8(0xc2f8 + i));
            self.decomp_bg_to_wram(0x6000 + i * 0x600, pack);
        }
        for i in 0..4 {
            let pack = usize::from(self.read8(0xc2fc + i));
            self.decomp_spr_to_wram(0x7800 + i * 0x600, pack);
        }
        self.write8(0x412, 0);
    }

    pub fn attract_decompress_story_gfx(&mut self) {
        self.decomp_spr_to_wram(0x14000, 0x67);
        self.decomp_spr_to_wram(0x14800, 0x68);
    }

    pub fn animate_mirror_warp(&mut self) {
        let state = self.read8(0x200);
        self.write8(0x200, state.wrapping_add(1));
        let nmi = MIRROR_WARP_NMI_LOAD[usize::from(state)];
        self.write8(0x17, nmi);
        self.write8(0x710, nmi);
        let xt = if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            8
        } else {
            0
        };
        match state {
            0 => {
                let ctr = self.read8(0x6ba).wrapping_add(1);
                self.write8(0x6ba, ctr);
                if ctr != 32 {
                    self.write8(0x200, 0);
                } else {
                    self.hooks.set_target_overworld_warp_to_pyramid(self.wram);
                }
            }
            1 => {
                self.animate_mirror_warp_decompress_new_tile_sets();
                self.decomp_bg_to_wram(0x14000, usize::from(VARIOUS_PACKS[xt]));
                self.decomp_bg_to_wram(0x14600, usize::from(VARIOUS_PACKS[xt + 1]));
                self.convert_wram(0x10000, 0x14000, 64, true);
                self.convert_wram(0x10800, 0x14600, 64, false);
            }
            2 => {
                self.decomp_bg_to_wram(0x14000, usize::from(VARIOUS_PACKS[xt + 2]));
                self.decomp_bg_to_wram(0x14600, usize::from(VARIOUS_PACKS[xt + 3]));
                self.convert_wram(0x10000, 0x14000, 64, false);
                self.convert_wram(0x10800, 0x14600, 64, true);
            }
            3 => {
                self.decomp_bg_to_wram(0x14000, usize::from(self.read8(0xc2f9)));
                self.decomp_bg_to_wram(0x14600, usize::from(self.read8(0xc2fa)));
                self.convert_wram(0x10000, 0x14000, 128, true);
            }
            4 => {
                self.decomp_bg_to_wram(0x14000, usize::from(VARIOUS_PACKS[xt + 4]));
                self.decomp_bg_to_wram(0x14600, usize::from(VARIOUS_PACKS[xt + 5]));
                self.convert_wram(0x10000, 0x14000, 128, false);
            }
            5 => {
                self.hooks.pre_overworld_load_overlays(self.wram);
                if matches!(self.read8(OVERWORLD_SCREEN), 27 | 91) {
                    self.write8(0x1d, 1);
                }
                self.write8(0x11, self.read8(0x11).wrapping_sub(1));
                self.write8(0x17, 12);
                self.write8(0x710, 12);
            }
            6 | 9 => {
                self.write8(0x17, 13);
                self.write8(0x710, 13);
            }
            7 => {
                self.hooks.overworld_draw_mirror_position(self.wram);
                self.increment8(0x710);
            }
            8 => {
                self.hooks.mirror_warp_load_sprites_and_colors(self.wram);
                self.write8(0x17, 12);
                self.write8(0x710, 12);
            }
            10 => {
                let screen = self.read8(OVERWORLD_SCREEN) & 0xbf;
                self.decompress_animated_overworld_tiles(if matches!(screen, 3 | 5 | 7) {
                    0x58
                } else {
                    0x5a
                });
            }
            11 => {
                let screen = self.read8(OVERWORLD_SCREEN);
                self.write8(
                    0x1d,
                    u8::from(matches!(
                        screen,
                        0 | 0x70 | 0x40 | 0x5b | 3 | 5 | 7 | 0x43 | 0x45 | 0x47
                    )),
                );
                let source = self
                    .assets
                    .sprite(usize::from(VARIOUS_PACKS[xt + 6]))
                    .to_vec();
                do_3_to_4_high_16(&mut self.wram[0x10000..0x10800], &source, 64);
            }
            12 => {
                self.decomp_spr_to_wram(0x14000, usize::from(self.read8(0xc2fc)));
                self.decomp_spr_to_wram(0x14600, usize::from(self.read8(0xc2fd)));
                let high = matches!(self.read16(0xc2fc), 0x52 | 0x53 | 0x5a | 0x5b);
                self.convert_wram(0x10000, 0x14000, 64, high);
                self.convert_wram(0x10800, 0x14600, 64, false);
            }
            13 => {
                self.decomp_spr_to_wram(0x14000, usize::from(self.read8(0xc2fe)));
                self.decomp_spr_to_wram(0x14600, usize::from(self.read8(0xc2ff)));
                self.convert_wram(0x10000, 0x14000, 128, false);
                self.hooks.handle_followers_after_mirroring(self.wram);
            }
            14 => self.write8(0x200, 14),
            _ => {}
        }
    }

    fn convert_to_vram(&mut self, vram_offset: usize, source: &[u8], high: bool) {
        let mut converted = vec![0u8; 64 * 32];
        convert_3_to_4(&mut converted, source, 64, high, true);
        for (i, pair) in converted.chunks_exact(2).enumerate() {
            self.vram[vram_offset + i] = u16::from_le_bytes([pair[0], pair[1]]);
        }
        if high {
            // C's `Do3To4High` uses this WRAM range as an eight-word
            // temporary, so the final converted tile remains observable.
            leave_high_conversion_scratch(self.wram, source);
        }
    }

    pub fn load_sprite_graphics(&mut self, vram_offset: usize, pack: usize, temp: usize) {
        self.decomp_spr_to_wram(temp, pack);
        let source = self.wram[temp..temp + 0x600].to_vec();
        self.convert_to_vram(
            vram_offset,
            &source,
            matches!(pack, 0x52 | 0x53 | 0x5a | 0x5b | 0x5c | 0x5e | 0x5f),
        );
    }

    pub fn load_background_graphics(
        &mut self,
        vram_offset: usize,
        pack: usize,
        slot: usize,
        temp: usize,
    ) {
        self.decomp_bg_to_wram(temp, pack);
        let source = self.wram[temp..temp + 0x600].to_vec();
        let high = if self.read8(0xaa1) >= 0x20 {
            matches!(slot, 7 | 2 | 3 | 4)
        } else {
            slot >= 4
        };
        self.convert_to_vram(vram_offset, &source, high);
    }

    pub fn load_common_sprites(&mut self) {
        let misc = usize::from(self.read8(0xaa4));
        let source = self.assets.sprite(misc).to_vec();
        self.convert_to_vram(0x4400, &source, true);
        if self.read8(0x10) != 1 {
            for (vram, pack) in [(0x4800, 6), (0x4c00, 7)] {
                let source = self.assets.sprite(pack).to_vec();
                self.convert_to_vram(vram, &source, false);
            }
        } else {
            self.load_sprite_graphics(0x4800, 94, 0x14000);
            self.load_sprite_graphics(0x4c00, 95, 0x14000);
        }
    }

    pub fn initialize_tilesets(&mut self) {
        self.load_common_sprites();
        let sprites = SPRITE_TILESETS[usize::from(self.read8(0xaa3))];
        for (i, selected) in sprites.into_iter().enumerate() {
            if selected != 0 {
                self.write8(0xc2fc + i, selected);
            }
            let pack = usize::from(self.read8(0xc2fc + i));
            self.load_sprite_graphics(0x5000 + i * 0x400, pack, 0x7800 + i * 0x600);
        }
        let main = MAIN_TILESETS[usize::from(self.read8(0xaa1))];
        let aux = AUX_TILESETS[usize::from(self.read8(0xaa2))];
        for i in 0..4 {
            self.write8(0xc2f8 + i, if aux[i] != 0 { aux[i] } else { main[i + 3] });
        }
        for (vram, pack, slot, temp) in [
            (0x2000, main[0], 7, 0x14000),
            (0x2400, main[1], 6, 0x14000),
            (0x2800, main[2], 5, 0x14000),
            (0x2c00, self.read8(0xc2f8), 4, 0x6000),
            (0x3000, self.read8(0xc2f9), 3, 0x6600),
            (0x3400, self.read8(0xc2fa), 2, 0x6c00),
            (0x3800, self.read8(0xc2fb), 1, 0x7200),
            (0x3c00, main[7], 0, 0x14000),
        ] {
            self.load_background_graphics(vram, usize::from(pack), slot, temp);
        }
    }

    pub fn load_default_graphics(&mut self) {
        let source = self.assets.sprite(0).to_vec();
        self.convert_to_vram(0x4000, &source, true);
        // The C converter uses g_ram[0xbf..0xcf] as eight 16-bit scratch words
        // array. It processes each tile's first 16 bytes in reverse-index
        // order and leaves the final tile's plane OR masks observable in
        // WRAM, so canonical state parity must preserve that residue.
        leave_default_graphics_scratch(self.wram, &source);
        self.decomp_and_upload_2bpp(0x7000, 0x6a);
        self.decomp_and_upload_2bpp(0x7400, 0x6b);
        self.decomp_and_upload_2bpp(0x7800, 0x69);
    }

    pub fn attract_load_bg3_gfx(&mut self) {
        self.decomp_and_upload_2bpp(0x7800, 0x67);
    }

    pub fn transfer_font_to_vram(&mut self) {
        let font = self.assets.dialogue_font();
        for (i, pair) in font[..0x1000].chunks_exact(2).enumerate() {
            self.vram[0x7000 + i] = u16::from_le_bytes([pair[0], pair[1]]);
        }
    }

    /// Exact graphics upload performed by C `LoadFileSelectGraphics`.
    pub fn load_file_select_graphics(&mut self) {
        self.decomp_spr_to_wram(0x14000, 0x5e);
        let source = self.wram[0x14000..0x14600].to_vec();
        do_3_to_4_high(&mut self.vram[0x5000..0x5400], &source);
        leave_high_conversion_scratch(self.wram, &source);

        self.decomp_spr_to_wram(0x14000, 0x5f);
        let source = self.wram[0x14000..0x14600].to_vec();
        do_3_to_4_high(&mut self.vram[0x5400..0x5800], &source);
        leave_high_conversion_scratch(self.wram, &source);

        self.transfer_font_to_vram();

        self.decomp_spr_to_wram(0x14000, 0x6b);
        for index in 0..0x300 {
            self.vram[0x7800 + index] = self.read16(0x14000 + index * 2);
        }
    }

    pub fn graphics_load_chr_half_slot(&mut self) {
        let mut k = usize::from(self.read8(0xaaa));
        if k == 0 {
            return;
        }
        let sp6 = GRAPHICS_LOAD_SP6[k - 1];
        if sp6 >= 0 {
            self.write8(0xab1, sp6 as u8);
            self.write16(0xaa8, 0x200);
            if k == 1 {
                self.write8(0xab1, 10);
                self.palette_load_sprite_environment();
            } else {
                self.palette_load_sprite_environment_dungeon();
            }
            self.increment8(FLAG_UPDATE_CGRAM);
        }
        let mut tilebytes = 0x44;
        let mut bank_offset = 0;
        self.write8(0xaaa, self.read8(0xaaa).wrapping_add(1));
        if self.read8(0xaaa) & 1 != 0 {
            self.write8(0xaaa, 0);
            if k != 18 {
                bank_offset = 0x300;
                tilebytes = 0x46;
                if k == 2 {
                    self.write8(0x112, 0);
                }
            }
        }
        self.write8(0x116, tilebytes);
        self.write8(0x17, 11);
        k = usize::from(GRAPHICS_HALF_SLOT_PACKS[k - 1]);
        if k == 1 {
            k = usize::from(self.read8(0xaa4));
        }
        let source = &self.assets.sprite(k)[bank_offset..bank_offset + 0x300];
        do_3_to_4_high_16(&mut self.wram[0x11000..0x11400], source, 32);
    }

    pub fn prep_trans_aux_gfx(&mut self) {
        self.convert_wram(0x10000, 0x6000, 0x40, true);
        if self.read8(0xaa2) >= 32 {
            self.convert_wram(0x10800, 0x6600, 0x80, true);
            self.convert_wram(0x11800, 0x7200, 0x40, false);
        } else {
            self.convert_wram(0x10800, 0x6600, 0xc0, false);
        }
    }

    pub fn load_new_sprite_gfx_set(&mut self) {
        self.convert_wram(0x10000, 0x7800, 0xc0, false);
        let pack = self.read8(0xc2ff);
        self.convert_wram(
            0x11800,
            0x8a00,
            0x40,
            matches!(pack, 0x52 | 0x53 | 0x5a | 0x5b),
        );
    }

    fn convert_wram(&mut self, destination: usize, source: usize, count: usize, high: bool) {
        let input = self.wram[source..source + count * 24].to_vec();
        let output = &mut self.wram[destination..destination + count * 32];
        if high {
            do_3_to_4_high_16(output, &input, count);
        } else {
            do_3_to_4_low_16(output, &input, count);
        }
    }

    pub fn apply_palette_filter_bounce(&mut self) {
        let countdown = self.read16(PALETTE_FILTER_COUNTDOWN);
        let table_offset = usize::from(countdown >= 0x10);
        let mask = 0x8000u16 >> (countdown & 15);
        let lighten = self.read16(DARKENING_OR_LIGHTENING) != 0;
        let delta = if lighten { 1u16 } else { u16::MAX };
        let mut j = 0usize;
        loop {
            let a = self.palette(AUX_PALETTE, j);
            let mut c = self.palette(MAIN_PALETTE, j);
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x1f) * 2)] & mask == 0 {
                c = c.wrapping_add(delta);
            }
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x3e0) >> 4)] & mask == 0 {
                c = c.wrapping_add(delta << 5);
            }
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x7c00) >> 9)] & mask == 0 {
                c = c.wrapping_add(delta << 10);
            }
            self.set_palette(MAIN_PALETTE, j, c);
            j += 1;
            j = match j {
                1 => 0x20,
                0xd8 => 0xe0,
                _ => j,
            };
            if j == 0xf0 {
                break;
            }
        }
        self.increment8(FLAG_UPDATE_CGRAM);
        let target = u16::from(self.read8(MOSAIC_TARGET_LEVEL));
        let reached = if !lighten {
            let next = countdown.wrapping_add(1);
            self.write16(PALETTE_FILTER_COUNTDOWN, next);
            next == target
        } else {
            self.write16(PALETTE_FILTER_COUNTDOWN, countdown.wrapping_sub(1));
            countdown == target
        };
        if !reached {
            return;
        }
        self.write16(
            DARKENING_OR_LIGHTENING,
            self.read16(DARKENING_OR_LIGHTENING) ^ 2,
        );
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.increment8(SUBSUBMODULE);
    }

    pub fn palette_filter_range(&mut self, from: usize, to: usize) {
        let countdown = self.read16(PALETTE_FILTER_COUNTDOWN);
        let table_offset = usize::from(countdown >= 0x10);
        let mask = 0x8000u16 >> (countdown & 15);
        let delta = if self.read16(DARKENING_OR_LIGHTENING) != 0 {
            1u16
        } else {
            u16::MAX
        };
        for j in from..to {
            let a = self.palette(AUX_PALETTE, j);
            let mut c = self.palette(MAIN_PALETTE, j);
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x1f) * 2)] & mask == 0 {
                c = c.wrapping_add(delta);
            }
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x3e0) >> 4)] & mask == 0 {
                c = c.wrapping_add(delta << 5);
            }
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x7c00) >> 9)] & mask == 0 {
                c = c.wrapping_add(delta << 10);
            }
            self.set_palette(MAIN_PALETTE, j, c);
        }
    }

    pub fn palette_filter_incr_countdown(&mut self) {
        let next = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, next);
        if next == 0x1f {
            self.write16(PALETTE_FILTER_COUNTDOWN, 0);
            let mode = self.read16(DARKENING_OR_LIGHTENING) ^ 2;
            self.write16(DARKENING_OR_LIGHTENING, mode);
            // The original increments WORD(link_actual_vel_y) here when mode is nonzero.
            if mode != 0 {
                let v = self.read16(0x27);
                self.write16(0x27, v.wrapping_add(1));
            }
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_restore_additive(&mut self, from: usize, to: usize) {
        for i in from / 2..to / 2 {
            let c = self.palette(MAIN_PALETTE, i);
            let d = self.palette(AUX_PALETTE, i);
            let mut out = c;
            if c & 0x1f != d & 0x1f {
                out = out.wrapping_add(1);
            }
            if c & 0x3e0 != d & 0x3e0 {
                out = out.wrapping_add(0x20);
            }
            if c & 0x7c00 != d & 0x7c00 {
                out = out.wrapping_add(0x400);
            }
            self.set_palette(MAIN_PALETTE, i, out);
        }
    }

    pub fn palette_filter_restore_subtractive(&mut self, from: usize, to: usize) {
        for i in from / 2..to / 2 {
            let c = self.palette(MAIN_PALETTE, i);
            let d = self.palette(AUX_PALETTE, i);
            let mut out = c;
            if c & 0x1f != d & 0x1f {
                out = out.wrapping_sub(1);
            }
            if c & 0x3e0 != d & 0x3e0 {
                out = out.wrapping_sub(0x20);
            }
            if c & 0x7c00 != d & 0x7c00 {
                out = out.wrapping_sub(0x400);
            }
            self.set_palette(MAIN_PALETTE, i, out);
        }
    }

    pub fn reset_hud_palettes_4_and_5(&mut self) {
        for i in 16..24 {
            self.set_palette(MAIN_PALETTE, i, 0);
        }
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.write16(DARKENING_OR_LIGHTENING, 2);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_history(&mut self) {
        self.palette_filter_range(0x10, 0x18);
        self.palette_filter_incr_countdown();
    }

    pub fn palette_filter_wish_ponds(&mut self) {
        self.write8(0x1d, 2);
        self.write8(0x9a, 0x30);
        self.palette_filter_wish_ponds_inner();
    }

    pub fn palette_filter_crystal(&mut self) {
        self.write8(0x1d, 1);
        self.palette_filter_wish_ponds_inner();
    }

    pub fn palette_filter_wish_ponds_inner(&mut self) {
        for i in 0xd0..0xd8 {
            self.set_palette(MAIN_PALETTE, i, 0);
        }
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.write16(DARKENING_OR_LIGHTENING, 2);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_restore_sp5f(&mut self) {
        for i in 208..216 {
            self.set_palette(MAIN_PALETTE, i, self.palette(AUX_PALETTE, i));
        }
        self.write8(0x1d, 0);
        self.write8(0x9a, 32);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_sp5f(&mut self) {
        for _ in 0..2 {
            self.palette_filter_range(208, 216);
            self.palette_filter_incr_countdown();
            if self.read16(PALETTE_FILTER_COUNTDOWN) == 0 {
                break;
            }
        }
    }

    pub fn kholdstare_shell_palette_filtering(&mut self) {
        let features = u32::from_le_bytes(self.wram[FEATURES0..FEATURES0 + 4].try_into().unwrap());
        let start = if features & 4096 != 0 { 0x50 } else { 0x40 };
        if self.read8(SUBSUBMODULE) == 0 {
            for i in start..start + 8 {
                self.set_palette(MAIN_PALETTE, i, self.palette(AUX_PALETTE, i));
            }
            self.write16(PALETTE_FILTER_COUNTDOWN, 0);
            self.write16(DARKENING_OR_LIGHTENING, 0);
            self.increment8(FLAG_UPDATE_CGRAM);
            self.write8(SUBSUBMODULE, 1);
            return;
        }
        for _ in 0..2 {
            self.palette_filter_range(start, start + 8);
            self.palette_filter_incr_countdown();
            if self.read16(PALETTE_FILTER_COUNTDOWN) == 0 {
                self.write8(0x1d, 0);
                break;
            }
        }
    }

    pub fn agahnim_warp_shadow_filter(&mut self, index: usize, settings_addr: usize) {
        self.write16(
            PALETTE_FILTER_COUNTDOWN,
            self.read16(settings_addr + index * 2),
        );
        self.write16(
            DARKENING_OR_LIGHTENING,
            self.read16(settings_addr + (index + 3) * 2),
        );
        let start = usize::from(PALETTE_FILTER_AGAHNIM[index] >> 1);
        for _ in 0..2 {
            self.palette_filter_range(start, start + 8);
            let next = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
            self.write16(PALETTE_FILTER_COUNTDOWN, next);
            if next == 0x1f {
                self.write16(PALETTE_FILTER_COUNTDOWN, 0);
                self.write16(
                    DARKENING_OR_LIGHTENING,
                    self.read16(DARKENING_OR_LIGHTENING) ^ 2,
                );
                break;
            }
        }
        self.write16(
            settings_addr + index * 2,
            self.read16(PALETTE_FILTER_COUNTDOWN),
        );
        self.write16(
            settings_addr + (index + 3) * 2,
            self.read16(DARKENING_OR_LIGHTENING),
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_fade_intro_one_step(&mut self) {
        self.palette_filter_restore_additive(0x100, 0x1a0);
        self.palette_filter_restore_additive(0xc0, 0x100);
        self.write8(
            PALETTE_FILTER_COUNTDOWN,
            self.read8(PALETTE_FILTER_COUNTDOWN).wrapping_sub(1),
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_fade_intro_2(&mut self) {
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.write8(
            PALETTE_FILTER_COUNTDOWN,
            self.read8(PALETTE_FILTER_COUNTDOWN).wrapping_sub(1),
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_initialize_white_filter(&mut self) {
        for i in 0..256 {
            self.set_palette(AUX_PALETTE, i, 0x7fff);
        }
        self.set_palette(MAIN_PALETTE, 32, self.palette(MAIN_PALETTE, 0));
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.write16(DARKENING_OR_LIGHTENING, 2);
        if self.read16(OVERWORLD_SCREEN) == 27 {
            for i in [0, 32] {
                self.set_palette(AUX_PALETTE, i, 0);
                self.set_palette(MAIN_PALETTE, i, 0);
            }
        }
        self.write8(0x6bb, 8); // mirror_vars.ctr
        self.write8(0x6ba, 0); // mirror_vars.ctr2
    }

    pub fn mirror_warp_run_animation_submodules(&mut self) {
        let counter = self.read8(0x6bb).wrapping_sub(1);
        self.write8(0x6bb, counter);
        if counter != 0 {
            self.animate_mirror_warp();
            return;
        }
        self.write8(0x6bb, 2);
        self.palette_filter_blinding_white();
    }

    pub fn palette_filter_blinding_white(&mut self) {
        if self.read16(DARKENING_OR_LIGHTENING) == 0xff {
            return;
        }
        if self.read16(DARKENING_OR_LIGHTENING) == 2 {
            self.palette_filter_restore_additive(0x40, 0x1b0);
            self.palette_filter_restore_additive(0x1c0, 0x1e0);
        } else {
            self.palette_filter_restore_subtractive(0x40, 0x1b0);
            self.palette_filter_restore_subtractive(0x1c0, 0x1e0);
        }
        self.palette_filter_start_blinding_white();
    }

    pub fn palette_filter_start_blinding_white(&mut self) {
        self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
        let next = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, next);
        if self.read16(DARKENING_OR_LIGHTENING) == 0 {
            if next == 66 {
                self.write16(DARKENING_OR_LIGHTENING, 0xff);
                self.write8(0x6bb, 32);
            }
        } else if next == 31 {
            self.write16(
                DARKENING_OR_LIGHTENING,
                self.read16(DARKENING_OR_LIGHTENING) ^ 2,
            );
            if self.read8(0x10) != 21 {
                return;
            }
            self.hooks.ppu_write(0x0c, 0);
            self.write8(HDMAEN_COPY, 0);
            for i in 0..240 {
                self.write16(HDMA_TABLE_DYNAMIC + i * 2, 0x778);
            }
            self.write8(HDMAEN_COPY, 0xc0);
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_blinding_white_triforce(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x200);
        self.palette_filter_start_blinding_white();
    }

    pub fn trinexx_flash_shell_palette_red(&mut self) {
        self.trinexx_red(true);
    }
    pub fn trinexx_unflash_shell_palette_red(&mut self) {
        self.trinexx_red(false);
    }
    fn trinexx_red(&mut self, flash: bool) {
        if self.read8(0x4be) == 0 {
            for i in 0x41..0x48 {
                let target = self.palette(AUX_PALETTE, i) & 0x1f;
                let value = self.palette(MAIN_PALETTE, i);
                let component = value & 0x1f;
                let next = if flash {
                    component + u16::from(component != 0x1f)
                } else {
                    component - u16::from(component != target)
                };
                self.set_palette(MAIN_PALETTE, i, value & 0xffe0 | next);
            }
            self.increment8(FLAG_UPDATE_CGRAM);
            let count = self.read8(0x4c0).wrapping_add(1);
            self.write8(0x4c0, count);
            if count >= 12 {
                self.write8(0x4c0, 0);
                self.write8(0x4be, 0);
                return;
            }
            self.write8(0x4be, 3);
        }
        self.write8(0x4be, self.read8(0x4be).wrapping_sub(1));
    }

    pub fn trinexx_flash_shell_palette_blue(&mut self) {
        self.trinexx_blue(true);
    }
    pub fn trinexx_unflash_shell_palette_blue(&mut self) {
        self.trinexx_blue(false);
    }
    fn trinexx_blue(&mut self, flash: bool) {
        if self.read8(0x4bf) == 0 {
            for i in 0x41..0x48 {
                let target = self.palette(AUX_PALETTE, i) & 0x7c00;
                let value = self.palette(MAIN_PALETTE, i);
                let component = value & 0x7c00;
                let next = if flash {
                    component + (u16::from(component != 0x7c00) << 10)
                } else {
                    component - (u16::from(component != target) << 10)
                };
                self.set_palette(MAIN_PALETTE, i, value & !0x7c00 | next);
            }
            self.increment8(FLAG_UPDATE_CGRAM);
            let count = self.read8(0x4c1).wrapping_add(1);
            self.write8(0x4c1, count);
            if count >= 12 {
                self.write8(0x4c1, 0);
                self.write8(0x4bf, 0);
                return;
            }
            self.write8(0x4bf, 3);
        }
        self.write8(0x4bf, self.read8(0x4bf).wrapping_sub(1));
    }

    pub fn palette_filter_restore_bg_subtractive_strict(&mut self) {
        if self.read16(DARKENING_OR_LIGHTENING) == 255 {
            return;
        }
        self.palette_filter_restore_subtractive(0x40, 0x100);
        let next = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, next);
        if next == 0x20 {
            self.write16(DARKENING_OR_LIGHTENING, 255);
            self.write16(0x1d, 0);
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_restore_bg_additive_strict(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x100);
        self.write16(
            PALETTE_FILTER_COUNTDOWN,
            self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1),
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn filter_majorly_whiten_color(&self, color: u16) -> u16 {
        let features = u32::from_le_bytes(self.wram[FEATURES0..FEATURES0 + 4].try_into().unwrap());
        let amount = if features & 65_536 != 0 { 3 } else { 14 };
        let red = (usize::from(color & 0x1f) + amount).min(0x1f) as u16;
        let green = (usize::from(color & 0x3e0) + (amount << 5)).min(0x3e0) as u16;
        let blue = (usize::from(color & 0x7c00) + (amount << 10)).min(0x7c00) as u16;
        red | green | blue
    }

    pub fn filter_majorly_whiten_bg(&mut self) {
        for i in 32..128 {
            let color = self.filter_majorly_whiten_color(self.palette(AUX_PALETTE, i));
            self.set_palette(MAIN_PALETTE, i, color);
        }
        let backdrop = if self.palette(AUX_PALETTE, 0) != 0 {
            self.palette(MAIN_PALETTE, 32)
        } else {
            0
        };
        self.set_palette(MAIN_PALETTE, 0, backdrop);
    }

    pub fn palette_set_bg_and_fixed_color(&mut self, color: u16) {
        for base in [MAIN_PALETTE, AUX_PALETTE] {
            self.set_palette(base, 0, color);
            self.set_palette(base, 32, color);
        }
        self.set_backdrop_color_black();
    }

    pub fn palette_bg_and_fixed_color_black(&mut self) {
        self.palette_set_bg_and_fixed_color(0);
    }

    pub fn set_backdrop_color_black(&mut self) {
        self.write8(0x9c, 0x20);
        self.write8(0x9d, 0x40);
        self.write8(0x9e, 0x80);
    }

    pub fn palette_get_ow_bg_color(&self) -> u16 {
        let screen = self.read16(OVERWORLD_SCREEN);
        if screen < 0x80 {
            return if screen & 0x40 != 0 { 0x2a32 } else { 0x2669 };
        }
        match self.read16(DUNGEON_ROOM) {
            0x180 | 0x182 | 0x183 => 0x19c6,
            _ => 0x2669,
        }
    }

    pub fn palette_set_ow_bg_color(&mut self) {
        let color = self.palette_get_ow_bg_color();
        self.palette_set_bg_and_fixed_color(color);
    }

    pub fn palette_special_ow(&mut self) {
        let color = self.palette_get_ow_bg_color();
        self.set_palette(AUX_PALETTE, 0, color);
        self.set_palette(AUX_PALETTE, 32, color);
        self.set_backdrop_color_black();
    }

    pub fn palette_set_translucency_swap(&mut self, value: bool) {
        self.write8(PALETTE_SWAP_FLAG, u8::from(value));
        for i in 0..8 {
            for (a, b) in [(0x80, 0xf0), (0x88, 0xf8), (0xb8, 0xd8)] {
                let ca = self.palette(AUX_PALETTE, i + a);
                let cb = self.palette(AUX_PALETTE, i + b);
                self.set_palette(AUX_PALETTE, i + b, ca);
                self.set_palette(MAIN_PALETTE, i + b, ca);
                self.set_palette(AUX_PALETTE, i + a, cb);
                self.set_palette(MAIN_PALETTE, i + a, cb);
            }
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_assert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(true);
    }
    pub fn palette_revert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(false);
    }

    pub fn overworld_copy_palettes_to_cache(&mut self) {
        self.wram
            .copy_within(AUX_PALETTE..AUX_PALETTE + 512, MAIN_PALETTE);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn special_overworld_copy_palettes_to_cache(&mut self) {
        for i in 32..256 {
            self.set_palette(MAIN_PALETTE, i, 0);
        }
        for i in 0..8 {
            for offset in [0, 8, 0x10, 0x18, 0xd8, 0xe8, 0xf0, 0xf8] {
                self.set_palette(
                    MAIN_PALETTE,
                    i + offset,
                    self.palette(AUX_PALETTE, i + offset),
                );
            }
        }
        self.write8(MOSAIC_COPY, 0xf7);
        self.write8(MOSAIC_LEVEL, 0xf7);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_restore_coldata(&mut self) {
        if self.read8(0x1b) != 0 {
            return;
        }
        let rgb = match self.read8(OVERWORLD_SCREEN) {
            3 | 5 | 7 => 0x8c4c26u32,
            0x43 | 0x45 | 0x47 => 0x874a26,
            0x5b => 0x894f33,
            _ => 0x804020,
        };
        self.write8(0x9c, rgb as u8);
        self.write8(0x9d, (rgb >> 8) as u8);
        self.write8(0x9e, (rgb >> 16) as u8);
    }

    pub fn palette_restore_bg_from_flash(&mut self) {
        self.wram
            .copy_within(AUX_PALETTE + 64..AUX_PALETTE + 256, MAIN_PALETTE + 64);
        self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
        self.palette_restore_coldata();
    }

    pub fn palette_restore_bg_and_hud(&mut self) {
        self.wram
            .copy_within(AUX_PALETTE..AUX_PALETTE + 256, MAIN_PALETTE);
        self.increment8(FLAG_UPDATE_CGRAM);
        self.palette_restore_coldata();
    }

    pub fn palette_load_single(&mut self, source: &[u16], destination: usize, entries: usize) {
        let destination = (destination + usize::from(self.read16(0xaa8))) / 2;
        for (i, &color) in source[..=entries].iter().enumerate() {
            self.set_palette(AUX_PALETTE, destination + i, color);
        }
    }

    pub fn palette_load_multiple(
        &mut self,
        source: &[u16],
        destination: usize,
        entries: usize,
        palettes: usize,
    ) {
        let width = entries + 1;
        for row in 0..=palettes {
            let destination = (destination + row * 0x20 + usize::from(self.read16(0xaa8))) / 2;
            for (i, &color) in source[row * width..(row + 1) * width].iter().enumerate() {
                self.set_palette(AUX_PALETTE, destination + i, color);
            }
        }
    }

    pub fn palette_load_multiple_arbitrary(
        &mut self,
        source: &[u16],
        destination: usize,
        entries: usize,
    ) {
        self.load_gear_palette(destination, source, entries + 1);
    }

    pub fn load_gear_palette(&mut self, destination: usize, source: &[u16], entries: usize) {
        for (i, &color) in source[..entries].iter().enumerate() {
            self.set_palette(AUX_PALETTE, destination / 2 + i, color);
            self.set_palette(MAIN_PALETTE, destination / 2 + i, color);
        }
    }

    fn load_asset_single(
        &mut self,
        asset: usize,
        offset: usize,
        destination: usize,
        entries: usize,
    ) {
        let colors = self.asset_words(asset, offset, entries + 1);
        self.palette_load_single(&colors, destination, entries);
    }
    fn load_asset_multiple(
        &mut self,
        asset: usize,
        offset: usize,
        destination: usize,
        entries: usize,
        palettes: usize,
    ) {
        let colors = self.asset_words(asset, offset, (entries + 1) * (palettes + 1));
        self.palette_load_multiple(&colors, destination, entries, palettes);
    }
    fn load_asset_arbitrary(
        &mut self,
        asset: usize,
        offset: usize,
        destination: usize,
        entries: usize,
    ) {
        let colors = self.asset_words(asset, offset, entries + 1);
        self.palette_load_multiple_arbitrary(&colors, destination, entries);
    }

    pub fn palette_load_sp0l(&mut self) {
        let offset = usize::from(self.read8(0xaac)) * 7;
        let destination = if self.read8(PALETTE_SWAP_FLAG) != 0 {
            0x1e2
        } else {
            0x102
        };
        self.load_asset_single(ASSET_PAL_SPR_AUX3, offset, destination, 6);
    }
    pub fn palette_load_sprite_main(&mut self) {
        let offset = if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            60
        } else {
            0
        };
        self.load_asset_multiple(ASSET_PAL_MAIN_SPR, offset, 0x122, 14, 3);
    }
    pub fn palette_load_sp5l(&mut self) {
        self.load_asset_single(
            ASSET_PAL_SPR_AUX1,
            usize::from(self.read8(0xaad)) * 7,
            0x1a2,
            6,
        );
    }
    pub fn palette_load_sp6l(&mut self) {
        self.load_asset_single(
            ASSET_PAL_SPR_AUX1,
            usize::from(self.read8(0xaae)) * 7,
            0x1c2,
            6,
        );
    }
    pub fn palette_load_sword(&mut self) {
        let sword = self.read8(0xf359);
        let offset = if sword as i8 > 0 {
            usize::from(sword - 1) * 3
        } else {
            0
        };
        self.load_asset_arbitrary(ASSET_PAL_SWORD, offset, 0x1b2, 2);
        self.increment8(FLAG_UPDATE_CGRAM);
    }
    pub fn palette_load_shield(&mut self) {
        let shield = self.read8(0xf35a);
        self.load_asset_arbitrary(
            ASSET_PAL_SHIELD,
            usize::from(shield.saturating_sub(1)) * 4,
            0x1b8,
            3,
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }
    pub fn palette_load_sprite_environment(&mut self) {
        if self.read8(0x1b) != 0 {
            self.palette_load_sprite_environment_dungeon();
        } else {
            self.palette_misc_sprite_outdoors();
        }
    }
    pub fn palette_load_sprite_environment_dungeon(&mut self) {
        self.load_asset_single(
            ASSET_PAL_MISC_SPR,
            usize::from(self.read8(0xab1)) * 7,
            0x1d2,
            6,
        );
    }
    pub fn palette_misc_sprite_outdoors(&mut self) {
        let index = if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            9
        } else {
            7
        };
        let destination = if self.read8(PALETTE_SWAP_FLAG) != 0 {
            0x1f2
        } else {
            0x112
        };
        self.load_asset_single(ASSET_PAL_MISC_SPR, index * 7, destination, 6);
        self.load_asset_single(ASSET_PAL_MISC_SPR, (index - 1) * 7, 0x1d2, 6);
    }
    pub fn palette_load_dungeon_map_sprite(&mut self) {
        self.load_asset_multiple(ASSET_PAL_MAP_SPR, 0, 0x182, 6, 2);
    }
    pub fn palette_load_link_armor_and_gloves(&mut self) {
        self.load_asset_arbitrary(
            ASSET_PAL_ARMOR_GLOVES,
            usize::from(self.read8(0xf35b)) * 15,
            0x1e2,
            14,
        );
        self.palette_update_gloves_color();
    }
    pub fn palette_update_gloves_color(&mut self) {
        let gloves = self.read8(0xf354);
        if gloves != 0 {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(
                    base,
                    0xfd,
                    self.assets.gloves_color(usize::from(gloves - 1)),
                );
            }
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }
    pub fn palette_load_dungeon_map_bg(&mut self) {
        self.load_asset_multiple(ASSET_PAL_MAP_BG, 0, 0x40, 15, 5);
    }
    pub fn palette_load_hud(&mut self) {
        self.load_asset_multiple(ASSET_HUD_PAL, usize::from(self.read8(0xab2)) * 32, 0, 15, 1);
    }
    pub fn palette_load_dungeon_set(&mut self) {
        let offset = usize::from(self.read8(0xab6) >> 1) * 90;
        self.load_asset_multiple(ASSET_PAL_DUNG_BG_MAIN, offset, 0x42, 14, 5);
        let destination = if self.read8(PALETTE_SWAP_FLAG) != 0 {
            0x1f2
        } else {
            0x112
        };
        self.load_asset_single(ASSET_PAL_DUNG_BG_MAIN, offset, destination, 6);
    }
    pub fn palette_load_owbg3(&mut self) {
        self.load_asset_single(
            ASSET_PAL_OW_AUX3,
            usize::from(self.read8(0xab8)) * 7,
            0xe2,
            6,
        );
    }
    pub fn palette_load_owbg_main(&mut self) {
        self.load_asset_multiple(
            ASSET_PAL_OW_MAIN,
            usize::from(self.read8(0xab3)) * 35,
            0x42,
            6,
            4,
        );
    }
    pub fn palette_load_owbg1(&mut self) {
        self.load_asset_multiple(
            ASSET_PAL_OW_AUX12,
            usize::from(self.read8(0xab4)) * 21,
            0x52,
            6,
            2,
        );
    }
    pub fn palette_load_owbg2(&mut self) {
        self.load_asset_multiple(
            ASSET_PAL_OW_AUX12,
            usize::from(self.read8(0xab5)) * 21,
            0xb2,
            6,
            2,
        );
    }

    pub fn load_gear_palettes(&mut self, sword: u8, shield: u8, armor: u8) {
        let sword_offset = if sword != 0 && sword != 255 {
            usize::from(sword - 1) * 3
        } else {
            0
        };
        self.load_asset_arbitrary(ASSET_PAL_SWORD, sword_offset, 0x1b2, 2);
        self.load_asset_arbitrary(
            ASSET_PAL_SHIELD,
            usize::from(shield.saturating_sub(1)) * 4,
            0x1b8,
            3,
        );
        self.load_asset_arbitrary(ASSET_PAL_ARMOR_GLOVES, usize::from(armor) * 15, 0x1e2, 14);
        self.increment8(FLAG_UPDATE_CGRAM);
    }
    pub fn load_actual_gear_palettes(&mut self) {
        self.load_gear_palettes(self.read8(0xf359), self.read8(0xf35a), self.read8(0xf35b));
        let features = u32::from_le_bytes(self.wram[FEATURES0..FEATURES0 + 4].try_into().unwrap());
        if features & 4096 != 0 {
            self.palette_update_gloves_color();
        }
    }
    pub fn palette_electro_themed_gear(&mut self) {
        self.load_gear_palettes(2, 2, 4);
    }
    pub fn load_gear_palettes_bunny(&mut self) {
        self.load_gear_palettes(self.read8(0xf359), self.read8(0xf35a), 3);
    }

    pub fn palette_load_agahnim(&mut self) {
        for destination in [0x162, 0x182, 0x1a2] {
            self.load_asset_arbitrary(ASSET_PAL_SPR_AUX1, 14 * 7, destination, 6);
        }
        self.load_asset_arbitrary(ASSET_PAL_SPR_AUX1, 21 * 7, 0x1c2, 6);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_load_for_file_select(&mut self) {
        for slot in 0..3 {
            let (armor, gloves, sword, shield) = self.hooks.file_select_gear(slot);
            let offset = slot * 0x20;
            self.palette_load_for_file_select_armor(offset, armor, gloves);
            self.palette_load_for_file_select_sword(offset, sword);
            self.palette_load_for_file_select_shield(offset, shield);
        }
        let main = self.asset_words(ASSET_PAL_MAIN_SPR, 0, 30);
        for i in 0..7 {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(base, 0xe8 + i, main[7 + i]);
                self.set_palette(base, 0xf8 + i, main[22 + i]);
            }
        }
    }
    pub fn palette_load_for_file_select_armor(&mut self, offset: usize, armor: u8, gloves: u8) {
        let colors = self.asset_words(ASSET_PAL_ARMOR_GLOVES, usize::from(armor) * 15, 15);
        for (i, color) in colors.into_iter().enumerate() {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(base, offset + 0x81 + i, color);
            }
        }
        if gloves != 0 {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(
                    base,
                    offset + 0x8d,
                    self.assets.gloves_color(usize::from(gloves - 1)),
                );
            }
        }
    }
    pub fn palette_load_for_file_select_sword(&mut self, offset: usize, sword: u8) {
        let colors = self.asset_words(ASSET_PAL_SWORD, usize::from(sword.saturating_sub(1)) * 3, 3);
        for (i, color) in colors.into_iter().enumerate() {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(base, offset + 0x99 + i, color);
            }
        }
    }
    pub fn palette_load_for_file_select_shield(&mut self, offset: usize, shield: u8) {
        let colors = self.asset_words(
            ASSET_PAL_SHIELD,
            usize::from(shield.saturating_sub(1)) * 4,
            4,
        );
        for (i, color) in colors.into_iter().enumerate() {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(base, offset + 0x9c + i, color);
            }
        }
    }

    pub fn overworld_load_palettes(&mut self, bg: u8, sprites: u8) {
        self.write16(0xaa8, 0);
        let bg_offset = usize::from(bg) * 3;
        // The credits scene requests palette 167; the C table access runs into
        // adjacent static data. Preserve the observed C oracle bytes here.
        let bg_info = if bg == 167 {
            [0, -49, -67]
        } else {
            [
                OW_BG_PAL_INFO.get(bg_offset).copied().unwrap_or(-1),
                OW_BG_PAL_INFO.get(bg_offset + 1).copied().unwrap_or(-1),
                OW_BG_PAL_INFO.get(bg_offset + 2).copied().unwrap_or(-1),
            ]
        };
        for (address, value) in [
            (0xab4, bg_info[0]),
            (0xab5, bg_info[1]),
            (0xab8, bg_info[2]),
        ] {
            if value >= 0 {
                self.write8(address, value as u8);
            }
        }
        let sprite_offset = usize::from(sprites) * 2;
        for (address, value) in [
            (0xaad, OW_SPR_PAL_INFO[sprite_offset]),
            (0xaae, OW_SPR_PAL_INFO[sprite_offset + 1]),
        ] {
            if value >= 0 {
                self.write8(address, value as u8);
            }
        }
        self.palette_load_owbg1();
        self.palette_load_owbg2();
        self.palette_load_owbg3();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
    }

    pub fn overworld_load_all_palettes(&mut self) {
        self.wram[AUX_PALETTE + 0x180..AUX_PALETTE + 0x200].fill(0);
        self.wram[MAIN_PALETTE..MAIN_PALETTE + 512].fill(0);
        for (address, value) in [
            (0xab3, 5),
            (0xab4, 3),
            (0xab5, 3),
            (0xab8, 0),
            (0xab1, 5),
            (0xaac, 11),
            (PALETTE_SWAP_FLAG, 0),
        ] {
            self.write8(address, value);
        }
        self.write16(0xaa8, 0);
        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_owbg_main();
        self.palette_load_owbg1();
        self.palette_load_owbg2();
        self.palette_load_owbg3();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();
        for i in 0..8 {
            self.set_palette(MAIN_PALETTE, 0xd8 + i, self.palette(AUX_PALETTE, 0xe8 + i));
        }
    }

    pub fn dungeon_load_palettes(&mut self) {
        self.write16(0xaa8, 0);
        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_sprite_environment();
        self.palette_load_link_armor_and_gloves();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
        self.overworld_load_palettes_inner();
    }

    pub fn overworld_load_palettes_inner(&mut self) {
        self.write8(0xc20a, self.read8(0xab6));
        self.write8(0xc20b, self.read8(0xab8));
        self.write8(0xc20c, self.read8(0xab7));
        self.write16(DARKENING_OR_LIGHTENING, 2);
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.write16(MOSAIC_TARGET_LEVEL, 0);
        self.overworld_copy_palettes_to_cache();
    }

    pub fn overworld_load_screens_palette_set(&mut self) {
        let screen = self.read8(OVERWORLD_SCREEN) & 0x3f;
        let mut index = if matches!(screen, 3 | 5 | 7) { 2 } else { 0 };
        if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            index += 1;
        }
        self.overworld_load_area_palettes_ex(index);
    }

    pub fn overworld_load_area_palettes_ex(&mut self, index: u8) {
        self.write8(0xab3, index);
        self.write16(0xaa8, self.read16(0xaa8) & 0xff);
        self.palette_load_sprite_main();
        self.palette_load_sprite_environment();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_link_armor_and_gloves();
        self.write8(0xaac, if self.read8(0xf3ca) & 0x40 != 0 { 3 } else { 1 });
        self.palette_load_sp0l();
        self.palette_load_hud();
        self.palette_load_owbg_main();
    }

    pub fn dungeon_handle_translucency_and_palette(&mut self) {
        if self.read8(PALETTE_SWAP_FLAG) != 0 {
            self.palette_revert_translucency_swap();
        }
        self.write8(0x99, 2);
        self.write8(0x9a, 0xb3);
        let mut torch = self.read8(0x45a);
        if self.read8(0xc005) == 0 {
            let property = self.read8(0x414);
            let mut color_math = match property {
                7 => 0x32,
                4 => 0x62,
                _ => 0x20,
            };
            if property == 2 {
                self.palette_assert_translucency_swap();
                if self.read8(DUNGEON_ROOM) == 13 {
                    self.wram[0xc019..0xc025].fill(0);
                    self.palette_load_agahnim();
                }
                color_math = 0x70;
            }
            self.write8(0x9a, color_math);
            torch = 3;
        }
        self.write8(0xc017, LIT_TORCHES_COLOR_PLUS[usize::from(torch)]);
        self.write16(PALETTE_FILTER_COUNTDOWN, 31);
        self.write8(MOSAIC_TARGET_LEVEL, 0);
        self.write16(DARKENING_OR_LIGHTENING, 2);
        self.write16(0xaa8, 0);
        self.palette_load_dungeon_set();
        self.palette_load_sp0l();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.increment8(SUBSUBMODULE);
    }

    pub fn handle_screen_flash(&mut self) {
        let count = self.read8(0xff9);
        if count == 0 || self.read8(0x11) != 0 {
            return;
        }
        self.write8(0xff9, count - 1);
        if count == 1 {
            self.palette_restore_bg_and_hud();
            return;
        }
        if count & 1 != 0 {
            self.filter_majorly_whiten_bg();
        } else {
            self.palette_restore_bg_from_flash();
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_whirlpool_blue(&mut self) {
        if self.read8(0x1a) & 1 != 0 {
            for i in 0x20..0x100 {
                let mut color = self.palette(MAIN_PALETTE, i);
                if color & 0x7c00 != 0x7c00 {
                    color = color.wrapping_add(0x400);
                }
                self.set_palette(MAIN_PALETTE, i, color);
            }
            self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
            let countdown = self.read16(PALETTE_FILTER_COUNTDOWN);
            if countdown & 1 == 0 {
                self.write8(MOSAIC_LEVEL, self.read8(MOSAIC_LEVEL).wrapping_add(16));
            }
            self.write16(PALETTE_FILTER_COUNTDOWN, countdown.wrapping_add(1));
            if countdown.wrapping_add(1) == 31 {
                self.write16(PALETTE_FILTER_COUNTDOWN, 0);
                self.increment8(SUBSUBMODULE);
                self.write8(MOSAIC_LEVEL, 0xf0);
            }
        }
        self.write8(BGMODE_COPY, 9);
        self.write8(MOSAIC_COPY, self.read8(MOSAIC_LEVEL) | 3);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_isolate_whirlpool_blue(&mut self) {
        for i in 0x20..0x100 {
            let mut color = self.palette(MAIN_PALETTE, i);
            if color & 0x3e0 != 0 {
                color = color.wrapping_sub(0x20);
            }
            if color & 0x1f != 0 {
                color = color.wrapping_sub(1);
            }
            self.set_palette(MAIN_PALETTE, i, color);
        }
        self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
        let countdown = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, countdown);
        if countdown == 31 {
            self.write16(PALETTE_FILTER_COUNTDOWN, 0);
            self.increment8(SUBSUBMODULE);
            self.write8(MOSAIC_LEVEL, 0xf0);
        }
        self.write8(BGMODE_COPY, 9);
        self.write8(MOSAIC_COPY, self.read8(MOSAIC_LEVEL) | 3);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_whirlpool_restore_blue(&mut self) {
        if self.read8(0x1a) & 1 != 0 {
            for i in 0x20..0x100 {
                let target = self.palette(AUX_PALETTE, i) & 0x7c00;
                let mut color = self.palette(MAIN_PALETTE, i);
                if color & 0x7c00 != target {
                    color = color.wrapping_sub(0x400);
                }
                self.set_palette(MAIN_PALETTE, i, color);
            }
            self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
            let countdown = self.read16(PALETTE_FILTER_COUNTDOWN);
            if countdown & 1 == 0 {
                self.write8(MOSAIC_LEVEL, self.read8(MOSAIC_LEVEL).wrapping_sub(16));
            }
            self.write16(PALETTE_FILTER_COUNTDOWN, countdown.wrapping_add(1));
            if countdown.wrapping_add(1) == 31 {
                self.write16(PALETTE_FILTER_COUNTDOWN, 0);
                self.increment8(SUBSUBMODULE);
                self.write8(MOSAIC_LEVEL, 0);
            }
        }
        self.write8(BGMODE_COPY, 9);
        self.write8(MOSAIC_COPY, self.read8(MOSAIC_LEVEL) | 3);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_whirlpool_restore_red_green(&mut self) {
        for i in 0x20..0x100 {
            let target = self.palette(AUX_PALETTE, i);
            let mut color = self.palette(MAIN_PALETTE, i);
            if color & 0x3e0 != target & 0x3e0 {
                color = color.wrapping_add(0x20);
            }
            if color & 0x1f != target & 0x1f {
                color = color.wrapping_add(1);
            }
            self.set_palette(MAIN_PALETTE, i, color);
        }
        self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
        let countdown = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, countdown);
        if countdown == 31 {
            self.write16(PALETTE_FILTER_COUNTDOWN, 0);
            self.increment8(SUBSUBMODULE);
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn iris_spotlight_calculate_circle_value(&self, radius: u8) -> u16 {
        let spotlight_radius = self.read16(0x67c) as u8;
        let index = (Self::snes_divide(u16::from(radius) << 8, spotlight_radius) >> 1) as u8;
        let curve = SPOTLIGHT_CIRCLE[usize::from(index)];
        let scaled = ((u16::from(curve) * u16::from(spotlight_radius)) >> 8) as u8;
        let spread = 2 * u16::from(scaled);
        if curve == 0 {
            return 0xff;
        }
        let center = self.read16(0x670);
        let left = center.wrapping_sub(spread);
        let left = if (left as i16) < 0 { 0 } else { left.min(255) };
        let right = center.wrapping_add(spread).min(255);
        let value = left | right << 8;
        if value == 0xffff { 0xff } else { value }
    }

    pub fn iris_spotlight_reset_table(&mut self) {
        for i in 0..240 {
            self.write16(HDMA_TABLE_DYNAMIC + i * 2, 0xff00);
        }
    }

    pub fn iris_spotlight_close(&mut self) {
        self.spotlight_internal(0x7e, 0);
    }
    pub fn spotlight_open(&mut self) {
        self.spotlight_internal(0, 2);
    }

    pub fn spotlight_internal(&mut self, radius: u8, mode: u8) {
        self.write16(0x67c, u16::from(radius));
        self.write16(0x67e, u16::from(mode));
        self.hooks.ppu_write(0x0c, 0);
        self.hooks.setup_hdma(0xf2fb, 0, 0x26);
        self.write8(0x96, 0x33);
        self.write8(0x97, 3);
        self.write8(0x98, 0x33);
        self.write8(0x1e, self.read8(0x1c));
        self.write8(0x1f, self.read8(0x1d));
        if self.read8(0x1b) == 0 {
            self.set_backdrop_color_black();
        }
        self.iris_spotlight_configure_table();
        self.write8(HDMAEN_COPY, 0x80);
        self.write8(INIDISP_COPY, 0xf);
    }

    pub fn iris_spotlight_configure_table(&mut self) {
        let center_y = self
            .read16(0x20)
            .wrapping_sub(self.read16(0xe8))
            .wrapping_add(12);
        let radius = self.read16(0x67c);
        self.write16(0x674, center_y.wrapping_sub(radius));
        self.write16(0x676, center_y.wrapping_add(radius));
        self.write16(
            0x670,
            self.read16(0x22)
                .wrapping_sub(self.read16(0xe2))
                .wrapping_add(8),
        );
        self.write16(0x67a, radius);
        let mut lower = center_y.wrapping_mul(2);
        if lower < 224 {
            lower = 224;
        }
        let mut upper = center_y.wrapping_mul(2).wrapping_sub(lower);
        loop {
            let mut value = 0xff;
            if lower < self.read16(0x676) {
                let r = self.read8(0x67a);
                if self.read16(0x67a) != 0 {
                    self.write16(0x67a, self.read16(0x67a).wrapping_sub(1));
                }
                value = self.iris_spotlight_calculate_circle_value(r);
            }
            if upper < 240 {
                self.write16(HDMA_TABLE_DYNAMIC + usize::from(upper) * 2, value);
            }
            if lower < 240 {
                self.write16(HDMA_TABLE_DYNAMIC + usize::from(lower) * 2, value);
            }
            if upper == center_y {
                break;
            }
            upper = upper.wrapping_add(1);
            lower = lower.wrapping_sub(1);
        }
        for i in 224..240 {
            self.write16(HDMA_TABLE_DYNAMIC + i * 2, 0);
        }
        self.wram
            .copy_within(HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 448, 0x17000);
        let mode = usize::from(self.read8(0x67e) >> 1);
        self.write16(
            0x67c,
            self.read16(0x67c)
                .wrapping_add_signed(i16::from(SPOTLIGHT_DELTA_SIZE[mode])),
        );
        if self.read16(0x67c) != u16::from(SPOTLIGHT_GOAL[mode]) {
            return;
        }
        if self.read16(0x67e) == 0 {
            self.write8(INIDISP_COPY, 0x80);
        } else {
            self.iris_spotlight_reset_table();
        }
        self.write8(SUBSUBMODULE, 0);
        self.write8(0x11, 0);
        if matches!(self.read8(0x10), 7 | 16) {
            if self.read8(0x1b) == 0 {
                self.write8(
                    0x12d,
                    self.read8(0x15b00 + usize::from(self.read8(OVERWORLD_SCREEN))) >> 4,
                );
            }
            if self.read8(0x132) != 0xff {
                self.write8(0x12c, self.read8(0x132));
            }
        }
        self.write8(0x10, self.read8(0x10c));
        if self.read8(0x10) == 6 {
            self.hooks.sprite_reset_all(self.wram);
        }
    }

    pub fn adjust_water_hdma_window(&mut self) {
        let center = self.read16(0x682).wrapping_sub(self.read16(0xe8));
        self.write16(0x674, center.wrapping_sub(self.read16(0x684)));
        self.write16(0x676, center.wrapping_add(self.read16(0x684)));
        self.adjust_water_hdma_window_x(center);
    }

    pub fn adjust_water_hdma_window_x(&mut self, center_y: u16) {
        let center_x = self.read16(0x680).wrapping_sub(self.read16(0xe2));
        self.write16(0x670, center_x);
        let spread = self.read16(0x686).saturating_sub(1);
        let left = center_x.wrapping_sub(spread).min(255);
        let right = center_x.wrapping_add(spread).min(255);
        let window = left | right << 8;
        let mut lower = center_y.wrapping_mul(2);
        if lower < 0xe0 {
            lower = 0xe0;
        }
        let mut upper = center_y.wrapping_mul(2).wrapping_sub(lower);
        loop {
            if upper as i16 >= 0 {
                let value = if self.read16(0x674) as i16 >= 0 && upper < self.read16(0x674) {
                    0xff
                } else {
                    window
                };
                if upper < 240 {
                    self.write16(
                        HDMA_TABLE_DYNAMIC + usize::from(upper) * 2,
                        if value == 0xffff { 0xff } else { value },
                    );
                }
            }
            let value = if lower >= self.read16(0x676) {
                0xff
            } else {
                if lower >= 225 && self.read16(0x678) != 0 {
                    self.write16(0x678, self.read16(0x678) - 1);
                }
                window
            };
            if lower < 240 {
                self.write16(
                    HDMA_TABLE_DYNAMIC + usize::from(lower) * 2,
                    if value == 0xffff { 0xff } else { value },
                );
            }
            let done = center_y == upper;
            lower = lower.wrapping_sub(1);
            upper = upper.wrapping_add(1);
            if done {
                break;
            }
        }
    }

    pub fn flood_dam_prep_flood_hdma(&mut self) {
        self.write16(0x674, self.read16(0x682).wrapping_sub(self.read16(0xe8)));
        let center_x = self.read16(0x680).wrapping_sub(self.read16(0xe2));
        self.write16(0x670, center_x);
        let radius = self.read16(0x686) ^ 1;
        let mut line = 0usize;
        let upper = usize::from(self.read16(0x676));
        loop {
            self.write16(HDMA_TABLE_DYNAMIC + line * 2, 0xff00);
            line += 1;
            if line == upper {
                break;
            }
        }
        let spread = radius.wrapping_add(1);
        let window =
            center_x.wrapping_add(spread) << 8 | u16::from(center_x.wrapping_sub(spread) as u8);
        let end = self.read16(0x676).wrapping_add(self.read16(0x684)) ^ 1;
        loop {
            if line >= usize::from(end) {
                self.write16(HDMA_TABLE_DYNAMIC + line * 2, 0xff);
            } else {
                let mut mapped = (line as u16).wrapping_mul(2);
                while mapped >= 480 {
                    mapped = mapped.wrapping_mul(2);
                }
                self.write16(
                    HDMA_TABLE_DYNAMIC + usize::from(mapped >> 1) * 2,
                    if window == 0xffff { 0xff } else { window },
                );
            }
            line += 1;
            if line >= 225 {
                break;
            }
        }
    }

    pub fn graphics_incremental_vram_upload(&mut self) {
        let counter = self.read8(0x412);
        if counter == 16 {
            return;
        }
        self.write8(0x19, 0x50 + counter);
        self.write16(0x118, u16::from(counter) << 9);
        self.write8(0x412, counter + 1);
    }

    pub fn link_zap_handle_mosaic(&mut self) {
        let mut level = self.read8(MOSAIC_LEVEL);
        if self.read8(0x647) == 0 {
            level = level.wrapping_add(0x10);
            if level == 0xc0 {
                self.write8(0x647, 1);
            }
        } else {
            level = level.wrapping_sub(0x10);
            if level == 0 {
                self.write8(0x647, 0);
            }
        }
        self.write8(MOSAIC_LEVEL, level);
        self.write8(MOSAIC_COPY, level >> 1 | 3);
        self.write8(BGMODE_COPY, 9);
    }

    pub fn player_set_custom_mosaic_level(&mut self, value: u8) {
        self.write8(0x647, 0);
        self.write8(MOSAIC_LEVEL, value);
        self.write8(MOSAIC_COPY, value >> 1 | 3);
        self.write8(BGMODE_COPY, 9);
    }

    pub fn reset_star_tile_graphics(&mut self) {
        self.write8(0x4bc, 0);
        self.dungeon_restore_star_tile_chr();
    }

    pub fn dungeon_restore_star_tile_chr(&mut self) {
        let (first, second) = if self.read8(0x4bc) != 0 {
            (32, 0)
        } else {
            (0, 32)
        };
        self.wram
            .copy_within(0xbdc0 + first..0xbdc0 + first + 32, 0x10000);
        self.wram
            .copy_within(0xbdc0 + second..0xbdc0 + second + 32, 0x10020);
        self.write8(0x17, 0x18);
    }

    pub fn dungeon_update_peg_gfx_buffer(&mut self, first: usize, second: usize) {
        self.wram
            .copy_within(0xb340 + first..0xb340 + first + 128, 0x10000);
        self.wram
            .copy_within(0xb340 + second..0xb340 + second + 128, 0x10080);
        self.write8(0x17, 23);
    }

    pub fn recover_peg_gfx_from_mapping(&mut self) {
        if self.read8(0xc172) != 0 {
            self.dungeon_update_peg_gfx_buffer(0x180, 0);
        } else {
            self.dungeon_update_peg_gfx_buffer(0, 0x180);
        }
    }

    pub fn update_pegs_step_1(&mut self) {
        if self.read8(0xc172) != 0 {
            self.dungeon_update_peg_gfx_buffer(0x80, 0x100);
        } else {
            self.dungeon_update_peg_gfx_buffer(0x100, 0x80);
        }
    }

    pub fn update_pegs_step_2(&mut self) {
        if self.read8(0xc172) != 0 {
            self.dungeon_update_peg_gfx_buffer(0x100, 0x80);
        } else {
            self.dungeon_update_peg_gfx_buffer(0x80, 0x100);
        }
    }
}

/// Zelda 3's command-stream decompressor. Invalid streams intentionally panic,
/// matching the C port's unchecked source and destination accesses.
pub fn decompress(dst: &mut [u8], src: &[u8]) -> usize {
    let mut si = 0usize;
    let mut di = 0usize;
    loop {
        let mut command = src[si];
        si += 1;
        if command == 0xff {
            return di;
        }
        let len;
        if command & 0xe0 != 0xe0 {
            len = usize::from(command & 0x1f) + 1;
            command &= 0xe0;
        } else {
            len = usize::from(src[si]) + (usize::from(command & 3) << 8) + 1;
            si += 1;
            command = command << 3 & 0xe0;
        }
        match command {
            0x00 => {
                dst[di..di + len].copy_from_slice(&src[si..si + len]);
                si += len;
                di += len;
            }
            0x20 => {
                let value = src[si];
                si += 1;
                dst[di..di + len].fill(value);
                di += len;
            }
            0x40 => {
                let lo = src[si];
                let hi = src[si + 1];
                si += 2;
                for n in 0..len {
                    dst[di + n] = if n & 1 == 0 { lo } else { hi };
                }
                di += len;
            }
            0x60 => {
                let mut value = src[si];
                si += 1;
                for out in &mut dst[di..di + len] {
                    *out = value;
                    value = value.wrapping_add(1);
                }
                di += len;
            }
            _ => {
                let mut offset = usize::from(src[si]) | usize::from(src[si + 1]) << 8;
                si += 2;
                for _ in 0..len {
                    dst[di] = dst[offset];
                    di += 1;
                    offset += 1;
                }
            }
        }
    }
}

pub fn do_3_to_4_high_16(dst: &mut [u8], src: &[u8], count: usize) {
    convert_3_to_4(dst, src, count, true, false);
}

pub fn do_3_to_4_low_16(dst: &mut [u8], src: &[u8], count: usize) {
    convert_3_to_4(dst, src, count, false, false);
}

pub fn do_3_to_4_high(vram: &mut [u16], src: &[u8]) {
    let mut converted = [0u8; 0x800];
    convert_3_to_4(&mut converted, src, 64, true, true);
    for (destination, pair) in vram.iter_mut().zip(converted.chunks_exact(2)) {
        *destination = u16::from_le_bytes([pair[0], pair[1]]);
    }
}

pub fn do_3_to_4_low(vram: &mut [u16], src: &[u8]) {
    let mut converted = [0u8; 0x800];
    convert_3_to_4(&mut converted, src, 64, false, true);
    for (destination, pair) in vram.iter_mut().zip(converted.chunks_exact(2)) {
        *destination = u16::from_le_bytes([pair[0], pair[1]]);
    }
}

pub fn expand_3_to_4_high(dst: &mut [u8], src: &[u8], base_offset: usize, count: usize) {
    let mut source = 0usize;
    let mut output = 0usize;
    for _ in 0..count {
        for row in 0..8 {
            let p0 = src[source + row * 2];
            let p1 = src[source + row * 2 + 1];
            let p2 = src[source + 0x10 + row];
            dst[output + row * 2] = p0;
            dst[output + row * 2 + 1] = p1;
            dst[output + 0x10 + row * 2] = p2;
            dst[output + 0x10 + row * 2 + 1] = p0 | p1 | p2;
        }
        output += 0x20;
        source += 0x18;
        if (base_offset + source) & 0x78 == 0 {
            source += 0x180;
        }
    }
}

fn convert_3_to_4(dst: &mut [u8], src: &[u8], count: usize, high: bool, _vram: bool) {
    for tile in 0..count {
        let input = tile * 24;
        let output = tile * 32;
        for row in 0..8 {
            let p0 = src[input + row * 2];
            let p1 = src[input + row * 2 + 1];
            let p2 = src[input + 16 + row];
            dst[output + row * 2] = p0;
            dst[output + row * 2 + 1] = p1;
            dst[output + 16 + row * 2] = p2;
            dst[output + 16 + row * 2 + 1] = if high { p0 | p1 | p2 } else { 0 };
        }
    }
}

fn leave_default_graphics_scratch(wram: &mut [u8], source: &[u8]) {
    let final_tile = &source[63 * 24..63 * 24 + 16];
    for (pair_index, pair) in final_tile.chunks_exact(2).enumerate() {
        let address = 0xbf + (7 - pair_index) * 2;
        wram[address] = pair[0] | pair[1];
        wram[address + 1] = 0;
    }
}

fn leave_high_conversion_scratch(wram: &mut [u8], source: &[u8]) {
    let final_tile = &source[63 * 24..63 * 24 + 16];
    for (pair_index, pair) in final_tile.chunks_exact(2).enumerate() {
        let address = 0xbf + (7 - pair_index) * 2;
        wram[address] = pair[0] | pair[1];
        wram[address + 1] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompressor_covers_all_five_commands_and_overlap() {
        let stream = [
            0x02, 1, 2, 3, // literal
            0x22, 0xaa, // byte fill
            0x43, 0x10, 0x20, // alternating fill
            0x62, 0xfe, // incrementing fill
            0x83, 0, 0, // overlapping dictionary copy
            0xff,
        ];
        let mut out = [0u8; 32];
        let n = decompress(&mut out, &stream);
        assert_eq!(n, 17);
        assert_eq!(
            &out[..17],
            &[
                1, 2, 3, 0xaa, 0xaa, 0xaa, 0x10, 0x20, 0x10, 0x20, 0xfe, 0xff, 0, 1, 2, 3, 0xaa
            ]
        );
    }

    #[test]
    fn decompressor_extended_length_and_divide_by_zero_match_snes() {
        let mut stream = vec![0xe0, 0x20];
        stream.extend(core::iter::repeat_n(0x5a, 33));
        stream.push(0xff);
        let mut out = [0u8; 64];
        assert_eq!(decompress(&mut out, &stream), 33);
        assert_eq!(&out[..33], &[0x5a; 33]);
        assert_eq!(
            LoadGfx::<FixtureAssets, NoHooks>::snes_divide(123, 0),
            0xffff
        );
    }

    #[test]
    fn planar_conversion_preserves_planes_and_synthesizes_high_plane() {
        let mut src = [0u8; 24];
        src[0] = 0x81;
        src[1] = 0x24;
        src[16] = 0x18;
        let mut high = [0u8; 32];
        let mut low = [0u8; 32];
        do_3_to_4_high_16(&mut high, &src, 1);
        do_3_to_4_low_16(&mut low, &src, 1);
        assert_eq!(&high[..2], &[0x81, 0x24]);
        assert_eq!(&high[16..18], &[0x18, 0xbd]);
        assert_eq!(&low[16..18], &[0x18, 0]);
    }

    #[test]
    fn default_graphics_preserves_the_c_final_tile_scratch_residue() {
        let mut source = [0u8; 64 * 24];
        for (index, byte) in source[63 * 24..63 * 24 + 16].iter_mut().enumerate() {
            *byte = index as u8;
        }
        let mut wram = [0xa5; 0xcf];
        leave_default_graphics_scratch(&mut wram, &source);
        assert_eq!(
            &wram[0xbf..0xcf],
            &[
                0x0f, 0, 0x0d, 0, 0x0b, 0, 0x09, 0, 0x07, 0, 0x05, 0, 0x03, 0, 0x01, 0
            ]
        );
    }

    #[test]
    fn palette_restore_and_translucency_swap_use_exact_wram_buffers() {
        let mut wram = Box::new([0u8; z3_core::WRAM_SIZE]);
        let mut vram = vec![0u16; 0x8000];
        let assets = FixtureAssets;
        let mut hooks = NoHooks;
        wram[AUX_PALETTE + 2..AUX_PALETTE + 4].copy_from_slice(&0x4210u16.to_le_bytes());
        wram[MAIN_PALETTE + 2..MAIN_PALETTE + 4].copy_from_slice(&0x0000u16.to_le_bytes());
        let mut gfx = LoadGfx {
            wram: &mut wram,
            vram: &mut vram,
            assets: &assets,
            hooks: &mut hooks,
        };
        gfx.palette_filter_restore_additive(2, 4);
        assert_eq!(gfx.palette(MAIN_PALETTE, 1), 0x0421);
        gfx.set_palette(AUX_PALETTE, 0x80, 0x1111);
        gfx.set_palette(AUX_PALETTE, 0xf0, 0x2222);
        gfx.palette_assert_translucency_swap();
        assert_eq!(gfx.palette(MAIN_PALETTE, 0x80), 0x2222);
        assert_eq!(gfx.palette(MAIN_PALETTE, 0xf0), 0x1111);
        assert_eq!(gfx.read8(PALETTE_SWAP_FLAG), 1);
    }

    #[test]
    fn tileset_selection_preserves_zero_as_inherit_semantics() {
        let mut wram = Box::new([0u8; z3_core::WRAM_SIZE]);
        let mut vram = vec![0u16; 0x8000];
        let assets = FixtureAssets;
        let mut hooks = NoHooks;
        wram[0xaa1] = 0;
        wram[0xaa2] = 0;
        wram[0xaa3] = 1;
        let mut gfx = LoadGfx {
            wram: &mut wram,
            vram: &mut vram,
            assets: &assets,
            hooks: &mut hooks,
        };
        gfx.animate_mirror_warp_decompress_new_tile_sets();
        assert_eq!(&gfx.wram[0xc2f8..0xc2fc], &[6, 14, 31, 24]);
        assert_eq!(&gfx.wram[0xc2fc..0xc300], &[70, 73, 12, 29]);
    }

    #[test]
    fn trinexx_flash_uses_four_frame_cadence_and_saturates_component() {
        let mut wram = Box::new([0u8; z3_core::WRAM_SIZE]);
        let mut vram = vec![0u16; 0x8000];
        let assets = FixtureAssets;
        let mut hooks = NoHooks;
        wram[MAIN_PALETTE + 0x41 * 2..MAIN_PALETTE + 0x41 * 2 + 2]
            .copy_from_slice(&0x001eu16.to_le_bytes());
        let mut gfx = LoadGfx {
            wram: &mut wram,
            vram: &mut vram,
            assets: &assets,
            hooks: &mut hooks,
        };
        gfx.trinexx_flash_shell_palette_red();
        assert_eq!(gfx.palette(MAIN_PALETTE, 0x41), 0x001f);
        assert_eq!(gfx.read8(0x4be), 2);
        gfx.trinexx_flash_shell_palette_red();
        assert_eq!(gfx.palette(MAIN_PALETTE, 0x41), 0x001f);
        assert_eq!(gfx.read8(0x4be), 1);
    }

    #[test]
    fn star_and_peg_uploads_preserve_messaging_buffer_layout() {
        let mut wram = Box::new([0u8; z3_core::WRAM_SIZE]);
        let mut vram = vec![0u16; 0x8000];
        let assets = FixtureAssets;
        let mut hooks = NoHooks;
        for i in 0..0x300 {
            wram[0xb340 + i] = i as u8;
        }
        for i in 0..64 {
            wram[0xbdc0 + i] = (0x80 + i) as u8;
        }
        let mut gfx = LoadGfx {
            wram: &mut wram,
            vram: &mut vram,
            assets: &assets,
            hooks: &mut hooks,
        };
        gfx.reset_star_tile_graphics();
        assert_eq!(&gfx.wram[0x10000..0x10020], &gfx.wram[0xbdc0..0xbde0]);
        assert_eq!(&gfx.wram[0x10020..0x10040], &gfx.wram[0xbde0..0xbe00]);
        assert_eq!(gfx.read8(0x17), 0x18);
        gfx.dungeon_update_peg_gfx_buffer(0x80, 0x100);
        assert_eq!(&gfx.wram[0x10000..0x10080], &gfx.wram[0xb3c0..0xb440]);
        assert_eq!(&gfx.wram[0x10080..0x10100], &gfx.wram[0xb440..0xb4c0]);
        assert_eq!(gfx.read8(0x17), 23);
    }

    #[test]
    fn incremental_upload_uses_exact_nmi_addresses_and_stops_at_sixteen() {
        let mut wram = Box::new([0u8; z3_core::WRAM_SIZE]);
        let mut vram = vec![0u16; 0x8000];
        let assets = FixtureAssets;
        let mut hooks = NoHooks;
        let mut gfx = LoadGfx {
            wram: &mut wram,
            vram: &mut vram,
            assets: &assets,
            hooks: &mut hooks,
        };
        gfx.write8(0x412, 15);
        gfx.graphics_incremental_vram_upload();
        assert_eq!(gfx.read8(0x19), 0x5f);
        assert_eq!(gfx.read16(0x118), 0x1e00);
        assert_eq!(gfx.read8(0x412), 16);
        gfx.graphics_incremental_vram_upload();
        assert_eq!(gfx.read8(0x412), 16);
    }

    #[test]
    fn asset_palette_loaders_honor_c_inclusive_counts_and_aux_offset() {
        let mut wram = Box::new([0xffu8; z3_core::WRAM_SIZE]);
        let mut vram = vec![0u16; 0x8000];
        let assets = ZeroPaletteAssets;
        let mut hooks = NoHooks;
        wram[0xaa8..0xaaa].copy_from_slice(&0x20u16.to_le_bytes());
        wram[0xab3] = 0;
        let mut gfx = LoadGfx {
            wram: &mut wram,
            vram: &mut vram,
            assets: &assets,
            hooks: &mut hooks,
        };
        gfx.palette_load_owbg_main();
        for row in 0..5 {
            let first = 0x31 + row * 0x10;
            assert_eq!(
                &gfx.wram[AUX_PALETTE + first * 2..AUX_PALETTE + (first + 7) * 2],
                &[0; 14]
            );
            assert_eq!(gfx.palette(AUX_PALETTE, first + 7), 0xffff);
        }
        assert_eq!(gfx.palette(MAIN_PALETTE, 0x31), 0xffff);
    }

    struct FixtureAssets;
    impl GfxAssets for FixtureAssets {
        fn sprite(&self, _: usize) -> &[u8] {
            &[]
        }
        fn background(&self, _: usize) -> &[u8] {
            &[]
        }
        fn raw(&self, _: usize) -> &[u8] {
            &[]
        }
        fn dialogue_font(&self) -> &[u8] {
            &[]
        }
    }
    struct NoHooks;
    impl LoadGfxHooks for NoHooks {}

    static ZERO_ASSET: [u8; 4096] = [0; 4096];
    struct ZeroPaletteAssets;
    impl GfxAssets for ZeroPaletteAssets {
        fn sprite(&self, _: usize) -> &[u8] {
            &[]
        }
        fn background(&self, _: usize) -> &[u8] {
            &[]
        }
        fn raw(&self, _: usize) -> &[u8] {
            &ZERO_ASSET
        }
        fn dialogue_font(&self) -> &[u8] {
            &[]
        }
    }
}
