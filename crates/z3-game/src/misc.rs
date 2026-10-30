//! Native translation of `src/misc.c`.
//!
//! All addresses are offsets in the original `g_ram` WRAM image.  Operations
//! owned by other C translation units are exposed through [`MiscHooks`].
use crate::inventory_receipt::ReceiptItem;
use crate::ram::Ram;
use crate::sound::{SoundPort, SoundRequest};

const MAIN: usize = 0x10;
const SUB: usize = 0x11;
const INIDISP: usize = 0x13;
const NMI_LOAD_BG: usize = 0x14;
const UPDATE_HUD: usize = 0x16;
const NMI_COPY_PACKETS: usize = 0x18;
const FRAME: usize = 0x1a;
const INDOORS: usize = 0x1b;
const TM: usize = 0x1c;
const TS: usize = 0x1d;
const LINK_Y: usize = 0x20;
const LINK_X: usize = 0x22;
const LINK_DIR_LAST: usize = 0x26;
const LINK_ANIM_STEPS: usize = 0x2e;
const LINK_FACING: usize = 0x2f;
const LINK_Y_VEL: usize = 0x30;
const LINK_X_VEL: usize = 0x31;
const BUTTON_MASK: usize = 0x3a;
const SPIN_DELAY: usize = 0x3d;
const CAPE_MODE: usize = 0x55;
const IS_BUNNY: usize = 0x56;
const PLAYER_STATE: usize = 0x5d;
const SPEED: usize = 0x5e;
const LINK_DIR: usize = 0x67;
const OVERLAY_INDEX: usize = 0x8c;
const MOSAIC: usize = 0x95;
const HDMAEN: usize = 0x9b;
const DUNGEON_ROOM: usize = 0xa0;
const SUBSUB: usize = 0xb0;
const BG2_HOFS: usize = 0xe2;
const BG1_VOFS: usize = 0xe6;
const VIRQ_TRIGGER: usize = 0xff;
const LINK_DMA_GRAPHICS: usize = 0x100;
const LINK_DMA_VAR1: usize = 0x102;
const LINK_DMA_VAR2: usize = 0x104;
const LINK_DMA_VAR3: usize = 0x107;
const LINK_DMA_VAR4: usize = 0x108;
const LINK_DMA_VAR5: usize = 0x109;
const DEATH_VAR5: usize = 0x10a;
const SAVED_MODULE: usize = 0x10c;
const BG1_X: usize = 0x11a;
const BG1_Y: usize = 0x11c;
const MUSIC: usize = 0x12c;
const SFX_AMBIENT: usize = 0x12d;
const SFX1: usize = 0x12e;
const SFX2: usize = 0x12f;
const MAP_STATE: usize = 0x200;
const BYTE_0223: usize = 0x223;
const PUSHED_BLOCK_INDEX: usize = 0x2c3;
const WORD_02CD: usize = 0x2cd;
const BYTE_02D4: usize = 0x2d4;
const BYTE_02D7: usize = 0x2d7;
const RECEIVE_ITEM: usize = 0x2d8;
const ITEM_POSE: usize = 0x2da;
const BUNNY_TIMER: usize = 0x2e2;
const IMMOBILIZED: usize = 0x2e4;
const RECEIPT_METHOD: usize = 0x2e9;
const TAGALONG_VAR5: usize = 0x2f9;
const LINK_STATE_BITS: usize = 0x308;
const PICK_THROW: usize = 0x309;
const TORCH_SELECTOR: usize = 0x333;
const BYTE_0379: usize = 0x379;
const DISABLE_DAMAGE: usize = 0x37b;
const ANC_ARR3: usize = 0x39f;
const ANC_ARR1: usize = 0x3a4;
const ANC_AUX_TIMER: usize = 0x3b1;
const HOLD_SWORD: usize = 0x3ef;
const DUNG_UNK6: usize = 0x3f4;
const BYTE_03FD: usize = 0x3fd;
const DUNG_SAVE_BITS: usize = 0x402;
const PALACE_X2: usize = 0x40c;
const OW_TRANS_DIR: usize = 0x410;
const NUM_LIT_TORCHES: usize = 0x45a;
const TORCHES_START: usize = 0x478;
const DEATH_VAR4: usize = 0x4aa;
const TORCH_TIMERS: usize = 0x4f0;
const OBJECT_POS: usize = 0x520;
const OBJECT_TILEMAP_POS: usize = 0x540;
const DUNG_BG_H: usize = 0x62c;
const DUNG_BG_V: usize = 0x62e;
const NMI_DISABLE: usize = 0x710;
const EXTENDED_OAM: usize = 0xa00;
const BYTEWISE_OAM: usize = 0xa20;
const DMA_SOURCE_6: usize = 0xac0;
const DMA_SOURCE_11: usize = 0xac2;
const DMA_SOURCE_7: usize = 0xac4;
const DMA_SOURCE_12: usize = 0xac6;
const DMA_SOURCE_8: usize = 0xac8;
const DMA_SOURCE_13: usize = 0xaca;
const DMA_SOURCE_3: usize = 0xacc;
const DMA_SOURCE_0: usize = 0xace;
const DMA_SOURCE_4: usize = 0xad0;
const DMA_SOURCE_1: usize = 0xad2;
const DMA_SOURCE_5: usize = 0xad4;
const DMA_SOURCE_2: usize = 0xad6;
const DMA_SOURCE_10: usize = 0xad8;
const DMA_SOURCE_15: usize = 0xada;
const ANIMATED_TILE_SRC: usize = 0xadc;
const DMA_SOURCE_9: usize = 0xae0;
const DMA_SOURCE_14: usize = 0xae2;
const DMA_VAR6: usize = 0xae8;
const DMA_VAR7: usize = 0xaea;
const DMA_SOURCE_16: usize = 0xaec;
const DMA_SOURCE_18: usize = 0xaee;
const DMA_SOURCE_17: usize = 0xaf0;
const DMA_SOURCE_19: usize = 0xaf2;
const TRAVEL_BIRD: usize = 0xaf4;
const DMA_SOURCE_20: usize = 0xaf6;
const DMA_SOURCE_21: usize = 0xaf8;
const ANC_ARR4: usize = 0xbf0;
const ANC_TYPE: usize = 0xc4a;
const ANC_STEP: usize = 0xc54;
const ANC_ITEM: usize = 0xc5e;
const ANC_TIMER: usize = 0xc68;
const SFX_LATCH: usize = 0xcf8;
const RNG_STATE: usize = 0xfa1;
const VRAM_UPLOAD: usize = 0x1100;
const MESSAGING_MODULE: usize = 0x1cd8;
const DIALOGUE: usize = 0x1cf0;
const OVERWORLD_TILEATTR: usize = 0x2000;
const LIGHTS_OUT: usize = 0xc005;
const PALETTE_COUNTDOWN: usize = 0xc007;
const PALETTE_DIRECTION: usize = 0xc009;
const BG_ANIM_COUNTDOWN: usize = 0xc00d;
const BG_ANIM_OFFSET: usize = 0xc00f;
const MOSAIC_LEVEL: usize = 0xc011;
const DMA_ANIM_COUNTDOWN: usize = 0xc013;
const DMA_ANIM_INDEX: usize = 0xc015;
const FIXED_COLOR: usize = 0xc017;
const SPR_GFX_SUBSETS: usize = 0xc2fc;
const MAIN_PALETTE: usize = 0xc500;
const SAVE_OW_INFO: usize = 0xf280;
const MUSHROOM: usize = 0xf344;
const MIRROR: usize = 0xf353;
const MOON_PEARL: usize = 0xf357;
const SWORD: usize = 0xf359;
#[cfg(test)]
const BOTTLES: usize = 0xf35c;
const ABILITY_FLAGS: usize = 0xf379;
const PROGRESS: usize = 0xf3c5;
const MAP_ICONS: usize = 0xf3c7;
const STARTING_POINT: usize = 0xf3c8;
const DARK_WORLD: usize = 0xf3ca;
const TORCH_DATA: usize = 0xfb40;
const TILE_ATTRIBUTES: usize = 0xfe00;
const HDMA_DYNAMIC: usize = 0x1dba0;

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
const PAN_TABLE: [u8; 3] = [0, 0x80, 0x40];
const TORCH_PANS: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
const LIT_TORCH_COLORS: [u8; 4] = [31, 8, 4, 0];
const DUNGEON_DEFAULT_ATTR: [u8; 384] = [
    1, 1, 1, 0, 2, 1, 2, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 1, 0, 0, 2, 0, 0, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0,
    0, 0, 0, 0x2a, 1, 0x20, 1, 1, 4, 1, 1, 0x18, 1, 2, 0x1c, 1, 0x28, 0x28, 0x2a, 0x2a, 1, 2, 1, 1,
    4, 0, 0, 0, 0x28, 1, 0xa, 0, 1, 1, 0xc, 0xc, 2, 2, 2, 2, 0x28, 0x2a, 0x20, 0x20, 0x20, 2, 8, 0,
    4, 4, 1, 1, 1, 2, 2, 2, 0, 0, 0x20, 0x20, 0, 2, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x18, 0x10, 0x10, 1, 1, 1, 1, 1, 4, 4, 4, 4, 4, 4, 1, 2, 2,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x62, 0x62, 0, 0, 0x24, 0x24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x62, 0x62, 0x27, 2, 2, 2,
    0x27, 0x27, 1, 0, 0, 0, 0, 0x24, 0, 0, 0, 0, 0x27, 0x27, 0x27, 0x27, 0x27, 0x10, 2, 1, 0, 0, 0,
    0x24, 0, 0, 0, 0, 0x27, 2, 2, 2, 0x27, 0x27, 0x27, 0x27, 2, 2, 2, 0x24, 0, 0, 0, 0, 0x27, 0x27,
    0x27, 0x27, 0x27, 0x20, 2, 2, 1, 2, 2, 0x23, 2, 0, 0, 0, 0x27, 0x27, 0x27, 0x27, 0x27, 0x20, 2,
    0x27, 2, 0x54, 0, 0, 0x27, 2, 2, 2, 0x27, 0x27, 0x27, 0x27, 0x27, 0x27, 2, 0x27, 2, 0x54, 0, 0,
    0x27, 2, 2, 2, 0x27, 0x27, 0, 0x27, 0x60, 0x60, 1, 1, 1, 1, 2, 2, 0xd, 0, 0, 0x4b, 0x67, 0x67,
    0x67, 0x67, 0x66, 0x66, 0x66, 0x66, 0, 0, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x27, 0x63, 0x27,
    0x55, 0x55, 1, 0x44, 0, 1, 0x20, 2, 2, 0x1c, 0x3a, 0x3b, 0, 0x27, 0x63, 0x27, 0x53, 0x53, 1,
    0x44, 1, 0xd, 0, 0, 0, 9, 9, 9, 9,
];
const LINK_DMA_SOURCES1: [u16; 303] = [
    0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0x8000,
    0x8000, 0x8000, 0x8000, 0x8000, 0x9440, 0x8080, 0x8080, 0x8080, 0x9400, 0x8040, 0x80c0, 0x80c0,
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8080, 0x8080, 0x8080,
    0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0xa8c0, 0xa900, 0x8000, 0xa8c0, 0xa900,
    0x9100, 0x8080, 0x8080, 0x90c0, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x9a00,
    0x9140, 0x9180, 0x8000, 0x9500, 0x9480, 0x94c0, 0x94c0, 0x9ae0, 0x8080, 0x8080, 0x9a60, 0x80c0,
    0x80c0, 0x9aa0, 0x8000, 0x8000, 0x9aa0, 0x8000, 0x8000, 0x8080, 0x8080, 0x8100, 0x8100, 0x85c0,
    0x8000, 0x8000, 0x85c0, 0x8000, 0x8000, 0xadc0, 0xadc0, 0xadc0, 0xadc0, 0xadc0, 0xad40, 0xad40,
    0xad40, 0xad40, 0xad40, 0xad80, 0xad80, 0xad80, 0xad80, 0xad80, 0xad80, 0x8040, 0x9400, 0x8040,
    0x8000, 0x8080, 0x8080, 0x9440, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8040, 0x8040, 0x8000,
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0xc440, 0x8140, 0x8140, 0xca40, 0x8000, 0x8000, 0x8000,
    0x8000, 0x8000, 0x8000, 0x8040, 0x85c0, 0x8040, 0x85c0, 0x8100, 0x80c0, 0x91c0, 0x8080, 0x8080,
    0x8040, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8080, 0x9100, 0xa0c0, 0xa100, 0xa100,
    0xa1c0, 0xa400, 0xa440, 0xa1c0, 0xa400, 0xa440, 0x8080, 0xc480, 0x8080, 0x8040, 0x8040, 0xca80,
    0xca80, 0xca00, 0xc400, 0xca00, 0xc400, 0x81c0, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080,
    0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0xa8c0, 0xa900,
    0x8000, 0x8000, 0xa8c0, 0xa900, 0x8000, 0xa8c0, 0xa900, 0x8000, 0x8000, 0xa8c0, 0xa900, 0x8040,
    0x8040, 0x8040, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000,
    0xd080, 0x8080, 0x90c0, 0xd000, 0x9080, 0xd040, 0x9080, 0xd040, 0xd080, 0xd080, 0xd080, 0xd080,
    0xd080, 0xd000, 0xd000, 0xd000, 0xd000, 0xd000, 0xd040, 0xd040, 0xd040, 0xd040, 0xd040, 0xd040,
    0x8040, 0xd000, 0x85c0, 0x85c0, 0x85c0, 0xdc40, 0xdc40, 0xdc40, 0x85c0, 0x85c0, 0x85c0, 0xdc40,
    0xdc40, 0xdc40, 0xe1c0, 0xd000, 0x8000, 0xe400, 0xe400, 0xe440, 0x90c0, 0x90c0, 0xd000, 0x8000,
    0x8000, 0xd040, 0x8000, 0x8000, 0xd040, 0xe400, 0xe400, 0xe400, 0x9080, 0xa5c0, 0xac40, 0xe480,
    0x8180, 0x90c0, 0x80c0, 0xe180, 0xd000, 0xe4c0, 0xe4c0, 0xe840, 0xe840, 0xe840, 0xe540, 0xe540,
    0xe540, 0xe900, 0xe900, 0xe900, 0xe900, 0x8080, 0x8080, 0x8000, 0xa9c0, 0x8080, 0x8140, 0x91c0,
    0x8040, 0xa800, 0xa840,
];
const LINK_DMA_SOURCES2: [u16; 303] = [
    0x8840, 0x8800, 0x8580, 0x8800, 0x8580, 0x84c0, 0x8500, 0x8540, 0x8500, 0x8540, 0x8400, 0x8440,
    0x8480, 0x8400, 0x8440, 0x8480, 0x9640, 0x8c40, 0x8c80, 0xad00, 0x9600, 0x8980, 0x8c00, 0xacc0,
    0x8880, 0x88c0, 0x8900, 0x8940, 0x8880, 0x88c0, 0x8900, 0x8940, 0xb0c0, 0xb100, 0xb140, 0xb100,
    0xb140, 0xb000, 0xb040, 0xb080, 0xec80, 0xecc0, 0xb180, 0xd440, 0xb1c0, 0xb180, 0xd440, 0xb1c0,
    0x8c80, 0xad00, 0x95c0, 0x99c0, 0xb440, 0x9580, 0xb480, 0xb4c0, 0x9580, 0xb480, 0xb4c0, 0x9c20,
    0x8000, 0x8000, 0x8000, 0x9700, 0x9680, 0x96c0, 0x96c0, 0x9ce0, 0x8c80, 0xb540, 0x9c60, 0xb580,
    0x8c00, 0x9ca0, 0x8900, 0xb500, 0x9ca0, 0x8900, 0xb500, 0x8c40, 0xec40, 0x8c00, 0xec00, 0x8dc0,
    0x9540, 0x89c0, 0x8dc0, 0x9540, 0x89c0, 0xb940, 0xb980, 0xb9c0, 0xb980, 0xb9c0, 0xb5c0, 0xb800,
    0xb840, 0xb800, 0xb840, 0xb880, 0xb8c0, 0xb900, 0xb880, 0xb8c0, 0xb900, 0x8980, 0x9600, 0xbcc0,
    0x8400, 0xbc80, 0x8c40, 0x9640, 0xa040, 0xa080, 0xa000, 0xbc40, 0xbd40, 0x8500, 0xbd00, 0xbd80,
    0xbd80, 0x88c0, 0x8900, 0xe9c0, 0x8900, 0xc640, 0xc040, 0xc000, 0xcc40, 0x8940, 0x88c0, 0x8900,
    0xe9c0, 0x8900, 0x8940, 0x8d40, 0x8d80, 0x8d40, 0x8d80, 0xbd00, 0xb000, 0xb000, 0xa480, 0xa480,
    0xa480, 0xa480, 0xac00, 0xac00, 0xac00, 0xac00, 0xa140, 0xa180, 0xa180, 0xa4c0, 0xa4c0, 0xa500,
    0x9d40, 0x9d80, 0x9dc0, 0x9d40, 0x9d80, 0x9dc0, 0x8d00, 0xc680, 0xc180, 0xc140, 0x8c00, 0xcc80,
    0xcc80, 0xcc00, 0xc600, 0xcc00, 0xc600, 0xbd00, 0x8580, 0x8800, 0xc9c0, 0xccc0, 0xcdc0, 0xcd00,
    0xcd40, 0xcd80, 0x8500, 0x8540, 0xc940, 0xc980, 0x8540, 0xc940, 0xc980, 0x8440, 0x8480, 0xc1c0,
    0xc900, 0xc580, 0xc5c0, 0xc8c0, 0x8440, 0x8480, 0xc1c0, 0xc900, 0xc580, 0xc5c0, 0xc8c0, 0xbd00,
    0xacc0, 0xc040, 0xd540, 0xd580, 0xd4c0, 0xd500, 0xd4c0, 0xd500, 0xd440, 0xd480, 0xd440, 0xd480,
    0xd1c0, 0xd400, 0xd100, 0xd100, 0xd140, 0xd180, 0xd140, 0xd180, 0xb0c0, 0xb100, 0xb140, 0xb100,
    0xb140, 0xdd40, 0xdd80, 0xddc0, 0xdd80, 0xddc0, 0xdc80, 0xdcc0, 0xdd00, 0xdc80, 0xdcc0, 0xdd00,
    0xd100, 0xd100, 0xe000, 0xe040, 0xe080, 0xe0c0, 0xe100, 0xe140, 0xe000, 0xe040, 0xe080, 0xe0c0,
    0xe100, 0xe140, 0x8000, 0xd0c0, 0x8000, 0xb940, 0xb980, 0xb940, 0xdd40, 0xdd80, 0xdd40, 0xdc80,
    0xdcc0, 0xc0c0, 0xdc80, 0xdcc0, 0xc0c0, 0xb9c0, 0xb980, 0xb9c0, 0xa560, 0xa5a0, 0xac80, 0xed00,
    0x8000, 0x8cc0, 0xbd00, 0xe380, 0xbdc0, 0xe500, 0xe500, 0xe880, 0xe8c0, 0xe8c0, 0xe800, 0xe5c0,
    0xe5c0, 0xe940, 0xe980, 0xe940, 0xe980, 0xbd40, 0x8c80, 0xa080, 0x8000, 0xa980, 0xbd00, 0xbdc0,
    0xb400, 0xa880, 0xedc0,
];
const LINK_DMA_SOURCES3: [u16; 27] = [
    0x9a40, 0x9e00, 0x9d20, 0x9f20, 0x9b20, 0xbc20, 0xbc20, 0xbe20, 0xbe20, 0xbe00, 0xbe00, 0xbe00,
    0xbe00, 0xa540, 0xa540, 0xa540, 0xa540, 0xbc00, 0xbc00, 0xbc00, 0xbc00, 0xa740, 0xa740, 0xa740,
    0xa740, 0xe780, 0xe780,
];
const LINK_DMA_SOURCES4: [u16; 8] = [
    0x9000, 0x9020, 0x9060, 0x91e0, 0x90a0, 0x90c0, 0x9100, 0x9140,
];
const LINK_DMA_SOURCES5: [u16; 3] = [0x9300, 0x9340, 0x9380];
const LINK_DMA_SOURCES6: [u16; 128] = [
    0x9480, 0x94c0, 0x94e0, 0x95c0, 0x9500, 0x9520, 0x9540, 0x9480, 0x9640, 0x9680, 0x96a0, 0x9780,
    0x96c0, 0x96e0, 0x9700, 0x9480, 0x9800, 0x9840, 0x98a0, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9ac0, 0x9b00, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9bc0, 0x9c00, 0x9c40, 0x9c80,
    0x9cc0, 0x9d00, 0x9d40, 0x9480, 0x9f40, 0x9f80, 0x9fc0, 0x9fe0, 0xa000, 0x9480, 0x9480, 0x9480,
    0xa100, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x98c0, 0x9900, 0x99c0, 0x99e0, 0x9a00, 0x9a20, 0x9a40, 0x9a60,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9a80, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
];
const LINK_DMA_SOURCES7: [u16; 16] = [
    0xe0, 0xe0, 0x60, 0x80, 0x1c0, 0xe0, 0x40, 0, 0x80, 0, 0x40, 0, 0, 0, 0, 0,
];
const LINK_DMA_CTRS0: [u16; 6] = [14, 4, 6, 16, 6, 8];
const LINK_DMA_SOURCES9: [u16; 15] = [
    0, 0x20, 0x40, 0, 0x20, 0x40, 0, 0x40, 0x80, 0, 0x40, 0x80, 0xb340, 0xb400, 0xb4c0,
];
const LINK_DMA_SOURCES8: [u16; 4] = [0xa480, 0xa4c0, 0xa500, 0xa540];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MiscCall {
    HudSearchEquipped,
    HudRebuild,
    HudUpdateEquipped,
    ModulePreDungeon,
    LoadOwMusicIfNeeded,
    InitializeMirrorHdma,
    PaletteWhiteFilter,
    OverworldLoadGfx,
    TerminateInteractives(u8),
    LinkResetProperties,
    MirrorBuildWaving,
    MirrorBuildDewaving,
    HdmaSetup {
        addr6: u32,
        addr7: u32,
        transfer_unit: u8,
        reg6: u8,
        reg7: u8,
        indirect_bank: u8,
    },
    ReloadSheets,
    HudRebuildIndoor,
    RenderText,
    ResetAncillaCutscene,
    OverworldSetSongList,
    SpriteMain,
    LinkOamMain,
    DungeonResetTorchAndPlayer,
    HudRefillLogic,
    LinkAnimateVictorySpin,
    AddVictorySpin,
    LinkMain,
    DungeonPrepExitSpotlight,
    SpotlightConfigure,
    PaletteRevertSwap,
    OpenSpotlightNext2,
    EnableForceBlank,
    EraseTileMaps,
    LoadDefaultGraphics,
    SpriteLoadGraphicsProperties,
    DecompressSword,
    DecompressShield,
    LinkInitialize,
    LoadFollowerGraphics,
    DungeonLoadPalettes,
    PaletteUpdateGloves,
    RefreshIcon,
    CapePoof(u8, u8),
    MsCutscene(u8, u8),
    PaletteLoadShield,
    PaletteLoadSword,
    DecodeAnimatedTile(u8),
    SaveDungeonKeys,
    FlagRoomQuadrants,
    SpriteResetAll,
    ResetAfterDeath(u8),
    DeathFunc15(bool),
}

