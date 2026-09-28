//! Native follower logic. State addresses match `src/variables.h`.
use crate::{oam, ram::Ram};

const MAIN: usize = 0x10;
const SUB: usize = 0x11;
const LINK_Y: usize = 0x20;
const LINK_X: usize = 0x22;
const LINK_Z: usize = 0x24;
const DIR: usize = 0x2f;
const YVEL: usize = 0x30;
const XVEL: usize = 0x31;
const PLAYER_STATE: usize = 0x5d;
const SPEED: usize = 0x5e;
const LOWER: usize = 0xee;
const IMMOBILE: usize = 0x2e4;
const VAR2: usize = 0x2cf;
const VAR3: usize = 0x2d0;
const VAR1: usize = 0x2d3;
const REACQUIRE: usize = 0x2d2;
const VAR4: usize = 0x2d6;
const RIPPLE: usize = 0x351;
const RUNNING: usize = 0x372;
const HOOKSHOT: usize = 0x37e;
const YLO: usize = 0x1a00;
const YHI: usize = 0x1a14;
const XLO: usize = 0x1a28;
const XHI: usize = 0x1a3c;
const Z: usize = 0x1a50;
const LAYER: usize = 0x1a64;
const FOLLOWER: usize = 0xf3cc;
const SAVED_Y: usize = 0xf3cd;
const SAVED_X: usize = 0xf3cf;
const SAVED_INDOORS: usize = 0xf3d1;
const SAVED_FLOOR: usize = 0xf3d2;
const DROPPED: usize = 0xf3d3;
const INDOORS: usize = 0x1b;
const FLAGS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
const FRAME: usize = 0x1a;
const AUX_STATE: usize = 0x4d;
const NEAR_PIT: usize = 0x5b;
const OAM_PRIORITY: usize = 0x64;
const OW_SCREEN: usize = 0x8a;
const OAM_CUR: usize = 0x90;
const OAM_EXT_CUR: usize = 0x92;
const DUNGEON_ROOM: usize = 0xa0;
const BG2_HOFS: usize = 0xe2;
const BG2_VOFS: usize = 0xe8;
const JOYPAD_L: usize = 0xf6;
const MUSIC_CONTROL: usize = 0x12c;
const MESSAGE_TIMER: usize = 0x2cd;
const VAR7: usize = 0x2d1;
const DRAW_STATE: usize = 0x2d7;
const EVENT_FLAGS: usize = 0x2f2;
const VAR5: usize = 0x2f9;
const ITEM_IN_HAND: usize = 0x301;
const STATE_BITS: usize = 0x308;
const PICKUP_SPRITE: usize = 0x314;
const BLINK: usize = 0x31f;
const SWIMCOLL_VAR7: usize = 0x33c;
const GRABBING_WALL: usize = 0x376;
const MASTER_SWORD: usize = 0x377;
const POSITION_MODE: usize = 0x37a;
const DUNG_SAVE_BITS: usize = 0x402;
const TRAPDOORS_DOWN: usize = 0x468;
const SUPER_BOMB_2: usize = 0x4b4;
const SUPER_BOMB_1: usize = 0x4b5;
const DOOR_POS: usize = 0x68e;
const DOOR_STEP: usize = 0x690;
const PALETTE_SWAP: usize = 0xabd;
const DMA_VAR6: usize = 0xae8;
const DMA_VAR7: usize = 0xaea;
const BLIND_BYTE: usize = 0xb69;
const ANC_Y_LO: usize = 0xbfa;
const ANC_X_LO: usize = 0xc04;
const ANC_Y_HI: usize = 0xc0e;
const ANC_X_HI: usize = 0xc18;
const ANC_Y_VEL: usize = 0xc22;
const ANC_X_VEL: usize = 0xc2c;
const SPR_Y_LO: usize = 0xd00;
const SPR_X_LO: usize = 0xd10;
const SPR_Y_HI: usize = 0xd20;
const SPR_X_HI: usize = 0xd30;
const SPR_IGNORE_PROJECTILE: usize = 0xba0;
const SPR_GRAPHICS: usize = 0xdc0;
const SPR_STATE: usize = 0xdd0;
const SPR_D: usize = 0xde0;
const SPR_DELAY_AUX2: usize = 0xe10;
const SPR_TYPE: usize = 0xe20;
const SPR_SUBTYPE2: usize = 0xe80;
const SPR_HEAD_DIR: usize = 0xeb0;
const SPR_FLOOR: usize = 0xf20;
const SPR_Z: usize = 0xf70;
const SPR_Z_VEL: usize = 0xf80;
const SORT_SPRITES: usize = 0xfb3;
const DIALOGUE: usize = 0x1cf0;
const SAVE_DUNG_INFO: usize = 0xf000;
const SAVE_OW_INFO: usize = 0xf280;

