//! One-for-one translation of `src/sprite_main.c` definitions beginning in
//! source lines 7001 through 14000.

#![allow(dead_code)]

use crate::ram::Ram;
use crate::sprite_main_bank0::SPRITE_SIMPLIFIED_TILE_ATTRIBUTE;

const TMP_COUNTER: usize = 0xfb5;

pub const DASH_TREE_TOP_CHAR_FLAGS: [u16; 16] = [
    0x3100, 0x3102, 0x7102, 0x7100, 0x3120, 0x3122, 0x7122, 0x7120, 0x3104, 0x3106, 0x7106, 0x7104,
    0x3124, 0x3126, 0x7126, 0x7124,
];
pub const DASH_TREE_TOP_X: [i8; 16] = [10, 22, 30, 1, 34, 5, 13, 29, 0, 17, 27, 44, 15, 33, 18, 26];
pub const DASH_TREE_TOP_Y: [i8; 16] = [0, 4, 2, 7, 10, 16, 24, 23, 34, 35, 30, 31, 46, 42, 10, 11];
pub const BOTTLE_VENDOR_FISH_REWARD_TYPE: [u8; 5] = [0xdb, 0xe0, 0xde, 0xe2, 0xd9];
pub const BOTTLE_VENDOR_FISH_REWARD_XV: [i8; 5] = [-6, -3, 0, 4, 7];
pub const BOTTLE_VENDOR_FISH_REWARD_YV: [i8; 5] = [11, 14, 16, 14, 11];
pub const TROUGH_BOY_DRAW: [(i8, i8, u16, u8); 8] = [
    (0, -8, 0x0882, 2),
    (0, 0, 0x0aaa, 2),
    (0, -8, 0x0882, 2),
    (0, 0, 0x0aaa, 2),
    (0, -8, 0x4880, 2),
    (0, 0, 0x0aaa, 2),
    (0, -8, 0x0880, 2),
    (0, 0, 0x0aaa, 2),
];
pub const MOBLIN_DRAW: [(i8, i8, u16, u8); 48] = [
    (-2, 3, 0x8091, 0),
    (-2, 11, 0x8090, 0),
    (0, -10, 0x0086, 2),
    (0, 0, 0x008a, 2),
    (-2, 7, 0x8091, 0),
    (-2, 15, 0x8090, 0),
    (0, -10, 0x0086, 2),
    (0, 0, 0x408a, 2),
    (0, -9, 0x0084, 2),
    (0, 0, 0x00a0, 2),
    (11, -5, 0x0090, 0),
    (11, 3, 0x0091, 0),
    (0, -9, 0x0084, 2),
    (0, 0, 0x40a0, 2),
    (11, -8, 0x0090, 0),
    (11, 0, 0x0091, 0),
    (-4, 8, 0x0080, 0),
    (4, 8, 0x0081, 0),
    (0, -9, 0x0088, 2),
    (0, 0, 0x00a6, 2),
    (-9, 6, 0x0080, 0),
    (-1, 6, 0x0081, 0),
    (0, -8, 0x0088, 2),
    (0, 0, 0x00a4, 2),
    (12, 8, 0x4080, 0),
    (4, 8, 0x4081, 0),
    (0, -9, 0x4088, 2),
    (0, 0, 0x40a6, 2),
    (17, 6, 0x4080, 0),
    (9, 6, 0x4081, 0),
    (0, -8, 0x4088, 2),
    (0, 0, 0x40a4, 2),
    (-3, -5, 0x8091, 0),
    (-3, 3, 0x8090, 0),
    (0, -10, 0x0086, 2),
    (0, 0, 0x00a8, 2),
    (11, -11, 0x0090, 0),
    (11, -3, 0x0091, 0),
    (0, -9, 0x0084, 2),
    (0, 0, 0x4082, 2),
    (-2, -3, 0x0080, 0),
    (6, -3, 0x0081, 0),
    (0, -9, 0x0088, 2),
    (0, 0, 0x00a2, 2),
    (10, -3, 0x4080, 0),
    (2, -3, 0x4081, 0),
    (0, -9, 0x4088, 2),
    (0, 0, 0x40a2, 2),
];
pub const SNAPDRAGON_DRAW: [(i8, i8, u16, u8); 32] = [
    (4, -8, 0x008f, 0),
    (12, -8, 0x009f, 0),
    (-4, 0, 0x008c, 2),
    (4, 0, 0x008d, 2),
    (4, -8, 0x002b, 0),
    (12, -8, 0x003b, 0),
    (-4, 0, 0x0028, 2),
    (4, 0, 0x0029, 2),
    (-4, -8, 0x003c, 0),
    (4, -8, 0x003d, 0),
    (-4, 0, 0x00aa, 2),
    (4, 0, 0x00ab, 2),
    (-4, -8, 0x003e, 0),
    (4, -8, 0x003f, 0),
    (-4, 0, 0x00ad, 2),
    (4, 0, 0x00ae, 2),
    (-4, -8, 0x409f, 0),
    (4, -8, 0x408f, 0),
    (-4, 0, 0x408d, 2),
    (4, 0, 0x408c, 2),
    (-4, -8, 0x403b, 0),
    (4, -8, 0x402b, 0),
    (-4, 0, 0x4029, 2),
    (4, 0, 0x4028, 2),
    (4, -8, 0x403d, 0),
    (12, -8, 0x403c, 0),
    (-4, 0, 0x40ab, 2),
    (4, 0, 0x40aa, 2),
    (4, -8, 0x403f, 0),
    (12, -8, 0x403e, 0),
    (-4, 0, 0x40ae, 2),
    (4, 0, 0x40ad, 2),
];
pub const HINOX_DRAW: [(i8, i8, u16, u8); 46] = [
    (0, -13, 0x0600, 2),
    (-8, -5, 0x0624, 2),
    (8, -5, 0x4624, 2),
    (0, 1, 0x0606, 2),
    (0, -13, 0x0600, 2),
    (-8, -5, 0x0624, 2),
    (8, -5, 0x4624, 2),
    (0, 1, 0x4606, 2),
    (-8, -6, 0x0624, 2),
    (8, -6, 0x4624, 2),
    (0, 0, 0x0606, 2),
    (0, -13, 0x0604, 2),
    (-8, -6, 0x0624, 2),
    (8, -6, 0x4624, 2),
    (0, 0, 0x4606, 2),
    (0, -13, 0x0604, 2),
    (-3, -13, 0x0602, 2),
    (0, -8, 0x060c, 2),
    (0, 0, 0x061c, 2),
    (-3, -12, 0x0602, 2),
    (0, -8, 0x060e, 2),
    (0, 0, 0x061e, 2),
    (3, -13, 0x4602, 2),
    (0, -8, 0x460c, 2),
    (0, 0, 0x461c, 2),
    (3, -12, 0x4602, 2),
    (0, -8, 0x460e, 2),
    (0, 0, 0x461e, 2),
    (-13, -16, 0x056e, 2),
    (0, -13, 0x0600, 2),
    (-8, -5, 0x0620, 2),
    (8, -5, 0x4624, 2),
    (0, 1, 0x0606, 2),
    (-8, -5, 0x0624, 2),
    (8, -5, 0x4620, 2),
    (0, 1, 0x0606, 2),
    (0, -13, 0x0604, 2),
    (13, -16, 0x056e, 2),
    (-8, -11, 0x056e, 2),
    (-3, -13, 0x0602, 2),
    (0, 0, 0x0622, 2),
    (0, -8, 0x060c, 2),
    (8, -11, 0x056e, 2),
    (3, -13, 0x4602, 2),
    (0, 0, 0x4622, 2),
    (0, -8, 0x460c, 2),
];
pub const FLUTE_BOY_DRAW: [(i8, i8, u16, u8); 16] = [
    (-1, -1, 0x0abe, 0),
    (0, 0, 0x0aaa, 2),
    (0, -10, 0x0aa8, 2),
    (0, 0, 0x0aaa, 2),
    (-1, -1, 0x0abe, 0),
    (0, 8, 0x0abf, 0),
    (0, -10, 0x0aa8, 2),
    (0, 0, 0x0aaa, 2),
    (-1, -1, 0x0abe, 0),
    (0, 0, 0x0aaa, 2),
    (0, -10, 0x0aa8, 2),
    (0, 0, 0x0aaa, 2),
    (-1, -1, 0x0abe, 0),
    (0, 8, 0x0abf, 0),
    (0, -10, 0x0aa8, 2),
    (0, 0, 0x0aaa, 2),
];
pub const FLUTE_AARDVARK_DRAW: [(i8, i8, u16, u8); 8] = [
    (0, -16, 0x06e6, 2),
    (0, -8, 0x06c8, 2),
    (0, -16, 0x06e6, 2),
    (0, -8, 0x06ca, 2),
    (0, -16, 0x06e8, 2),
    (0, -8, 0x06ca, 2),
    (0, -16, 0x00cc, 2),
    (0, -8, 0x00dc, 2),
];

pub trait SpriteMainBank1Runtime {
    fn draw_single_large(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn return_if_inactive(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_damage_to_link_same_layer(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn move_xy(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_y(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_x(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn move_z(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sfx2_with_pan(&mut self, _ram: &mut Ram<'_>, _k: usize, _sound: u8) {}
    fn link_cancel_dash(&mut self, _ram: &mut Ram<'_>) {}
    fn link_receive_item(&mut self, _ram: &mut Ram<'_>, _item: u8, _method: i32) {}
    fn random_number(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
    fn prep_oam(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<(u8, u8)> {
        None
    }
    fn prep_oam_wide(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<(u16, u16)> {
        None
    }
    fn prep_oam_with_flags(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<(u8, u8, u8)> {
        None
    }
    fn prep_oam_with_flags_wide(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
    ) -> Option<(u16, u16, u8)> {
        None
    }
    fn emit_oam(&mut self, _ram: &mut Ram<'_>, _k: usize, _x: u8, _y: u8, _char: u8, _flags: u8) {}
    fn emit_oam_wide(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _x: u16,
        _y: u16,
        _char: u8,
        _flags: u8,
    ) {
    }
    fn nullify_hookshot_drag(&mut self, _ram: &mut Ram<'_>) {}
    fn repel_dash(&mut self, _ram: &mut Ram<'_>) {}
    fn spawn_dynamic(&mut self, _ram: &mut Ram<'_>, _source: usize, _kind: u8) -> Option<usize> {
        None
    }
    fn set_spawned_coordinates(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn spawn_dynamic_xy(&mut self, _ram: &mut Ram<'_>, _source: usize, _kind: u8) -> Option<usize> {
        None
    }
    fn draw_multiple(&mut self, _ram: &mut Ram<'_>, _k: usize, _pieces: &[(i8, i8, u16, u8)]) {}
    fn draw_shadow(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn behave_as_barrier(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn track_body_to_head(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn direction_to_face_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn show_solicited_message(&mut self, _ram: &mut Ram<'_>, _k: usize, _msg: u16) -> u16 {
        0
    }
    fn show_unconditional_message(&mut self, _ram: &mut Ram<'_>, _msg: u16) {}
    fn fish_overlaps_merchant(
        &mut self,
        _ram: &mut Ram<'_>,
        _merchant: usize,
        _fish: usize,
    ) -> bool {
        false
    }
    fn sfx3_with_pan(&mut self, _ram: &mut Ram<'_>, _k: usize, _sound: u8) {}
    fn decode_animated_sprite_tile(&mut self, _ram: &mut Ram<'_>, _tile: u8) {}
    fn sprite_init_bump_damage(&mut self, _kind: u8) -> u8 {
        0
    }
    fn apply_speed_towards_link(&mut self, _ram: &mut Ram<'_>, _k: usize, _speed: u8) {}
    fn is_right_of_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn initialized_segmented(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn swamola_initialize_segments(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn blind_prepare_battle(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn ganon_animation_idle(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn cutscene_agahnim_spawn_zelda(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn vitreous_spawn_smaller_eyes(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn spawn_bully(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn trinexx_components_initialize(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn helmasaur_king_initialize(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn load_properties(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn prep_snitches(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn heart_upgrade_check_obtained(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn load_follower_graphics(&mut self, _ram: &mut Ram<'_>) {}
    fn follower_initialize(&mut self, _ram: &mut Ram<'_>) {}
    fn arrghus_handle_puffs(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn talking_tree_spawn_eyeball(&mut self, _ram: &mut Ram<'_>, _k: usize, _direction: u8) {}
    fn shopkeeper_spawn_item(&mut self, _ram: &mut Ram<'_>, _k: usize, _position: u8, _what: u8) {}
    fn active_cross_bank(&mut self, _ram: &mut Ram<'_>, _k: usize, _kind: u8) {}
    fn priest_spawn_mantle(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn prep_cross_bank(&mut self, _ram: &mut Ram<'_>, _k: usize, _kind: u8) {}
    fn vulture_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn return_if_recoiling(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_damage_to_and_from_link(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn project_speed_towards_link(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _speed: u8,
    ) -> (i8, i8, i16, i16) {
        (0, 0, 0, 0)
    }
    fn check_tile_collision(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn check_tile_collision2(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn spawn_dynamic_ex(
        &mut self,
        _ram: &mut Ram<'_>,
        _source: usize,
        _kind: u8,
        _slot: usize,
    ) -> Option<usize> {
        None
    }
    fn moldorm_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn convert_velocity_to_angle(&mut self, _x_velocity: u8, _y_velocity: u8) -> u8 {
        0
    }
    fn giant_moldorm_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn check_damage_from_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn check_damage_to_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn calculate_sfx_pan(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn misc_bug_fixes_enabled(&self) -> bool {
        false
    }
    fn is_below_link(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn hide_oam_piece_if_small(&mut self, _ram: &mut Ram<'_>, _piece: usize) {}
    fn set_oam_piece_char_and_hflip(
        &mut self,
        _ram: &mut Ram<'_>,
        _piece: usize,
        _character: u8,
        _hflip: u8,
    ) {
    }
    fn antfairy_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bounce_from_tile_collision(&mut self, _ram: &mut Ram<'_>, _k: usize) -> u8 {
        0
    }
    fn chicken_incr_subtype2(&mut self, _ram: &mut Ram<'_>, _k: usize, _amount: u8) {}
    fn return_if_lifted(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn draw_distress(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16, _time: u8) {}
    fn direction_to_face_link_with_differences(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
    ) -> (u8, i8, i8) {
        (0, 0, 0)
    }
    fn return_if_lifted_permissive(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn set_oam_piece_character_and_low_flags(
        &mut self,
        _ram: &mut Ram<'_>,
        _piece: usize,
        _character: u8,
        _low_flags: u8,
        _flag_mask: u8,
    ) {
    }
    fn throwable_scenery_interact(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn oam_allocate_region_a(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn oam_allocate_region_b(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn schedule_for_breakage(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_multiple_player_deferred(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _pieces: &[(i8, i8, u16, u8)],
    ) {
    }
    fn draw_multiple_and_get_offscreen(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _pieces: &[(i8, i8, u16, u8)],
    ) -> u8 {
        0
    }
    fn flute_boy_check_if_player_close(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn palette_assert_translucency_swap(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_filter_sp5f(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_filter_restore_sp5f(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_revert_translucency_swap(&mut self, _ram: &mut Ram<'_>) {}
    fn become_follower(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn return_if_paused(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn apply_ricochet(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn place_weapon_tink(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn garnish_spawn_sparkle(&mut self, _ram: &mut Ram<'_>, _k: usize, _a: u8, _b: u8) {}
    fn oam_allocate_defer_to_player(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn check_if_link_is_busy(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn hobo_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn oam_allocate_region_c(&mut self, _ram: &mut Ram<'_>, _bytes: u8) {}
    fn get_16bit_coords(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn show_message_on_contact(&mut self, _ram: &mut Ram<'_>, _k: usize, _msg: u16) -> u16 {
        0
    }
    fn elder_wife_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn elder_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_sahasrahla(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_aginah(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn emit_oam_sized(&mut self, _ram: &mut Ram<'_>, _k: usize, _piece: (u8, u8, u8, u8, u8)) {}
    fn correct_oam_entries(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _last_entry: usize,
        _size: u8,
    ) {
    }
    fn draw_absorbable_transient(&mut self, _ram: &mut Ram<'_>, _k: usize, _shadow: bool) -> bool {
        false
    }
    fn check_absorption_by_player(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn handle_dragging_by_ancilla(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn draw_ripple_if_in_water(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn bounce_off_wall(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn spawn_small_splash(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<usize> {
        None
    }
    fn garnish_alloc(&mut self, _ram: &mut Ram<'_>) -> usize {
        0
    }
    fn garnish_alloc_force(&mut self, _ram: &mut Ram<'_>) -> usize {
        0
    }
    fn link_reset_properties_c(&mut self, _ram: &mut Ram<'_>) {}
    fn landmine_check_detonation_from_hammer(&mut self, _ram: &mut Ram<'_>, _k: usize) -> bool {
        false
    }
    fn spawn_bomb(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<usize> {
        None
    }
    fn move_xyz(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn show_message_minimal(&mut self, _ram: &mut Ram<'_>) {}
    fn overworld_draw_map16_persist(&mut self, _ram: &mut Ram<'_>, _pos: u16, _tile: u16) {}
    fn thief_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn fortune_teller_light_or_dark_world(&mut self, _ram: &mut Ram<'_>, _k: usize, _dark: bool) {}
    fn prep_and_draw_single_large_no_prep(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _x: u16,
        _y: u16,
        _flags: u8,
    ) {
    }
    fn sprite_func3(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn octoballoon_find(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn apply_recoil_to_link(&mut self, _ram: &mut Ram<'_>, _k: usize, _speed: u8) {}
    fn invert_speed_xy(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_uncle(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_priest(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_sanctuary_mantle(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_evil_barrier(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn tutorial_soldier_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn tutorial_guard_show_message_on_contact(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _msg: u16,
    ) -> bool {
        false
    }
    fn medallion_tablet_main(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_dust_cloud(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn rupee_pull_spawn_prize(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn spawn_poof_garnish(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn open_gargoyles_domain(&mut self, _ram: &mut Ram<'_>) {}
    fn spawn_dust_cloud(&mut self, _ram: &mut Ram<'_>, _k: usize) -> Option<usize> {
        None
    }
    fn talking_tree_mouth(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn talking_tree_eye(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_mad_batter_bolt(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn palette_restore_bg_and_hud(&mut self, _ram: &mut Ram<'_>) {}
    fn hud_refresh_icon(&mut self, _ram: &mut Ram<'_>) {}
    fn check_player_hitbox_overlap(
        &mut self,
        _ram: &mut Ram<'_>,
        _x: u16,
        _y: u16,
        _width: u8,
        _height: u8,
    ) -> bool {
        false
    }
    fn ancilla_add_gt_cutscene(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_waterfall_splash(&mut self, _ram: &mut Ram<'_>) {}
    fn create_pyramid_hole(&mut self, _ram: &mut Ram<'_>) {}
    fn garnish_spawn_pyramid_debris(
        &mut self,
        _ram: &mut Ram<'_>,
        _x: i8,
        _y: i8,
        _x_vel: i8,
        _y_vel: i8,
    ) {
    }
    fn emit_absolute_oam(
        &mut self,
        _ram: &mut Ram<'_>,
        _index: usize,
        _piece: (i8, i8, u8, u8, u8),
    ) {
    }
    fn handle_absorption_by_player(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn faerie_handle_movement(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn find_empty_bottle(&mut self, _ram: &mut Ram<'_>) -> Option<usize> {
        None
    }
    fn ancilla_check_damage_to_sprite_preset(
        &mut self,
        _ram: &mut Ram<'_>,
        _j: usize,
        _preset: u8,
    ) {
    }
    fn get_tile_attribute(&mut self, _ram: &mut Ram<'_>, _floor: u8, _x: u16, _y: u16) -> u8 {
        0
    }
    fn overworld_read_tile_attribute(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) -> u8 {
        0
    }
    fn project_speed_towards_location(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _x: u16,
        _y: u16,
        _speed: u8,
    ) -> (i8, i8) {
        (0, 0)
    }
    fn fish_draw(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn thrown_sprite_tile_and_sprite_interaction(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn draw_single_small(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn sprite_wish_pond3(&mut self, _ram: &mut Ram<'_>, _k: usize) {}
    fn link_reset_properties_a(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_terminate_sparkle_objects(&mut self, _ram: &mut Ram<'_>) {}
    fn add_happiness_pond_rupees(&mut self, _ram: &mut Ram<'_>, _choice: u8) {}
    fn palette_filter_wish_ponds(&mut self, _ram: &mut Ram<'_>) {}
    fn wish_pond_item_oam_flag(&mut self, _graphics: u8) -> u8 {
        5
    }
    fn receive_item_draw_kind(&mut self, _graphics: u8) -> u8 {
        0
    }
}

pub struct SpriteMainBank1<'a, 'r, R: SpriteMainBank1Runtime> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
}

impl<R: SpriteMainBank1Runtime> SpriteMainBank1<'_, '_, R> {
    fn sprite_x(&self, k: usize) -> u16 {
        u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()])
    }

    fn sprite_y(&self, k: usize) -> u16 {
        u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()])
    }

    fn set_sprite_x(&mut self, k: usize, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.ram.set_sprite_x_low(k, lo);
        self.ram.set_sprite_x_high(k, hi);
    }

    fn set_sprite_y(&mut self, k: usize, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.ram.set_sprite_y_low(k, lo);
        self.ram.set_sprite_y_high(k, hi);
    }

    pub fn sprite_book_of_mudora(&mut self, k: usize) {
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            self.ram.set_sprite_ai_state(k, 3);
        }
        self.runtime.move_xy(self.ram, k);
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(1));
        self.runtime.move_z(self.ram, k);
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_velocity_y(k, 0);
            self.ram.set_sprite_z(k, 0);
            let velocity = (0u8.wrapping_sub(self.ram.sprite(k).velocity_z())) >> 2;
            self.ram.set_sprite_velocity_z(k, velocity);
            if velocity & 254 != 0 {
                self.runtime.sfx2_with_pan(self.ram, k, 0x21);
            }
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                let x =
                    u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()]);
                let y =
                    u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()]);
                if self.ram.read_u8(0x2f) == 0
                    && x.wrapping_sub(self.ram.read_u16(0x22)).wrapping_add(39) < 47
                    && y.wrapping_sub(self.ram.read_u16(0x20)).wrapping_add(40) < 46
                    && self.ram.read_u16(0x11a) | self.ram.read_u16(0x11c) != 0
                {
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 => {
                self.ram.set_sprite_velocity_z(k, 32);
                self.ram.set_sprite_velocity_y(k, (-5i8) as u8);
                self.ram.write_u8(0x12f, 27);
                self.ram.set_sprite_ai_state(k, 2);
            }
            2 => {
                if self.ram.sprite(k).z() == 0 {
                    self.ram.set_sprite_floor(k, self.ram.read_u8(0xee));
                }
            }
            3 => {
                self.runtime.link_cancel_dash(self.ram);
                self.ram.write_u8(0x2d8, 0);
                self.runtime.link_receive_item(self.ram, 0x1d, 0);
                self.ram.set_sprite_state(k, 0);
            }
            _ => {}
        }
    }

    pub fn sprite_lumberjack_tree(&mut self, k: usize) {
        self.ram.set_sprite_flags2(k, 0x8f);
        self.ram.set_sprite_flags4(k, 0x47);
        self.dash_tree_top_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            self.runtime.nullify_hookshot_drag(self.ram);
            self.ram.write_u8(0x5e, 0);
            self.runtime.repel_dash(self.ram);
        }
        self.runtime.move_xy(self.ram, k);
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(1));
        self.runtime.move_z(self.ram, k);
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            self.ram
                .set_sprite_velocity_z(k, 0u8.wrapping_sub(self.ram.sprite(k).velocity_z()) >> 2);
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_subtype2(k, 0);
                let x =
                    u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()]);
                let y =
                    u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()]);
                if x.wrapping_sub(self.ram.read_u16(0x22)).wrapping_add(24) < 65
                    && y.wrapping_sub(self.ram.read_u16(0x20)).wrapping_add(32) < 81
                    && (self.ram.read_u16(0x11a) | self.ram.read_u16(0x11c)) as u8 != 0
                {
                    self.ram.set_sprite_velocity_z(k, 20);
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 if self.ram.sprite(k).z() == 0 => {
                self.ram.set_sprite_ai_state(k, 2);
                self.ram.write_u8(0x12f, 0x1b);
                self.ram.set_sprite_velocity_x(k, (-4i8) as u8);
                self.ram.set_sprite_velocity_y(k, (-4i8) as u8);
                for (xv, yv) in [(5i8, 5i8), (5, -4), (-4, 4)] {
                    if let Some(j) = self.lumberjack_tree_spawn_leaves(k) {
                        self.ram.set_sprite_velocity_x(j, xv as u8);
                        self.ram.set_sprite_velocity_y(j, yv as u8);
                    }
                }
            }
            2 if self.ram.sprite(k).main_delay() == 0 => {
                self.ram.set_sprite_main_delay(k, 8);
                let s = self.ram.sprite(k).subtype2();
                if s == 6 {
                    self.ram.set_sprite_state(k, 0);
                }
                self.ram.set_sprite_subtype2(k, s.wrapping_add(1));
            }
            _ => {}
        }
    }

    pub fn lumberjack_tree_spawn_leaves(&mut self, k: usize) -> Option<usize> {
        let j = self.runtime.spawn_dynamic(self.ram, k, 0x3b)?;
        self.ram.set_sprite_graphics(j, 2);
        self.ram
            .set_sprite_velocity_z(j, self.ram.sprite(k).velocity_z());
        self.ram.set_sprite_subtype2(j, 1);
        self.ram.set_sprite_ai_state(j, 2);
        self.ram.set_sprite_main_delay(j, 8);
        self.runtime.set_spawned_coordinates(self.ram, j);
        Some(j)
    }

    pub fn dash_tree_top_draw(&mut self, k: usize) {
        const CH: [u8; 6] = [8, 8, 0x28, 0x28, 0x2a, 0x2a];
        const FL: [u8; 6] = [0x31, 0x71, 0x31, 0x71, 0x31, 0x71];
        let Some((x, y)) = self.runtime.prep_oam(self.ram, k) else {
            return;
        };
        let x = x.wrapping_sub(0x20);
        let y = y.wrapping_sub(0x20);
        self.ram.write_u8(0xfa8, x);
        self.ram.write_u8(0xfa9, y);
        let phase = self.ram.sprite(k).subtype2();
        let first = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for offset in 0..16 {
            let i = if phase == 0 { offset } else { 15 - offset };
            let (ox, oy, ch, fl) = if phase == 0 {
                let word = DASH_TREE_TOP_CHAR_FLAGS[i];
                (
                    ((i & 3) * 16) as u8,
                    ((i >> 2) * 16) as u8,
                    word as u8,
                    (word >> 8) as u8,
                )
            } else {
                let j = usize::from(phase - 1);
                (
                    DASH_TREE_TOP_X[i] as u8,
                    DASH_TREE_TOP_Y[i] as u8,
                    CH[j],
                    FL[j],
                )
            };
            crate::oam::write(
                self.ram,
                first + offset,
                z3_core::c_types::OamEnt {
                    x: x.wrapping_add(ox),
                    y: y.wrapping_add(oy),
                    charnum: ch,
                    flags: fl,
                },
            );
        }
        self.runtime.correct_oam_entries(self.ram, k, 15, 2);
    }

    pub fn sprite_trough_boy(&mut self, k: usize) {
        self.trough_boy_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.runtime.track_body_to_head(self.ram, k);
        let head_direction = self.runtime.direction_to_face_link(self.ram, k) ^ 3;
        self.ram.set_sprite_head_direction(k, head_direction);
        if self.ram.read_u8(0xf3c7) < 3 {
            if self.runtime.show_solicited_message(self.ram, k, 0x147) & 0x100 != 0 {
                self.ram.write_u8(0xf3c7, 2);
            }
        } else {
            self.runtime.show_solicited_message(self.ram, k, 0x148);
        }
    }
    pub fn trough_boy_draw(&mut self, k: usize) {
        let start = usize::from(self.ram.sprite(k).scratch_d()) * 2;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &TROUGH_BOY_DRAW[start..start + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }
    pub fn bottle_merchant_detect_fish(&mut self, k: usize) {
        for i in (0..16).rev() {
            if self.ram.sprite(i).state() != 0 && self.ram.sprite(i).kind() == 0xd2 {
                if self.runtime.fish_overlaps_merchant(self.ram, k, i) {
                    self.ram.set_sprite_scratch_e(k, 0x80 | i as u8);
                }
                return;
            }
        }
    }
    pub fn bottle_merchant_buy_fish(&mut self, k: usize) {
        self.runtime.sfx3_with_pan(self.ram, k, 0x13);
        self.ram.write_u8(0xfb5, 4);
        for i in (0..5).rev() {
            let Some(j) =
                self.runtime
                    .spawn_dynamic(self.ram, k, BOTTLE_VENDOR_FISH_REWARD_TYPE[i])
            else {
                return;
            };
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram
                .set_sprite_x_low(j, self.ram.sprite(j).x_low().wrapping_add(4));
            self.ram.set_sprite_stunned(j, 0xff);
            self.ram
                .set_sprite_velocity_x(j, BOTTLE_VENDOR_FISH_REWARD_XV[i] as u8);
            self.ram
                .set_sprite_velocity_y(j, BOTTLE_VENDOR_FISH_REWARD_YV[i] as u8);
            self.ram.set_sprite_velocity_z(j, 32);
            self.ram.set_sprite_aux4_delay(j, 32);
            self.ram.write_u8(0xfb5, (i as u8).wrapping_sub(1));
        }
    }

    pub fn sprite_prep_throwable_scenery(&mut self, _k: usize) {}

    pub fn sprite_prep_mantle(&mut self, k: usize) {
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_add(3));
        self.ram
            .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(8));
    }

    pub fn sprite_prep_switch(&mut self, k: usize) {
        if matches!(self.ram.read_u8(0x48e), 0xce | 4 | 0x3f) {
            self.ram.set_sprite_oam_flags(k, 0xd);
        }
    }
    pub fn sprite_prep_switch_facing_up(&mut self, _k: usize) {}
    pub fn sprite_prep_do_nothing_a(&mut self, _k: usize) {}

    fn prep_dark_world_stats(
        &mut self,
        k: usize,
        damage: [u8; 2],
        health: [u8; 2],
        flags: Option<[u8; 2]>,
    ) {
        let i = usize::from(self.ram.read_u8(0xfff) != 0);
        self.ram.set_sprite_bump_damage(k, damage[i]);
        self.ram.set_sprite_health(k, health[i]);
        if let Some(flags) = flags {
            self.ram.set_sprite_flags5(k, flags[i]);
        }
    }
    pub fn sprite_prep_rat(&mut self, k: usize) {
        self.prep_dark_world_stats(k, [0, 5], [2, 8], None);
    }
    pub fn sprite_prep_keese(&mut self, k: usize) {
        self.prep_dark_world_stats(k, [0x80, 0x85], [1, 4], Some([0, 7]));
    }
    pub fn sprite_prep_rope(&mut self, k: usize) {
        self.prep_dark_world_stats(k, [1, 5], [4, 8], Some([1, 7]));
    }

    pub fn sprite_prep_pokey(&mut self, k: usize) {
        const XV: [i8; 4] = [16, -16, 16, -16];
        const YV: [i8; 4] = [16, 16, -16, -16];
        self.ram.set_sprite_scratch_a(k, 3);
        self.ram.set_sprite_scratch_b(k, 8);
        let i = usize::from(self.runtime.random_number(self.ram) & 3);
        self.ram.set_sprite_velocity_x(k, XV[i] as u8);
        self.ram.set_sprite_velocity_y(k, YV[i] as u8);
    }
    pub fn sprite_prep_gibo(&mut self, k: usize) {
        self.ram.set_sprite_z(k, 16);
        self.ram.set_sprite_scratch_g(k, 8);
    }
    pub fn sprite_prep_octoballoon(&mut self, k: usize) {
        const D: [u8; 4] = [192, 208, 224, 240];
        self.ram.set_sprite_main_delay(k, D[k & 3]);
    }
    pub fn sprite_prep_raven(&mut self, k: usize) {
        self.prep_dark_world_stats(k, [0x81, 0x88], [4, 8], Some([6, 2]));
        self.sprite_prep_vulture(k);
    }
    pub fn sprite_prep_vulture(&mut self, k: usize) {
        self.ram.set_sprite_z(k, 0);
        self.ram
            .set_sprite_scratch_a(k, (self.ram.sprite(k).x_low() & 16) >> 4);
        self.ram.set_sprite_subtype(k, 254);
    }
    pub fn sprite_prep_poe(&mut self, k: usize) {
        self.ram.set_sprite_z(k, 12);
        self.ram.set_sprite_subtype(k, 254);
    }
    pub fn sprite_prep_do_nothing_c(&mut self, _k: usize) {}
    pub fn sprite_prep_bomber(&mut self, k: usize) {
        self.ram.set_sprite_z(k, 16);
        self.ram.set_sprite_subtype(k, 254);
    }
    pub fn sprite_prep_zoro(&mut self, k: usize) {
        self.ram
            .set_sprite_scratch_d(k, self.ram.sprite(k).kind().wrapping_sub(0x9c) << 1);
        self.ram
            .set_sprite_graphics(k, self.ram.sprite(k).graphics().wrapping_sub(1));
    }
    pub fn sprite_prep_popo(&mut self, k: usize) {
        self.ram.set_sprite_scratch_b(k, 7);
    }
    pub fn sprite_prep_popo2(&mut self, k: usize) {
        self.ram.set_sprite_scratch_b(k, 15);
    }
    pub fn sprite_prep_statue(&mut self, k: usize) {
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_add(7));
    }
    pub fn sprite_prep_green_stalfos(&mut self, k: usize) {
        self.ram.set_sprite_z(k, 9);
    }
    pub fn sprite_prep_water_lever(&mut self, k: usize) {
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_add(5));
    }
    pub fn sprite_prep_weak_guard(&mut self, k: usize) {
        let d = self.runtime.random_number(self.ram) & 3;
        self.ram.set_sprite_scratch_d(k, d);
        self.ram.set_sprite_head_direction(k, d);
        self.ram.set_sprite_main_delay(k, 16);
    }
    pub fn sprite_prep_wall_cannon(&mut self, k: usize) {
        let d = self.ram.sprite(k).kind().wrapping_sub(0x66);
        self.ram.set_sprite_scratch_d(k, d);
        self.ram.set_sprite_scratch_a(k, d & 2);
    }
    pub fn sprite_prep_ignore_projectiles(&mut self, k: usize) {
        self.ram.set_sprite_ignore_projectile(
            k,
            self.ram.sprite(k).ignore_projectile().wrapping_add(1),
        );
    }
    pub fn sprite_prep_haunted_grove_ostrich(&mut self, k: usize) {
        if self.ram.read_u8(0xf34c) >= 2 {
            self.ram.set_sprite_state(k, 0);
        }
        self.sprite_prep_ignore_projectiles(k);
    }
    pub fn sprite_prep_digging_game_guy_bounce(&mut self, k: usize) {
        let y = u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()]);
        if self.ram.read_u16(0x20) < y {
            self.ram.set_sprite_ai_state(k, 5);
            self.ram
                .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_sub(9));
            self.ram.set_sprite_graphics(k, 1);
        }
        self.sprite_prep_ignore_projectiles(k);
    }
    pub fn sprite_prep_rupee_pull(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        let x = u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()])
            .wrapping_sub(8);
        let [lo, hi] = x.to_le_bytes();
        self.ram.set_sprite_x_low(k, lo);
        self.ram.set_sprite_x_high(k, hi);
    }

    pub fn sprite_prep_whirlpool(&mut self, k: usize) {
        self.ram.set_sprite_ignore_projectile(
            k,
            self.ram.sprite(k).ignore_projectile().wrapping_add(1),
        );
        self.ram.set_sprite_scratch_a(k, 1);
    }

    pub fn sprite_prep_sage(&mut self, k: usize) {
        self.ram.set_sprite_ignore_projectile(
            k,
            self.ram.sprite(k).ignore_projectile().wrapping_add(1),
        );
        if self.ram.read_u8(0x48e) == 10 {
            self.ram
                .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
            self.ram.set_sprite_oam_flags(k, 11);
        }
    }

    pub fn sprite_prep_kiki(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        let screen = usize::from(self.ram.read_u8(0x8a));
        if self.ram.read_u8(0xf280 + screen) & 0x20 != 0 {
            self.ram.set_sprite_state(k, 0);
        }
    }

    pub fn sprite_prep_sick_kid(&mut self, k: usize) {
        if self.ram.read_u8(0xf34d) != 0 {
            self.ram.set_sprite_ai_state(k, 3);
        }
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_magic_bat(&mut self, k: usize) {
        self.ram
            .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(8));
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_master_sword(&mut self, k: usize) {
        self.ram
            .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(6));
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_add(6));
    }

    fn sprite_prep_roller(&mut self, k: usize, vertical: bool, d: u8) {
        let coordinate = if vertical {
            self.ram.sprite(k).y_low()
        } else {
            self.ram.sprite(k).x_low()
        };
        let state = if vertical {
            (coordinate & 16) >> 4
        } else {
            (!coordinate & 16) >> 4
        };
        self.ram.set_sprite_ai_state(k, state);
        if state != 0 {
            self.ram
                .set_sprite_flags4(k, self.ram.sprite(k).flags4().wrapping_add(1));
        }
        self.ram.set_sprite_scratch_d(k, d);
    }

    pub fn sprite_prep_roller_horizontal_right_first(&mut self, k: usize) {
        self.sprite_prep_roller(k, false, 0);
    }
    pub fn sprite_prep_roller_left_right(&mut self, k: usize) {
        self.sprite_prep_roller(k, false, 1);
    }
    pub fn sprite_prep_roller_vertical_down_first(&mut self, k: usize) {
        self.sprite_prep_roller(k, true, 2);
    }
    pub fn sprite_prep_roller_up_down(&mut self, k: usize) {
        self.sprite_prep_roller(k, true, 3);
    }

    pub fn sprite_prep_spark(&mut self, k: usize) {
        self.ram
            .set_sprite_subtype(k, self.ram.sprite(k).subtype().wrapping_sub(1));
    }

    pub fn sprite_prep_antifairy(&mut self, k: usize) {
        const X_VELOCITY: [i8; 2] = [16, -16];
        let j = usize::from((self.ram.sprite(k).x_low() >> 4) & 1);
        self.ram.set_sprite_velocity_x(k, X_VELOCITY[j] as u8);
        self.ram.set_sprite_velocity_y(k, (-16i8) as u8);
    }

    pub fn sprite_prep_king_zora(&mut self, k: usize) {
        if self.ram.read_u8(0xf356) != 0 {
            self.ram.set_sprite_state(k, 0);
        } else {
            self.sprite_prep_ignore_projectiles(k);
        }
    }

    pub fn sprite_prep_do_nothing_d(&mut self, _k: usize) {}

    pub fn sprite_prep_octorok(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [3, 5];
        const HEALTH: [u8; 2] = [2, 4];
        let j = usize::from(self.ram.read_u8(0xfff) != 0);
        self.ram.set_sprite_health(k, HEALTH[j]);
        self.ram.set_sprite_bump_damage(k, BUMP_DAMAGE[j]);
        let delay = self.runtime.random_number(self.ram) & 127;
        self.ram.set_sprite_main_delay(k, delay);
    }

    pub fn sprite_prep_walking_zora(&mut self, k: usize) {
        self.ram.set_sprite_main_delay(k, 96);
    }

    pub fn sprite_prep_move_down_8px(&mut self, k: usize) {
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_add(8));
    }

    pub fn sprite_prep_zazakku(&mut self, _k: usize) {}

    pub fn sprite_prep_stalfos(&mut self, k: usize) {
        let subtype = self.ram.sprite(k).x_low() & 16;
        self.ram.set_sprite_subtype(k, subtype);
        if subtype != 0 {
            self.ram.set_sprite_oam_flags(k, 7);
        }
    }

    pub fn sprite_prep_move_down_8px_right_8px(&mut self, k: usize) {
        self.ram
            .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(8));
        self.sprite_prep_move_down_8px(k);
    }

    pub fn sprite_prep_mini_helmasaur(&mut self, k: usize) {
        self.ram.set_sprite_scratch_a(k, 16);
        self.ram.set_sprite_ai_state(k, 1);
    }

    pub fn sprite_prep_absorbable(&mut self, k: usize) {
        if self.ram.read_u8(0x1b) == 0 {
            self.ram
                .set_sprite_scratch_e(k, self.ram.sprite(k).scratch_e().wrapping_add(1));
            self.sprite_prep_ignore_projectiles(k);
        }
    }

    pub fn sprite_prep_overworld_bonk_item(&mut self, k: usize) {
        self.ram
            .set_sprite_scratch_e(k, self.ram.sprite(k).scratch_e().wrapping_add(1));
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_shield_pickup(&mut self, _k: usize) {}

    pub fn sprite_prep_fairy(&mut self, k: usize) {
        let a = self.runtime.random_number(self.ram) & 1;
        self.ram.set_sprite_scratch_a(k, a);
        self.ram.set_sprite_scratch_d(k, a ^ 1);
        self.sprite_prep_absorbable(k);
    }

    pub fn sprite_prep_nice_bee(&mut self, k: usize) {
        let bottles = self.ram.read_u8(0xf35c)
            | self.ram.read_u8(0xf35d)
            | self.ram.read_u8(0xf35e)
            | self.ram.read_u8(0xf35f);
        if bottles & 8 != 0 {
            self.ram.set_sprite_state(k, 0);
        }
        self.sprite_prep_overworld_bonk_item(k);
    }

    pub fn sprite_prep_do_nothing_g(&mut self, _k: usize) {}

    pub fn sprite_prep_fire_bar(&mut self, k: usize) {
        self.ram
            .set_sprite_scratch_b(k, self.ram.sprite(k).scratch_b().wrapping_add(1));
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_spike(&mut self, k: usize) {
        self.ram.set_sprite_velocity_x(k, 32);
        self.ram.set_sprite_velocity_y(k, (-16i8) as u8);
        self.runtime.move_y(self.ram, k);
        self.ram.set_sprite_velocity_y(k, 0);
    }

    pub fn sprite_prep_rock_stal(&mut self, k: usize) {
        self.ram.set_sprite_velocity_y(k, (-16i8) as u8);
        self.runtime.move_y(self.ram, k);
        self.ram.set_sprite_velocity_y(k, 0);
    }

    pub fn sprite_prep_blob(&mut self, k: usize) {
        self.ram.set_sprite_graphics(k, 4);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_do_nothing_h(&mut self, _k: usize) {}

    pub fn sprite_prep_big_key(&mut self, k: usize) {
        self.ram
            .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(8));
        self.ram.set_sprite_subtype(k, 0xff);
        self.sprite_prep_big_key_load_graphics(k);
    }

    pub fn sprite_prep_big_key_load_graphics(&mut self, k: usize) {
        self.runtime.decode_animated_sprite_tile(self.ram, 0x22);
        self.sprite_prep_key_set_item_drop(k);
    }

    pub fn sprite_prep_small_key(&mut self, k: usize) {
        self.ram.set_sprite_subtype(k, 0xff);
        let die_action = self.ram.read_u8(0xb9b);
        self.ram.set_sprite_death_action(k, die_action);
        self.ram.write_u8(0xb9b, die_action.wrapping_add(1));
    }

    pub fn sprite_prep_key_set_item_drop(&mut self, k: usize) {
        let die_action = self.ram.read_u8(0xb9b);
        self.ram.set_sprite_death_action(k, die_action);
        self.ram.write_u8(0xb9b, die_action.wrapping_add(1));
    }

    pub fn sprite_return_if_boss_finished(&mut self, k: usize) -> bool {
        if self.ram.read_u16(0x402) & 0x8000 != 0 {
            self.ram.set_sprite_state(k, 0);
            return true;
        }
        for j in (0..16).rev() {
            let kind = self.ram.sprite(j).kind();
            if self.runtime.sprite_init_bump_damage(kind) & 0x10 == 0 {
                self.ram.set_sprite_state(j, 0);
            }
        }
        false
    }

    pub fn sprite_prep_agahnims_barrier(&mut self, k: usize) {
        let screen = usize::from(self.ram.read_u8(0x8a));
        if self.ram.read_u8(0xf280 + screen) & 0x40 != 0 {
            self.ram.set_sprite_graphics(k, 4);
        }
        self.sprite_prep_move_down_8px_right_8px(k);
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_sub(12));
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_catfish(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right_8px(k);
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_sub(12));
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_babasu(&mut self, k: usize) {
        self.sprite_prep_move_down_8px(k);
        self.sprite_prep_zoro(k);
    }

    pub fn sprite_prep_laser_eye_bounce(&mut self, k: usize) {
        let kind = self.ram.sprite(k).kind();
        self.ram.set_sprite_scratch_d(k, kind.wrapping_sub(0x95));
        if kind >= 0x97 {
            self.ram
                .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(8));
            let head = (self.ram.sprite(k).x_low() & 16) ^ 16;
            self.ram.set_sprite_head_direction(k, head);
            if head == 0 {
                let delta = if kind & 1 != 0 { -8i8 } else { 8 };
                self.ram
                    .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_add(delta as u8));
            }
        } else {
            let head = self.ram.sprite(k).y_low() & 16;
            self.ram.set_sprite_head_direction(k, head);
            if head == 0 {
                let delta = if kind & 1 != 0 { -8i8 } else { 8 };
                self.ram
                    .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(delta as u8));
            }
        }
    }

    pub fn sprite_prep_bari(&mut self, k: usize) {
        self.ram.set_sprite_z(k, 6);
        if self.ram.read_u8(0x48e) == 206 {
            self.ram
                .set_sprite_scratch_c(k, self.ram.sprite(k).scratch_c().wrapping_sub(1));
        }
        let delay = (self.runtime.random_number(self.ram) & 63).wrapping_add(128);
        self.ram.set_sprite_aux1_delay(k, delay);
    }

    pub fn sprite_prep_thieves_town_grate(&mut self, k: usize) {
        if self.ram.read_u8(0xf280 + 0x58) & 0x20 != 0 {
            self.ram.set_sprite_state(k, 0);
        }
        self.sprite_prep_rupee_pull(k);
    }

    pub fn sprite_prep_storyteller(&mut self, k: usize) {
        const ROOMS: [u8; 5] = [0x0e, 0x0e, 0x12, 0x1a, 0x14];
        let room = self.ram.read_u8(0x48e);
        let mut r = ROOMS.iter().position(|&v| v == room).unwrap_or(0xff) as u8;
        if r == 0 && self.ram.sprite(k).x_high() & 1 != 0 {
            r = 1;
        }
        self.ram.set_sprite_subtype2(k, r);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_adults(&mut self, k: usize) {
        const ROOMS: [u8; 3] = [3, 0xe1, 0x19];
        let room = self.ram.read_u8(0x48e);
        let subtype = ROOMS.iter().position(|&v| v == room).unwrap_or(0xff) as u8;
        self.ram.set_sprite_subtype2(k, subtype);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_hardhat_beetle(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [6, 8];
        const HEALTH: [u8; 2] = [32, 6];
        const A: [u8; 2] = [16, 12];
        const STATE: [u8; 2] = [1, 3];
        const FLAGS5: [u8; 2] = [2, 6];
        const BUMP_DAMAGE: [u8; 2] = [5, 3];
        let j = usize::from(self.ram.sprite(k).x_low() & 0x10 != 0);
        self.ram.set_sprite_oam_flags(k, OAM_FLAGS[j]);
        self.ram.set_sprite_health(k, HEALTH[j]);
        self.ram.set_sprite_scratch_a(k, A[j]);
        self.ram.set_sprite_ai_state(k, STATE[j]);
        self.ram.set_sprite_flags5(k, FLAGS5[j]);
        self.ram.set_sprite_bump_damage(k, BUMP_DAMAGE[j]);
    }

    pub fn sprite_prep_bonk_item(&mut self, k: usize) {
        const DASH_ITEM_MASK: [u16; 2] = [0x4000, 0x2000];
        if self.ram.read_u8(0x1b) == 0 {
            self.ram.set_sprite_graphics(k, 2);
            return;
        }
        self.ram.set_sprite_floor(k, 2);
        if self.ram.read_u16(0x48e) == 0x107 {
            if self.ram.read_u8(0xf34e) != 0 {
                self.ram.set_sprite_state(k, 0);
            } else {
                self.runtime.decode_animated_sprite_tile(self.ram, 0x0e);
            }
        } else {
            let j = self.ram.read_u8(0xb9b);
            self.ram.write_u8(0xb9b, j.wrapping_add(1));
            self.ram.set_sprite_death_action(k, j);
            if self.ram.read_u16(0x402) & DASH_ITEM_MASK[usize::from(j)] != 0 {
                self.ram.set_sprite_state(k, 0);
            }
            self.ram
                .set_sprite_graphics(k, self.ram.sprite(k).graphics().wrapping_add(1));
            self.ram.set_sprite_oam_flags(k, 8);
            self.ram
                .set_sprite_flags3(k, self.ram.sprite(k).flags3() | 0x20);
        }
    }

    pub fn sprite_prep_locksmith(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        match self.ram.read_u8(0xf3cc) {
            9 => {
                self.ram.set_sprite_state(k, 0);
                return;
            }
            12 => self.ram.set_sprite_ai_state(k, 2),
            _ => {}
        }
        if self.ram.read_u8(0xf3c9) & 0x10 != 0 {
            self.ram.set_sprite_ai_state(k, 4);
        }
    }

    pub fn sprite_prep_tektite(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [9, 7];
        const HEALTH: [u8; 2] = [8, 12];
        const BUMP_DAMAGE: [u8; 2] = [3, 5];
        let j = usize::from((self.ram.sprite(k).x_low() >> 4) & 1);
        self.ram.set_sprite_scratch_a(k, j as u8);
        self.ram.set_sprite_oam_flags(k, OAM_FLAGS[j]);
        self.ram.set_sprite_health(k, HEALTH[j]);
        self.ram.set_sprite_bump_damage(k, BUMP_DAMAGE[j]);
        self.runtime.apply_speed_towards_link(self.ram, k, 16);
        self.ram.set_sprite_velocity_z(k, 32);
        self.ram
            .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
    }

    pub fn sprite_prep_big_fairy(&mut self, k: usize) {
        self.ram.set_sprite_z(k, 24);
        self.sprite_prep_move_down_8px_right_8px(k);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_mrs_sahasrahla(&mut self, k: usize) {
        self.sprite_prep_move_down_8px(k);
        self.sprite_prep_magic_bat(k);
    }

    pub fn sprite_prep_fortune_teller(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right_8px(k);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_fairy_pond(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [10, 2];
        let j = usize::from((self.ram.sprite(k).x_low() >> 4) & 1);
        self.ram.set_sprite_scratch_a(k, j as u8);
        self.ram.set_sprite_oam_flags(k, OAM_FLAGS[j]);
    }

    pub fn sprite_prep_kodongo(&mut self, k: usize) {
        self.ram
            .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(4));
        let y = u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()])
            .wrapping_sub(5);
        let [lo, hi] = y.to_le_bytes();
        self.ram.set_sprite_y_low(k, lo);
        self.ram.set_sprite_y_high(k, hi);
        self.sprite_prep_spark(k);
    }

    pub fn sprite_prep_lost_woods_bird(&mut self, k: usize) {
        let velocity = (self.runtime.random_number(self.ram) & 0x1f).wrapping_sub(0x10);
        self.ram.set_sprite_velocity_z(k, velocity);
        self.ram.set_sprite_z(k, 64);
        self.sprite_prep_lost_woods_squirrel(k);
    }

    pub fn sprite_prep_lost_woods_squirrel(&mut self, k: usize) {
        let x_velocity = if self.runtime.is_right_of_link(self.ram, k) {
            -16i8
        } else {
            16
        };
        self.ram.set_sprite_velocity_x(k, x_velocity as u8);
        let y_velocity = if (self.ram.read_u8(0x69e) as i8) < 0 {
            4i8
        } else {
            -4
        };
        self.ram.set_sprite_velocity_y(k, y_velocity as u8);
        self.ram.set_sprite_ignore_projectile(k, y_velocity as u8);
    }

    pub fn sprite_prep_falling_ice(&mut self, k: usize) {
        if !self.sprite_return_if_boss_finished(k) {
            self.sprite_prep_ignore_projectiles(k);
        }
    }

    pub fn sprite_prep_armos_knight(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram.set_sprite_main_delay(k, 255);
        self.ram
            .write_u8(0xff8, self.ram.read_u8(0xff8).wrapping_add(1));
        self.sprite_prep_move_down_8px_right_8px(k);
    }

    pub fn sprite_prep_desert_statue(&mut self, k: usize) {
        let instance = self.ram.read_u8(0xb6a);
        self.ram.set_sprite_scratch_a(k, instance);
        self.ram.write_u8(0xb6a, instance.wrapping_add(1));
        self.sprite_prep_move_down_8px_right_8px(k);
        let x = self.ram.sprite(k).x_low();
        self.ram.set_sprite_scratch_d(
            k,
            if x < 0x30 {
                1
            } else if x < 0xe0 {
                3
            } else {
                2
            },
        );
    }

    pub fn sprite_prep_kyameron(&mut self, k: usize) {
        self.ram.set_sprite_scratch_a(k, self.ram.sprite(k).x_low());
        self.ram
            .set_sprite_scratch_b(k, self.ram.sprite(k).x_high());
        self.ram.set_sprite_scratch_c(k, self.ram.sprite(k).y_low());
        self.ram
            .set_sprite_head_direction(k, self.ram.sprite(k).y_high());
    }

    pub fn sprite_prep_geldman(&mut self, k: usize) {
        self.ram
            .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(8));
        self.sprite_prep_kyameron(k);
    }

    pub fn sprite_prep_swimming_zora(&mut self, k: usize) {
        self.ram.set_sprite_main_delay(k, 64);
        self.sprite_prep_geldman(k);
    }

    pub fn sprite_prep_big_spike(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right_8px(k);
        self.sprite_prep_kyameron(k);
    }

    pub fn sprite_prep_pedestal_plaque(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        if self.ram.read_u8(0x8a) == 48 {
            self.ram
                .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(7));
        }
    }

    pub fn sprite_prep_kholdstare_shell(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram.set_sprite_aux1_delay(k, 192);
        self.sprite_prep_move_down_8px_right_8px(k);
    }

    pub fn sprite_prep_kholdstare(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram.set_sprite_ai_state(k, 3);
        self.sprite_prep_ignore_projectiles(k);
        self.sprite_prep_move_down_8px_right_8px(k);
    }

    pub fn sprite_prep_bumper(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        self.sprite_prep_move_down_8px_right_8px(k);
    }

    pub fn sprite_prep_moldorm(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_prep_ignore_projectiles(k);
        self.runtime.initialized_segmented(self.ram, k);
    }

    pub fn sprite_prep_lanmolas(&mut self, k: usize) {
        const INITIAL_DELAY: [u8; 3] = [128, 192, 255];
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram.set_sprite_main_delay(k, INITIAL_DELAY[k]);
        self.ram.set_sprite_z(k, 0xff);
        for i in 0..64 {
            self.ram.write_u8(0x1fe00 + k * 0x40 + i, 0xff);
        }
        self.ram.set_garnish_y_low(k, 7);
    }

    pub fn sprite_prep_crystal_switch(&mut self, k: usize) {
        const PALETTE: [u8; 2] = [2, 4];
        let j = usize::from(self.ram.read_u16(0xc172) & 1);
        self.ram
            .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() | PALETTE[j]);
    }

    pub fn sprite_prep_flute_kid(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        let subtype = (self.ram.read_u8(0xf3ca) >> 6) & 1;
        self.ram.set_sprite_subtype2(k, subtype);
        let flute = self.ram.read_u8(0xf34c);
        if subtype != 0 {
            if self.ram.read_u8(0xf3c9) & 8 != 0 || flute > 2 {
                self.ram.set_sprite_graphics(k, 3);
                self.ram.set_sprite_ai_state(k, 5);
            } else if flute == 2 {
                self.ram.set_sprite_graphics(k, 1);
            }
            self.ram
                .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(8));
            self.ram
                .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_sub(8));
        } else if flute >= 2 {
            self.ram.set_sprite_state(k, 0);
        } else {
            self.ram
                .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(7));
        }
    }

    pub fn sprite_prep_arrghus(&mut self, k: usize) {
        if !self.sprite_return_if_boss_finished(k) {
            self.ram.set_sprite_z(k, 24);
        }
    }

    pub fn sprite_prep_mothula(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram.set_sprite_main_delay(k, 80);
        self.sprite_prep_ignore_projectiles(k);
        self.ram.set_sprite_graphics(k, 2);
        self.ram
            .write_u8(0x41a, self.ram.read_u8(0x41a).wrapping_add(1));
        self.ram.set_sprite_scratch_c(k, 112);
    }

    pub fn sprite_prep_agahnim(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [11, 7];
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram.set_sprite_graphics(k, 0);
        self.ram.set_sprite_scratch_d(k, 3);
        self.sprite_prep_move_down_8px_right_8px(k);
        let j = usize::from(self.ram.read_u8(0xfff) != 0);
        self.ram.set_sprite_oam_flags(k, OAM_FLAGS[j]);
    }

    pub fn sprite_prep_swamola(&mut self, k: usize) {
        self.runtime.swamola_initialize_segments(self.ram, k);
        self.sprite_prep_kyameron(k);
    }

    pub fn sprite_prep_blind(&mut self, k: usize) {
        if !self.sprite_return_if_boss_finished(k) {
            self.runtime.blind_prepare_battle(self.ram, k);
        }
    }

    pub fn sprite_prep_ganon(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.runtime.ganon_animation_idle(self.ram, k);
        self.ram.set_sprite_main_delay(k, 128);
        self.ram.set_sprite_room(k, 2);
        self.ram.write_u8(0x12c, 0x1e);
    }

    pub fn sprite_prep_mini_vitreous(&mut self, k: usize) {
        self.sprite_return_if_boss_finished(k);
    }

    pub fn sprite_prep_cutscene_agahnim(&mut self, k: usize) {
        if self.ram.read_u16(0x402) & 0x4000 != 0 {
            self.ram.set_sprite_state(k, 0);
        } else {
            self.runtime.cutscene_agahnim_spawn_zelda(self.ram, k);
            self.sprite_prep_ignore_projectiles(k);
        }
    }

    pub fn sprite_prep_vitreous(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_prep_move_down_8px_right_8px(k);
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_sub(16));
        self.runtime.vitreous_spawn_smaller_eyes(self.ram, k);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_mini_moldorm_bounce(&mut self, k: usize) {
        let xlo = self.ram.sprite(k).x_low();
        let xhi = self.ram.sprite(k).x_high();
        let ylo = self.ram.sprite(k).y_low();
        let yhi = self.ram.sprite(k).y_high();
        for i in k * 32..k * 32 + 32 {
            self.ram.write_u8(0x1fc00 + i, xlo);
            self.ram.write_u8(0x1fc80 + i, xhi);
            self.ram.write_u8(0x1fd00 + i, ylo);
            self.ram.write_u8(0x1fd80 + i, yhi);
        }
    }

    pub fn sprite_prep_chainchomp_bounce(&mut self, k: usize) {
        let x = self.ram.read_u16(0xfd8);
        let y = self.ram.read_u16(0xfda);
        for i in k * 8..k * 8 + 6 {
            self.ram.write_u16(0x1fc00 + i * 2, x);
            self.ram.write_u16(0x1fd00 + i * 2, y);
        }
        self.ram.set_sprite_scratch_a(k, self.ram.sprite(k).x_low());
        self.ram
            .set_sprite_scratch_b(k, self.ram.sprite(k).x_high());
        self.ram.set_sprite_scratch_c(k, self.ram.sprite(k).y_low());
        self.ram
            .set_sprite_scratch_g(k, self.ram.sprite(k).y_high());
    }

    pub fn sprite_prep_bully_and_victim(&mut self, k: usize) {
        self.runtime.spawn_bully(self.ram, k);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub fn sprite_prep_purple_chest(&mut self, k: usize) {
        let follower = self.ram.read_u8(0xf3cc);
        let progress = self.ram.read_u8(0xf3c9);
        if follower != 12 && progress & 0x10 == 0 && progress & 0x20 != 0 {
            self.sprite_prep_ignore_projectiles(k);
        } else {
            self.ram.set_sprite_state(k, 0);
        }
    }

    pub fn sprite_prep_trinexx(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.runtime.trinexx_components_initialize(self.ram, k);
        for i in 0..16 {
            self.ram.clear_cached_sprite(i);
        }
    }

    pub fn sprite_prep_helmasaur_king(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.runtime.helmasaur_king_initialize(self.ram, k);
        for i in 0..16 {
            self.ram.clear_cached_sprite(i);
        }
    }

    pub fn sprite_prep_fire_debirando(&mut self, k: usize) {
        self.ram.set_sprite_kind(k, 0x63);
        self.runtime.load_properties(self.ram, k);
        self.ram
            .set_sprite_scratch_g(k, self.ram.sprite(k).scratch_g().wrapping_sub(1));
        self.sprite_prep_debirando_pit(k);
    }

    pub fn sprite_prep_debirando_pit(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [6, 8];
        let g = self.ram.sprite(k).scratch_g().wrapping_add(1);
        self.ram.set_sprite_scratch_g(k, g);
        self.ram.set_sprite_main_delay(k, 0);
        self.ram.set_sprite_graphics(k, 6);
        self.sprite_prep_ignore_projectiles(k);
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x64) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.set_sprite_main_delay(j, 96);
            self.ram.set_sprite_head_direction(k, j as u8);
            self.ram.set_sprite_scratch_g(j, g);
            self.ram.set_sprite_oam_flags(j, OAM_FLAGS[usize::from(g)]);
        }
    }

    pub fn sprite_prep_arrow_game_bounce(&mut self, k: usize) {
        const X: [u8; 8] = [0, 0x40, 0x80, 0xc0, 0x30, 0x60, 0x90, 0xc0];
        const Y: [u8; 8] = [0, 0x4f, 0x4f, 0x4f, 0x5a, 0x5a, 0x5a, 0x5a];
        const A: [u8; 8] = [0, 1, 1, 1, 2, 2, 2, 2];
        const XV: [i8; 2] = [-8, 12];
        const FLAGS4: [u8; 2] = [0x1c, 0x15];
        self.ram.write_u8(0xb88, 0);
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_sub(9));
        for i in (1..8).rev() {
            self.ram.set_sprite_kind(i, 0x65);
            self.ram.set_sprite_state(i, 9);
            self.runtime.load_properties(self.ram, i);
            self.ram.set_sprite_x_high(i, self.ram.read_u8(0x23));
            self.ram.set_sprite_x_low(i, X[i]);
            self.ram.set_sprite_y_high(i, self.ram.read_u8(0x21));
            self.ram.set_sprite_y_low(i, Y[i]);
            self.ram.set_sprite_scratch_a(i, A[i]);
            let j = usize::from(A[i] - 1);
            self.ram.set_sprite_graphics(i, j as u8);
            self.ram.set_sprite_velocity_x(i, XV[j] as u8);
            self.ram.set_sprite_flags4(i, FLAGS4[j]);
            self.ram.set_sprite_oam_flags(i, 13);
            self.ram.set_sprite_floor(i, self.ram.read_u8(0xee));
            let subtype = self.runtime.random_number(self.ram);
            self.ram.set_sprite_subtype2(i, subtype);
        }
        self.sprite_prep_ignore_projectiles(k);
        self.ram.set_sprite_subtype(k, self.ram.read_u8(0xf377));
    }

    pub fn sprite_prep_haunted_grove_animal(&mut self, k: usize) {
        let direction = u8::from(self.runtime.is_right_of_link(self.ram, k));
        self.ram.set_sprite_scratch_d(k, direction);
        self.sprite_prep_haunted_grove_ostrich(k);
    }

    pub fn sprite_prep_snitch_bounce_1(&mut self, k: usize) {
        self.runtime.prep_snitches(self.ram, k);
    }

    pub fn sprite_prep_snitch_bounce_2(&mut self, k: usize) {
        self.runtime.prep_snitches(self.ram, k);
    }

    pub fn sprite_prep_snitch_bounce_3(&mut self, k: usize) {
        self.runtime.prep_snitches(self.ram, k);
    }

    pub fn sprite_prep_hobo_spawn_smoke(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x2b) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.set_sprite_subtype2(j, 0);
            self.ram.set_sprite_ignore_projectile(j, 0);
        }
    }

    pub fn sprite_prep_hobo_spawn_fire(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x2b) {
            self.set_sprite_x(j, 0x194);
            self.set_sprite_y(j, 0x03f);
            self.ram.set_sprite_subtype2(j, 2);
            self.ram.set_sprite_ignore_projectile(j, 2);
            self.ram.set_sprite_flags2(j, 0);
            self.ram
                .set_sprite_oam_flags(j, (self.ram.sprite(j).oam_flags() & !0x0e) | 2);
        }
    }

    pub fn sprite_prep_hobo(&mut self, k: usize) {
        for _ in 1..16 {
            self.sprite_prep_hobo_spawn_smoke(k);
        }
        for i in (1..16).rev() {
            if self.ram.sprite(i).kind() == 0x2b {
                self.ram.set_sprite_state(i, 0);
            }
        }
        self.sprite_prep_hobo_spawn_fire(k);
        if self.ram.read_u8(0xf3c9) & 1 != 0 {
            self.ram.set_sprite_ai_state(0, 3);
        }
        self.ram.set_sprite_ignore_projectile(0, 1);
    }

    pub fn sprite_prep_heart_piece(&mut self, k: usize) {
        self.runtime.heart_upgrade_check_obtained(self.ram, k);
    }

    pub fn sprite_prep_blind_maiden(&mut self, k: usize) {
        if self.ram.read_u16(0xf000 + 0xac * 2) & 0x800 == 0 {
            self.sprite_prep_ignore_projectiles(k);
            if self.ram.read_u8(0xf3cc) != 6 {
                self.ram.write_u8(0xf3cc, 6);
                self.ram.write_u8(0xf3d3, 0);
                self.ram.write_u8(0x2f9, 0);
                self.runtime.load_follower_graphics(self.ram);
                self.runtime.follower_initialize(self.ram);
                self.ram.write_u8(0xf3cc, 0);
                return;
            }
        }
        self.ram.set_sprite_state(k, 0);
    }

    pub fn sprite_prep_bomb_shoppe(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        let source_x =
            u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()]);
        let source_y =
            u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()]);
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xb5) {
            self.set_sprite_x(j, source_x.wrapping_sub(24));
            self.set_sprite_y(j, source_y.wrapping_sub(24));
            self.ram.set_sprite_subtype2(j, 1);
            self.ram.set_sprite_ignore_projectile(j, 1);
        }
        if self.ram.read_u8(0xf37a) & 5 == 5
            && self.ram.read_u8(0xf3c9) & 32 != 0
            && let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xb5)
        {
            self.set_sprite_x(j, source_x.wrapping_sub(56));
            self.set_sprite_y(j, source_y.wrapping_sub(24));
            self.ram.set_sprite_subtype2(j, 2);
            self.ram.set_sprite_ignore_projectile(j, 2);
        }
    }

    pub fn sprite_prep_arrghi(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        let subtype = self.runtime.random_number(self.ram);
        self.ram.set_sprite_subtype2(k, subtype);
        if k == 13 {
            self.ram.write_u8(0xb08 + 2, 0);
            self.ram.write_u8(0xb08 + 3, 0);
            self.runtime.arrghus_handle_puffs(self.ram, 0);
        }
        self.ram
            .set_sprite_x_low(k, self.ram.read_u8(0xb08 + k + 7));
        self.ram
            .set_sprite_x_high(k, self.ram.read_u8(0xb18 + k + 7));
        self.ram
            .set_sprite_y_low(k, self.ram.read_u8(0xb28 + k + 7));
        self.ram
            .set_sprite_y_high(k, self.ram.read_u8(0xb38 + k + 7));
    }

    pub fn sprite_prep_talking_tree(&mut self, k: usize) {
        self.sprite_prep_rupee_pull(k);
        self.runtime.talking_tree_spawn_eyeball(self.ram, k, 0);
        self.runtime.talking_tree_spawn_eyeball(self.ram, k, 1);
    }

    pub fn sprite_prep_old_man_bounce(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        if self.ram.read_u8(0x48e) == 0xe4 {
            self.ram.set_sprite_subtype2(k, 2);
            return;
        }
        if self.ram.read_u8(0xf3cc) == 0 {
            if self.ram.read_u8(0xf353) == 2 {
                self.ram.set_sprite_state(k, 0);
            }
            self.ram.write_u8(0xf3cc, 4);
            self.runtime.load_follower_graphics(self.ram);
            self.ram.write_u8(0xf3cc, 0);
        } else {
            self.ram.set_sprite_state(k, 0);
            self.runtime.load_follower_graphics(self.ram);
        }
    }

    pub fn sprite_prep_shopkeeper(&mut self, k: usize) {
        const ROOMS: [u8; 13] = [
            0x0f, 0x10, 0, 6, 0x18, 0x12, 0x1e, 0xff, 0x1f, 0x23, 0x24, 0x25, 0x27,
        ];
        self.sprite_prep_ignore_projectiles(k);
        self.ram
            .set_sprite_flags2(k, self.ram.sprite(k).flags2() | 2);
        self.ram
            .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() | 12);
        self.ram
            .set_sprite_flags3(k, self.ram.sprite(k).flags3() | 16);
        let room = self.ram.read_u8(0x48e);
        let j = ROOMS.iter().position(|&v| v == room).unwrap_or(usize::MAX);
        match j {
            0 => {
                self.runtime.shopkeeper_spawn_item(self.ram, k, 0, 7);
                self.runtime.shopkeeper_spawn_item(self.ram, k, 1, 8);
                self.runtime.shopkeeper_spawn_item(self.ram, k, 2, 12);
            }
            1 => {
                self.runtime.shopkeeper_spawn_item(self.ram, k, 0, 9);
                self.runtime.shopkeeper_spawn_item(self.ram, k, 1, 13);
                self.runtime.shopkeeper_spawn_item(self.ram, k, 2, 11);
            }
            2 => {
                self.ram.set_sprite_subtype2(k, 4);
                self.ram.write_u8(0x4c4, 0xff);
            }
            3 => {
                self.ram.set_sprite_subtype2(k, 1);
                self.ram.set_sprite_graphics(k, 1);
                self.ram.write_u8(0x4c4, 0xff);
            }
            4 => {
                self.ram.set_sprite_subtype2(k, 3);
                self.ram.write_u8(0x4c4, 0xff);
            }
            5 | 7 | 8 => {
                self.runtime.shopkeeper_spawn_item(self.ram, k, 0, 7);
                self.runtime.shopkeeper_spawn_item(self.ram, k, 1, 10);
                self.runtime.shopkeeper_spawn_item(self.ram, k, 2, 12);
            }
            6 | 9 | 12 => self.ram.set_sprite_subtype2(k, 2),
            10 => self.ram.set_sprite_subtype2(k, 5),
            11 => self.ram.set_sprite_subtype2(k, 6),
            _ => panic!("unknown shopkeeper room {room:#x}"),
        }
    }

    pub fn sprite_prep_zelda_bounce(&mut self, k: usize) {
        if self.ram.read_u8(0xf359) >= 2 {
            self.ram.set_sprite_state(k, 0);
            return;
        }
        self.sprite_prep_ignore_projectiles(k);
        let direction = self.runtime.direction_to_face_link(self.ram, k) ^ 3;
        self.ram.set_sprite_scratch_d(k, direction);
        self.ram.set_sprite_head_direction(k, direction);
        let follower = self.ram.read_u8(0xf3cc);
        self.ram.write_u8(0xf3cc, 1);
        self.runtime.load_follower_graphics(self.ram);
        self.ram.write_u8(0xf3cc, follower);
        if self.ram.read_u8(0x48e) == 0x12 {
            self.ram.set_sprite_subtype2(k, 2);
            if self.ram.read_u8(0xf3c6) & 4 == 0 {
                self.ram.set_sprite_state(k, 0);
            } else {
                let x =
                    u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()])
                        .wrapping_add(6);
                let y =
                    u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()])
                        .wrapping_add(15);
                self.set_sprite_x(k, x);
                self.set_sprite_y(k, y);
                self.ram.set_sprite_flags4(k, 3);
            }
        } else {
            self.ram.set_sprite_subtype2(k, 0);
            if follower == 1 || self.ram.read_u8(0xf3c6) & 4 != 0 {
                self.ram.set_sprite_state(k, 0);
            }
        }
    }

    pub fn smithy_spawn_dwarf_pal(&mut self, k: usize) -> Option<usize> {
        let j = self.runtime.spawn_dynamic(self.ram, k, 0x1a)?;
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.ram
            .set_sprite_x_low(j, self.ram.sprite(j).x_low().wrapping_add(0x2c));
        self.ram.set_sprite_scratch_d(j, 1);
        self.ram.set_sprite_scratch_a(j, 4);
        self.ram.set_sprite_ignore_projectile(j, 4);
        Some(j)
    }

    pub fn smithy_spawn_dumb_barrier_sprite(&mut self, k: usize) {
        let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x31) else {
            return;
        };
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.ram.set_sprite_subtype2(j, 1);
        self.ram.set_sprite_flags4(j, 0);
        self.ram.set_sprite_ignore_projectile(j, 1);
    }

    pub fn sprite_prep_smithy(&mut self, k: usize) {
        self.sprite_prep_ignore_projectiles(k);
        let progress = self.ram.read_u8(0xf3c9);
        if self.ram.read_u8(0xf3ca) & 64 != 0 {
            if progress & 32 != 0 || self.ram.read_u8(0xf3cc) != 0 {
                self.ram.set_sprite_state(k, 0);
            } else {
                self.ram.set_sprite_subtype2(k, 2);
            }
            return;
        }
        self.smithy_spawn_dumb_barrier_sprite(k);
        self.ram
            .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(2));
        self.ram
            .set_sprite_y_low(k, self.ram.sprite(k).y_low().wrapping_sub(3));
        if progress & 32 == 0 {
            return;
        }
        if let Some(j) = self.smithy_spawn_dwarf_pal(k) {
            self.smithy_spawn_dumb_barrier_sprite(j);
            self.ram.set_sprite_scratch_e(j, k as u8);
            self.ram.set_sprite_scratch_e(k, j as u8);
            if progress & 0x80 != 0 {
                self.ram.set_sprite_ai_state(k, 5);
                self.ram.set_sprite_ai_state(j, 5);
            }
        }
    }

    pub fn sprite_active_main(&mut self, k: usize) {
        let kind = self.ram.sprite(k).kind();
        if !self.dispatch_active_local(k, kind) {
            self.runtime.active_cross_bank(self.ram, k, kind);
        }
    }

    /// Dispatches active-table entries owned by this source bank, allowing the
    /// whole-game router to continue into subsequent banks without a callback.
    pub fn dispatch_active_local(&mut self, k: usize, kind: u8) -> bool {
        match kind {
            0x01 => self.sprite_01_vulture_bounce(k),
            0x02 => self.sprite_02_stalfos_head(k),
            0x08 | 0x0a => self.sprite_08_octorok(k),
            0x09 => self.sprite_09_giant_moldorm(k),
            0x0b => self.sprite_0b_cucco(k),
            0x0c => self.sprite_0c_octorok_stone(k),
            0x0d => self.sprite_0d_buzzblob(k),
            0x0e => self.sprite_0e_snapdragon(k),
            0x0f => self.sprite_0f_octoballoon(k),
            0x10 => self.sprite_10_octoballoon_baby(k),
            0x11 => self.sprite_11_hinox(k),
            0x12 => self.sprite_12_moblin(k),
            0x13 => self.sprite_13_mini_helmasaur(k),
            0x14 => self.sprite_14_thieves_town_grate(k),
            0x15 => self.sprite_15_antifairy(k),
            0x16 => self.sprite_16_elder_bounce(k),
            0x17 => self.sprite_17_hoarder(k),
            0x18 => self.sprite_18_mini_moldorm(k),
            0x19 => self.sprite_19_poe(k),
            0x1a => self.sprite_1a_smithy(k),
            0x1b => self.sprite_1b_arrow(k),
            0x1c => self.sprite_1c_statue(k),
            0x1d => self.sprite_1d_flute_quest(k),
            0x1e => self.sprite_1e_crystal_switch(k),
            0x1f => self.sprite_1f_sick_kid(k),
            0x20 => self.sprite_20_sluggula(k),
            0x21 => self.sprite_21_water_switch(k),
            0x22 => self.sprite_22_ropa(k),
            0x23 | 0x24 => self.sprite_23_red_bari(k),
            0x25 => self.sprite_25_talking_tree(k),
            0x26 => self.sprite_26_hardhat_beetle(k),
            0x27 => self.sprite_27_deadrock(k),
            0x28 => self.sprite_28_dark_world_hint_npc(k),
            0x29 => self.sprite_human_multi_1(k),
            0x2a => self.sprite_sweeping_lady(k),
            0x2b => self.sprite_2b_hobo(k),
            0x2c => self.sprite_lumberjacks(k),
            0x2d => self.sprite_2d_telepathic_tile(k),
            0x2e => self.sprite_2e_flute_kid(k),
            0x2f => self.sprite_maze_game_lady(k),
            0x30 => self.sprite_maze_game_guy(k),
            0x31 => self.sprite_fortune_teller(k),
            0x33 => self.sprite_33_rupee_pull(k),
            0x37 => self.sprite_37_waterfall(k),
            0x38 => self.sprite_38_eye_statue(k),
            0x39 => self.sprite_39_locksmith(k),
            0x3a => self.sprite_3a_magic_bat(k),
            0x3c => self.sprite_trough_boy(k),
            0x3e => self.sprite_17_hoarder(k),
            0x3f | 0x40 => self.sprite_tutorial_guard_or_barrier(k),
            0x71 => self.sprite_71_leever(k),
            0x72 => self.sprite_72_fairy_pond(k),
            0x73 => self.sprite_73_uncle_and_priest(k),
            0x78 => self.sprite_78_mrs_sahasrahla(k),
            0xd3 => self.sprite_d3_stal(k),
            0xd4 => self.sprite_d4_landmine(k),
            0xd8 => self.sprite_d8_heart(k),
            0xd2 => self.sprite_d2_flopping_fish(k),
            0xd9..=0xe2 => self.sprite_d9_green_rupee(k),
            0xe3 => self.sprite_e3_fairy(k),
            0xe4 | 0xe5 => self.sprite_e4_small_key(k),
            0xf2 => self.sprite_f2_medallion_tablet(k),
            0xec => self.sprite_ec_thrown_item(k),
            _ => return false,
        }
        true
    }

    pub fn sprite_1c_statue(&mut self, k: usize) {
        const DIR: [u8; 4] = [4, 6, 0, 2];
        const JOY: [u8; 4] = [1, 2, 4, 8];
        const XV: [i8; 4] = [-16, 16, 0, 0];
        const YV: [i8; 4] = [0, 0, -16, 16];
        if self.ram.sprite(k).scratch_d() != 0 {
            self.ram.set_sprite_scratch_d(k, 0);
            self.ram.write_u8(0x5e, 0);
            self.ram.write_u8(0x48, 0);
        }
        if self.ram.sprite(k).main_delay() != 0 {
            self.ram.set_sprite_scratch_d(k, 1);
            self.ram.write_u8(0x48, 129);
            self.ram.write_u8(0x5e, 8);
        }
        self.movable_statue_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.statue_block_sprites(k);
        let on_switch = self.statue_check_for_switch(k);
        self.ram.write_u8(0x642, u8::from(on_switch));
        self.runtime.move_xy(self.ram, k);
        self.runtime.get_16bit_coords(self.ram, k);
        self.runtime.check_tile_collision2(self.ram, k);
        self.ram.set_sprite_velocity_x(k, 0);
        self.ram.set_sprite_velocity_y(k, 0);
        let mut j;
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            self.ram.set_sprite_main_delay(k, 7);
            self.runtime.repel_dash(self.ram);
            if self.ram.sprite(k).aux1_delay() != 0 {
                self.runtime.nullify_hookshot_drag(self.ram);
                return;
            }
            j = usize::from(self.runtime.direction_to_face_link(self.ram, k));
            self.ram.set_sprite_velocity_x(k, XV[j] as u8);
            self.ram.set_sprite_velocity_y(k, YV[j] as u8);
        } else {
            let x = self.sprite_x(k);
            let y = self.sprite_y(k);
            if self.ram.sprite(k).main_delay() == 0 {
                self.ram.set_sprite_aux1_delay(k, 13);
            }
            let near = x.wrapping_sub(self.ram.read_u16(0x22)).wrapping_add(16) < 35
                && y.wrapping_sub(self.ram.read_u16(0x20)).wrapping_add(12) < 36;
            // The C expression short-circuits before direction_to_face_link,
            // which also writes tmp_counter in WRAM.
            j = if near {
                usize::from(self.runtime.direction_to_face_link(self.ram, k))
            } else {
                0
            };
            if near && self.ram.read_u8(0x2f) == DIR[j] && self.ram.read_u8(0x372) == 0 {
                self.ram.write_u8(0x2fa, 1);
                self.ram.set_sprite_scratch_a(k, 1);
                if self.ram.read_u8(0x376) & 2 == 0
                    || JOY[j] & self.ram.read_u8(0xf0) == 0
                    || self.ram.read_u8(0x30) | self.ram.read_u8(0x31) == 0
                {
                    return;
                }
                j ^= 1;
                self.ram.set_sprite_velocity_x(k, XV[j] as u8);
                self.ram.set_sprite_velocity_y(k, YV[j] as u8);
            } else {
                if self.ram.sprite(k).scratch_a() != 0 {
                    self.ram.set_sprite_scratch_a(k, 0);
                    self.ram.write_u8(0x5e, 0);
                    self.ram.write_u8(0x376, 0);
                    self.ram.write_u8(0x2fa, 0);
                    self.ram.write_u8(0x50, self.ram.read_u8(0x50) & !1);
                }
                return;
            }
        }
        if self.ram.read_u8(0x376) & 2 == 0 {
            self.runtime.nullify_hookshot_drag(self.ram);
        }
        if self.ram.sprite(k).wall_collision() & 15 == 0 && self.ram.sprite(k).aux4_delay() == 0 {
            self.runtime.sfx2_with_pan(self.ram, k, 0x22);
            self.ram.set_sprite_aux4_delay(k, 8);
        }
    }

    pub fn statue_check_for_switch(&mut self, k: usize) -> bool {
        const X: [i8; 4] = [3, 12, 3, 12];
        const Y: [i8; 4] = [3, 3, 12, 12];
        for i in (0..4).rev() {
            let x = self.sprite_x(k).wrapping_add(X[i] as u16);
            let y = self.sprite_y(k).wrapping_add(Y[i] as u16);
            let t = self
                .runtime
                .get_tile_attribute(self.ram, self.ram.sprite(k).floor(), x, y);
            if !matches!(t, 0x23 | 0x24 | 0x25 | 0x3b) {
                return false;
            }
        }
        true
    }

    pub fn statue_block_sprites(&mut self, k: usize) {
        let cx = self.sprite_x(k);
        let cy = self.sprite_y(k);
        for j in (0..16).rev() {
            if self.ram.sprite(j).kind() == 0x1c
                || j == k
                || ((j as u8) ^ self.ram.read_u8(0x1a)) & 1 != 0
                || self.ram.sprite(j).state() < 9
            {
                continue;
            }
            let x = self.sprite_x(j);
            let y = self.sprite_y(j);
            if cx.wrapping_sub(x).wrapping_add(12) < 24 && cy.wrapping_sub(y).wrapping_add(12) < 36
            {
                self.ram.set_sprite_scratch_f(j, 4);
                let (vx, vy) = self
                    .runtime
                    .project_speed_towards_location(self.ram, k, x, y, 32);
                self.ram.set_sprite_recoil_y(j, vy as u8);
                self.ram.set_sprite_recoil_x(j, vx as u8);
            }
        }
    }

    pub fn sprite_72_fairy_pond(&mut self, k: usize) {
        const X: [u8; 8] = [0, 4, 8, 12, 16, 20, 24, 0];
        const Y: [u8; 8] = [0, 8, 16, 24, 32, 40, 4, 36];
        if self.ram.sprite(k).scratch_a() != 0 {
            let c = self.ram.sprite(k).scratch_c().wrapping_sub(1);
            self.ram.set_sprite_scratch_c(k, c);
            if c == 0 {
                self.ram.set_sprite_state(k, 0);
            }
            self.ram.set_sprite_graphics(k, c >> 3);
            self.runtime.oam_allocate_region_c(self.ram, 4);
            self.runtime.draw_single_small(self.ram, k);
            return;
        }
        if self.ram.sprite(k).scratch_b() != 0 {
            self.faerie_queen_draw(k);
            self.ram
                .set_sprite_graphics(k, self.ram.read_u8(0x1a) >> 4 & 1);
            if self.ram.read_u8(0x1a) & 15 != 0 {
                return;
            }
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x72) {
                let rx = usize::from(self.runtime.random_number(self.ram) & 7);
                let ry = usize::from(self.runtime.random_number(self.ram) & 7);
                self.set_sprite_x(j, self.sprite_x(k).wrapping_add(u16::from(X[rx])));
                self.set_sprite_y(j, self.sprite_y(k).wrapping_add(u16::from(Y[ry])));
                self.ram.set_sprite_scratch_c(j, 31);
                self.ram.set_sprite_scratch_a(j, 31);
                self.ram.set_sprite_flags2(j, 0);
                self.ram.set_sprite_flags3(j, 0x48);
                self.ram.set_sprite_oam_flags(j, 8);
                self.ram.set_sprite_scratch_b(j, 1);
            }
            return;
        }
        let _ = self.runtime.prep_oam(self.ram, k);
        self.sprite_wish_pond2(k);
    }

    pub fn sprite_wish_pond2(&mut self, k: usize) {
        self.wish_pond2_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.read_u8(0xa0) != 21 {
            self.runtime.sprite_wish_pond3(self.ram, k);
        } else {
            self.sprite_happiness_pond(k);
        }
    }

    pub fn wish_pond2_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 8] = [
            (32, -64, 0x0024, 0),
            (32, -56, 0x0034, 0),
            (32, -64, 0x0024, 0),
            (32, -56, 0x0034, 0),
            (32, -64, 0x0024, 2),
            (32, -64, 0x0024, 2),
            (32, -64, 0x0024, 2),
            (32, -64, 0x0024, 2),
        ];
        if self.ram.read_u8(0xa0) == 21 {
            return;
        }
        let s = self.ram.sprite(k).ai_state();
        if !matches!(s, 5 | 6 | 11 | 12) {
            return;
        }
        let g = usize::from(self.ram.sprite(k).graphics());
        let mut flag = self.runtime.wish_pond_item_oam_flag(g as u8);
        if flag == 0xff {
            flag = 5;
        }
        self.ram.set_sprite_oam_flags(k, (flag & 7) * 2);
        let start = usize::from(self.runtime.receive_item_draw_kind(g as u8) >> 1) * 4;
        self.runtime
            .draw_multiple(self.ram, k, &D[start..start + 4]);
    }

    pub fn faerie_queen_draw(&mut self, k: usize) {
        const X: [u8; 24] = [
            0, 16, 0, 8, 16, 24, 0, 8, 16, 24, 0, 16, 0, 16, 0, 8, 16, 24, 0, 8, 16, 24, 0, 16,
        ];
        const Y: [u8; 24] = [
            0, 0, 16, 16, 16, 16, 24, 24, 24, 24, 32, 32, 0, 0, 16, 16, 16, 16, 24, 24, 24, 24, 32,
            32,
        ];
        const CH: [u8; 24] = [
            0xc7, 0xc7, 0xcf, 0xca, 0xca, 0xcf, 0xdf, 0xda, 0xda, 0xdf, 0xcb, 0xcb, 0xcd, 0xcd,
            0xc9, 0xca, 0xca, 0xc9, 0xd9, 0xda, 0xda, 0xd9, 0xcb, 0xcb,
        ];
        const FL: [u8; 24] = [
            0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x40, 0x40, 0, 0x40, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0,
            0x40, 0x40, 0, 0x40,
        ];
        const SIZE: [u8; 24] = [
            2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2,
        ];
        const D: [(i8, i8, u16, u8); 20] = [
            (0, 0, 0x00e9, 2),
            (16, 0, 0x40e9, 2),
            (0, 0, 0x00e9, 2),
            (16, 0, 0x40e9, 2),
            (0, 0, 0x00e9, 2),
            (16, 0, 0x40e9, 2),
            (0, 16, 0x00eb, 2),
            (16, 16, 0x40eb, 2),
            (0, 32, 0x00ed, 2),
            (16, 32, 0x40ed, 2),
            (0, 0, 0x00ef, 0),
            (24, 0, 0x40ef, 0),
            (0, 8, 0x00ff, 0),
            (24, 8, 0x40ff, 0),
            (0, 0, 0x00e9, 2),
            (16, 0, 0x40e9, 2),
            (0, 16, 0x00eb, 2),
            (16, 16, 0x40eb, 2),
            (0, 32, 0x00ed, 2),
            (16, 32, 0x40ed, 2),
        ];
        let g = usize::from(self.ram.sprite(k).graphics());
        if self.ram.read_u8(0xf3ca) == 0 {
            let Some((x, y, flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
                return;
            };
            let first = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
            for i in (0..12).rev() {
                let j = g * 12 + i;
                crate::oam::set_plain(
                    self.ram,
                    first + (11 - i),
                    x.wrapping_add(X[j]),
                    y.wrapping_add(Y[j]),
                    CH[j],
                    flags | FL[j],
                    SIZE[j],
                );
            }
            self.runtime.correct_oam_entries(self.ram, k, 11, 0xff);
        } else {
            self.runtime
                .draw_multiple(self.ram, k, &D[g * 10..g * 10 + 10]);
        }
    }

    pub fn sprite_happiness_pond(&mut self, k: usize) {
        const COST: [u8; 4] = [5, 20, 25, 50];
        const HEX: [u8; 4] = [5, 0x20, 0x25, 0x50];
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.write_u8(0x2e4, 0);
                if self.ram.sprite(k).main_delay() == 0
                    && !self.runtime.check_if_link_is_busy(self.ram)
                    && self.runtime.show_message_on_contact(self.ram, k, 0x89) & 0x100 != 0
                {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.runtime.link_reset_properties_a(self.ram);
                    self.runtime.ancilla_terminate_sparkle_objects(self.ram);
                    self.ram.write_u8(0x2f, 0);
                }
            }
            1 => {
                if self.ram.read_u8(0x1ce8) == 0 {
                    let i =
                        usize::from((self.ram.read_u8(0xf370) | self.ram.read_u8(0xf371)) != 0) * 2;
                    self.ram.set_sprite_graphics(k, i as u8);
                    self.ram.write_u8(0x1cf2, HEX[i]);
                    self.ram.write_u8(0x1cf3, HEX[i + 1]);
                    self.runtime.show_unconditional_message(self.ram, 0x14e);
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.write_u8(0x2e4, 1);
                } else {
                    self.happiness_pond_later(k);
                }
            }
            2 => {
                let i = usize::from(self.ram.sprite(k).graphics() + self.ram.read_u8(0x1ce8));
                self.ram.write_u8(0x1cf3, HEX[i]);
                if self.ram.read_u16(0xf360) < u16::from(COST[i]) {
                    self.happiness_pond_later(k);
                } else {
                    self.ram.set_sprite_scratch_d(k, COST[i]);
                    self.ram.set_sprite_head_direction(k, i as u8);
                    self.ram.set_sprite_ai_state(k, 3);
                }
            }
            3 => {
                let n = u16::from(self.ram.sprite(k).scratch_d());
                self.ram.set_sprite_main_delay(k, 80);
                self.ram
                    .write_u16(0xf360, self.ram.read_u16(0xf360).wrapping_sub(n));
                let t = self.ram.read_u8(0xf36a).wrapping_add(n as u8);
                self.ram.write_u8(0xf36a, t);
                let choice = self.ram.sprite(k).head_direction();
                self.runtime.add_happiness_pond_rupees(self.ram, choice);
                if t >= 100 {
                    self.ram.write_u8(0xf36a, t - 100);
                    self.ram.set_sprite_ai_state(k, 5);
                } else {
                    self.ram.write_u8(0x1cf2, t / 10 * 16 + t % 10);
                    self.ram.set_sprite_ai_state(k, 4);
                }
            }
            4 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.runtime.show_unconditional_message(self.ram, 0x94);
                    self.ram.set_sprite_ai_state(k, 13);
                }
            }
            5 => {
                if self.ram.sprite(k).main_delay() == 0
                    && let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x72)
                {
                    self.set_sprite_x(j, self.sprite_x(k));
                    self.set_sprite_y(j, self.sprite_y(k).wrapping_sub(80));
                    self.ram.write_u8(0x12c, 0x1b);
                    self.ram.write_u8(0x133, 0);
                    self.ram.set_sprite_scratch_b(j, 1);
                    self.runtime.palette_assert_translucency_swap(self.ram);
                    self.runtime.palette_filter_wish_ponds(self.ram);
                    self.ram.set_sprite_scratch_e(k, j as u8);
                    self.ram.set_sprite_ai_state(k, 6);
                    self.ram.set_sprite_main_delay(k, 255);
                }
            }
            6 => {
                if self.ram.read_u8(0x1a) & 7 == 0 {
                    self.runtime.palette_filter_sp5f(self.ram);
                    if self.ram.read_u16(0xc007) == 0 {
                        self.runtime.show_unconditional_message(self.ram, 0x95);
                        self.runtime.palette_revert_translucency_swap(self.ram);
                        self.ram.write_u8(0x1d, 0);
                        self.ram.write_u8(0x9a, 0x20);
                        self.ram
                            .write_u8(0x15, self.ram.read_u8(0x15).wrapping_add(1));
                        self.ram.set_sprite_ai_state(k, 7);
                    }
                }
            }
            7 => self
                .ram
                .set_sprite_ai_state(k, if self.ram.read_u8(0x1ce8) == 0 { 8 } else { 12 }),
            state @ (8 | 12) => {
                const MAX_BOMBS: [u8; 8] = [0x10, 0x15, 0x20, 0x25, 0x30, 0x35, 0x40, 0x50];
                const MAX_ARROWS: [u8; 8] = [0x30, 0x35, 0x40, 0x45, 0x50, 0x55, 0x60, 0x70];
                let bomb = state == 8;
                let upgrade_addr = if bomb { 0xf370 } else { 0xf371 };
                let i = self.ram.read_u8(upgrade_addr).wrapping_add(1);
                if i != 8 {
                    self.ram.write_u8(upgrade_addr, i);
                    let value = if bomb {
                        MAX_BOMBS[usize::from(i)]
                    } else {
                        MAX_ARROWS[usize::from(i)]
                    };
                    self.ram.write_u8(0x1cf2, value);
                    self.ram.write_u8(if bomb { 0xf375 } else { 0xf376 }, value);
                    self.runtime
                        .show_unconditional_message(self.ram, if bomb { 0x96 } else { 0x97 });
                } else {
                    self.ram
                        .write_u16(0xf360, self.ram.read_u16(0xf360).wrapping_add(100));
                    self.runtime.show_unconditional_message(self.ram, 0x98);
                }
                self.ram.set_sprite_ai_state(k, 9);
            }
            9 => {
                self.runtime.palette_assert_translucency_swap(self.ram);
                self.ram.write_u8(0x1d, 2);
                self.ram.write_u8(0x9a, 0x30);
                self.ram
                    .write_u8(0x15, self.ram.read_u8(0x15).wrapping_add(1));
                self.ram.set_sprite_ai_state(k, 10);
            }
            10 => {
                if self.ram.read_u8(0x1a) & 7 == 0 {
                    self.runtime.palette_filter_sp5f(self.ram);
                    let countdown = self.ram.read_u16(0xc007);
                    if countdown == 30 {
                        let fairy = usize::from(self.ram.sprite(k).scratch_e());
                        self.ram.set_sprite_state(fairy, 0);
                    } else if countdown == 0 {
                        self.ram.set_sprite_ai_state(k, 11);
                    }
                }
            }
            11 => {
                self.runtime.palette_filter_restore_sp5f(self.ram);
                self.runtime.palette_revert_translucency_swap(self.ram);
                self.ram.set_sprite_ai_state(k, 0);
                self.ram.set_sprite_main_delay(k, 255);
            }
            13 => {
                self.runtime.show_unconditional_message(self.ram, 0x154);
                self.ram.set_sprite_ai_state(k, 14);
            }
            14 => {
                const MSG: [u16; 4] = [0x150, 0x151, 0x152, 0x153];
                const L: [u8; 4] = [1, 0, 0, 2];
                let i = usize::from(self.runtime.random_number(self.ram) & 3);
                self.ram.write_u8(0xcf9, L[i]);
                self.ram.write_u8(0xcfa, 0);
                self.runtime.show_unconditional_message(self.ram, MSG[i]);
                self.ram.set_sprite_ai_state(k, 0);
                self.ram.set_sprite_main_delay(k, 255);
            }
            _ => {}
        }
    }
    fn happiness_pond_later(&mut self, k: usize) {
        self.runtime.show_unconditional_message(self.ram, 0x14c);
        self.ram.set_sprite_ai_state(k, 0);
        self.ram.set_sprite_main_delay(k, 255);
    }

    pub fn sprite_e3_fairy(&mut self, k: usize) {
        self.ram.set_sprite_ignore_projectile(k, 1);
        let state = self.ram.sprite(k).ai_state();
        if state == 0 {
            if self.ram.read_u8(0x1b) == 0 {
                self.ram.set_sprite_object_priority(k, 48);
            }
            if self.runtime.draw_absorbable_transient(self.ram, k, true) {
                return;
            }
        }
        self.fairy_check_if_touchable(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match state {
            0 => {
                if self.ram.sprite(k).aux4_delay() == 0 {
                    if self.runtime.check_damage_to_link(self.ram, k) {
                        self.runtime.handle_absorption_by_player(self.ram, k);
                    } else if self.runtime.check_damage_from_link(self.ram, k) != 0 {
                        self.ram.set_sprite_ai_state(k, 1);
                        self.runtime.show_unconditional_message(self.ram, 0xc9);
                        return;
                    }
                }
                if self.ram.sprite(k).state() == 0 && self.runtime.misc_bug_fixes_enabled() {
                    return;
                }
                if self.runtime.handle_dragging_by_ancilla(self.ram, k) {
                    return;
                }
                self.runtime.faerie_handle_movement(self.ram, k);
            }
            1 => {
                if self.ram.read_u8(0x1ce8) == 0 {
                    if let Some(j) = self.runtime.find_empty_bottle(self.ram) {
                        self.ram.write_u8(0xf35c + j, 6);
                        self.runtime.hud_refresh_icon(self.ram);
                        self.ram.set_sprite_state(k, 0);
                        return;
                    }
                    self.runtime.show_unconditional_message(self.ram, 0xca);
                }
                self.ram.set_sprite_aux4_delay(k, 48);
                self.ram.set_sprite_ai_state(k, 0);
            }
            _ => {}
        }
    }

    pub fn sprite_d2_flopping_fish(&mut self, k: usize) {
        const XV: [i8; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
        const YV: [i8; 8] = [-16, -12, 0, 12, 16, 12, 0, -12];
        const TAB: [u8; 2] = [2, 0];
        const G: [u8; 3] = [1, 5, 3];
        const G2: [u8; 17] = [5, 5, 6, 6, 5, 5, 4, 4, 3, 7, 7, 8, 8, 7, 7, 8, 8];
        if self.ram.read_u8(0xfc6) < 3 {
            self.runtime.fish_draw(self.ram, k);
        }
        if self.ram.sprite(k).state() == 10 {
            self.ram.set_sprite_ai_state(k, 4);
            self.ram
                .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 4 & 1) + 3);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.runtime.check_tile_collision(self.ram, k);
                if self.ram.read_u8(0xfa5) == 8 {
                    self.ram.set_sprite_state(k, 0);
                } else {
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 => {
                self.runtime.return_if_lifted_permissive(self.ram, k);
                self.runtime.bounce_from_tile_collision(self.ram, k);
                self.runtime.move_xyz(self.ram, k);
                self.ram
                    .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(2));
                if (self.ram.sprite(k).z() as i8) < 0 {
                    self.ram.set_sprite_z(k, 0);
                    let tile = self.ram.read_u8(0xfa5);
                    if tile == 9 {
                        self.runtime.spawn_small_splash(self.ram, k);
                    } else if tile == 8 {
                        self.ram.set_sprite_state(k, 0);
                        self.runtime.spawn_small_splash(self.ram, k);
                    }
                    let zv = (self.runtime.random_number(self.ram) & 15) + 16;
                    self.ram.set_sprite_velocity_z(k, zv);
                    let j = usize::from(self.runtime.random_number(self.ram) & 7);
                    self.ram.set_sprite_velocity_x(k, XV[j] as u8);
                    self.ram.set_sprite_velocity_y(k, YV[j] as u8);
                    self.ram
                        .set_sprite_scratch_d(k, self.ram.sprite(k).scratch_d().wrapping_add(1));
                    self.ram.set_sprite_subtype2(k, 3);
                }
                let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(k, s);
                if s & 7 == 0 {
                    let j = usize::from(self.ram.sprite(k).scratch_d() & 1);
                    let a = self.ram.sprite(k).scratch_a();
                    if a != TAB[j] {
                        self.ram.set_sprite_scratch_a(
                            k,
                            a.wrapping_add(if j != 0 { u8::MAX } else { 1 }),
                        );
                    }
                }
                self.ram.set_sprite_graphics(
                    k,
                    G[usize::from(self.ram.sprite(k).scratch_a())]
                        + (self.ram.read_u8(0x1a) >> 3 & 1),
                );
            }
            2 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_velocity_z(k, 48);
                    self.runtime.spawn_small_splash(self.ram, k);
                }
            }
            3 => {
                self.runtime.move_z(self.ram, k);
                let zv = self.ram.sprite(k).velocity_z().wrapping_sub(2);
                self.ram.set_sprite_velocity_z(k, zv);
                if zv == 0 && self.ram.sprite(k).scratch_a() != 0 {
                    self.ram.write_u16(0x1cf0, 0x176);
                    self.runtime.show_message_minimal(self.ram);
                }
                if (self.ram.sprite(k).z() as i8) < 0 {
                    self.ram.set_sprite_z(k, 0);
                    self.runtime.spawn_small_splash(self.ram, k);
                    if self.ram.sprite(k).scratch_a() != 0
                        && let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xdb)
                    {
                        self.runtime.set_spawned_coordinates(self.ram, j);
                        self.ram
                            .set_sprite_x_low(j, self.ram.sprite(j).x_low().wrapping_add(4));
                        self.ram.set_sprite_stunned(j, 255);
                        self.ram.set_sprite_velocity_z(j, 48);
                        self.ram.set_sprite_aux3_delay(j, 48);
                        self.runtime.apply_speed_towards_link(self.ram, j, 16);
                    }
                    self.ram.set_sprite_state(k, 0);
                }
                let s = self.ram.sprite(k).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(k, s);
                self.ram.set_sprite_graphics(k, G2[usize::from(s >> 2)]);
            }
            4 => {
                if self.ram.sprite(k).z() == 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                }
                self.runtime.move_xy(self.ram, k);
                self.runtime
                    .thrown_sprite_tile_and_sprite_interaction(self.ram, k);
            }
            _ => {}
        }
    }

    pub fn sprite_08_octorok(&mut self, k: usize) {
        const TAB0: [u8; 20] = [0, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 1, 1, 0];
        const TAB1: [u8; 10] = [2, 2, 2, 2, 2, 2, 2, 2, 1, 0];
        const NEXT: [u8; 4] = [2, 3, 1, 0];
        const DIR: [u8; 4] = [3, 2, 0, 1];
        const XV: [i8; 4] = [24, -24, 0, 0];
        const YV: [i8; 4] = [0, 0, 24, -24];
        const FL: [u8; 4] = [0x40, 0, 0, 0];
        let d = self.ram.sprite(k).scratch_d();
        if self.ram.sprite(k).aux1_delay() != 0 {
            self.ram.set_sprite_scratch_d(k, DIR[usize::from(d)]);
        }
        self.ram.set_sprite_oam_flags(
            k,
            (self.ram.sprite(k).oam_flags() & !0x40)
                | FL[usize::from(d)]
                | if self.ram.sprite(k).graphics() == 7 {
                    0x40
                } else {
                    0
                },
        );
        self.octorock_draw(k);
        self.ram.set_sprite_scratch_d(k, d);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.move_xy(self.ram, k);
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        if self.ram.sprite(k).ai_state() & 1 == 0 {
            let s = self.ram.sprite(k).subtype2().wrapping_add(1);
            self.ram.set_sprite_subtype2(k, s);
            self.ram.set_sprite_graphics(k, (s >> 3 & 3) | (d & 2) << 1);
            if self.ram.sprite(k).main_delay() == 0 {
                self.ram
                    .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
                self.ram.set_sprite_main_delay(
                    k,
                    if self.ram.sprite(k).kind() == 8 {
                        60
                    } else {
                        160
                    },
                );
            } else {
                self.ram.set_sprite_velocity_x(k, XV[usize::from(d)] as u8);
                self.ram.set_sprite_velocity_y(k, YV[usize::from(d)] as u8);
                if self.runtime.check_tile_collision(self.ram, k) {
                    self.ram.set_sprite_scratch_d(k, d ^ 1);
                }
            }
            return;
        }
        self.ram.set_sprite_velocity_x(k, 0);
        self.ram.set_sprite_velocity_y(k, 0);
        let delay = self.ram.sprite(k).main_delay();
        if delay == 0 {
            let n = (self.runtime.random_number(self.ram) & 0x3f) + 48;
            self.ram
                .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
            self.ram.set_sprite_main_delay(k, n);
            self.ram.set_sprite_scratch_d(k, n & 3);
        } else if self.ram.sprite(k).kind() == 8 {
            if delay == 28 {
                self.octorok_fire_loogie(k);
            }
            self.ram
                .set_sprite_scratch_c(k, TAB0[usize::from(delay >> 3)]);
        } else {
            if delay < 128 {
                if delay & 15 == 0 {
                    self.ram.set_sprite_scratch_d(k, NEXT[usize::from(d)]);
                }
                if delay & 15 == 8 {
                    self.octorok_fire_loogie(k);
                }
            }
            self.ram
                .set_sprite_scratch_c(k, TAB1[usize::from(delay >> 4)]);
        }
    }

    pub fn octorok_fire_loogie(&mut self, k: usize) {
        const X: [i8; 4] = [12, -12, 0, 0];
        const Y: [i8; 4] = [4, 4, 12, -12];
        const XV: [i8; 4] = [44, -44, 0, 0];
        const YV: [i8; 4] = [0, 0, 44, -44];
        self.runtime.sfx2_with_pan(self.ram, k, 7);
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x0c) {
            let i = usize::from(self.ram.sprite(k).scratch_d());
            let x = self.sprite_x(k).wrapping_add(X[i] as i16 as u16);
            let y = self.sprite_y(k).wrapping_add(Y[i] as i16 as u16);
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            self.ram.set_sprite_velocity_x(j, XV[i] as u8);
            self.ram.set_sprite_velocity_y(j, YV[i] as u8);
        }
    }

    pub fn octorock_draw(&mut self, k: usize) {
        const X: [i8; 9] = [8, 0, 4, 8, 0, 4, 9, -1, 4];
        const Y: [i8; 9] = [6, 6, 9, 6, 6, 9, 6, 6, 9];
        const CH: [u8; 9] = [0xbb, 0xbb, 0xba, 0xab, 0xab, 0xaa, 0xa9, 0xa9, 0xb9];
        const FL: [u8; 9] = [0x65, 0x25, 0x25, 0x65, 0x25, 0x25, 0x65, 0x25, 0x25];
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags_wide(self.ram, k) else {
            return;
        };
        let d = usize::from(self.ram.sprite(k).scratch_d());
        if d != 3 {
            let j = usize::from(self.ram.sprite(k).scratch_c()) * 3 + d;
            self.runtime.emit_oam_wide(
                self.ram,
                k,
                x.wrapping_add(X[j] as i16 as u16),
                y.wrapping_add(Y[j] as i16 as u16),
                CH[j],
                FL[j] | flags,
            );
        } else {
            self.ram
                .write_u16(0x90, self.ram.read_u16(0x90).wrapping_add(4));
            self.ram
                .write_u16(0x92, self.ram.read_u16(0x92).wrapping_add(1));
        }
        let f = self.ram.sprite(k).flags2();
        self.ram.set_sprite_flags2(k, f.wrapping_sub(1));
        self.runtime
            .prep_and_draw_single_large_no_prep(self.ram, k, x, y, flags);
        self.ram.set_sprite_flags2(k, f);
    }

    pub fn sprite_02_stalfos_head(&mut self, k: usize) {
        const FL: [u8; 4] = [0, 0, 0, 0x40];
        const GFX: [u8; 4] = [0, 1, 2, 1];
        self.ram.set_sprite_floor(k, self.ram.read_u8(0xee));
        if self.ram.sprite(k).aux1_delay() != 0 {
            self.runtime.oam_allocate_region_c(self.ram, 8);
        }
        let j = usize::from(self.ram.sprite(k).subtype2() >> 3 & 3);
        self.ram
            .set_sprite_oam_flags(k, (self.ram.sprite(k).oam_flags() & !0x40) | FL[j]);
        self.ram.set_sprite_graphics(k, GFX[j]);
        self.ram.set_sprite_object_priority(k, 0x30);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        if self.ram.sprite(k).scratch_f() != 0 {
            self.ram.set_sprite_velocity_x(k, 0);
            self.ram.set_sprite_velocity_y(k, 0);
        }
        self.runtime.move_xy(self.ram, k);
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        let delay = self.ram.sprite(k).main_delay();
        if delay == 0 && (k as u8 ^ self.ram.read_u8(0x1a)) & 3 != 0 {
            return;
        }
        if delay != 0 && delay & 1 != 0 {
            return;
        }
        let (mut tx, mut ty, _, _) = self.runtime.project_speed_towards_link(self.ram, k, 16);
        if delay == 0 {
            tx = tx.wrapping_neg();
            ty = ty.wrapping_neg();
        }
        self.ram.sprite_mut(k).approach_velocity(tx as u8, ty as u8);
    }

    pub fn sprite_0d_buzzblob(&mut self, k: usize) {
        const GFX: [u8; 4] = [0, 1, 0, 2];
        const PRIO: [u8; 4] = [10, 2, 8, 2];
        let aux = self.ram.sprite(k).aux1_delay();
        if aux != 0 {
            self.ram.set_sprite_object_priority(
                k,
                (self.ram.sprite(k).object_priority() & 0xf1) | PRIO[usize::from(aux >> 1 & 3)],
            );
        }
        self.sprite_cukeman(k);
        self.buzz_blob_draw(k);
        let phase = usize::from(self.ram.sprite(k).subtype2() >> 3 & 3);
        self.ram
            .set_sprite_graphics(k, GFX[phase] + u8::from(aux != 0) * 3);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        if self.ram.sprite(k).main_delay() == 0 {
            self.buzzblob_select_new_direction(k);
        }
        if aux == 0 {
            self.runtime.move_xy(self.ram, k);
        }
        self.runtime.check_tile_collision2(self.ram, k);
        self.runtime.bounce_off_wall(self.ram, k);
        self.runtime.check_damage_to_and_from_link(self.ram, k);
    }

    pub fn buzz_blob_draw(&mut self, k: usize) {
        const X: [i8; 3] = [0, 8, 0];
        const Y: [i8; 3] = [-8, -8, 0];
        const CH: [u8; 18] = [
            0xf0, 0xf0, 0xe1, 0, 0, 0xce, 0, 0, 0xce, 0xe3, 0xe3, 0xca, 0xe4, 0xe5, 0xcc, 0xe5,
            0xe4, 0xcc,
        ];
        const FL: [u8; 18] = [
            0, 0x40, 0, 0, 0, 0, 0, 0, 0x40, 0, 0x40, 0, 0, 0, 0, 0x40, 0x40, 0x40,
        ];
        const SIZE: [u8; 3] = [0, 0, 2];
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags_wide(self.ram, k) else {
            return;
        };
        let g = usize::from(self.ram.sprite(k).graphics());
        let oam_base = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for (piece, i) in (0..3).rev().enumerate() {
            let ch = CH[g * 3 + i];
            crate::oam::set_helper0(
                self.ram,
                oam_base + piece,
                x.wrapping_add(X[i] as u16),
                y.wrapping_add(Y[i] as u16),
                ch,
                FL[g * 3 + i] | flags,
                SIZE[i],
            );
            if ch == 0 {
                self.ram
                    .write_u8(crate::oam::OAM_BUFFER + (oam_base + piece) * 4 + 1, 240);
            }
        }
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_cukeman(&mut self, k: usize) {
        if self.ram.sprite(k).head_direction() == 0 {
            return;
        }
        let x = self.sprite_x(k);
        let y = self.sprite_y(k);
        if self.ram.sprite(k).state() == 9
            && (self.ram.read_u8(0x11) | self.ram.read_u8(0xfc1)) == 0
            && x.wrapping_sub(self.ram.read_u16(0x22)).wrapping_add(0x18) < 0x30
            && self.ram.read_u16(0x20).wrapping_sub(y).wrapping_add(0x20) < 0x30
            && self.ram.read_u8(0xf6) & 0x80 != 0
        {
            let subtype = self.ram.sprite(k).subtype();
            self.ram.write_u16(0x1cf0, 0x17a + u16::from(subtype & 1));
            self.ram.set_sprite_subtype(k, subtype.wrapping_add(1));
            self.runtime.show_message_minimal(self.ram);
        }
        let old = self.ram.sprite(k).oam_flags() & 0xf0;
        self.ram.set_sprite_oam_flags(k, old | 8);
        self.cukeman_draw(k);
        self.ram.set_sprite_oam_flags(k, old | 0xd);
        self.runtime.oam_allocate_region_a(self.ram, 0x10);
    }

    pub fn cukeman_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 18] = [
            (0, 0, 0x01f3, 0),
            (7, 0, 0x41f3, 0),
            (4, 7, 0x07e0, 0),
            (-1, 2, 0x01f3, 0),
            (6, 1, 0x41f3, 0),
            (4, 8, 0x07e0, 0),
            (1, 1, 0x01f3, 0),
            (8, 2, 0x41f3, 0),
            (4, 8, 0x07e0, 0),
            (-2, 0, 0x01f3, 0),
            (10, 0, 0x41f3, 0),
            (4, 7, 0x07e0, 0),
            (0, 0, 0x01f3, 0),
            (8, 0, 0x41f3, 0),
            (4, 6, 0x07e0, 0),
            (-5, 0, 0x01f3, 0),
            (16, 0, 0x41f3, 0),
            (4, 8, 0x07e0, 0),
        ];
        let g = usize::from(self.ram.sprite(k).graphics()) * 3;
        self.runtime.draw_multiple(self.ram, k, &D[g..g + 3]);
    }

    pub fn sprite_0f_octoballoon(&mut self, k: usize) {
        const Z: [u8; 8] = [16, 17, 18, 19, 20, 19, 18, 17];
        let phase = usize::from(self.ram.sprite(k).subtype2() >> 3 & 7);
        self.ram.set_sprite_z(k, Z[phase]);
        self.octoballoon_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).main_delay() == 0 {
            self.ram.set_sprite_main_delay(k, 3);
            if !self.runtime.octoballoon_find(self.ram) {
                self.ram.set_sprite_state(k, 6);
                self.ram.set_sprite_aux1_delay(k, 0);
                self.ram.set_sprite_main_delay(k, 15);
                return;
            }
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        if (k as u8 ^ self.ram.read_u8(0x1a)) & 15 == 0 {
            let (tx, ty, _, _) = self.runtime.project_speed_towards_link(self.ram, k, 4);
            self.ram.sprite_mut(k).approach_velocity(tx as u8, ty as u8);
        }
        self.runtime.move_xy(self.ram, k);
        if self.runtime.check_damage_to_link(self.ram, k) {
            self.octoballoon_recoil_link(k);
        }
        self.runtime.check_damage_from_link(self.ram, k);
        self.runtime.check_tile_collision2(self.ram, k);
        self.runtime.bounce_off_wall(self.ram, k);
    }

    pub fn octoballoon_draw(&mut self, k: usize) {
        const X: [i8; 12] = [-4, 4, -4, 4, -8, 8, -8, 8, -4, 4, -4, 4];
        const Y: [i8; 12] = [-4, -4, 4, 4, -8, -8, 8, 8, -4, -4, 4, 4];
        const CH: [u8; 12] = [
            0x8c, 0x8c, 0x9c, 0x9c, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86,
        ];
        const FL: [u8; 12] = [0, 0x40, 0, 0x40, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0];
        let mut d = 0;
        if self.ram.sprite(k).state() == 6 {
            let delay = self.ram.sprite(k).main_delay();
            if delay == 6 && self.ram.read_u8(0x11) == 0 {
                self.octoballoon_form_babby(k);
            }
            d = usize::from((delay >> 1 & 4) + 4);
        }
        let Some((_, _, flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        // The C loop advances a local OAM pointer, leaving the global OAM
        // pointer unchanged for SpriteDraw_Shadow. Its coordinates are 16-bit.
        let x = self.ram.read_u16(0);
        let y = self.ram.read_u16(2);
        let base = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for i in (0..4).rev() {
            let j = d + i;
            crate::oam::set_helper0(
                self.ram,
                base + (3 - i),
                x.wrapping_add_signed(i16::from(X[j])),
                y.wrapping_add_signed(i16::from(Y[j])),
                CH[j],
                FL[j] | flags,
                2,
            );
        }
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_37_waterfall(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.waterfall(k),
            1 => self.sprite_bat_crash(k),
            _ => {}
        }
    }

    pub fn waterfall(&mut self, k: usize) {
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            if self.ram.read_u8(0x8a) == 0x43 {
                self.runtime.ancilla_add_gt_cutscene(self.ram);
            } else {
                self.runtime.ancilla_add_waterfall_splash(self.ram);
            }
        }
    }

    pub fn sprite_bat_crash(&mut self, k: usize) {
        const XP: [u16; 4] = [0x7dc, 0x7f0, 0x820, 0x818];
        const YP: [u16; 4] = [0x62e, 0x636, 0x630, 0x5e0];
        const DELAY: [u8; 5] = [4, 3, 4, 6, 0];
        self.retreat_bat_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.move_xy(self.ram, k);
        self.bat_crash_draw_hardcoded_garbage(k);
        self.ram.write_u16(0x11c, 0);
        let shake = self.ram.sprite(k).aux3_delay();
        if shake != 0 {
            if shake == 1 {
                self.ram.write_u8(0x12d, 5);
            }
            self.ram
                .write_u16(0x11c, if shake & 1 != 0 { 1 } else { u16::MAX });
        }
        if self.ram.sprite(k).main_delay() == 0 {
            let gfx = self.ram.sprite(k).graphics().wrapping_add(1) & 3;
            self.ram.set_sprite_graphics(k, gfx);
            if gfx == 0 && self.ram.sprite(k).ai_state() < 2 {
                self.runtime.sfx2_with_pan(self.ram, k, 3);
            }
            let d = usize::from(self.ram.sprite(k).scratch_d());
            self.ram.set_sprite_main_delay(k, DELAY[d]);
        }
        match self.ram.sprite(k).ai_state() {
            0 | 1 => {
                let mut j = usize::from(self.ram.sprite(k).scratch_a());
                if self.ram.sprite(k).ai_state() == 0 {
                    let x = self.ram.read_u16(0xfd8);
                    if XP[j] < x {
                        if j >= 2 {
                            self.ram.set_sprite_ai_state(k, 1);
                            self.ram.set_sprite_aux1_delay(k, 208);
                        }
                        self.ram.set_sprite_scratch_a(k, (j + 1) as u8);
                        self.ram.set_sprite_scratch_d(
                            k,
                            self.ram.sprite(k).scratch_d().wrapping_add(1),
                        );
                    }
                } else if self.ram.sprite(k).aux1_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.runtime.sfx3_with_pan(self.ram, k, 0x26);
                    self.ram
                        .set_sprite_scratch_d(k, self.ram.sprite(k).scratch_d().wrapping_add(1));
                    self.set_sprite_x(k, 0x7e8);
                    self.set_sprite_y(k, 0x5e0);
                    self.ram.set_sprite_velocity_x(k, 0);
                    self.ram.set_sprite_velocity_y(k, 64);
                    self.ram.set_sprite_aux1_delay(k, 45);
                    return;
                } else {
                    if self.ram.read_u8(0x1a) & 3 == 0 {
                        self.ram.set_sprite_velocity_x(
                            k,
                            self.ram.sprite(k).velocity_x().wrapping_sub(1),
                        );
                    }
                    j = usize::from(self.ram.sprite(k).scratch_a());
                }
                if self.ram.read_u8(0x1a) & 7 == 0 {
                    let yv = if YP[j] >= self.ram.read_u16(0xfda) {
                        1
                    } else {
                        u8::MAX
                    };
                    self.ram
                        .set_sprite_velocity_y(k, self.ram.sprite(k).velocity_y().wrapping_add(yv));
                }
                if self.ram.read_u8(0x1a) & 15 == 0 {
                    self.ram
                        .set_sprite_velocity_x(k, self.ram.sprite(k).velocity_x().wrapping_add(1));
                }
            }
            2 => {
                let delay = self.ram.sprite(k).aux1_delay();
                if delay == 0 {
                    self.ram.set_sprite_velocity_y(k, 0);
                    self.ram.set_sprite_aux1_delay(k, 96);
                    self.ram.set_sprite_ai_state(k, 3);
                }
                if delay == 9 {
                    self.bat_crash_spawn_debris(k);
                    self.runtime.create_pyramid_hole(self.ram);
                }
            }
            3 if self.ram.sprite(k).aux1_delay() == 0 => {
                self.ram.set_sprite_state(k, 0);
                self.ram
                    .write_u8(0x200, self.ram.read_u8(0x200).wrapping_add(1));
            }
            _ => {}
        }
    }

    pub fn sprite_spawn_bat_crash_cutscene(&mut self) {
        if let Some(j) = self.runtime.spawn_dynamic(self.ram, 0, 0x37) {
            self.ram.set_sprite_velocity_y(j, 0);
            self.ram.set_sprite_scratch_b(j, 0);
            self.ram.set_sprite_scratch_d(j, 0);
            self.ram.set_sprite_floor(j, 0);
            self.ram.set_sprite_subtype2(j, 1);
            self.ram.set_sprite_flags2(j, 1);
            self.ram.set_sprite_flags3(j, 1);
            self.ram.set_sprite_oam_flags(j, 1);
            self.set_sprite_x(j, 0x7cc);
            self.set_sprite_y(j, 0x632);
            self.ram.set_sprite_flags5(j, 0x80);
        }
    }

    pub fn bat_crash_draw_hardcoded_garbage(&mut self, _k: usize) {
        const O: [(i8, i8, u8, u8, u8); 8] = [
            (104, -105, 0x57, 1, 2),
            (120, -105, 0x57, 1, 2),
            (-120, -105, 0x57, 1, 2),
            (104, -89, 0x57, 1, 2),
            (120, -89, 0x57, 1, 2),
            (-120, -89, 0x57, 1, 2),
            (101, -112, 0x57, 1, 2),
            (-117, -112, 0x57, 1, 2),
        ];
        for (i, &(x, y, character, flags, _)) in O.iter().enumerate() {
            let address = 0x800 + (76 + i) * 4;
            self.ram.write_u8(address, x as u8);
            self.ram.write_u8(address + 1, y as u8);
            self.ram.write_u8(address + 2, character);
            self.ram.write_u8(address + 3, flags);
        }
        // C writes nine bytewise-extended entries, but only eight base entries.
        self.ram.slice_mut(0xa20 + 76, 9).fill(2);
    }

    pub fn bat_crash_spawn_debris(&mut self, k: usize) {
        const X: [i8; 30] = [
            -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32,
            -8, 0, 8, 16, 24, 32,
        ];
        const Y: [i8; 30] = [
            48, 48, 48, 48, 48, 48, 40, 40, 40, 40, 40, 40, 32, 32, 32, 32, 32, 32, 24, 24, 24, 24,
            24, 24, 16, 16, 16, 16, 16, 16,
        ];
        const XV: [i8; 30] = [
            -30, -25, -8, 8, 25, 30, -50, -45, -20, 20, 45, 50, -50, -35, -25, 25, 35, 50, -45,
            -50, -60, 60, 50, 45, -30, -35, -40, 40, 35, 30,
        ];
        const YV: [i8; 30] = [
            2, 5, 10, 10, 5, 2, 5, 20, 30, 30, 20, 5, 10, 30, 40, 40, 30, 10, -20, -40, -60, -60,
            -40, -20, -10, -20, -40, -40, -20, -10,
        ];
        for i in (0..30).rev() {
            self.runtime
                .garnish_spawn_pyramid_debris(self.ram, X[i], Y[i], XV[i], YV[i]);
        }
        self.ram.set_sprite_aux3_delay(k, 32);
    }

    pub fn retreat_bat_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 18] = [
            (0, 0, 0x044b, 0),
            (5, -4, 0x045b, 0),
            (-2, -4, 0x0464, 2),
            (-2, -4, 0x0449, 2),
            (-8, -9, 0x046c, 2),
            (8, -9, 0x446c, 2),
            (-8, -7, 0x044c, 2),
            (8, -7, 0x444c, 2),
            (-8, -9, 0x0444, 2),
            (8, -9, 0x4444, 2),
            (-8, -8, 0x0462, 2),
            (8, -8, 0x4462, 2),
            (-8, -7, 0x0460, 2),
            (8, -7, 0x4460, 2),
            (0, 0, 0x044e, 2),
            (16, 0, 0x444e, 2),
            (0, 16, 0x046e, 2),
            (16, 16, 0x446e, 2),
        ];
        const OFF: [usize; 20] = [
            0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 6, 6, 8, 10, 12, 10, 14, 14, 14, 14,
        ];
        const COUNT: [usize; 20] = [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4];
        self.ram.write_u16(0x90, 0x960);
        self.ram.write_u16(0x92, 0xa78);
        let j = usize::from(self.ram.sprite(k).scratch_d()) * 4
            + usize::from(self.ram.sprite(k).graphics());
        self.runtime
            .draw_multiple(self.ram, k, &D[OFF[j]..OFF[j] + COUNT[j]]);
    }

    pub fn sprite_prep_haunted_grove_ostritch(&mut self, k: usize) {
        self.sprite_prep_haunted_grove_ostrich(k);
    }

    pub fn sprite_21_water_switch(&mut self, k: usize) {
        const DELAY: [u8; 10] = [40, 6, 3, 3, 3, 5, 1, 1, 3, 12];
        const DIR: [u8; 10] = [0, 1, 2, 3, 4, 5, 5, 6, 7, 6];
        self.push_switch_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.sprite(k).scratch_c() != 0 {
                    let b = self.ram.sprite(k).scratch_b().wrapping_sub(1);
                    self.ram.set_sprite_scratch_b(k, b);
                    if b == 0 {
                        self.ram.set_sprite_ai_state(k, 1);
                    }
                    if self.ram.read_u8(0x1a) & 3 == 0 {
                        self.runtime.sfx2_with_pan(self.ram, k, 0x22);
                    }
                } else {
                    self.ram.set_sprite_scratch_b(k, 48);
                }
            }
            1 if self.ram.sprite(k).main_delay() == 0 => {
                let a = self.ram.sprite(k).scratch_a().wrapping_add(1);
                self.ram.set_sprite_scratch_a(k, a);
                if a == 10 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram
                        .write_u8(0x642, self.ram.read_u8(0x642).wrapping_add(1));
                    self.runtime.sfx3_with_pan(self.ram, k, 0x25);
                } else {
                    self.ram.set_sprite_main_delay(k, DELAY[usize::from(a)]);
                    self.ram.set_sprite_scratch_d(k, DIR[usize::from(a)]);
                    self.runtime.sfx2_with_pan(self.ram, k, 0x22);
                }
            }
            _ => {}
        }
    }

    pub fn push_switch_draw(&mut self, k: usize) {
        const O: [(i8, i8, u8, u8); 40] = [
            (4, 20, 0xdc, 0x20),
            (4, 12, 0xdd, 0x20),
            (4, 12, 0xdd, 0x20),
            (4, 12, 0xdd, 0x20),
            (0, 0, 0xca, 0x20),
            (3, 12, 0xdd, 0x20),
            (3, 20, 0xdc, 0x20),
            (3, 20, 0xdc, 0x20),
            (3, 20, 0xdc, 0x20),
            (0, 0, 0xca, 0x20),
            (-8, 8, 0xea, 0x20),
            (0, 8, 0xeb, 0x20),
            (-8, 16, 0xfa, 0x20),
            (0, 16, 0xfb, 0x20),
            (0, 0, 0xca, 0x20),
            (-12, 4, 0xcc, 0x20),
            (-4, 4, 0xcd, 0x20),
            (-4, 4, 0xcd, 0x20),
            (-4, 4, 0xcd, 0x20),
            (0, 0, 0xca, 0x20),
            (-10, 4, 0xcc, 0x20),
            (-4, 4, 0xcd, 0x20),
            (-4, 4, 0xcd, 0x20),
            (-4, 4, 0xcd, 0x20),
            (0, 0, 0xca, 0x20),
            (-8, 4, 0xcc, 0x20),
            (-4, 4, 0xcd, 0x20),
            (-4, 4, 0xcd, 0x20),
            (-4, 4, 0xcd, 0x20),
            (0, 0, 0xca, 0x20),
            (4, 3, 0xe2, 0x20),
            (-6, 4, 0xcc, 0x20),
            (-4, 4, 0xcd, 0x20),
            (-4, 4, 0xcd, 0x20),
            (0, 0, 0xca, 0x20),
            (4, 3, 0xf1, 0x20),
            (-6, 4, 0xcc, 0x20),
            (-4, 4, 0xcd, 0x20),
            (-4, 4, 0xcd, 0x20),
            (0, 0, 0xca, 0x20),
        ];
        const WH: [u8; 16] = [8, 6, 16, 16, 16, 8, 16, 8, 16, 8, 16, 8, 16, 3, 16, 8];
        self.runtime.oam_allocate_defer_to_player(self.ram, k);
        let Some((_x, _y, _prep_flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        let d = usize::from(self.ram.sprite(k).scratch_d());
        let flags = if self.ram.read_u8(0xabd) != 0 {
            self.ram.sprite(k).oam_flags() | 0xe
        } else {
            self.ram.sprite(k).oam_flags() & !0xe
        };
        self.ram.set_sprite_oam_flags(k, flags);
        let r1 = self.ram.sprite(k).scratch_b() >> 2 & 3;
        let map = self.ram.read_u16(0xfa8);
        let xv = (map as u8).wrapping_sub(r1);
        let yv = (map >> 8) as u8 - (r1 >> 1);
        // C reserves one OAM entry, then writes five entries starting at the
        // next one without advancing the shared pointer past those writes.
        let oam_start = self.ram.read_u16(0x90).wrapping_add(4);
        let ext_start = self.ram.read_u16(0x92).wrapping_add(1);
        self.ram.write_u16(0x90, oam_start);
        self.ram.write_u16(0x92, ext_start);
        for (i, &(ox, oy, ch, fl)) in O[d * 5..d * 5 + 5].iter().enumerate() {
            let size = if i == 4 { 2 } else { 0 };
            let piece_x = if i == 4 { map as u8 } else { xv };
            let piece_y = if i == 4 { (map >> 8) as u8 } else { yv };
            self.runtime.emit_oam_sized(
                self.ram,
                k,
                (
                    piece_x.wrapping_add(ox as u8),
                    piece_y.wrapping_add(oy as u8),
                    ch,
                    fl | flags,
                    size,
                ),
            );
        }
        self.ram.write_u16(0x90, oam_start);
        self.ram.write_u16(0x92, ext_start);
        self.runtime.correct_oam_entries(self.ram, k, 4, 0xff);
        if self.ram.sprite(k).floor() != self.ram.read_u8(0xee) {
            return;
        }
        self.ram.set_sprite_scratch_c(k, 0);
        let hit_x = self.sprite_x(k).wrapping_add(O[d * 4].0 as i16 as u16);
        let hit_y = self.sprite_y(k).wrapping_add(O[d * 4].1 as i16 as u16);
        let overlaps = self.runtime.check_player_hitbox_overlap(
            self.ram,
            hit_x,
            hit_y,
            WH[d * 2],
            WH[d * 2 + 1],
        );
        if overlaps {
            let old_y = self.sprite_y(k);
            self.set_sprite_y(k, old_y.wrapping_add(19));
            let direction = self.runtime.direction_to_face_link(self.ram, k);
            self.set_sprite_y(k, old_y);
            if direction == 0 && self.ram.read_u8(0x2f) == 4 {
                self.ram.set_sprite_scratch_c(k, 1);
            }
        } else if !self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            return;
        }
        self.runtime.nullify_hookshot_drag(self.ram);
        self.ram.write_u8(0x5e, 0);
        self.runtime.repel_dash(self.ram);
    }

    pub fn sprite_01_vulture_bounce(&mut self, k: usize) {
        const GRAPHICS: [u8; 4] = [1, 2, 3, 2];
        self.ram
            .set_sprite_object_priority(k, self.ram.sprite(k).object_priority() | 0x30);
        self.runtime.vulture_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.runtime.move_xy(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                let timer = self.ram.sprite(k).subtype2().wrapping_add(1);
                self.ram.set_sprite_subtype2(k, timer);
                if timer == 160 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.runtime.sfx3_with_pan(self.ram, k, 0x1e);
                    self.ram.set_sprite_main_delay(k, 16);
                }
            }
            1 => {
                let frame = self.ram.read_u8(0x1a);
                self.ram
                    .set_sprite_graphics(k, GRAPHICS[usize::from((frame >> 1) & 3)]);
                if self.ram.sprite(k).main_delay() != 0 {
                    self.ram
                        .set_sprite_z(k, self.ram.sprite(k).z().wrapping_add(1));
                    return;
                }
                if (k as u8 ^ frame) & 1 != 0 {
                    return;
                }
                let (x_velocity, y_velocity, x_difference, y_difference) = self
                    .runtime
                    .project_speed_towards_link(self.ram, k, (k as u8 & 0x0f) + 24);
                self.ram
                    .set_sprite_velocity_x(k, y_velocity.wrapping_neg() as u8);
                self.ram.set_sprite_velocity_y(k, x_velocity as u8);
                if (x_difference.wrapping_add(0x28) as u8) < 0x50
                    && (y_difference.wrapping_add(0x28) as u8) < 0x50
                {
                    return;
                }
                self.ram.set_sprite_velocity_y(
                    k,
                    self.ram
                        .sprite(k)
                        .velocity_y()
                        .wrapping_add((y_velocity >> 2) as u8),
                );
                self.ram.set_sprite_velocity_x(
                    k,
                    self.ram
                        .sprite(k)
                        .velocity_x()
                        .wrapping_add((x_velocity >> 2) as u8),
                );
            }
            _ => {}
        }
    }

    pub fn sprite_20_sluggula(&mut self, k: usize) {
        const GRAPHICS: [u8; 8] = [0, 1, 0, 1, 2, 3, 4, 5];
        const OAM_FLAGS: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0, 0, 0];
        const VELOCITY: [i8; 6] = [16, -16, 0, 0, 16, -16];
        let j = usize::from(
            (self.ram.sprite(k).scratch_d() << 1) | ((self.ram.sprite(k).subtype2() & 8) >> 3),
        );
        self.ram.set_sprite_graphics(k, GRAPHICS[j]);
        self.ram
            .set_sprite_oam_flags(k, (self.ram.sprite(k).oam_flags() & 191) | OAM_FLAGS[j]);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        match self.ram.sprite(k).ai_state() {
            0 => {
                let delay = self.ram.sprite(k).main_delay();
                if delay == 0 {
                    let delay = (self.runtime.random_number(self.ram) & 31).wrapping_add(32);
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_main_delay(k, delay);
                    self.sluggula_set_velocity(k, delay & 3, &VELOCITY);
                } else if delay == 16 && self.runtime.random_number(self.ram) & 1 == 0 {
                    self.sluggula_drop_bomb(k);
                }
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    self.ram.set_sprite_main_delay(k, 32);
                }
                self.runtime.move_xy(self.ram, k);
                if self.runtime.check_tile_collision(self.ram, k) {
                    let direction = self.ram.sprite(k).scratch_d() ^ 1;
                    self.sluggula_set_velocity(k, direction, &VELOCITY);
                }
            }
            _ => {}
        }
    }

    fn sluggula_set_velocity(&mut self, k: usize, direction: u8, velocity: &[i8; 6]) {
        self.ram.set_sprite_scratch_d(k, direction);
        let j = usize::from(direction);
        self.ram.set_sprite_velocity_x(k, velocity[j] as u8);
        self.ram.set_sprite_velocity_y(k, velocity[j + 2] as u8);
    }

    pub fn sluggula_drop_bomb(&mut self, k: usize) {
        if let Some(j) = self.runtime.spawn_dynamic_ex(self.ram, k, 0x4a, 11) {
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.sprite_transmute_to_bomb(j);
        }
    }

    pub fn sprite_transmute_to_bomb(&mut self, k: usize) {
        self.ram.set_sprite_kind(k, 0x4a);
        self.ram.set_sprite_scratch_c(k, 1);
        self.ram.set_sprite_aux1_delay(k, 255);
        self.ram.set_sprite_flags3(k, 0x18);
        self.ram.set_sprite_oam_flags(k, 8);
        self.ram.set_sprite_health(k, 0);
    }

    pub fn sprite_18_mini_moldorm(&mut self, k: usize) {
        const X_VELOCITY: [i8; 16] = [
            24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 9, 17, 22,
        ];
        const Y_VELOCITY: [i8; 16] = [
            0, 9, 17, 22, 24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9,
        ];
        const NEXT_DIRECTION: [u8; 16] = [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7];
        self.runtime.moldorm_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).scratch_f() != 0 {
            self.sprite_prep_mini_moldorm_bounce(k);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        let direction = usize::from(self.ram.sprite(k).scratch_d());
        self.ram
            .set_sprite_velocity_x(k, X_VELOCITY[direction] as u8);
        self.ram
            .set_sprite_velocity_y(k, Y_VELOCITY[direction] as u8);
        self.runtime.move_xy(self.ram, k);
        if self.runtime.check_tile_collision(self.ram, k) {
            if self.runtime.random_number(self.ram) & 1 != 0 {
                self.ram.set_sprite_head_direction(
                    k,
                    (self.ram.sprite(k).head_direction() as i8).wrapping_neg() as u8,
                );
            }
            self.ram.set_sprite_scratch_d(k, NEXT_DIRECTION[direction]);
        }
        match self.ram.sprite(k).ai_state() {
            0 if self.ram.sprite(k).main_delay() == 0 => {
                let g = self.ram.sprite(k).scratch_g().wrapping_add(1);
                if g == 6 {
                    self.ram.set_sprite_scratch_g(k, 0);
                    self.ram.set_sprite_ai_state(k, 2);
                } else {
                    self.ram.set_sprite_scratch_g(k, g);
                    self.ram.set_sprite_ai_state(k, 1);
                }
                let head = (self.runtime.random_number(self.ram) & 2).wrapping_sub(1);
                self.ram.set_sprite_head_direction(k, head);
                let delay = (self.runtime.random_number(self.ram) & 0x1f).wrapping_add(0x20);
                self.ram.set_sprite_main_delay(k, delay);
            }
            1 => {
                let delay = self.ram.sprite(k).main_delay();
                if delay == 0 {
                    let next_delay = (self.runtime.random_number(self.ram) & 15).wrapping_add(8);
                    self.ram.set_sprite_main_delay(k, next_delay);
                    self.ram.set_sprite_ai_state(k, 0);
                } else if delay & 3 == 0 {
                    self.ram.set_sprite_scratch_d(
                        k,
                        self.ram
                            .sprite(k)
                            .scratch_d()
                            .wrapping_add(self.ram.sprite(k).head_direction())
                            & 0x0f,
                    );
                }
            }
            2 if ((k as u8) ^ self.ram.read_u8(0x1a)) & 3 == 0 => {
                self.runtime.apply_speed_towards_link(self.ram, k, 31);
                let angle = self.runtime.convert_velocity_to_angle(
                    self.ram.sprite(k).velocity_x(),
                    self.ram.sprite(k).velocity_y(),
                );
                let delta = angle.wrapping_sub(self.ram.sprite(k).scratch_d());
                if delta == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    self.ram.set_sprite_main_delay(k, 48);
                } else {
                    let step = if (delta as i8) < 0 { 0xff } else { 1 };
                    self.ram.set_sprite_scratch_d(
                        k,
                        self.ram.sprite(k).scratch_d().wrapping_add(step) & 0x0f,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn sprite_09_giant_moldorm(&mut self, k: usize) {
        const X_VELOCITY: [i8; 32] = [
            24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 9, 17, 22, 36, 33, 25, 13, 0,
            -13, -25, -33, -36, -33, -25, -13, 0, 13, 25, 33,
        ];
        const Y_VELOCITY: [i8; 32] = [
            0, 9, 17, 22, 24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 13, 25, 33, 36, 33,
            25, 13, 0, -13, -25, -33, -36, -33, -25, -13,
        ];
        const NEXT_DIRECTION: [u8; 16] = [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7];
        self.runtime.giant_moldorm_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).ai_state() == 3 {
            let death_delay = self.ram.sprite(k).aux4_delay();
            if death_delay == 0 {
                self.ram.set_sprite_state(k, 4);
                self.ram.set_sprite_scratch_a(k, 0);
                self.ram.set_sprite_main_delay(k, 224);
            } else {
                self.ram.set_sprite_hit_timer(k, death_delay | 224);
            }
            return;
        }
        self.runtime.check_damage_from_link(self.ram, k);
        let low_health = self.ram.sprite(k).health() < 3;
        self.ram.set_sprite_subtype2(
            k,
            self.ram
                .sprite(k)
                .subtype2()
                .wrapping_add(if low_health { 2 } else { 1 }),
        );
        let frame = self.ram.read_u8(0x1a);
        if frame & if low_health { 3 } else { 7 } == 0 {
            self.runtime.sfx3_with_pan(self.ram, k, 0x31);
        }
        if self.ram.sprite(k).scratch_f() != 0 {
            self.ram.set_sprite_aux2_delay(k, 64);
            if frame & 3 == 0 {
                self.ram
                    .set_sprite_scratch_f(k, self.ram.sprite(k).scratch_f().wrapping_sub(1));
            }
            return;
        }
        if self.ram.read_u8(0x46) == 0 && self.runtime.check_damage_to_link(self.ram, k) {
            self.runtime.link_cancel_dash(self.ram);
            let (x_velocity, y_velocity, _, _) =
                self.runtime.project_speed_towards_link(self.ram, k, 0x28);
            self.ram.write_u8(0x27, y_velocity as u8);
            self.ram.write_u8(0x28, x_velocity as u8);
            self.ram.write_u8(0x46, 24);
            self.ram.set_sprite_aux1_delay(k, 48);
            let pan = self.runtime.calculate_sfx_pan(self.ram, k);
            self.ram.write_u8(
                0x12f,
                pan | if self.runtime.misc_bug_fixes_enabled() {
                    0x32
                } else {
                    0
                },
            );
        }
        let direction = usize::from(self.ram.sprite(k).scratch_d());
        let velocity_index = direction + usize::from(low_health) * 16;
        self.ram
            .set_sprite_velocity_x(k, X_VELOCITY[velocity_index] as u8);
        self.ram
            .set_sprite_velocity_y(k, Y_VELOCITY[velocity_index] as u8);
        self.runtime.move_xy(self.ram, k);
        if self.runtime.check_tile_collision(self.ram, k) {
            self.ram.set_sprite_scratch_d(k, NEXT_DIRECTION[direction]);
            self.runtime.sfx2_with_pan(self.ram, k, 0x21);
        }
        match self.ram.sprite(k).ai_state() {
            0 if self.ram.sprite(k).main_delay() == 0 => {
                let g = self.ram.sprite(k).scratch_g().wrapping_add(1);
                let next_state = if g == 3 {
                    self.ram.set_sprite_scratch_g(k, 0);
                    2
                } else {
                    self.ram.set_sprite_scratch_g(k, g);
                    1
                };
                self.ram.set_sprite_ai_state(k, next_state);
                let head = (self.runtime.random_number(self.ram) & 2).wrapping_sub(1);
                self.ram.set_sprite_head_direction(k, head);
                let delay = (self.runtime.random_number(self.ram) & 31).wrapping_add(32);
                self.ram.set_sprite_main_delay(k, delay);
            }
            1 => {
                let delay = self.ram.sprite(k).main_delay();
                if delay == 0 {
                    let delay = (self.runtime.random_number(self.ram) & 15).wrapping_add(8);
                    self.ram.set_sprite_main_delay(k, delay);
                    self.ram.set_sprite_ai_state(k, 0);
                } else if delay & 3 == 0 {
                    self.ram.set_sprite_scratch_d(
                        k,
                        self.ram
                            .sprite(k)
                            .scratch_d()
                            .wrapping_add(self.ram.sprite(k).head_direction())
                            & 0x0f,
                    );
                }
            }
            2 if ((k as u8) ^ frame) & 3 == 0 => {
                self.runtime.apply_speed_towards_link(self.ram, k, 0x1f);
                let angle = self.runtime.convert_velocity_to_angle(
                    self.ram.sprite(k).velocity_x(),
                    self.ram.sprite(k).velocity_y(),
                );
                let delta = angle.wrapping_sub(self.ram.sprite(k).scratch_d());
                if delta == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    self.ram.set_sprite_main_delay(k, 48);
                } else {
                    let step = if (delta as i8) < 0 { 0xff } else { 1 };
                    self.ram
                        .set_sprite_scratch_d(k, self.ram.sprite(k).scratch_d().wrapping_add(step));
                }
            }
            _ => {}
        }
    }

    pub fn sprite_27_deadrock(&mut self, k: usize) {
        const GRAPHICS: [u8; 9] = [0, 1, 0, 1, 2, 2, 3, 3, 4];
        const OAM_FLAGS: [u8; 9] = [0x40, 0x40, 0, 0, 0, 0x40, 0, 0x40, 0];
        const X_VELOCITY: [i8; 4] = [32, -32, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, 32, -32];
        let aux2 = self.ram.sprite(k).aux2_delay();
        let index = if if aux2 != 0 {
            aux2 & 4 != 0
        } else {
            self.ram.sprite(k).ai_state() != 2
        } {
            usize::from(self.ram.sprite(k).scratch_a())
        } else {
            8
        };
        self.ram.set_sprite_graphics(k, GRAPHICS[index]);
        self.ram.set_sprite_oam_flags(
            k,
            (self.ram.sprite(k).oam_flags() & !0x40) | OAM_FLAGS[index],
        );
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let f = self.ram.sprite(k).scratch_f();
        if f == 0
            && self.runtime.check_damage_from_link(self.ram, k) & 1 != 0
            && self.ram.read_u8(0x12e) == 0
        {
            self.runtime.sfx2_with_pan(self.ram, k, 0x0b);
        }
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            self.runtime.nullify_hookshot_drag(self.ram);
            self.runtime.repel_dash(self.ram);
        }
        if f == 14 {
            self.ram.set_sprite_ai_state(k, 2);
            self.ram.set_sprite_aux1_delay(k, 255);
            self.ram.set_sprite_aux2_delay(k, 64);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 if self.ram.sprite(k).main_delay() == 0 => {
                self.ram
                    .set_sprite_flags2(k, self.ram.sprite(k).flags2() & !0x80);
                self.ram
                    .set_sprite_deflection_bits(k, self.ram.sprite(k).deflection_bits() & !4);
                self.ram
                    .set_sprite_flags3(k, self.ram.sprite(k).flags3() & !0x40);
                self.ram.set_sprite_ai_state(k, 1);
                let delay = (self.runtime.random_number(self.ram) & 31).wrapping_add(32);
                self.ram.set_sprite_main_delay(k, delay);
                let b = self.ram.sprite(k).scratch_b().wrapping_add(1);
                let direction = if b == 4 {
                    self.ram.set_sprite_scratch_b(k, 0);
                    self.runtime.direction_to_face_link(self.ram, k)
                } else {
                    self.ram.set_sprite_scratch_b(k, b);
                    self.runtime.random_number(self.ram) & 3
                };
                self.deadrock_set_direction(k, direction, &X_VELOCITY, &Y_VELOCITY);
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    self.ram.set_sprite_main_delay(k, 32);
                } else {
                    self.runtime.move_xy(self.ram, k);
                    if self.runtime.check_tile_collision(self.ram, k) {
                        let direction = self.ram.sprite(k).scratch_d() ^ 1;
                        self.deadrock_set_direction(k, direction, &X_VELOCITY, &Y_VELOCITY);
                        // C jumps to the `set_dir` label in case 0, whose
                        // trailing `break` skips the walking-animation update.
                        return;
                    }
                    let subtype = self.ram.sprite(k).subtype2().wrapping_add(1);
                    self.ram.set_sprite_subtype2(k, subtype);
                    self.ram.set_sprite_scratch_a(
                        k,
                        (self.ram.sprite(k).scratch_d() << 1) | ((subtype >> 2) & 1),
                    );
                }
            }
            2 => {
                self.ram
                    .set_sprite_flags2(k, self.ram.sprite(k).flags2() | 0x80);
                self.ram
                    .set_sprite_deflection_bits(k, self.ram.sprite(k).deflection_bits() | 4);
                self.ram
                    .set_sprite_flags3(k, self.ram.sprite(k).flags3() | 0x40);
                if self.ram.read_u8(0x1a) & 1 == 0 {
                    let aux1 = self.ram.sprite(k).aux1_delay();
                    if aux1 == 0 {
                        self.ram.set_sprite_ai_state(k, 0);
                        self.ram.set_sprite_main_delay(k, 16);
                    } else if aux1 == 0x20 {
                        self.ram.set_sprite_aux2_delay(k, 0x40);
                    }
                } else {
                    self.ram
                        .set_sprite_aux1_delay(k, self.ram.sprite(k).aux1_delay().wrapping_add(1));
                }
            }
            _ => {}
        }
    }

    fn deadrock_set_direction(
        &mut self,
        k: usize,
        direction: u8,
        x_velocity: &[i8; 4],
        y_velocity: &[i8; 4],
    ) {
        self.ram.set_sprite_scratch_d(k, direction);
        let j = usize::from(direction);
        self.ram.set_sprite_velocity_x(k, x_velocity[j] as u8);
        self.ram.set_sprite_velocity_y(k, y_velocity[j] as u8);
    }

    pub fn sprite_19_poe(&mut self, k: usize) {
        const ACCELERATION: [i8; 4] = [1, -1, 2, -2];
        const Z_TARGET: [i8; 2] = [8, -8];
        const X_TARGET: [i8; 4] = [16, -16, 28, -28];
        const OAM_FLAGS: [u8; 2] = [0x40, 0];
        const Y_VELOCITY: [i8; 2] = [8, -8];
        let direction = usize::from(self.ram.sprite(k).velocity_x() >> 7);
        self.ram.set_sprite_scratch_d(k, direction as u8);
        self.ram.set_sprite_oam_flags(
            k,
            (self.ram.sprite(k).oam_flags() & !0x40) | OAM_FLAGS[direction],
        );
        if self.ram.sprite(k).scratch_e() == 0 {
            self.ram
                .set_sprite_object_priority(k, self.ram.sprite(k).object_priority() | 0x30);
        }
        self.poe_draw(k);
        self.ram
            .write_u16(0x90, self.ram.read_u16(0x90).wrapping_add(4));
        self.ram
            .write_u16(0x92, self.ram.read_u16(0x92).wrapping_add(1));
        self.ram
            .set_sprite_flags2(k, self.ram.sprite(k).flags2().wrapping_sub(1));
        self.runtime.draw_single_large(self.ram, k);
        self.ram
            .set_sprite_flags2(k, self.ram.sprite(k).flags2().wrapping_add(1));
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        if self.ram.sprite(k).scratch_e() != 0 {
            let z = self.ram.sprite(k).z().wrapping_add(1);
            self.ram.set_sprite_z(k, z);
            if z == 12 {
                self.ram.set_sprite_scratch_e(k, 0);
            }
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        self.runtime.move_xy(self.ram, k);
        let frame = self.ram.read_u8(0x1a);
        if frame & 1 == 0 {
            let phase = usize::from(self.ram.sprite(k).scratch_g() & 1);
            let z_velocity = self
                .ram
                .sprite(k)
                .velocity_z()
                .wrapping_add(ACCELERATION[phase] as u8);
            self.ram.set_sprite_velocity_z(k, z_velocity);
            if z_velocity == Z_TARGET[phase] as u8 {
                self.ram
                    .set_sprite_scratch_g(k, self.ram.sprite(k).scratch_g().wrapping_add(1));
            }
        }
        self.runtime.move_z(self.ram, k);
        self.ram.set_sprite_velocity_y(k, 0);
        match self.ram.sprite(k).ai_state() {
            0 if self.ram.sprite(k).main_delay() == 0 => {
                self.ram.set_sprite_ai_state(k, 1);
                let head = if self.runtime.random_number(self.ram) & 12 == 0 {
                    u8::from(self.runtime.is_below_link(self.ram, k))
                } else {
                    self.runtime.random_number(self.ram) & 1
                };
                self.ram.set_sprite_head_direction(k, head);
            }
            1 => {
                if frame & 1 == 0 {
                    let j = usize::from(
                        (self.ram.sprite(k).animation_clock() & 1)
                            + if self.ram.read_u8(0xfff) != 0 { 2 } else { 0 },
                    );
                    let x_velocity = self
                        .ram
                        .sprite(k)
                        .velocity_x()
                        .wrapping_add(ACCELERATION[j] as u8);
                    self.ram.set_sprite_velocity_x(k, x_velocity);
                    if x_velocity == X_TARGET[j] as u8 {
                        self.ram.set_sprite_animation_clock(
                            k,
                            self.ram.sprite(k).animation_clock().wrapping_add(1),
                        );
                        self.ram.set_sprite_ai_state(k, 0);
                        let delay = (self.runtime.random_number(self.ram) & 31).wrapping_add(16);
                        self.ram.set_sprite_main_delay(k, delay);
                    }
                }
                let head = usize::from(self.ram.sprite(k).head_direction());
                self.ram.set_sprite_velocity_y(k, Y_VELOCITY[head] as u8);
            }
            _ => {}
        }
    }

    pub fn poe_draw(&mut self, k: usize) {
        const X: [i8; 2] = [9, -1];
        const CHARACTER: [u8; 4] = [0x7c, 0x80, 0xb7, 0x80];
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags_wide(self.ram, k) else {
            return;
        };
        let direction = usize::from(self.ram.sprite(k).scratch_d());
        let character = CHARACTER[usize::from((self.ram.sprite(k).subtype2() >> 3) & 3)];
        let oam_ptr = self.ram.read_u16(0x90);
        let oam_ext_ptr = self.ram.read_u16(0x92);
        self.runtime.emit_oam_wide(
            self.ram,
            k,
            x.wrapping_add_signed(i16::from(X[direction])),
            y.wrapping_add(9),
            character,
            (flags & 0xf0) | 2,
        );
        // C's `Poe_Draw` writes through `GetOamCurPtr()` without advancing
        // either global cursor; the caller advances exactly one entry.
        self.ram.write_u16(0x90, oam_ptr);
        self.ram.write_u16(0x92, oam_ext_ptr);
    }

    pub fn moblin_materialize_spear(&mut self, k: usize) {
        const X: [i8; 4] = [11, -2, -3, 11];
        const Y: [i8; 4] = [-3, -3, 3, -11];
        const X_VELOCITY: [i8; 4] = [32, -32, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, 32, -32];
        let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x1b) else {
            return;
        };
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.ram.set_sprite_scratch_a(j, 3);
        let direction = self.ram.sprite(k).scratch_d();
        self.ram.set_sprite_scratch_d(j, direction);
        let i = usize::from(direction);
        let x = u16::from_le_bytes([self.ram.sprite(j).x_low(), self.ram.sprite(j).x_high()])
            .wrapping_add(X[i] as u16);
        let y = u16::from_le_bytes([self.ram.sprite(j).y_low(), self.ram.sprite(j).y_high()])
            .wrapping_add(Y[i] as u16);
        self.set_sprite_x(j, x);
        self.set_sprite_y(j, y);
        self.ram.set_sprite_velocity_x(j, X_VELOCITY[i] as u8);
        self.ram.set_sprite_velocity_y(j, Y_VELOCITY[i] as u8);
    }

    pub fn sprite_12_moblin(&mut self, k: usize) {
        const X_VELOCITY: [i8; 4] = [16, -16, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, 16, -16];
        const DELAY: [u8; 4] = [0x10, 0x20, 0x30, 0x40];
        const THROW_GRAPHICS: [u8; 8] = [11, 10, 8, 9, 7, 5, 0, 2];
        const DIRECTIONS: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
        const GRAPHICS: [u8; 4] = [6, 4, 0, 2];
        self.moblin_draw(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.runtime.move_xy(self.ram, k);
        self.runtime.check_tile_collision2(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 if self.ram.sprite(k).main_delay() == 0 => {
                let delay = DELAY[usize::from(self.runtime.random_number(self.ram) & 3)];
                self.ram.set_sprite_main_delay(k, delay);
                self.ram.set_sprite_ai_state(k, 1);
                let direction = self.ram.sprite(k).head_direction();
                self.ram.set_sprite_scratch_d(k, direction);
                let j = usize::from(direction);
                self.ram.set_sprite_velocity_x(k, X_VELOCITY[j] as u8);
                self.ram.set_sprite_velocity_y(k, Y_VELOCITY[j] as u8);
            }
            1 => {
                let direction = self.ram.sprite(k).scratch_d();
                self.ram.set_sprite_graphics(
                    k,
                    (self.ram.sprite(k).subtype2() & 1) + GRAPHICS[usize::from(direction)],
                );
                if self.ram.sprite(k).wall_collision() == 0 {
                    if self.ram.sprite(k).main_delay() != 0 {
                        let e = self.ram.sprite(k).scratch_e().wrapping_sub(1);
                        self.ram.set_sprite_scratch_e(k, e);
                        if (e as i8) < 0 {
                            self.ram.set_sprite_scratch_e(k, 11);
                            self.ram.set_sprite_subtype2(
                                k,
                                self.ram.sprite(k).subtype2().wrapping_add(1),
                            );
                        }
                        return;
                    }
                    if direction == self.runtime.direction_to_face_link(self.ram, k) {
                        self.ram.set_sprite_ai_state(k, 2);
                        self.ram.set_sprite_main_delay(k, 32);
                        self.zero_velocity(k);
                        self.ram.set_sprite_velocity_z(k, 0);
                        return;
                    }
                    self.ram.set_sprite_main_delay(k, 0x10);
                } else {
                    self.ram.set_sprite_main_delay(k, 0x0c);
                }
                let selector =
                    usize::from((direction << 1) | (self.runtime.random_number(self.ram) & 1));
                self.ram.set_sprite_head_direction(k, DIRECTIONS[selector]);
                self.ram.set_sprite_ai_state(k, 0);
                let c = self.ram.sprite(k).scratch_c().wrapping_add(1);
                if c == 4 {
                    self.ram.set_sprite_scratch_c(k, 0);
                    let face = self.runtime.direction_to_face_link(self.ram, k);
                    self.ram.set_sprite_head_direction(k, face);
                } else {
                    self.ram.set_sprite_scratch_c(k, c);
                }
                self.zero_velocity(k);
                self.ram.set_sprite_velocity_z(k, 0);
            }
            2 => {
                let mut j = self.ram.sprite(k).scratch_d();
                let delay = self.ram.sprite(k).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                }
                if delay < 16 {
                    if delay == 15 {
                        self.moblin_materialize_spear(k);
                        self.ram.set_sprite_aux1_delay(k, 32);
                    }
                    j = j.wrapping_add(4);
                }
                self.ram
                    .set_sprite_graphics(k, THROW_GRAPHICS[usize::from(j)]);
            }
            _ => {}
        }
    }

    fn zero_velocity(&mut self, k: usize) {
        self.ram.set_sprite_velocity_x(k, 0);
        self.ram.set_sprite_velocity_y(k, 0);
    }

    pub fn moblin_draw(&mut self, k: usize) {
        const OBJECT_OFFSET: [usize; 12] = [2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];
        const HEAD_CHARACTER: [u8; 4] = [0x88, 0x88, 0x86, 0x84];
        const HEAD_HFLIP: [u8; 4] = [0x40, 0, 0, 0];
        let graphics = usize::from(self.ram.sprite(k).graphics());
        let start = graphics * 4;
        self.runtime
            .draw_multiple(self.ram, k, &MOBLIN_DRAW[start..start + 4]);
        if self.ram.sprite(k).pause() != 0 {
            return;
        }
        if self.ram.sprite(k).aux1_delay() != 0 {
            for piece in 0..4 {
                self.runtime.hide_oam_piece_if_small(self.ram, piece);
            }
        }
        let direction = usize::from(self.ram.sprite(k).head_direction());
        self.runtime.set_oam_piece_char_and_hflip(
            self.ram,
            OBJECT_OFFSET[graphics],
            HEAD_CHARACTER[direction],
            HEAD_HFLIP[direction],
        );
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_0e_snapdragon(&mut self, k: usize) {
        const DELAY: [u8; 4] = [0x20, 0x30, 0x40, 0x50];
        const GRAPHICS: [u8; 4] = [4, 0, 6, 2];
        const X_VELOCITY: [i8; 8] = [8, -8, 8, -8, 16, -16, 16, -16];
        const Y_VELOCITY: [i8; 8] = [8, 8, -8, -8, 16, 16, -16, -16];
        let direction = usize::from(self.ram.sprite(k).scratch_d());
        self.ram
            .set_sprite_graphics(k, self.ram.sprite(k).scratch_b() + GRAPHICS[direction]);
        self.snapdragon_draw(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        self.ram.set_sprite_scratch_b(k, 0);
        match self.ram.sprite(k).ai_state() {
            0 => {
                let delay = self.ram.sprite(k).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    let delay_index = usize::from((self.runtime.random_number(self.ram) & 12) >> 2);
                    self.ram.set_sprite_main_delay(k, DELAY[delay_index]);
                    let a = self.ram.sprite(k).scratch_a().wrapping_sub(1);
                    self.ram.set_sprite_scratch_a(k, a);
                    if (a as i8) < 0 {
                        self.ram.set_sprite_scratch_a(k, 3);
                        self.ram.set_sprite_main_delay(k, 96);
                        self.ram.set_sprite_scratch_c(
                            k,
                            self.ram.sprite(k).scratch_c().wrapping_add(1),
                        );
                        let direction = u8::from(self.runtime.is_below_link(self.ram, k)) * 2
                            + u8::from(self.runtime.is_right_of_link(self.ram, k));
                        self.ram.set_sprite_scratch_d(k, direction);
                    } else {
                        let direction = self.runtime.random_number(self.ram) & 3;
                        self.ram.set_sprite_scratch_d(k, direction);
                    }
                } else if delay & 0x18 != 0 {
                    self.ram
                        .set_sprite_scratch_b(k, self.ram.sprite(k).scratch_b().wrapping_add(1));
                }
            }
            1 => {
                self.ram
                    .set_sprite_scratch_b(k, self.ram.sprite(k).scratch_b().wrapping_add(1));
                self.runtime.move_xy(self.ram, k);
                if self.runtime.check_tile_collision(self.ram, k) {
                    self.ram
                        .set_sprite_scratch_d(k, self.ram.sprite(k).scratch_d() ^ 3);
                }
                let velocity_index = usize::from(
                    self.ram.sprite(k).scratch_d()
                        + if self.ram.sprite(k).scratch_c() != 0 {
                            4
                        } else {
                            0
                        },
                );
                self.ram
                    .set_sprite_velocity_x(k, X_VELOCITY[velocity_index] as u8);
                self.ram
                    .set_sprite_velocity_y(k, Y_VELOCITY[velocity_index] as u8);
                self.runtime.move_z(self.ram, k);
                self.ram
                    .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(4));
                if (self.ram.sprite(k).z() as i8) < 0 {
                    self.ram.set_sprite_z(k, 0);
                    if self.ram.sprite(k).main_delay() == 0 {
                        self.ram.set_sprite_ai_state(k, 0);
                        self.ram.set_sprite_scratch_c(k, 0);
                        self.ram.set_sprite_main_delay(k, 63);
                    } else {
                        self.ram.set_sprite_velocity_z(k, 20);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn snapdragon_draw(&mut self, k: usize) {
        let start = usize::from(self.ram.sprite(k).graphics()) * 4;
        self.runtime
            .draw_multiple(self.ram, k, &SNAPDRAGON_DRAW[start..start + 4]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_11_hinox(&mut self, k: usize) {
        const RANDOM_DIRECTIONS: [u8; 8] = [2, 3, 3, 2, 0, 1, 1, 0];
        const WALK_GRAPHICS: [u8; 4] = [6, 4, 0, 2];
        const BOMB_X: [i8; 4] = [8, -8, -13, 13];
        const BOMB_Y: [i8; 4] = [-11, -11, -16, -16];
        const BOMB_X_VELOCITY: [i8; 4] = [24, -24, 0, 0];
        const BOMB_Y_VELOCITY: [i8; 4] = [0, 0, 24, -24];
        const THROW_GRAPHICS: [u8; 8] = [11, 10, 8, 9, 7, 5, 1, 3];
        self.hinox_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).scratch_f() != 0 {
            self.hinox_face_link(k);
            self.ram.set_sprite_ai_state(k, 2);
            self.ram.set_sprite_main_delay(k, 48);
        }
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 if self.ram.sprite(k).main_delay() == 0 => {
                if self.runtime.random_number(self.ram) & 3 == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_main_delay(k, 64);
                } else {
                    let c = self.ram.sprite(k).scratch_c().wrapping_add(1);
                    if c == 4 {
                        self.ram.set_sprite_scratch_c(k, 0);
                        self.hinox_face_link(k);
                    } else {
                        self.ram.set_sprite_scratch_c(k, c);
                        let index = usize::from(
                            self.ram.sprite(k).scratch_d() * 2
                                + (self.runtime.random_number(self.ram) & 1),
                        );
                        self.hinox_set_direction(k, RANDOM_DIRECTIONS[index]);
                    }
                }
            }
            1 => {
                if self.ram.sprite(k).main_delay() != 0 {
                    let a = self.ram.sprite(k).scratch_a().wrapping_sub(1);
                    self.ram.set_sprite_scratch_a(k, a);
                    if (a as i8) < 0 {
                        self.ram.set_sprite_scratch_a(k, 11);
                        self.ram
                            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
                    }
                    self.runtime.move_xy(self.ram, k);
                    if !self.runtime.check_tile_collision(self.ram, k) {
                        let direction = usize::from(self.ram.sprite(k).scratch_d());
                        self.ram.set_sprite_graphics(
                            k,
                            WALK_GRAPHICS[direction] + (self.ram.sprite(k).subtype2() & 1),
                        );
                        return;
                    }
                }
                self.ram.set_sprite_main_delay(k, 16);
                self.ram.set_sprite_ai_state(k, 0);
            }
            2 => {
                let delay = self.ram.sprite(k).main_delay();
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    self.ram.set_sprite_main_delay(k, 2);
                    return;
                }
                let direction = usize::from(self.ram.sprite(k).scratch_d());
                if delay == 32 {
                    if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x4a) {
                        self.runtime.set_spawned_coordinates(self.ram, j);
                        self.sprite_transmute_to_bomb(j);
                        self.ram.set_sprite_aux1_delay(j, 64);
                        let x = u16::from_le_bytes([
                            self.ram.sprite(j).x_low(),
                            self.ram.sprite(j).x_high(),
                        ])
                        .wrapping_add(BOMB_X[direction] as u16);
                        let y = u16::from_le_bytes([
                            self.ram.sprite(j).y_low(),
                            self.ram.sprite(j).y_high(),
                        ])
                        .wrapping_add(BOMB_Y[direction] as u16);
                        self.set_sprite_x(j, x);
                        self.set_sprite_y(j, y);
                        self.ram
                            .set_sprite_velocity_x(j, BOMB_X_VELOCITY[direction] as u8);
                        self.ram
                            .set_sprite_velocity_y(j, BOMB_Y_VELOCITY[direction] as u8);
                        self.ram.set_sprite_velocity_z(j, 40);
                    }
                } else {
                    let index = direction + if delay < 32 { 4 } else { 0 };
                    self.ram.set_sprite_graphics(k, THROW_GRAPHICS[index]);
                }
            }
            _ => {}
        }
    }

    pub fn hinox_draw(&mut self, k: usize) {
        const COUNT: [usize; 12] = [4, 4, 4, 4, 3, 3, 3, 3, 5, 5, 4, 4];
        const OFFSET: [usize; 12] = [0, 4, 8, 12, 16, 19, 22, 25, 28, 33, 38, 42];
        let graphics = usize::from(self.ram.sprite(k).graphics());
        let start = OFFSET[graphics];
        self.runtime
            .draw_multiple(self.ram, k, &HINOX_DRAW[start..start + COUNT[graphics]]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_23_red_bari(&mut self, k: usize) {
        const XV: [i8; 16] = [
            0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
        ];
        const YV: [i8; 16] = [
            -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
        ];
        const G: [u8; 2] = [0, 3];
        let c = self.ram.sprite(k).scratch_c();
        if (c as i8) < 0 {
            let h = self.ram.sprite(k).head_direction();
            if h != 16 {
                self.ram.set_sprite_head_direction(k, h.wrapping_add(1));
            } else {
                self.ram.set_sprite_velocity_x(k, 255);
                self.ram.set_sprite_subtype(k, 255);
                self.runtime.check_tile_collision2(self.ram, k);
                self.ram.set_sprite_subtype(k, 0);
                let t = self.ram.read_u8(0xfa5);
                if t == 0 {
                    self.ram.set_sprite_scratch_c(k, 0);
                    self.ram.set_sprite_ignore_projectile(k, 0);
                    let delay = (self.runtime.random_number(self.ram) & 63).wrapping_add(128);
                    self.ram.set_sprite_aux1_delay(k, delay);
                } else {
                    self.ram.set_sprite_ignore_projectile(k, t);
                }
            }
            return;
        }
        if c != 0 {
            self.runtime.draw_single_small(self.ram, k);
        } else if self.ram.sprite(k).graphics() >= 2 {
            self.runtime.draw_single_large(self.ram, k);
        } else {
            self.red_bari_draw(k);
        }
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        let recoil_from_split = self.ram.sprite(k).aux2_delay() != 0;
        if !recoil_from_split && self.ram.sprite(k).ai_state() == 2 {
            self.ram.set_sprite_ignore_projectile(k, 2);
            self.ram
                .set_sprite_velocity_x(k, [8u8, 248][usize::from(self.ram.read_u8(0x1a) >> 1 & 1)]);
            self.runtime.move_x(self.ram, k);
            if self.ram.sprite(k).main_delay() == 0 {
                self.red_bari_split(k);
                self.ram.set_sprite_state(k, 0);
            }
            return;
        }
        if !recoil_from_split {
            self.runtime.check_damage_to_and_from_link(self.ram, k);
        }
        if !recoil_from_split && (k as u8 ^ self.ram.read_u8(0x1a)) & 15 == 0 {
            let a = self.ram.sprite(k).scratch_a().wrapping_add(
                if self.ram.sprite(k).scratch_b() & 1 != 0 {
                    255
                } else {
                    1
                },
            );
            self.ram.set_sprite_scratch_a(k, a);
            if self.runtime.random_number(self.ram) & 3 == 0 {
                self.ram
                    .set_sprite_scratch_b(k, self.ram.sprite(k).scratch_b().wrapping_add(1));
            }
        }
        if !recoil_from_split {
            let i = usize::from(self.ram.sprite(k).scratch_a() & 15);
            self.ram.set_sprite_velocity_x(k, XV[i] as u8);
            self.ram.set_sprite_velocity_y(k, YV[i] as u8);
        }
        if recoil_from_split
            || ((k as u8 ^ self.ram.read_u8(0x1a)) & 3) | self.ram.sprite(k).main_delay() == 0
        {
            if self.ram.sprite(k).wall_collision() == 0 {
                self.runtime.move_xy(self.ram, k);
            }
            self.runtime.check_tile_collision2(self.ram, k);
        }
        let c = usize::from(self.ram.sprite(k).scratch_c());
        self.ram
            .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 3 & 1) + G[c]);
        if self.ram.sprite(k).ai_state() != 0 {
            if self.ram.sprite(k).main_delay() != 0 {
                self.ram
                    .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 1 & 2) + G[c]);
                return;
            }
            self.ram.set_sprite_ai_state(k, 0);
        } else if self.ram.sprite(k).aux1_delay() != 0 {
            return;
        } else if self.runtime.random_number(self.ram) & 1 == 0 {
            self.ram.set_sprite_main_delay(k, 128);
            self.ram.set_sprite_ai_state(k, 1);
            return;
        }
        let d = (self.runtime.random_number(self.ram) & 63).wrapping_add(128);
        self.ram.set_sprite_aux1_delay(k, d);
    }

    pub fn red_bari_split(&mut self, k: usize) {
        const X: [i8; 2] = [0, 8];
        const X_VELOCITY: [i8; 2] = [-32, 32];
        for index in (0..=1).rev() {
            self.ram.write_u8(TMP_COUNTER, index as u8);
            let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x23) else {
                continue;
            };
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram.set_sprite_flags3(j, 0x33);
            self.ram.set_sprite_oam_flags(j, 3);
            self.ram.set_sprite_flags4(j, 1);
            self.ram.set_sprite_scratch_c(j, 1);
            let x = u16::from_le_bytes([self.ram.sprite(j).x_low(), self.ram.sprite(j).x_high()])
                .wrapping_add(X[index] as u16);
            self.set_sprite_x(j, x);
            self.ram.set_sprite_velocity_x(j, X_VELOCITY[index] as u8);
            self.ram.set_sprite_aux2_delay(j, 8);
            self.ram.set_sprite_aux1_delay(j, 64);
        }
        self.ram.write_u8(TMP_COUNTER, 0xff);
    }

    pub fn red_bari_draw(&mut self, k: usize) {
        const PIECES: [(i8, i8, u16, u8); 8] = [
            (0, 0, 0x0022, 0),
            (8, 0, 0x4022, 0),
            (0, 8, 0x0032, 0),
            (8, 8, 0x4032, 0),
            (0, 0, 0x0023, 0),
            (8, 0, 0x4023, 0),
            (0, 8, 0x0033, 0),
            (8, 8, 0x4033, 0),
        ];
        let start = usize::from(self.ram.sprite(k).graphics()) * 4;
        self.runtime
            .draw_multiple(self.ram, k, &PIECES[start..start + 4]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_13_mini_helmasaur(&mut self, k: usize) {
        const GRAPHICS: [u8; 8] = [3, 4, 3, 4, 2, 2, 5, 5];
        const OAM_FLAGS: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0x40, 0x40, 0];
        let mut index = usize::from(
            ((self.ram.sprite(k).subtype2() >> 2) & 1) | (self.ram.sprite(k).scratch_d() << 1),
        );
        self.ram.set_sprite_graphics(k, GRAPHICS[index]);
        self.ram.set_sprite_oam_flags(
            k,
            (self.ram.sprite(k).oam_flags() & !0x40) | OAM_FLAGS[index],
        );
        if ((k as u8) ^ self.ram.read_u8(0x1a)) & 15 == 0 {
            let x_velocity = self.ram.sprite(k).velocity_x();
            let y_velocity = self.ram.sprite(k).velocity_y();
            let abs_x = (x_velocity as i8).unsigned_abs();
            let abs_y = (y_velocity as i8).unsigned_abs();
            index = usize::from(if abs_x >= abs_y {
                x_velocity >> 7
            } else {
                (y_velocity >> 7) + 2
            });
            self.ram.set_sprite_scratch_d(k, index as u8);
        }
        self.runtime.draw_single_large(self.ram, k);
        self.helmasaur_hardhat_beetle_common(k);
    }

    pub fn sprite_26_hardhat_beetle(&mut self, k: usize) {
        self.ram
            .set_sprite_graphics(k, (self.ram.sprite(k).subtype2() >> 2) & 1);
        self.hardhat_beetle_draw(k);
        self.helmasaur_hardhat_beetle_common(k);
    }

    pub fn helmasaur_hardhat_beetle_common(&mut self, k: usize) {
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram
            .set_sprite_subtype2(k, self.ram.sprite(k).subtype2().wrapping_add(1));
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        let wall = self.ram.sprite(k).wall_collision();
        if wall & 15 != 0 {
            if wall & 3 != 0 {
                self.ram.set_sprite_velocity_x(k, 0);
            }
            self.ram.set_sprite_velocity_y(k, 0);
        } else {
            self.runtime.move_xy(self.ram, k);
        }
        self.runtime.check_tile_collision(self.ram, k);
        let frame = self.ram.read_u8(0x1a);
        if ((k as u8) ^ frame) & 31 == 0 {
            let (x_velocity, y_velocity, _, _) = self.runtime.project_speed_towards_link(
                self.ram,
                k,
                self.ram.sprite(k).scratch_a(),
            );
            self.ram.set_sprite_scratch_b(k, y_velocity as u8);
            self.ram.set_sprite_scratch_c(k, x_velocity as u8);
        }
        if ((k as u8) ^ frame) & self.ram.sprite(k).ai_state() != 0 {
            return;
        }
        let y = self.ram.sprite(k).velocity_y();
        let target_y = self.ram.sprite(k).scratch_b();
        let y_step = if (y.wrapping_sub(target_y) as i8) < 0 {
            1
        } else {
            0xff
        };
        self.ram.set_sprite_velocity_y(k, y.wrapping_add(y_step));
        let x = self.ram.sprite(k).velocity_x();
        let target_x = self.ram.sprite(k).scratch_c();
        let x_step = if (x.wrapping_sub(target_x) as i8) < 0 {
            1
        } else {
            0xff
        };
        self.ram.set_sprite_velocity_x(k, x.wrapping_add(x_step));
    }

    pub fn hardhat_beetle_draw(&mut self, k: usize) {
        const PIECES: [(i8, i8, u16, u8); 4] = [
            (0, -4, 0x0140, 2),
            (0, 2, 0x0142, 2),
            (0, -5, 0x0140, 2),
            (0, 2, 0x0144, 2),
        ];
        let start = usize::from(self.ram.sprite(k).graphics()) * 2;
        self.runtime
            .draw_multiple(self.ram, k, &PIECES[start..start + 2]);
        if self.ram.sprite(k).flags3() & 0x10 != 0 {
            self.runtime.draw_shadow(self.ram, k);
        }
    }

    pub fn sprite_15_antifairy(&mut self, k: usize) {
        self.runtime.antfairy_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.runtime.check_damage_to_link(self.ram, k) && self.ram.sprite(k).main_delay() == 0 {
            self.ram.set_sprite_main_delay(k, 16);
            let magic = self.ram.read_u8(0xf36e);
            if magic < 8 {
                self.ram.write_u8(0xf36e, 0);
            } else {
                self.ram.write_u8(0xf36e, magic - 8);
                self.ram.write_u8(0x12f, 0x1d);
            }
        }
        self.runtime.move_xy(self.ram, k);
        self.runtime.bounce_from_tile_collision(self.ram, k);
    }

    pub fn sprite_0b_cucco(&mut self, k: usize) {
        let x_velocity = self.ram.sprite(k).velocity_x();
        if x_velocity != 0 {
            self.ram.set_sprite_oam_flags(
                k,
                (self.ram.sprite(k).oam_flags() & !0x40)
                    | if (x_velocity as i8) < 0 { 0 } else { 0x40 },
            );
        }
        self.runtime.draw_single_large(self.ram, k);
        if self.ram.sprite(k).head_direction() != 0 {
            self.ram.set_sprite_kind(k, 0x3d);
            self.runtime.load_properties(self.ram, k);
            self.ram
                .set_sprite_subtype(k, self.ram.sprite(k).subtype().wrapping_add(1));
            self.ram.set_sprite_main_delay(k, 48);
            self.ram.write_u8(0x12e, 21);
            self.ram.set_sprite_ignore_projectile(k, 21);
            return;
        }
        if self.ram.sprite(k).state() == 10 {
            self.ram.set_sprite_ai_state(k, 3);
            if self.ram.read_u8(0x11) == 0 {
                self.runtime.chicken_incr_subtype2(self.ram, k, 3);
                self.cucco_draw_panic(k);
                if self.ram.read_u8(0x1a) & 0x0f == 0 {
                    self.bawk_bawk(k);
                }
            }
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).scratch_c() != 0 {
            self.ram
                .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() | 0x10);
            self.runtime.move_xy(self.ram, k);
            self.ram.set_sprite_z(k, 12);
            self.ram.set_sprite_ignore_projectile(k, 12);
            if ((k as u8) ^ self.ram.read_u8(0x1a)) & 7 == 0 {
                self.runtime.check_damage_to_link(self.ram, k);
            }
            self.runtime.chicken_incr_subtype2(self.ram, k, 4);
            return;
        }
        self.ram.set_sprite_health(k, 255);
        if self.ram.sprite(k).scratch_b() >= 35 {
            self.cucco_summon_avenger(k);
        }
        if self.ram.sprite(k).scratch_f() != 0 {
            self.ram.set_sprite_scratch_f(k, 0);
            if self.ram.sprite(k).scratch_b() < 35 {
                self.ram
                    .set_sprite_scratch_b(k, self.ram.sprite(k).scratch_b().wrapping_add(1));
                self.bawk_bawk(k);
            }
            self.ram.set_sprite_ai_state(k, 2);
        }
        self.runtime.check_damage_from_link(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => self.cucco_calm(k),
            1 => self.chicken_hopping(k),
            2 => self.cucco_flee(k),
            3 => self.cucco_carried(k),
            _ => {}
        }
    }

    pub fn cucco_calm(&mut self, k: usize) {
        const X_VELOCITY: [i8; 16] = [
            0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
        ];
        const Y_VELOCITY: [i8; 16] = [
            -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
        ];
        if self.ram.sprite(k).main_delay() == 0 {
            let j = usize::from(self.runtime.random_number(self.ram) & 0x0f);
            self.ram.set_sprite_velocity_x(k, X_VELOCITY[j] as u8);
            self.ram.set_sprite_velocity_y(k, Y_VELOCITY[j] as u8);
            let delay = (self.runtime.random_number(self.ram) & 0x1f).wrapping_add(0x10);
            self.ram.set_sprite_main_delay(k, delay);
            self.ram
                .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
        }
        self.ram.set_sprite_graphics(k, 0);
        self.runtime.return_if_lifted(self.ram, k);
    }

    pub fn chicken_hopping(&mut self, k: usize) {
        if ((k as u8) ^ self.ram.read_u8(0x1a)) & 1 != 0 && self.cucco_do_movement_xy(k) {
            self.ram.set_sprite_ai_state(k, 0);
        }
        self.runtime.move_z(self.ram, k);
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(2));
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            if self.ram.sprite(k).main_delay() == 0 {
                self.ram.set_sprite_main_delay(k, 32);
                self.ram.set_sprite_ai_state(k, 0);
            }
            self.ram.set_sprite_velocity_z(k, 10);
        }
        self.runtime.chicken_incr_subtype2(self.ram, k, 4);
    }

    pub fn cucco_flee(&mut self, k: usize) {
        self.runtime.return_if_lifted(self.ram, k);
        self.cucco_do_movement_xy(k);
        self.ram.set_sprite_z(k, 0);
        if ((k as u8) ^ self.ram.read_u8(0x1a)) & 0x1f == 0 {
            let (x_velocity, y_velocity, _, _) =
                self.runtime.project_speed_towards_link(self.ram, k, 16);
            self.ram
                .set_sprite_velocity_x(k, x_velocity.wrapping_neg() as u8);
            self.ram
                .set_sprite_velocity_y(k, y_velocity.wrapping_neg() as u8);
        }
        self.runtime.chicken_incr_subtype2(self.ram, k, 5);
        self.cucco_draw_panic(k);
    }

    pub fn cucco_draw_panic(&mut self, k: usize) {
        let Some((x, y)) = self.runtime.prep_oam_wide(self.ram, k) else {
            return;
        };
        let frame = self.ram.read_u8(0x1a);
        self.runtime.draw_distress(self.ram, x, y, frame);
    }

    pub fn cucco_carried(&mut self, k: usize) {
        self.runtime.move_z(self.ram, k);
        if self.cucco_do_movement_xy(k) {
            self.ram.set_sprite_velocity_x(
                k,
                (self.ram.sprite(k).velocity_x() as i8).wrapping_neg() as u8,
            );
            self.ram.set_sprite_velocity_y(
                k,
                (self.ram.sprite(k).velocity_y() as i8).wrapping_neg() as u8,
            );
            self.runtime.move_xy(self.ram, k);
            for _ in 0..2 {
                self.ram
                    .set_sprite_velocity_x(k, ((self.ram.sprite(k).velocity_x() as i8) >> 1) as u8);
                self.ram
                    .set_sprite_velocity_y(k, ((self.ram.sprite(k).velocity_y() as i8) >> 1) as u8);
            }
            self.bawk_bawk(k);
        }
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(1));
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            self.ram.set_sprite_ai_state(k, 2);
            let (x_velocity, y_velocity, _, _) =
                self.runtime.project_speed_towards_link(self.ram, k, 16);
            self.ram
                .set_sprite_velocity_x(k, x_velocity.wrapping_neg() as u8);
            self.ram
                .set_sprite_velocity_y(k, y_velocity.wrapping_neg() as u8);
            self.runtime.chicken_incr_subtype2(self.ram, k, 5);
            self.cucco_draw_panic(k);
        } else {
            self.runtime.chicken_incr_subtype2(self.ram, k, 4);
        }
    }

    pub fn cucco_do_movement_xy(&mut self, k: usize) -> bool {
        self.runtime.move_xy(self.ram, k);
        self.runtime.check_tile_collision(self.ram, k)
    }

    pub fn cucco_summon_avenger(&mut self, k: usize) {
        const OFFSET: [u8; 2] = [0, 0xff];
        let frame = self.ram.read_u8(0x1a);
        if ((k as u8) ^ frame) & 0x0f != 0 || self.ram.read_u8(0x1b) != 0 {
            return;
        }
        let Some(j) = self.runtime.spawn_dynamic_ex(self.ram, k, 0x0b, 10) else {
            return;
        };
        self.runtime.sfx3_with_pan(self.ram, j, 0x1e);
        self.ram.set_sprite_scratch_c(j, 1);
        let random = self.runtime.random_number(self.ram);
        let mut x = self.ram.read_u16(0xe2);
        let mut y = self.ram.read_u16(0xe8);
        if random & 2 != 0 {
            x = x.wrapping_add(random as u16);
            y = y.wrapping_add(OFFSET[usize::from(random & 1)] as u16);
        } else {
            y = y.wrapping_add(random as u16);
            x = x.wrapping_add(OFFSET[usize::from(random & 1)] as u16);
        }
        self.set_sprite_x(j, x);
        self.set_sprite_y(j, y);
        self.runtime.apply_speed_towards_link(self.ram, j, 32);
        self.bawk_bawk(k);
    }

    pub fn bawk_bawk(&mut self, k: usize) {
        self.runtime.sfx2_with_pan(self.ram, k, 0x30);
    }

    pub fn sprite_17_hoarder(&mut self, k: usize) {
        if self.ram.sprite(k).ai_state() != 0 {
            self.sprite_hoarder_frantic(k);
        } else {
            self.sprite_hoarder_covered(k);
        }
    }

    pub fn sprite_hoarder_covered(&mut self, k: usize) {
        const GRAPHICS: [u8; 4] = [3, 4, 5, 4];
        const X_VELOCITY: [i8; 4] = [-12, 12, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, -12, 12];
        self.covered_rupee_crab_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.set_sprite_graphics(k, 0);
        let (direction, x_difference, y_difference) = self
            .runtime
            .direction_to_face_link_with_differences(self.ram, k);
        let mut approach = self.ram.sprite(k).main_delay() != 0;
        if !approach
            && (y_difference.wrapping_add(0x30) as u8) < 0x60
            && (x_difference.wrapping_add(0x20) as u8) < 0x40
        {
            self.ram.set_sprite_main_delay(k, 32);
            approach = true;
        }
        if approach {
            let j = usize::from(direction);
            self.ram.set_sprite_velocity_x(k, X_VELOCITY[j] as u8);
            self.ram.set_sprite_velocity_y(k, Y_VELOCITY[j] as u8);
            if self.ram.sprite(k).wall_collision() == 0 {
                self.runtime.move_xy(self.ram, k);
            }
            self.runtime.check_tile_collision2(self.ram, k);
            self.runtime.check_damage_from_link(self.ram, k);
            let subtype = self.ram.sprite(k).subtype2().wrapping_add(1);
            self.ram.set_sprite_subtype2(k, subtype);
            self.ram
                .set_sprite_graphics(k, GRAPHICS[usize::from((subtype >> 1) & 3)]);
        }
        let kind = self.ram.sprite(k).kind();
        if kind != 0x3e || self.ram.read_u8(0xf354) >= 1 {
            self.runtime.return_if_lifted_permissive(self.ram, k);
        }
        if self.ram.sprite(k).state() != 9 {
            self.ram
                .set_sprite_scratch_c(k, if kind == 0x17 { 2 } else { 1 });
            self.ram.set_sprite_kind(k, 0xec);
            self.ram
                .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() & !1);
            self.ram.set_sprite_graphics(k, 0);
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x3e) {
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.ram
                    .set_sprite_flags2(j, self.ram.sprite(j).flags2() & !0x80);
                self.ram.set_sprite_aux2_delay(j, 128);
                self.ram.set_sprite_oam_flags(j, 9);
                self.ram.set_sprite_ai_state(j, 9);
            }
        }
    }

    pub fn sprite_hoarder_frantic(&mut self, k: usize) {
        const GRAPHICS: [u8; 4] = [0, 1, 0, 1];
        const OAM_FLAGS: [u8; 4] = [0, 0, 0x40, 0];
        const X_VELOCITY: [i8; 4] = [-16, 16, -16, 16];
        const Y_VELOCITY: [i8; 4] = [-16, -16, 16, 16];
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_from_link(self.ram, k);
        if self.ram.sprite(k).aux2_delay() == 0 {
            self.runtime.check_damage_to_link(self.ram, k);
        }
        let subtype = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, subtype);
        let mut j = usize::from((subtype >> 1) & 3);
        self.ram.set_sprite_graphics(k, GRAPHICS[j]);
        self.ram
            .set_sprite_oam_flags(k, (self.ram.sprite(k).oam_flags() & !0x40) | OAM_FLAGS[j]);
        if self.ram.sprite(k).wall_collision() != 0 {
            self.ram.set_sprite_aux4_delay(k, 16);
            j = usize::from(self.runtime.random_number(self.ram) & 3);
            self.ram.set_sprite_velocity_x(k, X_VELOCITY[j] as u8);
            self.ram.set_sprite_velocity_y(k, Y_VELOCITY[j] as u8);
        } else {
            self.runtime.move_xy(self.ram, k);
        }
        self.runtime.check_tile_collision2(self.ram, k);
        let frame = self.ram.read_u8(0x1a);
        if self.ram.sprite(k).aux4_delay() == 0 && ((k as u8) ^ frame) & 31 == 0 {
            let (x_velocity, y_velocity, _, _) =
                self.runtime.project_speed_towards_link(self.ram, k, 16);
            self.ram
                .set_sprite_velocity_y(k, y_velocity.wrapping_neg() as u8);
            self.ram
                .set_sprite_velocity_x(k, x_velocity.wrapping_neg() as u8);
        }
        if frame & 1 != 0 {
            return;
        }
        let g = self.ram.sprite(k).scratch_g().wrapping_add(1);
        self.ram.set_sprite_scratch_g(k, g);
        let (end, kind) = if g == 192 {
            self.ram.set_sprite_main_delay(k, 15);
            self.ram.set_sprite_state(k, 6);
            self.ram
                .set_sprite_flags2(k, self.ram.sprite(k).flags2().wrapping_add(4));
            (1usize, 0xd9)
        } else {
            if g & 15 != 0 {
                return;
            }
            (
                0,
                if self.ram.sprite(k).head_direction() == 6 {
                    0xdb
                } else {
                    0xd9
                },
            )
        };
        if let Some(spawned) = self.runtime.spawn_dynamic_ex(self.ram, k, kind, end) {
            self.ram
                .set_sprite_head_direction(k, self.ram.sprite(k).head_direction().wrapping_add(1));
            self.runtime.set_spawned_coordinates(self.ram, spawned);
            self.ram
                .set_sprite_x_low(spawned, self.ram.sprite(spawned).x_low().wrapping_add(8));
            self.ram.set_sprite_velocity_z(spawned, 32);
            self.ram.set_sprite_aux4_delay(spawned, 16);
            let (x_velocity, y_velocity, _, _) = self
                .runtime
                .project_speed_towards_link(self.ram, spawned, 16);
            self.ram.set_sprite_velocity_y(spawned, !y_velocity as u8);
            self.ram.set_sprite_velocity_x(spawned, !x_velocity as u8);
            self.runtime.sfx3_with_pan(self.ram, k, 0x30);
        }
    }

    pub fn covered_rupee_crab_draw(&mut self, k: usize) {
        const Y: [i8; 12] = [0, 0, 0, -3, 0, -5, 0, -6, 0, -6, 0, -6];
        const CHARACTER: [u8; 12] = [
            0x44, 0x44, 0xe8, 0x44, 0xe8, 0x44, 0xe6, 0x44, 0xe8, 0x44, 0xe6, 0x44,
        ];
        const FLAGS: [u8; 12] = [0, 0xc, 3, 0xc, 3, 0xc, 3, 0xc, 3, 0xc, 0x43, 0xc];
        let Some((x, y, base_flags)) = self.runtime.prep_oam_with_flags_wide(self.ram, k) else {
            return;
        };
        if self.ram.read_u8(0xfc6) >= 3 {
            return;
        }
        let character_offset = if self.ram.sprite(k).kind() == 0x17 {
            2
        } else {
            0
        };
        let graphics_offset = usize::from(self.ram.sprite(k).graphics()) * 2;
        let oam_base = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for (piece, i) in (0..=1).rev().enumerate() {
            let index = i + graphics_offset;
            let character = CHARACTER[index];
            crate::oam::set_helper0(
                self.ram,
                oam_base + piece,
                x,
                y.wrapping_add(Y[index] as u16),
                character.wrapping_add(if character == 0x44 {
                    character_offset
                } else {
                    0
                }),
                (base_flags & !1) | FLAGS[index],
                2,
            );
        }
    }

    pub fn sprite_ec_thrown_item(&mut self, k: usize) {
        const CHARACTER: [u8; 12] = [
            0x42, 0x44, 0x46, 0, 0x46, 0x44, 0x42, 0x44, 0x44, 0, 0x46, 0x44,
        ];
        const FLAGS: [u8; 9] = [0x0c, 0x0c, 0x0c, 0, 0, 0, 0xb0, 0x08, 0xb4];
        if self.ram.read_u8(0xfc6) < 3 {
            if self.ram.read_u8(0xfb3) != 0 && self.ram.sprite(k).floor() != 0 {
                let slot = 0x2c + (k & 3);
                self.ram.write_u16(0x90, (0x0800 + slot * 4) as u16);
                self.ram.write_u16(0x92, (0x0a20 + slot) as u16);
            }
            let state = self.ram.sprite(k).state();
            self.ram.set_sprite_ignore_projectile(k, state);
            let scenery = usize::from(self.ram.sprite(k).scratch_c());
            if scenery >= 6 {
                self.sprite_draw_thrown_item_gigantic(k);
            } else {
                self.runtime.draw_single_large(self.ram, k);
                let world =
                    u8::from(self.ram.read_u8(0x1b) != 0) + u8::from(self.ram.read_u8(0xfff) != 0);
                let character = CHARACTER[scenery + if world >= 2 { 6 } else { 0 }];
                self.runtime.set_oam_piece_character_and_low_flags(
                    self.ram,
                    0,
                    character,
                    FLAGS[scenery],
                    0x0f,
                );
                self.ram.set_sprite_oam_flags(
                    k,
                    (self.ram.sprite(k).oam_flags() & 0xc0) | (FLAGS[scenery] & 0x0f),
                );
            }
        }
        if self.ram.sprite(k).state() == 9 {
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.runtime.throwable_scenery_interact(self.ram, k);
        }
    }

    pub fn sprite_draw_thrown_item_gigantic(&mut self, k: usize) {
        const X: [i8; 4] = [-8, 8, -8, 8];
        const Y: [i8; 4] = [-14, -14, 2, 2];
        const FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];
        const X2: [i8; 3] = [-6, 0, 6];
        const OAM_FLAGS: [u8; 2] = [0x0c, 0];
        let scenery = usize::from(self.ram.sprite(k).scratch_c() - 6);
        self.ram.set_sprite_oam_flags(k, OAM_FLAGS[scenery]);
        let Some((x, y, base_flags)) = self.runtime.prep_oam_with_flags_wide(self.ram, k) else {
            return;
        };
        let oam_base = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for i in (0..4).rev() {
            crate::oam::set_helper0(
                self.ram,
                oam_base + (3 - i),
                x.wrapping_add_signed(i16::from(X[i])),
                y.wrapping_add_signed(i16::from(Y[i])),
                0x4a,
                FLAGS[i] | base_flags,
                2,
            );
        }
        self.runtime.oam_allocate_region_b(self.ram, 12);
        let world_y = u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()]);
        let screen_y = world_y.wrapping_sub(self.ram.read_u16(0xe8));
        let oam_base = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for i in (0..3).rev() {
            crate::oam::set_helper0(
                self.ram,
                oam_base + (2 - i),
                x.wrapping_add_signed(i16::from(X2[i])),
                screen_y.wrapping_add(12),
                0x6c,
                0x24,
                2,
            );
        }
    }

    pub fn throwable_scenery_scatter_into_debris(&mut self, k: usize) {
        const X: [i8; 4] = [-8, 8, -8, 8];
        const Y: [i8; 4] = [-8, -8, 8, 8];
        let scenery = self.ram.sprite(k).scratch_c();
        if (scenery as i8) >= 0 && scenery >= 6 {
            for i in (0..4).rev() {
                let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xec) else {
                    continue;
                };
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.ram.set_sprite_z(j, self.ram.sprite(k).z());
                let x =
                    u16::from_le_bytes([self.ram.sprite(j).x_low(), self.ram.sprite(j).x_high()])
                        .wrapping_add(X[i] as u16);
                let y =
                    u16::from_le_bytes([self.ram.sprite(j).y_low(), self.ram.sprite(j).y_high()])
                        .wrapping_add(Y[i] as u16);
                self.set_sprite_x(j, x);
                self.set_sprite_y(j, y);
                self.ram.set_sprite_scratch_c(j, 1);
                self.runtime.schedule_for_breakage(self.ram, j);
                self.ram
                    .set_sprite_oam_flags(j, if scenery < 7 { 12 } else { 0 });
            }
            self.ram.set_sprite_state(k, 0);
            return;
        }
        self.ram.set_sprite_state(k, 0);
        let Some((_, _, flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        let garnish = self.ram.garnish_find_free_force();
        self.ram.set_garnish_kind(garnish, 22);
        self.ram.set_garnish_active_marker(22);
        self.ram
            .set_garnish_x_low(garnish, self.ram.sprite(k).x_low());
        self.ram
            .set_garnish_x_high(garnish, self.ram.sprite(k).x_high());
        let y = u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()])
            .wrapping_sub(u16::from(self.ram.sprite(k).z()))
            .wrapping_add(0x10);
        let [ylo, yhi] = y.to_le_bytes();
        self.ram.set_garnish_y_low(garnish, ylo);
        self.ram.set_garnish_y_high(garnish, yhi);
        self.ram.set_garnish_oam_flags(garnish, flags);
        self.ram
            .set_garnish_floor(garnish, self.ram.sprite(k).floor());
        self.ram.set_garnish_countdown(garnish, 31);
        self.ram.set_garnish_source_sprite(garnish, scenery);
    }

    pub fn dark_world_hint_npc_idle(&mut self, k: usize) {
        if self.runtime.show_solicited_message(self.ram, k, 0xfe) & 0x100 != 0 {
            self.ram.set_sprite_ai_state(k, 1);
        }
    }

    pub fn sprite_28_dark_world_hint_npc(&mut self, k: usize) {
        self.storyteller_1_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        if self.ram.sprite(k).main_delay() == 0 {
            self.ram
                .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 4) & 1);
        }
        let subtype = self.ram.sprite(k).subtype2();
        if subtype <= 2 {
            self.dark_world_hint_npc_dialogue(k, [0xff, 0x101, 0x102][usize::from(subtype)]);
        } else if subtype == 3 {
            if self.ram.sprite(k).main_delay() == 0 {
                if self.ram.read_u8(0x1a) & 0x3f == 0 {
                    self.ram
                        .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() ^ 0x40);
                }
                if self.runtime.random_number(self.ram) == 0 {
                    self.ram.set_sprite_main_delay(k, 32);
                }
            }
            self.runtime.show_solicited_message(self.ram, k, 0x149);
        } else if subtype == 4 {
            self.ram
                .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 1) & 1);
            self.runtime.move_z(self.ram, k);
            if (self.ram.sprite(k).z() as i8) < 0 {
                self.ram.set_sprite_z(k, 0);
            }
            let step = if self.ram.sprite(k).z() >= 4 { 0xff } else { 1 };
            self.ram
                .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_add(step));
            self.dark_world_hint_npc_dialogue(k, 0x103);
        }
    }

    fn dark_world_hint_npc_dialogue(&mut self, k: usize, success_message: u16) {
        match self.ram.sprite(k).ai_state() {
            0 => self.dark_world_hint_npc_idle(k),
            1 => {
                if self.ram.read_u8(0x1ce8) == 0 && self.dark_world_hint_npc_handle_payment() {
                    self.runtime
                        .show_unconditional_message(self.ram, success_message);
                    self.ram.set_sprite_ai_state(k, 2);
                } else {
                    self.runtime.show_unconditional_message(self.ram, 0x100);
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            2 => self.dark_world_hint_npc_restore_health(k),
            _ => {}
        }
    }

    pub fn dark_world_hint_npc_restore_health(&mut self, k: usize) {
        self.ram.write_u8(0xf372, 0xa0);
        self.ram.set_sprite_ai_state(k, 0);
    }

    pub fn dark_world_hint_npc_handle_payment(&mut self) -> bool {
        let rupees = self.ram.read_u16(0xf360);
        if rupees < 20 {
            return false;
        }
        self.ram.write_u16(0xf360, rupees - 20);
        true
    }

    pub fn storyteller_1_draw(&mut self, k: usize) {
        const PIECES: [(i8, i8, u16, u8); 10] = [
            (0, 0, 0x0a4a, 2),
            (0, 0, 0x4a6e, 2),
            (0, 0, 0x0a24, 2),
            (0, 0, 0x4a24, 2),
            (0, 0, 0x0804, 2),
            (0, 0, 0x4804, 2),
            (0, 0, 0x0a6a, 2),
            (0, 0, 0x0a6c, 2),
            (0, 0, 0x0a0e, 2),
            (0, 0, 0x0a2e, 2),
        ];
        let index = usize::from(self.ram.sprite(k).subtype2() * 2 + self.ram.sprite(k).graphics());
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &PIECES[index..index + 1]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_2e_flute_kid(&mut self, k: usize) {
        match self.ram.sprite(k).head_direction() {
            0 => match self.ram.sprite(k).subtype2() {
                0 => self.flute_kid_human(k),
                1 => self.sprite_flute_kid_stumpy(k),
                _ => {}
            },
            1 => self.sprite_flute_kid_quaver(k),
            _ => {}
        }
    }

    pub fn flute_kid_human(&mut self, k: usize) {
        if self.ram.sprite(k).ai_state() != 3 {
            let offscreen = self.flute_boy_draw(k);
            self.ram.set_sprite_scratch_c(k, offscreen);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).scratch_c() == 0 && self.ram.sprite(k).scratch_b() == 0 {
            self.ram.write_u8(0x12d, 11);
            self.ram.set_sprite_scratch_b(k, 11);
        }
        self.ram
            .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 5) & 1);
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.read_u8(0xf34c) >= 2
                    || self.runtime.flute_boy_check_if_player_close(self.ram, k)
                {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram
                        .set_sprite_scratch_d(k, self.ram.sprite(k).scratch_d().wrapping_add(1));
                    self.ram
                        .write_u8(0xfdd, self.ram.read_u8(0xfdd).wrapping_add(1));
                    self.ram.set_sprite_main_delay(k, 176);
                    self.ram.write_u8(0x2e4, 1);
                }
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_main_delay(k, 25);
                    self.flute_kid_spawn_quaver(k);
                }
            }
            1 => {
                self.ram.write_u8(0x2e4, 1);
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.write_u8(0x1d, 2);
                    self.ram.write_u8(0x9a, 48);
                    self.ram.write_u8(0xc007, 0);
                    self.ram.write_u8(0xc009, 0);
                    self.runtime.palette_assert_translucency_swap(self.ram);
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.write_u8(0x12d, 128);
                    self.runtime.sfx2_with_pan(self.ram, k, 0x33);
                }
            }
            2 if self.ram.read_u8(0x1a) & 15 == 0 => {
                self.runtime.palette_filter_sp5f(self.ram);
                if self.ram.read_u8(0xc007) == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                }
            }
            3 => {
                self.runtime.palette_filter_restore_sp5f(self.ram);
                self.runtime.palette_revert_translucency_swap(self.ram);
                self.ram.set_sprite_state(k, 0);
                self.ram.write_u8(0x2e4, 0);
            }
            _ => {}
        }
    }

    pub fn sprite_flute_kid_stumpy(&mut self, k: usize) {
        const GRAPHICS: [i8; 20] = [1, 1, 1, 1, 2, 1, 2, 1, 2, 1, 2, 3, 2, 3, 2, 3, 2, 3, 2, -1];
        const DELAY: [i8; 19] = [
            -1, -1, -1, 16, 2, 12, 6, 8, 10, 4, 14, 2, 10, 6, 6, 10, 2, 14, 2,
        ];
        self.flute_aardvark_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => match self.ram.read_u8(0xf34c) & 3 {
                0 => {
                    if self.runtime.show_solicited_message(self.ram, k, 0xe5) & 0x100 != 0 {
                        self.ram.set_sprite_ai_state(k, 1);
                    }
                }
                1 => {
                    self.runtime.show_solicited_message(self.ram, k, 0xe8);
                }
                2 => {
                    self.ram.set_sprite_graphics(k, 1);
                    if self.runtime.show_solicited_message(self.ram, k, 0xe9) & 0x100 != 0 {
                        self.ram.set_sprite_ai_state(k, 3);
                    }
                }
                3 => self.ram.set_sprite_graphics(k, 3),
                _ => unreachable!(),
            },
            1 => {
                if self.ram.read_u8(0x1ce8) == 0 {
                    self.runtime.show_unconditional_message(self.ram, 0xe6);
                    self.ram.set_sprite_ai_state(k, 2);
                } else {
                    self.runtime.show_unconditional_message(self.ram, 0xe7);
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            2 => {
                self.ram.write_u8(0x2e9, 0);
                self.runtime.link_receive_item(self.ram, 0x13, 0);
                self.ram.set_sprite_ai_state(k, 0);
            }
            3 => {
                if self.ram.read_u8(0x202) == 13 && self.ram.read_u8(0xf0) & 0x40 != 0 {
                    self.ram.set_sprite_ai_state(k, 4);
                    self.ram.write_u8(0x12c, 0xf2);
                    self.ram.write_u8(0x12e, 0);
                    self.ram.write_u8(0x12d, 23);
                    self.ram
                        .write_u8(0x2e4, self.ram.read_u8(0x2e4).wrapping_add(1));
                }
            }
            4 if self.ram.sprite(k).main_delay() == 0 => {
                let a = self.ram.sprite(k).scratch_a();
                if a >= 3 {
                    self.runtime.sfx2_with_pan(self.ram, k, 0x33);
                }
                self.ram.set_sprite_scratch_a(k, a.wrapping_add(1));
                let graphic = GRAPHICS[usize::from(a)];
                if graphic >= 0 {
                    self.ram.set_sprite_graphics(k, graphic as u8);
                    self.ram
                        .set_sprite_main_delay(k, DELAY[usize::from(a)] as u8);
                } else {
                    self.ram.write_u8(0x12c, 0xf3);
                    self.ram.set_sprite_ai_state(k, 5);
                    self.ram.write_u8(0x2e4, 0);
                }
            }
            5 => {
                self.ram.set_sprite_graphics(k, 3);
                self.ram.write_u8(0xf3c9, self.ram.read_u8(0xf3c9) | 8);
            }
            _ => {}
        }
    }

    pub fn sprite_flute_kid_quaver(&mut self, k: usize) {
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.move_xy(self.ram, k);
        self.runtime.move_z(self.ram, k);
        if self.ram.sprite(k).main_delay() == 0 {
            self.ram.set_sprite_state(k, 0);
        }
        let frame = self.ram.read_u8(0x1a);
        if frame & 1 == 0 {
            let delta: u8 = if ((frame >> 5) ^ self.ram.read_u8(0xfa0)) & 1 != 0 {
                0xff
            } else {
                1
            };
            self.ram
                .set_sprite_velocity_x(k, self.ram.sprite(k).velocity_x().wrapping_add(delta));
        }
    }

    pub fn flute_kid_spawn_quaver(&mut self, k: usize) {
        let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x2e) else {
            return;
        };
        self.runtime.set_spawned_coordinates(self.ram, j);
        let x = u16::from_le_bytes([self.ram.sprite(j).x_low(), self.ram.sprite(j).x_high()])
            .wrapping_add(4);
        let y = u16::from_le_bytes([self.ram.sprite(j).y_low(), self.ram.sprite(j).y_high()])
            .wrapping_sub(4);
        self.set_sprite_x(j, x);
        self.set_sprite_y(j, y);
        self.ram.set_sprite_head_direction(j, 1);
        self.ram.set_sprite_velocity_z(j, 8);
        self.ram.set_sprite_main_delay(j, 96);
        self.ram.set_sprite_ignore_projectile(j, 96);
    }

    pub fn flute_boy_draw(&mut self, k: usize) -> u8 {
        self.runtime.oam_allocate_region_b(self.ram, 0x10);
        let start = usize::from(self.ram.sprite(k).scratch_d()) * 8
            + usize::from(self.ram.sprite(k).graphics()) * 4;
        self.runtime
            .draw_multiple_and_get_offscreen(self.ram, k, &FLUTE_BOY_DRAW[start..start + 4])
    }

    pub fn flute_aardvark_draw(&mut self, k: usize) {
        let start = usize::from(self.ram.sprite(k).graphics()) * 2;
        self.runtime.draw_multiple_player_deferred(
            self.ram,
            k,
            &FLUTE_AARDVARK_DRAW[start..start + 2],
        );
    }

    pub fn sprite_1a_smithy(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.smithy_main(k),
            1 => self.smithy_spark(k),
            2 => self.smithy_frog(k),
            3 => self.smithy_homecoming(k),
            _ => {}
        }
    }

    pub fn smithy_homecoming(&mut self, k: usize) {
        self.returning_smithy_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                const DELAY: [u8; 3] = [104, 12, 0];
                const DIR: [i8; 3] = [0, 2, -1];
                const XV: [i8; 4] = [0, 0, -13, 13];
                const YV: [i8; 4] = [-13, 13, 0, 0];
                self.runtime.move_xy(self.ram, k);
                self.ram
                    .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 3) & 1);
                if self.ram.sprite(k).main_delay() != 0 {
                    return;
                }
                let a = self.ram.sprite(k).scratch_a();
                self.ram.set_sprite_scratch_a(k, a.wrapping_add(1));
                self.ram.set_sprite_main_delay(k, DELAY[usize::from(a)]);
                let d = DIR[usize::from(a)];
                if d >= 0 {
                    self.ram.set_sprite_scratch_d(k, d as u8);
                    self.ram.set_sprite_velocity_x(k, XV[d as usize] as u8);
                    self.ram.set_sprite_velocity_y(k, YV[d as usize] as u8);
                } else {
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 => {
                self.runtime.behave_as_barrier(self.ram, k);
                self.runtime.show_solicited_message(self.ram, k, 0xe3);
                self.ram.write_u8(0x2e4, 0);
                self.ram.set_sprite_scratch_d(k, 1);
                self.ram.write_u8(0xf3c9, self.ram.read_u8(0xf3c9) | 32);
            }
            _ => {}
        }
    }

    pub fn smithy_frog(&mut self, k: usize) {
        self.smithy_frog_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(2));
        self.runtime.move_z(self.ram, k);
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            self.ram.set_sprite_velocity_z(k, 16);
        }
        if self.ram.sprite(k).ai_state() == 0 {
            self.ram.set_sprite_scratch_d(k, 1);
            if self.runtime.show_solicited_message(self.ram, k, 0xe1) & 0x100 != 0 {
                self.ram.set_sprite_ai_state(k, 1);
            }
        } else {
            self.ram.write_u8(0xf3cc, 7);
            self.runtime.load_follower_graphics(self.ram);
            self.runtime.become_follower(self.ram, k);
            self.ram.set_sprite_state(k, 0);
        }
    }

    pub fn returning_smithy_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 8] = [
            (0, 0, 0x4122, 2),
            (0, 0, 0x0122, 2),
            (0, 0, 0x4122, 2),
            (0, 0, 0x0122, 2),
            (0, 0, 0x0122, 2),
            (0, 0, 0x0122, 2),
            (0, 0, 0x4122, 2),
            (0, 0, 0x4122, 2),
        ];
        const DMA: [u8; 8] = [0xc0, 0xc0, 0xa0, 0xa0, 0x80, 0x60, 0x80, 0x60];
        let j = usize::from(self.ram.sprite(k).scratch_d() * 2 + self.ram.sprite(k).graphics());
        self.ram.write_u8(0xaea, DMA[j]);
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &P[j..j + 1]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn smithy_frog_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 1] = [(0, 0, 0x00c8, 2)];
        self.runtime.draw_multiple_player_deferred(self.ram, k, &P);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn smithy_main(&mut self, k: usize) {
        const GFX: [u8; 8] = [0, 1, 2, 3, 3, 2, 1, 0];
        const DELAY: [u8; 8] = [24, 4, 1, 16, 16, 5, 10, 16];
        self.smithy_draw(k);
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(2));
        self.runtime.move_z(self.ram, k);
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            self.ram.set_sprite_velocity_z(k, 0);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let partner = usize::from(self.ram.sprite(k).scratch_e());
        let ps = self.ram.sprite(partner).ai_state();
        let state = self.ram.sprite(k).ai_state();
        if ps == 5
            || ps == 7
            || ps == 9
            || state == 5
            || state == 7
            || state == 9
            || (state | ps) == 0
        {
            let old = self.ram.sprite(k).scratch_b();
            self.ram.set_sprite_scratch_b(k, old.wrapping_sub(1));
            if old == 0 {
                let a = self.ram.sprite(k).scratch_a();
                self.ram.set_sprite_scratch_a(k, a.wrapping_add(1) & 7);
                self.ram.set_sprite_graphics(k, GFX[usize::from(a)]);
                self.ram.set_sprite_scratch_b(k, DELAY[usize::from(a)]);
                if a == 1 {
                    self.ram.set_sprite_velocity_z(k, 16);
                }
                if a == 3 {
                    self.smithy_spawn_spark(k);
                    self.runtime.sfx2_with_pan(self.ram, k, 5);
                }
            }
        }
        match state {
            0 => {
                self.ram.set_sprite_scratch_c(k, 0);
                if self.ram.read_u8(0xf3cc) != 8 {
                    if self.smithy_listen_for_hammer(k) {
                        self.runtime.show_unconditional_message(self.ram, 0xe4);
                        self.ram.set_sprite_aux1_delay(k, 96);
                        self.ram.set_sprite_scratch_c(k, 1);
                    } else if self.ram.read_u8(0xf3c9) & 0x20 != 0 {
                        if self.runtime.show_solicited_message(self.ram, k, 0xd8) & 0x100 != 0 {
                            self.ram.set_sprite_ai_state(k, 1);
                            self.ram.set_sprite_scratch_c(k, 1);
                        }
                    } else {
                        self.runtime.show_solicited_message(self.ram, k, 0xdf);
                    }
                } else if self.ram.read_u8(0x20) < 0xc2 {
                    self.runtime.show_unconditional_message(self.ram, 0xe0);
                    self.ram.set_sprite_ai_state(k, 10);
                    self.ram
                        .write_u8(0x2e4, self.ram.read_u8(0x2e4).wrapping_add(1));
                }
            }
            1 => {
                if self.ram.read_u8(0x1ce8) == 0 {
                    self.runtime.show_unconditional_message(self.ram, 0xd9);
                    self.ram.set_sprite_ai_state(k, 2);
                } else {
                    self.runtime.show_unconditional_message(self.ram, 0xdc);
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            2 => {
                if self.ram.read_u8(0x1ce8) == 0 {
                    if self.ram.read_u8(0xf359) < 3 {
                        self.runtime.show_unconditional_message(self.ram, 0xda);
                        self.ram.set_sprite_ai_state(k, 3);
                    } else {
                        self.runtime.show_unconditional_message(self.ram, 0xdb);
                        self.ram.set_sprite_ai_state(k, 0);
                    }
                } else {
                    self.runtime.show_unconditional_message(self.ram, 0xdc);
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            3 => {
                let rupees = self.ram.read_u16(0xf360);
                if self.ram.read_u8(0x1ce8) != 0 || rupees < 10 {
                    self.runtime.show_unconditional_message(self.ram, 0xdc);
                    self.ram.set_sprite_ai_state(k, 0);
                } else {
                    self.ram.write_u16(0xf360, rupees - 10);
                    self.runtime.show_unconditional_message(self.ram, 0xdd);
                    self.ram.set_sprite_ai_state(partner, 5);
                    self.ram.set_sprite_ai_state(k, 5);
                    self.ram.write_u8(0xabf, 0);
                    self.ram.write_u8(0xf359, 255);
                    self.ram.write_u8(0xf3c9, self.ram.read_u8(0xf3c9) | 128);
                }
            }
            4 | 5 => {
                self.ram.set_sprite_scratch_c(k, 0);
                if self.smithy_listen_for_hammer(k) {
                    self.runtime.show_unconditional_message(self.ram, 0xe4);
                    self.ram.set_sprite_aux1_delay(k, 96);
                    self.ram.set_sprite_scratch_c(k, 1);
                } else if self.ram.read_u8(0xabf) != 0 {
                    if self.runtime.show_solicited_message(self.ram, k, 0xde) & 0x100 != 0 {
                        self.ram.set_sprite_ai_state(k, state.wrapping_add(1));
                        self.ram.set_sprite_graphics(k, 4);
                    }
                } else {
                    self.runtime.show_solicited_message(self.ram, k, 0xe2);
                }
            }
            6 => {
                self.ram.set_sprite_ai_state(k, 0);
                self.ram.set_sprite_ai_state(partner, 0);
                self.ram.write_u8(0x2e9, 0);
                self.runtime.link_receive_item(self.ram, 2, 0);
                self.ram.write_u8(0xf3c9, self.ram.read_u8(0xf3c9) & !0x80);
            }
            7..=9 => {}
            10 => {
                if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x1a) {
                    self.set_sprite_x(j, self.ram.read_u16(0x22));
                    self.set_sprite_y(j, self.ram.read_u16(0x20));
                    self.ram.set_sprite_subtype2(j, 3);
                    self.ram.set_sprite_ignore_projectile(j, 3);
                }
                self.ram.set_sprite_ai_state(k, 11);
                self.ram.write_u8(0xf3cc, 0);
                self.ram.set_sprite_graphics(k, 4);
            }
            11 => {
                self.runtime.show_solicited_message(self.ram, k, 0xe3);
            }
            _ => {}
        }
    }

    pub fn smithy_listen_for_hammer(&mut self, k: usize) -> bool {
        self.ram.sprite(k).aux1_delay() == 0
            && self.ram.read_u8(0x202) == 3
            && self.ram.read_u8(0x301) & 2 != 0
            && self.ram.read_u8(0x300) == 2
            && self.runtime.check_damage_to_link_same_layer(self.ram, k)
    }

    pub fn smithy_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 20] = [
            (1, 0, 0x4040, 2),
            (-11, -10, 0x4060, 2),
            (-1, 0, 0x0040, 2),
            (11, -10, 0x0060, 2),
            (1, 0, 0x4040, 2),
            (-3, -14, 0x4044, 2),
            (-1, 0, 0x0040, 2),
            (3, -14, 0x0044, 2),
            (1, 0, 0x4042, 2),
            (11, -10, 0x0060, 2),
            (-1, 0, 0x0042, 2),
            (-11, -10, 0x4060, 2),
            (1, 0, 0x4042, 2),
            (13, 2, 0x4062, 2),
            (-1, 0, 0x0042, 2),
            (-13, 2, 0x0062, 2),
            (0, 0, 0x4064, 2),
            (0, 0, 0x4062, 2),
            (0, 0, 0x0064, 2),
            (0, 0, 0x0064, 2),
        ];
        let start = usize::from(self.ram.sprite(k).graphics()) * 4
            + usize::from(self.ram.sprite(k).scratch_d()) * 2;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &P[start..start + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn smithy_spark(&mut self, k: usize) {
        const GFX: [i8; 7] = [0, 1, 2, 1, 2, 1, -1];
        const DELAY: [u8; 6] = [4, 1, 3, 2, 1, 1];
        self.smithy_spark_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).main_delay() == 0 {
            let a = self.ram.sprite(k).scratch_a();
            self.ram.set_sprite_scratch_a(k, a.wrapping_add(1) & 7);
            let g = GFX[usize::from(a)];
            if g < 0 {
                self.ram.set_sprite_state(k, 0);
                return;
            }
            self.ram.set_sprite_graphics(k, g as u8);
            self.ram.set_sprite_main_delay(k, DELAY[usize::from(a)]);
        }
    }

    pub fn smithy_spawn_spark(&mut self, k: usize) {
        let Some(j) = self.runtime.spawn_dynamic_xy(self.ram, k, 0x1a) else {
            return;
        };
        let dx = if self.ram.sprite(k).scratch_d() != 0 {
            (-15i8) as u8
        } else {
            15
        };
        self.ram
            .set_sprite_x_low(j, self.ram.sprite(j).x_low().wrapping_add(dx));
        self.ram
            .set_sprite_y_low(j, self.ram.sprite(j).y_low().wrapping_add(2));
        self.ram.set_sprite_subtype2(j, 1);
    }

    pub fn smithy_spark_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 6] = [
            (0, 3, 0x41aa, 2),
            (0, -1, 0x41aa, 2),
            (-4, 0, 0x0190, 0),
            (12, 0, 0x4190, 0),
            (-5, -2, 0x0191, 0),
            (13, -2, 0x0191, 0),
        ];
        self.runtime.oam_allocate_region_b(self.ram, 8);
        let start = usize::from(self.ram.sprite(k).graphics()) * 2;
        self.runtime
            .draw_multiple(self.ram, k, &P[start..start + 2]);
    }

    pub fn sprite_1b_arrow(&mut self, k: usize) {
        const XV: [i8; 8] = [0, 0, 16, 16, 0, 0, -16, -16];
        const YV: [i8; 8] = [16, 16, 0, 0, -16, -16, 0, 0];
        const DIR: [u8; 4] = [0, 2, 1, 3];
        self.enemy_arrow_draw(k);
        if self.runtime.return_if_paused(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).state() == 9 {
            let mut j = self.ram.sprite(k).main_delay();
            if j != 0 {
                j = j.wrapping_sub(1);
                if j == 0 {
                    self.ram.set_sprite_state(k, 0);
                } else if j >= 32 && j & 1 == 0 {
                    let n = usize::from(
                        (self.ram.read_u8(0x1a) << 1 & 4) | self.ram.sprite(k).scratch_d(),
                    );
                    self.ram.set_sprite_velocity_x(k, XV[n] as u8);
                    self.ram.set_sprite_velocity_y(k, YV[n] as u8);
                    self.runtime.move_xy(self.ram, k);
                }
                return;
            }
            self.runtime.check_damage_to_link_same_layer(self.ram, k);
            if self.ram.sprite(k).scratch_e() == 0 && self.runtime.check_tile_collision(self.ram, k)
            {
                if self.ram.sprite(k).scratch_a() != 0 {
                    self.runtime.sfx2_with_pan(self.ram, k, 5);
                    self.runtime.schedule_for_breakage(self.ram, k);
                    self.runtime.place_weapon_tink(self.ram, k);
                } else {
                    self.ram.set_sprite_main_delay(k, 48);
                    self.ram.set_sprite_scratch_a(k, 2);
                    self.runtime.sfx2_with_pan(self.ram, k, 8);
                }
            } else {
                self.runtime.move_xy(self.ram, k);
            }
        } else {
            if self.ram.sprite(k).ai_state() == 0 {
                self.runtime.apply_ricochet(self.ram, k);
                self.ram.set_sprite_velocity_z(k, 24);
                self.ram.set_sprite_main_delay(k, 255);
                self.ram.set_sprite_ai_state(k, 1);
                self.ram.set_sprite_hit_timer(k, 0);
            }
            self.ram.set_sprite_scratch_d(
                k,
                DIR[usize::from((self.ram.sprite(k).main_delay() >> 3) & 3)],
            );
            self.runtime.move_z(self.ram, k);
            self.runtime.move_xy(self.ram, k);
            self.ram
                .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(2));
            if (self.ram.sprite(k).z() as i8) < 0 {
                self.ram.set_sprite_state(k, 0);
            }
        }
    }

    pub fn enemy_arrow_draw(&mut self, k: usize) {
        const X: [i8; 8] = [-8, 0, 0, 8, 0, 0, 0, 0];
        const Y: [i8; 8] = [0, 0, 0, 0, -8, 0, 0, 8];
        const CH: [u8; 32] = [
            0x3a, 0x3d, 0x3d, 0x3a, 0x2a, 0x2b, 0x2b, 0x2a, 0x7c, 0x6c, 0x6c, 0x7c, 0x7b, 0x6b,
            0x6b, 0x7b, 0x3a, 0x3b, 0x3b, 0x3a, 0x2a, 0x3c, 0x3c, 0x2a, 0x81, 0x80, 0x80, 0x81,
            0x91, 0x90, 0x90, 0x91,
        ];
        const FL: [u8; 32] = [
            8, 8, 0x48, 0x48, 8, 8, 0x88, 0x88, 9, 0x49, 9, 0x49, 9, 0x89, 9, 0x89, 8, 0x88, 0xc8,
            0x48, 8, 8, 0x88, 0x88, 0x49, 0x49, 9, 9, 0x89, 0x89, 9, 9,
        ];
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags_wide(self.ram, k) else {
            return;
        };
        // C advances a local `OamEnt *` while drawing the two arrow pieces;
        // the global allocation cursors remain at the sprite's region base.
        let oam_ptr = self.ram.read_u16(0x90);
        let oam_ext_ptr = self.ram.read_u16(0x92);
        let r6 = usize::from(self.ram.sprite(k).scratch_d()) * 2;
        let r7 = usize::from(self.ram.sprite(k).scratch_a()) * 8;
        for i in (0..=1).rev() {
            let j = r6 + i;
            self.runtime.emit_oam_wide(
                self.ram,
                k,
                x.wrapping_add_signed(i16::from(X[j])),
                y.wrapping_add_signed(i16::from(Y[j])),
                CH[j + r7],
                FL[j + r7] | flags,
            );
        }
        self.ram.write_u16(0x90, oam_ptr);
        self.ram.write_u16(0x92, oam_ext_ptr);
    }

    pub fn sprite_1e_crystal_switch(&mut self, k: usize) {
        let pal = [2u8, 4][usize::from(self.ram.read_u16(0xc172) & 1)];
        self.ram
            .set_sprite_oam_flags(k, (self.ram.sprite(k).oam_flags() & !0xe) | pal);
        self.runtime.oam_allocate_defer_to_player(self.ram, k);
        self.runtime.draw_single_large(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            self.runtime.nullify_hookshot_drag(self.ram);
            self.ram.write_u8(0x5e, 0);
            self.runtime.repel_dash(self.ram);
        }
        if self.ram.sprite(k).main_delay() == 0 {
            let a = self.ram.read_u8(0x1a) & 7;
            let b = self.runtime.random_number(self.ram) & 7;
            self.runtime.garnish_spawn_sparkle(self.ram, k, a, b);
            self.ram.set_sprite_main_delay(k, 31);
        }
        let f = self.ram.sprite(k).scratch_f();
        if f == 0 {
            if (self.ram.read_u8(0x3c).wrapping_sub(9) as i8) < 0 {
                self.runtime.check_damage_from_link(self.ram, k);
            }
        } else {
            self.ram.set_sprite_scratch_f(k, f.wrapping_sub(1));
            if f == 11 {
                self.ram.write_u16(0xc172, self.ram.read_u16(0xc172) ^ 1);
                self.ram.write_u8(0x11, 22);
                self.runtime.sfx3_with_pan(self.ram, k, 0x25);
            }
        }
    }

    pub fn sprite_1f_sick_kid(&mut self, k: usize) {
        const GFX: [i8; 8] = [0, 1, 0, 1, 0, 1, 2, -1];
        const DELAY: [u8; 7] = [8, 12, 8, 12, 8, 96, 16];
        self.bug_net_kid_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.runtime.check_if_link_is_busy(self.ram)
                    || !self.runtime.check_damage_to_link_same_layer(self.ram, k)
                {
                    return;
                }
                let bottles = self.ram.read_u8(0xf35c)
                    | self.ram.read_u8(0xf35d)
                    | self.ram.read_u8(0xf35e)
                    | self.ram.read_u8(0xf35f);
                if bottles < 2 {
                    self.runtime.show_solicited_message(self.ram, k, 0x104);
                } else {
                    self.ram
                        .write_u8(0x2e4, self.ram.read_u8(0x2e4).wrapping_add(1));
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 if self.ram.sprite(k).main_delay() == 0 => {
                let a = self.ram.sprite(k).scratch_a();
                let g = GFX[usize::from(a)];
                if g >= 0 {
                    self.ram.set_sprite_graphics(k, g as u8);
                    self.ram.set_sprite_main_delay(k, DELAY[usize::from(a)]);
                    self.ram.set_sprite_scratch_a(k, a.wrapping_add(1));
                } else {
                    self.runtime.show_unconditional_message(self.ram, 0x105);
                    self.ram.set_sprite_ai_state(k, 2);
                }
            }
            2 => {
                self.ram.write_u8(0x2e9, 0);
                self.runtime.link_receive_item(self.ram, 0x21, 0);
                self.ram.write_u8(0x2e4, 0);
                self.ram.set_sprite_ai_state(k, 3);
            }
            3 => {
                self.ram.set_sprite_graphics(k, 1);
                self.runtime.show_solicited_message(self.ram, k, 0x106);
            }
            _ => {}
        }
    }

    pub fn bug_net_kid_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 18] = [
            (4, 0, 0x0027, 0),
            (0, -5, 0x000e, 2),
            (-8, 6, 0x040a, 2),
            (8, 6, 0x440a, 2),
            (-8, 14, 0x840a, 2),
            (8, 14, 0xc40a, 2),
            (0, -5, 0x000e, 2),
            (0, -5, 0x000e, 2),
            (-8, 6, 0x040a, 2),
            (8, 6, 0x440a, 2),
            (-8, 14, 0x840a, 2),
            (8, 14, 0xc40a, 2),
            (0, -5, 0x002e, 2),
            (0, -5, 0x002e, 2),
            (-8, 7, 0x040a, 2),
            (8, 7, 0x440a, 2),
            (-8, 14, 0x840a, 2),
            (8, 14, 0xc40a, 2),
        ];
        let start = usize::from(self.ram.sprite(k).graphics()) * 6;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &P[start..start + 6]);
    }

    pub fn sprite_78_mrs_sahasrahla(&mut self, k: usize) {
        self.runtime.elder_wife_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.read_u8(0xf359) < 2 {
                    if self.runtime.show_solicited_message(self.ram, k, 0x2b) & 0x100 != 0 {
                        self.ram.set_sprite_ai_state(k, 1);
                    }
                } else {
                    self.runtime.show_solicited_message(self.ram, k, 0x2e);
                }
                self.ram
                    .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 4) & 1);
            }
            1 => {
                self.runtime.show_unconditional_message(self.ram, 0x2c);
                self.ram.set_sprite_ai_state(k, 2);
            }
            2 => {
                if self.ram.read_u8(0x1ce8) == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    self.runtime.show_unconditional_message(self.ram, 0x2d);
                } else {
                    self.runtime.show_unconditional_message(self.ram, 0x2c);
                }
            }
            3 => {
                self.runtime.show_solicited_message(self.ram, k, 0x2d);
                self.ram
                    .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 4) & 1);
            }
            _ => {}
        }
    }

    pub fn sprite_16_elder_bounce(&mut self, k: usize) {
        self.runtime.elder_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        match self.ram.sprite(k).subtype2() {
            0 => self.runtime.sprite_sahasrahla(self.ram, k),
            1 => self.runtime.sprite_aginah(self.ram, k),
            _ => {}
        }
    }

    pub fn sprite_2d_telepathic_tile(&mut self, _k: usize) {
        panic!("unreachable Sprite_2D_TelepathicTile, matching C assert(0)");
    }

    pub fn sprite_1d_flute_quest(&mut self, k: usize) {
        let _ = self.runtime.prep_oam(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        let flute = self.ram.read_u8(0xf34c);
        if (self.ram.read_u8(0x8a) == 0x18 && flute == 3)
            || (self.ram.read_u8(0x8a) != 0x18 && flute & 2 != 0)
        {
            self.ram.set_sprite_state(k, 0);
        }
    }

    pub fn sprite_71_leever(&mut self, k: usize) {
        const EMERGE: [u8; 16] = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 2, 1, 2, 1, 0, 0];
        const ATTACK: [u8; 4] = [9, 10, 11, 12];
        const SPEED: [u8; 2] = [12, 8];
        if self.ram.sprite(k).ai_state() != 0 {
            self.leever_draw(k);
        } else {
            let _ = self.runtime.prep_oam(self.ram, k);
        }
        if self.ram.sprite(k).pause() != 0 {
            self.ram.set_sprite_state(k, 8);
        }
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                let delay = self.ram.sprite(k).main_delay();
                self.ram.set_sprite_ignore_projectile(k, delay);
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.set_sprite_main_delay(k, 127);
                } else {
                    self.runtime.apply_speed_towards_link(self.ram, k, 16);
                    self.runtime.move_xy(self.ram, k);
                    self.runtime.check_tile_collision2(self.ram, k);
                }
            }
            1 => {
                let delay = self.ram.sprite(k).main_delay();
                self.ram.set_sprite_ignore_projectile(k, delay);
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 2);
                    let d = (self.runtime.random_number(self.ram) & 63).wrapping_add(160);
                    self.ram.set_sprite_main_delay(k, d);
                    self.ram.set_sprite_velocity_x(k, 0);
                    self.ram.set_sprite_velocity_y(k, 0);
                } else {
                    self.ram
                        .set_sprite_graphics(k, EMERGE[usize::from(delay >> 3)]);
                }
            }
            2 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                if self.ram.sprite(k).main_delay() == 0 {
                    self.leever_stop_attack(k);
                } else {
                    let subtype = self.ram.sprite(k).subtype2();
                    if subtype & 7 == 0 {
                        self.runtime.apply_speed_towards_link(
                            self.ram,
                            k,
                            SPEED[usize::from(self.ram.sprite(k).scratch_a())],
                        );
                    }
                    self.runtime.move_xy(self.ram, k);
                    if self.runtime.check_tile_collision(self.ram, k) {
                        self.leever_stop_attack(k);
                    } else {
                        let next = subtype.wrapping_add(1);
                        self.ram.set_sprite_subtype2(k, next);
                        self.ram
                            .set_sprite_graphics(k, ATTACK[usize::from((next >> 2) & 3)]);
                    }
                }
            }
            3 => {
                let delay = self.ram.sprite(k).main_delay();
                self.ram.set_sprite_ignore_projectile(k, delay);
                if delay == 0 {
                    self.ram.set_sprite_ai_state(k, 0);
                    let d = (self.runtime.random_number(self.ram) & 31).wrapping_add(64);
                    self.ram.set_sprite_main_delay(k, d);
                } else {
                    self.ram
                        .set_sprite_graphics(k, EMERGE[usize::from((delay >> 3) ^ 15)]);
                }
            }
            _ => {}
        }
    }

    fn leever_stop_attack(&mut self, k: usize) {
        self.ram
            .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
        self.ram.set_sprite_main_delay(k, 127);
    }

    pub fn leever_draw(&mut self, k: usize) {
        const NUM: [u8; 14] = [1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1];
        const X: [i8; 56] = [
            2, 6, 6, 6, 0, 8, 8, 8, 0, 8, 8, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 0, 0, 8, 0, 0, 0, 8, 0,
            0, 0, 8, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const Y: [i8; 56] = [
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 5, 5, 8, 8, 5, 5, 8, 8, 2, 2, 8, 8, 1, 1, 8, 8, 0,
            0, 8, 8, -1, -1, 8, 8, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2,
            -2, 0,
        ];
        const CH: [u8; 56] = [
            0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x38, 0x38, 0x38, 0x38, 8, 9, 0x28,
            0x28, 8, 9, 0xd9, 0xd9, 8, 8, 0xd8, 0xd8, 8, 8, 0xda, 0xda, 6, 6, 0xd9, 0xd9, 0x26,
            0x26, 0xd8, 0xd8, 0x6c, 6, 6, 0, 0x6c, 0x26, 0x26, 0, 0x6c, 6, 6, 0, 0x6c, 0x26, 0x26,
            0, 0x6c, 8, 8, 0,
        ];
        const FL: [u8; 56] = [
            1, 0x41, 0x41, 0x41, 1, 0x41, 0x41, 0x41, 1, 0x41, 0x41, 0x41, 1, 1, 1, 0x41, 1, 1, 0,
            0x40, 1, 1, 0, 0x40, 1, 1, 0, 0x40, 1, 1, 0, 0x40, 0, 1, 0, 0x40, 6, 0x41, 0x41, 0, 6,
            0x41, 0x41, 0, 6, 1, 1, 0, 6, 1, 1, 0, 6, 1, 1, 0,
        ];
        const BIG: [u8; 56] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2,
            2, 0, 0, 2, 2, 0, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0,
        ];
        let Some((x, y, base_flags)) = self.runtime.prep_oam_with_flags_wide(self.ram, k) else {
            return;
        };
        let d = usize::from(self.ram.sprite(k).graphics());
        let oam_base = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for i in (0..=usize::from(NUM[d])).rev() {
            let j = d * 4 + i;
            let mut f = base_flags;
            if CH[j] >= 0x60 || CH[j] == 0x28 || CH[j] == 0x38 {
                f &= 0xf0;
            }
            crate::oam::set_helper0(
                self.ram,
                oam_base + usize::from(NUM[d]) - i,
                x.wrapping_add_signed(i16::from(X[j])),
                y.wrapping_add_signed(i16::from(Y[j])),
                CH[j],
                FL[j] | f,
                BIG[j],
            );
        }
    }

    pub fn sprite_d8_heart(&mut self, k: usize) {
        if self.runtime.draw_absorbable_transient(self.ram, k, true)
            || self.runtime.return_if_inactive(self.ram, k)
        {
            return;
        }
        self.runtime.check_absorption_by_player(self.ram, k);
        if self.ram.sprite(k).state() == 0 && self.runtime.misc_bug_fixes_enabled() {
            return;
        }
        if self.runtime.handle_dragging_by_ancilla(self.ram, k) {
            return;
        }
        self.runtime.move_xy(self.ram, k);
        self.runtime.move_z(self.ram, k);
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            self.ram
                .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
            self.ram.set_sprite_graphics(k, 0);
        }
        let mut flags = self.ram.sprite(k).oam_flags() & !0x40;
        if (self.ram.sprite(k).velocity_x() as i8) >= 0 {
            flags |= 0x40;
        }
        self.ram.set_sprite_oam_flags(k, flags);
        match self.ram.sprite(k).ai_state().min(3) {
            0 => {
                self.ram.set_sprite_ai_state(k, 1);
                self.ram.set_sprite_main_delay(k, 18);
                self.ram.set_sprite_velocity_z(k, 20);
                self.ram.set_sprite_graphics(k, 1);
                self.ram.set_sprite_scratch_d(k, 0);
            }
            1 => {
                if self.ram.sprite(k).main_delay() != 0 {
                    self.ram
                        .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(1));
                } else {
                    self.ram.set_sprite_ai_state(k, 2);
                    self.ram.set_sprite_velocity_z(k, 253);
                    self.ram.set_sprite_velocity_x(k, 0);
                }
            }
            2 if self.ram.sprite(k).main_delay() == 0 => {
                let d = self.ram.sprite(k).scratch_d();
                let accel = [1u8, 0xff][usize::from(d & 1)];
                let xv = self.ram.sprite(k).velocity_x().wrapping_add(accel);
                self.ram.set_sprite_velocity_x(k, xv);
                if xv == [10u8, (-10i8) as u8][usize::from(d & 1)] {
                    self.ram.set_sprite_scratch_d(k, d.wrapping_add(1));
                    self.ram.set_sprite_main_delay(k, 8);
                }
            }
            3 => {
                self.ram.set_sprite_velocity_z(k, 0);
                self.ram.set_sprite_velocity_x(k, 0);
                self.ram.set_sprite_velocity_y(k, 0);
            }
            _ => {}
        }
    }

    pub fn sprite_e4_small_key(&mut self, k: usize) {
        let action = usize::from(self.ram.sprite(k).death_action());
        if self.ram.read_u16(0x402) & (u16::from([0x40u8, 0x20][action]) << 8) != 0 {
            self.ram.set_sprite_state(k, 0);
            return;
        }
        self.runtime.draw_ripple_if_in_water(self.ram, k);
        if self.runtime.draw_absorbable_transient(self.ram, k, false) {
            return;
        }
        self.sprite_absorbable_main(k);
    }

    pub fn sprite_d9_green_rupee(&mut self, k: usize) {
        self.runtime.draw_ripple_if_in_water(self.ram, k);
        if self.runtime.draw_absorbable_transient(self.ram, k, true) {
            return;
        }
        self.sprite_absorbable_main(k);
    }

    pub fn sprite_absorbable_main(&mut self, k: usize) {
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.move_z(self.ram, k);
        self.runtime.move_xy(self.ram, k);
        if self.ram.sprite(k).aux3_delay() == 0 {
            self.runtime.check_tile_collision2(self.ram, k);
            self.runtime.bounce_off_wall(self.ram, k);
        }
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(2));
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            self.ram
                .set_sprite_velocity_x(k, ((self.ram.sprite(k).velocity_x() as i8) >> 1) as u8);
            self.ram
                .set_sprite_velocity_y(k, ((self.ram.sprite(k).velocity_y() as i8) >> 1) as u8);
            let rebound = 0u8.wrapping_sub(self.ram.sprite(k).velocity_z()) >> 1;
            if rebound < 9 {
                self.ram.set_sprite_velocity_z(k, 0);
                self.ram.set_sprite_velocity_x(k, 0);
                self.ram.set_sprite_velocity_y(k, 0);
            } else {
                self.ram.set_sprite_velocity_z(k, rebound);
                let tile = self.ram.sprite(k).scratch_i();
                if tile == 8 || tile == 9 {
                    self.ram.set_sprite_velocity_z(k, 0);
                    if let Some(j) = self.runtime.spawn_small_splash(self.ram, k)
                        && self.ram.sprite(k).flags3() & 0x20 != 0
                    {
                        let x = u16::from_le_bytes([
                            self.ram.sprite(j).x_low(),
                            self.ram.sprite(j).x_high(),
                        ])
                        .wrapping_sub(4);
                        let y = u16::from_le_bytes([
                            self.ram.sprite(j).y_low(),
                            self.ram.sprite(j).y_high(),
                        ])
                        .wrapping_sub(4);
                        self.set_sprite_x(j, x);
                        self.set_sprite_y(j, y);
                    }
                } else if self.ram.sprite(k).kind() >= 0xe4 && self.ram.read_u8(0x1b) != 0 {
                    self.runtime.sfx2_with_pan(self.ram, k, 5);
                }
            }
        }
        if self.runtime.handle_dragging_by_ancilla(self.ram, k) {
            return;
        }
        self.runtime.check_absorption_by_player(self.ram, k);
    }

    pub fn pipe_validate_entry(&mut self) -> bool {
        for k in (0..5).rev() {
            if self.ram.read_u8(0xc4a + k) == 0x31 {
                self.ram.write_u8(0x37a, 0);
                self.ram.write_u8(0x50, 0);
                self.ram.write_u8(0xc4a + k, 0);
                break;
            }
        }
        (self.ram.read_u8(0x308) & 0x80) | self.ram.read_u8(0x4d) != 0
    }

    pub fn ancilla_terminate_sparkle_objects(&mut self) {
        for i in (0..5).rev() {
            if matches!(
                self.ram.read_u8(0xc4a + i),
                0x2a | 0x2b | 0x30 | 0x31 | 0x18 | 0x19 | 0x0c
            ) {
                self.ram.write_u8(0xc4a + i, 0);
            }
        }
    }

    pub fn sprite_spawn_superficial_bomb_blast(&mut self, k: usize) -> Option<usize> {
        let j = self.runtime.spawn_dynamic(self.ram, k, 0x4a)?;
        self.ram.set_sprite_state(j, 6);
        self.ram.set_sprite_aux1_delay(j, 31);
        self.ram.set_sprite_scratch_c(j, 3);
        self.ram.set_sprite_flags2(j, 3);
        self.ram.set_sprite_oam_flags(j, 4);
        self.runtime.sfx2_with_pan(self.ram, k, 0x15);
        self.runtime.set_spawned_coordinates(self.ram, j);
        Some(j)
    }

    pub fn sprite_spawn_dummy_death_animation(&mut self, k: usize) {
        let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x0b) else {
            return;
        };
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.ram.set_sprite_state(j, 6);
        self.ram.set_sprite_main_delay(j, 15);
        self.runtime.sfx2_with_pan(self.ram, k, 0x14);
        self.ram.set_sprite_floor(j, 2);
    }

    pub fn sprite_magic_bat_spawn_lightning(&mut self, k: usize) {
        const XV: [i8; 4] = [-8, -4, 4, 8];
        const ST2: [u8; 4] = [0, 0x11, 0x22, 0x33];
        for _ in 0..4 {
            let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x3a) else {
                continue;
            };
            self.runtime.sfx3_with_pan(self.ram, k, 1);
            self.runtime.set_spawned_coordinates(self.ram, j);
            let x = u16::from_le_bytes([self.ram.sprite(j).x_low(), self.ram.sprite(j).x_high()])
                .wrapping_add(4);
            let y = u16::from_le_bytes([self.ram.sprite(j).y_low(), self.ram.sprite(j).y_high()])
                .wrapping_add(12)
                .wrapping_sub(u16::from(self.ram.sprite(k).z()));
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
            self.ram.set_sprite_z(j, 0);
            self.ram.set_sprite_velocity_y(j, 24);
            self.ram.set_sprite_head_direction(j, 24);
            self.ram.set_sprite_ignore_projectile(j, 24);
            self.ram.set_sprite_flags2(j, 0x80);
            self.ram.set_sprite_flags3(j, 3);
            self.ram.set_sprite_oam_flags(j, 3);
            self.ram.set_sprite_main_delay(j, 32);
            self.ram.set_sprite_graphics(j, 2);
            let g = self.ram.sprite(k).scratch_g();
            self.ram.set_sprite_velocity_x(j, XV[usize::from(g)] as u8);
            self.ram.set_sprite_subtype2(j, ST2[usize::from(g)]);
            self.ram.set_sprite_floor(j, 2);
            self.ram.set_sprite_scratch_g(k, g.wrapping_add(1));
        }
    }

    pub fn fireball_spawn_trail_garnish(&mut self, k: usize) {
        if (k as u8 ^ self.ram.read_u8(0x1a)) & 3 != 0 {
            return;
        }
        let j = self.runtime.garnish_alloc(self.ram);
        self.ram.set_garnish_kind(j, 8);
        self.ram.set_garnish_active_marker(8);
        self.ram.set_garnish_countdown(j, 11);
        let x = self.ram.read_u16(0xfd8);
        let y = self.ram.read_u16(0xfda).wrapping_add(16);
        self.ram.set_garnish_x_low(j, x as u8);
        self.ram.set_garnish_x_high(j, (x >> 8) as u8);
        self.ram.set_garnish_y_low(j, y as u8);
        self.ram.set_garnish_y_high(j, (y >> 8) as u8);
        self.ram.set_garnish_source_sprite(j, k as u8);
    }

    pub fn garnish_spawn_pyramid_debris(&mut self, x: i8, y: i8, xv: i8, yv: i8) {
        let k = self.runtime.garnish_alloc_force(self.ram);
        self.ram.write_u8(0x12f, 3);
        self.ram.write_u8(0x12e, 31);
        self.ram.write_u8(0x12d, 5);
        self.ram.set_garnish_kind(k, 19);
        self.ram.set_garnish_active_marker(19);
        self.ram.set_garnish_x_low(k, 232u8.wrapping_add(x as u8));
        self.ram.set_garnish_y_low(k, 96u8.wrapping_add(y as u8));
        self.ram.set_garnish_x_velocity(k, xv as u8);
        self.ram.set_garnish_y_velocity(k, yv as u8);
        let delay = (self.runtime.random_number(self.ram) & 31).wrapping_add(48);
        self.ram.set_garnish_countdown(k, delay);
    }

    pub fn snitch_spawn_guard(&mut self, k: usize) {
        const X: [u16; 3] = [0x120, 0x340, 0x2e0];
        const Y: [u16; 3] = [0x100, 0x3b0, 0x160];
        let Some(j) = self.runtime.spawn_dynamic_ex(self.ram, k, 0x45, 0) else {
            return;
        };
        let i = if self.ram.sprite(k).kind() == 0x3d {
            0
        } else if self.ram.sprite(k).kind() == 0x35 {
            1
        } else {
            2
        };
        self.set_sprite_x(j, X[i] + (self.ram.read_u16(0xfbc) & 0xff00));
        self.set_sprite_y(j, Y[i] + (self.ram.read_u16(0xfbe) & 0xff00));
        self.ram.set_sprite_floor(j, 0);
        self.ram.set_sprite_health(j, 4);
        self.ram.set_sprite_deflection_bits(j, 0x80);
        self.ram.set_sprite_flags5(j, 0x90);
        self.ram.set_sprite_oam_flags(j, 0x0b);
    }

    pub fn shop_keeper_rapid_terminate_receive_item(&mut self) {
        for i in (0..5).rev() {
            if self.ram.read_u8(0xc4a + i) == 0x22 {
                self.ram.write_u8(0x3b1 + i, 1);
            }
        }
    }

    pub fn sprite_initialize_secondary_item_minigame(&mut self, what: u8) {
        self.ram.write_u8(0x3fc, what);
        self.runtime.link_reset_properties_c(self.ram);
        for k in (0..5).rev() {
            let kind = self.ram.read_u8(0xc4a + k);
            if kind == 0x30 || kind == 0x31 {
                self.ram.write_u8(0xc4a + k, 0);
            } else if kind == 5 {
                self.ram.write_u8(0x35f, 0);
                self.ram.write_u8(0xc4a + k, 0);
            }
        }
    }

    pub fn bottle_merchant_buy_bee(&mut self, k: usize) {
        const XV: [i8; 5] = [-6, -3, 0, 4, 7];
        const YV: [i8; 5] = [11, 14, 16, 14, 11];
        self.runtime.sfx3_with_pan(self.ram, k, 0x13);
        for i in (0..5).rev() {
            let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xdb) else {
                continue;
            };
            self.runtime.set_spawned_coordinates(self.ram, j);
            self.ram
                .set_sprite_x_low(j, self.ram.sprite(j).x_low().wrapping_add(4));
            self.ram.set_sprite_stunned(j, 0xff);
            self.ram.set_sprite_velocity_x(j, XV[i] as u8);
            self.ram.set_sprite_velocity_y(j, YV[i] as u8);
            self.ram.set_sprite_velocity_z(j, 32);
            self.ram.set_sprite_aux4_delay(j, 32);
        }
    }

    pub fn sprite_chicken_lady(&mut self, k: usize) {
        self.ram.set_sprite_scratch_d(k, 1);
        self.lady_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.ram.sprite(k).main_delay() == 1 {
            self.ram.write_u16(0x1cf0, 0x17d);
            self.runtime.show_message_minimal(self.ram);
        }
        self.ram
            .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 4) & 1);
    }

    pub fn lady_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 16] = [
            (0, -8, 0x00e0, 2),
            (0, 0, 0x00e8, 2),
            (0, -7, 0x00e0, 2),
            (0, 1, 0x40e8, 2),
            (0, -8, 0x00c0, 2),
            (0, 0, 0x00c2, 2),
            (0, -7, 0x00c0, 2),
            (0, 1, 0x40c2, 2),
            (0, -8, 0x00e2, 2),
            (0, 0, 0x00e4, 2),
            (0, -7, 0x00e2, 2),
            (0, 1, 0x00e6, 2),
            (0, -8, 0x40e2, 2),
            (0, 0, 0x40e4, 2),
            (0, -7, 0x40e2, 2),
            (0, 1, 0x40e6, 2),
        ];
        let start = usize::from(self.ram.sprite(k).graphics()) * 2
            + usize::from(self.ram.sprite(k).scratch_d()) * 4;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &P[start..start + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn dust_cloud_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 24] = [
            (0, -3, 0x008b, 0),
            (3, 0, 0x009b, 0),
            (-3, 0, 0xc08b, 0),
            (0, 3, 0xc09b, 0),
            (0, -5, 0x008a, 2),
            (5, 0, 0x008a, 2),
            (-5, 0, 0x008a, 2),
            (0, 5, 0x008a, 2),
            (0, -7, 0x0086, 2),
            (7, 0, 0x0086, 2),
            (-7, 0, 0x0086, 2),
            (0, 7, 0x0086, 2),
            (0, -9, 0x8086, 2),
            (9, 0, 0x8086, 2),
            (-9, 0, 0x8086, 2),
            (0, 9, 0x8086, 2),
            (0, -9, 0xc086, 2),
            (9, 0, 0xc086, 2),
            (-9, 0, 0xc086, 2),
            (0, 9, 0xc086, 2),
            (0, -7, 0x4086, 2),
            (7, 0, 0x4086, 2),
            (-7, 0, 0x4086, 2),
            (0, 7, 0x4086, 2),
        ];
        self.ram.set_sprite_oam_flags(k, 0x14);
        let g = usize::from(self.ram.sprite(k).graphics()) * 4;
        self.runtime.draw_multiple(self.ram, k, &D[g..g + 4]);
    }

    pub fn medallion_tablet_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 20] = [
            (-8, -16, 0x008c, 2),
            (8, -16, 0x408c, 2),
            (-8, 0, 0x00ac, 2),
            (8, 0, 0x40ac, 2),
            (-8, -13, 0x008a, 2),
            (8, -13, 0x408a, 2),
            (-8, 0, 0x00ac, 2),
            (8, 0, 0x40ac, 2),
            (-8, -8, 0x008a, 2),
            (8, -8, 0x408a, 2),
            (-8, 0, 0x00ac, 2),
            (8, 0, 0x40ac, 2),
            (-8, -4, 0x008a, 2),
            (8, -4, 0x408a, 2),
            (-8, 0, 0x00aa, 2),
            (8, 0, 0x40aa, 2),
            (-8, 0, 0x00aa, 2),
            (8, 0, 0x40aa, 2),
            (-8, 0, 0x00aa, 2),
            (8, 0, 0x40aa, 2),
        ];
        let g = usize::from(self.ram.sprite(k).graphics()) * 4;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &D[g..g + 4]);
    }

    pub fn crystal_maiden_draw(&mut self, k: usize) {
        const DMA: [u8; 16] = [
            0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40,
            0x60,
        ];
        const D: [(i8, i8, u16, u8); 16] = [
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
        let j = usize::from(self.ram.sprite(k).scratch_d()) * 2
            + usize::from(self.ram.sprite(k).graphics());
        self.ram.write_u8(0xae8, DMA[j * 2]);
        self.ram.write_u8(0xaea, DMA[j * 2 + 1]);
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &D[j * 2..j * 2 + 2]);
    }

    pub fn priest_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 20] = [
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
        let j = usize::from(self.ram.sprite(k).scratch_d()) * 2
            + usize::from(self.ram.sprite(k).graphics());
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &D[j * 2..j * 2 + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn uncle_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 48] = [
            (0, -10, 0x0e00, 2),
            (0, 0, 0x0c06, 2),
            (0, -10, 0x0e00, 2),
            (0, 0, 0x0c06, 2),
            (0, -10, 0x0e00, 2),
            (0, 0, 0x0c06, 2),
            (0, -10, 0x0e02, 2),
            (0, 0, 0x0c06, 2),
            (0, -10, 0x0e02, 2),
            (0, 0, 0x0c06, 2),
            (0, -10, 0x0e02, 2),
            (0, 0, 0x0c06, 2),
            (-7, 2, 0x0d07, 2),
            (-7, 2, 0x0d07, 2),
            (10, 12, 0x8d05, 0),
            (10, 4, 0x8d15, 0),
            (0, -10, 0x0e00, 2),
            (0, 0, 0x0c04, 2),
            (-7, 1, 0x0d07, 2),
            (-7, 1, 0x0d07, 2),
            (10, 13, 0x8d05, 0),
            (10, 5, 0x8d15, 0),
            (0, -9, 0x0e00, 2),
            (0, 1, 0x4c04, 2),
            (-7, 8, 0x8d05, 0),
            (1, 8, 0x8d06, 0),
            (0, -10, 0x0e02, 2),
            (-6, -1, 0x4d07, 2),
            (0, 0, 0x0c23, 2),
            (0, 0, 0x0c23, 2),
            (-9, 7, 0x8d05, 0),
            (-1, 7, 0x8d06, 0),
            (0, -9, 0x0e02, 2),
            (-6, 0, 0x4d07, 2),
            (0, 1, 0x0c25, 2),
            (0, 1, 0x0c25, 2),
            (-10, -17, 0x0d07, 2),
            (15, -12, 0x8d15, 0),
            (15, -4, 0x8d05, 0),
            (0, -28, 0x0e08, 2),
            (-8, -19, 0x0c20, 2),
            (8, -19, 0x4c20, 2),
            (0, -28, 0x0e08, 2),
            (0, -28, 0x0e08, 2),
            (-8, -19, 0x0c20, 2),
            (8, -19, 0x4c20, 2),
            (-8, -19, 0x0c20, 2),
            (8, -19, 0x4c20, 2),
        ];
        const DMA3: [u8; 8] = [8, 8, 0, 0, 6, 6, 0, 0];
        const DMA4: [u8; 8] = [0, 0, 0, 0, 4, 4, 0, 0x8b];
        self.runtime.oam_allocate_region_b(self.ram, 0x18);
        let d = usize::from(self.ram.sprite(k).scratch_d());
        let g = usize::from(self.ram.sprite(k).graphics());
        let j = d * 2 + g;
        self.ram.write_u8(0x107, DMA3[j]);
        self.ram.write_u8(0x108, DMA4[j]);
        let s = d * 12 + g * 6;
        self.runtime.draw_multiple(self.ram, k, &D[s..s + 6]);
        if d != 0 && d != 3 {
            self.runtime.draw_shadow(self.ram, k);
        }
    }

    pub fn bomber_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 22] = [
            (0, 0, 0x40c6, 2),
            (0, 0, 0x40c6, 2),
            (0, 0, 0x40c4, 2),
            (0, 0, 0x40c4, 2),
            (0, 0, 0x00c6, 2),
            (0, 0, 0x00c6, 2),
            (0, 0, 0x00c4, 2),
            (0, 0, 0x00c4, 2),
            (-8, 0, 0x00c0, 2),
            (8, 0, 0x40c0, 2),
            (-8, 0, 0x00c2, 2),
            (8, 0, 0x40c2, 2),
            (-8, 0, 0x00e0, 2),
            (8, 0, 0x40e0, 2),
            (-8, 0, 0x00e2, 2),
            (8, 0, 0x40e2, 2),
            (-8, 0, 0x00e4, 2),
            (8, 0, 0x40e4, 2),
            (0, 0, 0x40e6, 2),
            (0, 0, 0x40e6, 2),
            (0, 0, 0x00e6, 2),
            (0, 0, 0x00e6, 2),
        ];
        let g = usize::from(self.ram.sprite(k).graphics()) * 2;
        self.runtime.draw_multiple(self.ram, k, &D[g..g + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_draw_zirro_bomb(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 15] = [
            (-11, 0, 0x019b, 0),
            (0, -8, 0xc19b, 0),
            (6, 6, 0x419b, 0),
            (-15, -6, 0x018a, 2),
            (-4, -14, 0x018a, 2),
            (2, 0, 0x018a, 2),
            (-15, -6, 0x0186, 2),
            (-4, -14, 0x0186, 2),
            (2, 0, 0x0186, 2),
            (-4, -4, 0x0186, 2),
            (-4, -4, 0x0186, 2),
            (-4, -4, 0x0186, 2),
            (-4, -4, 0x01aa, 2),
            (-4, -4, 0x01aa, 2),
            (-4, -4, 0x01aa, 2),
        ];
        let delay = self.ram.sprite(k).main_delay();
        if delay == 0 {
            self.ram.set_sprite_state(k, 0);
        }
        let start = usize::from(delay >> 2) * 3;
        self.runtime
            .draw_multiple(self.ram, k, &D[start..start + 3]);
    }

    pub fn drinking_guy_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 6] = [
            (8, 2, 0x00ae, 0),
            (0, -9, 0x0822, 2),
            (0, 0, 0x0006, 2),
            (7, 0, 0x00af, 0),
            (0, -9, 0x0822, 2),
            (0, 0, 0x0006, 2),
        ];
        let g = usize::from(self.ram.sprite(k).graphics()) * 3;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &D[g..g + 3]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn player_bee_hone_in_on_target(&mut self, j: usize, k: usize) {
        if self.ram.sprite(j).kind() != 0x88 && self.ram.sprite(j).flags2() & 2 != 0 {
            return;
        }
        if self
            .sprite_x(k)
            .wrapping_sub(self.sprite_x(j))
            .wrapping_add(16)
            >= 24
            || self
                .sprite_y(k)
                .wrapping_sub(self.sprite_y(j))
                .wrapping_sub(8)
                >= 24
        {
            return;
        }
        if self.ram.sprite(j).kind() == 0x75 {
            self.ram.set_sprite_scratch_e(j, (k + 1) as u8);
            return;
        }
        self.runtime
            .ancilla_check_damage_to_sprite_preset(self.ram, j, 1);
        self.ram.set_sprite_aux1_delay(j, 15);
        self.ram
            .set_sprite_recoil_x(j, self.ram.sprite(k).velocity_x() << 1);
        self.ram
            .set_sprite_recoil_y(j, self.ram.sprite(k).velocity_y() << 1);
        self.ram
            .set_sprite_scratch_b(k, self.ram.sprite(k).scratch_b().wrapping_add(1));
    }

    pub fn sprite_spawn_fireball(&mut self, k: usize) -> Option<usize> {
        self.runtime.sfx3_with_pan(self.ram, k, 0x19);
        let j = self.runtime.spawn_dynamic_ex(self.ram, k, 0x55, 13)?;
        self.set_sprite_x(j, self.sprite_x(k).wrapping_add(4));
        self.set_sprite_y(
            j,
            self.sprite_y(k)
                .wrapping_add(4)
                .wrapping_sub(u16::from(self.ram.sprite(k).z())),
        );
        self.ram
            .set_sprite_flags3(j, (self.ram.sprite(j).flags3() & 0xfe) | 0x40);
        self.ram.set_sprite_oam_flags(j, 6);
        self.ram.set_sprite_flags4(j, 0x54);
        self.ram.set_sprite_scratch_e(j, 0x54);
        self.ram.set_sprite_flags2(j, 0x20);
        self.runtime.apply_speed_towards_link(self.ram, j, 0x20);
        self.ram.set_sprite_main_delay(j, 20);
        self.ram.set_sprite_aux1_delay(j, 16);
        self.ram.set_sprite_flags5(j, 0);
        self.ram.set_sprite_deflection_bits(j, 0x48);
        Some(j)
    }

    pub fn lanmola_spawn_shrapnel(&mut self, k: usize) {
        const YV: [i8; 8] = [28, -28, 28, -28, 0, 36, 0, -36];
        const XV: [i8; 8] = [-28, -28, 28, 28, -36, 0, 36, 0];
        let mut i =
            if self.ram.sprite(0).state() + self.ram.sprite(1).state() + self.ram.sprite(2).state()
                < 10
            {
                7
            } else {
                3
            };
        loop {
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0xc2) {
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.ram
                    .set_sprite_x_low(j, self.ram.sprite(j).x_low().wrapping_add(4));
                self.ram
                    .set_sprite_y_low(j, self.ram.sprite(j).y_low().wrapping_add(4));
                self.ram.set_sprite_ignore_projectile(j, 1);
                self.ram.set_sprite_bump_damage(j, 1);
                self.ram.set_sprite_flags4(j, 1);
                self.ram.set_sprite_z(j, 0);
                self.ram.set_sprite_flags2(j, 0x20);
                self.ram.set_sprite_velocity_x(j, XV[i] as u8);
                self.ram.set_sprite_velocity_y(j, YV[i] as u8);
                let g = self.runtime.random_number(self.ram) & 1;
                self.ram.set_sprite_graphics(j, g);
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    pub fn movable_statue_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 3] =
            [(0, -8, 0x00c2, 0), (8, -8, 0x40c2, 0), (0, 0, 0x00c0, 2)];
        self.runtime.draw_multiple_player_deferred(self.ram, k, &D);
    }

    pub fn movable_mantle_draw(&mut self, k: usize) {
        const X: [u8; 6] = [0, 16, 32, 0, 16, 32];
        const Y: [u8; 6] = [0, 0, 0, 16, 16, 16];
        const CH: [u8; 6] = [0xc, 0xe, 0xc, 0x2c, 0x2e, 0x2c];
        const FL: [u8; 6] = [0x31, 0x31, 0x71, 0x31, 0x31, 0x71];
        self.runtime.oam_allocate_region_b(self.ram, 0x20);
        let Some((x, y, _)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        // The C routine writes six tiles at the allocated OAM pointer without
        // advancing it, then corrects their extended bits and clips their Y.
        let first = (usize::from(self.ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for (offset, i) in (0..6).rev().enumerate() {
            crate::oam::write(
                self.ram,
                first + offset,
                z3_core::c_types::OamEnt {
                    x: x.wrapping_add(X[i]),
                    y: y.wrapping_add(Y[i]),
                    charnum: CH[i],
                    flags: FL[i],
                },
            );
        }
        self.runtime.correct_oam_entries(self.ram, k, 5, 2);
    }

    pub fn mothula_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 24] = [
            (-24, -8, 0x0080, 2),
            (-8, -8, 0x0082, 2),
            (8, -8, 0x4082, 2),
            (24, -8, 0x4080, 2),
            (-24, 8, 0x00a0, 2),
            (-8, 8, 0x00a2, 2),
            (8, 8, 0x40a2, 2),
            (24, 8, 0x40a0, 2),
            (-24, -8, 0x0084, 2),
            (-8, -8, 0x0086, 2),
            (8, -8, 0x4086, 2),
            (24, -8, 0x4084, 2),
            (-24, 8, 0x00a4, 2),
            (-8, 8, 0x00a6, 2),
            (8, 8, 0x40a6, 2),
            (24, 8, 0x40a4, 2),
            (-8, -8, 0x0088, 2),
            (-8, -8, 0x0088, 2),
            (8, -8, 0x4088, 2),
            (8, -8, 0x4088, 2),
            (-8, 8, 0x00a8, 2),
            (-8, 8, 0x00a8, 2),
            (8, 8, 0x40a8, 2),
            (8, 8, 0x40a8, 2),
        ];
        const X: [i8; 27] = [
            0, 3, 6, 9, 12, -3, -6, -9, -12, 0, 2, 4, 6, 8, -2, -4, -6, -8, 0, 1, 2, 3, 4, -1, -2,
            -3, -4,
        ];
        // Mothula owns a fixed OAM region. Sprite_DrawMultiple and the wing
        // pieces use local pointers, leaving the global pointer at 0x920.
        self.ram.write_u16(0x90, 0x920);
        self.ram.write_u16(0x92, 0xa68);
        let g = usize::from(self.ram.sprite(k).graphics());
        self.runtime
            .draw_multiple(self.ram, k, &D[g * 8..g * 8 + 8]);
        if self.ram.sprite(k).pause() != 0 {
            return;
        }
        let Some((x, y, _flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        let yy = y.wrapping_add(self.ram.sprite(k).z()).wrapping_add(16);
        for (offset, i) in (0..9).rev().enumerate() {
            crate::oam::set_helper0(
                self.ram,
                (0x920 - 0x800) / 4 + 10 + offset,
                u16::from(x).wrapping_add_signed(i16::from(X[g * 9 + i])),
                u16::from(yy),
                0x6c,
                0x24,
                2,
            );
        }
    }

    pub fn probe_check_tile_solidity(&mut self, k: usize) -> bool {
        let floor = self.ram.sprite(k).floor();
        // The C routine intentionally tests the coordinates captured before
        // Probe advances the sprite, not the sprite's newly-written position.
        let x = self.ram.read_u16(0xfd8);
        let y = self.ram.read_u16(0xfda);
        let tile = if self.ram.read_u8(0x1b) != 0 {
            self.runtime.get_tile_attribute(self.ram, floor, x, y)
        } else {
            self.runtime
                .overworld_read_tile_attribute(self.ram, x >> 3, y)
        };
        self.ram.write_u8(0xfa5, tile);
        SPRITE_SIMPLIFIED_TILE_ATTRIBUTE[usize::from(tile)] >= 1
    }

    pub fn pikit_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 8] = [
            (0, 0, 0x00c8, 2),
            (0, 0, 0x00c8, 2),
            (0, 0, 0x00ca, 2),
            (0, 0, 0x00ca, 2),
            (-8, 0, 0x00cc, 2),
            (8, 0, 0x40cc, 2),
            (-8, 0, 0x00ce, 2),
            (8, 0, 0x40ce, 2),
        ];
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        let oam_ptr = self.ram.read_u16(0x90);
        let oam_ext_ptr = self.ram.read_u16(0x92);
        self.sprite_draw_pikit_tongue_at(k, x, y, flags);
        let oam_address = usize::from(oam_ptr);
        self.ram.write_u8(0xfb5, self.ram.read_u8(oam_address));
        self.ram.write_u8(0xfb6, self.ram.read_u8(oam_address + 1));
        self.ram.write_u16(0x90, oam_ptr.wrapping_add(24));
        self.ram.write_u16(0x92, oam_ext_ptr.wrapping_add(6));
        let g = usize::from(self.ram.sprite(k).graphics()) * 2;
        self.runtime.draw_multiple(self.ram, k, &D[g..g + 2]);
        let f = self.ram.sprite(k).flags2();
        self.ram.set_sprite_flags2(k, f.wrapping_sub(6));
        self.runtime.draw_shadow(self.ram, k);
        self.ram.set_sprite_flags2(k, f);
        self.sprite_draw_pikit_loot(k);
    }

    pub fn sprite_draw_pikit_tongue(&mut self, k: usize) {
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        self.sprite_draw_pikit_tongue_at(k, x, y, flags);
    }

    fn sprite_draw_pikit_tongue_at(&mut self, k: usize, x: u8, y: u8, flags: u8) {
        const M: [u8; 4] = [0x33, 0x66, 0x99, 0xcc];
        const CH: [u8; 8] = [0xee, 0xfd, 0xed, 0xfd, 0xee, 0xfd, 0xed, 0xfd];
        const FL: [u8; 8] = [0, 0, 0, 0x40, 0x40, 0xc0, 0x80, 0x80];
        if self.ram.sprite(k).ai_state() != 2 || self.ram.sprite(k).pause() != 0 {
            return;
        }
        let oam_ptr = self.ram.read_u16(0x90);
        let oam_ext_ptr = self.ram.read_u16(0x92);
        let x = x.wrapping_add(4);
        let y = y.wrapping_add(3);
        let a = self.ram.sprite(k).scratch_a() as i8;
        let b = self.ram.sprite(k).scratch_b() as i8;
        self.runtime.emit_oam_sized(
            self.ram,
            k,
            (
                x.wrapping_add(a as u8),
                y.wrapping_add(b as u8),
                0xfe,
                flags,
                0,
            ),
        );
        for i in (0..4).rev() {
            let ox = (i16::from(a) * i16::from(M[i]) / 256) as i8;
            let oy = (i16::from(b) * i16::from(M[i]) / 256) as i8;
            let g = usize::from(self.ram.sprite(k).scratch_d());
            self.runtime.emit_oam_sized(
                self.ram,
                k,
                (
                    x.wrapping_add(ox as u8),
                    y.wrapping_add(oy as u8),
                    CH[g],
                    FL[g] | flags,
                    0,
                ),
            );
        }
        self.runtime
            .emit_oam_sized(self.ram, k, (x, y, 0xfe, flags, 0));
        self.ram.write_u16(0x90, oam_ptr);
        self.ram.write_u16(0x92, oam_ext_ptr);
        self.runtime.correct_oam_entries(self.ram, k, 5, 0);
    }

    pub fn sprite_draw_pikit_loot(&mut self, k: usize) {
        const X: [i8; 20] = [
            -4, 4, -4, 4, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -4, 4, -4, 4,
        ];
        const Y: [i8; 20] = [
            -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4,
        ];
        const CH: [u8; 20] = [
            0x6e, 0x6f, 0x7e, 0x7f, 0x63, 0x7c, 0x73, 0x7c, 0xb, 0x7c, 0x1b, 0x7c, 0xec, 0xf9,
            0xfc, 0xf9, 0xea, 0xeb, 0xfa, 0xfb,
        ];
        const FL: [u8; 5] = [0x24, 0x24, 0x28, 0x29, 0x2f];
        let raw = self.ram.sprite(k).scratch_g();
        if raw == 0 {
            return;
        }
        let mut g = usize::from(raw - 1);
        if g == 3 {
            g = usize::from(self.ram.sprite(k).subtype()) + 2;
        }
        self.runtime.oam_allocate_region_c(self.ram, 0x10);
        let oam_ptr = self.ram.read_u16(0x90);
        let oam_ext_ptr = self.ram.read_u16(0x92);
        let x = self.ram.read_u8(0xfb5);
        let y = self.ram.read_u8(0xfb6);
        for i in (0..4).rev() {
            let j = g * 4 + i;
            self.runtime.emit_oam_sized(
                self.ram,
                k,
                (
                    x.wrapping_add(X[j] as u8),
                    y.wrapping_add(Y[j] as u8),
                    CH[j],
                    FL[g],
                    0,
                ),
            );
        }
        self.ram.write_u16(0x90, oam_ptr);
        self.ram.write_u16(0x92, oam_ext_ptr);
        self.runtime.correct_oam_entries(self.ram, k, 3, 0);
    }

    pub fn archery_game_guy_draw(&mut self, k: usize) {
        const X: [i8; 15] = [0, 0, 0, 0, 0, -5, 0, -1, -1, 0, 0, 0, 0, 1, 1];
        const Y: [i8; 15] = [
            0, -10, -10, 0, -10, -3, 0, -10, -10, 0, -10, -10, 0, -10, -10,
        ];
        const CH: [u8; 15] = [0x26, 6, 6, 8, 6, 0x3a, 0x26, 6, 6, 0x26, 6, 6, 0x26, 6, 6];
        const FL: [u8; 15] = [8, 6, 6, 8, 6, 8, 8, 6, 6, 8, 6, 6, 8, 6, 6];
        const SIZE: [u8; 15] = [2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2];
        self.runtime.oam_allocate_defer_to_player(self.ram, k);
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        let g = usize::from(self.ram.sprite(k).graphics());
        for i in (0..3).rev() {
            let j = g * 3 + i;
            self.runtime.emit_oam_sized(
                self.ram,
                k,
                (
                    x.wrapping_add(X[j] as u8),
                    y.wrapping_add(Y[j] as u8),
                    CH[j],
                    FL[j] | flags,
                    SIZE[j],
                ),
            );
        }
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn kholdstare_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 16] = [
            (-8, -8, 0x0080, 2),
            (8, -8, 0x0082, 2),
            (-8, 8, 0x00a0, 2),
            (8, 8, 0x00a2, 2),
            (-7, -7, 0x0080, 2),
            (7, -7, 0x0082, 2),
            (-7, 7, 0x00a0, 2),
            (7, 7, 0x00a2, 2),
            (-7, -7, 0x0084, 2),
            (7, -7, 0x0086, 2),
            (-7, 7, 0x00a4, 2),
            (7, 7, 0x00a6, 2),
            (-8, -8, 0x0084, 2),
            (8, -8, 0x0086, 2),
            (-8, 8, 0x00a4, 2),
            (8, 8, 0x00a6, 2),
        ];
        const X: [i8; 16] = [8, 7, 4, 2, 0, -2, -4, -7, -8, -7, -4, -2, 0, 2, 4, 7];
        const Y: [i8; 16] = [0, 2, 4, 7, 8, 7, 4, 2, 0, -2, -4, -7, -8, -7, -4, -2];
        const CH: [u8; 16] = [
            0xac, 0xac, 0xaa, 0x8c, 0x8c, 0x8c, 0xaa, 0xac, 0xac, 0xaa, 0xaa, 0x8c, 0x8c, 0x8c,
            0xaa, 0xac,
        ];
        const FL: [u8; 16] = [
            0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0xc0,
        ];
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags(self.ram, k) else {
            return;
        };
        let a = usize::from(self.ram.sprite(k).scratch_a());
        self.runtime.emit_oam_sized(
            self.ram,
            k,
            (
                x.wrapping_add(X[a] as u8),
                y.wrapping_add(Y[a] as u8),
                CH[a],
                FL[a] | flags,
                2,
            ),
        );
        let g = usize::from(self.ram.sprite(k).graphics()) * 4;
        self.runtime.draw_multiple(self.ram, k, &D[g..g + 4]);
    }

    pub fn wizzbeam_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 8] = [
            (0, -4, 0x00c5, 0),
            (0, 4, 0x80c5, 0),
            (0, -4, 0x40c5, 0),
            (0, 4, 0xc0c5, 0),
            (-4, 0, 0x40d2, 0),
            (4, 0, 0x00d2, 0),
            (-4, 0, 0xc0d2, 0),
            (4, 0, 0x80d2, 0),
        ];
        let d = usize::from(self.ram.sprite(k).scratch_d()) * 2;
        self.runtime.draw_multiple(self.ram, k, &D[d..d + 2]);
    }

    pub fn wizzrobe_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 24] = [
            (0, -8, 0x00b2, 0),
            (8, -8, 0x00b3, 0),
            (0, 0, 0x0088, 2),
            (0, -8, 0x00b2, 0),
            (8, -8, 0x00b3, 0),
            (0, 0, 0x0086, 2),
            (0, -8, 0x00b2, 0),
            (8, -8, 0x00b3, 0),
            (0, 0, 0x008c, 2),
            (0, -8, 0x00b2, 0),
            (8, -8, 0x00b3, 0),
            (0, 0, 0x008a, 2),
            (0, -8, 0x00b2, 0),
            (8, -8, 0x00b3, 0),
            (0, 0, 0x408c, 2),
            (0, -8, 0x00b2, 0),
            (8, -8, 0x00b3, 0),
            (0, 0, 0x408a, 2),
            (0, -8, 0x00b2, 0),
            (8, -8, 0x00b3, 0),
            (0, 0, 0x00a4, 2),
            (0, -8, 0x00b2, 0),
            (8, -8, 0x00b3, 0),
            (0, 0, 0x008e, 2),
        ];
        let s = usize::from(self.ram.sprite(k).graphics()) * 3;
        self.runtime.draw_multiple(self.ram, k, &D[s..s + 3]);
    }

    pub fn freezor_draw(&mut self, k: usize) {
        const D0: [(i8, i8, u16, u8); 28] = [
            (-8, 0, 0x00a6, 2),
            (8, 0, 0x40a6, 2),
            (-8, 0, 0x00a6, 2),
            (8, 0, 0x40a6, 2),
            (-8, 0, 0x00a6, 2),
            (8, 0, 0x40a6, 2),
            (0, 11, 0x00ab, 0),
            (8, 11, 0x40ab, 0),
            (-8, 0, 0x00ac, 2),
            (8, 0, 0x40a8, 2),
            (0, 11, 0x00ba, 0),
            (8, 11, 0x00bb, 0),
            (-8, 0, 0x00a8, 2),
            (8, 0, 0x40ac, 2),
            (0, 11, 0x40bb, 0),
            (8, 11, 0x40ba, 0),
            (0, 2, 0x00ae, 0),
            (8, 2, 0x40ae, 0),
            (0, 10, 0x00be, 0),
            (8, 10, 0x40be, 0),
            (0, 4, 0x00af, 0),
            (8, 4, 0x40af, 0),
            (0, 12, 0x00bf, 0),
            (8, 12, 0x40bf, 0),
            (0, 8, 0x00aa, 0),
            (8, 8, 0x40aa, 0),
            (0, 8, 0x00aa, 0),
            (8, 8, 0x40aa, 0),
        ];
        const D1: [(i8, i8, u16, u8); 8] = [
            (0, 0, 0x00ae, 0),
            (8, 0, 0x40ae, 0),
            (0, 8, 0x00be, 0),
            (8, 8, 0x40be, 0),
            (-2, 0, 0x00ae, 0),
            (10, 0, 0x40ae, 0),
            (-2, 8, 0x00be, 0),
            (10, 8, 0x40be, 0),
        ];
        let g = usize::from(self.ram.sprite(k).graphics());
        if g != 7 {
            self.runtime
                .draw_multiple(self.ram, k, &D0[g * 4..g * 4 + 4]);
        } else {
            self.runtime.draw_multiple(self.ram, k, &D1);
        }
    }

    pub fn zazak_draw(&mut self, k: usize) {
        const CH: [u8; 8] = [0x82, 0x82, 0x80, 0x84, 0x88, 0x88, 0x86, 0x84];
        const FL: [u8; 8] = [0x40, 0, 0, 0, 0x40, 0, 0, 0];
        const D: [(i8, i8, u16, u8); 24] = [
            (0, -8, 0x0008, 2),
            (-4, 0, 0x00a0, 2),
            (4, 0, 0x00a1, 2),
            (0, -7, 0x0008, 2),
            (-4, 1, 0x40a1, 2),
            (4, 1, 0x40a0, 2),
            (0, -8, 0x000e, 2),
            (-4, 0, 0x00a3, 2),
            (4, 0, 0x00a4, 2),
            (0, -7, 0x000e, 2),
            (-4, 1, 0x40a4, 2),
            (4, 1, 0x40a3, 2),
            (0, -9, 0x000c, 2),
            (0, 0, 0x00a6, 2),
            (0, 0, 0x00a6, 2),
            (0, -8, 0x000c, 2),
            (0, 0, 0x00a8, 2),
            (0, 0, 0x00a8, 2),
            (0, -9, 0x400c, 2),
            (0, 0, 0x40a6, 2),
            (0, 0, 0x40a6, 2),
            (0, -8, 0x400c, 2),
            (0, 0, 0x40a8, 2),
            (0, 0, 0x40a8, 2),
        ];
        let g = usize::from(self.ram.sprite(k).graphics()) * 3;
        self.runtime.draw_multiple(self.ram, k, &D[g..g + 3]);
        if self.ram.sprite(k).pause() != 0 {
            return;
        }
        let i = usize::from(self.ram.sprite(k).head_direction())
            + if self.ram.sprite(k).aux1_delay() == 0 {
                0
            } else {
                4
            };
        self.runtime
            .set_oam_piece_character_and_low_flags(self.ram, 0, CH[i], FL[i], 0x40);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn stalfos_draw(&mut self, k: usize) {
        const CH: [u8; 4] = [2, 2, 0, 4];
        const FL: [u8; 4] = [0x70, 0x30, 0x30, 0x30];
        const D: [(i8, i8, u16, u8); 36] = [
            (0, -10, 0x0000, 2),
            (0, 0, 0x0006, 2),
            (0, 0, 0x0006, 2),
            (0, -9, 0x0000, 2),
            (0, 1, 0x4006, 2),
            (0, 1, 0x4006, 2),
            (0, -10, 0x0004, 2),
            (0, 0, 0x0006, 2),
            (0, 0, 0x0006, 2),
            (0, -9, 0x0004, 2),
            (0, 1, 0x4006, 2),
            (0, 1, 0x4006, 2),
            (0, -10, 0x0002, 2),
            (5, 5, 0x002e, 0),
            (0, 0, 0x0024, 2),
            (0, -10, 0x0002, 2),
            (0, 0, 0x000e, 2),
            (0, 0, 0x000e, 2),
            (0, -10, 0x4002, 2),
            (3, 5, 0x402e, 0),
            (0, 0, 0x4024, 2),
            (0, -10, 0x4002, 2),
            (0, 0, 0x400e, 2),
            (0, 0, 0x000e, 2),
            (2, -8, 0x4002, 2),
            (0, 0, 0x4008, 2),
            (0, 0, 0x4008, 2),
            (-2, -8, 0x0002, 2),
            (0, 0, 0x0008, 2),
            (0, 0, 0x0008, 2),
            (0, -6, 0x0000, 2),
            (0, 0, 0x000a, 2),
            (0, 0, 0x000a, 2),
            (0, 0, 0x000a, 2),
            (0, -6, 0x0004, 2),
            (0, -6, 0x0004, 2),
        ];
        if self.ram.sprite(k).aux2_delay() != 0 {
            let _ = self.runtime.prep_oam(self.ram, k);
            return;
        }
        let g = usize::from(self.ram.sprite(k).graphics());
        self.runtime
            .draw_multiple(self.ram, k, &D[g * 3..g * 3 + 3]);
        if g < 8 && self.ram.sprite(k).pause() == 0 {
            let i = usize::from(self.ram.sprite(k).head_direction());
            self.runtime
                .set_oam_piece_character_and_low_flags(self.ram, 0, CH[i], FL[i], 0x70);
        }
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn babusu_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 40] = [
            (0, 4, 0x4380, 0),
            (0, 4, 0x4380, 0),
            (0, 4, 0x43b6, 0),
            (0, 4, 0x43b6, 0),
            (0, 4, 0x43b7, 0),
            (8, 4, 0x0380, 0),
            (0, 4, 0x4380, 0),
            (8, 4, 0x03b6, 0),
            (8, 4, 0x03b7, 0),
            (8, 4, 0x03b7, 0),
            (8, 4, 0x0380, 0),
            (8, 4, 0x0380, 0),
            (4, 0, 0x8380, 0),
            (4, 0, 0x8380, 0),
            (4, 0, 0x83b6, 0),
            (4, 0, 0x83b6, 0),
            (4, 0, 0x83b7, 0),
            (4, 8, 0x0380, 0),
            (4, 0, 0x8380, 0),
            (4, 8, 0x03b6, 0),
            (4, 8, 0x03b7, 0),
            (4, 8, 0x03b7, 0),
            (4, 8, 0x0380, 0),
            (4, 8, 0x0380, 0),
            (0, -8, 0x0a4e, 2),
            (0, 0, 0x0a5e, 2),
            (0, -8, 0x4a4e, 2),
            (0, 0, 0x4a5e, 2),
            (8, 0, 0x0a6c, 2),
            (0, 0, 0x0a6b, 2),
            (8, 0, 0x8a6c, 2),
            (0, 0, 0x8a6b, 2),
            (0, 8, 0x8a4e, 2),
            (0, 0, 0x8a5e, 2),
            (0, 8, 0xca4e, 2),
            (0, 0, 0xca5e, 2),
            (-8, 0, 0x4a6c, 2),
            (0, 0, 0x4a6b, 2),
            (-8, 0, 0xca6c, 2),
            (0, 0, 0xca6b, 2),
        ];
        let g = self.ram.sprite(k).graphics();
        if g != 0xff {
            let s = usize::from(g) * 2;
            self.runtime.draw_multiple(self.ram, k, &D[s..s + 2]);
        } else {
            let _ = self.runtime.prep_oam(self.ram, k);
        }
    }

    pub fn overworld_draw_wooden_door(&mut self, pos: u16, unlocked: bool) {
        self.runtime.overworld_draw_map16_persist(
            self.ram,
            pos,
            if unlocked { 0xda5 } else { 0xda4 },
        );
        self.runtime.overworld_draw_map16_persist(
            self.ram,
            pos.wrapping_add(2),
            if unlocked { 0xda7 } else { 0xda6 },
        );
        self.ram.write_u8(0x14, 1);
    }

    pub fn sprite_d4_landmine(&mut self, k: usize) {
        const FLAGS: [u8; 4] = [4, 2, 8, 2];
        self.landmine_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if !self
            .runtime
            .landmine_check_detonation_from_hammer(self.ram, k)
        {
            let delay = self.ram.sprite(k).main_delay();
            if delay == 0 {
                self.ram.set_sprite_oam_flags(k, 4);
                if self.runtime.check_damage_to_link(self.ram, k) {
                    self.ram.set_sprite_main_delay(k, 8);
                }
                return;
            }
            if delay != 1 {
                self.ram
                    .set_sprite_oam_flags(k, FLAGS[usize::from((delay >> 1) & 3)]);
                return;
            }
        }
        self.ram.set_sprite_state(k, 0);
        if let Some(j) = self.runtime.spawn_bomb(self.ram, k) {
            self.ram.set_sprite_state(j, 6);
            self.ram.set_sprite_scratch_c(j, 2);
            self.ram.set_sprite_oam_flags(j, 2);
            self.ram.set_sprite_flags4(j, 9);
            self.ram.set_sprite_aux1_delay(j, 31);
            self.ram.set_sprite_flags2(j, 3);
            let pan = self.runtime.calculate_sfx_pan(self.ram, k);
            self.ram.write_u8(0x12e, pan | 12);
        }
    }

    pub fn landmine_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 2] = [(0, 4, 0x0070, 0), (8, 4, 0x4070, 0)];
        self.runtime.oam_allocate_region_b(self.ram, 8);
        if self.ram.read_u8(0xfc6) < 3 {
            self.runtime.draw_multiple(self.ram, k, &P);
        }
    }

    pub fn sprite_d3_stal(&mut self, k: usize) {
        const GFX: [u8; 5] = [2, 2, 1, 0, 1];
        if self.ram.read_u8(0xfc6) < 3 {
            if self.ram.sprite(k).ai_state() == 0 {
                self.runtime.oam_allocate_region_b(self.ram, 4);
            }
            self.stal_draw(k);
        }
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_ignore_projectile(k, 1);
                if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
                    self.runtime.nullify_hookshot_drag(self.ram);
                    self.runtime.repel_dash(self.ram);
                    if self.ram.sprite(k).main_delay() == 0 {
                        self.ram.set_sprite_main_delay(k, 64);
                        self.runtime.sfx2_with_pan(self.ram, k, 0x22);
                    }
                }
                let delay = self.ram.sprite(k).main_delay();
                if delay != 0 {
                    if delay.wrapping_sub(1) != 0 {
                        self.ram.set_sprite_hit_timer(k, delay.wrapping_sub(1) | 64);
                    } else {
                        self.ram.set_sprite_ignore_projectile(k, 0);
                        self.ram.set_sprite_ai_state(k, 1);
                        self.ram.set_sprite_hit_timer(k, 0);
                        self.ram
                            .set_sprite_flags3(k, self.ram.sprite(k).flags3() & !0x40);
                        self.ram
                            .set_sprite_flags2(k, self.ram.sprite(k).flags2() & !0x80);
                    }
                }
            }
            1 => {
                self.runtime.check_damage_to_and_from_link(self.ram, k);
                self.runtime.move_xyz(self.ram, k);
                self.runtime.check_tile_collision(self.ram, k);
                self.ram
                    .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(2));
                if (self.ram.sprite(k).z() as i8) < 0 {
                    self.ram.set_sprite_z(k, 0);
                    self.ram.set_sprite_velocity_z(k, 16);
                    self.runtime.apply_speed_towards_link(self.ram, k, 12);
                }
                if self.ram.read_u8(0x1a) & 3 == 0 {
                    let mut s = self.ram.sprite(k).subtype2().wrapping_add(1);
                    if s == 5 {
                        s = 0;
                    }
                    self.ram.set_sprite_subtype2(k, s);
                }
                self.ram
                    .set_sprite_graphics(k, GFX[usize::from(self.ram.sprite(k).subtype2())]);
            }
            _ => {}
        }
    }

    pub fn stal_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 6] = [
            (0, 0, 0x0044, 2),
            (4, 11, 0x0070, 0),
            (0, 0, 0x0044, 2),
            (4, 12, 0x0070, 0),
            (0, 0, 0x0044, 2),
            (4, 13, 0x0070, 0),
        ];
        let n = if self.ram.sprite(k).ai_state() != 0 {
            2
        } else {
            1
        };
        let start = usize::from(self.ram.sprite(k).graphics()) * 2;
        self.runtime
            .draw_multiple(self.ram, k, &P[start..start + n]);
        if n == 2 {
            self.runtime.draw_shadow(self.ram, k);
        }
    }

    pub fn sprite_human_multi_1(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.sprite_flute_dad(k),
            1 => self.sprite_thief_hideout_guy(k),
            2 => self.sprite_blinds_hut_guy(k),
            _ => {}
        }
    }

    pub fn sprite_blinds_hut_guy(&mut self, k: usize) {
        self.blind_hideout_guy_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.runtime.track_body_to_head(self.ram, k);
        self.ram.set_sprite_head_direction(k, 0);
        let j = self.runtime.show_solicited_message(self.ram, k, 0x172);
        if j & 0x100 != 0 {
            self.ram.set_sprite_scratch_d(k, j as u8);
            self.ram.set_sprite_head_direction(k, j as u8);
        }
    }

    pub fn sprite_thief_hideout_guy(&mut self, k: usize) {
        if self.ram.read_u8(0x1a) & 3 == 0 {
            self.ram.set_sprite_graphics(k, 2);
            let d = self.runtime.direction_to_face_link(self.ram, k);
            self.ram
                .set_sprite_head_direction(k, if d == 3 { 2 } else { d });
        }
        self.ram.set_sprite_oam_flags(k, 15);
        self.runtime.oam_allocate_defer_to_player(self.ram, k);
        self.runtime.thief_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        self.runtime.show_solicited_message(self.ram, k, 0x171);
        self.ram.set_sprite_graphics(k, 2);
    }

    pub fn sprite_flute_dad(&mut self, k: usize) {
        self.flute_boy_father_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        let frame = self.ram.read_u8(0x1a);
        self.ram
            .set_sprite_graphics(k, if frame < 48 { 2 } else { (frame >> 7) & 1 });
        if self.ram.sprite(k).ai_state() != 0 {
            self.runtime.show_solicited_message(self.ram, k, 0xa3);
            self.ram.set_sprite_graphics(k, 2);
        } else if self.ram.read_u8(0xf34c) < 2 {
            self.runtime.show_solicited_message(self.ram, k, 0xa1);
        } else if self.runtime.show_solicited_message(self.ram, k, 0xa4) & 0x100 == 0
            && self.ram.read_u8(0x202) == 13
            && self.ram.read_u8(0xf0) & 0x40 != 0
            && self.runtime.check_damage_to_link_same_layer(self.ram, k)
        {
            self.runtime.show_unconditional_message(self.ram, 0xa2);
            self.ram.set_sprite_ai_state(k, 1);
            self.ram.set_sprite_graphics(k, 2);
        }
    }

    pub fn flute_boy_father_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 6] = [
            (0, -7, 0x0086, 2),
            (0, 0, 0x0088, 2),
            (0, -6, 0x0086, 2),
            (0, 0, 0x0088, 2),
            (0, -8, 0x0084, 2),
            (0, 0, 0x0088, 2),
        ];
        let s = usize::from(self.ram.sprite(k).graphics()) * 2;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &P[s..s + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn blind_hideout_guy_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 16] = [
            (0, -8, 0x000c, 2),
            (0, 0, 0x00ca, 2),
            (0, -8, 0x000c, 2),
            (0, 0, 0x40ca, 2),
            (0, -8, 0x000c, 2),
            (0, 0, 0x00ca, 2),
            (0, -8, 0x000c, 2),
            (0, 0, 0x40ca, 2),
            (0, -8, 0x000e, 2),
            (0, 0, 0x00ca, 2),
            (0, -8, 0x000e, 2),
            (0, 0, 0x40ca, 2),
            (0, -8, 0x400e, 2),
            (0, 0, 0x00ca, 2),
            (0, -8, 0x400e, 2),
            (0, 0, 0x40ca, 2),
        ];
        let s = usize::from(self.ram.sprite(k).graphics()) * 2
            + usize::from(self.ram.sprite(k).scratch_d()) * 4;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &P[s..s + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_sweeping_lady(&mut self, k: usize) {
        self.sweeping_lady_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.show_solicited_message(self.ram, k, 0xa5);
        self.runtime.behave_as_barrier(self.ram, k);
        self.ram
            .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 4) & 1);
    }

    pub fn sweeping_lady_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 4] = [
            (0, -7, 0x008e, 2),
            (0, 5, 0x008a, 2),
            (0, -8, 0x008e, 2),
            (0, 4, 0x008c, 2),
        ];
        let s = usize::from(self.ram.sprite(k).graphics()) * 2;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &P[s..s + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_lumberjacks(&mut self, k: usize) {
        const MSG: [u16; 4] = [0x12c, 0x12d, 0x12e, 0x12d];
        self.lumberjacks_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        if self.lumberjack_check_proximity(k, 0) {
            self.runtime.nullify_hookshot_drag(self.ram);
            self.ram.write_u8(0x5e, 0);
            self.runtime.link_cancel_dash(self.ram);
        }
        if !self.runtime.check_if_link_is_busy(self.ram)
            && self.lumberjack_check_proximity(k, 1)
            && self.ram.read_u8(0xf6) & 0x80 != 0
        {
            let i = usize::from(self.ram.read_u8(0x22) >= self.ram.sprite(k).x_low())
                + usize::from(self.ram.read_u8(0xf359) >= 2) * 2;
            self.runtime.show_unconditional_message(self.ram, MSG[i]);
        }
        self.ram
            .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 5) & 1);
    }

    pub fn lumberjack_check_proximity(&self, _k: usize, j: usize) -> bool {
        const X: [u16; 2] = [48, 52];
        const Y: [u16; 2] = [19, 20];
        const W: [u16; 2] = [98, 106];
        const H: [u16; 2] = [37, 40];
        let sx = self.ram.read_u16(0xfd8);
        let sy = self.ram.read_u16(0xfda);
        sx.wrapping_sub(self.ram.read_u16(0x22)).wrapping_add(X[j]) < W[j]
            && sy.wrapping_sub(self.ram.read_u16(0x20)).wrapping_add(Y[j]) < H[j]
    }

    pub fn lumberjacks_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 33] = [
            (-23, 5, 0x02be, 0),
            (-15, 5, 0x02bf, 0),
            (-7, 5, 0x02bf, 0),
            (1, 5, 0x02bf, 0),
            (9, 5, 0x02bf, 0),
            (17, 5, 0x02bf, 0),
            (25, 5, 0x42be, 0),
            (-32, -8, 0x40a8, 2),
            (-32, 4, 0x40a6, 2),
            (30, -8, 0x00a8, 2),
            (31, 4, 0x00a4, 2),
            (-19, 5, 0x02be, 0),
            (-11, 5, 0x02bf, 0),
            (-3, 5, 0x02bf, 0),
            (5, 5, 0x02bf, 0),
            (13, 5, 0x02bf, 0),
            (21, 5, 0x02bf, 0),
            (29, 5, 0x42be, 0),
            (-31, -8, 0x40a8, 2),
            (-32, 4, 0x40a4, 2),
            (31, -8, 0x00a8, 2),
            (31, 4, 0x00a6, 2),
            (-19, 5, 0x02be, 0),
            (-11, 5, 0x02bf, 0),
            (-3, 5, 0x02bf, 0),
            (5, 5, 0x02bf, 0),
            (13, 5, 0x02bf, 0),
            (21, 5, 0x02bf, 0),
            (29, 5, 0x42be, 0),
            (-32, -8, 0x400e, 2),
            (-32, 4, 0x40a4, 2),
            (32, -8, 0x000e, 2),
            (31, 4, 0x00a6, 2),
        ];
        let s = usize::from(self.ram.sprite(k).graphics()) * 11;
        self.runtime.draw_multiple(self.ram, k, &P[s..s + 11]);
    }

    pub fn sprite_fortune_teller(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => {
                self.fortune_teller_draw(k);
                if self.runtime.return_if_inactive(self.ram, k) {
                    return;
                }
                let dark = (self.ram.read_u8(0xf3ca) >> 6) & 1 != 0;
                self.runtime
                    .fortune_teller_light_or_dark_world(self.ram, k, dark);
            }
            1 => {
                if self.runtime.return_if_inactive(self.ram, k) {
                    return;
                }
                if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
                    self.runtime.nullify_hookshot_drag(self.ram);
                    self.ram.write_u8(0x5e, 0);
                    self.runtime.link_cancel_dash(self.ram);
                }
            }
            _ => {}
        }
    }

    pub fn fortune_teller_perform_pseudo_science(&mut self, k: usize) {
        const READINGS: [u16; 16] = [
            0xea, 0xeb, 0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb,
            0xfc, 0xfd,
        ];
        self.ram.set_sprite_graphics(k, 0);
        self.ram
            .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
        let mut slots = [0usize; 2];
        let mut n = 0usize;
        if self.ram.read_u8(0xf3c7) >= 3 {
            let checks = [
                (self.ram.read_u8(0xf34e) == 0, 2),
                (self.ram.read_u8(0xf374) & 2 == 0, 1),
                (self.ram.read_u8(0xf344) < 2, 3),
                (self.ram.read_u8(0xf356) == 0, 4),
                (self.ram.read_u8(0xf357) == 0, 5),
                (self.ram.read_u8(0xf3c5) < 3, 6),
                (self.ram.read_u8(0xf37b) == 0, 7),
                (self.ram.read_u8(0xf347) == 0, 8),
                (self.ram.read_u8(0xf3c9) & 0x10 == 0, 9),
                (self.ram.read_u8(0xf3c9) & 0x20 == 0, 10),
                (self.ram.read_u8(0xf352) == 0, 11),
                (self.ram.read_u8(0xf2db) & 2 == 0, 12),
                (self.ram.read_u8(0xf359) < 4, 13),
                (true, 14),
                (true, 15),
            ];
            for (condition, message) in checks {
                if condition {
                    slots[n] = message;
                    n += 1;
                    if n == 2 {
                        break;
                    }
                }
            }
        }
        let flags = self.ram.read_u8(0xf3c6) ^ 0x40;
        self.ram.write_u8(0xf3c6, flags);
        self.runtime
            .show_unconditional_message(self.ram, READINGS[slots[usize::from(flags & 0x40 != 0)]]);
    }

    pub fn fortune_teller_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 12] = [
            (0, -48, 0x000c, 2),
            (0, -32, 0x002c, 0),
            (8, -32, 0x402c, 0),
            (0, -48, 0x000a, 2),
            (0, -32, 0x002a, 0),
            (8, -32, 0x402a, 0),
            (-4, -40, 0x0066, 2),
            (4, -40, 0x4066, 2),
            (-4, -40, 0x0066, 2),
            (-4, -40, 0x0068, 2),
            (4, -40, 0x4068, 2),
            (-4, -40, 0x0068, 2),
        ];
        let j =
            usize::from((self.ram.read_u8(0xf3ca) >> 6 & 1) * 2 + self.ram.sprite(k).graphics());
        self.runtime
            .draw_multiple(self.ram, k, &P[j * 3..j * 3 + 3]);
    }

    pub fn sprite_10_octoballoon_baby(&mut self, k: usize) {
        let subtype = self.ram.sprite(k).subtype2();
        if subtype == 0 {
            self.ram.set_sprite_state(k, 0);
        }
        if subtype >= 64 || subtype & 1 == 0 {
            self.runtime.draw_single_small(self.ram, k);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.set_sprite_subtype2(k, subtype.wrapping_sub(1));
        if self.runtime.return_if_recoiling(self.ram, k) {
            return;
        }
        self.ram
            .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(1));
        self.runtime.move_z(self.ram, k);
        if (self.ram.sprite(k).z() as i8) < 0 {
            self.ram.set_sprite_z(k, 0);
            self.ram.set_sprite_velocity_z(k, 16);
        }
        self.runtime.move_xy(self.ram, k);
        self.runtime.check_tile_collision2(self.ram, k);
        self.runtime.bounce_off_wall(self.ram, k);
        self.runtime.check_damage_to_and_from_link(self.ram, k);
    }

    pub fn buzzblob_select_new_direction(&mut self, k: usize) {
        const XV: [i8; 8] = [3, 2, -2, -3, -2, 2, 0, 0];
        const YV: [i8; 8] = [0, 2, 2, 0, -2, -2, 0, 0];
        const DELAY: [u8; 8] = [48, 48, 48, 48, 48, 48, 64, 64];
        let i = usize::from(self.runtime.random_number(self.ram) & 7);
        self.ram.set_sprite_velocity_x(k, XV[i] as u8);
        self.ram.set_sprite_velocity_y(k, YV[i] as u8);
        self.ram.set_sprite_main_delay(k, DELAY[i]);
    }

    pub fn sprite_0c_octorok_stone(&mut self, k: usize) {
        if self.ram.sprite(k).state() == 6 {
            self.sprite_draw_octorok_stone_crumbling(k);
            if self.runtime.return_if_paused(self.ram, k) {
                return;
            }
            if self.ram.sprite(k).main_delay() == 30 {
                self.runtime.sfx2_with_pan(self.ram, k, 0x1f);
            }
        } else {
            self.runtime.draw_single_large(self.ram, k);
            if self.runtime.return_if_inactive(self.ram, k) {
                return;
            }
            self.runtime.check_damage_to_link(self.ram, k);
            self.runtime.move_xy(self.ram, k);
            if (k as u8 ^ self.ram.read_u8(0x1a)) & 3 == 0
                && self.runtime.check_tile_collision(self.ram, k)
            {
                self.runtime.sprite_func3(self.ram, k);
            }
        }
    }

    pub fn sprite_draw_octorok_stone_crumbling(&mut self, k: usize) {
        const X: [i8; 16] = [
            0, 8, 0, 8, -8, 16, -8, 16, -12, 20, -12, 20, -14, 22, -14, 22,
        ];
        const Y: [i8; 16] = [
            0, 0, 8, 8, -8, -8, 16, 16, -12, -12, 20, 20, -14, -14, 22, 22,
        ];
        const FL: [u8; 16] = [
            0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
        ];
        let Some((x, y, flags)) = self.runtime.prep_oam_with_flags_wide(self.ram, k) else {
            return;
        };
        let oam_pointer = self.ram.read_u16(0x90);
        let extended_pointer = self.ram.read_u16(0x92);
        let g = usize::from((self.ram.sprite(k).main_delay() >> 1 & 0xc) ^ 0xc);
        for i in (0..4).rev() {
            let j = g + i;
            self.runtime.emit_oam_wide(
                self.ram,
                k,
                x.wrapping_add_signed(i16::from(X[j])),
                y.wrapping_add_signed(i16::from(Y[j])),
                0xbc,
                FL[j] | 0x2d | flags,
            );
        }
        self.ram.write_u16(0x90, oam_pointer);
        self.ram.write_u16(0x92, extended_pointer);
    }

    pub fn octoballoon_recoil_link(&mut self, k: usize) {
        if self.ram.read_u8(0x46) == 0 {
            self.ram.write_u8(0x46, 4);
            self.runtime.apply_recoil_to_link(self.ram, k, 16);
            self.runtime.invert_speed_xy(self.ram, k);
        }
    }

    pub fn octoballoon_form_babby(&mut self, k: usize) {
        const XV: [i8; 6] = [16, 11, -11, -16, -11, 11];
        const YV: [i8; 6] = [0, 11, 11, 0, -11, -11];
        self.runtime.sfx2_with_pan(self.ram, k, 12);
        for i in (0..6).rev() {
            if let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x10) {
                self.runtime.set_spawned_coordinates(self.ram, j);
                self.ram.set_sprite_velocity_x(j, XV[i] as u8);
                self.ram.set_sprite_velocity_y(j, YV[i] as u8);
                self.ram.set_sprite_velocity_z(j, 48);
                self.ram.set_sprite_subtype2(j, 255);
            }
        }
    }

    pub fn fairy_check_if_touchable(&mut self, k: usize) {
        if self.ram.read_u8(0x11) == 2 {
            let message = self.ram.read_u16(0x1cf0);
            if message == 0xc9 || message == 0xca {
                self.ram.set_sprite_aux4_delay(k, 40);
            }
        }
    }

    pub fn sprite_73_uncle_and_priest(&mut self, k: usize) {
        match self.ram.sprite(k).scratch_e() {
            0 => self.runtime.sprite_uncle(self.ram, k),
            1 => self.runtime.sprite_priest(self.ram, k),
            2 => self.runtime.sprite_sanctuary_mantle(self.ram, k),
            _ => {}
        }
    }

    pub fn sprite_tutorial_guard_or_barrier(&mut self, k: usize) {
        if self.ram.sprite(k).kind() == 0x40 {
            self.runtime.sprite_evil_barrier(self.ram, k);
            return;
        }
        const LOCK: [u8; 4] = [3, 2, 0, 1];
        const GFX: [u8; 4] = [2, 1, 0, 3];
        let old = self.ram.sprite(k).scratch_d();
        if self.ram.sprite(k).aux1_delay() != 0 {
            self.ram.set_sprite_scratch_d(k, LOCK[usize::from(old)]);
        }
        self.ram
            .set_sprite_graphics(k, GFX[usize::from(self.ram.sprite(k).scratch_d())]);
        self.runtime.tutorial_soldier_draw(self.ram, k);
        self.ram.set_sprite_scratch_d(k, old);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.check_damage_from_link(self.ram, k);
        let y = self.ram.sprite(k).y_low();
        if self.ram.read_u8(0x40a) == 0x1b && (y == 0x50 || y == 0x90) {
            self.runtime.tutorial_guard_show_message_on_contact(
                self.ram,
                k,
                if y == 0x50 { 0xb2 } else { 0xb3 },
            );
        } else {
            let v = self.ram.read_u8(0xb69);
            if self
                .runtime
                .tutorial_guard_show_message_on_contact(self.ram, k, u16::from(v) + 0xf)
            {
                self.ram.write_u8(0xb69, if v != 6 { v + 1 } else { 0 });
            }
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        if (k as u8 ^ self.ram.read_u8(0x1a)) & 0x1f == 0 {
            let d = self.runtime.direction_to_face_link(self.ram, k);
            self.ram.set_sprite_scratch_d(k, d);
            if d != old && (d ^ old) & 2 == 0 {
                self.ram.set_sprite_aux1_delay(k, 12);
            }
        }
    }

    pub fn sprite_f2_medallion_tablet(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.runtime.medallion_tablet_main(self.ram, k),
            1 => self.runtime.sprite_dust_cloud(self.ram, k),
            _ => {}
        }
    }

    pub fn sprite_33_rupee_pull(&mut self, k: usize) {
        let _ = self.runtime.prep_oam(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.check_if_link_is_busy(self.ram)
        {
            return;
        }
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            self.ram.write_u8(0x3f8, 1);
            self.ram.set_sprite_scratch_a(k, 1);
        } else if self.ram.sprite(k).scratch_a() != 0 {
            self.ram.write_u8(0x3f8, 0);
            if self.ram.read_u8(0x308) & 1 != 0 {
                self.ram.set_sprite_state(k, 0);
                self.runtime.rupee_pull_spawn_prize(self.ram, k);
                self.runtime.spawn_poof_garnish(self.ram, k);
            }
        }
    }

    pub fn sprite_14_thieves_town_grate(&mut self, k: usize) {
        let _ = self.runtime.prep_oam(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.check_if_link_is_busy(self.ram)
        {
            return;
        }
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            self.ram.write_u8(0x3f8, 1);
            self.ram.set_sprite_scratch_a(k, 1);
            return;
        }
        if self.ram.sprite(k).scratch_a() == 0 {
            return;
        }
        self.ram.write_u8(0x3f8, 0);
        if self.ram.read_u8(0x308) & 1 == 0 {
            return;
        }
        self.runtime.sfx2_with_pan(self.ram, k, 0x1f);
        self.runtime.open_gargoyles_domain(self.ram);
        if let Some(j) = self.runtime.spawn_dust_cloud(self.ram, k) {
            let x = u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()]);
            let y = u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()]);
            self.set_sprite_x(j, x);
            self.set_sprite_y(j, y);
        }
        self.ram.set_sprite_state(k, 0);
    }

    pub fn sprite_25_talking_tree(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.runtime.talking_tree_mouth(self.ram, k),
            1 => self.runtime.talking_tree_eye(self.ram, k),
            _ => {}
        }
    }

    pub fn sprite_3a_magic_bat(&mut self, k: usize) {
        if self.ram.sprite(k).head_direction() != 0 {
            self.runtime.sprite_mad_batter_bolt(self.ram, k);
            return;
        }
        if self.ram.sprite(k).ai_state() != 0 {
            self.runtime.draw_single_large(self.ram, k);
        }
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.move_xy(self.ram, k);
        self.runtime.move_z(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                if self.ram.read_u8(0xf37b) >= 2
                    || !self.runtime.check_damage_to_link_same_layer(self.ram, k)
                {
                    return;
                }
                for i in (0..5).rev() {
                    if self.ram.read_u8(0xc4a + i) == 0x1a {
                        self.sprite_spawn_superficial_bomb_blast(k);
                        let pan = self.runtime.calculate_sfx_pan(self.ram, k);
                        self.ram.write_u8(0x12d, pan | 0xd);
                        self.ram.set_sprite_ai_state(k, 1);
                        self.ram.set_sprite_scratch_a(k, 20);
                        self.ram.write_u8(0x2e4, 1);
                        self.ram
                            .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() | 32);
                        return;
                    }
                }
            }
            1 => {
                if self.ram.sprite(k).main_delay() == 0 {
                    let a = self.ram.sprite(k).scratch_a().wrapping_sub(1);
                    self.ram.set_sprite_scratch_a(k, a);
                    self.ram.set_sprite_main_delay(k, a);
                    if a != 1 {
                        self.ram.set_sprite_velocity_z(k, a >> 2);
                        let accel = [(-8i8) as u8, 7][usize::from(a & 1)];
                        self.ram.set_sprite_velocity_x(
                            k,
                            self.ram.sprite(k).velocity_x().wrapping_add(accel),
                        );
                        self.ram
                            .set_sprite_graphics(k, self.ram.sprite(k).graphics() ^ 1);
                    } else {
                        self.runtime.show_unconditional_message(self.ram, 0x110);
                        self.ram.set_sprite_ai_state(k, 2);
                        self.ram.set_sprite_graphics(k, 0);
                        self.ram.set_sprite_velocity_z(k, 0);
                        self.ram.set_sprite_velocity_x(k, 0);
                        self.ram.set_sprite_main_delay(k, 255);
                    }
                }
            }
            2 => {
                let d = self.ram.sprite(k).main_delay();
                if d == 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                    self.ram.set_sprite_aux1_delay(k, 64);
                }
                const FL: [u8; 8] = [0xa, 4, 2, 4, 2, 0xa, 4, 2];
                self.ram.set_sprite_oam_flags(
                    k,
                    (self.ram.sprite(k).oam_flags() & !0xe) | FL[usize::from(d >> 1 & 7)],
                );
                if d == 240 {
                    self.sprite_magic_bat_spawn_lightning(k);
                }
            }
            3 => {
                let d = self.ram.sprite(k).aux1_delay();
                if d == 0 {
                    self.runtime.show_unconditional_message(self.ram, 0x111);
                    self.runtime.palette_restore_bg_and_hud(self.ram);
                    self.ram
                        .write_u8(0x15, self.ram.read_u8(0x15).wrapping_add(1));
                    self.ram.set_sprite_ai_state(k, 4);
                    self.ram.write_u8(0xf37b, 1);
                    self.runtime.hud_refresh_icon(self.ram);
                } else if d == 0x10 {
                    self.ram.write_u8(0xff9, 0x10);
                }
            }
            4 => {
                self.sprite_spawn_dummy_death_animation(k);
                self.ram.set_sprite_state(k, 0);
                self.ram.write_u8(0x2e4, 0);
            }
            _ => {}
        }
    }

    pub fn running_boy_spawn_dust_garnish(&mut self, k: usize) {
        let counter = self.ram.sprite(k).death_action().wrapping_add(1);
        self.ram.set_sprite_death_action(k, counter);
        if counter & 0x0f != 0 {
            return;
        }
        let j = self.runtime.garnish_alloc_force(self.ram);
        self.ram.set_garnish_kind(j, 20);
        self.ram.set_garnish_active_marker(20);
        let x = u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()])
            .wrapping_add(4);
        let y = u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()])
            .wrapping_add(28);
        self.ram.set_garnish_x_low(j, x as u8);
        self.ram.set_garnish_x_high(j, (x >> 8) as u8);
        self.ram.set_garnish_y_low(j, y as u8);
        self.ram.set_garnish_y_high(j, (y >> 8) as u8);
        self.ram.set_garnish_countdown(j, 10);
    }

    pub fn sprite_maze_game_lady(&mut self, k: usize) {
        self.lady_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.track_body_to_head(self.ram, k);
        let head = self.runtime.direction_to_face_link(self.ram, k) ^ 3;
        self.ram.set_sprite_head_direction(k, head);
        match self.ram.sprite(k).ai_state() {
            0 => {
                let message = if self.ram.sprite(k).x_low() < self.ram.read_u8(0x22) {
                    0xcc
                } else {
                    0xd0
                };
                let result = self.runtime.show_message_on_contact(self.ram, k, message);
                if message == 0xcc && result & 0x100 != 0 {
                    self.ram.set_sprite_scratch_d(k, result as u8);
                    self.ram.set_sprite_head_direction(k, result as u8);
                    self.ram.set_sprite_ai_state(k, 1);
                    self.ram.write_u16(0x1fe00, 0);
                    self.ram.write_u16(0x1fe02, 0);
                    self.ram.set_sprite_scratch_a(k, 0);
                    self.ram.write_u8(0xabf, 0);
                }
            }
            1 => {
                self.runtime.sfx3_with_pan(self.ram, k, 7);
                self.ram.set_sprite_ai_state(k, 2);
            }
            2 => {
                let a = self.ram.sprite(k).scratch_a().wrapping_add(1);
                self.ram.set_sprite_scratch_a(k, a);
                if a == 63 {
                    self.ram.set_sprite_scratch_a(k, 0);
                    let lo = self.ram.read_u16(0x1fe00).wrapping_add(1);
                    self.ram.write_u16(0x1fe00, lo);
                    if lo == 0 {
                        let hi = self.ram.read_u16(0x1fe02).wrapping_add(1);
                        self.ram.write_u16(0x1fe02, hi);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn sprite_maze_game_guy(&mut self, k: usize) {
        self.maze_game_guy_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.track_body_to_head(self.ram, k);
        self.ram.set_sprite_head_direction(k, 0);
        self.runtime.behave_as_barrier(self.ram, k);
        self.ram
            .set_sprite_graphics(k, self.ram.read_u8(0x1a) >> 3 & 1);
        if self.ram.read_u8(0xabf) != 0 {
            self.runtime.show_message_on_contact(self.ram, k, 0xd0);
            return;
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                let elapsed = self.ram.read_u16(0x1fe00);
                self.ram.write_u16(0x1fe04, elapsed);
                self.ram.write_u16(0x1fe06, self.ram.read_u16(0x1fe02));
                let mut t = elapsed % 6000;
                let a = t / 600;
                t %= 600;
                let b = t / 60;
                t %= 60;
                let c = t / 10;
                t %= 10;
                self.ram.write_u8(0x1cf2, t as u8 | (c as u8) << 4);
                self.ram.write_u8(0x1cf3, b as u8 | (a as u8) << 4);
                let result = self.runtime.show_message_on_contact(self.ram, k, 0xcb);
                if result & 0x100 != 0 {
                    self.ram.set_sprite_scratch_d(k, result as u8);
                    self.ram.set_sprite_head_direction(k, result as u8);
                    self.ram.set_sprite_ai_state(k, 1);
                }
            }
            1 => {
                let (message, state) = if self
                    .ram
                    .read_u8(0xf280 + usize::from(self.ram.read_u8(0x8a)))
                    & 0x40
                    != 0
                {
                    (0xcf, 4)
                } else if self.ram.read_u16(0x1fe04) < 16 {
                    (0xcd, 3)
                } else {
                    (0xce, 2)
                };
                self.runtime.show_unconditional_message(self.ram, message);
                self.ram.set_sprite_ai_state(k, state);
                if state != 4 {
                    let handler = self.ram.read_u8(0x5d);
                    self.ram.set_sprite_scratch_d(k, handler);
                    self.ram.set_sprite_head_direction(k, handler);
                }
            }
            state @ 2..=4 => {
                let message = [0, 0, 0xce, 0xcd, 0xcf][usize::from(state)];
                let result = if state == 2 {
                    self.runtime.show_message_on_contact(self.ram, k, message)
                } else {
                    self.runtime.show_solicited_message(self.ram, k, message)
                };
                if result & 0x100 != 0 {
                    self.ram.set_sprite_scratch_d(k, result as u8);
                    self.ram.set_sprite_head_direction(k, result as u8);
                }
            }
            _ => {}
        }
    }

    pub fn maze_game_guy_draw(&mut self, k: usize) {
        const D: [(i8, i8, u16, u8); 16] = [
            (0, -10, 0x0000, 2),
            (0, 0, 0x0020, 2),
            (0, -10, 0x0000, 2),
            (0, 0, 0x0020, 2),
            (0, -10, 0x0000, 2),
            (0, 0, 0x0020, 2),
            (0, -10, 0x0000, 2),
            (0, 0, 0x0020, 2),
            (0, -10, 0x4002, 2),
            (0, 0, 0x0020, 2),
            (0, -10, 0x4002, 2),
            (0, 0, 0x0020, 2),
            (0, -10, 0x0002, 2),
            (0, 0, 0x0020, 2),
            (0, -10, 0x0002, 2),
            (0, 0, 0x0020, 2),
        ];
        let start = usize::from(self.ram.sprite(k).graphics()) * 2
            + usize::from(self.ram.sprite(k).scratch_d()) * 4;
        self.runtime
            .draw_multiple_player_deferred(self.ram, k, &D[start..start + 2]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_39_locksmith(&mut self, k: usize) {
        self.middle_aged_man_draw(k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.behave_as_barrier(self.ram, k);
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.runtime.show_solicited_message(self.ram, k, 0x107);
                let old_x = self.ram.sprite(k).x_low();
                self.ram.set_sprite_x_low(k, old_x.wrapping_sub(16));
                self.runtime.get_16bit_coords(self.ram, k);
                self.ram.set_sprite_velocity_x(k, 1);
                self.ram.set_sprite_velocity_y(k, 1);
                if !self.runtime.check_tile_collision(self.ram, k) {
                    self.ram.set_sprite_ai_state(k, 1);
                    if self.ram.read_u8(0xf3cc) != 0 {
                        self.ram.set_sprite_ai_state(k, 5);
                    }
                }
                self.ram.set_sprite_x_low(k, old_x);
            }
            1 => {
                self.ram.write_u8(0xf3cc, 9);
                self.ram.write_u8(0x2f9, 0);
                self.runtime.load_follower_graphics(self.ram);
                self.runtime.follower_initialize(self.ram);
                self.ram.write_u16(0x2cd, 0x40);
                self.ram.set_sprite_state(k, 0);
            }
            2 => {
                if self.runtime.check_if_link_is_busy(self.ram) {
                    return;
                }
                let result = if self.ram.read_u8(0xf3d3) != 0 {
                    self.runtime.show_solicited_message(self.ram, k, 0x109)
                } else {
                    self.runtime.show_message_on_contact(self.ram, k, 0x109)
                };
                if result & 0x100 != 0 {
                    self.ram.set_sprite_ai_state(k, 3);
                }
            }
            3 => {
                if self.ram.read_u8(0x1ce8) == 0 {
                    if self.ram.read_u8(0xf3d3) != 0 {
                        self.runtime.show_unconditional_message(self.ram, 0x10c);
                        self.ram.set_sprite_ai_state(k, 2);
                    } else {
                        self.ram.write_u8(0x2e9, 0);
                        self.runtime.link_receive_item(self.ram, 0x16, 0);
                        self.ram.write_u8(0xf3c9, self.ram.read_u8(0xf3c9) | 0x10);
                        self.ram.set_sprite_ai_state(k, 4);
                        self.ram.write_u8(0xf3cc, 0);
                    }
                } else {
                    self.runtime.show_unconditional_message(self.ram, 0x10a);
                    self.ram.set_sprite_ai_state(k, 2);
                }
            }
            4 => {
                self.runtime.show_solicited_message(self.ram, k, 0x10b);
            }
            5 => {
                self.runtime.show_solicited_message(self.ram, k, 0x107);
            }
            _ => {}
        }
    }

    pub fn middle_aged_man_draw(&mut self, k: usize) {
        const P: [(i8, i8, u16, u8); 2] = [(0, -8, 0x00ea, 2), (0, 0, 0x00ec, 2)];
        self.runtime.draw_multiple_player_deferred(self.ram, k, &P);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn sprite_38_eye_statue(&mut self, k: usize) {
        if self.ram.sprite(k).scratch_b() != 0 {
            return;
        }
        if self.runtime.prep_oam(self.ram, k).is_none()
            || self.runtime.return_if_inactive(self.ram, k)
        {
            return;
        }
        if self.runtime.direction_to_face_link(self.ram, k) == 2
            && self.ram.sprite(k).unknown2() == 9
        {
            self.ram
                .write_u8(0x642, self.ram.read_u8(0x642).wrapping_add(1));
            self.ram.set_sprite_scratch_b(k, 1);
        }
    }

    pub fn sprite_2b_hobo(&mut self, k: usize) {
        match self.ram.sprite(k).subtype2() {
            0 => self.sprite_hobo_bum(k),
            1 => self.sprite_hobo_bubble(k),
            2 => self.sprite_hobo_fire(k),
            3 => self.sprite_hobo_smoke(k),
            _ => {}
        }
    }

    pub fn sprite_hobo_bum(&mut self, k: usize) {
        self.runtime.hobo_draw(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram.set_sprite_flags4(k, 3);
        if self.runtime.check_damage_to_link_same_layer(self.ram, k) {
            self.runtime.nullify_hookshot_drag(self.ram);
            self.ram.write_u8(0x5e, 0);
            self.runtime.link_cancel_dash(self.ram);
        }
        match self.ram.sprite(k).ai_state() {
            0 => {
                self.ram.set_sprite_flags4(k, 7);
                if self.runtime.check_damage_to_link_same_layer(self.ram, k)
                    && self.ram.read_u8(0xf6) & 0x80 != 0
                {
                    self.ram.set_sprite_ai_state(k, 1);
                    let bubble = usize::from(self.ram.sprite(k).scratch_e());
                    self.ram.set_sprite_main_delay(bubble, 4);
                    self.ram.write_u8(0x2e4, 1);
                }
                if self.ram.sprite(k).aux2_delay() == 0 {
                    self.ram.set_sprite_aux2_delay(k, 160);
                    let bubble = self.hobo_spawn_bubble(k).map_or(255, |j| j as u8);
                    self.ram.set_sprite_scratch_e(k, bubble);
                }
            }
            1 if self.ram.sprite(k).main_delay() == 0 => {
                const GFX: [u8; 7] = [0, 1, 0, 1, 0, 1, 2];
                const DELAY: [u8; 7] = [6, 2, 6, 6, 2, 100, 30];
                let a = self.ram.sprite(k).scratch_a();
                if a != 7 {
                    self.ram.set_sprite_graphics(k, GFX[usize::from(a)]);
                    self.ram.set_sprite_main_delay(k, DELAY[usize::from(a)]);
                    self.ram.set_sprite_scratch_a(k, a.wrapping_add(1));
                } else {
                    self.runtime.show_unconditional_message(self.ram, 0xd7);
                    self.ram.set_sprite_ai_state(k, 2);
                }
            }
            2 => {
                self.ram.set_sprite_ai_state(k, 3);
                self.ram.set_sprite_graphics(k, 1);
                let screen = usize::from(self.ram.read_u8(0x8a));
                self.ram
                    .write_u8(0xf280 + screen, self.ram.read_u8(0xf280 + screen) | 0x20);
                self.ram.write_u8(0x2e9, 0);
                self.runtime.link_receive_item(self.ram, 0x16, 0);
                self.ram.write_u8(0xf3c9, self.ram.read_u8(0xf3c9) | 1);
            }
            3 => {
                self.ram.write_u8(0x2e4, 0);
                self.ram.set_sprite_graphics(k, 0);
                if self.ram.sprite(k).main_delay() == 0 {
                    self.ram.set_sprite_main_delay(k, 160);
                    self.hobo_spawn_bubble(k);
                }
            }
            _ => {}
        }
    }

    pub fn sprite_hobo_bubble(&mut self, k: usize) {
        self.runtime.oam_allocate_region_c(self.ram, 4);
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram
            .set_sprite_graphics(k, ((self.ram.read_u8(0x1a) >> 4) & 1) + 2);
        if self.ram.sprite(k).aux1_delay() == 0 {
            self.ram
                .set_sprite_graphics(k, self.ram.sprite(k).graphics().wrapping_add(1));
            self.runtime.move_z(self.ram, k);
            if self.ram.sprite(k).main_delay() == 0 {
                self.ram.set_sprite_state(k, 0);
            }
        }
        if self.ram.sprite(k).main_delay() < 4 {
            self.ram.set_sprite_graphics(k, 3);
        }
    }

    pub fn hobo_spawn_bubble(&mut self, k: usize) -> Option<usize> {
        let j = self.runtime.spawn_dynamic(self.ram, k, 0x2b)?;
        self.runtime.set_spawned_coordinates(self.ram, j);
        self.ram.set_sprite_subtype2(j, 1);
        self.ram.set_sprite_velocity_z(j, 2);
        self.ram.set_sprite_main_delay(j, 96);
        self.ram.set_sprite_aux1_delay(j, 48);
        self.ram.set_sprite_ignore_projectile(j, 48);
        self.ram.set_sprite_flags2(j, 0);
        Some(j)
    }

    pub fn sprite_hobo_fire(&mut self, k: usize) {
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.ram
            .set_sprite_graphics(k, (self.ram.read_u8(0x1a) >> 3) & 1);
        self.ram
            .set_sprite_oam_flags(k, self.ram.sprite(k).oam_flags() & !0x40);
        if self.ram.sprite(k).main_delay() == 0 {
            self.hobo_spawn_smoke(k);
            self.ram.set_sprite_main_delay(k, 47);
        }
    }

    pub fn sprite_hobo_smoke(&mut self, k: usize) {
        const FLAGS: [u8; 4] = [0, 64, 128, 192];
        self.ram.set_sprite_graphics(k, 6);
        self.runtime.draw_single_small(self.ram, k);
        if self.runtime.return_if_inactive(self.ram, k) {
            return;
        }
        self.runtime.move_xy(self.ram, k);
        self.runtime.move_z(self.ram, k);
        let flag = FLAGS[usize::from((self.ram.read_u8(0x1a) >> 4) & 3)];
        self.ram
            .set_sprite_oam_flags(k, (self.ram.sprite(k).oam_flags() & 0x3f) | flag);
        if self.ram.sprite(k).main_delay() == 0 {
            self.ram.set_sprite_state(k, 0);
        }
    }

    pub fn hobo_spawn_smoke(&mut self, k: usize) {
        let Some(j) = self.runtime.spawn_dynamic(self.ram, k, 0x2b) else {
            return;
        };
        self.runtime.set_spawned_coordinates(self.ram, j);
        let y = u16::from_le_bytes([self.ram.sprite(j).y_low(), self.ram.sprite(j).y_high()])
            .wrapping_sub(4);
        self.set_sprite_y(j, y);
        self.ram.set_sprite_subtype2(j, 3);
        self.ram.set_sprite_velocity_z(j, 7);
        self.ram.set_sprite_main_delay(j, 96);
        self.ram.set_sprite_ignore_projectile(j, 96);
        self.ram.set_sprite_flags2(j, 0);
    }

    pub fn sprite_22_ropa(&mut self, k: usize) {
        self.ropa_draw(k);
        if self.runtime.return_if_inactive(self.ram, k)
            || self.runtime.return_if_recoiling(self.ram, k)
        {
            return;
        }
        self.runtime.check_damage_to_and_from_link(self.ram, k);
        let subtype = self.ram.sprite(k).subtype2().wrapping_add(1);
        self.ram.set_sprite_subtype2(k, subtype);
        self.ram.set_sprite_graphics(k, (subtype >> 3) & 3);
        match self.ram.sprite(k).ai_state() {
            0 if self.ram.sprite(k).main_delay() == 0 => {
                self.runtime.apply_speed_towards_link(self.ram, k, 16);
                let z_velocity = (self.runtime.random_number(self.ram) & 15).wrapping_add(20);
                self.ram.set_sprite_velocity_z(k, z_velocity);
                self.ram.set_sprite_ai_state(k, 1);
            }
            1 => {
                self.runtime.move_xy(self.ram, k);
                if self.runtime.check_tile_collision(self.ram, k) {
                    self.ram.set_sprite_velocity_x(k, 0);
                    self.ram.set_sprite_velocity_y(k, 0);
                }
                self.runtime.move_z(self.ram, k);
                self.ram
                    .set_sprite_velocity_z(k, self.ram.sprite(k).velocity_z().wrapping_sub(2));
                if (self.ram.sprite(k).z() as i8) < 0 {
                    self.ram.set_sprite_z(k, 0);
                    self.ram.set_sprite_main_delay(k, 48);
                    self.ram.set_sprite_ai_state(k, 0);
                }
            }
            _ => {}
        }
    }

    pub fn ropa_draw(&mut self, k: usize) {
        const PIECES: [(i8, i8, u16, u8); 12] = [
            (0, -8, 0x0026, 0),
            (8, -8, 0x0027, 0),
            (0, 0, 0x0008, 2),
            (0, -8, 0x0036, 0),
            (8, -8, 0x0037, 0),
            (0, 0, 0x000a, 2),
            (0, -8, 0x4027, 0),
            (8, -8, 0x4026, 0),
            (0, 0, 0x4008, 2),
            (0, -8, 0x4037, 0),
            (8, -8, 0x4036, 0),
            (0, 0, 0x4008, 2),
        ];
        let start = usize::from(self.ram.sprite(k).graphics()) * 3;
        self.runtime
            .draw_multiple(self.ram, k, &PIECES[start..start + 3]);
        self.runtime.draw_shadow(self.ram, k);
    }

    pub fn hinox_throw_bomb(&mut self, _k: usize) {}

    pub fn hinox_face_link(&mut self, k: usize) {
        let direction = self.runtime.direction_to_face_link(self.ram, k);
        self.hinox_set_direction(k, direction);
        self.ram
            .set_sprite_velocity_x(k, self.ram.sprite(k).velocity_x() << 1);
        self.ram
            .set_sprite_velocity_y(k, self.ram.sprite(k).velocity_y() << 1);
    }

    pub fn hinox_set_direction(&mut self, k: usize, direction: u8) {
        const X_VELOCITY: [i8; 4] = [8, -8, 0, 0];
        const Y_VELOCITY: [i8; 4] = [0, 0, 8, -8];
        self.ram.set_sprite_scratch_d(k, direction);
        let delay = (self.runtime.random_number(self.ram) & 63).wrapping_add(96);
        self.ram.set_sprite_main_delay(k, delay);
        self.ram
            .set_sprite_ai_state(k, self.ram.sprite(k).ai_state().wrapping_add(1));
        let index = usize::from(direction);
        self.ram.set_sprite_velocity_x(k, X_VELOCITY[index] as u8);
        self.ram.set_sprite_velocity_y(k, Y_VELOCITY[index] as u8);
    }

    pub fn sprite_module_initialize(&mut self, k: usize) {
        self.runtime.load_properties(self.ram, k);
        self.ram
            .set_sprite_state(k, self.ram.sprite(k).state().wrapping_add(1));
        self.sprite_module_initialize_after_properties(k);
    }

    /// Dispatch the bank-1 portion of the prep table after the common property
    /// load and state increment have already been performed by the caller.
    pub fn sprite_module_initialize_after_properties(&mut self, k: usize) {
        let kind = self.ram.sprite(k).kind();
        match kind {
            0x00 => self.sprite_prep_raven(k),
            0x01 => self.sprite_prep_vulture(k),
            0x02 | 0x05 | 0x0b | 0x0c | 0x0d | 0x0e | 0x10 | 0x11 | 0x12 | 0x17 | 0x1b | 0x20
            | 0x22 | 0x27 | 0x38 | 0x3e | 0x3f | 0x50 | 0x58 | 0x61 | 0x6a | 0x6b | 0x6c | 0x94
            | 0x99 | 0x9b | 0xae | 0xaf | 0xb0 | 0xb1 | 0xb8 | 0xbf | 0xc2 | 0xc4 | 0xd0 | 0xd1
            | 0xed | 0xef | 0xf0 | 0xf1 => self.sprite_prep_do_nothing_a(k),
            0x03 => {}
            0x04 | 0x06 => self.sprite_prep_switch(k),
            0x07 => self.sprite_prep_switch_facing_up(k),
            0x08 | 0x0a => self.sprite_prep_octorok(k),
            0x09 => self.sprite_prep_moldorm(k),
            0x0f => self.sprite_prep_octoballoon(k),
            0x13 => self.sprite_prep_mini_helmasaur(k),
            0x14 => self.sprite_prep_thieves_town_grate(k),
            0x15 | 0x77 => self.sprite_prep_antifairy(k),
            0x16 => self.sprite_prep_sage(k),
            0x18 => self.sprite_prep_mini_moldorm_bounce(k),
            0x19 => self.sprite_prep_poe(k),
            0x1a => self.sprite_prep_smithy(k),
            0x1c => self.sprite_prep_statue(k),
            0x1d | 0x2a | 0x2d | 0x2f | 0x30 | 0x32 | 0x36 | 0x37 | 0x3c | 0x72 | 0x75 | 0xbc
            | 0xc5 | 0xc6 | 0xd2 | 0xd4 => self.sprite_prep_ignore_projectiles(k),
            0x1e => self.sprite_prep_crystal_switch(k),
            0x1f => self.sprite_prep_sick_kid(k),
            0x21 => self.sprite_prep_water_lever(k),
            0x23 | 0x24 => self.sprite_prep_bari(k),
            0x25 => self.sprite_prep_talking_tree(k),
            0x26 => self.sprite_prep_hardhat_beetle(k),
            0x28 => self.sprite_prep_storyteller(k),
            0x29 => self.sprite_prep_adults(k),
            0x2b => self.sprite_prep_hobo(k),
            0x2c | 0x3a => self.sprite_prep_magic_bat(k),
            0x2e => self.sprite_prep_flute_kid(k),
            0x31 => self.sprite_prep_fortune_teller(k),
            0x33 => self.sprite_prep_rupee_pull(k),
            0x34 => self.sprite_prep_snitch_bounce_2(k),
            0x35 => self.sprite_prep_snitch_bounce_3(k),
            0x39 => self.sprite_prep_locksmith(k),
            0x3b => self.sprite_prep_bonk_item(k),
            0x3d => self.sprite_prep_snitch_bounce_1(k),
            0x40 => self.sprite_prep_agahnims_barrier(k),
            0x41..=0x43 => self.sprite_prep_standard_guard(k),
            0x44..=0x4a => self.sprite_prep_trooper_and_archer_soldier(k),
            0x4b => self.sprite_prep_weak_guard(k),
            0x4c => self.sprite_prep_geldman(k),
            0x4d | 0x9a => self.sprite_prep_kyameron(k),
            0x4e => self.sprite_prep_popo(k),
            0x4f => self.sprite_prep_popo2(k),
            0x51 => self.sprite_prep_do_nothing_d(k),
            0x52 => self.sprite_prep_king_zora(k),
            0x53 => self.sprite_prep_armos_knight(k),
            0x54 => self.sprite_prep_lanmolas(k),
            0x55 => self.sprite_prep_swimming_zora(k),
            0x56 => self.sprite_prep_walking_zora(k),
            0x57 => self.sprite_prep_desert_statue(k),
            0x59 => self.sprite_prep_lost_woods_bird(k),
            0x5a => self.sprite_prep_lost_woods_squirrel(k),
            0x5b | 0x5c => self.sprite_prep_spark(k),
            0x5d => self.sprite_prep_roller_vertical_down_first(k),
            0x5e => self.sprite_prep_roller_up_down(k),
            0x5f => self.sprite_prep_roller_horizontal_right_first(k),
            0x60 => self.sprite_prep_roller_left_right(k),
            0x62 => self.sprite_prep_master_sword(k),
            0x63 => self.sprite_prep_debirando_pit(k),
            0x64 => self.sprite_prep_fire_debirando(k),
            0x65 => self.sprite_prep_arrow_game_bounce(k),
            0x66..=0x69 => self.sprite_prep_wall_cannon(k),
            0x6d => self.sprite_prep_rat(k),
            0x6e => self.sprite_prep_rope(k),
            0x6f => self.sprite_prep_keese(k),
            0x70 | 0x7b | 0x80 | 0x81 | 0x85 | 0x87 | 0x89 | 0x8b | 0x8e | 0x90 | 0x91 => {
                self.sprite_prep_do_nothing_g(k)
            }
            0x71 => self.sprite_prep_fairy_pond(k),
            0x73 => self.sprite_prep_uncle_and_priest_bounce(k),
            0x74 | 0x82 | 0x83 | 0x84 | 0xe7 | 0xe8 | 0xe9 | 0xea | 0xf2 => {
                self.runtime.prep_cross_bank(self.ram, k, kind)
            }
            0x76 => self.sprite_prep_zelda_bounce(k),
            0x78 => self.sprite_prep_mrs_sahasrahla(k),
            0x79 | 0xac => self.sprite_prep_overworld_bonk_item(k),
            0x7a => self.sprite_prep_agahnim(k),
            0x7c => self.sprite_prep_green_stalfos(k),
            0x7d => self.sprite_prep_big_spike(k),
            0x7e | 0x7f => self.sprite_prep_fire_bar(k),
            0x86 => self.sprite_prep_kodongo(k),
            0x88 => self.sprite_prep_mothula(k),
            0x8a => self.sprite_prep_spike(k),
            0x8c => self.sprite_prep_arrghus(k),
            0x8d => self.sprite_prep_arrghi(k),
            0x8f => self.sprite_prep_blob(k),
            0x92 => self.sprite_prep_helmasaur_king(k),
            0x93 => self.sprite_prep_bumper(k),
            0x95..=0x98 => self.sprite_prep_laser_eye_bounce(k),
            0x9c => self.sprite_prep_zoro(k),
            0x9d => self.sprite_prep_babasu(k),
            0x9e => self.sprite_prep_haunted_grove_ostrich(k),
            0x9f | 0xa0 => self.sprite_prep_haunted_grove_animal(k),
            0xa1 => self.sprite_prep_move_down_8px(k),
            0xa2 => self.sprite_prep_kholdstare(k),
            0xa3 => self.sprite_prep_kholdstare_shell(k),
            0xa4 => self.sprite_prep_falling_ice(k),
            0xa5 | 0xa6 => self.sprite_prep_zazakku(k),
            0xa7 => self.sprite_prep_stalfos(k),
            0xa8 | 0xa9 => self.sprite_prep_bomber(k),
            0xaa => self.sprite_prep_do_nothing_c(k),
            0xab => self.sprite_prep_do_nothing_h(k),
            0xad => self.sprite_prep_old_man_bounce(k),
            0xb2 => self.sprite_prep_nice_bee(k),
            0xb3 => self.sprite_prep_pedestal_plaque(k),
            0xb4 => self.sprite_prep_purple_chest(k),
            0xb5 => self.sprite_prep_bomb_shoppe(k),
            0xb6 => self.sprite_prep_kiki(k),
            0xb7 => self.sprite_prep_blind_maiden(k),
            0xb9 => self.sprite_prep_bully_and_victim(k),
            0xba => self.sprite_prep_whirlpool(k),
            0xbb => self.sprite_prep_shopkeeper(k),
            0xbd => self.sprite_prep_vitreous(k),
            0xbe => self.sprite_prep_mini_vitreous(k),
            0xc0 => self.sprite_prep_catfish(k),
            0xc1 => self.sprite_prep_cutscene_agahnim(k),
            0xc3 => self.sprite_prep_gibo(k),
            0xc7 => self.sprite_prep_pokey(k),
            0xc8 => self.sprite_prep_big_fairy(k),
            0xc9 => self.sprite_prep_tektite(k),
            0xca => self.sprite_prep_chainchomp_bounce(k),
            0xcb..=0xcd => self.sprite_prep_trinexx(k),
            0xce => self.sprite_prep_blind(k),
            0xcf => self.sprite_prep_swamola(k),
            0xd3 => self.sprite_prep_rock_stal(k),
            0xd5 => self.sprite_prep_digging_game_guy_bounce(k),
            0xd6 | 0xd7 => self.sprite_prep_ganon(k),
            0xd8..=0xe2 => self.sprite_prep_absorbable(k),
            0xe3 => self.sprite_prep_fairy(k),
            0xe4 => self.sprite_prep_small_key(k),
            0xe5 => self.sprite_prep_big_key(k),
            0xe6 => self.sprite_prep_shield_pickup(k),
            0xeb => self.sprite_prep_heart_piece(k),
            0xec => self.sprite_prep_throwable_scenery(k),
            0xee => self.sprite_prep_mantle(k),
            _ => unreachable!("sprite prep type outside 0..=0xf2: {kind:#x}"),
        }
    }

    pub fn sprite_prep_trooper_and_archer_soldier(&mut self, k: usize) {
        let submodule = self.ram.read_u8(0x11);
        self.ram.write_u8(0x11, 0);
        self.ram
            .set_sprite_deflection_bits(k, (self.ram.sprite(k).deflection_bits() >> 1) | 0x80);
        self.sprite_active_main(k);
        self.sprite_active_main(k);
        self.ram
            .set_sprite_deflection_bits(k, self.ram.sprite(k).deflection_bits() << 1);
        self.ram.write_u8(0x11, submodule);
    }

    pub fn sprite_prep_standard_guard(&mut self, k: usize) {
        const SOLDIER_TAB: [u8; 8] = [0, 2, 1, 3, 6, 4, 5, 7];
        let subtype = self.ram.sprite(k).subtype();
        if subtype != 0 {
            if subtype & 7 >= 5 {
                let j = usize::from(u8::from(subtype & 7 != 5) * 4 + ((subtype >> 3) & 3));
                self.ram.set_sprite_scratch_b(k, SOLDIER_TAB[j]);
                self.ram
                    .set_sprite_flags(k, (self.ram.sprite(k).flags() & 0x0f) | 0x50);
                self.sprite_prep_trooper_and_archer_soldier(k);
                return;
            }
            self.ram
                .set_sprite_scratch_d(k, ((subtype & 7).wrapping_sub(1)) ^ 1);
        }
        if self.ram.read_u8(0x1b) != 0 {
            self.ram
                .set_sprite_flags5(k, self.ram.sprite(k).flags5() & !0x80);
            return;
        }
        self.ram.set_sprite_ai_state(k, 1);
        self.ram.set_sprite_main_delay(k, 112);
        let direction = self.runtime.direction_to_face_link(self.ram, k);
        self.ram.set_sprite_scratch_d(k, direction);
        self.ram.set_sprite_head_direction(k, direction);
        self.sprite_prep_trooper_and_archer_soldier(k);
    }

    pub fn sprite_prep_uncle_and_priest_bounce(&mut self, k: usize) {
        let room = self.ram.read_u8(0x48e);
        if room == 18 {
            self.runtime.priest_spawn_mantle(self.ram, k);
            if self.ram.read_u8(0xf3c5) >= 3 {
                self.ram.write_u8(0xf3c6, self.ram.read_u8(0xf3c6) | 2);
            }
            if self.ram.read_u8(0xf3c6) & 2 != 0 {
                self.ram.set_sprite_state(k, 0);
                return;
            }
            self.ram.set_sprite_scratch_e(k, 1);
            self.ram
                .set_sprite_flags2(k, (self.ram.sprite(k).flags2() & 0xf0) | 2);
            self.ram.set_sprite_flags4(k, 3);
            let j = if self.ram.read_u8(0xf359) >= 2 {
                self.ram.set_sprite_scratch_d(k, 4);
                self.ram.set_sprite_graphics(k, 0);
                0usize
            } else {
                let direction = self.runtime.direction_to_face_link(self.ram, k) ^ 3;
                self.ram.set_sprite_scratch_d(k, direction);
                self.ram.set_sprite_head_direction(k, direction);
                if self.ram.read_u8(0xf3cc) == 1 {
                    self.ram.write_u8(0xf3c6, self.ram.read_u8(0xf3c6) | 4);
                    self.ram
                        .write_u8(0xf280 + 0x1b, self.ram.read_u8(0xf280 + 0x1b) | 0x20);
                    self.ram.set_sprite_main_delay(k, 170);
                    1
                } else {
                    2
                }
            };
            self.ram.set_sprite_subtype2(k, j as u8);
            let x = u16::from_le_bytes([self.ram.sprite(k).x_low(), self.ram.sprite(k).x_high()])
                .wrapping_sub(6);
            let y = u16::from_le_bytes([self.ram.sprite(k).y_low(), self.ram.sprite(k).y_high()])
                .wrapping_add([0i16, -9, 0][j] as u16);
            self.set_sprite_x(k, x);
            self.set_sprite_y(k, y);
            self.sprite_prep_ignore_projectiles(k);
            self.ram.write_u8(0x1fe01, 0);
        } else if room == 4 {
            if self.ram.read_u8(0xf3c6) & 0x10 == 0 {
                self.ram
                    .set_sprite_x_low(k, self.ram.sprite(k).x_low().wrapping_add(8));
            } else {
                self.ram.set_sprite_state(k, 0);
            }
        } else if self.ram.read_u8(0xf3c6) & 1 == 0 {
            self.ram.set_sprite_scratch_d(k, 3);
            self.ram.set_sprite_subtype2(k, 1);
        } else {
            self.ram.set_sprite_state(k, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;
    struct FaerieOam;
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for FaerieOam {}
    impl SpriteMainBank1Runtime for FaerieOam {
        fn prep_oam_with_flags(&mut self, _: &mut Ram<'_>, _: usize) -> Option<(u8, u8, u8)> {
            Some((0x80, 0x40, 0x30))
        }
    }
    #[test]
    fn fairy_queen_draw_uses_local_oam_pointer() {
        let mut b = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut b);
        ram.write_u16(0x90, 0x830);
        ram.write_u16(0x92, 0xa2c);
        let mut runtime = FaerieOam;
        SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut runtime,
        }
        .faerie_queen_draw(5);
        assert_eq!(ram.read_u16(0x90), 0x830);
        assert_eq!(ram.read_u16(0x92), 0xa2c);
        assert_eq!(ram.read_u8(0x830), 0x90);
    }
    struct Rt;
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for Rt {}
    impl SpriteMainBank1Runtime for Rt {}

    struct StatefulOam;
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for StatefulOam {}
    impl SpriteMainBank1Runtime for StatefulOam {
        fn prep_oam_with_flags_wide(
            &mut self,
            _ram: &mut Ram<'_>,
            _k: usize,
        ) -> Option<(u16, u16, u8)> {
            Some((0x20, 0x30, 0x70))
        }

        fn emit_oam_wide(
            &mut self,
            ram: &mut Ram<'_>,
            _k: usize,
            x: u16,
            y: u16,
            character: u8,
            flags: u8,
        ) {
            let index = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
            crate::oam::set_helper0(ram, index, x, y, character, flags, 0);
            ram.write_u16(0x90, ram.read_u16(0x90).wrapping_add(4));
            ram.write_u16(0x92, ram.read_u16(0x92).wrapping_add(1));
        }
    }

    #[test]
    fn poe_draw_writes_current_entry_without_advancing_global_oam_cursors() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = StatefulOam;
        ram.write_u16(0x90, crate::oam::OAM_BUFFER as u16 + 13 * 4);
        ram.write_u16(0x92, crate::oam::BYTEWISE_EXTENDED_OAM as u16 + 13);
        ram.set_sprite_scratch_d(0, 0);
        ram.set_sprite_subtype2(0, 0);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };

        bank.poe_draw(0);

        assert_eq!(
            bank.ram.read_u16(0x90),
            crate::oam::OAM_BUFFER as u16 + 13 * 4
        );
        assert_eq!(
            bank.ram.read_u16(0x92),
            crate::oam::BYTEWISE_EXTENDED_OAM as u16 + 13
        );
        let entry = crate::oam::read(bank.ram, 13);
        assert_eq!(
            (entry.x, entry.y, entry.charnum, entry.flags),
            (0x29, 0x39, 0x7c, 0x72)
        );
    }

    #[test]
    fn book_fall_state_uses_signed_bounce_and_quarter_velocity() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_ai_state(0, 2);
        ram.set_sprite_z(0, 0xff);
        ram.set_sprite_velocity_z(0, 0xf1);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_book_of_mudora(0);
        assert_eq!(bank.ram.sprite(0).z(), 0);
        assert_eq!(bank.ram.sprite(0).velocity_z(), 4);
    }

    #[test]
    fn dark_world_enemy_prep_selects_exact_parallel_stats() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.write_u8(0xfff, 1);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_prep_keese(3);
        assert_eq!(bank.ram.sprite(3).bump_damage(), 0x85);
        assert_eq!(bank.ram.sprite(3).health(), 4);
        assert_eq!(bank.ram.sprite(3).flags5(), 7);
    }

    #[test]
    fn lumberjack_tree_landing_enters_leaf_phase_with_wrapping_velocity() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_ai_state(0, 1);
        ram.set_sprite_velocity_z(0, 1);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_lumberjack_tree(0);
        assert_eq!(bank.ram.sprite(0).ai_state(), 2);
        assert_eq!(bank.ram.sprite(0).velocity_x(), (-4i8) as u8);
        assert_eq!(bank.ram.read_u8(0x12f), 0x1b);
    }

    #[test]
    fn roller_prep_preserves_complemented_bit_and_wrapping_flag_increment() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_x_low(2, 0);
        ram.set_sprite_flags4(2, 0xff);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_prep_roller_left_right(2);
        assert_eq!(bank.ram.sprite(2).ai_state(), 1);
        assert_eq!(bank.ram.sprite(2).flags4(), 0);
        assert_eq!(bank.ram.sprite(2).scratch_d(), 1);
    }

    #[test]
    fn nice_bee_is_removed_when_any_bottle_contains_a_good_bee() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_state(5, 9);
        ram.write_u8(0xf35e, 8);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_prep_nice_bee(5);
        assert_eq!(bank.ram.sprite(5).state(), 0);
        assert_eq!(bank.ram.sprite(5).scratch_e(), 1);
        assert_eq!(bank.ram.sprite(5).ignore_projectile(), 1);
    }

    #[test]
    fn initializer_dispatches_local_prep_after_loading_and_incrementing_state() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_kind(4, 0x62);
        ram.set_sprite_state(4, 9);
        ram.set_sprite_x_low(4, 0xfd);
        ram.set_sprite_y_low(4, 0xfe);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_module_initialize(4);
        assert_eq!(bank.ram.sprite(4).state(), 10);
        assert_eq!(bank.ram.sprite(4).x_low(), 3);
        assert_eq!(bank.ram.sprite(4).y_low(), 4);
    }

    #[test]
    fn giant_moldorm_death_wait_preserves_aux_timer_quirk() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_ai_state(2, 3);
        ram.set_sprite_aux4_delay(2, 7);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_09_giant_moldorm(2);
        assert_eq!(bank.ram.sprite(2).hit_timer(), 0xe7);
        assert_eq!(bank.ram.sprite(2).state(), 0);
    }

    #[test]
    fn cucco_calm_uses_exact_keese_velocity_ring() {
        struct ZeroRandom;
        impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for ZeroRandom {}
        impl SpriteMainBank1Runtime for ZeroRandom {}
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = ZeroRandom;
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.cucco_calm(0);
        assert_eq!(bank.ram.sprite(0).velocity_x(), 0);
        assert_eq!(bank.ram.sprite(0).velocity_y(), (-16i8) as u8);
        assert_eq!(bank.ram.sprite(0).main_delay(), 0x10);
        assert_eq!(bank.ram.sprite(0).ai_state(), 1);
    }

    #[test]
    fn hint_payment_uses_goal_counter_and_exact_twenty_rupee_threshold() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.write_u16(0xf360, 19);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        assert!(!bank.dark_world_hint_npc_handle_payment());
        assert_eq!(bank.ram.read_u16(0xf360), 19);
        bank.ram.write_u16(0xf360, 20);
        assert!(bank.dark_world_hint_npc_handle_payment());
        assert_eq!(bank.ram.read_u16(0xf360), 0);
    }

    #[test]
    fn flute_quaver_uses_wrapping_signed_horizontal_acceleration() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.write_u8(0x1a, 0x20);
        ram.write_u8(0xfa0, 0);
        ram.set_sprite_velocity_x(0, 0);
        ram.set_sprite_main_delay(0, 1);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_flute_kid_quaver(0);
        assert_eq!(bank.ram.sprite(0).velocity_x(), 0xff);
    }

    #[test]
    fn smithy_hammer_cycle_preserves_postdecrement_zero_trigger() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_ai_state(0, 7);
        ram.set_sprite_scratch_a(0, 4);
        ram.set_sprite_scratch_b(0, 0);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.smithy_main(0);
        assert_eq!(bank.ram.sprite(0).scratch_a(), 5);
        assert_eq!(bank.ram.sprite(0).graphics(), 3);
        assert_eq!(bank.ram.sprite(0).scratch_b(), 16);
    }

    #[test]
    fn hobo_smoke_rotates_only_high_oam_flag_bits() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.write_u8(0x1a, 0x20);
        ram.set_sprite_oam_flags(0, 0x2a);
        ram.set_sprite_main_delay(0, 1);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_hobo_smoke(0);
        assert_eq!(bank.ram.sprite(0).oam_flags(), 0xaa);
    }

    #[test]
    fn happiness_pond_bomb_upgrade_uses_hex_capacity_and_advances_palette_state() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_ai_state(0, 8);
        ram.write_u8(0xf370, 2);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_happiness_pond(0);
        assert_eq!(bank.ram.read_u8(0xf370), 3);
        assert_eq!(bank.ram.read_u8(0xf375), 0x25);
        assert_eq!(bank.ram.read_u8(0x1cf2), 0x25);
        assert_eq!(bank.ram.sprite(0).ai_state(), 9);
    }

    #[test]
    fn maze_timer_carries_from_low_word_after_exact_sixty_three_ticks() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut rt = Rt;
        ram.set_sprite_ai_state(0, 2);
        ram.set_sprite_scratch_a(0, 62);
        ram.write_u16(0x1fe00, 0xffff);
        let mut bank = SpriteMainBank1 {
            ram: &mut ram,
            runtime: &mut rt,
        };
        bank.sprite_maze_game_lady(0);
        assert_eq!(bank.ram.sprite(0).scratch_a(), 0);
        assert_eq!(bank.ram.read_u16(0x1fe00), 0);
        assert_eq!(bank.ram.read_u16(0x1fe02), 1);
    }
}