pub trait MiscHooks {
    fn call(&mut self, ram: &mut Ram<'_>, call: MiscCall);
    fn route_main_module(&mut self, ram: &mut Ram<'_>, module: u8);
    fn load_song_bank(&mut self, ram: &mut Ram<'_>, bank: u8);
    fn predefined_tile_word(&self, word_index: usize) -> u16;
    fn refill_magic(&mut self, ram: &mut Ram<'_>) -> bool;
    fn refill_health(&mut self, ram: &mut Ram<'_>) -> bool;
    fn add_ancilla(&mut self, ram: &mut Ram<'_>, kind: u8, arg: u8) -> i32;
    fn set_ancilla_xy(&mut self, ram: &mut Ram<'_>, ancilla: usize, x: u16, y: u16);
    fn sprite_x(&self, ram: &Ram<'_>, sprite: usize) -> u16;
    fn dungeon_item_tile_action(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8;
    fn overworld_item_tile_action(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8;
}

#[must_use]
pub const fn src_ptr(src: u16) -> usize {
    (src >> 1) as usize
}

#[must_use]
pub fn get_random_number(r: &mut Ram<'_>) -> u8 {
    let mut t = r.read_u8(RNG_STATE).wrapping_add(r.read_u8(FRAME));
    t = if t & 1 != 0 { t >> 1 } else { (t >> 1) ^ 0xb8 };
    r.write_u8(RNG_STATE, t);
    t
}

#[must_use]
pub fn calculate_sfx_pan(r: &Ram<'_>, x: u16) -> u8 {
    let x = x.wrapping_sub(r.read_u16(BG2_HOFS).wrapping_add(80));
    let o = if x < 80 {
        0
    } else {
        1 + usize::from((x as i16) >= 0)
    };
    PAN_TABLE[o]
}

#[must_use]
pub fn link_calculate_sfx_pan(r: &Ram<'_>) -> u8 {
    calculate_sfx_pan(r, r.read_u16(LINK_X))
}

#[must_use]
pub fn sprite_calculate_sfx_pan<H: MiscHooks>(r: &Ram<'_>, hooks: &H, k: usize) -> u8 {
    calculate_sfx_pan(r, hooks.sprite_x(r, k))
}

#[must_use]
pub fn calculate_sfx_pan_arbitrary(r: &Ram<'_>, a: u8) -> u8 {
    TORCH_PANS[usize::from(a.wrapping_sub(r.read_u16(BG2_HOFS) as u8) >> 5 & 7)]
}

fn play_sfx_set_pan(r: &mut Ram<'_>, a: u8) -> u8 {
    r.write_u8(SFX_LATCH, a);
    a | link_calculate_sfx_pan(r)
}

pub fn ancilla_sfx2_near(r: &mut Ram<'_>, a: u8) -> u8 {
    let v = play_sfx_set_pan(r, a);
    SoundPort::EffectsA.overwrite(r, SoundRequest::from_raw(v));
    v
}

pub fn ancilla_sfx3_near(r: &mut Ram<'_>, a: u8) {
    let v = play_sfx_set_pan(r, a);
    SoundPort::EffectsB.overwrite(r, SoundRequest::from_raw(v));
}

pub fn sprite_sfx1_with_pan<H: MiscHooks>(r: &mut Ram<'_>, hooks: &H, k: usize, a: u8) {
    SoundPort::Ambient.queue_if_idle_with(r, |ram| {
        SoundRequest::at_world_x(ram, a, hooks.sprite_x(ram, k))
    });
}
pub fn sprite_sfx2_with_pan<H: MiscHooks>(r: &mut Ram<'_>, hooks: &H, k: usize, a: u8) {
    SoundPort::EffectsA.queue_if_idle_with(r, |ram| {
        SoundRequest::at_world_x(ram, a, hooks.sprite_x(ram, k))
    });
}
pub fn sprite_sfx3_with_pan<H: MiscHooks>(r: &mut Ram<'_>, hooks: &H, k: usize, a: u8) {
    SoundPort::EffectsB.queue_if_idle_with(r, |ram| {
        SoundRequest::at_world_x(ram, a, hooks.sprite_x(ram, k))
    });
}

pub fn item_receipt_give_bottled_item(r: &mut Ram<'_>, item: u8) {
    crate::inventory_receipt::give_bottled_receipt(r, item);
}

pub fn main_show_text_message(r: &mut Ram<'_>) {
    if r.read_u8(MAIN) != 14 {
        r.write_u8(BYTE_0223, 0);
        r.write_u8(MESSAGING_MODULE, 0);
        r.write_u8(SUB, 2);
        r.write_u8(SAVED_MODULE, r.read_u8(MAIN));
        r.write_u8(MAIN, 14);
    }
}

pub fn handle_item_tile_action_overworld<H: MiscHooks>(
    r: &mut Ram<'_>,
    hooks: &mut H,
    x: u16,
    y: u16,
) -> u8 {
    if r.read_u8(INDOORS) != 0 {
        hooks.dungeon_item_tile_action(r, x, y)
    } else {
        hooks.overworld_item_tile_action(r, x, y)
    }
}

pub fn dungeon_prep_overlay_dma_watergate(
    r: &mut Ram<'_>,
    mut dst: usize,
    mut r8: u16,
    r6: u16,
    loops: usize,
) -> usize {
    for _ in 0..loops {
        let x = usize::from(r8 >> 1);
        let values = [
            (r8 & 0x40) << 4 | (r8 & 0x303f) >> 1 | (r8 & 0xf80) >> 2,
            r6,
            r.read_u16(OVERWORLD_TILEATTR + x * 2),
            r.read_u16(OVERWORLD_TILEATTR + (x + if r6 & 1 == 0 { 1 } else { 64 }) * 2),
            r.read_u16(OVERWORLD_TILEATTR + (x + if r6 & 1 == 0 { 2 } else { 128 }) * 2),
            r.read_u16(OVERWORLD_TILEATTR + (x + if r6 & 1 == 0 { 3 } else { 192 }) * 2),
        ];
        for (i, value) in values.into_iter().enumerate() {
            r.write_u16(VRAM_UPLOAD + (dst + i) * 2, value);
        }
        r8 = r8.wrapping_add(if r6 & 1 == 0 { 128 } else { 2 });
        dst += 6;
    }
    r.write_u16(VRAM_UPLOAD + dst * 2, 0xffff);
    dst
}

pub fn dungeon_prep_overlay_dma_next_prep(r: &mut Ram<'_>, dst: usize, r8: u16) -> usize {
    dungeon_prep_overlay_dma_watergate(r, dst, r8, 0x880 + u16::from((r8 & 0x3f) >= 0x3a), 4)
}

pub fn room_draw_adjust_torch_lighting_change<H: MiscHooks>(
    r: &mut Ram<'_>,
    hooks: &H,
    x: u16,
    y: u16,
    r8: u16,
) {
    let src = src_ptr(y);
    let x = usize::from(x >> 1);
    for (off, si) in [(0, 0), (64, 1), (1, 2), (65, 3)] {
        r.write_u16(
            OVERWORLD_TILEATTR + (x + off) * 2,
            hooks.predefined_tile_word(src + si),
        );
    }
    dungeon_prep_overlay_dma_next_prep(r, 0, r8);
}

pub fn nmi_prepare_sprites(r: &mut Ram<'_>) {
    for i in 0..32 {
        let mut v = 0;
        for j in 0..4 {
            v |= r.read_u8(BYTEWISE_OAM + 4 * i + j) << (2 * j);
        }
        r.write_u8(EXTENDED_OAM + i, v);
    }
    let set_pair = |r: &mut Ram<'_>, a: usize, b: usize, value: u16, delta: u16| {
        r.write_u16(a, value);
        r.write_u16(b, value.wrapping_add(delta));
    };
    set_pair(
        r,
        DMA_SOURCE_3,
        DMA_SOURCE_0,
        LINK_DMA_SOURCES1[usize::from(r.read_u16(LINK_DMA_GRAPHICS) >> 1)],
        0x200,
    );
    set_pair(
        r,
        DMA_SOURCE_4,
        DMA_SOURCE_1,
        LINK_DMA_SOURCES2[usize::from(r.read_u16(LINK_DMA_GRAPHICS) >> 1)],
        0x200,
    );
    r.write_u16(
        DMA_SOURCE_5,
        LINK_DMA_SOURCES3[usize::from(r.read_u16(LINK_DMA_VAR1) >> 1)],
    );
    r.write_u16(
        DMA_SOURCE_2,
        LINK_DMA_SOURCES3[usize::from(r.read_u16(LINK_DMA_VAR2) >> 1)],
    );
    set_pair(
        r,
        DMA_SOURCE_6,
        DMA_SOURCE_11,
        LINK_DMA_SOURCES4[usize::from(r.read_u8(LINK_DMA_VAR3) >> 1)],
        0x180,
    );
    let d7 = if r.read_u8(LINK_DMA_VAR4) == 0x8b {
        0xe099
    } else {
        LINK_DMA_SOURCES5[usize::from(r.read_u8(LINK_DMA_VAR4) >> 1)]
    };
    set_pair(r, DMA_SOURCE_7, DMA_SOURCE_12, d7, 0xc0);
    let v5 = r.read_u8(LINK_DMA_VAR5);
    let d8 = LINK_DMA_SOURCES6[usize::from(v5)];
    set_pair(
        r,
        DMA_SOURCE_8,
        DMA_SOURCE_13,
        d8,
        LINK_DMA_SOURCES7[usize::from((v5 & 0xf8) >> 3)],
    );
    set_pair(
        r,
        DMA_SOURCE_10,
        DMA_SOURCE_15,
        LINK_DMA_SOURCES8[usize::from(r.read_u8(PUSHED_BLOCK_INDEX) & 3)],
        0x100,
    );

    let bg_count = r.read_u16(BG_ANIM_COUNTDOWN).wrapping_sub(1);
    r.write_u16(BG_ANIM_COUNTDOWN, bg_count);
    if bg_count == 0 {
        let overlay = r.read_u16(OVERLAY_INDEX) as u8;
        r.write_u16(
            BG_ANIM_COUNTDOWN,
            if matches!(overlay, 0xb5 | 0xbc) {
                0x17
            } else {
                9
            },
        );
        let mut t = r.read_u16(BG_ANIM_OFFSET).wrapping_add(0x400);
        if t == 0xc00 {
            t = 0;
        }
        r.write_u16(BG_ANIM_OFFSET, t);
        r.write_u16(ANIMATED_TILE_SRC, 0xa680u16.wrapping_add(t));
    }
    let dma_count = r.read_u16(DMA_ANIM_COUNTDOWN).wrapping_sub(1);
    r.write_u16(DMA_ANIM_COUNTDOWN, dma_count);
    if dma_count == 0 {
        let mut t = r.read_u16(DMA_ANIM_INDEX).wrapping_add(2);
        if t == 12 {
            t = 0;
        }
        r.write_u16(DMA_ANIM_INDEX, t);
        let i = usize::from(t >> 1);
        r.write_u16(DMA_ANIM_COUNTDOWN, LINK_DMA_CTRS0[i]);
        let d9 = LINK_DMA_SOURCES9[i].wrapping_add(0xb280);
        set_pair(r, DMA_SOURCE_9, DMA_SOURCE_14, d9, 0x60);
    }
    let d16 = 0xb940u16.wrapping_add(r.read_u16(DMA_VAR6).wrapping_mul(2));
    set_pair(r, DMA_SOURCE_16, DMA_SOURCE_18, d16, 0x200);
    let d17 = 0xb940u16.wrapping_add(r.read_u16(DMA_VAR7).wrapping_mul(2));
    set_pair(r, DMA_SOURCE_17, DMA_SOURCE_19, d17, 0x200);
    let d20 = 0xb540u16.wrapping_add(r.read_u16(TRAVEL_BIRD).wrapping_mul(2));
    set_pair(r, DMA_SOURCE_20, DMA_SOURCE_21, d20, 0x200);
}

pub fn sound_load_intro_song_bank<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.load_song_bank(r, 0);
}
pub fn load_overworld_songs<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.load_song_bank(r, 0);
}
pub fn load_dungeon_songs<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.load_song_bank(r, 1);
}
pub fn load_credits_songs<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.load_song_bank(r, 2);
}

pub fn dungeon_light_torch<H: MiscHooks>(r: &mut Ram<'_>, hooks: &H) {
    let selector = r.read_u8(TORCH_SELECTOR);
    if selector & 0xf0 != 0xc0 {
        r.write_u8(TORCH_SELECTOR, 0);
        return;
    }
    let timer = if r.read_u16(DUNGEON_ROOM) as u8 == 0 {
        0x80
    } else {
        0xc0
    };
    let i = usize::from(selector & 0xf) + usize::from(r.read_u16(TORCHES_START) >> 1);
    let opos = usize::from(r.read_u16(OBJECT_POS + i * 2));
    let mut tilemap = r.read_u16(OBJECT_TILEMAP_POS + i * 2);
    if tilemap & 0x8000 != 0 {
        return;
    }
    tilemap |= 0x8000;
    r.write_u16(OBJECT_TILEMAP_POS + i * 2, tilemap);
    // This branch is unreachable for the two timer values selected above, but
    // is retained because it is present in the C routine.
    if timer == 0 {
        r.write_u16(TORCH_DATA + opos * 2, tilemap);
    }
    let x = tilemap & 0x3fff;
    room_draw_adjust_torch_lighting_change(r, hooks, x, 0xeca, x);
    let pan = calculate_sfx_pan_arbitrary(r, ((x & 0x7f) * 2) as u8);
    r.write_u8(SFX1, 42 | pan);
    r.write_u8(NMI_COPY_PACKETS, 1);
    if r.read_u8(LIGHTS_OUT) != 0 {
        let old = r.read_u8(NUM_LIT_TORCHES);
        r.write_u8(NUM_LIT_TORCHES, old.wrapping_add(1));
        if old < 3 {
            r.write_u8(TS, 0);
            r.write_u8(
                FIXED_COLOR,
                LIT_TORCH_COLORS[usize::from(old.wrapping_add(1))],
            );
            r.write_u8(SUB, 10);
            r.write_u8(SUBSUB, 0);
        }
    }
    r.write_u8(TORCH_TIMERS + usize::from(selector & 0xf), timer);
    r.write_u8(TORCH_SELECTOR, 0);
}

pub fn init_load_default_tile_attr(r: &mut Ram<'_>) {
    for (i, &v) in DUNGEON_DEFAULT_ATTR[..0x140].iter().enumerate() {
        r.write_u8(TILE_ATTRIBUTES + i, v);
    }
    for (i, &v) in DUNGEON_DEFAULT_ATTR[0x140..].iter().enumerate() {
        r.write_u8(TILE_ATTRIBUTES + 0x1c0 + i, v);
    }
}

pub fn load_dungeon_room_rebuild_hud<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    r.write_u8(MOSAIC_LEVEL, 0);
    r.write_u8(MOSAIC, 7);
    for call in [
        MiscCall::HudSearchEquipped,
        MiscCall::HudRebuild,
        MiscCall::HudUpdateEquipped,
        MiscCall::ModulePreDungeon,
    ] {
        hooks.call(r, call);
    }
}

pub fn module_unknown0() {
    panic!("C Module_Unknown0 assertion");
}
pub fn module_unknown1() {
    panic!("C Module_Unknown1 assertion");
}

fn kill_agahnim_load_music<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    r.write_u8(NMI_DISABLE, 0);
    inc8(r, MAP_STATE);
    inc8(r, SUB);
    hooks.call(r, MiscCall::LoadOwMusicIfNeeded);
}
fn kill_agahnim_init<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    r.write_u8(MUSIC, 8);
    r.write_u8(OW_TRANS_DIR, 8);
    hooks.call(r, MiscCall::InitializeMirrorHdma);
    r.write_u8(MAP_STATE, 0);
    hooks.call(r, MiscCall::PaletteWhiteFilter);
    hooks.call(r, MiscCall::OverworldLoadGfx);
    inc8(r, SUB);
    r.write_u8(PLAYER_STATE, 20);
    r.write_u16(BG1_X, 0);
    r.write_u16(BG1_Y, 0);
    r.write_u16(DUNG_SAVE_BITS, 0);
    r.write_u16(LINK_Y_VEL, 0);
    r.write_u16(MAIN_PALETTE, 0x7fff);
    r.write_u16(MAIN_PALETTE + 64, 0x7fff);
    hooks.call(r, MiscCall::TerminateInteractives(0));
    hooks.call(r, MiscCall::LinkResetProperties);
}
fn kill_agahnim_func2<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    r.write_u8(HDMAEN, 192);
    hooks.call(r, MiscCall::MirrorBuildWaving);
    inc8(r, SUB);
    r.write_u8(SUBSUB, 0);
}
fn kill_agahnim_func3<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.call(r, MiscCall::MirrorBuildWaving);
    if r.read_u8(SUBSUB) != 0 {
        r.write_u8(SUBSUB, 0);
        inc8(r, SUB);
    }
}
fn kill_agahnim_func4<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.call(r, MiscCall::MirrorBuildDewaving);
    if r.read_u8(SUBSUB) != 0 {
        r.write_u8(SUBSUB, 0);
        inc8(r, SUB);
    }
}
fn kill_agahnim_func5<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.call(
        r,
        MiscCall::HdmaSetup {
            addr6: 0,
            addr7: 0xf2fb,
            transfer_unit: 0x41,
            reg6: 0,
            reg7: 0x26,
            indirect_bank: 0,
        },
    );
    for i in 0..240 {
        r.write_u16(HDMA_DYNAMIC + i * 2, 0xff00);
    }
    r.write_u16(PALETTE_COUNTDOWN, 0);
    r.write_u16(PALETTE_DIRECTION, 0);
    r.write_u16(DIALOGUE, 0x35);
    main_show_text_message(r);
    hooks.call(r, MiscCall::ReloadSheets);
    hooks.call(r, MiscCall::HudRebuildIndoor);
    r.write_u8(HDMAEN, 0x80);
    r.write_u8(MAIN, 21);
    r.write_u8(SUB, 6);
    r.write_u8(SUBSUB, 24);
}
fn kill_agahnim_func6(r: &mut Ram<'_>) {
    let v = r.read_u8(SUBSUB).wrapping_sub(1);
    r.write_u8(SUBSUB, v);
    if v == 0 {
        inc8(r, SUB);
        r.write_u8(SFX_AMBIENT, 9);
    }
}
fn kill_agahnim_func7<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.call(r, MiscCall::RenderText);
    if r.read_u8(SUB) == 0 {
        r.write_u8(MAP_STATE, 0);
        r.write_u8(SFX_AMBIENT, 5);
        if r.read_u8(MOON_PEARL) == 0 {
            r.write_u16(DIALOGUE, 0x36);
            main_show_text_message(r);
            r.write_u8(SFX_AMBIENT, 0);
            r.write_u8(MAIN, 21);
            r.write_u8(SUB, 8);
        } else {
            r.write_u8(SUB, 9);
        }
    }
}
fn kill_agahnim_func8<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.call(r, MiscCall::RenderText);
    if r.read_u8(SUB) == 0 {
        r.write_u8(SUBSUB, 32);
        r.write_u8(SUB, 12);
    }
}
fn kill_agahnim_func12<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    let v = r.read_u8(SUBSUB).wrapping_sub(1);
    r.write_u8(SUBSUB, v);
    if v != 0 {
        return;
    }
    hooks.call(r, MiscCall::ResetAncillaCutscene);
    hooks.call(r, MiscCall::OverworldSetSongList);
    r.write_u8(SAVE_OW_INFO + 0x1b, r.read_u8(SAVE_OW_INFO + 0x1b) | 32);
    r.write_u8(PALACE_X2, 255);
    r.write_u8(SUB, 0);
    r.write_u8(MAP_STATE, 0);
    r.write_u8(NMI_DISABLE, 0);
    r.write_u8(MAIN, 9);
    r.write_u8(BG1_VOFS, 0);
    r.write_u8(MUSIC, if r.read_u8(MOON_PEARL) != 0 { 9 } else { 4 });
    r.write_u8(MAP_ICONS, 6);
}