const TAB5: [u8; 15] = [0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const TAB4: [u8; 15] = [0, 0, 3, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const TIMED_TYPES: [u8; 3] = [5, 9, 10];
const TIMED_DELAYS: [u16; 3] = [0xdf3, 0x6f9, 0xdf3];
const TIMED_MESSAGES: [u16; 3] = [0x20, 0x108, 0x11d];
const INDOOR_INFOS: [TagalongMessageInfo; 12] = [
    TagalongMessageInfo::new(0x1ef0, 0x288, 1, 0x99, 4),
    TagalongMessageInfo::new(0x1e58, 0x2f0, 2, 0x9a, 4),
    TagalongMessageInfo::new(0x1ea8, 0x3b8, 4, 0x9b, 4),
    TagalongMessageInfo::new(0xcf8, 0x25b, 1, 0x21, 1),
    TagalongMessageInfo::new(0xcf8, 0x39d, 2, 0x21, 1),
    TagalongMessageInfo::new(0xc78, 0x238, 4, 0x21, 1),
    TagalongMessageInfo::new(0xa30, 0x2f8, 1, 0x22, 1),
    TagalongMessageInfo::new(0x178, 0x550, 1, 0x23, 1),
    TagalongMessageInfo::new(0x168, 0x4f8, 2, 0x2a, 1),
    TagalongMessageInfo::new(0x1bd8, 0x16fc, 1, 0x124, 6),
    TagalongMessageInfo::new(0x1520, 0x167c, 1, 0x124, 6),
    TagalongMessageInfo::new(0x5ac, 0x4fc, 1, 0x29, 1),
];
const OUTDOOR_INFOS: [TagalongMessageInfo; 5] = [
    TagalongMessageInfo::new(0x3c0, 0x730, 1, 0x9d, 4),
    TagalongMessageInfo::new(0x648, 0xf50, 0, 0xffff, 10),
    TagalongMessageInfo::new(0x6c8, 0xd78, 1, 0xffff, 10),
    TagalongMessageInfo::new(0x688, 0xc78, 2, 0xffff, 10),
    TagalongMessageInfo::new(0xe8, 0x90, 0, 0x28, 14),
];
const INDOOR_OFFSETS: [usize; 8] = [0, 3, 6, 7, 9, 10, 11, 12];
const OUTDOOR_OFFSETS: [usize; 4] = [0, 1, 4, 5];
const INDOOR_ROOMS: [u16; 7] = [0xf1, 0x61, 0x51, 2, 0xdb, 0xab, 0x22];
const OUTDOOR_ROOMS: [u16; 3] = [3, 0x5e, 0];
#[derive(Clone, Copy)]
struct SprXy {
    y1: i8,
    x1: i8,
    y2: i8,
    x2: i8,
}
const fn xy(y1: i8, x1: i8, y2: i8, x2: i8) -> SprXy {
    SprXy { y1, x1, y2, x2 }
}
const SPR_XY: [SprXy; 56] = [
    xy(-2, 0, 0, 0),
    xy(-2, 0, 0, 0),
    xy(-2, 0, 0, 0),
    xy(-2, 0, 0, 0),
    xy(-1, 0, 1, 0),
    xy(-1, 0, 1, 0),
    xy(-1, 0, 1, 0),
    xy(-1, 0, 1, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(1, 0, 0, 0),
    xy(1, 0, 0, 0),
    xy(1, 0, 0, 0),
    xy(1, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, -3, 0, 0),
    xy(0, 3, 0, 0),
    xy(1, 0, 0, 0),
    xy(1, 0, 0, 0),
    xy(1, -3, 1, 0),
    xy(1, 3, 1, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 1, 0),
    xy(0, 0, 1, 0),
    xy(0, 0, 1, 0),
    xy(0, 0, 1, 0),
    xy(-1, 0, 0, 0),
    xy(-1, 0, 0, 0),
    xy(-1, 0, 0, 0),
    xy(-1, 0, 0, 0),
    xy(0, 0, 1, 0),
    xy(0, 0, 1, 0),
    xy(0, 0, 1, 0),
    xy(0, 0, 1, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(0, 0, 0, 0),
    xy(2, 0, 0, 0),
    xy(2, 0, 0, 0),
    xy(2, -1, 0, 0),
    xy(2, 1, 0, 0),
    xy(3, 0, 1, 0),
    xy(3, 0, 1, 0),
    xy(3, -1, 1, 0),
    xy(3, 1, 1, 0),
];
#[derive(Clone, Copy)]
struct DmaFlags {
    dma6: u8,
    dma7: u8,
    flags: u8,
}
const fn df(dma6: u8, dma7: u8, flags: u8) -> DmaFlags {
    DmaFlags { dma6, dma7, flags }
}
const DMA_FLAGS: [DmaFlags; 16] = [
    df(0x20, 0xc0, 0),
    df(0, 0xa0, 0),
    df(0x40, 0x60, 0),
    df(0x40, 0x60, 0x44),
    df(0x20, 0xc0, 4),
    df(0, 0xa0, 4),
    df(0x40, 0x80, 0),
    df(0x40, 0x80, 0x44),
    df(0x20, 0xe0, 0),
    df(0, 0xe0, 0),
    df(0x40, 0xe0, 0),
    df(0x40, 0xe0, 0x44),
    df(0x20, 0xe0, 4),
    df(0, 0xe0, 4),
    df(0x40, 0xe0, 4),
    df(0x40, 0xe0, 0x40),
];
const DRAW_PALS: [u8; 14] = [0, 4, 4, 4, 4, 0, 7, 4, 4, 3, 4, 4, 4, 4];
const DRAW_OFFS: [u16; 14] = [
    0, 0, 0x80, 0x80, 0x80, 0, 0, 0xc0, 0xc0, 0x100, 0x180, 0x180, 0x140, 0x140,
];
const SPR_INFO: [u8; 24] = [
    0xd8, 0x24, 0xd8, 0x64, 0xd9, 0x24, 0xd9, 0x64, 0xda, 0x24, 0xda, 0x64, 0xc8, 0x22, 0xc8, 0x62,
    0xc9, 0x22, 0xc9, 0x62, 0xca, 0x22, 0xca, 0x62,
];
const SPR_OFFS0: [u16; 2] = [0x170, 0xc0];
const SPR_OFFS1: [u16; 2] = [0x1c0, 0x110];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProjectSpeed {
    pub x: u8,
    pub y: u8,
}
pub trait TagalongHooks {
    fn enhanced_features0(&self) -> u32;
    fn project_speed_towards_player(
        &mut self,
        ram: &mut Ram<'_>,
        ancilla: usize,
        speed: u8,
    ) -> ProjectSpeed;
    fn ancilla_move_y(&mut self, ram: &mut Ram<'_>, ancilla: usize);
    fn ancilla_move_x(&mut self, ram: &mut Ram<'_>, ancilla: usize);
    fn show_text_message(&mut self, ram: &mut Ram<'_>);
    fn add_super_bomb_explosion(&mut self, ram: &mut Ram<'_>, kind: u8, arg: u8) -> i32;
    fn old_man_revert_to_sprite(&mut self, ram: &mut Ram<'_>, history: usize);
    fn sprite_spawn_dynamically(&mut self, ram: &mut Ram<'_>, source: usize, kind: u8) -> i32;
    fn sprite_prep_load_properties(&mut self, ram: &mut Ram<'_>, sprite: usize);
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TagalongMessageInfo {
    pub y: u16,
    pub x: u16,
    pub bit: u16,
    pub message: u16,
    pub tagalong: u16,
}
impl TagalongMessageInfo {
    const fn new(y: u16, x: u16, bit: u16, message: u16, tagalong: u16) -> Self {
        Self {
            y,
            x,
            bit,
            message,
            tagalong,
        }
    }
}

#[must_use]
pub fn is_following(r: &Ram<'_>) -> bool {
    r.read_u8(IMMOBILE) == 0
        && r.read_u8(SUB) != 10
        && !(r.read_u8(MAIN) == 9 && r.read_u8(SUB) == 0x23)
        && !(r.read_u8(MAIN) == 14 && matches!(r.read_u8(SUB), 1 | 2))
}
#[must_use]
pub fn validate_message_freedom(r: &Ram<'_>) -> bool {
    matches!(r.read_u8(PLAYER_STATE), 0 | 4 | 17)
        && (r.read_u8(0x3a) & 0x80
            | r.read_u8(MASTER_SWORD)
            | r.read_u8(ITEM_IN_HAND)
            | r.read_u8(POSITION_MODE)
            | r.read_u8(0x2ec)
            | r.read_u8(PICKUP_SPRITE)
            | r.read_u8(STATE_BITS)
            | r.read_u8(GRABBING_WALL))
            == 0
}

pub fn initialize<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &H) {
    let y = r.read_u16(LINK_Y);
    let x = r.read_u16(LINK_X);
    set16(r, YLO, YHI, 0, y);
    set16(r, XLO, XHI, 0, x);
    r.write_u8(
        LAYER,
        FLAGS[usize::from(r.read_u8(LOWER))] >> 2 | r.read_u8(DIR) >> 1,
    );
    r.write_u8(REACQUIRE, 64);
    for a in [VAR2, VAR1, VAR3, VAR4, SPEED] {
        r.write_u8(a, 0)
    }
    if hooks.enhanced_features0() & 4 != 0 {
        r.write_u8(PLAYER_STATE, 0);
        r.write_u8(RUNNING, 0);
    }
}

pub fn move_towards_link<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    loop {
        let j = usize::from(r.read_u8(VAR1));
        for (dst, src) in [
            (ANC_Y_LO, YLO),
            (ANC_Y_HI, YHI),
            (ANC_X_LO, XLO),
            (ANC_X_HI, XHI),
        ] {
            r.write_u8(dst + 9, r.read_u8(src + j));
        }
        let speed = hooks.project_speed_towards_player(r, 9, 24);
        r.write_u8(ANC_X_VEL + 9, speed.x);
        r.write_u8(ANC_Y_VEL + 9, speed.y);
        hooks.ancilla_move_y(r, 9);
        hooks.ancilla_move_x(r, 9);
        let x = get16(r, ANC_X_LO, ANC_X_HI, 9);
        let y = get16(r, ANC_Y_LO, ANC_Y_HI, 9);
        if ad(x, r.read_u16(LINK_X)) < 2 && ad(y, r.read_u16(LINK_Y)) < 2 {
            return;
        }
        let k = r.read_u8(VAR1).wrapping_add(1);
        r.write_u8(VAR1, k);
        if k == 18 {
            return;
        }
        let k = usize::from(k);
        set16(r, YLO, YHI, k, y);
        set16(r, XLO, XHI, k, x);
        r.write_u8(LAYER + k, FLAGS[usize::from(r.read_u8(LOWER))] >> 2 | 1);
    }
}

pub fn sprite_become_follower<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H, k: usize) {
    r.write_u8(VAR5, 0);
    let y = get16(r, SPR_Y_LO, SPR_Y_HI, k).wrapping_sub(6);
    let x = get16(r, SPR_X_LO, SPR_X_HI, k).wrapping_add(1);
    set16(r, YLO, YHI, 0, y);
    set16(r, XLO, XHI, 0, x);
    r.write_u8(LAYER, FLAGS[usize::from(r.read_u8(LOWER))] >> 2 | 1);
    r.write_u8(REACQUIRE, 64);
    for a in [VAR1, VAR2, VAR3, VAR4, SPEED, VAR5, DROPPED] {
        r.write_u8(a, 0);
    }
    move_towards_link(r, hooks);
}

pub fn main<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    let follower = r.read_u8(FOLLOWER);
    if follower == 0 {
        return;
    }
    if follower == 14 {
        handle_trigger(r, hooks);
        return;
    }
    let j = TIMED_TYPES.iter().position(|&v| v == follower);
    if let Some(j) = j
        && r.read_u8(SUB) == 0
        && !(j == 2 && r.read_u16(OW_SCREEN) & 0x40 != 0)
    {
        let timer = r.read_u16(MESSAGE_TIMER).wrapping_sub(1);
        r.write_u16(MESSAGE_TIMER, timer);
        if (timer as i16) < 0 {
            if !validate_message_freedom(r) {
                r.write_u16(MESSAGE_TIMER, 0);
            } else {
                r.write_u16(MESSAGE_TIMER, TIMED_DELAYS[j]);
                r.write_u16(DIALOGUE, TIMED_MESSAGES[j]);
                hooks.show_text_message(r);
            }
        }
    }
    if j != Some(0) {
        no_timed_message(r, hooks);
    }
}

pub fn no_timed_message<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if r.read_u8(DROPPED) != 0 {
        not_following(r, hooks);
        return;
    }
    let follower = r.read_u8(FOLLOWER);
    let aux = r.read_u8(AUX_STATE);
    let may_drop = if follower == 12 {
        aux == 0
    } else if follower == 13 {
        !(aux == 2 || r.read_u8(NEAR_PIT) == 2)
    } else {
        false
    };
    let input_drop = may_drop
        && r.read_u8(SUB) == 0
        && aux != 1
        && r.read_u8(STATE_BITS) & 0x80 == 0
        && r.read_u8(VAR5) == 0
        && r.read_u8(VAR3) == 0
        && r.read_i8(Z + usize::from(r.read_u8(VAR2))) <= 0
        && r.read_u8(JOYPAD_L) & 0x80 != 0;
    let forced_bomb_drop = follower == 13 && (aux == 2 || r.read_u8(NEAR_PIT) == 2);
    if input_drop || forced_bomb_drop {
        drop_follower(r, hooks);
    } else {
        check_game_mode(r, hooks);
    }
}

fn drop_follower<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if r.read_u8(FOLLOWER) == 13 && r.read_u8(INDOORS) == 0 {
        if matches!(r.read_u8(PLAYER_STATE), 8..=10) {
            check_game_mode(r, hooks);
            return;
        }
        r.write_u8(SUPER_BOMB_2, 3);
        r.write_u8(SUPER_BOMB_1, 0xbb);
    }
    r.write_u8(DROPPED, 128);
    r.write_u8(REACQUIRE, 64);
    drop_snapshot(r);
    not_following(r, hooks);
}
#[must_use]
pub fn check_blind_trigger(r: &Ram<'_>) -> bool {
    let k = usize::from(r.read_u8(VAR2));
    let x = get16(r, XLO, XHI, k).wrapping_add(8);
    let y = get16(r, YLO, YHI, k)
        .wrapping_add(i16::from(r.read_i8(Z + k)) as u16)
        .wrapping_add(12);
    ad(0x1568, y) < 24 && ad(0x1980, x) < 24
}
#[must_use]
pub fn check_for_trigger(r: &Ram<'_>, i: &TagalongMessageInfo) -> bool {
    let x = r
        .read_u16(LINK_X)
        .wrapping_add(12)
        .wrapping_sub(i.x.wrapping_add(8));
    let y = r
        .read_u16(LINK_Y)
        .wrapping_add(12)
        .wrapping_sub(i.y.wrapping_add(8));
    abs(x) < 24 && abs(y) < 28
}
#[must_use]
pub fn check_proximity_to_link(r: &mut Ram<'_>) -> bool {
    let t = r.read_u8(REACQUIRE).wrapping_sub(1);
    r.write_u8(REACQUIRE, t);
    if (t as i8) >= 0 {
        return true;
    }
    r.write_u8(REACQUIRE, 0);
    let y = r.read_u16(SAVED_Y);
    let x = r.read_u16(SAVED_X);
    y.wrapping_sub(1) >= r.read_u16(LINK_Y)
        || y.wrapping_add(19) < r.read_u16(LINK_Y)
        || x.wrapping_sub(1) >= r.read_u16(LINK_X)
        || x.wrapping_add(19) < r.read_u16(LINK_X)
}
pub fn check_game_mode<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if is_following(r) && (r.read_u8(XVEL) | r.read_u8(YVEL)) != 0 {
        let mut k = r.read_u8(VAR1).wrapping_add(1);
        if k == 20 {
            k = 0
        }
        r.write_u8(VAR1, k);
        let mut z = r.read_u8(LINK_Z);
        if z >= 0xf0 {
            z = 0
        }
        let i = usize::from(k);
        r.write_u8(Z + i, z);
        set16(
            r,
            YLO,
            YHI,
            i,
            r.read_u16(LINK_Y).wrapping_sub(u16::from(z)),
        );
        set16(r, XLO, XHI, i, r.read_u16(LINK_X));
        let mut l = r.read_u8(DIR) >> 1 | FLAGS[usize::from(r.read_u8(LOWER))] >> 2;
        if r.read_u8(PLAYER_STATE) == 4 {
            l |= 0x20
        } else {
            if r.read_u8(PLAYER_STATE) == 19 && r.read_u8(HOOKSHOT) != 0 {
                l |= 0x10
            }
            let q = r.read_u8(RIPPLE);
            if q != 0 {
                l |= if q == 1 { 0x80 } else { 0x40 }
            }
        }
        r.write_u8(LAYER + i, l)
    }
    match r.read_u8(FOLLOWER) {
        2 | 4 => old_man(r, hooks),
        3 | 11 => old_man_unused(r, hooks),
        5 | 14 => panic!("C port asserts for follower type with unknown Y"),
        _ => basic_mover(r, hooks),
    }
}
pub fn disable(r: &mut Ram<'_>) {
    if matches!(r.read_u8(FOLLOWER), 9 | 10) {
        r.write_u8(FOLLOWER, 0)
    }
}
pub fn drop_snapshot(r: &mut Ram<'_>) {
    let k = usize::from(r.read_u8(VAR2));
    r.write_u16(SAVED_Y, get16(r, YLO, YHI, k));
    r.write_u16(SAVED_X, get16(r, XLO, XHI, k));
    r.write_u8(SAVED_FLOOR, r.read_u8(LOWER));
    r.write_u8(SAVED_INDOORS, r.read_u8(INDOORS));
    r.write_u8(DROPPED, 128)
}
pub fn old_man_unused_state(r: &mut Ram<'_>) {
    r.write_u8(SPEED, 16);
    if r.read_u8(RUNNING) == 0 {
        r.write_u8(SPEED, 0)
    }
}

pub fn handle_trigger<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if r.read_u8(SUB) != 0 {
        return;
    }
    let (infos, start, end): (&[TagalongMessageInfo], usize, usize) = if r.read_u8(INDOORS) != 0 {
        let Some(j) = INDOOR_ROOMS
            .iter()
            .position(|&v| v == r.read_u16(DUNGEON_ROOM))
        else {
            return;
        };
        (&INDOOR_INFOS, INDOOR_OFFSETS[j], INDOOR_OFFSETS[j + 1])
    } else {
        let Some(j) = OUTDOOR_ROOMS
            .iter()
            .position(|&v| v == r.read_u16(OW_SCREEN))
        else {
            return;
        };
        (&OUTDOOR_INFOS, OUTDOOR_OFFSETS[j], OUTDOOR_OFFSETS[j + 1])
    };
    let st = usize::from(next_history(r.read_u8(VAR2)));
    for info in &infos[start..end] {
        if info.tagalong != u16::from(r.read_u8(FOLLOWER)) || !check_for_trigger(r, info) {
            continue;
        }
        if info.bit as u8 & r.read_u8(EVENT_FLAGS) != 0 {
            return;
        }
        r.write_u8(EVENT_FLAGS, r.read_u8(EVENT_FLAGS) | info.bit as u8);
        r.write_u16(DIALOGUE, info.message);
        if info.message == 0xffff {
            if info.bit & 3 == 0 {
                kiki_revert_to_sprite(r, hooks, st);
            } else if r.read_u8(SAVE_OW_INFO + usize::from(r.read_u8(OW_SCREEN))) & 1 == 0 {
                kiki_spawn_handler_a(r, hooks, st);
            }
            return;
        }
        if info.message == 0x9d {
            hooks.old_man_revert_to_sprite(r, st);
        } else if info.message == 0x28 {
            r.write_u8(FOLLOWER, 0);
        }
        hooks.show_text_message(r);
        return;
    }
}

pub fn draw<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if r.read_u8(VAR5) != 0 {
        return;
    }
    let raw_k = r.read_u8(VAR2);
    // The original computes priority with the raw unsigned history index and
    // only then treats a negative index as zero for coordinates/animation.
    let priority_k = usize::from(raw_k);
    let k = if (raw_k as i8) < 0 { 0 } else { priority_k };
    let priority = if r.read_u8(Z + priority_k) != 0 && r.read_u8(INDOORS) == 0 {
        0x20
    } else if r.read_u8(SUB) == 14 {
        FLAGS[usize::from(r.read_u8(LOWER))]
    } else {
        (r.read_u8(LAYER + priority_k) & 0xc) << 2
    };
    r.write_u16(OAM_PRIORITY, u16::from(priority) << 8);
    let x = get16(r, XLO, XHI, k);
    let y = get16(r, YLO, YHI, k);
    animate_movement(r, hooks, r.read_u8(LAYER + k), x, y);
}

pub fn animate_movement<H: TagalongHooks>(
    r: &mut Ram<'_>,
    _hooks: &mut H,
    ain: u8,
    xin: u16,
    yin: u16,
) {
    let follower = r.read_u8(FOLLOWER);
    let mut yt = 0;
    let av;
    let mut sc = 0;
    if ain >> 2 & 8 != 0 && matches!(follower, 1 | 6) {
        yt = 8;
        av = if r.read_u16(SWIMCOLL_VAR7) | r.read_u16(SWIMCOLL_VAR7 + 2) != 0 {
            r.read_u8(FRAME) >> 1 & 4
        } else {
            r.read_u8(FRAME) >> 2 & 4
        };
    } else if matches!(r.read_u8(SUB), 8 | 14 | 16) {
        av = if r.read_u8(RUNNING) != 0 {
            r.read_u8(FRAME) & 4
        } else {
            r.read_u8(FRAME) >> 1 & 4
        };
    } else if follower == 11 {
        av = r.read_u8(FRAME) >> 1 & 4;
    } else if (matches!(follower, 12 | 13) && r.read_u8(DROPPED) != 0)
        || r.read_u8(IMMOBILE) != 0
        || r.read_u8(SUB) == 10
        || (r.read_u8(MAIN) == 9 && r.read_u8(SUB) == 0x23)
        || (r.read_u8(MAIN) == 14 && matches!(r.read_u8(SUB), 1 | 2))
        || r.read_u8(YVEL) | r.read_u8(XVEL) == 0
    {
        av = 4;
        sc = 4;
    } else {
        av = if r.read_u8(RUNNING) != 0 {
            r.read_u8(FRAME) & 4
        } else {
            r.read_u8(FRAME) >> 1 & 4
        };
    }
    let frame = usize::from((ain & 3).wrapping_add(av).wrapping_add(yt));
    let sort = usize::from(r.read_u8(SORT_SPRITES));
    let spr_offs = if (r.read_u16(LINK_Y) == yin && ain & 3 == 0) || r.read_u16(LINK_Y) < yin {
        SPR_OFFS0[sort] >> 2
    } else {
        SPR_OFFS1[sort] >> 2
    };
    r.write_u16(OAM_EXT_CUR, 0xa20 + spr_offs);
    r.write_u16(OAM_CUR, 0x800 + spr_offs * 4);
    let mut oi = usize::from(spr_offs);
    let scrolly = yin.wrapping_sub(r.read_u16(BG2_VOFS));
    let scrollx = xin.wrapping_sub(r.read_u16(BG2_HOFS));
    let mut sk = 0usize;
    let mut draw_extra = true;
    if follower == 1 || follower == 6 || ain & 0x20 == 0 {
        if ain & 0xc0 == 0 {
            draw_extra = false;
        } else {
            if ain & 0x80 == 0 {
                sk += 12;
            }
            if ain & 0x80 == 0 && sc != 0 {
                r.write_u8(DRAW_STATE, 0);
            } else {
                bump_draw_state(r);
            }
        }
    } else {
        bump_draw_state(r);
    }
    if draw_extra {
        sk += usize::from(r.read_u8(DRAW_STATE)) * 4;
        set_oam_follower(
            r,
            oi,
            scrollx,
            scrolly.wrapping_add(16),
            SPR_INFO[sk],
            SPR_INFO[sk + 1],
            0,
        );
        set_oam_follower(
            r,
            oi + 1,
            scrollx.wrapping_add(8),
            scrolly.wrapping_add(16),
            SPR_INFO[sk + 2],
            SPR_INFO[sk + 3],
            0,
        );
        oi += 2;
    }
    let mut pal = DRAW_PALS[usize::from(follower)];
    if pal == 7 && r.read_u8(PALETTE_SWAP) != 0 {
        pal = 0;
    }
    if follower == 13 {
        let bomb = r.read_u8(SUPER_BOMB_2);
        let colorful = if _hooks.enhanced_features0() & 4096 != 0 {
            bomb <= 1
        } else {
            bomb == 1
        };
        if colorful {
            pal = r.read_u8(FRAME) & 7;
        }
    }
    let si = frame + usize::from(DRAW_OFFS[usize::from(follower)] >> 3);
    let sprd = SPR_XY[si];
    let sprf = DMA_FLAGS[frame];
    if !matches!(follower, 12 | 13) {
        set_oam_follower(
            r,
            oi,
            add_i8(scrollx, sprd.x1),
            add_i8(scrolly, sprd.y1),
            0x20,
            sprf.flags & 0xf0 | pal << 1 | r.read_u8(OAM_PRIORITY + 1),
            2,
        );
        oi += 1;
        r.write_u8(DMA_VAR6, sprf.dma6);
    }
    set_oam_follower(
        r,
        oi,
        add_i8(scrollx, sprd.x2),
        add_i8(scrolly, sprd.y2).wrapping_add(8),
        0x22,
        (sprf.flags & 0xf) << 4 | pal << 1 | r.read_u8(OAM_PRIORITY + 1),
        2,
    );
    r.write_u8(DMA_VAR7, sprf.dma7);
}

fn bump_draw_state(r: &mut Ram<'_>) {
    if r.read_u8(FRAME) & 7 == 0 {
        let v = r.read_u8(DRAW_STATE).wrapping_add(1);
        r.write_u8(DRAW_STATE, if v == 3 { 0 } else { v });
    }
}

fn set_oam_follower(
    r: &mut Ram<'_>,
    index: usize,
    x: u16,
    y: u16,
    charnum: u8,
    flags: u8,
    mut big: u8,
) {
    let visible = x.wrapping_add(0x80) < 0x180 && {
        big |= (x >> 8) as u8 & 1;
        y.wrapping_add(0x10) < 0x100
    };
    oam::write(
        r,
        index,
        z3_core::c_types::OamEnt {
            x: x as u8,
            y: if visible { y as u8 } else { 0xf0 },
            charnum,
            flags,
        },
    );
    r.write_u8(oam::BYTEWISE_EXTENDED_OAM + index, big);
}

pub fn blind_spawn_from_maiden<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H, x: u16, y: u16) {
    r.write_u8(SPR_STATE, 9);
    r.write_u8(SPR_TYPE, 206);
    set16(r, SPR_X_LO, SPR_X_HI, 0, x);
    set16(r, SPR_Y_LO, SPR_Y_HI, 0, y.wrapping_sub(16));
    hooks.sprite_prep_load_properties(r, 0);
    r.write_u8(SPR_DELAY_AUX2, 192);
    r.write_u8(SPR_GRAPHICS, 21);
    r.write_u8(SPR_D, 2);
    r.write_u8(SPR_IGNORE_PROJECTILE, 2);
    r.write_u16(DUNG_SAVE_BITS, r.read_u16(DUNG_SAVE_BITS) | 0x2000);
    r.write_u8(BLIND_BYTE, 0);
}

pub fn kiki_revert_to_sprite<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H, k: usize) {
    let j = kiki_spawn_handler_monke(r, hooks, k);
    assert!(j >= 0, "C caller requires a free sprite slot");
    r.write_u8(SPR_SUBTYPE2 + j as usize, 1);
    r.write_u8(FOLLOWER, 0);
}

pub fn kiki_spawn_handler_monke<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H, k: usize) -> i32 {
    let j = hooks.sprite_spawn_dynamically(r, k, 0xb6);
    if j >= 0 {
        let j = j as usize;
        let dir = r.read_u8(LAYER + k) & 3;
        r.write_u8(SPR_HEAD_DIR + j, dir);
        r.write_u8(SPR_D + j, dir);
        set16(
            r,
            SPR_X_LO,
            SPR_X_HI,
            j,
            get16(r, XLO, XHI, k).wrapping_add(2),
        );
        set16(
            r,
            SPR_Y_LO,
            SPR_Y_HI,
            j,
            get16(r, YLO, YHI, k).wrapping_add(2),
        );
        r.write_u8(SPR_FLOOR + j, r.read_u8(LOWER));
        r.write_u8(SPR_IGNORE_PROJECTILE + j, 1);
        r.write_u8(SPR_FLOOR + j, 2);
        r.write_u8(SPEED, 0);
    }
    j
}

