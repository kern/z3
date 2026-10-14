//! One-for-one translation of `src/sprite_main.c` definitions beginning on
//! source lines 14001 through 20000.
//!
//! State owned by the sprite bank is mutated directly in WRAM. Operations
//! implemented by another translation unit cross the explicit runtime seam.

#![allow(dead_code)]
#![allow(
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::manual_range_contains,
    clippy::needless_range_loop,
    clippy::precedence,
    clippy::possible_missing_else
)]

use crate::ram::Ram;
use crate::sprite::PrepOamCoordsRet;
use crate::sprite_main_bank0::{
    GANON_DRAW_CHARACTER, GANON_DRAW_CHARACTER_2, GANON_DRAW_FLAGS, GANON_DRAW_FLAGS_2,
    GANON_DRAW_X, GANON_DRAW_Y, LARGE_SHADOW_DRAW_DATA, SINUS_LOOKUP_TABLE,
};

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
const SPRITE_DELAY_AUX2: usize = 0xe10;
const SPRITE_TYPE: usize = 0xe20;
const SPRITE_SUBTYPE: usize = 0xe30;
const SPRITE_FLAGS2: usize = 0xe40;
const SPRITE_HEALTH: usize = 0xe50;
const SPRITE_FLAGS3: usize = 0xe60;
const SPRITE_WALLCOLL: usize = 0xe70;
const SPRITE_SUBTYPE2: usize = 0xe80;
const SPRITE_E: usize = 0xe90;
const SPRITE_F: usize = 0xea0;
const SPRITE_HEAD_DIR: usize = 0xeb0;
const SPRITE_ANIM_CLOCK: usize = 0xec0;
const SPRITE_G: usize = 0xed0;
const SPRITE_DELAY_AUX3: usize = 0xee0;
const SPRITE_HIT_TIMER: usize = 0xef0;
const SPRITE_PAUSE: usize = 0xf00;
const SPRITE_DELAY_AUX4: usize = 0xf10;
const SPRITE_FLOOR: usize = 0xf20;
const SPRITE_Y_RECOIL: usize = 0xf30;
const SPRITE_X_RECOIL: usize = 0xf40;
const SPRITE_OAM_FLAGS: usize = 0xf50;
const SPRITE_FLAGS4: usize = 0xf60;
const SPRITE_Z: usize = 0xf70;
const SPRITE_Z_VEL: usize = 0xf80;
const SPRITE_OBJ_PRIO: usize = 0xb89;
const SPRITE_ROOM: usize = 0xc9a;
const SPRITE_FLAGS: usize = 0xb6b;
const SPRITE_IGNORE_PROJECTILE: usize = 0xba0;
const SPRITE_FLAGS5: usize = 0xbe0;
const SPRITE_DEFL_BITS: usize = 0xcaa;
const SPRITE_BUMP_DAMAGE: usize = 0xcd2;

const OVERLORD_X_LO: usize = 0xb08;
const OVERLORD_TYPE: usize = 0xb00;
const OVERLORD_X_HI: usize = 0xb10;
const OVERLORD_Y_LO: usize = 0xb18;
const OVERLORD_Y_HI: usize = 0xb20;
const OVERLORD_GEN2: usize = 0xb30;
const OVERLORD_GEN1: usize = 0xb28;
const OVERLORD_FLOOR: usize = 0xb40;

const FRAME_COUNTER: usize = 0x1a;
const PLAYER_IS_INDOORS: usize = 0x1b;
const LINK_Y: usize = 0x20;
const LINK_X: usize = 0x22;
const LINK_IS_ON_LOWER_LEVEL: usize = 0xee;
const OAM_CUR_PTR: usize = 0x90;
const OAM_EXT_CUR_PTR: usize = 0x92;
const SOUND_EFFECT_2: usize = 0x12f;
const SOUND_EFFECT_1: usize = 0x12e;
const CUR_SPRITE_X: usize = 0xfd8;
const CUR_SPRITE_Y: usize = 0xfda;
const DUNG_SAVEGAME_STATE_BITS: usize = 0x402;
const CURRENT_DUNGEON_ROOM: usize = 0x4c5;
const FLAG_IS_LINK_IMMOBILIZED: usize = 0x2e4;
const MUSIC_CONTROL: usize = 0x12c;
const SOUND_EFFECT_AMBIENT: usize = 0x12d;
const BG1_Y_OFFSET: usize = 0x11c;
const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;
const LINK_SHIELD_TYPE: usize = 0xf35a;
const LINK_TIMER_TEMPBUNNY: usize = 0x3f5;
const GARNISH_COUNTDOWN: usize = 0x1f90e;
const TMP_COUNTER: usize = 0xfb5;
const SWAMOLA_TARGET_X_LO: usize = 0x1fd5c;
const SWAMOLA_TARGET_Y_LO: usize = 0x1fd68;
const SWAMOLA_TARGET_X_HI: usize = 0x1fd62;
const SWAMOLA_TARGET_Y_HI: usize = 0x1fd6e;
const SWAMOLA_X_LO: usize = 0x1fa5c;
const SWAMOLA_X_HI: usize = 0x1fb1c;
const SWAMOLA_Y_LO: usize = 0x1fbdc;
const SWAMOLA_Y_HI: usize = 0x1fc9c;
const ENHANCED_FEATURES0: usize = 0x64c;
const BYTE_7E0FB6: usize = 0xfb6;
const SPRITE_STUNNED: usize = 0xb58;
const SPRITE_Z_SUBPOS: usize = 0xf90;
const SPRITE_LIMIT_INSTANCE: usize = 0xb6a;
const BYTE_7E0B69: usize = 0xb69;
const FOLLOWER_INDICATOR: usize = 0xf3cc;
const GARNISH_ACTIVE: usize = 0xfb4;
const GARNISH_TYPE: usize = 0x1f800;
const GARNISH_Y_LO: usize = 0x1f81e;
const GARNISH_X_LO: usize = 0x1f83c;
const GARNISH_Y_HI: usize = 0x1f85a;
const GARNISH_X_HI: usize = 0x1f878;
const GARNISH_SPRITE: usize = 0x1f92c;
const GARNISH_OAM_FLAGS: usize = 0x1f9fe;
const COUNTDOWN_FOR_BLINK: usize = 0x31f;
const LINK_DISABLE_SPRITE_DAMAGE: usize = 0x37b;
const LINK_AUXILIARY_STATE: usize = 0x4d;
const LINK_GIVE_DAMAGE: usize = 0x373;
const LINK_INCAPACITATED_TIMER: usize = 0x46;
const LINK_ACTUAL_VEL_X: usize = 0x28;
const LINK_ACTUAL_VEL_Y: usize = 0x27;
const LOAD_CHR_HALFSLOT_EVEN_ODD: usize = 0xaaa;
const IS_IN_DARK_WORLD: usize = 0xfff;
const CHAINCHOMP_X_HIST: usize = 0x1fc00;
const CHAINCHOMP_Y_HIST: usize = 0x1fd00;
const NUM_SPRITES_KILLED: usize = 0xcfb;
const NUMBER_OF_TIMES_HURT_BY_SPRITES: usize = 0xcfc;
const LINK_HEARTS_FILLER: usize = 0xf372;
const LINK_HEALTH_CAPACITY: usize = 0xf36c;
const LINK_HEALTH_CURRENT: usize = 0xf36d;
const SUBMODULE_INDEX: usize = 0x11;
const FLAG_UNK1: usize = 0xfc1;
const ALT_SPRITE_TYPE: usize = 0x1d10;
const ALT_SPRITE_X_HI: usize = 0x1d30;
const ALT_SPRITE_Y_HI: usize = 0x1d50;
const INTRO_TIMES_PAL_FLASH: usize = 0xff9;
const DMA_VAR6: usize = 0xae8;
const DMA_VAR7: usize = 0xaea;
const LINK_ITEM_BOMBS: usize = 0xf343;
const LINK_RUPEES_GOAL: usize = 0xf360;
const LINK_NUM_ARROWS: usize = 0xf377;
const BUTTON_B_FRAMES: usize = 0x3c;
const BG2VOFS_COPY2: usize = 0xe8;
const DUNGEON_ROOM_INDEX2: usize = 0x48e;
const OVERWORLD_SCREEN_INDEX: usize = 0x8a;
const LINK_ELECTROCUTE_ON_TOUCH: usize = 0x360;
const LINK_SWORD_TYPE: usize = 0xf359;
const BG2HOFS_COPY2: usize = 0xe2;
const MOLDORM_X_LO: usize = 0x1fc00;
const MOLDORM_X_HI: usize = 0x1fc80;
const MOLDORM_Y_LO: usize = 0x1fd00;
const MOLDORM_Y_HI: usize = 0x1fd80;
const ITEM_RECEIPT_METHOD: usize = 0x2e9;
const BG1_X_OFFSET: usize = 0x11a;
const LINK_ITEM_QUAKE_MEDALLION: usize = 0xf349;
const BYTE_7E0FF8: usize = 0xff8;
const CHOICE_IN_MULTISELECT_BOX: usize = 0x1ce8;
const LINK_POSITION_MODE: usize = 0x37a;
const IS_ARCHER_OR_SHOVEL_GAME: usize = 0x3fc;
const SUPER_BOMB_INDICATOR_UNK2: usize = 0x4b4;
const SUPER_BOMB_INDICATOR_UNK1: usize = 0x4b5;
const BEAMOS_X_HI: usize = 0x1fe00;
const PLAYER_OAM_Y_OFFSET: usize = 0x44;
const LINK_IS_RUNNING: usize = 0x372;
const LINK_ITEM_IN_HAND: usize = 0x301;
const REPULSESPARK_TIMER: usize = 0xfac;
const REPULSESPARK_X_LO: usize = 0xfad;
const REPULSESPARK_Y_LO: usize = 0xfae;
const ANCILLA_Z: usize = 0x29e;
const ANCILLA_Y_LO: usize = 0xbfa;
const ANCILLA_X_LO: usize = 0xc04;
const ANCILLA_Y_VEL: usize = 0xc22;
const ANCILLA_X_VEL: usize = 0xc2c;
const TM_COPY: usize = 0x1c;
const TS_COPY: usize = 0x1d;
const DUNG_FLOOR_Y_VEL: usize = 0x310;
const DUNG_FLOOR_X_VEL: usize = 0x312;
const DUNG_HDR_COLLISION_2_MIRROR: usize = 0x428;
const FLAG_BLOCK_LINK_MENU: usize = 0xffc;

type DrawPiece = (i8, i8, u16, u8);

pub trait SpriteMainBank2aRuntime {
    fn return_if_inactive(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn return_if_recoiling(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn move_xy(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_y(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_x(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_z(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn check_tile_collision(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_damage_to_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_damage_to_and_from_link(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn direction_to_face_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn direction_to_face_link_with_differences(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
    ) -> (u8, u8) {
        (0, 0)
    }
    fn direction_to_face_location(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _x: u16,
        _y: u16,
    ) -> u8 {
        0
    }
    fn apply_speed_towards_link(&mut self, _ram: &mut Ram<'_>, _k: usize, _speed: u8) {}
    fn project_speed_towards_link(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _speed: u8,
    ) -> (u8, u8) {
        (0, 0)
    }
    fn project_speed_towards_location(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _x: u16,
        _y: u16,
        _speed: u8,
    ) -> (u8, u8) {
        (0, 0)
    }
    fn approach_target_speed(&mut self, _ram: &mut Ram<'_>, _k: usize, _x: u8, _y: u8) {}
    fn random_number(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
    fn spawn_dynamic(&mut self, _ram: &mut Ram<'_>, _source: usize, _kind: u8) -> Option<usize> {
        None
    }
    fn spawn_dynamic_ex(
        &mut self,
        _ram: &mut Ram<'_>,
        _source: usize,
        _kind: u8,
        _slot: usize,
    ) -> Option<usize> {
        None
    }
    fn set_spawned_coordinates(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn spawn_fire_phlegm(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<usize> {
        None
    }
    fn spawn_boss_poof(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<usize> {
        None
    }
    fn spawn_poof_garnish(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn garnish_flame_trail(&mut self, _ram: &mut Ram<'_>, _k: usize, _low: bool) -> usize {
        0
    }
    fn sfx2_with_pan(&mut self, _ram: &mut Ram<'_>, _k: usize, _sound: u8) {}
    fn sfx3_with_pan(&mut self, _ram: &mut Ram<'_>, _k: usize, _sound: u8) {}
    fn sfx1_with_pan(&mut self, _ram: &mut Ram<'_>, _k: usize, _sound: u8) {}
    fn kill_friends(&mut self, _ram: &mut Ram<'_>) {}
    fn spawn_probe_always(&mut self, _ram: &mut Ram<'_>, _k: usize, _direction: u8) {}
    fn prep_oam(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn prep_oam_or_double_ret(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<(u8, u8, u8)> {
        None
    }
    fn prep_oam_or_double_ret_wide(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
    ) -> Option<(u16, u16, u8)> {
        None
    }
    fn draw_multiple(&mut self, _ram: &mut Ram<'_>, _k: usize, _pieces: &[DrawPiece]) {}
    fn correct_oam_entries(&mut self, _ram: &mut Ram<'_>, _k: usize, _count: usize, _size: u8) {}
    fn last_draw_info(&self) -> PrepOamCoordsRet {
        PrepOamCoordsRet::default()
    }
    fn draw_shadow(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_shadow_custom(&mut self, _ram: &mut Ram<'_>, _k: usize, _size: u8) {}
    fn antfairy_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn antfairy_retile(&mut self, _ram: &mut Ram<'_>, _chars: u8) {}
    fn emit_oam(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _x: u8,
        _y: u8,
        _char: u8,
        _flags: u8,
        _size: u8,
    ) {
    }
    fn get_16bit_coords(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn is_right_of_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> (u8, i8) {
        (0, 0)
    }
    fn show_message_minimal(&mut self, _ram: &mut Ram<'_>) {}
    fn extinguish_torch(&mut self, _ram: &mut Ram<'_>, _adjust_translucency: bool) {}
    fn convert_velocity_to_angle(&mut self, _x: u8, _y: u8) -> u8 {
        0
    }
    fn ganon_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn oam_allocate_region_b(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn draw_single_large(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn garnish_alloc_overwrite_old(&mut self, _ram: &mut Ram<'_>) -> usize {
        0
    }
    fn spawn_fireball(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<usize> {
        None
    }
    fn check_damage_to_link_same_layer(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn calculate_sfx_pan(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn set_current_oam_char_flags(&mut self, _ram: &mut Ram<'_>, _character: u8, _high_flags: u8) {}
    fn set_oam_char_flags(
        &mut self,
        _ram: &mut Ram<'_>,
        _entry: usize,
        _character: u8,
        _high_flags: u8,
    ) {
    }
    fn set_current_oam_y(&mut self, _ram: &mut Ram<'_>, _entry: usize, _y: u8) {}
    fn garnish_alloc_overwrite_old_low(&mut self, _ram: &mut Ram<'_>) -> usize {
        0
    }
    fn move_xyz(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bounce_from_tile_collision(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn draw_single_small(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn attempt_damage_to_link_plus_recoil(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_big_shadow(&mut self, _ram: &mut Ram<'_>, _k: usize, _animation: u8) {}
    fn load_properties(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn transmute_to_bomb(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn decode_animated_sprite_tile(&mut self, _ram: &mut Ram<'_>, _tile: u8) {}
    fn draw_multiple_player_deferred(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _pieces: &[DrawPiece],
    ) {
    }
    fn garnish_spawn_sparkle(&mut self, _ram: &mut Ram<'_>, _k: usize, _x: i8, _y: i8) -> usize {
        0
    }
    fn link_cancel_dash(&mut self, _ram: &mut Ram<'_>) {}
    fn force_prize_drop(&mut self, _ram: &mut Ram<'_>, _k: usize, _a: u8, _b: u8) {}
    fn check_damage_from_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn show_solicited_message(&mut self, _ram: &mut Ram<'_>, _k: usize, _msg: u16) -> u16 {
        0
    }
    fn show_message_unconditional(&mut self, _ram: &mut Ram<'_>, _msg: u16) {}
    fn giant_moldorm_draw_segment(&mut self, _ram: &mut Ram<'_>, _k: usize, _offset: u8) {}
    fn giant_moldorm_draw_segment_ab(&mut self, _ram: &mut Ram<'_>, _k: usize, _lookback: u8) {}
    fn draw_water_ripple_with_oam_adjust(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn oam_allocate_region_c(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn oam_allocate_region_a(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn link_receive_item(&mut self, _ram: &mut Ram<'_>, _item: u8, _method: u8) {}
    fn behave_as_barrier(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn initialize_secondary_item_minigame(&mut self, _ram: &mut Ram<'_>, _which: u8) {}
    fn pikit_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bomber_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_zirro_bomb(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn zirro_drop_bomb(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn helmasaur_hammer_overlap(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn helmasaur_bomb_overlap(&mut self, _ram: &mut Ram<'_>, _k: usize, _j: usize) -> bool {
        false
    }
    fn manually_set_death_flag_uw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn is_below_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn trinexx_flash_shell_palette(&mut self, _ram: &mut Ram<'_>, _blue: bool, _flash: bool) {}
}

pub struct SpriteMainBank2a<'a, 'r, R: SpriteMainBank2aRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
}

impl<R: SpriteMainBank2aRuntime> SpriteMainBank2a<'_, '_, R> {
    pub fn dispatch_active(&mut self, k: usize, kind: u8) -> bool {
        match kind {
            0x00 => self.sprite_raven(k),
            0x92 => self.sprite_92_helmasaur_king(k),
            0xa8 | 0xa9 => self.sprite_a8_green_zirro(k),
            0xaa => self.sprite_aa_pikit(k),
            0xbd => self.sprite_bd_vitreous(k),
            0xbe => self.sprite_be_vitreous_eye(k),
            0xbf => self.sprite_bf_lightning(k),
            0xc0 => self.sprite_c0_catfish(k),
            0xc1 => self.sprite_c1_cutscene_agahnim(k),
            0xc2 => self.sprite_c2_boulder(k),
            0xc3 => self.sprite_c3_gibo(k),
            0xc4 => self.sprite_c4_thief(k),
            0xc5 => self.sprite_c5_medusa(k),
            0xc6 => self.sprite_c6_4_way_shooter(k),
            0xc7 => self.sprite_c7_pokey(k),
            0xc8 => self.sprite_c8_big_fairy(k),
            0xc9 => self.sprite_c9_tektite(k),
            0xca => self.sprite_ca_chain_chomp(k),
            0xcb => self.sprite_cb_trinexx_rock_head(k),
            0xce => self.sprite_ce_blind(k),
            0xcf => self.sprite_cf_swamola(k),
            0xd0 => self.sprite_d0_lynel(k),
            0xd1 => self.sprite_d1_bunny_beam(k),
            0xd5 => self.sprite_d5_dig_game_guy(k),
            0xd6 | 0xd7 => self.sprite_d6_ganon(k),
            _ => return false,
        }
        true
    }
    pub fn dispatch_prep(&mut self, k: usize, kind: u8) -> bool {
        match kind {
            0xce => self.sprite_prep_blind_prepare_battle(k),
            0xcf => {
                self.sprite_prep_swamola_initialize_segments(k);
                self.ram
                    .write_u8(SPRITE_A + k, self.ram.read_u8(SPRITE_X_LO + k));
                self.ram
                    .write_u8(SPRITE_B + k, self.ram.read_u8(SPRITE_X_HI + k));
                self.ram
                    .write_u8(SPRITE_C + k, self.ram.read_u8(SPRITE_Y_LO + k));
                self.ram
                    .write_u8(SPRITE_HEAD_DIR + k, self.ram.read_u8(SPRITE_Y_HI + k));
            }
            _ => return false,
        }
        true
    }

    #[inline]
    fn sprite_x(&self, k: usize) -> u16 {
        u16::from_le_bytes([
            self.ram.read_u8(SPRITE_X_LO + k),
            self.ram.read_u8(SPRITE_X_HI + k),
        ])
    }
    #[inline]
    fn sprite_y(&self, k: usize) -> u16 {
        u16::from_le_bytes([
            self.ram.read_u8(SPRITE_Y_LO + k),
            self.ram.read_u8(SPRITE_Y_HI + k),
        ])
    }
    #[inline]
    fn set_sprite_x(&mut self, k: usize, v: u16) {
        let [lo, hi] = v.to_le_bytes();
        self.ram.write_u8(SPRITE_X_LO + k, lo);
        self.ram.write_u8(SPRITE_X_HI + k, hi);
    }
    #[inline]
    fn set_sprite_y(&mut self, k: usize, v: u16) {
        let [lo, hi] = v.to_le_bytes();
        self.ram.write_u8(SPRITE_Y_LO + k, lo);
        self.ram.write_u8(SPRITE_Y_HI + k, hi);
    }

    pub fn fish_draw(&mut self, k: usize) {
        const BODY: [DrawPiece; 16] = [
            (-4, 8, 0x045e, 0),
            (4, 8, 0x045f, 0),
            (-4, 8, 0x845e, 0),
            (4, 8, 0x845f, 0),
            (-4, 8, 0x445f, 0),
            (4, 8, 0x445e, 0),
            (-4, 8, 0xc45f, 0),
            (4, 8, 0xc45e, 0),
            (0, 0, 0x0461, 0),
            (0, 8, 0x0471, 0),
            (0, 0, 0x4461, 0),
            (0, 8, 0x4471, 0),
            (0, 0, 0x8471, 0),
            (0, 8, 0x8461, 0),
            (0, 0, 0xc471, 0),
            (0, 8, 0xc461, 0),
        ];
        const RIPPLE: [DrawPiece; 9] = [
            (-2, 11, 0x0438, 0),
            (0, 11, 0x0438, 0),
            (2, 11, 0x0438, 0),
            (-1, 11, 0x0438, 0),
            (0, 11, 0x0438, 0),
            (1, 11, 0x0438, 0),
            (0, 11, 0x0438, 0),
            (0, 11, 0x0438, 0),
            (0, 11, 0x0438, 0),
        ];
        let g = self.ram.read_u8(SPRITE_GRAPHICS + k);
        if g == 0 {
            self.runtime.prep_oam(self.ram, k);
            return;
        }
        self.ram.write_u16(
            CUR_SPRITE_X,
            self.ram.read_u16(CUR_SPRITE_X).wrapping_add(4),
        );
        self.runtime.draw_multiple(
            self.ram,
            k,
            &BODY[(g as usize - 1) * 2..(g as usize - 1) * 2 + 2],
        );
        self.ram.write_u16(
            CUR_SPRITE_Y,
            self.ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_add(self.ram.read_u8(SPRITE_Z + k) as u16),
        );
        let j = (self.ram.read_u8(SPRITE_Z + k) >> 2).min(2) as usize;
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(8));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(2),
        );
        self.runtime
            .draw_multiple(self.ram, k, &RIPPLE[j * 3..j * 3 + 3]);
        self.runtime.get_16bit_coords(self.ram, k);
    }

    pub fn chimney_smoke_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (0, 0, 0x0086, 0),
            (8, 0, 0x0087, 0),
            (0, 8, 0x0096, 0),
            (8, 8, 0x0097, 0),
            (1, 1, 0x0086, 0),
            (7, 1, 0x0087, 0),
            (1, 7, 0x0096, 0),
            (7, 7, 0x0097, 0),
        ];
        let i = (self.ram.read_u8(SPRITE_GRAPHICS + k) as usize & 1) * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
    }

    pub fn sprite_d1_bunny_beam(&mut self, k: usize) {
        if self.ram.read_u8(PLAYER_IS_INDOORS) != 0 {
            self.sprite_bunny_beam(k)
        } else {
            self.sprite_chimney(k)
        }
    }

    pub fn sprite_chimney(&mut self, k: usize) {
        self.ram.write_u8(SPRITE_FLAGS3 + k, 64);
        self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + k, 64);
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 0 {
            if self.runtime.return_if_inactive(self.ram, k)
                || self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0
            {
                return;
            }
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 67);
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xd1) {
                self.runtime.set_spawned_coordinates(self.ram, j);
                let t = self.ram.read_u8(SPRITE_X_LO + j).wrapping_add(8);
                let carry = u8::from(t < 8);
                self.ram.write_u8(SPRITE_X_LO + j, t);
                self.ram.write_u8(
                    SPRITE_Y_LO + j,
                    self.ram
                        .read_u8(SPRITE_Y_LO + j)
                        .wrapping_add(4)
                        .wrapping_add(carry),
                );
                self.ram.write_u8(SPRITE_OAM_FLAGS + j, 4);
                self.ram.write_u8(SPRITE_AI_STATE + j, 4);
                self.ram.write_u8(SPRITE_FLAGS2 + j, 67);
                self.ram.write_u8(SPRITE_FLAGS3 + j, 67);
                self.ram.write_u8(SPRITE_X_VEL + j, (-4i8) as u8);
                self.ram.write_u8(SPRITE_Y_VEL + j, (-6i8) as u8);
            }
        } else {
            self.ram.write_u8(SPRITE_OBJ_PRIO + k, 0x30);
            self.chimney_smoke_draw(k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.runtime.move_xy(self.ram, k);
            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
            if s & 7 == 0 {
                let d = self.ram.read_u8(SPRITE_D + k) & 1;
                let v =
                    self.ram
                        .read_u8(SPRITE_X_VEL + k)
                        .wrapping_add(if d != 0 { 255 } else { 1 });
                self.ram.write_u8(SPRITE_X_VEL + k, v);
                if v == if d != 0 { (-4i8) as u8 } else { 4 } {
                    self.ram
                        .write_u8(SPRITE_D + k, self.ram.read_u8(SPRITE_D + k).wrapping_add(1));
                }
            }
            if s & 31 == 0 {
                self.ram.write_u8(
                    SPRITE_GRAPHICS + k,
                    self.ram.read_u8(SPRITE_GRAPHICS + k).wrapping_add(1),
                );
            }
        }
    }

    pub fn sprite_bunny_beam(&mut self, k: usize) {
        const GFX: [u8; 6] = [0xd7, 0xd7, 0xd7, 0x91, 0x91, 0x91];
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 0 {
            self.runtime.prep_oam(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            if !self.runtime.check_tile_collision(self.ram, k) {
                self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
            }
            return;
        }
        self.runtime.antfairy_draw(self.ram, k);
        if self.ram.read_u8(SPRITE_PAUSE + k) == 0 {
            self.runtime.antfairy_retile(
                self.ram,
                GFX[self.ram.read_u8(SPRITE_GRAPHICS + k) as usize],
            );
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
            self.ram.write_u8(SPRITE_BUMP_DAMAGE + k, 0x30);
            if self.runtime.check_damage_to_link(self.ram, k) {
                self.ram.write_u8(SPRITE_STATE + k, 0);
                self.ram.write_u16(LINK_TIMER_TEMPBUNNY, 256);
            }
            if self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL) == self.ram.read_u8(SPRITE_FLOOR + k) {
                self.runtime.apply_speed_towards_link(self.ram, k, 16);
            }
            self.runtime.move_xy(self.ram, k);
            if self.runtime.check_tile_collision(self.ram, k) {
                self.ram.write_u8(SPRITE_STATE + k, 0);
                self.runtime.spawn_poof_garnish(self.ram, k);
                self.runtime.sfx2_with_pan(self.ram, k, 0x15);
            }
        }
    }

    pub fn sprite_d0_lynel(&mut self, k: usize) {
        const ATTACK: [u8; 4] = [5, 2, 8, 10];
        const GFX: [u8; 8] = [3, 0, 6, 9, 4, 1, 7, 10];
        self.lynel_draw(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        let d = self.runtime.direction_to_face_link(self.ram, k);
        self.ram.write_u8(SPRITE_D + k, d);
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    const XT: [i8; 4] = [-96, 96, 0, 0];
                    const YT: [i8; 4] = [8, 8, -96, 112];
                    let x = self
                        .ram
                        .read_u16(LINK_X)
                        .wrapping_add_signed(XT[d as usize] as i16);
                    let y = self
                        .ram
                        .read_u16(LINK_Y)
                        .wrapping_add_signed(YT[d as usize] as i16);
                    let [a, b] = x.to_le_bytes();
                    let [c, e] = y.to_le_bytes();
                    self.ram.write_u8(SPRITE_A + k, a);
                    self.ram.write_u8(SPRITE_B + k, b);
                    self.ram.write_u8(SPRITE_C + k, c);
                    self.ram.write_u8(SPRITE_E + k, e);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 80);
                }
                let i = ((self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 4) | d) as usize;
                self.ram.write_u8(SPRITE_GRAPHICS + k, GFX[i]);
            }
            1 => {
                let mut advance = self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0;
                if !advance {
                    if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0 {
                        let x = u16::from_le_bytes([
                            self.ram.read_u8(SPRITE_A + k),
                            self.ram.read_u8(SPRITE_B + k),
                        ]);
                        let y = u16::from_le_bytes([
                            self.ram.read_u8(SPRITE_C + k),
                            self.ram.read_u8(SPRITE_E + k),
                        ]);
                        advance = x
                            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                            .wrapping_add(5)
                            < 10
                            && y.wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                                .wrapping_add(5)
                                < 10;
                        if !advance {
                            let (xv, yv) = self
                                .runtime
                                .project_speed_towards_location(self.ram, k, x, y, 24);
                            self.ram.write_u8(SPRITE_X_VEL + k, xv);
                            self.ram.write_u8(SPRITE_Y_VEL + k, yv);
                        }
                    }
                    if !advance {
                        self.runtime.move_xy(self.ram, k);
                        advance = self.runtime.check_tile_collision(self.ram, k);
                        if !advance {
                            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
                            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
                            self.ram
                                .write_u8(SPRITE_GRAPHICS + k, GFX[((s & 4) | d) as usize]);
                        }
                    }
                }
                if advance {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                }
            }
            2 => {
                let delay = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if delay == 0 {
                    let next = (self.runtime.random_number(self.ram) & 15) + 16;
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, next);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    return;
                }
                if delay == 16 {
                    if let Some(j) = self.runtime.spawn_fire_phlegm(self.ram, k) {
                        if self.ram.read_u8(LINK_SHIELD_TYPE) != 3 {
                            self.ram.write_u8(SPRITE_FLAGS5 + j, 0);
                        }
                    }
                }
                self.ram.write_u8(SPRITE_GRAPHICS + k, ATTACK[d as usize]);
                let _ = self.runtime.check_tile_collision(self.ram, k);
            }
            _ => {}
        }
    }

    pub fn lynel_draw(&mut self, k: usize) {
        const D: [DrawPiece; 33] = [
            (-5, -11, 0x00cc, 2),
            (-4, 0, 0x00e4, 2),
            (4, 0, 0x00e5, 2),
            (-5, -10, 0x00cc, 2),
            (-4, 0, 0x00e7, 2),
            (4, 0, 0x00e8, 2),
            (-5, -11, 0x00c8, 2),
            (-4, 0, 0x00e4, 2),
            (4, 0, 0x00e5, 2),
            (5, -11, 0x40cc, 2),
            (-4, 0, 0x40e5, 2),
            (4, 0, 0x40e4, 2),
            (5, -10, 0x40cc, 2),
            (-4, 0, 0x40e8, 2),
            (4, 0, 0x40e7, 2),
            (5, -11, 0x40c8, 2),
            (-4, 0, 0x40e8, 2),
            (4, 0, 0x40e7, 2),
            (0, -9, 0x00ce, 2),
            (-4, 0, 0x00ea, 2),
            (4, 0, 0x00eb, 2),
            (0, -9, 0x00ce, 2),
            (-4, 0, 0x40eb, 2),
            (4, 0, 0x40ea, 2),
            (0, -9, 0x00ca, 2),
            (-4, 0, 0x40eb, 2),
            (4, 0, 0x00eb, 2),
            (0, -14, 0x00c6, 2),
            (-4, 0, 0x00ed, 2),
            (4, 0, 0x00ee, 2),
            (0, -14, 0x00c6, 2),
            (-4, 0, 0x40ee, 2),
            (4, 0, 0x40ed, 2),
        ];
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 3;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 3]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_spawn_phantom_ganon(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc9) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 2);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 2);
            self.ram.write_u8(SPRITE_ANIM_CLOCK + j, 1);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, 0);
        }
    }

    pub fn sprite_phantom_ganon(&mut self, k: usize) {
        const GFX: [u8; 4] = [0, 1, 2, 1];
        const XV: [u8; 2] = [32, (-32i8) as u8];
        const YV: [u8; 2] = [16, (-16i8) as u8];
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 0 {
            self.phantom_ganon_draw(k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.runtime.move_y(self.ram, k);
            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
            if s & 31 == 0 {
                let yv = self.ram.read_u8(SPRITE_Y_VEL + k).wrapping_sub(1);
                self.ram.write_u8(SPRITE_Y_VEL + k, yv);
                if yv == 252 {
                    if let Some(j) = self.runtime.spawn_boss_poof(self.ram, k) {
                        self.set_sprite_y(j, self.sprite_y(j).wrapping_sub(20));
                    }
                } else if yv == 251 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                    self.ram.write_u8(SPRITE_Y_VEL + k, (-4i8) as u8);
                }
            }
        } else {
            self.ganon_bat_draw(k);
            if self.ram.read_u8(SPRITE_PAUSE + k) != 0 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
                self.ram.write_u16(
                    DUNG_SAVEGAME_STATE_BITS,
                    self.ram.read_u16(DUNG_SAVEGAME_STATE_BITS) | 0x8000,
                );
            }
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.ram.write_u8(
                SPRITE_GRAPHICS + k,
                GFX[(self.ram.read_u8(FRAME_COUNTER) >> 2 & 3) as usize],
            );
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) < 208 {
                    let j = (self.ram.read_u8(SPRITE_HEAD_DIR + k) & 1) as usize;
                    let y = self.ram.read_u8(SPRITE_Y_VEL + k).wrapping_add(if j != 0 {
                        255
                    } else {
                        1
                    });
                    self.ram.write_u8(SPRITE_Y_VEL + k, y);
                    if y == YV[j] {
                        self.ram.write_u8(
                            SPRITE_HEAD_DIR + k,
                            self.ram.read_u8(SPRITE_HEAD_DIR + k).wrapping_add(1),
                        );
                    }
                    let j = (self.ram.read_u8(SPRITE_D + k) & 1) as usize;
                    let x = self.ram.read_u8(SPRITE_X_VEL + k).wrapping_add(if j != 0 {
                        255
                    } else {
                        1
                    });
                    self.ram.write_u8(SPRITE_X_VEL + k, x);
                    if x == XV[j] {
                        self.ram
                            .write_u8(SPRITE_D + k, self.ram.read_u8(SPRITE_D + k).wrapping_add(1));
                    }
                    if x == 0 {
                        self.runtime.sfx3_with_pan(self.ram, k, 0x1e);
                    }
                }
                let x = self.ram.read_u16(LINK_X) & 0xff00 | 0x78;
                let y = self.ram.read_u16(LINK_Y) & 0xff00 | 0x50;
                let (ptx, pty) = self
                    .runtime
                    .project_speed_towards_location(self.ram, k, x, y, 5);
                let xv = self.ram.read_u8(SPRITE_X_VEL + k);
                let yv = self.ram.read_u8(SPRITE_Y_VEL + k);
                self.ram.write_u8(SPRITE_X_VEL + k, xv.wrapping_add(ptx));
                self.ram.write_u8(SPRITE_Y_VEL + k, yv.wrapping_add(pty));
                self.runtime.move_xy(self.ram, k);
                self.ram.write_u8(SPRITE_X_VEL + k, xv);
                self.ram.write_u8(SPRITE_Y_VEL + k, yv);
            } else {
                self.runtime.move_xy(self.ram, k);
                if self.ram.read_u8(SPRITE_X_VEL + k) != 64 {
                    self.ram.write_u8(
                        SPRITE_X_VEL + k,
                        self.ram.read_u8(SPRITE_X_VEL + k).wrapping_add(1),
                    );
                    self.ram.write_u8(
                        SPRITE_Y_VEL + k,
                        self.ram.read_u8(SPRITE_Y_VEL + k).wrapping_sub(1),
                    );
                }
            }
        }
    }

    pub fn ganon_bat_draw(&mut self, k: usize) {
        const D: [DrawPiece; 6] = [
            (-8, 0, 0x0560, 2),
            (8, 0, 0x4560, 2),
            (-8, 0, 0x0562, 2),
            (8, 0, 0x4562, 2),
            (-8, 0, 0x0544, 2),
            (8, 0, 0x4544, 2),
        ];
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
    }
    pub fn phantom_ganon_draw(&mut self, k: usize) {
        const D: [DrawPiece; 16] = [
            (-16, -8, 0x0d46, 2),
            (-8, -8, 0x0d47, 2),
            (8, -8, 0x4d47, 2),
            (16, -8, 0x4d46, 2),
            (-16, 8, 0x0d69, 2),
            (-8, 8, 0x0d6a, 2),
            (8, 8, 0x4d6a, 2),
            (16, 8, 0x4d69, 2),
            (-16, -8, 0x0d46, 2),
            (-8, -8, 0x0d47, 2),
            (8, -8, 0x4d47, 2),
            (16, -8, 0x4d46, 2),
            (-16, 8, 0x0d66, 2),
            (-8, 8, 0x0d67, 2),
            (8, 8, 0x4d67, 2),
            (16, 8, 0x4d66, 2),
        ];
        self.ram.write_u16(OAM_CUR_PTR, 0x950);
        self.ram.write_u16(OAM_EXT_CUR_PTR, 0xa74);
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 8;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 8]);
    }
    pub fn swish_every16_frames(&mut self, k: usize) {
        if self.ram.read_u8(FRAME_COUNTER) & 15 == 0 {
            self.runtime.sfx3_with_pan(self.ram, k, 6);
        }
    }

    pub fn sprite_ganon_trident(&mut self, k: usize) {
        self.trident_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.swish_every16_frames(k);
        self.runtime.move_xy(self.ram, k);
        const GF: [u8; 8] = [8, 7, 6, 5, 4, 3, 2, 1];
        let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_sub(1);
        self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
        self.ram.write_u8(SPRITE_G + k, GF[(s >> 2 & 7) as usize]);
        if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) & 1 != 0 {
                return;
            }
            let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 32);
            self.runtime.approach_target_speed(self.ram, k, x, y);
        } else {
            let x = self
                .sprite_x(0)
                .wrapping_add_signed(if self.ram.read_u8(SPRITE_D) != 0 {
                    -16
                } else {
                    24
                });
            let y = self.sprite_y(0).wrapping_sub(16);
            if self.ganon_attempt_trident_catch(x, y) {
                self.ram.write_u8(SPRITE_STATE + k, 0);
                self.ram.write_u8(SPRITE_AI_STATE, 3);
                self.ram.write_u8(SPRITE_DELAY_MAIN, 16);
            }
            let (xv, yv) = self
                .runtime
                .project_speed_towards_location(self.ram, k, x, y, 32);
            self.runtime.approach_target_speed(self.ram, k, xv, yv);
        }
    }
    pub fn sprite_fire_bat_trailer(&mut self, k: usize) {
        self.fire_bat_draw(k);
        if !self.runtime.return_if_inactive(self.ram, k) {
            self.fire_bat_move(k);
        }
    }
    pub fn sprite_spiral_fire_bat(&mut self, k: usize) {
        self.fire_bat_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let x = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_A + k),
            self.ram.read_u8(SPRITE_B + k),
        ]);
        let y = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_C + k),
            self.ram.read_u8(SPRITE_E + k),
        ]);
        let (a, b) = self
            .runtime
            .project_speed_towards_location(self.ram, k, x, y, 2);
        let (c, d) = self
            .runtime
            .project_speed_towards_location(self.ram, k, x, y, 80);
        self.ram.write_u8(SPRITE_X_VEL + k, d.wrapping_sub(a));
        self.ram
            .write_u8(SPRITE_Y_VEL + k, (0u8).wrapping_sub(c).wrapping_sub(b));
        self.fire_bat_move(k);
    }
    pub fn fire_bat_move(&mut self, k: usize) {
        self.fire_bat_animate(k);
        self.runtime.move_xy(self.ram, k);
        if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 7 == 0 {
            let j = self.runtime.garnish_flame_trail(self.ram, k, true);
            self.ram.write_u8(
                GARNISH_COUNTDOWN + j,
                if self.ram.read_u8(SPRITE_ANIM_CLOCK + k) == 5 {
                    0x2f
                } else {
                    0x4f
                },
            );
        }
    }
    pub fn sprite_fire_bat_launched(&mut self, k: usize) {
        self.fire_bat_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let _ = self.runtime.check_damage_to_link(self.ram, k);
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                self.get_position_relative_to_the_great_overlord_ganon(k);
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                } else {
                    self.ram.write_u8(SPRITE_GRAPHICS + k, d >> 2 & 1);
                }
            }
            1 => {
                self.get_position_relative_to_the_great_overlord_ganon(k);
                let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
                self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
                self.ram.write_u8(SPRITE_GRAPHICS + k, s >> 2 & 1);
            }
            2 => {
                self.runtime.move_xy(self.ram, k);
                self.ram.write_u8(SPRITE_DEFL_BITS + k, 64);
                let a = self.ram.read_u8(SPRITE_DELAY_AUX1 + k);
                if a == 0 {
                    let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                    if d == 0 {
                        self.fire_bat_animate(k);
                        self.fire_bat_animate(k);
                    } else {
                        let mut t = d.wrapping_sub(1);
                        if t == 0 {
                            t = 35;
                            self.ram.write_u8(SPRITE_DELAY_AUX1 + k, t);
                        }
                        self.ram.write_u8(SPRITE_GRAPHICS + k, t >> 2 & 1);
                    }
                } else if a == 1 {
                    self.runtime.apply_speed_towards_link(self.ram, k, 48);
                    self.runtime.sfx3_with_pan(self.ram, k, 0x1e);
                    self.fire_bat_animate(k);
                    self.fire_bat_animate(k);
                } else {
                    const G: [u8; 9] = [4, 4, 4, 3, 3, 3, 2, 2, 2];
                    self.ram.write_u8(SPRITE_GRAPHICS + k, G[(a >> 2) as usize]);
                }
            }
            _ => {}
        }
    }
    pub fn get_position_relative_to_the_great_overlord_ganon(&mut self, k: usize) {
        const X: [i8; 2] = [20, -18];
        const Y: [i8; 2] = [-20, -20];
        let j = self.ram.read_u8(SPRITE_D) as usize;
        let x = u16::from_le_bytes([
            self.ram.read_u8(OVERLORD_X_HI + k),
            self.ram.read_u8(OVERLORD_Y_HI + k),
        ])
        .wrapping_add_signed(X[j] as i16);
        let y = u16::from_le_bytes([
            self.ram.read_u8(OVERLORD_GEN2 + k),
            self.ram.read_u8(OVERLORD_FLOOR + k),
        ])
        .wrapping_add_signed(Y[j] as i16);
        self.set_sprite_x(k, x);
        self.set_sprite_y(k, y);
    }
    pub fn fire_bat_animate(&mut self, k: usize) {
        const G: [u8; 4] = [4, 5, 6, 5];
        let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
        self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
        self.ram
            .write_u8(SPRITE_GRAPHICS + k, G[(s >> 2 & 3) as usize]);
    }
    pub fn fire_bat_draw(&mut self, k: usize) {
        const X: [i8; 2] = [-8, 8];
        const CH: [u8; 7] = [0x88, 0x88, 0x8a, 0x8c, 0x68, 0xaa, 0xa8];
        const FL: [u8; 14] = [
            0, 0xc0, 0x80, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40,
        ];
        if let Some((x, y, flags)) = self.runtime.prep_oam_or_double_ret_wide(self.ram, k) {
            let g = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize;
            let first = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
            for (entry, i) in (0..2).rev().enumerate() {
                crate::oam::set_helper0(
                    self.ram,
                    first + entry,
                    x.wrapping_add_signed(X[i] as i16),
                    y,
                    CH[g],
                    FL[g * 2 + i] | flags,
                    2,
                );
            }
        }
    }
    pub fn ganon_attempt_trident_catch(&self, x: u16, y: u16) -> bool {
        self.ram
            .read_u16(CUR_SPRITE_X)
            .wrapping_sub(x)
            .wrapping_add(4)
            < 8
            && self
                .ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_sub(y)
                .wrapping_add(4)
                < 8
    }
    pub fn ganon_handle_fire_bat_circle(&mut self, _k: usize) {
        const X: [i8; 16] = [
            0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
        ];
        const Y: [i8; 16] = [
            32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16, 0, 16, 24, 28,
        ];
        let phase = self.ram.read_u16(OVERLORD_X_LO).wrapping_sub(4);
        self.ram.write_u16(OVERLORD_X_LO, phase);
        for i in 0..8 {
            let t = phase.wrapping_add((i * 64) as u16) & 0x1ff;
            if self.ram.read_u8(SPRITE_AI_STATE + i + 1) != 2 {
                let j = (((t >> 5).wrapping_sub(4)) & 15) as usize;
                self.ram.write_u8(SPRITE_X_VEL + i + 1, (X[j] >> 2) as u8);
                self.ram.write_u8(SPRITE_Y_VEL + i + 1, (Y[j] >> 2) as u8);
            }
            let x = self
                .sprite_x(0)
                .wrapping_add_signed(ganon_sin(t, self.ram.read_u8(OVERLORD_X_LO + 2)) as i16);
            self.ram.write_u8(OVERLORD_X_HI + i + 1, x as u8);
            self.ram.write_u8(OVERLORD_Y_HI + i + 1, (x >> 8) as u8);
            let y = self.sprite_y(0).wrapping_add_signed(ganon_sin(
                t.wrapping_add(0x80),
                self.ram.read_u8(OVERLORD_X_LO + 2),
            ) as i16);
            self.ram.write_u8(OVERLORD_GEN2 + i + 1, y as u8);
            self.ram.write_u8(OVERLORD_FLOOR + i + 1, (y >> 8) as u8);
        }
        self.ram.write_u8(TMP_COUNTER, 8);
    }
    pub fn ganon_spawn_spiral_bat(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic_ex(self.ram, k, 0xc9, 8) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(SPRITE_ANIM_CLOCK + j, 4);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, 3);
            self.ram.write_u8(SPRITE_FLAGS3 + j, 0x40);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 1);
            self.ram.write_u8(SPRITE_DEFL_BITS + j, 0x80);
            self.ram.write_u8(SPRITE_Y_HI + j, 128);
            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 48);
            self.ram.write_u8(SPRITE_BUMP_DAMAGE + j, 7);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 7);
        }
    }

    pub fn sprite_d6_ganon(&mut self, k: usize) {
        let state = self.ram.read_u8(SPRITE_AI_STATE + k);
        if (state as i8) < 0 {
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) & 1 == 0 {
                self.ganon_draw(k);
            }
            return;
        }
        if self.ram.read_u8(SPRITE_DELAY_AUX4 + k) != 0 {
            self.ram.write_u8(
                SPRITE_GRAPHICS + k,
                [16, 10][self.ram.read_u8(SPRITE_D + k) as usize],
            );
        }
        let room = self.ram.read_u8(CURRENT_DUNGEON_ROOM);
        if room == 2 && room != self.ram.read_u8(SPRITE_ROOM + k) {
            self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 64);
        }
        self.ram.write_u8(SPRITE_ROOM + k, room);
        self.ganon_draw(k);
        if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) != 0 {
            self.ram.write_u8(SPRITE_GRAPHICS + k, 15);
            self.ganon_enable_invincibility(k);
            self.runtime.check_damage_to_and_from_link(self.ram, k);
            return;
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.read_u8(SPRITE_DELAY_AUX2 + k) {
            1 => self.runtime.extinguish_torch(self.ram, false),
            16 => self.runtime.extinguish_torch(self.ram, true),
            _ => {}
        }
        let (side, delta) = self.runtime.is_right_of_link(self.ram, k);
        const HEAD: [u8; 2] = [2, 0];
        self.ram.write_u8(
            SPRITE_HEAD_DIR + k,
            if (delta as u8).wrapping_add(32) < 64 {
                1
            } else {
                HEAD[side as usize]
            },
        );
        if self.ram.read_u8(SPRITE_DELAY_AUX4 + k) != 0 {
            self.ram.write_u8(
                SPRITE_IGNORE_PROJECTILE + k,
                self.ram.read_u8(SPRITE_DELAY_AUX4 + k),
            );
            if self.runtime.return_if_recoiling(self.ram, k) {
                return;
            }
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 0);
            return;
        }
        if self.ram.read_u8(SPRITE_IGNORE_PROJECTILE + k)
            | self.ram.read_u8(FLAG_IS_LINK_IMMOBILIZED)
            == 0
            && room == 2
        {
            self.runtime.check_damage_to_and_from_link(self.ram, k);
        }
        self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + k, 0);
        match state {
            0 => {
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
                } else if d == 32 {
                    self.ram.write_u8(MUSIC_CONTROL, 0x1f);
                } else if d == 64 {
                    self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x16f);
                    self.runtime.show_message_minimal(self.ram);
                }
            }
            1 => {
                self.ganon_health_floor(k, 208);
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if d < 64 {
                    if d == 0 {
                        self.ganon_select_warp_location(k, 5);
                    } else {
                        self.ram.write_u8(
                            SPRITE_GRAPHICS + k,
                            [2, 10][self.ram.read_u8(SPRITE_D + k) as usize],
                        );
                    }
                } else if d != 64 {
                    self.ganon_phase1_animate_trident_spin(k);
                } else {
                    const X: [i8; 2] = [24, -16];
                    const Y: [i8; 2] = [4, 4];
                    const XV: [i8; 16] = [
                        32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16, 0, 16, 24, 28,
                    ];
                    const YV: [i8; 16] = [
                        0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
                    ];
                    self.ram.write_u8(SPRITE_G + k, 0);
                    if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc9) {
                        self.runtime.set_spawned_coordinates(self.ram, j);
                        let side = self.ram.read_u8(SPRITE_D + k) as usize;
                        self.set_sprite_x(j, self.sprite_x(j).wrapping_add_signed(X[side] as i16));
                        self.set_sprite_y(j, self.sprite_y(j).wrapping_add_signed(Y[side] as i16));
                        self.runtime.apply_speed_towards_link(self.ram, k, 31);
                        let a = self
                            .runtime
                            .convert_velocity_to_angle(
                                self.ram.read_u8(SPRITE_X_VEL + k),
                                self.ram.read_u8(SPRITE_Y_VEL + k),
                            )
                            .wrapping_sub(2)
                            & 15;
                        self.ram.write_u8(SPRITE_X_VEL + j, XV[a as usize] as u8);
                        self.ram.write_u8(SPRITE_Y_VEL + j, YV[a as usize] as u8);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + j, 112);
                        self.ram.write_u8(SPRITE_ANIM_CLOCK + j, 2);
                        self.ram.write_u8(SPRITE_OAM_FLAGS + j, 1);
                        self.ram.write_u8(SPRITE_FLAGS2 + j, 4);
                        self.ram.write_u8(SPRITE_DEFL_BITS + j, 0x84);
                        self.ram.write_u8(SPRITE_D + j, 2);
                        self.ram.write_u8(SPRITE_BUMP_DAMAGE + j, 7);
                        self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 7);
                    }
                }
            }
            2 => {
                self.ganon_health_floor(k, 208);
                self.ram.write_u8(
                    SPRITE_GRAPHICS + k,
                    [0, 8][self.ram.read_u8(SPRITE_D + k) as usize],
                );
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
                    self.ram.write_u8(
                        SPRITE_IGNORE_PROJECTILE + k,
                        self.ram
                            .read_u8(SPRITE_IGNORE_PROJECTILE + k)
                            .wrapping_add(1),
                    );
                    if self.ram.read_u8(SPRITE_DELAY_MAIN + k) & 1 != 0 {
                        self.ram.write_u8(SPRITE_GRAPHICS + k, 255);
                    }
                }
            }
            3 => {
                self.ganon_health_floor(k, 208);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
                    self.ganon_phase1_animate_trident_spin(k);
                } else {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 6);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 127);
                    self.ganon_handle_animation_idle(k);
                }
            }
            4 => {
                self.ganon_health_floor(k, 208);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
                    self.ganon_shake_head(k);
                } else {
                    self.ganon_select_warp_location(k, 5);
                }
            }
            5 | 10 | 13 | 18 => {
                if state == 13 {
                    self.ram.write_u8(SPRITE_HEALTH + k, 100);
                }
                self.ram.write_u8(
                    SPRITE_IGNORE_PROJECTILE + k,
                    self.ram
                        .read_u8(SPRITE_IGNORE_PROJECTILE + k)
                        .wrapping_add(1),
                );
                let x = u16::from_le_bytes([
                    self.ram.read_u8(SWAMOLA_TARGET_X_LO),
                    self.ram.read_u8(SPRITE_X_HI + k),
                ]);
                let y = u16::from_le_bytes([
                    self.ram.read_u8(SWAMOLA_TARGET_Y_LO),
                    self.ram.read_u8(SPRITE_Y_HI + k),
                ]);
                if self.ganon_attempt_trident_catch(x, y) {
                    self.ram
                        .write_u8(SPRITE_D + k, self.ram.read_u8(SPRITE_SUBTYPE + k) >> 2);
                    let hp = self.ram.read_u8(SPRITE_HEALTH + k);
                    let (ns, delay) = if state == 5 {
                        (2, 32)
                    } else if hp >= 161 {
                        (11, 40)
                    } else if hp >= 97 {
                        (14, 40)
                    } else {
                        (17, 104)
                    };
                    self.ram.write_u8(SPRITE_AI_STATE + k, ns);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, delay);
                } else {
                    let (xv, yv) = self
                        .runtime
                        .project_speed_towards_location(self.ram, k, x, y, 32);
                    self.runtime.approach_target_speed(self.ram, k, xv, yv);
                    self.runtime.move_xy(self.ram, k);
                    if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0
                        || self.ram.read_u8(FRAME_COUNTER) & 1 != 0
                    {
                        self.ram.write_u8(SPRITE_GRAPHICS + k, 255);
                        return;
                    }
                    self.ram.write_u8(
                        SPRITE_GRAPHICS + k,
                        [2, 10][self.ram.read_u8(SPRITE_D + k) as usize],
                    );
                    if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
                        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xd6) {
                            self.runtime.set_spawned_coordinates(self.ram, j);
                            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 24);
                            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 24);
                            self.ram.write_u8(SPRITE_AI_STATE + j, 255);
                            self.ram.write_u8(
                                SPRITE_GRAPHICS + j,
                                self.ram.read_u8(SPRITE_GRAPHICS + k),
                            );
                            self.ram.write_u8(
                                SPRITE_HEAD_DIR + j,
                                self.ram.read_u8(SPRITE_HEAD_DIR + k),
                            );
                        }
                    }
                }
            }
            6 => {
                self.ganon_health_floor(k, 208);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    if self.ram.read_u8(SPRITE_HEALTH + k) >= 209 {
                        self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
                    } else {
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 7);
                    }
                } else {
                    self.ganon_shake_head(k);
                }
            }
            7 => {
                self.ganon_health_floor(k, 160);
                self.ram.write_u8(OVERLORD_X_LO + 2, 40);
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 8);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                } else {
                    if d < 0xc0 && d & 15 == 0 {
                        self.ganon_spawn_spiral_bat(k);
                    }
                    self.ganon_phase1_animate_trident_spin(k);
                    self.ganon_handle_fire_bat_circle(k);
                }
            }
            8 => {
                const TAB: [i8; 16] = [0, 0, 0, 0, -1, -1, -2, -1, 0, 0, 0, 0, 1, 2, 1, 1];
                const DEL: [u8; 8] = [0x10, 0x30, 0x50, 0x70, 0x90, 0xb0, 0xd0, 0xbd];
                self.ganon_health_floor(k, 160);
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 9);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 127);
                    self.ganon_handle_animation_idle(k);
                    for j in (1..=8).rev() {
                        self.ram.write_u8(SPRITE_AI_STATE + j, 2);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + j, DEL[j - 1]);
                    }
                } else {
                    self.ram.write_u8(
                        OVERLORD_X_LO + 2,
                        self.ram
                            .read_u8(OVERLORD_X_LO + 2)
                            .wrapping_add_signed(TAB[(d >> 4 & 15) as usize]),
                    );
                    self.ganon_phase1_animate_trident_spin(k);
                    self.ganon_handle_fire_bat_circle(k);
                }
            }
            9 => {
                self.ganon_health_floor(k, 160);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ganon_select_warp_location(k, 10);
                } else {
                    self.ganon_shake_head(k);
                }
            }
            11 => {
                self.ram.write_u8(
                    SPRITE_IGNORE_PROJECTILE + k,
                    self.ram
                        .read_u8(SPRITE_IGNORE_PROJECTILE + k)
                        .wrapping_add(1),
                );
                self.ganon_handle_animation_idle(k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 7);
                } else if self.ram.read_u8(SPRITE_DELAY_MAIN + k) & 1 != 0 {
                    self.ram.write_u8(SPRITE_GRAPHICS + k, 255);
                }
            }
            _ => self.ganon_late_state(k, state),
        }
    }

    fn ganon_health_floor(&mut self, k: usize, floor: u8) {
        if self.ram.read_u8(SPRITE_HEALTH + k) < floor.wrapping_add(1) {
            self.ram.write_u8(SPRITE_HEALTH + k, floor);
        }
    }
    fn ganon_late_state(&mut self, k: usize, state: u8) {
        let delay = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
        match state {
            12 => {
                if delay == 0 {
                    self.ganon_select_warp_location(k, 13);
                    return;
                }
                let mut frame = 0usize;
                if delay < 96 {
                    frame = 1;
                    if delay < 72 {
                        if delay == 66 {
                            self.ganon_func1(k, 3);
                        }
                        frame = 2;
                    }
                }
                if self.ram.read_u8(SPRITE_D + k) != 0 {
                    frame += 3;
                }
                self.ram
                    .write_u8(SPRITE_GRAPHICS + k, [5, 6, 7, 13, 14, 10][frame]);
                if self.ram.read_u8(SPRITE_HIT_TIMER + k) & 127 == 1 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 15);
                    self.ram.write_u8(SPRITE_Z_VEL + k, 24);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 0);
                }
            }
            14 => {
                self.ram.write_u8(
                    SPRITE_IGNORE_PROJECTILE + k,
                    self.ram
                        .read_u8(SPRITE_IGNORE_PROJECTILE + k)
                        .wrapping_add(1),
                );
                self.ganon_handle_animation_idle(k);
                self.ram.write_u8(SPRITE_G + k, 0);
                if delay == 0 {
                    if self.runtime.random_number(self.ram) & 1 != 0 {
                        self.ganon_select_warp_location(k, 13);
                    } else {
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 127);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 12);
                    }
                } else if delay & 1 != 0 {
                    self.ram.write_u8(SPRITE_GRAPHICS + k, 255);
                }
            }
            15 => {
                if delay != 0 {
                    if delay == 1 {
                        self.ram.write_u8(SPRITE_AI_STATE + k, 16);
                        self.ram.write_u8(SPRITE_Z_VEL + k, 160);
                        return;
                    }
                } else {
                    self.runtime.move_z(self.ram, k);
                    let z = self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(1);
                    self.ram.write_u8(SPRITE_Z_VEL + k, z);
                    if z == 0 {
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                    }
                }
                self.ram.write_u8(
                    SPRITE_GRAPHICS + k,
                    [6, 14][self.ram.read_u8(SPRITE_D + k) as usize],
                );
            }
            16 => {
                self.ram.write_u16(BG1_Y_OFFSET, 0);
                if delay != 0 {
                    if delay == 1 {
                        self.ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
                        self.ganon_select_warp_location(k, 13);
                        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                        self.ganon_spawn_falling_tiles_overlord(k);
                        if self.ram.read_u8(SPRITE_ANIM_CLOCK + k) >= 4 {
                            self.ganon_select_warp_location(k, 10);
                            self.ram.write_u8(SPRITE_HEALTH + k, 96);
                            self.ram.write_u8(SPRITE_DELAY_AUX2 + k, 224);
                            self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x170);
                            self.runtime.show_message_minimal(self.ram);
                        }
                    } else {
                        self.ram.write_u16(
                            BG1_Y_OFFSET,
                            if delay.wrapping_sub(1) & 1 != 0 {
                                u16::MAX
                            } else {
                                1
                            },
                        );
                        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                    }
                } else {
                    self.runtime.move_z(self.ram, k);
                    if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
                        self.ram.write_u8(SPRITE_Z_VEL + k, 0);
                        self.ram.write_u8(SPRITE_Z + k, 0);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 96);
                        self.ram.write_u8(SOUND_EFFECT_AMBIENT, 7);
                        self.runtime.sfx2_with_pan(self.ram, k, 0xc);
                    }
                    self.ram.write_u8(
                        SPRITE_GRAPHICS + k,
                        [2, 10][self.ram.read_u8(SPRITE_D + k) as usize],
                    );
                }
            }
            17 => {
                self.ram.write_u8(
                    SPRITE_GRAPHICS + k,
                    [6, 14][self.ram.read_u8(SPRITE_D + k) as usize],
                );
                if delay == 0 {
                    self.ganon_select_warp_location(k, 18);
                    return;
                } else if delay == 52 {
                    self.ganon_func1(k, 5);
                } else if delay < 52 {
                    self.ram.write_u8(
                        SPRITE_GRAPHICS + k,
                        [7, 10][self.ram.read_u8(SPRITE_D + k) as usize],
                    );
                }
                if delay >= 72 || delay < 40 {
                    self.ram.write_u8(
                        SPRITE_IGNORE_PROJECTILE + k,
                        self.ram
                            .read_u8(SPRITE_IGNORE_PROJECTILE + k)
                            .wrapping_add(1),
                    );
                    if delay & 1 != 0 {
                        self.ram.write_u8(SPRITE_GRAPHICS + k, 255);
                    }
                }
                self.ganon_enable_invincibility(k);
            }
            19 => {
                self.ram.write_u8(SPRITE_OAM_FLAGS + k, 5);
                self.ram.write_u8(SPRITE_FLAGS + k, 2);
                if delay == 0 {
                    self.ram.write_u8(SPRITE_OAM_FLAGS + k, 1);
                    self.ganon_select_warp_location(k, 18);
                    self.ram.write_u8(SPRITE_TYPE + k, 0xd6);
                    self.ram.write_u8(SPRITE_HIT_TIMER + k, 0);
                } else {
                    self.ram.write_u8(
                        SPRITE_GRAPHICS + k,
                        [5, 13][self.ram.read_u8(SPRITE_D + k) as usize],
                    );
                }
            }
            _ => {}
        }
    }
    pub fn ganon_draw(&mut self, k: usize) {
        // The runtime's Ganon callback routes back to this bank; implement the
        // C draw directly rather than recursing through that callback.
        if (self.ram.read_u8(SPRITE_GRAPHICS + k) as i8) < 0
            || (self.ram.read_u8(SPRITE_AI_STATE + k) != 19
                && self.ram.read_u8(SPRITE_DELAY_AUX4 + k) == 0
                && self.ram.read_u8(CURRENT_DUNGEON_ROOM) == 0)
        {
            self.runtime.prep_oam_or_double_ret(self.ram, k);
            return;
        }
        self.trident_draw(k);
        let Some((x, y, flags)) = self.runtime.prep_oam_or_double_ret_wide(self.ram, k) else {
            return;
        };
        let first = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4 + 5;
        let g = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize;
        for i in 0..12 {
            let j = g * 12 + i;
            crate::oam::set_plain(
                self.ram,
                first + i,
                x.wrapping_add_signed(GANON_DRAW_X[j] as i16) as u8,
                y.wrapping_add_signed(GANON_DRAW_Y[j] as i16) as u8,
                GANON_DRAW_CHARACTER[j],
                flags | (GANON_DRAW_FLAGS[j] & if flags & 15 >= 5 { 0xf0 } else { 0xff }),
                2,
            );
        }
        const HEAD_OFFSET: [usize; 17] = [1, 1, 1, 1, 1, 1, 15, 1, 4, 4, 4, 4, 4, 4, 4, 15, 15];
        if HEAD_OFFSET[g] != 15 {
            let entry = first + HEAD_OFFSET[g];
            let j = usize::from(self.ram.read_u8(SPRITE_HEAD_DIR + k)) * 2
                + if self.ram.read_u8(SPRITE_D + k) != 0 {
                    6
                } else {
                    0
                };
            for i in 0..2 {
                let mut oam = crate::oam::read(self.ram, entry + i);
                oam.charnum = GANON_DRAW_CHARACTER_2[j + i];
                oam.flags = (oam.flags & 0x3f) | GANON_DRAW_FLAGS_2[j + i];
                crate::oam::write(self.ram, entry + i, oam);
            }
        }
        if self.ram.read_u8(SUBMODULE_INDEX) != 0 {
            self.runtime.correct_oam_entries(self.ram, k, 9, 0xff);
        }
        if self.ram.read_u8(SPRITE_G + k) == 9 {
            self.ram.write_u16(OAM_CUR_PTR, 0x828);
            self.ram.write_u16(OAM_EXT_CUR_PTR, 0xa2a);
            self.runtime
                .draw_multiple(self.ram, k, &[(16, -3, 0x4c0a, 2), (16, 5, 0x4c1a, 2)]);
        }
        let z = u16::from(self.ram.read_u8(SPRITE_Z + k)).wrapping_sub(1);
        let frame = usize::from((z >> 11).min(4));
        self.ram.write_u16(
            CUR_SPRITE_Y,
            self.ram.read_u16(CUR_SPRITE_Y).wrapping_add(z),
        );
        self.ram.write_u16(OAM_CUR_PTR, 0x9f4);
        self.ram.write_u16(OAM_EXT_CUR_PTR, 0xa9d);
        let saved_flags = self.ram.read_u8(SPRITE_OAM_FLAGS + k);
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 0);
        self.ram.write_u8(SPRITE_OBJ_PRIO + k, 48);
        self.runtime.draw_multiple(
            self.ram,
            k,
            &LARGE_SHADOW_DRAW_DATA[frame * 3..frame * 3 + 3],
        );
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, saved_flags);
        self.runtime.get_16bit_coords(self.ram, k);
    }

    pub fn ganon_enable_invincibility(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_HIT_TIMER + k) & 127 == 26 {
            self.ram.write_u8(SPRITE_HIT_TIMER + k, 0);
            self.ram.write_u8(SPRITE_AI_STATE + k, 19);
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 127);
            self.ram.write_u8(SPRITE_TYPE + k, 215);
        }
    }

    pub fn ganon_spawn_falling_tiles_overlord(&mut self, k: usize) {
        const TY: [u8; 4] = [12, 13, 14, 15];
        const X: [u8; 4] = [0x18, 0xd8, 0xd8, 0x18];
        const Y: [u8; 4] = [0x28, 0x28, 0xd8, 0xd8];
        let Some(j) = (0..8)
            .rev()
            .find(|&j| self.ram.read_u8(OVERLORD_TYPE + j) == 0)
        else {
            return;
        };
        let t = self.ram.read_u8(SPRITE_ANIM_CLOCK + k) as usize;
        if t >= 4 {
            return;
        }
        self.ram.write_u8(SPRITE_ANIM_CLOCK + k, (t + 1) as u8);
        self.ram.write_u8(OVERLORD_TYPE + j, TY[t]);
        self.ram.write_u8(OVERLORD_X_LO + j, X[t]);
        self.ram
            .write_u8(OVERLORD_X_HI + j, (self.ram.read_u16(LINK_X) >> 8) as u8);
        self.ram.write_u8(OVERLORD_Y_LO + j, Y[t]);
        self.ram
            .write_u8(OVERLORD_Y_HI + j, (self.ram.read_u16(LINK_Y) >> 8) as u8);
        self.ram.write_u8(OVERLORD_GEN1 + j, 0);
        self.ram.write_u8(OVERLORD_GEN2 + j, 0);
    }

    pub fn ganon_func1(&mut self, k: usize, t: u8) {
        self.ram.write_u8(TMP_COUNTER, t);
        let Some(j) = self.runtime.spawn_dynamic_ex(self.ram, k, 0xc9, 8) else {
            return;
        };
        self.runtime.sfx2_with_pan(self.ram, k, 0x2a);
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, t);
        self.ram.write_u8(SPRITE_ANIM_CLOCK + j, t);
        self.ram.write_u8(SPRITE_OAM_FLAGS + j, 3);
        self.ram.write_u8(SPRITE_FLAGS3 + j, 0x40);
        self.ram.write_u8(SPRITE_FLAGS2 + j, 0x21);
        self.ram.write_u8(SPRITE_DEFL_BITS + j, 0x40);
        self.set_sprite_y(
            j,
            self.sprite_y(j)
                .wrapping_add_signed(if self.ram.read_u8(SPRITE_D + k) != 0 {
                    -16
                } else {
                    0
                }),
        );
        self.runtime.apply_speed_towards_link(self.ram, j, 32);
        self.ram.write_u8(SPRITE_DELAY_MAIN + j, 16);
        self.ram
            .write_u8(SPRITE_A + j, self.ram.read_u8(SPRITE_X_LO));
        self.ram
            .write_u8(SPRITE_B + j, self.ram.read_u8(SPRITE_X_HI));
        self.ram
            .write_u8(SPRITE_C + j, self.ram.read_u8(SPRITE_Y_LO));
        self.ram
            .write_u8(SPRITE_E + j, self.ram.read_u8(SPRITE_Y_HI));
        self.ram.write_u8(SPRITE_BUMP_DAMAGE + j, 7);
        self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 7);
    }

    pub fn ganon_phase1_animate_trident_spin(&mut self, k: usize) {
        const GF: [u8; 16] = [8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1];
        const GX: [u8; 16] = [0, 0, 1, 1, 0, 0, 1, 1, 8, 8, 9, 9, 8, 8, 9, 9];
        let j = ((self.ram.read_u8(SPRITE_DELAY_MAIN + k) >> 2 & 7)
            + if self.ram.read_u8(SPRITE_D + k) != 0 {
                8
            } else {
                0
            }) as usize;
        self.ram.write_u8(SPRITE_G + k, GF[j]);
        self.ram.write_u8(SPRITE_GRAPHICS + k, GX[j]);
        self.swish_every16_frames(k);
    }

    pub fn ganon_handle_animation_idle(&mut self, k: usize) {
        let d = self.ram.read_u8(SPRITE_D + k) as usize;
        self.ram.write_u8(SPRITE_G + k, [9, 10][d]);
        self.ram.write_u8(SPRITE_GRAPHICS + k, [2, 10][d]);
    }

    pub fn ganon_select_warp_location(&mut self, k: usize, state: u8) {
        const NEXT: [u8; 32] = [
            4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0,
            1, 2, 3,
        ];
        const Y: [u8; 8] = [0x40, 0x30, 0x30, 0x40, 0xb0, 0xc0, 0xc0, 0xb0];
        const X: [u8; 8] = [0x30, 0x50, 0xa0, 0xc0, 0x40, 0x60, 0x90, 0xb0];
        let j = NEXT[((self.runtime.random_number(self.ram) & 3)
            | self.ram.read_u8(SPRITE_SUBTYPE + k).wrapping_shl(2)) as usize];
        self.ram.write_u8(SPRITE_SUBTYPE + k, j);
        self.ram.write_u8(SWAMOLA_TARGET_X_LO, X[j as usize]);
        self.ram.write_u8(SWAMOLA_TARGET_Y_LO, Y[j as usize]);
        self.ram.write_u8(SPRITE_AI_STATE + k, state);
        self.ram.write_u8(SPRITE_X_VEL + k, 0);
        self.ram.write_u8(SPRITE_Y_VEL + k, 0);
        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
        self.runtime.sfx3_with_pan(self.ram, k, 0x28);
    }

    pub fn ganon_shake_head(&mut self, k: usize) {
        const H: [u8; 18] = [0, 0, 0, 1, 2, 2, 2, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 16];
        self.ram.write_u8(
            SPRITE_HEAD_DIR + k,
            H[(self.ram.read_u8(SPRITE_DELAY_MAIN + k) >> 3) as usize],
        );
    }

    pub fn sprite_prep_swamola_initialize_segments(&mut self, k: usize) {
        const BUGGY: [usize; 6] = [0x1c, 0xa9, 0x03, 0x9d, 0x90, 0x0d];
        let mut j = if self.ram.read_u32(ENHANCED_FEATURES0) & 1 != 0 {
            k * 32
        } else {
            BUGGY[k]
        };
        for _ in 0..32 {
            self.ram
                .write_u8(SWAMOLA_X_LO + j, self.ram.read_u8(SPRITE_X_LO + k));
            self.ram
                .write_u8(SWAMOLA_X_HI + j, self.ram.read_u8(SPRITE_X_HI + k));
            self.ram
                .write_u8(SWAMOLA_Y_LO + j, self.ram.read_u8(SPRITE_Y_LO + k));
            self.ram
                .write_u8(SWAMOLA_Y_HI + j, self.ram.read_u8(SPRITE_Y_HI + k));
            j += 1;
        }
    }

    pub fn sprite_cf_swamola(&mut self, k: usize) {
        const DIR: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        const X: [i8; 9] = [0, 0, 32, 32, 32, 0, -32, -32, -32];
        const Y: [i8; 9] = [0, -32, -32, 0, 32, 32, 32, 0, -32];
        let state = self.ram.read_u8(SPRITE_AI_STATE + k);
        if state != 0 {
            if (state as i8) < 0 {
                self.sprite_swamola_ripples(k);
                return;
            }
            self.swamola_draw(k);
        }
        self.runtime.get_16bit_coords(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.write_u8(
            SPRITE_SUBTYPE2 + k,
            self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1),
        );
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        let old = self.ram.read_u8(SPRITE_Y_VEL + k);
        self.ram.write_u8(
            SPRITE_Y_VEL + k,
            old.wrapping_add(self.ram.read_u8(SPRITE_Z_VEL + k)),
        );
        self.runtime.move_xy(self.ram, k);
        self.ram.write_u8(SPRITE_Y_VEL + k, old);
        match state {
            0 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    let j = DIR[(self.runtime.random_number(self.ram) & 7) as usize];
                    if j != self.ram.read_u8(SPRITE_D + k) {
                        let x = u16::from_le_bytes([
                            self.ram.read_u8(SPRITE_A + k),
                            self.ram.read_u8(SPRITE_B + k),
                        ])
                        .wrapping_add_signed(X[j as usize] as i16);
                        let y = u16::from_le_bytes([
                            self.ram.read_u8(SPRITE_C + k),
                            self.ram.read_u8(SPRITE_HEAD_DIR + k),
                        ])
                        .wrapping_add_signed(Y[j as usize] as i16);
                        let [a, b] = x.to_le_bytes();
                        let [c, d] = y.to_le_bytes();
                        self.ram.write_u8(SWAMOLA_TARGET_X_LO + k, a);
                        self.ram.write_u8(SWAMOLA_TARGET_X_HI + k, b);
                        self.ram.write_u8(SWAMOLA_TARGET_Y_LO + k, c);
                        self.ram.write_u8(SWAMOLA_TARGET_Y_HI + k, d);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                        self.ram.write_u8(SPRITE_X_VEL + k, 0);
                        self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                        self.ram.write_u8(SPRITE_Z_VEL + k, (-15i8) as u8);
                        self.swamola_spawn_ripples(k);
                    }
                }
            }
            1 => {
                if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 3 == 0 {
                    let z = self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_add(1);
                    self.ram.write_u8(SPRITE_Z_VEL + k, z);
                    if z == 0 {
                        self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    }
                    let (x, y) = self.swamola_project_velocity_towards_target(k);
                    self.runtime.approach_target_speed(self.ram, k, x, y);
                }
            }
            2 => {
                let j = (self.ram.read_u8(SPRITE_G + k) & 1) as usize;
                let z = self
                    .ram
                    .read_u8(SPRITE_Z_VEL + k)
                    .wrapping_add_signed([2, -2][j]);
                self.ram.write_u8(SPRITE_Z_VEL + k, z);
                if z == [12, (-12i8) as u8][j] {
                    self.ram
                        .write_u8(SPRITE_G + k, self.ram.read_u8(SPRITE_G + k).wrapping_add(1));
                }
                let x = u16::from_le_bytes([
                    self.ram.read_u8(SWAMOLA_TARGET_X_LO + k),
                    self.ram.read_u8(SWAMOLA_TARGET_X_HI + k),
                ]);
                let y = u16::from_le_bytes([
                    self.ram.read_u8(SWAMOLA_TARGET_Y_LO + k),
                    self.ram.read_u8(SWAMOLA_TARGET_Y_HI + k),
                ]);
                if self
                    .ram
                    .read_u16(CUR_SPRITE_X)
                    .wrapping_sub(x)
                    .wrapping_add(8)
                    < 16
                    && self
                        .ram
                        .read_u16(CUR_SPRITE_Y)
                        .wrapping_sub(y)
                        .wrapping_add(8)
                        < 16
                {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                }
                let (xv, yv) = self.swamola_project_velocity_towards_target(k);
                self.ram.write_u8(SPRITE_X_VEL + k, xv);
                self.ram.write_u8(SPRITE_Y_VEL + k, yv);
            }
            3 => {
                if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 3 == 0 {
                    let z = self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_add(1);
                    self.ram.write_u8(SPRITE_Z_VEL + k, z);
                    if z == 16 {
                        self.ram.write_u8(SPRITE_AI_STATE + k, 4);
                        self.swamola_spawn_ripples(k);
                        self.ram.write_u8(SPRITE_Y_HI + k, 128);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 80);
                    }
                    self.runtime.approach_target_speed(self.ram, k, 0, 0);
                }
            }
            4 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    let j = DIR[(self.runtime.random_number(self.ram) & 7) as usize];
                    self.ram.write_u8(SPRITE_D + k, j);
                    let x = u16::from_le_bytes([
                        self.ram.read_u8(SPRITE_A + k),
                        self.ram.read_u8(SPRITE_B + k),
                    ])
                    .wrapping_add_signed(X[j as usize] as i16);
                    let y = u16::from_le_bytes([
                        self.ram.read_u8(SPRITE_C + k),
                        self.ram.read_u8(SPRITE_HEAD_DIR + k),
                    ])
                    .wrapping_add_signed(Y[j as usize] as i16);
                    self.set_sprite_x(k, x);
                    self.set_sprite_y(k, y);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
                    self.ram.write_u8(SPRITE_X_VEL + k, 0);
                    self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                    self.ram.write_u8(SPRITE_Z_VEL + k, 0);
                }
            }
            _ => {}
        }
    }

    pub fn swamola_project_velocity_towards_target(&mut self, k: usize) -> (u8, u8) {
        let x = u16::from_le_bytes([
            self.ram.read_u8(SWAMOLA_TARGET_X_LO + k),
            self.ram.read_u8(SWAMOLA_TARGET_X_HI + k),
        ]);
        let y = u16::from_le_bytes([
            self.ram.read_u8(SWAMOLA_TARGET_Y_LO + k),
            self.ram.read_u8(SWAMOLA_TARGET_Y_HI + k),
        ]);
        self.runtime
            .project_speed_towards_location(self.ram, k, x, y, 15)
    }
    pub fn swamola_spawn_ripples(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xcf) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(SPRITE_AI_STATE + j, 128);
            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 32);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, 4);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 4);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 0);
        }
    }
    pub fn sprite_swamola_ripples(&mut self, k: usize) {
        self.swamola_ripples_draw(k);
        if !self.runtime.return_if_inactive(self.ram, k)
            && self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0
        {
            self.ram.write_u8(SPRITE_STATE + k, 0);
        }
    }
    pub fn swamola_ripples_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (0, 4, 0x00d8, 0),
            (8, 4, 0x40d8, 0),
            (0, 4, 0x00d9, 0),
            (8, 4, 0x40d9, 0),
            (0, 4, 0x00da, 0),
            (8, 4, 0x40da, 0),
            (0, 4, 0x00d9, 0),
            (8, 4, 0x40d9, 0),
        ];
        self.runtime.oam_allocate_region_b(self.ram, 8);
        let i = (self.ram.read_u8(SPRITE_DELAY_MAIN + k) >> 2 & 3) as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
    }
    pub fn swamola_draw(&mut self, k: usize) {
        const G: [u8; 16] = [7, 6, 5, 4, 3, 4, 5, 6, 7, 6, 5, 4, 3, 4, 5, 6];
        const G2: [u8; 4] = [0, 0, 1, 2];
        const FL: [u8; 16] = [
            0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40,
        ];
        const OFF: [u8; 4] = [8, 16, 22, 26];
        let a = self.runtime.convert_velocity_to_angle(
            self.ram.read_u8(SPRITE_X_VEL + k),
            self.ram
                .read_u8(SPRITE_Y_VEL + k)
                .wrapping_add(self.ram.read_u8(SPRITE_Z_VEL + k)),
        ) as usize;
        self.ram.write_u8(SPRITE_GRAPHICS + k, G[a]);
        self.ram.write_u8(
            SPRITE_OAM_FLAGS + k,
            self.ram.read_u8(SPRITE_OAM_FLAGS + k) & 63 | FL[a],
        );
        self.runtime.draw_single_large(self.ram, k);
        let mut j = (self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 31) as usize + k * 32;
        self.ram
            .write_u8(SWAMOLA_X_LO + j, self.ram.read_u8(SPRITE_X_LO + k));
        self.ram
            .write_u8(SWAMOLA_X_HI + j, self.ram.read_u8(SPRITE_X_HI + k));
        self.ram
            .write_u8(SWAMOLA_Y_LO + j, self.ram.read_u8(SPRITE_Y_LO + k));
        self.ram
            .write_u8(SWAMOLA_Y_HI + j, self.ram.read_u8(SPRITE_Y_HI + k));
        let neg = (self.ram.read_u8(SPRITE_Y_VEL + k) as i8) < 0;
        let delta: i16 = if neg { -1 } else { 1 };
        if neg {
            self.ram
                .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(20));
            self.ram.write_u16(
                OAM_EXT_CUR_PTR,
                self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(5),
            );
        }
        for i in 0..4 {
            self.ram.write_u8(SPRITE_GRAPHICS + k, G2[i]);
            j = (self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_sub(OFF[i]) & 31) as usize + k * 32;
            self.ram.write_u16(
                CUR_SPRITE_X,
                u16::from_le_bytes([
                    self.ram.read_u8(SWAMOLA_X_LO + j),
                    self.ram.read_u8(SWAMOLA_X_HI + j),
                ]),
            );
            self.ram.write_u16(
                CUR_SPRITE_Y,
                u16::from_le_bytes([
                    self.ram.read_u8(SWAMOLA_Y_LO + j),
                    self.ram.read_u8(SWAMOLA_Y_HI + j),
                ]),
            );
            self.ram.write_u16(
                OAM_CUR_PTR,
                self.ram
                    .read_u16(OAM_CUR_PTR)
                    .wrapping_add_signed(delta * 4),
            );
            self.ram.write_u16(
                OAM_EXT_CUR_PTR,
                self.ram
                    .read_u16(OAM_EXT_CUR_PTR)
                    .wrapping_add_signed(delta),
            );
            self.runtime.draw_single_large(self.ram, k);
        }
        self.ram.write_u8(BYTE_7E0FB6, 4);
    }

    pub fn sprite_prep_blind_prepare_battle(&mut self, k: usize) {
        if self.ram.read_u8(FOLLOWER_INDICATOR) != 6
            && self.ram.read_u16(DUNG_SAVEGAME_STATE_BITS) & 0x2000 != 0
        {
            self.ram.write_u8(SPRITE_DELAY_AUX2 + k, 96);
            self.ram.write_u8(SPRITE_C + k, 1);
            self.ram.write_u8(SPRITE_D + k, 2);
            self.ram.write_u8(SPRITE_HEAD_DIR + k, 4);
            self.ram.write_u8(SPRITE_GRAPHICS + k, 7);
            self.ram.write_u8(BYTE_7E0B69, 0);
        } else {
            self.ram.write_u8(SPRITE_STATE + k, 0);
        }
    }
    pub fn blind_laser_spawn_trail_garnish(&mut self, j: usize) {
        let k = self.runtime.garnish_alloc_overwrite_old(self.ram);
        self.ram.write_u8(GARNISH_TYPE + k, 15);
        self.ram.write_u8(GARNISH_ACTIVE, 15);
        self.ram
            .write_u8(GARNISH_OAM_FLAGS + k, self.ram.read_u8(SPRITE_GRAPHICS + j));
        self.ram.write_u8(GARNISH_SPRITE + k, j as u8);
        self.ram
            .write_u8(GARNISH_X_LO + k, self.ram.read_u8(SPRITE_X_LO + j));
        self.ram
            .write_u8(GARNISH_X_HI + k, self.ram.read_u8(SPRITE_X_HI + j));
        let y = self.sprite_y(j).wrapping_add(16);
        self.ram.write_u8(GARNISH_Y_LO + k, y as u8);
        self.ram.write_u8(GARNISH_Y_HI + k, (y >> 8) as u8);
        self.ram.write_u8(GARNISH_COUNTDOWN + k, 10);
    }
    pub fn sprite_blind_head(&mut self, k: usize) {
        const XL: [u8; 2] = [0x98, 0x58];
        const YL: [u8; 2] = [0xb0, 0x50];
        const YV: [u8; 2] = [24, (-24i8) as u8];
        const XV: [u8; 2] = [32, (-32i8) as u8];
        const CH: [u8; 16] = [
            0x86, 0x86, 0x84, 0x82, 0x80, 0x82, 0x84, 0x86, 0x86, 0x86, 0x88, 0x8a, 0x8c, 0x8a,
            0x88, 0x86,
        ];
        const FL: [u8; 16] = [
            0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0,
        ];
        self.ram.write_u8(
            SPRITE_OBJ_PRIO + k,
            self.ram.read_u8(SPRITE_OBJ_PRIO + k) | 48,
        );
        self.runtime.draw_single_large(self.ram, k);
        let h = self.ram.read_u8(SPRITE_HEAD_DIR + k) as usize;
        self.runtime
            .set_current_oam_char_flags(self.ram, CH[h], FL[h]);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.read_u8(SPRITE_F + k) == 14 {
            self.ram.write_u8(SPRITE_F + k, 8);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        let sub = self.ram.read_u8(SPRITE_SUBTYPE + k).wrapping_sub(1);
        self.ram.write_u8(SPRITE_SUBTYPE + k, sub);
        if (sub as i8) < 0 {
            self.ram.write_u8(SPRITE_SUBTYPE + k, 2);
            self.ram.write_u8(
                SPRITE_HEAD_DIR + k,
                self.ram.read_u8(SPRITE_HEAD_DIR + k).wrapping_add(1) & 15,
            );
        }
        if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.ram.write_u8(
            SPRITE_SUBTYPE2 + k,
            self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1),
        );
        if let Some(j) = self.blind_spit_fireball(k, 0x1f) {
            let z = self.ram.read_u8(SPRITE_Z_SUBPOS + k).wrapping_sub(1);
            self.ram.write_u8(SPRITE_Z_SUBPOS + k, z);
            if (z as i8) < 0 {
                self.ram.write_u8(SPRITE_Z_SUBPOS + k, 4);
                let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 32);
                self.ram.write_u8(SPRITE_X_VEL + j, x);
                self.ram.write_u8(SPRITE_Y_VEL + j, y);
            }
        }
        let j = (self.ram.read_u8(SPRITE_G + k) & 1) as usize;
        let x = self.ram.read_u8(SPRITE_X_VEL + k);
        if x != XV[j] {
            self.ram.write_u8(
                SPRITE_X_VEL + k,
                x.wrapping_add(if j != 0 { 255 } else { 1 }),
            );
        }
        if self.ram.read_u8(SPRITE_X_LO + k) & !1 == XL[j] {
            self.ram
                .write_u8(SPRITE_G + k, self.ram.read_u8(SPRITE_G + k).wrapping_add(1));
        }
        let j = (self.ram.read_u8(SPRITE_ANIM_CLOCK + k) & 1) as usize;
        let y = self.ram.read_u8(SPRITE_Y_VEL + k);
        if y != YV[j] {
            self.ram.write_u8(
                SPRITE_Y_VEL + k,
                y.wrapping_add(if j != 0 { 255 } else { 1 }),
            );
        }
        if self.ram.read_u8(SPRITE_Y_LO + k) & !1 == YL[j] {
            self.ram.write_u8(
                SPRITE_ANIM_CLOCK + k,
                self.ram.read_u8(SPRITE_ANIM_CLOCK + k).wrapping_add(1),
            );
        }
        if self.ram.read_u8(SPRITE_F + k) == 0 {
            self.runtime.move_xy(self.ram, k);
        }
    }
    pub fn blind_spawn_head(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xce) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(SPRITE_FLAGS3 + j, 0x5b);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, 0xb);
            self.ram.write_u8(SPRITE_DEFL_BITS + j, 4);
            self.ram.write_u8(SPRITE_A + j, 2);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 1);
            self.ram.write_u8(SPRITE_FLAGS4 + j, 0);
            self.ram.write_u8(SPRITE_FLAGS + j, 0);
            self.ram.write_u8(SPRITE_Z + j, 23);
            self.ram.write_u8(
                SPRITE_Y_LO + j,
                self.ram.read_u8(SPRITE_Y_LO + j).wrapping_add(23),
            );
            self.ram
                .write_u8(SPRITE_G + j, (self.ram.read_u8(SPRITE_X_LO + j) >> 7) & 1);
            self.ram.write_u8(
                SPRITE_ANIM_CLOCK + j,
                (self.ram.read_u8(SPRITE_Y_LO + j).wrapping_sub(23) >> 7) & 1,
            );
            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 48);
        }
    }
    pub fn sprite_ce_blind(&mut self, k: usize) {
        let a = self.ram.read_u8(SPRITE_A + k);
        if (a as i8) < 0 {
            self.sprite_blind_laser(k);
        } else if a == 2 {
            self.sprite_blind_head(k);
        } else {
            self.sprite_blind_blind_blind(k);
        }
    }
    pub fn sprite_blind_laser(&mut self, k: usize) {
        const G: [u8; 16] = [7, 7, 8, 9, 10, 9, 8, 7, 7, 7, 8, 9, 10, 9, 8, 7];
        const F: [u8; 16] = [
            0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80,
        ];
        let h = self.ram.read_u8(SPRITE_HEAD_DIR + k) as usize;
        self.ram.write_u8(SPRITE_GRAPHICS + k, G[h]);
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, F[h] | 3);
        self.runtime.prep_oam(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
        if d != 0 {
            if d == 1 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            return;
        }
        self.runtime.check_damage_to_link_same_layer(self.ram, k);
        self.set_sprite_x(
            k,
            self.sprite_x(k)
                .wrapping_add_signed(self.ram.read_u8(SPRITE_X_VEL + k) as i8 as i16),
        );
        self.set_sprite_y(
            k,
            self.sprite_y(k)
                .wrapping_add_signed(self.ram.read_u8(SPRITE_Y_VEL + k) as i8 as i16),
        );
        if self.runtime.check_tile_collision(self.ram, k) {
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 12);
        }
        self.blind_laser_spawn_trail_garnish(k);
    }
    pub fn sprite_blind_blind_blind(&mut self, k: usize) {
        self.ram.write_u8(
            SPRITE_OBJ_PRIO + k,
            self.ram.read_u8(SPRITE_OBJ_PRIO + k) | 0x30,
        );
        self.blind_draw(k);
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 1);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let a = self.ram.read_u8(SPRITE_F + k);
        if a != 0 {
            self.ram.write_u8(SPRITE_F + k, a - 1);
        }
        if a == 11 {
            self.ram.write_u8(SPRITE_HIT_TIMER + k, 0);
            self.ram.write_u8(SPRITE_WALLCOLL + k, 0);
            if self.ram.read_u8(SPRITE_DELAY_AUX4 + k) == 0 {
                self.ram.write_u8(SPRITE_HEALTH + k, 128);
                self.ram.write_u8(SPRITE_DELAY_AUX4 + k, 48);
                self.ram.write_u8(
                    SPRITE_OAM_FLAGS + k,
                    self.ram.read_u8(SPRITE_OAM_FLAGS + k) & 1,
                );
                let heads = self.ram.read_u8(SPRITE_Z_SUBPOS + k).wrapping_add(1);
                self.ram.write_u8(SPRITE_Z_SUBPOS + k, heads);
                if heads < 3 {
                    self.ram.write_u8(SPRITE_WALLCOLL + k, 96);
                    self.ram.write_u8(SPRITE_SUBTYPE + k, 1);
                } else {
                    self.ram.write_u8(SPRITE_Z_SUBPOS + k, 0);
                    let defeated = self.ram.read_u8(SPRITE_LIMIT_INSTANCE).wrapping_add(1);
                    self.ram.write_u8(SPRITE_LIMIT_INSTANCE, defeated);
                    if defeated == 3 {
                        self.runtime.kill_friends(self.ram);
                        self.ram.write_u8(SPRITE_STATE + k, 4);
                        self.ram.write_u8(SPRITE_A + k, 0);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                        self.ram.write_u8(SPRITE_HIT_TIMER + k, 255);
                        self.ram.write_u8(
                            FLAG_BLOCK_LINK_MENU,
                            self.ram.read_u8(FLAG_BLOCK_LINK_MENU).wrapping_add(1),
                        );
                        self.runtime.sfx3_with_pan(self.ram, k, 0x22);
                        return;
                    }
                    self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                    self.ram.write_u8(SPRITE_X_VEL + k, 0);
                    self.ram.write_u8(SPRITE_C + k, 6);
                    self.ram.write_u8(SPRITE_DELAY_AUX2 + k, 255);
                    self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + k, 255);
                    self.blind_spawn_head(k);
                }
            }
        }
        if self.ram.read_u8(SPRITE_A + k) != 0 {
            const G: [u8; 7] = [20, 19, 18, 17, 16, 15, 15];
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            self.ram.write_u8(
                SPRITE_GRAPHICS + k,
                G[(self.ram.read_u8(SPRITE_DELAY_MAIN + k) >> 3) as usize],
            );
            return;
        }
        self.ram.write_u8(
            SPRITE_SUBTYPE2 + k,
            self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1),
        );
        if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 1 == 0 {
            self.ram.write_u8(
                SPRITE_DELAY_MAIN + k,
                self.ram.read_u8(SPRITE_DELAY_MAIN + k).wrapping_add(1),
            );
        }
        if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) != 0 {
            self.ram.write_u8(SPRITE_AI_STATE + k, 0);
            if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) == 8 {
                self.blind_spawn_laser(k);
            }
            self.blind_check_bump_damage(k);
            return;
        }
        self.ram
            .write_u8(BYTE_7E0B69, self.ram.read_u8(BYTE_7E0B69).wrapping_add(1));
        if self.ram.read_u8(SPRITE_STUNNED + k) == 0 {
            if self.ram.read_u8(SPRITE_AI_STATE + k) != 0 {
                self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 16);
                self.ram.write_u8(SPRITE_STUNNED + k, 128);
                self.ram.write_u8(SPRITE_AI_STATE + k, 0);
            }
        } else {
            self.ram.write_u8(
                SPRITE_STUNNED + k,
                self.ram.read_u8(SPRITE_STUNNED + k).wrapping_sub(1),
            );
            self.ram.write_u8(SPRITE_AI_STATE + k, 0);
        }
        self.ram
            .write_u8(SPRITE_X_HI + k, (self.ram.read_u16(LINK_X) >> 8) as u8);
        self.ram
            .write_u8(SPRITE_Y_HI + k, (self.ram.read_u16(LINK_Y) >> 8) as u8);
        match self.ram.read_u8(SPRITE_C + k) {
            0 => {
                self.ram.write_u8(DMA_VAR6, 0);
                self.ram.write_u8(DMA_VAR7, 0xa0);
                let d = self.ram.read_u8(SPRITE_DELAY_AUX2 + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_C + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_AUX2 + k, 96);
                } else if d == 80 {
                    self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x123);
                    self.runtime.show_message_minimal(self.ram);
                } else if d == 24 {
                    let _ = self.spawn_boss_poof(k);
                }
            }
            1 => {
                self.blind_check_bump_damage(k);
                self.ram.write_u8(SPRITE_GRAPHICS + k, 9);
                let d = self.ram.read_u8(SPRITE_DELAY_AUX2 + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_C + k, 2);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                    self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + k, 0);
                } else if d < 64 {
                    self.ram.write_u8(SPRITE_Y_VEL + k, (-8i8) as u8);
                    self.runtime.move_y(self.ram, k);
                }
                self.blind_animate(k);
                self.ram.write_u8(SPRITE_HEAD_DIR + k, 4);
            }
            2 => {
                const YV: [u8; 2] = [18, (-18i8) as u8];
                const XV: [u8; 2] = [24, (-24i8) as u8];
                const XP: [u8; 2] = [164, 76];
                self.blind_check_bump_damage(k);
                self.blind_animate(k);
                let subtype2 = self.ram.read_u8(SPRITE_SUBTYPE2 + k);
                let change_wall = ((subtype2 & 127 == 0
                    && self.runtime.is_below_link(self.ram, k).wrapping_add(2)
                        != self.ram.read_u8(SPRITE_D + k))
                    || self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0)
                    && self.ram.read_u8(SPRITE_X_LO + k) < 0x78;
                if change_wall {
                    self.ram.write_u8(SPRITE_C + k, 3);
                    self.ram
                        .write_u8(SPRITE_Y_VEL + k, self.ram.read_u8(SPRITE_Y_VEL + k) & !1);
                    self.ram
                        .write_u8(SPRITE_X_VEL + k, self.ram.read_u8(SPRITE_X_VEL + k) & !1);
                    self.ram.write_u8(SPRITE_DELAY_AUX2 + k, 0x30);
                    return;
                }
                let j = (self.ram.read_u8(SPRITE_B + k) & 1) as usize;
                let y =
                    self.ram
                        .read_u8(SPRITE_Y_VEL + k)
                        .wrapping_add(if j != 0 { 255 } else { 1 });
                self.ram.write_u8(SPRITE_Y_VEL + k, y);
                if y == YV[j] {
                    self.ram
                        .write_u8(SPRITE_B + k, self.ram.read_u8(SPRITE_B + k).wrapping_add(1));
                }
                let j = (self.ram.read_u8(SPRITE_G + k) & 1) as usize;
                let x = self.ram.read_u8(SPRITE_X_VEL + k);
                if x != XV[j] {
                    self.ram.write_u8(
                        SPRITE_X_VEL + k,
                        x.wrapping_add(if j != 0 { 255 } else { 1 }),
                    );
                }
                if self.ram.read_u8(SPRITE_X_LO + k) & !1 == XP[j] {
                    self.ram
                        .write_u8(SPRITE_G + k, self.ram.read_u8(SPRITE_G + k).wrapping_add(1));
                }
                self.runtime.move_xy(self.ram, k);
                let wallcoll = self.ram.read_u8(SPRITE_WALLCOLL + k);
                if wallcoll != 0 {
                    self.blind_fireball_flurry(k, wallcoll);
                } else if subtype2 & 7 == 0 {
                    self.runtime.spawn_probe_always(
                        self.ram,
                        k,
                        self.ram.read_u8(SPRITE_HEAD_DIR + k) << 2,
                    );
                }
            }
            3 => {
                self.blind_check_bump_damage(k);
                if self.ram.read_u8(SPRITE_DELAY_AUX2 + k) != 0 {
                    self.blind_decelerate_x(k);
                    self.runtime.move_x(self.ram, k);
                    self.blind_decelerate_y(k);
                } else {
                    const YV: [u8; 2] = [64, (-64i8) as u8];
                    const YP: [u8; 2] = [0x90, 0x50];
                    let j = self.ram.read_u8(SPRITE_D + k).wrapping_sub(2) as usize;
                    let y = self.ram.read_u8(SPRITE_Y_VEL + k);
                    if y != YV[j] {
                        self.ram.write_u8(
                            SPRITE_Y_VEL + k,
                            y.wrapping_add(if j != 0 { 254 } else { 2 }),
                        );
                    }
                    if self.ram.read_u8(SPRITE_Y_LO + k) & !3 == YP[j] {
                        self.ram.write_u8(SPRITE_C + k, 4);
                        self.ram
                            .write_u8(SPRITE_B + k, self.ram.read_u8(SPRITE_D + k).wrapping_sub(1));
                    }
                    self.runtime.move_xy(self.ram, k);
                    self.blind_decelerate_x(k);
                }
            }
            4 => {
                self.blind_check_bump_damage(k);
                if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 7 == 0 {
                    let j = self.ram.read_u8(SPRITE_D + k).wrapping_sub(2) as usize;
                    let gfx = [0, 9][j];
                    if self.ram.read_u8(SPRITE_GRAPHICS + k) == gfx {
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 254);
                        self.ram.write_u8(SPRITE_C + k, 2);
                        self.ram
                            .write_u8(SPRITE_D + k, self.ram.read_u8(SPRITE_D + k) ^ 1);
                        self.ram
                            .write_u8(SPRITE_G + k, self.ram.read_u8(SPRITE_X_LO + k) >> 7);
                    } else {
                        self.ram.write_u8(
                            SPRITE_GRAPHICS + k,
                            self.ram
                                .read_u8(SPRITE_GRAPHICS + k)
                                .wrapping_add(if j != 0 { 1 } else { 255 }),
                        );
                    }
                }
                self.blind_decelerate_y(k);
            }
            5 => self.blind_fireball_flurry(k, 0x65),
            6 => {
                self.ram.write_u8(SPRITE_HIT_TIMER + k, 0);
                self.ram.write_u8(SPRITE_HEAD_DIR + k, 12);
                let d = self.ram.read_u8(SPRITE_DELAY_AUX2 + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_C + k, 7);
                    self.ram.write_u8(SPRITE_DELAY_AUX2 + k, 39);
                    self.runtime.sfx1_with_pan(self.ram, k, 0x13);
                } else if d >= 224 {
                    self.ram.write_u8(
                        SPRITE_GRAPHICS + k,
                        [14, 13, 12, 10][((d - 224) >> 3) as usize],
                    );
                } else {
                    self.ram.write_u8(SPRITE_GRAPHICS + k, 14);
                }
            }
            7 => {
                let d = self.ram.read_u8(SPRITE_DELAY_AUX2 + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_C + k, 2);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
                    self.ram.write_u8(
                        SPRITE_D + k,
                        (self.ram.read_u8(SPRITE_Y_LO + k) >> 7).wrapping_add(2),
                    );
                    let x = self.ram.read_u8(SPRITE_X_LO + k);
                    self.ram
                        .write_u8(SPRITE_G + k, x.wrapping_shl(2) | (x >> 7));
                    self.ram.write_u8(SPRITE_X_VEL + k, 0);
                    self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                    self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + k, 0);
                } else {
                    self.ram
                        .write_u8(SPRITE_GRAPHICS + k, [10, 11, 12, 13, 14][(d >> 3) as usize]);
                }
            }
            _ => {}
        }
    }
    pub fn blind_draw(&mut self, k: usize) {
        const POOF: [DrawPiece; 37] = [
            (-16, -20, 0x0586, 2),
            (-11, -28, 0x0586, 2),
            (-23, -26, 0x0586, 2),
            (-8, -17, 0x0586, 2),
            (-20, -13, 0x0586, 2),
            (-16, -37, 0x0586, 2),
            (-27, -31, 0x0586, 2),
            (-10, -28, 0x0586, 2),
            (-5, -28, 0x0586, 2),
            (-20, -27, 0x0586, 2),
            (-27, -17, 0x0586, 2),
            (-4, -17, 0x0586, 2),
            (-16, -13, 0x0586, 2),
            (-18, -37, 0x458a, 2),
            (-5, -33, 0x458a, 2),
            (-32, -32, 0x058a, 2),
            (-23, -31, 0x458a, 2),
            (-15, -24, 0x458a, 2),
            (-23, -31, 0x458a, 2),
            (-15, -24, 0x458a, 2),
            (-29, -22, 0x058a, 2),
            (-5, -22, 0x058a, 2),
            (-16, -14, 0x058a, 2),
            (-12, -32, 0x458a, 2),
            (-26, -29, 0x458a, 2),
            (-6, -22, 0x458a, 2),
            (-19, -20, 0x058a, 2),
            (-26, -29, 0x458a, 2),
            (-6, -22, 0x458a, 2),
            (-19, -20, 0x058a, 2),
            (-17, -27, 0x059b, 0),
            (-10, -26, 0x059b, 0),
            (0, -22, 0x459b, 0),
            (-19, -16, 0x459b, 0),
            (-6, -12, 0x059b, 0),
            (0, 13, 0x0b20, 2),
            (0, 23, 0x0b22, 2),
        ];
        const POOF_OFFSETS: [usize; 8] = [0, 1, 5, 13, 23, 30, 35, 37];
        const BODY: [DrawPiece; 105] = [
            (-8, 7, 0x0c8e, 2),
            (8, 7, 0x4c8e, 2),
            (-8, 23, 0x0ca0, 2),
            (8, 23, 0x4ca4, 2),
            (0, 0, 0x0a8c, 2),
            (-19, 3, 0x0aa6, 2),
            (19, 3, 0x4aa6, 2),
            (-8, 7, 0x0c8e, 2),
            (8, 7, 0x4c8e, 2),
            (-8, 23, 0x0ca2, 2),
            (8, 23, 0x4ca0, 2),
            (0, 0, 0x0a8c, 2),
            (-19, 3, 0x0aa8, 2),
            (19, 3, 0x4aa8, 2),
            (-8, 7, 0x0c8e, 2),
            (8, 7, 0x4c8e, 2),
            (-8, 23, 0x0ca4, 2),
            (8, 23, 0x4ca2, 2),
            (0, 0, 0x0a8c, 2),
            (-19, 3, 0x0aaa, 2),
            (19, 3, 0x4aaa, 2),
            (-15, 5, 0x0aa6, 2),
            (-6, 7, 0x0c8e, 2),
            (6, 7, 0x4c8e, 2),
            (-6, 23, 0x0ca4, 2),
            (6, 23, 0x4ca0, 2),
            (0, 0, 0x0a8a, 2),
            (16, -1, 0x4aa6, 2),
            (-11, 9, 0x0aa6, 2),
            (-4, 7, 0x0c8e, 2),
            (5, 7, 0x4c8e, 2),
            (-4, 23, 0x0ca4, 2),
            (5, 23, 0x4ca0, 2),
            (0, 0, 0x0a88, 2),
            (10, -2, 0x4aa6, 2),
            (0, 0, 0x0a84, 2),
            (13, 8, 0x4aa6, 2),
            (-10, -2, 0x0aa6, 2),
            (-5, 7, 0x0c8e, 2),
            (5, 7, 0x4c8e, 2),
            (-5, 23, 0x0ca0, 2),
            (5, 23, 0x4ca4, 2),
            (0, 0, 0x0a82, 2),
            (18, 4, 0x4aa6, 2),
            (-15, -1, 0x0aa6, 2),
            (-6, 7, 0x0c8e, 2),
            (6, 7, 0x4c8e, 2),
            (-6, 23, 0x0ca0, 2),
            (6, 23, 0x4ca4, 2),
            (0, 0, 0x0a80, 2),
            (-19, 3, 0x0aa6, 2),
            (19, 3, 0x4aa6, 2),
            (-8, 7, 0x0c8e, 2),
            (8, 7, 0x4c8e, 2),
            (-8, 23, 0x0ca0, 2),
            (8, 23, 0x4ca4, 2),
            (0, 0, 0x0a80, 2),
            (-19, 3, 0x0aa8, 2),
            (19, 3, 0x4aa8, 2),
            (-8, 7, 0x0c8e, 2),
            (8, 7, 0x4c8e, 2),
            (-8, 23, 0x0ca2, 2),
            (8, 23, 0x4ca0, 2),
            (0, 0, 0x0a80, 2),
            (-19, 3, 0x0aaa, 2),
            (19, 3, 0x4aaa, 2),
            (-8, 7, 0x0c8e, 2),
            (8, 7, 0x4c8e, 2),
            (-8, 23, 0x0ca0, 2),
            (8, 23, 0x4ca4, 2),
            (-8, 9, 0x0c8e, 2),
            (8, 9, 0x4c8e, 2),
            (-8, 23, 0x0cae, 2),
            (8, 23, 0x4cae, 2),
            (8, 23, 0x4cae, 2),
            (8, 23, 0x4cae, 2),
            (0, 2, 0x0a8c, 2),
            (-8, 16, 0x0c8e, 2),
            (8, 16, 0x4c8e, 2),
            (-8, 23, 0x0cae, 2),
            (8, 23, 0x4cae, 2),
            (8, 23, 0x4cae, 2),
            (8, 23, 0x4cae, 2),
            (0, 9, 0x0a8c, 2),
            (-8, 23, 0x0cae, 2),
            (8, 23, 0x4cae, 2),
            (8, 23, 0x4cae, 2),
            (8, 23, 0x4cae, 2),
            (8, 23, 0x4cae, 2),
            (8, 23, 0x4cae, 2),
            (0, 16, 0x0a8c, 2),
            (-8, 23, 0x0cac, 2),
            (8, 23, 0x4cac, 2),
            (8, 23, 0x4cac, 2),
            (8, 23, 0x4cac, 2),
            (8, 23, 0x4cac, 2),
            (8, 23, 0x4cac, 2),
            (0, 20, 0x0a8c, 2),
            (-8, 23, 0x0cac, 2),
            (8, 23, 0x4cac, 2),
            (8, 23, 0x4cac, 2),
            (8, 23, 0x4cac, 2),
            (8, 23, 0x4cac, 2),
            (8, 23, 0x4cac, 2),
            (0, 23, 0x0a8c, 2),
        ];
        const HEAD_CHARS: [u8; 16] = [
            0x86, 0x86, 0x84, 0x82, 0x80, 0x82, 0x84, 0x86, 0x86, 0x86, 0x88, 0x8a, 0x8c, 0x8a,
            0x88, 0x86,
        ];
        const HEAD_FLAGS: [u8; 16] = [
            0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0,
        ];
        const HEAD_OAM_INDEX: [usize; 10] = [4, 4, 4, 5, 5, 0, 0, 0, 0, 0];

        let graphics = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize;
        if graphics >= 15 {
            let frame = graphics - 15;
            let start = POOF_OFFSETS[frame];
            let end = POOF_OFFSETS[frame + 1];
            self.runtime.draw_multiple(self.ram, k, &POOF[start..end]);
            return;
        }

        self.runtime
            .draw_multiple(self.ram, k, &BODY[graphics * 7..graphics * 7 + 7]);
        if self.ram.read_u8(SPRITE_WALLCOLL + k) == 0 {
            if self.ram.read_u8(SPRITE_C + k) == 6 {
                self.runtime.set_current_oam_y(self.ram, 6, 0xf0);
                return;
            }
            if self.ram.read_u8(SPRITE_C + k) == 4 {
                return;
            }
        }
        if graphics >= 10 {
            return;
        }
        let head = self.ram.read_u8(SPRITE_HEAD_DIR + k) as usize;
        self.runtime.set_oam_char_flags(
            self.ram,
            HEAD_OAM_INDEX[graphics],
            HEAD_CHARS[head],
            HEAD_FLAGS[head],
        );
    }
    pub fn blind_spit_fireball(&mut self, k: usize, mask: u8) -> Option<usize> {
        const X: [i8; 16] = [
            -32, -28, -24, -16, 0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28,
        ];
        const Y: [i8; 16] = [
            0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
        ];
        if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & mask != 0 {
            return None;
        }
        let j = self.runtime.spawn_fireball(self.ram, k)?;
        self.runtime.sfx3_with_pan(self.ram, k, 0x19);
        let i = self.ram.read_u8(SPRITE_HEAD_DIR + k) as usize;
        self.ram.write_u8(SPRITE_X_VEL + j, X[i] as u8);
        self.ram.write_u8(SPRITE_Y_VEL + j, Y[i] as u8);
        self.ram.write_u8(
            SPRITE_DEFL_BITS + j,
            self.ram.read_u8(SPRITE_DEFL_BITS + j) | 8,
        );
        self.ram.write_u8(SPRITE_BUMP_DAMAGE + j, 4);
        Some(j)
    }
    pub fn spawn_boss_poof(&mut self, k: usize) -> Option<usize> {
        let j = self.runtime.spawn_dynamic(self.ram, k, 0xce)?;
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.set_sprite_x(j, self.sprite_x(j).wrapping_add(16));
        self.set_sprite_y(j, self.sprite_y(j).wrapping_add(40));
        self.ram.write_u8(SPRITE_GRAPHICS + j, 15);
        self.ram.write_u8(SPRITE_A + j, 1);
        self.ram.write_u8(SPRITE_DELAY_MAIN + j, 47);
        self.ram.write_u8(SPRITE_FLAGS2 + j, 9);
        self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 9);
        self.ram.write_u8(SOUND_EFFECT_1, 12);
        Some(j)
    }
    pub fn blind_fireball_flurry(&mut self, k: usize, a: u8) {
        self.ram.write_u8(
            SPRITE_WALLCOLL + k,
            self.ram.read_u8(SPRITE_WALLCOLL + k).wrapping_sub(1),
        );
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, (a & 7) * 2 + 1);
        let e = self.ram.read_u8(SPRITE_E + k).wrapping_sub(1);
        self.ram.write_u8(SPRITE_E + k, e);
        if (e as i8) < 0 {
            self.ram
                .write_u8(SPRITE_E + k, self.ram.read_u8(SPRITE_SUBTYPE + k));
            self.ram.write_u8(
                SPRITE_HEAD_DIR + k,
                self.ram.read_u8(SPRITE_HEAD_DIR + k).wrapping_add(1) & 15,
            );
        }
        if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 31 == 0
            && self.ram.read_u8(SPRITE_SUBTYPE + k) != 5
        {
            self.ram.write_u8(
                SPRITE_SUBTYPE + k,
                self.ram.read_u8(SPRITE_SUBTYPE + k).wrapping_add(1),
            );
        }
        self.blind_animate_robes(k);
        let _ = self.blind_spit_fireball(k, 15);
    }
    pub fn blind_decelerate_x(&mut self, k: usize) {
        let v = self.ram.read_u8(SPRITE_X_VEL + k);
        if v != 0 {
            self.ram.write_u8(
                SPRITE_X_VEL + k,
                v.wrapping_add(if (v as i8) < 0 { 2 } else { (-2i8) as u8 }),
            );
        }
        self.blind_animate_robes(k);
        let w = self.ram.read_u8(SPRITE_WALLCOLL + k);
        if w != 0 {
            self.blind_fireball_flurry(k, w);
        }
    }
    pub fn blind_decelerate_y(&mut self, k: usize) {
        let v = self.ram.read_u8(SPRITE_Y_VEL + k);
        if v != 0 {
            self.ram.write_u8(
                SPRITE_Y_VEL + k,
                v.wrapping_add(if (v as i8) < 0 { 4 } else { (-4i8) as u8 }),
            );
        }
        self.runtime.move_y(self.ram, k);
        let w = self.ram.read_u8(SPRITE_WALLCOLL + k);
        if w != 0 {
            self.blind_fireball_flurry(k, w);
        }
    }
    pub fn blind_check_bump_damage(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_DELAY_AUX4 + k) | self.ram.read_u8(SPRITE_F + k) == 0 {
            self.runtime.check_damage_to_and_from_link(self.ram, k);
        }
        if self
            .ram
            .read_u16(LINK_X)
            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
            .wrapping_add(14)
            < 28
            && self
                .ram
                .read_u16(LINK_Y)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                < 28
            && self.ram.read_u8(COUNTDOWN_FOR_BLINK) | self.ram.read_u8(LINK_DISABLE_SPRITE_DAMAGE)
                == 0
        {
            self.ram.write_u8(LINK_AUXILIARY_STATE, 1);
            self.ram.write_u8(LINK_GIVE_DAMAGE, 8);
            self.ram.write_u8(LINK_INCAPACITATED_TIMER, 16);
            self.ram
                .write_u8(LINK_ACTUAL_VEL_X, self.ram.read_u8(LINK_ACTUAL_VEL_X) ^ 255);
            self.ram
                .write_u8(LINK_ACTUAL_VEL_Y, self.ram.read_u8(LINK_ACTUAL_VEL_Y) ^ 255);
        }
    }
    pub fn blind_animate(&mut self, k: usize) {
        const H: [u8; 17] = [0, 1, 2, 3, 4, 3, 2, 1, 0, 15, 14, 13, 12, 13, 14, 15, 0];
        const T: [u8; 8] = [0, 1, 1, 2, 2, 3, 3, 4];
        if self.ram.read_u8(SPRITE_WALLCOLL + k) == 0 {
            let mut t = T[(self.ram.read_u8(LINK_X) >> 5) as usize] as i8;
            if self.ram.read_u8(SPRITE_D + k) == 3 {
                t = -t;
            }
            let base = (self.ram.read_u8(SPRITE_D + k).wrapping_sub(2) * 8) as usize;
            let q = self.ram.read_u8(BYTE_7E0B69);
            let idx = (q >> 3 & 7) as usize + (q >> 2 & 1) as usize + base;
            self.ram
                .write_u8(SPRITE_HEAD_DIR + k, H[idx].wrapping_add_signed(t) & 15);
        }
        self.blind_animate_robes(k);
    }
    pub fn blind_animate_robes(&mut self, k: usize) {
        const G: [u8; 8] = [7, 8, 9, 8, 0, 1, 2, 1];
        let i = (self.ram.read_u8(SPRITE_SUBTYPE2 + k) >> 3 & 3)
            + self
                .ram
                .read_u8(SPRITE_D + k)
                .wrapping_sub(2)
                .wrapping_shl(2);
        self.ram.write_u8(SPRITE_GRAPHICS + k, G[i as usize]);
    }
    pub fn blind_spawn_laser(&mut self, k: usize) {
        const X: [i8; 16] = [-8, -8, -8, -4, 0, 4, 8, 8, 8, 8, 8, 4, 0, -4, -8, -8];
        const Y: [i8; 16] = [0, 0, 4, 8, 8, 8, 4, 0, 0, 0, -4, -8, -8, -8, -4, 0];
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xce) {
            let sound = self.runtime.calculate_sfx_pan(self.ram, k) | 0x26;
            self.ram.write_u8(SOUND_EFFECT_2, sound);
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(
                SPRITE_X_LO + j,
                self.ram.read_u8(SPRITE_X_LO + j).wrapping_add(4),
            );
            let i = self.ram.read_u8(SPRITE_HEAD_DIR + k) as usize;
            self.ram.write_u8(SPRITE_HEAD_DIR + j, i as u8);
            self.ram.write_u8(SPRITE_X_VEL + j, X[i] as u8);
            self.ram.write_u8(SPRITE_Y_VEL + j, Y[i] as u8);
            self.ram.write_u8(SPRITE_A + j, 128);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 128);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 0x40);
            self.ram.write_u8(SPRITE_FLAGS4 + j, 0x14);
        }
    }

    pub fn sprite_cc_cd_common(&mut self, k: usize) {
        if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 {
            let m = if self.runtime.is_right_of_link(self.ram, k).0 != 0 {
                -1i8
            } else {
                1
            };
            let v = self.ram.read_u8(SPRITE_X_VEL + k);
            let target = (m as i16 * 16) as i8 as u8;
            if v != target {
                self.ram
                    .write_u8(SPRITE_X_VEL + k, v.wrapping_add_signed(m));
            }
        }
        if self.runtime.check_tile_collision(self.ram, k) {
            self.ram.write_u8(SPRITE_STATE + k, 0);
        }
    }
    pub fn sprite_cd_spawn_garnish(&mut self, k: usize) {
        let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
        self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
        if s & 7 != 0 {
            return;
        }
        self.runtime.sfx3_with_pan(self.ram, k, 0x14);
        let j = self.runtime.garnish_alloc_overwrite_old(self.ram);
        self.ram.write_u8(GARNISH_ACTIVE, 0xc);
        self.ram.write_u8(GARNISH_TYPE + j, 0xc);
        self.ram.write_u8(GARNISH_SPRITE + j, k as u8);
        let x = self.sprite_x(k);
        let y = self.sprite_y(k).wrapping_add(16);
        self.ram.write_u8(GARNISH_X_LO + j, x as u8);
        self.ram.write_u8(GARNISH_X_HI + j, (x >> 8) as u8);
        self.ram.write_u8(GARNISH_Y_LO + j, y as u8);
        self.ram.write_u8(GARNISH_Y_HI + j, (y >> 8) as u8);
        self.ram.write_u8(GARNISH_COUNTDOWN + j, 127);
    }
    pub fn sprite_trinexx_fire_add_fire_garnish(&mut self, k: usize) {
        let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
        self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
        if s & 7 == 0 {
            self.runtime.sfx2_with_pan(self.ram, k, 0x2a);
            self.garnish_flame_trail(k, false);
        }
    }
    pub fn garnish_flame_trail(&mut self, k: usize, low: bool) -> usize {
        let j = if low {
            self.runtime.garnish_alloc_overwrite_old_low(self.ram)
        } else {
            self.runtime.garnish_alloc_overwrite_old(self.ram)
        };
        self.ram.write_u8(GARNISH_ACTIVE, 0x10);
        self.ram.write_u8(GARNISH_TYPE + j, 0x10);
        self.ram.write_u8(GARNISH_SPRITE + j, k as u8);
        let x = self.sprite_x(k);
        let y = self.sprite_y(k).wrapping_add(16);
        self.ram.write_u8(GARNISH_X_LO + j, x as u8);
        self.ram.write_u8(GARNISH_X_HI + j, (x >> 8) as u8);
        self.ram.write_u8(GARNISH_Y_LO + j, y as u8);
        self.ram.write_u8(GARNISH_Y_HI + j, (y >> 8) as u8);
        self.ram.write_u8(GARNISH_COUNTDOWN + j, 127);
        j
    }
    pub fn sprite_ca_chain_chomp(&mut self, k: usize) {
        self.chain_chomp_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.chain_chomp_handle_leash(k);
        if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0
            && (self.ram.read_u8(SPRITE_X_VEL + k) | self.ram.read_u8(SPRITE_Y_VEL + k)) != 0
        {
            let a = self.runtime.convert_velocity_to_angle(
                self.ram.read_u8(SPRITE_X_VEL + k),
                self.ram.read_u8(SPRITE_Y_VEL + k),
            ) & 15;
            self.ram.write_u8(SPRITE_D + k, a);
        }
        self.runtime.move_xyz(self.ram, k);
        self.ram.write_u8(
            SPRITE_Z_VEL + k,
            self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
        );
        if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
            self.ram.write_u8(SPRITE_Z + k, 0);
            self.ram.write_u8(SPRITE_Z_VEL + k, 0);
        }
        self.runtime.get_16bit_coords(self.ram, k);
        let x = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_A + k),
            self.ram.read_u8(SPRITE_B + k),
        ]);
        let y = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_C + k),
            self.ram.read_u8(SPRITE_G + k),
        ]);
        let close = self
            .ram
            .read_u16(CUR_SPRITE_X)
            .wrapping_sub(x)
            .wrapping_add(48)
            < 96
            && self
                .ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_sub(y)
                .wrapping_add(48)
                < 96;
        self.ram.write_u8(SPRITE_ANIM_CLOCK + k, u8::from(close));
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    const X: [i8; 16] = [
                        0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
                    ];
                    const Y: [i8; 16] = [
                        -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
                    ];
                    let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
                    self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
                    if s == 4 {
                        let j = (self.runtime.random_number(self.ram) & 15) as usize;
                        self.ram.write_u8(SPRITE_SUBTYPE2 + k, 0);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                        self.ram
                            .write_u8(SPRITE_X_VEL + k, X[j].wrapping_mul(4) as u8);
                        self.ram
                            .write_u8(SPRITE_Y_VEL + k, Y[j].wrapping_mul(4) as u8);
                        let _ = self.runtime.random_number(self.ram);
                        self.runtime.apply_speed_towards_link(self.ram, k, 64);
                        self.runtime.sfx3_with_pan(self.ram, k, 4);
                    } else {
                        let delay = (self.runtime.random_number(self.ram) & 31) + 16;
                        let j = (self.runtime.random_number(self.ram) & 15) as usize;
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, delay);
                        self.ram.write_u8(SPRITE_X_VEL + k, X[j] as u8);
                        self.ram.write_u8(SPRITE_Y_VEL + k, Y[j] as u8);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    }
                } else {
                    self.ram.write_u8(SPRITE_X_VEL + k, 0);
                    self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                }
            }
            1 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                }
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) & 15 == 0 {
                    self.chain_chomp_move_chain(k);
                }
                if self.ram.read_u8(SPRITE_Z + k) == 0 {
                    self.ram.write_u8(SPRITE_Z_VEL + k, 16);
                }
                if !close {
                    let (xv, yv) = self
                        .runtime
                        .project_speed_towards_location(self.ram, k, x, y, 16);
                    self.ram.write_u8(SPRITE_X_VEL + k, xv);
                    self.ram.write_u8(SPRITE_Y_VEL + k, yv);
                    self.runtime.move_xy(self.ram, k);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 12);
                }
            }
            2 => {
                if !close {
                    self.ram.write_u8(
                        SPRITE_X_VEL + k,
                        (0u8).wrapping_sub(self.ram.read_u8(SPRITE_X_VEL + k)),
                    );
                    self.ram.write_u8(
                        SPRITE_Y_VEL + k,
                        (0u8).wrapping_sub(self.ram.read_u8(SPRITE_Y_VEL + k)),
                    );
                    self.runtime.move_xy(self.ram, k);
                    self.ram.write_u8(SPRITE_X_VEL + k, 0);
                    self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                    self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 48);
                }
                self.chain_chomp_move_chain(k);
                self.chain_chomp_move_chain(k);
            }
            3 => {
                if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
                }
                self.chain_chomp_move_chain(k);
                self.chain_chomp_move_chain(k);
            }
            _ => {}
        }
    }

    fn chain_hist_x(&self, p: usize) -> u16 {
        self.ram.read_u16(CHAINCHOMP_X_HIST + p * 2)
    }
    fn chain_hist_y(&self, p: usize) -> u16 {
        self.ram.read_u16(CHAINCHOMP_Y_HIST + p * 2)
    }
    fn set_chain_hist_x(&mut self, p: usize, v: u16) {
        self.ram.write_u16(CHAINCHOMP_X_HIST + p * 2, v)
    }
    fn set_chain_hist_y(&mut self, p: usize, v: u16) {
        self.ram.write_u16(CHAINCHOMP_Y_HIST + p * 2, v)
    }
    pub fn chain_chomp_move_chain(&mut self, k: usize) {
        const M: [u8; 6] = [205, 154, 102, 51, 8, 0xbd];
        let x = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_A + k),
            self.ram.read_u8(SPRITE_B + k),
        ]);
        let y = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_C + k),
            self.ram.read_u8(SPRITE_G + k),
        ]);
        let mut p = k * 8;
        let dx = self.chain_hist_x(p).wrapping_sub(x);
        let dy = self.chain_hist_y(p).wrapping_sub(y);
        p += 1;
        for i in (0..6).rev() {
            let x3 = x.wrapping_add(chain_chomp_one_mult(dx, M[(p & 7) - 1]));
            let y3 = y.wrapping_add(chain_chomp_one_mult(dy, M[(p & 7) - 1]));
            let d = self.chain_hist_x(p).wrapping_sub(x3);
            if d != 0 {
                self.set_chain_hist_x(
                    p,
                    self.chain_hist_x(p)
                        .wrapping_add(if (d as i16) < 0 { 1 } else { u16::MAX }),
                );
            }
            let d = self.chain_hist_y(p).wrapping_sub(y3);
            if d != 0 {
                self.set_chain_hist_y(
                    p,
                    self.chain_hist_y(p)
                        .wrapping_add(if (d as i16) < 0 { 1 } else { u16::MAX }),
                );
            }
            let _ = i;
            p += 1;
        }
    }
    pub fn chain_chomp_handle_leash(&mut self, k: usize) {
        let mut p = k * 8;
        self.set_chain_hist_x(p, self.ram.read_u16(CUR_SPRITE_X));
        self.set_chain_hist_y(p, self.ram.read_u16(CUR_SPRITE_Y));
        for _ in 0..6 {
            let x = self.chain_hist_x(p).wrapping_sub(self.chain_hist_x(p + 1));
            if (x.wrapping_sub(8) as i16) >= 0 {
                self.set_chain_hist_x(p + 1, self.chain_hist_x(p).wrapping_sub(8));
            } else if (x.wrapping_add(8) as i16) < 0 {
                self.set_chain_hist_x(p + 1, self.chain_hist_x(p).wrapping_add(8));
            }
            let y = self.chain_hist_y(p).wrapping_sub(self.chain_hist_y(p + 1));
            if (y.wrapping_sub(8) as i16) >= 0 {
                self.set_chain_hist_y(p + 1, self.chain_hist_y(p).wrapping_sub(8));
            } else if (y.wrapping_add(8) as i16) < 0 {
                self.set_chain_hist_y(p + 1, self.chain_hist_y(p).wrapping_add(8));
            }
            p += 1;
        }
    }
    pub fn chain_chomp_draw(&mut self, k: usize) {
        const G: [u8; 16] = [0, 1, 2, 3, 3, 3, 2, 1, 0, 0, 0, 4, 4, 4, 0, 0];
        const F: [u8; 16] = [
            0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40,
        ];
        let d = self.ram.read_u8(SPRITE_D + k) as usize;
        self.ram.write_u8(SPRITE_GRAPHICS + k, G[d]);
        self.ram.write_u8(
            SPRITE_OAM_FLAGS + k,
            self.ram.read_u8(SPRITE_OAM_FLAGS + k) & 0x3f | F[d],
        );
        self.runtime.draw_single_large(self.ram, k);
        let first = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4 + 1;
        let flags =
            (self.ram.read_u8(SPRITE_OAM_FLAGS + k) ^ self.ram.read_u8(SPRITE_OBJ_PRIO + k)) & 0xf0
                | 0x0d;
        let radius = u16::from((self.ram.read_u8(SPRITE_DELAY_AUX1 + k) & 1) + 4);
        for entry in 0..6usize {
            let history = k * 8 + entry;
            let x = self
                .chain_hist_x(history)
                .wrapping_add(radius)
                .wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2));
            let y = self
                .chain_hist_y(history)
                .wrapping_add(radius)
                .wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2));
            crate::oam::set_helper0(self.ram, first + entry, x, y, 0x8b, flags, 0);
        }
    }
    pub fn sprite_c9_tektite(&mut self, k: usize) {
        let j = self.ram.read_u8(SPRITE_ANIM_CLOCK + k);
        if j != 0 {
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + k, j);
            self.ram.write_u8(SPRITE_OBJ_PRIO + k, 0x30);
        }
        match j {
            0 => self.sprite_tektite(k),
            1 => self.sprite_phantom_ganon(k),
            2 => self.sprite_ganon_trident(k),
            3 => self.sprite_spiral_fire_bat(k),
            4 => self.sprite_fire_bat_launched(k),
            5 => self.sprite_fire_bat_trailer(k),
            _ => {}
        }
    }
    pub fn tektite_draw(&mut self, k: usize) {
        const D: [DrawPiece; 6] = [
            (-8, 0, 0x00c8, 2),
            (8, 0, 0x40c8, 2),
            (-8, 0, 0x00ca, 2),
            (8, 0, 0x40ca, 2),
            (-8, 0, 0x00ea, 2),
            (8, 0, 0x40ea, 2),
        ];
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }
    pub fn sprite_tektite(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) != 0 {
            self.ram.write_u8(SPRITE_GRAPHICS + k, 0);
        }
        self.tektite_draw(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.runtime.move_xyz(self.ram, k);
        let _ = self.runtime.bounce_from_tile_collision(self.ram, k);
        self.ram.write_u8(
            SPRITE_Z_VEL + k,
            self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(1),
        );
        if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
            self.ram.write_u8(SPRITE_Z + k, 0);
            self.ram.write_u8(SPRITE_Z_VEL + k, 0);
        }
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                const DIR: [u8; 4] = [3, 2, 1, 0];
                const XV: [u8; 4] = [16, 240, 16, 240];
                const YV: [u8; 4] = [16, 16, 240, 240];
                let direction = self.runtime.direction_to_face_link(self.ram, k);
                let (dx, dy) = self
                    .runtime
                    .direction_to_face_link_with_differences(self.ram, k);
                if dx.wrapping_add(40) < 80
                    && dy.wrapping_add(40) < 80
                    && self.ram.read_u8(0x44) != 0x80
                    && self.ram.read_u8(SPRITE_Z + k) | self.ram.read_u8(SPRITE_PAUSE + k) == 0
                    && self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL)
                        == self.ram.read_u8(SPRITE_FLOOR + k)
                    && direction != DIR[usize::from(self.ram.read_u8(0x2f) >> 1)]
                {
                    let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 32);
                    self.ram.write_u8(SPRITE_X_VEL + k, x.wrapping_neg());
                    self.ram.write_u8(SPRITE_Y_VEL + k, y.wrapping_neg());
                    self.ram.write_u8(SPRITE_Z_VEL + k, 16);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    return;
                }
                let delay = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if delay == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    let hops = self.ram.read_u8(SPRITE_B + k).wrapping_add(1);
                    self.ram.write_u8(SPRITE_B + k, hops);
                    let j = if hops == 4 {
                        self.ram.write_u8(SPRITE_B + k, 0);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                        let timer = (self.runtime.random_number(self.ram) & 63) + 48;
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, timer);
                        self.ram.write_u8(SPRITE_Z_VEL + k, 12);
                        usize::from(self.runtime.is_below_link(self.ram, k)) * 2
                            + usize::from(self.runtime.is_right_of_link(self.ram, k).0)
                    } else {
                        let zv = (self.runtime.random_number(self.ram) & 7) + 24;
                        self.ram.write_u8(SPRITE_Z_VEL + k, zv);
                        usize::from(self.runtime.random_number(self.ram) & 3)
                    };
                    self.ram.write_u8(SPRITE_X_VEL + k, XV[j]);
                    self.ram.write_u8(SPRITE_Y_VEL + k, YV[j]);
                } else {
                    self.ram.write_u8(SPRITE_GRAPHICS + k, delay >> 4 & 1);
                }
            }
            1 => {
                if self.ram.read_u8(SPRITE_Z + k) == 0 {
                    self.tektite_reset_state(k);
                } else {
                    self.ram.write_u8(SPRITE_GRAPHICS + k, 2);
                }
            }
            2 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.tektite_reset_state(k);
                } else {
                    if self.ram.read_u8(SPRITE_Z + k) == 0 {
                        self.ram.write_u8(SPRITE_Z_VEL + k, 12);
                        self.ram.write_u8(SPRITE_Z + k, 1);
                        self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 8);
                    }
                    self.ram.write_u8(SPRITE_GRAPHICS + k, 2);
                }
            }
            _ => {}
        }
    }

    fn tektite_reset_state(&mut self, k: usize) {
        self.ram.write_u8(SPRITE_AI_STATE + k, 0);
        let delay = (self.runtime.random_number(self.ram) & 63) + 72;
        self.ram.write_u8(SPRITE_DELAY_MAIN + k, delay);
        self.ram.write_u8(SPRITE_X_VEL + k, 0);
        self.ram.write_u8(SPRITE_Y_VEL + k, 0);
    }

    pub fn helmasaur_king_initialize(&mut self, k: usize) {
        self.ram.write_u8(OVERLORD_GEN1 + 7, 0x30);
        self.ram.write_u8(OVERLORD_GEN1 + 5, 0x80);
        self.ram.write_u8(OVERLORD_GEN1 + 6, 0);
        self.ram.write_u8(OVERLORD_GEN2, 0);
        self.ram.write_u8(OVERLORD_GEN2 + 3, 0);
        self.ram.write_u8(OVERLORD_GEN2 + 1, 0);
        self.ram.write_u8(OVERLORD_GEN2 + 2, 0);
        self.helmasaur_king_reinitialize(k);
    }
    pub fn helmasaur_king_reinitialize(&mut self, k: usize) {
        const T: [u8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 6, 5, 4,
            3, 2, 1,
        ];
        let t = self.ram.read_u8(SPRITE_SUBTYPE2 + k) as usize;
        for i in (0..4).rev() {
            self.ram.write_u8(OVERLORD_X_LO + i, T[(t + i * 8) & 31]);
        }
    }
    pub fn helmasaur_king_handle_movement(&mut self, k: usize) {
        let mut n = 1
            + usize::from(self.ram.read_u8(FRAME_COUNTER) & 3 == 0)
            + usize::from(self.ram.read_u8(SPRITE_C + k) >= 3);
        while n != 0 {
            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
            if s & 15 == 0 {
                self.ram.write_u8(SOUND_EFFECT_1, 0x21);
            }
            n -= 1;
        }
        self.runtime.move_xy(self.ram, k);
    }
    pub fn helmasaur_king_attempt_damage(&mut self, k: usize) {
        if self.ram.read_u8(FRAME_COUNTER) & 7 == 0
            && self
                .ram
                .read_u16(LINK_X)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                .wrapping_add(36)
                < 72
            && self
                .ram
                .read_u16(LINK_Y)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                .wrapping_add(40)
                < 64
        {
            self.runtime.attempt_damage_to_link_plus_recoil(self.ram, k);
        }
    }

    pub fn helmasaur_king_maybe_fireball(&mut self, k: usize) -> bool {
        let s = self.ram.read_u8(SPRITE_SUBTYPE + k).wrapping_add(1);
        self.ram.write_u8(SPRITE_SUBTYPE + k, s);
        if s != 4 {
            return false;
        }
        self.ram.write_u8(SPRITE_SUBTYPE + k, 0);
        if self.runtime.random_number(self.ram) & 1 != 0 {
            self.ram.write_u8(SPRITE_DELAY_AUX2 + k, 127);
            self.runtime.sfx3_with_pan(self.ram, k, 0x2a);
        } else {
            self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 160);
        }
        true
    }
    pub fn helmasaur_king_chip_away_at_mask(&mut self, k: usize) {
        self.ram
            .write_u8(TMP_COUNTER, self.ram.read_u8(SPRITE_C + k).wrapping_add(7));
        self.helmasaur_king_spawn_mask_debris(k);
        self.runtime.sfx2_with_pan(self.ram, k, 0x1f);
    }
    pub fn helmasaur_king_explode_mask(&mut self, k: usize) {
        for j in 1..16 {
            self.ram.write_u8(SPRITE_STATE + j, 0);
        }
        self.ram.write_u8(TMP_COUNTER, 7);
        loop {
            self.helmasaur_king_spawn_mask_debris(k);
            let t = self.ram.read_u8(TMP_COUNTER).wrapping_sub(1);
            self.ram.write_u8(TMP_COUNTER, t);
            if (t as i8) < 0 {
                break;
            }
        }
        self.runtime.sfx2_with_pan(self.ram, k, 0x1f);
    }
    pub fn helmasaur_king_spit_fireball(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x70) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.set_sprite_y(j, self.sprite_y(j).wrapping_add(28));
            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 32);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 32);
        }
    }

    pub fn helmasaur_king_spawn_mask_debris(&mut self, k: usize) {
        const X: [i8; 10] = [-16, 0, 16, -16, 0, 16, -8, 8, -16, 16];
        const Y: [i8; 10] = [24, 27, 24, 24, 27, 24, 27, 27, 24, 24];
        const Z: [i8; 10] = [29, 32, 29, 13, 16, 13, 0, 0, 13, 13];
        const XV: [i8; 10] = [-16, -4, 14, -12, 4, 18, -2, 2, -12, 18];
        const YV: [i8; 10] = [-8, -4, -6, 4, 2, 7, 6, 8, 4, 7];
        const ZV: [i8; 10] = [32, 40, 36, 37, 39, 34, 30, 33, 37, 34];
        const F: [u8; 10] = [0, 0, 0x40, 0, 0, 0x40, 0, 0x40, 0, 0x40];
        const G: [u8; 10] = [0, 1, 0, 2, 3, 2, 4, 4, 5, 5];
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x92) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            let i = self.ram.read_u8(TMP_COUNTER) as usize;
            self.set_sprite_x(j, self.sprite_x(j).wrapping_add_signed(X[i] as i16));
            self.set_sprite_y(j, self.sprite_y(j).wrapping_add_signed(Y[i] as i16));
            self.ram.write_u8(SPRITE_Z + j, Z[i] as u8);
            self.ram.write_u8(SPRITE_X_VEL + j, XV[i] as u8);
            self.ram.write_u8(SPRITE_Y_VEL + j, YV[i] as u8);
            self.ram.write_u8(SPRITE_Z_VEL + j, ZV[i] as u8);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, F[i] | 13);
            self.ram.write_u8(SPRITE_GRAPHICS + j, G[i]);
            self.ram.write_u8(SPRITE_C + j, 128);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 0);
            self.ram.write_u8(SPRITE_DELAY_AUX1 + j, 12);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 12);
            self.ram.write_u8(SPRITE_SUBTYPE + j, i as u8);
        }
    }

    pub fn sprite_initialized_segmented(&mut self, k: usize) {
        for i in 0..128 {
            self.ram
                .write_u8(CHAINCHOMP_X_HIST + i, self.ram.read_u8(SPRITE_X_LO + k));
            self.ram.write_u8(
                CHAINCHOMP_X_HIST + 128 + i,
                self.ram.read_u8(SPRITE_X_HI + k),
            );
            self.ram
                .write_u8(CHAINCHOMP_Y_HIST + i, self.ram.read_u8(SPRITE_Y_LO + k));
            self.ram.write_u8(
                CHAINCHOMP_Y_HIST + 128 + i,
                self.ram.read_u8(SPRITE_Y_HI + k),
            );
        }
    }
    pub fn giant_moldorm_incremental_segment_explosion(&mut self, k: usize) {
        let d = self.ram.read_u8(SPRITE_DELAY_AUX4 + k);
        if self.ram.read_u8(SPRITE_STATE + k) == 9
            && d != 0
            && d < 80
            && d & 15 | self.ram.read_u8(SUBMODULE_INDEX) | self.ram.read_u8(FLAG_UNK1) == 0
        {
            self.ram
                .write_u8(SPRITE_B + k, self.ram.read_u8(SPRITE_B + k).wrapping_add(1));
            self.sprite_make_boss_explosion(k);
        }
    }
    pub fn sprite_draw_moldorm_head(&mut self, k: usize) {
        const D: [DrawPiece; 16] = [
            (-8, -8, 0x0080, 2),
            (8, -8, 0x0082, 2),
            (-8, 8, 0x00a0, 2),
            (8, 8, 0x00a2, 2),
            (-8, -8, 0x4082, 2),
            (8, -8, 0x4080, 2),
            (-8, 8, 0x40a2, 2),
            (8, 8, 0x40a0, 2),
            (-6, -6, 0x0080, 2),
            (6, -6, 0x0082, 2),
            (-6, 6, 0x00a0, 2),
            (6, 6, 0x00a2, 2),
            (-6, -6, 0x4082, 2),
            (6, -6, 0x4080, 2),
            (-6, 6, 0x40a2, 2),
            (6, 6, 0x40a0, 2),
        ];
        let t = ((self.ram.read_u8(SPRITE_SUBTYPE2 + k) >> 1 & 1)
            + (self.ram.read_u8(SPRITE_DELAY_AUX1 + k) & 2)) as usize
            * 4;
        self.runtime.draw_multiple(self.ram, k, &D[t..t + 4]);
    }

    pub fn trinexx_components_initialize(&mut self, k: usize) {
        match self.ram.read_u8(SPRITE_TYPE + k) {
            0xcb => {
                self.ram.write_u8(
                    SPRITE_X_LO + k,
                    self.ram.read_u8(SPRITE_X_LO + k).wrapping_add(8),
                );
                self.ram.write_u8(
                    SPRITE_Y_LO + k,
                    self.ram.read_u8(SPRITE_Y_LO + k).wrapping_add(16),
                );
                self.trinexx_cache_position(k);
                for i in [2usize, 3, 5, 7] {
                    self.ram.write_u8(OVERLORD_X_LO + i, 0);
                }
                self.ram.write_u8(OVERLORD_X_HI, 0);
                self.ram.write_u8(OVERLORD_X_LO + 6, 255);
                self.trinexx_restore_xy(k);
            }
            0xcc | 0xcd => {
                if self.ram.read_u8(SPRITE_TYPE + k) == 0xcc {
                    self.ram.write_u8(SPRITE_GRAPHICS + k, 3);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
                } else {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                }
                for j in (0..=0x1a).rev() {
                    self.ram.write_u8(ALT_SPRITE_TYPE + j, 0x40);
                    self.ram.write_u8(ALT_SPRITE_X_HI + j, 0);
                    self.ram.write_u8(ALT_SPRITE_Y_HI + j, 0);
                }
                self.ram.write_u8(SPRITE_SUBTYPE2 + k, 1);
                self.trinexx_cache_position(k);
            }
            _ => {}
        }
    }
    pub fn trinexx_restore_xy(&mut self, k: usize) {
        self.ram
            .write_u8(SPRITE_X_LO + k, self.ram.read_u8(SPRITE_A + k));
        let y = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_C + k),
            self.ram.read_u8(SPRITE_G + k),
        ])
        .wrapping_add(12);
        self.set_sprite_y(k, y);
    }
    pub fn trinexx_cache_position(&mut self, k: usize) {
        self.ram
            .write_u8(SPRITE_A + k, self.ram.read_u8(SPRITE_X_LO + k));
        self.ram
            .write_u8(SPRITE_B + k, self.ram.read_u8(SPRITE_X_HI + k));
        self.ram
            .write_u8(SPRITE_C + k, self.ram.read_u8(SPRITE_Y_LO + k));
        self.ram
            .write_u8(SPRITE_G + k, self.ram.read_u8(SPRITE_Y_HI + k));
    }

    pub fn vitreous_animate(&mut self, k: usize, a: u8) {
        if (0x40..=0x42).contains(&a) {
            self.sprite_spawn_lightning(k);
        }
        self.ram.write_u8(SPRITE_GRAPHICS + k, 0);
        let (side, delta) = self.runtime.is_right_of_link(self.ram, k);
        if (delta as u8).wrapping_add(16) >= 32 {
            self.ram
                .write_u8(SPRITE_GRAPHICS + k, [2, 1][side as usize]);
        }
    }
    pub fn vitreous_set_minions_forth(&mut self, k: usize) {
        const W: [u8; 16] = [5, 6, 7, 8, 9, 10, 11, 12, 13, 5, 6, 7, 8, 9, 10, 11];
        let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
        self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
        if s & 63 == 0 {
            let j = W[(self.runtime.random_number(self.ram) & 15) as usize] as usize;
            if self.ram.read_u8(SPRITE_AI_STATE + j) == 0 {
                self.ram.write_u8(SPRITE_AI_STATE + j, 1);
                self.ram.write_u8(SOUND_EFFECT_1, 0x15);
            } else {
                self.ram.write_u8(SPRITE_SUBTYPE2 + k, s.wrapping_sub(1));
            }
        }
    }
    pub fn sprite_spawn_lightning(&mut self, k: usize) {
        const X: [i8; 8] = [-8, 8, 8, -8, 8, -8, -8, 8];
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xbf) {
            self.ram.write_u8(SOUND_EFFECT_2, 0x26);
            self.runtime.set_spawned_coordinates(self.ram, j);
            let i = (self.runtime.random_number(self.ram) & 7) as usize;
            self.ram.write_u8(SPRITE_A + j, i as u8);
            let old = self.sprite_x(j);
            let x = old.wrapping_add_signed(X[i] as i16);
            self.set_sprite_x(j, x);
            let carry = u8::from(u32::from(old) + u32::from(X[i] as u16) > 0xffff);
            self.ram.write_u8(
                SPRITE_Y_LO + j,
                self.ram
                    .read_u8(SPRITE_Y_LO + j)
                    .wrapping_add(12)
                    .wrapping_add(carry),
            );
            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 2);
            self.ram.write_u8(INTRO_TIMES_PAL_FLASH, 32);
        }
    }
    pub fn toppo_flustered(&mut self, k: usize) {
        self.ram.write_u8(SPRITE_FLAGS2 + k, 130);
        self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + k, 130);
        self.ram.write_u8(SPRITE_FLAGS3 + k, 73);
        let s = self.ram.read_u8(SPRITE_SUBTYPE + k);
        if s == 0 {
            if self.runtime.check_damage_to_link(self.ram, k) {
                self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x174);
                self.runtime.show_message_minimal(self.ram);
                self.ram.write_u8(SPRITE_SUBTYPE + k, 1);
            }
        } else if s < 16 {
            self.ram.write_u8(SPRITE_SUBTYPE + k, s + 1);
        } else if s == 16 {
            self.ram.write_u8(SPRITE_FLAGS5 + k, 0);
            self.ram.write_u8(SPRITE_STATE + k, 6);
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 15);
            self.ram.write_u8(
                SPRITE_FLAGS2 + k,
                self.ram.read_u8(SPRITE_FLAGS2 + k).wrapping_add(4),
            );
            self.runtime.sfx2_with_pan(self.ram, k, 0x15);
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x4d) {
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.runtime.force_prize_drop(self.ram, j, 6, 6);
            }
            self.ram.write_u8(SPRITE_SUBTYPE + k, 17);
        }
        let a = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
        self.ram.write_u8(SPRITE_SUBTYPE2 + k, a);
        self.ram.write_u8(SPRITE_GRAPHICS + k, ((a & 4) >> 2) + 3);
    }
    pub fn old_mountain_man_draw(&mut self, k: usize) {
        const D0: [DrawPiece; 2] = [(0, 0, 0x00ac, 2), (0, 8, 0x00ae, 2)];
        const D1: [DrawPiece; 16] = [
            (0, 0, 0x0120, 2),
            (0, 8, 0x0122, 2),
            (0, 1, 0x0120, 2),
            (0, 9, 0x4122, 2),
            (0, 0, 0x0120, 2),
            (0, 8, 0x0122, 2),
            (0, 1, 0x0120, 2),
            (0, 9, 0x4122, 2),
            (-2, 0, 0x0120, 2),
            (0, 8, 0x0122, 2),
            (-2, 1, 0x0120, 2),
            (0, 9, 0x0122, 2),
            (2, 0, 0x4120, 2),
            (0, 8, 0x4122, 2),
            (2, 1, 0x4120, 2),
            (0, 9, 0x4122, 2),
        ];
        const DMA: [u8; 16] = [
            0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40,
            0x60,
        ];
        if self.ram.read_u8(SPRITE_SUBTYPE2 + k) != 2 {
            let j = (self.ram.read_u8(SPRITE_D + k) * 4 + self.ram.read_u8(SPRITE_GRAPHICS + k) * 2)
                as usize;
            self.ram.write_u8(DMA_VAR6, DMA[j]);
            self.ram.write_u8(DMA_VAR7, DMA[j + 1]);
            self.runtime
                .draw_multiple_player_deferred(self.ram, k, &D1[j..j + 2]);
        } else {
            self.runtime.draw_multiple_player_deferred(self.ram, k, &D0);
        }
    }

    pub fn helmasaur_fireball_tri_split(&mut self, k: usize) {
        const X: [i8; 3] = [0, 28, -28];
        const Y: [i8; 3] = [-32, 24, 24];
        const D: [u8; 6] = [32, 80, 128, 32, 80, 128];
        self.runtime.sfx3_with_pan(self.ram, k, 0x36);
        self.ram.write_u8(SPRITE_STATE + k, 0);
        let r = self.runtime.random_number(self.ram);
        self.ram.write_u8(BYTE_7E0FB6, r);
        for i in (0..3).rev() {
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x70) {
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.ram.write_u8(SPRITE_X_VEL + j, X[i] as u8);
                self.ram.write_u8(SPRITE_Y_VEL + j, Y[i] as u8);
                self.ram.write_u8(SPRITE_AI_STATE + j, 3);
                self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 3);
                self.ram
                    .write_u8(SPRITE_DELAY_MAIN + j, D[((r & 3) as usize) + i]);
                self.ram.write_u8(SPRITE_HEAD_DIR + j, 0);
                self.ram.write_u8(SPRITE_GRAPHICS + j, 1);
            }
        }
        self.ram.write_u8(TMP_COUNTER, 255);
    }
    pub fn helmasaur_fireball_quad_split(&mut self, k: usize) {
        const X: [i8; 4] = [32, 32, -32, -32];
        const Y: [i8; 4] = [-32, 32, -32, 32];
        self.runtime.sfx3_with_pan(self.ram, k, 0x36);
        self.ram.write_u8(SPRITE_STATE + k, 0);
        for i in (0..4).rev() {
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x70) {
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.ram.write_u8(SPRITE_X_VEL + j, X[i] as u8);
                self.ram.write_u8(SPRITE_Y_VEL + j, Y[i] as u8);
                self.ram.write_u8(SPRITE_AI_STATE + j, 4);
                self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 4);
            }
        }
        self.ram.write_u8(TMP_COUNTER, 255);
    }
    pub fn thief_scan_for_booty(&mut self, k: usize) -> Option<usize> {
        for j in (0..16).rev() {
            let t = self.ram.read_u8(SPRITE_TYPE + j);
            if self.ram.read_u8(SPRITE_STATE + j) != 0 && (t == 0xdc || t == 0xe1 || t == 0xd9) {
                self.thief_target_booty(k, j);
                return Some(j);
            }
        }
        self.ram.write_u8(SPRITE_AI_STATE + k, 0);
        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 64);
        None
    }
    pub fn thief_target_booty(&mut self, k: usize, target: usize) {
        if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0 {
            let (x, y) = self.runtime.project_speed_towards_location(
                self.ram,
                k,
                self.sprite_x(target),
                self.sprite_y(target),
                19,
            );
            self.ram.write_u8(SPRITE_X_VEL + k, x);
            self.ram.write_u8(SPRITE_Y_VEL + k, y);
        }
        for j in (0..16).rev() {
            let t = self.ram.read_u8(SPRITE_TYPE + j);
            if (((j as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3
                | self.ram.read_u8(SPRITE_DELAY_AUX4 + j))
                == 0
                && self.ram.read_u8(SPRITE_STATE + j) != 0
                && (t == 0xdc || t == 0xe1 || t == 0xd9)
            {
                self.thief_grab_booty(k, j);
            }
        }
    }
    pub fn thief_grab_booty(&mut self, k: usize, j: usize) {
        if self
            .sprite_x(j)
            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
            .wrapping_add(8)
            < 16
            && self
                .sprite_y(j)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                .wrapping_add(12)
                < 24
        {
            self.ram.write_u8(SPRITE_STATE + j, 0);
            const S: [u8; 15] = [
                0xb, 0xa, 0xa, 0xa, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0x2f, 0x2f, 0xb,
            ];
            let t = self.ram.read_u8(SPRITE_TYPE + j).wrapping_sub(0xd8) as usize;
            self.runtime.sfx3_with_pan(self.ram, t, S[t]);
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 14);
        }
    }
    pub fn thief_check_collision_with_link(&mut self, k: usize) {
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 32);
            self.ram.write_u8(LINK_ACTUAL_VEL_Y, y);
            self.ram.write_u8(SPRITE_Y_RECOIL + k, y ^ 255);
            self.ram.write_u8(LINK_ACTUAL_VEL_X, x);
            self.ram.write_u8(SPRITE_X_RECOIL + k, x ^ 255);
            self.ram.write_u8(LINK_INCAPACITATED_TIMER, 4);
            self.ram.write_u8(SPRITE_F + k, 12);
            self.runtime.sfx2_with_pan(self.ram, k, 0xb);
        }
    }
    pub fn thief_spill_items(&mut self, k: usize) {
        const TY: [u8; 4] = [0xd9, 0xe1, 0xdc, 0xd9];
        const X: [i8; 6] = [0, 24, 24, 0, -24, -24];
        const Y: [i8; 6] = [-32, -16, 16, 32, 16, -16];
        self.ram.write_u8(TMP_COUNTER, 5);
        loop {
            let kind = self.runtime.random_number(self.ram) & 3;
            self.ram.write_u8(BYTE_7E0FB6, kind);
            let available = match kind {
                1 => self.ram.read_u8(LINK_NUM_ARROWS) as u16,
                2 => self.ram.read_u8(LINK_ITEM_BOMBS) as u16,
                _ => self.ram.read_u16(LINK_RUPEES_GOAL),
            };
            if available == 0 {
                return;
            }
            let Some(j) = self
                .runtime
                .spawn_dynamic_ex(self.ram, k, TY[kind as usize], 7)
            else {
                return;
            };
            match kind {
                1 => self
                    .ram
                    .write_u8(LINK_NUM_ARROWS, self.ram.read_u8(LINK_NUM_ARROWS) - 1),
                2 => self
                    .ram
                    .write_u8(LINK_ITEM_BOMBS, self.ram.read_u8(LINK_ITEM_BOMBS) - 1),
                _ => self
                    .ram
                    .write_u16(LINK_RUPEES_GOAL, self.ram.read_u16(LINK_RUPEES_GOAL) - 1),
            }
            self.set_sprite_x(j, self.ram.read_u16(LINK_X));
            self.set_sprite_y(j, self.ram.read_u16(LINK_Y));
            let i = self.ram.read_u8(TMP_COUNTER) as usize;
            self.ram.write_u8(SPRITE_Z_VEL + j, 24);
            self.ram.write_u8(SPRITE_X_VEL + j, X[i] as u8);
            self.ram.write_u8(SPRITE_Y_VEL + j, Y[i] as u8);
            self.ram.write_u8(SPRITE_DELAY_AUX4 + j, 32);
            self.ram.write_u8(SPRITE_HEAD_DIR + j, 1);
            self.ram.write_u8(SPRITE_STUNNED + j, 255);
            let t = (i as u8).wrapping_sub(1);
            self.ram.write_u8(TMP_COUNTER, t);
            if (t as i8) < 0 {
                break;
            }
        }
    }

    pub fn sprite_draw_antfairy(&mut self, k: usize) {
        const D: [DrawPiece; 30] = [
            (4, 2, 0x02e1, 0),
            (4, -3, 0x02e3, 0),
            (-1, 2, 0x02e3, 0),
            (9, 2, 0x02e3, 0),
            (4, 7, 0x02e3, 0),
            (4, 2, 0x02e1, 0),
            (3, -3, 0x02e3, 0),
            (9, 1, 0x02e3, 0),
            (-1, 3, 0x02e3, 0),
            (5, 7, 0x02e3, 0),
            (4, 2, 0x02e1, 0),
            (1, -3, 0x02e3, 0),
            (9, -1, 0x02e3, 0),
            (-1, 5, 0x02e3, 0),
            (7, 7, 0x02e3, 0),
            (4, 2, 0x02e1, 0),
            (0, -2, 0x02e3, 0),
            (8, -2, 0x02e3, 0),
            (0, 6, 0x02e3, 0),
            (8, 6, 0x02e3, 0),
            (4, 2, 0x02e1, 0),
            (7, -3, 0x02e3, 0),
            (-1, -1, 0x02e3, 0),
            (9, 5, 0x02e3, 0),
            (1, 7, 0x02e3, 0),
            (4, 2, 0x02e1, 0),
            (5, -3, 0x02e3, 0),
            (-1, 1, 0x02e3, 0),
            (9, 3, 0x02e3, 0),
            (3, 7, 0x02e3, 0),
        ];
        let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
        self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
        if s & 1 | self.ram.read_u8(SUBMODULE_INDEX) | self.ram.read_u8(FLAG_UNK1) == 0 {
            let g = self.ram.read_u8(SPRITE_GRAPHICS + k).wrapping_add(1);
            self.ram
                .write_u8(SPRITE_GRAPHICS + k, if g == 6 { 0 } else { g });
        }
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 5;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 5]);
    }

    pub fn sprite_mad_batter_bolt(&mut self, k: usize) {
        const X: [i8; 8] = [0, 4, 8, 12, 12, 4, 8, 0];
        const Y: [i8; 8] = [0, 4, 8, 12, 12, 4, 8, 0];
        if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 16 != 0 {
            self.runtime.oam_allocate_region_b(self.ram, 4);
        }
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 0 {
            self.runtime.move_xy(self.ram, k);
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                self.ram.write_u8(SPRITE_AI_STATE + k, 1);
            }
        } else {
            let a = self.ram.read_u8(SPRITE_AI_STATE + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_AI_STATE + k, a);
            if a == 0 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
            if s & 7 == 0 {
                self.ram.write_u8(SOUND_EFFECT_2, 48);
            }
            self.set_sprite_x(
                k,
                self.ram
                    .read_u16(LINK_X)
                    .wrapping_add_signed(X[(s >> 2 & 7) as usize] as i16),
            );
            self.set_sprite_y(
                k,
                self.ram
                    .read_u16(LINK_Y)
                    .wrapping_add_signed(Y[(s >> 4 & 7) as usize] as i16),
            );
        }
    }

    pub fn sprite_draw_king_helmasaur_body(&mut self, k: usize, _info: &PrepOamCoordsRet) {
        const D: [DrawPiece; 19] = [
            (-24, -32, 0x0b80, 2),
            (-8, -32, 0x0b82, 2),
            (8, -32, 0x4b82, 2),
            (24, -32, 0x4b80, 2),
            (-24, -16, 0x0b84, 2),
            (-8, -16, 0x0b86, 2),
            (8, -16, 0x4b86, 2),
            (24, -16, 0x4b84, 2),
            (-24, 0, 0x0b88, 2),
            (-8, 0, 0x0b8a, 2),
            (8, 0, 0x4b8a, 2),
            (24, 0, 0x4b88, 2),
            (-24, 16, 0x0b8c, 2),
            (-8, 16, 0x0b8e, 2),
            (8, 16, 0x4b8e, 2),
            (24, 16, 0x4b8c, 2),
            (-8, 32, 0x0ba0, 2),
            (8, 32, 0x4ba0, 2),
            (0, -40, 0x0bac, 2),
        ];
        self.runtime.draw_multiple(self.ram, k, &D);
    }
    pub fn sprite_draw_king_helmasaur_mouth(&mut self, k: usize, info: &PrepOamCoordsRet) {
        const Y: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9,
            8, 7, 6, 5, 4, 3, 2, 1,
        ];
        let d = self.ram.read_u8(SPRITE_DELAY_AUX2 + k);
        if d == 0 {
            return;
        }
        self.runtime.oam_allocate_region_b(self.ram, 4);
        self.runtime.emit_oam(
            self.ram,
            k,
            info.x as u8,
            info.y
                .wrapping_add(Y[(d >> 2) as usize] as u16)
                .wrapping_add(0x13) as u8,
            0xaa,
            info.flags ^ 0xb,
            2,
        );
    }

    pub fn sprite_spawn_bomb(&mut self, k: usize) -> Option<usize> {
        let j = self.runtime.spawn_dynamic(self.ram, k, 0x4a)?;
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.runtime.transmute_to_bomb(self.ram, j);
        self.ram.write_u8(SPRITE_DELAY_AUX1 + j, 80);
        self.ram.write_u8(SPRITE_X_VEL + j, 24);
        self.ram.write_u8(SPRITE_Z_VEL + j, 48);
        Some(j)
    }
    pub fn catfish_regurgitate_medallion(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc0) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(SPRITE_X_VEL + j, 24);
            self.ram.write_u8(SPRITE_Z_VEL + j, 48);
            self.ram.write_u8(SPRITE_A + j, 17);
            self.runtime.sfx2_with_pan(self.ram, j, 0x20);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 0x83);
            self.ram.write_u8(SPRITE_FLAGS3 + j, 0x58);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, 8);
            self.runtime.decode_animated_sprite_tile(self.ram, 0x1c);
        }
    }
    pub fn sprite_zora_regurgitate_flippers(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc0) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(SPRITE_Z_VEL + j, 32);
            self.ram.write_u8(SPRITE_Y_VEL + j, 16);
            self.ram.write_u8(SPRITE_A + j, 30);
            self.runtime.sfx2_with_pan(self.ram, j, 0x20);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 0x83);
            self.ram.write_u8(SPRITE_FLAGS3 + j, 0x54);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, 4);
            self.ram.write_u8(SPRITE_DELAY_AUX3 + j, 0x30);
            self.runtime.decode_animated_sprite_tile(self.ram, 0x11);
        }
    }
    pub fn catfish_spawn_plop(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xec) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(SPRITE_STATE + j, 3);
            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 15);
            self.ram.write_u8(SPRITE_AI_STATE + j, 0);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 3);
            self.runtime.sfx2_with_pan(self.ram, k, 0x28);
        }
    }
    pub fn sprite_spawn_water_splash(&mut self, k: usize) -> Option<usize> {
        let j = self.runtime.spawn_dynamic(self.ram, k, 0xc0)?;
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.ram.write_u8(SPRITE_A + j, 0x80);
        self.ram.write_u8(SPRITE_FLAGS2 + j, 2);
        self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 2);
        self.ram.write_u8(SPRITE_OAM_FLAGS + j, 4);
        self.ram.write_u8(SPRITE_DELAY_MAIN + j, 31);
        Some(j)
    }
    pub fn sprite_catfish_splash_of_water(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (-8, -4, 0x0080, 0),
            (18, -7, 0x0080, 0),
            (-5, -2, 0x00bf, 0),
            (15, -4, 0x40af, 0),
            (0, -4, 0x00e7, 2),
            (0, -4, 0x00e7, 2),
            (0, -4, 0x00c0, 2),
            (0, -4, 0x00c0, 2),
        ];
        let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
        if d == 0 {
            self.ram.write_u8(SPRITE_STATE + k, 0);
        }
        let i = (d >> 3) as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
    }

    pub fn sprite_c8_big_fairy(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_HEAD_DIR + k) != 0 {
            self.sprite_fairy_cloud(k);
        } else {
            self.sprite_big_fairy(k);
        }
    }
    pub fn big_faerie_draw(&mut self, k: usize) {
        const D: [DrawPiece; 16] = [
            (-4, -8, 0x008e, 2),
            (4, -8, 0x408e, 2),
            (-4, 8, 0x00ae, 2),
            (4, 8, 0x40ae, 2),
            (-4, -8, 0x008c, 2),
            (4, -8, 0x408c, 2),
            (-4, 8, 0x00ac, 2),
            (4, 8, 0x40ac, 2),
            (-4, -8, 0x008a, 2),
            (4, -8, 0x408a, 2),
            (-4, 8, 0x00aa, 2),
            (4, 8, 0x40aa, 2),
            (-4, -8, 0x008c, 2),
            (4, -8, 0x408c, 2),
            (-4, 8, 0x00ac, 2),
            (4, 8, 0x40ac, 2),
        ];
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        self.runtime.draw_shadow(self.ram, k);
    }
    pub fn faerie_cloud_draw(&mut self, k: usize) {
        const XY: [i8; 8] = [-12, -6, 0, 6, 12, 18, 0, 6];
        let a = self.ram.read_u8(SPRITE_A + k);
        if (a as i8) >= 0 && a & self.ram.read_u8(SPRITE_SUBTYPE2 + k) == 0 {
            let x = XY[(self.runtime.random_number(self.ram) & 7) as usize];
            let y = XY[(self.runtime.random_number(self.ram) & 7) as usize];
            self.runtime.garnish_spawn_sparkle(self.ram, k, x, y);
        }
    }
    pub fn sprite_fairy_cloud(&mut self, k: usize) {
        self.runtime.prep_oam(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.write_u8(
            SPRITE_SUBTYPE2 + k,
            self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1),
        );
        self.faerie_cloud_draw(k);
        if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 31 == 0 {
            self.runtime.sfx2_with_pan(self.ram, k, 0x31);
        }
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                self.ram.write_u8(SPRITE_A + k, 0);
                self.runtime.apply_speed_towards_link(self.ram, k, 8);
                self.runtime.move_xy(self.ram, k);
                self.runtime.get_16bit_coords(self.ram, k);
                if self
                    .ram
                    .read_u16(LINK_X)
                    .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                    .wrapping_add(3)
                    < 6
                    && self
                        .ram
                        .read_u16(LINK_Y)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                        .wrapping_add(11)
                        < 6
                {
                    self.ram.write_u16(
                        LINK_HEARTS_FILLER,
                        self.ram.read_u16(LINK_HEARTS_FILLER).wrapping_add(0xa0),
                    );
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                }
            }
            1 => {
                if self.ram.read_u8(LINK_HEALTH_CURRENT) == self.ram.read_u8(LINK_HEALTH_CAPACITY) {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    self.ram.write_u8(SPRITE_DELAY_AUX2, 112);
                }
            }
            2 => {
                let a = self.ram.read_u8(SPRITE_A + k);
                if self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 15 == 0 && (a as i8) >= 0 {
                    let a = a.wrapping_mul(2).wrapping_add(1);
                    self.ram.write_u8(SPRITE_A + k, a);
                    if a >= 0x80 {
                        self.ram.write_u8(SPRITE_A + k, 255);
                        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                        self.ram.write_u8(SPRITE_STATE + k, 0);
                    }
                }
            }
            _ => {}
        }
    }
    pub fn sprite_big_fairy(&mut self, k: usize) {
        let mut i = self.ram.read_u8(SPRITE_DELAY_AUX2 + k);
        if i != 0 && i < 0x40 {
            i -= 1;
            if i == 0 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            if i & 1 != 0 {
                return;
            }
        }
        self.big_faerie_draw(k);
        let g = self.ram.read_u8(SPRITE_G + k).wrapping_sub(1);
        self.ram.write_u8(SPRITE_G + k, g);
        if (g as i8) < 0 {
            self.ram.write_u8(SPRITE_G + k, 5);
            self.ram.write_u8(
                SPRITE_GRAPHICS + k,
                self.ram.read_u8(SPRITE_GRAPHICS + k).wrapping_add(1) & 3,
            );
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.write_u8(
            SPRITE_SUBTYPE2 + k,
            self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1),
        );
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 0 {
            self.faerie_cloud_draw(k);
            self.ram.write_u8(SPRITE_A + k, 1);
            let (dx, dy) = self
                .runtime
                .direction_to_face_link_with_differences(self.ram, k);
            if dx.wrapping_add(0x30) < 0x60 && dy.wrapping_add(0x30) < 0x60 {
                self.runtime.link_cancel_dash(self.ram);
                self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x15a);
                self.runtime.show_message_minimal(self.ram);
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc8) {
                    self.runtime.set_spawned_coordinates(self.ram, j);
                    self.ram.write_u8(SPRITE_HEAD_DIR + j, 1);
                    self.ram.write_u8(
                        SPRITE_Y_LO + j,
                        self.ram
                            .read_u8(SPRITE_Y_LO + j)
                            .wrapping_sub(self.ram.read_u8(SPRITE_Z + k)),
                    );
                    self.ram.write_u8(SPRITE_Z + j, 0);
                }
            }
        }
    }

    pub fn talking_tree_spawn_bomb(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x4a) {
            self.runtime.transmute_to_bomb(self.ram, j);
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(SPRITE_DELAY_AUX1 + j, 64);
            self.ram.write_u8(SPRITE_Y_VEL + j, 24);
            self.ram.write_u8(SPRITE_Z_VEL + j, 18);
        }
    }
    pub fn talking_tree_draw(&mut self, k: usize) {
        const D: [DrawPiece; 12] = [
            (1, -1, 0x00e8, 0),
            (1, 7, 0x00f8, 0),
            (7, -1, 0x40e8, 0),
            (7, 7, 0x40f8, 0),
            (0, -1, 0x00e8, 0),
            (0, 7, 0x00f8, 0),
            (8, -1, 0x40e8, 0),
            (8, 7, 0x40f8, 0),
            (0, 0, 0x00e8, 0),
            (0, 7, 0x00f8, 0),
            (8, 0, 0x40e8, 0),
            (8, 7, 0x40f8, 0),
        ];
        let g = self.ram.read_u8(SPRITE_GRAPHICS + k).wrapping_sub(1);
        if (g as i8) < 0 {
            return;
        }
        let i = g as usize * 4;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &D[i..i + 4]);
    }
    pub fn talking_tree_eye(&mut self, k: usize) {
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let j = self.ram.read_u8(SPRITE_HEAD_DIR + k) as usize;
        let bx = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_A + k),
            self.ram.read_u8(SPRITE_B + k),
        ]);
        let by = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_C + k),
            self.ram.read_u8(SPRITE_E + k),
        ]);
        self.set_sprite_x(k, bx.wrapping_add_signed([9, -9][j]));
        self.set_sprite_y(k, by);
        let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 2);
        if (y as i8) >= 0 {
            self.ram.write_u8(SPRITE_D + k, x.wrapping_add(2));
        } else if self.ram.read_u8(SPRITE_D + k) != 2 {
            let d = self.ram.read_u8(SPRITE_D + k);
            self.ram
                .write_u8(SPRITE_D + k, d.wrapping_add(if d >= 2 { 255 } else { 1 }));
        }
        let j = self.ram.read_u8(SPRITE_D + k) as usize;
        self.set_sprite_x(k, bx.wrapping_add_signed([-2, -1, 0, 1, 2][j]));
        self.set_sprite_y(k, by.wrapping_add_signed([-1, 0, 0, 0, -1][j]));
    }
    pub fn sprite_prep_talking_tree_spawn_eyeball(&mut self, k: usize, dir: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x25) {
            let x = self.sprite_x(k).wrapping_add_signed([-4, 14][dir]);
            let y = self.sprite_y(k).wrapping_sub(11);
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            let [a, b] = x.to_le_bytes();
            let [c, e] = y.to_le_bytes();
            self.ram.write_u8(SPRITE_HEAD_DIR + j, dir as u8);
            self.ram.write_u8(SPRITE_A + j, a);
            self.ram.write_u8(SPRITE_B + j, b);
            self.ram.write_u8(SPRITE_C + j, c);
            self.ram.write_u8(SPRITE_E + j, e);
            self.ram.write_u8(SPRITE_SUBTYPE2 + j, 1);
        }
    }
    pub fn rupee_pull_spawn_prize(&mut self, k: usize) {
        const X: [i8; 4] = [-18, -12, 12, 18];
        const Y: [i8; 4] = [16, 24, 24, 16];
        const TY: [u8; 3] = [0xd9, 0xda, 0xdb];
        let killed = self.ram.read_u8(NUM_SPRITES_KILLED);
        if killed != 0 {
            let kind = if killed < 4 {
                0
            } else if self.ram.read_u8(NUMBER_OF_TIMES_HURT_BY_SPRITES) != 0 {
                1
            } else {
                2
            };
            self.ram.write_u8(BYTE_7E0FB6, kind);
            self.ram.write_u8(TMP_COUNTER, 3);
            loop {
                let i = self.ram.read_u8(TMP_COUNTER) as usize;
                let Some(j) = self.runtime.spawn_dynamic(self.ram, k, TY[kind as usize]) else {
                    break;
                };
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.ram.write_u8(SPRITE_X_VEL + j, X[i] as u8);
                self.ram.write_u8(SPRITE_Y_VEL + j, Y[i] as u8);
                self.ram.write_u8(SPRITE_STUNNED + j, 255);
                self.ram.write_u8(SPRITE_DELAY_AUX4 + j, 32);
                self.ram.write_u8(SPRITE_DELAY_AUX3 + j, 32);
                self.ram.write_u8(SPRITE_Z_VEL + j, 32);
                let t = (i as u8).wrapping_sub(1);
                self.ram.write_u8(TMP_COUNTER, t);
                if (t as i8) < 0 {
                    break;
                }
            }
        }
        self.ram.write_u8(NUM_SPRITES_KILLED, 0);
        self.ram.write_u8(NUMBER_OF_TIMES_HURT_BY_SPRITES, 0);
    }
    pub fn digging_game_guy_draw(&mut self, k: usize) {
        const D: [DrawPiece; 9] = [
            (0, -8, 0x0a40, 2),
            (4, 9, 0x0c56, 0),
            (0, 0, 0x0a42, 2),
            (0, -8, 0x0a40, 2),
            (0, 0, 0x0a42, 2),
            (0, 0, 0x0a42, 2),
            (-1, -7, 0x0a40, 2),
            (-1, 0, 0x0a44, 2),
            (-1, 0, 0x0a44, 2),
        ];
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 3;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &D[i..i + 3]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_schedule_boss_for_death(&mut self, k: usize) {
        self.ram.write_u8(SPRITE_STATE + k, 4);
        self.ram.write_u8(SPRITE_A + k, 0);
        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 224);
    }
    pub fn sprite_make_boss_explosion(&mut self, k: usize) {
        self.runtime.sfx2_with_pan(self.ram, k, 0xc);
        self.sprite_make_boss_death_explosion_no_sound(k);
    }
    pub fn sprite_make_boss_death_explosion_no_sound(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0) {
            self.ram.write_u8(LOAD_CHR_HALFSLOT_EVEN_ODD, 11);
            self.ram.write_u8(SPRITE_STATE + j, 4);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 3);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, 12);
            self.set_sprite_x(j, self.ram.read_u16(CUR_SPRITE_X));
            self.set_sprite_y(j, self.ram.read_u16(CUR_SPRITE_Y));
            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 31);
            self.ram.write_u8(SPRITE_A + j, 31);
            self.ram.write_u8(SPRITE_FLOOR + j, 2);
        }
    }
    pub fn vulture_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (-8, 0, 0x0086, 2),
            (8, 0, 0x4086, 2),
            (-8, 0, 0x0080, 2),
            (8, 0, 0x4080, 2),
            (-8, 0, 0x0082, 2),
            (8, 0, 0x4082, 2),
            (-8, 0, 0x0084, 2),
            (8, 0, 0x4084, 2),
        ];
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_c2_boulder(&mut self, k: usize) {
        if self.ram.read_u8(PLAYER_IS_INDOORS) == 0 {
            self.boulder_outdoors_main(k);
            return;
        }
        if self.ram.read_u8(0xfc6) < 3 {
            self.runtime.draw_single_small(self.ram, k);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.write_u8(
            SPRITE_OAM_FLAGS + k,
            self.ram.read_u8(FRAME_COUNTER) << 2 & 0xc0,
        );
        self.runtime.move_xyz(self.ram, k);
        if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 != 0 {
            return;
        }
        if self
            .ram
            .read_u16(CUR_SPRITE_X)
            .wrapping_sub(self.ram.read_u16(LINK_X))
            .wrapping_add(4)
            < 16
            && self
                .ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_sub(self.ram.read_u16(LINK_Y))
                .wrapping_sub(4)
                < 12
        {
            self.runtime.attempt_damage_to_link_plus_recoil(self.ram, k);
        }
        if self.runtime.check_tile_collision(self.ram, k) {
            self.ram.write_u8(SPRITE_STATE + k, 0);
        }
    }
    pub fn boulder_outdoors_main(&mut self, k: usize) {
        const Z: [u8; 2] = [32, 48];
        const Y: [u8; 2] = [8, 32];
        const X: [u8; 4] = [24, 16, (-24i8) as u8, (-16i8) as u8];
        self.ram.write_u8(SPRITE_OBJ_PRIO + k, 0x30);
        self.boulder_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.write_u8(
            SPRITE_SUBTYPE2 + k,
            self.ram
                .read_u8(SPRITE_SUBTYPE2 + k)
                .wrapping_sub(self.ram.read_u8(SPRITE_D + k)),
        );
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.runtime.move_xyz(self.ram, k);
        self.ram.write_u8(
            SPRITE_Z_VEL + k,
            self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
        );
        if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
            self.ram.write_u8(SPRITE_Z + k, 0);
            let collision = usize::from(self.runtime.check_tile_collision(self.ram, k));
            self.ram.write_u8(SPRITE_Z_VEL + k, Z[collision]);
            self.ram.write_u8(SPRITE_Y_VEL + k, Y[collision]);
            let j = collision + ((self.runtime.random_number(self.ram) & 1) as usize) * 2;
            self.ram.write_u8(SPRITE_X_VEL + k, X[j]);
            self.ram
                .write_u8(SPRITE_D + k, ((j & 2) as u8).wrapping_sub(1));
            self.runtime.sfx2_with_pan(self.ram, k, 0xb);
        }
    }
    pub fn boulder_draw(&mut self, k: usize) {
        const D: [DrawPiece; 16] = [
            (-8, -8, 0x01cc, 2),
            (8, -8, 0x01ce, 2),
            (-8, 8, 0x01ec, 2),
            (8, 8, 0x01ee, 2),
            (-8, -8, 0x41ce, 2),
            (8, -8, 0x41cc, 2),
            (-8, 8, 0x41ee, 2),
            (8, 8, 0x41ec, 2),
            (-8, -8, 0xc1ee, 2),
            (8, -8, 0xc1ec, 2),
            (-8, 8, 0xc1ce, 2),
            (8, 8, 0xc1cc, 2),
            (-8, -8, 0x81ec, 2),
            (8, -8, 0x81ee, 2),
            (-8, 8, 0x81cc, 2),
            (8, 8, 0x81ce, 2),
        ];
        let i = (self.ram.read_u8(SPRITE_SUBTYPE2 + k) >> 3 & 3) as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        self.sprite_draw_large_shadow2(k);
    }
    pub fn sprite_draw_big_shadow(&mut self, k: usize, animation: u8) {
        self.ram.write_u16(
            CUR_SPRITE_Y,
            self.ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_add(self.ram.read_u8(SPRITE_Z + k) as u16),
        );
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(16));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(4),
        );
        self.runtime.draw_big_shadow(self.ram, k, animation);
        self.runtime.get_16bit_coords(self.ram, k);
    }
    pub fn sprite_draw_large_shadow2(&mut self, k: usize) {
        self.sprite_draw_big_shadow(k, (self.ram.read_u8(SPRITE_Z + k) >> 3).min(4));
    }
    pub fn cutscene_agahnim_spawn_zelda_on_altar(&mut self, k: usize) {
        self.ram.write_u8(
            SPRITE_X_LO + k,
            self.ram.read_u8(SPRITE_X_LO + k).wrapping_add(8),
        );
        self.ram.write_u8(
            SPRITE_Y_LO + k,
            self.ram.read_u8(SPRITE_Y_LO + k).wrapping_add(6),
        );
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc1) {
            self.ram.write_u8(SPRITE_A + j, 1);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 1);
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.write_u8(
                SPRITE_Y_LO + j,
                self.ram.read_u8(SPRITE_Y_LO + j).wrapping_add(40),
            );
            self.ram.write_u8(SPRITE_FLAGS2 + j, 0);
            self.ram.write_u8(SPRITE_OAM_FLAGS + j, 12);
        }
    }

    pub fn trident_draw(&mut self, k: usize) {
        const D: [DrawPiece; 50] = [
            (10, -10, 0x0864, 0),
            (5, -15, 0x0864, 0),
            (0, -20, 0x0864, 0),
            (-5, -25, 0x0864, 0),
            (-18, -38, 0x0844, 2),
            (1, -4, 0x0865, 0),
            (1, -11, 0x0865, 0),
            (1, -18, 0x0865, 0),
            (1, -25, 0x0865, 0),
            (-3, -40, 0x0862, 2),
            (-8, -9, 0x4864, 0),
            (-3, -14, 0x4864, 0),
            (3, -20, 0x4864, 0),
            (9, -26, 0x4864, 0),
            (12, -37, 0x4844, 2),
            (-10, -20, 0x4874, 0),
            (-3, -20, 0x4874, 0),
            (4, -20, 0x4874, 0),
            (11, -20, 0x4874, 0),
            (18, -23, 0x4860, 2),
            (-10, -30, 0xc864, 0),
            (-4, -24, 0xc864, 0),
            (2, -18, 0xc864, 0),
            (8, -12, 0xc864, 0),
            (12, -8, 0xc844, 2),
            (1, -32, 0x8865, 0),
            (1, -25, 0x8865, 0),
            (1, -18, 0x8865, 0),
            (1, -11, 0x8865, 0),
            (-3, -5, 0x8862, 2),
            (13, -30, 0x8864, 0),
            (8, -25, 0x8864, 0),
            (2, -19, 0x8864, 0),
            (-4, -13, 0x8864, 0),
            (-16, -9, 0x8844, 2),
            (14, -20, 0x0874, 0),
            (7, -20, 0x0874, 0),
            (0, -20, 0x0874, 0),
            (-7, -20, 0x0874, 0),
            (-21, -23, 0x0860, 2),
            (13, -30, 0x8864, 0),
            (8, -25, 0x8864, 0),
            (2, -19, 0x8864, 0),
            (-4, -13, 0x8864, 0),
            (-16, -9, 0x8844, 2),
            (-10, -30, 0xc864, 0),
            (-4, -24, 0xc864, 0),
            (-4, -24, 0xc864, 0),
            (-4, -24, 0xc864, 0),
            (-4, -24, 0xc864, 0),
        ];
        let g = self.ram.read_u8(SPRITE_G + k);
        if g == 0 {
            return;
        }
        const X: [i8; 5] = [24, -16, 0, 16, -8];
        const Y: [i8; 5] = [4, 4, 16, 21, 19];
        let j = if g == 9 {
            3
        } else if g >= 9 {
            4
        } else {
            self.ram.read_u8(SPRITE_D + k) as usize
        };
        self.ram.write_u16(
            CUR_SPRITE_X,
            self.ram
                .read_u16(CUR_SPRITE_X)
                .wrapping_add_signed(X[j] as i16),
        );
        self.ram.write_u16(
            CUR_SPRITE_Y,
            self.ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_add_signed(Y[j] as i16),
        );
        let bak = self.ram.read_u8(SPRITE_OBJ_PRIO + k);
        self.ram.write_u8(SPRITE_OBJ_PRIO + k, bak & !0xf);
        let i = (g as usize - 1) * 5;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 5]);
        self.ram.write_u8(SPRITE_OBJ_PRIO + k, bak);
        self.runtime.get_16bit_coords(self.ram, k);
    }

    pub fn sprite_c5_medusa(&mut self, k: usize) {
        self.runtime.prep_oam(self.ram, k);
        if self.ram.read_u8(PLAYER_IS_INDOORS) == 0 {
            self.ram.write_u8(SPRITE_X_VEL + k, 255);
            self.ram.write_u8(SPRITE_SUBTYPE + k, 255);
            if !self.runtime.check_tile_collision(self.ram, k)
                || self.runtime.return_if_inactive(self.ram, k)
            {
                return;
            }
            self.ram.write_u8(SPRITE_TYPE + k, 0x19);
            self.runtime.load_properties(self.ram, k);
            self.ram
                .write_u8(SPRITE_E + k, self.ram.read_u8(SPRITE_E + k).wrapping_add(1));
            self.ram.write_u8(
                SPRITE_X_LO + k,
                self.ram.read_u8(SPRITE_X_LO + k).wrapping_add(8),
            );
            self.ram.write_u8(
                SPRITE_Y_LO + k,
                self.ram.read_u8(SPRITE_Y_LO + k).wrapping_sub(8),
            );
            self.runtime.sfx3_with_pan(self.ram, k, 0x19);
            self.ram.write_u8(SPRITE_DEFL_BITS + k, 0x80);
        } else {
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
            if s & 0x7f == 0
                && self.ram.read_u8(SPRITE_FLOOR + k) == self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL)
                && let Some(j) = self.runtime.spawn_fireball(self.ram, k)
            {
                self.ram.write_u8(
                    SPRITE_DEFL_BITS + j,
                    self.ram.read_u8(SPRITE_DEFL_BITS + j) | 8,
                );
                self.ram.write_u8(SPRITE_BUMP_DAMAGE + j, 4);
            }
        }
    }

    pub fn sprite_c6_4_way_shooter(&mut self, k: usize) {
        const XO: [i8; 4] = [12, -12, 0, 0];
        const YO: [i8; 4] = [0, 0, 12, -12];
        const V: [i8; 6] = [0, 0, 40, -40, 0, 0];
        self.runtime.prep_oam(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let delay = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
        if delay == 24 {
            if let Some(j) = self.runtime.spawn_fireball(self.ram, k) {
                self.ram.write_u8(
                    SPRITE_DEFL_BITS + j,
                    self.ram.read_u8(SPRITE_DEFL_BITS + j) | 8,
                );
                self.ram.write_u8(SPRITE_BUMP_DAMAGE + j, 4);
                let i = self.runtime.direction_to_face_link(self.ram, j) as usize;
                self.ram.write_u8(SPRITE_X_VEL + j, V[i + 2] as u8);
                self.ram.write_u8(SPRITE_Y_VEL + j, V[i] as u8);
                self.set_sprite_x(j, self.sprite_x(j).wrapping_add_signed(XO[i] as i16));
                self.set_sprite_y(j, self.sprite_y(j).wrapping_add_signed(YO[i] as i16));
            }
        } else if delay == 0
            && self.ram.read_u8(BUTTON_B_FRAMES) != 0
            && self.ram.read_u8(SPRITE_FLOOR + k) == self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL)
        {
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
        }
    }

    pub fn thief_draw(&mut self, k: usize) {
        const D: [DrawPiece; 24] = [
            (0, -6, 0x0000, 2),
            (0, 0, 0x0006, 2),
            (0, -6, 0x0000, 2),
            (0, 0, 0x4006, 2),
            (0, -6, 0x0000, 2),
            (0, 0, 0x0020, 2),
            (0, -7, 0x0004, 2),
            (0, 0, 0x0022, 2),
            (0, -7, 0x0004, 2),
            (0, 0, 0x4022, 2),
            (0, -7, 0x0004, 2),
            (0, 0, 0x0024, 2),
            (0, -8, 0x0002, 2),
            (0, 0, 0x000a, 2),
            (0, -7, 0x0002, 2),
            (0, 0, 0x000e, 2),
            (0, -7, 0x0002, 2),
            (0, 0, 0x000a, 2),
            (0, -8, 0x4002, 2),
            (0, 0, 0x400a, 2),
            (0, -7, 0x4002, 2),
            (0, 0, 0x400e, 2),
            (0, -7, 0x4002, 2),
            (0, 0, 0x400a, 2),
        ];
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
        if self.ram.read_u8(SPRITE_PAUSE + k) == 0 {
            let j = self.ram.read_u8(SPRITE_HEAD_DIR + k) as usize;
            const CH: [u8; 4] = [2, 2, 0, 4];
            const FL: [u8; 4] = [0x40, 0, 0, 0];
            self.runtime
                .set_current_oam_char_flags(self.ram, CH[j], FL[j]);
            self.runtime.draw_shadow(self.ram, k);
        }
    }

    pub fn great_catfish_draw(&mut self, k: usize) {
        const D: [DrawPiece; 28] = [
            (-4, 4, 0x008c, 2),
            (4, 4, 0x008d, 2),
            (-4, 4, 0x008c, 2),
            (4, 4, 0x008d, 2),
            (-4, -4, 0x008c, 2),
            (4, -4, 0x008d, 2),
            (-4, 4, 0x009c, 2),
            (4, 4, 0x009d, 2),
            (-4, -4, 0x408d, 2),
            (4, -4, 0x408c, 2),
            (-4, 4, 0x409d, 2),
            (4, 4, 0x409c, 2),
            (-4, -4, 0xc09d, 2),
            (4, -4, 0xc09c, 2),
            (-4, 4, 0xc08d, 2),
            (4, 4, 0xc08c, 2),
            (-4, 4, 0xc09d, 2),
            (4, 4, 0xc09c, 2),
            (-4, 4, 0xc09d, 2),
            (4, 4, 0xc09c, 2),
            (0, 8, 0x00bd, 0),
            (8, 8, 0x40bd, 0),
            (8, 8, 0x40bd, 0),
            (8, 8, 0x40bd, 0),
            (-8, 0, 0x0086, 2),
            (8, 0, 0x4086, 2),
            (8, 0, 0x4086, 2),
            (8, 0, 0x4086, 2),
        ];
        let g = self.ram.read_u8(SPRITE_GRAPHICS + k);
        if g != 0 {
            let i = (g as usize - 1) * 4;
            self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        }
    }

    pub fn lightning_spawn_garnish(&mut self, k: usize) {
        let j = self.runtime.garnish_alloc_overwrite_old(self.ram);
        self.ram.write_u8(GARNISH_TYPE + j, 9);
        self.ram.write_u8(GARNISH_ACTIVE, 9);
        self.ram
            .write_u8(GARNISH_SPRITE + j, self.ram.read_u8(SPRITE_A + k));
        self.ram
            .write_u8(GARNISH_X_LO + j, self.ram.read_u8(SPRITE_X_LO + k));
        self.ram
            .write_u8(GARNISH_X_HI + j, self.ram.read_u8(SPRITE_X_HI + k));
        let y = self.sprite_y(k).wrapping_add(16);
        self.ram.write_u8(GARNISH_Y_LO + j, y as u8);
        self.ram.write_u8(GARNISH_Y_HI + j, (y >> 8) as u8);
        self.ram.write_u8(GARNISH_COUNTDOWN + j, 32);
    }

    pub fn sprite_bf_lightning(&mut self, k: usize) {
        const GFX: [u8; 8] = [0, 1, 2, 3, 0, 1, 2, 3];
        const FLAGS: [u8; 8] = [0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40];
        const XO: [i8; 64] = [
            -15, 0, 0, -15, 0, -15, -15, 0, -15, 0, 0, -15, 0, -15, -15, 0, 0, 15, 15, 0, 15, 0, 0,
            15, 0, 15, 15, 0, 15, 0, 0, 15, 0, 15, 15, 0, 15, 0, 0, 15, 0, 15, 15, 0, 15, 0, 0, 15,
            -15, 0, 0, -15, 0, -15, -15, 0, -15, 0, 0, -15, 0, -15, -15, 0,
        ];
        let j = self.ram.read_u8(SPRITE_A + k) as usize;
        let f = (self.ram.read_u8(SPRITE_OAM_FLAGS + k) & 0xb1)
            | FLAGS[j]
            | (self.ram.read_u8(FRAME_COUNTER) << 1 & 14);
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, f);
        self.ram.write_u8(
            SPRITE_GRAPHICS + k,
            GFX[j] + u8::from(self.ram.read_u8(DUNGEON_ROOM_INDEX2) == 0x20) * 4,
        );
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0
        {
            return;
        }
        self.lightning_spawn_garnish(k);
        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 2);
        self.set_sprite_y(k, self.sprite_y(k).wrapping_add(16));
        if self
            .ram
            .read_u8(SPRITE_Y_LO + k)
            .wrapping_sub(self.ram.read_u8(BG2VOFS_COPY2))
            >= 0xd0
        {
            self.ram.write_u8(SPRITE_STATE + k, 0);
            return;
        }
        let rr = self.runtime.random_number(self.ram) & 7;
        self.set_sprite_x(
            k,
            self.sprite_x(k)
                .wrapping_add_signed(XO[j * 8 + rr as usize] as i16),
        );
        self.ram.write_u8(SPRITE_A + k, rr);
    }

    pub fn sprite_armos_crusher(&mut self, k: usize) {
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 7);
        let shake = self.ram.read_u8(SPRITE_DELAY_AUX4 + k);
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
        match self.ram.read_u8(SPRITE_G + k) {
            0 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) | self.ram.read_u8(SPRITE_Z + k) == 0 {
                    self.runtime.apply_speed_towards_link(self.ram, k, 32);
                    self.ram.write_u8(SPRITE_Z_VEL + k, 32);
                    self.ram.write_u8(SPRITE_G + k, 1);
                    let [xl, xh] = self.ram.read_u16(LINK_X).to_le_bytes();
                    let [yl, yh] = self.ram.read_u16(LINK_Y).to_le_bytes();
                    self.ram.write_u8(SPRITE_B + k, xl);
                    self.ram.write_u8(SPRITE_C + k, xh);
                    self.ram.write_u8(SPRITE_E + k, yl);
                    self.ram.write_u8(SPRITE_HEAD_DIR + k, yh);
                    self.runtime.sfx2_with_pan(self.ram, k, 0x20);
                }
            }
            1 => {
                self.ram.write_u8(
                    SPRITE_Z_VEL + k,
                    self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_add(3),
                );
                let mut advance = self.runtime.check_tile_collision(self.ram, k);
                if !advance {
                    self.runtime.get_16bit_coords(self.ram, k);
                    let x = u16::from_le_bytes([
                        self.ram.read_u8(SPRITE_B + k),
                        self.ram.read_u8(SPRITE_C + k),
                    ]);
                    let y = u16::from_le_bytes([
                        self.ram.read_u8(SPRITE_E + k),
                        self.ram.read_u8(SPRITE_HEAD_DIR + k),
                    ]);
                    advance = x
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                        .wrapping_add(16)
                        < 32
                        && y.wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                            .wrapping_add(16)
                            < 32;
                }
                if advance {
                    self.ram.write_u8(SPRITE_G + k, 2);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 16);
                    self.ram.write_u8(SPRITE_X_VEL + k, 0);
                    self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                }
            }
            2 => {
                self.ram.write_u8(SPRITE_Z_VEL + k, 0);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_G + k, 3);
                }
            }
            3 => {
                self.ram.write_u8(SPRITE_Z_VEL + k, (-104i8) as u8);
                if (self.ram.read_u8(SPRITE_Z + k) as i8) >= 0 {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                    self.ram.write_u8(SPRITE_DELAY_AUX4 + k, 32);
                    self.ram.write_u8(SPRITE_G + k, 0);
                    self.runtime.sfx2_with_pan(self.ram, k, 0xc);
                }
            }
            _ => {}
        }
    }

    pub fn talking_tree_mouth(&mut self, k: usize) {
        const G2: [u8; 4] = [0, 2, 3, 1];
        self.talking_tree_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.write_u8(SPRITE_FLAGS4 + k, 0);
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                self.ram.write_u8(SPRITE_GRAPHICS + k, 0);
                if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
                    self.runtime.link_cancel_dash(self.ram);
                    self.ram.write_u8(LINK_INCAPACITATED_TIMER, 16);
                    let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 48);
                    self.ram.write_u8(LINK_ACTUAL_VEL_Y, y);
                    self.ram.write_u8(LINK_ACTUAL_VEL_X, x);
                    self.runtime.sfx3_with_pan(self.ram, k, 0x32);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
                }
            }
            1 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 8);
                }
                self.ram.write_u8(
                    SPRITE_GRAPHICS + k,
                    self.ram.read_u8(SPRITE_DELAY_MAIN + k) >> 1 & 3,
                );
            }
            2 => {
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                self.ram
                    .write_u8(SPRITE_GRAPHICS + k, G2[(d >> 1) as usize]);
                if d == 7 {
                    self.talking_tree_spawn_bomb(k);
                }
                if d == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                }
            }
            3 => {
                self.ram.write_u8(SPRITE_FLAGS4 + k, 7);
                let mut j;
                if self.ram.read_u8(SPRITE_A + k) == 0 {
                    const M: [u16; 2] = [0x82, 0x7d];
                    j = ((self.ram.read_u8(SPRITE_X_LO + k) >> 4) & 1) ^ 1;
                    self.ram.write_u8(SPRITE_A + k, j);
                    if self
                        .runtime
                        .show_solicited_message(self.ram, k, M[j as usize])
                        & 0x100
                        == 0
                    {
                        self.ram.write_u8(SPRITE_A + k, 0);
                    }
                } else {
                    const M: [u16; 4] = [0x7e, 0x7f, 0x80, 0x81];
                    const S: [u8; 4] = [0x58, 0x5d, 0x72, 0x6b];
                    j = S
                        .iter()
                        .position(|&v| v == self.ram.read_u8(OVERWORLD_SCREEN_INDEX))
                        .unwrap_or(4) as u8;
                    if (j as usize) < M.len() {
                        self.runtime
                            .show_message_unconditional(self.ram, M[j as usize]);
                    }
                    self.ram.write_u8(SPRITE_A + k, 0);
                }
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    const G: [u8; 8] = [1, 2, 3, 1, 3, 1, 2, 3];
                    const D: [u8; 8] = [13, 13, 13, 11, 11, 6, 16, 8];
                    j = self.ram.read_u8(SPRITE_B + k).wrapping_add(1) & 7;
                    self.ram.write_u8(SPRITE_B + k, j);
                    self.ram.write_u8(SPRITE_GRAPHICS + k, G[j as usize]);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, D[j as usize]);
                }
            }
            _ => {}
        }
    }

    pub fn sprite_c4_thief(&mut self, k: usize) {
        const GFX: [u8; 12] = [11, 8, 2, 5, 9, 6, 0, 3, 10, 7, 1, 4];
        self.thief_draw(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        let _ = self.runtime.check_damage_from_link(self.ram, k);
        if self.ram.read_u8(SPRITE_AI_STATE + k) != 3 {
            let j = self.runtime.direction_to_face_link(self.ram, k);
            self.ram.write_u8(SPRITE_HEAD_DIR + k, j);
            if (j ^ self.ram.read_u8(SPRITE_D + k)) == 1 {
                self.ram.write_u8(SPRITE_D + k, j);
            }
        }
        let mut common = false;
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                self.thief_check_collision_with_link(k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0
                    && self
                        .ram
                        .read_u16(LINK_X)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                        .wrapping_add(0x50)
                        < 0xa0
                    && self
                        .ram
                        .read_u16(LINK_Y)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                        .wrapping_add(0x50)
                        < 0xa0
                {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 16);
                }
                self.ram.write_u8(
                    SPRITE_GRAPHICS + k,
                    GFX[self.ram.read_u8(SPRITE_D + k) as usize],
                );
            }
            1 => {
                self.thief_check_collision_with_link(k);
                let d = self.runtime.direction_to_face_link(self.ram, k);
                self.ram.write_u8(SPRITE_D + k, d);
                self.ram.write_u8(SPRITE_HEAD_DIR + k, d);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                }
                common = true;
            }
            2 => {
                self.runtime.apply_speed_towards_link(self.ram, k, 18);
                if self.ram.read_u8(SPRITE_WALLCOLL + k) == 0 {
                    self.runtime.move_xy(self.ram, k);
                }
                let _ = self.runtime.check_tile_collision(self.ram, k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0
                    && (self
                        .ram
                        .read_u16(LINK_X)
                        .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                        .wrapping_add(0x50)
                        >= 0xa0
                        || self
                            .ram
                            .read_u16(LINK_Y)
                            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                            .wrapping_add(0x50)
                            >= 0xa0)
                {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
                }
                if self.runtime.check_damage_to_link(self.ram, k) {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                    self.thief_spill_items(k);
                    self.runtime.sfx2_with_pan(self.ram, k, 0xb);
                }
                common = true;
            }
            3 => {
                self.thief_check_collision_with_link(k);
                let target = self.thief_scan_for_booty(k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
                    self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
                    let d = self.ram.read_u8(SPRITE_D + k);
                    self.ram
                        .write_u8(SPRITE_GRAPHICS + k, GFX[(4 + d + (s & 4)) as usize]);
                    if self.ram.read_u8(SPRITE_WALLCOLL + k) == 0 {
                        self.runtime.move_xy(self.ram, k);
                    }
                    let _ = self.runtime.check_tile_collision(self.ram, k);
                    self.ram
                        .write_u8(SPRITE_D + k, self.ram.read_u8(SPRITE_HEAD_DIR + k));
                }
                if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0
                    && let Some(j) = target
                {
                    let d = self.runtime.direction_to_face_location(
                        self.ram,
                        k,
                        self.sprite_x(j),
                        self.sprite_y(j),
                    );
                    self.ram.write_u8(SPRITE_HEAD_DIR + k, d);
                }
            }
            _ => {}
        }
        if common {
            if self.ram.read_u8(FRAME_COUNTER) & 31 == 0 {
                self.ram
                    .write_u8(SPRITE_D + k, self.ram.read_u8(SPRITE_HEAD_DIR + k));
            }
            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
            let d = self.ram.read_u8(SPRITE_D + k);
            self.ram
                .write_u8(SPRITE_GRAPHICS + k, GFX[(4 + d + (s & 4)) as usize]);
        }
    }

    pub fn sprite_raven(&mut self, k: usize) {
        self.ram.write_u8(
            SPRITE_OBJ_PRIO + k,
            self.ram.read_u8(SPRITE_OBJ_PRIO + k) | 0x30,
        );
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.runtime.move_xy(self.ram, k);
        let state = self.ram.read_u8(SPRITE_AI_STATE + k);
        if state == 0 {
            let (right, _) = self.runtime.is_right_of_link(self.ram, k);
            self.ram.write_u8(
                SPRITE_OAM_FLAGS + k,
                (self.ram.read_u8(SPRITE_OAM_FLAGS + k) & !0x40) | (right * 0x40),
            );
            let x = self
                .ram
                .read_u16(LINK_X)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X));
            let y = self
                .ram
                .read_u16(LINK_Y)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y));
            if x.wrapping_add(0x50 + u16::from((x as i16) >= 0)) < 0xa0
                && y.wrapping_add(0x58 + u16::from((y as i16) >= 0)) < 0xa0
            {
                self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 24);
                self.runtime.sfx3_with_pan(self.ram, k, 0x1e);
            }
            return;
        }
        if state == 1 {
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                self.ram.write_u8(
                    SPRITE_DELAY_MAIN + k,
                    [16, 248][self.ram.read_u8(SPRITE_A + k) as usize],
                );
                self.runtime.apply_speed_towards_link(self.ram, k, 32);
            }
            self.ram
                .write_u8(SPRITE_Z + k, self.ram.read_u8(SPRITE_Z + k).wrapping_add(1));
            self.ram.write_u8(
                SPRITE_GRAPHICS + k,
                (self.ram.read_u8(FRAME_COUNTER) >> 1 & 1) + 1,
            );
            return;
        }
        if state == 2
            && self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0
            && !(self.ram.read_u8(IS_IN_DARK_WORLD) != 0 && self.ram.read_u8(SPRITE_A + k) != 0)
        {
            self.ram.write_u8(SPRITE_AI_STATE + k, 3);
        }
        let fleeing = state == 3;
        if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 1 == 0 {
            let (mut xv, mut yv) =
                self.runtime
                    .project_speed_towards_link(self.ram, k, if fleeing { 48 } else { 32 });
            if fleeing {
                xv = 0u8.wrapping_sub(xv);
                yv = 0u8.wrapping_sub(yv);
            }
            let ox = self.ram.read_u8(SPRITE_X_VEL + k);
            if ox != xv {
                self.ram.write_u8(
                    SPRITE_X_VEL + k,
                    ox.wrapping_add(if (ox.wrapping_sub(xv) as i8) < 0 {
                        1
                    } else {
                        255
                    }),
                );
            }
            let oy = self.ram.read_u8(SPRITE_Y_VEL + k);
            if oy != yv {
                self.ram.write_u8(
                    SPRITE_Y_VEL + k,
                    oy.wrapping_add(if (oy.wrapping_sub(yv) as i8) < 0 {
                        1
                    } else {
                        255
                    }),
                );
            }
        }
        self.ram.write_u8(
            SPRITE_GRAPHICS + k,
            (self.ram.read_u8(FRAME_COUNTER) >> 1 & 1) + 1,
        );
        let flip = (self.ram.read_u8(SPRITE_X_VEL + k) >> 7) & 1;
        self.ram.write_u8(
            SPRITE_OAM_FLAGS + k,
            (self.ram.read_u8(SPRITE_OAM_FLAGS + k) & !0x40) | (flip * 0x40),
        );
    }

    pub fn sprite_evil_barrier(&mut self, k: usize) {
        self.evil_barrier_draw(k);
        if self.ram.read_u8(SPRITE_GRAPHICS + k) == 4 {
            return;
        }
        self.ram.write_u8(
            SPRITE_GRAPHICS + k,
            self.ram.read_u8(FRAME_COUNTER) >> 1 & 3,
        );
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.runtime.check_damage_from_link(self.ram, k) && self.ram.read_u8(LINK_SWORD_TYPE) < 2
        {
            self.ram.write_u8(SPRITE_HIT_TIMER + k, 0);
            self.runtime.attempt_damage_to_link_plus_recoil(self.ram, k);
            if self.ram.read_u8(COUNTDOWN_FOR_BLINK) == 0 {
                self.ram.write_u8(LINK_ELECTROCUTE_ON_TOUCH, 64);
            }
        }
        if self
            .ram
            .read_u16(LINK_Y)
            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
            .wrapping_add(8)
            < 24
            && self
                .ram
                .read_u16(LINK_X)
                .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                .wrapping_add(32)
                < 64
            && (self.ram.read_u8(LINK_ACTUAL_VEL_Y).wrapping_sub(1) as i8) < 0
        {
            self.ram.write_u8(LINK_ELECTROCUTE_ON_TOUCH, 64);
            self.ram.write_u8(LINK_INCAPACITATED_TIMER, 12);
            self.ram.write_u8(LINK_AUXILIARY_STATE, 1);
            self.ram.write_u8(LINK_GIVE_DAMAGE, 2);
            self.ram.write_u8(LINK_ACTUAL_VEL_X, 0);
            self.ram.write_u8(LINK_ACTUAL_VEL_Y, 48);
        }
    }

    pub fn evil_barrier_draw(&mut self, k: usize) {
        const D: [DrawPiece; 45] = [
            (0, 0, 0x00e8, 2),
            (-29, 3, 0x00ca, 0),
            (-29, 11, 0x00da, 0),
            (37, 3, 0x40ca, 0),
            (37, 11, 0x40da, 0),
            (-24, -2, 0x00e6, 2),
            (-8, -2, 0x00e6, 2),
            (8, -2, 0x40e6, 2),
            (24, -2, 0x40e6, 2),
            (0, 0, 0x00cc, 2),
            (-29, 3, 0x00cb, 0),
            (-29, 11, 0x00db, 0),
            (37, 3, 0x40cb, 0),
            (37, 11, 0x40db, 0),
            (0, 0, 0x00cc, 2),
            (0, 0, 0x00cc, 2),
            (0, 0, 0x00cc, 2),
            (0, 0, 0x00cc, 2),
            (0, 0, 0x00cc, 2),
            (-29, 3, 0x00cb, 0),
            (-29, 11, 0x00db, 0),
            (37, 3, 0x40cb, 0),
            (37, 11, 0x40db, 0),
            (-24, -2, 0x80e6, 2),
            (-8, -2, 0x80e6, 2),
            (8, -2, 0xc0e6, 2),
            (24, -2, 0xc0e6, 2),
            (0, 0, 0x00e8, 2),
            (-29, 3, 0x00ca, 0),
            (-29, 11, 0x00da, 0),
            (37, 3, 0x40ca, 0),
            (37, 11, 0x40da, 0),
            (0, 0, 0x00e8, 2),
            (0, 0, 0x00e8, 2),
            (0, 0, 0x00e8, 2),
            (0, 0, 0x00e8, 2),
            (-29, 3, 0x00cb, 0),
            (-29, 11, 0x00db, 0),
            (37, 3, 0x40cb, 0),
            (37, 11, 0x40db, 0),
            (37, 11, 0x40db, 0),
            (37, 11, 0x40db, 0),
            (37, 11, 0x40db, 0),
            (37, 11, 0x40db, 0),
            (37, 11, 0x40db, 0),
        ];
        self.ram.write_u16(
            CUR_SPRITE_Y,
            self.ram.read_u16(CUR_SPRITE_Y).wrapping_add(8),
        );
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 9;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 9]);
        self.runtime.get_16bit_coords(self.ram, k);
    }

    pub fn sprite_c3_gibo(&mut self, k: usize) {
        const OF: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        if self.ram.read_u8(SPRITE_B + k) != 0 {
            self.runtime.draw_single_large(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.runtime.check_damage_to_and_from_link(self.ram, k);
            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
            self.ram.write_u8(
                SPRITE_OAM_FLAGS + k,
                (self.ram.read_u8(SPRITE_OAM_FLAGS + k) & 0x3f) | OF[(s >> 2 & 3) as usize],
            );
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
                self.runtime.move_xy(self.ram, k);
                self.runtime.bounce_from_tile_collision(self.ram, k);
            }
            return;
        }
        self.gibo_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.write_u8(
            SPRITE_ANIM_CLOCK + k,
            self.ram.read_u8(SPRITE_ANIM_CLOCK + k).wrapping_add(1),
        );
        let mut j = self.ram.read_u8(SPRITE_HEAD_DIR + k) as usize;
        if self.ram.read_u8(SPRITE_STATE + j) == 6 {
            self.ram
                .write_u8(SPRITE_STATE + k, self.ram.read_u8(SPRITE_STATE + j));
            self.ram.write_u8(
                SPRITE_DELAY_MAIN + k,
                self.ram.read_u8(SPRITE_DELAY_MAIN + j),
            );
            self.ram.write_u8(
                SPRITE_FLAGS2 + k,
                self.ram.read_u8(SPRITE_FLAGS2 + k).wrapping_add(4),
            );
            return;
        }
        self.ram.write_u8(
            SPRITE_SUBTYPE2 + k,
            self.ram.read_u8(FRAME_COUNTER) >> 3 & 3,
        );
        if self.ram.read_u8(FRAME_COUNTER) & 63 == 0 {
            let (r, _) = self.runtime.is_right_of_link(self.ram, k);
            self.ram.write_u8(SPRITE_D + k, r << 2);
        }
        let _ = self.runtime.check_damage_to_link(self.ram, k);
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                let delay = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if delay == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
                    self.ram
                        .write_u8(SPRITE_A + k, self.ram.read_u8(SPRITE_A + k).wrapping_add(1));
                    if let Some(n) = self.runtime.spawn_dynamic(self.ram, k, 0xc3) {
                        const XV: [i8; 8] = [16, 16, 0, -16, -16, -16, 0, 16];
                        const YV: [i8; 8] = [0, 0, 16, -16, 16, 16, -16, -16];
                        j = n;
                        self.runtime.set_spawned_coordinates(self.ram, j);
                        self.ram.write_u8(SPRITE_HEAD_DIR + k, j as u8);
                        self.ram.write_u8(SPRITE_FLAGS2 + j, 1);
                        self.ram.write_u8(SPRITE_B + j, 1);
                        self.ram.write_u8(SPRITE_FLAGS3 + j, 16);
                        self.ram
                            .write_u8(SPRITE_HEALTH + j, self.ram.read_u8(SPRITE_G + k));
                        self.ram.write_u8(SPRITE_OAM_FLAGS + j, 7);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + j, 48);
                        let c = self.ram.read_u8(SPRITE_C + k).wrapping_add(1);
                        self.ram.write_u8(SPRITE_C + k, c);
                        let i = if c == 3 {
                            self.ram.write_u8(SPRITE_C + k, 0);
                            self.runtime.direction_to_face_link(self.ram, k) as usize
                        } else {
                            (self.runtime.random_number(self.ram) & 7) as usize
                        };
                        self.ram.write_u8(SPRITE_X_VEL + j, XV[i] as u8);
                        self.ram.write_u8(SPRITE_Y_VEL + j, YV[i] as u8);
                    }
                } else if delay == 32 {
                    self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 32);
                }
            }
            1 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                }
            }
            2 => {
                if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0 {
                    let x = self.sprite_x(j);
                    let y = self.sprite_y(j);
                    if self
                        .ram
                        .read_u16(CUR_SPRITE_X)
                        .wrapping_sub(x)
                        .wrapping_add(2)
                        < 4
                        && self
                            .ram
                            .read_u16(CUR_SPRITE_Y)
                            .wrapping_sub(y)
                            .wrapping_add(2)
                            < 4
                    {
                        self.ram.write_u8(SPRITE_STATE + j, 0);
                        self.ram.write_u8(SPRITE_A + k, 0);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                        self.ram
                            .write_u8(SPRITE_G + k, self.ram.read_u8(SPRITE_HEALTH + j));
                        let d = (self.runtime.random_number(self.ram) & 31) + 32;
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, d);
                        return;
                    }
                    let (xv, yv) = self
                        .runtime
                        .project_speed_towards_location(self.ram, k, x, y, 16);
                    self.ram.write_u8(SPRITE_X_VEL + k, xv);
                    self.ram.write_u8(SPRITE_Y_VEL + k, yv);
                }
                self.runtime.move_xy(self.ram, k);
            }
            _ => {}
        }
    }

    pub fn gibo_draw(&mut self, k: usize) {
        const OF: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        const OF2: [u8; 2] = [11, 7];
        const D: [DrawPiece; 32] = [
            (4, -4, 0x408a, 2),
            (-4, -4, 0x408f, 0),
            (12, 12, 0x408e, 0),
            (-4, 4, 0x408c, 2),
            (4, -4, 0x40aa, 2),
            (-4, -4, 0x409f, 0),
            (12, 12, 0x409e, 0),
            (-4, 4, 0x40ac, 2),
            (3, -3, 0x40aa, 2),
            (-3, -3, 0x409f, 0),
            (11, 11, 0x409e, 0),
            (-3, 3, 0x40ac, 2),
            (3, -3, 0x408a, 2),
            (-3, -3, 0x408f, 0),
            (11, 11, 0x408e, 0),
            (-3, 3, 0x408c, 2),
            (-3, -4, 0x008a, 2),
            (13, -4, 0x008f, 0),
            (-3, 12, 0x008e, 0),
            (5, 4, 0x008c, 2),
            (-3, -4, 0x00aa, 2),
            (13, -4, 0x009f, 0),
            (-3, 12, 0x009e, 0),
            (5, 4, 0x00ac, 2),
            (-2, -3, 0x00aa, 2),
            (12, -3, 0x009f, 0),
            (-2, 11, 0x009e, 0),
            (4, 3, 0x00ac, 2),
            (-2, -3, 0x008a, 2),
            (12, -3, 0x008f, 0),
            (-2, 11, 0x008e, 0),
            (4, 3, 0x008c, 2),
        ];
        if self.ram.read_u8(SPRITE_A + k) == 0 {
            let b0 = self.ram.read_u8(SPRITE_FLAGS2 + k);
            let b1 = self.ram.read_u8(SPRITE_OAM_FLAGS + k);
            self.ram.write_u8(SPRITE_FLAGS2 + k, 1);
            self.ram.write_u8(
                SPRITE_OAM_FLAGS + k,
                OF[(self.ram.read_u8(SPRITE_ANIM_CLOCK + k) >> 2 & 3) as usize]
                    | OF2[(self.ram.read_u8(SPRITE_DELAY_AUX1 + k) >> 2 & 1) as usize],
            );
            self.runtime.draw_single_large(self.ram, k);
            self.ram.write_u8(SPRITE_OAM_FLAGS + k, b1);
            self.ram.write_u8(SPRITE_FLAGS2 + k, b0);
        }
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(8));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(2),
        );
        let i = (self
            .ram
            .read_u8(SPRITE_SUBTYPE2 + k)
            .wrapping_add(self.ram.read_u8(SPRITE_D + k))) as usize
            * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
    }

    pub fn sprite_c7_pokey(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_C + k) != 0 {
            self.runtime.draw_single_large(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.runtime.check_damage_to_and_from_link(self.ram, k);
            self.runtime.move_xyz(self.ram, k);
            self.ram.write_u8(
                SPRITE_Z_VEL + k,
                self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
            );
            if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
                self.ram.write_u8(SPRITE_Z_VEL + k, 16);
                self.ram.write_u8(SPRITE_Z + k, 0);
            }
            if self.runtime.bounce_from_tile_collision(self.ram, k) {
                self.runtime.sfx2_with_pan(self.ram, k, 0x21);
            }
            if self.ram.read_u8(SPRITE_G + k) >= 3 {
                self.ram.write_u8(SPRITE_STATE + k, 6);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 10);
                self.ram.write_u8(SPRITE_FLAGS5 + k, 0);
                self.runtime.sfx2_with_pan(self.ram, k, 0x1e);
            }
            return;
        }
        self.hokbok_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.read_u8(SPRITE_A + k) != 0 && self.ram.read_u8(SPRITE_F + k) == 15 {
            self.ram.write_u8(SPRITE_F + k, 6);
            self.ram.write_u8(
                SPRITE_Z + k,
                self.ram
                    .read_u8(SPRITE_Z + k)
                    .wrapping_add(self.ram.read_u8(SPRITE_B + k)),
            );
            let a = self.ram.read_u8(SPRITE_A + k).wrapping_sub(1);
            self.ram.write_u8(SPRITE_A + k, a);
            if a == 0 {
                self.ram.write_u8(SPRITE_HEALTH + k, 17);
            }
            let xv = self.ram.read_u8(SPRITE_X_VEL + k);
            let yv = self.ram.read_u8(SPRITE_Y_VEL + k);
            self.ram.write_u8(
                SPRITE_X_VEL + k,
                xv.wrapping_add(if (xv as i8) < 0 { 252 } else { 4 }),
            );
            self.ram.write_u8(
                SPRITE_Y_VEL + k,
                yv.wrapping_add(if (yv as i8) < 0 { 252 } else { 4 }),
            );
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc7) {
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.ram.write_u8(SPRITE_C + j, 1);
                self.ram.write_u8(SPRITE_HEALTH + j, 1);
                self.ram
                    .write_u8(SPRITE_X_VEL + j, self.ram.read_u8(SPRITE_X_RECOIL + k));
                self.ram
                    .write_u8(SPRITE_Y_VEL + j, self.ram.read_u8(SPRITE_Y_RECOIL + k));
                self.ram.write_u8(SPRITE_DEFL_BITS + j, 64);
            }
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_Z_VEL + k, 16);
                } else {
                    const B: [u8; 8] = [8, 7, 6, 5, 4, 5, 6, 7];
                    self.ram.write_u8(
                        SPRITE_B + k,
                        B[(self.ram.read_u8(SPRITE_DELAY_MAIN + k) >> 1) as usize],
                    );
                }
            }
            1 => {
                self.runtime.move_xyz(self.ram, k);
                self.ram.write_u8(
                    SPRITE_Z_VEL + k,
                    self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
                );
                if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
                    self.ram.write_u8(SPRITE_Z + k, 0);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 15);
                }
                self.runtime.bounce_from_tile_collision(self.ram, k);
            }
            _ => {}
        }
    }

    pub fn hokbok_draw(&mut self, k: usize) {
        let Some((x, mut y, flags)) = self.runtime.prep_oam_or_double_ret_wide(self.ram, k) else {
            return;
        };
        let d = self.ram.read_u8(SPRITE_B + k);
        let a = self.ram.read_u8(SPRITE_A + k);
        let oam = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4 + 3;
        for i in (0..=a).rev() {
            let ch =
                (if i == 0 { 0xa2u8 } else { 0xa0u8 }).wrapping_sub(if d < 7 { 0x20 } else { 0 });
            crate::oam::set_helper0(self.ram, oam - usize::from(a - i), x, y, ch, flags, 2);
            y = y.wrapping_sub(u16::from(d));
        }
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_draw_moldorm_segment_c(&mut self, k: usize) {
        self.ram.write_u8(SPRITE_GRAPHICS + k, 0);
        self.ram.write_u16(
            OAM_CUR_PTR,
            self.ram.read_u16(OAM_CUR_PTR).wrapping_add(0x10),
        );
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(4),
        );
        self.runtime.giant_moldorm_draw_segment(self.ram, k, 0x28);
    }

    pub fn moldorm_handle_tail(&mut self, k: usize) {
        self.sprite_draw_moldorm_tail(k);
        if self.ram.read_u8(SPRITE_DELAY_AUX2 + k) == 0 {
            self.ram.write_u8(SPRITE_A + k, 1);
            self.ram.write_u8(SPRITE_FLAGS4 + k, 0);
            self.ram.write_u8(SPRITE_DEFL_BITS + k, 0);
            let x = self.sprite_x(k);
            let y = self.sprite_y(k);
            self.set_sprite_x(k, self.ram.read_u16(CUR_SPRITE_X));
            self.set_sprite_y(k, self.ram.read_u16(CUR_SPRITE_Y));
            let _ = self.runtime.check_damage_from_link(self.ram, k);
            self.ram.write_u8(SPRITE_A + k, 0);
            self.ram.write_u8(SPRITE_FLAGS4 + k, 9);
            self.ram.write_u8(SPRITE_DEFL_BITS + k, 4);
            self.set_sprite_x(k, x);
            self.set_sprite_y(k, y);
        }
    }

    pub fn sprite_draw_moldorm_tail(&mut self, k: usize) {
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(4));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(1),
        );
        self.ram.write_u8(
            SPRITE_GRAPHICS + k,
            self.ram.read_u8(SPRITE_GRAPHICS + k).wrapping_add(1),
        );
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 13);
        self.runtime.giant_moldorm_draw_segment(self.ram, k, 0x30);
    }

    pub fn sprite_draw_moldorm_eyeballs(&mut self, k: usize, info: &PrepOamCoordsRet) {
        const X: [i16; 16] = [
            16, 15, 12, 6, 0, -6, -12, -13, -16, -13, -12, -6, 0, 6, 12, 15,
        ];
        const Y: [i16; 16] = [
            0, 6, 12, 15, 16, 15, 12, 6, 0, -6, -12, -13, -16, -13, -12, -6,
        ];
        const CH: [u8; 16] = [
            0xaa, 0xaa, 0xa8, 0xa8, 0x8a, 0x8a, 0xa8, 0xa8, 0xaa, 0xaa, 0xa8, 0xa8, 0x8a, 0x8a,
            0xa8, 0xa8,
        ];
        const FL: [u8; 16] = [
            0, 0, 0, 0, 0x80, 0x80, 0x40, 0x40, 0x40, 0x40, 0xc0, 0xc0, 0, 0, 0x80, 0x80,
        ];
        let r7 = if self.ram.read_u8(SPRITE_F + k) != 0 {
            self.ram.read_u8(FRAME_COUNTER)
        } else {
            0
        };
        let mut r6 = self.ram.read_u8(SPRITE_D + k).wrapping_sub(1);
        let first = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for offset in 0..2 {
            let p = (r6 & 15) as usize;
            let q = (r6.wrapping_add(r7) & 15) as usize;
            crate::oam::set_helper0(
                self.ram,
                first + offset,
                info.x.wrapping_add_signed(X[p]),
                info.y.wrapping_add_signed(Y[p]),
                CH[q],
                info.flags | FL[q],
                2,
            );
            r6 = r6.wrapping_add(2);
        }
    }

    pub fn helmasaur_king_swing_tail(&mut self, k: usize) {
        self.ram.write_u8(
            OVERLORD_X_LO + 4,
            self.ram.read_u8(OVERLORD_X_LO + 4).wrapping_add(1),
        );
        self.helmasaur_king_reinitialize(k);
        let mask = if self.ram.read_u8(SPRITE_ANIM_CLOCK + k) != 0 {
            0
        } else {
            1
        };
        if self.ram.read_u8(FRAME_COUNTER) & mask == 0 {
            let j = self.ram.read_u8(SPRITE_D + k) & 1;
            let v = self
                .ram
                .read_u8(OVERLORD_GEN2)
                .wrapping_add(if j != 0 { 255 } else { 1 });
            self.ram.write_u8(OVERLORD_GEN2, v);
            if v == [16u8, 240][j as usize] {
                self.ram
                    .write_u8(SPRITE_D + k, self.ram.read_u8(SPRITE_D + k).wrapping_add(1));
            }
            self.ram.write_u16(
                OVERLORD_GEN1 + 5,
                self.ram
                    .read_u16(OVERLORD_GEN1 + 5)
                    .wrapping_add_signed(v as i8 as i16),
            );
        }
        let a = self.ram.read_u8(SPRITE_ANIM_CLOCK + k);
        if a == 0 {
            return;
        }
        if self.ram.read_u8(OVERLORD_GEN2) == 0 {
            self.runtime.sfx3_with_pan(self.ram, k, 6);
        }
        if a == 2 {
            let j = self.ram.read_u8(SPRITE_HEAD_DIR + k);
            let v = self
                .ram
                .read_u16(OVERLORD_GEN2 + 1)
                .wrapping_add_signed(if j != 0 { -4 } else { 4 });
            self.ram.write_u16(OVERLORD_GEN2 + 1, v);
            if v as u8 == if j != 0 { 132 } else { 124 } {
                self.ram.write_u8(SPRITE_ANIM_CLOCK + k, 3);
            }
            self.ram.write_u8(
                OVERLORD_GEN1 + 7,
                self.ram.read_u8(OVERLORD_GEN1 + 7).wrapping_add(3),
            );
        } else if a == 3 {
            let j = self.ram.read_u8(SPRITE_HEAD_DIR + k) ^ 1;
            let v = self
                .ram
                .read_u16(OVERLORD_GEN2 + 1)
                .wrapping_add_signed(if j != 0 { -4 } else { 4 });
            self.ram.write_u16(OVERLORD_GEN2 + 1, v);
            if v == 0 {
                self.ram.write_u8(SPRITE_ANIM_CLOCK + k, 0);
            }
            self.ram.write_u8(
                OVERLORD_GEN1 + 7,
                self.ram.read_u8(OVERLORD_GEN1 + 7).wrapping_sub(3),
            );
        } else if self.ram.read_u8(OVERLORD_GEN2) | self.ram.read_u8(SPRITE_DELAY_AUX3 + k) == 0 {
            let hd = self.ram.read_u8(OVERLORD_GEN1 + 6) & 1;
            self.ram.write_u8(SPRITE_HEAD_DIR + k, hd);
            let (r, _) = self.runtime.is_right_of_link(self.ram, k);
            if (r ^ 1) == hd {
                self.ram.write_u8(SPRITE_ANIM_CLOCK + k, 2);
                let p = self.runtime.calculate_sfx_pan(self.ram, k);
                self.ram.write_u8(SOUND_EFFECT_2, p | 0x26);
            }
        }
    }

    pub fn helmasaur_king_draw(&mut self, k: usize) {
        self.ram.write_u16(OAM_CUR_PTR, 0x89c);
        self.ram.write_u16(OAM_EXT_CUR_PTR, 0xa47);
        let Some((x, y, flags)) = self.runtime.prep_oam_or_double_ret(self.ram, k) else {
            return;
        };
        let info = PrepOamCoordsRet {
            x: x.into(),
            y: y.into(),
            r4: 0,
            flags,
        };
        self.king_helmasaur_operate_tail(k, &info);
        self.sprite_draw_king_helmasaur_eyes(k, &info);
        self.king_helmasaur_mask(k, &info);
        self.sprite_draw_king_helmasaur_body(k, &info);
        self.sprite_draw_king_helmasaur_legs(k, &info);
        self.sprite_draw_king_helmasaur_mouth(k, &info);
    }

    pub fn sprite_draw_king_helmasaur_eyes(&mut self, k: usize, info: &PrepOamCoordsRet) {
        const X: [i8; 2] = [-3, 11];
        const CH: [u8; 8] = [0xce, 0xcf, 0xde, 0xde, 0xde, 0xde, 0xcf, 0xce];
        const FL: [u8; 2] = [0x3b, 0x7b];
        self.ram.write_u16(
            OAM_CUR_PTR,
            self.ram.read_u16(OAM_CUR_PTR).wrapping_add(0x40),
        );
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(0x10),
        );
        let j = (self.ram.read_u8(OVERLORD_X_LO + 4) >> 2 & 7) as usize;
        let first = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for i in (0..2).rev() {
            crate::oam::set_plain(
                self.ram,
                first + (1 - i),
                info.x.wrapping_add_signed(X[i] as i16) as u8,
                info.y.wrapping_add(0x14) as u8,
                CH[j],
                FL[i],
                0,
            );
        }
        if self.ram.read_u8(SUBMODULE_INDEX) != 0 {
            self.helmasaur_correct_oam_entries(k, 1, 0);
        }
    }

    pub fn king_helmasaur_mask(&mut self, k: usize, _info: &PrepOamCoordsRet) {
        const D: [DrawPiece; 24] = [
            (-16, -5, 0x0dae, 2),
            (0, -5, 0x0dc0, 2),
            (16, -5, 0x4dae, 2),
            (-16, 11, 0x0dc2, 2),
            (0, 11, 0x0dc4, 2),
            (16, 11, 0x4dc2, 2),
            (-8, 27, 0x0dc6, 2),
            (8, 27, 0x4dc6, 2),
            (-16, -5, 0x0dae, 2),
            (0, -5, 0x0dc0, 2),
            (16, -5, 0x4dae, 2),
            (-16, 11, 0x0dc8, 2),
            (0, 11, 0x0dc4, 2),
            (16, 11, 0x4dc2, 2),
            (-8, 27, 0x0dc6, 2),
            (8, 27, 0x4dc6, 2),
            (-16, -5, 0x0dae, 2),
            (0, -5, 0x0dc0, 2),
            (16, -5, 0x4dae, 2),
            (-16, 11, 0x0dc8, 2),
            (0, 11, 0x0dc4, 2),
            (16, 11, 0x4dc8, 2),
            (-8, 27, 0x0dc6, 2),
            (8, 27, 0x4dc6, 2),
        ];
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(8));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(2),
        );
        let c = self.ram.read_u8(SPRITE_C + k);
        if c >= 3 {
            return;
        }
        let i = c as usize * 8;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 8]);
        self.ram.write_u16(
            OAM_CUR_PTR,
            self.ram.read_u16(OAM_CUR_PTR).wrapping_add(0x20),
        );
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(8),
        );
    }

    pub fn sprite_draw_king_helmasaur_legs(&mut self, k: usize, info: &PrepOamCoordsRet) {
        const X: [i8; 4] = [-28, -28, 28, 28];
        const Y: [i8; 4] = [-28, 4, -28, 4];
        const CH: [u8; 4] = [0xa2, 0xa6, 0xa2, 0xa6];
        const FL: [u8; 4] = [0xb, 0xb, 0x4b, 0x4b];
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(76));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(19),
        );
        let first = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        for i in (0..4).rev() {
            let x = info.x.wrapping_add_signed(X[i] as i16) as u8;
            let y = info
                .y
                .wrapping_add_signed(Y[i] as i16)
                .wrapping_add(self.ram.read_u8(OVERLORD_X_LO + i) as u16) as u8;
            let f = FL[i] ^ info.flags;
            let entry = first + 2 * (3 - i);
            crate::oam::set_plain(self.ram, entry, x, y, CH[i], f, 2);
            crate::oam::set_plain(self.ram, entry + 1, x, y.wrapping_add(16), CH[i] + 2, f, 2);
        }
        self.ram.write_u8(TMP_COUNTER, 0xff);
        if self.ram.read_u8(SUBMODULE_INDEX) != 0 {
            self.helmasaur_correct_oam_entries(k, 7, 2);
        }
    }

    fn helmasaur_correct_oam_entries(&mut self, k: usize, last: usize, size: u8) {
        let oam_ptr = usize::from(self.ram.read_u16(OAM_CUR_PTR));
        let ext_ptr = usize::from(self.ram.read_u16(OAM_EXT_CUR_PTR));
        let sprite_x = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_X_LO + k),
            self.ram.read_u8(SPRITE_X_HI + k),
        ]);
        let sprite_y = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_Y_LO + k),
            self.ram.read_u8(SPRITE_Y_HI + k),
        ]);
        let scroll_x = sprite_x.wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2)) as u8;
        let scroll_y = sprite_y.wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2)) as u8;
        for offset in 0..=last {
            let addr = oam_ptr + offset * 4;
            let x = sprite_x
                .wrapping_add((self.ram.read_u8(addr).wrapping_sub(scroll_x) as i8) as i16 as u16);
            let y = sprite_y.wrapping_add(
                (self.ram.read_u8(addr + 1).wrapping_sub(scroll_y) as i8) as i16 as u16,
            );
            self.ram.write_u8(
                ext_ptr + offset,
                size.wrapping_add(u8::from(
                    x.wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2)) >= 0x100,
                )),
            );
            if y.wrapping_add(0x10)
                .wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2))
                >= 0x100
            {
                self.ram.write_u8(addr + 1, 0xf0);
            }
        }
    }

    pub fn king_helmasaur_operate_tail(&mut self, k: usize, info: &PrepOamCoordsRet) {
        const M: [u8; 32] = [
            0xff, 0xf0, 0xe0, 0xd0, 0xc0, 0xb0, 0xa0, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30,
            0x20, 0x10, 0xff, 0xf8, 0xf0, 0xe8, 0xe0, 0xd8, 0xd0, 0xc8, 0xbc, 0xb0, 0xa0, 0x90,
            0x70, 0x40, 0x20, 0x10,
        ];
        const MB: [u8; 16] = [
            0xff, 0xf0, 0xe0, 0xd0, 0xc0, 0xb0, 0xa0, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30,
            0x20, 0x10,
        ];
        for i in 0..16 {
            let j = i + if self.ram.read_u8(SPRITE_ANIM_CLOCK + k) != 0 {
                16
            } else {
                0
            };
            let rs = self
                .ram
                .read_u16(OVERLORD_GEN1 + 5)
                .wrapping_add(self.ram.read_u16(OVERLORD_GEN2 + 1));
            let f = ((rs >> 8) as u8).wrapping_sub(1);
            let r6 = ((if (f as i8) < 0 {
                0u16.wrapping_sub(rs)
            } else {
                rs
            }) as u8 as u16
                * M[j] as u16
                >> 8) as u8;
            let angle = (rs & 0xff00)
                | if (f as i8) < 0 {
                    (r6 ^ 0xff) as u16
                } else {
                    r6 as u16
                };
            let r15 = ((self.ram.read_u8(OVERLORD_GEN1 + 7) as u16 * MB[i] as u16) >> 8) as u8;
            self.ram
                .write_u8(OVERLORD_X_LO + i + 5, ganon_sin(angle, r15) as u8);
            self.ram.write_u8(
                OVERLORD_Y_LO + i + 5,
                ganon_sin(angle.wrapping_add(0x80), r15).wrapping_sub(40) as u8,
            );
        }
        let start = self.ram.read_u8(OVERLORD_GEN2 + 3);
        let oam_ptr = self.ram.read_u16(OAM_CUR_PTR);
        let ext_ptr = self.ram.read_u16(OAM_EXT_CUR_PTR);
        for i in start..16 {
            let entry =
                (usize::from(oam_ptr) - crate::oam::OAM_BUFFER) / 4 + usize::from(i - start);
            crate::oam::set_plain(
                self.ram,
                entry,
                info.x
                    .wrapping_add(self.ram.read_u8(OVERLORD_X_LO + i as usize + 5) as u16)
                    as u8,
                info.y
                    .wrapping_add(self.ram.read_u8(OVERLORD_Y_LO + i as usize + 5) as u16)
                    as u8,
                if i == start { 0xe4 } else { 0xac },
                info.flags ^ 0x1b,
                2,
            );
        }
        // The C draw loop writes through a local OAM pointer. It leaves the
        // global cursors at the tail's start for the following correction and
        // the eyes, mask, body and legs that use fixed offsets from it.
        let sprite_x = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_X_LO + k),
            self.ram.read_u8(SPRITE_X_HI + k),
        ]);
        let sprite_y = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_Y_LO + k),
            self.ram.read_u8(SPRITE_Y_HI + k),
        ]);
        let scroll_x = sprite_x.wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2)) as u8;
        let scroll_y = sprite_y.wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2)) as u8;
        for offset in 0..=16usize {
            let entry = (usize::from(oam_ptr) - crate::oam::OAM_BUFFER) / 4 + offset;
            let oam_addr = crate::oam::OAM_BUFFER + entry * 4;
            let x = sprite_x.wrapping_add(
                (self.ram.read_u8(oam_addr).wrapping_sub(scroll_x) as i8) as i16 as u16,
            );
            let y = sprite_y.wrapping_add(
                (self.ram.read_u8(oam_addr + 1).wrapping_sub(scroll_y) as i8) as i16 as u16,
            );
            let ext_addr = usize::from(ext_ptr) + offset;
            let ext = 2u8;
            self.ram.write_u8(
                ext_addr,
                ext.wrapping_add(u8::from(
                    x.wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2)) >= 0x100,
                )),
            );
            if y.wrapping_add(0x10)
                .wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2))
                >= 0x100
            {
                self.ram.write_u8(oam_addr + 1, 0xf0);
            }
        }
        self.ram.write_u8(TMP_COUNTER, 16);
    }

    pub fn goriya_draw(&mut self, k: usize) {
        const D2: [DrawPiece; 3] = [(10, 4, 0x4077, 0), (-2, 4, 0x0077, 0), (4, 4, 0x0076, 0)];
        const D: [DrawPiece; 32] = [
            (-4, -8, 0x0044, 2),
            (12, -8, 0x4044, 0),
            (-4, 8, 0x0064, 0),
            (4, 0, 0x4054, 2),
            (-4, -8, 0x0044, 2),
            (12, -8, 0x4044, 0),
            (-4, 8, 0x4074, 0),
            (4, 0, 0x4062, 2),
            (-4, -8, 0x0044, 0),
            (4, -8, 0x4044, 2),
            (-4, 0, 0x0062, 2),
            (12, 8, 0x4064, 0),
            (-4, -8, 0x0046, 2),
            (12, -8, 0x4046, 0),
            (-4, 8, 0x0066, 0),
            (4, 0, 0x4056, 2),
            (-4, -8, 0x0046, 2),
            (12, -8, 0x4046, 0),
            (-4, 8, 0x4075, 0),
            (4, 0, 0x406a, 2),
            (-4, -8, 0x0046, 0),
            (4, -8, 0x4046, 2),
            (-4, 0, 0x006a, 2),
            (12, 8, 0x0075, 0),
            (-2, -8, 0x004e, 2),
            (0, 0, 0x006c, 2),
            (-2, -7, 0x004e, 2),
            (0, 0, 0x006e, 2),
            (2, -8, 0x404e, 2),
            (0, 0, 0x406c, 2),
            (2, -7, 0x404e, 2),
            (0, 0, 0x406e, 2),
        ];
        const O: [usize; 11] = [0, 4, 8, 12, 16, 20, 24, 26, 28, 30, 32];
        let d = self.ram.read_u8(SPRITE_D + k) as usize;
        if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) != 0 && d != 3 {
            self.runtime.draw_multiple(self.ram, k, &D2[d..d + 1]);
        }
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(4));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(1),
        );
        let g = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize;
        self.runtime.draw_multiple(self.ram, k, &D[O[g]..O[g + 1]]);
        self.ram.write_u8(
            SPRITE_FLAGS2 + k,
            self.ram.read_u8(SPRITE_FLAGS2 + k).wrapping_sub(1),
        );
        self.runtime.draw_shadow(self.ram, k);
        self.ram.write_u8(
            SPRITE_FLAGS2 + k,
            self.ram.read_u8(SPRITE_FLAGS2 + k).wrapping_add(1),
        );
    }

    pub fn vitreous_spawn_smaller_eyes(&mut self, k: usize) {
        const X: [i8; 13] = [8, 22, -8, -22, 0, 14, 19, 33, 26, -14, -19, -33, -26];
        const Y: [i8; 13] = [-8, -12, -8, -12, 0, -20, -1, -12, -24, -20, -1, -12, -24];
        const G: [u8; 13] = [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        self.ram.write_u8(SPRITE_G + k, 9);
        self.ram.write_u8(SPRITE_GRAPHICS + k, 4);
        let bx = self.sprite_x(k);
        let by = self.sprite_y(k);
        let _ = self.runtime.spawn_dynamic_ex(self.ram, k, 4, 13);
        for j in (1usize..=13).rev() {
            self.ram.write_u8(SPRITE_STATE + j, 9);
            self.ram.write_u8(SPRITE_TYPE + j, 0xbe);
            self.runtime.load_properties(self.ram, j);
            self.ram.write_u8(SPRITE_FLOOR + j, 0);
            self.set_sprite_x(j, bx.wrapping_add_signed(X[j - 1] as i16));
            self.set_sprite_y(j, by.wrapping_add_signed(Y[j - 1] as i16 + 32));
            self.ram
                .write_u8(SPRITE_A + j, self.ram.read_u8(SPRITE_X_LO + j));
            self.ram
                .write_u8(SPRITE_B + j, self.ram.read_u8(SPRITE_X_HI + j));
            self.ram
                .write_u8(SPRITE_C + j, self.ram.read_u8(SPRITE_Y_LO + j));
            self.ram
                .write_u8(SPRITE_D + j, self.ram.read_u8(SPRITE_Y_HI + j));
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, G[j - 1]);
            self.ram.write_u8(SPRITE_GRAPHICS + j, G[j - 1]);
            let r = self.runtime.random_number(self.ram);
            self.ram
                .write_u8(SPRITE_SUBTYPE2 + j, (((j - 1) * 8) as u8).wrapping_add(r));
        }
    }

    pub fn sprite_c0_catfish(&mut self, k: usize) {
        let a = self.ram.read_u8(SPRITE_A + k);
        if a & 0x80 != 0 {
            self.sprite_catfish_splash_of_water(k)
        } else if a == 0 {
            self.catfish_big_fish(k)
        } else {
            self.sprite_catfish_quake_medallion(k)
        }
    }

    pub fn sprite_catfish_quake_medallion(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_Z + k) == 0 {
            self.runtime.draw_water_ripple_with_oam_adjust(self.ram, k);
            if self.ram.read_u8(SUBMODULE_INDEX) == 0
                && self.runtime.check_damage_to_link_same_layer(self.ram, k)
            {
                self.ram.write_u8(SPRITE_STATE + k, 0);
                self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
                self.runtime
                    .link_receive_item(self.ram, self.ram.read_u8(SPRITE_A + k), 0);
            }
        }
        if self.ram.read_u8(SPRITE_DELAY_AUX3 + k) != 0 {
            self.runtime.oam_allocate_region_c(self.ram, 8);
        }
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.move_xyz(self.ram, k);
        self.ram.write_u8(
            SPRITE_Z_VEL + k,
            self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
        );
        if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
            self.ram.write_u8(SPRITE_Z + k, 0);
            self.ram.write_u8(
                SPRITE_X_VEL + k,
                ((self.ram.read_u8(SPRITE_X_VEL + k) as i8) >> 1) as u8,
            );
            self.ram.write_u8(
                SPRITE_Y_VEL + k,
                ((self.ram.read_u8(SPRITE_Y_VEL + k) as i8) >> 1) as u8,
            );
            let j = self.ram.read_u8(SPRITE_AI_STATE + k);
            if j == 4 {
                self.ram.write_u8(SPRITE_X_VEL + k, 0);
                self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                self.ram.write_u8(SPRITE_Z_VEL + k, 0);
            } else {
                self.ram.write_u8(SPRITE_AI_STATE + k, j + 1);
                self.ram
                    .write_u8(SPRITE_Z_VEL + k, [0x20, 0x10, 8, 0][j as usize]);
                if j < 2
                    && let Some(n) = self.sprite_spawn_water_splash(k)
                {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + n, 16);
                }
            }
        }
    }

    pub fn catfish_big_fish(&mut self, k: usize) {
        self.great_catfish_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                for j in (0usize..16).rev() {
                    if j != k
                        && self.ram.read_u8(SPRITE_STATE + j) == 3
                        && self
                            .ram
                            .read_u16(CUR_SPRITE_X)
                            .wrapping_sub(self.sprite_x(j))
                            .wrapping_add(32)
                            < 64
                        && self
                            .ram
                            .read_u16(CUR_SPRITE_Y)
                            .wrapping_sub(self.sprite_y(j))
                            .wrapping_add(32)
                            < 64
                    {
                        self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                        return;
                    }
                }
            }
            1 => {
                let j = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if j == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                    self.ram.write_u16(BG1_X_OFFSET, 0);
                    self.ram.write_u8(SOUND_EFFECT_AMBIENT, 5);
                    self.ram.write_u8(SPRITE_Z_VEL + k, 48);
                    self.ram.write_u8(SPRITE_X_VEL + k, 0);
                    self.catfish_spawn_plop(k);
                } else if j < 0xc0 {
                    if j == 0xbf {
                        self.ram.write_u8(SOUND_EFFECT_AMBIENT, 7);
                    }
                    self.ram
                        .write_u16(BG1_X_OFFSET, if j & 1 != 0 { u16::MAX } else { 1 });
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                }
            }
            2 => {
                const G: [u8; 16] = [1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 0, 0, 0, 0];
                let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
                self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
                self.runtime.move_xyz(self.ram, k);
                let z = self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2);
                self.ram.write_u8(SPRITE_Z_VEL + k, z);
                if z == (-48i8) as u8 {
                    self.catfish_spawn_plop(k);
                }
                if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
                    self.ram.write_u8(SPRITE_Z + k, 0);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                }
                self.ram.write_u8(SPRITE_GRAPHICS + k, G[(s >> 2) as usize]);
            }
            3 => {
                const G: [u8; 20] = [0, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6, 6];
                let j = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if j == 0 {
                    self.ram.write_u8(SPRITE_STATE + k, 0);
                } else {
                    if j == 160 || j == 252 || j == 4 {
                        self.sprite_spawn_water_splash(k);
                    } else if j == 10 {
                        self.catfish_spawn_plop(k);
                    } else if j == 96 {
                        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                        self.ram.write_u16(
                            DIALOGUE_MESSAGE_INDEX,
                            if self.ram.read_u8(LINK_ITEM_QUAKE_MEDALLION) != 0 {
                                0x12b
                            } else {
                                0x12a
                            },
                        );
                        self.runtime.show_message_minimal(self.ram);
                        return;
                    } else if j == 80 {
                        if self.ram.read_u8(LINK_ITEM_QUAKE_MEDALLION) != 0 {
                            if self.runtime.random_number(self.ram) & 1 != 0 {
                                self.sprite_spawn_bomb(k);
                            } else {
                                self.runtime.spawn_fireball(self.ram, k);
                            }
                        } else {
                            self.catfish_regurgitate_medallion(k);
                        }
                    }
                    if j < 160 {
                        self.ram.write_u8(SPRITE_GRAPHICS + k, G[(j >> 3) as usize]);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn vitreous_draw(&mut self, k: usize) {
        const D: [DrawPiece; 24] = [
            (-8, -8, 0x01c0, 2),
            (8, -8, 0x41c0, 2),
            (-8, 8, 0x01e0, 2),
            (8, 8, 0x41e0, 2),
            (-8, -8, 0x01c8, 2),
            (8, -8, 0x01ca, 2),
            (-8, 8, 0x01e8, 2),
            (8, 8, 0x01ea, 2),
            (-8, -8, 0x41ca, 2),
            (8, -8, 0x41c8, 2),
            (-8, 8, 0x41ea, 2),
            (8, 8, 0x41e8, 2),
            (-8, -8, 0x01c2, 2),
            (8, -8, 0x41c2, 2),
            (-8, 8, 0x01e2, 2),
            (8, 8, 0x41e2, 2),
            (-8, -8, 0x01c4, 2),
            (8, -8, 0x41c4, 2),
            (-8, 8, 0x01e4, 2),
            (8, 8, 0x41e4, 2),
            (-7, -7, 0x01c4, 2),
            (7, -7, 0x41c4, 2),
            (-7, 7, 0x01e4, 2),
            (7, 7, 0x41e4, 2),
        ];
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 2 && self.ram.read_u8(SPRITE_STATE + k) == 9 {
            self.ram.write_u16(OAM_CUR_PTR, 0x800);
            self.ram.write_u16(OAM_EXT_CUR_PTR, 0xa20);
        }
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 2 {
            self.ram.write_u8(
                SPRITE_OBJ_PRIO + k,
                self.ram.read_u8(SPRITE_OBJ_PRIO + k) & !0xe,
            );
            self.sprite_draw_large_shadow2(k);
        }
    }

    pub fn sprite_bd_vitreous(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_DELAY_AUX4 + k) != 0 {
            self.ram.write_u8(SPRITE_GRAPHICS + k, 3);
        }
        self.vitreous_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.vitreous_set_minions_forth(k);
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                self.ram.write_u8(BYTE_7E0FF8, 0);
                self.ram.write_u8(SPRITE_F + k, 0);
                self.ram
                    .write_u8(SPRITE_FLAGS3 + k, self.ram.read_u8(SPRITE_FLAGS3 + k) | 64);
                if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
                    let a = self.ram.read_u8(SPRITE_A + k).wrapping_sub(1);
                    self.ram.write_u8(SPRITE_A + k, a);
                    if a == 0 {
                        self.ram.write_u8(
                            SPRITE_FLAGS3 + k,
                            self.ram.read_u8(SPRITE_FLAGS3 + k) & !0x40,
                        );
                        self.ram.write_u8(SPRITE_DELAY_AUX4 + k, 16);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
                        if self.ram.read_u8(SPRITE_G + k) == 0 {
                            self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 64);
                            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + k, 0);
                            self.ram.write_u8(SOUND_EFFECT_1, 0x35);
                            return;
                        }
                    }
                }
                self.ram.write_u8(
                    SPRITE_GRAPHICS + k,
                    if self.ram.read_u8(FRAME_COUNTER) & 0x30 != 0 {
                        4
                    } else {
                        5
                    },
                );
            }
            1 => {
                self.ram.write_u8(SPRITE_F + k, 0);
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if d == 0 {
                    const A: [u8; 10] = [0x20, 0x20, 0x20, 0x40, 0x60, 0x80, 0xa0, 0xc0, 0xe0, 0];
                    self.ram.write_u8(SPRITE_DELAY_AUX4 + k, 16);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram
                        .write_u8(SPRITE_A + k, A[self.ram.read_u8(SPRITE_G + k) as usize]);
                } else {
                    self.vitreous_animate(k, d);
                }
            }
            2 => {
                self.vitreous_animate(k, 0x8b);
                if self.runtime.return_if_recoiling(self.ram, k) {
                    return;
                }
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) != 0 {
                    let v =
                        [8u8, 248][((self.ram.read_u8(SPRITE_DELAY_MAIN + k) & 2) >> 1) as usize];
                    self.ram.write_u8(SPRITE_X_VEL + k, v);
                    self.runtime.move_x(self.ram, k);
                } else {
                    self.runtime.move_xyz(self.ram, k);
                    self.runtime.check_tile_collision(self.ram, k);
                    self.ram.write_u8(
                        SPRITE_Z_VEL + k,
                        self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
                    );
                    if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
                        self.ram.write_u8(SPRITE_Z + k, 0);
                        self.ram.write_u8(SPRITE_Z_VEL + k, 32);
                        self.runtime.apply_speed_towards_link(self.ram, k, 16);
                        self.runtime.sfx2_with_pan(self.ram, k, 0x21);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn sprite_be_vitreous_eye(&mut self, k: usize) {
        const DX: [i8; 4] = [1, 0, -1, 0];
        const DY: [i8; 4] = [0, 1, 0, -1];
        let q = (self.ram.read_u8(SPRITE_SUBTYPE2 + k) >> 4 & 3) as usize;
        self.ram.write_u16(
            CUR_SPRITE_X,
            self.ram
                .read_u16(CUR_SPRITE_X)
                .wrapping_add_signed(DX[q] as i16),
        );
        self.ram.write_u16(
            CUR_SPRITE_Y,
            self.ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_add_signed(DY[q] as i16),
        );
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.write_u8(
            SPRITE_SUBTYPE2 + k,
            self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1),
        );
        if self.ram.read_u8(SPRITE_GRAPHICS + k) != 0 {
            return;
        }
        let _ = self.runtime.check_damage_from_link(self.ram, k);
        let _ = self.runtime.check_damage_to_link(self.ram, k);
        if self.ram.read_u8(SPRITE_F + k) == 14 {
            self.ram.write_u8(SPRITE_F + k, 5);
        }
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                let [xl, xh] = self.ram.read_u16(LINK_X).to_le_bytes();
                let [yl, yh] = self.ram.read_u16(LINK_Y).to_le_bytes();
                self.ram.write_u8(SPRITE_G + k, xl);
                self.ram.write_u8(SPRITE_HEAD_DIR + k, xh);
                self.ram.write_u8(SPRITE_ANIM_CLOCK + k, yl);
                self.ram.write_u8(SPRITE_SUBTYPE + k, yh);
            }
            1 | 2 => {
                if self.runtime.return_if_recoiling(self.ram, k) {
                    return;
                }
                let returning = self.ram.read_u8(SPRITE_AI_STATE + k) == 2;
                if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 1 == 0 {
                    let (x, y) = if returning {
                        (
                            u16::from_le_bytes([
                                self.ram.read_u8(SPRITE_A + k),
                                self.ram.read_u8(SPRITE_B + k),
                            ]),
                            u16::from_le_bytes([
                                self.ram.read_u8(SPRITE_C + k),
                                self.ram.read_u8(SPRITE_D + k),
                            ]),
                        )
                    } else {
                        (
                            u16::from_le_bytes([
                                self.ram.read_u8(SPRITE_G + k),
                                self.ram.read_u8(SPRITE_HEAD_DIR + k),
                            ]),
                            u16::from_le_bytes([
                                self.ram.read_u8(SPRITE_ANIM_CLOCK + k),
                                self.ram.read_u8(SPRITE_SUBTYPE + k),
                            ]),
                        )
                    };
                    let (xv, yv) = self
                        .runtime
                        .project_speed_towards_location(self.ram, k, x, y, 16);
                    self.ram.write_u8(SPRITE_X_VEL + k, xv);
                    self.ram.write_u8(SPRITE_Y_VEL + k, yv);
                }
                self.runtime.move_xy(self.ram, k);
                let tx = if returning {
                    self.ram.read_u8(SPRITE_A + k)
                } else {
                    self.ram.read_u8(SPRITE_G + k)
                };
                let ty = if returning {
                    self.ram.read_u8(SPRITE_C + k)
                } else {
                    self.ram.read_u8(SPRITE_ANIM_CLOCK + k)
                };
                if tx
                    .wrapping_sub(self.ram.read_u8(SPRITE_X_LO + k))
                    .wrapping_add(4)
                    < 8
                    && ty
                        .wrapping_sub(self.ram.read_u8(SPRITE_Y_LO + k))
                        .wrapping_add(4)
                        < 8
                {
                    if returning {
                        self.ram
                            .write_u8(SPRITE_X_LO + k, self.ram.read_u8(SPRITE_A + k));
                        self.ram
                            .write_u8(SPRITE_X_HI + k, self.ram.read_u8(SPRITE_B + k));
                        self.ram
                            .write_u8(SPRITE_Y_LO + k, self.ram.read_u8(SPRITE_C + k));
                        self.ram
                            .write_u8(SPRITE_Y_HI + k, self.ram.read_u8(SPRITE_D + k));
                        self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    } else {
                        self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn sprite_d5_dig_game_guy(&mut self, k: usize) {
        self.digging_game_guy_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.runtime.move_xy(self.ram, k);
        self.ram.write_u8(SPRITE_X_VEL + k, 0);
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                if self.ram.read_u8(SPRITE_Y_LO + k).wrapping_add(7) < self.ram.read_u8(LINK_Y)
                    && self.runtime.direction_to_face_link(self.ram, k) == 2
                {
                    if self.ram.read_u8(FOLLOWER_INDICATOR) == 0 {
                        if self.runtime.show_solicited_message(self.ram, k, 0x187) & 0x100 != 0 {
                            self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                        }
                    } else {
                        self.runtime.show_solicited_message(self.ram, k, 0x18c);
                    }
                }
            }
            1 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0
                    && self.ram.read_u16(LINK_RUPEES_GOAL) >= 80
                {
                    self.ram
                        .write_u16(LINK_RUPEES_GOAL, self.ram.read_u16(LINK_RUPEES_GOAL) - 80);
                    self.runtime.show_message_unconditional(self.ram, 0x188);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    self.ram.write_u8(SPRITE_GRAPHICS + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 80);
                    self.ram.write_u8(BEAMOS_X_HI, 0);
                    self.ram.write_u8(BEAMOS_X_HI + 1, 0);
                    self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 5);
                    self.runtime.initialize_secondary_item_minigame(self.ram, 1);
                    self.ram.write_u8(MUSIC_CONTROL, 14);
                } else {
                    self.runtime.show_message_unconditional(self.ram, 0x189);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                }
            }
            2 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                    self.ram.write_u8(SPRITE_GRAPHICS + k, 1);
                } else if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) == 0 {
                    let g = self.ram.read_u8(SPRITE_GRAPHICS + k) ^ 3;
                    self.ram.write_u8(SPRITE_GRAPHICS + k, g);
                    if g & 1 != 0 {
                        self.ram.write_u8(SPRITE_X_VEL + k, (-16i8) as u8);
                    }
                    self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 5);
                }
            }
            3 => {
                self.ram.write_u8(SPRITE_AI_STATE + k, 4);
                self.ram.write_u8(SUPER_BOMB_INDICATOR_UNK1, 0);
                self.ram.write_u8(SUPER_BOMB_INDICATOR_UNK2, 30);
            }
            4 => {
                if (self.ram.read_u8(SUPER_BOMB_INDICATOR_UNK2) as i8) > 0
                    || self.ram.read_u8(LINK_POSITION_MODE) & 1 != 0
                {
                    return;
                }
                self.ram.write_u8(MUSIC_CONTROL, 9);
                self.ram.write_u8(SPRITE_AI_STATE + k, 5);
                self.ram.write_u8(IS_ARCHER_OR_SHOVEL_GAME, 0);
                self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x18a);
                self.runtime.show_message_minimal(self.ram);
                self.ram.write_u8(SUPER_BOMB_INDICATOR_UNK2, 254);
            }
            5 => {
                self.runtime.show_solicited_message(self.ram, k, 0x18b);
            }
            _ => {}
        }
    }

    pub fn sprite_aa_pikit(&mut self, k: usize) {
        const G: [u8; 24] = [
            2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2,
        ];
        const O: [i8; 72] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 12, 16, 24, 32, 32, 24, 16, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, -12, -16, -24, -32, -32, -24, -16, -12, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const T0: [usize; 8] = [24, 24, 0, 48, 48, 48, 0, 24];
        const T1: [usize; 8] = [0, 24, 24, 24, 0, 48, 48, 48];
        self.runtime.pikit_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    let c = self.ram.read_u8(SPRITE_C + k).wrapping_add(1);
                    self.ram.write_u8(SPRITE_C + k, c);
                    let j = if c == 4 {
                        self.ram.write_u8(SPRITE_C + k, 0);
                        self.runtime.direction_to_face_link(self.ram, k)
                    } else {
                        self.runtime.random_number(self.ram) & 3
                    };
                    self.ram
                        .write_u8(SPRITE_X_VEL + k, [16u8, 240, 0, 0][j as usize]);
                    self.ram
                        .write_u8(SPRITE_Y_VEL + k, [0u8, 0, 16, 240][j as usize]);
                    let r = self.runtime.random_number(self.ram);
                    self.ram.write_u8(SPRITE_Z_VEL + k, (r & 7) + 19);
                }
                let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
                self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
                self.ram.write_u8(SPRITE_GRAPHICS + k, s >> 3 & 1);
            }
            1 => {
                self.runtime.move_xyz(self.ram, k);
                self.runtime.check_tile_collision(self.ram, k);
                self.ram.write_u8(
                    SPRITE_Z_VEL + k,
                    self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
                );
                if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
                    self.ram.write_u8(SPRITE_Z + k, 0);
                    self.ram.write_u8(SPRITE_Z_VEL + k, 0);
                    // Sprite_DirectionToFaceLink supplies the low-byte deltas
                    // used by C's proximity test.  Its Y delta includes the
                    // player's +8 origin adjustment and the sprite Z height.
                    // Calling the route also preserves tmp_counter.
                    let _ = self.runtime.direction_to_face_link(self.ram, k);
                    let sprite_x = u16::from(self.ram.read_u8(SPRITE_X_LO + k))
                        | (u16::from(self.ram.read_u8(SPRITE_X_HI + k)) << 8);
                    let dx = self.ram.read_u16(LINK_X).wrapping_sub(sprite_x) as u8;
                    let dy = self
                        .ram
                        .read_u8(LINK_Y)
                        .wrapping_add(8)
                        .wrapping_add(self.ram.read_u8(SPRITE_Z + k))
                        .wrapping_sub(self.ram.read_u8(SPRITE_Y_LO + k));
                    if dx.wrapping_add(48) < 96 && dy.wrapping_add(48) < 96 {
                        self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                        let (xv, yv) = self.runtime.project_speed_towards_link(self.ram, k, 31);
                        self.ram.write_u8(
                            SPRITE_D + k,
                            self.runtime.convert_velocity_to_angle(xv, yv) >> 1,
                        );
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 95);
                        return;
                    }
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 16);
                }
                let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
                self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
                self.ram.write_u8(SPRITE_GRAPHICS + k, s >> 3 & 1);
            }
            2 => {
                let mut j = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if j == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 16);
                    self.ram.write_u8(SPRITE_A + k, 0);
                    self.ram.write_u8(SPRITE_B + k, 0);
                    self.ram.write_u8(SPRITE_G + k, 0);
                    return;
                }
                j >>= 2;
                self.ram.write_u8(SPRITE_GRAPHICS + k, G[j as usize]);
                let d = self.ram.read_u8(SPRITE_D + k) as usize;
                let xo = O[j as usize + T0[d]];
                let yo = O[j as usize + T1[d]];
                self.ram.write_u8(SPRITE_A + k, xo as u8);
                self.ram.write_u8(SPRITE_B + k, yo as u8);
                if self.ram.read_u8(SPRITE_G + k) == 0
                    && self
                        .ram
                        .read_u16(CUR_SPRITE_X)
                        .wrapping_add_signed(xo as i16)
                        .wrapping_sub(self.ram.read_u16(LINK_X))
                        .wrapping_add(12)
                        < 24
                    && self
                        .ram
                        .read_u16(CUR_SPRITE_Y)
                        .wrapping_add_signed(yo as i16)
                        .wrapping_sub(self.ram.read_u16(LINK_Y))
                        .wrapping_add(12)
                        < 32
                    && self.ram.read_u8(SPRITE_DELAY_MAIN + k) < 46
                {
                    let p = self.runtime.calculate_sfx_pan(self.ram, k);
                    self.ram.write_u8(SOUND_EFFECT_1, p | 0x26);
                    let item = (self.runtime.random_number(self.ram) & 3) + 1;
                    self.ram.write_u8(SPRITE_E + k, item);
                    self.ram.write_u8(SPRITE_G + k, item);
                    let addr = match item {
                        1 => LINK_ITEM_BOMBS,
                        2 => LINK_NUM_ARROWS,
                        3 => LINK_RUPEES_GOAL,
                        _ => LINK_SHIELD_TYPE,
                    };
                    if item == 4 {
                        // The shield is remembered on the Pikit so its stolen-item
                        // draw uses the right palette, then removed wholesale from
                        // Link.  Level 3 is intentionally immune to theft.
                        let shield = self.ram.read_u8(addr);
                        self.ram.write_u8(SPRITE_SUBTYPE + k, shield);
                        if shield != 0 && shield != 3 {
                            self.ram.write_u8(addr, 0);
                        } else {
                            self.ram.write_u8(SPRITE_G + k, 0);
                        }
                    } else if self.ram.read_u8(addr) != 0 {
                        self.ram
                            .write_u8(addr, self.ram.read_u8(addr).wrapping_sub(1));
                    } else {
                        self.ram.write_u8(SPRITE_G + k, 0);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn sprite_a8_green_zirro(&mut self, k: usize) {
        const G: [u8; 4] = [9, 10, 8, 7];
        self.ram.write_u8(SPRITE_OBJ_PRIO + k, 0x30);
        if self.ram.read_u8(SPRITE_A + k) != 0 {
            if self.ram.read_u8(SPRITE_AI_STATE + k) == 0 {
                self.runtime.draw_single_small(self.ram, k);
                if self.runtime.return_if_inactive(self.ram, k) {
                    return;
                }
                self.runtime.move_xy(self.ram, k);
                self.runtime.move_z(self.ram, k);
                self.ram.write_u8(
                    SPRITE_Z_VEL + k,
                    self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
                );
                if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
                    self.ram.write_u8(SPRITE_Z + k, 0);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 19);
                    self.ram.write_u8(
                        SPRITE_FLAGS2 + k,
                        self.ram.read_u8(SPRITE_FLAGS2 + k).wrapping_add(1),
                    );
                    self.runtime.sfx2_with_pan(self.ram, k, 0xc);
                }
            } else {
                self.runtime.draw_zirro_bomb(self.ram, k);
                if self.runtime.return_if_inactive(self.ram, k) {
                    return;
                }
                if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 {
                    self.ram.write_u8(
                        SPRITE_DELAY_MAIN + k,
                        self.ram.read_u8(SPRITE_DELAY_MAIN + k).wrapping_add(1),
                    );
                }
                let _ = self.runtime.check_damage_to_link(self.ram, k);
            }
            return;
        }
        if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) != 0 {
            self.ram.write_u8(
                SPRITE_GRAPHICS + k,
                G[self.ram.read_u8(SPRITE_D + k) as usize],
            );
        }
        self.ram.write_u8(
            SPRITE_OBJ_PRIO + k,
            self.ram.read_u8(SPRITE_OBJ_PRIO + k) | 0x30,
        );
        self.runtime.bomber_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) == 8 {
            self.runtime.zirro_drop_bomb(self.ram, k);
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
            let j = self.ram.read_u8(SPRITE_G + k) & 1;
            let z = self
                .ram
                .read_u8(SPRITE_Z_VEL + k)
                .wrapping_add(if j != 0 { 255 } else { 1 });
            self.ram.write_u8(SPRITE_Z_VEL + k, z);
            if z == if j != 0 { 248 } else { 8 } {
                self.ram
                    .write_u8(SPRITE_G + k, self.ram.read_u8(SPRITE_G + k).wrapping_add(1));
            }
        }
        self.runtime.move_z(self.ram, k);
        // C calls Sprite_DirectionToFaceLink here even though it only uses the
        // returned deltas.  Preserve its tmp_counter side effect before the
        // proximity check; some state paths do not reach `set_dir` below.
        let _ = self.runtime.direction_to_face_link(self.ram, k);
        let dx = self
            .ram
            .read_u16(LINK_X)
            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X)) as u8;
        let dy = self
            .ram
            .read_u16(LINK_Y)
            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y)) as u8;
        if dx.wrapping_add(40) < 80
            && dy.wrapping_add(40) < 80
            && self.ram.read_u8(PLAYER_OAM_Y_OFFSET) != 0x80
            && (self.ram.read_u8(LINK_IS_RUNNING) != 0
                || (self.ram.read_u8(BUTTON_B_FRAMES).wrapping_sub(9) as i8) < 0)
        {
            let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 0x30);
            self.ram.write_u8(SPRITE_X_VEL + k, 0u8.wrapping_sub(x));
            self.ram.write_u8(SPRITE_Y_VEL + k, 0u8.wrapping_sub(y));
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 8);
            self.ram.write_u8(SPRITE_AI_STATE + k, 2);
        }
        let state = self.ram.read_u8(SPRITE_AI_STATE + k);
        // The C switch reaches `set_dir` from every state-0/state-2 path and
        // only from the non-expired state-1 path.  In particular, state 1
        // transitioning back to state 0 must not advance `sprite_subtype2`.
        let mut set_dir = state != 1;
        if state == 0 && self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
            const XV: [i8; 8] = [16, 12, 0, -12, -16, -12, 0, 12];
            const YV: [i8; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
            const T: [u8; 4] = [0, 4, 2, 6];
            self.ram.write_u8(SPRITE_AI_STATE + k, 1);
            let b = self.ram.read_u8(SPRITE_B + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_B + k, b);
            let i = if b == 3 {
                self.ram.write_u8(SPRITE_B + k, 0);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
                T[self.runtime.direction_to_face_link(self.ram, k) as usize]
            } else {
                let r = self.runtime.random_number(self.ram);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, (r & 0x1f) | 0x20);
                r & 7
            };
            self.ram.write_u8(SPRITE_X_VEL + k, XV[i as usize] as u8);
            self.ram.write_u8(SPRITE_Y_VEL + k, YV[i as usize] as u8);
        } else if state == 1 {
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                set_dir = false;
                self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 10);
                if self.ram.read_u8(SPRITE_TYPE + k) == 0xa8 {
                    self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 16);
                }
            } else {
                set_dir = true;
                self.runtime.move_xy(self.ram, k);
            }
        } else if state == 2 {
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                self.ram.write_u8(SPRITE_AI_STATE + k, 0);
            }
            self.ram.write_u8(
                SPRITE_SUBTYPE2 + k,
                self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(2),
            );
            self.runtime.move_xy(self.ram, k);
        }
        if set_dir {
            let d = self.runtime.direction_to_face_link(self.ram, k);
            self.ram.write_u8(SPRITE_D + k, d);
            let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
            self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
            self.ram
                .write_u8(SPRITE_GRAPHICS + k, d << 1 | (s >> 3 & 1));
        }
    }

    pub fn moldorm_draw(&mut self, k: usize) {
        const X: [i8; 16] = [11, 10, 9, 6, 3, 0, -2, -3, -4, -3, -2, 1, 4, 7, 9, 10];
        const Y: [i8; 16] = [4, 6, 9, 10, 11, 10, 9, 6, 3, 0, -2, -3, -4, -3, -2, 1];
        const CH: [u8; 3] = [0x5d, 0x62, 0x60];
        const XY: [i8; 3] = [4, 0, 0];
        const BIG: [u8; 3] = [0, 2, 2];
        const OFF: [u8; 3] = [21, 26, 0];
        let Some((_, _, flags)) = self.runtime.prep_oam_or_double_ret(self.ram, k) else {
            return;
        };
        // The compact runtime return loses the high coordinate bits. C's
        // SetOamHelper0 needs them when the head straddles the screen edge.
        let x = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_X_LO + k),
            self.ram.read_u8(SPRITE_X_HI + k),
        ])
        .wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2));
        let y = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_Y_LO + k),
            self.ram.read_u8(SPRITE_Y_HI + k),
        ])
        .wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2))
        .wrapping_sub(u16::from(self.ram.read_u8(SPRITE_Z + k)));
        let oam_entry = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let mut base = self.ram.read_u8(SPRITE_D + k).wrapping_sub(1);
        for piece in 0..2 {
            crate::oam::set_helper0(
                self.ram,
                oam_entry + piece,
                x.wrapping_add_signed(i16::from(X[(base & 15) as usize])),
                y.wrapping_add_signed(i16::from(Y[(base & 15) as usize])),
                0x4d,
                flags,
                0,
            );
            base = base.wrapping_add(2);
        }
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(8));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(2),
        );
        let j = (self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 0x1f) as usize + k * 32;
        self.ram
            .write_u8(MOLDORM_X_LO + j, self.ram.read_u8(SPRITE_X_LO + k));
        self.ram
            .write_u8(MOLDORM_X_HI + j, self.ram.read_u8(SPRITE_X_HI + k));
        self.ram
            .write_u8(MOLDORM_Y_LO + j, self.ram.read_u8(SPRITE_Y_LO + k));
        self.ram
            .write_u8(MOLDORM_Y_HI + j, self.ram.read_u8(SPRITE_Y_HI + k));
        for (piece, i) in (0..3).rev().enumerate() {
            let j = (self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(OFF[i]) & 0x1f) as usize
                + k * 32;
            let xx = u16::from_le_bytes([
                self.ram.read_u8(MOLDORM_X_LO + j),
                self.ram.read_u8(MOLDORM_X_HI + j),
            ])
            .wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2))
            .wrapping_add_signed(XY[i] as i16);
            let yy = u16::from_le_bytes([
                self.ram.read_u8(MOLDORM_Y_LO + j),
                self.ram.read_u8(MOLDORM_Y_HI + j),
            ])
            .wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2))
            .wrapping_add_signed(XY[i] as i16);
            crate::oam::set_helper0(
                self.ram,
                oam_entry + 2 + piece,
                xx,
                yy,
                CH[i],
                flags,
                BIG[i],
            );
        }
    }

    pub fn giant_moldorm_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.runtime.prep_oam_or_double_ret_wide(self.ram, k) else {
            return;
        };
        let info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.ram.write_u8(SPRITE_OAM_FLAGS + k, 11);
        self.sprite_draw_moldorm_eyeballs(k, &info);
        self.ram
            .write_u16(OAM_CUR_PTR, self.ram.read_u16(OAM_CUR_PTR).wrapping_add(8));
        self.ram.write_u16(
            OAM_EXT_CUR_PTR,
            self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(2),
        );
        let j = (self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 0x7f) as usize;
        self.ram
            .write_u8(MOLDORM_X_LO + j, self.ram.read_u8(SPRITE_X_LO + k));
        self.ram
            .write_u8(MOLDORM_X_HI + j, self.ram.read_u8(SPRITE_X_HI + k));
        self.ram
            .write_u8(MOLDORM_Y_LO + j, self.ram.read_u8(SPRITE_Y_LO + k));
        self.ram
            .write_u8(MOLDORM_Y_HI + j, self.ram.read_u8(SPRITE_Y_HI + k));
        self.sprite_draw_moldorm_head(k);
        if self.ram.read_u8(SPRITE_B + k) < 4 {
            self.runtime.giant_moldorm_draw_segment_ab(self.ram, k, 16);
            if self.ram.read_u8(SPRITE_B + k) < 3 {
                self.runtime.giant_moldorm_draw_segment_ab(self.ram, k, 28);
                if self.ram.read_u8(SPRITE_B + k) < 2 {
                    self.sprite_draw_moldorm_segment_c(k);
                    if self.ram.read_u8(SPRITE_B + k) == 0 {
                        self.moldorm_handle_tail(k);
                    }
                }
            }
        }
        self.giant_moldorm_incremental_segment_explosion(k);
        self.runtime.get_16bit_coords(self.ram, k);
    }

    pub fn sprite_cutscene_agahnim_zelda(&mut self, k: usize) {
        const D: [DrawPiece; 4] = [
            (-4, 0, 0x0103, 2),
            (4, 0, 0x0104, 2),
            (-4, 0, 0x0100, 2),
            (4, 0, 0x0101, 2),
        ];
        let j = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
        if j != 0 {
            self.sprite_draw_altar_zelda_warp(k);
            if j == 1 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            if j < 12 {
                return;
            }
        }
        self.runtime.oam_allocate_region_a(self.ram, 8);
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
        let info = PrepOamCoordsRet {
            x: self.ram.read_u16(CUR_SPRITE_X),
            y: self.ram.read_u16(CUR_SPRITE_Y),
            r4: 0,
            flags: self.ram.read_u8(SPRITE_OAM_FLAGS + k),
        };
        self.altar_zelda_draw_body(k, &info);
    }

    pub fn altar_zelda_draw_body(&mut self, k: usize, info: &PrepOamCoordsRet) {
        const XO: [u8; 16] = [4, 4, 3, 3, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        self.runtime.oam_allocate_region_a(self.ram, 8);
        let z = self.ram.read_u8(SPRITE_Z + k).min(31);
        let x = XO[(z >> 1) as usize];
        let y = self
            .sprite_y(k)
            .wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2));
        let entry = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        crate::oam::set_helper0(
            self.ram,
            entry,
            info.x.wrapping_add(x as u16),
            y.wrapping_add(7),
            0x6c,
            0x24,
            2,
        );
        crate::oam::set_helper0(
            self.ram,
            entry + 1,
            info.x.wrapping_sub(x as u16),
            y.wrapping_add(7),
            0x6c,
            0x24,
            2,
        );
    }

    pub fn sprite_draw_altar_zelda_warp(&mut self, k: usize) {
        const D: [DrawPiece; 10] = [
            (4, 4, 0x0480, 0),
            (4, 4, 0x0480, 0),
            (4, 4, 0x04b7, 0),
            (4, 4, 0x04b7, 0),
            (-6, 0, 0x0524, 2),
            (6, 0, 0x4524, 2),
            (-8, 0, 0x0524, 2),
            (8, 0, 0x4524, 2),
            (0, 0, 0x05c6, 2),
            (0, 0, 0x05c6, 2),
        ];
        self.runtime.oam_allocate_region_a(self.ram, 8);
        let i = (self.ram.read_u8(SPRITE_DELAY_MAIN + k) >> 2) as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
    }

    pub fn helmasaur_king_check_mask_damage_from_hammer(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_C + k) >= 3
            || self.ram.read_u8(LINK_ITEM_IN_HAND) & 10 == 0
            || self.ram.read_u8(PLAYER_OAM_Y_OFFSET) == 0x80
        {
            return;
        }
        if !self.runtime.helmasaur_hammer_overlap(self.ram, k) {
            return;
        }
        self.ram.write_u8(
            SPRITE_HEALTH + k,
            self.ram.read_u8(SPRITE_HEALTH + k).wrapping_sub(1),
        );
        self.ram.write_u8(SOUND_EFFECT_2, 0x21);
        let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 0x30);
        self.ram.write_u8(LINK_ACTUAL_VEL_Y, y);
        self.ram.write_u8(LINK_ACTUAL_VEL_X, x);
        self.ram.write_u8(LINK_INCAPACITATED_TIMER, 8);
        if self.ram.read_u8(REPULSESPARK_TIMER) == 0 {
            self.ram.write_u8(REPULSESPARK_X_LO, y);
            self.ram.write_u8(REPULSESPARK_Y_LO, x);
            self.ram.write_u8(REPULSESPARK_TIMER, 5);
        }
        self.runtime.sfx2_with_pan(self.ram, k, 5);
    }

    pub fn king_helmasaur_check_bomb_damage(&mut self, k: usize, j: usize) {
        if self.runtime.helmasaur_bomb_overlap(self.ram, k, j) {
            self.ram.write_u8(
                ANCILLA_X_VEL + j,
                0u8.wrapping_sub(self.ram.read_u8(ANCILLA_X_VEL + j)),
            );
            self.ram.write_u8(
                ANCILLA_Y_VEL + j,
                ((0i8.wrapping_sub(self.ram.read_u8(ANCILLA_Y_VEL + j) as i8)) >> 1) as u8,
            );
            self.ram.write_u8(SPRITE_DELAY_AUX4 + k, 32);
            self.ram.write_u8(REPULSESPARK_TIMER, 5);
            self.ram
                .write_u8(REPULSESPARK_X_LO, self.ram.read_u8(ANCILLA_X_LO + j));
            self.ram.write_u8(
                REPULSESPARK_Y_LO,
                self.ram
                    .read_u8(ANCILLA_Y_LO + j)
                    .wrapping_sub(self.ram.read_u8(ANCILLA_Z + j)),
            );
            self.ram.write_u8(SOUND_EFFECT_1, 5);
        }
    }

    pub fn sprite_trinexx_final_phase(&mut self, k: usize) {
        const G3: [u8; 8] = [6, 7, 0, 1, 2, 3, 4, 5];
        const G: [u8; 8] = [7, 7, 1, 1, 3, 3, 5, 5];
        const XV: [i8; 4] = [0, -31, 0, 31];
        const YV: [i8; 4] = [31, 0, -31, 0];
        let j = (self.runtime.convert_velocity_to_angle(
            self.ram.read_u8(SPRITE_X_VEL + k),
            self.ram.read_u8(SPRITE_Y_VEL + k),
        ) >> 1) as usize;
        let gfx = G3[j];
        self.ram.write_u8(
            SPRITE_GRAPHICS + k,
            if self.ram.read_u8(SPRITE_DELAY_AUX1 + k) != 0 {
                G[gfx as usize]
            } else {
                gfx
            },
        );
        self.sprite_trinexx_d_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if (self.ram.read_u8(SPRITE_AI_STATE + k) as i8) < 0 {
            let t = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
            self.ram.write_u8(SPRITE_HIT_TIMER + k, t | 0xe0);
            if t == 0 {
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 12);
                if self.ram.read_u8(SPRITE_ANIM_CLOCK + k) == 0 {
                    self.ram.write_u8(SPRITE_HIT_TIMER + k, 255);
                    self.sprite_schedule_boss_for_death(k);
                } else {
                    self.ram.write_u8(
                        SPRITE_ANIM_CLOCK + k,
                        self.ram.read_u8(SPRITE_ANIM_CLOCK + k).wrapping_sub(1),
                    );
                    self.sprite_make_boss_explosion(k);
                }
            }
            return;
        }
        if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
            self.runtime.sfx3_with_pan(self.ram, k, 0x31);
        }
        let s = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(1);
        self.ram.write_u8(SPRITE_SUBTYPE2 + k, s);
        let q = (s & 0x7f) as usize;
        self.ram
            .write_u8(MOLDORM_X_LO + q, self.ram.read_u8(SPRITE_X_LO + k));
        self.ram
            .write_u8(MOLDORM_Y_LO + q, self.ram.read_u8(SPRITE_Y_LO + k));
        self.ram
            .write_u8(MOLDORM_X_HI + q, self.ram.read_u8(SPRITE_X_HI + k));
        self.ram
            .write_u8(MOLDORM_Y_HI + q, self.ram.read_u8(SPRITE_Y_HI + k));
        if self.ram.read_u8(SPRITE_F + k) == 14 {
            self.ram.write_u8(SPRITE_F + k, 8);
            if self.ram.read_u8(SPRITE_AI_STATE + k) == 0 {
                self.ram.write_u8(SPRITE_AI_STATE + k, 2);
            }
        }
        self.runtime.move_xy(self.ram, k);
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 0 {
            let a = self.ram.read_u8(SPRITE_A + k).wrapping_sub(1);
            self.ram.write_u8(SPRITE_A + k, a);
            if a == 0 {
                self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 192);
            }
            self.runtime.get_16bit_coords(self.ram, k);
            if self.runtime.check_tile_collision(self.ram, k) {
                self.ram.write_u8(
                    SPRITE_D + k,
                    self.ram.read_u8(SPRITE_D + k).wrapping_add(1) & 3,
                );
                self.ram.write_u8(SPRITE_DELAY_AUX1 + k, 8);
            }
            let d = self.ram.read_u8(SPRITE_D + k) as usize;
            self.ram.write_u8(SPRITE_X_VEL + k, XV[d] as u8);
            self.ram.write_u8(SPRITE_Y_VEL + k, YV[d] as u8);
        } else if self.ram.read_u8(SPRITE_AI_STATE + k) == 1
            && self.ram.read_u8(FRAME_COUNTER) & 1 == 0
        {
            let (x, y) = self.runtime.project_speed_towards_link(self.ram, k, 31);
            self.runtime.approach_target_speed(self.ram, k, x, y);
        }
    }
    pub fn sprite_trinexx_d_draw(&mut self, k: usize) {
        const HP: [u8; 24] = [
            8, 12, 16, 24, 32, 40, 48, 52, 56, 60, 64, 68, 72, 76, 80, 84, 88, 92, 96, 100, 104,
            108, 112, 116,
        ];
        const G: [u8; 24] = [
            2, 2, 2, 3, 3, 3, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const OO: [u16; 24] = [
            16, 4, 4, 4, 16, 16, 16, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        ];
        self.ram.write_u8(
            SPRITE_OBJ_PRIO + k,
            self.ram.read_u8(SPRITE_OBJ_PRIO + k) | 0x30,
        );
        let info = PrepOamCoordsRet {
            x: self.ram.read_u16(CUR_SPRITE_X),
            y: self.ram.read_u16(CUR_SPRITE_Y),
            r4: 0,
            flags: self.ram.read_u8(SPRITE_OAM_FLAGS + k),
        };
        self.sprite_draw_trinexx_rock_head(k, &info);
        for i in 0..self.ram.read_u8(SPRITE_ANIM_CLOCK + k) as usize {
            let j = (self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_sub(HP[i]) & 0x7f) as usize;
            self.ram.write_u16(
                CUR_SPRITE_X,
                u16::from_le_bytes([
                    self.ram.read_u8(MOLDORM_X_LO + j),
                    self.ram.read_u8(MOLDORM_X_HI + j),
                ]),
            );
            self.ram.write_u16(
                CUR_SPRITE_Y,
                u16::from_le_bytes([
                    self.ram.read_u8(MOLDORM_Y_LO + j),
                    self.ram.read_u8(MOLDORM_Y_HI + j),
                ]),
            );
            if self
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
                && (self.ram.read_u8(SPRITE_AI_STATE + k) as i8) >= 0
                && (self.ram.read_u8(COUNTDOWN_FOR_BLINK)
                    | self.ram.read_u8(LINK_DISABLE_SPRITE_DAMAGE)
                    | self.ram.read_u8(SUBMODULE_INDEX)
                    | self.ram.read_u8(FLAG_UNK1))
                    == 0
            {
                self.ram.write_u8(LINK_AUXILIARY_STATE, 1);
                self.ram.write_u8(LINK_GIVE_DAMAGE, 8);
                self.ram.write_u8(LINK_INCAPACITATED_TIMER, 16);
                self.ram
                    .write_u8(LINK_ACTUAL_VEL_X, self.ram.read_u8(LINK_ACTUAL_VEL_X) ^ 255);
                self.ram
                    .write_u8(LINK_ACTUAL_VEL_Y, self.ram.read_u8(LINK_ACTUAL_VEL_Y) ^ 255);
            }
            self.ram.write_u16(
                OAM_CUR_PTR,
                self.ram.read_u16(OAM_CUR_PTR).wrapping_add(OO[i]),
            );
            self.ram.write_u16(
                OAM_EXT_CUR_PTR,
                self.ram.read_u16(OAM_EXT_CUR_PTR).wrapping_add(OO[i] >> 2),
            );
            self.ram.write_u8(SPRITE_OAM_FLAGS + k, 1);
            if i == 4 && self.ram.read_u8(SPRITE_AI_STATE + k) != 0 {
                self.sprite_trinexx_check_damage_to_flashing_segment(k);
                self.ram.write_u8(
                    SPRITE_OAM_FLAGS + k,
                    (self.ram.read_u8(SPRITE_SUBTYPE2 + k) & 6)
                        ^ self.ram.read_u8(SPRITE_OAM_FLAGS + k),
                );
            }
            self.ram.write_u8(SPRITE_GRAPHICS + k, G[i]);
            if G[i] != 3 {
                self.runtime.draw_single_large(self.ram, k);
            } else {
                self.ram.write_u8(SPRITE_GRAPHICS + k, 8);
                self.sprite_draw_trinexx_rock_head(k, &info);
            }
        }
        self.ram
            .write_u8(BYTE_7E0FB6, self.ram.read_u8(SPRITE_ANIM_CLOCK + k));
    }
    pub fn sprite_trinexx_check_damage_to_flashing_segment(&mut self, k: usize) {
        let x = self.sprite_x(k);
        let y = self.sprite_y(k);
        self.set_sprite_x(k, self.ram.read_u16(CUR_SPRITE_X));
        self.set_sprite_y(k, self.ram.read_u16(CUR_SPRITE_Y));
        self.ram.write_u8(SPRITE_DEFL_BITS + k, 0x80);
        self.ram.write_u8(SPRITE_FLAGS3 + k, 0);
        let _ = self.runtime.check_damage_from_link(self.ram, k);
        self.ram.write_u8(SPRITE_DEFL_BITS + k, 0x84);
        self.ram.write_u8(SPRITE_FLAGS3 + k, 0x40);
        self.set_sprite_x(k, x);
        self.set_sprite_y(k, y);
    }
    pub fn sprite_cb_trinexx_rock_head(&mut self, k: usize) {
        if self.ram.read_u8(OVERLORD_X_HI) != 0 {
            self.sprite_trinexx_final_phase(k);
            return;
        }
        self.ram.write_u8(TM_COPY, 0x17);
        self.ram.write_u8(TS_COPY, 0);
        self.sprite_draw_trinexx_rock_head_and_body(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if (self.ram.read_u8(SPRITE_AI_STATE + k) as i8) < 0 {
            let a = self.ram.read_u8(SPRITE_AI_STATE + k);
            self.ram.write_u8(FLAG_BLOCK_LINK_MENU, a);
            let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
            if d == 0 {
                self.ram.write_u8(OVERLORD_X_HI, 1);
                self.sprite_initialized_segmented(k);
                self.ram.write_u8(SPRITE_SUBTYPE2 + k, 0);
                self.ram.write_u8(SPRITE_HEAD_DIR + k, 0);
                self.ram.write_u8(
                    SPRITE_FLAGS3 + k,
                    self.ram.read_u8(SPRITE_FLAGS3 + k) & !0x40,
                );
                self.ram.write_u8(SPRITE_DEFL_BITS + k, 0x80);
                self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                self.ram.write_u8(SPRITE_D + k, 0);
                self.ram.write_u8(SPRITE_ANIM_CLOCK + k, 16);
                self.ram.write_u8(SPRITE_X_VEL + k, 0);
                self.ram.write_u8(SPRITE_Y_VEL + k, 0);
                self.ram.write_u8(SPRITE_A + k, 128);
                self.ram.write_u8(DUNG_FLOOR_Y_VEL + 1, 255);
            } else if d < 0xe0 {
                if d & 3 == 0 {
                    self.runtime.sfx2_with_pan(self.ram, k, 0xc);
                }
                if d & 1 == 0 {
                    const O: [i8; 8] = [0, 8, 16, 24, -24, -16, -8, 0];
                    let rx = (self.runtime.random_number(self.ram) & 7) as usize;
                    let ry = (self.runtime.random_number(self.ram) & 7) as usize;
                    self.ram.write_u16(
                        CUR_SPRITE_X,
                        self.sprite_x(k).wrapping_add_signed(O[rx] as i16),
                    );
                    self.ram.write_u16(
                        CUR_SPRITE_Y,
                        self.sprite_y(k)
                            .wrapping_add_signed(O[ry] as i16)
                            .wrapping_sub(8),
                    );
                    self.sprite_make_boss_death_explosion_no_sound(k);
                }
                self.ram.write_u8(SPRITE_HEAD_DIR + k, 255);
            } else if d < 255 {
                if d & 3 == 0 {
                    self.ram.write_u16(DUNG_FLOOR_Y_VEL, u16::MAX);
                    self.ram.write_u8(DUNG_HDR_COLLISION_2_MIRROR, 1);
                }
                self.ram.write_u8(SPRITE_Y_VEL + k, 248);
                self.runtime.move_y(self.ram, k);
                self.trinexx_cache_position(k);
                self.ram.write_u8(
                    SPRITE_C + k,
                    self.ram.read_u8(SPRITE_Y_LO + k).wrapping_sub(12),
                );
                self.ram.write_u16(
                    OVERLORD_X_LO + 7,
                    self.ram.read_u16(OVERLORD_X_LO + 7).wrapping_add(2),
                );
            }
            return;
        }
        if (self.ram.read_u8(SPRITE_STATE + 1) | self.ram.read_u8(SPRITE_STATE + 2)) == 0
            && self.ram.read_u8(SPRITE_AI_STATE + k) < 2
        {
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
            self.ram.write_u8(SPRITE_AI_STATE + k, 255);
            self.ram.write_u8(SOUND_EFFECT_2, 0x22);
            return;
        }
        self.trinexx_wag_tail(k);
        self.trinexx_handle_shell_collision(k);
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        if self.ram.read_u8(FRAME_COUNTER) & 63 == 0 {
            let (a, b) = self.runtime.is_right_of_link(self.ram, k);
            self.ram.write_u8(
                SPRITE_GRAPHICS + k,
                if (b as u8).wrapping_add(24) < 48 {
                    0
                } else if a != 0 {
                    1
                } else {
                    7
                },
            );
        }
        if self.ram.read_u8(OVERLORD_X_LO + 6) != 0 {
            if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
                self.ram.write_u8(
                    OVERLORD_X_LO + 6,
                    self.ram.read_u8(OVERLORD_X_LO + 6).wrapping_sub(1),
                );
            }
            return;
        }
        if (self.ram.read_u8(SPRITE_STATE + 1) != 0 && self.ram.read_u8(SPRITE_AI_STATE + 1) == 3)
            || (self.ram.read_u8(SPRITE_STATE + 2) != 0
                && self.ram.read_u8(SPRITE_AI_STATE + 2) == 3)
        {
            return;
        }
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    let j = self.runtime.random_number(self.ram) & 3;
                    if self.ram.read_u8(SPRITE_SUBTYPE + k) & 0x7f == j {
                        return;
                    }
                    let a = self.ram.read_u8(SPRITE_ANIM_CLOCK + k).wrapping_add(1);
                    self.ram.write_u8(SPRITE_ANIM_CLOCK + k, a);
                    if a == 2 {
                        self.ram.write_u8(SPRITE_ANIM_CLOCK + k, 0);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 80);
                        return;
                    }
                    self.ram
                        .write_u8(OVERLORD_X_LO, [0x60, 0x78, 0x78, 0x90][j as usize]);
                    self.ram
                        .write_u8(OVERLORD_X_LO + 1, [0x80, 0x70, 0x60, 0x80][j as usize]);
                    let r = self.runtime.random_number(self.ram);
                    self.ram
                        .write_u8(SPRITE_SUBTYPE + k, j + u8::from(r & 3 == 0) * 0x80);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                }
            }
            1 => {
                if self.ram.read_u8(SPRITE_SUBTYPE + k) == 0xff
                    && (self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0
                        || self.runtime.is_below_link(self.ram, k) == 0)
                {
                    self.ram.write_u8(SPRITE_SUBTYPE + k, 0);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
                } else {
                    let x = (self.ram.read_u8(SPRITE_X_HI + k) as u16) << 8
                        | self.ram.read_u8(OVERLORD_X_LO) as u16;
                    let y = (self.ram.read_u8(SPRITE_Y_HI + k) as u16) << 8
                        | self.ram.read_u8(OVERLORD_X_LO + 1) as u16;
                    let speed = if (self.ram.read_u8(SPRITE_SUBTYPE + k) as i8) < 0 {
                        16
                    } else {
                        8
                    };
                    let (xv, yv) = self
                        .runtime
                        .project_speed_towards_location(self.ram, k, x, y, speed);
                    self.ram.write_u8(SPRITE_X_VEL + k, xv);
                    self.ram.write_u8(SPRITE_Y_VEL + k, yv);
                    let bx = self.ram.read_u8(SPRITE_X_LO + k);
                    let by = self.ram.read_u8(SPRITE_Y_LO + k);
                    self.runtime.move_xy(self.ram, k);
                    self.ram.write_u16(
                        DUNG_FLOOR_Y_VEL,
                        (by.wrapping_sub(self.ram.read_u8(SPRITE_Y_LO + k)) as i8 as i16) as u16,
                    );
                    self.ram.write_u16(
                        DUNG_FLOOR_X_VEL,
                        (bx.wrapping_sub(self.ram.read_u8(SPRITE_X_LO + k)) as i8 as i16) as u16,
                    );
                    self.ram.write_u8(DUNG_HDR_COLLISION_2_MIRROR, 1);
                    self.trinexx_cache_position(k);
                    self.ram.write_u8(
                        SPRITE_C + k,
                        self.ram.read_u8(SPRITE_Y_LO + k).wrapping_sub(12),
                    );
                    if self
                        .ram
                        .read_u8(OVERLORD_X_LO)
                        .wrapping_sub(self.ram.read_u8(SPRITE_X_LO + k))
                        .wrapping_add(2)
                        < 4
                        && self
                            .ram
                            .read_u8(OVERLORD_X_LO + 1)
                            .wrapping_sub(self.ram.read_u8(SPRITE_Y_LO + k))
                            .wrapping_add(2)
                            < 4
                    {
                        self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
                    }
                }
                let steps = if (self.ram.read_u8(SPRITE_SUBTYPE + k) as i8) < 0 {
                    2
                } else {
                    1
                };
                for _ in 0..steps {
                    let delta = if (self.ram.read_u8(SPRITE_X_VEL + k) as i8) < 0 {
                        1
                    } else {
                        255
                    };
                    let subtype2 = self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(delta);
                    self.ram.write_u8(SPRITE_SUBTYPE2 + k, subtype2);
                    if subtype2 & 0xf == 0 {
                        self.runtime.sfx2_with_pan(self.ram, k, 0x21);
                    }
                }
            }
            2 => {
                self.trinexx_wag_tail(k);
                self.trinexx_wag_tail(k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                    self.runtime.apply_speed_towards_link(self.ram, k, 48);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 64);
                    self.ram.write_u8(SOUND_EFFECT_2, 0x26);
                }
            }
            3 => {
                self.runtime.move_xy(self.ram, k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.trinexx_restore_xy(k);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 48);
                } else if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0x20 {
                    self.ram.write_u8(
                        SPRITE_X_VEL + k,
                        0u8.wrapping_sub(self.ram.read_u8(SPRITE_X_VEL + k)),
                    );
                    self.ram.write_u8(
                        SPRITE_Y_VEL + k,
                        0u8.wrapping_sub(self.ram.read_u8(SPRITE_Y_VEL + k)),
                    );
                }
            }
            _ => {}
        }
    }
    pub fn trinexx_wag_tail(&mut self, _k: usize) {
        if self.ram.read_u8(OVERLORD_X_LO + 5) == 0 {
            let v = self.ram.read_u8(OVERLORD_X_LO + 4).wrapping_add(1);
            self.ram.write_u8(OVERLORD_X_LO + 4, v);
            if v & 3 == 0 {
                let j = self.ram.read_u8(OVERLORD_X_LO + 3) & 1;
                let p =
                    self.ram
                        .read_u8(OVERLORD_X_LO + 2)
                        .wrapping_add(if j != 0 { 255 } else { 1 });
                self.ram.write_u8(OVERLORD_X_LO + 2, p);
                if p == if j != 0 { 0 } else { 6 } {
                    self.ram.write_u8(
                        OVERLORD_X_LO + 3,
                        self.ram.read_u8(OVERLORD_X_LO + 3).wrapping_add(1),
                    );
                    self.ram.write_u8(OVERLORD_X_LO + 5, 8);
                }
            }
        } else {
            self.ram.write_u8(
                OVERLORD_X_LO + 5,
                self.ram.read_u8(OVERLORD_X_LO + 5).wrapping_sub(1),
            );
        }
    }
    pub fn trinexx_handle_shell_collision(&mut self, k: usize) {
        let x = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_A + k),
            self.ram.read_u8(SPRITE_B + k),
        ]);
        let y = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_C + k),
            self.ram.read_u8(SPRITE_G + k),
        ]);
        if x.wrapping_sub(self.ram.read_u16(LINK_X)).wrapping_add(40) < 80
            && y.wrapping_sub(self.ram.read_u16(LINK_Y)).wrapping_add(16) < 64
            && (self.ram.read_u8(COUNTDOWN_FOR_BLINK)
                | self.ram.read_u8(LINK_DISABLE_SPRITE_DAMAGE))
                == 0
        {
            self.ram.write_u8(LINK_AUXILIARY_STATE, 1);
            self.ram.write_u8(LINK_GIVE_DAMAGE, 8);
            self.ram.write_u8(LINK_INCAPACITATED_TIMER, 16);
            let (xv, yv) = self.runtime.project_speed_towards_link(self.ram, k, 32);
            self.ram.write_u8(LINK_ACTUAL_VEL_X, xv);
            self.ram.write_u8(LINK_ACTUAL_VEL_Y, yv);
        }
    }
    pub fn sprite_draw_trinexx_rock_head(&mut self, k: usize, _info: &PrepOamCoordsRet) {
        const D: [DrawPiece; 36] = [
            (-8, -8, 0x40c0, 2),
            (8, -8, 0x00c0, 2),
            (-8, 8, 0x40e0, 2),
            (8, 8, 0x00e0, 2),
            (-8, -8, 0x0000, 2),
            (8, -8, 0x0002, 2),
            (-8, 8, 0x0020, 2),
            (8, 8, 0x0022, 2),
            (-8, -8, 0x00c2, 2),
            (8, -8, 0x00c4, 2),
            (-8, 8, 0x80c2, 2),
            (8, 8, 0x80c4, 2),
            (-8, -8, 0x8020, 2),
            (8, -8, 0x8022, 2),
            (-8, 8, 0x8000, 2),
            (8, 8, 0x8002, 2),
            (-8, -8, 0xc0e0, 2),
            (8, -8, 0x80e0, 2),
            (-8, 8, 0xc0c0, 2),
            (8, 8, 0x80c0, 2),
            (-8, -8, 0xc022, 2),
            (8, -8, 0xc020, 2),
            (-8, 8, 0xc002, 2),
            (8, 8, 0xc000, 2),
            (-8, -8, 0x40c4, 2),
            (8, -8, 0x40c2, 2),
            (-8, 8, 0xc0c4, 2),
            (8, 8, 0xc0c2, 2),
            (-8, -8, 0x4002, 2),
            (8, -8, 0x4000, 2),
            (-8, 8, 0x4022, 2),
            (8, 8, 0x4020, 2),
            (-8, -8, 0x0026, 2),
            (8, -8, 0x4026, 2),
            (-8, 8, 0x8026, 2),
            (8, 8, 0xc026, 2),
        ];
        if (self.ram.read_u8(SPRITE_AI_STATE + k) as i8) >= 0 {
            self.ram.write_u8(
                SPRITE_OBJ_PRIO + k,
                self.ram.read_u8(SPRITE_OBJ_PRIO + k) | 0x30,
            );
        }
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
    }
    pub fn sprite_draw_trinexx_rock_head_and_body(&mut self, k: usize) {
        if (self.ram.read_u8(SPRITE_HEAD_DIR + k) as i8) < 0 {
            return;
        }
        self.sprite_draw_trinexx_rock_head(k, &PrepOamCoordsRet::default());
        // C keeps drawing the body even if Sprite_DrawMultiple culls the head.
        // The prep call still fills `info` in that case.
        let info = self.runtime.last_draw_info();
        let ix = info.x as u8;
        let iy = info.y as u8;
        let mut flags = info.flags;
        flags &= !0x10;
        if self.ram.read_u8(SPRITE_AI_STATE + k) == 3 {
            const M: [u8; 8] = [0xfc, 0xe0, 0xc0, 0xa0, 0x80, 0x60, 0x40, 0x20];
            let xb = self
                .ram
                .read_u8(SPRITE_A + k)
                .wrapping_sub(self.ram.read_u8(SPRITE_X_LO + k));
            let yb = self
                .ram
                .read_u8(SPRITE_C + k)
                .wrapping_sub(self.ram.read_u8(SPRITE_Y_LO + k));
            let base =
                (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4 + 4;
            for (entry, i) in (0..8).rev().enumerate() {
                let m = M[i];
                crate::oam::set_plain(
                    self.ram,
                    base + entry,
                    ix.wrapping_add(trinexx_mult(xb, m)),
                    iy.wrapping_add(trinexx_mult(yb, m)),
                    0x28,
                    flags,
                    2,
                );
            }
            self.ram.write_u8(BYTE_7E0FB6, 0x30);
        }
        self.ram.write_u16(OAM_CUR_PTR, 0x9f0);
        self.ram.write_u16(OAM_EXT_CUR_PTR, 0xa9c);
        const XO: [i8; 16] = [0, 2, 3, 4, 4, 4, 3, 2, 0, -2, -3, -4, -4, -4, -3, -2];
        const YO: [i8; 16] = [-4, -4, -3, -2, 0, 2, 3, 4, 4, 4, 3, 2, 0, -2, -3, -4];
        let xb = self
            .ram
            .read_u8(SPRITE_A + k)
            .wrapping_sub(self.ram.read_u8(BG2HOFS_COPY2));
        let yb = u16::from_le_bytes([
            self.ram.read_u8(SPRITE_C + k),
            self.ram.read_u8(SPRITE_Y_HI + k),
        ])
        .wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2));
        let xi = if self.ram.read_u8(SPRITE_X_VEL + k).wrapping_add(3) < 7 {
            0
        } else {
            self.ram.read_u8(SPRITE_SUBTYPE2 + k) >> 2
        };
        let yi = self.ram.read_u8(SPRITE_SUBTYPE2 + k) >> 2;
        for (entry, i) in (0..2).rev().enumerate() {
            let x = xb
                .wrapping_add(if i != 0 { 228 } else { 28 })
                .wrapping_add_signed(XO[((xi as usize) + (1 - i) * 8) & 15]);
            let y = (yb as u8)
                .wrapping_sub(8)
                .wrapping_add_signed(YO[((yi as usize) + i * 8) & 15]);
            let f = flags | if i == 0 { 0x40 } else { 0 };
            crate::oam::set_plain(self.ram, 124 + entry * 2, x, y, 0xc, f, 2);
            crate::oam::set_plain(self.ram, 125 + entry * 2, x, y.wrapping_add(16), 0x2a, f, 2);
        }
        const X: [i8; 35] = [
            0, 3, 9, 16, 24, 0, 2, 7, 13, 20, 0, 1, 4, 9, 15, 0, 0, 0, 0, 0, 0, -1, -4, -9, -15, 0,
            -2, -7, -13, -20, 0, -3, -9, -16, -24,
        ];
        const Y: [i8; 35] = [
            24, 32, 37, 37, 33, 24, 32, 39, 42, 44, 24, 32, 40, 47, 52, 24, 33, 42, 52, 61, 24, 32,
            40, 47, 52, 24, 32, 39, 42, 44, 24, 32, 37, 37, 33,
        ];
        const CH: [u8; 5] = [6, 0x28, 0x28, 0x2c, 0x2c];
        let g = self.ram.read_u8(OVERLORD_X_LO + 2) as usize;
        let tail = self.ram.read_u16(OVERLORD_X_LO + 7);
        for i in 0..5 {
            let j = g * 5 + i;
            crate::oam::set_helper0(
                self.ram,
                91 + i,
                u16::from(xb).wrapping_add_signed(i16::from(X[j])),
                yb.wrapping_sub(Y[j] as u16)
                    .wrapping_sub(0x20)
                    .wrapping_add(tail),
                CH[i],
                flags,
                2,
            );
        }
        self.ram.write_u8(TMP_COUNTER, 0xff);
        if self.ram.read_u8(0x11) != 0 {
            let sprite_x = self.sprite_x(k);
            let sprite_y = self.sprite_y(k);
            let scroll_x = sprite_x.wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2)) as u8;
            let scroll_y = sprite_y.wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2)) as u8;
            for entry in 0..4usize {
                let index = 124 + entry;
                let mut oam = crate::oam::read(self.ram, index);
                let x = sprite_x.wrapping_add((oam.x.wrapping_sub(scroll_x) as i8) as i16 as u16);
                let y = sprite_y.wrapping_add((oam.y.wrapping_sub(scroll_y) as i8) as i16 as u16);
                self.ram.write_u8(
                    0xa9c + entry,
                    2 + u8::from(x.wrapping_sub(self.ram.read_u16(BG2HOFS_COPY2)) >= 0x100),
                );
                if y.wrapping_add(0x10)
                    .wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2))
                    >= 0x100
                {
                    oam.y = 0xf0;
                    crate::oam::write(self.ram, index, oam);
                }
            }
        }
    }
    pub fn sprite_sidenexx(&mut self, k: usize) {
        let ty = self.ram.read_u8(SPRITE_TYPE + k);
        let xx = u16::from_le_bytes([self.ram.read_u8(SPRITE_A), self.ram.read_u8(SPRITE_B)])
            .wrapping_add_signed([-14i8, 13][ty.wrapping_sub(0xcc) as usize] as i16);
        let yy = u16::from_le_bytes([self.ram.read_u8(SPRITE_C), self.ram.read_u8(SPRITE_G)])
            .wrapping_sub(0x20);
        let [xl, xh] = xx.to_le_bytes();
        let [yl, yh] = yy.to_le_bytes();
        self.ram.write_u8(SPRITE_A + k, xl);
        self.ram.write_u8(SPRITE_B + k, xh);
        self.ram.write_u8(SPRITE_C + k, yl);
        self.ram.write_u8(SPRITE_G + k, yh);
        self.ram.write_u8(
            SPRITE_OBJ_PRIO + k,
            self.ram.read_u8(SPRITE_OBJ_PRIO + k) | 0x30,
        );
        self.trinexx_head_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if (self.ram.read_u8(SPRITE_AI_STATE + k) as i8) < 0 {
            self.ram.write_u8(
                SPRITE_IGNORE_PROJECTILE + k,
                self.ram.read_u8(SPRITE_AI_STATE + k),
            );
            self.sidenexx_explode(k);
            return;
        }
        if self.ram.read_u8(SPRITE_HIT_TIMER + k) != 0 && self.ram.read_u8(SPRITE_AI_STATE + k) != 4
        {
            self.ram.write_u8(SPRITE_HIT_TIMER + k, 0);
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 128);
            self.ram.write_u8(SPRITE_AI_STATE + k, 4);
            self.ram
                .write_u8(SPRITE_Z_VEL + k, self.ram.read_u8(SPRITE_OAM_FLAGS + k));
            self.ram.write_u8(SPRITE_OAM_FLAGS + k, 3);
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.ram.write_u8(
            SPRITE_DEFL_BITS + k,
            self.ram.read_u8(SPRITE_DEFL_BITS + k) | 4,
        );
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                self.ram.write_u8(
                    SPRITE_FLAGS3 + k,
                    self.ram.read_u8(SPRITE_FLAGS3 + k) | 0x40,
                );
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                    self.ram.write_u8(SPRITE_SUBTYPE2 + k, 9);
                    self.ram.write_u8(
                        SPRITE_FLAGS3 + k,
                        self.ram.read_u8(SPRITE_FLAGS3 + k) & !0x40,
                    );
                }
            }
            1 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    let i = (self.runtime.random_number(self.ram) & 7) + 1;
                    let old = self.ram.read_u8(SPRITE_D + k);
                    if i < 5 && old != i {
                        self.ram.write_u8(SPRITE_D + k, i);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                        if old == 1
                            && self.runtime.random_number(self.ram) & 1 == 0
                            && self.ram.read_u8(SPRITE_AI_STATE) < 2
                        {
                            self.ram.write_u8(SPRITE_GRAPHICS + k, 0);
                            self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 127);
                        }
                    }
                }
            }
            2 => {
                const T0: [u8; 45] = [
                    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x58, 0x64, 0x6a, 0x6f,
                    0x74, 0x7a, 0x7e, 0x80, 0x80, 0x39, 0x48, 0x52, 0x5c, 0x65, 0x73, 0x77, 0x7a,
                    0x80, 0x1e, 0x24, 0x29, 0x2e, 0x34, 0x3a, 0x44, 0x4d, 0x80, 0x0a, 0x11, 0x17,
                    0x1c, 0x22, 0x2a, 0x36, 0x3a, 0x80,
                ];
                const T1: [u8; 45] = [
                    0x30, 0x28, 0x23, 0x1e, 0x19, 0x13, 0x0c, 6, 0, 0x2f, 0x26, 0x21, 0x1d, 0x18,
                    0x12, 0x0c, 6, 0, 0x2f, 0x27, 0x22, 0x1d, 0x18, 0x12, 0x0c, 6, 0, 0x2f, 0x27,
                    0x22, 0x1d, 0x18, 0x12, 0x0c, 6, 0, 0x48, 0x3a, 0x32, 0x29, 0x22, 0x19, 0x10,
                    7, 0,
                ];
                let mut j = self.ram.read_u8(SPRITE_D + k) as usize * 9;
                let mut f = k * 9;
                let mut n = 0;
                for _ in (0..9).rev() {
                    for _ in 0..2 {
                        let v = self.ram.read_u8(ALT_SPRITE_TYPE + f);
                        if v != T0[j] {
                            self.ram.write_u8(
                                ALT_SPRITE_TYPE + f,
                                v.wrapping_add(if (v.wrapping_sub(T0[j]) as i8) < 0 {
                                    1
                                } else {
                                    255
                                }),
                            );
                            n += 1;
                        }
                    }
                    let v = self.ram.read_u8(ALT_SPRITE_Y_HI + f);
                    if v != T1[j] {
                        self.ram.write_u8(
                            ALT_SPRITE_Y_HI + f,
                            v.wrapping_add(if (v.wrapping_sub(T1[j]) as i8) < 0 {
                                1
                            } else {
                                255
                            }),
                        );
                        n += 1;
                    }
                    j += 1;
                    f += 1;
                }
                if n == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    let r = self.runtime.random_number(self.ram);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, r & 15);
                }
            }
            3 => {
                const M: [u8; 8] = [1, 1, 3, 3, 7, 15, 31, 31];
                let j = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if j == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                    return;
                }
                if j == 64 {
                    self.sidenexx_exhale_danger(k);
                }
                self.ram.write_u8(
                    SPRITE_SUBTYPE + k,
                    if j < 8 {
                        j
                    } else if j < 121 {
                        8
                    } else {
                        !j.wrapping_add(0x80)
                    },
                );
                if j >= 64 && self.ram.read_u8(FRAME_COUNTER) & M[((j - 64) >> 3) as usize] == 0 {
                    let x = (self.runtime.random_number(self.ram) & 15) as i8 - 3;
                    let y = (self.runtime.random_number(self.ram) & 15) as i8 + 12;
                    let g = self.runtime.garnish_spawn_sparkle(self.ram, k, x, y);
                    if ty == 0xcc {
                        self.ram.write_u8(GARNISH_TYPE + g, 0xe);
                    }
                }
            }
            4 => {
                self.ram.write_u8(
                    SPRITE_DEFL_BITS + k,
                    self.ram.read_u8(SPRITE_DEFL_BITS + k) & !4,
                );
                self.ram.write_u8(SPRITE_SUBTYPE + k, 0);
                let j = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if j == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                    self.ram
                        .write_u8(SPRITE_OAM_FLAGS + k, self.ram.read_u8(SPRITE_Z_VEL + k));
                    self.ram.write_u8(SPRITE_HIT_TIMER + k, 0);
                }
                if j >= 63 && j < 78 {
                    self.runtime
                        .trinexx_flash_shell_palette(self.ram, ty == 0xcd, true);
                } else if j < 15 {
                    self.runtime
                        .trinexx_flash_shell_palette(self.ram, ty == 0xcd, false);
                }
            }
            _ => {}
        }
    }
    pub fn sidenexx_exhale_danger(&mut self, k: usize) {
        let ty = self.ram.read_u8(SPRITE_TYPE + k);
        if ty == 0xcd {
            for i in 0..2 {
                if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xcd) {
                    self.runtime.set_spawned_coordinates(self.ram, j);
                    self.ram
                        .write_u8(SPRITE_C + j, if i != 0 { 1 } else { 254 });
                    self.runtime.sfx3_with_pan(self.ram, k, 0x19);
                    self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 1);
                    self.ram.write_u8(SPRITE_E + j, 1);
                    self.ram.write_u8(SPRITE_Y_VEL + j, 24);
                    self.ram.write_u8(SPRITE_FLAGS2 + j, 0);
                    self.ram.write_u8(SPRITE_FLAGS3 + j, 0x40);
                }
            }
            self.ram.write_u8(BYTE_7E0FB6, 1);
        } else if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, ty) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.runtime.sfx2_with_pan(self.ram, k, 0x2a);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 1);
            self.ram.write_u8(SPRITE_E + j, 1);
            self.ram.write_u8(SPRITE_Y_VEL + j, 24);
            self.ram.write_u8(SPRITE_FLAGS2 + j, 0);
            self.ram.write_u8(SPRITE_FLAGS3 + j, 0x40);
        }
    }
    pub fn sidenexx_explode(&mut self, k: usize) {
        if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 12);
            if self.ram.read_u8(SPRITE_SUBTYPE2 + k) == 1 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            self.ram.write_u8(
                SPRITE_SUBTYPE2 + k,
                self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_sub(1),
            );
            self.ram.write_u8(
                CUR_SPRITE_X,
                self.ram
                    .read_u8(CUR_SPRITE_X)
                    .wrapping_add(self.ram.read_u8(BG2HOFS_COPY2)),
            );
            self.ram.write_u8(
                CUR_SPRITE_Y,
                self.ram
                    .read_u8(CUR_SPRITE_Y)
                    .wrapping_add(self.ram.read_u8(BG2VOFS_COPY2)),
            );
            self.sprite_make_boss_explosion(k);
        }
    }
    pub fn trinexx_head_draw(&mut self, k: usize) {
        self.ram
            .write_u8(SPRITE_X_LO + k, self.ram.read_u8(SPRITE_A + k));
        self.ram
            .write_u8(SPRITE_X_HI + k, self.ram.read_u8(SPRITE_B + k));
        self.ram
            .write_u8(SPRITE_Y_LO + k, self.ram.read_u8(SPRITE_C + k));
        self.ram
            .write_u8(SPRITE_Y_HI + k, self.ram.read_u8(SPRITE_G + k));
        self.runtime.get_16bit_coords(self.ram, k);
        let Some((ix, iy, flags)) = self.runtime.prep_oam_or_double_ret(self.ram, k) else {
            return;
        };
        let oam_base = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        let mut oam_entry = oam_base;
        let mut i = 0u8;
        loop {
            let j = i as usize + k * 9;
            let r6 = if k != 2 {
                0x100u16
                    .wrapping_add((0u8.wrapping_sub(self.ram.read_u8(ALT_SPRITE_TYPE + j))) as u16)
            } else {
                self.ram.read_u8(ALT_SPRITE_TYPE + j) as u16
            };
            let amp = self.ram.read_u8(ALT_SPRITE_Y_HI + j);
            let dx = ganon_sin(r6, amp);
            let dy = ganon_sin(r6.wrapping_add(0x80), amp);
            self.ram.write_u8(0xfa8, dx as u8);
            self.ram.write_u8(0xfa9, dy as u8);
            if i == 0 {
                const X: [i8; 5] = [-8, 8, -8, 8, 0];
                const Y: [i8; 5] = [-8, -8, 8, 8, 2];
                const CH: [u8; 5] = [4, 4, 0x24, 0x24, 0xa];
                const FL: [u8; 5] = [0x40, 0, 0x40, 0, 0];
                for m in 0..5 {
                    let center_x = ix.wrapping_add(dx as u8);
                    let center_y = iy.wrapping_add(dy as u8);
                    self.ram.write_u8(CUR_SPRITE_X, center_x);
                    self.ram.write_u8(CUR_SPRITE_Y, center_y);
                    let x = center_x.wrapping_add_signed(X[m]);
                    let y = center_y.wrapping_add_signed(Y[m]).wrapping_add(if m == 4 {
                        self.ram.read_u8(SPRITE_SUBTYPE + k)
                    } else {
                        0
                    });
                    crate::oam::set_plain(self.ram, oam_entry, x, y, CH[m], flags | FL[m], 2);
                    oam_entry += 1;
                }
                self.set_sprite_x(
                    k,
                    u16::from_le_bytes([
                        self.ram.read_u8(SPRITE_A + k),
                        self.ram.read_u8(SPRITE_B + k),
                    ])
                    .wrapping_add_signed(dx as i16),
                );
                self.set_sprite_y(
                    k,
                    u16::from_le_bytes([
                        self.ram.read_u8(SPRITE_C + k),
                        self.ram.read_u8(SPRITE_G + k),
                    ])
                    .wrapping_add_signed(dy as i16),
                );
            } else {
                let x = ix.wrapping_add(dx as u8);
                let y = iy.wrapping_add(dy as u8);
                self.ram.write_u8(CUR_SPRITE_X, x);
                self.ram.write_u8(CUR_SPRITE_Y, y);
                crate::oam::set_plain(self.ram, oam_entry, x, y, 8, flags, 2);
                oam_entry += 1;
            }
            i = i.wrapping_add(1);
            if i == self.ram.read_u8(SPRITE_SUBTYPE2 + k) {
                break;
            }
        }
        self.ram.write_u8(TMP_COUNTER, i);
        self.ram
            .write_u8(BYTE_7E0FB6, i.wrapping_mul(4).wrapping_add(16));
        if self.ram.read_u8(0x11) != 0 {
            self.runtime.correct_oam_entries(self.ram, k, 4, 2);
        }
    }
    pub fn sprite_c1_cutscene_agahnim(&mut self, k: usize) {
        match self.ram.read_u8(SPRITE_A + k) {
            0 => self.cutscene_agahnim_agahnim(k),
            1 => self.sprite_cutscene_agahnim_zelda(k),
            _ => {}
        }
    }
    pub fn cutscene_agahnim_agahnim(&mut self, k: usize) {
        const G: [u8; 4] = [2, 0, 3, 0];
        let mut info = PrepOamCoordsRet {
            x: self.ram.read_u16(CUR_SPRITE_X),
            y: self.ram.read_u16(CUR_SPRITE_Y),
            r4: 0,
            flags: self.ram.read_u8(SPRITE_OAM_FLAGS + k),
        };
        if self.ram.read_u8(SPRITE_C + k) != 0 {
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) & 1 == 0 {
                self.chatty_agahnim_draw(k, &mut info);
            }
            return;
        }
        self.chatty_agahnim_draw(k, &mut info);
        self.sprite_draw_cutscene_agahnim_spell(k, &info);
        if self.ram.read_u8(SPRITE_PAUSE + k) != 0 {
            self.ram.write_u8(SPRITE_AI_STATE + k, 0);
            self.ram.write_u8(SPRITE_B + k, 0);
            self.ram.write_u8(SPRITE_GRAPHICS + k, 0);
            self.ram.write_u8(SPRITE_DELAY_MAIN + k, 64);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                    self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x13d);
                    self.runtime.show_message_minimal(self.ram);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                }
            }
            1 => {
                let j = self.ram.read_u8(SPRITE_B + k).wrapping_add(1);
                self.ram.write_u8(SPRITE_B + k, j);
                self.ram.write_u8(
                    SPRITE_GRAPHICS + k,
                    if self.ram.read_u8(SPRITE_Z + 15) < 16 {
                        G[(j >> 5 & 3) as usize]
                    } else {
                        1
                    },
                );
                if j & 15 == 0 {
                    self.ram.write_u8(SPRITE_GRAPHICS + 15, 1);
                    let z = self.ram.read_u8(SPRITE_Z + 15).wrapping_add(1);
                    self.ram.write_u8(SPRITE_Z + 15, z);
                    if z == 22 {
                        self.ram.write_u8(SOUND_EFFECT_2, 0x27);
                        self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                        self.ram.write_u8(SPRITE_DELAY_MAIN + k, 255);
                        self.ram.write_u8(SPRITE_SUBTYPE2 + k, 2);
                        self.ram.write_u8(SPRITE_SUBTYPE + k, 255);
                    }
                }
            }
            2 => {
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if d == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 80);
                } else if d == 120 {
                    self.ram.write_u8(INTRO_TIMES_PAL_FLASH, 120);
                } else if d < 128 && d & 3 == 0 {
                    self.ram.write_u8(SOUND_EFFECT_2, 0x2b);
                    if self.ram.read_u8(SPRITE_SUBTYPE2 + k) != 14 {
                        self.ram.write_u8(
                            SPRITE_SUBTYPE2 + k,
                            self.ram.read_u8(SPRITE_SUBTYPE2 + k).wrapping_add(4),
                        );
                    }
                }
            }
            3 => {
                let d = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
                if d != 0 {
                    if d & 3 == 0 && self.ram.read_u8(SPRITE_SUBTYPE + k) != 9 {
                        self.ram.write_u8(
                            SPRITE_SUBTYPE + k,
                            self.ram.read_u8(SPRITE_SUBTYPE + k).wrapping_add(2),
                        );
                    }
                } else {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + 15, 19);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 4);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 80);
                    self.ram.write_u8(SPRITE_SUBTYPE2 + k, 0);
                    self.ram.write_u8(SOUND_EFFECT_1, 0x33);
                }
            }
            4 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u16(DIALOGUE_MESSAGE_INDEX, 0x13e);
                    self.runtime.show_message_minimal(self.ram);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 5);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 2);
                }
            }
            5 => {
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 1 {
                    self.ram.write_u8(SOUND_EFFECT_2, 0x28);
                }
                self.ram.write_u8(SPRITE_Y_VEL + k, (-32i8) as u8);
                self.runtime.move_y(self.ram, k);
                if self.ram.read_u8(SPRITE_Y_LO + k) < 48 {
                    self.ram.write_u8(SPRITE_DELAY_AUX4 + k, 66);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 6);
                }
                self.sprite_agahnim_apply_motion_blur(k);
            }
            6 => {
                if self.ram.read_u8(SPRITE_DELAY_AUX4 + k) == 0 {
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                    self.ram.write_u8(SPRITE_STATE + k, 0);
                    self.runtime.manually_set_death_flag_uw(self.ram, k);
                    self.ram.write_u16(
                        DUNG_SAVEGAME_STATE_BITS,
                        self.ram.read_u16(DUNG_SAVEGAME_STATE_BITS) | 0x4000,
                    );
                }
            }
            _ => {}
        }
    }
    pub fn sprite_agahnim_apply_motion_blur(&mut self, k: usize) -> i32 {
        if self.ram.read_u8(FRAME_COUNTER) & 3 != 0 {
            return -1;
        }
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc1) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram
                .write_u8(SPRITE_GRAPHICS + j, self.ram.read_u8(SPRITE_GRAPHICS + k));
            self.ram.write_u8(SPRITE_DELAY_MAIN + j, 32);
            self.ram.write_u8(SPRITE_IGNORE_PROJECTILE + j, 32);
            self.ram.write_u8(SPRITE_C + j, 32);
            j as i32
        } else {
            -1
        }
    }
    pub fn chatty_agahnim_draw(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        const D: [DrawPiece; 16] = [
            (-8, -8, 0x0b82, 2),
            (8, -8, 0x4b82, 2),
            (-8, 8, 0x0ba2, 2),
            (8, 8, 0x4ba2, 2),
            (-8, -8, 0x0b80, 2),
            (8, -8, 0x4b80, 2),
            (-8, 8, 0x0ba0, 2),
            (8, 8, 0x4ba0, 2),
            (-8, -8, 0x0b80, 2),
            (8, -8, 0x4b82, 2),
            (-8, 8, 0x0ba0, 2),
            (8, 8, 0x4ba2, 2),
            (-8, -8, 0x0b82, 2),
            (8, -8, 0x4b80, 2),
            (-8, 8, 0x0ba2, 2),
            (8, 8, 0x4ba0, 2),
        ];
        if self.ram.read_u8(SPRITE_DELAY_AUX4 + k) & 1 != 0 {
            return;
        }
        if self.ram.read_u8(SPRITE_C + k) == 0 {
            self.ram.write_u16(OAM_CUR_PTR, 0x900);
            self.ram.write_u16(OAM_EXT_CUR_PTR, 0xa60);
        }
        let i = self.ram.read_u8(SPRITE_GRAPHICS + k) as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        self.runtime.draw_shadow_custom(self.ram, k, 18);
        *info = self.runtime.last_draw_info();
    }
    pub fn sprite_draw_cutscene_agahnim_spell(&mut self, k: usize, info: &PrepOamCoordsRet) {
        const D: [(i8, i8, u8, u8); 28] = [
            (-10, -16, 0xce, 0x06),
            (18, -16, 0xce, 0x06),
            (20, -13, 0x26, 0x06),
            (20, -5, 0x36, 0x06),
            (-12, -13, 0x26, 0x46),
            (-12, -5, 0x36, 0x46),
            (18, 0, 0x26, 0x06),
            (18, 8, 0x36, 0x06),
            (-10, 0, 0x26, 0x46),
            (-10, 8, 0x36, 0x46),
            (-8, 0, 0x22, 0x06),
            (8, 0, 0x22, 0x46),
            (-8, 16, 0x22, 0x86),
            (8, 16, 0x22, 0xc6),
            (-10, -16, 0xce, 0x04),
            (18, -16, 0xce, 0x04),
            (20, -13, 0x26, 0x44),
            (20, -5, 0x36, 0x44),
            (-12, -13, 0x26, 0x04),
            (-12, -5, 0x36, 0x04),
            (18, 0, 0x26, 0x44),
            (18, 8, 0x36, 0x44),
            (-10, 0, 0x26, 0x04),
            (-10, 8, 0x36, 0x04),
            (-8, 0, 0x20, 0x04),
            (8, 0, 0x20, 0x44),
            (-8, 16, 0x20, 0x84),
            (8, 16, 0x20, 0xc4),
        ];
        const B: [u8; 14] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2];
        self.runtime.oam_allocate_region_a(self.ram, 0x38);
        let base = if self.ram.read_u8(FRAME_COUNTER) & 2 == 0 {
            14
        } else {
            0
        };
        let sub = self.ram.read_u8(SPRITE_SUBTYPE2 + k);
        if sub == 0 {
            return;
        }
        let mut kn = sub.wrapping_sub(1);
        let end = self.ram.read_u8(SPRITE_SUBTYPE + k);
        let mut t = end.wrapping_add(1) as usize;
        let oam_entry = (usize::from(self.ram.read_u16(OAM_CUR_PTR)) - crate::oam::OAM_BUFFER) / 4;
        loop {
            let d = D[base + t];
            crate::oam::set_plain(
                self.ram,
                oam_entry + t,
                info.x.wrapping_add_signed(d.0 as i16) as u8,
                info.y.wrapping_add_signed(d.1 as i16).wrapping_sub(8) as u8,
                d.2,
                d.3 | 0x31,
                B[t],
            );
            kn = kn.wrapping_sub(1);
            if kn == end {
                break;
            }
            t += 1;
        }
    }
    pub fn sprite_92_helmasaur_king(&mut self, k: usize) {
        if (self.ram.read_u8(SPRITE_C + k) as i8) < 0 {
            if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 1 {
                self.ram.write_u8(SPRITE_STATE + k, 0);
            }
            self.runtime.draw_single_large(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            if (self.ram.read_u8(FRAME_COUNTER) & 7) | self.ram.read_u8(SPRITE_DELAY_AUX1 + k) == 0
            {
                self.ram.write_u8(
                    SPRITE_OAM_FLAGS + k,
                    self.ram.read_u8(SPRITE_OAM_FLAGS + k) ^ 0x40,
                );
            }
            self.runtime.move_xyz(self.ram, k);
            self.ram.write_u8(
                SPRITE_Z_VEL + k,
                self.ram.read_u8(SPRITE_Z_VEL + k).wrapping_sub(2),
            );
            if (self.ram.read_u8(SPRITE_Z + k) as i8) < 0 {
                self.ram.write_u8(SPRITE_Z + k, 0);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 12);
                self.ram.write_u8(SPRITE_Z_VEL + k, 24);
                self.ram.write_u8(SPRITE_GRAPHICS + k, 6);
            }
            return;
        }
        if self.ram.read_u8(SPRITE_C + k) < 3 {
            self.ram.write_u8(
                SPRITE_OBJ_PRIO + k,
                self.ram.read_u8(SPRITE_OBJ_PRIO + k) & !0xe,
            );
            self.ram.write_u8(SPRITE_FLAGS + k, 0xa);
        } else {
            self.ram.write_u8(SPRITE_FLAGS4 + k, 0x1f);
            self.ram.write_u8(SPRITE_FLAGS + k, 2);
        }
        self.helmasaur_king_draw(k);
        if self.ram.read_u8(SPRITE_STATE + k) == 6 {
            let t = self.ram.read_u8(SPRITE_DELAY_MAIN + k);
            if t == 0 {
                self.ram.write_u8(SPRITE_STATE + k, 4);
                self.ram.write_u8(SPRITE_A + k, 0);
                self.ram.write_u8(SPRITE_DELAY_MAIN + k, 224);
                return;
            }
            self.ram.write_u8(SPRITE_HIT_TIMER + k, t | 240);
            let j = self.ram.read_u8(OVERLORD_GEN2 + 3);
            if t < 128 && t & 7 == 0 && j != 16 {
                self.ram.write_u8(OVERLORD_GEN2 + 3, j + 1);
                self.ram.write_u16(
                    CUR_SPRITE_X,
                    self.sprite_x(k).wrapping_add_signed(
                        self.ram.read_u8(OVERLORD_X_LO + 5 + j as usize) as i8 as i16,
                    ),
                );
                self.ram.write_u16(
                    CUR_SPRITE_Y,
                    self.sprite_y(k).wrapping_add_signed(
                        self.ram.read_u8(OVERLORD_Y_LO + 5 + j as usize) as i8 as i16,
                    ),
                );
                self.sprite_make_boss_explosion(k);
            }
            return;
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        const TAB: [u8; 13] = [3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 1, 1, 0];
        let t = TAB[(self.ram.read_u8(SPRITE_HEALTH + k) >> 2) as usize];
        self.ram.write_u8(SPRITE_C + k, t);
        if t != self.ram.read_u8(SPRITE_E + k) {
            if t == 3 {
                self.ram.write_u8(SPRITE_HIT_TIMER + k, 0);
                self.helmasaur_king_explode_mask(k);
            } else {
                self.helmasaur_king_chip_away_at_mask(k);
            }
        }
        self.ram.write_u8(SPRITE_E + k, t);
        let _ = self.runtime.check_damage_from_link(self.ram, k);
        self.helmasaur_king_swing_tail(k);
        self.helmasaur_king_attempt_damage(k);
        self.helmasaur_king_check_mask_damage_from_hammer(k);
        let a1 = self.ram.read_u8(SPRITE_DELAY_AUX1 + k);
        let a2 = self.ram.read_u8(SPRITE_DELAY_AUX2 + k);
        if a1 == 0 && a2 != 0 {
            if a2 == 0x40 {
                self.helmasaur_king_spit_fireball(k);
                if t >= 3 && self.ram.read_u8(SPRITE_ANIM_CLOCK + k) == 0 {
                    self.ram.write_u8(SPRITE_ANIM_CLOCK + k, 1);
                    self.ram.write_u8(SPRITE_DELAY_AUX3 + k, 32);
                }
            }
            return;
        } else if a1 != 0 {
            if a1 == 96 && self.ram.read_u8(SPRITE_ANIM_CLOCK + k) == 0 {
                self.ram.write_u8(SPRITE_ANIM_CLOCK + k, 1);
                self.ram.write_u8(SPRITE_DELAY_AUX3 + k, 32);
            }
            return;
        }
        const XV: [i8; 8] = [-12, -12, -4, 0, 4, 12, 12, 0];
        const YV: [i8; 8] = [0, 4, 12, 12, 12, 4, 0, 12];
        match self.ram.read_u8(SPRITE_AI_STATE + k) {
            0 => {
                if (self.ram.read_u8(SPRITE_HIT_TIMER + k) != 0
                    || self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0)
                    && !self.helmasaur_king_maybe_fireball(k)
                {
                    let j = (self.runtime.random_number(self.ram) & 7) as usize;
                    self.ram.write_u8(SPRITE_X_VEL + k, XV[j] as u8);
                    self.ram.write_u8(SPRITE_Y_VEL + k, YV[j] as u8);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 64);
                    if t >= 3 {
                        self.ram
                            .write_u8(SPRITE_X_VEL + k, self.ram.read_u8(SPRITE_X_VEL + k) << 1);
                        self.ram
                            .write_u8(SPRITE_Y_VEL + k, self.ram.read_u8(SPRITE_Y_VEL + k) << 1);
                        self.ram.write_u8(
                            SPRITE_DELAY_MAIN + k,
                            self.ram.read_u8(SPRITE_DELAY_MAIN + k) >> 1,
                        );
                    }
                    self.ram.write_u8(SPRITE_AI_STATE + k, 1);
                }
            }
            1 => {
                self.helmasaur_king_handle_movement(k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 32);
                    self.ram.write_u8(SPRITE_AI_STATE + k, 2);
                }
            }
            2 => {
                if (self.ram.read_u8(SPRITE_HIT_TIMER + k) != 0
                    || self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0)
                    && !self.helmasaur_king_maybe_fireball(k)
                {
                    self.ram.write_u8(
                        SPRITE_DELAY_MAIN + k,
                        if self.ram.read_u8(SPRITE_E + k) >= 3 {
                            32
                        } else {
                            64
                        },
                    );
                    self.ram.write_u8(
                        SPRITE_X_VEL + k,
                        0u8.wrapping_sub(self.ram.read_u8(SPRITE_X_VEL + k)),
                    );
                    self.ram.write_u8(
                        SPRITE_Y_VEL + k,
                        0u8.wrapping_sub(self.ram.read_u8(SPRITE_Y_VEL + k)),
                    );
                    self.ram.write_u8(SPRITE_AI_STATE + k, 3);
                }
            }
            3 => {
                self.helmasaur_king_handle_movement(k);
                if self.ram.read_u8(SPRITE_DELAY_MAIN + k) == 0 {
                    self.ram.write_u8(SPRITE_AI_STATE + k, 0);
                    self.ram.write_u8(SPRITE_DELAY_MAIN + k, 64);
                }
            }
            _ => {}
        }
    }
}

fn ganon_sin(angle: u16, amplitude: u8) -> i8 {
    let a = SINUS_LOOKUP_TABLE[(angle & 0xff) as usize];
    let t = if a >= 256 {
        amplitude
    } else {
        let p = a * amplitude as u16;
        ((p >> 8) + ((p >> 7) & 1)) as u8
    };
    if angle & 0x100 != 0 {
        (0u8).wrapping_sub(t) as i8
    } else {
        t as i8
    }
}

fn trinexx_mult(a: u8, b: u8) -> u8 {
    let n = if (a as i8) < 0 {
        0u8.wrapping_sub(a)
    } else {
        a
    };
    let p = n as u16 * b as u16;
    let r = ((p >> 8) + ((p >> 7) & 1)) as u8;
    if (a as i8) < 0 {
        0u8.wrapping_sub(r)
    } else {
        r
    }
}

fn chain_chomp_one_mult(a: u16, b: u8) -> u16 {
    let lo = a as u8;
    let magnitude = if (lo as i8) < 0 {
        (0u8).wrapping_sub(lo)
    } else {
        lo
    };
    let product = ((magnitude as u16 * b as u16) >> 8) as u8;
    if (lo as i8) < 0 {
        !(product as u16)
    } else {
        product as u16
    }
}
