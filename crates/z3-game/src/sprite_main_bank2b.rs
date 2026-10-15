//! Translation of the final `sprite_main.c` bank (definitions beginning on
//! canonical source lines 20001..=25878).
//!
//! The game state remains in the canonical WRAM image. Calls owned by another
//! translation unit cross the small [`SpriteMainBank2bRuntime`] seam.

#![allow(dead_code)]
#![allow(
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::if_same_then_else
)]

use crate::sprite::SpriteHitBox;
use crate::sprite_state::SpriteField;
use crate::sprite_state::layout::{SPRITE_X_VEL, SPRITE_Y_VEL};
use crate::{oam, ram::Ram};

const FRAME_COUNTER: usize = 0x1a;
const PLAYER_IS_INDOORS: usize = 0x1b;
const LINK_Y: usize = 0x20;
const LINK_X: usize = 0x22;
const LINK_DIRECTION_LAST: usize = 0x26;
const LINK_DIRECTION_FACING: usize = 0x2f;
const LINK_Y_VEL: usize = 0x30;
const LINK_X_VEL: usize = 0x31;
const LINK_VISIBILITY_STATUS: usize = 0x4b;
const LINK_DIRECTION: usize = 0x67;
const LINK_ACTUAL_VEL_Y: usize = 0x27;
const LINK_ACTUAL_VEL_X: usize = 0x28;
const LINK_INCAPACITATED_TIMER: usize = 0x46;
const LINK_CAPE_MODE: usize = 0x55;
const JOYPAD1_H_LAST: usize = 0xf0;
const FILTERED_JOYPAD_L: usize = 0xf6;
const PLAYER_ON_SOMARIA_PLATFORM: usize = 0x2f5;
const LINK_HEARTS_FILLER: usize = 0xf372;
const LINK_BOTTLE_INFO: usize = 0xf35c;
const LINK_ITEM_BOTTLE_INDEX: usize = 0xf34f;
const LINK_SHIELD_TYPE: usize = 0xf35a;
const LINK_ITEM_BOMBS: usize = 0xf343;
const LINK_BOMB_UPGRADES: usize = 0xf370;
const LINK_ARROW_UPGRADES: usize = 0xf371;
const LINK_NUM_ARROWS: usize = 0xf377;
const LINK_BOMB_FILLER: usize = 0xf375;
const LINK_ITEM_MOON_PEARL: usize = 0xf357;
const LINK_RUPEES_GOAL: usize = 0xf360;
const LINK_HEALTH_CAPACITY: usize = 0xf36c;
const LINK_HEALTH_CURRENT: usize = 0xf36d;
const DRAG_PLAYER_X: usize = 0xb7c;
const DRAG_PLAYER_Y: usize = 0xb7e;
const BYTE_7E0FDD: usize = 0xfdd;
const BYTE_7E0FC6: usize = 0xfc6;
const SUBMODULE_INDEX: usize = 0x11;
const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;
const FLAG_IS_LINK_IMMOBILIZED: usize = 0x2e4;
const LINK_DISABLE_SPRITE_DAMAGE: usize = 0x37b;
const DUNGEON_ROOM_INDEX2: usize = 0x48e;
const TMP_COUNTER: usize = 0xfb5;
const CUR_PALACE_INDEX_X2: usize = 0x40c;
const FOLLOWER_INDICATOR: usize = 0xf3cc;
const GARNISH_ACTIVE: usize = 0xfb4;
const GARNISH_TYPE: usize = 0x1f800;
const GARNISH_Y_LO: usize = 0x1f81e;
const GARNISH_X_LO: usize = 0x1f83c;
const GARNISH_Y_HI: usize = 0x1f85a;
const GARNISH_X_HI: usize = 0x1f878;
const GARNISH_COUNTDOWN: usize = 0x1f90e;
const GARNISH_SPRITE: usize = 0x1f92c;
const GARNISH_FLOOR: usize = 0x1f968;
const GARNISH_OAM_FLAGS: usize = 0x1f9fe;
const CUR_SPRITE_X: usize = 0xfd8;
const CUR_SPRITE_Y: usize = 0xfda;
const IS_NMI_THREAD_ACTIVE: usize = 0x12a;
const DUNG_FLOOR_X_OFFS: usize = 0x422;
const DUNG_FLOOR_Y_OFFS: usize = 0x424;
const INTRO_DID_RUN_STEP: usize = 0x1f00;
const OVERWORLD_SCREEN_INDEX: usize = 0x8a;
const SUBSUBMODULE_INDEX: usize = 0xb0;
const LINK_TRIGGERED_BY_WHIRLPOOL: usize = 0x2db;
const LINK_PLAYER_HANDLER_STATE: usize = 0x5d;
const LAST_LIGHT_VS_DARK_WORLD: usize = 0x7b;
const LINK_Y_SAFE_LO: usize = 0x3e;
const LINK_X_SAFE_LO: usize = 0x3f;
const LINK_Y_SAFE_HI: usize = 0x40;
const LINK_X_SAFE_HI: usize = 0x41;
const PLAYER_NEAR_PIT_STATE: usize = 0x5b;
const DUNGEON_ROOM_INDEX: usize = 0xa0;
const ALT_SPRITE_SPAWNED_FLAG: usize = 0x1de0;
const TS_COPY: usize = 0x1d;
const LINK_ANIMATION_STEPS: usize = 0x2e;
const LINK_RECEIVEITEM_INDEX: usize = 0x2d8;
const LINK_POSE_FOR_ITEM: usize = 0x2da;
const CHOICE_IN_MULTISELECT_BOX: usize = 0x1ce8;
const POLY_CONFIG1: usize = 0x1f02;
const POLY_B: usize = 0x1f05;
const PALETTE_FILTER_COUNTDOWN: usize = 0xc007;
const LINK_HAS_CRYSTALS: usize = 0xf37a;
const SAVEGAME_MAP_ICONS_INDICATOR: usize = 0xf3c7;
const SRAM_PROGRESS_INDICATOR_3: usize = 0xf3c9;
const LINK_IS_ON_LOWER_LEVEL: usize = 0xee;
const LINK_SPEED_SETTING: usize = 0x5e;
const PLAYER_OAM_Y_OFFSET: usize = 0x44;
const LINK_ITEM_IN_HAND: usize = 0x301;
const LINK_POSITION_MODE: usize = 0x37a;
const HUD_CUR_ITEM: usize = 0x202;
const FILTERED_JOYPAD_H: usize = 0xf4;
const PLAYER_HANDLER_TIMER: usize = 0x300;
const SOUND_EFFECT_1: usize = 0x12e;
const MUSIC_CONTROL: usize = 0x12c;
const SOUND_EFFECT_2: usize = 0x12f;
const FLAG_BLOCK_LINK_MENU: usize = 0xffc;
const BYTE_7E0FF8: usize = 0xff8;
const CGADSUB_COPY: usize = 0x9a;
const TM_COPY: usize = 0x1c;
const LINK_IS_RUNNING: usize = 0x372;
const LINK_SWORD_TYPE: usize = 0xf359;
const SRAM_PROGRESS_INDICATOR: usize = 0xf3c5;
const WHICH_STARTING_POINT: usize = 0xf3c8;
const ITEM_RECEIPT_METHOD: usize = 0x2e9;
const BYTE_7E0FDE: usize = 0xfde;
const BUTTON_B_FRAMES: usize = 0x3c;
const LINK_DELAY_TIMER_SPIN_ATTACK: usize = 0x3d;
const TAGALONG_EVENT_FLAGS: usize = 0x2f2;
const BITMASK_OF_DRAGSTATE: usize = 0x48;
const TRIGGER_SPECIAL_ENTRANCE: usize = 0x4c6;
const OVERLORD_X_LO: usize = 0xb08;
const OVERLORD_X_HI: usize = 0xb10;
const OVERLORD_Y_HI: usize = 0xb20;
const OVERLORD_GEN2: usize = 0xb30;
const OVERLORD_FLOOR: usize = 0xb40;
const OVERLORD_Y_LO: usize = 0xb18;
const OVERLORD_GEN1: usize = 0xb28;
const OVERLORD_GEN3: usize = 0xb38;
const ANCILLA_Y_LO: usize = 0xbfa;
const ANCILLA_X_LO: usize = 0xc04;
const ANCILLA_Y_HI: usize = 0xc0e;
const ANCILLA_X_HI: usize = 0xc18;
const ANCILLA_TYPE: usize = 0xc4a;
const OVERWORLD_ENTRANCE_SEQUENCE_COUNTER: usize = 0xc8;
const BG2VOFS_COPY2: usize = 0xe8;
const TAGALONG_Y_LO: usize = 0x1a00;
const TAGALONG_Y_HI: usize = 0x1a14;
const TAGALONG_X_LO: usize = 0x1a28;
const TAGALONG_X_HI: usize = 0x1a3c;
const TAGALONG_LAYERBITS: usize = 0x1a64;
const IS_IN_DARK_WORLD: usize = 0xfff;
const FLAG_UNK1: usize = 0xfc1;
const DUNG_SAVEGAME_STATE_BITS: usize = 0x402;
const DUNG_HDR_COLLISION_2_MIRROR: usize = 0x428;
const MINIGAME_CREDITS: usize = 0x4c4;

pub type DrawPiece = (i8, i8, u16, u8);