pub fn module_main_routing<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    match r.read_u8(MAIN) {
        5 => module05_load_file(r, hooks),
        12 => module_unknown0(),
        13 => module_unknown1(),
        19 => module13_boss_victory_pendant(r, hooks),
        21 => module15_mirror_warp_from_aga(r, hooks),
        22 => module16_boss_victory_crystal(r, hooks),
        23 => module17_save_and_quit(r, hooks),
        module @ 0..=27 => hooks.route_main_module(r, module),
        _ => panic!("main module index outside C routing table"),
    }
}

pub fn module05_load_file<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.call(r, MiscCall::EnableForceBlank);
    r.write_u8(MAP_STATE, 0);
    r.write_u8(DUNG_UNK6, 0);
    for a in [BYTE_02D4, BYTE_02D7, TAGALONG_VAR5, BYTE_0379, BYTE_03FD] {
        r.write_u8(a, 0);
    }
    for call in [
        MiscCall::EraseTileMaps,
        MiscCall::LoadDefaultGraphics,
        MiscCall::SpriteLoadGraphicsProperties,
    ] {
        hooks.call(r, call);
    }
    init_load_default_tile_attr(r);
    hooks.call(r, MiscCall::DecompressSword);
    hooks.call(r, MiscCall::DecompressShield);
    hooks.call(r, MiscCall::LinkInitialize);
    hooks.call(r, MiscCall::LoadFollowerGraphics);
    for i in 0..4 {
        r.write_u8(SPR_GFX_SUBSETS + i, 70);
    }
    r.write_u16(WORD_02CD, 0x200);
    r.write_u8(VIRQ_TRIGGER, 48);
    if r.read_u8(DARK_WORLD) != 0 {
        if r.read_u8(INDOORS) != 0 {
            load_dungeon_room_rebuild_hud(r, hooks);
            return;
        }
        for call in [
            MiscCall::HudSearchEquipped,
            MiscCall::HudRebuild,
            MiscCall::HudUpdateEquipped,
        ] {
            hooks.call(r, call);
        }
        r.write_u8(DEATH_VAR5, 0);
        r.write_u16(DUNGEON_ROOM, 32);
        r.write_u8(MAIN, 8);
        r.write_u8(SUB, 0);
        r.write_u8(SUBSUB, 0);
        r.write_u8(DEATH_VAR4, 0);
    } else if r.read_u8(MOSAIC_LEVEL) != 0
        || (r.read_u8(DEATH_VAR5) != 0 && r.read_u8(DEATH_VAR4) == 0)
        || r.read_u8(PROGRESS) < 2
        || r.read_u8(STARTING_POINT) == 5
    {
        load_dungeon_room_rebuild_hud(r, hooks);
    } else {
        r.write_u16(DIALOGUE, if r.read_u8(MIRROR) == 2 { 0x185 } else { 0x184 });
        main_show_text_message(r);
        hooks.call(r, MiscCall::DungeonLoadPalettes);
        r.write_u8(INIDISP, 15);
        r.write_u8(TM, 4);
        r.write_u8(TS, 0);
        r.write_u8(MAIN, 27);
    }
}

