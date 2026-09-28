//! Native translation of `src/overworld.c`.
//!
//! This module is being brought across in source order.  The first slice keeps
//! the table lookups and graphics-property setup byte-for-byte compatible with
//! the C implementation; later slices add the transition and map builders.

use crate::ram::Ram;

const BGMODE: usize = 0x94;
const MOSAIC: usize = 0x95;
const OVERWORLD_AREA_INDEX: usize = 0x40a;
const PALETTE_FILTER_COUNTDOWN: usize = 0xc007;
const MOSAIC_LEVEL: usize = 0xc011;
const SRAM_PROGRESS_INDICATOR: usize = 0xf3c5;
const FOLLOWER_INDICATOR: usize = 0xf3cc;
const OVERWORLD_SPRITE_GFX: usize = 0xfcc0;
const OVERWORLD_SPRITE_PALETTES: usize = 0xfd40;
const MAP16_DECODE_0: usize = 0x14400;
const MAP16_DECODE_1: usize = 0x14410;
const MAP16_DECODE_2: usize = 0x14420;
const MAP16_DECODE_3: usize = 0x14430;
const MAP16_DECODE_LAST: usize = 0x14440;
const MAP16_DECODE_TMP: usize = 0x14442;
const OVERWORLD_OFFSET_BASE_Y_ADDR: usize = 0x708;
const OVERWORLD_OFFSET_MASK_Y: usize = 0x70a;
const OVERWORLD_OFFSET_BASE_X_ADDR: usize = 0x70c;
const OVERWORLD_OFFSET_MASK_X: usize = 0x70e;
const DUNG_BG2: usize = 0x2000;
const DUNG_BG1: usize = 0x4000;
const LINK_Y: usize = 0x20;
const LINK_X: usize = 0x22;
const LINK_Z: usize = 0x24;
const LINK_FACING: usize = 0x2f;
const SOUND_EFFECT_1: usize = 0x12e;
const NUM_MEMORIZED_TILES: usize = 0x4ac;
const VRAM_UPLOAD_OFFSET: usize = 0x1000;
const VRAM_UPLOAD_DATA: usize = 0x1002;
const MEMORIZED_TILE_ADDR: usize = 0xf800;
const MEMORIZED_TILE_VALUE: usize = 0xfa00;
const LINK_ITEM_IN_HAND: usize = 0x301;
const DUNG_SECRETS_UNK1: usize = 0xb9c;
const ENEMY_DAMAGE_SCRATCH: usize = 0x14000;
const ENEMY_DAMAGE_DATA: usize = 0x16000;

const OVERWORLD_OFFSET_BASE_X: [u16; 64] = [
    0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00,
    0, 0x200, 0x400, 0x600, 0x800, 0xa00, 0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00,
    0xc00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x200, 0x400, 0x600, 0x800, 0xa00,
    0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00,
    0xa00, 0xe00,
];
const OVERWORLD_OFFSET_BASE_Y: [u16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x200, 0, 0, 0, 0, 0x200, 0x400, 0x400, 0x400, 0x400, 0x400,
    0x400, 0x400, 0x400, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600,
    0x800, 0x600, 0x600, 0x800, 0x600, 0x600, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00,
    0xa00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xe00, 0xe00,
    0xe00, 0xc00, 0xc00, 0xe00,
];

#[must_use]
pub fn offset_base(area: usize) -> (u16, u16) {
    (OVERWORLD_OFFSET_BASE_X[area], OVERWORLD_OFFSET_BASE_Y[area])
}
const OW_SCROLL_VARS0: usize = 0x600;
const OW_SCROLL_VARS1: usize = 0x608;
const UP_DOWN_SCROLL_TARGET: usize = 0x610;
const UP_DOWN_SCROLL_TARGET_END: usize = 0x612;
const LEFT_RIGHT_SCROLL_TARGET: usize = 0x614;
const LEFT_RIGHT_SCROLL_TARGET_END: usize = 0x616;
const LINK_SWORD_TYPE: usize = 0xf359;
const OVERWORLD_MUSIC: usize = 0x15b00;
const SUBMODULE: usize = 0x11;
const PLAYER_IS_INDOORS: usize = 0x1b;
const LINK_VISIBILITY: usize = 0x4b;
const LINK_IS_BUNNY: usize = 0x56;
const LINK_PLAYER_STATE: usize = 0x5d;
const LINK_IS_BUNNY_MIRROR: usize = 0x2e0;
const LINK_IS_RUNNING: usize = 0x372;
const LINK_TIMER_TEMP_BUNNY: usize = 0x3f5;
const LINK_NEED_TRANSFORM_POOF: usize = 0x3f7;
const ENHANCED_FEATURES0: usize = 0x64c;
const LINK_ITEM_MOON_PEARL: usize = 0xf357;
const LINK_HEALTH_CURRENT: usize = 0xf36d;
const MAIN_MODULE: usize = 0x10;
const DUNGEON_ROOM: usize = 0xa0;
const OVERWORLD_SCREEN_INDEX: usize = 0x8a;
const LINK_DIRECTION: usize = 0x67;
const ALLOW_SCROLL_Z: usize = 0x78;
const OVERLAY_INDEX: usize = 0x8c;
const OVERWORLD_TRANS_DIR: usize = 0x410;
const OVERWORLD_TRANS_DIR2: usize = 0x416;
const OVERWORLD_SCREEN_TRANSITION: usize = 0x418;
const BYTE_069C: usize = 0x69c;
const OW_COUNTDOWN_TRANSITION: usize = 0x69a;
const PEG_PUZZLE_PROGRESS: usize = 0x4c8;
const DOOR_OPEN_CLOSED_COUNTER: usize = 0x692;
const BIG_ROCK_STARTING_ADDRESS: usize = 0x698;
const SOUND_EFFECT_2: usize = 0x12f;
const SAVE_OW_EVENT_INFO: usize = 0xf280;
const ANCILLA_TYPE: usize = 0xc4a;
const WHICH_ENTRANCE: usize = 0x10e;
const BYTE_010F: usize = 0x10f;
const SAVEGAME_IS_DARKWORLD: usize = 0xf3ca;
const SUBSUBMODULE: usize = 0xb0;
const DARKENING_OR_LIGHTENING_SCREEN: usize = 0xc009;
const AUX_PALETTE_BUFFER: usize = 0xc300;
const MAIN_PALETTE_BUFFER: usize = 0xc500;
const MAPBAK_PALETTE: usize = 0x1dd80;
const TS_COPY: usize = 0x1d;
const HDMAEN_COPY: usize = 0x9b;
const BG2HOFS_COPY2: usize = 0xe2;
const MIRROR_VARS: usize = 0x6a0;
const HDMA_TABLE_DYNAMIC: usize = 0x1dba0;
const FRAME_COUNTER: usize = 0x1a;
const BG1HOFS_COPY2: usize = 0xe0;
const BG1VOFS_COPY2: usize = 0xe6;
const BG2VOFS_COPY2: usize = 0xe8;
const BG2HOFS_COPY: usize = 0x11e;
const BG1HOFS_COPY: usize = 0x120;
const BG2VOFS_COPY: usize = 0x122;
const BG1VOFS_COPY: usize = 0x124;
const FLAG_UPDATE_CGRAM: usize = 0x15;
const TRIGGER_SPECIAL_ENTRANCE: usize = 0x4c6;
const NMI_DISABLE_CORE_UPDATES: usize = 0x710;
const SOUND_EFFECT_AMBIENT: usize = 0x12d;
const OVERWORLD_MAP_STATE: usize = 0x200;
const COLDATA_COPY2: usize = 0x9e;
const COLDATA_COPY0: usize = 0x9c;
const COLDATA_COPY1: usize = 0x9d;
const MUSIC_CONTROL: usize = 0x12c;
const LINK_Y_VELOCITY: usize = 0x30;
const LINK_X_VELOCITY: usize = 0x31;
const LINK_INCAPACITATED_TIMER: usize = 0x46;
const SET_WHEN_DAMAGING_ENEMIES: usize = 0x47;
const LINK_Y_CACHED: usize = 0xc184;
const LINK_X_CACHED: usize = 0xc186;
const ROOM_SCROLL_VARS_CACHED: usize = 0xc188;
const SCROLL_TARGETS_CACHED: usize = 0xc190;
const CAMERA_SCROLL_CACHED: usize = 0xc198;
const QUADRANTS_CACHED: usize = 0xc19c;
const LINK_DIRECTION_CACHED: usize = 0xc1a6;
const MAPBAK_TM: usize = 0xc211;
const MAPBAK_TS: usize = 0xc212;
const NMI_LOAD_BG_FROM_VRAM: usize = 0x14;
const FLAG_LINK_IMMOBILIZED: usize = 0x2e4;
const FLAG_UNK1: usize = 0xfc1;
const BIRD_TRAVEL_VAR1: usize = 0x1af0;
const MAP16_LOAD_SRC_OFFSET: usize = 0x84;
const MAP16_LOAD_DST_OFFSET: usize = 0x86;
const MAP16_LOAD_VAR2: usize = 0x88;
const MAP16_ROW_BUFFER: usize = 0x500;
const DUNG_BG2_ATTR_TABLE: usize = 0x12000;
const MAP16_STRIPE_HEADERS: usize = 0x14000;
const NMI_SUBROUTINE_INDEX: usize = 0x17;
const INIDISP_COPY: usize = 0x13;
const OW_ENTRANCE_VALUE: usize = 0x696;
const ORANGE_BLUE_BARRIER_STATE: usize = 0xc172;
const WORD_C174: usize = 0xc174;
const WORD_C176: usize = 0xc176;
const UVRAM_DATA: usize = 0x1100;
const CAMERA_SCROLL_LOW: usize = 0x618;
const CAMERA_SCROLL_HIGH: usize = 0x61a;
const BG1HOFS_SUBPIXEL: usize = 0x620;
const BG1VOFS_SUBPIXEL: usize = 0x622;
const OVERWORLD_UNK1: usize = 0x624;
const TRANSITION_COUNTER: usize = 0x126;
const CURRENT_AREA_OF_PLAYER: usize = 0x700;
const OVERWORLD_AREA_IS_BIG: usize = 0x712;
const FLAG_OVERWORLD_AREA_DID_CHANGE: usize = 0xabf;
const BYTE_069E: usize = 0x69e;
const MUSIC_UNK1: usize = 0x130;
const QUADRANT_FULLSIZE_X: usize = 0xa6;
const LINK_QUADRANT_X: usize = 0xa9;
const LINK_IS_ON_LOWER_LEVEL: usize = 0xee;
const SAVED_MODULE_FOR_MENU: usize = 0x10c;
const TAGALONG_VAR5: usize = 0x2f9;
const COUNTDOWN_FOR_BLINK: usize = 0x31f;
const LINK_DISABLE_SPRITE_DAMAGE: usize = 0x37b;
const LINK_IS_ON_LOWER_LEVEL_MIRROR: usize = 0x476;
const DUNG_FLAG_STATECHANGE_WATERPUZZLE: usize = 0x642;
const IS_STANDING_IN_DOORWAY: usize = 0x6c;
const DUNG_CUR_FLOOR: usize = 0xa4;
const TM_COPY: usize = 0x1c;
const BUTTON_MASK_B_Y: usize = 0x3a;
const BUTTON_B_FRAMES: usize = 0x3c;
const LINK_FORCE_HOLD_SWORD_UP: usize = 0x3ef;
const EXIT_CACHE: usize = 0xc140;
const BYTE_0AA0: usize = 0xaa0;
const MAIN_TILE_THEME_INDEX: usize = 0xaa1;
const AUX_TILE_THEME_INDEX: usize = 0xaa2;
const SPRITE_GRAPHICS_INDEX: usize = 0xaa3;
const TILEMAP_LOCATION_CALC_MASK: usize = 0xec;
const OVERWORLD_RIGHT_BOTTOM_BOUND: usize = 0x716;
const LINK_QUADRANT_Y: usize = 0xaa;
const QUADRANT_FULLSIZE_Y: usize = 0xa7;
const PLAYER_OAM_Y_OFFSET: usize = 0x44;
const PLAYER_OAM_X_OFFSET: usize = 0x45;
const LINK_DIRECTION_MASK_A: usize = 0x42;
const LINK_DIRECTION_MASK_B: usize = 0x43;
const LINK_ACTUAL_VELOCITY_Z: usize = 0x29;
const OVERWORLD_ENTRANCE_SEQUENCE_COUNTER: usize = 0xc8;
const BG1_X_OFFSET: usize = 0x11a;
const BG1_Y_OFFSET: usize = 0x11c;
const MOSAIC_TARGET_LEVEL: usize = 0xc00b;
const LINK_MAYBE_SWIM_FASTER: usize = 0x32a;
const LAST_MUSIC_CONTROL: usize = 0x133;
const LINK_STATE_BITS: usize = 0x308;
const LINK_PICKING_THROW_STATE: usize = 0x309;
const IS_NMI_THREAD_ACTIVE: usize = 0x12a;
const NMI_FLAG_UPDATE_POLYHEDRAL: usize = 0x1f0c;
const TMW_COPY: usize = 0x1e;
const TSW_COPY: usize = 0x1f;
const W12SEL_COPY: usize = 0x96;
const W34SEL_COPY: usize = 0x97;
const WOBJSEL_COPY: usize = 0x98;
const DOOR_ANIMATION_STEP: usize = 0x690;
const FLAG_UPDATE_HUD: usize = 0x16;
const HUD_FLOOR_CHANGED_TIMER: usize = 0x4a0;
const LINK_DIRECTION_LAST: usize = 0x26;
const LINK_SPEED_SETTING: usize = 0x5e;
const DRAW_WATER_RIPPLES_OR_GRASS: usize = 0x351;
const OVERLAY_STATE_CACHE: usize = 0xc213;
const CGWSEL_COPY: usize = 0x99;
const CGADSUB_COPY: usize = 0x9a;
const SPECIAL_EXIT_CACHE: usize = 0xc100;
const WORD_0486: usize = 0x486;
const WORD_0488: usize = 0x488;
const MOVE_OVERLAY_COUNTER: usize = 0x494;
const INTERACTION_SCRATCH_X: usize = 0x72;
const INTERACTION_SCRATCH_Y: usize = 0x74;
const INDEX_OF_INTERACTING_TILE: usize = 0x76;
const LINK_POSITION_MODE: usize = 0x37a;
const INTERACTING_TILE_POSITION: usize = 0x4b2;
const LINK_CANT_CHANGE_DIRECTION: usize = 0x50;
const QUEUED_MUSIC_CONTROL: usize = 0x132;
const DUNGEON_UNK6: usize = 0x3f4;
const DUNGEON_SAVEGAME_STATE_BITS: usize = 0x402;
const CURRENT_PALACE_INDEX_X2: usize = 0x40c;
const DUNGEON_DARK_WITH_LANTERN: usize = 0x458;
const DUNGEON_HEADER_COLLISION: usize = 0x46c;
const DUNGEON_WANT_LIGHTS_OUT: usize = 0xc005;
const OVERWORLD_FIXED_COLOR_PLUS_MINUS: usize = 0xc017;
const SAVEGAME_MASTER_SWORD_FLAGS: usize = 0xf300;
const LINK_NUM_KEYS: usize = 0xf36f;
const INCREMENTAL_VRAM_COUNTER: usize = 0x412;
const OVERWORLD_AREA_IS_BIG_BACKUP: usize = 0x714;
const MISC_SPRITE_GRAPHICS_INDEX: usize = 0xaa4;
const LINK_AUXILIARY_STATE: usize = 0x4d;
const LINK_POSE_FOR_ITEM: usize = 0x2da;
const BIG_KEY_DOOR_MESSAGE_TRIGGERED: usize = 0x4b8;
const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;
const FOLLOWER_DROPPED: usize = 0xf3d3;
const JOYPAD_H_LAST: usize = 0xf0;
const FILTERED_JOYPAD_H: usize = 0xf4;
const FLAG_CUSTOM_SPELL_ACTIVE: usize = 0x112;
const SUPER_BOMB_INDICATOR_UNK2: usize = 0x4b4;
const FLAG_BLOCK_LINK_MENU: usize = 0xffc;
const VARIOUS_PACKS: [u8; 16] = [
    0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x5b, 0x01, 0x5a, 0x42, 0x43, 0x44, 0x45, 0x3f, 0x59, 0x0b, 0x5a,
];
const SWITCH_AREA_MASK: [u16; 4] = [0xf80, 0xf80, 0x3f, 0x3f];
const SWITCH_AREA_DELTA: [i16; 4] = [2, -2, 16, -16];
const OVERWORLD_AREA_HEADS: [u8; 64] = [
    0, 0, 2, 3, 3, 5, 5, 7, 0, 0, 10, 3, 3, 5, 5, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 24, 26,
    27, 27, 29, 30, 30, 24, 24, 34, 27, 27, 37, 30, 30, 40, 41, 42, 43, 44, 45, 46, 47, 48, 48, 50,
    51, 52, 53, 53, 55, 48, 48, 58, 59, 60, 53, 53, 63,
];
const SWITCH_AREA_SOURCE_OFFSET: [u16; 256] = [
    0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x1060, 0x1060, 0x1060,
    0x1060, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60,
    0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x1060, 0x1060, 0x60, 0x1060, 0x1060, 0x60, 0x60, 0x60,
    0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60,
    0x60, 0x60, 0x1060, 0x1060, 0x60, 0x80, 0x80, 0x40, 0x80, 0x80, 0x80, 0x80, 0x40, 0x1080,
    0x1080, 0x40, 0x1080, 0x1080, 0x1080, 0x1080, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x80, 0x80, 0x40, 0x80, 0x80, 0x40, 0x80, 0x80, 0x1080, 0x1080, 0x40, 0x1080, 0x1080,
    0x40, 0x1080, 0x1080, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x80, 0x80, 0x40, 0x40,
    0x40, 0x80, 0x80, 0x40, 0x1080, 0x1080, 0x40, 0x40, 0x40, 0x1080, 0x1080, 0x40, 0x1800, 0x1840,
    0x1800, 0x1800, 0x1840, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800,
    0x1840, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1840,
    0x1800, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800,
    0x1800, 0x1840, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1840,
    0x1800, 0x1800, 0x1800, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800, 0x1800, 0x1800, 0x1800,
    0x1840, 0x1800, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040,
    0x1000, 0x2000, 0x2040, 0x2000, 0x2040, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000,
    0x1000, 0x1000, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x2000, 0x2040,
    0x1000, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000,
    0x1000, 0x1000, 0x2000, 0x2040, 0x1000, 0x1000, 0x1000, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040,
    0x1000, 0x1000, 0x1000, 0x2000, 0x2040, 0x1000,
];

const BEST_ACTION_X: [i8; 4] = [7, 7, -3, 16];
const BEST_ACTION_Y: [i8; 4] = [6, 24, 12, 12];

const UP_DOWN_SCROLL_TARGETS: [u16; 64] = [
    0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0x120, 0xff20,
    0xff20, 0xff20, 0xff20, 0x120, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x520,
    0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x720, 0x520, 0x520, 0x720,
    0x520, 0x520, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0xb20, 0xb20, 0xb20,
    0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xd20, 0xd20, 0xd20, 0xb20, 0xb20, 0xd20,
];
const LEFT_RIGHT_SCROLL_TARGETS: [u16; 64] = [
    0xff00, 0xff00, 0x300, 0x500, 0x500, 0x900, 0x900, 0xd00, 0xff00, 0xff00, 0x300, 0x500, 0x500,
    0x900, 0x900, 0xd00, 0xff00, 0x100, 0x300, 0x500, 0x700, 0x900, 0xb00, 0xd00, 0xff00, 0xff00,
    0x300, 0x500, 0x500, 0x900, 0xb00, 0xb00, 0xff00, 0xff00, 0x300, 0x500, 0x500, 0x900, 0xb00,
    0xb00, 0xff00, 0x100, 0x300, 0x500, 0x700, 0x900, 0xb00, 0xd00, 0xff00, 0xff00, 0x300, 0x500,
    0x700, 0x900, 0x900, 0xd00, 0xff00, 0xff00, 0x300, 0x500, 0x700, 0x900, 0x900, 0xd00,
];
const SPECIAL_SWITCH_MAP8: [u16; 4] = [0x105, 0x1e4, 0xad, 0xb9];
const SPECIAL_SWITCH_SCREEN: [u16; 4] = [0, 45, 15, 129];
const SPECIAL_SWITCH_DIRECTION: [u8; 4] = [8, 2, 8, 8];
const SPECIAL_SWITCH_EXIT: [u16; 4] = [0x180, 0x181, 0x182, 0x189];
const SPECIAL_SWITCH_B_MAP8: [u16; 3] = [0x17c, 0x1e4, 0x0ad];
const SPECIAL_SWITCH_B_SCREEN: [u16; 3] = [0x80, 0x80, 0x81];
const SPECIAL_SWITCH_B_DIRECTION: [u8; 3] = [4, 1, 4];
const DOOR_ANIMATION_TILES: [u16; 56] = [
    0xda8, 0xda9, 0xdaa, 0xdab, 0xdac, 0xdad, 0xdae, 0xdaf, 0xdb0, 0xdb1, 0xdb2, 0xdb3, 0xdb6,
    0xdb7, 0xdb8, 0xdb9, 0xdba, 0xdbb, 0xdbc, 0xdbd, 0xdcd, 0xdce, 0xdcf, 0xdd0, 0xdd3, 0xdd4,
    0xdd5, 0xdd6, 0xdd7, 0xdd8, 0xdd9, 0xdda, 0xdd1, 0xdd2, 0xdd3, 0xdd4, 0xdd1, 0xdd2, 0xdd7,
    0xdd8, 0x918, 0x919, 0x91a, 0x91b, 0xddb, 0xddc, 0xddd, 0xdde, 0xdd1, 0xdd2, 0xddb, 0xddc,
    0xe21, 0xe22, 0xe23, 0xe24,
];
const LW_TURTLE_ROCK_PEG_POSITIONS: [u16; 3] = [0x826, 0x5a0, 0x81a];
const DW_PALETTE_ANIM: [u16; 35] = [
    0x884, 0xcc7, 0x150a, 0x154d, 0x7ff6, 0x5944, 0x7ad1, 0x884, 0xcc7, 0x150a, 0x154d, 0x5bff,
    0x7ad1, 0x21af, 0x1084, 0x48c0, 0x6186, 0x7e6d, 0x7fe0, 0x5944, 0x7e20, 0x1084, 0x000e, 0x1059,
    0x291f, 0x7fe0, 0x5944, 0x7e20, 0x1084, 0x1508, 0x196c, 0x21af, 0x7ff6, 0x1d4c, 0x7ad1,
];
const DW_PALETTE_ANIM2: [u16; 40] = [
    0x7fff, 0x884, 0x1cc8, 0x1dce, 0x3694, 0x4718, 0x1d4a, 0x18ac, 0x7fff, 0x1908, 0x2d2f, 0x3614,
    0x4eda, 0x471f, 0x1d4a, 0x390f, 0x7fff, 0x34cd, 0x5971, 0x5635, 0x7f1b, 0x7fff, 0x1d4a, 0x3d54,
    0x7fff, 0x1908, 0x2d2f, 0x3614, 0x4eda, 0x471f, 0x1d4a, 0x390f, 0x7fff, 0x884, 0x52a, 0x21ef,
    0x3ab5, 0x4b39, 0x1d4c, 0x18ac,
];
const MISERY_MIRE_ENTRANCE_BITS: [u8; 26] = [
    0xff, 0xf7, 0xf7, 0xfb, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
    0xaa, 0x88, 0x88, 0x88, 0x88, 0x80, 0x80, 0x80, 0x80, 0x80,
];
const SECONDARY_OVERLAY_PER_OW: [u16; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1c0c, 0x1c0c, 0, 0,
    0, 0, 0, 0, 0x1c0c, 0x1c0c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x3b0,
    0x180c, 0x180c, 0x288, 0, 0, 0, 0, 0, 0x180c, 0x180c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1ab6, 0x1ab6, 0, 0xe2e, 0xe2e, 0, 0, 0, 0x1ab6, 0x1ab6, 0,
    0xe2e, 0xe2e, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x3b0, 0, 0, 0x288, 0, 0, 0, 0, 0,
    0, 0, 0,
];
const OVERWORLD_DRAW_STRIP: [u16; 3] = [0x3d0, 0x410, 0xf410];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point16U {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MirrorHdmaSetup {
    pub channel6_address: u32,
    pub channel7_address: u32,
    pub transfer_unit: u8,
    pub channel6_register: u8,
    pub channel7_register: u8,
    pub indirect_bank: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntranceAnimation {
    PalaceOfDarkness,
    SkullWoods,
    MiseryMire,
    TurtleRock,
    GanonsTower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapStripeAxis {
    X,
    Y,
}

const OVERWORLD_ENTRANCE_TAB0: [u16; 44] = [
    0xfe, 0xc5, 0xfe, 0x114, 0x115, 0x175, 0x156, 0xf5, 0xe2, 0x1ef, 0x119, 0xfe, 0x172, 0x177,
    0x13f, 0x172, 0x112, 0x161, 0x172, 0x14c, 0x156, 0x1ef, 0xfe, 0xfe, 0xfe, 0x10b, 0x173, 0x143,
    0x149, 0x175, 0x103, 0x100, 0x1cc, 0x15e, 0x167, 0x128, 0x131, 0x112, 0x16d, 0x163, 0x173,
    0xfe, 0x113, 0x177,
];

const OVERWORLD_ENTRANCE_TAB1: [u16; 44] = [
    0x14a, 0xc4, 0x14f, 0x115, 0x114, 0x174, 0x155, 0xf5, 0xee, 0x1eb, 0x118, 0x146, 0x171, 0x155,
    0x137, 0x174, 0x173, 0x121, 0x164, 0x155, 0x157, 0x128, 0x114, 0x123, 0x113, 0x109, 0x118,
    0x161, 0x149, 0x117, 0x174, 0x101, 0x1cc, 0x131, 0x51, 0x14e, 0x131, 0x112, 0x17a, 0x163,
    0x172, 0x1bd, 0x152, 0x167,
];

/// Asset views used by the translated overworld helpers.
pub trait OverworldAssets {
    fn entrance_areas(&self) -> &[u16];
    fn entrance_positions(&self) -> &[u16];
    fn sign_text(&self) -> &[u16];
    fn background_palettes(&self) -> &[u8];
    fn sprite_graphics(&self) -> &[u8];
    fn sprite_palettes(&self) -> &[u8];
    fn sprite_offsets(&self) -> &[u16];
    fn sprites(&self) -> &[u8];
    fn map16_to_map8(&self) -> &[u16];
    fn map8_tile_attributes(&self) -> &[u8];
    fn enemy_damage_data(&self) -> &[u8];
    fn overworld_music_sets(&self) -> &[u8];
    fn overworld_music_sets2(&self) -> &[u8];
    fn fall_hole_areas(&self) -> &[u16];
    fn fall_hole_positions(&self) -> &[u16];
    fn fall_hole_entrances(&self) -> &[u8];
    fn whirlpool_areas(&self) -> &[u16];
    fn map32_to_map16(&self) -> [&[u8]; 4] {
        [&[], &[], &[], &[]]
    }
    fn overworld_hibytes(&self, _screen: usize) -> &[u8] {
        &[]
    }
    fn overworld_lobytes(&self, _screen: usize) -> &[u8] {
        &[]
    }
    fn overworld_map_is_small(&self) -> &[u8] {
        &[]
    }
    fn overworld_secrets(&self) -> &[u8] {
        &[]
    }
    fn overworld_secret_offsets(&self) -> &[u16] {
        &[]
    }
    fn exit_screen_indices(&self) -> &[u8] {
        &[]
    }
    fn exit_rooms(&self) -> &[u16] {
        &[]
    }
    fn exit_map16_source_offsets(&self) -> &[u16] {
        &[]
    }
    fn exit_scroll_x(&self) -> &[u16] {
        &[]
    }
    fn exit_scroll_y(&self) -> &[u16] {
        &[]
    }
    fn exit_link_x(&self) -> &[u16] {
        &[]
    }
    fn exit_link_y(&self) -> &[u16] {
        &[]
    }
    fn exit_camera_x(&self) -> &[u16] {
        &[]
    }
    fn exit_camera_y(&self) -> &[u16] {
        &[]
    }
    fn exit_normal_doors(&self) -> &[u16] {
        &[]
    }
    fn exit_fancy_doors(&self) -> &[u16] {
        &[]
    }
    fn exit_unk1(&self) -> &[i8] {
        &[]
    }
    fn exit_unk3(&self) -> &[i8] {
        &[]
    }
    fn bird_scroll_x(&self) -> &[u16] {
        &[]
    }
    fn bird_scroll_y(&self) -> &[u16] {
        &[]
    }
    fn bird_link_x(&self) -> &[u16] {
        &[]
    }
    fn bird_link_y(&self) -> &[u16] {
        &[]
    }
    fn bird_camera_x(&self) -> &[u16] {
        &[]
    }
    fn bird_camera_y(&self) -> &[u16] {
        &[]
    }
    fn bird_map16_source_offsets(&self) -> &[u16] {
        &[]
    }
    fn bird_screen_indices(&self) -> &[u8] {
        &[]
    }
    fn bird_unk1(&self) -> &[i8] {
        &[]
    }
    fn bird_unk3(&self) -> &[i8] {
        &[]
    }
    fn overworld_aux_tile_themes(&self) -> &[u8] {
        &[]
    }
    fn entrance_ids(&self) -> &[u8] {
        &[]
    }
    fn special_exit_top(&self) -> &[u16] {
        &[]
    }
    fn special_exit_bottom(&self) -> &[u16] {
        &[]
    }
    fn special_exit_left(&self) -> &[u16] {
        &[]
    }
    fn special_exit_right(&self) -> &[u16] {
        &[]
    }
    fn special_exit_up_target(&self) -> &[i16] {
        &[]
    }
    fn special_exit_down_target(&self) -> &[i16] {
        &[]
    }
    fn special_exit_left_target(&self) -> &[i16] {
        &[]
    }
    fn special_exit_right_target(&self) -> &[i16] {
        &[]
    }
    fn special_exit_left_edge(&self) -> &[u16] {
        &[]
    }
    fn special_exit_direction(&self) -> &[u8] {
        &[]
    }
    fn special_exit_sprite_gfx(&self) -> &[u8] {
        &[]
    }
    fn special_exit_aux_gfx(&self) -> &[u8] {
        &[]
    }
    fn special_exit_bg_palette(&self) -> &[u8] {
        &[]
    }
    fn special_exit_sprite_palette(&self) -> &[u8] {
        &[]
    }
}

/// Cross-translation-unit calls made while entering the overworld. The WRAM
/// mutations owned by `overworld.c` stay in the translated routine below;
/// graphics, HUD, sprite, follower, and palette entry points remain explicit.
pub trait PreOverworldHooks {
    fn load_from_dungeon(&mut self, ram: &mut Ram<'_>);
    fn load_from_special_overworld(&mut self, ram: &mut Ram<'_>);
    fn hud_refill_logic(&mut self, ram: &mut Ram<'_>);
    fn decompress_animated_tiles(&mut self, ram: &mut Ram<'_>, tiles: u8);
    fn initialize_tilesets(&mut self, ram: &mut Ram<'_>);
    fn load_screen_palette_set(&mut self, ram: &mut Ram<'_>);
    fn load_palettes(&mut self, ram: &mut Ram<'_>, background: u8, sprites: u8);
    fn set_overworld_background_color(&mut self, ram: &mut Ram<'_>);
    fn load_palettes_inner(&mut self, ram: &mut Ram<'_>);
    fn copy_special_palettes_to_cache(&mut self, ram: &mut Ram<'_>);
    fn initialize_follower(&mut self, ram: &mut Ram<'_>);
    fn decode_animated_sprite_tile(&mut self, ram: &mut Ram<'_>, tile: u8);
    fn reload_overworld_sprites(&mut self, ram: &mut Ram<'_>);
    fn initialize_mirror_portal(&mut self, ram: &mut Ram<'_>);
    fn reset_torch_background_and_player(&mut self, ram: &mut Ram<'_>);
    fn load_bunny_palettes(&mut self, ram: &mut Ram<'_>);
    fn load_overworld_music_if_needed(&mut self, ram: &mut Ram<'_>);
}

pub trait PlayerControlHooks {
    fn did_press_map(&mut self, ram: &mut Ram<'_>) -> bool;
    fn display_select_menu(&mut self, ram: &mut Ram<'_>);
    fn handle_item_switch_inputs(&mut self, ram: &mut Ram<'_>);
    fn animate_entrance(&mut self, ram: &mut Ram<'_>);
    fn link_main(&mut self, ram: &mut Ram<'_>);
    fn super_bomb_indicator(&mut self, ram: &mut Ram<'_>);
    fn load_chr_half_slot(&mut self, ram: &mut Ram<'_>);
    fn operate_camera_scroll(&mut self, ram: &mut Ram<'_>);
    fn use_entrance(&mut self, ram: &mut Ram<'_>);
    fn animate_dark_world_palette(&mut self, ram: &mut Ram<'_>);
    fn handle_transitions(&mut self, ram: &mut Ram<'_>);
    fn scroll_and_check_special_exit(&mut self, ram: &mut Ram<'_>);
}

pub trait TransitionHooks {
    fn check_edge_screen_transition(&mut self, ram: &mut Ram<'_>) -> bool;
    fn reset_torch_background_and_player(&mut self, ram: &mut Ram<'_>);
    fn is_playing_music_track(&mut self, track: u8) -> bool;
    fn load_palettes(&mut self, ram: &mut Ram<'_>, background: u8, sprites: u8);
    fn copy_palettes_to_cache(&mut self, ram: &mut Ram<'_>);
}

#[must_use]
pub fn lookup_in_ow_entrance_tab(r0: u16, r2: u16) -> bool {
    OVERWORLD_ENTRANCE_TAB0
        .iter()
        .zip(OVERWORLD_ENTRANCE_TAB1.iter())
        .rev()
        .any(|(&a, &b)| r0 == a && r2 == b)
}

/// Returns the last matching entrance, matching the C loop from index 128 to 0.
#[must_use]
pub fn lookup_in_ow_entrance_tab2(
    ram: &Ram<'_>,
    assets: &impl OverworldAssets,
    position: u16,
) -> Option<usize> {
    let area = ram.read_u16(OVERWORLD_AREA_INDEX);
    let positions = assets.entrance_positions();
    let areas = assets.entrance_areas();
    let last = 128usize.min(positions.len().saturating_sub(1));
    if positions.is_empty() || areas.len() <= last {
        return None;
    }
    (0..=last)
        .rev()
        .find(|&index| positions[index] == position && areas[index] == area)
}

#[must_use]
pub fn can_enter_with_tagalong(ram: &Ram<'_>, entrance: i32) -> bool {
    let follower = ram.read_u8(FOLLOWER_INDICATOR);
    matches!(follower, 0 | 1 | 5 | 14) || matches!(follower, 7 | 8) && entrance >= 59
}

/// Converts the original one-hot direction mask (8, 4, 2, 1) to 3, 2, 1, 0.
#[must_use]
pub fn dir_to_enum(mut direction: u8) -> u8 {
    let mut result = 3;
    while direction & 1 == 0 {
        result -= 1;
        direction >>= 1;
    }
    result
}

pub fn reset_mosaic_down(ram: &mut Ram<'_>) {
    let mut level = ram.read_u8(MOSAIC_LEVEL);
    if ram.read_u16(PALETTE_FILTER_COUNTDOWN) & 1 != 0 {
        level = level.wrapping_sub(0x10);
        ram.write_u8(MOSAIC_LEVEL, level);
    }
    ram.write_u8(BGMODE, 9);
    ram.write_u8(MOSAIC, level | 7);
}

pub fn overworld_func1d() -> ! {
    panic!("Overworld_Func1D is an unreachable assertion stub in the C port")
}

pub fn overworld_func1e() -> ! {
    panic!("Overworld_Func1E is an unreachable assertion stub in the C port")
}

pub fn conditional_mosaic_control(ram: &mut Ram<'_>) {
    let mut level = ram.read_u8(MOSAIC_LEVEL);
    if ram.read_u16(PALETTE_FILTER_COUNTDOWN) & 1 != 0 {
        level = level.wrapping_add(0x10);
        ram.write_u8(MOSAIC_LEVEL, level);
    }
    ram.write_u8(BGMODE, 9);
    ram.write_u8(MOSAIC, level | 7);
}

pub fn reset_mosaic_always_increase(ram: &mut Ram<'_>) {
    let level = ram.read_u8(MOSAIC_LEVEL).wrapping_add(0x10);
    ram.write_u8(MOSAIC_LEVEL, level);
    ram.write_u8(BGMODE, 9);
    ram.write_u8(MOSAIC, level | 7);
}

pub fn mosaic_transition_load_sprite_graphics(
    ram: &mut Ram<'_>,
    mut load_new_sprite_gfx_set: impl FnMut(&mut Ram<'_>),
) {
    load_new_sprite_gfx_set(ram);
    ram.write_u8(INIDISP_COPY, 0x0f);
    ram.write_u8(HDMAEN_COPY, 0x80);
    ram.write_u8(
        PALETTE_FILTER_COUNTDOWN,
        ram.read_u8(MOSAIC_TARGET_LEVEL).wrapping_sub(1),
    );
    ram.write_u8(MOSAIC_TARGET_LEVEL, 0);
    ram.write_u8(DARKENING_OR_LIGHTENING_SCREEN, 2);
    ram.write_u8(SUBSUBMODULE, ram.read_u8(SUBSUBMODULE).wrapping_add(1));
}

#[allow(clippy::too_many_arguments)]
pub fn start_mosaic_transition(
    ram: &mut Ram<'_>,
    mut conditional_mosaic_control: impl FnMut(&mut Ram<'_>),
    mut is_playing_music_track: impl FnMut(u8) -> bool,
    mut reset_transition_props_and_interface: impl FnMut(&mut Ram<'_>),
    mut apply_palette_filter_bounce: impl FnMut(&mut Ram<'_>),
    mut decode_animated_sprite_tile: impl FnMut(&mut Ram<'_>, u8),
    mut load_from_special_overworld: impl FnMut(&mut Ram<'_>),
) {
    conditional_mosaic_control(ram);
    match ram.read_u8(SUBSUBMODULE) {
        0 => {
            let screen = ram.read_u8(OVERWORLD_SCREEN_INDEX);
            if screen != 0x80 {
                let music = ram.read_u8(OVERWORLD_MUSIC + usize::from(screen)) & 0x0f;
                if !is_playing_music_track(music) {
                    ram.write_u8(MUSIC_CONTROL, 0xf1);
                }
            }
            reset_transition_props_and_interface(ram);
        }
        1 => apply_palette_filter_bounce(ram),
        _ => {
            ram.write_u8(INIDISP_COPY, 0x80);
            ram.write_u8(SUBSUBMODULE, 0);
            if ram.read_u16(OVERWORLD_SCREEN_INDEX) & 0x3f == 0 {
                decode_animated_sprite_tile(ram, 0x1e);
            }
            if ram.read_u8(OVERWORLD_AREA_INDEX) != 0 && ram.read_u8(MAIN_MODULE) != 11 {
                ram.write_u8(TM_COPY, 0x16);
                ram.write_u8(TS_COPY, 1);
                ram.write_u8(CGWSEL_COPY, 0x82);
                ram.write_u8(CGADSUB_COPY, 0x20);
                ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
                return;
            }
            if ram.read_u8(SUBMODULE) == 36 {
                load_from_special_overworld(ram);
                if ram.read_u16(OVERWORLD_SCREEN_INDEX) & 0x3f == 0 {
                    decode_animated_sprite_tile(ram, 0x1e);
                }
            }
            ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
        }
    }
}

pub fn handle_rain_overlay(ram: &mut Ram<'_>) {
    let screen = ram.read_u8(OVERWORLD_SCREEN_INDEX);
    if (screen != 0x70 && ram.read_u8(SRAM_PROGRESS_INDICATOR) >= 2)
        || ram.read_u8(SAVE_OW_EVENT_INFO + 0x70) & 0x20 != 0
    {
        return;
    }
    match ram.read_u8(FRAME_COUNTER) {
        3 | 88 => ram.write_u8(CGADSUB_COPY, 0x32),
        5 | 44 | 90 => ram.write_u8(CGADSUB_COPY, 0x72),
        36 => {
            ram.write_u8(SOUND_EFFECT_1, 54);
            ram.write_u8(CGADSUB_COPY, 0x32);
        }
        _ => {}
    }
    if ram.read_u8(FRAME_COUNTER) & 3 != 0 {
        return;
    }
    let index = ram.read_u8(MOVE_OVERLAY_COUNTER).wrapping_add(1) & 3;
    ram.write_u8(MOVE_OVERLAY_COUNTER, index);
    const X: [u16; 4] = [1, 0, 1, 0];
    const Y: [u16; 4] = [0, 17, 0, 17];
    ram.write_u16(
        BG1HOFS_COPY2,
        ram.read_u16(BG1HOFS_COPY2)
            .wrapping_add(X[usize::from(index)] << 8),
    );
    ram.write_u16(
        BG1VOFS_COPY2,
        ram.read_u16(BG1VOFS_COPY2)
            .wrapping_add(Y[usize::from(index)] << 8),
    );
}

#[allow(clippy::too_many_arguments)]
pub fn module09_overworld(
    ram: &mut Ram<'_>,
    mut dispatch_submodule: impl FnMut(&mut Ram<'_>),
    mut sprite_main: impl FnMut(&mut Ram<'_>),
    mut link_oam_main: impl FnMut(&mut Ram<'_>),
    mut hud_refill_logic: impl FnMut(&mut Ram<'_>),
) {
    dispatch_submodule(ram);
    let bg2x = ram.read_u16(BG2HOFS_COPY2);
    let bg2y = ram.read_u16(BG2VOFS_COPY2);
    let bg1x = ram.read_u16(BG1HOFS_COPY2);
    let bg1y = ram.read_u16(BG1VOFS_COPY2);
    let x_offset = ram.read_u16(BG1_X_OFFSET);
    let y_offset = ram.read_u16(BG1_Y_OFFSET);
    ram.write_u16(BG2HOFS_COPY2, bg2x.wrapping_add(x_offset));
    ram.write_u16(BG2HOFS_COPY, bg2x.wrapping_add(x_offset));
    ram.write_u16(BG2VOFS_COPY2, bg2y.wrapping_add(y_offset));
    ram.write_u16(BG2VOFS_COPY, bg2y.wrapping_add(y_offset));
    ram.write_u16(BG1HOFS_COPY2, bg1x.wrapping_add(x_offset));
    ram.write_u16(BG1HOFS_COPY, bg1x.wrapping_add(x_offset));
    ram.write_u16(BG1VOFS_COPY2, bg1y.wrapping_add(y_offset));
    ram.write_u16(BG1VOFS_COPY, bg1y.wrapping_add(y_offset));
    sprite_main(ram);
    ram.write_u16(BG2HOFS_COPY2, bg2x);
    ram.write_u16(BG2VOFS_COPY2, bg2y);
    ram.write_u16(BG1HOFS_COPY2, bg1x);
    ram.write_u16(BG1VOFS_COPY2, bg1y);
    link_oam_main(ram);
    hud_refill_logic(ram);
    handle_rain_overlay(ram);
}

pub fn module09_player_control(ram: &mut Ram<'_>, hooks: &mut impl PlayerControlHooks) {
    let controls_blocked = ram.read_u8(FLAG_CUSTOM_SPELL_ACTIVE)
        | ram.read_u8(FLAG_LINK_IMMOBILIZED)
        | ram.read_u8(FLAG_BLOCK_LINK_MENU)
        | ram.read_u8(TRIGGER_SPECIAL_ENTRANCE);
    if controls_blocked == 0 {
        if ram.read_u8(FILTERED_JOYPAD_H) & 0x10 != 0 {
            ram.write_u8(OVERWORLD_MAP_STATE, 0);
            ram.write_u8(SUBMODULE, 1);
            ram.write_u8(SAVED_MODULE_FOR_MENU, ram.read_u8(MAIN_MODULE));
            ram.write_u8(MAIN_MODULE, 14);
            return;
        }
        if hooks.did_press_map(ram) {
            ram.write_u8(OVERWORLD_MAP_STATE, 0);
            ram.write_u8(SUBMODULE, 7);
            ram.write_u8(SAVED_MODULE_FOR_MENU, ram.read_u8(MAIN_MODULE));
            ram.write_u8(MAIN_MODULE, 14);
            return;
        }
        if ram.read_u8(JOYPAD_H_LAST) & 0x20 != 0 {
            hooks.display_select_menu(ram);
            return;
        }
        hooks.handle_item_switch_inputs(ram);
    }
    if ram.read_u8(TRIGGER_SPECIAL_ENTRANCE) != 0 {
        hooks.animate_entrance(ram);
    }
    hooks.link_main(ram);
    if ram.read_u8(SUPER_BOMB_INDICATOR_UNK2) != 0xff {
        hooks.super_bomb_indicator(ram);
    }
    let area = (ram.read_u16(LINK_Y) & 0x1e00) >> 5 | (ram.read_u16(LINK_X) & 0x1e00) >> 8;
    ram.write_u16(CURRENT_AREA_OF_PLAYER, area);
    hooks.load_chr_half_slot(ram);
    hooks.operate_camera_scroll(ram);
    if ram.read_u8(MAIN_MODULE) != 11 {
        hooks.use_entrance(ram);
        hooks.animate_dark_world_palette(ram);
        hooks.handle_transitions(ram);
    } else {
        hooks.scroll_and_check_special_exit(ram);
    }
}

pub fn handle_transitions(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    hooks: &mut impl TransitionHooks,
) {
    if ram.read_u8(OVERWORLD_TRANS_DIR2) != 0 {
        handle_map_scroll(ram, assets);
    }
    let area = usize::from(ram.read_u8(CURRENT_AREA_OF_PLAYER) >> 1);
    let bound = ram.read_u16(OVERWORLD_RIGHT_BOTTOM_BOUND);
    let mut transition = None;
    if ram.read_u8(LINK_Y_VELOCITY) != 0 {
        let direction = ram.read_u8(LINK_DIRECTION) & 12;
        let offset = ram
            .read_u16(LINK_Y)
            .wrapping_sub(OVERWORLD_OFFSET_BASE_Y[area]);
        if offset < 4 {
            transition = Some((direction, 6u8, 8u8));
        } else if offset >= bound {
            transition = Some((direction, 4, 4));
        }
    }
    if ram.read_u8(LINK_X_VELOCITY) != 0 {
        let direction = ram.read_u8(LINK_DIRECTION) & 3;
        let offset = ram
            .read_u16(LINK_X)
            .wrapping_sub(OVERWORLD_OFFSET_BASE_X[area]);
        if offset < 6 {
            transition = Some((direction, 2, 2));
        } else if offset >= bound.wrapping_add(4) {
            transition = Some((direction, 0, 1));
        }
    }
    let Some((direction, y, expected_direction)) = transition else {
        check_special_switch_area(ram, assets);
        return;
    };
    if expected_direction != direction || hooks.check_edge_screen_transition(ram) {
        check_special_switch_area(ram, assets);
        return;
    }
    let transition_index = usize::from(y >> 1);
    hooks.reset_torch_background_and_player(ram);
    let source = ram.read_u16(MAP16_LOAD_SRC_OFFSET) & SWITCH_AREA_MASK[transition_index];
    let pushed = usize::from(
        (ram.read_u16(CURRENT_AREA_OF_PLAYER)
            .wrapping_add_signed(SWITCH_AREA_DELTA[transition_index])
            >> 1)
            & 0x3f,
    );
    ram.write_u16(
        MAP16_LOAD_SRC_OFFSET,
        source.wrapping_add(SWITCH_AREA_SOURCE_OFFSET[transition_index * 64 + pushed]),
    );
    let old_screen = ram.read_u8(OVERWORLD_SCREEN_INDEX);
    if old_screen == 0x2a {
        ram.write_u8(SOUND_EFFECT_AMBIENT, 0x80);
    }
    let new_area = OVERWORLD_AREA_HEADS[pushed] | ram.read_u8(SAVEGAME_IS_DARKWORLD);
    ram.write_u8(OVERWORLD_SCREEN_INDEX, new_area);
    ram.write_u8(OVERWORLD_AREA_INDEX, new_area);
    if ram.read_u8(SAVEGAME_IS_DARKWORLD) == 0 || ram.read_u8(LINK_ITEM_MOON_PEARL) != 0 {
        let music = ram.read_u8(OVERWORLD_MUSIC + usize::from(new_area));
        if music & 0xf0 == 0 {
            ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
        }
        if !hooks.is_playing_music_track(music & 0x0f) {
            ram.write_u8(MUSIC_CONTROL, 0xf1);
        }
    }
    load_gfx_and_screen_size(ram, assets);
    ram.write_u8(SUBMODULE, 1);
    ram.write_u8(OVERWORLD_TRANS_DIR, direction);
    ram.write_u8(OVERWORLD_TRANS_DIR2, direction);
    let direction_enum = dir_to_enum(direction);
    ram.write_u8(BYTE_069C, direction_enum);
    ram.write_u16(OVERWORLD_SCREEN_TRANSITION, u16::from(direction_enum));
    ram.write_u8(OW_ENTRANCE_VALUE, 0);
    ram.write_u8(BIG_ROCK_STARTING_ADDRESS, 0);
    ram.write_u16(TRANSITION_COUNTER, 0);
    if old_screen & 0x3f == 0 || new_area & 0xbf == 0 {
        ram.write_u8(SUBSUBMODULE, 0);
        ram.write_u8(SUBMODULE, 13);
        ram.write_u8(MOSAIC, 0);
        ram.write_u8(MOSAIC_LEVEL, 0);
    } else {
        let background = assets.background_palettes()[usize::from(new_area)];
        let sprites = ram.read_u8(OVERWORLD_SPRITE_PALETTES + usize::from(new_area));
        hooks.load_palettes(ram, background, sprites);
        hooks.copy_palettes_to_cache(ram);
    }
}

pub fn overworld_func1c(
    ram: &mut Ram<'_>,
    mut load_new_sprite_gfx_set: impl FnMut(&mut Ram<'_>),
    mut incremental_vram_upload: impl FnMut(&mut Ram<'_>),
    mut apply_palette_filter_bounce: impl FnMut(&mut Ram<'_>),
) {
    reset_mosaic_down(ram);
    match ram.read_u8(SUBSUBMODULE) {
        0 => mosaic_transition_load_sprite_graphics(ram, &mut load_new_sprite_gfx_set),
        1 => {
            incremental_vram_upload(ram);
            apply_palette_filter_bounce(ram);
        }
        _ => {
            let screen = ram.read_u8(OVERWORLD_SCREEN_INDEX);
            if screen < 0x80 {
                ram.write_u8(MUSIC_CONTROL, if screen & 0x3f != 0 { 2 } else { 5 });
            }
            ram.write_u8(SUBMODULE, 8);
            ram.write_u8(SUBSUBMODULE, 0);
        }
    }
}

pub fn overworld_func22(ram: &mut Ram<'_>) {
    let brightness = ram.read_u8(INIDISP_COPY).wrapping_add(1);
    ram.write_u8(INIDISP_COPY, brightness);
    if brightness == 15 {
        ram.write_u8(SUBMODULE, 0);
        ram.write_u8(SUBSUBMODULE, 0);
    }
}

pub fn overworld_func18(
    ram: &mut Ram<'_>,
    mut enter_special_area: impl FnMut(&mut Ram<'_>),
    mut load_overlays: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(LINK_MAYBE_SWIM_FASTER, 0);
    let main_module = ram.read_u8(MAIN_MODULE);
    let submodule = ram.read_u8(SUBMODULE);
    enter_special_area(ram);
    load_overlays(ram);
    ram.write_u8(SUBMODULE, submodule.wrapping_add(1));
    ram.write_u8(MAIN_MODULE, main_module);
}

pub fn overworld_func19(ram: &mut Ram<'_>, mut load_and_advance: impl FnMut(&mut Ram<'_>)) {
    let main_module = ram.read_u8(MAIN_MODULE);
    let submodule = ram.read_u8(SUBMODULE);
    load_and_advance(ram);
    ram.write_u8(SUBMODULE, submodule.wrapping_add(1));
    ram.write_u8(MAIN_MODULE, main_module);
}

pub fn fade_back_in_from_mosaic(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut load_palettes: impl FnMut(&mut Ram<'_>, u8, u8),
    mut load_new_sprite_gfx_set: impl FnMut(&mut Ram<'_>),
    mut incremental_vram_upload: impl FnMut(&mut Ram<'_>),
    mut apply_palette_filter_bounce: impl FnMut(&mut Ram<'_>),
    mut is_playing_music_track: impl FnMut(u8) -> bool,
) {
    reset_mosaic_down(ram);
    match ram.read_u8(SUBSUBMODULE) {
        0 => {
            let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
            let background = assets.background_palettes()[screen];
            let sprites = ram.read_u8(OVERWORLD_SPRITE_PALETTES + screen);
            load_palettes(ram, background, sprites);
            mosaic_transition_load_sprite_graphics(ram, &mut load_new_sprite_gfx_set);
        }
        1 => {
            incremental_vram_upload(ram);
            apply_palette_filter_bounce(ram);
        }
        _ => {
            ram.write_u8(LAST_MUSIC_CONTROL, ram.read_u8(MUSIC_UNK1));
            let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
            if !matches!(screen, 0x80 | 0x2a) {
                let music = ram.read_u8(OVERWORLD_MUSIC + screen);
                let ambient = music >> 4;
                ram.write_u8(SOUND_EFFECT_AMBIENT, if ambient != 0 { ambient } else { 5 });
                if !is_playing_music_track(music & 0x0f) {
                    ram.write_u8(MUSIC_CONTROL, music & 0x0f);
                }
            }
            ram.write_u8(SUBMODULE, 8);
            ram.write_u8(SUBSUBMODULE, 0);
            if ram.read_u8(MAIN_MODULE) == 11 {
                ram.write_u8(MAIN_MODULE, 9);
                ram.write_u8(SUBMODULE, 31);
                ram.write_u8(OW_COUNTDOWN_TRANSITION, 12);
            }
        }
    }
}

pub fn set_song_list(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let progress = ram.read_u8(SRAM_PROGRESS_INDICATOR);
    let (special_song, offset) = if progress >= 3 {
        (2, 0xc0)
    } else if ram.read_u8(LINK_SWORD_TYPE) >= 2 {
        (2, 0x80)
    } else if progress >= 2 {
        (5, 0x40)
    } else {
        (5, 0)
    };
    ram.slice_mut(OVERWORLD_MUSIC, 64)
        .copy_from_slice(&assets.overworld_music_sets()[offset..offset + 64]);
    ram.slice_mut(OVERWORLD_MUSIC + 64, 96)
        .copy_from_slice(&assets.overworld_music_sets2()[..96]);
    ram.write_u8(OVERWORLD_MUSIC + 128, special_song);
}

pub fn load_cached_entrance_properties(ram: &mut Ram<'_>) {
    ram.write_u16(OVERWORLD_AREA_INDEX, ram.read_u16(EXIT_CACHE));
    ram.write_u16(TM_COPY, ram.read_u16(EXIT_CACHE + 2));
    let vertical = ram.read_u16(EXIT_CACHE + 4);
    for address in [BG2VOFS_COPY2, BG2VOFS_COPY, BG1VOFS_COPY2, BG1VOFS_COPY] {
        ram.write_u16(address, vertical);
    }
    let horizontal = ram.read_u16(EXIT_CACHE + 6);
    for address in [BG2HOFS_COPY2, BG2HOFS_COPY, BG1HOFS_COPY2, BG1HOFS_COPY] {
        ram.write_u16(address, horizontal);
    }
    ram.write_u16(LINK_Y, ram.read_u16(EXIT_CACHE + 8));
    ram.write_u16(LINK_X, ram.read_u16(EXIT_CACHE + 10));
    if ram.read_u16(DUNGEON_ROOM) < 0x124 {
        ram.write_u16(LINK_Y, ram.read_u16(LINK_Y).wrapping_sub(0x10));
    }
    ram.write_u16(LINK_FACING, 2);
    if ram.read_u16(OW_ENTRANCE_VALUE) == 0xffff {
        ram.write_u16(LINK_Y, ram.read_u16(LINK_Y).wrapping_add(0x20));
        ram.write_u16(LINK_FACING, 0);
    }
    ram.write_u16(OVERWORLD_SCREEN_INDEX, ram.read_u16(EXIT_CACHE + 12));
    let source = ram.read_u16(EXIT_CACHE + 14);
    ram.write_u16(MAP16_LOAD_SRC_OFFSET, source);
    ram.write_u16(MAP16_LOAD_VAR2, (source.wrapping_sub(0x400) & 0xf80) >> 7);
    ram.write_u16(
        MAP16_LOAD_DST_OFFSET,
        (source.wrapping_sub(0x10) & 0x3e) >> 1,
    );
    let camera_y = ram.read_u16(EXIT_CACHE + 16);
    let camera_x = ram.read_u16(EXIT_CACHE + 18);
    ram.write_u16(CAMERA_SCROLL_LOW, camera_y);
    ram.write_u16(CAMERA_SCROLL_HIGH, camera_y.wrapping_sub(2));
    ram.write_u16(CAMERA_SCROLL_LOW + 4, camera_x);
    ram.write_u16(CAMERA_SCROLL_HIGH + 4, camera_x.wrapping_sub(2));
    for offset in (0..8).step_by(2) {
        ram.write_u16(
            OW_SCROLL_VARS0 + offset,
            ram.read_u16(EXIT_CACHE + 20 + offset),
        );
        ram.write_u16(
            UP_DOWN_SCROLL_TARGET + offset,
            ram.read_u16(EXIT_CACHE + 28 + offset),
        );
        ram.write_u16(
            OVERWORLD_UNK1 + offset,
            ram.read_u16(EXIT_CACHE + 42 + offset),
        );
    }
    ram.write_u8(BYTE_0AA0, ram.read_u8(EXIT_CACHE + 36));
    ram.write_u8(MAIN_TILE_THEME_INDEX, ram.read_u8(EXIT_CACHE + 37));
    ram.write_u8(AUX_TILE_THEME_INDEX, ram.read_u8(EXIT_CACHE + 38));
    ram.write_u8(SPRITE_GRAPHICS_INDEX, ram.read_u8(EXIT_CACHE + 39));
}

pub fn load_overworld_from_dungeon(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut load_gfx_and_screen_size: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(PLAYER_IS_INDOORS, 0);
    ram.write_u8(DUNGEON_DARK_WITH_LANTERN, 0);
    ram.write_u16(OVERWORLD_FIXED_COLOR_PLUS_MINUS, 0);
    ram.write_u16(CURRENT_PALACE_INDEX_X2, 0xff);
    ram.write_u16(NUM_MEMORIZED_TILES, 0);
    let room = ram.read_u16(DUNGEON_ROOM);
    if room != 0x104 && (0x100..0x180).contains(&room) {
        load_cached_entrance_properties(ram);
    } else {
        let index = (0..79)
            .rev()
            .find(|&index| assets.exit_rooms()[index] == room)
            .expect("dungeon room missing from overworld exit table");
        let vertical = assets.exit_scroll_y()[index];
        for address in [BG1VOFS_COPY2, BG2VOFS_COPY2, BG1VOFS_COPY, BG2VOFS_COPY] {
            ram.write_u16(address, vertical);
        }
        let horizontal = assets.exit_scroll_x()[index];
        for address in [BG1HOFS_COPY2, BG2HOFS_COPY2, BG1HOFS_COPY, BG2HOFS_COPY] {
            ram.write_u16(address, horizontal);
        }
        ram.write_u16(LINK_Y, assets.exit_link_y()[index]);
        ram.write_u16(LINK_X, assets.exit_link_x()[index]);
        let source = assets.exit_map16_source_offsets()[index];
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, source);
        ram.write_u16(MAP16_LOAD_VAR2, (source.wrapping_sub(0x400) & 0xf80) >> 7);
        ram.write_u16(
            MAP16_LOAD_DST_OFFSET,
            (source.wrapping_sub(0x10) & 0x3e) >> 1,
        );
        let camera_y = assets.exit_camera_y()[index];
        ram.write_u16(CAMERA_SCROLL_LOW, camera_y);
        ram.write_u16(CAMERA_SCROLL_HIGH, camera_y.wrapping_sub(2));
        let camera_x = assets.exit_camera_x()[index];
        ram.write_u16(CAMERA_SCROLL_LOW + 4, camera_x);
        ram.write_u16(CAMERA_SCROLL_HIGH + 4, camera_x.wrapping_sub(2));
        ram.write_u16(LINK_FACING, 2);
        ram.write_u16(OW_ENTRANCE_VALUE, assets.exit_normal_doors()[index]);
        ram.write_u16(BIG_ROCK_STARTING_ADDRESS, assets.exit_fancy_doors()[index]);
        let screen = u16::from(assets.exit_screen_indices()[index]);
        ram.write_u16(OVERWORLD_AREA_INDEX, screen);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, screen);
        let unk1 = assets.exit_unk1()[index] as i16 as u16;
        let unk3 = assets.exit_unk3()[index] as i16 as u16;
        ram.write_u16(OVERWORLD_UNK1, unk1);
        ram.write_u16(OVERWORLD_UNK1 + 2, 0u16.wrapping_sub(unk1));
        ram.write_u16(OVERWORLD_UNK1 + 4, unk3);
        ram.write_u16(OVERWORLD_UNK1 + 6, 0u16.wrapping_sub(unk3));
    }
    load_new_screen_properties(ram, &mut load_gfx_and_screen_size);
}

pub fn enter_special_area(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut load_from_dungeon: impl FnMut(&mut Ram<'_>),
    mut load_palettes: impl FnMut(&mut Ram<'_>, u8, u8),
    mut apply_special_overworld_palette: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u16(NUM_MEMORIZED_TILES, 0);
    ram.write_u16(SPECIAL_EXIT_CACHE, ram.read_u16(OVERWORLD_AREA_INDEX));
    ram.write_u16(SPECIAL_EXIT_CACHE + 2, ram.read_u16(TM_COPY));
    ram.write_u16(SPECIAL_EXIT_CACHE + 4, ram.read_u16(BG2VOFS_COPY2));
    ram.write_u16(SPECIAL_EXIT_CACHE + 6, ram.read_u16(BG2HOFS_COPY2));
    ram.write_u16(SPECIAL_EXIT_CACHE + 8, ram.read_u16(LINK_Y));
    ram.write_u16(SPECIAL_EXIT_CACHE + 10, ram.read_u16(LINK_X));
    ram.write_u16(
        SPECIAL_EXIT_CACHE + 12,
        ram.read_u16(OVERWORLD_SCREEN_INDEX),
    );
    ram.write_u16(SPECIAL_EXIT_CACHE + 14, ram.read_u16(MAP16_LOAD_SRC_OFFSET));
    ram.write_u16(SPECIAL_EXIT_CACHE + 16, ram.read_u16(CAMERA_SCROLL_LOW));
    ram.write_u16(SPECIAL_EXIT_CACHE + 18, ram.read_u16(CAMERA_SCROLL_LOW + 4));
    for offset in (0..8).step_by(2) {
        ram.write_u16(
            SPECIAL_EXIT_CACHE + 20 + offset,
            ram.read_u16(OW_SCROLL_VARS0 + offset),
        );
        ram.write_u16(
            SPECIAL_EXIT_CACHE + 28 + offset,
            ram.read_u16(UP_DOWN_SCROLL_TARGET + offset),
        );
        ram.write_u16(
            SPECIAL_EXIT_CACHE + 42 + offset,
            ram.read_u16(OVERWORLD_UNK1 + offset),
        );
    }
    ram.write_u8(SPECIAL_EXIT_CACHE + 36, ram.read_u8(BYTE_0AA0));
    ram.write_u8(SPECIAL_EXIT_CACHE + 37, ram.read_u8(MAIN_TILE_THEME_INDEX));
    ram.write_u8(SPECIAL_EXIT_CACHE + 38, ram.read_u8(AUX_TILE_THEME_INDEX));
    ram.write_u8(SPECIAL_EXIT_CACHE + 39, ram.read_u8(SPRITE_GRAPHICS_INDEX));
    load_from_dungeon(ram);
    if ram.read_u16(DUNGEON_ROOM) == 0x1010 {
        ram.write_u16(DUNGEON_ROOM, 0x182);
    }
    let room_backup = ram.read_u8(DUNGEON_ROOM);
    let index = room_backup.wrapping_sub(0x80);
    ram.write_u8(DUNGEON_ROOM, index);
    let i = usize::from(index);
    ram.write_u8(LINK_FACING, assets.special_exit_direction()[i]);
    ram.write_u8(INCREMENTAL_VRAM_COUNTER, 0);
    ram.write_u8(SPRITE_GRAPHICS_INDEX, assets.special_exit_sprite_gfx()[i]);
    ram.write_u8(AUX_TILE_THEME_INDEX, assets.special_exit_aux_gfx()[i]);
    load_palettes(
        ram,
        assets.special_exit_bg_palette()[i],
        assets.special_exit_sprite_palette()[i],
    );
    let area = i & 0x3f;
    ram.write_u16(
        OVERWORLD_OFFSET_BASE_Y_ADDR,
        assets.special_exit_top()[area],
    );
    ram.write_u16(
        OVERWORLD_OFFSET_BASE_X_ADDR,
        assets.special_exit_left_edge()[area] >> 3,
    );
    ram.write_u16(OVERWORLD_OFFSET_MASK_Y, 0x3f0);
    ram.write_u16(OVERWORLD_OFFSET_MASK_X, 0x7e);
    let area = i & 0x7f;
    ram.write_u16(OW_SCROLL_VARS0, assets.special_exit_top()[area]);
    ram.write_u16(OW_SCROLL_VARS0 + 2, assets.special_exit_bottom()[area]);
    ram.write_u16(OW_SCROLL_VARS0 + 4, assets.special_exit_left()[area]);
    ram.write_u16(OW_SCROLL_VARS0 + 6, assets.special_exit_right()[area]);
    ram.write_u16(
        UP_DOWN_SCROLL_TARGET,
        assets.special_exit_up_target()[area] as u16,
    );
    ram.write_u16(
        UP_DOWN_SCROLL_TARGET_END,
        assets.special_exit_down_target()[area] as u16,
    );
    ram.write_u16(
        LEFT_RIGHT_SCROLL_TARGET,
        assets.special_exit_left_target()[area] as u16,
    );
    ram.write_u16(
        LEFT_RIGHT_SCROLL_TARGET_END,
        assets.special_exit_right_target()[area] as u16,
    );
    ram.write_u8(DUNGEON_ROOM, room_backup);
    apply_special_overworld_palette(ram);
}

pub fn load_overworld_from_special_overworld(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut load_palettes: impl FnMut(&mut Ram<'_>, u8, u8),
    mut palette_special_overworld: impl FnMut(&mut Ram<'_>),
    mut reset_swimming_state: impl FnMut(&mut Ram<'_>),
    mut load_gfx_and_screen_size: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u16(NUM_MEMORIZED_TILES, 0);
    ram.write_u16(OVERWORLD_AREA_INDEX, ram.read_u16(SPECIAL_EXIT_CACHE));
    ram.write_u16(TM_COPY, ram.read_u16(SPECIAL_EXIT_CACHE + 2));
    let vertical = ram.read_u16(SPECIAL_EXIT_CACHE + 4);
    for address in [BG2VOFS_COPY2, BG2VOFS_COPY, BG1VOFS_COPY2, BG1VOFS_COPY] {
        ram.write_u16(address, vertical);
    }
    let horizontal = ram.read_u16(SPECIAL_EXIT_CACHE + 6);
    for address in [BG2HOFS_COPY2, BG2HOFS_COPY, BG1HOFS_COPY2, BG1HOFS_COPY] {
        ram.write_u16(address, horizontal);
    }
    ram.write_u16(LINK_Y, ram.read_u16(SPECIAL_EXIT_CACHE + 8));
    ram.write_u16(LINK_X, ram.read_u16(SPECIAL_EXIT_CACHE + 10));
    ram.write_u16(
        OVERWORLD_SCREEN_INDEX,
        ram.read_u16(SPECIAL_EXIT_CACHE + 12),
    );
    let source = ram.read_u16(SPECIAL_EXIT_CACHE + 14);
    ram.write_u16(MAP16_LOAD_SRC_OFFSET, source);
    ram.write_u16(MAP16_LOAD_VAR2, (source.wrapping_sub(0x400) & 0xf80) >> 7);
    ram.write_u16(
        MAP16_LOAD_DST_OFFSET,
        (source.wrapping_sub(0x10) & 0x3e) >> 1,
    );
    let camera_y = ram.read_u16(SPECIAL_EXIT_CACHE + 16);
    let camera_x = ram.read_u16(SPECIAL_EXIT_CACHE + 18);
    ram.write_u16(CAMERA_SCROLL_LOW, camera_y);
    ram.write_u16(CAMERA_SCROLL_HIGH, camera_y.wrapping_sub(2));
    ram.write_u16(CAMERA_SCROLL_LOW + 4, camera_x);
    ram.write_u16(CAMERA_SCROLL_HIGH + 4, camera_x.wrapping_sub(2));
    for offset in (0..8).step_by(2) {
        ram.write_u16(
            OW_SCROLL_VARS0 + offset,
            ram.read_u16(SPECIAL_EXIT_CACHE + 20 + offset),
        );
        ram.write_u16(
            UP_DOWN_SCROLL_TARGET + offset,
            ram.read_u16(SPECIAL_EXIT_CACHE + 28 + offset),
        );
        ram.write_u16(
            OVERWORLD_UNK1 + offset,
            ram.read_u16(SPECIAL_EXIT_CACHE + 42 + offset),
        );
    }
    ram.write_u8(BYTE_0AA0, ram.read_u8(SPECIAL_EXIT_CACHE + 36));
    ram.write_u8(MAIN_TILE_THEME_INDEX, ram.read_u8(SPECIAL_EXIT_CACHE + 37));
    ram.write_u8(AUX_TILE_THEME_INDEX, ram.read_u8(SPECIAL_EXIT_CACHE + 38));
    ram.write_u8(SPRITE_GRAPHICS_INDEX, ram.read_u8(SPECIAL_EXIT_CACHE + 39));
    let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
    let background = assets.background_palettes()[screen];
    let sprites = ram.read_u8(OVERWORLD_SPRITE_PALETTES + screen);
    load_palettes(ram, background, sprites);
    palette_special_overworld(ram);
    ram.write_u8(LINK_QUADRANT_X, 0);
    ram.write_u8(LINK_QUADRANT_Y, 2);
    ram.write_u8(QUADRANT_FULLSIZE_X, 2);
    ram.write_u8(QUADRANT_FULLSIZE_Y, 2);
    ram.write_u8(PLAYER_OAM_X_OFFSET, 0x80);
    ram.write_u8(PLAYER_OAM_Y_OFFSET, 0x80);
    ram.write_u8(LINK_DIRECTION_MASK_A, 0x0f);
    ram.write_u8(LINK_DIRECTION_MASK_B, 0x0f);
    ram.write_u8(LINK_Z, 0xff);
    ram.write_u8(LINK_ACTUAL_VELOCITY_Z, 0xff);
    reset_swimming_state(ram);
    load_gfx_and_screen_size(ram);
    ram.write_u8(OVERWORLD_RIGHT_BOTTOM_BOUND, 228);
    ram.write_u16(
        OVERWORLD_AREA_IS_BIG,
        ram.read_u16(OVERWORLD_AREA_IS_BIG) & 0xff,
    );
}

pub fn load_new_screen_properties(
    ram: &mut Ram<'_>,
    mut load_gfx_and_screen_size: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u16(TILEMAP_LOCATION_CALC_MASK, !7u16);
    load_gfx_and_screen_size(ram);
    ram.write_u8(OVERWORLD_RIGHT_BOTTOM_BOUND, 0xe4);
    ram.write_u16(
        OVERWORLD_AREA_IS_BIG,
        ram.read_u16(OVERWORLD_AREA_IS_BIG) & 0xff,
    );
    let big = ram.read_u16(OVERWORLD_AREA_IS_BIG) != 0;
    let area = usize::from(ram.read_u16(OVERWORLD_SCREEN_INDEX) & 0x3f);
    set_camera_boundaries(ram, big, area);
    ram.write_u8(LINK_QUADRANT_X, 0);
    ram.write_u8(LINK_QUADRANT_Y, 2);
    ram.write_u8(QUADRANT_FULLSIZE_X, 2);
    ram.write_u8(QUADRANT_FULLSIZE_Y, 2);
    ram.write_u8(PLAYER_OAM_X_OFFSET, 0x80);
    ram.write_u8(PLAYER_OAM_Y_OFFSET, 0x80);
    ram.write_u8(LINK_DIRECTION_MASK_A, 0x0f);
    ram.write_u8(LINK_DIRECTION_MASK_B, 0x0f);
    ram.write_u8(LINK_Z, 0xff);
    ram.write_u8(LINK_ACTUAL_VELOCITY_Z, 0xff);
}

pub fn load_gfx_and_screen_size(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
    ram.write_u8(INCREMENTAL_VRAM_COUNTER, 0);
    ram.write_u8(
        SPRITE_GRAPHICS_INDEX,
        ram.read_u8(OVERWORLD_SPRITE_GFX + screen),
    );
    ram.write_u8(
        AUX_TILE_THEME_INDEX,
        assets.overworld_aux_tile_themes()[screen],
    );
    ram.write_u8(
        OVERWORLD_AREA_IS_BIG_BACKUP,
        ram.read_u8(OVERWORLD_AREA_IS_BIG),
    );
    let is_small = assets.overworld_map_is_small()[screen & 0x3f] != 0;
    ram.write_u8(OVERWORLD_AREA_IS_BIG, if is_small { 0 } else { 0x20 });
    ram.write_u8(
        OVERWORLD_RIGHT_BOTTOM_BOUND + 1,
        if is_small { 1 } else { 3 },
    );
    ram.write_u8(
        MAIN_TILE_THEME_INDEX,
        if ram.read_u16(OVERWORLD_SCREEN_INDEX) & 0x40 != 0 {
            0x21
        } else {
            0x20
        },
    );
    let pack = 6 + if ram.read_u16(OVERWORLD_SCREEN_INDEX) & 0x40 != 0 {
        8
    } else {
        0
    };
    ram.write_u8(MISC_SPRITE_GRAPHICS_INDEX, VARIOUS_PACKS[pack]);
    let area = screen & 0xbf;
    ram.write_u16(OVERWORLD_OFFSET_BASE_Y_ADDR, OVERWORLD_OFFSET_BASE_Y[area]);
    ram.write_u16(
        OVERWORLD_OFFSET_BASE_X_ADDR,
        OVERWORLD_OFFSET_BASE_X[area] >> 3,
    );
    let mask = if is_small { 0x1f0 } else { 0x3f0 };
    ram.write_u16(OVERWORLD_OFFSET_MASK_Y, mask);
    ram.write_u16(OVERWORLD_OFFSET_MASK_X, mask >> 3);
}

pub fn load_overlays(
    ram: &mut Ram<'_>,
    mut initialize_sprite_slots: impl FnMut(&mut Ram<'_>),
    mut reload_overworld_sprites: impl FnMut(&mut Ram<'_>),
    mut load_overlays_inner: impl FnMut(&mut Ram<'_>),
) {
    initialize_sprite_slots(ram);
    reload_overworld_sprites(ram);
    ram.write_u8(LINK_STATE_BITS, 0);
    ram.write_u8(LINK_PICKING_THROW_STATE, 0);
    ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
    load_overlays_inner(ram);
}

pub fn pre_overworld_load_overlays(
    ram: &mut Ram<'_>,
    mut load_overlays_inner: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
    load_overlays_inner(ram);
}

pub fn load_overlays_inner(
    ram: &mut Ram<'_>,
    mut load_overworld_overlay: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u16(OVERLAY_STATE_CACHE, ram.read_u16(OVERWORLD_SCREEN_INDEX));
    ram.write_u16(OVERLAY_STATE_CACHE + 2, ram.read_u16(MAP16_LOAD_SRC_OFFSET));
    ram.write_u16(OVERLAY_STATE_CACHE + 4, ram.read_u16(MAP16_LOAD_VAR2));
    ram.write_u16(OVERLAY_STATE_CACHE + 6, ram.read_u16(MAP16_LOAD_DST_OFFSET));
    ram.write_u8(
        OVERLAY_STATE_CACHE + 8,
        ram.read_u8(OVERWORLD_SCREEN_TRANSITION),
    );
    ram.write_u16(OVERLAY_STATE_CACHE + 10, ram.read_u16(OVERWORLD_TRANS_DIR));
    ram.write_u16(OVERLAY_STATE_CACHE + 12, ram.read_u16(OVERWORLD_TRANS_DIR2));
    ram.write_u16(OVERLAY_INDEX, 0);
    ram.write_u16(BG1VOFS_SUBPIXEL, 0);
    ram.write_u16(BG1HOFS_SUBPIXEL, 0);

    let screen = ram.read_u16(OVERWORLD_SCREEN_INDEX);
    let overlay = if screen >= 0x80 {
        match ram.read_u16(DUNGEON_ROOM) {
            0x180 if ram.read_u8(SAVE_OW_EVENT_INFO + 0x80) & 0x40 == 0 => 0x97,
            0x181 => 0x94,
            0x189 => 0x93,
            0x182 | 0x183 => {
                ram.write_u8(SOUND_EFFECT_AMBIENT, 1);
                ram.write_u8(TS_COPY, 0);
                ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
                return;
            }
            _ => {
                ram.write_u8(TS_COPY, 0);
                ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
                return;
            }
        }
    } else if screen & 0x3f == 0 {
        if screen & 0x40 == 0 && ram.read_u8(SAVE_OW_EVENT_INFO + 0x80) & 0x40 != 0 {
            0x9e
        } else {
            0x9d
        }
    } else if matches!(screen, 3 | 5 | 7) {
        0x95
    } else if matches!(screen, 0x43 | 0x45 | 0x47) {
        0x9c
    } else if screen == 0x70 {
        if ram.read_u8(SAVE_OW_EVENT_INFO + 0x70) & 0x20 == 0 {
            0x9f
        } else {
            ram.write_u8(TS_COPY, 0);
            ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
            return;
        }
    } else if ram.read_u8(SRAM_PROGRESS_INDICATOR) < 2 {
        0x9f
    } else {
        0x96
    };

    ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x390);
    ram.write_u16(OVERLAY_INDEX, overlay);
    ram.write_u16(OVERWORLD_SCREEN_INDEX, overlay);
    ram.write_u16(MAP16_LOAD_VAR2, 0x1f);
    ram.write_u16(MAP16_LOAD_DST_OFFSET, 0);
    ram.write_u16(OVERWORLD_SCREEN_TRANSITION, 0);
    ram.write_u16(OVERWORLD_TRANS_DIR, 0);
    ram.write_u16(OVERWORLD_TRANS_DIR2, 0);
    ram.write_u8(CGWSEL_COPY, 0x82);
    ram.write_u8(TM_COPY, 0x16);
    ram.write_u8(TS_COPY, 1);
    ram.write_u8(
        SOUND_EFFECT_AMBIENT,
        ram.read_u8(OVERWORLD_MUSIC + usize::from(overlay)) >> 4,
    );
    if matches!(overlay, 0x97 | 0x94 | 0x93 | 0x9d | 0x9e | 0x9f) {
        ram.write_u8(CGADSUB_COPY, 0x72);
    } else if matches!(overlay, 0x95 | 0x9c)
        || matches!(ram.read_u8(OVERLAY_STATE_CACHE), 0x5b | 0x1b)
            && matches!(ram.read_u8(SUBMODULE), 35 | 44)
    {
        ram.write_u8(CGADSUB_COPY, 0x20);
    } else {
        ram.write_u8(TS_COPY, 0);
        ram.write_u8(CGADSUB_COPY, 0x20);
    }
    load_overworld_overlay(ram);
    if ram.read_u8(OVERLAY_INDEX) == 0x94 {
        ram.write_u16(BG1VOFS_COPY2, ram.read_u16(BG1VOFS_COPY2) | 0x100);
    }
    ram.write_u16(OVERWORLD_SCREEN_INDEX, ram.read_u16(OVERLAY_STATE_CACHE));
    ram.write_u16(MAP16_LOAD_SRC_OFFSET, ram.read_u16(OVERLAY_STATE_CACHE + 2));
    ram.write_u16(MAP16_LOAD_VAR2, ram.read_u16(OVERLAY_STATE_CACHE + 4));
    ram.write_u16(MAP16_LOAD_DST_OFFSET, ram.read_u16(OVERLAY_STATE_CACHE + 6));
    ram.write_u8(
        OVERWORLD_SCREEN_TRANSITION,
        ram.read_u8(OVERLAY_STATE_CACHE + 8),
    );
    ram.write_u16(OVERWORLD_TRANS_DIR, ram.read_u16(OVERLAY_STATE_CACHE + 10));
    ram.write_u16(OVERWORLD_TRANS_DIR2, ram.read_u16(OVERLAY_STATE_CACHE + 12));
}

pub fn spotlight_configure_table_and_control(
    ram: &mut Ram<'_>,
    mut configure_table: impl FnMut(&mut Ram<'_>),
    mut open_spotlight_next: impl FnMut(&mut Ram<'_>),
) {
    configure_table(ram);
    ram.write_u8(IS_NMI_THREAD_ACTIVE, 0);
    ram.write_u8(NMI_FLAG_UPDATE_POLYHEDRAL, 0);
    if ram.read_u8(SUBMODULE) != 0 {
        return;
    }
    if ram.read_u8(MAIN_MODULE) == 6 {
        ram.write_u16(LINK_Y, ram.read_u16(EXIT_CACHE + 8));
    }
    open_spotlight_next(ram);
}

pub fn module10_open_iris(ram: &mut Ram<'_>, mut spotlight_open: impl FnMut(&mut Ram<'_>)) {
    spotlight_open(ram);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
}

pub fn module10_spotlight_open(
    ram: &mut Ram<'_>,
    mut sprite_main: impl FnMut(&mut Ram<'_>),
    mut spotlight_open: impl FnMut(&mut Ram<'_>),
    mut configure_table: impl FnMut(&mut Ram<'_>),
    mut open_spotlight_next: impl FnMut(&mut Ram<'_>),
    mut link_oam_main: impl FnMut(&mut Ram<'_>),
) {
    sprite_main(ram);
    if ram.read_u8(SUBMODULE) == 0 {
        module10_open_iris(ram, &mut spotlight_open);
    } else {
        spotlight_configure_table_and_control(ram, &mut configure_table, &mut open_spotlight_next);
    }
    link_oam_main(ram);
}

pub fn open_spotlight_next(
    ram: &mut Ram<'_>,
    mut enable_force_blank: impl FnMut(&mut Ram<'_>),
    mut reset_link_items: impl FnMut(&mut Ram<'_>),
) {
    if ram.read_u8(MAIN_MODULE) != 9 {
        enable_force_blank(ram);
        reset_link_items(ram);
    }
    if ram.read_u8(MAIN_MODULE) == 9 {
        if ram.read_u16(DUNGEON_ROOM) != 0x20 {
            ram.write_u8(
                SUBMODULE,
                if ram.read_u8(LINK_FACING) != 0 {
                    0x0a
                } else {
                    0x0b
                },
            );
        }
        ram.write_u8(OW_COUNTDOWN_TRANSITION, 16);
        let rock = ram.read_u16(BIG_ROCK_STARTING_ADDRESS);
        if (ram.read_u8(OW_ENTRANCE_VALUE) | rock as u8) != 0 && rock >> 8 != 0 {
            ram.write_u8(
                DOOR_OPEN_CLOSED_COUNTER,
                if rock & 0x8000 != 0 { 0x18 } else { 0 },
            );
            ram.write_u16(BIG_ROCK_STARTING_ADDRESS, rock & 0x7fff);
            ram.write_u8(DOOR_ANIMATION_STEP, 0);
            ram.write_u8(SUBMODULE, 9);
            ram.write_u8(SUBSUBMODULE, 0);
            ram.write_u8(SOUND_EFFECT_2, 21);
        }
    }
    for address in [
        W12SEL_COPY,
        W34SEL_COPY,
        WOBJSEL_COPY,
        TMW_COPY,
        TSW_COPY,
        LINK_FORCE_HOLD_SWORD_UP,
    ] {
        ram.write_u8(address, 0);
    }
    match ram.read_u8(OVERWORLD_SCREEN_INDEX) {
        3 | 5 | 7 => {
            ram.write_u8(COLDATA_COPY0, 0x26);
            ram.write_u8(COLDATA_COPY1, 0x4c);
            ram.write_u8(COLDATA_COPY2, 0x8c);
        }
        0x43 | 0x45 | 0x47 => {
            ram.write_u8(COLDATA_COPY0, 0x26);
            ram.write_u8(COLDATA_COPY1, 0x4a);
            ram.write_u8(COLDATA_COPY2, 0x87);
        }
        _ => {}
    }
}

pub fn dungeon_prep_exit_with_spotlight(
    ram: &mut Ram<'_>,
    mut entrance_music_track: impl FnMut(u16) -> u8,
    mut hud_floor_indicator: impl FnMut(&mut Ram<'_>),
    mut iris_spotlight_close: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(IS_NMI_THREAD_ACTIVE, 0);
    ram.write_u8(NMI_FLAG_UPDATE_POLYHEDRAL, 0);
    if ram.read_u8(PLAYER_IS_INDOORS) == 0 {
        terminate_waterfall_splashes(ram);
        ram.write_u16(EXIT_CACHE + 8, ram.read_u16(LINK_Y));
    }
    let mut music = entrance_music_track(ram.read_u16(WHICH_ENTRANCE));
    let should_change = if music != 3 {
        true
    } else {
        music = ram.read_u8(SRAM_PROGRESS_INDICATOR);
        music >= 2
    };
    if should_change {
        if music != 0xf2 {
            music = 0xf1;
        } else if ram.read_u8(MUSIC_UNK1) == 12 {
            music = 7;
        }
        ram.write_u8(MUSIC_CONTROL, music);
    }
    ram.write_u8(HUD_FLOOR_CHANGED_TIMER, 0);
    hud_floor_indicator(ram);
    ram.write_u8(
        FLAG_UPDATE_HUD,
        ram.read_u8(FLAG_UPDATE_HUD).wrapping_add(1),
    );
    iris_spotlight_close(ram);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
}

pub fn module0f_spotlight_close(
    ram: &mut Ram<'_>,
    mut sprite_main: impl FnMut(&mut Ram<'_>),
    mut prep_exit: impl FnMut(&mut Ram<'_>),
    mut configure_and_control: impl FnMut(&mut Ram<'_>),
    mut handle_velocity: impl FnMut(&mut Ram<'_>),
    mut handle_moving_animation: impl FnMut(&mut Ram<'_>),
    mut link_oam_main: impl FnMut(&mut Ram<'_>),
) {
    sprite_main(ram);
    if ram.read_u8(SUBMODULE) == 0 {
        prep_exit(ram);
    } else {
        configure_and_control(ram);
    }
    let outdoors = ram.read_u8(PLAYER_IS_INDOORS) == 0;
    if outdoors {
        if ram.read_u8(OVERWORLD_SCREEN_INDEX) == 0x0f {
            ram.write_u8(DRAW_WATER_RIPPLES_OR_GRASS, 1);
        }
        ram.write_u8(LINK_SPEED_SETTING, 6);
        handle_velocity(ram);
        ram.write_u8(LINK_X_VELOCITY, 0);
        ram.write_u8(LINK_Y_VELOCITY, 0);
    }
    let direction_index = if outdoors {
        usize::from(ram.read_u16(WHICH_ENTRANCE) == 0x43)
    } else {
        usize::from(ram.read_u8(LINK_FACING) >> 1)
    };
    let direction = [8u8, 4, 2, 1][direction_index];
    ram.write_u8(LINK_DIRECTION, direction);
    ram.write_u8(LINK_DIRECTION_LAST, direction);
    handle_moving_animation(ram);
    link_oam_main(ram);
}

pub fn module09_load_aux_gfx(
    ram: &mut Ram<'_>,
    mut load_transition_aux_gfx: impl FnMut(&mut Ram<'_>),
    mut prepare_transition_aux_gfx: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(
        SAVE_OW_EVENT_INFO + 0x3b,
        ram.read_u8(SAVE_OW_EVENT_INFO + 0x3b) & !0x20,
    );
    ram.write_u8(
        SAVE_OW_EVENT_INFO + 0x7b,
        ram.read_u8(SAVE_OW_EVENT_INFO + 0x7b) & !0x20,
    );
    ram.write_u16(0xf000 + 267 * 2, ram.read_u16(0xf000 + 267 * 2) & !0x80);
    ram.write_u16(0xf000 + 40 * 2, ram.read_u16(0xf000 + 40 * 2) & !0x100);
    load_transition_aux_gfx(ram);
    prepare_transition_aux_gfx(ram);
    ram.write_u8(NMI_DISABLE_CORE_UPDATES, 9);
    ram.write_u8(NMI_SUBROUTINE_INDEX, 9);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
}

pub fn finish_transition_gfx(ram: &mut Ram<'_>) {
    ram.write_u8(NMI_DISABLE_CORE_UPDATES, 10);
    ram.write_u8(NMI_SUBROUTINE_INDEX, 10);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
}

pub fn module09_load_new_map_and_gfx(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    load_event_overlay: impl FnMut(&mut Ram<'_>),
    trigger_map_load_stripe: impl FnMut(&mut Ram<'_>, MapStripeAxis, u8),
    mut load_new_sprite_gfx_set: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u16(PEG_PUZZLE_PROGRESS, 0);
    some_tile_map_change(ram, assets, load_event_overlay);
    ram.write_u8(
        NMI_DISABLE_CORE_UPDATES,
        ram.read_u8(NMI_DISABLE_CORE_UPDATES).wrapping_add(1),
    );
    create_initial_new_screen_map_to_scroll(ram, assets, trigger_map_load_stripe);
    load_new_sprite_gfx_set(ram);
}

pub fn run_scroll_transition(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut handle_moving_animation: impl FnMut(&mut Ram<'_>),
    mut incremental_vram_upload: impl FnMut(&mut Ram<'_>),
    initialize_sprite_slots: impl FnMut(&mut Ram<'_>),
) {
    handle_moving_animation(ram);
    incremental_vram_upload(ram);
    let result = scroll_transition(ram, initialize_sprite_slots);
    if result & 0x0f == 0 {
        ram.write_u8(OVERWORLD_TRANS_DIR2, ram.read_u8(OVERWORLD_TRANS_DIR));
        transition_scroll_and_load_map(ram, assets);
        ram.write_u8(OVERWORLD_TRANS_DIR2, 0);
    }
}

pub fn start_scroll_transition(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
    if ram.read_u8(OVERWORLD_TRANS_DIR) >= 4 {
        ram.write_u8(OVERWORLD_TRANS_DIR2, ram.read_u8(OVERWORLD_TRANS_DIR));
        transition_scroll_and_load_map(ram, assets);
        ram.write_u8(OVERWORLD_TRANS_DIR2, 0);
    }
}

pub fn module09_load_new_sprites(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut reload_overworld_sprites: impl FnMut(&mut Ram<'_>),
) {
    if ram.read_u16(OVERWORLD_SCREEN_TRANSITION) == 1 {
        ram.write_u16(BG2VOFS_COPY2, ram.read_u16(BG2VOFS_COPY2).wrapping_add(2));
        ram.write_u16(LINK_Y, ram.read_u16(LINK_Y).wrapping_add(2));
    }
    reload_overworld_sprites(ram);
    ram.write_u16(NUM_MEMORIZED_TILES, 0);
    if ram.read_u8(SRAM_PROGRESS_INDICATOR) >= 2 && ram.read_u8(SUBMODULE) != 18 {
        set_fixed_color_and_scroll(ram);
    }
    start_scroll_transition(ram, assets);
}

pub fn ease_off_scroll_transition(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut disable_follower: impl FnMut(&mut Ram<'_>),
) {
    let small =
        assets.overworld_map_is_small()[usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX))] != 0;
    if small {
        ram.write_u8(OVERWORLD_TRANS_DIR2, ram.read_u8(OVERWORLD_TRANS_DIR));
        transition_scroll_and_load_map(ram, assets);
        ram.write_u8(OVERWORLD_TRANS_DIR2, 0);
    }
    let counter = ram.read_u8(SUBSUBMODULE).wrapping_add(1);
    ram.write_u8(SUBSUBMODULE, counter);
    if counter < 8 || matches!(ram.read_u8(OVERWORLD_TRANS_DIR), 8 | 2) && counter < 9 {
        return;
    }
    ram.write_u8(SUBSUBMODULE, 0);
    ram.write_u8(OVERWORLD_TRANS_DIR, 0);
    if small {
        ram.write_u16(
            MAP16_LOAD_SRC_OFFSET,
            ram.read_u16(ORANGE_BLUE_BARRIER_STATE),
        );
        ram.write_u16(MAP16_LOAD_DST_OFFSET, ram.read_u16(WORD_C174));
        ram.write_u16(MAP16_LOAD_VAR2, ram.read_u16(WORD_C176));
    }
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
    disable_follower(ram);
}

pub fn walk_from_exit_facing_down(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut handle_moving_animation: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(LINK_DIRECTION_LAST, 4);
    handle_moving_animation(ram);
    ram.write_u16(LINK_Y, ram.read_u16(LINK_Y).wrapping_add(1));
    let countdown = ram.read_u8(OW_COUNTDOWN_TRANSITION).wrapping_sub(1);
    ram.write_u8(OW_COUNTDOWN_TRANSITION, countdown);
    if countdown != 0 {
        return;
    }
    ram.write_u8(SUBMODULE, 0);
    ram.write_u16(LINK_Y, ram.read_u16(LINK_Y).wrapping_add(3));
    ram.write_u8(LINK_Y_VELOCITY, 3);
    operate_camera_scroll(ram);
    if ram.read_u8(OVERWORLD_TRANS_DIR2) != 0 {
        handle_map_scroll(ram, assets);
    }
}

pub fn walk_from_exit_facing_up(
    ram: &mut Ram<'_>,
    mut handle_moving_animation: impl FnMut(&mut Ram<'_>),
) {
    handle_moving_animation(ram);
    ram.write_u16(LINK_Y, ram.read_u16(LINK_Y).wrapping_sub(1));
    let countdown = ram.read_u8(OW_COUNTDOWN_TRANSITION).wrapping_sub(1);
    ram.write_u8(OW_COUNTDOWN_TRANSITION, countdown);
    if countdown == 0 {
        ram.write_u8(SUBMODULE, 0);
    }
}

pub fn open_big_door_from_exit(
    ram: &mut Ram<'_>,
    mut update_map_32x32_conditional: impl FnMut(&mut Ram<'_>),
) {
    if ram.read_u8(DOOR_ANIMATION_STEP) != 3 {
        update_map_32x32_conditional(ram);
        return;
    }
    ram.write_u8(OW_COUNTDOWN_TRANSITION, 36);
    ram.write_u8(OVERWORLD_TRANS_DIR2, 0);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
}

pub fn open_big_door(
    ram: &mut Ram<'_>,
    mut update_map_32x32_conditional: impl FnMut(&mut Ram<'_>),
) {
    if ram.read_u8(DOOR_ANIMATION_STEP) != 3 {
        update_map_32x32_conditional(ram);
        return;
    }
    ram.write_u8(SUBMODULE, 0);
    ram.write_u8(SUBSUBMODULE, 0);
    ram.write_u8(OVERWORLD_TRANS_DIR2, 0);
}

pub fn update_map_32x32_conditional(
    ram: &mut Ram<'_>,
    mut update_map_32x32: impl FnMut(&mut Ram<'_>),
) {
    if ram.read_u16(DOOR_OPEN_CLOSED_COUNTER) & 7 != 0 {
        ram.write_u8(
            DOOR_OPEN_CLOSED_COUNTER,
            ram.read_u8(DOOR_OPEN_CLOSED_COUNTER).wrapping_add(1),
        );
    } else {
        update_map_32x32(ram);
    }
}

pub fn update_map_32x32_b(ram: &mut Ram<'_>, mut update_map_32x32: impl FnMut(&mut Ram<'_>)) {
    update_map_32x32(ram);
    ram.write_u8(DOOR_OPEN_CLOSED_COUNTER, 0);
}

pub fn update_map_32x32(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let memorized_offset = usize::from(ram.read_u16(NUM_MEMORIZED_TILES));
    let tile_index = usize::from(ram.read_u16(DOOR_OPEN_CLOSED_COUNTER) >> 1);
    let base = ram.read_u16(BIG_ROCK_STARTING_ADDRESS);
    for (index, position_delta) in [0u16, 2, 0x80, 0x82].into_iter().enumerate() {
        let position = base.wrapping_add(position_delta);
        let value = DOOR_ANIMATION_TILES[tile_index + index];
        ram.write_u16(MEMORIZED_TILE_ADDR + memorized_offset + index * 2, position);
        ram.write_u16(MEMORIZED_TILE_VALUE + memorized_offset + index * 2, value);
        draw_map16_persist(ram, assets, position, value);
    }
    let upload_end = VRAM_UPLOAD_DATA + usize::from(ram.read_u16(VRAM_UPLOAD_OFFSET));
    ram.write_u16(upload_end, 0xffff);
    ram.write_u16(
        NUM_MEMORIZED_TILES,
        ram.read_u16(NUM_MEMORIZED_TILES).wrapping_add(8),
    );
    let step = if ram.read_u16(DOOR_OPEN_CLOSED_COUNTER) == 32 {
        2
    } else {
        1
    };
    ram.write_u16(
        DOOR_ANIMATION_STEP,
        ram.read_u16(DOOR_ANIMATION_STEP).wrapping_add(step),
    );
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
    ram.write_u8(
        DOOR_OPEN_CLOSED_COUNTER,
        ram.read_u8(DOOR_OPEN_CLOSED_COUNTER).wrapping_add(1),
    );
}

pub fn take_damage_from_pit(ram: &mut Ram<'_>) {
    ram.write_u8(LINK_VISIBILITY, 12);
    ram.write_u8(
        SUBMODULE,
        if ram.read_u8(PLAYER_IS_INDOORS) != 0 {
            20
        } else {
            42
        },
    );
    let health = ram.read_u8(LINK_HEALTH_CURRENT).wrapping_sub(8);
    ram.write_u8(LINK_HEALTH_CURRENT, if health >= 0xa8 { 0 } else { health });
}

pub fn force_nonbunny_status(ram: &mut Ram<'_>) {
    ram.write_u8(LINK_PLAYER_STATE, 0);
    ram.write_u16(LINK_TIMER_TEMP_BUNNY, 0);
    ram.write_u8(LINK_NEED_TRANSFORM_POOF, 0);
    ram.write_u8(LINK_IS_BUNNY, 0);
    ram.write_u8(LINK_IS_BUNNY_MIRROR, 0);
    if ram.read_u8(ENHANCED_FEATURES0) & 4 != 0 {
        ram.write_u8(LINK_IS_RUNNING, 0);
    }
}

pub fn adjust_link_bunny_status(ram: &mut Ram<'_>) {
    if ram.read_u8(LINK_ITEM_MOON_PEARL) != 0 {
        force_nonbunny_status(ram);
    }
}

pub fn module08_overworld_load(
    ram: &mut Ram<'_>,
    mut dispatch_pre_overworld: impl FnMut(&mut Ram<'_>, u8),
) {
    let submodule = ram.read_u8(SUBMODULE);
    dispatch_pre_overworld(ram, submodule);
}

pub fn pre_overworld_load_properties(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    hooks: &mut impl PreOverworldHooks,
) {
    ram.write_u8(CGWSEL_COPY, 0x82);
    ram.write_u8(DUNGEON_UNK6, 0);
    adjust_link_bunny_status(ram);
    if ram.read_u8(MAIN_MODULE) == 8 {
        hooks.load_from_dungeon(ram);
    } else {
        hooks.load_from_special_overworld(ram);
    }
    set_song_list(ram, assets);
    ram.write_u8(LINK_NUM_KEYS, 0xff);
    hooks.hud_refill_logic(ram);

    let screen = ram.read_u8(OVERWORLD_SCREEN_INDEX);
    let room = ram.read_u8(DUNGEON_ROOM);
    let mut animated_tiles = 0x58;
    let mut skip_dark_world_override = false;
    let mut track = if matches!(screen, 3 | 5 | 7) {
        2
    } else if matches!(screen, 0x43 | 0x45 | 0x47) {
        9
    } else {
        animated_tiles = 0x5a;
        if screen >= 0x40 {
            if ram.read_u8(QUEUED_MUSIC_CONTROL) == 0xf2 {
                skip_dark_world_override = true;
                0xf3
            } else if ram.read_u8(SRAM_PROGRESS_INDICATOR) < 2 {
                3
            } else {
                2
            }
        } else if room == 0xe3 || room == 0x18 || room == 0x2f || room == 0x1f && screen == 0x18 {
            if ram.read_u8(SRAM_PROGRESS_INDICATOR) < 3 {
                7
            } else {
                2
            }
        } else if ram.read_u16(SAVEGAME_MASTER_SWORD_FLAGS) & 0x40 != 0 {
            2
        } else if room == 0 || room == 0xe1 {
            5
        } else if ram.read_u8(QUEUED_MUSIC_CONTROL) == 0xf2 {
            skip_dark_world_override = true;
            0xf3
        } else if ram.read_u8(SRAM_PROGRESS_INDICATOR) < 2 {
            3
        } else {
            2
        }
    };
    if !skip_dark_world_override && ram.read_u8(SAVEGAME_IS_DARKWORLD) != 0 {
        track = if matches!(screen, 0x40 | 0x43 | 0x45 | 0x47) {
            13
        } else {
            9
        };
        if ram.read_u8(LINK_ITEM_MOON_PEARL) == 0 {
            track = 4;
        }
    }
    ram.write_u8(QUEUED_MUSIC_CONTROL, track);
    hooks.decompress_animated_tiles(ram, animated_tiles);
    hooks.initialize_tilesets(ram);
    hooks.load_screen_palette_set(ram);
    let palette_index = usize::from(screen);
    let sprite_palette = ram.read_u8(OVERWORLD_SPRITE_PALETTES + palette_index);
    hooks.load_palettes(
        ram,
        assets.background_palettes()[palette_index],
        sprite_palette,
    );
    hooks.set_overworld_background_color(ram);
    if ram.read_u8(MAIN_MODULE) == 8 {
        hooks.load_palettes_inner(ram);
    } else {
        hooks.copy_special_palettes_to_cache(ram);
    }
    set_fixed_color_and_scroll(ram);
    ram.write_u8(OVERWORLD_FIXED_COLOR_PLUS_MINUS, 0);
    hooks.initialize_follower(ram);
    if ram.read_u8(OVERWORLD_SCREEN_INDEX) & 0x3f == 0 {
        hooks.decode_animated_sprite_tile(ram, 0x1e);
    }
    ram.write_u8(SAVED_MODULE_FOR_MENU, 9);
    hooks.reload_overworld_sprites(ram);
    if ram.read_u16(OVERWORLD_SCREEN_INDEX) & 0x40 == 0 {
        hooks.initialize_mirror_portal(ram);
    }
    ram.write_u8(
        SOUND_EFFECT_AMBIENT,
        if ram.read_u8(SRAM_PROGRESS_INDICATOR) < 2 {
            1
        } else {
            5
        },
    );
    if ram.read_u8(FOLLOWER_INDICATOR) == 6 {
        ram.write_u8(FOLLOWER_INDICATOR, 0);
    }
    ram.write_u8(IS_STANDING_IN_DOORWAY, 0);
    ram.write_u8(BUTTON_MASK_B_Y, 0);
    ram.write_u8(BUTTON_B_FRAMES, 0);
    ram.write_u8(LINK_CANT_CHANGE_DIRECTION, 0);
    ram.write_u8(LINK_SPEED_SETTING, 0);
    ram.write_u8(DRAW_WATER_RIPPLES_OR_GRASS, 0);
    hooks.reset_torch_background_and_player(ram);
    if ram.read_u8(LINK_ITEM_MOON_PEARL) == 0 && ram.read_u8(SAVEGAME_IS_DARKWORLD) != 0 {
        ram.write_u8(LINK_IS_BUNNY, 1);
        ram.write_u8(LINK_IS_BUNNY_MIRROR, 1);
        ram.write_u8(LINK_PLAYER_STATE, 23);
        hooks.load_bunny_palettes(ram);
    }
    ram.write_u8(BGMODE, 9);
    ram.write_u8(DUNGEON_WANT_LIGHTS_OUT, 0);
    ram.write_u8(DUNGEON_HEADER_COLLISION, 0);
    ram.write_u8(LINK_IS_ON_LOWER_LEVEL, 0);
    ram.write_u8(LINK_IS_ON_LOWER_LEVEL_MIRROR, 0);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
    ram.write_u8(
        FLAG_UPDATE_HUD,
        ram.read_u8(FLAG_UPDATE_HUD).wrapping_add(1),
    );
    ram.write_u16(DUNGEON_SAVEGAME_STATE_BITS, 0);
    hooks.load_overworld_music_if_needed(ram);
}

pub fn recover_position_after_drowning(
    ram: &mut Ram<'_>,
    mut cache_quadrant_visits: impl FnMut(&mut Ram<'_>),
    mut reset_link_state: impl FnMut(&mut Ram<'_>),
    mut initialize_follower: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u16(LINK_X, ram.read_u16(LINK_X_CACHED));
    ram.write_u16(LINK_Y, ram.read_u16(LINK_Y_CACHED));
    ram.write_u16(OW_SCROLL_VARS0, ram.read_u16(ROOM_SCROLL_VARS_CACHED));
    ram.write_u16(
        OW_SCROLL_VARS0 + 4,
        ram.read_u16(ROOM_SCROLL_VARS_CACHED + 2),
    );
    ram.write_u16(OW_SCROLL_VARS1, ram.read_u16(ROOM_SCROLL_VARS_CACHED + 4));
    ram.write_u16(
        OW_SCROLL_VARS1 + 4,
        ram.read_u16(ROOM_SCROLL_VARS_CACHED + 6),
    );
    for offset in (0..8).step_by(2) {
        ram.write_u16(
            UP_DOWN_SCROLL_TARGET + offset,
            ram.read_u16(SCROLL_TARGETS_CACHED + offset),
        );
    }
    if ram.read_u8(PLAYER_IS_INDOORS) != 0 {
        let y = ram.read_u16(CAMERA_SCROLL_CACHED);
        let x = ram.read_u16(CAMERA_SCROLL_CACHED + 2);
        ram.write_u16(CAMERA_SCROLL_LOW, y);
        ram.write_u16(CAMERA_SCROLL_HIGH, y.wrapping_add(2));
        ram.write_u16(CAMERA_SCROLL_LOW + 4, x);
        ram.write_u16(CAMERA_SCROLL_HIGH + 4, x.wrapping_add(2));
    }
    ram.write_u16(QUADRANT_FULLSIZE_X, ram.read_u16(QUADRANTS_CACHED));
    ram.write_u16(LINK_QUADRANT_X, ram.read_u16(QUADRANTS_CACHED + 2));
    if ram.read_u8(PLAYER_IS_INDOORS) == 0 {
        ram.write_u16(
            CAMERA_SCROLL_HIGH,
            ram.read_u16(CAMERA_SCROLL_LOW).wrapping_sub(2),
        );
        ram.write_u16(
            CAMERA_SCROLL_HIGH + 4,
            ram.read_u16(CAMERA_SCROLL_LOW + 4).wrapping_sub(2),
        );
    }
    ram.write_u8(LINK_FACING, ram.read_u8(LINK_DIRECTION_CACHED));
    ram.write_u8(
        LINK_IS_ON_LOWER_LEVEL,
        ram.read_u8(LINK_DIRECTION_CACHED + 1),
    );
    ram.write_u8(
        LINK_IS_ON_LOWER_LEVEL_MIRROR,
        ram.read_u8(LINK_DIRECTION_CACHED + 2),
    );
    ram.write_u8(
        IS_STANDING_IN_DOORWAY,
        ram.read_u8(LINK_DIRECTION_CACHED + 3),
    );
    ram.write_u8(DUNG_CUR_FLOOR, ram.read_u8(LINK_DIRECTION_CACHED + 4));
    ram.write_u8(LINK_VISIBILITY, 0);
    ram.write_u8(COUNTDOWN_FOR_BLINK, 0x90);
    cache_quadrant_visits(ram);
    ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 0);
    reset_link_state(ram);
    ram.write_u8(TAGALONG_VAR5, 0);
    initialize_follower(ram);
    ram.write_u8(DUNG_FLAG_STATECHANGE_WATERPUZZLE, 0);
    ram.write_u8(OVERWORLD_MAP_STATE, 0);
    ram.write_u8(SUBSUBMODULE, 0);
    ram.write_u16(OVERWORLD_SCREEN_TRANSITION, 0);
    ram.write_u8(SUBMODULE, 0);
    if ram.read_u8(LINK_HEALTH_CURRENT) == 0 {
        ram.write_u8(MAPBAK_TM, ram.read_u8(TM_COPY));
        ram.write_u8(MAPBAK_TS, ram.read_u8(TS_COPY));
        ram.write_u8(SAVED_MODULE_FOR_MENU, ram.read_u8(MAIN_MODULE));
        ram.write_u8(MAIN_MODULE, 18);
        ram.write_u8(SUBMODULE, 1);
        ram.write_u8(COUNTDOWN_FOR_BLINK, 0);
    }
}

pub fn reset_ancilla_and_cutscene(
    ram: &mut Ram<'_>,
    mut terminate_select_interactives: impl FnMut(&mut Ram<'_>, u8),
) {
    terminate_select_interactives(ram, 0);
    ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 0);
    ram.write_u8(BUTTON_B_FRAMES, 0);
    ram.write_u8(BUTTON_MASK_B_Y, 0);
    ram.write_u8(LINK_FORCE_HOLD_SWORD_UP, 0);
    ram.write_u8(FLAG_LINK_IMMOBILIZED, 0);
}

pub fn set_target_overworld_warp_to_pyramid(
    ram: &mut Ram<'_>,
    mut load_overworld_from_dungeon: impl FnMut(&mut Ram<'_>),
    mut decompress_animated_overworld_tiles: impl FnMut(&mut Ram<'_>, u8),
    mut terminate_select_interactives: impl FnMut(&mut Ram<'_>, u8),
) {
    if ram.read_u8(MAIN_MODULE) != 21 {
        return;
    }
    load_overworld_from_dungeon(ram);
    decompress_animated_overworld_tiles(ram, 0x5a);
    reset_ancilla_and_cutscene(ram, &mut terminate_select_interactives);
}

#[must_use]
pub fn get_sign_text(assets: &impl OverworldAssets, area: usize) -> u16 {
    assets.sign_text()[area]
}

#[must_use]
pub fn get_background_palette(assets: &impl OverworldAssets, index: usize) -> u8 {
    assets.background_palettes()[index]
}

#[must_use]
pub fn get_overworld_sprite_ptr<'a>(
    ram: &Ram<'_>,
    assets: &'a impl OverworldAssets,
    area: usize,
) -> &'a [u8] {
    let progress = ram.read_u8(SRAM_PROGRESS_INDICATOR);
    let bank = if progress == 3 {
        2
    } else if progress == 2 {
        1
    } else {
        0
    };
    let offset = usize::from(assets.sprite_offsets()[area + bank * 144]);
    &assets.sprites()[offset..]
}

#[must_use]
pub fn get_overworld_bg_palette(assets: &impl OverworldAssets, index: usize) -> u8 {
    assets.background_palettes()[index]
}

pub fn load_sprite_graphics_properties(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    ram.slice_mut(OVERWORLD_SPRITE_GFX + 64, 64)
        .copy_from_slice(&assets.sprite_graphics()[0xc0..0x100]);
    ram.slice_mut(OVERWORLD_SPRITE_PALETTES + 64, 64)
        .copy_from_slice(&assets.sprite_palettes()[0xc0..0x100]);
    load_sprite_graphics_properties_light_world_only(ram, assets);
}

pub fn load_sprite_graphics_properties_light_world_only(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
) {
    let progress = ram.read_u8(SRAM_PROGRESS_INDICATOR);
    let bank = if progress < 2 {
        0
    } else if progress != 3 {
        1
    } else {
        2
    };
    let start = bank * 64;
    ram.slice_mut(OVERWORLD_SPRITE_GFX, 64)
        .copy_from_slice(&assets.sprite_graphics()[start..start + 64]);
    ram.slice_mut(OVERWORLD_SPRITE_PALETTES, 64)
        .copy_from_slice(&assets.sprite_palettes()[start..start + 64]);
}

/// Expands the custom bank-02 map stream used by the overworld data.
///
/// The return value is the number of bytes written, exactly like the C helper.
/// Back-references are copied one byte at a time so overlapping runs retain
/// their original LZ-style behavior.
pub fn decompress_bank02(destination: &mut [u8], source: &[u8]) -> usize {
    let mut src = 0usize;
    let mut dst = 0usize;
    loop {
        let mut command = source[src];
        src += 1;
        if command == 0xff {
            return dst;
        }
        let length;
        if command & 0xe0 != 0xe0 {
            length = usize::from(command & 0x1f) + 1;
            command &= 0xe0;
        } else {
            length = usize::from(source[src]) + (usize::from(command & 3) << 8) + 1;
            src += 1;
            command = command.wrapping_shl(3) & 0xe0;
        }

        match command {
            0x00 => {
                destination[dst..dst + length].copy_from_slice(&source[src..src + length]);
                src += length;
                dst += length;
            }
            command if command & 0x80 != 0 => {
                let mut offset = usize::from(source[src]) << 8 | usize::from(source[src + 1]);
                src += 2;
                for _ in 0..length {
                    destination[dst] = destination[offset];
                    dst += 1;
                    offset += 1;
                }
            }
            command if command & 0x40 == 0 => {
                let value = source[src];
                src += 1;
                destination[dst..dst + length].fill(value);
                dst += length;
            }
            command if command & 0x20 == 0 => {
                let low = source[src];
                let high = source[src + 1];
                src += 2;
                for index in 0..length {
                    destination[dst + index] = if index & 1 == 0 { low } else { high };
                }
                dst += length;
            }
            _ => {
                let mut value = source[src];
                src += 1;
                for output in &mut destination[dst..dst + length] {
                    *output = value;
                    value = value.wrapping_add(1);
                }
                dst += length;
            }
        }
    }
}

/// Decodes one map32 cell into its four map16 quadrants in WRAM.
///
/// `destination` is the byte address corresponding to the C `uint16 *dst`.
/// The four lookup tables are assets 34 through 37 from `assets.h`.
pub fn parse_map32_definition(
    ram: &mut Ram<'_>,
    map32_to_map16: [&[u8]; 4],
    destination: usize,
    input: u16,
) {
    let aligned = input & !7;
    if aligned != ram.read_u16(MAP16_DECODE_LAST) {
        ram.write_u16(MAP16_DECODE_LAST, aligned);
        ram.write_u16(MAP16_DECODE_TMP, aligned >> 1);
        let asset_offset = usize::from((aligned >> 1) + (aligned >> 2));
        for (decode, table) in [
            MAP16_DECODE_0,
            MAP16_DECODE_1,
            MAP16_DECODE_2,
            MAP16_DECODE_3,
        ]
        .into_iter()
        .zip(map32_to_map16)
        {
            let packed = &table[asset_offset..asset_offset + 6];
            ram.write_u8(decode, packed[0]);
            ram.write_u8(decode + 2, packed[1]);
            ram.write_u8(decode + 4, packed[2]);
            ram.write_u8(decode + 6, packed[3]);
            ram.write_u8(decode + 1, packed[4] >> 4);
            ram.write_u8(decode + 3, packed[4] & 0x0f);
            ram.write_u8(decode + 5, packed[5] >> 4);
            ram.write_u8(decode + 7, packed[5] & 0x0f);
        }
    }
    let index = usize::from(input & 7);
    ram.write_u16(destination, ram.read_u16(MAP16_DECODE_0 + index));
    ram.write_u16(destination + 128, ram.read_u16(MAP16_DECODE_2 + index));
    ram.write_u16(destination + 2, ram.read_u16(MAP16_DECODE_1 + index));
    ram.write_u16(destination + 130, ram.read_u16(MAP16_DECODE_3 + index));
}

#[must_use]
pub fn read_tile_attribute(ram: &Ram<'_>, tile_attributes: &[u8], x: u16, y: u16) -> u8 {
    let x_offset = x.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_X_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_X);
    let y_offset = y.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_Y);
    let map_byte_offset = x_offset | y_offset.wrapping_shl(3);
    let map16 = ram.read_u16(DUNG_BG2 + usize::from(map_byte_offset & !1));
    tile_attributes[usize::from(map16)]
}

pub fn set_fixed_color_and_scroll(ram: &mut Ram<'_>) {
    ram.write_u8(TS_COPY, 0);
    let screen = ram.read_u16(OVERWORLD_SCREEN_INDEX);
    let mut palette = 0x19c6;
    if screen == 0x80 {
        if ram.read_u16(DUNGEON_ROOM) == 0x181 {
            ram.write_u8(TS_COPY, 1);
            palette = if screen & 0x40 != 0 { 0x2a32 } else { 0x2669 };
        }
    } else if screen != 0x81 {
        palette = 0;
        if screen != 0x5b && !matches!(screen & 0xbf, 3 | 5 | 7) {
            palette = if screen & 0x40 != 0 { 0x2a32 } else { 0x2669 };
        }
    }
    for address in [
        MAIN_PALETTE_BUFFER,
        AUX_PALETTE_BUFFER,
        MAIN_PALETTE_BUFFER + 64,
        AUX_PALETTE_BUFFER + 64,
    ] {
        ram.write_u16(address, palette);
    }
    ram.write_u8(COLDATA_COPY0, 0x20);
    ram.write_u8(COLDATA_COPY1, 0x40);
    ram.write_u8(COLDATA_COPY2, 0x80);
    if !matches!(screen, 0 | 0x40 | 0x5b) {
        if screen == 0x70 {
            ram.write_u8(TS_COPY, 1);
            ram.write_u8(
                FLAG_UPDATE_CGRAM,
                ram.read_u8(FLAG_UPDATE_CGRAM).wrapping_add(1),
            );
            return;
        }
        let color = if matches!(screen, 3 | 5 | 7) {
            0x8c4c26u32
        } else if matches!(screen, 0x43 | 0x45) {
            0x874a26
        } else {
            ram.write_u8(
                FLAG_UPDATE_CGRAM,
                ram.read_u8(FLAG_UPDATE_CGRAM).wrapping_add(1),
            );
            return;
        };
        ram.write_u8(COLDATA_COPY0, color as u8);
        ram.write_u8(COLDATA_COPY1, (color >> 8) as u8);
        ram.write_u8(COLDATA_COPY2, (color >> 16) as u8);
    }
    if ram.read_u8(SUBMODULE) != 4 {
        ram.write_u16(BG1VOFS_COPY2, ram.read_u16(BG2VOFS_COPY2));
        ram.write_u16(BG1HOFS_COPY2, ram.read_u16(BG2HOFS_COPY2));
        if screen & 0x3f == 0x1b {
            let delta = (ram.read_u16(BG2HOFS_COPY2).wrapping_sub(0x778) as i16) >> 1;
            ram.write_u16(
                BG1HOFS_COPY2,
                ram.read_u16(BG2HOFS_COPY2).wrapping_sub(delta as u16),
            );
            let vertical = ram.read_u16(BG1VOFS_COPY2);
            if vertical >= 0x6c0 {
                let adjusted = vertical.wrapping_sub(0x600) & 0x3ff;
                ram.write_u16(
                    BG1VOFS_COPY2,
                    if adjusted < 0x180 {
                        adjusted >> 1 | 0x600
                    } else {
                        0x6c0
                    },
                );
            } else {
                ram.write_u16(BG1VOFS_COPY2, (vertical & 0xff) >> 1 | 0x600);
            }
        }
    } else if screen & 0x3f == 0x1b {
        let horizontal = if ram.read_u8(OVERWORLD_TRANS_DIR) != 8 {
            0x838
        } else {
            ram.read_u16(BG2HOFS_COPY2)
        };
        ram.write_u16(BG1HOFS_COPY2, horizontal);
        ram.write_u16(BG1VOFS_COPY2, 0x6c0);
    }
    ram.write_u8(TS_COPY, 1);
    ram.write_u8(
        FLAG_UPDATE_CGRAM,
        ram.read_u8(FLAG_UPDATE_CGRAM).wrapping_add(1),
    );
}

#[must_use]
pub fn get_link_map16_coords(ram: &Ram<'_>) -> (u16, Point16U) {
    let direction = usize::from(ram.read_u8(LINK_FACING) >> 1);
    let x = ram
        .read_u16(LINK_X)
        .wrapping_add_signed(i16::from(BEST_ACTION_X[direction]))
        & !0x0f;
    let y = ram
        .read_u16(LINK_Y)
        .wrapping_add_signed(i16::from(BEST_ACTION_Y[direction]))
        & !0x0f;
    let vertical = y.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_Y);
    let horizontal = (x >> 3).wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_X_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_X);
    (
        vertical.wrapping_shl(3).wrapping_add(horizontal),
        Point16U { x, y },
    )
}

#[must_use]
pub fn get_map16_of_link_mult8<'a>(ram: &Ram<'_>, assets: &'a impl OverworldAssets) -> &'a [u16] {
    let x = ram.read_u16(LINK_X).wrapping_add(8) >> 3;
    let y = ram.read_u16(LINK_Y).wrapping_add(12);
    let vertical = y.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_Y);
    let horizontal = x.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_X_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_X);
    let position = vertical.wrapping_mul(8).wrapping_add(horizontal);
    let map16 = usize::from(ram.read_u16(DUNG_BG2 + usize::from(position & !1)));
    &assets.map16_to_map8()[map16 * 4..][..4]
}

pub fn check_special_switch_area(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let map8 = get_map16_of_link_mult8(ram, assets)[0] & 0x1ff;
    let screen = ram.read_u16(OVERWORLD_SCREEN_INDEX);
    for index in (0..4).rev() {
        if SPECIAL_SWITCH_MAP8[index] == map8 && SPECIAL_SWITCH_SCREEN[index] == screen {
            let direction = SPECIAL_SWITCH_DIRECTION[index];
            ram.write_u16(DUNGEON_ROOM, SPECIAL_SWITCH_EXIT[index]);
            ram.write_u8(OVERWORLD_TRANS_DIR, direction);
            ram.write_u8(OVERWORLD_TRANS_DIR2, direction);
            ram.write_u8(LINK_DIRECTION, direction);
            let direction_enum = u16::from(dir_to_enum(direction));
            ram.write_u16(BYTE_069C, direction_enum);
            ram.write_u16(OVERWORLD_SCREEN_TRANSITION, direction_enum);
            ram.write_u8(SUBMODULE, 23);
            ram.write_u8(MAIN_MODULE, 11);
            break;
        }
    }
}

pub fn scroll_and_check_for_special_overworld_exit(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
) {
    if ram.read_u8(OVERWORLD_TRANS_DIR2) != 0 {
        handle_map_scroll(ram, assets);
    }
    let map8 = get_map16_of_link_mult8(ram, assets)[0] & 0x1ff;
    let screen = ram.read_u16(OVERWORLD_SCREEN_INDEX);
    for index in (0..3).rev() {
        if SPECIAL_SWITCH_B_MAP8[index] == map8 && SPECIAL_SWITCH_B_SCREEN[index] == screen {
            let direction = SPECIAL_SWITCH_B_DIRECTION[index];
            ram.write_u8(LINK_DIRECTION, direction);
            let transition = u16::from(dir_to_enum(direction));
            ram.write_u8(BYTE_069C, transition as u8);
            ram.write_u16(OVERWORLD_SCREEN_TRANSITION, transition);
            ram.write_u8(SUBMODULE, 36);
            ram.write_u8(SUBSUBMODULE, 0);
            ram.write_u8(DUNGEON_ROOM, 0);
            break;
        }
    }
}

pub fn pick_hammer_sfx(ram: &mut Ram<'_>, assets: &impl OverworldAssets, map16: u16) {
    let map8 = assets.map16_to_map8()[usize::from(map16) * 4] & 0x1ff;
    let attribute = assets.map8_tile_attributes()[usize::from(map8)];
    let effect = match attribute {
        0x50..=0x51 => 26,
        0x52..=0x53 => 17,
        0x54..=0x57 => 5,
        _ => return,
    };
    ram.write_u8(SOUND_EFFECT_1, effect);
}

pub fn use_entrance(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut show_text_message: impl FnMut(&mut Ram<'_>),
) {
    let x_coord = ram.read_u16(LINK_X) >> 3;
    let y_coord = ram.read_u16(LINK_Y).wrapping_add(7);
    let mut position = (y_coord.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_Y))
    .wrapping_mul(8)
    .wrapping_add(
        x_coord.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_X_ADDR))
            & ram.read_u16(OVERWORLD_OFFSET_MASK_X),
    );
    let map16 = usize::from(ram.read_u16(DUNG_BG2 + usize::from(position)));
    let map8 = assets.map16_to_map8();
    let mut map8_offset = map16 * 4;
    if ram.read_u8(LINK_FACING) == 0 {
        let mut tile = map8[map8_offset + 1] & 0x41ff;
        if tile == 0xe9 {
            draw_open_entrance_tiles(ram, assets, position);
            return;
        }
        if matches!(tile, 0x149 | 0x169)
            && (tile & 0x20 == 0 || ram.read_u8(SRAM_PROGRESS_INDICATOR) & 0x0f < 3)
        {
            setup_big_overworld_door(ram, position, tile);
            return;
        }
        map8_offset = usize::from(ram.read_u16(DUNG_BG2 + usize::from(position) + 2)) * 4;
        tile = map8[map8_offset] & 0x41ff;
        if tile == 0x40e9 {
            position = position.wrapping_sub(2);
            draw_open_entrance_tiles(ram, assets, position);
            return;
        }
        if matches!(tile, 0x4149 | 0x4169) {
            position = position.wrapping_sub(2);
            if tile & 0x20 == 0 || ram.read_u8(SRAM_PROGRESS_INDICATOR) & 0x0f < 3 {
                setup_big_overworld_door(ram, position, tile);
                return;
            }
        }
    }
    if !lookup_in_ow_entrance_tab(map8[map8_offset + 2] & 0x1ff, map8[map8_offset + 3] & 0x1ff) {
        ram.write_u16(BIG_KEY_DOOR_MESSAGE_TRIGGERED, 0);
        return;
    }
    let Some(index) = lookup_in_ow_entrance_tab2(ram, assets, position) else {
        return;
    };
    let entrance = assets.entrance_ids()[index];
    let blocked = ram.read_u8(FOLLOWER_DROPPED) == 0
        && (ram.read_u8(LINK_POSE_FOR_ITEM) == 1
            || !can_enter_with_tagalong(ram, i32::from(entrance) - 1));
    if blocked {
        if ram.read_u16(BIG_KEY_DOOR_MESSAGE_TRIGGERED) == 0 {
            ram.write_u16(BIG_KEY_DOOR_MESSAGE_TRIGGERED, 1);
            ram.write_u16(DIALOGUE_MESSAGE_INDEX, 5);
            show_text_message(ram);
        }
    } else {
        ram.write_u8(WHICH_ENTRANCE, entrance);
        ram.write_u8(LINK_AUXILIARY_STATE, 0);
        ram.write_u8(LINK_INCAPACITATED_TIMER, 0);
        ram.write_u8(MAIN_MODULE, 15);
        ram.write_u8(SAVED_MODULE_FOR_MENU, 6);
        ram.write_u8(SUBMODULE, 0);
        ram.write_u8(SUBSUBMODULE, 0);
    }
}

fn draw_open_entrance_tiles(ram: &mut Ram<'_>, assets: &impl OverworldAssets, position: u16) {
    draw_map16_persist(ram, assets, position, 0xda4);
    draw_map16_persist(ram, assets, position.wrapping_add(2), 0xda6);
    ram.write_u8(SOUND_EFFECT_2, 21);
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
}

fn setup_big_overworld_door(ram: &mut Ram<'_>, position: u16, tile: u16) {
    ram.write_u16(DOOR_OPEN_CLOSED_COUNTER, 0);
    if tile & 0x20 != 0 {
        ram.write_u16(DOOR_OPEN_CLOSED_COUNTER, 24);
    }
    ram.write_u16(BIG_ROCK_STARTING_ADDRESS, position.wrapping_sub(0x80));
    ram.write_u8(SOUND_EFFECT_2, 21);
    ram.write_u8(SUBSUBMODULE, 0);
    ram.write_u8(DOOR_ANIMATION_STEP, 0);
    ram.write_u8(SUBMODULE, 12);
}

/// Applies an overworld tool to the Map16 tile under `(x, y)`.
///
/// `spawn_smashed_terrain` and `add_bush_poof` are the two cross-module calls
/// made by the original routine after a bush or patch of thick grass is cut.
/// Keeping them explicit lets this state mutation remain independently
/// comparable with the compiled C implementation.
pub fn tool_and_tile_interaction(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    x: u16,
    y: u16,
    mut spawn_smashed_terrain: impl FnMut(&mut Ram<'_>, u16, u16, u16),
    mut add_bush_poof: impl FnMut(&mut Ram<'_>, u16, u16),
) -> u16 {
    ram.write_u16(INTERACTING_TILE_POSITION, 0);
    ram.write_u16(INDEX_OF_INTERACTING_TILE, 0);
    let position = y.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_Y);
    let position = position.wrapping_mul(8).wrapping_add(
        x.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_X_ADDR))
            & ram.read_u16(OVERWORLD_OFFSET_MASK_X),
    );
    let attribute = ram.read_u16(DUNG_BG2 + usize::from(position));
    let item = ram.read_u8(LINK_ITEM_IN_HAND);

    let replacement = if item & 2 != 0 {
        if attribute == 0x21b {
            ram.write_u8(SOUND_EFFECT_1, 17);
            let _ = handle_peg_puzzles(ram, position);
            Some(0xdcb)
        } else {
            pick_hammer_sfx(ram, assets, attribute);
            return attribute;
        }
    } else if item & 0x40 == 0
        && matches!(
            attribute,
            0x34 | 0x71 | 0x35 | 0x10d | 0x10f | 0xe1 | 0xe2 | 0xda | 0xf8 | 0x10e
        )
    {
        if ram.read_u8(LINK_POSITION_MODE) != 1 {
            return attribute;
        }
        if ram.read_u16(OVERWORLD_SCREEN_INDEX) == 0x2a && position == 0x492 {
            ram.write_u16(INTERACTING_TILE_POSITION, position);
        }
        Some(0xdc9)
    } else if item & 0x40 == 0 && attribute == 0x37e {
        if ram.read_u8(LINK_POSITION_MODE) == 1 {
            return attribute;
        }
        ram.write_u16(INTERACTION_SCRATCH_X, x.wrapping_mul(8).wrapping_sub(8));
        ram.write_u16(INTERACTION_SCRATCH_Y, y.wrapping_sub(8) & !7);
        ram.write_u16(INDEX_OF_INTERACTING_TILE, 3);
        Some(0xdc5)
    } else if matches!(attribute, 0x36 | 0x72a) {
        let interaction = if attribute == 0x36 { 2 } else { 4 };
        if ram.read_u8(LINK_POSITION_MODE) != 1 {
            ram.write_u16(INTERACTION_SCRATCH_X, (x & !1).wrapping_mul(8));
            ram.write_u16(INTERACTION_SCRATCH_Y, y & !0xf);
            ram.write_u16(INDEX_OF_INTERACTING_TILE, interaction);
            Some(if attribute == 0x72a { 0xdc8 } else { 0xdc7 })
        } else {
            None
        }
    } else {
        return attribute;
    };

    if let Some(mut value) = replacement {
        let secret = reveal_secret(ram, assets, position);
        if secret != 0 {
            value = secret;
        }
        ram.write_u16(DUNG_BG2 + usize::from(position), value);
        memorize_map16_change(ram, position, value);
        draw_map16(ram, assets, position, value);
        ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
    }

    let map8_index = usize::from(attribute) * 4 + usize::from((y & 8) >> 2) + usize::from(x & 1);
    let map8 = assets.map16_to_map8()[map8_index] & 0x1ff;
    let result = assets.map8_tile_attributes()[usize::from(map8)];
    let interaction = ram.read_u16(INDEX_OF_INTERACTING_TILE);
    if interaction != 0 {
        let sx = ram.read_u16(INTERACTION_SCRATCH_X);
        let sy = ram.read_u16(INTERACTION_SCRATCH_Y);
        spawn_smashed_terrain(ram, interaction, sx, sy);
        add_bush_poof(ram, sx, sy);
    }
    u16::from(result)
}

pub fn memorize_map16_change(ram: &mut Ram<'_>, position: u16, value: u16) {
    if matches!(value, 0xdc5 | 0xdc9) {
        return;
    }
    let byte_offset = usize::from(ram.read_u16(NUM_MEMORIZED_TILES));
    ram.write_u16(MEMORIZED_TILE_VALUE + byte_offset, value);
    ram.write_u16(MEMORIZED_TILE_ADDR + byte_offset, position);
    ram.write_u16(NUM_MEMORIZED_TILES, (byte_offset as u16).wrapping_add(2));
}

/// Advances either overworld peg puzzle. The return value represents the C
/// call to `Overworld_DoMapUpdate32x32_B`, allowing the caller to perform that
/// translated map update without hiding an external gameplay dependency.
#[must_use]
pub fn handle_peg_puzzles(ram: &mut Ram<'_>, position: u16) -> bool {
    match ram.read_u16(OVERWORLD_SCREEN_INDEX) {
        7 => {
            if ram.read_u8(SAVE_OW_EVENT_INFO + 7) & 0x20 != 0 {
                return false;
            }
            let progress = ram.read_u16(PEG_PUZZLE_PROGRESS);
            let expected = if progress == 0xffff {
                None
            } else {
                LW_TURTLE_ROCK_PEG_POSITIONS.get(usize::from(progress >> 1))
            };
            if expected.copied() == Some(position) {
                ram.write_u16(SOUND_EFFECT_1, 0x2d00);
                let progress = progress.wrapping_add(2);
                ram.write_u16(PEG_PUZZLE_PROGRESS, progress);
                if progress == 6 {
                    ram.write_u16(SOUND_EFFECT_1, 0x1b00);
                    let event = ram.read_u8(SAVE_OW_EVENT_INFO + 7) | 0x20;
                    ram.write_u8(SAVE_OW_EVENT_INFO + 7, event);
                    ram.write_u8(SUBMODULE, 47);
                }
            } else {
                ram.write_u16(SOUND_EFFECT_1, 0x003c);
                ram.write_u16(PEG_PUZZLE_PROGRESS, 0xffff);
            }
        }
        98 => {
            let progress = ram.read_u16(PEG_PUZZLE_PROGRESS).wrapping_add(1);
            ram.write_u16(PEG_PUZZLE_PROGRESS, progress);
            if progress == 22 {
                let event = ram.read_u8(SAVE_OW_EVENT_INFO + 0x62) | 0x20;
                ram.write_u8(SAVE_OW_EVENT_INFO + 0x62, event);
                ram.write_u8(SOUND_EFFECT_2, 27);
                ram.write_u16(DOOR_OPEN_CLOSED_COUNTER, 0x50);
                ram.write_u16(BIG_ROCK_STARTING_ADDRESS, 0xd20);
                return true;
            }
        }
        _ => {}
    }
    false
}

pub fn terminate_waterfall_splashes(ram: &mut Ram<'_>) {
    if ram.read_u8(OVERWORLD_SCREEN_INDEX) == 0x0f {
        for slot in (0..5).rev() {
            if ram.read_u8(ANCILLA_TYPE + slot) == 0x41 {
                ram.write_u8(ANCILLA_TYPE + slot, 0);
            }
        }
    }
}

pub fn get_pit_destination(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let x = ram.read_u16(LINK_X) & !7;
    let y = ram.read_u16(LINK_Y) & !7;
    let row = y.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_Y);
    let position = row.wrapping_shl(3).wrapping_add(
        (x >> 3).wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_X_ADDR))
            & ram.read_u16(OVERWORLD_OFFSET_MASK_X),
    );
    let area = ram.read_u16(OVERWORLD_AREA_INDEX);
    let destination = (0..19).rev().find_map(|index| {
        (assets.fall_hole_positions()[index] == position && assets.fall_hole_areas()[index] == area)
            .then_some(assets.fall_hole_entrances()[index])
    });
    if let Some(entrance) = destination {
        ram.write_u8(WHICH_ENTRANCE, entrance);
    } else {
        ram.write_u8(SAVEGAME_IS_DARKWORLD, 0);
        ram.write_u8(WHICH_ENTRANCE, 130);
    }
    ram.write_u8(BYTE_010F, 0);
}

pub fn palette_anim_get_master_sword2(ram: &mut Ram<'_>) {
    let palette = ram.slice(AUX_PALETTE_BUFFER, 512).to_vec();
    ram.slice_mut(MAPBAK_PALETTE, 512).copy_from_slice(&palette);
    for index in 0..256 {
        ram.write_u16(AUX_PALETTE_BUFFER + index * 2, 0x7fff);
    }
    ram.write_u16(
        MAIN_PALETTE_BUFFER + 32 * 2,
        ram.read_u16(MAIN_PALETTE_BUFFER),
    );
    ram.write_u16(PALETTE_FILTER_COUNTDOWN, 0);
    ram.write_u16(DARKENING_OR_LIGHTENING_SCREEN, 2);
    ram.write_u8(SUBSUBMODULE, ram.read_u8(SUBSUBMODULE).wrapping_add(1));
}

pub fn palette_anim_get_master_sword3(ram: &mut Ram<'_>) {
    if ram.read_u16(DARKENING_OR_LIGHTENING_SCREEN) == 0
        && ram.read_u16(PALETTE_FILTER_COUNTDOWN) == 31
    {
        let palette = ram.slice(MAPBAK_PALETTE, 512).to_vec();
        ram.slice_mut(AUX_PALETTE_BUFFER, 512)
            .copy_from_slice(&palette);
        ram.write_u8(TS_COPY, 0);
    }
}

pub fn palette_anim_get_master_sword(
    ram: &mut Ram<'_>,
    mut palette_filter_blinding_white: impl FnMut(&mut Ram<'_>),
) {
    if ram.read_u8(SUBSUBMODULE) == 0 {
        palette_anim_get_master_sword2(ram);
        return;
    }
    palette_filter_blinding_white(ram);
    if ram.read_u16(DARKENING_OR_LIGHTENING_SCREEN) == 0xff {
        for index in 0x58..0x60 {
            ram.write_u16(MAIN_PALETTE_BUFFER + index * 2, 0);
            ram.write_u16(AUX_PALETTE_BUFFER + index * 2, 0);
        }
        ram.write_u16(PALETTE_FILTER_COUNTDOWN, 0);
        ram.write_u16(DARKENING_OR_LIGHTENING_SCREEN, 0);
        ram.write_u8(SUBMODULE, 0);
    } else {
        palette_anim_get_master_sword3(ram);
    }
}

pub fn overworld_func2b(
    ram: &mut Ram<'_>,
    palette_filter_blinding_white: impl FnMut(&mut Ram<'_>),
) {
    palette_anim_get_master_sword(ram, palette_filter_blinding_white);
}

pub fn ganon_tower_entrance_flash(
    ram: &mut Ram<'_>,
    mut palette_filter_blinding_white: impl FnMut(&mut Ram<'_>),
) {
    if ram.read_u8(SUBSUBMODULE) == 0 {
        ram.write_u8(SOUND_EFFECT_1, 0x2e);
        palette_anim_get_master_sword2(ram);
    } else {
        palette_filter_blinding_white(ram);
        if ram.read_u16(DARKENING_OR_LIGHTENING_SCREEN) == 0xff {
            ram.write_u16(PALETTE_FILTER_COUNTDOWN, 0xff);
            ram.write_u8(SUBSUBMODULE, ram.read_u8(SUBSUBMODULE).wrapping_add(1));
        } else {
            palette_anim_get_master_sword3(ram);
        }
    }
}

pub fn dw_death_mountain_palette_animation(ram: &mut Ram<'_>) {
    if ram.read_u8(TRIGGER_SPECIAL_ENTRANCE) != 0 {
        return;
    }
    let screen = ram.read_u8(OVERWORLD_SCREEN_INDEX);
    if !matches!(screen, 0x43 | 0x45 | 0x47) {
        return;
    }
    let frame = ram.read_u8(FRAME_COUNTER);
    if matches!(frame, 5 | 44 | 90) {
        for index in 1..8 {
            for base in [0x30usize, 0x38, 0x48, 0x70, 0x78] {
                let value = ram.read_u16(AUX_PALETTE_BUFFER + (base + index) * 2);
                ram.write_u16(MAIN_PALETTE_BUFFER + (base + index) * 2, value);
            }
        }
    } else if matches!(frame, 3 | 36 | 88) {
        if frame == 36 {
            ram.write_u8(SOUND_EFFECT_1, 54);
        }
        for index in 1..8 {
            for (group, base) in [0x30usize, 0x38, 0x48, 0x70, 0x78].into_iter().enumerate() {
                ram.write_u16(
                    MAIN_PALETTE_BUFFER + (base + index) * 2,
                    DW_PALETTE_ANIM[group * 7 + index - 1],
                );
            }
        }
    }
    ram.write_u8(
        FLAG_UPDATE_CGRAM,
        ram.read_u8(FLAG_UPDATE_CGRAM).wrapping_add(1),
    );
    let mut offset = 32usize;
    if matches!(screen, 0x43 | 0x45) {
        if ram.read_u8(SAVE_OW_EVENT_INFO + 0x43) & 0x20 != 0 {
            return;
        }
        offset = usize::from(frame & 0x0c) * 2;
    }
    for index in 0..8 {
        ram.write_u16(
            MAIN_PALETTE_BUFFER + (0x68 + index) * 2,
            DW_PALETTE_ANIM2[offset + index],
        );
    }
}

pub fn module09_whirlpool(
    ram: &mut Ram<'_>,
    mut palette_filter_whirlpool_blue: impl FnMut(&mut Ram<'_>),
    mut palette_filter_isolate_whirlpool_blue: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(
        NMI_DISABLE_CORE_UPDATES,
        ram.read_u8(NMI_DISABLE_CORE_UPDATES).wrapping_add(1),
    );
    match ram.read_u8(SUBSUBMODULE) {
        0 => {
            ram.write_u8(SOUND_EFFECT_1, 0x34);
            ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
            ram.write_u8(OVERWORLD_MAP_STATE, 0);
            ram.write_u16(PALETTE_FILTER_COUNTDOWN, 0);
            ram.write_u8(SUBSUBMODULE, 1);
        }
        1 => palette_filter_whirlpool_blue(ram),
        2 => palette_filter_isolate_whirlpool_blue(ram),
        3 => {
            ram.write_u8(COLDATA_COPY2, 0x9f);
            ram.write_u8(
                MUSIC_CONTROL,
                if ram.read_u8(SAVEGAME_IS_DARKWORLD) != 0 {
                    9
                } else {
                    2
                },
            );
            ram.write_u8(SUBMODULE, 0);
            ram.write_u8(SUBSUBMODULE, 0);
            ram.write_u8(OVERWORLD_MAP_STATE, 0);
            ram.write_u8(NMI_DISABLE_CORE_UPDATES, 0);
        }
        _ => {}
    }
}

pub fn scroll_to_land(
    ram: &mut Ram<'_>,
    mut operate_camera_scroll: impl FnMut(&mut Ram<'_>),
    mut handle_map_scroll: impl FnMut(&mut Ram<'_>),
) {
    let old_x = ram.read_u16(LINK_X);
    let target_x = ram.read_u16(LINK_X_CACHED);
    let mut x = old_x;
    if x != target_x {
        let delta = if x > target_x { u16::MAX } else { 1 };
        x = x.wrapping_add(delta);
        if x != target_x {
            x = x.wrapping_add(delta);
        }
        ram.write_u16(LINK_X, x);
    }
    let old_y = ram.read_u16(LINK_Y);
    let target_y = ram.read_u16(LINK_Y_CACHED);
    let mut y = old_y;
    if y != target_y {
        let delta = if y > target_y { u16::MAX } else { 1 };
        y = y.wrapping_add(delta);
        if y != target_y {
            y = y.wrapping_add(delta);
        }
        ram.write_u16(LINK_Y, y);
    }
    ram.write_u8(LINK_Y_VELOCITY, y.wrapping_sub(old_y) as u8);
    ram.write_u8(LINK_X_VELOCITY, x.wrapping_sub(old_x) as u8);
    if y == target_y && x == target_x {
        ram.write_u8(SUBSUBMODULE, ram.read_u8(SUBSUBMODULE).wrapping_add(1));
        ram.write_u8(LINK_INCAPACITATED_TIMER, 0);
        ram.write_u8(SET_WHEN_DAMAGING_ENEMIES, 0);
    }
    operate_camera_scroll(ram);
    if ram.read_u8(OVERWORLD_TRANS_DIR2) != 0 {
        handle_map_scroll(ram);
    }
}

pub fn module09_recover_from_drowning(
    ram: &mut Ram<'_>,
    mut scroll_toward_land: impl FnMut(&mut Ram<'_>),
    mut recover_position: impl FnMut(&mut Ram<'_>),
) {
    if ram.read_u8(SUBSUBMODULE) == 0 {
        scroll_toward_land(ram);
    } else {
        recover_position(ram);
    }
}

pub fn overworld_func2f(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    const POSITION: u16 = 0x720;
    const VALUE: u16 = 0x212;
    ram.write_u16(DUNG_BG2 + usize::from(POSITION), VALUE);
    memorize_map16_change(ram, POSITION, VALUE);
    draw_map16(ram, assets, POSITION, VALUE);
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
    ram.write_u8(SUBMODULE, 0);
}

#[must_use]
pub fn animate_entrance(ram: &mut Ram<'_>) -> EntranceAnimation {
    let sequence = ram.read_u8(TRIGGER_SPECIAL_ENTRANCE);
    ram.write_u8(FLAG_LINK_IMMOBILIZED, sequence);
    ram.write_u8(FLAG_UNK1, sequence);
    ram.write_u8(NMI_DISABLE_CORE_UPDATES, sequence);
    match sequence {
        1 => EntranceAnimation::PalaceOfDarkness,
        2 => EntranceAnimation::SkullWoods,
        3 => EntranceAnimation::MiseryMire,
        4 => EntranceAnimation::TurtleRock,
        5 => EntranceAnimation::GanonsTower,
        _ => panic!("invalid overworld entrance sequence {sequence}"),
    }
}

pub fn flute_menu_load_transport(
    ram: &mut Ram<'_>,
    mut load_bird_travel_position: impl FnMut(&mut Ram<'_>, u8),
) {
    ram.write_u16(NUM_MEMORIZED_TILES, 0);
    let position = ram.read_u8(BIRD_TRAVEL_VAR1);
    ram.write_u16(
        BIRD_TRAVEL_VAR1,
        ram.read_u16(BIRD_TRAVEL_VAR1).wrapping_shl(1),
    );
    load_bird_travel_position(ram, position);
}

#[allow(clippy::too_many_arguments)]
pub fn load_bird_travel_position(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    index: usize,
    mut load_gfx_and_screen_size: impl FnMut(&mut Ram<'_>),
    mut reset_sprites: impl FnMut(&mut Ram<'_>),
    mut reload_overworld_sprites: impl FnMut(&mut Ram<'_>),
    mut reset_torch_background_and_player: impl FnMut(&mut Ram<'_>),
) {
    let vertical = assets.bird_scroll_y()[index];
    for address in [BG1VOFS_COPY2, BG2VOFS_COPY2, BG1VOFS_COPY, BG2VOFS_COPY] {
        ram.write_u16(address, vertical);
    }
    let horizontal = assets.bird_scroll_x()[index];
    for address in [BG1HOFS_COPY2, BG2HOFS_COPY2, BG1HOFS_COPY, BG2HOFS_COPY] {
        ram.write_u16(address, horizontal);
    }
    ram.write_u16(LINK_Y, assets.bird_link_y()[index]);
    ram.write_u16(LINK_X, assets.bird_link_x()[index]);
    let unk1 = assets.bird_unk1()[index] as i16 as u16;
    let unk3 = assets.bird_unk3()[index] as i16 as u16;
    ram.write_u16(OVERWORLD_UNK1, unk1);
    ram.write_u16(OVERWORLD_UNK1 + 2, 0u16.wrapping_sub(unk1));
    ram.write_u16(OVERWORLD_UNK1 + 4, unk3);
    ram.write_u16(OVERWORLD_UNK1 + 6, 0u16.wrapping_sub(unk3));
    let screen = u16::from(assets.bird_screen_indices()[index]);
    ram.write_u16(OVERWORLD_AREA_INDEX, screen);
    ram.write_u16(OVERWORLD_SCREEN_INDEX, screen);
    let source = assets.bird_map16_source_offsets()[index];
    ram.write_u16(MAP16_LOAD_SRC_OFFSET, source);
    ram.write_u16(MAP16_LOAD_VAR2, (source.wrapping_sub(0x400) & 0xf80) >> 7);
    ram.write_u16(
        MAP16_LOAD_DST_OFFSET,
        (source.wrapping_sub(0x10) & 0x3e) >> 1,
    );
    let camera_y = assets.bird_camera_y()[index];
    ram.write_u16(CAMERA_SCROLL_LOW, camera_y);
    ram.write_u16(CAMERA_SCROLL_HIGH, camera_y.wrapping_sub(2));
    let camera_x = assets.bird_camera_x()[index];
    ram.write_u16(CAMERA_SCROLL_LOW + 4, camera_x);
    ram.write_u16(CAMERA_SCROLL_HIGH + 4, camera_x.wrapping_sub(2));
    ram.write_u16(OW_ENTRANCE_VALUE, 0);
    ram.write_u16(BIG_ROCK_STARTING_ADDRESS, 0);
    load_new_screen_properties(ram, &mut load_gfx_and_screen_size);
    reset_sprites(ram);
    reload_overworld_sprites(ram);
    ram.write_u8(IS_STANDING_IN_DOORWAY, 0);
    reset_torch_background_and_player(ram);
}

pub fn find_partner_whirlpool_exit(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut load_bird_travel_position: impl FnMut(&mut Ram<'_>, usize),
) -> bool {
    let screen = ram.read_u16(OVERWORLD_SCREEN_INDEX);
    let Some(index) = assets
        .whirlpool_areas()
        .iter()
        .rposition(|&area| area == screen)
    else {
        return false;
    };
    ram.write_u16(NUM_MEMORIZED_TILES, 0);
    load_bird_travel_position(ram, index + 9);
    true
}

pub fn flute_menu_load_selected_screen_palettes(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut load_screens_palette_set: impl FnMut(&mut Ram<'_>),
    mut load_palettes: impl FnMut(&mut Ram<'_>, u8, u8),
    mut set_background_color: impl FnMut(&mut Ram<'_>),
    mut load_palettes_inner: impl FnMut(&mut Ram<'_>),
) {
    load_screens_palette_set(ram);
    let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
    let background = assets.background_palettes()[screen];
    let sprites = ram.read_u8(OVERWORLD_SPRITE_PALETTES + screen);
    load_palettes(ram, background, sprites);
    set_background_color(ram);
    load_palettes_inner(ram);
}

pub fn copy_map16_to_buffer(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    source: &[u8],
    r20: u16,
    mut r14: usize,
    mut header_address: usize,
) {
    let mut source_offset =
        usize::from(ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x410) & 0x1fff);
    let mut destination_offset = usize::from(ram.read_u16(MAP16_LOAD_DST_OFFSET));
    for _ in 0..32 {
        let value = u16::from_le_bytes([source[source_offset], source[source_offset + 1]]);
        ram.write_u16(MAP16_ROW_BUFFER + destination_offset * 2, value);
        destination_offset = (destination_offset + 1) & 0x1f;
        source_offset = (source_offset + 2) & 0x1fff;
    }

    let mut offset = ram.read_u16(MAP16_LOAD_VAR2);
    let mut tilemap_address = 0u16;
    if offset >= 0x10 {
        offset &= 0x0f;
        tilemap_address = 0x800;
    }
    tilemap_address = tilemap_address.wrapping_add(offset * 64);
    let mut row_buffer = MAP16_ROW_BUFFER;
    for _ in 0..2 {
        ram.write_u16(header_address, tilemap_address | r20);
        header_address += 2;
        for _ in 0..16 {
            let map16 = usize::from(ram.read_u16(row_buffer));
            row_buffer += 2;
            let map8 = &assets.map16_to_map8()[map16 * 4..][..4];
            ram.write_u16(DUNG_BG2_ATTR_TABLE + r14, map8[0]);
            ram.write_u16(DUNG_BG2_ATTR_TABLE + r14 + 64, map8[2]);
            ram.write_u16(DUNG_BG2_ATTR_TABLE + r14 + 2, map8[1]);
            ram.write_u16(DUNG_BG2_ATTR_TABLE + r14 + 66, map8[3]);
            r14 += 4;
        }
        tilemap_address = tilemap_address.wrapping_add(0x400);
        r14 += 0x40;
    }
}

pub fn map16_to_map8(ram: &mut Ram<'_>, assets: &impl OverworldAssets, source: &[u8], r20: u16) {
    ram.write_u16(
        MAP16_LOAD_SRC_OFFSET,
        ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_add(0x1000),
    );
    let mut r14 = 0usize;
    let mut header_address = MAP16_STRIPE_HEADERS;
    for _ in 0..32 {
        copy_map16_to_buffer(ram, assets, source, r20, r14, header_address);
        r14 += 0x100;
        header_address += 4;
        ram.write_u16(
            MAP16_LOAD_SRC_OFFSET,
            ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x80),
        );
        ram.write_u16(
            MAP16_LOAD_VAR2,
            ram.read_u16(MAP16_LOAD_VAR2).wrapping_sub(1) & 0x1f,
        );
    }
}

pub fn decompress_and_draw_one_quadrant(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
    screen: usize,
) {
    let mut decoded = [0u8; 4096];
    decompress_bank02(&mut decoded, assets.overworld_hibytes(screen));
    for (index, &value) in decoded[..256].iter().enumerate() {
        ram.write_u8(0x14001 + index * 2, value);
    }
    decoded.fill(0);
    decompress_bank02(&mut decoded, assets.overworld_lobytes(screen));
    for (index, &value) in decoded[..256].iter().enumerate() {
        ram.write_u8(0x14000 + index * 2, value);
    }
    ram.write_u16(MAP16_DECODE_LAST, 0xffff);
    let tables = assets.map32_to_map16();
    for row in 0..16 {
        for column in 0..16 {
            let input = ram
                .read_u16(0x14000 + (row * 16 + column) * 2)
                .wrapping_mul(2);
            parse_map32_definition(ram, tables, destination + row * 256 + column * 4, input);
        }
    }
}

pub fn decompress_and_draw_all_quadrants(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let screen = usize::from(ram.read_u16(OVERWORLD_SCREEN_INDEX));
    for (destination, screen_offset) in [(0x2000, 0), (0x2040, 1), (0x3000, 8), (0x3040, 9)] {
        decompress_and_draw_one_quadrant(ram, assets, destination, screen + screen_offset);
    }
}

pub fn load_suboverlay_map32(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let screen = usize::from(ram.read_u16(OVERWORLD_SCREEN_INDEX));
    decompress_and_draw_one_quadrant(ram, assets, 0x4000, screen);
}

pub fn load_overworld_overlay(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    load_suboverlay_map32(ram, assets);
    let source = ram.slice(0x4000, 0x2001).to_vec();
    map16_to_map8(ram, assets, &source, 0x1000);
    ram.write_u8(NMI_DISABLE_CORE_UPDATES, 4);
    ram.write_u8(NMI_SUBROUTINE_INDEX, 4);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
}

pub fn handle_overlays_and_bomb_doors(
    ram: &mut Ram<'_>,
    mut load_event_overlay: impl FnMut(&mut Ram<'_>),
) {
    let screen = ram.read_u16(OVERWORLD_SCREEN_INDEX);
    if screen == 0x33 {
        ram.write_u16(DUNG_BG2 + 340 * 2, 0x20f);
    } else if screen == 0x2f {
        ram.write_u16(DUNG_BG2 + 1497 * 2, 0x20f);
    }
    let screen_byte = usize::from(screen as u8);
    let event = ram.read_u8(SAVE_OW_EVENT_INFO + screen_byte);
    if screen_byte < 0x80 && event & 0x20 != 0 {
        load_event_overlay(ram);
    }
    if event & 2 != 0 {
        let position = usize::from(SECONDARY_OVERLAY_PER_OW[usize::from(screen)]);
        ram.write_u16(DUNG_BG2 + position, 0xdb4);
        ram.write_u16(DUNG_BG2 + position + 2, 0xdb5);
    }
}

pub fn draw_quadrants_and_overlays(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    load_event_overlay: impl FnMut(&mut Ram<'_>),
) {
    decompress_and_draw_all_quadrants(ram, assets);
    for index in 0..64 {
        ram.write_u16(DUNG_BG1 + index * 2, 0xdc4);
    }
    let mut position = ram.read_u16(OW_ENTRANCE_VALUE);
    if !matches!(position, 0 | 0xffff) {
        let (first, second) = if position < 0x8000 {
            (0xda4, 0xda6)
        } else {
            position &= 0x1fff;
            (0xdb4, 0xdb5)
        };
        ram.write_u16(DUNG_BG2 + usize::from(position), first);
        memorize_map16_change(ram, position, first);
        ram.write_u16(DUNG_BG2 + usize::from(position.wrapping_add(2)), second);
        memorize_map16_change(ram, position.wrapping_add(2), second);
        ram.write_u16(OW_ENTRANCE_VALUE, 0);
    }
    handle_overlays_and_bomb_doors(ram, load_event_overlay);
}

pub fn some_tile_map_change(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    load_event_overlay: impl FnMut(&mut Ram<'_>),
) {
    decompress_and_draw_all_quadrants(ram, assets);
    for index in 0..64 {
        ram.write_u16(DUNG_BG1 + index * 2, 0xdc4);
    }
    handle_overlays_and_bomb_doors(ram, load_event_overlay);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
}

pub fn create_initial_new_screen_map_to_scroll(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut trigger_map_load_stripe: impl FnMut(&mut Ram<'_>, MapStripeAxis, u8),
) -> bool {
    let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
    let small = assets.overworld_map_is_small()[screen] != 0;
    let direction = ram.read_u8(OVERWORLD_TRANS_DIR2);
    match (small, direction) {
        (false, 8) => {
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_add(0x380),
            );
            ram.write_u16(MAP16_LOAD_VAR2, 31);
            trigger_map_load_stripe(ram, MapStripeAxis::Y, 7);
        }
        (false, 4) => {
            let position = ram.read_u16(MAP16_LOAD_SRC_OFFSET) & 0x7f;
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, position.wrapping_add(0x780));
            ram.write_u16(MAP16_LOAD_VAR2, 7);
            trigger_map_load_stripe(ram, MapStripeAxis::Y, 8);
            ram.write_u16(
                MAP16_LOAD_VAR2,
                ram.read_u16(MAP16_LOAD_VAR2).wrapping_add(9) & 0x1f,
            );
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0xb80),
            );
        }
        (false, 2) => {
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_add(14),
            );
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 31);
            trigger_map_load_stripe(ram, MapStripeAxis::X, 7);
        }
        (false, 1) => {
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET)
                    .wrapping_sub(0x60)
                    .wrapping_add(0x1e),
            );
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 7);
            trigger_map_load_stripe(ram, MapStripeAxis::X, 8);
            ram.write_u16(
                MAP16_LOAD_DST_OFFSET,
                ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_add(9) & 0x1f,
            );
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x2e),
            );
        }
        (true, 8) => {
            ram.write_u16(
                ORANGE_BLUE_BARRIER_STATE,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x700),
            );
            ram.write_u16(WORD_C174, ram.read_u16(MAP16_LOAD_DST_OFFSET));
            ram.write_u16(WORD_C176, 10);
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x1390);
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 0);
            ram.write_u16(MAP16_LOAD_VAR2, 31);
            trigger_map_load_stripe(ram, MapStripeAxis::Y, 7);
        }
        (true, 4) => {
            ram.write_u16(
                ORANGE_BLUE_BARRIER_STATE,
                u16::from(ram.read_u8(MAP16_LOAD_SRC_OFFSET)),
            );
            ram.write_u16(WORD_C174, ram.read_u16(MAP16_LOAD_DST_OFFSET));
            ram.write_u16(WORD_C176, 24);
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x790);
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 0);
            ram.write_u16(MAP16_LOAD_VAR2, 7);
            trigger_map_load_stripe(ram, MapStripeAxis::Y, 8);
            ram.write_u16(
                MAP16_LOAD_VAR2,
                ram.read_u16(MAP16_LOAD_VAR2).wrapping_add(9) & 0x1f,
            );
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0xb80),
            );
        }
        (true, 2) => {
            ram.write_u16(
                ORANGE_BLUE_BARRIER_STATE,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x20),
            );
            ram.write_u16(WORD_C174, 8);
            ram.write_u16(WORD_C176, ram.read_u16(MAP16_LOAD_VAR2));
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x44e);
            ram.write_u16(MAP16_LOAD_VAR2, 0);
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 31);
            trigger_map_load_stripe(ram, MapStripeAxis::X, 7);
        }
        (true, 1) => {
            ram.write_u16(
                ORANGE_BLUE_BARRIER_STATE,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x60),
            );
            ram.write_u16(WORD_C174, 0x18);
            ram.write_u16(WORD_C176, ram.read_u16(MAP16_LOAD_VAR2));
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x41e);
            ram.write_u16(MAP16_LOAD_VAR2, 0);
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 7);
            trigger_map_load_stripe(ram, MapStripeAxis::X, 8);
            ram.write_u16(
                MAP16_LOAD_DST_OFFSET,
                ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_add(9) & 0x1f,
            );
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x2e),
            );
        }
        _ => {
            ram.write_u8(SUBMODULE, 0);
            return false;
        }
    }
    true
}

pub fn buffer_and_build_map16_stripes_x(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut destination: usize,
) -> usize {
    let table_index = usize::from((ram.read_u8(OVERWORLD_TRANS_DIR2) >> 1) & 1);
    let mut position = ram
        .read_u16(MAP16_LOAD_SRC_OFFSET)
        .wrapping_sub(OVERWORLD_DRAW_STRIP[table_index]);
    let mut buffer_index = usize::from(ram.read_u16(MAP16_LOAD_VAR2));
    for _ in 0..32 {
        let tile = if position >= 0x2000 {
            0
        } else {
            ram.read_u16(DUNG_BG2 + usize::from(position & !1))
        };
        ram.write_u16(MAP16_ROW_BUFFER + buffer_index * 2, tile);
        buffer_index = (buffer_index + 1) & 0x1f;
        position = position.wrapping_add(128);
    }
    let mut offset = ram.read_u16(MAP16_LOAD_DST_OFFSET);
    let mut tilemap_address = 0u16;
    if offset >= 0x10 {
        offset &= 0x0f;
        tilemap_address = 0x400;
    }
    tilemap_address = tilemap_address.wrapping_add(offset * 2);
    let mut row_buffer = MAP16_ROW_BUFFER;
    for _ in 0..2 {
        ram.write_u16(destination, tilemap_address);
        ram.write_u16(destination + 33 * 2, tilemap_address.wrapping_add(1));
        destination += 2;
        for _ in 0..16 {
            let tile = usize::from(ram.read_u16(row_buffer));
            row_buffer += 2;
            let map8 = &assets.map16_to_map8()[tile * 4..][..4];
            ram.write_u16(destination, map8[0]);
            ram.write_u16(destination + 33 * 2, map8[1]);
            ram.write_u16(destination + 2, map8[2]);
            ram.write_u16(destination + 34 * 2, map8[3]);
            destination += 4;
        }
        destination += 33 * 2;
        tilemap_address = tilemap_address.wrapping_add(0x800);
    }
    destination
}

pub fn buffer_and_build_map16_stripes_y(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut destination: usize,
) -> usize {
    let table_index = 1 + usize::from((ram.read_u8(OVERWORLD_TRANS_DIR2) >> 2) & 1);
    let mut position = ram
        .read_u16(MAP16_LOAD_SRC_OFFSET)
        .wrapping_sub(OVERWORLD_DRAW_STRIP[table_index]);
    let mut buffer_index = usize::from(ram.read_u16(MAP16_LOAD_DST_OFFSET));
    for _ in 0..32 {
        let tile = if position >= 0x2000 {
            0
        } else {
            ram.read_u16(DUNG_BG2 + usize::from(position & !1))
        };
        ram.write_u16(MAP16_ROW_BUFFER + buffer_index * 2, tile);
        position = position.wrapping_add(2);
        buffer_index = (buffer_index + 1) & 0x1f;
    }
    let mut offset = ram.read_u16(MAP16_LOAD_VAR2);
    let mut tilemap_address = 0u16;
    if offset >= 0x10 {
        offset &= 0x0f;
        tilemap_address = 0x800;
    }
    tilemap_address = tilemap_address.wrapping_add(offset * 64);
    let mut row_buffer = MAP16_ROW_BUFFER;
    for _ in 0..2 {
        ram.write_u16(destination, tilemap_address);
        destination += 2;
        for _ in 0..16 {
            let tile = usize::from(ram.read_u16(row_buffer));
            row_buffer += 2;
            let map8 = &assets.map16_to_map8()[tile * 4..][..4];
            ram.write_u16(destination, map8[0]);
            ram.write_u16(destination + 32 * 2, map8[2]);
            ram.write_u16(destination + 2, map8[1]);
            ram.write_u16(destination + 33 * 2, map8[3]);
            destination += 4;
        }
        destination += 32 * 2;
        tilemap_address = tilemap_address.wrapping_add(0x400);
    }
    destination
}

fn build_full_stripe_during_transition(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut destination: usize,
    direction: u8,
) -> usize {
    match direction {
        8 => {
            ram.write_u16(destination, 0x80);
            destination = buffer_and_build_map16_stripes_y(ram, assets, destination + 2);
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x80),
            );
            ram.write_u16(
                MAP16_LOAD_VAR2,
                ram.read_u16(MAP16_LOAD_VAR2).wrapping_sub(1) & 0x1f,
            );
        }
        4 => {
            ram.write_u16(destination, 0x80);
            destination = buffer_and_build_map16_stripes_y(ram, assets, destination + 2);
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_add(0x80),
            );
            ram.write_u16(
                MAP16_LOAD_VAR2,
                ram.read_u16(MAP16_LOAD_VAR2).wrapping_add(1) & 0x1f,
            );
        }
        2 => {
            ram.write_u16(destination, 0x8040);
            destination = buffer_and_build_map16_stripes_x(ram, assets, destination + 2);
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(2),
            );
            ram.write_u16(
                MAP16_LOAD_DST_OFFSET,
                ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_sub(1) & 0x1f,
            );
        }
        1 => {
            ram.write_u16(destination, 0x8040);
            destination = buffer_and_build_map16_stripes_x(ram, assets, destination + 2);
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_add(2),
            );
            ram.write_u16(
                MAP16_LOAD_DST_OFFSET,
                ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_add(1) & 0x1f,
            );
        }
        _ => {}
    }
    destination
}

pub fn transition_scroll_and_load_map(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let direction = ram.read_u8(OVERWORLD_TRANS_DIR2);
    let destination = if matches!(direction, 1 | 2 | 4 | 8) {
        build_full_stripe_during_transition(ram, assets, UVRAM_DATA, direction)
    } else {
        ram.write_u8(SUBMODULE, 0);
        UVRAM_DATA
    };
    ram.write_u16(destination, 0xffff);
    ram.write_u16(destination + 2, 0xffff);
    if destination != UVRAM_DATA {
        ram.write_u8(NMI_SUBROUTINE_INDEX, 3);
    }
}

pub fn trigger_and_finish_map_load_stripe(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    axis: MapStripeAxis,
    count: u8,
) {
    let (direction, header) = match axis {
        MapStripeAxis::X => (2, 0x8040),
        MapStripeAxis::Y => (8, 0x80),
    };
    ram.write_u8(OVERWORLD_TRANS_DIR2, direction);
    ram.write_u8(NMI_SUBROUTINE_INDEX, 3);
    ram.write_u16(UVRAM_DATA, header);
    let mut destination = UVRAM_DATA + 2;
    for _ in 0..count {
        match axis {
            MapStripeAxis::X => {
                destination = buffer_and_build_map16_stripes_x(ram, assets, destination);
                ram.write_u16(
                    MAP16_LOAD_SRC_OFFSET,
                    ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(2),
                );
                ram.write_u16(
                    MAP16_LOAD_DST_OFFSET,
                    ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_sub(1) & 0x1f,
                );
            }
            MapStripeAxis::Y => {
                destination = buffer_and_build_map16_stripes_y(ram, assets, destination);
                ram.write_u16(
                    MAP16_LOAD_SRC_OFFSET,
                    ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x80),
                );
                ram.write_u16(
                    MAP16_LOAD_VAR2,
                    ram.read_u16(MAP16_LOAD_VAR2).wrapping_sub(1) & 0x1f,
                );
            }
        }
    }
    ram.write_u16(destination, 0xffff);
}

fn check_for_newly_loaded_map_area(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut destination: usize,
    direction: u8,
) -> usize {
    let source = ram.read_u16(MAP16_LOAD_SRC_OFFSET);
    let blocked = match direction {
        8 => source.wrapping_sub(0x80) & 0x8000 != 0,
        4 => source >= 0x1800,
        2 => source & 0x7f == 0,
        1 => source & 0x7f >= 0x60,
        _ => true,
    };
    if blocked {
        return destination;
    }
    let screen = usize::from(ram.read_u16(OVERWORLD_SCREEN_INDEX));
    if assets.overworld_map_is_small()[screen] == 0 {
        let header = if matches!(direction, 1 | 2) {
            0x8040
        } else {
            0x80
        };
        ram.write_u16(destination, header);
        destination = if matches!(direction, 1 | 2) {
            buffer_and_build_map16_stripes_x(ram, assets, destination + 2)
        } else {
            buffer_and_build_map16_stripes_y(ram, assets, destination + 2)
        };
    }
    match direction {
        8 => {
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, source.wrapping_sub(0x80));
            ram.write_u16(
                MAP16_LOAD_VAR2,
                ram.read_u16(MAP16_LOAD_VAR2).wrapping_sub(1) & 0x1f,
            );
        }
        4 => {
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, source.wrapping_add(0x80));
            ram.write_u16(
                MAP16_LOAD_VAR2,
                ram.read_u16(MAP16_LOAD_VAR2).wrapping_add(1) & 0x1f,
            );
        }
        2 => {
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, source.wrapping_sub(2));
            ram.write_u16(
                MAP16_LOAD_DST_OFFSET,
                ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_sub(1) & 0x1f,
            );
        }
        1 => {
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, source.wrapping_add(2));
            ram.write_u16(
                MAP16_LOAD_DST_OFFSET,
                ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_add(1) & 0x1f,
            );
        }
        _ => {}
    }
    destination
}

pub fn handle_map_scroll(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let transition = ram.read_u8(OVERWORLD_TRANS_DIR2);
    let direction = match transition {
        1 => 1,
        2 => 2,
        4..=6 => 4,
        8..=10 => 8,
        _ => {
            ram.write_u8(SUBMODULE, 0);
            ram.write_u16(UVRAM_DATA, 0xffff);
            ram.write_u16(UVRAM_DATA + 2, 0xffff);
            ram.write_u16(
                OVERWORLD_SCREEN_TRANSITION,
                ram.read_u16(OVERWORLD_TRANS_DIR2),
            );
            return;
        }
    };
    let destination = check_for_newly_loaded_map_area(ram, assets, UVRAM_DATA, direction);
    ram.write_u8(
        OVERWORLD_TRANS_DIR2,
        if matches!(transition, 5 | 6 | 9 | 10) {
            transition & 3
        } else {
            0
        },
    );
    ram.write_u16(destination, 0xffff);
    ram.write_u16(destination + 2, 0xffff);
    if destination != UVRAM_DATA {
        ram.write_u8(NMI_SUBROUTINE_INDEX, 3);
    }
    ram.write_u16(
        OVERWORLD_SCREEN_TRANSITION,
        ram.read_u16(OVERWORLD_TRANS_DIR2),
    );
}

pub fn camera_boundary_check(
    ram: &mut Ram<'_>,
    xa: usize,
    ya: usize,
    velocity: i16,
    r8: usize,
) -> i16 {
    let axis = ya >> 1;
    let camera_axis = r8 >> 1;
    let background_offset = if xa != 0 {
        BG2VOFS_COPY2
    } else {
        BG2HOFS_COPY2
    };
    let boundary = OW_SCROLL_VARS0 + axis * 2;
    if ram.read_u16(background_offset) == ram.read_u16(boundary) {
        ram.write_u16(OVERWORLD_UNK1 + axis * 2, 0);
        ram.write_u16(OVERWORLD_UNK1 + (axis ^ 1) * 2, 0);
        return 0;
    }
    ram.write_u16(
        background_offset,
        ram.read_u16(background_offset)
            .wrapping_add_signed(velocity),
    );
    let high_address = CAMERA_SCROLL_HIGH + camera_axis * 2;
    let low_address = CAMERA_SCROLL_LOW + camera_axis * 2;
    let camera = ram.read_u16(high_address).wrapping_add_signed(velocity);
    ram.write_u16(high_address, camera);
    ram.write_u16(low_address, camera.wrapping_add(2));
    let accumulator = OVERWORLD_UNK1 + axis * 2;
    let value = ram.read_u16(accumulator).wrapping_add(1);
    ram.write_u16(accumulator, value);
    if value.wrapping_sub(0x10) & 0x8000 == 0 {
        ram.write_u16(accumulator, value.wrapping_sub(0x10));
        ram.write_u16(
            OVERWORLD_TRANS_DIR2,
            ram.read_u16(OVERWORLD_TRANS_DIR2) | [8, 4, 2, 1][axis],
        );
    }
    let value = ram.read_u16(accumulator);
    ram.write_u16(OVERWORLD_UNK1 + (axis ^ 1) * 2, value.wrapping_neg());
    velocity
}

fn add_bg1_scroll(ram: &mut Ram<'_>, subpixel_address: usize, offset_address: usize, amount: u32) {
    let value =
        u32::from(ram.read_u16(subpixel_address)) | (u32::from(ram.read_u16(offset_address)) << 16);
    let value = value.wrapping_add(amount);
    ram.write_u16(subpixel_address, value as u16);
    ram.write_u16(offset_address, (value >> 16) as u16);
}

pub fn operate_camera_scroll(ram: &mut Ram<'_>) {
    let z = if ram.read_u8(ALLOW_SCROLL_Z) != 0 && ram.read_u16(LINK_Z) != 0xffff {
        ram.read_u16(LINK_Z)
    } else {
        0
    };
    let y = ram.read_u16(LINK_Y).wrapping_sub(z).wrapping_add(12);
    let y_velocity = ram.read_u8(LINK_Y_VELOCITY);
    if y_velocity != 0 {
        let negative = y_velocity & 0x80 != 0;
        let velocity = if negative { -1 } else { 1 };
        let count = if negative {
            (!y_velocity).wrapping_add(1)
        } else {
            y_velocity
        };
        let mut delta = 0i16;
        for _ in 0..count {
            if negative {
                if y <= ram.read_u16(CAMERA_SCROLL_LOW) {
                    delta = delta.wrapping_add(camera_boundary_check(ram, 6, 0, velocity, 0));
                }
            } else if y >= ram.read_u16(CAMERA_SCROLL_HIGH) {
                delta = delta.wrapping_add(camera_boundary_check(ram, 6, 2, velocity, 0));
            }
        }
        ram.write_u16(BYTE_069E, delta as u16);
        let overlay = ram.read_u8(OVERLAY_INDEX);
        if !matches!(overlay, 0x97 | 0x9d) && delta != 0 {
            let mut word_delta = delta as u16;
            let subpixel = if matches!(overlay, 0xb5 | 0xbe) {
                let value = (word_delta & 3) << 14;
                word_delta >>= 2;
                if word_delta >= 0x3000 {
                    word_delta |= 0xf000;
                }
                value
            } else {
                let value = (word_delta & 1) << 15;
                word_delta >>= 1;
                if word_delta >= 0x7000 {
                    word_delta |= 0xf000;
                }
                value
            };
            add_bg1_scroll(
                ram,
                BG1VOFS_SUBPIXEL,
                BG1VOFS_COPY2,
                u32::from(subpixel) | (u32::from(word_delta) << 16),
            );
            if ram.read_u8(OVERWORLD_SCREEN_INDEX) & 0x3f == 0x1b {
                ram.write_u16(
                    BG1VOFS_COPY2,
                    ram.read_u16(BG1VOFS_COPY2).clamp(0x600, 0x6c0),
                );
            }
        }
    }

    let x = ram.read_u16(LINK_X).wrapping_add(8);
    let x_velocity = ram.read_u8(LINK_X_VELOCITY);
    if x_velocity != 0 {
        let negative = x_velocity & 0x80 != 0;
        let velocity = if negative { -1 } else { 1 };
        let count = if negative {
            (!x_velocity).wrapping_add(1)
        } else {
            x_velocity
        };
        let mut delta = 0i16;
        for _ in 0..count {
            if negative {
                if x <= ram.read_u16(CAMERA_SCROLL_LOW + 4) {
                    delta = delta.wrapping_add(camera_boundary_check(ram, 0, 4, velocity, 4));
                }
            } else if x >= ram.read_u16(CAMERA_SCROLL_HIGH + 4) {
                delta = delta.wrapping_add(camera_boundary_check(ram, 0, 6, velocity, 4));
            }
        }
        ram.write_u16(BYTE_069E + 1, delta as u16);
        let overlay = ram.read_u8(OVERLAY_INDEX);
        if !matches!(overlay, 0x97 | 0x9d) && delta != 0 {
            let mut word_delta = delta as u16;
            let subpixel = if matches!(overlay, 0x95 | 0x9e) {
                let value = (word_delta & 3) << 14;
                word_delta >>= 2;
                if word_delta >= 0x3000 {
                    word_delta |= 0xf000;
                }
                value
            } else {
                let value = (word_delta & 1) << 15;
                word_delta >>= 1;
                if word_delta >= 0x7000 {
                    word_delta |= 0xf000;
                }
                value
            };
            add_bg1_scroll(
                ram,
                BG1HOFS_SUBPIXEL,
                BG1HOFS_COPY2,
                u32::from(subpixel) | (u32::from(word_delta) << 16),
            );
        }
    }

    if ram.read_u8(OVERWORLD_SCREEN_INDEX) != 0x47 {
        match ram.read_u8(OVERLAY_INDEX) {
            0x9c => {
                add_bg1_scroll(ram, BG1VOFS_SUBPIXEL, BG1VOFS_COPY2, 0xffff_e000);
                let value = ram
                    .read_u16(BG1VOFS_COPY2)
                    .wrapping_add(ram.read_u16(BYTE_069E));
                ram.write_u16(BG1VOFS_COPY2, value);
                ram.write_u16(BG1HOFS_COPY2, ram.read_u16(BG2HOFS_COPY2));
            }
            0x97 | 0x9d => {
                add_bg1_scroll(ram, BG1VOFS_SUBPIXEL, BG1VOFS_COPY2, 0x2000);
                add_bg1_scroll(ram, BG1HOFS_SUBPIXEL, BG1HOFS_COPY2, 0x2000);
            }
            _ => {}
        }
    }
    if ram.read_u16(DUNGEON_ROOM) == 0x181 {
        ram.write_u16(BG1VOFS_COPY2, ram.read_u16(BG2VOFS_COPY2) | 0x100);
        ram.write_u16(BG1HOFS_COPY2, ram.read_u16(BG2HOFS_COPY2));
    }
}

pub fn scroll_transition(
    ram: &mut Ram<'_>,
    mut initialize_sprite_slots: impl FnMut(&mut Ram<'_>),
) -> u16 {
    let counter = ram.read_u8(TRANSITION_COUNTER).wrapping_add(1);
    ram.write_u8(TRANSITION_COUNTER, counter);
    let transition = usize::from(ram.read_u16(OVERWORLD_SCREEN_TRANSITION));
    let delta = [-8i16, 8, -8, 8][transition];
    let threshold = [27u8, 27, 30, 30][transition];
    let camera_delta = [-0x70i16, 0x70, -0x70, 0x70][transition];
    let value;
    if transition < 2 {
        ram.write_u8(BYTE_069E, delta as u8);
        value = ram.read_u16(BG2VOFS_COPY2).wrapping_add_signed(delta);
        ram.write_u16(BG2VOFS_COPY2, value);
        let screen = ram.read_u8(OVERWORLD_SCREEN_INDEX);
        if !matches!(screen, 0x1b | 0x5b) {
            ram.write_u16(BG1VOFS_COPY2, value);
        }
        if counter >= threshold {
            ram.write_u16(LINK_Y, ram.read_u16(LINK_Y).wrapping_add_signed(delta));
        }
        if value != ram.read_u16(UP_DOWN_SCROLL_TARGET + transition * 2) {
            return value;
        }
        if transition == 0 {
            ram.write_u16(BG2VOFS_COPY2, value.wrapping_sub(2));
        }
        let coordinate = ram.read_u16(LINK_Y) & !7;
        ram.write_u16(LINK_Y, coordinate);
        let high = coordinate
            .wrapping_add_signed(camera_delta)
            .wrapping_add(11);
        ram.write_u16(CAMERA_SCROLL_HIGH, high);
        ram.write_u16(CAMERA_SCROLL_LOW, high.wrapping_add(2));
        ram.write_u16(OVERWORLD_UNK1, 0);
        ram.write_u16(OVERWORLD_UNK1 + 2, 0);
    } else {
        ram.write_u8(BYTE_069E + 1, delta as u8);
        value = ram.read_u16(BG2HOFS_COPY2).wrapping_add_signed(delta);
        ram.write_u16(BG2HOFS_COPY2, value);
        let screen = ram.read_u8(OVERWORLD_SCREEN_INDEX);
        if !matches!(screen, 0x1b | 0x5b) {
            ram.write_u16(BG1HOFS_COPY2, value);
        }
        if counter >= threshold {
            ram.write_u16(LINK_X, ram.read_u16(LINK_X).wrapping_add_signed(delta));
        }
        if value != ram.read_u16(UP_DOWN_SCROLL_TARGET + transition * 2) {
            return value;
        }
        let coordinate = ram.read_u16(LINK_X) & !7;
        ram.write_u16(LINK_X, coordinate);
        let high = coordinate
            .wrapping_add_signed(camera_delta)
            .wrapping_add(11);
        ram.write_u16(CAMERA_SCROLL_HIGH + 4, high);
        ram.write_u16(CAMERA_SCROLL_LOW + 4, high.wrapping_add(2));
        ram.write_u16(OVERWORLD_UNK1 + 4, 0);
        ram.write_u16(OVERWORLD_UNK1 + 6, 0);
    }
    let area_delta = [-8i16, 8, -1, 1][transition];
    let area = ram
        .read_u16(CURRENT_AREA_OF_PLAYER)
        .wrapping_shr(1)
        .wrapping_add_signed(area_delta);
    let big = ram.read_u16(OVERWORLD_AREA_IS_BIG) != 0;
    set_camera_boundaries(ram, big, usize::from(area));
    ram.write_u8(FLAG_OVERWORLD_AREA_DID_CHANGE, 1);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
    ram.write_u8(SUBSUBMODULE, 0);
    ram.write_u8(TRANSITION_COUNTER, 0);
    initialize_sprite_slots(ram);
    value
}

pub fn finalize_entry_onto_screen(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut handle_moving_animation: impl FnMut(&mut Ram<'_>),
    mut operate_camera_scroll: impl FnMut(&mut Ram<'_>),
) {
    handle_moving_animation(ram);
    let direction = usize::from(ram.read_u8(BYTE_069C));
    let mut coordinate = if direction & 1 != 0 { 2i32 } else { -2 };
    if direction & 2 != 0 {
        coordinate += i32::from(ram.read_u16(LINK_X));
        ram.write_u16(LINK_X, coordinate as u16);
    } else {
        coordinate += i32::from(ram.read_u16(LINK_Y));
        ram.write_u16(LINK_Y, coordinate as u16);
    }
    if coordinate & 0xfe == [0xe0i32, 8, 0xe0, 0x10][direction] {
        ram.write_u8(SUBMODULE, 0);
        ram.write_u8(SUBSUBMODULE, 0);
        let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
        let music = ram.read_u8(OVERWORLD_MUSIC + screen);
        ram.write_u8(SOUND_EFFECT_AMBIENT, music >> 4);
        if ram.read_u8(MUSIC_UNK1) == 0xf1 {
            ram.write_u8(MUSIC_CONTROL, music & 0x0f);
        }
    }
    operate_camera_scroll(ram);
    if ram.read_u8(OVERWORLD_TRANS_DIR2) != 0 {
        handle_map_scroll(ram, assets);
    }
}

pub fn overworld_func1f(
    ram: &mut Ram<'_>,
    mut handle_moving_animation: impl FnMut(&mut Ram<'_>),
    mut operate_camera_scroll: impl FnMut(&mut Ram<'_>),
) {
    handle_moving_animation(ram);
    let direction = ram.read_u8(BYTE_069C);
    let velocity = if direction & 1 != 0 { 1i8 } else { -1 };
    if direction & 2 != 0 {
        ram.write_u16(
            LINK_X,
            ram.read_u16(LINK_X)
                .wrapping_add_signed(i16::from(velocity)),
        );
        ram.write_u8(LINK_X_VELOCITY, velocity as u8);
    } else {
        ram.write_u16(
            LINK_Y,
            ram.read_u16(LINK_Y)
                .wrapping_add_signed(i16::from(velocity)),
        );
        ram.write_u8(LINK_Y_VELOCITY, velocity as u8);
    }
    let countdown = ram.read_u8(OW_COUNTDOWN_TRANSITION).wrapping_sub(1);
    ram.write_u8(OW_COUNTDOWN_TRANSITION, countdown);
    if countdown == 0 {
        ram.write_u8(MAIN_MODULE, 9);
        ram.write_u8(SUBMODULE, 0);
        ram.write_u8(SUBSUBMODULE, 0);
    }
    operate_camera_scroll(ram);
}

pub fn load_ambient_overlay(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    load_map_data: bool,
    mut draw_quadrants_and_overlays: impl FnMut(&mut Ram<'_>),
) {
    let saved_source = ram.read_u16(MAP16_LOAD_SRC_OFFSET);
    let saved_destination = ram.read_u16(MAP16_LOAD_DST_OFFSET);
    let saved_var2 = ram.read_u16(MAP16_LOAD_VAR2);
    let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
    if assets.overworld_map_is_small()[screen] != 0 {
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x390);
        ram.write_u16(MAP16_LOAD_VAR2, 0x1f);
        ram.write_u16(MAP16_LOAD_DST_OFFSET, 0);
    }
    if load_map_data {
        draw_quadrants_and_overlays(ram);
    }
    let source = ram.slice(0x2000, 0x2001).to_vec();
    map16_to_map8(ram, assets, &source, 0);
    ram.write_u16(MAP16_LOAD_VAR2, saved_var2);
    ram.write_u16(MAP16_LOAD_DST_OFFSET, saved_destination);
    ram.write_u16(MAP16_LOAD_SRC_OFFSET, saved_source);
    ram.write_u8(NMI_SUBROUTINE_INDEX, 4);
    ram.write_u8(NMI_DISABLE_CORE_UPDATES, 4);
    ram.write_u8(SUBMODULE, ram.read_u8(SUBMODULE).wrapping_add(1));
    ram.write_u8(INIDISP_COPY, 0);
}

pub fn load_ambient_overlay_false(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    draw_quadrants_and_overlays: impl FnMut(&mut Ram<'_>),
) {
    load_ambient_overlay(ram, assets, false, draw_quadrants_and_overlays);
}

pub fn load_and_build_screen(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    draw_quadrants_and_overlays: impl FnMut(&mut Ram<'_>),
) {
    load_ambient_overlay(ram, assets, true, draw_quadrants_and_overlays);
}

pub fn module08_load_and_advance(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    draw_quadrants_and_overlays: impl FnMut(&mut Ram<'_>),
) {
    load_and_build_screen(ram, assets, draw_quadrants_and_overlays);
    ram.write_u8(MAIN_MODULE, 16);
    ram.write_u8(SUBMODULE, 0);
    ram.write_u8(SUBSUBMODULE, 0);
}

pub fn initialize_mirror_hdma(ram: &mut Ram<'_>, mut hdma_setup: impl FnMut(MirrorHdmaSetup)) {
    ram.write_u8(HDMAEN_COPY, 0);
    for offset in [0, 10, 12, 14, 16] {
        ram.write_u16(MIRROR_VARS + offset, 0);
    }
    ram.write_u16(MIRROR_VARS + 20, 8);
    ram.write_u16(MIRROR_VARS + 22, 8);
    ram.write_u16(MIRROR_VARS + 18, 21);
    ram.write_u16(MIRROR_VARS + 2, (-0x200i16) as u16);
    ram.write_u16(MIRROR_VARS + 4, 0x200);
    ram.write_u16(MIRROR_VARS + 6, (-0x40i16) as u16);
    ram.write_u16(MIRROR_VARS + 8, 0x40);
    hdma_setup(MirrorHdmaSetup {
        channel6_address: 0xf2fb,
        channel7_address: 0xf2fb,
        transfer_unit: 0x42,
        channel6_register: 0x0d,
        channel7_register: 0x0f,
        indirect_bank: 0,
    });
    let horizontal_scroll = ram.read_u16(BG2HOFS_COPY2);
    for scanline in 0..240 {
        ram.write_u16(HDMA_TABLE_DYNAMIC + scanline * 2, horizontal_scroll);
    }
    ram.write_u8(HDMAEN_COPY, 0xc0);
}

fn shift_mirror_hdma_table(ram: &mut Ram<'_>) {
    let mut y = 232usize;
    loop {
        let value = ram.read_u16(HDMA_TABLE_DYNAMIC + (y - 8) * 2);
        for offset in [0, 2, 4, 6] {
            ram.write_u16(HDMA_TABLE_DYNAMIC + (y + offset) * 2, value);
        }
        y -= 8;
        if y == 0 {
            break;
        }
    }
}

pub fn mirror_warp_build_waving_hdma_table(
    ram: &mut Ram<'_>,
    mut run_animation_submodules: impl FnMut(&mut Ram<'_>),
) {
    run_animation_submodules(ram);
    if ram.read_u8(FRAME_COUNTER) & 1 != 0 {
        return;
    }
    shift_mirror_hdma_table(ram);
    let phase = usize::from(ram.read_u16(MIRROR_VARS) >> 1);
    let bound = i32::from(ram.read_u16(MIRROR_VARS + 2 + phase * 2));
    let mut t = i32::from(ram.read_u16(MIRROR_VARS + 12))
        + i32::from(ram.read_u16(MIRROR_VARS + 6 + phase * 2));
    if (((t - bound) ^ bound) & 0x8000) == 0 {
        t = bound;
        ram.write_u16(MIRROR_VARS + 10, 0);
        ram.write_u16(MIRROR_VARS + 14, 0);
        ram.write_u16(MIRROR_VARS, ram.read_u16(MIRROR_VARS) ^ 2);
    }
    ram.write_u16(MIRROR_VARS + 12, t as u16);
    t += i32::from(ram.read_u16(MIRROR_VARS + 14));
    ram.write_u16(MIRROR_VARS + 14, (t & 0xff) as u16);
    if t & 0x8000 != 0 {
        t |= 0xff;
    } else {
        t &= !0xff;
    }
    t = i32::from(ram.read_u16(MIRROR_VARS + 10)) + i32::from((t as u16).swap_bytes());
    ram.write_u16(MIRROR_VARS + 10, t as u16);
    if ram.read_u16(PALETTE_FILTER_COUNTDOWN) >= 0x30 && t & !7 == 0 {
        ram.write_u16(MIRROR_VARS + 2, (-0x100i16) as u16);
        ram.write_u16(MIRROR_VARS + 4, 0x100);
        ram.write_u8(SUBSUBMODULE, ram.read_u8(SUBSUBMODULE).wrapping_add(1));
        t = 0;
    }
    let value = (t + i32::from(ram.read_u16(BG2HOFS_COPY2))) as u16;
    for index in [0, 2, 4, 6] {
        ram.write_u16(HDMA_TABLE_DYNAMIC + index * 2, value);
    }
}

pub fn mirror_warp_build_dewaving_hdma_table(
    ram: &mut Ram<'_>,
    mut run_animation_submodules: impl FnMut(&mut Ram<'_>),
    mut set_fixed_color_and_scroll: impl FnMut(&mut Ram<'_>),
) {
    run_animation_submodules(ram);
    if ram.read_u8(FRAME_COUNTER) & 1 != 0 {
        return;
    }
    shift_mirror_hdma_table(ram);
    let value = [0xc0usize, 0xc8, 0xd0, 0xd8]
        .into_iter()
        .fold(0, |value, index| {
            value | ram.read_u16(HDMA_TABLE_DYNAMIC + index * 2)
        });
    if value != ram.read_u16(BG2HOFS_COPY2) {
        return;
    }
    ram.write_u8(HDMAEN_COPY, 0);
    ram.write_u8(SUBSUBMODULE, ram.read_u8(SUBSUBMODULE).wrapping_add(1));
    set_fixed_color_and_scroll(ram);
    if ram.read_u8(OVERWORLD_SCREEN_INDEX) & 0x3f != 0x1b {
        let horizontal = ram.read_u16(BG2HOFS_COPY2);
        ram.write_u16(BG1HOFS_COPY2, horizontal);
        ram.write_u16(BG1HOFS_COPY, horizontal);
        ram.write_u16(BG2HOFS_COPY, horizontal);
        let vertical = ram.read_u16(BG2VOFS_COPY2);
        ram.write_u16(BG1VOFS_COPY2, vertical);
        ram.write_u16(BG1VOFS_COPY, vertical);
        ram.write_u16(BG2VOFS_COPY, vertical);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn module09_mirror_warp(
    ram: &mut Ram<'_>,
    mut initialize_hdma: impl FnMut(&mut Ram<'_>),
    mut initialize_white_filter: impl FnMut(&mut Ram<'_>),
    mut load_gfx_and_screen_size: impl FnMut(&mut Ram<'_>),
    mut build_waving_table: impl FnMut(&mut Ram<'_>),
    mut build_dewaving_table: impl FnMut(&mut Ram<'_>),
    mut finalize_destination: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(
        NMI_DISABLE_CORE_UPDATES,
        ram.read_u8(NMI_DISABLE_CORE_UPDATES).wrapping_add(1),
    );
    match ram.read_u8(SUBSUBMODULE) {
        0 => {
            if ram.read_u8(OVERWORLD_SCREEN_INDEX) >= 0x80 {
                ram.write_u8(SUBMODULE, 0);
                ram.write_u8(SUBSUBMODULE, 0);
                ram.write_u8(OVERWORLD_MAP_STATE, 0);
                return;
            }
            ram.write_u8(MUSIC_CONTROL, 8);
            ram.write_u8(FLAG_OVERWORLD_AREA_DID_CHANGE, 8);
            ram.write_u8(COUNTDOWN_FOR_BLINK, 0x90);
            initialize_hdma(ram);
            let world = ram.read_u8(SAVEGAME_IS_DARKWORLD) ^ 0x40;
            ram.write_u8(SAVEGAME_IS_DARKWORLD, world);
            ram.write_u16(PEG_PUZZLE_PROGRESS, 0);
            let screen = (ram.read_u8(OVERWORLD_SCREEN_INDEX) & 0x3f) | world;
            ram.write_u8(OVERWORLD_SCREEN_INDEX, screen);
            ram.write_u8(OVERWORLD_AREA_INDEX, screen);
            ram.write_u8(OVERWORLD_MAP_STATE, 0);
            initialize_white_filter(ram);
            load_gfx_and_screen_size(ram);
            ram.write_u8(SUBSUBMODULE, ram.read_u8(SUBSUBMODULE).wrapping_add(1));
        }
        1 => {
            ram.write_u8(SUBSUBMODULE, 2);
            ram.write_u8(HDMAEN_COPY, 0xc0);
            build_waving_table(ram);
        }
        2 => build_waving_table(ram),
        3 => build_dewaving_table(ram),
        _ => finalize_destination(ram),
    }
}

pub fn mirror_warp_finalize_and_load_destination(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut reset_hdma_and_iris: impl FnMut(&mut Ram<'_>),
    mut reload_previously_loaded_sheets: impl FnMut(&mut Ram<'_>),
) {
    reset_hdma_and_iris(ram);
    ram.write_u8(PALETTE_FILTER_COUNTDOWN, 0);
    ram.write_u8(DARKENING_OR_LIGHTENING_SCREEN, 0);
    reload_previously_loaded_sheets(ram);
    set_song_list(ram, assets);
    ram.write_u8(HDMAEN_COPY, 0x80);
    let music = ram.read_u8(OVERWORLD_MUSIC + usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX)));
    ram.write_u8(MUSIC_CONTROL, music & 0x0f);
    ram.write_u8(SOUND_EFFECT_AMBIENT, music >> 4);
    if ram.read_u8(OVERWORLD_SCREEN_INDEX) >= 0x40 && ram.read_u8(LINK_ITEM_MOON_PEARL) == 0 {
        ram.write_u8(MUSIC_CONTROL, 4);
    }
    ram.write_u8(SAVED_MODULE_FOR_MENU, ram.read_u8(SUBMODULE));
    ram.write_u8(SUBMODULE, 0);
    ram.write_u8(SUBSUBMODULE, 0);
    ram.write_u8(OVERWORLD_MAP_STATE, 0);
    ram.write_u8(NMI_DISABLE_CORE_UPDATES, 0);
}

pub fn draw_screen_at_current_mirror_position(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut draw_quadrants_and_overlays: impl FnMut(&mut Ram<'_>),
) {
    let source = ram.read_u16(MAP16_LOAD_SRC_OFFSET);
    let destination = ram.read_u16(MAP16_LOAD_DST_OFFSET);
    let variable = ram.read_u16(MAP16_LOAD_VAR2);
    let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
    if assets.overworld_map_is_small()[screen] != 0 {
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x390);
        ram.write_u16(MAP16_LOAD_VAR2, (0x390u16.wrapping_sub(0x400) & 0xf80) >> 7);
        ram.write_u16(
            MAP16_LOAD_DST_OFFSET,
            (0x390u16.wrapping_sub(0x10) & 0x3e) >> 1,
        );
    }
    draw_quadrants_and_overlays(ram);
    if ram.read_u8(SUBMODULE) == 44 {
        recover_mirror_bonk_changed_tiles(ram);
    }
    ram.write_u16(MAP16_LOAD_VAR2, variable);
    ram.write_u16(MAP16_LOAD_DST_OFFSET, destination);
    ram.write_u16(MAP16_LOAD_SRC_OFFSET, source);
}

#[allow(clippy::too_many_arguments)]
pub fn mirror_warp_load_sprites_and_colors(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut load_screen_palette_set: impl FnMut(&mut Ram<'_>),
    mut load_palettes: impl FnMut(&mut Ram<'_>, u8, u8),
    mut apply_special_overworld_palette: impl FnMut(&mut Ram<'_>),
    mut reset_sprites: impl FnMut(&mut Ram<'_>),
    mut reload_overworld_sprites: impl FnMut(&mut Ram<'_>),
    mut reset_link_items: impl FnMut(&mut Ram<'_>),
    mut reset_torch_background_and_player: impl FnMut(&mut Ram<'_>),
    mut initialize_mirror_portal: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u8(COUNTDOWN_FOR_BLINK, 0x90);
    let source_offset = ram.read_u16(MAP16_LOAD_SRC_OFFSET);
    let destination_offset = ram.read_u16(MAP16_LOAD_DST_OFFSET);
    let variable = ram.read_u16(MAP16_LOAD_VAR2);
    let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
    if assets.overworld_map_is_small()[screen] != 0 {
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x390);
        ram.write_u16(MAP16_LOAD_VAR2, 0x1f);
        ram.write_u16(MAP16_LOAD_DST_OFFSET, 0);
    }
    let source = ram.slice(DUNG_BG2, 0x2001).to_vec();
    map16_to_map8(ram, assets, &source, 0);
    ram.write_u16(MAP16_LOAD_VAR2, variable);
    ram.write_u16(MAP16_LOAD_DST_OFFSET, destination_offset);
    ram.write_u16(MAP16_LOAD_SRC_OFFSET, source_offset);
    load_screen_palette_set(ram);
    let background = assets.background_palettes()[screen];
    let sprites = ram.read_u8(OVERWORLD_SPRITE_PALETTES + screen);
    load_palettes(ram, background, sprites);
    apply_special_overworld_palette(ram);
    set_fixed_color_and_scroll(ram);
    if matches!(ram.read_u8(OVERWORLD_SCREEN_INDEX), 0x1b | 0x5b) {
        ram.write_u8(TS_COPY, 1);
    }
    for index in 0..96 {
        ram.write_u16(MAIN_PALETTE_BUFFER + (32 + index) * 2, 0x7fff);
    }
    ram.write_u16(MAIN_PALETTE_BUFFER, 0x7fff);
    if ram.read_u16(OVERWORLD_SCREEN_INDEX) == 0x5b {
        ram.write_u16(MAIN_PALETTE_BUFFER, 0);
        ram.write_u16(MAIN_PALETTE_BUFFER + 32 * 2, 0);
    }
    reset_sprites(ram);
    reload_overworld_sprites(ram);
    reset_link_items(ram);
    reset_torch_background_and_player(ram);
    ram.write_u8(LINK_PLAYER_STATE, 20);
    if ram.read_u16(OVERWORLD_SCREEN_INDEX) & 0x40 == 0 {
        initialize_mirror_portal(ram);
    }
}

#[must_use]
pub fn find_map16_vram_address(address: u16) -> u16 {
    (if address & 0x3f >= 0x20 { 0x400 } else { 0 })
        + (if address & 0x0fff >= 0x800 { 0x800 } else { 0 })
        + (address & 0x1f)
        + ((address & 0x780) >> 1)
}

pub fn draw_map16(ram: &mut Ram<'_>, assets: &impl OverworldAssets, position: u16, value: u16) {
    let vram_address = find_map16_vram_address(position);
    let destination = VRAM_UPLOAD_DATA + usize::from(ram.read_u16(VRAM_UPLOAD_OFFSET));
    let source = &assets.map16_to_map8()[usize::from(value) * 4..][..4];
    ram.write_u16(destination, vram_address.swap_bytes());
    ram.write_u16(destination + 2, 0x300);
    ram.write_u16(destination + 4, source[0]);
    ram.write_u16(destination + 6, source[1]);
    ram.write_u16(
        destination + 8,
        vram_address.wrapping_add(0x20).swap_bytes(),
    );
    ram.write_u16(destination + 10, 0x300);
    ram.write_u16(destination + 12, source[2]);
    ram.write_u16(destination + 14, source[3]);
    ram.write_u16(destination + 16, 0xffff);
    let offset = ram.read_u16(VRAM_UPLOAD_OFFSET).wrapping_add(16);
    ram.write_u16(VRAM_UPLOAD_OFFSET, offset);
}

pub fn draw_map16_persist(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    position: u16,
    value: u16,
) {
    ram.write_u16(DUNG_BG2 + usize::from(position & !1), value);
    draw_map16(ram, assets, position, value);
}

pub fn alter_tile_hardcore(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    position: u16,
    value: u16,
) {
    draw_map16_persist(ram, assets, position, value);
}

pub fn overworld_weathervane_explosion() {}

pub fn bomb_tiles_32x32(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    x: u16,
    y: u16,
    mut spawn_smashed_terrain: impl FnMut(&mut Ram<'_>, u8, u16, u16),
) {
    let x = x.wrapping_sub(23) & !7;
    let mut y = y.wrapping_sub(20) & !7;
    for _ in 0..3 {
        let mut tile_x = x;
        for _ in 0..3 {
            bomb_tile(ram, assets, tile_x, y, &mut spawn_smashed_terrain);
            tile_x = tile_x.wrapping_add(16);
        }
        y = y.wrapping_add(16);
    }
    ram.write_u16(WORD_0486, x);
    ram.write_u16(WORD_0488, y);
}

pub fn bomb_tile(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    x: u16,
    y: u16,
    mut spawn_smashed_terrain: impl FnMut(&mut Ram<'_>, u8, u16, u16),
) {
    let vertical = y.wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_Y);
    let horizontal = (x >> 3).wrapping_sub(ram.read_u16(OVERWORLD_OFFSET_BASE_X_ADDR))
        & ram.read_u16(OVERWORLD_OFFSET_MASK_X);
    let position = vertical.wrapping_shl(3).wrapping_add(horizontal);
    if ram.read_u8(FOLLOWER_INDICATOR) != 13 {
        let map16 = ram.read_u16(DUNG_BG2 + usize::from(position & !1));
        let replacement = match map16 {
            0x36 => Some((2, 0xdc7)),
            0x72a => Some((4, 0xdc8)),
            0x37e => Some((3, 0xdc5)),
            _ => None,
        };
        if let Some((kind, default_tile)) = replacement {
            let secret = reveal_secret(ram, assets, position);
            let value = if secret != 0 { secret } else { default_tile };
            ram.write_u16(DUNG_BG2 + usize::from(position & !1), value);
            memorize_map16_change(ram, position, value);
            draw_map16(ram, assets, position, value);
            spawn_smashed_terrain(ram, kind, x & !7, y & !7);
            ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
            return;
        }
    }
    if reveal_secret(ram, assets, position) == 0xdb4 {
        ram.write_u16(DUNG_BG2 + usize::from(position & !1), 0xdb4);
        memorize_map16_change(ram, position, 0xdb4);
        draw_map16(ram, assets, position, 0xdb4);
        ram.write_u16(DUNG_BG2 + usize::from(position & !1) + 2, 0xdb5);
        memorize_map16_change(ram, position, 0xdb5);
        draw_map16(ram, assets, position.wrapping_add(2), 0xdb5);
        ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
        let screen = usize::from(ram.read_u16(OVERWORLD_SCREEN_INDEX));
        ram.write_u8(
            SAVE_OW_EVENT_INFO + screen,
            ram.read_u8(SAVE_OW_EVENT_INFO + screen) | 2,
        );
    }
}

#[must_use]
pub fn lift_small_overworld_object(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    map16: u16,
    position: u16,
    mut replacement: u16,
    point: Point16U,
) -> u8 {
    let secret = reveal_secret(ram, assets, position);
    if secret != 0 {
        replacement = secret;
    }
    ram.write_u16(DUNG_BG2 + usize::from(position & !1), replacement);
    memorize_map16_change(ram, position, replacement);
    draw_map16(ram, assets, position, replacement);
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
    let quadrant =
        usize::from(map16) * 4 + usize::from(point.x & 8 != 0) * 2 + usize::from(point.y & 8 != 0);
    let map8 = assets.map16_to_map8()[quadrant] & 0x1ff;
    assets.map8_tile_attributes()[usize::from(map8)]
}

#[must_use]
pub fn smash_rock_pile_from_lift(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    map16: u16,
    position: u16,
    piece: usize,
    mut point: Point16U,
    mut update_map_32x32: impl FnMut(&mut Ram<'_>),
) -> u8 {
    let tile_deltas = [0i16, -1, -64, -65];
    let y_deltas = [0i16, 0, -64, -64];
    let x_deltas = [0i16, -1, 0, -1];
    let base = (position >> 1).wrapping_add_signed(tile_deltas[piece]);
    let base = base.wrapping_mul(2);
    ram.write_u16(BIG_ROCK_STARTING_ADDRESS, base);
    ram.write_u16(DOOR_OPEN_CLOSED_COUNTER, 40);
    ram.write_u16(0, point.y);
    ram.write_u16(2, point.x);
    let secret = reveal_secret(ram, assets, base);
    point.y = ram.read_u16(0);
    point.x = ram.read_u16(2);
    if secret == 0xffff {
        let screen = usize::from(ram.read_u16(OVERWORLD_SCREEN_INDEX));
        ram.write_u8(
            SAVE_OW_EVENT_INFO + screen,
            ram.read_u8(SAVE_OW_EVENT_INFO + screen) | 0x20,
        );
        ram.write_u8(SOUND_EFFECT_2, 27);
        ram.write_u16(DOOR_OPEN_CLOSED_COUNTER, 80);
    }
    point.x = point.x.wrapping_add_signed(x_deltas[piece] * 2);
    point.y = point.y.wrapping_add_signed(y_deltas[piece] * 2);
    update_map_32x32(ram);
    let quadrant =
        usize::from(map16) * 4 + usize::from(point.x & 8 != 0) * 2 + usize::from(point.y & 8 != 0);
    let map8 = assets.map16_to_map8()[quadrant] & 0x1ff;
    assets.map8_tile_attributes()[usize::from(map8)]
}

#[must_use]
pub fn smash_rock_pile(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    down_one_tile: bool,
    mut update_map_32x32: impl FnMut(&mut Ram<'_>),
) -> Option<u8> {
    let saved_y = ram.read_u16(LINK_Y);
    if down_one_tile {
        ram.write_u16(LINK_Y, saved_y.wrapping_add(8));
    }
    let (position, point) = get_link_map16_coords(ram);
    ram.write_u16(LINK_Y, saved_y);
    let map16 = ram.read_u16(DUNG_BG2 + usize::from(position & !1));
    if (0x226..=0x229).contains(&map16) {
        Some(smash_rock_pile_from_lift(
            ram,
            assets,
            map16,
            position,
            usize::from(map16 - 0x226),
            point,
            &mut update_map_32x32,
        ))
    } else if map16 == 0x36 {
        Some(lift_small_overworld_object(
            ram, assets, map16, position, 0xdc7, point,
        ))
    } else {
        None
    }
}

#[must_use]
pub fn handle_liftable_tiles(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut update_map_32x32: impl FnMut(&mut Ram<'_>),
) -> u8 {
    let (position, point) = get_link_map16_coords(ram);
    let map16 = ram.read_u16(DUNG_BG2 + usize::from(position & !1));
    let rock_piece = match map16 {
        0x36d | 0x23b => Some(0),
        0x36e | 0x23c => Some(1),
        0x374 | 0x23d => Some(2),
        0x375 | 0x23e => Some(3),
        _ => None,
    };
    if let Some(piece) = rock_piece {
        return smash_rock_pile_from_lift(
            ram,
            assets,
            map16,
            position,
            piece,
            point,
            &mut update_map_32x32,
        );
    }
    let replacement = match map16 {
        0x36 => Some(0xdc7),
        0x72a => Some(0xdc8),
        0x20f | 0x239 => Some(0xdca),
        0x101 => Some(0xdc6),
        _ => None,
    };
    if let Some(replacement) = replacement {
        return lift_small_overworld_object(ram, assets, map16, position, replacement, point);
    }
    let quadrant =
        usize::from(map16) * 4 + usize::from(point.x & 8 != 0) * 2 + usize::from(point.y & 8 != 0);
    let map8 = assets.map16_to_map8()[quadrant] & 0x1ff;
    assets.map8_tile_attributes()[usize::from(map8)]
}

pub fn alter_weathervane(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut update_map_32x32: impl FnMut(&mut Ram<'_>),
) {
    ram.write_u16(DOOR_OPEN_CLOSED_COUNTER, 0x68);
    ram.write_u16(BIG_ROCK_STARTING_ADDRESS, 0xc3e);
    update_map_32x32(ram);
    draw_map16_persist(ram, assets, 0xc42, 0xe21);
    draw_map16_persist(ram, assets, 0xcc2, 0xe25);
    ram.write_u8(
        SAVE_OW_EVENT_INFO + 0x18,
        ram.read_u8(SAVE_OW_EVENT_INFO + 0x18) | 0x20,
    );
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
}

pub fn open_gargoyles_domain(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    for (position, value) in [
        (0xd3e, 0xe1b),
        (0xd40, 0xe1c),
        (0xdbe, 0xe1d),
        (0xdc0, 0xe1e),
        (0xe3e, 0xe1f),
        (0xe40, 0xe20),
    ] {
        draw_map16_persist(ram, assets, position, value);
    }
    ram.write_u8(
        SAVE_OW_EVENT_INFO + 0x58,
        ram.read_u8(SAVE_OW_EVENT_INFO + 0x58) | 0x20,
    );
    ram.write_u8(SOUND_EFFECT_2, 0x1b);
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
}

pub fn create_pyramid_hole(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    for (position, value) in [
        (0x3bc, 0xe3f),
        (0x3be, 0xe40),
        (0x3c0, 0xe41),
        (0x43c, 0xe42),
        (0x43e, 0xe43),
        (0x440, 0xe44),
        (0x4bc, 0xe45),
        (0x4be, 0xe46),
        (0x4c0, 0xe47),
    ] {
        draw_map16_persist(ram, assets, position, value);
    }
    ram.write_u16(SOUND_EFFECT_AMBIENT, 0x3515);
    ram.write_u8(
        SAVE_OW_EVENT_INFO + 0x5b,
        ram.read_u8(SAVE_OW_EVENT_INFO + 0x5b) | 0x20,
    );
    ram.write_u8(SOUND_EFFECT_2, 3);
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
}

pub fn entrance_play_jingle(ram: &mut Ram<'_>) {
    ram.write_u8(SOUND_EFFECT_2, 27);
    ram.write_u8(TRIGGER_SPECIAL_ENTRANCE, 0);
    ram.write_u8(SUBSUBMODULE, 0);
    ram.write_u8(NMI_DISABLE_CORE_UPDATES, 0);
    ram.write_u8(FLAG_LINK_IMMOBILIZED, 0);
    ram.write_u8(FLAG_UNK1, 0);
    ram.write_u16(BG1_X_OFFSET, 0);
    ram.write_u16(BG1_Y_OFFSET, 0);
}

pub fn animate_entrance_palace_of_darkness(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let state = ram.read_u8(SUBSUBMODULE);
    if state > 4 {
        return;
    }
    let counter = ram
        .read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER)
        .wrapping_add(1);
    ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, counter);
    let target = if state == 0 { 0x40 } else { 0x20 };
    if counter != target {
        return;
    }
    if state == 4 {
        entrance_play_jingle(ram);
        return;
    }
    entrance_advance_and_boom(ram);
    let updates: &[(u16, u16)] = match state {
        0 => {
            ram.write_u8(
                SAVE_OW_EVENT_INFO + 0x5e,
                ram.read_u8(SAVE_OW_EVENT_INFO + 0x5e) | 0x20,
            );
            &[
                (0x1e6, 0xe31),
                (0x2ea, 0xe30),
                (0x26a, 0xe26),
                (0x2ea, 0xe27),
            ]
        }
        1 => &[(0x26a, 0xe28), (0x2ea, 0xe29)],
        2 => &[(0x26a, 0xe2a), (0x2ea, 0xe2b), (0x36a, 0xe2c)],
        3 => &[(0x26a, 0xe2d), (0x2ea, 0xe2e), (0x36a, 0xe2f)],
        _ => return,
    };
    for &(position, value) in updates {
        draw_map16_persist(ram, assets, position, value);
    }
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
}

pub fn animate_entrance_skull_woods(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let state = ram.read_u8(SUBSUBMODULE);
    if state > 4 {
        return;
    }
    let counter = ram
        .read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER)
        .wrapping_add(1);
    ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, counter);
    let target = if state == 0 { 4 } else { 12 };
    if counter != target {
        return;
    }
    ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0);
    ram.write_u8(SUBSUBMODULE, state.wrapping_add(1));
    let updates: &[(u16, u16)] = match state {
        0 => {
            let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
            ram.write_u8(
                SAVE_OW_EVENT_INFO + screen,
                ram.read_u8(SAVE_OW_EVENT_INFO + screen) | 0x20,
            );
            &[(0x409 * 2, 0xe06), (0x40a * 2, 0xe06)]
        }
        1 => &[
            (0x3c8 * 2, 0xe07),
            (0x3c9 * 2, 0xe08),
            (0x3ca * 2, 0xe09),
            (0x3cb * 2, 0xe0a),
        ],
        2 => &[
            (0x388 * 2, 0xe07),
            (0x389 * 2, 0xe08),
            (0x38a * 2, 0xe09),
            (0x38b * 2, 0xe0a),
        ],
        3 => &[
            (0x2c8 * 2, 0xe11),
            (0x2cb * 2, 0xe12),
            (0x308 * 2, 0xe0d),
            (0x309 * 2, 0xe0e),
            (0x30a * 2, 0xe0f),
            (0x30b * 2, 0xe10),
            (0x349 * 2, 0xe0b),
            (0x34a * 2, 0xe0c),
        ],
        4 => &[
            (0x2c8 * 2, 0xe13),
            (0x2cb * 2, 0xe14),
            (0x308 * 2, 0xe15),
            (0x309 * 2, 0xe16),
            (0x30a * 2, 0xe17),
            (0x30b * 2, 0xe18),
            (0x349 * 2, 0xe19),
            (0x34a * 2, 0xe1a),
        ],
        _ => return,
    };
    for &(position, value) in updates {
        draw_map16_persist(ram, assets, position, value);
    }
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
    ram.write_u8(SOUND_EFFECT_2, 0x16);
    if state == 4 {
        entrance_play_jingle(ram);
    }
}

pub fn animate_entrance_misery_mire(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let state = ram.read_u8(SUBSUBMODULE);
    if state >= 2 {
        let x = if ram.read_u8(FRAME_COUNTER) & 1 != 0 {
            u16::MAX
        } else {
            1
        };
        ram.write_u16(BG1_X_OFFSET, x);
        ram.write_u16(BG1_Y_OFFSET, 0u16.wrapping_sub(x));
    }
    if state > 5 {
        return;
    }
    let counter = ram
        .read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER)
        .wrapping_add(1);
    ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, counter);
    if state == 0 {
        if counter < 32 {
            return;
        }
        let bit = counter.wrapping_sub(32);
        if bit == 207 {
            ram.write_u8(SUBSUBMODULE, 1);
            ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0);
        }
        let enabled = MISERY_MIRE_ENTRANCE_BITS[usize::from(bit >> 3)] & (0x80 >> (bit & 7)) != 0;
        ram.write_u8(TS_COPY, u8::from(enabled));
        return;
    }
    if matches!(state, 1 | 2) {
        if counter == 16 {
            ram.write_u8(SUBSUBMODULE, state.wrapping_add(1));
            ram.write_u8(SOUND_EFFECT_AMBIENT, 7);
        }
        if counter != 72 {
            return;
        }
        entrance_advance_and_boom(ram);
        let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
        ram.write_u8(
            SAVE_OW_EVENT_INFO + screen,
            ram.read_u8(SAVE_OW_EVENT_INFO + screen) | 0x20,
        );
        draw_misery_mire_rows(ram, assets, 0xe48, 0);
        return;
    }
    if state == 3 && counter == 72 {
        entrance_advance_and_boom(ram);
        draw_misery_mire_rows(ram, assets, 0xe54, 1);
    } else if state == 4 && counter == 80 {
        entrance_advance_and_boom(ram);
        draw_misery_mire_rows(ram, assets, 0xe64, 2);
    } else if state == 5 && counter == 128 {
        entrance_play_jingle(ram);
        ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
    }
}

fn draw_misery_mire_rows(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut value: u16,
    upper_rows: usize,
) {
    const ROWS: [[u16; 4]; 4] = [
        [0x522, 0x524, 0x526, 0x528],
        [0x5a2, 0x5a4, 0x5a6, 0x5a8],
        [0x622, 0x624, 0x626, 0x628],
        [0x6a2, 0x6a4, 0x6a6, 0x6a8],
    ];
    let first = 2usize.saturating_sub(upper_rows);
    for row in &ROWS[first..] {
        for &position in row {
            draw_map16_persist(ram, assets, position, value);
            value = value.wrapping_add(1);
        }
    }
    for &position in &[0x722, 0x724, 0x726, 0x728] {
        draw_map16_persist(ram, assets, position, value);
        value = value.wrapping_add(1);
    }
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
}

pub fn animate_entrance_turtle_rock(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut approach_fixed_color: impl FnMut(&mut Ram<'_>, u8),
    mut restore_additive_palette: impl FnMut(&mut Ram<'_>, u8, u8),
    mut restore_subtractive_palette: impl FnMut(&mut Ram<'_>, u8, u8),
) {
    let x = if ram.read_u8(FRAME_COUNTER) & 1 != 0 {
        u16::MAX
    } else {
        1
    };
    ram.write_u16(BG1_X_OFFSET, x);
    ram.write_u16(BG1_Y_OFFSET, 0u16.wrapping_sub(x));
    let state = ram.read_u8(SUBSUBMODULE);
    match state {
        0..=3 => {
            if state == 0 {
                let screen = usize::from(ram.read_u16(OVERWORLD_SCREEN_INDEX));
                ram.write_u8(
                    SAVE_OW_EVENT_INFO + screen,
                    ram.read_u8(SAVE_OW_EVENT_INFO + screen) | 0x20,
                );
                approach_fixed_color(ram, 0);
            }
            ram.write_u16(UVRAM_DATA, 0x10 + u16::from(state) * 4);
            ram.write_u16(UVRAM_DATA + 2, 0xfe47);
            ram.write_u16(UVRAM_DATA + 4, 0x1e3);
            ram.write_u8(UVRAM_DATA + 6, 0xff);
            ram.write_u8(SUBSUBMODULE, state + 1);
            ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
        }
        4 => {
            for index in 0..8 {
                ram.write_u16(MAIN_PALETTE_BUFFER + (0x58 + index) * 2, 0);
                ram.write_u16(AUX_PALETTE_BUFFER + (0x68 + index) * 2, 0);
            }
            ram.write_u16(BG1VOFS_COPY2, ram.read_u16(BG2VOFS_COPY2));
            ram.write_u16(BG1HOFS_COPY2, ram.read_u16(BG2HOFS_COPY2));
            ram.write_u8(SUBSUBMODULE, 5);
            ram.write_u8(
                FLAG_UPDATE_CGRAM,
                ram.read_u8(FLAG_UPDATE_CGRAM).wrapping_add(1),
            );
        }
        5 => {
            entrance_draw_many_turtle_rock(ram, assets);
            ram.write_u8(TS_COPY, 1);
            ram.write_u8(CGWSEL_COPY, 2);
            ram.write_u8(CGADSUB_COPY, 0x22);
            let end = usize::from(ram.read_u16(VRAM_UPLOAD_OFFSET));
            let mut offset = 0usize;
            loop {
                ram.write_u16(
                    UVRAM_DATA + offset,
                    ram.read_u16(UVRAM_DATA + offset) | 0x10,
                );
                for entry in [4, 6] {
                    if ram.read_u16(UVRAM_DATA + offset + entry) == 0x8aa {
                        ram.write_u16(UVRAM_DATA + offset + entry, 0x1e3);
                    }
                }
                offset += 8;
                if offset == end {
                    break;
                }
            }
            ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0);
            ram.write_u8(SUBSUBMODULE, 6);
        }
        6 => {
            if ram.read_u8(FRAME_COUNTER) & 1 == 0 {
                let counter = ram.read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER);
                if counter & 7 == 0 {
                    restore_additive_palette(ram, 0xb0, 0xc0);
                    restore_subtractive_palette(ram, 0xd0, 0xe0);
                    ram.write_u8(
                        FLAG_UPDATE_CGRAM,
                        ram.read_u8(FLAG_UPDATE_CGRAM).wrapping_add(1),
                    );
                    ram.write_u8(SOUND_EFFECT_2, 2);
                }
                let counter = counter.wrapping_sub(1);
                ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, counter);
                if counter == 0 {
                    ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0x30);
                    ram.write_u8(SUBSUBMODULE, 7);
                }
            }
        }
        7 => {
            let counter = ram.read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER);
            if ram.read_u8(FRAME_COUNTER) & 1 == 0 && counter & 7 == 0 {
                ram.write_u8(SOUND_EFFECT_2, 2);
            }
            let counter = counter.wrapping_sub(1);
            ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, counter);
            if counter == 0 {
                entrance_draw_many_turtle_rock(ram, assets);
                ram.write_u8(TS_COPY, 0);
                ram.write_u8(CGWSEL_COPY, 0x82);
                ram.write_u8(CGADSUB_COPY, 0x20);
                ram.write_u8(SUBSUBMODULE, 8);
                ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
            }
        }
        8 => entrance_play_jingle(ram),
        _ => {}
    }
}

pub fn entrance_advance_and_boom(ram: &mut Ram<'_>) {
    ram.write_u8(SUBSUBMODULE, ram.read_u8(SUBSUBMODULE).wrapping_add(1));
    ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0);
    ram.write_u8(SOUND_EFFECT_1, 12);
    ram.write_u8(SOUND_EFFECT_2, 7);
}

pub fn entrance_draw_many_turtle_rock(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let mut value = 0xe78;
    for row in [0x99e, 0xa1e, 0xa9e, 0xb1e] {
        for column in 0..4 {
            draw_map16_persist(ram, assets, row + column * 2, value);
            value += 1;
        }
    }
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
    ram.write_u8(NMI_DISABLE_CORE_UPDATES, 1);
}

pub fn animate_entrance_ganons_tower(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    mut entrance_flash: impl FnMut(&mut Ram<'_>),
) {
    let state = ram.read_u8(SUBSUBMODULE);
    match state {
        0 | 1 => {
            let screen = usize::from(ram.read_u8(OVERWORLD_SCREEN_INDEX));
            ram.write_u8(
                SAVE_OW_EVENT_INFO + screen,
                ram.read_u8(SAVE_OW_EVENT_INFO + screen) | 0x20,
            );
            entrance_flash(ram);
        }
        2 => {
            entrance_flash(ram);
            if ram.read_u8(TS_COPY) == 0 {
                ram.write_u8(TS_COPY, 1);
                let counter = ram
                    .read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER)
                    .wrapping_add(1);
                ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, counter);
                if counter == 3 {
                    ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0);
                    ram.write_u8(SOUND_EFFECT_AMBIENT, 7);
                } else {
                    ram.write_u8(SUBSUBMODULE, 0);
                }
            }
        }
        3..=11 => {
            let threshold = [48u8, 48, 52, 32, 32, 32, 32, 32, 32][usize::from(state - 3)];
            let counter = ram
                .read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER)
                .wrapping_add(1);
            ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, counter);
            if counter != threshold {
                return;
            }
            if state == 11 {
                ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
            }
            entrance_advance_and_boom(ram);
            let updates: &[(u16, u16)] = match state {
                3 => &[
                    (0x45e, 0xe88),
                    (0x460, 0xe89),
                    (0x4de, 0xea2),
                    (0x4e0, 0xea3),
                    (0x55e, 0xe8a),
                    (0x560, 0xe8b),
                ],
                4 => &[
                    (0x45e, 0xe8c),
                    (0x460, 0xe8d),
                    (0x4de, 0xe8e),
                    (0x4e0, 0xe8f),
                    (0x55e, 0xe90),
                    (0x560, 0xe91),
                ],
                5 => &[
                    (0x45e, 0xe92),
                    (0x460, 0xe93),
                    (0x4de, 0xe94),
                    (0x4e0, 0xe94),
                    (0x55e, 0xe95),
                    (0x560, 0xe95),
                ],
                6 => &[
                    (0x45e, 0xe96),
                    (0x460, 0xe97),
                    (0x4de, 0xe98),
                    (0x4e0, 0xe99),
                ],
                7 => &[(0x4de, 0xe9a), (0x4e0, 0xe9b)],
                8 => &[
                    (0x4de, 0xe9c),
                    (0x4e0, 0xe9d),
                    (0x55e, 0xe9e),
                    (0x560, 0xe9f),
                ],
                9 => &[(0x55e, 0xe9a), (0x560, 0xe9b)],
                10 => &[
                    (0x55e, 0xe9c),
                    (0x560, 0xe9d),
                    (0x5de, 0xea0),
                    (0x5e0, 0xea1),
                ],
                11 => &[(0x5de, 0xe9a), (0x5e0, 0xe9b)],
                _ => unreachable!(),
            };
            for &(position, value) in updates {
                draw_map16_persist(ram, assets, position, value);
            }
            ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 1);
        }
        12 => {
            let counter = ram
                .read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER)
                .wrapping_add(1);
            ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, counter);
            if counter != 72 {
                return;
            }
            entrance_play_jingle(ram);
            ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0);
            ram.write_u8(MUSIC_CONTROL, 13);
            ram.write_u8(SOUND_EFFECT_AMBIENT, 9);
        }
        _ => {}
    }
}

#[must_use]
pub fn reveal_secret(ram: &mut Ram<'_>, assets: &impl OverworldAssets, position: u16) -> u16 {
    ram.write_u8(DUNG_SECRETS_UNK1, 0);
    let screen = usize::from(ram.read_u16(OVERWORLD_SCREEN_INDEX));
    if screen >= 0x80 {
        adjust_secret_for_powder(ram);
        return 0;
    }
    let secrets = assets.overworld_secrets();
    let mut offset = usize::from(assets.overworld_secret_offsets()[screen]);
    let data = loop {
        let encoded = u16::from_le_bytes([secrets[offset], secrets[offset + 1]]);
        if encoded == 0xffff {
            adjust_secret_for_powder(ram);
            return 0;
        }
        if encoded & 0x7fff == position {
            break secrets[offset + 2];
        }
        offset += 3;
    };
    if data != 0 && data < 0x80 {
        ram.write_u8(DUNG_SECRETS_UNK1, ram.read_u8(DUNG_SECRETS_UNK1) | data);
    }
    if data < 0x80 {
        adjust_secret_for_powder(ram);
        return 0;
    }
    ram.write_u8(DUNG_SECRETS_UNK1, 0xff);
    if data != 0x84 && ram.read_u8(SAVE_OW_EVENT_INFO + screen) & 2 == 0 {
        if screen == 0x5b && ram.read_u8(FOLLOWER_INDICATOR) != 13 {
            adjust_secret_for_powder(ram);
            return 0;
        }
        ram.write_u8(SOUND_EFFECT_2, 0x1b);
    } else if data == 0x82 && ram.read_u8(ENHANCED_FEATURES0 + 1) & 0x10 != 0 {
        ram.write_u8(SOUND_EFFECT_2, 0x1b);
    }
    adjust_secret_for_powder(ram);
    [0xdcc, 0x212, 0xffff, 0xdb4][usize::from((data & 0x0f) >> 1)]
}

pub fn adjust_secret_for_powder(ram: &mut Ram<'_>) {
    if ram.read_u8(LINK_ITEM_IN_HAND) & 0x40 != 0 {
        ram.write_u16(DUNG_SECRETS_UNK1, 4);
    }
}

pub fn recover_mirror_bonk_changed_tiles(ram: &mut Ram<'_>) {
    let count = usize::from(ram.read_u16(NUM_MEMORIZED_TILES) >> 1);
    for index in 0..count {
        let position = usize::from(ram.read_u16(MEMORIZED_TILE_ADDR + index * 2));
        let value = ram.read_u16(MEMORIZED_TILE_VALUE + index * 2);
        ram.write_u16(DUNG_BG2 + position, value);
    }
}

pub fn decompress_enemy_damage_subclasses(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    let compressed = assets.enemy_damage_data();
    ram.slice_mut(ENEMY_DAMAGE_SCRATCH, compressed.len())
        .copy_from_slice(compressed);
    for (index, &packed) in compressed[..0x800].iter().enumerate() {
        ram.write_u8(ENEMY_DAMAGE_DATA + index * 2, packed >> 4);
        ram.write_u8(ENEMY_DAMAGE_DATA + index * 2 + 1, packed & 0x0f);
    }
}

pub fn set_camera_boundaries(ram: &mut Ram<'_>, big: bool, area: usize) {
    let size = usize::from(big);
    let y_start = OVERWORLD_OFFSET_BASE_Y[area];
    let x_start = OVERWORLD_OFFSET_BASE_X[area];
    ram.write_u16(OW_SCROLL_VARS0, y_start);
    ram.write_u16(
        OW_SCROLL_VARS0 + 2,
        y_start.wrapping_add([0x11e, 0x31e][size]),
    );
    ram.write_u16(OW_SCROLL_VARS0 + 4, x_start);
    ram.write_u16(
        OW_SCROLL_VARS0 + 6,
        x_start.wrapping_add([0x100, 0x300][size]),
    );
    let up_down = UP_DOWN_SCROLL_TARGETS[area];
    let left_right = LEFT_RIGHT_SCROLL_TARGETS[area];
    ram.write_u16(UP_DOWN_SCROLL_TARGET, up_down);
    ram.write_u16(
        UP_DOWN_SCROLL_TARGET_END,
        up_down.wrapping_add([0x2e0, 0x4e0][size]),
    );
    ram.write_u16(LEFT_RIGHT_SCROLL_TARGET, left_right);
    ram.write_u16(
        LEFT_RIGHT_SCROLL_TARGET_END,
        left_right.wrapping_add([0x300, 0x500][size]),
    );
}

/// Apply the persistent event overlay for the current overworld screen.
/// This is a direct table-form translation of `Overworld_LoadEventOverlay`.
pub fn overworld_load_event_overlay(ram: &mut Ram<'_>) {
    let screen = ram.read_u16(OVERWORLD_SCREEN_INDEX);
    let mut put = |x: usize, y: usize, tile: u16| {
        ram.write_u16(DUNG_BG2 + (y * 64 + x) * 2, tile);
    };
    match screen {
        0..=2 => {
            put(11, 16, 0xe32);
            put(12, 16, 0xe32);
            put(13, 16, 0xe32);
            put(14, 16, 0xe32);
            put(11, 17, 0xe32);
            put(14, 17, 0xe32);
            put(12, 17, 0xe33);
            put(13, 17, 0xe34);
            put(11, 18, 0xe35);
            put(12, 18, 0xe36);
            put(13, 18, 0xe37);
            put(14, 18, 0xe38);
            put(11, 19, 0xe39);
            put(12, 19, 0xe3a);
            put(13, 19, 0xe3b);
            put(14, 19, 0xe3c);
            put(12, 20, 0xe3d);
            put(13, 20, 0xe3e);
        }
        3..=7 => {
            put(16, 14, 0x212);
        }
        8..=19 => {
            put(3, 10, 0x918);
            put(4, 10, 0x919);
            put(3, 11, 0x91a);
            put(4, 11, 0x91b);
        }
        20 => {
            put(25, 10, 0xdd1);
            put(26, 10, 0xdd2);
            put(25, 11, 0xdd7);
            put(26, 11, 0xdd8);
            put(25, 12, 0xdd9);
            put(26, 12, 0xdda);
        }
        21 | 22 | 23 | 24 | 25 | 32 | 33 => {
            put(31, 24, 0xe21);
            put(33, 24, 0xe21);
            put(32, 24, 0xe22);
            put(31, 25, 0xe23);
            put(32, 25, 0xe24);
            put(33, 25, 0xe25);
        }
        26 | 27 | 28 | 35 | 36 => {
            put(30, 39, 0xdc1);
            put(31, 39, 0xdc2);
            put(30, 40, 0xdbe);
            put(31, 40, 0xdbf);
            put(32, 39, 0xdc2);
            put(33, 39, 0xdc3);
            put(32, 40, 0xdbf);
            put(33, 40, 0xdc0);
        }
        29 | 30 | 31 | 34 | 37 | 38 | 39 | 40 | 41 | 42 | 43 | 107 => {
            put(24, 6, 0x918);
            put(25, 6, 0x919);
            put(24, 7, 0x91a);
            put(25, 7, 0x91b);
        }
        44 | 45 | 46 | 47 | 48 | 49 | 56 | 57 => {
            put(44, 6, 0x918);
            put(45, 6, 0x919);
            put(44, 7, 0x91a);
            put(45, 7, 0x91b);
        }
        50 | 51 | 52 | 53 | 54 | 55 | 119 => {
            put(6, 8, 0x918);
            put(7, 8, 0x919);
            put(6, 9, 0x91a);
            put(7, 9, 0x91b);
        }
        58 => {
            put(15, 20, 0x918);
            put(16, 20, 0x919);
            put(15, 21, 0x91a);
            put(16, 21, 0x91b);
        }
        59 | 123 => {
            put(22, 7, 0xddf);
            put(18, 8, 0xddf);
            put(16, 9, 0xddf);
            put(15, 10, 0xddf);
            put(14, 12, 0xddf);
            put(26, 14, 0xddf);
            put(23, 7, 0xde0);
            put(17, 9, 0xde0);
            put(24, 7, 0xde1);
            put(28, 8, 0xde1);
            put(29, 9, 0xde1);
            put(21, 11, 0xde1);
            put(29, 14, 0xde1);
            put(19, 8, 0xde2);
            put(20, 8, 0xde2);
            put(21, 8, 0xde2);
            put(25, 8, 0xde2);
            put(26, 8, 0xde2);
            put(27, 8, 0xde2);
            put(22, 8, 0xde3);
            put(18, 9, 0xde3);
            put(16, 10, 0xde3);
            put(15, 12, 0xde3);
            put(23, 8, 0xde4);
            put(19, 9, 0xde4);
            put(20, 9, 0xde4);
            put(24, 9, 0xde4);
            put(27, 9, 0xde4);
            put(17, 10, 0xde4);
            put(18, 10, 0xde4);
            put(19, 10, 0xde4);
            put(28, 10, 0xde4);
            put(16, 11, 0xde4);
            put(17, 11, 0xde4);
            put(18, 11, 0xde4);
            put(19, 11, 0xde4);
            put(16, 12, 0xde4);
            put(17, 12, 0xde4);
            put(15, 13, 0xde4);
            put(16, 13, 0xde4);
            put(15, 14, 0xde4);
            put(16, 14, 0xde4);
            put(19, 16, 0xde4);
            put(19, 17, 0xde4);
            put(20, 17, 0xde4);
            put(19, 18, 0xde4);
            put(24, 8, 0xde5);
            put(28, 9, 0xde5);
            put(20, 11, 0xde5);
            put(21, 12, 0xde5);
            put(21, 9, 0xde6);
            put(25, 9, 0xde6);
            put(20, 10, 0xde6);
            put(28, 11, 0xde6);
            put(21, 17, 0xde6);
            put(20, 18, 0xde6);
            put(22, 9, 0xde7);
            put(24, 10, 0xde7);
            put(15, 15, 0xde7);
            put(16, 15, 0xde7);
            put(19, 19, 0xde7);
            put(28, 19, 0xde7);
            put(23, 9, 0xde8);
            put(26, 9, 0xde8);
            put(27, 10, 0xde8);
            put(17, 15, 0xde8);
            put(18, 16, 0xde8);
            put(23, 10, 0xde9);
            put(26, 10, 0xde9);
            put(14, 15, 0xde9);
            put(17, 16, 0xde9);
            put(26, 18, 0xde9);
            put(27, 19, 0xde9);
            put(29, 10, 0xdea);
            put(28, 12, 0xdea);
            put(28, 13, 0xdea);
            put(29, 18, 0xdea);
            put(15, 11, 0xdeb);
            put(27, 11, 0xdeb);
            put(27, 12, 0xdeb);
            put(14, 13, 0xdeb);
            put(27, 13, 0xdeb);
            put(14, 14, 0xdeb);
            put(18, 17, 0xdeb);
            put(18, 18, 0xdeb);
            put(18, 12, 0xdec);
            put(17, 13, 0xdec);
            put(19, 12, 0xded);
            put(20, 12, 0xdee);
            put(18, 13, 0xdef);
            put(27, 15, 0xdef);
            put(19, 13, 0xdf0);
            put(19, 14, 0xdf0);
            put(20, 14, 0xdf0);
            put(21, 14, 0xdf0);
            put(21, 15, 0xdf0);
            put(27, 16, 0xdf0);
            put(28, 16, 0xdf0);
            put(20, 13, 0xdf1);
            put(28, 15, 0xdf1);
            put(21, 13, 0xdf2);
            put(17, 14, 0xdf3);
            put(18, 15, 0xdf3);
            put(20, 16, 0xdf3);
            put(18, 14, 0xdf4);
            put(19, 15, 0xdf5);
            put(20, 15, 0xdf6);
            put(27, 17, 0xdf6);
            put(26, 15, 0xdf7);
            put(29, 15, 0xdf8);
            put(21, 16, 0xdf9);
            put(26, 16, 0xdfa);
            put(29, 16, 0xdfb);
            put(26, 17, 0xdfc);
            put(28, 17, 0xdfd);
            put(29, 17, 0xdfe);
            put(27, 18, 0xdff);
            put(28, 18, 0xe00);
            put(21, 10, 0xe01);
            put(25, 10, 0xe01);
            put(21, 18, 0xe01);
            put(29, 11, 0xe02);
            put(20, 19, 0xe02);
            put(29, 19, 0xe02);
            put(18, 19, 0xe03);
            put(27, 14, 0xe04);
            put(28, 14, 0xe05);
        }
        60 | 61 | 62 | 63 | 64 | 65 | 72 | 73 => {
            put(8, 11, 0xe13);
            put(11, 11, 0xe14);
            put(8, 12, 0xe15);
            put(9, 12, 0xe16);
            put(10, 12, 0xe17);
            put(11, 12, 0xe18);
            put(9, 13, 0xe19);
            put(10, 13, 0xe1a);
            put(9, 16, 0xe06);
            put(10, 16, 0xe06);
            put(8, 14, 0xe07);
            put(8, 15, 0xe07);
            put(9, 14, 0xe08);
            put(9, 15, 0xe08);
            put(10, 14, 0xe09);
            put(10, 15, 0xe09);
            put(11, 14, 0xe0a);
            put(11, 15, 0xe0a);
        }
        66 | 67 | 68 | 75 | 76 => {
            put(47, 8, 0xe96);
            put(48, 8, 0xe97);
            put(47, 9, 0xe9c);
            put(47, 10, 0xe9c);
            put(48, 9, 0xe9d);
            put(48, 10, 0xe9d);
            put(47, 11, 0xe9a);
            put(48, 11, 0xe9b);
        }
        69 | 70 | 77 | 78 => {
            put(52, 16, 0x918);
            put(53, 16, 0x919);
            put(52, 17, 0x91a);
            put(53, 17, 0x91b);
        }
        71 => {
            put(15, 19, 0xe78);
            put(16, 19, 0xe79);
            put(17, 19, 0xe7a);
            put(18, 19, 0xe7b);
            put(15, 20, 0xe7c);
            put(16, 20, 0xe7d);
            put(17, 20, 0xe7e);
            put(18, 20, 0xe7f);
            put(15, 21, 0xe80);
            put(16, 21, 0xe81);
            put(17, 21, 0xe82);
            put(18, 21, 0xe83);
            put(15, 22, 0xe84);
            put(16, 22, 0xe85);
            put(17, 22, 0xe86);
            put(18, 22, 0xe87);
        }
        74 | 79 | 80 | 81 | 82 | 83 | 84 | 85 | 86 | 87 | 88 | 89 | 96 | 97 => {
            put(31, 26, 0xe1b);
            put(32, 26, 0xe1c);
            put(31, 27, 0xe1d);
            put(32, 27, 0xe1e);
            put(31, 28, 0xe1f);
            put(32, 28, 0xe20);
        }
        90 | 91 | 92 | 99 | 100 => {
            put(30, 7, 0xe3f);
            put(31, 7, 0xe40);
            put(32, 7, 0xe41);
            put(30, 8, 0xe42);
            put(31, 8, 0xe43);
            put(32, 8, 0xe44);
            put(30, 9, 0xe45);
            put(31, 9, 0xe46);
            put(32, 9, 0xe47);
        }
        93 | 94 | 95 | 102 | 103 => {
            put(51, 3, 0xe31);
            put(53, 4, 0xe2d);
            put(53, 5, 0xe2e);
            put(53, 6, 0xe2f);
        }
        98 => {
            put(16, 26, 0x918);
            put(17, 26, 0x919);
            put(16, 27, 0x91a);
            put(17, 27, 0x91b);
        }
        101 | 104 | 105 | 106 | 108 | 109 | 110 | 111 | 112 | 113 | 120 | 121 => {
            put(17, 10, 0xe64);
            put(18, 10, 0xe65);
            put(19, 10, 0xe66);
            put(20, 10, 0xe67);
            put(17, 11, 0xe68);
            put(18, 11, 0xe69);
            put(19, 11, 0xe6a);
            put(20, 11, 0xe6b);
            put(17, 12, 0xe6c);
            put(18, 12, 0xe6d);
            put(19, 12, 0xe6e);
            put(20, 12, 0xe6f);
            put(17, 13, 0xe70);
            put(18, 13, 0xe71);
            put(19, 13, 0xe72);
            put(20, 13, 0xe73);
            put(17, 14, 0xe74);
            put(18, 14, 0xe75);
            put(19, 14, 0xe76);
            put(20, 14, 0xe77);
        }
        114 | 115 | 116 | 117 | 118 | 122 | 124 | 125 | 126 | 127 => {
            panic!("event overlay is unreachable for this overworld screen");
        }
        _ => {}
    }
}

// Source-shaped entry points retained for one-to-one coverage of overworld.c.
// Several of these were originally folded into direction-parameterized helpers;
// keeping the wrappers makes the correspondence auditable without duplicating
// their behavior.

#[must_use]
pub fn get_map8to_tile_attr(assets: &impl OverworldAssets) -> &[u8] {
    assets.map8_tile_attributes()
}

#[must_use]
pub fn get_map16to_map8_table(assets: &impl OverworldAssets) -> &[u16] {
    assets.map16_to_map8()
}

pub fn build_full_stripe_during_transition_north(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
) -> usize {
    build_full_stripe_during_transition(ram, assets, destination, 8)
}
pub fn build_full_stripe_during_transition_south(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
) -> usize {
    build_full_stripe_during_transition(ram, assets, destination, 4)
}
pub fn build_full_stripe_during_transition_west(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
) -> usize {
    build_full_stripe_during_transition(ram, assets, destination, 2)
}
pub fn build_full_stripe_during_transition_east(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
) -> usize {
    build_full_stripe_during_transition(ram, assets, destination, 1)
}

pub fn check_for_newly_loaded_map_areas_north(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
) -> usize {
    check_for_newly_loaded_map_area(ram, assets, destination, 8)
}
pub fn check_for_newly_loaded_map_areas_south(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
) -> usize {
    check_for_newly_loaded_map_area(ram, assets, destination, 4)
}
pub fn check_for_newly_loaded_map_areas_west(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
) -> usize {
    check_for_newly_loaded_map_area(ram, assets, destination, 2)
}
pub fn check_for_newly_loaded_map_areas_east(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    destination: usize,
) -> usize {
    check_for_newly_loaded_map_area(ram, assets, destination, 1)
}

fn create_initial_screen_view(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    small: bool,
    direction: u8,
) {
    ram.write_u8(OVERWORLD_TRANS_DIR2, direction);
    match (small, direction) {
        (false, 8) => {
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_add(0x380),
            );
            ram.write_u16(MAP16_LOAD_VAR2, 31);
            trigger_and_finish_map_load_stripe_y(ram, assets, 7);
        }
        (false, 4) => {
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                (ram.read_u16(MAP16_LOAD_SRC_OFFSET) & 0x7f).wrapping_add(0x780),
            );
            ram.write_u16(MAP16_LOAD_VAR2, 7);
            trigger_and_finish_map_load_stripe_y(ram, assets, 8);
            ram.write_u16(
                MAP16_LOAD_VAR2,
                ram.read_u16(MAP16_LOAD_VAR2).wrapping_add(9) & 0x1f,
            );
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0xb80),
            );
        }
        (false, 2) => {
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_add(14),
            );
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 31);
            trigger_and_finish_map_load_stripe_x(ram, assets, 7);
        }
        (false, 1) => {
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET)
                    .wrapping_sub(0x60)
                    .wrapping_add(0x1e),
            );
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 7);
            trigger_and_finish_map_load_stripe_x(ram, assets, 8);
            ram.write_u16(
                MAP16_LOAD_DST_OFFSET,
                ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_add(9) & 0x1f,
            );
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x2e),
            );
        }
        (true, 8) => {
            ram.write_u16(
                ORANGE_BLUE_BARRIER_STATE,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x700),
            );
            ram.write_u16(WORD_C174, ram.read_u16(MAP16_LOAD_DST_OFFSET));
            ram.write_u16(WORD_C176, 10);
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x1390);
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 0);
            ram.write_u16(MAP16_LOAD_VAR2, 31);
            trigger_and_finish_map_load_stripe_y(ram, assets, 7);
        }
        (true, 4) => {
            ram.write_u16(
                ORANGE_BLUE_BARRIER_STATE,
                u16::from(ram.read_u8(MAP16_LOAD_SRC_OFFSET)),
            );
            ram.write_u16(WORD_C174, ram.read_u16(MAP16_LOAD_DST_OFFSET));
            ram.write_u16(WORD_C176, 24);
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x790);
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 0);
            ram.write_u16(MAP16_LOAD_VAR2, 7);
            trigger_and_finish_map_load_stripe_y(ram, assets, 8);
            ram.write_u16(
                MAP16_LOAD_VAR2,
                ram.read_u16(MAP16_LOAD_VAR2).wrapping_add(9) & 0x1f,
            );
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0xb80),
            );
        }
        (true, 2) => {
            ram.write_u16(
                ORANGE_BLUE_BARRIER_STATE,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x20),
            );
            ram.write_u16(WORD_C174, 8);
            ram.write_u16(WORD_C176, ram.read_u16(MAP16_LOAD_VAR2));
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x44e);
            ram.write_u16(MAP16_LOAD_VAR2, 0);
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 31);
            trigger_and_finish_map_load_stripe_x(ram, assets, 7);
        }
        (true, 1) => {
            ram.write_u16(
                ORANGE_BLUE_BARRIER_STATE,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x60),
            );
            ram.write_u16(WORD_C174, 0x18);
            ram.write_u16(WORD_C176, ram.read_u16(MAP16_LOAD_VAR2));
            ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x41e);
            ram.write_u16(MAP16_LOAD_VAR2, 0);
            ram.write_u16(MAP16_LOAD_DST_OFFSET, 7);
            trigger_and_finish_map_load_stripe_x(ram, assets, 8);
            ram.write_u16(
                MAP16_LOAD_DST_OFFSET,
                ram.read_u16(MAP16_LOAD_DST_OFFSET).wrapping_add(9) & 0x1f,
            );
            ram.write_u16(
                MAP16_LOAD_SRC_OFFSET,
                ram.read_u16(MAP16_LOAD_SRC_OFFSET).wrapping_sub(0x2e),
            );
        }
        _ => unreachable!("the C callers pass one of the four direction bits"),
    }
}

pub fn create_initial_ow_screen_view_big_north(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    create_initial_screen_view(ram, assets, false, 8);
}
pub fn create_initial_ow_screen_view_big_south(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    create_initial_screen_view(ram, assets, false, 4);
}
pub fn create_initial_ow_screen_view_big_west(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    create_initial_screen_view(ram, assets, false, 2);
}
pub fn create_initial_ow_screen_view_big_east(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    create_initial_screen_view(ram, assets, false, 1);
}
pub fn create_initial_ow_screen_view_small_north(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    create_initial_screen_view(ram, assets, true, 8);
}
pub fn create_initial_ow_screen_view_small_south(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    create_initial_screen_view(ram, assets, true, 4);
}
pub fn create_initial_ow_screen_view_small_west(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    create_initial_screen_view(ram, assets, true, 2);
}
pub fn create_initial_ow_screen_view_small_east(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    create_initial_screen_view(ram, assets, true, 1);
}

pub fn trigger_and_finish_map_load_stripe_x(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    count: u8,
) {
    trigger_and_finish_map_load_stripe(ram, assets, MapStripeAxis::X, count);
}
pub fn trigger_and_finish_map_load_stripe_y(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    count: u8,
) {
    trigger_and_finish_map_load_stripe(ram, assets, MapStripeAxis::Y, count);
}

pub fn ganon_tower_entrance_func1(ram: &mut Ram<'_>, filter: impl FnMut(&mut Ram<'_>)) {
    ganon_tower_entrance_flash(ram, filter);
}
pub fn mirror_bonk_recover_changed_tiles(ram: &mut Ram<'_>) {
    recover_mirror_bonk_changed_tiles(ram);
}
pub fn sprite_load_graphics_properties(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    load_sprite_graphics_properties(ram, assets);
}
pub fn sprite_load_graphics_properties_light_world_only(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
) {
    load_sprite_graphics_properties_light_world_only(ram, assets);
}
pub fn overworld_finish_trans_gfx(ram: &mut Ram<'_>) {
    finish_transition_gfx(ram);
}
pub fn overworld_set_fixed_col_and_scroll(ram: &mut Ram<'_>) {
    set_fixed_color_and_scroll(ram);
}
pub fn overworld_load_bird_travel_pos(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    index: u8,
    load_gfx: impl FnMut(&mut Ram<'_>),
    reset_sprites: impl FnMut(&mut Ram<'_>),
    reload_sprites: impl FnMut(&mut Ram<'_>),
    reset_torch: impl FnMut(&mut Ram<'_>),
) {
    load_bird_travel_position(
        ram,
        assets,
        usize::from(index),
        load_gfx,
        reset_sprites,
        reload_sprites,
        reset_torch,
    );
}

pub fn overworld_do_map_update32x32(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    update_map_32x32(ram, assets);
}
pub fn overworld_do_map_update32x32_b(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    update_map_32x32_b(ram, |ram| update_map_32x32(ram, assets));
}
pub fn overworld_do_map_update32x32_conditional(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    update_map_32x32_conditional(ram, |ram| update_map_32x32(ram, assets));
}

pub fn overworld_lifting_small_obj(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    a: u16,
    position: u16,
    y: u16,
    point: Point16U,
) -> u8 {
    lift_small_overworld_object(ram, assets, a, position, y, point)
}

pub fn overworld_animate_entrance_po_d(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    animate_entrance_palace_of_darkness(ram, assets);
}
pub fn overworld_animate_entrance_skull(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    animate_entrance_skull_woods(ram, assets);
}
pub fn overworld_animate_entrance_mire(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    animate_entrance_misery_mire(ram, assets);
}
pub fn overworld_entrance_draw_many_tr(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    entrance_draw_many_turtle_rock(ram, assets);
}

pub fn module08_02_load_and_advance(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    module08_load_and_advance(ram, assets, |ram| {
        draw_quadrants_and_overlays(ram, assets, overworld_load_event_overlay);
    });
}

pub fn module09_00_player_control(ram: &mut Ram<'_>, hooks: &mut impl PlayerControlHooks) {
    module09_player_control(ram, hooks);
}
pub fn module09_09_open_big_door_from_exiting(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    open_big_door_from_exit(ram, |ram| {
        update_map_32x32_conditional(ram, |ram| update_map_32x32(ram, assets));
    });
}
pub fn module09_0a_walk_from_exiting_facing_down(
    ram: &mut Ram<'_>,
    assets: &impl OverworldAssets,
    animate: impl FnMut(&mut Ram<'_>),
) {
    walk_from_exit_facing_down(ram, assets, animate);
}
pub fn module09_0b_walk_from_exiting_facing_up(
    ram: &mut Ram<'_>,
    animate: impl FnMut(&mut Ram<'_>),
) {
    walk_from_exit_facing_up(ram, animate);
}
pub fn module09_2a_recover_from_drowning(
    ram: &mut Ram<'_>,
    scroll: impl FnMut(&mut Ram<'_>),
    recover: impl FnMut(&mut Ram<'_>),
) {
    module09_recover_from_drowning(ram, scroll, recover);
}
pub fn module09_2e_whirlpool(
    ram: &mut Ram<'_>,
    blue: impl FnMut(&mut Ram<'_>),
    isolate: impl FnMut(&mut Ram<'_>),
) {
    module09_whirlpool(ram, blue, isolate);
}
pub fn module10_00_open_iris(ram: &mut Ram<'_>, spotlight: impl FnMut(&mut Ram<'_>)) {
    module10_open_iris(ram, spotlight);
}
pub fn open_spotlight_next2(
    ram: &mut Ram<'_>,
    blank: impl FnMut(&mut Ram<'_>),
    reset: impl FnMut(&mut Ram<'_>),
) {
    open_spotlight_next(ram, blank, reset);
}
pub fn overworld_mosaic_transition_load_sprite_graphics_and_set_mosaic(
    ram: &mut Ram<'_>,
    load: impl FnMut(&mut Ram<'_>),
) {
    mosaic_transition_load_sprite_graphics(ram, load);
}
pub fn overworld_overlay_handle_rain(ram: &mut Ram<'_>) {
    handle_rain_overlay(ram);
}
pub fn overworld_load_overlays2(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    load_overlays_inner(ram, |ram| load_overworld_overlay(ram, assets));
}
pub fn scroll_and_check_for_sow_exit(ram: &mut Ram<'_>, assets: &impl OverworldAssets) {
    scroll_and_check_for_special_overworld_exit(ram, assets);
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    struct Assets {
        areas: [u16; 129],
        positions: [u16; 129],
        entrance_ids: [u8; 129],
        signs: [u16; 2],
        backgrounds: [u8; 2],
        graphics: [u8; 256],
        palettes: [u8; 256],
        sprite_offsets: Vec<u16>,
        sprites: Vec<u8>,
        map16: Vec<u16>,
        attributes: Vec<u8>,
        enemy_damage: Vec<u8>,
        music_sets: Vec<u8>,
        music_sets2: Vec<u8>,
        fall_areas: [u16; 19],
        fall_positions: [u16; 19],
        fall_entrances: [u8; 19],
        whirlpools: Vec<u16>,
        map32: [Vec<u8>; 4],
        hibytes: Vec<Vec<u8>>,
        lobytes: Vec<Vec<u8>>,
        map_is_small: [u8; 256],
        secrets: Vec<u8>,
        secret_offsets: [u16; 128],
        aux_themes: [u8; 256],
        special_top: [u16; 128],
        special_bottom: [u16; 128],
        special_left: [u16; 128],
        special_right: [u16; 128],
        special_targets: [[i16; 128]; 4],
        special_left_edge: [u16; 64],
        special_direction: [u8; 128],
        special_sprite_gfx: [u8; 128],
        special_aux_gfx: [u8; 128],
        special_bg_palette: [u8; 128],
        special_sprite_palette: [u8; 128],
    }

    impl OverworldAssets for Assets {
        fn entrance_areas(&self) -> &[u16] {
            &self.areas
        }
        fn entrance_positions(&self) -> &[u16] {
            &self.positions
        }
        fn entrance_ids(&self) -> &[u8] {
            &self.entrance_ids
        }
        fn sign_text(&self) -> &[u16] {
            &self.signs
        }
        fn background_palettes(&self) -> &[u8] {
            &self.backgrounds
        }
        fn sprite_graphics(&self) -> &[u8] {
            &self.graphics
        }
        fn sprite_palettes(&self) -> &[u8] {
            &self.palettes
        }
        fn sprite_offsets(&self) -> &[u16] {
            &self.sprite_offsets
        }
        fn sprites(&self) -> &[u8] {
            &self.sprites
        }
        fn map16_to_map8(&self) -> &[u16] {
            &self.map16
        }
        fn map8_tile_attributes(&self) -> &[u8] {
            &self.attributes
        }
        fn enemy_damage_data(&self) -> &[u8] {
            &self.enemy_damage
        }
        fn overworld_music_sets(&self) -> &[u8] {
            &self.music_sets
        }
        fn overworld_music_sets2(&self) -> &[u8] {
            &self.music_sets2
        }
        fn fall_hole_areas(&self) -> &[u16] {
            &self.fall_areas
        }
        fn fall_hole_positions(&self) -> &[u16] {
            &self.fall_positions
        }
        fn fall_hole_entrances(&self) -> &[u8] {
            &self.fall_entrances
        }
        fn whirlpool_areas(&self) -> &[u16] {
            &self.whirlpools
        }
        fn map32_to_map16(&self) -> [&[u8]; 4] {
            [
                &self.map32[0],
                &self.map32[1],
                &self.map32[2],
                &self.map32[3],
            ]
        }
        fn overworld_hibytes(&self, screen: usize) -> &[u8] {
            &self.hibytes[screen]
        }
        fn overworld_lobytes(&self, screen: usize) -> &[u8] {
            &self.lobytes[screen]
        }
        fn overworld_map_is_small(&self) -> &[u8] {
            &self.map_is_small
        }
        fn overworld_secrets(&self) -> &[u8] {
            &self.secrets
        }
        fn overworld_secret_offsets(&self) -> &[u16] {
            &self.secret_offsets
        }
        fn overworld_aux_tile_themes(&self) -> &[u8] {
            &self.aux_themes
        }
        fn special_exit_top(&self) -> &[u16] {
            &self.special_top
        }
        fn special_exit_bottom(&self) -> &[u16] {
            &self.special_bottom
        }
        fn special_exit_left(&self) -> &[u16] {
            &self.special_left
        }
        fn special_exit_right(&self) -> &[u16] {
            &self.special_right
        }
        fn special_exit_up_target(&self) -> &[i16] {
            &self.special_targets[0]
        }
        fn special_exit_down_target(&self) -> &[i16] {
            &self.special_targets[1]
        }
        fn special_exit_left_target(&self) -> &[i16] {
            &self.special_targets[2]
        }
        fn special_exit_right_target(&self) -> &[i16] {
            &self.special_targets[3]
        }
        fn special_exit_left_edge(&self) -> &[u16] {
            &self.special_left_edge
        }
        fn special_exit_direction(&self) -> &[u8] {
            &self.special_direction
        }
        fn special_exit_sprite_gfx(&self) -> &[u8] {
            &self.special_sprite_gfx
        }
        fn special_exit_aux_gfx(&self) -> &[u8] {
            &self.special_aux_gfx
        }
        fn special_exit_bg_palette(&self) -> &[u8] {
            &self.special_bg_palette
        }
        fn special_exit_sprite_palette(&self) -> &[u8] {
            &self.special_sprite_palette
        }
    }

    #[derive(Default)]
    struct RecordingPreHooks {
        calls: Vec<&'static str>,
        animated_tiles: u8,
        palette: (u8, u8),
    }

    impl PreOverworldHooks for RecordingPreHooks {
        fn load_from_dungeon(&mut self, _: &mut Ram<'_>) {
            self.calls.push("dungeon");
        }
        fn load_from_special_overworld(&mut self, _: &mut Ram<'_>) {
            self.calls.push("special");
        }
        fn hud_refill_logic(&mut self, _: &mut Ram<'_>) {
            self.calls.push("hud");
        }
        fn decompress_animated_tiles(&mut self, _: &mut Ram<'_>, tiles: u8) {
            self.animated_tiles = tiles;
            self.calls.push("decompress");
        }
        fn initialize_tilesets(&mut self, _: &mut Ram<'_>) {
            self.calls.push("tilesets");
        }
        fn load_screen_palette_set(&mut self, _: &mut Ram<'_>) {
            self.calls.push("palette_set");
        }
        fn load_palettes(&mut self, _: &mut Ram<'_>, background: u8, sprites: u8) {
            self.palette = (background, sprites);
            self.calls.push("palettes");
        }
        fn set_overworld_background_color(&mut self, _: &mut Ram<'_>) {
            self.calls.push("background");
        }
        fn load_palettes_inner(&mut self, _: &mut Ram<'_>) {
            self.calls.push("palettes_inner");
        }
        fn copy_special_palettes_to_cache(&mut self, _: &mut Ram<'_>) {
            self.calls.push("palette_cache");
        }
        fn initialize_follower(&mut self, _: &mut Ram<'_>) {
            self.calls.push("follower");
        }
        fn decode_animated_sprite_tile(&mut self, _: &mut Ram<'_>, tile: u8) {
            assert_eq!(tile, 0x1e);
            self.calls.push("decode");
        }
        fn reload_overworld_sprites(&mut self, _: &mut Ram<'_>) {
            self.calls.push("sprites");
        }
        fn initialize_mirror_portal(&mut self, _: &mut Ram<'_>) {
            self.calls.push("mirror");
        }
        fn reset_torch_background_and_player(&mut self, _: &mut Ram<'_>) {
            self.calls.push("torch");
        }
        fn load_bunny_palettes(&mut self, _: &mut Ram<'_>) {
            self.calls.push("bunny");
        }
        fn load_overworld_music_if_needed(&mut self, _: &mut Ram<'_>) {
            self.calls.push("music");
        }
    }

    #[derive(Default)]
    struct RecordingPlayerHooks(Vec<&'static str>);

    impl PlayerControlHooks for RecordingPlayerHooks {
        fn did_press_map(&mut self, _: &mut Ram<'_>) -> bool {
            false
        }
        fn display_select_menu(&mut self, _: &mut Ram<'_>) {
            self.0.push("menu");
        }
        fn handle_item_switch_inputs(&mut self, _: &mut Ram<'_>) {
            self.0.push("item");
        }
        fn animate_entrance(&mut self, _: &mut Ram<'_>) {
            self.0.push("animate");
        }
        fn link_main(&mut self, _: &mut Ram<'_>) {
            self.0.push("link");
        }
        fn super_bomb_indicator(&mut self, _: &mut Ram<'_>) {
            self.0.push("bomb");
        }
        fn load_chr_half_slot(&mut self, _: &mut Ram<'_>) {
            self.0.push("chr");
        }
        fn operate_camera_scroll(&mut self, _: &mut Ram<'_>) {
            self.0.push("camera");
        }
        fn use_entrance(&mut self, _: &mut Ram<'_>) {
            self.0.push("entrance");
        }
        fn animate_dark_world_palette(&mut self, _: &mut Ram<'_>) {
            self.0.push("palette");
        }
        fn handle_transitions(&mut self, _: &mut Ram<'_>) {
            self.0.push("transitions");
        }
        fn scroll_and_check_special_exit(&mut self, _: &mut Ram<'_>) {
            self.0.push("special");
        }
    }

    #[derive(Default)]
    struct RecordingTransitionHooks {
        reset: bool,
    }

    impl TransitionHooks for RecordingTransitionHooks {
        fn check_edge_screen_transition(&mut self, _: &mut Ram<'_>) -> bool {
            false
        }
        fn reset_torch_background_and_player(&mut self, _: &mut Ram<'_>) {
            self.reset = true;
        }
        fn is_playing_music_track(&mut self, _: u8) -> bool {
            false
        }
        fn load_palettes(&mut self, _: &mut Ram<'_>, _: u8, _: u8) {
            panic!("mosaic path does not load palettes")
        }
        fn copy_palettes_to_cache(&mut self, _: &mut Ram<'_>) {
            panic!("mosaic path does not cache palettes")
        }
    }

    fn assets() -> Assets {
        Assets {
            areas: [0; 129],
            positions: [0; 129],
            entrance_ids: [0; 129],
            signs: [0x1234, 0xabcd],
            backgrounds: [7, 9],
            graphics: core::array::from_fn(|i| i as u8),
            palettes: core::array::from_fn(|i| (255 - i) as u8),
            sprite_offsets: vec![0; 432],
            sprites: (0..=255).collect(),
            map16: vec![0; 16],
            attributes: vec![0; 512],
            enemy_damage: vec![0; 0x800],
            music_sets: (0..256).map(|i| i as u8).collect(),
            music_sets2: (0..96).map(|i| 255 - i as u8).collect(),
            fall_areas: [0; 19],
            fall_positions: [0; 19],
            fall_entrances: [0; 19],
            whirlpools: Vec::new(),
            map32: core::array::from_fn(|_| Vec::new()),
            hibytes: Vec::new(),
            lobytes: Vec::new(),
            map_is_small: [0; 256],
            secrets: vec![0xff, 0xff],
            secret_offsets: [0; 128],
            aux_themes: [0; 256],
            special_top: [0; 128],
            special_bottom: [0; 128],
            special_left: [0; 128],
            special_right: [0; 128],
            special_targets: [[0; 128]; 4],
            special_left_edge: [0; 64],
            special_direction: [0; 128],
            special_sprite_gfx: [0; 128],
            special_aux_gfx: [0; 128],
            special_bg_palette: [0; 128],
            special_sprite_palette: [0; 128],
        }
    }

    #[test]
    fn fixed_entrance_pairs_match_as_pairs_only() {
        assert!(lookup_in_ow_entrance_tab(0xfe, 0x14a));
        assert!(lookup_in_ow_entrance_tab(0x177, 0x167));
        assert!(!lookup_in_ow_entrance_tab(0xfe, 0x167));
    }

    #[test]
    fn dynamic_entrance_search_prefers_highest_index() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_AREA_INDEX, 3);
        let mut data = assets();
        data.areas[4] = 3;
        data.positions[4] = 0xbeef;
        data.areas[128] = 3;
        data.positions[128] = 0xbeef;
        assert_eq!(lookup_in_ow_entrance_tab2(&ram, &data, 0xbeef), Some(128));
    }

    #[test]
    fn special_area_caches_current_world_and_loads_signed_destination_tables() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut data = assets();
        let index = 3usize;
        data.special_direction[index] = 6;
        data.special_sprite_gfx[index] = 0x45;
        data.special_aux_gfx[index] = 0x67;
        data.special_bg_palette[index] = 0x12;
        data.special_sprite_palette[index] = 0x34;
        data.special_top[index] = 0x1111;
        data.special_bottom[index] = 0x2222;
        data.special_left[index] = 0x3333;
        data.special_right[index] = 0x4444;
        data.special_targets[0][index] = -1;
        data.special_targets[1][index] = -2;
        data.special_targets[2][index] = 0x1234;
        data.special_targets[3][index] = -0x1234;
        data.special_left_edge[index] = 0x2a8;

        for (word, value) in [
            (OVERWORLD_AREA_INDEX, 0x10),
            (TM_COPY, 0x1514),
            (BG2VOFS_COPY2, 0x2020),
            (BG2HOFS_COPY2, 0x3030),
            (LINK_Y, 0x4040),
            (LINK_X, 0x5050),
            (OVERWORLD_SCREEN_INDEX, 0x60),
            (MAP16_LOAD_SRC_OFFSET, 0x7070),
            (CAMERA_SCROLL_LOW, 0x8080),
            (CAMERA_SCROLL_LOW + 4, 0x9090),
        ] {
            ram.write_u16(word, value);
        }
        for offset in (0..8).step_by(2) {
            ram.write_u16(OW_SCROLL_VARS0 + offset, 0xa000 + offset as u16);
            ram.write_u16(UP_DOWN_SCROLL_TARGET + offset, 0xb000 + offset as u16);
            ram.write_u16(OVERWORLD_UNK1 + offset, 0xc000 + offset as u16);
        }
        ram.write_u8(BYTE_0AA0, 0xd0);
        ram.write_u8(MAIN_TILE_THEME_INDEX, 0xd1);
        ram.write_u8(AUX_TILE_THEME_INDEX, 0xd2);
        ram.write_u8(SPRITE_GRAPHICS_INDEX, 0xd3);

        let mut loaded = false;
        let mut palette = None;
        let mut special_palette = false;
        enter_special_area(
            &mut ram,
            &data,
            |ram| {
                loaded = true;
                ram.write_u16(DUNGEON_ROOM, 0x83);
            },
            |_, bg, sprite| palette = Some((bg, sprite)),
            |_| special_palette = true,
        );

        assert!(loaded && special_palette);
        assert_eq!(palette, Some((0x12, 0x34)));
        assert_eq!(ram.read_u16(SPECIAL_EXIT_CACHE), 0x10);
        assert_eq!(ram.read_u16(SPECIAL_EXIT_CACHE + 8), 0x4040);
        assert_eq!(ram.read_u16(SPECIAL_EXIT_CACHE + 10), 0x5050);
        assert_eq!(ram.read_u16(SPECIAL_EXIT_CACHE + 42), 0xc000);
        assert_eq!(ram.read_u8(DUNGEON_ROOM), 0x83);
        assert_eq!(ram.read_u8(LINK_FACING), 6);
        assert_eq!(ram.read_u8(SPRITE_GRAPHICS_INDEX), 0x45);
        assert_eq!(ram.read_u8(AUX_TILE_THEME_INDEX), 0x67);
        assert_eq!(ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR), 0x1111);
        assert_eq!(ram.read_u16(OVERWORLD_OFFSET_BASE_X_ADDR), 0x55);
        assert_eq!(ram.read_u16(UP_DOWN_SCROLL_TARGET), 0xffff);
        assert_eq!(ram.read_u16(UP_DOWN_SCROLL_TARGET_END), 0xfffe);
        assert_eq!(ram.read_u16(LEFT_RIGHT_SCROLL_TARGET), 0x1234);
        assert_eq!(ram.read_u16(LEFT_RIGHT_SCROLL_TARGET_END), 0xedcc);
    }

    #[test]
    fn use_entrance_selects_highest_matching_dynamic_entrance() {
        let mut data = assets();
        data.map16.resize(4, 0);
        data.map16[2] = 0xfe;
        data.map16[3] = 0x14a;
        data.areas[128] = 0;
        data.positions[128] = 0;
        data.entrance_ids[128] = 9;
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(LINK_FACING, 2);
        use_entrance(&mut ram, &data, |_| panic!("unexpected message"));
        assert_eq!(ram.read_u8(WHICH_ENTRANCE), 9);
        assert_eq!(ram.read_u8(MAIN_MODULE), 15);
        assert_eq!(ram.read_u8(SAVED_MODULE_FOR_MENU), 6);
        assert_eq!(ram.read_u8(SUBMODULE), 0);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 0);
    }

    #[test]
    fn tagalong_gate_and_direction_encoding_match_c() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for (direction, expected) in [(1, 3), (2, 2), (4, 1), (8, 0)] {
            assert_eq!(dir_to_enum(direction), expected);
        }
        ram.write_u8(FOLLOWER_INDICATOR, 7);
        assert!(!can_enter_with_tagalong(&ram, 58));
        assert!(can_enter_with_tagalong(&ram, 59));
        ram.write_u8(FOLLOWER_INDICATOR, 5);
        assert!(can_enter_with_tagalong(&ram, 0));
    }

    #[test]
    fn mosaic_and_graphics_property_helpers_preserve_wrapping_semantics() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(MOSAIC_LEVEL, 0);
        ram.write_u16(PALETTE_FILTER_COUNTDOWN, 1);
        reset_mosaic_down(&mut ram);
        assert_eq!(ram.read_u8(MOSAIC_LEVEL), 0xf0);
        assert_eq!(ram.read_u8(BGMODE), 9);
        assert_eq!(ram.read_u8(MOSAIC), 0xf7);

        ram.write_u8(SRAM_PROGRESS_INDICATOR, 3);
        let data = assets();
        load_sprite_graphics_properties(&mut ram, &data);
        assert_eq!(
            ram.slice(OVERWORLD_SPRITE_GFX, 128),
            &data.graphics[128..256]
        );
        assert_eq!(
            ram.slice(OVERWORLD_SPRITE_PALETTES, 128),
            &data.palettes[128..256]
        );
        assert_eq!(get_sign_text(&data, 1), 0xabcd);
        assert_eq!(get_background_palette(&data, 1), 9);
    }

    #[test]
    fn pre_overworld_load_preserves_music_goto_and_bunny_contract() {
        let data = assets();
        // The production table has 160 entries; use a dedicated asset below
        // by exercising screen zero while selecting the dark-world path via
        // the save flag.
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(MAIN_MODULE, 8);
        ram.write_u8(OVERWORLD_SCREEN_INDEX, 0);
        ram.write_u8(SAVEGAME_IS_DARKWORLD, 1);
        ram.write_u8(LINK_ITEM_MOON_PEARL, 0);
        ram.write_u8(OVERWORLD_SPRITE_PALETTES, 12);
        ram.write_u8(FOLLOWER_INDICATOR, 6);
        let mut hooks = RecordingPreHooks::default();
        pre_overworld_load_properties(&mut ram, &data, &mut hooks);

        assert_eq!(ram.read_u8(QUEUED_MUSIC_CONTROL), 4);
        assert_eq!(hooks.animated_tiles, 0x5a);
        assert_eq!(hooks.palette, (7, 12));
        assert_eq!(ram.read_u8(LINK_IS_BUNNY), 1);
        assert_eq!(ram.read_u8(LINK_IS_BUNNY_MIRROR), 1);
        assert_eq!(ram.read_u8(LINK_PLAYER_STATE), 23);
        assert_eq!(ram.read_u8(FOLLOWER_INDICATOR), 0);
        assert_eq!(ram.read_u8(LINK_NUM_KEYS), 0xff);
        assert_eq!(ram.read_u8(SUBMODULE), 1);
        assert_eq!(ram.read_u8(FLAG_UPDATE_HUD), 1);
        assert!(hooks.calls.contains(&"dungeon"));
        assert!(hooks.calls.contains(&"palettes_inner"));
        assert!(hooks.calls.contains(&"mirror"));
        assert!(hooks.calls.contains(&"bunny"));
        assert_eq!(hooks.calls.last(), Some(&"music"));
    }

    #[test]
    fn player_control_preserves_callback_order_and_current_area_encoding() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(MAIN_MODULE, 9);
        ram.write_u16(LINK_X, 0x1400);
        ram.write_u16(LINK_Y, 0x0a00);
        let mut hooks = RecordingPlayerHooks::default();
        module09_player_control(&mut ram, &mut hooks);
        assert_eq!(ram.read_u16(CURRENT_AREA_OF_PLAYER), 0x54);
        assert_eq!(
            hooks.0,
            [
                "item",
                "link",
                "bomb",
                "chr",
                "camera",
                "entrance",
                "palette",
                "transitions"
            ]
        );
    }

    #[test]
    fn edge_transition_uses_exact_source_table_and_enters_mosaic_for_area_head_zero() {
        let data = assets();
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(OVERWORLD_SCREEN_INDEX, 1);
        ram.write_u8(LINK_X_VELOCITY, 1);
        ram.write_u8(LINK_DIRECTION, 1);
        ram.write_u16(LINK_X, 10);
        ram.write_u16(CURRENT_AREA_OF_PLAYER, 0);
        ram.write_u16(OVERWORLD_RIGHT_BOTTOM_BOUND, 0);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0xffff);
        let mut hooks = RecordingTransitionHooks::default();
        handle_transitions(&mut ram, &data, &mut hooks);
        assert!(hooks.reset);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0xfe0);
        assert_eq!(ram.read_u8(OVERWORLD_SCREEN_INDEX), 0);
        assert_eq!(ram.read_u8(OVERWORLD_AREA_INDEX), 0);
        assert_eq!(ram.read_u8(OVERWORLD_TRANS_DIR), 1);
        assert_eq!(ram.read_u8(OVERWORLD_TRANS_DIR2), 1);
        assert_eq!(ram.read_u8(BYTE_069C), 3);
        assert_eq!(ram.read_u8(SUBMODULE), 13);
        assert_eq!(ram.read_u8(MOSAIC), 0);
        assert_eq!(ram.read_u8(MOSAIC_LEVEL), 0);
    }

    #[test]
    fn mosaic_transition_preserves_terminal_decode_and_special_reload_order() {
        use core::cell::RefCell;

        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(SUBSUBMODULE, 2);
        ram.write_u8(SUBMODULE, 36);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 0x40);
        let calls = RefCell::new(Vec::new());
        start_mosaic_transition(
            &mut ram,
            |_| calls.borrow_mut().push("mosaic"),
            |_| false,
            |_| calls.borrow_mut().push("reset"),
            |_| calls.borrow_mut().push("palette"),
            |_, value| {
                assert_eq!(value, 0x1e);
                calls.borrow_mut().push("decode");
            },
            |state| {
                calls.borrow_mut().push("load");
                state.write_u16(OVERWORLD_SCREEN_INDEX, 0x80);
            },
        );
        assert_eq!(calls.into_inner(), ["mosaic", "decode", "load", "decode"]);
        assert_eq!(ram.read_u8(INIDISP_COPY), 0x80);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 0);
        assert_eq!(ram.read_u8(SUBMODULE), 37);
    }

    #[test]
    fn rain_overlay_preserves_flash_frames_and_four_step_scroll_pattern() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(OVERWORLD_SCREEN_INDEX, 0x70);
        ram.write_u8(FRAME_COUNTER, 36);
        ram.write_u8(MOVE_OVERLAY_COUNTER, 3);
        ram.write_u16(BG1HOFS_COPY2, 0xff80);
        ram.write_u16(BG1VOFS_COPY2, 0x1234);
        handle_rain_overlay(&mut ram);
        assert_eq!(ram.read_u8(SOUND_EFFECT_1), 54);
        assert_eq!(ram.read_u8(CGADSUB_COPY), 0x32);
        assert_eq!(ram.read_u8(MOVE_OVERLAY_COUNTER), 0);
        assert_eq!(ram.read_u16(BG1HOFS_COPY2), 0x0080);
        assert_eq!(ram.read_u16(BG1VOFS_COPY2), 0x1234);

        ram.write_u8(SAVE_OW_EVENT_INFO + 0x70, 0x20);
        ram.write_u8(FRAME_COUNTER, 44);
        handle_rain_overlay(&mut ram);
        assert_eq!(ram.read_u8(MOVE_OVERLAY_COUNTER), 0);
        assert_eq!(ram.read_u8(CGADSUB_COPY), 0x32);
    }

    #[test]
    fn bank02_decompresses_every_command_shape_and_overlapping_copy() {
        let source = [
            0x02, 1, 2, 3, // literal, length 3
            0x21, 9, // fill, length 2
            0x43, 4, 5, // alternating, length 4
            0x62, 6, // incrementing, length 3
            0x84, 0, 0, // copy first five output bytes
            0x83, 0, 12, // overlapping copy from the preceding sequence
            0xff,
        ];
        let mut destination = [0u8; 32];
        let length = decompress_bank02(&mut destination, &source);
        assert_eq!(length, 21);
        assert_eq!(
            &destination[..length],
            &[
                1, 2, 3, 9, 9, 4, 5, 4, 5, 6, 7, 8, 1, 2, 3, 9, 9, 1, 2, 3, 9
            ]
        );

        // Extended command: command 0 becomes a 257-byte literal run.
        let mut extended = vec![0xe1, 0];
        extended.extend(0..=255);
        extended.push(0);
        extended.push(0xff);
        let mut output = [0u8; 257];
        assert_eq!(decompress_bank02(&mut output, &extended), 257);
        assert_eq!(&output[..4], &[0, 1, 2, 3]);
        assert_eq!(output[256], 0);
    }

    #[test]
    fn map32_definition_expands_packed_high_nibbles_and_caches_group() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(MAP16_DECODE_LAST, 0xffff);
        let tables = [
            [0x10, 0x11, 0x12, 0x13, 0x45, 0x67],
            [0x20, 0x21, 0x22, 0x23, 0x89, 0xab],
            [0x30, 0x31, 0x32, 0x33, 0xcd, 0xef],
            [0x40, 0x41, 0x42, 0x43, 0x12, 0x34],
        ];
        parse_map32_definition(
            &mut ram,
            [&tables[0], &tables[1], &tables[2], &tables[3]],
            0x2000,
            4,
        );
        assert_eq!(ram.read_u16(0x2000), 0x0612);
        assert_eq!(ram.read_u16(0x2002), 0x0a22);
        assert_eq!(ram.read_u16(0x2080), 0x0e32);
        assert_eq!(ram.read_u16(0x2082), 0x0342);
        assert_eq!(ram.read_u16(MAP16_DECODE_TMP), 0);

        // Same eight-entry group uses the cached bytes instead of rereading assets.
        let replacement = [[0xff; 6]; 4];
        parse_map32_definition(
            &mut ram,
            [
                &replacement[0],
                &replacement[1],
                &replacement[2],
                &replacement[3],
            ],
            0x2100,
            6,
        );
        assert_eq!(ram.read_u16(0x2100), 0x0713);
        assert_eq!(ram.read_u16(0x2102), 0x0b23);
    }

    #[test]
    fn tile_attribute_uses_wrapping_masked_overworld_coordinates() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_OFFSET_BASE_X_ADDR, 0xfff8);
        ram.write_u16(OVERWORLD_OFFSET_BASE_Y_ADDR, 0x0100);
        ram.write_u16(OVERWORLD_OFFSET_MASK_X, 0x003f);
        ram.write_u16(OVERWORLD_OFFSET_MASK_Y, 0x003f);
        // (4 - 0xfff8) wraps to 12; (0x103 - 0x100) is 3. Combined byte
        // offset is 12 | (3 << 3) = 28.
        ram.write_u16(DUNG_BG2 + 28, 5);
        assert_eq!(
            read_tile_attribute(&ram, &[0, 1, 2, 3, 4, 0xa7], 4, 0x103),
            0xa7
        );
    }

    #[test]
    fn link_map16_coordinates_keep_the_c_ports_unusual_x_scaling() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(LINK_X, 0x128);
        ram.write_u16(LINK_Y, 0x234);
        ram.write_u8(LINK_FACING, 4); // left: (-3, 12)
        ram.write_u16(OVERWORLD_OFFSET_BASE_X_ADDR, 0x20);
        ram.write_u16(OVERWORLD_OFFSET_BASE_Y_ADDR, 0x200);
        ram.write_u16(OVERWORLD_OFFSET_MASK_X, 0x3f);
        ram.write_u16(OVERWORLD_OFFSET_MASK_Y, 0x7f);
        let (position, point) = get_link_map16_coords(&ram);
        assert_eq!(point, Point16U { x: 0x120, y: 0x240 });
        assert_eq!(position, 0x204);
    }

    #[test]
    fn map16_under_link_uses_eight_pixel_x_and_twelve_pixel_y_probe() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(LINK_X, 0x128);
        ram.write_u16(LINK_Y, 0x214);
        ram.write_u16(OVERWORLD_OFFSET_BASE_X_ADDR, 0x20);
        ram.write_u16(OVERWORLD_OFFSET_BASE_Y_ADDR, 0x200);
        ram.write_u16(OVERWORLD_OFFSET_MASK_X, 0x3f);
        ram.write_u16(OVERWORLD_OFFSET_MASK_Y, 0x3f);
        // x probe = 0x26, y delta = 0x20, so position = 0x106.
        ram.write_u16(DUNG_BG2 + 0x106, 2);
        let mut data = assets();
        data.map16[8..12].copy_from_slice(&[0x10, 0x11, 0x12, 0x13]);
        assert_eq!(
            get_map16_of_link_mult8(&ram, &data),
            &[0x10, 0x11, 0x12, 0x13]
        );
    }

    #[test]
    fn special_switch_area_sets_exit_direction_and_modules() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(LINK_X, 0);
        ram.write_u16(LINK_Y, 0);
        ram.write_u16(OVERWORLD_OFFSET_MASK_X, 0);
        ram.write_u16(OVERWORLD_OFFSET_MASK_Y, 0);
        ram.write_u16(DUNG_BG2, 2);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 15);
        let mut data = assets();
        data.map16[8] = 0xad;
        check_special_switch_area(&mut ram, &data);
        assert_eq!(ram.read_u16(DUNGEON_ROOM), 0x182);
        assert_eq!(ram.read_u8(LINK_DIRECTION), 8);
        assert_eq!(ram.read_u16(BYTE_069C), 0);
        assert_eq!(ram.read_u16(OVERWORLD_SCREEN_TRANSITION), 0);
        assert_eq!(ram.read_u8(SUBMODULE), 23);
        assert_eq!(ram.read_u8(MAIN_MODULE), 11);
    }

    #[test]
    fn hammer_sound_uses_first_map8_quadrant_and_attribute_ranges() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut data = assets();
        data.map16[8] = 0x205; // map16 2, first quadrant; mask to attribute 5
        data.attributes[5] = 0x53;
        pick_hammer_sfx(&mut ram, &data, 2);
        assert_eq!(ram.read_u8(SOUND_EFFECT_1), 17);

        data.attributes[5] = 0x4f;
        ram.write_u8(SOUND_EFFECT_1, 0xaa);
        pick_hammer_sfx(&mut ram, &data, 2);
        assert_eq!(ram.read_u8(SOUND_EFFECT_1), 0xaa);
    }

    #[test]
    fn tool_interaction_cuts_bush_updates_map_and_emits_both_effects() {
        use core::cell::Cell;

        let mut data = assets();
        data.map16.resize(0x4000, 0);
        data.map16[0x36 * 4] = 5;
        data.attributes[5] = 0x51;
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_OFFSET_MASK_X, 0xffff);
        ram.write_u16(OVERWORLD_OFFSET_MASK_Y, 0xffff);
        ram.write_u16(DUNG_BG2, 0x36);

        let terrain = Cell::new(None);
        let poof = Cell::new(None);
        let result = tool_and_tile_interaction(
            &mut ram,
            &data,
            0,
            0,
            |_, kind, x, y| terrain.set(Some((kind, x, y))),
            |_, x, y| poof.set(Some((x, y))),
        );

        assert_eq!(result, 0x51);
        assert_eq!(ram.read_u16(DUNG_BG2), 0xdc7);
        assert_eq!(ram.read_u16(MEMORIZED_TILE_VALUE), 0xdc7);
        assert_eq!(ram.read_u16(NUM_MEMORIZED_TILES), 2);
        assert_eq!(ram.read_u8(NMI_LOAD_BG_FROM_VRAM), 1);
        assert_eq!(terrain.get(), Some((2, 0, 0)));
        assert_eq!(poof.get(), Some((0, 0)));
    }

    #[test]
    fn memorized_and_vram_map16_updates_match_byte_offsets_and_endianness() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut data = assets();
        data.map16.resize(0x40, 0);
        data.map16[0x0c..0x10].copy_from_slice(&[0x1111, 0x2222, 0x3333, 0x4444]);

        memorize_map16_change(&mut ram, 0x0822, 3);
        assert_eq!(ram.read_u16(NUM_MEMORIZED_TILES), 2);
        assert_eq!(ram.read_u16(MEMORIZED_TILE_ADDR), 0x0822);
        assert_eq!(ram.read_u16(MEMORIZED_TILE_VALUE), 3);
        memorize_map16_change(&mut ram, 0x1234, 0xdc5);
        assert_eq!(ram.read_u16(NUM_MEMORIZED_TILES), 2);

        draw_map16_persist(&mut ram, &data, 0x0822, 3);
        assert_eq!(ram.read_u16(DUNG_BG2 + 0x0822), 3);
        assert_eq!(ram.read_u16(VRAM_UPLOAD_OFFSET), 16);
        assert_eq!(
            ram.slice(VRAM_UPLOAD_DATA, 18),
            &[
                0x0c, 0x02, 0x00, 0x03, 0x11, 0x11, 0x22, 0x22, 0x0c, 0x22, 0x00, 0x03, 0x33, 0x33,
                0x44, 0x44, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn peg_puzzles_preserve_word_sfx_and_exact_completion_side_effects() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 7);

        assert!(!handle_peg_puzzles(&mut ram, 0x826));
        assert_eq!(ram.read_u16(PEG_PUZZLE_PROGRESS), 2);
        assert_eq!(ram.read_u16(SOUND_EFFECT_1), 0x2d00);
        assert!(!handle_peg_puzzles(&mut ram, 0x5a0));
        assert!(!handle_peg_puzzles(&mut ram, 0x81a));
        assert_eq!(ram.read_u16(SOUND_EFFECT_1), 0x1b00);
        assert_eq!(ram.read_u8(SAVE_OW_EVENT_INFO + 7), 0x20);
        assert_eq!(ram.read_u8(SUBMODULE), 47);

        ram.write_u16(PEG_PUZZLE_PROGRESS, 0);
        ram.write_u8(SAVE_OW_EVENT_INFO + 7, 0);
        assert!(!handle_peg_puzzles(&mut ram, 0x1234));
        assert_eq!(ram.read_u16(PEG_PUZZLE_PROGRESS), 0xffff);
        assert_eq!(ram.read_u16(SOUND_EFFECT_1), 0x003c);

        ram.write_u16(OVERWORLD_SCREEN_INDEX, 98);
        ram.write_u16(PEG_PUZZLE_PROGRESS, 21);
        assert!(handle_peg_puzzles(&mut ram, 0));
        assert_eq!(ram.read_u8(SAVE_OW_EVENT_INFO + 0x62), 0x20);
        assert_eq!(ram.read_u8(SOUND_EFFECT_2), 27);
        assert_eq!(ram.read_u16(DOOR_OPEN_CLOSED_COUNTER), 0x50);
        assert_eq!(ram.read_u16(BIG_ROCK_STARTING_ADDRESS), 0xd20);
    }

    #[test]
    fn waterfall_cleanup_only_terminates_splashes_on_screen_fifteen() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 0x10f);
        for (slot, kind) in [0x41, 3, 0x41, 0x40, 0x41].into_iter().enumerate() {
            ram.write_u8(ANCILLA_TYPE + slot, kind);
        }
        terminate_waterfall_splashes(&mut ram);
        assert_eq!(ram.slice(ANCILLA_TYPE, 5), &[0, 3, 0, 0x40, 0]);

        ram.write_u16(OVERWORLD_SCREEN_INDEX, 0x10e);
        ram.write_u8(ANCILLA_TYPE, 0x41);
        terminate_waterfall_splashes(&mut ram);
        assert_eq!(ram.read_u8(ANCILLA_TYPE), 0x41);
    }

    #[test]
    fn pit_destination_scans_from_last_entry_and_uses_houlihan_fallback() {
        let mut data = assets();
        data.fall_areas[2] = 9;
        data.fall_positions[2] = 0x106;
        data.fall_entrances[2] = 17;
        data.fall_areas[18] = 9;
        data.fall_positions[18] = 0x106;
        data.fall_entrances[18] = 44;
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(LINK_X, 0x37);
        ram.write_u16(LINK_Y, 0x27);
        ram.write_u16(OVERWORLD_OFFSET_MASK_X, 0x3f);
        ram.write_u16(OVERWORLD_OFFSET_MASK_Y, 0x3f);
        ram.write_u16(OVERWORLD_AREA_INDEX, 9);
        ram.write_u8(BYTE_010F, 0xa5);
        get_pit_destination(&mut ram, &data);
        assert_eq!(ram.read_u8(WHICH_ENTRANCE), 44);
        assert_eq!(ram.read_u8(BYTE_010F), 0);

        ram.write_u16(OVERWORLD_AREA_INDEX, 10);
        ram.write_u8(SAVEGAME_IS_DARKWORLD, 0x40);
        get_pit_destination(&mut ram, &data);
        assert_eq!(ram.read_u8(SAVEGAME_IS_DARKWORLD), 0);
        assert_eq!(ram.read_u8(WHICH_ENTRANCE), 130);
    }

    #[test]
    fn master_sword_palette_animation_preserves_backup_and_terminal_clear() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for index in 0..256 {
            ram.write_u16(AUX_PALETTE_BUFFER + index * 2, index as u16 ^ 0x4210);
        }
        ram.write_u16(MAIN_PALETTE_BUFFER, 0x1234);
        ram.write_u16(MAIN_PALETTE_BUFFER + 64, 0xabcd);
        palette_anim_get_master_sword2(&mut ram);
        assert_eq!(ram.read_u16(MAPBAK_PALETTE + 17 * 2), 17 ^ 0x4210);
        assert_eq!(ram.read_u16(AUX_PALETTE_BUFFER + 17 * 2), 0x7fff);
        assert_eq!(ram.read_u16(MAIN_PALETTE_BUFFER + 64), 0x1234);
        assert_eq!(ram.read_u16(DARKENING_OR_LIGHTENING_SCREEN), 2);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 1);

        ram.write_u16(DARKENING_OR_LIGHTENING_SCREEN, 0);
        ram.write_u16(PALETTE_FILTER_COUNTDOWN, 31);
        ram.write_u8(TS_COPY, 0xa5);
        palette_anim_get_master_sword3(&mut ram);
        assert_eq!(ram.read_u16(AUX_PALETTE_BUFFER + 17 * 2), 17 ^ 0x4210);
        assert_eq!(ram.read_u8(TS_COPY), 0);

        ram.write_u16(DARKENING_OR_LIGHTENING_SCREEN, 1);
        ram.write_u16(PALETTE_FILTER_COUNTDOWN, 0x1234);
        ram.write_u8(SUBMODULE, 9);
        palette_anim_get_master_sword(&mut ram, |state| {
            state.write_u16(DARKENING_OR_LIGHTENING_SCREEN, 0xff);
        });
        assert_eq!(ram.read_u16(PALETTE_FILTER_COUNTDOWN), 0);
        assert_eq!(ram.read_u16(DARKENING_OR_LIGHTENING_SCREEN), 0);
        assert_eq!(ram.read_u8(SUBMODULE), 0);
        for index in 0x58..0x60 {
            assert_eq!(ram.read_u16(MAIN_PALETTE_BUFFER + index * 2), 0);
            assert_eq!(ram.read_u16(AUX_PALETTE_BUFFER + index * 2), 0);
        }
    }

    #[test]
    fn mirror_hdma_initialization_matches_struct_layout_and_scanline_fill() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(BG2HOFS_COPY2, 0x778);
        let mut setup = None;
        initialize_mirror_hdma(&mut ram, |args| setup = Some(args));
        assert_eq!(ram.read_u8(HDMAEN_COPY), 0xc0);
        assert_eq!(ram.read_u16(MIRROR_VARS), 0);
        assert_eq!(ram.read_i16(MIRROR_VARS + 2), -0x200);
        assert_eq!(ram.read_u16(MIRROR_VARS + 4), 0x200);
        assert_eq!(ram.read_i16(MIRROR_VARS + 6), -0x40);
        assert_eq!(ram.read_u16(MIRROR_VARS + 8), 0x40);
        assert_eq!(ram.read_u16(MIRROR_VARS + 18), 21);
        assert_eq!(ram.read_u16(MIRROR_VARS + 20), 8);
        assert_eq!(ram.read_u16(MIRROR_VARS + 22), 8);
        assert_eq!(ram.read_u16(HDMA_TABLE_DYNAMIC), 0x778);
        assert_eq!(ram.read_u16(HDMA_TABLE_DYNAMIC + 239 * 2), 0x778);
        assert_eq!(
            setup,
            Some(MirrorHdmaSetup {
                channel6_address: 0xf2fb,
                channel7_address: 0xf2fb,
                transfer_unit: 0x42,
                channel6_register: 0x0d,
                channel7_register: 0x0f,
                indirect_bank: 0,
            })
        );
    }

    #[test]
    fn mirror_wave_builders_preserve_c_shift_and_terminal_scroll_copies() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for index in 0..240 {
            ram.write_u16(HDMA_TABLE_DYNAMIC + index * 2, index as u16);
        }
        ram.write_u16(BG2HOFS_COPY2, 0x778);
        ram.write_u16(MIRROR_VARS + 2, (-0x200i16) as u16);
        ram.write_u16(MIRROR_VARS + 6, (-0x40i16) as u16);
        let mut animation_calls = 0;
        mirror_warp_build_waving_hdma_table(&mut ram, |_| animation_calls += 1);
        assert_eq!(animation_calls, 1);
        assert_eq!(ram.read_u16(HDMA_TABLE_DYNAMIC + 232 * 2), 224);
        assert_eq!(ram.read_u16(HDMA_TABLE_DYNAMIC), 0x777);
        assert_eq!(ram.read_u16(HDMA_TABLE_DYNAMIC + 2 * 2), 0x777);

        for index in 0..240 {
            ram.write_u16(HDMA_TABLE_DYNAMIC + index * 2, 0x778);
        }
        ram.write_u16(BG2VOFS_COPY2, 0x456);
        ram.write_u8(HDMAEN_COPY, 0xc0);
        let mut fixed_calls = 0;
        mirror_warp_build_dewaving_hdma_table(
            &mut ram,
            |_| animation_calls += 1,
            |_| fixed_calls += 1,
        );
        assert_eq!(animation_calls, 2);
        assert_eq!(fixed_calls, 1);
        assert_eq!(ram.read_u8(HDMAEN_COPY), 0);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 1);
        assert_eq!(ram.read_u16(BG1HOFS_COPY2), 0x778);
        assert_eq!(ram.read_u16(BG1HOFS_COPY), 0x778);
        assert_eq!(ram.read_u16(BG2HOFS_COPY), 0x778);
        assert_eq!(ram.read_u16(BG1VOFS_COPY2), 0x456);
        assert_eq!(ram.read_u16(BG1VOFS_COPY), 0x456);
        assert_eq!(ram.read_u16(BG2VOFS_COPY), 0x456);
    }

    #[test]
    fn mirror_warp_state_one_falls_through_to_wave_builder() {
        use core::cell::Cell;

        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(SUBSUBMODULE, 1);
        let waves = Cell::new(0);
        module09_mirror_warp(
            &mut ram,
            |_| panic!("unexpected initialize"),
            |_| panic!("unexpected white filter"),
            |_| panic!("unexpected graphics load"),
            |_| waves.set(waves.get() + 1),
            |_| panic!("unexpected dewave"),
            |_| panic!("unexpected finalize"),
        );
        assert_eq!(waves.get(), 1);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 2);
        assert_eq!(ram.read_u8(HDMAEN_COPY), 0xc0);
        assert_eq!(ram.read_u8(NMI_DISABLE_CORE_UPDATES), 1);
    }

    #[test]
    fn death_mountain_palette_animation_uses_exact_flash_frames() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 0x43);
        ram.write_u8(FRAME_COUNTER, 36);
        dw_death_mountain_palette_animation(&mut ram);
        assert_eq!(ram.read_u8(SOUND_EFFECT_1), 54);
        assert_eq!(ram.read_u16(MAIN_PALETTE_BUFFER + 0x31 * 2), 0x884);
        assert_eq!(ram.read_u16(MAIN_PALETTE_BUFFER + 0x39 * 2), 0x884);
        assert_eq!(ram.read_u16(MAIN_PALETTE_BUFFER + 0x69 * 2), 0x1908);
        assert_eq!(ram.read_u8(FLAG_UPDATE_CGRAM), 1);

        ram.write_u8(FRAME_COUNTER, 5);
        ram.write_u16(AUX_PALETTE_BUFFER + 0x31 * 2, 0x4567);
        dw_death_mountain_palette_animation(&mut ram);
        assert_eq!(ram.read_u16(MAIN_PALETTE_BUFFER + 0x31 * 2), 0x4567);
    }

    #[test]
    fn whirlpool_state_machine_preserves_callback_and_terminal_order() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        module09_whirlpool(&mut ram, |_| panic!(), |_| panic!());
        assert_eq!(ram.read_u8(SOUND_EFFECT_1), 0x34);
        assert_eq!(ram.read_u8(SOUND_EFFECT_AMBIENT), 5);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 1);
        assert_eq!(ram.read_u8(NMI_DISABLE_CORE_UPDATES), 1);

        let mut blue = 0;
        module09_whirlpool(&mut ram, |_| blue += 1, |_| panic!());
        assert_eq!(blue, 1);
        ram.write_u8(SUBSUBMODULE, 2);
        let mut isolate = 0;
        module09_whirlpool(&mut ram, |_| panic!(), |_| isolate += 1);
        assert_eq!(isolate, 1);

        ram.write_u8(SUBSUBMODULE, 3);
        ram.write_u8(SAVEGAME_IS_DARKWORLD, 0x40);
        module09_whirlpool(&mut ram, |_| panic!(), |_| panic!());
        assert_eq!(ram.read_u8(COLDATA_COPY2), 0x9f);
        assert_eq!(ram.read_u8(MUSIC_CONTROL), 9);
        assert_eq!(ram.read_u8(SUBMODULE), 0);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 0);
        assert_eq!(ram.read_u8(NMI_DISABLE_CORE_UPDATES), 0);
    }

    #[test]
    fn drowning_recovery_walks_two_pixels_and_dispatches_scroll_in_c_order() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(LINK_X, 12);
        ram.write_u16(LINK_X_CACHED, 9);
        ram.write_u16(LINK_Y, 5);
        ram.write_u16(LINK_Y_CACHED, 6);
        ram.write_u8(OVERWORLD_TRANS_DIR2, 8);
        let order = std::cell::RefCell::new(Vec::new());
        scroll_to_land(
            &mut ram,
            |_| order.borrow_mut().push(1),
            |_| order.borrow_mut().push(2),
        );
        assert_eq!(*order.borrow(), [1, 2]);
        assert_eq!(ram.read_u16(LINK_X), 10);
        assert_eq!(ram.read_u16(LINK_Y), 6);
        assert_eq!(ram.read_u8(LINK_X_VELOCITY), (-2i8) as u8);
        assert_eq!(ram.read_u8(LINK_Y_VELOCITY), 1);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 0);

        scroll_to_land(&mut ram, |_| {}, |_| {});
        assert_eq!(ram.read_u16(LINK_X), 9);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 1);
        assert_eq!(ram.read_u8(LINK_INCAPACITATED_TIMER), 0);
        assert_eq!(ram.read_u8(SET_WHEN_DAMAGING_ENEMIES), 0);
    }

    #[test]
    fn whirlpool_exit_tile_update_memorizes_draws_and_requests_upload() {
        let mut data = assets();
        data.map16.resize((0x212 + 1) * 4, 0);
        data.map16[0x212 * 4..0x212 * 4 + 4].copy_from_slice(&[0x1111, 0x2222, 0x3333, 0x4444]);
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(SUBMODULE, 47);
        overworld_func2f(&mut ram, &data);
        assert_eq!(ram.read_u16(DUNG_BG2 + 0x720), 0x212);
        assert_eq!(ram.read_u16(MEMORIZED_TILE_ADDR), 0x720);
        assert_eq!(ram.read_u16(MEMORIZED_TILE_VALUE), 0x212);
        assert_eq!(ram.read_u16(NUM_MEMORIZED_TILES), 2);
        assert_eq!(ram.read_u16(VRAM_UPLOAD_OFFSET), 16);
        assert_eq!(ram.read_u8(NMI_LOAD_BG_FROM_VRAM), 1);
        assert_eq!(ram.read_u8(SUBMODULE), 0);
    }

    #[test]
    fn entrance_dispatch_sets_all_three_immobilization_mirrors() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for (sequence, expected) in [
            EntranceAnimation::PalaceOfDarkness,
            EntranceAnimation::SkullWoods,
            EntranceAnimation::MiseryMire,
            EntranceAnimation::TurtleRock,
            EntranceAnimation::GanonsTower,
        ]
        .into_iter()
        .enumerate()
        {
            ram.write_u8(TRIGGER_SPECIAL_ENTRANCE, sequence as u8 + 1);
            assert_eq!(animate_entrance(&mut ram), expected);
            assert_eq!(ram.read_u8(FLAG_LINK_IMMOBILIZED), sequence as u8 + 1);
            assert_eq!(ram.read_u8(FLAG_UNK1), sequence as u8 + 1);
            assert_eq!(ram.read_u8(NMI_DISABLE_CORE_UPDATES), sequence as u8 + 1);
        }
    }

    #[test]
    fn flute_transport_captures_byte_before_shifting_shared_word() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(NUM_MEMORIZED_TILES, 12);
        ram.write_u16(BIRD_TRAVEL_VAR1, 0x8123);
        let mut observed = None;
        flute_menu_load_transport(&mut ram, |state, position| {
            observed = Some((position, state.read_u16(BIRD_TRAVEL_VAR1)));
        });
        assert_eq!(ram.read_u16(NUM_MEMORIZED_TILES), 0);
        assert_eq!(observed, Some((0x23, 0x0246)));
    }

    #[test]
    fn partner_whirlpool_uses_last_matching_area_and_transport_offset() {
        let mut data = assets();
        data.whirlpools = vec![4, 7, 4];
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 4);
        ram.write_u16(NUM_MEMORIZED_TILES, 8);
        let mut destination = None;
        assert!(find_partner_whirlpool_exit(&mut ram, &data, |_, index| {
            destination = Some(index)
        }));
        assert_eq!(destination, Some(11));
        assert_eq!(ram.read_u16(NUM_MEMORIZED_TILES), 0);

        ram.write_u16(OVERWORLD_SCREEN_INDEX, 5);
        assert!(!find_partner_whirlpool_exit(&mut ram, &data, |_, _| {
            panic!()
        }));
    }

    #[test]
    fn map16_stripe_builder_preserves_wrapping_row_buffer_and_vram_headers() {
        let mut data = assets();
        data.map16 = (0..32u16)
            .flat_map(|tile| [tile * 4, tile * 4 + 1, tile * 4 + 2, tile * 4 + 3])
            .collect();
        let mut source = vec![0u8; 0x2000];
        for tile in 0..32u16 {
            source[usize::from(tile) * 2..usize::from(tile) * 2 + 2]
                .copy_from_slice(&tile.to_le_bytes());
        }
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x410);
        ram.write_u16(MAP16_LOAD_DST_OFFSET, 31);
        ram.write_u16(MAP16_LOAD_VAR2, 0x12);
        copy_map16_to_buffer(&mut ram, &data, &source, 0x1000, 0, MAP16_STRIPE_HEADERS);
        assert_eq!(ram.read_u16(MAP16_ROW_BUFFER), 1);
        assert_eq!(ram.read_u16(MAP16_ROW_BUFFER + 31 * 2), 0);
        assert_eq!(ram.read_u16(MAP16_STRIPE_HEADERS), 0x1880);
        assert_eq!(ram.read_u16(MAP16_STRIPE_HEADERS + 2), 0x1c80);
        assert_eq!(ram.read_u16(DUNG_BG2_ATTR_TABLE), 4);
        assert_eq!(ram.read_u16(DUNG_BG2_ATTR_TABLE + 2), 5);
        assert_eq!(ram.read_u16(DUNG_BG2_ATTR_TABLE + 64), 6);
        assert_eq!(ram.read_u16(DUNG_BG2_ATTR_TABLE + 66), 7);
    }

    #[test]
    fn quadrant_decompression_interleaves_planes_then_expands_map32_cells() {
        let mut data = assets();
        data.map32 = [
            vec![1, 2, 3, 4, 0x50, 0x60],
            vec![7, 8, 9, 10, 0xb0, 0xc0],
            vec![13, 14, 15, 16, 0x10, 0x20],
            vec![19, 20, 21, 22, 0x30, 0x40],
        ];
        let zeros = vec![0xe4, 0xff, 0, 0xff];
        data.hibytes = vec![zeros.clone()];
        data.lobytes = vec![zeros];
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        decompress_and_draw_one_quadrant(&mut ram, &data, 0x2000, 0);
        assert_eq!(ram.read_u16(0x2000), 0x501);
        assert_eq!(ram.read_u16(0x2002), 0xb07);
        assert_eq!(ram.read_u16(0x2080), 0x10d);
        assert_eq!(ram.read_u16(0x2082), 0x313);
        assert_eq!(ram.read_u16(0x2100), 0x501);
        assert_eq!(ram.read_u16(MAP16_DECODE_LAST), 0);
    }

    #[test]
    fn overlay_load_builds_map8_buffer_and_arms_nmi() {
        let mut data = assets();
        data.map32 = core::array::from_fn(|_| vec![0; 6]);
        let zeros = vec![0xe4, 0xff, 0, 0xff];
        data.hibytes = vec![zeros.clone()];
        data.lobytes = vec![zeros];
        data.map16 = vec![0x1111, 0x2222, 0x3333, 0x4444];
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(SUBMODULE, 6);
        load_overworld_overlay(&mut ram, &data);
        assert_eq!(ram.read_u16(DUNG_BG2_ATTR_TABLE), 0x1111);
        assert_eq!(ram.read_u16(DUNG_BG2_ATTR_TABLE + 2), 0x2222);
        assert_eq!(ram.read_u8(NMI_DISABLE_CORE_UPDATES), 4);
        assert_eq!(ram.read_u8(NMI_SUBROUTINE_INDEX), 4);
        assert_eq!(ram.read_u8(SUBMODULE), 7);
    }

    #[test]
    fn ambient_overlay_temporarily_uses_small_map_offsets_then_restores_them() {
        let mut data = assets();
        data.map16 = vec![0x1111, 0x2222, 0x3333, 0x4444];
        data.map_is_small[2] = 1;
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 2);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x777);
        ram.write_u16(MAP16_LOAD_DST_OFFSET, 9);
        ram.write_u16(MAP16_LOAD_VAR2, 3);
        ram.write_u8(SUBMODULE, 5);
        ram.write_u8(INIDISP_COPY, 15);
        let mut observed = None;
        load_ambient_overlay(&mut ram, &data, true, |state| {
            observed = Some((
                state.read_u16(MAP16_LOAD_SRC_OFFSET),
                state.read_u16(MAP16_LOAD_DST_OFFSET),
                state.read_u16(MAP16_LOAD_VAR2),
            ));
        });
        assert_eq!(observed, Some((0x390, 0, 0x1f)));
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0x777);
        assert_eq!(ram.read_u16(MAP16_LOAD_DST_OFFSET), 9);
        assert_eq!(ram.read_u16(MAP16_LOAD_VAR2), 3);
        assert_eq!(ram.read_u8(NMI_SUBROUTINE_INDEX), 4);
        assert_eq!(ram.read_u8(NMI_DISABLE_CORE_UPDATES), 4);
        assert_eq!(ram.read_u8(SUBMODULE), 6);
        assert_eq!(ram.read_u8(INIDISP_COPY), 0);
    }

    #[test]
    fn bomb_door_overlay_uses_event_bits_and_byte_screen_index() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 0x33);
        handle_overlays_and_bomb_doors(&mut ram, |_| {});
        assert_eq!(ram.read_u16(DUNG_BG2 + 340 * 2), 0x20f);

        let screen = SECONDARY_OVERLAY_PER_OW
            .iter()
            .position(|&position| position != 0)
            .unwrap();
        ram.write_u16(OVERWORLD_SCREEN_INDEX, screen as u16);
        ram.write_u8(SAVE_OW_EVENT_INFO + screen, 0x22);
        let mut event_calls = 0;
        handle_overlays_and_bomb_doors(&mut ram, |_| event_calls += 1);
        assert_eq!(event_calls, 1);
        let position = usize::from(SECONDARY_OVERLAY_PER_OW[screen]);
        assert_eq!(ram.read_u16(DUNG_BG2 + position), 0xdb4);
        assert_eq!(ram.read_u16(DUNG_BG2 + position + 2), 0xdb5);
    }

    #[test]
    fn initial_scroll_map_dispatches_all_big_and_small_directions() {
        for small in [false, true] {
            for (direction, axis, count) in [
                (1, MapStripeAxis::X, 8),
                (2, MapStripeAxis::X, 7),
                (4, MapStripeAxis::Y, 8),
                (8, MapStripeAxis::Y, 7),
            ] {
                let mut data = assets();
                data.map_is_small[0] = u8::from(small);
                let mut bytes = [0; WRAM_SIZE];
                let mut ram = Ram::new(&mut bytes);
                ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x1234);
                ram.write_u16(MAP16_LOAD_DST_OFFSET, 5);
                ram.write_u16(MAP16_LOAD_VAR2, 6);
                ram.write_u8(OVERWORLD_TRANS_DIR2, direction);
                let mut call = None;
                assert!(create_initial_new_screen_map_to_scroll(
                    &mut ram,
                    &data,
                    |_, got_axis, got_count| call = Some((got_axis, got_count))
                ));
                assert_eq!(call, Some((axis, count)));
            }
        }

        let mut data = assets();
        data.map_is_small[0] = 1;
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x1234);
        ram.write_u16(MAP16_LOAD_DST_OFFSET, 5);
        ram.write_u16(MAP16_LOAD_VAR2, 6);
        ram.write_u8(OVERWORLD_TRANS_DIR2, 8);
        assert!(create_initial_new_screen_map_to_scroll(
            &mut ram,
            &data,
            |_, _, _| {}
        ));
        assert_eq!(ram.read_u16(ORANGE_BLUE_BARRIER_STATE), 0xb34);
        assert_eq!(ram.read_u16(WORD_C174), 5);
        assert_eq!(ram.read_u16(WORD_C176), 10);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0x1390);
        assert_eq!(ram.read_u16(MAP16_LOAD_DST_OFFSET), 0);
        assert_eq!(ram.read_u16(MAP16_LOAD_VAR2), 31);
    }

    #[test]
    fn horizontal_and_vertical_stripe_buffers_match_interleaved_vram_layout() {
        let mut data = assets();
        data.map16 = (0..32u16)
            .flat_map(|tile| [tile * 4, tile * 4 + 1, tile * 4 + 2, tile * 4 + 3])
            .collect();
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for index in 0..32 {
            ram.write_u16(DUNG_BG2 + index * 128, index as u16);
        }
        ram.write_u8(OVERWORLD_TRANS_DIR2, 1);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x3d0);
        assert_eq!(
            buffer_and_build_map16_stripes_x(&mut ram, &data, UVRAM_DATA),
            UVRAM_DATA + 264
        );
        assert_eq!(ram.read_u16(UVRAM_DATA), 0);
        assert_eq!(ram.read_u16(UVRAM_DATA + 66), 1);
        assert_eq!(ram.read_u16(UVRAM_DATA + 2), 0);
        assert_eq!(ram.read_u16(UVRAM_DATA + 68), 1);
        assert_eq!(ram.read_u16(UVRAM_DATA + 4), 2);
        assert_eq!(ram.read_u16(UVRAM_DATA + 70), 3);

        for index in 0..32 {
            ram.write_u16(DUNG_BG2 + index * 2, index as u16);
        }
        ram.write_u8(OVERWORLD_TRANS_DIR2, 4);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0xf410);
        assert_eq!(
            buffer_and_build_map16_stripes_y(&mut ram, &data, UVRAM_DATA),
            UVRAM_DATA + 260
        );
        assert_eq!(ram.read_u16(UVRAM_DATA), 0);
        assert_eq!(ram.read_u16(UVRAM_DATA + 2), 0);
        assert_eq!(ram.read_u16(UVRAM_DATA + 4), 1);
        assert_eq!(ram.read_u16(UVRAM_DATA + 64 + 2), 2);
        assert_eq!(ram.read_u16(UVRAM_DATA + 66 + 2), 3);
    }

    #[test]
    fn transition_and_initial_stripe_wrappers_update_offsets_after_build() {
        let mut data = assets();
        data.map16 = (0..32u16)
            .flat_map(|tile| [tile * 4, tile * 4 + 1, tile * 4 + 2, tile * 4 + 3])
            .collect();
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for index in 0..32 {
            ram.write_u16(DUNG_BG2 + index * 128, index as u16);
        }
        ram.write_u8(OVERWORLD_TRANS_DIR2, 1);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x3d0);
        transition_scroll_and_load_map(&mut ram, &data);
        assert_eq!(ram.read_u16(UVRAM_DATA), 0x8040);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0x3d2);
        assert_eq!(ram.read_u16(MAP16_LOAD_DST_OFFSET), 1);
        assert_eq!(ram.read_u8(NMI_SUBROUTINE_INDEX), 3);
        assert_eq!(ram.read_u16(UVRAM_DATA + 2 + 264), 0xffff);
        assert_eq!(ram.read_u16(UVRAM_DATA + 2 + 266), 0xffff);

        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x410);
        ram.write_u16(MAP16_LOAD_DST_OFFSET, 0);
        ram.write_u16(MAP16_LOAD_VAR2, 0);
        trigger_and_finish_map_load_stripe(&mut ram, &data, MapStripeAxis::Y, 2);
        assert_eq!(ram.read_u8(OVERWORLD_TRANS_DIR2), 8);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0x310);
        assert_eq!(ram.read_u16(MAP16_LOAD_VAR2), 30);
        assert_eq!(ram.read_u16(UVRAM_DATA), 0x80);
    }

    #[test]
    fn incremental_map_scroll_honors_boundaries_and_diagonal_direction_bits() {
        let mut data = assets();
        data.map_is_small[0] = 1;
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x80);
        ram.write_u16(MAP16_LOAD_VAR2, 0);
        ram.write_u8(OVERWORLD_TRANS_DIR2, 8);
        handle_map_scroll(&mut ram, &data);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0);
        assert_eq!(ram.read_u16(MAP16_LOAD_VAR2), 31);
        assert_eq!(ram.read_u8(OVERWORLD_TRANS_DIR2), 0);
        assert_eq!(ram.read_u16(UVRAM_DATA), 0xffff);

        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x1700);
        ram.write_u16(MAP16_LOAD_VAR2, 7);
        ram.write_u8(OVERWORLD_TRANS_DIR2, 5);
        handle_map_scroll(&mut ram, &data);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0x1780);
        assert_eq!(ram.read_u16(MAP16_LOAD_VAR2), 8);
        assert_eq!(ram.read_u8(OVERWORLD_TRANS_DIR2), 1);
        assert_eq!(ram.read_u16(OVERWORLD_SCREEN_TRANSITION), 1);

        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0);
        ram.write_u8(OVERWORLD_TRANS_DIR2, 8);
        handle_map_scroll(&mut ram, &data);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0);
    }

    #[test]
    fn camera_boundary_step_updates_axis_accumulator_and_transition_bit() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(BG2HOFS_COPY2, 100);
        ram.write_u16(OW_SCROLL_VARS0, 200);
        ram.write_u16(CAMERA_SCROLL_HIGH, 50);
        ram.write_u16(OVERWORLD_UNK1, 15);
        assert_eq!(camera_boundary_check(&mut ram, 0, 0, 1, 0), 1);
        assert_eq!(ram.read_u16(BG2HOFS_COPY2), 101);
        assert_eq!(ram.read_u16(CAMERA_SCROLL_HIGH), 51);
        assert_eq!(ram.read_u16(CAMERA_SCROLL_LOW), 53);
        assert_eq!(ram.read_u16(OVERWORLD_UNK1), 0);
        assert_eq!(ram.read_u16(OVERWORLD_TRANS_DIR2), 8);

        ram.write_u16(BG2HOFS_COPY2, 200);
        ram.write_u16(OVERWORLD_UNK1, 0x1234);
        ram.write_u16(OVERWORLD_UNK1 + 2, 0x5678);
        assert_eq!(camera_boundary_check(&mut ram, 0, 0, -1, 0), 0);
        assert_eq!(ram.read_u16(OVERWORLD_UNK1), 0);
        assert_eq!(ram.read_u16(OVERWORLD_UNK1 + 2), 0);
    }

    #[test]
    fn camera_scroll_accumulates_pixels_and_half_speed_bg1_motion() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(LINK_Y, 100);
        ram.write_u8(LINK_Y_VELOCITY, 2);
        ram.write_u16(CAMERA_SCROLL_HIGH, 50);
        ram.write_u16(BG2VOFS_COPY2, 100);
        ram.write_u16(OW_SCROLL_VARS0 + 2, 200);
        ram.write_u16(BG1VOFS_COPY2, 10);

        operate_camera_scroll(&mut ram);

        assert_eq!(ram.read_u16(BG2VOFS_COPY2), 102);
        assert_eq!(ram.read_u16(CAMERA_SCROLL_HIGH), 52);
        assert_eq!(ram.read_u16(CAMERA_SCROLL_LOW), 54);
        assert_eq!(ram.read_u16(BYTE_069E), 2);
        assert_eq!(ram.read_u16(BG1VOFS_COPY2), 11);
        assert_eq!(ram.read_u16(BG1VOFS_SUBPIXEL), 0);
    }

    #[test]
    fn camera_scroll_preserves_negative_half_pixel_carry() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(LINK_X, 8);
        ram.write_u8(LINK_X_VELOCITY, 0xff);
        ram.write_u16(CAMERA_SCROLL_LOW + 4, 50);
        ram.write_u16(BG2HOFS_COPY2, 100);
        ram.write_u16(OW_SCROLL_VARS0 + 4, 200);
        ram.write_u16(BG1HOFS_COPY2, 10);

        operate_camera_scroll(&mut ram);

        assert_eq!(ram.read_u16(BG2HOFS_COPY2), 99);
        assert_eq!(ram.read_u16(BYTE_069E + 1), 0xffff);
        assert_eq!(ram.read_u16(BG1HOFS_COPY2), 9);
        assert_eq!(ram.read_u16(BG1HOFS_SUBPIXEL), 0x8000);
    }

    #[test]
    fn screen_scroll_transition_returns_early_until_target_then_finalizes() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_TRANSITION, 1);
        ram.write_u16(BG2VOFS_COPY2, 100);
        ram.write_u16(UP_DOWN_SCROLL_TARGET + 2, 500);
        assert_eq!(scroll_transition(&mut ram, |_| panic!()), 108);
        assert_eq!(ram.read_u8(TRANSITION_COUNTER), 1);

        ram.write_u16(OVERWORLD_SCREEN_TRANSITION, 3);
        ram.write_u16(BG2HOFS_COPY2, 100);
        ram.write_u16(UP_DOWN_SCROLL_TARGET + 6, 108);
        ram.write_u8(TRANSITION_COUNTER, 29);
        ram.write_u16(LINK_X, 0x208);
        ram.write_u16(CURRENT_AREA_OF_PLAYER, 2);
        let mut initialized = 0;
        assert_eq!(scroll_transition(&mut ram, |_| initialized += 1), 108);
        assert_eq!(initialized, 1);
        assert_eq!(ram.read_u16(LINK_X), 0x210);
        assert_eq!(ram.read_u8(TRANSITION_COUNTER), 0);
        assert_eq!(ram.read_u8(FLAG_OVERWORLD_AREA_DID_CHANGE), 1);
        assert_eq!(ram.read_u8(SUBMODULE), 1);
        assert_eq!(
            ram.read_u16(OW_SCROLL_VARS0 + 4),
            OVERWORLD_OFFSET_BASE_X[2]
        );
    }

    #[test]
    fn finalize_entry_moves_link_then_restores_modules_music_and_camera() {
        use std::cell::RefCell;

        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let data = assets();
        ram.write_u8(BYTE_069C, 3);
        ram.write_u16(LINK_X, 0x0e);
        ram.write_u8(SUBMODULE, 9);
        ram.write_u8(SUBSUBMODULE, 4);
        ram.write_u8(OVERWORLD_SCREEN_INDEX, 2);
        ram.write_u8(OVERWORLD_MUSIC + 2, 0x9a);
        ram.write_u8(MUSIC_UNK1, 0xf1);

        let calls = RefCell::new(Vec::new());
        finalize_entry_onto_screen(
            &mut ram,
            &data,
            |_| calls.borrow_mut().push(1),
            |_| calls.borrow_mut().push(2),
        );

        assert_eq!(ram.read_u16(LINK_X), 0x10);
        assert_eq!(ram.read_u8(SUBMODULE), 0);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 0);
        assert_eq!(ram.read_u8(SOUND_EFFECT_AMBIENT), 9);
        assert_eq!(ram.read_u8(MUSIC_CONTROL), 0x0a);
        assert_eq!(*calls.borrow(), [1, 2]);
    }

    #[test]
    fn func1f_moves_one_pixel_and_completes_the_countdown() {
        use std::cell::RefCell;

        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(BYTE_069C, 2);
        ram.write_u16(LINK_X, 0x100);
        ram.write_u8(OW_COUNTDOWN_TRANSITION, 1);
        ram.write_u8(MAIN_MODULE, 3);
        ram.write_u8(SUBMODULE, 4);
        ram.write_u8(SUBSUBMODULE, 5);
        let calls = RefCell::new(Vec::new());

        overworld_func1f(
            &mut ram,
            |_| calls.borrow_mut().push(1),
            |_| calls.borrow_mut().push(2),
        );

        assert_eq!(ram.read_u16(LINK_X), 0xff);
        assert_eq!(ram.read_u8(LINK_X_VELOCITY), 0xff);
        assert_eq!(ram.read_u8(MAIN_MODULE), 9);
        assert_eq!(ram.read_u8(SUBMODULE), 0);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 0);
        assert_eq!(*calls.borrow(), [1, 2]);
    }

    #[test]
    fn sprite_pointer_selects_progress_bank_with_exact_144_entry_stride() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut data = assets();
        data.sprite_offsets[4] = 3;
        data.sprite_offsets[148] = 7;
        data.sprite_offsets[292] = 11;
        assert_eq!(get_overworld_sprite_ptr(&ram, &data, 4)[0], 3);
        ram.write_u8(SRAM_PROGRESS_INDICATOR, 2);
        assert_eq!(get_overworld_sprite_ptr(&ram, &data, 4)[0], 7);
        ram.write_u8(SRAM_PROGRESS_INDICATOR, 3);
        assert_eq!(get_overworld_sprite_ptr(&ram, &data, 4)[0], 11);
        ram.write_u8(SRAM_PROGRESS_INDICATOR, 4);
        assert_eq!(get_overworld_sprite_ptr(&ram, &data, 4)[0], 3);
    }

    #[test]
    fn powder_secret_adjustment_only_changes_state_for_the_powder_bit() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(DUNG_SECRETS_UNK1, 0x1234);
        adjust_secret_for_powder(&mut ram);
        assert_eq!(ram.read_u16(DUNG_SECRETS_UNK1), 0x1234);
        ram.write_u8(LINK_ITEM_IN_HAND, 0x40);
        adjust_secret_for_powder(&mut ram);
        assert_eq!(ram.read_u16(DUNG_SECRETS_UNK1), 4);
    }

    #[test]
    fn secret_reveal_preserves_chime_event_and_powder_rules() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut data = assets();
        data.secrets = vec![0x34, 0x12, 0x82, 0xff, 0xff];
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 2);
        assert_eq!(reveal_secret(&mut ram, &data, 0x1234), 0x212);
        assert_eq!(ram.read_u8(DUNG_SECRETS_UNK1), 0xff);
        assert_eq!(ram.read_u8(SOUND_EFFECT_2), 0x1b);

        ram.write_u8(SOUND_EFFECT_2, 0);
        ram.write_u8(SAVE_OW_EVENT_INFO + 2, 2);
        ram.write_u8(ENHANCED_FEATURES0 + 1, 0x10);
        ram.write_u8(LINK_ITEM_IN_HAND, 0x40);
        assert_eq!(reveal_secret(&mut ram, &data, 0x1234), 0x212);
        assert_eq!(ram.read_u8(DUNG_SECRETS_UNK1), 4);
        assert_eq!(ram.read_u8(SOUND_EFFECT_2), 0x1b);

        data.secrets[2] = 5;
        ram.write_u8(LINK_ITEM_IN_HAND, 0);
        assert_eq!(reveal_secret(&mut ram, &data, 0x1234), 0);
        assert_eq!(ram.read_u8(DUNG_SECRETS_UNK1), 5);
    }

    #[test]
    fn bomb_tile_replaces_bush_and_spawns_smashed_terrain() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut data = assets();
        data.map16.resize(0xdc9 * 4, 0);
        ram.write_u16(OVERWORLD_OFFSET_MASK_X, 0x3f);
        ram.write_u16(OVERWORLD_OFFSET_MASK_Y, 0x3f0);
        ram.write_u16(DUNG_BG2 + 2, 0x36);
        let mut spawned = None;
        bomb_tile(&mut ram, &data, 16, 0, |_, kind, x, y| {
            spawned = Some((kind, x, y));
        });
        assert_eq!(ram.read_u16(DUNG_BG2 + 2), 0xdc7);
        assert_eq!(ram.read_u16(NUM_MEMORIZED_TILES), 2);
        assert_eq!(ram.read_u16(MEMORIZED_TILE_ADDR), 2);
        assert_eq!(ram.read_u16(MEMORIZED_TILE_VALUE), 0xdc7);
        assert_eq!(spawned, Some((2, 16, 0)));
        assert_eq!(ram.read_u8(NMI_LOAD_BG_FROM_VRAM), 1);
    }

    #[test]
    fn mirror_bonk_restore_replays_every_memorized_map16_change() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(NUM_MEMORIZED_TILES, 4);
        ram.write_u16(MEMORIZED_TILE_ADDR, 0x20);
        ram.write_u16(MEMORIZED_TILE_ADDR + 2, 0x42);
        ram.write_u16(MEMORIZED_TILE_VALUE, 0x1234);
        ram.write_u16(MEMORIZED_TILE_VALUE + 2, 0xabcd);
        recover_mirror_bonk_changed_tiles(&mut ram);
        assert_eq!(ram.read_u16(DUNG_BG2 + 0x20), 0x1234);
        assert_eq!(ram.read_u16(DUNG_BG2 + 0x42), 0xabcd);
    }

    #[test]
    fn enemy_damage_subclasses_expand_high_nibble_then_low_nibble() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut data = assets();
        data.enemy_damage[0] = 0xa5;
        data.enemy_damage[1] = 0x3c;
        data.enemy_damage[0x7ff] = 0xe1;
        decompress_enemy_damage_subclasses(&mut ram, &data);
        assert_eq!(ram.slice(ENEMY_DAMAGE_SCRATCH, 2), &[0xa5, 0x3c]);
        assert_eq!(ram.slice(ENEMY_DAMAGE_DATA, 4), &[0xa, 5, 3, 0xc]);
        assert_eq!(ram.slice(ENEMY_DAMAGE_DATA + 0xffe, 2), &[0xe, 1]);
    }

    #[test]
    fn camera_boundaries_use_area_origin_and_small_or_big_extents() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        set_camera_boundaries(&mut ram, false, 18);
        assert_eq!(ram.slice(OW_SCROLL_VARS0, 8), &[0, 4, 0x1e, 5, 0, 4, 0, 5]);
        assert_eq!(ram.read_u16(UP_DOWN_SCROLL_TARGET), 0x320);
        assert_eq!(ram.read_u16(UP_DOWN_SCROLL_TARGET_END), 0x600);
        assert_eq!(ram.read_u16(LEFT_RIGHT_SCROLL_TARGET), 0x300);
        assert_eq!(ram.read_u16(LEFT_RIGHT_SCROLL_TARGET_END), 0x600);

        set_camera_boundaries(&mut ram, true, 58);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS0), 0xe00);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS0 + 2), 0x111e);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS0 + 4), 0x400);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS0 + 6), 0x700);
    }

    #[test]
    fn mosaic_growth_and_song_bank_selection_preserve_c_wrapping_and_overwrite() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(MOSAIC_LEVEL, 0xf8);
        ram.write_u16(PALETTE_FILTER_COUNTDOWN, 1);
        conditional_mosaic_control(&mut ram);
        assert_eq!(ram.read_u8(MOSAIC_LEVEL), 8);
        assert_eq!(ram.read_u8(MOSAIC), 15);
        reset_mosaic_always_increase(&mut ram);
        assert_eq!(ram.read_u8(MOSAIC_LEVEL), 0x18);

        let data = assets();
        ram.write_u8(SRAM_PROGRESS_INDICATOR, 2);
        ram.write_u8(LINK_SWORD_TYPE, 1);
        set_song_list(&mut ram, &data);
        assert_eq!(ram.slice(OVERWORLD_MUSIC, 3), &[0x40, 0x41, 0x42]);
        assert_eq!(ram.read_u8(OVERWORLD_MUSIC + 128), 5);
        assert_eq!(ram.read_u8(OVERWORLD_MUSIC + 159), 160);

        ram.write_u8(SRAM_PROGRESS_INDICATOR, 3);
        set_song_list(&mut ram, &data);
        assert_eq!(ram.slice(OVERWORLD_MUSIC, 3), &[0xc0, 0xc1, 0xc2]);
        assert_eq!(ram.read_u8(OVERWORLD_MUSIC + 128), 2);
    }

    #[test]
    fn pit_damage_and_bunny_reset_keep_c_underflow_and_feature_rules() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(LINK_HEALTH_CURRENT, 4);
        take_damage_from_pit(&mut ram);
        assert_eq!(ram.read_u8(LINK_VISIBILITY), 12);
        assert_eq!(ram.read_u8(SUBMODULE), 42);
        assert_eq!(ram.read_u8(LINK_HEALTH_CURRENT), 0);
        ram.write_u8(PLAYER_IS_INDOORS, 1);
        ram.write_u8(LINK_HEALTH_CURRENT, 0x20);
        take_damage_from_pit(&mut ram);
        assert_eq!(ram.read_u8(SUBMODULE), 20);
        assert_eq!(ram.read_u8(LINK_HEALTH_CURRENT), 0x18);

        ram.write_u8(LINK_ITEM_MOON_PEARL, 1);
        ram.write_u8(LINK_IS_BUNNY, 1);
        ram.write_u8(LINK_IS_BUNNY_MIRROR, 1);
        ram.write_u8(LINK_IS_RUNNING, 1);
        ram.write_u8(ENHANCED_FEATURES0, 4);
        adjust_link_bunny_status(&mut ram);
        assert_eq!(ram.read_u8(LINK_IS_BUNNY), 0);
        assert_eq!(ram.read_u8(LINK_IS_BUNNY_MIRROR), 0);
        assert_eq!(ram.read_u8(LINK_IS_RUNNING), 0);
    }

    #[test]
    fn drowning_recovery_restores_cached_state_and_enters_death_module() {
        use std::cell::RefCell;

        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(LINK_Y_CACHED, 0x1234);
        ram.write_u16(LINK_X_CACHED, 0x5678);
        for offset in (0..8).step_by(2) {
            ram.write_u16(ROOM_SCROLL_VARS_CACHED + offset, 0x100 + offset as u16);
            ram.write_u16(SCROLL_TARGETS_CACHED + offset, 0x200 + offset as u16);
        }
        ram.write_u16(CAMERA_SCROLL_LOW, 0x300);
        ram.write_u16(CAMERA_SCROLL_LOW + 4, 0x400);
        ram.write_u16(QUADRANTS_CACHED, 0x4321);
        ram.write_u16(QUADRANTS_CACHED + 2, 0x8765);
        for (offset, value) in [8u8, 7, 6, 5, 4].into_iter().enumerate() {
            ram.write_u8(LINK_DIRECTION_CACHED + offset, value);
        }
        ram.write_u8(TM_COPY, 0x15);
        ram.write_u8(TS_COPY, 0x16);
        ram.write_u8(MAIN_MODULE, 9);
        ram.write_u8(LINK_HEALTH_CURRENT, 0);
        let calls = RefCell::new(Vec::new());

        recover_position_after_drowning(
            &mut ram,
            |_| calls.borrow_mut().push(1),
            |_| calls.borrow_mut().push(2),
            |_| calls.borrow_mut().push(3),
        );

        assert_eq!(ram.read_u16(LINK_Y), 0x1234);
        assert_eq!(ram.read_u16(LINK_X), 0x5678);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS0), 0x100);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS0 + 4), 0x102);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS1), 0x104);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS1 + 4), 0x106);
        assert_eq!(ram.read_u16(CAMERA_SCROLL_HIGH), 0x2fe);
        assert_eq!(ram.read_u16(CAMERA_SCROLL_HIGH + 4), 0x3fe);
        assert_eq!(ram.read_u16(QUADRANT_FULLSIZE_X), 0x4321);
        assert_eq!(ram.read_u16(LINK_QUADRANT_X), 0x8765);
        assert_eq!(*calls.borrow(), [1, 2, 3]);
        assert_eq!(ram.read_u8(MAPBAK_TM), 0x15);
        assert_eq!(ram.read_u8(MAPBAK_TS), 0x16);
        assert_eq!(ram.read_u8(SAVED_MODULE_FOR_MENU), 9);
        assert_eq!(ram.read_u8(MAIN_MODULE), 18);
        assert_eq!(ram.read_u8(SUBMODULE), 1);
        assert_eq!(ram.read_u8(COUNTDOWN_FOR_BLINK), 0);
    }

    #[test]
    fn cached_entrance_restore_rebuilds_scroll_and_map16_derived_fields() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(EXIT_CACHE, 0x23);
        ram.write_u16(EXIT_CACHE + 2, 0x1514);
        ram.write_u16(EXIT_CACHE + 4, 0x222);
        ram.write_u16(EXIT_CACHE + 6, 0x444);
        ram.write_u16(EXIT_CACHE + 8, 0x666);
        ram.write_u16(EXIT_CACHE + 10, 0x888);
        ram.write_u16(EXIT_CACHE + 12, 0x2a);
        ram.write_u16(EXIT_CACHE + 14, 0x790);
        ram.write_u16(EXIT_CACHE + 16, 0x321);
        ram.write_u16(EXIT_CACHE + 18, 0x543);
        for offset in (0..8).step_by(2) {
            ram.write_u16(EXIT_CACHE + 20 + offset, 0x1000 + offset as u16);
            ram.write_u16(EXIT_CACHE + 28 + offset, 0x2000 + offset as u16);
            ram.write_u16(EXIT_CACHE + 42 + offset, 0x3000 + offset as u16);
        }
        ram.write_u8(EXIT_CACHE + 36, 0x51);
        ram.write_u8(EXIT_CACHE + 37, 0x52);
        ram.write_u8(EXIT_CACHE + 38, 0x53);
        ram.write_u8(EXIT_CACHE + 39, 0x54);
        ram.write_u16(DUNGEON_ROOM, 0x130);

        load_cached_entrance_properties(&mut ram);

        assert_eq!(ram.read_u16(OVERWORLD_AREA_INDEX), 0x23);
        assert_eq!(ram.read_u16(TM_COPY), 0x1514);
        assert_eq!(ram.read_u16(BG1VOFS_COPY2), 0x222);
        assert_eq!(ram.read_u16(BG2HOFS_COPY2), 0x444);
        assert_eq!(ram.read_u16(LINK_Y), 0x666);
        assert_eq!(ram.read_u16(LINK_X), 0x888);
        assert_eq!(ram.read_u16(MAP16_LOAD_VAR2), 7);
        assert_eq!(ram.read_u16(MAP16_LOAD_DST_OFFSET), 0);
        assert_eq!(ram.read_u16(CAMERA_SCROLL_HIGH), 0x31f);
        assert_eq!(ram.read_u16(CAMERA_SCROLL_HIGH + 4), 0x541);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS0 + 6), 0x1006);
        assert_eq!(ram.read_u16(LEFT_RIGHT_SCROLL_TARGET_END), 0x2006);
        assert_eq!(ram.read_u16(OVERWORLD_UNK1 + 6), 0x3006);
        assert_eq!(ram.read_u8(BYTE_0AA0), 0x51);
        assert_eq!(ram.read_u8(SPRITE_GRAPHICS_INDEX), 0x54);
    }

    #[test]
    fn new_screen_properties_loads_gfx_before_deriving_bounds() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        load_new_screen_properties(&mut ram, |ram| {
            ram.write_u16(OVERWORLD_AREA_IS_BIG, 0x101);
            ram.write_u16(OVERWORLD_SCREEN_INDEX, 2);
        });
        assert_eq!(ram.read_u16(TILEMAP_LOCATION_CALC_MASK), 0xfff8);
        assert_eq!(ram.read_u8(OVERWORLD_RIGHT_BOTTOM_BOUND), 0xe4);
        assert_eq!(ram.read_u16(OVERWORLD_AREA_IS_BIG), 1);
        assert_eq!(ram.read_u16(OW_SCROLL_VARS0), OVERWORLD_OFFSET_BASE_Y[2]);
        assert_eq!(ram.read_u8(LINK_QUADRANT_X), 0);
        assert_eq!(ram.read_u8(LINK_QUADRANT_Y), 2);
        assert_eq!(ram.read_u8(PLAYER_OAM_X_OFFSET), 0x80);
        assert_eq!(ram.read_u8(LINK_DIRECTION_MASK_A), 0x0f);
        assert_eq!(ram.read_u8(LINK_Z), 0xff);
        assert_eq!(ram.read_u8(LINK_ACTUAL_VELOCITY_Z), 0xff);
    }

    #[test]
    fn gfx_and_screen_size_selects_dark_world_themes_and_big_area_masks() {
        let mut data = assets();
        data.aux_themes[0x45] = 0x2a;
        data.map_is_small[5] = 0;
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(OVERWORLD_SCREEN_INDEX, 0x45);
        ram.write_u8(OVERWORLD_SPRITE_GFX + 0x45, 0x37);
        ram.write_u8(OVERWORLD_AREA_IS_BIG, 0xa5);
        load_gfx_and_screen_size(&mut ram, &data);
        assert_eq!(ram.read_u8(SPRITE_GRAPHICS_INDEX), 0x37);
        assert_eq!(ram.read_u8(AUX_TILE_THEME_INDEX), 0x2a);
        assert_eq!(ram.read_u8(OVERWORLD_AREA_IS_BIG_BACKUP), 0xa5);
        assert_eq!(ram.read_u8(OVERWORLD_AREA_IS_BIG), 0x20);
        assert_eq!(ram.read_u8(OVERWORLD_RIGHT_BOTTOM_BOUND + 1), 3);
        assert_eq!(ram.read_u8(MAIN_TILE_THEME_INDEX), 0x21);
        assert_eq!(ram.read_u8(MISC_SPRITE_GRAPHICS_INDEX), 0x0b);
        assert_eq!(
            ram.read_u16(OVERWORLD_OFFSET_BASE_Y_ADDR),
            OVERWORLD_OFFSET_BASE_Y[5]
        );
        assert_eq!(ram.read_u16(OVERWORLD_OFFSET_MASK_Y), 0x3f0);
        assert_eq!(ram.read_u16(OVERWORLD_OFFSET_MASK_X), 0x7e);
    }

    #[test]
    fn ganon_tower_terminal_jingle_resets_entrance_state() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let data = assets();
        ram.write_u8(SUBSUBMODULE, 12);
        ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 71);
        ram.write_u8(TRIGGER_SPECIAL_ENTRANCE, 5);
        ram.write_u8(NMI_DISABLE_CORE_UPDATES, 5);
        ram.write_u8(FLAG_LINK_IMMOBILIZED, 5);
        ram.write_u8(FLAG_UNK1, 5);
        ram.write_u16(BG1_X_OFFSET, 0x1234);
        animate_entrance_ganons_tower(&mut ram, &data, |_| panic!());
        assert_eq!(ram.read_u8(SUBSUBMODULE), 0);
        assert_eq!(ram.read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER), 0);
        assert_eq!(ram.read_u8(TRIGGER_SPECIAL_ENTRANCE), 0);
        assert_eq!(ram.read_u8(NMI_DISABLE_CORE_UPDATES), 0);
        assert_eq!(ram.read_u16(BG1_X_OFFSET), 0);
        assert_eq!(ram.read_u8(SOUND_EFFECT_2), 27);
        assert_eq!(ram.read_u8(MUSIC_CONTROL), 13);
        assert_eq!(ram.read_u8(SOUND_EFFECT_AMBIENT), 9);
    }

    #[test]
    fn palace_of_darkness_entrance_first_stage_matches_tiles_and_event() {
        let mut data = assets();
        data.map16.resize(0x4000, 0);
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0x3f);
        animate_entrance_palace_of_darkness(&mut ram, &data);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 1);
        assert_eq!(ram.read_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER), 0);
        assert_eq!(ram.read_u8(SAVE_OW_EVENT_INFO + 0x5e), 0x20);
        assert_eq!(ram.read_u16(DUNG_BG2 + 0x1e6), 0xe31);
        assert_eq!(ram.read_u16(DUNG_BG2 + 0x2ea), 0xe27);
        assert_eq!(ram.read_u16(DUNG_BG2 + 0x26a), 0xe26);
        assert_eq!(ram.read_u8(SOUND_EFFECT_1), 12);
        assert_eq!(ram.read_u8(SOUND_EFFECT_2), 7);
        assert_eq!(ram.read_u8(NMI_LOAD_BG_FROM_VRAM), 1);
    }

    #[test]
    fn turtle_rock_entrance_initial_packet_and_shake_match_c() {
        let data = assets();
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 7);
        ram.write_u8(FRAME_COUNTER, 1);
        let mut fixed_color = None;
        animate_entrance_turtle_rock(
            &mut ram,
            &data,
            |_, value| fixed_color = Some(value),
            |_, _, _| panic!("unexpected additive restore"),
            |_, _, _| panic!("unexpected subtractive restore"),
        );
        assert_eq!(fixed_color, Some(0));
        assert_eq!(ram.read_u8(SAVE_OW_EVENT_INFO + 7), 0x20);
        assert_eq!(ram.read_u16(BG1_X_OFFSET), 0xffff);
        assert_eq!(ram.read_u16(BG1_Y_OFFSET), 1);
        assert_eq!(ram.read_u16(UVRAM_DATA), 0x10);
        assert_eq!(ram.read_u16(UVRAM_DATA + 2), 0xfe47);
        assert_eq!(ram.read_u16(UVRAM_DATA + 4), 0x1e3);
        assert_eq!(ram.read_u8(UVRAM_DATA + 6), 0xff);
        assert_eq!(ram.read_u8(SUBSUBMODULE), 1);
        assert_eq!(ram.read_u8(NMI_LOAD_BG_FROM_VRAM), 1);
    }

    #[test]
    fn event_overlay_applies_castle_and_pyramid_tables_exactly() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(OVERWORLD_SCREEN_INDEX, 0);
        overworld_load_event_overlay(&mut ram);
        assert_eq!(ram.read_u16(DUNG_BG2 + (16 * 64 + 11) * 2), 0xe32);
        assert_eq!(ram.read_u16(DUNG_BG2 + (17 * 64 + 12) * 2), 0xe33);
        assert_eq!(ram.read_u16(DUNG_BG2 + (20 * 64 + 13) * 2), 0xe3e);

        ram.write_u16(OVERWORLD_SCREEN_INDEX, 59);
        overworld_load_event_overlay(&mut ram);
        assert_eq!(ram.read_u16(DUNG_BG2 + (7 * 64 + 22) * 2), 0xddf);
        assert_eq!(ram.read_u16(DUNG_BG2 + (13 * 64 + 20) * 2), 0xdf1);
        assert_eq!(ram.read_u16(DUNG_BG2 + (14 * 64 + 28) * 2), 0xe05);
    }

    #[test]
    fn directional_stripe_wrappers_preserve_c_boundary_behavior() {
        let mut data = assets();
        data.map16.resize(0x4000, 0);
        data.map_is_small[0] = 1;
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(MAP16_LOAD_SRC_OFFSET, 0x80);
        ram.write_u16(MAP16_LOAD_VAR2, 5);
        let destination = check_for_newly_loaded_map_areas_north(&mut ram, &data, UVRAM_DATA);
        assert_eq!(destination, UVRAM_DATA);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0);
        assert_eq!(ram.read_u16(MAP16_LOAD_VAR2), 4);

        // Once already at the northern boundary the C helper returns without
        // touching either the destination or scroll state.
        let destination = check_for_newly_loaded_map_areas_north(&mut ram, &data, UVRAM_DATA + 6);
        assert_eq!(destination, UVRAM_DATA + 6);
        assert_eq!(ram.read_u16(MAP16_LOAD_SRC_OFFSET), 0);
        assert_eq!(ram.read_u16(MAP16_LOAD_VAR2), 4);
    }
}