pub fn kiki_spawn_handler_a<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H, k: usize) {
    let j = kiki_spawn_handler_monke(r, hooks, k);
    assert!(j >= 0, "C caller requires a free sprite slot");
    r.write_u8(SPR_SUBTYPE2 + j as usize, 2);
}

pub fn kiki_spawn_handler_b<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H, k: usize) {
    let j = kiki_spawn_handler_monke(r, hooks, k);
    assert!(j >= 0, "C caller requires a free sprite slot");
    let j = j as usize;
    r.write_u8(SPR_Z + j, 1);
    r.write_u8(SPR_Z_VEL + j, 16);
    r.write_u8(SPR_SUBTYPE2 + j, 3);
    r.write_u8(FOLLOWER, 0);
}

pub fn basic_mover<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if !is_following(r) {
        draw(r, hooks);
        return;
    }
    handle_trigger(r, hooks);
    if r.read_u8(FOLLOWER) == 10 && r.read_u8(AUX_STATE) != 0 && r.read_u8(BLINK) != 0 {
        let k = next_history(r.read_u8(VAR2));
        kiki_spawn_handler_b(r, hooks, usize::from(k));
        r.write_u8(FOLLOWER, 0);
        return;
    }
    if r.read_u8(FOLLOWER) == 6
        && r.read_u16(DUNGEON_ROOM) == 0xac
        && r.read_u16(SAVE_DUNG_INFO + 101 * 2) & 0x100 != 0
        && check_blind_trigger(r)
    {
        let k = usize::from(r.read_u8(VAR2));
        let x = get16(r, XLO, XHI, k);
        let y = get16(r, YLO, YHI, k);
        r.write_u8(FOLLOWER, 0);
        blind_spawn_from_maiden(r, hooks, x, y);
        r.write_u8(TRAPDOORS_DOWN, r.read_u8(TRAPDOORS_DOWN).wrapping_add(1));
        r.write_u8(DOOR_POS, 0);
        r.write_u8(DOOR_STEP, 0);
        r.write_u8(SUB, 5);
        r.write_u8(MUSIC_CONTROL, 21);
        return;
    }
    // C has two distinct labels here: `label_e` performs the 15-entry lag
    // check, while `label_d` advances unconditionally.  In particular,
    // entering or remaining in the hookshot state jumps to `label_e` and must
    // not itself advance the follower history cursor.
    let mut check_lag = false;
    let mut advance = false;
    if r.read_u8(VAR3) == 0 {
        if r.read_u8(PLAYER_STATE) == 19 && r.read_u8(HOOKSHOT) != 0 {
            r.write_u8(VAR3, 1);
            check_lag = true;
        }
    } else if r.read_u8(PLAYER_STATE) == 19 {
        check_lag = true;
    } else if r.read_u8(VAR7) != r.read_u8(VAR2) {
        advance = true;
    } else {
        r.write_u8(VAR3, 0);
    }
    let k = usize::from(r.read_u8(VAR2));
    if !check_lag && !advance && r.read_i8(Z + k) > 0 {
        if r.read_u8(VAR1) != r.read_u8(VAR2) {
            advance = true;
        } else {
            r.write_u8(Z + k, 0);
            set16(r, YLO, YHI, k, r.read_u16(LINK_Y));
            set16(r, XLO, XHI, k, r.read_u16(LINK_X));
        }
    }
    if advance
        || ((check_lag || (r.read_u8(XVEL) | r.read_u8(YVEL)) != 0)
            && history_minus(r.read_u8(VAR1), 15) == r.read_u8(VAR2))
    {
        r.write_u8(VAR2, next_history(r.read_u8(VAR2)));
    }
    draw(r, hooks);
}