pub fn boss_victory_heal<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if !hooks.refill_magic(r) {
        inc8(r, MAP_STATE);
    }
    if !hooks.refill_health(r) {
        inc8(r, MAP_STATE);
    }
    if r.read_u8(MAP_STATE) == 0 {
        r.write_u8(BUTTON_MASK, r.read_u8(BUTTON_MASK) & !0x40);
        hooks.call(r, MiscCall::DungeonResetTorchAndPlayer);
        r.write_u8(LINK_FACING, 2);
        r.write_u8(LINK_DIR_LAST, 4);
        inc8(r, UPDATE_HUD);
        inc8(r, SUB);
        r.write_u8(SUBSUB, 16);
        inc8(r, IMMOBILIZED);
    }
    r.write_u8(MAP_STATE, 0);
    hooks.call(r, MiscCall::HudRefillLogic);
}
pub fn dungeon_start_victory_spin<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    let v = r.read_u8(SUBSUB).wrapping_sub(1);
    r.write_u8(SUBSUB, v);
    if v != 0 {
        return;
    }
    r.write_u8(IMMOBILIZED, 0);
    r.write_u8(LINK_FACING, 2);
    hooks.call(r, MiscCall::LinkAnimateVictorySpin);
    hooks.call(r, MiscCall::TerminateInteractives(0));
    hooks.call(r, MiscCall::AddVictorySpin);
    inc8(r, SUB);
}
pub fn dungeon_run_victory_spin<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    hooks.call(r, MiscCall::LinkMain);
    if r.read_u8(PLAYER_STATE) != 0 {
        return;
    }
    if r.read_u8(SWORD).wrapping_add(1) & 0xfe != 0 {
        r.write_u8(SFX1, 0x2c);
    }
    r.write_u8(HOLD_SWORD, 1);
    r.write_u8(SUBSUB, 32);
    inc8(r, SUB);
}
pub fn dungeon_close_victory_spin(r: &mut Ram<'_>) {
    let v = r.read_u8(SUBSUB).wrapping_sub(1);
    r.write_u8(SUBSUB, v);
    if v != 0 {
        return;
    }
    inc8(r, SUB);
    r.write_u8(LINK_Y_VEL, 0);
    r.write_u8(LINK_X_VEL, 0);
    r.write_u8(FIXED_COLOR, 0);
}

