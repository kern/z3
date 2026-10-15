//! One-for-one translation of `src/sprite_main.c` definitions beginning in
//! source lines 1 through 7000.
//!
//! Sprite-owned WRAM mutations stay in this module. Calls owned by other
//! translation units cross the explicit [`SpriteMainBank0Runtime`] boundary.

use crate::sprite_state::SpriteField;
use crate::{
    ram::Ram,
    sprite::{DrawMultipleData, PrepOamCoordsRet, ProjectSpeedRet, SpriteSpawnInfo},
};

const GARNISH_TYPE: usize = 0x1f800;
const GARNISH_Y_LO: usize = 0x1f81e;
const GARNISH_X_LO: usize = 0x1f83c;
const GARNISH_Y_HI: usize = 0x1f85a;
const GARNISH_X_HI: usize = 0x1f878;

const LINK_Y: usize = 0x20;
const LINK_X: usize = 0x22;
const MAIN_MODULE_INDEX: usize = 0x10;
const FRAME_COUNTER: usize = 0x1a;
const PLAYER_IS_INDOORS: usize = 0x1b;
const LINK_ACTUAL_VEL_X: usize = 0x28;
const LINK_ACTUAL_VEL_Y: usize = 0x27;
const BUTTON_B_FRAMES: usize = 0x3c;
const PLAYER_OAM_Y_OFFSET: usize = 0x44;
const BITMASK_OF_DRAGSTATE: usize = 0x48;
const LINK_INCAPACITATED_TIMER: usize = 0x46;
const LINK_SPEED_SETTING: usize = 0x5e;
const LINK_CAPE_MODE: usize = 0x55;
const LINK_PLAYER_HANDLER_STATE: usize = 0x5d;
const FLAG_UPDATE_CGRAM_IN_NMI: usize = 0x15;
const TS_COPY: usize = 0x1d;
const LINK_DIRECTION_FACING: usize = 0x2f;
const OAM_CUR_PTR: usize = 0x90;
const OAM_EXT_CUR_PTR: usize = 0x92;
const BG2_HOFS_COPY2: usize = 0xe2;
const BG2_VOFS_COPY2: usize = 0xe8;
const LINK_IS_ON_LOWER_LEVEL: usize = 0xee;
const FILTERED_JOYPAD_L: usize = 0xf6;
const FILTERED_JOYPAD_H: usize = 0xf4;
const JOYPAD1_H_LAST: usize = 0xf0;
const JOYPAD1_L_LAST: usize = 0xf2;
const CGADSUB_COPY: usize = 0x9a;
const COLDATA_COPY0: usize = 0x9c;
const COLDATA_COPY1: usize = 0x9d;
const DUNGEON_ROOM_INDEX: usize = 0xa0;
const OVERWORLD_SCREEN_INDEX: usize = 0x8a;
const LAST_LIGHT_VS_DARK_WORLD: usize = 0x7b;
const SUBSUBMODULE_INDEX: usize = 0xb0;
const MUSIC_CONTROL: usize = 0x12c;
const SOUND_EFFECT_AMBIENT: usize = 0x12d;
const SOUND_EFFECT_1: usize = 0x12e;
const SOUND_EFFECT_2: usize = 0x12f;
const LAST_MUSIC_CONTROL: usize = 0x133;
const BG1_X_OFFSET: usize = 0x11a;
const BG1_Y_OFFSET: usize = 0x11c;
const FLAG_IS_LINK_IMMOBILIZED: usize = 0x2e4;
const LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE: usize = 0x2db;
const BYTE_7E02F0: usize = 0x2f0;
const ITEM_RECEIPT_METHOD: usize = 0x2e9;
const PLAYER_HANDLER_TIMER: usize = 0x300;
const LINK_ITEM_IN_HAND: usize = 0x301;
const LINK_STATE_BITS: usize = 0x308;
const LINK_PICKING_THROW_STATE: usize = 0x309;
const LINK_IS_BUNNY_MIRROR: usize = 0x2e0;
const TAGALONG_VAR5: usize = 0x2f9;
const TAGALONG_VAR2: usize = 0x2cf;
const WORD_7E02CD: usize = 0x2cd;
const COUNTDOWN_FOR_BLINK: usize = 0x31f;
const LINK_IS_RUNNING: usize = 0x372;
const LINK_UNK_MASTER_SWORD: usize = 0x377;
const BYTE_7E0379: usize = 0x379;
const PLAYER_SLEEP_IN_BED_STATE: usize = 0x37c;
const LINK_POSE_DURING_OPENING: usize = 0x37d;
const LINK_DISABLE_SPRITE_DAMAGE: usize = 0x37b;
const LINK_POSITION_MODE: usize = 0x37a;
const DUNG_SAVEGAME_STATE_BITS: usize = 0x402;
const CUR_PALACE_INDEX_X2: usize = 0x40c;
const DUNGEON_ROOM_INDEX2: usize = 0x48e;
const DUNG_DOOR_OPENED: usize = 0x400;
const SUBMODULE_INDEX: usize = 0x11;
const ENHANCED_FEATURES0: usize = 0x64c;
const DUNG_FLAG_STATECHANGE_WATERPUZZLE: usize = 0x642;
const CHOICE_IN_MULTISELECT_BOX: usize = 0x1ce8;
const HUD_CUR_ITEM: usize = 0x202;
const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;
const DIALOGUE_NUMBER: usize = 0x1cf2;
const LINK_X_PREVIOUS: usize = 0xfc2;
const LINK_Y_PREVIOUS: usize = 0xfc4;
const BYTE_7E0FF8: usize = 0xff8;
const CUR_SPRITE_X: usize = 0xfd8;
const CUR_SPRITE_Y: usize = 0xfda;
const LINK_RUPEES_GOAL: usize = 0xf360;
const LINK_HEARTS_FILLER: usize = 0xf372;
const LINK_WHICH_PENDANTS: usize = 0xf374;
const LINK_NUM_ARROWS: usize = 0xf377;
const LINK_HEART_PIECES: usize = 0xf36b;
const LINK_NUM_KEYS: usize = 0xf36f;
const SAVEGAME_IS_DARKWORLD: usize = 0xf3ca;
const LINK_ITEM_BOW: usize = 0xf340;
const SAVE_DUNG_INFO: usize = 0xf000;
const LINK_ITEM_MUSHROOM: usize = 0xf344;
const LINK_ITEM_ICE_ROD: usize = 0xf346;
const LINK_ITEM_TORCH: usize = 0xf34a;
const LINK_ITEM_BOOK_OF_MUDORA: usize = 0xf34e;
const LINK_ITEM_BOOTS: usize = 0xf355;
const LINK_ITEM_FLIPPERS: usize = 0xf356;
const LINK_ITEM_BOMBOS_MEDALLION: usize = 0xf347;
const LINK_ITEM_ETHER_MEDALLION: usize = 0xf348;
const LINK_SHIELD_TYPE: usize = 0xf35a;
const LINK_SWORD_TYPE: usize = 0xf359;
const SAVE_OW_EVENT_INFO: usize = 0xf280;
const WHICH_STARTING_POINT: usize = 0xf3c8;
const SAVEGAME_MAP_ICONS_INDICATOR: usize = 0xf3c7;
const SRAM_PROGRESS_INDICATOR: usize = 0xf3c5;
const SRAM_PROGRESS_FLAGS: usize = 0xf3c6;
const SRAM_PROGRESS_INDICATOR_3: usize = 0xf3c9;
const PALETTE_FILTER_COUNTDOWN: usize = 0xc007;
const FOLLOWER_INDICATOR: usize = 0xf3cc;
const FOLLOWER_DROPPED: usize = 0xf3d3;
const LINK_PREVENT_FROM_MOVING: usize = 0xb7b;
const DRAG_PLAYER_X: usize = 0xb7c;
const DRAG_PLAYER_Y: usize = 0xb7e;
const BYTE_7E0B88: usize = 0xb88;
const ARCHERY_GAME_ARROWS_LEFT: usize = 0xb99;
const ARCHERY_GAME_OUT_OF_ARROWS: usize = 0xb9a;
const SPRITE_LIMIT_INSTANCE: usize = 0xb6a;
const OVERLORD_X_HI: usize = 0xb10;
const OVERLORD_X_LO: usize = 0xb08;
const OVERLORD_Y_LO: usize = 0xb18;
const OVERLORD_Y_HI: usize = 0xb20;
const OVERLORD_GEN2: usize = 0xb30;
const OVERLORD_FLOOR: usize = 0xb40;
const ACTIVATE_BOMB_TRAP_OVERLORD: usize = 0xcf4;
const ANCILLA_TYPE: usize = 0xc4a;
const LINK_BOTTLE_INFO: usize = 0xf35c;
const GARNISH_ACTIVE: usize = 0xfb4;
const DUNGMAP_VAR7: usize = 0xfa8;
const DUNGMAP_VAR8: usize = 0xfaa;
const CUR_OBJECT_INDEX: usize = 0xfa0;
const SPRITE_ALERT_FLAG: usize = 0xfdc;
const BYTE_7E0FDE: usize = 0xfde;
const OVERWORLD_AREA_INDEX: usize = 0x40a;
const OVERWORLD_OFFSET_BASE_Y: usize = 0x708;
const OVERWORLD_OFFSET_MASK_Y: usize = 0x70a;
const OVERWORLD_OFFSET_BASE_X: usize = 0x70c;
const OVERWORLD_OFFSET_MASK_X: usize = 0x70e;
const FLAG_OVERWORLD_AREA_DID_CHANGE: usize = 0xabf;
const SPRITE_TILETYPE: usize = 0xfa5;
const LOAD_CHR_HALFSLOT_EVEN_ODD: usize = 0xaaa;
const FLAG_UNK1: usize = 0xfc1;
const BYTE_7E0FC6: usize = 0xfc6;
const IS_IN_DARK_WORLD: usize = 0xfff;
const GARNISH_COUNTDOWN: usize = 0x1f90e;
const GARNISH_SPRITE: usize = 0x1f92c;
const BIRD_TRAVEL_X_LO: usize = 0x1ab0;
const BIRD_TRAVEL_X_HI: usize = 0x1ac0;
const BIRD_TRAVEL_Y_LO: usize = 0x1ad0;
const BIRD_TRAVEL_Y_HI: usize = 0x1ae0;
const TAGALONG_LAYERBITS: usize = 0x1a64;
const MOLDORM_X_LO: usize = 0x1fc00;
const MOLDORM_X_HI: usize = 0x1fc80;
const MOLDORM_Y_LO: usize = 0x1fd00;
const MOLDORM_Y_HI: usize = 0x1fd80;
const BEAMOS_X_LO: usize = 0x1fd80;
const BEAMOS_X_HI: usize = 0x1fe00;
const BEAMOS_Y_LO: usize = 0x1fe80;
const BEAMOS_Y_HI: usize = 0x1ff00;
const BYTE_7FFE01: usize = 0x1fe01;

const GIANT_MOLDORM_SEGMENT: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4084,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40a4,
        ext: 2,
    },
];

pub const KEESE_X_VELOCITY: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];

const SOLDIER_DRAW1_CHAR: [u8; 4] = [0x42, 0x42, 0x40, 0x44];
const SOLDIER_DRAW1_FLAGS: [u8; 4] = [0x40, 0, 0, 0];
const SOLDIER_DRAW1_Y: [i8; 26] = [
    7, 8, 7, 8, 8, 7, 8, 7, 8, 7, 8, 8, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
];
const SOLDIER_DRAW2_X: [i8; 104] = [
    -4, 4, 10, 10, -4, 4, 10, 10, -4, 4, 10, 10, -4, 4, 10, 10, -4, -4, 0, 0, -4, -4, 0, 0, -3, -3,
    0, 0, -3, -3, -4, 4, -3, -3, -4, 4, -3, -3, -4, 4, -3, -3, -4, 4, 12, 12, 0, 0, 12, 12, 0, 0,
    11, 11, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, 4,
    0, 0, -4, 4, 0, 0, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const SOLDIER_DRAW2_Y: [i8; 104] = [
    0, 0, 2, 10, 0, 0, 2, 10, 0, 0, 1, 9, 0, 0, 2, 10, -2, 6, 1, 1, -2, 6, 2, 2, -2, 6, 1, 1, -5,
    3, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, -5, 3, 0, 0, -2, 6, 1, 1, -2, 6, 2, 2, -2, 6, 1, 1, 0, 0, 8,
    8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8,
    8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8,
];
const SOLDIER_DRAW2_CHARACTER: [u8; 104] = [
    0x48, 0x49, 0x6d, 0x7d, 0x49, 0x48, 0x6d, 0x7d, 0x46, 0x46, 0x6d, 0x7d, 0x4b, 0x46, 0x6d, 0x7d,
    0x4d, 0x5d, 0x4e, 0x4e, 0x4d, 0x5d, 0x60, 0x60, 0x4d, 0x5d, 0x62, 0x62, 0x6d, 0x7d, 0x64, 0x64,
    0x6d, 0x7d, 0x66, 0x67, 0x6d, 0x7d, 0x67, 0x66, 0x6d, 0x7d, 0x64, 0x69, 0x4d, 0x5d, 0x4e, 0x4e,
    0x4d, 0x5d, 0x60, 0x60, 0x4d, 0x5d, 0x62, 0x62, 2, 3, 0x20, 0x20, 2, 0xc, 0x20, 0x20, 2, 0xc,
    0x20, 0x20, 8, 8, 0x20, 0x20, 0xe, 0xe, 0x20, 0x20, 0xe, 0xe, 0x20, 0x20, 5, 6, 0x20, 0x20,
    0x22, 6, 0x20, 0x20, 0x22, 6, 0x20, 0x20, 8, 8, 0x20, 0x20, 0xe, 0xe, 0x20, 0x20, 0xe, 0xe,
    0x20, 0x20,
];
const SOLDIER_DRAW2_FLAGS: [u8; 104] = [
    0, 0, 0, 0, 0x40, 0x40, 0, 0, 0, 0x40, 0, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40,
];
const SOLDIER_DRAW2_BIG: [u8; 104] = [
    2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2,
    0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2,
];
const SOLDIER_DRAW2_OAM_INDEX: [u8; 4] = [12, 12, 12, 4];
const SOLDIER_DRAW3_X: [i8; 28] = [
    -3, -3, -4, -4, -4, -4, -4, -4, -11, -3, -11, -3, -16, -8, 12, 12, 12, 12, 12, 12, 12, 12, 21,
    13, 21, 13, 24, 16,
];
const SOLDIER_DRAW3_Y: [i8; 28] = [
    11, 19, 11, 19, 10, 18, 14, 22, 8, 8, 8, 8, 6, 6, -10, -2, -9, -1, -9, -1, -16, -8, 8, 8, 8, 8,
    6, 6,
];
const SOLDIER_DRAW3_CHARACTER: [u8; 28] = [
    0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x6c, 0x7c, 0x6c, 0x7c, 0x6c, 0x7c, 0x6b, 0x7b,
    0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6c, 0x7c, 0x6c, 0x7c, 0x6c, 0x7c,
];
const SOLDIER_DRAW3_FLAGS: [u8; 28] = [
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40,
];
const SOLDIER_DRAW3_OAM_INDEX: [u8; 4] = [4, 4, 4, 20];
pub const KEESE_Y_VELOCITY: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];
pub const ZAZAK_Y_VELOCITY: [i8; 4] = [0, 0, 16, -16];
pub const FLUTE_BOY_ANIMAL_X_VELOCITY: [i8; 4] = [16, -16, 0, 0];
pub const DESERT_BARRIER_X_VELOCITY: [i8; 4] = [16, -16, 0, 0];
pub const DESERT_BARRIER_Y_VELOCITY: [i8; 4] = [0, 0, 16, -16];
pub const CRYSTAL_SWITCH_PALETTE: [u8; 2] = [2, 4];
pub const ZAZAK_DIRECTION: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
pub const BAD_PULL_DOWN_SWITCH_X: [i8; 5] = [-4, 12, 0, -4, 4];
pub const BAD_PULL_DOWN_SWITCH_Y: [i8; 5] = [-3, -3, 0, 5, 5];
pub const BAD_PULL_DOWN_SWITCH_CHAR: [u8; 5] = [0xd2, 0xd2, 0xc4, 0xe4, 0xe4];
pub const BAD_PULL_DOWN_SWITCH_FLAGS: [u8; 5] = [0x40, 0, 0, 0x40, 0];
pub const BAD_PULL_DOWN_SWITCH_BIG: [u8; 5] = [0, 0, 2, 2, 2];
pub const BAD_PULL_SWITCH_TAB5: [u8; 6] = [0, 1, 2, 3, 4, 5];
pub const BAD_PULL_SWITCH_TAB4: [u8; 12] = [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5];
pub const THIEF_GRAPHICS: [u8; 12] = [11, 8, 2, 5, 9, 6, 0, 3, 10, 7, 1, 4];
pub const WISH_POND_X: [u8; 8] = [0, 4, 8, 12, 16, 20, 24, 0];
pub const WISH_POND_Y: [u8; 8] = [0, 8, 16, 24, 32, 40, 4, 36];
pub const WISH_POND_OAM_FLAGS: [u8; 76] = [
    5, 0xff, 5, 5, 5, 5, 5, 1, 2, 1, 1, 1, 2, 2, 2, 4, 4, 4, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 4, 4, 2,
    1, 6, 1, 2, 1, 2, 2, 1, 2, 2, 4, 1, 1, 4, 2, 1, 4, 2, 2, 4, 4, 4, 2, 1, 4, 1, 2, 2, 1, 2, 2, 1,
    1, 4, 4, 1, 2, 2, 4, 4, 4, 2, 5, 2, 1,
];
pub const WISH_POND_ITEM_OFFSETS: [u8; 32] = [
    0, 4, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 20, 21, 22, 22, 23, 24, 25, 28, 30, 31, 32, 33,
    33, 37, 40, 42, 42, 42, 42,
];
pub const WISH_POND_ITEM_DATA: [u8; 50] = [
    0x3a, 0x3a, 0x3b, 0x3b, 0x0c, 0x2a, 0x0a, 0x27, 0x29, 0x0d, 0x07, 0x08, 0x0f, 0x10, 0x11, 0x12,
    0x09, 0x13, 0x14, 0x4a, 0x21, 0x1d, 0x15, 0x18, 0x19, 0x31, 0x1a, 0x1a, 0x1b, 0x1c, 0x4b, 0x1e,
    0x1f, 0x49, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x22, 0x23, 0x29, 0x16, 0x2b, 0x2c, 0x2d, 0x3d,
    0x3c, 0x48,
];
pub const SINUS_LOOKUP_TABLE: [u16; 256] = [
    0, 3, 6, 9, 12, 15, 18, 21, 25, 28, 31, 34, 37, 40, 40, 46, 49, 53, 56, 59, 62, 65, 68, 71, 74,
    77, 80, 83, 86, 89, 92, 95, 97, 100, 103, 106, 109, 112, 115, 117, 120, 123, 126, 128, 131,
    134, 136, 139, 142, 144, 147, 149, 152, 155, 157, 159, 162, 164, 167, 169, 171, 174, 176, 178,
    181, 183, 185, 187, 189, 191, 193, 195, 197, 199, 201, 203, 205, 207, 209, 211, 212, 214, 216,
    217, 219, 221, 222, 224, 225, 227, 228, 230, 231, 232, 234, 235, 236, 237, 238, 239, 241, 242,
    243, 244, 244, 245, 246, 247, 248, 249, 249, 250, 251, 251, 252, 252, 253, 253, 254, 254, 254,
    255, 255, 255, 255, 255, 255, 255, 256, 255, 255, 255, 255, 255, 255, 255, 254, 254, 254, 253,
    253, 252, 252, 251, 251, 250, 249, 249, 248, 247, 246, 245, 244, 244, 243, 242, 241, 239, 238,
    237, 236, 235, 234, 232, 231, 230, 228, 227, 225, 224, 222, 221, 219, 217, 216, 214, 212, 211,
    209, 207, 205, 203, 201, 199, 197, 195, 193, 191, 189, 187, 185, 183, 181, 178, 176, 174, 171,
    169, 167, 164, 162, 159, 157, 155, 152, 149, 147, 144, 142, 139, 136, 134, 131, 128, 126, 123,
    120, 117, 115, 112, 109, 106, 103, 100, 97, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59,
    56, 53, 49, 46, 43, 40, 37, 34, 31, 28, 25, 21, 18, 15, 12, 9, 6, 3,
];
pub const SOLDIER_DRAW_SHADOW: [u8; 4] = [0xc, 0xc, 0xa, 0xa];
pub const HOBO_DRAW_DATA: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00ab,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: -11,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00ab,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
];
pub const WATER_TURBULENCE_DRAW_DATA: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: -10,
        y: 14,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 16,
        char_flags: 0x40c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: 18,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 18,
        char_flags: 0x40c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 16,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 10,
        y: 14,
        char_flags: 0x40c0,
        ext: 2,
    },
];

// Source tables declared before the two 243-entry dispatch tables but consumed
// by definitions in later source banks. They live here because their C
// definitions are part of this bank's line range.
pub const PRIEST_DRAW_DATA: [(i8, i8, u16, u8); 20] = [
    (0, -8, 0x0e20, 2),
    (0, 0, 0x0e26, 2),
    (0, -8, 0x0e20, 2),
    (0, 0, 0x4e26, 2),
    (0, -8, 0x0e0e, 2),
    (0, 0, 0x0e24, 2),
    (0, -8, 0x0e0e, 2),
    (0, 0, 0x0e24, 2),
    (0, -8, 0x0e22, 2),
    (0, 0, 0x0e28, 2),
    (0, -8, 0x0e22, 2),
    (0, 0, 0x0e2a, 2),
    (0, -8, 0x4e22, 2),
    (0, 0, 0x4e28, 2),
    (0, -8, 0x4e22, 2),
    (0, 0, 0x4e2a, 2),
    (-7, 1, 0x0e0a, 2),
    (3, 3, 0x0e0c, 2),
    (-7, 1, 0x0e0a, 2),
    (3, 3, 0x0e0c, 2),
];
pub const CRYSTAL_MAIDEN_DMA: [u8; 16] = [
    0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40, 0x60,
];
pub const CRYSTAL_MAIDEN_SPRITE_DATA: [(i8, i8, u16, u8); 16] = [
    (1, -7, 0x0120, 2),
    (1, 3, 0x0122, 2),
    (1, -7, 0x0120, 2),
    (1, 3, 0x4122, 2),
    (1, -7, 0x0120, 2),
    (1, 3, 0x0122, 2),
    (1, -7, 0x0120, 2),
    (1, 3, 0x4122, 2),
    (1, -7, 0x0120, 2),
    (1, 3, 0x0122, 2),
    (1, -7, 0x0120, 2),
    (1, 3, 0x0122, 2),
    (1, -7, 0x4120, 2),
    (1, 3, 0x4122, 2),
    (1, -7, 0x4120, 2),
    (1, 3, 0x4122, 2),
];
pub const HEART_REFILL_ACCEL_X: [i8; 2] = [1, -1];
pub const HEART_REFILL_VELOCITY_TARGET: [i8; 2] = [10, -10];
pub const THROWABLE_SCENERY_CHARACTER: [u8; 12] = [
    0x42, 0x44, 0x46, 0, 0x46, 0x44, 0x42, 0x44, 0x44, 0, 0x46, 0x44,
];
pub const THROWABLE_SCENERY_FLAGS: [u8; 9] = [0xc, 0xc, 0xc, 0, 0, 0, 0xb0, 0x08, 0xb4];
pub const THROWABLE_SCENERY_LARGE_X: [i16; 4] = [-8, 8, -8, 8];
pub const THROWABLE_SCENERY_LARGE_Y: [i16; 4] = [-14, -14, 2, 2];
pub const THROWABLE_SCENERY_LARGE_FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];
pub const THROWABLE_SCENERY_LARGE_X2: [i16; 3] = [-6, 0, 6];
pub const THROWABLE_SCENERY_LARGE_OAM_FLAGS: [u8; 2] = [0xc, 0];
pub const SCATTER_DEBRIS_X: [i8; 4] = [-8, 8, -8, 8];
pub const SCATTER_DEBRIS_Y: [i8; 4] = [-8, -8, 8, 8];
pub const MOVABLE_MANTLE_X: [u8; 6] = [0, 0x10, 0x20, 0, 0x10, 0x20];
pub const MOVABLE_MANTLE_Y: [u8; 6] = [0, 0, 0, 0x10, 0x10, 0x10];
pub const MOVABLE_MANTLE_CHARACTER: [u8; 6] = [0xc, 0xe, 0xc, 0x2c, 0x2e, 0x2c];
pub const MOVABLE_MANTLE_FLAGS: [u8; 6] = [0x31, 0x31, 0x71, 0x31, 0x31, 0x71];
pub const SPRITE_SIMPLIFIED_TILE_ATTRIBUTE: [u8; 256] = [
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];
pub const LARGE_SHADOW_DRAW_DATA: [(i8, i8, u16, u8); 15] = [
    (-6, 19, 0x086c, 2),
    (0, 19, 0x086c, 2),
    (6, 19, 0x086c, 2),
    (-5, 19, 0x086c, 2),
    (0, 19, 0x086c, 2),
    (5, 19, 0x086c, 2),
    (-4, 19, 0x086c, 2),
    (0, 19, 0x086c, 2),
    (4, 19, 0x086c, 2),
    (-3, 19, 0x086c, 2),
    (0, 19, 0x086c, 2),
    (3, 19, 0x086c, 2),
    (-2, 19, 0x086c, 2),
    (0, 19, 0x086c, 2),
    (2, 19, 0x086c, 2),
];
pub const HELMASAUR_SEQUENCE: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 6, 5, 4, 3, 2, 1,
];
pub const FLUTE_BOY_ANIMAL_OAM_FLAGS: [u8; 2] = [0x40, 0];
pub const FLUTE_BOY_ANIMAL_GRAPHICS: [u8; 3] = [0, 1, 2];
pub const GIBO_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
pub const GIBO_OAM_FLAGS_2: [u8; 2] = [11, 7];
pub const BLIND_HEAD_DRAW_CHARACTER: [u8; 16] = [
    0x86, 0x86, 0x84, 0x82, 0x80, 0x82, 0x84, 0x86, 0x86, 0x86, 0x88, 0x8a, 0x8c, 0x8a, 0x88, 0x86,
];
pub const BLIND_HEAD_DRAW_FLAGS: [u8; 16] = [
    0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0,
];
pub const GANON_G_FUNCTION_2: [u8; 16] = [8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1];
pub const SPRITE_TUTORIAL_ENTITIES: [u8; 4] = [2, 1, 0, 3];
pub const SPAWN_BEE_INITIAL_DELAY: [u8; 4] = [64, 64, 255, 255];
pub const SPAWN_BEE_INITIAL_VELOCITY: [i8; 8] = [15, 5, -5, -15, 20, 10, -10, -20];
pub const GANON_DRAW_X: [i8; 204] = [
    18, -8, 8, -8, 8, -18, -18, 18, -8, 8, -8, 8, 18, -8, 8, -8, 8, -18, -18, 18, -8, 8, -8, 8, 16,
    -8, 8, -8, 8, -18, -18, 16, -8, 8, -11, 11, 16, -8, 8, -8, 8, -18, -18, 16, -8, 8, -11, 11, 16,
    -8, 8, -8, 8, -18, -18, 16, -8, 8, -11, 11, 18, -8, 8, -8, 8, -18, -18, 18, -8, 8, -8, 8, 18,
    -8, 8, -8, 8, -18, -18, 18, -8, 8, -8, 8, 18, -8, 8, -8, 8, -18, -18, 18, -8, 8, -11, 11, -8,
    8, -8, 8, -8, 8, -8, 8, -18, -18, 18, 18, -8, 8, -8, 8, -8, 8, -8, 8, -18, -18, 18, 18, -8, 8,
    -8, 8, -8, 8, -10, 10, -18, -18, 18, 18, -8, 8, -8, 8, -8, 8, -10, 10, -18, -18, 18, 18, -8, 8,
    -8, 8, -8, 8, -10, 10, -18, -18, 18, 18, -8, 8, -8, 8, -8, 8, -8, 8, -18, -18, 18, 18, -8, 8,
    -8, 8, -8, 8, -8, 8, -18, -18, 18, 18, -7, -8, 8, -8, 8, -9, 8, -14, -14, -8, 8, 8, -8, 8, -8,
    8, -18, -18, 18, 18, -8, 8, -11, 11,
];
pub const GANON_DRAW_Y: [i8; 204] = [
    -8, -16, -16, -13, -13, -9, -1, -16, 3, 3, 8, 8, -8, -16, -16, -13, -13, -9, -1, -16, 3, 3, 8,
    8, 5, -10, -10, -13, -13, -7, 1, -3, 3, 3, 8, 8, 5, -10, -10, -13, -13, -7, 1, -3, 3, 3, 8, 8,
    5, -10, -10, -13, -13, -7, 1, -3, 3, 3, 8, 8, -1, -16, -16, -13, -13, -9, -1, -9, 3, 3, 8, 8,
    -10, -16, -16, -13, -13, -18, -10, -18, 3, 3, 8, 8, 1, -10, -10, -13, -13, -7, 1, -7, 3, 3, 8,
    8, -12, -12, 4, 4, -18, -18, 10, 10, -16, -8, -4, 4, -12, -12, 4, 4, -18, -18, 10, 10, -16, -8,
    -4, 4, -12, -12, 4, 4, -12, -12, 10, 10, -4, 4, -4, 4, -12, -12, 4, 4, -12, -12, 10, 10, -4, 4,
    -4, 4, -12, -12, 4, 4, -12, -12, 10, 10, -4, 4, -4, 4, -12, -12, 4, 4, -18, -18, 10, 10, -4, 4,
    -4, 4, -12, -12, 4, 4, -18, -18, 10, 10, -16, -8, -16, -8, -7, -12, -12, 4, 4, 7, 13, -11, -4,
    -16, -16, -16, -10, -10, -13, -13, -7, -7, -7, -7, 3, 3, 8, 8,
];
pub const GANON_DRAW_CHARACTER: [u8; 204] = [
    0x16, 0, 0, 2, 2, 8, 0x18, 6, 0x22, 0x22, 0x20, 0x20, 0x46, 0, 0, 2, 2, 8, 0x18, 0x36, 0x22,
    0x22, 0x20, 0x20, 0x1a, 0, 0, 4, 4, 0x38, 0x48, 0xa, 0x24, 0x24, 0x20, 0x20, 0x1a, 0x40, 0x42,
    4, 4, 0x38, 0x48, 0xa, 0x24, 0x24, 0x20, 0x20, 0x1a, 0x42, 0x40, 4, 4, 0x38, 0x48, 0xa, 0x24,
    0x24, 0x20, 0x20, 0x18, 0, 0, 2, 2, 8, 0x18, 8, 0x22, 0x22, 0x20, 0x20, 0x16, 0x6a, 0x6a, 0xe,
    0xe, 6, 0x16, 6, 0x22, 0x22, 0x20, 0x20, 0x48, 0, 0, 4, 4, 0x38, 0x48, 0x38, 0x24, 0x24, 0x20,
    0x20, 0x4e, 0x4e, 0x6e, 0x6e, 0x6c, 0x6c, 0xa2, 0xa2, 0xc, 0x1c, 0x3c, 0x4c, 0x4e, 0x4e, 0x6e,
    0x6e, 0x6c, 0x6c, 0xa2, 0xa2, 0x3a, 0x4a, 0x3c, 0x4c, 0x84, 0x84, 0xa4, 0xa4, 0xa0, 0xa0, 0xa2,
    0xa2, 0x3c, 0x4c, 0x3c, 0x4c, 0x84, 0x84, 0xa4, 0xa4, 0x80, 0x82, 0xa2, 0xa2, 0x3c, 0x4c, 0x3c,
    0x4c, 0x84, 0x84, 0xa4, 0xa4, 0x82, 0x80, 0xa2, 0xa2, 0x3c, 0x4c, 0x3c, 0x4c, 0x4e, 0x4e, 0x6e,
    0x6e, 0x6c, 0x6c, 0xa2, 0xa2, 0x3c, 0x4c, 0x3c, 0x4c, 0x4e, 0x4e, 0x6e, 0x6e, 0x6c, 0x6c, 0xa2,
    0xa2, 0xc, 0x1c, 0xc, 0x1c, 0xe0, 0xc6, 0xc8, 0xe6, 0xe8, 0x20, 0x20, 8, 0x18, 0xc0, 0xc2,
    0xc2, 0, 0, 0xce, 0xce, 0xec, 0xec, 0xec, 0xec, 0xee, 0xee, 0xc4, 0xc4,
];
pub const GANON_DRAW_FLAGS: [u8; 204] = [
    0x4c, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0x4c, 0x4c, 0xc, 0x4c, 0xa, 0x4a,
    0xc, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0x4c, 0x4c, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0xc, 0x4c, 0xa, 0x4a,
    0xc, 0x4c, 0x4c, 0xc, 0xc, 0xa, 0x4a, 0xc, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0x4c, 0x4c, 0x4c, 0x4c,
    0xa, 0x4a, 0xc, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0x4c, 0x4c, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0xc, 0x4c,
    0xa, 0x4a, 0xc, 0x4c, 0x4c, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0x4c, 0x4c,
    0xc, 0x4c, 0xa, 0x4a, 0xc, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0x4c, 0xa, 0x4a, 0xa, 0x4a, 0xc, 0x4c,
    0xc, 0x4c, 0xc, 0xc, 0x4c, 0x4c, 0xa, 0x4a, 0xa, 0x4a, 0xc, 0x4c, 0xc, 0x4c, 0xc, 0xc, 0x4c,
    0x4c, 0xa, 0x4a, 0xa, 0x4a, 0xc, 0x4c, 0xc, 0x4c, 0xc, 0xc, 0x4c, 0x4c, 0xa, 0x4a, 0xa, 0x4a,
    0xc, 0xc, 0xc, 0x4c, 0xc, 0xc, 0x4c, 0x4c, 0xa, 0x4a, 0xa, 0x4a, 0x4c, 0x4c, 0xc, 0x4c, 0xc,
    0xc, 0x4c, 0x4c, 0xa, 0x4a, 0xa, 0x4a, 0xc, 0x4c, 0xc, 0x4c, 0xc, 0xc, 0x4c, 0x4c, 0xa, 0x4a,
    0xa, 0x4a, 0xc, 0x4c, 0xc, 0x4c, 0xc, 0xc, 0x4c, 0x4c, 0xc, 0xa, 0xa, 0xa, 0xa, 0xc, 0x4c, 0xc,
    0xc, 0xc, 0xc, 0xc, 0xc, 0x4c, 0xa, 0x4a, 0xc, 0xc, 0x4c, 0x4c, 0xa, 0x4a, 0xc, 0x4c,
];
pub const GANON_DRAW_CHARACTER_2: [u8; 12] = [
    0x40, 0x42, 0, 0, 0x42, 0x40, 0x82, 0x80, 0xa0, 0xa0, 0x80, 0x82,
];
pub const GANON_DRAW_FLAGS_2: [u8; 12] = [0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0, 0];

// These two unused C globals are retained for source-inventory parity.
pub static mut CTR: i32 = 0;
pub static mut FOO: bool = false;

pub trait SpriteMainBank0Runtime {
    fn sprite_active_cross_bank(&mut self, _ram: &mut Ram<'_>, _slot: usize, _kind: u8) {}

    fn sprite_prep_cross_bank(&mut self, _ram: &mut Ram<'_>, _slot: usize, _kind: u8) {}

    fn draw_moldorm_segment(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _pieces: &[DrawMultipleData; 4],
    ) {
    }

    fn draw_single_large(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_return_if_lifted(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn show_solicited_message(&mut self, _ram: &mut Ram<'_>, _slot: usize, _message: u16) -> u16 {
        0
    }

    fn show_message_unconditional(&mut self, _ram: &mut Ram<'_>, _message: u16) {}

    fn fortune_teller_perform_pseudo_science(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_check_if_link_is_busy(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }

    fn show_message_on_contact(&mut self, _ram: &mut Ram<'_>, _slot: usize, _message: u16) -> u16 {
        0
    }

    fn link_reset_properties_a(&mut self, _ram: &mut Ram<'_>) {}

    fn add_tossed_pond_item(&mut self, _ram: &mut Ram<'_>, _item: u8) {}

    fn hud_refresh_icon(&mut self, _ram: &mut Ram<'_>) {}

    fn sprite_spawn_dynamically(
        &mut self,
        _ram: &mut Ram<'_>,
        _parent: usize,
        _kind: u8,
    ) -> Option<(usize, SpriteSpawnInfo)> {
        None
    }

    fn palette_assert_translucency_swap(&mut self, _ram: &mut Ram<'_>) {}

    fn palette_filter_wish_ponds(&mut self, _ram: &mut Ram<'_>) {}

    fn palette_filter_sp5f(&mut self, _ram: &mut Ram<'_>) {}

    fn palette_revert_translucency_swap(&mut self, _ram: &mut Ram<'_>) {}

    fn palette_filter_restore_sp5f(&mut self, _ram: &mut Ram<'_>) {}

    fn link_receive_item(&mut self, _ram: &mut Ram<'_>, _item: u8, _arg: u8) {}

    fn sprite_spawn_dynamically_ex(
        &mut self,
        _ram: &mut Ram<'_>,
        _parent: usize,
        _kind: u8,
        _limit: usize,
    ) -> Option<(usize, SpriteSpawnInfo)> {
        None
    }

    fn queue_sfx2_with_pan(&mut self, _ram: &mut Ram<'_>, _slot: usize, _sound: u8) {}

    fn queue_sfx1_with_pan(&mut self, _ram: &mut Ram<'_>, _slot: usize, _sound: u8) {}

    fn movable_mantle_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_return_if_inactive(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn sprite_check_damage_to_link_same_layer(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn sprite_nullify_hookshot_drag(&mut self, _ram: &mut Ram<'_>) {}

    fn sprite_repel_dash(&mut self, _ram: &mut Ram<'_>) {}

    fn sprite_move_xy(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_move_x(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_check_tile_collision(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> u8 {
        0
    }

    fn prep_oam_coord_or_double_ret(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
    ) -> Option<PrepOamCoordsRet> {
        None
    }

    fn draw_guard_head(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _info: &PrepOamCoordsRet,
        _oam_offset: usize,
    ) {
    }

    fn draw_bnc_body(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _info: &PrepOamCoordsRet,
        _oam_offset: usize,
    ) {
    }

    fn draw_bnc_flail(&mut self, _ram: &mut Ram<'_>, _slot: usize, _info: &PrepOamCoordsRet) {}

    fn draw_shadow_custom(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _info: &PrepOamCoordsRet,
        _size: u8,
    ) {
    }

    fn sprite_cannonball(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn bee_handle_interactions(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_find_empty_bottle(&mut self, _ram: &mut Ram<'_>) -> i32 {
        -1
    }

    fn wizzbeam_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_check_damage_to_link(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn prep_oam_coord(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> PrepOamCoordsRet {
        PrepOamCoordsRet::default()
    }

    fn load_follower_graphics(&mut self, _ram: &mut Ram<'_>) {}

    fn follower_initialize(&mut self, _ram: &mut Ram<'_>) {}

    fn sprite_sidenexx(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn trinexx_fire_add_fire_garnish(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_cc_cd_common(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_cd_spawn_garnish(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn draw_multiple_player_deferred(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _pieces: &[DrawMultipleData; 4],
    ) {
    }

    fn landmine_hitboxes_overlap(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn oam_allocate_from_region_c(&mut self, _ram: &mut Ram<'_>, _priority: u8) {}

    fn draw_multiple(&mut self, _ram: &mut Ram<'_>, _slot: usize, _pieces: &[DrawMultipleData]) {}

    fn draw_multiple_with_info(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        pieces: &[DrawMultipleData],
        _info: &mut PrepOamCoordsRet,
    ) {
        self.draw_multiple(ram, slot, pieces);
    }

    fn sprite_attempt_damage_to_link_plus_recoil(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_move_y(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn helmasaur_fireball_tri_split(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn helmasaur_fireball_quad_split(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn queue_sfx3_with_pan(&mut self, _ram: &mut Ram<'_>, _slot: usize, _sound: u8) {}

    fn archery_game_guy_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn link_cancel_dash(&mut self, _ram: &mut Ram<'_>) {}

    fn sprite_show_message_minimal(&mut self, _ram: &mut Ram<'_>) {}

    fn sprite_initialize_secondary_item_minigame(&mut self, _ram: &mut Ram<'_>, _kind: u8) {}

    fn oam_allocate_from_region_a(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}

    fn sprite_draw_distress_custom(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16, _frame: u8) {}

    fn sprite_direction_to_face_link(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> (u8, u8) {
        (0, 0)
    }

    /// Mirrors the single C `Sprite_DirectionToFaceLink` call: the returned
    /// direction and both byte-sized deltas are produced together.
    fn sprite_direction_and_deltas(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> (u8, u8, u8) {
        (0, 0, 0)
    }

    fn oam_allocate_from_region_b(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}

    fn sprite_apply_speed_towards_link(&mut self, _ram: &mut Ram<'_>, _slot: usize, _speed: u8) {}

    fn sprite_check_damage_to_and_from_link(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_check_damage_to_and_from_link_result(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
    ) -> bool {
        false
    }

    fn sprite_spawn_fireball(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_correct_oam_entries(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _last_entry: usize,
        _big: u8,
    ) {
    }

    fn sprite_spawn_probe_always(&mut self, _ram: &mut Ram<'_>, _slot: usize, _angle: u8) {}

    fn sprite_prep_load_palette(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_move_z(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_return_if_recoiling(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn sprite_draw_shadow(&mut self, _ram: &mut Ram<'_>, _slot: usize, _info: &PrepOamCoordsRet) {}

    fn sprite_draw_shadow_with_info(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        info: &mut PrepOamCoordsRet,
    ) {
        self.sprite_draw_shadow(ram, slot, info);
    }

    fn sprite_draw_thin_and_tall(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_book_of_mudora(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_lumberjack_tree(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn draw_single_small(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn fireball_spawn_trail_garnish(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn zora_fireball_shield_overlap(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _x: u16,
        _y: u16,
        _width: u8,
        _height: u8,
    ) -> bool {
        false
    }

    fn sprite_place_repulse_spark_2(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_kill_self(&mut self, ram: &mut Ram<'_>, slot: usize) {
        ram.set_sprite_state(slot, 0);
    }

    fn sprite_zora_regurgitate_flippers(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_direction_to_face_link_direction(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> u8 {
        0
    }

    fn sprite_check_damage_from_link(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn thrown_sprite_tile_and_sprite_interaction(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_return_if_paused(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn sprite_check_if_screen_is_clear(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }

    fn sprite_project_speed_towards_location(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _x: u16,
        _y: u16,
        _speed: u8,
    ) -> ProjectSpeedRet {
        ProjectSpeedRet::default()
    }

    fn oam_allocate_defer_to_player(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_convert_velocity_to_angle(&mut self, _x_velocity: u8, _y_velocity: u8) -> u8 {
        0
    }

    fn guard_parry_sword_attacks(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn get_tile_attribute(&mut self, _ram: &mut Ram<'_>, _floor: u8, _x: &mut u16, _y: u16) -> u8 {
        0
    }

    fn toppo_flustered(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_bomb_hitbox_overlaps(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _x: u16,
        _y: u16,
        _size: u8,
    ) -> bool {
        false
    }

    fn ancilla_check_damage_to_sprite_preset(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _damage: u8,
    ) {
    }

    fn sprite_add_xy(&mut self, _ram: &mut Ram<'_>, _slot: usize, _x: i8, _y: i8) {}

    fn probe_check_tile_solidity(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn sprite_is_below_link(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn sprite_is_right_of_link(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn sprite_behave_as_barrier(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_track_body_to_head(&mut self, _ram: &mut Ram<'_>, _slot: usize) -> bool {
        false
    }

    fn priest_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn uncle_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn draw_multiple_player_deferred_slice(
        &mut self,
        _ram: &mut Ram<'_>,
        _slot: usize,
        _pieces: &[DrawMultipleData],
    ) -> PrepOamCoordsRet {
        PrepOamCoordsRet::default()
    }

    fn lady_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_chicken_lady(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn snitch_spawn_guard(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn overworld_draw_wooden_door(&mut self, _ram: &mut Ram<'_>, _position: u16, _open: bool) {}

    fn running_boy_spawn_dust_garnish(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn bottle_merchant_detect_fish(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn bottle_merchant_buy_bee(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn bottle_merchant_buy_fish(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn crystal_maiden_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn save_palace_deaths(&mut self, _ram: &mut Ram<'_>) {}

    fn dungeon_flag_room_data_quadrants(&mut self, _ram: &mut Ram<'_>) {}

    fn sprite_become_follower(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_load_graphics_properties_light_world_only(&mut self, _ram: &mut Ram<'_>) {}

    fn decode_animated_sprite_tile_variable(&mut self, _ram: &mut Ram<'_>, _index: u8) {}

    fn sprite_get_16_bit_coords(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_draw_water_ripple_with_oam_adjust(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn sprite_spawn_water_splash(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn dust_cloud_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn medallion_tablet_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}

    fn bombos_tablet_start_cutscene(&mut self, _ram: &mut Ram<'_>) {}

    fn ether_tablet_start_cutscene(&mut self, _ram: &mut Ram<'_>) {}
}

pub struct SpriteMainBank0<'a, 'r, R: SpriteMainBank0Runtime> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
}

impl<R: SpriteMainBank0Runtime> SpriteMainBank0<'_, '_, R> {
    /// Exact 243-entry active-table routing. Entries whose C definitions begin
    /// after this bank cross the explicit runtime boundary with their table
    /// index intact.
    pub fn dispatch_active(&mut self, slot: usize, kind: u8)
    where
        R: crate::sprite_main_bank1::SpriteMainBank1Runtime
            + crate::sprite_main_bank2a::SpriteMainBank2aRuntime
            + crate::sprite_main_bank2b::SpriteMainBank2bRuntime,
    {
        assert!(kind <= 0xf2, "sprite active type outside 0..=0xf2");
        if kind == 3 || self.dispatch_active_local(slot, kind) {
            return;
        }
        let handled = crate::sprite_main_bank1::SpriteMainBank1 {
            ram: self.ram,
            runtime: self.runtime,
        }
        .dispatch_active_local(slot, kind);
        let handled = handled
            || (crate::sprite_main_bank2a::SpriteMainBank2a {
                ram: self.ram,
                runtime: self.runtime,
            })
            .dispatch_active(slot, kind);
        if !handled
            && !(crate::sprite_main_bank2b::SpriteMainBank2b {
                ram: self.ram,
                runtime: self.runtime,
            })
            .dispatch_active(slot, kind)
        {
            self.runtime.sprite_active_cross_bank(self.ram, slot, kind);
        }
    }

    /// Exact 243-entry prep-table routing, with later-bank definitions passed
    /// through the same stable sprite-type index.
    pub fn dispatch_prep(&mut self, slot: usize, kind: u8)
    where
        R: crate::sprite_main_bank1::SpriteMainBank1Runtime
            + crate::sprite_main_bank2a::SpriteMainBank2aRuntime
            + crate::sprite_main_bank2b::SpriteMainBank2bRuntime,
    {
        assert!(kind <= 0xf2, "sprite prep type outside 0..=0xf2");
        if kind == 3 || self.dispatch_prep_local(slot, kind) {
            return;
        }
        if !(crate::sprite_main_bank2a::SpriteMainBank2a {
            ram: self.ram,
            runtime: self.runtime,
        })
        .dispatch_prep(slot, kind)
            && !(crate::sprite_main_bank2b::SpriteMainBank2b {
                ram: self.ram,
                runtime: self.runtime,
            })
            .dispatch_prep(slot, kind)
        {
            crate::sprite_main_bank1::SpriteMainBank1 {
                ram: self.ram,
                runtime: self.runtime,
            }
            .sprite_module_initialize_after_properties(slot);
        }
    }

    /// Dispatches the active-table entries whose target definitions are owned
    /// by this source bank. Returns `false` for entries owned by later banks.
    pub fn dispatch_active_local(&mut self, slot: usize, kind: u8) -> bool {
        match kind {
            0x04..=0x07 => self.pull_switch_bounce(slot),
            0x32 => self.sprite_quarrel_bros(slot),
            0x34 => self.sprite_young_snitch_lady(slot),
            0x35 => self.sprite_inn_keeper(slot),
            0x36 => self.sprite_witch(slot),
            0x3b => self.sprite_dash_item(slot),
            0x3d => self.sprite_old_snitch_lady(slot),
            0x41..=0x43 => self.sprite_41_blue_guard(slot),
            0x44 => self.sprite_44_bluesain_bolt(slot),
            0x45 => self.sprite_45_hog_spear_man(slot),
            0x46 => self.sprite_46_blue_archer(slot),
            0x47 => self.sprite_47_green_bush_guard(slot),
            0x48 => self.sprite_48_red_javelin_guard(slot),
            0x49 => self.sprite_49_red_bush_guard(slot),
            0x4a => self.sprite_4a_bomb_guard(slot),
            0x4b => self.sprite_4b_green_knife_guard(slot),
            0x4c => self.sprite_4c_geldman(slot),
            0x4d => self.sprite_4d_toppo(slot),
            0x4e | 0x4f => self.sprite_4e_popo(slot),
            0x50 => self.sprite_50_cannonball(slot),
            0x51 => self.sprite_51_armos_statue(slot),
            0x52 => self.sprite_52_king_zora(slot),
            0x53 => self.sprite_53_armos_knight(slot),
            0x54 => self.sprite_54_lanmolas(slot),
            0x55 => self.sprite_55_zora(slot),
            0x56 => self.sprite_56_walking_zora(slot),
            0x57 => self.sprite_57_desert_statue(slot),
            0x58 => self.sprite_58_crab(slot),
            0x59 => self.sprite_59_lost_woods_bird(slot),
            0x5a => self.sprite_5a_lost_woods_squirrel(slot),
            0x5b | 0x5c => self.sprite_5b_spark_clockwise(slot),
            0x5d..=0x60 => self.sprite_5d_roller_vertical_down_first(slot),
            0x61 => self.sprite_61_beamos(slot),
            0x62 => self.sprite_62_master_sword(slot),
            0x63 => self.sprite_63_debirando_pit(slot),
            0x64 => self.sprite_64_debirando(slot),
            0x65 => self.sprite_65_archery_game(slot),
            0x66..=0x69 => self.sprite_66_wall_cannon_vertical_left(slot),
            0x6a => self.sprite_6a_ball_n_chain(slot),
            0x6b => self.sprite_6b_cannon_trooper(slot),
            0x6c => self.sprite_6c_mirror_portal(slot),
            0x6d => self.sprite_6d_rat(slot),
            0x6e => self.sprite_6e_rope(slot),
            0x6f => self.sprite_6f_keese(slot),
            0x70 => self.sprite_70_king_helmasaur_fireball(slot),
            0x74 => self.sprite_running_man(slot),
            0x75 => self.sprite_bottle_vendor(slot),
            0x76 => self.sprite_76_zelda(slot),
            0xcc => self.sprite_cc(slot),
            0xcd => self.sprite_cd(slot),
            0xe7 => self.sprite_e7_mushroom(slot),
            0xe8 => self.sprite_e8_fake_sword(slot),
            0xe9 => self.sprite_e9_potion_shop(slot),
            0xea => self.sprite_heart_container(slot),
            0xeb => self.sprite_heart_piece(slot),
            0xee => self.sprite_ee_movable_mantle(slot),
            _ => return false,
        }
        true
    }

    /// Dispatches prep-table entries whose exact target definitions are owned
    /// by this bank. Later source banks own the remaining prep entries.
    pub fn dispatch_prep_local(&mut self, slot: usize, kind: u8) -> bool {
        match kind {
            0x74 => self.sprite_prep_running_man(slot),
            0xe7 => self.sprite_prep_mushroom(slot),
            0xe8 => Self::sprite_prep_fake_sword(slot),
            0xe9 => self.sprite_prep_potion_shop(slot),
            0xea => self.sprite_prep_heart_container(slot),
            0xf2 => self.sprite_prep_medallion_table(slot),
            _ => return false,
        }
        true
    }

    #[must_use]
    fn oam_current_index(&self) -> usize {
        (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4
    }

    #[must_use]
    fn sprite_x(&self, slot: usize) -> u16 {
        u16::from(self.ram.sprite(slot).x_low()) | (u16::from(self.ram.sprite(slot).x_high()) << 8)
    }

    #[must_use]
    fn sprite_y(&self, slot: usize) -> u16 {
        u16::from(self.ram.sprite(slot).y_low()) | (u16::from(self.ram.sprite(slot).y_high()) << 8)
    }

    fn set_sprite_x(&mut self, slot: usize, x: u16) {
        self.ram.set_sprite_x_low(slot, x as u8);
        self.ram.set_sprite_x_high(slot, (x >> 8) as u8);
    }

    fn set_sprite_y(&mut self, slot: usize, y: u16) {
        self.ram.set_sprite_y_low(slot, y as u8);
        self.ram.set_sprite_y_high(slot, (y >> 8) as u8);
    }

    fn set_spawned_coordinates(&mut self, slot: usize, info: &SpriteSpawnInfo) {
        self.set_sprite_x(slot, info.r0_x);
        self.set_sprite_y(slot, info.r2_y);
        self.ram.set_sprite_z(slot, info.r4_z);
    }

    pub fn pull_switch_bounce(&mut self, slot: usize) {
        match self.ram.sprite(slot).kind() {
            5 | 7 => self.pull_switch_facing_up(slot),
            _ => self.pull_switch_facing_down(slot),
        }
    }

    pub fn giant_moldorm_draw_segment_ab(&mut self, slot: usize, lookback: u8) {
        let subtype = self.ram.sprite(slot).subtype2();
        let history = usize::from(subtype.wrapping_sub(lookback) & 0x7f);
        self.ram.write_u16(
            CUR_SPRITE_X,
            u16::from(self.ram.read_u8(MOLDORM_X_LO + history))
                | (u16::from(self.ram.read_u8(MOLDORM_X_HI + history)) << 8),
        );
        self.ram.write_u16(
            CUR_SPRITE_Y,
            u16::from(self.ram.read_u8(MOLDORM_Y_LO + history))
                | (u16::from(self.ram.read_u8(MOLDORM_Y_HI + history)) << 8),
        );
        self.ram.write_u16(
            OAM_CUR_PTR,
            self.ram.read_u16(OAM_CUR_PTR).wrapping_add(0x10),
        );
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(4),
        );
        let offset = usize::from((subtype >> 1 & 1) * 4);
        let pieces: &[DrawMultipleData; 4] = GIANT_MOLDORM_SEGMENT[offset..offset + 4]
            .try_into()
            .expect("fixed four-piece moldorm frame");
        self.runtime.draw_moldorm_segment(self.ram, slot, pieces);
    }

    pub fn giant_moldorm_draw_segment_c_or_tail(&mut self, slot: usize, lookback: u8) {
        const OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        let subtype = self.ram.sprite(slot).subtype2();
        let history = usize::from(subtype.wrapping_sub(lookback) & 0x7f);
        self.ram.write_u16(
            CUR_SPRITE_X,
            u16::from(self.ram.read_u8(MOLDORM_X_LO + history))
                | (u16::from(self.ram.read_u8(MOLDORM_X_HI + history)) << 8),
        );
        self.ram.write_u16(
            CUR_SPRITE_Y,
            u16::from(self.ram.read_u8(MOLDORM_Y_LO + history))
                | (u16::from(self.ram.read_u8(MOLDORM_Y_HI + history)) << 8),
        );
        let saved = self.ram.sprite(slot).oam_flags();
        self.ram.set_sprite_oam_flags(
            slot,
            (saved & 0x3f) | OAM_FLAGS[usize::from(subtype >> 1 & 3)],
        );
        self.runtime.draw_single_large(self.ram, slot);
        self.ram.set_sprite_oam_flags(slot, saved);
    }

    pub fn chicken_incr_subtype2(&mut self, slot: usize, amount: u8) {
        let subtype = self.ram.sprite(slot).subtype2().wrapping_add(amount);
        self.ram.set_sprite_subtype2(slot, subtype);
        self.ram.set_sprite_graphics(slot, subtype >> 4 & 1);
        self.runtime.sprite_return_if_lifted(self.ram, slot);
    }

    #[must_use]
    pub fn octoballoon_find(&self) -> bool {
        (0..16)
            .rev()
            .any(|slot| self.ram.sprite(slot).state() != 0 && self.ram.sprite(slot).kind() == 0x10)
    }

    #[must_use]
    pub fn flute_boy_check_if_player_close(&self, slot: usize) -> bool {
        let sprite_x = self.sprite_x(slot);
        let sprite_y = self.sprite_y(slot).wrapping_sub(16);
        let x = self
            .ram
            .read_u16(LINK_X)
            .wrapping_sub(sprite_x)
            .wrapping_sub(u16::from((sprite_y as i16) < 0));
        let y = self
            .ram
            .read_u16(LINK_Y)
            .wrapping_sub(sprite_y)
            .wrapping_sub(u16::from((x as i16) < 0));
        let x = if (x as i16) < 0 { !x } else { x };
        let y = if (y as i16) < 0 { !y } else { y };
        x < 48 && y < 48
    }

    pub fn fortune_teller_light_or_dark_world(&mut self, slot: usize, dark_world: bool) {
        const PRICES: [u16; 4] = [10, 15, 20, 30];
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.set_sprite_graphics(slot, 0);
                let selection = crate::misc::get_random_number(self.ram) & 3;
                self.ram.set_sprite_scratch_a(slot, selection << 1);
                self.ram.set_sprite_ai_state(
                    slot,
                    if self.ram.read_u16(LINK_RUPEES_GOAL) < PRICES[usize::from(selection)] {
                        1
                    } else {
                        2
                    },
                );
            }
            1 => {
                self.runtime.show_solicited_message(self.ram, slot, 0xf2);
            }
            2 => {
                if self.runtime.show_solicited_message(self.ram, slot, 0xf3) & 0x100 != 0 {
                    self.ram.set_sprite_main_delay(slot, 0xff);
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                    self.ram.set_sprite_ai_state(slot, 3);
                }
            }
            3 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0 {
                    if self.ram.sprite(slot).main_delay() == 0 {
                        let state = self.ram.sprite(slot).ai_state();
                        self.ram.set_sprite_ai_state(slot, state.wrapping_add(1));
                    }
                    self.ram
                        .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 4 & 1);
                } else {
                    self.runtime.show_message_unconditional(self.ram, 0xf5);
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                }
            }
            4 => self
                .runtime
                .fortune_teller_perform_pseudo_science(self.ram, slot),
            5 => {
                if !dark_world {
                    self.ram.set_sprite_graphics(slot, 0);
                }
                let price = PRICES[usize::from(self.ram.sprite(slot).scratch_a() >> 1)];
                self.ram.write_u8(
                    DIALOGUE_NUMBER,
                    (price / 10) as u8 | ((price % 10) as u8) << 4,
                );
                self.ram.write_u8(DIALOGUE_NUMBER + 1, 0);
                self.runtime.show_message_unconditional(self.ram, 0xf4);
                let state = self.ram.sprite(slot).ai_state();
                self.ram.set_sprite_ai_state(slot, state.wrapping_add(1));
            }
            6 => {
                let price = PRICES[usize::from(self.ram.sprite(slot).scratch_a() >> 1)];
                self.ram.write_u16(
                    LINK_RUPEES_GOAL,
                    self.ram.read_u16(LINK_RUPEES_GOAL).wrapping_sub(price),
                );
                let state = self.ram.sprite(slot).ai_state();
                self.ram.set_sprite_ai_state(slot, state.wrapping_add(1));
                self.ram.write_u8(LINK_HEARTS_FILLER, 160);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
            }
            _ => {}
        }
    }

    #[must_use]
    pub fn garnish_alloc_force(&self) -> usize {
        (0..30)
            .rev()
            .find(|&slot| self.ram.read_u8(GARNISH_TYPE + slot) == 0)
            .unwrap_or(0)
    }

    #[must_use]
    pub fn garnish_alloc(&self) -> i32 {
        self.garnish_alloc_limit(29)
    }

    #[must_use]
    pub fn garnish_alloc_low(&self) -> i32 {
        self.garnish_alloc_limit(14)
    }

    #[must_use]
    pub fn garnish_alloc_limit(&self, limit: i32) -> i32 {
        (0..=limit)
            .rev()
            .find(|&slot| self.ram.read_u8(GARNISH_TYPE + slot as usize) == 0)
            .unwrap_or(-1)
    }

    pub fn garnish_alloc_overwrite_old_low(&mut self) -> usize {
        let slot = self.garnish_alloc_low();
        if slot >= 0 {
            slot as usize
        } else {
            self.advance_garnish_overwrite_cursor(14)
        }
    }

    pub fn garnish_alloc_overwrite_old(&mut self) -> usize {
        let slot = self.garnish_alloc();
        if slot >= 0 {
            slot as usize
        } else {
            self.advance_garnish_overwrite_cursor(29)
        }
    }

    fn advance_garnish_overwrite_cursor(&mut self, reset: u8) -> usize {
        let cursor = self.ram.read_u8(BYTE_7E0FF8).wrapping_sub(1);
        let cursor = if (cursor as i8) < 0 { reset } else { cursor };
        self.ram.write_u8(BYTE_7E0FF8, cursor);
        usize::from(cursor)
    }

    pub fn garnish_set_x(&mut self, slot: usize, x: u16) {
        self.ram.write_u8(GARNISH_X_LO + slot, x as u8);
        self.ram.write_u8(GARNISH_X_HI + slot, (x >> 8) as u8);
    }

    pub fn garnish_set_y(&mut self, slot: usize, y: u16) {
        self.ram.write_u8(GARNISH_Y_LO + slot, y as u8);
        self.ram.write_u8(GARNISH_Y_HI + slot, (y >> 8) as u8);
    }

    pub fn sprite_wish_pond3(&mut self, slot: usize) {
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                if self.ram.sprite(slot).main_delay() != 0
                    || self.runtime.sprite_check_if_link_is_busy(self.ram)
                {
                    return;
                }
                if self.runtime.show_message_on_contact(self.ram, slot, 0x14a) & 0x100 != 0 {
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.runtime.link_reset_properties_a(self.ram);
                    self.ram.write_u8(LINK_DIRECTION_FACING, 0);
                    self.ram.set_sprite_head_direction(slot, 0);
                }
            }
            1 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0 {
                    self.runtime.show_message_unconditional(self.ram, 0x8a);
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                } else {
                    self.runtime.show_message_unconditional(self.ram, 0x14b);
                    self.ram.set_sprite_ai_state(slot, 0);
                    self.ram.set_sprite_main_delay(slot, 0xff);
                }
            }
            2 => {
                self.ram.set_sprite_ai_state(slot, 3);
                let choice = usize::from(self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX));
                self.ram.set_sprite_scratch_c(slot, choice as u8);
                let item = self.ram.read_u8(LINK_ITEM_BOW + choice);
                self.ram.write_u8(LINK_ITEM_BOW + choice, 0);
                let item_variant = if choice == 3 || choice == 32 { 1 } else { item };
                let item_index = usize::from(WISH_POND_ITEM_OFFSETS[choice])
                    .wrapping_add(usize::from(item_variant))
                    .wrapping_sub(1);
                let pond_item = WISH_POND_ITEM_DATA[item_index];
                self.runtime.add_tossed_pond_item(self.ram, pond_item);
                self.runtime.hud_refresh_icon(self.ram);
                self.ram.set_sprite_graphics(slot, pond_item);
                self.ram.set_sprite_scratch_d(slot, item);
                self.ram.set_sprite_main_delay(slot, 0xff);
            }
            3 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    let (spawned, info) = self
                        .runtime
                        .sprite_spawn_dynamically(self.ram, slot, 0x72)
                        .expect("Sprite_WishPond3 spawn must succeed");
                    self.set_sprite_x(spawned, info.r0_x);
                    self.set_sprite_y(spawned, info.r2_y.wrapping_sub(80));
                    self.ram.write_u8(MUSIC_CONTROL, 0x1b);
                    self.ram.write_u8(LAST_MUSIC_CONTROL, 0);
                    self.ram.set_sprite_scratch_b(spawned, 1);
                    self.runtime.palette_assert_translucency_swap(self.ram);
                    self.runtime.palette_filter_wish_ponds(self.ram);
                    self.ram.set_sprite_scratch_e(slot, spawned as u8);
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.ram.set_sprite_main_delay(slot, 0xff);
                }
            }
            4 => {
                if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
                    self.runtime.palette_filter_sp5f(self.ram);
                    if self.ram.read_u8(PALETTE_FILTER_COUNTDOWN) == 0 {
                        self.runtime.show_message_unconditional(self.ram, 0x8b);
                        self.runtime.palette_revert_translucency_swap(self.ram);
                        self.ram.write_u8(TS_COPY, 0);
                        self.ram.write_u8(CGADSUB_COPY, 0x20);
                        let value = self.ram.read_u8(FLAG_UPDATE_CGRAM_IN_NMI);
                        self.ram
                            .write_u8(FLAG_UPDATE_CGRAM_IN_NMI, value.wrapping_add(1));
                        self.ram.set_sprite_ai_state(slot, 5);
                    }
                }
            }
            5 => self.ram.set_sprite_ai_state(
                slot,
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0 {
                    6
                } else {
                    11
                },
            ),
            6 => {
                self.ram.set_sprite_ai_state(slot, 7);
                let graphics = self.ram.sprite(slot).graphics();
                if self.ram.read_u8(SAVEGAME_IS_DARKWORLD) == 0 {
                    match graphics {
                        12 => {
                            self.ram.set_sprite_graphics(slot, 42);
                            self.ram.set_sprite_head_direction(slot, 1);
                        }
                        4 => {
                            self.ram.set_sprite_graphics(slot, 5);
                            self.ram.set_sprite_head_direction(slot, 2);
                        }
                        22 => {
                            self.ram.set_sprite_graphics(slot, 44);
                            self.ram.set_sprite_head_direction(slot, 3);
                        }
                        _ => {
                            self.runtime.show_message_unconditional(self.ram, 0x14d);
                            return;
                        }
                    }
                } else {
                    match graphics {
                        58 => {
                            self.ram.set_sprite_graphics(slot, 59);
                            self.ram.set_sprite_head_direction(slot, 4);
                            self.runtime.show_message_unconditional(self.ram, 0x14f);
                            return;
                        }
                        2 => {
                            self.ram.set_sprite_graphics(slot, 3);
                            self.ram.set_sprite_head_direction(slot, 5);
                        }
                        22 => {
                            self.ram.set_sprite_graphics(slot, 44);
                            self.ram.set_sprite_head_direction(slot, 3);
                        }
                        _ => {
                            self.runtime.show_message_unconditional(self.ram, 0x14d);
                            return;
                        }
                    }
                }
                self.runtime.show_message_unconditional(self.ram, 0x8c);
            }
            7 => {
                let choice = usize::from(self.ram.sprite(slot).scratch_c());
                if choice == 3 {
                    self.ram
                        .write_u8(LINK_ITEM_BOW + choice, self.ram.sprite(slot).scratch_d());
                }
                self.runtime.palette_assert_translucency_swap(self.ram);
                self.ram.write_u8(TS_COPY, 2);
                self.ram.write_u8(CGADSUB_COPY, 0x30);
                let value = self.ram.read_u8(FLAG_UPDATE_CGRAM_IN_NMI);
                self.ram
                    .write_u8(FLAG_UPDATE_CGRAM_IN_NMI, value.wrapping_add(1));
                self.ram.set_sprite_ai_state(slot, 8);
            }
            8 => {
                if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
                    self.runtime.palette_filter_sp5f(self.ram);
                    match self.ram.read_u8(PALETTE_FILTER_COUNTDOWN) {
                        30 => {
                            let spawned = usize::from(self.ram.sprite(slot).scratch_e());
                            self.ram.set_sprite_state(spawned, 0);
                        }
                        0 => self.ram.set_sprite_ai_state(slot, 9),
                        _ => {}
                    }
                }
            }
            9 => {
                self.runtime.palette_filter_restore_sp5f(self.ram);
                self.runtime.palette_revert_translucency_swap(self.ram);
                self.ram.write_u8(ITEM_RECEIPT_METHOD, 2);
                let item = self.ram.sprite(slot).graphics();
                self.runtime.link_receive_item(self.ram, item, 2);
                self.ram.set_sprite_ai_state(slot, 10);
            }
            10 => {
                const MESSAGES: [u16; 5] = [0x8f, 0x90, 0x92, 0x91, 0x93];
                let head_direction = self.ram.sprite(slot).head_direction();
                if head_direction != 0 {
                    self.runtime.show_message_unconditional(
                        self.ram,
                        MESSAGES[usize::from(head_direction - 1)],
                    );
                }
                self.ram.set_sprite_ai_state(slot, 0);
                self.ram.set_sprite_main_delay(slot, 0xff);
            }
            11 => {
                self.runtime.show_message_unconditional(self.ram, 0x8d);
                self.ram.set_sprite_ai_state(slot, 12);
            }
            12 => self.ram.set_sprite_ai_state(
                slot,
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0 {
                    13
                } else {
                    6
                },
            ),
            13 => {
                self.runtime.show_message_unconditional(self.ram, 0x8e);
                self.ram.set_sprite_ai_state(slot, 7);
            }
            _ => {}
        }
    }

    pub fn sprite_spawn_small_splash(&mut self, slot: usize) -> i32 {
        let Some((spawned, info)) = self
            .runtime
            .sprite_spawn_dynamically_ex(self.ram, slot, 0xec, 14)
        else {
            return -1;
        };
        self.set_sprite_x(spawned, info.r0_x);
        self.set_sprite_y(spawned, info.r2_y);
        self.ram.set_sprite_z(spawned, info.r4_z);
        self.ram.write_u8(SOUND_EFFECT_1, 0);
        self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x28);
        self.ram.set_sprite_state(spawned, 3);
        self.ram.set_sprite_main_delay(spawned, 15);
        self.ram.set_sprite_ai_state(spawned, 0);
        self.ram.set_sprite_flags2(spawned, 3);
        spawned as i32
    }

    pub fn heart_upgrade_check_if_already_obtained(&mut self, slot: usize) {
        if self.ram.read_u8(PLAYER_IS_INDOORS) == 0 {
            let screen = usize::from(self.ram.read_u8(OVERWORLD_SCREEN_INDEX));
            if (screen == 0x3b && self.ram.read_u8(SAVE_OW_EVENT_INFO + 0x3b) & 0x20 == 0)
                || self.ram.read_u8(SAVE_OW_EVENT_INFO + screen) & 0x40 != 0
            {
                self.ram.set_sprite_state(slot, 0);
            }
        } else {
            let dungeon_mask = if self.ram.sprite(slot).x_high() & 1 != 0 {
                0x2000
            } else {
                0x4000
            };
            if self.ram.read_u16(DUNG_SAVEGAME_STATE_BITS) & dungeon_mask != 0 {
                self.ram.set_sprite_state(slot, 0);
            }
        }
    }

    pub fn sprite_ee_movable_mantle(&mut self, slot: usize) {
        self.runtime.movable_mantle_draw(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || !self
                .runtime
                .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            return;
        }
        self.runtime.sprite_nullify_hookshot_drag(self.ram);
        self.runtime.sprite_repel_dash(self.ram);
        if self.ram.read_u8(FOLLOWER_INDICATOR) != 1
            || self.ram.read_u8(LINK_ITEM_TORCH) == 0
            || self.ram.read_u8(LINK_IS_RUNNING) != 0
            || self.ram.sprite(slot).scratch_g() == 0x90
            || (self.ram.read_u8(LINK_ACTUAL_VEL_X).wrapping_sub(24) as i8) < 0
        {
            return;
        }
        self.ram.write_u8(WHICH_STARTING_POINT, 4);
        let subtype = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, subtype);
        if subtype & 1 == 0 {
            let progress = self.ram.sprite(slot).scratch_g().wrapping_add(1);
            self.ram.set_sprite_scratch_g(slot, progress);
        }
        if self.ram.sprite(slot).scratch_g() < 8 {
            return;
        }
        if self.ram.read_u8(SOUND_EFFECT_1) == 0 {
            self.ram.write_u8(SOUND_EFFECT_1, 34);
        }
        self.ram.set_sprite_velocity_x(slot, 2);
        self.runtime.sprite_move_xy(self.ram, slot);
    }

    pub fn sprite_good_or_bad_archery_target(&mut self, slot: usize) {
        const CASH_PRIZE: [u16; 10] = [4, 8, 16, 32, 64, 99, 99, 99, 99, 99];
        if self.ram.sprite(slot).scratch_a() == 1 {
            if self.ram.sprite(slot).scratch_g() >= 5 {
                self.ram.set_sprite_scratch_b(slot, 6);
            }
            let flags2 = self.ram.sprite(slot).flags2() & !0x1f;
            self.ram.set_sprite_flags2(slot, flags2);
            let aux2 = self.ram.sprite(slot).aux2_delay();
            let animation = if aux2 != 0 {
                aux2
            } else {
                self.ram.sprite(slot).subtype2() >> 3
            };
            let oam_flags = (self.ram.sprite(slot).oam_flags() & !0x40) | ((animation & 4) << 4);
            self.ram.set_sprite_oam_flags(slot, oam_flags);
            self.ram
                .write_u8(CUR_SPRITE_Y, self.ram.read_u8(CUR_SPRITE_Y).wrapping_sub(3));
            self.runtime.draw_single_large(self.ram, slot);
            if aux2 != 0 {
                if aux2 == 96 && self.ram.read_u8(SUBMODULE_INDEX) == 0 {
                    self.ram.set_sprite_main_delay(0, 112);
                    let prize_index =
                        usize::from(self.ram.sprite(slot).scratch_b().wrapping_sub(1));
                    self.ram.write_u16(
                        LINK_RUPEES_GOAL,
                        self.ram
                            .read_u16(LINK_RUPEES_GOAL)
                            .wrapping_add(CASH_PRIZE[prize_index]),
                    );
                }
                self.ram.set_sprite_flags2(slot, flags2 | 5);
                self.archery_game_draw_prize(slot);
            }
        } else {
            let flags2 = self.ram.sprite(slot).flags2() & !0x1f;
            self.ram.set_sprite_flags2(slot, flags2);
            self.ram
                .write_u8(CUR_SPRITE_Y, self.ram.read_u8(CUR_SPRITE_Y).wrapping_add(3));
            self.runtime.draw_single_large(self.ram, slot);
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).aux3_delay() == 1 {
            self.ram.write_u8(SOUND_EFFECT_1, 0x3c);
        }
        let subtype = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, subtype);
        self.runtime.sprite_move_x(self.ram, slot);
        if self.ram.sprite(slot).aux1_delay() == 0 {
            let delay = self.ram.sprite(slot).main_delay();
            self.ram.set_sprite_ignore_projectile(slot, delay);
            if delay == 0 {
                if self.runtime.sprite_check_tile_collision(self.ram, slot) != 0 {
                    self.ram.set_sprite_main_delay(slot, 16);
                    self.ram.set_sprite_aux2_delay(slot, 0);
                }
            } else if delay == 1 {
                const TARGET_X: [u8; 2] = [(-24_i8) as u8, 8];
                let graphics = usize::from(self.ram.sprite(slot).graphics());
                self.ram.set_sprite_x_low(slot, TARGET_X[graphics]);
                self.ram
                    .set_sprite_x_high(slot, (self.ram.read_u16(LINK_X) >> 8) as u8);
                self.ram.set_sprite_aux1_delay(slot, 32);
                self.ram.set_sprite_scratch_g(slot, 0);
            }
        }
    }

    #[must_use]
    pub const fn chain_ball_mult(a: u16, b: u8) -> u8 {
        if a >= 256 {
            b
        } else {
            let product = a * b as u16;
            ((product >> 8) + (product >> 7 & 1)) as u8
        }
    }

    pub fn chain_ball_trooper_draw(&mut self, slot: usize) {
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.sprite_draw_guard_head(slot, &info, 0x18 / 4);
        self.sprite_draw_bnc_body(slot, &info, 0x14 / 4);
        self.sprite_draw_bnc_flail(slot, &info);
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            let shadow = SOLDIER_DRAW_SHADOW[usize::from(self.ram.sprite(slot).scratch_d())];
            self.runtime
                .draw_shadow_custom(self.ram, slot, &info, shadow);
        }
    }

    pub fn sprite_6b_cannon_trooper(&mut self, slot: usize) {
        assert_ne!(
            self.ram.sprite(slot).scratch_c(),
            0,
            "Sprite_6B_CannonTrooper reached its asserted impossible variant"
        );
        self.runtime.sprite_cannonball(self.ram, slot);
    }

    pub fn bee_put_in_bottle(&mut self, slot: usize) {
        self.runtime.bee_handle_interactions(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0 {
            let bottle = self.runtime.sprite_find_empty_bottle(self.ram);
            if bottle >= 0 {
                self.ram.write_u8(
                    LINK_BOTTLE_INFO + bottle as usize,
                    7_u8.wrapping_add(self.ram.sprite(slot).head_direction()),
                );
                self.runtime.hud_refresh_icon(self.ram);
                self.ram.set_sprite_state(slot, 0);
                return;
            }
            self.runtime.show_message_unconditional(self.ram, 0xca);
        }
        self.ram.set_sprite_aux4_delay(slot, 64);
        self.ram.set_sprite_ai_state(slot, 1);
    }

    #[must_use]
    pub const fn guruguru_bar_mult(a: u16, b: u8) -> u8 {
        Self::chain_ball_mult(a, b)
    }

    #[must_use]
    pub fn guruguru_bar_sin(a: u16, b: u8) -> i8 {
        let value = Self::guruguru_bar_mult(SINUS_LOOKUP_TABLE[usize::from(a & 0xff)], b);
        if a & 0x100 != 0 {
            0_u8.wrapping_sub(value) as i8
        } else {
            value as i8
        }
    }

    #[must_use]
    pub const fn arrgi_mult(a: u16, b: u8) -> u8 {
        Self::chain_ball_mult(a, b)
    }

    #[must_use]
    pub fn arrgi_sin(a: u16, b: u8) -> i8 {
        let value = Self::arrgi_mult(SINUS_LOOKUP_TABLE[usize::from(a & 0xff)], b);
        if a & 0x100 != 0 {
            0_u8.wrapping_sub(value) as i8
        } else {
            value as i8
        }
    }

    #[must_use]
    pub const fn helmasaur_mult(a: u16, b: u8) -> u8 {
        Self::chain_ball_mult(a, b)
    }

    #[must_use]
    pub fn helmasaur_sin(a: u16, b: u8) -> i8 {
        let value = Self::helmasaur_mult(SINUS_LOOKUP_TABLE[usize::from(a & 0xff)], b);
        if a & 0x100 != 0 {
            0_u8.wrapping_sub(value) as i8
        } else {
            value as i8
        }
    }

    pub fn sprite_wizzbeam(&mut self, slot: usize) {
        self.runtime.wizzbeam_draw(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.ram
            .set_sprite_oam_flags(slot, self.ram.sprite(slot).oam_flags() ^ 6);
        self.ram
            .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
        if self.ram.sprite(slot).ai_state() == 0 {
            let _ = self.runtime.sprite_check_damage_to_link(self.ram, slot);
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        if self.runtime.sprite_check_tile_collision(self.ram, slot) != 0 {
            self.ram.set_sprite_state(slot, 0);
        }
    }

    pub fn kiki_lying_in_wait(&mut self, slot: usize) {
        self.runtime.prep_oam_coord(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.read_u8(LINK_IS_BUNNY_MIRROR)
            | self.ram.read_u8(LINK_DISABLE_SPRITE_DAMAGE)
            | self.ram.read_u8(COUNTDOWN_FOR_BLINK)
            != 0
            || self.ram.read_u8(FOLLOWER_INDICATOR) == 10
        {
            return;
        }
        let screen = usize::from(self.ram.read_u8(OVERWORLD_SCREEN_INDEX));
        if self.ram.read_u8(SAVE_OW_EVENT_INFO + screen) & 0x20 != 0 {
            return;
        }
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            if self.ram.read_u16(ENHANCED_FEATURES0) & 4096 != 0 {
                self.ram.write_u8(FOLLOWER_DROPPED, 0);
            }
            self.ram.write_u8(FOLLOWER_INDICATOR, 10);
            self.ram.write_u8(TAGALONG_VAR5, 0);
            self.runtime.load_follower_graphics(self.ram);
            self.runtime.follower_initialize(self.ram);
        }
    }

    #[must_use]
    pub fn chain_chomp_one_mult(a: u8, b: u8) -> i32 {
        let magnitude = if (a as i8) < 0 {
            0_u8.wrapping_sub(a)
        } else {
            a
        };
        let product = ((u16::from(magnitude) * u16::from(b)) >> 8) as u8;
        if (a as i8) < 0 {
            !i32::from(product)
        } else {
            i32::from(product)
        }
    }

    #[must_use]
    pub fn trinexx_mult(a: u8, b: u8) -> u8 {
        let magnitude = if (a as i8) < 0 {
            0_u8.wrapping_sub(a)
        } else {
            a
        };
        let product = u16::from(magnitude) * u16::from(b);
        let result = ((product >> 8) + (product >> 7 & 1)) as u8;
        if (a as i8) < 0 {
            0_u8.wrapping_sub(result)
        } else {
            result
        }
    }

    #[must_use]
    pub const fn trinexx_head_mult(a: u16, b: u8) -> u8 {
        Self::chain_ball_mult(a, b)
    }

    #[must_use]
    pub fn trinexx_head_sin(a: u16, b: u8) -> i8 {
        let value = Self::trinexx_head_mult(SINUS_LOOKUP_TABLE[usize::from(a & 0xff)], b);
        if a & 0x100 != 0 {
            0_u8.wrapping_sub(value) as i8
        } else {
            value as i8
        }
    }

    pub fn sprite_cc(&mut self, slot: usize) {
        if self.ram.sprite(slot).scratch_e() == 0 {
            self.runtime.sprite_sidenexx(self.ram, slot);
            return;
        }
        self.runtime.prep_oam_coord(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        self.runtime.trinexx_fire_add_fire_garnish(self.ram, slot);
        self.runtime.sprite_cc_cd_common(self.ram, slot);
    }

    pub fn sprite_cd(&mut self, slot: usize) {
        if self.ram.sprite(slot).scratch_e() == 0 {
            self.runtime.sprite_sidenexx(self.ram, slot);
            return;
        }
        self.runtime.prep_oam_coord(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        let saved_x_velocity = self.ram.sprite(slot).velocity_x();
        self.ram.set_sprite_velocity_x(
            slot,
            saved_x_velocity.wrapping_add(self.ram.sprite(slot).scratch_c()),
        );
        self.runtime.sprite_move_xy(self.ram, slot);
        self.ram.set_sprite_velocity_x(slot, saved_x_velocity);
        self.runtime.sprite_cd_spawn_garnish(self.ram, slot);
        self.runtime.sprite_cc_cd_common(self.ram, slot);
    }

    #[must_use]
    pub const fn ganon_mult(a: u16, b: u8) -> u8 {
        Self::chain_ball_mult(a, b)
    }

    #[must_use]
    pub fn ganon_sin(a: u16, b: u8) -> i8 {
        let value = Self::ganon_mult(SINUS_LOOKUP_TABLE[usize::from(a & 0xff)], b);
        if a & 0x100 != 0 {
            0_u8.wrapping_sub(value) as i8
        } else {
            value as i8
        }
    }

    pub fn sprite_prep_incr_xy_low8(&mut self, slot: usize) {
        self.ram
            .set_sprite_x_low(slot, self.ram.sprite(slot).x_low().wrapping_add(8));
        self.ram
            .set_sprite_y_low(slot, self.ram.sprite(slot).y_low().wrapping_add(8));
    }

    pub const fn sprite_prep_fake_sword(_slot: usize) {}

    pub fn sprite_prep_medallion_table(&mut self, slot: usize) {
        self.ram.set_sprite_ignore_projectile(
            slot,
            self.ram.sprite(slot).ignore_projectile().wrapping_add(1),
        );
        if self.ram.read_u8(OVERWORLD_SCREEN_INDEX) != 3 {
            self.ram
                .set_sprite_x_low(slot, self.ram.sprite(slot).x_low().wrapping_add(8));
            if self.ram.read_u8(LINK_ITEM_BOMBOS_MEDALLION) != 0 {
                self.ram.set_sprite_graphics(slot, 4);
                self.ram.set_sprite_ai_state(slot, 3);
            }
        } else if self.ram.read_u8(LINK_ITEM_ETHER_MEDALLION) != 0 {
            self.ram.set_sprite_graphics(slot, 4);
            self.ram.set_sprite_ai_state(slot, 3);
        }
    }

    pub fn hobo_draw(&mut self, slot: usize) {
        let offset = usize::from(self.ram.sprite(slot).graphics()) * 4;
        let pieces: &[DrawMultipleData; 4] = HOBO_DRAW_DATA[offset..offset + 4]
            .try_into()
            .expect("fixed four-piece hobo frame");
        self.runtime
            .draw_multiple_player_deferred(self.ram, slot, pieces);
    }

    pub fn landmine_check_detonation_from_hammer(&mut self, slot: usize) -> bool {
        if self.ram.read_u8(LINK_ITEM_IN_HAND) & 10 == 0
            || self.ram.read_u8(PLAYER_OAM_Y_OFFSET) == 0x80
        {
            return false;
        }
        self.runtime.landmine_hitboxes_overlap(self.ram, slot)
    }

    pub fn sprite_draw_large_water_turbulence(&mut self, slot: usize) {
        let saved_oam_flags = self.ram.sprite(slot).oam_flags();
        let flags = if self.ram.sprite(slot).subtype2() >> 1 & 1 != 0 {
            0x44
        } else {
            4
        };
        self.ram.set_sprite_oam_flags(slot, flags);
        let priority = self.ram.sprite(slot).object_priority() & !0xf;
        self.ram.set_sprite_object_priority(slot, priority);
        self.runtime.oam_allocate_from_region_c(self.ram, priority);
        self.runtime
            .draw_multiple(self.ram, slot, &WATER_TURBULENCE_DRAW_DATA);
        self.ram.set_sprite_oam_flags(slot, saved_oam_flags);
    }

    pub fn sprite_spawn_sparkle_garnish(&mut self, slot: usize) {
        const COORDINATE: [i8; 4] = [-4, 0, 4, 8];
        if self.ram.read_u8(FRAME_COUNTER) & 3 != 0 {
            return;
        }
        let garnish = self.garnish_alloc_force();
        self.ram.write_u8(GARNISH_TYPE + garnish, 0x12);
        self.ram.write_u8(GARNISH_ACTIVE, 0x12);
        let x_offset = COORDINATE[usize::from(crate::misc::get_random_number(self.ram) & 3)];
        let x = self.sprite_x(slot).wrapping_add(x_offset as i16 as u16);
        let y_offset = COORDINATE[usize::from(crate::misc::get_random_number(self.ram) & 3)];
        let y = self.sprite_y(slot).wrapping_add(y_offset as i16 as u16);
        self.garnish_set_x(garnish, x);
        self.garnish_set_y(garnish, y);
        self.ram.write_u8(GARNISH_SPRITE + garnish, slot as u8);
        self.ram.write_u8(GARNISH_COUNTDOWN + garnish, 15);
    }

    pub fn sprite_70_king_helmasaur_fireball(&mut self, slot: usize) {
        const CHARACTER: [u8; 3] = [0xcc, 0xcc, 0xca];
        const FLAGS: [u8; 2] = [0x33, 0x73];
        const GRAPHICS: [u8; 4] = [2, 2, 1, 0];
        let subtype = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, subtype);
        let flags = FLAGS[usize::from(subtype >> 2 & 1)];
        let oam_address = usize::from(self.ram.read_u16(OAM_CUR_PTR));
        let screen_x = self
            .ram
            .sprite(slot)
            .x_low()
            .wrapping_sub(self.ram.read_u8(BG2_HOFS_COPY2));
        self.ram.write_u8(oam_address, screen_x);
        if screen_x.wrapping_add(32) < 64 {
            self.ram.set_sprite_state(slot, 0);
            return;
        }
        let screen_y = self
            .ram
            .sprite(slot)
            .y_low()
            .wrapping_sub(self.ram.read_u8(BG2_VOFS_COPY2));
        self.ram.write_u8(oam_address + 1, screen_y);
        if screen_y.wrapping_add(16) < 32 {
            self.ram.set_sprite_state(slot, 0);
            return;
        }
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        self.ram.write_u8(oam_address + 2, CHARACTER[graphics]);
        self.ram.write_u8(oam_address + 3, flags);
        let oam_index = (oam_address - crate::oam::OAM_BUFFER) / 4;
        self.ram
            .write_u8(crate::oam::BYTEWISE_EXTENDED_OAM + oam_index, 2);

        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if ((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0
            && self
                .ram
                .read_u16(LINK_X)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                .wrapping_add(8)
                < 16
            && self
                .ram
                .read_u16(LINK_Y)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                .wrapping_add(16)
                < 16
        {
            self.runtime
                .sprite_attempt_damage_to_link_plus_recoil(self.ram, slot);
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_main_delay(slot, 18);
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.ram.set_sprite_velocity_y(slot, 36);
                }
            }
            1 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 31);
                }
                self.ram.set_sprite_velocity_y(
                    slot,
                    self.ram.sprite(slot).velocity_y().wrapping_sub(2),
                );
                self.runtime.sprite_move_y(self.ram, slot);
            }
            2 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.runtime.helmasaur_fireball_tri_split(self.ram, slot);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, GRAPHICS[usize::from(delay >> 3)]);
                }
            }
            3 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.runtime.helmasaur_fireball_quad_split(self.ram, slot);
                } else if self.ram.sprite(slot).head_direction() < 20 {
                    self.ram.set_sprite_head_direction(
                        slot,
                        self.ram.sprite(slot).head_direction().wrapping_add(1),
                    );
                    self.runtime.sprite_move_xy(self.ram, slot);
                }
            }
            4 => self.runtime.sprite_move_xy(self.ram, slot),
            _ => {}
        }
    }

    pub fn sprite_66_wall_cannon_vertical_left(&mut self, slot: usize) {
        const X_VELOCITY: [i8; 4] = [0, 0, -16, 16];
        const Y_VELOCITY: [i8; 4] = [-16, 16, 0, 0];
        const GRAPHICS: [u8; 4] = [0, 0, 2, 2];
        const OAM_FLAGS: [u8; 4] = [0x40, 0, 0, 0x80];
        const SPAWN_X: [i8; 4] = [8, -8, 0, 0];
        const SPAWN_Y: [i8; 4] = [0, 0, 8, -8];
        const SPAWN_X_VELOCITY: [i8; 4] = [24, -24, 0, 0];
        const SPAWN_Y_VELOCITY: [i8; 4] = [0, 0, 24, -24];
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        self.ram.set_sprite_graphics(
            slot,
            GRAPHICS[direction] + u8::from(self.ram.sprite(slot).aux2_delay() != 0),
        );
        self.ram.set_sprite_oam_flags(
            slot,
            (self.ram.sprite(slot).oam_flags() & !0x40) | OAM_FLAGS[direction],
        );
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).main_delay() == 0 {
            self.ram.set_sprite_main_delay(slot, 128);
            self.ram
                .set_sprite_scratch_a(slot, self.ram.sprite(slot).scratch_a() ^ 1);
        }
        let movement = usize::from(self.ram.sprite(slot).scratch_a());
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, X_VELOCITY[movement]);
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityY, Y_VELOCITY[movement]);
        self.runtime.sprite_move_xy(self.ram, slot);
        if ((slot << 2).wrapping_add(usize::from(self.ram.read_u8(FRAME_COUNTER)))) & 31 == 0 {
            self.ram.set_sprite_aux2_delay(slot, 16);
        }
        if self.ram.sprite(slot).aux2_delay() != 1 || self.ram.sprite(slot).pause() != 0 {
            return;
        }
        if let Some((spawned, info)) = self
            .runtime
            .sprite_spawn_dynamically_ex(self.ram, slot, 0x6b, 13)
        {
            self.runtime.queue_sfx3_with_pan(self.ram, slot, 7);
            self.ram.set_sprite_scratch_c(spawned, 1);
            self.ram.set_sprite_graphics(spawned, 1);
            self.set_sprite_x(
                spawned,
                info.r0_x.wrapping_add(SPAWN_X[direction] as i16 as u16),
            );
            self.set_sprite_y(
                spawned,
                info.r2_y.wrapping_add(SPAWN_Y[direction] as i16 as u16),
            );
            self.ram.set_sprite_signed_field(
                spawned,
                SpriteField::VelocityX,
                SPAWN_X_VELOCITY[direction],
            );
            self.ram.set_sprite_signed_field(
                spawned,
                SpriteField::VelocityY,
                SPAWN_Y_VELOCITY[direction],
            );
            self.ram
                .set_sprite_flags2(spawned, (self.ram.sprite(spawned).flags2() & 0xf0) | 1);
            self.ram
                .set_sprite_flags3(spawned, self.ram.sprite(spawned).flags3() | 0x47);
            self.ram.set_sprite_deflection_bits(
                spawned,
                self.ram.sprite(spawned).deflection_bits() | 0x44,
            );
            self.ram.set_sprite_main_delay(spawned, 32);
        }
    }

    pub fn sprite_65_archery_game(&mut self, slot: usize) {
        self.ram
            .write_u8(LINK_NUM_ARROWS, self.ram.sprite(slot).subtype());
        if self.ram.sprite(slot).scratch_a() == 0 {
            self.archery_game_host(slot);
        } else {
            self.sprite_good_or_bad_archery_target(slot);
        }
    }

    pub fn archery_game_host(&mut self, slot: usize) {
        if self.ram.read_u8(ARCHERY_GAME_ARROWS_LEFT) == 0 {
            self.ram.write_u8(
                ARCHERY_GAME_OUT_OF_ARROWS,
                self.ram.read_u8(ARCHERY_GAME_OUT_OF_ARROWS).wrapping_add(1),
            );
        }
        self.runtime.archery_game_guy_draw(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.ram.set_sprite_flags4(slot, 0);
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            self.runtime.sprite_nullify_hookshot_drag(self.ram);
            self.ram.write_u8(LINK_SPEED_SETTING, 0);
            self.runtime.link_cancel_dash(self.ram);
        }
        let delay = self.ram.sprite(slot).main_delay();
        if delay != 0 {
            if delay & 7 == 0 {
                self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x11);
            }
            self.ram.set_sprite_graphics(slot, (delay & 4) >> 2);
        } else {
            const GRAPHICS: [u8; 4] = [3, 4, 3, 2];
            let graphics_index = if self.ram.sprite(slot).ai_state() != 0 {
                usize::from(self.ram.read_u8(FRAME_COUNTER) >> 5 & 3)
            } else {
                0
            };
            self.ram.set_sprite_graphics(slot, GRAPHICS[graphics_index]);
        }

        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.set_sprite_flags4(slot, 10);
                if self
                    .runtime
                    .sprite_check_damage_to_link_same_layer(self.ram, slot)
                    && self.ram.read_u8(FILTERED_JOYPAD_L) & 0x80 != 0
                {
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.archery_game_guy_show_msg(slot, 0x85);
                }
            }
            1 | 3 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0
                    && self.ram.read_u16(LINK_RUPEES_GOAL) >= 20
                {
                    self.ram.set_sprite_head_direction(slot, 0);
                    self.ram.write_u8(BYTE_7E0B88, 0);
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.archery_game_guy_show_msg(slot, 0x86);
                } else {
                    self.ram.set_sprite_ai_state(slot, 0);
                    self.archery_game_guy_show_msg(slot, 0x87);
                }
            }
            2 => self.archery_game_host_proctor_game(slot),
            _ => {}
        }
    }

    pub fn archery_game_guy_show_msg(&mut self, slot: usize, message: u16) {
        self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, message);
        self.runtime.sprite_show_message_minimal(self.ram);
        self.ram.set_sprite_main_delay(slot, 0);
    }

    pub fn archery_game_host_proctor_game(&mut self, slot: usize) {
        const NUM_SPRITES: [u8; 6] = [5, 4, 3, 2, 1, 0];
        const X: [i8; 18] = [
            0, 0, 0, 0, 48, 48, 48, 48, 8, 8, 16, 16, 24, 24, 32, 32, 40, 40,
        ];
        const Y: [i8; 18] = [-8, 0, 8, 16, -8, 0, 8, 16, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8];
        const CHARACTER: [u8; 18] = [
            0x2b, 0x3b, 0x3b, 0x2b, 0x2b, 0x3b, 0x3b, 0x2b, 0x63, 0x73, 0x63, 0x73, 0x63, 0x73,
            0x63, 0x73, 0x63, 0x73,
        ];
        const FLAGS: [u8; 18] = [
            0x33, 0x33, 0xb3, 0xb3, 0x73, 0x73, 0xf3, 0xf3, 0x32, 0x32, 0x32, 0x32, 0x32, 0x32,
            0x32, 0x32, 0x32, 0x32,
        ];
        if self.ram.sprite(slot).head_direction() == 0 {
            self.ram.write_u8(ARCHERY_GAME_ARROWS_LEFT, 5);
            self.runtime
                .sprite_initialize_secondary_item_minigame(self.ram, 2);
            self.ram.set_sprite_aux1_delay(slot, 39);
            self.ram.write_u16(
                LINK_RUPEES_GOAL,
                self.ram.read_u16(LINK_RUPEES_GOAL).wrapping_sub(20),
            );
            self.ram.set_sprite_head_direction(
                slot,
                self.ram.sprite(slot).head_direction().wrapping_add(1),
            );
        }
        self.runtime.oam_allocate_from_region_a(self.ram, 0x34);
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let aux1 = self.ram.sprite(slot).aux1_delay();
        let sprite_count = if aux1 != 0 {
            NUM_SPRITES[usize::from(aux1 >> 3)]
        } else {
            self.ram.read_u8(ARCHERY_GAME_ARROWS_LEFT)
        };
        let last = usize::from(sprite_count) * 2 + 7;
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for (entry, index) in (0..=last).rev().enumerate() {
            let x = info
                .x
                .wrapping_sub(20)
                .wrapping_add(X[index] as i16 as u16)
                .wrapping_add(1) as u8;
            let y = info
                .y
                .wrapping_sub(48)
                .wrapping_add(Y[index] as i16 as u16)
                .wrapping_add(1) as u8;
            crate::oam::set_plain(
                self.ram,
                first_oam + entry,
                x,
                y,
                CHARACTER[index],
                FLAGS[index],
                0,
            );
        }
        let ancilla_active =
            (0..5).fold(0, |active, i| active | self.ram.read_u8(ANCILLA_TYPE + i));
        if self.ram.read_u8(ARCHERY_GAME_ARROWS_LEFT)
            | self.ram.sprite(slot).aux4_delay()
            | ancilla_active
            != 0
        {
            return;
        }
        self.ram.set_sprite_flags4(slot, 0xa);
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
            && self.ram.read_u8(FILTERED_JOYPAD_L) & 0x80 != 0
        {
            self.archery_game_guy_show_msg(slot, 0x88);
            self.ram.set_sprite_ai_state(slot, 3);
        }
    }

    pub fn archery_game_draw_prize(&mut self, slot: usize) {
        const X: [i8; 5] = [-8, -8, 0, 8, 16];
        const Y: [i8; 5] = [-24, -16, -20, -20, -20];
        const CHARACTER: [u8; 3] = [0xb, 0x1b, 0xb6];
        const FLAGS: [u8; 5] = [0x38, 0x38, 0x34, 0x35, 0x35];
        const CHARACTER_3: [u8; 6] = [0x12, 0x32, 0x31, 3, 0x22, 0x33];
        const CHARACTER_4: [u8; 6] = [0x7c, 0x7c, 0x22, 2, 0x12, 0x33];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam =
            (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4 + 1;
        let prize = usize::from(self.ram.sprite(slot).scratch_b().wrapping_sub(1));
        for (entry, index) in (0..=4).rev().enumerate() {
            let character = match index {
                4 => CHARACTER_4[prize],
                3 => CHARACTER_3[prize],
                _ => CHARACTER[index],
            };
            crate::oam::set_plain(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[index] as i16 as u16) as u8,
                info.y.wrapping_add(Y[index] as i16 as u16) as u8,
                character,
                FLAGS[index] & if character < 0x7c { 0xff } else { 0xfe },
                0,
            );
        }
        self.runtime.sprite_draw_distress_custom(
            self.ram,
            info.x,
            info.y,
            self.ram.read_u8(FRAME_COUNTER),
        );
    }

    pub fn sprite_63_debirando_pit(&mut self, slot: usize) {
        const OPENING_GRAPHICS: [u8; 4] = [5, 4, 3, 3];
        const CLOSING_GRAPHICS: [u8; 4] = [3, 3, 4, 5];
        let (x_difference, y_difference) =
            self.runtime.sprite_direction_to_face_link(self.ram, slot);
        if y_difference.wrapping_add(0x20) < 0x40 && x_difference.wrapping_add(0x20) < 0x40 {
            self.runtime.oam_allocate_from_region_b(self.ram, 16);
        }
        self.debirando_pit_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        let paired = usize::from(self.ram.sprite(slot).head_direction());
        if self.ram.sprite(paired).state() == 6 {
            self.ram
                .set_sprite_state(slot, self.ram.sprite(paired).state());
            self.ram
                .set_sprite_main_delay(slot, self.ram.sprite(paired).main_delay());
            self.ram
                .set_sprite_flags2(slot, self.ram.sprite(slot).flags2().wrapping_add(4));
            return;
        }
        if self.ram.sprite(slot).graphics() < 3
            && self
                .runtime
                .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            self.runtime.link_cancel_dash(self.ram);
            if self.ram.read_u8(FILTERED_JOYPAD_L) & 16 == 0 {
                self.ram.write_u8(LINK_PREVENT_FROM_MOVING, 1);
            }
            self.runtime
                .sprite_apply_speed_towards_link(self.ram, slot, 16);
            let y_velocity = self.ram.sprite(slot).velocity_y();
            let y_magnitude = if (y_velocity as i8) < 0 {
                0_u8.wrapping_sub(y_velocity)
            } else {
                y_velocity
            };
            let y_total = u16::from(y_magnitude) + u16::from(self.ram.sprite(slot).scratch_a());
            self.ram.set_sprite_scratch_a(slot, y_total as u8);
            if y_total >= 256 {
                self.ram.write_u16(
                    DRAG_PLAYER_Y,
                    if (y_velocity as i8) < 0 { 1 } else { 0xffff },
                );
            }
            let x_velocity = self.ram.sprite(slot).velocity_x();
            let x_magnitude = if (x_velocity as i8) < 0 {
                0_u8.wrapping_sub(x_velocity)
            } else {
                x_velocity
            };
            let x_total = u16::from(x_magnitude) + u16::from(self.ram.sprite(slot).scratch_b());
            self.ram.set_sprite_scratch_b(slot, x_total as u8);
            if x_total >= 256 {
                self.ram.write_u16(
                    DRAG_PLAYER_X,
                    if (x_velocity as i8) < 0 { 1 } else { 0xffff },
                );
            }
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.set_sprite_graphics(slot, 6);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.ram.set_sprite_main_delay(slot, 63);
                }
            }
            1 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 0xff);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, OPENING_GRAPHICS[usize::from(delay >> 4)]);
                }
            }
            2 => {
                if self.ram.read_u8(FRAME_COUNTER) & 15 == 0 {
                    let graphics = self.ram.sprite(slot).graphics().wrapping_add(1);
                    self.ram
                        .set_sprite_graphics(slot, if graphics >= 3 { 0 } else { graphics });
                }
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_main_delay(slot, 63);
                    self.ram.set_sprite_ai_state(slot, 3);
                }
            }
            3 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 0);
                    self.ram.set_sprite_main_delay(slot, 32);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, CLOSING_GRAPHICS[usize::from(delay >> 4)]);
                }
            }
            _ => {}
        }
    }

    pub fn debirando_pit_draw(&mut self, slot: usize) {
        const X: [i16; 24] = [
            -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, 0, 8, 0, 8, 0, 8, 0, 8, -8, 8, -8, 8,
        ];
        const Y: [i16; 24] = [
            -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -8, -8, 8, 8,
        ];
        const CHARACTER: [u8; 24] = [
            4, 4, 4, 4, 0x22, 0x22, 0x22, 0x22, 2, 2, 2, 2, 0x29, 0x29, 0x29, 0x29, 0x39, 0x39,
            0x39, 0x39, 0x2a, 0x2a, 0x2a, 0x2a,
        ];
        const FLAGS: [u8; 24] = [
            0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0,
            0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
        ];
        const BIG: [u8; 6] = [2, 2, 2, 0, 0, 2];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        if graphics == 6 {
            return;
        }
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for (entry, piece) in (0..4).rev().enumerate() {
            let index = graphics * 4 + piece;
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[index] as u16),
                info.y.wrapping_add(Y[index] as u16),
                CHARACTER[index],
                FLAGS[index] | info.flags,
                BIG[graphics],
            );
        }
    }

    pub fn sprite_64_debirando(&mut self, slot: usize) {
        const EMERGE_GRAPHICS: [u8; 2] = [1, 0];
        const SUBMERGE_GRAPHICS: [u8; 2] = [0, 1];
        self.debirando_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                let delay = self.ram.sprite(slot).main_delay();
                self.ram.set_sprite_ignore_projectile(slot, delay);
                if delay == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.set_sprite_main_delay(slot, 31);
                }
            }
            1 => {
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.set_sprite_main_delay(slot, 128);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, EMERGE_GRAPHICS[usize::from(delay >> 4)]);
                }
            }
            2 => {
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_main_delay(slot, 31);
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                } else {
                    if delay & 31
                        | self.ram.sprite(slot).scratch_g()
                        | self.ram.read_u8(SUBMODULE_INDEX)
                        | self.ram.sprite(slot).pause()
                        | self.ram.read_u8(FLAG_UNK1)
                        == 0
                    {
                        self.runtime.sprite_spawn_fireball(self.ram, slot);
                    }
                    let subtype = self.ram.sprite(slot).subtype2().wrapping_add(1);
                    self.ram.set_sprite_subtype2(slot, subtype);
                    self.ram.set_sprite_graphics(slot, (subtype >> 3 & 1) + 2);
                }
            }
            3 => {
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 0);
                    self.ram.set_sprite_main_delay(slot, 223);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, SUBMERGE_GRAPHICS[usize::from(delay >> 4)]);
                }
            }
            _ => {}
        }
    }

    pub fn debirando_draw(&mut self, slot: usize) {
        const X: [i8; 16] = [0, 8, 0, 8, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];
        const Y: [i8; 16] = [2, 2, 6, 6, -2, -2, 6, 6, -4, -4, -4, -4, -4, -4, -4, -4];
        const CHARACTER: [u8; 16] = [
            0, 0, 0xd8, 0xd8, 0, 0, 0xd9, 0xd9, 0, 0, 0, 0, 0x20, 0x20, 0x20, 0x20,
        ];
        const FLAGS: [u8; 16] = [1, 0x41, 0, 0x40, 1, 1, 0, 0x40, 1, 1, 1, 1, 1, 1, 1, 1];
        const BIG: [u8; 16] = [0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];
        if self.ram.sprite(slot).ai_state() == 0 {
            return;
        }
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let base = usize::from(self.ram.sprite(slot).graphics()) * 4;
        for (entry, piece) in (0..4).rev().enumerate() {
            let index = base + piece;
            let flags = FLAGS[index];
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[index] as i16 as u16),
                info.y.wrapping_add(Y[index] as i16 as u16),
                CHARACTER[index],
                (flags ^ info.flags) & if flags & 0xf == 0 { 0xf0 } else { 0xff },
                BIG[index],
            );
        }
    }

    pub fn sprite_62_master_sword(&mut self, slot: usize) {
        match self.ram.sprite(slot).subtype2() {
            0 => self.master_sword_main(slot),
            1 => self.sprite_master_sword_light_fountain(slot),
            2 => self.sprite_master_sword_light_beam(slot),
            3 => self.sprite_master_sword_prop(slot),
            4 => self.sprite_master_sword_light_well(slot),
            _ => {}
        }
    }

    pub fn master_sword_main(&mut self, slot: usize) {
        let screen = usize::from(self.ram.read_u8(OVERWORLD_SCREEN_INDEX));
        if self.ram.read_u8(MAIN_MODULE_INDEX) != 26
            && self.ram.read_u8(SAVE_OW_EVENT_INFO + screen) & 0x40 != 0
        {
            self.ram.set_sprite_state(slot, 0);
            return;
        }
        let state = self.ram.sprite(slot).ai_state();
        if state != 5 {
            self.master_sword_draw(slot);
        }
        match state {
            0 => {
                if self.runtime.sprite_check_if_link_is_busy(self.ram)
                    || !self
                        .runtime
                        .sprite_check_damage_to_link_same_layer(self.ram, slot)
                    || self.ram.read_u8(LINK_DIRECTION_FACING) != 2
                    || self.ram.read_u8(FILTERED_JOYPAD_L) & 0x80 == 0
                    || self.ram.read_u8(LINK_WHICH_PENDANTS) & 7 != 7
                {
                    return;
                }
                self.ram.write_u8(MUSIC_CONTROL, 10);
                self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 1);
                self.master_sword_spawn_pendant_prop(slot, 9);
                self.master_sword_spawn_pendant_prop(slot, 11);
                self.master_sword_spawn_pendant_prop(slot, 15);
                self.master_sword_spawn_light_well(slot);
                self.ram.set_sprite_ai_state(slot, 1);
                self.ram.set_sprite_main_delay(slot, 240);
            }
            1 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.master_sword_spawn_light_fountain(slot);
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 192);
                }
                self.ram.write_u8(LINK_UNK_MASTER_SWORD, 10);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
            }
            2 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.master_sword_spawn_light_beam(slot, 0, 0xff);
                    self.ram.set_sprite_ai_state(slot, 3);
                    self.ram.set_sprite_main_delay(slot, 8);
                }
                self.ram.write_u8(LINK_UNK_MASTER_SWORD, 10);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
            }
            3 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.master_sword_spawn_light_beam(slot, 1, 0xff);
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.ram.set_sprite_main_delay(slot, 16);
                }
                self.ram.write_u8(LINK_UNK_MASTER_SWORD, 11);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
            }
            4 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.write_u8(
                        SAVE_OW_EVENT_INFO + screen,
                        self.ram.read_u8(SAVE_OW_EVENT_INFO + screen) | 0x40,
                    );
                    self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
                    self.runtime.link_receive_item(self.ram, 1, 0);
                    self.ram.write_u8(SAVEGAME_MAP_ICONS_INDICATOR, 5);
                    self.ram.write_u8(LINK_UNK_MASTER_SWORD, 0);
                    self.ram.set_sprite_ai_state(slot, 5);
                }
            }
            5 => self.ram.set_sprite_state(slot, 0),
            _ => {}
        }
    }

    pub fn sprite_master_sword_light_fountain(&mut self, slot: usize) {
        const GRAPHICS: [u8; 9] = [0, 1, 1, 2, 2, 2, 1, 1, 0];
        const NUM_LIGHT_BEAMS: [u8; 9] = [0, 0, 1, 1, 2, 2, 0, 0, 0];
        self.sprite_draw_light_fountain(slot);
        let clock = self.ram.sprite(slot).scratch_a().wrapping_add(1);
        self.ram.set_sprite_scratch_a(slot, clock);
        if clock == 0 {
            self.ram
                .set_sprite_scratch_c(slot, self.ram.sprite(slot).scratch_c().wrapping_add(1));
            self.ram.set_sprite_state(slot, 0);
        }
        self.ram.set_sprite_scratch_d(slot, clock >> 2 & 3);
        let frame = usize::from(clock >> 5 & 7);
        self.ram.set_sprite_graphics(slot, GRAPHICS[frame]);
        if NUM_LIGHT_BEAMS[frame] != 0 {
            self.master_sword_spawn_light_beam(slot, clock >> 2 & 1, NUM_LIGHT_BEAMS[frame]);
        }
    }

    pub fn sprite_master_sword_light_well(&mut self, slot: usize) {
        self.sprite_draw_light_fountain(slot);
        let clock = self.ram.sprite(slot).scratch_a().wrapping_add(1);
        self.ram.set_sprite_scratch_a(slot, clock);
        if clock == 0 {
            self.ram
                .set_sprite_scratch_c(slot, self.ram.sprite(slot).scratch_c().wrapping_add(1));
            self.ram.set_sprite_state(slot, 0);
        }
        self.ram.set_sprite_scratch_d(slot, clock >> 2 & 3);
        self.ram.set_sprite_graphics(slot, 0);
    }

    pub fn sprite_draw_light_fountain(&mut self, slot: usize) {
        const DATA: [DrawMultipleData; 12] = [
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x0082,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x4082,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0xc082,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x8082,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x00a0,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x40a0,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0xc0a0,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x80a0,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x0080,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x4080,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0xc080,
                ext: 2,
            },
            DrawMultipleData {
                x: -6,
                y: 4,
                char_flags: 0x8080,
                ext: 2,
            },
        ];
        self.runtime.oam_allocate_from_region_c(self.ram, 4);
        let index = usize::from(self.ram.sprite(slot).graphics()) * 4
            + usize::from(self.ram.sprite(slot).scratch_d());
        self.runtime
            .draw_multiple(self.ram, slot, &DATA[index..index + 1]);
    }

    pub fn master_sword_spawn_light_well(&mut self, slot: usize) {
        let (spawned, info) = self
            .runtime
            .sprite_spawn_dynamically(self.ram, slot, 0x62)
            .expect("MasterSword_SpawnLightWell spawn must succeed");
        self.set_spawned_coordinates(spawned, &info);
        self.ram.set_sprite_subtype2(spawned, 4);
        self.ram.set_sprite_oam_flags(spawned, 5);
        self.ram.set_sprite_flags2(spawned, 0);
    }

    pub fn master_sword_spawn_light_fountain(&mut self, slot: usize) {
        let (spawned, info) = self
            .runtime
            .sprite_spawn_dynamically(self.ram, slot, 0x62)
            .expect("MasterSword_SpawnLightFountain spawn must succeed");
        self.set_spawned_coordinates(spawned, &info);
        self.ram.set_sprite_subtype2(spawned, 1);
        self.ram.set_sprite_oam_flags(spawned, 5);
        self.ram.set_sprite_flags2(spawned, 0);
    }

    pub fn sprite_master_sword_light_beam(&mut self, slot: usize) {
        self.runtime.draw_single_large(self.ram, slot);
        if self.ram.sprite(slot).scratch_a() != 0 {
            self.runtime.sprite_move_xy(self.ram, slot);
            if self.ram.read_u8(FRAME_COUNTER) & 3 != 0 {
                return;
            }
            self.master_sword_spawn_replacement_light_beam(slot);
        }
        let lifetime = self.ram.sprite(slot).scratch_b().wrapping_sub(1);
        self.ram.set_sprite_scratch_b(slot, lifetime);
        if lifetime == 0 {
            self.ram.set_sprite_state(slot, 0);
        }
    }

    pub fn master_sword_spawn_replacement_light_beam(&mut self, slot: usize) {
        let Some((spawned, info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0x62)
        else {
            return;
        };
        self.set_sprite_x(spawned, info.r0_x);
        self.set_sprite_y(spawned, info.r2_y);
        self.ram.set_sprite_subtype2(spawned, 2);
        self.ram.set_sprite_scratch_b(spawned, 3);
        self.ram
            .set_sprite_graphics(spawned, self.ram.sprite(slot).graphics());
        self.ram
            .set_sprite_oam_flags(spawned, self.ram.sprite(slot).oam_flags());
        self.ram.set_sprite_flags2(spawned, 0);
    }

    pub fn master_sword_spawn_light_beam(&mut self, slot: usize, pattern: u8, lifetime: u8) {
        const X_VELOCITY: [[i8; 2]; 4] = [[0, -48], [0, 48], [-96, -48], [96, 48]];
        const Y_VELOCITY: [[i8; 2]; 4] = [[-96, -48], [96, 48], [0, 48], [0, -48]];
        const GRAPHICS: [[u8; 2]; 4] = [[1, 0], [1, 0], [3, 2], [3, 2]];
        const FLAGS: [[u8; 2]; 4] = [[5, 0x45], [5, 0x45], [5, 5], [5, 5]];
        let pattern = usize::from(pattern);
        for beam in 0..4 {
            let Some((spawned, info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0x62)
            else {
                return;
            };
            self.set_sprite_x(spawned, info.r0_x.wrapping_sub(4));
            self.set_sprite_y(spawned, info.r2_y.wrapping_add(4));
            self.ram.set_sprite_subtype2(spawned, 2);
            self.ram.set_sprite_scratch_a(spawned, 2);
            self.ram.set_sprite_flags2(spawned, 0);
            self.ram.set_sprite_signed_field(
                spawned,
                SpriteField::VelocityX,
                X_VELOCITY[beam][pattern],
            );
            self.ram.set_sprite_signed_field(
                spawned,
                SpriteField::VelocityY,
                Y_VELOCITY[beam][pattern],
            );
            self.ram
                .set_sprite_graphics(spawned, GRAPHICS[beam][pattern]);
            self.ram.set_sprite_oam_flags(spawned, FLAGS[beam][pattern]);
            self.ram.set_sprite_scratch_b(spawned, lifetime);
        }
    }

    pub fn master_sword_spawn_pendant_prop(&mut self, slot: usize, oam_flags: u8) {
        const X_VELOCITY: [i8; 4] = [-4, 4, 0, 0];
        const Y_VELOCITY: [i8; 4] = [-2, -2, -4, -4];
        let Some((spawned, _)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0x62) else {
            return;
        };
        self.ram.set_sprite_oam_flags(spawned, oam_flags);
        self.set_sprite_x(spawned, self.ram.read_u16(LINK_X));
        self.set_sprite_y(spawned, self.ram.read_u16(LINK_Y).wrapping_add(8));
        self.ram.set_sprite_graphics(spawned, 4);
        self.ram.set_sprite_subtype2(spawned, 3);
        self.ram.set_sprite_flags2(spawned, 64);
        self.ram.set_sprite_main_delay(spawned, 228);
        let direction = usize::from(oam_flags >> 1 & 3);
        self.ram
            .set_sprite_signed_field(spawned, SpriteField::VelocityX, X_VELOCITY[direction]);
        self.ram
            .set_sprite_signed_field(spawned, SpriteField::VelocityY, Y_VELOCITY[direction]);
    }

    pub fn sprite_master_sword_prop(&mut self, slot: usize) {
        self.runtime.oam_allocate_from_region_b(self.ram, 4);
        self.runtime.draw_single_large(self.ram, slot);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.runtime.sprite_move_xy(self.ram, slot);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.ram.set_sprite_main_delay(slot, 208);
                    self.ram
                        .set_sprite_scratch_a(slot, self.ram.sprite(slot).oam_flags());
                }
            }
            1 => {
                self.ram.set_sprite_oam_flags(
                    slot,
                    (self.ram.sprite(slot).oam_flags() & !0xe)
                        | ((((slot << 1) as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 0xe),
                );
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram
                        .set_sprite_oam_flags(slot, self.ram.sprite(slot).scratch_a());
                }
            }
            2 => {
                self.runtime.sprite_move_xy(self.ram, slot);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_velocity_x(
                        slot,
                        self.ram.sprite(slot).velocity_x().wrapping_shl(1),
                    );
                    self.ram.set_sprite_velocity_y(
                        slot,
                        self.ram.sprite(slot).velocity_y().wrapping_shl(1),
                    );
                    self.ram.set_sprite_main_delay(slot, 6);
                }
                let lifetime = self.ram.sprite(slot).scratch_e().wrapping_add(1);
                self.ram.set_sprite_scratch_e(slot, lifetime);
                if lifetime == 0 {
                    self.ram.set_sprite_state(slot, 0);
                }
            }
            _ => {}
        }
    }

    pub fn master_sword_draw(&mut self, slot: usize) {
        const X: [i8; 6] = [-8, 0, -8, 0, -8, 0];
        const Y: [i8; 6] = [-8, -8, 0, 0, 8, 8];
        const CHARACTER: [u8; 6] = [0xc3, 0xc4, 0xd3, 0xd4, 0xe0, 0xf0];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for (entry, piece) in (0..6).rev().enumerate() {
            crate::oam::write(
                self.ram,
                first_oam + entry,
                z3_core::c_types::OamEnt {
                    x: info.x.wrapping_add(X[piece] as i16 as u16) as u8,
                    y: info.y.wrapping_add(Y[piece] as i16 as u16) as u8,
                    charnum: CHARACTER[piece],
                    flags: info.flags,
                },
            );
        }
        self.runtime
            .sprite_correct_oam_entries(self.ram, slot, 5, 0);
    }

    pub fn sprite_5d_roller_vertical_down_first(&mut self, slot: usize) {
        const XY_VELOCITY: [i8; 6] = [-16, 16, 0, 0, -16, 16];
        self.ram.set_sprite_graphics(
            slot,
            (self.ram.sprite(slot).subtype2() >> 1 & 1) | (self.ram.sprite(slot).scratch_d() & 2),
        );
        self.spike_roller_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        if self.ram.sprite(slot).main_delay() == 0 {
            self.ram.set_sprite_main_delay(slot, 112);
            self.ram
                .set_sprite_scratch_d(slot, self.ram.sprite(slot).scratch_d() ^ 1);
        }
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, XY_VELOCITY[direction]);
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityY, XY_VELOCITY[direction + 2]);
        self.runtime.sprite_move_xy(self.ram, slot);
        self.ram
            .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
    }

    pub fn spike_roller_draw(&mut self, slot: usize) {
        const X: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60,
            0x70, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70,
        ];
        const Y: [u8; 32] = [
            0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60,
            0x70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const CHARACTER: [u8; 32] = [
            0x8e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x8e, 0x8e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e,
            0x9e, 0x8e, 0x88, 0x89, 0x89, 0x89, 0x89, 0x89, 0x89, 0x88, 0x88, 0x89, 0x89, 0x89,
            0x89, 0x89, 0x89, 0x88,
        ];
        const FLAGS: [u8; 32] = [
            0, 0, 0, 0x80, 0, 0, 0, 0x80, 0x40, 0x40, 0x40, 0xc0, 0x40, 0x40, 0x40, 0xc0, 0, 0, 0,
            0x40, 0, 0, 0, 0x40, 0x80, 0x80, 0x80, 0xc0, 0x80, 0x80, 0x80, 0xc0,
        ];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        let mut first_character = CHARACTER[graphics * 8];
        let last_piece = if self.ram.sprite(slot).ai_state() != 0 {
            7
        } else {
            3
        };
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for (entry, piece) in (0..=last_piece).rev().enumerate() {
            let index = graphics * 8 + piece;
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(u16::from(X[index])),
                info.y.wrapping_add(u16::from(Y[index])),
                if first_character != 0 {
                    first_character
                } else {
                    CHARACTER[index]
                },
                FLAGS[index] | info.flags,
                2,
            );
            first_character = 0;
        }
    }

    pub fn sprite_61_beamos(&mut self, slot: usize) {
        match self.ram.sprite(slot).scratch_c() {
            1 => {
                self.sprite_beamos_laser(slot);
                return;
            }
            value if value != 0 => {
                self.sprite_beamos_laser_hit(slot);
                return;
            }
            _ => {}
        }
        self.beamos_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_check_tile_collision(self.ram, slot);
        let _ = self.runtime.sprite_check_damage_to_link(self.ram, slot);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if ((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0 {
                    self.runtime.sprite_spawn_probe_always(
                        self.ram,
                        slot,
                        self.ram.sprite(slot).scratch_d(),
                    );
                    self.ram.set_sprite_scratch_d(
                        slot,
                        self.ram.sprite(slot).scratch_d().wrapping_add(1),
                    );
                }
                self.ram
                    .set_sprite_scratch_d(slot, self.ram.sprite(slot).scratch_d() & 63);
            }
            3 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.ram.set_sprite_main_delay(slot, 80);
                    self.runtime.sprite_prep_load_palette(self.ram, slot);
                } else {
                    if delay == 15 {
                        self.beamos_fire_laser(slot);
                    }
                    self.ram.set_sprite_oam_flags(
                        slot,
                        self.ram.sprite(slot).oam_flags() ^ (delay >> 1 & 0xe),
                    );
                }
            }
            _ => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 0);
                }
            }
        }
    }

    pub fn beamos_fire_laser(&mut self, slot: usize) {
        let instance = self.ram.read_u8(SPRITE_LIMIT_INSTANCE);
        if instance >= 4 {
            return;
        }
        let Some((spawned, info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0x61)
        else {
            return;
        };
        self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x19);
        let x_offset = self.ram.read_i8(DUNGMAP_VAR7) as i16 as u16;
        let y_offset = self.ram.read_i8(DUNGMAP_VAR7 + 1) as i16 as u16;
        self.set_sprite_x(spawned, info.r0_x.wrapping_add(x_offset));
        self.set_sprite_y(spawned, info.r2_y.wrapping_add(y_offset));
        self.runtime
            .sprite_apply_speed_towards_link(self.ram, spawned, 0x20);
        self.ram.set_sprite_flags2(spawned, 0x3f);
        self.ram.set_sprite_flags4(spawned, 0x54);
        self.ram.set_sprite_scratch_c(spawned, 1);
        self.ram.set_sprite_deflection_bits(spawned, 0x48);
        self.ram.set_sprite_oam_flags(spawned, 3);
        self.ram.set_sprite_bump_damage(spawned, 4);
        self.ram.set_sprite_aux1_delay(spawned, 12);
        self.ram.set_sprite_graphics(spawned, instance);
        self.ram
            .write_u8(SPRITE_LIMIT_INSTANCE, instance.wrapping_add(1));
        let history_base = usize::from(instance) * 32;
        for history in history_base..history_base + 32 {
            self.ram
                .write_u8(BEAMOS_X_LO + history, self.ram.sprite(spawned).x_low());
            self.ram
                .write_u8(BEAMOS_X_HI + history, self.ram.sprite(spawned).x_high());
            self.ram
                .write_u8(BEAMOS_Y_LO + history, self.ram.sprite(spawned).y_low());
            self.ram
                .write_u8(BEAMOS_Y_HI + history, self.ram.sprite(spawned).y_high());
        }
    }

    pub fn beamos_draw(&mut self, slot: usize) {
        const Y: [i8; 2] = [-16, 0];
        const CHARACTER: [u8; 2] = [0x48, 0x68];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let sprite_offset = if self.ram.sprite(slot).scratch_d() < 0x20 {
            self.runtime.oam_allocate_from_region_b(self.ram, 12);
            1
        } else {
            self.runtime.oam_allocate_from_region_c(self.ram, 12);
            0
        };
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4
            + sprite_offset;
        for (entry, piece) in (0..2).rev().enumerate() {
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x,
                info.y.wrapping_add(Y[piece] as i16 as u16),
                CHARACTER[piece],
                info.flags,
                2,
            );
        }
        self.sprite_draw_beamos_eyeball(slot, &info);
    }

    pub fn sprite_draw_beamos_eyeball(&mut self, slot: usize, info: &PrepOamCoordsRet) {
        const X: [i8; 32] = [
            -1, 0, 1, 2, 3, 4, 5, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 15, 14, 13, 12, 11, 10, 8,
            7, 5, 4, 3, 2, 1, 0, -2,
        ];
        const Y: [i8; 32] = [
            11, 12, 13, 14, 14, 15, 15, 15, 15, 15, 15, 14, 14, 13, 12, 11, 10, 9, 8, 7, 7, 6, 6,
            6, 6, 6, 6, 7, 7, 8, 9, 10,
        ];
        const CHARACTER: [u8; 32] = [
            0x5b, 0x5b, 0x5a, 0x5a, 0x4b, 0x4b, 0x4a, 0x4a, 0x4a, 0x4a, 0x4b, 0x4b, 0x5a, 0x5a,
            0x5b, 0x5b, 0x5b, 0x5b, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x5b, 0x5b, 0x4c, 0x4c,
            0x4c, 0x4c, 0x4c, 0x4c,
        ];
        const FLAGS: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let oam_offset = if self.ram.sprite(slot).scratch_d() < 0x20 {
            0
        } else {
            2
        };
        let oam_index =
            (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4 + oam_offset;
        let angle = usize::from(self.ram.sprite(slot).scratch_d() >> 1);
        let x_offset = X[angle].wrapping_sub(3) as u8;
        let y_offset = Y[angle].wrapping_sub(18) as u8;
        self.ram.write_u8(DUNGMAP_VAR7, x_offset);
        self.ram.write_u8(DUNGMAP_VAR7 + 1, y_offset);
        crate::oam::write(
            self.ram,
            oam_index,
            z3_core::c_types::OamEnt {
                x: x_offset.wrapping_add(info.x as u8),
                y: y_offset.wrapping_add(info.y as u8),
                charnum: CHARACTER[angle],
                flags: (info.flags & 0x31) | 0xa | FLAGS[angle],
            },
        );
        self.ram.write_u16(
            OAM_CUR_PTR,
            self.ram
                .read_u16(OAM_CUR_PTR)
                .wrapping_add((oam_offset * 4) as u16),
        );
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram
                .read_u16(OAM_EXT_CUR_PTR)
                .wrapping_add(oam_offset as u16),
        );
        self.runtime
            .sprite_correct_oam_entries(self.ram, slot, 0, 0);
    }

    pub fn sprite_beamos_laser(&mut self, slot: usize) {
        if self.ram.sprite(slot).aux1_delay() != 0 {
            return;
        }
        self.beamos_laser_draw(slot);
        if self.ram.sprite(slot).state() == 0 {
            self.ram.write_u8(
                SPRITE_LIMIT_INSTANCE,
                self.ram.read_u8(SPRITE_LIMIT_INSTANCE).wrapping_sub(1),
            );
            return;
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        for _ in 0..4 {
            let subtype = self.ram.sprite(slot).subtype2();
            self.ram.set_sprite_subtype2(slot, subtype.wrapping_add(1));
            let history =
                usize::from(subtype & 31) + usize::from(self.ram.sprite(slot).graphics()) * 32;
            self.ram
                .write_u8(BEAMOS_Y_HI + history, self.ram.sprite(slot).y_high());
            self.ram
                .write_u8(BEAMOS_Y_LO + history, self.ram.sprite(slot).y_low());
            self.ram
                .write_u8(BEAMOS_X_HI + history, self.ram.sprite(slot).x_high());
            self.ram
                .write_u8(BEAMOS_X_LO + history, self.ram.sprite(slot).x_low());
            self.runtime.sprite_move_xy(self.ram, slot);
        }
        let delay = self.ram.sprite(slot).main_delay();
        if delay != 0 {
            if delay == 1 {
                self.ram.set_sprite_state(slot, 0);
                self.ram.write_u8(
                    SPRITE_LIMIT_INSTANCE,
                    self.ram.read_u8(SPRITE_LIMIT_INSTANCE).wrapping_sub(1),
                );
            }
            return;
        }
        if !self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
            && self.runtime.sprite_check_tile_collision(self.ram, slot) == 0
        {
            return;
        }
        self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x26);
        self.ram.set_sprite_main_delay(slot, 16);
        self.ram.set_sprite_velocity_x(slot, 0);
        self.ram.set_sprite_velocity_y(slot, 0);
        if let Some((spawned, info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0x61) {
            self.set_spawned_coordinates(spawned, &info);
            self.ram.set_sprite_main_delay(spawned, 16);
            self.ram.set_sprite_flags2(spawned, 3);
            self.ram.set_sprite_scratch_c(spawned, 2);
            self.ram.set_sprite_flags3(spawned, 0x40);
        }
        self.ram.set_sprite_y_high(slot, 128);
    }

    pub fn beamos_laser_draw(&mut self, slot: usize) {
        let info = self.runtime.prep_oam_coord(self.ram, slot);
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for (entry, piece) in (0..32).rev().enumerate() {
            let history = graphics * 32 + piece;
            let x = (u16::from(self.ram.read_u8(BEAMOS_X_LO + history))
                | (u16::from(self.ram.read_u8(BEAMOS_X_HI + history)) << 8))
                .wrapping_sub(self.ram.read_u16(BG2_HOFS_COPY2));
            let y = (u16::from(self.ram.read_u8(BEAMOS_Y_LO + history))
                | (u16::from(self.ram.read_u8(BEAMOS_Y_HI + history)) << 8))
                .wrapping_sub(self.ram.read_u16(BG2_VOFS_COPY2));
            crate::oam::set_helper0(self.ram, first_oam + entry, x, y, 0x5c, info.flags, 0);
        }
    }

    pub fn sprite_beamos_laser_hit(&mut self, slot: usize) {
        const X: [i8; 4] = [-4, 4, -4, 4];
        const Y: [i8; 4] = [-4, -4, 4, 4];
        const FLAGS: [u8; 4] = [6, 0x46, 0x86, 0xc6];
        if self.ram.sprite(slot).main_delay() == 0 {
            self.ram.set_sprite_state(slot, 0);
        }
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for (entry, piece) in (0..4).rev().enumerate() {
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[piece] as i16 as u16),
                info.y.wrapping_add(Y[piece] as i16 as u16),
                0xd6,
                FLAGS[piece] | (info.flags & 0x30),
                0,
            );
        }
    }

    pub fn sprite_5b_spark_clockwise(&mut self, slot: usize) {
        const OAM_FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];
        const DIRECTIONS: [u8; 8] = [1, 3, 2, 0, 7, 5, 6, 4];
        const X_VELOCITY: [i8; 8] = [1, 1, -1, -1, -1, -1, 1, 1];
        const Y_VELOCITY: [i8; 8] = [-1, 1, 1, -1, -1, 1, 1, -1];
        const X_VELOCITY_2: [i8; 8] = [8, 0, -8, 0, -8, 0, 8, 0];
        const Y_VELOCITY_2: [i8; 8] = [0, 8, 0, -8, 0, 8, 0, -8];
        const MASK_2: [u8; 8] = [1, 4, 2, 8, 2, 4, 1, 8];
        const MASK: [u8; 8] = [8, 1, 4, 2, 8, 2, 4, 1];
        const NEXT_2: [u8; 8] = [1, 2, 3, 0, 5, 6, 7, 4];
        const NEXT: [u8; 8] = [3, 0, 1, 2, 7, 4, 5, 6];
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
            self.ram
                .set_sprite_oam_flags(slot, self.ram.sprite(slot).oam_flags() ^ 6);
        }
        if self.ram.sprite(slot).ai_state() == 0 {
            self.ram.set_sprite_ai_state(slot, 1);
            self.ram.set_sprite_velocity_x(slot, 1);
            self.ram.set_sprite_velocity_y(slot, 1);
            let mut collision = self.runtime.sprite_check_tile_collision(self.ram, slot);
            self.ram
                .set_sprite_signed_field(slot, SpriteField::VelocityX, -1);
            self.ram
                .set_sprite_signed_field(slot, SpriteField::VelocityY, -1);
            collision |= self.runtime.sprite_check_tile_collision(self.ram, slot);
            let direction = if collision < 4 {
                usize::from(if collision & 1 != 0 { 0_u8 } else { 1 })
            } else {
                usize::from(if collision & 4 != 0 { 2_u8 } else { 3 })
            };
            let kind_offset = usize::from(self.ram.sprite(slot).kind() != 0x5c) * 4;
            self.ram
                .set_sprite_scratch_d(slot, DIRECTIONS[kind_offset + direction]);
        }
        self.ram.set_sprite_oam_flags(
            slot,
            (self.ram.sprite(slot).oam_flags() & 0x3f)
                | OAM_FLAGS[usize::from(self.ram.read_u8(FRAME_COUNTER) >> 2 & 3)],
        );
        self.runtime.sprite_move_xy(self.ram, slot);
        let _ = self.runtime.sprite_check_damage_to_link(self.ram, slot);
        let mut direction = usize::from(self.ram.sprite(slot).scratch_d());
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, X_VELOCITY[direction]);
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityY, Y_VELOCITY[direction]);
        self.runtime.sprite_check_tile_collision(self.ram, slot);
        direction = usize::from(self.ram.sprite(slot).scratch_d());
        let aux2 = self.ram.sprite(slot).aux2_delay();
        if aux2 != 0 {
            if aux2 == 6 {
                direction = usize::from(NEXT[direction]);
            }
        } else if self.ram.sprite(slot).wall_collision() & MASK[direction] == 0 {
            self.ram.set_sprite_aux2_delay(slot, 10);
        }
        if self.ram.sprite(slot).wall_collision() & MASK_2[direction] != 0 {
            direction = usize::from(NEXT_2[direction]);
        }
        self.ram.set_sprite_scratch_d(slot, direction as u8);
        self.ram.set_sprite_signed_field(
            slot,
            SpriteField::VelocityX,
            X_VELOCITY_2[direction].wrapping_mul(2),
        );
        self.ram.set_sprite_signed_field(
            slot,
            SpriteField::VelocityY,
            Y_VELOCITY_2[direction].wrapping_mul(2),
        );
    }

    pub fn sprite_59_lost_woods_bird(&mut self, slot: usize) {
        if self.ram.sprite(slot).aux1_delay() != 0 {
            return;
        }
        let horizontal_flip = if self.ram.sprite(slot).signed_field(SpriteField::VelocityX) < 0 {
            0
        } else {
            0x40
        };
        self.ram.set_sprite_oam_flags(
            slot,
            (self.ram.sprite(slot).oam_flags() & !0x40) | horizontal_flip,
        );
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        self.runtime.sprite_move_z(self.ram, slot);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.set_sprite_graphics(slot, 0);
                let z_velocity = self.ram.sprite(slot).velocity_z().wrapping_sub(1);
                self.ram.set_sprite_velocity_z(slot, z_velocity);
                if (z_velocity.wrapping_sub(0xf1) as i8) < 0 {
                    self.ram.set_sprite_ai_state(slot, 1);
                }
            }
            1 => {
                let z_velocity = self.ram.sprite(slot).velocity_z().wrapping_add(2);
                self.ram.set_sprite_velocity_z(slot, z_velocity);
                if (z_velocity.wrapping_sub(0x10) as i8) >= 0 {
                    self.ram.set_sprite_ai_state(slot, 0);
                }
                let animation = self.ram.sprite(slot).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(slot, animation);
                self.ram.set_sprite_graphics(slot, animation >> 1 & 1);
            }
            _ => {}
        }
    }

    pub fn sprite_5a_lost_woods_squirrel(&mut self, slot: usize) {
        if self.ram.sprite(slot).aux1_delay() != 0 {
            return;
        }
        let horizontal_flip = if self.ram.sprite(slot).signed_field(SpriteField::VelocityX) < 0 {
            0
        } else {
            0x40
        };
        self.ram.set_sprite_oam_flags(
            slot,
            (self.ram.sprite(slot).oam_flags() & !0x40) | horizontal_flip,
        );
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        self.runtime.sprite_move_z(self.ram, slot);
        self.ram
            .set_sprite_velocity_z(slot, self.ram.sprite(slot).velocity_z().wrapping_sub(2));
        if self.ram.sprite(slot).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_z(slot, 0);
            self.ram.set_sprite_velocity_z(slot, 16);
            self.ram.set_sprite_main_delay(slot, 12);
        }
        self.ram
            .set_sprite_graphics(slot, u8::from(self.ram.sprite(slot).main_delay() != 0));
    }

    pub fn sprite_58_crab(&mut self, slot: usize) {
        const X_VELOCITY: [i8; 4] = [28, -28, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, 12, -12];
        self.crab_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || self.runtime.sprite_return_if_recoiling(self.ram, slot)
        {
            return;
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        self.runtime.sprite_move_xy(self.ram, slot);
        if self.runtime.sprite_check_tile_collision(self.ram, slot) != 0
            || self.ram.sprite(slot).main_delay() == 0
        {
            let delay = (crate::misc::get_random_number(self.ram) & 63).wrapping_add(32);
            self.ram.set_sprite_main_delay(slot, delay);
            self.ram.set_sprite_scratch_d(slot, delay & 3);
        }
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, X_VELOCITY[direction & 3]);
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityY, Y_VELOCITY[direction & 3]);
        let animation = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, animation);
        self.ram
            .set_sprite_graphics(slot, animation >> if direction < 2 { 1 } else { 3 } & 1);
    }

    pub fn crab_draw(&mut self, slot: usize) {
        const X: [i16; 4] = [-8, 8, -8, 8];
        const CHARACTER: [u8; 4] = [0x8e, 0x8e, 0xae, 0xae];
        const FLAGS: [u8; 4] = [0, 0x40, 0, 0x40];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let base = usize::from(self.ram.sprite(slot).graphics()) * 2;
        for (entry, piece) in (0..2).rev().enumerate() {
            let index = base + piece;
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[index] as u16),
                info.y,
                CHARACTER[index],
                FLAGS[index] | info.flags,
                2,
            );
        }
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_57_desert_statue(&mut self, slot: usize) {
        const NEXT_DIRECTION: [u8; 4] = [3, 2, 0, 1];
        self.desert_barrier_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        let damages_link = self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot);
        if damages_link {
            self.runtime.sprite_nullify_hookshot_drag(self.ram);
            self.runtime.sprite_repel_dash(self.ram);
        }
        if self.ram.sprite(slot).main_delay() != 0
            || self.ram.sprite(slot).signed_field(SpriteField::AiState) < 0
        {
            return;
        }
        if self.ram.sprite(slot).ai_state() == 0 {
            let activation = self.ram.read_u8(BYTE_7E02F0);
            if activation == 0 {
                return;
            }
            self.ram.set_sprite_ai_state(slot, activation);
            self.ram.set_sprite_main_delay(slot, 128);
            self.ram.write_u8(SOUND_EFFECT_AMBIENT, 7);
        }
        if damages_link && self.ram.read_u8(LINK_INCAPACITATED_TIMER) == 0 {
            self.ram.write_u8(LINK_INCAPACITATED_TIMER, 16);
            self.runtime
                .sprite_apply_speed_towards_link(self.ram, slot, 32);
        }
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        self.ram.set_sprite_signed_field(
            slot,
            SpriteField::VelocityX,
            DESERT_BARRIER_X_VELOCITY[direction],
        );
        self.ram.set_sprite_signed_field(
            slot,
            SpriteField::VelocityY,
            DESERT_BARRIER_Y_VELOCITY[direction],
        );
        self.runtime.sprite_move_xy(self.ram, slot);
        if self.runtime.sprite_check_tile_collision(self.ram, slot) != 0 {
            self.ram
                .set_sprite_scratch_d(slot, NEXT_DIRECTION[direction]);
        }
        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
        let subtype = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, subtype);
        if subtype & 1 == 0 {
            let progress = self.ram.sprite(slot).scratch_g().wrapping_add(1);
            self.ram.set_sprite_scratch_g(slot, progress);
            if progress == 130 {
                self.ram.set_sprite_ai_state(slot, 128);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
            }
        }
    }

    pub fn desert_barrier_draw(&mut self, slot: usize) {
        const DATA: [DrawMultipleData; 4] = [
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x008e,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x408e,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x00ae,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0x40ae,
                ext: 2,
            },
        ];
        let delay = self.ram.sprite(slot).main_delay();
        if delay == 1 {
            self.ram.write_u8(SOUND_EFFECT_2, 0x1b);
            self.ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
        }
        self.ram.write_u8(
            CUR_SPRITE_X,
            self.ram.read_u8(CUR_SPRITE_X).wrapping_add(delay >> 1 & 1),
        );
        let (x_difference, y_difference) =
            self.runtime.sprite_direction_to_face_link(self.ram, slot);
        if x_difference.wrapping_add(0x20) < 0x40 && y_difference.wrapping_add(0x20) < 0x40 {
            self.runtime.oam_allocate_from_region_b(self.ram, 16);
        }
        self.runtime.draw_multiple(self.ram, slot, &DATA);
    }

    pub fn sprite_55_zora(&mut self, slot: usize) {
        if self.ram.sprite(slot).scratch_e() != 0 {
            self.sprite_fireball(slot);
        } else {
            self.sprite_zora_main(slot);
        }
    }

    pub fn sprite_fireball(&mut self, slot: usize) {
        const OFFSETS: [u8; 4] = [3, 2, 0, 0];
        const X: [i8; 4] = [4, 4, -4, 16];
        const Y: [i8; 4] = [0, 16, 8, 8];
        const WIDTH: [u8; 4] = [8, 8, 4, 4];
        const HEIGHT: [u8; 4] = [4, 4, 8, 8];
        self.ram
            .set_sprite_ignore_projectile(slot, self.ram.sprite(slot).scratch_e());
        if self.ram.sprite(slot).main_delay() != 0 {
            self.runtime.oam_allocate_from_region_c(self.ram, 4);
        }
        self.runtime.draw_single_small(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.fireball_spawn_trail_garnish(self.ram, slot);
        if self.runtime.sprite_check_damage_to_link(self.ram, slot) {
            self.ram.set_sprite_state(slot, 0);
            return;
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        if self.ram.read_u8(PLAYER_IS_INDOORS) != 0
            && self.ram.sprite(slot).aux1_delay() == 0
            && ((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0
            && self.runtime.sprite_check_tile_collision(self.ram, slot) != 0
        {
            self.ram.set_sprite_state(slot, 0);
            return;
        }
        if self.ram.read_u8(LINK_IS_BUNNY_MIRROR) | self.ram.read_u8(LINK_DISABLE_SPRITE_DAMAGE)
            != 0
            || self.ram.read_i8(LINK_STATE_BITS) < 0
            || self.ram.read_u8(LINK_SHIELD_TYPE) < 2
            || self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL) != self.ram.sprite(slot).floor()
        {
            return;
        }
        let facing = usize::from(self.ram.read_u8(LINK_DIRECTION_FACING) >> 1);
        let hitbox = if self.ram.read_u8(BUTTON_B_FRAMES) != 0 {
            usize::from(OFFSETS[facing])
        } else {
            facing
        };
        let x = self
            .ram
            .read_u16(LINK_X)
            .wrapping_add(X[hitbox] as i16 as u16);
        let y = self
            .ram
            .read_u16(LINK_Y)
            .wrapping_add(Y[hitbox] as i16 as u16);
        if self.runtime.zora_fireball_shield_overlap(
            self.ram,
            slot,
            x,
            y,
            WIDTH[hitbox],
            HEIGHT[hitbox],
        ) {
            self.runtime.sprite_place_repulse_spark_2(self.ram, slot);
            self.ram.set_sprite_state(slot, 0);
            self.runtime.queue_sfx2_with_pan(self.ram, slot, 6);
        }
    }

    pub fn sprite_zora_main(&mut self, slot: usize) {
        const SURFACE_XY: [i8; 8] = [-32, -24, -16, -8, 8, 16, 24, 32];
        const SURFACING_GRAPHICS: [u8; 16] = [4, 3, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 0, 0];
        const ATTACK_GRAPHICS: [u8; 8] = [5, 5, 6, 10, 6, 5, 5, 5];
        const SUBMERGE_GRAPHICS: [u8; 12] = [12, 11, 9, 8, 7, 0, 0, 0, 0, 0, 0, 0];
        if self.ram.sprite(slot).ai_state() == 0 {
            self.runtime.prep_oam_coord(self.ram, slot);
        } else {
            self.zora_draw(slot);
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                let delay = self.ram.sprite(slot).main_delay();
                self.ram.set_sprite_ignore_projectile(slot, delay);
                if delay == 0 {
                    let original_x = u16::from(self.ram.sprite(slot).scratch_a())
                        | (u16::from(self.ram.sprite(slot).scratch_b()) << 8);
                    let original_y = u16::from(self.ram.sprite(slot).scratch_c())
                        | (u16::from(self.ram.sprite(slot).head_direction()) << 8);
                    let x_offset =
                        SURFACE_XY[usize::from(crate::misc::get_random_number(self.ram) & 7)];
                    let y_offset =
                        SURFACE_XY[usize::from(crate::misc::get_random_number(self.ram) & 7)];
                    self.set_sprite_x(slot, original_x.wrapping_add(x_offset as i16 as u16));
                    self.set_sprite_y(slot, original_y.wrapping_add(y_offset as i16 as u16));
                    self.ram.write_u16(CUR_SPRITE_X, self.sprite_x(slot));
                    self.ram.write_u16(CUR_SPRITE_Y, self.sprite_y(slot));
                    let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
                    let tile_type = self.ram.read_u8(SPRITE_TILETYPE);
                    let spawn = tile_type == 8
                        || (self.ram.read_u16(ENHANCED_FEATURES0) & 16384 != 0
                            && tile_type == 9
                            && self.ram.sprite(slot).aux2_delay() == 1);
                    if spawn {
                        self.ram.set_sprite_main_delay(slot, 127);
                        self.ram.set_sprite_ai_state(
                            slot,
                            self.ram.sprite(slot).ai_state().wrapping_add(1),
                        );
                        self.ram
                            .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() | 0x40);
                    } else if self.ram.read_u16(ENHANCED_FEATURES0) & 16384 != 0 {
                        self.set_sprite_x(slot, original_x);
                        self.set_sprite_y(slot, original_y);
                        self.ram.set_sprite_ignore_projectile(slot, 1);
                        if self.ram.sprite(slot).aux2_delay() == 0 {
                            self.ram.set_sprite_aux2_delay(slot, 32);
                        }
                    }
                }
            }
            1 => {
                let delay = self.ram.sprite(slot).main_delay();
                self.ram.set_sprite_ignore_projectile(slot, delay);
                if delay == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.set_sprite_main_delay(slot, 127);
                    self.ram
                        .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() & !0x40);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, SURFACING_GRAPHICS[usize::from(delay >> 3)]);
                }
            }
            2 => {
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.set_sprite_main_delay(slot, 23);
                } else {
                    if delay == 48 {
                        self.runtime.sprite_spawn_fireball(self.ram, slot);
                    }
                    self.ram
                        .set_sprite_graphics(slot, ATTACK_GRAPHICS[usize::from(delay >> 4)]);
                }
            }
            3 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_main_delay(slot, 128);
                    self.ram.set_sprite_graphics(slot, 0);
                    self.ram.set_sprite_ai_state(slot, 0);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, SUBMERGE_GRAPHICS[usize::from(delay >> 2)]);
                }
            }
            _ => {}
        }
    }

    pub fn zora_draw(&mut self, slot: usize) {
        const X: [i8; 26] = [
            4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, 11, 0, 4, -8, 18, -8, 18,
        ];
        const Y: [i8; 26] = [
            4, 4, 0, 0, 0, 0, 0, -3, 0, -3, -3, -3, -3, -3, -3, -3, -6, -6, -8, -9, -3, 5, -10,
            -11, -10, -11,
        ];
        const CHARACTER: [u8; 26] = [
            0xa8, 0xa8, 0x88, 0x88, 0x88, 0x88, 0x88, 0xa4, 0x88, 0xa4, 0xa4, 0xa4, 0xa6, 0xa6,
            0xa4, 0xc0, 0x8a, 0x8a, 0xae, 0xaf, 0xa6, 0x8d, 0xcf, 0xcf, 0xdf, 0xdf,
        ];
        const FLAGS: [u8; 26] = [
            0x25, 0x25, 0x25, 0x25, 0xe5, 0xe5, 0x25, 0x20, 0xe5, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x24, 0x25, 0x25, 0x24, 0x64, 0x20, 0x26, 0x24, 0x64, 0x24, 0x64,
        ];
        const BIG: [u8; 26] = [
            0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 2, 0, 0, 0, 0, 0,
        ];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let base = usize::from(self.ram.sprite(slot).graphics()) * 2;
        for (entry, piece) in (0..2).rev().enumerate() {
            let index = base + piece;
            let flags = FLAGS[index];
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[index] as i16 as u16),
                info.y.wrapping_add(Y[index] as i16 as u16),
                CHARACTER[index],
                flags | if flags & 0xf != 0 { 0 } else { info.flags },
                BIG[index],
            );
        }
    }

    pub fn sprite_52_king_zora(&mut self, slot: usize) {
        self.zora_king_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self
                    .ram
                    .read_u16(LINK_X)
                    .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                    .wrapping_add(16)
                    < 32
                    && self
                        .ram
                        .read_u16(LINK_Y)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                        .wrapping_add(48)
                        < 96
                {
                    self.runtime.link_cancel_dash(self.ram);
                    self.ram.set_sprite_main_delay(slot, 127);
                    self.ram.write_u8(SOUND_EFFECT_1, 0x35);
                    self.ram.set_sprite_ai_state(slot, 1);
                    for other in (0..16).rev() {
                        if other != slot && self.ram.sprite(other).deflection_bits() & 0x80 == 0 {
                            if self.ram.sprite(other).state() == 10 {
                                self.ram.write_u8(LINK_STATE_BITS, 0);
                                self.ram.write_u8(LINK_PICKING_THROW_STATE, 0);
                            }
                            self.runtime.sprite_kill_self(self.ram, other);
                        }
                    }
                }
            }
            1 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 127);
                    self.ram.write_u16(BG1_X_OFFSET, 0);
                    self.ram.set_sprite_graphics(slot, 4);
                } else {
                    self.ram
                        .write_u16(BG1_X_OFFSET, if delay & 1 != 0 { 0xffff } else { 1 });
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                }
            }
            2 => {
                const SURFACING_GRAPHICS: [u8; 16] =
                    [0, 0, 0, 3, 9, 8, 7, 6, 9, 8, 7, 6, 5, 4, 5, 4];
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 3);
                    self.ram.set_sprite_main_delay(slot, 127);
                } else {
                    if delay == 28 {
                        self.ram.set_sprite_aux2_delay(slot, 15);
                        self.sprite_spawn_big_splash(slot);
                    }
                    self.ram
                        .set_sprite_graphics(slot, SURFACING_GRAPHICS[usize::from(delay >> 3)]);
                }
            }
            3 => {
                const DIALOGUE_GRAPHICS: [u8; 8] = [0, 0, 1, 2, 1, 2, 0, 0];
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.ram.set_sprite_main_delay(slot, 36);
                    return;
                }
                self.ram
                    .set_sprite_graphics(slot, DIALOGUE_GRAPHICS[usize::from(delay >> 4)]);
                match delay {
                    80 => {
                        self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x142);
                        self.runtime.sprite_show_message_minimal(self.ram);
                    }
                    79 => {
                        self.ram.write_u16(
                            DIALOGUE_MESSAGE_INDEX,
                            if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0 {
                                0x143
                            } else {
                                0x146
                            },
                        );
                        self.runtime.sprite_show_message_minimal(self.ram);
                        if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) != 0 {
                            self.ram.set_sprite_main_delay(slot, 0x30);
                        }
                    }
                    78 => {
                        if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0
                            && self.ram.read_u16(LINK_RUPEES_GOAL) >= 500
                        {
                            self.ram.write_u16(
                                LINK_RUPEES_GOAL,
                                self.ram.read_u16(LINK_RUPEES_GOAL).wrapping_sub(500),
                            );
                            self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x144);
                            self.runtime.sprite_show_message_minimal(self.ram);
                            self.ram.set_sprite_scratch_e(slot, 1);
                        } else {
                            self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x145);
                            self.runtime.sprite_show_message_minimal(self.ram);
                            self.ram.set_sprite_main_delay(slot, 0x30);
                        }
                    }
                    77 if self.ram.sprite(slot).scratch_e() != 0 => self
                        .runtime
                        .sprite_zora_regurgitate_flippers(self.ram, slot),
                    _ => {}
                }
            }
            4 => {
                const SUBMERGE_GRAPHICS: [u8; 21] = [
                    12, 12, 12, 12, 12, 12, 11, 11, 11, 11, 11, 10, 10, 10, 10, 3, 3, 3, 3, 3, 3,
                ];
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.runtime.sprite_kill_self(self.ram, slot);
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                } else {
                    if delay == 29 {
                        self.ram.set_sprite_aux2_delay(slot, 15);
                        self.sprite_spawn_big_splash(slot);
                    }
                    self.ram
                        .set_sprite_graphics(slot, SUBMERGE_GRAPHICS[usize::from(delay >> 1)]);
                }
            }
            _ => {}
        }
    }

    pub fn sprite_spawn_big_splash(&mut self, slot: usize) {
        const X: [i8; 8] = [-8, -5, 4, 13, 16, 13, 4, -5];
        const Y: [i8; 8] = [4, -5, -8, -5, 4, 13, 16, 13];
        const X_VELOCITY: [i8; 8] = [-8, -6, 0, 6, 8, 6, 0, -6];
        const Y_VELOCITY: [i8; 8] = [0, -6, -8, -6, 0, 6, 8, 6];
        self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x24);
        for index in (0..8).rev() {
            if let Some((spawned, info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 8)
            {
                self.ram.set_sprite_state(spawned, 3);
                self.set_sprite_x(
                    spawned,
                    info.r0_x
                        .wrapping_add(X[index] as i16 as u16)
                        .wrapping_sub(4),
                );
                self.set_sprite_y(
                    spawned,
                    info.r2_y
                        .wrapping_add(Y[index] as i16 as u16)
                        .wrapping_sub(4),
                );
                self.ram.set_sprite_signed_field(
                    spawned,
                    SpriteField::VelocityX,
                    X_VELOCITY[index],
                );
                self.ram.set_sprite_signed_field(
                    spawned,
                    SpriteField::VelocityY,
                    Y_VELOCITY[index],
                );
                self.ram.set_sprite_scratch_a(spawned, index as u8);
                let z_velocity = (crate::misc::get_random_number(self.ram) & 15).wrapping_add(24);
                self.ram.set_sprite_velocity_z(spawned, z_velocity);
                self.ram.set_sprite_ai_state(spawned, 1);
                self.ram.set_sprite_z(spawned, 0);
                self.ram
                    .set_sprite_flags3(spawned, self.ram.sprite(spawned).flags3() | 0x40);
                self.ram
                    .set_sprite_ignore_projectile(spawned, self.ram.sprite(spawned).flags3());
            }
        }
    }

    pub fn zora_king_draw(&mut self, slot: usize) {
        let Some(mut info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        if self.ram.sprite(slot).ai_state() >= 2 {
            const X: [i8; 52] = [
                -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, 0, 0, 0, 0, 0, 0, 0, 0, -8,
                8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -9, 9, -9, 9, -10, 10, -10, 10,
                -11, 11, -11, 11,
            ];
            const Y: [i8; 52] = [
                -18, -18, -2, -2, -18, -18, -2, -2, -18, -18, -2, -2, -12, -12, 4, 4, 0, 0, 0, 0,
                0, 0, 0, 0, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -5, -5, 5, 5,
                -5, -5, 5, 5, -5, -5, 5, 5,
            ];
            const CHARACTER: [u8; 52] = [
                0xc0, 0xc0, 0xe0, 0xe0, 0xc2, 0xea, 0xe2, 0xe2, 0xea, 0xc2, 0xe2, 0xe2, 0xc0, 0xc0,
                0xe4, 0xe6, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0xc4, 0xc6, 0xe4, 0xe6,
                0xc6, 0xc4, 0xe6, 0xe4, 0xe6, 0xe4, 0xc6, 0xc4, 0xe4, 0xe6, 0xc4, 0xc6, 0x88, 0x88,
                0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88,
            ];
            const FLAGS: [u8; 52] = [
                0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 5, 5, 5, 5, 5, 5,
                0xc5, 0xc5, 0xc5, 0xc5, 5, 5, 5, 5, 0x45, 0x45, 0x45, 0x45, 0xc5, 0xc5, 0xc5, 0xc5,
                0x85, 0x85, 0x85, 0x85, 4, 0x44, 0x84, 0xc4, 4, 0x44, 0x84, 0xc4, 4, 0x44, 0x84,
                0xc4,
            ];
            let first_oam =
                (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
            let base = usize::from(self.ram.sprite(slot).graphics()) * 4;
            for (entry, piece) in (0..4).rev().enumerate() {
                let index = base + piece;
                let flags = FLAGS[index];
                crate::oam::write(
                    self.ram,
                    first_oam + entry,
                    z3_core::c_types::OamEnt {
                        x: info.x.wrapping_add(X[index] as i16 as u16) as u8,
                        y: info.y.wrapping_add(Y[index] as i16 as u16) as u8,
                        charnum: CHARACTER[index],
                        flags: (if flags & 0xf != 0 {
                            flags
                        } else {
                            flags | info.flags
                        }) | 0x20,
                    },
                );
            }
            self.runtime
                .sprite_correct_oam_entries(self.ram, slot, 3, 2);
            let Some(next_info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
                return;
            };
            info = next_info;
        }
        let splash_timer = self.ram.sprite(slot).aux2_delay();
        if splash_timer == 0 {
            return;
        }
        const X: [i8; 8] = [-23, 23, 23, 23, -20, -15, 13, 18];
        const Y: [i8; 8] = [-8, -8, -8, -8, -7, 0, 0, -7];
        const CHARACTER: [u8; 8] = [0xae, 0xae, 0xae, 0xae, 0xac, 0xac, 0xac, 0xac];
        const FLAGS: [u8; 8] = [0, 0x40, 0x40, 0x40, 0, 0, 0x40, 0x40];
        self.runtime.oam_allocate_from_region_c(self.ram, 0x10);
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let base = usize::from(splash_timer >> 1 & 4);
        for (entry, piece) in (0..4).rev().enumerate() {
            let index = base + piece;
            crate::oam::set_plain(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[index] as i16 as u16) as u8,
                info.y.wrapping_add(Y[index] as i16 as u16) as u8,
                CHARACTER[index],
                FLAGS[index] | 0x24,
                2,
            );
        }
    }

    pub fn sprite_56_walking_zora(&mut self, slot: usize) {
        const SURFACING_GRAPHICS: [u8; 16] = [4, 3, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 0, 0];
        const RECRUIT_GRAPHICS: [u8; 8] = [0, 2, 4, 6, 1, 3, 5, 7];
        if self.ram.sprite(slot).scratch_f() != 0 {
            self.ram.set_sprite_scratch_f(slot, 0);
            self.ram.set_sprite_scratch_b(slot, 3);
            self.ram.set_sprite_scratch_g(slot, 192);
            self.ram.set_sprite_signed_field(
                slot,
                SpriteField::VelocityX,
                self.ram.sprite(slot).signed_field(SpriteField::RecoilX) >> 1,
            );
            self.ram.set_sprite_signed_field(
                slot,
                SpriteField::VelocityY,
                self.ram.sprite(slot).signed_field(SpriteField::RecoilY) >> 1,
            );
        }
        match self.ram.sprite(slot).scratch_b() {
            0 => {
                self.runtime.prep_oam_coord(self.ram, slot);
                if self.runtime.sprite_return_if_inactive(self.ram, slot) {
                    return;
                }
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_main_delay(slot, 127);
                    self.ram.set_sprite_scratch_b(
                        slot,
                        self.ram.sprite(slot).scratch_b().wrapping_add(1),
                    );
                    self.ram
                        .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() | 0x40);
                }
            }
            1 => {
                self.zora_draw(slot);
                if self.runtime.sprite_return_if_inactive(self.ram, slot) {
                    return;
                }
                let delay = self.ram.sprite(slot).main_delay();
                self.ram.set_sprite_ignore_projectile(slot, delay);
                if delay == 0 {
                    self.ram
                        .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() & !0x40);
                    self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x28);
                    self.ram.set_sprite_scratch_b(
                        slot,
                        self.ram.sprite(slot).scratch_b().wrapping_add(1),
                    );
                    self.ram.set_sprite_velocity_z(slot, 48);
                    let direction = self
                        .runtime
                        .sprite_direction_to_face_link_direction(self.ram, slot);
                    self.ram.set_sprite_head_direction(slot, direction);
                    self.ram.set_sprite_scratch_d(slot, direction);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, SURFACING_GRAPHICS[usize::from(delay >> 3)]);
                }
            }
            2 => {
                let graphics_index = usize::from(self.ram.sprite(slot).subtype2() >> 1 & 4)
                    + usize::from(self.ram.sprite(slot).scratch_d());
                self.ram
                    .set_sprite_graphics(slot, RECRUIT_GRAPHICS[graphics_index]);
                self.walking_zora_draw(slot);
                if self.runtime.sprite_return_if_inactive(self.ram, slot) {
                    return;
                }
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                self.runtime.sprite_move_z(self.ram, slot);
                self.ram.set_sprite_velocity_z(
                    slot,
                    self.ram.sprite(slot).velocity_z().wrapping_sub(2),
                );
                if (self.ram.sprite(slot).z().wrapping_sub(1) as i8) < 0 {
                    if (self.ram.sprite(slot).velocity_z().wrapping_add(16) as i8) < 0 {
                        self.ram.set_sprite_velocity_x(slot, 0);
                        self.ram.set_sprite_velocity_y(slot, 0);
                    }
                    self.ram.set_sprite_z(slot, 0);
                    self.ram.set_sprite_velocity_z(slot, 0);
                    if ((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 15 == 0 {
                        let direction = self
                            .runtime
                            .sprite_direction_to_face_link_direction(self.ram, slot);
                        self.ram.set_sprite_head_direction(slot, direction);
                        if ((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 31 == 0 {
                            self.ram.set_sprite_scratch_d(slot, direction);
                            self.runtime
                                .sprite_apply_speed_towards_link(self.ram, slot, 8);
                        }
                    }
                }
                self.runtime.sprite_move_xy(self.ram, slot);
                let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
                if (self.ram.sprite(slot).z().wrapping_sub(1) as i8) < 0 {
                    self.walking_zora_adjust_shadow(slot);
                    if self.ram.read_u8(SPRITE_TILETYPE) == 8 {
                        self.runtime.sprite_kill_self(self.ram, slot);
                        self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x28);
                        self.ram.set_sprite_state(slot, 3);
                        self.ram.set_sprite_main_delay(slot, 15);
                        self.ram.set_sprite_ai_state(slot, 0);
                        self.ram.set_sprite_flags2(slot, 3);
                    }
                }
                self.ram
                    .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
            }
            3 => {
                self.runtime.sprite_check_damage_from_link(self.ram, slot);
                if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 {
                    let timer = self.ram.sprite(slot).scratch_g().wrapping_sub(1);
                    self.ram.set_sprite_scratch_g(slot, timer);
                    if timer == 0 {
                        self.ram.set_sprite_scratch_b(slot, 2);
                        if self.ram.sprite(slot).state() == 10 {
                            self.ram.write_u8(LINK_STATE_BITS, 0);
                            self.ram.write_u8(LINK_PICKING_THROW_STATE, 0);
                        }
                        self.ram.set_sprite_state(slot, 9);
                    }
                }
                if self.ram.sprite(slot).scratch_g() < 48
                    && self.ram.read_u8(FRAME_COUNTER) & 1 == 0
                {
                    let offset: i16 = if self.ram.read_u8(FRAME_COUNTER) & 2 != 0 {
                        -1
                    } else {
                        1
                    };
                    self.set_sprite_x(slot, self.sprite_x(slot).wrapping_add(offset as u16));
                }
                self.ram.set_sprite_graphics(slot, 0);
                self.ram.set_sprite_wall_collision(slot, 0);
                self.walking_zora_draw_water_ripples(slot);
                self.ram
                    .set_sprite_flags2(slot, self.ram.sprite(slot).flags2().wrapping_sub(2));
                self.runtime.draw_single_large(self.ram, slot);
                self.ram
                    .set_sprite_flags2(slot, self.ram.sprite(slot).flags2().wrapping_add(2));
                self.ram.set_sprite_animation_clock(slot, 0);
                if self.runtime.sprite_return_if_inactive(self.ram, slot)
                    || self.runtime.sprite_return_if_recoiling(self.ram, slot)
                {
                    return;
                }
                self.runtime.sprite_move_xy(self.ram, slot);
                self.runtime
                    .thrown_sprite_tile_and_sprite_interaction(self.ram, slot);
                self.walking_zora_adjust_shadow(slot);
            }
            _ => {}
        }
    }

    pub fn walking_zora_adjust_shadow(&mut self, slot: usize) {
        self.ram.set_sprite_animation_clock(
            slot,
            u8::from(self.ram.sprite(slot).z() == 0 && self.ram.read_u8(SPRITE_TILETYPE) == 9),
        );
    }

    pub fn walking_zora_draw(&mut self, slot: usize) {
        const CHARACTER: [u8; 4] = [0xce, 0xce, 0xa4, 0xee];
        const FLAGS: [u8; 4] = [0x40, 0, 0, 0];
        const CHARACTER_2: [u8; 8] = [0xcc, 0xec, 0xcc, 0xec, 0xe8, 0xe8, 0xca, 0xca];
        const FLAGS_2: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0x40, 0, 0x40];
        self.walking_zora_draw_water_ripples(slot);
        let Some(mut info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        if graphics == 0 || graphics == 2 {
            info.y = info.y.wrapping_sub(1);
        }
        let direction = usize::from(self.ram.sprite(slot).head_direction());
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        crate::oam::set_helper0(
            self.ram,
            first_oam,
            info.x,
            info.y.wrapping_sub(6),
            CHARACTER[direction],
            info.flags | FLAGS[direction],
            2,
        );
        crate::oam::set_helper0(
            self.ram,
            first_oam + 1,
            info.x,
            info.y.wrapping_add(2),
            CHARACTER_2[graphics],
            info.flags | FLAGS_2[graphics],
            2,
        );
        if self.ram.sprite(slot).animation_clock() == 0 {
            self.runtime.sprite_draw_shadow(self.ram, slot, &info);
        }
    }

    pub fn walking_zora_draw_water_ripples(&mut self, slot: usize) {
        if self.ram.sprite(slot).animation_clock() != 0 {
            self.sprite_draw_water_ripple_with_oam_adjust(slot);
        }
    }

    pub fn sprite_draw_water_ripple_with_oam_adjust(&mut self, slot: usize) {
        self.sprite_draw_water_ripple(slot);
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(8));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(2),
        );
    }

    pub fn sprite_draw_water_ripple(&mut self, slot: usize) {
        const DATA: [DrawMultipleData; 6] = [
            DrawMultipleData {
                x: 0,
                y: 10,
                char_flags: 0x01d8,
                ext: 0,
            },
            DrawMultipleData {
                x: 8,
                y: 10,
                char_flags: 0x41d8,
                ext: 0,
            },
            DrawMultipleData {
                x: 0,
                y: 10,
                char_flags: 0x01d9,
                ext: 0,
            },
            DrawMultipleData {
                x: 8,
                y: 10,
                char_flags: 0x41d9,
                ext: 0,
            },
            DrawMultipleData {
                x: 0,
                y: 10,
                char_flags: 0x01da,
                ext: 0,
            },
            DrawMultipleData {
                x: 8,
                y: 10,
                char_flags: 0x41da,
                ext: 0,
            },
        ];
        const INDEX: [usize; 4] = [0, 1, 2, 1];
        let frame = INDEX[usize::from(self.ram.read_u8(FRAME_COUNTER) >> 2 & 3)] * 2;
        self.runtime
            .draw_multiple(self.ram, slot, &DATA[frame..frame + 2]);
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let flags = (crate::oam::read(self.ram, first_oam).flags & 0x30) | 4;
        let mut first = crate::oam::read(self.ram, first_oam);
        first.flags = flags;
        crate::oam::write(self.ram, first_oam, first);
        let mut second = crate::oam::read(self.ram, first_oam + 1);
        second.flags = flags | 0x40;
        crate::oam::write(self.ram, first_oam + 1, second);
    }

    pub fn sprite_53_armos_knight(&mut self, slot: usize) {
        const GRAPHICS: [u8; 5] = [5, 4, 3, 2, 1];
        const X_VELOCITY: [i8; 2] = [16, -16];
        self.ram
            .set_sprite_object_priority(slot, self.ram.sprite(slot).object_priority() | 0x30);
        self.armos_knight_draw(slot);
        if self.runtime.sprite_return_if_paused(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).state() != 9 {
            let delay = self.ram.sprite(slot).main_delay();
            if delay != 0 {
                self.ram
                    .set_sprite_graphics(slot, GRAPHICS[usize::from(delay >> 3)]);
                return;
            }
            let remaining = self.ram.read_u8(BYTE_7E0FF8).wrapping_sub(1);
            self.ram.write_u8(BYTE_7E0FF8, remaining);
            if remaining == 1 {
                for other in (0..6).rev() {
                    self.ram.set_sprite_health(other, 48);
                    self.ram.set_sprite_velocity_x(other, 0);
                    self.ram.set_sprite_velocity_y(other, 0);
                    self.ram.set_sprite_velocity_z(other, 0);
                }
            }
            self.ram.set_sprite_state(slot, 0);
            if self.runtime.sprite_check_if_screen_is_clear(self.ram) {
                let (spawned, info) = self
                    .runtime
                    .sprite_spawn_dynamically(self.ram, slot, 0xea)
                    .expect("Armos reward spawn must succeed");
                self.set_spawned_coordinates(spawned, &info);
                self.ram.set_sprite_velocity_z(spawned, 32);
                self.ram.set_sprite_scratch_a(spawned, 1);
            }
            return;
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        self.runtime.sprite_move_z(self.ram, slot);
        self.ram
            .set_sprite_velocity_z(slot, self.ram.sprite(slot).velocity_z().wrapping_sub(4));
        if self.ram.sprite(slot).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_velocity_z(slot, 0);
            self.ram.set_sprite_z(slot, 0);
            if self.ram.read_u8(BYTE_7E0FF8) != 1 && self.ram.sprite(slot).scratch_a() != 0 {
                self.ram.set_sprite_velocity_z(slot, 48);
                self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x16);
            }
        }
        if self.ram.sprite(slot).scratch_f() != 0 {
            self.ram.set_sprite_velocity_x(slot, 0);
            self.ram.set_sprite_velocity_y(slot, 0);
            self.ram.set_sprite_ai_state(slot, 0);
            self.ram.set_sprite_scratch_g(slot, 0);
        }
        if self.runtime.sprite_return_if_recoiling(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).scratch_a() == 0 {
            let delay = self.ram.sprite(slot).main_delay();
            if delay == 0 {
                self.ram
                    .set_sprite_scratch_a(slot, self.ram.sprite(slot).scratch_a().wrapping_add(1));
                self.ram.set_sprite_flags2(
                    slot,
                    (self.ram.sprite(slot).flags2() & 0x7f).wrapping_sub(2),
                );
                self.ram
                    .set_sprite_deflection_bits(slot, self.ram.sprite(slot).deflection_bits() & !4);
                self.ram
                    .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() & !0x40);
            } else {
                if delay == 64 {
                    self.ram.write_u8(SOUND_EFFECT_1, 0x35);
                } else if delay < 64 {
                    let direction = usize::from(((delay >> 1) ^ slot as u8) & 1);
                    self.ram.set_sprite_signed_field(
                        slot,
                        SpriteField::VelocityX,
                        X_VELOCITY[direction],
                    );
                    self.runtime.sprite_move_x(self.ram, slot);
                    self.ram.set_sprite_velocity_x(slot, 0);
                }
                self.runtime.sprite_check_damage_from_link(self.ram, slot);
                if self
                    .runtime
                    .sprite_check_damage_to_link_same_layer(self.ram, slot)
                {
                    self.runtime.sprite_nullify_hookshot_drag(self.ram);
                    self.runtime.sprite_repel_dash(self.ram);
                }
            }
        } else if self.ram.read_u8(BYTE_7E0FF8) == 1 {
            self.sprite_armos_crusher(slot);
        } else {
            self.runtime
                .sprite_check_damage_to_and_from_link(self.ram, slot);
            if self.ram.sprite(slot).ai_state() == 0 {
                let target_x = u16::from(self.ram.read_u8(OVERLORD_X_HI + slot))
                    | (u16::from(self.ram.read_u8(OVERLORD_Y_HI + slot)) << 8);
                let target_y = u16::from(self.ram.read_u8(OVERLORD_GEN2 + slot))
                    | (u16::from(self.ram.read_u8(OVERLORD_FLOOR + slot)) << 8);
                let velocity = self
                    .runtime
                    .sprite_project_speed_towards_location(self.ram, slot, target_x, target_y, 16);
                self.ram.set_sprite_velocity_x(slot, velocity.x);
                self.ram.set_sprite_velocity_y(slot, velocity.y);
                self.ram.write_u16(CUR_SPRITE_X, self.sprite_x(slot));
                self.ram.write_u16(CUR_SPRITE_Y, self.sprite_y(slot));
                if target_x
                    .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                    .wrapping_add(2)
                    < 4
                    && target_y
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                        .wrapping_add(2)
                        < 4
                {
                    self.ram.set_sprite_ai_state(slot, 1);
                }
            } else {
                self.ram
                    .set_sprite_x_low(slot, self.ram.read_u8(OVERLORD_X_HI + slot));
                self.ram
                    .set_sprite_x_high(slot, self.ram.read_u8(OVERLORD_Y_HI + slot));
                self.ram
                    .set_sprite_y_low(slot, self.ram.read_u8(OVERLORD_GEN2 + slot));
                self.ram
                    .set_sprite_y_high(slot, self.ram.read_u8(OVERLORD_FLOOR + slot));
            }
        }
    }

    pub fn armos_knight_draw(&mut self, slot: usize) {
        const X: [i8; 24] = [
            -8, 8, -8, 8, -10, 10, -10, 10, -10, 10, -10, 10, -12, 12, -12, 12, -14, 14, -14, 14,
            -16, 24, -16, 24,
        ];
        const Y: [i8; 24] = [
            -8, -8, 8, 8, -10, -10, 10, 10, -10, -10, 10, 10, -12, -12, 12, 12, -14, -14, 14, 14,
            -16, -16, 24, 24,
        ];
        const CHARACTER: [u8; 24] = [
            0xc0, 0xc2, 0xe0, 0xe2, 0xc0, 0xc2, 0xe0, 0xe2, 0xc4, 0xc4, 0xc4, 0xc4, 0xc6, 0xc6,
            0xc6, 0xc6, 0xc8, 0xc8, 0xc8, 0xc8, 0xd8, 0xd8, 0xd8, 0xd8,
        ];
        const FLAGS: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80,
            0x40, 0, 0xc0, 0x80,
        ];
        const BIG: [u8; 24] = [
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0,
        ];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        if self.ram.sprite(slot).scratch_a() == 0 && self.ram.read_u8(SUBMODULE_INDEX) != 7 {
            self.runtime.oam_allocate_defer_to_player(self.ram, slot);
        }
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        for (entry, piece) in (0..4).rev().enumerate() {
            let index = graphics * 4 + piece;
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[index] as i16 as u16),
                info.y.wrapping_add(Y[index] as i16 as u16),
                CHARACTER[index],
                FLAGS[index] | info.flags,
                BIG[index],
            );
        }
        if graphics != 0 {
            return;
        }
        if self.ram.sprite(slot).scratch_a() != 0 {
            let sprite_index = 76 + slot * 2;
            self.ram.write_u16(
                OAM_CUR_PTR,
                (crate::oam::OAM_BUFFER + sprite_index * 4) as u16,
            );
            self.ram.write_u16(
                OAM_EXT_CUR_PTR,
                (crate::oam::BYTEWISE_EXTENDED_OAM + sprite_index) as u16,
            );
        }
        let first_oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let spread = u16::from(self.ram.sprite(slot).z().min(32) >> 3);
        let y = self
            .sprite_y(slot)
            .wrapping_sub(self.ram.read_u16(BG2_VOFS_COPY2));
        crate::oam::set_helper0(
            self.ram,
            first_oam + 4,
            info.x.wrapping_sub(8).wrapping_add(spread),
            y.wrapping_add(12),
            0xe4,
            0x25,
            2,
        );
        crate::oam::set_helper0(
            self.ram,
            first_oam + 5,
            info.x.wrapping_add(8).wrapping_sub(spread),
            y.wrapping_add(12),
            0xe4,
            0x65,
            2,
        );
    }

    fn sprite_armos_crusher(&mut self, slot: usize) {
        self.ram.set_sprite_oam_flags(slot, 7);
        let shake = self.ram.sprite(slot).aux4_delay();
        self.ram.write_u16(
            BG1_Y_OFFSET,
            if shake == 0 {
                0
            } else if shake & 1 != 0 {
                u16::MAX
            } else {
                1
            },
        );
        match self.ram.sprite(slot).scratch_g() {
            0 => {
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                if self.ram.sprite(slot).main_delay() | self.ram.sprite(slot).z() == 0 {
                    self.runtime
                        .sprite_apply_speed_towards_link(self.ram, slot, 32);
                    self.ram.set_sprite_velocity_z(slot, 32);
                    self.ram.set_sprite_scratch_g(slot, 1);
                    let link_x = self.ram.read_u16(LINK_X);
                    let link_y = self.ram.read_u16(LINK_Y);
                    self.ram.set_sprite_scratch_b(slot, link_x as u8);
                    self.ram.set_sprite_scratch_c(slot, (link_x >> 8) as u8);
                    self.ram.set_sprite_scratch_e(slot, link_y as u8);
                    self.ram
                        .set_sprite_head_direction(slot, (link_y >> 8) as u8);
                    self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x20);
                }
            }
            1 => {
                self.ram.set_sprite_velocity_z(
                    slot,
                    self.ram.sprite(slot).velocity_z().wrapping_add(3),
                );
                let collision = self.runtime.sprite_check_tile_collision(self.ram, slot) != 0;
                self.runtime.sprite_get_16_bit_coords(self.ram, slot);
                let target_x = u16::from(self.ram.sprite(slot).scratch_b())
                    | (u16::from(self.ram.sprite(slot).scratch_c()) << 8);
                let target_y = u16::from(self.ram.sprite(slot).scratch_e())
                    | (u16::from(self.ram.sprite(slot).head_direction()) << 8);
                if collision
                    || target_x
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                        .wrapping_add(16)
                        < 32
                        && target_y
                            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                            .wrapping_add(16)
                            < 32
                {
                    self.ram.set_sprite_scratch_g(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 16);
                    self.ram.set_sprite_velocity_x(slot, 0);
                    self.ram.set_sprite_velocity_y(slot, 0);
                }
            }
            2 => {
                self.ram.set_sprite_velocity_z(slot, 0);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_scratch_g(slot, 3);
                }
            }
            _ => {
                self.ram.set_sprite_velocity_z(slot, (-104_i8) as u8);
                if self.ram.sprite(slot).signed_field(SpriteField::Z) >= 0 {
                    self.ram.set_sprite_main_delay(slot, 32);
                    self.ram.set_sprite_aux4_delay(slot, 32);
                    self.ram.set_sprite_scratch_g(slot, 0);
                    self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x0c);
                }
            }
        }
    }

    fn lanmola_spawn_shrapnel(&mut self, slot: usize) {
        const X_VELOCITY: [i8; 8] = [-28, -28, 28, 28, -36, 0, 36, 0];
        const Y_VELOCITY: [i8; 8] = [28, -28, 28, -28, 0, 36, 0, -36];
        let mut counter = if (0..3)
            .map(|i| self.ram.sprite(i).state())
            .fold(0_u8, u8::wrapping_add)
            < 10
        {
            7
        } else {
            3
        };
        loop {
            self.ram.write_u8(0xfb5, counter);
            if let Some((spawned, info)) =
                self.runtime.sprite_spawn_dynamically(self.ram, slot, 0xc2)
            {
                self.set_spawned_coordinates(spawned, &info);
                self.ram
                    .set_sprite_x_low(spawned, info.r0_x.wrapping_add(4) as u8);
                self.ram
                    .set_sprite_y_low(spawned, info.r2_y.wrapping_add(4) as u8);
                self.ram.set_sprite_ignore_projectile(spawned, 1);
                self.ram.set_sprite_bump_damage(spawned, 1);
                self.ram.set_sprite_flags4(spawned, 1);
                self.ram.set_sprite_z(spawned, 0);
                self.ram.set_sprite_flags2(spawned, 0x20);
                self.ram.set_sprite_signed_field(
                    spawned,
                    SpriteField::VelocityX,
                    X_VELOCITY[usize::from(counter)],
                );
                self.ram.set_sprite_signed_field(
                    spawned,
                    SpriteField::VelocityY,
                    Y_VELOCITY[usize::from(counter)],
                );
                let graphics = crate::misc::get_random_number(self.ram) & 1;
                self.ram.set_sprite_graphics(spawned, graphics);
            }
            counter = counter.wrapping_sub(1);
            self.ram.write_u8(0xfb5, counter);
            if (counter as i8) < 0 {
                break;
            }
        }
    }

    pub fn sprite_54_lanmolas(&mut self, slot: usize) {
        const RANDOM_B: [u8; 8] = [0x58, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0x98];
        const RANDOM_C: [u8; 8] = [0x68, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xa8, 0x80];
        const Z_VELOCITY: [i8; 2] = [2, -2];
        self.runtime.prep_oam_coord(self.ram, slot);
        self.lanmola_draw(slot);
        if self.runtime.sprite_return_if_paused(self.ram, slot) {
            return;
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.ram.sprite(slot).main_delay() | self.ram.sprite(slot).pause() == 0 {
                    self.ram.set_sprite_main_delay(slot, 127);
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x35);
                }
            }
            1 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.lanmola_spawn_shrapnel(slot);
                    self.ram.write_u8(SOUND_EFFECT_AMBIENT, 0x13);
                    let random_b = usize::from(crate::misc::get_random_number(self.ram) & 7);
                    let random_c = usize::from(crate::misc::get_random_number(self.ram) & 7);
                    self.ram.set_sprite_scratch_b(slot, RANDOM_B[random_b]);
                    self.ram.set_sprite_scratch_c(slot, RANDOM_C[random_c]);
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_velocity_z(slot, 24);
                    self.ram.set_sprite_animation_clock(slot, 0);
                    self.ram.set_sprite_scratch_g(slot, 0);
                    self.lanmola_set_burrow_anchor(slot);
                }
            }
            2 => {
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                self.runtime.sprite_move_z(self.ram, slot);
                if self.ram.sprite(slot).animation_clock() == 0 {
                    let velocity = self.ram.sprite(slot).velocity_z().wrapping_sub(1);
                    self.ram.set_sprite_velocity_z(slot, velocity);
                    if velocity == 0 {
                        self.ram.set_sprite_animation_clock(
                            slot,
                            self.ram.sprite(slot).animation_clock().wrapping_add(1),
                        );
                    }
                } else if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
                    let phase = usize::from(self.ram.sprite(slot).scratch_g() & 1);
                    let velocity = self
                        .ram
                        .sprite(slot)
                        .velocity_z()
                        .wrapping_add(Z_VELOCITY[phase] as u8);
                    self.ram.set_sprite_velocity_z(slot, velocity);
                    if velocity == DESERT_BARRIER_X_VELOCITY[phase] as u8 {
                        self.ram.set_sprite_scratch_g(
                            slot,
                            self.ram.sprite(slot).scratch_g().wrapping_add(1),
                        );
                    }
                }
                let x = self.sprite_x(slot);
                let y = self.sprite_y(slot);
                let target_x = u16::from(self.ram.sprite(slot).scratch_b())
                    | (u16::from(self.ram.sprite(slot).x_high()) << 8);
                let target_y = u16::from(self.ram.sprite(slot).scratch_c())
                    | (u16::from(self.ram.sprite(slot).y_high()) << 8);
                if x.wrapping_sub(target_x).wrapping_add(2) < 4
                    && y.wrapping_sub(target_y).wrapping_add(2) < 4
                {
                    self.ram.set_sprite_ai_state(slot, 3);
                }
                let velocity = self
                    .runtime
                    .sprite_project_speed_towards_location(self.ram, slot, target_x, target_y, 10);
                self.ram.set_sprite_velocity_y(slot, velocity.y);
                self.ram.set_sprite_velocity_x(slot, velocity.x);
                self.runtime.sprite_move_xy(self.ram, slot);
            }
            3 => {
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                self.runtime.sprite_move_xy(self.ram, slot);
                self.runtime.sprite_move_z(self.ram, slot);
                let z_velocity = self.ram.sprite(slot).velocity_z();
                if (z_velocity.wrapping_add(20) as i8) >= 0 {
                    self.ram
                        .set_sprite_velocity_z(slot, z_velocity.wrapping_sub(1));
                }
                if self.ram.sprite(slot).signed_field(SpriteField::Z) < 0 {
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.ram.set_sprite_main_delay(slot, 128);
                    self.lanmola_set_burrow_anchor(slot);
                }
            }
            4 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 0);
                    let random_b = usize::from(crate::misc::get_random_number(self.ram) & 7);
                    let random_c = usize::from(crate::misc::get_random_number(self.ram) & 7);
                    self.ram.set_sprite_x_low(slot, RANDOM_B[random_b]);
                    self.ram.set_sprite_y_low(slot, RANDOM_C[random_c]);
                }
            }
            5 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_state(slot, 0);
                    if self.runtime.sprite_check_if_screen_is_clear(self.ram) {
                        let (spawned, info) = self
                            .runtime
                            .sprite_spawn_dynamically(self.ram, slot, 0xea)
                            .expect("Lanmola reward spawn must succeed");
                        self.set_spawned_coordinates(spawned, &info);
                        self.ram.set_sprite_velocity_z(spawned, 32);
                        self.ram.set_sprite_scratch_a(spawned, 3);
                    }
                }
                if (32..160).contains(&delay) && delay & 15 == 0 {
                    let history =
                        usize::from(
                            self.ram.sprite(slot).subtype2().wrapping_sub(
                                self.ram.read_u8(GARNISH_Y_LO + slot).wrapping_mul(8),
                            ) & 0x3f,
                        ) + slot * 0x40;
                    let x_low = self
                        .ram
                        .read_u8(MOLDORM_X_LO + history)
                        .wrapping_sub(self.ram.read_u8(BG2_HOFS_COPY2));
                    let y_low = self
                        .ram
                        .read_u8(MOLDORM_Y_LO + history)
                        .wrapping_sub(self.ram.read_u8(BEAMOS_X_HI + history))
                        .wrapping_sub(self.ram.read_u8(BG2_VOFS_COPY2));
                    if let Some((spawned, _)) =
                        self.runtime.sprite_spawn_dynamically(self.ram, slot, 0)
                    {
                        self.ram.write_u8(LOAD_CHR_HALFSLOT_EVEN_ODD, 11);
                        self.ram.set_sprite_state(spawned, 4);
                        self.ram.set_sprite_main_delay(spawned, 31);
                        self.ram.set_sprite_scratch_a(spawned, 31);
                        self.set_sprite_x(
                            spawned,
                            self.ram
                                .read_u16(BG2_HOFS_COPY2)
                                .wrapping_add(u16::from(x_low)),
                        );
                        self.set_sprite_y(
                            spawned,
                            self.ram
                                .read_u16(BG2_VOFS_COPY2)
                                .wrapping_add(u16::from(y_low)),
                        );
                        self.ram.set_sprite_flags2(spawned, 3);
                        self.ram.set_sprite_oam_flags(spawned, 0xc);
                        self.runtime.queue_sfx2_with_pan(self.ram, slot, 0xc);
                        let segments = self.ram.read_u8(GARNISH_Y_LO + slot);
                        if (segments as i8) >= 0 {
                            self.ram
                                .write_u8(GARNISH_Y_LO + slot, segments.wrapping_sub(1));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn lanmola_set_burrow_anchor(&mut self, slot: usize) {
        self.ram
            .set_sprite_scratch_d(slot, self.ram.sprite(slot).x_low());
        self.ram
            .set_sprite_wall_collision(slot, self.ram.sprite(slot).y_low());
        self.ram.set_sprite_aux1_delay(slot, 74);
    }

    pub fn lanmola_draw(&mut self, slot: usize) {
        const SPRITE_OFFSET: [usize; 4] = [76, 60, 44, 28];
        const CHARACTER_1: [u8; 16] = [
            0xc4, 0xe2, 0xc2, 0xe0, 0xc0, 0xe0, 0xc2, 0xe2, 0xc4, 0xe2, 0xc2, 0xe0, 0xc0, 0xe0,
            0xc2, 0xe2,
        ];
        const CHARACTER_0: [u8; 16] = [
            0xcc, 0xe4, 0xca, 0xe6, 0xc8, 0xe6, 0xca, 0xe4, 0xcc, 0xe4, 0xca, 0xe6, 0xc8, 0xe6,
            0xca, 0xe4,
        ];
        const FLAGS: [u8; 16] = [
            0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40,
        ];
        let sprite_offset = SPRITE_OFFSET[slot];
        self.ram.write_u16(
            OAM_CUR_PTR,
            (crate::oam::OAM_BUFFER + sprite_offset * 4) as u16,
        );
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            (crate::oam::BYTEWISE_EXTENDED_OAM + sprite_offset) as u16,
        );
        let angle = self.runtime.sprite_convert_velocity_to_angle(
            self.ram.sprite(slot).velocity_x(),
            self.ram
                .sprite(slot)
                .velocity_y()
                .wrapping_sub(self.ram.sprite(slot).velocity_z()),
        );
        self.ram.set_sprite_graphics(slot, angle);
        let mut history_head = self.ram.sprite(slot).subtype2();
        let mut shadow_head = history_head;
        let history = slot * 64 + usize::from(history_head);
        self.ram
            .write_u8(MOLDORM_X_LO + history, self.ram.sprite(slot).x_low());
        self.ram
            .write_u8(MOLDORM_Y_LO + history, self.ram.sprite(slot).y_low());
        self.ram
            .write_u8(BEAMOS_X_HI + history, self.ram.sprite(slot).z());
        self.ram.write_u8(BEAMOS_Y_HI + history, angle);
        if self.ram.sprite(slot).state() == 9
            && self.ram.read_u8(SUBMODULE_INDEX) | self.ram.read_u8(FLAG_UNK1) == 0
        {
            self.ram
                .set_sprite_subtype2(slot, history_head.wrapping_add(1) & 63);
        }
        let sprite_flags =
            self.ram.sprite(slot).oam_flags() | self.ram.sprite(slot).object_priority();
        let segment_count = self.ram.read_u8(GARNISH_Y_LO + slot);
        if (segment_count as i8) < 0 {
            return;
        }
        let reverse = self.ram.sprite(slot).signed_field(SpriteField::VelocityY) < 0;
        let mut oam_index = sprite_offset as isize + if reverse { 7 } else { 0 };
        for segment in (0..=usize::from(segment_count)).rev() {
            let history = slot * 64 + usize::from(history_head);
            history_head = history_head.wrapping_sub(8) & 63;
            let index = oam_index as usize;
            let mut oam = crate::oam::read(self.ram, index);
            oam.x = self
                .ram
                .read_u8(MOLDORM_X_LO + history)
                .wrapping_sub(self.ram.read_u8(BG2_HOFS_COPY2));
            if self.ram.read_i8(BEAMOS_X_HI + history) >= 0 {
                oam.y = self
                    .ram
                    .read_u8(MOLDORM_Y_LO + history)
                    .wrapping_sub(self.ram.read_u8(BEAMOS_X_HI + history))
                    .wrapping_sub(self.ram.read_u8(BG2_VOFS_COPY2));
            }
            let angle = usize::from(self.ram.read_u8(BEAMOS_Y_HI + history));
            oam.charnum = if segment_count != 7 || segment != 0 {
                if usize::from(segment_count) == segment {
                    CHARACTER_1[angle]
                } else {
                    0xc6
                }
            } else {
                CHARACTER_0[angle]
            };
            oam.flags = FLAGS[angle] | sprite_flags;
            crate::oam::write(self.ram, index, oam);
            self.ram
                .write_u8(crate::oam::BYTEWISE_EXTENDED_OAM + index, 2);
            oam_index += if reverse { -1 } else { 1 };
        }
        for entry in 0..=usize::from(segment_count) {
            let history = slot * 64 + usize::from(shadow_head);
            shadow_head = shadow_head.wrapping_sub(8) & 63;
            let index = sprite_offset + 8 + entry;
            let mut oam = crate::oam::read(self.ram, index);
            oam.x = self
                .ram
                .read_u8(MOLDORM_X_LO + history)
                .wrapping_sub(self.ram.read_u8(BG2_HOFS_COPY2));
            if self.ram.read_i8(BEAMOS_X_HI + history) >= 0 {
                oam.y = self
                    .ram
                    .read_u8(MOLDORM_Y_LO + history)
                    .wrapping_add(10)
                    .wrapping_sub(self.ram.read_u8(BG2_VOFS_COPY2));
            }
            oam.charnum = 0x6c;
            oam.flags = 0x34;
            crate::oam::write(self.ram, index, oam);
            self.ram
                .write_u8(crate::oam::BYTEWISE_EXTENDED_OAM + index, 2);
        }
        let state = self.ram.sprite(slot).ai_state();
        if state == 1 {
            const INDEX: [usize; 16] = [4, 5, 4, 5, 4, 5, 4, 5, 4, 3, 2, 2, 1, 1, 0, 0];
            const CHARACTER: [u8; 6] = [0xee, 0xee, 0xec, 0xec, 0xce, 0xce];
            const FLAGS: [u8; 6] = [0, 0x40, 0, 0x40, 0, 0x40];
            self.runtime.oam_allocate_from_region_b(self.ram, 4);
            let oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
            let frame = INDEX[usize::from(self.ram.sprite(slot).main_delay() >> 3)];
            crate::oam::set_plain(
                self.ram,
                oam,
                self.ram
                    .sprite(slot)
                    .x_low()
                    .wrapping_sub(self.ram.read_u8(BG2_HOFS_COPY2)),
                self.ram
                    .sprite(slot)
                    .y_low()
                    .wrapping_sub(self.ram.read_u8(BG2_VOFS_COPY2)),
                CHARACTER[frame],
                FLAGS[frame] | 0x31,
                2,
            );
        } else if state != 5 && self.ram.sprite(slot).aux1_delay() != 0 {
            const X: [i8; 8] = [-8, 8, -10, 10, -16, 16, -24, 32];
            const Y: [i8; 8] = [0, 0, -1, -1, -1, -1, 3, 3];
            const CHARACTER: [u8; 8] = [0xe8, 0xe8, 0xe8, 0xe8, 0xea, 0xea, 0xea, 0xea];
            const FLAGS: [u8; 8] = [0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40];
            const BIG: [u8; 8] = [2, 2, 2, 2, 2, 2, 0, 0];
            if ((self.ram.sprite(slot).velocity_y() >> 6) ^ state) & 2 != 0 {
                self.runtime.oam_allocate_from_region_b(self.ram, 8);
            } else {
                self.runtime.oam_allocate_from_region_c(self.ram, 8);
            }
            let first_oam =
                (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
            let base = usize::from(((self.ram.sprite(slot).aux1_delay() >> 2 & 3) ^ 3) * 2);
            let x = self
                .ram
                .sprite(slot)
                .scratch_d()
                .wrapping_sub(self.ram.read_u8(BG2_HOFS_COPY2));
            let y = self
                .ram
                .sprite(slot)
                .wall_collision()
                .wrapping_sub(self.ram.read_u8(BG2_VOFS_COPY2));
            for (entry, piece) in (0..2).rev().enumerate() {
                let index = base + piece;
                crate::oam::set_plain(
                    self.ram,
                    first_oam + entry,
                    x.wrapping_add(X[index] as u8),
                    y.wrapping_add(Y[index] as u8),
                    CHARACTER[index],
                    FLAGS[index] | 0x31,
                    BIG[index],
                );
            }
        }
    }

    pub fn sprite_6d_rat(&mut self, slot: usize) {
        const GRAPHICS: [u8; 16] = [0, 0, 3, 3, 1, 2, 4, 5, 1, 2, 4, 5, 0, 0, 3, 3];
        const FLAGS: [u8; 16] = [
            0, 0x40, 0, 0x40, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x80, 0xc0, 0x80, 0xc0,
        ];
        const IDLE: [u8; 8] = [10, 11, 6, 7, 2, 3, 14, 15];
        const WALK: [u8; 8] = [8, 9, 4, 5, 0, 1, 12, 13];
        const X_VELOCITY: [i8; 4] = [24, -24, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, 24, -24];
        const BOUNCE: [u8; 4] = [2, 3, 1, 0];
        let animation = usize::from(self.ram.sprite(slot).scratch_a());
        self.ram.set_sprite_graphics(slot, GRAPHICS[animation]);
        self.ram.set_sprite_oam_flags(
            slot,
            (self.ram.sprite(slot).oam_flags() & 0x3f) | FLAGS[animation],
        );
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || self.runtime.sprite_return_if_recoiling(self.ram, slot)
        {
            return;
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        self.runtime.sprite_move_xy(self.ram, slot);
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        if self.ram.sprite(slot).ai_state() != 0 {
            if self.ram.sprite(slot).main_delay() == 0 {
                if self.ram.read_u8(IS_IN_DARK_WORLD) == 0 {
                    self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x17);
                }
                self.ram.set_sprite_ai_state(slot, 0);
                self.ram.set_sprite_main_delay(slot, 80);
            }
            let mut direction = usize::from(self.ram.sprite(slot).scratch_d());
            if self.ram.sprite(slot).wall_collision() != 0 {
                direction = usize::from(BOUNCE[direction]);
                self.ram.set_sprite_scratch_d(slot, direction as u8);
            }
            self.ram
                .set_sprite_signed_field(slot, SpriteField::VelocityX, X_VELOCITY[direction]);
            self.ram
                .set_sprite_signed_field(slot, SpriteField::VelocityY, Y_VELOCITY[direction]);
            let frame = usize::from(self.ram.read_u8(FRAME_COUNTER) >> 2 & 1);
            self.ram
                .set_sprite_scratch_a(slot, WALK[direction * 2 + frame]);
        } else {
            self.ram.set_sprite_velocity_x(slot, 0);
            self.ram.set_sprite_velocity_y(slot, 0);
            if self.ram.sprite(slot).main_delay() == 0 {
                let random = crate::misc::get_random_number(self.ram);
                self.ram.set_sprite_scratch_d(slot, random & 3);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                self.ram
                    .set_sprite_main_delay(slot, (random & 0x7f).wrapping_add(0x40));
            }
            let direction = usize::from(self.ram.sprite(slot).scratch_d());
            let frame = usize::from(self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
            self.ram
                .set_sprite_scratch_a(slot, IDLE[direction * 2 + frame]);
        }
    }

    pub fn sprite_6e_rope(&mut self, slot: usize) {
        const GRAPHICS: [i8; 8] = [0, 0, 2, 3, 2, 3, 1, 1];
        const FLAGS: [u8; 8] = [0, 0x40, 0, 0, 0x40, 0x40, 0, 0x40];
        const ANIMATION: [u8; 8] = [4, 5, 2, 3, 0, 1, 6, 7];
        const X_VELOCITY: [i8; 8] = [8, -8, 0, 0, 16, -16, 0, 0];
        const Y_VELOCITY: [i8; 8] = [0, 0, 8, -8, 0, 0, 16, -16];
        const BOUNCE: [u8; 4] = [2, 3, 1, 0];
        let animation = usize::from(self.ram.sprite(slot).scratch_a());
        self.ram
            .set_sprite_graphics(slot, GRAPHICS[animation] as u8);
        self.ram.set_sprite_oam_flags(
            slot,
            (self.ram.sprite(slot).oam_flags() & 0x3f) | FLAGS[animation],
        );
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).scratch_e() != 0 {
            let oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
            let mut entry = crate::oam::read(self.ram, oam);
            entry.flags |= 0x30;
            crate::oam::write(self.ram, oam, entry);
            let old_z = self.ram.sprite(slot).z();
            self.runtime.sprite_move_z(self.ram, slot);
            let z_velocity = self.ram.sprite(slot).velocity_z();
            if (z_velocity.wrapping_sub(0xc0) as i8) >= 0 {
                self.ram
                    .set_sprite_velocity_z(slot, z_velocity.wrapping_sub(2));
            }
            let z = self.ram.sprite(slot).z();
            if (old_z ^ z) as i8 >= 0 || (z as i8) >= 0 {
                return;
            }
            self.ram.set_sprite_z(slot, 0);
            self.ram.set_sprite_velocity_z(slot, 0);
            self.ram.set_sprite_scratch_e(slot, 0);
            self.ram
                .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() & !0x10);
            return;
        }
        self.ram.set_sprite_flags2(slot, 0);
        if self.runtime.sprite_return_if_recoiling(self.ram, slot) {
            return;
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        self.runtime.sprite_move_xy(self.ram, slot);
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        if self.ram.sprite(slot).ai_state() != 0 {
            if self.ram.sprite(slot).main_delay() == 0 {
                self.ram.set_sprite_ai_state(slot, 0);
                self.ram.set_sprite_main_delay(slot, 32);
            }
            let mut direction = usize::from(self.ram.sprite(slot).scratch_d());
            if self.ram.sprite(slot).wall_collision() != 0 {
                direction = usize::from(BOUNCE[direction]);
                self.ram.set_sprite_scratch_d(slot, direction as u8);
            }
            let velocity_index = direction + usize::from(self.ram.sprite(slot).scratch_g());
            self.ram.set_sprite_signed_field(
                slot,
                SpriteField::VelocityX,
                X_VELOCITY[velocity_index],
            );
            self.ram.set_sprite_signed_field(
                slot,
                SpriteField::VelocityY,
                Y_VELOCITY[velocity_index],
            );
            let mut frame = self.ram.read_u8(FRAME_COUNTER);
            if velocity_index < 4 {
                frame >>= 1;
            }
            let direction = usize::from(self.ram.sprite(slot).scratch_d());
            self.ram
                .set_sprite_scratch_a(slot, ANIMATION[direction * 2 + usize::from(frame >> 1 & 1)]);
        } else {
            self.ram.set_sprite_velocity_x(slot, 0);
            self.ram.set_sprite_velocity_y(slot, 0);
            if self.ram.sprite(slot).main_delay() == 0 {
                self.ram.set_sprite_scratch_g(slot, 0);
                let random = crate::misc::get_random_number(self.ram);
                self.ram.set_sprite_scratch_d(slot, random & 3);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                self.ram
                    .set_sprite_main_delay(slot, (random & 0x7f).wrapping_add(0x40));
                let (direction, x_difference, y_difference) =
                    self.runtime.sprite_direction_and_deltas(self.ram, slot);
                if y_difference.wrapping_add(0x10) < 0x20 || x_difference.wrapping_add(0x18) < 0x20
                {
                    self.ram.set_sprite_scratch_g(slot, 4);
                    self.ram.set_sprite_scratch_d(slot, direction);
                }
            }
            let direction = usize::from(self.ram.sprite(slot).scratch_d());
            self.ram.set_sprite_scratch_a(
                slot,
                ANIMATION[direction * 2 + usize::from(self.ram.read_u8(FRAME_COUNTER) >> 3 & 1)],
            );
        }
    }

    pub fn sprite_6f_keese(&mut self, slot: usize) {
        const STEP: [i8; 2] = [1, -1];
        const START_ANGLE: [i8; 4] = [2, 10, 6, 14];
        self.ram
            .set_sprite_object_priority(slot, self.ram.sprite(slot).object_priority() | 0x30);
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || self.runtime.sprite_return_if_recoiling(self.ram, slot)
        {
            return;
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        self.runtime.sprite_move_xy(self.ram, slot);
        if self.ram.sprite(slot).ai_state() != 0 {
            let delay = self.ram.sprite(slot).main_delay();
            if delay == 0 {
                self.ram.set_sprite_ai_state(slot, 0);
                self.ram.set_sprite_main_delay(slot, 64);
                self.ram.set_sprite_graphics(slot, 0);
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
            } else {
                if delay & 7 == 0 {
                    let phase = usize::from(self.ram.sprite(slot).scratch_b() & 1);
                    self.ram.set_sprite_scratch_a(
                        slot,
                        self.ram
                            .sprite(slot)
                            .scratch_a()
                            .wrapping_add(STEP[phase] as u8),
                    );
                    if crate::misc::get_random_number(self.ram) & 3 == 0 {
                        self.ram.set_sprite_scratch_b(
                            slot,
                            self.ram.sprite(slot).scratch_b().wrapping_add(1),
                        );
                    }
                }
                let angle = usize::from(self.ram.sprite(slot).scratch_a() & 0xf);
                self.ram.set_sprite_signed_field(
                    slot,
                    SpriteField::VelocityX,
                    KEESE_X_VELOCITY[angle],
                );
                self.ram.set_sprite_signed_field(
                    slot,
                    SpriteField::VelocityY,
                    KEESE_Y_VELOCITY[angle],
                );
                self.ram
                    .set_sprite_graphics(slot, (self.ram.read_u8(FRAME_COUNTER) >> 2 & 1) + 1);
            }
        } else {
            if (((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3)
                | self.ram.sprite(slot).main_delay()
                != 0
            {
                return;
            }
            let (direction, x_difference, y_difference) =
                self.runtime.sprite_direction_and_deltas(self.ram, slot);
            if y_difference.wrapping_add(0x28) >= 0x50 || x_difference.wrapping_add(0x28) >= 0x50 {
                return;
            }
            self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x1e);
            self.ram
                .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
            self.ram.set_sprite_main_delay(slot, 64);
            self.ram.set_sprite_scratch_b(slot, 64);
            self.ram
                .set_sprite_scratch_a(slot, START_ANGLE[usize::from(direction)] as u8);
        }
    }

    pub fn sprite_cannonball(&mut self, slot: usize) {
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        let delay = self.ram.sprite(slot).main_delay();
        if delay == 30 {
            self.sprite_spawn_poof_garnish(slot);
        } else if delay == 0 && self.runtime.sprite_check_tile_collision(self.ram, slot) != 0 {
            self.ram.set_sprite_state(slot, 0);
            self.ram
                .set_sprite_x_low(slot, self.ram.sprite(slot).x_low().wrapping_add(4));
            self.ram
                .set_sprite_y_low(slot, self.ram.sprite(slot).y_low().wrapping_add(4));
            self.runtime.sprite_place_repulse_spark_2(self.ram, slot);
            self.runtime.queue_sfx2_with_pan(self.ram, slot, 5);
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
    }

    pub fn sprite_spawn_poof_garnish(&mut self, sprite: usize) {
        let garnish = self.garnish_alloc_force();
        self.ram.write_u8(GARNISH_TYPE + garnish, 10);
        self.ram.write_u8(GARNISH_ACTIVE, 10);
        self.ram
            .write_u8(GARNISH_X_LO + garnish, self.ram.sprite(sprite).x_low());
        self.ram
            .write_u8(GARNISH_X_HI + garnish, self.ram.sprite(sprite).x_high());
        self.garnish_set_y(garnish, self.sprite_y(sprite).wrapping_add(16));
        self.ram
            .write_u8(GARNISH_SPRITE + garnish, self.ram.sprite(sprite).floor());
        self.ram.write_u8(GARNISH_COUNTDOWN + garnish, 15);
    }

    pub fn sprite_6c_mirror_portal(&mut self, slot: usize) {
        const FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        if self.ram.read_u8(SAVEGAME_IS_DARKWORLD) != 0 {
            self.ram.set_sprite_state(slot, 0);
        } else {
            if self.ram.read_u8(OVERWORLD_SCREEN_INDEX) >= 0x80 {
                return;
            }
            if self.ram.read_u8(SUBMODULE_INDEX) != 0x23 && self.ram.read_u8(BYTE_7E0FC6) < 3 {
                self.runtime.draw_single_large(self.ram, slot);
            }
            if self.runtime.sprite_return_if_inactive(self.ram, slot) {
                return;
            }
            let frame = usize::from(self.ram.read_u8(FRAME_COUNTER) >> 2 & 3);
            self.ram.set_sprite_oam_flags(
                slot,
                (self.ram.sprite(slot).oam_flags() & 0x3f) | FLAGS[frame],
            );
            if self.runtime.sprite_check_if_link_is_busy(self.ram) {
                return;
            }
            if self
                .runtime
                .sprite_check_damage_to_link_same_layer(self.ram, slot)
            {
                if self.ram.sprite(slot).scratch_a() != 0
                    && self.ram.read_u8(LINK_DISABLE_SPRITE_DAMAGE)
                        | self.ram.read_u8(COUNTDOWN_FOR_BLINK)
                        == 0
                    && self.ram.read_u8(FLAG_IS_LINK_IMMOBILIZED) == 0
                {
                    self.ram.write_u8(SUBMODULE_INDEX, 0x23);
                    self.ram.write_u8(LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE, 1);
                    self.ram.write_u8(SUBSUBMODULE_INDEX, 0);
                    self.ram.write_u8(LINK_ACTUAL_VEL_Y, 0);
                    self.ram.write_u8(LINK_ACTUAL_VEL_X, 0);
                    self.ram.write_u8(LINK_PLAYER_HANDLER_STATE, 20);
                    self.ram.write_u8(
                        LAST_LIGHT_VS_DARK_WORLD,
                        self.ram.read_u8(OVERWORLD_SCREEN_INDEX) & 0x40,
                    );
                    self.ram.set_sprite_state(slot, 0);
                }
            } else {
                self.ram.set_sprite_scratch_a(slot, 1);
            }
        }
        let timer = self.ram.sprite(slot).scratch_b().wrapping_add(1);
        self.ram.set_sprite_scratch_b(slot, timer);
        if timer == 0 {
            self.ram.set_sprite_scratch_a(slot, 1);
        }
        self.ram
            .set_sprite_x_low(slot, self.ram.read_u8(BIRD_TRAVEL_X_LO + 15));
        self.ram
            .set_sprite_x_high(slot, self.ram.read_u8(BIRD_TRAVEL_X_HI + 15));
        let y = u16::from(self.ram.read_u8(BIRD_TRAVEL_Y_LO + 15))
            | (u16::from(self.ram.read_u8(BIRD_TRAVEL_Y_HI + 15)) << 8);
        self.set_sprite_y(slot, y.wrapping_add(8));
    }

    pub fn sprite_6a_ball_n_chain(&mut self, slot: usize) {
        const ANGLE_STEP: [u8; 4] = [0x0d, 0x60, 0x22, 0x10];
        const ATTACK_DIRECTION: [u8; 4] = [3, 1, 2, 0];
        self.chain_ball_trooper_draw(slot);
        let state = usize::from(self.ram.sprite(slot).ai_state());
        if state < 2 {
            self.ram.write_u8(DUNGMAP_VAR8 + 1, 0x80);
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.guard_parry_sword_attacks(self.ram, slot);
        let angle = (u16::from(self.ram.sprite(slot).scratch_b()) << 8)
            | u16::from(self.ram.sprite(slot).scratch_a());
        let angle = angle.wrapping_add(u16::from(ANGLE_STEP[state]));
        self.ram.set_sprite_scratch_a(slot, angle as u8);
        self.ram
            .set_sprite_scratch_b(slot, ((angle >> 8) as u8) & 1);
        if self.runtime.sprite_return_if_recoiling(self.ram, slot) {
            return;
        }
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        self.runtime.sprite_move_xy(self.ram, slot);
        let _ = self.runtime.sprite_check_damage_to_link(self.ram, slot);

        let frame_matches = (((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 0xf) == 0;
        let mut x_difference = 0;
        let mut y_difference = 0;
        if frame_matches {
            let (direction, x, y) = self.runtime.sprite_direction_and_deltas(self.ram, slot);
            self.ram.set_sprite_head_direction(slot, direction);
            x_difference = x;
            y_difference = y;
        }
        match state {
            0 => {
                if frame_matches {
                    let direction = self.ram.sprite(slot).head_direction();
                    self.ram.set_sprite_scratch_d(slot, direction);
                    if y_difference.wrapping_add(0x40) < 0x68
                        && x_difference.wrapping_add(0x30) < 0x60
                    {
                        self.ram.set_sprite_ai_state(slot, 1);
                        self.ram.set_sprite_main_delay(slot, 24);
                        return;
                    }
                    self.runtime
                        .sprite_apply_speed_towards_link(self.ram, slot, 8);
                }
                self.ball_n_chain_animate(slot);
            }
            1 => {
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_main_delay(slot, 48);
                    self.ram.set_sprite_ai_state(slot, 2);
                }
            }
            2 => {
                let attack_index = usize::from(
                    (self.ram.sprite(slot).scratch_a() >> 7 & 1)
                        + self.ram.sprite(slot).scratch_b() * 2,
                );
                if self.ram.sprite(slot).main_delay() == 0
                    && self.ram.sprite(slot).head_direction() == ATTACK_DIRECTION[attack_index]
                {
                    self.ram.set_sprite_ai_state(slot, 3);
                    self.ram.set_sprite_aux2_delay(slot, 31);
                }
                self.ball_n_chain_attack_common(slot, frame_matches);
            }
            3 => {
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                let delay = self.ram.sprite(slot).aux2_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 0);
                } else if delay >= 0x10 {
                    self.ball_n_chain_attack_common(slot, frame_matches);
                    return;
                }
                self.ram
                    .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
                self.ball_n_chain_animate(slot);
            }
            _ => unreachable!("ball-and-chain guard has only four AI states"),
        }
    }

    fn ball_n_chain_attack_common(&mut self, slot: usize, frame_matches: bool) {
        self.ram
            .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
        self.ball_n_chain_animate(slot);
        if frame_matches {
            self.runtime.queue_sfx3_with_pan(self.ram, slot, 6);
        }
    }

    pub fn ball_n_chain_animate(&mut self, slot: usize) {
        const GRAPHICS: [u8; 32] = [
            0x10, 0x11, 0x12, 0x13, 0x10, 0x11, 0x12, 0x13, 6, 7, 8, 9, 6, 7, 8, 9, 0, 1, 2, 3, 0,
            1, 4, 5, 0xa, 0xb, 0xc, 0xd, 0xa, 0xb, 0xe, 0xf,
        ];
        let phase = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, phase);
        let index =
            usize::from(self.ram.sprite(slot).scratch_d()) * 8 + usize::from(phase >> 2 & 7);
        self.ram.set_sprite_graphics(slot, GRAPHICS[index]);
    }

    pub fn sprite_draw_guard_head(
        &mut self,
        slot: usize,
        info: &PrepOamCoordsRet,
        sprite_offset: usize,
    ) {
        const CHARACTER: [u8; 4] = [2, 2, 0, 4];
        const FLAGS: [u8; 4] = [0x40, 0, 0, 0];
        let direction = usize::from(self.ram.sprite(slot).head_direction());
        let oam = self.oam_current_index() + sprite_offset;
        crate::oam::set_helper0(
            self.ram,
            oam,
            info.x,
            info.y.wrapping_sub(9),
            CHARACTER[direction],
            info.flags | FLAGS[direction],
            2,
        );
    }

    pub fn sprite_draw_bnc_body(
        &mut self,
        slot: usize,
        info: &PrepOamCoordsRet,
        mut sprite_offset: usize,
    ) {
        const X: [i8; 72] = [
            -4, 4, 12, -4, 4, 13, -4, 4, 13, -4, 4, 13, -4, 4, 13, -4, 4, 13, 0, 0, 4, 0, 0, 5, 0,
            0, 6, 0, 0, 4, -4, 4, -6, -4, 4, -5, -4, 4, -5, -4, 4, -6, -4, 4, -5, -4, 4, -6, 0, 0,
            4, 0, 0, 3, 0, 0, 2, 0, 0, 4, 0, 0, 0, 0, 0, 0, -4, 4, 4, -4, 4, 4,
        ];
        const Y: [i8; 72] = [
            0, 0, -4, 0, 0, -4, 0, 0, -3, 0, 0, -2, 0, 0, -3, 0, 0, -2, 0, 0, 1, 0, 0, 1, 0, 0, 2,
            0, 0, 2, 0, 0, -2, 0, 0, -2, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, 1, 0, 0, 1,
            0, 0, 2, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const CHARACTER: [u8; 72] = [
            0x46, 6, 0x2f, 0x46, 6, 0x2f, 0x48, 0xd, 0x2f, 0x48, 0xd, 0x2f, 0x49, 0xc, 0x2f, 0x49,
            0xc, 0x2f, 8, 8, 0x2f, 8, 8, 0x2f, 0x22, 0x22, 0x2f, 0x22, 0x22, 0x2f, 0xa, 0x64, 0x2f,
            0xa, 0x64, 0x2f, 0x2c, 0x67, 0x2f, 0x2c, 0x67, 0x2f, 0x2d, 0x66, 0x2f, 0x2d, 0x66,
            0x2f, 8, 8, 0x2f, 8, 8, 0x2f, 0x22, 0x22, 0x2f, 0x22, 0x22, 0x2f, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62, 0x46, 0x4b, 0x4b, 0x69, 0x64, 0x64,
        ];
        const FLAGS: [u8; 72] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0x40, 0x40, 0, 0x40, 0x40,
        ];
        const BIG: [u8; 72] = [
            2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2,
            0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2,
            2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ];
        const COUNT: [u8; 24] = [
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1,
        ];
        const OFFSET: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8,
        ];
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        sprite_offset += usize::from(OFFSET[graphics] >> 2);
        let first_oam = self.oam_current_index() + sprite_offset;
        let count = usize::from(COUNT[graphics]);
        for reverse in (0..=count).rev() {
            let table = graphics * 3 + reverse;
            let oam_delta = if count == 2 {
                match reverse {
                    2 => 0,
                    1 => 2,
                    _ => 3,
                }
            } else {
                count - reverse
            };
            crate::oam::set_helper0(
                self.ram,
                first_oam + oam_delta,
                info.x.wrapping_add(X[table] as i16 as u16),
                info.y.wrapping_add(Y[table] as i16 as u16),
                CHARACTER[table],
                info.flags | FLAGS[table],
                BIG[table],
            );
        }
    }

    pub fn sprite_draw_bnc_flail(&mut self, slot: usize, info: &PrepOamCoordsRet) {
        const SCALE: [u8; 4] = [0x33, 0x66, 0x99, 0xcc];
        const LENGTH: [u8; 32] = [
            0x10, 0x12, 0x14, 0x16, 0x18, 0x1a, 0x1c, 0x1e, 0x20, 0x22, 0x24, 0x26, 0x28, 0x2a,
            0x2c, 0x2e, 0x30, 0x2e, 0x2c, 0x2a, 0x28, 0x26, 0x24, 0x22, 0x20, 0x1e, 0x1c, 0x1a,
            0x18, 0x16, 0x14, 0x12,
        ];
        const X_OFFSET: [i8; 4] = [4, 4, 12, -5];
        const Y_OFFSET: [i8; 4] = [-2, -2, -6, -4];
        let first_oam = self.oam_current_index();
        self.ram.write_u8(DUNGMAP_VAR7, info.x as u8);
        self.ram.write_u8(DUNGMAP_VAR7 + 1, info.y as u8);
        let angle = u16::from(self.ram.sprite(slot).scratch_a())
            | (u16::from(self.ram.sprite(slot).scratch_b()) << 8);
        let length = if self.ram.sprite(slot).ai_state() < 2 {
            0
        } else {
            LENGTH[usize::from(self.ram.sprite(slot).aux2_delay())]
        };
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        let x_offset = X_OFFSET[direction] as u8;
        let y_offset = Y_OFFSET[direction] as u8;
        let perpendicular = angle.wrapping_add(0x80) & 0x1ff;
        let x_magnitude =
            Self::chain_ball_mult(SINUS_LOOKUP_TABLE[usize::from(angle & 0xff)], length);
        let x_delta = if angle & 0x100 != 0 {
            x_magnitude.wrapping_neg()
        } else {
            x_magnitude
        };
        let y_magnitude = Self::chain_ball_mult(
            SINUS_LOOKUP_TABLE[usize::from(perpendicular & 0xff)],
            length,
        );
        let y_delta = if perpendicular & 0x100 != 0 {
            y_magnitude.wrapping_neg()
        } else {
            y_magnitude
        };
        let ball_x = x_delta.wrapping_sub(4).wrapping_add(x_offset);
        let ball_y = y_delta.wrapping_sub(4).wrapping_add(y_offset);
        self.ram.write_u8(DUNGMAP_VAR8 + 1, ball_x);
        self.ram.write_u8(DUNGMAP_VAR8, ball_y);
        crate::oam::set_plain(
            self.ram,
            first_oam,
            ball_x.wrapping_add(info.x as u8),
            ball_y.wrapping_add(info.y as u8),
            0x2a,
            0x2d,
            2,
        );
        for (entry, scale) in SCALE.iter().enumerate().rev() {
            let mut dx = ((u16::from(*scale) * u16::from(x_magnitude)) >> 8) as u8;
            if x_delta & 0x80 != 0 {
                dx = dx.wrapping_neg();
            }
            let mut dy = ((u16::from(*scale) * u16::from(y_magnitude)) >> 8) as u8;
            if y_delta & 0x80 != 0 {
                dy = dy.wrapping_neg();
            }
            crate::oam::set_plain(
                self.ram,
                first_oam + 4 - entry,
                dx.wrapping_add(info.x as u8).wrapping_add(x_offset),
                dy.wrapping_add(info.y as u8).wrapping_add(y_offset),
                0x3f,
                0x2d,
                0,
            );
        }
        self.runtime
            .sprite_correct_oam_entries(self.ram, slot, 4, 0xff);
    }

    pub fn sprite_50_cannonball(&mut self, slot: usize) {
        if self.ram.sprite(slot).ai_state() == 0 {
            self.runtime.draw_single_large(self.ram, slot);
        } else {
            self.sprite_draw_big_cannonball(slot);
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        let phase = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, phase);
        self.ram.set_sprite_graphics(slot, phase >> 2 & 1);
        self.runtime.sprite_move_xy(self.ram, slot);
        let delay = self.ram.sprite(slot).main_delay();
        if delay != 0 {
            if delay == 1 {
                self.ram.set_sprite_state(slot, 0);
            }
            return;
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        if self.ram.sprite(slot).aux2_delay() == 0
            && self.runtime.sprite_check_tile_collision(self.ram, slot) != 0
        {
            self.ram.set_sprite_main_delay(slot, 16);
        }
    }

    pub fn sprite_draw_big_cannonball(&mut self, slot: usize) {
        const X: [i8; 4] = [-8, 8, -8, 8];
        const Y: [i8; 4] = [-8, -8, 8, 8];
        const CHARACTER: [u8; 8] = [0x84, 0x88, 0x88, 0x88, 0x86, 0x88, 0x88, 0x88];
        const FLAGS: [u8; 4] = [0, 0, 0xc0, 0x80];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = self.oam_current_index();
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        for (entry, piece) in (0..4).rev().enumerate() {
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[piece] as i16 as u16),
                info.y.wrapping_add(Y[piece] as i16 as u16),
                CHARACTER[graphics * 4 + piece],
                FLAGS[piece] | info.flags,
                2,
            );
        }
    }

    pub fn sprite_51_armos_statue(&mut self, slot: usize) {
        self.armos_draw(slot);
        if self.ram.sprite(slot).scratch_f() != 0 {
            self.ram.set_sprite_velocity_x(slot, 0);
            self.ram.set_sprite_velocity_y(slot, 0);
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_move_z(self.ram, slot);
        self.ram
            .set_sprite_velocity_z(slot, self.ram.sprite(slot).velocity_z().wrapping_sub(2));
        if self.ram.sprite(slot).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_z(slot, 0);
            self.ram.set_sprite_velocity_z(slot, 0);
            self.ram.set_sprite_velocity_x(slot, 0);
            self.ram.set_sprite_velocity_y(slot, 0);
        }
        if self.ram.sprite(slot).ai_state() == 0 {
            self.ram
                .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() | 0x40);
            let delay = self.ram.sprite(slot).main_delay();
            if delay == 1 {
                self.ram
                    .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() & !0x40);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                self.ram
                    .set_sprite_flags2(slot, self.ram.sprite(slot).flags2() & !0x80);
                self.ram.set_sprite_oam_flags(slot, 0xb);
            } else {
                if (((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3) == 0
                    && self
                        .ram
                        .read_u16(LINK_X)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                        .wrapping_add(31)
                        < 62
                    && self
                        .ram
                        .read_u16(LINK_Y)
                        .wrapping_add(8)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                        .wrapping_add(48)
                        < 88
                    && delay == 0
                {
                    self.ram.set_sprite_main_delay(slot, 48);
                    self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x22);
                }
                if self
                    .runtime
                    .sprite_check_damage_to_link_same_layer(self.ram, slot)
                {
                    self.runtime.sprite_nullify_hookshot_drag(self.ram);
                    self.runtime.sprite_repel_dash(self.ram);
                }
                let delay = self.ram.sprite(slot).main_delay();
                if delay != 0 {
                    self.ram.set_sprite_oam_flags(
                        slot,
                        self.ram.sprite(slot).oam_flags() ^ (delay >> 1 & 0xe),
                    );
                }
            }
        } else {
            self.runtime
                .sprite_check_damage_to_and_from_link(self.ram, slot);
            if self.runtime.sprite_return_if_recoiling(self.ram, slot) {
                return;
            }
            self.runtime.sprite_move_xy(self.ram, slot);
            let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
            if (self.ram.sprite(slot).main_delay() | self.ram.sprite(slot).z()) == 0 {
                self.ram.set_sprite_main_delay(slot, 8);
                self.ram.set_sprite_velocity_z(slot, 16);
                self.runtime
                    .sprite_apply_speed_towards_link(self.ram, slot, 12);
            }
        }
    }

    pub fn armos_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 2] = [
            DrawMultipleData {
                x: 0,
                y: -16,
                char_flags: 0x00c0,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x00e0,
                ext: 2,
            },
        ];
        let mut info = PrepOamCoordsRet::default();
        if self.ram.sprite(slot).ai_state() == 0 {
            let Some(prepared) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
                return;
            };
            info = prepared;
        }
        self.runtime
            .draw_multiple_with_info(self.ram, slot, &PIECES, &mut info);
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_4e_popo(&mut self, slot: usize) {
        self.bot_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || self.runtime.sprite_return_if_recoiling(self.ram, slot)
        {
            return;
        }
        let phase = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, phase);
        self.ram.set_sprite_scratch_a(slot, phase >> 4 & 3);
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.ram.set_sprite_main_delay(slot, 105);
                }
            }
            1 => {
                self.ram
                    .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
                if self.ram.sprite(slot).main_delay() == 0 {
                    let random = crate::misc::get_random_number(self.ram);
                    self.ram
                        .set_sprite_main_delay(slot, (random & 63).wrapping_add(128));
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    let angle = usize::from(crate::misc::get_random_number(self.ram) & 15);
                    self.ram.set_sprite_signed_field(
                        slot,
                        SpriteField::VelocityX,
                        KEESE_X_VELOCITY[angle].wrapping_shl(2),
                    );
                    self.ram.set_sprite_signed_field(
                        slot,
                        SpriteField::VelocityY,
                        KEESE_Y_VELOCITY[angle].wrapping_shl(2),
                    );
                }
            }
            2 => {
                self.ram
                    .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
                if self.ram.sprite(slot).main_delay() != 0 {
                    if (((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER))
                        & self.ram.sprite(slot).scratch_b())
                        != 0
                    {
                        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
                        return;
                    }
                    self.runtime.sprite_move_xy(self.ram, slot);
                    if self.ram.sprite(slot).wall_collision() == 0 {
                        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
                        return;
                    }
                }
                self.ram.set_sprite_ai_state(slot, 0);
                self.ram.set_sprite_main_delay(slot, 80);
            }
            _ => unreachable!("Popo has only three AI states"),
        }
    }

    pub fn bot_draw(&mut self, slot: usize) {
        const GRAPHICS: [u8; 4] = [0, 1, 0, 1];
        const FLAGS: [u8; 4] = [0, 0, 0x40, 0x40];
        let frame = usize::from(self.ram.sprite(slot).scratch_a());
        self.ram.set_sprite_graphics(slot, GRAPHICS[frame]);
        self.ram.set_sprite_oam_flags(
            slot,
            (self.ram.sprite(slot).oam_flags() & !0x40) | FLAGS[frame],
        );
        self.runtime.draw_single_large(self.ram, slot);
    }

    pub fn sprite_4c_geldman(&mut self, slot: usize) {
        const EMERGE_GRAPHICS: [u8; 8] = [3, 2, 0, 0, 0, 0, 0, 0];
        const PURSUE_GRAPHICS: [u8; 2] = [4, 5];
        const SUBMERGE_GRAPHICS: [u8; 5] = [0, 1, 2, 3, 3];
        let state = self.ram.sprite(slot).ai_state();
        if state < 2 {
            let _ = self.runtime.prep_oam_coord(self.ram, slot);
        } else {
            self.gerudo_man_draw(slot);
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || self.runtime.sprite_return_if_recoiling(self.ram, slot)
        {
            return;
        }
        self.ram.set_sprite_ignore_projectile(slot, 1);
        match state {
            0 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram
                        .set_sprite_x_low(slot, self.ram.sprite(slot).scratch_a());
                    self.ram
                        .set_sprite_x_high(slot, self.ram.sprite(slot).scratch_b());
                    self.ram
                        .set_sprite_y_low(slot, self.ram.sprite(slot).scratch_c());
                    self.ram
                        .set_sprite_y_high(slot, self.ram.sprite(slot).head_direction());
                    self.ram.set_sprite_ai_state(slot, 1);
                }
            }
            1 => {
                if (((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 7) == 0
                    && self
                        .ram
                        .read_u16(LINK_X)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                        .wrapping_add(0x30)
                        < 0x60
                    && self
                        .ram
                        .read_u16(LINK_Y)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                        .wrapping_add(0x30)
                        < 0x60
                {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 31);
                }
            }
            2 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 3);
                    self.ram.set_sprite_main_delay(slot, 96);
                    self.runtime
                        .sprite_apply_speed_towards_link(self.ram, slot, 16);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, EMERGE_GRAPHICS[usize::from(delay >> 2)]);
                }
            }
            3 => {
                self.ram.set_sprite_ignore_projectile(slot, 0);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.ram.set_sprite_main_delay(slot, 8);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, PURSUE_GRAPHICS[usize::from(delay >> 2 & 1)]);
                    self.runtime
                        .sprite_check_damage_to_and_from_link(self.ram, slot);
                    self.runtime.sprite_move_xy(self.ram, slot);
                }
            }
            4 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 0);
                    self.ram.set_sprite_main_delay(slot, 16);
                } else {
                    self.ram
                        .set_sprite_graphics(slot, SUBMERGE_GRAPHICS[usize::from(delay >> 1)]);
                }
            }
            _ => unreachable!("Geldman has only five AI states"),
        }
    }

    pub fn gerudo_man_draw(&mut self, slot: usize) {
        const X: [i8; 18] = [4, 4, 4, 4, 4, 4, -8, 8, 8, -8, 8, 8, -16, 0, 16, -16, 0, 16];
        const Y: [i8; 18] = [8, 8, 8, 8, 8, 8, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        const CHARACTER: [u8; 18] = [
            0xb8, 0xb8, 0xb8, 0xb8, 0xb8, 0xb8, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa4, 0xa2,
            0xa0, 0xa0, 0xa2, 0xa4,
        ];
        const FLAGS: [u8; 18] = [
            0, 0, 0, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0,
        ];
        const BIG: [u8; 18] = [0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = self.oam_current_index();
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        for (entry, piece) in (0..3).rev().enumerate() {
            let table = graphics * 3 + piece;
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[table] as i16 as u16),
                info.y.wrapping_add(Y[table] as i16 as u16),
                CHARACTER[table],
                FLAGS[table] | info.flags,
                BIG[table],
            );
        }
    }

    pub fn sprite_4d_toppo(&mut self, slot: usize) {
        const X_OFFSET: [i8; 4] = [-32, 32, 0, 0];
        const Y_OFFSET: [i8; 4] = [0, 0, -32, 32];
        let state = self.ram.sprite(slot).ai_state();
        if state != 0 {
            self.ram
                .set_sprite_object_priority(slot, self.ram.sprite(slot).object_priority() | 0x30);
            self.toppo_draw(slot);
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        match state {
            0 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.ram.set_sprite_main_delay(slot, 8);
                    let direction = usize::from(crate::misc::get_random_number(self.ram) & 3);
                    let home_x = u16::from(self.ram.sprite(slot).scratch_a())
                        | (u16::from(self.ram.sprite(slot).scratch_b()) << 8);
                    let home_y = u16::from(self.ram.sprite(slot).scratch_c())
                        | (u16::from(self.ram.sprite(slot).head_direction()) << 8);
                    self.set_sprite_x(slot, home_x.wrapping_add(X_OFFSET[direction] as i16 as u16));
                    self.set_sprite_y(slot, home_y.wrapping_add(Y_OFFSET[direction] as i16 as u16));
                }
            }
            1 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 16);
                } else {
                    self.ram.set_sprite_graphics(slot, delay >> 2 & 1);
                    self.toppo_verify_tile(slot);
                }
            }
            2 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 3);
                    self.ram.set_sprite_velocity_z(slot, 64);
                }
                self.ram.set_sprite_graphics(slot, 2);
                self.toppo_verify_tile(slot);
            }
            3 => {
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                self.runtime.sprite_move_z(self.ram, slot);
                self.ram.set_sprite_velocity_z(
                    slot,
                    self.ram.sprite(slot).velocity_z().wrapping_sub(2),
                );
                if self.ram.sprite(slot).signed_field(SpriteField::Z) < 0 {
                    self.ram.set_sprite_z(slot, 0);
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.ram.set_sprite_main_delay(slot, 16);
                    self.toppo_verify_tile(slot);
                }
            }
            4 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 0);
                    self.ram.set_sprite_main_delay(slot, 32);
                } else {
                    self.ram.set_sprite_graphics(slot, delay >> 2 & 1);
                }
                self.toppo_verify_tile(slot);
            }
            5 => self.runtime.toppo_flustered(self.ram, slot),
            _ => unreachable!("Toppo has only six AI states"),
        }
    }

    pub fn toppo_verify_tile(&mut self, slot: usize) {
        let mut x = self.sprite_x(slot);
        let y = self.sprite_y(slot);
        if self.runtime.get_tile_attribute(self.ram, 0, &mut x, y) != 0x40 {
            self.ram.set_sprite_ai_state(slot, 5);
        }
    }

    pub fn toppo_draw(&mut self, slot: usize) {
        const X: [i8; 15] = [0, 8, 8, 0, 8, 8, 0, 0, 8, 0, 0, 0, 0, 0, 0];
        const Y: [i8; 15] = [8, 8, 8, 8, 8, 8, 0, 8, 8, 0, 0, 0, 0, 0, 0];
        const CHARACTER: [u8; 15] = [
            0xc8, 0xc8, 0xc8, 0xca, 0xca, 0xca, 0xc0, 0xc8, 0xc8, 0xc2, 0xc2, 0xc2, 0xc2, 0xc2,
            0xc2,
        ];
        const FLAGS: [u8; 15] = [
            0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0x40, 0, 0, 0, 0x40, 0x40, 0x40,
        ];
        const BIG: [u8; 15] = [0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 2, 2, 2, 2, 2];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = self.oam_current_index();
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        let y_base = self
            .sprite_y(slot)
            .wrapping_sub(self.ram.read_u16(BG2_VOFS_COPY2));
        for (entry, piece) in (0..3).rev().enumerate() {
            let table = graphics * 3 + piece;
            let big = BIG[table];
            let mut flags = FLAGS[table] | info.flags;
            if big == 0 {
                flags = (flags & !0xf) | 2;
            }
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[table] as i16 as u16),
                (if big != 0 { info.y } else { y_base }).wrapping_add(Y[table] as i16 as u16),
                CHARACTER[table],
                flags,
                big,
            );
        }
    }

    pub fn sprite_4b_green_knife_guard(&mut self, slot: usize) {
        const GRAPHICS: [u8; 8] = [0, 2, 4, 6, 1, 3, 5, 7];
        const X_VELOCITY: [i8; 8] = [12, -12, 0, 0, 18, -18, 0, 0];
        const Y_VELOCITY: [i8; 8] = [0, 0, 12, -12, 0, 0, 18, -18];
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        let animation = usize::from(self.ram.sprite(slot).subtype2() >> 1 & 4);
        self.ram
            .set_sprite_graphics(slot, GRAPHICS[direction + animation]);
        self.recruit_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || self.runtime.sprite_return_if_recoiling(self.ram, slot)
        {
            return;
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        self.runtime.sprite_move_xy(self.ram, slot);
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        if self.ram.sprite(slot).ai_state() != 0 {
            self.green_knife_guard_moving(slot);
            return;
        }
        if self.ram.sprite(slot).main_delay() != 0 {
            return;
        }
        let random_delay = (crate::misc::get_random_number(self.ram) & 0x3f).wrapping_add(0x30);
        self.ram.set_sprite_main_delay(slot, random_delay);
        self.ram
            .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
        let facing = self.ram.sprite(slot).head_direction();
        self.ram.set_sprite_scratch_d(slot, facing);
        let (link_direction, x_difference, y_difference) =
            self.runtime.sprite_direction_and_deltas(self.ram, slot);
        let mut velocity = usize::from(facing);
        if facing == link_direction
            && (x_difference.wrapping_add(0x10) < 0x20 || y_difference.wrapping_add(0x10) < 0x20)
        {
            velocity += 4;
            self.ram.set_sprite_main_delay(slot, 128);
        }
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, X_VELOCITY[velocity]);
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityY, Y_VELOCITY[velocity]);
    }

    pub fn green_knife_guard_moving(&mut self, slot: usize) {
        const HEAD_DIRECTION: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
        let mut delay = 0x10;
        let should_stop = if self.ram.sprite(slot).wall_collision() == 0 {
            if self.ram.sprite(slot).main_delay() != 0 {
                false
            } else {
                delay = 0x30;
                true
            }
        } else {
            true
        };
        if should_stop {
            self.ram.set_sprite_main_delay(slot, delay);
            self.ram.set_sprite_velocity_x(slot, 0);
            self.ram.set_sprite_velocity_y(slot, 0);
            let index = (usize::from(self.ram.sprite(slot).scratch_d()) * 2)
                | usize::from(crate::misc::get_random_number(self.ram) & 1);
            self.ram
                .set_sprite_head_direction(slot, HEAD_DIRECTION[index]);
            self.ram.set_sprite_ai_state(slot, 0);
        }
        let increment = if self.ram.sprite(slot).aux1_delay() != 0 {
            2
        } else {
            1
        };
        self.ram.set_sprite_subtype2(
            slot,
            self.ram.sprite(slot).subtype2().wrapping_add(increment),
        );
    }

    pub fn recruit_draw(&mut self, slot: usize) {
        const HEAD_CHARACTER: [u8; 4] = [0x42, 0x42, 0x40, 0x44];
        const HEAD_FLAGS: [u8; 4] = [0x40, 0, 0, 0];
        const BODY_X: [i16; 8] = [2, 2, -2, -2, 0, 0, 0, 0];
        const BODY_CHARACTER: [u8; 8] = [0x8a, 0x8c, 0x8a, 0x8c, 0x86, 0x88, 0x8e, 0xa0];
        const BODY_FLAGS: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0, 0, 0];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = self.oam_current_index();
        let head = usize::from(self.ram.sprite(slot).head_direction());
        crate::oam::set_helper0(
            self.ram,
            first_oam,
            info.x,
            info.y.wrapping_sub(11),
            HEAD_CHARACTER[head],
            HEAD_FLAGS[head] | info.flags,
            2,
        );
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        crate::oam::set_helper0(
            self.ram,
            first_oam + 1,
            info.x.wrapping_add(BODY_X[graphics] as u16),
            info.y,
            BODY_CHARACTER[graphics],
            BODY_FLAGS[graphics] | info.flags,
            2,
        );
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_4a_bomb_guard(&mut self, slot: usize) {
        let variant = self.ram.sprite(slot).scratch_c();
        if variant == 0 {
            self.bomb_guard(slot);
            return;
        }
        if variant < 2 {
            self.sprite_bomb_explosion_incoming(slot);
            return;
        }
        if variant == 2 {
            for other in (0..16).rev() {
                if other != usize::from(self.ram.read_u8(CUR_OBJECT_INDEX))
                    && self.ram.sprite(other).state() >= 9
                    && ((((self.ram.read_u8(FRAME_COUNTER) ^ other as u8) & 7)
                        | self.ram.sprite(other).hit_timer())
                        == 0)
                {
                    self.sprite_bomb_check_damage_to_sprite(slot, other);
                }
            }
            let _ = self.runtime.sprite_check_damage_to_link(self.ram, slot);
        }
        self.sprite_draw_sprite_bomb_explosion(slot);
        if self.ram.sprite(slot).aux1_delay() == 0 {
            self.ram.set_sprite_state(slot, 0);
        }
    }

    pub fn sprite_bomb_check_damage_to_sprite(&mut self, slot: usize, target: usize) {
        let x = self.sprite_x(slot).wrapping_sub(16);
        let y = self.sprite_y(slot).wrapping_sub(16);
        if !self
            .runtime
            .sprite_bomb_hitbox_overlaps(self.ram, target, x, y, 48)
            || self.ram.sprite(target).kind() == 0x11
        {
            return;
        }
        self.runtime
            .ancilla_check_damage_to_sprite_preset(self.ram, target, 8);
        let target_y = self
            .sprite_y(target)
            .wrapping_sub(u16::from(self.ram.sprite(target).z()));
        let velocity = self.runtime.sprite_project_speed_towards_location(
            self.ram,
            slot,
            self.sprite_x(target),
            target_y,
            32,
        );
        self.ram.set_sprite_recoil_y(target, velocity.y);
        self.ram.set_sprite_recoil_x(target, velocity.x);
    }

    pub fn sprite_bomb_explosion_incoming(&mut self, slot: usize) {
        if self.ram.sprite(slot).scratch_e() != 0 {
            self.ram
                .set_sprite_object_priority(slot, self.ram.sprite(slot).object_priority() | 48);
        }
        self.runtime.draw_single_large(self.ram, slot);
        if self.ram.sprite(slot).hit_timer() != 0 || self.ram.sprite(slot).aux1_delay() == 1 {
            self.ram.set_sprite_hit_timer(slot, 0);
            if self.ram.sprite(slot).state() == 10 {
                self.ram.write_u8(LINK_STATE_BITS, 0);
                self.ram.write_u8(LINK_PICKING_THROW_STATE, 0);
            }
            self.runtime.queue_sfx2_with_pan(self.ram, slot, 0xc);
            self.ram
                .set_sprite_scratch_c(slot, self.ram.sprite(slot).scratch_c().wrapping_add(1));
            self.ram.set_sprite_flags4(slot, 9);
            self.ram.set_sprite_oam_flags(slot, 2);
            self.ram.set_sprite_aux1_delay(slot, 31);
            self.ram.set_sprite_state(slot, 6);
            self.ram.set_sprite_flags2(slot, 3);
            return;
        }
        let delay = self.ram.sprite(slot).aux1_delay();
        if delay < 64 {
            self.ram.set_sprite_oam_flags(
                slot,
                (self.ram.sprite(slot).oam_flags() & !0xe) | (delay >> 1 & 0xe),
            );
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).aux3_delay() == 0 {
            self.runtime.sprite_check_damage_from_link(self.ram, slot);
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        if self.ram.read_u8(PLAYER_IS_INDOORS) != 0 {
            let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        }
        self.runtime
            .thrown_sprite_tile_and_sprite_interaction(self.ram, slot);
    }

    pub fn bomb_guard(&mut self, slot: usize) {
        self.bomb_trooper_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        let direction = self
            .runtime
            .sprite_direction_to_face_link_direction(self.ram, slot);
        self.ram.set_sprite_head_direction(slot, direction);
        self.ram.set_sprite_scratch_d(slot, direction);
        if self.ram.sprite(slot).ai_state() == 0 {
            if self.ram.sprite(slot).main_delay() == 0 {
                self.ram.set_sprite_ai_state(slot, 1);
                self.ram.set_sprite_main_delay(slot, 112);
            }
        } else {
            let delay = self.ram.sprite(slot).main_delay();
            if delay == 0 {
                self.ram.set_sprite_ai_state(slot, 0);
                self.ram.set_sprite_main_delay(slot, 32);
                return;
            }
            self.ram.set_sprite_subtype2(slot, u8::from(delay >= 80));
            if delay == 32 {
                self.bomb_guard_create_bomb(slot);
            }
            const GRAPHICS: [u8; 64] = [
                25, 25, 24, 24, 23, 23, 23, 23, 19, 19, 18, 18, 17, 17, 17, 17, 16, 16, 15, 15, 14,
                14, 14, 14, 22, 22, 21, 21, 20, 20, 20, 20, 20, 20, 18, 18, 18, 16, 16, 16, 21, 21,
                8, 8, 8, 6, 6, 6, 22, 22, 4, 4, 4, 3, 3, 3, 23, 23, 15, 15, 15, 11, 11, 11,
            ];
            let index = usize::from((direction << 3) | (delay >> 4)) + 32;
            self.ram.set_sprite_graphics(slot, GRAPHICS[index]);
        }
    }

    pub fn bomb_guard_create_bomb(&mut self, slot: usize) {
        const X: [i8; 4] = [0, 1, 9, -8];
        const Y: [i8; 4] = [-12, -12, -15, -13];
        const Z_VELOCITY: [u8; 16] = [
            32, 40, 48, 56, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        ];
        let Some((spawned, info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0x4a)
        else {
            return;
        };
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        self.set_sprite_x(spawned, info.r0_x.wrapping_add(X[direction] as i16 as u16));
        self.set_sprite_y(spawned, info.r2_y.wrapping_add(Y[direction] as i16 as u16));
        self.runtime
            .sprite_apply_speed_towards_link(self.ram, spawned, 16);
        self.ram.set_sprite_scratch_c(spawned, 1);
        let (_, x_difference, y_difference) =
            self.runtime.sprite_direction_and_deltas(self.ram, spawned);
        let absolute_x = if x_difference & 0x80 != 0 {
            x_difference.wrapping_neg()
        } else {
            x_difference
        };
        let absolute_y = if y_difference & 0x80 != 0 {
            y_difference.wrapping_neg()
        } else {
            y_difference
        };
        self.ram.set_sprite_velocity_z(
            spawned,
            Z_VELOCITY[usize::from((absolute_y | absolute_x) >> 4)],
        );
        self.ram
            .set_sprite_flags3(spawned, (self.ram.sprite(slot).flags3() & 0xee) | 0x18);
        self.ram.set_sprite_oam_flags(spawned, 8);
        self.ram.set_sprite_aux1_delay(spawned, 255);
        self.ram.set_sprite_health(spawned, 0);
        self.runtime.queue_sfx3_with_pan(self.ram, spawned, 0x13);
    }

    pub fn bomb_trooper_draw(&mut self, slot: usize) {
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.sprite_draw_guard_head(slot, &info, 2);
        self.sprite_draw_bnc_body(slot, &info, 1);
        if self.ram.sprite(slot).graphics() < 20 {
            self.sprite_draw_bomb_guard_arm(slot, &info);
        }
        self.runtime.draw_shadow_custom(self.ram, slot, &info, 10);
    }

    pub fn sprite_draw_bomb_guard_arm(&mut self, slot: usize, info: &PrepOamCoordsRet) {
        const X: [i8; 8] = [-1, 1, 2, 0, 9, 9, -8, -8];
        const Y: [i8; 8] = [-12, -12, -12, -12, -16, -14, -12, -14];
        let table =
            usize::from((self.ram.sprite(slot).scratch_d() * 2) | self.ram.sprite(slot).subtype2());
        crate::oam::set_helper0(
            self.ram,
            self.oam_current_index(),
            info.x.wrapping_add(X[table] as i16 as u16),
            info.y.wrapping_add(Y[table] as i16 as u16),
            0x6e,
            (info.flags & 0x30) | 8,
            2,
        );
    }

    pub fn sprite_draw_sprite_bomb_explosion(&mut self, slot: usize) {
        const X: [i8; 16] = [-12, 12, -12, 12, -8, 8, -8, 8, -8, 8, -8, 8, 0, 0, 0, 0];
        const Y: [i8; 16] = [-12, -12, 12, 12, -8, -8, 8, 8, -8, -8, 8, 8, 0, 0, 0, 0];
        const CHARACTER: [u8; 16] = [
            0x88, 0x88, 0x88, 0x88, 0x8a, 0x8a, 0x8a, 0x8a, 0x84, 0x84, 0x84, 0x84, 0x86, 0x86,
            0x86, 0x86,
        ];
        const FLAGS: [u8; 16] = [
            0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0,
        ];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = self.oam_current_index();
        let base = usize::from(self.ram.sprite(slot).aux1_delay() >> 1 & 0xc);
        for (entry, piece) in (0..4).rev().enumerate() {
            let table = base + piece;
            crate::oam::write(
                self.ram,
                first_oam + entry,
                z3_core::c_types::OamEnt {
                    x: info.x.wrapping_add(X[table] as i16 as u16) as u8,
                    y: info.y.wrapping_add(Y[table] as i16 as u16) as u8,
                    charnum: CHARACTER[table],
                    flags: FLAGS[table] | info.flags,
                },
            );
        }
        self.runtime
            .sprite_correct_oam_entries(self.ram, slot, 3, 2);
    }

    pub fn sprite_41_blue_guard(&mut self, slot: usize) {
        if self.ram.sprite(slot).scratch_c() != 0 {
            self.probe(slot);
        } else {
            self.guard_main(slot);
        }
    }

    pub fn probe(&mut self, slot: usize) {
        self.runtime.sprite_add_xy(
            self.ram,
            slot,
            self.ram.sprite(slot).signed_field(SpriteField::VelocityX),
            self.ram.sprite(slot).signed_field(SpriteField::VelocityY),
        );
        let parent = usize::from(self.ram.sprite(slot).scratch_c().wrapping_sub(1));
        let is_close = if self.ram.sprite(parent).kind() == 0xce {
            let x = self
                .ram
                .read_u16(CUR_SPRITE_X)
                .wrapping_sub(self.ram.read_u16(LINK_X))
                .wrapping_add(16);
            let y = self
                .ram
                .read_u16(LINK_Y)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                .wrapping_add(24);
            x < 32 && y < 32
        } else {
            if (self.runtime.probe_check_tile_solidity(self.ram, slot)
                && self.ram.read_u8(SPRITE_TILETYPE) != 9)
                || self.ram.read_u8(LINK_CAPE_MODE) != 0
            {
                self.ram.set_sprite_state(slot, 0);
                return;
            }
            let x = self
                .ram
                .read_u16(CUR_SPRITE_X)
                .wrapping_sub(self.ram.read_u16(LINK_X));
            let y = self
                .ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_sub(self.ram.read_u16(LINK_Y));
            x < 16
                && y < 16
                && self.ram.sprite(slot).floor() == self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL)
        };
        if is_close {
            if self.ram.sprite(parent).ai_state() != 3 {
                self.ram.set_sprite_ai_state(parent, 3);
                if self.ram.sprite(parent).kind() != 0xce {
                    self.ram.set_sprite_main_delay(parent, 16);
                    self.ram.set_sprite_subtype2(parent, 0);
                }
            }
            self.ram.set_sprite_state(slot, 0);
        } else {
            let Some(oam) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
                return;
            };
            if oam.x | oam.y >= 256 {
                self.ram.set_sprite_state(slot, 0);
            }
        }
    }

    pub fn guard_main(&mut self, slot: usize) {
        const DIRECTION_LOCK: [u8; 4] = [3, 2, 0, 1];
        const LOCKED_GRAPHICS: [u8; 4] = [8, 0, 12, 5];
        const PATROL_DELAY: [u8; 4] = [0x60, 0xc0, 0xff, 0x40];
        const X_VELOCITY: [i8; 4] = [8, -8, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, 8, -8];
        const HEAD_DIRECTIONS: [u8; 32] = [
            0, 2, 2, 2, 0, 3, 3, 3, 1, 3, 3, 3, 1, 2, 2, 2, 2, 0, 0, 0, 2, 1, 1, 1, 3, 1, 1, 1, 3,
            0, 0, 0,
        ];
        const ANIMATION_CLOCK: [u8; 4] = [13, 13, 12, 12];
        let saved_graphics = self.ram.sprite(slot).graphics();
        let saved_direction = self.ram.sprite(slot).scratch_d();
        if self.ram.sprite(slot).aux1_delay() != 0 {
            self.ram
                .set_sprite_scratch_d(slot, DIRECTION_LOCK[usize::from(saved_direction)]);
            self.ram
                .set_sprite_graphics(slot, LOCKED_GRAPHICS[usize::from(saved_direction)]);
        }
        self.guard_handle_all_animation(slot);
        self.ram.set_sprite_scratch_d(slot, saved_direction);
        self.ram.set_sprite_graphics(slot, saved_graphics);
        if self.ram.sprite(slot).state() == 5 {
            if self.ram.read_u8(SUBMODULE_INDEX) == 0 {
                self.ram
                    .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
                self.guard_tick_and_update_body(slot);
                self.ram
                    .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
                self.guard_tick_and_update_body(slot);
            }
            return;
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.guard_parry_sword_attacks(self.ram, slot);
        if (self.runtime.sprite_check_damage_to_link(self.ram, slot)
            || self.ram.read_u8(SPRITE_ALERT_FLAG) != 0)
            && self.ram.sprite(slot).ai_state() < 3
        {
            self.ram.set_sprite_ai_state(slot, 3);
            self.guard_set_timer_and_assert_tile_hit_box(slot, 0x20);
        } else if self.ram.sprite(slot).scratch_f() >= 4 {
            self.ram.set_sprite_ai_state(slot, 4);
            self.guard_set_timer_and_assert_tile_hit_box(slot, 0x80);
        }
        if self.runtime.sprite_return_if_recoiling(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).subtype() & 7 < 5 {
            if self.ram.sprite(slot).wall_collision() == 0 {
                self.runtime.sprite_move_xy(self.ram, slot);
            }
            let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        } else {
            self.runtime.sprite_move_xy(self.ram, slot);
        }
        if self.ram.sprite(slot).ai_state() != 4 {
            self.ram.set_sprite_scratch_g(slot, 0);
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                if self.ram.sprite(slot).main_delay() != 0 {
                    return;
                }
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                let subtype = self.ram.sprite(slot).subtype();
                if subtype != 0 && subtype & 7 < 5 {
                    self.ram
                        .set_sprite_main_delay(slot, PATROL_DELAY[usize::from(subtype >> 3 & 3)]);
                    self.ram.set_sprite_scratch_d(slot, saved_direction ^ 1);
                    self.ram.set_sprite_subtype2(slot, 0);
                } else {
                    let random_delay =
                        (crate::misc::get_random_number(self.ram) & 0x3f).wrapping_add(0x28);
                    self.ram.set_sprite_main_delay(slot, random_delay);
                    let old = self.ram.sprite(slot).scratch_d();
                    let new_direction = crate::misc::get_random_number(self.ram) & 3;
                    self.ram.set_sprite_scratch_d(slot, new_direction);
                    if old == new_direction || (old ^ new_direction) & 2 != 0 {
                        return;
                    }
                }
                self.ram.set_sprite_aux1_delay(slot, 12);
            }
            1 => {
                self.sprite_guard_send_out_probe(slot);
                if self.ram.sprite(slot).subtype() & 7 >= 5 {
                    self.guard_shoot_probe_and_stuff(slot);
                    return;
                }
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_velocity_x(slot, 0);
                    self.ram.set_sprite_velocity_y(slot, 0);
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 160);
                    return;
                }
                if self.ram.sprite(slot).subtype2() & 1 == 0 {
                    self.ram.set_sprite_main_delay(
                        slot,
                        self.ram.sprite(slot).main_delay().wrapping_add(1),
                    );
                }
                if self.ram.sprite(slot).wall_collision() & 0xf != 0 {
                    self.ram
                        .set_sprite_scratch_d(slot, self.ram.sprite(slot).scratch_d() ^ 1);
                    self.guard_set_glance_to12(slot);
                }
                let direction = usize::from(self.ram.sprite(slot).scratch_d());
                self.ram.set_sprite_signed_field(
                    slot,
                    SpriteField::VelocityX,
                    X_VELOCITY[direction],
                );
                self.ram.set_sprite_signed_field(
                    slot,
                    SpriteField::VelocityY,
                    Y_VELOCITY[direction],
                );
                self.ram.set_sprite_head_direction(slot, direction as u8);
                self.guard_tick_and_update_body(slot);
            }
            2 => {
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                self.sprite_guard_send_out_probe(slot);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_main_delay(slot, 0x20);
                    self.ram.set_sprite_ai_state(slot, 0);
                } else if delay < 0x80 {
                    let index =
                        usize::from((self.ram.sprite(slot).scratch_d() * 8) | (delay >> 3 & 7));
                    self.ram
                        .set_sprite_head_direction(slot, HEAD_DIRECTIONS[index]);
                }
            }
            3 => {
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                let direction = self
                    .runtime
                    .sprite_direction_to_face_link_direction(self.ram, slot);
                self.ram.set_sprite_head_direction(slot, direction);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.guard_set_timer_and_assert_tile_hit_box(slot, 255);
                }
            }
            4 => {
                if self.ram.sprite(slot).main_delay() != 0 {
                    self.soldier_func12(slot);
                } else {
                    let direction = usize::from(self.ram.sprite(slot).scratch_d());
                    self.ram
                        .set_sprite_animation_clock(slot, ANIMATION_CLOCK[direction]);
                    self.ram.set_sprite_velocity_x(slot, 0);
                    self.ram.set_sprite_velocity_y(slot, 0);
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 160);
                }
            }
            _ => unreachable!("guard has only five AI states"),
        }
    }

    pub fn guard_set_glance_to12(&mut self, slot: usize) {
        self.ram.set_sprite_aux1_delay(slot, 12);
    }

    pub fn guard_shoot_probe_and_stuff(&mut self, slot: usize) {
        const X_STEP: [i8; 8] = [1, 1, -1, -1, -1, -1, 1, 1];
        const Y_STEP: [i8; 8] = [-1, 1, 1, -1, -1, 1, 1, -1];
        const X_VELOCITY: [i8; 8] = [8, 0, -8, 0, -8, 0, 8, 0];
        const Y_VELOCITY: [i8; 8] = [0, 8, 0, -8, 0, 8, 0, -8];
        const DIRECTION: [u8; 8] = [0, 2, 1, 3, 1, 2, 0, 3];
        const MASK2: [u8; 8] = [1, 4, 2, 8, 2, 4, 1, 8];
        const MASK: [u8; 8] = [8, 1, 4, 2, 8, 2, 4, 1];
        const NEXT2: [u8; 8] = [1, 2, 3, 0, 5, 6, 7, 4];
        const NEXT: [u8; 8] = [3, 0, 1, 2, 7, 4, 5, 6];
        let mut index = usize::from(self.ram.sprite(slot).scratch_b());
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, X_STEP[index]);
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityY, Y_STEP[index]);
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        let aux = self.ram.sprite(slot).aux2_delay();
        if aux != 0 {
            if aux == 44 {
                index = usize::from(NEXT[index]);
                self.ram.set_sprite_scratch_b(slot, index as u8);
            }
        } else if self.ram.sprite(slot).wall_collision() & MASK[index] == 0 {
            self.ram.set_sprite_aux2_delay(slot, 88);
        }
        if self.ram.sprite(slot).wall_collision() & MASK2[index] != 0 {
            index = usize::from(NEXT2[index]);
            self.ram.set_sprite_scratch_b(slot, index as u8);
        }
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, X_VELOCITY[index]);
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityY, Y_VELOCITY[index]);
        self.ram.set_sprite_head_direction(slot, DIRECTION[index]);
        self.ram.set_sprite_scratch_d(slot, DIRECTION[index]);
        self.guard_tick_and_update_body(slot);
    }

    pub fn guard_tick_and_update_body(&mut self, slot: usize) {
        const GRAPHICS: [u8; 32] = [
            11, 12, 13, 12, 4, 5, 6, 5, 0, 1, 2, 3, 7, 8, 9, 10, 17, 18, 17, 18, 7, 8, 7, 8, 3, 4,
            3, 4, 13, 14, 13, 14,
        ];
        let phase = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, phase);
        let index =
            usize::from(self.ram.sprite(slot).scratch_d()) * 4 + usize::from(phase >> 3 & 3);
        self.ram.set_sprite_graphics(slot, GRAPHICS[index]);
    }

    pub fn guard_set_timer_and_assert_tile_hit_box(&mut self, slot: usize, delay: u8) {
        self.ram.set_sprite_main_delay(slot, delay);
        self.ram.set_sprite_subtype(slot, 0);
        self.ram
            .set_sprite_flags(slot, (self.ram.sprite(slot).flags() & 0xf) | 0x60);
    }

    pub fn soldier_func12(&mut self, slot: usize) {
        if (((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 0x1f) == 0 {
            if self.ram.sprite(slot).scratch_g() == 0 {
                self.ram.set_sprite_scratch_g(slot, 1);
                self.runtime.queue_sfx3_with_pan(self.ram, slot, 4);
            }
            self.runtime
                .sprite_apply_speed_towards_link(self.ram, slot, 16);
            let direction = self
                .runtime
                .sprite_direction_to_face_link_direction(self.ram, slot);
            self.ram.set_sprite_scratch_d(slot, direction);
            self.ram.set_sprite_head_direction(slot, direction);
        }
        self.guard_apply_speed_in_direction(slot);
        self.ram
            .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
        self.guard_tick_and_update_body(slot);
    }

    pub fn guard_apply_speed_in_direction(&mut self, slot: usize) {
        const VELOCITY: [i8; 6] = [14, -14, 0, 0, 14, -14];
        if self.ram.sprite(slot).wall_collision() == 0 {
            return;
        }
        let index = if self.ram.sprite(slot).wall_collision() & 3 != 0 {
            2 + usize::from(self.runtime.sprite_is_below_link(self.ram, slot))
        } else {
            usize::from(self.runtime.sprite_is_right_of_link(self.ram, slot))
        };
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, VELOCITY[index]);
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityY, VELOCITY[index + 2]);
    }

    pub fn sprite_guard_send_out_probe(&mut self, slot: usize) {
        const STAGGER: [u8; 4] = [0x10, 0x30, 0, 0x20];
        if ((((slot as u8).wrapping_add(self.ram.read_u8(FRAME_COUNTER))) & 3)
            | self.ram.sprite(slot).pause())
            != 0
        {
            return;
        }
        let animation = self.ram.sprite(slot).animation_clock();
        self.ram
            .set_sprite_animation_clock(slot, animation.wrapping_add(1));
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        let angle = ((animation & 0x1f).wrapping_add(STAGGER[direction])) & 0x3f;
        self.sprite_spawn_probe_always(slot, angle);
    }

    pub fn sprite_spawn_probe_always(&mut self, slot: usize, angle: u8) {
        const X_VELOCITY: [i8; 64] = [
            -16, -16, -16, -16, -16, -16, -16, -16, -16, -14, -12, -10, -8, -6, -4, -2, 0, 2, 4, 6,
            8, 10, 12, 14, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14, 12,
            10, 8, 6, 4, 2, 0, -2, -4, -6, -8, -10, -12, -14, -16, -16, -16, -16, -16, -16, -16,
            -16, -16,
        ];
        const Y_VELOCITY: [i8; 64] = [
            0, 2, 4, 6, 8, 10, 12, 14, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
            16, 14, 12, 10, 8, 6, 4, 2, 0, -2, -4, -6, -8, -10, -12, -14, -16, -16, -16, -16, -16,
            -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -14, -12, -10, -8, -6, -4,
            -2, 0,
        ];
        let Some((spawned, info)) = self
            .runtime
            .sprite_spawn_dynamically_ex(self.ram, slot, 0x41, 10)
        else {
            return;
        };
        self.set_sprite_x(spawned, info.r0_x.wrapping_add(8));
        self.set_sprite_y(spawned, info.r2_y.wrapping_add(4));
        self.ram.set_sprite_scratch_d(spawned, angle);
        self.ram.set_sprite_signed_field(
            spawned,
            SpriteField::VelocityX,
            X_VELOCITY[usize::from(angle)],
        );
        self.ram.set_sprite_signed_field(
            spawned,
            SpriteField::VelocityY,
            Y_VELOCITY[usize::from(angle)],
        );
        self.ram
            .set_sprite_flags2(spawned, (self.ram.sprite(spawned).flags2() & 0xf0) | 0xa0);
        self.ram.set_sprite_scratch_c(spawned, slot as u8 + 1);
        self.ram
            .set_sprite_ignore_projectile(spawned, slot as u8 + 1);
        self.ram.set_sprite_flags4(spawned, 0x40);
        self.ram.set_sprite_flags3(spawned, 0x40);
        self.ram.set_sprite_deflection_bits(spawned, 2);
    }

    pub fn guard_handle_all_animation(&mut self, slot: usize) {
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.guard_animate_head(slot, 0, &info);
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        self.guard_animate_body(
            slot,
            usize::from(SOLDIER_DRAW2_OAM_INDEX[direction] >> 2),
            &info,
        );
        self.guard_animate_weapon(slot, &info);
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            self.runtime
                .draw_shadow_custom(self.ram, slot, &info, SOLDIER_DRAW_SHADOW[direction]);
        }
    }

    pub fn guard_animate_head(&mut self, slot: usize, oam_offset: usize, info: &PrepOamCoordsRet) {
        let direction = usize::from(self.ram.sprite(slot).head_direction());
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        crate::oam::set_helper0(
            self.ram,
            self.oam_current_index() + oam_offset,
            info.x,
            info.y.wrapping_sub(SOLDIER_DRAW1_Y[graphics] as u16),
            SOLDIER_DRAW1_CHAR[direction],
            SOLDIER_DRAW1_FLAGS[direction] | info.flags,
            2,
        );
    }

    pub fn guard_animate_body(&mut self, slot: usize, oam_offset: usize, info: &PrepOamCoordsRet) {
        let graphics = usize::from(self.ram.sprite(slot).graphics()) * 4;
        let sprite_type = self.ram.sprite(slot).kind();
        let mut oam = self.oam_current_index() + oam_offset;
        for piece in (0..4).rev() {
            let table = graphics + piece;
            if sprite_type >= 0x46
                && (SOLDIER_DRAW2_BIG[table] == 0
                    || (piece == 3 && SOLDIER_DRAW2_CHARACTER[table] == 0x20))
            {
                continue;
            }
            let mut flags = SOLDIER_DRAW2_FLAGS[table] | info.flags;
            if SOLDIER_DRAW2_CHARACTER[table] == 0x20 {
                flags = (flags & 0xf1) | 2;
            } else if SOLDIER_DRAW2_BIG[table] == 0 {
                flags = (flags & 0xf1) | 8;
            }
            crate::oam::set_helper0(
                self.ram,
                oam,
                info.x.wrapping_add(SOLDIER_DRAW2_X[table] as i16 as u16),
                info.y.wrapping_add(SOLDIER_DRAW2_Y[table] as i16 as u16),
                SOLDIER_DRAW2_CHARACTER[table],
                flags,
                SOLDIER_DRAW2_BIG[table],
            );
            if SOLDIER_DRAW2_CHARACTER[table] == 0x20 && sprite_type == 0x46 {
                let mut entry = crate::oam::read(self.ram, oam);
                entry.y = 0xf0;
                crate::oam::write(self.ram, oam, entry);
            }
            oam += 1;
        }
    }

    pub fn guard_animate_weapon(&mut self, slot: usize, info: &PrepOamCoordsRet) {
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        let first_oam =
            self.oam_current_index() + usize::from(SOLDIER_DRAW3_OAM_INDEX[direction] >> 2);
        let graphics = usize::from(self.ram.sprite(slot).graphics()) * 2;
        let sprite_type = self.ram.sprite(slot).kind();
        for (entry, piece) in (0..2).rev().enumerate() {
            let table = graphics + piece;
            self.ram.write_u16(
                DUNGMAP_VAR8,
                (u16::from(SOLDIER_DRAW3_X[table] as u8) << 8)
                    | u16::from(SOLDIER_DRAW3_Y[table] as u8),
            );
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(SOLDIER_DRAW3_X[table] as i16 as u16),
                info.y.wrapping_add(SOLDIER_DRAW3_Y[table] as i16 as u16),
                SOLDIER_DRAW3_CHARACTER[table] + u8::from(sprite_type < 0x43) * 3,
                SOLDIER_DRAW3_FLAGS[table] | info.flags,
                0,
            );
        }
    }

    pub fn sprite_45_hog_spear_man(&mut self, slot: usize) {
        self.guard_handle_all_animation(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.bolt_guard_trigger_chase_theme(slot);
        self.runtime.guard_parry_sword_attacks(self.ram, slot);
        if self.runtime.sprite_return_if_recoiling(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).wall_collision() == 0 {
            self.runtime.sprite_move_xy(self.ram, slot);
        }
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        let _ = self.runtime.sprite_check_damage_to_link(self.ram, slot);
        if (((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 15) == 0 {
            let direction = self
                .runtime
                .sprite_direction_to_face_link_direction(self.ram, slot);
            self.ram.set_sprite_head_direction(slot, direction);
            self.ram.set_sprite_scratch_d(slot, direction);
            self.runtime
                .sprite_apply_speed_towards_link(self.ram, slot, 18);
            self.guard_apply_speed_in_direction(slot);
        }
        self.ram
            .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
        self.guard_tick_and_update_body(slot);
    }

    pub fn bolt_guard_trigger_chase_theme(&mut self, slot: usize) {
        let counter = self.ram.sprite(slot).scratch_g();
        if counter != 16 {
            self.ram.set_sprite_scratch_g(slot, counter.wrapping_add(1));
            if counter == 15 {
                self.runtime.queue_sfx3_with_pan(self.ram, slot, 4);
                if self.ram.read_u8(SRAM_PROGRESS_INDICATOR) == 2
                    && self.ram.read_u8(OVERWORLD_AREA_INDEX) == 24
                {
                    self.ram.write_u8(MUSIC_CONTROL, 12);
                }
            }
        }
    }

    pub fn sprite_44_bluesain_bolt(&mut self, slot: usize) {
        const GRAPHICS: [u8; 32] = [
            0x10, 0x11, 0x12, 0x13, 0x10, 0x11, 0x12, 0x13, 6, 7, 8, 9, 6, 7, 8, 9, 0, 1, 2, 3, 0,
            1, 4, 5, 0xa, 0xb, 0xc, 0xd, 0xa, 0xb, 0xe, 0xf,
        ];
        self.psycho_trooper_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.bolt_guard_trigger_chase_theme(slot);
        self.runtime.guard_parry_sword_attacks(self.ram, slot);
        if self.runtime.sprite_return_if_recoiling(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).wall_collision() == 0 {
            self.runtime.sprite_move_xy(self.ram, slot);
        }
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        let _ = self.runtime.sprite_check_damage_to_link(self.ram, slot);
        if (((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 15) == 0 {
            let direction = self
                .runtime
                .sprite_direction_to_face_link_direction(self.ram, slot);
            self.ram.set_sprite_scratch_d(slot, direction);
            self.ram.set_sprite_head_direction(slot, direction);
            self.runtime
                .sprite_apply_speed_towards_link(self.ram, slot, 18);
            self.guard_apply_speed_in_direction(slot);
        }
        let phase = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, phase);
        let index =
            usize::from(phase >> 1 & 7) | (usize::from(self.ram.sprite(slot).scratch_d()) << 3);
        self.ram.set_sprite_graphics(slot, GRAPHICS[index]);
    }

    pub fn psycho_trooper_draw(&mut self, slot: usize) {
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.sprite_draw_guard_head(slot, &info, 3);
        self.sprite_draw_bnc_body(slot, &info, 2);
        self.sprite_draw_guard_spear(slot, &info, 0);
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            let direction = usize::from(self.ram.sprite(slot).scratch_d());
            self.runtime
                .draw_shadow_custom(self.ram, slot, &info, SOLDIER_DRAW_SHADOW[direction]);
        }
    }

    pub fn sprite_draw_guard_spear(
        &mut self,
        slot: usize,
        info: &PrepOamCoordsRet,
        sprite_offset: usize,
    ) {
        const X: [i8; 16] = [15, 7, 17, 9, -8, 0, -10, -2, 13, 13, 13, 13, -4, -4, -4, -4];
        const Y: [i8; 16] = [
            -2, -2, -2, -2, -2, -2, -2, -2, 8, 0, 10, 2, -14, -6, -16, -8,
        ];
        const CHARACTER: [u8; 16] = [
            0x6f, 0x7f, 0x6f, 0x7f, 0x6f, 0x7f, 0x6f, 0x7f, 0x6e, 0x7e, 0x6e, 0x7e, 0x6e, 0x7e,
            0x6e, 0x7e,
        ];
        const FLAGS: [u8; 16] = [
            0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0,
        ];
        let first_oam = self.oam_current_index() + sprite_offset;
        let base = usize::from(
            self.ram.sprite(slot).scratch_d() * 4
                + (((self.ram.sprite(slot).scratch_a() ^ 1) << 1) & 2),
        );
        for (entry, piece) in (0..2).rev().enumerate() {
            let table = base + piece;
            self.ram.write_u8(DUNGMAP_VAR8 + 1, X[table] as u8);
            self.ram.write_u8(DUNGMAP_VAR8, Y[table] as u8);
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[table] as i16 as u16),
                info.y.wrapping_add(Y[table] as i16 as u16),
                CHARACTER[table].wrapping_sub(u8::from(self.ram.sprite(slot).kind() >= 0x48) * 3),
                ((FLAGS[table] | info.flags) & 0xf1) | 8,
                0,
            );
        }
    }

    pub fn sprite_48_red_javelin_guard(&mut self, slot: usize) {
        const DIRECTION_LOCK: [u8; 4] = [3, 2, 0, 1];
        const LOCKED_GRAPHICS: [u8; 4] = [12, 0, 18, 8];
        let saved_graphics = self.ram.sprite(slot).graphics();
        let saved_direction = self.ram.sprite(slot).scratch_d();
        if self.ram.sprite(slot).aux1_delay() != 0 {
            self.ram
                .set_sprite_scratch_d(slot, DIRECTION_LOCK[usize::from(saved_direction)]);
            self.ram
                .set_sprite_graphics(slot, LOCKED_GRAPHICS[usize::from(saved_direction)]);
        }
        self.javelin_trooper_draw(slot);
        self.ram.set_sprite_scratch_d(slot, saved_direction);
        self.ram.set_sprite_graphics(slot, saved_graphics);
        self.soldier_throwing_common(slot);
    }

    pub fn sprite_46_blue_archer(&mut self, slot: usize) {
        const DIRECTION_LOCK: [u8; 4] = [3, 2, 0, 1];
        const LOCKED_GRAPHICS: [u8; 4] = [8, 0, 12, 5];
        let saved_graphics = self.ram.sprite(slot).graphics();
        let saved_direction = self.ram.sprite(slot).scratch_d();
        if self.ram.sprite(slot).aux1_delay() != 0 {
            self.ram
                .set_sprite_scratch_d(slot, DIRECTION_LOCK[usize::from(saved_direction)]);
            self.ram
                .set_sprite_graphics(slot, LOCKED_GRAPHICS[usize::from(saved_direction)]);
        }
        self.archer_soldier_draw(slot);
        self.ram.set_sprite_scratch_d(slot, saved_direction);
        self.ram.set_sprite_graphics(slot, saved_graphics);
        self.soldier_throwing_common(slot);
    }

    pub fn soldier_throwing_common(&mut self, slot: usize) {
        const DIRECTION_FLAGS: [u8; 4] = [3, 3, 12, 12];
        const TARGET_X: [i8; 8] = [-80, 80, 0, -8, -80, 80, -8, 8];
        const TARGET_Y: [i8; 8] = [8, 8, -80, 80, 8, 8, -80, 80];
        const X_VELOCITY: [i8; 4] = [8, -8, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, 8, -8];
        const HEAD_DIRECTIONS: [u8; 32] = [
            0, 2, 2, 2, 0, 3, 3, 3, 1, 3, 3, 3, 1, 2, 2, 2, 2, 0, 0, 0, 2, 1, 1, 1, 3, 1, 1, 1, 3,
            0, 0, 0,
        ];
        const ANIMATION_CLOCK: [u8; 4] = [13, 13, 12, 12];
        const ATTACK_GRAPHICS: [u8; 64] = [
            25, 25, 24, 24, 23, 23, 23, 23, 19, 19, 18, 18, 17, 17, 17, 17, 16, 16, 15, 15, 14, 14,
            14, 14, 22, 22, 21, 21, 20, 20, 20, 20, 20, 20, 18, 18, 18, 16, 16, 16, 21, 21, 8, 8,
            8, 6, 6, 6, 22, 22, 4, 4, 4, 3, 3, 3, 23, 23, 15, 15, 15, 11, 11, 11,
        ];
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if (self
            .runtime
            .sprite_check_damage_to_and_from_link_result(self.ram, slot)
            || self.ram.read_u8(SPRITE_ALERT_FLAG) != 0)
            && self.ram.sprite(slot).ai_state() < 3
        {
            self.ram.set_sprite_ai_state(slot, 3);
            self.ram.set_sprite_main_delay(slot, 32);
        }
        if self.ram.sprite(slot).scratch_f() >= 4 {
            self.ram.set_sprite_ai_state(slot, 4);
            self.ram.set_sprite_main_delay(slot, 60);
            self.ram.set_sprite_subtype2(slot, 0);
        }
        if self.runtime.sprite_return_if_recoiling(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).wall_collision() == 0 {
            self.runtime.sprite_move_xy(self.ram, slot);
        }
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    let delay =
                        0x50_u8.wrapping_add(crate::misc::get_random_number(self.ram) & 0x7f);
                    self.ram.set_sprite_main_delay(slot, delay);
                    let old = self.ram.sprite(slot).scratch_d();
                    let new_direction = crate::misc::get_random_number(self.ram) & 3;
                    self.ram.set_sprite_scratch_d(slot, new_direction);
                    if new_direction != old && (new_direction ^ old) & 2 == 0 {
                        self.ram.set_sprite_aux1_delay(slot, 12);
                    }
                }
            }
            1 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 160);
                    return;
                }
                self.sprite_guard_send_out_probe(slot);
                if self.ram.sprite(slot).wall_collision() & 0xf != 0 {
                    self.ram
                        .set_sprite_scratch_d(slot, self.ram.sprite(slot).scratch_d() ^ 1);
                    self.guard_set_glance_to12(slot);
                }
                let direction = usize::from(self.ram.sprite(slot).scratch_d());
                self.ram.set_sprite_signed_field(
                    slot,
                    SpriteField::VelocityX,
                    X_VELOCITY[direction],
                );
                self.ram.set_sprite_signed_field(
                    slot,
                    SpriteField::VelocityY,
                    Y_VELOCITY[direction],
                );
                self.ram.set_sprite_head_direction(slot, direction as u8);
                self.soldier_throwing_animate(slot);
            }
            2 => {
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                self.sprite_guard_send_out_probe(slot);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_main_delay(slot, 32);
                    self.ram.set_sprite_ai_state(slot, 0);
                } else if delay < 0x80 {
                    let index =
                        usize::from((self.ram.sprite(slot).scratch_d() * 8) | (delay >> 3 & 7));
                    self.ram
                        .set_sprite_head_direction(slot, HEAD_DIRECTIONS[index]);
                }
            }
            3 => {
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                let direction = self
                    .runtime
                    .sprite_direction_to_face_link_direction(self.ram, slot);
                self.ram.set_sprite_head_direction(slot, direction);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 4);
                    self.ram.set_sprite_main_delay(slot, 60);
                    self.ram.set_sprite_subtype2(slot, 0);
                }
            }
            4 => {
                let mut direction = usize::from(self.ram.sprite(slot).scratch_d());
                if self.ram.sprite(slot).wall_collision() & DIRECTION_FLAGS[direction] != 0
                    || self.ram.sprite(slot).main_delay() == 0
                {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.set_sprite_main_delay(slot, 24);
                    return;
                }
                if (self.ram.read_u8(FRAME_COUNTER) ^ slot as u8) & 7 == 0 {
                    let face = self
                        .runtime
                        .sprite_direction_to_face_link_direction(self.ram, slot);
                    self.ram.set_sprite_scratch_d(slot, face);
                    self.ram.set_sprite_head_direction(slot, face);
                    direction = usize::from(face);
                    if self.ram.sprite(slot).kind() == 0x48 {
                        direction += 4;
                    }
                    let target_x = self
                        .ram
                        .read_u16(LINK_X)
                        .wrapping_add(TARGET_X[direction] as i16 as u16);
                    let target_y = self
                        .ram
                        .read_u16(LINK_Y)
                        .wrapping_add(TARGET_Y[direction] as i16 as u16);
                    let speed = self.runtime.sprite_project_speed_towards_location(
                        self.ram, slot, target_x, target_y, 24,
                    );
                    self.ram.set_sprite_velocity_x(slot, speed.x);
                    self.ram.set_sprite_velocity_y(slot, speed.y);
                    if speed.xdiff.wrapping_add(6) < 12 && speed.ydiff.wrapping_add(6) < 12 {
                        self.ram.set_sprite_ai_state(
                            slot,
                            self.ram.sprite(slot).ai_state().wrapping_add(1),
                        );
                        self.ram.set_sprite_main_delay(slot, 24);
                        return;
                    }
                }
                self.ram
                    .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
                self.soldier_throwing_animate(slot);
            }
            5 => {
                let direction = usize::from(self.ram.sprite(slot).scratch_d());
                self.ram
                    .set_sprite_animation_clock(slot, ANIMATION_CLOCK[direction]);
                self.ram.set_sprite_velocity_x(slot, 0);
                self.ram.set_sprite_velocity_y(slot, 0);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 160);
                    return;
                }
                self.ram
                    .set_sprite_subtype2(slot, if delay >= 40 { 255 } else { 0 });
                if delay == 12 {
                    self.guard_launch_projectile(slot);
                }
                let table = direction * 8
                    + usize::from(delay >> 3)
                    + if self.ram.sprite(slot).kind() == 0x48 {
                        32
                    } else {
                        0
                    };
                self.ram.set_sprite_graphics(slot, ATTACK_GRAPHICS[table]);
            }
            _ => unreachable!("throwing soldier has only six AI states"),
        }
    }

    fn soldier_throwing_animate(&mut self, slot: usize) {
        const GRAPHICS: [u8; 32] = [
            11, 12, 13, 12, 4, 5, 6, 5, 0, 1, 2, 3, 7, 8, 9, 10, 17, 18, 17, 18, 7, 8, 7, 8, 3, 4,
            3, 4, 13, 14, 13, 14,
        ];
        let phase = self.ram.sprite(slot).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(slot, phase);
        if phase & 0xf == 0 {
            let frame = self.ram.sprite(slot).scratch_a().wrapping_add(1);
            self.ram
                .set_sprite_scratch_a(slot, if frame == 2 { 0 } else { frame });
        }
        let table = usize::from(self.ram.sprite(slot).scratch_d()) * 4
            + usize::from(self.ram.sprite(slot).scratch_a())
            + if self.ram.sprite(slot).kind() == 0x48 {
                16
            } else {
                0
            };
        self.ram.set_sprite_graphics(slot, GRAPHICS[table]);
    }

    pub fn guard_launch_projectile(&mut self, slot: usize) {
        const X: [i8; 8] = [16, -8, 3, 11, 12, -4, 12, -4];
        const Y: [i8; 8] = [2, 2, 16, -8, -2, -2, 2, -8];
        const X_VELOCITY: [i8; 8] = [48, -48, 0, 0, 32, -32, 0, 0];
        const Y_VELOCITY: [i8; 8] = [0, 0, 48, -48, 0, 0, 32, -32];
        const FLAGS4: [u8; 4] = [5, 5, 6, 6];
        let Some((spawned, info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0x1b)
        else {
            return;
        };
        self.runtime.queue_sfx3_with_pan(self.ram, slot, 5);
        let mut direction = usize::from(self.ram.sprite(slot).scratch_d());
        if self.ram.sprite(slot).kind() >= 0x48 {
            direction += 4;
        }
        self.set_sprite_x(spawned, info.r0_x.wrapping_add(X[direction] as i16 as u16));
        self.set_sprite_y(spawned, info.r2_y.wrapping_add(Y[direction] as i16 as u16));
        self.ram
            .set_sprite_signed_field(spawned, SpriteField::VelocityX, X_VELOCITY[direction]);
        self.ram
            .set_sprite_signed_field(spawned, SpriteField::VelocityY, Y_VELOCITY[direction]);
        direction &= 3;
        self.ram.set_sprite_scratch_d(spawned, direction as u8);
        self.ram.set_sprite_flags4(spawned, FLAGS4[direction]);
        self.ram.set_sprite_z(spawned, 0);
        let is_arrow = u8::from(self.ram.sprite(slot).kind() >= 0x48);
        self.ram.set_sprite_scratch_a(spawned, is_arrow);
        if is_arrow != 0 && self.ram.read_u8(LINK_SHIELD_TYPE) == 0 {
            self.ram
                .set_sprite_flags5(spawned, self.ram.sprite(spawned).flags5() & !0x20);
        }
    }

    pub fn bush_javelin_soldier_draw(&mut self, slot: usize) {
        let saved_graphics = self.ram.sprite(slot).graphics();
        self.ram.set_sprite_graphics(slot, 0);
        let saved_flags = self.ram.sprite(slot).oam_flags();
        self.ram
            .set_sprite_oam_flags(slot, (saved_flags & 0xf1) | 2);
        let saved_y = self.ram.read_u16(CUR_SPRITE_Y);
        self.ram.write_u16(CUR_SPRITE_Y, saved_y.wrapping_add(8));
        self.runtime.draw_single_large(self.ram, slot);
        self.ram.write_u16(CUR_SPRITE_Y, saved_y);
        self.ram.set_sprite_oam_flags(slot, saved_flags);
        self.ram.set_sprite_graphics(slot, saved_graphics);
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.guard_animate_head(slot, 4, &info);
        self.sprite_draw_bnc_body(slot, &info, 3);
        if self.ram.sprite(slot).graphics() < 20 {
            self.sprite_draw_guard_spear(slot, &info, 1);
        }
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            let direction = usize::from(self.ram.sprite(slot).scratch_d());
            self.runtime
                .draw_shadow_custom(self.ram, slot, &info, SOLDIER_DRAW_SHADOW[direction]);
        }
    }

    pub fn javelin_trooper_draw(&mut self, slot: usize) {
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.sprite_draw_guard_head(slot, &info, 3);
        self.sprite_draw_bnc_body(slot, &info, 2);
        if self.ram.sprite(slot).graphics() < 20 {
            self.sprite_draw_guard_spear(slot, &info, 0);
        }
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            let direction = usize::from(self.ram.sprite(slot).scratch_d());
            self.runtime
                .draw_shadow_custom(self.ram, slot, &info, SOLDIER_DRAW_SHADOW[direction]);
        }
    }

    pub fn sprite_49_red_bush_guard(&mut self, slot: usize) {
        let state = self.ram.sprite(slot).ai_state();
        if state != 0 {
            if state == 2 {
                self.bush_javelin_soldier_draw(slot);
            } else {
                self.bush_soldier_common_draw(slot);
            }
        }
        self.sprite_bush_guard_main(slot);
    }

    pub fn sprite_47_green_bush_guard(&mut self, slot: usize) {
        if self.ram.sprite(slot).ai_state() != 0 {
            if self.ram.sprite(slot).graphics() >= 14 {
                self.archer_soldier_draw(slot);
            } else {
                self.bush_soldier_common_draw(slot);
            }
        }
        self.sprite_bush_guard_main(slot);
    }

    pub fn sprite_bush_guard_main(&mut self, slot: usize) {
        const GRAPHICS: [u8; 32] = [
            4, 4, 4, 4, 4, 4, 4, 4, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
            1, 0, 1,
        ];
        const RETREAT_GRAPHICS: [u8; 16] = [0, 1, 0, 1, 0, 1, 0, 1, 0, 2, 3, 4, 4, 4, 4, 4];
        const ATTACK_GRAPHICS: [u8; 64] = [
            25, 25, 24, 24, 23, 23, 23, 23, 19, 19, 18, 18, 17, 17, 17, 17, 16, 16, 15, 15, 14, 14,
            14, 14, 22, 22, 21, 21, 20, 20, 20, 20, 20, 20, 18, 18, 18, 16, 16, 16, 21, 21, 8, 8,
            8, 6, 6, 6, 22, 22, 4, 4, 4, 3, 3, 3, 23, 23, 15, 15, 15, 11, 11, 11,
        ];
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.ram.set_sprite_ignore_projectile(slot, 1);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(slot, 1);
                    self.ram.set_sprite_main_delay(slot, 64);
                }
            }
            1 => {
                self.runtime.sprite_check_damage_from_link(self.ram, slot);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_main_delay(slot, 48);
                    let direction = self
                        .runtime
                        .sprite_direction_to_face_link_direction(self.ram, slot);
                    self.ram.set_sprite_head_direction(slot, direction);
                    self.ram.set_sprite_scratch_d(slot, direction);
                } else {
                    if delay == 0x20 {
                        self.bush_guard_spawn_foliage(slot);
                    }
                    self.ram
                        .set_sprite_graphics(slot, GRAPHICS[usize::from(delay >> 2)]);
                }
            }
            2 => {
                self.ram.set_sprite_ignore_projectile(slot, 0);
                self.runtime
                    .sprite_check_damage_to_and_from_link(self.ram, slot);
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(slot, 3);
                    self.ram.set_sprite_main_delay(slot, 48);
                    self.bush_guard_retreat(slot, &RETREAT_GRAPHICS);
                    return;
                }
                self.ram
                    .set_sprite_scratch_a(slot, if delay < 40 { 0xff } else { 0 });
                if delay == 16 {
                    self.guard_launch_projectile(slot);
                }
                let table = usize::from(self.ram.sprite(slot).scratch_d()) * 8
                    + usize::from(delay >> 3)
                    + if self.ram.sprite(slot).kind() == 0x49 {
                        32
                    } else {
                        0
                    };
                self.ram.set_sprite_graphics(slot, ATTACK_GRAPHICS[table]);
            }
            3 => self.bush_guard_retreat(slot, &RETREAT_GRAPHICS),
            _ => unreachable!("bush guard has only four AI states"),
        }
    }

    fn bush_guard_retreat(&mut self, slot: usize, graphics: &[u8; 16]) {
        self.runtime
            .sprite_check_damage_to_and_from_link(self.ram, slot);
        let delay = self.ram.sprite(slot).main_delay();
        if delay == 0 {
            self.ram.set_sprite_ai_state(slot, 0);
            self.ram.set_sprite_main_delay(slot, 64);
        } else {
            self.ram
                .set_sprite_graphics(slot, graphics[usize::from(delay >> 2)]);
        }
    }

    pub fn bush_guard_spawn_foliage(&mut self, slot: usize) {
        let Some((spawned, info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0xec)
        else {
            return;
        };
        self.set_spawned_coordinates(spawned, &info);
        self.ram.set_sprite_state(spawned, 6);
        self.ram.set_sprite_main_delay(spawned, 32);
        self.ram
            .set_sprite_flags2(spawned, self.ram.sprite(spawned).flags2().wrapping_add(3));
        self.ram.set_sprite_scratch_c(spawned, 2);
    }

    pub fn bush_soldier_common_draw(&mut self, slot: usize) {
        const Y: [i8; 14] = [8, 8, 8, 8, 2, 8, 0, 8, -3, 8, -3, 8, -3, 8];
        const CHARACTER: [u8; 14] = [
            0x20, 0x20, 0x20, 0x20, 0x40, 0x20, 0x40, 0x20, 0x40, 0x20, 0x42, 0x20, 0x42, 0x20,
        ];
        const FLAGS: [u8; 14] = [9, 3, 0x49, 0x43, 9, 3, 0x49, 0x43, 9, 3, 0x49, 0x43, 9, 3];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = self.oam_current_index();
        let base = usize::from(self.ram.sprite(slot).graphics()) * 2;
        for (entry, piece) in (0..2).rev().enumerate() {
            let table = base + piece;
            let mut flags = FLAGS[table] | 0x20;
            if piece == 0 {
                flags = (flags & !0xe) | info.flags;
            }
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x,
                info.y.wrapping_add(Y[table] as i16 as u16),
                CHARACTER[table],
                flags,
                2,
            );
        }
    }

    pub fn archer_soldier_draw(&mut self, slot: usize) {
        const WEAPON_OFFSET: [u8; 4] = [0, 0, 0, 16];
        const HEAD_OFFSET: [u8; 4] = [16, 16, 16, 0];
        const BODY_OFFSET: [u8; 4] = [20, 20, 20, 4];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let direction = usize::from(self.ram.sprite(slot).scratch_d());
        self.guard_animate_head(slot, usize::from(HEAD_OFFSET[direction] >> 2), &info);
        self.guard_animate_body(slot, usize::from(BODY_OFFSET[direction] >> 2), &info);
        self.sprite_draw_archer_weapon(slot, usize::from(WEAPON_OFFSET[direction] >> 2), &info);
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            self.runtime
                .draw_shadow_custom(self.ram, slot, &info, SOLDIER_DRAW_SHADOW[direction]);
        }
    }

    pub fn sprite_draw_archer_weapon(
        &mut self,
        slot: usize,
        sprite_offset: usize,
        info: &PrepOamCoordsRet,
    ) {
        const DEFAULT_FRAME: [u8; 4] = [9, 3, 0, 6];
        const X: [i8; 48] = [
            -1, 7, 3, 3, -1, 7, 3, 3, -1, 7, 7, 7, -5, -5, -10, -2, -4, -4, -6, 2, -5, -5, -5, -5,
            6, 14, 11, 11, 6, 14, 11, 11, 6, 14, 14, 14, 11, 11, 18, 10, 12, 12, 14, 6, 11, 11, 11,
            11,
        ];
        const Y: [i8; 48] = [
            7, 7, 3, 11, 6, 6, 1, 9, 7, 7, 7, 7, -2, 6, 2, 2, -2, 6, 2, 2, -2, 6, 6, 6, -6, -6,
            -12, -4, -6, -6, -9, -1, -6, -6, -6, -6, -2, 6, 2, 2, -2, 6, 2, 2, -2, 6, 6, 6,
        ];
        const CHARACTER: [u8; 48] = [
            0xa, 0xa, 0x2a, 0x2b, 0x1a, 0x1a, 0x2a, 0x2b, 0xa, 0xa, 0xa, 0xa, 0xb, 0xb, 0x3d, 0x3a,
            0x1b, 0x1b, 0x3d, 0x3a, 0xb, 0xb, 0xb, 0xb, 0xa, 0xa, 0x2b, 0x2a, 0xa, 0xa, 0x2b, 0x2a,
            0xa, 0xa, 0xa, 0xa, 0xb, 0xb, 0x3d, 0x3a, 0x1b, 0x1b, 0x3d, 0x3a, 0xb, 0xb, 0xb, 0xb,
        ];
        const FLAGS: [u8; 48] = [
            0xd, 0x4d, 8, 8, 0xd, 0x4d, 8, 8, 0xd, 0x4d, 0x4d, 0x4d, 0xd, 0x8d, 0x48, 0x48, 0xd,
            0x8d, 0x48, 0x48, 0xd, 0x8d, 0x8d, 0x8d, 0x8d, 0xcd, 0x88, 0x88, 0x8d, 0xcd, 0x88,
            0x88, 0x8d, 0xcd, 0xcd, 0xcd, 0x4d, 0xcd, 8, 8, 0x4d, 0xcd, 8, 8, 0x4d, 0xcd, 0xcd,
            0xcd,
        ];
        let mut base = i16::from(self.ram.sprite(slot).graphics()) - 14;
        if base < 0 {
            base = i16::from(DEFAULT_FRAME[usize::from(self.ram.sprite(slot).scratch_d())]);
        }
        let first_oam = self.oam_current_index() + sprite_offset;
        for (entry, piece) in (0..4).rev().enumerate() {
            let table = base as usize * 4 + piece;
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[table] as i16 as u16),
                info.y.wrapping_add(Y[table] as i16 as u16),
                CHARACTER[table],
                FLAGS[table] | 0x20,
                0,
            );
        }
    }

    pub fn tutorial_soldier_draw(&mut self, slot: usize) {
        const X: [i16; 20] = [
            4, 0, -6, -6, 2, 0, 0, -7, -7, -7, 0, 0, 15, 15, 15, 6, 14, -4, 4, 0,
        ];
        const Y: [i16; 20] = [
            0, -10, -4, 12, 12, 0, -9, -11, -3, 5, 0, -9, -11, -3, 5, -11, 5, 0, 0, -9,
        ];
        const CHARACTER: [u8; 20] = [
            0x46, 0x40, 0, 0x28, 0x29, 0x4e, 0x42, 0x39, 0x2a, 0x3a, 0x4e, 0x42, 0x39, 0x2a, 0x3a,
            0x26, 0x38, 0x64, 0x64, 0x44,
        ];
        const FLAGS: [u8; 20] = [
            0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0, 0x40, 0,
        ];
        const BIG: [u8; 20] = [2, 2, 2, 0, 0, 2, 2, 0, 0, 0, 2, 2, 0, 0, 0, 2, 0, 2, 2, 2];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        let first_oam = self.oam_current_index();
        let base = usize::from(self.ram.sprite(slot).graphics()) * 5;
        for (entry, piece) in (0..5).rev().enumerate() {
            let table = base + piece;
            let mut flags = FLAGS[table] | info.flags;
            if CHARACTER[table] < 0x40 {
                flags = (flags & 0xf1) | 8;
            }
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[table] as u16),
                info.y.wrapping_add(Y[table] as u16),
                CHARACTER[table],
                flags,
                BIG[table],
            );
        }
        self.runtime.draw_shadow_custom(self.ram, slot, &info, 12);
    }

    pub fn pull_switch_facing_up(&mut self, slot: usize) {
        const DELAY: [u8; 10] = [8, 24, 4, 4, 4, 4, 4, 4, 2, 10];
        const LINK_FRAME: [u8; 10] = [6, 7, 8, 8, 8, 8, 8, 9, 9, 9];
        self.pull_switch_handle_up_pulling(slot);
        let mut graphics = self.ram.sprite(slot).graphics();
        if graphics != 0 && graphics != 11 {
            self.ram
                .write_u8(LINK_UNK_MASTER_SWORD, LINK_FRAME[usize::from(graphics - 1)]);
            self.ram
                .write_u16(LINK_Y, self.sprite_y(slot).wrapping_sub(19));
            self.ram.write_u16(LINK_X, self.sprite_x(slot));
            if self.ram.sprite(slot).main_delay() == 0 {
                graphics = graphics.wrapping_add(1);
                self.ram.set_sprite_graphics(slot, graphics);
                if graphics == 11 {
                    self.ram.write_u8(SOUND_EFFECT_2, 0x1b);
                    self.ram.write_u8(DUNG_FLAG_STATECHANGE_WATERPUZZLE, 1);
                }
                self.ram
                    .set_sprite_main_delay(slot, DELAY[usize::from(graphics - 2)]);
            }
        }
        if self.ram.sprite(slot).kind() != 7 {
            self.bad_pull_down_switch_draw(slot);
        } else {
            self.bad_pull_up_switch_draw(slot);
        }
    }

    pub fn pull_switch_handle_up_pulling(&mut self, slot: usize) {
        if !self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            return;
        }
        self.ram.write_u8(LINK_ACTUAL_VEL_Y, 0);
        self.ram.write_u8(LINK_ACTUAL_VEL_X, 0);
        self.runtime.sprite_repel_dash(self.ram);
        self.ram.write_u8(BITMASK_OF_DRAGSTATE, 0);
        let y = (self.ram.read_u16(LINK_Y) as u8).wrapping_sub(self.ram.sprite(slot).y_low());
        if y.wrapping_sub(2) as i8 >= 0 {
            self.ram
                .write_u16(LINK_Y, self.sprite_y(slot).wrapping_add(9));
        } else if (y.wrapping_sub(244) as i8) < 0 {
            self.ram
                .write_u8(BYTE_7E0379, self.ram.read_u8(BYTE_7E0379).wrapping_add(1));
            if self.ram.read_u8(JOYPAD1_L_LAST) & 0x80 != 0
                && self.ram.read_u8(JOYPAD1_H_LAST) & 3 == 0
                && self.ram.sprite(slot).graphics() == 0
            {
                self.ram.set_sprite_graphics(slot, 1);
                self.ram.set_sprite_main_delay(slot, 8);
                self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x22);
            }
            self.ram
                .write_u16(LINK_Y, self.sprite_y(slot).wrapping_sub(21));
        } else if ((self.ram.read_u16(LINK_X) as u8).wrapping_sub(self.ram.sprite(slot).x_low())
            as i8)
            < 0
        {
            self.ram
                .write_u16(LINK_X, self.sprite_x(slot).wrapping_sub(16));
        } else {
            self.ram
                .write_u16(LINK_X, self.sprite_x(slot).wrapping_add(14));
        }
    }

    pub fn bad_pull_down_switch_draw(&mut self, slot: usize) {
        const X: [i8; 5] = [-4, 12, 0, -4, 4];
        const Y: [i8; 5] = [-3, -3, 0, 5, 5];
        const CHARACTER: [u8; 5] = [0xd2, 0xd2, 0xc4, 0xe4, 0xe4];
        const FLAGS: [u8; 5] = [0x40, 0, 0, 0x40, 0];
        const BIG: [u8; 5] = [0, 0, 2, 2, 2];
        const MAP: [u8; 12] = [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5];
        const OFFSET: [u8; 6] = [0, 1, 2, 3, 4, 5];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.runtime.oam_allocate_defer_to_player(self.ram, slot);
        let first_oam = self.oam_current_index();
        let y_offset = OFFSET[usize::from(MAP[usize::from(self.ram.sprite(slot).graphics())])];
        for (entry, piece) in (0..5).rev().enumerate() {
            crate::oam::set_plain(
                self.ram,
                first_oam + entry,
                info.x.wrapping_add(X[piece] as i16 as u16) as u8,
                info.y
                    .wrapping_add(Y[piece] as i16 as u16)
                    .wrapping_sub(if piece == 2 { u16::from(y_offset) } else { 0 })
                    as u8,
                CHARACTER[piece],
                FLAGS[piece] | 0x21,
                BIG[piece],
            );
        }
        self.runtime
            .sprite_correct_oam_entries(self.ram, slot, 4, 0xff);
    }

    pub fn bad_pull_up_switch_draw(&mut self, slot: usize) {
        const CHARACTER: [u8; 2] = [0xa2, 0xa4];
        const MAP: [u8; 12] = [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5];
        const OFFSET: [u8; 6] = [0, 1, 2, 3, 4, 5];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.runtime.oam_allocate_defer_to_player(self.ram, slot);
        let first_oam = self.oam_current_index();
        let y_offset = OFFSET[usize::from(MAP[usize::from(self.ram.sprite(slot).graphics())])];
        for (entry, piece) in (0..2).rev().enumerate() {
            crate::oam::set_helper0(
                self.ram,
                first_oam + entry,
                info.x,
                info.y
                    .wrapping_sub(if piece == 0 { u16::from(y_offset) } else { 0 }),
                CHARACTER[piece],
                info.flags,
                2,
            );
        }
    }

    pub fn pull_switch_facing_down(&mut self, slot: usize) {
        const DELAY: [u8; 12] = [5; 12];
        const LINK_FRAME: [u8; 12] = [1, 1, 2, 2, 3, 3, 1, 1, 4, 4, 5, 5];
        const Y_OFFSET: [u8; 12] = [9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14];
        self.pull_switch_handle_down_pulling(slot);
        let mut graphics = self.ram.sprite(slot).graphics();
        if graphics != 0 && graphics != 13 {
            let index = usize::from(graphics - 1);
            self.ram.write_u8(LINK_UNK_MASTER_SWORD, LINK_FRAME[index]);
            self.ram.write_u16(
                LINK_Y,
                self.sprite_y(slot).wrapping_add(u16::from(Y_OFFSET[index])),
            );
            self.ram.write_u16(LINK_X, self.sprite_x(slot));
            if self.ram.sprite(slot).main_delay() == 0 {
                graphics = graphics.wrapping_add(1);
                self.ram.set_sprite_graphics(slot, graphics);
                if graphics == 13 {
                    if self.ram.sprite(slot).kind() == 6 {
                        self.ram.write_u8(ACTIVATE_BOMB_TRAP_OVERLORD, 1);
                        self.ram.write_u8(SOUND_EFFECT_1, 0x3c);
                    } else {
                        self.ram.write_u8(DUNG_FLAG_STATECHANGE_WATERPUZZLE, 1);
                        self.ram.write_u8(SOUND_EFFECT_2, 0x1b);
                    }
                }
                self.ram
                    .set_sprite_main_delay(slot, DELAY[usize::from(graphics - 2)]);
            }
        }
        self.good_pull_switch_draw(slot);
        if self.ram.sprite(slot).pause() != 0 {
            self.ram.set_sprite_graphics(slot, 0);
        }
    }

    pub fn good_pull_switch_draw(&mut self, slot: usize) {
        const OFFSET: [u8; 14] = [1, 1, 2, 3, 2, 3, 4, 5, 6, 7, 6, 7, 7, 7];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.runtime.oam_allocate_defer_to_player(self.ram, slot);
        let first_oam = self.oam_current_index();
        let offset = OFFSET[usize::from(self.ram.sprite(slot).graphics())];
        crate::oam::write(
            self.ram,
            first_oam,
            z3_core::c_types::OamEnt {
                x: info.x as u8,
                y: info.y.wrapping_sub(1) as u8,
                charnum: 0xee,
                flags: info.flags,
            },
        );
        crate::oam::write(
            self.ram,
            first_oam + 1,
            z3_core::c_types::OamEnt {
                x: info.x as u8,
                y: info.y.wrapping_sub(1).wrapping_add(u16::from(offset)) as u8,
                charnum: 0xce,
                flags: info.flags,
            },
        );
        self.runtime
            .sprite_correct_oam_entries(self.ram, slot, 1, 2);
    }

    pub fn pull_switch_handle_down_pulling(&mut self, slot: usize) {
        if !self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            return;
        }
        self.ram.write_u8(LINK_ACTUAL_VEL_Y, 0);
        self.ram.write_u8(LINK_ACTUAL_VEL_X, 0);
        self.runtime.sprite_repel_dash(self.ram);
        self.ram.write_u8(BITMASK_OF_DRAGSTATE, 0);
        let y = (self.ram.read_u16(LINK_Y) as u8).wrapping_sub(self.ram.sprite(slot).y_low());
        if y.wrapping_sub(2) as i8 >= 0 {
            self.ram
                .write_u8(BYTE_7E0379, self.ram.read_u8(BYTE_7E0379).wrapping_add(1));
            if self.ram.read_u8(JOYPAD1_L_LAST) & 0x80 != 0
                && self.ram.read_u8(JOYPAD1_H_LAST) & 3 == 0
            {
                self.ram.write_u8(
                    LINK_UNK_MASTER_SWORD,
                    self.ram.read_u8(LINK_UNK_MASTER_SWORD).wrapping_add(1),
                );
                if self.ram.read_u8(JOYPAD1_H_LAST) & 4 != 0
                    && self.ram.sprite(slot).graphics() == 0
                {
                    self.ram.set_sprite_graphics(slot, 1);
                    self.ram.set_sprite_main_delay(slot, 12);
                    self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x22);
                }
            }
            self.ram
                .write_u16(LINK_Y, self.sprite_y(slot).wrapping_add(9));
        } else if (y.wrapping_sub(244) as i8) < 0 {
            self.ram
                .write_u16(LINK_Y, self.sprite_y(slot).wrapping_sub(21));
        } else if ((self.ram.read_u16(LINK_X) as u8).wrapping_sub(self.ram.sprite(slot).x_low())
            as i8)
            < 0
        {
            self.ram
                .write_u16(LINK_X, self.sprite_x(slot).wrapping_sub(16));
        } else {
            self.ram
                .write_u16(LINK_X, self.sprite_x(slot).wrapping_add(14));
        }
    }

    pub fn priest_spawn_mantle(&mut self, slot: usize) {
        self.ram
            .set_sprite_state(15, self.ram.sprite(15).state().wrapping_add(1));
        let spawned = self
            .runtime
            .sprite_spawn_dynamically(self.ram, slot, 0x73)
            .map(|(spawned, _)| spawned);
        self.ram.set_sprite_state(15, 0);
        let Some(spawned) = spawned else { return };
        self.ram
            .set_sprite_flags2(spawned, (self.ram.sprite(spawned).flags2() & 0xf0) | 3);
        self.ram.set_sprite_x_low(spawned, 0xf0);
        self.ram.set_sprite_x_high(spawned, 4);
        self.ram.set_sprite_y_low(spawned, 0x37);
        self.ram.set_sprite_y_high(spawned, 2);
        self.ram.set_sprite_scratch_e(spawned, 2);
        self.ram.set_sprite_flags4(spawned, 11);
        self.ram
            .set_sprite_deflection_bits(spawned, self.ram.sprite(spawned).deflection_bits() | 0x20);
        self.ram.set_sprite_subtype2(spawned, 1);
        if self.ram.read_u16(LINK_Y) < self.sprite_y(spawned) {
            self.ram.set_sprite_scratch_c(spawned, 1);
        }
    }

    pub fn sprite_sanctuary_mantle(&mut self, slot: usize) {
        self.sage_mantle_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).scratch_c() != 0 {
            self.ram.set_sprite_scratch_a(slot, 0x40);
            self.sanctuary_mantle_state(slot);
            return;
        }
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            self.runtime.sprite_nullify_hookshot_drag(self.ram);
            self.ram.write_u8(LINK_SPEED_SETTING, 0);
            self.runtime.sprite_repel_dash(self.ram);
            self.ram.set_sprite_aux1_delay(slot, 7);
            self.sanctuary_mantle_drag(slot);
            self.sanctuary_mantle_state(slot);
        } else if self.ram.sprite(slot).aux1_delay() != 0 {
            self.sanctuary_mantle_drag(slot);
        } else if self.ram.sprite(slot).subtype2() == 0 {
            self.ram.set_sprite_scratch_a(slot, 0);
            self.ram.write_u8(BITMASK_OF_DRAGSTATE, 0);
            self.ram.write_u8(LINK_SPEED_SETTING, 0);
            self.ram
                .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
        }
    }

    fn sanctuary_mantle_drag(&mut self, slot: usize) {
        self.ram.set_sprite_subtype2(slot, 0);
        self.ram.write_u8(BITMASK_OF_DRAGSTATE, 0x81);
        self.ram.write_u8(LINK_SPEED_SETTING, 8);
    }

    fn sanctuary_mantle_state(&mut self, slot: usize) {
        match self.ram.sprite(slot).ai_state() {
            0 => {
                let x = self.sprite_x(slot);
                self.set_sprite_x(slot, x.wrapping_add(19));
                let direction = self
                    .runtime
                    .sprite_direction_to_face_link_direction(self.ram, slot);
                self.set_sprite_x(slot, x);
                if direction == 1 || direction == 3 {
                    let count = self.ram.sprite(slot).scratch_a().wrapping_add(1);
                    self.ram.set_sprite_scratch_a(slot, count);
                    if count >= 64 {
                        self.ram.set_sprite_ai_state(
                            slot,
                            self.ram.sprite(slot).ai_state().wrapping_add(1),
                        );
                        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                    }
                }
            }
            1 => {
                self.runtime.queue_sfx3_with_pan(self.ram, slot, 24);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                self.ram.set_sprite_main_delay(slot, 168);
                self.ram.set_sprite_velocity_x(slot, 3);
                self.ram.set_sprite_aux1_delay(slot, 2);
            }
            2 => {
                self.runtime.sprite_move_xy(self.ram, slot);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                    self.ram.set_sprite_velocity_x(slot, 0);
                    self.ram.set_sprite_scratch_c(slot, 0);
                } else {
                    self.ram.set_sprite_aux1_delay(slot, 2);
                }
            }
            _ => unreachable!("sanctuary mantle has only three AI states"),
        }
    }

    pub fn sage_mantle_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 4] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x162c,
                ext: 2,
            },
            DrawMultipleData {
                x: 16,
                y: 0,
                char_flags: 0x562c,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 16,
                char_flags: 0x062e,
                ext: 2,
            },
            DrawMultipleData {
                x: 16,
                y: 16,
                char_flags: 0x462e,
                ext: 2,
            },
        ];
        if self.ram.sprite(slot).scratch_c() == 0 {
            self.runtime.oam_allocate_from_region_b(self.ram, 0x10);
        }
        self.runtime.draw_multiple(self.ram, slot, &PIECES);
    }

    pub fn sprite_priest(&mut self, slot: usize) {
        if self.ram.sprite(slot).scratch_a() == 0 {
            self.runtime.priest_draw(self.ram, slot);
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
        if self.runtime.sprite_track_body_to_head(self.ram, slot) {
            self.runtime.sprite_move_xy(self.ram, slot);
        }
        match self.ram.sprite(slot).subtype2() {
            0 => self.priest_dying(slot),
            1 => self.priest_run_rescue_cutscene(slot),
            2 => self.priest_chillin(slot),
            _ => unreachable!("priest has only three subtypes"),
        }
    }

    pub fn priest_dying(&mut self, slot: usize) {
        self.ram.set_sprite_head_direction(slot, 4);
        self.ram.set_sprite_scratch_d(slot, 4);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.runtime.show_solicited_message(self.ram, slot, 0x1b) & 0x100 != 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.set_sprite_graphics(
                        slot,
                        self.ram.sprite(slot).graphics().wrapping_add(1),
                    );
                    self.ram.write_u8(
                        SRAM_PROGRESS_FLAGS,
                        self.ram.read_u8(SRAM_PROGRESS_FLAGS) | 2,
                    );
                    self.ram.set_sprite_aux2_delay(slot, 128);
                }
            }
            1 => {
                self.ram.set_sprite_graphics(slot, 0);
                let delay = self.ram.sprite(slot).aux2_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                }
                self.ram
                    .set_sprite_scratch_a(slot, self.ram.read_u8(FRAME_COUNTER) & 2);
                if delay & 7 == 0 {
                    self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x33);
                }
            }
            2 => self.ram.set_sprite_state(slot, 0),
            _ => unreachable!("dying priest has only three AI states"),
        }
    }

    pub fn priest_run_rescue_cutscene(&mut self, slot: usize) {
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.set_sprite_head_direction(slot, 0);
                self.ram.set_sprite_scratch_d(slot, 0);
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.runtime.show_message_unconditional(self.ram, 0x17);
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.write_u8(BYTE_7FFE01, 1);
                    self.priest_spawn_rescued_princess();
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                    self.ram.write_u8(SAVEGAME_MAP_ICONS_INDICATOR, 1);
                }
            }
            1 => {
                if self.ram.read_u8(BYTE_7FFE01) == 2 {
                    self.runtime.show_message_unconditional(self.ram, 0x18);
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                }
            }
            2 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                } else {
                    self.ram.set_sprite_ai_state(slot, 1);
                }
            }
            3 => {
                let direction = self
                    .runtime
                    .sprite_direction_to_face_link_direction(self.ram, slot)
                    ^ 3;
                self.ram.set_sprite_head_direction(slot, direction);
                let message = self.runtime.show_solicited_message(self.ram, slot, 0x16);
                if message & 0x100 != 0 {
                    self.ram.set_sprite_scratch_d(slot, message as u8);
                    self.ram.set_sprite_head_direction(slot, message as u8);
                }
            }
            _ => unreachable!("rescue priest has only four AI states"),
        }
    }

    pub fn priest_chillin(&mut self, slot: usize) {
        let direction = self
            .runtime
            .sprite_direction_to_face_link_direction(self.ram, slot)
            ^ 3;
        self.ram.set_sprite_head_direction(slot, direction);
        let message_id = if self.ram.read_u8(LINK_WHICH_PENDANTS) & 7 == 7 {
            0x1a
        } else if self.ram.read_u8(SAVEGAME_MAP_ICONS_INDICATOR) >= 3 {
            0x19
        } else {
            0x16
        };
        let message = self
            .runtime
            .show_solicited_message(self.ram, slot, message_id);
        if message & 0x100 != 0 {
            self.ram.set_sprite_scratch_d(slot, message as u8);
            self.ram.set_sprite_head_direction(slot, message as u8);
            self.ram.write_u8(LINK_HEARTS_FILLER, 0xa0);
        }
    }

    pub fn priest_spawn_rescued_princess(&mut self) {
        let Some((spawned, _)) = self.runtime.sprite_spawn_dynamically(self.ram, 0, 0x76) else {
            return;
        };
        let layer = self
            .ram
            .read_u8(TAGALONG_LAYERBITS + usize::from(self.ram.read_u8(TAGALONG_VAR2)))
            & 3;
        self.ram.set_sprite_scratch_d(spawned, layer);
        self.ram.set_sprite_head_direction(spawned, layer);
        self.set_sprite_x(spawned, self.ram.read_u16(LINK_X));
        self.set_sprite_y(spawned, self.ram.read_u16(LINK_Y));
        self.ram.set_sprite_subtype2(spawned, 1);
        self.ram.write_u8(FOLLOWER_INDICATOR, 0);
        self.ram.set_sprite_ignore_projectile(
            spawned,
            self.ram.sprite(spawned).ignore_projectile().wrapping_add(1),
        );
        self.ram.set_sprite_flags4(spawned, 3);
    }

    pub fn sprite_uncle(&mut self, slot: usize) {
        self.runtime.uncle_draw(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).subtype2() == 0 {
            self.uncle_at_house(slot);
        } else {
            self.uncle_in_passage(slot);
        }
    }

    pub fn uncle_at_house(&mut self, slot: usize) {
        const DELAY: [u8; 2] = [64, 224];
        const DIRECTION: [u8; 2] = [2, 1];
        const X_VELOCITY: [i8; 4] = [0, 0, -12, 12];
        const Y_VELOCITY: [i8; 4] = [-12, 12, 0, 0];
        self.runtime.sprite_move_xy(self.ram, slot);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                self.ram.write_u16(LINK_X_PREVIOUS, 0x940);
                self.ram.write_u16(LINK_Y_PREVIOUS, 0x215a);
                self.runtime.show_message_unconditional(self.ram, 0x1f);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
            }
            1 => {
                if self.ram.read_u8(FRAME_COUNTER) & 3 != 0 {
                    return;
                }
                if self.ram.read_u8(COLDATA_COPY0) != 32 {
                    self.ram.write_u8(
                        COLDATA_COPY0,
                        self.ram.read_u8(COLDATA_COPY0).wrapping_sub(1),
                    );
                    self.ram.write_u8(
                        COLDATA_COPY1,
                        self.ram.read_u8(COLDATA_COPY1).wrapping_sub(1),
                    );
                    return;
                }
                self.ram.write_u8(
                    LINK_POSE_DURING_OPENING,
                    self.ram.read_u8(LINK_POSE_DURING_OPENING).wrapping_add(1),
                );
                self.ram.write_u8(
                    PLAYER_SLEEP_IN_BED_STATE,
                    self.ram.read_u8(PLAYER_SLEEP_IN_BED_STATE).wrapping_add(1),
                );
                self.ram.write_u16(LINK_Y, 0x2157);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
            }
            2 => {
                self.runtime.show_message_unconditional(self.ram, 0x0d);
                self.ram.write_u8(MUSIC_CONTROL, 3);
                self.ram.set_sprite_graphics(slot, 1);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
            }
            3 => {
                self.ram
                    .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
                if self.ram.sprite(slot).main_delay() == 0 {
                    let phase = usize::from(self.ram.sprite(slot).scratch_a());
                    if phase == 2 {
                        self.ram.set_sprite_ai_state(
                            slot,
                            self.ram.sprite(slot).ai_state().wrapping_add(1),
                        );
                    } else {
                        self.ram.set_sprite_scratch_a(slot, phase as u8 + 1);
                        if phase == 0 {
                            self.ram.set_sprite_y_low(
                                slot,
                                self.ram.sprite(slot).y_low().wrapping_sub(2),
                            );
                        }
                        self.ram.set_sprite_main_delay(slot, DELAY[phase]);
                        let direction = usize::from(DIRECTION[phase]);
                        self.ram.set_sprite_scratch_d(slot, direction as u8);
                        self.ram.set_sprite_signed_field(
                            slot,
                            SpriteField::VelocityX,
                            X_VELOCITY[direction],
                        );
                        self.ram.set_sprite_signed_field(
                            slot,
                            SpriteField::VelocityY,
                            Y_VELOCITY[direction],
                        );
                    }
                }
            }
            4 => {
                self.ram.write_u8(FOLLOWER_INDICATOR, 5);
                self.ram.write_u16(WORD_7E02CD, 0xdf3);
                self.ram.write_u8(
                    SRAM_PROGRESS_FLAGS,
                    self.ram.read_u8(SRAM_PROGRESS_FLAGS) | 0x10,
                );
                self.ram.set_sprite_state(slot, 0);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
            }
            _ => unreachable!("uncle at house has only five AI states"),
        }
    }

    pub fn uncle_in_passage(&mut self, slot: usize) {
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self
                    .runtime
                    .sprite_check_damage_to_link_same_layer(self.ram, slot)
                {
                    self.runtime.link_cancel_dash(self.ram);
                }
                if self.runtime.show_message_on_contact(self.ram, slot, 0xe) & 0x100 != 0 {
                    self.ram.write_u8(FOLLOWER_INDICATOR, 0);
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                }
            }
            1 => {
                self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
                self.runtime.link_receive_item(self.ram, 0, 0);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                self.ram.set_sprite_graphics(slot, 1);
                self.ram.write_u8(WHICH_STARTING_POINT, 3);
                self.ram.write_u8(
                    SRAM_PROGRESS_FLAGS,
                    self.ram.read_u8(SRAM_PROGRESS_FLAGS) | 1,
                );
                self.ram.write_u8(SRAM_PROGRESS_INDICATOR, 1);
            }
            _ => {}
        }
    }

    pub fn sprite_quarrel_bros(&mut self, slot: usize) {
        self.quarrel_bros_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        let _ = self.runtime.sprite_track_body_to_head(self.ram, slot);
        let direction = self
            .runtime
            .sprite_direction_to_face_link_direction(self.ram, slot)
            ^ 3;
        self.ram.set_sprite_head_direction(slot, direction);
        let message = if self.ram.read_u16(DUNGEON_ROOM_INDEX) & 1 == 0 {
            0x131
        } else if self.ram.read_u16(DUNG_DOOR_OPENED) & 0xff00 == 0 {
            0x12f
        } else {
            0x130
        };
        let _ = self.runtime.show_solicited_message(self.ram, slot, message);
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
    }

    pub fn quarrel_bros_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 16] = [
            DrawMultipleData {
                x: 0,
                y: -12,
                char_flags: 0x0004,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x000a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -11,
                char_flags: 0x0004,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x400a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -12,
                char_flags: 0x0004,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x000a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -11,
                char_flags: 0x0004,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x400a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -12,
                char_flags: 0x0008,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x000a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -11,
                char_flags: 0x0008,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x400a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -12,
                char_flags: 0x4008,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x000a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -11,
                char_flags: 0x4008,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x400a,
                ext: 2,
            },
        ];
        let start = usize::from(self.ram.sprite(slot).graphics()) * 2
            + usize::from(self.ram.sprite(slot).scratch_d()) * 4;
        let info = self.runtime.draw_multiple_player_deferred_slice(
            self.ram,
            slot,
            &PIECES[start..start + 2],
        );
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_young_snitch_lady(&mut self, slot: usize) {
        self.sprite_old_snitch_lady(slot);
    }

    pub fn young_snitch_lady_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 16] = [
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x0026,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x00e8,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x0026,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x40e8,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x0024,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x00c2,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x0024,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x40c2,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x0028,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x00e4,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x0028,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x00e6,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x4028,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x40e4,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x4028,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x40e6,
                ext: 2,
            },
        ];
        let start = usize::from(self.ram.sprite(slot).graphics()) * 2
            + usize::from(self.ram.sprite(slot).scratch_d()) * 4;
        let info = self.runtime.draw_multiple_player_deferred_slice(
            self.ram,
            slot,
            &PIECES[start..start + 2],
        );
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_prep_snitches(&mut self, slot: usize) {
        self.ram.set_sprite_scratch_d(slot, 2);
        self.ram.set_sprite_head_direction(slot, 2);
        self.ram.set_sprite_ignore_projectile(
            slot,
            self.ram.sprite(slot).ignore_projectile().wrapping_add(1),
        );
        self.ram
            .set_sprite_scratch_a(slot, self.ram.sprite(slot).x_low());
        self.ram
            .set_sprite_scratch_b(slot, self.ram.sprite(slot).x_high());
        self.ram
            .set_sprite_signed_field(slot, SpriteField::VelocityX, -9);
    }

    pub fn sprite_old_snitch_lady(&mut self, slot: usize) {
        const X_OFFSET: [i8; 2] = [-32, 32];
        const X_VELOCITY: [i8; 4] = [0, 0, -9, 9];
        const Y_VELOCITY: [i8; 4] = [-9, 9, 0, 0];
        if self.ram.sprite(slot).kind() == 0x34 {
            if self.ram.sprite(slot).ai_state() < 2 {
                self.young_snitch_lady_draw(slot);
            }
        } else {
            if self.ram.sprite(slot).subtype() != 0 {
                self.runtime.sprite_chicken_lady(self.ram, slot);
                return;
            }
            if self.ram.sprite(slot).ai_state() < 3 {
                self.runtime.lady_draw(self.ram, slot);
            }
        }
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).ai_state() < 3 {
            if self.ram.read_u8(PLAYER_IS_INDOORS) != 0 {
                let _ = self.runtime.sprite_track_body_to_head(self.ram, slot);
                let direction = self
                    .runtime
                    .sprite_direction_to_face_link_direction(self.ram, slot)
                    ^ 3;
                self.ram.set_sprite_head_direction(slot, direction);
                let _ = self.runtime.show_solicited_message(self.ram, slot, 0xad);
                return;
            }
            if self.ram.sprite(slot).ai_state() == 0
                && self
                    .runtime
                    .sprite_check_damage_to_link_same_layer(self.ram, slot)
            {
                let direction = self
                    .runtime
                    .sprite_direction_to_face_link_direction(self.ram, slot)
                    ^ 3;
                self.ram.set_sprite_scratch_d(slot, direction);
                self.ram.set_sprite_main_delay(slot, 1);
            } else if self.runtime.sprite_track_body_to_head(self.ram, slot) {
                self.runtime.sprite_move_xy(self.ram, slot);
            } else {
                self.ram.set_sprite_main_delay(slot, 1);
            }
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    let home = u16::from(self.ram.sprite(slot).scratch_a())
                        | (u16::from(self.ram.sprite(slot).scratch_b()) << 8);
                    let side = usize::from(self.ram.sprite(slot).scratch_c());
                    if home.wrapping_add(X_OFFSET[side] as i16 as u16) == self.sprite_x(slot) {
                        let direction = usize::from(self.ram.sprite(slot).scratch_d() ^ 1);
                        self.ram.set_sprite_head_direction(slot, direction as u8);
                        self.ram.set_sprite_signed_field(
                            slot,
                            SpriteField::VelocityX,
                            X_VELOCITY[direction],
                        );
                        self.ram.set_sprite_signed_field(
                            slot,
                            SpriteField::VelocityY,
                            Y_VELOCITY[direction],
                        );
                        self.ram.set_sprite_scratch_c(slot, side as u8 ^ 1);
                    }
                }
                self.ram.set_sprite_graphics(
                    slot,
                    ((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) >> 4 & 1,
                );
                let saved_flags = self.ram.sprite(slot).flags4();
                self.ram.set_sprite_flags4(slot, 3);
                let message = self.runtime.show_message_on_contact(self.ram, slot, 0x2f);
                self.ram.set_sprite_flags4(slot, saved_flags);
                if message & 0x100 != 0 {
                    self.ram.set_sprite_scratch_d(slot, message as u8);
                    self.runtime.snitch_spawn_guard(self.ram, slot);
                    self.ram.set_sprite_ai_state(slot, 1);
                }
            }
            1 => {
                let overlord = usize::from(self.ram.read_u8(BYTE_7E0FDE));
                let x = u16::from(self.ram.read_u8(OVERLORD_X_LO + overlord))
                    | (u16::from(self.ram.read_u8(OVERLORD_X_HI + overlord)) << 8);
                let y = u16::from(self.ram.read_u8(OVERLORD_Y_LO + overlord))
                    | (u16::from(self.ram.read_u8(OVERLORD_Y_HI + overlord)) << 8);
                if y >= self.sprite_y(slot) {
                    self.ram.set_sprite_ai_state(slot, 2);
                    self.ram.set_sprite_velocity_x(slot, 0);
                    self.ram.set_sprite_velocity_y(slot, 0);
                    self.ram.set_sprite_flags4(slot, 2);
                    let position = y.wrapping_sub(self.ram.read_u16(OVERWORLD_OFFSET_BASE_Y))
                        & self.ram.read_u16(OVERWORLD_OFFSET_MASK_Y);
                    let position = position.wrapping_mul(8).wrapping_add(
                        (x >> 3).wrapping_sub(self.ram.read_u16(OVERWORLD_OFFSET_BASE_X))
                            & self.ram.read_u16(OVERWORLD_OFFSET_MASK_X),
                    );
                    self.runtime
                        .overworld_draw_wooden_door(self.ram, position, false);
                    self.ram.set_sprite_main_delay(slot, 16);
                } else {
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                    let speed = self
                        .runtime
                        .sprite_project_speed_towards_location(self.ram, slot, x, y, 64);
                    self.ram.set_sprite_velocity_x(slot, speed.x);
                    self.ram.set_sprite_velocity_y(slot, speed.y);
                    self.ram.set_sprite_scratch_d(slot, 0);
                    self.ram.set_sprite_head_direction(slot, 0);
                    self.ram.set_sprite_graphics(
                        slot,
                        ((slot as u8) ^ self.ram.read_u8(FRAME_COUNTER)) >> 3 & 1,
                    );
                }
            }
            2 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    let overlord = usize::from(self.ram.read_u8(BYTE_7E0FDE));
                    let x = u16::from(self.ram.read_u8(OVERLORD_X_LO + overlord))
                        | (u16::from(self.ram.read_u8(OVERLORD_X_HI + overlord)) << 8);
                    let y = u16::from(self.ram.read_u8(OVERLORD_Y_LO + overlord))
                        | (u16::from(self.ram.read_u8(OVERLORD_Y_HI + overlord)) << 8);
                    self.set_sprite_x(slot, x);
                    self.set_sprite_y(slot, y);
                    let position = y.wrapping_sub(self.ram.read_u16(OVERWORLD_OFFSET_BASE_Y))
                        & self.ram.read_u16(OVERWORLD_OFFSET_MASK_Y);
                    let position = position.wrapping_mul(8).wrapping_add(
                        (x >> 3).wrapping_sub(self.ram.read_u16(OVERWORLD_OFFSET_BASE_X))
                            & self.ram.read_u16(OVERWORLD_OFFSET_MASK_X),
                    );
                    self.runtime
                        .overworld_draw_wooden_door(self.ram, position, true);
                    self.ram.set_sprite_ai_state(slot, 3);
                }
                self.runtime.sprite_move_xy(self.ram, slot);
            }
            3 => {
                self.ram.set_sprite_state(slot, 0);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
            }
            _ => unreachable!("snitch lady has only four AI states"),
        }
    }

    pub fn sprite_prep_running_man(&mut self, slot: usize) {
        self.ram.set_sprite_head_direction(slot, 2);
        self.ram.set_sprite_scratch_d(slot, 2);
        self.ram.set_sprite_ignore_projectile(
            slot,
            self.ram.sprite(slot).ignore_projectile().wrapping_add(1),
        );
    }

    pub fn sprite_running_man(&mut self, slot: usize) {
        const START_X_VELOCITY: [i8; 2] = [-24, 24];
        const X_VELOCITY: [i8; 4] = [0, 0, -54, 54];
        const Y_VELOCITY: [i8; 4] = [-54, 54, 0, 0];
        const DIRECTIONS: [i8; 4] = [3, 1, 3, -1];
        const DELAYS: [u8; 4] = [120, 24, 128, 3];
        self.running_man_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        let _ = self.runtime.sprite_track_body_to_head(self.ram, slot);
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
        self.ram.set_sprite_subtype(slot, 255);
        let _ = self.runtime.sprite_check_tile_collision(self.ram, slot);
        let saved_flags = self.ram.sprite(slot).flags4();
        self.ram.set_sprite_flags4(slot, 7);
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            self.ram
                .set_sprite_scratch_c(slot, self.ram.sprite(slot).ai_state());
            self.ram.set_sprite_ai_state(slot, 3);
        }
        self.ram.set_sprite_flags4(slot, saved_flags);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                let _ = self.runtime.sprite_track_body_to_head(self.ram, slot);
                let face = self
                    .runtime
                    .sprite_direction_to_face_link_direction(self.ram, slot);
                self.ram.set_sprite_head_direction(slot, face ^ 3);
                if self
                    .runtime
                    .sprite_check_damage_to_link_same_layer(self.ram, slot)
                {
                    self.runtime.link_cancel_dash(self.ram);
                    self.ram.set_sprite_scratch_d(slot, face ^ 3);
                    self.ram.set_sprite_head_direction(slot, face | 2);
                    self.ram.set_sprite_ai_state(slot, (face & 1) + 1);
                    self.ram.set_sprite_signed_field(
                        slot,
                        SpriteField::VelocityX,
                        START_X_VELOCITY[usize::from(face & 1)],
                    );
                    self.ram.set_sprite_main_delay(slot, 32);
                } else {
                    self.ram.set_sprite_velocity_x(slot, 0);
                    self.ram.set_sprite_velocity_y(slot, 0);
                }
            }
            1 | 2 => {
                if self.ram.sprite(slot).main_delay() != 0 {
                    self.ram
                        .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
                    self.runtime.sprite_move_xy(self.ram, slot);
                } else {
                    self.runtime.running_boy_spawn_dust_garnish(self.ram, slot);
                    self.ram
                        .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 2 & 1);
                    let direction = usize::from(self.ram.sprite(slot).head_direction());
                    self.ram.set_sprite_signed_field(
                        slot,
                        SpriteField::VelocityX,
                        X_VELOCITY[direction],
                    );
                    self.ram.set_sprite_signed_field(
                        slot,
                        SpriteField::VelocityY,
                        Y_VELOCITY[direction],
                    );
                    self.runtime.sprite_move_xy(self.ram, slot);
                    let remaining = self.ram.sprite(slot).scratch_a();
                    if remaining != 0 {
                        self.ram.set_sprite_scratch_a(slot, remaining - 1);
                        return;
                    }
                    if self.ram.sprite(slot).ai_state() == 1 {
                        self.ram.set_sprite_scratch_a(slot, 255);
                        self.ram.set_sprite_head_direction(slot, 2);
                    } else {
                        let phase = usize::from(self.ram.sprite(slot).scratch_b());
                        self.ram.set_sprite_scratch_b(slot, phase as u8 + 1);
                        self.ram.set_sprite_scratch_a(slot, DELAYS[phase]);
                        if DIRECTIONS[phase] < 0 {
                            self.ram.set_sprite_ai_state(slot, 0);
                            self.ram.set_sprite_subtype2(slot, 0);
                        } else {
                            self.ram
                                .set_sprite_head_direction(slot, DIRECTIONS[phase] as u8);
                        }
                    }
                }
            }
            3 => {
                self.runtime.show_message_unconditional(self.ram, 0xa6);
                let handler = self.ram.read_u8(LINK_PLAYER_HANDLER_STATE);
                if handler >= 2 {
                    self.ram.set_sprite_scratch_d(slot, handler);
                }
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).scratch_c());
            }
            _ => unreachable!("running man has only four AI states"),
        }
    }

    pub fn running_man_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 16] = [
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x002c,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x08ee,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x002c,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x48ee,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x002a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x08ca,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x002a,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x48ca,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x002e,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x08cc,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x002e,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x08ce,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x402e,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x48cc,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x402e,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 1,
                char_flags: 0x48ce,
                ext: 2,
            },
        ];
        let start = (usize::from(self.ram.sprite(slot).scratch_d()) * 4
            + usize::from(self.ram.sprite(slot).graphics()) * 2)
            & 0xf;
        let info = self.runtime.draw_multiple_player_deferred_slice(
            self.ram,
            slot,
            &PIECES[start..start + 2],
        );
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_bottle_vendor(&mut self, slot: usize) {
        let visible = self.bottle_vendor_draw(slot);
        self.ram.set_sprite_scratch_a(slot, visible);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.bottle_merchant_detect_fish(self.ram, slot);
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
        if self.runtime.sprite_check_if_link_is_busy(self.ram) {
            return;
        }
        if crate::misc::get_random_number(self.ram) == 0 {
            self.ram.set_sprite_main_delay(slot, 20);
            self.ram.set_sprite_graphics(slot, 1);
        } else if self.ram.sprite(slot).main_delay() == 0 {
            self.ram.set_sprite_graphics(slot, 0);
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.ram.sprite(slot).scratch_a() == 0 && self.ram.sprite(slot).scratch_e() != 0
                {
                    self.ram.set_sprite_ai_state(slot, 3);
                } else if self.ram.read_u8(SRAM_PROGRESS_INDICATOR_3) & 2 != 0 {
                    let _ = self.runtime.show_solicited_message(self.ram, slot, 0xd4);
                } else if self.runtime.show_solicited_message(self.ram, slot, 0xd1) & 0x100 != 0 {
                    self.ram.set_sprite_ai_state(slot, 1);
                }
            }
            1 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0
                    && self.ram.read_u16(LINK_RUPEES_GOAL) >= 100
                {
                    self.runtime.show_message_unconditional(self.ram, 0xd2);
                    self.ram.set_sprite_ai_state(slot, 2);
                } else {
                    self.runtime.show_message_unconditional(self.ram, 0xd3);
                    self.ram.set_sprite_ai_state(slot, 0);
                }
            }
            2 => {
                self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
                self.runtime.link_receive_item(self.ram, 0x16, 0);
                self.ram.write_u8(
                    SRAM_PROGRESS_INDICATOR_3,
                    self.ram.read_u8(SRAM_PROGRESS_INDICATOR_3) | 2,
                );
                self.ram.write_u16(
                    LINK_RUPEES_GOAL,
                    self.ram.read_u16(LINK_RUPEES_GOAL).wrapping_sub(100),
                );
                self.ram.set_sprite_ai_state(slot, 0);
            }
            3 => {
                let message = if self.ram.sprite(slot).signed_field(SpriteField::ScratchE) >= 0 {
                    0xd5
                } else {
                    0xd6
                };
                self.runtime.show_message_unconditional(self.ram, message);
                self.ram.set_sprite_ai_state(slot, 4);
            }
            4 => {
                let target = self.ram.sprite(slot).scratch_e();
                if target as i8 >= 0 {
                    self.ram.set_sprite_state(usize::from(target - 1), 0);
                    self.runtime.bottle_merchant_buy_bee(self.ram, slot);
                } else {
                    self.ram.set_sprite_state(usize::from(target & 0xf), 0);
                    self.runtime.bottle_merchant_buy_fish(self.ram, slot);
                }
                self.ram.set_sprite_scratch_e(slot, 0);
                self.ram.set_sprite_ai_state(slot, 0);
            }
            _ => unreachable!("bottle vendor has only five AI states"),
        }
    }

    pub fn bottle_vendor_draw(&mut self, slot: usize) -> u8 {
        const PIECES: [DrawMultipleData; 4] = [
            DrawMultipleData {
                x: 0,
                y: -7,
                char_flags: 0x00ac,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0088,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -6,
                char_flags: 0x00ac,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x00a2,
                ext: 2,
            },
        ];
        let start = usize::from(self.ram.sprite(slot).graphics()) * 2;
        let info = self.runtime.draw_multiple_player_deferred_slice(
            self.ram,
            slot,
            &PIECES[start..start + 2],
        );
        let mut info = info;
        self.runtime
            .sprite_draw_shadow_with_info(self.ram, slot, &mut info);
        ((info.x | info.y) >> 8) as u8
    }

    pub fn sprite_76_zelda(&mut self, slot: usize) {
        self.runtime.crystal_maiden_draw(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
        if self.runtime.sprite_track_body_to_head(self.ram, slot) {
            self.runtime.sprite_move_xy(self.ram, slot);
        }
        match self.ram.sprite(slot).subtype2() {
            0 => self.zelda_in_cell(slot),
            1 => self.zelda_entering_sanctuary(slot),
            2 => self.zelda_at_sanctuary(slot),
            _ => unreachable!("Zelda has only three subtypes"),
        }
    }

    pub fn zelda_in_cell(&mut self, slot: usize) {
        const X_VELOCITY: [i8; 4] = [0, 0, -9, 9];
        const Y_VELOCITY: [i8; 4] = [-9, 9, 0, 0];
        let direction = self
            .runtime
            .sprite_direction_to_face_link_direction(self.ram, slot)
            ^ 3;
        self.ram.set_sprite_head_direction(slot, direction);
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if !self
                    .runtime
                    .sprite_check_damage_to_link_same_layer(self.ram, slot)
                {
                    return;
                }
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                self.ram.write_u8(
                    FLAG_IS_LINK_IMMOBILIZED,
                    self.ram.read_u8(FLAG_IS_LINK_IMMOBILIZED).wrapping_add(1),
                );
                let direction = usize::from(self.ram.sprite(slot).head_direction());
                self.ram.set_sprite_signed_field(
                    slot,
                    SpriteField::VelocityX,
                    X_VELOCITY[direction],
                );
                self.ram.set_sprite_signed_field(
                    slot,
                    SpriteField::VelocityY,
                    Y_VELOCITY[direction],
                );
                self.ram.set_sprite_main_delay(slot, 16);
            }
            1 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.runtime.show_message_unconditional(self.ram, 0x1c);
                    self.ram.set_sprite_velocity_x(slot, 0);
                    self.ram.set_sprite_velocity_y(slot, 0);
                    self.ram.write_u8(MUSIC_CONTROL, 25);
                }
                self.ram
                    .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
            }
            2 => {
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                self.runtime.show_message_unconditional(self.ram, 0x25);
            }
            3 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) != 0 {
                    self.ram.set_sprite_ai_state(slot, 2);
                } else {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.runtime.show_message_unconditional(self.ram, 0x24);
                }
            }
            4 => {
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                self.ram.write_u8(WHICH_STARTING_POINT, 2);
                self.runtime.save_palace_deaths(self.ram);
                self.ram.write_u8(FOLLOWER_INDICATOR, 1);
                self.runtime.dungeon_flag_room_data_quadrants(self.ram);
                self.runtime.sprite_become_follower(self.ram, slot);
                self.ram.set_sprite_state(slot, 0);
                self.ram.write_u8(MUSIC_CONTROL, 16);
            }
            _ => unreachable!("Zelda in cell has only five AI states"),
        }
    }

    pub fn zelda_entering_sanctuary(&mut self, slot: usize) {
        const DELAY: [u8; 4] = [38, 26, 44, 1];
        const DIRECTION: [u8; 4] = [1, 3, 1, 2];
        const X_VELOCITY: [i8; 4] = [0, 0, -9, 9];
        const Y_VELOCITY: [i8; 4] = [-9, 9, 0, 0];
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    let phase = usize::from(self.ram.sprite(slot).scratch_a());
                    if phase >= 4 {
                        self.ram.set_sprite_ai_state(
                            slot,
                            self.ram.sprite(slot).ai_state().wrapping_add(1),
                        );
                        self.ram.set_sprite_head_direction(slot, 0);
                        self.ram.set_sprite_scratch_d(slot, 0);
                        self.ram.set_sprite_velocity_x(slot, 0);
                        self.ram.set_sprite_velocity_y(slot, 0);
                        return;
                    }
                    self.ram.set_sprite_main_delay(slot, DELAY[phase]);
                    let direction = usize::from(DIRECTION[phase]);
                    self.ram.set_sprite_scratch_d(slot, direction as u8);
                    self.ram.set_sprite_head_direction(slot, direction as u8);
                    self.ram.set_sprite_scratch_a(slot, phase as u8 + 1);
                    self.ram.set_sprite_signed_field(
                        slot,
                        SpriteField::VelocityX,
                        X_VELOCITY[direction],
                    );
                    self.ram.set_sprite_signed_field(
                        slot,
                        SpriteField::VelocityY,
                        Y_VELOCITY[direction],
                    );
                }
                self.ram
                    .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
            }
            1 => {
                self.runtime.show_message_unconditional(self.ram, 0x1d);
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                self.ram.write_u8(BYTE_7FFE01, 2);
                self.ram.write_u8(WHICH_STARTING_POINT, 1);
                self.runtime.save_palace_deaths(self.ram);
                self.ram.write_u8(SRAM_PROGRESS_INDICATOR, 2);
                self.runtime
                    .sprite_load_graphics_properties_light_world_only(self.ram);
            }
            2 => {
                let direction = self
                    .runtime
                    .sprite_direction_to_face_link_direction(self.ram, slot)
                    ^ 3;
                self.ram.set_sprite_head_direction(slot, direction);
                let message = self.runtime.show_solicited_message(self.ram, slot, 0x1e);
                if message & 0x100 != 0 {
                    self.ram.set_sprite_scratch_d(slot, message as u8);
                    self.ram.set_sprite_head_direction(slot, message as u8);
                }
            }
            _ => unreachable!("Zelda entering sanctuary has three AI states"),
        }
    }

    pub fn zelda_at_sanctuary(&mut self, slot: usize) {
        let direction = self
            .runtime
            .sprite_direction_to_face_link_direction(self.ram, slot)
            ^ 3;
        self.ram.set_sprite_head_direction(slot, direction);
        let message_id = if self.ram.read_u8(LINK_WHICH_PENDANTS) & 7 == 7 {
            0x27
        } else if self.ram.read_u8(SAVEGAME_MAP_ICONS_INDICATOR) >= 3 {
            0x26
        } else {
            0x1e
        };
        let message = self
            .runtime
            .show_solicited_message(self.ram, slot, message_id);
        if message & 0x100 != 0 {
            self.ram.set_sprite_scratch_d(slot, message as u8);
            self.ram.set_sprite_head_direction(slot, message as u8);
            self.ram.write_u8(LINK_HEARTS_FILLER, 0xa0);
        }
    }

    pub fn sprite_prep_mushroom(&mut self, slot: usize) {
        if self.ram.read_u8(LINK_ITEM_MUSHROOM) >= 2 {
            self.ram.set_sprite_state(slot, 0);
        } else {
            self.ram.set_sprite_graphics(slot, 0);
            self.ram
                .set_sprite_oam_flags(slot, self.ram.sprite(slot).oam_flags() | 8);
            self.ram.set_sprite_ignore_projectile(
                slot,
                self.ram.sprite(slot).ignore_projectile().wrapping_add(1),
            );
        }
    }

    pub fn sprite_e7_mushroom(&mut self, slot: usize) {
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_check_if_link_is_busy(self.ram) {
            return;
        }
        if self.ram.read_u16(ENHANCED_FEATURES0) & 4096 != 0
            && self.ram.read_u8(SUBMODULE_INDEX) != 0
        {
            return;
        }
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            self.ram.set_sprite_state(slot, 0);
            self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
            self.runtime.link_receive_item(self.ram, 0x29, 0);
        } else if self.ram.read_u8(FRAME_COUNTER) & 0x1f == 0 {
            self.ram
                .set_sprite_oam_flags(slot, self.ram.sprite(slot).oam_flags() ^ 0x40);
        }
    }

    pub fn sprite_e8_fake_sword(&mut self, slot: usize) {
        self.fake_sword_draw(slot);
        if self.runtime.sprite_return_if_paused(self.ram, slot) {
            return;
        }
        if self.ram.sprite(slot).unknown3() == 3 {
            if self.ram.sprite(slot).scratch_c() == 0 {
                self.ram.set_sprite_scratch_c(slot, 1);
                self.runtime.show_message_unconditional(self.ram, 0x6f);
            }
        } else {
            self.runtime.sprite_move_xy(self.ram, slot);
            self.runtime
                .thrown_sprite_tile_and_sprite_interaction(self.ram, slot);
        }
    }

    pub fn fake_sword_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 2] = [
            DrawMultipleData {
                x: 4,
                y: 0,
                char_flags: 0x00f4,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 8,
                char_flags: 0x00f5,
                ext: 0,
            },
        ];
        let _ = self
            .runtime
            .draw_multiple_player_deferred_slice(self.ram, slot, &PIECES);
    }

    pub fn sprite_prep_heart_container(&mut self, slot: usize) {
        self.heart_upgrade_check_if_already_obtained(slot);
    }

    pub fn sprite_heart_container(&mut self, slot: usize) {
        if self.ram.read_u8(CUR_PALACE_INDEX_X2) == 26 {
            self.ram.set_sprite_state(slot, 0);
            return;
        }
        self.ram
            .set_sprite_ignore_projectile(slot, self.ram.sprite(slot).scratch_g());
        if self.ram.sprite(slot).scratch_g() == 0 {
            self.runtime
                .decode_animated_sprite_tile_variable(self.ram, 3);
            self.runtime.sprite_get_16_bit_coords(self.ram, slot);
            self.ram.set_sprite_scratch_g(slot, 1);
        }
        if self.ram.read_u8(DUNGEON_ROOM_INDEX2) == 6 && self.ram.sprite(slot).z() == 0 {
            self.runtime
                .sprite_draw_water_ripple_with_oam_adjust(self.ram, slot);
        }
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.ram
            .set_sprite_velocity_z(slot, self.ram.sprite(slot).velocity_z().wrapping_sub(2));
        self.runtime.sprite_move_z(self.ram, slot);
        if self.ram.sprite(slot).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_z(slot, 0);
            self.ram.set_sprite_velocity_z(
                slot,
                self.ram.sprite(slot).velocity_z().wrapping_neg() >> 2,
            );
            if self.ram.read_u8(DUNGEON_ROOM_INDEX2) == 6 && self.ram.sprite(slot).subtype() == 0 {
                self.ram
                    .set_sprite_flags2(slot, self.ram.sprite(slot).flags2().wrapping_add(2));
                self.ram.set_sprite_subtype(slot, 1);
                self.runtime.sprite_spawn_water_splash(self.ram, slot);
            }
        }
        if self.runtime.sprite_check_if_link_is_busy(self.ram)
            || !self
                .runtime
                .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            return;
        }
        self.ram.set_sprite_state(slot, 0);
        if self.ram.sprite(slot).scratch_a() != 0 {
            self.ram.write_u8(ITEM_RECEIPT_METHOD, 2);
            self.runtime.link_receive_item(self.ram, 0x3e, 2);
            self.ram.write_u16(
                DUNG_SAVEGAME_STATE_BITS,
                self.ram.read_u16(DUNG_SAVEGAME_STATE_BITS) | 0x8000,
            );
            return;
        }
        self.runtime.link_cancel_dash(self.ram);
        self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
        self.runtime.link_receive_item(self.ram, 0x26, 0);
        self.heart_upgrade_set_obtained_flag(slot);
    }

    pub fn sprite_heart_piece(&mut self, slot: usize) {
        const MESSAGE: [u16; 4] = [0x158, 0x155, 0x156, 0x157];
        if self.ram.sprite(slot).ai_state() == 0 {
            self.ram
                .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
            self.heart_upgrade_check_if_already_obtained(slot);
            if self.ram.sprite(slot).state() == 0 {
                return;
            }
        }
        self.runtime.draw_single_large(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || self.runtime.sprite_check_if_link_is_busy(self.ram)
        {
            return;
        }
        if self.runtime.sprite_check_tile_collision(self.ram, slot) & 3 != 0 {
            self.ram
                .set_sprite_velocity_x(slot, self.ram.sprite(slot).velocity_x().wrapping_neg());
        }
        self.ram
            .set_sprite_velocity_z(slot, self.ram.sprite(slot).velocity_z().wrapping_sub(1));
        self.runtime.sprite_move_z(self.ram, slot);
        self.runtime.sprite_move_xy(self.ram, slot);
        if self.ram.sprite(slot).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_z(slot, 0);
            self.ram.set_sprite_velocity_z(
                slot,
                ((self.ram.sprite(slot).velocity_z() ^ 255) & 248) >> 1,
            );
            self.ram.set_sprite_signed_field(
                slot,
                SpriteField::VelocityX,
                self.ram.sprite(slot).signed_field(SpriteField::VelocityX) >> 1,
            );
        }
        if self.ram.sprite(slot).aux4_delay() != 0
            || !self
                .runtime
                .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            return;
        }
        let pieces = self.ram.read_u8(LINK_HEART_PIECES).wrapping_add(1) & 3;
        self.ram.write_u8(LINK_HEART_PIECES, pieces);
        if pieces == 0 {
            self.runtime.link_cancel_dash(self.ram);
            self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
            self.runtime.link_receive_item(self.ram, 0x26, 0);
        } else {
            self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x2d);
            self.runtime
                .show_message_unconditional(self.ram, MESSAGE[usize::from(pieces)]);
        }
        self.ram.set_sprite_state(slot, 0);
        self.heart_upgrade_set_obtained_flag(slot);
    }

    pub fn heart_upgrade_set_obtained_flag(&mut self, slot: usize) {
        if self.ram.read_u8(PLAYER_IS_INDOORS) == 0 {
            let event = SAVE_OW_EVENT_INFO + usize::from(self.ram.read_u8(OVERWORLD_SCREEN_INDEX));
            self.ram.write_u8(event, self.ram.read_u8(event) | 0x40);
        } else {
            let flag = if self.ram.sprite(slot).x_high() & 1 != 0 {
                0x2000
            } else {
                0x4000
            };
            self.ram.write_u16(
                DUNG_SAVEGAME_STATE_BITS,
                self.ram.read_u16(DUNG_SAVEGAME_STATE_BITS) | flag,
            );
        }
    }

    pub fn sprite_aginah(&mut self, slot: usize) {
        let message = if self.ram.read_u8(SRAM_PROGRESS_FLAGS) & 0x20 == 0 {
            self.ram.write_u8(
                SRAM_PROGRESS_FLAGS,
                self.ram.read_u8(SRAM_PROGRESS_FLAGS) | 0x20,
            );
            0x125
        } else if self.ram.read_u8(LINK_SWORD_TYPE) >= 2 {
            0x128
        } else if self.ram.read_u8(LINK_WHICH_PENDANTS) & 7 == 7 {
            0x126
        } else if self.ram.read_u8(LINK_WHICH_PENDANTS) & 2 != 0 {
            0x129
        } else if self.ram.read_u8(LINK_ITEM_BOOK_OF_MUDORA) != 0 {
            0x127
        } else {
            self.ram.write_u8(
                SRAM_PROGRESS_FLAGS,
                self.ram.read_u8(SRAM_PROGRESS_FLAGS) | 0x20,
            );
            0x125
        };
        let _ = self.runtime.show_solicited_message(self.ram, slot, message);
        self.ram
            .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 5 & 1);
    }

    pub fn sprite_sahasrahla(&mut self, slot: usize) {
        match self.ram.sprite(slot).ai_state() {
            0 => self.sasha_idle(slot),
            1 => {
                self.runtime.show_message_unconditional(self.ram, 0x33);
                self.ram.set_sprite_ai_state(slot, 0);
                self.ram.write_u8(SAVEGAME_MAP_ICONS_INDICATOR, 3);
            }
            2 => {
                self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
                self.runtime.link_receive_item(self.ram, 0x4b, 0);
                self.ram.set_sprite_ai_state(slot, 3);
                self.ram.write_u8(SAVEGAME_MAP_ICONS_INDICATOR, 3);
            }
            3 => {
                self.runtime.show_message_unconditional(self.ram, 0x37);
                self.ram.set_sprite_ai_state(slot, 0);
            }
            _ => unreachable!("Sahasrahla has only four AI states"),
        }
    }

    pub fn sasha_idle(&mut self, slot: usize) {
        let pendants = self.ram.read_u8(LINK_WHICH_PENDANTS);
        if pendants & 4 == 0 {
            if self.runtime.show_solicited_message(self.ram, slot, 0x32) & 0x100 != 0 {
                self.ram.set_sprite_ai_state(slot, 1);
            }
        } else if self.ram.read_u8(LINK_ITEM_BOOTS) == 0 {
            let message = if self.ram.read_u8(SAVEGAME_MAP_ICONS_INDICATOR) >= 3 {
                0x38
            } else {
                0x39
            };
            if self.runtime.show_solicited_message(self.ram, slot, message) & 0x100 != 0 {
                self.ram.set_sprite_ai_state(slot, 2);
            }
        } else {
            let message = if self.ram.read_u8(LINK_ITEM_ICE_ROD) == 0 {
                0x37
            } else if pendants & 7 != 7 {
                0x34
            } else if self.ram.read_u8(LINK_SWORD_TYPE) < 2 {
                0x30
            } else {
                0x31
            };
            let _ = self.runtime.show_solicited_message(self.ram, slot, message);
        }
        self.ram
            .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 5 & 1);
    }

    pub fn elder_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 4] = [
            DrawMultipleData {
                x: 0,
                y: -9,
                char_flags: 0x00a0,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x00a2,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x00a0,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x40a4,
                ext: 2,
            },
        ];
        let start = usize::from(self.ram.sprite(slot).graphics()) * 2;
        let info = self.runtime.draw_multiple_player_deferred_slice(
            self.ram,
            slot,
            &PIECES[start..start + 2],
        );
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_dust_cloud(&mut self, slot: usize) {
        const GRAPHICS: [u8; 9] = [0, 1, 2, 3, 4, 5, 1, 0, 0xff];
        self.runtime.dust_cloud_draw(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot)
            || self.ram.sprite(slot).main_delay() != 0
        {
            return;
        }
        self.ram.set_sprite_main_delay(slot, 5);
        let phase = usize::from(self.ram.sprite(slot).scratch_a());
        if GRAPHICS[phase] as i8 >= 0 {
            self.ram.set_sprite_graphics(slot, GRAPHICS[phase]);
            self.ram.set_sprite_scratch_a(slot, phase as u8 + 1);
        } else {
            self.ram.set_sprite_state(slot, 0);
        }
    }

    pub fn sprite_spawn_dust_cloud(&mut self, slot: usize) -> i32 {
        let Some((spawned, mut info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0xf2)
        else {
            return -1;
        };
        info.r2_y = info
            .r2_y
            .wrapping_add(u16::from(crate::misc::get_random_number(self.ram) & 15));
        let x_offset = i16::from(crate::misc::get_random_number(self.ram) & 15) - 8;
        info.r0_x = info.r0_x.wrapping_add(x_offset as u16);
        self.set_spawned_coordinates(spawned, &info);
        self.ram.set_sprite_subtype2(spawned, 1);
        spawned as i32
    }

    pub fn medallion_tablet_main(&mut self, slot: usize) {
        self.runtime.medallion_tablet_draw(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.ram.write_u8(
            LINK_POSITION_MODE,
            self.ram.read_u8(LINK_POSITION_MODE) & !0x20,
        );
        self.ram.set_sprite_scratch_a(slot, 0);
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            self.runtime.sprite_nullify_hookshot_drag(self.ram);
            self.ram.write_u8(LINK_SPEED_SETTING, 0);
            self.runtime.sprite_repel_dash(self.ram);
            self.ram
                .set_sprite_scratch_a(slot, self.ram.sprite(slot).scratch_a().wrapping_add(1));
        }
        if self.runtime.sprite_check_if_link_is_busy(self.ram) {
            return;
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self.ram.read_u8(OVERWORLD_SCREEN_INDEX) != 3 {
                    self.bombos_tablet(slot);
                } else {
                    self.ether_tablet(slot);
                }
            }
            1 => {
                if self.ram.sprite(slot).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.set_sprite_main_delay(slot, 128);
                }
            }
            2 => {
                let delay = self.ram.sprite(slot).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(
                        slot,
                        self.ram.sprite(slot).ai_state().wrapping_add(1),
                    );
                    self.ram.set_sprite_main_delay(slot, 240);
                } else {
                    if matches!(delay, 0x20 | 0x40 | 0x60) {
                        self.ram.set_sprite_graphics(
                            slot,
                            self.ram.sprite(slot).graphics().wrapping_add(1),
                        );
                    }
                    if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
                        let _ = self.sprite_spawn_dust_cloud(slot);
                    }
                }
            }
            3 => self.ram.set_sprite_graphics(slot, 4),
            _ => unreachable!("medallion tablet has only four AI states"),
        }
    }

    pub fn bombos_tablet(&mut self, slot: usize) {
        self.medallion_tablet_interaction(slot, [0x10d, 0x10f], false);
    }

    pub fn ether_tablet(&mut self, slot: usize) {
        self.medallion_tablet_interaction(slot, [0x10d, 0x10e], true);
    }

    fn medallion_tablet_interaction(&mut self, slot: usize, messages: [u16; 2], ether: bool) {
        if self.ram.read_u8(LINK_DIRECTION_FACING) != 0
            || self
                .runtime
                .sprite_direction_to_face_link_direction(self.ram, slot)
                != 2
        {
            return;
        }
        let too_high = if ether {
            self.ram.sprite(slot).y_low().wrapping_add(16) < self.ram.read_u16(LINK_Y) as u8
        } else {
            self.ram.read_u16(CUR_SPRITE_Y).wrapping_add(16) < self.ram.read_u16(LINK_Y)
        };
        if too_high
            || (self.ram.read_u8(FILTERED_JOYPAD_H) & 0x80 != 0
                && self.ram.read_u8(LINK_SWORD_TYPE) == 2)
        {
            return;
        }
        let use_book =
            self.ram.read_u8(HUD_CUR_ITEM) == 15 && self.ram.read_u8(FILTERED_JOYPAD_H) & 0x40 != 0;
        let action = usize::from(use_book);
        if !use_book && self.ram.read_u8(FILTERED_JOYPAD_L) & 0x80 == 0 {
            return;
        }
        if use_book {
            self.ram.write_u8(PLAYER_HANDLER_TIMER, 0);
            self.ram.write_u8(LINK_POSITION_MODE, 32);
            self.ram.write_u8(SOUND_EFFECT_1, 0);
            let sword = self.ram.read_u8(LINK_SWORD_TYPE);
            if sword as i8 >= 0 && sword >= 2 {
                self.ram
                    .set_sprite_ai_state(slot, self.ram.sprite(slot).ai_state().wrapping_add(1));
                if ether {
                    self.runtime.ether_tablet_start_cutscene(self.ram);
                } else {
                    self.runtime.bombos_tablet_start_cutscene(self.ram);
                }
                self.ram.set_sprite_main_delay(slot, 64);
            }
        }
        self.runtime
            .show_message_unconditional(self.ram, messages[action]);
    }

    pub fn elder_wife_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 4] = [
            DrawMultipleData {
                x: 0,
                y: -5,
                char_flags: 0x008e,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 5,
                char_flags: 0x0028,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -4,
                char_flags: 0x008e,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 5,
                char_flags: 0x4028,
                ext: 2,
            },
        ];
        let start = usize::from(self.ram.sprite(slot).graphics()) * 2;
        let _ = self.runtime.draw_multiple_player_deferred_slice(
            self.ram,
            slot,
            &PIECES[start..start + 2],
        );
    }

    pub fn sprite_prep_potion_shop(&mut self, slot: usize) {
        self.magic_shop_assistant_spawn_powder(slot);
        self.magic_shop_assistant_spawn_green_cauldron(slot);
        self.magic_shop_assistant_spawn_blue_cauldron(slot);
        self.magic_shop_assistant_spawn_red_cauldron(slot);
        self.ram.set_sprite_ignore_projectile(
            slot,
            self.ram.sprite(slot).ignore_projectile().wrapping_add(1),
        );
    }

    pub fn magic_shop_assistant_spawn_powder(&mut self, slot: usize) {
        if self.ram.read_u8(FLAG_OVERWORLD_AREA_DID_CHANGE) == 0
            || self.ram.read_u8(LINK_ITEM_MUSHROOM) == 2
            || self.ram.read_u16(SAVE_DUNG_INFO + 0x109 * 2) & 0x80 == 0
        {
            return;
        }
        let Some((spawned, mut info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0xe9)
        else {
            return;
        };
        info.r0_x = info.r0_x.wrapping_sub(16);
        self.set_spawned_coordinates(spawned, &info);
        self.finish_potion_shop_spawn(spawned, 1);
    }

    pub fn magic_shop_assistant_spawn_green_cauldron(&mut self, slot: usize) {
        self.spawn_potion_cauldron(slot, 2, -40, -72);
    }

    pub fn magic_shop_assistant_spawn_blue_cauldron(&mut self, slot: usize) {
        self.spawn_potion_cauldron(slot, 3, 8, -72);
    }

    pub fn magic_shop_assistant_spawn_red_cauldron(&mut self, slot: usize) {
        self.spawn_potion_cauldron(slot, 4, -88, -72);
    }

    fn spawn_potion_cauldron(&mut self, slot: usize, subtype: u8, x: i16, y: i16) {
        let Some((spawned, mut info)) = self.runtime.sprite_spawn_dynamically(self.ram, slot, 0xe9)
        else {
            return;
        };
        info.r0_x = info.r0_x.wrapping_add(x as u16);
        info.r2_y = info.r2_y.wrapping_add(y as u16);
        self.set_spawned_coordinates(spawned, &info);
        self.finish_potion_shop_spawn(spawned, subtype);
    }

    fn finish_potion_shop_spawn(&mut self, spawned: usize, subtype: u8) {
        self.ram.set_sprite_subtype2(spawned, subtype);
        self.ram.set_sprite_flags4(spawned, 3);
        self.ram
            .set_sprite_deflection_bits(spawned, self.ram.sprite(spawned).deflection_bits() | 0x20);
    }

    pub fn sprite_e9_potion_shop(&mut self, slot: usize) {
        match self.ram.sprite(slot).subtype2() {
            0 => self.sprite_magic_shop_assistant_main(slot),
            1 => self.sprite_bag_of_powder(slot),
            2 => self.sprite_green_cauldron(slot),
            3 => self.sprite_blue_cauldron(slot),
            4 => self.sprite_red_cauldron(slot),
            _ => {}
        }
    }

    pub fn sprite_bag_of_powder(&mut self, slot: usize) {
        self.magic_powder_item_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
        if !self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
            || self.ram.read_u8(FILTERED_JOYPAD_L) & 0x80 == 0
        {
            return;
        }
        self.runtime.link_cancel_dash(self.ram);
        self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
        self.runtime.link_receive_item(self.ram, 0x0d, 0);
        self.ram.set_sprite_state(slot, 0);
    }

    pub fn magic_powder_item_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 2] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x04e6,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x04e6,
                ext: 2,
            },
        ];
        let _ = self
            .runtime
            .draw_multiple_player_deferred_slice(self.ram, slot, &PIECES);
    }

    pub fn sprite_green_cauldron(&mut self, slot: usize) {
        self.green_potion_item_draw(slot);
        self.potion_cauldron_main(slot, 60, 0x2f);
    }

    pub fn green_potion_item_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 3] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x08c0,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 18,
                char_flags: 0x0a30,
                ext: 0,
            },
            DrawMultipleData {
                x: -1,
                y: 18,
                char_flags: 0x0a22,
                ext: 0,
            },
        ];
        let _ = self
            .runtime
            .draw_multiple_player_deferred_slice(self.ram, slot, &PIECES);
    }

    pub fn sprite_blue_cauldron(&mut self, slot: usize) {
        self.blue_potion_item_draw(slot);
        self.potion_cauldron_main(slot, 160, 0x30);
    }

    pub fn blue_potion_item_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 4] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x04c0,
                ext: 2,
            },
            DrawMultipleData {
                x: 13,
                y: 18,
                char_flags: 0x0a30,
                ext: 0,
            },
            DrawMultipleData {
                x: 5,
                y: 18,
                char_flags: 0x0a22,
                ext: 0,
            },
            DrawMultipleData {
                x: -3,
                y: 18,
                char_flags: 0x0a31,
                ext: 0,
            },
        ];
        let _ = self
            .runtime
            .draw_multiple_player_deferred_slice(self.ram, slot, &PIECES);
    }

    pub fn sprite_red_cauldron(&mut self, slot: usize) {
        self.red_potion_item_draw(slot);
        self.potion_cauldron_main(slot, 120, 0x2e);
    }

    fn potion_cauldron_main(&mut self, slot: usize, price: u16, item: u8) {
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
        if self.ram.sprite(slot).main_delay() != 0 {
            return;
        }
        if !self.potion_cauldron_check_bottles() {
            if self.runtime.show_message_on_contact(self.ram, slot, 0x4f) & 0x100 != 0 {
                self.potion_cauldron_go_beep(slot);
            }
            return;
        }
        if !self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
            || self.ram.read_u8(FILTERED_JOYPAD_L) & 0x80 == 0
        {
            return;
        }
        if self.ram.read_u16(LINK_RUPEES_GOAL) < price {
            self.runtime.show_message_unconditional(self.ram, 0x17c);
            self.potion_cauldron_go_beep(slot);
            return;
        }
        if self.runtime.sprite_find_empty_bottle(self.ram) < 0 {
            self.runtime.show_message_unconditional(self.ram, 0x50);
            self.potion_cauldron_go_beep(slot);
            return;
        }
        self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x1d);
        self.ram.set_sprite_main_delay(slot, 64);
        self.ram.write_u16(
            LINK_RUPEES_GOAL,
            self.ram.read_u16(LINK_RUPEES_GOAL).wrapping_sub(price),
        );
        self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
        self.runtime.link_receive_item(self.ram, item, 0);
    }

    pub fn potion_cauldron_go_beep(&mut self, slot: usize) {
        self.runtime.queue_sfx2_with_pan(self.ram, slot, 0x3c);
    }

    pub fn red_potion_item_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 4] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x02c0,
                ext: 2,
            },
            DrawMultipleData {
                x: 13,
                y: 18,
                char_flags: 0x0a30,
                ext: 0,
            },
            DrawMultipleData {
                x: 5,
                y: 18,
                char_flags: 0x0a02,
                ext: 0,
            },
            DrawMultipleData {
                x: -3,
                y: 18,
                char_flags: 0x0a31,
                ext: 0,
            },
        ];
        let _ = self
            .runtime
            .draw_multiple_player_deferred_slice(self.ram, slot, &PIECES);
    }

    pub fn potion_cauldron_check_bottles(&self) -> bool {
        (self.ram.read_u8(LINK_BOTTLE_INFO)
            | self.ram.read_u8(LINK_BOTTLE_INFO + 1)
            | self.ram.read_u8(LINK_BOTTLE_INFO + 2)
            | self.ram.read_u8(LINK_BOTTLE_INFO + 3))
            >= 2
    }

    pub fn sprite_magic_shop_assistant_main(&mut self, slot: usize) {
        self.shopkeeper_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
        if self.runtime.sprite_check_if_link_is_busy(self.ram) {
            return;
        }
        if self.ram.sprite(slot).ai_state() != 0 {
            self.ram.write_u8(LINK_HEARTS_FILLER, 160);
            self.ram.set_sprite_ai_state(slot, 0);
        }
        self.ram
            .set_sprite_graphics(slot, self.ram.read_u8(FRAME_COUNTER) >> 5 & 1);
        let has_filled_bottle = (0..4).any(|i| self.ram.read_u8(LINK_BOTTLE_INFO + i) >= 2);
        let message = if has_filled_bottle || self.ram.read_u8(FLAG_OVERWORLD_AREA_DID_CHANGE) == 0
        {
            0x4e
        } else {
            0x4d
        };
        if self.runtime.show_solicited_message(self.ram, slot, message) & 0x100 != 0 {
            self.ram.set_sprite_ai_state(slot, 1);
        }
    }

    pub fn shopkeeper_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 4] = [
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x0c00,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0c10,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x0c00,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x4c10,
                ext: 2,
            },
        ];
        let start = usize::from(self.ram.sprite(slot).graphics()) * 2;
        let info = self.runtime.draw_multiple_player_deferred_slice(
            self.ram,
            slot,
            &PIECES[start..start + 2],
        );
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_dash_item(&mut self, slot: usize) {
        match self.ram.sprite(slot).graphics() {
            0 => self.runtime.sprite_book_of_mudora(self.ram, slot),
            1 => self.sprite_bonk_key(slot),
            2 => self.runtime.sprite_lumberjack_tree(self.ram, slot),
            _ => {}
        }
    }

    pub fn sprite_bonk_key(&mut self, slot: usize) {
        self.runtime.sprite_draw_thin_and_tall(self.ram, slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            self.ram.set_sprite_ai_state(slot, 3);
        }
        self.runtime.sprite_move_xy(self.ram, slot);
        self.ram
            .set_sprite_velocity_z(slot, self.ram.sprite(slot).velocity_z().wrapping_sub(1));
        self.runtime.sprite_move_z(self.ram, slot);
        if self.ram.sprite(slot).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_velocity_y(slot, 0);
            self.ram.set_sprite_z(slot, 0);
            self.ram.set_sprite_velocity_z(
                slot,
                self.ram.sprite(slot).velocity_z().wrapping_neg() >> 2,
            );
            if self.ram.sprite(slot).velocity_z() & 254 != 0 {
                self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x14);
            }
        }
        match self.ram.sprite(slot).ai_state() {
            0 => {
                if self
                    .ram
                    .read_u16(CUR_SPRITE_X)
                    .wrapping_sub(self.ram.read_u16(LINK_X))
                    .wrapping_add(16)
                    < 33
                    && self
                        .ram
                        .read_u16(CUR_SPRITE_Y)
                        .wrapping_sub(self.ram.read_u16(LINK_Y))
                        .wrapping_add(24)
                        < 41
                    && self.ram.read_u16(BG1_X_OFFSET) | self.ram.read_u16(BG1_Y_OFFSET) != 0
                {
                    self.ram.set_sprite_ai_state(slot, 1);
                }
            }
            1 => {
                self.ram.set_sprite_velocity_z(slot, 32);
                self.ram
                    .set_sprite_signed_field(slot, SpriteField::VelocityY, -5);
                self.ram.write_u8(SOUND_EFFECT_2, 27);
                self.ram.set_sprite_ai_state(slot, 2);
            }
            2 => {
                if self.ram.sprite(slot).z() == 0 {
                    self.ram
                        .set_sprite_floor(slot, self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL));
                }
            }
            3 => {
                self.ram.write_u8(
                    LINK_NUM_KEYS,
                    self.ram.read_u8(LINK_NUM_KEYS).wrapping_add(1),
                );
                self.ram.set_sprite_state(slot, 0);
                let flag = if self.ram.sprite(slot).death_action() != 0 {
                    0x2000
                } else {
                    0x4000
                };
                self.ram.write_u16(
                    DUNG_SAVEGAME_STATE_BITS,
                    self.ram.read_u16(DUNG_SAVEGAME_STATE_BITS) | flag,
                );
                self.runtime.queue_sfx3_with_pan(self.ram, slot, 0x2f);
            }
            _ => {}
        }
    }

    pub fn sprite_inn_keeper(&mut self, slot: usize) {
        self.inn_keeper_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        self.runtime.sprite_behave_as_barrier(self.ram, slot);
        let message = if self.ram.read_u8(LINK_ITEM_FLIPPERS) != 0 {
            0x183
        } else {
            0x182
        };
        let _ = self.runtime.show_solicited_message(self.ram, slot, message);
    }

    pub fn inn_keeper_draw(&mut self, slot: usize) {
        const PIECES: [DrawMultipleData; 2] = [
            DrawMultipleData {
                x: 0,
                y: -8,
                char_flags: 0x00c4,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x00ca,
                ext: 2,
            },
        ];
        let info = self
            .runtime
            .draw_multiple_player_deferred_slice(self.ram, slot, &PIECES);
        self.runtime.sprite_draw_shadow(self.ram, slot, &info);
    }

    pub fn sprite_witch(&mut self, slot: usize) {
        self.witch_draw(slot);
        if self.runtime.sprite_return_if_inactive(self.ram, slot) {
            return;
        }
        let saved_flags = self.ram.sprite(slot).flags4();
        self.ram.set_sprite_flags4(slot, 2);
        if self
            .runtime
            .sprite_check_damage_to_link_same_layer(self.ram, slot)
        {
            self.runtime.sprite_nullify_hookshot_drag(self.ram);
            self.ram.write_u8(LINK_SPEED_SETTING, 0);
            self.runtime.link_cancel_dash(self.ram);
        }
        self.ram.set_sprite_flags4(slot, saved_flags);
        if self.ram.read_u8(FRAME_COUNTER) == 0 {
            let animation_rate = (crate::misc::get_random_number(self.ram) & 1) + 2;
            self.ram.set_sprite_scratch_a(slot, animation_rate);
        }
        let shift = self.ram.sprite(slot).scratch_a() + 1;
        self.ram
            .set_sprite_graphics(slot, (self.ram.read_u8(FRAME_COUNTER) >> shift) & 7);
        if self.runtime.sprite_check_if_link_is_busy(self.ram) {
            return;
        }
        match self.ram.sprite(slot).ai_state() {
            0 => match self.ram.read_u8(LINK_ITEM_MUSHROOM) {
                0 => {
                    let message = if self.ram.read_u16(SAVE_DUNG_INFO + 0x109 * 2) & 0x80 != 0 {
                        0x4b
                    } else {
                        0x4a
                    };
                    let _ = self.runtime.show_solicited_message(self.ram, slot, message);
                }
                1 => {
                    if self.ram.read_u8(JOYPAD1_H_LAST) & 0x40 == 0 {
                        let _ = self.runtime.show_solicited_message(self.ram, slot, 0x4c);
                    } else if self
                        .runtime
                        .sprite_check_damage_to_link_same_layer(self.ram, slot)
                        && self.ram.read_u8(HUD_CUR_ITEM) == 5
                    {
                        self.witch_accept_shroom(slot);
                    }
                }
                _ => {
                    let _ = self.runtime.show_solicited_message(self.ram, slot, 0x4a);
                }
            },
            1 => {
                self.ram.set_sprite_ai_state(slot, 0);
                self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
                self.runtime.link_receive_item(self.ram, 0x18, 0);
            }
            _ => unreachable!("witch has only two AI states"),
        }
    }

    pub fn witch_accept_shroom(&mut self, slot: usize) {
        self.ram.write_u8(LINK_ITEM_MUSHROOM, 0);
        self.ram.write_u16(
            SAVE_DUNG_INFO + 0x109 * 2,
            self.ram.read_u16(SAVE_DUNG_INFO + 0x109 * 2) | 0x80,
        );
        self.ram.write_u8(SOUND_EFFECT_1, 0);
        self.runtime.hud_refresh_icon(self.ram);
        self.runtime.show_message_unconditional(self.ram, 0x4b);
        self.runtime.queue_sfx1_with_pan(self.ram, slot, 0xd);
        self.ram.write_u8(FLAG_OVERWORLD_AREA_DID_CHANGE, 0);
    }

    pub fn witch_draw(&mut self, slot: usize) {
        const BODY: [(i8, i8, u8); 16] = [
            (-3, 8, 0xae),
            (-3, 16, 0xbe),
            (-2, 8, 0xae),
            (-2, 16, 0xbe),
            (-1, 8, 0xaf),
            (-1, 16, 0xbf),
            (0, 9, 0xaf),
            (0, 17, 0xbf),
            (1, 10, 0xaf),
            (1, 18, 0xbf),
            (0, 11, 0xaf),
            (0, 18, 0xbf),
            (-1, 10, 0xae),
            (-1, 18, 0xbe),
            (-3, 9, 0xae),
            (-3, 17, 0xbe),
        ];
        const FIXED: [(i8, i8, u8, u8); 3] =
            [(0, -4, 0x80, 0), (-11, 15, 0x86, 4), (-3, 15, 0x86, 0x44)];
        const CAULDRON: [(i8, i8, u8); 2] = [(0, 4, 0x84), (0, 4, 0x82)];
        let Some(info) = self.runtime.prep_oam_coord_or_double_ret(self.ram, slot) else {
            return;
        };
        self.runtime.oam_allocate_defer_to_player(self.ram, slot);
        let first_oam = self.oam_current_index();
        let graphics = usize::from(self.ram.sprite(slot).graphics()) * 2;
        let x = self.ram.read_u8(DUNGMAP_VAR7);
        let y = self.ram.read_u8(DUNGMAP_VAR7 + 1);
        for piece in 0..2 {
            let data = BODY[graphics + piece];
            crate::oam::set_plain(
                self.ram,
                first_oam + piece,
                x.wrapping_add(data.0 as u8),
                y.wrapping_add(data.1 as u8),
                info.r4 | data.2,
                info.flags,
                0,
            );
        }
        for (piece, data) in FIXED.iter().enumerate() {
            crate::oam::set_plain(
                self.ram,
                first_oam + piece + 2,
                x.wrapping_add(data.0 as u8),
                y.wrapping_add(data.1 as u8),
                info.r4 ^ data.2,
                info.flags ^ data.3,
                2,
            );
        }
        let cauldron = usize::from((graphics as u16).wrapping_sub(6) < 6);
        let data = CAULDRON[cauldron];
        crate::oam::set_plain(
            self.ram,
            first_oam + 5,
            x.wrapping_add(data.0 as u8),
            y.wrapping_add(data.1 as u8),
            info.r4 | data.2,
            info.flags,
            2,
        );
        self.runtime
            .sprite_correct_oam_entries(self.ram, slot, 5, 0xff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[derive(Default)]
    struct Hooks {
        calls: Vec<(&'static str, usize)>,
        moldorm_frame: Option<[DrawMultipleData; 4]>,
        observed_oam_flags: Option<u8>,
        solicited_result: u16,
        messages: Vec<u16>,
        pond_items: Vec<u8>,
        prep: Option<PrepOamCoordsRet>,
        active_cross: Vec<(usize, u8)>,
        prep_cross: Vec<(usize, u8)>,
    }

    impl SpriteMainBank0Runtime for Hooks {
        fn sprite_active_cross_bank(&mut self, _ram: &mut Ram<'_>, slot: usize, kind: u8) {
            self.active_cross.push((slot, kind));
        }

        fn sprite_prep_cross_bank(&mut self, _ram: &mut Ram<'_>, slot: usize, kind: u8) {
            self.prep_cross.push((slot, kind));
        }

        fn draw_moldorm_segment(
            &mut self,
            _ram: &mut Ram<'_>,
            _slot: usize,
            pieces: &[DrawMultipleData; 4],
        ) {
            self.moldorm_frame = Some(*pieces);
        }

        fn draw_single_large(&mut self, ram: &mut Ram<'_>, slot: usize) {
            self.observed_oam_flags = Some(ram.sprite(slot).oam_flags());
        }

        fn sprite_return_if_lifted(&mut self, _ram: &mut Ram<'_>, slot: usize) {
            self.calls.push(("Sprite_ReturnIfLifted", slot));
        }

        fn show_solicited_message(
            &mut self,
            _ram: &mut Ram<'_>,
            _slot: usize,
            message: u16,
        ) -> u16 {
            self.messages.push(message);
            self.solicited_result
        }

        fn show_message_unconditional(&mut self, _ram: &mut Ram<'_>, message: u16) {
            self.messages.push(message);
        }

        fn add_tossed_pond_item(&mut self, _ram: &mut Ram<'_>, item: u8) {
            self.pond_items.push(item);
        }

        fn prep_oam_coord_or_double_ret(
            &mut self,
            _ram: &mut Ram<'_>,
            _slot: usize,
        ) -> Option<PrepOamCoordsRet> {
            self.prep
        }
    }
    impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for Hooks {}

    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for Hooks {}

    impl crate::sprite_main_bank1::SpriteMainBank1Runtime for Hooks {
        fn active_cross_bank(&mut self, _ram: &mut Ram<'_>, k: usize, kind: u8) {
            self.active_cross.push((k, kind));
        }

        fn prep_cross_bank(&mut self, _ram: &mut Ram<'_>, k: usize, kind: u8) {
            self.prep_cross.push((k, kind));
        }
    }

    #[test]
    fn first_handlers_match_source_state_and_callback_ordering() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.set_sprite_kind(2, 7);
        ram.set_sprite_subtype2(2, 3);
        ram.write_u8(MOLDORM_X_LO + 0x7f, 0x34);
        ram.write_u8(MOLDORM_X_HI + 0x7f, 0x12);
        ram.write_u8(MOLDORM_Y_LO + 0x7f, 0x78);
        ram.write_u8(MOLDORM_Y_HI + 0x7f, 0x56);
        ram.write_u16(OAM_CUR_PTR, 0x800);
        ram.write_u16(OAM_EXT_CUR_PTR, 0xa20);

        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        bank.pull_switch_bounce(2);
        bank.giant_moldorm_draw_segment_ab(2, 4);
        bank.chicken_incr_subtype2(2, 0x0d);

        assert_eq!(bank.ram.read_u16(CUR_SPRITE_X), 0x1234);
        assert_eq!(bank.ram.read_u16(CUR_SPRITE_Y), 0x5678);
        assert_eq!(bank.ram.read_u16(OAM_CUR_PTR), 0x810);
        assert_eq!(bank.ram.read_u16(OAM_EXT_CUR_PTR), 0xa24);
        assert_eq!(bank.ram.sprite(2).subtype2(), 0x10);
        assert_eq!(bank.ram.sprite(2).graphics(), 1);
        assert_eq!(hooks.calls, [("Sprite_ReturnIfLifted", 2)]);
        assert_eq!(
            hooks.moldorm_frame,
            Some(GIANT_MOLDORM_SEGMENT[4..8].try_into().unwrap())
        );
    }

    #[test]
    fn moldorm_tail_restores_oam_flags_after_draw() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.set_sprite_subtype2(5, 6);
        ram.set_sprite_oam_flags(5, 0x35);
        {
            let mut bank = SpriteMainBank0 {
                ram: &mut ram,
                runtime: &mut hooks,
            };
            bank.giant_moldorm_draw_segment_c_or_tail(5, 0);
            assert_eq!(bank.ram.sprite(5).oam_flags(), 0x35);
        }
        assert_eq!(hooks.observed_oam_flags, Some(0xb5));
    }

    #[test]
    fn octoballoon_and_flute_boy_queries_preserve_c_wrapping() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.set_sprite_state(15, 1);
        ram.set_sprite_kind(15, 0x10);
        ram.set_sprite_x_low(1, 0x10);
        ram.set_sprite_x_high(1, 1);
        ram.set_sprite_y_low(1, 0x20);
        ram.set_sprite_y_high(1, 1);
        ram.write_u16(LINK_X, 0x120);
        ram.write_u16(LINK_Y, 0x120);
        let bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        assert!(bank.octoballoon_find());
        assert!(bank.flute_boy_check_if_player_close(1));
    }

    #[test]
    fn fortune_teller_formats_price_then_deducts_with_c_wrapping() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.set_sprite_ai_state(3, 5);
        ram.set_sprite_scratch_a(3, 6);
        ram.set_sprite_graphics(3, 9);
        ram.write_u16(LINK_RUPEES_GOAL, 25);
        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        bank.fortune_teller_light_or_dark_world(3, false);
        assert_eq!(bank.ram.read_u8(DIALOGUE_NUMBER), 3);
        assert_eq!(bank.ram.read_u8(DIALOGUE_NUMBER + 1), 0);
        assert_eq!(bank.ram.sprite(3).graphics(), 0);
        assert_eq!(bank.ram.sprite(3).ai_state(), 6);
        bank.fortune_teller_light_or_dark_world(3, false);
        assert_eq!(bank.ram.read_u16(LINK_RUPEES_GOAL), 0xfffb);
        assert_eq!(bank.ram.sprite(3).ai_state(), 7);
        assert_eq!(bank.ram.read_u8(LINK_HEARTS_FILLER), 160);
        assert_eq!(hooks.messages, [0xf4]);
    }

    #[test]
    fn garnish_allocators_scan_backwards_and_rotate_when_full() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.slice_mut(GARNISH_TYPE, 30).fill(1);
        ram.write_u8(GARNISH_TYPE + 9, 0);
        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        assert_eq!(bank.garnish_alloc(), 9);
        assert_eq!(bank.garnish_alloc_low(), 9);
        bank.ram.write_u8(GARNISH_TYPE + 9, 1);
        assert_eq!(bank.garnish_alloc(), -1);
        assert_eq!(bank.garnish_alloc_force(), 0);
        bank.ram.write_u8(BYTE_7E0FF8, 0);
        assert_eq!(bank.garnish_alloc_overwrite_old_low(), 14);
        assert_eq!(bank.garnish_alloc_overwrite_old_low(), 13);
        bank.garnish_set_x(4, 0xabcd);
        bank.garnish_set_y(4, 0x1234);
        assert_eq!(bank.ram.read_u8(GARNISH_X_LO + 4), 0xcd);
        assert_eq!(bank.ram.read_u8(GARNISH_X_HI + 4), 0xab);
        assert_eq!(bank.ram.read_u8(GARNISH_Y_LO + 4), 0x34);
        assert_eq!(bank.ram.read_u8(GARNISH_Y_HI + 4), 0x12);
    }

    #[test]
    fn wish_pond_removes_and_restores_bow_slot_in_source_order() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.set_sprite_ai_state(2, 2);
        ram.write_u8(CHOICE_IN_MULTISELECT_BOX, 3);
        ram.write_u8(LINK_ITEM_BOW + 3, 9);
        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        bank.sprite_wish_pond3(2);
        assert_eq!(bank.ram.sprite(2).ai_state(), 3);
        assert_eq!(bank.ram.sprite(2).scratch_c(), 3);
        assert_eq!(bank.ram.sprite(2).scratch_d(), 9);
        assert_eq!(bank.ram.sprite(2).graphics(), 0x27);
        assert_eq!(bank.ram.read_u8(LINK_ITEM_BOW + 3), 0);
        assert_eq!(bank.runtime.pond_items, [0x27]);

        bank.ram.set_sprite_ai_state(2, 7);
        bank.sprite_wish_pond3(2);
        assert_eq!(bank.ram.read_u8(LINK_ITEM_BOW + 3), 9);
        assert_eq!(bank.ram.read_u8(TS_COPY), 2);
        assert_eq!(bank.ram.read_u8(CGADSUB_COPY), 0x30);
        assert_eq!(bank.ram.sprite(2).ai_state(), 8);
    }

    #[test]
    fn fixed_point_helpers_keep_c_rounding_and_complement_rules() {
        assert_eq!(SpriteMainBank0::<Hooks>::chain_ball_mult(256, 37), 37);
        assert_eq!(SpriteMainBank0::<Hooks>::chain_ball_mult(255, 255), 254);
        assert_eq!(SpriteMainBank0::<Hooks>::guruguru_bar_sin(0x180, 16), -16);
        assert_eq!(
            SpriteMainBank0::<Hooks>::chain_chomp_one_mult(0x80, 255),
            -128
        );
        assert_eq!(
            SpriteMainBank0::<Hooks>::trinexx_mult((-64_i8) as u8, 255),
            0xc0
        );
        assert_eq!(SINUS_LOOKUP_TABLE[0x80], 256);
    }

    #[test]
    fn medallion_prep_changes_only_low_coordinate_and_owned_state() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(OVERWORLD_SCREEN_INDEX, 2);
        ram.set_sprite_x_low(4, 0xfc);
        ram.set_sprite_x_high(4, 0x55);
        ram.write_u8(LINK_ITEM_BOMBOS_MEDALLION, 1);
        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        bank.sprite_prep_medallion_table(4);
        assert_eq!(bank.ram.sprite(4).ignore_projectile(), 1);
        assert_eq!(bank.ram.sprite(4).x_low(), 4);
        assert_eq!(bank.ram.sprite(4).x_high(), 0x55);
        assert_eq!(bank.ram.sprite(4).graphics(), 4);
        assert_eq!(bank.ram.sprite(4).ai_state(), 3);
    }

    #[test]
    fn beamos_hit_oam_matches_reverse_source_iteration() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks {
            prep: Some(PrepOamCoordsRet {
                x: 100,
                y: 80,
                r4: 0,
                flags: 0x31,
            }),
            ..Hooks::default()
        };
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OAM_CUR_PTR, crate::oam::OAM_BUFFER as u16);
        ram.set_sprite_state(2, 9);
        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        bank.sprite_beamos_laser_hit(2);
        assert_eq!(bank.ram.sprite(2).state(), 0);
        assert_eq!(crate::oam::read(bank.ram, 0).x, 104);
        assert_eq!(crate::oam::read(bank.ram, 0).y, 84);
        assert_eq!(crate::oam::read(bank.ram, 0).flags, 0xf6);
        assert_eq!(crate::oam::read(bank.ram, 3).x, 96);
        assert_eq!(crate::oam::read(bank.ram, 3).y, 76);
        assert_eq!(crate::oam::read(bank.ram, 3).flags, 0x36);
    }

    #[test]
    fn ball_n_chain_attack_common_does_not_fall_through_state_three() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks {
            prep: Some(PrepOamCoordsRet {
                x: 80,
                y: 64,
                r4: 0,
                flags: 0x31,
            }),
            ..Hooks::default()
        };
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OAM_CUR_PTR, crate::oam::OAM_BUFFER as u16);
        ram.set_sprite_ai_state(1, 3);
        ram.set_sprite_aux2_delay(1, 16);
        ram.set_sprite_scratch_d(1, 0);
        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        bank.sprite_6a_ball_n_chain(1);
        assert_eq!(bank.ram.sprite(1).subtype2(), 2);
        assert_eq!(bank.ram.sprite(1).graphics(), 0x10);
    }

    #[test]
    fn local_dispatch_preserves_active_and_prep_table_indices() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.set_sprite_kind(2, 5);
        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        assert!(bank.dispatch_active_local(2, 0x04));
        assert!(bank.runtime.calls.is_empty());
        assert!(!bank.dispatch_active_local(2, 0x03));

        assert!(bank.dispatch_prep_local(2, 0xe9));
        assert_eq!(bank.ram.sprite(2).ignore_projectile(), 1);
        assert!(!bank.dispatch_prep_local(2, 0x00));

        bank.ram.set_sprite_kind(2, 0);
        bank.dispatch_active(2, 0x00);
        bank.dispatch_prep(2, 0x00);
        assert!(bank.runtime.active_cross.is_empty());
        assert!(bank.runtime.prep_cross.is_empty());
    }

    #[test]
    fn pull_switch_dispatch_runs_the_translated_c_handler() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        let slot = 2;
        ram.set_sprite_kind(slot, 7);
        ram.set_sprite_graphics(slot, 10);
        ram.set_sprite_main_delay(slot, 0);
        ram.set_sprite_x_low(slot, 0x34);
        ram.set_sprite_x_high(slot, 0x12);
        ram.set_sprite_y_low(slot, 0x78);
        ram.set_sprite_y_high(slot, 0x56);

        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        bank.pull_switch_bounce(slot);

        assert_eq!(bank.ram.sprite(slot).graphics(), 11);
        assert_eq!(bank.ram.sprite(slot).main_delay(), 10);
        assert_eq!(bank.ram.read_u16(LINK_X), 0x1234);
        assert_eq!(bank.ram.read_u16(LINK_Y), 0x5678 - 19);
        assert_eq!(bank.ram.read_u8(SOUND_EFFECT_2), 0x1b);
        assert_eq!(bank.ram.read_u8(DUNG_FLAG_STATECHANGE_WATERPUZZLE), 1);
    }

    #[test]
    fn potion_bottle_or_and_bonk_key_save_flags_match_c_widths() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks::default();
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(LINK_BOTTLE_INFO + 3, 2);
        ram.set_sprite_ai_state(1, 3);
        ram.set_sprite_death_action(1, 1);
        ram.write_u8(LINK_NUM_KEYS, 0xff);
        ram.write_u16(DUNG_SAVEGAME_STATE_BITS, 0x0010);
        let mut bank = SpriteMainBank0 {
            ram: &mut ram,
            runtime: &mut hooks,
        };
        assert!(bank.potion_cauldron_check_bottles());
        bank.sprite_bonk_key(1);
        assert_eq!(bank.ram.read_u8(LINK_NUM_KEYS), 0);
        assert_eq!(bank.ram.sprite(1).state(), 0);
        assert_eq!(bank.ram.read_u16(DUNG_SAVEGAME_STATE_BITS), 0x2010);
    }
}
