//! Graphics decompression, tileset loading, palette filtering, and HDMA-window logic.
//!
//! This is a direct, bounds-checked Rust port of `src/load_gfx.c`. WRAM offsets and
//! VRAM word addresses use the same values as the C implementation.

mod conversion;
mod cpu;
mod upload;

pub use cpu::GraphicsCpu;
pub use upload::GraphicsUpload;

use conversion::{convert_words, expand_pair_from_wram, high_plane_scratch, planar_wram};

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

/// Compatibility facade retaining the translated C entrypoints. Native
/// composition uses [`GraphicsCpu`] or [`GraphicsUpload`] directly so callback
/// execution and VRAM mutation never borrow the same service owner.
pub struct LoadGfx<'a, A: GfxAssets, H: LoadGfxHooks> {
    pub wram: &'a mut [u8; z3_core::WRAM_SIZE],
    pub vram: &'a mut [u16],
    pub assets: &'a A,
    pub hooks: &'a mut H,
}

impl<A: GfxAssets, H: LoadGfxHooks> LoadGfx<'_, A, H> {
    fn cpu(&mut self) -> GraphicsCpu<'_, A, H> {
        GraphicsCpu {
            wram: self.wram,
            assets: self.assets,
            hooks: self.hooks,
        }
    }

    fn upload(&mut self) -> GraphicsUpload<'_, A> {
        GraphicsUpload {
            wram: self.wram,
            vram: self.vram,
            assets: self.assets,
        }
    }

    #[cfg(test)]
    fn read8(&self, address: usize) -> u8 {
        self.wram[address]
    }

    #[cfg(test)]
    fn write8(&mut self, address: usize, value: u8) {
        self.wram[address] = value;
    }

    fn read16(&self, address: usize) -> u16 {
        u16::from_le_bytes([self.wram[address], self.wram[address + 1]])
    }

    #[cfg(test)]
    fn palette(&self, base: usize, index: usize) -> u16 {
        self.read16(base + index * 2)
    }

    #[cfg(test)]
    fn set_palette(&mut self, base: usize, index: usize, value: u16) {
        self.wram[base + index * 2..base + index * 2 + 2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn snes_divide(dividend: u16, divisor: u8) -> u16 {
        if divisor == 0 {
            0xffff
        } else {
            dividend / u16::from(divisor)
        }
    }

    pub fn erase_tile_maps(&mut self, high: u16, low: u16) {
        self.upload().erase_tile_maps(high, low)
    }

    pub fn erase_tile_maps_normal(&mut self) {
        self.upload().erase_tile_maps_normal()
    }

    pub fn erase_tile_maps_triforce(&mut self) {
        self.upload().erase_tile_maps_triforce()
    }

    pub fn erase_tile_maps_dungeonmap(&mut self) {
        self.upload().erase_tile_maps_dungeonmap()
    }

    pub fn enable_force_blank(&mut self) {
        self.cpu().enable_force_blank()
    }

    pub fn load_overworld_map_palette(&mut self) {
        self.cpu().load_overworld_map_palette()
    }

    pub fn load_item_animation_gfx_one(
        &mut self,
        destination: usize,
        count: usize,
        table_index: usize,
        from_temp: bool,
    ) -> usize {
        self.cpu()
            .load_item_animation_gfx_one(destination, count, table_index, from_temp)
    }

    pub fn load_item_gfx_into_wram_4bpp_buffer(&mut self) {
        self.cpu().load_item_gfx_into_wram_4bpp_buffer()
    }

    pub fn decompress_sword_graphics(&mut self) {
        self.cpu().decompress_sword_graphics()
    }

    pub fn decompress_shield_graphics(&mut self) {
        self.cpu().decompress_shield_graphics()
    }

    pub fn decompress_animated_dungeon_tiles(&mut self, pack: u8) {
        self.cpu().decompress_animated_dungeon_tiles(pack)
    }

    pub fn decompress_animated_overworld_tiles(&mut self, pack: u8) {
        self.cpu().decompress_animated_overworld_tiles(pack)
    }

    pub fn load_item_gfx_auxiliary(&mut self) {
        self.cpu().load_item_gfx_auxiliary()
    }

    pub fn load_follower_graphics(&mut self) {
        self.cpu().load_follower_graphics()
    }

    pub fn write_to_4bpp_buffer_at_7f4000(&mut self, index: u8) {
        self.cpu().write_to_4bpp_buffer_at_7f4000(index)
    }

    pub fn decode_animated_sprite_tile_variable(&mut self, index: u8) {
        self.cpu().decode_animated_sprite_tile_variable(index)
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

    pub fn decomp_spr_to_wram(&mut self, offset: usize, gfx: usize) -> usize {
        self.cpu().decomp_spr_to_wram(offset, gfx)
    }

    pub fn decomp_bg_to_wram(&mut self, offset: usize, gfx: usize) -> usize {
        self.cpu().decomp_bg_to_wram(offset, gfx)
    }

    pub fn animate_mirror_warp_decompress_new_tile_sets(&mut self) {
        self.cpu().animate_mirror_warp_decompress_new_tile_sets()
    }

    pub fn gfx_load_sprites_inner(&mut self, destination: usize) {
        self.cpu().gfx_load_sprites_inner(destination)
    }

    pub fn load_trans_aux_gfx(&mut self) {
        self.cpu().load_trans_aux_gfx()
    }

    pub fn load_trans_aux_gfx_sprite(&mut self) {
        self.cpu().load_trans_aux_gfx_sprite()
    }

    pub fn reload_previously_loaded_sheets(&mut self) {
        self.cpu().reload_previously_loaded_sheets()
    }

    pub fn attract_decompress_story_gfx(&mut self) {
        self.cpu().attract_decompress_story_gfx()
    }

    pub fn animate_mirror_warp(&mut self) {
        self.cpu().animate_mirror_warp()
    }

    pub fn load_sprite_graphics(&mut self, vram_offset: usize, pack: usize, temp: usize) {
        self.upload().load_sprite_graphics(vram_offset, pack, temp)
    }

    pub fn load_background_graphics(
        &mut self,
        vram_offset: usize,
        pack: usize,
        slot: usize,
        temp: usize,
    ) {
        self.upload()
            .load_background_graphics(vram_offset, pack, slot, temp)
    }

    pub fn load_common_sprites(&mut self) {
        self.upload().load_common_sprites()
    }

    pub fn initialize_tilesets(&mut self) {
        self.upload().initialize_tilesets()
    }

    pub fn load_default_graphics(&mut self) {
        self.upload().load_default_graphics()
    }

    pub fn attract_load_bg3_gfx(&mut self) {
        self.upload().attract_load_bg3_gfx()
    }

    pub fn transfer_font_to_vram(&mut self) {
        self.upload().transfer_font_to_vram()
    }

    pub fn load_file_select_graphics(&mut self) {
        self.upload().load_file_select_graphics()
    }

    pub fn graphics_load_chr_half_slot(&mut self) {
        self.cpu().graphics_load_chr_half_slot()
    }

    pub fn prep_trans_aux_gfx(&mut self) {
        self.cpu().prep_trans_aux_gfx()
    }

    pub fn load_new_sprite_gfx_set(&mut self) {
        self.cpu().load_new_sprite_gfx_set()
    }

    pub fn apply_palette_filter_bounce(&mut self) {
        self.cpu().apply_palette_filter_bounce()
    }

    pub fn palette_filter_range(&mut self, from: usize, to: usize) {
        self.cpu().palette_filter_range(from, to)
    }

    pub fn palette_filter_incr_countdown(&mut self) {
        self.cpu().palette_filter_incr_countdown()
    }

    pub fn palette_filter_restore_additive(&mut self, from: usize, to: usize) {
        self.cpu().palette_filter_restore_additive(from, to)
    }

    pub fn palette_filter_restore_subtractive(&mut self, from: usize, to: usize) {
        self.cpu().palette_filter_restore_subtractive(from, to)
    }

    pub fn reset_hud_palettes_4_and_5(&mut self) {
        self.cpu().reset_hud_palettes_4_and_5()
    }

    pub fn palette_filter_history(&mut self) {
        self.cpu().palette_filter_history()
    }

    pub fn palette_filter_wish_ponds(&mut self) {
        self.cpu().palette_filter_wish_ponds()
    }

    pub fn palette_filter_crystal(&mut self) {
        self.cpu().palette_filter_crystal()
    }

    pub fn palette_filter_wish_ponds_inner(&mut self) {
        self.cpu().palette_filter_wish_ponds_inner()
    }

    pub fn palette_filter_restore_sp5f(&mut self) {
        self.cpu().palette_filter_restore_sp5f()
    }

    pub fn palette_filter_sp5f(&mut self) {
        self.cpu().palette_filter_sp5f()
    }

    pub fn kholdstare_shell_palette_filtering(&mut self) {
        self.cpu().kholdstare_shell_palette_filtering()
    }

    pub fn agahnim_warp_shadow_filter(&mut self, index: usize, settings_addr: usize) {
        self.cpu().agahnim_warp_shadow_filter(index, settings_addr)
    }

    pub fn palette_fade_intro_one_step(&mut self) {
        self.cpu().palette_fade_intro_one_step()
    }

    pub fn palette_fade_intro_2(&mut self) {
        self.cpu().palette_fade_intro_2()
    }

    pub fn palette_filter_initialize_white_filter(&mut self) {
        self.cpu().palette_filter_initialize_white_filter()
    }

    pub fn mirror_warp_run_animation_submodules(&mut self) {
        self.cpu().mirror_warp_run_animation_submodules()
    }

    pub fn palette_filter_blinding_white(&mut self) {
        self.cpu().palette_filter_blinding_white()
    }

    pub fn palette_filter_start_blinding_white(&mut self) {
        self.cpu().palette_filter_start_blinding_white()
    }

    pub fn palette_filter_blinding_white_triforce(&mut self) {
        self.cpu().palette_filter_blinding_white_triforce()
    }

    pub fn trinexx_flash_shell_palette_red(&mut self) {
        self.cpu().trinexx_flash_shell_palette_red()
    }

    pub fn trinexx_unflash_shell_palette_red(&mut self) {
        self.cpu().trinexx_unflash_shell_palette_red()
    }

    pub fn trinexx_flash_shell_palette_blue(&mut self) {
        self.cpu().trinexx_flash_shell_palette_blue()
    }

    pub fn trinexx_unflash_shell_palette_blue(&mut self) {
        self.cpu().trinexx_unflash_shell_palette_blue()
    }

    pub fn palette_filter_restore_bg_subtractive_strict(&mut self) {
        self.cpu().palette_filter_restore_bg_subtractive_strict()
    }

    pub fn palette_filter_restore_bg_additive_strict(&mut self) {
        self.cpu().palette_filter_restore_bg_additive_strict()
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
        self.cpu().filter_majorly_whiten_bg()
    }

    pub fn palette_set_bg_and_fixed_color(&mut self, color: u16) {
        self.cpu().palette_set_bg_and_fixed_color(color)
    }

    pub fn palette_bg_and_fixed_color_black(&mut self) {
        self.cpu().palette_bg_and_fixed_color_black()
    }

    pub fn set_backdrop_color_black(&mut self) {
        self.cpu().set_backdrop_color_black()
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
        self.cpu().palette_set_ow_bg_color()
    }

    pub fn palette_special_ow(&mut self) {
        self.cpu().palette_special_ow()
    }

    pub fn palette_set_translucency_swap(&mut self, value: bool) {
        self.cpu().palette_set_translucency_swap(value)
    }

    pub fn palette_assert_translucency_swap(&mut self) {
        self.cpu().palette_assert_translucency_swap()
    }

    pub fn palette_revert_translucency_swap(&mut self) {
        self.cpu().palette_revert_translucency_swap()
    }

    pub fn overworld_copy_palettes_to_cache(&mut self) {
        self.cpu().overworld_copy_palettes_to_cache()
    }

    pub fn special_overworld_copy_palettes_to_cache(&mut self) {
        self.cpu().special_overworld_copy_palettes_to_cache()
    }

    pub fn palette_restore_coldata(&mut self) {
        self.cpu().palette_restore_coldata()
    }

    pub fn palette_restore_bg_from_flash(&mut self) {
        self.cpu().palette_restore_bg_from_flash()
    }

    pub fn palette_restore_bg_and_hud(&mut self) {
        self.cpu().palette_restore_bg_and_hud()
    }

    pub fn palette_load_single(&mut self, source: &[u16], destination: usize, entries: usize) {
        self.cpu().palette_load_single(source, destination, entries)
    }

    pub fn palette_load_multiple(
        &mut self,
        source: &[u16],
        destination: usize,
        entries: usize,
        palettes: usize,
    ) {
        self.cpu()
            .palette_load_multiple(source, destination, entries, palettes)
    }

    pub fn palette_load_multiple_arbitrary(
        &mut self,
        source: &[u16],
        destination: usize,
        entries: usize,
    ) {
        self.cpu()
            .palette_load_multiple_arbitrary(source, destination, entries)
    }

    pub fn load_gear_palette(&mut self, destination: usize, source: &[u16], entries: usize) {
        self.cpu().load_gear_palette(destination, source, entries)
    }

    pub fn palette_load_sp0l(&mut self) {
        self.cpu().palette_load_sp0l()
    }

    pub fn palette_load_sprite_main(&mut self) {
        self.cpu().palette_load_sprite_main()
    }

    pub fn palette_load_sp5l(&mut self) {
        self.cpu().palette_load_sp5l()
    }

    pub fn palette_load_sp6l(&mut self) {
        self.cpu().palette_load_sp6l()
    }

    pub fn palette_load_sword(&mut self) {
        self.cpu().palette_load_sword()
    }

    pub fn palette_load_shield(&mut self) {
        self.cpu().palette_load_shield()
    }

    pub fn palette_load_sprite_environment(&mut self) {
        self.cpu().palette_load_sprite_environment()
    }

    pub fn palette_load_sprite_environment_dungeon(&mut self) {
        self.cpu().palette_load_sprite_environment_dungeon()
    }

    pub fn palette_misc_sprite_outdoors(&mut self) {
        self.cpu().palette_misc_sprite_outdoors()
    }

    pub fn palette_load_dungeon_map_sprite(&mut self) {
        self.cpu().palette_load_dungeon_map_sprite()
    }

    pub fn palette_load_link_armor_and_gloves(&mut self) {
        self.cpu().palette_load_link_armor_and_gloves()
    }

    pub fn palette_update_gloves_color(&mut self) {
        self.cpu().palette_update_gloves_color()
    }

    pub fn palette_load_dungeon_map_bg(&mut self) {
        self.cpu().palette_load_dungeon_map_bg()
    }

    pub fn palette_load_hud(&mut self) {
        self.cpu().palette_load_hud()
    }

    pub fn palette_load_dungeon_set(&mut self) {
        self.cpu().palette_load_dungeon_set()
    }

    pub fn palette_load_owbg3(&mut self) {
        self.cpu().palette_load_owbg3()
    }

    pub fn palette_load_owbg_main(&mut self) {
        self.cpu().palette_load_owbg_main()
    }

    pub fn palette_load_owbg1(&mut self) {
        self.cpu().palette_load_owbg1()
    }

    pub fn palette_load_owbg2(&mut self) {
        self.cpu().palette_load_owbg2()
    }

    pub fn load_gear_palettes(&mut self, sword: u8, shield: u8, armor: u8) {
        self.cpu().load_gear_palettes(sword, shield, armor)
    }

    pub fn load_actual_gear_palettes(&mut self) {
        self.cpu().load_actual_gear_palettes()
    }

    pub fn palette_electro_themed_gear(&mut self) {
        self.cpu().palette_electro_themed_gear()
    }

    pub fn load_gear_palettes_bunny(&mut self) {
        self.cpu().load_gear_palettes_bunny()
    }

    pub fn palette_load_agahnim(&mut self) {
        self.cpu().palette_load_agahnim()
    }

    pub fn palette_load_for_file_select(&mut self) {
        self.cpu().palette_load_for_file_select()
    }

    pub fn palette_load_for_file_select_armor(&mut self, offset: usize, armor: u8, gloves: u8) {
        self.cpu()
            .palette_load_for_file_select_armor(offset, armor, gloves)
    }

    pub fn palette_load_for_file_select_sword(&mut self, offset: usize, sword: u8) {
        self.cpu().palette_load_for_file_select_sword(offset, sword)
    }

    pub fn palette_load_for_file_select_shield(&mut self, offset: usize, shield: u8) {
        self.cpu()
            .palette_load_for_file_select_shield(offset, shield)
    }

    pub fn overworld_load_palettes(&mut self, bg: u8, sprites: u8) {
        self.cpu().overworld_load_palettes(bg, sprites)
    }

    pub fn overworld_load_all_palettes(&mut self) {
        self.cpu().overworld_load_all_palettes()
    }

    pub fn dungeon_load_palettes(&mut self) {
        self.cpu().dungeon_load_palettes()
    }

    pub fn overworld_load_palettes_inner(&mut self) {
        self.cpu().overworld_load_palettes_inner()
    }

    pub fn overworld_load_screens_palette_set(&mut self) {
        self.cpu().overworld_load_screens_palette_set()
    }

    pub fn overworld_load_area_palettes_ex(&mut self, index: u8) {
        self.cpu().overworld_load_area_palettes_ex(index)
    }

    pub fn dungeon_handle_translucency_and_palette(&mut self) {
        self.cpu().dungeon_handle_translucency_and_palette()
    }

    pub fn handle_screen_flash(&mut self) {
        self.cpu().handle_screen_flash()
    }

    pub fn palette_filter_whirlpool_blue(&mut self) {
        self.cpu().palette_filter_whirlpool_blue()
    }

    pub fn palette_filter_isolate_whirlpool_blue(&mut self) {
        self.cpu().palette_filter_isolate_whirlpool_blue()
    }

    pub fn palette_filter_whirlpool_restore_blue(&mut self) {
        self.cpu().palette_filter_whirlpool_restore_blue()
    }

    pub fn palette_filter_whirlpool_restore_red_green(&mut self) {
        self.cpu().palette_filter_whirlpool_restore_red_green()
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
        self.cpu().iris_spotlight_reset_table()
    }

    pub fn iris_spotlight_close(&mut self) {
        self.cpu().iris_spotlight_close()
    }

    pub fn spotlight_open(&mut self) {
        self.cpu().spotlight_open()
    }

    pub fn spotlight_internal(&mut self, radius: u8, mode: u8) {
        self.cpu().spotlight_internal(radius, mode)
    }

    pub fn iris_spotlight_configure_table(&mut self) {
        self.cpu().iris_spotlight_configure_table()
    }

    pub fn adjust_water_hdma_window(&mut self) {
        self.cpu().adjust_water_hdma_window()
    }

    pub fn adjust_water_hdma_window_x(&mut self, center_y: u16) {
        self.cpu().adjust_water_hdma_window_x(center_y)
    }

    pub fn flood_dam_prep_flood_hdma(&mut self) {
        self.cpu().flood_dam_prep_flood_hdma()
    }

    pub fn graphics_incremental_vram_upload(&mut self) {
        self.cpu().graphics_incremental_vram_upload()
    }

    pub fn link_zap_handle_mosaic(&mut self) {
        self.cpu().link_zap_handle_mosaic()
    }

    pub fn player_set_custom_mosaic_level(&mut self, value: u8) {
        self.cpu().player_set_custom_mosaic_level(value)
    }

    pub fn reset_star_tile_graphics(&mut self) {
        self.cpu().reset_star_tile_graphics()
    }

    pub fn dungeon_restore_star_tile_chr(&mut self) {
        self.cpu().dungeon_restore_star_tile_chr()
    }

    pub fn dungeon_update_peg_gfx_buffer(&mut self, first: usize, second: usize) {
        self.cpu().dungeon_update_peg_gfx_buffer(first, second)
    }

    pub fn recover_peg_gfx_from_mapping(&mut self) {
        self.cpu().recover_peg_gfx_from_mapping()
    }

    pub fn update_pegs_step_1(&mut self) {
        self.cpu().update_pegs_step_1()
    }

    pub fn update_pegs_step_2(&mut self) {
        self.cpu().update_pegs_step_2()
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

fn decode_sprite<A: GfxAssets>(destination: &mut [u8], assets: &A, gfx: usize) -> usize {
    let gfx = gfx.max(12);
    let source = assets.sprite(gfx);
    if gfx >= 103 || source.len() != 0x600 {
        decompress(destination, source)
    } else {
        destination[..0x600].copy_from_slice(source);
        0x600
    }
}

/// Hook-free implementation of C's `DecompAndUpload2bpp`.
fn decomp_and_upload_2bpp<A: GfxAssets>(
    wram: &mut [u8; z3_core::WRAM_SIZE],
    vram: &mut [u16],
    assets: &A,
    offset: usize,
    pack: usize,
) {
    decode_sprite(&mut wram[0x14000..], assets, pack);
    for (target, bytes) in vram[offset..offset + 1024]
        .iter_mut()
        .zip(wram[0x14000..0x14800].chunks_exact(2))
    {
        *target = u16::from_le_bytes([bytes[0], bytes[1]]);
    }
}

pub fn do_3_to_4_high_16(dst: &mut [u8], src: &[u8], count: usize) {
    convert_3_to_4(dst, src, count, true, false);
}

pub fn do_3_to_4_low_16(dst: &mut [u8], src: &[u8], count: usize) {
    convert_3_to_4(dst, src, count, false, false);
}

pub fn do_3_to_4_high(vram: &mut [u16], src: &[u8]) {
    convert_words(vram, src, 64, true);
}

pub fn do_3_to_4_low(vram: &mut [u16], src: &[u8]) {
    convert_words(vram, src, 64, false);
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
    wram[0xbf..0xcf].copy_from_slice(&high_plane_scratch(source));
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