pub fn module13_boss_victory_pendant<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    match r.read_u8(SUB) {
        0 => boss_victory_heal(r, hooks),
        1 => dungeon_start_victory_spin(r, hooks),
        2 => dungeon_run_victory_spin(r, hooks),
        3 => dungeon_close_victory_spin(r),
        4 => hooks.call(r, MiscCall::DungeonPrepExitSpotlight),
        5 => hooks.call(r, MiscCall::SpotlightConfigure),
        _ => panic!("boss victory state"),
    }
    hooks.call(r, MiscCall::SpriteMain);
    hooks.call(r, MiscCall::LinkOamMain);
}

pub fn module15_mirror_warp_from_aga<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    match r.read_u8(SUB) {
        0 => kill_agahnim_load_music(r, hooks),
        1 => kill_agahnim_init(r, hooks),
        2 => kill_agahnim_func2(r, hooks),
        3 => kill_agahnim_func3(r, hooks),
        4 => kill_agahnim_func4(r, hooks),
        5 => kill_agahnim_func5(r, hooks),
        6 => kill_agahnim_func6(r),
        7 => kill_agahnim_func7(r, hooks),
        8 => kill_agahnim_func8(r, hooks),
        9 => boss_victory_heal(r, hooks),
        10 => dungeon_start_victory_spin(r, hooks),
        11 => dungeon_run_victory_spin(r, hooks),
        12 => kill_agahnim_func12(r, hooks),
        _ => panic!("Agahnim state"),
    }
    let sub = r.read_u8(SUB);
    if !(2..5).contains(&sub) {
        hooks.call(r, MiscCall::SpriteMain);
        hooks.call(r, MiscCall::LinkOamMain);
    }
}

