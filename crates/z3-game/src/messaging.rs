//! Messaging, maps, interface, and game-over state machines from `src/messaging.c`.
//!
//! This module retains the original overlapping WRAM layout.  The text decoder
//! is intentionally language-sensitive: the US and PAL data encode commands in
//! different byte ranges.

use crate::ram::Ram;

const MAIN_MODULE: usize = 0x10;
const SUBMODULE: usize = 0x11;
const NMI_LOAD_BG: usize = 0x14;
const NMI_SUBROUTINE: usize = 0x17;
const LINK_Y: usize = 0x20;
const BG2_VOFS2: usize = 0xe8;
const SAVED_MODULE: usize = 0x10c;
const SOUND1: usize = 0x12e;
const SOUND2: usize = 0x12f;
const INIDISP: usize = 0x13;
const HUD_UPDATE: usize = 0x16;
const TM_COPY: usize = 0x1c;
const BUTTON_MASK_BY: usize = 0x3a;
const BGMODE: usize = 0x94;
const MOSAIC: usize = 0x95;
const HDMAEN: usize = 0x9b;
const SUBSUBMODULE: usize = 0xb0;
const DUNG_DRAW_WIDTH: usize = 0xb2;
const SOME_MENU_CTR: usize = 0xc8;
const BG1_HOFS2: usize = 0xe0;
const BG1_VOFS2: usize = 0xe6;
const JOYPAD_H: usize = 0xf0;
const JOYPAD_L: usize = 0xf2;
const NMI_DISABLE_CORE: usize = 0x710;
const OW_MAP_STATE: usize = 0x200;
const IS_HEART_ANIM: usize = 0x20a;
const OW_MAP_FLAGS: usize = 0x636;
const MODE7_ZOOM_TIMER: usize = 0x637;
const M7Y: usize = 0x63a;
const VWF_FLAG_NEXT_LINE: usize = 0x720;
const VWF_CUR_LINE: usize = 0x722;
const VWF_VAR1: usize = 0x724;
const VWF_LINE_PTR: usize = 0x726;
const VRAM_UPLOAD_DATA: usize = 0x1002;
const VWF_TILEMAP: usize = 0x1300;
const TEXT_TOPLEFT_COPY: usize = 0x1cd0;
const TEXT_TOPLEFT: usize = 0x1cd2;
const TEXT_RENDER_STATE: usize = 0x1cd4;
const VWF_LINE_SPEED_CUR: usize = 0x1cd5;
const VWF_LINE_SPEED: usize = 0x1cd6;
const TEXT_INCREMENTAL_STATE: usize = 0x1cd7;
const MESSAGING_MODULE: usize = 0x1cd8;
const DIALOGUE_READ_POS: usize = 0x1cd9;
const TEXT_WAIT_COUNTDOWN: usize = 0x1ce0;
const TEXT_TILEMAP_CUR: usize = 0x1ce2;
const CHOICE: usize = 0x1ce8;
const CHOICE_BACKUP: usize = 0x1cf4;
const TEXT_WAIT_COUNTDOWN2: usize = 0x1ce9;
const DIALOGUE_SCROLL_SPEED: usize = 0x1cea;
const DIALOGUE_INDEX: usize = 0x1cf0;
const DIALOGUE_NUMBER: usize = 0x1cf2;
const VWF_ARR: usize = 0xc230;
const MESSAGE_PIXELS: usize = 0x10000;
const MESSAGE_TEXT: usize = 0x11200;
const MAPBAK_BG1_HOFS2: usize = 0xc200;
const MAPBAK_BG2_HOFS2: usize = 0xc202;
const MAPBAK_BG1_VOFS2: usize = 0xc204;
const MAPBAK_BG2_VOFS2: usize = 0xc206;
const MAPBAK_TM: usize = 0xc211;
const MAPBAK_BG1_X: usize = 0xc221;
const MAPBAK_BG1_Y: usize = 0xc223;
const MAPBAK_CGWSEL: usize = 0xc225;
const MAPBAK_HDMAEN: usize = 0xc229;
const LINK_ITEM_BOW: usize = 0xf340;
const LINK_HEARTS_FILLER: usize = 0xf372;
const LINK_PENDANTS: usize = 0xf374;
const LINK_CRYSTALS: usize = 0xf37a;
const MAP_ICONS: usize = 0xf3c7;

pub const TEXT_COMMAND_START_US: u8 = 0x67;
pub const TEXT_DICTIONARY_BASE: u8 = 0x88;
pub const CMD_NEXT_PIC: u8 = 0;
pub const CMD_CHOOSE: u8 = 1;
pub const CMD_ITEM: u8 = 2;
pub const CMD_NAME: u8 = 3;
pub const CMD_WINDOW: u8 = 4;
pub const CMD_NUMBER: u8 = 5;
pub const CMD_POSITION: u8 = 6;
pub const CMD_SCROLL_SPEED: u8 = 7;
pub const CMD_SELECTION_CHANGE: u8 = 8;
pub const CMD_CHOOSE3: u8 = 10;
pub const CMD_CHOOSE2: u8 = 11;
pub const CMD_SCROLL: u8 = 12;
pub const CMD_LINE1: u8 = 13;
pub const CMD_LINE2: u8 = 14;
pub const CMD_LINE3: u8 = 15;
pub const CMD_COLOR: u8 = 16;
pub const CMD_WAIT: u8 = 17;
pub const CMD_SOUND: u8 = 18;
pub const CMD_SPEED: u8 = 19;
pub const CMD_MARK: u8 = 20;
pub const CMD_MARK2: u8 = 21;
pub const CMD_CLEAR: u8 = 22;
pub const CMD_WAIT_KEY: u8 = 23;
pub const CMD_END: u8 = 24;
pub const CMD_LETTER: u8 = 25;

const TEXT_POSITIONS: [u16; 2] = [0x6125, 0x6244];
const TEXT_INITIALIZATION: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0x39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1c, 4, 0, 0, 0,
    0, 0,
];
const BORDER_TILES: [u16; 9] = [
    0x28f3, 0x28f4, 0x68f3, 0x28c8, 0x387f, 0x68c8, 0xa8f3, 0xa8f4, 0xe8f3,
];
pub const COMMAND_LENGTHS: [u8; 25] = [
    1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1,
];
const US_MULTIBYTE: [u8; 25] = [
    0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0,
];
const ROW_POSITIONS: [u16; 3] = [0, 2, 4];
const WAIT_DURATIONS: [u16; 16] = [
    31, 63, 94, 125, 156, 188, 219, 250, 281, 313, 344, 375, 406, 438, 469, 500,
];
const VWF_MASKS: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
pub const HEALTH_AFTER_DEATH: [u8; 21] = [
    0x18, 0x18, 0x18, 0x18, 0x18, 0x20, 0x20, 0x28, 0x28, 0x30, 0x30, 0x38, 0x38, 0x38, 0x40, 0x40,
    0x40, 0x48, 0x48, 0x48, 0x50,
];
const BIRD_X_LO: [u8; 8] = [0x80, 0xcf, 0x10, 0xb8, 0x30, 0x70, 0x70, 0xf0];
const BIRD_X_HI: [u8; 8] = [6, 0xc, 2, 8, 0xf, 0, 7, 0xe];
const BIRD_Y_LO: [u8; 8] = [0x5b, 0x98, 0xc0, 0x20, 0x50, 0xb0, 0x30, 0x80];
const BIRD_Y_HI: [u8; 8] = [3, 5, 7, 0xb, 0xb, 0xf, 0xf, 0xf];
const OVERWORLD_MAP_CHARS: [u8; 7] = [0x79, 0x6e, 0x6f, 0x6d, 0x7c, 0x6c, 0x7f];
const PENDANT_MASK: [u8; 3] = [4, 1, 2];
const CRYSTAL_MASK: [u8; 7] = [2, 0x40, 8, 0x20, 1, 4, 0x10];
const MAP_SCROLL_TARGET: [i16; 8] = [0, 0, 224, 480, -72, -224, 0, 0];
const MAP_SCROLL_STEP: [i16; 8] = [0, 0, 1, 2, -1, -2, 1, 2];
const OVERWORLD_MAP_PROJECTION: [u8; 333] = [
    0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xdf,
    0xde, 0xdd, 0xdc, 0xdb, 0xda, 0xd8, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0, 0xcf, 0xce,
    0xcd, 0xcc, 0xcb, 0xca, 0xc9, 0xc7, 0xc6, 0xc5, 0xc4, 0xc3, 0xc2, 0xc1, 0xc0, 0xbf, 0xbe, 0xbd,
    0xbc, 0xbb, 0xba, 0xb9, 0xb8, 0xb7, 0xb6, 0xb5, 0xb4, 0xb3, 0xb2, 0xb1, 0xb0, 0xaf, 0xae, 0xad,
    0xac, 0xab, 0xaa, 0xa9, 0xa8, 0xa7, 0xa6, 0xa5, 0xa4, 0xa3, 0xa2, 0xa1, 0xa0, 0x9f, 0x9e, 0x9d,
    0x9c, 0x9b, 0x9b, 0x9a, 0x99, 0x98, 0x97, 0x96, 0x95, 0x94, 0x93, 0x92, 0x91, 0x90, 0x8f, 0x8e,
    0x8d, 0x8c, 0x8b, 0x8b, 0x8a, 0x89, 0x88, 0x87, 0x86, 0x85, 0x84, 0x83, 0x82, 0x81, 0x81, 0x80,
    0x7f, 0x7e, 0x7d, 0x7c, 0x7b, 0x7a, 0x79, 0x79, 0x78, 0x77, 0x76, 0x75, 0x74, 0x73, 0x72, 0x72,
    0x71, 0x70, 0x6f, 0x6e, 0x6d, 0x6c, 0x6c, 0x6b, 0x6a, 0x69, 0x68, 0x67, 0x67, 0x66, 0x65, 0x64,
    0x63, 0x62, 0x62, 0x61, 0x60, 0x5f, 0x5e, 0x5d, 0x5d, 0x5c, 0x5b, 0x5a, 0x59, 0x59, 0x58, 0x57,
    0x56, 0x55, 0x55, 0x54, 0x53, 0x52, 0x51, 0x51, 0x50, 0x4f, 0x4e, 0x4e, 0x4d, 0x4c, 0x4b, 0x4a,
    0x4a, 0x49, 0x48, 0x47, 0x47, 0x46, 0x45, 0x44, 0x44, 0x43, 0x42, 0x41, 0x41, 0x40, 0x3f, 0x3e,
    0x3e, 0x3d, 0x3c, 0x3c, 0x3b, 0x3a, 0x39, 0x39, 0x38, 0x37, 0x36, 0x36, 0x35, 0x34, 0x34, 0x33,
    0x32, 0x32, 0x31, 0x30, 0x2f, 0x2f, 0x2e, 0x2d, 0x2d, 0x2c, 0x2b, 0x2b, 0x2a, 0x29, 0x29, 0x28,
    0x27, 0x27, 0x26, 0x25, 0x25, 0x24, 0x23, 0x23, 0x22, 0x21, 0x21, 0x20, 0x1f, 0x1f, 0x1e, 0x1d,
    0x1d, 0x1c, 0x1c, 0x1b, 0x1a, 0x1a, 0x19, 0x18, 0x18, 0x17, 0x17, 0x16, 0x15, 0x15, 0x14, 0x14,
    0x13, 0x12, 0x12, 0x11, 0x10, 0x10, 0x0f, 0x0f, 0x0e, 0x0e, 0x0d, 0x0c, 0x0c, 0x0b, 0x0b, 0x0a,
    9, 9, 8, 8, 7, 7, 6, 5, 5, 4, 4, 3, 3, 2, 1, 1, 0, 0, 0, 0, 0xff, 0xfe, 0xfe, 0xfd, 0xfc, 0xfc,
    0xfb, 0xfb, 0xfa, 0xf9, 0xf9, 0xf8, 0xf7, 0xf7, 0xf6, 0xf5, 0xf4, 0xf4, 0xf3, 0xf2, 0xf2, 0xf1,
    0xf0, 0xef, 0xee, 0xee, 0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe8, 0xe8, 0xe7, 0xe6, 0xe5, 0xe4, 0xe3,
    0xe2, 0xe1, 0xe0,
];
const OW_MAP_X: [[u16; 9]; 7] = [
    [0x7ff, 0x2c0, 0xd00, 0xf31, 0x6d, 0x7e0, 0xf40, 0xf40, 0x8dc],
    [
        0xff00, 0xff00, 0xff00, 0x8d0, 0xff00, 0xff00, 0xff00, 0x82, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0x108, 0xff00, 0xff00, 0xff00, 0xf11, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0x6d, 0xff00, 0xff00, 0xff00, 0x1d0, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0x100, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xca0, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0x759, 0xff00,
    ],
];
const OW_MAP_Y: [[u16; 9]; 7] = [
    [0x730, 0x6a0, 0x710, 0x620, 0x70, 0x640, 0x620, 0x620, 0x30],
    [
        0xff00, 0xff00, 0xff00, 0x80, 0xff00, 0xff00, 0xff00, 0xb0, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0xd70, 0xff00, 0xff00, 0xff00, 0x103, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0x70, 0xff00, 0xff00, 0xff00, 0x780, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xca0, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xda0, 0xff00,
    ],
    [
        0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xed0, 0xff00,
    ],
];
const OW_MAP_INFO: [[u16; 9]; 7] = [
    [0, 0, 0, 0x6038, 0x6234, 0x6632, 0x6434, 0x6434, 0x6632],
    [0, 0, 0, 0x6032, 0, 0, 0, 0x6434, 0],
    [0, 0, 0, 0x6034, 0, 0, 0, 0x6434, 0],
    [0, 0, 0, 0x6234, 0, 0, 0, 0x6434, 0],
    [0, 0, 0, 0, 0, 0, 0, 0x6434, 0],
    [0, 0, 0, 0, 0, 0, 0, 0x6434, 0],
    [0, 0, 0, 0, 0, 0, 0, 0x6434, 0],
];
const DUNG_TAB0: [i8; 14] = [-1, -1, -1, -1, -1, 2, 0, 10, 4, 8, -1, 6, 12, 14];
const DUNG_TAB1: [u16; 8] = [
    0x2108, 0x2109, 0x2109, 0x210a, 0x210b, 0x210c, 0x210d, 0x211d,
];
const DUNG_TAB2: [u16; 8] = [
    0x2118, 0x2119, 0xa109, 0x211a, 0x211b, 0x211c, 0x2118, 0xa11d,
];
const DUNG_TAB3: [u8; 14] = [
    0x60, 0x84, 0, 0xb, 0x32, 0x21, 0x33, 0x21, 0x38, 0x21, 0x3a, 0x21, 0x7f, 0x20,
];
const DUNG_TAB4: [u8; 14] = [
    0x60, 0xa4, 0, 0xb, 0x42, 0x21, 0x43, 0x21, 0x49, 0x21, 0x4a, 0x21, 0x7f, 0x20,
];
const DUNG_TAB5: [u16; 14] = [
    0x21, 0x23, 0x20, 0x21, 0x70, 0x12, 0x11, 0x212, 2, 0x217, 0x160, 0x12, 0x113, 0x171,
];
const DUNG_TAB6: [u16; 21] = [
    0xaa10, 0x100, 0x1b2f, 0xc910, 0x300, 0x1b2f, 0x1b2e, 0xe510, 0xb00, 0x1b2f, 0x1b2e, 0x5b2f,
    0x1b2f, 0x1b2e, 0x1b2e, 0x311, 0x100, 0x1b2f, 0x411, 0xc40, 0x1b2e,
];
const DUNG_TAB7: [u16; 9] = [
    0x1223, 0x1263, 0x12a3, 0x12e3, 0x1323, 0x11e3, 0x11a3, 0x1163, 0x1123,
];
const DUNG_TAB8: [u16; 7] = [0x1b28, 0x1b29, 0x1b2a, 0x1b2b, 0x1b2c, 0x1b2d, 0x1b2e];
const DUNG_TAB9: [u16; 8] = [0xf26, 0xf27, 0x4f27, 0x4f26, 0x8f26, 0x8f27, 0xcf27, 0xcf26];
const DUNG_TAB10: [u16; 4] = [0xe2, 0xf8, 0x3a2, 0x3b8];
const DUNG_TAB11: [u16; 4] = [0x1f19, 0x5f19, 0x9f19, 0xdf19];
const DUNG_TAB12: [u16; 2] = [0xe4, 0x3a4];
const DUNG_TAB13: [u16; 2] = [0x1f1a, 0x9f1a];
const DUNG_TAB14: [u16; 2] = [0x122, 0x138];
const DUNG_TAB15: [u16; 2] = [0x1f1b, 0x5f1b];
const DUNG_TAB16: [u16; 8] = [
    0x1f1e, 0x1f1f, 0x1f20, 0x1f21, 0x1f22, 0x1f23, 0x1f24, 0x1f25,
];
const DUNG_TAB21: [u16; 3] = [137, 167, 79];
const DUNG_TAB22: [u16; 3] = [169, 119, 190];
const DUNG_TAB24: [u16; 2] = [0x1f, 0x7f];
const DUNG_TAB25: [u16; 14] = [15, 15, 200, 51, 32, 6, 90, 144, 41, 222, 7, 172, 164, 13];
const DUNG_TAB28: [i16; 14] = [
    -1, -1, 1, 1, 6, 0xff, 0xff, 0xff, 0xfe, 0xf9, 5, 0xff, 0xfd, 6,
];
const DUNG_TAB29: [i8; 4] = [-9, 8, -9, 8];
const DUNG_TAB30: [i8; 4] = [-8, -8, 9, 9];
const DUNG_TAB31: [u8; 4] = [0xf1, 0xb1, 0x71, 0x31];
const DUNG_TAB32: [u8; 4] = [0xc, 0xc, 8, 0xa];
const DUNG_TAB33: [u8; 8] = [187, 171, 155, 139, 123, 107, 91, 75];
const DUNG_TAB34: [u8; 8] = [0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25];
const DUNG_TAB35: [usize; 2] = [0, 8];
const DUNG_TAB36: [u8; 4] = [0x37, 0x38, 0x38, 0x37];
const DUNG_TAB37: [i16; 14] = [
    -1, -1, 0x808, 8, 0, 8, 0x808, 8, 0x808, 0x800, 0x404, 0x808, 8, 8,
];
const DUNG_TAB38: [u8; 4] = [0x39, 0x3b, 0x3d, 0x3b];
const UPPER_BITMASKS: [u16; 16] = [
    0x8000, 0x4000, 0x2000, 0x1000, 0x800, 0x400, 0x200, 0x100, 0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1,
];
const PRAYING_SCENE_TAB1: [u8; 129] = [
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
const PRAYING_SCENE_TAB0: [u8; 129] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xfd, 0xfd, 0xfc, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8,
    0xf7, 0xf6, 0xf5, 0xf4, 0xf3, 0xf1, 0xf0, 0xee, 0xed, 0xeb, 0xe9, 0xe8, 0xe6, 0xe4, 0xe2, 0xdf,
    0xdd, 0xdb, 0xd8, 0xd6, 0xd3, 0xd0, 0xcd, 0xca, 0xc7, 0xc4, 0xc1, 0xbd, 0xb9, 0xb6, 0xb1, 0xad,
    0xa9, 0xa4, 0x9f, 0x9a, 0x95, 0x8f, 0x89, 0x82, 0x7b, 0x74, 0x6c, 0x63, 0x59, 0x4d, 0x3f, 0x2d,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];
