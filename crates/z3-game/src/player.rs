//! Player control, movement, items, and collision logic ported from `player.c`.
//!
//! WRAM addresses deliberately retain the original aliases. Byte and word
//! arithmetic is explicitly wrapping to preserve the 65c816/C behavior.

use crate::ram::Ram;
use crate::swim_state::SwimAxis;

pub const SWIMMING_TAB1: [u8; 4] = [2, 0, 1, 0];
pub const SWIMMING_TAB2: [u8; 2] = [32, 8];
pub const SPIN_ATTACK_DELAYS: [u8; 18] = [1, 0, 0, 0, 0, 3, 0, 0, 1, 0, 3, 3, 3, 3, 4, 4, 1, 5];
pub const FIRE_BEAM_SOUNDS: [u8; 8] = [1, 2, 3, 4, 0, 9, 18, 27];
pub const TAGALONG_ARR1: [i8; 15] = [-1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const TAGALONG_ARR2: [i8; 15] = [-1, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const LINK_SPIN_GRAPHICS_BY_DIR: [u8; 48] = [
    10, 11, 10, 6, 7, 8, 9, 2, 3, 4, 5, 10, 0, 1, 0, 2, 3, 4, 5, 6, 7, 8, 9, 0, 12, 13, 12, 4, 5,
    6, 7, 8, 9, 2, 3, 12, 14, 15, 14, 8, 9, 2, 3, 4, 5, 6, 7, 14,
];
pub const LINK_SPIN_DELAYS: [u8; 12] = [1, 5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 5];
pub const GRAB_WALL_DIRS: [u8; 4] = [4, 8, 1, 2];
pub const GRAB_WALL_ANIM_STEPS: [u8; 7] = [0, 1, 2, 3, 1, 2, 3];
pub const GRAB_WALL_ANIM_TIMER: [u8; 7] = [0, 5, 5, 12, 5, 5, 12];
pub const CAPE_DEPLETION_TIMERS: [u8; 3] = [4, 8, 8];
const BOMBOS_ANIM_DELAYS: [u8; 20] = [5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 7, 1, 1, 1, 1, 1, 13];
const BOMBOS_ANIM_STATES: [u8; 20] = [
    0, 1, 2, 3, 0, 1, 2, 3, 8, 9, 10, 11, 12, 10, 8, 13, 14, 15, 16, 17,
];
const ETHER_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 3, 3];
const ETHER_ANIM_DELAYS_NO_FLASH: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 24, 24];
const ETHER_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7];
const QUAKE_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 19];
const QUAKE_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 18, 19, 20, 22];
pub const LINK_ITEM_MAGIC_COSTS: [u8; 27] = [
    16, 8, 4, 32, 16, 8, 8, 4, 2, 8, 4, 2, 8, 4, 2, 16, 8, 4, 4, 2, 2, 8, 4, 2, 16, 8, 4,
];
pub const AVOID_JUDDER: [i8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7,
];
pub const PUSH_BLOCK_TILE_SOLID: [u8; 256] = [
    0, 1, 2, 3, 2, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1,
    0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1,
    0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

fn add_i8(value: u16, delta: i8) -> u16 {
    value.wrapping_add(delta as i16 as u16)
}

fn recoil_z_velocity(copy: u8, counter: u8) -> u8 {
    let mut velocity = copy >> 1;
    if counter == 1 {
        velocity >>= 1;
    }
    velocity
}

#[inline]
pub const fn bit_sum4(value: u8) -> u8 {
    (value & 1) + ((value >> 1) & 1) + ((value >> 2) & 1) + ((value >> 3) & 1)
}

#[repr(u8)]
pub enum PlayerState {
    Ground = 0,
    FallingIntoHole = 1,
    RecoilWall = 2,
    SpinAttacking = 3,
    Swimming = 4,
    TurtleRock = 5,
    RecoilOther = 6,
    Electrocution = 7,
    Ether = 8,
    Bombos = 9,
    Quake = 10,
    FallOfLeftRightLedge = 12,
    JumpOffLedgeDiag = 14,
    StartDash = 17,
    StopDash = 18,
    Hookshot = 19,
    Mirror = 20,
    HoldUpItem = 21,
    AsleepInBed = 22,
    PermaBunny = 23,
    ReceivingEther = 25,
    ReceivingBombos = 26,
    OpeningDesertPalace = 27,
    TempBunny = 28,
    PullForRupees = 29,
    SpinAttackMotion = 30,
}

pub trait PlayerRuntime {
    fn set_and_save_visited_quadrant_flags(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_flag_room_data_quadrants(&mut self, _ram: &mut Ram<'_>) {}
    fn overworld_get_pit_destination(&mut self, _ram: &mut Ram<'_>) {}
    fn take_damage_from_pit(&mut self, _ram: &mut Ram<'_>) {}
    fn link_oam_main(&mut self, _ram: &mut Ram<'_>) {}
    fn follower_initialize(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_start_inter_room_transition(&mut self, _ram: &mut Ram<'_>, _direction: u8) {}
    fn adjust_quadrant_and_camera(&mut self, _ram: &mut Ram<'_>, _direction: u8) {}
    fn load_gear_palettes_bunny(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_gravestone(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_dash_tremor(&mut self, _ram: &mut Ram<'_>) {}
    fn prepare_apply_rumble_to_sprites(&mut self, _ram: &mut Ram<'_>) {}
    fn mirror_save_room_data(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_move_y(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn ancilla_move_x(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn ancilla_move_z(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn ancilla_check_link_collision(&mut self, _ram: &mut Ram<'_>, _k: usize, _kind: u8) -> bool {
        false
    }
    fn ancilla_check_tile_collision_class2(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn ancilla_sfx2_pan(&mut self, _ram: &mut Ram<'_>, _k: usize, _sound: u8) {}
    fn ancilla_sfx3_pan(&mut self, _ram: &mut Ram<'_>, _k: usize, _sound: u8) {}
    fn sprite_nullify_hookshot_drag(&mut self, _ram: &mut Ram<'_>) {}
    fn overworld_do_map_update_32x32_b(&mut self, _ram: &mut Ram<'_>) {}
    fn tile_detect_movement_vertical_slopes(&mut self, _ram: &mut Ram<'_>, _direction: u8) {}
    fn tile_detect_movement_horizontal_slopes(&mut self, _ram: &mut Ram<'_>, _direction: u8) {}
    fn tile_detect_movement_x(&mut self, _ram: &mut Ram<'_>, _direction: u8) {}
    fn tile_detect_movement_y(&mut self, _ram: &mut Ram<'_>, _direction: u8) {}
    fn player_tile_detect_nearby(&mut self, _ram: &mut Ram<'_>) {}
    fn tile_detect_reset_state(&mut self, _ram: &mut Ram<'_>) {}
    fn tile_detection_execute(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16, _bits: u8) {}
    fn handle_item_tile_action_overworld(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) -> u8 {
        0
    }
    fn tile_detect_execute_inner(
        &mut self,
        _ram: &mut Ram<'_>,
        _tile: u8,
        _arg1: u8,
        _arg2: u8,
        _arg3: bool,
    ) {
    }
    fn enhanced_features0(&self) -> u32 {
        0
    }
    fn ancilla_add_splash(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) -> bool {
        false
    }
    fn handle_nudging_in_a_door(&mut self, _ram: &mut Ram<'_>, _speed: i8) {}
    fn ancilla_sfx2_near(&mut self, _ram: &mut Ram<'_>, _sound: u8) -> u8 {
        0
    }
    fn ancilla_sfx3_near(&mut self, _ram: &mut Ram<'_>, _sound: u8) {}
    fn dungeon_delete_rupee_tile(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) {}
    fn open_chest_for_item(&mut self, _ram: &mut Ram<'_>, _tile_type: u16) {}
    fn draw_push_block_oam(&mut self, _ram: &mut Ram<'_>, _x: i16, _y: i16, _character: u8) {}
    fn overworld_smash_rock_pile(
        &mut self,
        _ram: &mut Ram<'_>,
        _second: bool,
    ) -> Option<(u8, u16, u16)> {
        None
    }
    fn sprite_spawn_immediately_smashed_terrain(
        &mut self,
        _ram: &mut Ram<'_>,
        _kind: u8,
        _x: u16,
        _y: u16,
    ) {
    }
    fn ancilla_add_ether_spell(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_bombos_spell(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_quake_spell(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_cape_poof(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn link_zap_handle_mosaic(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_electro_themed_gear(&mut self, _ram: &mut Ram<'_>) {}
    fn player_set_custom_mosaic_level(&mut self, _ram: &mut Ram<'_>, _level: u8) {}
    fn ancilla_add_item_receipt(
        &mut self,
        _ram: &mut Ram<'_>,
        _type_: u8,
        _arg: u8,
        _chest_position: i32,
    ) {
    }
    fn hud_refresh_icon(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_blanket(&mut self, _ram: &mut Ram<'_>, _type_: u8) {}
    fn ancilla_add_snoring(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) -> Option<usize> {
        None
    }
    fn decode_animated_sprite_tile_variable(&mut self, _ram: &mut Ram<'_>, _tile: u8) {}
    fn ancilla_set_xy(&mut self, _ram: &mut Ram<'_>, _k: usize, _x: u16, _y: u16) {}
    fn ancilla_terminate_select_interactives(&mut self, _ram: &mut Ram<'_>, _arg: u8) {}
    fn ancilla_add_bomb(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn release_fairy(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn release_bee_from_bottle(&mut self, _ram: &mut Ram<'_>, _index: usize) -> bool {
        false
    }
    fn hud_rebuild(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_magic_powder(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn dungeon_light_torch(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_lamp_flame(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_exploding_weather_vane(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_duck_take_off(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn dungeon_get_tele_msg(&mut self, _ram: &mut Ram<'_>, _room: u16) -> u16 {
        0
    }
    fn overworld_get_sign_text(&mut self, _ram: &mut Ram<'_>, _screen: u16) -> u16 {
        0
    }
    fn tile_check_for_mirror_bonk(&mut self, _ram: &mut Ram<'_>) {}
    fn load_follower_graphics(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_dwarf_poof(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_bunny_poof(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_fire_rod_shot(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_ice_rod_shot(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_arrow(
        &mut self,
        _ram: &mut Ram<'_>,
        _type_: u8,
        _direction: u8,
        _arg: u8,
        _x: u16,
        _y: u16,
    ) -> Option<usize> {
        None
    }
    fn ancilla_add_boomerang(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) -> i32 {
        -1
    }
    fn ancilla_add_hit_stars(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn overworld_read_tile_attribute(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) -> u8 {
        0
    }
    fn sprite_spawn_small_splash(&mut self, _ram: &mut Ram<'_>, _arg: u8) -> Option<usize> {
        None
    }
    fn sprite_set_x(&mut self, _ram: &mut Ram<'_>, _slot: usize, _x: u16) {}
    fn sprite_set_y(&mut self, _ram: &mut Ram<'_>, _slot: usize, _y: u16) {}
    fn ancilla_add_somaria_block(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) -> bool {
        false
    }
    fn random_number(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
    fn sprite_spawn_dynamically(
        &mut self,
        _ram: &mut Ram<'_>,
        _source: usize,
        _kind: u8,
    ) -> Option<usize> {
        None
    }
    fn sprite_sfx3_with_pan(&mut self, _ram: &mut Ram<'_>, _slot: usize, _sound: u8) {}
    fn current_item_button_override(
        &mut self,
        _ram: &mut Ram<'_>,
    ) -> Option<(u8, Option<u8>, bool)> {
        None
    }
    fn add_sword_beam(&mut self, _ram: &mut Ram<'_>, _arg: u8) {}
    fn calculate_sfx_pan(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
    fn calculate_sword_hit_box(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_spawn_sword_charge_sparkle(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_charged_spin_attack_sparkle(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_wall_tap_spark(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn tile_detect_sword_swing_deep_in_door(&mut self, _ram: &mut Ram<'_>, _doorway: u8) {}
    fn ancilla_add_sword_swing_sparkle(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_spin_attack_init_spark(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8) {}
    fn ancilla_add_dash_dust(&mut self, _ram: &mut Ram<'_>, _charging: bool, _type_: u8, _arg: u8) {
    }
    fn dungeon_is_pit_that_hurts_player(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn liftable_probe_offset(&self, _direction: usize) -> (i8, i8) {
        (0, 0)
    }
    fn dungeon_check_for_liftable_tile(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0xff
    }
    fn open_chest(&mut self, _ram: &mut Ram<'_>, _tile: u8) -> Option<(u8, i32)> {
        None
    }
    fn item_memory_location(&self, _item: u8) -> usize {
        0
    }
    fn lift_and_replace_tile(&mut self, _ram: &mut Ram<'_>, _indoors: bool) -> (u8, u16, u16) {
        (0xff, 0, 0)
    }
    fn sprite_spawn_throwable_terrain(
        &mut self,
        _ram: &mut Ram<'_>,
        _kind: usize,
        _x: u16,
        _y: u16,
    ) {
    }
    fn ancilla_add_falling_prize(&mut self, _ram: &mut Ram<'_>, _type_: u8, _arg: u8, _arg2: u8) {}
    fn load_actual_gear_palettes(&mut self, _ram: &mut Ram<'_>) {}
    fn main_show_text_message(&mut self, _ram: &mut Ram<'_>) {}
}

pub struct Player<'a, 'r, R: PlayerRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
    pub applied_movement_to_camera: bool,
}

impl<R: PlayerRuntime> Player<'_, '_, R> {
    fn inc8(&mut self, a: usize) -> u8 {
        let v = self.ram.read_u8(a).wrapping_add(1);
        self.ram.write_u8(a, v);
        v
    }
    pub fn dungeon_handle_layer_change(&mut self) {
        self.ram.write_u8(0x476, 1);
        if self.ram.read_u8(0x44a) == 0 {
            self.inc8(0xa0);
        }
        if self.ram.read_u8(0x44a) != 2 {
            self.ram.write_u8(0xee, 1)
        }
        self.ram.write_u8(0x47a, 0);
        self.runtime.set_and_save_visited_quadrant_flags(self.ram)
    }
    pub fn cache_camera_properties(&mut self) {
        const WORDS: [(usize, usize); 13] = [
            (0xc180, 0xe2),
            (0xc182, 0xe8),
            (0xc184, 0x20),
            (0xc186, 0x22),
            (0xc188, 0x600),
            (0xc18a, 0x604),
            (0xc18c, 0x608),
            (0xc18e, 0x60c),
            (0xc190, 0x610),
            (0xc192, 0x612),
            (0xc194, 0x614),
            (0xc196, 0x616),
            (0xc198, 0x618),
        ];
        for (dst, src) in WORDS {
            self.ram.write_u16(dst, self.ram.read_u16(src))
        }
        self.ram.write_u16(0xc19a, self.ram.read_u16(0x61c));
        for (dst, src) in [
            (0xc19c, 0xa6),
            (0xc19d, 0xa7),
            (0xc19e, 0xa9),
            (0xc19f, 0xaa),
            (0xc1a6, 0x2f),
            (0xc1a7, 0xee),
            (0xc1a8, 0x476),
            (0xc1a9, 0x6c),
            (0xc1aa, 0xa4),
        ] {
            self.ram.write_u8(dst, self.ram.read_u8(src))
        }
    }
    pub fn check_ability_to_swim(&mut self) {
        if self.ram.read_u8(0x2e0) == 0 && self.ram.player().flippers() != 0 {
            return;
        }
        if self.ram.player().moon_pearl() != 0 {
            self.ram.write_u8(0x56, 0)
        }
        self.ram.write_u8(0x4b, 0xc);
        self.ram
            .write_u8(0x11, if self.ram.read_u8(0x1b) != 0 { 20 } else { 42 })
    }
    pub fn link_main(&mut self) {
        self.ram.write_u16(0xfc2, self.ram.player().x());
        self.ram.write_u16(0xfc4, self.ram.player().y());
        self.ram.write_u8(0xfc1, 0);
        if self.ram.read_u8(0x2e4) == 0 {
            self.link_control_handler()
        }
        self.handle_somaria_and_graves()
    }
    pub fn handle_somaria_and_graves(&mut self) {
        if self.ram.read_u8(0x1b) == 0 && self.ram.read_u8(0x3e9) != 0 {
            for k in (0..5).rev() {
                if self.ram.read_u8(0xc4a + k) == 0x24 {
                    self.gravestone_move(k)
                }
            }
        }
        for k in (0..5).rev() {
            if self.ram.read_u8(0xc4a + k) == 0x2c {
                self.somaria_block_handle_player_interaction(k);
                return;
            }
        }
    }

    pub fn somaria_block_handle_player_interaction(&mut self, k: usize) {
        const YV: [i8; 4] = [-40, 40, 0, 0];
        const XV: [i8; 4] = [0, 0, -40, 40];
        const ZV: [u8; 4] = [48, 24, 16, 8];
        self.ram.write_u8(0xfa0, k as u8);
        if self.ram.read_u8(0x394 + k) != 0 {
            return;
        }
        if self.ram.read_u8(0x3c5 + k) == 0 {
            let z = self.ram.read_u8(0x29e + k);
            if self.ram.read_u8(0x4d) != 0
                || self.ram.read_u8(0x308) & 1 != 0
                || !matches!(z, 0 | 0xff)
                || self.ram.read_u8(0x380 + k) != 0
                || self.ram.read_u8(0x385 + k) != 0
            {
                return;
            }
            let input = self.ram.read_u8(0xf0) & 0xf;
            if input == 0 {
                self.ram.write_u8(0x39f + k, 0);
                self.ram.write_u8(0x48, 0);
                self.ram.write_u8(0x38a + k, 0xff);
                if self.ram.read_u8(0x372) == 0 {
                    self.ram.write_u8(0x5e, 0);
                    return;
                }
            } else if input == self.ram.read_u8(0x39f + k) {
                if self.ram.read_u8(0x5e) == 18 {
                    self.ram.write_u8(0x48, self.ram.read_u8(0x48) | 0x81);
                }
            } else {
                self.ram.write_u8(0x39f + k, input);
                self.ram.write_u8(0x5e, 0);
            }
            if !self.runtime.ancilla_check_link_collision(self.ram, k, 4)
                || self.ram.read_u8(0xc7c + k) != self.ram.read_u8(0xee)
            {
                return;
            }
            if self.ram.read_u8(0x372) == 0 || self.ram.read_u8(0x2f1) == 64 {
                self.ram.write_u8(0xc2c + k, 0);
                self.ram.write_u8(0xc22 + k, 0);
                self.ram.write_u8(0x39f + k, input);
                if input & 3 != 0 {
                    self.ram
                        .write_u8(0xc2c + k, if input & 1 != 0 { 16 } else { 0xf0 });
                    self.ram
                        .write_u8(0xc72 + k, if input & 1 != 0 { 3 } else { 2 });
                } else {
                    self.ram
                        .write_u8(0xc22 + k, if input & 8 != 0 { 0xf0 } else { 16 });
                    self.ram
                        .write_u8(0xc72 + k, if input & 8 != 0 { 0 } else { 1 });
                }
                if self.ram.player().actual_velocity_y() == 0
                    || self.ram.player().actual_velocity_x() == 0
                {
                    if !self
                        .runtime
                        .ancilla_check_tile_collision_class2(self.ram, k)
                    {
                        self.runtime.ancilla_move_y(self.ram, k);
                        self.runtime.ancilla_move_x(self.ram, k);
                        let a = self.ram.read_u8(0x38a + k).wrapping_add(1);
                        self.ram.write_u8(0x38a + k, a);
                        if self.ram.read_u8(0x308) & 0x80 == 0 && a & 7 == 0 {
                            self.runtime.ancilla_sfx2_pan(self.ram, k, 0x22);
                        }
                    }
                    self.ram.write_u8(0x48, 0x81);
                    self.ram.write_u8(0x5e, 0x12);
                }
                self.runtime.sprite_nullify_hookshot_drag(self.ram);
                return;
            }
            if self.ram.read_u8(0x2ec) == (k + 1) as u8 {
                self.ram.write_u8(0x2ec, 0);
            }
            self.link_cancel_dash();
            self.runtime.ancilla_sfx3_pan(self.ram, k, 0x32);
            let d = usize::from(self.ram.player().facing() >> 1);
            self.ram.write_u8(0xc72 + k, d as u8);
            self.ram.write_u8(0xc22 + k, YV[d] as u8);
            self.ram.write_u8(0xc2c + k, XV[d] as u8);
            self.ram.write_u8(0x294 + k, 48);
            self.ram.write_u8(0x3c5 + k, 1);
            self.ram.write_u8(0x29e + k, 0);
        }
        self.ram
            .write_u8(0x294 + k, self.ram.read_u8(0x294 + k).wrapping_sub(2));
        self.runtime.ancilla_move_y(self.ram, k);
        self.runtime.ancilla_move_x(self.ram, k);
        self.runtime.ancilla_move_z(self.ram, k);
        let z = self.ram.read_u8(0x29e + k);
        if z != 0 && z < 252 {
            return;
        }
        self.runtime.ancilla_sfx2_pan(self.ram, k, 0x21);
        self.ram.write_u8(0x29e + k, 0);
        let phase = self.ram.read_u8(0x3c5 + k);
        self.ram.write_u8(0x3c5 + k, phase.wrapping_add(1));
        if phase == 3 {
            self.ram.write_u8(0xbf0 + k, 0);
            self.ram.write_u8(0x3c5 + k, 0);
        } else {
            self.ram.write_u8(0x294 + k, ZV[usize::from(phase - 1)]);
            self.ram
                .write_u8(0xc22 + k, (self.ram.read_u8(0xc22 + k) as i8 / 2) as u8);
            self.ram
                .write_u8(0xc2c + k, (self.ram.read_u8(0xc2c + k) as i8 / 2) as u8);
        }
    }

    pub fn sprite_dungeon_draw_single_push_block(&mut self, index: u8) {
        const KIND: [u8; 9] = [0, 1, 2, 3, 4, 0, 0, 0, 0];
        const CHARACTER: [u8; 4] = [0xc, 0xc, 0xc, 0xff];
        let slot = usize::from(index >> 1);
        let y = u16::from_le_bytes([
            self.ram.read_u16(0x5f0 + slot * 2) as u8,
            self.ram.read_u16(0x5ec + slot * 2) as u8,
        ])
        .wrapping_sub(self.ram.read_u16(0xe8).wrapping_add(1));
        let x = u16::from_le_bytes([
            self.ram.read_u16(0x5e4 + slot * 2) as u8,
            self.ram.read_u16(0x5e0 + slot * 2) as u8,
        ])
        .wrapping_sub(self.ram.read_u16(0xe2));
        let character = CHARACTER[usize::from(KIND[usize::from(self.ram.read_u8(0x2c3))])];
        if character != 0xff {
            self.runtime
                .draw_push_block_oam(self.ram, x as i16, y as i16, character);
        }
    }

    pub fn gravestone_move(&mut self, k: usize) {
        if self.ram.read_u8(0x11) != 0 {
            return;
        }
        self.ram.write_u8(0xc22 + k, (-8i8) as u8);
        self.runtime.ancilla_move_y(self.ram, k);
        self.gravestone_act_as_barrier(k);
        let target = u16::from_be_bytes([self.ram.read_u8(0x38f + k), self.ram.read_u8(0x38a + k)]);
        let current =
            u16::from_le_bytes([self.ram.read_u8(0xbfa + k), self.ram.read_u8(0xc0e + k)]);
        if current >= target {
            return;
        }
        self.ram.write_u8(0xc4a + k, 0);
        self.ram.write_u8(0x3e9, 0);
        self.ram.write_u8(0x48, self.ram.read_u8(0x48) & !4);
        let starting_address =
            u16::from_le_bytes([self.ram.read_u8(0x732 + k), self.ram.read_u8(0x728 + k)]);
        self.ram.write_u16(0x72, starting_address);
        self.ram.write_u16(0x698, starting_address);
        self.ram.write_u16(
            0x692,
            match starting_address {
                0x532 => 0x48,
                0x488 => 0x60,
                _ => 0x40,
            },
        );
        self.runtime.overworld_do_map_update_32x32_b(self.ram)
    }

    pub fn gravestone_act_as_barrier(&mut self, k: usize) {
        let x = u16::from_le_bytes([self.ram.read_u8(0xc04 + k), self.ram.read_u8(0xc18 + k)]);
        let y = u16::from_le_bytes([self.ram.read_u8(0xbfa + k), self.ram.read_u8(0xc0e + k)]);
        let right = x.wrapping_add(0x20);
        let bottom = y.wrapping_add(0x18);
        let link_x = self.ram.player().x().wrapping_add(8);
        let link_y = self.ram.player().y().wrapping_add(8);
        if link_y >= y && link_y < bottom && link_x >= x && link_x < right {
            let displacement = link_y.abs_diff(bottom);
            self.ram
                .set_player_y(self.ram.player().y().wrapping_add(displacement));
            self.ram.set_player_velocity_y(
                self.ram
                    .player()
                    .velocity_y()
                    .wrapping_add(displacement as u8),
            );
            self.ram.write_u8(0x48, self.ram.read_u8(0x48) | 4)
        }
        if self.ram.player().facing() != 0 {
            self.ram.set_player_facing(self.ram.player().facing() & !4)
        }
    }

    pub fn ancilla_add_dug_up_flute(&mut self, type_: u8, arg: u8) {
        let Some(k) = self.runtime.ancilla_add(self.ram, type_, arg) else {
            return;
        };
        self.ram.write_u8(0xc54 + k, 0);
        self.ram.write_u8(0x29e + k, 0);
        self.ram.write_u8(0x294 + k, 24);
        self.ram.write_u8(
            0xc2c + k,
            if self.ram.player().facing() == 4 {
                (-8i8) as u8
            } else {
                8
            },
        );
        self.runtime
            .decode_animated_sprite_tile_variable(self.ram, 12);
        self.runtime.ancilla_set_xy(self.ram, k, 0x490, 0xa8a)
    }

    pub fn ancilla_add_cane_of_byrna_init_spark(&mut self, type_: u8, arg: u8) {
        for k in (0..5).rev() {
            if self.ram.read_u8(0xc4a + k) == 0x31 {
                self.ram.write_u8(0xc4a + k, 0)
            }
        }
        if let Some(k) = self.runtime.ancilla_add(self.ram, type_, arg) {
            self.ram.write_u8(0xc5e + k, 0);
            self.ram.write_u8(0x3b1 + k, 9);
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x39f + k, 2)
        }
    }

    pub fn ancilla_add_shovel_dirt(&mut self, type_: u8, arg: u8) {
        if let Some(k) = self.runtime.ancilla_add(self.ram, type_, arg) {
            self.ram.write_u8(0xc5e + k, 0);
            self.ram.write_u8(0xc68 + k, 20);
            let x = self.ram.player().x();
            let y = self.ram.player().y();
            self.runtime.ancilla_set_xy(self.ram, k, x, y)
        }
    }

    pub fn ancilla_add_hookshot(&mut self, type_: u8, arg: u8) {
        const Y_VELOCITY: [i8; 4] = [-64, 64, 0, 0];
        const X_VELOCITY: [i8; 4] = [0, 0, -64, 64];
        const Y_OFFSET: [i8; 4] = [4, 20, 8, 8];
        const X_OFFSET: [i8; 4] = [0, 0, -4, 11];
        let Some(k) = self.runtime.ancilla_add(self.ram, type_, arg) else {
            return;
        };
        self.ram.write_u8(0x3b1 + k, 3);
        self.ram.write_u8(0xc5e + k, 0);
        self.ram.write_u8(0xc54 + k, 0);
        self.ram.write_u8(0x385 + k, 0);
        self.ram.write_u8(0x37e, 0);
        self.ram.write_u8(0x39d, k as u8);
        self.ram.write_u8(0x380 + k, 0);
        self.ram.write_u8(0x394 + k, 0xff);
        self.ram.write_u8(0x3a4 + k, 0);
        self.ram.write_u8(0xc68 + k, 0);
        let direction = usize::from(self.ram.player().facing() >> 1);
        self.ram.write_u8(0xc72 + k, direction as u8);
        self.ram.write_u8(0xc2c + k, X_VELOCITY[direction] as u8);
        self.ram.write_u8(0xc22 + k, Y_VELOCITY[direction] as u8);
        let x = add_i8(self.ram.player().x(), X_OFFSET[direction]);
        let y = add_i8(self.ram.player().y(), Y_OFFSET[direction]);
        self.runtime.ancilla_set_xy(self.ram, k, x, y)
    }

    pub fn reset_some_things_after_death(&mut self, speed: u8) {
        for address in [
            0x345, 0x3f3, 0x322, 0x2e4, 0xabd, 0x36b, 0x373, 0x27, 0x28, 0x29, 0x24, 0x351, 0x316,
            0x31f, 0x5d, 0x4b,
        ] {
            self.ram.write_u8(address, 0)
        }
        self.ram.write_u8(0x5e, speed);
        self.runtime
            .ancilla_terminate_select_interactives(self.ram, 0);
        self.link_reset_properties_a()
    }

    pub fn link_item_bombs(&mut self) {
        if self.ram.read_u8(0x6c) != 0
            || self.ram.read_u8(0xf3cc) == 13
            || !self.check_y_button_press()
        {
            return;
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        let count = if self.runtime.enhanced_features0() & 512 != 0 {
            3
        } else {
            1
        };
        self.runtime.ancilla_add_bomb(self.ram, 7, count);
        self.ram.write_u8(0x301, 0)
    }

    pub fn link_item_bottle(&mut self) {
        if !self.check_y_button_press() {
            return;
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        let index = usize::from(self.ram.read_u8(0xf34f).wrapping_sub(1));
        let contents = self.ram.read_u8(0xf35c + index);
        if contents == 0 {
            return;
        }
        let fail = match contents {
            0..=2 => true,
            3 if self.ram.player().max_health() == self.ram.player().health() => true,
            4 if self.ram.player().magic() == 128 => true,
            5 if self.ram.player().max_health() == self.ram.player().health()
                && self.ram.player().magic() == 128 =>
            {
                true
            }
            3..=5 => {
                self.ram.write_u8(0xf35c + index, 2);
                self.ram.write_u8(0x301, 0);
                self.ram.write_u8(
                    0x11,
                    match contents {
                        3 => 4,
                        4 => 8,
                        _ => 9,
                    },
                );
                self.ram.write_u8(0x10c, self.ram.read_u8(0x10));
                self.ram.write_u8(0x10, 14);
                self.ram.write_u8(0x208, 7);
                self.runtime.hud_rebuild(self.ram);
                false
            }
            6 => {
                self.ram.write_u8(0x301, 0);
                if !self.runtime.release_fairy(self.ram) {
                    true
                } else {
                    self.ram.write_u8(0xf35c + index, 2);
                    self.runtime.hud_rebuild(self.ram);
                    false
                }
            }
            7 | 8 => {
                if !self.runtime.release_bee_from_bottle(self.ram, index) {
                    true
                } else {
                    self.ram.write_u8(0xf35c + index, 2);
                    self.runtime.hud_rebuild(self.ram);
                    false
                }
            }
            _ => false,
        };
        if fail {
            self.runtime.ancilla_sfx2_near(self.ram, 60);
        }
    }

    pub fn link_item_lamp(&mut self) {
        if self.ram.read_u8(0x6c) != 0 || !self.check_y_button_press() {
            return;
        }
        if self.ram.read_u8(0xf34a) != 0 && self.link_check_magic_cost(6) {
            self.runtime.ancilla_add_magic_powder(self.ram, 0x1a, 0);
            self.runtime.dungeon_light_torch(self.ram);
            self.runtime.ancilla_add_lamp_flame(self.ram, 0x2f, 2)
        }
        self.ram.write_u8(0x301, 0);
        self.ram.write_u8(0x3a, 0);
        self.ram.write_u8(0x3c, 0);
        self.ram.write_u8(0x50, 0)
    }

    pub fn link_item_book(&mut self) {
        if self.ram.read_u8(0x3a) & 0x40 != 0
            || self.ram.read_u8(0x6c) != 0
            || !self.check_y_button_press()
        {
            return;
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        if self.ram.read_u8(0x2ed) != 0 {
            self.link_perform_desert_prayer()
        } else {
            self.runtime.ancilla_sfx2_near(self.ram, 60);
        }
    }

    pub fn link_item_powder(&mut self) {
        const ANIMATION_DELAYS: [u8; 10] = [2, 1, 1, 3, 2, 2, 2, 2, 6, 0];
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x6c) != 0 || !self.check_y_button_press() {
                return;
            }
            if self.ram.read_u8(0xf344) != 2 {
                self.runtime.ancilla_sfx2_near(self.ram, 60);
                self.link_item_powder_finish();
                return;
            }
            if !self.link_check_magic_cost(2) {
                self.link_item_powder_finish();
                return;
            }
            self.ram.write_u8(0x3d, ANIMATION_DELAYS[0]);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
            self.ram.write_u8(0x301, 0x40)
        }
        for address in [0x31, 0x30, 0x67, 0x2b, 0x2a, 0x6b] {
            self.ram.write_u8(address, 0)
        }
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let phase = self.inc8(0x300);
        self.ram
            .write_u8(0x3d, ANIMATION_DELAYS[usize::from(phase)]);
        if phase == 4 {
            self.runtime.ancilla_add_magic_powder(self.ram, 26, 0)
        }
        if phase != 9 {
            return;
        }
        if self.ram.read_u8(0x11) == 0 {
            self.tile_detect_main_handler(1)
        }
        self.link_item_powder_finish()
    }

    fn link_item_powder_finish(&mut self) {
        self.ram.write_u8(0x301, 0);
        self.ram.write_u8(0x300, 0);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40)
    }

    pub fn link_item_flute(&mut self) {
        if self.ram.read_u8(0x3a) & 0x40 != 0 {
            let countdown = self.ram.read_u8(0x3f0).wrapping_sub(1);
            self.ram.write_u8(0x3f0, countdown);
            if countdown != 0 {
                return;
            }
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40)
        }
        if !self.check_y_button_press() {
            return;
        }
        self.ram.write_u8(0x3f0, 128);
        self.runtime.ancilla_sfx2_near(self.ram, 19);
        if self.ram.read_u8(0x1b) != 0
            || self.ram.read_u16(0x8a) & 0x40 != 0
            || self.ram.read_u8(0x10) == 11
        {
            return;
        }
        for i in (0..5).rev() {
            if self.ram.read_u8(0xc4a + i) == 0x27 {
                return;
            }
        }
        if self.ram.read_u8(0xf34c) == 2 {
            let y = self.ram.player().y();
            let x = self.ram.player().x();
            if self.ram.read_u16(0x8a) == 0x18
                && (0x760..0x7e0).contains(&y)
                && (0x1cf..0x230).contains(&x)
            {
                self.ram.write_u8(0x11, 45);
                self.runtime
                    .ancilla_add_exploding_weather_vane(self.ram, 55, 0)
            }
        } else {
            self.runtime.ancilla_add_duck_take_off(self.ram, 39, 4);
            self.ram.write_u8(0x3f8, 0)
        }
    }

    pub fn link_perform_dash(&mut self) {
        if self.ram.read_u8(0x2f5) != 0
            || self.ram.read_u8(0x314) | self.ram.read_u8(0x2ec) != 0
            || self.ram.read_u8(0x308) & 0x80 != 0
        {
            return;
        }
        self.ram.write_u8(0x3b, 0);
        self.ram.write_u8(0x374, 29);
        self.ram.write_u8(0x2f1, 64);
        self.ram
            .set_player_action_state(PlayerState::StartDash as u8);
        self.ram.write_u8(0x372, 1);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & 0x80);
        for address in [0x308, 0x301, 0x48, 0x6b] {
            self.ram.write_u8(address, 0)
        }
        let follower = self.ram.read_u8(0xf3cc);
        if follower == TAGALONG_ARR1[usize::from(follower)] as u8 {
            self.ram.write_u8(0x5e, 0);
            self.ram.write_u8(0x2d2, 64)
        }
    }

    pub fn link_perform_grab(&mut self) {
        if self.ram.read_u8(0x3a) & 0x80 != 0 && self.ram.read_u8(0x3c) >= 9 {
            return;
        }
        self.ram.write_u8(0x376, 1);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
        for address in [0x2e, 0x30a, 0x30b, 0x30d] {
            self.ram.write_u8(address, 0)
        }
    }

    pub fn link_a_press_pull_object(&mut self) {
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let index = usize::from(self.ram.player().facing() >> 1);
        if self.ram.read_u8(0xf0) & GRAB_WALL_DIRS[index] == 0 {
            self.ram.write_u8(0x30d, 0);
            self.set_grab_animation()
        } else {
            let timer = self.ram.read_u8(0x30b).wrapping_sub(1);
            self.ram.write_u8(0x30b, timer);
            if (timer as i8) < 0 {
                let next = self.ram.read_u8(0x30d).wrapping_add(1);
                self.ram.write_u8(0x30d, if next == 7 { 1 } else { next });
                self.set_grab_animation()
            }
        }
        if self.ram.read_u8(0xf2) & 0x80 == 0 {
            self.ram.write_u8(0x30d, 0);
            self.ram.write_u8(0x30a, 0);
            self.ram.write_u8(0x376, 0);
            self.ram.write_u8(0x3b, 0);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1)
        }
    }

    fn set_grab_animation(&mut self) {
        let index = usize::from(self.ram.read_u8(0x30d));
        self.ram.write_u8(0x30a, GRAB_WALL_ANIM_STEPS[index]);
        self.ram.write_u8(0x30b, GRAB_WALL_ANIM_TIMER[index])
    }

    pub fn link_perform_statue_drag(&mut self) {
        self.ram.write_u8(0x376, 2);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
        self.ram.write_u8(0x2e, 0);
        self.ram.write_u8(0x30a, 0);
        self.ram.write_u8(0x30b, GRAB_WALL_ANIM_TIMER[0]);
        self.ram.write_u8(0x30d, 0)
    }

    pub fn link_a_press_statue_drag(&mut self) {
        self.ram.write_u8(0x5e, 20);
        let index = usize::from(self.ram.player().facing() >> 1);
        let direction = self.ram.read_u8(0xf0) & GRAB_WALL_DIRS[index];
        let mut set_animation = true;
        if direction == 0 {
            self.ram.write_u8(0x67, 0);
            self.ram.set_player_velocity_x(0);
            self.ram.set_player_velocity_y(0);
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x30d, 0)
        } else {
            self.ram.write_u8(0x67, direction);
            let timer = self.ram.read_u8(0x30b).wrapping_sub(1);
            self.ram.write_u8(0x30b, timer);
            if (timer as i8) >= 0 {
                set_animation = false
            } else {
                let next = self.ram.read_u8(0x30d).wrapping_add(1);
                self.ram.write_u8(0x30d, if next == 7 { 1 } else { next })
            }
        }
        if set_animation {
            self.set_grab_animation()
        }
        if self.ram.read_u8(0xf2) & 0x80 == 0 {
            for address in [0x5e, 0x2fa, 0x30d, 0x30a, 0x376, 0x3b] {
                self.ram.write_u8(address, 0)
            }
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1)
        }
    }

    pub fn link_perform_rupee_pull(&mut self) {
        if self.ram.player().facing() != 0 {
            return;
        }
        self.link_reset_properties_a();
        self.ram.write_u8(0x376, 2);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 2);
        self.ram.write_u8(0x2e, 0);
        self.ram.write_u8(0x30a, 0);
        self.ram.write_u8(0x30b, GRAB_WALL_ANIM_TIMER[0]);
        self.ram.write_u8(0x30d, 0);
        self.ram
            .set_player_action_state(PlayerState::PullForRupees as u8);
        self.ram.set_player_actual_velocity_y(0);
        self.ram.set_player_actual_velocity_x(0);
        self.ram.write_u8(0x3a, 0)
    }

    pub fn link_perform_read(&mut self) {
        let message = if self.ram.read_u8(0x1b) != 0 {
            let room = self.ram.read_u16(0xa0);
            self.runtime.dungeon_get_tele_msg(self.ram, room)
        } else if self.ram.read_u8(0xf3c5) < 2 {
            0x3a
        } else {
            let screen = self.ram.read_u16(0x8a);
            self.runtime.overworld_get_sign_text(self.ram, screen)
        };
        self.ram.write_u16(0x1cf0, message);
        self.runtime.main_show_text_message(self.ram);
        self.ram.write_u8(0x3b, 0)
    }

    pub fn link_hop_hopping_south_ow(&mut self) {
        self.ram.write_u8(0x66, 1);
        self.ram.write_u8(0x50, 0);
        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_y(0);
        self.ram.write_u8(0x351, 0);
        if self.ram.read_u8(0x46) == 0 && self.ram.read_u8(0x362) == 0 {
            self.runtime.ancilla_sfx2_near(self.ram, 32);
            self.link_hop_find_tile_to_land_on_south();
            if self.ram.read_u8(0x1b) == 0 {
                self.ram.write_u8(0xee, 2)
            }
        }
        self.ram
            .set_player_actual_velocity_z(self.ram.read_u8(0x362));
        self.ram.write_u8(0x2c7, self.ram.read_u8(0x363));
        self.ram.set_player_z(self.ram.read_u16(0x364));
        self.ram
            .set_player_actual_velocity_z(self.ram.player().actual_velocity_z().wrapping_sub(2));
        self.link_move_position();
        if (self.ram.player().actual_velocity_z() as i8) < 0 {
            if self.ram.player().actual_velocity_z() < 0xa0 {
                self.ram.set_player_actual_velocity_z(0xa0)
            }
            if self.ram.player().z() >= 0xfff0 {
                self.ram.set_player_z(0);
                self.link_splash_upon_landing();
                if self.ram.read_u8(0x5b) != 0 {
                    self.ram
                        .set_player_action_state(PlayerState::FallingIntoHole as u8)
                }
                if self.ram.player().action_state() != PlayerState::Swimming as u8
                    && self.ram.player().action_state() != PlayerState::FallingIntoHole as u8
                    && self.ram.read_u8(0x345) == 0
                {
                    self.runtime.ancilla_sfx2_near(self.ram, 33);
                }
                self.ram.write_u8(0x37b, 0);
                self.ram.write_u8(0x78, 0);
                self.ram.write_u8(0x4d, 0);
                self.ram.set_player_actual_velocity_z(0xff);
                self.ram.set_player_z(0xffff);
                self.ram.write_u8(0x46, 0);
                if self.ram.read_u8(0x1b) == 0 {
                    self.ram.write_u8(0xee, 0)
                }
            } else {
                self.ram.set_player_velocity_y(
                    self.ram.read_u16(0x364).wrapping_sub(self.ram.player().z()) as u8,
                )
            }
        } else {
            self.ram.set_player_velocity_y(
                self.ram.read_u16(0x364).wrapping_sub(self.ram.player().z()) as u8,
            )
        }
        self.ram
            .write_u8(0x362, self.ram.player().actual_velocity_z());
        self.ram.write_u8(0x363, self.ram.read_u8(0x2c7));
        self.ram.write_u16(0x364, self.ram.player().z())
    }

    pub fn link_state_crossing_worlds(&mut self) {
        self.link_reset_properties_b();
        self.runtime.tile_check_for_mirror_bonk(self.ram);
        let changed_world = (self.ram.read_u16(0x8a) as u8 & 0x40) != self.ram.read_u8(0x7b);
        let diagonal = self.ram.read_u16(0x0c) | self.ram.read_u16(0x0e);
        if changed_world && diagonal & 0xc != 0 && bit_sum4(diagonal as u8) >= 2 {
            self.link_crossing_worlds_restart_mirror();
            return;
        }
        if bit_sum4(self.ram.read_u8(0x341)) >= 2 {
            if self.ram.player().flippers() != 0 {
                self.ram.write_u8(0x345, 1);
                self.ram.write_u8(0x340, self.ram.read_u8(0x26));
                self.link_reset_swimming_state();
                self.ram
                    .set_player_action_state(PlayerState::Swimming as u8);
                self.link_force_unequip_cape_quietly();
                self.ram.write_u8(0x5e, 0);
                return;
            }
            if changed_world {
                self.link_crossing_worlds_restart_mirror();
                return;
            }
            self.check_ability_to_swim()
        }
        if self.ram.read_u8(0x345) != 0 {
            self.ram.write_u8(0x345, 0);
            self.ram.write_u8(0x26, self.ram.read_u8(0x340))
        }
        for address in [0x374, 0x372, 0x5e, 0x3a, 0x3c, 0x50, 0x27] {
            self.ram.write_u8(address, 0)
        }
        self.ram.set_swim_acceleration_mode(
            SwimAxis::Vertical,
            self.ram.swim_axis(SwimAxis::Vertical).acceleration_mode() & 0xff00,
        );
        if changed_world {
            self.ram.write_u16(0x4ac, 0)
        }
        self.ram.set_player_action_state(
            if self.ram.player().moon_pearl() != 0 || self.ram.read_u16(0x8a) & 0x40 == 0 {
                PlayerState::Ground as u8
            } else {
                PlayerState::PermaBunny as u8
            },
        )
    }

    fn link_crossing_worlds_restart_mirror(&mut self) {
        self.ram.write_u8(0x11, 44);
        self.ram.write_u8(0x3f8, 0);
        self.ram.write_u8(0x2db, 1);
        self.ram.write_u8(0xb0, 0);
        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_y(0);
        self.ram.set_player_action_state(PlayerState::Mirror as u8)
    }

    pub fn handle_followers_after_mirroring(&mut self) {
        self.tile_detect_main_handler(0);
        self.ram.write_u8(0x2e, 0);
        let follower = self.ram.read_u8(0xf3cc);
        if matches!(follower, 12 | 13) {
            if follower == 13 {
                self.ram.write_u8(0x4b4, 0xfe);
                self.ram.write_u8(0x4b5, 0)
            }
            if self.ram.read_u8(0xf3d3) != 0 {
                self.ram.write_u8(0xf3d3, 0);
                self.ram.write_u8(0xf3cc, 0)
            }
        } else if matches!(follower, 9 | 10) {
            self.ram.write_u8(0xf3cc, 0)
        } else if matches!(follower, 7 | 8) {
            self.ram.write_u8(0xf3cc, follower ^ (7 ^ 8));
            self.runtime.load_follower_graphics(self.ram);
            self.runtime.ancilla_add_dwarf_poof(self.ram, 0x40, 4)
        }
        if self.ram.player().moon_pearl() == 0 {
            self.runtime.ancilla_add_bunny_poof(self.ram, 0x23, 4);
            self.link_force_unequip_cape_quietly();
            self.ram.write_u8(0x2e2, 0)
        } else if self.ram.read_u8(0x55) != 0 {
            self.link_force_unequip_cape();
            self.ram.write_u8(0x2e2, 0)
        }
    }

    pub fn link_item_rod(&mut self) {
        const DELAYS: [u8; 3] = [3, 3, 5];
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x6c) != 0 || !self.check_y_button_press() {
                return;
            }
            if !self.link_check_magic_cost(0) {
                self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
                return;
            }
            self.ram.write_u8(0x350, 1);
            if self.ram.read_u8(0x307) == 1 {
                self.runtime.ancilla_add_fire_rod_shot(self.ram, 2, 1)
            } else {
                self.runtime.ancilla_add_ice_rod_shot(self.ram, 11, 1)
            }
            self.ram.write_u8(0x3d, DELAYS[0]);
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x301, 1)
        }
        self.halt_link_when_using_items();
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let phase = self.inc8(0x300);
        if phase != 3 {
            self.ram.write_u8(0x3d, DELAYS[usize::from(phase)]);
            return;
        }
        self.ram.write_u8(0x350, 0);
        self.ram.write_u8(0x5e, 0);
        self.ram.write_u8(0x300, 0);
        self.ram.write_u8(0x3d, 0);
        self.ram.write_u8(0x301, self.ram.read_u8(0x301) & !1);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40)
    }

    pub fn link_item_bow(&mut self) {
        const DELAYS: [u8; 3] = [3, 3, 8];
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x6c) != 0 || !self.check_y_button_press() {
                return;
            }
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
            self.ram.write_u8(0x3d, DELAYS[0]);
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x301, 16)
        }
        self.halt_link_when_using_items();
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let phase = self.inc8(0x300);
        if phase != 3 {
            self.ram.write_u8(0x3d, DELAYS[usize::from(phase)]);
            return;
        }
        let direction = self.ram.player().facing();
        let x = self.ram.player().x();
        let y = self.ram.player().y();
        if let Some(object) = self
            .runtime
            .ancilla_add_arrow(self.ram, 9, direction, 2, x, y)
        {
            if self.ram.read_u8(0xb99) != 0 {
                self.ram
                    .write_u8(0xb99, self.ram.read_u8(0xb99).wrapping_sub(1));
                self.ram
                    .write_u8(0xf377, self.ram.read_u8(0xf377).wrapping_add(2))
            }
            if self.ram.read_u8(0xb9a) == 0 && self.ram.read_u8(0xf377) != 0 {
                let arrows = self.ram.read_u8(0xf377).wrapping_sub(1);
                self.ram.write_u8(0xf377, arrows);
                if arrows == 0 {
                    self.runtime.hud_refresh_icon(self.ram)
                }
            } else {
                self.ram.write_u8(0xc4a + object, 0);
                self.runtime.ancilla_sfx2_near(self.ram, 60);
            }
        }
        self.ram.write_u8(0x300, 0);
        self.ram.write_u8(0x3d, 0);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
        self.ram.write_u8(0x301, self.ram.read_u8(0x301) & !0x10);
        if self.ram.read_u8(0x3c) >= 9 {
            self.ram.write_u8(0x3c, 9)
        }
    }

    pub fn link_item_boomerang(&mut self) {
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x6c) != 0
                || !self.check_y_button_press()
                || self.ram.read_u8(0x35f) != 0
            {
                return;
            }
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x301, 0x80);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x3d, 7);
            let result = self.runtime.ancilla_add_boomerang(self.ram, 5, 0);
            if self.ram.read_u8(0x3c) >= 9 {
                self.link_reset_boomerang_y_stuff();
                return;
            }
            if result == 0 {
                self.ram.write_u8(0x26, self.ram.read_u8(0xf0) & 0xf)
            } else {
                self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1)
            }
        } else {
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1)
        }
        if self.ram.read_u8(0x301) != 0 {
            self.halt_link_when_using_items();
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
            let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
            self.ram.write_u8(0x3d, delay);
            if (delay as i8) >= 0 {
                return;
            }
            self.ram.write_u8(0x3d, 5);
            if self.inc8(0x300) != 2 {
                return;
            }
        }
        self.link_reset_boomerang_y_stuff()
    }

    pub fn link_item_hammer(&mut self) {
        const DELAYS: [u8; 3] = [3, 3, 16];
        if self.ram.read_u8(0x301) & 0x10 != 0 {
            return;
        }
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x6c) != 0 || self.ram.read_u8(0xf4) & 0x40 == 0 {
                return;
            }
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) | 0x40);
            self.ram.write_u8(0x3d, DELAYS[0]);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x301, 2)
        }
        self.halt_link_when_using_items();
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let phase = self.inc8(0x300);
        if phase < 3 {
            self.ram.write_u8(0x3d, DELAYS[usize::from(phase)])
        }
        if phase == 1 {
            self.tile_detect_main_handler(3);
            self.runtime.ancilla_add_hit_stars(self.ram, 22, 0);
            if self.ram.read_u8(0x12e) == 0 {
                self.runtime.ancilla_sfx2_near(self.ram, 16);
                self.spawn_hammer_water_splash()
            }
        } else if phase == 3 {
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x3d, 0);
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            self.ram.write_u8(0x301, self.ram.read_u8(0x301) & !2)
        }
    }

    pub fn spawn_hammer_water_splash(&mut self) {
        const X_OFFSET: [i8; 4] = [0, 12, -8, 24];
        const Y_OFFSET: [i8; 4] = [8, 32, 24, 24];
        if self.ram.read_u8(0x11) | self.ram.read_u8(0x2e4) | self.ram.read_u8(0xfc1) != 0 {
            return;
        }
        let direction = usize::from(self.ram.player().facing() >> 1);
        let x = add_i8(self.ram.player().x(), X_OFFSET[direction]);
        let y = add_i8(self.ram.player().y(), Y_OFFSET[direction]);
        let tile = if self.ram.read_u8(0x1b) != 0 {
            let mut index = if self.ram.read_u8(0xee) >= 1 {
                0x1000
            } else {
                0
            };
            index += usize::from((x & 0x1f8) >> 3);
            index += usize::from((y & 0x1f8) << 3);
            self.ram.read_u8(0x12000 + index)
        } else {
            self.runtime
                .overworld_read_tile_attribute(self.ram, x >> 3, y)
        };
        if matches!(tile, 8 | 9)
            && let Some(slot) = self.runtime.sprite_spawn_small_splash(self.ram, 0)
        {
            self.runtime.sprite_set_x(self.ram, slot, x.wrapping_sub(8));
            self.runtime
                .sprite_set_y(self.ram, slot, y.wrapping_sub(16));
            self.ram.write_u8(0xf20 + slot, self.ram.read_u8(0xee));
            self.ram.write_u8(0xf70 + slot, 0)
        }
    }

    fn begin_medallion_cast(&mut self) -> bool {
        if self.ram.read_u8(0x6c) != 0
            || self.ram.read_u8(0xffc) != 0
            || self.ram.read_u16(0x402) & 0x8000 != 0
            || self.ram.player().sword().wrapping_add(1) & !1 == 0
            || (self.ram.read_u8(0xf3d3) != 0 && self.ram.read_u8(0xf3cc) == 13)
        {
            self.runtime.ancilla_sfx2_near(self.ram, 60);
            return false;
        }
        if self.ram.read_u8(0xc4a) | self.ram.read_u8(0xc4b) | self.ram.read_u8(0xc4c) != 0 {
            return false;
        }
        self.link_check_magic_cost(1)
    }

    pub fn link_item_ether(&mut self) {
        if !self.check_y_button_press() {
            return;
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        if !self.begin_medallion_cast() {
            return;
        }
        self.ram.set_player_action_state(PlayerState::Ether as u8);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
        self.ram.write_u8(0x3d, ETHER_ANIM_DELAYS[0]);
        self.ram.write_u8(0x31c, 0);
        self.ram.write_u8(0x31d, 0);
        self.ram.write_u8(0x324, 0);
        self.runtime.ancilla_sfx3_near(self.ram, 35)
    }

    pub fn link_state_using_ether(&mut self) {
        self.inc8(0xfc1);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let mut step = self.inc8(0x31d);
        match step {
            4 => self.runtime.ancilla_sfx3_near(self.ram, 35),
            9 => {
                self.runtime.ancilla_sfx2_near(self.ram, 44);
            }
            12 => {
                step = 10;
                self.ram.write_u8(0x31d, step)
            }
            _ => {}
        }
        let delays = if self.runtime.enhanced_features0() & 65536 != 0 {
            &ETHER_ANIM_DELAYS_NO_FLASH
        } else {
            &ETHER_ANIM_DELAYS
        };
        self.ram.write_u8(0x3d, delays[usize::from(step)]);
        self.ram
            .write_u8(0x31c, ETHER_ANIM_STATES[usize::from(step)]);
        if self.ram.read_u8(0x324) == 0 && step == 10 {
            self.ram.write_u8(0x324, 1);
            self.runtime.ancilla_add_ether_spell(self.ram, 24, 0);
            self.ram.write_u8(0x4d, 0);
            self.ram.write_u8(0x46, 0)
        }
    }

    pub fn link_item_bombos(&mut self) {
        if !self.check_y_button_press() {
            return;
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        if !self.begin_medallion_cast() {
            return;
        }
        self.ram.set_player_action_state(PlayerState::Bombos as u8);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
        self.ram.write_u8(0x3d, BOMBOS_ANIM_DELAYS[0]);
        self.ram.write_u8(0x31c, BOMBOS_ANIM_STATES[0]);
        self.ram.write_u8(0x31d, 0);
        self.ram.write_u8(0x324, 0);
        self.runtime.ancilla_sfx3_near(self.ram, 35)
    }

    pub fn link_state_using_bombos(&mut self) {
        self.inc8(0xfc1);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let mut step = self.inc8(0x31d);
        match step {
            4 => self.runtime.ancilla_sfx3_near(self.ram, 35),
            10 => {
                self.runtime.ancilla_sfx2_near(self.ram, 44);
            }
            20 => {
                step = 19;
                self.ram.write_u8(0x31d, step)
            }
            _ => {}
        }
        self.ram
            .write_u8(0x3d, BOMBOS_ANIM_DELAYS[usize::from(step)]);
        self.ram
            .write_u8(0x31c, BOMBOS_ANIM_STATES[usize::from(step)]);
        if self.ram.read_u8(0x324) == 0 && step == 19 {
            self.ram.write_u8(0x324, 1);
            self.runtime.ancilla_add_bombos_spell(self.ram, 25, 0);
            self.ram.write_u8(0x4d, 0);
            self.ram.write_u8(0x46, 0)
        }
    }

    pub fn link_item_quake(&mut self) {
        if !self.check_y_button_press() {
            return;
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        if !self.begin_medallion_cast() {
            return;
        }
        self.ram.set_player_action_state(PlayerState::Quake as u8);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
        self.ram.write_u8(0x3d, QUAKE_ANIM_DELAYS[0]);
        self.ram.write_u8(0x31c, QUAKE_ANIM_STATES[0]);
        self.ram.write_u8(0x31d, 0);
        self.ram.write_u8(0x324, 0);
        self.ram.write_u8(0x362, 40);
        self.ram.write_u8(0x363, 40);
        self.ram.write_u8(0x364, 0);
        self.runtime.ancilla_sfx3_near(self.ram, 35)
    }

    pub fn link_state_using_quake(&mut self) {
        self.inc8(0xfc1);
        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_y(0);
        let mut step = self.ram.read_u8(0x31d);
        if step == 10 {
            self.ram
                .set_player_actual_velocity_z(self.ram.read_u8(0x362));
            self.ram.write_u8(0x2c7, self.ram.read_u8(0x363));
            self.ram.set_player_z_low(self.ram.read_u8(0x364));
            self.ram.write_u8(0x4d, 2);
            self.player_change_z(2);
            self.link_move_position();
            self.ram
                .write_u8(0x362, self.ram.player().actual_velocity_z());
            self.ram.write_u8(0x363, self.ram.read_u8(0x2c7));
            self.ram.write_u8(0x364, self.ram.player().z_low());
            if (self.ram.player().z_low() as i8) >= 0 {
                self.ram.write_u8(
                    0x31c,
                    if (self.ram.player().actual_velocity_z() as i8) < 0 {
                        21
                    } else {
                        20
                    },
                );
                return;
            }
        } else {
            let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
            self.ram.write_u8(0x3d, delay);
            if (delay as i8) >= 0 {
                return;
            }
        }
        step = self.inc8(0x31d);
        match step {
            4 => self.runtime.ancilla_sfx3_near(self.ram, 35),
            10 => {
                self.runtime.ancilla_sfx2_near(self.ram, 44);
            }
            11 => {
                self.runtime.ancilla_sfx2_near(self.ram, 12);
            }
            12 => {
                step = 11;
                self.ram.write_u8(0x31d, step)
            }
            _ => {}
        }
        self.ram
            .write_u8(0x3d, QUAKE_ANIM_DELAYS[usize::from(step)]);
        self.ram
            .write_u8(0x31c, QUAKE_ANIM_STATES[usize::from(step)]);
        if self.ram.read_u8(0x324) == 0 && step == 11 {
            self.ram.write_u8(0x324, 1);
            self.runtime.ancilla_add_quake_spell(self.ram, 28, 0);
            self.ram.write_u8(0x4d, 0);
            self.ram.write_u8(0x46, 0)
        }
    }

    pub fn link_item_mirror(&mut self) {
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if !self.check_y_button_press() {
                return;
            }
            if self.ram.read_u8(0xf3cc) == 10 {
                self.ram.write_u16(0x1cf0, 289);
                self.runtime.main_show_text_message(self.ram);
                return;
            }
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        if self.ram.read_u8(0x6c) != 0
            || (self.ram.read_u8(0x37f) == 0
                && self.runtime.enhanced_features0() & 8 == 0
                && self.ram.read_u8(0x1b) == 0
                && self.ram.read_u16(0x8a) & 0x40 == 0)
        {
            self.runtime.ancilla_sfx2_near(self.ram, 60);
            return;
        }
        self.do_sword_interaction_with_tiles_mirror()
    }

    pub fn link_item_hookshot(&mut self) {
        if self.ram.read_u8(0x3a) & 0x40 != 0
            || self.ram.read_u8(0x6c) != 0
            || self.ram.read_u8(0x48) & 2 != 0
            || !self.check_y_button_press()
        {
            return;
        }
        self.reset_all_acceleration();
        self.ram.write_u8(0x300, 0);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
        self.ram.write_u8(0x3d, 7);
        self.ram.write_u8(0x2e, 0);
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        self.ram.write_u8(0x37a, 4);
        self.ram
            .set_player_action_state(PlayerState::Hookshot as u8);
        self.ram.write_u8(0x37b, 1);
        self.ancilla_add_hookshot(31, 3)
    }

    pub fn link_item_cane_of_somaria(&mut self) {
        const DELAYS: [u8; 3] = [3, 3, 5];
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x2f5) != 0
                || self.ram.read_u8(0x6c) != 0
                || !self.check_y_button_press()
            {
                return;
            }
            let existing = (0..5).rev().any(|i| self.ram.read_u8(0xc4a + i) == 0x2c);
            let mut charged = false;
            if !existing {
                if !self.link_check_magic_cost(4) {
                    if self.runtime.enhanced_features0() & 4096 != 0 {
                        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40)
                    }
                    return;
                }
                charged = true
            }
            self.ram.write_u8(0x350, 1);
            if !self.runtime.ancilla_add_somaria_block(self.ram, 0x2c, 1)
                && (charged || self.runtime.enhanced_features0() & 4096 == 0)
            {
                self.refund_magic(4)
            }
            self.ram.write_u8(0x3d, DELAYS[0]);
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x301, 0);
            self.ram.write_u8(0x37a, self.ram.read_u8(0x37a) | 8)
        }
        self.halt_link_when_using_items();
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let phase = self.inc8(0x300);
        if phase != 3 {
            self.ram.write_u8(0x3d, DELAYS[usize::from(phase)]);
            return;
        }
        self.ram.write_u8(0x5e, 0);
        self.ram.write_u8(0x300, 0);
        self.ram.write_u8(0x3d, 0);
        self.ram.write_u8(0x350, 0);
        self.ram.write_u8(0x37a, self.ram.read_u8(0x37a) & !8);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40)
    }

    pub fn link_item_cane_of_byrna(&mut self) {
        const DELAYS: [u8; 4] = [19, 7, 13, 32];
        if self.search_for_byrna_spark() {
            return;
        }
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x6c) != 0 || !self.check_y_button_press() {
                return;
            }
            if !self.link_check_magic_cost(8) {
                self.link_item_byrna_finish();
                return;
            }
            self.ancilla_add_cane_of_byrna_init_spark(48, 0);
            self.ram.write_u8(0x79, 0);
            self.ram.write_u8(0x3d, DELAYS[0]);
            self.ram.write_u8(0x30d, 0);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x37a, 8);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
            self.ram.write_u8(0x2e, 0)
        }
        self.halt_link_when_using_items();
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let phase = self.inc8(0x300);
        self.ram.write_u8(0x3d, DELAYS[usize::from(phase)]);
        if phase == 1 {
            self.runtime.ancilla_sfx3_near(self.ram, 42)
        } else if phase == 3 {
            self.link_item_byrna_finish()
        }
    }

    fn link_item_byrna_finish(&mut self) {
        self.ram.write_u8(0x30d, 0);
        self.ram.write_u8(0x300, 0);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & 0x80);
        self.ram.write_u8(0x37a, 0);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1)
    }

    pub fn search_for_byrna_spark(&self) -> bool {
        if self.ram.read_u8(0x37a) & 8 != 0 {
            return false;
        }
        (0..5).rev().any(|i| self.ram.read_u8(0xc4a + i) == 0x31)
    }

    pub fn link_item_net(&mut self) {
        const TIMERS: [u8; 40] = [
            11, 6, 7, 8, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 9, 4, 5, 6, 7, 8, 1, 2, 3,
            4, 10, 8, 1, 2, 3, 4, 5, 6, 7, 8,
        ];
        let base = usize::from(self.ram.player().facing() >> 1) * 10;
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x6c) != 0 || !self.check_y_button_press() {
                return;
            }
            self.ram.write_u8(0x300, TIMERS[base]);
            self.ram.write_u8(0x3d, 3);
            self.ram.write_u8(0x30d, 0);
            self.ram.write_u8(0x37a, 16);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
            self.ram.write_u8(0x2e, 0);
            self.runtime.ancilla_sfx2_near(self.ram, 50);
        }
        self.halt_link_when_using_items();
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let phase = self.inc8(0x30d);
        self.ram.write_u8(0x3d, 3);
        // C reads one byte past kBugNetTimers for the final facing/phase,
        // then immediately clears this timer below; that transient read is unobservable.
        if phase != 10 {
            self.ram.write_u8(0x300, TIMERS[base + usize::from(phase)]);
        }
        if phase == 10 {
            self.ram.write_u8(0x30d, 0);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & 0x80);
            self.ram.write_u8(0x37a, 0);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            self.ram.write_u8(0x45, 0x80);
            self.ram.write_u8(0x44, 0x80)
        }
    }

    pub fn link_item_shovel_and_flute(&mut self) {
        match self.ram.read_u8(0xf34c) {
            1 => self.link_item_shovel(),
            2..=u8::MAX => self.link_item_flute(),
            _ => {}
        }
    }

    pub fn link_item_shovel(&mut self) {
        const DELAYS: [u8; 6] = [7, 18, 16, 7, 18, 16];
        const POSES: [u8; 6] = [0, 1, 2, 0, 1, 2];
        if self.ram.read_u8(0x3a) & 0x40 == 0 {
            if self.ram.read_u8(0x6c) != 0 || !self.check_y_button_press() {
                return;
            }
            self.ram.write_u8(0x3d, DELAYS[0]);
            self.ram.write_u8(0x30d, 0);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x37a, 1);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
            self.ram.write_u8(0x2e, 0);
        }
        self.halt_link_when_using_items();
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let timer = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, timer);
        if (timer as i8) >= 0 {
            return;
        }
        let phase = self.inc8(0x30d);
        self.ram.write_u8(0x3d, DELAYS[usize::from(phase)]);
        self.ram.write_u8(0x300, POSES[usize::from(phase)]);
        if self.ram.read_u8(0x300) == 1 {
            self.tile_detect_main_handler(2);
            if self.ram.read_u8(0x4b2) != 0 {
                self.runtime.ancilla_sfx3_near(self.ram, 27);
                self.ancilla_add_dug_up_flute(54, 0);
            }
            if (self.ram.read_u16(0x357) | self.ram.read_u16(0x35b)) & 1 == 0 {
                self.runtime.ancilla_add_hit_stars(self.ram, 22, 0);
                self.runtime.ancilla_sfx2_near(self.ram, 5);
            } else {
                self.ancilla_add_shovel_dirt(23, 0);
                if self.ram.read_u8(0x3fc) != 0 {
                    self.digging_game_guy_attempt_prize_spawn();
                }
                self.runtime.ancilla_sfx2_near(self.ram, 18);
            }
        }
        if phase == 3 {
            self.ram.write_u8(0x30d, 0);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & 0x80);
            self.ram.write_u8(0x37a, 0);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
        }
    }

    pub fn digging_game_guy_attempt_prize_spawn(&mut self) {
        const X_VEL: [i8; 2] = [-16, 16];
        const X_OFF: [i8; 2] = [0, 19];
        const ITEMS: [u8; 4] = [0xdb, 0xda, 0xd9, 0xdf];
        self.ram
            .write_u8(0x1fe01, self.ram.read_u8(0x1fe01).wrapping_add(1));
        if self.ram.player().y() >= 0xb18 {
            return;
        }
        let roll = self.runtime.random_number(self.ram) & 7;
        let item = if roll < 4 {
            ITEMS[usize::from(roll)]
        } else if roll == 4 {
            if self.ram.read_u8(0x1fe01) < 25
                || self.ram.read_u8(0x1fe00) != 0
                || self.runtime.random_number(self.ram) & 3 != 0
            {
                return;
            }
            self.ram.write_u8(0x1fe00, 0xeb);
            0xeb
        } else {
            return;
        };
        let Some(slot) = self.runtime.sprite_spawn_dynamically(self.ram, 4, item) else {
            return;
        };
        let side = usize::from(self.ram.player().facing() != 4);
        self.ram.write_u8(0xd50 + slot, X_VEL[side] as u8);
        self.ram.write_u8(0xd40 + slot, 0);
        self.ram.write_u8(0xf80 + slot, 24);
        self.ram.write_u8(0xb58 + slot, 255);
        self.ram.write_u8(0xf10 + slot, 48);
        let x = add_i8(self.ram.player().x(), X_OFF[side]) & !0xf;
        let y = self.ram.player().y().wrapping_add(22) & !0xf;
        self.runtime.sprite_set_x(self.ram, slot, x);
        self.runtime.sprite_set_y(self.ram, slot, y);
        self.ram.write_u8(0xf20 + slot, 0);
        self.runtime.sprite_sfx3_with_pan(self.ram, slot, 0x30);
    }

    pub fn link_handle_y_item(&mut self) {
        if (1..9).contains(&self.ram.read_u8(0x3c)) {
            return;
        }
        let mut item = self.ram.read_u8(0x303);
        if self.ram.read_u8(0x2e0) != 0 && !matches!(item, 11 | 20) {
            return;
        }
        if self.ram.read_u8(0x3fc) != 0 && self.ram.read_u8(0x2e0) == 0 {
            if self.ram.read_u8(0x3fc) == 2 {
                self.link_item_bow()
            } else {
                self.link_item_shovel()
            }
            return;
        }
        let old_down = self.ram.read_u8(0xf0);
        let old_pressed = self.ram.read_u8(0xf4);
        let old_bottle = self.ram.read_u8(0xf34f);
        if self.ram.read_u8(0x301) | self.ram.read_u8(0x37a) == 0
            && old_down & 0x40 == 0
            && let Some((override_item, bottle, pressed)) =
                self.runtime.current_item_button_override(self.ram)
        {
            item = override_item;
            if let Some(index) = bottle {
                self.ram.write_u8(0xf34f, index);
            }
            self.ram.write_u8(0xf0, old_down | 0x40);
            self.ram
                .write_u8(0xf4, old_pressed | if pressed { 0x40 } else { 0 });
        }
        if item != self.ram.read_u8(0x304) {
            if self.ram.read_u8(0x304) == 8 && self.ram.read_u8(0xf34c) & 2 != 0 {
                self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
            }
            if self.ram.read_u8(0x304) == 19 && self.ram.read_u8(0x55) != 0 {
                self.link_force_unequip_cape();
            }
        }
        if self.ram.read_u8(0x301) | self.ram.read_u8(0x37a) == 0 {
            self.ram.write_u8(0x304, item);
        }
        let active = self.ram.read_u8(0x304);
        if matches!(active, 5 | 6) {
            self.ram.write_u8(0x307, active - 4);
        }
        match active {
            0 => {}
            1 => self.link_item_bombs(),
            2 => self.link_item_boomerang(),
            3 => self.link_item_bow(),
            4 => self.link_item_hammer(),
            5 | 6 => self.link_item_rod(),
            7 => self.link_item_net(),
            8 => self.link_item_shovel_and_flute(),
            9 => self.link_item_lamp(),
            10 => self.link_item_powder(),
            11 => self.link_item_bottle(),
            12 => self.link_item_book(),
            13 => self.link_item_cane_of_byrna(),
            14 => self.link_item_hookshot(),
            15 => self.link_item_bombos(),
            16 => self.link_item_ether(),
            17 => self.link_item_quake(),
            18 => self.link_item_cane_of_somaria(),
            19 => self.link_item_cape(),
            20 => self.link_item_mirror(),
            21 => self.link_item_shovel(),
            _ => panic!("invalid active item"),
        }
        self.ram.write_u8(0xf0, old_down);
        self.ram.write_u8(0xf4, old_pressed);
        self.ram.write_u8(0xf34f, old_bottle);
    }
    pub fn handle_sword_sfx_and_beam(&mut self) {
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        self.ram.write_u8(0x3c, 0);
        self.ram.write_u8(0x79, 0);
        let sword = self.ram.player().sword();
        if self.ram.player().max_health().wrapping_sub(4) < self.ram.player().health()
            && sword.wrapping_add(1) & 0xfe != 0
            && sword >= 2
            && !(0..5).rev().any(|i| self.ram.read_u8(0xc4a + i) == 0x31)
        {
            self.runtime.add_sword_beam(self.ram, 0);
        }
        let index = sword.wrapping_sub(1);
        if !matches!(index, 0xfe | 0xff) {
            let pan = self.runtime.calculate_sfx_pan(self.ram);
            self.ram
                .write_u8(0x12e, FIRE_BEAM_SOUNDS[usize::from(index)] | pan);
        }
        self.ram.write_u8(0x3d, 1);
    }

    pub fn link_handle_sword_cooldown(&mut self) {
        let delay = self.ram.read_u8(0x2e3).wrapping_sub(1);
        self.ram.write_u8(0x2e3, delay);
        if (delay as i8) >= 0 {
            return;
        }
        self.ram.write_u8(0x2e3, 0);
        if self.ram.read_u8(0x301) | self.ram.read_u8(0x37a) != 0 {
            return;
        }
        if self.ram.read_u8(0x3c) < 9 {
            if self.ram.read_u8(0x372) == 0 {
                self.link_check_for_sword_swing();
            }
        } else {
            self.handle_sword_controls();
        }
    }

    pub fn link_check_for_sword_swing(&mut self) {
        if self.ram.read_u8(0x3b) & 0x10 != 0 {
            return;
        }
        if self.ram.read_u8(0x3a) & 0x80 == 0 {
            if self.ram.read_u8(0xf4) & 0x80 == 0 {
                return;
            }
            let doorway = self.ram.read_u8(0x6c);
            if doorway != 0 {
                self.runtime
                    .tile_detect_sword_swing_deep_in_door(self.ram, doorway);
                if self.ram.read_u16(14) & 0x30 == 0x30 {
                    return;
                }
            }
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) | 0x80);
            self.handle_sword_sfx_and_beam();
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
            self.ram.write_u8(0x2e, 0);
        }
        if self.ram.read_u8(0xf0) & 0x80 == 0 {
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) | 1);
        }
        self.halt_link_when_using_items();
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) < 0 {
            let frame = self.inc8(0x3c);
            if frame >= 9 {
                self.handle_sword_controls();
                return;
            }
            self.ram
                .write_u8(0x3d, SPIN_ATTACK_DELAYS[usize::from(frame)]);
            if frame == 5 {
                let sword = self.ram.player().sword();
                if !matches!(sword, 0 | 1 | 0xff) {
                    self.runtime
                        .ancilla_add_sword_swing_sparkle(self.ram, 0x26, 4);
                }
                if !matches!(sword, 0 | 0xff) {
                    self.tile_detect_main_handler(if sword == 1 { 1 } else { 6 });
                }
            } else if frame >= 4
                && self.ram.read_u8(0x3a) & 1 != 0
                && self.ram.read_u8(0xf0) & 0x80 != 0
            {
                self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !1);
                self.handle_sword_sfx_and_beam();
                return;
            }
        }
        self.runtime.calculate_sword_hit_box(self.ram);
    }

    pub fn handle_sword_controls(&mut self) {
        if self.ram.read_u8(0xf0) & 0x80 != 0 {
            self.player_sword_spin_attack_jerks_hold_down();
        } else if self.ram.read_u8(0x79) < 48 {
            self.link_reset_sword_and_item_usage();
        } else {
            self.link_reset_sword_and_item_usage();
            self.ram.write_u8(0x79, 0);
            self.link_activate_spin_attack();
        }
    }

    pub fn link_activate_spin_attack(&mut self) {
        self.runtime
            .ancilla_add_spin_attack_init_spark(self.ram, 42, 0);
        self.link_animate_victory_spin();
    }

    pub fn link_animate_victory_spin(&mut self) {
        self.ram
            .set_player_action_state(PlayerState::SpinAttacking as u8);
        let offset = (self.ram.player().facing() >> 1).wrapping_mul(12);
        self.ram.write_u8(0x31e, offset);
        self.ram.write_u8(0x3d, 3);
        self.ram
            .write_u8(0x31c, LINK_SPIN_GRAPHICS_BY_DIR[usize::from(offset)]);
        self.ram.write_u8(0x31d, 0);
        self.ram.write_u8(0x3c, 144);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
        self.ram.write_u8(0x3a, 0x80);
        self.link_state_spin_attack();
    }

    pub fn link_state_spin_attack(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.ram.read_u8(0x4d) != 0 {
            for i in (0..5).rev() {
                if matches!(self.ram.read_u8(0xc4a + i), 0x2a | 0x2b) {
                    self.ram.write_u8(0xc4a + i, 0);
                }
            }
            self.ram.set_player_z(self.ram.player().z() & 0xff);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            for address in [0x3d, 0x3c, 0x3a, 0x3b, 0x31c, 0x31d, 0x5e] {
                self.ram.write_u8(address, 0);
            }
            if self.ram.read_u8(0x360) != 0 {
                if self.ram.read_u8(0x55) != 0 {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.ram.write_u8(0x37b, 1);
                self.ram.write_u8(0x300, 0);
                self.ram.write_u8(0x3d, 2);
                self.ram.write_u8(0x2e, 0);
                self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
                self.runtime.ancilla_sfx3_near(self.ram, 43);
                self.ram
                    .set_player_action_state(PlayerState::Electrocution as u8);
                self.link_state_zapped();
            } else {
                self.ram
                    .set_player_action_state(PlayerState::RecoilWall as u8);
                self.link_state_recoil();
            }
            return;
        }
        if self.ram.read_u8(0x46) != 0 {
            self.link_handle_recoil_and_timer(false);
        } else {
            self.ram.write_u8(0x67, 0);
            self.link_handle_velocity();
            self.link_handle_cardinal_collision();
            self.ram
                .set_player_action_state(PlayerState::SpinAttacking as u8);
            self.ram.write_u8(0x302, 0);
            self.handle_indoor_camera_and_doors();
        }
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        let step = self.inc8(0x31d);
        if step == 2 {
            self.runtime.ancilla_sfx3_near(self.ram, 35);
        }
        if step == 12 {
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            for address in [0x3d, 0x3c, 0x31c, 0x31d] {
                self.ram.write_u8(address, 0);
            }
            if self.ram.player().action_state() != PlayerState::SpinAttackMotion as u8 {
                self.ram.write_u8(0x3a, 0);
            }
            self.ram.set_player_action_state(PlayerState::Ground as u8);
        } else {
            let index = usize::from(step.wrapping_add(self.ram.read_u8(0x31e)));
            self.ram.write_u8(0x31c, LINK_SPIN_GRAPHICS_BY_DIR[index]);
            self.ram.write_u8(0x3d, LINK_SPIN_DELAYS[usize::from(step)]);
            self.tile_detect_main_handler(8);
        }
    }

    pub fn player_sword_spin_attack_jerks_hold_down(&mut self) {
        if (self.ram.read_u8(0x48) & 0x80 != 0 || self.ram.read_u8(0x48) & 9 == 0)
            && self.ram.read_u8(0x47) == 0
        {
            self.ram.write_u8(0x3c, 9);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
            self.ram.write_u8(0x3d, 0);
            if !matches!(self.ram.read_u8(0x5e), 4 | 16) {
                self.ram.write_u8(0x5e, 12);
                if self.ram.player().sword().wrapping_add(1) & !1 == 0 {
                    return;
                }
                if (0..5)
                    .rev()
                    .any(|i| matches!(self.ram.read_u8(0xc4a + i), 0x30 | 0x31))
                {
                    return;
                }
                if self.ram.read_u8(0x79) >= 6 && self.ram.read_u8(0x1a) & 3 == 0 {
                    self.runtime.ancilla_spawn_sword_charge_sparkle(self.ram)
                }
                if self.ram.read_u8(0x79) < 64 && self.inc8(0x79) == 48 {
                    self.runtime.ancilla_sfx2_near(self.ram, 55);
                    self.runtime
                        .ancilla_add_charged_spin_attack_sparkle(self.ram)
                }
            } else {
                self.runtime.calculate_sword_hit_box(self.ram)
            }
            return;
        } else if self.ram.read_u8(0x47) == 1 {
            self.link_reset_sword_and_item_usage();
            return;
        }
        if self.ram.read_u8(0x3c) == 9 {
            self.ram.write_u8(0x3c, 10);
            self.ram.write_u8(0x3d, SPIN_ATTACK_DELAYS[10])
        }
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) < 0 {
            let mut frame = self.ram.read_u8(0x3c).wrapping_add(1);
            if frame == 13 {
                if self.ram.player().sword().wrapping_add(1) & !1 != 0
                    && self.ram.read_u8(0x48) & 9 != 0
                {
                    self.runtime.ancilla_add_wall_tap_spark(self.ram, 27, 1);
                    let sound = if self.ram.read_u8(0x48) & 8 != 0 {
                        6
                    } else {
                        5
                    };
                    self.runtime.ancilla_sfx2_near(self.ram, sound);
                    self.tile_detect_main_handler(1)
                }
                frame = 10
            }
            self.ram.write_u8(0x3c, frame);
            self.ram
                .write_u8(0x3d, SPIN_ATTACK_DELAYS[usize::from(frame)])
        }
        self.runtime.calculate_sword_hit_box(self.ram)
    }

    pub fn link_handle_liftables(&mut self) -> u8 {
        const REQUIRED_GLOVES: [u8; 7] = [0, 1, 0, 0, 2, 1, 2];
        const TILE_KIND: [u8; 7] = [2, 3, 1, 4, 0, 5, 6];
        self.ram.write_u16(0x59, 0);
        self.runtime.tile_detect_reset_state(self.ram);
        let direction = usize::from(self.ram.player().facing() >> 1);
        let (x_offset, y_offset) = self.runtime.liftable_probe_offset(direction);
        let mask = self.ram.read_u16(0xec);
        let y0 = add_i8(self.ram.player().y(), y_offset) & mask;
        let y1 = self.ram.player().y().wrapping_add(20) & mask;
        let x0 = (add_i8(self.ram.player().x(), x_offset) & mask) >> 3;
        let x1 = (self.ram.player().x().wrapping_add(8) & mask) >> 3;
        self.runtime.tile_detection_execute(self.ram, x0, y0, 1);
        self.runtime.tile_detection_execute(self.ram, x1, y1, 2);
        let mut action = if (self.ram.read_u16(0x0e) | u16::from(self.ram.read_u8(0x36d))) & 1 != 0
        {
            3
        } else {
            2
        };
        if self.ram.read_u8(0x1b) != 0 {
            let tile = self.runtime.dungeon_check_for_liftable_tile(self.ram);
            if tile != 0xff {
                self.ram.write_u8(0x368, TILE_KIND[usize::from(tile & 0xf)]);
            } else {
                if self.ram.read_u16(0x366) & 1 != 0
                    && self.ram.player().facing() == 0
                    && self.ram.read_u8(0x36a) == 0
                {
                    action = 4;
                }
                if self.ram.read_u16(0x2e5) & 1 != 0 {
                    action = 5;
                }
                return action;
            }
        } else {
            if self.ram.read_u16(0x366) & 1 == 0 {
                if self.ram.read_u16(0x2e5) & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            if self.ram.player().facing() == 0 && self.ram.read_u8(0x36a) == 0 {
                action = 4;
                if self.ram.read_u16(0x2e5) & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            self.ram.write_u8(0x368, self.ram.read_u8(0x36a) >> 1);
        }
        let kind = usize::from(self.ram.read_u8(0x368));
        if REQUIRED_GLOVES[kind] <= self.ram.player().gloves() {
            action = 1;
        }
        if self.ram.read_u16(0x2e5) & 1 != 0 {
            action = 5;
        }
        action
    }

    pub fn link_perform_open_chest(&mut self) {
        if self.ram.player().facing() != 0
            || self.ram.read_u8(0x2e9) != 0
            || self.ram.read_u8(0x4d) != 0
        {
            return;
        }
        self.ram.write_u8(0x3b, 0);
        let tile = self.ram.read_u8(0x76);
        let Some((mut item, chest_position)) = self.runtime.open_chest(self.ram, tile) else {
            self.ram.write_u8(0x2e9, 0);
            return;
        };
        self.ram.write_u8(0x2e9, 1);
        let alternate = match item {
            12 => 68,
            18 => 53,
            42 => 70,
            _ => 0xff,
        };
        if alternate != 0xff {
            let address = self.runtime.item_memory_location(item);
            if self.ram.read_u8(address) != 0 {
                item = alternate;
            }
        }
        self.link_receive_item(item, chest_position)
    }

    pub fn link_perform_throw(&mut self) {
        const LIFT_TILES: [u8; 9] = [0x54, 0x52, 0x50, 0xff, 0x51, 0x53, 0x55, 0x56, 0x57];
        if self.ram.read_u8(0x314) | self.ram.read_u8(0x2ec) == 0 {
            self.link_reset_sword_and_item_usage();
            self.ram.write_u8(0x3b, 0);
            if !(0..16).rev().any(|i| self.ram.read_u8(0xdd0 + i) == 0) {
                return;
            }
            if matches!(self.ram.read_u8(0x368), 5 | 6) {
                self.ram.write_u8(0x300, 1)
            } else {
                let indoors = self.ram.read_u8(0x1b) != 0;
                let (attribute, x, y) = self.runtime.lift_and_replace_tile(self.ram, indoors);
                let Some(kind) = LIFT_TILES.iter().rposition(|&v| v == attribute) else {
                    return;
                };
                self.ram.write_u8(0x314, 1);
                self.runtime
                    .sprite_spawn_throwable_terrain(self.ram, kind, x, y);
                self.ram.write_u8(0xf6, self.ram.read_u8(0xf6) & !0x80);
                self.ram.write_u8(0x300, 0)
            }
        } else {
            self.ram.write_u8(0x300, 0)
        }
        self.ram.write_u8(0x3a, 0);
        self.ram.write_u8(0x30b, 6);
        self.ram.write_u8(0x309, 1);
        self.ram.write_u8(0x308, 0x80);
        self.ram.write_u8(0x30a, 0);
        self.ram.write_u8(0x5e, 12);
        self.ram.write_u8(0x2e, 0);
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & 0xf0);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1)
    }

    pub fn link_a_press_perform_basic(&mut self, action_x2: u8) {
        match action_x2 >> 1 {
            0 => self.link_perform_desert_prayer(),
            1 => self.link_perform_throw(),
            2 => self.link_perform_dash(),
            3 => self.link_perform_grab(),
            4 => self.link_perform_read(),
            5 => self.link_perform_open_chest(),
            6 => self.link_perform_statue_drag(),
            7 => self.link_perform_rupee_pull(),
            _ => panic!("invalid A-button action"),
        }
    }

    pub fn link_a_press_lift_carry_throw(&mut self) {
        const TIMER: [u8; 10] = [8, 24, 8, 24, 8, 32, 6, 8, 13, 13];
        const POSE: [u8; 10] = [0, 1, 0, 1, 0, 1, 0, 1, 2, 3];
        const CARRY_TIMER: [u8; 29] = [
            6, 7, 7, 5, 10, 0, 23, 0, 18, 0, 18, 0, 8, 0, 8, 0, 254, 255, 17, 0, 0x54, 0x52, 0x50,
            0xff, 0x51, 0x53, 0x55, 0x56, 0x57,
        ];
        if self.ram.read_u8(0x308) == 0 {
            return;
        }
        if self.ram.read_u8(0x309) & 2 != 0 && self.ram.read_u8(0x30b) >= 5 {
            self.ram.write_u8(0x30b, 5);
        }
        if self.ram.read_u8(0x309) != 0 {
            self.halt_link_when_using_items();
        }
        if self.ram.read_u8(0x309) & 1 != 0 {
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x2d, 0);
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        }
        let timer = self.ram.read_u8(0x30b).wrapping_sub(1);
        self.ram.write_u8(0x30b, timer);
        if timer != 0 {
            return;
        }
        if self.ram.read_u8(0x309) & 2 != 0 {
            self.ram.write_u8(0x308, 0);
            self.ram.write_u8(0x48, 0);
            self.ram.write_u8(0x5e, 0);
            if self.ram.player().action_state() == 24 {
                self.ram.set_player_action_state(0);
            }
        } else if self.ram.read_u8(0x300) != 0 {
            let phase = self.ram.read_u8(0x300);
            if phase.wrapping_add(1) != 9 {
                let phase = phase.wrapping_add(1);
                self.ram.write_u8(0x300, phase);
                self.ram.write_u8(0x30b, TIMER[usize::from(phase)]);
                self.ram.write_u8(0x30a, POSE[usize::from(phase)]);
                if phase == 6 {
                    self.ram.write_u8(0xb9c, 0);
                    let indoors = self.ram.read_u8(0x1b) != 0;
                    let (kind, x, y) = self.runtime.lift_and_replace_tile(self.ram, indoors);
                    self.ram.set_player_action_state(24);
                    self.ram.write_u8(0x314, 1);
                    self.runtime.sprite_spawn_throwable_terrain(
                        self.ram,
                        usize::from((kind & 0xf) + 1),
                        x,
                        y,
                    );
                    self.ram.write_u8(0xf6, self.ram.read_u8(0xf6) & !0x80);
                }
                return;
            }
        } else {
            let step = self.ram.read_u8(0x30a);
            if usize::from(step) >= CARRY_TIMER.len() - 1 {
                return;
            }
            let next = step.wrapping_add(1);
            self.ram.write_u8(0x30a, next);
            self.ram.write_u8(0x30b, CARRY_TIMER[usize::from(next)]);
            if next != 3 {
                return;
            }
        }
        self.ram.write_u8(0x309, 0);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
    }

    pub fn link_handle_a_press(&mut self) {
        const ABILITY: [u8; 8] = [0xe0, 0x40, 4, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0];
        self.ram.write_u8(0x2f4, 0);
        if self.ram.read_u8(0x301) != 0
            || self.ram.read_u8(0x37a) & 0x1f != 0
            || self.ram.read_u8(0x379) != 0
        {
            return;
        }
        if self.ram.read_u8(0x3c) < 9 && self.ram.read_u8(0x3a) & 0x80 != 0 {
            return;
        }
        if self.ram.read_u8(0x308) | self.ram.read_u8(0x376) == 0 {
            let action;
            if !self.link_check_new_a_press() {
                self.ram.write_u8(0x3b, 0);
                return;
            }
            if self.ram.read_u8(0x3f8) != 0 && self.ram.player().facing() == 0 {
                action = 7;
            } else if self.ram.read_u8(0x2fa) != 0 {
                action = 6;
            } else {
                if self.ram.read_u8(0x2ec) == 0 {
                    if self.ram.read_u8(0x314) == 0 {
                        action = self.link_handle_liftables();
                    } else {
                        self.ram.write_u8(0x2f4, self.ram.read_u8(0x314));
                        action = 1;
                    }
                } else {
                    action = 1;
                }
                if action == 1 {
                    if self.ram.read_u8(0x3c) != 0 {
                        self.link_reset_sword_and_item_usage();
                    }
                    if self.ram.read_u8(0x301) | self.ram.read_u8(0x37a) != 0 {
                        self.ram.write_u8(0x301, 0);
                        self.ram.write_u8(0x37a, 0);
                        self.link_reset_boomerang_y_stuff();
                        self.ram.write_u8(0x35f, 0);
                        if self.ram.read_u8(0xc4a) == 5 {
                            self.ram.write_u8(0xc4a, 0);
                        }
                    }
                }
            }
            if ABILITY[usize::from(action)] & self.ram.read_u8(0xf379) == 0 {
                self.ram.write_u8(0x3b, 0);
                return;
            }
            self.ram.write_u8(0x36c, action);
            self.link_a_press_perform_basic(action.wrapping_mul(2));
        }
        self.ram.write_u8(0x306, self.ram.read_u8(0x36c));
        match self.ram.read_u8(0x36c) {
            1 => self.link_a_press_lift_carry_throw(),
            3 => self.link_a_press_pull_object(),
            6 => self.link_a_press_statue_drag(),
            _ => {}
        }
    }

    pub fn link_state_recoil(&mut self) {
        self.save_safe_return_coordinates();
        self.link_handle_change_in_z_velocity();
        self.ram.write_u8(0x50, 0);
        self.ram.write_u8(0x351, 0);
        if (self.ram.player().z_low() as i8) >= 0
            || (self.ram.player().actual_velocity_z() as i8) >= 0
        {
            self.link_handle_recoil_and_timer(false);
            return;
        }
        self.tile_detect_main_handler(5);
        if self.ram.read_u16(0x341) & 1 != 0 {
            self.ram
                .set_player_action_state(PlayerState::Swimming as u8);
            self.link_set_to_deep_water();
            self.link_reset_sword_and_item_usage();
            self.runtime.ancilla_add_splash(self.ram, 21, 0);
            self.link_handle_recoil_and_timer(true);
        } else {
            let recoil = self.inc8(0x2c6);
            if recoil != 4 {
                // Preserve C's unusual `do ... while (!--s)` exactly. It
                // shifts twice only when the starting counter is one, and
                // once for every other nonzero value.
                self.ram.set_player_actual_velocity_z(recoil_z_velocity(
                    self.ram.read_u8(0x2c7),
                    recoil,
                ))
            } else {
                self.ram.write_u8(0x2c6, 3)
            }
            self.link_handle_recoil_and_timer(false)
        }
    }

    pub fn link_handle_recoil_and_timer(&mut self, jump_into_middle: bool) {
        let mut finish_landing = jump_into_middle;
        if !jump_into_middle {
            self.ram.write_u8(0x69, 0);
            self.ram.write_u8(0x68, 0);
            self.ram.write_u8(0x6a, 0);
            self.link_handle_recoiling();
            let timer = self.ram.read_u8(0x46).wrapping_sub(1);
            self.ram.write_u8(0x46, timer);
            if timer == 0 {
                self.ram.write_u8(0x46, 1);
                let z = (self.ram.player().z_low() & 0xfe) as i8;
                if z <= 0 && (self.ram.player().actual_velocity_z() as i8) < 0 {
                    if self.ram.read_u8(0x4d) != 0 {
                        self.ram.write_u8(0x37b, 0);
                        let previous = self.ram.player().action_state();
                        self.ram.write_u16(0x72, u16::from(previous));
                        if previous != 6 {
                            for a in [0x3c, 0x3a, 0x3d, 0x79] {
                                self.ram.write_u8(a, 0);
                            }
                        }
                        self.link_splash_upon_landing();
                        if self.ram.read_u8(0x2e0) == 0 || self.ram.read_u8(0x345) == 0 {
                            if self.ram.read_u8(0x2f8) != 0 {
                                self.ram.write_u8(0x2f8, 0);
                                self.runtime.ancilla_sfx2_near(self.ram, 33);
                            } else if previous != 2 && self.ram.player().action_state() != 4 {
                                self.runtime.ancilla_sfx2_near(self.ram, 33);
                            }
                            if self.ram.player().action_state() == 4 {
                                self.link_force_unequip_cape_quietly();
                                if self.ram.read_u8(0x1b) != 0
                                    && previous != 2
                                    && self.ram.player().flippers() != 0
                                {
                                    self.ram.write_u8(0xee, 1);
                                }
                                self.runtime.ancilla_add_splash(self.ram, 21, 0);
                            }
                            self.tile_detect_main_handler(0);
                            if self.ram.read_u16(0x357) & 1 != 0 {
                                self.runtime.ancilla_sfx2_near(self.ram, 26);
                            }
                            if self.ram.read_u16(0x359) & 1 != 0 && self.ram.read_u8(0x12e) != 36 {
                                self.runtime.ancilla_sfx2_near(self.ram, 28);
                            }
                            if self.ram.read_u16(0x341) & 1 != 0 {
                                self.ram.set_player_action_state(4);
                                self.link_set_to_deep_water();
                                self.link_reset_sword_and_item_usage();
                                self.runtime.ancilla_add_splash(self.ram, 21, 0);
                            }
                        }
                        if self.ram.read_u8(0xee) == 2 {
                            self.ram.write_u8(0xee, 0);
                        }
                        if self.ram.read_u8(0x47a) != 0 {
                            self.dungeon_handle_layer_change();
                        }
                        finish_landing = true;
                    }
                    self.ram.write_u8(0x2e, 0);
                    self.ram.write_u8(0x46, 0);
                }
            }
        } else {
            if self.ram.read_u8(0xee) == 2 {
                self.ram.write_u8(0xee, 0);
            }
            if self.ram.read_u8(0x47a) != 0 {
                self.dungeon_handle_layer_change();
            }
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x46, 0);
        }
        if finish_landing {
            self.ram.set_player_z(0);
            self.ram.write_u8(0x4d, 0);
            self.ram.write_u8(0x5e, 0);
            self.ram.write_u8(0x50, 0);
            for a in [0x301, 0x37a, 0x300, 0x37b, 0x360, 0x28, 0x27] {
                self.ram.write_u8(a, 0);
            }
        }
        if self.ram.player().action_state() != 5 && self.ram.read_u8(0x46) >= 33 {
            let t = self.ram.read_u8(0x2c5).wrapping_sub(1);
            self.ram.write_u8(0x2c5, t);
            if (t as i8) >= 0 {
                self.link_recoil_finish();
                return;
            }
            self.ram.write_u8(0x2c5, self.ram.read_u8(0x46) >> 4);
        }
        self.flag67_with_directions();
        if self.ram.player().action_state() != 6 {
            self.link_handle_diagonal_collision();
            if self.ram.read_u8(0x67) & 3 == 0 {
                self.ram.set_player_actual_velocity_x(0);
            }
            if self.ram.read_u8(0x67) & 0xc == 0 {
                self.ram.set_player_actual_velocity_y(0);
            }
        }
        self.link_move_position();
        self.link_recoil_finish();
    }

    fn link_recoil_finish(&mut self) {
        if self.ram.player().action_state() != 6 {
            self.link_handle_cardinal_collision();
            self.ram.write_u8(0x302, 0);
        }
        self.handle_indoor_camera_and_doors();
        if matches!(self.ram.player().z_low(), 0 | 0xe0..=0xff) {
            self.runtime.player_tile_detect_nearby(self.ram);
            if self.ram.read_u16(0x59) & 0xf == 0xf {
                self.ram.set_player_action_state(1);
                self.ram.write_u8(0x5e, 4);
            }
        }
        self.ram.set_player_z_high(0)
    }

    pub fn link_state_default(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.ram.player().action_state() == PlayerState::PermaBunny as u8 {
                self.player_handler_17_bunny();
            }
            return;
        }
        self.ram.write_u8(0x2ca, 0);
        if self.ram.read_u8(0x4d) != 0 {
            self.handle_link_from_1d();
        } else {
            self.player_handler_00_ground_3();
        }
    }

    pub fn player_handler_00_ground_3(&mut self) {
        self.ram.set_player_z(0xffff);
        self.ram.set_player_actual_velocity_z(0xff);
        self.ram.write_u8(0x2c6, 0);
        if !self.link_handle_toss() {
            self.link_handle_a_press();
            if self.ram.read_u8(0x308) | self.ram.read_u8(0x376) == 0
                && self.ram.read_u8(0x377) == 0
                && self.ram.player().action_state() != PlayerState::StartDash as u8
            {
                self.link_handle_y_item();
                if self.runtime.enhanced_features0() & 4096 != 0
                    && ((self.ram.read_u8(0x10) == 14 && self.ram.read_u8(0x11) != 2)
                        || matches!(self.ram.player().action_state(), 8..=10))
                {
                    self.ram.set_player_actual_velocity_y(0);
                    self.ram.set_player_actual_velocity_x(0);
                    return;
                }
                if self.ram.read_u8(0xf3c5) != 0 {
                    self.link_handle_sword_cooldown();
                    if self.ram.player().action_state() == PlayerState::SpinAttacking as u8 {
                        self.ram.set_player_actual_velocity_y(0);
                        self.ram.set_player_actual_velocity_x(0);
                        return;
                    }
                }
            }
        }
        self.link_handle_cape_passive_lift_check();
        if self.ram.read_u8(0x46) != 0 {
            self.ram.write_u8(0x6b, 0);
            for address in [0x2e9, 0x30d, 0x30e, 0x30a, 0x3b, 0x309, 0x308, 0x376] {
                self.ram.write_u8(address, 0);
            }
            if self.ram.read_u8(0x3a) & 0x80 == 0 {
                self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }
        if self.ram.read_u8(0x377) != 0 {
            self.ram.write_u8(0x67, 0);
        } else if self.ram.read_u8(0x2e1) == 0
            && self.ram.read_u8(0x376) & !2 == 0
            && self.ram.read_u8(0x308) & 0x7f == 0
            && (self.ram.read_u8(0x308) & 0x80 == 0 || self.ram.read_u8(0x309) & 1 == 0)
            && self.ram.read_u8(0x301) == 0
            && self.ram.read_u8(0x37a) == 0
            && (self.ram.read_u8(0x3c) >= 9
                || self.ram.read_u8(0x3a) & 0x20 != 0
                || self.ram.read_u8(0x3a) & 0x80 == 0)
        {
            if self.ram.read_u8(0x34a) != 0 {
                self.ram.set_swim_speed_limit(SwimAxis::Vertical, 0x180);
                self.ram.set_swim_speed_limit(SwimAxis::Horizontal, 0x180);
                self.link_handle_swim_movements();
                return;
            }
            self.reset_all_acceleration();
            let mut direction = self.ram.read_u16(0x49) as u8 & 0xf;
            if direction == 0 && self.ram.read_u8(0x376) & 2 == 0 {
                direction = self.ram.read_u8(0xf0) & 0xf;
                if direction == 0 {
                    for address in [0x31, 0x30, 0x67, 0x26, 0x2e] {
                        self.ram.write_u8(address, 0);
                    }
                    self.ram.write_u8(0x48, self.ram.read_u8(0x48) & !0xf);
                    self.ram.write_u8(0x371, 32);
                    self.ram.write_u8(0x375, 19);
                }
            }
            if direction != 0 {
                self.ram.write_u8(0x67, direction);
                if direction != self.ram.read_u8(0x26) {
                    self.ram.write_u8(0x26, direction);
                    self.ram.write_u8(0x6b, 0);
                    for address in [0x2b, 0x2a, 0x2e9, 0x48] {
                        self.ram.write_u8(address, 0);
                    }
                    self.ram.write_u8(0x371, 32);
                    self.ram.write_u8(0x375, 19);
                }
            }
        }
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        if self.ram.read_u8(0x377) != 0 {
            self.ram.set_player_actual_velocity_y(0);
            self.ram.set_player_actual_velocity_x(0);
        }
        self.ram.write_u8(0x302, 0);
        self.handle_indoor_camera_and_doors();
    }

    pub fn handle_link_from_1d(&mut self) {
        for address in [
            0x301, 0x37a, 0x20b, 0x350, 0x30d, 0x30e, 0x30a, 0x3b, 0x308, 0x309, 0x376, 0x48,
        ] {
            self.ram.write_u8(address, 0);
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        self.link_reset_swimming_state();
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
        self.ram.set_player_z(self.ram.player().z() & 0xff);
        if self.ram.read_u8(0x360) != 0 {
            if self.ram.read_u8(0x55) != 0 {
                self.link_force_unequip_cape_quietly();
            }
            self.link_reset_sword_and_item_usage();
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x3d, 2);
            self.ram.write_u8(0x2e, 0);
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
            self.runtime.ancilla_sfx3_near(self.ram, 43);
            self.ram
                .set_player_action_state(PlayerState::Electrocution as u8);
            self.link_state_zapped();
        } else {
            self.ram.write_u8(0x6b, 0);
            self.ram
                .set_player_action_state(PlayerState::RecoilWall as u8);
            self.link_state_recoil();
        }
    }

    pub fn link_state_temporary_bunny(&mut self) {
        let timer = self.ram.read_u16(0x3f5);
        if timer == 0 {
            self.runtime.ancilla_add_cape_poof(self.ram, 0x23, 4);
            self.runtime.ancilla_sfx2_near(self.ram, 0x15);
            self.ram.write_u8(0x2e2, 32);
            self.ram.set_player_action_state(PlayerState::Ground as u8);
            self.link_reset_properties_c();
            self.ram.write_u8(0x3f7, 0);
            self.ram.write_u8(0x56, 0);
            self.ram.write_u8(0x2e0, 0);
            self.runtime.load_actual_gear_palettes(self.ram);
            self.ram.write_u8(0x3f7, 0);
            self.link_state_default();
        } else {
            self.ram.write_u16(0x3f5, timer.wrapping_sub(1));
            self.player_handler_17_bunny();
        }
    }

    pub fn player_handler_17_bunny(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.ram.write_u8(0x2ca, 0);
        if self.ram.read_u8(0x345) == 0 {
            if self.ram.read_u8(0x4d) == 0 {
                self.link_temp_bunny_func2();
                return;
            }
            if self.ram.player().moon_pearl() != 0 {
                self.ram.write_u8(0x2e0, 0);
            }
        }
        self.link_state_bunny_recache();
    }

    pub fn link_temp_bunny_func2(&mut self) {
        if self.ram.read_u8(0x46) != 0 {
            self.link_handle_recoil_and_timer(false);
            return;
        }
        self.ram.set_player_z(0xffff);
        self.ram.set_player_actual_velocity_z(0xff);
        self.ram.write_u8(0x2c6, 0);
        if self.ram.read_u8(0x34a) != 0 {
            self.ram.set_swim_speed_limit(SwimAxis::Vertical, 0x180);
            self.ram.set_swim_speed_limit(SwimAxis::Horizontal, 0x180);
            self.link_handle_swim_movements();
            return;
        }
        self.reset_all_acceleration();
        self.link_handle_y_item();
        let mut direction = self.ram.read_u16(0x49) as u8 & 0xf;
        if direction == 0 {
            direction = self.ram.read_u8(0xf0) & 0xf;
        }
        if direction == 0 {
            for address in [0x28, 0x27, 0x67, 0x26, 0x2e] {
                self.ram.write_u8(address, 0);
            }
            self.ram.write_u8(0x48, self.ram.read_u8(0x48) & !9);
            self.ram.write_u8(0x371, 32);
            self.ram.write_u8(0x375, 19);
        } else {
            self.ram.write_u8(0x67, direction);
            if direction != self.ram.read_u8(0x26) {
                self.ram.write_u8(0x26, direction);
                for address in [0x2b, 0x2a, 0x2e9, 0x48] {
                    self.ram.write_u8(address, 0);
                }
                self.ram.write_u8(0x371, 32);
                self.ram.write_u8(0x375, 19);
            }
        }
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.ram.write_u8(0x302, 0);
        self.handle_indoor_camera_and_doors();
    }

    pub fn link_state_holding_big_rock(&mut self) {
        if self.ram.read_u8(0x4d) != 0 {
            for address in [
                0x301, 0x37a, 0x20b, 0x350, 0x30d, 0x30e, 0x30a, 0x3b, 0x308, 0x309, 0x376, 0x48,
            ] {
                self.ram.write_u8(address, 0);
            }
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            self.ram.set_player_z(self.ram.player().z() & 0xff00);
            if self.ram.read_u8(0x360) != 0 {
                self.link_reset_sword_and_item_usage();
                self.ram.write_u8(0x37b, 1);
                self.ram.write_u8(0x300, 0);
                self.ram.write_u8(0x3d, 2);
                self.ram.write_u8(0x2e, 0);
                self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
                self.runtime.ancilla_sfx3_near(self.ram, 43);
                self.ram
                    .set_player_action_state(PlayerState::Electrocution as u8);
                self.link_state_zapped();
            } else {
                self.ram
                    .set_player_action_state(PlayerState::RecoilWall as u8);
                self.link_state_recoil();
            }
            return;
        }
        self.ram.set_player_z(0xffff);
        self.ram.set_player_actual_velocity_z(0xff);
        self.ram.write_u8(0x2c6, 0);
        if self.ram.read_u8(0x46) != 0 {
            for address in [0x30d, 0x30e, 0x30a, 0x3b, 0x308, 0x309, 0x376] {
                self.ram.write_u8(address, 0);
            }
            if self.ram.read_u8(0x3a) & 0x80 == 0 {
                self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }
        self.link_handle_a_press();
        let direction = self.ram.read_u8(0xf0) & 0xf;
        if direction == 0 {
            for address in [0x27, 0x28, 0x67, 0x26, 0x2e] {
                self.ram.write_u8(address, 0);
            }
            self.ram.write_u8(0x48, self.ram.read_u8(0x48) & !9);
            self.ram.write_u8(0x371, 32);
            self.ram.write_u8(0x375, 19);
        } else {
            self.ram.write_u8(0x67, direction);
            if direction != self.ram.read_u8(0x26) {
                self.ram.write_u8(0x26, direction);
                for address in [0x2b, 0x2a, 0x2e9, 0x48] {
                    self.ram.write_u8(address, 0);
                }
                self.ram.write_u8(0x371, 32);
                self.ram.write_u8(0x375, 19);
            }
        }
        self.link_handle_moving_animation_full_long_entry();
        self.ram.write_u8(0x302, 0);
        self.handle_indoor_camera_and_doors();
    }

    pub fn player_handler_04_swimming(&mut self) {
        if self.ram.read_u8(0x4d) != 0 {
            self.ram
                .set_player_action_state(PlayerState::RecoilWall as u8);
            self.ram.set_player_z(self.ram.player().z() & 0xff);
            self.reset_all_acceleration();
            self.ram.write_u8(0x32a, 0);
            self.ram.write_u8(0x34f, 0);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            self.link_state_recoil();
            return;
        }
        for address in [0x3a, 0x3c, 0x3d, 0x79, 0x308, 0x309] {
            self.ram.write_u8(address, 0);
        }
        if self.ram.player().flippers() == 0 {
            return;
        }
        if self.ram.swim_axis(SwimAxis::Vertical).speed()
            | self.ram.swim_axis(SwimAxis::Horizontal).speed()
            == 0
        {
            if self.ram.swim_axis(SwimAxis::Vertical).mode_low() != 2
                && self.ram.swim_axis(SwimAxis::Horizontal).mode_low() != 2
            {
                self.reset_all_acceleration();
            }
            self.ram.write_u8(0x2e, self.ram.read_u8(0x2e) & 1);
            if self.inc8(0x2d) >= 16 {
                self.ram.write_u8(0x2d, 0);
                self.ram.write_u8(0x2cc, 0);
                self.ram.write_u8(0x2e, (self.ram.read_u8(0x2e) & 1) ^ 1);
            }
        } else if self.inc8(0x2d) >= 8 {
            self.ram.write_u8(0x2d, 0);
            let frame = self.ram.read_u8(0x2e).wrapping_add(1) & 3;
            self.ram.write_u8(0x2e, frame);
            self.ram.write_u8(0x2cc, SWIMMING_TAB1[usize::from(frame)]);
        }
        if self.ram.read_u8(0x34f) == 0 {
            let stroke = (self.ram.read_u8(0xf6) | self.ram.read_u8(0xf4)) & 0xc0;
            if self.ram.swim_axis(SwimAxis::Vertical).speed()
                | self.ram.swim_axis(SwimAxis::Horizontal).speed()
                == 0
                || stroke == 0
            {
                self.link_handle_swim_movements();
                return;
            }
            self.ram.write_u8(0x34f, stroke);
            self.runtime.ancilla_sfx2_near(self.ram, 37);
            self.ram.write_u8(0x32a, 1);
            self.ram.write_u8(0x2cb, 7);
            self.link_handle_swim_accels();
        }
        let countdown = self.ram.read_u8(0x2cb).wrapping_sub(1);
        self.ram.write_u8(0x2cb, countdown);
        if (countdown as i8) < 0 {
            self.ram.write_u8(0x2cb, 7);
            if self.inc8(0x32a) == 5 {
                self.ram.write_u8(0x32a, 0);
                self.ram.write_u8(0x34f, self.ram.read_u8(0x34f) & !0xc0);
            }
        }
        self.link_handle_swim_movements();
    }

    pub fn link_state_exiting_dash(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.ram.read_u8(0xf0) & 0xf != 0 || self.ram.read_u8(0x374) >= 16 {
            self.ram.write_u8(0x374, 0);
            self.ram.write_u8(0x5e, 0);
            self.ram.set_player_action_state(PlayerState::Ground as u8);
            self.ram.write_u8(0x372, 0);
            self.ram.set_swim_acceleration_mode(
                SwimAxis::Vertical,
                self.ram.swim_axis(SwimAxis::Vertical).acceleration_mode() & 0xff00,
            );
            if self.ram.read_u8(0x3c) < 9 {
                self.ram.write_u8(0x50, 0);
            }
        } else {
            self.inc8(0x374);
        }
        self.link_handle_moving_animation_full_long_entry();
    }

    pub fn link_state_tree_pull(&mut self) {
        const STEPS2: [u8; 10] = [0, 1, 2, 3, 4, 0, 1, 2, 3, 0x20];
        self.cache_camera_properties_if_outdoors();
        if self.ram.read_u8(0x4d) != 0 {
            self.handle_link_from_1d();
            return;
        }
        if self.ram.read_u8(0x376) != 0 {
            if self.ram.read_u8(0x3a) == 0 {
                if self.ram.read_u8(0xf2) & 0x80 == 0 {
                    self.ram.write_u8(0x376, 0);
                    self.ram.write_u8(0x30d, 0);
                    self.ram.write_u8(0x30b, 2);
                    self.ram.write_u8(0x30a, 0);
                    self.ram.write_u8(0x50, 0);
                    self.ram.set_player_action_state(PlayerState::Ground as u8);
                    self.link_state_default();
                    return;
                }
                if self.ram.read_u8(0xf0) & 4 != 0 {
                    self.ram.write_u8(0x3a, 4);
                    self.runtime.ancilla_sfx2_near(self.ram, 0x22);
                } else {
                    self.link_move_position();
                    self.link_handle_cardinal_collision();
                    self.handle_indoor_camera_and_doors();
                    return;
                }
            }
            if self.ram.read_u8(0x3a) != 0 {
                let timer = self.ram.read_u8(0x30b).wrapping_sub(1);
                self.ram.write_u8(0x30b, timer);
                if (timer as i8) < 0 {
                    let step = self.inc8(0x30d);
                    self.ram.write_u8(
                        0x30a,
                        if step == 7 {
                            GRAB_WALL_ANIM_TIMER[0]
                        } else {
                            GRAB_WALL_ANIM_STEPS[usize::from(step)]
                        },
                    );
                    self.ram.write_u8(
                        0x30b,
                        if step == 7 {
                            CAPE_DEPLETION_TIMERS[0]
                        } else {
                            GRAB_WALL_ANIM_TIMER[usize::from(step)]
                        },
                    );
                    if step == 7 {
                        self.ram.write_u8(0x376, 0);
                        self.ram.write_u8(0x30d, 0);
                        self.ram.write_u8(0x30b, 2);
                        self.ram.write_u8(0x30a, 0);
                        self.ram.write_u8(0x308, 1);
                        self.ram.write_u8(0x309, 0);
                    } else {
                        self.link_move_position();
                        self.link_handle_cardinal_collision();
                        self.handle_indoor_camera_and_doors();
                        return;
                    }
                } else {
                    self.link_move_position();
                    self.link_handle_cardinal_collision();
                    self.handle_indoor_camera_and_doors();
                    return;
                }
            }
        }
        if self.ram.read_u8(0x48) & 9 != 0 {
            self.ram.set_player_facing(0);
            self.ram.write_u8(0x308, 0);
            self.ram.write_u8(0x50, 0);
            self.ram.set_player_action_state(PlayerState::Ground as u8);
            return;
        }
        if self.ram.read_u8(0x30d) == 9 {
            if self.ram.read_u8(0xf4) & 0xf != 0 {
                self.ram.set_player_action_state(PlayerState::Ground as u8);
                self.link_state_default();
                return;
            }
        } else {
            self.runtime.ancilla_add_dash_dust(self.ram, true, 0x1e, 0);
            let timer = self.ram.read_u8(0x30b).wrapping_sub(1);
            self.ram.write_u8(0x30b, timer);
            if (timer as i8) < 0 {
                let step = self.inc8(0x30d);
                self.ram.write_u8(0x30a, STEPS2[usize::from(step)]);
                self.ram.write_u8(0x30b, 2);
                self.ram.set_player_actual_velocity_y(48);
                if step == 9 {
                    self.ram.set_player_facing(0);
                    self.ram.write_u8(0x308, 0);
                    self.ram.write_u8(0x50, 0);
                    self.ram.set_player_action_state(PlayerState::Ground as u8);
                    return;
                }
            }
            self.flag67_with_directions();
            if self.ram.read_u8(0x67) & 3 == 0 {
                self.ram.set_player_actual_velocity_x(0);
            }
            if self.ram.read_u8(0x67) & 0xc == 0 {
                self.ram.set_player_actual_velocity_y(0);
            }
            self.link_move_position();
        }
        self.link_handle_cardinal_collision();
        self.handle_indoor_camera_and_doors();
    }

    pub fn link_state_handling_jump(&mut self) {
        self.ram
            .set_player_actual_velocity_z(self.ram.read_u8(0x362));
        self.ram.write_u8(0x2c7, self.ram.read_u8(0x363));
        self.ram.set_player_z_low(self.ram.read_u8(0x364));
        let vz = self.ram.player().actual_velocity_z().wrapping_sub(2);
        self.ram.set_player_actual_velocity_z(vz);
        self.link_move_position();
        if (vz as i8) < 0 {
            if vz < 0xa0 {
                self.ram.set_player_actual_velocity_z(0xa0);
            }
            if self.ram.player().z_low() >= 0xf0 {
                self.ram.set_player_z(0);
                if matches!(self.ram.player().action_state(), 12 | 14) {
                    self.tile_detect_main_handler(0);
                    if self.ram.read_u16(0x341) & 1 != 0 {
                        self.ram
                            .set_player_action_state(PlayerState::Swimming as u8);
                        self.link_set_to_deep_water();
                        self.link_reset_sword_and_item_usage();
                        self.runtime.ancilla_add_splash(self.ram, 21, 0);
                    } else if self.ram.read_u16(0x59) & 1 != 0 {
                        self.ram.write_u8(0x5c, 9);
                        self.ram.write_u8(0x5a, 0);
                        self.ram.write_u8(0x5b, 1);
                        self.ram
                            .set_player_action_state(PlayerState::FallingIntoHole as u8);
                        self.link_finish_jump_landing();
                        return;
                    }
                }
                self.link_splash_upon_landing();
                if self.ram.player().action_state() != PlayerState::Swimming as u8
                    && self.ram.read_u8(0x345) == 0
                {
                    self.runtime.ancilla_sfx2_near(self.ram, 33);
                }
                self.link_finish_jump_landing();
                return;
            }
        }
        self.ram.set_player_velocity_y(
            self.ram
                .read_u8(0x364)
                .wrapping_sub(self.ram.player().z_low()),
        );
        self.ram
            .write_u8(0x362, self.ram.player().actual_velocity_z());
        self.ram.write_u8(0x363, self.ram.read_u8(0x2c7));
        self.ram.write_u8(0x364, self.ram.player().z_low());
    }

    fn link_finish_jump_landing(&mut self) {
        if self.ram.player().action_state() != PlayerState::Swimming as u8
            || self.ram.read_u8(0x2e0) == 0
        {
            self.ram.write_u8(0x37b, 0);
        }
        self.ram.write_u8(0x78, 0);
        self.ram.write_u8(0x4d, 0);
        self.ram.set_player_actual_velocity_z(0xff);
        self.ram.set_player_z(0xffff);
        self.ram.write_u8(0x46, 0);
        if self.ram.read_u8(0x1b) == 0 {
            self.ram.write_u8(0xee, 0);
        }
        self.ram
            .write_u8(0x362, self.ram.player().actual_velocity_z());
        self.ram.write_u8(0x363, self.ram.read_u8(0x2c7));
        self.ram.write_u8(0x364, self.ram.player().z_low());
    }

    pub fn link_state_hopping_horizontally_ow(&mut self) {
        self.ram.write_u8(
            0x67,
            if (self.ram.player().actual_velocity_x() as i8) < 0 {
                6
            } else {
                5
            },
        );
        self.ram.write_u8(0x50, 0);
        self.ram.set_player_actual_velocity_y(0);
        self.ram.write_u8(0x351, 0);
        self.link_state_handling_jump();
    }

    pub fn link_state_0f(&mut self) {
        panic!("unreachable player state 0x0f")
    }

    pub fn link_state_hopping_diagonally_up_ow(&mut self) {
        self.ram.write_u8(0x351, 0);
        self.player_change_z(2);
        self.link_move_position();
        if (self.ram.player().z_low() as i8) < 0 {
            self.link_splash_upon_landing();
            if self.ram.player().action_state() != PlayerState::Swimming as u8
                && self.ram.read_u8(0x345) == 0
            {
                self.runtime.ancilla_sfx2_near(self.ram, 33);
            }
            self.ram.write_u8(0x37b, 0);
            self.ram.write_u8(0x4d, 0);
            self.ram.set_player_actual_velocity_z(0xff);
            self.ram.set_player_z(0xffff);
            self.ram.write_u8(0x46, 0);
            self.ram.write_u8(0x50, 0);
        }
    }

    pub fn link_state_hopping_diagonally_down_ow(&mut self) {
        const VX: [u8; 24] = [
            4, 4, 4, 10, 10, 10, 11, 18, 18, 18, 20, 20, 20, 20, 22, 22, 26, 26, 26, 26, 28, 28,
            28, 28,
        ];
        let direction = if (self.ram.player().actual_velocity_x() as i8) < 0 {
            2
        } else {
            3
        };
        self.ram.write_u8(0x66, direction);
        self.ram.write_u8(0x50, 0);
        self.ram.set_player_actual_velocity_y(0);
        self.ram.write_u8(0x351, 0);
        if self.ram.read_u8(0x46) == 0 && self.ram.read_u8(0x362) == 0 {
            self.ram.write_u8(0x66, 1);
            let old_x = self.ram.player().x();
            self.runtime.ancilla_sfx2_near(self.ram, 32);
            self.link_hop_find_landing_spot_diagonally_down();
            self.ram.set_player_x(old_x);
            let distance = self.ram.player().y().wrapping_sub(self.ram.read_u16(0x32));
            let velocity = VX[usize::from(((distance >> 3).min(23)) as u8)];
            self.ram.set_player_actual_velocity_x(if direction != 2 {
                velocity
            } else {
                0u8.wrapping_sub(velocity)
            });
            if self.ram.read_u8(0x1b) == 0 {
                self.ram.write_u8(0xee, 2);
            }
        }
        self.link_state_handling_jump();
    }

    pub fn link_state_hookshotting(&mut self) {
        const YA: [i8; 4] = [-8, -16, 0, 0];
        const XA: [i8; 4] = [0, 0, 4, -12];
        const YV: [i8; 4] = [-64, 64, 0, 0];
        const XV: [i8; 4] = [0, 0, -64, 64];
        self.ram.write_u8(0x373, 0);
        self.ram.write_u8(0x4d, 0);
        self.ram.write_u8(0x46, 0);
        let slot = (0..5).rev().find(|&i| self.ram.read_u8(0xc4a + i) == 0x1f);
        let Some(_slot) = slot else {
            let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
            self.ram.write_u8(0x3d, delay);
            if (delay as i8) >= 0 {
                return;
            }
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x37b, 0);
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            self.ram.write_u8(0x37a, self.ram.read_u8(0x37a) & !4);
            self.ram.set_player_action_state(PlayerState::Ground as u8);
            if self.ram.read_u8(0x3c) >= 9 {
                self.ram.write_u8(0x3c, 9);
            }
            return;
        };
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram
            .write_u8(0x3d, if (delay as i8) < 0 { 0 } else { delay });
        if self.ram.read_u8(0x37e) == 0 {
            self.save_safe_return_coordinates();
            self.ram.set_player_actual_velocity_y(0);
            self.ram.set_player_actual_velocity_x(0);
            self.link_handle_cardinal_collision();
            return;
        }
        self.ram.write_u8(0x2f5, 0);
        let effect = usize::from(self.ram.read_u8(0x39d));
        let timer = self.ram.read_u8(0xc5e + effect).wrapping_sub(1);
        self.ram.write_u8(0xc5e + effect, timer);
        if (timer as i8) >= 0 {
            let direction = usize::from(self.ram.read_u8(0xc72 + effect));
            let x = u16::from_le_bytes([
                self.ram.read_u8(0xc04 + effect),
                self.ram.read_u8(0xc18 + effect),
            ]);
            let y = u16::from_le_bytes([
                self.ram.read_u8(0xbfa + effect),
                self.ram.read_u8(0xc0e + effect),
            ]);
            self.ram.set_player_actual_velocity_y(0);
            self.ram.set_player_actual_velocity_x(0);
            let yd = add_i8(y, YA[direction]).wrapping_sub(self.ram.player().y()) as i16;
            if yd.unsigned_abs() >= 2 {
                self.ram.set_player_actual_velocity_y(YV[direction] as u8);
            }
            let xd = add_i8(x, XA[direction]).wrapping_sub(self.ram.player().x()) as i16;
            if xd.unsigned_abs() >= 2 {
                self.ram.set_player_actual_velocity_x(XV[direction] as u8);
            }
            if self.ram.player().actual_velocity_y() | self.ram.player().actual_velocity_x() != 0 {
                self.link_move_position();
                self.tile_detect_main_handler(5);
                if self.ram.read_u8(0x1b) != 0 {
                    let ledges = (self.ram.read_u8(0x36d) >> 4)
                        | self.ram.read_u8(0x36d)
                        | self.ram.read_u8(0x36e);
                    if ledges & 1 != 0 {
                        let t = self.ram.read_u8(0x3f9).wrapping_sub(1);
                        self.ram.write_u8(0x3f9, t);
                        if (t as i8) < 0 {
                            self.ram.write_u8(0x3f9, 3);
                            self.ram.write_u8(0x37e, self.ram.read_u8(0x37e) ^ 2);
                        }
                    }
                }
                self.ram.write_u8(0x351, 0);
                if self.ram.read_u8(0x37e) & 2 == 0 {
                    if self.ram.read_u16(0x357) & 1 != 0 {
                        self.ram.write_u8(0x351, 2);
                        if !self.link_permission_for_slosh_sounds() {
                            self.runtime.ancilla_sfx2_near(self.ram, 26);
                        }
                    } else if (self.ram.read_u16(0x359) | self.ram.read_u16(0x341)) & 1 != 0 {
                        self.ram.write_u8(0x351, 1);
                        self.runtime.ancilla_sfx2_near(
                            self.ram,
                            if self.ram.read_u8(0x8a) == 0x70 {
                                27
                            } else {
                                28
                            },
                        );
                    }
                }
                self.handle_indoor_camera_and_doors();
                return;
            }
        } else {
            self.ram.write_u8(0xc5e + effect, 0);
        }
        self.ram.write_u8(0xc4a + effect, 0);
        self.ram.write_u8(0x2d1, self.ram.read_u8(0x2d3));
        for address in [0x300, 0x3d, 0x37e, 0x37b] {
            self.ram.write_u8(address, 0);
        }
        self.ram.set_player_action_state(PlayerState::Ground as u8);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
        self.ram.write_u8(0x37a, self.ram.read_u8(0x37a) & !4);
        if self.ram.read_u8(0x3a4 + effect) != 0 {
            self.ram.write_u8(0xee, self.ram.read_u8(0xee) ^ 1);
            self.ram
                .write_u8(0xa4, self.ram.read_u8(0xa4).wrapping_sub(1));
            if self.ram.read_u16(0x44a) == 0 {
                self.ram.write_u8(0x48e, self.ram.read_u8(0x4a));
                self.ram
                    .write_u8(0x4a, self.ram.read_u8(0x4a).wrapping_add(0x10));
            }
            if self.ram.read_u16(0x44a) != 2 {
                self.ram.write_u8(0xee, self.ram.read_u8(0xee) ^ 1);
            }
            self.runtime.dungeon_flag_room_data_quadrants(self.ram);
        }
        self.runtime.player_tile_detect_nearby(self.ram);
        if self.ram.read_u16(0x341) & 0xf != 0 && self.ram.read_u8(0x345) == 0 {
            self.ram.write_u8(0x345, 1);
            self.ram.write_u8(0x340, self.ram.read_u8(0x26));
            self.link_reset_swimming_state();
            self.runtime.ancilla_add_splash(self.ram, 21, 0);
            self.ram
                .set_player_action_state(PlayerState::Swimming as u8);
            self.link_force_unequip_cape_quietly();
            for address in [0x308, 0x309, 0x376, 0x5e] {
                self.ram.write_u8(address, 0);
            }
            if self.ram.read_u8(0x1b) != 0 {
                self.ram.write_u8(0xee, 1);
            }
            if self.ram.read_u8(0x3c) >= 9 {
                self.ram.write_u8(0x3c, 9);
            }
        } else if self.ram.read_u16(0x59) & 0xf != 0 {
            self.ram.write_u8(0x5c, 9);
            self.ram.write_u8(0x5a, 0);
            self.ram.write_u8(0x5b, 1);
            self.ram
                .set_player_action_state(PlayerState::FallingIntoHole as u8);
            if self.ram.read_u8(0x3c) >= 9 {
                self.ram.write_u8(0x3c, 9);
            }
        } else {
            self.save_safe_return_coordinates();
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
        }
    }

    pub fn link_state_dashing(&mut self) {
        const FACING: [u8; 4] = [8, 4, 2, 1];
        const SOUND_MASK: [u8; 3] = [7, 15, 15];
        const TURN: [u8; 16] = [0, 1, 2, 0, 4, 4, 4, 0, 8, 8, 8, 0, 0, 0, 0, 0];
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.ram.player().action_state() == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        if self.ram.read_u8(0x372) == 0 {
            for a in [0x37b, 0x374, 0x5e, 0x50] {
                self.ram.write_u8(a, 0);
            }
            self.ram.set_player_action_state(0);
            return;
        }
        if self.ram.read_u8(0x3a) & 0x80 != 0 && self.ram.read_u8(0x3c) >= 9 {
            self.ram.write_u8(0x3c, 9);
        }
        self.ram.write_u8(0x2ca, 0);
        if self.ram.read_u8(0x4d) != 0 {
            for a in [0x37b, 0x374, 0x5e, 0x50, 0x372, 0x48] {
                self.ram.write_u8(a, 0);
            }
            if self.ram.read_u8(0x360) != 0 {
                if self.ram.read_u8(0x55) != 0 {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.ram.write_u8(0x37b, 1);
                self.ram.write_u8(0x300, 0);
                self.ram.write_u8(0x3d, 2);
                self.ram.write_u8(0x2e, 0);
                self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
                self.runtime.ancilla_sfx3_near(self.ram, 43);
                self.ram.set_player_action_state(7);
                self.link_state_zapped();
            } else {
                self.ram.set_player_action_state(2);
                self.link_state_recoil();
            }
            return;
        }
        let countdown = self.ram.read_u8(0x374);
        let mut sound = countdown;
        if sound == 0 {
            sound = self.ram.read_u8(0x4f);
            self.ram.write_u8(0x4f, sound.wrapping_sub(1));
        }
        if SOUND_MASK[usize::from(countdown >> 4)] & sound == 0 {
            self.runtime.ancilla_sfx2_near(self.ram, 35);
        }
        let countdown = countdown.wrapping_sub(1);
        self.ram.write_u8(0x374, countdown);
        if (countdown as i8) >= 0 {
            self.ram.write_u8(0x4f, 0);
            if self.ram.read_u8(0xf2) & 0x80 == 0 {
                for a in [0x2e, 0x374, 0x5e, 0x372] {
                    self.ram.write_u8(a, 0);
                }
                self.ram.set_player_action_state(0);
                if self.ram.read_u8(0x3a) & 0x80 == 0 {
                    self.ram.write_u8(0x50, 0);
                }
                return;
            }
            self.runtime.ancilla_add_dash_dust(self.ram, true, 30, 0);
            self.ram.set_player_velocity_y(0);
            self.ram.set_player_velocity_x(0);
            self.ram.write_u8(0x2f1, 64);
            self.ram.write_u8(0x5e, 16);
            let mut d = self.ram.read_u8(0xf0) & 0xf;
            if self.ram.read_u8(0x3a) & 0x80 != 0 || self.ram.read_u8(0x6c) != 0 || d == 0 {
                d = FACING[usize::from(self.ram.player().facing() >> 1)];
            }
            self.ram.write_u8(0x340, d);
            self.ram.write_u8(0x67, d);
            self.ram.write_u8(0x26, d);
            self.ram.write_u8(0x6b, 0);
            self.ram.write_u8(0x2e9, 0);
            self.link_handle_moving_animation_full_long_entry();
            let ox = self.ram.player().x();
            let oy = self.ram.player().y();
            self.save_safe_return_coordinates();
            self.link_handle_moving_floor();
            self.link_apply_conveyor();
            if self.ram.read_u8(0x2f5) != 0 {
                self.link_handle_velocity_and_sand_drag(ox, oy);
            }
            self.ram.set_player_velocity_y(
                self.ram
                    .player()
                    .y()
                    .wrapping_sub(u16::from(self.ram.read_u8(0x3e))) as u8,
            );
            self.ram.set_player_velocity_x(
                self.ram
                    .player()
                    .x()
                    .wrapping_sub(u16::from(self.ram.read_u8(0x3f))) as u8,
            );
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
            return;
        }
        self.ram.write_u8(0x374, 0);
        let follower = usize::from(self.ram.read_u8(0xf3cc));
        if follower < TAGALONG_ARR1.len() && TAGALONG_ARR1[follower] == follower as i8 {
            self.ram.write_u8(0xf3cc, TAGALONG_ARR2[follower] as u8);
        }
        if self.ram.read_u8(0x2e) >= 6 {
            self.ram.write_u8(0x2e, 0);
        }
        let dash = self.ram.read_u8(0x2f1).wrapping_sub(1);
        self.ram.write_u8(0x2f1, dash.max(32));
        self.runtime.ancilla_add_dash_dust(self.ram, false, 30, 0);
        self.ram.write_u8(0x79, 0);
        if self.ram.player().sword().wrapping_add(1) & 0xfe != 0 {
            self.tile_detect_main_handler(7);
        }
        if self.ram.read_u8(0xf3c5) != 0 {
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) | 0x80);
            self.ram.write_u8(0x3c, 9);
        }
        self.ram.write_u8(0x46, 0);
        let turning = self.runtime.enhanced_features0() & 4 != 0;
        let stop = if turning {
            if self.ram.read_u8(0xf2) & 0x80 == 0 {
                self.ram.write_u8(0x374, 0x11);
                true
            } else {
                let d = TURN[usize::from(self.ram.read_u8(0xf0) & 0xf)];
                if d != 0 && d != self.ram.read_u8(0x26) {
                    self.ram.write_u8(0x67, d);
                    self.ram.write_u8(0x26, d);
                    self.ram.write_u8(0x340, d);
                    self.link_handle_moving_animation_full_long_entry();
                }
                false
            }
        } else {
            let d = self.ram.read_u8(0xf0) & 0xf;
            d != 0 && d != FACING[usize::from(self.ram.player().facing() >> 1)]
        };
        if stop {
            self.ram.set_player_action_state(18);
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x80);
            self.ram.write_u8(0x3c, 0);
            self.ram.write_u8(0x3d, 0);
            self.link_state_exiting_dash();
            return;
        }
        if self.ram.read_u8(0x5e) == 0 && turning {
            self.ram.write_u8(0x5e, 16);
        }
        let mut d = self.ram.read_u16(0x49) as u8 & 0xf;
        if d == 0 {
            d = FACING[usize::from(self.ram.player().facing() >> 1)];
        }
        self.ram.write_u8(0x67, d);
        self.ram.write_u8(0x26, d);
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.ram.write_u8(0x302, 0);
        self.handle_indoor_camera_and_doors();
    }

    pub fn link_state_pits(&mut self) {
        const PIT_DIRS: [u8; 4] = [12, 3, 10, 5];
        const DIRS: [u8; 8] = [5, 6, 9, 10, 4, 8, 1, 2];
        const DIRS2: [u8; 8] = [10, 9, 6, 5, 8, 4, 2, 1];
        self.ram.write_u8(0x67, 0);
        if self.ram.read_u8(0x302) != 0 && self.inc8(0x2ca) == 0x20 {
            self.ram.write_u8(0x2ca, 31);
        } else if self.ram.read_u8(0x372) != 0 {
            if self.ram.read_u8(0x374) != 0
                && (self.runtime.enhanced_features0() & 4096 == 0
                    || self.ram.read_u8(0xf2) & 0x80 != 0)
            {
                self.link_state_dashing();
                return;
            }
            let input = self.ram.read_u8(0xf0) & 0xf;
            if input != 0 {
                self.link_cancel_dash();
                if self.ram.read_u8(0x4d) != 1 {
                    self.ram.write_u8(0x67, input);
                }
            }
        } else if self.ram.read_u8(0x4d) != 1 {
            self.ram.write_u8(0x67, self.ram.read_u8(0xf0) & 0xf);
        }
        self.tile_detect_main_handler(4);
        if self.ram.read_u16(0x59) & 1 == 0 {
            if self.runtime.enhanced_features0() & 4096 != 0 {
                self.ram.write_u8(0x5b, 0);
            }
            if self.ram.read_u8(0x372) != 0 {
                self.link_state_dashing();
                return;
            }
            self.ram.write_u8(0x5e, 0);
            self.link_cancel_dash();
            if self.ram.read_u8(0x3a) & 0x80 == 0 {
                self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            }
            self.ram.write_u8(0x5b, 0);
            let state = if self.ram.read_u8(0x2e0) == 0 {
                0
            } else if self.ram.player().moon_pearl() != 0 {
                28
            } else {
                23
            };
            self.ram.set_player_action_state(state);
            match state {
                23 => self.player_handler_17_bunny(),
                28 => self.link_state_temporary_bunny(),
                _ => self.link_state_default(),
            }
            return;
        }
        self.runtime.player_tile_detect_nearby(self.ram);
        self.ram.write_u8(0x5e, 4);
        let pit = self.ram.read_u16(0x59) as u8 & 0xf;
        if pit == 0 {
            self.ram.write_u8(0x5b, 0);
            self.ram.write_u8(0x5e, 0);
            self.ram
                .set_player_action_state(if self.ram.read_u8(0x2e0) == 0 {
                    0
                } else if self.ram.player().moon_pearl() != 0 {
                    28
                } else {
                    23
                });
            self.link_cancel_dash();
            if self.ram.read_u8(0x3a) & 0x80 == 0 {
                self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
            }
            return;
        }
        if pit != 0xf {
            let mut index = PIT_DIRS.iter().position(|&v| v == pit).map_or_else(
                || {
                    let mut i = 3usize;
                    let mut p = pit;
                    while p & 1 == 0 {
                        i -= 1;
                        p >>= 1;
                    }
                    i
                },
                |i| i + 4,
            );
            self.ram.write_u8(0x2c9, index as u8);
            if self.ram.read_u8(0x67) & DIRS[index] != 0 {
                self.ram.write_u8(0x26, self.ram.read_u8(0x67));
                self.ram.write_u8(0x5e, 6);
                self.link_handle_moving_animation_full_long_entry();
            } else {
                let old = self.ram.read_u8(0x67);
                index = usize::from(self.ram.read_u8(0x2c9));
                self.ram.write_u8(0x67, old | DIRS2[index]);
                if old != 0 {
                    self.link_handle_moving_animation_full_long_entry();
                }
            }
            self.link_handle_diagonal_collision();
            self.link_handle_velocity();
            self.link_handle_cardinal_collision();
            self.apply_links_movement_to_camera();
            return;
        }
        if self.ram.read_u8(0x5b) != 2 {
            if self.ram.player().moon_pearl() != 0 {
                for a in [0x3f7, 0x56, 0x2e0] {
                    self.ram.write_u8(a, 0);
                }
                self.ram.write_u16(0x3f5, 0);
            }
            self.ram.write_u8(0x67, 0);
            self.ram.write_u8(0x5b, 2);
            self.ram.write_u8(0x37b, 1);
            for a in [0x3a, 0x3c, 0x301, 0x37a, 0x46, 0x4d] {
                self.ram.write_u8(a, 0);
            }
            self.runtime.ancilla_sfx3_near(self.ram, 31);
        }
        for a in [0x50, 0x46, 0x24, 0x25, 0x29, 0x4d, 0x373, 0x2e1] {
            self.ram.write_u8(a, 0);
        }
        self.link_force_unequip_cape_quietly();
        self.inc8(0x37b);
        let timer = self.ram.read_u8(0x5c).wrapping_sub(1);
        self.ram.write_u8(0x5c, timer);
        if (timer as i8) >= 0 {
            return;
        }
        let frame = self.inc8(0x5a);
        self.ram.write_u8(0x5c, 9);
        if self.ram.read_u8(0xf3cc) != 13 && frame == 1 {
            self.ram.write_u8(0x2f9, frame);
        }
        if frame == 6 {
            self.link_cancel_dash();
            self.ram.write_u8(0x11, 7);
            self.ram.write_u8(0x5a, 6);
            self.ram.write_u8(0x5b, 3);
            self.ram.write_u8(0x4b, 12);
            self.ram.write_u8(0x57, 16);
            let y = (self.ram.player().y().wrapping_sub(self.ram.read_u16(0xe8))) as u8;
            for a in [0x308, 0x309, 0x376, 0x30b] {
                self.ram.write_u8(a, 0);
            }
            if self.ram.read_u8(0x1b) != 0 {
                self.ram.write_u8(0xa2, self.ram.read_u8(0xa0));
                self.runtime.dungeon_flag_room_data_quadrants(self.ram);
                if self.runtime.dungeon_is_pit_that_hurts_player(self.ram) {
                    self.ram.write_u8(0x11, 20);
                    let hp = self.ram.player().health().wrapping_sub(8);
                    self.ram.set_player_health(if hp >= 0xa8 { 0 } else { hp });
                    return;
                }
            }
            self.ram.write_u8(0xa2, self.ram.read_u8(0xa0));
            self.ram.write_u8(0xa0, self.ram.read_u8(0xc000));
            self.ram.write_u16(0x51, self.ram.player().y());
            self.ram.set_player_y(
                self.ram
                    .player()
                    .y()
                    .wrapping_sub(u16::from(y))
                    .wrapping_sub(0x10),
            );
            if self.ram.read_u8(0x1b) != 0 {
                self.handle_layer_of_destination();
            } else if self.ram.read_u8(0x8a) != 5 {
                self.runtime.overworld_get_pit_destination(self.ram);
                self.ram.write_u8(0x10, 17);
                self.ram.write_u8(0x11, 0);
                self.ram.write_u8(0xb0, 0);
            } else {
                self.runtime.take_damage_from_pit(self.ram);
            }
        }
    }

    pub fn handle_dungeon_landing_from_pit(&mut self) {
        self.runtime.link_oam_main(self.ram);
        self.ram.write_u16(0xfc2, self.ram.player().x());
        self.ram.write_u16(0xfc4, self.ram.player().y());
        if self.ram.read_u8(0x11) == 7 {
            self.ram.write_u8(0x4b, 0);
        }
        if self.ram.read_u8(0x1a) & 3 == 0 && self.inc8(0x5a) == 10 {
            self.ram.write_u8(0x5a, 6);
        }
        self.ram.write_u8(0x67, 4);
        self.link_handle_velocity();
        let y = self.ram.player().y();
        let target = self.ram.read_u16(0x51);
        let landed = if (y as i16) < 0 && (target as i16) >= 0 {
            (target.wrapping_sub(y) as i16) < 0
        } else {
            target < y
        };
        if !landed {
            return;
        }
        if self.runtime.enhanced_features0() & 4096 != 0 {
            self.ram.write_u8(0x47a, 0);
        }
        self.ram.set_player_y(target);
        for a in [0x2e, 0x57, 0x5a, 0x5b, 0x5e, 0xb0, 0x11, 0x37b] {
            self.ram.write_u8(a, 0);
        }
        let follower = self.ram.read_u8(0xf3cc);
        if follower != 0 && follower != 3 {
            self.ram.write_u8(0x2f9, 0);
            if follower == 13 {
                for a in [0xf3cc, 0x4b4, 0x4b5, 0xf3d3] {
                    self.ram.write_u8(a, 0);
                }
            } else {
                self.runtime.follower_initialize(self.ram);
            }
        }
        self.tile_detect_main_handler(0);
        if self.ram.read_u16(0x359) & 1 != 0 {
            self.runtime.ancilla_sfx2_near(self.ram, 0x24);
        }
        self.runtime.player_tile_detect_nearby(self.ram);
        if self.ram.read_u8(0x12e) & 0x3f != 0x24 {
            self.runtime.ancilla_sfx2_near(self.ram, 0x21);
        }
        if self.ram.read_u8(0xad) == 2 && self.ram.read_u16(0x34c) & 0xf != 0 {
            self.ram.write_u8(0x322, 3);
        }
        if self.ram.read_u16(0x341) & 0xf == 0xf {
            self.ram.write_u8(0x345, 1);
            self.ram.write_u8(0x340, self.ram.read_u8(0x26));
            self.link_reset_swimming_state();
            self.ram.write_u8(0xee, 1);
            self.runtime.ancilla_add_splash(self.ram, 0x15, 1);
            self.ram.set_player_action_state(4);
            self.link_force_unequip_cape_quietly();
            for a in [0x308, 0x309, 0x376, 0x5e] {
                self.ram.write_u8(a, 0);
            }
        } else {
            self.ram
                .set_player_action_state(if self.ram.read_u16(0x59) & 0xf != 0 {
                    1
                } else {
                    0
                });
        }
    }

    pub fn link_handle_diagonal_collision(&mut self) {
        if self.check_if_room_needs_double_layer_check() {
            self.player_limit_directions_inner();
            self.create_velocity_from_moving_background()
        }
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & 0xf);
        self.player_limit_directions_inner()
    }
    pub fn player_limit_directions_inner(&mut self) {
        const MASK: [u8; 4] = [7, 0xb, 0xd, 0xe];
        self.ram.write_u8(0x42, 0xf);
        self.ram.write_u8(0x43, 0xf);
        self.ram.write_u8(0x6a, 0);
        let mut dir = self.ram.read_u8(0x67);
        if dir & 0xc != 0 {
            self.inc8(0x6a);
            let toward = if dir & 8 != 0 { 0 } else { 1 };
            self.ram.write_u8(0x66, toward);
            self.runtime
                .tile_detect_movement_vertical_slopes(self.ram, toward);
            let r14 = self.ram.read_u16(14);
            if r14 & 0x30 != 0
                && self.ram.read_u16(0x62) & 2 == 0
                && (((r14 & 0x30) >> 4) as u8 & dir) == 0
                && dir & 3 != 0
            {
                self.ram
                    .write_u8(0x42, MASK[if dir & 2 != 0 { 2 } else { 3 }])
            } else {
                let blocked = (self.ram.read_u8(0x46c) == 0
                    && self.ram.read_u8(0x4d) != 0
                    && self.ram.read_u16(12) & 3 != 0)
                    || r14 & 3 != 0;
                if blocked {
                    self.ram.write_u8(0x6b, 0);
                    if self.ram.read_u8(0x34a) != 0
                        && self.ram.read_u8(0x2e8) & 3 == 0
                        && dir & 3 != 0
                    {
                        self.ram.clear_blocked_swim_axis(SwimAxis::Vertical);
                    }
                    self.ram.write_u8(0x302, 1);
                    self.ram.write_u8(0x42, MASK[toward as usize])
                }
            }
            if dir & 3 != 0 {
                self.inc8(0x6a);
                let toward = if dir & 2 != 0 { 2 } else { 3 };
                self.ram.write_u8(0x66, toward);
                self.runtime
                    .tile_detect_movement_horizontal_slopes(self.ram, toward);
                let r14 = self.ram.read_u16(14);
                if r14 & 0x30 != 0
                    && self.ram.read_u16(0x62) & 2 != 0
                    && (((r14 & 0x30) >> 2) as u8 & dir) == 0
                    && dir & 0xc != 0
                {
                    self.ram
                        .write_u8(0x43, MASK[if dir & 8 != 0 { 0 } else { 1 }])
                } else {
                    let blocked = (self.ram.read_u8(0x46c) == 0
                        && self.ram.read_u8(0x4d) != 0
                        && self.ram.read_u16(12) & 3 != 0)
                        || r14 & 3 != 0;
                    if blocked {
                        self.ram.write_u8(0x6b, 0);
                        if self.ram.read_u8(0x34a) != 0
                            && self.ram.read_u8(0x2e8) & 3 == 0
                            && dir & 0xc != 0
                        {
                            self.ram.clear_blocked_swim_axis(SwimAxis::Horizontal);
                        }
                        self.ram.write_u8(0x302, 1);
                        self.ram.write_u8(0x43, MASK[toward as usize])
                    }
                }
                dir &= self.ram.read_u8(0x42) & self.ram.read_u8(0x43);
                self.ram.write_u8(0x67, dir)
            }
        }
        dir = self.ram.read_u8(0x67);
        let diag = self.ram.read_u8(0x6b);
        if dir & 0xf != 0 && diag & 0xf != 0 {
            self.ram.write_u8(0x67, diag & 0xf)
        }
        if self.ram.read_u8(0x6a) == 2 {
            self.ram.write_u8(
                0x6a,
                if self.ram.player().facing() & 4 != 0 {
                    2
                } else {
                    1
                },
            )
        } else {
            self.ram.write_u8(0x6a, 0)
        }
    }
    pub fn check_if_room_needs_double_layer_check(&mut self) -> bool {
        let c = self.ram.read_u8(0x46c);
        if c == 0 || c == 4 {
            return false;
        }
        if c >= 2 {
            self.ram.set_player_y(
                self.ram.player().y().wrapping_add(
                    self.ram
                        .read_u16(0xe6)
                        .wrapping_sub(self.ram.read_u16(0xe8)),
                ),
            );
            self.ram.write_u16(0x318, self.ram.player().y());
            self.ram.set_player_x(
                self.ram.player().x().wrapping_add(
                    self.ram
                        .read_u16(0xe0)
                        .wrapping_sub(self.ram.read_u16(0xe2)),
                ),
            );
            self.ram.write_u16(0x31a, self.ram.player().x())
        }
        self.ram.write_u8(0xee, 1);
        true
    }
    pub fn create_velocity_from_moving_background(&mut self) {
        if self.ram.read_u8(0x46c) != 1 {
            let x = self.ram.player().x().wrapping_sub(self.ram.read_u16(0x31a));
            let y = self.ram.player().y().wrapping_sub(self.ram.read_u16(0x318));
            self.ram.set_player_y(
                self.ram.player().y().wrapping_add(
                    self.ram
                        .read_u16(0xe8)
                        .wrapping_sub(self.ram.read_u16(0xe6)),
                ),
            );
            self.ram.set_player_x(
                self.ram.player().x().wrapping_add(
                    self.ram
                        .read_u16(0xe2)
                        .wrapping_sub(self.ram.read_u16(0xe0)),
                ),
            );
            if self.ram.read_u8(0x67) != 0 {
                self.ram
                    .set_player_velocity_x(self.ram.player().velocity_x().wrapping_add(x as u8));
                self.ram
                    .set_player_velocity_y(self.ram.player().velocity_y().wrapping_add(y as u8))
            }
        }
        self.ram.write_u8(0xee, 0)
    }
    pub fn link_state_on_ice(&self) {
        panic!("LinkState_OnIce is an unreachable original-code stub")
    }

    pub fn link_handle_change_in_z_velocity(&mut self) {
        self.player_change_z(
            if self.ram.player().action_state() == PlayerState::TurtleRock as u8 {
                1
            } else {
                2
            },
        )
    }

    pub fn player_change_z(&mut self, amount: u8) {
        let velocity = self.ram.player().actual_velocity_z();
        if (velocity as i8) < 0 {
            if self.ram.player().z_low() == 0 {
                return;
            }
            if (self.ram.player().z_low() as i8) < 0 {
                self.ram.set_player_z(0xffff);
                self.ram.set_player_actual_velocity_z(0xff);
                return;
            }
        }
        self.ram
            .set_player_actual_velocity_z(velocity.wrapping_sub(amount))
    }

    pub fn link_set_to_deep_water(&mut self) {
        self.ram.write_u8(0x345, 1);
        self.ram.write_u8(0x340, self.ram.read_u8(0x26));
        self.link_reset_swimming_state();
        self.ram.write_u8(0x376, 0);
        self.ram.write_u8(0x5e, 0)
    }

    pub fn ether_tablet_start_cutscene(&mut self) {
        self.ram.write_u8(0x3c, 0xc0);
        self.ram.write_u8(0x3d, 0);
        self.ram
            .set_player_action_state(PlayerState::ReceivingEther as u8);
        self.ram.write_u8(0x37b, 1);
        self.ram.write_u8(0xffc, 1)
    }

    pub fn link_state_receiving_ether(&mut self) {
        self.ram.write_u8(0x4d, 0);
        self.ram.write_u8(0x46, 0);
        self.ram.write_u8(0x373, 0);
        let timer = self.ram.read_u16(0x3c).wrapping_sub(1);
        self.ram.write_u16(0x3c, timer);
        if (timer as i16) < 0 {
            self.ram.write_u8(0x3c, 0);
            self.ram.write_u8(0x3d, 0)
        } else if timer == 0xbf {
            self.ram.write_u8(0x3ef, 1)
        } else if timer == 160 {
            let x = self.ram.player().x();
            let y = self.ram.player().y();
            self.ram.set_player_x(0x6b0);
            self.ram.set_player_y(0x37);
            self.runtime.ancilla_add_ether_spell(self.ram, 0x18, 0);
            self.ram.set_player_x(x);
            self.ram.set_player_y(y)
        } else if timer == 0 {
            self.runtime.ancilla_add_falling_prize(self.ram, 0x29, 0, 4);
            self.ram.write_u8(0x2e4, 1);
            self.ram.write_u8(0xffc, 0)
        }
    }

    pub fn bombos_tablet_start_cutscene(&mut self) {
        self.ram.write_u8(0x3c, 0xe0);
        self.ram.write_u8(0x3d, 0);
        self.ram
            .set_player_action_state(PlayerState::ReceivingBombos as u8);
        self.ram.write_u8(0x37b, 1);
        self.ram.write_u8(0x112, 1)
    }

    pub fn link_state_receiving_bombos(&mut self) {
        self.ram.write_u8(0x4d, 0);
        self.ram.write_u8(0x46, 0);
        self.ram.write_u8(0x373, 0);
        let timer = self.ram.read_u16(0x3c).wrapping_sub(1);
        self.ram.write_u16(0x3c, timer);
        if (timer as i16) < 0 {
            self.ram.write_u8(0x3c, 0);
            self.ram.write_u8(0x3d, 0)
        } else if timer == 223 {
            self.ram.write_u8(0x3ef, 1)
        } else if timer == 160 {
            let x = self.ram.player().x();
            let y = self.ram.player().y();
            self.ram.set_player_x(0x378);
            self.ram.set_player_y(0xeb0);
            self.runtime.ancilla_add_bombos_spell(self.ram, 0x19, 0);
            self.ram.set_player_x(x);
            self.ram.set_player_y(y)
        } else if timer == 0 {
            self.runtime.ancilla_add_falling_prize(self.ram, 0x29, 5, 4);
            self.ram.write_u8(0x2e4, 1)
        }
    }

    pub fn link_state_reading_desert_tablet(&mut self) {
        let timer = self.ram.read_u8(0x3c).wrapping_sub(1);
        self.ram.write_u8(0x3c, timer);
        if timer == 0 {
            self.ram.set_player_action_state(PlayerState::Ground as u8);
            self.link_perform_desert_prayer()
        }
    }

    pub fn link_perform_desert_prayer(&mut self) {
        self.ram.write_u8(0x11, 5);
        self.ram.write_u8(0x10c, self.ram.read_u8(0x10));
        self.ram.write_u8(0x10, 14);
        self.ram.write_u8(0xfc1, 1);
        self.ram.write_u8(0x30b, 22);
        self.ram.write_u8(0x30a, 0);
        self.ram.write_u8(0x308, 2);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 1);
        self.ram.write_u8(0x2e, 0);
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        self.ram.write_u8(0x12d, 17);
        self.ram.write_u8(0x12c, 242);
    }

    pub fn link_state_bunny_recache(&mut self) {
        self.ram.write_u8(0x3f7, 0);
        self.ram.write_u16(0x3f5, 0);
        if self.ram.player().moon_pearl() != 0 {
            self.ram.write_u8(0x56, 0);
            self.ram.write_u8(0x4d, 0)
        }
        self.ram.write_u8(0x2e, 0);
        self.ram.write_u8(0x2e1, 0);
        self.ram.write_u8(0x50, 0);
        self.link_reset_swimming_state();
        self.ram
            .set_player_action_state(PlayerState::RecoilWall as u8);
        if self.ram.player().moon_pearl() != 0 {
            self.ram.set_player_action_state(PlayerState::Ground as u8);
            self.runtime.load_actual_gear_palettes(self.ram)
        }
    }

    pub fn link_splash_upon_landing(&mut self) {
        if self.ram.read_u8(0x2e0) != 0 {
            if self.ram.read_u8(0x345) != 0 {
                self.runtime.ancilla_add_splash(self.ram, 21, 0);
                self.link_state_bunny_recache();
                return;
            }
            self.ram
                .set_player_action_state(if self.ram.player().moon_pearl() != 0 {
                    PlayerState::TempBunny as u8
                } else {
                    PlayerState::PermaBunny as u8
                })
        } else if self.ram.read_u8(0x345) != 0 {
            if self.ram.player().action_state() != PlayerState::RecoilOther as u8 {
                self.runtime.ancilla_add_splash(self.ram, 21, 0);
            }
            self.link_force_unequip_cape_quietly();
            self.ram
                .set_player_action_state(PlayerState::Swimming as u8)
        } else {
            self.ram.set_player_action_state(PlayerState::Ground as u8)
        }
    }

    pub fn sprite_repel_dash(&mut self) {
        self.ram.write_u8(0x66, self.ram.player().facing() >> 1);
        self.repel_dash()
    }

    pub fn flag67_with_directions(&mut self) {
        let mut direction = 0;
        let y = self.ram.player().actual_velocity_y();
        if y != 0 {
            direction |= if (y as i8) < 0 { 8 } else { 4 }
        }
        let x = self.ram.player().actual_velocity_x();
        if x != 0 {
            direction |= if (x as i8) < 0 { 2 } else { 1 }
        }
        self.ram.write_u8(0x67, direction)
    }

    pub fn dungeon_pit_do_damage(&mut self) {
        self.ram.write_u8(0x11, 20);
        let health = self.ram.player().health().wrapping_sub(8);
        self.ram
            .set_player_health(if health >= 0xa8 { 0 } else { health })
    }

    pub fn link_flag_max_accels(&mut self) {
        if self.ram.read_u8(0x34a) == 0 {
            return;
        }
        for axis in SwimAxis::FORWARD {
            let acceleration = self.ram.swim_axis(axis).speed();
            if acceleration != 0 {
                self.ram.set_swim_speed_limit(axis, acceleration);
                self.ram.set_swim_acceleration_mode(axis, 1)
            }
        }
    }

    pub fn link_set_ice_max_accel(&mut self) {
        if self.ram.read_u8(0x34a) != 0 {
            self.ram.set_swim_speed_limit(SwimAxis::Vertical, 0x180);
            self.ram.set_swim_speed_limit(SwimAxis::Horizontal, 0x180)
        }
    }

    pub fn link_set_momentum(&mut self) {
        let joy = self.ram.read_u8(0xf0) & 0xf;
        let mut mask = 12;
        let mut bit = 8;
        for axis in SwimAxis::FORWARD {
            if joy & mask != 0 {
                let moving = self.ram.read_u8(0x34a);
                self.ram.set_swim_hold_timer(
                    axis,
                    u16::from(if moving != 0 {
                        SWIMMING_TAB2[usize::from(moving - 1)]
                    } else {
                        32
                    }),
                );
                if (self.ram.read_u8(0x340) | self.ram.read_u8(0x67)) & mask == mask {
                    self.ram.set_swim_acceleration_mode(axis, 2)
                } else {
                    self.ram
                        .set_swim_direction_selector(axis, u16::from(joy & bit == 0));
                    self.ram.set_swim_acceleration_mode(axis, 0)
                }
                if self.ram.swim_axis(axis).speed_limit() == 0 {
                    self.ram.set_swim_speed_limit(axis, 240)
                }
            }
            mask >>= 2;
            bit >>= 2;
        }
    }

    pub fn link_handle_swim_movements(&mut self) {
        let mut direction = self.ram.read_u8(0x49) & 0xf;
        if direction == 0 {
            direction = self.ram.read_u8(0xf0) & 0xf
        }
        if direction == 0 {
            self.ram.set_player_velocity_x(0);
            self.ram.set_player_velocity_y(0);
            self.link_flag_max_accels();
            if self.ram.read_u8(0x34a) != 0 {
                if self.ram.read_u8(0x372) != 0 {
                    direction = self.ram.read_u8(0x340)
                } else {
                    if self.ram.swim_axis(SwimAxis::Vertical).speed()
                        | self.ram.swim_axis(SwimAxis::Horizontal).speed()
                        == 0
                    {
                        self.ram.write_u8(0x48, 0);
                        self.link_reset_swimming_state()
                    }
                    self.link_handle_swim_movements_finish();
                    return;
                }
            } else {
                if self.ram.player().action_state() != PlayerState::Swimming as u8 {
                    self.ram.write_u8(0x2e, 0)
                }
                self.link_handle_swim_movements_finish();
                return;
            }
        }
        if direction != self.ram.read_u8(0x340) {
            self.ram.write_u8(0x340, direction);
            self.ram.set_player_subpixel_x(0);
            self.ram.set_player_subpixel_y(0);
            self.ram.write_u8(0x6b, 0);
            self.ram.write_u8(0x48, 0)
        }
        self.link_set_ice_max_accel();
        self.link_set_momentum();
        self.link_set_the_max_accel();
        self.link_handle_swim_movements_finish()
    }

    fn link_handle_swim_movements_finish(&mut self) {
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.ram.write_u8(0x302, 0);
        self.handle_indoor_camera_and_doors()
    }

    pub fn link_handle_swim_accels(&mut self) {
        const ACCELERATION_STEPS: [u16; 9] = [128, 160, 192, 224, 256, 288, 320, 352, 384];
        let mut mask = 12;
        for axis in SwimAxis::FORWARD {
            if self.ram.read_u8(0xf0) & mask != 0 {
                let velocity = self.ram.swim_axis(axis).speed();
                let maximum = self.ram.swim_axis(axis).speed_limit();
                if velocity != 0 && maximum >= 384 {
                    let mut step = ACCELERATION_STEPS[8];
                    for candidate in ACCELERATION_STEPS {
                        step = candidate;
                        if candidate >= velocity {
                            break;
                        }
                    }
                    self.ram.set_swim_speed_limit(axis, step)
                } else if maximum != 0 {
                    self.ram
                        .set_swim_speed_limit(axis, maximum.wrapping_add(160).min(384))
                } else {
                    self.ram.set_swim_speed(axis, 1);
                    self.ram.set_swim_speed_limit(axis, 240)
                }
            }
            mask >>= 2;
        }
    }

    pub fn link_set_the_max_accel(&mut self) {
        if self.ram.read_u8(0x34a) != 0 || self.ram.read_u8(0x34f) != 0 {
            return;
        }
        let mut mask = 12;
        for axis in SwimAxis::FORWARD {
            if self.ram.read_u8(0xf0) & mask != 0
                && self.ram.swim_axis(axis).acceleration_mode() != 2
            {
                let velocity = self.ram.swim_axis(axis).speed();
                if self.ram.swim_axis(axis).acceleration_latch() != 0
                    || (velocity >= 240 && velocity >= self.ram.swim_axis(axis).speed_limit())
                {
                    self.ram.set_swim_acceleration_mode(axis, 0);
                    if velocity >= 240 {
                        self.ram.set_swim_acceleration_latch(axis, 1);
                        self.ram.set_swim_acceleration_mode(axis, 1)
                    } else {
                        self.ram.set_swim_speed_limit(axis, 240);
                        self.ram.set_swim_acceleration_latch(axis, 0)
                    }
                }
            } else {
                self.ram.set_swim_speed_limit(axis, 240);
                self.ram.set_swim_acceleration_latch(axis, 0)
            }
            mask >>= 2;
        }
    }

    pub fn link_reset_state_after_damaging_pit(&mut self) {
        self.link_reset_swimming_state();
        self.ram.set_player_action_state(
            if self.ram.read_u8(0x56) != 0 && self.ram.player().moon_pearl() == 0 {
                PlayerState::PermaBunny as u8
            } else {
                PlayerState::Ground as u8
            },
        );
        self.ram.write_u8(0x26, self.ram.read_u8(0x340));
        self.ram.write_u8(0x345, 0);
        self.ram.write_u8(0x37b, 0);
        self.ram.write_u8(0x5a, 0);
        self.ram.write_u8(0x5b, 0)
    }

    pub fn link_reset_boomerang_y_stuff(&mut self) {
        self.ram.write_u8(0x301, 0);
        self.ram.write_u8(0x300, 0);
        self.ram.write_u8(0x3d, 0);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        if self.ram.read_u8(0x3a) & 0x80 == 0 {
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1)
        }
    }

    pub fn check_y_button_press(&mut self) -> bool {
        if self.ram.read_u8(0x3a) & 0x40 != 0
            || self.ram.read_u8(0x46) != 0
            || self.ram.read_u8(0xf4) & 0x40 == 0
        {
            false
        } else {
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) | 0x40);
            true
        }
    }

    pub fn link_check_magic_cost(&mut self, item: u8) -> bool {
        let cost =
            LINK_ITEM_MAGIC_COSTS[usize::from(item) * 3 + usize::from(self.ram.read_u8(0xf37b))];
        let magic = self.ram.player().magic();
        if magic != 0 {
            let remaining = magic.wrapping_sub(cost);
            if remaining < 0x80 {
                self.ram.set_player_magic(remaining);
                return true;
            }
        }
        if item != 3 {
            self.runtime.ancilla_sfx2_near(self.ram, 60);
            self.ram.write_u16(0x1cf0, 123);
            self.runtime.main_show_text_message(self.ram)
        }
        false
    }

    pub fn refund_magic(&mut self, item: u8) {
        let cost =
            LINK_ITEM_MAGIC_COSTS[usize::from(item) * 3 + usize::from(self.ram.read_u8(0xf37b))];
        let mut magic = u16::from(self.ram.player().magic()) + u16::from(cost);
        if self.runtime.enhanced_features0() & 4096 != 0 && magic >= 128 {
            magic = 128
        }
        self.ram.set_player_magic(magic as u8)
    }

    pub fn link_item_reset_from_overworld_things(&mut self) {
        self.ram.write_u8(0x30a, 0);
        self.ram.write_u8(0x3b, 0);
        self.ram.write_u8(0x308, 0);
        self.ram.write_u8(0x309, 0);
        self.ram.write_u8(0x376, 0);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1)
    }

    pub fn link_check_new_a_press(&mut self) -> bool {
        if self.ram.read_u8(0x3b) & 0x80 != 0
            || self.ram.read_u8(0x46) != 0
            || self.ram.read_u8(0xf6) & 0x80 == 0
        {
            false
        } else {
            self.ram.write_u8(0x3b, self.ram.read_u8(0x3b) | 0x80);
            true
        }
    }

    pub fn link_handle_toss(&mut self) -> bool {
        if self.ram.read_u8(0x3b) & 0x80 == 0
            || self.ram.read_u8(0xf6) & 0x80 == 0
            || self.ram.read_u8(0x309) & 1 != 0
        {
            return false;
        }
        self.ram.write_u8(0x30d, 0);
        self.ram.write_u8(0x30e, 0);
        self.ram.write_u8(0x30a, 0);
        self.ram.write_u8(0x3b, 0);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
        true
    }

    pub fn link_handle_velocity_and_sand_drag(&mut self, old_x: u16, old_y: u16) {
        self.ram
            .set_player_y(self.ram.player().y().wrapping_add(self.ram.read_u16(0xb7e)));
        self.ram
            .set_player_x(self.ram.player().x().wrapping_add(self.ram.read_u16(0xb7c)));
        self.ram
            .set_player_velocity_y(self.ram.player().y().wrapping_sub(old_y) as u8);
        self.ram
            .set_player_velocity_x(self.ram.player().x().wrapping_sub(old_x) as u8)
    }

    pub fn link_apply_moving_floor_velocity(&mut self) {
        self.ram.write_u8(0x6a, 0);
        self.ram
            .set_player_y(self.ram.player().y().wrapping_add(self.ram.read_u16(0x310)));
        self.ram
            .set_player_x(self.ram.player().x().wrapping_add(self.ram.read_u16(0x312)))
    }

    pub fn link_handle_recoiling(&mut self) {
        self.ram.write_u8(0x67, 0);
        let y = self.ram.player().actual_velocity_y();
        if y != 0 {
            let direction = if (y as i8) < 0 { 8 } else { 4 };
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) | direction);
            self.ram.write_u8(0x26, self.ram.read_u8(0x67));
            self.player_handle_incapacitated_inner2()
        }
        let x = self.ram.player().actual_velocity_x();
        if x != 0 {
            let direction = if (x as i8) < 0 { 2 } else { 1 };
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) | direction);
            self.ram.write_u8(0x26, self.ram.read_u8(0x67))
        }
        self.player_handle_incapacitated_inner2()
    }

    pub fn player_handle_incapacitated_inner2(&mut self) {
        let diagonal = self.ram.read_u8(0x6b);
        if diagonal & 0xc != 0
            && diagonal & 3 != 0
            && self.ram.player().action_state() == PlayerState::RecoilWall as u8
        {
            self.ram.set_player_actual_velocity_x(
                0u8.wrapping_sub(self.ram.player().actual_velocity_x()),
            );
            self.ram.set_player_actual_velocity_y(
                0u8.wrapping_sub(self.ram.player().actual_velocity_y()),
            )
        }
        if self.ram.read_u8(0x6c) == 1 {
            self.ram.write_u8(0x26, self.ram.read_u8(0x26) & 0xc);
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & 0xc);
            self.ram.set_player_actual_velocity_x(0)
        } else if self.ram.read_u8(0x6c) == 2 {
            self.ram.write_u8(0x26, self.ram.read_u8(0x26) & 3);
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & 3);
            self.ram.set_player_actual_velocity_y(0)
        }
    }

    fn save_safe_return_coordinates(&mut self) {
        let y = self.ram.player().y();
        let x = self.ram.player().x();
        self.ram.write_u8(0x3e, y as u8);
        self.ram.write_u8(0x40, (y >> 8) as u8);
        self.ram.write_u8(0x3f, x as u8);
        self.ram.write_u8(0x41, (x >> 8) as u8)
    }

    pub fn link_handle_velocity(&mut self) {
        const SPEED_MOD: [u8; 27] = [
            24, 16, 10, 24, 16, 8, 8, 4, 12, 16, 9, 25, 20, 13, 16, 8, 64, 42, 16, 8, 4, 2, 48, 24,
            32, 21, 0,
        ];

        if (self.ram.read_u8(0x11) == 2 && self.ram.read_u8(0x10) == 14)
            || self.ram.read_u8(0xb7b) != 0
        {
            let x = self.ram.player().x();
            let y = self.ram.player().y();
            self.save_safe_return_coordinates();
            self.link_handle_velocity_and_sand_drag(x, y);
            return;
        }
        if self.ram.player().action_state() == PlayerState::Swimming as u8 {
            self.handle_swim_stroke_and_subpixels();
            return;
        }

        let mut speed_index;
        if self.ram.read_u8(0x34a) != 0 {
            if self.ram.read_u8(0x372) == 0 {
                self.handle_swim_stroke_and_subpixels();
                return;
            }
            speed_index = 24;
        } else {
            if self.ram.read_u8(0x372) != 0 {
                self.ram.write_u8(0x57, 0);
                debug_assert!(self.ram.read_u8(0x2f1) >= 32);
            }
            if self.ram.read_u8(0x316) | self.ram.read_u8(0x317) == 0xf {
                return;
            }
            speed_index = self.ram.read_u8(0x5e);
            if self.ram.read_u8(0x351) != 0 {
                speed_index = match speed_index {
                    16 => 22,
                    12 => 14,
                    _ => 12,
                }
            }
        }

        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_y(0);
        self.ram.write_u8(0x68, 0);
        self.ram.write_u8(0x69, 0);
        if self.ram.read_u8(0x67) & 0xc != 0 && self.ram.read_u8(0x67) & 3 != 0 {
            speed_index = speed_index.wrapping_add(1)
        }

        if self.ram.read_u8(0x5b) != 0 {
            if self.ram.read_u8(0x5b) == 3 {
                let modifier = self.ram.read_u8(0x57);
                self.ram.write_u8(
                    0x57,
                    if modifier < 48 {
                        modifier.wrapping_add(8)
                    } else {
                        32
                    },
                )
            }
        } else if self.ram.read_u8(0x57) != 0 {
            speed_index = if matches!(self.ram.read_u8(0x11), 8 | 16) {
                10
            } else {
                2
            };
            let modifier = self.ram.read_u8(0x57);
            if modifier != 1 {
                if modifier < 16 {
                    self.ram.write_u8(0x57, modifier.wrapping_add(1));
                    speed_index = 26
                } else {
                    self.ram.write_u8(0x57, 0);
                    self.ram.write_u8(0x5e, 0)
                }
            }
        }

        let velocity = self
            .ram
            .read_u8(0x57)
            .wrapping_add(SPEED_MOD[speed_index as usize]);
        let direction = self.ram.read_u8(0x67);
        if direction & 3 != 0 {
            self.ram
                .set_player_actual_velocity_x(if direction & 2 != 0 {
                    0u8.wrapping_sub(velocity)
                } else {
                    velocity
                })
        }
        if direction & 0xc != 0 {
            self.ram
                .set_player_actual_velocity_y(if direction & 8 != 0 {
                    0u8.wrapping_sub(velocity)
                } else {
                    velocity
                })
        }
        self.ram.set_player_actual_velocity_z(0xff);
        self.ram.set_player_z(0xffff);
        self.ram.set_player_subpixel_z(0);
        self.link_move_position()
    }

    pub fn link_move_position(&mut self) {
        let old_x = self.ram.player().x();
        let old_y = self.ram.player().y();
        self.save_safe_return_coordinates();
        if self.ram.player().action_state() != PlayerState::Quake as u8
            && self.ram.read_u8(0x2f5) == 2
        {
            self.link_handle_velocity_and_sand_drag(old_x, old_y);
            return;
        }

        let x_fixed = i32::from(self.ram.player().x()) * 256
            + i32::from(self.ram.player().subpixel_x())
            + i32::from(self.ram.player().actual_velocity_x() as i8) * 16;
        self.ram.set_player_subpixel_x(x_fixed as u8);
        self.ram.set_player_x(((x_fixed as u32) >> 8) as u16);
        let y_fixed = i32::from(self.ram.player().y()) * 256
            + i32::from(self.ram.player().subpixel_y())
            + i32::from(self.ram.player().actual_velocity_y() as i8) * 16;
        self.ram.set_player_subpixel_y(y_fixed as u8);
        self.ram.set_player_y(((y_fixed as u32) >> 8) as u16);
        if self.ram.read_u8(0x4d) != 0 {
            let z_fixed = i32::from(self.ram.player().z()) * 256
                + i32::from(self.ram.player().subpixel_z())
                + i32::from(self.ram.player().actual_velocity_z() as i8) * 16;
            self.ram.set_player_subpixel_z(z_fixed as u8);
            self.ram.set_player_z(((z_fixed as u32) >> 8) as u16)
        }
        self.link_handle_moving_floor();
        self.link_apply_conveyor();
        self.link_handle_velocity_and_sand_drag(old_x, old_y)
    }

    pub fn handle_swim_stroke_and_subpixels(&mut self) {
        const SWIMMING_ACCELERATION: [i8; 12] = [8, -12, -8, -16, 4, -6, -12, -6, 10, -16, -12, -6];
        const DIRECTION_MASK: [u8; 2] = [!0xc, !3];
        const DIRECTION_BIT: [u8; 4] = [8, 4, 2, 1];

        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_y(0);
        let mut velocity = [0u16; 2];
        for axis in SwimAxis::REVERSE {
            let countdown = self.ram.swim_axis(axis).hold_timer().wrapping_sub(1);
            self.ram.set_swim_hold_timer(axis, countdown);
            if (countdown as i16) < 0 {
                self.ram.set_swim_hold_timer(axis, 0);
                self.ram.set_swim_acceleration_mode(axis, 1)
            }
            let mut table_index = self.ram.swim_axis(axis).acceleration_mode();
            if self.ram.read_u8(0x34a) != 0 {
                table_index =
                    table_index.wrapping_add(u16::from(self.ram.read_u8(0x34a)).wrapping_mul(4))
            }
            let mut sum = self
                .ram
                .swim_axis(axis)
                .speed()
                .wrapping_add(SWIMMING_ACCELERATION[table_index as usize] as i16 as u16);
            if (sum as i16) <= 0 {
                let direction = self.ram.read_u8(0x67) & DIRECTION_MASK[axis.index()];
                self.ram.write_u8(0x67, direction);
                self.ram.write_u8(0x26, direction);
                if self.ram.swim_axis(axis).acceleration_mode() == 2 {
                    self.ram.set_swim_acceleration_mode(axis, 0);
                    self.ram.set_swim_speed_limit(axis, 240);
                    self.ram.set_swim_speed(axis, 2)
                } else {
                    self.ram.set_swim_acceleration_mode(axis, 0);
                    self.ram.set_swim_speed_limit(axis, 0);
                    self.ram.set_swim_speed(axis, 0)
                }
            } else {
                let direction_index =
                    self.ram.swim_axis(axis).direction_selector() as usize + axis.index() * 2;
                self.ram.write_u8(
                    0x67,
                    self.ram.read_u8(0x67) | DIRECTION_BIT[direction_index],
                );
                if sum >= self.ram.swim_axis(axis).speed_limit() {
                    sum = self.ram.swim_axis(axis).speed_limit()
                }
                self.ram.set_swim_speed(axis, sum)
            }
            velocity[axis.index()] = self.ram.swim_axis(axis).speed();
            if self.ram.read_u8(0x6a) | self.ram.read_u8(0x6b) != 0 {
                velocity[axis.index()] =
                    velocity[axis.index()].wrapping_sub(velocity[axis.index()] >> 2)
            }
            if self.ram.swim_axis(axis).direction_selector() == 0 {
                velocity[axis.index()] = 0u16.wrapping_sub(velocity[axis.index()])
            }
        }
        self.player_something_with_velocity_tired_or_swim(velocity[1], velocity[0])
    }

    pub fn player_something_with_velocity_tired_or_swim(
        &mut self,
        x_velocity: u16,
        y_velocity: u16,
    ) {
        let old_x = self.ram.player().x();
        let old_y = self.ram.player().y();
        self.save_safe_return_coordinates();

        let x_fixed = i32::from(old_x) * 256
            + i32::from(self.ram.player().subpixel_x())
            + i32::from(x_velocity as i16);
        self.ram.set_player_subpixel_x(x_fixed as u8);
        self.ram.set_player_x(((x_fixed as u32) >> 8) as u16);
        let x_high = (x_velocity >> 8) as u8;
        self.ram.set_player_actual_velocity_x(
            (if (x_high as i8) < 0 {
                0u8.wrapping_sub(x_high)
            } else {
                x_high
            })
            .wrapping_shl(4)
                | (x_velocity as u8 >> 4),
        );

        let y_fixed = i32::from(old_y) * 256
            + i32::from(self.ram.player().subpixel_y())
            + i32::from(y_velocity as i16);
        self.ram.set_player_subpixel_y(y_fixed as u8);
        self.ram.set_player_y(((y_fixed as u32) >> 8) as u16);
        let y_high = (y_velocity >> 8) as u8;
        self.ram.set_player_actual_velocity_y(
            (if (y_high as i8) < 0 {
                0u8.wrapping_sub(y_high)
            } else {
                y_high
            })
            .wrapping_shl(4)
                | (y_velocity as u8 >> 4),
        );

        if self.ram.read_u8(0x46c) == 4 {
            self.link_apply_moving_floor_velocity()
        }
        self.ram.write_u8(0x69, 0);
        self.ram.write_u8(0x68, 0);
        self.link_handle_velocity_and_sand_drag(old_x, old_y)
    }

    pub fn link_handle_moving_floor(&mut self) {
        if self.ram.read_u8(0x46c) == 0
            || !matches!(self.ram.player().z_low(), 0 | 0xff)
            || self.ram.read_u8(0x322) & 3 != 3
            || self.ram.player().action_state() == PlayerState::Hookshot as u8
        {
            return;
        }
        let floor_y = self.ram.read_u16(0x310);
        if floor_y != 0 {
            self.ram.write_u8(
                0x67,
                self.ram.read_u8(0x67) | if (floor_y as u8 as i8) < 0 { 8 } else { 4 },
            )
        }
        let floor_x = self.ram.read_u16(0x312);
        if floor_x != 0 {
            self.ram.write_u8(
                0x67,
                self.ram.read_u8(0x67) | if (floor_x as u8 as i8) < 0 { 2 } else { 1 },
            )
        }
        self.link_apply_moving_floor_velocity()
    }

    pub fn link_apply_conveyor(&mut self) {
        const DIRECTION_FLAG: [u8; 4] = [8, 4, 2, 1];
        const BELT_Y: [i8; 4] = [-8, 8, 0, 0];
        const BELT_X: [i8; 4] = [0, 0, -8, 8];

        let conveyor = self.ram.read_u8(0x3f3);
        if conveyor == 0
            || !matches!(self.ram.player().z_low(), 0 | 0xff)
            || self.ram.read_u8(0x376) & 1 != 0
            || self.ram.player().action_state() == PlayerState::Hookshot as u8
            || self.ram.read_u8(0x4d) != 0
        {
            return;
        }
        let index = usize::from(conveyor - 1);
        if self.ram.read_u8(0x372) != 0
            && self.ram.read_u8(0x2f1) == 32
            && self.ram.read_u8(0x67) & DIRECTION_FLAG[index] != 0
        {
            return;
        }
        self.ram.write_u8(0x6a, 0);
        self.ram
            .write_u8(0x67, self.ram.read_u8(0x67) | DIRECTION_FLAG[index]);
        let y_fixed = i32::from(self.ram.player().y()) * 256
            + i32::from(self.ram.read_u8(0x41c))
            + i32::from(BELT_Y[index]) * 16;
        self.ram.write_u8(0x41c, y_fixed as u8);
        self.ram.set_player_y(((y_fixed as u32) >> 8) as u16);
        let x_fixed = i32::from(self.ram.player().x()) * 256
            + i32::from(self.ram.read_u8(0x41d))
            + i32::from(BELT_X[index]) * 16;
        self.ram.write_u8(0x41d, x_fixed as u8);
        self.ram.set_player_x(((x_fixed as u32) >> 8) as u16)
    }

    pub fn link_handle_moving_animation_full_long_entry(&mut self) {
        const FACING_BITS: [u8; 4] = [8, 4, 2, 1];
        if self.ram.player().action_state() == PlayerState::Swimming as u8 {
            self.link_handle_moving_animation_swimming();
            return;
        }
        let mut direction = self.ram.read_u8(0x26);
        if direction == 0 {
            return;
        }
        if self.ram.read_u8(0x34a) != 0 {
            direction = self.ram.read_u8(0x340)
        }
        if self.ram.read_u8(0x50) == 0 {
            let facing = if self.ram.read_u8(0x6a) == 0 {
                if direction & 0xc != 0 { 0 } else { 4 }
            } else if self.ram.read_u8(0x6c) != 0 {
                self.ram.read_u8(0x6c).wrapping_mul(2) & !3
            } else {
                if direction & FACING_BITS[usize::from(self.ram.player().facing() >> 1)] != 0 {
                    self.link_handle_moving_animation_start_with_dash();
                    return;
                }
                if direction & 0xc != 0 { 0 } else { 4 }
            };
            let facing = if facing != 4 {
                facing + if direction & 4 != 0 { 2 } else { 0 }
            } else {
                facing + if direction & 1 != 0 { 2 } else { 0 }
            };
            self.ram.set_player_facing(facing)
        }
        self.link_handle_moving_animation_start_with_dash()
    }

    pub fn link_handle_moving_animation_start_with_dash(&mut self) {
        const SLOW_DELAYS: [u8; 16] = [4, 4, 4, 4, 1, 1, 1, 1, 2, 2, 2, 2, 8, 8, 8, 8];
        const WALK_DELAYS: [u8; 24] = [
            1, 2, 3, 2, 2, 2, 3, 2, 1, 1, 2, 1, 1, 1, 2, 1, 2, 2, 3, 2, 2, 2, 3, 2,
        ];
        if self.ram.read_u8(0x372) != 0 {
            self.link_handle_moving_animation_dash();
            return;
        }
        let mut delay_index = self.ram.player().facing() >> 1;
        if self.ram.read_u8(0x5e) == 6 {
            delay_index = delay_index.wrapping_add(4)
        } else if self.ram.read_u8(0x34a) != 0 {
            if self.ram.read_u8(0xf0) & 0xf == 0 {
                self.ram.write_u8(0x2e, 0);
                return;
            }
            delay_index = delay_index.wrapping_add(4)
        }

        let state = self.ram.player().action_state();
        if state == PlayerState::PermaBunny as u8
            || (self.runtime.enhanced_features0() & 4096 != 0
                && state == PlayerState::TempBunny as u8)
        {
            if self.ram.read_u8(0x2e) < 4 && self.ram.read_u8(0x2f5) != 2 {
                let counter = self.inc8(0x2d);
                if counter >= SLOW_DELAYS[delay_index as usize] {
                    self.ram.write_u8(0x2d, 0);
                    if self.inc8(0x2e) == 4 {
                        self.ram.write_u8(0x2e, 0)
                    }
                }
            } else {
                self.ram.write_u8(0x2e, 0)
            }
            return;
        }

        if matches!(self.ram.read_u8(0x11), 18 | 19) {
            delay_index = 12
        } else if self.ram.read_u8(0x11) != PlayerState::JumpOffLedgeDiag as u8
            && self.ram.read_u8(0x308) & 0x80 == 0
        {
            if self.ram.read_u8(0x48) & 0x8d != 0 {
                delay_index = 12
            } else if self.ram.read_u8(0x351) == 0 && self.ram.read_u8(0x3c) == 0 {
                let mut index = self.ram.read_u8(0x2e);
                if self.ram.read_u8(0x5e) == 6 {
                    index = index.wrapping_add(8)
                }
                if self.ram.read_u8(0x34a) != 0 {
                    index = index.wrapping_add(8)
                }
                if self.ram.read_u8(0x2f5) == 2 {
                    return;
                }
                if self.inc8(0x2d) >= WALK_DELAYS[index as usize] {
                    self.ram.write_u8(0x2d, 0);
                    if self.inc8(0x2e) == 9 {
                        self.ram.write_u8(0x2e, 1)
                    }
                }
                return;
            }
        }

        if self.ram.read_u8(0x2e) < 6 && self.ram.read_u8(0x2f5) != 2 {
            if self.inc8(0x2d) >= SLOW_DELAYS[delay_index as usize] {
                self.ram.write_u8(0x2d, 0);
                if self.inc8(0x2e) == 6 {
                    self.ram.write_u8(0x2e, 0)
                }
            }
        } else {
            self.ram.write_u8(0x2e, 0)
        }
    }

    pub fn link_handle_moving_animation_swimming(&mut self) {
        const FACING_BITS: [u8; 4] = [8, 4, 2, 1];
        let direction = self.ram.read_u8(0x340);
        if direction == 0 || self.ram.read_u8(0x50) != 0 {
            return;
        }
        let mut facing;
        if self.ram.read_u8(0x6a) != 0 {
            if self.ram.read_u8(0x6c) != 0 {
                facing = self.ram.read_u8(0x6c).wrapping_mul(2) & !3
            } else {
                if direction & FACING_BITS[usize::from(self.ram.player().facing() >> 1)] != 0 {
                    return;
                }
                facing = if direction & 0xc != 0 { 0 } else { 4 }
            }
        } else {
            facing = if direction & 0xc != 0 { 0 } else { 4 }
        }
        if facing != 4 {
            facing = facing.wrapping_add(if direction & 4 != 0 { 2 } else { 0 })
        } else {
            facing = facing.wrapping_add(if direction & 1 != 0 { 2 } else { 0 })
        }
        self.ram.set_player_facing(facing)
    }

    pub fn link_handle_moving_animation_dash(&mut self) {
        const DASH_THRESHOLDS: [u8; 7] = [48, 36, 24, 16, 12, 8, 4];
        const DASH_DELAYS: [u8; 56] = [
            3, 3, 5, 3, 3, 3, 5, 3, 2, 2, 4, 2, 2, 2, 4, 2, 2, 2, 3, 2, 2, 2, 3, 2, 1, 1, 2, 1, 1,
            1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const FAST_DELAYS: [u8; 7] = [1, 2, 2, 2, 2, 2, 2];
        let mut tier = 6usize;
        while self.ram.read_u8(0x374) >= DASH_THRESHOLDS[tier] && tier != 0 {
            tier -= 1
        }
        if self.ram.read_u8(0x3c) < 9 && self.ram.read_u8(0x351) == 0 {
            if self.inc8(0x2d) >= DASH_DELAYS[tier * 8] {
                self.ram.write_u8(0x2d, 0);
                if self.inc8(0x2e) == 9 {
                    self.ram.write_u8(0x2e, 1)
                }
            }
        } else if self.inc8(0x2d) >= FAST_DELAYS[tier] {
            self.ram.write_u8(0x2d, 0);
            if self.inc8(0x2e) >= 6 {
                self.ram.write_u8(0x2e, 0)
            }
        }
    }

    pub fn handle_indoor_camera_and_doors(&mut self) {
        if self.ram.read_u8(0x1b) != 0 {
            if self.ram.read_u8(0x6c) != 0 {
                self.handle_door_transitions()
            } else {
                self.apply_links_movement_to_camera()
            }
        }
    }

    pub fn handle_door_transitions(&mut self) {
        self.ram.write_u8(0x69, 0);
        self.ram.write_u8(0x68, 0);
        if self.runtime.enhanced_features0() & 4096 != 0
            && !(self.ram.read_u8(0x10) == 7 && self.ram.read_u8(0x11) == 0)
        {
            return;
        }

        let last_direction = self.ram.read_u8(0x26);
        if last_direction & 0xc != 0 && self.ram.read_u8(0x6c) == 1 {
            if last_direction & 4 != 0 {
                let position = self.ram.player().y().wrapping_add(28);
                if position & 0xfc == 0 {
                    self.ram.write_u8(
                        0x68,
                        (position >> 8).wrapping_sub(u16::from(self.ram.read_u8(0x40))) as u8,
                    )
                }
            } else {
                let position = self.ram.player().y().wrapping_sub(18);
                self.ram.write_u8(
                    0x68,
                    (position >> 8).wrapping_sub(u16::from(self.ram.read_u8(0x40))) as u8,
                )
            }
        }
        if last_direction & 3 != 0 && self.ram.read_u8(0x6c) == 2 {
            if last_direction & 1 != 0 {
                let position = self.ram.player().x().wrapping_add(21);
                if position & 0xfc == 0 {
                    self.ram.write_u8(
                        0x69,
                        (position >> 8).wrapping_sub(u16::from(self.ram.read_u8(0x41))) as u8,
                    )
                }
            } else {
                let position = self.ram.player().x().wrapping_sub(8);
                self.ram.write_u8(
                    0x69,
                    (position >> 8).wrapping_sub(u16::from(self.ram.read_u8(0x41))) as u8,
                )
            }
        }

        let (delta, negative_direction, positive_direction) = if self.ram.read_u8(0x69) != 0 {
            (self.ram.read_u8(0x69), 1, 0)
        } else if self.ram.read_u8(0x68) != 0 {
            (self.ram.read_u8(0x68), 3, 2)
        } else {
            return;
        };
        self.ram.write_u8(0x30b, 0);
        self.ram.write_u8(0x308, 0);
        self.ram.write_u8(0x309, 0);
        self.ram.write_u8(0x376, 0);
        self.runtime.dungeon_start_inter_room_transition(
            self.ram,
            if (delta as i8) < 0 {
                negative_direction
            } else {
                positive_direction
            },
        )
    }

    pub fn apply_links_movement_to_camera(&mut self) {
        self.applied_movement_to_camera = true;
        self.ram.write_u8(
            0x68,
            (self.ram.player().y() >> 8).wrapping_sub(u16::from(self.ram.read_u8(0x40))) as u8,
        );
        self.ram.write_u8(
            0x69,
            (self.ram.player().x() >> 8).wrapping_sub(u16::from(self.ram.read_u8(0x41))) as u8,
        );
        let x_delta = self.ram.read_u8(0x69);
        if x_delta != 0 {
            self.runtime
                .adjust_quadrant_and_camera(self.ram, if (x_delta as i8) < 0 { 1 } else { 0 })
        }
        let y_delta = self.ram.read_u8(0x68);
        if y_delta != 0 {
            self.runtime
                .adjust_quadrant_and_camera(self.ram, if (y_delta as i8) < 0 { 3 } else { 2 })
        }
    }

    pub fn link_initialize(&mut self) {
        self.ram.set_player_facing(2);
        for address in [
            0x26, 0x301, 0x37a, 0x20b, 0x350, 0x30d, 0x30e, 0x30a, 0x2e1, 0x3b, 0x308, 0x309, 0x376,
        ] {
            self.ram.write_u8(address, 0)
        }
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
        self.link_reset_swimming_state();
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
        self.ram.set_player_z(self.ram.player().z() & 0xff);
        for address in [0x4d, 0x46, 0x31f, 0x360, 0x2da, 0x55] {
            self.ram.write_u8(address, 0)
        }
        self.link_force_unequip_cape_quietly();
        self.link_reset_sword_and_item_usage();
        self.ram.write_u8(0x37b, 0);
        self.ram.write_u8(0x300, 0);
        self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
        self.ram.write_u8(0x2f5, 0);
        self.ram.write_u8(0x79, 0);

        if self.runtime.enhanced_features0() & 4096 != 0 {
            self.ram.write_u8(0x47a, 0);
            self.ram.write_u8(0x2fa, 0);
            self.ram.write_u8(0x3f3, 0);
            self.ram.write_u8(0x34a, 0);
            self.ram.write_u16(0x11c, 0);
            self.ram.write_u16(0x11a, 0);
            if self.ram.player().moon_pearl() == 0 && self.ram.read_u8(0xf3ca) != 0 {
                self.ram
                    .set_player_action_state(PlayerState::PermaBunny as u8);
                self.ram.write_u8(0x56, 1);
                self.ram.write_u8(0x2e0, 1);
                self.runtime.load_gear_palettes_bunny(self.ram)
            }
        }
    }

    pub fn link_reset_properties_a(&mut self) {
        self.ram.write_u8(0x26, 0);
        self.ram.write_u8(0x67, 0);
        self.ram.write_u8(0x34a, 0);
        self.link_reset_swimming_state();
        for address in [
            0x2e1, 0x31f, 0x3db, 0x56, 0x2e0, 0x3f5, 0x3f7, 0x3fc, 0x3f8, 0x3fa, 0x3e9, 0x373,
            0x31e, 0x2f2, 0x2f8, 0x2ea, 0x2e9, 0x2db,
        ] {
            self.ram.write_u8(address, 0)
        }
        self.link_reset_properties_b()
    }

    pub fn link_reset_properties_b(&mut self) {
        for address in [0x2f5, 0x79, 0x302, 0x2f4, 0x48, 0x5a, 0x5b] {
            self.ram.write_u8(address, 0)
        }
        self.link_reset_properties_c()
    }

    pub fn link_reset_properties_c(&mut self) {
        if self.runtime.enhanced_features0() & 4096 != 0 {
            self.ram.write_u8(0x112, 0)
        }
        for address in [
            0x36c, 0x31c, 0x31d, 0x315, 0x3ef, 0x2e3, 0x301, 0x37a, 0x20b, 0x350, 0x30d, 0x30e,
            0x30a, 0x3b, 0x3a, 0x3c, 0x308, 0x309, 0x376, 0x50, 0x4d, 0x46, 0x360, 0x2da, 0x55,
        ] {
            self.ram.write_u8(address, 0)
        }
        self.ram.write_u16(0x2f6, 0);
        self.link_reset_sword_and_item_usage();
        for address in [0x37b, 0x300, 0x37e, 0x2ec, 0x314, 0x3f8, 0x2fa] {
            self.ram.write_u8(address, 0)
        }
    }

    pub fn link_handle_bunny_transformation(&mut self) -> bool {
        if self.ram.read_u16(0x3f5) == 0 {
            return false;
        }
        if self.ram.read_u8(0x3f7) == 0 {
            if matches!(
                self.ram.player().action_state(),
                x if x == PlayerState::PermaBunny as u8 || x == PlayerState::TempBunny as u8
            ) {
                self.ram.write_u16(0x3f5, 0);
                return false;
            }
            if self.ram.read_u8(0x309) & 2 != 0 {
                self.ram.write_u8(0x308, 0)
            }
            let carrying = self.ram.read_u8(0x308) & 0x80;
            self.link_reset_properties_a();
            self.ram.write_u8(0x308, carrying);
            for i in (0..5).rev() {
                if matches!(self.ram.read_u8(0xc4a + i), 0x30 | 0x31) {
                    self.ram.write_u8(0xc4a + i, 0)
                }
            }
            self.link_cancel_dash();
            self.runtime.ancilla_add_cape_poof(self.ram, 0x23, 4);
            self.runtime.ancilla_sfx2_near(self.ram, 0x14);
            self.ram.write_u8(0x2e2, 20);
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x3f7, 1);
            self.ram.write_u8(0x4b, 12)
        }
        let timer = self.ram.read_u8(0x2e2).wrapping_sub(1);
        self.ram.write_u8(0x2e2, timer);
        if (timer as i8) < 0 {
            self.ram
                .set_player_action_state(PlayerState::TempBunny as u8);
            self.ram.write_u8(0x2e0, 1);
            self.ram.write_u8(0x56, 1);
            self.runtime.load_gear_palettes_bunny(self.ram);
            self.ram.write_u8(0x4b, 0);
            self.ram.write_u8(0x37b, 0);
            self.ram.write_u8(0x3f7, 0)
        }
        true
    }

    pub fn link_state_zapped(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.runtime.link_zap_handle_mosaic(self.ram);
        let delay = self.ram.read_u8(0x3d).wrapping_sub(1);
        self.ram.write_u8(0x3d, delay);
        if (delay as i8) >= 0 {
            return;
        }
        self.ram.write_u8(0x3d, 2);
        let phase = self.inc8(0x300);
        if phase & 1 != 0 {
            self.runtime.palette_electro_themed_gear(self.ram)
        } else {
            self.runtime.load_actual_gear_palettes(self.ram)
        }
        if phase == 8 {
            self.ram.write_u8(0x300, 0);
            self.ram.set_player_action_state(PlayerState::Ground as u8);
            self.ram.write_u8(0x37b, 0);
            self.ram.write_u8(0x360, 0);
            self.ram.write_u8(0x4d, 0);
            self.runtime.player_set_custom_mosaic_level(self.ram, 0)
        }
    }

    pub fn player_handler_15_hold_item(&mut self) {}

    pub fn link_receive_item(&mut self, item: u8, chest_position: i32) {
        if self.ram.read_u8(0x4d) != 0 {
            self.ram.write_u8(0x4d, 0);
            self.ram.write_u8(0x46, 0);
            self.ram.write_u8(0x31f, 0);
            self.ram.write_u8(0x308, 0)
        }
        self.ram.write_u8(0x2d8, item);
        if item == 0x3e {
            self.runtime.ancilla_sfx3_near(self.ram, 0x2e)
        }
        self.ram.write_u8(0x2d9, 0x60);
        if matches!(self.ram.read_u8(0x2e9), 0 | 3) {
            for address in [0x308, 0x3a, 0x3b, 0x3c, 0x5e, 0x50, 0x301, 0x37a, 0x300] {
                self.ram.write_u8(address, 0)
            }
            self.ram
                .set_player_action_state(PlayerState::HoldUpItem as u8);
            self.ram.write_u8(0x2da, if item == 0x20 { 2 } else { 1 });
            self.ram.write_u8(0x37b, 1)
        }
        self.runtime
            .ancilla_add_item_receipt(self.ram, 0x22, 4, chest_position);
        if !matches!(item, 0x20 | 0x37 | 0x38 | 0x39) {
            self.runtime.hud_refresh_icon(self.ram)
        }
        self.link_cancel_dash()
    }

    pub fn link_tuck_into_bed(&mut self) {
        self.ram.set_player_y(0x215a);
        self.ram.set_player_x(0x940);
        self.ram
            .set_player_action_state(PlayerState::AsleepInBed as u8);
        self.ram.write_u8(0x37c, 0);
        self.ram.write_u8(0x37d, 0);
        self.ram.write_u8(0x374, 3);
        self.runtime.ancilla_add_blanket(self.ram, 0x20)
    }

    pub fn link_state_sleeping(&mut self) {
        match self.ram.read_u8(0x37c) {
            0 => {
                if self.ram.read_u8(0x1a) & 0x1f == 0 {
                    self.runtime.ancilla_add_snoring(self.ram, 0x21, 1)
                }
            }
            1 => {
                if self.ram.read_u8(0x11) != 0 {
                    return;
                }
                let countdown = self.ram.read_u8(0x374).wrapping_sub(1);
                self.ram.write_u8(0x374, countdown);
                if (countdown as i8) < 0 {
                    self.ram.write_u8(0x374, 0);
                    let high = self.ram.read_u8(0xf4);
                    if ((high & 0xe0) | high.wrapping_shl(4) | self.ram.read_u8(0xf6)) & 0xf0 != 0 {
                        self.inc8(0x37d);
                        self.ram.set_player_facing(6);
                        self.inc8(0x37c);
                        self.ram.write_u8(0x374, 4)
                    }
                }
            }
            2 => {
                let countdown = self.ram.read_u8(0x374).wrapping_sub(1);
                self.ram.write_u8(0x374, countdown);
                if (countdown as i8) < 0 {
                    self.ram.set_player_actual_velocity_y(4);
                    self.ram.set_player_actual_velocity_x(21);
                    self.ram.set_player_actual_velocity_z(24);
                    self.ram.write_u8(0x2c7, 24);
                    self.ram.write_u8(0x46, 16);
                    self.ram.write_u8(0x4d, 2);
                    self.ram
                        .set_player_action_state(PlayerState::RecoilOther as u8)
                }
            }
            _ => {}
        }
    }

    pub fn link_item_cape(&mut self) {
        if self.ram.read_u8(0x55) == 0 {
            let timer = self.ram.read_u8(0x2e2).wrapping_sub(1);
            self.ram.write_u8(0x2e2, timer);
            if (timer as i8) >= 0 {
                self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
                self.halt_link_when_using_items();
                return;
            }
            self.ram.write_u8(0x2e2, 0);
            if self.ram.read_u8(0x6c) != 0 || !self.check_y_button_press() {
                return;
            }
            self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x40);
            if self.ram.player().magic() == 0 {
                self.runtime.ancilla_sfx2_near(self.ram, 60);
                self.ram.write_u16(0x1cf0, 123);
                self.runtime.main_show_text_message(self.ram);
                return;
            }
            self.ram.write_u8(0x300, 0);
            self.ram.write_u8(0x55, 1);
            self.ram.write_u8(
                0x4c,
                CAPE_DEPLETION_TIMERS[usize::from(self.ram.read_u8(0xf37b))],
            );
            self.ram.write_u8(0x2e2, 20);
            self.runtime.ancilla_add_cape_poof(self.ram, 35, 4);
            self.runtime.ancilla_sfx2_near(self.ram, 20);
        } else {
            self.ram.write_u8(0x37b, 1);
            self.halt_link_when_using_items();
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & !0xf);
            let depletion = self.ram.read_u8(0x4c).wrapping_sub(1);
            self.ram.write_u8(0x4c, depletion);
            if depletion == 0 {
                self.ram.write_u8(
                    0x4c,
                    CAPE_DEPLETION_TIMERS[usize::from(self.ram.read_u8(0xf37b))],
                );
                let magic = self.ram.player().magic();
                if (magic == 0 && self.runtime.enhanced_features0() & 4096 != 0) || {
                    let decremented = magic.wrapping_sub(1);
                    self.ram.set_player_magic(decremented);
                    decremented == 0
                } {
                    self.link_force_unequip_cape();
                    return;
                }
            }
            let timer = self.ram.read_u8(0x2e2).wrapping_sub(1);
            self.ram.write_u8(0x2e2, timer);
            if (timer as i8) < 0 {
                self.ram.write_u8(0x2e2, 0);
                if self.ram.read_u8(0xf4) & 0x40 != 0 {
                    self.link_force_unequip_cape()
                }
            }
        }
    }

    pub fn link_force_unequip_cape(&mut self) {
        self.runtime.ancilla_add_cape_poof(self.ram, 35, 4);
        self.runtime.ancilla_sfx2_near(self.ram, 21);
        self.link_force_unequip_cape_quietly()
    }

    pub fn halt_link_when_using_items(&mut self) {
        if self.ram.read_u8(0xad) == 2 && self.ram.read_u8(0x322) & 3 == 3 {
            for address in [0x30, 0x31, 0x67, 0x2a, 0x2b, 0x6b] {
                self.ram.write_u8(address, 0)
            }
        }
        if self.ram.read_u8(0x2f5) != 0 {
            self.ram.write_u8(0x67, 0)
        }
    }

    pub fn link_handle_cape_passive_lift_check(&mut self) {
        if self.ram.read_u8(0x308) & 0x80 != 0
            || (self.runtime.enhanced_features0() & 4096 != 0 && self.ram.read_u8(0x376) != 0)
        {
            self.player_check_handle_cape_stuff()
        }
    }

    pub fn player_check_handle_cape_stuff(&mut self) {
        if self.ram.read_u8(0x55) == 0 || self.ram.read_u8(0x304) != 19 {
            return;
        }
        if self.ram.read_u8(0x304) == self.ram.read_u8(0x303) {
            let depletion = self.ram.read_u8(0x4c).wrapping_sub(1);
            self.ram.write_u8(0x4c, depletion);
            if depletion != 0 {
                return;
            }
            self.ram.write_u8(
                0x4c,
                CAPE_DEPLETION_TIMERS[usize::from(self.ram.read_u8(0xf37b))],
            );
            let magic = self.ram.player().magic();
            if magic == 0 {
                return;
            }
            let magic = magic.wrapping_sub(1);
            self.ram.set_player_magic(magic);
            if magic != 0 {
                return;
            }
        }
        self.link_force_unequip_cape()
    }

    pub fn link_check_for_edge_screen_transition(&mut self) -> bool {
        let state = self.ram.player().action_state();
        if matches!(state, 3 | 8 | 9 | 10) || self.ram.read_u8(0x46) == 0 {
            return false;
        }
        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_y(0);
        self.ram.write_u8(0x2c6, 3);
        self.ram.set_player_x(self.ram.read_u16(0xfc2));
        self.ram.set_player_y(self.ram.read_u16(0xfc4));
        true
    }

    pub fn cache_camera_properties_if_outdoors(&mut self) {
        if self.ram.read_u8(0x1b) == 0 {
            self.cache_camera_properties()
        }
    }
    pub fn link_handle_cardinal_collision(&mut self) {
        self.ram.write_u16(0x6e, 0);
        self.ram.write_u16(0x38, 0);

        let can_check_second_layer = if self.ram.read_u8(0x6b) & 0x30 != 0 {
            true
        } else {
            self.link_handle_diagonal_kickback();
            self.ram.read_u8(0x6d) == 0
        };
        if can_check_second_layer && self.check_if_room_needs_double_layer_check() {
            let collision = self.ram.read_u8(0x46c);
            let vertical_first = if collision < 2 || collision == 3 {
                true
            } else {
                self.ram.write_u8(0x315, 2);
                self.runtime.player_tile_detect_nearby(self.ram);
                let nearby = self.ram.read_u8(0x0e);
                self.ram.write_u8(0x316, nearby);
                if nearby == 0 {
                    true
                } else {
                    self.ram.set_player_velocity_y(
                        self.ram
                            .player()
                            .velocity_y()
                            .wrapping_add(self.ram.read_u8(0x310)),
                    );
                    self.ram.set_player_velocity_x(
                        self.ram
                            .player()
                            .velocity_x()
                            .wrapping_add(self.ram.read_u8(0x312)),
                    );
                    match self.ram.read_u8(0x0e) {
                        12 | 3 => true,
                        10 | 5 => false,
                        a if a & 0xc == 0 && a & 3 == 0 => true,
                        _ if self.ram.player().velocity_y() != 0 => false,
                        _ if self.ram.player().velocity_x() == 0 => true,
                        _ => (self.ram.read_u8(0x310) as i8) < 0,
                    }
                }
            };
            if vertical_first {
                self.run_slope_collision_checks_vertical_first()
            } else {
                self.run_slope_collision_checks_horizontal_first()
            }
            self.create_velocity_from_moving_background();
        }

        match self.ram.read_u8(0x46c) {
            2 => {
                self.runtime.player_tile_detect_nearby(self.ram);
                if self.ram.read_u8(0x0e) | self.ram.read_u8(0x316) == 0xf {
                    if self.ram.read_u8(0x31f) == 0 {
                        self.ram.write_u8(0x31f, 58)
                    }
                    if self.ram.read_u8(0x67) == 0 {
                        if self.ram.read_u8(0x310) != 0 {
                            self.ram.set_player_velocity_y(
                                0u8.wrapping_sub(self.ram.player().velocity_y()),
                            )
                        }
                        if self.ram.read_u8(0x312) != 0 {
                            self.ram.set_player_velocity_x(
                                0u8.wrapping_sub(self.ram.player().velocity_x()),
                            )
                        }
                    }
                }
                self.ram.write_u8(0x315, 1);
                self.run_slope_collision_checks_vertical_first()
            }
            3 => {
                self.ram.write_u8(0x315, 1);
                self.run_slope_collision_checks_horizontal_first()
            }
            4 => {
                self.ram.write_u8(0x315, 1);
                self.run_slope_collision_checks_vertical_first()
            }
            _ if self.ram.player().velocity_x() | self.ram.player().velocity_y() != 0 => {
                self.ram.write_u8(0x315, 1);
                self.run_slope_collision_checks_vertical_first()
            }
            _ => {
                let state = self.ram.player().action_state();
                if !matches!(state, 19 | 8 | 9 | 10 | 3) {
                    self.runtime.player_tile_detect_nearby(self.ram);
                    if self.ram.read_u8(0x59) & 0xf != 0 {
                        self.ram
                            .set_player_action_state(PlayerState::FallingIntoHole as u8);
                        if self.ram.read_u8(0x372) == 0 {
                            self.ram.write_u8(0x5e, 4)
                        }
                    }
                }
            }
        }

        self.tile_detect_main_handler(0);
        if self.ram.read_u8(0x6a) != 0 {
            self.ram.write_u8(0x6b, 0)
        }
        if self.ram.player().action_state() != 11 {
            let vel = self
                .ram
                .player()
                .y()
                .wrapping_sub(self.ram.read_u8(0x3e) as u16) as u8;
            self.ram.set_player_velocity_y(vel);
            if vel != 0 {
                self.ram.write_u8(
                    0x67,
                    (self.ram.read_u8(0x67) & 3) | if (vel as i8) < 0 { 8 } else { 4 },
                )
            }
        }
        let vel = self
            .ram
            .player()
            .x()
            .wrapping_sub(self.ram.read_u8(0x3f) as u16) as u8;
        self.ram.set_player_velocity_x(vel);
        if vel != 0 {
            self.ram.write_u8(
                0x67,
                (self.ram.read_u8(0x67) & 0xc) | if (vel as i8) < 0 { 2 } else { 1 },
            )
        }

        if self.ram.read_u8(0x1b) == 0
            || self.ram.read_u8(0x46c) != 4
            || self.ram.player().action_state() != PlayerState::Swimming as u8
        {
            return;
        }
        let floor_y = self.ram.read_u8(0x310);
        if floor_y != 0 && self.ram.player().velocity_y().wrapping_sub(floor_y) == 0 {
            let mask = if (floor_y as i8) < 0 { !8 } else { !4 };
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & mask)
        }
        let floor_x = self.ram.read_u8(0x312);
        if floor_x != 0 && self.ram.player().velocity_x().wrapping_sub(floor_x) == 0 {
            let mask = if (floor_x as i8) < 0 { !2 } else { !1 };
            self.ram.write_u8(0x67, self.ram.read_u8(0x67) & mask)
        }
    }

    pub fn run_slope_collision_checks_vertical_first(&mut self) {
        if self.ram.read_u8(0x6b) & 0x20 == 0 {
            self.start_movement_collision_checks_y()
        }
        if self.ram.read_u8(0x6b) & 0x10 == 0 {
            self.start_movement_collision_checks_x()
        }
    }

    pub fn run_slope_collision_checks_horizontal_first(&mut self) {
        if self.ram.read_u8(0x6b) & 0x10 == 0 {
            self.start_movement_collision_checks_x()
        }
        if self.ram.read_u8(0x6b) & 0x20 == 0 {
            self.start_movement_collision_checks_y()
        }
    }

    fn start_movement_collision_checks_y(&mut self) {
        let vel = self.ram.player().velocity_y();
        if vel == 0 {
            return;
        }
        let direction = if self.ram.read_u8(0x6c) == 1 {
            if self.ram.player().y_low() < 0x80 {
                0
            } else {
                1
            }
        } else if (vel as i8) < 0 {
            0
        } else {
            1
        };
        self.ram.write_u8(0x66, direction);
        self.runtime.tile_detect_movement_y(self.ram, direction);
        if self.ram.read_u8(0x1b) != 0 {
            self.start_movement_collision_checks_y_handle_indoors()
        } else {
            self.start_movement_collision_checks_y_handle_outdoors()
        }
    }

    fn start_movement_collision_checks_x(&mut self) {
        let vel = self.ram.player().velocity_x();
        if vel == 0 {
            return;
        }
        let direction = if self.ram.read_u8(0x6c) == 2 {
            if self.ram.player().x_low() < 0x80 {
                2
            } else {
                3
            }
        } else if (vel as i8) < 0 {
            2
        } else {
            3
        };
        self.ram.write_u8(0x66, direction);
        self.runtime.tile_detect_movement_x(self.ram, direction);
        if self.ram.read_u8(0x1b) != 0 {
            self.start_movement_collision_checks_x_handle_indoors()
        } else {
            self.start_movement_collision_checks_x_handle_outdoors()
        }
    }

    fn start_movement_collision_checks_y_handle_indoors(&mut self) {
        let mut r14 = self.ram.read_u16(0x0e);
        if (self.ram.read_u8(0x308) as i8) < 0 || self.ram.read_u8(0x46) != 0 {
            r14 |= r14 >> 4;
            self.ram.write_u16(0x0e, r14)
        } else {
            if self.ram.read_u8(0x6c) == 2 {
                if self.ram.read_u8(0x6a) == 0 {
                    if self.ram.read_u8(0x46c) != 3 || self.ram.read_u8(0xee) == 0 {
                        self.link_add_in_velocity_y();
                        self.change_axis_of_perpendicular_door_movement_y();
                        return;
                    }
                    self.finish_indoor_collision_y();
                    return;
                } else if self.ram.read_u16(0x62) != 0 {
                    self.link_add_in_velocity_y();
                    self.clear_door_axis_state();
                    self.finish_indoor_collision_y();
                    return;
                }
            }
            r14 = self.ram.read_u16(0x0e);
            if r14 & 0x70 != 0 {
                if r14 >> 8 & 7 != 0 {
                    self.ram.write_u16(
                        0x49,
                        if (self.ram.player().velocity_y() as i8) < 0 {
                            8
                        } else {
                            4
                        },
                    )
                }
                self.ram.write_u8(0x6c, 1);
                self.ram.write_u8(0x3f3, 0);
                if r14 & 0x70 != 0x70 {
                    if r14 & 5 != 0 {
                        self.ram.write_u8(0x6b, 0);
                        self.link_add_in_velocity_y_falling();
                        self.calculate_snap_scratch_y();
                        self.ram.write_u8(0x6c, 0);
                        if r14 & 0x20 != 0 && r14 & 1 == 0 && self.ram.player().x() & 7 == 1 {
                            self.ram.set_player_x(self.ram.player().x() & !7)
                        }
                        if self.ram.read_u8(0x315) & 2 == 0 {
                            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !2)
                        }
                        return;
                    }
                    if r14 & 0x20 == 0 {
                        self.clear_door_axis_state();
                    } else {
                        if self.ram.read_u8(0x315) & 2 == 0 {
                            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !2)
                        }
                        return;
                    }
                } else {
                    if self.ram.read_u8(0x315) & 2 == 0 {
                        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !2)
                    }
                    return;
                }
            }
        }
        if self.ram.read_u8(0x315) & 2 == 0 {
            self.ram.write_u8(0x6c, 0);
            self.clear_door_axis_state()
        }
        self.finish_indoor_collision_y()
    }

    fn clear_door_axis_state(&mut self) {
        if self.ram.read_u8(0x315) & 2 == 0 {
            self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !2);
            self.ram.write_u8(0xef, 0);
            self.ram.write_u16(0x49, 0)
        }
    }

    fn finish_indoor_collision_y(&mut self) {
        let r14 = self.ram.read_u16(0x0e);
        if r14 & 7 == 0 && self.ram.read_u16(0x0c) & 5 != 0 {
            self.ram.write_u8(0x3f3, 0);
            self.flag_moving_into_slopes_y();
            if self.ram.read_u8(0x6b) & 0xf != 0 {
                return;
            }
        }
        self.ram.write_u8(0x6b, 0);
        if self.ram.read_u8(0x2e7) & 0x20 != 0 {
            let bak = self.ram.read_u16(0x0e);
            let tile = self.ram.read_u16(0x2ea);
            self.runtime.open_chest_for_item(self.ram, tile);
            self.ram.write_u16(0x2ea, 0);
            self.ram.write_u16(0x0e, bak)
        }
        self.handle_indoor_floor_markers();
        let mut hopped = false;
        if self.ram.read_u8(0x36d) & 7 == 7 && self.run_ledge_hop_timer() {
            self.link_cancel_dash();
            self.inc8(0x47a);
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x4d, 2);
            self.runtime.ancilla_sfx2_near(self.ram, 0x20);
            hopped = true;
        } else if self.ram.read_u16(0x341) & 7 == 7 && self.ram.read_u8(0x345) == 0 {
            self.link_cancel_dash();
            if self.ram.read_u8(0x1d) == 0 {
                self.dungeon_handle_layer_change()
            } else {
                self.enter_deep_water();
                self.runtime.ancilla_sfx2_near(self.ram, 0x20);
            }
            hopped = true;
        } else if self.ram.read_u16(0x343) & 2 != 0 && self.ram.read_u8(0x345) != 0 {
            if self.ram.read_u8(0x4d) != 0 {
                self.ram.write_u16(0x0e, 7)
            } else {
                self.link_cancel_dash();
                self.ram.write_u8(0x26, self.ram.read_u8(0x340));
                self.ram.write_u8(0x345, 0);
                if self.runtime.ancilla_add_splash(self.ram, 0x15, 0) {
                    self.ram.write_u8(0x345, 1);
                    self.ram.write_u16(0x0e, 7)
                } else {
                    self.ram.write_u8(0x37b, 1);
                    self.link_hop_in_or_out_of_water_y()
                }
            }
        }
        if hopped {
            self.ram.write_u8(0x37b, 1);
            self.link_hop_in_or_out_of_water_y()
        }
        if self.handle_stairs_and_pits_y() {
            return;
        }
        self.handle_spikes_and_push_y()
    }

    fn start_movement_collision_checks_x_handle_indoors(&mut self) {
        let mut r14 = self.ram.read_u16(0x0e);
        if (self.ram.read_u8(0x308) as i8) < 0 || self.ram.read_u8(0x46) != 0 {
            r14 |= r14 >> 4;
            self.ram.write_u16(0x0e, r14)
        } else {
            if self.ram.read_u8(0x6a) == 0 {
                self.ram.write_u8(0x57, 0)
            }
            if self.ram.read_u8(0x6c) == 1 && self.ram.read_u8(0x6a) == 0 {
                if self.ram.read_u8(0x46c) != 3 || self.ram.read_u8(0xee) == 0 {
                    self.snap_on_x();
                    let speed = self.change_axis_of_perpendicular_door_movement_x();
                    self.runtime.handle_nudging_in_a_door(self.ram, speed);
                    return;
                }
                self.finish_indoor_collision_x();
                return;
            }
            r14 = self.ram.read_u16(0x0e);
            if r14 & 0x70 != 0 {
                if r14 >> 8 & 7 != 0 {
                    self.ram.write_u16(
                        0x49,
                        if (self.ram.player().velocity_x() as i8) < 0 {
                            2
                        } else {
                            1
                        },
                    )
                }
                self.ram.write_u8(0x6c, 2);
                self.ram.write_u8(0x3f3, 0);
                if r14 & 0x70 != 0x70 && r14 & 7 != 0 {
                    self.ram.write_u8(0x6b, 0);
                    self.ram.write_u8(0x6c, 0);
                    self.snap_on_x();
                    self.calculate_snap_scratch_x();
                    return;
                }
                if self.ram.read_u8(0x315) & 2 == 0 {
                    self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !2)
                }
                return;
            }
        }
        if self.ram.read_u8(0x315) & 2 == 0 {
            self.ram.write_u8(0x6c, 0);
            self.clear_door_axis_state()
        }
        self.finish_indoor_collision_x()
    }

    fn finish_indoor_collision_x(&mut self) {
        if self.ram.read_u16(0x0e) & 2 == 0 && self.ram.read_u16(0x0c) & 5 != 0 {
            self.ram.write_u8(0x3f3, 0);
            self.flag_moving_into_slopes_x();
            if self.ram.read_u8(0x6b) & 0xf != 0 {
                return;
            }
        }
        self.ram.write_u8(0x6b, 0);
        self.handle_indoor_floor_markers();
        let mut hopped = false;
        if self.ram.read_u8(0x36e) & 7 == 7 && self.run_ledge_hop_timer() {
            self.link_cancel_dash();
            self.inc8(0x47a);
            self.ram.write_u8(0x4d, 2);
            hopped = true;
        } else if self.ram.read_u16(0x341) & 7 == 7
            && self.ram.read_u8(0x345) == 0
            && self.ram.player().action_state() != PlayerState::RecoilOther as u8
        {
            self.ram.set_player_y(u16::from_le_bytes([
                self.ram.read_u8(0x3e),
                self.ram.read_u8(0x40),
            ]));
            self.ram.set_player_x(u16::from_le_bytes([
                self.ram.read_u8(0x3f),
                self.ram.read_u8(0x41),
            ]));
            self.link_cancel_dash();
            if self.ram.read_u8(0x1d) == 0 {
                self.dungeon_handle_layer_change()
            } else {
                self.enter_deep_water()
            }
            hopped = true;
        } else if self.ram.read_u16(0x343) & 7 == 7 && self.ram.read_u8(0x345) != 0 {
            if self.ram.read_u8(0x4d) != 0 {
                self.ram.write_u16(0x0e, 7)
            } else {
                self.link_cancel_dash();
                self.ram.write_u8(0x26, self.ram.read_u8(0x340));
                self.ram.write_u8(0x345, 0);
                self.runtime.ancilla_add_splash(self.ram, 0x15, 0);
                self.ram.write_u8(0x37b, 1);
                self.link_hop_in_or_out_of_water_x()
            }
        }
        if hopped {
            self.ram.write_u8(0x37b, 1);
            self.link_hop_in_or_out_of_water_x();
            self.runtime.ancilla_sfx2_near(self.ram, 0x20);
        }
        if self.handle_pit() {
            return;
        }
        self.ram.write_u8(0x5b, 0);
        self.handle_spikes_and_push_x()
    }

    fn handle_indoor_floor_markers(&mut self) {
        let r14 = self.ram.read_u16(0x0e);
        if self.ram.read_u8(0xee) == 0 {
            if self.ram.read_u16(0x34c) & 7 != 0 {
                self.ram.write_u8(0x322, self.ram.read_u8(0x322) | 1)
            } else if self.ram.read_u8(0x2e8) & 7 == 0 && r14 & 2 == 0 {
                self.ram.write_u8(0x322, self.ram.read_u8(0x322) & !1)
            }
        } else if self.ram.read_u16(0x320) & 7 != 0 {
            self.ram.write_u8(0x322, self.ram.read_u8(0x322) | 2)
        } else {
            self.ram.write_u8(0x322, self.ram.read_u8(0x322) & !2)
        }
        let misc = self.ram.read_u16(0x2f6);
        if misc & 0x2200 != 0 {
            const DX: [u16; 4] = [8, 8, 0, 15];
            const DY: [u16; 4] = [8, 24, 16, 16];
            let direction = self.ram.read_u8(0x66) as usize;
            self.ram
                .set_player_rupees_goal(self.ram.player().rupees_goal().wrapping_add(5));
            let y = self
                .ram
                .player()
                .y()
                .wrapping_add(DY[direction])
                .wrapping_sub(if misc & 0x2000 != 0 { 8 } else { 0 });
            let x = self.ram.player().x().wrapping_add(DX[direction]);
            self.runtime.dungeon_delete_rupee_tile(self.ram, x, y);
            self.runtime.ancilla_sfx3_near(self.ram, 10)
        }
        let var4 = self.ram.read_u16(0x3f1);
        if var4 & 0x22 != 0 {
            self.ram
                .write_u8(0x3f3, if var4 & 0x20 != 0 { 2 } else { 1 })
        } else if var4 & 0x2200 != 0 {
            self.ram
                .write_u8(0x3f3, if var4 & 0x2000 != 0 { 4 } else { 3 })
        } else if self.ram.read_u8(0x2e8) & 7 == 0 && r14 & 2 == 0 {
            self.ram.write_u8(0x3f3, 0)
        }
    }

    fn enter_deep_water(&mut self) {
        self.ram.write_u8(0x345, 1);
        self.ram.write_u8(0x340, self.ram.read_u8(0x26));
        self.ram.write_u8(0x308, 0);
        self.ram.write_u8(0x309, 0);
        self.ram.write_u8(0x376, 0);
        self.ram.write_u8(0x5e, 0);
        self.link_reset_swimming_state()
    }

    fn handle_stairs_and_pits_y(&mut self) -> bool {
        if self.ram.read_u8(0x58) & 7 == 7 {
            if self.ram.read_u8(0x46) != 0 {
                self.ram.write_u16(
                    0x0e,
                    (self.ram.read_u16(0x0e) & !0xff) | u16::from(self.ram.read_u8(0x58) & 7),
                );
                self.handle_pushing_bonking_snaps_y();
                return true;
            }
            let stairs = self.ram.read_u16(0x2c0);
            if stairs & 0x77 != 0 {
                self.ram
                    .write_u8(0x11, if stairs & 0x70 != 0 { 16 } else { 8 });
                self.ram.write_u8(0x10, 7);
                self.link_cancel_dash()
            } else if self.runtime.enhanced_features0() & 4 != 0 {
                self.link_cancel_dash()
            }
            if self.ram.read_u8(0x66) & 2 == 0 {
                self.ram.write_u8(0x5e, 2);
                self.ram.write_u8(0x57, 1);
                return true;
            }
        }
        if self.ram.read_u8(0x5e) == 2 {
            self.ram
                .write_u8(0x5e, if self.ram.read_u8(0x372) != 0 { 16 } else { 0 })
        }
        if self.ram.read_u8(0x57) == 1 {
            self.ram.write_u8(0x57, 2)
        }
        self.handle_pit()
    }

    fn handle_pit(&mut self) -> bool {
        if self.ram.read_u8(0x59) & 5 != 0 && self.ram.read_u16(0x0e) & 2 == 0 {
            if !matches!(self.ram.player().action_state(), 5 | 2) {
                self.ram.write_u8(0x5c, 9);
                self.ram.write_u8(0x5a, 0);
                self.ram.write_u8(0x5b, 1);
                self.ram
                    .set_player_action_state(PlayerState::FallingIntoHole as u8)
            }
            true
        } else {
            false
        }
    }

    fn start_movement_collision_checks_y_handle_outdoors(&mut self) {
        if self.ram.read_u8(0x5e) == 2 {
            self.ram
                .write_u8(0x5e, if self.ram.read_u8(0x372) != 0 { 16 } else { 0 })
        }
        if self.handle_pit() {
            return;
        }
        self.ram.write_u8(
            0x368,
            if self.ram.read_u16(0x366) & 2 != 0 {
                self.ram.read_u8(0x36a) >> 1
            } else {
                0
            },
        );
        if self.ram.read_u16(0x341) & 2 != 0
            && self.ram.read_u8(0x345) == 0
            && self.ram.read_u8(0x4d) == 0
        {
            self.link_reset_sword_and_item_usage();
            self.link_cancel_dash();
            self.enter_deep_water();
            if self.ram.read_u8(0x351) == 1 {
                self.link_force_unequip_cape_quietly();
                if self.ram.player().flippers() != 0 {
                    if self.ram.read_u8(0x2e0) == 0 {
                        self.ram
                            .set_player_action_state(PlayerState::Swimming as u8)
                    }
                } else {
                    self.outdoor_water_recoil_y()
                }
            } else {
                self.outdoor_water_recoil_y()
            }
        }
        if self.ram.read_u8(0x345) != 0 {
            if self.ram.read_u8(0x36d) & 7 != 0 {
                self.ram
                    .write_u16(0x0e, u16::from(self.ram.read_u8(0x36d) & 7));
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            if self.ram.read_u8(0x58) & 7 == 7 || self.ram.read_u16(0x343) & 7 == 7 {
                self.link_cancel_dash();
                self.ram.write_u8(0x345, 0);
                if self.ram.read_u8(0x4d) == 0 {
                    self.ram.write_u8(0x26, self.ram.read_u8(0x340));
                    self.ram.write_u8(0x37b, 1);
                    self.runtime.ancilla_add_splash(self.ram, 0x15, 0);
                    self.link_hop_in_or_out_of_water_y();
                    return;
                }
            }
        }
        if self.ram.read_u8(0x36e) & 2 != 0 || self.ram.read_u8(0x370) & 0x22 != 0 {
            self.ram.write_u16(0x0e, 7);
            self.handle_pushing_bonking_snaps_y();
            return;
        }
        if self.ram.read_u8(0x36d) & 0x70 != 0 && self.run_ledge_hop_timer() {
            self.link_cancel_dash();
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x78, 1);
            self.ram.set_player_action_state(11);
            self.ram.write_u8(0x46, 0);
            self.ram.write_u16(0x364, 0xffff);
            self.ram.write_u8(0x48, 0);
            self.ram.write_u8(0x5e, 0);
            let z = if self.ram.read_u8(0x345) != 0 { 14 } else { 20 };
            self.ram.write_u8(0x362, z);
            self.ram.write_u8(0x363, z);
            self.ram
                .write_u8(0x4d, if self.ram.read_u8(0x345) != 0 { 4 } else { 2 });
            return;
        }
        if self.ram.read_u8(0x36d) & 7 != 0 && self.run_ledge_hop_timer() {
            self.runtime.ancilla_sfx2_near(self.ram, 0x20);
            self.ram.write_u8(0x37b, 1);
            self.link_cancel_dash();
            self.ram.write_u8(0x48, 0);
            self.ram.write_u8(0x5e, 0);
            self.link_find_valid_landing_tile_north();
            return;
        }
        if self.ram.read_u8(0x345) == 0 {
            let down_ledge = self.ram.read_u8(0x36f);
            if down_ledge & 7 != 0 && self.ram.read_u8(0x36d) & 0x77 == 0 {
                let mask = if self.ram.read_u16(0x76) == 0x2f {
                    4
                } else {
                    1
                };
                if down_ledge & mask != 0 && self.run_ledge_hop_timer() {
                    self.link_cancel_dash();
                    self.ram
                        .set_player_actual_velocity_x(if down_ledge & 4 != 0 { 16 } else { 0xf0 });
                    self.ram.write_u8(0x37b, 1);
                    self.ram.write_u8(0x48, 0);
                    self.ram.write_u8(0x5e, 0);
                    self.ram.write_u8(0x78, 1);
                    self.ram.write_u8(0x4d, 2);
                    self.ram.write_u8(0x362, 20);
                    self.ram.write_u8(0x363, 20);
                    self.ram.write_u16(0x364, self.ram.read_u16(0x364) | 0xff);
                    self.ram.write_u8(0x46, 0);
                    self.ram
                        .set_player_action_state(PlayerState::JumpOffLedgeDiag as u8);
                    return;
                }
            }
            if self.ram.read_u8(0x36e) & 0x70 != 0
                && self.ram.read_u8(0x36d) & 0x77 == 0
                && self.run_ledge_hop_timer()
            {
                self.link_cancel_dash();
                self.runtime.ancilla_sfx2_near(self.ram, 0x20);
                self.ram.write_u8(
                    0x66,
                    if self.ram.read_u8(0x36e) & 0x40 != 0 {
                        3
                    } else {
                        2
                    },
                );
                self.ram.write_u8(0x37b, 1);
                self.ram.write_u8(0x48, 0);
                self.ram.write_u8(0x5e, 0);
                self.link_find_valid_landing_tile_diagonal_north();
                return;
            }
        }
        if self.ram.read_u8(0x58) & 7 == 7 {
            if self.ram.read_u8(0x46) != 0 {
                self.ram
                    .write_u16(0x0e, u16::from(self.ram.read_u8(0x58) & 7));
                self.handle_pushing_bonking_snaps_y();
                return;
            } else if self.ram.read_u8(0x66) & 2 == 0 {
                self.ram.write_u8(0x5e, 2);
                self.ram.write_u8(0x57, 1);
                return;
            }
        }
        if self.ram.read_u8(0x5e) == 2 {
            self.ram
                .write_u8(0x5e, if self.ram.read_u8(0x372) != 0 { 16 } else { 0 })
        }
        if self.ram.read_u8(0x57) == 1 {
            self.ram.write_u8(0x57, 2)
        }
        if self.ram.read_u16(0x0e) & 7 == 0 && self.ram.read_u16(0x0c) & 5 != 0 {
            self.flag_moving_into_slopes_y();
            if self.ram.read_u8(0x6b) & 0xf != 0 {
                return;
            }
        }
        self.ram.write_u8(0x6b, 0);
        if self.ram.read_u8(0x2e7) & 2 != 0 && self.ram.read_u8(0x66) == 0 {
            let trigger = if self.ram.read_u8(0x372) != 0 {
                true
            } else {
                let timeout = self.ram.read_u8(0x61).wrapping_sub(1);
                self.ram.write_u8(0x61, timeout);
                (timeout as i8) < 0
            };
            if trigger {
                let bak = self.ram.read_u16(0x0e);
                self.runtime.ancilla_add_gravestone(self.ram);
                self.ram.write_u16(0x0e, bak);
                self.ram.write_u8(0x61, 52)
            }
        } else {
            self.ram.write_u8(0x61, 52)
        }
        self.handle_spikes_and_push_y()
    }

    fn start_movement_collision_checks_x_handle_outdoors(&mut self) {
        if self.ram.read_u8(0x6a) == 0 {
            self.ram.write_u8(0x57, 0);
            if self.ram.read_u8(0x5e) == 2 {
                self.ram.write_u8(0x5e, 0)
            }
        }
        if self.handle_pit() {
            return;
        }
        self.ram.write_u8(
            0x369,
            if self.ram.read_u16(0x366) & 2 != 0 {
                self.ram.read_u8(0x36a) >> 1
            } else {
                0
            },
        );
        if self.ram.read_u16(0x341) & 4 != 0
            && self.ram.read_u8(0x345) == 0
            && self.ram.read_u8(0x4d) == 0
        {
            self.link_cancel_dash();
            self.link_reset_sword_and_item_usage();
            self.enter_deep_water();
            if self.ram.read_u8(0x351) == 1 {
                self.link_force_unequip_cape_quietly();
                if self.ram.player().flippers() != 0 {
                    if self.ram.read_u8(0x2e0) == 0 {
                        self.ram
                            .set_player_action_state(PlayerState::Swimming as u8)
                    }
                } else {
                    self.outdoor_water_recoil_x()
                }
            } else {
                self.outdoor_water_recoil_x()
            }
        }
        let blocks = if self.ram.read_u8(0x345) != 0 {
            self.ram.read_u8(0x36e) & 7 == 7
        } else {
            self.ram.read_u8(0x36d) & 0x42 != 0
        };
        if blocks {
            self.ram.write_u16(0x0e, 7);
            self.handle_pushing_bonking_snaps_x();
            return;
        }
        if self.ram.read_u16(0x343) & 7 == 7 && self.ram.read_u8(0x345) != 0 {
            self.link_cancel_dash();
            if self.ram.read_u8(0x4d) == 0 {
                self.ram.write_u8(0x26, self.ram.read_u8(0x340));
                self.ram.write_u8(0x345, 0);
                self.runtime.ancilla_add_splash(self.ram, 0x15, 0);
                self.ram.write_u8(0x37b, 1);
                self.link_hop_in_or_out_of_water_x();
                return;
            }
        }
        if self.ram.read_u8(0x36e) & 7 != 0 && self.run_ledge_hop_timer() {
            self.runtime.ancilla_sfx2_near(self.ram, 0x20);
            self.ram
                .set_player_actual_velocity_x(if self.ram.read_u8(0x66) & 1 != 0 {
                    16
                } else {
                    0xf0
                });
            self.link_cancel_dash();
            self.ram.write_u8(0x4d, 2);
            self.ram.write_u8(0x362, 20);
            self.ram.write_u8(0x363, 20);
            self.ram.write_u16(0x364, self.ram.read_u16(0x364) | 0xff);
            self.ram
                .set_player_action_state(PlayerState::FallOfLeftRightLedge as u8);
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x78, 1);
            self.ram.write_u8(0x48, 0);
            self.ram.write_u8(0x5e, 0);
            if self.ram.read_u8(0x1b) == 0 {
                self.ram.write_u8(0xee, 2)
            }
            let saved_x = self.ram.player().x();
            let offset = (self.ram.read_u8(0x66) & !2).wrapping_mul(2);
            let found = self.link_hopping_horizontally_find_tile_x(offset);
            self.ram.write_u8(0x66, 1);
            if found != 0xff {
                self.link_hopping_horizontally_find_tile_y()
            } else {
                self.link_hop_find_tile_to_land_on_south()
            }
            self.ram.set_player_x(saved_x);
            return;
        }
        if self.ram.read_u8(0x370) & 0x77 != 0 && self.run_ledge_hop_timer() {
            let sound = self.runtime.ancilla_sfx2_near(self.ram, 0x20);
            self.ram
                .set_player_action_state(if sound & 7 == 0 { 16 } else { 15 });
            self.ram
                .set_player_actual_velocity_x(if self.ram.read_u8(0x66) & 1 != 0 {
                    16
                } else {
                    0xf0
                });
            self.link_cancel_dash();
            self.ram.write_u8(0x4d, 2);
            self.ram.write_u8(0x362, 20);
            self.ram.write_u8(0x363, 20);
            self.ram.write_u16(0x364, self.ram.read_u16(0x364) | 0xff);
            self.ram.write_u8(0x46, 0);
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x78, 1);
            self.ram.write_u8(0x48, 0);
            self.ram.write_u8(0x5e, 0);
            return;
        }
        if self.ram.read_u8(0x36e) & 0x70 != 0
            && self.ram.read_u8(0x36e) & 7 == 0
            && self.ram.read_u8(0x370) & 0x77 == 0
            && self.ram.player().action_state() != 13
            && self.run_ledge_hop_timer()
        {
            self.runtime.ancilla_sfx2_near(self.ram, 0x20);
            self.link_cancel_dash();
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x48, 0);
            self.ram.write_u8(0x5e, 0);
            self.link_find_valid_landing_tile_diagonal_north();
            return;
        }
        if self.ram.read_u8(0x36f) & 7 != 0
            && self.ram.read_u8(0x36e) & 7 == 0
            && self.ram.read_u8(0x370) & 0x77 == 0
            && self.run_ledge_hop_timer()
        {
            self.ram
                .set_player_actual_velocity_x(if self.ram.read_u8(0x66) & 1 != 0 {
                    16
                } else {
                    0xf0
                });
            self.link_cancel_dash();
            self.ram.write_u8(0x4d, 2);
            self.ram.write_u8(0x362, 20);
            self.ram.write_u8(0x363, 20);
            self.ram.write_u16(0x364, self.ram.read_u16(0x364) | 0xff);
            self.ram
                .set_player_action_state(PlayerState::JumpOffLedgeDiag as u8);
            self.ram.write_u8(0x46, 0);
            self.ram.write_u8(0x37b, 1);
            self.ram.write_u8(0x78, 1);
            self.ram.write_u8(0x48, 0);
            self.ram.write_u8(0x5e, 0);
            return;
        }
        if self.ram.read_u16(0x0e) & 7 == 0 && self.ram.read_u16(0x0c) & 5 != 0 {
            let skip = self.ram.read_u8(0x372) != 0 && self.ram.player().facing() & 4 == 0;
            if !skip || self.runtime.enhanced_features0() & 4096 != 0 {
                self.flag_moving_into_slopes_x();
                if self.ram.read_u8(0x6b) & 0xf != 0 {
                    return;
                }
            }
        }
        self.ram.write_u8(0x6b, 0);
        self.handle_spikes_and_push_x()
    }

    fn handle_spikes_and_push_y(&mut self) {
        self.ram.write_u8(0x5a, 0);
        if self.handle_spike_rebound(false) {
            return;
        }
        self.handle_moving_push_blocks(false);
        self.handle_pushing_bonking_snaps_y()
    }

    fn handle_spikes_and_push_x(&mut self) {
        if self.handle_spike_rebound(true) {
            return;
        }
        self.handle_moving_push_blocks(true);
        if self.ram.read_u8(0x6a) == 0 {
            self.ram.write_u8(0x57, 0);
            if self.ram.read_u8(0x5e) == 2 {
                self.ram.write_u8(0x5e, 0)
            }
        }
        self.handle_pushing_bonking_snaps_x()
    }

    fn handle_moving_push_blocks(&mut self, horizontal: bool) {
        if self.ram.read_u8(0x1b) == 0
            || !matches!(self.ram.read_u8(0x46c), 0 | 4) && self.ram.read_u8(0xee) != 0
        {
            return;
        }
        let bits = self.ram.read_u16(0x5f);
        if bits != 0 && self.ram.read_u8(0x6a) == 0 {
            self.ram.write_u8(0x2c2, bits as u8);
            let timer = self.ram.read_u8(0x61).wrapping_sub(1);
            self.ram.write_u8(0x61, timer);
            if (timer as i8) >= 0 {
                return;
            }
            {
                let mut scan = bits;
                for object in (0..16).rev() {
                    if scan & 0x8000 != 0 {
                        let slot = self.find_free_moving_block_slot(object);
                        if slot != 0xff {
                            self.ram.write_u16(0x0e, u16::from(slot));
                            if !self.initialize_push_block(slot, object * 2) {
                                self.sprite_dungeon_draw_single_push_block(slot * 2);
                                self.ram.write_u16(0x0e, 4);
                                let direction = u16::from(self.ram.read_u8(0x66)) * 2;
                                self.ram.write_u16(0x5f8 + usize::from(slot) * 2, direction);
                                self.ram.write_u8(0x474, direction as u8);
                                let coord_addr = if horizontal { 0x5e4 } else { 0x5f0 };
                                let coord = self.ram.read_u16(coord_addr + usize::from(slot) * 2);
                                let subtract = if horizontal {
                                    self.ram.read_u8(0x66) != 2
                                } else {
                                    self.ram.read_u8(0x66) == 1
                                };
                                self.ram.write_u16(
                                    0x5e8 + usize::from(slot) * 2,
                                    coord.wrapping_sub(subtract as u16) & 0xf,
                                )
                            }
                        }
                    }
                    scan <<= 1;
                }
            }
        }
        self.ram.write_u8(0x61, 21)
    }

    fn find_free_moving_block_slot(&mut self, object: u8) -> u8 {
        if self.ram.read_u8(0x5fd) == 0 {
            self.ram.write_u8(0x5fd, object.wrapping_add(1));
            1
        } else if self.ram.read_u8(0x5fc) == 0 {
            self.ram.write_u8(0x5fc, object.wrapping_add(1));
            0
        } else {
            0xff
        }
    }

    fn initialize_push_block(&mut self, slot: u8, object_index: u8) -> bool {
        let object = usize::from(object_index >> 1);
        let pos = self.ram.read_u16(0x540 + object * 2);
        let x = ((pos & 0x007e) << 2).wrapping_add(self.ram.read_u16(0x62c) & 0xff00);
        let y = ((pos & 0x1f80) >> 4).wrapping_add(self.ram.read_u16(0x62e) & 0xff00);
        let offset = usize::from(slot) * 2;
        self.ram.write_u16(0x5e4 + offset, u16::from(x as u8));
        self.ram
            .write_u16(0x5e0 + offset, u16::from((x >> 8) as u8));
        self.ram.write_u16(0x5f0 + offset, u16::from(y as u8));
        self.ram
            .write_u16(0x5ec + offset, u16::from((y >> 8) as u8));
        self.ram.write_u16(0x5e8 + offset, 0);
        self.ram.write_u16(0x5f4 + offset, 0);
        if self.ram.read_u8(0xae) != 38
            && self.ram.read_u16(0x500 + object * 2) == 0
            && !self.push_block_attempt_to_push_the_block(0, x, y)
        {
            self.runtime.ancilla_sfx2_near(self.ram, 0x22);
            self.ram.write_u16(0x500 + object * 2, 1);
            return false;
        }
        self.ram.write_u8(0x5fc + usize::from(slot), 0);
        true
    }

    pub fn push_block_attempt_to_push_the_block(&self, what: u8, x: u16, y: u16) -> bool {
        const Y0: [i8; 4] = [-4, 20, 4, 4];
        const Y1: [i8; 4] = [-4, 20, 12, 12];
        const X0: [i8; 4] = [4, 4, -4, 20];
        const X1: [i8; 4] = [12, 12, -4, 20];
        let index = usize::from(what.wrapping_mul(4).wrapping_add(self.ram.read_u8(0x66)));
        let mask = self.ram.read_u16(0xec);
        for (dx, dy) in [(X0[index], Y0[index]), (X1[index], Y1[index])] {
            let new_x = (add_i8(x, dx) & mask) >> 3;
            let new_y = add_i8(y, dy) & mask;
            let tile = self.push_block_get_target_tile_flag(new_x, new_y);
            if PUSH_BLOCK_TILE_SOLID[usize::from(tile)] != 0 && tile != 9 {
                return true;
            }
        }
        false
    }

    fn push_block_get_target_tile_flag(&self, x: u16, y: u16) -> u8 {
        let index = usize::from((y & !7).wrapping_mul(8).wrapping_add(x & 0x3f))
            + if self.ram.read_u8(0xee) != 0 {
                0x1000
            } else {
                0
            };
        self.ram.read_u8(0x12000 + index)
    }

    fn handle_spike_rebound(&mut self, horizontal: bool) -> bool {
        if self.ram.read_u8(0x2e8) & 7 == 0 {
            return false;
        }
        if self.ram.read_u8(0x46) | self.ram.read_u8(0x31f) | self.ram.read_u8(0x55) == 0 {
            let direction = self.ram.read_u8(0x66);
            let coord = self.ram.read_u16(if horizontal { 0x22 } else { 0x20 });
            let on_edge = if horizontal {
                if direction == 2 {
                    coord & 4 == 0
                } else {
                    coord & 4 != 0
                }
            } else if direction == 0 {
                coord & 4 == 0
            } else {
                coord & 4 != 0
            };
            if on_edge {
                self.ram.write_u8(0x373, 8);
                self.link_cancel_dash();
                if !horizontal || self.ram.read_u8(0x1b) != 0 {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_apply_tile_rebound();
                return true;
            }
        } else {
            self.ram.write_u16(
                0x0e,
                (self.ram.read_u16(0x0e) & !0xff) | u16::from(self.ram.read_u8(0x2e8) & 7),
            )
        }
        false
    }

    fn handle_pushing_bonking_snaps_y(&mut self) {
        let r14 = self.ram.read_u16(0x0e);
        if r14 & 7 != 0 {
            let mut reached_nudge = false;
            if self.ram.player().action_state() == PlayerState::Swimming as u8 {
                if self.ram.read_u8(0x310) == 0 {
                    self.reset_all_acceleration()
                }
                if self.ram.read_u8(0x6a) != 0 {
                    self.link_add_in_velocity_y_falling();
                    reached_nudge = true;
                }
            }
            if !reached_nudge {
                if r14 & 2 != 0 || r14 & 5 == 5 {
                    let bak = r14;
                    self.link_bonk_and_smash();
                    self.repel_dash();
                    self.ram.write_u16(0x0e, bak)
                }
                self.ram.write_u8(0x302, 1);
                if r14 & 2 == 2 {
                    self.link_add_in_velocity_y_falling()
                } else {
                    if self.ram.read_u8(0x6a) == 1 {
                        self.reset_push_timer_and_drag();
                        return;
                    }
                    self.link_add_in_velocity_y_falling();
                    if self.ram.read_u8(0x6a) == 2 {
                        self.reset_push_timer_and_drag();
                        return;
                    }
                }
            }
            {
                if r14 & 5 == 5 {
                    self.link_bonk_and_smash();
                    self.repel_dash()
                } else {
                    let vel = self.ram.player().velocity_y();
                    let tt = if r14 & 4 != 0 {
                        if (vel as i8) < 0 {
                            vel
                        } else {
                            0u8.wrapping_sub(vel)
                        }
                    } else if (vel as i8) < 0 {
                        0u8.wrapping_sub(vel)
                    } else {
                        vel
                    };
                    let nudge = if (tt as i8) < 0 { -1 } else { 1 };
                    if r14 & 2 == 0 {
                        if self.ram.player().x() & 7 != 0 {
                            self.ram.set_player_x(add_i8(self.ram.player().x(), nudge));
                            self.handle_nudging(nudge);
                            return;
                        }
                        self.link_bonk_and_smash();
                        self.repel_dash()
                    }
                }
                if self.update_drag_state() {
                    return;
                }
            }
        } else {
            if self.ram.read_u8(0xee) != 0 {
                return;
            }
            self.ram.write_u8(0x48, self.ram.read_u8(0x48) & !9)
        }
        self.reset_push_timer_and_drag()
    }

    fn handle_pushing_bonking_snaps_x(&mut self) {
        let r14 = self.ram.read_u16(0x0e);
        if r14 & 7 != 0 {
            if self.ram.player().action_state() == PlayerState::Swimming as u8
                && self.ram.read_u8(0x312) == 0
            {
                self.reset_all_acceleration()
            }
            if r14 & 2 != 0 {
                let bak = r14;
                self.link_bonk_and_smash();
                self.repel_dash();
                self.ram.write_u16(0x0e, bak)
            }
            self.ram.write_u8(0x302, 1);
            if r14 & 7 == 7 {
                self.snap_on_x()
            } else {
                if self.ram.read_u8(0x6a) == 2 {
                    self.reset_push_timer_and_drag();
                    return;
                }
                self.snap_on_x();
                if self.ram.read_u8(0x6a) == 1 {
                    self.reset_push_timer_and_drag();
                    return;
                }
            }
            {
                if r14 & 5 == 5 {
                    self.link_bonk_and_smash();
                    self.repel_dash()
                } else {
                    let vel = self.ram.player().velocity_x();
                    let tt = if r14 & 4 != 0 {
                        if (vel as i8) < 0 {
                            vel
                        } else {
                            0u8.wrapping_sub(vel)
                        }
                    } else if (vel as i8) < 0 {
                        0u8.wrapping_sub(vel)
                    } else {
                        vel
                    };
                    let nudge = if (tt as i8) < 0 { -1 } else { 1 };
                    if r14 & 2 == 0 {
                        if self.ram.player().y() & 7 != 0 {
                            self.ram.set_player_y(add_i8(self.ram.player().y(), nudge));
                            self.handle_nudging(nudge);
                            return;
                        }
                        self.link_bonk_and_smash();
                        self.repel_dash()
                    }
                }
                if self.update_drag_state() {
                    return;
                }
            }
        } else {
            if self.ram.read_u8(0xee) != 0 {
                return;
            }
            self.ram.write_u8(0x48, self.ram.read_u8(0x48) & !9)
        }
        self.reset_push_timer_and_drag()
    }

    fn reset_push_timer_and_drag(&mut self) {
        self.ram.write_u8(0x371, 32);
        self.ram.write_u8(0x48, self.ram.read_u8(0x48) & !2)
    }

    fn update_drag_state(&mut self) -> bool {
        if self.ram.read_u8(0x66).wrapping_mul(2) == self.ram.player().facing() {
            self.ram.write_u8(
                0x48,
                self.ram.read_u8(0x48) | ((self.ram.read_u8(0x315) & 1) << 1),
            );
            if self.ram.read_u8(0x3c) == 0 {
                let timer = self.ram.read_u8(0x371).wrapping_sub(1);
                self.ram.write_u8(0x371, timer);
                if (timer as i8) >= 0 {
                    return true;
                }
            }
            let bits = if self.ram.read_u16(0x2f6) & 0x20 != 0 {
                self.ram.read_u8(0x315) << 3
            } else {
                self.ram.read_u8(0x315)
            };
            self.ram.write_u8(0x48, self.ram.read_u8(0x48) | bits)
        }
        false
    }

    fn reset_all_acceleration(&mut self) {
        self.ram.reset_swim_acceleration();
    }

    fn link_bonk_and_smash(&mut self) {
        if self.ram.read_u8(0x372) != 0
            && self.ram.read_u8(0x2f1) != 64
            && self.ram.read_u8(0x2ef) & 0x70 != 0
        {
            const LIFT_TILE: [u8; 9] = [0x54, 0x52, 0x50, 0xff, 0x51, 0x53, 0x55, 0x56, 0x57];
            for second in [false, true] {
                if let Some((tile, x, y)) = self.runtime.overworld_smash_rock_pile(self.ram, second)
                    && let Some(kind) = LIFT_TILE.iter().position(|&value| value == tile)
                {
                    if matches!(kind, 2 | 4) {
                        self.runtime.ancilla_sfx3_near(self.ram, 0x32)
                    }
                    self.runtime
                        .sprite_spawn_immediately_smashed_terrain(self.ram, kind as u8, x, y)
                }
            }
        }
    }

    fn handle_nudging(&mut self, amount: i8) {
        const Y0: [u16; 8] = [8, 8, 23, 23, 8, 23, 8, 23];
        const X0: [u16; 8] = [0, 15, 0, 15, 0, 0, 15, 15];
        const Y1: [u16; 8] = [23, 23, 8, 8, 8, 23, 8, 23];
        const X1: [u16; 8] = [0, 15, 0, 15, 15, 15, 0, 0];
        let direction = self.ram.read_u8(0x66);
        let p = if direction & 2 == 0 {
            if direction & 1 != 0 { 4 } else { 0 }
        } else if direction & 1 != 0 {
            12
        } else {
            8
        };
        let offset = if self.ram.read_u16(0x0e) & 4 != 0 {
            0
        } else {
            2
        };
        let index = ((offset + p) >> 1) as usize;
        self.ram.write_u8(0x59, 0);
        self.runtime.tile_detect_reset_state(self.ram);
        let mask = self.ram.read_u16(0xec);
        let y0 = self.ram.player().y().wrapping_add(Y0[index]) & mask;
        let x0 = (self.ram.player().x().wrapping_add(X0[index]) & mask) >> 3;
        let y1 = self.ram.player().y().wrapping_add(Y1[index]) & mask;
        let x1 = (self.ram.player().x().wrapping_add(X1[index]) & mask) >> 3;
        self.runtime.tile_detection_execute(self.ram, x0, y0, 1);
        self.runtime.tile_detection_execute(self.ram, x1, y1, 2);
        if (self.ram.read_u16(0x0e) | u16::from(self.ram.read_u8(0x36e))) & 3 != 0
            || (self.ram.read_u8(0x36d) | self.ram.read_u8(0x370)) & 0x33 != 0
        {
            if direction & 2 != 0 {
                self.ram
                    .set_player_y(add_i8(self.ram.player().y(), -amount))
            } else {
                self.ram
                    .set_player_x(add_i8(self.ram.player().x(), -amount))
            }
        }
    }

    fn run_ledge_hop_timer(&mut self) -> bool {
        let mut result = false;
        if self.ram.read_u8(0x4d) != 1 {
            if self.ram.read_u8(0x372) == 0 {
                let timer = self.ram.read_u8(0x375).wrapping_sub(1);
                self.ram.write_u8(0x375, timer);
                if (timer as i8) < 0 {
                    self.ram.write_u8(0x375, 19);
                    return true;
                }
            } else {
                result = true
            }
        }
        self.ram.set_player_y(self.ram.read_u16(0xfc4));
        self.ram.set_player_x(self.ram.read_u16(0xfc2));
        self.ram.set_player_subpixel_y(0);
        self.ram.set_player_subpixel_x(0);
        result
    }

    fn link_add_in_velocity_y_falling(&mut self) {
        // Integer promotion in the C expression preserves a negative offset.
        // Treating it as `u8` turns -6 into 250 and crosses a coordinate page.
        let delta = i16::from(self.ram.read_u8(0x51) & 7)
            - if (self.ram.player().velocity_y() as i8) < 0 {
                8
            } else {
                0
            };
        self.ram
            .set_player_y(self.ram.player().y().wrapping_add_signed(-delta))
    }

    fn calculate_snap_scratch_y(&mut self) {
        let mut vel = self.ram.player().velocity_y();
        if self.ram.read_u16(0x0e) & 4 != 0 {
            if (vel as i8) >= 0 {
                vel = 0u8.wrapping_sub(vel)
            }
        } else if (vel as i8) < 0 {
            vel = 0u8.wrapping_sub(vel)
        }
        self.ram.set_player_x(add_i8(
            self.ram.player().x(),
            if (vel as i8) >= 0 { 1 } else { -1 },
        ))
    }

    fn link_add_in_velocity_y(&mut self) {
        self.ram.set_player_y(
            self.ram
                .player()
                .y()
                .wrapping_sub(self.ram.player().velocity_y() as i8 as i16 as u16),
        )
    }

    fn change_axis_of_perpendicular_door_movement_y(&mut self) {
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 2);
        let r14 = self.ram.read_u16(0x0e);
        if (r14 | r14 >> 4) & 7 == 0 {
            self.ram.write_u8(0x6c, 0);
            return;
        }
        let high_half = self.ram.player().x_low() >= 0x80;
        let delta = if high_half { -1 } else { 1 };
        if self.ram.read_u8(0x50) & 1 == 0 {
            self.ram.set_player_facing(if high_half { 4 } else { 6 })
        }
        self.ram.set_player_x(add_i8(self.ram.player().x(), delta))
    }

    fn snap_on_x(&mut self) {
        let x = self.ram.player().x();
        let delta = (x & 7).wrapping_sub(if (self.ram.player().velocity_x() as i8) < 0 {
            8
        } else {
            0
        });
        self.ram.set_player_x(x.wrapping_sub(delta))
    }

    fn calculate_snap_scratch_x(&mut self) {
        let mut vel = self.ram.player().velocity_x() as i8;
        if self.ram.read_u16(0x0e) & 4 != 0 {
            if vel >= 0 {
                vel = vel.wrapping_neg()
            }
        } else if vel < 0 {
            vel = vel.wrapping_neg()
        }
        self.ram
            .set_player_y(add_i8(self.ram.player().y(), if vel < 0 { -1 } else { 1 }))
    }

    fn change_axis_of_perpendicular_door_movement_x(&mut self) -> i8 {
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) | 2);
        let r14 = self.ram.read_u16(0x0e);
        let low = ((r14 | r14 >> 4) & 0xf) as u8;
        if low & 7 == 0 {
            self.ram.write_u8(0x6c, 0);
            return low as i8;
        }
        let high_half = self.ram.player().y_low() >= 0x80;
        let mut vel = self.ram.player().velocity_x() as i8;
        if high_half {
            if vel >= 0 {
                vel = vel.wrapping_neg()
            }
        } else if vel < 0 {
            vel = vel.wrapping_neg()
        }
        if self.ram.read_u8(0x50) & 1 == 0 {
            self.ram.set_player_facing(if high_half { 0 } else { 2 })
        }
        self.ram.set_player_y(add_i8(self.ram.player().y(), vel));
        vel
    }

    fn link_hop_in_or_out_of_water_y(&mut self) {
        const VELOCITY: [i8; 3] = [24, 16, 16];
        const VELOCITY_Z: [u8; 3] = [36, 24, 24];
        let index = if self.ram.read_u8(0x1b) == 0 {
            2
        } else if self.ram.read_u8(0x47a) != 0 {
            0
        } else {
            self.ram.read_u8(0x1d) as usize
        };
        let mut velocity = VELOCITY[index];
        if self.ram.read_u8(0x66) == 0 {
            velocity = -velocity
        }
        self.ram.set_player_actual_velocity_y(velocity as u8);
        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_z(VELOCITY_Z[index]);
        self.ram.write_u8(0x2c7, VELOCITY_Z[index]);
        self.ram.set_player_z(0);
        self.ram.write_u8(0x46, 16);
        if self.ram.read_u8(0x4d) != 2 {
            self.ram.write_u8(0x4d, 1);
            self.ram.write_u8(0x360, 0)
        }
        self.ram
            .set_player_action_state(PlayerState::RecoilOther as u8)
    }

    fn link_hop_in_or_out_of_water_x(&mut self) {
        const VELOCITY: [i8; 3] = [28, 24, 16];
        const VELOCITY_Z: [u8; 3] = [32, 24, 24];
        let index = if self.ram.read_u8(0x1b) == 0 {
            2
        } else if self.ram.read_u8(0x47a) != 0 {
            0
        } else {
            self.ram.read_u8(0x1d) as usize
        };
        let mut velocity = VELOCITY[index];
        if self.ram.read_u8(0x66) & 1 == 0 {
            velocity = -velocity
        }
        self.ram.set_player_actual_velocity_x(velocity as u8);
        self.ram.set_player_actual_velocity_y(0);
        self.ram.set_player_actual_velocity_z(VELOCITY_Z[index]);
        self.ram.write_u8(0x2c7, VELOCITY_Z[index]);
        self.ram.write_u8(0x46, 16);
        if self.ram.read_u8(0x4d) != 2 {
            self.ram.write_u8(0x4d, 1);
            self.ram.write_u8(0x360, 0)
        }
        self.ram
            .set_player_action_state(PlayerState::RecoilOther as u8)
    }

    fn repel_dash(&mut self) {
        if self.ram.read_u8(0x372) != 0 && self.ram.read_u8(0x2f1) != 64 {
            self.link_reset_swimming_state();
            self.runtime.ancilla_add_dash_tremor(self.ram);
            self.runtime.prepare_apply_rumble_to_sprites(self.ram);
            if !matches!(self.ram.read_u8(0x12f) & 0x3f, 27 | 50) {
                self.runtime.ancilla_sfx3_near(self.ram, 3)
            }
            self.link_apply_tile_rebound()
        }
    }

    fn link_apply_tile_rebound(&mut self) {
        const Y: [i8; 4] = [24, -24, 0, 0];
        const X: [i8; 4] = [0, 0, 24, -24];
        const SW11_Y: [u16; 4] = [1, 0, 0, 0];
        const SW11_X: [u16; 4] = [0, 0, 1, 0];
        const SW7_Y: [u16; 8] = [384, 384, 0, 0, 256, 256, 0, 0];
        const SW7_X: [u16; 8] = [0, 0, 384, 384, 0, 0, 256, 256];
        const DIRECTION: [u8; 4] = [8, 4, 2, 1];
        let direction = self.ram.read_u8(0x66) as usize;
        self.ram.set_player_actual_velocity_y(Y[direction] as u8);
        self.ram.set_player_actual_velocity_x(X[direction] as u8);
        self.ram.write_u8(0x46, 24);
        self.ram.set_player_actual_velocity_z(36);
        self.ram.write_u8(0x2c7, 36);
        let moving = self.ram.read_u8(0x34a);
        if moving != 0 {
            self.ram.write_u8(0x340, DIRECTION[direction]);
            self.ram.write_u8(0x67, DIRECTION[direction]);
            self.ram
                .set_swim_direction_selector(SwimAxis::Vertical, SW11_Y[direction]);
            self.ram
                .set_swim_direction_selector(SwimAxis::Horizontal, SW11_X[direction]);
            let index = (moving as usize - 1) * 4 + direction;
            self.ram.set_swim_speed(SwimAxis::Vertical, SW7_Y[index]);
            self.ram.set_swim_speed(SwimAxis::Horizontal, SW7_X[index])
        }
        self.ram.write_u8(0x4d, 1);
        self.ram.write_u8(0x2f8, 1);
        self.ram.write_u8(0x74, 0);
        self.ram.write_u8(0x360, 0);
        self.ram.write_u8(0x5e, 0);
        self.ram.write_u8(0x50, 0);
        self.ram.write_u8(0x6b, 0);
        if direction & 2 != 0 {
            self.ram.set_player_velocity_y(0)
        } else {
            self.ram.set_player_velocity_x(0)
        }
    }

    fn link_reset_sword_and_item_usage(&mut self) {
        self.ram.write_u8(0x5e, 0);
        self.ram.write_u8(0x48, self.ram.read_u8(0x48) & !9);
        self.ram.write_u8(0x3d, 0);
        self.ram.write_u8(0x3c, 0);
        self.ram.write_u8(0x3a, self.ram.read_u8(0x3a) & !0x81);
        self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1)
    }

    fn link_force_unequip_cape_quietly(&mut self) {
        self.ram.write_u8(0x2e2, 32);
        self.ram.write_u8(0x37b, 0);
        self.ram.write_u8(0x55, 0);
        self.ram.write_u8(0x360, 0)
    }

    fn outdoor_water_recoil_y(&mut self) {
        self.runtime.ancilla_sfx2_near(self.ram, 0x20);
        self.restore_safe_coordinates();
        self.ram.write_u8(0x37b, 1);
        self.link_hop_in_or_out_of_water_y()
    }

    fn link_hop_find_tile_to_land_on_south(&mut self) {
        const Y: [i8; 2] = [-8, 8];
        const Y2: [i8; 2] = [-16, 16];
        let original = self.ram.player().y();
        self.ram.write_u16(0x32, original);
        self.ram
            .set_player_velocity_y(original.wrapping_sub(u16::from(self.ram.read_u8(0x3e))) as u8);
        loop {
            let direction = self.ram.read_u8(0x66) as usize;
            self.ram
                .set_player_y(add_i8(self.ram.player().y(), Y[direction]));
            self.runtime
                .tile_detect_movement_y(self.ram, direction as u8);
            let tiles = self.ram.read_u16(0x343)
                | u16::from(self.ram.read_u8(0x59))
                | self.ram.read_u16(0x35b)
                | self.ram.read_u16(0x357)
                | self.ram.read_u16(0x341);
            if tiles & 7 == 7 {
                break;
            }
        }
        if self.ram.read_u16(0x341) & 7 != 0 {
            self.ram.write_u8(0x345, 1);
            if self.ram.read_u8(0x4d) != 4 {
                self.ram.write_u8(0x4d, 2)
            }
            self.ram.write_u8(0x340, self.ram.read_u8(0x26));
            self.link_reset_swimming_state();
            self.ram.write_u8(0x376, 0);
            self.ram.write_u8(0x5e, 0)
        }
        if self.ram.read_u8(0x59) & 7 != 0 {
            self.ram.write_u8(0x5c, 9);
            self.ram.write_u8(0x5a, 0);
            self.ram.write_u8(0x5b, 1)
        }
        let direction = self.ram.read_u8(0x66) as usize;
        self.ram
            .set_player_y(add_i8(self.ram.player().y(), Y2[direction]));
        let y = self.ram.player().y();
        self.ram.write_u8(0x3e, y as u8);
        self.ram.write_u8(0x40, (y >> 8) as u8);
        self.ram.write_u8(0x46, 1);
        let z = if self.ram.player().z_low() >= 0xf0 {
            0
        } else {
            self.ram.player().z_low()
        };
        let z = y.wrapping_sub(original).wrapping_add(u16::from(z));
        self.ram.set_player_z(z);
        self.ram.write_u16(0x364, z)
    }

    fn link_hopping_horizontally_find_tile_y(&mut self) {
        const Y: [i8; 2] = [-8, 8];
        const Y2: [i8; 2] = [-16, 16];
        const VEL_Z: [u8; 8] = [32, 32, 32, 40, 48, 56, 64, 72];
        const VEL_X: [u8; 8] = [16, 28, 28, 28, 28, 28, 28, 28];
        let original = self.ram.player().y();
        self.ram.write_u16(0x32, original);
        self.ram
            .set_player_velocity_y(original.wrapping_sub(u16::from(self.ram.read_u8(0x3e))) as u8);
        let direction = self.ram.read_u8(0x66) as usize;
        self.ram.set_player_y(add_i8(original, Y[direction]));
        self.runtime
            .tile_detect_movement_y(self.ram, direction as u8);
        let tiles = self.ram.read_u16(0x343)
            | self.ram.read_u16(0x35b)
            | self.ram.read_u16(0x357)
            | self.ram.read_u16(0x341);
        if tiles & 7 != 7 {
            self.ram.set_player_y(original);
            self.ram.write_u8(0x46, 1);
            let original_vel = self.ram.player().actual_velocity_x() as i8;
            let index = (original_vel.unsigned_abs() >> 4) as usize;
            self.ram.write_u8(0x362, VEL_Z[index]);
            self.ram.write_u8(0x363, VEL_Z[index]);
            let vel = VEL_X[index];
            self.ram.set_player_actual_velocity_x(if original_vel < 0 {
                0u8.wrapping_sub(vel)
            } else {
                vel
            })
        } else {
            self.ram
                .set_player_y(add_i8(self.ram.player().y(), Y2[direction]));
            let y = self.ram.player().y();
            self.ram.write_u8(0x3e, y as u8);
            self.ram.write_u8(0x40, (y >> 8) as u8);
            self.ram.write_u8(0x46, 1);
            let z = if self.ram.player().z_low() == 0xff {
                0
            } else {
                self.ram.player().z_low()
            };
            let z = y.wrapping_sub(original).wrapping_add(u16::from(z));
            self.ram.write_u16(0x364, z);
            self.ram.set_player_z(z)
        }
        if self.ram.read_u16(0x341) & 7 != 0 {
            self.ram.write_u8(0x4d, 2);
            self.enter_deep_water()
        }
    }

    fn link_hopping_horizontally_find_tile_x(&mut self, offset: u8) -> u8 {
        const STEP: [i8; 2] = [-8, 8];
        const FALLBACK: [i8; 2] = [-32, 32];
        const EXTRA: [i8; 2] = [-16, 16];
        const VEL_X: [u8; 24] = [
            20, 20, 20, 24, 24, 24, 24, 28, 28, 36, 36, 36, 36, 36, 36, 38, 38, 38, 38, 38, 38, 38,
            40, 40,
        ];
        const VEL_Z: [u8; 24] = [
            20, 20, 20, 20, 20, 20, 20, 24, 24, 32, 32, 32, 36, 36, 36, 38, 38, 38, 38, 38, 38, 38,
            40, 40,
        ];
        let side = usize::from(offset >> 1);
        let original = self.ram.player().x();
        self.ram.write_u16(0x32, original);
        let mut remaining = 7u8;
        loop {
            self.ram
                .set_player_x(add_i8(self.ram.player().x(), STEP[side]));
            let direction = self.ram.read_u8(0x66);
            self.runtime.tile_detect_movement_x(self.ram, direction);
            let tiles = self.ram.read_u16(0x343)
                | self.ram.read_u16(0x35b)
                | self.ram.read_u16(0x357)
                | self.ram.read_u16(0x341)
                | u16::from(self.ram.read_u8(0x59));
            if tiles & 7 == 7 {
                if self.ram.read_u16(0x341) & 7 == 7 {
                    self.ram.write_u8(0x345, 1);
                    self.ram.write_u8(0x4d, 2);
                    self.ram.write_u8(0x340, self.ram.read_u8(0x26));
                    self.ram.write_u8(0x2cb, 0);
                    self.ram.write_u8(0x5e, 0);
                    self.ram.write_u8(0x376, 0);
                    self.reset_all_acceleration()
                }
                break;
            }
            if remaining == 0 {
                self.ram.set_player_x(add_i8(original, FALLBACK[side]));
                remaining = 0xff;
                break;
            }
            remaining = remaining.wrapping_sub(1);
        }
        self.ram
            .set_player_x(add_i8(self.ram.player().x(), EXTRA[side]));
        let distance = original.abs_diff(self.ram.player().x()) >> 3;
        let mut velocity = VEL_X[distance as usize];
        if offset != 2 {
            velocity = 0u8.wrapping_sub(velocity)
        }
        self.ram.set_player_actual_velocity_x(velocity);
        self.ram.write_u8(0x362, VEL_Z[distance as usize]);
        self.ram.write_u8(0x363, VEL_Z[distance as usize]);
        remaining
    }

    fn outdoor_water_recoil_x(&mut self) {
        self.restore_safe_coordinates();
        self.ram.write_u8(0x37b, 1);
        self.link_hop_in_or_out_of_water_x();
        self.runtime.ancilla_sfx2_near(self.ram, 0x20);
    }

    fn restore_safe_coordinates(&mut self) {
        self.ram.set_player_y(u16::from_le_bytes([
            self.ram.read_u8(0x3e),
            self.ram.read_u8(0x40),
        ]));
        self.ram.set_player_x(u16::from_le_bytes([
            self.ram.read_u8(0x3f),
            self.ram.read_u8(0x41),
        ]))
    }

    fn link_find_valid_landing_tile_north(&mut self) {
        const DY: [u8; 32] = [
            16, 16, 20, 20, 24, 24, 28, 28, 32, 32, 36, 36, 40, 40, 44, 44, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        ];
        const DZ: [u8; 32] = [
            24, 24, 24, 24, 28, 28, 28, 28, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 44, 44,
            44, 44, 48, 48, 48, 48, 52, 52, 52, 52,
        ];
        const TIMER: [u8; 32] = [
            16, 16, 20, 20, 24, 24, 28, 28, 32, 32, 36, 36, 40, 40, 44, 44, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        ];
        let original = self.ram.player().y();
        self.ram.write_u16(0x32, original);
        loop {
            self.ram
                .set_player_y(self.ram.player().y().wrapping_sub(16));
            let direction = self.ram.read_u8(0x66);
            self.runtime.tile_detect_movement_y(self.ram, direction);
            let tiles = self.ram.read_u16(0x343)
                | self.ram.read_u16(0x35b)
                | self.ram.read_u16(0x357)
                | self.ram.read_u16(0x341);
            if tiles & 7 == 7 {
                break;
            }
        }
        if self.ram.read_u16(0x341) & 7 != 0 {
            self.ram.write_u8(0x4d, 1);
            self.ram.write_u8(0x360, 0);
            self.ram.write_u8(0x345, 1);
            self.ram.write_u8(0x340, self.ram.read_u8(0x26));
            self.link_reset_swimming_state();
            self.ram.write_u8(0x376, 0);
            self.ram.write_u8(0x5e, 0)
        }
        self.ram
            .set_player_y(self.ram.player().y().wrapping_sub(16));
        let distance = original.wrapping_sub(self.ram.player().y());
        self.ram.write_u16(0x32, distance);
        self.ram.set_player_y(original);
        let index = (distance as u8 >> 3) as usize;
        let mut dy = DY[index] as i8;
        if self.ram.read_u8(0x66) == 0 {
            dy = -dy
        }
        self.ram.set_player_actual_velocity_y(dy as u8);
        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_z(DZ[index]);
        self.ram.write_u8(0x2c7, DZ[index]);
        self.ram.set_player_z(0);
        self.ram.write_u8(0x46, TIMER[index]);
        self.ram.write_u8(0x4d, 2);
        self.ram.write_u8(0x360, 0);
        self.ram
            .set_player_action_state(PlayerState::RecoilOther as u8)
    }

    fn link_find_valid_landing_tile_diagonal_north(&mut self) {
        const DX: [u8; 32] = [
            8, 8, 8, 8, 16, 16, 16, 16, 24, 24, 24, 24, 16, 16, 16, 16, 8, 20, 20, 20, 24, 24, 24,
            24, 28, 28, 28, 28, 32, 32, 32, 32,
        ];
        const DY: [u8; 32] = [
            8, 8, 8, 8, 16, 16, 20, 20, 24, 24, 24, 24, 32, 32, 32, 32, 8, 20, 20, 20, 24, 24, 24,
            24, 28, 28, 28, 28, 32, 32, 32, 32,
        ];
        const DZ: [u8; 32] = [
            32, 32, 32, 32, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 32, 40, 40, 40, 44, 44,
            44, 44, 48, 48, 48, 48, 52, 52, 52, 52,
        ];
        let saved_y_safe = self.ram.read_u8(0x3e);
        let saved_x = self.ram.player().x();
        let direction = self.ram.read_u8(0x66);
        self.ram
            .set_player_actual_velocity_x(if direction != 2 { 1 } else { 0xff });
        self.ram.write_u8(0x66, 0);
        self.link_hop_find_landing_spot_diagonally_down();
        self.ram.set_player_x(saved_x);
        self.ram.write_u8(0x3e, saved_y_safe);
        let original_y = self.ram.read_u16(0x32);
        let distance = original_y.wrapping_sub(self.ram.player().y());
        let index = (distance >> 3) as usize;
        self.ram.set_player_y(original_y);
        self.ram
            .set_player_actual_velocity_y(0u8.wrapping_sub(DY[index]));
        self.ram.set_player_actual_velocity_x(if direction != 2 {
            DX[index]
        } else {
            0u8.wrapping_sub(DX[index])
        });
        self.ram.set_player_actual_velocity_z(DZ[index]);
        self.ram.write_u8(0x2c7, DZ[index]);
        self.ram.set_player_z(0);
        self.ram.write_u8(0x364, 0);
        self.ram.write_u8(0x4d, 2);
        self.ram.write_u8(0x360, 0);
        self.ram.set_player_action_state(13)
    }

    fn link_hop_find_landing_spot_diagonally_down(&mut self) {
        const DX: [i8; 2] = [-8, 8];
        const DY: [i8; 2] = [-9, 9];
        const BITS: [u8; 2] = [6, 3];
        const DY2: [i8; 2] = [-24, 24];
        let original = self.ram.player().y();
        self.ram.write_u16(0x32, original);
        self.ram
            .set_player_velocity_y(original.wrapping_sub(u16::from(self.ram.read_u8(0x3e))) as u8);
        let bits = loop {
            let side = usize::from((self.ram.player().actual_velocity_x() as i8) >= 0);
            self.ram
                .set_player_x(add_i8(self.ram.player().x(), DX[side]));
            let direction = self.ram.read_u8(0x66) as usize;
            self.ram
                .set_player_y(add_i8(self.ram.player().y(), DY[direction]));
            self.runtime
                .tile_detect_movement_y(self.ram, direction as u8);
            let bits = BITS[side];
            let tiles = self.ram.read_u16(0x343)
                | self.ram.read_u16(0x35b)
                | self.ram.read_u16(0x357)
                | self.ram.read_u16(0x341);
            if tiles as u8 & bits == bits {
                break bits;
            }
        };
        if self.ram.read_u16(0x341) as u8 & bits != 0 {
            self.ram.write_u8(0x345, 1);
            self.ram.write_u8(0x4d, 2);
            self.ram.write_u8(0x340, self.ram.read_u8(0x26));
            self.link_reset_swimming_state();
            self.ram.write_u8(0x5e, 0);
            self.ram.write_u8(0x376, 0)
        }
        let direction = self.ram.read_u8(0x66) as usize;
        self.ram
            .set_player_y(add_i8(self.ram.player().y(), DY2[direction]));
        let y = self.ram.player().y();
        self.ram.write_u8(0x3e, y as u8);
        self.ram.write_u8(0x40, (y >> 8) as u8);
        self.ram.write_u8(0x46, 1);
        let mirror = y
            .wrapping_sub(original)
            .wrapping_add(u16::from(self.ram.player().z_low()));
        self.ram.write_u16(0x364, mirror);
        self.ram.set_player_z(mirror)
    }

    pub fn link_handle_diagonal_kickback(&mut self) {
        let x_vel = self.ram.player().velocity_x();
        let y_vel = self.ram.player().velocity_y();
        let mut deadlocked = false;
        if x_vel != 0 && y_vel != 0 {
            let old_y = self.ram.player().y();
            let old_x = self.ram.player().x();
            self.ram.write_u16(0x2de, old_y);
            self.ram.write_u16(0x2dc, old_x);
            self.runtime
                .tile_detect_movement_x(self.ram, if (x_vel as i8) < 0 { 2 } else { 3 });
            if self.ram.read_u16(0x0c) & 5 != 0 {
                self.flag_moving_into_slopes_x();
                if self.ram.read_u8(0x6b) & 0xf != 0 {
                    let xd = self.ram.player().x().wrapping_sub(old_x) as u8;
                    self.ram.set_player_x(old_x);
                    self.ram.set_player_velocity_x(xd);
                    self.runtime
                        .tile_detect_movement_y(self.ram, if (y_vel as i8) < 0 { 0 } else { 1 });
                    if self.ram.read_u16(0x0c) & 5 != 0 {
                        self.flag_moving_into_slopes_y();
                        if self.ram.read_u8(0x6b) & 0xf != 0 {
                            let moving = self.ram.read_u8(0x6b);
                            self.ram.write_u8(0x6d, moving);
                            let yd = self.ram.player().y().wrapping_sub(old_y) as u8;
                            self.ram.set_player_velocity_y(yd);
                            const X0: [i8; 10] = [0, 1, 1, 1, 2, 2, 2, 3, 3, 3];
                            const X1: [i8; 10] = [0, -1, -1, -1, -2, -2, -2, -3, -3, -3];
                            const Y0: [i8; 10] = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3];
                            const Y1: [i8; 16] =
                                [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, -91, 48, -16, 4, -91, 49];
                            let dx = if (xd as i8) < 0 {
                                X1[(0u8.wrapping_sub(xd)) as usize]
                            } else {
                                X0[xd as usize]
                            };
                            let dy = if (yd as i8) < 0 {
                                Y1[(0u8.wrapping_sub(yd)) as usize]
                            } else {
                                Y0[yd as usize]
                            };
                            self.ram.set_player_x(add_i8(self.ram.player().x(), dx));
                            self.ram.set_player_y(add_i8(self.ram.player().y(), dy));
                            deadlocked = true;
                        }
                    }
                }
            }
        }
        if !deadlocked {
            self.ram.write_u8(0x6d, 0)
        }
        self.ram.write_u8(0x6b, 0)
    }

    pub fn flag_moving_into_slopes_y(&mut self) {
        let state = self.ram.read_u16(0x6e) as u8;
        let r12 = self.ram.read_u16(0x0c);
        let x = self.ram.player().x();
        let pos = self.ram.read_u16(0x51) as u8;
        let o = state
            .wrapping_mul(4)
            .wrapping_add(x.wrapping_sub(if r12 & 4 != 0 { 1 } else { 0 }) as u8 & 7)
            as usize;
        let mut y = (pos & 7) as i8;
        if self.ram.read_u16(0x38) & 5 != 0 {
            let mut ym = (pos & 7) as i8;
            if self.runtime.enhanced_features0() & 4096 != 0 {
                if state & 2 != 0 {
                    ym = -ym
                } else {
                    ym = AVOID_JUDDER[o] - (8 - ym)
                }
            } else {
                if state & 2 == 0 {
                    ym = 8 - ym
                } else {
                    ym += 8
                }
                ym = AVOID_JUDDER[o] - ym
            }
            let vel = self.ram.player().velocity_y();
            if vel == 0 {
                return;
            }
            if (vel as i8) < 0 {
                ym = -ym
            }
            y = ym
        } else {
            y = AVOID_JUDDER[o] - y
        }
        let vel = self.ram.player().velocity_y();
        if (vel as i8) < 0 {
            if y <= 0 {
                return;
            }
            self.ram.set_player_y(add_i8(self.ram.player().y(), y));
            self.ram.write_u8(0x6b, 8)
        } else {
            if y >= 0 {
                return;
            }
            self.ram.set_player_y(add_i8(self.ram.player().y(), y));
            self.ram.write_u8(0x6b, 4)
        }
        self.ram.write_u8(
            0x6b,
            self.ram.read_u8(0x6b) | if r12 & 4 != 0 { 0x12 } else { 0x11 },
        )
    }

    pub fn flag_moving_into_slopes_x(&mut self) {
        let state = self.ram.read_u16(0x6e) as u8;
        let r12 = self.ram.read_u16(0x0c);
        let coord = self.ram.player().x();
        let mut x = (coord.wrapping_sub((state == 6) as u16) & 7) as i8;
        let y_addr = if r12 & 4 != 0 { 0x53 } else { 0x51 };
        let o = state
            .wrapping_mul(4)
            .wrapping_add(self.ram.read_u16(y_addr) as u8 & 7) as usize;
        if self.ram.read_u16(0x38) & 5 != 0 {
            let mut xm = (coord & 7) as i8;
            if state != 4 && state != 6 {
                xm = -xm
            } else {
                xm = AVOID_JUDDER[o] - (8 - xm)
            }
            let vel = self.ram.player().velocity_x();
            if vel == 0 {
                return;
            }
            if (vel as i8) < 0 {
                xm = -xm
            }
            x = xm
        } else {
            x = AVOID_JUDDER[o] - x
        }
        let vel = self.ram.player().velocity_x();
        if (vel as i8) < 0 {
            if x <= 0 {
                return;
            }
            self.ram.set_player_x(add_i8(self.ram.player().x(), x));
            self.ram.write_u8(0x6b, 2)
        } else {
            if x >= 0 {
                return;
            }
            self.ram.set_player_x(add_i8(self.ram.player().x(), x));
            self.ram.write_u8(0x6b, 1)
        }
        self.ram.write_u8(
            0x6b,
            self.ram.read_u8(0x6b) | if state & 2 != 0 { 0x28 } else { 0x24 },
        )
    }

    pub fn tile_detect_main_handler(&mut self, item: u8) {
        const X: [i8; 40] = [
            8, 8, 8, 8, 6, 8, -1, 22, 19, 19, 0, 19, 6, 8, -1, 22, 8, 8, 8, 8, 8, 8, 0, 15, 6, 8,
            -10, 29, 6, 8, -6, 22, 6, 8, -4, 22, -4, 22, -4, 22,
        ];
        const Y: [i8; 40] = [
            20, 20, 20, 20, 4, 28, 16, 16, 22, 22, 22, 22, 4, 24, 16, 16, 16, 16, 16, 16, 20, 20,
            23, 23, -4, 36, 16, 16, 4, 28, 16, 16, 4, 28, 16, 16, 4, 4, 28, 28,
        ];
        self.ram.write_u8(0x59, 0);
        self.runtime.tile_detect_reset_state(self.ram);
        let mut offset = if item == 8 {
            let a = self.ram.read_u8(0x31c).wrapping_sub(2);
            if a >= 8 {
                return;
            }
            [10u8, 6, 14, 2, 12, 4, 8, 0][a as usize] + 0x40
        } else {
            item.wrapping_mul(8)
                .wrapping_add(self.ram.player().facing())
        };
        offset >>= 1;
        let mask = self.ram.read_u16(0xec);
        let x = add_i8(self.ram.player().x(), X[offset as usize]) & mask;
        let y = add_i8(self.ram.player().y(), Y[offset as usize]) & mask;
        if matches!(item, 1 | 2 | 3 | 6 | 7 | 8) {
            self.tile_behavior_handle_item_and_execute(x >> 3, y);
            return;
        }
        self.runtime.tile_detection_execute(self.ram, x >> 3, y, 1);
        if item == 5 {
            return;
        }
        self.tile_detect_main_handler_post_detection();
    }

    fn tile_detect_main_handler_post_detection(&mut self) {
        if self.ram.read_u16(0x357) & 0x10 != 0 {
            let tx = self.ram.player().x() as u8 & 0xf;
            let ty = self.ram.player().y().wrapping_add(8) as u8 & 0xf;
            if !(4..11).contains(&ty)
                && !(4..12).contains(&tx)
                && self.ram.read_u8(0x31f) == 0
                && self.ram.read_u8(0x4d) == 0
            {
                if self.ram.read_u8(0x1b) != 0 {
                    self.runtime.dungeon_flag_room_data_quadrants(self.ram);
                    self.runtime.ancilla_sfx2_near(self.ram, 0x33);
                    self.ram.write_u8(0x5e, 0);
                    self.ram.write_u8(0x11, 21);
                    self.ram.write_u8(0xa2, self.ram.read_u8(0xa0));
                    self.ram.write_u8(0xa0, self.ram.read_u8(0xc000));
                    self.handle_layer_of_destination();
                } else if self.ram.read_u8(0x2db) == 0 {
                    self.do_sword_interaction_with_tiles_mirror()
                }
            }
        } else {
            self.ram.write_u8(0x2db, 0);
            if self.ram.read_u16(0x357) & 1 != 0 {
                self.ram.write_u8(0x351, 2);
                if !self.link_permission_for_slosh_sounds() && self.ram.read_u8(0x4d) == 0 {
                    self.runtime.ancilla_sfx2_near(self.ram, 26);
                }
                return;
            }
            if self.ram.read_u16(0x359) & 1 != 0 {
                self.ram.write_u8(0x351, 1);
                if self.ram.read_u8(0x1b) == 0
                    && self.ram.read_u8(0x345) != 0
                    && self.ram.read_u8(0x2e0) == 0
                {
                    if self.ram.player().flippers() != 0 {
                        self.ram.write_u8(0x345, 0);
                        self.ram.write_u8(0x26, self.ram.read_u8(0x340));
                        self.ram.set_player_action_state(0)
                    }
                } else if !self.link_permission_for_slosh_sounds() {
                    if self.ram.read_u8(0x8a) == 0x70 {
                        self.runtime.ancilla_sfx2_near(self.ram, 27);
                    } else if self.ram.read_u8(0x4d) == 0 {
                        self.runtime.ancilla_sfx2_near(self.ram, 28);
                    }
                }
                return;
            }
            if self.ram.read_u8(0x1b) == 0
                && self.ram.read_u8(0x345) == 0
                && self.ram.read_u16(0x341) & 1 != 0
            {
                self.ram.write_u8(0x351, 1);
                if !self.link_permission_for_slosh_sounds() {
                    if self.ram.read_u8(0x8a) == 0x70 {
                        self.runtime.ancilla_sfx2_near(self.ram, 27);
                    } else if self.ram.read_u8(0x4d) == 0 {
                        self.runtime.ancilla_sfx2_near(self.ram, 28);
                    }
                }
                return;
            }
        }
        self.ram.write_u8(0x351, 0);
        if self.ram.read_u8(0x2ee) & 1 != 0 {
            self.ram.write_u8(0x2ed, 1);
            return;
        }
        self.ram.write_u8(0x2ed, 0);
        if self.ram.read_u8(0x2ee) & 0x10 != 0 {
            self.ram.write_u8(0x373, 0);
            if self.ram.read_u8(0x55) == 0
                && !self.search_for_byrna_spark()
                && self.ram.read_u8(0x31f) == 0
            {
                self.ram.write_u8(0x3f7, 0);
                self.ram.write_u16(0x3f5, 0);
                if self.ram.player().moon_pearl() != 0 {
                    self.ram.write_u8(0x56, 0);
                    self.ram.write_u8(0x2e0, 0)
                }
                self.ram.write_u8(0x373, 8);
                self.link_cancel_dash();
                return;
            }
        }
        if self.ram.read_u16(0x348) & 0x11 != 0 {
            if self.ram.read_u8(0x34a) != 0 {
                if self.ram.read_u8(0x6a) != 0 {
                    self.ram.write_u8(0x26, self.ram.read_u8(0x340))
                }
            } else {
                let direction = self.ram.read_u8(0x67);
                if direction & 0xc != 0 {
                    self.ram.set_swim_speed(SwimAxis::Vertical, 0x180)
                }
                if direction & 3 != 0 {
                    self.ram.set_swim_speed(SwimAxis::Vertical, 0x180)
                }
                self.ram.write_u8(
                    0x34a,
                    if self.ram.read_u16(0x348) & 1 != 0 {
                        1
                    } else {
                        2
                    },
                );
                self.ram.write_u8(0x340, self.ram.read_u8(0x26));
                self.link_reset_swimming_state()
            }
        } else {
            if self.ram.player().action_state() != PlayerState::Swimming as u8 {
                if self.ram.read_u8(0x34a) != 0 {
                    self.ram.write_u8(0x26, self.ram.read_u8(0x340))
                }
                self.link_reset_swimming_state()
            }
            self.ram.write_u8(0x34a, 0)
        }
        if self.ram.read_u8(0x2e8) & 0x10 != 0 && self.ram.read_u8(0x31f) == 0 {
            self.ram.write_u8(0x31f, 58)
        }
    }

    pub fn link_permission_for_slosh_sounds(&self) -> bool {
        if self.ram.read_u8(0x67) & 0xf == 0 {
            true
        } else if self.ram.player().action_state() != PlayerState::StartDash as u8 {
            self.ram.read_u8(0x1a) & 0xf != 0
        } else {
            self.ram.read_u8(0x1a) & 7 != 0
        }
    }

    fn tile_behavior_handle_item_and_execute(&mut self, x: u16, y: u16) {
        let tile = self
            .runtime
            .handle_item_tile_action_overworld(self.ram, x, y);
        self.runtime
            .tile_detect_execute_inner(self.ram, tile, 0, 1, false)
    }

    fn handle_layer_of_destination(&mut self) {
        let plane = self.ram.read_u8(0x63c);
        self.ram.write_u8(0x476, (plane >= 1) as u8);
        self.ram.write_u8(0xee, (plane >= 2) as u8)
    }

    fn do_sword_interaction_with_tiles_mirror(&mut self) {
        if self.ram.read_u8(0x1b) != 0 {
            if self.ram.read_u8(0xffc) != 0 {
                return;
            }
            self.runtime.mirror_save_room_data(self.ram);
            if self.ram.read_u8(0x12e) != 60 {
                self.ram.write_u8(0x5fc, 0);
                self.ram.write_u8(0x5fd, 0)
            }
        } else if self.ram.read_u8(0x10) != 11 {
            let world = self.ram.read_u8(0x8a) & 0x40;
            self.ram.write_u8(0x7b, world);
            if world != 0 {
                let y = self.ram.player().y();
                let x = self.ram.player().x();
                self.ram.write_u8(0x1adf, y as u8);
                self.ram.write_u8(0x1aef, (y >> 8) as u8);
                self.ram.write_u8(0x1abf, x as u8);
                self.ram.write_u8(0x1acf, (x >> 8) as u8)
            }
            self.ram.write_u8(0x11, 35);
            self.ram.write_u8(0x3f8, 0);
            self.ram.write_u8(0x2db, 1);
            self.ram.write_u8(0xb0, 0);
            self.ram.set_player_actual_velocity_x(0);
            self.ram.set_player_actual_velocity_y(0);
            self.ram.set_player_action_state(PlayerState::Mirror as u8)
        }
    }

    fn link_cancel_dash(&mut self) {
        if self.ram.read_u8(0x372) == 0 {
            return;
        }
        for i in 0..5 {
            if self.ram.read_u8(0xc4a + i) == 0x1e {
                self.ram.write_u8(0xc4a + i, 0)
            }
        }
        self.ram.write_u8(0x374, 0);
        self.ram.write_u8(0x5e, 0);
        self.ram.write_u8(0x372, 0);
        self.ram.write_u8(0x50, 0);
        self.ram.set_swim_acceleration_mode(SwimAxis::Vertical, 0)
    }

    pub fn link_reset_swimming_state(&mut self) {
        self.ram.write_u8(0x2cb, 0);
        self.ram.write_u8(0x34f, 0);
        self.ram.write_u8(0x32a, 0);
        self.ram.reset_swim_acceleration();
    }
    pub fn link_control_handler(&mut self) {
        let damage = self.ram.read_u8(0x373);
        if damage != 0 {
            if self.ram.read_u8(0x55) != 0 {
                self.ram.write_u8(0x373, 0);
                self.ram.write_u8(0x4d, 0);
                self.ram.write_u8(0x46, 0);
            } else if self.ram.read_u8(0x37b) == 0 {
                self.ram.write_u8(0x373, 0);
                if self.ram.read_u8(0xc4a) == 5
                    && self.ram.read_u8(0x300) == 0
                    && self.ram.read_u8(0x3d) != 0
                {
                    self.ram.write_u8(0xc4a, 0);
                    self.ram.write_u8(0x35f, 0);
                }
                if self.ram.read_u8(0x31f) == 0 {
                    self.ram.write_u8(0x31f, 58);
                }
                self.runtime.ancilla_sfx2_near(self.ram, 38);
                self.ram
                    .write_u8(0xcfc, self.ram.read_u8(0xcfc).wrapping_add(1));
                let mut health = self.ram.player().health().wrapping_sub(damage);
                if health == 0 || health >= 0xa8 {
                    self.ram.write_u8(0xc211, self.ram.read_u8(0x1c));
                    self.ram.write_u8(0xc212, self.ram.read_u8(0x1d));
                    self.ram.write_u8(0x10c, self.ram.read_u8(0x10));
                    self.ram.write_u8(0x10, 18);
                    self.ram.write_u8(0x11, 1);
                    self.ram.write_u8(0x31f, 0);
                    self.ram.write_u8(0xf372, 0);
                    health = 0;
                }
                self.ram.set_player_health(health);
            }
        }
        if self.ram.player().action_state() != 0 {
            self.player_check_handle_cape_stuff();
        }
        match self.ram.player().action_state() {
            0 => self.link_state_default(),
            1 => self.link_state_pits(),
            2 | 6 => self.link_state_recoil(),
            3 | 30 => self.link_state_spin_attack(),
            4 => self.player_handler_04_swimming(),
            5 => self.link_state_on_ice(),
            7 => self.link_state_zapped(),
            8 => self.link_state_using_ether(),
            9 => self.link_state_using_bombos(),
            10 => self.link_state_using_quake(),
            11 => self.link_hop_hopping_south_ow(),
            12 => self.link_state_hopping_horizontally_ow(),
            13 => self.link_state_hopping_diagonally_up_ow(),
            14 => self.link_state_hopping_diagonally_down_ow(),
            15 | 16 => self.link_state_0f(),
            17 => self.link_state_dashing(),
            18 => self.link_state_exiting_dash(),
            19 => self.link_state_hookshotting(),
            20 => self.link_state_crossing_worlds(),
            21 => self.player_handler_15_hold_item(),
            22 => self.link_state_sleeping(),
            23 => self.player_handler_17_bunny(),
            24 => self.link_state_holding_big_rock(),
            25 => self.link_state_receiving_ether(),
            26 => self.link_state_receiving_bombos(),
            27 => self.link_state_reading_desert_tablet(),
            28 => self.link_state_temporary_bunny(),
            29 => self.link_state_tree_pull(),
            state => panic!("invalid player handler state {state}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_sum4_matches_the_c_nibble_popcount() {
        for value in 0..=u8::MAX {
            assert_eq!(bit_sum4(value), (value & 0x0f).count_ones() as u8);
        }
    }
    use z3_core::WRAM_SIZE;
    struct Rt;
    impl PlayerRuntime for Rt {}
    #[derive(Default)]
    struct RecordingRt {
        calls: Vec<&'static str>,
    }
    struct LandingRt;
    impl PlayerRuntime for LandingRt {
        fn tile_detect_movement_y(&mut self, ram: &mut Ram<'_>, _direction: u8) {
            ram.write_u16(0x343, 7)
        }
    }
    #[derive(Default)]
    struct SpellRt {
        ether_position: Option<(u16, u16)>,
        bombos_position: Option<(u16, u16)>,
    }
    impl PlayerRuntime for SpellRt {
        fn ancilla_add_ether_spell(&mut self, ram: &mut Ram<'_>, _type_: u8, _arg: u8) {
            self.ether_position = Some((ram.player().x(), ram.player().y()))
        }
        fn ancilla_add_bombos_spell(&mut self, ram: &mut Ram<'_>, _type_: u8, _arg: u8) {
            self.bombos_position = Some((ram.player().x(), ram.player().y()))
        }
    }
    impl PlayerRuntime for RecordingRt {
        fn tile_detect_movement_x(&mut self, _ram: &mut Ram<'_>, _direction: u8) {
            self.calls.push("x")
        }
        fn tile_detect_movement_y(&mut self, _ram: &mut Ram<'_>, _direction: u8) {
            self.calls.push("y")
        }
        fn tile_detect_reset_state(&mut self, _ram: &mut Ram<'_>) {
            self.calls.push("reset")
        }
        fn tile_detection_execute(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16, _bits: u8) {
            self.calls.push("execute")
        }
    }
    #[test]
    fn tables_match_header_contract() {
        assert_eq!(SWIMMING_TAB1, [2, 0, 1, 0]);
        assert_eq!(SWIMMING_TAB2, [32, 8]);
        assert_eq!(LINK_SPIN_GRAPHICS_BY_DIR.len(), 48)
    }
    #[test]
    fn push_block_tile_table_matches_c_payload() {
        let hash = PUSH_BLOCK_TILE_SOLID
            .iter()
            .fold(0x811c_9dc5u32, |hash, &byte| {
                (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
            });
        assert_eq!(PUSH_BLOCK_TILE_SOLID.len(), 256);
        assert_eq!(hash, 0x7be4_20db)
    }
    #[test]
    fn swim_gate_selects_indoor_submodule() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x1b, 1);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.check_ability_to_swim();
        assert_eq!(p.ram.read_u8(0x11), 20);
        assert_eq!(p.ram.read_u8(0x4b), 12)
    }
    #[test]
    fn double_layer_translation_round_trips_and_adds_velocity() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x46c, 2);
        ram.set_player_y(0x100);
        ram.set_player_x(0x200);
        ram.write_u16(0xe6, 0x30);
        ram.write_u16(0xe8, 0x10);
        ram.write_u16(0xe0, 0x50);
        ram.write_u16(0xe2, 0x20);
        ram.write_u8(0x67, 1);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        assert!(p.check_if_room_needs_double_layer_check());
        assert_eq!(p.ram.player().y(), 0x120);
        assert_eq!(p.ram.player().x(), 0x230);
        p.ram.set_player_x(0x232);
        p.ram.set_player_y(0x123);
        p.create_velocity_from_moving_background();
        assert_eq!(p.ram.player().y(), 0x103);
        assert_eq!(p.ram.player().x(), 0x202);
        assert_eq!(p.ram.player().velocity_y(), 3);
        assert_eq!(p.ram.player().velocity_x(), 2);
        assert_eq!(p.ram.read_u8(0xee), 0)
    }
    #[test]
    fn direction_masks_preserve_facing_axis_priority() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x67, 5);
        ram.set_player_facing(4);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.player_limit_directions_inner();
        assert_eq!(p.ram.read_u8(0x67), 5);
        assert_eq!(p.ram.read_u8(0x6a), 2)
    }

    #[test]
    fn slope_runners_preserve_axis_order_and_skip_masked_axes() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = RecordingRt::default();
        ram.set_player_velocity_y(1);
        ram.set_player_velocity_x(1);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.run_slope_collision_checks_vertical_first();
        assert_eq!(p.runtime.calls, ["y", "x"]);
        p.runtime.calls.clear();
        p.ram.write_u8(0x6b, 0x20);
        p.run_slope_collision_checks_vertical_first();
        assert_eq!(p.runtime.calls, ["x"]);
        p.runtime.calls.clear();
        p.ram.write_u8(0x6b, 0);
        p.run_slope_collision_checks_horizontal_first();
        assert_eq!(p.runtime.calls, ["x", "y"])
    }

    #[test]
    fn tile_main_handler_uses_facing_probe_and_grass_early_return() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = RecordingRt::default();
        ram.write_u16(0xec, 0xffff);
        ram.set_player_x(0x100);
        ram.set_player_y(0x200);
        ram.write_u16(0x357, 1);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.tile_detect_main_handler(0);
        assert_eq!(p.runtime.calls, ["reset", "execute"]);
        assert_eq!(p.ram.read_u8(0x351), 2)
    }

    #[test]
    fn cardinal_collision_reconstructs_velocity_and_direction_from_safe_position() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = RecordingRt::default();
        ram.write_u16(0xec, 0xffff);
        ram.set_player_y(0x101);
        ram.set_player_x(0x0ff);
        ram.write_u8(0x3e, 0);
        ram.write_u8(0x3f, 0);
        ram.set_player_action_state(PlayerState::Hookshot as u8);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_handle_cardinal_collision();
        assert_eq!(p.ram.player().velocity_y(), 1);
        assert_eq!(p.ram.player().velocity_x(), 0xff);
        assert_eq!(p.ram.read_u8(0x67), 6)
    }

    #[test]
    fn diagonal_kickback_clears_stale_deadlock_without_two_axes() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_velocity_x(1);
        ram.set_player_velocity_y(0);
        ram.write_u8(0x6d, 0xff);
        ram.write_u8(0x6b, 0x3f);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_handle_diagonal_kickback();
        assert_eq!(p.ram.read_u8(0x6d), 0);
        assert_eq!(p.ram.read_u8(0x6b), 0)
    }

    #[test]
    fn ledge_timer_wraps_and_reports_hop_without_restoring_position() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x375, 0);
        ram.set_player_y(0x1234);
        ram.set_player_x(0x5678);
        ram.write_u16(0xfc4, 0xabcd);
        ram.write_u16(0xfc2, 0xef01);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        assert!(p.run_ledge_hop_timer());
        assert_eq!(p.ram.read_u8(0x375), 19);
        assert_eq!(p.ram.player().y(), 0x1234);
        assert_eq!(p.ram.player().x(), 0x5678)
    }

    #[test]
    fn perpendicular_door_x_preserves_original_signed_velocity_quirk() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u16(0x0e, 1);
        ram.set_player_y(0x80);
        ram.set_player_velocity_x(2);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        assert_eq!(p.change_axis_of_perpendicular_door_movement_x(), -2);
        assert_eq!(p.ram.player().y(), 0x7e);
        assert_eq!(p.ram.player().facing(), 0)
    }

    #[test]
    fn tile_rebound_sets_axis_velocity_and_clears_colliding_input_axis() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x66, 2);
        ram.set_player_velocity_y(9);
        ram.set_player_velocity_x(7);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_apply_tile_rebound();
        assert_eq!(p.ram.player().actual_velocity_x(), 24);
        assert_eq!(p.ram.player().actual_velocity_y(), 0);
        assert_eq!(p.ram.player().velocity_y(), 0);
        assert_eq!(p.ram.player().velocity_x(), 7);
        assert_eq!(p.ram.read_u8(0x46), 24)
    }

    #[test]
    fn change_z_preserves_negative_landing_sentinels() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_actual_velocity_z(0xfe);
        ram.set_player_z(0x8081);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.player_change_z(2);
        assert_eq!(p.ram.player().z(), 0xffff);
        assert_eq!(p.ram.player().actual_velocity_z(), 0xff);
        p.ram.set_player_z(0x8001);
        p.ram.set_player_actual_velocity_z(0xfe);
        p.player_change_z(2);
        assert_eq!(p.ram.player().z(), 0x8001);
        assert_eq!(p.ram.player().actual_velocity_z(), 0xfc);
        p.ram.set_player_z(0);
        p.ram.set_player_actual_velocity_z(0xfe);
        p.player_change_z(2);
        assert_eq!(p.ram.player().actual_velocity_z(), 0xfe)
    }

    #[test]
    fn diagonal_north_landing_search_uses_original_distance_tables() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = LandingRt;
        ram.set_player_y(100);
        ram.set_player_x(200);
        ram.write_u8(0x3e, 90);
        ram.write_u8(0x66, 2);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_find_valid_landing_tile_diagonal_north();
        assert_eq!(p.ram.player().y(), 100);
        assert_eq!(p.ram.player().x(), 200);
        assert_eq!(p.ram.player().actual_velocity_y(), 0xf0);
        assert_eq!(p.ram.player().actual_velocity_x(), 0xf0);
        assert_eq!(p.ram.player().actual_velocity_z(), 32);
        assert_eq!(p.ram.player().action_state(), 13)
    }

    #[test]
    fn push_block_timeout_does_not_reset_before_signed_underflow() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x1b, 1);
        ram.write_u16(0x5f, 1);
        ram.write_u8(0x61, 3);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.handle_moving_push_blocks(false);
        assert_eq!(p.ram.read_u8(0x61), 2)
    }

    #[test]
    fn tablet_spells_use_scripted_coordinates_then_restore_link() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = SpellRt::default();
        ram.set_player_x(0x1234);
        ram.set_player_y(0x5678);
        ram.write_u16(0x3c, 161);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_state_receiving_ether();
        assert_eq!(p.runtime.ether_position, Some((0x6b0, 0x37)));
        assert_eq!(p.ram.player().x(), 0x1234);
        assert_eq!(p.ram.player().y(), 0x5678);
        p.ram.write_u16(0x3c, 161);
        p.link_state_receiving_bombos();
        assert_eq!(p.runtime.bombos_position, Some((0x378, 0xeb0)));
        assert_eq!(p.ram.player().x(), 0x1234);
        assert_eq!(p.ram.player().y(), 0x5678)
    }

    #[test]
    fn edge_transition_restores_previous_coordinates_and_stops_recoil() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_action_state(PlayerState::RecoilOther as u8);
        ram.write_u8(0x46, 1);
        ram.set_player_actual_velocity_y(9);
        ram.set_player_actual_velocity_x(7);
        ram.write_u16(0xfc4, 0x1111);
        ram.write_u16(0xfc2, 0x2222);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        assert!(p.link_check_for_edge_screen_transition());
        assert_eq!(p.ram.player().y(), 0x1111);
        assert_eq!(p.ram.player().x(), 0x2222);
        assert_eq!(
            p.ram.player().actual_velocity_y() | p.ram.player().actual_velocity_x(),
            0
        );
        assert_eq!(p.ram.read_u8(0x2c6), 3)
    }

    #[test]
    fn velocity_pipeline_uses_signed_fixed_point_and_saves_unmoved_coordinates() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_x(0x1200);
        ram.set_player_y(0x3400);
        ram.write_u8(0x67, 1);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_handle_velocity();
        assert_eq!(p.ram.player().actual_velocity_x(), 24);
        assert_eq!(p.ram.player().x(), 0x1201);
        assert_eq!(p.ram.player().subpixel_x(), 0x80);
        assert_eq!(p.ram.read_u8(0x3f), 0);
        assert_eq!(p.ram.read_u8(0x41), 0x12);
        assert_eq!(p.ram.player().velocity_x(), 1);
        assert_eq!(p.ram.player().z(), 0xffff)
    }

    #[test]
    fn move_position_preserves_negative_subpixel_borrow() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_actual_velocity_x(0xff);
        ram.set_player_actual_velocity_y(1);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_move_position();
        assert_eq!(p.ram.player().x(), 0xffff);
        assert_eq!(p.ram.player().subpixel_x(), 0xf0);
        assert_eq!(p.ram.player().y(), 0);
        assert_eq!(p.ram.player().subpixel_y(), 0x10);
        assert_eq!(p.ram.player().velocity_x(), 0xff)
    }

    #[test]
    fn moving_floor_and_conveyor_keep_their_distinct_subpixel_rules() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x46c, 1);
        ram.write_u8(0x322, 3);
        ram.set_player_z(0xffff);
        ram.write_u16(0x310, 0x00ff);
        ram.write_u16(0x312, 1);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_handle_moving_floor();
        assert_eq!(p.ram.read_u8(0x67), 9);
        assert_eq!(p.ram.player().y(), 0x00ff);
        assert_eq!(p.ram.player().x(), 1);

        p.ram.set_player_y(0x0100);
        p.ram.set_player_x(0x0200);
        p.ram.write_u8(0x3f3, 1);
        p.ram.write_u8(0x41c, 0);
        p.ram.write_u8(0x41d, 0x55);
        p.ram.write_u8(0x67, 0);
        p.link_apply_conveyor();
        assert_eq!(p.ram.player().y(), 0x00ff);
        assert_eq!(p.ram.read_u8(0x41c), 0x80);
        assert_eq!(p.ram.player().x(), 0x0200);
        assert_eq!(p.ram.read_u8(0x41d), 0x55);
        assert_eq!(p.ram.read_u8(0x67), 8)
    }

    #[test]
    fn camera_delta_uses_split_safe_coordinate_high_bytes() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_y(0x1200);
        ram.set_player_x(0x34ff);
        ram.write_u8(0x40, 0x13);
        ram.write_u8(0x41, 0x33);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.apply_links_movement_to_camera();
        assert!(p.applied_movement_to_camera);
        assert_eq!(p.ram.read_u8(0x68), 0xff);
        assert_eq!(p.ram.read_u8(0x69), 1)
    }

    #[test]
    fn reset_properties_c_clears_full_width_misc_tile_accumulator() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u16(0x2f6, 0xffff);
        ram.write_u8(0x3a, 0xff);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_reset_properties_c();
        assert_eq!(p.ram.read_u16(0x2f6), 0);
        assert_eq!(p.ram.read_u8(0x3a), 0)
    }

    #[test]
    fn sleeping_countdown_does_not_tick_while_a_submodule_is_active() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x37c, 1);
        ram.write_u8(0x374, 3);
        ram.write_u8(0x11, 1);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_state_sleeping();
        assert_eq!(p.ram.read_u8(0x374), 3)
    }

    #[test]
    fn swim_acceleration_rounds_up_to_the_original_step_table() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0xf0, 8);
        ram.write_u16(0x33c, 225);
        ram.write_u16(0x334, 384);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_handle_swim_accels();
        assert_eq!(p.ram.read_u16(0x334), 256)
    }

    #[test]
    fn cape_preserves_original_zero_magic_underflow_without_bugfix_feature() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x55, 1);
        ram.write_u8(0x4c, 1);
        ram.write_u8(0x2e2, 1);
        ram.set_player_magic(0);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_item_cape();
        assert_eq!(p.ram.player().magic(), 0xff);
        assert_eq!(p.ram.read_u8(0x55), 1)
    }

    #[test]
    fn gravestone_barrier_pushes_link_to_bottom_edge_and_accumulates_y_velocity() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0xc04, 0x00);
        ram.write_u8(0xc18, 0x01);
        ram.write_u8(0xbfa, 0x00);
        ram.write_u8(0xc0e, 0x02);
        ram.set_player_x(0x100);
        ram.set_player_y(0x208);
        ram.set_player_velocity_y(3);
        ram.set_player_facing(6);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.gravestone_act_as_barrier(0);
        assert_eq!(p.ram.player().y(), 0x210);
        assert_eq!(p.ram.player().velocity_y(), 11);
        assert_eq!(p.ram.read_u8(0x48) & 4, 4);
        assert_eq!(p.ram.player().facing(), 2)
    }

    #[test]
    fn red_potion_consumption_enters_refill_module_and_replaces_bottle() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0xf4, 0x40);
        ram.write_u8(0xf34f, 1);
        ram.write_u8(0xf35c, 3);
        ram.set_player_max_health(0x50);
        ram.set_player_health(0x30);
        ram.write_u8(0x10, 7);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_item_bottle();
        assert_eq!(p.ram.read_u8(0xf35c), 2);
        assert_eq!(p.ram.read_u8(0x10c), 7);
        assert_eq!(p.ram.read_u8(0x10), 14);
        assert_eq!(p.ram.read_u8(0x11), 4);
        assert_eq!(p.ram.read_u8(0x208), 7)
    }

    #[test]
    fn recoil_jump_entry_runs_the_shared_landing_cleanup() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        for address in [
            0x4d, 0x5e, 0x50, 0x301, 0x37a, 0x300, 0x37b, 0x360, 0x28, 0x27,
        ] {
            ram.write_u8(address, 0xff);
        }
        ram.set_player_action_state(PlayerState::RecoilOther as u8);
        ram.write_u8(0xee, 2);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_handle_recoil_and_timer(true);
        assert_eq!(p.ram.player().z(), 0);
        for address in [
            0x4d, 0x5e, 0x50, 0x301, 0x37a, 0x300, 0x37b, 0x360, 0x28, 0x27,
        ] {
            assert_eq!(p.ram.read_u8(address), 0, "address {address:#x}");
        }
        assert_eq!(p.ram.read_u8(0xee), 0);
    }

    #[test]
    fn victory_spin_selects_direction_table_and_enters_first_delay() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_facing(4);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_animate_victory_spin();
        assert_eq!(
            p.ram.player().action_state(),
            PlayerState::SpinAttacking as u8
        );
        assert_eq!(p.ram.read_u8(0x31e), 24);
        assert_eq!(p.ram.read_u8(0x31c), LINK_SPIN_GRAPHICS_BY_DIR[24]);
        assert_eq!(p.ram.read_u8(0x3d), 2);
        assert_eq!(p.ram.read_u8(0x3a), 0x80);
    }

    #[test]
    fn desert_prayer_sets_the_complete_module_and_animation_contract() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x10, 7);
        ram.write_u8(0x50, 0x80);
        ram.write_u8(0x67, 0xff);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_perform_desert_prayer();
        assert_eq!(
            (
                p.ram.read_u8(0x10c),
                p.ram.read_u8(0x10),
                p.ram.read_u8(0x11)
            ),
            (7, 14, 5)
        );
        assert_eq!((p.ram.read_u8(0x30b), p.ram.read_u8(0x308)), (22, 2));
        assert_eq!((p.ram.read_u8(0x12d), p.ram.read_u8(0x12c)), (17, 242));
        assert_eq!(p.ram.read_u8(0x50), 0x81);
        assert_eq!(p.ram.read_u8(0x67), 0xf0);
    }

    #[test]
    fn exiting_dash_uses_the_dash_countdown_not_the_sword_delay() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x374, 15);
        ram.write_u8(0x3d, 7);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_state_exiting_dash();
        assert_eq!(p.ram.read_u8(0x374), 16);
        assert_eq!(p.ram.read_u8(0x3d), 7);
    }

    #[test]
    fn horizontal_hop_selects_direction_from_signed_x_velocity() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_actual_velocity_x(0xff);
        ram.write_u8(0x362, 4);
        ram.write_u8(0x364, 0x20);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_state_hopping_horizontally_ow();
        assert_eq!(p.ram.read_u8(0x67), 6);
        assert_eq!(p.ram.read_u8(0x50), 0);
        assert_eq!(p.ram.read_u8(0x351), 0);
    }

    #[test]
    fn upward_collision_snap_preserves_signed_c_integer_promotion() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.set_player_y(0x0dca);
        ram.write_u8(0x51, 2);
        ram.set_player_velocity_y(0xfe);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_add_in_velocity_y_falling();
        assert_eq!(p.ram.player().y(), 0x0dd0);
    }

    #[test]
    fn pending_sprite_damage_is_applied_before_player_dispatch() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        let mut rt = Rt;
        ram.write_u8(0x373, 8);
        ram.set_player_health(0x88);
        ram.set_player_action_state(22);
        let mut p = Player {
            ram: &mut ram,
            runtime: &mut rt,
            applied_movement_to_camera: false,
        };
        p.link_control_handler();
        assert_eq!(p.ram.read_u8(0x373), 0);
        assert_eq!(p.ram.player().health(), 0x80);
        assert_eq!(p.ram.read_u8(0x31f), 58);
        assert_eq!(p.ram.read_u8(0xcfc), 1);
    }

    #[test]
    fn recoil_do_while_shifts_twice_only_for_counter_one() {
        assert_eq!(recoil_z_velocity(36, 1), 9);
        assert_eq!(recoil_z_velocity(36, 2), 18);
        assert_eq!(recoil_z_velocity(36, 3), 18);
    }
}