pub fn module16_boss_victory_crystal<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    match r.read_u8(SUB) {
        0 => boss_victory_heal(r, hooks),
        1 => dungeon_start_victory_spin(r, hooks),
        2 => dungeon_run_victory_spin(r, hooks),
        3 => dungeon_close_victory_spin(r),
        4 => module16_04_fade_and_end(r, hooks),
        _ => {}
    }
    hooks.call(r, MiscCall::SpriteMain);
    hooks.call(r, MiscCall::LinkOamMain);
}
pub fn module16_04_fade_and_end<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    let v = r.read_u8(INIDISP).wrapping_sub(1);
    r.write_u8(INIDISP, v);
    if v != 0 {
        return;
    }
    r.write_u16(BG1_X, 0);
    r.write_u16(BG1_Y, 0);
    r.write_u8(LINK_Y_VEL, 0);
    r.write_u8(IMMOBILIZED, 0);
    hooks.call(r, MiscCall::PaletteRevertSwap);
    r.write_u8(PLAYER_STATE, 0);
    r.write_u8(RECEIVE_ITEM, 0);
    r.write_u8(ITEM_POSE, 0);
    r.write_u8(DISABLE_DAMAGE, 0);
    r.write_u8(MAIN, r.read_u8(SAVED_MODULE));
    r.write_u8(SUB, 0);
    r.write_u8(SUBSUB, 0);
    hooks.call(r, MiscCall::OpenSpotlightNext2);
}