const DUNG_TAB23: [u16; 744] = [
    0xb61, 0x5361, 0x8b61, 0x8b62, 0xb60, 0xb63, 0x8b60, 0xb64, 0xb00, 0xb00, 0xb65, 0xb66, 0xb67,
    0x4b67, 0x9367, 0xd367, 0xb60, 0x5360, 0x8b60, 0xcb60, 0xb6a, 0x4b6a, 0x4b6d, 0xb6d, 0x1368,
    0x1369, 0xb00, 0xb00, 0xb6a, 0x136b, 0xb6c, 0xb6d, 0x136e, 0x4b6e, 0xb00, 0xb00, 0x136f, 0xb00,
    0xb00, 0xb00, 0x1340, 0xb00, 0xb78, 0x1744, 0x536d, 0x136d, 0x4b76, 0xb76, 0xb70, 0xb71, 0xb72,
    0x8b71, 0xb75, 0xb76, 0x8b75, 0x8b76, 0xb00, 0xb53, 0xb00, 0xb55, 0x1354, 0x5354, 0xb00, 0xb00,
    0x4b53, 0xb00, 0xb56, 0xb57, 0xb00, 0xb59, 0xb00, 0x135e, 0x135a, 0x135b, 0x135f, 0x535f,
    0xb5c, 0xb5d, 0x535e, 0xcb58, 0xb50, 0x4b50, 0x1352, 0x5352, 0xb00, 0xb40, 0x1345, 0xb46,
    0x8b42, 0xb47, 0xb42, 0xb49, 0x1348, 0x5348, 0x174a, 0x574a, 0x4b47, 0xcb42, 0x4b49, 0x4b42,
    0xb00, 0xb4b, 0xb00, 0xb4d, 0xb4c, 0x4b4c, 0xb4e, 0x4b4e, 0xb51, 0xb44, 0xb00, 0xb00, 0xb4f,
    0x4b4f, 0x934f, 0xd34f, 0xb00, 0xb00, 0xb00, 0xb40, 0xb00, 0xb41, 0xb00, 0xb42, 0xb00, 0xb00,
    0xb43, 0xb43, 0xb00, 0xb00, 0x9344, 0xb00, 0x1340, 0xb00, 0x1341, 0xb00, 0x1740, 0xb40, 0xb42,
    0xb7d, 0x4b7a, 0xb7a, 0xb7e, 0x4b7e, 0xb40, 0x8b4d, 0x4bba, 0xb55, 0xb40, 0x8b55, 0x1378,
    0xcb53, 0x4b76, 0x4b75, 0x13bb, 0x53bb, 0x4b7f, 0x4b42, 0xb83, 0x13bc, 0xb00, 0xb00, 0xb79,
    0xb00, 0xb6e, 0x4b7c, 0xb00, 0xb41, 0x1340, 0x8b55, 0xb42, 0xb7b, 0x8b42, 0x9344, 0x1341,
    0xb00, 0xb53, 0x9344, 0x8b53, 0x9344, 0x8b42, 0x9344, 0xb42, 0x9344, 0x934d, 0xb00, 0x8b53,
    0x9344, 0xb00, 0xb00, 0xb40, 0xb00, 0xb41, 0xb00, 0x1384, 0xb00, 0xbb8, 0x13b9, 0x4b85, 0xcb7c,
    0xb87, 0x13b0, 0x4b7b, 0x9344, 0xb00, 0xb00, 0xb40, 0xb00, 0xb91, 0x5391, 0xb9c, 0x4b9c,
    0x8b42, 0x1392, 0xb93, 0x1394, 0xb95, 0xb96, 0x9395, 0x8b96, 0xb97, 0xb98, 0x8b97, 0x8b98,
    0x1799, 0x5799, 0x9799, 0xd799, 0x4b98, 0x4b97, 0xcb98, 0xcb97, 0x937b, 0xb00, 0xb7b, 0xb00,
    0xba6, 0x4ba6, 0xcb7a, 0x8b7a, 0xb8e, 0x4b8e, 0x938e, 0xcb8e, 0x934d, 0xb8f, 0x1390, 0x5390,
    0xb00, 0xb00, 0xb00, 0x8b48, 0xb00, 0x934e, 0xb00, 0x8b4d, 0x8b72, 0x1346, 0xb45, 0xb46,
    0x5744, 0x1744, 0xb00, 0xb00, 0x134d, 0xb00, 0x8b54, 0xb00, 0x1349, 0x1349, 0xb00, 0xb00,
    0xb4b, 0x8b48, 0xb72, 0x4b72, 0xb00, 0xb74, 0xb00, 0xbb0, 0xb71, 0x1747, 0x17af, 0xb4b, 0xb6f,
    0x1370, 0xb4b, 0xb00, 0xb6b, 0x8b6c, 0x8b6b, 0xbad, 0xb73, 0xb00, 0x13ae, 0xb46, 0x176b,
    0x576b, 0xb6a, 0x4b6a, 0x1368, 0x5368, 0x1369, 0x5369, 0x8b4e, 0xb00, 0x9354, 0xb00, 0xb00,
    0xb00, 0xb00, 0x5377, 0xb00, 0x974d, 0xb00, 0x4b7b, 0xb40, 0x8b4d, 0xb51, 0xb8d, 0x537a,
    0x137a, 0x4b42, 0x8b40, 0xb00, 0xb00, 0xb00, 0xb00, 0xb00, 0xb00, 0xb40, 0xb00, 0xcb7a, 0x576e,
    0xb00, 0xb00, 0xb6e, 0xb9f, 0xb00, 0x4ba5, 0x13a0, 0x13a1, 0xba2, 0xba3, 0xba4, 0xb00, 0xba5,
    0xb00, 0xb40, 0x8b55, 0xb42, 0xcb87, 0x8b95, 0xba7, 0x8b42, 0xbaf, 0x4b78, 0xb00, 0x4b78,
    0xb00, 0x8b42, 0xb51, 0xb78, 0x8b51, 0xba8, 0xba9, 0xbac, 0x8ba9, 0xbaa, 0x17ab, 0x13b4,
    0x8bab, 0x17b1, 0xb41, 0x4b44, 0x4b42, 0xb00, 0xbad, 0xb00, 0x13ae, 0x1340, 0xbb7, 0xb42,
    0xbb6, 0xb00, 0xb00, 0x139d, 0x139e, 0xb00, 0xb00, 0xb00, 0xb79, 0xb00, 0xb00, 0x8b42, 0xb86,
    0xb42, 0x8b7b, 0x8b42, 0xb7b, 0xb87, 0x8b7b, 0x9387, 0xb7b, 0xb40, 0x13b3, 0x1378, 0xb8d,
    0x8b42, 0xb88, 0x5378, 0xb40, 0x4b44, 0xd342, 0x97b5, 0x4b78, 0x13b3, 0x8b55, 0x4b7b, 0xb8d,
    0xb89, 0x138a, 0xb8b, 0xb8c, 0xb00, 0xb7c, 0xb00, 0xb00, 0xb00, 0x9348, 0xb00, 0xb56, 0xb00,
    0xb00, 0xb88, 0xb00, 0xb00, 0xb48, 0xb00, 0xb00, 0xb00, 0x9348, 0x1786, 0xb65, 0xb00, 0xb00,
    0xcb5a, 0xb00, 0xb00, 0x5388, 0xb00, 0xb00, 0x4b5a, 0xb00, 0xb00, 0xb00, 0xb00, 0xcb5b, 0x13ab,
    0xbac, 0xcb5a, 0xb00, 0x137e, 0xb00, 0xb00, 0x137e, 0xb00, 0xb00, 0xb00, 0x8b48, 0x1783,
    0x1384, 0xb00, 0xb00, 0x1385, 0xb00, 0xb00, 0x537e, 0xb00, 0xb00, 0xb00, 0x8b48, 0xb43, 0xcb43,
    0xb00, 0xb00, 0x1379, 0x137a, 0xb5a, 0x137b, 0xb00, 0xb00, 0xb00, 0x8b48, 0x137f, 0x1380,
    0xb00, 0xb00, 0x1381, 0x1382, 0xb00, 0xb48, 0xb00, 0xb00, 0xb00, 0xb00, 0x1387, 0x1377, 0x5746,
    0xb47, 0x1349, 0xb48, 0x1375, 0x4b42, 0x174a, 0x574a, 0xb43, 0x1344, 0xb45, 0x1746, 0x1742,
    0x5742, 0x8b42, 0xcb42, 0x1375, 0x5375, 0x8b42, 0xcb42, 0x4b40, 0x1340, 0xb41, 0x4b41, 0x4b46,
    0xb71, 0x1786, 0x8b71, 0x1347, 0xb4d, 0xb65, 0xb5b, 0xb00, 0xb00, 0x9348, 0xb00, 0xb00, 0xb00,
    0xb00, 0x8b48, 0x4b66, 0x8b65, 0x4b5b, 0xb65, 0x9365, 0xb66, 0xb63, 0x8b66, 0x4b51, 0xb5f,
    0xcb76, 0xb60, 0xb64, 0x4b4f, 0x4b60, 0x8b76, 0x4b76, 0xb61, 0xd376, 0x1362, 0x4b61, 0xb76,
    0xcb58, 0x8b51, 0xb00, 0xb00, 0x5746, 0xb5e, 0xb00, 0xb00, 0xb5e, 0xb46, 0xb00, 0xb00, 0x8b48,
    0xb00, 0xb4f, 0xb51, 0xcb76, 0x8b76, 0x5351, 0xb51, 0x8b4f, 0x8b51, 0x4b76, 0xb76, 0xcb51,
    0x8b58, 0xb54, 0xb00, 0x8b66, 0xb00, 0x9348, 0x8b48, 0xb56, 0x4b45, 0xb00, 0xb57, 0xb00, 0xb59,
    0x4b50, 0xb58, 0xcb50, 0x8b50, 0x5758, 0x1751, 0xcb58, 0x8b51, 0xb56, 0x4b56, 0xb65, 0x5756,
    0x9348, 0x8b48, 0xb4c, 0xb4b, 0xb4d, 0xb00, 0x8b54, 0xb00, 0xb4f, 0xb50, 0x8b4f, 0x8b50,
    0x4b50, 0xb51, 0xcb58, 0x8b51, 0xb52, 0xb54, 0xb53, 0x9354, 0x9748, 0x9748, 0x138d, 0x138e,
    0x1391, 0x1392, 0x138c, 0x138f, 0x1393, 0x1390, 0x9393, 0x138f, 0x1394, 0x1395, 0x138e, 0x138c,
    0x175d, 0x1399, 0x975d, 0x538f, 0x1397, 0x1398, 0x179a, 0x138c, 0x1399, 0x1766, 0x138f, 0xd75d,
    0x538e, 0x538f, 0x1391, 0x1392, 0x139b, 0x539b, 0x139c, 0x539c, 0x138f, 0x138e, 0x5392, 0x5391,
    0x138a, 0x538a, 0x138b, 0x538b, 0xb00, 0xcb5b, 0xb00, 0x8b54, 0x4b74, 0x13a6, 0xb00, 0x4b48,
    0x13a0, 0x13a1, 0x538e, 0x138e, 0xd38e, 0x53a3, 0x13a4, 0xb00, 0x97aa, 0xb00, 0x538e, 0x1399,
    0x13a4, 0xb00, 0x138e, 0xb00, 0xb00, 0x5393, 0xb00, 0x574e, 0x4b7d, 0xb00, 0x8b7d, 0x139f,
    0x97aa, 0x13a4, 0x13a9, 0x53a9, 0x13a5, 0x13a6, 0x93a5, 0xd3a5, 0xd38e, 0x938e, 0x13a4, 0x13aa,
    0xb00, 0x13a6, 0xb00, 0x8b5f, 0x139b, 0x13a6, 0x139c, 0x53a2, 0xb00, 0xb00, 0x138c, 0xb00,
    0x9394, 0x139e, 0xb00, 0xb00,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedCommand(pub u32);
impl DecodedCommand {
    pub const fn new(param: u8, command: u8, multibyte: bool) -> Self {
        Self(((param as u32) << 6) | ((command as u32) << 1) | (multibyte as u32))
    }
    pub const fn param(self) -> u8 {
        (self.0 >> 6) as u8
    }
    pub const fn command(self) -> u8 {
        ((self.0 >> 1) & 0x1f) as u8
    }
    pub const fn multibyte(self) -> bool {
        self.0 & 1 != 0
    }
}

#[must_use]
pub fn decode_command(dialogue_flags: u8, a: u8, next: u8) -> DecodedCommand {
    if dialogue_flags & 1 == 0 {
        if a < TEXT_COMMAND_START_US {
            return DecodedCommand::new(a, CMD_LETTER, false);
        }
        if a >= 0x80 {
            return DecodedCommand::new(26, CMD_LETTER, false);
        }
        let c = a - TEXT_COMMAND_START_US;
        return DecodedCommand::new(
            if US_MULTIBYTE[usize::from(c)] != 0 {
                next
            } else {
                0
            },
            c,
            US_MULTIBYTE[usize::from(c)] != 0,
        );
    }
    if a < 0x7f {
        return DecodedCommand::new(a, CMD_LETTER, false);
    }
    if a < 0x87 {
        const SIMPLE: [u8; 8] = [
            CMD_END,
            CMD_SCROLL,
            CMD_WAIT_KEY,
            CMD_LINE1,
            CMD_LINE2,
            CMD_LINE3,
            CMD_NAME,
            CMD_NAME,
        ];
        return DecodedCommand::new(0, SIMPLE[usize::from(a - 0x7f)], false);
    }
    match next >> 4 {
        0 => DecodedCommand::new(next & 15, CMD_WAIT, true),
        1 => DecodedCommand::new(next & 15, CMD_COLOR, true),
        2 => DecodedCommand::new(next & 15, CMD_NUMBER, true),
        3 => DecodedCommand::new(next & 15, CMD_SPEED, true),
        4 => DecodedCommand::new(45, CMD_SOUND, true),
        8 => {
            const EXT: [(u8, u8); 9] = [
                (0, CMD_CHOOSE),
                (0, CMD_CHOOSE2),
                (0, CMD_CHOOSE3),
                (0, CMD_SELECTION_CHANGE),
                (0, CMD_ITEM),
                (0, CMD_NEXT_PIC),
                (2, CMD_WINDOW),
                (0, CMD_POSITION),
                (1, CMD_POSITION),
            ];
            let (p, c) = EXT[usize::from(next - 0x80)];
            DecodedCommand::new(p, c, true)
        }
        _ => DecodedCommand::new(26, CMD_LETTER, false),
    }
}

pub struct MessagingAssets<'a> {
    pub dialogue_flags: u8,
    pub dictionary: &'a [&'a [u8]],
    pub messages: &'a [&'a [u8]],
    pub font_tiles: &'a [u8],
    pub font_widths: &'a [u8],
    pub light_overworld_tilemap: &'a [u8],
    pub dungeon_map_floor_layout: &'a [u8],
    pub dungeon_map_info: &'a [u8],
}
pub trait MessagingRuntime {
    fn misc_bugfixes_enabled(&self) -> bool {
        false
    }
    fn run_hud_module(&mut self, _ram: &mut Ram<'_>) {}
    fn hud_floor_indicator(&mut self, _ram: &mut Ram<'_>) {}
    fn link_oam_main(&mut self, _ram: &mut Ram<'_>) {}
    fn enable_force_blank(&mut self, _ram: &mut Ram<'_>) {}
    fn reset_hud_palettes_4_and_5(&mut self, _ram: &mut Ram<'_>) {}
    fn decompress_story_gfx(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_filter_history(&mut self, _ram: &mut Ram<'_>) {}
    fn apply_palette_filter_bounce(&mut self, _ram: &mut Ram<'_>) {}
    fn reset_transition_props_and_advance_reset_interface(&mut self, _ram: &mut Ram<'_>) {}
    fn hud_refill_health(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn hud_refill_magic(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn draw_selected_item(&mut self, _ram: &mut Ram<'_>, _item: u8) {}
    fn world_map_load_light(&mut self, _ram: &mut Ram<'_>) {}
    fn world_map_load_dark(&mut self, _ram: &mut Ram<'_>) {}
    fn world_map_load_sprite_gfx(&mut self, _ram: &mut Ram<'_>) {}
    fn world_map_restore_gfx(&mut self, _ram: &mut Ram<'_>) {}
    fn world_map_exit(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_map_step(&mut self, _ram: &mut Ram<'_>, _state: u8) {}
    fn death_step(&mut self, _ram: &mut Ram<'_>, _state: u8) {}
    fn flute_load_selected_screen(&mut self, _ram: &mut Ram<'_>) {}
    fn flute_load_overlay(&mut self, _ram: &mut Ram<'_>) {}
    fn flute_finish(&mut self, _ram: &mut Ram<'_>) {}
    fn flute_sprite_main(&mut self, _ram: &mut Ram<'_>) {}
    fn transfer_mode7_characters(&mut self, _ram: &mut Ram<'_>) {}
    fn overworld_map_setup_hdma(&mut self, _ram: &mut Ram<'_>, _zoomed: bool) {}
    fn interface_gameplay_step(&mut self, _ram: &mut Ram<'_>, _run_world: bool) {}
    fn desert_prayer_setup_hdma(&mut self, _ram: &mut Ram<'_>) {}
    fn iris_spotlight_reset_table(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_map_prepare_graphics(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_map_recover_graphics(&mut self, _ram: &mut Ram<'_>) {}
    fn death_finalize_choice(
        &mut self,
        _ram: &mut Ram<'_>,
        _sram: &mut [u8],
        _count_as_death: bool,
    ) {
    }
    fn display_select_menu(&mut self, _ram: &mut Ram<'_>) {}
    fn link_reset_properties_c(&mut self, _ram: &mut Ram<'_>) {}
    fn load_actual_gear_palettes(&mut self, _ram: &mut Ram<'_>) {}
    fn calculate_link_sfx_pan(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
}
pub struct Messaging<'a, 'r, 's, 'd, R: MessagingRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub sram: &'s mut [u8],
    pub assets: &'d MessagingAssets<'d>,
    pub runtime: &'a mut R,
}

impl<R: MessagingRuntime> Messaging<'_, '_, '_, '_, R> {
    pub fn render_text(&mut self) {
        match self.ram.read_u8(MESSAGING_MODULE) {
            0 => self.initialize(),
            1 => self.text_render(),
            2 => self.post_death_save_options(),
            _ => {}
        }
    }
    pub fn post_death_save_options(&mut self) {
        self.ram.write_u16(DIALOGUE_INDEX, 3);
        self.initialize_state_loop();
        self.ram.write_u16(TEXT_TOPLEFT, 0x61e8);
        self.ram.write_u8(TEXT_RENDER_STATE, 2);
        for _ in 0..5 {
            self.text_render();
        }
    }
    pub fn text_render(&mut self) {
        match self.ram.read_u8(TEXT_RENDER_STATE) {
            0 => self.draw_border(),
            1 => self.draw_border_incremental(),
            2 => {
                self.build_character_tilemap();
                self.ram.write_u8(TEXT_RENDER_STATE, 3)
            }
            3 => self.draw_message_characters(),
            4 => self.draw_finish(),
            _ => {}
        }
    }
    pub fn draw_border(&mut self) {
        self.draw_border_initialize();
        let mut d = self.draw_border_row(0, 0);
        for _ in 0..6 {
            d = self.draw_border_row(d, 6)
        }
        self.draw_border_row(d, 12);
        self.ram.write_u8(NMI_LOAD_BG, 1);
        self.ram.write_u8(TEXT_RENDER_STATE, 2);
    }
    pub fn draw_border_incremental(&mut self) {
        self.ram.write_u8(NMI_LOAD_BG, 1);
        let s = self.ram.read_u8(TEXT_INCREMENTAL_STATE);
        let a = if s == 0 {
            0
        } else if s < 7 {
            1
        } else {
            2
        };
        match a {
            0 => {
                self.draw_border_initialize();
                self.draw_border_row(0, 0);
            }
            1 => {
                self.draw_border_row(0, 6);
            }
            _ => {
                self.ram.write_u8(TEXT_RENDER_STATE, 2);
                self.draw_border_row(0, 12);
            }
        }
        self.ram.write_u8(TEXT_INCREMENTAL_STATE, s.wrapping_add(1));
    }
    pub fn draw_finish(&mut self) {
        self.draw_border_initialize();
        self.ram.write_u16(
            VRAM_UPLOAD_DATA,
            self.ram.read_u16(TEXT_TOPLEFT_COPY).swap_bytes(),
        );
        self.ram.write_u16(VRAM_UPLOAD_DATA + 2, 0x2e42);
        self.ram.write_u16(VRAM_UPLOAD_DATA + 4, 0x387f);
        self.ram.write_u16(VRAM_UPLOAD_DATA + 6, 0xffff);
        self.ram.write_u8(NMI_LOAD_BG, 1);
        self.ram.write_u8(MESSAGING_MODULE, 0);
        self.ram.write_u8(SUBMODULE, 0);
        self.ram
            .write_u8(MAIN_MODULE, self.ram.read_u8(SAVED_MODULE));
    }
    fn command_done(&mut self, c: DecodedCommand) {
        self.ram.write_u16(
            DIALOGUE_READ_POS,
            self.ram
                .read_u16(DIALOGUE_READ_POS)
                .wrapping_add(1 + u16::from(c.multibyte())),
        );
    }
    pub fn draw_message_characters(&mut self) {
        let mut diagnostic_iterations = 0usize;
        loop {
            diagnostic_iterations += 1;
            if diagnostic_iterations > 10_000 {
                panic!(
                    "message parser loop: module={:02x} sub={:02x} state={:02x} pos={:04x} byte={:02x}",
                    self.ram.read_u8(MAIN_MODULE),
                    self.ram.read_u8(SUBMODULE),
                    self.ram.read_u8(TEXT_RENDER_STATE),
                    self.ram.read_u16(DIALOGUE_READ_POS),
                    self.ram
                        .read_u8(MESSAGE_TEXT + usize::from(self.ram.read_u16(DIALOGUE_READ_POS)))
                );
            }
            let p = usize::from(self.ram.read_u16(DIALOGUE_READ_POS));
            let a = self.ram.read_u8(MESSAGE_TEXT + p);
            let c = decode_command(
                self.assets.dialogue_flags,
                a,
                self.ram.read_u8(MESSAGE_TEXT + p + 1),
            );
            let mut restart = false;
            match c.command() {
                CMD_LETTER => {
                    if self.ram.read_u8(VWF_LINE_SPEED_CUR) >= 2 {
                        self.ram
                            .write_u8(VWF_LINE_SPEED_CUR, self.ram.read_u8(VWF_LINE_SPEED_CUR) - 1)
                    } else {
                        self.vwf_render_single(c.param());
                        self.command_done(c);
                        restart = self.ram.read_u8(VWF_LINE_SPEED_CUR) == 0;
                    }
                }
                CMD_NEXT_PIC => {
                    if self.ram.read_u8(MAIN_MODULE) != 20 {
                        self.command_done(c)
                    } else {
                        self.runtime.palette_filter_history(self.ram);
                        if self.ram.read_u8(0xc007) == 0 {
                            self.command_done(c)
                        }
                    }
                }
                CMD_ITEM => self.choose_item(),
                CMD_SCROLL_SPEED => {
                    self.ram.write_u8(DIALOGUE_SCROLL_SPEED, c.param());
                    self.command_done(c)
                }
                CMD_LINE1 | CMD_LINE2 | CMD_LINE3 => {
                    self.ram.write_u16(
                        VWF_CUR_LINE,
                        ROW_POSITIONS[usize::from(c.command() - CMD_LINE1)],
                    );
                    self.ram.write_u16(VWF_FLAG_NEXT_LINE, 1);
                    self.command_done(c)
                }
                CMD_WAIT => {
                    let n = if self.ram.read_u8(JOYPAD_L) & 0x80 != 0 {
                        1
                    } else {
                        self.ram.read_u16(TEXT_WAIT_COUNTDOWN)
                    };
                    if n == 0 {
                        self.ram.write_u16(
                            TEXT_WAIT_COUNTDOWN,
                            WAIT_DURATIONS[usize::from(c.param())] - 1,
                        )
                    } else if n == 1 {
                        self.ram.write_u8(TEXT_WAIT_COUNTDOWN, 0);
                        self.command_done(c)
                    } else {
                        self.ram.write_u16(TEXT_WAIT_COUNTDOWN, n - 1)
                    }
                }
                CMD_SOUND => {
                    self.ram.write_u8(SOUND2, c.param());
                    self.command_done(c)
                }
                CMD_SPEED => {
                    self.ram.write_u8(VWF_LINE_SPEED, c.param());
                    self.ram.write_u8(VWF_LINE_SPEED_CUR, c.param());
                    self.command_done(c)
                }
                CMD_CHOOSE3 => self.choose_three(),
                CMD_CHOOSE => self.choose_two(1),
                CMD_SELECTION_CHANGE => self.choose_two(11),
                CMD_CHOOSE2 => self.choose_one_or_two(),
                CMD_SCROLL => {
                    if self.draw_scroll() {
                        self.command_done(c)
                    }
                }
                CMD_END => {
                    if self.accept_after_delay(false) {
                        self.ram.write_u8(TEXT_RENDER_STATE, 4)
                    }
                }
                CMD_WAIT_KEY => {
                    if self.accept_after_delay(true) {
                        self.command_done(c)
                    }
                }
                _ => {}
            }
            self.ram.write_u8(NMI_SUBROUTINE, 2);
            self.ram.write_u8(NMI_DISABLE_CORE, 2);
            if !restart {
                break;
            }
        }
    }
    fn accept_after_delay(&mut self, masked: bool) -> bool {
        let n = self.ram.read_u8(TEXT_WAIT_COUNTDOWN2);
        if n != 0 {
            let v = n - 1;
            self.ram.write_u8(TEXT_WAIT_COUNTDOWN2, v);
            if v == 1 {
                self.ram.write_u8(SOUND2, 36)
            }
            false
        } else {
            let y = self.ram.read_u8(0xf4) | self.ram.read_u8(0xf6);
            if (if masked { y & 0xc0 } else { y }) != 0 {
                self.ram.write_u8(TEXT_WAIT_COUNTDOWN2, 28);
                true
            } else {
                false
            }
        }
    }
    pub fn choose_two(&mut self, message_base: u16) {
        let delay = self.ram.read_u8(TEXT_WAIT_COUNTDOWN2);
        if delay != 0 {
            let delay = delay - 1;
            self.ram.write_u8(TEXT_WAIT_COUNTDOWN2, delay);
            if delay == 1 {
                self.ram.write_u8(SOUND2, 36)
            }
            return;
        }
        if (self.ram.read_u8(0xf4) | self.ram.read_u8(0xf6)) & 0xc0 != 0 {
            self.ram.write_u8(SOUND1, 43);
            self.ram.write_u8(TEXT_RENDER_STATE, 4);
            return;
        }
        let y = self.ram.read_u8(0xf4);
        if y & 12 != 0 {
            let t = if y & 8 != 0 { 0 } else { 1 };
            if self.ram.read_u8(CHOICE) != t {
                self.ram.write_u8(CHOICE, t);
                self.ram.write_u8(SOUND2, 32);
                self.ram
                    .write_u16(DIALOGUE_INDEX, message_base + u16::from(t));
                self.load_character_buffer();
                self.init_vwf_state();
            }
        }
    }
    fn choose_one_or_two(&mut self) {
        let delay = self.ram.read_u8(TEXT_WAIT_COUNTDOWN2);
        if delay != 0 {
            let delay = delay - 1;
            self.ram.write_u8(TEXT_WAIT_COUNTDOWN2, delay);
            if delay == 1 {
                self.ram.write_u8(SOUND2, 36);
            }
            return;
        }
        let input = self.ram.read_u8(0xf4) | (self.ram.read_u8(0xf6) & 0xc0);
        if input & 0xd0 != 0 {
            self.ram.write_u8(SOUND1, 43);
            self.ram.write_u8(TEXT_RENDER_STATE, 4);
        } else if input & 12 != 0 {
            let choice = u8::from(input & 8 == 0);
            if self.ram.read_u8(CHOICE) != choice {
                self.ram.write_u8(CHOICE, choice);
                self.ram.write_u8(SOUND2, 32);
                self.ram.write_u16(DIALOGUE_INDEX, 9 + u16::from(choice));
                self.load_character_buffer();
                self.init_vwf_state();
            }
        }
    }
    pub fn choose_item(&mut self) {
        let delay = self.ram.read_u8(TEXT_WAIT_COUNTDOWN2);
        if delay != 0 {
            let delay = delay.wrapping_sub(1);
            self.ram.write_u8(TEXT_WAIT_COUNTDOWN2, delay);
            if delay == 1 {
                self.find_y_item_next()
            }
            return;
        }
        if (self.ram.read_u8(0xf4) | self.ram.read_u8(0xf6)) & 0xc0 != 0 {
            self.ram.write_u8(TEXT_RENDER_STATE, 4);
            return;
        }
        let input = self.ram.read_u8(0xf4);
        if input & 5 != 0 {
            self.ram
                .write_u8(CHOICE, self.ram.read_u8(CHOICE).wrapping_add(1));
            self.find_y_item_next();
        } else if input & 10 != 0 {
            self.ram
                .write_u8(CHOICE, self.ram.read_u8(CHOICE).wrapping_sub(1));
            self.find_y_item_previous();
        } else {
            self.find_y_item_next();
        }
        self.refresh();
    }
    fn item_is_selectable(&self, item: u8) -> bool {
        item != 15
            && (self.ram.read_u8(LINK_ITEM_BOW + usize::from(item)) != 0
                || item == 32 && self.ram.read_u8(LINK_ITEM_BOW + 33) != 0)
    }
    pub fn find_y_item_previous(&mut self) {
        loop {
            let mut item = self.ram.read_u8(CHOICE);
            if item & 0x80 != 0 {
                item = 31;
                self.ram.write_u8(CHOICE, item)
            }
            if self.item_is_selectable(item) {
                break;
            }
            self.ram.write_u8(CHOICE, item.wrapping_sub(1));
        }
        let item = self.ram.read_u8(CHOICE);
        self.runtime.draw_selected_item(self.ram, item);
    }
    pub fn find_y_item_next(&mut self) {
        loop {
            let mut item = self.ram.read_u8(CHOICE);
            if item >= 32 {
                item = 0;
                self.ram.write_u8(CHOICE, item)
            }
            if self.item_is_selectable(item) {
                break;
            }
            self.ram.write_u8(CHOICE, item.wrapping_add(1));
        }
        let item = self.ram.read_u8(CHOICE);
        self.runtime.draw_selected_item(self.ram, item);
    }
    pub fn draw_scroll(&mut self) -> bool {
        let mut r = self.ram.read_u8(DIALOGUE_SCROLL_SPEED);
        loop {
            for i in (0..0x7e0).step_by(16) {
                for j in 0..7 {
                    let v = self.ram.read_u16(MESSAGE_PIXELS + i + (j + 1) * 2);
                    self.ram.write_u16(MESSAGE_PIXELS + i + j * 2, v)
                }
                let v = self.ram.read_u16(MESSAGE_PIXELS + i + 168 * 2);
                self.ram.write_u16(MESSAGE_PIXELS + i + 14, v)
            }
            for i in (0x34f..=0x3ef).step_by(8) {
                self.ram.write_u16(MESSAGE_PIXELS + i * 2, 0)
            }
            let v = self.ram.read_u8(0x1cdf).wrapping_add(1);
            self.ram.write_u8(0x1cdf, v);
            if v & 15 == 0 {
                self.ram.write_u16(VWF_CUR_LINE, 4);
                self.ram.write_u16(VWF_FLAG_NEXT_LINE, 1);
                return true;
            }
            if r == 0 {
                return false;
            }
            r = r.wrapping_sub(1)
        }
    }
    pub fn vwf_render_single(&mut self, c: u8) {
        if c != 0x59 {
            self.ram.write_u8(SOUND2, 12)
        }
        self.ram
            .write_u8(VWF_LINE_SPEED_CUR, self.ram.read_u8(VWF_LINE_SPEED));
        if self.ram.read_u16(VWF_FLAG_NEXT_LINE) != 0 {
            let i = usize::from(self.ram.read_u16(VWF_CUR_LINE) >> 1);
            self.ram.write_u16(VWF_LINE_PTR, [0, 0x2a0, 0x540][i]);
            self.ram.write_u16(VWF_VAR1, [0, 0x40, 0x80][i]);
            self.ram.write_u16(VWF_FLAG_NEXT_LINE, 0)
        }
        let i = usize::from(self.ram.read_u16(VWF_VAR1));
        let width = self
            .assets
            .font_widths
            .get(usize::from(c))
            .copied()
            .unwrap_or(0);
        assert!(width <= 8, "VWF glyph width exceeds one tile");
        let a = self.ram.read_u8(VWF_ARR + i);
        self.ram.write_u8(VWF_ARR + i + 1, a.wrapping_add(width));
        let glyph = usize::from((c & 0x70) * 2 + (c & 0xf));
        let bit_pos = usize::from(a) * 2;
        self.composite_glyph_half(
            glyph * 16,
            bit_pos,
            usize::from(self.ram.read_u16(VWF_LINE_PTR)),
            width,
        );
        self.composite_glyph_half(
            (glyph + 16) * 16,
            bit_pos,
            usize::from(self.ram.read_u16(VWF_LINE_PTR)) + 0x150,
            width,
        );
        self.ram.write_u16(VWF_VAR1, (i + 1) as u16);
    }

    fn composite_glyph_half(&mut self, source: usize, bit_pos: usize, line: usize, width: u8) {
        if width == 0 {
            return;
        }
        for row in (0..16).step_by(2) {
            let Some(bytes) = self.assets.font_tiles.get(source + row..source + row + 2) else {
                return;
            };
            let mut bits = u16::from_le_bytes([bytes[0], bytes[1]]);
            let y0 = bit_pos + line;
            let mut x = (y0 & 0xff0) + row;
            let mut y = (y0 >> 1) & 7;
            let mut left = width;
            loop {
                let mask = VWF_MASKS[y];
                let p0 = self.ram.read_u8(MESSAGE_PIXELS + x);
                let p1 = self.ram.read_u8(MESSAGE_PIXELS + x + 1);
                self.ram.write_u8(
                    MESSAGE_PIXELS + x,
                    if bits & 0x0080 != 0 {
                        p0 ^ mask
                    } else {
                        p0 & !mask
                    },
                );
                self.ram.write_u8(
                    MESSAGE_PIXELS + x + 1,
                    if bits & 0x8000 != 0 {
                        p1 ^ mask
                    } else {
                        p1 & !mask
                    },
                );
                bits = (bits & !0x8080) << 1;
                left -= 1;
                y += 1;
                if left == 0 || y == 8 {
                    break;
                }
            }
            x += 16;
            if bits != 0 {
                self.ram.write_u16(MESSAGE_PIXELS + x, bits)
            }
        }
    }
    pub fn init_vwf_state(&mut self) {
        for a in [VWF_CUR_LINE, VWF_FLAG_NEXT_LINE, VWF_VAR1, VWF_LINE_PTR] {
            self.ram.write_u16(a, 0)
        }
    }
    pub fn set_default_window_position(&mut self) {
        let y = self
            .ram
            .read_u16(LINK_Y)
            .wrapping_sub(self.ram.read_u16(BG2_VOFS2));
        self.ram
            .write_u16(TEXT_TOPLEFT, TEXT_POSITIONS[usize::from(y < 0x78)]);
    }
    pub fn initialize_state_loop(&mut self) {
        self.ram
            .slice_mut(TEXT_TOPLEFT_COPY, 32)
            .copy_from_slice(&TEXT_INITIALIZATION);
        self.init_vwf_state();
        self.set_default_window_position();
        self.ram.write_u16(TEXT_TILEMAP_CUR, 0x3980);
        self.load_character_buffer();
        self.ram.slice_mut(MESSAGE_PIXELS, 0x7e0).fill(0);
        self.ram.write_u8(NMI_SUBROUTINE, 2);
        self.ram.write_u8(NMI_DISABLE_CORE, 2);
    }
    pub fn initialize(&mut self) {
        if self.ram.read_u8(MAIN_MODULE) == 20 {
            self.runtime.reset_hud_palettes_4_and_5(self.ram)
        }
        self.runtime.decompress_story_gfx(self.ram);
        self.initialize_state_loop();
    }
    pub fn filter_player_name_character(mut a: u8) -> u8 {
        if a >= 0x5f {
            if a >= 0x76 {
                a = a.wrapping_sub(0x42)
            } else if a == 0x5f {
                a = 8
            } else if a == 0x60 {
                a = 0x22
            } else if a == 0x61 {
                a = 0x3e
            }
        }
        a
    }
    fn write_player_name(&mut self, dst: &mut Vec<u8>) {
        let slot = u16::from_le_bytes([self.sram[0x1ffe], self.sram[0x1fff]]);
        let offs = ((slot >> 1).wrapping_sub(1) as usize) * 0x500;
        let start = dst.len();
        for i in 0..6 {
            let p = 0x3d9 + offs + i * 2;
            let a = u16::from_le_bytes([self.sram[p], self.sram[p + 1]]);
            let character =
                Self::filter_player_name_character((a as u8 & 15) | ((a >> 1) as u8 & 0xf0));
            // C writes all six name bytes into WRAM before rewinding the
            // destination pointer past trailing padding characters.
            self.ram.write_u8(MESSAGE_TEXT + start + i, character);
            dst.push(character);
        }
        while dst.last() == Some(&0x59) {
            dst.pop();
        }
    }
    pub fn load_character_buffer(&mut self) {
        let msg = self.assets.messages[usize::from(self.ram.read_u16(DIALOGUE_INDEX))];
        let mut out = Vec::with_capacity(256);
        let mut i = 0;
        while i < msg.len() {
            let c = msg[i];
            i += 1;
            if c >= TEXT_DICTIONARY_BASE {
                out.extend_from_slice(
                    self.assets.dictionary[usize::from(c - TEXT_DICTIONARY_BASE)],
                );
                continue;
            }
            let next = msg.get(i).copied().unwrap_or(0);
            let cmd = decode_command(self.assets.dialogue_flags, c, next);
            match cmd.command() {
                CMD_NAME => self.write_player_name(&mut out),
                CMD_WINDOW => self.ram.write_u8(TEXT_RENDER_STATE, cmd.param()),
                CMD_NUMBER => {
                    let t = cmd.param();
                    let v = self.ram.read_u8(DIALOGUE_NUMBER + usize::from(t >> 1));
                    out.push(0x34 + if t & 1 != 0 { v >> 4 } else { v & 15 });
                }
                CMD_POSITION => self
                    .ram
                    .write_u16(TEXT_TOPLEFT, TEXT_POSITIONS[usize::from(cmd.param())]),
                CMD_COLOR => self.ram.write_u16(
                    TEXT_TILEMAP_CUR,
                    (0x387f & 0xe300) | 0x180 | (u16::from(cmd.param()) << 10) & 0x3c00,
                ),
                _ => {
                    out.push(c);
                    if cmd.multibyte() {
                        out.push(next)
                    }
                }
            }
            i += usize::from(cmd.multibyte());
        }
        out.push(0x7f);
        self.ram
            .slice_mut(MESSAGE_TEXT, out.len())
            .copy_from_slice(&out);
        self.ram.write_u16(DIALOGUE_READ_POS, 0);
    }
    pub fn draw_border_initialize(&mut self) {
        self.ram
            .write_u16(TEXT_TOPLEFT_COPY, self.ram.read_u16(TEXT_TOPLEFT));
    }
    pub fn draw_border_row(&mut self, word_offset: usize, y: usize) -> usize {
        let y = y >> 1;
        self.ram.write_u16(
            VRAM_UPLOAD_DATA + word_offset * 2,
            self.ram.read_u16(TEXT_TOPLEFT_COPY).swap_bytes(),
        );
        self.ram.write_u16(
            TEXT_TOPLEFT_COPY,
            self.ram.read_u16(TEXT_TOPLEFT_COPY).wrapping_add(0x20),
        );
        self.ram
            .write_u16(VRAM_UPLOAD_DATA + (word_offset + 1) * 2, 0x2f00);
        self.ram
            .write_u16(VRAM_UPLOAD_DATA + (word_offset + 2) * 2, BORDER_TILES[y]);
        for i in 0..22 {
            self.ram.write_u16(
                VRAM_UPLOAD_DATA + (word_offset + 3 + i) * 2,
                BORDER_TILES[y + 1],
            );
        }
        self.ram.write_u16(
            VRAM_UPLOAD_DATA + (word_offset + 25) * 2,
            BORDER_TILES[y + 2],
        );
        self.ram
            .write_u16(VRAM_UPLOAD_DATA + (word_offset + 26) * 2, 0xffff);
        word_offset + 26
    }
    pub fn build_character_tilemap(&mut self) {
        let mut t = self.ram.read_u16(TEXT_TILEMAP_CUR);
        for i in 0..126 {
            self.ram.write_u16(VWF_TILEMAP + i * 2, t);
            t = t.wrapping_add(1);
        }
        self.ram.write_u16(TEXT_TILEMAP_CUR, t);
        self.refresh();
    }
    pub fn refresh(&mut self) {
        self.draw_border_initialize();
        self.ram.write_u16(
            TEXT_TOPLEFT_COPY,
            self.ram.read_u16(TEXT_TOPLEFT_COPY).wrapping_add(0x21),
        );
        let mut d = 0usize;
        let mut s = 0usize;
        for _ in 0..6 {
            self.ram.write_u16(
                VRAM_UPLOAD_DATA + d * 2,
                self.ram.read_u16(TEXT_TOPLEFT_COPY).swap_bytes(),
            );
            self.ram.write_u16(
                TEXT_TOPLEFT_COPY,
                self.ram.read_u16(TEXT_TOPLEFT_COPY).wrapping_add(0x20),
            );
            self.ram.write_u16(VRAM_UPLOAD_DATA + (d + 1) * 2, 0x2900);
            for i in 0..21 {
                self.ram.write_u16(
                    VRAM_UPLOAD_DATA + (d + 2 + i) * 2,
                    self.ram.read_u16(VWF_TILEMAP + (s + i) * 2),
                );
            }
            d += 23;
            s += 21;
        }
        self.ram.write_u16(VRAM_UPLOAD_DATA + d * 2, 0xffff);
        self.ram.write_u8(NMI_LOAD_BG, 1);
    }
    pub fn choose_three(&mut self) {
        if self.ram.read_u8(TEXT_WAIT_COUNTDOWN2) != 0 {
            let n = self.ram.read_u8(TEXT_WAIT_COUNTDOWN2).wrapping_sub(1);
            self.ram.write_u8(TEXT_WAIT_COUNTDOWN2, n);
            if n == 1 {
                self.ram.write_u8(SOUND2, 36)
            }
            return;
        }
        let y = (self.ram.read_u8(0xf6) & 0xc0) | self.ram.read_u8(0xf4);
        if y & 0xd0 != 0 {
            self.ram.write_u8(SOUND1, 43);
            self.ram.write_u8(TEXT_RENDER_STATE, 4)
        } else if y & 12 != 0 {
            let old = self.ram.read_u8(CHOICE);
            let choice = if y & 8 != 0 {
                if old == 0 { 2 } else { old - 1 }
            } else if old == 2 {
                0
            } else {
                old + 1
            };
            self.ram.write_u8(CHOICE, choice);
            self.ram.write_u8(SOUND2, 32);
            self.ram.write_u16(DIALOGUE_INDEX, u16::from(choice) + 6);
            self.load_character_buffer();
            self.init_vwf_state();
        }
    }

    /// `SaveGameFile`: copy the live 0x500-byte save block to both SRAM banks
    /// and reproduce the game's subtractive checksum.
    pub fn save_game_file(&mut self) {
        let slot = u16::from_le_bytes([self.sram[0x1ffe], self.sram[0x1fff]]);
        let offset = usize::from((slot >> 1).wrapping_sub(1)) * 0x500;
        let mut block = [0u8; 0x500];
        block.copy_from_slice(self.ram.slice(0xf000, 0x500));
        self.sram[offset..offset + 0x500].copy_from_slice(&block);
        self.sram[offset + 0xf00..offset + 0x1400].copy_from_slice(&block);
        let mut checksum = 0x5a5au16;
        for word in block[..0x4fe].chunks_exact(2) {
            checksum = checksum.wrapping_sub(u16::from_le_bytes([word[0], word[1]]));
        }
        self.ram.write_u16(0xf4fe, checksum);
        let bytes = checksum.to_le_bytes();
        self.sram[offset + 0x4fe..offset + 0x500].copy_from_slice(&bytes);
        self.sram[offset + 0x13fe..offset + 0x1400].copy_from_slice(&bytes);
    }

    pub fn copy_save_to_wram(&mut self, offset: usize) {
        for address in [0x1abf, 0x1acf, 0x1adf, 0x1aef, 0x1aff] {
            self.ram.write_u8(address, 0);
        }
        self.ram
            .slice_mut(0xf000, 0x500)
            .copy_from_slice(&self.sram[offset..offset + 0x500]);
        self.ram.write_u16(0xc00d, 7);
        self.ram.write_u16(0xc013, 7);
        self.ram.write_u16(0xc00f, 0);
        self.ram.write_u16(0xc015, 0);
        self.ram.write_u16(0x219, 0x6040);
        self.ram.write_u16(0x21d, 0x4841);
        self.ram.write_u16(0x21f, 0x7f);
        self.ram.write_u16(0x221, 0xffff);
        if self.runtime.misc_bugfixes_enabled() {
            self.ram.write_u8(0xc011, 0);
        }
        self.ram.write_u8(0x204, 0x80);
        self.ram.write_u8(0x10, 5);
        self.ram.write_u8(0x11, 0);
        self.ram.write_u8(0x10e, 0);
        self.ram.write_u8(0x710, 0);
        self.ram.write_u8(0xab2, 0);
    }

    pub fn red_potion(&mut self) {
        if self.runtime.hud_refill_health(self.ram) {
            self.finish_potion()
        }
    }
    pub fn green_potion(&mut self) {
        if self.runtime.hud_refill_magic(self.ram) {
            self.finish_potion()
        }
    }
    pub fn blue_potion(&mut self) {
        if self.runtime.hud_refill_health(self.ram) {
            self.ram.write_u8(SUBMODULE, 8)
        }
        if self.runtime.hud_refill_magic(self.ram) {
            self.ram.write_u8(SUBMODULE, 4)
        }
    }
    fn finish_potion(&mut self) {
        self.ram
            .write_u8(BUTTON_MASK_BY, self.ram.read_u8(BUTTON_MASK_BY) & !0x40);
        self.ram
            .write_u8(HUD_UPDATE, self.ram.read_u8(HUD_UPDATE).wrapping_add(1));
        self.ram.write_u8(SUBMODULE, 0);
        self.ram
            .write_u8(MAIN_MODULE, self.ram.read_u8(SAVED_MODULE));
    }

    pub fn run_submodule(&mut self) {
        match self.ram.read_u8(SUBMODULE) {
            1 => self.runtime.run_hud_module(self.ram),
            2 => self.render_text(),
            3 => self.dungeon_map(),
            4 => self.red_potion(),
            5 => self.module0e_05_desert_prayer(),
            7 => self.overworld_map(),
            8 => self.green_potion(),
            9 => self.blue_potion(),
            10 => self.flute_menu(),
            11 => self.module0e_0b_save_menu(),
            _ => {}
        }
    }

    pub fn dungeon_map(&mut self) {
        let state = self.ram.read_u8(OW_MAP_STATE);
        match state {
            0 => self.dungeon_map_backup(),
            1 => self.dungeon_map_draw(),
            2 => self.dungeon_map_lighten(),
            3 => self.dungeon_map_handle_input_and_sprites(),
            4 => self.dungeon_map_slide(),
            5 => self.dungeon_map_fade(),
            6 | 7 => self.runtime.dungeon_map_step(self.ram, state),
            8 => self.dungeon_map_restore(),
            _ => {}
        }
    }
    fn palace(&self) -> usize {
        usize::from(self.ram.read_u16(0x40c) >> 1)
    }
    pub fn dungeon_map_draw(&mut self) {
        match self.ram.read_u8(0x20d) {
            0 => self.runtime.dungeon_map_prepare_graphics(self.ram),
            1 => self.dungeon_map_draw_level(),
            2 => self.dungeon_map_draw_floors_backdrop(),
            3 => self.dungeon_map_draw_rooms(),
            4 => self.dungeon_map_draw_room_markers(),
            _ => {}
        }
    }

    fn dungeon_map_draw_rooms(&mut self) {
        self.ram.write_u8(0x210, 0);
        self.ram.write_u16(0x211, 0);
        let lowest_floor = 0u8.wrapping_sub((DUNG_TAB5[self.palace()] & 0xf) as u8);
        let dungeon_floor = self.ram.read_u8(0xa4);
        if dungeon_floor != lowest_floor {
            self.ram.write_u16(0x20e, u16::from(dungeon_floor));
        } else {
            self.ram
                .write_u16(0x20e, u16::from(dungeon_floor).wrapping_add(1));
            self.ram.write_u16(0x211, 2);
        }
        self.dungeon_map_draw_floor_numbers(0, !0x1000);
        self.dungeon_map_draw_border(0, !0x1000);
        self.dungeon_map_draw_layout(0);
        self.ram
            .write_u8(0x20e, self.ram.read_u8(0x20e).wrapping_sub(1));
        self.dungeon_map_draw_floor_numbers(0x300, !0x1000);
        self.dungeon_map_draw_border(0x300, !0x1000);
        self.dungeon_map_draw_layout(0x300);
        self.ram
            .write_u16(0x20e, self.ram.read_u16(0x20e).wrapping_add(1));
        self.ram.write_u16(6, 0);
        self.ram.write_u16(10, 0);
        self.ram.write_u8(NMI_SUBROUTINE, 8);
        self.ram.write_u8(0x116, 0x22);
        self.ram
            .write_u8(0x20d, self.ram.read_u8(0x20d).wrapping_add(1));
    }

    fn dungeon_map_draw_layout(&mut self, pd: u16) {
        for row in 0..5 {
            let offset = usize::from((292 + 128 * row as u16 + pd) & 0xfff) >> 1;
            self.dungeon_map_draw_row(row, offset);
        }
    }

    fn dungeon_map_draw_row(&mut self, row: usize, mut offset: usize) {
        let palace = self.palace();
        let t5 = DUNG_TAB5[palace];
        let mask = UPPER_BITMASKS[palace];
        let floor = usize::from(self.ram.read_u8(0x20e).wrapping_add((t5 & 0xf) as u8));
        let layout =
            z3_core::assets::packed_array_entry(self.assets.dungeon_map_floor_layout, palace)
                .expect("dungeon map floor layout");
        for column in 0..5 {
            let room = layout[floor * 25 + row * 5 + column];
            let (room_flags, info) = if room == 0xf {
                (0, 0x51)
            } else {
                let flags = self.ram.read_u16(0xf000 + usize::from(room) * 2) as u8 & 0xf;
                let mut count = 0usize;
                let mut index = 0usize;
                while layout[index] != room {
                    count += usize::from(layout[index] != 0xf);
                    index += 1;
                }
                (
                    flags,
                    usize::from(
                        self.get_other_dungmap_info(count)
                            .expect("dungeon map tile info"),
                    ),
                )
            };
            let tiles = Self::dungeon_room_quadrants(
                info as u8,
                room_flags,
                self.ram.read_u16(0xf368) & mask != 0,
            );
            let address = MESSAGE_PIXELS + offset * 2;
            self.ram.write_u16(address, tiles[0]);
            self.ram.write_u16(address + 2, tiles[1]);
            self.ram.write_u16(address + 0x40, tiles[2]);
            self.ram.write_u16(address + 0x42, tiles[3]);
            offset += 2;
        }
    }
    pub fn dungeon_map_draw_level(&mut self) {
        let i = DUNG_TAB0[self.palace()] >> 1;
        if i >= 0 {
            let i = i as usize;
            self.ram.write_u8(VRAM_UPLOAD_DATA + 32, 0xff);
            self.ram.write_u16(VRAM_UPLOAD_DATA + 14, DUNG_TAB1[i]);
            self.ram.write_u16(VRAM_UPLOAD_DATA + 30, DUNG_TAB2[i]);
            for j in (0..14).rev() {
                self.ram.write_u8(VRAM_UPLOAD_DATA + j, DUNG_TAB3[j]);
                self.ram.write_u8(VRAM_UPLOAD_DATA + j + 16, DUNG_TAB4[j]);
            }
            self.ram.write_u8(NMI_LOAD_BG, 1);
        }
        self.ram
            .write_u8(0x20d, self.ram.read_u8(0x20d).wrapping_add(1));
    }
    pub fn dungeon_map_draw_floors_backdrop(&mut self) {
        let t5 = DUNG_TAB5[self.palace()];
        let mut offs = 0usize;
        if t5 & 0x100 != 0 {
            for value in DUNG_TAB6 {
                self.ram.write_u16(VRAM_UPLOAD_DATA + offs * 2, value);
                offs += 1;
            }
            let mut address = 0x1123u16;
            for _ in 0..16 {
                self.ram
                    .write_u16(VRAM_UPLOAD_DATA + offs * 2, address.swap_bytes());
                self.ram.write_u16(VRAM_UPLOAD_DATA + (offs + 1) * 2, 0xe40);
                self.ram
                    .write_u16(VRAM_UPLOAD_DATA + (offs + 2) * 2, 0x1b2e);
                address = address.wrapping_add(0x20);
                offs += 3;
            }
        }
        let low = t5 as u8;
        let ix = if low >= 0x50 {
            usize::from((low >> 4) - 4)
        } else if t5 & 0xf >= 5 {
            usize::from(t5 & 0xf)
        } else {
            0
        };
        let org = DUNG_TAB7[ix];
        let mut address = org;
        let mut j = 0usize;
        loop {
            self.ram
                .write_u16(VRAM_UPLOAD_DATA + offs * 2, address.swap_bytes());
            self.ram.write_u16(VRAM_UPLOAD_DATA + (offs + 1) * 2, 0xe40);
            self.ram.write_u16(
                VRAM_UPLOAD_DATA + (offs + 2) * 2,
                DUNG_TAB8[j] + if t5 & 0x200 != 0 { 0x400 } else { 0 },
            );
            offs += 3;
            if j != 6 {
                j += 1
            }
            address = address.wrapping_add(0x20);
            if address >= 0x1360 {
                break;
            }
        }
        self.ram.write_u16(0x1000, (offs * 2) as u16);
        offs = self.dungeon_map_build_floor_boxes(t5 as u8, org, offs);
        self.ram.write_u8(VRAM_UPLOAD_DATA + offs * 2, 0xff);
        self.ram
            .write_u8(0x20d, self.ram.read_u8(0x20d).wrapping_add(1));
        self.ram.write_u8(NMI_LOAD_BG, 1);
    }
    fn dungeon_map_build_floor_boxes(
        &mut self,
        t5: u8,
        mut address: u16,
        mut offs: usize,
    ) -> usize {
        let n = usize::from((t5 & 0xf) + (t5 >> 4));
        address = address
            .wrapping_sub(0x3e)
            .wrapping_add(u16::from(t5 & 0xf) * 0x40);
        for _ in 0..n {
            for half in 0..2 {
                self.ram
                    .write_u16(VRAM_UPLOAD_DATA + offs * 2, address.swap_bytes());
                self.ram.write_u16(VRAM_UPLOAD_DATA + (offs + 1) * 2, 0x700);
                offs += 2;
                for &tile in DUNG_TAB9.iter().skip(half * 4).take(4) {
                    self.ram.write_u16(VRAM_UPLOAD_DATA + offs * 2, tile);
                    offs += 1;
                }
                if half == 0 {
                    address = address.wrapping_add(0x20)
                }
            }
            address = address.wrapping_sub(0x60);
        }
        self.ram.write_u16(0x1000, (offs * 2) as u16);
        offs
    }
    pub fn dungeon_map_draw_border(&mut self, pd: u16, mask: u16) {
        for i in 0..4 {
            let p = usize::from((DUNG_TAB10[i] + pd) & 0xfff);
            self.ram.write_u16(MESSAGE_PIXELS + p, DUNG_TAB11[i] & mask);
        }
        for i in 0..2 {
            for j in (0..20).step_by(2) {
                let p = usize::from((DUNG_TAB12[i] + pd + j) & 0xfff);
                self.ram.write_u16(MESSAGE_PIXELS + p, DUNG_TAB13[i] & mask);
            }
        }
        for i in 0..2 {
            for j in (0..0x280).step_by(0x40) {
                let p = usize::from((DUNG_TAB14[i] + pd + j) & 0xfff);
                self.ram.write_u16(MESSAGE_PIXELS + p, DUNG_TAB15[i] & mask);
            }
        }
    }
    pub fn dungeon_map_draw_floor_numbers(&mut self, pd: u16, mask: u16) {
        let mut p = 0xdeu16;
        loop {
            let q = usize::from((p + pd) & 0xfff);
            self.ram.write_u16(MESSAGE_PIXELS + q, 0xf00);
            self.ram.write_u16(MESSAGE_PIXELS + q + 2, 0xf00);
            p += 0x40;
            if p == 0x39e {
                break;
            }
        }
        let q = usize::from((0x35e + pd) & 0xfff);
        let floor = self.ram.read_u8(0x20e);
        let a = if floor & 0x80 != 0 {
            0x1f1c
        } else {
            DUNG_TAB16[usize::from(floor & 0xf)]
        };
        let b = if floor & 0x80 != 0 {
            DUNG_TAB16[usize::from(!floor & 0xf)]
        } else {
            0x1f1d
        };
        self.ram.write_u16(MESSAGE_PIXELS + q, a & mask);
        self.ram.write_u16(MESSAGE_PIXELS + q + 2, b & mask);
    }
    #[must_use]
    pub fn dungeon_room_quadrants(yv: u8, room_flags: u8, has_map: bool) -> [u16; 4] {
        let mut out = [0; 4];
        for quadrant in 0..4 {
            let original = DUNG_TAB23[usize::from(yv) * 4 + quadrant];
            let door_bit = 8 >> quadrant;
            let modifier = if original != 0xb00 && room_flags & door_bit == 0 {
                if original & 0x1000 == 0 {
                    0x400
                } else if has_map {
                    out[quadrant] = (original & !0x1c00) | 0xc00;
                    continue;
                } else {
                    0
                }
            } else {
                0
            };
            out[quadrant] = if has_map || room_flags & door_bit != 0 {
                original.wrapping_add(modifier)
            } else {
                0xb00
            };
        }
        out
    }
    pub fn dungeon_map_backup(&mut self) {
        let a = self.ram.read_u8(INIDISP).wrapping_sub(1);
        self.ram.write_u8(INIDISP, a);
        if a != 0 {
            return;
        }
        self.ram.write_u8(MOSAIC, 3);
        self.ram.write_u8(MAPBAK_HDMAEN, self.ram.read_u8(HDMAEN));
        self.ram.write_u8(INIDISP, 0x80);
        self.ram.write_u8(HDMAEN, 0);
        self.ram.write_u8(OW_MAP_STATE, 1);
        self.ram.write_u8(0x20d, 0);
        self.ram.write_u8(0x9c, 0x20);
        self.ram.write_u8(0x9d, 0x40);
        self.ram.write_u8(0x9e, 0x80);
        self.ram.write_u16(0x100, 0x250);
        for i in 0..512 {
            self.ram.write_u8(0x1dd80 + i, self.ram.read_u8(0xc500 + i));
        }
        self.ram.write_u16(MAPBAK_BG1_X, self.ram.read_u16(0x11a));
        self.ram.write_u16(MAPBAK_BG1_Y, self.ram.read_u16(0x11c));
        self.ram.write_u16(0x11a, 0);
        self.ram.write_u16(0x11c, 0);
        for (d, s) in [
            (MAPBAK_BG1_HOFS2, BG1_HOFS2),
            (MAPBAK_BG2_HOFS2, 0xe2),
            (MAPBAK_BG1_VOFS2, BG1_VOFS2),
            (MAPBAK_BG2_VOFS2, BG2_VOFS2),
        ] {
            self.ram.write_u16(d, self.ram.read_u16(s));
            self.ram.write_u16(s, 0);
        }
        self.ram.write_u16(0xe4, 0);
        self.ram.write_u16(0xea, 0);
        self.ram.write_u16(MAPBAK_CGWSEL, self.ram.read_u16(0x99));
        self.ram.write_u8(0x99, 2);
        self.ram.write_u8(0x9a, 0x20);
        for i in 0..2048 {
            self.ram.write_u16(MESSAGE_PIXELS + i * 2, 0x300);
        }
        self.ram.write_u8(SOUND2, 16);
        self.ram.write_u8(0x12c, 0xf2);
    }
    pub fn dungeon_map_lighten(&mut self) {
        let a = self.ram.read_u8(INIDISP).wrapping_add(1);
        self.ram.write_u8(INIDISP, a);
        if a == 15 {
            self.ram.write_u8(OW_MAP_STATE, 3)
        }
    }
    pub fn dungeon_map_handle_input(&mut self) {
        if self.did_press_button_for_map() {
            self.ram
                .write_u8(OW_MAP_STATE, self.ram.read_u8(OW_MAP_STATE).wrapping_add(2));
            self.ram.write_u8(0x20d, 0);
            return;
        }
        self.dungeon_map_floor_select();
        if self.ram.read_u8(0x210) != 0 {
            self.dungeon_map_scroll_floors()
        }
    }
    pub fn dungeon_map_floor_select(&mut self) {
        let t5 = DUNG_TAB5[self.palace()];
        let up = (t5 >> 4) as u8 & 15;
        let down = t5 as u8 & 15;
        if up.wrapping_add(down) < 3
            || self.ram.read_u8(0x210) != 0
            || self.ram.read_u8(JOYPAD_H) & 12 == 0
        {
            return;
        }
        let input = self.ram.read_u8(JOYPAD_H);
        let mut floor = self.ram.read_u8(0x20e);
        let mut r6 = self.ram.read_u16(6);
        if input & 8 != 0 {
            if up.wrapping_sub(1) == floor {
                return;
            }
            floor = floor.wrapping_add(1);
            r6 = r6.wrapping_sub(0x300) & 0xfff;
        } else {
            if (0u8.wrapping_sub(down).wrapping_add(1)) == floor {
                return;
            }
            floor = floor.wrapping_sub(2);
            r6 = r6.wrapping_add(0x600) & 0xfff;
        }
        self.ram.write_u8(0x20e, floor);
        self.dungeon_map_draw_floor_numbers(r6, !0x1000);
        self.dungeon_map_draw_border(r6, !0x1000);
        self.dungeon_map_draw_layout(r6);
        self.ram
            .write_u8(0x210, self.ram.read_u8(0x210).wrapping_add(1));
        self.ram.write_u16(10, u16::from(input));
        let x = usize::from((input >> 3) & 1);
        self.ram.write_u16(
            0x213,
            self.ram
                .read_u16(BG2_VOFS2)
                .wrapping_add([0x60u16, 0xffa0][x]),
        );
        if x == 0 {
            r6 = r6.wrapping_sub(0x300) & 0xfff;
            self.ram
                .write_u8(0x20e, self.ram.read_u8(0x20e).wrapping_add(1));
        }
        self.ram.write_u16(6, r6);
        self.ram.write_u8(NMI_SUBROUTINE, 8);
    }
    pub fn dungeon_map_scroll_floors(&mut self) {
        let x = usize::from((self.ram.read_u16(10) >> 3) & 1);
        let delta = [-4i16, 4][x];
        let scroll = [4i16, -4][x];
        self.ram
            .write_i16(0x217, self.ram.read_i16(0x217).wrapping_add(delta));
        self.ram
            .write_i16(0xfaa, self.ram.read_i16(0xfaa).wrapping_add(delta));
        self.ram
            .write_i16(BG2_VOFS2, self.ram.read_i16(BG2_VOFS2).wrapping_add(scroll));
        if self.ram.read_u16(BG2_VOFS2) == self.ram.read_u16(0x213) {
            self.ram.write_u8(0x210, 0)
        }
    }
    pub fn dungeon_map_slide(&mut self) {
        let d = self.ram.read_u16(0x213);
        self.ram
            .write_u16(BG2_VOFS2, self.ram.read_u16(BG2_VOFS2).wrapping_add(d));
        self.ram
            .write_u16(0x217, self.ram.read_u16(0x217).wrapping_sub(d));
        let n = self.ram.read_u8(0x205).wrapping_sub(1);
        self.ram.write_u8(0x205, n);
        if n == 0 {
            self.ram
                .write_u8(OW_MAP_STATE, self.ram.read_u8(OW_MAP_STATE).wrapping_sub(1))
        }
    }
    pub fn dungeon_map_fade(&mut self) {
        let a = self.ram.read_u8(INIDISP).wrapping_sub(1);
        self.ram.write_u8(INIDISP, a);
        if a != 0 {
            return;
        }
        self.ram.write_u8(INIDISP, 0x80);
        self.ram.write_u8(HDMAEN, 0);
        self.ram
            .write_u8(OW_MAP_STATE, self.ram.read_u8(OW_MAP_STATE).wrapping_add(1));
        self.ram.write_u16(0x99, self.ram.read_u16(MAPBAK_CGWSEL));
        for (d, s) in [
            (BG1_HOFS2, MAPBAK_BG1_HOFS2),
            (0xe2, MAPBAK_BG2_HOFS2),
            (BG1_VOFS2, MAPBAK_BG1_VOFS2),
            (BG2_VOFS2, MAPBAK_BG2_VOFS2),
        ] {
            self.ram.write_u16(d, self.ram.read_u16(s));
        }
        self.ram.write_u16(0x11a, self.ram.read_u16(MAPBAK_BG1_X));
        self.ram.write_u16(0x11c, self.ram.read_u16(MAPBAK_BG1_Y));
        self.ram
            .write_u8(0x15, self.ram.read_u8(0x15).wrapping_add(1));
    }
    pub fn dungeon_map_restore(&mut self) {
        let a = self.ram.read_u8(INIDISP).wrapping_add(1);
        self.ram.write_u8(INIDISP, a);
        if a != 15 {
            return;
        }
        self.ram
            .write_u8(MAIN_MODULE, self.ram.read_u8(SAVED_MODULE));
        self.ram.write_u8(SUBMODULE, 0);
        self.ram.write_u8(OW_MAP_STATE, 0);
        self.ram.write_u8(SUBSUBMODULE, 0);
        self.ram.write_u8(HDMAEN, self.ram.read_u8(MAPBAK_HDMAEN));
    }
    pub fn dungeon_map_draw_floor_number_objects(&mut self, mut sprite: usize) -> usize {
        let t5 = DUNG_TAB5[self.palace()];
        let mut above = (t5 >> 4) as u8 & 15;
        let below = t5 as u8 & 15;
        let mut row = 7usize;
        if above.wrapping_add(below) != 8 && above < 4 {
            row = 6;
            let mut i = 3u8;
            while i != 0 && i != above {
                row -= 1;
                i -= 1
            }
            if below >= 5 {
                let mut i = 5u8;
                while i != below && below != 8 {
                    row += 1;
                    i += 1
                }
            }
        }
        let mut y = DUNG_TAB33[row].wrapping_add(1);
        above = above.wrapping_sub(1);
        let end = 0u8.wrapping_sub(below);
        loop {
            let (left, right) = if above & 0x80 != 0 {
                (0x1c, DUNG_TAB34[usize::from(above ^ 0xff)])
            } else {
                (DUNG_TAB34[usize::from(above)], 0x1d)
            };
            crate::oam::set_plain(self.ram, sprite, 0x30, y, left, 0x3d, 0);
            crate::oam::set_plain(self.ram, sprite + 1, 0x38, y, right, 0x3d, 0);
            sprite += 2;
            y = y.wrapping_add(16);
            let old = above;
            above = above.wrapping_sub(1);
            if old == end {
                break;
            }
        }
        sprite
    }
    pub fn module_game_over(&mut self) {
        let state = self.ram.read_u8(SUBMODULE);
        match state {
            0 => {
                self.ram.write_u8(SUBMODULE, 1);
                self.death_initialize()
            }
            1 => self.death_initialize(),
            2 => self.game_over_delay_before_iris(),
            3 => self.game_over_iris_wipe(),
            4 => self.death_player_swoon(),
            5 => self.game_over_splat_and_fade(),
            6 => self.death_load_game_over_gfx(),
            7 => self.animate_game_over_letters(),
            8 => self.game_over_finalize_text(),
            9 => self.game_over_save_or_continue(),
            10 => self.runtime.death_step(self.ram, state),
            11 => self.runtime.death_step(self.ram, 11),
            12 => self.game_over_rise_a_little(),
            13 => self.game_over_restore_0d(),
            14 => self.runtime.death_step(self.ram, 14),
            15 => self.game_over_resituate_link(),
            _ => {}
        }
        if self.ram.read_u8(SUBMODULE) != 9 {
            self.runtime.link_oam_main(self.ram);
        }
    }
    pub fn death_initialize(&mut self) {
        self.death_func1();
    }
    pub fn game_over_delay_before_iris(&mut self) {
        let n = self.ram.read_u8(SOME_MENU_CTR).wrapping_sub(1);
        self.ram.write_u8(SOME_MENU_CTR, n);
        if n != 0 {
            return;
        }
        self.death_initialize_game_over_letters();
        self.runtime.death_step(self.ram, 2);
        self.ram.write_u8(0x98, 0x30);
        self.ram.write_u8(0x97, 0);
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
    }
    pub fn game_over_iris_wipe(&mut self) {
        self.runtime.death_step(self.ram, 3)
    }
    pub fn game_over_splat_and_fade(&mut self) {
        let n = self.ram.read_u8(SOME_MENU_CTR);
        if n != 0 {
            self.ram.write_u8(SOME_MENU_CTR, n - 1);
            return;
        }
        self.runtime.death_step(self.ram, 5);
    }
    pub fn death_load_game_over_gfx(&mut self) {
        self.ram.write_u8(SOME_MENU_CTR, 12);
        self.runtime.death_step(self.ram, 6);
        self.ram.write_u8(SUBMODULE, 7);
        self.death_player_swoon();
    }
    pub fn game_over_finalize_text(&mut self) {
        self.animate_game_over_letters();
        let main = self.ram.read_u8(MAIN_MODULE);
        let sub = self.ram.read_u8(SUBMODULE);
        self.ram.write_u8(MESSAGING_MODULE, 2);
        self.render_text();
        self.ram.write_u8(SUBMODULE, sub.wrapping_add(1));
        self.ram.write_u8(MAIN_MODULE, main);
        self.ram.write_u8(SOME_MENU_CTR, 2);
        self.ram.write_u8(0x12c, 11);
    }
    pub fn game_over_rise_a_little(&mut self) {
        self.runtime.death_step(self.ram, 12);
    }
    pub fn game_over_restore_0d(&mut self) {
        if self.ram.read_u8(IS_HEART_ANIM) == 0 {
            self.runtime.death_step(self.ram, 13);
        } else {
            self.runtime.death_step(self.ram, 0x31);
        }
    }
    pub fn game_over_resituate_link(&mut self) {
        self.runtime.death_step(self.ram, 15);
        if self.ram.read_u8(0xc007) == 32 {
            self.ram
                .write_u8(MAIN_MODULE, self.ram.read_u8(SAVED_MODULE));
            self.ram.write_u8(SUBMODULE, 0);
            self.ram.write_u8(0x31f, 144);
        }
    }
    pub fn death_initialize_game_over_letters(&mut self) {
        self.ram.write_u8(0x35f, 0);
        for i in 0..8 {
            self.ram.write_u8(0xc04 + i, 0xb0);
            self.ram.write_u8(0xc18 + i, 0);
        }
        self.ram.write_u8(0xc4a, 1);
        self.ram.write_u8(0x39d, 6);
    }
    pub fn animate_game_over_letters(&mut self) {
        match self.ram.read_u8(0xc4a) {
            0 => self
                .ram
                .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1)),
            1 => self.game_over_sweep_left(),
            2 => self.game_over_unfurl_right(),
            _ => self.runtime.death_step(self.ram, 0x30),
        }
    }
    fn ancilla_move_x(&mut self, k: usize) {
        let velocity = self.ram.read_i8(0xc2c + k) as i16;
        let frac = self.ram.read_u8(0xc40 + k);
        let sum = u16::from(frac).wrapping_add((velocity as u16) << 4);
        self.ram.write_u8(0xc40 + k, sum as u8);
        let x = u16::from_le_bytes([self.ram.read_u8(0xc04 + k), self.ram.read_u8(0xc18 + k)]);
        let x = x.wrapping_add(((sum as i16) >> 8) as u16);
        self.ram.write_u8(0xc04 + k, x as u8);
        self.ram.write_u8(0xc18 + k, (x >> 8) as u8);
    }
    pub fn game_over_sweep_left(&mut self) {
        const TARGET: [u8; 8] = [0x40, 0x50, 0x60, 0x70, 0x88, 0x98, 0xa8, 0x40];
        let mut k = usize::from(self.ram.read_u8(0x35f));
        self.ram.write_u8(0xc2c + k, 0x80);
        self.ancilla_move_x(k);
        if self.ram.read_u8(0xc04 + k) < TARGET[k] {
            self.ram.write_u8(0xc04 + k, TARGET[k]);
            k += 1;
            self.ram.write_u8(0x35f, k as u8);
            if k == 8 {
                self.ram.write_u8(0x35f, 7);
                self.ram.write_u8(0xc4a, 2);
                self.ram.write_u8(0x39d, 0);
                self.ram.write_u8(SOUND2, 38);
                return;
            }
        }
        if k == 7 {
            let end = usize::from(self.ram.read_u8(0x39d));
            for j in (end + 1..7).rev() {
                self.ram.write_u8(0xc04 + j, self.ram.read_u8(0xc04 + 7));
            }
            if self.ram.read_u8(0xc04 + 7) < TARGET[end] {
                self.ram.write_u8(0x39d, end.wrapping_sub(1) as u8)
            }
        }
        self.runtime.death_step(self.ram, 0x30);
    }
    pub fn game_over_unfurl_right(&mut self) {
        const TARGET: [u8; 8] = [0x58, 0x60, 0x68, 0x70, 0x88, 0x90, 0x98, 0xa0];
        let k = usize::from(self.ram.read_u8(0x35f));
        self.ram.write_u8(0xc2c + k, 0x60);
        self.ancilla_move_x(k);
        let mut end = usize::from(self.ram.read_u8(0x39d));
        if self.ram.read_u8(0xc04 + k) >= TARGET[end] {
            self.ram.write_u8(0xc04 + end, TARGET[end]);
            end += 1;
            self.ram.write_u8(0x39d, end as u8);
            if end == 8 {
                self.ram
                    .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
                self.ram.write_u8(0xc4a, 3);
                return;
            }
        }
        for j in end..=k {
            self.ram.write_u8(0xc04 + j, self.ram.read_u8(0xc04 + k));
        }
        self.runtime.death_step(self.ram, 0x30);
    }
    pub fn death_player_swoon(&mut self) {
        const FRAME: [u8; 15] = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 5];
        const DELAY: [u8; 15] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 0x62];
        let timer = self.ram.read_u8(0x30b).wrapping_sub(1);
        self.ram.write_u8(0x30b, timer);
        let mut k = usize::from(self.ram.read_u8(0x30d));
        if timer & 0x80 != 0 {
            k += 1;
            if k == 15 {
                return;
            }
            if k == 14 {
                self.ram
                    .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
            }
            self.ram.write_u8(0x30d, k as u8);
            self.ram.write_u8(0x30a, FRAME[k]);
            self.ram.write_u8(0x30b, DELAY[k]);
        }
        if k == 13 && self.ram.read_u8(0x4b) != 12 {
            let y = self
                .ram
                .read_u16(LINK_Y)
                .wrapping_add(16)
                .wrapping_sub(self.ram.read_u16(BG2_VOFS2));
            let x = self
                .ram
                .read_u16(0x22)
                .wrapping_add(7)
                .wrapping_sub(self.ram.read_u16(0xe2));
            let flags = [0x20, 0x10][usize::from(self.ram.read_u8(0xee) & 1)] | 2;
            crate::oam::set_plain(self.ram, 0x74, x as u8, y as u8, 0xaa, flags, 2);
        }
    }

    pub fn game_over_save_or_continue(&mut self) {
        let mut timer = self.ram.read_u8(SOME_MENU_CTR);
        if self.ram.read_u8(0xf4) & 0x20 != 0 {
            self.ram
                .write_u8(SUBSUBMODULE, (self.ram.read_u8(SUBSUBMODULE) + 1) % 3);
        } else {
            timer = timer.wrapping_sub(1);
            if timer == 0 {
                timer = 1;
                let input = self.ram.read_u8(JOYPAD_H) & 12;
                if input != 0 {
                    let choice = self.ram.read_u8(SUBSUBMODULE);
                    self.ram.write_u8(
                        SUBSUBMODULE,
                        if input & 4 != 0 {
                            (choice + 1) % 3
                        } else {
                            if choice == 0 { 2 } else { choice - 1 }
                        },
                    );
                    timer = 12;
                    self.ram.write_u8(SOUND2, 32);
                }
            }
        }
        self.ram.write_u8(SOME_MENU_CTR, timer);
    }

    pub fn overworld_map(&mut self) {
        match self.ram.read_u8(OW_MAP_STATE) {
            0 => self.world_map_fade_out(),
            1 => self.runtime.world_map_load_light(self.ram),
            2 => self.runtime.world_map_load_dark(self.ram),
            3 => self.runtime.world_map_load_sprite_gfx(self.ram),
            4 => self.world_map_brighten(),
            5 => self.world_map_player_control(),
            6 => self.runtime.world_map_restore_gfx(self.ram),
            7 => self.runtime.world_map_exit(self.ram),
            _ => {}
        }
    }
    pub fn world_map_fade_out(&mut self) {
        let brightness = self.ram.read_u8(INIDISP).wrapping_sub(1);
        self.ram.write_u8(INIDISP, brightness);
        if brightness != 0 {
            return;
        }
        self.ram.write_u8(MAPBAK_HDMAEN, self.ram.read_u8(HDMAEN));
        self.runtime.enable_force_blank(self.ram);
        self.ram.write_u8(MOSAIC, 3);
        self.ram.write_u8(OW_MAP_STATE, 1);
        self.ram.write_u16(MAPBAK_TM, self.ram.read_u16(TM_COPY));
        for (dst, src) in [
            (MAPBAK_BG1_HOFS2, BG1_HOFS2),
            (MAPBAK_BG2_HOFS2, 0xe2),
            (MAPBAK_BG1_VOFS2, BG1_VOFS2),
            (MAPBAK_BG2_VOFS2, BG2_VOFS2),
        ] {
            self.ram.write_u16(dst, self.ram.read_u16(src));
        }
        for address in [BG1_HOFS2, 0xe2, 0xe4, BG1_VOFS2, BG2_VOFS2, 0xea] {
            self.ram.write_u16(address, 0)
        }
        self.ram.write_u16(MAPBAK_CGWSEL, self.ram.read_u16(0x99));
        self.ram.write_u16(0x100, 0x1fc);
        if self.ram.read_u8(0x8a) < 0x80 {
            self.ram.write_u16(0xc108, self.ram.read_u16(0x20));
            self.ram.write_u16(0xc10a, self.ram.read_u16(0x22));
        }
        if self.ram.read_u8(0xf3c5) < 2 {
            self.ram.write_u8(0x99, 0x80);
            self.ram.write_u8(0x9a, 0x61);
        }
        self.ram.write_u8(SOUND2, 16);
        self.ram.write_u8(SOUND1 - 1, 5);
        self.ram.write_u8(SOUND1 - 2, 0xf2);
        self.ram.write_u8(BGMODE, 7);
    }
    pub fn world_map_brighten(&mut self) {
        let a = self.ram.read_u8(INIDISP).wrapping_add(1);
        self.ram.write_u8(INIDISP, a);
        if a == 15 {
            self.ram
                .write_u8(OW_MAP_STATE, self.ram.read_u8(OW_MAP_STATE) + 1)
        }
    }
    pub fn did_press_button_for_map(&self) -> bool {
        if self.ram.read_u8(0x656) != 0 {
            self.ram.read_u8(0xf4) & 0x20 != 0
        } else {
            self.ram.read_u8(0xf6) & 0x40 != 0
        }
    }
    pub fn world_map_player_control(&mut self) {
        if self.ram.read_u8(OW_MAP_FLAGS) & 0x80 != 0 {
            self.ram
                .write_u8(OW_MAP_FLAGS, self.ram.read_u8(OW_MAP_FLAGS) & !0x80);
            self.overworld_map_setup_hdma();
        }
        if self.ram.read_u8(OW_MAP_FLAGS) == 0 && self.did_press_button_for_map() {
            self.ram.write_u8(OW_MAP_STATE, 6);
            return;
        }
        let wait = self.ram.read_u8(DUNG_DRAW_WIDTH);
        if wait != 0 {
            self.ram.write_u8(DUNG_DRAW_WIDTH, wait - 1)
        } else if self.ram.read_u8(0xf6) & 0x30 != 0 || self.did_press_button_for_map() {
            self.ram.write_u8(SOUND2, 36);
            self.ram.write_u8(DUNG_DRAW_WIDTH, 8);
            let zoom = usize::from(self.ram.read_u8(OW_MAP_FLAGS) ^ 1);
            self.ram.write_u8(OW_MAP_FLAGS, zoom as u8 | 0x80);
            self.ram.write_u8(MODE7_ZOOM_TIMER, [33, 12][zoom]);
            if zoom == 1 {
                let y = ((self.ram.read_u16(0xc108) >> 4).wrapping_sub(0x48)) & !1;
                self.ram.write_u16(BG1_VOFS2, y);
                self.ram.write_u16(M7Y, y.wrapping_add(0x100));
                let x = (self.ram.read_u16(0xc10a) >> 4).wrapping_sub(0x80) as i16;
                let scaled = ((5 * x.unsigned_abs()) >> 1) as i16;
                let x = (if x < 0 { -scaled } else { scaled } as u16).wrapping_add(0x80) & !1;
                self.ram.write_u16(BG1_HOFS2, x);
            } else {
                self.ram.write_u16(BG1_VOFS2, 200);
                self.ram.write_u16(M7Y, 456);
                self.ram.write_u16(BG1_HOFS2, 128);
            }
        }
        if self.ram.read_u8(OW_MAP_FLAGS) != 0 {
            let k = usize::from((self.ram.read_u8(JOYPAD_H) & 12) >> 1);
            let cur = self.ram.read_i16(BG1_VOFS2);
            if cur != MAP_SCROLL_TARGET[k] {
                self.ram
                    .write_i16(BG1_VOFS2, cur.wrapping_add(MAP_SCROLL_STEP[k]));
                self.ram
                    .write_u16(M7Y, self.ram.read_u16(BG1_VOFS2).wrapping_add(0x100));
            }
            let k = usize::from((self.ram.read_u8(JOYPAD_H) & 3) * 2 + 1);
            let cur = self.ram.read_i16(BG1_HOFS2);
            if cur != MAP_SCROLL_TARGET[k] {
                self.ram
                    .write_i16(BG1_HOFS2, cur.wrapping_add(MAP_SCROLL_STEP[k]));
            }
        }
        self.world_map_handle_sprites();
    }

    fn world_map_oam_coordinates(&self) -> Option<(u16, u16)> {
        let world_y = self.ram.read_u16(0xc108);
        let world_x = self.ram.read_u16(0xc10a);
        if self.ram.read_u8(OW_MAP_FLAGS) == 0 {
            let index = -(i32::from(world_y >> 4))
                + i32::from(self.ram.read_u16(M7Y))
                + i32::from((world_y >> 3) & 1)
                - 0xc0;
            let projection = OVERWORLD_MAP_PROJECTION[index as usize];
            let y = ((13 * u16::from(projection)) >> 4) as u8;
            let mut distance = (world_x >> 4) as u8;
            let left = distance < 0x80;
            distance = distance.wrapping_sub(0x80);
            if distance & 0x80 != 0 {
                distance = !distance;
            }
            let scale = ((u16::from(if y < 224 { y } else { 0 }) * 0x54) >> 8) as u8 + 0xb2;
            let delta = ((u16::from(distance) * u16::from(scale)) >> 8) as u8;
            let projected_x = if left {
                0x80u8.wrapping_sub(delta)
            } else {
                delta.wrapping_add(0x80)
            };
            Some((
                u16::from(projected_x)
                    .wrapping_sub(self.ram.read_u16(BG1_HOFS2))
                    .wrapping_add(0x80),
                u16::from(y.wrapping_add(12)),
            ))
        } else {
            let vertical = 0u16
                .wrapping_sub(world_y >> 4)
                .wrapping_add(self.ram.read_u16(M7Y))
                .wrapping_sub(0x80);
            if vertical >= 0x100 {
                return None;
            }
            let projection_index = (vertical * 37) >> 4;
            if projection_index >= 333 {
                return None;
            }
            let y = OVERWORLD_MAP_PROJECTION[usize::from(projection_index)];
            let mut distance = world_x.wrapping_sub(0x7f8);
            let left = world_x < 0x7f8;
            if distance & 0x8000 != 0 {
                distance = distance.wrapping_neg();
            }
            let scale = ((u16::from(if y < 226 { y } else { 0 }) * 84) >> 8) as u8 + 178;
            let projected = (u16::from(distance as u8) * u16::from(scale)) >> 8;
            let projected = u16::from((distance >> 8) as u8) * u16::from(scale) + projected;
            let map_x = if left {
                0x800u16.wrapping_sub(projected)
            } else {
                projected.wrapping_add(0x800)
            };
            let left = map_x < 0x800;
            let distance = if left {
                0x800u16.wrapping_sub(map_x)
            } else {
                map_x.wrapping_sub(0x800)
            };
            let projected = (u16::from(distance as u8) * 45) >> 8;
            let projected = (distance >> 8) * 45 + projected;
            let x = if left {
                0x80u16.wrapping_sub(projected)
            } else {
                projected.wrapping_add(0x80)
            }
            .wrapping_sub(self.ram.read_u16(BG1_HOFS2));
            let extension = if self.ram.read_u32(0x64c) & 1 != 0 {
                0x48
            } else {
                0
            };
            if x.wrapping_add(0x80 + extension) >= 0x100 + extension * 2 {
                return None;
            }
            Some((x.wrapping_add(0x81), u16::from(y) + 16))
        }
    }

    pub fn flute_menu(&mut self) {
        match self.ram.read_u8(OW_MAP_STATE) {
            0 => self.world_map_fade_out(),
            1 => {
                self.ram.write_u8(0x1af0, 0);
                self.runtime.world_map_load_light(self.ram)
            }
            2 => self.runtime.world_map_load_sprite_gfx(self.ram),
            3 => self.world_map_brighten(),
            4 => {
                self.ram.write_u8(SOME_MENU_CTR, 0x10);
                self.ram.write_u8(OW_MAP_STATE, 5)
            }
            5 => self.flute_handle_selection(),
            6 => self.runtime.world_map_restore_gfx(self.ram),
            7 => self.runtime.flute_load_selected_screen(self.ram),
            8 => self.runtime.flute_load_overlay(self.ram),
            9 => {
                let a = self.ram.read_u8(INIDISP) + 1;
                self.ram.write_u8(INIDISP, a);
                if a == 15 {
                    self.bird_travel_finish_doit();
                } else {
                    self.runtime.flute_sprite_main(self.ram)
                }
            }
            _ => {}
        }
    }
    pub fn flute_handle_selection(&mut self) {
        const BIRD_CHAR: [u8; 8] = [0x7f, 0x79, 0x6c, 0x6d, 0x6e, 0x6f, 0x7c, 0x7d];
        let timer = self.ram.read_u8(SOME_MENU_CTR);
        if timer == 0 && (self.ram.read_u8(JOYPAD_L) | self.ram.read_u8(JOYPAD_H)) & 0xc0 != 0 {
            self.ram.write_u8(OW_MAP_STATE, 6);
            return;
        }
        if timer != 0 {
            self.ram.write_u8(SOME_MENU_CTR, timer - 1)
        }
        let mut choice = self.ram.read_u8(0x1af0);
        if self.ram.read_u8(0xf4) & 10 != 0 {
            choice = choice.wrapping_sub(1);
            self.ram.write_u8(SOUND2, 32)
        }
        if self.ram.read_u8(0xf4) & 5 != 0 {
            choice = choice.wrapping_add(1);
            self.ram.write_u8(SOUND2, 32)
        }
        self.ram.write_u8(0x1af0, choice & 7);
        if self.ram.read_u8(0x1a) & 0x10 != 0
            && let Some((x, y)) = self.world_map_oam_coordinates()
        {
            self.world_map_add_sprite(16, 2, 0x3e, 0, x.wrapping_sub(4), y.wrapping_sub(4));
        }
        let saved_x = self.ram.read_u16(0xc10a);
        let saved_y = self.ram.read_u16(0xc108);
        for i in 0..8 {
            self.ram.write_u8(0x1ab0 + i, BIRD_X_LO[i]);
            self.ram.write_u8(0x1ac0 + i, BIRD_X_HI[i]);
            self.ram.write_u8(0x1ad0 + i, BIRD_Y_LO[i]);
            self.ram.write_u8(0x1ae0 + i, BIRD_Y_HI[i]);
            self.ram
                .write_u16(0xc10a, u16::from_le_bytes([BIRD_X_LO[i], BIRD_X_HI[i]]));
            self.ram
                .write_u16(0xc108, u16::from_le_bytes([BIRD_Y_LO[i], BIRD_Y_HI[i]]));
            if let Some((x, y)) = self.world_map_oam_coordinates() {
                let flags = if i == usize::from(choice & 7) {
                    0x30 + (self.ram.read_u8(0x1a) & 6)
                } else {
                    0x32
                };
                self.world_map_add_sprite(i, 0, flags, BIRD_CHAR[i], x, y);
            }
        }
        self.ram.write_u16(0xc10a, saved_x);
        self.ram.write_u16(0xc108, saved_y);
    }

    pub fn map_has_pendant(&self, k: usize) -> bool {
        self.ram.read_u8(MAP_ICONS) == 3 && self.ram.read_u8(LINK_PENDANTS) & PENDANT_MASK[k] != 0
    }
    pub fn map_has_crystal(&self, k: usize) -> bool {
        self.ram.read_u8(MAP_ICONS) == 7 && self.ram.read_u8(LINK_CRYSTALS) & CRYSTAL_MASK[k] != 0
    }
    pub fn world_map_add_sprite(
        &mut self,
        sprite: usize,
        mut big: u8,
        mut flags: u8,
        mut ch: u8,
        mut x: u16,
        mut y: u16,
    ) {
        if self.ram.read_u8(0x1a) & 0x10 == 0 && ch == 100 {
            assert!(sprite >= 8);
            ch = OVERWORLD_MAP_CHARS[sprite - 8];
            flags = 0x32;
            big = 0;
        } else {
            x = x.wrapping_sub(4);
            y = y.wrapping_sub(4);
        }
        if self.ram.read_u32(0x64c) & 1 != 0 {
            big |= (x >> 8) as u8 & 1;
        }
        crate::oam::set_plain(self.ram, sprite, x as u8, y as u8, ch, flags, big);
    }

    // C entry-point spellings.  Keeping these small forwarding methods is
    // useful for differential traces: a C symbol has one unambiguous Rust
    // landing point even when the idiomatic implementation above has a
    // shorter name.
    pub fn get_dungmap_floor_layout(&self) -> Option<&[u8]> {
        z3_core::assets::packed_array_entry(self.assets.dungeon_map_floor_layout, self.palace())
    }
    pub fn get_other_dungmap_info(&self, count: usize) -> Option<u8> {
        z3_core::assets::packed_array_entry(self.assets.dungeon_map_info, self.palace())?
            .get(count)
            .copied()
    }
    pub fn dung_map_4(&mut self) {
        self.dungeon_map_slide();
    }
    pub fn module_messaging_6(&self) {
        panic!("Module_Messaging_6 is an unreachable C assertion")
    }
    pub fn overworld_map_setup_hdma(&mut self) {
        let zoomed = self.ram.read_u8(OW_MAP_FLAGS) != 0;
        self.runtime.overworld_map_setup_hdma(self.ram, zoomed);
    }
    pub fn get_light_overworld_tilemap(&self) -> Option<&[u8]> {
        Some(self.assets.light_overworld_tilemap)
    }
    pub fn transfer_mode7_characters(&mut self) {
        self.runtime.transfer_mode7_characters(self.ram);
    }
    pub fn module0e_interface(&mut self) {
        let indoors = self.ram.read_u8(0x1b) != 0;
        let submodule = self.ram.read_u8(SUBMODULE);
        let map_state = self.ram.read_u8(OW_MAP_STATE);
        let skip_world = if indoors {
            submodule == 3 && map_state != 0 && map_state != 7
        } else {
            matches!(submodule, 7 | 10) && map_state != 0
        };
        self.runtime.interface_gameplay_step(self.ram, !skip_world);
        self.run_interface();
        let x = self.ram.read_u16(0x11a);
        let y = self.ram.read_u16(0x11c);
        self.ram
            .write_u16(0x11e, self.ram.read_u16(0xe2).wrapping_add(x));
        self.ram
            .write_u16(0x122, self.ram.read_u16(BG2_VOFS2).wrapping_add(y));
        self.ram
            .write_u16(0x120, self.ram.read_u16(BG1_HOFS2).wrapping_add(x));
        self.ram
            .write_u16(0x124, self.ram.read_u16(BG1_VOFS2).wrapping_add(y));
    }
    pub fn module_messaging_0(&self) {
        panic!("Module_Messaging_0 is an unreachable C assertion")
    }
    pub fn run_interface(&mut self) {
        self.run_submodule();
    }
    pub fn module0e_05_desert_prayer(&mut self) {
        match self.ram.read_u8(SUBSUBMODULE) {
            0 => self
                .runtime
                .reset_transition_props_and_advance_reset_interface(self.ram),
            1 => self.runtime.apply_palette_filter_bounce(self.ram),
            2 => {
                self.desert_prayer_initialize_iris_hdma();
                self.ram
                    .write_u8(0xc007, self.ram.read_u8(0xc00b).wrapping_sub(1));
                self.ram.write_u8(0xc00b, 0);
                self.ram.write_u8(0xc009, 2);
            }
            3 => {
                self.runtime.apply_palette_filter_bounce(self.ram);
                self.desert_prayer_build_iris_hdma_table();
            }
            4 => self.desert_prayer_build_iris_hdma_table(),
            _ => {}
        }
    }
    pub fn module0e_04_red_potion(&mut self) {
        self.red_potion();
    }
    pub fn module0e_08_green_potion(&mut self) {
        self.green_potion();
    }
    pub fn module0e_09_blue_potion(&mut self) {
        self.blue_potion();
    }
    pub fn module0e_0b_save_menu(&mut self) {
        self.render_text();
        self.ram.write_u8(HUD_UPDATE, 0);
        self.ram.write_u8(NMI_DISABLE_CORE, 0);
        let ss = self.ram.read_u8(SUBSUBMODULE);
        if ss < 3 {
            self.ram.write_u8(SUBSUBMODULE, ss + 1);
        } else {
            self.ram.write_u8(NMI_LOAD_BG, 0);
        }
        if self.ram.read_u8(SUBMODULE) == 0 {
            self.ram.write_u8(SUBSUBMODULE, 0);
            self.ram.write_u8(NMI_LOAD_BG, 1);
            if self.ram.read_u8(CHOICE) != 0 {
                self.ram.write_u8(0x12d, 15);
                self.ram.write_u8(MAIN_MODULE, 23);
                self.ram.write_u8(SUBMODULE, 1);
                self.ram.write_u16(0x430, 0);
            } else {
                self.ram.write_u8(CHOICE, self.ram.read_u8(CHOICE_BACKUP));
            }
        }
    }
    pub fn module1b_spawn_select(&mut self) {
        self.render_text();
        if self.ram.read_u8(SUBMODULE) == 0 {
            self.ram.write_u8(NMI_LOAD_BG, 0);
            self.ram.write_u8(SUBSUBMODULE, 0);
            self.runtime.display_select_menu(self.ram);
        }
    }
    pub fn clean_up_and_prep_desert_prayer_hdma(&mut self) {
        self.runtime.desert_prayer_setup_hdma(self.ram);
        self.ram.write_u8(0x96, 0x33);
        self.ram.write_u8(0x97, 3);
        self.ram.write_u8(0x98, 0x33);
        self.ram.write_u8(0x1e, self.ram.read_u8(TM_COPY));
        self.ram.write_u8(0x1f, self.ram.read_u8(TM_COPY + 1));
        self.ram.write_u8(0x9b, 0x80);
        for i in 0..240 {
            self.ram.write_u16(0x1dba0 + i * 2, 0);
        }
    }
    pub fn desert_prayer_initialize_iris_hdma(&mut self) {
        self.clean_up_and_prep_desert_prayer_hdma();
        self.ram.write_u16(0x67c, 0x26);
        self.ram.write_u8(0x67e, 0);
        self.desert_prayer_build_iris_hdma_table();
        self.ram
            .write_u8(SUBSUBMODULE, self.ram.read_u8(SUBSUBMODULE).wrapping_add(1));
    }
    pub fn desert_hdma_calculate_iris_shape_line(&self) -> (u8, u16) {
        let radius = self.ram.read_u8(0x67c);
        if radius == 0 {
            return (0, 0);
        }
        let dy = self.ram.read_u8(0x67a);
        let q = usize::from((((u16::from(dy) << 8) / u16::from(radius)) >> 1) as u8);
        let expanded = self.ram.read_u8(0x67e) != 0;
        let curve = if expanded {
            PRAYING_SCENE_TAB1[q]
        } else {
            PRAYING_SCENE_TAB0[q]
        };
        let width = (u16::from(curve) * u16::from(radius)) >> 8;
        (curve, if expanded { width << 1 } else { width })
    }
    pub fn desert_prayer_build_iris_hdma_table(&mut self) {
        let r14 = self
            .ram
            .read_u16(LINK_Y)
            .wrapping_sub(self.ram.read_u16(BG2_VOFS2))
            .wrapping_add(12);
        let radius = self.ram.read_u16(0x67c);
        let mut lower = r14.wrapping_sub(radius);
        let upper = lower.wrapping_add(radius.wrapping_mul(2));
        self.ram.write_u16(0x674, lower);
        self.ram.write_u16(0x676, upper);
        self.ram.write_u16(
            0x670,
            self.ram
                .read_u16(0x22)
                .wrapping_sub(self.ram.read_u16(0xe2))
                .wrapping_add(8),
        );
        let mut shape_line = 1u16;
        let mut r4 = if lower as i16 >= 0 { 0 } else { lower };
        loop {
            let mut left = 0x100u16;
            let mut right = 0x100u16;
            let k;
            let inside = (lower as i16) < 0 || (r4 >= lower && r4 < upper);
            if !inside {
                k = r4.wrapping_sub(1);
            } else if radius < shape_line {
                shape_line = 1;
                lower = 0;
                self.ram.write_u16(0x674, 0);
                r4 = upper;
                if r4 >= 225 {
                    break;
                }
                k = r4.wrapping_sub(1);
            } else {
                self.ram.write_u16(0x67a, shape_line);
                let (curve, half_width) = self.desert_hdma_calculate_iris_shape_line();
                if curve == 0 {
                    lower = 0;
                    self.ram.write_u16(0x674, 0);
                } else {
                    let center = self.ram.read_u16(0x670);
                    right = center.wrapping_add(half_width);
                    left = center.wrapping_sub(half_width);
                }
                k = r14.wrapping_sub(shape_line as u8 as u16).wrapping_sub(1);
            }
            let low = if left < 256 {
                left as u8
            } else if left < 512 {
                255
            } else {
                0
            };
            let high = if right < 256 { right as u8 } else { 255 };
            let packed = u16::from(high) << 8 | u16::from(low);
            let packed = if packed == 0xffff { 0xff } else { packed };
            if k < 240 {
                self.ram.write_u16(0x1dba0 + usize::from(k) * 2, packed);
            }
            if (lower as i16) < 0 || (r4 >= lower && r4 < upper) {
                // C promotes BYTE(spotlight_var4) to int before subtracting 2.
                // At the first line, 1 - 2 is -1, not u8::MAX.
                let mirror = r14
                    .wrapping_add(u16::from(shape_line as u8))
                    .wrapping_sub(2);
                if mirror < 240 {
                    self.ram
                        .write_u16(0x1dba0 + usize::from(mirror) * 2, packed);
                }
                shape_line = shape_line.wrapping_add(1);
            }
            r4 = r4.wrapping_add(1);
            if r4 as i16 >= 0 && r4 >= 225 {
                break;
            }
        }
        self.ram.write_u16(0x67a, shape_line);

        if self.ram.read_u8(SUBSUBMODULE) != 4 {
            return;
        }
        let mut expanding = self.ram.read_u8(0x67e);
        if expanding != 1 && (self.ram.read_u8(0xf4) | self.ram.read_u8(0xf6)) & 0xc0 != 0 {
            expanding = 1;
            self.ram.write_u8(0x67e, 1);
            self.ram.write_u8(0x67c, self.ram.read_u8(0x67c) >> 1);
        }
        if expanding != 0 {
            let next = self.ram.read_u8(0x67c).wrapping_add(8);
            self.ram.write_u8(0x67c, next);
            if next >= 0xc0 {
                self.ram.write_u8(0x2f0, self.ram.read_u8(0x2f0) ^ 1);
                self.ram.write_u8(0x12c, 0xf3);
                self.ram.write_u8(0x12d, 0);
                self.ram.write_u8(0xfc1, 0);
                self.ram.write_u8(0x30a, 0);
                self.ram.write_u8(BUTTON_MASK_BY, 0);
                self.ram.write_u8(0x308, 0);
                self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
                self.ram.write_u8(SUBSUBMODULE, 0);
                self.ram.write_u8(SUBMODULE, 0);
                self.ram
                    .write_u8(MAIN_MODULE, self.ram.read_u8(SAVED_MODULE));
                for address in [0x1e, 0x1f, 0x96, 0x97, 0x98] {
                    self.ram.write_u8(address, 0);
                }
                self.runtime.iris_spotlight_reset_table(self.ram);
                return;
            }
        }
        const DELAYS: [u8; 5] = [22, 22, 22, 64, 1];
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if delay & 0x80 != 0 {
            let frame = usize::from(self.ram.read_u8(0x30a).wrapping_add(1));
            if frame != 4 {
                self.ram.write_u8(0x30a, frame as u8);
            }
            self.ram.write_u8(0x3d, DELAYS[frame]);
        }
    }

    pub fn game_over_text_sweep_left(&mut self) {
        self.game_over_sweep_left();
    }
    pub fn game_over_text_unfurl_right(&mut self) {
        self.game_over_unfurl_right();
    }
    pub fn module12_game_over(&mut self) {
        self.module_game_over();
    }
    pub fn game_over_advance_immediately(&mut self) {
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
        self.death_func1();
    }
    pub fn death_func1(&mut self) {
        self.ram.write_u8(0xc227, self.ram.read_u8(0x130));
        self.ram.write_u8(0xc228, self.ram.read_u8(0x131));
        self.ram.write_u8(0x12c, 0xf1);
        self.ram.write_u8(0x12d, 5);
        self.ram.write_u8(OW_MAP_STATE, 5);
        self.ram.write_u8(0x3f3, 0);
        self.ram.write_u8(0x322, 0);
        self.ram.write_u8(0x55, 0);
        self.ram.write_u16(MAPBAK_BG1_X, self.ram.read_u16(0xc007));
        self.ram.write_u16(MAPBAK_BG1_Y, self.ram.read_u16(0xc009));
        let palette = self.ram.slice(0xc300, 256).to_vec();
        self.ram.slice_mut(0x1dd80, 256).copy_from_slice(&palette);
        self.ram.slice_mut(0xc340, 192).fill(0);
        self.ram.write_u16(0xc007, 0);
        self.ram.write_u16(0xc009, 0);
        self.ram.write_u16(MAPBAK_CGWSEL, self.ram.read_u16(0x99));
        self.ram.write_u8(SOME_MENU_CTR, 32);
        self.ram.write_u16(0x11a, 0);
        self.ram.write_u16(0x11c, 0);
        self.ram.write_u8(0x4a0, 0);
        self.runtime.hud_floor_indicator(self.ram);
        self.ram
            .write_u8(HUD_UPDATE, self.ram.read_u8(HUD_UPDATE).wrapping_add(1));
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
    }
    pub fn death_func6(&mut self) {
        self.death_load_game_over_gfx();
    }
    pub fn death_func4(&mut self) {
        self.death_player_swoon();
    }
    pub fn animate_game_over_letters_bounce(&mut self) {
        self.animate_game_over_letters();
    }
    pub fn game_over_finalize_gameovr(&mut self) {
        self.game_over_finalize_text();
    }
    pub fn game_over_save_and_or_continue(&mut self) {
        self.game_over_save_or_continue();
    }
    pub fn death_func15(&mut self, count_as_death: bool) {
        self.runtime
            .death_finalize_choice(self.ram, self.sram, count_as_death);
    }
    pub fn game_over_animate_choice_fairy(&mut self) {
        const Y: [u8; 3] = [0x94, 0xa4, 0xb4];
        const CH: [u8; 2] = [0x29, 0x2a];
        let choice = usize::from(self.ram.read_u8(SUBSUBMODULE).min(2));
        let frame = usize::from((self.ram.read_u8(0x1a) >> 3) & 1);
        crate::oam::set_plain(self.ram, 0x14, 0x34, Y[choice], CH[frame], 0x78, 2);
    }
    pub fn game_over_initialize_revival_fairy(&mut self) {
        self.ram.write_u8(LINK_HEARTS_FILLER, 56);
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
        self.ram.write_u8(OW_MAP_STATE, 0);
        self.runtime.death_step(self.ram, 10);
    }
    pub fn revival_fairy_main_bounce(&mut self) {
        self.runtime.death_step(self.ram, 11);
    }
    pub fn game_over_restore0e(&mut self) {
        self.runtime.death_step(self.ram, 14);
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
    }

    pub fn module0e_0a_flute_menu(&mut self) {
        self.flute_menu();
    }
    pub fn flute_menu_handle_selection(&mut self) {
        self.flute_handle_selection();
    }
    pub fn flute_menu_load_selected_screen(&mut self) {
        self.runtime.flute_load_selected_screen(self.ram);
    }
    pub fn overworld_load_overlay_and_map(&mut self) {
        self.runtime.flute_load_overlay(self.ram);
    }
    pub fn flute_menu_fade_in_and_quack(&mut self) {
        let value = self.ram.read_u8(INIDISP).wrapping_add(1);
        self.ram.write_u8(INIDISP, value);
        if value == 15 {
            self.bird_travel_finish_doit();
        } else {
            self.runtime.flute_sprite_main(self.ram);
        }
    }
    pub fn bird_travel_finish_doit(&mut self) {
        self.ram.write_u8(OW_MAP_STATE, 0);
        self.ram.write_u8(SUBSUBMODULE, 0);
        self.ram
            .write_u8(MAIN_MODULE, self.ram.read_u8(SAVED_MODULE));
        self.ram.write_u8(SUBMODULE, 0);
        self.ram.write_u8(HDMAEN, self.ram.read_u8(MAPBAK_HDMAEN));
        self.runtime.flute_finish(self.ram);
    }
    pub fn messaging_overworld_map(&mut self) {
        self.overworld_map();
    }
    pub fn world_map_load_light_world_map(&mut self) {
        self.runtime.world_map_load_light(self.ram);
    }
    pub fn world_map_load_dark_world_map(&mut self) {
        self.runtime.world_map_load_dark(self.ram);
    }
    pub fn world_map_restore_graphics(&mut self) {
        self.runtime.world_map_restore_gfx(self.ram);
    }
    pub fn attract_set_up_conclusion_hdma(&mut self) {
        self.runtime.overworld_map_setup_hdma(self.ram, false);
        self.ram.write_u8(HDMAEN, 0x80);
        self.ram.write_u8(BGMODE, 9);
        self.ram.write_u8(NMI_DISABLE_CORE, 0);
    }
    pub fn world_map_exit_map(&mut self) {
        self.runtime.world_map_exit(self.ram);
    }
    pub fn world_map_set_up_hdma(&mut self) {
        self.overworld_map_setup_hdma();
    }
    pub fn world_map_fill_tilemap_with_ef(&mut self) {
        self.runtime.transfer_mode7_characters(self.ram);
    }
    pub fn world_map_handle_sprites(&mut self) {
        let frame = self.ram.read_u8(0x1a);
        if frame & 0x10 != 0
            && let Some((x, y)) = self.world_map_oam_coordinates()
        {
            self.world_map_add_sprite(0, 2, 0x3e, 0, x.wrapping_sub(4), y.wrapping_sub(4));
        }
        let saved_y = self.ram.read_u16(0xc108);
        let saved_x = self.ram.read_u16(0xc10a);
        let bird = 15usize;
        if self.ram.read_u8(0x8a) < 0x40
            && (self.ram.read_u8(0x1ab0 + bird)
                | self.ram.read_u8(0x1ac0 + bird)
                | self.ram.read_u8(0x1ad0 + bird)
                | self.ram.read_u8(0x1ae0 + bird))
                != 0
        {
            if frame == 0 {
                self.ram.write_u8(
                    0x1af0 + bird,
                    self.ram.read_u8(0x1af0 + bird).wrapping_add(1),
                );
            }
            self.ram.write_u16(
                0xc10a,
                u16::from_le_bytes([
                    self.ram.read_u8(0x1ab0 + bird),
                    self.ram.read_u8(0x1ac0 + bird),
                ]),
            );
            self.ram.write_u16(
                0xc108,
                u16::from_le_bytes([
                    self.ram.read_u8(0x1ad0 + bird),
                    self.ram.read_u8(0x1ae0 + bird),
                ]),
            );
            if let Some((x, y)) = self.world_map_oam_coordinates() {
                const BIRD_FLAGS: [u8; 4] = [0x34, 0x74, 0xf4, 0xb4];
                self.world_map_add_sprite(
                    15,
                    2,
                    BIRD_FLAGS[usize::from(frame >> 1 & 3)],
                    0x6a,
                    x,
                    y,
                );
            }
        }
        let map_icons = usize::from(self.ram.read_u8(MAP_ICONS));
        let suppress = self.ram.read_u8(0xf280 + 0x5b) & 0x20 != 0
            || (((map_icons >= 6) as u8 ^ self.ram.read_u8(0xfff)) & 1) != 0;
        if !suppress && map_icons < 9 {
            for icon in 0..7 {
                let owned = if icon < 3 {
                    self.map_has_pendant(icon) || self.map_has_crystal(icon)
                } else {
                    self.map_has_crystal(icon)
                };
                let x = OW_MAP_X[icon][map_icons];
                if owned || x & 0x8000 != 0 {
                    continue;
                }
                let mut y = OW_MAP_Y[icon][map_icons];
                let info = OW_MAP_INFO[icon][map_icons];
                let character = (info >> 8) as u8;
                let mut x = x;
                if character != 0 {
                    if character != 100 && frame & 0x10 != 0 {
                        continue;
                    }
                    x = x.wrapping_sub(4);
                    y = y.wrapping_sub(4);
                }
                self.ram.write_u16(0xc10a, x);
                self.ram.write_u16(0xc108, y);
                if let Some((x, y)) = self.world_map_oam_coordinates() {
                    let (big, flags, character) = if character == 0 {
                        (
                            0,
                            0x32,
                            [0x68, 0x69, 0x78, 0x69][usize::from(frame >> 3 & 3)],
                        )
                    } else {
                        (2, info as u8, character)
                    };
                    self.world_map_add_sprite(14 - icon, big, flags, character, x, y);
                }
            }
        }
        self.ram.write_u16(0xc10a, saved_x);
        self.ram.write_u16(0xc108, saved_y);
    }
    pub fn world_map_calculate_oam_coordinates(&mut self) -> bool {
        if let Some((x, y)) = self.world_map_oam_coordinates() {
            self.ram.write_u16(0, x);
            self.ram.write_u16(2, y);
            true
        } else {
            false
        }
    }
    pub fn overworld_map_check_for_pendant(&self, k: usize) -> bool {
        self.map_has_pendant(k)
    }
    pub fn overworld_map_check_for_crystal(&self, k: usize) -> bool {
        self.map_has_crystal(k)
    }

    pub fn module0e_03_dungeon_map(&mut self) {
        self.dungeon_map();
    }
    pub fn module0e_03_01_draw_map(&mut self) {
        self.dungeon_map_draw();
    }
    pub fn module0e_03_01_00_prep_map_graphics(&mut self) {
        self.runtime.dungeon_map_prepare_graphics(self.ram);
    }
    pub fn module0e_03_01_01_draw_level(&mut self) {
        self.dungeon_map_draw_level();
    }
    pub fn module0e_03_01_02_draw_floors_backdrop(&mut self) {
        self.dungeon_map_draw_floors_backdrop();
    }
    pub fn dungeon_map_build_floor_list_boxes(
        &mut self,
        t5: u8,
        address: u16,
        offset: usize,
    ) -> usize {
        self.dungeon_map_build_floor_boxes(t5, address, offset)
    }
    pub fn module0e_03_01_03_draw_rooms(&mut self) {
        self.dungeon_map_draw_rooms();
    }
    pub fn dungeon_map_draw_border_for_rooms(&mut self, pd: u16, mask: u16) {
        self.dungeon_map_draw_border(pd, mask);
    }
    pub fn dungeon_map_draw_floor_numbers_by_room(&mut self, pd: u16, mask: u16) {
        self.dungeon_map_draw_floor_numbers(pd, mask);
    }
    pub fn dungeon_map_draw_dungeon_layout(&mut self, pd: u16) {
        self.dungeon_map_draw_layout(pd);
    }
    pub fn dungeon_map_draw_single_row_of_rooms(&mut self, row: usize, offset: usize) {
        self.dungeon_map_draw_row(row, offset);
    }
    pub fn dungeon_map_draw_room_markers(&mut self) {
        let palace = self.palace();
        let low_floors = (DUNG_TAB5[palace] & 0xf) as u8;
        let floor = usize::from(low_floors.wrapping_add(self.ram.read_u8(0xa4)));
        let mut room = self.ram.read_u16(0xa0);
        for i in 0..3 {
            if room == DUNG_TAB21[i] {
                room = DUNG_TAB22[i];
            }
        }
        let layout =
            z3_core::assets::packed_array_entry(self.assets.dungeon_map_floor_layout, palace)
                .expect("dungeon map floor layout");
        let current_floor = &layout[floor * 25..floor * 25 + 25];
        let position = current_floor
            .iter()
            .position(|&value| value == room as u8)
            .unwrap_or(25);
        let x = ((position % 5) * 16) as u8;
        let y = ((position / 5) * 16) as u8;
        let marker_x = x
            .wrapping_add(0x90)
            .wrapping_add(((self.ram.read_u16(0x22) & 0x1e0) >> 5) as u8);
        self.ram.write_u16(0x215, u16::from(marker_x));
        self.ram.write_u16(0xcf5, u16::from(y));
        let marker_y = y
            .wrapping_add(DUNG_TAB24[usize::from(self.ram.read_u16(0x211) >> 1)] as u8)
            .wrapping_add(((self.ram.read_u16(0x20) & 0x1e0) >> 5) as u8);
        self.ram.write_u16(0x217, u16::from(marker_y));

        let boss_floor = usize::from(low_floors.wrapping_add(DUNG_TAB28[palace] as u8));
        let boss_layout = &layout[boss_floor * 25..boss_floor * 25 + 25];
        let boss_room = DUNG_TAB25[palace] as u8;
        let boss_position = boss_layout
            .iter()
            .rposition(|&value| value != 0xf && value == boss_room)
            .unwrap_or(0);
        let boss_x = ((boss_position % 5) * 16) as u16;
        let boss_y = ((boss_position / 5) * 16) as u16;
        self.ram.write_u16(0xfa8, boss_x);
        self.ram.write_u16(
            0xfaa,
            boss_y
                .wrapping_add_signed(
                    i16::from(
                        self.ram
                            .read_u8(0x20e)
                            .wrapping_sub(DUNG_TAB28[palace] as u8) as i8,
                    ) * 0x60,
                )
                .wrapping_add(DUNG_TAB24[0]),
        );
        self.ram
            .write_u8(OW_MAP_STATE, self.ram.read_u8(OW_MAP_STATE).wrapping_add(1));
        self.ram.write_u8(INIDISP, 0);
        self.ram.write_u8(0x20d, 0);
    }
    pub fn dungeon_map_handle_input_and_sprites(&mut self) {
        self.dungeon_map_handle_input();
        self.dungeon_map_draw_sprites();
    }
    pub fn want_exit_dungeon_map(&self) -> bool {
        self.did_press_button_for_map()
    }
    pub fn dungeon_map_handle_movement_input(&mut self) {
        self.dungeon_map_floor_select();
        if self.ram.read_u8(0x210) != 0 {
            self.dungeon_map_scroll_floors();
        }
    }
    pub fn dungeon_map_handle_floor_select(&mut self) {
        self.dungeon_map_floor_select();
    }
    pub fn dungeon_map_draw_sprites(&mut self) {
        let palace = self.palace();
        let low_floors = (DUNG_TAB5[palace] & 0xf) as u8;
        let floor = low_floors.wrapping_add(self.ram.read_u8(0xa4));
        let mut sprite = 0;
        self.dungeon_map_draw_link_pointing(sprite, low_floors, floor);
        sprite += 1;
        let mut marker = 0;
        loop {
            sprite = self.dungeon_map_draw_location_marker(sprite, marker);
            marker += 1;
            if sprite == 9 {
                break;
            }
        }
        sprite = self.dungeon_map_draw_blinking_indicator(sprite);
        sprite = self.dungeon_map_draw_boss_icon(sprite);
        let _ = self.dungeon_map_draw_floor_number_objects(sprite);
        self.dungeon_map_draw_floor_blinker();
    }
    pub fn dungeon_map_draw_link_pointing(&mut self, sprite: usize, r2: u8, mut r3: u8) {
        let t5 = DUNG_TAB5[self.palace()] as u8;
        if 4u8.wrapping_sub(r2) < 0x80 {
            r3 = r3.wrapping_add(4u8.wrapping_sub(r2));
            let excess = (t5 >> 4).wrapping_sub(4);
            if excess < 0x80 {
                r3 = r3.wrapping_sub(excess);
            }
        }
        crate::oam::set_plain(
            self.ram,
            sprite,
            0x19,
            DUNG_TAB33[usize::from(r3)].wrapping_sub(4),
            0,
            if self.ram.read_u8(0xabd) != 0 {
                0x30
            } else {
                0x3e
            },
            2,
        );
    }
    pub fn dungeon_map_draw_blinking_indicator(&mut self, sprite: usize) -> usize {
        let y = if self.ram.read_u16(0x217) < 256 {
            self.ram.read_u8(0x217)
        } else {
            0xf0
        };
        crate::oam::set_plain(
            self.ram,
            sprite,
            self.ram.read_u8(0x215).wrapping_sub(3),
            y.wrapping_sub(3),
            0x34,
            DUNG_TAB38[usize::from((self.ram.read_u8(0x1a) >> 2) & 3)],
            0,
        );
        sprite + 1
    }
    pub fn dungeon_map_draw_location_marker(&mut self, sprite: usize, marker: u16) -> usize {
        let base_x = self.ram.read_u8(0x215) & 0xf0;
        let base_y = self
            .ram
            .read_u8(0xcf5)
            .wrapping_add(DUNG_TAB24[marker as usize] as u8);
        for i in (0..4).rev() {
            let mut frame = usize::from((self.ram.read_u8(0x1a) >> 2) & 1);
            if self.ram.read_u8(0x217).wrapping_add(1) & 0xf0 == base_y.wrapping_add(1)
                && self.ram.read_u16(0x217) < 256
            {
                frame += 2;
            }
            crate::oam::set_plain(
                self.ram,
                sprite + (3 - i),
                base_x.wrapping_add_signed(DUNG_TAB29[i]),
                base_y.wrapping_add_signed(DUNG_TAB30[i]),
                0,
                DUNG_TAB32[frame] | DUNG_TAB31[i],
                2,
            );
        }
        sprite + 4
    }
    pub fn dungeon_map_draw_floor_blinker(&mut self) {
        let mut floor = self.ram.read_u8(0x20e);
        let t5 = DUNG_TAB5[self.palace()] as u8;
        let mut flag = usize::from(((t5 >> 4) & 0xf).wrapping_add(t5 & 0xf) != 1);
        floor = floor.wrapping_sub(flag as u8);
        let mut row;
        let mut count = flag;
        loop {
            row = floor.wrapping_add(t5 & 0xf);
            let below = 4u8.wrapping_sub(t5 & 0xf);
            if below < 0x80 {
                row = row.wrapping_add(below);
                let above = (t5 >> 4).wrapping_sub(4);
                if above < 0x80 {
                    row = row.wrapping_sub(above);
                }
            }
            floor = floor.wrapping_add(1);
            if count == 0 {
                break;
            }
            count -= 1;
        }
        if self.ram.read_u8(0x1a) & 0x10 == 0 {
            return;
        }
        let y = DUNG_TAB33[usize::from(row)].wrapping_sub(4);
        loop {
            let mut x = 40u8;
            let sprite = 0x40 + DUNG_TAB35[flag];
            for i in (0..4).rev() {
                let flags = 0x3d | if i == 0 { 0x40 } else { 0 };
                crate::oam::set_plain(
                    self.ram,
                    sprite + (3 - i),
                    x,
                    y.wrapping_add((flag * 16) as u8),
                    DUNG_TAB36[i],
                    flags,
                    0,
                );
                crate::oam::set_plain(
                    self.ram,
                    sprite + (3 - i) + 4,
                    x,
                    y.wrapping_add((flag * 16 + 8) as u8),
                    DUNG_TAB36[i],
                    flags | 0x80,
                    0,
                );
                x = x.wrapping_add(8);
            }
            if flag == 0 {
                break;
            }
            flag -= 1;
        }
    }
    pub fn dungeon_map_draw_boss_icon(&mut self, sprite: usize) -> usize {
        let palace = self.palace();
        if self
            .ram
            .read_u16(0xf000 + usize::from(DUNG_TAB25[palace]) * 2)
            & 0x800
            != 0
            || self.ram.read_u16(0xf364) & UPPER_BITMASKS[palace] == 0
            || DUNG_TAB28[palace] < 0
        {
            return sprite;
        }
        let sprite = self.dungeon_map_draw_boss_icon_by_floor(sprite);
        if self.ram.read_u8(0x1a) & 0xf >= 10 {
            return sprite;
        }
        let xy = DUNG_TAB37[palace] as u16;
        crate::oam::set_plain(
            self.ram,
            sprite,
            (xy >> 8)
                .wrapping_add(self.ram.read_u16(0xfa8))
                .wrapping_add(0x90) as u8,
            if self.ram.read_u16(0xfaa) < 256 {
                (xy as u8).wrapping_add(self.ram.read_u8(0xfaa))
            } else {
                0xf0
            },
            0x31,
            0x33,
            0,
        );
        sprite + 1
    }
    pub fn dungeon_map_draw_boss_icon_by_floor(&mut self, sprite: usize) -> usize {
        let palace = self.palace();
        let t5 = DUNG_TAB5[palace] as u8;
        let low_floors = t5 & 0xf;
        let mut row = low_floors.wrapping_add(DUNG_TAB28[palace] as u8);
        let below = 4u8.wrapping_sub(low_floors);
        if below < 0x80 {
            row = row.wrapping_add(below);
            let above = (t5 >> 4).wrapping_sub(4);
            if above < 0x80 {
                row = row.wrapping_sub(above);
            }
        }
        if self.ram.read_u8(0x1a) & 0xf >= 10 {
            return sprite;
        }
        crate::oam::set_plain(
            self.ram,
            sprite,
            0x4c,
            DUNG_TAB33[usize::from(row)],
            0x31,
            0x33,
            0,
        );
        sprite + 1
    }
    pub fn dungeon_map_recover_gfx(&mut self) {
        self.runtime.dungeon_map_recover_graphics(self.ram);
    }
    pub fn toggle_star_tiles_and_advance(&mut self) {
        self.runtime.dungeon_map_step(self.ram, 0x1b);
    }

    pub fn render_text_post_death_save_options(&mut self) {
        self.post_death_save_options();
    }
    pub fn text_initialize(&mut self) {
        self.initialize();
    }
    pub fn text_initialize_init_module_state_loop(&mut self) {
        self.initialize_state_loop();
    }
    pub fn text_init_vwf_state(&mut self) {
        self.init_vwf_state();
    }
    #[must_use]
    pub fn text_decode_cmd(&self, a: u8, next: u8) -> DecodedCommand {
        decode_command(self.assets.dialogue_flags, a, next)
    }
    pub fn text_load_character_buffer(&mut self) {
        self.load_character_buffer();
    }
    pub fn text_write_player_name(&mut self, output: &mut Vec<u8>) {
        self.write_player_name(output);
    }
    #[must_use]
    pub fn text_filter_player_name_characters(a: u8) -> u8 {
        Self::filter_player_name_character(a)
    }
    pub fn render_text_draw_border(&mut self) {
        self.draw_border();
    }
    pub fn render_text_draw_border_incremental(&mut self) {
        self.draw_border_incremental();
    }
    pub fn render_text_draw_character_tilemap(&mut self) {
        self.build_character_tilemap();
    }
    pub fn render_text_draw_message_characters(&mut self) {
        self.draw_message_characters();
    }
    pub fn render_text_draw_finish(&mut self) {
        self.draw_finish();
    }
    pub fn render_text_draw_choose2_low_or3(&mut self) {
        self.choose_two(0);
    }
    pub fn render_text_draw_choose_item(&mut self) {
        self.choose_item();
    }
    pub fn render_text_find_y_item_previous(&mut self) {
        self.find_y_item_previous();
    }
    pub fn render_text_find_y_item_next(&mut self) {
        self.find_y_item_next();
    }
    pub fn render_text_draw_selected_y_item(&mut self) {
        self.runtime
            .draw_selected_item(self.ram, self.ram.read_u8(CHOICE));
    }
    pub fn render_text_draw_choose2_hi_or3(&mut self) {
        self.choose_two(2);
    }
    pub fn render_text_draw_choose3(&mut self) {
        self.choose_three();
    }
    pub fn render_text_draw_choose1_or2(&mut self) {
        self.choose_two(1);
    }
    pub fn render_text_draw_scroll(&mut self) -> bool {
        self.draw_scroll()
    }
    pub fn render_text_set_default_window_position(&mut self) {
        self.set_default_window_position();
    }
    pub fn render_text_draw_border_initialize(&mut self) {
        self.draw_border_initialize();
    }
    pub fn render_text_draw_border_row(&mut self, offset: usize, y: usize) -> usize {
        self.draw_border_row(offset, y)
    }
    pub fn text_build_character_tilemap(&mut self) {
        self.build_character_tilemap();
    }
    pub fn render_text_refresh(&mut self) {
        self.refresh();
    }
    pub fn text_generate_message_pointers(&mut self) {
        let mut pointer = 0x1c8000u32;
        for index in 0..398usize {
            if index == 359 {
                pointer = 0x0edf40;
            }
            let destination = 0x1c000 + index * 3;
            self.ram.write_u16(destination, pointer as u16);
            self.ram.write_u8(destination + 2, (pointer >> 16) as u8);
            pointer = pointer.wrapping_add(
                self.assets
                    .messages
                    .get(index)
                    .map_or(1, |message| message.len() as u32 + 1),
            );
        }
    }
    pub fn dung_map_lighten_up_map(&mut self) {
        self.dungeon_map_lighten();
    }
    pub fn dung_map_backup(&mut self) {
        self.dungeon_map_backup();
    }
    pub fn dung_map_fade_map_to_black(&mut self) {
        self.dungeon_map_fade();
    }
    pub fn dung_map_restore_old(&mut self) {
        self.dungeon_map_restore();
    }
    pub fn death_prep_faint(&mut self) {
        self.ram.write_u8(0x2f, 2);
        self.ram.write_u8(0x36b, 1);
        self.ram.write_u8(0x30d, 0);
        self.ram.write_u8(0x30a, 0);
        self.ram.write_u8(0x30b, 5);
        self.ram.write_u8(LINK_HEARTS_FILLER, 0);
        self.ram.write_u8(0xf36d, 0);
        self.runtime.link_reset_properties_c(self.ram);
        for address in [
            0x2f5, 0x351, 0x2e0, 0x48, 0x2ec, 0x4d, 0x46, 0x373, 0x2e1, 0x5e, 0x3f7,
        ] {
            self.ram.write_u8(address, 0);
        }
        if self.ram.read_u8(0xf357) != 0 {
            self.ram.write_u8(0x56, 0);
        }
        self.ram.write_u16(0x3f5, 0);
        let pan = self.runtime.calculate_link_sfx_pan(self.ram);
        self.ram.write_u8(SOUND1, 0x27 | pan);
        if (0..4).any(|i| self.ram.read_u8(0xf35c + i) == 6) {
            return;
        }
        self.ram.write_u16(0x430, 0);
    }
    pub fn display_select_menu(&mut self) {
        display_select_menu_core(self.ram);
    }
}

pub fn display_select_menu_core(ram: &mut Ram<'_>) {
    ram.write_u8(CHOICE_BACKUP, ram.read_u8(CHOICE));
    ram.write_u16(0x1cf0, 0x186);
    let module = ram.read_u8(MAIN_MODULE);
    crate::misc::main_show_text_message(ram);
    ram.write_u8(MAIN_MODULE, module);
    ram.write_u8(SUBSUBMODULE, 0);
    ram.write_u8(SUBMODULE, 11);
    ram.write_u8(SAVED_MODULE, module);
    ram.write_u8(MAIN_MODULE, 14);
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;
    struct Rt;
    impl MessagingRuntime for Rt {}
    fn fixture(f: impl FnOnce(&mut Messaging<'_, '_, '_, '_, Rt>)) {
        let mut bytes = [0; WRAM_SIZE];
        let mut sram = [0; 0x2000];
        let messages: [&[u8]; 9] = [&[], &[], &[], &[], &[], &[], b"A", b"B", b"C"];
        let dict: [&[u8]; 1] = [b"HI"];
        let assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dict,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        let mut m = Messaging {
            ram: &mut ram,
            sram: &mut sram,
            assets: &assets,
            runtime: &mut rt,
        };
        f(&mut m)
    }
    #[test]
    fn decoding_matches_us_and_eu_layouts() {
        assert_eq!(
            decode_command(0, b'A', 0),
            DecodedCommand::new(b'A', CMD_LETTER, false)
        );
        assert_eq!(
            decode_command(0, 0x6b, 2),
            DecodedCommand::new(2, CMD_WINDOW, true)
        );
        assert_eq!(
            decode_command(1, 0x80, 0),
            DecodedCommand::new(0, CMD_SCROLL, false)
        );
        assert_eq!(
            decode_command(1, 0x87, 0x3c),
            DecodedCommand::new(12, CMD_SPEED, true)
        );
    }
    #[test]
    fn line_layout_builds_exact_border_packet() {
        fixture(|m| {
            m.ram.write_u16(TEXT_TOPLEFT, 0x6125);
            m.draw_border_initialize();
            assert_eq!(m.draw_border_row(0, 0), 26);
            assert_eq!(m.ram.read_u16(VRAM_UPLOAD_DATA), 0x2561);
            assert_eq!(m.ram.read_u16(VRAM_UPLOAD_DATA + 4), 0x28f3);
            assert_eq!(m.ram.read_u16(VRAM_UPLOAD_DATA + 50), 0x68f3);
            assert_eq!(m.ram.read_u16(VRAM_UPLOAD_DATA + 52), 0xffff);
        });
    }
    #[test]
    fn three_way_choice_wraps_and_reloads_message() {
        fixture(|m| {
            m.ram.write_u8(CHOICE, 0);
            m.ram.write_u8(0xf4, 8);
            m.choose_three();
            assert_eq!(m.ram.read_u8(CHOICE), 2);
            assert_eq!(m.ram.read_u16(DIALOGUE_INDEX), 8);
            assert_eq!(m.ram.slice(MESSAGE_TEXT, 2), b"C\x7f");
            assert_eq!(m.ram.read_u8(SOUND2), 32);
        });
    }
    #[test]
    fn vwf_composites_both_bitplanes_and_carries_to_next_tile() {
        let mut bytes = [0; WRAM_SIZE];
        let mut sram = [0; 0x2000];
        let messages: [&[u8]; 1] = [&[]];
        let dict: [&[u8]; 0] = [];
        let mut font = [0u8; 0x900];
        // Character zero uses glyph rows 0 and 16. Set both planes in the
        // first row, with an eight-pixel width beginning seven pixels in.
        font[0] = 0x80;
        font[1] = 0x80;
        font[16 * 16] = 0x80;
        let widths = [8u8];
        let assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dict,
            messages: &messages,
            font_tiles: &font,
            font_widths: &widths,
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(VWF_ARR, 7);
        let mut rt = Rt;
        let mut m = Messaging {
            ram: &mut ram,
            sram: &mut sram,
            assets: &assets,
            runtime: &mut rt,
        };
        m.vwf_render_single(0);
        assert_eq!(m.ram.read_u8(MESSAGE_PIXELS), 1);
        assert_eq!(m.ram.read_u8(MESSAGE_PIXELS + 1), 1);
        assert_eq!(m.ram.read_u8(VWF_ARR + 1), 15);
        assert_eq!(m.ram.read_u8(MESSAGE_PIXELS + 0x150), 1);
    }

    #[test]
    fn save_game_writes_mirrors_and_subtractive_checksum() {
        fixture(|m| {
            m.sram[0x1ffe] = 2;
            m.ram.write_u16(0xf000, 1);
            m.ram.write_u16(0xf002, 2);
            m.save_game_file();
            assert_eq!(&m.sram[..4], &[1, 0, 2, 0]);
            assert_eq!(&m.sram[0xf00..0xf04], &[1, 0, 2, 0]);
            assert_eq!(m.ram.read_u16(0xf4fe), 0x5a57);
            assert_eq!(u16::from_le_bytes([m.sram[0x4fe], m.sram[0x4ff]]), 0x5a57);
        });
    }
    #[test]
    fn dungeon_room_quadrants_preserve_hidden_door_rules() {
        assert_eq!(
            Messaging::<Rt>::dungeon_room_quadrants(0, 0xf, false),
            [0xb61, 0x5361, 0x8b61, 0x8b62]
        );
        assert_eq!(
            Messaging::<Rt>::dungeon_room_quadrants(0, 0, false),
            [0xb00; 4]
        );
        assert_eq!(
            Messaging::<Rt>::dungeon_room_quadrants(0, 0, true),
            [0xf61, 0x4f61, 0x8f61, 0x8f62]
        );
    }

    #[test]
    fn dungeon_floor_backdrop_emits_terminated_vram_packet() {
        fixture(|m| {
            m.ram.write_u16(0x40c, 0);
            m.dungeon_map_draw_floors_backdrop();
            let offset = usize::from(m.ram.read_u16(0x1000));
            assert_eq!(m.ram.read_u8(VRAM_UPLOAD_DATA + offset), 0xff);
            assert_eq!(m.ram.read_u8(0x20d), 1);
            assert_eq!(m.ram.read_u8(NMI_LOAD_BG), 1);
        });
    }

    #[test]
    fn desert_iris_width_uses_the_exact_c_lookup_curves() {
        fixture(|m| {
            m.ram.write_u8(0x67c, 0x26);
            m.ram.write_u8(0x67a, 1);
            m.ram.write_u8(0x67e, 0);
            assert_eq!(m.desert_hdma_calculate_iris_shape_line(), (0xff, 37));
            m.ram.write_u8(0x67e, 1);
            assert_eq!(m.desert_hdma_calculate_iris_shape_line(), (0xff, 74));
            m.ram.write_u8(0x67a, 0x26);
            m.ram.write_u8(0x67e, 0);
            assert_eq!(m.desert_hdma_calculate_iris_shape_line(), (0, 0));
        });
    }

    #[test]
    fn save_menu_accepts_save_and_quit_like_the_c_state_machine() {
        fixture(|m| {
            m.ram.write_u8(SUBMODULE, 0);
            m.ram.write_u8(SUBSUBMODULE, 3);
            m.ram.write_u8(CHOICE, 1);
            m.ram.write_u8(MESSAGING_MODULE, 0xff);
            m.module0e_0b_save_menu();
            assert_eq!(m.ram.read_u8(MAIN_MODULE), 23);
            assert_eq!(m.ram.read_u8(SUBMODULE), 1);
            assert_eq!(m.ram.read_u8(SUBSUBMODULE), 0);
            assert_eq!(m.ram.read_u8(NMI_LOAD_BG), 1);
            assert_eq!(m.ram.read_u8(0x12d), 15);
        });
    }
}
