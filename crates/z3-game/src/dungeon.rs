//! Dungeon room construction, transitions, doors, tags, and runtime effects.
//!
//! This module preserves the original WRAM-backed tilemaps and wrapping
//! arithmetic from `src/dungeon.c`. Cross-translation-unit calls and ROM assets
//! are exposed through [`DungeonHooks`].

use crate::ram::Ram;
use z3_core::WRAM_SIZE;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DungeonEntrance {
    pub entrance: u16,
    pub room: u16,
    pub scroll_y: u16,
    pub scroll_x: u16,
    pub player_y: u16,
    pub player_x: u16,
    pub camera_y: u16,
    pub camera_x: u16,
    pub door_settings: u16,
    pub relative_coords: [u8; 8],
    pub blockset: u8,
    pub floor: u8,
    pub palace: u8,
    pub doorway_orientation: u8,
    pub starting_bg: u8,
    pub quadrant1: u8,
    pub quadrant2: u8,
    pub music: u8,
}

pub trait DungeonHooks {
    fn dungeon_draw_pushable_block(
        &mut self,
        _ram: &mut Ram<'_>,
        _tilemap: u16,
        _object_offset: u16,
    ) {
    }
    fn dungeon_draw_lightable_torch(
        &mut self,
        _ram: &mut Ram<'_>,
        _descriptor: u16,
        _object_offset: u16,
    ) {
    }
    fn dungeon_movable_block(&self, _index: usize) -> (u16, u16) {
        (0xffff, 0)
    }
    fn dungeon_torch_data(&self, _index: usize) -> u16 {
        0xffff
    }
    fn dungeon_transition_aux_gfx_sprite(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_transition_aux_gfx(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_nmi_upload_tilemap(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_entrance(&self, _starting_point: bool, _index: usize) -> DungeonEntrance {
        DungeonEntrance::default()
    }
    fn dungeon_entrance_music(&self, _entrance: usize) -> u8 {
        0xff
    }
    fn dungeon_starting_music(&self, _starting_point: usize) -> u8 {
        0xff
    }
    fn dungeon_entrance_music_track(&self, _entrance: usize) -> u8 {
        0xff
    }
    fn hud_rebuild(&mut self, _ram: &mut Ram<'_>) {}
    fn hud_refill_logic(&mut self, _ram: &mut Ram<'_>) {}
    fn hud_floor_indicator(&mut self, _ram: &mut Ram<'_>) {}
    fn initialize_tilesets(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_load_palettes(&mut self, _ram: &mut Ram<'_>) {}
    fn decompress_animated_dungeon_tiles(&mut self, _ram: &mut Ram<'_>, _tile_set: u8) {}
    fn load_gear_palettes_bunny(&mut self, _ram: &mut Ram<'_>) {}
    fn hud_remove_super_bomb_indicator(&mut self, _ram: &mut Ram<'_>) {}
    fn link_tuck_into_bed(&mut self, _ram: &mut Ram<'_>) {}
    fn enable_force_blank(&mut self, _ram: &mut Ram<'_>) {}
    fn sprite_repel_dash(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_add_door_debris(&mut self, _ram: &mut Ram<'_>) -> i32 {
        -1
    }
    fn overworld_bomb_tiles_32x32(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) {}
    fn dungeon_calculate_sfx_pan(&self, _ram: &Ram<'_>, _x: u16) -> u8 {
        0
    }
    fn dungeon_object_data(&self, _word_offset: usize) -> &[u16] {
        &[]
    }
    fn dungeon_tele_message(&self, _room: usize) -> u16 {
        0
    }
    fn dungeon_pits_hurt_player(&self) -> &[u16] {
        &[]
    }
    fn dungeon_tile_attribute(&self, _ram: &Ram<'_>, _tile: usize) -> u8 {
        0
    }
    fn dungeon_room_door_info(&self, _room: usize) -> &[u16] {
        &[]
    }
    fn dungeon_room_header(&self, _room: usize) -> &[u8] {
        &[]
    }
    fn dungeon_room_chests(&self) -> &[u8] {
        &[]
    }
    fn dungeon_secrets(&self) -> &[u8] {
        &[]
    }
    fn default_room_layout(&self, _layout: usize) -> &[u8] {
        &[]
    }
    fn dungeon_room_layout(&self, _layout: usize) -> &[u8] {
        &[]
    }
    fn dungeon_restore_star_tile_chr(&mut self, _ram: &mut Ram<'_>) {}
    fn cache_camera_properties(&mut self, _ram: &mut Ram<'_>) {}
    fn prep_trans_aux_gfx(&mut self, _ram: &mut Ram<'_>) {}
    fn iris_spotlight_reset_table(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_prep_overlay_dma_next(
        &mut self,
        _ram: &mut Ram<'_>,
        destination: usize,
        _source: u16,
    ) -> usize {
        destination
    }
    fn dungeon_prep_overlay_dma_watergate(
        &mut self,
        _ram: &mut Ram<'_>,
        destination: usize,
        _source: u16,
        _mode: u16,
        _loops: usize,
    ) -> usize {
        destination
    }
    fn sprite_check_if_screen_is_clear(&self, _ram: &Ram<'_>) -> bool {
        false
    }
    fn sprite_check_if_room_is_clear(&self, _ram: &Ram<'_>) -> bool {
        false
    }
    fn load_overworld_songs(&mut self, _ram: &mut Ram<'_>) {}
    fn load_dungeon_songs(&mut self, _ram: &mut Ram<'_>) {}
    fn apply_palette_filter_bounce(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_handle_translucency_and_palette(&mut self, _ram: &mut Ram<'_>) {}
    fn hud_restore_torch_background(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_terminate_select_interactives(&mut self, _ram: &mut Ram<'_>, _kind: u8) -> u8 {
        0
    }
    fn turn_while_dashing_enabled(&self) -> bool {
        false
    }
    fn overworld_copy_palettes_to_cache(&mut self, _ram: &mut Ram<'_>) {}
    fn orient_lamp_light_cone(&mut self, _ram: &mut Ram<'_>) {}
    fn link_handle_moving_animation_full_long_entry(&mut self, _ram: &mut Ram<'_>) {}
    fn mirror_bg1_bg2_offsets(&mut self, _ram: &mut Ram<'_>) {}
    fn conditional_mosaic_control(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_custom_tile_attributes(&self, _theme: usize) -> &[u8] {
        &[]
    }
    fn update_pegs_step1(&mut self, _ram: &mut Ram<'_>) {}
    fn update_pegs_step2(&mut self, _ram: &mut Ram<'_>) {}
    fn recover_peg_gfx_from_mapping(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_reset_sprites(&mut self, _ram: &mut Ram<'_>) {}
    fn load_new_sprite_gfx_set(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_assert_translucency_swap(&mut self, _ram: &mut Ram<'_>) {}
    fn room_draw_adjust_torch_lighting_change(
        &mut self,
        _ram: &mut Ram<'_>,
        _x: u16,
        _source: u16,
        _dma: u16,
    ) {
    }
    fn ancilla_add_blast_wall(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_spawn_falling_prize(&mut self, _ram: &mut Ram<'_>, _kind: u8) -> i32 {
        -1
    }
    fn spotlight_open(&mut self, _ram: &mut Ram<'_>) {}
    fn iris_spotlight_configure_table(&mut self, _ram: &mut Ram<'_>) {}
    fn sprite_main(&mut self, _ram: &mut Ram<'_>) {}
    fn overworld_reset_mosaic_always_increase(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_revert_translucency_swap(&mut self, _ram: &mut Ram<'_>) {}
    fn recover_position_after_drowning(&mut self, _ram: &mut Ram<'_>) {}
    fn zelda_is_playing_music_track(&self, _track: u8) -> bool {
        false
    }
    fn adjust_water_hdma_window(&mut self, _ram: &mut Ram<'_>) {}
    fn adjust_water_hdma_window_x(&mut self, _ram: &mut Ram<'_>, _upper: u16) {}
    fn save_dungeon_keys(&mut self, _ram: &mut Ram<'_>) {}
    fn graphics_load_chr_half_slot(&mut self, _ram: &mut Ram<'_>) {}
    fn dungeon_room_overlay(&self, _index: usize) -> &[u8] {
        &[]
    }
    fn sprite_load_graphics_properties_light_world_only(&mut self, _ram: &mut Ram<'_>) {}
    fn oam_allocate_from_region_b(&mut self, _ram: &mut Ram<'_>, _bytes: u8) -> (u16, u16) {
        (0, 0)
    }
    fn handle_indoor_camera_and_doors(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_sfx2_near(&mut self, _ram: &mut Ram<'_>, _sound: u8) {}
    fn link_move_position(&mut self, _ram: &mut Ram<'_>) {}
    fn link_handle_moving_animation_start_with_dash(&mut self, _ram: &mut Ram<'_>) {}
    fn follower_initialize(&mut self, _ram: &mut Ram<'_>) {}
    fn flood_dam_prep_flood_hdma(&mut self, _ram: &mut Ram<'_>) {}
    fn graphics_incremental_vram_upload(&mut self, _ram: &mut Ram<'_>) {}
    fn link_oam_main(&mut self, _ram: &mut Ram<'_>) {}
    fn misc_bug_fixes_enabled(&self) -> bool {
        false
    }
    fn palette_filter_crystal(&mut self, _ram: &mut Ram<'_>) {}
    fn crystal_cutscene_initialize_polyhedral(&mut self, _ram: &mut Ram<'_>) {}
    fn sprite_spawn_dynamically(&mut self, _ram: &mut Ram<'_>, _parent: u8, _kind: u8) -> i32 {
        -1
    }
    fn load_follower_graphics(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_filter_restore_bg_subtractive_strict(&mut self, _ram: &mut Ram<'_>) {}
    fn polyhedral_initialize_thread(&mut self, _ram: &mut Ram<'_>) {}
    fn main_show_text_message(&mut self, _ram: &mut Ram<'_>) {}
    fn get_random_number(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
    fn break_pots_with_sword_enabled(&self) -> bool {
        false
    }
    fn sprite_spawn_immediately_smashed_terrain(
        &mut self,
        _ram: &mut Ram<'_>,
        _kind: u8,
        _x: u16,
        _y: u16,
    ) {
    }
    fn ancilla_add_bush_poof(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) {}
    fn link_handle_velocity(&mut self, _ram: &mut Ram<'_>) {}
    fn apply_links_movement_to_camera(&mut self, _ram: &mut Ram<'_>) {}
    fn link_check_for_edge_screen_transition(&mut self, _ram: &mut Ram<'_>) -> bool {
        false
    }
    fn did_press_button_for_map(&self, _ram: &Ram<'_>) -> bool {
        false
    }
    fn display_select_menu(&mut self, _ram: &mut Ram<'_>) {}
    fn hud_handle_item_switch_inputs(&mut self, _ram: &mut Ram<'_>) {}
    fn link_main(&mut self, _ram: &mut Ram<'_>) {}
    fn handle_dungeon_landing_from_pit(&mut self, _ram: &mut Ram<'_>) {}
}

pub struct Dungeon<'a, 'h, H: DungeonHooks> {
    pub ram: Ram<'a>,
    pub hooks: &'h mut H,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoomBounds {
    pub a0: u16,
    pub b0: u16,
    pub a1: u16,
    pub b1: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DungPalInfo {
    pub pal0: u8,
    pub pal1: u8,
    pub pal2: u8,
    pub pal3: u8,
}

impl RoomBounds {
    pub fn add_a(&mut self) {
        self.a0 = self.a0.wrapping_add(0x100);
        self.a1 = self.a1.wrapping_add(0x100);
    }
    pub fn add_b(&mut self) {
        self.b0 = self.b0.wrapping_add(0x200);
        self.b1 = self.b1.wrapping_add(0x200);
    }
    pub fn sub_b(&mut self) {
        self.b0 = self.b0.wrapping_sub(0x200);
        self.b1 = self.b1.wrapping_sub(0x200);
    }
    pub fn sub_a(&mut self) {
        self.a0 = self.a0.wrapping_sub(0x100);
        self.a1 = self.a1.wrapping_sub(0x100);
    }
}

impl<'a, 'h, H: DungeonHooks> Dungeon<'a, 'h, H> {
    pub fn new(wram: &'a mut [u8; WRAM_SIZE], hooks: &'h mut H) -> Self {
        Self {
            ram: Ram::new(wram),
            hooks,
        }
    }

    fn r8(&self, address: usize) -> u8 {
        self.ram.read_u8(address)
    }

    fn w8(&mut self, address: usize, value: u8) {
        self.ram.write_u8(address, value);
    }

    fn r16(&self, address: usize) -> u16 {
        self.ram.read_u16(address)
    }

    fn w16(&mut self, address: usize, value: u16) {
        self.ram.write_u16(address, value);
    }

    fn set_bg1(&mut self, word_offset: usize, value: u16) {
        self.w16(DUNG_BG1 + word_offset * 2, value);
    }

    fn set_bg2(&mut self, word_offset: usize, value: u16) {
        self.w16(DUNG_BG2 + word_offset * 2, value);
    }

    fn object_data<const N: usize>(&self, byte_offset: usize) -> [u16; N] {
        let source = self.hooks.dungeon_object_data(byte_offset / 2);
        assert!(source.len() >= N);
        core::array::from_fn(|index| source[index])
    }

    #[must_use]
    pub fn dsto_ptr(&self, offset: u16) -> usize {
        usize::from(self.r16(0xbf)) + usize::from(offset) * 2
    }

    pub fn object_fill_nx1(&mut self, count: usize, tile: u16, dst: usize) {
        for i in 0..count {
            self.w16(dst + i * 2, tile);
        }
    }

    pub fn object_draw_5x4(&mut self, src: &[u16], dst: usize) {
        assert!(src.len() >= 20);
        for row in 0..5 {
            for column in 0..4 {
                self.w16(dst + xy(column, row) * 2, src[row * 4 + column]);
            }
        }
    }

    pub fn object_draw_4x2_both_bgs(&mut self, src: &[u16], destination: usize) {
        assert!(src.len() >= 8);
        for row in 0..2 {
            for column in 0..4 {
                let value = src[row * 4 + column];
                let offset = destination + xy(column, row);
                self.set_bg1(offset, value);
                self.set_bg2(offset, value);
            }
        }
    }

    pub fn object_chest_platform_helper(&mut self, src: &[u16], mut destination: usize) {
        assert!(src.len() >= 19);
        self.set_bg2(destination, src[0]);
        let width = usize::from(self.r8(0xb2));
        for _ in 0..width {
            self.set_bg2(destination + 1, src[3]);
            destination += 1;
        }
        self.set_bg2(destination + 1, src[6]);
        for offset in 2..=5 {
            self.set_bg2(destination + offset, src[9]);
        }
        self.set_bg2(destination + 6, src[12]);
        for _ in 0..width {
            self.set_bg2(destination + 7, src[15]);
            destination += 1;
        }
        self.set_bg2(destination + 7, src[18]);
    }

    pub fn object_hole(&mut self, src: &[u16], destination: usize) {
        self.object_size_a_to_a_plus15(4);
        let width = usize::from(self.r16(0xb2));
        assert!(!src.is_empty());
        for row in 0..width {
            self.object_fill_nx1(width, src[0], destination + xy(0, row) * 2);
        }

        let top_bottom = self.hooks.dungeon_object_data(0x63c / 2);
        assert!(top_bottom.len() >= 6);
        let edge = [
            top_bottom[0],
            top_bottom[1],
            top_bottom[2],
            top_bottom[3],
            top_bottom[4],
            top_bottom[5],
        ];
        self.w16(destination, edge[0]);
        self.object_fill_nx1(width - 2, edge[1], destination + 2);
        self.w16(destination + (width - 1) * 2, edge[2]);
        let bottom = destination + xy(0, width - 1) * 2;
        self.w16(bottom, edge[3]);
        self.object_fill_nx1(width - 2, edge[4], bottom + 2);
        self.w16(bottom + (width - 1) * 2, edge[5]);

        let sides = self.hooks.dungeon_object_data(0x648 / 2);
        assert!(sides.len() >= 2);
        let sides = [sides[0], sides[1]];
        for row in 1..width - 1 {
            self.w16(destination + xy(0, row) * 2, sides[0]);
            self.w16(destination + xy(width - 1, row) * 2, sides[1]);
        }
    }

    pub fn room_draw_a_many_32x32_blocks(&mut self, count: usize, src: &[u16], dst: usize) {
        assert!(src.len() >= 8);
        for block in 0..count {
            for row in 0..4 {
                for column in 0..4 {
                    self.w16(
                        dst + xy(block * 4 + column, row) * 2,
                        src[(row & 1) * 4 + column],
                    );
                }
            }
        }
    }

    pub fn room_draw_1x3_rightwards(&mut self, count: usize, src: &[u16], dst: usize) {
        assert!(src.len() >= count * 3);
        for column in 0..count {
            for row in 0..3 {
                self.w16(dst + xy(column, row) * 2, src[column * 3 + row]);
            }
        }
    }

    pub fn object_table_helper(&mut self, src: &[u16], mut dst: usize) {
        assert!(src.len() >= 4);
        let width = usize::from(self.r8(0xb2));
        self.w16(dst, src[0]);
        for _ in 0..width {
            self.w16(dst + 2, src[1]);
            self.w16(dst + 4, src[2]);
            dst += 4;
        }
        self.w16(dst + 2, src[3]);
    }

    pub fn draw_water_thing(&mut self, dst: usize, src: &[u16]) {
        assert!(src.len() >= 16);
        for row in 0..4 {
            for column in 0..4 {
                self.w16(dst + xy(column, row) * 2, src[row * 4 + column]);
            }
        }
    }

    pub fn room_draw_4x4(&mut self, src: &[u16], dst: usize) {
        self.room_draw_object_nx4(4, src, dst);
    }

    pub fn room_draw_object_nx4(&mut self, count: usize, src: &[u16], dst: usize) {
        assert!(src.len() >= count * 4);
        for column in 0..count {
            for row in 0..4 {
                self.w16(dst + xy(column, row) * 2, src[column * 4 + row]);
            }
        }
    }

    pub fn object_draw_nx4_both_bgs(&mut self, count: usize, src: &[u16], destination: usize) {
        assert!(src.len() >= count * 4);
        for column in 0..count {
            for row in 0..4 {
                let value = src[column * 4 + row];
                let offset = destination + xy(column, row);
                self.set_bg1(offset, value);
                self.set_bg2(offset, value);
            }
        }
    }

    pub fn object_draw_nx3_both_bgs(&mut self, count: usize, src: &[u16], destination: usize) {
        assert!(src.len() >= count * 3);
        for column in 0..count {
            for row in 0..3 {
                let value = src[column * 3 + row];
                let offset = destination + xy(column, row);
                self.set_bg1(offset, value);
                self.set_bg2(offset, value);
            }
        }
    }

    fn write_both_bgs(&mut self, destination: u16, x: usize, y: usize, value: u16) {
        let offset = usize::from(destination.wrapping_add((y * 64 + x) as u16));
        self.set_bg1(offset, value);
        self.set_bg2(offset, value);
    }

    fn offset_words(destination: usize, words: isize) -> usize {
        destination.wrapping_add_signed(words * 2)
    }

    /// Exact translation of `LoadType1ObjectSubtype1`.
    pub fn load_type1_object_subtype1(&mut self, index: u8, mut dst: usize, mut dsto: u16) {
        let parameter = OBJECT_SUBTYPE1_PARAMS[usize::from(index)];
        let mut source = self
            .hooks
            .dungeon_object_data(usize::from(parameter) / 2)
            .to_vec();
        let width = |this: &Self| usize::from(this.r16(0xb2));
        match index {
            0x00 | 0xb8 | 0xb9 => {
                self.room_draw_get_object_size_1to15or32();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    dst += xy(2, 0) * 2;
                }
            }
            0x01 | 0x02 | 0xb6 | 0xb7 => {
                self.room_draw_get_object_size_1to15or26();
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(2, &source, dst);
                    dst += xy(2, 0) * 2;
                }
            }
            0x03 | 0x04 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    for x in 0..2 {
                        for y in 0..4 {
                            self.write_both_bgs(dsto, x, y, source[x * 4 + y]);
                        }
                    }
                    dsto = dsto.wrapping_add(2);
                }
            }
            0x05 | 0x06 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(2, &source, dst);
                    dst += 12;
                }
            }
            0x07 | 0x08 | 0x53 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    dst += 4;
                }
            }
            0x09 | 0x0c | 0x0d | 0x10 | 0x11 | 0x14 => {
                self.object_size_a_to_a_plus15(6);
                for _ in 0..width(self) {
                    self.room_draw_draw_object_2x2_and1(&source, dst);
                    dst = Self::offset_words(dst, -63);
                }
            }
            0x0a | 0x0b | 0x0e | 0x0f | 0x12 | 0x13 => {
                self.object_size_a_to_a_plus15(6);
                for _ in 0..width(self) {
                    self.room_draw_draw_object_2x2_and1(&source, dst);
                    dst += 65 * 2;
                }
            }
            0x15..=0x20 => {
                self.object_size_a_to_a_plus15(6);
                let step = if matches!(index, 0x15 | 0x18 | 0x19 | 0x1c | 0x1d | 0x20) {
                    -63
                } else {
                    65
                };
                for _ in 0..width(self) {
                    for (y, &tile) in source.iter().take(5).enumerate() {
                        self.write_both_bgs(dsto, 0, y, tile);
                    }
                    dsto = dsto.wrapping_add_signed(step);
                }
            }
            0x21 => {
                let count = (self.r16(0xb2) << 2 | self.r16(0xb4)) * 2 + 1;
                self.w16(0xb2, count);
                self.room_draw_1x3_rightwards(2, &source, dst);
                dst += 4;
                for _ in 0..count {
                    self.room_draw_1x3_rightwards(1, &source[3..], dst);
                    dst += 2;
                }
                self.room_draw_1x3_rightwards(1, &source[6..], dst);
            }
            0x22 | 0x5f => {
                self.object_size_a_to_a_plus15(if index == 0x22 { 2 } else { 21 });
                if self.r16(dst) & 0x3ff != 0xe2 {
                    self.w16(dst, source[0]);
                }
                for _ in 0..width(self) {
                    dst += 2;
                    self.w16(dst, source[1]);
                }
                self.w16(dst + 2, source[2]);
            }
            0x23..=0x2e | 0x3f..=0x46 | 0xb3 | 0xb4 => {
                self.room_draw_get_object_size_1to16();
                let tile = self.r16(dst) & 0x3ff;
                if !matches!(tile, 0x1db | 0x1a6 | 0x1dd | 0x1fc) {
                    self.w16(dst, source[0]);
                }
                for _ in 0..width(self) {
                    dst += 2;
                    self.w16(dst, source[1]);
                }
                self.w16(dst + 2, source[2]);
            }
            0x2f | 0x30 => {
                self.object_size_a_to_a_plus15(10);
                let rail = source[0];
                let lower = index == 0x30;
                if self.r16(dst + if lower { 128 } else { 0 }) & 0x3ff != 0xe2 {
                    if lower {
                        self.w16(dst, rail);
                        self.w16(dst + 2, rail);
                        self.w16(dst + 128, source[1]);
                        self.w16(dst + 130, source[2]);
                    } else {
                        self.w16(dst, source[1]);
                        self.w16(dst + 2, source[2]);
                        self.w16(dst + 128, rail);
                        self.w16(dst + 130, rail);
                    }
                    dst += 4;
                }
                for _ in 0..width(self) {
                    self.w16(dst, if lower { rail } else { source[3] });
                    self.w16(dst + 128, if lower { source[3] } else { rail });
                    dst += 2;
                }
                if lower {
                    self.w16(dst, rail);
                    self.w16(dst + 2, rail);
                    self.w16(dst + 128, source[4]);
                    self.w16(dst + 130, source[5]);
                } else {
                    self.w16(dst, source[4]);
                    self.w16(dst + 2, source[5]);
                    self.w16(dst + 128, rail);
                    self.w16(dst + 130, rail);
                }
            }
            0x31 | 0x32 | 0x35 | 0x54 | 0x57..=0x5a => {}
            0x33 | 0xb2 | 0xba => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_4x4(&source, dst);
                    dst += 8;
                }
            }
            0x34 | 0x50 => {
                self.object_size_a_to_a_plus15(if index == 0x34 { 4 } else { 2 });
                for _ in 0..width(self) {
                    self.w16(dst, source[0]);
                    dst += 2;
                }
            }
            0x36 | 0x37 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_4x4(&source, dst);
                    dst += 12;
                }
            }
            0x38 => {
                source = self.hooks.dungeon_object_data(0xe26 / 2).to_vec();
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_1x3_rightwards(2, &source, dst);
                    dst += 8;
                }
            }
            0x39 | 0x3d => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(2, &source, dst);
                    dst += 12;
                }
            }
            0x3a | 0x3b => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_1x3_rightwards(4, &source, dst);
                    dst += 16;
                }
            }
            0x3c => {
                source = self.hooks.dungeon_object_data(0x8ca / 2).to_vec();
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    self.room_draw_rightwards_2x2(&source[4..], dst + 768);
                    dst += 8;
                }
            }
            0x3e | 0x4b => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    dst += 28;
                }
            }
            0x47 => {
                self.room_draw_get_object_size_1to16();
                let count = self.r16(0xb2) << 1;
                self.w16(0xb2, count);
                self.room_draw_draw_object_2x2_and1(&source, dst);
                dst += 2;
                for _ in 0..count {
                    self.room_draw_draw_object_2x2_and1(&source[5..], dst);
                    dst += 2;
                }
                self.room_draw_draw_object_2x2_and1(&source[10..], dst);
            }
            0x48 => {
                self.room_draw_get_object_size_1to16();
                let count = self.r16(0xb2) << 1;
                self.w16(0xb2, count);
                self.room_draw_1x3_rightwards(1, &source, dst);
                dst += 2;
                for _ in 0..count {
                    for y in 0..3 {
                        self.w16(dst + y * 128, source[3 + y]);
                    }
                    dst += 2;
                }
                self.room_draw_1x3_rightwards(1, &source[6..], dst);
            }
            0x49 | 0x4a | 0x55 | 0x56 => {
                self.room_draw_get_object_size_1to16();
                self.room_draw_downwards_4x2_variable_spacing(
                    if index >= 0x55 { 12 } else { 4 },
                    &source,
                    dst,
                );
            }
            0x4c => {
                self.room_draw_get_object_size_1to16();
                let count = self.r16(0xb2) << 1;
                self.w16(0xb2, count);
                self.room_draw_rightward_bar_segment(&source, dst);
                dst += 2;
                for _ in 0..count {
                    self.room_draw_rightward_bar_segment(&source[3..], dst);
                    dst += 2;
                }
                self.room_draw_rightward_bar_segment(&source[6..], dst);
            }
            0x4d..=0x4f => {
                self.room_draw_get_object_size_1to16();
                self.room_draw_object_nx4(1, &source, dst);
                dst += 2;
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(2, &source[4..], dst);
                    dst += 4;
                }
                self.room_draw_rightward_shelf_end(&source[12..], dst);
            }
            0x51 | 0x52 | 0x5b | 0x5c => {
                self.room_draw_get_object_size_1to16();
                let count = width(self);
                self.room_draw_1x3_rightwards(2, &source, dst);
                dst += 4;
                for _ in 1..count {
                    self.room_draw_1x3_rightwards(2, &source[6..], dst);
                    dst += 4;
                }
                self.room_draw_1x3_rightwards(2, &source[12..], dst);
            }
            0x5d => {
                self.room_draw_get_object_size_1to16();
                let count = self.r16(0xb2) + 1;
                self.w16(0xb2, count);
                self.room_draw_1x3_rightwards(2, &source, dst);
                dst += 4;
                for _ in 0..count {
                    self.room_draw_rightward_bar_segment(&source[6..], dst);
                    dst += 2;
                }
                self.room_draw_1x3_rightwards(2, &source[9..], dst);
            }
            0x5e | 0xbb => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    dst += 8;
                }
            }
            0x60 | 0x92 | 0x93 => {
                self.room_draw_get_object_size_1to15or32();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    dst += 256;
                }
            }
            0x61 | 0x62 | 0x90 | 0x91 => {
                self.room_draw_get_object_size_1to15or26();
                self.room_draw_downwards_4x2_variable_spacing(128, &source, dst);
            }
            0x63 | 0x64 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    for y in 0..2 {
                        for x in 0..4 {
                            self.write_both_bgs(dsto, x, y, source[y * 4 + x]);
                        }
                    }
                    dsto = dsto.wrapping_add(128);
                }
            }
            0x65 | 0x66 => {
                self.room_draw_get_object_size_1to16();
                self.room_draw_downwards_4x2_variable_spacing(384, &source, dst);
            }
            0x67 | 0x68 | 0x7d => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    dst += 256;
                }
            }
            0x69 | 0x8a => {
                self.object_size_a_to_a_plus15(if index == 0x69 { 2 } else { 21 });
                if self.r16(dst) & 0x3ff != 0xe3 {
                    self.w16(dst, source[0]);
                }
                for _ in 0..width(self) {
                    dst += 128;
                    self.w16(dst, source[1]);
                }
                self.w16(dst + 128, source[2]);
            }
            0x6a | 0x6b | 0x79 | 0x7a | 0x8d | 0x8e => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.w16(dst, source[0]);
                    dst += 128;
                }
            }
            0x6c | 0x6d => {
                self.object_size_a_to_a_plus15(10);
                let rail = source[0];
                let right = index == 0x6d;
                if self.r16(dst + if right { 2 } else { 0 }) & 0x3ff != 0xe3 {
                    if right {
                        self.w16(dst, rail);
                        self.w16(dst + 128, rail);
                        self.w16(dst + 2, source[1]);
                        self.w16(dst + 130, source[2]);
                    } else {
                        self.w16(dst, source[1]);
                        self.w16(dst + 128, source[2]);
                        self.w16(dst + 2, rail);
                        self.w16(dst + 130, rail);
                    }
                    dst += 256;
                }
                for _ in 0..width(self) {
                    self.w16(dst, if right { rail } else { source[3] });
                    self.w16(dst + 2, if right { source[3] } else { rail });
                    dst += 128;
                }
                if right {
                    self.w16(dst, rail);
                    self.w16(dst + 128, rail);
                    self.w16(dst + 2, source[4]);
                    self.w16(dst + 130, source[5]);
                } else {
                    self.w16(dst, source[4]);
                    self.w16(dst + 128, source[5]);
                    self.w16(dst + 2, rail);
                    self.w16(dst + 130, rail);
                }
            }
            0x6e | 0x6f | 0x72 | 0x7e => {}
            0x70 | 0x94 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_4x4(&source, dst);
                    dst += 512;
                }
            }
            0x71 => {
                self.object_size_a_to_a_plus15(4);
                for _ in 0..width(self) {
                    self.w16(dst, source[0]);
                    dst += 128;
                }
            }
            0x73 | 0x74 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_4x4(&source, dst);
                    dst += 768;
                }
            }
            0x75 | 0x87 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(2, &source, dst);
                    dst += 768;
                }
            }
            0x76 | 0x77 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(3, &source, dst);
                    dst += 1024;
                }
            }
            0x78 | 0x7b => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    dst += 1792;
                }
            }
            0x7c => {
                self.room_draw_get_object_size_1to16();
                let count = self.r16(0xb2) + 1;
                self.w16(0xb2, count);
                for _ in 0..count {
                    self.w16(dst, source[0]);
                    dst += 128;
                }
            }
            0x7f | 0x80 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(2, &source, dst);
                    dst += 1536;
                }
            }
            0x81..=0x84 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(3, &source, dst);
                    dst += 768;
                }
            }
            0x85 | 0x86 => {
                self.room_draw_get_object_size_1to16();
                let count = width(self);
                self.object_draw_3x2(&source, dst);
                dst += 256;
                for _ in 1..count {
                    self.object_draw_3x2(&source[6..], dst);
                    dst += 256;
                }
                self.object_draw_3x2(&source[12..], dst);
            }
            0x88 => {
                self.room_draw_get_object_size_1to16();
                let count = width(self);
                self.room_draw_rightwards_2x2(&source, dst);
                dst += 256;
                for _ in 0..count {
                    self.w16(dst, source[4]);
                    self.w16(dst + 2, source[5]);
                    dst += 128;
                }
                self.room_draw_1x3_rightwards(2, &source[6..], dst);
            }
            0x89 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_rightwards_2x2(&source, dst);
                    dst += 512;
                }
            }
            0x8b | 0x8c => {
                self.object_size_a_to_a_plus15(8);
                for _ in 0..width(self) {
                    self.w16(dst, source[0]);
                    dst += 128;
                }
            }
            0x8f => {
                self.object_size_a_to_a_plus15(2);
                let count = self.r16(0xb2) << 1;
                self.w16(0xb2, count);
                self.w16(dst, source[0]);
                self.w16(dst + 2, source[1]);
                for _ in 0..count {
                    self.w16(dst + 128, source[2]);
                    self.w16(dst + 130, source[3]);
                    dst += 128;
                }
            }
            0x95 | 0x96 => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    if index == 0x95 {
                        self.room_draw_single_pot(&source, dst, dsto);
                    } else {
                        self.room_draw_hammer_peg_single(&source, dst, dsto);
                    }
                    dst += 256;
                    dsto = dsto.wrapping_add(128);
                }
            }
            0x97..=0x9f | 0xad..=0xaf | 0xbe | 0xbf => {}
            0xa0 | 0xa5 | 0xa9 => {
                self.object_size_a_to_a_plus15(4);
                let mut count = width(self);
                while count != 0 {
                    self.object_fill_nx1(count, source[0], dst);
                    dst += 128;
                    count -= 1;
                }
            }
            0xa1 | 0xa6 | 0xaa => {
                self.object_size_a_to_a_plus15(4);
                let count = width(self);
                for n in 1..=count {
                    self.object_fill_nx1(n, source[0], dst);
                    dst += 128;
                }
            }
            0xa2 | 0xa7 | 0xab => {
                self.object_size_a_to_a_plus15(4);
                let mut count = width(self);
                while count != 0 {
                    self.object_fill_nx1(count, source[0], dst);
                    dst += 130;
                    count -= 1;
                }
            }
            0xa3 | 0xa8 | 0xac => {
                self.object_size_a_to_a_plus15(4);
                let mut count = width(self);
                while count != 0 {
                    self.object_fill_nx1(count, source[0], dst);
                    dst = Self::offset_words(dst, -63);
                    count -= 1;
                }
            }
            0xa4 => self.object_hole(&source, dst),
            0xb0 | 0xb1 => {
                self.object_size_a_to_a_plus15(8);
                self.object_fill_nx1(width(self), source[0], dst);
            }
            0xb5 => {
                source = self.hooks.dungeon_object_data(0xb16 / 2).to_vec();
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    self.room_draw_object_nx4(2, &source, dst);
                    dst += 4;
                }
            }
            0xbc | 0xbd => {
                self.room_draw_get_object_size_1to16();
                for _ in 0..width(self) {
                    if index == 0xbc {
                        self.room_draw_single_pot(&source, dst, dsto);
                    } else {
                        self.room_draw_hammer_peg_single(&source, dst, dsto);
                    }
                    dst += 4;
                    dsto = dsto.wrapping_add(2);
                }
            }
            0xc0 | 0xc2 => {
                let tile = source[0];
                for by in 0..=usize::from(self.r16(0xb4)) {
                    for bx in 0..=usize::from(self.r16(0xb2)) {
                        for y in 0..4 {
                            for x in 0..4 {
                                self.w16(dst + xy(bx * 4 + x, by * 4 + y) * 2, tile);
                            }
                        }
                    }
                }
            }
            0xc1 => {
                let object_width = self.r16(0xb2).wrapping_add(4);
                let object_height = self.r16(0xb4).wrapping_add(1);
                self.w16(0xb2, object_width);
                self.w16(0xb4, object_height);
                let origin = dst;
                self.room_draw_1x3_rightwards(3, &source, dst);
                dst += 6;
                for _ in 0..object_width {
                    self.room_draw_1x3_rightwards(2, &source[9..], dst);
                    dst += 4;
                }
                self.room_draw_1x3_rightwards(3, &source[15..], dst);
                dst = origin + 384;
                for _ in 0..object_height {
                    let mut row = dst;
                    self.object_draw_3x2(&source[24..], row);
                    row += 6;
                    for _ in 0..object_width {
                        self.room_draw_rightwards_2x2(&source[30..], row);
                        row += 4;
                    }
                    self.object_draw_3x2(&source[34..], row);
                    dst += 256;
                }
                let bottom_origin = dst;
                self.room_draw_1x3_rightwards(3, &source[40..], dst);
                dst += 6;
                for _ in 0..object_width {
                    self.room_draw_1x3_rightwards(2, &source[49..], dst);
                    dst += 4;
                }
                self.room_draw_1x3_rightwards(3, &source[55..], dst);
                let overlay = self.hooks.dungeon_object_data(0x590 / 2).to_vec();
                let overlay_dst = Self::offset_words(
                    bottom_origin,
                    (object_width + 2) as isize - (object_height + 1) as isize * 64,
                );
                self.room_draw_rightwards_2x2(&overlay, overlay_dst);
            }
            0xc3 | 0xd7 => {
                let object_width = self.r16(0xb2).wrapping_add(1);
                let object_height = self.r16(0xb4).wrapping_add(1);
                self.w16(0xb2, object_width);
                self.w16(0xb4, object_height);
                for _ in 0..object_height {
                    for bx in 0..object_width {
                        for y in 0..3 {
                            for x in 0..3 {
                                self.w16(dst + xy(usize::from(bx) * 3 + x, y) * 2, source[0]);
                            }
                        }
                    }
                    dst += 384;
                }
                self.w16(0xb4, 0);
            }
            0xc4 | 0xdb | 0xc5..=0xca | 0xd1 | 0xd2 | 0xd9 | 0xdf..=0xe8 => {
                if index == 0xc4 {
                    source = self
                        .hooks
                        .dungeon_object_data(usize::from(self.r16(0x46a)) / 2)
                        .to_vec();
                } else if index == 0xdb {
                    source = self
                        .hooks
                        .dungeon_object_data(usize::from(self.r16(0x490)) / 2)
                        .to_vec();
                }
                let object_width = self.r16(0xb2).wrapping_add(1);
                let object_height = self.r16(0xb4).wrapping_add(1);
                self.w16(0xb2, object_width);
                self.w16(0xb4, object_height);
                for _ in 0..object_height {
                    self.room_draw_a_many_32x32_blocks(usize::from(object_width), &source, dst);
                    dst += 512;
                }
                self.w16(0xb4, 0);
            }
            0xcd | 0xce => {
                if !self.room_draw_check_if_wall_is_moved() {
                    return;
                }
                self.w8(0x428, self.r8(0x428).wrapping_add(1));
                let size0 = usize::from(MOVING_WALL_SIZES0[usize::from(self.r16(0xb2) & 3)]);
                let size1 = usize::from(MOVING_WALL_SIZES1[usize::from(self.r16(0xb4) & 3)]);
                self.w8(0x41e, (self.r16(0xb4) as u8).wrapping_mul(2));
                if index == 0xcd {
                    self.moving_wall_fill_replacement_buffer(
                        dsto.wrapping_sub(size1 as u16).wrapping_sub(1),
                    );
                    let wall = self.hooks.dungeon_object_data(0x3d8 / 2).to_vec();
                    let mut column = Self::offset_words(dst, -(size1 as isize));
                    for _ in 0..size1 {
                        self.w16(column, wall[0]);
                        let mut p = column;
                        for _ in 0..size0 * 2 + 4 {
                            self.w16(p + 128, wall[1]);
                            p += 128;
                        }
                        self.w16(p + 128, wall[2]);
                        column += 2;
                    }
                    let face = self.hooks.dungeon_object_data(0x72a / 2).to_vec();
                    self.room_draw_1x3_rightwards(3, &face, dst);
                    dst += 384;
                    for _ in 0..size0 {
                        self.object_draw_3x2(&face[9..], dst);
                        dst += 256;
                    }
                    self.room_draw_1x3_rightwards(3, &face[15..], dst);
                } else {
                    self.moving_wall_fill_replacement_buffer(dsto.wrapping_add(3 + size1 as u16));
                    let face = self.hooks.dungeon_object_data(0x75a / 2).to_vec();
                    let mut p = dst;
                    self.room_draw_1x3_rightwards(3, &face, p);
                    p += 384;
                    for _ in 0..size0 {
                        self.object_draw_3x2(&face[9..], p);
                        p += 256;
                    }
                    self.room_draw_1x3_rightwards(3, &face[15..], p);
                    let wall = self.hooks.dungeon_object_data(0x3d8 / 2).to_vec();
                    let mut column = dst + 6;
                    for _ in 0..size1 {
                        self.w16(column, wall[0]);
                        let mut q = column;
                        for _ in 0..size0 * 2 + 4 {
                            self.w16(q + 128, wall[1]);
                            q += 128;
                        }
                        self.w16(q + 128, wall[2]);
                        column += 2;
                    }
                }
            }
            0xd8 => {
                let object_width = self.r16(0xb2).wrapping_add(2);
                let object_height = self.r16(0xb4).wrapping_add(2);
                self.w16(0xb2, object_width);
                self.w16(0xb4, object_height);
                self.w16(0x686, object_width << 4);
                self.w16(0x684, object_height << 4);
                self.w16(0x688, (object_height << 4).wrapping_sub(24));
                self.w16(
                    0x680,
                    ((dsto & 0x3f) << 3)
                        .wrapping_add(object_width << 4)
                        .wrapping_add(self.r16(0x62c)),
                );
                self.w16(
                    0x682,
                    ((dsto & 0xfc0) >> 3)
                        .wrapping_add(object_height << 4)
                        .wrapping_add(self.r16(0x62e)),
                );
                if self.r16(0x402) & 0x800 != 0 {
                    self.w8(0xaf, 0);
                    self.w8(0x414, 0);
                    self.w16(0x440, self.r16(0x442));
                    self.w16(0x448, self.r16(0x444));
                    self.w16(0x444, 0);
                    self.w16(0x442, 0);
                    self.w16(0x49e, self.r16(0x4ae));
                    self.w16(0x4ae, 0);
                    dsto = dsto
                        .wrapping_add((object_width - 1) << 1)
                        .wrapping_add((object_height - 1) << 7);
                    let water = self.hooks.dungeon_object_data(0x1438 / 2).to_vec();
                    self.draw_water_thing(DUNG_BG2 + usize::from(dsto) * 2, &water);
                } else {
                    source = self.hooks.dungeon_object_data(0x110 / 2).to_vec();
                    for _ in 0..object_height {
                        self.room_draw_a_many_32x32_blocks(usize::from(object_width), &source, dst);
                        dst += 512;
                    }
                    self.w16(0xb4, 0);
                }
            }
            0xda => {
                let object_width = self.r16(0xb2).wrapping_add(2);
                let object_height = self.r16(0xb4).wrapping_add(2);
                self.w16(0xb2, object_width);
                self.w16(0xb4, object_height);
                self.w16(0x686, (object_width << 4).wrapping_sub(24));
                self.w16(0x688, (object_height << 4).wrapping_sub(8));
                self.w16(0x684, self.r16(0x688).wrapping_sub(24));
                self.w16(0x68a, 0);
                self.w16(
                    0x680,
                    ((dsto & 0x3f) << 3)
                        .wrapping_add(object_width << 4)
                        .wrapping_add(self.r16(0x62c)),
                );
                self.w16(
                    0x682,
                    ((dsto & 0xfc0) >> 3)
                        .wrapping_add(object_height << 4)
                        .wrapping_add(self.r16(0x62e))
                        .wrapping_sub(8),
                );
                if self.r16(0x402) & 0x800 != 0 {
                    self.w8(0xaf, 0);
                } else {
                    self.w8(0x414, 0);
                    self.w16(0x440, self.r16(0x442));
                    self.w16(0x448, self.r16(0x444));
                    self.w16(0x444, 0);
                    self.w16(0x442, 0);
                    self.w16(0x49e, self.r16(0x4ae));
                    self.w16(0x4ae, 0);
                }
                source = self.hooks.dungeon_object_data(0x110 / 2).to_vec();
                let rows = object_height * 2 - 1;
                for _ in 0..rows {
                    let row = dst;
                    for _ in 0..object_width {
                        for y in 0..2 {
                            for x in 0..4 {
                                self.w16(dst + xy(x, y) * 2, source[y * 4 + x]);
                            }
                        }
                        dst += 8;
                    }
                    dst = row + 256;
                }
            }
            0xdc => {
                if self.r16(0xbf) == 0x4000 {
                    dsto |= 0x1000;
                }
                let object_width = self.r16(0xb2).wrapping_add(1);
                let object_height = self.r16(0xb4).wrapping_mul(2).wrapping_add(5);
                self.w16(0xb2, object_width);
                self.w16(0xb4, object_height);
                source = self.hooks.dungeon_object_data(0xab4 / 2).to_vec();
                for _ in 0..object_height {
                    self.object_chest_platform_helper(&source, usize::from(dsto));
                    dsto = dsto.wrapping_add(64);
                }
                self.object_chest_platform_helper(&source[1..], usize::from(dsto));
                dsto = dsto.wrapping_add(64);
                self.object_chest_platform_helper(&source[2..], usize::from(dsto));
                self.w16(0xb4, 0);
            }
            0xdd => {
                let object_width = self.r16(0xb2).wrapping_add(1);
                let middle = self.r16(0xb4).wrapping_mul(2).wrapping_add(1);
                self.w16(0xb2, object_width);
                self.w16(0xb4, middle);
                self.object_table_helper(&source, dst);
                dst += 128;
                for _ in 0..middle {
                    self.object_table_helper(&source[4..], dst);
                    dst += 128;
                }
                self.object_table_helper(&source[8..], dst);
                dst += 128;
                self.object_table_helper(&source[12..], dst);
                self.w16(0xb4, 0);
            }
            0xde => {
                let object_width = self.r16(0xb2).wrapping_add(1);
                let object_height = self.r16(0xb4).wrapping_add(1);
                self.w16(0xb2, object_width);
                self.w16(0xb4, object_height);
                for _ in 0..object_height {
                    let mut row = dst;
                    for _ in 0..object_width {
                        self.room_draw_rightwards_2x2(&source, row);
                        row += 4;
                    }
                    dst += 256;
                }
                self.w16(0xb4, 0);
            }
            0xcb | 0xcc | 0xcf | 0xd0 | 0xd3..=0xd6 | 0xe9..=0xf7 | 0xf8..=0xff => {
                panic!("invalid subtype-1 object {index:#04x}")
            }
        }
    }

    pub fn load_type1_object_subtype2(&mut self, index: u8, destination: usize, dsto: u16) {
        let offset = OBJECT_SUBTYPE2_PARAMS[usize::from(index)];
        let source = self
            .hooks
            .dungeon_object_data(usize::from(offset >> 1))
            .to_vec();
        match index {
            0x00..=0x07 | 0x1c | 0x24 | 0x25 | 0x29 => {
                self.room_draw_object_nx4(4, &source, destination);
            }
            0x08..=0x0f => self.object_draw_nx4_both_bgs(4, &source, usize::from(dsto)),
            0x10..=0x13 => self.object_draw_nx4_both_bgs(3, &source, usize::from(dsto)),
            0x14..=0x17 => self.object_draw_nx3_both_bgs(4, &source, usize::from(dsto)),
            0x18..=0x1b | 0x27 | 0x2b | 0x34 => {
                self.room_draw_rightwards_2x2(&source, destination);
            }
            0x1d | 0x21 | 0x26 => self.room_draw_1x3_rightwards(2, &source, destination),
            0x1e => self.room_draw_rightwards_2x2(&source, destination),
            0x1f => {
                let count = self.r16(0x432);
                let layer = if self.r16(0xbf) == 0x4000 { 0 } else { 0x1000 };
                self.w16(0x6a0 + usize::from(count), dsto | layer);
                self.w16(0x432, count.wrapping_add(2));
                self.room_draw_rightwards_2x2(&source, destination);
            }
            0x20 => {
                self.w8(0x45a, self.r8(0x45a).wrapping_add(1));
                self.room_draw_rightwards_2x2(&source, destination);
            }
            0x22 | 0x28 => self.object_draw_5x4(&source, destination),
            0x23 => self.room_draw_1x3_rightwards(4, &source, destination),
            0x2a => {
                self.w16(0xb2, 1);
                self.room_draw_downwards_4x2_variable_spacing(1, &source, destination);
            }
            0x2c => self.room_draw_1x3_rightwards(6, &source, destination),
            0x2d..=0x3f => {
                self.load_type1_object_subtype2_stairs(index, destination, dsto, &source)
            }
            _ => panic!("invalid subtype-2 object"),
        }
    }

    /// Exact translation of `LoadType1ObjectSubtype3`.
    pub fn load_type1_object_subtype3(&mut self, index: u8, dst: usize, dsto: u16) {
        let offset = OBJECT_SUBTYPE3_PARAMS[usize::from(index)];
        let mut source = self
            .hooks
            .dungeon_object_data(usize::from(offset >> 1))
            .to_vec();
        match index {
            0x00 => {
                let room = usize::from(self.r16(0xa0));
                let open = if self.r8(0xaf) == 27 {
                    self.r16(0xf000 + room * 2) & 0x100 != 0
                } else {
                    self.r8(0xaf) == 25 && self.r16(0x402) & 0x800 != 0
                };
                if open {
                    source = self.hooks.dungeon_object_data(0x162c / 2).to_vec();
                    self.room_draw_water_holding_object(5, &source, dst);
                } else {
                    self.w16(0x47c, dsto.wrapping_mul(2));
                    self.room_draw_water_holding_object(3, &source, dst);
                }
            }
            0x01 => {
                source = self.hooks.dungeon_object_data(0x162c / 2).to_vec();
                self.room_draw_water_holding_object(5, &source, dst);
            }
            0x02 => self.room_draw_water_holding_object(7, &source, dst),
            0x03 | 0x0e => {
                self.w8(0x3f4, self.r8(0x3f4).wrapping_add(1));
                self.w16(dst, source[0]);
            }
            0x04..=0x0c | 0x0f => self.w16(dst, source[0]),
            0x0d | 0x17 => self.load_subtype3_prison_cell(dsto),
            0x10
            | 0x11
            | 0x13
            | 0x1a
            | 0x22..=0x25
            | 0x3e..=0x46
            | 0x49
            | 0x4a
            | 0x4f..=0x53
            | 0x56..=0x59
            | 0x5e
            | 0x5f
            | 0x63..=0x65
            | 0x75
            | 0x7c..=0x7e => {
                self.room_draw_rightwards_2x2(&source, dst);
            }
            0x12 => {
                if self.r16(0x402) & 0x1000 != 0 {
                    return;
                }
                source = self.hooks.dungeon_object_data(0x1dd6 / 2).to_vec();
                let layer = if self.r16(0xbf) == 0x4000 { 0x1000 } else { 0 };
                let base = usize::from(dsto | layer);
                for column in 0..3 {
                    for row in [0, 3, 6] {
                        self.set_bg2(base + xy(column * 2, row), source[0]);
                    }
                    for row in [1, 4, 7] {
                        self.set_bg2(base + xy(column * 2, row), source[1]);
                    }
                }
            }
            0x14 | 0x4e | 0x67 | 0x68 | 0x6c | 0x6d | 0x79 => {
                self.room_draw_1x3_rightwards(4, &source, dst);
            }
            0x15 => {
                if self.r16(0x402) & 0x8000 == 0 {
                    source = self.hooks.dungeon_object_data(0x1dfa / 2).to_vec();
                    self.room_draw_some_big_decors(10, &source, usize::from(dsto));
                }
            }
            0x16 => self.room_draw_hammer_peg_single(&source, dst, dsto),
            0x18 => {
                let count = self.r16(0x498);
                let slot = usize::from(count >> 1);
                self.w16(0x498, count.wrapping_add(2));
                if self.r16(0x402) & CHEST_OPEN_MASKS[slot] == 0 {
                    self.w16(0x6e0 + slot * 2, dsto.wrapping_mul(2));
                    source = self.hooks.dungeon_object_data(0x1494 / 2).to_vec();
                    self.room_draw_rightwards_2x2(&source, dst);
                } else {
                    self.w16(0x6e0 + slot * 2, 0);
                }
            }
            0x19 => self.load_subtype3_chest(dst, dsto),
            0x1b | 0x1c => {
                let (counter, table) = if index == 0x1b {
                    (0x49a, 0x6b8)
                } else {
                    (0x49c, 0x6ec)
                };
                let count = self.r16(counter);
                self.w16(table + usize::from(count), dsto);
                self.w16(counter, count.wrapping_add(2));
                self.draw_subtype3_stair_both_bgs(&source, dsto);
            }
            0x1d => self.load_subtype3_wet_stairs(&source, dst, dsto),
            0x1e..=0x21 => {
                let counter = match index {
                    0x1e => 0x4a2,
                    0x1f => 0x4a6,
                    0x20 => 0x4a4,
                    _ => 0x4a8,
                };
                let old = self.r16(counter);
                self.w16(0x6b0 + usize::from(old), dsto);
                let next = old.wrapping_add(2);
                let propagated: &[usize] = match index {
                    0x1e => &[0x4a2, 0x4a4, 0x43a, 0x480, 0x484, 0x4a6, 0x4a8],
                    0x1f => &[0x4a6, 0x4a8],
                    0x20 => &[0x4a4, 0x43a, 0x480, 0x484, 0x4a6, 0x4a8],
                    _ => &[0x4a8],
                };
                for &address in propagated {
                    self.w16(address, next);
                }
                self.room_draw_object_nx4(4, &source, dst);
            }
            0x26..=0x29 => self.load_subtype3_lower_stairs(index, &source, dsto),
            0x2a => {
                for (destination, source_offset) in [
                    (0x514, 0x16dc),
                    (0x554, 0x17f6),
                    (0x1514, 0x1914),
                    (0x1554, 0x1a2a),
                ] {
                    self.room_draw_single_lamp_cone(destination, source_offset);
                }
            }
            0x2b => self.draw_big_gray_segment(0x1010, &source, dst, dsto),
            0x2c => {
                for (state, source_offset, delta) in [
                    (0x2020, 0xe62, 0u16),
                    (0x2121, 0xe6a, 2),
                    (0x2222, 0xe72, 128),
                    (0x2323, 0xe7a, 130),
                ] {
                    source = self.hooks.dungeon_object_data(source_offset / 2).to_vec();
                    self.draw_big_gray_segment(
                        state,
                        &source,
                        dst + usize::from(delta) * 2,
                        dsto.wrapping_add(delta),
                    );
                }
            }
            0x2d => self.load_subtype3_agahnim_altar(dsto),
            0x2e => self.room_draw_agahnims_windows(usize::from(dsto)),
            0x2f => self.room_draw_single_pot(&source, dst, dsto),
            0x30 => self.draw_big_gray_segment(0x1212, &source, dst, dsto),
            0x31 => {
                let count = self.r16(0x496);
                let slot = usize::from(count >> 1);
                let layer = if self.r16(0xbf) == 0x4000 { 0x2000 } else { 0 };
                self.w16(0x6e0 + slot * 2, dsto.wrapping_mul(2) | 0x8000 | layer);
                let open = self.r16(0x402) & CHEST_OPEN_MASKS[slot] != 0;
                if open {
                    self.w16(0x6e0 + slot * 2, 0);
                }
                let next = count.wrapping_add(2);
                self.w16(0x496, next);
                self.w16(0x498, next);
                source = self
                    .hooks
                    .dungeon_object_data(if open { 0x14c4 / 2 } else { 0x14ac / 2 })
                    .to_vec();
                self.room_draw_1x3_rightwards(4, &source, dst);
            }
            0x32 => self.room_draw_1x3_rightwards(4, &source, dst),
            0x33 => {
                let room = usize::from(self.r16(0xa0));
                if self.r8(0xaf) == 27 && self.r16(0xf000 + room * 2) & 0x100 == 0 {
                    self.w8(0x414, 0);
                    self.load_subtype3_wet_stairs(&source, dst, dsto);
                } else {
                    if self.r8(0xaf) == 27 {
                        self.w8(0x99, 2);
                        self.w8(0x9a, 0x62);
                    }
                    let count = self.r16(0x4ae);
                    self.w16(0x6ec + usize::from(count), dsto);
                    self.w16(0x4ae, count.wrapping_add(2));
                    self.room_draw_4x4(&source, dst);
                }
            }
            0x34..=0x39 => panic!("invalid subtype-3 object {index:#04x}"),
            0x3a | 0x3b => {
                self.room_draw_1x3_rightwards(4, &source, dst);
                self.room_draw_1x3_rightwards(4, &source[12..], dst + xy(0, 3) * 2);
            }
            0x3c | 0x3d | 0x5c => self.room_draw_object_nx4(6, &source, dst),
            0x47 => self.room_draw_bombable_floor(&source, dst, dsto),
            0x48 | 0x66 | 0x6b | 0x7a => self.room_draw_4x4(&source, dst),
            0x4b | 0x76 | 0x77 => self.room_draw_1x3_rightwards(8, &source, dst),
            0x4c => {
                source = self.hooks.dungeon_object_data(0x1f92 / 2).to_vec();
                self.room_draw_some_big_decors(6, &source, usize::from(dsto));
            }
            0x4d | 0x5d => self.room_draw_1x3_rightwards(6, &source, dst),
            0x54 => self.room_draw_fortune_teller_room(usize::from(dsto)),
            0x55 | 0x5b => {
                for (column, &tile) in source[..3].iter().enumerate() {
                    self.w16(dst + xy(column, 0) * 2, tile);
                }
                for row in 1..=3 {
                    for (column, &tile) in source[3..6].iter().enumerate() {
                        self.w16(dst + xy(column, row) * 2, tile);
                    }
                }
                for (column, &tile) in source[6..9].iter().enumerate() {
                    self.w16(dst + xy(column, 4) * 2, tile);
                }
            }
            0x5a => self.room_draw_water_holding_object(2, &source, dst),
            0x60 | 0x61 => {
                self.room_draw_1x3_rightwards(3, &source, dst);
                self.room_draw_1x3_rightwards(3, &source[9..], dst + xy(0, 3) * 2);
            }
            0x62 => self.load_subtype3_large_bg1_object(dsto),
            0x69 | 0x6a | 0x6e | 0x6f => self.room_draw_object_nx4(3, &source, dst),
            0x70 => {
                self.room_draw_4x4(&source, dst);
                source = self.hooks.dungeon_object_data(0x2376 / 2).to_vec();
                self.room_draw_4x4(&source, dst + xy(0, 2) * 2);
                source = self.hooks.dungeon_object_data(0x2396 / 2).to_vec();
                self.room_draw_4x4(&source, dst + xy(0, 6) * 2);
            }
            0x71 => {
                if self.r16(0xf000 + 101 * 2) & 0x100 != 0 {
                    self.object_draw_8x8(&source, dst);
                }
            }
            0x72 => {
                if self.r16(0x402) & 0x8000 == 0 {
                    self.room_draw_some_big_decors(10, &source, usize::from(dsto));
                }
            }
            0x73 => {
                source = self.hooks.dungeon_object_data(0xe0 / 2).to_vec();
                self.room_draw_floor_chunks(&source);
            }
            0x74 => self.object_draw_8x8(&source, dst),
            0x78 => {
                self.room_draw_4x4(&source, dst);
                self.room_draw_4x4(&source[16..], Self::offset_words(dst, 254));
                self.room_draw_4x4(&source[16..], Self::offset_words(dst, 258));
            }
            0x7b => {
                self.room_draw_a_many_32x32_blocks(5, &source, dst);
                self.room_draw_a_many_32x32_blocks(5, &source, dst + xy(0, 4) * 2);
            }
            _ => panic!("invalid subtype-3 object {index:#04x}"),
        }
    }

    fn load_subtype3_prison_cell(&mut self, dsto: u16) {
        let source = self.object_data::<6>(0x1488);
        let layer = if self.r16(0xbf) == 0x4000 { 0x1000 } else { 0 };
        let base = usize::from(dsto | layer);
        for column in 0..5 {
            self.set_bg2(base + xy(2 + column, 0), source[1]);
            self.set_bg2(base + xy(9 + column, 0), source[1]);
            for (row, source_index) in [(1, 2), (2, 4), (3, 5)] {
                self.set_bg2(base + xy(2 + column, row), source[source_index]);
                self.set_bg2(base + xy(9 + column, row), source[source_index] | 0x4000);
            }
        }
        self.set_bg2(base, source[0]);
        self.set_bg2(base + xy(15, 0), source[0] | 0x4000);
        for column in [1, 7, 8, 14] {
            self.set_bg2(base + xy(column, 0), source[1]);
        }
        self.set_bg2(base + xy(1, 2), source[3]);
        self.set_bg2(base + xy(14, 2), source[3] | 0x4000);
    }

    fn load_subtype3_chest(&mut self, dst: usize, dsto: u16) {
        if self.r8(0x10) == 26 {
            return;
        }
        let count = self.r16(0x496).wrapping_add(2);
        let slot = usize::from(self.r16(0x496) >> 1);
        self.w16(0x496, count);
        self.w16(0x498, count);
        let tag_matches =
            |tag: u8| tag == 0x27 || tag == 0x3c || tag == 0x3e || (0x29..0x33).contains(&tag);
        let tag_index = if tag_matches(self.r8(0xae)) {
            Some(0usize)
        } else if tag_matches(self.r8(0xaf)) {
            Some(1usize)
        } else {
            None
        };
        let layer = if self.r16(0xbf) == 0x4000 { 0x1000 } else { 0 };
        self.w16(0x6e0 + slot * 2, (dsto | layer).wrapping_mul(2));
        if self.r16(0x402) & CHEST_OPEN_MASKS[slot] == 0 {
            if let Some(tag) = tag_index {
                if self.r16(0x402) & CHEST_OPEN_MASKS[tag] == 0 {
                    return;
                }
                self.w8(0xae + tag, 0);
            }
            let source = self.object_data::<4>(0x149c);
            self.room_draw_rightwards_2x2(&source, dst);
        } else {
            self.w16(0x6e0 + slot * 2, 0);
            if let Some(tag) = tag_index {
                self.w8(0xae + tag, 0);
            }
            let source = self.object_data::<4>(0x14a4);
            self.room_draw_rightwards_2x2(&source, dst);
        }
    }

    fn draw_subtype3_stair_both_bgs(&mut self, source: &[u16], dsto: u16) {
        for column in 0..4 {
            for row in 0..4 {
                self.write_both_bgs(dsto, column, row, source[column * 4 + row]);
            }
        }
    }

    fn load_subtype3_wet_stairs(&mut self, source: &[u16], dst: usize, dsto: u16) {
        let count = self.r16(0x49e);
        self.w16(0x6ec + usize::from(count), dsto);
        self.w16(0x49e, count.wrapping_add(2));
        self.room_draw_4x4(source, dst);
    }

    fn load_subtype3_lower_stairs(&mut self, index: u8, source: &[u16], dsto: u16) {
        let counter = match index {
            0x26 => 0x4a2,
            0x27 => 0x4a6,
            0x28 => 0x4a4,
            0x29 => 0x4a8,
            _ => unreachable!(),
        };
        let old = self.r16(counter);
        let layer = if self.r16(0xbf) == 0x4000 { 0x1000 } else { 0 };
        self.w16(0x6b0 + usize::from(old), dsto | layer);
        let next = old.wrapping_add(2);
        let propagated: &[usize] = match index {
            0x26 => &[0x4a2, 0x4a4, 0x43a, 0x480, 0x484, 0x4a6, 0x4a8],
            0x27 => &[0x4a6, 0x4a8],
            0x28 => &[0x4a4, 0x43a, 0x480, 0x484, 0x4a6, 0x4a8],
            _ => &[0x4a8],
        };
        for &address in propagated {
            self.w16(address, next);
        }
        let down = index >= 0x28;
        for column in 0..4 {
            if down {
                for row in 0..3 {
                    self.set_bg1(
                        usize::from(dsto) + xy(column, row),
                        source[column * 4 + row],
                    );
                }
                let value = source[column * 4 + 3];
                self.set_bg1(usize::from(dsto) + xy(column, 3), value);
                self.set_bg2(usize::from(dsto) + xy(column, 3), value);
            } else {
                let value = source[column * 4];
                self.set_bg1(usize::from(dsto) + column, value);
                self.set_bg2(usize::from(dsto) + column, value);
                for row in 1..4 {
                    self.set_bg1(
                        usize::from(dsto) + xy(column, row),
                        source[column * 4 + row],
                    );
                }
            }
        }
        let priority_base = if down {
            dsto.wrapping_add(256)
        } else {
            dsto.wrapping_sub(256)
        };
        for row in 0..4 {
            let address = DUNG_BG2 + usize::from(priority_base.wrapping_add((row * 64) as u16)) * 2;
            self.w16(address, self.r16(address) | 0x2000);
        }
    }

    fn load_subtype3_agahnim_altar(&mut self, dsto: u16) {
        let source = self.object_data::<84>(0x1b4a);
        let base = usize::from(dsto);
        for row in 0..14 {
            let values = [
                source[row],
                source[14 + row],
                source[28 + row],
                source[42 + row],
                source[56 + row],
                source[70 + row],
            ];
            self.set_bg2(base + xy(0, row), values[0]);
            self.set_bg2(base + xy(13, row), values[0] | 0x4000);
            for column in [1, 2] {
                self.set_bg2(base + xy(column, row), values[1]);
            }
            for column in [11, 12] {
                self.set_bg2(base + xy(column, row), values[1] ^ 0x4000);
            }
            for (column, value) in [
                (3, values[2]),
                (4, values[3]),
                (5, values[4]),
                (6, values[5]),
            ] {
                self.set_bg2(base + xy(column, row), value);
                self.set_bg2(base + xy(13 - column, row), value ^ 0x4000);
            }
        }
    }

    fn load_subtype3_large_bg1_object(&mut self, dsto: u16) {
        let source = self.object_data::<242>(0x20f6);
        let base = usize::from(dsto);
        for column in 0..22 {
            for row in 0..11 {
                self.set_bg1(base + xy(column, row), source[column * 11 + row]);
            }
        }
        let trim = self.object_data::<6>(0x22da);
        for column in 0..3 {
            self.set_bg1(base + xy(9 + column, 11), trim[column]);
            self.set_bg1(base + xy(9 + column, 12), trim[column + 3]);
        }
    }

    fn load_type1_object_subtype2_stairs(
        &mut self,
        index: u8,
        destination: usize,
        dsto: u16,
        source: &[u16],
    ) {
        match index {
            0x2d => self.record_floor_stairs(0x438, dsto, destination, 0x1088, true),
            0x2e | 0x2f => self.record_floor_stairs(0x43a, dsto, destination, 0x10a8, false),
            0x30 | 0x3d => panic!("unused subtype-2 object"),
            0x31 => {
                let old = self.r16(0x43e);
                self.w16(0x6b8 + usize::from(old), dsto);
                let next = old.wrapping_add(2);
                for address in [0x43e, 0x446, 0x448] {
                    self.w16(address, next);
                }
                self.object_draw_nx4_both_bgs(4, source, usize::from(dsto));
            }
            0x32 => self.record_non_submerged_stairs(source, destination, dsto),
            0x33 => {
                let room = usize::from(self.r16(0xa0));
                if self.r8(0xaf) == 27 && self.r16(0xf000 + room * 2) & 0x100 == 0 {
                    self.w8(0x414, 0);
                    let tiles = self.hooks.dungeon_object_data(0x10c8 / 2).to_vec();
                    self.record_non_submerged_stairs(&tiles, destination, dsto);
                } else {
                    let old = self.r16(0x442);
                    self.w16(0x6b8 + usize::from(old), dsto);
                    let next = old.wrapping_add(2);
                    self.w16(0x442, next);
                    self.w16(0x444, next);
                    let tiles = self.hooks.dungeon_object_data(0x10c8 / 2).to_vec();
                    self.room_draw_4x4(&tiles, destination);
                }
            }
            0x34 => self.room_draw_rightwards_2x2(source, destination),
            0x35 | 0x36 => self.load_water_ladder(index, destination, dsto),
            0x37 => self.load_large_water_gate(source, destination, dsto),
            0x38..=0x3b => self.load_wall_staircase(index, destination, dsto),
            0x3c => self.load_sanctuary_wall(source, destination, dsto),
            0x3e => self.room_draw_1x3_rightwards(6, source, destination),
            0x3f => {
                let layer = if self.r16(0xbf) == 0x4000 { 0 } else { 0x1000 };
                let base = usize::from(dsto | layer);
                for column in 0..8 {
                    for row in 0..7 {
                        self.set_bg2(base + xy(column, row), source[column * 7 + row]);
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn record_floor_stairs(&mut self, counter: usize, dsto: u16, dst: usize, src: usize, up: bool) {
        let old = self.r16(counter);
        let layer = if self.r16(0xbf) == 0x4000 { 0 } else { 0x1000 };
        self.w16(0x6b0 + usize::from(old), dsto | layer);
        let next = old.wrapping_add(2);
        let addresses: &[usize] = if up {
            &[
                0x438, 0x47e, 0x482, 0x4a2, 0x4a4, 0x43a, 0x480, 0x484, 0x4a6, 0x4a8,
            ]
        } else {
            &[0x43a, 0x480, 0x484, 0x4a6, 0x4a8]
        };
        for &address in addresses {
            self.w16(address, next);
        }
        let tiles = self.hooks.dungeon_object_data(src / 2).to_vec();
        self.room_draw_4x4(&tiles, dst);
    }

    fn record_non_submerged_stairs(&mut self, source: &[u16], dst: usize, dsto: u16) {
        let old = self.r16(0x440);
        self.w16(0x6b8 + usize::from(old), dsto);
        let next = old.wrapping_add(2);
        for address in [0x440, 0x446, 0x448] {
            self.w16(address, next);
        }
        self.room_draw_4x4(source, dst);
    }

    fn load_water_ladder(&mut self, kind: u8, dst: usize, dsto: u16) {
        let room = usize::from(self.r16(0xa0));
        let inactive =
            kind == 0x36 || self.r8(0xaf) == 27 && self.r16(0xf000 + room * 2) & 0x100 == 0;
        let tiles = self.hooks.dungeon_object_data(0x1108 / 2).to_vec();
        if inactive {
            let old = self.r16(0x446);
            self.w16(0x6b8 + usize::from(old), dsto);
            let next = old.wrapping_add(2);
            self.w16(0x446, next);
            self.w16(0x448, next);
            self.object_draw_4x2_both_bgs(&tiles, usize::from(dsto));
        } else {
            let old = self.r16(0x444);
            self.w16(0x6b8 + usize::from(old), dsto);
            self.w16(0x444, old.wrapping_add(2));
            self.w16(0xb2, 1);
            self.room_draw_downwards_4x2_variable_spacing(1, &tiles, dst);
        }
    }

    fn load_large_water_gate(&mut self, source: &[u16], dst: usize, dsto: u16) {
        if self.r16(0x402) & 0x800 == 0 {
            self.room_draw_object_nx4(10, source, dst);
            self.w8(0x470, 0xf);
            self.w16(0x472, dsto.wrapping_mul(2));
        } else {
            let tiles = self.hooks.dungeon_object_data(0x13e8 / 2).to_vec();
            self.room_draw_object_nx4(10, &tiles, dst);
            let saved = (self.r16(0xb7), self.r16(0xba), self.r8(0xb9));
            self.room_tag_operate_water_flooring();
            self.w8(0xb9, saved.2);
            self.w16(0xba, saved.1);
            self.w16(0xb7, saved.0);
        }
    }

    fn load_wall_staircase(&mut self, kind: u8, dst: usize, dsto: u16) {
        let (counter, src, bg1, prop): (usize, usize, bool, &[usize]) = match kind {
            0x38 => (
                0x47e,
                0x1148,
                false,
                &[
                    0x47e, 0x482, 0x4a2, 0x4a4, 0x43a, 0x480, 0x484, 0x4a6, 0x4a8,
                ],
            ),
            0x39 => (0x480, 0x1160, false, &[0x480, 0x484, 0x4a6, 0x4a8]),
            0x3a => (
                0x482,
                0x1178,
                true,
                &[0x482, 0x4a2, 0x4a4, 0x43a, 0x480, 0x484, 0x4a6, 0x4a8],
            ),
            _ => (0x484, 0x1190, true, &[0x484, 0x4a6, 0x4a8]),
        };
        let old = self.r16(counter);
        let layer = if self.r16(0xbf) == 0x4000 { 0 } else { 0x1000 };
        self.w16(0x6b0 + usize::from(old), dsto.wrapping_sub(0x40) | layer);
        for &a in prop {
            self.w16(a, old.wrapping_add(2));
        }
        let tiles = self.hooks.dungeon_object_data(src / 2).to_vec();
        self.room_draw_1x3_rightwards(4, &tiles, dst);
        let base = if bg1 { DUNG_BG1 } else { DUNG_BG2 };
        for p in [dsto.wrapping_sub(1), dsto.wrapping_add(4)] {
            let a = base + usize::from(p) * 2;
            self.w16(a, self.r16(a) | 0x2000);
        }
    }

    fn load_sanctuary_wall(&mut self, source: &[u16], dst: usize, dsto: u16) {
        let mut p = usize::from(dsto);
        for row in 0..6 {
            for o in [0, 4, 8, 14, 18, 22] {
                self.set_bg2(p + o, source[row]);
            }
            for o in [1, 5, 9, 15, 19, 23] {
                self.set_bg2(p + o, source[row] | 0x4000);
            }
            for o in [2, 6, 16, 20] {
                self.set_bg2(p + o, source[6 + row]);
            }
            for o in [3, 7, 17, 21] {
                self.set_bg2(p + o, source[6 + row] | 0x4000);
            }
            p += xy(0, 1);
        }
        self.room_draw_1x3_rightwards(4, &source[12..], dst + 20);
    }

    pub fn room_draw_rightwards_2x2(&mut self, src: &[u16], dst: usize) {
        assert!(src.len() >= 4);
        self.w16(dst + xy(0, 0) * 2, src[0]);
        self.w16(dst + xy(0, 1) * 2, src[1]);
        self.w16(dst + xy(1, 0) * 2, src[2]);
        self.w16(dst + xy(1, 1) * 2, src[3]);
    }

    pub fn object_draw_3x2(&mut self, src: &[u16], dst: usize) {
        assert!(src.len() >= 6);
        for row in 0..2 {
            for column in 0..3 {
                self.w16(dst + xy(column, row) * 2, src[row * 3 + column]);
            }
        }
    }

    pub fn room_draw_water_holding_object(&mut self, count: usize, src: &[u16], dst: usize) {
        assert!(src.len() >= count * 4);
        for row in 0..count {
            for column in 0..4 {
                self.w16(dst + xy(column, row) * 2, src[row * 4 + column]);
            }
        }
    }

    pub fn room_draw_some_big_decors(&mut self, width: usize, src: &[u16], destination: usize) {
        assert!(src.len() >= width * 8);
        let destination = destination + if self.r16(0xbf) == 0x4000 { 0x1000 } else { 0 };
        for row in 0..8 {
            for column in 0..width {
                self.set_bg2(destination + xy(column, row), src[row * width + column]);
            }
        }
    }

    pub fn object_draw_8x8(&mut self, src: &[u16], dst: usize) {
        assert!(src.len() >= 64);
        self.room_draw_4x4(&src[0..16], dst + xy(0, 0) * 2);
        self.room_draw_4x4(&src[16..32], dst + xy(4, 0) * 2);
        self.room_draw_4x4(&src[32..48], dst + xy(0, 4) * 2);
        self.room_draw_4x4(&src[48..64], dst + xy(4, 4) * 2);
    }

    pub fn room_draw_floor_chunks(&mut self, src: &[u16]) {
        assert!(src.len() >= 8);
        for quadrant in DUNGEON_QUADRANT_OFFSETS {
            let mut destination = self.dsto_ptr(quadrant / 2);
            for _ in 0..8 {
                self.room_draw_a_many_32x32_blocks(8, src, destination);
                destination += xy(0, 4) * 2;
            }
        }
    }

    pub fn room_draw_draw_floors(&mut self, level_data: &[u8]) {
        let offset = usize::from(self.r16(0xba));
        assert!(offset < level_data.len());
        for (index, &value) in DUNGEON_DRAW_OBJECT_OFFSETS_BG2.iter().enumerate() {
            self.w8(0xbf + index, value);
        }
        let floor_types = level_data[offset];
        self.w16(0xba, self.r16(0xba).wrapping_add(1));
        let floor1 = floor_types & 0xf0;
        self.w16(0x490, u16::from(floor1));
        let source = self.hooks.dungeon_object_data(usize::from(floor1) / 2);
        assert!(source.len() >= 8);
        let mut tiles = [0u16; 8];
        tiles.copy_from_slice(&source[..8]);
        self.room_draw_floor_chunks(&tiles);

        for (index, &value) in DUNGEON_DRAW_OBJECT_OFFSETS_BG1.iter().enumerate() {
            self.w8(0xbf + index, value);
        }
        let floor2 = (floor_types & 0x0f) << 4;
        self.w16(0x46a, u16::from(floor2));
        let source = self.hooks.dungeon_object_data(usize::from(floor2) / 2);
        assert!(source.len() >= 8);
        tiles.copy_from_slice(&source[..8]);
        self.room_draw_floor_chunks(&tiles);
    }

    /// Draws one of the three object streams in a room-layout record, followed
    /// by its door stream.  The cursor at `$BA` deliberately has the same
    /// slightly unusual ownership as the original parser.
    pub fn room_draw_draw_all_objects(&mut self, level_data: &[u8]) {
        loop {
            self.w16(0xb2, 0);
            self.w16(0xb4, 0);
            let offset = usize::from(self.r16(0xba));
            assert!(offset + 1 < level_data.len());
            let descriptor = u16::from_le_bytes([level_data[offset], level_data[offset + 1]]);
            if descriptor == 0xffff {
                return;
            }
            if descriptor == 0xfff0 {
                break;
            }
            self.room_data_draw_object(descriptor, level_data);
        }
        loop {
            self.w16(0xba, self.r16(0xba).wrapping_add(2));
            let offset = usize::from(self.r16(0xba));
            assert!(offset + 1 < level_data.len());
            let descriptor = u16::from_le_bytes([level_data[offset], level_data[offset + 1]]);
            if descriptor == 0xffff {
                return;
            }
            self.room_data_draw_object_door(descriptor);
        }
    }

    pub fn room_data_draw_object(&mut self, descriptor: u16, level_data: &[u8]) {
        let offset = usize::from(self.r16(0xba));
        assert!(offset + 2 < level_data.len());
        let mut index = level_data[offset + 2];
        self.w16(0xba, (offset as u16).wrapping_add(3));
        if descriptor & 0xfc != 0xfc {
            self.w16(0xb2, descriptor & 3);
            self.w16(0xb4, (descriptor >> 8) & 3);
            let x = usize::from((descriptor as u8) >> 2);
            let y = usize::from(descriptor >> 10);
            let destination = (y * 64 + x) as u16;
            if index < 0xf8 {
                self.load_type1_object_subtype1(index, self.dsto_ptr(destination), destination);
            } else {
                index = ((index & 7) << 4)
                    | (((descriptor >> 8) as u8 & 3) << 2)
                    | (descriptor as u8 & 3);
                self.load_type1_object_subtype3(index, self.dsto_ptr(destination), destination);
            }
        } else {
            let x = usize::from(((descriptor & 3) << 4) | ((descriptor >> 12) & 0xf));
            let y = usize::from(((descriptor >> 8) & 0xf) << 2 | u16::from(index >> 6));
            let destination = (y * 64 + x) as u16;
            self.load_type1_object_subtype2(index & 0x3f, self.dsto_ptr(destination), destination);
        }
    }

    pub fn dungeon_load_room(&mut self) {
        self.dungeon_load_header();
        self.w16(0x4b8, 0);
        self.w8(0xc208, self.r8(0x414));
        self.w8(0xc209, self.r8(0xae));
        self.w8(0x0c, 0x30);
        self.w8(0x0d, 0xff);

        // Room-construction scratch state. These are individual C globals, but
        // they form dense WRAM groups in the original memory map.
        for address in [
            0x40a, 0x41e, 0x420, 0x422, 0x424, 0x428, 0x42a, 0x42c, 0x42e, 0x430, 0x432, 0x434,
            0x436, 0x438, 0x43a, 0x43c, 0x43e, 0x440, 0x442, 0x444, 0x446, 0x448, 0x44a, 0x44c,
            0x44e, 0x450, 0x452, 0x454, 0x456, 0x45c, 0x460, 0x466, 0x470, 0x474, 0x478, 0x480,
            0x484, 0x48c, 0x492, 0x496, 0x498, 0x49a, 0x49c, 0x49e,
        ] {
            self.w16(address, 0);
        }
        self.w16(0x101a, 0xffff);
        self.ram.slice_mut(0x1980, 0x60).fill(0);
        self.ram.slice_mut(0x4f0, 16).fill(0);
        self.ram.slice_mut(0x500, 32).fill(0);
        self.ram.slice_mut(0x520, 32).fill(0);
        self.ram.slice_mut(0x540, 32).fill(0);

        let room = usize::from(self.r16(0xa0));
        let room_layout = self.hooks.dungeon_room_layout(room).to_vec();
        self.w16(0xba, 0);
        self.room_draw_draw_floors(&room_layout);
        let old_offset = self.r16(0xba);
        let layout = room_layout[usize::from(old_offset)];
        self.w8(0xa5, layout);
        let default_layout = self
            .hooks
            .default_room_layout(usize::from(layout >> 2))
            .to_vec();
        self.w16(0xba, 0);
        self.room_draw_draw_all_objects(&default_layout);
        self.w16(0xba, old_offset.wrapping_add(1));
        self.room_draw_draw_all_objects(&room_layout);
        self.w16(0xba, self.r16(0xba).wrapping_add(2));
        for (index, &value) in DUNGEON_DRAW_OBJECT_OFFSETS_BG2.iter().enumerate() {
            self.w8(0xbf + index, value);
        }
        self.room_draw_draw_all_objects(&room_layout);
        self.w16(0xba, self.r16(0xba).wrapping_add(2));
        for (index, &value) in DUNGEON_DRAW_OBJECT_OFFSETS_BG1.iter().enumerate() {
            self.w8(0xbf + index, value);
        }
        self.room_draw_draw_all_objects(&room_layout);

        for object_offset in (0..0x18c).step_by(4) {
            let (object_room, tilemap) = self.hooks.dungeon_movable_block(object_offset / 4);
            if object_room == self.r16(0xa0) {
                self.hooks.dungeon_draw_pushable_block(
                    &mut self.ram,
                    tilemap,
                    object_offset as u16,
                );
            }
        }
        self.w16(0x478, self.r16(0x480));
        let mut offset = 0usize;
        while offset != 0x120 {
            if self.hooks.dungeon_torch_data(offset / 2) == self.r16(0xa0) {
                offset += 2;
                loop {
                    let torch = self.hooks.dungeon_torch_data(offset / 2);
                    offset += 2;
                    if torch == 0xffff {
                        break;
                    }
                    self.hooks.dungeon_draw_lightable_torch(
                        &mut self.ram,
                        torch,
                        (offset - 2) as u16,
                    );
                }
                break;
            }
            offset += 2;
            loop {
                let torch = self.hooks.dungeon_torch_data(offset / 2);
                offset += 2;
                if torch == 0xffff {
                    break;
                }
            }
        }
        self.w16(0xba, 0x120);
    }

    fn prepare_quadrant_upload(&mut self, background: usize, target_adjustment: u8) {
        let table_index = usize::from((self.r8(0x418) & 0x0f).wrapping_add(self.r8(0x45c)));
        let mut source = background + usize::from(UPLOAD_BG_SRCS[table_index]);
        let mut position = 0usize;
        loop {
            loop {
                let destination = 0x1000 + position * 2;
                self.w16(destination, self.r16(source));
                self.w16(destination + 2, self.r16(source + 2));
                self.w16(destination + 64, self.r16(source + xy(0, 1) * 2));
                self.w16(destination + 66, self.r16(source + xy(1, 1) * 2));
                self.w16(destination + 128, self.r16(source + xy(0, 2) * 2));
                self.w16(destination + 130, self.r16(source + xy(1, 2) * 2));
                self.w16(destination + 192, self.r16(source + xy(0, 3) * 2));
                self.w16(destination + 194, self.r16(source + xy(1, 3) * 2));
                source += 4;
                position += 2;
                if position & 0x1f == 0 {
                    break;
                }
            }
            source += 224 * 2;
            position += 128 - 32;
            if position == 0x400 {
                break;
            }
        }
        self.w8(
            0x116,
            UPLOAD_BG_DSTS[table_index].wrapping_add(target_adjustment),
        );
        self.w8(0x17, 1);
        self.w8(0x710, 1);
    }

    pub fn dungeon_prepare_next_room_quadrant_upload(&mut self) {
        self.prepare_quadrant_upload(DUNG_BG2, 0);
        self.w8(0x45c, self.r8(0x45c).wrapping_add(4));
    }

    pub fn tile_map_prep_not_water_on_tag(&mut self) {
        self.prepare_quadrant_upload(DUNG_BG1, 0x10);
    }

    pub fn water_flood_build_one_quadrant_for_vram(&mut self) {
        assert_ne!(self.r8(0xae), 25);
        self.tile_map_prep_not_water_on_tag();
    }

    #[must_use]
    pub fn room_draw_check_if_wall_is_moved(&mut self) -> bool {
        self.w8(0x41c, 0);
        self.w8(0x41d, 0);
        self.w16(0x41a, 0);
        let tag_index = if (0x1c..0x20).contains(&self.r8(0xae)) {
            Some(0usize)
        } else if (0x1c..0x20).contains(&self.r8(0xaf)) {
            Some(1usize)
        } else {
            None
        };
        if let Some(index) = tag_index
            && self.r16(0x402) & (0x1000 >> index) != 0
        {
            self.w8(0x46c, 0);
            self.w8(0xae + index, 0);
            self.w8(0x414, 0);
            return false;
        }
        true
    }

    pub fn moving_wall_fill_replacement_buffer(&mut self, destination: u16) {
        for index in 0..64 {
            self.w16(0xc880 + index * 2, 0x1ec);
        }
        self.w16(
            0x42a,
            (destination & 0x1f) | if destination & 0x20 != 0 { 0x400 } else { 0 } | 0x1000,
        );
    }

    pub fn room_draw_single_lamp_cone(&mut self, destination_x2: u16, source_byte_offset: usize) {
        let source = self.hooks.dungeon_object_data(source_byte_offset / 2);
        assert!(source.len() >= 144);
        let mut tiles = [0u16; 144];
        tiles.copy_from_slice(&source[..144]);
        let destination = usize::from(destination_x2) / 2;
        for row in 0..12 {
            for column in 0..12 {
                self.set_bg1(destination + xy(column, row), tiles[row * 12 + column]);
            }
        }
    }

    pub fn room_draw_agahnims_windows(&mut self, destination: usize) {
        let source = self.object_data::<24>(0x1bf2);
        for column in 0..6 {
            for row in 0..4 {
                let tile = source[column * 4 + row];
                for x in [7, 13, 19] {
                    self.set_bg2(destination + xy(x + column, 4 + row), tile);
                }
            }
        }

        let source = self.object_data::<5>(0x1c22);
        for (row_offset, &tile) in source.iter().enumerate() {
            for step in 0..7 {
                self.set_bg2(destination + xy(2 + step, 10 - step + row_offset), tile);
                self.set_bg2(
                    destination + xy(23 + step, 4 + step + row_offset),
                    tile | 0x4000,
                );
            }
        }

        let source = self.object_data::<24>(0x1c2c);
        for row_offset in 0..6 {
            for column_offset in 0..4 {
                let tile = source[row_offset * 4 + column_offset];
                for y in [11, 17, 23] {
                    self.set_bg2(destination + xy(2 + column_offset, y + row_offset), tile);
                    self.set_bg2(
                        destination + xy(29 - column_offset, y + row_offset),
                        tile | 0x4000,
                    );
                }
            }
        }

        let source = self.object_data::<12>(0x1c5c);
        for column_offset in 0..6 {
            self.set_bg2(
                destination + xy(12 + column_offset, 9),
                source[column_offset],
            );
            self.set_bg2(
                destination + xy(18 + column_offset, 10),
                source[column_offset + 6],
            );
        }

        let source = self.object_data::<12>(0x1c74);
        for row_offset in 0..6 {
            for y in [14, 20] {
                self.set_bg2(destination + xy(7, y + row_offset), source[row_offset * 2]);
                self.set_bg2(
                    destination + xy(8, y + row_offset),
                    source[row_offset * 2 + 1],
                );
            }
        }

        let source = self.object_data::<25>(0x1c8c);
        for column_offset in 0..5 {
            for row_offset in 0..5 {
                self.set_bg2(
                    destination + xy(7 + column_offset, 9 + row_offset),
                    source[column_offset * 5 + row_offset],
                );
            }
        }
        for column in 14..18 {
            for row in [28, 29] {
                let address = DUNG_BG2 + (destination + xy(column, row)) * 2;
                self.w16(address, self.r16(address) | 0x2000);
            }
        }
    }

    pub fn room_draw_fortune_teller_room(&mut self, destination: usize) {
        let source = self.object_data::<26>(0x202e);
        for section in 0..6 {
            let x = section * 2;
            for row in 0..2 {
                self.set_bg2(destination + xy(x + 1, row), source[0]);
                self.set_bg2(destination + xy(x + 2, row), source[0]);
            }
            self.set_bg2(destination + xy(x + 1, 2), source[1]);
            self.set_bg2(destination + xy(x + 2, 2), source[1] | 0x4000);
        }
        for row_offset in 0..3 {
            let outer = source[2 + row_offset];
            for x in [0, 2, 10, 12] {
                self.set_bg2(destination + xy(x, 3 + row_offset), outer);
                self.set_bg2(destination + xy(x + 1, 3 + row_offset), outer | 0x4000);
            }
            let inner = source[5 + row_offset];
            for x in [4, 6, 8] {
                self.set_bg2(destination + xy(x, 3 + row_offset), inner);
                self.set_bg2(destination + xy(x + 1, 3 + row_offset), inner | 0x4000);
            }
        }
        for row in 0..2 {
            self.set_bg2(destination + xy(0, row), source[8]);
            self.set_bg2(destination + xy(13, row), source[8] | 0x4000);
        }
        self.set_bg2(destination + xy(0, 2), source[9]);
        self.set_bg2(destination + xy(13, 2), source[9] | 0x4000);

        for row_offset in 0..4 {
            for (x, source_index) in [(3, 10), (4, 14), (5, 18), (6, 22)] {
                let tile = source[source_index + row_offset];
                self.set_bg2(destination + xy(x, 10 + row_offset), tile);
                self.set_bg2(destination + xy(13 - x, 10 + row_offset), tile ^ 0x4000);
            }
        }
    }

    pub fn room_draw_make_door_parts_high_priority_y(&mut self, destination: usize) {
        for row in 0..7 {
            for column in 0..4 {
                let address = DUNG_BG2 + (destination + xy(column, row)) * 2;
                self.w16(address, self.r16(address) | 0x2000);
            }
        }
    }

    pub fn room_draw_make_door_parts_high_priority_x(&mut self, destination: usize) {
        for column in 0..5 {
            for row in 0..4 {
                let address = DUNG_BG2 + (destination + xy(column, row)) * 2;
                self.w16(address, self.r16(address) | 0x2000);
            }
        }
    }

    pub fn room_draw_make_door_high_priority_north(&mut self, mut destination: u16) {
        let original = destination;
        destination &= 0xf07f >> 1;
        loop {
            for column in 0..4 {
                let address = DUNG_BG2 + (usize::from(destination) + column) * 2;
                self.w16(address, self.r16(address) | 0x2000);
            }
            destination = destination.wrapping_add(64);
            if destination == original {
                break;
            }
        }
    }

    pub fn room_draw_make_door_high_priority_south(&mut self, mut destination: u16) {
        loop {
            for column in 0..4 {
                let address = DUNG_BG2 + (usize::from(destination) + column) * 2;
                self.w16(address, self.r16(address) | 0x2000);
            }
            destination = destination.wrapping_add(64);
            if destination & 0x7c0 == 0 {
                break;
            }
        }
    }

    pub fn room_draw_make_door_high_priority_west(&mut self, mut destination: u16) {
        let original = destination;
        destination &= 0xffe0;
        loop {
            for row in 0..4 {
                let address = DUNG_BG2 + (usize::from(destination) + xy(0, row)) * 2;
                self.w16(address, self.r16(address) | 0x2000);
            }
            destination = destination.wrapping_add(1);
            if destination == original {
                break;
            }
        }
    }

    pub fn room_draw_make_door_high_priority_east(&mut self, mut destination: u16) {
        loop {
            for row in 0..4 {
                let address = DUNG_BG2 + (usize::from(destination) + xy(0, row)) * 2;
                self.w16(address, self.r16(address) | 0x2000);
            }
            destination = destination.wrapping_add(1);
            if destination & 0x1f == 0 {
                break;
            }
        }
    }

    pub fn room_draw_downwards_4x2_variable_spacing(
        &mut self,
        increment_words: usize,
        src: &[u16],
        mut destination: usize,
    ) {
        assert!(src.len() >= 8);
        let count = self.r16(0xb2);
        for _ in 0..count {
            for row in 0..2 {
                for column in 0..4 {
                    self.w16(destination + xy(column, row) * 2, src[row * 4 + column]);
                }
            }
            destination += increment_words * 2;
        }
    }

    pub fn room_draw_draw_object_2x2_and1(&mut self, src: &[u16], destination: usize) -> usize {
        assert!(src.len() >= 5);
        for (row, &tile) in src[..5].iter().enumerate() {
            self.w16(destination + xy(0, row) * 2, tile);
        }
        destination
    }

    pub fn room_draw_rightward_shelf_end(&mut self, src: &[u16], destination: usize) -> usize {
        assert!(src.len() >= 4);
        for (row, &tile) in src[..4].iter().enumerate() {
            self.w16(destination + xy(0, row) * 2, tile);
        }
        destination
    }

    pub fn room_draw_rightward_bar_segment(&mut self, src: &[u16], destination: usize) -> usize {
        assert!(src.len() >= 3);
        for (row, &tile) in src[..3].iter().enumerate() {
            self.w16(destination + xy(0, row) * 2, tile);
        }
        destination
    }

    pub fn object_bombable_floor_helper(
        &mut self,
        state: u16,
        src: &[u16],
        src_below: &[u16],
        destination: usize,
        destination_offset: u16,
    ) {
        assert!(src.len() >= 4 && src_below.len() >= 4);
        let index = usize::from(self.r16(0x42c) >> 1);
        self.w16(0x500 + index * 2, state);
        self.w16(0x42c, self.r16(0x42c).wrapping_add(2));
        self.w16(0x520 + index * 2, self.r16(0xba));
        self.w16(
            0x540 + index * 2,
            destination_offset.wrapping_mul(2) | if self.r16(0xbf) == 0x4000 { 0x2000 } else { 0 },
        );
        for (base, &tile) in [0x560, 0x580, 0x5a0, 0x5c0].into_iter().zip(src_below) {
            self.w16(base + index * 2, tile);
        }
        self.room_draw_rightwards_2x2(src, destination);
    }

    pub fn draw_big_gray_segment(
        &mut self,
        state: u16,
        src: &[u16],
        destination: usize,
        destination_offset: u16,
    ) {
        assert!(src.len() >= 4);
        let index = usize::from(self.r16(0x42c) >> 1);
        self.w16(0x500 + index * 2, state);
        self.w16(0x42c, self.r16(0x42c).wrapping_add(2));
        self.w16(0x520 + index * 2, self.r16(0xba));
        self.w16(
            0x540 + index * 2,
            destination_offset.wrapping_mul(2) | if self.r16(0xbf) == 0x4000 { 0x2000 } else { 0 },
        );
        for (base, offset) in [(0x560, 0), (0x580, 64), (0x5a0, 1), (0x5c0, 65)] {
            self.w16(base + index * 2, self.r16(destination + offset * 2));
        }
        self.room_draw_rightwards_2x2(src, destination);
    }

    pub fn room_draw_single_pot(
        &mut self,
        src: &[u16],
        destination: usize,
        destination_offset: u16,
    ) {
        let index = usize::from(self.r16(0x42c) >> 1);
        self.w16(0x42c, self.r16(0x42c).wrapping_add(2));
        self.w16(0x500 + index * 2, 0x1111);
        self.w16(0x520 + index * 2, self.r16(0xba));
        self.w16(
            0x540 + index * 2,
            destination_offset.wrapping_mul(2) | if self.r16(0xbf) == 0x4000 { 0x2000 } else { 0 },
        );
        for (base, tile) in [
            (0x560, 0x0d0e),
            (0x580, 0x0d1e),
            (0x5a0, 0x4d0e),
            (0x5c0, 0x4d1e),
        ] {
            self.w16(base + index * 2, tile);
        }
        if self.r8(0xf3ca) != 0 {
            let dark = self.object_data::<4>(0xe92);
            self.room_draw_rightwards_2x2(&dark, destination);
        } else {
            self.room_draw_rightwards_2x2(src, destination);
        }
    }

    pub fn room_draw_hammer_peg_single(
        &mut self,
        src: &[u16],
        destination: usize,
        destination_offset: u16,
    ) {
        let index = usize::from(self.r16(0x42c) >> 1);
        self.w16(0x42c, self.r16(0x42c).wrapping_add(2));
        self.w16(0x500 + index * 2, 0x4040);
        self.w16(0x520 + index * 2, self.r16(0xba));
        self.w16(
            0x540 + index * 2,
            destination_offset.wrapping_mul(2) | if self.r16(0xbf) == 0x4000 { 0x2000 } else { 0 },
        );
        for (base, tile) in [
            (0x560, 0x19d8),
            (0x580, 0x19d9),
            (0x5a0, 0x59d8),
            (0x5c0, 0x59d9),
        ] {
            self.w16(base + index * 2, tile);
        }
        self.room_draw_rightwards_2x2(src, destination);
    }

    pub fn draw_objects_pushable_block(&mut self, destination_x2: u16, object_slot: u16) {
        let index = usize::from(self.r16(0x42c) >> 1);
        self.w16(0x42c, self.r16(0x42c).wrapping_add(2));
        self.w16(0x500 + index * 2, 0);
        self.w16(0x520 + index * 2, object_slot);
        self.w16(0x540 + index * 2, destination_x2);
        let destination = self.dsto_ptr((destination_x2 >> 1) & 0x1fff);
        for (base, offset) in [(0x560, 0), (0x580, 64), (0x5a0, 1), (0x5c0, 65)] {
            self.w16(base + index * 2, self.r16(destination + offset * 2));
        }
        let source = self.object_data::<4>(0xe52);
        self.room_draw_rightwards_2x2(&source, destination);
    }

    pub fn draw_objects_lightable_torch(&mut self, destination_x2: u16, object_slot: u16) {
        let index = usize::from(self.r16(0x42e) >> 1);
        self.w16(0x42e, self.r16(0x42e).wrapping_add(2));
        self.w16(0x540 + index * 2, destination_x2);
        self.w16(0x520 + index * 2, object_slot);
        let mut source_offset = 0xec2;
        if destination_x2 & 0x8000 != 0 {
            source_offset = 0xeca;
            if self.r8(0x45a) < 3 {
                self.w8(0x45a, self.r8(0x45a).wrapping_add(1));
            }
        }
        let source = self.object_data::<4>(source_offset);
        let destination = self.dsto_ptr((destination_x2 >> 1) & 0x1fff);
        self.room_draw_rightwards_2x2(&source, destination);
    }

    pub fn room_draw_bombable_floor(
        &mut self,
        _src: &[u16],
        destination: usize,
        destination_offset: u16,
    ) {
        if self.r16(0xa0) == 101 && self.r16(0x402) & 0x1000 != 0 {
            self.w16(0xb2, 0);
            self.w16(0xb4, 0);
            let hole = self.object_data::<1>(0x5aa);
            self.object_hole(&hole, destination);
            return;
        }
        let source = self.object_data::<16>(0x220);
        let below = self.object_data::<16>(0x5ba);
        for (index, (state, offset)) in [(0x3030, 0u16), (0x3131, 2), (0x3232, 128), (0x3333, 130)]
            .into_iter()
            .enumerate()
        {
            self.object_bombable_floor_helper(
                state,
                &source[index * 4..index * 4 + 4],
                &below[index * 4..index * 4 + 4],
                destination + usize::from(offset) * 2,
                destination_offset.wrapping_add(offset),
            );
        }
    }

    pub fn room_draw_16x16_single(&mut self, index_x2: u8) {
        let index = usize::from(index_x2 >> 1);
        let position = (self.r16(0x540 + index * 2) & 0x3fff) >> 1;
        let tiles = [
            self.r16(0x560 + index * 2),
            self.r16(0x580 + index * 2),
            self.r16(0x5a0 + index * 2),
            self.r16(0x5c0 + index * 2),
        ];
        let attribute = self
            .hooks
            .dungeon_tile_attribute(&self.ram, usize::from(tiles[3] & 0x3ff));
        self.dungeon_store_2x2(position, tiles, attribute);
    }

    pub fn dungeon_prep_sprite_induced_dma(&mut self, x: u16, y: u16, value: u8) {
        let position = ((y.wrapping_add(1) & 0x1f8) << 3) | ((x & 0x1f8) >> 3);
        let source = self.object_data::<4>(usize::from(
            PREP_SPRITE_INDUCED_DMA_SRCS[usize::from(value >> 1)],
        ));
        let upload = 0x1002 + usize::from(self.r16(0x1000));
        let attribute = self
            .hooks
            .dungeon_tile_attribute(&self.ram, usize::from(source[3] & 0x3ff));
        for (index, delta) in [0u16, 64, 1, 65].into_iter().enumerate() {
            let position = position.wrapping_add(delta);
            self.w16(upload + index * 6, Self::dungeon_map_vram_addr(position));
            self.w16(upload + index * 6 + 2, 0x100);
            self.w16(upload + index * 6 + 4, source[index]);
            self.w16(DUNG_BG2 + usize::from(position) * 2, source[index]);
            self.w8(0x12000 + usize::from(position), attribute);
        }
        self.w16(upload + 24, 0xffff);
        self.w16(0x1000, self.r16(0x1000).wrapping_add(24));
    }

    pub fn dungeon_update_tile_map_with_common_tile(&mut self, x: u16, y: u16, value: u8) {
        if value == 8 {
            self.dungeon_prep_sprite_induced_dma(x.wrapping_add(16), y, value.wrapping_add(2));
        }
        self.dungeon_prep_sprite_induced_dma(x, y, value);
        self.w8(0x14, 1);
    }

    pub fn dungeon_delete_rupee_tile(&mut self, x: u16, y: u16) {
        let position = (y & 0x1f8).wrapping_mul(8) | ((x & 0x1f8) >> 3);
        let upload = 0x1002 + usize::from(self.r16(0x1000));
        let attribute =
            u16::from(self.hooks.dungeon_tile_attribute(&self.ram, 0x190f & 0x3ff)) * 0x101;
        for (index, delta) in [0u16, 64].into_iter().enumerate() {
            let position = position.wrapping_add(delta);
            self.w16(upload + index * 6, Self::dungeon_map_vram_addr(position));
            self.w16(upload + index * 6 + 2, 0x100);
            self.w16(upload + index * 6 + 4, 0x190f);
            self.w16(DUNG_BG2 + usize::from(position) * 2, 0x190f);
            self.w16(0x12000 + usize::from(position), attribute);
        }
        self.w16(upload + 12, 0xffff);
        self.w16(0x1000, self.r16(0x1000).wrapping_add(24));
        self.w16(0x402, self.r16(0x402) | 0x1000);
        self.w8(0x14, 1);
    }

    pub fn open_big_chest(&mut self, location: u16, chest_position: &mut u16) {
        let mut position = usize::from(location >> 1);
        let source = self.object_data::<12>(0x14c4);
        for column in 0..4 {
            for row in 0..3 {
                self.set_bg2(position + xy(column, row), source[column * 3 + row]);
            }
        }
        self.hooks
            .dungeon_prep_overlay_dma_next(&mut self.ram, 0, location);
        *chest_position = location.wrapping_add(2);
        position = usize::from(location >> 1);
        for row in 0..3 {
            for column in [0usize, 2] {
                self.w16(0x12000 + position + xy(column, row), 0x2727);
            }
        }
        self.dungeon_flag_room_data_quadrants();
        self.w8(0x12f, 14);
        self.w8(0x14, 1);
        self.w8(0xb9e, 1);
    }

    pub fn open_mini_game_chest(&mut self, chest_position: &mut u16) -> u8 {
        let credits = self.r8(0x4c4);
        if credits == 0 || credits == 0xff {
            self.w16(0x1cf0, if credits == 0 { 0x163 } else { 0x162 });
            self.hooks.main_show_text_message(&mut self.ram);
            return 0xff;
        }
        self.w8(0x4c4, credits.wrapping_sub(1));
        let mut position = usize::from(
            (self.r16(0x20).wrapping_sub(4) & 0x1f8).wrapping_mul(8)
                | ((self.r16(0x22).wrapping_add(7) & 0x1f8) >> 3),
        );
        if self.r16(0x12000 + position) != 0x6363 {
            position -= 1;
            if self.r16(0x12000 + position) != 0x6363 {
                position += 2;
            }
        }
        *chest_position = (position * 2) as u16;
        self.w16(0x12000 + position, 0x202);
        self.w16(0x12000 + position + xy(0, 1), 0x202);
        let source = self.object_data::<4>(0x14a4);
        let wrong_position = position + xy(0, 2);
        for (index, delta) in [0usize, xy(0, 1), xy(1, 0), xy(1, 1)]
            .into_iter()
            .enumerate()
        {
            self.set_bg2(wrong_position + delta, source[index]);
        }
        let upload = 0x1002 + usize::from(self.r16(0x1000));
        for (index, delta) in [0usize, 64, 1, 65].into_iter().enumerate() {
            self.w16(
                upload + index * 6,
                Self::dungeon_map_vram_addr((position + delta) as u16),
            );
            self.w16(upload + index * 6 + 2, 0x100);
            self.w16(upload + index * 6 + 4, source[index]);
        }
        self.w16(upload + 24, 0xffff);
        self.w16(0x1000, self.r16(0x1000).wrapping_add(24));

        let random = self.hooks.get_random_number(&mut self.ram);
        let room = self.r8(0xa0);
        let mut choice;
        let prize = if room == 0 {
            choice = random & 0xf;
            DUNGEON_RUPEE_CHEST_MINIGAME_PRIZES[usize::from(choice)]
        } else if room == 0x18 {
            choice = 0x10 + (random & 0xf);
            DUNGEON_RUPEE_CHEST_MINIGAME_PRIZES[usize::from(choice)]
        } else {
            choice = random & 7;
            if choice >= 2 && choice == self.r8(0xc8) {
                choice = choice.wrapping_add(1) & 7;
            }
            if choice == 7 {
                if self.r16(0x402) & 0x4000 != 0 {
                    choice = 0;
                } else {
                    self.w16(0x402, self.r16(0x402) | 0x4000);
                }
            }
            DUNGEON_MINIGAME_CHEST_PRIZES1[usize::from(choice)]
        };
        self.w8(0xc8, choice);
        self.w8(0x14, 1);
        self.w8(0x12f, 14);
        prize
    }

    pub fn open_chest_for_item(&mut self, tile: u8, chest_position: &mut u16) -> u8 {
        if tile == 0x63 {
            return self.open_mini_game_chest(chest_position);
        }
        let original_index = usize::from(tile.wrapping_sub(0x58));
        let mut chest_index = original_index as i32;
        let location = self.r16(0x6e0 + original_index * 2);
        let palace = usize::from(self.r16(0x40c) >> 1);
        let has_big_key = self.r16(0xf366) & UPPER_BITMASKS[palace] != 0;

        let (position, source, attribute, data) = if location & 0x8000 != 0 {
            if !has_big_key {
                self.w16(0x1cf0, 0x7a);
                self.hooks.main_show_text_message(&mut self.ram);
                return 0xff;
            }
            self.w16(0x402, self.r16(0x402) | CHEST_OPEN_MASKS[original_index]);
            self.w8(0x12e, 0x29);
            self.w8(0x12f, 0x15);
            let source = self.object_data::<4>(usize::from(self.r16(0x46a)));
            (usize::from((location & 0x7fff) >> 1), source, 0, 0xff)
        } else {
            let chest_data = self.hooks.dungeon_room_chests().to_vec();
            let mut data = None;
            for record in chest_data.chunks_exact(3) {
                let room = u16::from_le_bytes([record[0], record[1]]);
                if room & 0x7fff == self.r16(0xa0) {
                    chest_index -= 1;
                    if chest_index < 0 {
                        data = Some((room, record[2]));
                        break;
                    }
                }
            }
            let Some((room, data)) = data else {
                return 0xff;
            };
            if room & 0x8000 != 0 {
                if !has_big_key {
                    self.w16(0x1cf0, 0x7a);
                    self.hooks.main_show_text_message(&mut self.ram);
                    return 0xff;
                }
                self.w16(0x402, self.r16(0x402) | CHEST_OPEN_MASKS[original_index]);
                self.open_big_chest(location, chest_position);
                return data;
            }
            self.w16(0x402, self.r16(0x402) | CHEST_OPEN_MASKS[original_index]);
            (
                usize::from(location >> 1),
                self.object_data::<4>(0x14a4),
                0x27,
                data,
            )
        };

        for (index, delta) in [0usize, 64, 1, 65].into_iter().enumerate() {
            self.w16(0x2000 + (position + delta) * 2, source[index]);
            self.w8(0x12000 + position + delta, attribute);
        }
        let upload = 0x1002 + usize::from(self.r16(0x1000));
        for (index, delta) in [0usize, 64, 1, 65].into_iter().enumerate() {
            self.w16(
                upload + index * 6,
                Self::dungeon_map_vram_addr((position + delta) as u16),
            );
            self.w16(upload + index * 6 + 2, 0x100);
            self.w16(upload + index * 6 + 4, source[index]);
        }
        self.w16(upload + 24, 0xffff);
        self.w16(0x1000, self.r16(0x1000).wrapping_add(24));
        self.w8(0x14, 1);
        self.dungeon_flag_room_data_quadrants();
        if self.r8(0x12f) == 0 {
            self.w8(0x12f, 14);
        }
        *chest_position = location & 0x7fff;
        data
    }

    pub fn room_bounds_add_a(bounds: &mut RoomBounds) {
        bounds.add_a();
    }

    pub fn room_bounds_add_b(bounds: &mut RoomBounds) {
        bounds.add_b();
    }

    pub fn room_bounds_sub_b(bounds: &mut RoomBounds) {
        bounds.sub_b();
    }

    pub fn room_bounds_sub_a(bounds: &mut RoomBounds) {
        bounds.sub_a();
    }

    #[must_use]
    pub fn dungeon_map_vram_addr_no_swap(mut position: u16) -> u16 {
        position = position.wrapping_mul(2);
        ((position & 0x40) << 4) | ((position & 0x303f) >> 1) | ((position & 0x0f80) >> 2)
    }

    #[must_use]
    pub fn dungeon_map_vram_addr(position: u16) -> u16 {
        Self::dungeon_map_vram_addr_no_swap(position).swap_bytes()
    }

    pub fn dungeon_store_2x2(&mut self, position: u16, tiles: [u16; 4], attribute: u8) {
        let upload = 0x1002 + usize::from(self.r16(0x1000));
        for (i, (delta, tile)) in [0u16, 64, 1, 65].into_iter().zip(tiles).enumerate() {
            let position = position.wrapping_add(delta);
            self.w16(upload + (i * 3) * 2, Self::dungeon_map_vram_addr(position));
            self.w16(upload + (i * 3 + 1) * 2, 0x100);
            self.w16(upload + (i * 3 + 2) * 2, tile);
            self.w16(DUNG_BG2 + usize::from(position) * 2, tile);
            self.w8(0x12000 + usize::from(position), attribute);
        }
        self.w16(upload + 24, 0xffff);
        self.w16(0x1000, self.r16(0x1000).wrapping_add(24));
        self.w8(0x14, 1);
    }

    pub fn door_up_entrance_door(&self, _destination: u16) {
        panic!("Door_Up_EntranceDoor is intentionally unimplemented in the original");
    }

    pub fn door_down_entrance_door(&self, _destination: u16) {
        panic!("Door_Down_EntranceDoor is intentionally unimplemented in the original");
    }

    pub fn door_left_entrance_door(&self, _destination: u16) {
        panic!("Door_Left_EntranceDoor is intentionally unimplemented in the original");
    }

    pub fn door_right_entrance_door(&self, _destination: u16) {
        panic!("Door_Right_EntranceDoor is intentionally unimplemented in the original");
    }

    pub fn door_draw_helper4(&mut self, door_type: u8, destination: u16) {
        let mut final_type =
            self.room_draw_flag_doors_and_get_final_type(1, door_type, destination);
        if final_type & 0x100 != 0 {
            return;
        }
        let new_type = if final_type == 0x1e || final_type == 0x36 {
            Some(0)
        } else if final_type == 0x38 {
            Some(24)
        } else {
            None
        };
        if let Some(new_type) = new_type {
            let slot = usize::from(self.r16(0x460) >> 1) - 1;
            self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | new_type);
            final_type = new_type;
        }
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA2[usize::from(final_type >> 1)],
        ));
        let destination = self.dsto_ptr(destination);
        for column in 0..4 {
            for row in 0..3 {
                self.w16(
                    destination + xy(column, row + 1) * 2,
                    source[column * 3 + row],
                );
            }
        }
    }

    pub fn room_draw_check_if_lower_layer_doors_y(&mut self, door_type: u8, destination: u16) {
        if door_type == 2 {
            self.room_draw_make_door_parts_high_priority_y(usize::from(destination) + xy(0, 4));
            self.door_draw_helper4(door_type, destination);
        } else if door_type == 8 {
            self.door_draw_helper4(door_type, destination);
            self.door_prioritize_cur_door();
        } else {
            self.door_draw_helper4(door_type, destination);
        }
    }

    pub fn room_draw_one_sided_shutters_east(&mut self, door_type: u8, destination: u16) {
        let mut final_type =
            self.room_draw_flag_doors_and_get_final_type(3, door_type, destination);
        if final_type & 0x100 != 0 {
            return;
        }
        let new_type = if final_type == 0x36 {
            Some(0)
        } else if final_type == 0x38 {
            Some(24)
        } else {
            None
        };
        if let Some(new_type) = new_type {
            let slot = usize::from(self.r16(0x460) >> 1) - 1;
            self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | new_type);
            final_type = new_type;
        }
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA4[usize::from(final_type >> 1)],
        ));
        let destination = self.dsto_ptr(destination) + 2;
        for column in 0..3 {
            for row in 0..4 {
                self.w16(destination + xy(column, row) * 2, source[column * 4 + row]);
            }
        }
    }

    pub fn room_draw_normal_ranged_doors_east(&mut self, door_type: u8, destination: u16) {
        if door_type == 2 {
            self.room_draw_make_door_parts_high_priority_x(usize::from(destination) + xy(4, 0));
        }
        self.room_draw_one_sided_shutters_east(door_type, destination);
        if door_type == 8 {
            self.door_prioritize_cur_door();
        }
    }

    pub fn room_draw_one_sided_lower_shutters_east(&mut self, door_type: u8, destination: u16) {
        let mut final_type =
            self.room_draw_flag_doors_and_get_final_type(3, door_type, destination);
        let new_type = if final_type == 72 {
            Some(64)
        } else if final_type == 74 {
            Some(68)
        } else {
            None
        };
        if let Some(new_type) = new_type {
            let slot = usize::from(self.r16(0x460) >> 1) - 1;
            self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | new_type);
            final_type = new_type;
        }
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA4[usize::from(final_type >> 1)],
        ));
        for column in 0..2 {
            for row in 0..4 {
                self.set_bg1(
                    usize::from(destination) + xy(column + 1, row),
                    source[column * 4 + row],
                );
            }
        }
        for row in 0..4 {
            self.set_bg2(usize::from(destination) + xy(3, row), source[8 + row]);
        }
        self.room_draw_make_door_high_priority_east(destination.wrapping_add(4));
        self.door_prioritize_cur_door();
    }

    pub fn room_draw_door_east(&mut self, door_type: u8, position: usize) {
        let destination = DOOR_POSITION_TO_TILEMAP_OFFS_RIGHT[position] / 2;
        match door_type {
            22 => self.room_draw_mark_layer_toggle_door(destination.wrapping_add(68)),
            6 => self.door_right_entrance_door(destination),
            20 => self.room_draw_mark_dungeon_toggle_door(destination.wrapping_add(68)),
            0..=63 => self.room_draw_normal_ranged_doors_east(door_type, destination),
            _ => self.room_draw_one_sided_lower_shutters_east(door_type, destination),
        }
    }

    pub fn room_draw_normal_ranged_doors_west(
        &mut self,
        door_type: u8,
        destination: u16,
        position: usize,
    ) {
        if position >= 6 {
            let saved_index = self.r16(0x460);
            self.w16(0x460, saved_index | 0x10);
            self.room_draw_normal_ranged_doors_east(
                door_type,
                DOOR_POSITION_TO_TILEMAP_OFFS_RIGHT[position - 6] / 2,
            );
            self.w16(0x460, saved_index);
        }
        let mut final_type =
            self.room_draw_flag_doors_and_get_final_type(2, door_type, destination);
        if final_type & 0x100 != 0 {
            return;
        }
        let new_type = if final_type == 0x36 {
            Some(24)
        } else if final_type == 0x38 {
            Some(0)
        } else {
            None
        };
        if let Some(new_type) = new_type {
            let slot = usize::from(self.r16(0x460) >> 1) - 1;
            self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | new_type);
            final_type = new_type;
        }
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA3[usize::from(final_type >> 1)],
        ));
        let destination = self.dsto_ptr(destination);
        for column in 0..3 {
            for row in 0..4 {
                self.w16(destination + xy(column, row) * 2, source[column * 4 + row]);
            }
        }
    }

    pub fn room_draw_high_range_door_west(
        &mut self,
        door_type: u8,
        destination: u16,
        position: usize,
    ) {
        if position >= 6 {
            let saved_index = self.r16(0x460);
            self.w16(0x460, saved_index | 0x10);
            self.room_draw_one_sided_lower_shutters_east(
                door_type,
                DOOR_POSITION_TO_TILEMAP_OFFS_RIGHT[position - 6] / 2,
            );
            self.w16(0x460, saved_index);
        }
        let mut final_type =
            self.room_draw_flag_doors_and_get_final_type(2, door_type, destination);
        let new_type = if final_type == 72 {
            Some(68)
        } else if final_type == 74 {
            Some(64)
        } else {
            None
        };
        if let Some(new_type) = new_type {
            let slot = usize::from(self.r16(0x460) >> 1) - 1;
            self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | new_type);
            final_type = new_type;
        }
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA3[usize::from(final_type >> 1)],
        ));
        for (row, &tile) in source[..4].iter().enumerate() {
            self.set_bg2(usize::from(destination) + xy(0, row), tile);
        }
        for column in 1..3 {
            for row in 0..4 {
                self.set_bg1(
                    usize::from(destination) + xy(column, row),
                    source[column * 4 + row],
                );
            }
        }
        self.room_draw_make_door_high_priority_west(destination);
        self.door_prioritize_cur_door();
    }

    pub fn room_draw_door_west(&mut self, door_type: u8, position: usize) {
        let destination = DOOR_POSITION_TO_TILEMAP_OFFS_LEFT[position] / 2;
        match door_type {
            22 => self.room_draw_mark_layer_toggle_door(destination.wrapping_add(62)),
            6 => self.door_left_entrance_door(destination),
            20 => self.room_draw_mark_dungeon_toggle_door(destination.wrapping_add(62)),
            2 => {
                self.room_draw_make_door_parts_high_priority_x(usize::from(destination & !0x1f));
                self.room_draw_normal_ranged_doors_west(door_type, destination, position);
            }
            8 => {
                self.room_draw_normal_ranged_doors_west(door_type, destination, position);
                self.door_prioritize_cur_door();
            }
            0..=63 => {
                self.room_draw_normal_ranged_doors_west(door_type, destination, position);
            }
            _ => self.room_draw_high_range_door_west(door_type, destination, position),
        }
    }

    pub fn room_draw_one_sided_shutters_north(&mut self, door_type: u8, destination: u16) {
        let mut final_type =
            self.room_draw_flag_doors_and_get_final_type(0, door_type, destination);
        if final_type & 0x100 != 0 {
            return;
        }
        let new_type = if final_type == 0x36 {
            Some(24)
        } else if final_type == 0x38 {
            Some(0)
        } else {
            None
        };
        if let Some(new_type) = new_type {
            let slot = usize::from(self.r16(0x460) >> 1) - 1;
            self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | new_type);
            final_type = new_type;
        }
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA[usize::from(final_type >> 1)],
        ));
        let destination = self.dsto_ptr(destination);
        for column in 0..4 {
            for row in 0..3 {
                self.w16(destination + xy(column, row) * 2, source[column * 3 + row]);
            }
        }
    }

    pub fn room_draw_normal_ranged_doors_north(
        &mut self,
        door_type: u8,
        destination: u16,
        position: usize,
    ) {
        if position >= 6 {
            let saved_index = self.r16(0x460);
            self.w16(0x460, saved_index | 0x10);
            self.room_draw_check_if_lower_layer_doors_y(
                door_type,
                DOOR_POSITION_TO_TILEMAP_OFFS_DOWN[position - 6] / 2,
            );
            self.w16(0x460, saved_index);
        }
        self.room_draw_one_sided_shutters_north(door_type, destination);
    }

    pub fn room_draw_one_sided_lower_shutters_south(&mut self, door_type: u8, destination: u16) {
        let mut final_type =
            self.room_draw_flag_doors_and_get_final_type(1, door_type, destination);
        let new_type = if final_type == 72 {
            Some(68)
        } else if final_type == 74 {
            Some(64)
        } else {
            None
        };
        if let Some(new_type) = new_type {
            let slot = usize::from(self.r16(0x460) >> 1) - 1;
            self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | new_type);
            final_type = new_type;
        }
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA2[usize::from(final_type >> 1)],
        ));
        for column in 0..4 {
            self.set_bg1(usize::from(destination) + xy(column, 1), source[column * 3]);
            self.set_bg1(
                usize::from(destination) + xy(column, 2),
                source[column * 3 + 1],
            );
            self.set_bg2(
                usize::from(destination) + xy(column, 3),
                source[column * 3 + 2],
            );
        }
        self.room_draw_make_door_high_priority_south(destination.wrapping_add(256));
        self.door_prioritize_cur_door();
    }

    pub fn room_draw_high_range_door_north(
        &mut self,
        door_type: u8,
        destination: u16,
        position: usize,
    ) {
        if position >= 6 && door_type != 70 {
            let saved_index = self.r16(0x460);
            self.w16(0x460, saved_index | 0x10);
            self.room_draw_one_sided_lower_shutters_south(
                door_type,
                DOOR_POSITION_TO_TILEMAP_OFFS_DOWN[position - 6] / 2,
            );
            self.w16(0x460, saved_index);
        }
        let mut final_type =
            self.room_draw_flag_doors_and_get_final_type(0, door_type, destination);
        let new_type = if final_type == 72 {
            Some(64)
        } else if final_type == 74 {
            Some(68)
        } else {
            None
        };
        if let Some(new_type) = new_type {
            let slot = usize::from(self.r16(0x460) >> 1) - 1;
            self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | new_type);
            final_type = new_type;
        }
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA[usize::from(final_type >> 1)],
        ));
        for column in 0..4 {
            self.set_bg2(usize::from(destination) + xy(column, 0), source[column * 3]);
            self.set_bg1(
                usize::from(destination) + xy(column, 1),
                source[column * 3 + 1],
            );
            self.set_bg1(
                usize::from(destination) + xy(column, 2),
                source[column * 3 + 2],
            );
        }
        if door_type != 70 {
            self.room_draw_make_door_high_priority_north(destination);
        }
        self.door_prioritize_cur_door();
    }

    pub fn room_draw_north_curtain_door(&mut self, destination: u16) {
        let final_type = self.room_draw_flag_doors_and_get_final_type(0, 50, destination);
        let source_offset = if final_type & 0x100 != 0 {
            0x78a
        } else {
            DOOR_TYPE_SRC_DATA[usize::from(final_type >> 1)]
        };
        let source = self.object_data::<16>(usize::from(source_offset));
        let destination = self.dsto_ptr(destination);
        self.room_draw_4x4(&source, destination);
    }

    pub fn door_up_stair_mask_locked(&mut self, door_type: u8, destination: u16) {
        let slot = usize::from(self.r16(0x460) >> 1);
        self.w16(0x19c0 + slot * 2, 0);
        self.w16(0x19a0 + slot * 2, destination.wrapping_mul(2));
        self.w16(
            0x1980 + slot * 2,
            ((slot as u16) << 8) | u16::from(door_type),
        );
        if self.r16(0x68c) & UPPER_BITMASKS[slot & 7] != 0 {
            self.w16(0x460, self.r16(0x460).wrapping_add(2));
            return;
        }
        if door_type < 36 {
            self.room_draw_one_sided_shutters_north(door_type, destination);
            return;
        }
        let final_type = self.room_draw_flag_doors_and_get_final_type(0, door_type, destination);
        let source = self.object_data::<12>(usize::from(
            DOOR_TYPE_SRC_DATA[usize::from(final_type >> 1)],
        ));
        for column in 0..4 {
            for row in 0..3 {
                self.set_bg1(
                    usize::from(destination) + xy(column, row),
                    source[column * 3 + row],
                );
            }
        }
        self.door_prioritize_cur_door();
    }

    pub fn room_draw_exploding_wall_segment(&mut self, source: &[u16], destination: u16) {
        assert!(source.len() >= 25);
        let first = self.dsto_ptr(destination);
        self.room_draw_exploding_wall_column(&source[..12], first);
        let middle = destination.wrapping_add(2);
        for column in 0..18 {
            for row in 0..6 {
                self.set_bg2(usize::from(middle) + xy(column, row), source[12]);
            }
        }
        let last = self.dsto_ptr(middle.wrapping_add(18));
        self.room_draw_exploding_wall_column(&source[13..25], last);
    }

    pub fn room_draw_door_exploding_wall(&mut self, position: usize) {
        let destination = DOOR_BLAST_WALL_UP_DSTS[position] / 2;
        let slot = usize::from(self.r16(0x460) >> 1);
        self.w16(
            0x19a0 + slot * 2,
            destination.wrapping_add(10).wrapping_mul(2),
        );
        self.w16(0x1980 + slot * 2, ((slot as u16) << 8) | 48);
        if self.r16(0x68c) & UPPER_BITMASKS[slot & 7] == 0 {
            self.w16(0x19c0 + slot * 2, 0);
            self.w16(0x460, self.r16(0x460).wrapping_add(2));
            return;
        }
        let tag_slot =
            usize::from(self.r8(0xae) != 0x20 && self.r8(0xae) != 0x25 && self.r8(0xae) != 0x28);
        self.w8(0xae + tag_slot, 0);
        self.w8(0xa7, 2);
        self.w8(0x453, 1);
        let upper = self.object_data::<25>(usize::from(DOOR_TYPE_SRC_DATA2[42]));
        self.room_draw_exploding_wall_segment(&upper, destination);
        self.w16(0x460, self.r16(0x460).wrapping_add(2));
        self.w16(0xfc, self.r16(0xfc) | 0x200);
        let lower = self.object_data::<25>(usize::from(DOOR_TYPE_SRC_DATA[42]));
        self.room_draw_exploding_wall_segment(&lower, destination.wrapping_add(384));
    }

    pub fn room_draw_door_north(&mut self, door_type: u8, position: usize) {
        let destination = DOOR_POSITION_TO_TILEMAP_OFFS_UP[position] / 2;
        match door_type {
            48 => self.room_draw_door_exploding_wall(position),
            22 => self.room_draw_mark_layer_toggle_door(destination.wrapping_sub(0x7f)),
            50 => self.room_draw_north_curtain_door(destination),
            6 => self.door_up_entrance_door(destination),
            20 => self.room_draw_mark_dungeon_toggle_door(destination.wrapping_sub(0x7f)),
            2 => {
                self.room_draw_make_door_parts_high_priority_y(usize::from(destination & 0x783f));
                self.room_draw_normal_ranged_doors_north(door_type, destination, position);
            }
            18 => {
                let count = self.r16(0x19e0);
                self.w16(0x19e2 + usize::from(count), destination.wrapping_mul(2));
                self.w16(0x19e0, count.wrapping_add(2));
            }
            8 => {
                self.room_draw_normal_ranged_doors_north(door_type, destination, position);
                self.door_prioritize_cur_door();
            }
            32..=38 => self.door_up_stair_mask_locked(door_type, destination),
            64.. => self.room_draw_high_range_door_north(door_type, destination, position),
            _ => self.room_draw_normal_ranged_doors_north(door_type, destination, position),
        }
    }

    pub fn room_draw_door_south(&mut self, door_type: u8, position: usize) {
        let mut destination = DOOR_POSITION_TO_TILEMAP_OFFS_DOWN[position] / 2;
        match door_type {
            22 => self.room_draw_mark_layer_toggle_door(destination.wrapping_add(257)),
            6 => self.door_down_entrance_door(destination),
            20 => self.room_draw_mark_dungeon_toggle_door(destination.wrapping_add(257)),
            18 => {
                let count = self.r16(0x19e0);
                self.w16(0x19e2 + usize::from(count), destination.wrapping_mul(2));
                self.w16(0x19e0, count.wrapping_add(2));
            }
            64.. => self.room_draw_one_sided_lower_shutters_south(door_type, destination),
            10 => {
                self.room_draw_flag_doors_and_get_final_type(1, door_type, destination);
                let source = self.object_data::<80>(0x2656);
                self.room_draw_some_big_decors(
                    10,
                    &source,
                    usize::from(destination.wrapping_sub(259)),
                );
            }
            12 => {
                destination |= 0x1000;
                self.room_draw_flag_doors_and_get_final_type(1, door_type, destination);
                let source = self.object_data::<80>(0x2656);
                self.room_draw_some_big_decors(
                    10,
                    &source,
                    usize::from(destination.wrapping_sub(259)),
                );
                destination = destination.wrapping_sub(0x1000).wrapping_add(448);
                for column in 0..10 {
                    let tile = self.r16(DUNG_BG1 + (usize::from(destination) + column) * 2);
                    self.set_bg2(usize::from(destination) + column, tile | 0x2000);
                }
            }
            14 | 16 => {
                if door_type == 16 {
                    self.room_draw_make_door_parts_high_priority_y(usize::from(destination) + 256);
                }
                self.room_draw_flag_doors_and_get_final_type(1, door_type, destination);
                let source = self.object_data::<16>(0x26f6);
                let pointer = self.dsto_ptr(destination);
                self.room_draw_4x4(&source, pointer);
            }
            4 => {
                let mut original = destination;
                destination |= 0x1000;
                self.room_draw_make_door_parts_high_priority_y(usize::from(destination) + 256);
                self.room_draw_flag_doors_and_get_final_type(1, door_type, destination);
                let source = self.object_data::<16>(0x26f6);
                let pointer = self.dsto_ptr(destination);
                self.room_draw_4x4(&source, pointer);
                for _ in 0..4 {
                    let position = usize::from(original) + xy(0, 3);
                    self.set_bg2(position, self.r16(DUNG_BG1 + position * 2) | 0x2000);
                    original = original.wrapping_add(1);
                }
            }
            _ => self.room_draw_check_if_lower_layer_doors_y(door_type, destination),
        }
    }

    pub fn room_data_draw_object_door(&mut self, descriptor: u16) {
        let door_type = (descriptor >> 8) as u8;
        let position = usize::from((descriptor >> 4) & 0xf);
        match descriptor & 3 {
            0 => self.room_draw_door_north(door_type, position),
            1 => self.room_draw_door_south(door_type, position),
            2 => self.room_draw_door_west(door_type, position),
            3 => self.room_draw_door_east(door_type, position),
            _ => unreachable!(),
        }
    }

    pub fn object_draw_door_left_3x4(&mut self, source_byte_offset: usize, door: usize) {
        let source = self.object_data::<12>(source_byte_offset);
        let destination = usize::from(self.r16(0x19a0 + door * 2)) / 2;
        for column in 0..3 {
            for row in 0..4 {
                self.set_bg2(destination + xy(column, row), source[column * 4 + row]);
            }
        }
    }

    pub fn object_draw_door_right_3x4(&mut self, source_byte_offset: usize, door: usize) {
        let source = self.object_data::<12>(source_byte_offset);
        let destination = usize::from(self.r16(0x19a0 + door * 2)) / 2;
        for column in 0..3 {
            for row in 0..4 {
                self.set_bg2(destination + xy(column + 1, row), source[column * 4 + row]);
            }
        }
    }

    pub fn object_draw_door_up_4x3(&mut self, source_byte_offset: usize, door: usize) {
        let source = self.object_data::<12>(source_byte_offset);
        let destination = usize::from(self.r16(0x19a0 + door * 2)) / 2;
        for column in 0..4 {
            for row in 0..3 {
                self.set_bg2(destination + xy(column, row), source[column * 3 + row]);
            }
        }
    }

    pub fn object_draw_door_down_4x3(&mut self, source_byte_offset: usize, door: usize) {
        let source = self.object_data::<12>(source_byte_offset);
        let destination = usize::from(self.r16(0x19a0 + door * 2)) / 2;
        for column in 0..4 {
            for row in 0..3 {
                self.set_bg2(destination + xy(column, row + 1), source[column * 3 + row]);
            }
        }
    }

    pub fn room_draw_exploding_wall_column(&mut self, src: &[u16], destination: usize) {
        assert!(src.len() >= 12);
        for row in 0..6 {
            self.w16(destination + xy(0, row) * 2, src[row]);
            self.w16(destination + xy(1, row) * 2, src[row + 6]);
        }
    }

    pub fn door_prioritize_cur_door(&mut self) {
        let index = usize::from(self.r16(0x460) >> 1) - 1;
        self.w16(0x19a0 + index * 2, self.r16(0x19a0 + index * 2) | 0x2000);
    }

    pub fn get_room_door_info(&self, room: usize) -> &[u16] {
        self.hooks.dungeon_room_door_info(room)
    }

    pub fn get_room_header_ptr(&self, room: usize) -> &[u8] {
        self.hooks.dungeon_room_header(room)
    }

    pub fn get_default_room_layout(&self, layout: usize) -> &[u8] {
        self.hooks.default_room_layout(layout)
    }

    pub fn get_dungeon_room_layout(&self, layout: usize) -> &[u8] {
        self.hooks.dungeon_room_layout(layout)
    }

    pub fn dungeon_load_adjacent_room_doors(&mut self, room: usize) {
        let doors = self.hooks.dungeon_room_door_info(room);
        let end = doors
            .iter()
            .position(|&door| door == 0xffff)
            .expect("dungeon room door list terminator");
        let mut copied = [0u16; 17];
        assert!(end < copied.len());
        copied[..=end].copy_from_slice(&doors[..=end]);
        let mut flags = (self.r16(0xf000 + room * 2) & 0xf000) | 0x0f00;
        for (index, &door) in copied[..=end].iter().enumerate() {
            self.w16(0x1110 + index * 2, door);
            if door == 0xffff {
                break;
            }
            if door & 0xff00 == 0x4000 || door & 0xff00 < 0x200 {
                flags |= UPPER_BITMASKS[index];
            }
        }
        self.w16(0x1100, flags);
    }

    pub fn dungeon_check_adjacent_rooms_for_open_doors(&mut self, start: usize, room: usize) {
        self.dungeon_load_adjacent_room_doors(room);
        for (adjacent_index, adjacent_mask) in UPPER_BITMASKS.iter().copied().enumerate().take(8) {
            let raw = self.r16(0x1110 + adjacent_index * 2);
            if raw == 0xffff {
                break;
            }
            let value = raw & 0xff;
            let Some(found) =
                (start..start + 6).find(|&index| value == ADJACENT_DOOR_LOOKUP[index])
            else {
                continue;
            };
            let reverse = ADJACENT_DOOR_LOOKUP2[found] as u8;
            for (door, door_mask) in UPPER_BITMASKS.iter().copied().enumerate().take(8) {
                let current = self.r16(0x19a0 + door * 2);
                if current as u8 != reverse {
                    continue;
                }
                let door_type = (current >> 8) as u8;
                if door_type == 0x30 {
                    break;
                }
                if door_type == 0x44 || door_type == 0x18 {
                    if room != usize::from(self.r16(0xa2)) {
                        break;
                    }
                    self.w16(0x468, 0);
                } else if self.r16(0x1100) & adjacent_mask == 0 {
                    break;
                }
                self.w16(0x68c, self.r16(0x68c) | door_mask);
                break;
            }
        }
    }

    pub fn dungeon_load_header(&mut self) {
        self.w8(0x642, 0);
        self.w8(0x646, 0);
        self.w8(0x641, 0);
        let submodule = self.r8(0x11);
        let (horizontal, vertical) = if submodule == 0 {
            (self.r16(0xe2) & !0x1ff, self.r16(0xe8) & !0x1ff)
        } else if submodule == 21 || (6..18).contains(&submodule) {
            (
                self.r16(0xe2).wrapping_add(0x20) & !0x1ff,
                self.r16(0xe8).wrapping_add(0x20) & !0x1ff,
            )
        } else {
            let direction = self.r8(0x67) & 0xf;
            if direction >> 1 < 2 {
                (
                    self.r16(0xe2)
                        .wrapping_add(HEADER_ADJUSTMENT[usize::from(direction >> 1)] as u16)
                        & !0x1ff,
                    self.r16(0xe8).wrapping_add(0x20) & !0x1ff,
                )
            } else {
                (
                    self.r16(0xe2).wrapping_add(0x20) & !0x1ff,
                    self.r16(0xe8)
                        .wrapping_add(HEADER_ADJUSTMENT[usize::from(direction >> 3)] as u16)
                        & !0x1ff,
                )
            }
        };
        self.w16(0x62c, horizontal);
        self.w16(0x62e, vertical);

        let room = usize::from(self.r16(0xa0));
        let source = self.hooks.dungeon_room_header(room);
        assert!(source.len() >= 14);
        let header = core::array::from_fn::<_, 14, _>(|index| source[index]);
        self.w8(0xc208, self.r8(0x414));
        self.w8(0x414, header[0] >> 5);
        self.w8(0x46c, (header[0] >> 2) & 7);
        self.w8(0xc006, self.r8(0xc005));
        self.w8(0xc005, header[0] & 1);
        let palette = DUNG_PAL_INFOS[usize::from(header[1])];
        self.w8(0xab6, palette.pal0);
        self.w8(0xaac, palette.pal1);
        self.w8(0xaad, palette.pal2);
        self.w8(0xaae, palette.pal3);
        self.w8(0xaa2, header[2]);
        self.w8(0xaa3, header[3].wrapping_add(0x40));
        self.w8(0xad, header[4]);
        self.w8(0xae, header[5]);
        self.w8(0xaf, header[6]);
        self.w8(0x63c, header[7] & 3);
        self.w8(0x63d, (header[7] >> 2) & 3);
        self.w8(0x63e, (header[7] >> 4) & 3);
        self.w8(0x63f, (header[7] >> 6) & 3);
        self.w8(0x640, header[8] & 3);
        for index in 0..5 {
            self.w8(0xc000 + index, header[9 + index]);
        }
        self.w16(0x468, 1);
        self.w8(0x4ba, 0);
        self.w16(0x110, (room as u16).wrapping_mul(3));

        let saved = self.r16(0xf000 + room * 2);
        self.w16(0x400, saved & 0xf000);
        self.w16(0x68c, (saved & 0xf000) | 0x0f00);
        self.w16(0x402, (saved & 0x0ff0) << 4);
        self.w16(0x408, saved & 0xf);

        let source = self.hooks.dungeon_room_door_info(room);
        let end = source
            .iter()
            .position(|&door| door == 0xffff)
            .expect("dungeon door terminator");
        let doors: Vec<u16> = source[..=end].to_vec();
        for (index, &door) in doors[..end].iter().enumerate() {
            self.w16(0x19a0 + index * 2, door);
        }
        self.w16(0x19a0 + end * 2, 0);

        if room & 0xf != 0 {
            self.dungeon_check_adjacent_rooms_for_open_doors(18, room - 1);
        }
        if (room + 1) & 0xf != 0 {
            self.dungeon_check_adjacent_rooms_for_open_doors(12, room + 1);
        }
        if room >= 16 {
            self.dungeon_check_adjacent_rooms_for_open_doors(6, room - 16);
        }
        if room + 16 < 0x140 {
            self.dungeon_check_adjacent_rooms_for_open_doors(0, room + 16);
        }
    }

    pub fn layer_effect_nothing(&mut self) {}

    pub fn dungeon_handle_layer_effect(&mut self) {
        match self.r8(0xad) {
            0 | 1 => self.layer_effect_nothing(),
            2 => self.layer_effect_scroll(),
            3 => self.layer_effect_water_rapids(),
            4 => self.layer_effect_trinexx(),
            5 => self.layer_effect_agahnim2(),
            6 => self.layer_effect_invisible_floor(),
            7 => self.layer_effect_ganon(),
            _ => panic!("null dungeon layer-effect handler"),
        }
    }

    pub fn layer_effect_scroll(&mut self) {
        if self.r16(0x402) & 0x8000 != 0 {
            self.w8(0xad, 0);
            return;
        }
        self.w16(0x312, 0);
        self.w16(0x310, 0);
        let flags = self.r16(0x41a);
        if flags & 1 != 0 {
            return;
        }
        let accumulated = u16::from(self.r8(0x41d)) + 0x80;
        self.w8(0x41d, accumulated as u8);
        let movement = if flags & 2 != 0 {
            (0u16).wrapping_sub(accumulated >> 8)
        } else {
            accumulated >> 8
        };
        if flags < 4 {
            self.w16(0x312, movement);
            let offset = self.r16(0x422).wrapping_sub(movement);
            self.w16(0x422, offset);
            self.w16(0xe0, self.r16(0xe2).wrapping_add(offset));
        } else {
            self.w16(0x310, movement);
            let offset = self.r16(0x424).wrapping_sub(movement);
            self.w16(0x424, offset);
            self.w16(0xe6, self.r16(0xe8).wrapping_add(offset));
        }
    }

    pub fn layer_effect_trinexx(&mut self) {
        self.w16(0x422, self.r16(0x422).wrapping_add(self.r16(0x312)));
        self.w16(0x424, self.r16(0x424).wrapping_add(self.r16(0x310)));
        self.w16(0x312, 0);
        self.w16(0x310, 0);
    }

    pub fn layer_effect_agahnim2(&mut self) {
        match self.r8(0x1a) & 0x7f {
            3 | 36 => {
                self.w16(0xc500 + 0x6d * 2, 0x1d59);
                self.w16(0xc500 + 0x6e * 2, 0x25ff);
                self.w16(0xc500 + 0x6f * 2, 0x1a);
                self.w16(0xc500 + 0x77 * 2, 0x1a);
                self.w8(0x15, self.r8(0x15).wrapping_add(1));
            }
            5 | 38 => {
                for index in [0x6d, 0x6e, 0x6f] {
                    self.w16(0xc500 + index * 2, self.r16(0xc300 + index * 2));
                }
                self.w16(0xc500 + 0x77 * 2, self.r16(0xc300 + 0x6f * 2));
                self.w8(0x15, self.r8(0x15).wrapping_add(1));
            }
            _ => {}
        }
        self.w8(0x1d, 2);
    }

    pub fn layer_effect_invisible_floor(&mut self) {
        let count = (0..16)
            .filter(|&index| self.r16(0x540 + index * 2) & 0x8000 != 0)
            .count();
        let (color1, color2) = if count == 0 { (0, 0) } else { (0x2940, 0x4e60) };
        if self.r16(0xc300 + 0x7b * 2) != color1 {
            self.w16(0xc300 + 0x7b * 2, color1);
            self.w16(0xc500 + 0x7b * 2, color1);
            self.w16(0xc300 + 0x7c * 2, color2);
            self.w16(0xc500 + 0x7c * 2, color2);
            self.w8(0x15, self.r8(0x15).wrapping_add(1));
        }
        self.w8(0x1d, 2);
    }

    pub fn layer_effect_ganon(&mut self) {
        let count = (0..16)
            .filter(|&i| self.r16(0x540 + i * 2) & 0x8000 != 0)
            .count() as u8;
        self.w8(0x4c5, count);
        let (ts, color_math) = match count {
            0 => (0, 0xb3),
            1 => (2, 0x70),
            _ => (0, 0x70),
        };
        self.w8(0x1d, ts);
        self.w8(0x9a, color_math);
    }

    pub fn layer_effect_water_rapids(&mut self) {
        let accumulated = u16::from(self.r8(0x41d)) + 0x80;
        self.w8(0x41d, accumulated as u8);
        self.w16(0x312, (0u16).wrapping_sub(accumulated >> 8));
    }

    pub fn write_attr1(&mut self, index: usize, attribute: u16) {
        self.w8(0x13000 + index, attribute as u8);
        self.w8(0x13001 + index, (attribute >> 8) as u8);
    }

    pub fn write_attr2(&mut self, index: usize, attribute: u16) {
        self.w8(0x12000 + index, attribute as u8);
        self.w8(0x12001 + index, (attribute >> 8) as u8);
    }

    pub fn room_draw_mark_dungeon_toggle_door(&mut self, destination: u16) {
        let count = self.r16(0x450);
        self.w16(0x6d0 + usize::from(count), destination);
        self.w16(0x450, count.wrapping_add(2));
    }

    pub fn room_draw_mark_layer_toggle_door(&mut self, destination: u16) {
        let count = self.r16(0x44e);
        self.w16(0x6c0 + usize::from(count), destination);
        self.w16(0x44e, count.wrapping_add(2));
    }

    pub fn room_draw_get_object_size_1to16(&mut self) {
        let size = (self.r16(0xb2) << 2 | self.r16(0xb4)).wrapping_add(1);
        self.w16(0xb2, size);
        self.w16(0xb4, 0);
    }

    pub fn object_size_a_to_a_plus15(&mut self, addend: u8) {
        let size = (self.r16(0xb2) << 2 | self.r16(0xb4)).wrapping_add(u16::from(addend));
        self.w16(0xb2, size);
        self.w16(0xb4, 0);
    }

    pub fn room_draw_get_object_size_1to15or26(&mut self) {
        let size = self.r16(0xb2) << 2 | self.r16(0xb4);
        self.w16(0xb2, if size == 0 { 26 } else { size });
    }

    pub fn room_draw_get_object_size_1to15or32(&mut self) {
        let size = self.r16(0xb2) << 2 | self.r16(0xb4);
        self.w16(0xb2, if size == 0 { 32 } else { size });
    }

    pub fn room_draw_flag_doors_and_get_final_type(
        &mut self,
        direction: u8,
        door_type: u8,
        destination: u16,
    ) -> u16 {
        let slot = usize::from(self.r16(0x460) >> 1);
        self.w16(0x19c0 + slot * 2, u16::from(direction));
        self.w16(0x19a0 + slot * 2, destination.wrapping_mul(2));
        self.w16(
            0x1980 + slot * 2,
            ((slot as u16) << 8) | u16::from(door_type),
        );

        let mut remapped = door_type;
        if slot & 7 < 4 && self.r16(0x68c) & UPPER_BITMASKS[slot & 7] != 0 {
            let trapdoor = door_type == 24 || door_type == 68;
            if !(trapdoor && self.r16(0x468) != 0) {
                remapped = DOOR_TYPE_REMAP[usize::from(door_type >> 1)];
                if !trapdoor && door_type >= 26 && door_type != 64 && door_type != 70 {
                    self.w16(0x400, self.r16(0x400) | UPPER_BITMASKS[slot]);
                }
            }
        }
        self.w16(0x460, (slot as u16).wrapping_mul(2).wrapping_add(2));

        if remapped == 50 || remapped == 8 {
            return 0x100 | u16::from(remapped);
        }
        if door_type != 26 {
            return u16::from(remapped);
        }
        self.w16(
            0x436,
            (((slot as u16) << 8) | u16::from(direction)).wrapping_mul(2),
        );
        self.w16(0x68c, self.r16(0x68c) | UPPER_BITMASKS[slot]);
        0
    }

    pub fn dungeon_flip_crystal_peg_attribute(&mut self) {
        for index in (0..=0xfff).rev() {
            for base in [0x12000, 0x13000] {
                let value = self.r8(base + index);
                if value & !1 == 0x66 {
                    self.w8(base + index, value ^ 1);
                }
            }
        }
    }

    pub fn door_load_blast_wall_attr(&mut self, door: usize) {
        assert!(door < 16);
        let mut position = usize::from(self.r16(0x19a0 + door * 2) >> 1);
        if self.r16(0x19c0 + door * 2) & 2 == 0 {
            for _ in 0..12 {
                self.write_attr2(position, 0x102);
                for column in (2..20).step_by(2) {
                    self.write_attr2(position + xy(column, 0), 0);
                }
                self.write_attr2(position + xy(20, 0), 0x201);
                position += xy(0, 1);
            }
        } else {
            for _ in 0..5 {
                self.write_attr2(position, 0x101);
                self.write_attr2(position + xy(0, 21), 0x101);
                self.write_attr2(position + xy(0, 1), 0x202);
                self.write_attr2(position + xy(0, 20), 0x202);
                for row in 2..20 {
                    self.write_attr2(position + xy(0, row), 0);
                }
                position += xy(2, 0);
            }
        }
    }

    pub fn dungeon_load_door_attribute(&mut self) {
        for door in 0..16 {
            if self.r16(0x19a0 + door * 2) != 0 {
                self.dungeon_load_single_door_attribute(door);
            }
        }
        self.dungeon_load_single_door_tile_attribute();
        self.change_door_to_switch();
        self.w8(0x200, self.r8(0x200).wrapping_add(1));
    }

    pub fn dungeon_load_single_door_attribute(&mut self, door: usize) {
        assert!(door < 16);
        let door_type = (self.r16(0x1980 + door * 2) as u8) & 0xfe;
        let alpha = matches!(door_type, 0 | 6 | 18 | 10 | 14);
        let mut beta = matches!(door_type, 2 | 4 | 8 | 12 | 16 | 64 | 70);
        if door_type == 48 {
            return;
        }
        if !alpha && !beta {
            if door_type >= 64 {
                if self.r16(0x68c) & UPPER_BITMASKS[door] == 0 {
                    let position = usize::from(self.r16(0x19a0 + door * 2) >> 1);
                    let attribute = (0xf0 + door as u16) * 0x101;
                    self.write_attr2(position + xy(1, 1), attribute);
                    self.write_attr2(position + xy(1, 2), attribute);
                    return;
                }
                beta = true;
            } else {
                let mask_index = if door_type == 24 || door_type == 68 {
                    door
                } else {
                    door & 7
                };
                if self.r16(0x68c) & UPPER_BITMASKS[mask_index] == 0 {
                    let position = usize::from(self.r16(0x19a0 + door * 2) >> 1);
                    let attribute = (0xf0 + door as u16) * 0x101;
                    self.write_attr2(position + xy(1, 1), attribute);
                    self.write_attr2(position + xy(1, 2), attribute);
                    return;
                }
            }
        }
        if alpha || !beta {
            if (32..=38).contains(&door_type) {
                return;
            }
            self.write_door_attribute_shape(door, door_type, false);
        } else {
            self.write_door_attribute_shape(door, door_type, true);
        }
    }

    fn write_door_attribute_shape(&mut self, door: usize, door_type: u8, large: bool) {
        let mut attribute = TILE_ATTRS_BY_DOOR[usize::from(door_type >> 1)];
        let address = self.r16(0x19a0 + door * 2);
        let direction = self.r16(0x19c0 + door * 2) & 3;
        if direction == 0 {
            let position = usize::from((address >> 1) & !0x7c0);
            if !large && self.is_exit_door_address(address) {
                attribute = 0x8e8e;
            }
            let rows = if large { 10 } else { 7 };
            for row in 0..rows {
                self.write_attr2(position + xy(1, row), attribute);
            }
            if !large {
                self.write_attr2(position + xy(1, 7), 0);
            }
        } else if direction == 1 {
            let compare_address = if large { address & 0x1fff } else { address };
            if (!large && matches!(door_type, 10 | 14) || large && matches!(door_type, 4 | 12 | 16))
                || self.is_exit_door_address(compare_address)
            {
                attribute = 0x8e8e;
            }
            let position = usize::from(address >> 1);
            let end = if large { 8 } else { 5 };
            for row in 1..=end {
                self.write_attr2(position + xy(1, row), attribute);
            }
        } else if direction == 2 {
            let position = usize::from((address >> 1) & !0x1f);
            let end = if large { 6 } else { 4 };
            for row in 1..=2 {
                for column in (0..=end).step_by(2) {
                    let value = if !large && column == 4 {
                        attribute.wrapping_add(0x101) & 0xff
                    } else {
                        attribute.wrapping_add(0x101)
                    };
                    self.write_attr2(position + xy(column, row), value);
                }
            }
        } else {
            let position = usize::from(address >> 1) + usize::from(large);
            let start = if large { 0 } else { 2 };
            let end = if large { 6 } else { 4 };
            for row in 1..=2 {
                for column in (start..=end).step_by(2) {
                    self.write_attr2(position + xy(column, row), attribute.wrapping_add(0x101));
                }
                if !large {
                    self.write_attr2(
                        position + xy(0, row),
                        attribute.wrapping_add(0x101) & 0xff00,
                    );
                }
            }
        }
    }

    fn is_exit_door_address(&self, address: u16) -> bool {
        (0..4).any(|index| self.r16(0x19e2 + index * 2) == address)
    }

    pub fn dungeon_load_single_door_tile_attribute(&mut self) {
        let floor_count = usize::from(self.r16(0x44e) >> 1);
        for index in 0..floor_count {
            let position = usize::from(self.r16(0x6c0 + index * 2));
            if self.r8(0x12000 + position) & 0xf0 == 0x80 {
                let attribute = self.r16(0x12000 + position) | 0x1010;
                self.write_attr2(position, attribute);
                self.write_attr2(position + xy(0, 1), attribute);
            } else {
                let attribute = self.r16(0x13000 + position) | 0x1010;
                self.write_attr1(position, attribute);
                self.write_attr1(position + xy(0, 1), attribute);
            }
        }
        let palace_count = usize::from(self.r16(0x450) >> 1);
        for index in 0..palace_count {
            let position = usize::from(self.r16(0x6d0 + index * 2));
            if self.r8(0x12000 + position) & 0xf0 == 0x80 {
                let attribute = self.r16(0x12000 + position) | 0x2020;
                self.write_attr2(position, attribute);
                self.write_attr2(position + xy(0, 1), attribute);
            } else {
                let attribute = self.r16(0x13000 + position) | 0x2020;
                self.write_attr1(position, attribute);
                self.write_attr1(position + xy(0, 1), attribute);
            }
        }
    }

    pub fn dungeon_load_object_attribute(&mut self) {
        for index in 0..usize::from(self.r16(0x432) >> 1) {
            let position = usize::from(self.r16(0x6a0 + index * 2));
            self.write_attr2(position, 0x3b3b);
            self.write_attr2(position + xy(0, 1), 0x3b3b);
        }

        let boundaries = [
            self.r16(0x438),
            self.r16(0x47e),
            self.r16(0x482),
            self.r16(0x4a2),
            self.r16(0x4a4),
            self.r16(0x43a),
            self.r16(0x480),
            self.r16(0x484),
            self.r16(0x4a6),
            self.r16(0x4a8),
        ];
        let mut offset = 0u16;
        let mut marker = 0x3030u16;
        for (kind, &end) in boundaries.iter().enumerate() {
            if kind == 5 {
                marker = (marker & 0x0707) | 0x3434;
            }
            while offset != end {
                let position = usize::from(self.r16(0x6b0 + usize::from(offset)));
                match kind {
                    0 => {
                        self.write_attr2(position + xy(1, 2), 0);
                        self.write_attr2(position + xy(1, 0), 0x2626);
                        self.write_attr2(position + xy(1, 1), marker);
                    }
                    1 | 2 | 6 | 7 => {
                        let wall = if kind == 2 || kind == 7 {
                            0x5f5f
                        } else {
                            0x5e5e
                        };
                        self.write_attr2(position + xy(1, 0), wall);
                        self.write_attr2(position + xy(1, 1), marker);
                        self.write_attr2(position + xy(1, 2), wall);
                        self.write_attr2(position + xy(1, 3), wall);
                    }
                    3 | 8 => {
                        self.write_attr2(position + xy(1, 0), 0x3838);
                        self.write_attr2(position + xy(1, 1), marker);
                        self.write_attr2(position + xy(1, 2), 0);
                        self.write_attr2(position + xy(1, 3), 0);
                    }
                    4 | 9 => {
                        self.write_attr2(position + xy(1, 0), 0);
                        self.write_attr2(position + xy(1, 1), 0);
                        self.write_attr2(position + xy(1, 2), marker);
                        self.write_attr2(position + xy(1, 3), 0x3939);
                    }
                    5 => {
                        self.write_attr2(position + xy(1, 2), marker);
                        self.write_attr2(position + xy(1, 3), 0x2626);
                    }
                    _ => unreachable!(),
                }
                offset = offset.wrapping_add(2);
                marker = marker.wrapping_add(0x101);
            }
        }

        let mut offset = 0u16;
        let (kind, attribute, end) = if self.r16(0x43c) != 0 {
            (0, 0x1f1f, self.r16(0x43c))
        } else if self.r16(0x43e) != 0 {
            (1, 0x1e1e, self.r16(0x43e))
        } else {
            (2, 0x1d1d, self.r16(0x440))
        };
        self.w16(0x44a, kind);
        while offset != end {
            let position = usize::from(self.r16(0x6b8 + usize::from(offset)));
            self.write_upper_inroom_stair(position, attribute);
            offset = offset.wrapping_add(2);
        }
        while offset != self.r16(0x448) {
            self.w16(0x44a, 2);
            let position = usize::from(self.r16(0x6b8 + usize::from(offset)));
            self.write_attr2(position, 0xa03);
            self.write_attr1(position, 0xa03);
            self.write_attr2(position + 2, 0x30a);
            self.write_attr1(position + 2, 0x30a);
            self.write_attr2(position + xy(0, 1), 0x803);
            self.write_attr2(position + xy(2, 1), 0x308);
            offset = offset.wrapping_add(2);
        }
        offset = 0;
        while offset != self.r16(0x442) {
            self.w16(0x44a, 2);
            let position = usize::from(self.r16(0x6b8 + usize::from(offset)));
            self.write_attr2(position, 0x003);
            self.write_attr2(position + 2, 0x300);
            self.write_attr1(position, 0xa03);
            self.write_attr1(position + 2, 0x30a);
            self.write_attr2(position + xy(0, 1), 0x808);
            self.write_attr2(position + xy(2, 1), 0x808);
            offset = offset.wrapping_add(2);
        }
        while offset != self.r16(0x444) {
            self.w16(0x44a, 2);
            let position = usize::from(self.r16(0x6b8 + usize::from(offset)));
            self.write_attr2(position, 0x003);
            self.write_attr2(position + 2, 0x300);
            self.write_attr1(position, 0xa03);
            self.write_attr1(position + 2, 0x30a);
            offset = offset.wrapping_add(2);
        }

        let mut marker = 0x7070u16;
        offset = 0;
        while offset != self.r16(0x42c) {
            let index = usize::from(offset >> 1);
            if self.r16(0x500 + index * 2) & 0xf0 != 0x30 {
                let position = usize::from((self.r16(0x540 + index * 2) & 0x3fff) >> 1);
                self.write_attr2(position, marker);
                self.write_attr2(position + xy(0, 1), marker);
            }
            offset = offset.wrapping_add(2);
            marker = marker.wrapping_add(0x101);
        }
        if offset != self.r16(0x42e) {
            marker = 0xc0c0;
            while offset != self.r16(0x42e) {
                let index = usize::from(offset >> 1);
                let position = usize::from((self.r16(0x540 + index * 2) & 0x3fff) >> 1);
                self.write_attr2(position, marker);
                self.write_attr2(position + xy(0, 1), marker);
                offset = offset.wrapping_add(2);
                marker = (marker & 0xefef).wrapping_add(0x101);
            }
            self.w16(0x42e, 0);
        }

        self.write_chest_attributes();
        self.write_lower_inroom_stair_attributes();
        self.w8(0x200, self.r8(0x200).wrapping_add(1));
    }

    pub fn dungeon_load_attribute_selectable(&mut self) {
        match self.r8(0x200) {
            0 => {
                self.w8(0x200, 1);
                self.w16(0xb2, 0);
                self.w16(0xb4, 0);
                self.dungeon_load_basic_attribute_full(0x40);
            }
            1 => self.dungeon_load_basic_attribute_full(0x40),
            2 => self.dungeon_load_object_attribute(),
            3 => self.dungeon_load_door_attribute(),
            4 => {
                self.w8(0x200, 5);
                if self.r8(0x4bc) != 0 {
                    self.dungeon_flip_crystal_peg_attribute();
                }
            }
            5 => {}
            state => panic!("invalid dungeon attribute state {state}"),
        }
    }

    pub fn dungeon_load_attribute_table(&mut self) {
        self.w16(0xb2, 0);
        self.w16(0xb4, 0);
        self.dungeon_load_basic_attribute_full(0x1000);
        self.dungeon_load_object_attribute();
        self.dungeon_load_door_attribute();
        if self.r8(0x4bc) != 0 {
            self.dungeon_flip_crystal_peg_attribute();
        }
        self.w8(0x200, 0);
    }

    fn write_upper_inroom_stair(&mut self, position: usize, attribute: u16) {
        self.write_attr2(position, 0x02);
        self.write_attr1(position + xy(0, 3), 0x02);
        self.write_attr2(position + 2, 0x0200);
        self.write_attr1(position + xy(2, 3), 0x0200);
        self.write_attr2(position + xy(0, 1), 0x01);
        self.write_attr1(position + xy(0, 2), 0x01);
        self.write_attr2(position + xy(2, 1), 0x0100);
        self.write_attr1(position + xy(2, 2), 0x0100);
        for row in 1..=2 {
            self.write_attr2(position + xy(1, row), attribute);
            self.write_attr1(position + xy(1, row), attribute);
        }
    }

    fn write_chest_attributes(&mut self) {
        let mut offset = 0u16;
        let mut marker = 0x5858u16;
        let chest_count = self.r16(0x496);
        if chest_count != 0 {
            let reveals =
                |tag: u8| tag == 0x27 || tag == 0x3c || tag == 0x3e || (0x29..0x33).contains(&tag);
            if reveals(self.r8(0xae)) || reveals(self.r8(0xaf)) {
                return;
            }
            while offset != chest_count {
                let address = 0x6e0 + usize::from(offset);
                let chest = self.r16(address);
                if chest != 0 {
                    let position = usize::from((chest & 0x7fff) >> 1);
                    self.write_attr2(position, marker);
                    self.write_attr2(position + xy(0, 1), marker);
                    if chest & 0x8000 != 0 {
                        self.w16(address, chest & 0x7fff);
                        self.write_attr2(position + xy(2, 1), marker);
                        self.write_attr2(position + xy(0, 2), marker);
                        self.write_attr2(position + xy(2, 2), marker);
                    }
                }
                offset = offset.wrapping_add(2);
                marker = marker.wrapping_add(0x101);
            }
        }
        while offset != self.r16(0x498) {
            let address = 0x6e0 + usize::from(offset);
            let chest = self.r16(address);
            self.w16(address, chest | 0x8000);
            let position = usize::from((chest & 0x7fff) >> 1);
            self.write_attr2(position, marker);
            self.write_attr2(position + xy(0, 1), marker);
            offset = offset.wrapping_add(2);
            marker = marker.wrapping_add(0x101);
        }
    }

    fn write_lower_inroom_stair_attributes(&mut self) {
        let (kind, attribute, end) = if self.r16(0x49a) != 0 {
            (0, 0x3f3f, self.r16(0x49a))
        } else if self.r16(0x49c) != 0 {
            (1, 0x3e3e, self.r16(0x49c))
        } else {
            (2, 0x3d3d, self.r16(0x49e))
        };
        self.w16(0x44a, kind);
        let mut offset = 0u16;
        while offset != end {
            let position = usize::from(self.r16(0x6ec + usize::from(offset)));
            self.write_attr1(position, 0x02);
            self.write_attr2(position + xy(0, 3), 0x02);
            self.write_attr1(position + xy(0, 1), 0x01);
            self.write_attr2(position + xy(0, 2), 0x01);
            self.write_attr1(position + 2, 0x0200);
            self.write_attr2(position + xy(2, 3), 0x0200);
            self.write_attr1(position + xy(2, 1), 0x0100);
            self.write_attr2(position + xy(2, 2), 0x0100);
            for row in 1..=2 {
                self.write_attr1(position + xy(1, row), attribute);
                self.write_attr2(position + xy(1, row), attribute);
            }
            offset = offset.wrapping_add(2);
        }
        if self.r16(0x4ae) != 0 {
            self.w16(0x44a, 2);
            offset = 0;
            while offset != self.r16(0x4ae) {
                let position = usize::from(self.r16(0x6ec + usize::from(offset)));
                self.write_attr1(position + xy(0, 3), 0xa03);
                self.write_attr1(position + xy(2, 3), 0x30a);
                self.write_attr2(position + xy(0, 3), 0x003);
                self.write_attr2(position + xy(2, 3), 0x300);
                self.write_attr2(position + xy(0, 2), 0x808);
                self.write_attr2(position + xy(2, 2), 0x808);
                offset = offset.wrapping_add(2);
            }
        }
    }

    pub fn room_tag_north_west_trigger(&mut self, slot: usize) {
        if self.r16(0x20) & 0x100 == 0 && self.r16(0x22) & 0x100 == 0 {
            self.room_tag_quadrant_trigger(slot);
        }
    }

    pub fn dung_tag_routine_0x2a(&mut self, slot: usize) {
        if self.r16(0x20) & 0x100 != 0 && self.r16(0x22) & 0x100 == 0 {
            self.room_tag_quadrant_trigger(slot);
        }
    }

    pub fn dung_tag_routine_0x2b(&mut self, slot: usize) {
        if self.r16(0x20) & 0x100 == 0 && self.r16(0x22) & 0x100 != 0 {
            self.room_tag_quadrant_trigger(slot);
        }
    }

    pub fn dung_tag_routine_0x2c(&mut self, slot: usize) {
        if self.r16(0x20) & 0x100 != 0 && self.r16(0x22) & 0x100 != 0 {
            self.room_tag_quadrant_trigger(slot);
        }
    }

    pub fn dung_tag_routine_0x2d(&mut self, slot: usize) {
        if self.r16(0x20) & 0x100 == 0 {
            self.room_tag_quadrant_trigger(slot);
        }
    }

    pub fn dung_tag_routine_0x2e(&mut self, slot: usize) {
        if self.r16(0x20) & 0x100 != 0 {
            self.room_tag_quadrant_trigger(slot);
        }
    }

    pub fn dung_tag_routine_0x2f(&mut self, slot: usize) {
        if self.r16(0x22) & 0x100 == 0 {
            self.room_tag_quadrant_trigger(slot);
        }
    }

    pub fn dung_tag_routine_0x30(&mut self, slot: usize) {
        if self.r16(0x22) & 0x100 != 0 {
            self.room_tag_quadrant_trigger(slot);
        }
    }

    pub fn room_tag_quadrant_trigger(&mut self, slot: usize) {
        let tag = self.r8(0xae + slot);
        if tag >= 0xb {
            if tag >= 0x29 {
                if self.hooks.sprite_check_if_screen_is_clear(&self.ram) {
                    self.room_tag_operate_chest_reveal(slot);
                }
            } else {
                let down = self.r8(0x641) ^ 1;
                if down != self.r8(0x468) {
                    self.w8(0x468, down);
                    self.w8(0x12f, 37);
                    self.w8(0x11, 5);
                    self.w16(0x68e, 0);
                    self.w16(0x690, 0);
                }
            }
        } else if self.hooks.sprite_check_if_screen_is_clear(&self.ram) {
            self.dung_tag_routine_trapdoors_up();
        }
    }

    pub fn dung_tag_routine_trapdoors_up(&mut self) {
        if self.r16(0x468) != 0 {
            self.w16(0x468, 0);
            self.w16(0x68e, 0);
            self.w16(0x690, 0);
            self.w8(0x12f, 0x1b);
            self.w8(0x11, 5);
        }
    }

    pub fn room_tag_room_trigger(&mut self, slot: usize) {
        if self.hooks.sprite_check_if_room_is_clear(&self.ram) {
            if self.r8(0xae + slot) == 10 {
                self.dung_tag_routine_trapdoors_up();
            } else {
                self.room_tag_operate_chest_reveal(slot);
            }
        }
    }

    pub fn room_tag_rekillable_boss(&mut self, _slot: usize) {
        if self.hooks.sprite_check_if_room_is_clear(&self.ram) {
            self.w8(0xffc, 0);
            self.w8(0xaf, 0);
        }
    }

    pub fn room_tag_room_trigger_block_door(&mut self, _slot: usize) {
        if self.r8(0x642) != 0 && self.r16(0x468) != 0 {
            self.w16(0x468, 0);
            self.w16(0x68e, 0);
            self.w16(0x690, 0);
            self.w8(0x11, 5);
        }
    }

    pub fn room_tag_prize_trigger_door_door(&mut self, slot: usize) {
        let prizes = if self.r8(0xf3ca) != 0 {
            self.r8(0xf37a)
        } else {
            self.r8(0xf374)
        };
        let palace = usize::from(self.r8(0x40c) >> 1);
        if prizes & DUNGEON_CRYSTAL_PENDANT_BIT[palace] != 0 {
            self.w16(0x468, 0);
            self.w16(0x68e, 0);
            self.w16(0x690, 0);
            self.w8(0x11, 5);
            self.w8(0xae + slot, 0);
        }
    }

    pub fn room_tag_kill_room_block(&mut self, slot: usize) {
        if self.r16(0x20) & 0x100 != 0
            && self.r16(0x22) & 0x100 != 0
            && self.hooks.sprite_check_if_screen_is_clear(&self.ram)
        {
            self.w8(0x12f, 0x1b);
            self.w8(0xae + slot, 0);
        }
    }

    pub fn room_tag_get_heart_for_prize(&mut self, slot: usize) {
        if self.r16(0x402) & 0x8000 == 0 {
            return;
        }
        let prizes = if self.r8(0xf3ca) != 0 {
            self.r8(0xf37a)
        } else {
            self.r8(0xf374)
        };
        let palace = usize::from(self.r8(0x40c) >> 1);
        if prizes & DUNGEON_CRYSTAL_PENDANT_BIT[palace] == 0 {
            self.w8(0x4c2, 128);
            if self
                .hooks
                .ancilla_spawn_falling_prize(&mut self.ram, BOSS_FINISHED_FALLING_ITEM[palace])
                < 0
            {
                return;
            }
        }
        self.w8(0xae + slot, 0);
    }

    pub fn room_tag_agahnim(&mut self, _slot: usize) {
        if self.r8(0xf280 + 0x5b) & 0x20 == 0 && self.r16(0x402) & 0x8000 != 0 {
            self.hooks.palette_revert_translucency_swap(&mut self.ram);
            self.w8(0xae, 0);
            self.prepare_dungeon_exit_from_boss_fight();
        }
    }

    pub fn room_tag_ganon_door(&mut self, _slot: usize) {
        for sprite in (0..16).rev() {
            let state = self.r8(0xdd0 + sprite);
            if state == 4 || state != 0 && self.r8(0xf60 + sprite) & 64 == 0 {
                return;
            }
        }
        if self.r8(0x5d) != 1 {
            self.w8(0x2e4, 26);
            self.w8(0x11, 26);
            self.w8(0xb0, 0);
            self.w8(0xae, 0);
            self.w8(0x3ef, 1);
            self.w8(0x3a, 0);
            self.w8(0x3c, 0);
            self.w16(0xc8, 0x364);
        }
    }

    pub fn room_tag_push_block_for_chest(&mut self, slot: usize) {
        if self.r8(0x14) == 0 && self.r8(0x641) != 0 {
            self.room_tag_operate_chest_reveal(slot);
        }
    }

    pub fn room_tag_trigger_chest(&mut self, slot: usize) {
        if self.r8(0x31f) == 0 && self.room_tag_maybe_check_shutters().is_some() {
            self.room_tag_operate_chest_reveal(slot);
        }
    }

    pub fn room_tag_switch_trigger_toggle_door(&mut self, _slot: usize) {
        if self.r8(0x430) == 0 {
            if let Some(attribute) = self.room_tag_maybe_check_shutters() {
                self.w16(0x68e, 0);
                self.w16(0x690, 0);
                self.w8(0x12f, 0x25);
                self.push_pressure_plate(attribute);
                self.w16(0x468, self.r16(0x468) ^ 1);
                self.w8(0x430, 1);
            }
        } else if self.room_tag_maybe_check_shutters().is_none() {
            self.w8(0x430, 0);
        }
    }

    pub fn room_tag_switch_trigger_hold_door(&mut self, _slot: usize) {
        let count = usize::from(self.r16(0x478) >> 1);
        let mut state = None;
        for index in 0..count {
            if self.r16(0x500 + index * 2) == 5 {
                let related = self.r16(0x466);
                if related != 0xffff {
                    state = Some(related);
                }
                break;
            }
        }
        let state = state.unwrap_or_else(|| {
            u16::from(
                self.r8(0x646) == 0
                    && self.r8(0x642) == 0
                    && self.room_tag_check_for_pressed_switch().is_none(),
            )
        });
        if state != self.r16(0x468) {
            self.w16(0x468, state);
            self.w16(0x68e, 0);
            self.w16(0x690, 0);
            if state == 0 {
                self.w8(0x12f, 0x25);
            }
            self.w8(0x11, 5);
        }
    }

    pub fn push_pressure_plate(&mut self, attribute: u8) {
        self.w8(0x11, 5);
        let position = self.r16(0x4b6);
        if attribute == 0x23 || position == 0 {
            return;
        }
        self.w8(0x10c, 5);
        self.w8(0x11, 23);
        self.w8(0xb0, 32);
        self.w16(0x22, self.r16(0x22).wrapping_add(2));
        let mut position = position;
        if self.r16(0x12000 + usize::from(position)) & 0xfe00 != 0x2400 {
            position = position.wrapping_add(1);
            self.w16(0x4b6, position);
        }
        self.dungeon_update_tile_map_with_common_tile(
            (position & 0x3f) << 3,
            (position >> 3) & 0x1f8,
            0x10,
        );
    }

    pub fn room_tag_operate_chest_reveal(&mut self, slot: usize) {
        self.w8(0xae + slot, 0);
        self.w16(0x1000, 0);
        self.w16(0x200, 0);
        let mut attribute = 0x5858;
        let count = usize::from(self.r16(0x496) >> 1);
        for index in 0..count {
            let position = usize::from((self.r16(0x6e0 + index * 2) >> 1) & 0x1fff);
            self.write_attr2(position, attribute);
            self.write_attr2(position + xy(0, 1), attribute);
            attribute = attribute.wrapping_add(0x101);

            let tiles = self.object_data::<4>(0x149c);
            for (delta, tile) in [xy(0, 0), xy(0, 1), xy(1, 0), xy(1, 1)]
                .into_iter()
                .zip(tiles)
            {
                self.set_bg2(position + delta, tile);
            }
            let upload = 0x1002 + index * 24;
            for (packet, delta) in [xy(0, 0), xy(0, 1), xy(1, 0), xy(1, 1)]
                .into_iter()
                .enumerate()
            {
                self.w16(
                    upload + packet * 6,
                    self.room_tag_build_chest_stripes((delta * 2) as u16, (index * 2) as u16),
                );
                self.w16(upload + packet * 6 + 2, 0x100);
                self.w16(upload + packet * 6 + 4, tiles[packet]);
            }
            self.w16(upload + 24, 0xffff);
            self.w16(0x1000, self.r16(0x1000).wrapping_add(24));
            self.w16(0x200, self.r16(0x200).wrapping_add(2));
        }
        self.w16(0x200, 0);
        self.w8(0x12f, 26);
        self.w8(0x14, 1);
    }

    pub fn room_tag_torch_puzzle_door(&mut self, _slot: usize) {
        let lit = (0..16)
            .filter(|&index| self.r16(0x540 + index * 2) & 0x8000 != 0)
            .count();
        let down = u16::from(lit < 4);
        if down != self.r16(0x468) {
            self.w16(0x468, down);
            self.w16(0x68e, 0);
            self.w16(0x690, 0);
            self.w8(0x12f, 0x1b);
            self.w8(0x11, 5);
        }
    }

    pub fn room_tag_switch_exploding_wall(&mut self, slot: usize) {
        if self.room_tag_maybe_check_shutters().is_some() {
            self.dung_tag_routine_blast_wall_stuff(slot);
        }
    }

    pub fn room_tag_pull_switch_exploding_wall(&mut self, slot: usize) {
        if self.r8(0x642) != 0 {
            self.dung_tag_routine_blast_wall_stuff(slot);
        }
    }

    pub fn dung_tag_routine_blast_wall_stuff(&mut self, slot: usize) {
        self.w8(0xae + slot, 0);
        let door = (0..16)
            .find(|&index| self.r16(0x1980 + index * 2) & !1 == 0x30)
            .expect("blast-wall door");
        self.w16(0x456, (door * 2) as u16);
        let mut index = (((self.r16(0x22) >> 8) & 1) + 1) * 2;
        if self.r16(0x19c0 + door * 2) & 2 != 0 {
            index = (self.r16(0x20) >> 8) & 1;
        }
        self.w16(
            0x10000 + 0x1c,
            u16::from(BLAST_WALL_TAB0[usize::from(index)]),
        );
        let position = self
            .r16(0x19a0 + door * 2)
            .wrapping_add(BLAST_WALL_TAB1[usize::from(index)]);
        self.w16(
            0x10000 + 0x1a,
            (position & 0x7e)
                .wrapping_mul(4)
                .wrapping_add(self.r16(0x62c)),
        );
        self.w16(
            0x10000 + 0x18,
            ((position & 0x1f80) >> 4).wrapping_add(self.r16(0x62e)),
        );
        self.w8(0x12f, 27);
        self.w8(0x454, 1);
        self.hooks.ancilla_add_blast_wall(&mut self.ram);
    }

    pub fn room_tag_torch_puzzle_chest(&mut self, slot: usize) {
        let lit = (0..16)
            .filter(|&index| self.r16(0x540 + index * 2) & 0x8000 != 0)
            .count();
        if lit >= 4 {
            self.room_tag_operate_chest_reveal(slot);
        }
    }

    pub fn room_tag_moving_wall_shake_it_up(&mut self, slot: usize) {
        let horizontal = if self.r8(0x1a) & 1 != 0 { 0xffff } else { 1 };
        self.w16(0x11a, horizontal);
        self.w16(0x11c, (0u16).wrapping_sub(horizontal));
        if self.r8(0xae + slot) == 0 {
            self.w16(0x11a, 0);
            self.w16(0x11c, 0);
        }
    }

    pub fn room_tag_moving_wall_torches_check(&mut self, slot: usize) {
        if self.r8(0x642) == 0 {
            let lit = (0..16)
                .filter(|&index| self.r16(0x540 + index * 2) & 0x8000 != 0)
                .count();
            if lit < 4 {
                return;
            }
        }
        self.w16(0x41a, self.r16(0x41a).wrapping_add(1));
        self.w16(0x642, 0);
        self.w16(0x402, self.r16(0x402) | (0x1000 >> slot));
        self.w8(0x12d, 7);
        self.w8(0x2e4, 1);
        self.w8(0xfc1, 1);
    }

    pub fn room_tag_moving_wall_east(&mut self, slot: usize) {
        let velocity = if self.r16(0x41a) == 0 {
            self.room_tag_moving_wall_torches_check(slot);
            0
        } else {
            self.w8(0xfc1, 1);
            self.room_tag_moving_wall_shake_it_up(slot);
            self.moving_wall_move_a_little()
        };
        self.w16(0x312, velocity);
        let offset = self.r16(0x422).wrapping_sub(velocity);
        self.w16(0x422, offset);
        self.w16(0xe0, self.r16(0xe2).wrapping_add(offset));
        if velocity != 0 {
            let first = MOVING_WALL_TAB1[usize::from(self.r8(0x41e) >> 1)] as u16;
            let second = MOVING_WALL_TAB1
                [usize::from(self.room_tag_advance_gigantic_wall(slot) >> 1)]
                as u16;
            if offset < first && offset < second {
                self.finish_moving_wall(slot);
            }
            self.w8(0x17, 5);
            let displacement = (offset.wrapping_neg() & 0x1f8) >> 3;
            self.w16(0x116, self.r16(0x42a).wrapping_sub(displacement) & 0x141f);
        }
    }

    pub fn room_tag_moving_wall_west(&mut self, slot: usize) {
        let velocity = if self.r16(0x41a) == 0 {
            self.room_tag_moving_wall_torches_check(slot);
            0
        } else {
            self.w8(0xfc1, 1);
            self.room_tag_moving_wall_shake_it_up(slot);
            self.moving_wall_move_a_little()
        };
        self.w16(0x312, velocity);
        let offset = self.r16(0x422).wrapping_add(velocity);
        self.w16(0x422, offset);
        self.w16(0xe0, self.r16(0xe2).wrapping_add(offset));
        if velocity != 0 {
            let first = MOVING_WALL_TAB0[usize::from(self.r8(0x41e) >> 1)];
            let second =
                MOVING_WALL_TAB0[usize::from(self.room_tag_advance_gigantic_wall(slot) >> 1)];
            if offset >= first && offset >= second {
                self.finish_moving_wall(slot);
            }
            self.w8(0x17, 5);
            let mut target = self.r16(0x42a).wrapping_add((offset & 0x1f8) >> 3);
            if target & 0x1020 != 0 {
                target = (target & 0x1020) ^ 0x420;
            }
            self.w16(0x116, target);
        }
    }

    fn finish_moving_wall(&mut self, slot: usize) {
        self.w8(0x12f, 0x1b);
        self.w8(0x12d, 5);
        self.w8(0xae + slot, 0);
        self.w8(0x2e4, 0);
        self.w8(0xfc1, 0);
        self.w16(0x11a, 0);
        self.w16(0x11c, 0);
    }

    pub fn moving_wall_move_a_little(&mut self) -> u16 {
        let accumulated = u16::from(self.r8(0x41d)) + 0x22;
        self.w8(0x41d, accumulated as u8);
        accumulated >> 8
    }

    pub fn room_tag_advance_gigantic_wall(&mut self, slot: usize) -> u8 {
        let mut index = self.r8(0x41e);
        if self.r8(0xae + slot) < 0x20 {
            self.w8(0x46c, 0);
            self.w8(0x1c, 0x16);
            index = index.wrapping_add(8);
        }
        index
    }

    pub fn room_tag_water_on(&mut self, _slot: usize) {
        if self.r8(0x642) != 0 {
            self.w8(0x12f, 0x1b);
            self.w8(0x12e, 0x2f);
            self.w8(0x11, 12);
            self.w8(0xb0, 0);
            self.w8(0x424, 1);
            self.w8(0xaf, 0);
            self.w16(0x402, self.r16(0x402) | 0x800);
            self.w8(0x642, 0);
            self.w8(0x45c, 0);
        }
    }

    pub fn room_tag_water_off(&mut self, _slot: usize) {
        if self.r8(0x642) == 0 {
            return;
        }
        self.w8(0x96, 3);
        self.w8(0x97, 0);
        self.w8(0x98, 0);
        self.w8(0x1e, 22);
        self.w8(0x1f, 1);
        self.w8(0x424, 1);
        self.hooks.adjust_water_hdma_window(&mut self.ram);
        self.w8(0x11, 11);
        self.w16(0xc007, 0);
        self.w16(0xc009, 0);
        self.w8(0xc00b, 31);
        self.w8(0x15, self.r8(0x15).wrapping_add(1));
        self.w8(0xaf, 0);
        self.w16(0x402, self.r16(0x402) | 0x800);
        self.w8(0x642, 0);
        let destination = ((self.r16(0x682) & 0x1ff).wrapping_sub(0x10) << 3)
            | ((self.r16(0x680) & 0x1ff).wrapping_sub(0x10) >> 3);
        let source = self.object_data::<16>(0x1438);
        self.draw_water_thing(DUNG_BG2 + usize::from(destination) * 2, &source);
        self.hooks
            .dungeon_prep_overlay_dma_next(&mut self.ram, 0, destination.wrapping_mul(2));
        self.w8(0x12f, 0x1b);
        self.w8(0x12e, 0x2e);
        self.w8(0x18, 1);
    }

    pub fn room_tag_water_gate(&mut self, _slot: usize) {
        if self.r16(0x402) & 0x800 != 0 || self.r8(0x642) == 0 {
            return;
        }
        self.w8(0x11, 13);
        self.w8(0xb0, 0);
        self.w8(0xaf, 0);
        self.w16(0x402, self.r16(0x402) | 0x800);
        self.w8(0x642, 0);
        self.w8(0x684, 0);
        self.w8(0x67a, 0);
        self.w8(0x96, 3);
        self.w8(0x97, 0);
        self.w8(0x98, 0);
        self.w8(0x1e, 0x16);
        self.w8(0x1f, 1);
        self.w8(0x99, 2);
        self.w8(0x9a, 0x62);
        self.w8(0xf280 + 0x3b, self.r8(0xf280 + 0x3b) | 32);
        self.w8(0xf280 + 0x7b, self.r8(0xf280 + 0x7b) | 32);
        self.w16(0xf000 + 0x28 * 2, self.r16(0xf000 + 0x28 * 2) | 0x100);
        self.room_tag_operate_water_flooring();

        let position = self.r16(0x472);
        let horizontal = ((position & 0x7e) << 2).wrapping_add(
            self.r16(0xb2)
                .wrapping_mul(16)
                .wrapping_add(self.r16(0x62c))
                .wrapping_add(40),
        );
        let upper = (position & 0x1f80) >> 4;
        self.w16(0x680, horizontal);
        self.w16(0x678, upper);
        self.w16(0x676, upper);
        self.w16(0x682, upper.wrapping_add(self.r16(0x62e)));
        self.w16(0x686, 0);
        self.w8(0x12f, 0x1b);
        self.w8(0x12e, 0x2f);
    }

    pub fn dungeon_load_basic_attribute_full(&mut self, loops: u16) {
        assert_ne!(loops, 0);
        for _ in 0..loops {
            let tile_index = usize::from(self.r16(0xb2) / 2);
            let tile0 = self.r16(DUNG_BG2 + tile_index * 2);
            let tile1 = self.r16(DUNG_BG2 + (tile_index + 1) * 2);
            let mut attribute0 = self
                .hooks
                .dungeon_tile_attribute(&self.ram, usize::from(tile0 & 0x3ff));
            let mut attribute1 = self
                .hooks
                .dungeon_tile_attribute(&self.ram, usize::from(tile1 & 0x3ff));
            if (0x10..0x1c).contains(&attribute0) {
                attribute0 |= (tile0 >> 14) as u8;
            }
            if (0x10..0x1c).contains(&attribute1) {
                attribute1 |= (tile1 >> 14) as u8;
            }
            let destination = usize::from(self.r16(0xb4));
            self.w8(0x12000 + destination, attribute0);
            self.w8(0x12000 + destination + 1, attribute1);
            self.w16(0xb4, self.r16(0xb4).wrapping_add(2));
            self.w16(0xb2, self.r16(0xb2).wrapping_add(4));
        }
        if self.r16(0xb4) == 0x2000 {
            self.w8(0x200, self.r8(0x200).wrapping_add(1));
        }
    }

    pub fn dung_tag_routine_0x00(&mut self, _slot: usize) {}

    pub fn dung_tag_routine_0x22_0x3b(&mut self, slot: usize, overlay: u8) {
        if self.r16(0x402) & 0x100 != 0 {
            self.w8(0xae + slot, 0);
            self.w8(0x4ba, overlay);
            self.w16(0xba, 0);
            self.w8(0xb0, 0);
            self.w8(0x12f, 0x1b);
            self.w8(0x11, 3);
        }
    }

    pub fn dung_tag_routine_0x1b(&mut self, _slot: usize) {}

    #[must_use]
    pub fn room_tag_get_tilemap_coords(&self) -> usize {
        usize::from(
            ((self.r16(0x22).wrapping_sub(1) & 0x1f8) >> 3)
                | ((self.r16(0x20).wrapping_add(14) & 0x1f8) << 3)
                | if self.r8(0xee) != 0 { 0x1000 } else { 0 },
        )
    }

    #[must_use]
    pub fn room_tag_maybe_check_shutters(&mut self) -> Option<u8> {
        self.w16(0x4b6, 0);
        if self.r8(0x2e4) != 0 || self.r8(0x4d) != 0 {
            return None;
        }
        let origin = self.room_tag_get_tilemap_coords();
        for position in [origin, origin + 64, origin + 1, origin + 65] {
            let attribute = self.r16(0x12000 + position);
            if attribute == 0x2323 || attribute == 0x2424 {
                if attribute == self.r16(0x12000 + position + 64) {
                    self.w16(0x4b6, position as u16);
                    return Some(attribute as u8);
                }
                return None;
            }
        }
        None
    }

    #[must_use]
    pub fn room_tag_check_for_pressed_switch(&mut self) -> Option<u8> {
        self.w16(0x4b6, 0);
        if self.r8(0x2e4) != 0 || self.r8(0x4d) != 0 {
            return None;
        }
        let origin = self.room_tag_get_tilemap_coords();
        for position in [origin, origin + 64, origin + 1, origin + 65] {
            let attribute = self.r16(0x12000 + position);
            if matches!(attribute, 0x2323 | 0x3a3a | 0x3b3b) {
                if attribute == self.r16(0x12000 + position + 64) {
                    self.w16(0x4b6, position as u16);
                    return Some(u8::from(attribute == 0x3b3b));
                }
                return None;
            }
        }
        None
    }

    pub fn dung_tag_routine_func2(&mut self, mut overlay: u8) {
        if self.r8(0x4ba) == 0 {
            self.w8(0x4ba, overlay);
        }
        if let Some(switch_offset) = self.room_tag_check_for_pressed_switch() {
            overlay = overlay.wrapping_add(switch_offset);
            if overlay != self.r8(0x4ba) {
                self.w8(0x4ba, overlay);
                self.w16(0xba, 0);
                self.w8(0xb0, 0);
                self.w8(0x12f, 27);
                self.w8(0x11, 3);
                self.w8(0x4bc, self.r8(0x4bc) ^ 1);
                self.hooks.dungeon_restore_star_tile_chr(&mut self.ram);
            }
        }
    }

    pub fn room_tag_holes0(&mut self, _slot: usize) {
        self.dung_tag_routine_func2(1);
    }

    pub fn dung_tag_routine_0x23(&mut self, _slot: usize) {
        self.dung_tag_routine_func2(3);
    }

    pub fn dung_tag_routine_0x34(&mut self, _slot: usize) {
        self.dung_tag_routine_func2(6);
    }

    pub fn dung_tag_routine_0x35(&mut self, _slot: usize) {
        self.dung_tag_routine_func2(8);
    }

    pub fn dung_tag_routine_0x36(&mut self, _slot: usize) {
        self.dung_tag_routine_func2(10);
    }

    pub fn dung_tag_routine_0x37(&mut self, _slot: usize) {
        self.dung_tag_routine_func2(12);
    }

    pub fn dung_tag_routine_0x39(&mut self, _slot: usize) {
        self.dung_tag_routine_func2(14);
    }

    pub fn dung_tag_routine_0x3a(&mut self, _slot: usize) {
        self.dung_tag_routine_func2(16);
    }

    pub fn room_tag_chest_holes0(&mut self, slot: usize) {
        self.dung_tag_routine_0x22_0x3b(slot, 0);
    }

    pub fn dung_tag_routine_0x3b(&mut self, slot: usize) {
        self.dung_tag_routine_0x22_0x3b(slot, 0x12);
    }

    pub fn room_tag_holes2(&mut self, slot: usize) {
        if self.room_tag_check_for_pressed_switch().is_some() {
            self.w8(0xae + slot, 0);
            self.w8(0x4ba, 5);
            self.w16(0xba, 0);
            self.w8(0xb0, 0);
            self.w8(0x12f, 0x1b);
            self.w8(0x11, 3);
        }
    }

    pub fn room_tag_operate_water_flooring(&mut self) {
        self.w16(0xba, 0);
        let source = self.object_data::<8>(0x110);
        let mut layout_offset = 0usize;
        loop {
            self.w16(0xb2, 0);
            self.w16(0xb4, 0);
            let encoded = u16::from(WATERGATE_LAYOUT[layout_offset])
                | (u16::from(WATERGATE_LAYOUT[layout_offset + 1]) << 8);
            if encoded == 0xffff {
                break;
            }
            let width = usize::from((encoded & 3) + 1);
            let height = usize::from(((encoded >> 8) & 3) + 1);
            self.w16(0xb2, width as u16);
            self.w16(0xb4, height as u16);
            self.w16(0xba, self.r16(0xba).wrapping_add(3));
            layout_offset += 3;
            let origin = usize::from((encoded & 0xfc) >> 2 | (encoded >> 10) << 6);
            for block_row in 0..height {
                for block_column in 0..width {
                    let destination = origin + xy(block_column * 4, block_row * 4);
                    for row in 0..4 {
                        for column in 0..4 {
                            self.set_bg1(
                                destination + xy(column, row),
                                source[(row & 1) * 4 + column],
                            );
                        }
                    }
                }
            }
            self.w16(0xb4, 0);
        }
    }

    fn run_room_tag(&mut self, tag: u8, slot: usize) {
        match tag {
            0 => self.dung_tag_routine_0x00(slot),
            1 | 11 | 41 => self.room_tag_north_west_trigger(slot),
            2 | 12 | 42 => self.dung_tag_routine_0x2a(slot),
            3 | 13 | 43 => self.dung_tag_routine_0x2b(slot),
            4 | 14 | 44 => self.dung_tag_routine_0x2c(slot),
            5 | 15 | 45 => self.dung_tag_routine_0x2d(slot),
            6 | 16 | 46 => self.dung_tag_routine_0x2e(slot),
            7 | 17 | 47 => self.dung_tag_routine_0x2f(slot),
            8 | 18 | 48 => self.dung_tag_routine_0x30(slot),
            9 | 19 | 49 => self.room_tag_quadrant_trigger(slot),
            10 | 50 => self.room_tag_room_trigger(slot),
            20 => self.room_tag_room_trigger_block_door(slot),
            21 => self.room_tag_prize_trigger_door_door(slot),
            22 => self.room_tag_switch_trigger_hold_door(slot),
            23 => self.room_tag_switch_trigger_toggle_door(slot),
            24 => self.room_tag_water_off(slot),
            25 => self.room_tag_water_on(slot),
            26 => self.room_tag_water_gate(slot),
            27 => self.dung_tag_routine_0x1b(slot),
            28 => self.room_tag_moving_wall_east(slot),
            29 => self.room_tag_moving_wall_west(slot),
            30 | 31 => self.room_tag_moving_wall_torches_check(slot),
            32 => self.room_tag_switch_exploding_wall(slot),
            33 => self.room_tag_holes0(slot),
            34 => self.room_tag_chest_holes0(slot),
            35 => self.dung_tag_routine_0x23(slot),
            36 => self.room_tag_holes2(slot),
            37 => self.room_tag_get_heart_for_prize(slot),
            38 => self.room_tag_kill_room_block(slot),
            39 => self.room_tag_trigger_chest(slot),
            40 => self.room_tag_pull_switch_exploding_wall(slot),
            51 => self.room_tag_torch_puzzle_door(slot),
            52 => self.dung_tag_routine_0x34(slot),
            53 => self.dung_tag_routine_0x35(slot),
            54 => self.dung_tag_routine_0x36(slot),
            55 => self.dung_tag_routine_0x37(slot),
            56 => self.room_tag_agahnim(slot),
            57 => self.dung_tag_routine_0x39(slot),
            58 => self.dung_tag_routine_0x3a(slot),
            59 => self.dung_tag_routine_0x3b(slot),
            60 => self.room_tag_push_block_for_chest(slot),
            61 => self.room_tag_ganon_door(slot),
            62 => self.room_tag_torch_puzzle_chest(slot),
            63 => self.room_tag_rekillable_boss(slot),
            _ => panic!("invalid dungeon room tag"),
        }
    }

    pub fn dungeon_handle_room_tags(&mut self) {
        if self.r8(0x4c7) == 0 {
            self.dungeon_detect_staircase();
            if self.hooks.misc_bug_fixes_enabled() && self.r8(0x11) != 0 {
                return;
            }
            for slot in 0..2 {
                self.w8(0x0e, slot as u8);
                self.run_room_tag(self.r8(0xae + slot), slot);
            }
        }
        self.w8(0x4c7, 0);
    }

    pub fn dungeon_detect_staircase(&mut self) {
        const BUGGY_LOOKUP: [i16; 8] = [7, 24, 8, 8, 0, 0, -1, 17];
        let direction = self.r8(0x67) & 12;
        if direction == 0 {
            return;
        }
        let y = self
            .r16(0x20)
            .wrapping_add_signed(BUGGY_LOOKUP[usize::from(direction >> 1)]);
        let position = usize::from(
            ((y & 0x1f8) << 3)
                | ((self.r16(0x22) & 0x1f8) >> 3)
                | if self.r8(0xee) != 0 { 0x1000 } else { 0 },
        );
        let attribute = self.r8(0x12000 + position + if direction == 4 { 0x80 } else { 0 });
        if !matches!(attribute, 0x26 | 0x38 | 0x39 | 0x5e | 0x5f) {
            return;
        }
        let staircase = self.r8(0x12000 + position + xy(0, 1));
        if staircase & 0xf8 != 0x30 {
            return;
        }
        if self.r8(0x308) & 0x80 != 0 {
            self.w16(0x20, self.r16(0xfc4));
            return;
        }
        self.w8(0x462, staircase);
        self.w8(0x463, (position >> 8) as u8);
        self.w16(0xa2, self.r16(0xa0));
        self.dungeon_flag_room_data_quadrants();
        if matches!(attribute, 0x38 | 0x39) {
            self.w8(0x464, 0x20);
            if attribute == 0x38 {
                self.dungeon_start_inter_room_trans_up();
            } else {
                self.dungeon_start_inter_room_trans_down();
            }
        }
        let index = usize::from(staircase & 3);
        self.w8(0xa0, self.r8(0xc001 + index));
        self.w8(0x48a, self.r8(0x63d + index));
        self.w8(
            0x492,
            if self.r8(0xee) != 0 || self.r8(0x476) != 0 {
                2
            } else {
                0
            },
        );
        self.w8(0xb0, 0);
        self.w8(0x48, 0);
        self.w8(0x3d, 0);
        self.w8(0x3a, 0);
        self.w8(0x3c, 0);
        self.w8(0x50, self.r8(0x50) & !1);
        match attribute {
            0x26 => {
                self.w8(0x11, 6);
                self.w8(0x12e, if self.r8(0x48a) < 0x34 { 22 } else { 24 });
            }
            0x38 | 0x39 => {
                self.w8(0x11, if attribute == 0x38 { 18 } else { 19 });
                self.w8(0x371, 7);
            }
            _ => {
                self.used_for_straight_inter_room_staircase();
                self.w8(0x11, 14);
            }
        }
    }

    pub fn clear_exploding_wall_from_tile_map_clear_one_pair(
        &mut self,
        destination: usize,
        src: &[u16],
    ) {
        assert!(src.len() >= 24);
        for column in 0..2 {
            for row in 0..12 {
                self.w16(destination + xy(column, row) * 2, src[column * 12 + row]);
            }
        }
    }

    #[must_use]
    pub fn get_door_graphics_index(&self, door: usize, raw_door: usize) -> u8 {
        let mut door_type = self.r16(0x1980 + door * 2) as u8 & 0xfe;
        if self.r16(0x68c) & UPPER_BITMASKS[raw_door] != 0 {
            door_type = DOOR_TYPE_REMAP[usize::from(door_type >> 1)];
        }
        door_type
    }

    pub fn draw_door_to_tile_map_north(&mut self, door: usize, raw_door: usize) {
        let graphics = usize::from(self.get_door_graphics_index(door, raw_door) >> 1);
        self.object_draw_door_up_4x3(usize::from(DOOR_TYPE_SRC_DATA[graphics]), door);
    }

    pub fn draw_door_to_tile_map_south(&mut self, door: usize, raw_door: usize) {
        let graphics = usize::from(self.get_door_graphics_index(door, raw_door) >> 1);
        self.object_draw_door_down_4x3(usize::from(DOOR_TYPE_SRC_DATA2[graphics]), door);
    }

    pub fn draw_door_to_tile_map_west(&mut self, door: usize, raw_door: usize) {
        let graphics = usize::from(self.get_door_graphics_index(door, raw_door) >> 1);
        self.object_draw_door_left_3x4(usize::from(DOOR_TYPE_SRC_DATA3[graphics]), door);
    }

    pub fn draw_door_to_tile_map_east(&mut self, door: usize, raw_door: usize) {
        let graphics = usize::from(self.get_door_graphics_index(door, raw_door) >> 1);
        self.object_draw_door_right_3x4(usize::from(DOOR_TYPE_SRC_DATA4[graphics]), door);
    }

    pub fn get_door_draw_data_index_north(&mut self, door: usize, raw_door: usize) {
        let door_type = self.r16(0x1980 + door * 2) as u8 & 0xfe;
        let mut frame = self.r16(0x692);
        if frame == 0 || frame == 4 {
            self.draw_door_to_tile_map_north(door, raw_door);
            return;
        }
        if door_type == 36 || door_type == 38 || door_type >= 0x42 {
            frame = frame.wrapping_add(4);
        }
        if door_type == 24 || door_type == 68 {
            frame = frame.wrapping_add(2);
        }
        self.object_draw_door_up_4x3(usize::from(DOOR_ANIM_UP_SRC[usize::from(frame >> 1)]), door);
    }

    pub fn get_door_draw_data_index_south(&mut self, door: usize, raw_door: usize) {
        let door_type = self.r16(0x1980 + door * 2) as u8 & 0xfe;
        let mut frame = self.r16(0x692);
        if frame == 0 || frame == 4 {
            self.draw_door_to_tile_map_south(door, raw_door);
            return;
        }
        if door_type >= 0x42 {
            frame = frame.wrapping_add(4);
        }
        if door_type == 24 || door_type == 68 {
            frame = frame.wrapping_add(2);
        }
        self.object_draw_door_down_4x3(
            usize::from(DOOR_ANIM_DOWN_SRC[usize::from(frame >> 1)]),
            door,
        );
    }

    pub fn get_door_draw_data_index_west(&mut self, door: usize, raw_door: usize) {
        let door_type = self.r16(0x1980 + door * 2) as u8 & 0xfe;
        let mut frame = self.r16(0x692);
        if frame == 0 || frame == 4 {
            self.draw_door_to_tile_map_west(door, raw_door);
            return;
        }
        if door_type >= 0x42 {
            frame = frame.wrapping_add(4);
        }
        if door_type == 24 || door_type == 68 {
            frame = frame.wrapping_add(2);
        }
        self.object_draw_door_left_3x4(
            usize::from(DOOR_ANIM_LEFT_SRC[usize::from(frame >> 1)]),
            door,
        );
    }

    pub fn get_door_draw_data_index_east(&mut self, door: usize, raw_door: usize) {
        let door_type = self.r16(0x1980 + door * 2) as u8 & 0xfe;
        let mut frame = self.r16(0x692);
        if frame == 0 || frame == 4 {
            self.draw_door_to_tile_map_east(door, raw_door);
            return;
        }
        if door_type >= 0x42 {
            frame = frame.wrapping_add(4);
        }
        if door_type == 24 || door_type == 68 {
            frame = frame.wrapping_add(2);
        }
        self.object_draw_door_right_3x4(
            usize::from(DOOR_ANIM_RIGHT_SRC[usize::from(frame >> 1)]),
            door,
        );
    }

    pub fn get_door_draw_data_index_north_clean_door_index(&mut self, door: usize) {
        self.get_door_draw_data_index_north(door, door);
    }

    pub fn get_door_draw_data_index_south_clean_door_index(&mut self, door: usize) {
        self.get_door_draw_data_index_south(door, door);
    }

    pub fn get_door_draw_data_index_west_clean_door_index(&mut self, door: usize) {
        self.get_door_draw_data_index_west(door, door);
    }

    pub fn get_door_draw_data_index_east_clean_door_index(&mut self, door: usize) {
        self.get_door_draw_data_index_east(door, door);
    }

    pub fn door_door_step1_north(&mut self, door: usize, mut dma: usize) -> usize {
        let mut position = self.r16(0x19a0 + door * 2);
        if position & 0x1fff >= DOOR_POSITION_TO_TILEMAP_OFFS_UP[6] {
            position = position.wrapping_sub(0x500);
            if self.r16(0x1980 + door * 2) as u8 & 0xfe >= 0x42 {
                position = position.wrapping_sub(0x300);
            }
            self.get_door_draw_data_index_south(door ^ 8, door & 7);
            dma = self
                .hooks
                .dungeon_prep_overlay_dma_next(&mut self.ram, dma, position);
            self.dungeon_load_single_door_attribute(door ^ 8);
        }
        self.get_door_draw_data_index_north(door, door & 7);
        dma
    }

    pub fn door_door_step1_south(&mut self, door: usize, mut dma: usize) -> usize {
        let mut position = self.r16(0x19a0 + door * 2);
        if position & 0x1fff < DOOR_POSITION_TO_TILEMAP_OFFS_DOWN[9] {
            position = position.wrapping_add(0x500);
            if self.r16(0x1980 + door * 2) as u8 & 0xfe >= 0x42 {
                position = position.wrapping_add(0x300);
            }
            self.get_door_draw_data_index_north(door ^ 8, door & 7);
            dma = self
                .hooks
                .dungeon_prep_overlay_dma_next(&mut self.ram, dma, position);
            self.dungeon_load_single_door_attribute(door ^ 8);
        }
        self.get_door_draw_data_index_south(door, door & 7);
        dma
    }

    pub fn door_door_step1_west(&mut self, door: usize, mut dma: usize) -> usize {
        let mut position = self.r16(0x19a0 + door * 2);
        if position & 0x7ff >= DOOR_POSITION_TO_TILEMAP_OFFS_LEFT[6] {
            position = position.wrapping_sub(16);
            if self.r16(0x1980 + door * 2) as u8 & 0xfe >= 0x42 {
                position = position.wrapping_sub(12);
            }
            self.get_door_draw_data_index_east(door ^ 8, door & 7);
            dma = self
                .hooks
                .dungeon_prep_overlay_dma_next(&mut self.ram, dma, position);
            self.dungeon_load_single_door_attribute(door ^ 8);
        }
        self.get_door_draw_data_index_west(door, door & 7);
        dma
    }

    pub fn door_door_step1_east(&mut self, door: usize, mut dma: usize) -> usize {
        let mut position = self.r16(0x19a0 + door * 2);
        if position & 0x7ff < DOOR_POSITION_TO_TILEMAP_OFFS_RIGHT[6] {
            position = position.wrapping_add(16);
            if self.r16(0x1980 + door * 2) as u8 & 0xfe >= 0x42 {
                position = position.wrapping_add(12);
            }
            self.get_door_draw_data_index_west(door ^ 8, door & 7);
            dma = self
                .hooks
                .dungeon_prep_overlay_dma_next(&mut self.ram, dma, position);
            self.dungeon_load_single_door_attribute(door ^ 8);
        }
        self.get_door_draw_data_index_east(door, door & 7);
        dma
    }

    pub fn draw_door_opening_step1(&mut self, door: usize, dma: usize) -> usize {
        self.w16(0x460, (door * 2) as u16);
        self.w16(0x694, (door * 2) as u16);
        match self.r16(0x19c0 + door * 2) & 3 {
            0 => self.door_door_step1_north(door, dma),
            1 => self.door_door_step1_south(door, dma),
            2 => self.door_door_step1_west(door, dma),
            3 => self.door_door_step1_east(door, dma),
            _ => unreachable!(),
        }
    }

    pub fn draw_shutter_door_steps(&mut self, door: usize) {
        self.w16(0x460, (door * 2) as u16);
        self.w16(0x694, (door * 2) as u16);
        match self.r16(0x19c0 + door * 2) & 3 {
            0 => self.get_door_draw_data_index_north_clean_door_index(door),
            1 => self.get_door_draw_data_index_south_clean_door_index(door),
            2 => self.get_door_draw_data_index_west_clean_door_index(door),
            3 => self.get_door_draw_data_index_east_clean_door_index(door),
            _ => unreachable!(),
        }
    }

    pub fn draw_eye_watch_door(&mut self, door: usize) {
        self.w16(0x460, (door * 2) as u16);
        self.w16(0x694, (door * 2) as u16);
        match self.r16(0x19c0 + door * 2) & 3 {
            0 => self.draw_door_to_tile_map_north(door, door),
            1 => self.draw_door_to_tile_map_south(door, door),
            2 => self.draw_door_to_tile_map_west(door, door),
            3 => self.draw_door_to_tile_map_east(door, door),
            _ => unreachable!(),
        }
    }

    pub fn dungeon_load_toggle_door_attr_other_entry(&mut self, door: usize) {
        self.dungeon_load_single_door_attribute(door);
        self.dungeon_load_single_door_tile_attribute();
    }

    pub fn draw_completely_open_door(&mut self) {
        let mut offset = self.r16(0x438);
        let mut marker = 0x3030u16.wrapping_add((offset >> 1).wrapping_mul(0x101));
        while offset != self.r16(0x47e) {
            let position = usize::from(self.r16(0x6b0 + usize::from(offset)));
            self.write_attr2(position + xy(1, 0), 0x5e5e);
            self.write_attr2(position + xy(1, 1), marker);
            self.write_attr2(position + xy(1, 2), 0);
            self.write_attr2(position + xy(1, 3), 0);
            offset = offset.wrapping_add(2);
            marker = marker.wrapping_add(0x101);
        }
        while offset != self.r16(0x482) {
            let position = usize::from(self.r16(0x6b0 + usize::from(offset)));
            self.write_attr2(position + xy(1, 0), 0x5f5f);
            self.write_attr2(position + xy(1, 1), marker);
            self.write_attr2(position + xy(1, 2), 0);
            self.write_attr2(position + xy(1, 3), 0);
            offset = offset.wrapping_add(2);
            marker = marker.wrapping_add(0x101);
        }
        let straight_end = self.r16(0x4a4);
        marker = marker.wrapping_add(
            straight_end
                .wrapping_sub(offset)
                .wrapping_div(2)
                .wrapping_mul(0x101),
        );
        offset = straight_end;
        marker = (marker & 0x0707) | 0x3434;
        marker = marker.wrapping_add(
            self.r16(0x43a)
                .wrapping_sub(offset)
                .wrapping_div(2)
                .wrapping_mul(0x101),
        );
        offset = self.r16(0x43a);
        while offset != self.r16(0x480) {
            let position = usize::from(self.r16(0x6b0 + usize::from(offset)));
            self.write_attr2(position + xy(1, 0), 0x5e5e);
            self.write_attr2(position + xy(1, 1), marker);
            self.write_attr2(position + xy(1, 2), 0);
            self.write_attr2(position + xy(1, 3), 0);
            offset = offset.wrapping_add(2);
            marker = marker.wrapping_add(0x101);
        }
        while offset != self.r16(0x484) {
            let position = usize::from(self.r16(0x6b0 + usize::from(offset)));
            self.write_attr2(position + xy(1, 0), 0x5f5f);
            self.write_attr2(position + xy(1, 1), marker);
            self.write_attr2(position + xy(1, 2), 0);
            self.write_attr2(position + xy(1, 3), 0);
            offset = offset.wrapping_add(2);
            marker = marker.wrapping_add(0x101);
        }
    }

    pub fn dungeon_opening_locked_door_combined(&mut self, skip_animation: bool) {
        let mut counter = 2u8;
        if skip_animation {
            self.w16(0x690, 16);
        } else {
            let step = self.r16(0x690).wrapping_add(1);
            self.w16(0x690, step);
            if step != 4 && step != 12 {
                if step != 16 {
                    return;
                }
            } else if step == 4 {
                self.w16(0x692, u16::from(counter));
                let door = usize::from(self.r8(0x12000 + usize::from(self.r16(0x68e))) & 0xf);
                let dma = self.draw_door_opening_step1(door, 0);
                let position = self.r16(0x19a0 + door * 2);
                self.hooks
                    .dungeon_prep_overlay_dma_next(&mut self.ram, dma, position);
                self.w8(0x12f, 21);
                self.w8(0x18, 1);
                return;
            }
        }
        let door_attribute = self.r8(0x12000 + usize::from(self.r16(0x68e)));
        if self.r16(0x690) == 12 || skip_animation {
            let mask = UPPER_BITMASKS[usize::from(door_attribute & 7)];
            self.w16(0x68c, self.r16(0x68c) | mask);
            self.w16(0x400, self.r16(0x400) | mask);
            counter = 4;
            self.w16(0x692, u16::from(counter));
            let door = usize::from(door_attribute & 0xf);
            let dma = self.draw_door_opening_step1(door, 0);
            let position = self.r16(0x19a0 + door * 2);
            self.hooks
                .dungeon_prep_overlay_dma_next(&mut self.ram, dma, position);
            self.w8(0x12f, 21);
            self.w8(0x18, 1);
        }
        if self.r16(0x690) == 16 {
            let door = usize::from(door_attribute & 0xf);
            self.dungeon_load_toggle_door_attr_other_entry(door);
            if door_attribute >= 0xf0 {
                let door_type = self.r16(0x1980 + door * 2) as u8;
                if (32..=38).contains(&door_type) {
                    self.draw_completely_open_door();
                }
            }
            self.w8(0x11, 0);
        }
    }

    pub fn open_cracked_door(&mut self) {
        self.dungeon_opening_locked_door_combined(true);
    }

    pub fn operate_shutter_doors(&mut self) {
        let mut dma = 0usize;
        let mut frame = 2u8;
        let step = self.r16(0x690).wrapping_add(1);
        self.w16(0x690, step);
        if step != 4 {
            frame = if self.r16(0x468) != 0 { 0 } else { 4 };
            if step != 8 {
                if step as u8 != 0x10 {
                    return;
                }
                self.w8(0x11, 0);
                self.w8(0x18, 0);
                return;
            }
        }
        self.w16(0x692, u16::from(frame));
        let mut current = 0u16;
        while current != 0x18 {
            self.w16(0x68e, current);
            let door = usize::from(current >> 1);
            let door_type = self.r16(0x1980 + door * 2) as u8 & 0xfe;
            if door_type == 68 || door_type == 24 {
                let mask = UPPER_BITMASKS[door];
                let down = self.r16(0x468) != 0;
                let opened = self.r16(0x68c) & mask != 0;
                if down == opened {
                    if step == 8 {
                        self.w8(0x12f, if down { 22 } else { 21 });
                        self.w16(0x68c, self.r16(0x68c) ^ mask);
                    }
                    self.draw_shutter_door_steps(door);
                    let position = self.r16(0x19a0 + door * 2);
                    dma = self
                        .hooks
                        .dungeon_prep_overlay_dma_next(&mut self.ram, dma, position);
                    if step == 8 {
                        self.dungeon_load_toggle_door_attr_other_entry(door);
                    }
                }
            }
            current = current.wrapping_add(2);
        }
        self.w16(0x68e, current.wrapping_sub(2));
        if dma != 0 {
            self.w8(0x710, 1);
            self.w8(0x18, 1);
            if step as u8 != 0x10 {
                return;
            }
        }
        self.w8(0x11, 0);
        self.w8(0x18, 0);
    }

    pub fn door_blast_wall_exploding_draw(&mut self, destination: usize) {
        let source = self.object_data::<49>(0x31ea);
        self.clear_exploding_wall_from_tile_map_clear_one_pair(
            DUNG_BG2 + destination * 2,
            &source[..24],
        );
        let mut column = destination + 2;
        let mut remaining = self.r16(0x454).wrapping_sub(1);
        while remaining != 0 {
            for row in 0..12 {
                self.set_bg2(column + xy(0, row), source[24]);
            }
            column += 1;
            remaining = remaining.wrapping_sub(1);
        }
        self.clear_exploding_wall_from_tile_map_clear_one_pair(
            DUNG_BG2 + column * 2,
            &source[25..],
        );
    }

    pub fn dungeon_clear_away_exploding_wall(&mut self) {
        self.w8(0x2e4, 6);
        self.w8(0xfc1, 6);
        if self.r8(0x10000) != 6 {
            return;
        }
        self.w16(0x45e, 0);
        self.w8(0x0c, 0);
        self.w16(0x690, 0);
        let door_x2 = self.r16(0x456);
        self.w16(0x460, door_x2);
        let door = usize::from(door_x2 >> 1);
        let address = self.r16(0x19a0 + door * 2).wrapping_sub(2);
        self.w16(0x19a0 + door * 2, address);
        let destination = usize::from(address >> 1);
        self.door_blast_wall_exploding_draw(destination);
        self.clear_and_stripe_exploding_wall(destination as u16);
        self.w16(0x710, 0xffff);
        let width = self.r16(0x454).wrapping_add(2);
        self.w16(0x454, width);
        if width == 21 {
            let mask = UPPER_BITMASKS[door];
            self.w16(0x68c, self.r16(0x68c) | mask);
            self.w16(0x400, self.r16(0x400) | mask);
            if self.r16(0x19c0 + door * 2) & 2 != 0 {
                self.w8(0x452, 1);
                self.w8(0xa6, 2);
            } else {
                self.w8(0x453, 1);
                self.w8(0xa7, 2);
            }
            self.w16(0xc19c, self.r16(0xa6));
            self.door_load_blast_wall_attr(door);
            self.w16(0x454, 0);
            self.w16(0x456, 0);
            self.dungeon_flag_room_data_quadrants();
            self.w8(0x2e4, 0);
            self.w8(0xfc1, 0);
        }
        self.w8(0x18, 3);
    }

    pub fn dungeon_draw_room_overlay_apply(&mut self, mut position: usize) {
        for _ in 0..4 {
            for column in 0..4 {
                let tile = self.r16(DUNG_BG2 + (position + column) * 2) & 0x3fe;
                self.w8(
                    0x12000 + position + column,
                    if tile == 0xee || tile == 0xfe {
                        0
                    } else {
                        0x20
                    },
                );
            }
            position += 64;
        }
    }

    pub fn apply_grayscale_fixed_incremental(&mut self) {
        let value = self.r8(0x9c) & 0x1f;
        let target = self.r8(0xc017);
        if value != target {
            self.dungeon_approach_fixed_color_variable(if value < target {
                value.wrapping_add(1)
            } else {
                value.wrapping_sub(1)
            });
        }
    }

    pub fn dungeon_approach_fixed_color_variable(&mut self, value: u8) {
        self.w8(0x9c, value | 0x20);
        self.w8(0x9d, value | 0x40);
        self.w8(0x9e, value | 0x80);
    }

    pub fn change_door_to_switch(&mut self) {
        assert_eq!(self.r16(0x4b0), 0);
    }

    pub fn dungeon_get_tele_msg(&self, room: usize) -> u16 {
        self.hooks.dungeon_tele_message(room)
    }

    #[must_use]
    pub fn get_dung_pal_info(index: usize) -> &'static DungPalInfo {
        &DUNG_PAL_INFOS[index]
    }

    pub fn dungeon_is_pit_that_hurts_player(&self) -> bool {
        let room = self.r16(0xa0);
        self.hooks
            .dungeon_pits_hurt_player()
            .iter()
            .rev()
            .any(|&candidate| candidate == room)
    }

    pub fn save_palace_deaths(&mut self) {
        let palace = usize::from(self.r16(0x40c) as u8);
        self.w16(0xf3e7 + (palace >> 1) * 2, self.r16(0xf403));
        if palace != 8 {
            self.w16(0xf403, 0);
        }
    }

    pub fn prepare_dungeon_exit_from_boss_fight(&mut self) {
        self.save_palace_deaths();
        self.hooks.save_dungeon_keys(&mut self.ram);
        self.w16(0x402, self.r16(0x402) | 0x8000);
        self.dungeon_flag_room_data_quadrants();
        let room = self.r8(0xa0);
        let index = DUNGEON_EXIT_FROM
            .iter()
            .position(|&candidate| candidate == room)
            .expect("boss room exit mapping");
        let destination = DUNGEON_EXIT_TO[index];
        self.w8(0xa0, destination);
        if destination == 0x20 {
            self.w8(0xf3c5, 3);
            self.w8(0xf282, self.r8(0xf282) | 0x20);
            self.w8(0xf3ca, self.r8(0xf3ca) ^ 0x40);
            self.hooks
                .sprite_load_graphics_properties_light_world_only(&mut self.ram);
            self.hooks
                .ancilla_terminate_select_interactives(&mut self.ram, 0);
            self.w8(0x37b, 0);
            self.w8(0x3c, 0);
            self.w8(0x3a, 0);
            self.w8(0x3ef, 0);
            self.w8(0x2e4, 1);
            self.w8(0x10c, 8);
            self.w8(0x10, 21);
            self.w8(0x11, 0);
            self.w8(0xb0, 0);
        } else if destination == 0x0d {
            self.w8(0x10, 24);
            self.w8(0x11, 0);
            self.w8(0x200, 0);
            self.w8(0x9a, 0x20);
        } else {
            if index >= 3 {
                self.w8(0x12c, 0xf1);
                self.w8(0x130, 0xf1);
                self.w8(0x10, 22);
            } else {
                self.w8(0x10, 19);
            }
            self.w8(0x10c, 8);
            self.w8(0x11, 0);
            self.w8(0xb0, 0);
        }
    }

    pub fn save_dungeon_keys(&mut self) {
        let mut index = self.r8(0x40c);
        if index == 0xff {
            return;
        }
        if index == 2 {
            index = 0;
        }
        self.w8(0xf37c + usize::from(index >> 1), self.r8(0xf36f));
    }

    pub fn mirror_save_room_data(&mut self) {
        if self.r8(0x40c) == 0xff {
            self.w8(0x12e, 60);
            return;
        }
        self.w8(0x11, 25);
        self.w8(0xb0, 0);
        self.w8(0x12e, 51);
        self.dungeon_flag_room_data_quadrants();
        self.save_dungeon_keys();
    }

    pub fn dungeon_adjust_after_spiral_stairs(&mut self) {
        let x_delta =
            (((self.r16(0xa0) & 0x0f) as i16 - (self.r16(0xa2) & 0x0f) as i16) * 0x200) as u16;
        self.w16(0x22, self.r16(0x22).wrapping_add(x_delta));
        self.w16(0xe2, self.r16(0xe2).wrapping_add(x_delta));
        for offset in [0usize, 2, 4, 6] {
            self.w16(
                0x608 + offset,
                self.r16(0x608 + offset).wrapping_add(x_delta),
            );
        }

        let y_delta = ((((self.r16(0xa0) & 0xf0) >> 4) as i16
            - ((self.r16(0xa2) & 0xf0) >> 4) as i16)
            * 0x200) as u16;
        self.w16(0x20, self.r16(0x20).wrapping_add(y_delta));
        self.w16(0xe8, self.r16(0xe8).wrapping_add(y_delta));
        for offset in [0usize, 2, 4, 6] {
            self.w16(
                0x600 + offset,
                self.r16(0x600 + offset).wrapping_add(y_delta),
            );
        }
    }

    pub fn dungeon_adjust_for_teleport_doors(&mut self, room: u8, flag: u8) {
        self.w16(0x48e, u16::from(room));
        self.w16(0xa2, u16::from(room));
        let x_delta = u16::from(room & 0x0f)
            .wrapping_mul(2)
            .wrapping_sub(self.r16(0x22) >> 8)
            .wrapping_add(u16::from(flag))
            << 8;
        self.w16(0x22, self.r16(0x22).wrapping_add(x_delta));
        self.w16(0xe2, self.r16(0xe2).wrapping_add(x_delta));
        for offset in [0usize, 2, 4, 6] {
            self.w16(
                0x608 + offset,
                self.r16(0x608 + offset).wrapping_add(x_delta),
            );
        }

        let y_delta = u16::from((room & 0xf0) >> 3).wrapping_sub(self.r16(0x20) >> 8) << 8;
        self.w16(0x20, self.r16(0x20).wrapping_add(y_delta));
        self.w16(0xe8, self.r16(0xe8).wrapping_add(y_delta));
        for offset in [0usize, 2, 4, 6] {
            self.w16(
                0x600 + offset,
                self.r16(0x600 + offset).wrapping_add(y_delta),
            );
        }
        let y_high = (self.r16(0x20) >> 8) as u8;
        for index in 0..20 {
            self.w8(0x1a14 + index, y_high);
        }
    }

    pub fn set_and_save_visited_quadrant_flags(&mut self) {
        let index = usize::from(
            (self.r8(0xa7) << 2)
                .wrapping_add(self.r8(0xa6) << 1)
                .wrapping_add(self.r8(0xaa))
                .wrapping_add(self.r8(0xa9)),
        );
        let visited = self.r16(0x408) | u16::from(QUADRANT_VISITING_FLAGS[index]);
        self.w16(0x408, visited);
        let room = usize::from(self.r16(0xa0));
        self.w16(0xf000 + room * 2, self.r16(0xf000 + room * 2) | visited);
    }

    pub fn save_quadrants_to_sram(&mut self) {
        let room = usize::from(self.r16(0xa0));
        self.w16(
            0xf000 + room * 2,
            self.r16(0xf000 + room * 2) | self.r16(0x408),
        );
    }

    pub fn dung_save_data_for_current_room(&mut self) {
        let room = usize::from(self.r16(0xa0));
        let value = (self.r16(0x402) >> 4) | (self.r16(0x400) & 0xf000) | self.r16(0x408);
        self.w16(0xf000 + room * 2, value);
    }

    pub fn dungeon_flag_room_data_quadrants(&mut self) {
        let index = usize::from(
            (self.r8(0xa7) << 2)
                .wrapping_add(self.r8(0xa6) << 1)
                .wrapping_add(self.r8(0xaa))
                .wrapping_add(self.r8(0xa9)),
        );
        self.w16(
            0x408,
            self.r16(0x408) | u16::from(QUADRANT_VISITING_FLAGS[index]),
        );
        self.dung_save_data_for_current_room();
    }

    pub fn dungeon_adjust_quadrant(&mut self) {
        self.w8(0xa8, self.r8(0x40e) | self.r8(0xaa) | self.r8(0xa9));
    }

    pub fn dungeon_adjust_for_room_layout(&mut self) {
        self.dungeon_adjust_quadrant();
        let layout = LAYOUT_QUADRANT_FLAGS[usize::from(self.r8(0xa8))];
        let full_x = if self.r8(0x452) != 0 || layout & if self.r8(0xa9) != 0 { 2 } else { 1 } == 0
        {
            2
        } else {
            0
        };
        let full_y = if self.r8(0x453) != 0 || layout & if self.r8(0xaa) != 0 { 8 } else { 4 } == 0
        {
            2
        } else {
            0
        };
        self.w8(0xa6, full_x);
        self.w8(0xa7, full_y);
        let override_value = self.r16(0xfc);
        if override_value as u8 != 0 {
            self.w8(0xa6, override_value as u8);
        }
        if (override_value >> 8) as u8 != 0 {
            self.w8(0xa7, (override_value >> 8) as u8);
        }
    }

    pub fn mirror_bg1_bg2_offs(&mut self) {
        self.w16(0xe0, self.r16(0xe2));
        self.w16(0xe6, self.r16(0xe8));
    }

    pub fn dungeon_transition_adjust_camera_x(&mut self, argument: u8) {
        let index = usize::from(argument) * 2;
        self.w16(0x614, UP_DOWN_SCROLL_X[index]);
        self.w16(0x616, UP_DOWN_SCROLL_X[index + 1]);
    }

    pub fn dungeon_transition_adjust_camera_y(&mut self, argument: u8) {
        let index = usize::from(argument);
        self.w16(0x610, UP_DOWN_SCROLL_Y[index]);
        self.w16(0x612, UP_DOWN_SCROLL_Y[index + 1]);
    }

    pub fn calculate_transition_landing(&mut self) -> u8 {
        let position = usize::from(
            ((self.r16(0x20).wrapping_add(12) & 0x1f8) << 3)
                | ((self.r16(0x22).wrapping_add(8) & 0x1f8) >> 3)
                | if self.r8(0xee) != 0 { 0x1000 } else { 0 },
        );
        let mut attribute = self.r8(0x12000 + position);
        let result = if attribute == 0 || attribute == 9 {
            0
        } else {
            attribute &= 0x8e;
            match attribute {
                0x80 => 1,
                0x82 => 2,
                0x84 | 0x88 => 3,
                0x86 => 4,
                _ => 2,
            }
        };
        self.w8(0x4e, result);
        result
    }

    pub fn subtile_transition_calculate_landing(&mut self) {
        let transition = self.r8(0x418);
        let mut landing = self.calculate_transition_landing();
        landing = match landing {
            2 => 1,
            4 => 2,
            value => value,
        };
        let index = usize::from(landing.wrapping_add(transition.wrapping_mul(5)));
        let mut value = STAIRCASE_TAB2[index];
        value = if value < 0 {
            value.wrapping_add(8)
        } else {
            value.wrapping_sub(8)
        };
        let address = if transition & 2 != 0 { 0x22 } else { 0x20 };
        self.w8(address, value as u8);
        self.w8(0x4b, 0);
    }

    pub fn dungeon_transition_move_link_out_door(&mut self) -> bool {
        let transition = self.r8(0x418);
        let index = usize::from(self.r8(0x4e).wrapping_add(transition.wrapping_mul(5)));
        let target = STAIRCASE_TAB2[index] as u8;
        let delta = if transition & 1 != 0 { 0xfffe } else { 2 };
        let address = if transition & 2 == 0 { 0x20 } else { 0x22 };
        self.w16(address, self.r16(address).wrapping_add(delta));
        self.r8(address) & 0xfe == target
    }

    pub fn dungeon_play_blip_and_cache_quadrant_visits(&mut self) {
        self.w8(0x4a0, 1);
        self.w8(0x12f, 36);
        self.set_and_save_visited_quadrant_flags();
    }

    #[must_use]
    pub fn manip_block_something(&self) -> (u16, u16) {
        let index = usize::from(self.r16(0x42c) >> 1);
        let position = self.r16(0x540 + index * 2);
        (
            (self.r16(0x22) & 0xfe00) | ((position & 0x007e) << 2),
            (self.r16(0x20) & 0xfe00) | ((position & 0x1f80) >> 4),
        )
    }

    #[must_use]
    pub fn room_tag_build_chest_stripes(&self, position: u16, index_x2: u16) -> u16 {
        let position = position.wrapping_add(self.r16(0x6e0 + usize::from(index_x2)));
        (((position & 0x40) << 4) | ((position & 0x303f) >> 1) | ((position & 0x0f80) >> 2))
            .swap_bytes()
    }

    pub fn dungeon_set_attr_for_activated_water_off(&mut self) {
        self.w8(0x99, 2);
        self.w8(0x9a, 0x32);
        self.w8(0x1d, 0);
        self.w8(0x96, 0);
        self.w8(0x46c, 0);
        self.w16(0x1e, 0);

        let count = usize::from(self.r16(0x442) >> 1);
        for index in 0..count {
            let destination = usize::from(self.r16(0x6b8 + index * 2));
            self.write_attr2(destination + xy(1, 1), 0x1d1d);
            self.write_attr2(destination + xy(1, 2), 0x1d1d);
        }
        let count = usize::from(self.r16(0x4ae) >> 1);
        for index in 0..count {
            let destination = usize::from(self.r16(0x6ec + index * 2));
            self.write_attr2(destination + xy(1, 1), 0x1d1d);
            self.write_attr2(destination + xy(1, 2), 0x1d1d);
        }
        self.w8(0x15, self.r8(0x15).wrapping_add(1));
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn dungeon_adjust_water_vomit(&mut self, source: &[u16], depth: usize) {
        assert!(depth != 0 && source.len() >= depth * 4);
        let mut destination = usize::from(self.r16(0x47c) >> 1) + xy(0, 2);
        for row in source.chunks_exact(4).take(depth) {
            for (column, &tile) in row.iter().enumerate() {
                self.set_bg2(destination + column, tile);
            }
            destination += xy(0, 1);
        }

        destination = usize::from(self.r16(0x47c) >> 1) + xy(0, 2);
        let mut upload = 0x1002;
        for _ in 0..4 {
            self.w16(upload, Self::dungeon_map_vram_addr(destination as u16));
            self.w16(upload + 2, 0x980);
            for row in 0..5 {
                self.w16(
                    upload + 4 + row * 2,
                    self.r16(DUNG_BG2 + (destination + xy(0, row)) * 2),
                );
            }
            upload += 14;
            destination += 1;
        }
        self.w16(upload, 0xffff);
        self.w8(0x14, 1);
    }

    pub fn dungeon_set_attr_for_activated_water(&mut self) {
        self.w16(0x1e, 0);
        let count = usize::from(self.r16(0x440) >> 1);
        for index in 0..count {
            let destination = usize::from(self.r16(0x6b8 + index * 2));
            self.write_attr2(destination, 0x003);
            self.write_attr2(destination + 2, 0x300);
            self.write_attr1(destination, 0xa03);
            self.write_attr1(destination + 2, 0x30a);
            self.write_attr2(destination + xy(0, 1), 0x808);
            self.write_attr2(destination + xy(2, 1), 0x808);
            for row in 1..=3 {
                self.write_attr1(destination + xy(0, row), 0x808);
                self.write_attr1(destination + xy(2, row), 0x808);
            }
        }

        let count = usize::from(self.r16(0x49e) >> 1);
        for index in 0..count {
            let destination = usize::from(self.r16(0x6ec + index * 2));
            self.write_attr2(destination + xy(0, 3), 0x003);
            self.write_attr2(destination + xy(2, 3), 0x300);
            self.write_attr1(destination + xy(0, 3), 0xa03);
            self.write_attr1(destination + xy(2, 3), 0x30a);
            self.write_attr2(destination + xy(0, 2), 0x808);
            self.write_attr2(destination + xy(2, 2), 0x808);
            for row in 0..=2 {
                self.write_attr1(destination + xy(0, row), 0x808);
                self.write_attr1(destination + xy(2, row), 0x808);
            }
        }
        self.w8(0x11, 0);
        self.w8(0x12, 0);
        self.w8(0xb0, 0);
    }

    pub fn flood_dam_fill(&mut self) {
        let water = self.r8(0x684).wrapping_add(1);
        self.w8(0x684, water);
        if water.wrapping_add(self.r8(0x676)) >= 225 {
            self.w8(0x45c, 0);
            self.w8(0x11, 0);
            self.w8(0xb0, 0);
            self.w8(0x1e, 0);
            self.w8(0x1f, 0);
            self.hooks.iris_spotlight_reset_table(&mut self.ram);
        }
    }

    pub fn flood_dam_expand(&mut self) {
        let watergate = self.r8(0x470).wrapping_add(1);
        self.w8(0x470, watergate);
        self.w16(0x686, u16::from(watergate >> 1));
        let offset = (watergate >> 1).wrapping_sub(8);
        self.w8(0x676, self.r8(0x678));
        let spotlight = self.r8(0x67a).wrapping_add(1);
        self.w8(0x67a, spotlight);
        self.w8(0x684, spotlight.wrapping_add(offset));
        if watergate & 0xf != 0 {
            return;
        }
        if watergate == 64 {
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        }
        let source_offset = WATERGATE_SRCS1[usize::from((watergate >> 4).wrapping_sub(1))];
        let source = self.object_data::<40>(usize::from(source_offset));
        let watergate_position = self.r16(0x472);
        self.room_draw_object_nx4(10, &source, DUNG_BG2 + usize::from(watergate_position));
        let mut position = watergate_position;
        let mut dma = 0;
        for _ in 0..3 {
            dma = self.hooks.dungeon_prep_overlay_dma_watergate(
                &mut self.ram,
                dma,
                position,
                0x881,
                4,
            );
            position = position.wrapping_add(6);
        }
        self.w8(0x18, 1);
    }

    pub fn ganon_extinguish_torch_adjust_translucency(&mut self) {
        self.hooks.palette_assert_translucency_swap(&mut self.ram);
        self.w8(0x333, 0xc0);
        self.dungeon_extinguish_torch();
    }

    pub fn ganon_extinguish_torch(&mut self) {
        self.w8(0x333, 193);
        self.dungeon_extinguish_torch();
    }

    pub fn dungeon_extinguish_torch(&mut self) {
        let torch = usize::from(self.r8(0x333) & 0xf);
        let object_x2 = self.r16(0x478).wrapping_add((torch as u16).wrapping_mul(2));
        let object = usize::from(object_x2 >> 1);
        let mut position = self.r16(0x540 + object * 2) & 0x7fff;
        let torch_data = usize::from((self.r16(0x520 + object * 2) & 0xff) >> 1);
        self.w16(0xfb40 + torch_data * 2, position);
        position &= 0x3fff;
        self.hooks
            .room_draw_adjust_torch_lighting_change(&mut self.ram, position, 0xec2, position);
        self.w8(0x18, 1);

        if self.r8(0xc005) != 0 && self.r8(0x45a) != 0 {
            let lit = self.r8(0x45a).wrapping_sub(1);
            self.w8(0x45a, lit);
            if lit < 3 {
                if lit == 0 {
                    self.w8(0x1d, 1);
                }
                self.w8(0xc017, LIT_TORCHES_COLOR_PLUS[usize::from(lit)]);
                self.w8(0x11, 10);
                self.w8(0xb0, 0);
            }
        }
        self.w8(0x4f0 + torch, 0);
        self.w8(0x333, 0);
    }

    pub fn dungeon_process_torches_and_doors(&mut self) {
        const X_OFFSETS: [i16; 4] = [0, 0, -1, 17];
        const Y_OFFSETS: [i16; 4] = [7, 24, 8, 8];
        const POS_OFFSETS: [u16; 4] = [2, 2, 0x80, 0x80];
        if self.r8(0x1a) & 3 == 0 && self.r8(0x112) == 0 {
            for torch in 0..16 {
                let timer = self.r8(0x4f0 + torch);
                if timer != 0 {
                    self.w8(0x4f0 + torch, timer.wrapping_sub(1));
                    if timer == 1 {
                        self.w8(0x333, 0xc0 + torch as u8);
                        self.dungeon_extinguish_torch();
                    }
                }
            }
        }
        if self.r8(0x2e4) == 0 {
            let direction = usize::from(self.r8(0x2f) >> 1);
            let mut position =
                (self.r16(0x20).wrapping_add_signed(Y_OFFSETS[direction]) & 0x1f8) << 3;
            position |= (self.r16(0x22).wrapping_add_signed(X_OFFSETS[direction]) & 0x1f8) >> 3;
            if self.r8(0xee) != 0 {
                position |= 0x1000;
            }
            let mut attribute = self.r8(0x12000 + usize::from(position));
            if attribute & 0xf0 != 0xf0 {
                position = position.wrapping_add(POS_OFFSETS[direction]);
                attribute = self.r8(0x12000 + usize::from(position));
            }
            if attribute & 0xf0 == 0xf0 {
                let door = usize::from(attribute & 0xf);
                self.w16(0x694, (door as u16) * 2);
                if self.r16(0x19c0 + door * 2) as usize & 3 == direction {
                    let door_type = self.r16(0x1980 + door * 2) as u8 & 0xfe;
                    if door_type == 0x42 {
                        if self.r8(0x372) != 0 && self.r8(0x2f1) < 63 {
                            self.w16(0x68e, position);
                            let debris = self.hooks.ancilla_add_door_debris(&mut self.ram);
                            if debris >= 0 {
                                let debris = debris as usize;
                                let address = self.r16(0x19a0 + door * 2);
                                self.w8(0x73c + debris, self.r16(0x19c0 + door * 2) as u8 & 3);
                                self.w16(
                                    0x728 + debris * 2,
                                    self.r16(0x62c).wrapping_add((address & 0x7e) * 4),
                                );
                                self.w16(
                                    0x732 + debris * 2,
                                    self.r16(0x62e).wrapping_add((address & 0x1f80) >> 4),
                                );
                            }
                            self.w8(0x12f, 27);
                            self.w8(0x11, 9);
                            self.hooks.sprite_repel_dash(&mut self.ram);
                            return;
                        }
                    } else if door_type == 0x1e {
                        self.w16(0x690, 0);
                        self.w16(0x68e, position);
                        let palace = usize::from(self.r16(0x40c) >> 1);
                        if self.r16(0xf366) & UPPER_BITMASKS[palace] != 0 {
                            self.open_key_door(door, position);
                            return;
                        }
                        if self.r16(0x4b8) == 0 {
                            self.w16(0x4b8, 1);
                            self.w16(0x1cf0, 0x7a);
                            self.hooks.main_show_text_message(&mut self.ram);
                        }
                    } else if (0x20..0x2c).contains(&door_type)
                        && door_type != 0x2a
                        && self.r8(0xf36f) != 0
                    {
                        self.w8(0xf36f, self.r8(0xf36f).wrapping_sub(1));
                        self.open_key_door(door, position);
                        return;
                    }
                }
            } else {
                self.w16(0x4b8, 0);
            }
        }

        let invisible = self.r16(0x436);
        if invisible & 0x80 == 0 && self.r8(0x6c) == 0 && self.r16(0x22) >> 8 == 0x0c {
            let direction = invisible as u8;
            let door = usize::from((invisible >> 8) >> 1);
            let mut opened = self.r16(0x68c);
            if direction != self.r8(0x2f) && direction ^ 2 == self.r8(0x2f) {
                opened |= UPPER_BITMASKS[door];
            } else {
                opened &= !UPPER_BITMASKS[door];
            }
            if opened != self.r16(0x68c) {
                self.w16(0x68c, opened);
                self.draw_eye_watch_door(door);
                let position = self.r16(0x19a0 + door * 2);
                self.hooks
                    .dungeon_prep_overlay_dma_next(&mut self.ram, 0, position);
                self.dungeon_load_toggle_door_attr_other_entry(door);
                self.w8(0x14, 1);
                self.w8(0x12f, 21);
                return;
            }
        }
        if self.r8(0x3a) & 0x80 == 0 || self.r8(0x3c) != 4 {
            return;
        }
        let mut position = (self
            .r16(0x20)
            .wrapping_add_signed(i16::from(self.r8(0x44) as i8))
            & 0x1f8)
            << 3;
        position |= (self
            .r16(0x22)
            .wrapping_add_signed(i16::from(self.r8(0x45) as i8))
            & 0x1f8)
            >> 3;
        let probes = [(0x41u8, 0u16), (0x40, 1), (1, 64), (0, 1)];
        let mut found = None;
        for (marker, delta) in probes {
            position = position.wrapping_add(delta);
            let attribute = self.r8(0x12000 + usize::from(position)) & 0xfc;
            if attribute == 0x6c || attribute & 0xf0 == 0xf0 {
                found = Some((marker, attribute));
                break;
            }
        }
        let Some((marker, attribute)) = found else {
            return;
        };
        let address;
        if attribute == 0x6c {
            if marker & 0x40 != 0
                && self.r8(0x12000 + usize::from(position.wrapping_sub(64))) & 0xfc != 0x6c
            {
                position = position.wrapping_add(64);
            }
            if marker & 1 != 0
                && self.r8(0x12000 + usize::from(position.wrapping_sub(1))) & 0xfc != 0x6c
            {
                position = position.wrapping_add(1);
            }
            let kind = self.r8(0x12000 + usize::from(position)) & 3;
            self.write_attr2(usize::from(position), 0x202);
            self.write_attr2(usize::from(position) + xy(0, 1), 0x202);
            let tiles: [u16; 4] = [0x7ea, 0x80a, 0x80a, 0x82a];
            let source = self
                .hooks
                .dungeon_object_data(usize::from(tiles[usize::from(kind)]) / 2)
                .to_vec();
            address = position.wrapping_sub(xy(1, 1) as u16).wrapping_mul(2);
            self.room_draw_object_nx4(4, &source, DUNG_BG2 + usize::from(address));
        } else {
            self.w16(0x68e, position);
            let door = usize::from(attribute & 0xf);
            if self.r16(0x1980 + door * 2) as u8 != 0x28 {
                return;
            }
            self.w8(0x12f, 27);
            address = self.r16(0x19a0 + door * 2);
            self.w16(0x68c, self.r16(0x68c) | UPPER_BITMASKS[door]);
            self.w16(0x400, self.r16(0x400) | UPPER_BITMASKS[door]);
            self.w16(0x692, 0);
            self.w16(0x460, (door as u16) * 2);
            self.w16(0x694, (door as u16) * 2);
            let source = self.object_data::<16>(DOOR_TYPE_SRC_DATA[0x56 / 2] as usize);
            self.room_draw_object_nx4(4, &source, DUNG_BG2 + usize::from(address));
            self.dungeon_load_toggle_door_attr_other_entry(door);
        }
        self.hooks
            .dungeon_prep_overlay_dma_next(&mut self.ram, 0, address);
        self.w8(
            0x12e,
            30 | self
                .hooks
                .dungeon_calculate_sfx_pan(&self.ram, (address & 0x7f) * 2),
        );
        self.w8(0x14, 1);
    }

    fn open_key_door(&mut self, door: usize, position: u16) {
        self.w16(0x690, 0);
        self.w16(0x68e, position);
        self.w8(0x11, 4);
        self.w8(
            0x12f,
            20 | [0, 0, 0x80, 0x40][usize::from(self.r16(0x19c0 + door * 2) & 3)],
        );
    }

    pub fn bomb_check_for_destructibles(&mut self, x: u16, y: u16, debris: u8) {
        if self.r8(0x10) != 7 {
            self.hooks.overworld_bomb_tiles_32x32(&mut self.ram, x, y);
            return;
        }
        let mut position = ((y & 0x1f8) << 3 | (x & 0x1f8) >> 3).wrapping_sub(0x82);
        for _ in 0..3 {
            for probe in 0..3 {
                let attribute = self.r8(0x12000 + usize::from(position));
                if attribute == 0x62 {
                    if self.r16(0xa0) == 0x65 {
                        self.w16(0x402, self.r16(0x402) | 0x1000);
                    }
                    let mut point = (0, 0);
                    self.thieves_attic_draw_lightened_hole(0, 0, &mut point);
                    self.w8(0x12f, 0x1b);
                    return;
                }
                if attribute & 0xf0 == 0xf0 {
                    let door = usize::from(attribute & 0xf);
                    let kind = self.r16(0x1980 + door * 2) as u8 & 0xfe;
                    if !matches!(kind, 0x42 | 0x2a | 0x2e) {
                        return;
                    }
                    self.w16(0x68e, position);
                    let address = self.r16(0x19a0 + door * 2);
                    let debris = usize::from(debris);
                    self.w16(
                        0x728 + debris * 2,
                        ((address & 0x7e) << 2).wrapping_add(self.r16(0x62c)),
                    );
                    self.w16(
                        0x732 + debris * 2,
                        ((address & 0x1f80) >> 4).wrapping_add(self.r16(0x62e)),
                    );
                    self.w8(0x73c + debris, self.r16(0x19c0 + door * 2) as u8 & 3);
                    self.w8(0x12f, 0x1b);
                    self.w8(0x11, 9);
                    return;
                }
                if probe != 2 {
                    position = position.wrapping_add(2);
                }
            }
            position = position.wrapping_add(0x7c);
        }
    }

    pub fn spiral_stairs_make_nearby_walls_high_priority_entering(&mut self) {
        let index = usize::from(self.r8(0x462) & 3);
        let position = usize::from(self.r16(0x6b0 + index * 2)).wrapping_sub(4);
        self.w16(0x48c, (position as u16).wrapping_mul(2));
        self.spiral_stairs_set_nearby_wall_priority(position, true);
    }

    pub fn spiral_stairs_make_nearby_walls_low_priority(&mut self) {
        let position = usize::from(self.r16(0x48c) >> 1);
        self.spiral_stairs_set_nearby_wall_priority(position, false);
    }

    fn spiral_stairs_set_nearby_wall_priority(&mut self, position: usize, high: bool) {
        for column in 0..5 {
            for row in 0..4 {
                let address = DUNG_BG2 + (position + xy(column, row)) * 2;
                let tile = self.r16(address);
                self.w16(address, if high { tile | 0x2000 } else { tile & !0x2000 });
            }
        }
        let dma = self.hooks.dungeon_prep_overlay_dma_next(
            &mut self.ram,
            0,
            (position as u16).wrapping_mul(2),
        );
        self.hooks.dungeon_prep_overlay_dma_next(
            &mut self.ram,
            dma,
            (position as u16).wrapping_mul(2).wrapping_add(8),
        );
        self.w8(0x18, 1);
    }

    pub fn dungeon_draw_room_overlay(&mut self, source: &[u8]) {
        let mut source = source;
        loop {
            assert!(source.len() >= 2);
            self.w16(0xb2, 0);
            self.w16(0xb4, 0);
            if u16::from_le_bytes([source[0], source[1]]) == 0xffff {
                return;
            }
            assert!(source.len() >= 3);
            let position = usize::from(source[0] >> 2) | (usize::from(source[1] >> 2) << 6);
            if source[2] == 0xa4 {
                let middle = self.object_data::<1>(0x5aa)[0];
                let top = self.object_data::<2>(0x63c)[1];
                let bottom = self.object_data::<2>(0x642)[1];
                for column in 0..4 {
                    self.set_bg2(position + xy(column, 0), top);
                    self.set_bg2(position + xy(column, 1), middle);
                    self.set_bg2(position + xy(column, 2), middle);
                    self.set_bg2(position + xy(column, 3), bottom);
                }
            } else {
                let filler = self.object_data::<6>(usize::from(self.r16(0x46a)));
                for row in 0..4 {
                    for column in 0..4 {
                        let tile = filler[(row & 1) * 4 + (column & 1)];
                        self.set_bg2(position + xy(column, row), tile);
                    }
                }
            }
            source = &source[3..];
        }
    }

    pub fn clear_and_stripe_exploding_wall(&mut self, mut destination: u16) {
        let mut mode = 0x80u16;
        let mut second_segment = false;
        let mut columns = self.r16(0x454).wrapping_add(3);
        let mut table_index = 0u16;
        if (columns.wrapping_sub(8) as i16) >= 0 {
            table_index = columns.wrapping_sub(6);
            second_segment = true;
            columns = 3;
        }
        let door = usize::from(self.r16(0x460) >> 1);
        if self.r16(0x19c0 + door * 2) & 2 == 0 {
            mode = mode.wrapping_add(1);
        }

        let mut upload = 0x1100;
        loop {
            for _ in 0..columns {
                let word_position = destination;
                self.w16(upload, Self::dungeon_map_vram_addr_no_swap(word_position));
                self.w16(upload + 2, mode | 0xa00);
                for row in 0..5 {
                    self.w16(
                        upload + 4 + row * 2,
                        self.r16(DUNG_BG2 + (usize::from(word_position) + xy(0, row)) * 2),
                    );
                }
                self.w16(
                    upload + 14,
                    Self::dungeon_map_vram_addr_no_swap(word_position).wrapping_add(0x4a0),
                );
                self.w16(upload + 16, mode | 0xe00);
                for row in 5..12 {
                    self.w16(
                        upload + 18 + (row - 5) * 2,
                        self.r16(DUNG_BG2 + (usize::from(word_position) + xy(0, row)) * 2),
                    );
                }
                destination = destination.wrapping_add(1);
                upload += 32;
            }
            if !second_segment {
                break;
            }
            second_segment = false;
            let index = usize::from(table_index >> 1) + if mode & 1 != 0 { 0 } else { 8 } - 1;
            destination = destination.wrapping_add(BLAST_WALL_TAB2[index] >> 1);
            columns = 3;
        }
        self.w16(upload, 0xffff);
    }

    pub fn dungeon_flood_swamp_water_prep_tile_map(&mut self) {
        self.water_flood_build_one_quadrant_for_vram();
        self.w8(0x45c, self.r8(0x45c).wrapping_add(4));
        let state = self.r8(0xb0).wrapping_add(1);
        self.w8(0xb0, state);
        if state == 6 {
            self.w8(0x45c, 0);
            self.w8(0xb0, 0);
            self.w8(0x11, 0);
        }
    }

    pub fn flood_dam_prep_tiles_init(&mut self) {
        self.w8(0x45c, 0);
        self.w8(0x418, 0);
        self.water_flood_build_one_quadrant_for_vram();
        self.w8(0x45c, self.r8(0x45c).wrapping_add(4));
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn watergate_main_state1(&mut self) {
        self.w8(0x418, 0);
        self.water_flood_build_one_quadrant_for_vram();
        self.w8(0x45c, self.r8(0x45c).wrapping_add(4));
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn reset_then_cache_room_entry_properties(&mut self) {
        self.w8(0x200, 0);
        self.w8(0xb0, 0);
        self.w8(0x418, 0);
        self.w8(0x11, 0);
        self.w8(0x642, 0);
        self.w8(0x641, 0);
        self.hooks.cache_camera_properties(&mut self.ram);
    }

    pub fn dungeon_transition_trigger_bgc34_update_and_advance(&mut self) {
        self.hooks.prep_trans_aux_gfx(&mut self.ram);
        self.w8(0x17, 9);
        self.w8(0x710, 9);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn dungeon_transition_trigger_bgc56_update_and_advance(&mut self) {
        self.w8(0x17, 10);
        self.w8(0x710, 10);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module_pre_dungeon_set_ambient_sfx(&mut self) {
        if self.r8(0xf3c5) < 2 {
            self.w8(0x12d, 5);
            if self.r8(0xa4) & 0x80 == 0 && self.r16(0xa0) != 2 && self.r16(0xa0) != 18 {
                self.w8(0x12d, 3);
            }
        }
    }

    pub fn module_pre_dungeon(&mut self) {
        self.w8(0x12d, 5);
        self.w8(0x12e, 0);
        self.w16(0xa0, 0);
        self.w16(0xa2, 0);
        self.w16(0x402, 0);
        self.ram.slice_mut(0xf6c8, 6).fill(0);
        self.dungeon_load_entrance();
        let palace = self.r16(0x40c) as u8;
        let keys = if palace == 0xff {
            0xff
        } else {
            let slot = if palace == 2 { 0 } else { palace >> 1 };
            self.r8(0xf37c + usize::from(slot))
        };
        self.w8(0xf36f, keys);
        self.hooks.hud_rebuild(&mut self.ram);
        self.w8(0x45a, 0);
        self.w8(0x458, 0);
        self.dungeon_load_and_draw_room();
        self.dungeon_load_custom_tile_attr();
        let theme = usize::from(self.r8(0xaa2));
        self.hooks
            .decompress_animated_dungeon_tiles(&mut self.ram, DUNG_ANIMATED_TILES[theme]);
        self.dungeon_load_attribute_table();
        self.w8(0xaa4, 10);
        self.hooks.initialize_tilesets(&mut self.ram);
        self.w8(0xab8, 10);
        self.hooks.dungeon_load_palettes(&mut self.ram);
        if self.r8(0x56) | self.r8(0x5d) != 0 {
            self.hooks.load_gear_palettes_bunny(&mut self.ram);
        }
        self.w16(0x62c, (self.r16(0xa0) & 0xf) << 9);
        self.w16(0x62e, ((self.r16(0xa0) & 0xff0) >> 3).swap_bytes());
        if self.r16(0xa0) == 0x104 && self.r8(0xf3c6) & 0x10 != 0 {
            self.w16(0xc005, 0);
        }
        self.set_and_save_visited_quadrant_flags();
        self.w8(0x99, 2);
        self.w8(0x9a, 0xb3);
        let mut lit = self.r8(0x45a);
        if self.r8(0xc005) == 0 {
            lit = 3;
            self.w8(
                0x9a,
                match self.r8(0x414) {
                    7 => 0x32,
                    4 => 0x62,
                    _ => 0x20,
                },
            );
        }
        self.w8(0xc017, LIT_TORCHES_COLOR_PLUS[usize::from(lit)]);
        self.dungeon_approach_fixed_color_variable(self.r8(0xc017));
        self.w8(0xc007, 0x1f);
        self.w8(0xc00b, 0);
        self.w8(0xc009, 2);
        self.w8(0x15, 0);
        self.w8(0x57, 0);
        self.w8(0x3a, 0);
        self.w8(0x3c, 0);
        self.dungeon_reset_torch_background_and_player();
        self.link_check_bunny_status();
        self.reset_then_cache_room_entry_properties();
        if self.r8(0xf3cc) == 13 {
            self.w8(0xf3cc, 0);
            self.w8(0xf3cd, 0);
            self.hooks.hud_remove_super_bomb_indicator(&mut self.ram);
        }
        self.w8(0x94, 9);
        self.hooks.follower_initialize(&mut self.ram);
        self.hooks.dungeon_reset_sprites(&mut self.ram);
        self.w8(0x2f0, 0);
        self.w8(0xaaa, self.r8(0xaaa).wrapping_add(1));
        if self.r8(0xf3c5) == 0 && self.r8(0xf3c6) & 0x10 == 0 {
            self.w8(0x9c, 0x30);
            self.w8(0x9d, 0x50);
            self.w8(0x9e, 0x80);
            self.w16(0xc005, 0);
            self.w8(0xc006, 0);
            self.hooks.link_tuck_into_bed(&mut self.ram);
        }
        self.w8(0x10c, 7);
        self.w8(0x10, 7);
        self.w8(0x11, 15);
        self.dungeon_load_song_bank_if_needed();
        self.module_pre_dungeon_set_ambient_sfx();
    }

    pub fn module07_dungeon(&mut self) {
        self.dungeon_handle_layer_effect();
        match self.r8(0x11) {
            0 => self.module07_00_player_control(),
            1 => self.module07_01_subtile_transition(),
            2 => self.module07_02_supertile_transition(),
            3 => self.module07_03_overlay_change(),
            4 => self.module07_04_unlock_door(),
            5 => self.module07_05_control_shutters(),
            6 => self.module07_06_fat_inter_room_stairs(),
            7 => self.module07_07_falling_transition(),
            8 => self.module07_08_north_intra_room_stairs(),
            9 => self.module07_09_open_cracked_door(),
            10 => self.module07_0a_change_brightness(),
            11 => self.module07_0b_drain_swamp_pool(),
            12 => self.module07_0c_flood_swamp_water(),
            13 => self.module07_0d_flood_dam(),
            14 => self.module07_0e_spiral_stairs(),
            15 => self.module07_0f_landing_wipe(),
            16 => self.module07_10_south_intra_room_stairs(),
            17..=19 => self.module07_11_straight_interroom_stairs(),
            20 => self.module07_14_recover_from_fall(),
            21 => self.module07_15_warp_pad(),
            22 => self.module07_16_update_pegs(),
            23 => self.module07_17_pressure_plate(),
            24 => self.module07_18_rescued_maiden(),
            25 => self.module07_19_mirror_fade(),
            26 => self.module07_1a_room_draw_open_triforce_door_bounce(),
            state => panic!("invalid dungeon submodule {state}"),
        }
        if !self.hooks.misc_bug_fixes_enabled() || self.r8(0x10) == 7 {
            self.w16(0x480, 0);
            self.dungeon_push_block_handler();
            if self.r8(0x11) == 0 {
                self.hooks.graphics_load_chr_half_slot(&mut self.ram);
                self.dungeon_handle_camera();
                if self.r8(0x11) == 0 {
                    self.dungeon_handle_room_tags();
                    if self.r8(0x11) == 0 {
                        self.dungeon_process_torches_and_doors();
                        if self.r16(0x422) != 0 {
                            self.dungeon_clear_away_exploding_wall();
                        }
                        if self.r8(0x6c) == 0 {
                            self.dungeon_try_screen_edge_transition();
                        }
                    }
                }
            }
        }
        self.hooks.orient_lamp_light_cone(&mut self.ram);
        let bg2x = self.r16(0xe2);
        let bg2y = self.r16(0xe8);
        let bg1x = self.r16(0xe0);
        let bg1y = self.r16(0xe6);
        self.w16(0xe2, bg2x.wrapping_add(self.r16(0x11c)));
        self.w16(0xe8, bg2y.wrapping_add(self.r16(0x11a)));
        self.w16(0xe0, bg1x.wrapping_add(self.r16(0x11c)));
        self.w16(0xe6, bg1y.wrapping_add(self.r16(0x11a)));
        if self.r8(0xc208) != 0 {
            self.w16(0xe0, self.r16(0xe2).wrapping_add(self.r16(0x310)));
            self.w16(0xe6, self.r16(0xe8).wrapping_add(self.r16(0x312)));
        }
        self.sprite_dungeon_draw_all_push_blocks();
        self.hooks.sprite_main(&mut self.ram);
        self.w16(0xe2, bg2x);
        self.w16(0xe8, bg2y);
        self.w16(0xe0, bg1x);
        self.w16(0xe6, bg1y);
        self.hooks.link_oam_main(&mut self.ram);
        self.hooks.hud_refill_logic(&mut self.ram);
        self.hooks.hud_floor_indicator(&mut self.ram);
    }

    pub fn load_ow_music_if_needed(&mut self) {
        if self.r8(0x136) != 0 {
            self.w8(0x136, 0);
            self.hooks.load_overworld_songs(&mut self.ram);
        }
    }

    pub fn dungeon_load_song_bank_if_needed(&mut self) {
        let music = self.r8(0x132);
        if music == 0xff || music == 0xf2 {
            return;
        }
        if matches!(music, 3 | 7 | 14) {
            self.load_ow_music_if_needed();
        } else if self.r8(0x136) == 0 {
            self.w8(0x136, 1);
            self.hooks.load_dungeon_songs(&mut self.ram);
        }
    }

    pub fn dungeon_transition_subtile_prep_transition(&mut self) {
        self.w16(0xc009, 0);
        self.w16(0xc007, 0);
        self.w8(0xc00b, 31);
        self.w16(0xaa6, 0);
        self.w8(0x646, 0);
        self.w8(0x642, 0);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn dungeon_transition_subtile_apply_filter(&mut self) {
        if self.r8(0xc005) == 0 {
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        } else {
            self.hooks.apply_palette_filter_bounce(&mut self.ram);
            if self.r8(0xc007) != 0 {
                self.hooks.apply_palette_filter_bounce(&mut self.ram);
            }
        }
    }

    pub fn dungeon_transition_subtile_reset_shutters(&mut self) {
        self.w8(0x468, 0);
        self.w8(0x690, 7);
        let submodule = self.r8(0x11);
        self.operate_shutter_doors();
        self.w8(0x11, submodule);
        self.w8(0xc005, 31);
        self.w8(0xc00b, 0);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_01_subtile_transition(&mut self) {
        self.w16(0xfc4, self.r16(0x20));
        self.w16(0xfc2, self.r16(0x22));
        self.hooks
            .link_handle_moving_animation_full_long_entry(&mut self.ram);
        match self.r8(0xb0) {
            0 => self.dungeon_transition_subtile_prep_transition(),
            1 | 6 => self.dungeon_transition_subtile_apply_filter(),
            2 => self.dungeon_transition_subtile_reset_shutters(),
            3 => self.dungeon_transition_scroll_room(),
            4 => self.dungeon_transition_find_subtile_landing(),
            5 => self.dungeon_intra_room_trans_state5(),
            7 => self.dungeon_transition_subtile_trigger_shutters(),
            _ => panic!("invalid subtile-transition state"),
        }
    }

    pub fn module07_00_player_control(&mut self) {
        if self.r8(0x112) | self.r8(0x2e4) | self.r8(0xffc) == 0 {
            let mut handled = false;
            if self.r8(0xf4) & 0x10 != 0 {
                self.w8(0x200, 0);
                self.w8(0x11, 1);
                self.w8(0x10c, self.r8(0x10));
                self.w8(0x10, 14);
                return;
            } else if self.hooks.did_press_button_for_map(&self.ram) {
                handled = self.try_open_dungeon_map();
            } else if self.r8(0xf0) & 0x20 != 0 {
                handled = self.try_open_select_menu();
            }
            if handled {
                return;
            }
            self.hooks.hud_handle_item_switch_inputs(&mut self.ram);
        }
        self.hooks.link_main(&mut self.ram);
    }

    fn try_open_dungeon_map(&mut self) -> bool {
        if self.r16(0x40c) as u8 == 0xff || self.r16(0xa0) as u8 == 0 {
            return false;
        }
        self.w8(0x200, 0);
        self.w8(0x11, 3);
        self.w8(0x10c, self.r8(0x10));
        self.w8(0x10, 14);
        true
    }

    fn try_open_select_menu(&mut self) -> bool {
        if self.r8(0xf3c5) == 0 {
            return false;
        }
        self.w8(0x200, 0);
        self.hooks.display_select_menu(&mut self.ram);
        true
    }

    pub fn dungeon_transition_subtile_trigger_shutters(&mut self) {
        self.reset_then_cache_room_entry_properties();
        if self.r8(0x468) == 0 {
            self.w8(0x468, 1);
            self.w8(0x68e, 0);
            self.w8(0x690, 0);
            self.w8(0x11, 5);
        }
    }

    pub fn dungeon_inter_room_trans_state12(&mut self) {
        if self.r8(0x11) == 2 {
            if self.r8(0x200) != 5 {
                return;
            }
            self.subtile_transition_calculate_landing();
            if self.r8(0xc005) | self.r8(0xc006) != 0 {
                self.hooks.apply_palette_filter_bounce(&mut self.ram);
            }
        }
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        self.dungeon_reset_torch_background_and_player();
    }

    pub fn dungeon_staircase14(&mut self) {
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        self.dungeon_reset_torch_background_and_player();
    }

    pub fn dungeon_reset_torch_background_and_player(&mut self) {
        let mut sub = SPIRAL_TAB1[usize::from(self.r8(0x414))] as u8;
        let mut main = 0x16;
        if sub & 0x80 != 0 {
            main = 0x17;
            sub = 0;
        }
        if self.r8(0x414) == 2 {
            sub = 3;
        }
        self.w8(0x1c, main);
        self.w8(0x1d, sub);
        self.hooks.hud_restore_torch_background(&mut self.ram);
        self.dungeon_reset_torch_background_and_player_inner();
    }

    pub fn dungeon_reset_torch_background_and_player_inner(&mut self) {
        self.hooks
            .ancilla_terminate_select_interactives(&mut self.ram, 0);
        if self.r8(0x372) != 0 && !self.hooks.turn_while_dashing_enabled() {
            self.w8(0x4d, 0);
            self.w8(0x46, 0);
            self.w8(0x29, 0xff);
            self.w8(0xc7, 0xff);
            self.w8(0x3d, 0);
            self.w8(0x5e, 0);
            self.w8(0x32b, 0);
            self.w8(0x372, 0);
            self.w8(0x5d, 0);
        }
    }

    pub fn dungeon_transition_run_filtering(&mut self) {
        if self.r8(0xc005) | self.r8(0xc006) != 0 {
            let torch_index = if self.r8(0xc005) != 0 {
                self.r8(0x45a)
            } else {
                3
            };
            self.w8(0xc017, LIT_TORCHES_COLOR_PLUS[usize::from(torch_index)]);
            self.dungeon_approach_fixed_color_variable(self.r8(0xc017));
            self.w8(0xc00b, 0);
        }
        self.hooks
            .dungeon_handle_translucency_and_palette(&mut self.ram);
    }

    pub fn dungeon_inter_room_trans_state7(&mut self) {
        self.w16(0xe0, self.r16(0xe2));
        self.w16(0xe6, self.r16(0xe8));
        let room = self.r16(0xa0);
        if room != 54 && room != 56 {
            let layers = if SPIRAL_TAB1[usize::from(self.r8(0x414))] != 0 {
                0x116
            } else {
                0x16
            };
            let current = u16::from(self.r8(0x1c)) | (u16::from(self.r8(0x1d)) << 8);
            if layers != current && (self.r8(0x1c) == 0x17 || self.r8(0x1c) | self.r8(0x1d) != 0x17)
            {
                self.w8(0x1c, layers as u8);
                self.w8(0x1d, (layers >> 8) as u8);
            }
        }
        self.dungeon_transition_run_filtering();
    }

    pub fn dungeon_inter_room_trans_state15(&mut self) {
        self.reset_then_cache_room_entry_properties();
        if self.r8(0x468) == 0 && (self.r8(0xa0) != 172 || self.r16(0x402) & 0x3000 != 0) {
            self.w8(0x468, 1);
            self.w8(0x68e, 0);
            self.w8(0x690, 0);
            self.w8(0x11, 5);
        }
        self.dungeon_play_music_if_defeated();
    }

    pub fn module07_02_faded_filter(&mut self) {
        if self.r8(0xc005) | self.r8(0xc006) != 0 {
            self.hooks.apply_palette_filter_bounce(&mut self.ram);
            if self.r8(0xc007) != 0 {
                self.hooks.apply_palette_filter_bounce(&mut self.ram);
            }
        } else {
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        }
    }

    pub fn dungeon_inter_room_trans_state10(&mut self) {
        if self.r8(0xc005) | self.r8(0xc006) != 0 {
            self.hooks.apply_palette_filter_bounce(&mut self.ram);
        }
        self.dungeon_inter_room_trans_not_dark_room();
    }

    pub fn dungeon_inter_room_trans_state3(&mut self) {
        if self.r8(0xc005) | self.r8(0xc006) != 0 {
            self.w8(0x1d, 0);
        }
        self.dungeon_adjust_for_room_layout();
        self.hooks.load_new_sprite_gfx_set(&mut self.ram);
        self.hooks.mirror_bg1_bg2_offsets(&mut self.ram);
        self.water_flood_build_one_quadrant_for_vram();
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_02_00_initialize_transition(&mut self) {
        let dark = self.r8(0x458);
        self.reset_transition_props_and_advance_submodule();
        self.w8(0x458, dark);
    }

    pub fn module07_02_supertile_transition(&mut self) {
        self.w16(0xfc4, self.r16(0x20));
        self.w16(0xfc2, self.r16(0x22));
        let state = self.r8(0xb0);
        if state != 0 {
            if state >= 7 {
                self.hooks.graphics_incremental_vram_upload(&mut self.ram);
            }
            self.dungeon_load_attribute_selectable();
        }
        self.hooks
            .link_handle_moving_animation_full_long_entry(&mut self.ram);
        match state {
            0 => self.module07_02_00_initialize_transition(),
            1 => self.module07_02_01_load_next_room(),
            2 | 14 => self.module07_02_faded_filter(),
            3 => self.dungeon_inter_room_trans_state3(),
            4 | 6 => self.dungeon_inter_room_trans_state4(),
            5 => self.dungeon_inter_room_trans_not_dark_room(),
            7 => self.dungeon_inter_room_trans_state7(),
            8 => self.dungeon_transition_scroll_room(),
            9 | 11 => self.dungeon_inter_room_trans_state9(),
            10 => self.dungeon_inter_room_trans_state10(),
            12 => self.dungeon_inter_room_trans_state12(),
            13 => self.dungeon_inter_room_trans_state13(),
            15 => self.dungeon_inter_room_trans_state15(),
            _ => panic!("invalid inter-room transition state"),
        }
    }

    pub fn module07_02_01_load_next_room(&mut self) {
        self.dungeon_load_room();
        self.hooks.dungeon_restore_star_tile_chr(&mut self.ram);
        self.hooks.dungeon_transition_aux_gfx_sprite(&mut self.ram);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        self.w8(0x200, 0);
        self.w8(0x48e, self.r8(0xa0));
        self.hooks.dungeon_reset_sprites(&mut self.ram);
        if self.r8(0x458) == 0 {
            self.hooks.mirror_bg1_bg2_offsets(&mut self.ram);
        }
        self.w8(0x458, 0);
    }

    pub fn dungeon_initialize_room_from_special(&mut self) {
        self.dungeon_adjust_after_spiral_stairs();
        self.dungeon_load_room();
        self.hooks.dungeon_restore_star_tile_chr(&mut self.ram);
        self.hooks.dungeon_transition_aux_gfx(&mut self.ram);
        self.dungeon_load_custom_tile_attr();
        self.w8(0x48e, self.r8(0xa0));
        self.hooks.follower_initialize(&mut self.ram);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_06_fat_inter_room_stairs(&mut self) {
        let state = self.r8(0xb0);
        if state >= 3 {
            self.dungeon_load_attribute_selectable();
        }
        if state >= 13 {
            self.hooks.graphics_incremental_vram_upload(&mut self.ram);
            let timer = self.r8(0x464);
            if timer != 0 {
                self.w8(0x464, timer.wrapping_sub(1));
                if timer == 0x10 {
                    self.w8(0x57, 2);
                }
                self.w8(0x67, if self.r8(0x462) & 4 != 0 { 4 } else { 8 });
                self.hooks.link_handle_velocity(&mut self.ram);
                self.dungeon_handle_camera();
            }
        }
        self.hooks
            .link_handle_moving_animation_full_long_entry(&mut self.ram);
        match state {
            0 => self.reset_transition_props_and_advance_reset_interface(),
            1 => {
                self.hooks.apply_palette_filter_bounce(&mut self.ram);
                if self.r8(0xc007) != 0 {
                    self.hooks.apply_palette_filter_bounce(&mut self.ram);
                }
            }
            2 => self.dungeon_initialize_room_from_special(),
            3 => self.dungeon_transition_trigger_bgc34_update_and_advance(),
            4 => self.dungeon_transition_trigger_bgc56_update_and_advance(),
            5 => self.dungeon_transition_load_sprite_gfx(),
            6 => self.dungeon_transition_adjust_for_fat_stair_scroll(),
            7 | 9 => self.dungeon_inter_room_trans_state4(),
            8 => self.dungeon_inter_room_trans_not_dark_room(),
            10 | 12 => self.dungeon_spiral_staircase11(),
            11 | 13 => self.dungeon_spiral_staircase12(),
            14 => self.dungeon_double_apply_and_increment_grayscale(),
            15 => self.dungeon_staircase14(),
            16 if self.r8(0xc009) | self.r8(0xc007) == 0 && self.r8(0x200) == 5 => {
                self.reset_then_cache_room_entry_properties()
            }
            16 => {}
            _ => panic!("invalid fat-stair transition state"),
        }
    }

    pub fn module07_07_falling_transition(&mut self) {
        let state = self.r8(0xb0);
        if state >= 6 {
            self.hooks.graphics_incremental_vram_upload(&mut self.ram);
            self.dungeon_load_attribute_selectable();
            self.apply_grayscale_fixed_incremental();
        }
        match state {
            0 => self.module07_07_00_handle_music_and_reset_room(),
            1 => self.hooks.apply_palette_filter_bounce(&mut self.ram),
            2 => self.dungeon_initialize_room_from_special(),
            3 => self.dungeon_transition_trigger_bgc34_update_and_advance(),
            4 => self.dungeon_transition_trigger_bgc56_update_and_advance(),
            5 => self.dungeon_transition_load_sprite_gfx(),
            6 => self.module07_07_06_sync_bg1and2(),
            7 | 9 | 11 | 13 => self.dungeon_inter_room_trans_state4(),
            8 | 10 | 12 => self.dungeon_inter_room_trans_not_dark_room(),
            14 => self.dungeon_staircase14(),
            15 => self.module07_07_0f_falling_fade_in(),
            16 => self.module07_07_10_land_link_from_falling(),
            17 => self.module07_07_11_cache_room_and_set_music(),
            _ => panic!("invalid falling transition state"),
        }
    }

    pub fn dungeon_transition_load_sprite_gfx(&mut self) {
        self.hooks.load_new_sprite_gfx_set(&mut self.ram);
        self.hooks.dungeon_reset_sprites(&mut self.ram);
        self.dungeon_transition_run_filtering();
    }

    pub fn module07_07_06_sync_bg1and2(&mut self) {
        self.hooks.mirror_bg1_bg2_offsets(&mut self.ram);
        self.dungeon_adjust_for_room_layout();
        let mut sub = SPIRAL_TAB1[usize::from(self.r8(0x414))] as u8;
        let mut main = 0x16;
        if sub & 0x80 != 0 {
            main = 0x17;
            sub = 0;
        }
        self.w8(0x1c, main);
        self.w8(0x1d, sub);
        self.water_flood_build_one_quadrant_for_vram();
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_07_11_cache_room_and_set_music(&mut self) {
        if self.r8(0x200) == 5 {
            self.reset_then_cache_room_entry_properties();
            self.dungeon_play_music_if_defeated();
            self.hooks.graphics_load_chr_half_slot(&mut self.ram);
        }
    }

    pub fn module07_07_0f_falling_fade_in(&mut self) {
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        if self.r8(0xc007) != 0 {
            return;
        }
        let high =
            (self.r16(0x20) >> 8) as u8 + u8::from(self.r16(0x20) as u8 >= self.r16(0x51) as u8);
        self.w8(0x52, high);
        self.dungeon_set_boss_music_unorthodox();
        let room = self.r8(0xa0);
        if matches!(room, 0x89 | 0x4f) {
            return;
        }
        if room == 0xa7 {
            self.w8(0x4a0, 0);
            self.w8(0xa4, 1);
            return;
        }
        self.w8(0xa4, self.r8(0xa4).wrapping_sub(1));
        self.dungeon_play_blip_and_cache_quadrant_visits();
    }

    pub fn module07_07_10_land_link_from_falling(&mut self) {
        self.hooks.handle_dungeon_landing_from_pit(&mut self.ram);
        if self.r8(0x11) != 0 {
            return;
        }
        self.w8(0x11, 7);
        self.w8(0xb0, 17);
        self.w8(0xaaa, 1);
        self.hooks.graphics_load_chr_half_slot(&mut self.ram);
    }

    pub fn dungeon_spiral_staircase11(&mut self) {
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        self.water_flood_build_one_quadrant_for_vram();
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn dungeon_inter_room_trans_state9(&mut self) {
        if self.r8(0xc005) | self.r8(0xc006) != 0 {
            self.hooks.apply_palette_filter_bounce(&mut self.ram);
        }
        self.dungeon_inter_room_trans_state4();
    }

    pub fn dungeon_spiral_staircase12(&mut self) {
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        self.dungeon_prepare_next_room_quadrant_upload();
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn dungeon_spiral_staircase17(&mut self) {
        self.spiral_stairs_find_landing_spot();
        let timer = self.r8(0x464).wrapping_sub(1);
        self.w8(0x464, timer);
        if timer == 0 {
            self.w8(0x464, if self.r8(0x462) & 4 != 0 { 10 } else { 24 });
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        }
    }

    pub fn dungeon_spiral_staircase18(&mut self) {
        self.spiral_stairs_find_landing_spot();
        let timer = self.r8(0x464).wrapping_sub(1);
        self.w8(0x464, timer);
        if timer == 0 {
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
            self.w8(0x200, 0);
        }
    }

    pub fn module07_0e_13_set_room_and_layer_and_cache(&mut self) {
        let plane = usize::from(self.r8(0x48a));
        self.w8(0x476, TELEPORT_PIT_LEVEL1[plane] as u8);
        self.w8(0xee, TELEPORT_PIT_LEVEL2[plane] as u8);
        self.w8(0x1c, self.r8(0x1c) | 0x10);
        self.w8(0x1d, self.r8(0x1d) & 0x0f);
        if self.r8(0x462) & 4 == 0 {
            self.spiral_stairs_make_nearby_walls_low_priority();
        }
        self.w8(0x48e, self.r8(0xa0));
        self.reset_then_cache_room_entry_properties();
    }

    pub fn dungeon_sync_backgrounds_from_spiral_stairs(&mut self) {
        if self.r8(0xf3cc) == 6 && self.r8(0xa0) == 100 {
            self.w8(0xf3cc, 0);
        }
        let old_layer = self.r8(0xee);
        let descending = self.r8(0x462) & 4 != 0;
        self.w16(
            0x20,
            self.r16(0x20)
                .wrapping_add(if descending { 48 } else { 0xffd0 }),
        );
        let plane = usize::from(self.r8(0x48a));
        self.w8(0xee, TELEPORT_PIT_LEVEL2[plane] as u8);
        self.spiral_stairs_make_nearby_walls_high_priority_exiting();
        self.w8(0xee, old_layer);
        self.w16(
            0x20,
            self.r16(0x20)
                .wrapping_add(if descending { 0xffd0 } else { 48 }),
        );
        self.w16(0xe0, self.r16(0xe2));
        self.w16(0xe6, self.r16(0xe8));
        self.dungeon_adjust_for_room_layout();
        let mut sub = SPIRAL_TAB1[usize::from(self.r8(0x414))] as u8;
        let mut main = 0x16;
        if sub & 0x80 != 0 {
            main = 0x17;
            sub = 0;
        }
        if self.r8(0x414) == 2 {
            sub = 3;
        }
        self.w8(0x1c, main);
        self.w8(0x1d, sub);
        self.w8(
            0xa4,
            self.r8(0xa4)
                .wrapping_add(if descending { 0xff } else { 1 }),
        );
        self.w8(0x464, 24);
        self.dungeon_play_blip_and_cache_quadrant_visits();
        self.hooks.hud_restore_torch_background(&mut self.ram);
        self.dungeon_inter_room_trans_not_dark_room();
    }

    pub fn dungeon_transition_adjust_for_fat_stair_scroll(&mut self) {
        self.w16(0xe0, self.r16(0xe2));
        self.w16(0xe6, self.r16(0xe8));
        self.dungeon_adjust_for_room_layout();
        let mut sub = SPIRAL_TAB1[usize::from(self.r8(0x414))] as u8;
        let mut main = 0x16;
        if sub & 0x80 != 0 {
            main = 0x17;
            sub = 0;
        }
        self.w8(0x1c, main);
        self.w8(0x1d, sub);
        self.w8(0x57, 1);
        if self.r8(0x462) & 4 != 0 {
            self.w8(0xa4, self.r8(0xa4).wrapping_sub(1));
            self.w8(0x464, 32);
            self.w8(0x12e, 0x19);
        } else {
            self.w8(0xa4, self.r8(0xa4).wrapping_add(1));
            self.w8(0x464, 48);
            self.w8(0x12e, 0x17);
        }
        self.w8(0x12f, 0x24);
        self.dungeon_play_blip_and_cache_quadrant_visits();
        self.dungeon_inter_room_trans_not_dark_room();
    }

    pub fn module07_08_00_init_stairs(&mut self) {
        self.w8(0x351, 0);
        let (timer, sound) = if self.r8(0x2e) & 8 != 0 {
            self.w8(0x476, 0);
            if self.r16(0x44a) as u8 != 2 {
                self.w8(0xee, 0);
            }
            (0x38, 23)
        } else {
            (0x3c, 25)
        };
        self.w8(0x464, timer);
        self.w8(0x12e, sound);
        self.w8(0x57, 1);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_08_01_climb_stairs(&mut self) {
        if self.r8(0x464) != 0 {
            return;
        }
        if self.r8(0x2e) & 4 != 0 {
            self.w8(0x476, 1);
            if self.r16(0x44a) as u8 != 2 {
                self.w8(0xee, 1);
            }
        }
        self.w8(0xb0, 0);
        self.w8(0x200, 0);
        self.w8(0x11, 0);
        self.set_and_save_visited_quadrant_flags();
    }

    pub fn module07_10_00_init_stairs(&mut self) {
        let (timer, sound) = if self.r8(0x2e) & 4 != 0 {
            self.w8(0x476, self.r8(0x476) ^ 1);
            if self.r16(0x44a) as u8 != 2 {
                self.w8(0xee, self.r8(0xee) ^ 1);
            }
            (0x38, 23)
        } else {
            (0x3c, 25)
        };
        self.w8(0x464, timer);
        self.w8(0x12e, sound);
        self.w8(0x57, 1);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_10_01_climb_stairs(&mut self) {
        if self.r8(0x464) != 0 {
            return;
        }
        if self.r8(0x2e) & 8 != 0 {
            self.w8(0x476, self.r8(0x476) ^ 1);
            if self.r16(0x44a) as u8 != 2 {
                self.w8(0xee, self.r8(0xee) ^ 1);
            }
        }
        self.w8(0xb0, 0);
        self.w8(0x200, 0);
        self.w8(0x11, 0);
        self.set_and_save_visited_quadrant_flags();
    }

    pub fn module07_08_north_intra_room_stairs(&mut self) {
        let timer = self.r8(0x464);
        if timer != 0 {
            self.w8(0x464, timer.wrapping_sub(1));
            if timer == 20 {
                self.w8(0x57, 2);
            }
            self.hooks.link_handle_velocity(&mut self.ram);
            self.hooks.apply_links_movement_to_camera(&mut self.ram);
            self.dungeon_handle_camera();
            self.hooks
                .link_handle_moving_animation_full_long_entry(&mut self.ram);
        }
        match self.r8(0xb0) {
            0 => self.module07_08_00_init_stairs(),
            1 => self.module07_08_01_climb_stairs(),
            _ => panic!("invalid north intra-room stair state"),
        }
    }

    pub fn module07_10_south_intra_room_stairs(&mut self) {
        let timer = self.r8(0x464);
        if timer != 0 {
            self.w8(0x464, timer.wrapping_sub(1));
            if timer == 20 {
                self.w8(0x57, 2);
            }
            self.hooks.link_handle_velocity(&mut self.ram);
            self.hooks.apply_links_movement_to_camera(&mut self.ram);
            self.dungeon_handle_camera();
            self.hooks
                .link_handle_moving_animation_full_long_entry(&mut self.ram);
        }
        match self.r8(0xb0) {
            0 => self.module07_10_00_init_stairs(),
            1 => self.module07_10_01_climb_stairs(),
            _ => panic!("invalid south intra-room stair state"),
        }
    }

    pub fn module07_11_0b_prep_destination(&mut self) {
        let mut sub = SPIRAL_TAB1[usize::from(self.r8(0x414))] as u8;
        let mut main = 0x16;
        if sub & 0x80 != 0 {
            main = 0x17;
            sub = 0;
        }
        self.w8(0x1c, main);
        self.w8(0x1d, sub);
        self.w8(0x57, 1);
        let descending = self.r8(0x462) & 4 != 0;
        self.w8(
            0xa4,
            self.r8(0xa4)
                .wrapping_add(if descending { 0xff } else { 1 }),
        );
        self.w8(0x464, if descending { 0x32 } else { 0x3c });
        self.w8(0x12e, if descending { 25 } else { 23 });
        let mut changed_layer = false;
        let direction_adjust = if self.r8(0x11) == 18 { 0xffe0 } else { 32 };
        if self.r8(0xee) != 0 {
            self.w16(0x20, self.r16(0x20).wrapping_add(direction_adjust));
            changed_layer = true;
        }
        let plane = usize::from(self.r8(0x48a));
        self.w8(0x476, TELEPORT_PIT_LEVEL1[plane] as u8);
        self.w8(0xee, TELEPORT_PIT_LEVEL2[plane] as u8);
        if self.r8(0xee) != 0 {
            self.w16(0x20, self.r16(0x20).wrapping_add(direction_adjust));
            changed_layer = true;
        }
        if !changed_layer {
            let adjustment = if self.r8(0x11) == 18 {
                if descending { 0xffe8 } else { 0xfff8 }
            } else {
                12
            };
            self.w16(0x20, self.r16(0x20).wrapping_add(adjustment));
        }
        self.dungeon_play_blip_and_cache_quadrant_visits();
        self.hooks.hud_restore_torch_background(&mut self.ram);
        self.dungeon_inter_room_trans_not_dark_room();
    }

    pub fn module07_0d_flood_dam(&mut self) {
        self.hooks.flood_dam_prep_flood_hdma(&mut self.ram);
        match self.r8(0xb0) {
            0 => self.flood_dam_prep_tiles_init(),
            1..=3 => self.watergate_main_state1(),
            4 => self.flood_dam_expand(),
            5 => self.flood_dam_fill(),
            _ => panic!("invalid flood-dam state"),
        }
    }

    pub fn module07_0f_landing_wipe(&mut self) {
        match self.r8(0xb0) {
            0 => self.module07_0f_00_init_spotlight(),
            1 => self.module07_0f_01_operate_spotlight(),
            _ => panic!("invalid landing-wipe state"),
        }
        self.hooks
            .link_handle_moving_animation_full_long_entry(&mut self.ram);
        self.hooks.link_oam_main(&mut self.ram);
    }

    pub fn module07_18_rescued_maiden(&mut self) {
        match self.r8(0xb0) {
            0 => {
                self.hooks
                    .palette_filter_restore_bg_subtractive_strict(&mut self.ram);
                self.w16(0xc500, self.r16(0xc540));
                if self.r8(0xc007) != 0xff {
                    return;
                }
                for position in 0..0x1000 {
                    self.set_bg1(position, 0x1ec);
                    self.set_bg2(position, 0x1ec);
                }
                self.w16(0x11c, 0);
                self.w16(0x11a, 0);
                self.w16(0x422, 0);
                self.w16(0x424, 0);
                self.w8(0x418, 0);
                self.w8(0x45c, 0);
                self.w8(0xb0, 1);
            }
            1 => {
                self.hooks.palette_filter_crystal(&mut self.ram);
                self.w8(0x1d, 1);
                self.w8(0x2e4, 2);
                let room = self.r16(0xa0);
                let boss = BOSS_ROOMS
                    .iter()
                    .position(|&candidate| candidate == room)
                    .expect("crystal cutscene boss room");
                let mut destination = usize::from(CRYSTAL_TAB0[boss - 4] >> 1);
                let mut tile = 0u16;
                for _ in 0..4 {
                    for column in 0..8 {
                        self.set_bg1(destination + xy(column, 0), 0x1f80 | tile);
                        self.set_bg1(destination + xy(column, 4), 0x1f88 | tile);
                        tile = tile.wrapping_add(1);
                    }
                    tile = tile.wrapping_add(8);
                    destination += xy(0, 1);
                }
                self.w8(0xb0, 2);
            }
            2 | 4 | 6 | 8 => self.dungeon_inter_room_trans_not_dark_room(),
            3 | 5 | 7 | 9 => self.dungeon_inter_room_trans_state4(),
            10 => {
                self.w8(0x12a, self.r8(0x12a).wrapping_add(1));
                self.hooks.polyhedral_initialize_thread(&mut self.ram);
                self.crystal_cutscene_initialize();
                self.w8(0x11, 0);
                self.w8(0xb0, 0);
            }
            _ => panic!("invalid rescued-maiden state"),
        }
    }

    pub fn module07_1a_room_draw_open_triforce_door_bounce(&mut self) {
        self.w8(0x2e4, 1);
        if self.r16(0xc8) != 0 {
            let low = self.r8(0xc8).wrapping_sub(1);
            self.w8(0xc8, low);
            if low != 0 {
                return;
            }
            let high = self.r8(0xc9).wrapping_sub(1);
            self.w8(0xc9, high);
            if high != 0 {
                return;
            }
            self.w8(0x12d, 21);
            self.w8(0x3ef, 0);
            self.w8(0x50, 0);
        }
        self.w8(0x2e4, 0);
        let state = self.r8(0xb0).wrapping_add(1);
        self.w8(0xb0, state);
        if state & 3 != 0 {
            return;
        }
        let source_offset = OPEN_GANON_DOOR_TAB[usize::from(state.wrapping_sub(4) >> 2)];
        let source = self.object_data::<32>(usize::from(source_offset));
        for column in 0..8 {
            for row in 0..4 {
                self.set_bg2(xy(44 + column, 3 + row), source[column * 4 + row]);
            }
        }
        self.hooks
            .dungeon_prep_overlay_dma_watergate(&mut self.ram, 0, 0x1d8, 0x881, 8);
        if state == 16 {
            self.write_attr2(xy(44, 5), 0x202);
            self.write_attr2(xy(44, 6), 0x202);
            self.write_attr2(xy(50, 5), 0x200);
            self.write_attr2(xy(50, 6), 0x200);
            for column in (45..=49).step_by(2) {
                for row in 0..=6 {
                    self.write_attr2(xy(column, row), 0);
                }
            }
            self.w16(0x600, 0xffc0);
            self.w8(0x11, 0);
            self.w8(0xb0, 0);
        }
        self.w8(0x18, 1);
    }

    pub fn reset_transition_props_and_advance_reset_interface(&mut self) {
        self.w8(0x200, 0);
        self.reset_transition_props_and_advance_submodule();
    }

    pub fn module07_0e_01_handle_music_and_reset_props(&mut self) {
        let room = self.r16(0xa0);
        if (room == 7 || room == 23 && !self.hooks.zelda_is_playing_music_track(17))
            && self.r8(0xf374) & 1 == 0
        {
            self.w8(0x12c, 0xf1);
        }
        self.w8(0x464, if self.r8(0x462) & 4 != 0 { 106 } else { 88 });
        self.w8(0x200, 0);
        self.reset_transition_props_and_advance_submodule();
    }

    pub fn reset_transition_props_and_advance_submodule(&mut self) {
        self.w16(0xc011, 0);
        self.w16(0xc009, 0);
        self.w16(0xc007, 0);
        self.w8(0xc00b, 31);
        self.w16(0xaa6, 0);
        self.w8(0x45a, 0);
        if self.r8(0x458) != 0 {
            self.w8(0x99, 2);
            self.w8(0x9a, 0xb3);
        }
        self.w8(0x458, 0);
        self.dungeon_reset_torch_background_and_player_inner();
        self.hooks.overworld_copy_palettes_to_cache(&mut self.ram);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_07_00_handle_music_and_reset_room(&mut self) {
        let room = self.r16(0xa0);
        if room == 0x10 || room == 7 || room == 0x17 {
            self.w8(0x12c, 0xf1);
        }
        self.reset_transition_props_and_advance_reset_interface();
    }

    pub fn dungeon_play_music_if_defeated(&mut self) {
        let room = self.r16(0xa0);
        let music = if room == 18 {
            0x14
        } else if room == 2 {
            0x10
        } else {
            if !BOSS_ROOMS.contains(&room) || self.hooks.sprite_check_if_screen_is_clear(&self.ram)
            {
                return;
            }
            0x15
        };
        self.w8(0x12c, music);
    }

    pub fn module07_03_overlay_change(&mut self) {
        let overlay = self
            .hooks
            .dungeon_room_overlay(usize::from(self.r8(0x4ba)))
            .to_vec();
        self.dungeon_draw_room_overlay(&overlay);
        let mut dma = 0;
        for entry in overlay.chunks_exact(3) {
            if u16::from_le_bytes([entry[0], entry[1]]) == 0xffff {
                break;
            }
            let position = usize::from(entry[0] >> 2) | (usize::from(entry[1] >> 2) << 6);
            dma = self.hooks.dungeon_prep_overlay_dma_next(
                &mut self.ram,
                dma,
                (position as u16).wrapping_mul(2),
            );
            self.dungeon_draw_room_overlay_apply(position);
        }
        self.w8(0x18, 1);
        self.w8(0x11, 0);
    }

    pub fn module07_04_unlock_door(&mut self) {
        self.dungeon_opening_locked_door_combined(false);
    }

    pub fn module07_05_control_shutters(&mut self) {
        self.operate_shutter_doors();
    }

    pub fn module07_09_open_cracked_door(&mut self) {
        self.open_cracked_door();
    }

    pub fn module07_0a_change_brightness(&mut self) {
        self.hooks.orient_lamp_light_cone(&mut self.ram);
        self.apply_grayscale_fixed_incremental();
        if self.r8(0x9c) & 0x1f == self.r8(0xc017) {
            self.w8(0x11, 0);
            self.w8(0xb0, 0);
        }
    }

    pub fn module07_0b_drain_swamp_pool(&mut self) {
        match self.r8(0xb0) {
            0 => {
                let counter = self.r8(0x424);
                if counter & 7 == 0 {
                    let index = usize::from((counter >> 2) & 3);
                    if self.r16(0x684) == self.r16(0x688) {
                        self.dungeon_set_attr_for_activated_water_off();
                        return;
                    }
                    let delta = TURN_OFF_WATER_TAB0[index] as u16;
                    self.w16(0x684, self.r16(0x684).wrapping_add(delta));
                    self.w16(0x686, self.r16(0x686).wrapping_add(delta));
                }
                self.w8(0x424, counter.wrapping_add(1));
                self.hooks.adjust_water_hdma_window(&mut self.ram);
            }
            1 => {
                let tile = self.object_data::<1>(0x1e0)[0];
                for position in 0..0x1000 {
                    self.set_bg1(position, tile);
                }
                self.w8(0x45c, 0);
                self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
            }
            2..=5 => self.dungeon_flood_swamp_water_prep_tile_map(),
            _ => {}
        }
    }

    pub fn module07_0c_flood_swamp_water(&mut self) {
        let state = self.r8(0xb0);
        match state {
            0..=3 => self.dungeon_flood_swamp_water_prep_tile_map(),
            4..=8 => {
                let counter = self.r8(0x424).wrapping_sub(1);
                self.w8(0x424, counter);
                if counter == 0 {
                    self.w8(0x424, 4);
                    let next_state = state.wrapping_add(1);
                    self.w8(0xb0, next_state);
                    let depth = usize::from(next_state - 4);
                    self.w16(0x686, 8);
                    self.w16(0x68a, 0);
                    self.w16(0x684, 0x30);
                    let source = self.hooks.dungeon_object_data((0x1654 + 0x10) / 2);
                    let tiles = core::array::from_fn::<_, 20, _>(|index| source[index]);
                    self.dungeon_adjust_water_vomit(&tiles[..depth * 4], depth);
                }
            }
            9 | 10 => {
                if state == 9 {
                    self.w8(0x96, 3);
                    self.w8(0x97, 0);
                    self.w8(0x98, 0);
                    self.w8(0x1e, 22);
                    self.w8(0x1f, 1);
                    self.w8(0x1d, 1);
                    self.w8(0x99, 2);
                    self.w8(0x9a, 98);
                    self.w8(0x424, 0);
                    self.w8(0xb0, 10);
                }
                let counter = self.r8(0x424);
                let index = usize::from(counter & 3);
                let target = 0x688u16.wrapping_sub(self.r16(0xe8)).wrapping_sub(0x24);
                self.w16(
                    0x686,
                    self.r16(0x686)
                        .wrapping_add(TURN_ON_WATER_TAB0[index] as u16),
                );
                let height = self
                    .r16(0x68a)
                    .wrapping_add(TURN_ON_WATER_TAB1[index] as u16);
                self.w16(0x68a, height);
                if height >= target {
                    self.w8(0x414, 7);
                    self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
                }
                self.w8(0x424, counter.wrapping_add(1));
                let lower = 0x688u16
                    .wrapping_sub(self.r16(0xe8))
                    .wrapping_sub(self.r16(0x684));
                let upper = lower.wrapping_add(height);
                self.w16(0x674, lower);
                self.w16(0x676, upper);
                self.hooks.adjust_water_hdma_window_x(&mut self.ram, upper);
            }
            11 => {
                let counter = self.r8(0x424);
                if counter & 7 == 0 {
                    let index = usize::from((counter >> 2) & 3);
                    if self.r16(0x684) == self.r16(0x688) {
                        self.dungeon_set_attr_for_activated_water();
                        return;
                    }
                    let delta = TURN_ON_WATER_TAB2[index] as u16;
                    self.w16(0x684, self.r16(0x684).wrapping_add(delta));
                    self.w16(0x686, self.r16(0x686).wrapping_add(delta));
                    let difference = self.r16(0x688).wrapping_sub(self.r16(0x684));
                    if difference == 0 || difference == 8 {
                        let offset = if difference == 0 { 0x16b4 } else { 0x168c };
                        let source = self.hooks.dungeon_object_data(offset / 2);
                        let tiles = core::array::from_fn::<_, 20, _>(|index| source[index]);
                        self.dungeon_adjust_water_vomit(&tiles, 5);
                    }
                }
                self.w8(0x424, counter.wrapping_add(1));
                self.hooks.adjust_water_hdma_window(&mut self.ram);
            }
            _ => {}
        }
    }

    pub fn dungeon_double_apply_and_increment_grayscale(&mut self) {
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        self.apply_grayscale_fixed_incremental();
    }

    pub fn dungeon_advance_then_set_boss_music_unorthodox(&mut self) {
        self.dungeon_reset_torch_background_and_player_inner();
        self.w8(0x464, 0x38);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        self.dungeon_set_boss_music_unorthodox();
    }

    pub fn dungeon_set_boss_music_unorthodox(&mut self) {
        let room = self.r16(0xa0);
        let music = if room == 16 {
            0x1c
        } else if room == 7 {
            0x15
        } else {
            if room != 23 || self.hooks.zelda_is_playing_music_track(17) {
                return;
            }
            0x11
        };
        if room != 16 && self.r8(0x130) != 0xf1 && self.r8(0xf374) & 1 != 0 {
            return;
        }
        self.w8(0x12c, music);
    }

    pub fn module07_0e_02_apply_filter_if(&mut self) {
        let staircase = self.r8(0x464);
        if staircase < 9 {
            self.hooks.apply_palette_filter_bounce(&mut self.ram);
            if self.r16(0xc007) != 0 {
                self.hooks.apply_palette_filter_bounce(&mut self.ram);
            }
        }
        if staircase != 0 {
            self.w8(0x464, staircase.wrapping_sub(1));
        } else {
            self.w8(0x2f9, 12);
            self.w8(0x4b, 12);
        }
    }

    pub fn module07_0e_spiral_stairs(&mut self) {
        let state = self.r8(0xb0);
        if state >= 7 {
            self.hooks.graphics_incremental_vram_upload(&mut self.ram);
            self.dungeon_load_attribute_selectable();
        }
        self.handle_link_on_spiral_stairs();
        match state {
            0 => self.module07_0e_00_init_priority_and_screens(),
            1 => self.module07_0e_01_handle_music_and_reset_props(),
            2 => self.module07_0e_02_apply_filter_if(),
            3 => self.dungeon_initialize_room_from_special(),
            4 => self.dungeon_transition_trigger_bgc34_update_and_advance(),
            5 => self.dungeon_transition_trigger_bgc56_update_and_advance(),
            6 => self.dungeon_transition_load_sprite_gfx(),
            7 => self.dungeon_sync_backgrounds_from_spiral_stairs(),
            8 | 10 => self.dungeon_inter_room_trans_state4(),
            9 => self.dungeon_inter_room_trans_not_dark_room(),
            11 | 13 => self.dungeon_spiral_staircase11(),
            12 | 14 => self.dungeon_spiral_staircase12(),
            15 => self.dungeon_double_apply_and_increment_grayscale(),
            16 => self.dungeon_advance_then_set_boss_music_unorthodox(),
            17 => self.dungeon_spiral_staircase17(),
            18 => self.dungeon_spiral_staircase18(),
            19 => self.module07_0e_13_set_room_and_layer_and_cache(),
            _ => panic!("invalid spiral-stair transition state"),
        }
    }

    pub fn module07_0e_00_init_priority_and_screens(&mut self) {
        self.spiral_stairs_make_nearby_walls_high_priority_entering();
        if self.r8(0xee) != 0 {
            self.w8(0x1c, self.r8(0x1c) & 0xf);
            self.w8(0x1d, self.r8(0x1d) | 0x10);
            self.w8(0xee, 3);
        }
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_0f_00_init_spotlight(&mut self) {
        self.hooks.spotlight_open(&mut self.ram);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_0f_01_operate_spotlight(&mut self) {
        self.hooks.sprite_main(&mut self.ram);
        self.hooks.iris_spotlight_configure_table(&mut self.ram);
        if self.r8(0x11) == 0 {
            self.w8(0x96, 0);
            self.w8(0x97, 0);
            self.w8(0x98, 0);
            self.w8(0x1e, 0);
            self.w8(0x1f, 0);
            self.w8(0xb0, 0);
        }
    }

    pub fn spiral_stairs_make_nearby_walls_high_priority_exiting(&mut self) {
        if self.r8(0x462) & 4 != 0 {
            return;
        }
        let target = self.r16(0x48c).wrapping_add(8) & 0x7f;
        let staircase = (0..16)
            .map(|index| self.r16(0x6b0 + index * 2))
            .find(|&position| position.wrapping_mul(2) & 0x7f == target)
            .expect("matching spiral staircase");
        let position = usize::from(staircase.wrapping_sub(4));
        self.w16(0x48c, (position as u16).wrapping_mul(2));
        for column in 0..5 {
            for row in 0..4 {
                let address = DUNG_BG2 + (position + xy(column, row)) * 2;
                self.w16(address, self.r16(address) | 0x2000);
            }
        }
    }

    pub fn adjust_quadrant_and_camera_right(&mut self) {
        self.w8(0xa9, self.r8(0xa9) ^ 1);
        self.dungeon_adjust_quadrant();
        self.adjust_room_bounds_a(0x608, 0x100);
        self.set_and_save_visited_quadrant_flags();
    }

    pub fn adjust_quadrant_and_camera_left(&mut self) {
        self.w8(0xa9, self.r8(0xa9) ^ 1);
        self.dungeon_adjust_quadrant();
        self.adjust_room_bounds_a(0x608, 0xff00);
        self.set_and_save_visited_quadrant_flags();
    }

    pub fn adjust_quadrant_and_camera_down(&mut self) {
        self.w8(0xaa, self.r8(0xaa) ^ 2);
        self.dungeon_adjust_quadrant();
        self.adjust_room_bounds_a(0x600, 0x100);
        self.set_and_save_visited_quadrant_flags();
    }

    pub fn adjust_quadrant_and_camera_up(&mut self) {
        self.w8(0xaa, self.r8(0xaa) ^ 2);
        self.dungeon_adjust_quadrant();
        self.adjust_room_bounds_a(0x600, 0xff00);
        self.set_and_save_visited_quadrant_flags();
    }

    fn adjust_room_bounds_a(&mut self, base: usize, delta: u16) {
        self.w16(base, self.r16(base).wrapping_add(delta));
        self.w16(base + 4, self.r16(base + 4).wrapping_add(delta));
    }

    fn adjust_room_bounds_b(&mut self, base: usize, delta: u16) {
        self.w16(base + 2, self.r16(base + 2).wrapping_add(delta));
        self.w16(base + 6, self.r16(base + 6).wrapping_add(delta));
    }

    fn finish_room_edge_plane_change(&mut self) {
        if self.r8(0xef) & 1 != 0 {
            self.w8(0xee, self.r8(0xee) ^ 1);
            self.w8(0x476, self.r8(0xee));
        }
        if self.r8(0xef) & 2 != 0 {
            self.w16(0x40c, self.r16(0x40c) ^ 2);
        }
    }

    fn update_fullsize_x(&mut self) {
        let flag = LAYOUT_QUADRANT_FLAGS[usize::from(self.r8(0xa8))];
        self.w8(
            0xa6,
            if self.r8(0x452) != 0 || flag & if self.r8(0xa9) != 0 { 2 } else { 1 } == 0 {
                2
            } else {
                0
            },
        );
    }

    fn update_fullsize_y(&mut self) {
        let flag = LAYOUT_QUADRANT_FLAGS[usize::from(self.r8(0xa8))];
        self.w8(
            0xa7,
            if self.r8(0x453) != 0 || flag & if self.r8(0xaa) != 0 { 8 } else { 4 } == 0 {
                2
            } else {
                0
            },
        );
    }

    pub fn dungeon_start_inter_room_trans_left(&mut self) {
        assert_eq!(self.r8(0x11), 0);
        self.w8(0xa9, self.r8(0xa9) ^ 1);
        self.dungeon_adjust_quadrant();
        self.adjust_room_bounds_a(0x608, 0xff00);
        self.dung_save_data_for_current_room();
        self.dungeon_transition_adjust_camera_x(self.r8(0xa9) ^ 1);
        self.handle_edge_transition_adjust_camera_boundaries(3);
        self.w8(0x11, 1);
        if self.r8(0xa9) != 0 {
            self.adjust_room_bounds_b(0x608, 0xfe00);
            self.w8(0xa2, self.r8(0xa0));
            if self.r8(0x114) & 0xcf == 0x89 {
                let room = self.r8(0xc003);
                self.w8(0xa0, room);
                self.dungeon_adjust_for_teleport_doors(room.wrapping_add(1), 0xff);
            } else {
                if self.r8(0xa0) != self.r8(0x48e) {
                    self.w8(0xa2, self.r8(0x48e));
                    self.dungeon_adjust_after_spiral_stairs();
                }
                self.w8(0xa0, self.r8(0xa0).wrapping_sub(1));
            }
            self.w8(0x11, 2);
            self.finish_room_edge_plane_change();
        }
        self.w8(0xef, 0);
        self.update_fullsize_y();
    }

    pub fn dung_start_inter_room_trans_left_plus(&mut self) {
        self.w16(0x22, self.r16(0x22).wrapping_sub(8));
        self.dungeon_start_inter_room_trans_left();
    }

    pub fn dungeon_start_inter_room_trans_up(&mut self) {
        assert_eq!(self.r8(0x11), 0);
        self.w8(0xaa, self.r8(0xaa) ^ 2);
        self.dungeon_adjust_quadrant();
        self.adjust_room_bounds_a(0x600, 0xff00);
        self.dung_save_data_for_current_room();
        self.dungeon_transition_adjust_camera_y(self.r8(0xaa) ^ 2);
        self.handle_edge_transition_adjust_camera_boundaries(1);
        self.w8(0x11, 1);
        if self.r8(0xaa) != 0 {
            self.adjust_room_bounds_b(0x600, 0xfe00);
            self.w8(0xa2, self.r8(0xa0));
            if self.r8(0x114) == 0x8e {
                self.dung_handle_exit_to_overworld();
                return;
            }
            if self.r16(0xa0) == 0 {
                self.save_dungeon_keys();
                self.w8(0x10, 25);
                self.w8(0x11, 0);
                self.w8(0xb0, 0);
                return;
            }
            if self.r8(0x48e) == self.r8(0xa0) {
                self.w8(0xa2, self.r8(0x48e));
                self.dungeon_adjust_after_spiral_stairs();
            }
            self.w8(0xa0, self.r8(0xa0).wrapping_sub(0x10));
            self.w8(0x11, 2);
            self.finish_room_edge_plane_change();
        }
        self.w8(0xef, 0);
        self.update_fullsize_x();
    }

    pub fn dungeon_start_inter_room_trans_down(&mut self) {
        assert_eq!(self.r8(0x11), 0);
        self.w8(0xaa, self.r8(0xaa) ^ 2);
        self.dungeon_adjust_quadrant();
        self.adjust_room_bounds_a(0x600, 0x100);
        self.dung_save_data_for_current_room();
        self.dungeon_transition_adjust_camera_y(self.r8(0xaa));
        self.handle_edge_transition_adjust_camera_boundaries(0);
        self.w8(0x11, 1);
        if self.r8(0xaa) == 0 {
            self.adjust_room_bounds_b(0x600, 0x200);
            self.w8(0xa2, self.r8(0xa0));
            if self.r8(0x114) == 0x8e {
                self.dung_handle_exit_to_overworld();
                return;
            }
            if self.r8(0xa0) != self.r8(0x48e) {
                self.w8(0xa2, self.r8(0x48e));
                self.dungeon_adjust_after_spiral_stairs();
            }
            self.w8(0xa0, self.r8(0xa0).wrapping_add(0x10));
            self.w8(0x11, 2);
            self.finish_room_edge_plane_change();
        }
        self.w8(0xef, 0);
        self.update_fullsize_x();
    }

    pub fn dungeon_start_inter_room_trans_right(&mut self) {
        assert_eq!(self.r8(0x11), 0);
        self.w8(0xa9, self.r8(0xa9) ^ 1);
        self.dungeon_adjust_quadrant();
        self.adjust_room_bounds_a(0x608, 0x100);
        self.dung_save_data_for_current_room();
        self.dungeon_transition_adjust_camera_x(self.r8(0xa9));
        self.handle_edge_transition_adjust_camera_boundaries(2);
        self.w8(0x11, 1);
        if self.r8(0xa9) == 0 {
            self.adjust_room_bounds_b(0x608, 0x200);
            self.w8(0xa2, self.r8(0xa0));
            if self.r8(0x114) & 0xcf == 0x89 {
                let room = self.r8(0xc004);
                self.w8(0xa0, room);
                self.dungeon_adjust_for_teleport_doors(room.wrapping_sub(1), 1);
            } else {
                if self.r8(0xa0) != self.r8(0x48e) {
                    self.w8(0xa2, self.r8(0x48e));
                    self.dungeon_adjust_after_spiral_stairs();
                }
                self.w8(0xa0, self.r8(0xa0).wrapping_add(1));
            }
            self.w8(0x11, 2);
            self.finish_room_edge_plane_change();
        }
        self.w8(0xef, 0);
        self.update_fullsize_y();
    }

    pub fn handle_edge_transition_movement_east_right_by8(&mut self) {
        self.w16(0x22, self.r16(0x22).wrapping_add(8));
        self.dungeon_start_inter_room_trans_right();
    }

    pub fn handle_edge_transition_movement_south_down_by16(&mut self) {
        self.w16(0x20, self.r16(0x20).wrapping_add(16));
        self.dungeon_start_inter_room_trans_down();
    }

    pub fn dungeon_handle_edge_transition_movement(&mut self, direction: u8) {
        let mask = [3u8, 3, 12, 12][usize::from(direction)];
        self.w8(0x67, self.r8(0x67) & mask);
        match direction {
            0 => self.dungeon_start_inter_room_trans_right(),
            1 => self.dungeon_start_inter_room_trans_left(),
            2 => self.dungeon_start_inter_room_trans_down(),
            3 => self.dungeon_start_inter_room_trans_up(),
            _ => panic!("invalid edge-transition direction"),
        }
    }

    pub fn dungeon_try_screen_edge_transition(&mut self) {
        let direction = if self.r8(0x30) != 0 {
            let y = self.r16(0x20) & 0x1ff;
            if y < 4 {
                Some(3)
            } else if y >= 476 {
                Some(2)
            } else {
                None
            }
        } else {
            None
        }
        .or_else(|| {
            if self.r8(0x31) == 0 {
                return None;
            }
            let x = self.r16(0x22) & 0x1ff;
            if x < 8 {
                Some(1)
            } else if x >= 489 {
                Some(0)
            } else {
                None
            }
        });
        if let Some(direction) = direction
            && !self
                .hooks
                .link_check_for_edge_screen_transition(&mut self.ram)
            && self.r8(0x10) == 7
        {
            self.dungeon_handle_edge_transition_movement(direction);
            if self.r8(0x10) == 7 {
                self.w8(0x11, 2);
            }
        }
    }

    pub fn handle_edge_transition_adjust_camera_boundaries(&mut self, transition: u8) {
        self.w8(0x418, transition);
        let direction = self.r8(0x67);
        if direction & 3 != 0 {
            let mut index = if direction & 1 != 0 { 0 } else { 2 };
            if self.r8(0xa9) != 0 {
                index += 1;
            }
            let low = CAMERA_BOUNDS_X[index];
            self.w16(0x61c, low);
            self.w16(0x61e, low + 2);
        } else {
            let mut index = if direction & 4 != 0 { 0 } else { 2 };
            if self.r8(0xaa) != 0 {
                index += 1;
            }
            let low = CAMERA_BOUNDS_Y[index];
            self.w16(0x618, low);
            self.w16(0x61a, low + 2);
        }
    }

    pub fn dungeon_handle_camera(&mut self) {
        let y_velocity = self.r8(0x30) as i8;
        if y_velocity != 0 {
            let z = if self.r8(0x78) != 0 && self.r16(0x24) != 0xffff {
                self.r16(0x24)
            } else {
                0
            };
            let y = self.r16(0x20).wrapping_sub(z) & 0x1ff;
            let y = y.wrapping_add(12);
            let scroll = if y_velocity < 0 { -1i16 } else { 1i16 };
            for _ in 0..y_velocity.unsigned_abs() {
                let mut quadrant = usize::from(self.r8(0xa7) >> 1);
                let reached = if y_velocity < 0 {
                    y <= self.r16(0x618)
                } else {
                    quadrant += 2;
                    y >= self.r16(0x61a)
                };
                if !reached || self.r16(0xe8) == self.r16(0x600 + quadrant * 2) {
                    continue;
                }
                self.w16(0xe8, self.r16(0xe8).wrapping_add_signed(scroll));
                if self.r16(0xa0) == 0xffff {
                    continue;
                }
                self.w16(0x622, self.r16(0x622).wrapping_add(0x8000));
                let half = (scroll >> 1) + i16::from(self.r16(0x622) & 0x8000 == 0);
                self.w16(0xe6, self.r16(0xe6).wrapping_add_signed(half));
                self.w16(0x618, self.r16(0x618).wrapping_add_signed(scroll));
                self.w16(0x61a, self.r16(0x618).wrapping_add(2));
            }
        }
        let x_velocity = self.r8(0x31) as i8;
        if x_velocity != 0 {
            let x = (self.r16(0x22) & 0x1ff).wrapping_add(8);
            let scroll = if x_velocity < 0 { -1i16 } else { 1i16 };
            for _ in 0..x_velocity.unsigned_abs() {
                let mut quadrant = usize::from(self.r8(0xa6) >> 1);
                let reached = if x_velocity < 0 {
                    x <= self.r16(0x61c)
                } else {
                    quadrant += 2;
                    x >= self.r16(0x61e)
                };
                if !reached || self.r16(0xe2) == self.r16(0x608 + quadrant * 2) {
                    continue;
                }
                self.w16(0xe2, self.r16(0xe2).wrapping_add_signed(scroll));
                if self.r16(0xa0) == 0xffff {
                    continue;
                }
                self.w16(0x620, self.r16(0x620).wrapping_add(0x8000));
                let half = (scroll >> 1) + i16::from(self.r16(0x620) & 0x8000 == 0);
                self.w16(0xe0, self.r16(0xe0).wrapping_add_signed(half));
                self.w16(0x61c, self.r16(0x61c).wrapping_add_signed(scroll));
                self.w16(0x61e, self.r16(0x61c).wrapping_add(2));
            }
        }
        if self.r16(0xa0) != 0xffff && matches!(self.r8(0x414), 0 | 2 | 3 | 4 | 6..=u8::MAX) {
            self.w16(0xe0, self.r16(0xe2));
            self.w16(0xe6, self.r16(0xe8));
        }
    }

    pub fn dung_handle_exit_to_overworld(&mut self) {
        self.hooks.save_dungeon_keys(&mut self.ram);
        self.save_quadrants_to_sram();
        self.w8(0x10c, 8);
        self.w8(0x10, 15);
        self.w8(0x11, 0);
        self.w8(0xb0, 0);
        self.dungeon_reset_torch_background_and_player_inner();
    }

    pub fn dungeon_transition_scroll_room(&mut self) {
        let counter = self.r8(0x126).wrapping_add(1);
        self.w8(0x126, counter);
        let direction = usize::from(self.r8(0x418));
        self.w16(0x11a, 0);
        self.w16(0x11c, 0);
        let delta = STAIRCASE_TAB3[direction] as u16;
        let value = if direction >= 2 {
            let value = self.r16(0xe2).wrapping_add(delta) & !1;
            self.w16(0xe0, value);
            self.w16(0xe2, value);
            if counter >= STAIRCASE_TAB4[direction] as u8 {
                self.w16(0x20, self.r16(0x20).wrapping_add(delta));
            }
            value
        } else {
            let value = self.r16(0xe8).wrapping_add(delta) & !1;
            self.w16(0xe6, value);
            self.w16(0xe8, value);
            if counter >= STAIRCASE_TAB4[direction] as u8 {
                self.w16(0x22, self.r16(0x22).wrapping_add(delta));
            }
            value
        };
        if value & 0x1fc == self.r16(0x610 + direction * 2) {
            self.set_and_save_visited_quadrant_flags();
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
            self.w8(0x126, 0);
            if self.r8(0x11) == 2 {
                self.water_flood_build_one_quadrant_for_vram();
            }
        }
    }

    pub fn module07_11_0a_scroll_camera(&mut self) {
        self.w8(0x4b, 12);
        self.w8(0x2f9, 12);
        let mut direction = usize::from(self.r8(0x418));
        let value = self
            .r16(0xe8)
            .wrapping_add(STAIRCASE_TAB3[direction] as u16)
            & !3;
        self.w16(0xe6, value);
        self.w16(0xe8, value);
        if value & 0x1fc == self.r16(0x610 + direction * 2) {
            if self.r8(0x11) >= 18 {
                direction += 2;
            }
            self.w16(
                0x22,
                self.r16(0x22)
                    .wrapping_add(STAIRCASE_TAB5[direction] as u16),
            );
            self.w8(0x4b, 0);
            self.w8(0x2f9, 0);
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        }
    }

    pub fn dungeon_transition_find_subtile_landing(&mut self) {
        self.dungeon_reset_torch_background_and_player_inner();
        self.subtile_transition_calculate_landing();
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        let room = usize::from(self.r16(0xa0));
        self.w16(
            0xf000 + room * 2,
            self.r16(0xf000 + room * 2) | self.r16(0x40c),
        );
    }

    pub fn dungeon_inter_room_trans_state13(&mut self) {
        if self.r8(0xc005) | self.r8(0xc006) != 0 {
            self.hooks.apply_palette_filter_bounce(&mut self.ram);
        }
        self.dungeon_intra_room_trans_state5();
    }

    pub fn dungeon_intra_room_trans_state5(&mut self) {
        self.hooks
            .link_handle_moving_animation_full_long_entry(&mut self.ram);
        if !self.dungeon_transition_move_link_out_door() {
            return;
        }
        if self.r8(0x4e) == 2 || self.r8(0x4e) == 4 {
            self.w8(0x6c, 0);
        }
        self.w8(0x49, 0);
        self.w8(0x4e, 0);
        self.w8(0x418, 0);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_11_11_keep_sliding(&mut self) {
        if self.r8(0x464) == 0 {
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        } else {
            self.apply_grayscale_fixed_incremental();
        }
    }

    pub fn module07_11_00_prep_and_reset(&mut self) {
        if self.r8(0x372) != 0 {
            self.w8(0x372, 0);
            self.w8(0x5e, 2);
        }
        self.w8(0x12e, if self.r8(0x462) & 4 != 0 { 24 } else { 22 });
        if self.r16(0xa0) == 48 || self.r16(0xa0) == 64 {
            self.w8(0x12c, 0xf1);
        }
        self.reset_transition_props_and_advance_reset_interface();
    }

    pub fn module07_11_straight_interroom_stairs(&mut self) {
        let state = self.r8(0xb0);
        if state >= 3 {
            self.dungeon_load_attribute_selectable();
        }
        if state >= 13 {
            self.hooks.graphics_incremental_vram_upload(&mut self.ram);
        }
        let timer = self.r8(0x464);
        if timer != 0 {
            self.w8(0x464, timer.wrapping_sub(1));
            if timer == 16 {
                self.w8(0x57, 2);
            }
            self.w8(0x67, if self.r8(0x11) == 18 { 8 } else { 4 });
            self.hooks.link_handle_velocity(&mut self.ram);
        }
        self.hooks
            .link_handle_moving_animation_full_long_entry(&mut self.ram);
        match state {
            0 => self.module07_11_00_prep_and_reset(),
            1 => self.module07_11_01_fade_out(),
            2 => self.module07_11_02_load_and_prep_room(),
            3 => self.module07_11_03_filter_and_load_bg_chars(),
            4 => self.module07_11_04_filter_do_bg_and_reset_sprites(),
            5 | 7 => self.dungeon_spiral_staircase11(),
            6 | 8 => self.dungeon_spiral_staircase12(),
            9 => self.module07_11_09_load_sprite_graphics(),
            10 => self.module07_11_0a_scroll_camera(),
            11 => self.module07_11_0b_prep_destination(),
            12 | 14 => self.dungeon_inter_room_trans_state4(),
            13 => self.dungeon_inter_room_trans_not_dark_room(),
            15 => self.dungeon_double_apply_and_increment_grayscale(),
            16 => self.module07_11_19_set_song_and_filter(),
            17 => self.module07_11_11_keep_sliding(),
            18 => self.reset_then_cache_room_entry_properties(),
            _ => panic!("invalid straight-stair transition state"),
        }
    }

    pub fn module07_11_02_load_and_prep_room(&mut self) {
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        self.dungeon_load_room();
        self.hooks.dungeon_restore_star_tile_chr(&mut self.ram);
        self.hooks.dungeon_transition_aux_gfx(&mut self.ram);
        self.dungeon_load_custom_tile_attr();
        self.dungeon_adjust_for_room_layout();
        self.hooks.follower_initialize(&mut self.ram);
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_11_01_fade_out(&mut self) {
        if self.r8(0x464) < 9 {
            self.hooks.apply_palette_filter_bounce(&mut self.ram);
            if self.r8(0xc007) == 23 {
                self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
            }
        }
    }

    pub fn module07_11_03_filter_and_load_bg_chars(&mut self) {
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        self.dungeon_transition_trigger_bgc34_update_and_advance();
    }

    pub fn module07_11_04_filter_do_bg_and_reset_sprites(&mut self) {
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        self.dungeon_transition_trigger_bgc56_update_and_advance();
        self.w8(0xa2, self.r8(0xa0));
        self.hooks.dungeon_reset_sprites(&mut self.ram);
    }

    pub fn module07_11_09_load_sprite_graphics(&mut self) {
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
        self.w8(0xb0, self.r8(0xb0).wrapping_sub(1));
        self.hooks.load_new_sprite_gfx_set(&mut self.ram);
        self.hooks
            .dungeon_handle_translucency_and_palette(&mut self.ram);
    }

    pub fn module07_11_19_set_song_and_filter(&mut self) {
        if self.r8(0x200) == 5 && self.r8(0xc009) == 0 {
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
            if self.r16(0xa0) == 48 {
                self.w8(0x12c, 0x1c);
            } else if self.r16(0xa0) == 64 {
                self.w8(0x12c, 0x10);
            }
        }
        self.apply_grayscale_fixed_incremental();
    }

    pub fn module07_14_00_scroll_camera(&mut self) {
        for _ in 0..2 {
            self.approach_word(0xe2, self.r16(0xc180));
            self.approach_word(0xe8, self.r16(0xc182));
        }
        if self.r16(0xe2) == self.r16(0xc180) && self.r16(0xe8) == self.r16(0xc182) {
            self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
        }
        if self.r8(0x458) == 0 {
            self.hooks.mirror_bg1_bg2_offsets(&mut self.ram);
        }
    }

    pub fn module07_14_recover_from_fall(&mut self) {
        match self.r8(0xb0) {
            0 => self.module07_14_00_scroll_camera(),
            1 => self.hooks.recover_position_after_drowning(&mut self.ram),
            _ => {}
        }
    }

    fn approach_word(&mut self, address: usize, target: u16) {
        let value = self.r16(address);
        if value != target {
            self.w16(
                address,
                if value < target {
                    value.wrapping_add(1)
                } else {
                    value.wrapping_sub(1)
                },
            );
        }
    }

    pub fn module07_15_01_apply_mosaic_and_filter(&mut self) {
        self.hooks.conditional_mosaic_control(&mut self.ram);
        self.w8(0x95, self.r8(0xc011) | 3);
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
    }

    pub fn module07_15_warp_pad(&mut self) {
        let state = self.r8(0xb0);
        if state >= 3 {
            self.hooks.graphics_incremental_vram_upload(&mut self.ram);
            self.dungeon_load_attribute_selectable();
        }
        match state {
            0 => self.reset_transition_props_and_advance_reset_interface(),
            1 => self.module07_15_01_apply_mosaic_and_filter(),
            2 => self.dungeon_initialize_room_from_special(),
            3 => self.dungeon_transition_load_sprite_gfx(),
            4 => self.module07_15_04_sync_room_props_and_build_overlay(),
            5 | 7 | 9 | 11 => self.dungeon_inter_room_trans_state4(),
            6 | 8 | 10 => self.dungeon_inter_room_trans_not_dark_room(),
            12 => self.dungeon_staircase14(),
            13 => self.module07_15_0e_fade_in_from_warp(),
            14 => self.module07_15_0f_finalize_and_cache_entry(),
            _ => panic!("invalid warp transition state"),
        }
    }

    pub fn module07_15_04_sync_room_props_and_build_overlay(&mut self) {
        self.apply_grayscale_fixed_incremental();
        if self.r16(0xa0) == 0x17 {
            self.w8(0xa4, 4);
        }
        self.hooks.mirror_bg1_bg2_offsets(&mut self.ram);
        self.dungeon_adjust_for_room_layout();
        let mut sub = SPIRAL_TAB1[usize::from(self.r8(0x414))] as u8;
        let mut main = 0x16;
        if sub & 0x80 != 0 {
            main = 0x17;
            sub = 0;
        }
        self.w8(0x1c, main);
        self.w8(0x1d, sub);
        self.water_flood_build_one_quadrant_for_vram();
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn module07_15_0e_fade_in_from_warp(&mut self) {
        if self.r16(0xc007) & 1 != 0 && self.r8(0xc011) != 0 {
            self.w8(0xc011, self.r8(0xc011).wrapping_sub(0x10));
        }
        self.w8(0x94, 9);
        self.w8(0x95, self.r8(0xc011) | 3);
        self.hooks.apply_palette_filter_bounce(&mut self.ram);
    }

    pub fn module07_15_0f_finalize_and_cache_entry(&mut self) {
        if self.r8(0x200) == 5 {
            self.set_and_save_visited_quadrant_flags();
            self.w8(0x11, 0);
            self.reset_then_cache_room_entry_properties();
        }
    }

    pub fn module07_17_pressure_plate(&mut self) {
        let state = self.r8(0xb0).wrapping_sub(1);
        self.w8(0xb0, state);
        if state != 0 {
            return;
        }
        self.w16(0x22, self.r16(0x22).wrapping_sub(2));
        let position = self.r16(0x4b6);
        self.dungeon_update_tile_map_with_common_tile(
            (position & 0x3f) << 3,
            (position >> 3) & 0x1f8,
            0xe,
        );
        self.w8(0x11, self.r8(0x10c));
    }

    pub fn module07_19_mirror_fade(&mut self) {
        self.hooks
            .overworld_reset_mosaic_always_increase(&mut self.ram);
        let brightness = self.r8(0x13).wrapping_sub(1);
        self.w8(0x13, brightness);
        if brightness == 0 {
            self.w8(0x10, 5);
            self.w8(0x11, 0);
            self.w8(0x14, 0);
            self.w8(0x133, self.r8(0x130));
            if self.r8(0xabd) != 0 {
                self.hooks.palette_revert_translucency_swap(&mut self.ram);
            }
        }
    }

    pub fn module07_16_update_pegs(&mut self) {
        let state = self.r8(0xb0).wrapping_add(1);
        self.w8(0xb0, state);
        if state & 3 != 0 {
            return;
        }
        match state >> 2 {
            0 | 1 => self.hooks.update_pegs_step1(&mut self.ram),
            2 => self.hooks.update_pegs_step2(&mut self.ram),
            3 => self.hooks.recover_peg_gfx_from_mapping(&mut self.ram),
            4 => {
                self.dungeon_flip_crystal_peg_attribute();
                self.w8(0xb0, 0);
                self.w8(0x11, 0);
            }
            _ => {}
        }
    }

    pub fn push_block_handle_falling(&mut self, mut object_x2: u8) {
        object_x2 >>= 1;
        let timeout = self.r8(0x2c4).wrapping_sub(1);
        self.w8(0x2c4, timeout);
        if timeout & 0x80 == 0 {
            return;
        }
        self.w8(0x2c4, 9);
        let animation = self.r8(0x2c3).wrapping_add(1);
        self.w8(0x2c3, animation);
        if animation == 4 {
            self.w8(0x500 + usize::from(object_x2) * 2, 0);
            self.w8(0x2c3, 0);
            let pushed = self.r8(0x5fd).wrapping_sub(1);
            let index = usize::from(pushed == object_x2);
            self.w8(0x5fc + index, 0);
        }
    }

    pub fn sprite_handle_pushed_blocks_one(&mut self, index: usize) {
        assert!(index < 2);
        let (oam, ext) = self.hooks.oam_allocate_from_region_b(&mut self.ram, 4);
        self.w16(0x90, oam);
        self.w16(0x92, ext);
        let y =
            u16::from(self.r8(0x5f0 + index * 2)) | (u16::from(self.r8(0x5ec + index * 2)) << 8);
        let x =
            u16::from(self.r8(0x5e4 + index * 2)) | (u16::from(self.r8(0x5e0 + index * 2)) << 8);
        let y = y.wrapping_sub(self.r16(0xe8).wrapping_add(1));
        let x = x.wrapping_sub(self.r16(0xe2));
        if self.r8(0x2c3) < 3 {
            let oam = usize::from(oam);
            self.w8(oam, x as u8);
            self.w8(oam + 1, y as u8);
            self.w8(oam + 2, 12);
            self.w8(oam + 3, 0x20);
            self.w8(usize::from(ext), 2);
        }
    }

    pub fn sprite_dungeon_draw_all_push_blocks(&mut self) {
        for index in (0..=1).rev() {
            if self.r8(0x5fc + index) != 0 {
                self.sprite_handle_pushed_blocks_one(index);
            }
        }
    }

    pub fn push_block_apply_velocity(&mut self, index: usize) {
        assert!(index < 2);
        let facing = self.r8(0x5f8 + index * 2);
        let direction = usize::from(facing >> 1);
        let mask = PUSHED_BLOCK_DIR_MASK[direction];
        self.w8(0x28, 0);
        self.w8(0x27, 0);
        let (lo_address, hi_address, velocity) = if mask & 3 != 0 {
            let velocity = if mask & 2 != 0 { -12i8 } else { 12i8 };
            self.w8(0x28, velocity as u8);
            (0x5e4, 0x5e0, velocity)
        } else {
            let velocity = if mask & 8 != 0 { -12i8 } else { 12i8 };
            self.w8(0x27, velocity as u8);
            (0x5f0, 0x5ec, velocity)
        };
        let coordinate = u32::from(self.r8(0x5f4 + index * 2))
            | (u32::from(self.r8(lo_address + index * 2)) << 8)
            | (u32::from(self.r8(hi_address + index * 2)) << 16);
        let coordinate = coordinate.wrapping_add_signed(i32::from(velocity) * 16);
        self.w8(0x5f4 + index * 2, coordinate as u8);
        self.w8(lo_address + index * 2, (coordinate >> 8) as u8);
        self.w8(hi_address + index * 2, (coordinate >> 16) as u8);
        if (coordinate >> 8) as u8 & 0xf == self.r8(0x5e8 + index * 2) {
            let object = usize::from(self.r8(0x5fc + index).wrapping_sub(1));
            self.w16(
                0x500 + object * 2,
                self.r16(0x500 + object * 2).wrapping_add(1),
            );
            self.w8(0x50, self.r8(0x50) & !4);
            self.w8(0x48, self.r8(0x48) & !4);
        }
        let x =
            u16::from(self.r8(0x5e4 + index * 2)) | (u16::from(self.r8(0x5e0 + index * 2)) << 8);
        let y =
            u16::from(self.r8(0x5f0 + index * 2)) | (u16::from(self.r8(0x5ec + index * 2)) << 8);
        for sprite in (0..16).rev() {
            if self.r8(0xdd0 + sprite) >= 9 {
                let sprite_x =
                    u16::from(self.r8(0xd10 + sprite)) | (u16::from(self.r8(0xd30 + sprite)) << 8);
                let sprite_y =
                    u16::from(self.r8(0xd00 + sprite)) | (u16::from(self.r8(0xd20 + sprite)) << 8);
                if x.wrapping_sub(sprite_x).wrapping_add(0x10) < 0x20
                    && y.wrapping_sub(sprite_y).wrapping_add(0x10) < 0x20
                {
                    self.w8(0xea0 + sprite, 8);
                    self.w8(0xf40 + sprite, PUSH_BLOCK_TAB1[direction]);
                    self.w8(0xf30 + sprite, PUSH_BLOCK_TAB2[direction]);
                }
            }
        }
    }

    pub fn push_block_handle_collision(&mut self, index: usize, x: u16, y: u16) {
        assert!(index < 2);
        self.w8(0x40, (self.r16(0x22) >> 8) as u8);
        self.w8(0x41, (self.r16(0x20) >> 8) as u8);
        let mut direction = 3usize;
        let mut mask = self.r8(0x67) & 0xf;
        while mask & 1 == 0 {
            mask >>= 1;
            if direction == 0 {
                return;
            }
            direction -= 1;
        }
        let link_axis = if direction < 2 {
            self.r16(0x20)
        } else {
            self.r16(0x22)
        };
        let block_axis = if direction < 2 { x } else { y };
        let r0 = link_axis.wrapping_add(PUSH_BLOCK_A[direction]);
        let r2 = link_axis.wrapping_add(PUSH_BLOCK_B[direction]);
        let r4 = block_axis.wrapping_add(PUSH_BLOCK_C[direction]);
        let r6 = block_axis.wrapping_add(PUSH_BLOCK_D[direction]);
        let coordinate_address = if direction < 2 { 0x22 } else { 0x20 };
        let r8 = self
            .r16(coordinate_address)
            .wrapping_add(PUSH_BLOCK_E[direction]);
        let r10 = (if direction < 2 { y } else { x }).wrapping_add(PUSH_BLOCK_F[direction]);
        self.w8(0x48, self.r8(0x48) & !4);
        if (r0 >= r4 && r0 < r6) || (r2 >= r4 && r2 < r6) {
            if self.r8(0x2f) == self.r8(0x5f8 + index * 2) {
                self.w8(
                    0x48,
                    self.r8(0x48) | if self.r8(0x5fc + index) != 0 { 4 } else { 1 },
                );
            }
            let difference = r8.wrapping_sub(r10);
            let collides = if direction & 1 != 0 {
                r8 >= r10 && difference < 8
            } else {
                difference >= 0xfff8
            };
            if collides {
                self.w16(
                    coordinate_address,
                    self.r16(coordinate_address).wrapping_sub(difference),
                );
                let velocity_address = if direction & 2 != 0 { 0x31 } else { 0x30 };
                self.w8(
                    velocity_address,
                    self.r8(velocity_address).wrapping_sub(difference as u8),
                );
            }
        }
        self.hooks.handle_indoor_camera_and_doors(&mut self.ram);
    }

    pub fn push_block_slide(&mut self, object_x2: u8) {
        if self.r8(0x11) != 0 {
            return;
        }
        let index = usize::from(self.r8(0x5fd).wrapping_sub(1).wrapping_mul(2) == object_x2);
        self.w8(0x2c4, 9);
        self.w8(0x2c3, 0);
        self.push_block_apply_velocity(index);
        let y =
            u16::from(self.r8(0x5f0 + index * 2)) | (u16::from(self.r8(0x5ec + index * 2)) << 8);
        let x =
            u16::from(self.r8(0x5e4 + index * 2)) | (u16::from(self.r8(0x5e0 + index * 2)) << 8);
        self.push_block_handle_collision(index, x, y);
    }

    pub fn dungeon_check_for_and_id_liftable_tile(&self) -> u16 {
        let index = usize::from(self.r8(0x2f) >> 1);
        let x = self
            .r16(0x22)
            .wrapping_add_signed(i16::from(DUNGEON_QUERY_IF_TILE_LIFTABLE_X[index]))
            & 0x1f8;
        let y = self
            .r16(0x20)
            .wrapping_add_signed(i16::from(DUNGEON_QUERY_IF_TILE_LIFTABLE_Y[index]))
            & 0x1f8;
        let position =
            usize::from((y << 3) | (x >> 3) | if self.r8(0xee) != 0 { 0x1000 } else { 0 });
        let attribute = self.r8(0x12000 + position);
        if attribute & 0xf0 != 0x70 {
            return 0xffff;
        }
        let replacement = self.r16(0x500 + usize::from(attribute & 0xf) * 2);
        if replacement == 0 {
            0xffff
        } else if replacement & 0xf0f0 == 0x2020 {
            0x55
        } else {
            DUNGEON_QUERY_IF_TILE_LIFTABLE_RV[usize::from(replacement & 0xf)]
        }
    }

    pub fn dungeon_push_block_handler(&mut self) {
        while self.r16(0x42c) != self.r16(0x478) {
            let object_x2 = self.r16(0x42c);
            let index = usize::from(object_x2 >> 1);
            match self.r16(0x500 + index * 2) {
                1 => {
                    self.room_draw_16x16_single(object_x2 as u8);
                    let direction = usize::from(self.r8(0x474) >> 1);
                    self.w16(
                        0x540 + index * 2,
                        self.r16(0x540 + index * 2)
                            .wrapping_add_signed(PUSH_BLOCK_MOVE_DISTANCES[direction]),
                    );
                    self.w16(0x500 + index * 2, 2);
                }
                2 => {
                    self.push_block_slide(object_x2 as u8);
                    if self.r16(0x500 + usize::from(self.r16(0x42c) >> 1) * 2) == 3 {
                        self.push_block_check_for_pit(self.r16(0x42c) as u8);
                        let state = 0x500 + usize::from(self.r16(0x42c) >> 1) * 2;
                        self.w16(state, self.r16(state).wrapping_add(1));
                    }
                }
                4 => self.push_block_handle_falling(object_x2 as u8),
                _ => {}
            }
            self.w16(0x42c, self.r16(0x42c).wrapping_add(2));
        }
    }

    pub fn push_block_check_for_pit(&mut self, object_x2: u8) {
        let index = usize::from(object_x2 >> 1);
        let tilemap = self.r16(0x540 + index * 2);
        if tilemap & 0x4000 == 0 {
            self.w8(0x641, self.r8(0x641) ^ 1);
        }
        let position = usize::from((tilemap & 0x3fff) >> 1);
        let attribute = self.r8(0x12000 + position);
        if attribute == 0x20 {
            self.w8(0x12e, 0x20);
            let movable = usize::from(self.r16(0x520 + index * 2) >> 2);
            self.w16(0xf940 + movable * 4, u16::from(self.r8(0xc000)));
            self.w16(0xf942 + movable * 4, tilemap);
            return;
        }
        let changable = usize::from(self.r8(0x5fd).wrapping_sub(1) == index as u8);
        self.w8(0x5fc + changable, 0);
        if attribute == 0x23 {
            self.w16(0x466, self.r16(0x468) ^ 1);
            self.w16(0x500 + index * 2, 4);
        } else {
            self.w16(0x500 + index * 2, 0xffff);
        }
        self.dungeon_store_2x2(position as u16, [0x922, 0x932, 0x923, 0x933], 0x27);
    }

    pub fn dungeon_lift_and_replace_liftable(&mut self, point: &mut (u16, u16)) -> u8 {
        let direction = usize::from(self.r8(0x2f) >> 1);
        let x = self
            .r16(0x22)
            .wrapping_add_signed(i16::from(DUNGEON_QUERY_IF_TILE_LIFTABLE_X[direction]));
        let y = self
            .r16(0x20)
            .wrapping_add_signed(i16::from(DUNGEON_QUERY_IF_TILE_LIFTABLE_Y[direction]));
        *point = (x, y);
        self.w16(0xc8, y);
        self.w16(0xca, x);
        let position = usize::from(
            ((y & 0x1f8) << 3) | ((x & 0x1f8) >> 3) | if self.r8(0xee) != 0 { 0x1000 } else { 0 },
        );
        let attribute = self.r8(0x12000 + position);
        assert_eq!(attribute & 0x70, 0x70);
        let index = usize::from(attribute & 0xf);
        let replacement = self.r16(0x500 + index * 2);
        if replacement & 0xf0f0 == 0x1010 {
            self.w16(0x42c, (index * 2) as u16);
            self.reveal_pot_item(position as u16, self.r16(0x540 + index * 2));
            self.room_draw_16x16_single(self.r16(0x42c) as u8);
            *point = self.manip_block_something();
            DUNGEON_QUERY_IF_TILE_LIFTABLE_RV[usize::from(replacement & 0xf)] as u8
        } else if replacement & 0xf0f0 == 0x2020 {
            self.thieves_attic_draw_lightened_hole(
                position as u16,
                (index as u16)
                    .wrapping_sub(replacement & 0xf)
                    .wrapping_mul(2),
                point,
            )
        } else {
            0
        }
    }

    pub fn thieves_attic_draw_lightened_hole(
        &mut self,
        position: u16,
        object_x2: u16,
        point: &mut (u16, u16),
    ) -> u8 {
        self.w16(0x42c, object_x2);
        self.reveal_pot_item(position, self.r16(0x540 + usize::from(object_x2 >> 1) * 2));
        for offset in [0u8, 2, 4, 6] {
            self.room_draw_16x16_single((object_x2 as u8).wrapping_add(offset));
        }
        *point = self.manip_block_something();
        0x55
    }

    pub fn handle_item_tile_action_dungeon(&mut self, x: u16, y: u16) -> u8 {
        let hammer = self.r8(0x301) & 2 != 0;
        if !hammer
            && (!self.hooks.break_pots_with_sword_enabled()
                || self.r8(0x3c) == 0
                || self.r8(0xf359) == 1)
        {
            return 0;
        }
        let position = usize::from(
            (y & 0x1f8).wrapping_mul(8) + x + if self.r8(0xee) != 0 { 0x1000 } else { 0 },
        );
        let tile = self.r8(0x12000 + position);
        if tile & 0xf0 != 0x70 {
            return 0;
        }
        let index = usize::from(tile & 0xf);
        let replacement = self.r16(0x500 + index * 2);
        if replacement & 0xf0f0 == 0x4040 {
            if !hammer {
                return 0;
            }
            self.w16(0x42c, (index * 2) as u16);
            self.room_draw_16x16_single(self.r16(0x42c) as u8);
            self.w8(0x12e, 0x11);
        } else if replacement & 0xf0f0 == 0x1010 {
            self.w16(0x42c, (index * 2) as u16);
            self.reveal_pot_item(position as u16, self.r16(0x540 + index * 2));
            self.room_draw_16x16_single(self.r16(0x42c) as u8);
            let point = self.manip_block_something();
            self.w8(0xb9c, self.r8(0xb9c) | 0x80);
            self.hooks
                .sprite_spawn_immediately_smashed_terrain(&mut self.ram, 1, point.0, point.1);
            self.hooks
                .ancilla_add_bush_poof(&mut self.ram, point.0, point.1);
        }
        0
    }

    pub fn reveal_pot_item(&mut self, position: u16, object_position: u16) {
        self.w8(0xb9c, 0);
        let secrets = self.hooks.dungeon_secrets().to_vec();
        let room = usize::from(self.r16(0xa0));
        assert!(secrets.len() >= room * 2 + 2);
        let mut cursor = usize::from(u16::from_le_bytes([
            secrets[room * 2],
            secrets[room * 2 + 1],
        ]));
        let mut index = 0usize;
        let data = loop {
            assert!(secrets.len() >= cursor + 3);
            let test = u16::from_le_bytes([secrets[cursor], secrets[cursor + 1]]);
            if test == 0xffff {
                return;
            }
            assert_eq!(test & 0x8000, 0);
            if test == object_position {
                break secrets[cursor + 2];
            }
            cursor += 3;
            index += 1;
        };
        if data == 0 {
            return;
        }
        if data < 0x80 {
            if data != 8 {
                let mask = 1u16 << index;
                let address = 0xf580 + room * 2;
                if self.r16(address) & mask != 0 {
                    return;
                }
                self.w16(address, self.r16(address) | mask);
            }
            self.w8(0xb9c, self.r8(0xb9c) | data);
        } else if data != 0x88 {
            let tile_index = usize::from(self.r8(0x12000 + usize::from(position)) & 0xf);
            let first =
                tile_index.wrapping_sub(usize::from(self.r16(0x500 + tile_index * 2) & 0xf));
            self.w16(0x42c, (first * 2) as u16);
            self.w8(0x12f, 0x1b);
            let source = self.object_data::<16>(0x5ba);
            for i in 0..4 {
                self.w16(0x560 + (first + i) * 2, source[i * 4]);
                self.w16(0x580 + (first + i) * 2, source[i * 4 + 1]);
                self.w16(0x5a0 + (first + i) * 2, source[i * 4 + 2]);
                self.w16(0x5c0 + (first + i) * 2, source[i * 4 + 3]);
            }
        } else {
            let index = usize::from(self.r16(0x42c) >> 1);
            self.w16(0x560 + index * 2, 0xd0b);
            self.w16(0x580 + index * 2, 0xd1b);
            self.w16(0x5a0 + index * 2, 0x4d0b);
            self.w16(0x5c0 + index * 2, 0x4d1b);
        }
    }

    pub fn used_for_straight_inter_room_staircase(&mut self) {
        for index in (0..10).rev() {
            if self.r8(0xc4a + index) == 13 {
                self.w8(0xc4a + index, 0);
            }
        }
        if self.r8(0x2e) >= 5 {
            self.w8(0x2e, 0);
        }
        self.w8(0x2b, 0);
        self.w8(0x2a, 0);
        self.w8(0x30a, 0);
        self.w8(0x371, 28);
        self.w8(0x378, 32);
        self.w8(0x37b, 1);
        let descending = self.r8(0x462) & 4 != 0;
        self.hooks
            .ancilla_sfx2_near(&mut self.ram, if descending { 0x18 } else { 0x16 });
        self.w16(
            0x53,
            self.r16(0x20)
                .wrapping_add(if descending { 0xfff1 } else { 16 }),
        );
        self.w16(0x51, self.r16(0x22));
    }

    pub fn reposition_link_after_spiral_stairs(&mut self) {
        const X: [i16; 4] = [-28, -28, 24, 24];
        const Y: [i16; 4] = [16, -10, -10, -32];

        self.w8(0x4b, 0);
        self.w8(0x2f9, 0);
        let plane = self.r8(0x48a);
        let shape = self.r8(0x492);
        let index =
            usize::from(plane == 0 && shape != 0) + if self.r8(0x462) & 4 != 0 { 2 } else { 0 };
        self.w16(0x22, self.r16(0x22).wrapping_add_signed(X[index]));
        self.w16(0x20, self.r16(0x20).wrapping_add_signed(Y[index]));

        if self.r8(0x1c) & 0x10 != 0 {
            if plane == 2 {
                self.w8(0xee, 3);
                self.w8(0x1c, self.r8(0x1c) & 0x0f);
                self.w8(0x1d, self.r8(0x1d) | 0x10);
                if shape != 2 {
                    self.w16(0x20, self.r16(0x20).wrapping_add(24));
                }
            }
        } else if plane != 2 {
            self.w8(0x1c, self.r8(0x1c) | 0x10);
            self.w8(0x1d, self.r8(0x1d) & 0x0f);
            if shape != 2 {
                self.w16(0x20, self.r16(0x20).wrapping_sub(24));
            }
        }
        self.hooks.follower_initialize(&mut self.ram);
    }

    pub fn handle_link_on_spiral_stairs(&mut self) {
        self.w16(0xfc2, self.r16(0x22));
        self.w16(0xfc4, self.r16(0x20));
        if self.r8(0x30a) != 0 {
            return;
        }
        self.w8(0x373, 0);
        self.w8(0x46, 0);
        self.w8(0x4d, 0);

        let descending = self.r8(0x462) & 4 != 0;
        self.w8(0x27, 0xfe);
        let timer = self.r8(0x371).wrapping_sub(1);
        self.w8(0x371, timer);
        if timer & 0x80 != 0 {
            self.w8(0x371, 0);
            self.w8(0x28, if descending { 0xfe } else { 2 });
            self.w8(0x27, if descending { 0 } else { 0xfe });
        }
        self.hooks.link_move_position(&mut self.ram);
        self.hooks
            .link_handle_moving_animation_start_with_dash(&mut self.ram);
        if self.r8(0x371) == 0 {
            let countdown = self.r8(0x378).wrapping_sub(1);
            self.w8(0x378, countdown);
            if countdown & 0x80 != 0 {
                self.w8(0x378, 0);
                self.w8(0x2f, if descending { 4 } else { 6 });
            }
        }

        let delta = (self.r16(0x22) as u8).wrapping_sub(self.r16(0x53) as u8) as i8;
        if delta.unsigned_abs() != 0 {
            return;
        }
        self.reposition_link_after_spiral_stairs();
        if self.r8(0xf3cc) != 0 {
            self.hooks.follower_initialize(&mut self.ram);
        }
        self.w16(
            0x53,
            self.r16(0x22)
                .wrapping_add(if descending { 0xfff8 } else { 12 }),
        );
        self.w8(0x30a, 1);
        self.w8(0x378, 6);
        self.hooks
            .ancilla_sfx2_near(&mut self.ram, if descending { 25 } else { 23 });
    }

    pub fn spiral_stairs_find_landing_spot(&mut self) {
        self.w8(0x373, 0);
        self.w8(0x46, 0);
        self.w8(0x4d, 0);
        self.w8(0x37b, 0);
        self.w16(0xfc2, self.r16(0x22));
        self.w16(0xfc4, self.r16(0x20));
        let countdown = self.r8(0x378).wrapping_sub(1);
        self.w8(0x378, countdown);
        if countdown & 0x80 != 0 {
            self.w8(0x378, 0);
            self.w8(0x2f, 2);
        }
        let descending = self.r8(0x462) & 4 != 0;
        self.w8(0x28, if descending { 0xfc } else { 4 });
        self.w8(0x27, if descending { 2 } else { 0 });
        if self.r8(0x30a) == 2 {
            self.w8(0x28, 0);
            self.w8(0x27, 16);
        }
        self.hooks.link_move_position(&mut self.ram);
        self.hooks
            .link_handle_moving_animation_start_with_dash(&mut self.ram);
        if self.r16(0x22) as u8 == self.r16(0x53) as u8 {
            self.w8(0x30a, 2);
        }
    }

    pub fn dungeon_load_custom_tile_attr(&mut self) {
        let theme = usize::from(self.r8(0xaa2));
        let source = self.hooks.dungeon_custom_tile_attributes(theme);
        assert!(source.len() >= 0x80);
        self.ram
            .slice_mut(0xfe00 + 0x140, 0x80)
            .copy_from_slice(&source[..0x80]);
    }

    pub fn dungeon_inter_room_trans_not_dark_room(&mut self) {
        self.water_flood_build_one_quadrant_for_vram();
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn dungeon_inter_room_trans_state4(&mut self) {
        self.dungeon_prepare_next_room_quadrant_upload();
        self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
    }

    pub fn link_check_bunny_status(&mut self) {
        if self.r8(0x5d) == 2 {
            let state = if self.r8(0x2e0) == 0 {
                0
            } else if self.r8(0xf357) != 0 {
                28
            } else {
                23
            };
            self.w8(0x5d, state);
        }
    }

    pub fn crystal_cutscene_initialize(&mut self) {
        self.w8(0x9a, 0x33);
        self.w8(0xc005, 0);
        self.w8(0xc007, 0);
        self.hooks.palette_assert_translucency_swap(&mut self.ram);
        self.hooks.palette_filter_crystal(&mut self.ram);
        for (index, &color) in CRYSTAL_MAIDEN_PAL.iter().enumerate() {
            self.w16(0xc500 + (112 + index) * 2, color);
        }
        self.w8(0x15, self.r8(0x15).wrapping_add(1));
        self.crystal_cutscene_spawn_maiden();
        self.hooks
            .crystal_cutscene_initialize_polyhedral(&mut self.ram);
    }

    pub fn crystal_cutscene_spawn_maiden(&mut self) {
        self.ram.slice_mut(0xdd0, 16).fill(0);
        let sprite = self.hooks.sprite_spawn_dynamically(&mut self.ram, 0, 0xab);
        assert!(sprite >= 0);
        let sprite = sprite as usize;
        self.w8(0xd30 + sprite, (self.r16(0x22) >> 8) as u8);
        self.w8(0xd20 + sprite, (self.r16(0x20) >> 8) as u8);
        self.w8(0xd10 + sprite, 0x78);
        self.w8(0xd00 + sprite, 0x7c);
        self.w8(0xde0 + sprite, 1);
        self.w8(0xf50 + sprite, 0xb);
        self.w8(0xe80 + sprite, 0);
        self.w8(0xf20 + sprite, 0);
        let interactive = self
            .hooks
            .ancilla_terminate_select_interactives(&mut self.ram, sprite as u8);
        self.w8(0xd90 + sprite, interactive);
        self.w8(0x2e9, 0);
        if self.r16(0x40c) as u8 == 24 {
            self.w8(0xf50 + sprite, 9);
            self.w8(0xf3cc, 1);
        } else {
            self.w8(0xf3cc, 6);
        }
        self.hooks.load_follower_graphics(&mut self.ram);
        self.w8(0xf3cc, 0);
        self.w16(
            0x422,
            self.r16(0xe2)
                .wrapping_sub(self.r16(0x22))
                .wrapping_add(0x79),
        );
        self.w16(0x424, 0x30u16.wrapping_sub(u16::from(self.r16(0xe6) as u8)));
        self.w8(0x428, 1);
    }

    pub fn module11_dungeon_falling_entrance(&mut self) {
        match self.r8(0xb0) {
            0 => {
                let entrance = usize::from(self.r8(0x10e));
                if self.hooks.dungeon_entrance_music_track(entrance) != 3 || self.r8(0xf3c5) >= 2 {
                    self.w8(0x12c, 0xf1);
                }
                self.reset_transition_props_and_advance_reset_interface();
            }
            1 => {
                if self.r8(0x1a) & 1 == 0 {
                    self.hooks.apply_palette_filter_bounce(&mut self.ram);
                }
            }
            2 => self.module11_02_load_entrance(),
            3 => self.dungeon_transition_load_sprite_gfx(),
            4 => {
                let brightness = self.r8(0x13).wrapping_add(1) & 0xf;
                self.w8(0x13, brightness);
                if brightness == 15 {
                    self.w8(0xb0, self.r8(0xb0).wrapping_add(1));
                }
                self.finish_dungeon_falling_entrance();
            }
            5 => self.finish_dungeon_falling_entrance(),
            _ => panic!("invalid dungeon falling-entrance state"),
        }
    }

    fn finish_dungeon_falling_entrance(&mut self) {
        self.hooks.handle_dungeon_landing_from_pit(&mut self.ram);
        if self.r8(0x11) != 0 {
            return;
        }
        self.w8(0x10, 7);
        self.w8(0xaaa, self.r8(0xaaa).wrapping_add(1));
        self.dungeon_play_blip_and_cache_quadrant_visits();
        self.reset_then_cache_room_entry_properties();
        self.w8(0x12c, self.r8(0x132));
        self.w8(0x133, self.r8(0x130));
    }

    pub fn module11_02_load_entrance(&mut self) {
        self.hooks.enable_force_blank(&mut self.ram);
        self.w8(0x99, 2);
        self.dungeon_load_entrance();
        let palace = self.r16(0x40c) as u8;
        let keys = if palace == 0xff {
            0xff
        } else {
            self.r8(0xf37c + usize::from(if palace == 2 { 0 } else { palace >> 1 }))
        };
        self.w8(0xf36f, keys);
        self.hooks.hud_rebuild(&mut self.ram);
        self.w8(0x5a, 4);
        self.w8(0x5b, 3);
        self.w8(0x4b, 12);
        self.w8(0x57, 16);
        let relative_y = self.r16(0x20).wrapping_sub(self.r16(0xe8)) as u8;
        self.w8(0x308, 0);
        self.w8(0x309, 0);
        self.w8(0x30b, 0);
        self.w16(0xa2, self.r16(0xa0));
        self.w16(0x51, self.r16(0x20));
        self.w16(
            0x20,
            self.r16(0x20).wrapping_sub(u16::from(relative_y) + 16),
        );
        let state = self.r8(0xb0);
        self.w8(0x45a, 0);
        self.w8(0x458, 0);
        self.dungeon_load_and_draw_room();
        self.dungeon_load_custom_tile_attr();
        let theme = usize::from(self.r8(0xaa2));
        self.hooks
            .decompress_animated_dungeon_tiles(&mut self.ram, DUNG_ANIMATED_TILES[theme]);
        self.dungeon_load_attribute_table();
        self.w8(0xb0, state.wrapping_add(1));
        self.w8(0xaa4, 10);
        self.hooks.initialize_tilesets(&mut self.ram);
        self.w8(0xab8, 10);
        self.hooks.dungeon_load_palettes(&mut self.ram);
        self.hooks.hud_restore_torch_background(&mut self.ram);
        self.w8(0x3a, 0);
        self.w8(0x3c, 0);
        self.dungeon_reset_torch_background_and_player();
        if self.r8(0x56) != 0 {
            self.hooks.load_gear_palettes_bunny(&mut self.ram);
        }
        self.w8(0x9b, 0x80);
        self.hooks.hud_refill_logic(&mut self.ram);
        self.module_pre_dungeon_set_ambient_sfx();
        self.w8(0x11, 7);
        self.dungeon_load_song_bank_if_needed();
    }

    pub fn dungeon_load_and_draw_room(&mut self) {
        let hdma = self.r8(0x9b);
        self.w8(0x9b, 0);
        self.dungeon_load_room();
        self.w8(0x418, 0);
        self.w8(0x200, 0);
        self.w8(0x45c, 0);
        while self.r8(0x45c) != 16 {
            self.tile_map_prep_not_water_on_tag();
            self.hooks.dungeon_nmi_upload_tilemap(&mut self.ram);
            self.dungeon_prepare_next_room_quadrant_upload();
            self.hooks.dungeon_nmi_upload_tilemap(&mut self.ram);
        }
        self.w8(0x9b, hdma);
        self.w8(0x17, 0);
        self.w8(0x200, 0);
        self.w8(0xb0, 0);
    }

    pub fn dungeon_load_entrance(&mut self) {
        self.w8(0x1b, 1);
        if self.r8(0x10a) != 0 {
            self.w8(0x10a, 0);
        }
        self.w16(0x11a, 0);
        self.w16(0x11c, 0);
        self.w16(0x10a, 0);
        let starting_point = self.r16(0xf3cc) == 4 || self.r16(0x4aa) != 0;
        let index = if starting_point {
            usize::from(self.r8(0xf3c8))
        } else {
            usize::from(self.r8(0x10e))
        };
        let entrance = self.hooks.dungeon_entrance(starting_point, index);
        if starting_point {
            self.w16(0x10e, entrance.entrance);
        }
        self.w16(0xa0, entrance.room);
        self.w16(0x48e, entrance.room);
        for address in [0xe6, 0xe8, 0x122, 0x124] {
            self.w16(address, entrance.scroll_y);
        }
        for address in [0xe0, 0xe2, 0x11e, 0x120] {
            self.w16(address, entrance.scroll_x);
        }
        if self.r16(0xf3c5) != 0 {
            self.w16(0x20, entrance.player_y);
            self.w16(0x22, entrance.player_x);
        }
        self.w16(0x618, entrance.camera_y);
        self.w16(0x61a, entrance.camera_y.wrapping_add(2));
        self.w16(0x61c, entrance.camera_x);
        self.w16(0x61e, entrance.camera_x.wrapping_add(2));
        self.w16(0xec, 0x1f8);
        self.w16(0x696, entrance.door_settings);
        self.w16(0x610, 0);
        self.w16(0x612, 0x110);
        self.w16(0x614, 0);
        self.w16(0x616, 0x100);
        self.w16(0x600, u16::from(entrance.relative_coords[0]) << 8);
        self.w16(0x602, u16::from(entrance.relative_coords[1]) << 8);
        self.w16(0x604, u16::from(entrance.relative_coords[2]) << 8 | 0x10);
        self.w16(0x606, u16::from(entrance.relative_coords[3]) << 8 | 0x10);
        self.w16(0x608, u16::from(entrance.relative_coords[4]) << 8);
        self.w16(0x60a, u16::from(entrance.relative_coords[5]) << 8);
        self.w16(0x60c, u16::from(entrance.relative_coords[6]) << 8);
        self.w16(0x60e, u16::from(entrance.relative_coords[7]) << 8);
        self.w8(
            0x2f,
            if starting_point || matches!(index, 0 | 0x43) {
                2
            } else {
                0
            },
        );
        self.w8(0xaa1, entrance.blockset);
        self.w8(0xa4, entrance.floor);
        self.w8(0x40c, entrance.palace);
        self.w8(0x6c, entrance.doorway_orientation);
        self.w8(0xee, entrance.starting_bg >> 4);
        self.w8(0x476, entrance.starting_bg & 0xf);
        self.w8(0xa6, entrance.quadrant1 >> 4);
        self.w8(0xa7, entrance.quadrant1 & 0xf);
        self.w8(0xa9, entrance.quadrant2 >> 4);
        self.w8(0xaa, entrance.quadrant2 & 0xf);
        if starting_point {
            self.w8(0x132, entrance.music);
            if index == 0 && self.r8(0xf3c5) == 0 {
                self.w8(0x132, 0xff);
            }
            self.w16(0x4aa, 0);
        } else {
            self.w16(0x698, 0);
            let mut music = entrance.music;
            if music == 3 && self.r8(0xf3c5) >= 2 {
                music = 18;
            }
            self.w8(0x132, music);
            if self.r16(0xa0) >= 0x100 {
                self.w8(0xa4, 0);
            }
        }
        self.w8(0x44, 0x80);
        self.w8(0x45, 0x80);
        self.w8(0x2c, 0xf);
        self.w8(0x2d, 0xf);
        self.w8(0x24, 0xff);
        self.w8(0x29, 0xff);
        self.ram.slice_mut(0xf940, 0x100).fill(0);
        self.ram.slice_mut(0xfb40, 0x280).fill(0);
        self.w16(0xc172, 0);
        self.w8(0x4bc, 0);
    }
}

const DUNG_BG2: usize = 0x2000;
const DUNG_BG1: usize = 0x4000;

const fn xy(x: usize, y: usize) -> usize {
    y * 64 + x
}

pub const BOSS_ROOMS: [u16; 12] = [200, 51, 7, 32, 6, 90, 41, 144, 222, 164, 172, 13];
pub const DUNGEON_EXIT_FROM: [u8; 12] = [200, 51, 7, 32, 6, 90, 41, 144, 222, 164, 172, 13];
pub const DUNGEON_EXIT_TO: [u8; 12] = [201, 99, 119, 32, 40, 74, 89, 152, 14, 214, 219, 13];
pub const OBJECT_SUBTYPE1_PARAMS: [u16; 256] = [
    0x3d8, 0x2e8, 0x2f8, 0x328, 0x338, 0x400, 0x410, 0x388, 0x390, 0x420, 0x42a, 0x434, 0x43e,
    0x448, 0x452, 0x45c, 0x466, 0x470, 0x47a, 0x484, 0x48e, 0x498, 0x4a2, 0x4ac, 0x4b6, 0x4c0,
    0x4ca, 0x4d4, 0x4de, 0x4e8, 0x4f2, 0x4fc, 0x506, 0x598, 0x600, 0x63c, 0x63c, 0x63c, 0x63c,
    0x63c, 0x642, 0x64c, 0x652, 0x658, 0x65e, 0x664, 0x66a, 0x688, 0x694, 0x6a8, 0x6a8, 0x6a8,
    0x6c8, 0x0, 0x78a, 0x7aa, 0xe26, 0x84a, 0x86a, 0x882, 0x8ca, 0x85a, 0x8fa, 0x91a, 0x920, 0x92a,
    0x930, 0x936, 0x93c, 0x942, 0x948, 0x94e, 0x96c, 0x97e, 0x98e, 0x902, 0x99e, 0x9d8, 0x9d8,
    0x9d8, 0x9fa, 0x156c, 0x1590, 0x1d86, 0x0, 0xa14, 0xa24, 0xa54, 0xa54, 0xa84, 0xa84, 0x14dc,
    0x1500, 0x61e, 0xe52, 0x600, 0x3d8, 0x2c8, 0x2d8, 0x308, 0x318, 0x3e0, 0x3f0, 0x378, 0x380,
    0x5fa, 0x648, 0x64a, 0x670, 0x67c, 0x6a8, 0x6a8, 0x6a8, 0x6c8, 0x0, 0x7aa, 0x7ca, 0x84a, 0x89a,
    0x8b2, 0x90a, 0x926, 0x928, 0x912, 0x9f8, 0x1d7e, 0x0, 0xa34, 0xa44, 0xa54, 0xa6c, 0xa84,
    0xa9c, 0x1524, 0x1548, 0x85a, 0x606, 0xe52, 0x5fa, 0x6a0, 0x6a2, 0xb12, 0xb14, 0x9b0, 0xb46,
    0xb56, 0x1f52, 0x1f5a, 0x288, 0xe82, 0x1df2, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x3d8, 0x3d8, 0x3d8, 0x3d8, 0x5aa, 0x5b2, 0x5b2, 0x5b2, 0x5b2, 0xe0, 0xe0, 0xe0, 0xe0, 0x110,
    0x0, 0x0, 0x6a4, 0x6a6, 0xae6, 0xb06, 0xb0c, 0xb16, 0xb26, 0xb36, 0x1f52, 0x1f5a, 0x288, 0xeba,
    0xe82, 0x1df2, 0x0, 0x0, 0x3d8, 0x510, 0x5aa, 0x5aa, 0x0, 0x168, 0xe0, 0x158, 0x100, 0x110,
    0x178, 0x72a, 0x72a, 0x72a, 0x75a, 0x670, 0x670, 0x130, 0x148, 0x72a, 0x72a, 0x72a, 0x75a,
    0xe0, 0x110, 0xf0, 0x110, 0x0, 0xab4, 0x8da, 0xade, 0x188, 0x1a0, 0x1b0, 0x1c0, 0x1d0, 0x1e0,
    0x1f0, 0x200, 0x120, 0x2a8, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
];
pub const OBJECT_SUBTYPE2_PARAMS: [u16; 64] = [
    0xb66, 0xb86, 0xba6, 0xbc6, 0xc66, 0xc86, 0xca6, 0xcc6, 0xbe6, 0xc06, 0xc26, 0xc46, 0xce6,
    0xd06, 0xd26, 0xd46, 0xd66, 0xd7e, 0xd96, 0xdae, 0xdc6, 0xdde, 0xdf6, 0xe0e, 0x398, 0x3a0,
    0x3a8, 0x3b0, 0xe32, 0xe26, 0xea2, 0xe9a, 0xeca, 0xed2, 0xede, 0xede, 0xf1e, 0xf3e, 0xf5e,
    0xf6a, 0xef6, 0xf72, 0xf92, 0xfa2, 0xfa2, 0x1088, 0x10a8, 0x10a8, 0x10c8, 0x10c8, 0x10c8,
    0x10c8, 0xe52, 0x1108, 0x1108, 0x12a8, 0x1148, 0x1160, 0x1178, 0x1190, 0x1458, 0x1488, 0x2062,
    0x2086,
];
pub const OBJECT_SUBTYPE3_PARAMS: [u16; 128] = [
    0x1614, 0x162c, 0x1654, 0xa0e, 0xa0c, 0x9fc, 0x9fe, 0xa00, 0xa02, 0xa04, 0xa06, 0xa08, 0xa0a,
    0, 0xa10, 0xa12, 0x1dda, 0x1de2, 0x1dd6, 0x1dea, 0x15fc, 0x1dfa, 0x1df2, 0x1488, 0x1494,
    0x149c, 0x14a4, 0x10e8, 0x10e8, 0x10e8, 0x11a8, 0x11c8, 0x11e8, 0x1208, 0x3b8, 0x3c0, 0x3c8,
    0x3d0, 0x1228, 0x1248, 0x1268, 0x1288, 0, 0xe5a, 0xe62, 0, 0, 0xe82, 0xe8a, 0x14ac, 0x14c4,
    0x10e8, 0x1614, 0x1614, 0x1614, 0x1614, 0x1614, 0x1614, 0x1cbe, 0x1cee, 0x1d1e, 0x1d4e, 0x1d8e,
    0x1d96, 0x1d9e, 0x1da6, 0x1dae, 0x1db6, 0x1dbe, 0x1dc6, 0x1dce, 0x220, 0x260, 0x280, 0x1f3a,
    0x1f62, 0x1f92, 0x1ff2, 0x2016, 0x1f42, 0xeaa, 0x1f4a, 0x1f52, 0x1f5a, 0x202e, 0x2062, 0x9b8,
    0x9c0, 0x9c8, 0x9d0, 0xfa2, 0xfb2, 0xfc4, 0xff4, 0x1018, 0x1020, 0x15b4, 0x15d8, 0x20f6, 0xeba,
    0x22e6, 0x22ee, 0x5da, 0x281e, 0x2ae0, 0x2d2a, 0x2f2a, 0x22f6, 0x2316, 0x232e, 0x2346, 0x235e,
    0x2376, 0x23b6, 0x1e9a, 0, 0x2436, 0x149c, 0x24b6, 0x24e6, 0x2516, 0x1028, 0x1040, 0x1060,
    0x1070, 0x1078, 0x1080, 0,
];
pub const DOOR_TYPE_SRC_DATA: [u16; 52] = [
    0x2716, 0x272e, 0x272e, 0x2746, 0x2746, 0x2746, 0x2746, 0x2746, 0x2746, 0x275e, 0x275e, 0x275e,
    0x275e, 0x2776, 0x278e, 0x27a6, 0x27be, 0x27be, 0x27d6, 0x27d6, 0x27ee, 0x2806, 0x2806, 0x281e,
    0x2836, 0x2836, 0x2836, 0x2836, 0x284e, 0x2866, 0x2866, 0x2866, 0x2866, 0x287e, 0x2896, 0x28ae,
    0x28c6, 0x28de, 0x28f6, 0x28f6, 0x28f6, 0x290e, 0x2926, 0x2958, 0x2978, 0x2990, 0x2990, 0x2990,
    0x2990, 0x29a8, 0x29c0, 0x29d8,
];
pub const DOOR_TYPE_SRC_DATA2: [u16; 48] = [
    0x29f0, 0x2a08, 0x2a08, 0x2a20, 0x2a20, 0x2a20, 0x2a20, 0x2a20, 0x2a20, 0x2a38, 0x2a38, 0x2a38,
    0x2a38, 0x2a50, 0x2a68, 0x2a80, 0x2a98, 0x2a98, 0x2a98, 0x2a98, 0x2a98, 0x2ab0, 0x2ac8, 0x2ae0,
    0x2af8, 0x2af8, 0x2af8, 0x2af8, 0x2b10, 0x2b28, 0x2b28, 0x2b28, 0x2b28, 0x2b40, 0x2b58, 0x2b70,
    0x2b88, 0x2ba0, 0x2bb8, 0x2bb8, 0x2bb8, 0x2bd0, 0x2be8, 0x2c1a, 0x2c3a, 0x2c52, 0x2c6a, 0x2c6a,
];
pub const DOOR_TYPE_SRC_DATA3: [u16; 48] = [
    0x2c6a, 0x2c82, 0x2c82, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2cb2, 0x2cb2, 0x2cb2,
    0x2cb2, 0x2cca, 0x2ce2, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2d12, 0x2d12, 0x2d2a,
    0x2d42, 0x2d42, 0x2d42, 0x2d42, 0x2d5a, 0x2d72, 0x2d72, 0x2d72, 0x2d72, 0x2d8a, 0x2da2, 0x2dba,
    0x2dd2, 0x2dea, 0x2e02, 0x2e02, 0x2e02, 0x2e1a, 0x2e32, 0x2e32, 0x2e52, 0x2e6a, 0x2e6a, 0x2e6a,
];
pub const DOOR_TYPE_SRC_DATA4: [u16; 47] = [
    0x2e6a, 0x2e82, 0x2e82, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2eb2, 0x2eb2, 0x2eb2,
    0x2eb2, 0x2eca, 0x2ee2, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2f12, 0x2f12, 0x2f2a,
    0x2f42, 0x2f42, 0x2f42, 0x2f42, 0x2f5a, 0x2f72, 0x2f72, 0x2f72, 0x2f72, 0x2f8a, 0x2fa2, 0x2fba,
    0x2fd2, 0x2fea, 0x3002, 0x3002, 0x3002, 0x301a, 0x3032, 0x3032, 0x3052, 0x306a, 0x306a,
];
pub const DUNG_PAL_INFOS: [DungPalInfo; 41] = [
    DungPalInfo {
        pal0: 0,
        pal1: 0,
        pal2: 3,
        pal3: 1,
    },
    DungPalInfo {
        pal0: 2,
        pal1: 0,
        pal2: 3,
        pal3: 1,
    },
    DungPalInfo {
        pal0: 4,
        pal1: 0,
        pal2: 10,
        pal3: 1,
    },
    DungPalInfo {
        pal0: 6,
        pal1: 0,
        pal2: 1,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 10,
        pal1: 2,
        pal2: 2,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 4,
        pal1: 4,
        pal2: 3,
        pal3: 10,
    },
    DungPalInfo {
        pal0: 12,
        pal1: 5,
        pal2: 8,
        pal3: 20,
    },
    DungPalInfo {
        pal0: 14,
        pal1: 0,
        pal2: 3,
        pal3: 10,
    },
    DungPalInfo {
        pal0: 2,
        pal1: 0,
        pal2: 15,
        pal3: 20,
    },
    DungPalInfo {
        pal0: 10,
        pal1: 2,
        pal2: 0,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 2,
        pal1: 0,
        pal2: 15,
        pal3: 12,
    },
    DungPalInfo {
        pal0: 6,
        pal1: 0,
        pal2: 6,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 0,
        pal1: 0,
        pal2: 14,
        pal3: 18,
    },
    DungPalInfo {
        pal0: 18,
        pal1: 5,
        pal2: 5,
        pal3: 11,
    },
    DungPalInfo {
        pal0: 18,
        pal1: 0,
        pal2: 2,
        pal3: 12,
    },
    DungPalInfo {
        pal0: 16,
        pal1: 5,
        pal2: 10,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 16,
        pal1: 0,
        pal2: 16,
        pal3: 12,
    },
    DungPalInfo {
        pal0: 22,
        pal1: 7,
        pal2: 2,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 22,
        pal1: 0,
        pal2: 7,
        pal3: 15,
    },
    DungPalInfo {
        pal0: 8,
        pal1: 0,
        pal2: 4,
        pal3: 12,
    },
    DungPalInfo {
        pal0: 8,
        pal1: 0,
        pal2: 4,
        pal3: 9,
    },
    DungPalInfo {
        pal0: 4,
        pal1: 0,
        pal2: 3,
        pal3: 1,
    },
    DungPalInfo {
        pal0: 20,
        pal1: 0,
        pal2: 4,
        pal3: 4,
    },
    DungPalInfo {
        pal0: 20,
        pal1: 0,
        pal2: 20,
        pal3: 12,
    },
    DungPalInfo {
        pal0: 24,
        pal1: 5,
        pal2: 7,
        pal3: 11,
    },
    DungPalInfo {
        pal0: 24,
        pal1: 6,
        pal2: 16,
        pal3: 12,
    },
    DungPalInfo {
        pal0: 26,
        pal1: 5,
        pal2: 8,
        pal3: 20,
    },
    DungPalInfo {
        pal0: 26,
        pal1: 2,
        pal2: 0,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 6,
        pal1: 0,
        pal2: 3,
        pal3: 10,
    },
    DungPalInfo {
        pal0: 28,
        pal1: 0,
        pal2: 3,
        pal3: 1,
    },
    DungPalInfo {
        pal0: 30,
        pal1: 0,
        pal2: 11,
        pal3: 17,
    },
    DungPalInfo {
        pal0: 4,
        pal1: 0,
        pal2: 11,
        pal3: 17,
    },
    DungPalInfo {
        pal0: 14,
        pal1: 0,
        pal2: 0,
        pal3: 2,
    },
    DungPalInfo {
        pal0: 32,
        pal1: 8,
        pal2: 19,
        pal3: 13,
    },
    DungPalInfo {
        pal0: 10,
        pal1: 0,
        pal2: 3,
        pal3: 10,
    },
    DungPalInfo {
        pal0: 20,
        pal1: 0,
        pal2: 4,
        pal3: 4,
    },
    DungPalInfo {
        pal0: 26,
        pal1: 2,
        pal2: 2,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 26,
        pal1: 10,
        pal2: 0,
        pal3: 0,
    },
    DungPalInfo {
        pal0: 0,
        pal1: 0,
        pal2: 3,
        pal3: 2,
    },
    DungPalInfo {
        pal0: 14,
        pal1: 0,
        pal2: 3,
        pal3: 7,
    },
    DungPalInfo {
        pal0: 26,
        pal1: 5,
        pal2: 5,
        pal3: 11,
    },
];
pub const DOOR_POSITION_TO_TILEMAP_OFFS_UP: [u16; 12] = [
    0x21c, 0x23c, 0x25c, 0x39c, 0x3bc, 0x3dc, 0x121c, 0x123c, 0x125c, 0x139c, 0x13bc, 0x13dc,
];
pub const DOOR_POSITION_TO_TILEMAP_OFFS_DOWN: [u16; 12] = [
    0xd1c, 0xd3c, 0xd5c, 0xb9c, 0xbbc, 0xbdc, 0x1d1c, 0x1d3c, 0x1d5c, 0x1b9c, 0x1bbc, 0x1bdc,
];
pub const DOOR_POSITION_TO_TILEMAP_OFFS_LEFT: [u16; 12] = [
    0x784, 0xf84, 0x1784, 0x78a, 0xf8a, 0x178a, 0x7c4, 0xfc4, 0x17c4, 0x7ca, 0xfca, 0x17ca,
];
pub const DOOR_POSITION_TO_TILEMAP_OFFS_RIGHT: [u16; 12] = [
    0x7b4, 0xfb4, 0x17b4, 0x7ae, 0xfae, 0x17ae, 0x7f4, 0xff4, 0x17f4, 0x7ee, 0xfee, 0x17ee,
];
pub const DOOR_TYPE_REMAP: [u8; 40] = [
    0, 2, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 80, 0, 80, 80, 96, 98, 100, 102, 82, 90, 80, 82, 84, 86,
    0, 80, 80, 0, 0, 0, 64, 88, 88, 0, 88, 88, 0, 0,
];
pub const SPIRAL_TAB1: [i8; 8] = [0, 1, 1, -1, 1, 1, 1, 1];
pub const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
pub const CAMERA_BOUNDS_X: [u16; 4] = [127, 383, 127, 383];
pub const CAMERA_BOUNDS_Y: [u16; 4] = [120, 376, 136, 392];
pub const WATERGATE_SRCS1: [u16; 4] = [0x12f8, 0x1348, 0x1398, 0x13e8];
pub const BLAST_WALL_TAB2: [u16; 16] = [
    4, 8, 0xc, 0x10, 0x14, 0x18, 0x1c, 0x20, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x700, 0x800,
];
pub const BLAST_WALL_TAB0: [u8; 5] = [4, 6, 0, 0, 2];
pub const BLAST_WALL_TAB1: [u16; 5] = [0, 0xa, 0, 0, 0x280];
pub const MOVING_WALL_TAB1: [i16; 8] = [-63, -127, -191, -255, -71, -135, -199, -263];
pub const MOVING_WALL_TAB0: [u16; 8] = [0x42, 0x82, 0xc2, 0x102, 0x4a, 0x8a, 0xca, 0x10a];
pub const DUNGEON_CRYSTAL_PENDANT_BIT: [u8; 13] = [0, 0, 4, 2, 0, 16, 2, 1, 64, 4, 1, 32, 8];
pub const BOSS_FINISHED_FALLING_ITEM: [u8; 13] = [0, 0, 1, 2, 0, 6, 6, 6, 6, 6, 3, 6, 6];
pub const TURN_OFF_WATER_TAB0: [i8; 16] =
    [-1, -1, -1, 1, -1, -1, -1, 1, -1, -1, -1, 1, -1, -1, -1, 1];
pub const TURN_ON_WATER_TAB2: [i8; 4] = [1, 1, 1, -1];
pub const TURN_ON_WATER_TAB1: [i8; 4] = [1, 2, 1, -1];
pub const TURN_ON_WATER_TAB0: [i8; 4] = [1, -1, 1, -1];
pub const ADJACENT_DOOR_LOOKUP: [u16; 24] = [
    0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x61, 0x71, 0x81, 0x91, 0xa1, 0xb1, 0x02, 0x12, 0x22, 0x32,
    0x42, 0x52, 0x63, 0x73, 0x83, 0x93, 0xa3, 0xb3,
];
pub const ADJACENT_DOOR_LOOKUP2: [u16; 24] = [
    0x61, 0x71, 0x81, 0x91, 0xa1, 0xb1, 0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x63, 0x73, 0x83, 0x93,
    0xa3, 0xb3, 0x02, 0x12, 0x22, 0x32, 0x42, 0x52,
];
pub const HEADER_ADJUSTMENT: [i16; 2] = [256, -256];
pub const PUSHED_BLOCK_DIR_MASK: [u8; 4] = [0x8, 0x4, 0x2, 0x1];
pub const PUSH_BLOCK_TAB1: [u8; 4] = [0, 0, 0xe0, 0x20];
pub const PUSH_BLOCK_TAB2: [u8; 4] = [0xe0, 0x20, 0, 0];
pub const PUSH_BLOCK_A: [u16; 4] = [0, 0, 8, 8];
pub const PUSH_BLOCK_B: [u16; 4] = [15, 15, 23, 23];
pub const PUSH_BLOCK_D: [u16; 4] = [15, 15, 15, 15];
pub const PUSH_BLOCK_C: [u16; 4] = [0, 0, 0, 0];
pub const PUSH_BLOCK_E: [u16; 4] = [8, 24, 0, 16];
pub const PUSH_BLOCK_F: [u16; 4] = [15, 0, 15, 0];
pub const TELEPORT_PIT_LEVEL1: [i8; 3] = [0, 1, 1];
pub const TELEPORT_PIT_LEVEL2: [i8; 3] = [0, 0, 1];
pub const CRYSTAL_MAIDEN_PAL: [u16; 8] =
    [0, 0x3821, 0x4463, 0x54a5, 0x5ce7, 0x6d29, 0x79ad, 0x7e10];
pub const CRYSTAL_TAB0: [u16; 7] = [0x1618, 0x1658, 0x1658, 0x1618, 0x658, 0x1618, 0x1658];
pub const OPEN_GANON_DOOR_TAB: [u16; 4] = [0x2556, 0x2596, 0x25d6, 0x2616];
pub const ORIENT_LAMP_BG_TAB0: [u16; 4] = [0, 256, 0, 256];
pub const ORIENT_LAMP_BG_TAB1: [u16; 4] = [0, 0, 256, 256];
pub const ORIENT_LAMP_BG_TAB2: [i16; 4] = [52, -2, 56, 6];
pub const ORIENT_LAMP_BG_TAB3: [i16; 4] = [64, 64, 82, -176];
pub const ORIENT_LAMP_BG_TAB4: [i16; 4] = [128, 384, 160, 160];
pub const STAIRCASE_TAB2: [i8; 20] = [
    12, 32, 48, 56, 72, -44, -40, -64, -64, -88, 12, 24, 40, 48, 64, -28, -40, -56, -64, -80,
];
pub const STAIRCASE_TAB3: [i8; 4] = [4, -4, 4, -4];
pub const STAIRCASE_TAB4: [i8; 4] = [52, 52, 59, 58];
pub const STAIRCASE_TAB5: [i8; 4] = [32, -64, 32, -32];
pub const MOVING_WALL_SIZES0: [u8; 4] = [5, 7, 11, 15];
pub const MOVING_WALL_SIZES1: [u8; 4] = [8, 16, 24, 32];
pub const WATERGATE_LAYOUT: [u8; 17] = [
    0x1b, 0xa1, 0xc9, 0x51, 0xa1, 0xc9, 0x92, 0xa1, 0xc9, 0xa1, 0x33, 0xc9, 0xa1, 0x72, 0xc9, 0xff,
    0xff,
];
pub const CHEST_OPEN_MASKS: [u16; 6] = [0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000];
pub const LAYOUT_QUADRANT_FLAGS: [u8; 32] = [
    0xf, 0xf, 0xf, 0xf, 0xb, 0xb, 7, 7, 0xf, 0xb, 0xf, 7, 0xb, 0xf, 7, 0xf, 0xe, 0xd, 0xe, 0xd,
    0xf, 0xf, 0xe, 0xd, 0xe, 0xd, 0xf, 0xf, 0xa, 9, 6, 5,
];
pub const QUADRANT_VISITING_FLAGS: [u8; 16] = [
    8, 4, 2, 1, 0xc, 0xc, 3, 3, 0xa, 5, 0xa, 5, 0xf, 0xf, 0xf, 0xf,
];
pub const DUNGEON_MINIGAME_CHEST_PRIZES1: [u8; 8] =
    [0x40, 0x41, 0x34, 0x42, 0x43, 0x44, 0x27, 0x17];
pub const DUNGEON_RUPEE_CHEST_MINIGAME_PRIZES: [u8; 32] = [
    0x47, 0x34, 0x46, 0x34, 0x46, 0x46, 0x34, 0x47, 0x46, 0x47, 0x34, 0x46, 0x47, 0x34, 0x46, 0x47,
    0x34, 0x47, 0x41, 0x47, 0x41, 0x41, 0x47, 0x34, 0x41, 0x34, 0x47, 0x41, 0x34, 0x47, 0x41, 0x34,
];
pub const DUNGEON_QUERY_IF_TILE_LIFTABLE_X: [i8; 4] = [7, 7, -3, 16];
pub const DUNGEON_QUERY_IF_TILE_LIFTABLE_Y: [i8; 4] = [3, 24, 14, 14];
pub const DUNGEON_QUERY_IF_TILE_LIFTABLE_RV: [u16; 16] = [
    0x5252, 0x5050, 0x5454, 0, 0x2323, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const DOOR_BLAST_WALL_UP_DSTS: [u16; 6] = [0xd8a, 0xdaa, 0xdca, 0x2b6, 0xab6, 0x12b6];
pub const DUNG_ANIMATED_TILES: [u8; 24] = [
    0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5f, 0x5d, 0x5f, 0x5f, 0x5e, 0x5f, 0x5e, 0x5e, 0x5d,
    0x5d, 0x5e, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d,
];
pub const DUNGEON_DRAW_OBJECT_OFFSETS_BG1: [u8; 33] = [
    0, 0x20, 0x7e, 2, 0x20, 0x7e, 4, 0x20, 0x7e, 6, 0x20, 0x7e, 0x80, 0x20, 0x7e, 0x82, 0x20, 0x7e,
    0x84, 0x20, 0x7e, 0x86, 0x20, 0x7e, 0, 0x21, 0x7e, 0x80, 0x21, 0x7e, 0, 0x22, 0x7e,
];
pub const DUNGEON_DRAW_OBJECT_OFFSETS_BG2: [u8; 33] = [
    0, 0x40, 0x7e, 2, 0x40, 0x7e, 4, 0x40, 0x7e, 6, 0x40, 0x7e, 0x80, 0x40, 0x7e, 0x82, 0x40, 0x7e,
    0x84, 0x40, 0x7e, 0x86, 0x40, 0x7e, 0, 0x41, 0x7e, 0x80, 0x41, 0x7e, 0, 0x42, 0x7e,
];
pub const UPLOAD_BG_SRCS: [u16; 16] = [
    0, 0x1000, 0, 0x40, 0x40, 0x1040, 0x1000, 0x1040, 0x1000, 0, 0x40, 0, 0x1040, 0x40, 0x1040,
    0x1000,
];
pub const UPLOAD_BG_DSTS: [u8; 16] = [1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15, 4, 8, 12, 16];
pub const TILE_ATTRS_BY_DOOR: [u16; 40] = [
    0x8080, 0x8484, 0, 0x101, 0x8484, 0x8e8e, 0, 0, 0x8888, 0x8e8e, 0x8080, 0x8080, 0x8282, 0x8080,
    0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8282, 0x8e8e, 0x8080, 0x8282, 0x8080, 0x8080,
    0x8080, 0x8282, 0x8282, 0x8080, 0x8080, 0x8080, 0x8484, 0x8484, 0x8686, 0x8888, 0x8686, 0x8686,
    0x8080, 0x8080,
];
pub const PUSH_BLOCK_MOVE_DISTANCES: [i16; 4] = [-0x100, 0x100, -4, 4];
pub const PREP_SPRITE_INDUCED_DMA_SRCS: [u16; 10] = [
    0xe0, 0xade, 0x5aa, 0x198, 0x210, 0x218, 0x1f3a, 0xeaa, 0xeb2, 0x140,
];
pub const DOOR_ANIM_UP_SRC: [u16; 5] = [0x306a, 0x306a, 0x3082, 0x309a, 0x30b2];
pub const DOOR_ANIM_DOWN_SRC: [u16; 5] = [0x30b2, 0x30ca, 0x30e2, 0x30fa, 0x3112];
pub const DOOR_ANIM_LEFT_SRC: [u16; 5] = [0x3112, 0x312a, 0x3142, 0x315a, 0x3172];
pub const DOOR_ANIM_RIGHT_SRC: [u16; 5] = [0x3172, 0x318a, 0x31a2, 0x31ba, 0x31d2];
pub const SPIRAL_STAIRCASE_X: [i8; 4] = [-28, -28, 24, 24];
pub const SPIRAL_STAIRCASE_Y: [i8; 4] = [16, -10, -10, -32];

const DUNGEON_QUADRANT_OFFSETS: [u16; 4] = [0, 0x40, 0x1000, 0x1040];
const UPPER_BITMASKS: [u16; 16] = [
    0x8000, 0x4000, 0x2000, 0x1000, 0x800, 0x400, 0x200, 0x100, 0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1,
];
pub const UP_DOWN_SCROLL_X: [u16; 4] = [0, 256, 256, 0];
pub const UP_DOWN_SCROLL_Y: [u16; 4] = [0, 272, 256, 16];

#[cfg(test)]
mod tests {
    use super::*;

    struct Hooks;
    impl DungeonHooks for Hooks {}

    struct AssetHooks {
        words: [u16; 1024],
    }

    impl DungeonHooks for AssetHooks {
        fn dungeon_object_data(&self, word_offset: usize) -> &[u16] {
            &self.words[word_offset..]
        }
    }

    struct LargeAssetHooks {
        words: Vec<u16>,
    }

    impl DungeonHooks for LargeAssetHooks {
        fn dungeon_object_data(&self, word_offset: usize) -> &[u16] {
            &self.words[word_offset..]
        }
    }

    struct AttributeHooks;
    impl DungeonHooks for AttributeHooks {
        fn dungeon_tile_attribute(&self, _ram: &Ram<'_>, tile: usize) -> u8 {
            [0x12, 0x20][tile]
        }
    }

    #[derive(Default)]
    struct RuntimeHooks {
        overworld_bombs: Vec<(u16, u16)>,
        landings: usize,
        filters: usize,
        mosaics: usize,
    }

    impl DungeonHooks for RuntimeHooks {
        fn overworld_bomb_tiles_32x32(&mut self, _ram: &mut Ram<'_>, x: u16, y: u16) {
            self.overworld_bombs.push((x, y));
        }
        fn handle_dungeon_landing_from_pit(&mut self, _ram: &mut Ram<'_>) {
            self.landings += 1;
        }
        fn apply_palette_filter_bounce(&mut self, _ram: &mut Ram<'_>) {
            self.filters += 1;
        }
        fn conditional_mosaic_control(&mut self, _ram: &mut Ram<'_>) {
            self.mosaics += 1;
        }
    }

    struct EntranceHooks;
    impl DungeonHooks for EntranceHooks {
        fn dungeon_entrance(&self, starting_point: bool, index: usize) -> DungeonEntrance {
            assert!(!starting_point);
            assert_eq!(index, 3);
            DungeonEntrance {
                room: 0x123,
                scroll_y: 0x200,
                scroll_x: 0x400,
                player_y: 0x250,
                player_x: 0x450,
                camera_y: 0x88,
                camera_x: 0x99,
                door_settings: 0x55aa,
                relative_coords: [1, 2, 3, 4, 5, 6, 7, 8],
                blockset: 9,
                floor: 4,
                palace: 6,
                doorway_orientation: 2,
                starting_bg: 0x10,
                quadrant1: 0x21,
                quadrant2: 0x12,
                music: 7,
                ..DungeonEntrance::default()
            }
        }
    }

    #[test]
    fn all_object_parser_switches_to_door_stream_at_fff0() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.room_draw_draw_all_objects(&[0xf0, 0xff, 0xff, 0xff]);
        assert_eq!(dungeon.r16(0xba), 2);
        assert_eq!((dungeon.r16(0xb2), dungeon.r16(0xb4)), (0, 0));
    }

    #[test]
    fn bombs_delegate_outdoors_without_touching_dungeon_state() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = RuntimeHooks::default();
        {
            let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
            dungeon.w8(0x10, 9);
            dungeon.bomb_check_for_destructibles(0x123, 0x456, 2);
            assert_eq!(dungeon.r8(0x11), 0);
        }
        assert_eq!(hooks.overworld_bombs, [(0x123, 0x456)]);
    }

    #[test]
    fn warp_dispatch_uses_exact_state_one_mosaic_filter_pair() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = RuntimeHooks::default();
        {
            let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
            dungeon.w8(0xb0, 1);
            dungeon.module07_15_warp_pad();
            assert_eq!(dungeon.r8(0xb0), 1);
        }
        assert_eq!((hooks.mosaics, hooks.filters), (1, 1));
    }

    #[test]
    fn falling_entrance_state_four_falls_through_to_landing() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = RuntimeHooks::default();
        {
            let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
            dungeon.w8(0xb0, 4);
            dungeon.w8(0x13, 13);
            dungeon.w8(0x11, 1);
            dungeon.module11_dungeon_falling_entrance();
            assert_eq!(dungeon.r8(0x13), 14);
        }
        assert_eq!(hooks.landings, 1);
    }

    #[test]
    fn entrance_record_populates_all_packed_room_geometry() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = EntranceHooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w8(0x10e, 3);
        dungeon.w8(0xf3c5, 1);
        dungeon.dungeon_load_entrance();
        assert_eq!((dungeon.r16(0xa0), dungeon.r16(0x48e)), (0x123, 0x123));
        assert_eq!((dungeon.r16(0xe8), dungeon.r16(0xe2)), (0x200, 0x400));
        assert_eq!((dungeon.r16(0x20), dungeon.r16(0x22)), (0x250, 0x450));
        assert_eq!((dungeon.r16(0x618), dungeon.r16(0x61c)), (0x88, 0x99));
        assert_eq!((dungeon.r16(0x600), dungeon.r16(0x606)), (0x100, 0x410));
        assert_eq!((dungeon.r16(0x608), dungeon.r16(0x60e)), (0x500, 0x800));
        assert_eq!((dungeon.r8(0xa6), dungeon.r8(0xa7)), (2, 1));
        assert_eq!((dungeon.r8(0xa9), dungeon.r8(0xaa)), (1, 2));
        assert_eq!(dungeon.r8(0x132), 7);
    }

    #[test]
    fn subtype1_ceiling_draws_one_rightward_2x2_unit() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = AssetHooks { words: [0; 1024] };
        hooks.words[0x3d8 / 2..0x3d8 / 2 + 4].copy_from_slice(&[1, 2, 3, 4]);
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0xb2, 0);
        dungeon.w16(0xb4, 1);
        dungeon.load_type1_object_subtype1(0x00, DUNG_BG2, 0);
        assert_eq!(dungeon.r16(DUNG_BG2), 1);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(0, 1) * 2), 2);
        assert_eq!(dungeon.r16(DUNG_BG2 + 2), 3);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(1, 1) * 2), 4);
    }

    #[test]
    fn subtype1_low_wall_mirrors_tiles_to_both_backgrounds() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = AssetHooks { words: [0; 1024] };
        hooks.words[0x328 / 2..0x328 / 2 + 8].copy_from_slice(&[10, 11, 12, 13, 14, 15, 16, 17]);
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0xb2, 0);
        dungeon.w16(0xb4, 0);
        dungeon.load_type1_object_subtype1(0x03, DUNG_BG2, 9);
        for x in 0..2 {
            for y in 0..4 {
                let offset = 9 + xy(x, y);
                let expected = 10 + (x * 4 + y) as u16;
                assert_eq!(dungeon.r16(DUNG_BG1 + offset * 2), expected);
                assert_eq!(dungeon.r16(DUNG_BG2 + offset * 2), expected);
            }
        }
    }

    #[test]
    fn subtype1_edge_preserves_protected_first_tile() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = AssetHooks { words: [0; 1024] };
        hooks.words[0x63c / 2..0x63c / 2 + 3].copy_from_slice(&[20, 21, 22]);
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0xb2, 0);
        dungeon.w16(0xb4, 0);
        dungeon.w16(DUNG_BG2, 0x1db);
        dungeon.load_type1_object_subtype1(0x23, DUNG_BG2, 0);
        assert_eq!(dungeon.r16(DUNG_BG2), 0x1db);
        assert_eq!(dungeon.r16(DUNG_BG2 + 2), 21);
        assert_eq!(dungeon.r16(DUNG_BG2 + 4), 22);
    }

    #[test]
    fn subtype3_closed_water_face_switches_to_open_asset_from_saved_room_state() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = LargeAssetHooks {
            words: vec![0; 0x1800],
        };
        for (index, word) in hooks.words[0x1614 / 2..0x1614 / 2 + 12]
            .iter_mut()
            .enumerate()
        {
            *word = 0x100 + index as u16;
        }
        for (index, word) in hooks.words[0x162c / 2..0x162c / 2 + 20]
            .iter_mut()
            .enumerate()
        {
            *word = 0x200 + index as u16;
        }
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w8(0xaf, 27);
        dungeon.w16(0xa0, 3);
        dungeon.w16(0xf000 + 3 * 2, 0x100);
        dungeon.w16(0x47c, 0xbeef);
        dungeon.load_type1_object_subtype3(0x00, DUNG_BG2, 0x123);
        assert_eq!(dungeon.r16(DUNG_BG2), 0x200);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(3, 4) * 2), 0x213);
        assert_eq!(dungeon.r16(0x47c), 0xbeef);
    }

    #[test]
    fn subtype3_chest_keeps_location_when_tag_blocks_drawing() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = LargeAssetHooks {
            words: vec![0; 0x1800],
        };
        hooks.words[0x149c / 2..0x149c / 2 + 4].copy_from_slice(&[1, 2, 3, 4]);
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w8(0xae, 0x27);
        dungeon.w16(0xbf, 0x4000);
        dungeon.load_type1_object_subtype3(0x19, DUNG_BG2, 0x20);
        assert_eq!(dungeon.r16(0x496), 2);
        assert_eq!(dungeon.r16(0x498), 2);
        assert_eq!(dungeon.r16(0x6e0), 0x2040);
        assert_eq!(dungeon.r8(0xae), 0x27);
        assert_eq!(dungeon.r16(DUNG_BG2), 0);
    }

    #[test]
    fn subtype3_lower_up_stairs_propagate_counters_and_prioritize_row_above() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = LargeAssetHooks {
            words: vec![0; 0x1800],
        };
        for (index, word) in hooks.words[0x1228 / 2..0x1228 / 2 + 16]
            .iter_mut()
            .enumerate()
        {
            *word = 0x300 + index as u16;
        }
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0xbf, 0x4000);
        dungeon.w16(0x4a2, 4);
        for row in 0..4 {
            dungeon.set_bg2(0x40 + xy(0, row), 0x123);
        }
        dungeon.load_type1_object_subtype3(0x26, DUNG_BG1 + 0x140 * 2, 0x140);
        assert_eq!(dungeon.r16(0x6b0 + 4), 0x1140);
        for address in [0x4a2, 0x4a4, 0x43a, 0x480, 0x484, 0x4a6, 0x4a8] {
            assert_eq!(dungeon.r16(address), 6);
        }
        assert_eq!(dungeon.r16(DUNG_BG1 + 0x140 * 2), 0x300);
        assert_eq!(dungeon.r16(DUNG_BG2 + 0x140 * 2), 0x300);
        assert_eq!(dungeon.r16(DUNG_BG2 + (0x40 + xy(0, 3)) * 2), 0x2123);
    }

    #[test]
    fn subtype3_large_bg1_object_uses_column_major_source_and_trim() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = LargeAssetHooks {
            words: vec![0; 0x1800],
        };
        for (index, word) in hooks.words[0x20f6 / 2..0x20f6 / 2 + 242]
            .iter_mut()
            .enumerate()
        {
            *word = index as u16 + 1;
        }
        hooks.words[0x22da / 2..0x22da / 2 + 6]
            .copy_from_slice(&[0xa0, 0xa1, 0xa2, 0xb0, 0xb1, 0xb2]);
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.load_type1_object_subtype3(0x62, DUNG_BG1, 0);
        assert_eq!(dungeon.r16(DUNG_BG1 + xy(0, 10) * 2), 11);
        assert_eq!(dungeon.r16(DUNG_BG1 + xy(21, 10) * 2), 242);
        assert_eq!(dungeon.r16(DUNG_BG1 + xy(11, 11) * 2), 0xa2);
        assert_eq!(dungeon.r16(DUNG_BG1 + xy(11, 12) * 2), 0xb2);
    }

    #[test]
    fn object_draw_5x4_uses_sixty_four_word_rows() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        let src = core::array::from_fn::<_, 20, _>(|i| 0x1000 + i as u16);
        dungeon.object_draw_5x4(&src, DUNG_BG2);
        assert_eq!(dungeon.r16(DUNG_BG2), 0x1000);
        assert_eq!(dungeon.r16(DUNG_BG2 + 3 * 2), 0x1003);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(0, 4) * 2), 0x1010);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(3, 4) * 2), 0x1013);
    }

    #[test]
    fn both_background_draw_writes_identical_words() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.object_draw_4x2_both_bgs(&[1, 2, 3, 4, 5, 6, 7, 8], 9);
        for row in 0..2 {
            for column in 0..4 {
                let offset = 9 + xy(column, row);
                assert_eq!(
                    dungeon.r16(DUNG_BG1 + offset * 2),
                    dungeon.r16(DUNG_BG2 + offset * 2)
                );
                assert_eq!(
                    dungeon.r16(DUNG_BG1 + offset * 2),
                    (row * 4 + column + 1) as u16
                );
            }
        }
    }

    #[test]
    fn chest_platform_preserves_postincrement_destination_quirk() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.ram.write_u8(0xb2, 2);
        let mut src = [0; 19];
        src[0] = 1;
        src[3] = 2;
        src[6] = 3;
        src[9] = 4;
        src[12] = 5;
        src[15] = 6;
        src[18] = 7;
        dungeon.object_chest_platform_helper(&src, 0);
        for (offset, expected) in [
            (0, 1),
            (1, 2),
            (2, 2),
            (3, 3),
            (4, 4),
            (8, 5),
            (9, 6),
            (10, 6),
            (11, 7),
        ] {
            assert_eq!(dungeon.r16(DUNG_BG2 + offset * 2), expected);
        }
    }

    #[test]
    fn many_32x32_blocks_repeat_the_source_two_rows_vertically() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.room_draw_a_many_32x32_blocks(2, &[1, 2, 3, 4, 5, 6, 7, 8], DUNG_BG1);
        for row in 0..4 {
            for column in 0..8 {
                assert_eq!(
                    dungeon.r16(DUNG_BG1 + xy(column, row) * 2),
                    [1, 2, 3, 4, 5, 6, 7, 8][(row & 1) * 4 + (column & 3)]
                );
            }
        }
    }

    #[test]
    fn store_2x2_builds_four_vram_packets_and_parallel_attributes() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.dungeon_store_2x2(0x83, [0x1111, 0x2222, 0x3333, 0x4444], 0x8e);
        for (i, position) in [0x83u16, 0xc3, 0x84, 0xc4].into_iter().enumerate() {
            assert_eq!(
                dungeon.r16(0x1002 + i * 6),
                Dungeon::<Hooks>::dungeon_map_vram_addr(position)
            );
            assert_eq!(dungeon.r16(0x1004 + i * 6), 0x100);
            assert_eq!(dungeon.r8(0x12000 + usize::from(position)), 0x8e);
        }
        assert_eq!(dungeon.r16(0x101a), 0xffff);
        assert_eq!(dungeon.r16(0x1000), 24);
        assert_eq!(dungeon.r8(0x14), 1);
    }

    #[test]
    fn room_bounds_helpers_use_wrapping_snes_words() {
        let mut bounds = RoomBounds {
            a0: 0xff80,
            b0: 0x100,
            a1: 0x80,
            b1: 0xffff,
        };
        Dungeon::<Hooks>::room_bounds_add_a(&mut bounds);
        Dungeon::<Hooks>::room_bounds_sub_b(&mut bounds);
        assert_eq!(
            bounds,
            RoomBounds {
                a0: 0x80,
                b0: 0xff00,
                a1: 0x180,
                b1: 0xfdff
            }
        );
    }

    #[test]
    fn object_hole_overwrites_fill_with_rom_edges() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = AssetHooks { words: [0; 1024] };
        hooks.words[0x63c / 2..0x63c / 2 + 6].copy_from_slice(&[10, 11, 12, 13, 14, 15]);
        hooks.words[0x648 / 2..0x648 / 2 + 2].copy_from_slice(&[16, 17]);
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0xb2, 0);
        dungeon.w16(0xb4, 0);
        dungeon.object_hole(&[9], DUNG_BG2);
        assert_eq!(dungeon.r16(0xb2), 4);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(0, 0) * 2), 10);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(3, 0) * 2), 12);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(0, 3) * 2), 13);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(3, 3) * 2), 15);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(0, 1) * 2), 16);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(3, 2) * 2), 17);
        assert_eq!(dungeon.r16(DUNG_BG2 + xy(1, 1) * 2), 9);
    }

    #[test]
    fn moved_wall_saved_state_clears_matching_tag_and_collision() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w8(0xaf, 0x1e);
        dungeon.w16(0x402, 0x800);
        dungeon.w8(0x46c, 7);
        dungeon.w8(0x414, 9);
        assert!(!dungeon.room_draw_check_if_wall_is_moved());
        assert_eq!(dungeon.r8(0xaf), 0);
        assert_eq!(dungeon.r8(0x46c), 0);
        assert_eq!(dungeon.r8(0x414), 0);
    }

    #[test]
    fn visited_quadrant_uses_packed_layout_index_and_persists_it() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0xa0, 3);
        dungeon.w8(0xa6, 2);
        dungeon.w8(0xa7, 2);
        dungeon.w8(0xa9, 1);
        dungeon.w8(0xaa, 0);
        dungeon.w16(0x408, 0x20);
        dungeon.w16(0xf000 + 3 * 2, 0x100);
        dungeon.set_and_save_visited_quadrant_flags();
        assert_eq!(dungeon.r16(0x408), 0x2f);
        assert_eq!(dungeon.r16(0xf000 + 3 * 2), 0x12f);
    }

    #[test]
    fn door_graphics_remaps_only_when_the_raw_door_mask_is_open() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0x1980 + 2 * 2, 0x0144);
        assert_eq!(dungeon.get_door_graphics_index(2, 2), 0x44);
        dungeon.w16(0x68c, UPPER_BITMASKS[2]);
        assert_eq!(dungeon.get_door_graphics_index(2, 2), 0x58);
    }

    #[test]
    fn basic_attributes_copy_flip_bits_only_for_directional_attributes() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = AttributeHooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(DUNG_BG2, 0x8000);
        dungeon.w16(DUNG_BG2 + 2, 0x4001);
        dungeon.dungeon_load_basic_attribute_full(1);
        assert_eq!(dungeon.r8(0x12000), 0x12 | 2);
        assert_eq!(dungeon.r8(0x12001), 0x20);
        assert_eq!(dungeon.r16(0xb2), 4);
        assert_eq!(dungeon.r16(0xb4), 2);
    }

    #[test]
    fn door_flagging_preserves_remap_trapdoor_and_invisible_paths() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0x68c, UPPER_BITMASKS[0]);
        assert_eq!(
            dungeon.room_draw_flag_doors_and_get_final_type(3, 0x48, 0x123),
            0x58
        );
        assert_eq!(dungeon.r16(0x19c0), 3);
        assert_eq!(dungeon.r16(0x19a0), 0x246);
        assert_eq!(dungeon.r16(0x1980), 0x48);
        assert_eq!(dungeon.r16(0x400), UPPER_BITMASKS[0]);
        assert_eq!(dungeon.r16(0x460), 2);

        dungeon.w16(0x468, 1);
        dungeon.w16(0x68c, UPPER_BITMASKS[1]);
        assert_eq!(
            dungeon.room_draw_flag_doors_and_get_final_type(0, 68, 7),
            68
        );

        dungeon.w16(0x68c, 0);
        assert_eq!(dungeon.room_draw_flag_doors_and_get_final_type(2, 26, 9), 0);
        assert_eq!(dungeon.r16(0x436), 0x0404);
        assert_eq!(dungeon.r16(0x68c), UPPER_BITMASKS[2]);
    }

    #[test]
    fn transition_landing_masks_attributes_and_moves_on_the_selected_axis() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0x20, 0x112);
        dungeon.w16(0x22, 0x108);
        let position = usize::from(
            ((0x112u16.wrapping_add(12) & 0x1f8) << 3) | ((0x108u16.wrapping_add(8) & 0x1f8) >> 3),
        );
        dungeon.w8(0x12000 + position, 0x84);
        assert_eq!(dungeon.calculate_transition_landing(), 3);
        assert_eq!(dungeon.r8(0x4e), 3);

        dungeon.w8(0x418, 2);
        dungeon.w8(0x4e, 0);
        dungeon.w16(0x22, 0x1234);
        let completed = dungeon.dungeon_transition_move_link_out_door();
        assert_eq!(dungeon.r16(0x22), 0x1236);
        assert_eq!(completed, (0x36 & 0xfe) == STAIRCASE_TAB2[10] as u8);
    }

    #[test]
    fn activated_water_attributes_rebuild_both_staircase_directions() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0x440, 2);
        dungeon.w16(0x49e, 2);
        dungeon.w16(0x6b8, 0x120);
        dungeon.w16(0x6ec, 0x320);
        dungeon.w8(0x11, 7);
        dungeon.w8(0x12, 7);
        dungeon.w8(0xb0, 7);
        dungeon.dungeon_set_attr_for_activated_water();
        assert_eq!(dungeon.r16(0x12000 + 0x120), 0x0003);
        assert_eq!(dungeon.r16(0x13000 + 0x120), 0x0a03);
        assert_eq!(dungeon.r16(0x12000 + 0x320 + xy(0, 3)), 0x0003);
        assert_eq!(dungeon.r16(0x13000 + 0x320 + xy(2, 3)), 0x030a);
        assert_eq!(
            (dungeon.r8(0x11), dungeon.r8(0x12), dungeon.r8(0xb0)),
            (0, 0, 0)
        );
    }

    #[test]
    fn water_vomit_builds_four_vertical_vram_packets_from_bg2() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0x47c, 0x40);
        let source = core::array::from_fn::<_, 20, _>(|index| 0x4000 + index as u16);
        dungeon.dungeon_adjust_water_vomit(&source, 5);
        let position = 0x20 + xy(0, 2);
        assert_eq!(dungeon.r16(DUNG_BG2 + position * 2), 0x4000);
        assert_eq!(dungeon.r16(DUNG_BG2 + (position + xy(3, 4)) * 2), 0x4013);
        assert_eq!(
            dungeon.r16(0x1002),
            Dungeon::<Hooks>::dungeon_map_vram_addr(position as u16)
        );
        assert_eq!(dungeon.r16(0x1004), 0x980);
        assert_eq!(dungeon.r16(0x1006), 0x4000);
        assert_eq!(dungeon.r16(0x1002 + 56), 0xffff);
        assert_eq!(dungeon.r8(0x14), 1);
    }

    #[test]
    fn camera_boundary_selection_tracks_direction_and_quadrant() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w8(0x67, 1);
        dungeon.w8(0xa9, 1);
        dungeon.handle_edge_transition_adjust_camera_boundaries(2);
        assert_eq!(dungeon.r8(0x418), 2);
        assert_eq!((dungeon.r16(0x61c), dungeon.r16(0x61e)), (383, 385));

        dungeon.w8(0x67, 8);
        dungeon.w8(0xaa, 0);
        dungeon.handle_edge_transition_adjust_camera_boundaries(3);
        assert_eq!((dungeon.r16(0x618), dungeon.r16(0x61a)), (136, 138));
    }

    #[test]
    fn falling_push_block_uses_signed_timeout_and_four_frame_cleanup() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w8(0x5fd, 4);
        dungeon.w8(0x5fc, 9);
        dungeon.w8(0x5fd, 4);
        dungeon.w8(0x500 + 3 * 2, 7);
        for expected in 1..=4 {
            dungeon.w8(0x2c4, 0);
            dungeon.push_block_handle_falling(6);
            assert_eq!(dungeon.r8(0x2c3), if expected == 4 { 0 } else { expected });
        }
        assert_eq!(dungeon.r8(0x500 + 3 * 2), 0);
        assert_eq!(dungeon.r8(0x5fc), 9);
        assert_eq!(dungeon.r8(0x5fd), 0);
    }

    #[test]
    fn exploding_wall_stripes_emit_two_vertical_packets_per_column() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut dungeon = Dungeon::new(&mut bytes, &mut hooks);
        dungeon.w16(0x454, 0);
        dungeon.w16(0x460, 0);
        dungeon.w16(0x19c0, 0);
        for column in 0..3 {
            for row in 0..12 {
                dungeon.set_bg2(100 + xy(column, row), (column * 0x100 + row) as u16);
            }
        }
        dungeon.clear_and_stripe_exploding_wall(100);
        assert_eq!(
            dungeon.r16(0x1100),
            Dungeon::<Hooks>::dungeon_map_vram_addr_no_swap(100)
        );
        assert_eq!(dungeon.r16(0x1102), 0xa81);
        assert_eq!(dungeon.r16(0x1104), 0);
        assert_eq!(dungeon.r16(0x1112), 5);
        assert_eq!(dungeon.r16(0x1100 + 3 * 32), 0xffff);
    }
}