pub fn triforce_room_link_approach_triforce(r: &mut Ram<'_>) {
    let y = r.read_u16(LINK_Y) as u8;
    if y < 152 {
        r.write_u8(LINK_ANIM_STEPS, 0);
        r.write_u8(LINK_DIR, 0);
        r.write_u8(LINK_DIR_LAST, 0);
        let t = r.read_u8(SPIN_DELAY).wrapping_sub(1);
        r.write_u8(SPIN_DELAY, t);
        if t == 0 {
            r.write_u8(ITEM_POSE, 2);
            inc8(r, SUBSUB);
        }
    } else {
        if y < 169 {
            r.write_u8(SPEED, 0x14);
        }
        r.write_u8(LINK_DIR, 8);
        r.write_u8(LINK_DIR_LAST, 8);
        r.write_u8(LINK_FACING, 0);
        r.write_u8(SPIN_DELAY, 64);
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
        let target = ReceiptItem::from_raw(4).spec().target;
        r.set_inventory_byte(target, ReceiptItem::from_raw(0).spec().initial_value as u8);
    }
    let spec = ReceiptItem::from_raw(j as u8).spec();
    let p = spec.target;
    let value = spec.initial_value;
    if value >= 0 {
        r.set_inventory_byte(p, value as u8);
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
        let v = r.inventory_byte(p) | t;
        r.set_inventory_byte(p, v);
        if v & 7 == 7 {
            r.write_u8(MAP_ICONS, 4);
        }
        inc8(r, MAP_STATE);
    } else if j == 0x22 {
        if r.inventory_byte(p) == 0 {
            r.set_inventory_byte(p, 1);
        }
    } else if matches!(j, 0x25 | 0x32 | 0x33) {
        let shift = r.read_u8(PALACE_X2) >> 1;
        let bit = if shift < 32 {
            // C promotes 0x8000 to a 32-bit int: shifts16..31 yield zero.
            (0x8000u32 >> shift) as u16
        } else {
            // Original C is undefined here. Preserve the existing Rust
            // behavior, including debug panic timing, rather than masking it.
            0x8000u16 >> shift
        };
        r.insert_inventory_dungeon_bits(p, bit);
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
            r.set_inventory_byte(p, 1);
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
            let v = r.inventory_byte(p).wrapping_add(t).min(99);
            r.set_inventory_byte(p, v);
            hooks.call(r, MiscCall::RefreshIcon);
        } else if j == 0x17 {
            r.set_inventory_byte(p, r.inventory_byte(p).wrapping_add(1) & 3);
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

pub fn module17_save_and_quit<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if r.read_u8(SUB) == 0 {
        inc8(r, SUB);
    }
    if r.read_u8(SUB) == 1 {
        let v = r.read_u8(INIDISP).wrapping_sub(1);
        r.write_u8(INIDISP, v);
        if v == 0 {
            r.write_u8(MOSAIC, 15);
            r.write_u8(SUBSUB, 1);
            hooks.call(r, MiscCall::DeathFunc15(false));
        }
    }
    hooks.call(r, MiscCall::SpriteMain);
    hooks.call(r, MiscCall::LinkOamMain);
}

pub fn wallmaster_send_player_to_last_entrance<H: MiscHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    for call in [
        MiscCall::SaveDungeonKeys,
        MiscCall::FlagRoomQuadrants,
        MiscCall::SpriteResetAll,
    ] {
        hooks.call(r, call);
    }
    r.write_u8(DEATH_VAR4, 0);
    r.write_u8(MAIN, 17);
    r.write_u8(SUB, 0);
    r.write_u8(NMI_LOAD_BG, 0);
    hooks.call(r, MiscCall::ResetAfterDeath(17));
}