pub trait SpriteMainBank2bRuntime {
    fn return_if_inactive(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn return_if_recoiling(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn return_if_paused(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn move_xy(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_x(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_z(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_xyz(&mut self, ram: &mut Ram<'_>, k: usize) {
        self.move_xy(ram, k);
        self.move_z(ram, k);
    }
    fn check_tile_collision(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn random(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
    fn project_speed(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _x: u16,
        _y: u16,
        _speed: u8,
    ) -> (i8, i8) {
        (0, 0)
    }
    fn check_damage_to_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_damage_same_layer(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_if_link_is_busy(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn check_damage_ignore_layer(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_damage_to_and_from_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_damage_from_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn check_if_room_is_clear(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn player_setup_action_hit_box(&mut self, _ram: &mut Ram<'_>, _hb: &mut SpriteHitBox) {}
    fn hit_boxes_overlap(&mut self, _hb: &SpriteHitBox) -> bool {
        false
    }
    fn setup_sprite_hit_box(&mut self, _ram: &mut Ram<'_>, _k: usize, _hb: &mut SpriteHitBox) {}
    fn give_damage(&mut self, _ram: &mut Ram<'_>, _k: usize, _damage: u8, _hit_timer: u8) {}
    fn player_bee_hone_in_on_target(&mut self, _ram: &mut Ram<'_>, _target: usize, _bee: usize) {}
    fn direction_to_face_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn is_below_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn is_right_of_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn spawn_sparkle(&mut self, _ram: &mut Ram<'_>, _k: usize, _x: i8, _y: i8) {}
    fn spawn_bee_sparkle(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn spawn_sparkle_limited(&mut self, ram: &mut Ram<'_>, k: usize, x: i8, y: i8) {
        self.spawn_sparkle(ram, k, x, y)
    }
    fn prep_oam(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn firebar_prep_oam(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<(u16, u16, u8)> {
        None
    }
    fn prep_oam_coords(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<(u16, u16, u8)> {
        None
    }
    fn spawn_poof_garnish(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn get_tile_attribute(&mut self, _ram: &mut Ram<'_>, _x: &mut u16, _y: u16) -> u8 {
        0
    }
    fn draw_multiple(&mut self, _ram: &mut Ram<'_>, _k: usize, _pieces: &[DrawPiece]) {}
    fn draw_multiple_player_deferred_with_shadow(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _pieces: &[DrawPiece],
    ) {
    }
    fn draw_single_large(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_single_small(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_distress(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_is_offscreen(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn guruguru_bar_sin(&mut self, _angle: u16, _radius: u8) -> i8 {
        0
    }
    fn convert_velocity_to_angle(&mut self, _x: i8, _y: i8) -> u8 {
        0
    }
    fn arrgi_sin(&mut self, _angle: u16, _radius: u8) -> i8 {
        0
    }
    fn attempt_damage_to_link_plus_recoil(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn allocate_oam_region_b(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn allocate_oam_region_c(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn draw_large_shadow2(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_shadow_custom(&mut self, _ram: &mut Ram<'_>, _k: usize, _y: u8) {}
    fn correct_oam_entries(&mut self, _ram: &mut Ram<'_>, _k: usize, _count: u8, _value: u8) {}
    fn behave_as_barrier(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn show_message(&mut self, _ram: &mut Ram<'_>, _message: u16) {}
    fn show_solicited_message(&mut self, _ram: &mut Ram<'_>, _k: usize, _message: u16) -> u16 {
        0
    }
    fn show_message_on_contact(&mut self, _ram: &mut Ram<'_>, _k: usize, _message: u16) -> u16 {
        0
    }
    fn queue_sfx2(&mut self, _ram: &mut Ram<'_>, _k: usize, _sfx: u8) {}
    fn queue_sfx3(&mut self, _ram: &mut Ram<'_>, _k: usize, _sfx: u8) {}
    fn link_receive_item(&mut self, _ram: &mut Ram<'_>, _item: u8) {}
    fn rapid_terminate_receive_item(&mut self, _ram: &mut Ram<'_>) {}
    fn link_handle_velocity(&mut self, _ram: &mut Ram<'_>) {}
    fn link_handle_moving_animation(&mut self, _ram: &mut Ram<'_>) {}
    fn handle_indoor_camera_and_doors(&mut self, _ram: &mut Ram<'_>) {}
    fn spawn_superficial_bomb_blast(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<usize> {
        None
    }
    fn pipe_validate_entry(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn link_reset_properties_a(&mut self, _ram: &mut Ram<'_>) {}
    fn halt_all_movement(&mut self, _ram: &mut Ram<'_>) {}
    fn load_follower_graphics(&mut self, _ram: &mut Ram<'_>) {}
    fn become_follower(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn track_body_to_head(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn set_enemy_damage(&mut self, _ram: &mut Ram<'_>, _index: usize, _value: u8) {}
    fn update_common_tile(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16, _tile: u8) {}
    fn garnish_alloc_low(&mut self, _ram: &mut Ram<'_>) -> Option<usize> {
        None
    }
    fn garnish_alloc(&mut self, _ram: &mut Ram<'_>) -> Option<usize> {
        None
    }
    fn garnish_alloc_with_limit(&mut self, ram: &mut Ram<'_>, limit: usize) -> Option<usize> {
        self.garnish_alloc(ram).filter(|slot| *slot <= limit)
    }
    fn garnish_alloc_overwrite_old(&mut self, _ram: &mut Ram<'_>) -> usize {
        0
    }
    fn spawn_dynamic(
        &mut self,
        _ram: &mut Ram<'_>,
        _parent: usize,
        _kind: u8,
    ) -> Option<(usize, u16, u16, u8)> {
        None
    }
    fn spawn_dynamic_ex(
        &mut self,
        ram: &mut Ram<'_>,
        parent: usize,
        kind: u8,
        _slots: usize,
    ) -> Option<(usize, u16, u16, u8)> {
        self.spawn_dynamic(ram, parent, kind)
    }
    fn place_weapon_tink(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn apply_speed_towards_link(&mut self, _ram: &mut Ram<'_>, _k: usize, _speed: u8) {}
    fn approach_target_speed(&mut self, _ram: &mut Ram<'_>, _k: usize, _x: i8, _y: i8) {}
    fn project_speed_towards_link(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _speed: u8,
    ) -> (i8, i8) {
        (0, 0)
    }
    fn link_cancel_dash(&mut self, _ram: &mut Ram<'_>) {}
    fn link_reset_swimming(&mut self, _ram: &mut Ram<'_>) {}
    fn agahnim_warp_shadow_filter(&mut self, _ram: &mut Ram<'_>, _index: usize) {}
    fn agahnim_apply_motion_blur(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<usize> {
        None
    }
    fn spawn_phantom_ganon(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bank2b_prepare_dungeon_exit_from_boss_fight(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_filter_sp5f(&mut self, _ram: &mut Ram<'_>) {}
    fn bank2b_stalfos_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bank2b_zazak_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bank2b_place_rupulse_spark(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bank2b_spawn_big_splash(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bank2b_old_mountain_man_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bank2b_cache_camera_properties(&mut self, _ram: &mut Ram<'_>) {}
    fn kiki_lying_in_wait(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_shadow(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_big_shadow(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_large_water_turbulence(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn nullify_hookshot_drag(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn repel_dash(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn kholdstare_shell_palette_filtering(&mut self, _ram: &mut Ram<'_>) {}
    fn freezor_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn manually_set_death_flag_uw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn get_16bit_coords(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn allocate_oam_defer_to_player(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn spawn_lightning(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn spike_trap_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn shopkeeper_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn maze_game_guy_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn babusu_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_antifairy(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn add_sword_charge_sparkle(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn flute_boy_ostrich_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn wizzbeam(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn wizzrobe_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn laser_beam_build_up_garnish(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn wallmaster_send_player_to_last_entrance(&mut self, _ram: &mut Ram<'_>) {}
    fn link_initialize(&mut self, _ram: &mut Ram<'_>) {}
    fn bee_put_in_bottle(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn mothula_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn thief_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn drinking_guy_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn goriya_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn crystal_maiden_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn kholdstare_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_94_tile(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn spawn_small_splash(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
}

pub struct SpriteMainBank2b<'a, 'r, R: SpriteMainBank2bRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
}

impl<R: SpriteMainBank2bRuntime> SpriteMainBank2b<'_, '_, R> {
    /// Dispatch the active sprite kinds owned by this final source bank.
    /// Returns false for kinds owned by an earlier/later translation bank.
    pub fn dispatch_active(&mut self, k: usize, kind: u8) -> bool {
        match kind {
            0x79 => self.sprite_79_bee(k),
            0x7a => self.sprite_7a_agahnim(k),
            0x7b => self.sprite_7b_agahnim_balls(k),
            0x7c => self.sprite_7c_green_stalfos(k),
            0x7d => self.sprite_7d_big_spike(k),
            0x7e | 0x7f => self.sprite_7e_firebar_clockwise(k),
            0x80 => self.sprite_80_firesnake(k),
            0x81 => self.sprite_81_hover(k),
            0x82 => self.sprite_82_antifairy_circle(k),
            0x83 | 0x84 => self.sprite_83_green_eyegore(k),
            0x85 => self.sprite_85_yellow_stalfos(k),
            0x86 => self.sprite_86_kodongo(k),
            0x87 => self.sprite_87_kodongo_fire(k),
            0x88 => self.sprite_88_mothula(k),
            0x89 => self.sprite_89_mothula_beam(k),
            0x8a => self.sprite_8a_spike_block(k),
            0x8b => self.sprite_8b_gibdo(k),
            0x8c => self.sprite_8c_arrghus(k),
            0x8d => self.sprite_8d_arrghi(k),
            0x8e => self.sprite_8e_terrorpin(k),
            0x8f => self.sprite_8f_blob(k),
            0x90 => self.sprite_90_wallmaster(k),
            0x91 => self.sprite_91_stalfos_knight(k),
            0x93 => self.sprite_93_bumper(k),
            0x94 => self.sprite_94_pirogusu(k),
            0x95..=0x98 => self.sprite_95_laser_eye_left(k),
            0x99 => self.sprite_99_pengator(k),
            0x9a => self.sprite_9a_kyameron(k),
            0x9b => self.sprite_9b_wizzrobe(k),
            0x9c | 0x9d => self.sprite_9c_zoro(k),
            0x9e => self.sprite_9e_haunted_grove_ostritch(k),
            0x9f => self.sprite_9f_haunted_grove_rabbit(k),
            0xa0 => self.sprite_a0_haunted_grove_bird(k),
            0xa1 => self.sprite_a1_freezor(k),
            0xa2 => self.sprite_a2_kholdstare(k),
            0xa3 => self.sprite_a3_kholdstare_shell(k),
            0xa4 => self.sprite_a4_falling_ice(k),
            0xa5 | 0xa6 => self.sprite_zazak_main(k),
            0xa7 => self.sprite_a7_stalfos(k),
            0xab => self.sprite_ab_crystal_maiden(k),
            0xac => self.sprite_ac_apple(k),
            0xad => self.sprite_ad_old_man(k),
            0xae..=0xb1 => self.sprite_ae_pipe_down(k),
            0xb2 => self.sprite_b2_player_bee(k),
            0xb3 => self.sprite_b3_pedestal_plaque(k),
            0xb4 => self.sprite_b4_purple_chest(k),
            0xb5 => self.sprite_b5_bomb_shop(k),
            0xb6 => self.sprite_b6_kiki(k),
            0xb7 => self.sprite_b7_blind_maiden(k),
            0xb8 => self.sprite_b8_dialogue_tester(k),
            0xb9 => self.sprite_b9_bully_and_pink_ball(k),
            0xba => self.sprite_ba_whirlpool(k),
            0xbb => self.sprite_bb_shopkeeper(k),
            0xbc => self.sprite_bc_drunkard(k),
            0xed | 0xef..=0xf1 => self.sprite_ed_somaria_platform(k),
            _ => return false,
        }
        true
    }

    pub fn dispatch_prep(&mut self, k: usize, kind: u8) -> bool {
        match kind {
            0x82 => self.sprite_prep_antifairy_circle(k),
            0x83 | 0x84 => self.sprite_prep_eyegore(k),
            _ => return false,
        }
        true
    }
    #[inline]
    fn sprite_x(&self, k: usize) -> u16 {
        u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()])
    }
    #[inline]
    fn sprite_y(&self, k: usize) -> u16 {
        u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()])
    }
    #[inline]
    fn set_sprite_x(&mut self, k: usize, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.ram.set_sprite_x_low(k, lo);
        self.ram.set_sprite_x_high(k, hi);
    }
    #[inline]
    fn set_sprite_y(&mut self, k: usize, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.ram.set_sprite_y_low(k, lo);
        self.ram.set_sprite_y_high(k, hi);
    }
    fn set_garnish_x(&mut self, j: usize, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.ram.write_u8(GARNISH_X_LO + j, lo);
        self.ram.write_u8(GARNISH_X_HI + j, hi);
    }
    fn set_garnish_y(&mut self, j: usize, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.ram.write_u8(GARNISH_Y_LO + j, lo);
        self.ram.write_u8(GARNISH_Y_HI + j, hi);
    }

    // Entries above the self-contained shop/platform tail retain an explicit
    // integration seam while their callees are migrated from the adjacent bank.

    pub fn stalfos_skellington(&mut self, k: usize) {
        const ANIM: [u8; 4] = [8, 9, 10, 11];
        const CHECK_DIR: [u8; 4] = [3, 2, 1, 0];
        let near = self.ram.sprite(k).state() == 9
            && self
                .ram
                .read_u16(LINK_X)
                .wrapping_sub(self.sprite_x(k))
                .wrapping_add(40)
                < 80
            && self
                .ram
                .read_u16(LINK_Y)
                .wrapping_sub(self.sprite_y(k))
                .wrapping_add(48)
                < 80
            && self.ram.read_u8(PLAYER_OAM_Y_OFFSET) != 0x80
            && (self.ram.sprite(k).z() | self.ram.sprite(k).pause()) == 0
            && self.ram.sprite(k).floor() == self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL);
        if near {
            let dir = self.runtime.direction_to_face_link(self.ram, k);
            let b = self.ram.read_u8(BUTTON_B_FRAMES);
            let jump = if self.ram.read_u8(LINK_IS_RUNNING) == 0 {
                if b >= 0x90 {
                    true
                } else if (b.wrapping_sub(9) as i8) >= 0 {
                    false
                } else {
                    dir != CHECK_DIR[(self.ram.read_u8(LINK_DIRECTION_FACING) >> 1) as usize]
                }
            } else {
                dir != CHECK_DIR[(self.ram.read_u8(LINK_DIRECTION_FACING) >> 1) as usize]
            };
            if jump {
                self.ram.set_sprite_scratch_d(k, dir);
                let (yv, xv) = self.runtime.project_speed_towards_link(self.ram, k, 32);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, xv.wrapping_neg());
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, yv.wrapping_neg());
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, 32);
                self.runtime.queue_sfx3(self.ram, k, 0x13);
                self.ram
                    .set_sprite_z(k, self.ram.sprite(k).z().wrapping_add(1));
            }
        }
        if self.ram.sprite(k).z() == 0 {
            self.sprite_zazak_main(k);
            return;
        }
        self.ram
            .set_sprite_graphics(k, ANIM[self.ram.sprite(k).scratch_d() as usize]);
        self.runtime.bank2b_stalfos_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).scratch_f() != 0 {
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        let t = self.runtime.check_tile_collision(self.ram, k);
        if t & 3 != 0 {
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
        }
        if t & 12 != 0 {
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
        }
        self.runtime.move_xyz(self.ram, k);
        self.ram.set_sprite_signed_field(
            k,
            SpriteField::VelocityZ,
            self.ram
                .sprite(k)
                .signed_field(SpriteField::VelocityZ)
                .wrapping_sub(2),
        );
        if (self.ram.sprite(k).z().wrapping_sub(1) as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
            self.runtime.queue_sfx2(self.ram, k, 0x21);
            if self.ram.sprite(k).subtype() != 0 {
                self.ram.set_sprite_aux3_delay(k, 16);
                self.ram.set_sprite_subtype2(k, 0);
            }
        }
    }

    pub fn sprite_zazak_main(&mut self, k: usize) {
        const ANIM: [u8; 8] = [6, 4, 0, 2, 7, 5, 1, 3];
        const DELAY: [u8; 4] = [16, 32, 64, 32];
        const XV: [i8; 4] = [16, -16, 0, 0];
        const YV: [i8; 4] = [0, 0, 16, -16];
        const DIR2: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
        if self.ram.sprite(k).scratch_b() != 0 {
            self.fire_phlegm_draw(k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.ram
                .set_sprite_graphics(k, self.ram.read_u8(FRAME_COUNTER) >> 1 & 1);
            self.runtime.check_damage_to_link(self.ram, k);
            self.runtime.move_xy(self.ram, k);
            if self.runtime.check_tile_collision(self.ram, k) != 0 {
                self.ram.set_sprite_state(k, 0);
                self.runtime.bank2b_place_rupulse_spark(self.ram, k);
            }
            return;
        }
        let t = self.ram.sprite(k).aux3_delay();
        if t != 0 {
            self.ram.set_sprite_ai_state(k, 0);
            self.ram.set_sprite_main_delay(k, 32);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
            let d = self.runtime.direction_to_face_link(self.ram, k);
            self.ram.set_sprite_head_direction(k, d);
        }
        if t == 1 {
            self.stalfos_throw_bone(k);
            self.ram.set_sprite_subtype2(k, 1);
        }
        let i = ((self.ram.sprite(k).subtype2() & 1) * 4 + self.ram.sprite(k).scratch_d()) as usize;
        self.ram.set_sprite_graphics(k, ANIM[i]);
        if self.ram.sprite(k).kind() == 0xa7 {
            self.runtime.bank2b_stalfos_draw(self.ram, k);
        } else {
            self.runtime.bank2b_zazak_draw(self.ram, k);
        }
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.runtime.move_xy(self.ram, k);
        self.runtime.check_tile_collision(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 if self.ram.sprite(k).main_delay() == 0 => {
                let r = self.runtime.random(self.ram);
                self.ram.set_sprite_main_delay(k, DELAY[(r & 3) as usize]);
                self.ram.set_sprite_ai_state(k, 1);
                let j = self.ram.sprite(k).head_direction() as usize;
                self.ram.set_sprite_scratch_d(k, j as u8);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, XV[j]);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, YV[j]);
            }
            1 => {
                if self.ram.sprite(k).wall_collision() != 0 {
                    self.ram.set_sprite_main_delay(k, 16);
                } else if self.ram.sprite(k).main_delay() != 0 {
                    let g = self.ram.sprite(k).scratch_g().wrapping_sub(1);
                    self.ram.set_sprite_scratch_g(k, g);
                    if (g as i8) < 0 {
                        self.ram.set_sprite_scratch_g(k, 11);
                        self.ram
                            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
                    }
                    return;
                } else if self.ram.sprite(k).kind() == 0xa6
                    && self.ram.sprite(k).scratch_d()
                        == self.runtime.direction_to_face_link(self.ram, k)
                    && self.ram.sprite(k).floor() == self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL)
                {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_main_delay(k, 48);
                    self.ram.set_sprite_aux1_delay(k, 48);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
                    return;
                } else {
                    self.ram.set_sprite_main_delay(k, 32);
                }
                let d = self.ram.sprite(k).scratch_d() as usize;
                let r = self.runtime.random(self.ram);
                self.ram
                    .set_sprite_head_direction(k, DIR2[d * 2 + (r & 1) as usize]);
                self.ram.set_sprite_ai_state(k, 0);
                let c = self.ram.sprite(k).scratch_c().wrapping_add(1);
                self.ram.set_sprite_scratch_c(k, c);
                if c == 4 {
                    self.ram.set_sprite_scratch_c(k, 0);
                    let h = self.runtime.direction_to_face_link(self.ram, k);
                    self.ram.set_sprite_head_direction(k, h);
                    self.ram.set_sprite_main_delay(k, 24);
                }
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
            }
            2 => {
                let d = self.ram.sprite(k).main_delay();
                if d == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                } else if d == 24 {
                    self.sprite_spawn_fire_phlegm(k);
                }
            }
            _ => {}
        }
    }

    pub fn sprite_8f_blob(&mut self, k: usize) {
        if self.ram.sprite(k).state() == 9 && self.ram.sprite(k).scratch_e() != 0 {
            self.ram.set_sprite_scratch_e(k, 0);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, 1);
            let t = self.runtime.check_tile_collision(self.ram, k);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
            if t != 0 {
                self.ram.set_sprite_state(k, 0);
                return;
            }
            self.runtime.queue_sfx2(self.ram, k, 0x20);
        }
        if self.ram.sprite(k).scratch_c() != 0 {
            self.ram.set_sprite_object_priority(k, 0x30);
        }
        self.zol_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).ai_state() >= 2 {
            self.runtime.check_damage_from_link(self.ram, k);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                let bak = self.ram.sprite(k).flags4();
                self.ram.set_sprite_flags4(k, bak | 9);
                self.ram
                    .set_sprite_flags2(k, self.ram.sprite(k).flags2() | 0x80);
                let hit = self.runtime.check_damage_to_link(self.ram, k);
                self.ram.set_sprite_flags4(k, bak);
                if hit {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_main_delay(k, 127);
                    self.ram
                        .set_sprite_flags2(k, self.ram.sprite(k).flags2() & !0x80);
                    self.set_sprite_x(k, self.ram.read_u16(LINK_X));
                    self.set_sprite_y(k, self.ram.read_u16(LINK_Y).wrapping_add(8));
                    self.ram.set_sprite_aux4_delay(k, 48);
                    self.ram.set_sprite_ignore_projectile(k, 0);
                }
            }
            1 => {
                let d = self.ram.sprite(k).main_delay();
                if d == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 32);
                    self.runtime.apply_speed_towards_link(self.ram, k, 16);
                    self.runtime.queue_sfx3(self.ram, k, 0x30);
                } else {
                    const G: [u8; 16] = [0, 1, 7, 7, 6, 6, 5, 5, 6, 6, 5, 5, 4, 4, 4, 4];
                    self.ram.set_sprite_graphics(k, G[(d >> 3) as usize]);
                }
            }
            2 => {
                let d = self.ram.sprite(k).main_delay();
                if d == 0 {
                    self.runtime.check_damage_from_link(self.ram, k);
                    self.runtime.move_xy(self.ram, k);
                    self.runtime.check_tile_collision(self.ram, k);
                    let old = self.ram.sprite(k).z();
                    self.runtime.move_z(self.ram, k);
                    let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                    if zv >= -64 {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityZ,
                            zv.wrapping_sub(2),
                        );
                    }
                    let z = self.ram.sprite(k).z();
                    if ((z ^ old) as i8) < 0 && (z as i8) < 0 {
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
                        self.ram.set_sprite_z(k, 0);
                        self.ram.set_sprite_scratch_c(k, 0);
                        self.ram.set_sprite_main_delay(k, 31);
                        self.ram.set_sprite_head_direction(k, 8);
                    }
                } else if d == 1 {
                    self.ram.set_sprite_main_delay(k, 32);
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_graphics(k, 0);
                } else {
                    self.ram
                        .set_sprite_graphics(k, if (d - 1) >> 4 == 0 { 0 } else { 1 });
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::VelocityX,
                        if self.ram.read_u8(FRAME_COUNTER) >> 1 & 1 == 0 {
                            -8
                        } else {
                            8
                        },
                    );
                    self.runtime.move_x(self.ram, k);
                }
            }
            3 => {
                self.runtime.check_damage_to_link(self.ram, k);
                if self.ram.sprite(k).aux1_delay() == 0 {
                    self.runtime.apply_speed_towards_link(self.ram, k, 48);
                    let r = self.runtime.random(self.ram);
                    self.ram.set_sprite_aux1_delay(k, r & 63 | 96);
                    let f = self.ram.sprite(k).oam_flags() & 0x3f
                        | if self.ram.sprite(k).signed_field(SpriteField::VelocityX) < 0 {
                            0x40
                        } else {
                            0
                        };
                    self.ram.set_sprite_oam_flags(k, f);
                }
                if self.ram.sprite(k).aux2_delay() == 0 {
                    let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                    self.ram.set_sprite_subtype2(k, s);
                    if (s & 14) == 0 && self.ram.sprite(k).wall_collision() == 0 {
                        self.runtime.move_xy(self.ram, k);
                        let g = self.ram.sprite(k).scratch_g().wrapping_add(1);
                        self.ram.set_sprite_scratch_g(k, g);
                        if g == self.ram.sprite(k).head_direction() {
                            self.ram.set_sprite_scratch_g(k, 0);
                            let r = self.runtime.random(self.ram);
                            self.ram.set_sprite_aux2_delay(k, (r & 31) + 64);
                            let r = self.runtime.random(self.ram);
                            self.ram.set_sprite_head_direction(k, r & 31 | 16);
                        }
                    }
                    self.runtime.check_tile_collision(self.ram, k);
                    self.ram.set_sprite_graphics(k, (s & 8) >> 3);
                } else {
                    self.ram.set_sprite_graphics(
                        k,
                        u8::from(self.ram.sprite(k).aux2_delay() & 0x10 != 0),
                    );
                }
            }
            _ => {}
        }
    }

    pub fn sprite_91_stalfos_knight(&mut self, k: usize) {
        if self.ram.sprite(k).ai_state() == 0 {
            self.runtime.prep_oam(self.ram, k);
        } else {
            self.stalfos_knight_draw(k);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).hit_timer() & 127 == 1 {
            self.ram.set_sprite_hit_timer(k, 0);
            self.ram.set_sprite_ai_state(k, 6);
            self.ram.set_sprite_main_delay(k, 255);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
            self.runtime.set_enemy_damage(self.ram, 0x918, 2);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        let ground = |this: &mut Self| {
            this.ram.set_sprite_ai_state(k, 2);
            this.ram.set_sprite_ignore_projectile(k, 0);
            this.ram.set_sprite_z(k, 0);
            this.ram
                .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
            this.ram.set_sprite_main_delay(k, 63);
        };
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_flags4(k, 9);
                self.ram.set_sprite_ignore_projectile(k, 9);
                let old = self.ram.sprite(k).flags2();
                self.ram.set_sprite_flags2(k, old | 128);
                let hit = self.runtime.check_damage_to_link(self.ram, k);
                self.ram.set_sprite_flags2(k, old);
                if hit {
                    self.ram.set_sprite_z(k, 144);
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_head_direction(k, 2);
                    self.ram.set_sprite_graphics(k, 2);
                    self.runtime.queue_sfx2(self.ram, k, 0x20);
                }
            }
            1 => {
                let old = self.ram.sprite(k).z();
                self.runtime.move_z(self.ram, k);
                let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                if zv >= -64 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, zv.wrapping_sub(3));
                }
                let z = self.ram.sprite(k).z();
                if ((old ^ z) as i8) < 0 && (z as i8) < 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_ignore_projectile(k, 0);
                    self.ram.set_sprite_z(k, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
                    self.ram.set_sprite_main_delay(k, 63);
                }
            }
            2 => {
                self.runtime.set_enemy_damage(self.ram, 0x918, 0);
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                let d = self.ram.sprite(k).main_delay();
                if d == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    let r = self.runtime.random(self.ram);
                    self.ram.set_sprite_scratch_b(k, r & 63);
                    self.ram.set_sprite_main_delay(k, 127);
                } else {
                    self.ram
                        .set_sprite_scratch_c(k, if d >> 5 == 0 { 0 } else { 1 });
                    self.ram
                        .set_sprite_graphics(k, if d >> 5 == 0 { 0 } else { 1 });
                    self.ram.set_sprite_head_direction(k, 2);
                }
            }
            3 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                let d = self.ram.sprite(k).main_delay();
                if d == self.ram.sprite(k).scratch_b() {
                    let h = self.runtime.is_right_of_link(self.ram, k);
                    self.ram.set_sprite_head_direction(k, h);
                    self.ram.set_sprite_ai_state(k, 4);
                    self.ram.set_sprite_main_delay(k, 32);
                } else {
                    const DIR: [u8; 16] = [0, 0, 0, 2, 1, 1, 1, 2, 0, 0, 0, 2, 1, 1, 1, 2];
                    self.ram
                        .set_sprite_head_direction(k, DIR[(d >> 3) as usize]);
                    self.ram.set_sprite_scratch_c(k, 0);
                    self.ram.set_sprite_graphics(k, 0);
                }
            }
            4 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 5);
                    self.ram.set_sprite_main_delay(k, 255);
                    self.ram.set_sprite_aux1_delay(k, 32);
                }
                self.ram.set_sprite_scratch_c(k, 1);
                self.ram.set_sprite_graphics(k, 1);
            }
            5 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                if self.ram.sprite(k).aux1_delay() == 0 {
                    self.runtime.move_xyz(self.ram, k);
                    self.runtime.check_tile_collision(self.ram, k);
                    let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                    if zv >= -64 {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityZ,
                            zv.wrapping_sub(2),
                        );
                    }
                    if (self.ram.sprite(k).z().wrapping_sub(1) as i8) < 0 {
                        self.ram.set_sprite_z(k, 0);
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
                        if self.ram.sprite(k).main_delay() == 0 {
                            ground(self);
                            return;
                        }
                        self.ram.set_sprite_aux1_delay(k, 16);
                    }
                    self.ram.set_sprite_graphics(
                        k,
                        if self.ram.sprite(k).signed_field(SpriteField::VelocityZ) < 24 {
                            2
                        } else {
                            0
                        },
                    );
                } else {
                    if self.ram.sprite(k).aux1_delay() == 1 {
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityZ, 48);
                        self.runtime.apply_speed_towards_link(self.ram, k, 16);
                        let h = self.runtime.is_right_of_link(self.ram, k);
                        self.ram.set_sprite_head_direction(k, h);
                        self.runtime.queue_sfx3(self.ram, k, 0x13);
                    }
                    self.ram.set_sprite_scratch_c(k, 1);
                    self.ram.set_sprite_graphics(k, 1);
                }
            }
            6 => {
                self.runtime.move_xyz(self.ram, k);
                self.runtime.check_tile_collision(self.ram, k);
                let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                if zv >= -64 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, zv.wrapping_sub(2));
                }
                if (self.ram.sprite(k).z().wrapping_sub(1) as i8) < 0 {
                    self.ram.set_sprite_z(k, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
                }
                let j = self.ram.sprite(k).main_delay();
                if j == 0 {
                    if self.runtime.random(self.ram) & 1 != 0 {
                        ground(self);
                    } else {
                        self.ram.set_sprite_ai_state(k, 7);
                        self.ram.set_sprite_main_delay(k, 80);
                    }
                } else {
                    if j >= 224 && j & 3 == 0 {
                        self.runtime.queue_sfx3(self.ram, k, 0x14);
                    }
                    const C: [u8; 32] = [
                        0, 4, 8, 12, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,
                        14, 14, 14, 14, 14, 14, 14, 15, 14, 12, 8, 4, 0,
                    ];
                    self.ram.set_sprite_scratch_c(k, C[(j >> 3) as usize]);
                    self.ram.set_sprite_graphics(k, 3);
                    self.ram.set_sprite_head_direction(k, 2);
                }
            }
            7 => {
                let d = self.ram.sprite(k).main_delay();
                if d == 0 {
                    ground(self);
                } else {
                    self.ram
                        .set_sprite_graphics(k, if d >> 2 & 1 == 0 { 1 } else { 4 });
                }
            }
            _ => {}
        }
    }
    pub fn sprite_8c_arrghus(&mut self, k: usize) {
        const GFX: [u8; 9] = [1, 1, 1, 2, 2, 1, 1, 0, 0];
        self.ram
            .set_sprite_object_priority(k, self.ram.sprite(k).object_priority() | 0x30);
        self.arrghus_draw(k);
        if (self.ram.sprite(k).state() != 9 || self.ram.sprite(k).z() < 96)
            && self.runtime.return_if_inactive(self.ram, k)
        {
            return;
        }
        self.arrghus_handle_puffs(k);
        self.ram.write_u8(OVERLORD_X_LO + 4, 1);
        if self.ram.sprite(k).hit_timer() & 127 == 2 {
            self.ram.set_sprite_ai_state(k, 3);
            self.runtime.queue_sfx3(self.ram, k, 0x32);
            self.ram.set_sprite_subtype2(k, 0);
            self.ram.set_sprite_flags3(k, 64);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_link(self.ram, k);
        let s = self.ram.sprite(k).subtype2();
        self.ram.set_sprite_subtype2(k, s.wrapping_add(1));
        if s & 3 == 0 {
            let mut g = self.ram.sprite(k).scratch_g().wrapping_add(1);
            if g == 9 {
                g = 0;
            }
            self.ram.set_sprite_scratch_g(k, g);
            self.ram.set_sprite_graphics(k, GFX[g as usize]);
        }
        let collision = self.runtime.check_tile_collision(self.ram, k);
        if collision != 0 {
            if self.ram.sprite(k).ai_state() == 5 {
                if collision & 3 != 0 {
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::VelocityX,
                        self.ram
                            .sprite(k)
                            .signed_field(SpriteField::VelocityX)
                            .wrapping_neg(),
                    );
                } else {
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::VelocityY,
                        self.ram
                            .sprite(k)
                            .signed_field(SpriteField::VelocityY)
                            .wrapping_neg(),
                    );
                }
            } else {
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
            }
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_main_delay(k, 48);
                }
                self.runtime.move_xy(self.ram, k);
                self.runtime.approach_target_speed(
                    self.ram,
                    k,
                    self.ram.sprite(k).signed_field(SpriteField::HeadDirection),
                    self.ram.sprite(k).signed_field(SpriteField::ScratchD),
                );
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    if !self.runtime.check_if_room_is_clear(self.ram) {
                        let n = self.ram.read_u8(OVERLORD_X_LO + 3).wrapping_add(1);
                        self.ram.write_u8(OVERLORD_X_LO + 3, n);
                        if n == 4 {
                            self.ram.write_u8(OVERLORD_X_LO + 3, 0);
                            self.ram.set_sprite_ai_state(k, 2);
                            self.ram.set_sprite_main_delay(k, 176);
                        } else {
                            let r = self.runtime.random(self.ram);
                            self.ram.set_sprite_main_delay(k, (r & 63) + 48);
                            let speed = (self.ram.sprite(k).main_delay() & 3) + 8;
                            let (yv, xv) =
                                self.runtime.project_speed_towards_link(self.ram, k, speed);
                            self.ram
                                .set_sprite_signed_field(k, SpriteField::HeadDirection, xv);
                            self.ram
                                .set_sprite_signed_field(k, SpriteField::ScratchD, yv);
                        }
                    } else {
                        self.ram.set_sprite_ai_state(k, 3);
                        self.runtime.queue_sfx3(self.ram, k, 0x32);
                        self.ram.set_sprite_subtype2(k, 0);
                    }
                } else {
                    self.runtime.move_xy(self.ram, k);
                    self.runtime.approach_target_speed(self.ram, k, 0, 0);
                }
            }
            2 => {
                self.ram.write_u8(OVERLORD_X_LO + 4, 8);
                let d = self.ram.sprite(k).main_delay();
                if d < 32 {
                    let n = self.ram.read_u8(OVERLORD_X_LO + 2).wrapping_sub(1);
                    self.ram.write_u8(OVERLORD_X_LO + 2, n);
                    if (n as i8) < 0 {
                        self.ram.write_u8(OVERLORD_X_LO + 2, 0);
                        self.ram.set_sprite_ai_state(k, 1);
                        self.ram.set_sprite_main_delay(k, 112);
                    }
                } else if d < 96 {
                    self.ram.write_u8(
                        OVERLORD_X_LO + 2,
                        self.ram.read_u8(OVERLORD_X_LO + 2).wrapping_add(1),
                    );
                } else if d == 96 {
                    self.runtime.queue_sfx3(self.ram, k, 0x26);
                } else if d & 15 == 0 {
                    self.runtime.queue_sfx3(self.ram, k, 6);
                }
            }
            3 => {
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, 120);
                self.runtime.move_z(self.ram, k);
                if self.ram.sprite(k).z() >= 224 {
                    self.ram.set_sprite_main_delay(k, 64);
                    self.ram.set_sprite_ai_state(k, 4);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
                    self.ram.set_sprite_x_low(k, self.ram.read_u8(LINK_X));
                    self.ram.set_sprite_y_low(k, self.ram.read_u8(LINK_Y));
                }
            }
            4 => {
                let mut a = self.ram.sprite(k).main_delay();
                if a == 0 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, -112);
                    let old = self.ram.sprite(k).z();
                    self.runtime.move_z(self.ram, k);
                    a = old ^ self.ram.sprite(k).z();
                    if (a as i8) < 0 && self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
                        self.ram.set_sprite_z(k, 0);
                        self.runtime.bank2b_spawn_big_splash(self.ram, k);
                        self.ram.set_sprite_ai_state(k, 5);
                        self.ram.set_sprite_main_delay(k, 32);
                        self.runtime.queue_sfx3(self.ram, k, 3);
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityX, 32);
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityY, 32);
                    }
                }
                if a == 1 {
                    self.runtime.queue_sfx2(self.ram, k, 0x20);
                }
            }
            5 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_flags3(k, 0);
                    self.runtime.move_xy(self.ram, k);
                    self.runtime.check_damage_from_link(self.ram, k);
                    if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
                        self.runtime.queue_sfx2(self.ram, k, 0x28);
                        let limit = if self.ram.sprite(k).signed_field(SpriteField::VelocityY) < 0 {
                            29
                        } else {
                            14
                        };
                        if let Some(j) = self.runtime.garnish_alloc_with_limit(self.ram, limit) {
                            self.ram.write_u8(GARNISH_TYPE + j, 21);
                            self.ram.write_u8(GARNISH_ACTIVE, 21);
                            self.ram
                                .write_u8(GARNISH_X_LO + j, self.ram.sprite(k).x_low());
                            self.ram
                                .write_u8(GARNISH_X_HI + j, self.ram.sprite(k).x_high());
                            self.ram.write_u8(
                                GARNISH_Y_LO + j,
                                self.ram.sprite(k).y_low().wrapping_add(24),
                            );
                            self.ram
                                .write_u8(GARNISH_Y_HI + j, self.ram.sprite(k).y_high());
                            self.ram.write_u8(GARNISH_COUNTDOWN + j, 15);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub fn sprite_85_yellow_stalfos(&mut self, k: usize) {
        const PRIO: [u8; 6] = [0x30, 0, 0, 0, 0x30, 0];
        const GFX: [u8; 32] = [
            8, 5, 1, 1, 8, 5, 1, 1, 8, 5, 1, 1, 7, 4, 2, 2, 7, 4, 2, 2, 7, 4, 2, 2, 7, 4, 2, 2, 7,
            4, 2, 2,
        ];
        const HX: [i8; 32] = [
            -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, 0, 0, 0, 0, 0,
            0, 0, 0, -1, 0, 1, 0, -1, 0, 1, 0, 0, 0, 0, 0,
        ];
        const HY: [u8; 32] = [
            13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 12, 11, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
        ];
        const NGFX: [u8; 16] = [1, 1, 1, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9];
        const NHY: [u8; 16] = [10, 10, 10, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7];
        if self.ram.sprite(k).scratch_a() == 0 {
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, 1);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, 1);
            if self.runtime.check_tile_collision(self.ram, k) != 0 {
                self.ram.set_sprite_state(k, 0);
                return;
            }
            self.ram.set_sprite_scratch_a(k, 1);
            self.ram.set_sprite_scratch_c(k, 10);
            self.ram
                .set_sprite_flags3(k, self.ram.sprite(k).flags3() | 64);
            self.runtime.queue_sfx2(self.ram, k, 0x20);
        }
        let ai = self.ram.sprite(k).ai_state() as usize;
        self.ram
            .set_sprite_object_priority(k, self.ram.sprite(k).object_priority() | PRIO[ai]);
        self.yellow_stalfos_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.read_u8(LINK_SWORD_TYPE) >= 3 {
            if self.runtime.return_if_recoiling(self.ram, k) {
                return;
            }
        } else if ai != 5 && self.ram.sprite(k).hit_timer() != 0 {
            self.ram.set_sprite_hit_timer(k, 0);
            self.ram.set_sprite_ai_state(k, 5);
            self.ram.set_sprite_main_delay(k, 255);
        }
        self.ram.set_sprite_ignore_projectile(k, 1);
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_head_direction(k, 2);
                let old = self.ram.sprite(k).z();
                self.runtime.move_z(self.ram, k);
                let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                if zv >= -64 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, zv.wrapping_sub(3));
                }
                if old as i8 >= 0 && self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_z(k, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
                    self.ram.set_sprite_main_delay(k, 64);
                    self.yellow_stalfos_animate(k);
                }
            }
            1 => {
                self.ram.set_sprite_ignore_projectile(k, 0);
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                let d = self.runtime.direction_to_face_link(self.ram, k);
                self.ram.set_sprite_head_direction(k, d);
                self.ram.set_sprite_scratch_d(k, d);
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_main_delay(k, 127);
                }
                self.ram
                    .set_sprite_flags3(k, self.ram.sprite(k).flags3() & !0x40);
            }
            2 => {
                self.ram.set_sprite_ignore_projectile(k, 0);
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                let j = self.ram.sprite(k).main_delay();
                if j == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_main_delay(k, 64);
                    return;
                }
                if j == 48 {
                    self.yellow_stalfos_emancipate_head(k);
                }
                self.ram.set_sprite_graphics(
                    k,
                    GFX[((j >> 2 & !3) | self.ram.sprite(k).scratch_d()) as usize],
                );
                let q = (j >> 2) as usize;
                self.ram
                    .set_sprite_signed_field(k, SpriteField::ScratchB, HX[q]);
                self.ram.set_sprite_scratch_c(k, HY[q]);
                self.ram
                    .set_sprite_flags3(k, self.ram.sprite(k).flags3() & !0x40);
            }
            3 => {
                self.ram.set_sprite_ignore_projectile(k, 0);
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 4);
                }
                self.yellow_stalfos_animate(k);
            }
            4 => {
                self.ram.set_sprite_graphics(k, 0);
                self.ram.set_sprite_head_direction(k, 2);
                let old = self.ram.sprite(k).z();
                self.runtime.move_z(self.ram, k);
                let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                if zv < 64 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, zv.wrapping_add(2));
                }
                if (old as i8) < 0 && self.ram.sprite(k).signed_field(SpriteField::Z) >= 0 {
                    self.ram.set_sprite_state(k, 0);
                }
            }
            5 => {
                self.ram.set_sprite_ignore_projectile(k, 0);
                self.runtime.check_damage_from_link(self.ram, k);
                let j = self.ram.sprite(k).main_delay();
                if j == 0 {
                    self.ram.set_sprite_ai_state(k, 4);
                }
                self.ram.set_sprite_graphics(k, NGFX[(j >> 4) as usize]);
                self.ram.set_sprite_scratch_c(k, NHY[(j >> 4) as usize]);
            }
            _ => {}
        }
    }
    pub fn sprite_7a_agahnim(&mut self, k: usize) {
        const START: [u8; 2] = [1, 6];
        const G0: [u8; 5] = [12, 13, 14, 15, 16];
        const TAB5: [i8; 2] = [32, -32];
        const TAB6: [u8; 2] = [9, 11];
        const DIR: [u8; 25] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 0, 1, 1, 4, 4, 0, 2, 2, 4, 4, 3, 2, 2,
        ];
        const G1: [u8; 6] = [2, 10, 8, 0, 4, 6];
        const T0: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0];
        const T1: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 6, 5, 4, 3, 2, 1, 0, 0, 0];
        const T2: [u8; 6] = [30, 24, 12, 0, 6, 18];
        const G2: [u8; 5] = [16, 15, 14, 13, 12];
        const T3: [u8; 16] = [
            0x38, 0x38, 0x38, 0x58, 0x78, 0x98, 0xb8, 0xb8, 0xb8, 0x98, 0x58, 0x58, 0x60, 0x90,
            0x98, 0x78,
        ];
        const T4: [u8; 16] = [
            0xb8, 0x78, 0x58, 0x48, 0x48, 0x48, 0x58, 0x78, 0xb8, 0xb8, 0xb8, 0x90, 0x70, 0x70,
            0x90, 0xa0,
        ];
        const G3: [u8; 7] = [0, 8, 10, 2, 2, 6, 4];
        self.agahnim_draw(k);
        if self.ram.sprite(k).pause() != 0 {
            self.ram.set_sprite_main_delay(k, 32);
            self.ram.set_sprite_graphics(k, 0);
            self.ram.set_sprite_scratch_d(k, 3);
        }
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.ram.set_sprite_ignore_projectile(k, 1);
        match self.ram.sprite(k).ai_state() {
            0 => self.ram.set_sprite_ai_state(
                k,
                START[usize::from(self.ram.read_u8(IS_IN_DARK_WORLD) != 0)],
            ),
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.runtime.show_message(self.ram, 0x13f);
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_main_delay(k, 32);
                }
            }
            2 => {
                self.ram.write_u8(BYTE_7E0FF8, 0);
                let d = self.ram.sprite(k).main_delay();
                if d == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_main_delay(k, 255);
                } else {
                    self.ram.set_sprite_graphics(k, G0[(d >> 3) as usize]);
                }
            }
            3 => {
                let d = self.ram.sprite(k).main_delay();
                if d == 192 {
                    self.runtime.queue_sfx3(self.ram, k, 0x27);
                }
                if !(16..239).contains(&d) {
                    let index = if self.ram.read_u8(IS_IN_DARK_WORLD) != 0 {
                        k
                    } else {
                        2
                    };
                    self.runtime.agahnim_warp_shadow_filter(self.ram, index);
                } else if k == 0 {
                    self.runtime.check_damage_to_and_from_link(self.ram, k);
                    self.ram.set_sprite_ignore_projectile(k, 0);
                }
                if d == 0 {
                    self.ram.set_sprite_ai_state(k, 4);
                    self.ram.set_sprite_main_delay(k, 39);
                    return;
                }
                if d >= 128 {
                    self.runtime.apply_speed_towards_link(self.ram, k, 2);
                    let ix =
                        ((self.ram.sprite(k).signed_field(SpriteField::VelocityY) as isize + 2) * 5
                            + 2
                            + self.ram.sprite(k).signed_field(SpriteField::VelocityX) as isize)
                            as usize;
                    self.ram.set_sprite_scratch_d(k, DIR[ix]);
                    self.runtime.apply_speed_towards_link(self.ram, k, 32);
                    if self.ram.sprite(k).subtype() == 4 {
                        self.ram.set_sprite_scratch_d(k, 3);
                    }
                } else if d == 112 {
                    self.agahnim_perform_attack(k);
                }
                let j = (d >> 4) as usize;
                self.ram.set_sprite_scratch_a(k, T0[j]);
                let t = T1[j];
                self.ram.set_sprite_head_direction(
                    k,
                    if t == 0 {
                        0
                    } else {
                        t + T2[self.ram.sprite(k).scratch_d() as usize]
                    },
                );
                self.ram
                    .set_sprite_graphics(k, G1[self.ram.sprite(k).scratch_d() as usize] + T0[j]);
            }
            4 => {
                let d = self.ram.sprite(k).main_delay();
                self.ram.set_sprite_ignore_projectile(k, d);
                if d == 0 {
                    self.ram.set_sprite_ai_state(k, 5);
                    let j = if self.ram.sprite(k).subtype() == 4 {
                        4
                    } else {
                        (self.runtime.random(self.ram) & 15) as usize
                    };
                    self.ram.set_sprite_scratch_c(k, T3[j]);
                    self.ram.set_sprite_scratch_e(k, T4[j]);
                    self.ram.set_sprite_scratch_g(k, 8);
                } else {
                    self.ram.set_sprite_graphics(k, G2[(d >> 3) as usize]);
                }
            }
            5 => {
                let tx = self.ram.sprite(k).scratch_c();
                let ty = self.ram.sprite(k).scratch_e();
                if self.ram.sprite(k).x_low().wrapping_sub(tx).wrapping_add(7) < 14
                    && self.ram.sprite(k).y_low().wrapping_sub(ty).wrapping_add(7) < 14
                {
                    self.ram.set_sprite_x_low(k, tx);
                    self.ram.set_sprite_y_low(k, ty);
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_main_delay(k, 39);
                    return;
                }
                let x = (u16::from(self.ram.sprite(k).x_high()) << 8) | u16::from(tx);
                let y = (u16::from(self.ram.sprite(k).y_high()) << 8) | u16::from(ty);
                let (yv, xv) =
                    self.runtime
                        .project_speed(self.ram, k, x, y, self.ram.sprite(k).scratch_g());
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, yv);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, xv);
                if self.ram.sprite(k).scratch_g() < 64 {
                    self.ram
                        .set_sprite_scratch_g(k, self.ram.sprite(k).scratch_g() + 1);
                }
                self.runtime.move_xy(self.ram, k);
            }
            6 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.runtime.show_message(self.ram, 0x141);
                    self.ram.set_sprite_ai_state(k, 7);
                    self.ram.set_sprite_main_delay(k, 80);
                }
            }
            7 => {
                let d = self.ram.sprite(k).main_delay();
                if self.ram.sprite(k).animation_clock() != 0 {
                    if d == 0 {
                        self.ram.set_sprite_ai_state(k, 3);
                        self.ram.set_sprite_main_delay(k, 32);
                    } else {
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityX, TAB5[k - 1]);
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityY,
                            self.ram
                                .sprite(k)
                                .signed_field(SpriteField::VelocityY)
                                .wrapping_add(2),
                        );
                        self.runtime.move_xy(self.ram, k);
                        if let Some(j) = self.runtime.agahnim_apply_motion_blur(self.ram, k) {
                            self.ram.set_sprite_oam_flags(j, 4);
                        }
                    }
                } else if d == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_main_delay(k, 32);
                } else if d == 64 {
                    self.ram.write_u8(SOUND_EFFECT_2, 0x28);
                    self.ram.write_u8(0xfb5, 1);
                    for n in (0..2).rev() {
                        if let Some((j, x, y, z)) =
                            self.runtime.spawn_dynamic_ex(self.ram, k, 0x7a, 2)
                        {
                            self.set_sprite_x(j, x);
                            self.set_sprite_y(j, y);
                            self.ram.set_sprite_z(j, z);
                            self.ram.set_sprite_flags3(j, TAB6[n]);
                            self.ram.set_sprite_animation_clock(j, TAB6[n] & 15);
                            self.ram.set_sprite_oam_flags(j, TAB6[n] & 15);
                            self.ram
                                .set_sprite_ai_state(j, self.ram.sprite(k).ai_state());
                            self.ram.set_sprite_main_delay(j, 32);
                        }
                        self.ram
                            .write_u8(0xfb5, self.ram.read_u8(0xfb5).wrapping_sub(1));
                    }
                }
            }
            8 => {
                self.ram.write_u8(FLAG_BLOCK_LINK_MENU, 2);
                self.ram.set_sprite_head_direction(k, 0);
                let d = self.ram.sprite(k).main_delay();
                if d >= 64 {
                    self.ram
                        .set_sprite_hit_timer(k, self.ram.sprite(k).hit_timer() | 0xe0);
                } else {
                    if d == 1 {
                        self.runtime.spawn_phantom_ganon(self.ram, k);
                        self.ram.write_u8(MUSIC_CONTROL, 0x1d);
                    }
                    self.ram.set_sprite_hit_timer(k, 0);
                    self.ram.set_sprite_graphics(k, 17);
                }
            }
            9 => {
                self.ram.set_sprite_head_direction(k, 0);
                let x = self.sprite_x(0);
                let y = self.sprite_y(0);
                if self.sprite_x(k).wrapping_sub(x).wrapping_add(4) < 8
                    && self.sprite_y(k).wrapping_sub(y).wrapping_add(4) < 8
                {
                    self.ram.set_sprite_state(k, 0);
                }
                let (yv, xv) = self.runtime.project_speed(self.ram, k, x, y, 32);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, yv);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, xv);
                self.runtime.move_xy(self.ram, k);
                self.runtime.agahnim_apply_motion_blur(self.ram, k);
            }
            10 => {
                let d = self.ram.sprite(k).main_delay();
                if d == 0 {
                    self.ram.set_sprite_state(k, 0);
                    self.runtime
                        .bank2b_prepare_dungeon_exit_from_boss_fight(self.ram);
                }
                if d < 16 {
                    self.ram.write_u8(CGADSUB_COPY, 0x7f);
                    self.ram.write_u8(TM_COPY, 6);
                    self.ram.write_u8(TS_COPY, 0x10);
                    self.runtime.palette_filter_sp5f(self.ram);
                }
                let mut z = self.ram.sprite(k).velocity_z();
                if z != 0xff {
                    z = z.wrapping_add(1);
                    self.ram.set_sprite_velocity_z(k, z);
                }
                let j = u16::from(self.ram.sprite(k).subpixel_z()) + u16::from(z);
                self.ram.set_sprite_subpixel_z(k, j as u8);
                if j & 0x100 != 0 {
                    let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                    self.ram.set_sprite_subtype2(k, s);
                    if s == 7 {
                        self.ram.set_sprite_subtype2(k, 0);
                        self.runtime.queue_sfx2(self.ram, k, 4);
                    }
                }
                self.ram
                    .set_sprite_graphics(k, G3[self.ram.sprite(k).subtype2() as usize]);
            }
            _ => {}
        }
    }
    pub fn agahnim_draw(&mut self, k: usize) {
        if self.runtime.sprite_is_offscreen(self.ram, k) {
            return;
        }
        const X0: [i8; 72] = [
            -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8,
            8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8,
            -8, 8, -6, 6, -6, 6, -8, 8, -8, 8, -6, 6, -6, 6, 0, 8, 0, 8, -8, 8, -8, 8,
        ];
        const Y0: [i8; 72] = [
            -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8,
            -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8,
            8, 8, -6, -6, 6, 6, -8, -8, 8, 8, -6, -6, 6, 6, 0, 0, 8, 8, 8, 8, 8, 8,
        ];
        const CH: [u8; 72] = [
            0x82, 0x82, 0xa2, 0xa2, 0x80, 0x80, 0xa0, 0xa0, 0x84, 0x84, 0xa4, 0xa4, 0x86, 0x86,
            0xa6, 0xa6, 0x88, 0x8a, 0xa8, 0xaa, 0x8c, 0x8e, 0xac, 0xae, 0xc4, 0xc2, 0xe4, 0xe6,
            0xc0, 0xc2, 0xe0, 0xe2, 0x8a, 0x88, 0xaa, 0xa8, 0x8e, 0x8c, 0xae, 0xac, 0xc2, 0xc4,
            0xe6, 0xe4, 0xc2, 0xc0, 0xe2, 0xe0, 0xec, 0xec, 0xec, 0xec, 0xec, 0xec, 0xec, 0xec,
            0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xdf, 0xdf, 0xdf, 0xdf, 0x40, 0x42,
            0x40, 0x42,
        ];
        const FL: [u8; 72] = [
            0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0,
            0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0,
        ];
        let g = self.ram.sprite(k).graphics() as usize;
        let mut body = [(0, 0, 0, 0); 4];
        for (entry, piece) in body.iter_mut().enumerate() {
            let i = 3 - entry;
            let j = g * 4 + i;
            *piece = (
                X0[j],
                Y0[j],
                CH[j] as u16 | ((FL[j] as u16) << 8),
                if (0x40..0x44).contains(&j) { 0 } else { 2 },
            );
        }
        self.runtime.draw_multiple(self.ram, k, &body);
        if g < 12 {
            self.runtime.draw_shadow_custom(self.ram, k, 18);
        }
        if self.ram.read_u8(SUBMODULE_INDEX) != 0 {
            self.runtime.correct_oam_entries(self.ram, k, 3, 0xff);
        }
        let Some((x, y, _)) = self.runtime.prep_oam_coords(self.ram, k) else {
            return;
        };
        if self.ram.sprite(k).scratch_d() != 0 {
            self.runtime.allocate_oam_region_c(self.ram, 8);
        } else {
            self.runtime.allocate_oam_region_b(self.ram, 8);
        }
        let h = self.ram.sprite(k).head_direction();
        if h == 0 {
            return;
        }
        let q = (h - 1) as usize;
        const X1: [i8; 72] = [
            -7, 15, -11, 11, -11, 11, -8, 8, -4, 4, 0, 0, -10, -1, -14, -5, -14, -5, -12, -7, -10,
            -7, -10, -10, 16, 8, 12, 4, 12, 4, 10, 6, 9, 7, 8, 8, -6, -6, -10, -10, -10, -10, -10,
            -10, -10, -10, -10, -10, 14, 14, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, -7, 15, -11,
            11, -11, 11, -8, 8, -4, 4, 0, 0,
        ];
        const Y1: [i8; 72] = [
            -5, -5, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9, -3, 9, -7, 5, -7, 5, -5, 3, -3, 3, -2,
            -2, -3, 9, -7, 5, -7, 5, -5, 3, -3, 3, -2, -2, -3, 9, -7, 5, -7, 5, -5, 3, -3, 3, -2,
            -2, -3, 9, -7, 5, -7, 5, -5, 3, -3, 3, -2, -2, -5, -5, -9, -9, -9, -9, -9, -9, -9, -9,
            -9, -9,
        ];
        const C1: [u8; 36] = [
            0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc,
            0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6,
            0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6,
        ];
        const BIG: [u8; 36] = [
            0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2,
            2, 0, 2, 2, 2, 2, 2,
        ];
        let flags = ((((self.ram.read_u8(FRAME_COUNTER) >> 1) & 2) + 2) + 0x31) as u16;
        let first = (usize::from(self.ram.read_u16(0x90)) - oam::OAM_BUFFER) / 4;
        for (entry, i) in (0..2).rev().enumerate() {
            let j = q * 2 + i;
            oam::set_plain(
                self.ram,
                first + entry,
                x.wrapping_add_signed(i16::from(X1[j])) as u8,
                y.wrapping_add_signed(i16::from(Y1[j])) as u8,
                C1[q],
                flags as u8,
                BIG[q],
            );
        }
    }
    pub fn sprite_7b_agahnim_balls(&mut self, k: usize) {
        let seeker = self.ram.sprite(k).scratch_b() != 0;
        if seeker {
            if self.ram.sprite(k).main_delay() != 0 {
                self.runtime.apply_speed_towards_link(self.ram, k, 32);
            }
            self.ram.set_sprite_oam_flags(k, 5);
        } else {
            self.ram
                .set_sprite_oam_flags(k, (self.ram.read_u8(FRAME_COUNTER) >> 1 & 2) + 3);
        }
        if self.ram.sprite(k).ai_state() != 0 {
            const GFX: [u8; 16] = [2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0];
            if self.ram.sprite(k).graphics() != 2 {
                self.runtime.draw_single_large(self.ram, k);
            } else {
                self.runtime.draw_single_small(self.ram, k);
            }
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            let d = self.ram.sprite(k).main_delay();
            self.ram.set_sprite_ignore_projectile(k, d);
            if d == 0 {
                self.ram.set_sprite_state(k, 0);
            } else if d == 6 {
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, 64);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, 64);
                self.runtime.move_xy(self.ram, k);
            }
            self.ram.set_sprite_graphics(k, GFX[d as usize]);
            return;
        }
        if seeker {
            self.seeker_energy_ball_draw(k);
        } else {
            self.runtime.draw_single_large(self.ram, k);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        self.runtime.move_xy(self.ram, k);
        if self.runtime.check_tile_collision(self.ram, k) != 0 {
            self.ram.set_sprite_state(k, 0);
            if seeker {
                self.runtime.queue_sfx3(self.ram, k, 0x36);
                self.create_six_blue_balls(k);
                return;
            }
        }
        if self.ram.sprite(k).scratch_a() != 0 && self.ram.sprite(0).ignore_projectile() == 0 {
            let mut hb = SpriteHitBox {
                r0_xlo: self.ram.sprite(k).x_low(),
                r8_xhi: self.ram.sprite(k).x_high(),
                r1_ylo: self.ram.sprite(k).y_low(),
                r9_yhi: self.ram.sprite(k).y_high(),
                r2: 15,
                r3: 15,
                ..Default::default()
            };
            self.runtime.setup_sprite_hit_box(self.ram, 0, &mut hb);
            if self.runtime.hit_boxes_overlap(&hb) {
                self.runtime.give_damage(self.ram, 0, 16, 0xa0);
                self.ram.set_sprite_state(k, 0);
                self.ram.set_sprite_signed_field(
                    0,
                    SpriteField::RecoilX,
                    self.ram.sprite(k).signed_field(SpriteField::VelocityX),
                );
                self.ram.set_sprite_signed_field(
                    0,
                    SpriteField::RecoilY,
                    self.ram.sprite(k).signed_field(SpriteField::VelocityY),
                );
            }
        } else {
            self.runtime.check_damage_to_link(self.ram, k);
            if self.runtime.check_damage_from_link(self.ram, k) & 1 != 0 {
                if seeker {
                    self.ram.set_sprite_state(k, 0);
                    self.runtime.queue_sfx3(self.ram, k, 0x36);
                    self.create_six_blue_balls(k);
                    return;
                }
                self.runtime.queue_sfx2(self.ram, k, 5);
                self.runtime.queue_sfx3(self.ram, k, 0x29);
                self.runtime.apply_speed_towards_link(self.ram, k, 0x30);
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_neg(),
                );
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityY,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityY)
                        .wrapping_neg(),
                );
                self.ram
                    .set_sprite_scratch_a(k, self.ram.sprite(k).scratch_a().wrapping_add(1));
            }
        }
        if ((k as u8 ^ self.ram.read_u8(FRAME_COUNTER)) & 3) != 0 || seeker {
            return;
        }
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0x7b) {
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            self.ram.set_sprite_main_delay(j, 15);
            self.ram.set_sprite_ai_state(j, 15);
            self.ram
                .set_sprite_scratch_b(j, self.ram.sprite(k).scratch_b());
        }
    }
    pub fn sprite_ad_old_man(&mut self, k: usize) {
        const MSG: [u16; 3] = [0x9e, 0x9f, 0xa0];
        self.runtime.bank2b_old_mountain_man_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).subtype2() {
            0 => match self.ram.sprite(k).ai_state() {
                0 => {
                    self.runtime.track_body_to_head(self.ram, k);
                    let h = self.runtime.direction_to_face_link(self.ram, k) ^ 3;
                    self.ram.set_sprite_head_direction(k, h);
                    let j = self.runtime.show_message_on_contact(self.ram, k, 0x9c);
                    if j & 0x100 != 0 {
                        self.ram.set_sprite_scratch_d(k, j as u8);
                        self.ram.set_sprite_head_direction(k, j as u8);
                        self.ram.set_sprite_ai_state(k, 1);
                    }
                }
                1 => {
                    self.ram.write_u8(FOLLOWER_INDICATOR, 4);
                    self.runtime.become_follower(self.ram, k);
                    self.ram.write_u8(WHICH_STARTING_POINT, 5);
                    self.ram.set_sprite_state(k, 0);
                    self.runtime.bank2b_cache_camera_properties(self.ram);
                }
                _ => {}
            },
            1 => {
                self.runtime.move_xy(self.ram, k);
                match self.ram.sprite(k).ai_state() {
                    0 => {
                        self.ram.set_sprite_ai_state(k, 1);
                        self.ram.write_u8(ITEM_RECEIPT_METHOD, 0);
                        self.runtime.link_receive_item(self.ram, 0x1a);
                        self.ram.write_u8(WHICH_STARTING_POINT, 1);
                        self.old_man_enable_cutscene();
                        self.ram.set_sprite_main_delay(k, 48);
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityX, 8);
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityY, 4);
                        self.ram.set_sprite_scratch_d(k, 3);
                        self.ram.set_sprite_head_direction(k, 3);
                    }
                    1 => {
                        self.old_man_enable_cutscene();
                        if self.ram.sprite(k).main_delay() == 0 {
                            self.ram.set_sprite_ai_state(k, 2);
                        }
                        self.ram.set_sprite_graphics(
                            k,
                            ((k as u8 ^ self.ram.read_u8(FRAME_COUNTER)) >> 3) & 1,
                        );
                    }
                    2 => {
                        self.ram.set_sprite_head_direction(k, 0);
                        self.ram.set_sprite_scratch_d(k, 0);
                        let j = self.ram.read_u8(BYTE_7E0FDE) as usize;
                        let x = u16::from_le_bytes([
                            self.ram.read_u8(OVERLORD_X_LO + j),
                            self.ram.read_u8(OVERLORD_X_HI + j),
                        ]);
                        let y = u16::from_le_bytes([
                            self.ram.read_u8(OVERLORD_Y_LO + j),
                            self.ram.read_u8(OVERLORD_Y_HI + j),
                        ]);
                        if y >= self.sprite_y(k) {
                            self.ram.set_sprite_ai_state(k, 3);
                            self.ram
                                .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                            self.ram
                                .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
                        } else {
                            let (yv, xv) = self.runtime.project_speed(self.ram, k, x, y, 8);
                            self.ram
                                .set_sprite_signed_field(k, SpriteField::VelocityY, yv);
                            self.ram
                                .set_sprite_signed_field(k, SpriteField::VelocityX, xv);
                            self.ram.set_sprite_graphics(
                                k,
                                ((k as u8 ^ self.ram.read_u8(FRAME_COUNTER)) >> 3) & 1,
                            );
                            self.old_man_enable_cutscene();
                        }
                    }
                    3 => {
                        self.ram.set_sprite_state(k, 0);
                        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                        self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 0);
                    }
                    _ => {}
                }
            }
            2 => {
                self.runtime.behave_as_barrier(self.ram, k);
                if self.ram.sprite(k).ai_state() != 0 {
                    self.ram.write_u8(LINK_HEARTS_FILLER, 160);
                    self.ram.set_sprite_ai_state(k, 0);
                }
                let j = if self.ram.read_u8(SRAM_PROGRESS_INDICATOR) >= 3 {
                    2
                } else {
                    self.ram.read_u8(LINK_ITEM_MOON_PEARL) as usize
                };
                if self.runtime.show_solicited_message(self.ram, k, MSG[j]) & 0x100 != 0 {
                    self.ram
                        .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
                }
            }
            _ => {}
        }
    }

    pub fn pengator_draw(&mut self, k: usize) {
        const BODY: [DrawPiece; 40] = [
            (-1, -8, 0x0082, 2),
            (0, 0, 0x0088, 2),
            (-1, -7, 0x0082, 2),
            (0, 0, 0x008a, 2),
            (-3, -6, 0x0082, 2),
            (0, 0, 0x0088, 2),
            (-6, -4, 0x0082, 2),
            (0, 0, 0x008a, 2),
            (-4, 0, 0x00a2, 2),
            (4, 0, 0x00a3, 2),
            (1, -8, 0x4082, 2),
            (0, 0, 0x4088, 2),
            (1, -7, 0x4082, 2),
            (0, 0, 0x408a, 2),
            (3, -6, 0x4082, 2),
            (0, 0, 0x4088, 2),
            (6, -4, 0x4082, 2),
            (0, 0, 0x408a, 2),
            (4, 0, 0x40a2, 2),
            (-4, 0, 0x40a3, 2),
            (0, -7, 0x0080, 2),
            (0, 0, 0x0086, 2),
            (0, -7, 0x4080, 2),
            (0, 0, 0x4086, 2),
            (0, -4, 0x0080, 2),
            (0, 0, 0x0086, 2),
            (0, -1, 0x0080, 2),
            (0, 0, 0x0086, 2),
            (-8, 0, 0x008e, 2),
            (8, 0, 0x408e, 2),
            (0, -8, 0x0084, 2),
            (0, 0, 0x008c, 2),
            (0, -8, 0x4084, 2),
            (0, 0, 0x408c, 2),
            (0, -7, 0x0084, 2),
            (0, 0, 0x008c, 2),
            (0, 0, 0x408c, 2),
            (0, -6, 0x4084, 2),
            (-8, 0, 0x00a0, 2),
            (8, 0, 0x40a0, 2),
        ];
        const FEET: [DrawPiece; 4] = [
            (0, 16, 0x00b5, 0),
            (8, 16, 0x40b5, 0),
            (0, -8, 0x00a5, 0),
            (8, -8, 0x40a5, 0),
        ];
        let gfx = self.ram.sprite(k).graphics() as usize;
        self.runtime
            .draw_multiple(self.ram, k, &BODY[gfx * 2..gfx * 2 + 2]);
        if gfx == 14 || gfx == 19 {
            self.ram
                .write_u16(0x90, self.ram.read_u16(0x90).wrapping_add(8));
            self.ram
                .write_u16(0x92, self.ram.read_u16(0x92).wrapping_add(2));
            let i = if gfx == 14 { 0 } else { 2 };
            self.runtime.draw_multiple(self.ram, k, &FEET[i..i + 2]);
        }
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn yellow_stalfos_draw(&mut self, k: usize) {
        const D: [DrawPiece; 22] = [
            (0, 0, 0x000a, 2),
            (0, 0, 0x000a, 2),
            (0, 0, 0x000c, 2),
            (0, 0, 0x000c, 2),
            (0, 0, 0x002c, 2),
            (0, 0, 0x002c, 2),
            (5, 5, 0x002e, 0),
            (0, 0, 0x0024, 2),
            (4, 1, 0x003e, 0),
            (0, 0, 0x0024, 2),
            (0, 0, 0x000e, 2),
            (0, 0, 0x000e, 2),
            (3, 5, 0x402e, 0),
            (0, 0, 0x4024, 2),
            (4, 1, 0x403e, 0),
            (0, 0, 0x4024, 2),
            (0, 0, 0x400e, 2),
            (0, 0, 0x400e, 2),
            (0, 0, 0x002a, 2),
            (0, 0, 0x002a, 2),
            (0, 0, 0x002a, 2),
            (0, 0, 0x002a, 2),
        ];
        let i = self.ram.sprite(k).graphics() as usize * 2;
        let oam = self.ram.read_u16(0x90);
        self.ram.write_u16(0x90, oam.wrapping_add(4));
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
        self.ram.write_u16(0x90, oam);
        if self.ram.sprite(k).pause() == 0 {
            self.yellow_stalfos_draw_head(k);
            self.runtime.draw_shadow(self.ram, k);
        }
    }
    pub fn yellow_stalfos_draw_head(&mut self, k: usize) {
        if self.ram.sprite(k).graphics() == 10 || self.ram.sprite(k).scratch_b() == 0x80 {
            return;
        }
        let j = self.ram.sprite(k).head_direction() as usize;
        let ch = [2u16, 2, 0, 4][j];
        let fl = [0x4000u16, 0, 0, 0][j];
        self.runtime.draw_multiple(
            self.ram,
            k,
            &[(
                self.ram.sprite(k).signed_field(SpriteField::ScratchB),
                self.ram
                    .sprite(k)
                    .signed_field(SpriteField::ScratchC)
                    .wrapping_neg(),
                ch | fl,
                2,
            )],
        );
    }

    pub fn arrghus_draw(&mut self, k: usize) {
        let add = self.ram.sprite(k).graphics() as u16 * 2;
        let mut d: [DrawPiece; 5] = [
            (-8, -4, 0x0080, 2),
            (8, -4, 0x4080, 2),
            (-8, 12, 0x00a0, 2),
            (8, 12, 0x40a0, 2),
            (0, 24, 0x00a8, 2),
        ];
        for p in &mut d[..4] {
            p.2 = p.2.wrapping_add(add);
        }
        let turbulent = self.ram.sprite(k).ai_state() == 5;
        let oam = usize::from(self.ram.read_u16(0x90));
        self.runtime.draw_multiple(self.ram, k, &d);
        if turbulent {
            self.ram.write_u8(oam + 4 * 4 + 1, 0xf0);
        }
        if self.ram.sprite(k).subtype2() & 8 != 0 {
            self.ram
                .write_u8(oam + 4 * 4 + 3, self.ram.read_u8(oam + 4 * 4 + 3) | 0x40);
        }
        if !turbulent {
            self.ram
                .write_u16(0x90, self.ram.read_u16(0x90).wrapping_add(4));
            self.ram
                .write_u16(0x92, self.ram.read_u16(0x92).wrapping_add(1));
            if self.ram.sprite(k).z() < 0xa0 {
                let f = self.ram.sprite(k).oam_flags();
                self.ram.set_sprite_oam_flags(k, f & !1);
                self.runtime.draw_big_shadow(self.ram, k);
                self.ram.set_sprite_oam_flags(k, f);
            }
        } else {
            self.runtime.draw_large_water_turbulence(self.ram, k);
        }
    }

    pub fn sprite_a3_kholdstare_shell(&mut self, k: usize) {
        if self.runtime.return_if_paused(self.ram, k) {
            return;
        }
        self.runtime.direction_to_face_link(self.ram, k);
        let dx = self
            .ram
            .read_u8(LINK_X)
            .wrapping_sub(self.ram.sprite(k).x_low());
        let dy = self
            .ram
            .read_u8(LINK_Y)
            .wrapping_add(8)
            .wrapping_add(self.ram.sprite(k).z())
            .wrapping_sub(self.ram.sprite(k).y_low());
        if dx.wrapping_add(32) < 64 && dy.wrapping_add(32) < 64 {
            self.runtime.nullify_hookshot_drag(self.ram, k);
            self.runtime.repel_dash(self.ram, k);
        }
        self.runtime.check_damage_from_link(self.ram, k);
        if self.ram.sprite(k).ai_state() == 0 {
            if self.ram.sprite(k).state() == 6 {
                self.ram.set_sprite_flags3(k, 0xc0);
                self.ram.set_sprite_ai_state(k, 1);
                self.ram.set_sprite_state(k, 9);
            } else if self.ram.sprite(k).hit_timer() != 0 {
                let x = if self.ram.sprite(k).hit_timer() & 2 != 0 {
                    u16::MAX
                } else {
                    1
                };
                self.ram.write_u16(DUNG_FLOOR_X_OFFS, x);
                self.ram.write_u8(DUNG_HDR_COLLISION_2_MIRROR, 1);
            } else {
                self.ram.write_u8(DUNG_HDR_COLLISION_2_MIRROR, 0);
            }
        } else {
            let old = self.ram.sprite(k).ai_state();
            self.ram.set_sprite_ai_state(k, old.wrapping_add(1));
            if old != 18 {
                self.runtime.kholdstare_shell_palette_filtering(self.ram);
            } else {
                self.ram.set_sprite_state(k, 0);
                self.ram.set_sprite_ai_state(2, 2);
                self.ram.set_sprite_main_delay(2, 128);
            }
        }
    }

    pub fn sprite_draw_stalfos_knight_head(&mut self, k: usize) {
        if self.ram.sprite(k).graphics() == 2 {
            return;
        }
        let i = self.ram.sprite(k).head_direction() as usize;
        let ch = [0x66u16, 0x66, 0x46, 0x46][i];
        let fl = [0x4000u16, 0, 0, 0][i];
        let y = self
            .ram
            .sprite(k)
            .signed_field(SpriteField::ScratchC)
            .wrapping_sub(12);
        self.runtime
            .draw_multiple(self.ram, k, &[(0, y, ch | fl, 2)]);
    }
    pub fn stalfos_knight_draw(&mut self, k: usize) {
        const D: [DrawPiece; 35] = [
            (-4, -8, 0x0064, 0),
            (-4, 0, 0x0061, 2),
            (4, 0, 0x0062, 2),
            (-3, 16, 0x0074, 0),
            (11, 16, 0x4074, 0),
            (-4, -7, 0x0064, 0),
            (-4, 1, 0x0061, 2),
            (4, 1, 0x0062, 2),
            (-3, 16, 0x0065, 0),
            (11, 16, 0x4065, 0),
            (-4, -8, 0x0048, 2),
            (4, -8, 0x0049, 2),
            (-4, 8, 0x004b, 2),
            (4, 8, 0x004c, 2),
            (4, 8, 0x004c, 2),
            (-4, 8, 0x0068, 2),
            (4, 8, 0x0069, 2),
            (4, 8, 0x0069, 2),
            (4, 8, 0x0069, 2),
            (4, 8, 0x0069, 2),
            (12, -7, 0x4064, 0),
            (-4, 1, 0x4062, 2),
            (4, 1, 0x4061, 2),
            (-3, 16, 0x0065, 0),
            (11, 16, 0x4065, 0),
            (12, -8, 0x4064, 0),
            (-4, 0, 0x4062, 2),
            (4, 0, 0x4061, 2),
            (-3, 16, 0x0074, 0),
            (11, 16, 0x4074, 0),
            (-4, -8, 0x4049, 2),
            (4, -8, 0x4048, 2),
            (-4, 8, 0x404c, 2),
            (4, 8, 0x404b, 2),
            (4, 8, 0x404b, 2),
        ];
        self.sprite_draw_stalfos_knight_head(k);
        let i = self.ram.sprite(k).graphics() as usize * 5;
        self.ram
            .write_u16(0x90, self.ram.read_u16(0x90).wrapping_add(4));
        self.ram
            .write_u16(0x92, self.ram.read_u16(0x92).wrapping_add(1));
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 5]);
        self.ram
            .write_u16(0x90, self.ram.read_u16(0x90).wrapping_sub(4));
        self.ram
            .write_u16(0x92, self.ram.read_u16(0x92).wrapping_sub(1));
        self.runtime.draw_shadow_custom(self.ram, k, 18);
    }

    pub fn flute_boy_ostrich_draw(&mut self, k: usize) {
        const D: [DrawPiece; 16] = [
            (-4, -8, 0x0080, 2),
            (4, -8, 0x0081, 2),
            (-4, 8, 0x00a3, 2),
            (4, 8, 0x00a4, 2),
            (-4, -8, 0x0080, 2),
            (4, -8, 0x0081, 2),
            (-4, 8, 0x00a0, 2),
            (4, 8, 0x00a1, 2),
            (-4, -8, 0x0080, 2),
            (4, -8, 0x0081, 2),
            (-4, 8, 0x0083, 2),
            (4, 8, 0x0084, 2),
            (-4, -7, 0x0080, 2),
            (4, -7, 0x0081, 2),
            (-4, 9, 0x00a3, 2),
            (4, 9, 0x00a4, 2),
        ];
        let i = self.ram.sprite(k).graphics() as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        self.runtime.draw_shadow_custom(self.ram, k, 18);
    }

    pub fn haunted_grove_bird_blink(&mut self, k: usize) {
        const X: [i8; 2] = [8, 0];
        let d = self.ram.sprite(k).scratch_d() as usize;
        let flags = if d == 0 { 0x40 } else { 0 };
        let piece = [(X[d], 0, 0x00ae | ((flags as u16) << 8), 0)];
        self.runtime.draw_multiple(self.ram, k, &piece);
    }

    pub fn sprite_a1_freezor(&mut self, k: usize) {
        self.runtime.freezor_draw(self.ram, k);
        if self.ram.sprite(k).state() != 9 {
            self.ram.set_sprite_ai_state(k, 3);
            self.ram.set_sprite_main_delay(k, 31);
            self.ram.set_sprite_ignore_projectile(k, 31);
            self.ram.set_sprite_state(k, 9);
            self.ram.set_sprite_hit_timer(k, 0);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).ai_state() != 3 && self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_ignore_projectile(
                    k,
                    self.ram.sprite(k).ignore_projectile().wrapping_add(1),
                );
                let dx = self
                    .ram
                    .read_u8(LINK_X)
                    .wrapping_sub(self.ram.sprite(k).x_low());
                if dx.wrapping_add(16) < 32 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_main_delay(k, 32);
                }
            }
            1 => {
                let t = self.ram.sprite(k).main_delay();
                self.ram.set_sprite_ignore_projectile(k, t);
                if t == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.runtime.update_common_tile(
                        self.ram,
                        self.sprite_x(k).wrapping_sub(5),
                        self.sprite_y(k),
                        8,
                    );
                    self.ram.set_sprite_aux1_delay(k, 96);
                    self.ram.set_sprite_scratch_d(k, 2);
                    self.ram.set_sprite_main_delay(k, 80);
                } else {
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::VelocityX,
                        if t & 1 != 0 { -16 } else { 16 },
                    );
                    self.runtime.move_x(self.ram, k);
                }
            }
            2 => {
                const X: [i8; 4] = [8, -8, 0, 0];
                const Y: [i8; 4] = [0, 0, 18, -18];
                const G: [u8; 4] = [1, 2, 1, 3];
                const SX: [i8; 8] = [-4, -2, 0, 2, 4, 6, 8, 10];
                self.runtime.check_damage_to_link(self.ram, k);
                if self.runtime.check_damage_from_link(self.ram, k) != 0 {
                    self.ram.set_sprite_hit_timer(k, 0);
                }
                if self.ram.sprite(k).aux1_delay() != 0
                    && ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 7 == 0
                {
                    let i = self.runtime.random(self.ram) as usize & 7;
                    self.runtime.spawn_sparkle(self.ram, k, SX[i], -4);
                }
                if self.ram.sprite(k).main_delay() == 0 {
                    let d = self.runtime.direction_to_face_link(self.ram, k);
                    self.ram.set_sprite_scratch_d(k, d);
                }
                let d = self.ram.sprite(k).scratch_d() as usize;
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, X[d]);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, Y[d]);
                if self.ram.sprite(k).wall_collision() & 15 == 0 {
                    self.runtime.move_xy(self.ram, k);
                }
                self.runtime.check_tile_collision(self.ram, k);
                self.ram.set_sprite_graphics(
                    k,
                    G[(((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) >> 2 & 3) as usize],
                );
            }
            3 => {
                let t = self.ram.sprite(k).main_delay();
                if t == 0 {
                    self.runtime.manually_set_death_flag_uw(self.ram, k);
                    self.ram.set_sprite_state(k, 0);
                }
                self.ram
                    .set_sprite_graphics(k, [6, 5, 4, 7][(t >> 3) as usize]);
            }
            _ => {}
        }
    }
    pub fn sprite_a0_haunted_grove_bird(&mut self, k: usize) {
        if self.ram.sprite(k).graphics() == 3 {
            self.haunted_grove_bird_blink(k);
        }
        const FLAGS: [u8; 4] = [0x40, 0, 0, 0];
        const XV: [i8; 4] = [16, -16, 0, 0];
        let d = self.ram.sprite(k).scratch_d() as usize;
        let f = self.ram.sprite(k).oam_flags() & !0x40 | FLAGS[d];
        self.ram.set_sprite_oam_flags(k, f);
        // The bird reserves an OAM entry even when its separate blink sprite
        // is not emitted, so the following large sprite lands in the next slot.
        self.ram
            .write_u16(0x90, self.ram.read_u16(0x90).wrapping_add(4));
        self.ram
            .write_u16(0x92, self.ram.read_u16(0x92).wrapping_add(1));
        self.ram
            .set_sprite_flags2(k, self.ram.sprite(k).flags2().wrapping_sub(1));
        self.runtime.draw_single_large(self.ram, k);
        self.ram
            .set_sprite_flags2(k, self.ram.sprite(k).flags2().wrapping_add(1));
        self.runtime.move_xyz(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_graphics(
                    k,
                    if self.ram.read_u8(FRAME_COUNTER) & 0x18 != 0 {
                        0
                    } else {
                        3
                    },
                );
                if self.ram.read_u8(BYTE_7E0FDD) != 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_scratch_d(k, (d as u8) ^ 1);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, XV[d ^ 1]);
                    self.ram.set_sprite_main_delay(k, 32);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 16);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, -8);
                }
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    let z = self
                        .ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityZ)
                        .wrapping_add(2);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, z);
                    if z >= 16 {
                        self.ram.set_sprite_ai_state(k, 2);
                    }
                }
                let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(k, s);
                self.ram.set_sprite_graphics(k, (s >> 1 & 1) + 1);
            }
            2 => {
                self.ram.set_sprite_graphics(k, 1);
                let z = self
                    .ram
                    .sprite(k)
                    .signed_field(SpriteField::VelocityZ)
                    .wrapping_sub(1);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, z);
                if z < -15 {
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            _ => {}
        }
    }

    pub fn gold_bee_spawn_self(&mut self, k: usize) {
        const VEL: [i8; 8] = [15, 5, -5, -15, 20, 10, -10, -20];
        if let Some((j, x, y, z)) = self.runtime.spawn_dynamic(self.ram, k, 0x79) {
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            self.ram.set_sprite_z(j, z);
            self.ram.set_sprite_ai_state(j, 1);
            self.ram.set_sprite_main_delay(j, 64);
            self.ram.set_sprite_scratch_a(j, 64);
            self.ram.set_sprite_aux4_delay(j, 96);
            self.ram.set_sprite_head_direction(j, 1);
            let a = self.runtime.random(self.ram) as usize & 7;
            let b = self.runtime.random(self.ram) as usize & 7;
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityX, VEL[a]);
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityY, VEL[b]);
        }
    }

    pub fn mothula_handle_spikes(&mut self, k: usize) {
        const X: [u8; 30] = [
            0x38, 0x48, 0x58, 0x68, 0x88, 0x98, 0xa8, 0xb8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8,
            0xc8, 0xb8, 0xa8, 0x98, 0x78, 0x68, 0x58, 0x48, 0x38, 0x28, 0x28, 0x28, 0x28, 0x28,
            0x28, 0x28,
        ];
        const Y: [u8; 30] = [
            0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x48, 0x58, 0x68, 0x78, 0x98, 0xa8,
            0xb8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xb8, 0xa8, 0x98, 0x78, 0x68,
            0x58, 0x48,
        ];
        const DIR: [u8; 30] = [
            2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0,
            0,
        ];
        let timer = self.ram.sprite(k).head_direction().wrapping_sub(1);
        self.ram.set_sprite_head_direction(k, timer);
        if timer != 0 {
            return;
        }
        self.ram.set_sprite_head_direction(k, 0x40);
        let Some((j, _, _, _)) = self.runtime.spawn_dynamic(self.ram, k, 0x8a) else {
            return;
        };
        let mut i = self.runtime.random(self.ram) as usize & 0x1f;
        if i >= 30 {
            i -= 30;
        }
        self.ram.set_sprite_scratch_a(j, X[i]);
        self.ram.set_sprite_x_low(j, X[i]);
        self.ram.set_sprite_scratch_b(j, Y[i].wrapping_sub(1));
        self.ram.set_sprite_y_low(j, Y[i].wrapping_sub(1));
        self.ram.set_sprite_scratch_d(j, DIR[i]);
        self.ram.set_sprite_scratch_e(j, 1);
        self.ram
            .set_sprite_x_high(j, self.ram.read_u8(0xfb0).wrapping_add(1));
        self.ram
            .set_sprite_y_high(j, self.ram.read_u8(0xfb1).wrapping_add(1));
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityX, 1);
        self.runtime.get_16bit_coords(self.ram, j);
        self.runtime.check_tile_collision(self.ram, j);
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityX, 0);
        self.ram.set_sprite_x_low(j, X[i]);
        self.ram.set_sprite_y_low(j, Y[i].wrapping_sub(1));
        if self.ram.sprite(j).wall_collision() == 0 {
            self.ram.set_sprite_state(j, 0);
            self.ram.set_sprite_head_direction(k, 1);
        }
    }

    pub fn pink_ball_distress(&mut self, k: usize) {
        self.runtime.draw_distress(self.ram, k);
    }

    pub fn sprite_pink_ball(&mut self, k: usize) {
        self.runtime.allocate_oam_defer_to_player(self.ram, k);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.pink_ball_handle_message(k);
        self.ram.set_sprite_oam_flags(
            k,
            self.ram.sprite(k).oam_flags() & !0x80 | self.ram.sprite(k).head_direction(),
        );
        self.runtime.move_xyz(self.ram, k);
        let t = self.runtime.check_tile_collision(self.ram, k);
        if t != 0 {
            let mut sound = false;
            if t & 3 == 0 {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityY,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityY)
                        .wrapping_neg(),
                );
                sound = self.ram.sprite(k).scratch_e() != 0;
            }
            // C jumps straight to play_sound after a vertical bounce when E
            // is set, leaving x velocity untouched.
            if !sound {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_neg(),
                );
            }
            if self.ram.sprite(k).scratch_e() != 0 {
                sound = true;
            }
            if sound {
                self.ball_guy_play_bounce_noise(k);
            }
        }
        let zv = self
            .ram
            .sprite(k)
            .signed_field(SpriteField::VelocityZ)
            .wrapping_sub(1);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityZ, zv);
        if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_z(k, 0);
            let z = (zv.wrapping_neg() as u8) >> 2;
            self.ram.set_sprite_velocity_z(k, z);
            if z & 0xfc != 0 {
                self.ball_guy_play_bounce_noise(k);
            }
            self.pink_ball_handle_deceleration(k);
        }
        let f = (k as u8) ^ self.ram.read_u8(FRAME_COUNTER);
        if self.ram.sprite(k).scratch_e() == 0 {
            if self.ram.sprite(k).head_direction() == 0 {
                self.pink_ball_distress(k);
                self.ram.set_sprite_graphics(k, f >> 3 & 1);
                if f & 0x3f == 0 {
                    let x =
                        (self.ram.read_u16(LINK_X) & 0xff00) | self.runtime.random(self.ram) as u16;
                    let y =
                        (self.ram.read_u16(LINK_Y) & 0xff00) | self.runtime.random(self.ram) as u16;
                    let (yv, xv) = self.runtime.project_speed(self.ram, k, x, y, 8);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::ScratchB, xv);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::ScratchA, yv);
                    if yv != 0 {
                        let q = (self.ram.sprite(k).oam_flags() | 64)
                            ^ (self.ram.sprite(k).velocity_x() >> 1 & 64);
                        self.ram.set_sprite_oam_flags(k, q);
                    }
                }
                self.ram
                    .set_sprite_velocity_x(k, self.ram.sprite(k).scratch_b());
                self.ram
                    .set_sprite_velocity_y(k, self.ram.sprite(k).scratch_a());
            } else {
                self.pink_ball_distress(k);
                if f != 0 {
                    self.ram.set_sprite_graphics(k, f >> 2 & 1);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
                } else {
                    self.ram.set_sprite_head_direction(k, 0);
                }
            }
        } else if self.ram.sprite(k).velocity_x() | self.ram.sprite(k).velocity_y() == 0 {
            self.ram.set_sprite_scratch_e(k, 0);
        } else {
            self.ram.set_sprite_graphics(k, f >> 2 & 1);
            self.ram.set_sprite_head_direction(k, f << 2 & 128);
        }
    }

    pub fn bully_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (0, -7, 0x46e0, 2),
            (0, 0, 0x46e2, 2),
            (0, -7, 0x46e0, 2),
            (0, 0, 0x46c4, 2),
            (0, -7, 0x06e0, 2),
            (0, 0, 0x06e2, 2),
            (0, -7, 0x06e0, 2),
            (0, 0, 0x06c4, 2),
        ];
        let i = self.ram.sprite(k).scratch_d() as usize * 4
            + self.ram.sprite(k).graphics() as usize * 2;
        self.runtime.allocate_oam_defer_to_player(self.ram, k);
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_bully(&mut self, k: usize) {
        self.bully_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.bully_handle_message(k);
        self.runtime.move_xyz(self.ram, k);
        let t = self.runtime.check_tile_collision(self.ram, k);
        if t != 0 {
            if t & 3 == 0 {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityY,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityY)
                        .wrapping_neg(),
                );
            } else {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_neg(),
                );
            }
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram
                    .set_sprite_graphics(k, ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) >> 3 & 1);
                let j = self.ram.sprite(k).head_direction() as usize;
                if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 0x1f == 0 {
                    let (y, x) = self.runtime.project_speed(
                        self.ram,
                        k,
                        self.sprite_x(j),
                        self.sprite_y(j),
                        14,
                    );
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, y);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, x);
                    if x != 0 {
                        self.ram.set_sprite_scratch_d(k, (x as u8) >> 7);
                    }
                }
                if self.ram.sprite(j).z() == 0
                    && self
                        .ram
                        .sprite(k)
                        .x_low()
                        .wrapping_sub(self.ram.sprite(j).x_low())
                        .wrapping_add(8)
                        < 16
                    && self
                        .ram
                        .sprite(k)
                        .y_low()
                        .wrapping_sub(self.ram.sprite(j).y_low())
                        .wrapping_add(8)
                        < 16
                {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ball_guy_play_bounce_noise(k);
                }
            }
            1 => {
                self.ram.set_sprite_ai_state(k, 2);
                let j = self.ram.sprite(k).head_direction() as usize;
                self.ram.set_sprite_signed_field(
                    j,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_mul(2),
                );
                self.ram.set_sprite_signed_field(
                    j,
                    SpriteField::VelocityY,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityY)
                        .wrapping_mul(2),
                );
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
                let z = self.runtime.random(self.ram) & 31;
                self.ram.set_sprite_velocity_z(j, z);
                self.ram.set_sprite_main_delay(k, 96);
                self.ram.set_sprite_graphics(k, 1);
                self.ram.set_sprite_scratch_e(j, 1);
            }
            2 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            _ => {}
        }
    }

    pub fn bomb_shop_entity_draw(&mut self, k: usize) {
        const D: [DrawPiece; 6] = [
            (0, 0, 0x0a48, 2),
            (0, 0, 0x0a4c, 2),
            (0, 0, 0x04c2, 2),
            (0, 0, 0x04c2, 2),
            (0, 0, 0x084e, 2),
            (0, 0, 0x084e, 2),
        ];
        let i = self.ram.sprite(k).subtype2() as usize * 2 + self.ram.sprite(k).graphics() as usize;
        self.runtime
            .draw_multiple_player_deferred_with_shadow(self.ram, k, &D[i..i + 1]);
    }
    pub fn kiki_draw(&mut self, k: usize) -> bool {
        const DMA: [u8; 32] = [
            0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40,
            0x60, 0, 0, 0xfa, 0xff, 0x20, 0, 0, 2, 0, 0, 0, 0, 0x22, 0, 0, 2,
        ];
        const A: [DrawPiece; 32] = [
            (0, -6, 0x0020, 2),
            (0, 0, 0x0022, 2),
            (0, -6, 0x0020, 2),
            (0, 0, 0x4022, 2),
            (0, -6, 0x0020, 2),
            (0, 0, 0x0022, 2),
            (0, -6, 0x0020, 2),
            (0, 0, 0x4022, 2),
            (-1, -6, 0x0020, 2),
            (0, 0, 0x0022, 2),
            (-1, -6, 0x0020, 2),
            (0, 0, 0x0022, 2),
            (1, -6, 0x4020, 2),
            (0, 0, 0x4022, 2),
            (1, -6, 0x4020, 2),
            (0, 0, 0x4022, 2),
            (0, -6, 0x01ce, 2),
            (0, 0, 0x01ee, 2),
            (0, -6, 0x01ce, 2),
            (0, 0, 0x01ee, 2),
            (0, -6, 0x41ce, 2),
            (0, 0, 0x41ee, 2),
            (0, -6, 0x41ce, 2),
            (0, 0, 0x41ee, 2),
            (-1, -6, 0x01ce, 2),
            (0, 0, 0x01ec, 2),
            (-1, -6, 0x41ce, 2),
            (0, 0, 0x01ec, 2),
            (1, -6, 0x41ce, 2),
            (0, 0, 0x41ec, 2),
            (1, -6, 0x01ce, 2),
            (0, 0, 0x41ec, 2),
        ];
        const B: [DrawPiece; 12] = [
            (0, -6, 0x01ca, 0),
            (8, -6, 0x41ca, 0),
            (0, 2, 0x01da, 0),
            (8, 2, 0x41da, 0),
            (0, 10, 0x01cb, 0),
            (8, 10, 0x41cb, 0),
            (0, -6, 0x01db, 0),
            (8, -6, 0x41db, 0),
            (0, 2, 0x01cc, 0),
            (8, 2, 0x41cc, 0),
            (0, 10, 0x01dc, 0),
            (8, 10, 0x41dd, 0),
        ];
        let d = self.ram.sprite(k).scratch_d();
        if d < 8 {
            let j = d as usize * 2 + self.ram.sprite(k).graphics() as usize;
            self.ram.write_u8(0xae8, DMA[j * 2]);
            self.ram.write_u8(0xaea, DMA[j * 2 + 1]);
            self.runtime
                .draw_multiple(self.ram, k, &A[j * 2..j * 2 + 2]);
        } else {
            let i = self.ram.sprite(k).graphics() as usize * 6;
            self.runtime.draw_multiple(self.ram, k, &B[i..i + 6]);
        }
        if self.ram.sprite(k).pause() == 0 {
            self.runtime.draw_shadow(self.ram, k);
        }
        let x = self
            .ram
            .read_u16(CUR_SPRITE_X)
            .wrapping_sub(self.ram.read_u16(0xe2));
        let y = self
            .ram
            .read_u16(CUR_SPRITE_Y)
            .wrapping_sub(self.ram.read_u16(0xe8))
            .wrapping_sub(u16::from(self.ram.sprite(k).z()));
        (x | y) & 0xff00 != 0
    }

    pub fn kiki_flee(&mut self, k: usize) {
        let mut flag = self.kiki_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).z() == 0
            && self.ram.read_u16(CUR_SPRITE_X).wrapping_sub(0xc98) < 0xd0
            && self.ram.read_u16(CUR_SPRITE_Y).wrapping_sub(0x6a5) < 0xd0
        {
            flag = true;
        }
        if flag {
            self.ram.set_sprite_state(k, 0);
        }
        self.ram.set_sprite_signed_field(
            k,
            SpriteField::VelocityZ,
            self.ram
                .sprite(k)
                .signed_field(SpriteField::VelocityZ)
                .wrapping_sub(2),
        );
        self.runtime.move_xyz(self.ram, k);
        if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_z(k, 0);
            let z = self.runtime.random(self.ram) & 15 | 16;
            self.ram.set_sprite_velocity_z(k, z);
        }
        let (yv, xv) = self.runtime.project_speed(self.ram, k, 0xcf5, 0x6fe, 16);
        let xx = xv.wrapping_mul(2);
        let yy = yv.wrapping_mul(2);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityX, xx);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityY, yy);
        self.ram.write_u8(
            TAGALONG_EVENT_FLAGS,
            self.ram.read_u8(TAGALONG_EVENT_FLAGS) & !3,
        );
        let ax = xv.unsigned_abs();
        let ay = yv.unsigned_abs();
        self.ram.set_sprite_scratch_d(
            k,
            if ax >= ay {
                ((xx as u8) >> 7) ^ 3
            } else {
                ((yy as u8) >> 7) ^ 1
            },
        );
        self.ram
            .set_sprite_graphics(k, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
    }

    pub fn kiki_offer_initial_service(&mut self, k: usize) {
        if self.ram.sprite(k).ai_state() >= 2 {
            self.kiki_draw(k);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.move_xyz(self.ram, k);
        self.ram.set_sprite_signed_field(
            k,
            SpriteField::VelocityZ,
            self.ram
                .sprite(k)
                .signed_field(SpriteField::VelocityZ)
                .wrapping_sub(1),
        );
        if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
            self.ram.set_sprite_z(k, 0);
        }
        self.ram
            .set_sprite_graphics(k, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.runtime.show_message(self.ram, 0x11e);
                self.ram.set_sprite_ai_state(k, 1);
            }
            1 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0
                    && self.shop_item_handle_cost(10)
                {
                    self.runtime.show_message(self.ram, 0x11f);
                    self.ram.write_u8(
                        TAGALONG_EVENT_FLAGS,
                        self.ram.read_u8(TAGALONG_EVENT_FLAGS) | 3,
                    );
                    self.ram.set_sprite_state(k, 0);
                } else {
                    self.runtime.show_message(self.ram, 0x120);
                    self.ram.write_u8(
                        TAGALONG_EVENT_FLAGS,
                        self.ram.read_u8(TAGALONG_EVENT_FLAGS) & !3,
                    );
                    self.ram.write_u8(FOLLOWER_INDICATOR, 0);
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.write_u8(
                        FLAG_IS_LINK_IMMOBILIZED,
                        self.ram.read_u8(FLAG_IS_LINK_IMMOBILIZED).wrapping_add(1),
                    );
                }
            }
            2 => {
                self.ram.set_sprite_ai_state(k, 3);
                let (y, x) = self.runtime.project_speed(self.ram, k, 0xc45, 0x6fe, 9);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, y);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, x);
                self.ram.set_sprite_scratch_d(k, ((x as u8) >> 7) ^ 3);
                self.ram.set_sprite_main_delay(k, 32);
            }
            3 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 4);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 16);
                    self.ram.set_sprite_main_delay(k, 16);
                }
            }
            4 => {
                if self.ram.sprite(k).main_delay() == 0 && self.ram.sprite(k).z() == 0 {
                    self.ram.set_sprite_state(k, 0);
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                }
            }
            _ => {}
        }
    }

    pub fn kiki_offer_entrance_service(&mut self, k: usize) {
        self.kiki_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.move_xyz(self.ram, k);
        self.ram.set_sprite_signed_field(
            k,
            SpriteField::VelocityZ,
            self.ram
                .sprite(k)
                .signed_field(SpriteField::VelocityZ)
                .wrapping_sub(1),
        );
        if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
            self.ram.set_sprite_z(k, 0);
        }
        let state = self.ram.sprite(k).ai_state();
        match state {
            0 => {
                self.runtime.show_message(self.ram, 0x11b);
                self.ram.set_sprite_ai_state(k, 1);
            }
            1 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) != 0
                    || !self.shop_item_handle_cost(100)
                {
                    self.runtime.show_message(self.ram, 0x11c);
                    self.ram.set_sprite_subtype2(k, 3);
                } else {
                    self.runtime.show_message(self.ram, 0x11d);
                    self.ram.write_u8(
                        FLAG_IS_LINK_IMMOBILIZED,
                        self.ram.read_u8(FLAG_IS_LINK_IMMOBILIZED).wrapping_add(1),
                    );
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_scratch_d(k, 0);
                }
            }
            2 | 4 | 6 => {
                const X: [u16; 3] = [0xf4f, 0xf70, 0xf5d];
                const Y: [u16; 3] = [0x661, 0x64c, 0x624];
                self.ram
                    .set_sprite_graphics(k, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
                let j = (state as usize / 2) - 1;
                if (X[j]
                    .wrapping_sub(self.ram.sprite(k).x_low() as u16)
                    .wrapping_add(2) as u8)
                    < 4
                    && (Y[j]
                        .wrapping_sub(self.ram.sprite(k).y_low() as u16)
                        .wrapping_add(2) as u8)
                        < 4
                {
                    self.ram.set_sprite_ai_state(k, state + 1);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
                    self.ram.set_sprite_aux1_delay(k, 32);
                    self.runtime.queue_sfx2(self.ram, k, 0x21);
                } else {
                    let (y, x) = self.runtime.project_speed(self.ram, k, X[j], Y[j], 9);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, x);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, y);
                }
            }
            3 | 5 => {
                if self.ram.sprite(k).aux1_delay() == 0 {
                    let z = [32, 28][((state >> 1) & 1) as usize];
                    self.ram.set_sprite_velocity_z(k, z);
                    self.ram.set_sprite_ai_state(k, state + 1);
                    self.runtime.queue_sfx2(self.ram, k, 0x20);
                    self.ram.set_sprite_scratch_d(k, ((state + 1) >> 1 & 1) | 4);
                } else {
                    self.ram.set_sprite_scratch_d(k, (state >> 1 & 1) | 6);
                    self.ram
                        .set_sprite_graphics(k, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
                }
            }
            7 => {
                self.ram
                    .set_sprite_graphics(k, self.ram.read_u8(FRAME_COUNTER) >> 3 & 1);
                if self.ram.sprite(k).z() != 0 || self.ram.sprite(k).main_delay() != 0 {
                    return;
                }
                let j = self.ram.sprite(k).scratch_a() as usize;
                self.ram.set_sprite_scratch_a(k, (j as u8).wrapping_add(1));
                let t = [2u8, 1, 0xff][j];
                if t as i8 >= 0 {
                    self.ram.set_sprite_scratch_d(k, t);
                    self.ram.set_sprite_main_delay(k, [82, 0][j]);
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::VelocityX,
                        [0, 0, -9, 9][t as usize],
                    );
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::VelocityY,
                        [-9, 9, 0, 0][t as usize],
                    );
                } else {
                    self.ram.set_sprite_ai_state(k, 8);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
                    self.ram.write_u8(TRIGGER_SPECIAL_ENTRANCE, 1);
                    self.ram.write_u8(SUBSUBMODULE_INDEX, 0);
                    self.ram.write_u8(OVERWORLD_ENTRANCE_SEQUENCE_COUNTER, 0);
                    self.ram.set_sprite_scratch_d(k, 0);
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                }
            }
            8 => {
                self.ram.set_sprite_scratch_d(k, 8);
                self.ram.set_sprite_graphics(k, 0);
                let z = (self.runtime.random(self.ram) & 15) + 16;
                self.ram.set_sprite_velocity_z(k, z);
                self.ram.set_sprite_ai_state(k, 9);
            }
            9 => {
                if self.ram.sprite(k).signed_field(SpriteField::VelocityZ) < 0
                    && self.ram.sprite(k).z() == 0
                {
                    self.ram.set_sprite_ai_state(k, 10);
                    self.runtime.queue_sfx3(self.ram, k, 0x25);
                }
            }
            _ => {}
        }
    }

    pub fn agahnim_perform_attack(&mut self, k: usize) {
        const X: [i8; 6] = [0, 10, 8, 0, -10, -10];
        const Y: [i8; 6] = [-9, -2, -2, -9, -2, -2];
        if k == 0 {
            let s = self.ram.sprite(k).subtype().wrapping_add(1);
            self.ram.set_sprite_subtype(
                k,
                if self.ram.read_u8(IS_IN_DARK_WORLD) != 0 {
                    s & 3
                } else {
                    s
                },
            );
        }
        if self.ram.sprite(k).subtype() == 5 {
            self.ram.set_sprite_subtype(k, 0);
            self.runtime.queue_sfx3(self.ram, k, 0x26);
            for _ in 0..4 {
                self.runtime.spawn_lightning(self.ram, k);
            }
        } else if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0x7b) {
            self.runtime.queue_sfx3(self.ram, k, 0x29);
            let i = self.ram.sprite(k).scratch_d() as usize;
            self.set_sprite_x(j, x.wrapping_add_signed(X[i] as i16));
            self.set_sprite_y(j, y.wrapping_add_signed(Y[i] as i16));
            self.ram
                .set_sprite_ignore_projectile(j, self.ram.sprite(j).y_high());
            self.ram
                .set_sprite_velocity_x(j, self.ram.sprite(k).velocity_x());
            self.ram
                .set_sprite_velocity_y(j, self.ram.sprite(k).velocity_y());
            if self.ram.sprite(k).subtype() >= 2 && self.runtime.random(self.ram) & 1 == 0 {
                self.ram.set_sprite_scratch_b(j, 1);
                self.ram.set_sprite_main_delay(j, 32);
            }
        }
    }
    pub fn create_six_blue_balls(&mut self, k: usize) {
        const XV: [i8; 6] = [0, 24, 24, 0, -24, -24];
        const YV: [i8; 6] = [-32, -16, 16, 32, 16, -16];
        self.runtime.queue_sfx3(self.ram, k, 0x36);
        self.ram.write_u8(TMP_COUNTER, 5);
        loop {
            let i = self.ram.read_u8(TMP_COUNTER) as usize;
            if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0x55) {
                self.set_sprite_x(j, x.wrapping_add(4));
                self.set_sprite_y(j, y.wrapping_add(4));
                self.ram
                    .set_sprite_flags3(j, self.ram.sprite(j).flags3() & !1 | 0x40);
                self.ram.set_sprite_oam_flags(j, 4);
                self.ram.set_sprite_aux1_delay(j, 4);
                self.ram.set_sprite_flags4(j, 20);
                self.ram.set_sprite_scratch_c(j, 20);
                self.ram.set_sprite_scratch_e(j, 20);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityX, XV[i]);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityY, YV[i]);
            }
            let n = self.ram.read_u8(TMP_COUNTER).wrapping_sub(1);
            self.ram.write_u8(TMP_COUNTER, n);
            if (n as i8) < 0 {
                break;
            }
        }
        self.ram.write_u8(TMP_COUNTER, 0);
    }

    pub fn sprite_7d_big_spike(&mut self, k: usize) {
        const XV: [i8; 4] = [32, -32, 0, 0];
        const YV: [i8; 4] = [0, 0, 32, -32];
        const RX: [i8; 4] = [-16, 16, 0, 0];
        const RY: [i8; 4] = [0, 0, -16, 16];
        const DELAY: [u8; 4] = [0x40, 0x40, 0x38, 0x38];
        self.runtime.spike_trap_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                let d = self.runtime.direction_to_face_link(self.ram, k) as usize;
                self.ram.set_sprite_scratch_d(k, d as u8);
                let dx = self
                    .ram
                    .read_u8(LINK_X)
                    .wrapping_sub(self.ram.sprite(k).x_low());
                let dy = self
                    .ram
                    .read_u8(LINK_Y)
                    .wrapping_add(8)
                    .wrapping_add(self.ram.sprite(k).z())
                    .wrapping_sub(self.ram.sprite(k).y_low());
                if dx.wrapping_add(16) < 32 || dy.wrapping_add(16) < 32 {
                    self.ram.set_sprite_main_delay(k, DELAY[d]);
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, XV[d]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, YV[d]);
                }
            }
            1 => {
                let collided = self.runtime.check_tile_collision(self.ram, k) != 0;
                if collided || self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_main_delay(k, 96);
                }
                self.runtime.move_xy(self.ram, k);
            }
            _ => {
                if self.ram.sprite(k).main_delay() == 0 {
                    let d = self.ram.sprite(k).scratch_d() as usize;
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, RX[d]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, RY[d]);
                    self.runtime.move_xy(self.ram, k);
                    if self.ram.sprite(k).x_low() == self.ram.sprite(k).scratch_a()
                        && self.ram.sprite(k).y_low() == self.ram.sprite(k).scratch_c()
                    {
                        self.ram.set_sprite_ai_state(k, 0);
                    }
                }
            }
        }
    }

    pub fn old_man_revert_to_sprite(&mut self, k: usize) {
        let Some((j, _, _, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xad) else {
            return;
        };
        let d = self.ram.read_u8(TAGALONG_LAYERBITS + k) & 3;
        self.ram.set_sprite_scratch_d(j, d);
        self.ram.set_sprite_head_direction(j, d);
        let y = u16::from_le_bytes([
            self.ram.read_u8(TAGALONG_Y_LO + k),
            self.ram.read_u8(TAGALONG_Y_HI + k),
        ])
        .wrapping_add(2);
        let x = u16::from_le_bytes([
            self.ram.read_u8(TAGALONG_X_LO + k),
            self.ram.read_u8(TAGALONG_X_HI + k),
        ])
        .wrapping_add(2);
        self.set_sprite_y(j, y);
        self.set_sprite_x(j, x);
        self.ram
            .set_sprite_floor(j, self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL));
        self.ram.set_sprite_ignore_projectile(j, 1);
        self.ram.set_sprite_subtype2(j, 1);
        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
        self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 1);
        self.ram.write_u8(FOLLOWER_INDICATOR, 0);
        self.ram.write_u8(LINK_SPEED_SETTING, 0);
    }

    pub fn sprite_bomb_shop_clerk(&mut self, k: usize) {
        const GFX: [u8; 8] = [0, 1, 0, 1, 0, 1, 0, 1];
        const DELAY: [u8; 8] = [255, 32, 255, 24, 15, 24, 255, 15];
        self.bomb_shop_entity_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).main_delay() == 0 {
            let j = self.ram.sprite(k).scratch_e() as usize;
            self.ram
                .set_sprite_scratch_e(k, (j as u8).wrapping_add(1) & 7);
            self.ram.set_sprite_main_delay(k, DELAY[j]);
            self.ram.set_sprite_graphics(k, GFX[j]);
            if GFX[j] == 0 {
                self.runtime.queue_sfx3(self.ram, k, 0x11);
                self.bomb_shop_clerk_exhalation(k);
            } else {
                self.runtime.queue_sfx3(self.ram, k, 0x12);
            }
        }
        let flag = self.ram.read_u8(LINK_HAS_CRYSTALS) & 5 == 5
            && self.ram.read_u8(SRAM_PROGRESS_INDICATOR_3) & 32 != 0;
        self.runtime
            .show_solicited_message(self.ram, k, if flag { 0x118 } else { 0x117 });
        self.runtime.behave_as_barrier(self.ram, k);
    }

    pub fn shopkeeper_standard_clerk(&mut self, k: usize) {
        let dark = self.ram.read_u8(IS_IN_DARK_WORLD) != 0;
        if dark {
            self.runtime.allocate_oam_defer_to_player(self.ram, k);
            self.runtime.draw_single_large(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            let f =
                self.ram.sprite(k).oam_flags() & 63 | (self.ram.read_u8(FRAME_COUNTER) << 3 & 64);
            self.ram.set_sprite_oam_flags(k, f);
        } else {
            self.ram.set_sprite_oam_flags(k, 7);
            self.runtime.shopkeeper_draw(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.ram
                .set_sprite_graphics(k, self.ram.read_u8(FRAME_COUNTER) >> 4 & 1);
        }
        self.runtime.behave_as_barrier(self.ram, k);
        let msg = if dark { 0x15f } else { 0x165 };
        self.runtime.show_solicited_message(self.ram, k, msg);
        if self.ram.sprite(k).ai_state() == 0
            && self.ram.read_u16(CUR_SPRITE_Y).wrapping_add(0x60) >= self.ram.read_u16(LINK_Y)
        {
            self.runtime.show_message(self.ram, msg);
            self.ram.set_sprite_ai_state(k, 1);
        }
    }

    pub fn nice_thief_with_gift(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.runtime.show_solicited_message(self.ram, k, 0x176) & 0x100 != 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 => {
                let bits = self.ram.read_u16(DUNG_SAVEGAME_STATE_BITS);
                if bits & 0x4000 == 0 {
                    self.ram.write_u16(DUNG_SAVEGAME_STATE_BITS, bits | 0x4000);
                    self.ram.set_sprite_ai_state(k, 2);
                    self.shop_item_handle_receipt(k, 0x46);
                } else {
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            2 => self.ram.set_sprite_ai_state(k, 0),
            _ => {}
        }
    }

    fn chest_game_logic(&mut self, k: usize, prompt: u16, cost: u16, credits: u8) {
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.read_u8(MINIGAME_CREDITS).wrapping_sub(1) >= 2
                    && self.runtime.show_solicited_message(self.ram, k, prompt) & 0x100 != 0
                {
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0
                    && self.shop_item_handle_cost(cost)
                {
                    self.ram.write_u8(MINIGAME_CREDITS, credits);
                    self.runtime.show_message(self.ram, 0x17f);
                    self.ram.set_sprite_ai_state(k, 2);
                } else {
                    self.runtime.show_message(self.ram, 0x180);
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            2 => {
                let m = if self.ram.read_u8(MINIGAME_CREDITS) == 0 {
                    0x163
                } else {
                    0x17f
                };
                self.runtime.show_solicited_message(self.ram, k, m);
            }
            _ => {}
        }
    }
    pub fn chest_game_guy(&mut self, k: usize) {
        self.runtime.allocate_oam_defer_to_player(self.ram, k);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        let f = self.ram.sprite(k).oam_flags() & 63 | (self.ram.read_u8(FRAME_COUNTER) << 3 & 64);
        self.ram.set_sprite_oam_flags(k, f);
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.read_u8(MINIGAME_CREDITS).wrapping_sub(1) >= 2
                    && self.runtime.show_solicited_message(self.ram, k, 0x160) & 0x100 != 0
                {
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) == 0
                    && self.shop_item_handle_cost(30)
                {
                    self.ram.write_u8(MINIGAME_CREDITS, 2);
                    self.runtime.show_message(self.ram, 0x164);
                    self.ram.set_sprite_ai_state(k, 2);
                } else {
                    self.runtime.show_message(self.ram, 0x161);
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            2 => {
                let m = if self.ram.read_u8(MINIGAME_CREDITS) == 0 {
                    0x163
                } else {
                    0x17f
                };
                self.runtime.show_solicited_message(self.ram, k, m);
            }
            _ => {}
        }
    }
    pub fn mini_chest_game_guy(&mut self, k: usize) {
        let d = self.runtime.direction_to_face_link(self.ram, k) ^ 3;
        self.ram.set_sprite_scratch_d(k, d);
        self.ram.set_sprite_graphics(k, 0);
        self.runtime.maze_game_guy_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.chest_game_logic(k, 0x17e, 20, 1);
    }
    pub fn lost_woods_chest_game_guy(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.chest_game_logic(k, 0x181, 100, 1);
    }

    pub fn zoro(&mut self, k: usize) {
        if self.ram.sprite(k).scratch_c() == 0 {
            self.ram.set_sprite_scratch_c(k, 1);
            if self.runtime.is_below_link(self.ram, k) != 0 {
                self.ram.set_sprite_state(k, 0);
                return;
            }
        }
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        let s = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, s);
        self.ram.set_sprite_graphics(k, s >> 1 & 1);
        self.ram.set_sprite_signed_field(
            k,
            SpriteField::VelocityX,
            [16, -16][(s >> 2 & 1) as usize],
        );
        self.runtime.move_xy(self.ram, k);
        if self.ram.sprite(k).main_delay() == 0
            && self.runtime.check_tile_collision(self.ram, k) != 0
        {
            self.ram.set_sprite_state(k, 0);
        }
        if s & 3 != 0 {
            return;
        }
        let Some(j) = self.runtime.garnish_alloc(self.ram) else {
            return;
        };
        self.ram.write_u8(GARNISH_TYPE + j, 6);
        self.ram.write_u8(GARNISH_ACTIVE, 6);
        self.set_garnish_x(j, self.sprite_x(k));
        self.set_garnish_y(j, self.sprite_y(k).wrapping_add(16));
        self.ram.write_u8(GARNISH_COUNTDOWN + j, 10);
        self.ram.write_u8(GARNISH_SPRITE + j, k as u8);
        self.ram
            .write_u8(GARNISH_FLOOR + j, self.ram.sprite(k).floor());
    }

    pub fn babasu(&mut self, k: usize) {
        const G: [u8; 6] = [5, 4, 3, 2, 1, 0];
        const DG: [u8; 4] = [6, 6, 0, 0];
        const V: [i8; 6] = [32, -32, 0, 0, 32, -32];
        self.runtime.babusu_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_ai_state(k, 1);
                self.ram.set_sprite_main_delay(k, 128);
                self.ram.set_sprite_graphics(k, 255);
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_main_delay(k, 55);
                }
            }
            2 => {
                let j = self.ram.sprite(k).main_delay();
                let i = self.ram.sprite(k).scratch_d() as usize;
                if j == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, V[i]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, V[i + 2]);
                    self.ram.set_sprite_main_delay(k, 32);
                }
                self.ram.set_sprite_graphics(
                    k,
                    if j >= 32 {
                        G[((j - 32) >> 2) as usize] + DG[i]
                    } else {
                        255
                    },
                );
            }
            3 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                self.runtime.move_xy(self.ram, k);
                let d = self.ram.sprite(k).scratch_d();
                self.ram.set_sprite_graphics(
                    k,
                    (self.ram.read_u8(FRAME_COUNTER) >> 1 & 1) + [18, 14, 12, 16][d as usize],
                );
                if self.ram.sprite(k).main_delay() == 0
                    && self.runtime.check_tile_collision(self.ram, k) != 0
                {
                    self.ram.set_sprite_scratch_d(k, d ^ 1);
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            _ => {}
        }
    }

    pub fn sprite_prep_antifairy_circle(&mut self, k: usize) {
        const X: [i8; 3] = [10, 20, 10];
        const Y: [i8; 3] = [-10, 0, 10];
        const XV: [i8; 3] = [18, 0, -18];
        const YV: [i8; 3] = [0, 18, 0];
        const A: [u8; 3] = [1, 1, 0];
        const B: [u8; 3] = [0, 1, 1];
        self.set_sprite_x(k, self.sprite_x(k).wrapping_sub(10));
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityY, -18);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
        self.ram.set_sprite_scratch_a(k, 0);
        self.ram.set_sprite_scratch_b(k, 0);
        self.ram.write_u8(TMP_COUNTER, 2);
        loop {
            let i = self.ram.read_u8(TMP_COUNTER) as usize;
            if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0x82) {
                self.set_sprite_x(j, x.wrapping_add_signed(X[i] as i16));
                self.set_sprite_y(j, y.wrapping_add_signed(Y[i] as i16));
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityX, XV[i]);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityY, YV[i]);
                self.ram.set_sprite_scratch_a(j, A[i]);
                self.ram.set_sprite_scratch_b(j, B[i]);
            }
            let n = self.ram.read_u8(TMP_COUNTER).wrapping_sub(1);
            self.ram.write_u8(TMP_COUNTER, n);
            if (n as i8) < 0 {
                break;
            }
        }
    }

    pub fn sprite_82_antifairy_circle(&mut self, k: usize) {
        const STEP: [i8; 2] = [1, -1];
        const TARGET: [u8; 2] = [18, (-18i8) as u8];
        self.runtime.draw_antifairy(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let a = (self.ram.sprite(k).scratch_a() & 1) as usize;
        let xv = self
            .ram
            .sprite(k)
            .signed_field(SpriteField::VelocityX)
            .wrapping_add(STEP[a]);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityX, xv);
        if xv as u8 == TARGET[a] {
            self.ram
                .set_sprite_scratch_a(k, self.ram.sprite(k).scratch_a().wrapping_add(1));
        }
        let b = (self.ram.sprite(k).scratch_b() & 1) as usize;
        let yv = self
            .ram
            .sprite(k)
            .signed_field(SpriteField::VelocityY)
            .wrapping_add(STEP[b]);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityY, yv);
        if yv as u8 == TARGET[b] {
            self.ram
                .set_sprite_scratch_b(k, self.ram.sprite(k).scratch_b().wrapping_add(1));
        }
        self.runtime.move_xy(self.ram, k);
        if xv != 0 && yv != 0 && self.runtime.check_if_room_is_clear(self.ram) {
            self.ram.set_sprite_kind(k, 0x15);
            self.ram.set_sprite_signed_field(
                k,
                SpriteField::VelocityX,
                if xv < 0 { -16 } else { 16 },
            );
            self.ram.set_sprite_signed_field(
                k,
                SpriteField::VelocityY,
                if yv < 0 { -16 } else { 16 },
            );
        }
        self.runtime.check_damage_to_link(self.ram, k);
    }

    pub fn sprite_81_hover(&mut self, k: usize) {
        const FLAGS: [u8; 4] = [0x40, 0, 0x40, 0];
        self.ram
            .set_sprite_object_priority(k, self.ram.sprite(k).object_priority() | 48);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).scratch_f() != 0 {
            self.ram.set_sprite_ai_state(k, 0);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        if self.ram.sprite(k).wall_collision() == 0 {
            self.runtime.move_xy(self.ram, k);
        }
        self.runtime.check_tile_collision(self.ram, k);
        let clock = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, clock);
        self.ram.set_sprite_graphics(k, clock >> 3 & 2);
        if self.ram.sprite(k).ai_state() == 0 {
            if self.ram.sprite(k).main_delay() == 0 {
                self.ram.set_sprite_ai_state(k, 1);
                let d = self.runtime.is_right_of_link(self.ram, k)
                    + self.runtime.is_below_link(self.ram, k) * 2;
                self.ram.set_sprite_scratch_d(k, d);
                let f = self.ram.sprite(k).oam_flags();
                self.ram
                    .set_sprite_oam_flags(k, f & !0x40 | FLAGS[d as usize]);
                let delay = (self.runtime.random(self.ram) & 15) + 12;
                self.ram.set_sprite_main_delay(k, delay);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
            }
        } else {
            const AX0: [i8; 4] = [1, -1, 1, -1];
            const AY0: [i8; 4] = [1, 1, -1, -1];
            const AX1: [i8; 4] = [-1, 1, -1, 1];
            const AY1: [i8; 4] = [-1, -1, 1, 1];
            let d = self.ram.sprite(k).scratch_d() as usize;
            if self.ram.sprite(k).main_delay() != 0 {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_add(AX0[d]),
                );
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityY,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityY)
                        .wrapping_add(AY0[d]),
                );
                self.ram.set_sprite_graphics(k, clock >> 3 & 1);
            } else {
                let xv = self
                    .ram
                    .sprite(k)
                    .signed_field(SpriteField::VelocityX)
                    .wrapping_add(AX1[d]);
                let yv = self
                    .ram
                    .sprite(k)
                    .signed_field(SpriteField::VelocityY)
                    .wrapping_add(AY1[d]);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, xv);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, yv);
                if yv == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    self.ram.set_sprite_main_delay(k, 64);
                }
            }
        }
    }

    pub fn crystal_maiden_run_cutscene(&mut self, k: usize) {
        self.ram
            .set_sprite_scratch_e(k, self.ram.sprite(k).scratch_e().wrapping_add(1));
        self.ram
            .write_u8(POLY_B, self.ram.read_u8(POLY_B).wrapping_add(6));
        if self.ram.read_u8(SUBMODULE_INDEX) != 0 {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.write_u8(TS_COPY, 0);
                self.ram.set_sprite_ai_state(k, 1);
            }
            1 => {
                self.ram.write_u8(TS_COPY, 1);
                self.ram.set_sprite_ai_state(k, 2);
            }
            2 => {
                let p = self.ram.read_u8(POLY_CONFIG1);
                if p < 6 {
                    self.ram.write_u8(POLY_CONFIG1, 0);
                    self.ram.set_sprite_ai_state(k, 3);
                } else {
                    let p = p.wrapping_sub(3);
                    self.ram.write_u8(POLY_CONFIG1, p);
                    if p >= 64 {
                        let ancilla_source = self.ram.sprite(k).scratch_a() as usize;
                        self.runtime
                            .add_sword_charge_sparkle(self.ram, ancilla_source);
                    }
                }
            }
            3 | 4 => {
                if self.ram.sprite(k).ai_state() == 3 {
                    self.ram.set_sprite_ai_state(k, 4);
                }
                if self.ram.sprite(k).scratch_e() & 1 == 0 {
                    self.runtime.palette_filter_sp5f(self.ram);
                    if self.ram.read_u8(PALETTE_FILTER_COUNTDOWN) == 0 {
                        self.ram.set_sprite_ai_state(k, 5);
                        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                        self.ram.write_u8(LINK_RECEIVEITEM_INDEX, 0);
                        self.ram.write_u8(LINK_POSE_FOR_ITEM, 0);
                        self.ram.write_u8(LINK_ANIMATION_STEPS, 0);
                        self.ram.write_u8(LINK_DIRECTION_FACING, 0);
                    }
                }
            }
            5 => {
                const MSG: [u16; 9] = [
                    0x133, 0x132, 0x137, 0x134, 0x136, 0x132, 0x135, 0x138, 0x13c,
                ];
                let mut j = self.ram.read_u8(CUR_PALACE_INDEX_X2).wrapping_sub(10);
                if j == 2 && self.ram.read_u8(SAVEGAME_MAP_ICONS_INDICATOR) < 7 {
                    self.ram.write_u8(SAVEGAME_MAP_ICONS_INDICATOR, 7);
                }
                if j == 14 && self.ram.read_u8(LINK_HAS_CRYSTALS) & 0x7f != 0x7f {
                    j = 16;
                }
                self.runtime.show_message(self.ram, MSG[(j >> 1) as usize]);
                self.ram.set_sprite_ai_state(k, 6);
                if self.ram.read_u8(LINK_HAS_CRYSTALS) & 0x7f == 0x7f {
                    self.ram.write_u8(SAVEGAME_MAP_ICONS_INDICATOR, 8);
                }
            }
            6 => {
                self.runtime.show_message(self.ram, 0x13a);
                self.ram.set_sprite_ai_state(k, 7);
            }
            7 => {
                if self.ram.read_u8(CHOICE_IN_MULTISELECT_BOX) != 0 {
                    self.ram.set_sprite_ai_state(k, 5);
                } else {
                    self.runtime.show_message(self.ram, 0x139);
                    self.ram.set_sprite_ai_state(k, 8);
                }
            }
            8 => {
                self.ram.write_u8(TS_COPY, 0);
                self.runtime
                    .bank2b_prepare_dungeon_exit_from_boss_fight(self.ram);
                self.ram.set_sprite_state(k, 0);
            }
            _ => {}
        }
    }

    pub fn zirro_drop_bomb(&mut self, k: usize) {
        const X: [i8; 4] = [14, -6, 4, 4];
        const Y: [i8; 4] = [7, 7, 12, -4];
        const XV: [i8; 4] = [16, -16, 0, 0];
        const YV: [i8; 4] = [0, 0, 16, -16];
        if let Some((j, x, y, z)) = self.runtime.spawn_dynamic(self.ram, k, 0xa8) {
            self.runtime.queue_sfx2(self.ram, k, 0x20);
            self.ram.set_sprite_z(j, z);
            let d = self.ram.sprite(j).scratch_d() as usize;
            self.set_sprite_x(j, x.wrapping_add_signed(X[d] as i16));
            self.set_sprite_y(j, y.wrapping_add_signed(Y[d] as i16));
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityX, XV[d]);
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityY, YV[d]);
            self.ram.set_sprite_scratch_a(j, 1);
            self.ram.set_sprite_ignore_projectile(j, 1);
            self.ram.set_sprite_flags4(j, 9);
            self.ram.set_sprite_flags3(j, 0x33);
            self.ram.set_sprite_oam_flags(j, 3);
        }
    }

    pub fn stalfos_bone_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (-4, -2, 0x802f, 0),
            (4, 2, 0x402f, 0),
            (-4, 2, 0x002f, 0),
            (4, -2, 0xc02f, 0),
            (2, -4, 0x403f, 0),
            (-2, 4, 0x803f, 0),
            (-2, -4, 0x003f, 0),
            (2, 4, 0xc03f, 0),
        ];
        let i = ((self.ram.sprite(k).subtype2() >> 2 & 3) * 2) as usize;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
    }

    pub fn sprite_stalfos_bone(&mut self, k: usize) {
        self.stalfos_bone_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_link(self.ram, k);
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        self.runtime.move_xy(self.ram, k);
        if self.ram.sprite(k).main_delay() == 0
            && self.runtime.check_tile_collision(self.ram, k) != 0
        {
            self.ram.set_sprite_state(k, 0);
            self.runtime.place_weapon_tink(self.ram, k);
        }
    }

    pub fn fire_phlegm_draw(&mut self, k: usize) {
        const D: [DrawPiece; 16] = [
            (0, 0, 0x00c3, 0),
            (-8, 0, 0x00c2, 0),
            (0, 0, 0x80c3, 0),
            (-8, 0, 0x80c2, 0),
            (0, 0, 0x40c3, 0),
            (8, 0, 0x40c2, 0),
            (0, 0, 0xc0c3, 0),
            (8, 0, 0xc0c2, 0),
            (0, 0, 0x00d4, 0),
            (0, -8, 0x00d3, 0),
            (0, 0, 0x40d4, 0),
            (0, -8, 0x40d3, 0),
            (0, 0, 0x80d4, 0),
            (0, 8, 0x80d3, 0),
            (0, 0, 0xc0d4, 0),
            (0, 8, 0xc0d3, 0),
        ];
        let i = (self.ram.sprite(k).scratch_d() * 4 + self.ram.sprite(k).graphics() * 2) as usize;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
    }

    pub fn sprite_spawn_fire_phlegm(&mut self, k: usize) -> i32 {
        const X: [i8; 4] = [16, -8, 4, 4];
        const Y: [i8; 4] = [-2, -2, 8, -20];
        const XV: [i8; 4] = [48, -48, 0, 0];
        const YV: [i8; 4] = [0, 0, 48, -48];
        let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xa5) else {
            return -1;
        };
        self.runtime.queue_sfx3(self.ram, k, 5);
        let d = self.ram.sprite(k).scratch_d() as usize;
        self.set_sprite_x(j, x.wrapping_add_signed(X[d] as i16));
        self.set_sprite_y(j, y.wrapping_add_signed(Y[d] as i16));
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityX, XV[d]);
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityY, YV[d]);
        self.ram
            .set_sprite_flags3(j, self.ram.sprite(j).flags3() | 0x40);
        self.ram.set_sprite_deflection_bits(j, 0x40);
        self.ram.set_sprite_flags2(j, 0x21);
        self.ram.set_sprite_scratch_b(j, 0x21);
        self.ram.set_sprite_oam_flags(j, 2);
        self.ram.set_sprite_flags4(j, 0x14);
        self.ram.set_sprite_ignore_projectile(j, 20);
        self.ram.set_sprite_bump_damage(j, 37);
        if self.ram.read_u8(LINK_SHIELD_TYPE) >= 3 {
            self.ram.set_sprite_flags5(j, 0x20);
        }
        j as i32
    }

    pub fn stalfos_throw_bone(&mut self, k: usize) {
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xa7) {
            self.ram.set_sprite_scratch_a(j, 1);
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            self.runtime.apply_speed_towards_link(self.ram, j, 32);
            self.ram.set_sprite_flags2(j, 0x21);
            self.ram.set_sprite_ignore_projectile(j, 33);
            self.ram
                .set_sprite_flags3(j, self.ram.sprite(j).flags3() | 0x40);
            self.ram.set_sprite_deflection_bits(j, 0x48);
            self.ram.set_sprite_main_delay(j, 16);
            self.ram.set_sprite_flags4(j, 0x14);
            self.ram.set_sprite_oam_flags(j, 7);
            self.ram.set_sprite_bump_damage(j, 32);
            self.runtime.queue_sfx2(self.ram, k, 2);
        }
    }

    pub fn generate_iceball(&mut self, k: usize) {
        let s = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, s);
        if s & 127 != 0 || self.ram.sprite(k).aux1_delay() != 0 {
            return;
        }
        if let Some((j, _, _, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xa4) {
            self.set_sprite_x(j, self.ram.read_u16(LINK_X));
            self.set_sprite_y(j, self.ram.read_u16(LINK_Y));
            self.ram.set_sprite_z(j, 0xe0);
            self.ram.set_sprite_scratch_c(j, 0xe0);
            self.runtime.queue_sfx2(self.ram, j, 0x20);
        }
    }

    pub fn sprite_9e_haunted_grove_ostritch(&mut self, k: usize) {
        const GFX: [u8; 4] = [0, 1, 0, 2];
        self.runtime.flute_boy_ostrich_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_graphics(
                    k,
                    if self.ram.read_u8(FRAME_COUNTER) & 0x18 != 0 {
                        3
                    } else {
                        0
                    },
                );
                if self.ram.read_u8(BYTE_7E0FDD) != 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, -8);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, -16);
                }
            }
            1 => {
                self.runtime.move_xyz(self.ram, k);
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityZ,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityZ)
                        .wrapping_sub(2),
                );
                if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 32);
                    self.ram.set_sprite_z(k, 0);
                    self.ram.set_sprite_subtype2(k, 0);
                    self.ram.set_sprite_scratch_a(k, 0);
                }
                let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(k, s);
                let a = self.ram.sprite(k).scratch_a();
                if s & 7 == 0 && a != 3 {
                    self.ram.set_sprite_scratch_a(k, a + 1);
                }
                self.ram
                    .set_sprite_graphics(k, GFX[self.ram.sprite(k).scratch_a() as usize]);
            }
            _ => {}
        }
    }

    pub fn sprite_9f_haunted_grove_rabbit(&mut self, k: usize) {
        const FLAGS: [u8; 2] = [0x40, 0];
        const XV: [i8; 4] = [16, -16, 0, 0];
        const GFX: [u8; 3] = [0, 1, 2];
        let d = self.ram.sprite(k).scratch_d() as usize;
        self.ram
            .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() & !0x40 | FLAGS[d]);
        self.runtime.draw_single_large(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_graphics(k, 3);
                if self.ram.read_u8(BYTE_7E0FDD) != 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    let nd = self.ram.sprite(k).scratch_d() ^ 1;
                    self.ram.set_sprite_scratch_d(k, nd);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, XV[nd as usize]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, -8);
                }
            }
            1 => {
                self.runtime.move_xyz(self.ram, k);
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityZ,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityZ)
                        .wrapping_sub(3),
                );
                if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 24);
                    self.ram.set_sprite_z(k, 0);
                    self.ram.set_sprite_subtype2(k, 0);
                    self.ram.set_sprite_scratch_a(k, 0);
                }
                let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(k, s);
                let a = self.ram.sprite(k).scratch_a();
                if s & 3 == 0 && a != 2 {
                    self.ram.set_sprite_scratch_a(k, a + 1);
                }
                self.ram
                    .set_sprite_graphics(k, GFX[self.ram.sprite(k).scratch_a() as usize]);
            }
            _ => {}
        }
    }

    pub fn wizzrobe_fire_beam(&mut self, k: usize) {
        const V: [i8; 6] = [32, -32, 0, 0, 32, -32];
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0x9b) {
            self.runtime.queue_sfx3(self.ram, k, 0x36);
            self.ram.set_sprite_scratch_c(j, 1);
            self.ram.set_sprite_ignore_projectile(j, 1);
            self.set_sprite_x(j, x.wrapping_add(4));
            self.set_sprite_y(j, y);
            let d = self.ram.sprite(k).scratch_d() as usize;
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityX, V[d]);
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityY, V[d + 2]);
            self.ram.set_sprite_deflection_bits(j, 0x48);
            self.ram.set_sprite_oam_flags(j, 2);
            self.ram.set_sprite_flags5(
                j,
                if self.ram.read_u8(LINK_SHIELD_TYPE) == 3 {
                    0x20
                } else {
                    0
                },
            );
            self.ram.set_sprite_flags4(j, 0x14);
        }
    }

    pub fn sprite_9b_wizzrobe(&mut self, k: usize) {
        const CLOAK_GFX: [u8; 4] = [4, 2, 0, 6];
        const ATTACK_GFX: [u8; 8] = [0, 1, 1, 1, 1, 1, 1, 0];
        if self.ram.sprite(k).scratch_c() != 0 {
            self.runtime.wizzbeam(self.ram, k);
            return;
        }
        let state = self.ram.sprite(k).ai_state();
        let delay = self.ram.sprite(k).main_delay();
        if state == 0 || state & 1 != 0 && delay & 1 != 0 {
            self.runtime.prep_oam(self.ram, k);
        } else {
            self.runtime.wizzrobe_draw(self.ram, k);
        }
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.ram.set_sprite_ignore_projectile(k, 1);
        match state {
            0 if delay == 0 => {
                self.ram.set_sprite_velocity_x(k, 1);
                self.ram.set_sprite_velocity_y(k, 1);
                if self.runtime.check_tile_collision(self.ram, k) == 0 {
                    let d = self.runtime.direction_to_face_link(self.ram, k);
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_main_delay(k, 63);
                    self.ram.set_sprite_scratch_d(k, d);
                    self.ram.set_sprite_graphics(k, CLOAK_GFX[d as usize]);
                } else {
                    self.ram.set_sprite_state(k, 0);
                }
            }
            1 if delay == 0 => {
                self.ram.set_sprite_ai_state(k, 2);
                self.ram.set_sprite_main_delay(k, 63);
            }
            2 => {
                self.ram.set_sprite_ignore_projectile(k, 0);
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_main_delay(k, 63);
                    return;
                }
                if delay == 32 {
                    self.wizzrobe_fire_beam(k);
                }
                let d = self.ram.sprite(k).scratch_d() as usize;
                self.ram
                    .set_sprite_graphics(k, ATTACK_GFX[(delay >> 3) as usize] + CLOAK_GFX[d]);
            }
            3 if delay == 0 => {
                if self.ram.sprite(k).scratch_b() != 0 {
                    self.ram.set_sprite_state(k, 0);
                }
                self.ram.set_sprite_ai_state(k, 0);
                let d = (self.runtime.random(self.ram) & 31) + 32;
                self.ram.set_sprite_main_delay(k, d);
            }
            _ => {}
        }
    }

    pub fn sprite_laser_beam(&mut self, k: usize) {
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.laser_beam_build_up_garnish(self.ram, k);
        self.runtime.move_xy(self.ram, k);
        self.runtime.check_damage_same_layer(self.ram, k);
        if self.ram.sprite(k).main_delay() == 0
            && self.runtime.check_tile_collision(self.ram, k) != 0
        {
            self.ram.set_sprite_state(k, 0);
            self.runtime.queue_sfx3(self.ram, k, 0x26);
        }
    }

    pub fn laser_eye_fire_beam(&mut self, k: usize) {
        const XY: [i8; 6] = [12, -4, 4, 4, 12, -4];
        const V: [i8; 6] = [112, -112, 0, 0, 112, -112];
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0x95) {
            let d = self.ram.sprite(k).scratch_d() as usize;
            self.ram.set_sprite_graphics(j, (d as u8 & 2) >> 1);
            self.set_sprite_x(j, x.wrapping_add_signed(XY[d] as i16));
            self.set_sprite_y(j, y.wrapping_add_signed(XY[d + 2] as i16));
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityX, V[d]);
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityY, V[d + 2]);
            self.ram.set_sprite_flags2(j, 0x20);
            self.ram.set_sprite_scratch_a(j, 0x20);
            self.ram.set_sprite_oam_flags(j, 5);
            self.ram.set_sprite_deflection_bits(j, 0x48);
            self.ram.set_sprite_ignore_projectile(j, 0x48);
            self.ram.set_sprite_main_delay(j, 5);
            if self.ram.read_u8(LINK_SHIELD_TYPE) == 3 {
                self.ram.set_sprite_flags5(j, 32);
            }
            self.runtime.queue_sfx3(self.ram, k, 0x19);
        }
    }
    pub fn sprite_95_laser_eye_left(&mut self, k: usize) {
        const DIRS: [u8; 4] = [2, 3, 0, 1];
        if self.ram.sprite(k).scratch_a() != 0 {
            self.sprite_laser_beam(k);
            return;
        }
        self.laser_eye_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                let d = self.ram.sprite(k).scratch_d();
                if self.ram.sprite(k).head_direction() == 0
                    && d != DIRS[(self.ram.read_u8(LINK_DIRECTION_FACING) >> 1) as usize]
                {
                    self.ram.set_sprite_graphics(k, 0)
                } else {
                    let delta = if d < 2 {
                        self.ram
                            .read_u16(LINK_Y)
                            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_Y))
                    } else {
                        self.ram
                            .read_u16(LINK_X)
                            .wrapping_sub(self.ram.read_u16(CUR_SPRITE_X))
                    };
                    if delta.wrapping_add(16) < 32 {
                        self.ram.set_sprite_main_delay(k, 32);
                        self.ram.set_sprite_ai_state(k, 1)
                    } else {
                        self.ram.set_sprite_graphics(k, 0)
                    }
                }
            }
            1 => {
                self.ram.set_sprite_graphics(k, 1);
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    self.laser_eye_fire_beam(k);
                    self.ram.set_sprite_aux4_delay(k, 12)
                }
            }
            _ => {}
        }
    }

    pub fn laser_eye_draw(&mut self, k: usize) {
        const D: [DrawPiece; 24] = [
            (8, -4, 0x40c8, 0),
            (8, 4, 0x40d8, 0),
            (8, 12, 0xc0c8, 0),
            (8, -4, 0x40c9, 0),
            (8, 4, 0x40d9, 0),
            (8, 12, 0xc0c9, 0),
            (0, -4, 0x00c8, 0),
            (0, 4, 0x00d8, 0),
            (0, 12, 0x80c8, 0),
            (0, -4, 0x00c9, 0),
            (0, 4, 0x00d9, 0),
            (0, 12, 0x80c9, 0),
            (-4, 8, 0x00d6, 0),
            (4, 8, 0x00d7, 0),
            (12, 8, 0x40d6, 0),
            (-4, 8, 0x00c6, 0),
            (4, 8, 0x00c7, 0),
            (12, 8, 0x40c6, 0),
            (-4, 0, 0x80d6, 0),
            (4, 0, 0x80d7, 0),
            (12, 0, 0xc0d6, 0),
            (-4, 0, 0x80c6, 0),
            (4, 0, 0x80c7, 0),
            (12, 0, 0xc0c6, 0),
        ];
        if self.ram.sprite(k).head_direction() != 0 {
            self.ram
                .set_sprite_graphics(k, u8::from(self.ram.sprite(k).aux4_delay() == 0));
        }
        self.ram.set_sprite_object_priority(k, 0x30);
        let i = ((self.ram.sprite(k).graphics() + self.ram.sprite(k).scratch_d() * 2) * 3) as usize;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 3]);
    }

    pub fn bumper_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (-8, -8, 0x00ec, 2),
            (8, -8, 0x40ec, 2),
            (-8, 8, 0x80ec, 2),
            (8, 8, 0xc0ec, 2),
            (-7, -7, 0x00ec, 2),
            (7, -7, 0x40ec, 2),
            (-7, 7, 0x80ec, 2),
            (7, 7, 0xc0ec, 2),
        ];
        let i = ((self.ram.sprite(k).main_delay() >> 1 & 1) * 4) as usize;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
    }

    pub fn sprite_90_wallmaster(&mut self, k: usize) {
        self.ram
            .set_sprite_object_priority(k, self.ram.sprite(k).object_priority() | 0x30);
        self.wall_master_draw(k);
        if self.ram.sprite(k).state() != 9 {
            self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
            self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 0);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).scratch_a() != 0 {
            let y = self
                .sprite_y(k)
                .wrapping_sub(self.ram.sprite(k).z() as u16)
                .wrapping_add(3);
            self.ram.write_u16(LINK_X, self.sprite_x(k));
            self.ram.write_u16(LINK_Y, y);
            self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
            self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 1);
            self.ram.write_u8(LINK_INCAPACITATED_TIMER, 0);
            self.ram.write_u8(LINK_ACTUAL_VEL_X, 0);
            self.ram.write_u8(LINK_ACTUAL_VEL_Y, 0);
            self.ram.write_u8(LINK_Y_VEL, 0);
            self.ram.write_u8(LINK_X_VEL, 0);
            if y.wrapping_sub(self.ram.read_u16(BG2VOFS_COPY2))
                .wrapping_sub(16)
                >= 0x100
            {
                self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 0);
                self.runtime
                    .wallmaster_send_player_to_last_entrance(self.ram);
                self.runtime.link_initialize(self.ram);
                return;
            }
        } else {
            self.runtime.check_damage_from_link(self.ram, k);
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                let old = self.ram.sprite(k).z();
                self.runtime.move_z(self.ram, k);
                let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                if zv >= -64 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, zv.wrapping_sub(3));
                }
                let z = self.ram.sprite(k).z();
                if ((old ^ z) as i8) < 0 && (z as i8) < 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_z(k, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
                    self.ram.set_sprite_main_delay(k, 63);
                }
            }
            1 => {
                let t = self.ram.sprite(k).main_delay();
                if t == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                }
                self.ram
                    .set_sprite_graphics(k, if t & 0x20 != 0 { 0 } else { 1 });
                if self.runtime.check_damage_to_link(self.ram, k) {
                    self.ram.set_sprite_scratch_a(k, 1);
                    self.ram.set_sprite_flags3(k, 64);
                    self.runtime.queue_sfx3(self.ram, k, 0x2a);
                }
            }
            2 => {
                let old = self.ram.sprite(k).z();
                self.runtime.move_z(self.ram, k);
                let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                if zv < 64 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, zv.wrapping_add(2));
                }
                let z = self.ram.sprite(k).z();
                if ((old ^ z) as i8) < 0 && (z as i8) >= 0 {
                    self.ram.set_sprite_state(k, 0);
                }
            }
            _ => {}
        }
    }
    pub fn wall_master_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (-4, 0, 0x01a6, 2),
            (12, 0, 0x01aa, 0),
            (-4, 16, 0x01ba, 0),
            (4, 8, 0x01a8, 2),
            (-4, 0, 0x01ab, 2),
            (12, 0, 0x01af, 0),
            (-4, 16, 0x01bf, 0),
            (4, 8, 0x01ad, 2),
        ];
        let i = self.ram.sprite(k).graphics() as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        self.runtime.draw_large_shadow2(self.ram, k);
    }

    pub fn zol_draw(&mut self, k: usize) {
        if self.ram.sprite(k).oam_flags() & 1 == 0 && self.ram.read_u8(BYTE_7E0FC6) >= 3 {
            return;
        }
        if self.ram.sprite(k).aux4_delay() != 0 {
            self.runtime.allocate_oam_region_b(self.ram, 8);
        }
        if self.ram.sprite(k).ai_state() == 0 {
            self.runtime.prep_oam(self.ram, k);
            return;
        }
        let gfx = self.ram.sprite(k).graphics();
        if gfx < 4 {
            const FLAGS: [u8; 4] = [0, 0, 0x40, 0x40];
            let old = self.ram.sprite(k).oam_flags();
            let flags = old ^ FLAGS[gfx as usize];
            self.ram.set_sprite_oam_flags(k, flags);
            self.ram
                .set_sprite_graphics(k, gfx + (((flags & 1) ^ 1) << 2));
            self.runtime.draw_single_large(self.ram, k);
            self.ram.set_sprite_graphics(k, gfx);
            self.ram.set_sprite_oam_flags(k, old);
        } else {
            const D: [DrawPiece; 8] = [
                (0, 8, 0x036c, 0),
                (8, 8, 0x036d, 0),
                (0, 8, 0x0060, 0),
                (8, 8, 0x0070, 0),
                (0, 8, 0x4070, 0),
                (8, 8, 0x4060, 0),
                (0, 0, 0x0040, 2),
                (0, 0, 0x0040, 2),
            ];
            let i = (gfx as usize - 4) * 2;
            self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
        }
    }

    pub fn terrorpin_set_up_hammer_hit_box(&self, k: usize, hit_box: &mut SpriteHitBox) {
        let x = self.sprite_x(k).wrapping_sub(16);
        let y = self.sprite_y(k).wrapping_sub(16);
        let [xlo, xhi] = x.to_le_bytes();
        let [ylo, yhi] = y.to_le_bytes();
        hit_box.r4_spr_xlo = xlo;
        hit_box.r10_spr_xhi = xhi;
        hit_box.r5_spr_ylo = ylo;
        hit_box.r11_spr_yhi = yhi;
        hit_box.r6_spr_xsize = 48;
        hit_box.r7_spr_ysize = 48;
    }

    pub fn sprite_8e_terrorpin(&mut self, k: usize) {
        const X: [i8; 8] = [8, -8, 0, 0, 12, -12, 0, 0];
        const Y: [i8; 8] = [0, 0, 8, -8, 0, 0, 12, -12];
        self.runtime.draw_single_large(self.ram, k);
        self.runtime.check_tile_collision(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        if self.ram.sprite(k).aux2_delay() == 0 {
            self.runtime.check_damage_from_link(self.ram, k);
        }
        self.terrorpin_check_for_hammer(k);
        self.runtime.move_xyz(self.ram, k);
        if self.ram.sprite(k).scratch_b() == 0 {
            if self.ram.sprite(k).aux4_delay() == 0 {
                let t = (self.runtime.random(self.ram) & 31) + 32;
                self.ram.set_sprite_aux4_delay(k, t);
                let d = self.runtime.direction_to_face_link(self.ram, k);
                self.ram.set_sprite_scratch_d(k, d);
            }
            let j = (self.ram.sprite(k).scratch_d() + self.ram.sprite(k).scratch_g()) as usize;
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, X[j]);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, Y[j]);
            let z = self
                .ram
                .sprite(k)
                .signed_field(SpriteField::VelocityZ)
                .wrapping_sub(2);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityZ, z);
            if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
                self.ram.set_sprite_z(k, 0);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
            }
            let shift = if self.ram.sprite(k).scratch_g() != 0 {
                2
            } else {
                3
            };
            self.ram
                .set_sprite_graphics(k, self.ram.read_u8(FRAME_COUNTER) >> shift & 1);
            self.ram
                .set_sprite_flags3(k, self.ram.sprite(k).flags3() | 64);
            self.ram.set_sprite_deflection_bits(k, 4);
            self.runtime.check_damage_to_link(self.ram, k);
        } else {
            self.ram
                .set_sprite_flags3(k, self.ram.sprite(k).flags3() & 191);
            self.ram.set_sprite_deflection_bits(k, 0);
            let t = self.ram.sprite(k).aux4_delay();
            if t == 0 {
                self.ram.set_sprite_scratch_b(k, 0);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, 32);
                self.ram.set_sprite_aux4_delay(k, 64);
                return;
            }
            let zv = self
                .ram
                .sprite(k)
                .signed_field(SpriteField::VelocityZ)
                .wrapping_sub(2);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityZ, zv);
            if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
                self.ram.set_sprite_z(k, 0);
                let q = (zv.wrapping_neg() as u8) >> 1;
                self.ram.set_sprite_velocity_z(k, if q < 9 { 0 } else { q });
                let x = self.ram.sprite(k).signed_field(SpriteField::VelocityX) >> 1;
                let y = self.ram.sprite(k).signed_field(SpriteField::VelocityY) >> 1;
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    if x == -1 { 0 } else { x },
                );
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityY,
                    if y == -1 { 0 } else { y },
                );
            }
            if t < 64 {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    [8, -8][(t >> 1 & 1) as usize],
                );
                self.ram
                    .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
            }
            self.ram.set_sprite_graphics(k, 2);
            let s = self.ram.sprite(k).subtype2().wrapping_add(1);
            self.ram.set_sprite_subtype2(k, s);
            self.ram.set_sprite_oam_flags(
                k,
                self.ram.sprite(k).oam_flags() & !0x40 | [0, 0x40][(s >> 3 & 1) as usize],
            );
        }
    }
    pub fn terrorpin_check_for_hammer(&mut self, k: usize) {
        if self.ram.sprite(k).z() == 0
            && self.ram.sprite(k).aux2_delay() == 0
            && self.ram.sprite(k).floor() == self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL)
            && self.ram.read_u8(PLAYER_OAM_Y_OFFSET) != 0x80
            && self.ram.read_u8(LINK_ITEM_IN_HAND) & 0xa != 0
        {
            let mut hb = SpriteHitBox::default();
            self.runtime.player_setup_action_hit_box(self.ram, &mut hb);
            self.terrorpin_set_up_hammer_hit_box(k, &mut hb);
            if self.runtime.hit_boxes_overlap(&hb) {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_neg(),
                );
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityY,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityY)
                        .wrapping_neg(),
                );
                self.ram.set_sprite_aux2_delay(k, 32);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, 32);
                self.ram.set_sprite_scratch_g(k, 4);
                let b = self.ram.sprite(k).scratch_b() ^ 1;
                self.ram.set_sprite_scratch_b(k, b);
                self.ram
                    .set_sprite_aux4_delay(k, if b != 0 { 0xff } else { 0x40 });
            }
        }
        self.ram.set_sprite_head_direction(k, 0);
    }

    pub fn arrghus_handle_puffs(&mut self, k: usize) {
        const T0: [u16; 13] = [
            0, 0x40, 0x80, 0xc0, 0x100, 0x140, 0x180, 0x1c0, 0, 0x66, 0xcc, 0x132, 0x198,
        ];
        const T1: [u16; 13] = [0, 0, 0, 0, 0, 0, 0, 0, 0x1ff, 0x1ff, 0x1ff, 0x1ff, 0x1ff];
        const T2: [u8; 13] = [
            0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0xc, 0xc, 0xc, 0xc, 0xc,
        ];
        const T3: [i8; 52] = [
            0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3, -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5,
            -4, -3, -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3, -2, -1, 0, -1, -2, -3, -4,
            -5, -6, -6, -5, -4, -3, -2, -1,
        ];
        let angle = self
            .ram
            .read_u16(OVERLORD_X_LO)
            .wrapping_add(u16::from(self.ram.read_u8(OVERLORD_X_LO + 4)));
        self.ram.write_u16(OVERLORD_X_LO, angle);
        if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 {
            let a = self.ram.sprite(k).scratch_a().wrapping_add(1);
            self.ram
                .set_sprite_scratch_a(k, if a == 13 { 0 } else { a });
        }
        if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
            let b = self.ram.sprite(k).scratch_b().wrapping_add(1);
            self.ram
                .set_sprite_scratch_b(k, if b == 13 { 0 } else { b });
        }
        for i in 0..13 {
            let r = (angle.wrapping_add(T0[i])) ^ T1[i];
            let radius = self.ram.read_u8(OVERLORD_X_LO + 2).wrapping_add(T2[i]);
            let sv = self.runtime.arrgi_sin(
                r,
                radius.wrapping_add_signed(T3[self.ram.sprite(k).scratch_a() as usize + i]),
            );
            let cv = self.runtime.arrgi_sin(
                r.wrapping_add(0x80),
                radius.wrapping_add_signed(T3[self.ram.sprite(k).scratch_b() as usize + i]),
            );
            let x = self.sprite_x(k).wrapping_add_signed(sv as i16);
            let y = self
                .sprite_y(k)
                .wrapping_add_signed(cv as i16)
                .wrapping_sub(16);
            let [xl, xh] = x.to_le_bytes();
            let [yl, yh] = y.to_le_bytes();
            self.ram.write_u8(OVERLORD_X_HI + i, xl);
            self.ram.write_u8(OVERLORD_Y_HI + i, xh);
            self.ram.write_u8(OVERLORD_GEN2 + i, yl);
            self.ram.write_u8(OVERLORD_FLOOR + i, yh);
        }
        self.ram.write_u8(TMP_COUNTER, 13);
    }
    pub fn sprite_8d_arrghi(&mut self, k: usize) {
        const G: [u8; 8] = [0, 1, 2, 2, 2, 2, 2, 1];
        self.ram
            .set_sprite_object_priority(k, self.ram.sprite(k).object_priority() | 0x30);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let s = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, s);
        self.ram.set_sprite_graphics(k, G[(s >> 3 & 7) as usize]);
        let b = self.ram.sprite(k).scratch_b();
        if b != 0 {
            let j = (b - 1) as usize;
            if self.ram.read_u8(ANCILLA_TYPE + j) != 0 {
                self.ram
                    .set_sprite_x_low(k, self.ram.read_u8(ANCILLA_X_LO + j));
                self.ram
                    .set_sprite_x_high(k, self.ram.read_u8(ANCILLA_X_HI + j));
                self.ram
                    .set_sprite_y_low(k, self.ram.read_u8(ANCILLA_Y_LO + j));
                self.ram
                    .set_sprite_y_high(k, self.ram.read_u8(ANCILLA_Y_HI + j));
                self.ram.set_sprite_oam_flags(k, 5);
                self.ram
                    .set_sprite_flags3(k, self.ram.sprite(k).flags3() & !0x40);
                return;
            }
            self.ram.set_sprite_ai_state(k, 1);
            self.ram.set_sprite_scratch_b(k, 0);
            self.ram.set_sprite_main_delay(k, 32);
        }
        if self.ram.sprite(k).main_delay() == 0 {
            self.runtime.check_damage_to_link(self.ram, k);
        }
        let i = k + 7;
        if self.ram.sprite(k).ai_state() == 0 {
            self.ram
                .set_sprite_x_low(k, self.ram.read_u8(OVERLORD_X_LO + i));
            self.ram
                .set_sprite_x_high(k, self.ram.read_u8(OVERLORD_Y_LO + i));
            self.ram
                .set_sprite_y_low(k, self.ram.read_u8(OVERLORD_GEN1 + i));
            self.ram
                .set_sprite_y_high(k, self.ram.read_u8(OVERLORD_GEN3 + i));
            return;
        }
        self.runtime.check_damage_from_link(self.ram, k);
        if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0 {
            let x = u16::from_le_bytes([
                self.ram.read_u8(OVERLORD_X_LO + i),
                self.ram.read_u8(OVERLORD_Y_LO + i),
            ]);
            let y = u16::from_le_bytes([
                self.ram.read_u8(OVERLORD_GEN1 + i),
                self.ram.read_u8(OVERLORD_GEN3 + i),
            ]);
            let (yv, xv) = self.runtime.project_speed(self.ram, k, x, y, 4);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, yv);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, xv);
            if self
                .ram
                .sprite(k)
                .x_low()
                .wrapping_sub(self.ram.read_u8(OVERLORD_X_LO + i))
                .wrapping_add(8)
                < 16
                && self
                    .ram
                    .sprite(k)
                    .y_low()
                    .wrapping_sub(self.ram.read_u8(OVERLORD_GEN1 + i))
                    .wrapping_add(8)
                    < 16
            {
                self.ram.set_sprite_ai_state(k, 0);
                self.ram.set_sprite_oam_flags(k, 0xd);
                self.ram
                    .set_sprite_flags3(k, self.ram.sprite(k).flags3() | 0x40);
            }
        }
        self.runtime.move_xy(self.ram, k);
    }
    pub fn sprite_8b_gibdo(&mut self, k: usize) {
        self.gibdo_draw(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                const T: [u8; 4] = [2, 6, 4, 0];
                const G: [u8; 8] = [4, 8, 11, 10, 0, 6, 3, 7];
                let d = self.ram.sprite(k).scratch_d();
                self.ram.set_sprite_graphics(k, G[d as usize]);
                if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
                    let target = T[self.ram.sprite(k).scratch_a() as usize];
                    if d != target {
                        self.ram.set_sprite_scratch_d(
                            k,
                            d.wrapping_add(if (d.wrapping_sub(target) as i8) < 0 {
                                1
                            } else {
                                u8::MAX
                            }),
                        );
                    } else {
                        let t = (self.runtime.random(self.ram) & 31) + 48;
                        self.ram.set_sprite_main_delay(k, t);
                        self.ram.set_sprite_ai_state(k, 1);
                    }
                }
            }
            1 => {
                const V: [i8; 10] = [-16, 0, 0, 0, 16, 0, 0, 0, -16, 0];
                const G: [u8; 8] = [9, 2, 0, 4, 11, 3, 1, 5];
                let d = self.ram.sprite(k).scratch_d() as usize;
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, V[d + 2]);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, V[d]);
                self.runtime.move_xy(self.ram, k);
                self.runtime.check_tile_collision(self.ram, k);
                let must_retarget = self.ram.sprite(k).main_delay() == 0
                    || self.ram.sprite(k).wall_collision() != 0;
                let face = must_retarget.then(|| self.runtime.direction_to_face_link(self.ram, k));
                if face.is_some_and(|face| face != self.ram.sprite(k).scratch_a()) {
                    let face = face.unwrap();
                    self.ram.set_sprite_scratch_a(k, face);
                    self.ram.set_sprite_ai_state(k, 0);
                } else {
                    let b = self.ram.sprite(k).scratch_b().wrapping_sub(1);
                    self.ram.set_sprite_scratch_b(k, b);
                    if (b as i8) < 0 {
                        self.ram.set_sprite_scratch_b(k, 14);
                        self.ram
                            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
                    }
                    let i = ((self.ram.sprite(k).subtype2() & 1) << 2
                        | self.ram.sprite(k).scratch_a()) as usize;
                    self.ram.set_sprite_graphics(k, G[i]);
                }
            }
            _ => {}
        }
    }
    pub fn gibdo_draw(&mut self, k: usize) {
        const D: [DrawPiece; 24] = [
            (0, -9, 0x0080, 2),
            (0, 0, 0x008a, 2),
            (0, -8, 0x0080, 2),
            (0, 1, 0x408a, 2),
            (0, -9, 0x0082, 2),
            (0, 0, 0x008c, 2),
            (0, -8, 0x0082, 2),
            (0, 0, 0x008e, 2),
            (0, -9, 0x0084, 2),
            (0, 0, 0x00a0, 2),
            (0, -8, 0x0084, 2),
            (0, 1, 0x40a0, 2),
            (0, -9, 0x0086, 2),
            (0, 0, 0x00a2, 2),
            (0, -9, 0x0088, 2),
            (0, 0, 0x00a4, 2),
            (0, -9, 0x4088, 2),
            (0, 0, 0x40a4, 2),
            (0, -9, 0x4082, 2),
            (0, 0, 0x408c, 2),
            (0, -9, 0x4086, 2),
            (0, 0, 0x40a2, 2),
            (0, -8, 0x4082, 2),
            (0, 1, 0x408e, 2),
        ];
        let i = self.ram.sprite(k).graphics() as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
        if self.ram.sprite(k).pause() == 0 {
            self.runtime.draw_shadow(self.ram, k);
        }
    }

    pub fn flying_tile_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (0, 0, 0x00d3, 0),
            (8, 0, 0x40d3, 0),
            (0, 8, 0x80d3, 0),
            (8, 8, 0xc0d3, 0),
            (0, 0, 0x00c3, 0),
            (8, 0, 0x40c3, 0),
            (0, 8, 0x80c3, 0),
            (8, 8, 0xc0c3, 0),
        ];
        let i = self.ram.sprite(k).graphics() as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn flame_draw(&mut self, k: usize) {
        const D: [DrawPiece; 12] = [
            (0, 0, 0x018e, 2),
            (0, 0, 0x018e, 2),
            (0, 0, 0x01a0, 2),
            (0, 0, 0x01a0, 2),
            (0, 0, 0x418e, 2),
            (0, 0, 0x418e, 2),
            (0, 0, 0x41a0, 2),
            (0, 0, 0x41a0, 2),
            (0, 0, 0x01a2, 2),
            (0, 0, 0x01a2, 2),
            (0, -6, 0x01a4, 0),
            (8, -6, 0x01a5, 0),
        ];
        let i = self.ram.sprite(k).graphics() as usize * 2;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 2]);
    }

    pub fn spike_trap_draw(&mut self, k: usize) {
        const D: [DrawPiece; 4] = [
            (-8, -8, 0x00c4, 2),
            (8, -8, 0x40c4, 2),
            (-8, 8, 0x80c4, 2),
            (8, 8, 0xc0c4, 2),
        ];
        self.runtime.draw_multiple(self.ram, k, &D);
    }

    pub fn sprite_86_kodongo(&mut self, k: usize) {
        const X: [i8; 4] = [1, -1, 0, 0];
        const Y: [i8; 4] = [0, 0, 1, -1];
        const G: [u8; 8] = [2, 2, 0, 5, 3, 3, 0, 5];
        const F: [u8; 8] = [0x40, 0, 0, 0, 0x40, 0, 0x40, 0x40];
        const FG: [u8; 8] = [2, 2, 0, 5, 4, 4, 1, 6];
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.ram.set_sprite_flags(k, 0);
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_ai_state(k, 1);
                let d = self.runtime.random(self.ram) & 3;
                self.ram.set_sprite_scratch_d(k, d);
                self.ram.set_sprite_flags(k, 176);
                loop {
                    let j = self.ram.sprite(k).scratch_d() as usize;
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, X[j]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, Y[j]);
                    if self.runtime.check_tile_collision(self.ram, k) == 0 {
                        break;
                    }
                    self.ram.set_sprite_scratch_d(k, (j as u8 + 1) & 3);
                }
                self.kodongo_set_direction(k);
            }
            1 => {
                self.runtime.move_xy(self.ram, k);
                if self.runtime.check_tile_collision(self.ram, k) != 0 {
                    self.ram
                        .set_sprite_scratch_d(k, self.ram.sprite(k).scratch_d() ^ 1);
                    self.kodongo_set_direction(k);
                }
                if self.ram.sprite(k).x_low() & 0x1f == 4
                    && self.ram.sprite(k).y_low() & 0x1f == 0x1b
                    && self.runtime.random(self.ram) & 3 == 0
                {
                    self.ram.set_sprite_main_delay(k, 111);
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_scratch_a(k, 0);
                }
                let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(k, s);
                let j = ((s & 4) | self.ram.sprite(k).scratch_d()) as usize;
                self.ram.set_sprite_graphics(k, G[j]);
                self.ram
                    .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() & !0x40 | F[j]);
            }
            2 => {
                let t = self.ram.sprite(k).main_delay();
                if t == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                }
                let j = t.wrapping_sub(0x20) < 0x30;
                if j && t & 0xf == 0 {
                    self.kodongo_spawn_fire(k);
                }
                self.ram.set_sprite_graphics(
                    k,
                    FG[(j as usize) * 4 + self.ram.sprite(k).scratch_d() as usize],
                );
            }
            _ => {}
        }
    }
    pub fn kodongo_set_direction(&mut self, k: usize) {
        const X: [i8; 4] = [16, -16, 0, 0];
        const Y: [i8; 4] = [0, 0, 16, -16];
        let d = self.ram.sprite(k).scratch_d() as usize;
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityX, X[d]);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityY, Y[d]);
    }
    pub fn yellow_stalfos_animate(&mut self, k: usize) {
        const G: [u8; 4] = [6, 3, 1, 1];
        let d = self.ram.sprite(k).scratch_d() as usize;
        self.ram.set_sprite_graphics(k, G[d]);
        self.ram
            .set_sprite_flags3(k, self.ram.sprite(k).flags3() & !0x40);
    }
    pub fn initialize_spawned_bee(&mut self, k: usize) {
        const DELAY: [u8; 4] = [64, 64, 255, 255];
        const V: [i8; 8] = [15, 5, -5, -15, 20, 10, -10, -20];
        self.ram.set_sprite_ai_state(k, 1);
        let d = DELAY[k & 3];
        self.ram.set_sprite_scratch_a(k, d);
        self.ram.set_sprite_main_delay(k, d);
        self.ram.set_sprite_aux4_delay(k, 96);
        let rx = (self.runtime.random(self.ram) & 7) as usize;
        let ry = (self.runtime.random(self.ram) & 7) as usize;
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityX, V[rx]);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityY, V[ry]);
    }

    pub fn release_bee_from_bottle(&mut self, x_value: usize) -> i32 {
        const V: [i8; 8] = [8, 2, -2, -8, 10, 5, -5, -10];
        let Some((j, _, _, _)) = self.runtime.spawn_dynamic(self.ram, x_value, 0xb2) else {
            return -1;
        };
        self.ram
            .set_sprite_floor(j, self.ram.read_u8(LINK_IS_ON_LOWER_LEVEL));
        self.set_sprite_x(j, self.ram.read_u16(LINK_X).wrapping_add(8));
        self.set_sprite_y(j, self.ram.read_u16(LINK_Y).wrapping_add(16));
        let bottle = self.ram.read_u8(LINK_ITEM_BOTTLE_INDEX).wrapping_sub(1) as usize;
        if self.ram.read_u8(LINK_BOTTLE_INFO + bottle) == 8 {
            self.ram.set_sprite_head_direction(j, 1);
        }
        self.initialize_spawned_bee(j);
        let a = self.runtime.random(self.ram) as usize & 7;
        let b = self.runtime.random(self.ram) as usize & 7;
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityX, V[a]);
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityY, V[b]);
        self.ram.set_sprite_main_delay(j, 64);
        self.ram.set_sprite_scratch_a(j, 64);
        j as i32
    }
    pub fn player_bee_find_target(&mut self, k: usize) -> Option<(u16, u16)> {
        let mut n = 16;
        let mut j = (k * 4) & 0xf;
        loop {
            let valid =
                if j == k || self.ram.sprite(j).state() < 9 || self.ram.sprite(j).pause() != 0 {
                    false
                } else if self.ram.sprite(j).flags2() & 0x80 == 0 {
                    self.ram.sprite(k).floor() == self.ram.sprite(j).floor()
                        && self.ram.sprite(j).flags4() & 0x40 == 0
                        && self.ram.sprite(j).ignore_projectile() == 0
                } else {
                    self.ram.sprite(k).head_direction() != 0
                        && self.ram.sprite(j).bump_damage() & 0x40 != 0
                };
            if valid {
                self.runtime.player_bee_hone_in_on_target(self.ram, j, k);
                let x = self
                    .sprite_x(j)
                    .wrapping_add((self.runtime.random(self.ram) & 3) as u16 * 5);
                let y = self
                    .sprite_y(j)
                    .wrapping_add((self.runtime.random(self.ram) & 3) as u16 * 5);
                return Some((x, y));
            }
            j = j.wrapping_sub(1) & 0xf;
            n -= 1;
            if n == 0 {
                return None;
            }
        }
    }
    pub fn bee_main(&mut self, k: usize) {
        self.bee_handle_z(k);
        self.runtime.draw_single_small(self.ram, k);
        self.bee_handle_interactions(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        if self.ram.sprite(k).head_direction() != 0 {
            self.runtime.spawn_bee_sparkle(self.ram, k);
        }
        self.bee_bzzt(k);
        self.runtime.move_xy(self.ram, k);
        let f = (k as u8) ^ self.ram.read_u8(FRAME_COUNTER);
        self.ram.set_sprite_graphics(k, f >> 1 & 1);
        if self.ram.sprite(k).aux4_delay() == 0 {
            self.runtime.check_damage_to_link(self.ram, k);
            if self.runtime.check_damage_from_link(self.ram, k) & 2 != 0 {
                self.runtime.show_message(self.ram, 0xc8);
                self.ram.set_sprite_ai_state(k, 2);
                return;
            }
        }
        if self.ram.read_u8(FRAME_COUNTER) == 0 && self.ram.sprite(k).scratch_a() != 16 {
            self.ram
                .set_sprite_scratch_a(k, self.ram.sprite(k).scratch_a().wrapping_sub(8));
        }
        if self.ram.sprite(k).main_delay() == 0 {
            let x = self
                .ram
                .read_u16(LINK_X)
                .wrapping_add((self.runtime.random(self.ram) & 3) as u16 * 5);
            let y = self
                .ram
                .read_u16(LINK_Y)
                .wrapping_add((self.runtime.random(self.ram) & 3) as u16 * 5);
            let (yv, xv) = self.runtime.project_speed(self.ram, k, x, y, 20);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, yv);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, xv);
            self.ram.set_sprite_oam_flags(
                k,
                self.ram.sprite(k).oam_flags() & !0x40 | if xv < 0 { 0 } else { 0x40 },
            );
            self.ram
                .set_sprite_main_delay(k, (k as u8).wrapping_add(self.ram.sprite(k).scratch_a()));
        }
    }
    pub fn sprite_b2_player_bee(&mut self, k: usize) {
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.sprite(k).scratch_e() == 0 {
                    self.ram.set_sprite_state(k, 0);
                    let mut b = 0;
                    for i in 0..4 {
                        b |= self.ram.read_u8(LINK_BOTTLE_INFO + i);
                    }
                    if b & 8 == 0 {
                        self.gold_bee_spawn_self(k);
                    }
                }
            }
            1 => {
                self.ram.set_sprite_ignore_projectile(k, 1);
                self.bee_handle_z(k);
                self.runtime.draw_single_small(self.ram, k);
                self.bee_handle_interactions(k);
                if self.runtime.return_if_inactive(self.ram, k) {
                    return;
                }
                self.bee_bzzt(k);
                self.runtime.move_xy(self.ram, k);
                let f = (k as u8) ^ self.ram.read_u8(FRAME_COUNTER);
                self.ram.set_sprite_graphics(k, f >> 1 & 1);
                if self.ram.sprite(k).head_direction() != 0 {
                    self.runtime.spawn_bee_sparkle(self.ram, k);
                }
                if self.ram.sprite(k).scratch_b()
                    >= [10, 20][self.ram.sprite(k).head_direction() as usize]
                {
                    self.ram.set_sprite_deflection_bits(k, 64);
                    return;
                }
                if self.ram.sprite(k).aux4_delay() != 0 {
                    return;
                }
                if self.runtime.check_damage_from_link(self.ram, k) & 2 != 0 {
                    self.runtime.show_message(self.ram, 0xc8);
                    self.ram.set_sprite_ai_state(k, 2);
                    return;
                }
                if f & 3 != 0 {
                    return;
                }
                let (x, y) = self.player_bee_find_target(k).unwrap_or_else(|| {
                    let x = self
                        .ram
                        .read_u16(LINK_X)
                        .wrapping_add((self.runtime.random(self.ram) & 3) as u16 * 5);
                    let y = self
                        .ram
                        .read_u16(LINK_Y)
                        .wrapping_add((self.runtime.random(self.ram) & 3) as u16 * 5);
                    (x, y)
                });
                if f & 7 != 0 {
                    return;
                }
                let (yv, xv) = self.runtime.project_speed(self.ram, k, x, y, 32);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, xv);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, yv);
                self.ram.set_sprite_oam_flags(
                    k,
                    self.ram.sprite(k).oam_flags() & !0x40 | if xv < 0 { 0 } else { 0x40 },
                );
            }
            2 => self.runtime.bee_put_in_bottle(self.ram, k),
            _ => {}
        }
    }
    pub fn sprite_b3_pedestal_plaque(&mut self, k: usize) {
        self.runtime.prep_oam(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.read_u8(FLAG_IS_LINK_IMMOBILIZED) == 0
            && self.runtime.check_if_link_is_busy(self.ram)
        {
            return;
        }
        self.ram.write_u8(
            LINK_POSITION_MODE,
            self.ram.read_u8(LINK_POSITION_MODE) & !0x20,
        );
        let desert = self.ram.read_u8(OVERWORLD_SCREEN_INDEX) == 48;
        if self.ram.read_u8(LINK_DIRECTION_FACING) != 0
            || !self.runtime.check_damage_same_layer(self.ram, k)
        {
            return;
        }
        let book =
            self.ram.read_u8(HUD_CUR_ITEM) == 15 && self.ram.read_u8(FILTERED_JOYPAD_H) & 0x40 != 0;
        if !book {
            if self.ram.read_u8(FILTERED_JOYPAD_L) & 0x80 == 0 {
                return;
            }
            self.runtime
                .show_message(self.ram, if desert { 0xbc } else { 0xb6 });
        } else {
            self.ram.write_u8(PLAYER_HANDLER_TIMER, 0);
            self.ram.write_u8(LINK_POSITION_MODE, 32);
            self.ram.write_u8(SOUND_EFFECT_1, 0);
            if desert {
                self.ram.write_u8(BUTTON_B_FRAMES, 1);
                self.ram.write_u8(LINK_DELAY_TIMER_SPIN_ATTACK, 0);
                self.ram.write_u8(LINK_PLAYER_HANDLER_STATE, 27);
            }
            self.runtime
                .show_message(self.ram, if desert { 0xbd } else { 0xb7 });
        }
    }
    pub fn spawn_bee_from_hive(&mut self, k: usize) {
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0x79) {
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            self.initialize_spawned_bee(j);
        }
    }
    pub fn bee_dormant_hive(&mut self, k: usize) {
        if self.ram.sprite(k).scratch_e() != 0 {
            return;
        }
        self.ram.set_sprite_state(k, 0);
        for _ in 0..12 {
            self.spawn_bee_from_hive(k);
        }
    }
    pub fn bee_handle_interactions(&mut self, k: usize) {
        if self.ram.read_u8(SUBMODULE_INDEX) == 2
            && matches!(self.ram.read_u16(DIALOGUE_MESSAGE_INDEX), 0xc8 | 0xca)
        {
            self.ram.set_sprite_aux4_delay(k, 40);
        }
    }
    pub fn sprite_b5_bomb_shop(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.sprite_bomb_shop_clerk(k),
            1 => self.sprite_bomb_shop_bomb(k),
            2 => self.sprite_bomb_shop_super_bomb(k),
            3 => self.sprite_bomb_shop_huff(k),
            _ => {}
        }
    }
    pub fn sprite_b6_kiki(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.runtime.kiki_lying_in_wait(self.ram, k),
            1 => self.kiki_offer_entrance_service(k),
            2 => self.kiki_offer_initial_service(k),
            3 => self.kiki_flee(k),
            _ => {}
        }
    }
    pub fn old_man_enable_cutscene(&mut self) {
        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
        self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 1);
    }
    pub fn sprite_b8_dialogue_tester(&self, _k: usize) {
        panic!("Sprite_B8_DialogueTester: canonical C assertion")
    }
    pub fn sprite_b9_bully_and_pink_ball(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.sprite_pink_ball(k),
            1 => self.pink_ball_distress(k),
            2 => self.sprite_bully(k),
            _ => {}
        }
    }
    pub fn spawn_bully(&mut self, k: usize) {
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xb9) {
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            self.ram.set_sprite_subtype2(j, 2);
            self.ram.set_sprite_head_direction(j, k as u8);
            self.ram.set_sprite_ignore_projectile(j, 1);
        }
    }
    fn ball_message(&mut self, k: usize, normal: u16, pearl: u16, bounce: bool) {
        if self.ram.sprite(k).aux4_delay() != 0 {
            return;
        }
        let msg = if self.ram.read_u8(LINK_ITEM_MOON_PEARL) & 1 != 0 {
            pearl
        } else {
            normal
        };
        if self.runtime.show_message_on_contact(self.ram, k, msg) & 0x100 != 0 {
            self.ram
                .set_sprite_velocity_x(k, self.ram.sprite(k).velocity_x() ^ 255);
            self.ram
                .set_sprite_velocity_y(k, self.ram.sprite(k).velocity_y() ^ 255);
            if bounce && self.ram.sprite(k).scratch_e() != 0 {
                self.ball_guy_play_bounce_noise(k)
            }
            self.ram.set_sprite_aux4_delay(k, 64);
        }
    }
    pub fn pink_ball_handle_message(&mut self, k: usize) {
        self.ball_message(k, 0x15b, 0x15c, true)
    }
    pub fn bully_handle_message(&mut self, k: usize) {
        self.ball_message(k, 0x15d, 0x15e, false)
    }
    pub fn sprite_9c_zoro(&mut self, k: usize) {
        if self.ram.sprite(k).scratch_e() != 0 {
            self.zoro(k)
        } else {
            self.babasu(k)
        }
    }
    pub fn mothula_main(&mut self, k: usize) {
        self.runtime.mothula_draw(self.ram, k);
        if self.ram.sprite(k).state() == 11 {
            self.ram.set_sprite_ai_state(k, 0);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.set_sprite_flags3(
            k,
            if self.ram.sprite(k).aux3_delay() != 0 {
                64
            } else {
                0
            },
        );
        if self.ram.sprite(k).scratch_f() & 127 == 6 {
            self.ram.set_sprite_scratch_f(k, 0);
            self.ram.set_sprite_aux3_delay(k, 32);
            self.ram.set_sprite_ai_state(k, 2);
            self.ram.set_sprite_main_delay(k, 0);
            self.ram.set_sprite_scratch_g(k, 64);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 => {
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, 8);
                self.runtime.move_z(self.ram, k);
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, 0);
                if self.ram.sprite(k).z() >= 24 {
                    self.ram.set_sprite_scratch_g(k, 128);
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_ignore_projectile(k, 0);
                    self.ram.set_sprite_main_delay(k, 64);
                }
                self.mothula_flap_wings(k);
            }
            2 => {
                let g = self.ram.sprite(k).scratch_g();
                if g == 0 {
                    self.ram.set_sprite_main_delay(k, 63);
                    self.ram.set_sprite_ai_state(k, 3);
                    return;
                }
                self.ram.set_sprite_scratch_g(k, g - 1);
                self.mothula_flap_wings(k);
                let a = (self.ram.sprite(k).scratch_a() & 1) as usize;
                let z = self
                    .ram
                    .sprite(k)
                    .signed_field(SpriteField::VelocityZ)
                    .wrapping_add(if a != 0 { -1 } else { 1 });
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, z);
                if z == [16, -16][a] {
                    self.ram
                        .set_sprite_scratch_a(k, self.ram.sprite(k).scratch_a().wrapping_add(1));
                }
                if self.ram.sprite(k).main_delay() == 0 {
                    let c = self.ram.sprite(k).scratch_c().wrapping_add(1);
                    self.ram.set_sprite_scratch_c(k, c);
                    if c == 7 {
                        self.ram.set_sprite_scratch_c(k, 0);
                        self.runtime.apply_speed_towards_link(self.ram, k, 32);
                        self.ram.set_sprite_main_delay(k, 128);
                    } else {
                        const V: [i8; 10] = [-16, -12, 0, 12, 16, 12, 0, -12, -16, -12];
                        let j = self.runtime.random(self.ram) as usize & 7;
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityX, V[j + 2]);
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityY, V[j]);
                        let t = (self.runtime.random(self.ram) & 31) + 64;
                        self.ram.set_sprite_main_delay(k, t);
                    }
                }
                if self.ram.sprite(k).wall_collision() == 0 {
                    self.runtime.move_xy(self.ram, k);
                }
                self.runtime.move_z(self.ram, k);
                if self.runtime.check_tile_collision(self.ram, k) != 0 {
                    self.ram.set_sprite_main_delay(k, 0);
                }
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                self.ram
                    .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(2));
            }
            3 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                let t = self.ram.sprite(k).main_delay();
                if t == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    let duration = self.runtime.random(self.ram) & 31 | 64;
                    self.ram.set_sprite_scratch_g(k, duration);
                } else {
                    if t == 0x20 {
                        self.mothula_spawn_beams(k);
                    }
                    self.mothula_flap_wings(k);
                }
            }
            _ => {}
        }
    }
    pub fn mothula_flap_wings(&mut self, k: usize) {
        const G: [u8; 4] = [0, 1, 2, 1];
        let s = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, s);
        let j = (s >> 2 & 3) as usize;
        if j == 0 {
            self.runtime.queue_sfx3(self.ram, k, 2);
        }
        self.ram.set_sprite_graphics(k, G[j]);
    }
    pub fn nice_thief_animate(&mut self, k: usize) {
        if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 {
            self.ram.set_sprite_graphics(k, 2);
            let d = self.runtime.direction_to_face_link(self.ram, k);
            self.ram
                .set_sprite_head_direction(k, if d == 3 { 2 } else { d });
        }
        self.runtime.allocate_oam_defer_to_player(self.ram, k);
        self.runtime.thief_draw(self.ram, k);
    }
    pub fn nice_thief_under_rock(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        let message = if self.ram.sprite(k).subtype2() == 5 {
            0x177
        } else {
            0x178
        };
        self.runtime.show_solicited_message(self.ram, k, message);
    }
    pub fn sprite_bc_drunkard(&mut self, k: usize) {
        self.runtime.drinking_guy_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if self.runtime.random(self.ram) == 0 {
            self.ram.set_sprite_main_delay(k, 32);
        }
        self.ram
            .set_sprite_graphics(k, u8::from(self.ram.sprite(k).main_delay() != 0));
        if self.runtime.show_solicited_message(self.ram, k, 0x175) & 0x100 != 0 {
            self.ram.set_sprite_graphics(k, 0);
        }
    }
    pub fn mothula_spawn_beams(&mut self, k: usize) {
        const X: [i8; 3] = [-16, 0, 16];
        const Y: [i8; 3] = [24, 32, 24];
        self.runtime.queue_sfx3(self.ram, k, 0x36);
        for i in (0..3).rev() {
            if let Some((j, x, y, z)) = self.runtime.spawn_dynamic(self.ram, k, 0x89) {
                self.set_sprite_x(j, x.wrapping_add_signed(X[i] as i16));
                self.set_sprite_y(j, y.wrapping_sub(z as u16).wrapping_add(3));
                self.ram.set_sprite_main_delay(j, 16);
                self.ram.set_sprite_ignore_projectile(j, 16);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityX, X[i]);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityY, Y[i]);
                self.ram.set_sprite_z(j, 0);
            }
        }
        self.ram.write_u8(TMP_COUNTER, 0xff);
    }
    pub fn yellow_stalfos_emancipate_head(&mut self, k: usize) {
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 2) {
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            self.ram.set_sprite_z(j, 13);
            self.runtime.apply_speed_towards_link(self.ram, j, 16);
            self.ram.set_sprite_main_delay(j, 255);
            self.ram.set_sprite_aux1_delay(j, 32);
        }
    }
    pub fn sprite_prep_eyegore(&mut self, k: usize) {
        if matches!(self.ram.read_u8(DUNGEON_ROOM_INDEX2), 12 | 27 | 75 | 107) {
            self.ram
                .set_sprite_scratch_b(k, self.ram.sprite(k).scratch_b().wrapping_add(1));
            if self.ram.sprite(k).kind() == 0x83 {
                self.ram.set_sprite_deflection_bits(k, 0);
            }
        }
    }
    pub fn sprite_83_green_eyegore(&mut self, k: usize) {
        const XV: [i8; 32] = [
            0, 16, -16, 0, 0, 13, -13, 0, 0, 13, -13, 0, 0, 0, 0, 0, 0, -24, 24, 0, 0, -16, 16, 0,
            0, -16, 16, 0, 0, 0, 0, 0,
        ];
        const YV: [i8; 32] = [
            0, 0, 0, 0, -16, -5, -5, 0, 16, 13, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -16, -16, 0,
            24, 16, 16, 0, 0, 0, 0, 0,
        ];
        const DIR: [u8; 32] = [
            0, 0, 1, 0, 3, 3, 3, 0, 2, 2, 2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 3, 3, 3, 0, 2, 2, 2, 0, 0,
            0, 0, 0,
        ];
        const GFX: [u8; 16] = [8, 6, 0, 3, 9, 7, 1, 4, 8, 6, 0, 3, 9, 7, 2, 5];
        if self.ram.sprite(k).scratch_b() == 0 {
            self.eyegore_main(k);
            return;
        }
        self.runtime.goriya_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        if self.ram.sprite(k).aux1_delay() == 8 {
            self.sprite_spawn_fire_phlegm(k);
        }
        if self.ram.read_u8(BITMASK_OF_DRAGSTATE) != 0
            || self.ram.read_u8(JOYPAD1_H_LAST) & 0xf == 0
        {
            self.ram.set_sprite_scratch_a(k, 0);
            self.runtime.check_damage_to_and_from_link(self.ram, k);
            self.runtime.check_tile_collision(self.ram, k);
            return;
        }
        let j = (self.ram.read_u8(JOYPAD1_H_LAST) & 0xf) as usize
            + if self.ram.sprite(k).kind() == 0x84 {
                16
            } else {
                0
            };
        self.ram.set_sprite_scratch_d(k, DIR[j]);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityX, XV[j]);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityY, YV[j]);
        if self.ram.sprite(k).wall_collision() == 0 {
            self.runtime.move_xy(self.ram, k);
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.runtime.check_tile_collision(self.ram, k);
        let s = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, s);
        self.ram
            .set_sprite_graphics(k, GFX[((s & 12) | self.ram.sprite(k).scratch_d()) as usize]);
        if self.ram.sprite(k).kind() == 0x84 {
            let dir = self.runtime.direction_to_face_link(self.ram, k);
            let dx = self
                .ram
                .read_u8(LINK_X)
                .wrapping_sub(self.ram.sprite(k).x_low());
            let dy = self
                .ram
                .read_u8(LINK_Y)
                .wrapping_add(8)
                .wrapping_add(self.ram.sprite(k).z())
                .wrapping_sub(self.ram.sprite(k).y_low());
            if (dx.wrapping_add(8) < 16 || dy.wrapping_add(8) < 16)
                && self.ram.sprite(k).scratch_d() == dir
            {
                if self.ram.sprite(k).scratch_a() & 0x1f == 0 {
                    self.ram.set_sprite_aux1_delay(k, 16);
                }
                self.ram
                    .set_sprite_scratch_a(k, self.ram.sprite(k).scratch_a().wrapping_add(1));
                return;
            }
        }
        self.ram.set_sprite_scratch_a(k, 0);
    }
    pub fn eyegore_main(&mut self, k: usize) {
        const CLOSE: [u8; 8] = [0, 0, 1, 1, 2, 2, 2, 2];
        const OPEN: [u8; 8] = [2, 2, 2, 2, 1, 1, 0, 0];
        const CHASE: [u8; 16] = [7, 5, 2, 9, 8, 6, 3, 10, 7, 5, 2, 9, 8, 6, 4, 11];
        const DELAY: [u8; 4] = [0x60, 0x80, 0xa0, 0x80];
        const XV: [i8; 4] = [16, -16, 0, 0];
        const YV: [i8; 4] = [0, 0, 16, -16];
        self.eyegore_draw(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.ram
            .set_sprite_flags3(k, self.ram.sprite(k).flags3() | 64);
        self.ram
            .set_sprite_deflection_bits(k, self.ram.sprite(k).deflection_bits() | 4);
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    // C calls Sprite_DirectionToFaceLink even when the range
                    // check fails; it also supplies the Z-adjusted Y delta.
                    self.runtime.direction_to_face_link(self.ram, k);
                    let dx = self
                        .ram
                        .read_u8(LINK_X)
                        .wrapping_sub(self.ram.sprite(k).x_low());
                    let dy = self
                        .ram
                        .read_u8(LINK_Y)
                        .wrapping_add(8)
                        .wrapping_add(self.ram.sprite(k).z())
                        .wrapping_sub(self.ram.sprite(k).y_low());
                    if dx.wrapping_add(48) < 96 && dy.wrapping_add(48) < 96 {
                        self.ram.set_sprite_ai_state(k, 1);
                        self.ram.set_sprite_main_delay(k, 63);
                    }
                }
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    let d = self.runtime.direction_to_face_link(self.ram, k);
                    self.ram.set_sprite_scratch_d(k, d);
                    self.ram.set_sprite_ai_state(k, 2);
                    let i = self.runtime.random(self.ram) as usize & 3;
                    self.ram.set_sprite_main_delay(k, DELAY[i]);
                } else {
                    self.ram.set_sprite_graphics(
                        k,
                        OPEN[(self.ram.sprite(k).main_delay() >> 3) as usize],
                    );
                }
            }
            2 => {
                self.ram
                    .set_sprite_flags3(k, self.ram.sprite(k).flags3() & !0x40);
                if self.ram.sprite(k).kind() != 0x84 {
                    self.ram
                        .set_sprite_deflection_bits(k, self.ram.sprite(k).deflection_bits() & !4);
                }
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_main_delay(k, 63);
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_graphics(k, 0);
                } else {
                    if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 31 == 0 {
                        let d = self.runtime.direction_to_face_link(self.ram, k);
                        self.ram.set_sprite_scratch_d(k, d);
                    }
                    let d = self.ram.sprite(k).scratch_d() as usize;
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, XV[d]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, YV[d]);
                    if self.ram.sprite(k).wall_collision() == 0 {
                        self.runtime.move_xy(self.ram, k);
                    }
                    self.runtime.check_tile_collision(self.ram, k);
                    let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                    self.ram.set_sprite_subtype2(k, s);
                    self.ram
                        .set_sprite_graphics(k, CHASE[((s & 12) | d as u8) as usize]);
                }
            }
            3 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    self.ram.set_sprite_main_delay(k, 96);
                } else {
                    self.ram.set_sprite_graphics(
                        k,
                        CLOSE[(self.ram.sprite(k).main_delay() >> 3) as usize],
                    );
                }
            }
            _ => {}
        }
    }
    pub fn eyegore_draw(&mut self, k: usize) {
        const D: [DrawPiece; 48] = [
            (-4, -4, 0x00a2, 2),
            (4, -4, 0x40a2, 2),
            (-4, 4, 0x009c, 2),
            (4, 4, 0x409c, 2),
            (-4, -4, 0x00a4, 2),
            (4, -4, 0x40a4, 2),
            (-4, 4, 0x009c, 2),
            (4, 4, 0x409c, 2),
            (-4, -4, 0x008c, 2),
            (4, -4, 0x408c, 2),
            (-4, 4, 0x009c, 2),
            (4, 4, 0x409c, 2),
            (-4, -3, 0x008c, 2),
            (12, -3, 0x408c, 0),
            (-4, 13, 0x00bc, 0),
            (4, 5, 0x408a, 2),
            (-4, -3, 0x008c, 0),
            (4, -3, 0x408c, 2),
            (-4, 5, 0x008a, 2),
            (12, 13, 0x40bc, 0),
            (0, -4, 0x00aa, 2),
            (0, 4, 0x00a6, 2),
            (0, -4, 0x00aa, 2),
            (0, 4, 0x00a6, 2),
            (0, -3, 0x00aa, 2),
            (0, 4, 0x00a8, 2),
            (0, -3, 0x00aa, 2),
            (0, 4, 0x00a8, 2),
            (0, -4, 0x40aa, 2),
            (0, 4, 0x40a6, 2),
            (0, -4, 0x40aa, 2),
            (0, 4, 0x40a6, 2),
            (0, -3, 0x40aa, 2),
            (0, 4, 0x40a8, 2),
            (0, -3, 0x40aa, 2),
            (0, 4, 0x40a8, 2),
            (-4, -4, 0x008e, 2),
            (4, -4, 0x408e, 2),
            (-4, 4, 0x009e, 2),
            (4, 4, 0x409e, 2),
            (-4, -3, 0x008e, 2),
            (12, -3, 0x408e, 0),
            (-4, 13, 0x00bd, 0),
            (4, 5, 0x40a0, 2),
            (-4, -3, 0x008e, 0),
            (4, -3, 0x408e, 2),
            (-4, 5, 0x00a0, 2),
            (12, 13, 0x40bd, 0),
        ];
        let i = self.ram.sprite(k).graphics() as usize * 4;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
        if self.ram.sprite(k).pause() == 0 {
            self.runtime.draw_shadow_custom(self.ram, k, 14);
        }
    }
    pub fn seeker_energy_ball_draw(&mut self, k: usize) {
        const D: [DrawPiece; 8] = [
            (4, -3, 0x00ce, 0),
            (11, 4, 0x00ce, 0),
            (4, 11, 0x00ce, 0),
            (-3, 4, 0x00ce, 0),
            (-1, -1, 0x00ce, 0),
            (9, -1, 0x00ce, 0),
            (-1, 9, 0x00ce, 0),
            (9, 9, 0x00ce, 0),
        ];
        let i = ((self.ram.sprite(k).subtype2() >> 2 & 1) * 4) as usize;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
    }
    pub fn sprite_79_bee(&mut self, k: usize) {
        match self.ram.sprite(k).ai_state() {
            0 => self.bee_dormant_hive(k),
            1 => self.bee_main(k),
            2 => self.runtime.bee_put_in_bottle(self.ram, k),
            _ => {}
        }
    }
    pub fn sprite_7e_firebar_clockwise(&mut self, k: usize) {
        const INC: [i16; 4] = [-2, 2, -1, 1];
        self.firebar_main(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        let j = (self.ram.sprite(k).kind() - 0x7e) as usize
            + usize::from(self.ram.read_u8(CUR_PALACE_INDEX_X2) == 18) * 2;
        let angle = (self.ram.sprite(k).scratch_a() as i16
            | (self.ram.sprite(k).scratch_b() as i16) << 8)
            .wrapping_add(INC[j])
            & 0x1ff;
        self.ram.set_sprite_scratch_a(k, angle as u8);
        self.ram.set_sprite_scratch_b(k, (angle >> 8) as u8);
    }
    pub fn firebar_main(&mut self, k: usize) {
        let Some((x, y, base_flags)) = self.runtime.firebar_prep_oam(self.ram, k) else {
            return;
        };
        self.ram.write_u8(0xfb6, base_flags);
        let angle =
            self.ram.sprite(k).scratch_a() as u16 | (self.ram.sprite(k).scratch_b() as u16) << 8;
        let s = self.runtime.guruguru_bar_sin(angle, 0x40);
        let c = self.runtime.guruguru_bar_sin((angle + 0x80) & 0x1ff, 0x40);
        let flags = (self.ram.sprite(k).subtype2() << 4 & 0xc0) | base_flags;
        let first = (usize::from(self.ram.read_u16(0x90)) - oam::OAM_BUFFER) / 4;
        for (offset, i) in (0..4).rev().enumerate() {
            let n = (i + 1) as i16;
            oam::set_plain(
                self.ram,
                first + offset,
                x.wrapping_add_signed(s as i16 * n / 4) as u8,
                y.wrapping_add_signed(c as i16 * n / 4) as u8,
                0x28,
                flags,
                2,
            );
        }
        self.runtime.correct_oam_entries(self.ram, k, 3, 0xff);
        if ((k as u8) ^ self.ram.read_u8(FRAME_COUNTER)) & 3 == 0
            && self.ram.read_u8(SUBMODULE_INDEX) == 0
            && self.ram.read_u8(FLAG_UNK1) == 0
        {
            let lx = self.ram.read_u16(LINK_X);
            let ly = self.ram.read_u16(LINK_Y);
            for index in first..first + 4 {
                let piece = oam::read(self.ram, index);
                if self.ram.read_u8(oam::BYTEWISE_EXTENDED_OAM + index) & 1 == 0
                    && (u16::from(piece.x)
                        .wrapping_add(self.ram.read_u16(0xe2))
                        .wrapping_sub(lx)
                        .wrapping_add(12) as u8)
                        < 24
                    && piece.y < 0xf0
                    && (u16::from(piece.y)
                        .wrapping_add(self.ram.read_u16(0xe8))
                        .wrapping_sub(ly)
                        .wrapping_add(4) as u8)
                        < 16
                {
                    self.runtime.attempt_damage_to_link_plus_recoil(self.ram, k);
                }
            }
        }
    }
    pub fn shop_item_red_potion150(&mut self, k: usize) {
        self.sprite_draw_shop_item(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if self.shop_item_check_for_a_press(k) {
            if self.sprite_find_empty_bottle() < 0 {
                self.reject_purchase(k, 0x16d);
            } else if self.shop_item_handle_cost(150) {
                self.ram.set_sprite_state(k, 0);
                self.shop_item_handle_receipt(k, 0x2e);
            } else {
                self.reject_purchase(k, 0x17c);
            }
        }
    }
    pub fn sprite_b4_purple_chest(&mut self, k: usize) {
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).ai_state() == 0 {
            if self.runtime.show_message_on_contact(self.ram, k, 0x116) & 0x100 != 0
                && self.ram.read_u8(FOLLOWER_INDICATOR) == 0
            {
                self.ram.set_sprite_ai_state(k, 1)
            }
        } else {
            self.ram.set_sprite_state(k, 0);
            self.ram.write_u8(FOLLOWER_INDICATOR, 12);
            self.runtime.load_follower_graphics(self.ram);
            self.runtime.become_follower(self.ram, k)
        }
    }
    pub fn sprite_b7_blind_maiden(&mut self, k: usize) {
        self.runtime.crystal_maiden_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.track_body_to_head(self.ram, k);
        let d = self.runtime.direction_to_face_link(self.ram, k) ^ 3;
        self.ram.set_sprite_head_direction(k, d);
        if self.ram.sprite(k).ai_state() == 0 {
            if self.runtime.show_message_on_contact(self.ram, k, 0x122) & 0x100 != 0 {
                self.ram.set_sprite_ai_state(k, 1)
            }
        } else {
            self.ram.set_sprite_state(k, 0);
            self.ram.write_u8(FOLLOWER_INDICATOR, 6);
            self.runtime.load_follower_graphics(self.ram);
            self.runtime.become_follower(self.ram, k)
        }
    }

    pub fn shop_item_handle_cost(&mut self, amount: u16) -> bool {
        let rupees = self.ram.read_u16(LINK_RUPEES_GOAL);
        if amount > rupees {
            return false;
        }
        self.ram
            .write_u16(LINK_RUPEES_GOAL, rupees.wrapping_sub(amount));
        true
    }

    pub fn sprite_find_empty_bottle(&self) -> i32 {
        (0..4)
            .find(|&i| self.ram.read_u8(LINK_BOTTLE_INFO + i) == 2)
            .map_or(-1, |i| i as i32)
    }

    pub fn bee_handle_z(&mut self, k: usize) {
        self.ram.set_sprite_z(k, 16);
        if self.ram.sprite(k).head_direction() != 0 {
            let flags = self.ram.sprite(k).oam_flags() & 0xf1;
            let palette = (((self.ram.read_u8(FRAME_COUNTER) >> 4) & 3) + 1) << 1;
            self.ram.set_sprite_oam_flags(k, flags | palette);
        }
    }

    pub fn bee_bzzt(&mut self, k: usize) {
        if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 31 == 0 {
            self.runtime.queue_sfx3(self.ram, k, 0x2c);
        }
    }

    pub fn pink_ball_handle_deceleration(&mut self, k: usize) {
        let x = self.ram.sprite(k).signed_field(SpriteField::VelocityX);
        if x != 0 {
            self.ram.set_sprite_signed_field(
                k,
                SpriteField::VelocityX,
                x.wrapping_add(if x < 0 { 2 } else { -2 }),
            );
        }
        let y = self.ram.sprite(k).signed_field(SpriteField::VelocityY);
        if y != 0 {
            self.ram.set_sprite_signed_field(
                k,
                SpriteField::VelocityY,
                y.wrapping_add(if y < 0 { 2 } else { -2 }),
            );
        }
    }

    pub fn ball_guy_play_bounce_noise(&mut self, k: usize) {
        self.runtime.queue_sfx3(self.ram, k, 0x32);
    }

    pub fn sprite_bomb_shop_huff(&mut self, k: usize) {
        const FLAGS: [u8; 4] = [4, 0x44, 0xc4, 0x84];
        self.runtime.allocate_oam_region_c(self.ram, 4);
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let flags = self.ram.sprite(k).oam_flags() & 0x30
            | FLAGS[(self.ram.read_u8(FRAME_COUNTER) >> 2 & 3) as usize];
        self.ram.set_sprite_oam_flags(k, flags);
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_add(1));
        self.runtime.move_z(self.ram, k);
        let delay = self.ram.sprite(k).main_delay();
        if delay == 0 {
            self.ram.set_sprite_state(k, 0);
        }
        self.ram.set_sprite_graphics(k, delay >> 3 & 3);
    }
    pub fn bomb_shop_clerk_exhalation(&mut self, k: usize) {
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xb5) {
            self.set_sprite_x(j, x.wrapping_add(4));
            self.set_sprite_y(j, y.wrapping_add(16));
            self.ram.set_sprite_subtype2(j, 3);
            self.ram.set_sprite_ignore_projectile(j, 3);
            self.ram.set_sprite_z(j, 4);
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityZ, -12);
            self.ram.set_sprite_main_delay(j, 23);
            self.ram
                .set_sprite_flags3(j, self.ram.sprite(j).flags3() & !0x11);
        }
    }
    pub fn shop_keeper_spawn_shop_item(&mut self, k: usize, pos: usize, what: u8) {
        const X: [i8; 3] = [-44, 8, 60];
        let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xbb) else {
            panic!("Sprite_SpawnDynamicallyEx invariant")
        };
        self.ram.set_sprite_ignore_projectile(j, what);
        self.ram.set_sprite_subtype2(j, what);
        self.set_sprite_x(j, x.wrapping_add_signed(X[pos] as i16));
        self.set_sprite_y(j, y.wrapping_add(0x27));
        self.ram
            .set_sprite_flags2(j, self.ram.sprite(j).flags2() | 4);
    }
    pub fn sprite_a2_kholdstare(&mut self, k: usize) {
        self.runtime.kholdstare_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).ai_state() < 2 {
            self.kholdstare_spawn_puff_cloud_garnish(k);
            if self.ram.read_u8(FRAME_COUNTER) & 7 == 0 {
                self.ram.write_u8(SOUND_EFFECT_1, 2);
            }
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        let s = self.ram.sprite(k).subtype2().wrapping_sub(1);
        self.ram.set_sprite_subtype2(k, s);
        if (s as i8) < 0 {
            self.ram.set_sprite_subtype2(k, 10);
            self.ram
                .set_sprite_graphics(k, (self.ram.sprite(k).graphics() + 1) & 3);
        }
        if self.ram.read_u8(FRAME_COUNTER) & 3 == 0 {
            let (y, x) = self.runtime.project_speed_towards_link(self.ram, k, 31);
            self.ram
                .set_sprite_scratch_a(k, self.runtime.convert_velocity_to_angle(x, y));
        }
        self.runtime.move_xy(self.ram, k);
        let state = self.ram.sprite(k).ai_state();
        if state < 2 {
            self.runtime.check_damage_to_and_from_link(self.ram, k);
        }
        match state {
            0 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    let d = (self.runtime.random(self.ram) & 63) + 32;
                    self.ram.set_sprite_main_delay(k, d);
                    return;
                }
                let tx = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
                let ty = self.ram.sprite(k).signed_field(SpriteField::SubpixelZ);
                let x = self.ram.sprite(k).signed_field(SpriteField::VelocityX);
                let y = self.ram.sprite(k).signed_field(SpriteField::VelocityY);
                if x != tx {
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::VelocityX,
                        x.wrapping_add(if x.wrapping_sub(tx) < 0 { 1 } else { -1 }),
                    );
                }
                if y != ty {
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::VelocityY,
                        y.wrapping_add(if y.wrapping_sub(ty) < 0 { 1 } else { -1 }),
                    );
                }
                self.kholdstare_check_collision(k);
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    let d = (self.runtime.random(self.ram) & 63) + 96;
                    self.ram.set_sprite_main_delay(k, d);
                    let r = self.runtime.random(self.ram);
                    if r & 0x1c == 0 {
                        let (y, x) = self.runtime.project_speed_towards_link(self.ram, k, 24);
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::VelocityZ, x);
                        self.ram
                            .set_sprite_signed_field(k, SpriteField::SubpixelZ, y);
                    } else {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityZ,
                            [16, 16, -16, -16][(r & 3) as usize],
                        );
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::SubpixelZ,
                            [-16, 16, 16, -16][(r & 3) as usize],
                        );
                    }
                } else {
                    let x = self.ram.sprite(k).signed_field(SpriteField::VelocityX);
                    let y = self.ram.sprite(k).signed_field(SpriteField::VelocityY);
                    if x != 0 {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityX,
                            x.wrapping_add(if x < 0 { 1 } else { -1 }),
                        );
                    }
                    if y != 0 {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityY,
                            y.wrapping_add(if y < 0 { 1 } else { -1 }),
                        );
                    }
                    self.kholdstare_check_collision(k);
                }
            }
            2 => {
                if self.ram.sprite(k).main_delay() == 1 {
                    self.ram.set_sprite_state(k, 0);
                    self.ram.set_sprite_state(k + 1, 0);
                    self.ram.set_sprite_state(k + 2, 0);
                    for i in (0..3).rev() {
                        if let Some((j, x, y, z)) =
                            self.runtime.spawn_dynamic_ex(self.ram, k, 0xa2, 4)
                        {
                            self.set_sprite_x(j, x);
                            self.set_sprite_y(j, y);
                            self.ram.set_sprite_z(j, z);
                            self.ram.set_sprite_signed_field(
                                j,
                                SpriteField::VelocityZ,
                                [32, -32, 0][i],
                            );
                            self.ram.set_sprite_signed_field(
                                j,
                                SpriteField::SubpixelZ,
                                [-32, -32, 48][i],
                            );
                            self.ram.set_sprite_main_delay(j, 32);
                        }
                    }
                    self.ram.write_u8(TMP_COUNTER, 0xff);
                } else {
                    self.ram
                        .set_sprite_hit_timer(k, self.ram.sprite(k).hit_timer() | 0xe0);
                }
            }
            _ => {}
        }
    }
    fn kholdstare_check_collision(&mut self, k: usize) {
        let c = self.runtime.check_tile_collision(self.ram, k);
        if c & 3 != 0 {
            self.ram.set_sprite_signed_field(
                k,
                SpriteField::VelocityX,
                self.ram
                    .sprite(k)
                    .signed_field(SpriteField::VelocityX)
                    .wrapping_neg(),
            );
            self.ram.set_sprite_signed_field(
                k,
                SpriteField::VelocityZ,
                self.ram
                    .sprite(k)
                    .signed_field(SpriteField::VelocityZ)
                    .wrapping_neg(),
            );
        }
        if c & 12 != 0 {
            self.ram.set_sprite_signed_field(
                k,
                SpriteField::VelocityY,
                self.ram
                    .sprite(k)
                    .signed_field(SpriteField::VelocityY)
                    .wrapping_neg(),
            );
            self.ram.set_sprite_signed_field(
                k,
                SpriteField::SubpixelZ,
                self.ram
                    .sprite(k)
                    .signed_field(SpriteField::SubpixelZ)
                    .wrapping_neg(),
            );
        }
    }
    pub fn kholdstare_spawn_puff_cloud_garnish(&mut self, k: usize) {
        const O: [i8; 8] = [-8, -6, -4, -2, 0, 2, 4, 6];
        if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 3 != 0 {
            return;
        }
        let Some(j) = self.runtime.garnish_alloc_low(self.ram) else {
            return;
        };
        self.ram.write_u8(GARNISH_TYPE + j, 7);
        self.ram.write_u8(GARNISH_ACTIVE, 7);
        self.ram.write_u8(GARNISH_COUNTDOWN + j, 31);
        let ox = O[(self.runtime.random(self.ram) & 7) as usize];
        let oy = O[(self.runtime.random(self.ram) & 7) as usize];
        self.set_garnish_x(j, self.sprite_x(k).wrapping_add_signed(ox as i16));
        self.set_garnish_y(
            j,
            self.sprite_y(k)
                .wrapping_add_signed(oy as i16)
                .wrapping_add(16),
        );
        self.ram.write_u8(GARNISH_FLOOR + j, 0);
    }
    pub fn laser_beam_build_up_garnish(&mut self, k: usize) {
        let j = self.runtime.garnish_alloc_overwrite_old(self.ram);
        self.ram.write_u8(GARNISH_TYPE + j, 4);
        self.ram.write_u8(GARNISH_ACTIVE, 4);
        self.set_garnish_x(j, self.sprite_x(k));
        self.set_garnish_y(j, self.sprite_y(k).wrapping_add(16));
        self.ram.write_u8(GARNISH_COUNTDOWN + j, 16);
        self.ram
            .write_u8(GARNISH_OAM_FLAGS + j, self.ram.sprite(k).graphics());
        self.ram.write_u8(GARNISH_SPRITE + j, k as u8);
        self.ram
            .write_u8(GARNISH_FLOOR + j, self.ram.sprite(k).floor());
    }
    pub fn sprite_94_pirogusu(&mut self, k: usize) {
        const A0: [u8; 4] = [2, 3, 0, 1];
        const A1: [u8; 8] = [9, 11, 5, 7, 5, 11, 7, 9];
        const A2: [u8; 8] = [16, 17, 18, 19, 12, 13, 14, 15];
        const V: [i8; 6] = [0, 0, 4, -4, 0, 0];
        const V2: [i8; 6] = [2, -2, 0, 0, 2, -2];
        const V3: [i8; 6] = [24, -24, 0, 0, 24, -24];
        if self.ram.sprite(k).scratch_e() != 0 {
            self.runtime.sprite_94_tile(self.ram, k);
            return;
        }
        self.ram
            .set_sprite_object_priority(k, self.ram.sprite(k).object_priority() | 0x30);
        self.pirogusu_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let d = self.ram.sprite(k).scratch_d() as usize;
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_main_delay(k, 31);
                }
                self.ram
                    .set_sprite_ignore_projectile(k, self.ram.sprite(k).main_delay());
                self.ram.set_sprite_scratch_a(k, A0[d]);
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_main_delay(k, 32);
                    self.ram.set_sprite_ignore_projectile(k, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, 0);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, 0);
                } else {
                    let i = ((self.ram.sprite(k).main_delay() >> 3 & 1) | (d as u8) << 1) as usize;
                    self.ram.set_sprite_scratch_a(k, A1[i]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, V[d + 2]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, V[d]);
                    self.runtime.move_xy(self.ram, k);
                }
            }
            2 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                self.runtime.move_xy(self.ram, k);
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_add(V2[d]),
                );
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityY,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityY)
                        .wrapping_add(V2[d + 2]),
                );
                if self.ram.sprite(k).main_delay() == 0 {
                    self.runtime.spawn_small_splash(self.ram, k);
                    self.ram.set_sprite_aux1_delay(k, 16);
                    self.ram.set_sprite_ai_state(k, 3);
                }
                let i = ((self.ram.read_u8(FRAME_COUNTER) >> 2 & 1) | (d as u8) << 1) as usize;
                self.ram.set_sprite_scratch_a(k, A2[i]);
            }
            3 => {
                if self.runtime.return_if_recoiling(self.ram, k) {
                    return;
                }
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                let i = ((self.ram.read_u8(FRAME_COUNTER) >> 2 & 1) | (d as u8) << 1) as usize;
                self.ram.set_sprite_scratch_a(k, A2[i] + 8);
                if self.ram.sprite(k).aux1_delay() == 0 {
                    self.pirogusu_spawn_splash(k);
                    self.runtime.move_xy(self.ram, k);
                    if self.runtime.check_tile_collision(self.ram, k) & 15 != 0 {
                        let q = [2, 3, 2, 3, 0, 1, 0, 1]
                            [d * 2 + (self.runtime.random(self.ram) & 1) as usize];
                        self.ram.set_sprite_scratch_d(k, q);
                    }
                    let q = self.ram.sprite(k).scratch_d() as usize;
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, V3[q]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, V3[q + 2]);
                }
            }
            _ => {}
        }
    }
    pub fn pirogusu_spawn_splash(&mut self, k: usize) {
        const T: [u8; 4] = [3, 4, 5, 4];
        if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 3 != 0 {
            return;
        }
        let x = T[(self.runtime.random(self.ram) & 3) as usize];
        let y = T[(self.runtime.random(self.ram) & 3) as usize];
        if let Some(j) = self.runtime.garnish_alloc_low(self.ram) {
            self.ram.write_u8(GARNISH_TYPE + j, 11);
            self.ram.write_u8(GARNISH_ACTIVE, 11);
            self.set_garnish_x(j, self.sprite_x(k).wrapping_add(x as u16));
            self.set_garnish_y(j, self.sprite_y(k).wrapping_add(y as u16).wrapping_add(16));
            self.ram.write_u8(GARNISH_COUNTDOWN + j, 15);
        }
    }

    pub fn pirogusu_draw(&mut self, k: usize) {
        const FLAGS: [u8; 28] = [
            0, 0x80, 0x40, 0, 0, 0, 0, 0x80, 0x80, 0xc0, 0x40, 0x40, 0, 0x40, 0x80, 0xc0, 0x40,
            0xc0, 0, 0x80, 0, 0x40, 0x80, 0xc0, 0x40, 0xc0, 0, 0x80,
        ];
        const G: [u8; 28] = [
            0, 0, 1, 1, 2, 3, 4, 3, 2, 3, 4, 3, 5, 5, 5, 5, 7, 7, 7, 7, 6, 6, 6, 6, 8, 8, 8, 8,
        ];
        let j = self.ram.sprite(k).scratch_a() as usize;
        self.ram
            .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() & 0x3f | FLAGS[j]);
        self.ram.set_sprite_graphics(k, G[j]);
        if j < 4 {
            self.ram.write_u16(
                CUR_SPRITE_X,
                self.ram.read_u16(CUR_SPRITE_X).wrapping_add(4),
            );
            self.ram.write_u16(
                CUR_SPRITE_Y,
                self.ram.read_u16(CUR_SPRITE_Y).wrapping_add(4),
            );
            self.runtime.draw_single_small(self.ram, k);
        } else {
            self.runtime.draw_single_large(self.ram, k);
        }
    }
    pub fn sprite_89_mothula_beam(&mut self, k: usize) {
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_link(self.ram, k);
        if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
            self.ram
                .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() ^ 0x80);
        }
        self.runtime.move_xy(self.ram, k);
        if self.ram.sprite(k).main_delay() == 0
            && self.runtime.check_tile_collision(self.ram, k) != 0
        {
            self.ram.set_sprite_state(k, 0);
        }
        if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 3 != 0 {
            return;
        }
        for j in (0..15).rev() {
            if self.ram.read_u8(GARNISH_TYPE + j) == 0 {
                self.ram.write_u8(GARNISH_TYPE + j, 2);
                self.ram.write_u8(GARNISH_ACTIVE, 2);
                self.set_garnish_x(j, self.sprite_x(k));
                self.set_garnish_y(j, self.sprite_y(k));
                self.ram.write_u8(GARNISH_COUNTDOWN + j, 16);
                self.ram.write_u8(GARNISH_SPRITE + j, k as u8);
                self.ram
                    .write_u8(GARNISH_FLOOR + j, self.ram.sprite(k).floor());
                break;
            }
        }
    }
    pub fn sprite_94_tile(&mut self, k: usize) {
        self.ram.set_sprite_object_priority(k, 0x30);
        self.flying_tile_draw(k);
        if self.runtime.return_if_paused(self.ram, k) {
            return;
        }
        let mut shatter = self.ram.sprite(k).hit_timer() != 0;
        if !shatter {
            self.ram.set_sprite_ignore_projectile(k, 1);
            match self.ram.sprite(k).ai_state() {
                0 => {
                    let y = (self.ram.sprite(k).y_low().wrapping_add(8) as u16)
                        | (self.ram.sprite(k).y_high() as u16) << 8;
                    self.runtime
                        .update_common_tile(self.ram, self.sprite_x(k), y, 6);
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_main_delay(k, 128)
                }
                1 => {
                    let delay = self.ram.sprite(k).main_delay();
                    if delay == 0 {
                        self.ram.set_sprite_ai_state(k, 2);
                        self.ram.set_sprite_main_delay(k, 16);
                        self.runtime.apply_speed_towards_link(self.ram, k, 32)
                    } else {
                        if delay >= 0x40 {
                            self.ram
                                .set_sprite_signed_field(k, SpriteField::VelocityZ, 4);
                            self.runtime.move_z(self.ram, k)
                        }
                        self.flying_tile_animate(k)
                    }
                }
                2 => {
                    self.ram.set_sprite_ignore_projectile(k, 0);
                    let delay = self.ram.sprite(k).main_delay();
                    if delay != 0 && delay & 3 == 0 {
                        self.runtime.apply_speed_towards_link(self.ram, k, 32)
                    }
                    if !self.runtime.check_damage_to_and_from_link(self.ram, k) {
                        self.runtime.move_xy(self.ram, k);
                        self.ram.write_u16(
                            CUR_SPRITE_Y,
                            self.ram
                                .read_u16(CUR_SPRITE_Y)
                                .wrapping_sub(self.ram.sprite(k).z() as u16),
                        );
                        if self.runtime.check_tile_collision(self.ram, k) == 0 {
                            self.flying_tile_animate(k);
                            return;
                        }
                    }
                    shatter = true
                }
                _ => {}
            }
        }
        if shatter {
            self.runtime.queue_sfx2(self.ram, k, 0x1f);
            self.ram.set_sprite_state(k, 6);
            self.ram.set_sprite_main_delay(k, 31);
            self.ram.set_sprite_kind(k, 0xec);
            self.ram.set_sprite_hit_timer(k, 0);
            self.ram.set_sprite_scratch_c(k, 0x80)
        }
    }
    fn flying_tile_animate(&mut self, k: usize) {
        let s = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, s);
        self.ram.set_sprite_graphics(k, s >> 2 & 1);
        if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 7 == 0 {
            self.runtime.queue_sfx2(self.ram, k, 7)
        }
    }
    pub fn sprite_88_mothula(&mut self, k: usize) {
        if self.ram.read_u32(0x64c) & 4096 != 0 {
            self.runtime.set_enemy_damage(self.ram, 0x884, 1);
            self.runtime.set_enemy_damage(self.ram, 0x885, 1)
        }
        self.mothula_main(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.mothula_handle_spikes(k);
    }

    pub fn sprite_a4_falling_ice(&mut self, k: usize) {
        if self.ram.sprite(k).scratch_c() == 0 {
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            if self.ram.sprite(2).state() < 9
                && self.ram.sprite(3).state() < 9
                && self.ram.sprite(4).state() < 9
            {
                self.ram.set_sprite_state(k, 0);
            }
            self.generate_iceball(k);
            return;
        }
        self.ram.set_sprite_ignore_projectile(k, 1);
        self.ram.set_sprite_object_priority(k, 0x30);
        self.runtime.draw_single_large(self.ram, k);
        if self.ram.sprite(k).ai_state() == 0 {
            self.ram
                .set_sprite_flags3(k, self.ram.sprite(k).flags3() ^ 16);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let delay = self.ram.sprite(k).main_delay();
        if delay != 0 {
            if delay == 1 {
                self.ram.set_sprite_state(k, 0);
            }
            self.ram.set_sprite_graphics(k, (delay >> 3) + 2);
            return;
        }
        self.runtime.move_xy(self.ram, k);
        let ai = self.ram.sprite(k).ai_state();
        let continue_fall = if ai == 0 {
            true
        } else {
            self.runtime.check_damage_to_link(self.ram, k);
            self.runtime.check_tile_collision(self.ram, k) == 0
        };
        if continue_fall {
            let old = self.ram.sprite(k).z();
            self.runtime.move_z(self.ram, k);
            let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
            if zv >= -64 {
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityZ, zv.wrapping_sub(3));
            }
            let z = self.ram.sprite(k).z();
            if !(((old ^ z) as i8) < 0 && (z as i8) < 0) {
                return;
            }
            self.ram.set_sprite_z(k, 0);
            if ai == 0 {
                self.ram.set_sprite_state(k, 0);
                self.ice_ball_split(k);
                return;
            }
        }
        self.ram.set_sprite_main_delay(k, 15);
        self.ram.set_sprite_oam_flags(k, 4);
        if self.ram.read_u8(SOUND_EFFECT_1) == 0 {
            self.runtime.queue_sfx2(self.ram, k, 0x1e);
            self.ram.set_sprite_graphics(k, 3);
        }
    }
    pub fn ice_ball_split(&mut self, k: usize) {
        const X: [i8; 8] = [0, 32, 0, -32, 24, 24, -24, -24];
        const Y: [i8; 8] = [-32, 0, 32, 0, -24, 24, -24, 24];
        self.runtime.queue_sfx2(self.ram, k, 0x1f);
        let b = (self.runtime.random(self.ram) & 4) as usize;
        for i in (0..4).rev() {
            if let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xa4) {
                self.set_sprite_x(j, x);
                self.set_sprite_y(j, y);
                self.ram.set_sprite_ai_state(j, 1);
                self.ram.set_sprite_graphics(j, 1);
                self.ram.set_sprite_scratch_c(j, 1);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityZ, 32);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityX, X[i + b]);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::VelocityY, Y[i + b]);
                self.ram.set_sprite_flags4(j, 0x1c);
            }
        }
        self.ram.write_u8(TMP_COUNTER, 0xff);
    }
    pub fn sprite_bomb_shop_bomb(&mut self, k: usize) {
        const MAX: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
        self.bomb_shop_entity_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if !self.shop_item_check_for_a_press(k) {
            return;
        }
        if self.ram.read_u8(LINK_ITEM_BOMBS) != MAX[self.ram.read_u8(LINK_BOMB_UPGRADES) as usize] {
            if !self.shop_item_handle_cost(100) {
                self.reject_purchase(k, 0x17c)
            } else {
                self.ram.write_u8(LINK_BOMB_FILLER, 27);
                self.ram.set_sprite_state(k, 0);
                self.runtime.show_message(self.ram, 0x119);
                self.shop_item_handle_receipt(k, 0x28)
            }
        } else {
            self.reject_purchase(k, 0x16e)
        }
    }
    pub fn sprite_bomb_shop_super_bomb(&mut self, k: usize) {
        self.bomb_shop_entity_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if self.shop_item_check_for_a_press(k) {
            if !self.shop_item_handle_cost(100) {
                self.reject_purchase(k, 0x17c)
            } else {
                self.ram.write_u8(FOLLOWER_INDICATOR, 13);
                self.runtime.load_follower_graphics(self.ram);
                self.runtime.become_follower(self.ram, k);
                self.ram.set_sprite_state(k, 0);
                self.runtime.show_message(self.ram, 0x11a)
            }
        }
    }

    pub fn firesnake_spawn_fireball(&mut self, j: usize) {
        if (j ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 7 != 0 {
            return;
        }
        let Some(k) = self.runtime.garnish_alloc(self.ram) else {
            return;
        };
        self.ram.write_u8(GARNISH_TYPE + k, 1);
        self.ram.write_u8(GARNISH_ACTIVE, 1);
        self.set_garnish_x(k, self.sprite_x(j));
        self.set_garnish_y(k, self.sprite_y(j).wrapping_add(16));
        self.ram.write_u8(GARNISH_COUNTDOWN + k, 32);
        self.ram.write_u8(GARNISH_SPRITE + k, j as u8);
        self.ram
            .write_u8(GARNISH_FLOOR + k, self.ram.sprite(j).floor());
    }
    pub fn sprite_80_firesnake(&mut self, k: usize) {
        const FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];
        const X: [i8; 4] = [24, -24, 0, 0];
        const Y: [i8; 4] = [0, 0, 24, -24];
        const DIR: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.ram.set_sprite_oam_flags(
            k,
            self.ram.sprite(k).oam_flags() & 0x3f
                | FLAGS[(self.ram.read_u8(FRAME_COUNTER) >> 2 & 3) as usize],
        );
        if self.ram.sprite(k).scratch_a() != 0 {
            let d = self.ram.sprite(k).main_delay();
            self.ram.set_sprite_ignore_projectile(k, d);
            if d == 0 {
                self.ram.set_sprite_state(k, 0)
            }
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.firesnake_spawn_fireball(k);
        if self.ram.sprite(k).wall_collision() == 0 {
            self.runtime.move_xy(self.ram, k)
        }
        if self.runtime.check_tile_collision(self.ram, k) != 0 {
            let d = self.ram.sprite(k).scratch_d() as usize;
            let random_bit = (self.runtime.random(self.ram) & 1) as usize;
            self.ram.set_sprite_scratch_d(k, DIR[d * 2 + random_bit])
        }
        let d = self.ram.sprite(k).scratch_d() as usize;
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityX, X[d]);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityY, Y[d]);
    }

    pub fn sprite_93_bumper(&mut self, k: usize) {
        const V: [i8; 4] = [0, 2, -2, 0];
        self.bumper_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_tile_collision(self.ram, k);
        if self.ram.read_u8(LINK_CAPE_MODE) == 0
            && self.runtime.check_damage_same_layer(self.ram, k)
        {
            self.runtime.link_cancel_dash(self.ram);
            self.ram.set_sprite_main_delay(k, 32);
            let (y, x) = self.runtime.project_speed_towards_link(self.ram, k, 0x30);
            let joy = self.ram.read_u8(JOYPAD1_H_LAST);
            self.ram.write_i8(
                LINK_ACTUAL_VEL_Y,
                y.wrapping_add(V[(joy >> 2 & 3) as usize]),
            );
            self.ram
                .write_i8(LINK_ACTUAL_VEL_X, x.wrapping_add(V[(joy & 3) as usize]));
            self.ram.write_u8(LINK_INCAPACITATED_TIMER, 20);
            self.runtime.link_reset_swimming(self.ram);
            self.runtime.queue_sfx3(self.ram, k, 0x32)
        }
        for j in (0..16).rev() {
            if (j ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 3 != 0
                || self.ram.sprite(j).z() != 0
                || self.ram.sprite(j).state() < 9
                || (self.ram.sprite(j).flags3() | self.ram.sprite(j).flags4()) & 0x40 != 0
            {
                continue;
            }
            let x = self.sprite_x(j);
            let y = self.sprite_y(j);
            if self
                .ram
                .read_u16(CUR_SPRITE_X)
                .wrapping_sub(x)
                .wrapping_add(16)
                < 32
                && self
                    .ram
                    .read_u16(CUR_SPRITE_Y)
                    .wrapping_sub(y)
                    .wrapping_add(16)
                    < 32
            {
                self.ram.set_sprite_scratch_f(j, 15);
                let (ry, rx) = self.runtime.project_speed(self.ram, k, x, y, 0x40);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::RecoilY, ry);
                self.ram
                    .set_sprite_signed_field(j, SpriteField::RecoilX, rx);
                self.ram.set_sprite_main_delay(k, 32);
                self.runtime.queue_sfx3(self.ram, k, 0x32)
            }
        }
    }

    pub fn spike_block_check_statue_collision(&self, k: usize) -> bool {
        for j in (0..16).rev() {
            if (j ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 1 == 0
                && self.ram.sprite(j).state() != 0
                && self.ram.sprite(j).kind() == 0x1c
                && self
                    .sprite_x(k)
                    .wrapping_sub(self.sprite_x(j))
                    .wrapping_add(16)
                    < 32
                && self
                    .sprite_y(k)
                    .wrapping_sub(self.sprite_y(j))
                    .wrapping_add(16)
                    < 32
            {
                return false;
            }
        }
        true
    }
    pub fn sprite_8a_spike_block(&mut self, k: usize) {
        if self.ram.sprite(k).scratch_e() == 0 {
            self.runtime.draw_single_large(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.runtime.check_damage_to_and_from_link(self.ram, k);
            self.runtime.move_xy(self.ram, k);
            self.runtime.check_tile_collision(self.ram, k);
            if self.ram.sprite(k).main_delay() == 0
                && (!self.spike_block_check_statue_collision(k)
                    || self.ram.sprite(k).wall_collision() & 15 != 0)
            {
                self.ram.set_sprite_main_delay(k, 4);
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_neg(),
                );
                self.runtime.queue_sfx2(self.ram, k, 5)
            }
            return;
        }
        self.runtime.allocate_oam_region_b(self.ram, 4);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        let state = self.ram.sprite(k).ai_state();
        let delay = self.ram.sprite(k).main_delay();
        if state == 0 {
            self.runtime
                .update_common_tile(self.ram, self.sprite_x(k), self.sprite_y(k), 0);
            self.ram.set_sprite_ai_state(k, 1);
            self.ram.set_sprite_main_delay(k, 64);
            self.ram.set_sprite_aux1_delay(k, 105)
        } else if delay != 0 {
            if delay == 1 {
                self.ram.set_sprite_x_low(k, self.ram.sprite(k).scratch_a());
                self.ram.set_sprite_y_low(k, self.ram.sprite(k).scratch_b())
            } else {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    if delay >> 1 & 1 != 0 { -8 } else { 8 },
                );
                self.runtime.move_x(self.ram, k);
                self.ram.set_sprite_velocity_x(k, 0)
            }
        } else if state == 1 {
            const XT: [i8; 4] = [32, -32, 0, 0];
            const YT: [i8; 4] = [0, 0, 32, -32];
            const XD: [i8; 4] = [1, -1, 0, 0];
            const YD: [i8; 4] = [0, 0, 1, -1];
            let d = self.ram.sprite(k).scratch_d() as usize;
            let x = self.ram.sprite(k).signed_field(SpriteField::VelocityX);
            if x != XT[d] {
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityX, x.wrapping_add(XD[d]))
            }
            let y = self.ram.sprite(k).signed_field(SpriteField::VelocityY);
            if y != YT[d] {
                self.ram
                    .set_sprite_signed_field(k, SpriteField::VelocityY, y.wrapping_add(YD[d]))
            }
            self.runtime.move_xy(self.ram, k);
            if self.ram.sprite(k).aux1_delay() == 0 {
                self.runtime.get_16bit_coords(self.ram, k);
                if self.runtime.check_tile_collision(self.ram, k) != 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_aux1_delay(k, 64)
                }
            }
        } else if self.ram.sprite(k).aux1_delay() == 0 {
            const X: [i8; 4] = [-16, 16, 0, 0];
            const Y: [i8; 4] = [0, 0, -16, 16];
            let d = self.ram.sprite(k).scratch_d() as usize;
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, X[d]);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, Y[d]);
            self.runtime.move_xy(self.ram, k);
            if self.ram.sprite(k).x_low() == self.ram.sprite(k).scratch_a()
                && self.ram.sprite(k).y_low() == self.ram.sprite(k).scratch_b()
            {
                self.ram.set_sprite_state(k, 0);
                self.runtime
                    .update_common_tile(self.ram, self.sprite_x(k), self.sprite_y(k), 2)
            }
        }
    }

    pub fn sprite_7c_green_stalfos(&mut self, k: usize) {
        const FACE: [u8; 4] = [4, 6, 0, 2];
        const FLAGS: [u8; 4] = [0x40, 0, 0, 0];
        const GFX: [u8; 4] = [0, 0, 1, 2];
        let mut d = self.ram.sprite(k).scratch_d() as usize;
        self.ram
            .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() & !0x40 | FLAGS[d]);
        self.ram.set_sprite_graphics(k, GFX[d]);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        d = self.runtime.direction_to_face_link(self.ram, k) as usize;
        if FACE[d] != self.ram.read_u8(LINK_DIRECTION_FACING) {
            self.ram.set_sprite_scratch_a(k, 0);
            if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 7 == 0 {
                let v = self.ram.sprite(k).scratch_b();
                if v != 4 {
                    self.ram.set_sprite_scratch_b(k, v + 1)
                }
                self.runtime.apply_speed_towards_link(self.ram, k, v);
                let right = u8::from(
                    self.ram.read_u16(LINK_X).wrapping_sub(self.sprite_x(k)) & 0x8000 != 0,
                );
                self.ram.set_sprite_scratch_d(k, right)
            }
        } else {
            self.ram.set_sprite_scratch_a(k, 1);
            if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 15 == 0 {
                let v = self.ram.sprite(k).scratch_b();
                if v != 0 {
                    self.ram.set_sprite_scratch_b(k, v - 1)
                }
                self.runtime.apply_speed_towards_link(self.ram, k, v);
                let right = u8::from(
                    self.ram.read_u16(LINK_X).wrapping_sub(self.sprite_x(k)) & 0x8000 != 0,
                );
                self.ram.set_sprite_scratch_d(k, right)
            }
        }
        self.runtime.move_xy(self.ram, k);
    }

    pub fn kodongo_spawn_fire(&mut self, k: usize) {
        const X: [i8; 4] = [8, -8, 0, 0];
        const Y: [i8; 4] = [0, 0, 8, -8];
        const XV: [i8; 4] = [24, -24, 0, 0];
        const YV: [i8; 4] = [0, 0, 24, -24];
        if let Some((j, x, y, _)) = self.runtime.spawn_dynamic_ex(self.ram, k, 0x87, 13) {
            let d = self.ram.sprite(k).scratch_d() as usize;
            self.set_sprite_x(j, x.wrapping_add_signed(X[d] as i16));
            self.set_sprite_y(j, y.wrapping_add_signed(Y[d] as i16));
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityX, XV[d]);
            self.ram
                .set_sprite_signed_field(j, SpriteField::VelocityY, YV[d]);
            self.ram.set_sprite_ignore_projectile(j, 1)
        }
    }

    pub fn sprite_87_kodongo_fire(&mut self, k: usize) {
        const FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        const G: [u8; 32] = [
            5, 4, 3, 1, 2, 0, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1,
            2, 3, 0,
        ];
        let mut delay = self.ram.sprite(k).main_delay();
        if delay == 0 {
            self.runtime.draw_single_large(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.ram.set_sprite_oam_flags(
                k,
                self.ram.sprite(k).oam_flags() & 0x3f
                    | FLAGS[(self.ram.read_u8(FRAME_COUNTER) >> 2 & 3) as usize],
            );
            if !self.runtime.check_damage_to_link(self.ram, k) {
                self.runtime.move_xy(self.ram, k);
                if self.runtime.check_tile_collision(self.ram, k) == 0 {
                    return;
                }
            }
            self.ram.set_sprite_main_delay(k, 127);
            self.ram
                .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() & 63);
            self.runtime.queue_sfx2(self.ram, k, 0x2a)
        } else {
            if self.runtime.check_damage_from_link(self.ram, k) & 1 != 0 {
                delay = delay.wrapping_sub(1);
                self.ram.set_sprite_main_delay(k, delay);
                if delay == 0 {
                    self.ram.set_sprite_state(k, 0)
                }
            }
            if self.ram.sprite(k).main_delay() == 1 {
                self.ram.set_sprite_state(k, 0)
            }
            self.ram
                .set_sprite_graphics(k, G[(self.ram.sprite(k).main_delay() >> 3) as usize]);
            self.flame_draw(k);
            self.runtime.check_damage_to_link(self.ram, k);
        }
    }

    pub fn sprite_a7_stalfos(&mut self, k: usize) {
        if self.ram.sprite(k).scratch_a() != 0 {
            self.sprite_stalfos_bone(k);
            return;
        }
        if self.ram.sprite(k).scratch_e() == 0 {
            self.stalfos_skellington(k);
            return;
        }
        if self.ram.sprite(k).main_delay() == 0 {
            self.ram.set_sprite_velocity_x(k, 1);
            self.ram.set_sprite_velocity_y(k, 1);
            if self.runtime.check_tile_collision(self.ram, k) != 0 {
                self.ram.set_sprite_state(k, 0);
                return;
            }
            self.ram.set_sprite_scratch_e(k, 0);
            self.runtime.queue_sfx2(self.ram, k, 0x15);
            self.runtime.spawn_poof_garnish(self.ram, k);
            self.ram.set_sprite_aux2_delay(k, 8);
            self.ram.set_sprite_main_delay(k, 64);
            self.ram.set_sprite_velocity_y(k, 0);
            self.ram.set_sprite_velocity_x(k, 0)
        }
        self.runtime.prep_oam(self.ram, k);
    }

    pub fn kyameron_draw(&mut self, k: usize) {
        const FLAGS: [u8; 12] = [0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0xc0, 0x80];
        const D: [DrawPiece; 28] = [
            (1, 8, 0x00b4, 0),
            (7, 8, 0x00b5, 0),
            (4, -3, 0x0086, 0),
            (0, -13, 0x80a2, 2),
            (2, 8, 0x00b4, 0),
            (6, 8, 0x00b5, 0),
            (4, -6, 0x0096, 0),
            (0, -20, 0x00a2, 2),
            (4, -1, 0x0096, 0),
            (0, -27, 0x00a2, 2),
            (0, -27, 0x00a2, 2),
            (0, -27, 0x00a2, 2),
            (-6, -6, 0x01df, 0),
            (14, -6, 0x41df, 0),
            (-6, 14, 0x81df, 0),
            (14, 14, 0xc1df, 0),
            (-6, -6, 0x0096, 0),
            (14, -6, 0x4096, 0),
            (-6, 14, 0x8096, 0),
            (14, 14, 0xc096, 0),
            (-4, -4, 0x018d, 0),
            (12, -4, 0x418d, 0),
            (-4, 12, 0x818d, 0),
            (12, 12, 0xc18d, 0),
            (0, 0, 0x018d, 0),
            (8, 0, 0x418d, 0),
            (0, 8, 0x818d, 0),
            (8, 8, 0xc18d, 0),
        ];
        let j = self.ram.sprite(k).graphics() as usize;
        if j < 12 {
            let old = self.ram.sprite(k).oam_flags();
            self.ram.set_sprite_oam_flags(k, old & 0x3f | FLAGS[j]);
            self.runtime.draw_single_large(self.ram, k);
            self.ram.set_sprite_oam_flags(k, old)
        } else {
            let i = (j - 12) * 4;
            self.runtime.draw_multiple(self.ram, k, &D[i..i + 4])
        }
    }

    pub fn sprite_9a_kyameron(&mut self, k: usize) {
        if self.ram.sprite(k).ai_state() == 0 {
            self.runtime.prep_oam(self.ram, k)
        } else {
            self.kyameron_draw(k)
        }
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.ram.set_sprite_ignore_projectile(k, 1);
        let state = self.ram.sprite(k).ai_state();
        let delay = self.ram.sprite(k).main_delay();
        match state {
            0 => {
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    let delay = (self.runtime.random(self.ram) & 63) + 96;
                    self.ram.set_sprite_main_delay(k, delay);
                    self.ram.set_sprite_x_low(k, self.ram.sprite(k).scratch_a());
                    self.ram
                        .set_sprite_x_high(k, self.ram.sprite(k).scratch_b());
                    self.ram.set_sprite_y_low(k, self.ram.sprite(k).scratch_c());
                    self.ram
                        .set_sprite_y_high(k, self.ram.sprite(k).head_direction());
                    self.ram.set_sprite_subtype2(k, 5);
                    self.ram.set_sprite_graphics(k, 8)
                }
            }
            1 => {
                if delay == 0 {
                    self.ram.set_sprite_main_delay(k, 31);
                    self.ram.set_sprite_ai_state(k, 2)
                }
                let s = self.ram.sprite(k).subtype2().wrapping_sub(1);
                self.ram.set_sprite_subtype2(k, s);
                if (s as i8) < 0 {
                    self.ram.set_sprite_subtype2(k, 5);
                    self.ram.set_sprite_graphics(
                        k,
                        self.ram.sprite(k).graphics().wrapping_add(1) & 3 | 8,
                    )
                }
            }
            2 => {
                const G: [i8; 8] = [4, 7, 14, 13, 12, 6, 6, 5];
                const X: [i8; 4] = [32, -32, 32, -32];
                const Y: [i8; 4] = [32, 32, -32, -32];
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    let j = (self.runtime.is_below_link(self.ram, k) * 2
                        + self.runtime.is_right_of_link(self.ram, k))
                        as usize;
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityX, X[j]);
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityY, Y[j])
                } else {
                    if delay == 7 {
                        self.set_sprite_y(k, self.sprite_y(k).wrapping_sub(29))
                    }
                    self.ram.set_sprite_signed_field(
                        k,
                        SpriteField::Graphics,
                        G[(delay >> 2) as usize],
                    )
                }
            }
            3 => {
                const G: [u8; 4] = [3, 2, 1, 0];
                self.ram.set_sprite_ignore_projectile(k, 0);
                let mut disperse = self.runtime.check_damage_to_and_from_link(self.ram, k);
                if !disperse {
                    self.runtime.move_xy(self.ram, k);
                    let c = self.runtime.check_tile_collision(self.ram, k);
                    if c & 3 != 0 {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityX,
                            self.ram
                                .sprite(k)
                                .signed_field(SpriteField::VelocityX)
                                .wrapping_neg(),
                        );
                        self.ram.set_sprite_animation_clock(
                            k,
                            self.ram.sprite(k).animation_clock().wrapping_add(1),
                        )
                    }
                    if c & 12 != 0 {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityY,
                            self.ram
                                .sprite(k)
                                .signed_field(SpriteField::VelocityY)
                                .wrapping_neg(),
                        );
                        self.ram.set_sprite_animation_clock(
                            k,
                            self.ram.sprite(k).animation_clock().wrapping_add(1),
                        )
                    }
                    disperse = self.ram.sprite(k).animation_clock() >= 3
                }
                if disperse {
                    self.ram.set_sprite_ai_state(k, 4);
                    self.ram.set_sprite_main_delay(k, 15);
                    self.runtime.queue_sfx2(self.ram, k, 0x28)
                }
                let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(k, s);
                self.ram.set_sprite_graphics(k, G[(s >> 3 & 3) as usize]);
                if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 7 == 0 {
                    let x = (self.runtime.random(self.ram) & 15).wrapping_sub(4) as i8;
                    let y = (self.runtime.random(self.ram) & 15).wrapping_sub(4) as i8;
                    self.runtime.spawn_sparkle(self.ram, k, x, y)
                }
            }
            4 => {
                if delay == 0 {
                    self.ram.set_sprite_animation_clock(k, 0);
                    self.ram.set_sprite_ai_state(k, 0);
                    self.ram.set_sprite_z(k, 0);
                    self.ram.set_sprite_main_delay(k, 64)
                } else {
                    self.ram.set_sprite_graphics(k, (delay >> 2) + 15)
                }
            }
            _ => {}
        }
    }

    pub fn sprite_99_pengator(&mut self, k: usize) {
        const BASE: [u8; 4] = [5, 0, 10, 15];
        let d = self.ram.sprite(k).scratch_d() as usize;
        self.ram
            .set_sprite_graphics(k, self.ram.sprite(k).scratch_a() + BASE[d]);
        self.pengator_draw(k);
        if self.ram.sprite(k).scratch_f() != 0 || self.ram.sprite(k).wall_collision() & 15 != 0 {
            self.ram.set_sprite_ai_state(k, 0);
            self.ram.set_sprite_velocity_x(k, 0);
            self.ram.set_sprite_velocity_y(k, 0)
        }
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.runtime.move_xyz(self.ram, k);
        self.ram.set_sprite_signed_field(
            k,
            SpriteField::VelocityZ,
            self.ram
                .sprite(k)
                .signed_field(SpriteField::VelocityZ)
                .wrapping_sub(2),
        );
        if self.ram.sprite(k).signed_field(SpriteField::Z) < 0 {
            self.ram.set_sprite_velocity_z(k, 0);
            self.ram.set_sprite_z(k, 0)
        }
        self.runtime.check_tile_collision(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                let d = self.runtime.direction_to_face_link(self.ram, k);
                self.ram.set_sprite_scratch_d(k, d);
                self.ram.set_sprite_ai_state(k, 1)
            }
            1 => {
                if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 3 == 0 {
                    const V: [i8; 6] = [1, -1, 0, 0, 1, -1];
                    const XT: [i8; 4] = [16, -16, 0, 0];
                    const YT: [i8; 4] = [0, 0, 16, -16];
                    let d = self.ram.sprite(k).scratch_d() as usize;
                    let mut changed = false;
                    let x = self.ram.sprite(k).signed_field(SpriteField::VelocityX);
                    if x != XT[d] {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityX,
                            x.wrapping_add(V[d]),
                        );
                        changed = true
                    }
                    let y = self.ram.sprite(k).signed_field(SpriteField::VelocityY);
                    if y != YT[d] {
                        self.ram.set_sprite_signed_field(
                            k,
                            SpriteField::VelocityY,
                            y.wrapping_add(V[d + 2]),
                        );
                        changed = true
                    }
                    if !changed {
                        self.ram.set_sprite_main_delay(k, 15);
                        self.ram.set_sprite_ai_state(k, 2)
                    }
                }
                self.ram
                    .set_sprite_scratch_a(k, (self.ram.read_u8(FRAME_COUNTER) & 4) >> 2)
            }
            2 => {
                const J: [u8; 4] = [4, 4, 3, 2];
                let delay = self.ram.sprite(k).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 3)
                } else if delay == 5 {
                    self.ram
                        .set_sprite_signed_field(k, SpriteField::VelocityZ, 24)
                }
                self.ram.set_sprite_scratch_a(k, J[(delay >> 2) as usize])
            }
            3 => {
                if (k ^ self.ram.read_u8(FRAME_COUNTER) as usize) & 7 == 0
                    && self.ram.sprite(k).z() == 0
                {
                    const GX: [i8; 8] = [4, 4, 4, 4, 0, 4, 8, 12];
                    const GY: [i8; 8] = [8, 10, 12, 14, 12, 12, 12, 12];
                    let d = self.ram.sprite(k).scratch_d();
                    let ix = (self.runtime.random(self.ram) & 3) as usize + usize::from(d >= 2) * 4;
                    let iy = (self.runtime.random(self.ram) & 3) as usize + usize::from(d >= 2) * 4;
                    self.runtime
                        .spawn_sparkle_limited(self.ram, k, GX[ix], GY[iy])
                }
            }
            _ => {}
        }
    }

    pub fn sprite_ab_crystal_maiden(&mut self, k: usize) {
        self.ram.write_u16(
            CUR_SPRITE_X,
            self.ram
                .read_u16(CUR_SPRITE_X)
                .wrapping_sub(self.ram.read_u16(DUNG_FLOOR_X_OFFS)),
        );
        self.ram.write_u16(
            CUR_SPRITE_Y,
            self.ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_sub(self.ram.read_u16(DUNG_FLOOR_Y_OFFS)),
        );
        if self.ram.sprite(k).ai_state() >= 3 {
            self.runtime.crystal_maiden_draw(self.ram, k)
        }
        self.ram.write_u8(IS_NMI_THREAD_ACTIVE, 1);
        if self.ram.read_u8(INTRO_DID_RUN_STEP) == 0 {
            self.crystal_maiden_run_cutscene(k);
            self.ram.write_u8(INTRO_DID_RUN_STEP, 1)
        }
    }

    pub fn sprite_ba_whirlpool(&mut self, k: usize) {
        const FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        let screen = self.ram.read_u8(OVERWORLD_SCREEN_INDEX);
        if screen == 0x1b {
            self.runtime.prep_oam(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            let x = self
                .ram
                .read_u16(CUR_SPRITE_X)
                .wrapping_sub(self.ram.read_u16(LINK_X))
                .wrapping_add(0x40);
            let y = self
                .ram
                .read_u16(CUR_SPRITE_Y)
                .wrapping_sub(self.ram.read_u16(LINK_Y))
                .wrapping_add(0xf);
            if x < 0x51 && y < 0x12 {
                self.ram.write_u8(SUBMODULE_INDEX, 35);
                self.ram.write_u8(LINK_TRIGGERED_BY_WHIRLPOOL, 1);
                self.ram.write_u8(SUBSUBMODULE_INDEX, 0);
                self.ram.write_u8(LINK_ACTUAL_VEL_Y, 0);
                self.ram.write_u8(LINK_ACTUAL_VEL_X, 0);
                self.ram.write_u8(LINK_PLAYER_HANDLER_STATE, 20);
                self.ram.write_u8(
                    LAST_LIGHT_VS_DARK_WORLD,
                    self.ram.read_u8(OVERWORLD_SCREEN_INDEX) & 0x40,
                )
            }
        } else {
            self.ram.set_sprite_oam_flags(
                k,
                self.ram.sprite(k).oam_flags() & 0x3f
                    | FLAGS[(self.ram.read_u8(FRAME_COUNTER) >> 3 & 3) as usize],
            );
            self.runtime.allocate_oam_region_b(self.ram, 4);
            self.ram.write_u16(
                CUR_SPRITE_X,
                self.ram.read_u16(CUR_SPRITE_X).wrapping_sub(5),
            );
            self.runtime.draw_single_large(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            if self.runtime.check_damage_same_layer(self.ram, k) {
                if self.ram.sprite(k).scratch_a() == 0 {
                    self.ram.write_u8(SUBMODULE_INDEX, 46);
                    self.ram.write_u8(SUBSUBMODULE_INDEX, 0)
                }
            } else {
                self.ram.set_sprite_scratch_a(k, 0)
            }
        }
    }

    pub fn sprite_ac_apple(&mut self, k: usize) {
        if self.ram.sprite(k).ai_state() != 0 {
            self.sprite_apple(k);
        } else if self.ram.sprite(k).scratch_e() == 0 {
            self.ram.set_sprite_state(k, 0);
            // The dynamic spawn itself is owned by the shared sprite allocator.
            // C executes `(random & 3) + 2`, then a do/while, hence 3..=6 calls.
            let mut n = (self.runtime.random(self.ram) & 3) + 3;
            while n != 0 {
                self.spawn_apple(k);
                n -= 1;
            }
        }
    }

    pub fn spawn_apple(&mut self, k: usize) {
        let Some((j, x, y, _)) = self.runtime.spawn_dynamic(self.ram, k, 0xac) else {
            return;
        };
        self.set_sprite_x(j, x);
        self.set_sprite_y(j, y);
        self.ram.set_sprite_ai_state(j, 1);
        self.ram.set_sprite_scratch_a(j, 255);
        self.ram.set_sprite_z(j, 8);
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityZ, 22);
        let target_x = (x & !0xff) | self.runtime.random(self.ram) as u16;
        let target_y = (y & !0xff) | self.runtime.random(self.ram) as u16;
        let (yv, xv) = self
            .runtime
            .project_speed(self.ram, k, target_x, target_y, 10);
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityX, xv);
        self.ram
            .set_sprite_signed_field(j, SpriteField::VelocityY, yv);
    }

    pub fn sprite_apple(&mut self, k: usize) {
        let a = self.ram.sprite(k).scratch_a();
        if a >= 16 || self.ram.read_u8(FRAME_COUNTER) & 2 != 0 {
            self.runtime.draw_single_large(self.ram, k);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if a == 0 {
            self.ram.set_sprite_state(k, 0);
            return;
        }
        self.runtime.move_xyz(self.ram, k);
        if self.runtime.check_damage_to_link(self.ram, k) {
            self.runtime.queue_sfx3(self.ram, k, 0x0b);
            self.ram.write_u8(
                LINK_HEARTS_FILLER,
                self.ram.read_u8(LINK_HEARTS_FILLER).wrapping_add(8),
            );
            self.ram.set_sprite_state(k, 0);
            return;
        }
        if self.ram.read_u8(FRAME_COUNTER) & 1 == 0 {
            self.ram.set_sprite_scratch_a(k, a.wrapping_sub(1));
        }
        if self.ram.sprite(k).z().wrapping_sub(1) as i8 >= 0 {
            self.ram
                .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(1));
            return;
        }
        self.ram.set_sprite_z(k, 0);
        let zv = self.ram.sprite(k).signed_field(SpriteField::VelocityZ);
        let rebound = if zv < 0 {
            (zv.wrapping_neg() as u8) >> 1
        } else {
            0
        };
        self.ram.set_sprite_velocity_z(k, rebound);
        for base in [SPRITE_X_VEL, SPRITE_Y_VEL] {
            let v = self.ram.read_i8(base + k);
            if v != 0 {
                self.ram
                    .write_i8(base + k, v.wrapping_add(if v < 0 { 1 } else { -1 }));
            }
        }
    }

    pub fn shop_item_check_for_a_press(&mut self, k: usize) -> bool {
        self.ram.read_u8(FILTERED_JOYPAD_L) & 0x80 != 0
            && self.runtime.check_damage_same_layer(self.ram, k)
    }

    pub fn shop_item_play_beep(&mut self, k: usize) {
        self.runtime.queue_sfx2(self.ram, k, 0x3c);
    }

    pub fn shop_item_handle_receipt(&mut self, k: usize, item: u8) {
        const MESSAGES: [u16; 7] = [0x168, 0x167, 0x167, 0x16c, 0x169, 0x16a, 0x16b];
        self.runtime.link_receive_item(self.ram, item);
        let j = self.ram.sprite(k).subtype2();
        if j >= 7 {
            self.runtime
                .show_message(self.ram, MESSAGES[(j - 7) as usize]);
            self.runtime.rapid_terminate_receive_item(self.ram);
        }
    }

    pub fn sprite_bb_shopkeeper(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.shopkeeper_standard_clerk(k),
            1 => self.chest_game_guy(k),
            2 => self.nice_thief_with_gift(k),
            3 => self.mini_chest_game_guy(k),
            4 => self.lost_woods_chest_game_guy(k),
            5 | 6 => self.nice_thief_under_rock(k),
            7 => self.shop_item_red_potion150(k),
            8 => self.shop_item_fighter_shield(k),
            9 => self.shop_item_fire_shield(k),
            10 => self.shop_item_heart(k),
            11 => self.shop_item_arrows(k),
            12 => self.shop_item_bombs(k),
            13 => self.shop_item_bee(k),
            _ => {}
        }
    }

    pub fn shop_item_make_shields_deflect(&mut self, k: usize) {
        self.ram.set_sprite_ignore_projectile(k, 0);
        self.ram.set_sprite_flags(k, 8);
        self.ram.set_sprite_deflection_bits(k, 4);
        self.ram.set_sprite_flags4(k, 0x1c);
        self.runtime.check_damage_from_link(self.ram, k);
        self.ram.set_sprite_flags4(k, 0x0a);
    }

    pub fn sprite_draw_shop_item(&mut self, k: usize) {
        const D: [DrawPiece; 35] = [
            (-4, 16, 0x0231, 0),
            (4, 16, 0x0213, 0),
            (12, 16, 0x0230, 0),
            (0, 0, 0x02c0, 2),
            (0, 11, 0x036c, 2),
            (0, 16, 0x0213, 0),
            (0, 16, 0x0213, 0),
            (8, 16, 0x0230, 0),
            (0, 0, 0x04ce, 2),
            (4, 12, 0x0338, 0),
            (-4, 16, 0x0213, 0),
            (4, 16, 0x0230, 0),
            (12, 16, 0x0230, 0),
            (0, 0, 0x08cc, 2),
            (4, 12, 0x0338, 0),
            (0, 16, 0x0231, 0),
            (0, 16, 0x0231, 0),
            (8, 16, 0x0230, 0),
            (4, 8, 0x0329, 0),
            (4, 11, 0x0338, 0),
            (-4, 16, 0x0203, 0),
            (-4, 16, 0x0203, 0),
            (4, 16, 0x0230, 0),
            (0, 0, 0x04c4, 2),
            (0, 11, 0x0338, 0),
            (0, 16, 0x0213, 0),
            (0, 16, 0x0213, 0),
            (8, 16, 0x0230, 0),
            (0, 0, 0x04e8, 2),
            (0, 11, 0x036c, 2),
            (0, 16, 0x0231, 0),
            (0, 16, 0x0231, 0),
            (8, 16, 0x0230, 0),
            (4, 8, 0x0ff4, 0),
            (4, 11, 0x0338, 0),
        ];
        let i = self.ram.sprite(k).subtype2().wrapping_sub(7) as usize * 5;
        self.runtime.allocate_oam_defer_to_player(self.ram, k);
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 5]);
    }

    fn reject_purchase(&mut self, k: usize, message: u16) {
        self.runtime.show_message(self.ram, message);
        self.shop_item_play_beep(k);
    }

    pub fn shop_item_fighter_shield(&mut self, k: usize) {
        self.sprite_draw_shop_item(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.shop_item_make_shields_deflect(k);
        if self.shop_item_check_for_a_press(k) {
            if self.ram.read_u8(LINK_SHIELD_TYPE) != 0 {
                self.reject_purchase(k, 0x166);
                return;
            }
            if !self.shop_item_handle_cost(50) {
                self.reject_purchase(k, 0x17c);
                return;
            }
            self.ram.set_sprite_state(k, 0);
            self.shop_item_handle_receipt(k, 4);
        }
        self.ram.set_sprite_flags4(k, 0x1c);
    }

    pub fn shop_item_fire_shield(&mut self, k: usize) {
        self.sprite_draw_shop_item(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.shop_item_make_shields_deflect(k);
        if self.shop_item_check_for_a_press(k) {
            if self.ram.read_u8(LINK_SHIELD_TYPE) >= 2 {
                self.reject_purchase(k, 0x166);
                return;
            }
            if !self.shop_item_handle_cost(500) {
                self.reject_purchase(k, 0x17c);
                return;
            }
            self.ram.set_sprite_state(k, 0);
            self.shop_item_handle_receipt(k, 5);
        }
        self.ram.set_sprite_flags4(k, 0x1c);
    }

    pub fn shop_item_heart(&mut self, k: usize) {
        self.sprite_draw_shop_item(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if self.shop_item_check_for_a_press(k) {
            if self.ram.read_u8(LINK_HEALTH_CURRENT) == self.ram.read_u8(LINK_HEALTH_CAPACITY) {
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(10) {
                self.ram.set_sprite_state(k, 0);
                self.shop_item_handle_receipt(k, 0x42);
            } else {
                self.reject_purchase(k, 0x17c);
            }
        }
    }

    pub fn shop_item_arrows(&mut self, k: usize) {
        const MAX: [u8; 8] = [30, 35, 40, 45, 50, 55, 60, 70];
        self.sprite_draw_shop_item(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if self.shop_item_check_for_a_press(k) {
            if self.ram.read_u8(LINK_NUM_ARROWS)
                == MAX[self.ram.read_u8(LINK_ARROW_UPGRADES) as usize]
            {
                self.runtime.show_solicited_message(self.ram, k, 0x16e);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(30) {
                self.ram.set_sprite_state(k, 0);
                self.shop_item_handle_receipt(k, 0x44);
            } else {
                self.reject_purchase(k, 0x17c);
            }
        }
    }

    pub fn shop_item_bombs(&mut self, k: usize) {
        const MAX: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
        self.sprite_draw_shop_item(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if self.shop_item_check_for_a_press(k) {
            if self.ram.read_u8(LINK_ITEM_BOMBS)
                == MAX[self.ram.read_u8(LINK_BOMB_UPGRADES) as usize]
            {
                self.runtime.show_solicited_message(self.ram, k, 0x16e);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(50) {
                self.ram.set_sprite_state(k, 0);
                self.shop_item_handle_receipt(k, 0x31);
            } else {
                self.reject_purchase(k, 0x17c);
            }
        }
    }

    pub fn shop_item_bee(&mut self, k: usize) {
        self.sprite_draw_shop_item(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if self.shop_item_check_for_a_press(k) {
            if self.sprite_find_empty_bottle() < 0 {
                self.runtime.show_solicited_message(self.ram, k, 0x16d);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(10) {
                self.ram.set_sprite_state(k, 0);
                self.shop_item_handle_receipt(k, 0x0e);
            } else {
                self.reject_purchase(k, 0x17c);
            }
        }
    }

    pub fn somaria_platform_locate_path(&mut self, k: usize) {
        loop {
            let tile = self.somaria_platform_and_pipe_check_tile(k);
            self.ram.set_sprite_scratch_e(k, tile);
            if (0xb0..0xbf).contains(&tile) {
                break;
            }
            self.set_sprite_x(k, self.sprite_x(k).wrapping_add(8));
            self.set_sprite_y(k, self.sprite_y(k).wrapping_add(8));
        }
        self.ram
            .set_sprite_x_low(k, (self.ram.sprite(k).x_low() & !7).wrapping_add(4));
        self.ram
            .set_sprite_y_low(k, (self.ram.sprite(k).y_low() & !7).wrapping_add(4));
        self.ram
            .set_sprite_head_direction(k, self.ram.sprite(k).scratch_d());
        self.somaria_platform_and_pipe_handle_movement(k);
        self.ram.set_sprite_ignore_projectile(
            k,
            self.ram.sprite(k).ignore_projectile().wrapping_add(1),
        );
        self.ram.write_u8(PLAYER_ON_SOMARIA_PLATFORM, 0);
        self.ram.set_sprite_aux4_delay(k, 14);
        self.ram
            .set_sprite_graphics(k, self.ram.sprite(k).graphics().wrapping_add(1));
    }

    pub fn sprite_ed_somaria_platform(&mut self, k: usize) {
        match self.ram.sprite(k).graphics() {
            0 => {
                self.somaria_platform_locate_path(k);
                if let Some(j) = self.runtime.spawn_superficial_bomb_blast(self.ram, k) {
                    self.set_sprite_x(j, self.sprite_x(j).wrapping_sub(8));
                    self.set_sprite_y(j, self.sprite_y(j).wrapping_sub(8));
                }
            }
            1 => {
                const DX: [i8; 8] = [0, 0, -1, 1, -1, 1, 1, -1];
                const DY: [i8; 8] = [-1, 1, 0, 0, -1, 1, -1, 1];
                self.somaria_platform_draw(k);
                if self.runtime.return_if_inactive(self.ram, k) {
                    return;
                }
                let no_drag =
                    self.ram.read_u16(DRAG_PLAYER_X) == 0 && self.ram.read_u16(DRAG_PLAYER_Y) == 0;
                if no_drag
                    && (self.ram.read_u8(PLAYER_NEAR_PIT_STATE).wrapping_sub(2) as i8) < 0
                    && self.runtime.check_damage_ignore_layer(self.ram, k)
                {
                    self.ram.set_sprite_scratch_c(k, 1);
                    self.runtime.link_cancel_dash(self.ram);
                    let state = self.ram.read_u8(LINK_PLAYER_HANDLER_STATE);
                    if state != 19 && state != 3 {
                        if self.ram.sprite(k).ai_state() != 0 {
                            self.somaria_platform_and_pipe_handle_movement(k);
                            return;
                        }
                        self.ram.set_sprite_scratch_a(
                            k,
                            self.ram.sprite(k).scratch_a().wrapping_add(1),
                        );
                        self.ram.write_u8(PLAYER_ON_SOMARIA_PLATFORM, 2);
                        if self.ram.sprite(k).scratch_a() & 7 == 0 {
                            let tile = self.somaria_platform_and_pipe_check_tile(k);
                            if tile != self.ram.sprite(k).scratch_e() {
                                self.ram.set_sprite_scratch_e(k, tile);
                                self.ram
                                    .set_sprite_head_direction(k, self.ram.sprite(k).scratch_d());
                                self.somaria_platform_and_pipe_handle_movement(k);
                                self.somaria_platform_handle_drag(k);
                            }
                        }
                        if self.ram.read_u8(DUNGEON_ROOM_INDEX) != 36 {
                            let d = self.ram.sprite(k).scratch_d() as usize;
                            self.ram.write_u16(
                                DRAG_PLAYER_X,
                                self.ram
                                    .read_u16(DRAG_PLAYER_X)
                                    .wrapping_add_signed(DX[d] as i16),
                            );
                            self.ram.write_u16(
                                DRAG_PLAYER_Y,
                                self.ram
                                    .read_u16(DRAG_PLAYER_Y)
                                    .wrapping_add_signed(DY[d] as i16),
                            );
                            self.runtime.move_xy(self.ram, k);
                            self.somaria_platform_drag_link(k);
                        } else {
                            self.ram.write_u8(PLAYER_ON_SOMARIA_PLATFORM, 1);
                        }
                        return;
                    }
                }
                if self.ram.sprite(k).scratch_c() != 0 {
                    self.ram.write_u8(PLAYER_ON_SOMARIA_PLATFORM, 0);
                    self.ram.set_sprite_scratch_c(k, 0);
                }
            }
            _ => {}
        }
    }

    pub fn somaria_platform_and_pipe_handle_movement(&mut self, k: usize) {
        const XV: [i8; 8] = [0, 0, -16, 16, -16, 16, 16, -16];
        const YV: [i8; 8] = [-16, 16, 0, 0, -16, 16, -16, 16];
        self.somaria_platform_handle_junctions(k);
        let d = self.ram.sprite(k).scratch_d() as usize;
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityX, XV[d]);
        self.ram
            .set_sprite_signed_field(k, SpriteField::VelocityY, YV[d]);
    }

    pub fn somaria_platform_and_pipe_check_tile(&mut self, k: usize) -> u8 {
        let mut x = self.sprite_x(k);
        let y = self.sprite_y(k);
        self.runtime.get_tile_attribute(self.ram, &mut x, y)
    }

    pub fn somaria_platform_draw(&mut self, k: usize) {
        const D: [DrawPiece; 16] = [
            (-16, -16, 0x00ac, 2),
            (0, -16, 0x40ac, 2),
            (-16, 0, 0x80ac, 2),
            (0, 0, 0xc0ac, 2),
            (-13, -13, 0x00ac, 2),
            (-3, -13, 0x40ac, 2),
            (-13, -3, 0x80ac, 2),
            (-3, -3, 0xc0ac, 2),
            (-10, -10, 0x00ac, 2),
            (-6, -10, 0x40ac, 2),
            (-10, -6, 0x80ac, 2),
            (-6, -6, 0xc0ac, 2),
            (-8, -8, 0x00ac, 2),
            (-8, -8, 0x40ac, 2),
            (-8, -8, 0x80ac, 2),
            (-8, -8, 0xc0ac, 2),
        ];
        self.runtime.allocate_oam_region_b(self.ram, 0x10);
        let i = (self.ram.sprite(k).aux4_delay() & 12) as usize;
        self.runtime.draw_multiple(self.ram, k, &D[i..i + 4]);
    }

    pub fn somaria_platform_handle_junctions(&mut self, k: usize) {
        let e = self.ram.sprite(k).scratch_e();
        let mut d = self.ram.sprite(k).scratch_d();
        let keys = self.ram.read_u8(JOYPAD1_H_LAST);
        let mut ai = 0;
        match e {
            0xb2 | 0xb5 => d ^= 3,
            0xb3 | 0xb4 => d ^= 2,
            0xb6 => {
                const MASK: [u8; 4] = [4, 8, 1, 2];
                ai = 1;
                if keys & MASK[d as usize] != 0 {
                    ai = 0;
                    d ^= 1;
                }
                self.ram.write_u8(LINK_VISIBILITY_STATUS, 0);
                self.ram.write_u8(PLAYER_ON_SOMARIA_PLATFORM, 1);
            }
            0xb7..=0xbb => {
                const MASKS: [[u8; 4]; 5] = [
                    [3, 7, 6, 5],
                    [11, 3, 10, 9],
                    [9, 5, 12, 13],
                    [10, 6, 14, 12],
                    [11, 7, 14, 13],
                ];
                let t = keys & MASKS[(e - 0xb7) as usize][d as usize];
                if t & 8 != 0 {
                    d = 0
                } else if t & 4 != 0 {
                    d = 1
                } else if t & 2 != 0 {
                    d = 2
                } else if t & 1 != 0 {
                    d = 3
                } else if e == 0xb7 && d == 0 {
                    d = 2
                } else if e == 0xb8 && d == 1 {
                    d = 2
                } else if e == 0xb9 && d == 2 {
                    d = 0
                } else if e == 0xba && d == 3 {
                    d = 0
                }
            }
            0xbc => {
                const MASK: [u8; 4] = [12, 12, 3, 3];
                ai = 1;
                let t = keys & MASK[d as usize];
                if t != 0 {
                    d = if t & 8 != 0 {
                        0
                    } else if t & 4 != 0 {
                        1
                    } else if t & 2 != 0 {
                        2
                    } else {
                        3
                    };
                    ai = 0;
                }
                self.ram.write_u8(PLAYER_ON_SOMARIA_PLATFORM, 1);
            }
            0xbe => {
                d ^= 1;
                self.ram.write_u8(LINK_VISIBILITY_STATUS, 0);
                self.ram.write_u8(PLAYER_ON_SOMARIA_PLATFORM, 1);
            }
            _ => return,
        }
        self.ram.set_sprite_scratch_d(k, d);
        self.ram.set_sprite_ai_state(k, ai);
    }

    pub fn somaria_platform_handle_drag_x(&mut self, k: usize) {
        if (self.ram.sprite(k).scratch_d() ^ self.ram.sprite(k).head_direction()) & 2 != 0 {
            let old = self.ram.sprite(k).x_low();
            let x = (old & !7).wrapping_add(4);
            let t = x.wrapping_sub(old);
            if t != 0 {
                self.ram.write_u16(DRAG_PLAYER_X, (t as i8 as i16) as u16);
                self.ram.set_sprite_x_low(k, x);
            }
        }
    }
    pub fn somaria_platform_handle_drag_y(&mut self, k: usize) {
        if (self.ram.sprite(k).scratch_d() ^ self.ram.sprite(k).head_direction()) & 2 != 0 {
            let old = self.ram.sprite(k).y_low();
            let y = (old & !7).wrapping_add(4);
            let t = y.wrapping_sub(old);
            if t != 0 {
                self.ram.write_u16(DRAG_PLAYER_Y, (t as i8 as i16) as u16);
                self.ram.set_sprite_y_low(k, y);
            }
        }
    }
    pub fn somaria_platform_handle_drag(&mut self, k: usize) {
        self.somaria_platform_handle_drag_x(k);
        self.somaria_platform_handle_drag_y(k);
    }

    pub fn somaria_platform_drag_link(&mut self, _k: usize) {
        let x = self
            .ram
            .read_u16(0xfd8)
            .wrapping_sub(8)
            .wrapping_sub(self.ram.read_u16(LINK_X));
        if x != 0 {
            let d = if x & 0x8000 != 0 { -1 } else { 1 };
            self.ram.write_u16(
                DRAG_PLAYER_X,
                self.ram
                    .read_u16(DRAG_PLAYER_X)
                    .wrapping_add_signed(d as i16),
            );
        }
        let y = self
            .ram
            .read_u16(0xfda)
            .wrapping_sub(16)
            .wrapping_sub(self.ram.read_u16(LINK_Y));
        if y != 0 {
            let d = if y & 0x8000 != 0 { -1 } else { 1 };
            self.ram.write_u16(
                DRAG_PLAYER_Y,
                self.ram
                    .read_u16(DRAG_PLAYER_Y)
                    .wrapping_add_signed(d as i16),
            );
        }
    }

    pub fn sprite_ae_pipe_down(&mut self, k: usize) {
        const DIRS: [u8; 4] = [8, 4, 2, 1];
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).graphics() {
            0 => {
                self.ram.write_u8(ALT_SPRITE_SPAWNED_FLAG, 0xff);
                self.ram
                    .set_sprite_scratch_d(k, self.ram.sprite(k).kind().wrapping_sub(0xae));
                self.somaria_platform_locate_path(k);
            }
            1 => {
                let mut tile = self.somaria_platform_and_pipe_check_tile(k);
                if tile == 0xbe {
                    self.ram
                        .set_sprite_graphics(k, self.ram.sprite(k).graphics().wrapping_add(1));
                    tile = self.ram.sprite(k).scratch_d() ^ 1;
                    self.ram.set_sprite_scratch_d(k, tile);
                }
                self.ram.set_sprite_scratch_e(k, tile);
                self.ram
                    .set_sprite_head_direction(k, self.ram.sprite(k).scratch_d());
                self.somaria_platform_and_pipe_handle_movement(k);
                self.runtime.move_xy(self.ram, k);
            }
            2 => {
                if self.ram.read_u8(ALT_SPRITE_SPAWNED_FLAG) == 0xff
                    && self.runtime.check_damage_ignore_layer(self.ram, k)
                {
                    if !self.runtime.pipe_validate_entry(self.ram) {
                        self.ram
                            .set_sprite_graphics(k, self.ram.sprite(k).graphics().wrapping_add(1));
                        self.ram.set_sprite_aux1_delay(k, 4);
                        self.runtime.link_reset_properties_a(self.ram);
                        self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                        self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 1);
                        self.ram.write_u8(ALT_SPRITE_SPAWNED_FLAG, k as u8);
                    } else {
                        self.runtime.halt_all_movement(self.ram);
                    }
                }
            }
            3 => {
                if self.ram.sprite(k).aux1_delay() == 0 {
                    self.ram
                        .set_sprite_graphics(k, self.ram.sprite(k).graphics().wrapping_add(1));
                    self.ram.write_u8(LINK_VISIBILITY_STATUS, 12);
                } else {
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 1);
                    self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 1);
                    let d = self.ram.sprite(k).scratch_d() as usize;
                    self.pipe_handle_player_movement(DIRS[d]);
                }
            }
            4 => {
                self.ram.set_sprite_subtype2(k, 3);
                let lx = self.ram.read_u16(LINK_X);
                let ly = self.ram.read_u16(LINK_Y);
                let [xl, xh] = lx.to_le_bytes();
                let [yl, yh] = ly.to_le_bytes();
                self.ram.write_u8(LINK_X_SAFE_LO, xl);
                self.ram.write_u8(LINK_X_SAFE_HI, xh);
                self.ram.write_u8(LINK_Y_SAFE_LO, yl);
                self.ram.write_u8(LINK_Y_SAFE_HI, yh);
                loop {
                    self.ram
                        .set_sprite_scratch_a(k, self.ram.sprite(k).scratch_a().wrapping_add(1));
                    if self.ram.sprite(k).scratch_a() & 7 == 0 {
                        let tile = self.somaria_platform_and_pipe_check_tile(k);
                        if (0xb2..0xb6).contains(&tile) {
                            self.runtime.queue_sfx2(self.ram, k, 0x0b);
                        }
                        if tile != self.ram.sprite(k).scratch_e() {
                            self.ram.set_sprite_scratch_e(k, tile);
                            if tile == 0xbe {
                                self.ram.set_sprite_graphics(
                                    k,
                                    self.ram.sprite(k).graphics().wrapping_add(1),
                                );
                                self.ram.set_sprite_aux1_delay(k, 24);
                            }
                            self.ram
                                .set_sprite_head_direction(k, self.ram.sprite(k).scratch_d());
                            self.somaria_platform_and_pipe_handle_movement(k);
                            self.somaria_platform_handle_drag(k);
                        }
                    }
                    self.runtime.move_xy(self.ram, k);
                    let x = self.sprite_x(k).wrapping_sub(8);
                    let y = self.sprite_y(k).wrapping_sub(14);
                    let lx = self.ram.read_u16(LINK_X);
                    let ly = self.ram.read_u16(LINK_Y);
                    if x != lx {
                        self.ram
                            .write_u16(LINK_X, lx.wrapping_add(if x < lx { u16::MAX } else { 1 }));
                    }
                    if y != ly {
                        self.ram
                            .write_u16(LINK_Y, ly.wrapping_add(if y < ly { u16::MAX } else { 1 }));
                    }
                    let n = self.ram.sprite(k).subtype2().wrapping_sub(1);
                    self.ram.set_sprite_subtype2(k, n);
                    if n == 0 {
                        break;
                    }
                }
                self.ram.write_u8(
                    LINK_X_VEL,
                    self.ram
                        .read_u8(LINK_X)
                        .wrapping_sub(self.ram.read_u8(LINK_X_SAFE_LO)),
                );
                self.ram.write_u8(
                    LINK_Y_VEL,
                    self.ram
                        .read_u8(LINK_Y)
                        .wrapping_sub(self.ram.read_u8(LINK_Y_SAFE_LO)),
                );
                let d = self.ram.sprite(k).scratch_d() as usize;
                self.ram.write_u8(LINK_DIRECTION_LAST, DIRS[d]);
                self.runtime.link_handle_moving_animation(self.ram);
                self.runtime.handle_indoor_camera_and_doors(self.ram);
                self.runtime.link_cancel_dash(self.ram);
            }
            5 => {
                if self.ram.sprite(k).aux1_delay() == 0 {
                    self.ram.write_u8(FLAG_IS_LINK_IMMOBILIZED, 0);
                    self.ram.write_u8(PLAYER_ON_SOMARIA_PLATFORM, 0);
                    self.ram.write_u8(LINK_DISABLE_SPRITE_DAMAGE, 0);
                    self.ram.write_u8(LINK_VISIBILITY_STATUS, 0);
                    self.ram.write_u8(LINK_X_VEL, 0);
                    self.ram.write_u8(LINK_Y_VEL, 0);
                    self.ram.write_u8(ALT_SPRITE_SPAWNED_FLAG, 0xff);
                    self.ram.set_sprite_graphics(k, 2);
                } else {
                    let d = (self.ram.sprite(k).scratch_d() ^ 1) as usize;
                    self.pipe_handle_player_movement(DIRS[d]);
                }
            }
            _ => {}
        }
    }

    pub fn pipe_handle_player_movement(&mut self, direction: u8) {
        self.ram.write_u8(LINK_DIRECTION_LAST, direction);
        self.ram.write_u8(LINK_DIRECTION, direction);
        self.runtime.link_handle_velocity(self.ram);
        self.runtime.link_handle_moving_animation(self.ram);
        self.runtime.handle_indoor_camera_and_doors(self.ram);
    }

    pub fn faerie_handle_movement(&mut self, k: usize) {
        let frame = self.ram.read_u8(FRAME_COUNTER);
        self.ram.set_sprite_graphics(k, frame >> 3 & 1);
        if self.ram.read_u8(PLAYER_IS_INDOORS) != 0 && self.ram.sprite(k).aux1_delay() == 0 {
            if self.runtime.check_tile_collision(self.ram, k) & 3 != 0 {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityX,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityX)
                        .wrapping_neg(),
                );
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::ScratchD,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::ScratchD)
                        .wrapping_neg(),
                );
                self.ram.set_sprite_aux1_delay(k, 32);
            }
            if self.ram.sprite(k).wall_collision() & 12 != 0 {
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::VelocityY,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::VelocityY)
                        .wrapping_neg(),
                );
                self.ram.set_sprite_signed_field(
                    k,
                    SpriteField::ScratchA,
                    self.ram
                        .sprite(k)
                        .signed_field(SpriteField::ScratchA)
                        .wrapping_neg(),
                );
                self.ram.set_sprite_aux1_delay(k, 32);
            }
        }
        let xv = self.ram.sprite(k).signed_field(SpriteField::VelocityX);
        if xv != 0 {
            let f = self.ram.sprite(k).oam_flags();
            self.ram
                .set_sprite_oam_flags(k, if xv < 0 { f & !0x40 } else { f | 0x40 });
        }
        self.runtime.move_xy(self.ram, k);
        if frame & 63 == 0 {
            let x = (self.ram.read_u16(LINK_X) & !0xff)
                .wrapping_add(self.runtime.random(self.ram) as u16);
            let y = (self.ram.read_u16(LINK_Y) & !0xff)
                .wrapping_add(self.runtime.random(self.ram) as u16);
            let (yv, xv) = self.runtime.project_speed(self.ram, k, x, y, 16);
            self.ram
                .set_sprite_signed_field(k, SpriteField::ScratchA, yv);
            self.ram
                .set_sprite_signed_field(k, SpriteField::ScratchD, xv);
        }
        if frame & 15 == 0 {
            let y = ((self.ram.sprite(k).signed_field(SpriteField::ScratchA) as i16
                + self.ram.sprite(k).signed_field(SpriteField::VelocityY) as i16)
                >> 1) as i8;
            let x = ((self.ram.sprite(k).signed_field(SpriteField::ScratchD) as i16
                + self.ram.sprite(k).signed_field(SpriteField::VelocityX) as i16)
                >> 1) as i8;
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityY, y);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityX, x);
        }
        self.runtime.move_z(self.ram, k);
        let dz = if self.runtime.random(self.ram) & 1 != 0 {
            -1
        } else {
            1
        };
        self.ram.set_sprite_signed_field(
            k,
            SpriteField::VelocityZ,
            self.ram
                .sprite(k)
                .signed_field(SpriteField::VelocityZ)
                .wrapping_add(dz),
        );
        let z = self.ram.sprite(k).z();
        if z < 8 {
            self.ram.set_sprite_z(k, 8);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityZ, 5);
        } else if z >= 24 {
            self.ram.set_sprite_z(k, 24);
            self.ram
                .set_sprite_signed_field(k, SpriteField::VelocityZ, -5);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Rt;
    impl SpriteMainBank2bRuntime for Rt {}
    fn with_bank(f: impl FnOnce(&mut SpriteMainBank2b<'_, '_, Rt>)) {
        let mut bytes = [0; z3_core::WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        let mut bank = SpriteMainBank2b {
            ram: &mut ram,
            runtime: &mut rt,
        };
        f(&mut bank);
    }
    #[test]
    fn arrghus_phase_adds_only_low_byte_of_step() {
        with_bank(|b| {
            b.ram.write_u16(OVERLORD_X_LO, 0x0800);
            b.ram.write_u8(OVERLORD_X_LO + 4, 0xa0);
            b.ram.write_u8(OVERLORD_X_LO + 5, 0xe0);
            b.arrghus_handle_puffs(0);
            assert_eq!(b.ram.read_u16(OVERLORD_X_LO), 0x08a0);
        });
    }
    #[test]
    fn shop_cost_is_atomic_and_16_bit() {
        with_bank(|b| {
            b.ram.write_u16(LINK_RUPEES_GOAL, 149);
            assert!(!b.shop_item_handle_cost(150));
            assert_eq!(b.ram.read_u16(LINK_RUPEES_GOAL), 149);
            b.ram.write_u16(LINK_RUPEES_GOAL, 150);
            assert!(b.shop_item_handle_cost(150));
            assert_eq!(b.ram.read_u16(LINK_RUPEES_GOAL), 0);
        });
    }
    #[test]
    fn somaria_corner_preserves_c_xor_rules() {
        with_bank(|b| {
            b.ram.set_sprite_scratch_d(0, 1);
            b.ram.set_sprite_scratch_e(0, 0xb2);
            b.somaria_platform_and_pipe_handle_movement(0);
            assert_eq!(b.ram.sprite(0).scratch_d(), 2);
            assert_eq!(b.ram.sprite(0).signed_field(SpriteField::VelocityX), -16);
            assert_eq!(b.ram.sprite(0).signed_field(SpriteField::VelocityY), 0);
        });
    }
    #[test]
    fn drag_alignment_uses_wrapping_byte_delta() {
        with_bank(|b| {
            b.ram.set_sprite_scratch_d(0, 2);
            b.ram.set_sprite_head_direction(0, 0);
            b.ram.set_sprite_x_low(0, 7);
            b.somaria_platform_handle_drag_x(0);
            assert_eq!(b.ram.sprite(0).x_low(), 4);
            assert_eq!(b.ram.read_i8(DRAG_PLAYER_X), -3);
        });
    }
    #[test]
    fn old_man_entering_home_grants_mirror_and_starts_cutscene() {
        with_bank(|b| {
            b.ram.set_sprite_subtype2(0, 1);
            b.sprite_ad_old_man(0);
            assert_eq!(b.ram.sprite(0).ai_state(), 1);
            assert_eq!(b.ram.read_u8(WHICH_STARTING_POINT), 1);
            assert_eq!(b.ram.read_u8(FLAG_IS_LINK_IMMOBILIZED), 1);
            assert_eq!(b.ram.sprite(0).signed_field(SpriteField::VelocityX), 8);
            assert_eq!(b.ram.sprite(0).signed_field(SpriteField::VelocityY), 4);
        });
    }
    #[test]
    fn yellow_stalfos_first_tick_preserves_c_initialization() {
        with_bank(|b| {
            b.sprite_85_yellow_stalfos(0);
            assert_eq!(b.ram.sprite(0).scratch_a(), 1);
            assert_eq!(b.ram.sprite(0).scratch_c(), 10);
            assert_eq!(b.ram.sprite(0).flags3(), 0x40);
            assert_eq!(b.ram.sprite(0).head_direction(), 2);
        });
    }
    #[test]
    fn agahnim_direction_table_uses_signed_velocity_components() {
        with_bank(|b| {
            b.ram.set_sprite_ai_state(0, 3);
            b.ram.set_sprite_main_delay(0, 128);
            b.ram.set_sprite_signed_field(0, SpriteField::VelocityX, -2);
            b.ram.set_sprite_signed_field(0, SpriteField::VelocityY, -2);
            b.sprite_7a_agahnim(0);
            assert_eq!(b.ram.sprite(0).scratch_d(), 0);
        });
    }
}