pub fn not_following<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if r.read_u8(SAVED_INDOORS) != r.read_u8(INDOORS) {
        return;
    }
    if r.read_u8(RUNNING) == 0 && !check_proximity_to_link(r) {
        initialize(r, hooks);
        r.write_u8(SAVED_INDOORS, r.read_u8(INDOORS));
        if r.read_u8(FOLLOWER) == 13 {
            r.write_u8(SUPER_BOMB_2, 254);
            r.write_u8(SUPER_BOMB_1, 0);
        }
        r.write_u8(DROPPED, 0);
        draw(r, hooks);
    } else {
        if r.read_u8(FOLLOWER) == 13 && r.read_u8(INDOORS) == 0 && r.read_u8(SUPER_BOMB_2) == 0 {
            if hooks.add_super_bomb_explosion(r, 0x3a, 0) >= 0 {
                r.write_u8(DROPPED, 0);
                if hooks.enhanced_features0() & 4096 != 0 {
                    r.write_u8(FOLLOWER, 0);
                    return;
                }
            } else {
                r.write_u8(SUPER_BOMB_1, 1);
            }
        }
        do_layers(r, hooks);
    }
}

pub fn old_man<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    if !is_following(r) {
        draw(r, hooks);
        return;
    }
    if r.read_u8(SPEED) != 4 {
        r.write_u8(SPEED, 12);
    }
    handle_trigger(r, hooks);
    match r.read_u8(FOLLOWER) {
        0 => return,
        4 => {
            let k = usize::from(r.read_u8(VAR2));
            if r.read_i8(Z + k) > 0 && r.read_u8(VAR1) != r.read_u8(VAR2) {
                r.write_u8(VAR2, next_history(r.read_u8(VAR2)));
                draw(r, hooks);
                return;
            }
        }
        _ => {
            let aux = r.read_u8(AUX_STATE);
            let recoil = aux & 1 != 0 && r.read_u8(PLAYER_STATE) == 6;
            if recoil && r.read_u8(VAR1) == r.read_u8(VAR2) {
                panic!("C port asserts because the transform X value is undefined");
            }
            if (recoil && r.read_u8(VAR1) != r.read_u8(VAR2)) || aux & 2 != 0 {
                let follower = usize::from(r.read_u8(FOLLOWER));
                r.write_u8(FOLLOWER, TAB4[follower]);
                r.write_u8(REACQUIRE, 64);
                let k = usize::from(r.read_u8(VAR2));
                r.write_u16(SAVED_Y, get16(r, YLO, YHI, k));
                r.write_u16(SAVED_X, get16(r, XLO, XHI, k));
                r.write_u8(SAVED_FLOOR, r.read_u8(LOWER));
                old_man_unused(r, hooks);
                return;
            }
        }
    }
    if r.read_u8(XVEL) | r.read_u8(YVEL) != 0 {
        if history_minus(r.read_u8(VAR1), 20) == r.read_u8(VAR2) {
            r.write_u8(VAR2, next_history(r.read_u8(VAR2)));
        }
    } else if r.read_u8(FRAME) & 3 == 0
        && r.read_u8(VAR1) != r.read_u8(VAR2)
        && history_minus(r.read_u8(VAR1), 9) != r.read_u8(VAR2)
    {
        r.write_u8(VAR2, next_history(r.read_u8(VAR2)));
    }
    draw(r, hooks);
}