fn inc8(r: &mut Ram<'_>, address: usize) {
    r.write_u8(address, r.read_u8(address).wrapping_add(1));
}

#[cfg(test)]
pub(crate) mod receipt_frozen;

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[derive(Default)]
    struct Hooks {
        calls: Vec<MiscCall>,
        xy: Option<(usize, u16, u16)>,
        magic: bool,
        health: bool,
    }
    impl MiscHooks for Hooks {
        fn call(&mut self, _: &mut Ram<'_>, call: MiscCall) {
            self.calls.push(call);
        }
        fn route_main_module(&mut self, _: &mut Ram<'_>, module: u8) {
            self.calls.push(MiscCall::DecodeAnimatedTile(module));
        }
        fn load_song_bank(&mut self, _: &mut Ram<'_>, bank: u8) {
            self.calls.push(MiscCall::DecodeAnimatedTile(bank));
        }
        fn predefined_tile_word(&self, word_index: usize) -> u16 {
            0x4000 | word_index as u16
        }
        fn refill_magic(&mut self, _: &mut Ram<'_>) -> bool {
            self.magic
        }
        fn refill_health(&mut self, _: &mut Ram<'_>) -> bool {
            self.health
        }
        fn add_ancilla(&mut self, _: &mut Ram<'_>, _: u8, _: u8) -> i32 {
            0
        }
        fn set_ancilla_xy(&mut self, _: &mut Ram<'_>, ancilla: usize, x: u16, y: u16) {
            self.xy = Some((ancilla, x, y));
        }
        fn sprite_x(&self, _: &Ram<'_>, sprite: usize) -> u16 {
            0x100 + sprite as u16
        }
        fn dungeon_item_tile_action(&mut self, _: &mut Ram<'_>, x: u16, _: u16) -> u8 {
            x as u8
        }
        fn overworld_item_tile_action(&mut self, _: &mut Ram<'_>, _: u16, y: u16) -> u8 {
            y as u8
        }
    }

    #[test]
    fn rng_matches_wrapping_lfsr_step() {
        let mut bytes = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut bytes);
        r.write_u8(RNG_STATE, 0xfe);
        r.write_u8(FRAME, 3);
        assert_eq!(get_random_number(&mut r), 0);
        r.write_u8(FRAME, 0);
        assert_eq!(get_random_number(&mut r), 0xb8);
    }

    #[test]
    fn sound_pan_uses_signed_wrapped_screen_delta() {
        let mut bytes = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut bytes);
        r.write_u16(BG2_HOFS, 100);
        assert_eq!(calculate_sfx_pan(&r, 99), 0x80);
        assert_eq!(calculate_sfx_pan(&r, 259), 0);
        assert_eq!(calculate_sfx_pan(&r, 260), 0x40);
    }

    #[test]
    fn nmi_packs_oam_and_advances_dma_animations() {
        let mut bytes = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut bytes);
        for (i, v) in [1, 2, 3, 0].into_iter().enumerate() {
            r.write_u8(BYTEWISE_OAM + i, v);
        }
        r.write_u16(BG_ANIM_COUNTDOWN, 1);
        r.write_u16(DMA_ANIM_COUNTDOWN, 1);
        nmi_prepare_sprites(&mut r);
        assert_eq!(r.read_u8(EXTENDED_OAM), 0x39);
        assert_eq!(r.read_u16(DMA_SOURCE_3), 0x8080);
        assert_eq!(r.read_u16(DMA_SOURCE_0), 0x8280);
        assert_eq!(r.read_u16(BG_ANIM_OFFSET), 0x400);
        assert_eq!(r.read_u16(ANIMATED_TILE_SRC), 0xaa80);
        assert_eq!(r.read_u16(DMA_ANIM_COUNTDOWN), 4);
        assert_eq!(r.read_u16(DMA_SOURCE_9), 0xb2a0);
    }

    #[test]
    fn overlay_dma_preserves_horizontal_and_vertical_stride_quirks() {
        let mut bytes = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut bytes);
        for i in 0..256 {
            r.write_u16(OVERWORLD_TILEATTR + i * 2, i as u16);
        }
        assert_eq!(
            dungeon_prep_overlay_dma_watergate(&mut r, 0, 0, 0x880, 1),
            6
        );
        assert_eq!(
            (
                r.read_u16(VRAM_UPLOAD + 4),
                r.read_u16(VRAM_UPLOAD + 6),
                r.read_u16(VRAM_UPLOAD + 10)
            ),
            (0, 1, 3)
        );
        assert_eq!(
            dungeon_prep_overlay_dma_watergate(&mut r, 8, 0, 0x881, 1),
            14
        );
        assert_eq!(
            (
                r.read_u16(VRAM_UPLOAD + 20),
                r.read_u16(VRAM_UPLOAD + 22),
                r.read_u16(VRAM_UPLOAD + 26)
            ),
            (0, 64, 192)
        );
    }

    #[test]
    fn bottle_and_item_receipt_paths_use_exact_save_offsets() {
        let mut bytes = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut bytes);
        item_receipt_give_bottled_item(&mut r, 0x2b);
        assert_eq!(r.read_u8(BOTTLES), 3);
        r.write_u8(RECEIVE_ITEM, 0x24);
        r.write_u8(0xf36f, 98);
        r.write_u16(LINK_X, 0x120);
        r.write_u16(LINK_Y, 0x220);
        let mut hooks = Hooks::default();
        ancilla_add_item_receipt(&mut r, &mut hooks, 0x22, 4, 0);
        assert_eq!(r.read_u8(0xf36f), 99);
        assert_eq!(hooks.xy, Some((0, 0x12a, 0x212)));
        assert!(hooks.calls.contains(&MiscCall::RefreshIcon));
    }

    #[test]
    fn default_attributes_keep_the_c_gap_untouched() {
        let mut bytes = [0xaa; WRAM_SIZE];
        let mut r = Ram::new(&mut bytes);
        init_load_default_tile_attr(&mut r);
        assert_eq!(
            r.slice(TILE_ATTRIBUTES, 0x140),
            &DUNGEON_DEFAULT_ATTR[..0x140]
        );
        assert!(
            r.slice(TILE_ATTRIBUTES + 0x140, 0x80)
                .iter()
                .all(|&v| v == 0xaa)
        );
        assert_eq!(
            r.slice(TILE_ATTRIBUTES + 0x1c0, 64),
            &DUNGEON_DEFAULT_ATTR[0x140..]
        );
    }

    #[test]
    fn main_router_uses_hex_named_local_module_at_decimal_index() {
        let mut bytes = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut bytes);
        let mut hooks = Hooks::default();
        r.write_u8(MAIN, 19);
        module_main_routing(&mut r, &mut hooks);
        assert!(hooks.calls.contains(&MiscCall::HudRefillLogic));
        assert!(hooks.calls.contains(&MiscCall::SpriteMain));
        assert!(!hooks.calls.contains(&MiscCall::DecodeAnimatedTile(19)));

        hooks.calls.clear();
        r.write_u8(MAIN, 9);
        module_main_routing(&mut r, &mut hooks);
        assert_eq!(hooks.calls, vec![MiscCall::DecodeAnimatedTile(9)]);
    }
}