pub fn old_man_unused<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    r.write_u8(SPEED, 16);
    if r.read_u8(RUNNING) == 0 && r.read_u8(AUX_STATE) == 0 && r.read_u8(PLAYER_STATE) != 4 {
        r.write_u8(SPEED, 0);
        if r.read_u8(PLAYER_STATE) != 19 && !check_proximity_to_link(r) {
            initialize(r, hooks);
            let follower = usize::from(r.read_u8(FOLLOWER));
            r.write_u8(FOLLOWER, TAB5[follower]);
            return;
        }
    }
    do_layers(r, hooks);
}

pub fn do_layers<H: TagalongHooks>(r: &mut Ram<'_>, hooks: &mut H) {
    r.write_u16(
        OAM_PRIORITY,
        u16::from(FLAGS[usize::from(r.read_u8(SAVED_FLOOR))]) << 8,
    );
    let a = if matches!(r.read_u8(FOLLOWER), 12 | 13) {
        2
    } else {
        1
    };
    animate_movement(r, hooks, a, r.read_u16(SAVED_X), r.read_u16(SAVED_Y));
}
fn get16(r: &Ram<'_>, lo: usize, hi: usize, k: usize) -> u16 {
    u16::from(r.read_u8(lo + k)) | u16::from(r.read_u8(hi + k)) << 8
}
fn set16(r: &mut Ram<'_>, lo: usize, hi: usize, k: usize, v: u16) {
    r.write_u8(lo + k, v as u8);
    r.write_u8(hi + k, (v >> 8) as u8)
}
fn abs(v: u16) -> u16 {
    if (v as i16) < 0 { v.wrapping_neg() } else { v }
}
fn ad(a: u16, b: u16) -> u16 {
    abs(a.wrapping_sub(b))
}
fn next_history(v: u8) -> u8 {
    if v.wrapping_add(1) == 20 {
        0
    } else {
        v.wrapping_add(1)
    }
}
fn history_minus(v: u8, amount: u8) -> u8 {
    let t = v.wrapping_sub(amount);
    if (t as i8) < 0 { t.wrapping_add(20) } else { t }
}
fn add_i8(v: u16, delta: i8) -> u16 {
    v.wrapping_add(i16::from(delta) as u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;
    struct Hooks {
        features: u32,
        spawn: i32,
        messages: usize,
        preps: usize,
    }
    impl Default for Hooks {
        fn default() -> Self {
            Self {
                features: 0,
                spawn: -1,
                messages: 0,
                preps: 0,
            }
        }
    }
    impl TagalongHooks for Hooks {
        fn enhanced_features0(&self) -> u32 {
            self.features
        }
        fn project_speed_towards_player(
            &mut self,
            _: &mut Ram<'_>,
            _: usize,
            _: u8,
        ) -> ProjectSpeed {
            ProjectSpeed::default()
        }
        fn ancilla_move_y(&mut self, _: &mut Ram<'_>, _: usize) {}
        fn ancilla_move_x(&mut self, _: &mut Ram<'_>, _: usize) {}
        fn show_text_message(&mut self, _: &mut Ram<'_>) {
            self.messages += 1;
        }
        fn add_super_bomb_explosion(&mut self, _: &mut Ram<'_>, _: u8, _: u8) -> i32 {
            -1
        }
        fn old_man_revert_to_sprite(&mut self, _: &mut Ram<'_>, _: usize) {}
        fn sprite_spawn_dynamically(&mut self, _: &mut Ram<'_>, _: usize, _: u8) -> i32 {
            self.spawn
        }
        fn sprite_prep_load_properties(&mut self, _: &mut Ram<'_>, _: usize) {
            self.preps += 1;
        }
    }
    #[test]
    fn initializes_exact_history() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        r.write_u16(LINK_X, 0x1234);
        r.write_u16(LINK_Y, 0xabcd);
        r.write_u8(LOWER, 1);
        r.write_u8(DIR, 6);
        initialize(&mut r, &Hooks::default());
        assert_eq!(get16(&r, XLO, XHI, 0), 0x1234);
        assert_eq!(get16(&r, YLO, YHI, 0), 0xabcd);
        assert_eq!(r.read_u8(LAYER), 7);
        assert_eq!(r.read_u8(REACQUIRE), 64)
    }
    #[test]
    fn proximity_countdown_matches_signed_branch() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        r.write_u16(SAVED_X, 100);
        r.write_u16(SAVED_Y, 200);
        r.write_u16(LINK_X, 100);
        r.write_u16(LINK_Y, 200);
        r.write_u8(REACQUIRE, 1);
        assert!(check_proximity_to_link(&mut r));
        r.write_u8(REACQUIRE, 0);
        assert!(!check_proximity_to_link(&mut r))
    }

    #[test]
    fn trigger_dispatch_sets_event_and_message() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut hooks = Hooks::default();
        r.write_u8(INDOORS, 1);
        r.write_u16(DUNGEON_ROOM, 0xf1);
        r.write_u8(FOLLOWER, 4);
        r.write_u16(LINK_Y, 0x1ef0 - 4);
        r.write_u16(LINK_X, 0x288 - 4);
        handle_trigger(&mut r, &mut hooks);
        assert_eq!(r.read_u8(EVENT_FLAGS), 1);
        assert_eq!(r.read_u16(DIALOGUE), 0x99);
        assert_eq!(hooks.messages, 1);
    }

    #[test]
    fn blind_conversion_writes_exact_sprite_slot_zero_fields() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut hooks = Hooks::default();
        blind_spawn_from_maiden(&mut r, &mut hooks, 0x345, 0x678);
        assert_eq!(r.read_u8(SPR_STATE), 9);
        assert_eq!(r.read_u8(SPR_TYPE), 206);
        assert_eq!(get16(&r, SPR_X_LO, SPR_X_HI, 0), 0x345);
        assert_eq!(get16(&r, SPR_Y_LO, SPR_Y_HI, 0), 0x668);
        assert_eq!(r.read_u8(SPR_DELAY_AUX2), 192);
        assert_eq!(r.read_u16(DUNG_SAVE_BITS), 0x2000);
        assert_eq!(hooks.preps, 1);
    }

    #[test]
    fn kiki_spawn_preserves_double_floor_write_quirk() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut hooks = Hooks {
            spawn: 3,
            ..Hooks::default()
        };
        set16(&mut r, XLO, XHI, 7, 0x1111);
        set16(&mut r, YLO, YHI, 7, 0x2222);
        r.write_u8(LAYER + 7, 0x23);
        r.write_u8(LOWER, 1);
        assert_eq!(kiki_spawn_handler_monke(&mut r, &mut hooks, 7), 3);
        assert_eq!(get16(&r, SPR_X_LO, SPR_X_HI, 3), 0x1113);
        assert_eq!(get16(&r, SPR_Y_LO, SPR_Y_HI, 3), 0x2224);
        assert_eq!(r.read_u8(SPR_HEAD_DIR + 3), 3);
        assert_eq!(r.read_u8(SPR_FLOOR + 3), 2);
    }

    #[test]
    fn stationary_draw_uses_c_oam_offsets_and_dma() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut hooks = Hooks::default();
        r.write_u8(FOLLOWER, 1);
        r.write_u16(LINK_Y, 0x80);
        animate_movement(&mut r, &mut hooks, 0, 0x40, 0x80);
        let head = oam::read(&r, 92);
        let body = oam::read(&r, 93);
        assert_eq!(
            (head.x, head.y, head.charnum, head.flags),
            (0x40, 0x7f, 0x20, 8)
        );
        assert_eq!(
            (body.x, body.y, body.charnum, body.flags),
            (0x40, 0x89, 0x22, 0x48)
        );
        assert_eq!(r.read_u8(DMA_VAR6), 0x20);
        assert_eq!(r.read_u8(DMA_VAR7), 0xc0);
    }

    #[test]
    fn hookshot_uses_lag_gate_before_advancing_history() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut hooks = Hooks::default();
        r.write_u8(FOLLOWER, 1);
        r.write_u8(PLAYER_STATE, 19);
        r.write_u8(HOOKSHOT, 1);
        r.write_u8(VAR1, 0);
        r.write_u8(VAR2, 2);

        basic_mover(&mut r, &mut hooks);

        assert_eq!(r.read_u8(VAR3), 1);
        assert_eq!(r.read_u8(VAR2), 2);

        // history_minus(17, 15) == 2, so the same hookshot branch now reaches
        // C's label_d and advances exactly once.
        r.write_u8(VAR1, 17);
        basic_mover(&mut r, &mut hooks);
        assert_eq!(r.read_u8(VAR2), 3);
    }

    #[test]
    fn negative_history_sentinel_only_normalizes_draw_coordinates() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut hooks = Hooks::default();
        r.write_u8(FOLLOWER, 1);
        r.write_u8(VAR2, 0x80);
        r.write_u8(Z + 0x80, 1);
        set16(&mut r, XLO, XHI, 0, 0x40);
        set16(&mut r, YLO, YHI, 0, 0x80);
        r.write_u16(LINK_Y, 0x80);

        draw(&mut r, &mut hooks);

        assert_eq!(r.read_u16(OAM_PRIORITY), 0x2000);
        assert_eq!(oam::read(&r, 92).x, 0x40);
    }
}
