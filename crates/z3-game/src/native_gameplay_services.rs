//! Concrete cross-translation-unit routing for the native gameplay service.
//!
//! These implementations are the shared production surface for both the native
//! host and the direct parity verifier. Keeping the routing on
//! `NativeGameServices` ensures that verification exercises the same callback
//! composition as the shipped runtime.

use crate::{
    ancilla::{AncillaHitBox, AncillaRuntime},
    integration::NativeGameServices,
    player::PlayerRuntime,
    ram::Ram,
    sprite::{
        DrawMultipleData, PrepOamCoordsRet, ProjectSpeedRet, Sprite, SpriteHitBox, SpriteHooks,
        SpriteSpawnInfo,
    },
    sprite_main_bank0::SpriteMainBank0Runtime,
};

fn with_gfx(
    services: &mut NativeGameServices<'_>,
    ram: &mut Ram<'_>,
    action: impl FnOnce(
        &mut crate::load_gfx::LoadGfx<'_, crate::assets::RuntimeAssets<'_>, NativeGameServices<'_>>,
    ),
) {
    let assets = services.assets;
    let mut scratch_vram = std::mem::replace(
        &mut services.scratch_vram,
        Box::new([0; z3_core::VRAM_WORDS]),
    );
    {
        let mut gfx = crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: scratch_vram.as_mut(),
            assets,
            hooks: services,
        };
        action(&mut gfx);
    }
    services.scratch_vram = scratch_vram;
}

fn with_dungeon(
    services: &mut NativeGameServices<'_>,
    ram: &mut Ram<'_>,
    action: impl FnOnce(&mut crate::dungeon::Dungeon<'_, '_, NativeGameServices<'_>>),
) {
    let mut dungeon = crate::dungeon::Dungeon {
        ram: Ram::new(ram.bytes_mut()),
        hooks: services,
    };
    action(&mut dungeon);
}

impl crate::tile_detect::TileDetectHooks for NativeGameServices<'_> {
    fn map16_to_map8(&self, index: usize) -> u16 {
        crate::overworld::OverworldAssets::map16_to_map8(self.assets)[index]
    }

    fn map8_to_tile_attr(&self, index: usize) -> u8 {
        crate::overworld::OverworldAssets::map8_tile_attributes(self.assets)[index]
    }

    fn ancilla_x(&self, ram: &Ram<'_>, index: usize) -> u16 {
        u16::from(ram.read_u8(0xc04 + index)) | (u16::from(ram.read_u8(0xc18 + index)) << 8)
    }

    fn ancilla_y(&self, ram: &Ram<'_>, index: usize) -> u16 {
        u16::from(ram.read_u8(0xbfa + index)) | (u16::from(ram.read_u8(0xc0e + index)) << 8)
    }
}

// These are the concrete production cross-bank routes. The seam inventory test
// below guards against accidentally inheriting a default callback when these
// runtime traits evolve.
impl crate::sprite_main_bank1::SpriteMainBank1Runtime for NativeGameServices<'_> {
    fn draw_single_large(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_draw_single_large(k);
    }
    fn check_damage_to_link_same_layer(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link_same_layer(k)
    }
    fn move_y(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_y(k);
    }
    fn move_x(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_x(k);
    }
    fn sfx2_with_pan(&mut self, ram: &mut Ram<'_>, k: usize, sound: u8) {
        crate::misc::sprite_sfx2_with_pan(ram, self, k, sound);
    }
    fn link_cancel_dash(&mut self, ram: &mut Ram<'_>) {
        crate::sprite_main_bank2a::SpriteMainBank2aRuntime::link_cancel_dash(self, ram);
    }
    fn link_receive_item(&mut self, ram: &mut Ram<'_>, item: u8, method: i32) {
        ram.write_u8(0x2e9, method as u8);
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_receive_item(item, -1);
    }
    fn prep_oam(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<(u8, u8)> {
        let mut v = PrepOamCoordsRet::default();
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        (!s.sprite_prep_oam_coord_or_double_ret(k, &mut v)).then_some((v.x as u8, v.y as u8))
    }
    fn prep_oam_with_flags(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<(u8, u8, u8)> {
        let mut v = PrepOamCoordsRet::default();
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        (!s.sprite_prep_oam_coord_or_double_ret(k, &mut v))
            .then_some((v.x as u8, v.y as u8, v.flags))
    }
    fn nullify_hookshot_drag(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).sprite_nullify_hookshot_drag();
    }
    fn repel_dash(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .sprite_repel_dash();
    }
    fn behave_as_barrier(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_behave_as_barrier(k);
    }
    fn track_body_to_head(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_track_body_to_head(k);
    }
    fn show_solicited_message(&mut self, ram: &mut Ram<'_>, k: usize, msg: u16) -> u16 {
        Sprite::new(ram.bytes_mut(), self).sprite_show_solicited_message(k, msg)
    }
    fn show_unconditional_message(&mut self, ram: &mut Ram<'_>, msg: u16) {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_unconditional(msg);
    }
    fn fish_overlaps_merchant(&mut self, ram: &mut Ram<'_>, merchant: usize, fish: usize) -> bool {
        let mut hitbox = SpriteHitBox {
            r0_xlo: ram.read_u8(0xd10 + merchant),
            r8_xhi: ram.read_u8(0xd30 + merchant),
            r1_ylo: ram.read_u8(0xd00 + merchant),
            r9_yhi: ram.read_u8(0xd20 + merchant),
            r2: 16,
            r3: 16,
            ..SpriteHitBox::default()
        };
        let sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_setup_hit_box(fish, &mut hitbox);
        Sprite::<Self>::check_if_hit_boxes_overlap(&hitbox)
    }
    fn sfx3_with_pan(&mut self, ram: &mut Ram<'_>, k: usize, sound: u8) {
        crate::misc::sprite_sfx3_with_pan(ram, self, k, sound);
    }
    fn decode_animated_sprite_tile(&mut self, ram: &mut Ram<'_>, tile: u8) {
        with_gfx(self, ram, |g| g.decode_animated_sprite_tile_variable(tile));
    }
    fn sprite_init_bump_damage(&mut self, kind: u8) -> u8 {
        crate::sprite::SPRITE_INIT_BUMP_DAMAGE[usize::from(kind)]
    }
    fn load_properties(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_prep_load_properties(k);
    }
    fn load_follower_graphics(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.load_follower_graphics());
    }
    fn follower_initialize(&mut self, ram: &mut Ram<'_>) {
        crate::tagalong::initialize(ram, self);
    }
    fn active_cross_bank(&mut self, ram: &mut Ram<'_>, k: usize, kind: u8) {
        if (crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self })
            .dispatch_active(k, kind)
        {
            return;
        }
        let _ = (crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self })
            .dispatch_active(k, kind);
    }
    fn prep_cross_bank(&mut self, ram: &mut Ram<'_>, k: usize, kind: u8) {
        if (crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self })
            .dispatch_prep(k, kind)
        {
            return;
        }
        let _ = (crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self })
            .dispatch_prep(k, kind);
    }
    fn return_if_paused(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_paused(k)
    }
    fn apply_ricochet(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_apply_ricochet(k);
    }
    fn place_weapon_tink(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_place_weapon_tink(k);
    }
    fn garnish_spawn_sparkle(&mut self, ram: &mut Ram<'_>, k: usize, a: u8, b: u8) {
        let _ = Sprite::new(ram.bytes_mut(), self).sprite_garnish_spawn_sparkle(
            k,
            u16::from(a),
            u16::from(b),
        );
    }
    fn check_if_link_is_busy(&mut self, ram: &mut Ram<'_>) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_if_link_is_busy()
    }
    fn get_16bit_coords(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_get_16_bit_coords(k);
    }
    fn show_message_on_contact(&mut self, ram: &mut Ram<'_>, k: usize, msg: u16) -> u16 {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_on_contact(k, msg)
    }
    fn draw_single_small(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_draw_single_small(k);
    }
    fn link_reset_properties_a(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_reset_properties_a();
    }
    fn palette_assert_translucency_swap(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_assert_translucency_swap());
    }
    fn palette_filter_sp5f(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_filter_sp5f());
    }
    fn palette_filter_restore_sp5f(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_filter_restore_sp5f());
    }
    fn palette_revert_translucency_swap(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_revert_translucency_swap());
    }
    fn palette_restore_bg_and_hud(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_restore_bg_and_hud());
    }
    fn hud_refresh_icon(&mut self, ram: &mut Ram<'_>) {
        crate::hud::Hud { ram, runtime: self }.refresh_icon();
    }
    fn check_player_hitbox_overlap(
        &mut self,
        ram: &mut Ram<'_>,
        x: u16,
        y: u16,
        width: u8,
        height: u8,
    ) -> bool {
        let mut hitbox = SpriteHitBox {
            r4_spr_xlo: x as u8,
            r10_spr_xhi: (x >> 8) as u8,
            r5_spr_ylo: y as u8,
            r11_spr_yhi: (y >> 8) as u8,
            r6_spr_xsize: width,
            r7_spr_ysize: height,
            ..SpriteHitBox::default()
        };
        let sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.link_setup_hit_box(&mut hitbox);
        Sprite::<Self>::check_if_hit_boxes_overlap(&hitbox)
    }
    fn get_tile_attribute(&mut self, ram: &mut Ram<'_>, floor: u8, x: u16, y: u16) -> u8 {
        let mut x = x;
        Sprite::new(ram.bytes_mut(), self).get_tile_attribute(floor, &mut x, y)
    }
    fn overworld_read_tile_attribute(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8 {
        crate::overworld::read_tile_attribute(ram, self.assets.bytes(164), x, y)
    }
    fn project_speed_towards_location(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        x: u16,
        y: u16,
        speed: u8,
    ) -> (i8, i8) {
        let v = Sprite::new(ram.bytes_mut(), self)
            .sprite_project_speed_towards_location(k, x, y, speed);
        (v.x as i8, v.y as i8)
    }
    fn thrown_sprite_tile_and_sprite_interaction(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).thrown_sprite_tile_and_sprite_interaction(k);
    }
    fn palette_filter_wish_ponds(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_filter_wish_ponds());
    }
    fn wish_pond_item_oam_flag(&mut self, graphics: u8) -> u8 {
        crate::sprite_main_bank0::WISH_POND_OAM_FLAGS[usize::from(graphics)]
    }
    fn receive_item_draw_kind(&mut self, graphics: u8) -> u8 {
        crate::misc::RECEIVE_ITEM_TAB1[usize::from(graphics)]
    }
    fn initialized_segmented(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_initialized_segmented(k);
    }
    fn swamola_initialize_segments(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_prep_swamola_initialize_segments(k);
    }
    fn blind_prepare_battle(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_prep_blind_prepare_battle(k);
    }
    fn ganon_animation_idle(&mut self, ram: &mut Ram<'_>, k: usize) {
        let direction = usize::from(ram.read_u8(0xde0 + k));
        ram.write_u8(0xed0 + k, [9, 10][direction]);
        ram.write_u8(0xdc0 + k, [2, 10][direction]);
    }
    fn cutscene_agahnim_spawn_zelda(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .cutscene_agahnim_spawn_zelda_on_altar(k);
    }
    fn vitreous_spawn_smaller_eyes(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .vitreous_spawn_smaller_eyes(k);
    }
    fn spawn_bully(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }.spawn_bully(k);
    }
    fn trinexx_components_initialize(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .trinexx_components_initialize(k);
    }
    fn helmasaur_king_initialize(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .helmasaur_king_initialize(k);
    }
    fn prep_snitches(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_prep_snitches(k);
    }
    fn heart_upgrade_check_obtained(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .heart_upgrade_check_if_already_obtained(k);
    }
    fn arrghus_handle_puffs(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }.arrghus_handle_puffs(k);
    }
    fn talking_tree_spawn_eyeball(&mut self, ram: &mut Ram<'_>, k: usize, direction: u8) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_prep_talking_tree_spawn_eyeball(k, usize::from(direction));
    }
    fn shopkeeper_spawn_item(&mut self, ram: &mut Ram<'_>, k: usize, position: u8, what: u8) {
        const X: [i8; 3] = [-44, 8, 60];
        let mut info = SpriteSpawnInfo::default();
        let spawned =
            Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically_ex(k, 0xbb, &mut info, 12);
        assert!(spawned >= 0, "shop item slot must be available");
        let spawned = spawned as usize;
        ram.write_u8(0xba0 + spawned, what);
        ram.write_u8(0xe80 + spawned, what);
        let x = info
            .r0_x
            .wrapping_add_signed(i16::from(X[usize::from(position)]));
        let y = info.r2_y.wrapping_add(0x27);
        ram.write_u8(0xd10 + spawned, x as u8);
        ram.write_u8(0xd30 + spawned, (x >> 8) as u8);
        ram.write_u8(0xd00 + spawned, y as u8);
        ram.write_u8(0xd20 + spawned, (y >> 8) as u8);
        ram.write_u8(0xe40 + spawned, ram.read_u8(0xe40 + spawned) | 4);
    }
    fn priest_spawn_mantle(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.priest_spawn_mantle(k);
    }
    fn vulture_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.vulture_draw(k);
    }
    fn project_speed_towards_link(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        speed: u8,
    ) -> (i8, i8, i16, i16) {
        let v = Sprite::new(ram.bytes_mut(), self).sprite_project_speed_towards_link(k, speed);
        (
            v.x as i8,
            v.y as i8,
            i16::from(v.xdiff as i8),
            i16::from(v.ydiff as i8),
        )
    }
    fn spawn_dynamic_ex(
        &mut self,
        ram: &mut Ram<'_>,
        source: usize,
        kind: u8,
        slot: usize,
    ) -> Option<usize> {
        let mut info = SpriteSpawnInfo::default();
        let v = Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically_ex(
            source,
            kind,
            &mut info,
            slot as i32,
        );
        (v >= 0).then_some(v as usize)
    }
    fn moldorm_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.moldorm_draw(k);
    }
    fn convert_velocity_to_angle(&mut self, x: u8, y: u8) -> u8 {
        Sprite::<Self>::sprite_convert_velocity_to_angle(x, y)
    }
    fn giant_moldorm_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.giant_moldorm_draw(k);
    }
    fn check_damage_from_link(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_from_link(k)
    }
    fn check_damage_to_link(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link(k)
    }
    fn calculate_sfx_pan(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        crate::misc::sprite_calculate_sfx_pan(ram, self, k)
    }
    fn misc_bug_fixes_enabled(&self) -> bool {
        self.enhanced_features0 & 1 != 0
    }
    fn antfairy_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.sprite_draw_antfairy(k);
    }
    fn bounce_from_tile_collision(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self).sprite_bounce_from_tile_collision(k)
    }
    fn chicken_incr_subtype2(&mut self, ram: &mut Ram<'_>, k: usize, amount: u8) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .chicken_incr_subtype2(k, amount);
    }
    fn return_if_lifted(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_lifted(k);
        ram.read_u8(0xdd0 + k) == 0
    }
    fn draw_distress(&mut self, ram: &mut Ram<'_>, x: u8, y: u8, time: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_draw_distress_custom(
            u16::from(x),
            u16::from(y),
            time,
        );
    }
    fn direction_to_face_link_with_differences(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
    ) -> (u8, i8, i8) {
        let v = Sprite::new(ram.bytes_mut(), self).sprite_direction_to_face_link(k);
        (v.0, v.1.a as i8, v.1.b as i8)
    }
    fn return_if_lifted_permissive(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_lifted_permissive(k);
    }
    fn set_oam_piece_character_and_low_flags(
        &mut self,
        ram: &mut Ram<'_>,
        piece: usize,
        character: u8,
        low_flags: u8,
    ) {
        let mut entry = crate::oam::read(ram, piece);
        entry.charnum = character;
        entry.flags = entry.flags & 0xf0 | low_flags;
        crate::oam::write(ram, piece, entry);
    }
    fn throwable_scenery_interact(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).throwable_scenery_interact_with_sprites_and_tiles(k);
    }
    fn oam_allocate_region_b(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_b(bytes);
    }
    fn schedule_for_breakage(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_schedule_for_breakage(k);
    }
    fn draw_multiple_player_deferred(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        pieces: &[(i8, i8, u16, u8)],
    ) {
        let p = pieces
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect::<Vec<_>>();
        let mut v = PrepOamCoordsRet::default();
        {
            let mut s = Sprite::new(ram.bytes_mut(), self);
            s.sprite_get_16_bit_coords(k);
            s.sprite_draw_multiple_player_deferred(k, &p, Some(&mut v));
        }
        self.last_sprite_oam = v;
    }
    fn draw_multiple_and_get_offscreen(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        pieces: &[(i8, i8, u16, u8)],
    ) -> u8 {
        let pieces = pieces
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect::<Vec<_>>();
        let mut info = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(k);
        sprite.sprite_draw_multiple(k, &pieces, Some(&mut info));
        ((info.x | info.y) >> 8) as u8
    }
    fn flute_boy_check_if_player_close(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .flute_boy_check_if_player_close(k)
    }
    fn become_follower(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::tagalong::sprite_become_follower(ram, self, k);
    }
    fn hobo_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.hobo_draw(k);
    }
    fn elder_wife_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.elder_wife_draw(k);
    }
    fn elder_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.elder_draw(k);
    }
    fn sprite_sahasrahla(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_sahasrahla(k);
    }
    fn sprite_aginah(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_aginah(k);
    }
    fn emit_oam_sized(
        &mut self,
        ram: &mut Ram<'_>,
        _: usize,
        (x, y, character, flags, size): (u8, u8, u8, u8, u8),
    ) {
        let entry = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        crate::oam::set_plain(ram, entry, x, y, character, flags, size);
        ram.write_u16(0x90, ram.read_u16(0x90).wrapping_add(4));
        ram.write_u16(0x92, ram.read_u16(0x92).wrapping_add(1));
    }
    fn draw_absorbable_transient(&mut self, ram: &mut Ram<'_>, k: usize, shadow: bool) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_draw_absorbable_transient(k, shadow)
    }
    fn check_absorption_by_player(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_check_absorption_by_player(k);
    }
    fn handle_dragging_by_ancilla(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_handle_dragging_by_ancilla(k)
    }
    fn draw_ripple_if_in_water(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_draw_ripple_if_in_water(k);
    }
    fn bounce_off_wall(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_bounce_off_wall(k);
    }
    fn spawn_small_splash(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<usize> {
        let v = crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_small_splash(k);
        (v >= 0).then_some(v as usize)
    }
    fn garnish_alloc(&mut self, ram: &mut Ram<'_>) -> usize {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .garnish_alloc()
            .max(0) as usize
    }
    fn garnish_alloc_force(&mut self, ram: &mut Ram<'_>) -> usize {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.garnish_alloc_force()
    }
    fn link_reset_properties_c(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_reset_properties_c();
    }
    fn landmine_check_detonation_from_hammer(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .landmine_check_detonation_from_hammer(k)
    }
    fn spawn_bomb(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<usize> {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.sprite_spawn_bomb(k)
    }
    fn move_xyz(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_xyz(k);
    }
    fn show_message_minimal(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_minimal();
    }
    fn overworld_draw_map16_persist(&mut self, ram: &mut Ram<'_>, pos: u16, tile: u16) {
        crate::overworld::draw_map16_persist(ram, self.assets, pos, tile);
    }
    fn thief_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.thief_draw(k);
    }
    fn fortune_teller_light_or_dark_world(&mut self, ram: &mut Ram<'_>, k: usize, dark: bool) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .fortune_teller_light_or_dark_world(k, dark);
    }
    fn prep_and_draw_single_large_no_prep(&mut self, ram: &mut Ram<'_>, k: usize) {
        let v = PrepOamCoordsRet::default();
        Sprite::new(ram.bytes_mut(), self).sprite_prep_and_draw_single_large_no_prep(k, &v);
    }
    fn sprite_func3(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_func3(k);
    }
    fn octoballoon_find(&mut self, ram: &mut Ram<'_>) -> bool {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.octoballoon_find()
    }
    fn apply_recoil_to_link(&mut self, ram: &mut Ram<'_>, k: usize, speed: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_apply_recoil_to_link(k, speed);
    }
    fn invert_speed_xy(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_invert_speed_xy(k);
    }
    fn sprite_uncle(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_uncle(k);
    }
    fn sprite_priest(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_priest(k);
    }
    fn sprite_sanctuary_mantle(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_sanctuary_mantle(k);
    }
    fn sprite_evil_barrier(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.sprite_evil_barrier(k);
    }
    fn tutorial_soldier_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.tutorial_soldier_draw(k);
    }
    fn tutorial_guard_show_message_on_contact(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        msg: u16,
    ) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_tutorial_guard_show_message_on_contact(k, msg)
    }
    fn medallion_tablet_main(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.medallion_tablet_main(k);
    }
    fn sprite_dust_cloud(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_dust_cloud(k);
    }
    fn rupee_pull_spawn_prize(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .rupee_pull_spawn_prize(k);
    }
    fn spawn_poof_garnish(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_poof_garnish(k);
    }
    fn open_gargoyles_domain(&mut self, ram: &mut Ram<'_>) {
        crate::overworld::open_gargoyles_domain(ram, self.assets);
    }
    fn spawn_dust_cloud(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<usize> {
        let v = crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_dust_cloud(k);
        (v >= 0).then_some(v as usize)
    }
    fn talking_tree_mouth(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.talking_tree_mouth(k);
    }
    fn talking_tree_eye(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.talking_tree_eye(k);
    }
    fn sprite_mad_batter_bolt(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_mad_batter_bolt(k);
    }
    fn ancilla_add_gt_cutscene(&mut self, ram: &mut Ram<'_>) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.ancilla_add_gt_cutscene();
    }
    fn ancilla_add_waterfall_splash(&mut self, ram: &mut Ram<'_>) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_waterfall_splash();
    }
    fn create_pyramid_hole(&mut self, ram: &mut Ram<'_>) {
        crate::overworld::create_pyramid_hole(ram, self.assets);
    }
    fn garnish_spawn_pyramid_debris(&mut self, ram: &mut Ram<'_>, x: i8, y: i8, xv: i8, yv: i8) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .garnish_spawn_pyramid_debris(x, y, xv, yv);
    }
    fn emit_absolute_oam(
        &mut self,
        ram: &mut Ram<'_>,
        index: usize,
        (x, y, character, flags, size): (i8, i8, u8, u8, u8),
    ) {
        crate::oam::set_plain(ram, index, x as u8, y as u8, character, flags, size);
    }
    fn handle_absorption_by_player(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_handle_absorption_by_player(k);
    }
    fn faerie_handle_movement(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .faerie_handle_movement(k);
    }
    fn find_empty_bottle(&mut self, ram: &mut Ram<'_>) -> Option<usize> {
        let v = crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .sprite_find_empty_bottle();
        (v >= 0).then_some(v as usize)
    }
    fn ancilla_check_damage_to_sprite_preset(&mut self, ram: &mut Ram<'_>, j: usize, preset: u8) {
        Sprite::new(ram.bytes_mut(), self).ancilla_check_damage_to_sprite_preset(j, preset);
    }
    fn fish_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.fish_draw(k);
    }
    fn sprite_wish_pond3(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_wish_pond3(k);
    }
    fn ancilla_terminate_sparkle_objects(&mut self, ram: &mut Ram<'_>) {
        crate::ancilla::AncillaRuntime::terminate_sparkle_objects(self, ram);
    }
    fn add_happiness_pond_rupees(&mut self, ram: &mut Ram<'_>, choice: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_happiness_pond_rupees(choice);
    }
    fn spawn_dynamic(&mut self, ram: &mut Ram<'_>, source: usize, kind: u8) -> Option<usize> {
        let mut info = SpriteSpawnInfo::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        let slot = sprite.sprite_spawn_dynamically(source, kind, &mut info);
        if slot < 0 {
            None
        } else {
            sprite.sprite_set_spawned_coordinates(slot as usize, info);
            Some(slot as usize)
        }
    }

    fn set_spawned_coordinates(&mut self, _ram: &mut Ram<'_>, _slot: usize) {
        // The trait splits the C spawn and coordinate-copy calls, but its spawn
        // result does not carry SpriteSpawnInfo. `spawn_dynamic` therefore
        // performs both C operations atomically; this matching follow-up is
        // deliberately inert and does not omit any C state mutation.
    }

    fn prep_oam_with_flags_wide(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
    ) -> Option<(u16, u16, u8)> {
        let mut result = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(slot);
        (!sprite.sprite_prep_oam_coord_or_double_ret(slot, &mut result)).then_some((
            result.x,
            result.y,
            result.flags,
        ))
    }

    fn emit_oam_wide(
        &mut self,
        ram: &mut Ram<'_>,
        _: usize,
        x: u16,
        y: u16,
        character: u8,
        flags: u8,
    ) {
        let entry = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        crate::oam::set_helper0(ram, entry, x, y, character, flags, 0);
        ram.write_u16(0x90, ram.read_u16(0x90).wrapping_add(4));
        ram.write_u16(0x92, ram.read_u16(0x92).wrapping_add(1));
    }

    fn emit_oam(&mut self, ram: &mut Ram<'_>, _: usize, x: u8, y: u8, character: u8, flags: u8) {
        let entry = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        crate::oam::set_helper0(ram, entry, u16::from(x), u16::from(y), character, flags, 0);
        ram.write_u16(0x90, ram.read_u16(0x90).wrapping_add(4));
        ram.write_u16(0x92, ram.read_u16(0x92).wrapping_add(1));
    }

    fn return_if_inactive(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_inactive(slot)
    }

    fn return_if_recoiling(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_recoiling(slot)
    }

    fn check_damage_to_and_from_link(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_and_from_link(slot);
    }

    fn apply_speed_towards_link(&mut self, ram: &mut Ram<'_>, slot: usize, speed: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_apply_speed_towards_link(slot, speed);
    }

    fn random_number(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::get_random_number(ram)
    }

    fn move_xy(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_xy(slot);
    }

    fn move_z(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_z(slot);
    }

    fn check_tile_collision(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_tile_collision(slot) != 0
    }

    fn check_tile_collision2(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_check_tile_collision2(slot);
    }

    fn is_below_link(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_is_below_link(slot)
            .a
            != 0
    }

    fn is_right_of_link(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_is_right_of_link(slot)
            .a
            != 0
    }

    fn direction_to_face_link(&mut self, ram: &mut Ram<'_>, slot: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_direction_to_face_link(slot)
            .0
    }

    fn draw_multiple(&mut self, ram: &mut Ram<'_>, slot: usize, pieces: &[(i8, i8, u16, u8)]) {
        let pieces = pieces
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect::<Vec<_>>();
        let mut result = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(slot);
        sprite.sprite_draw_multiple(slot, &pieces, Some(&mut result));
        self.last_sprite_oam = result;
    }

    fn draw_shadow(&mut self, ram: &mut Ram<'_>, slot: usize) {
        let mut info = self.last_sprite_oam;
        Sprite::new(ram.bytes_mut(), self).sprite_draw_shadow(slot, &mut info);
    }

    fn hide_oam_piece_if_small(&mut self, ram: &mut Ram<'_>, piece: usize) {
        let base = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        let entry = base + piece;
        if ram.read_u8(crate::oam::BYTEWISE_EXTENDED_OAM + entry) & 2 == 0 {
            ram.write_u8(crate::oam::OAM_BUFFER + entry * 4 + 1, 0xf0);
        }
    }

    fn set_oam_piece_char_and_hflip(
        &mut self,
        ram: &mut Ram<'_>,
        piece: usize,
        character: u8,
        hflip: u8,
    ) {
        let base = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        let mut entry = crate::oam::read(ram, base + piece);
        entry.charnum = character;
        entry.flags = entry.flags & !0x40 | hflip;
        crate::oam::write(ram, base + piece, entry);
    }

    fn oam_allocate_defer_to_player(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_defer_to_player(slot);
    }

    fn oam_allocate_region_c(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_c(bytes);
    }
}
impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for NativeGameServices<'_> {
    fn approach_target_speed(&mut self, ram: &mut Ram<'_>, k: usize, x: u8, y: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_approach_target_speed(k, x, y);
    }
    fn spawn_dynamic(&mut self, ram: &mut Ram<'_>, source: usize, kind: u8) -> Option<usize> {
        let mut info = SpriteSpawnInfo::default();
        let slot =
            Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically(source, kind, &mut info);
        (slot >= 0).then_some(slot as usize)
    }
    fn spawn_dynamic_ex(
        &mut self,
        ram: &mut Ram<'_>,
        source: usize,
        kind: u8,
        slot: usize,
    ) -> Option<usize> {
        let mut info = SpriteSpawnInfo::default();
        let spawned = Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically_ex(
            source,
            kind,
            &mut info,
            slot as i32,
        );
        (spawned >= 0).then_some(spawned as usize)
    }
    fn set_spawned_coordinates(&mut self, ram: &mut Ram<'_>, slot: usize) {
        let info = SpriteSpawnInfo::default();
        Sprite::new(ram.bytes_mut(), self).sprite_set_spawned_coordinates(slot, info);
    }
    fn spawn_fire_phlegm(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<usize> {
        let slot = crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .sprite_spawn_fire_phlegm(k);
        (slot >= 0).then_some(slot as usize)
    }
    fn spawn_boss_poof(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<usize> {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.spawn_boss_poof(k)
    }
    fn spawn_poof_garnish(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_poof_garnish(k);
    }
    fn garnish_flame_trail(&mut self, ram: &mut Ram<'_>, k: usize, low: bool) -> usize {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .garnish_flame_trail(k, low)
    }
    fn sfx2_with_pan(&mut self, ram: &mut Ram<'_>, k: usize, sound: u8) {
        crate::misc::sprite_sfx2_with_pan(ram, self, k, sound);
    }
    fn sfx3_with_pan(&mut self, ram: &mut Ram<'_>, k: usize, sound: u8) {
        crate::misc::sprite_sfx3_with_pan(ram, self, k, sound);
    }
    fn return_if_inactive(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_inactive(k)
    }
    fn return_if_recoiling(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_recoiling(k)
    }
    fn move_xy(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_xy(k);
    }
    fn move_y(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_y(k);
    }
    fn move_x(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_x(k);
    }
    fn move_z(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_z(k);
    }
    fn move_xyz(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_xyz(k);
    }
    fn check_tile_collision(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_tile_collision(k) != 0
    }
    fn check_damage_to_link(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link(k)
    }
    fn check_damage_to_link_same_layer(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link_same_layer(k)
    }
    fn check_damage_to_and_from_link(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_and_from_link(k);
    }
    fn check_damage_from_link(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_from_link(k) != 0
    }
    fn direction_to_face_link(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_direction_to_face_link(k)
            .0
    }
    fn direction_to_face_location(&mut self, ram: &mut Ram<'_>, k: usize, x: u16, y: u16) -> u8 {
        Sprite::new(ram.bytes_mut(), self).sprite_direction_to_face_location(k, x, y)
    }
    fn apply_speed_towards_link(&mut self, ram: &mut Ram<'_>, k: usize, speed: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_apply_speed_towards_link(k, speed);
    }
    fn project_speed_towards_link(&mut self, ram: &mut Ram<'_>, k: usize, speed: u8) -> (u8, u8) {
        let speed = Sprite::new(ram.bytes_mut(), self).sprite_project_speed_towards_link(k, speed);
        (speed.x, speed.y)
    }
    fn project_speed_towards_location(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        x: u16,
        y: u16,
        speed: u8,
    ) -> (u8, u8) {
        let speed = Sprite::new(ram.bytes_mut(), self)
            .sprite_project_speed_towards_location(k, x, y, speed);
        (speed.x, speed.y)
    }
    fn random_number(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::get_random_number(ram)
    }
    fn prep_oam(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut result = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(k);
        sprite.sprite_prep_oam_coord(k, &mut result);
    }
    fn prep_oam_or_double_ret(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<(u8, u8, u8)> {
        let mut result = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(k);
        (!sprite.sprite_prep_oam_coord_or_double_ret(k, &mut result)).then_some((
            result.x as u8,
            result.y as u8,
            result.flags,
        ))
    }
    fn draw_shadow(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut v = PrepOamCoordsRet::default();
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_prep_oam_coord(k, &mut v);
        s.sprite_draw_shadow(k, &mut v);
    }
    fn draw_shadow_custom(&mut self, ram: &mut Ram<'_>, k: usize, size: u8) {
        let mut v = PrepOamCoordsRet::default();
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_prep_oam_coord(k, &mut v);
        s.sprite_draw_shadow_custom(k, &mut v, size);
    }
    fn antfairy_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.sprite_draw_antfairy(k);
    }
    fn antfairy_retile(&mut self, ram: &mut Ram<'_>, character: u8) {
        let base = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        for entry in base..base + 5 {
            let mut oam = crate::oam::read(ram, entry);
            oam.charnum = character;
            oam.flags = (oam.flags & 0xf0) | 2;
            crate::oam::write(ram, entry, oam);
        }
    }
    fn emit_oam(&mut self, ram: &mut Ram<'_>, _: usize, x: u8, y: u8, ch: u8, flags: u8, size: u8) {
        let entry = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        crate::oam::set_plain(ram, entry, x, y, ch, flags, size);
    }
    fn show_message_minimal(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_minimal();
    }
    fn extinguish_torch(&mut self, ram: &mut Ram<'_>, adjust: bool) {
        with_dungeon(self, ram, |d| {
            if adjust {
                d.ganon_extinguish_torch_adjust_translucency()
            } else {
                d.ganon_extinguish_torch()
            }
        });
    }
    fn convert_velocity_to_angle(&mut self, x: u8, y: u8) -> u8 {
        Sprite::<Self>::sprite_convert_velocity_to_angle(x, y)
    }
    fn ganon_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.ganon_draw(k);
    }
    fn garnish_alloc_overwrite_old(&mut self, ram: &mut Ram<'_>) -> usize {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .garnish_alloc_overwrite_old()
    }
    fn garnish_alloc_overwrite_old_low(&mut self, ram: &mut Ram<'_>) -> usize {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .garnish_alloc_overwrite_old_low()
    }
    fn spawn_fireball(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<usize> {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.sprite_spawn_fireball(k)
    }
    fn calculate_sfx_pan(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        crate::misc::sprite_calculate_sfx_pan(ram, self, k)
    }
    fn set_current_oam_char_flags(&mut self, ram: &mut Ram<'_>, character: u8, high_flags: u8) {
        let e = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        let mut o = crate::oam::read(ram, e);
        o.charnum = character;
        o.flags = (o.flags & 0xf) | high_flags;
        crate::oam::write(ram, e, o);
    }
    fn set_oam_char_flags(
        &mut self,
        ram: &mut Ram<'_>,
        entry: usize,
        character: u8,
        high_flags: u8,
    ) {
        let mut o = crate::oam::read(ram, entry);
        o.charnum = character;
        o.flags = (o.flags & 0xf) | high_flags;
        crate::oam::write(ram, entry, o);
    }
    fn set_current_oam_y(&mut self, ram: &mut Ram<'_>, entry: usize, y: u8) {
        let base = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        let mut o = crate::oam::read(ram, base + entry);
        o.y = y;
        crate::oam::write(ram, base + entry, o);
    }
    fn bounce_from_tile_collision(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_bounce_from_tile_collision(k) != 0
    }
    fn attempt_damage_to_link_plus_recoil(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_attempt_damage_to_link_plus_recoil(k);
    }
    fn draw_big_shadow(&mut self, ram: &mut Ram<'_>, k: usize, animation: u8) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_draw_big_shadow(k, animation);
    }
    fn load_properties(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_prep_load_properties(k);
    }
    fn transmute_to_bomb(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_transmute_to_bomb(k);
    }
    fn decode_animated_sprite_tile(&mut self, ram: &mut Ram<'_>, tile: u8) {
        with_gfx(self, ram, |g| g.decode_animated_sprite_tile_variable(tile));
    }
    fn draw_multiple_player_deferred(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        pieces: &[(i8, i8, u16, u8)],
    ) {
        let p = pieces
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect::<Vec<_>>();
        let mut v = PrepOamCoordsRet::default();
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_draw_multiple_player_deferred(k, &p, Some(&mut v));
    }
    fn garnish_spawn_sparkle(&mut self, ram: &mut Ram<'_>, k: usize, x: i8, y: i8) -> usize {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_garnish_spawn_sparkle(k, x as u16, y as u16)
            .max(0) as usize
    }
    fn link_cancel_dash(&mut self, ram: &mut Ram<'_>) {
        if ram.read_u8(0x372) != 0 {
            for slot in 0..5 {
                if ram.read_u8(0xc4a + slot) == 0x1e {
                    ram.write_u8(0xc4a + slot, 0);
                }
            }
            ram.write_u8(0x3a9, 0);
            ram.write_u8(0x5e, 0);
            ram.write_u8(0x372, 0);
            ram.write_u8(0x378, 0);
            ram.write_u8(0x34c, 0);
        }
    }
    fn force_prize_drop(&mut self, ram: &mut Ram<'_>, k: usize, a: u8, b: u8) {
        Sprite::new(ram.bytes_mut(), self).force_prize_drop(k, a, usize::from(b));
    }
    fn show_solicited_message(&mut self, ram: &mut Ram<'_>, k: usize, msg: u16) -> u16 {
        Sprite::new(ram.bytes_mut(), self).sprite_show_solicited_message(k, msg)
    }
    fn show_message_unconditional(&mut self, ram: &mut Ram<'_>, msg: u16) {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_unconditional(msg);
    }
    fn giant_moldorm_draw_segment(&mut self, ram: &mut Ram<'_>, k: usize, offset: u8) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .giant_moldorm_draw_segment_c_or_tail(k, offset);
    }
    fn giant_moldorm_draw_segment_ab(&mut self, ram: &mut Ram<'_>, k: usize, lookback: u8) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .giant_moldorm_draw_segment_ab(k, lookback);
    }
    fn draw_water_ripple_with_oam_adjust(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_draw_water_ripple_with_oam_adjust(k);
    }
    fn link_receive_item(&mut self, ram: &mut Ram<'_>, item: u8, method: u8) {
        ram.write_u8(0x2e9, method);
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_receive_item(item, -1);
    }
    fn behave_as_barrier(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_behave_as_barrier(k);
    }
    fn initialize_secondary_item_minigame(&mut self, ram: &mut Ram<'_>, which: u8) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_initialize_secondary_item_minigame(which);
    }
    fn helmasaur_hammer_overlap(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        let mut hitbox = crate::sprite::SpriteHitBox::default();
        let sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.player_setup_action_hit_box(&mut hitbox);
        let old_y = ram.read_u8(0xd00 + k);
        ram.write_u8(0xd00 + k, old_y.wrapping_add(8));
        Sprite::new(ram.bytes_mut(), self).sprite_setup_hit_box(k, &mut hitbox);
        ram.write_u8(0xd00 + k, old_y);
        Sprite::<Self>::check_if_hit_boxes_overlap(&hitbox)
    }
    fn helmasaur_bomb_overlap(&mut self, ram: &mut Ram<'_>, k: usize, j: usize) -> bool {
        let mut hitbox = crate::sprite::SpriteHitBox::default();
        Sprite::new(ram.bytes_mut(), self).sprite_setup_hit_box(k, &mut hitbox);
        let x = (u16::from(ram.read_u8(0xc04 + j)) | u16::from(ram.read_u8(0xc18 + j)) << 8)
            .wrapping_sub(6);
        let y = (u16::from(ram.read_u8(0xbfa + j)) | u16::from(ram.read_u8(0xc0e + j)) << 8)
            .wrapping_sub(u16::from(ram.read_u8(0x29e + j)));
        [hitbox.r0_xlo, hitbox.r8_xhi] = x.to_le_bytes();
        [hitbox.r1_ylo, hitbox.r9_yhi] = y.to_le_bytes();
        hitbox.r2 = 2;
        hitbox.r3 = 15;
        Sprite::<Self>::check_if_hit_boxes_overlap(&hitbox)
    }
    fn manually_set_death_flag_uw(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_manually_set_death_flag_uw(k);
    }
    fn trinexx_flash_shell_palette(&mut self, ram: &mut Ram<'_>, blue: bool, _flash: bool) {
        with_gfx(self, ram, |g| {
            if blue {
                g.trinexx_flash_shell_palette_blue()
            } else {
                g.trinexx_flash_shell_palette_red()
            }
        });
    }
    fn draw_multiple(&mut self, ram: &mut Ram<'_>, k: usize, pieces: &[(i8, i8, u16, u8)]) {
        let pieces = pieces
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect::<Vec<_>>();
        let mut result = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(k);
        sprite.sprite_draw_multiple(k, &pieces, Some(&mut result));
    }
    fn draw_single_large(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(k);
        sprite.sprite_draw_single_large(k);
    }
    fn draw_single_small(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(k);
        sprite.sprite_draw_single_small(k);
    }
    fn get_16bit_coords(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_get_16_bit_coords(k);
    }
    fn is_right_of_link(&mut self, ram: &mut Ram<'_>, k: usize) -> (u8, i8) {
        let v = Sprite::new(ram.bytes_mut(), self).sprite_is_right_of_link(k);
        (v.a, v.b as i8)
    }
    fn is_below_link(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self).sprite_is_below_link(k).a
    }
    fn oam_allocate_region_a(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_a(bytes);
    }
    fn oam_allocate_region_b(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_b(bytes);
    }
    fn oam_allocate_region_c(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_c(bytes);
    }
    fn pikit_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.pikit_draw(k);
    }
    fn bomber_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.bomber_draw(k);
    }
    fn draw_zirro_bomb(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.sprite_draw_zirro_bomb(k);
    }
    fn zirro_drop_bomb(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }.zirro_drop_bomb(k);
    }
}

impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for NativeGameServices<'_> {
    fn move_xyz(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_xyz(k);
    }
    fn check_if_link_is_busy(&mut self, ram: &mut Ram<'_>) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_if_link_is_busy()
    }
    fn check_if_room_is_clear(&mut self, ram: &mut Ram<'_>) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_if_room_is_clear()
    }
    fn player_setup_action_hit_box(
        &mut self,
        ram: &mut Ram<'_>,
        hb: &mut crate::sprite::SpriteHitBox,
    ) {
        Sprite::new(ram.bytes_mut(), self).player_setup_action_hit_box(hb);
    }
    fn hit_boxes_overlap(&mut self, hb: &crate::sprite::SpriteHitBox) -> bool {
        Sprite::<Self>::check_if_hit_boxes_overlap(hb)
    }
    fn setup_sprite_hit_box(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        hb: &mut crate::sprite::SpriteHitBox,
    ) {
        Sprite::new(ram.bytes_mut(), self).sprite_setup_hit_box(k, hb);
    }
    fn give_damage(&mut self, ram: &mut Ram<'_>, k: usize, damage: u8, hit_timer: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_give_damage(k, damage, hit_timer);
    }
    fn player_bee_hone_in_on_target(&mut self, ram: &mut Ram<'_>, target: usize, bee: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .player_bee_hone_in_on_target(target, bee);
    }
    fn spawn_sparkle(&mut self, ram: &mut Ram<'_>, k: usize, x: i8, y: i8) {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_garnish_spawn_sparkle_limited(k, x as u16, y as u16);
    }
    fn spawn_sparkle_limited(&mut self, ram: &mut Ram<'_>, k: usize, x: i8, y: i8) {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_garnish_spawn_sparkle_limited(k, x as u16, y as u16);
    }
    fn spawn_poof_garnish(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_poof_garnish(k);
    }
    fn get_tile_attribute(&mut self, ram: &mut Ram<'_>, x: &mut u16, y: u16) -> u8 {
        Sprite::new(ram.bytes_mut(), self).get_tile_attribute(0, x, y)
    }
    fn draw_distress(&mut self, ram: &mut Ram<'_>, k: usize) {
        let frame = ram.read_u8(0x1a);
        let mut info = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(k);
        if !sprite.sprite_prep_oam_coord_or_double_ret(k, &mut info) {
            sprite.sprite_draw_distress_custom(info.x, info.y, frame);
        }
    }
    fn sprite_is_offscreen(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        let mut info = PrepOamCoordsRet::default();
        Sprite::new(ram.bytes_mut(), self).sprite_prep_oam_coord_or_double_ret(k, &mut info)
    }
    fn guruguru_bar_sin(&mut self, angle: u16, radius: u8) -> i8 {
        crate::sprite_main_bank0::SpriteMainBank0::<Self>::guruguru_bar_sin(angle, radius)
    }
    fn convert_velocity_to_angle(&mut self, x: i8, y: i8) -> u8 {
        Sprite::<Self>::sprite_convert_velocity_to_angle(x as u8, y as u8)
    }
    fn arrgi_sin(&mut self, angle: u16, radius: u8) -> i8 {
        crate::sprite_main_bank0::SpriteMainBank0::<Self>::arrgi_sin(angle, radius)
    }
    fn draw_large_shadow2(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_draw_large_shadow2(k);
    }
    fn show_message(&mut self, ram: &mut Ram<'_>, message: u16) {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_unconditional(message);
    }
    fn queue_sfx2(&mut self, ram: &mut Ram<'_>, k: usize, sfx: u8) {
        crate::misc::sprite_sfx2_with_pan(ram, self, k, sfx);
    }
    fn queue_sfx3(&mut self, ram: &mut Ram<'_>, k: usize, sfx: u8) {
        crate::misc::sprite_sfx3_with_pan(ram, self, k, sfx);
    }
    fn link_receive_item(&mut self, ram: &mut Ram<'_>, item: u8) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_receive_item(item, -1);
    }
    fn rapid_terminate_receive_item(&mut self, ram: &mut Ram<'_>) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .shop_keeper_rapid_terminate_receive_item();
    }
    fn link_handle_velocity(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_handle_velocity();
    }
    fn link_handle_moving_animation(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_handle_moving_animation_full_long_entry();
    }
    fn handle_indoor_camera_and_doors(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .handle_indoor_camera_and_doors();
    }
    fn link_reset_properties_a(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_reset_properties_a();
    }
    fn halt_all_movement(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).sprite_halt_all_movement();
    }
    fn load_follower_graphics(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.load_follower_graphics());
    }
    fn become_follower(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::tagalong::sprite_become_follower(ram, self, k);
    }
    fn track_body_to_head(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_track_body_to_head(k);
    }
    fn set_enemy_damage(&mut self, ram: &mut Ram<'_>, index: usize, value: u8) {
        ram.write_u8(0x16000 + index, value);
    }
    fn update_common_tile(&mut self, ram: &mut Ram<'_>, x: u16, y: u16, tile: u8) {
        with_dungeon(self, ram, |dungeon| {
            dungeon.dungeon_update_tile_map_with_common_tile(x, y, tile);
        });
    }
    fn garnish_alloc_low(&mut self, ram: &mut Ram<'_>) -> Option<usize> {
        let v =
            crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.garnish_alloc_low();
        (v >= 0).then_some(v as usize)
    }
    fn garnish_alloc(&mut self, ram: &mut Ram<'_>) -> Option<usize> {
        let v = crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.garnish_alloc();
        (v >= 0).then_some(v as usize)
    }
    fn garnish_alloc_with_limit(&mut self, ram: &mut Ram<'_>, limit: usize) -> Option<usize> {
        let v = crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .garnish_alloc_limit(limit as i32);
        (v >= 0).then_some(v as usize)
    }
    fn garnish_alloc_overwrite_old(&mut self, ram: &mut Ram<'_>) -> usize {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .garnish_alloc_overwrite_old()
    }
    fn spawn_dynamic_ex(
        &mut self,
        ram: &mut Ram<'_>,
        parent: usize,
        kind: u8,
        slots: usize,
    ) -> Option<(usize, u16, u16, u8)> {
        let mut info = SpriteSpawnInfo::default();
        let slot = Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically_ex(
            parent,
            kind,
            &mut info,
            slots as i32,
        );
        (slot >= 0).then_some((slot as usize, info.r0_x, info.r2_y, info.r4_z))
    }
    fn place_weapon_tink(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_place_weapon_tink(k);
    }
    fn apply_speed_towards_link(&mut self, ram: &mut Ram<'_>, k: usize, speed: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_apply_speed_towards_link(k, speed);
    }
    fn approach_target_speed(&mut self, ram: &mut Ram<'_>, k: usize, x: i8, y: i8) {
        Sprite::new(ram.bytes_mut(), self).sprite_approach_target_speed(k, x as u8, y as u8);
    }
    fn link_cancel_dash(&mut self, ram: &mut Ram<'_>) {
        crate::sprite_main_bank2a::SpriteMainBank2aRuntime::link_cancel_dash(self, ram);
    }
    fn link_reset_swimming(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_reset_swimming_state();
    }
    fn spawn_superficial_bomb_blast(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<usize> {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_spawn_superficial_bomb_blast(k)
    }
    fn pipe_validate_entry(&mut self, ram: &mut Ram<'_>) -> bool {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.pipe_validate_entry()
    }
    fn agahnim_warp_shadow_filter(&mut self, ram: &mut Ram<'_>, index: usize) {
        with_gfx(self, ram, |gfx| {
            gfx.agahnim_warp_shadow_filter(index, 0xc019)
        });
    }
    fn agahnim_apply_motion_blur(&mut self, ram: &mut Ram<'_>, k: usize) -> Option<usize> {
        let slot = crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_agahnim_apply_motion_blur(k);
        (slot >= 0).then_some(slot as usize)
    }
    fn spawn_phantom_ganon(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_spawn_phantom_ganon(k);
    }
    fn bank2b_prepare_dungeon_exit_from_boss_fight(&mut self, ram: &mut Ram<'_>) {
        with_dungeon(self, ram, |dungeon| {
            dungeon.prepare_dungeon_exit_from_boss_fight()
        });
    }
    fn palette_filter_sp5f(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.palette_filter_sp5f());
    }
    fn bank2b_place_rupulse_spark(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_place_rupulse_spark_2(k);
    }
    fn bank2b_spawn_big_splash(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_spawn_big_splash(k);
    }
    fn bank2b_old_mountain_man_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.old_mountain_man_draw(k);
    }
    fn bank2b_cache_camera_properties(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .cache_camera_properties();
    }
    fn draw_big_shadow(&mut self, ram: &mut Ram<'_>, k: usize) {
        const SHADOW: [DrawMultipleData; 3] = [
            DrawMultipleData {
                x: -6,
                y: 19,
                char_flags: 0x086c,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 19,
                char_flags: 0x086c,
                ext: 2,
            },
            DrawMultipleData {
                x: 6,
                y: 19,
                char_flags: 0x086c,
                ext: 2,
            },
        ];
        let z = u16::from(ram.read_u8(0xf70 + k));
        ram.write_u16(0xfda, ram.read_u16(0xfda).wrapping_add(z));
        ram.write_u16(0x90, ram.read_u16(0x90).wrapping_add(16));
        ram.write_u16(0x92, ram.read_u16(0x92).wrapping_add(4));
        Sprite::new(ram.bytes_mut(), self).sprite_draw_multiple(k, &SHADOW, None);
        Sprite::new(ram.bytes_mut(), self).sprite_get_16_bit_coords(k);
    }
    fn kholdstare_shell_palette_filtering(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.kholdstare_shell_palette_filtering());
    }
    fn spawn_lightning(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_spawn_lightning(k);
    }
    fn spike_trap_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }.spike_trap_draw(k);
    }
    fn shopkeeper_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.shopkeeper_draw(k);
    }
    fn maze_game_guy_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.maze_game_guy_draw(k);
    }
    fn babusu_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.babusu_draw(k);
    }
    fn draw_antifairy(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.sprite_draw_antfairy(k);
    }
    fn add_sword_charge_sparkle(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::ancilla::Ancilla { ram, runtime: self }.add_sword_charge_sparkle_from(k);
    }
    fn flute_boy_ostrich_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .flute_boy_ostrich_draw(k);
    }
    fn wizzbeam(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_wizzbeam(k);
    }
    fn wizzrobe_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.wizzrobe_draw(k);
    }
    fn laser_beam_build_up_garnish(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .laser_beam_build_up_garnish(k);
    }
    fn wallmaster_send_player_to_last_entrance(&mut self, ram: &mut Ram<'_>) {
        crate::misc::wallmaster_send_player_to_last_entrance(ram, self);
    }
    fn link_initialize(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_initialize();
    }
    fn bee_put_in_bottle(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.bee_put_in_bottle(k);
    }
    fn thief_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.thief_draw(k);
    }
    fn drinking_guy_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.drinking_guy_draw(k);
    }
    fn goriya_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.goriya_draw(k);
    }
    fn sprite_94_tile(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }.sprite_94_tile(k);
    }
    fn spawn_small_splash(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_small_splash(k);
    }
    fn spawn_dynamic(
        &mut self,
        ram: &mut Ram<'_>,
        parent: usize,
        kind: u8,
    ) -> Option<(usize, u16, u16, u8)> {
        let mut info = SpriteSpawnInfo::default();
        let slot =
            Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically(parent, kind, &mut info);
        (slot >= 0).then_some((slot as usize, info.r0_x, info.r2_y, info.r4_z))
    }
    fn return_if_inactive(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_inactive(k)
    }
    fn return_if_recoiling(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_recoiling(k)
    }
    fn return_if_paused(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_paused(k)
    }
    fn move_xy(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_xy(k);
    }
    fn move_x(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_x(k);
    }
    fn move_z(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_z(k);
    }
    fn check_tile_collision(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self).sprite_check_tile_collision(k)
    }
    fn random(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::get_random_number(ram)
    }
    fn project_speed(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        x: u16,
        y: u16,
        speed: u8,
    ) -> (i8, i8) {
        let v = Sprite::new(ram.bytes_mut(), self)
            .sprite_project_speed_towards_location(k, x, y, speed);
        (v.x as i8, v.y as i8)
    }
    fn project_speed_towards_link(&mut self, ram: &mut Ram<'_>, k: usize, speed: u8) -> (i8, i8) {
        let v = Sprite::new(ram.bytes_mut(), self).sprite_project_speed_towards_link(k, speed);
        (v.x as i8, v.y as i8)
    }
    fn check_damage_to_link(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link(k)
    }
    fn check_damage_same_layer(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link_same_layer(k)
    }
    fn check_damage_ignore_layer(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link_ignore_layer(k)
    }
    fn check_damage_to_and_from_link(&mut self, ram: &mut Ram<'_>, k: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_and_from_link(k)
    }
    fn check_damage_from_link(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_from_link(k)
    }
    fn direction_to_face_link(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_direction_to_face_link(k)
            .0
    }
    fn is_below_link(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self).sprite_is_below_link(k).a
    }
    fn is_right_of_link(&mut self, ram: &mut Ram<'_>, k: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_is_right_of_link(k)
            .a
    }
    fn prep_oam(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut v = PrepOamCoordsRet::default();
        Sprite::new(ram.bytes_mut(), self).sprite_prep_oam_coord(k, &mut v);
    }
    fn draw_multiple(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        pieces: &[crate::sprite_main_bank2b::DrawPiece],
    ) {
        let p = pieces
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect::<Vec<_>>();
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_draw_multiple(k, &p, None);
    }
    fn draw_single_large(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_draw_single_large(k);
    }
    fn draw_single_small(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_draw_single_small(k);
    }
    fn allocate_oam_region_b(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_b(bytes);
    }
    fn allocate_oam_region_c(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_c(bytes);
    }
    fn allocate_oam_defer_to_player(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_defer_to_player(k);
    }
    fn get_16bit_coords(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_get_16_bit_coords(k);
    }
    fn draw_shadow(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut v = PrepOamCoordsRet::default();
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_prep_oam_coord(k, &mut v);
        s.sprite_draw_shadow(k, &mut v);
    }
    fn draw_shadow_custom(&mut self, ram: &mut Ram<'_>, k: usize, y: u8) {
        let mut v = PrepOamCoordsRet::default();
        let mut s = Sprite::new(ram.bytes_mut(), self);
        s.sprite_get_16_bit_coords(k);
        s.sprite_prep_oam_coord(k, &mut v);
        s.sprite_draw_shadow_custom(k, &mut v, y);
    }
    fn correct_oam_entries(&mut self, ram: &mut Ram<'_>, k: usize, count: u8, value: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_correct_oam_entries(k, usize::from(count), value);
    }
    fn behave_as_barrier(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_behave_as_barrier(k);
    }
    fn show_solicited_message(&mut self, ram: &mut Ram<'_>, k: usize, msg: u16) -> u16 {
        Sprite::new(ram.bytes_mut(), self).sprite_show_solicited_message(k, msg)
    }
    fn show_message_on_contact(&mut self, ram: &mut Ram<'_>, k: usize, msg: u16) -> u16 {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_on_contact(k, msg)
    }
    fn attempt_damage_to_link_plus_recoil(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_attempt_damage_to_link_plus_recoil(k);
    }
    fn manually_set_death_flag_uw(&mut self, ram: &mut Ram<'_>, k: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_manually_set_death_flag_uw(k);
    }
    fn nullify_hookshot_drag(&mut self, ram: &mut Ram<'_>, _: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_nullify_hookshot_drag();
    }
    fn repel_dash(&mut self, ram: &mut Ram<'_>, _: usize) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .sprite_repel_dash();
    }
    fn draw_large_water_turbulence(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_draw_large_water_turbulence(k);
    }
    fn kiki_lying_in_wait(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.kiki_lying_in_wait(k);
    }
    fn bank2b_stalfos_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.stalfos_draw(k);
    }
    fn bank2b_zazak_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.zazak_draw(k);
    }
    fn freezor_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.freezor_draw(k);
    }
    fn crystal_maiden_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.crystal_maiden_draw(k);
    }
    fn mothula_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.mothula_draw(k);
    }
    fn kholdstare_draw(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.kholdstare_draw(k);
    }
}

impl crate::tagalong::TagalongHooks for NativeGameServices<'_> {
    fn enhanced_features0(&self) -> u32 {
        self.enhanced_features0
    }

    fn project_speed_towards_player(
        &mut self,
        ram: &mut Ram<'_>,
        ancilla: usize,
        speed: u8,
    ) -> crate::tagalong::ProjectSpeed {
        let value = crate::ancilla::Ancilla { ram, runtime: self }
            .project_speed_towards_player(ancilla, speed);
        crate::tagalong::ProjectSpeed {
            x: value.x,
            y: value.y,
        }
    }

    fn ancilla_move_y(&mut self, ram: &mut Ram<'_>, ancilla: usize) {
        crate::ancilla::Ancilla { ram, runtime: self }.move_y(ancilla);
    }

    fn ancilla_move_x(&mut self, ram: &mut Ram<'_>, ancilla: usize) {
        crate::ancilla::Ancilla { ram, runtime: self }.move_x(ancilla);
    }

    fn show_text_message(&mut self, ram: &mut Ram<'_>) {
        crate::misc::main_show_text_message(ram);
    }

    fn add_super_bomb_explosion(&mut self, ram: &mut Ram<'_>, kind: u8, arg: u8) -> i32 {
        crate::ancilla::Ancilla { ram, runtime: self }
            .add_super_bomb_explosion(kind, arg)
            .map_or(-1, |slot| slot as i32)
    }

    fn old_man_revert_to_sprite(&mut self, ram: &mut Ram<'_>, history: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .old_man_revert_to_sprite(history);
    }

    fn sprite_spawn_dynamically(&mut self, ram: &mut Ram<'_>, source: usize, kind: u8) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically(source, kind, &mut info)
    }

    fn sprite_prep_load_properties(&mut self, ram: &mut Ram<'_>, sprite: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_prep_load_properties(sprite);
    }
}

impl crate::overlord::OverlordHooks for NativeGameServices<'_> {
    fn random_number(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::get_random_number(ram)
    }

    fn spawn_sprite(
        &mut self,
        ram: &mut Ram<'_>,
        source: usize,
        sprite_type: u8,
        highest_slot: usize,
    ) -> Option<(usize, crate::overlord::SpawnInfo)> {
        let mut info = SpriteSpawnInfo::default();
        let slot = Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically_ex(
            source,
            sprite_type,
            &mut info,
            highest_slot as i32,
        );
        (slot >= 0).then_some((
            slot as usize,
            crate::overlord::SpawnInfo {
                overlord_x: info.r5_overlord_x,
                overlord_y: info.r7_overlord_y,
            },
        ))
    }

    fn tile_attribute(&mut self, ram: &mut Ram<'_>, floor: u8, x: &mut u16, y: u16) -> u8 {
        SpriteMainBank0Runtime::get_tile_attribute(self, ram, floor, x, y)
    }

    fn garnish_alloc(&mut self, ram: &mut Ram<'_>) -> Option<usize> {
        (0..30).rev().find(|&slot| ram.read_u8(0x1f800 + slot) == 0)
    }

    fn calculate_sfx_pan(&mut self, ram: &mut Ram<'_>, x: u8) -> u8 {
        crate::misc::calculate_sfx_pan_arbitrary(ram, x)
    }

    fn queue_sfx2_with_pan(&mut self, ram: &mut Ram<'_>, sprite: usize, sfx: u8) {
        SpriteMainBank0Runtime::queue_sfx2_with_pan(self, ram, sprite, sfx);
    }

    fn queue_sfx3_with_pan(&mut self, ram: &mut Ram<'_>, sprite: usize, sfx: u8) {
        SpriteMainBank0Runtime::queue_sfx3_with_pan(self, ram, sprite, sfx);
    }

    fn transmute_to_bomb(&mut self, ram: &mut Ram<'_>, sprite: usize) {
        ram.write_u8(0xe20 + sprite, 0x4a);
        ram.write_u8(0xdb0 + sprite, 1);
        ram.write_u8(0xe00 + sprite, 0xff);
        ram.write_u8(0xe60 + sprite, 0x18);
        ram.write_u8(0xf50 + sprite, 8);
        ram.write_u8(0xe50 + sprite, 0);
    }

    fn sinus_lookup(&self, angle: u8) -> u16 {
        crate::sprite_main_bank0::SINUS_LOOKUP_TABLE[usize::from(angle)]
    }
}

impl SpriteHooks for NativeGameServices<'_> {
    fn garnish_alloc_limit(&mut self, ram: &mut Ram<'_>, limit: usize) -> Option<usize> {
        let slot = crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .garnish_alloc_limit(limit as i32);
        (slot >= 0).then_some(slot as usize)
    }

    fn random_u8(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::get_random_number(ram)
    }

    fn ancilla_spawn_falling_prize(&mut self, ram: &mut Ram<'_>, item: u8) -> i32 {
        crate::ancilla::Ancilla { ram, runtime: self }
            .add_falling_prize(0x29, item, 4)
            .map_or(-1, |slot| slot as i32)
    }

    fn tile_attribute(&mut self, ram: &mut Ram<'_>, _: u8, x: &mut u16, y: u16) -> u8 {
        crate::tile_detect::overworld_tile_attribute(ram, self, *x, y)
    }

    fn handle_indoor_camera_and_doors(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .handle_indoor_camera_and_doors();
    }

    fn link_cancel_dash(&mut self, ram: &mut Ram<'_>) {
        crate::sprite_main_bank2a::SpriteMainBank2aRuntime::link_cancel_dash(self, ram);
    }

    fn enhanced_features0(&self) -> u32 {
        self.enhanced_features0
    }

    fn sprite_prep_small_key(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_prep_small_key(slot);
    }

    fn throwable_scenery_flags(&self, kind: u8) -> u8 {
        crate::sprite_main_bank0::THROWABLE_SCENERY_FLAGS[usize::from(kind)]
    }

    fn link_calculate_sfx_pan(&self, ram: &Ram<'_>) -> u8 {
        crate::misc::link_calculate_sfx_pan(ram)
    }

    fn sprite_sfx2_with_pan(&mut self, ram: &mut Ram<'_>, slot: usize, sfx: u8) {
        crate::misc::sprite_sfx2_with_pan(ram, self, slot, sfx);
    }

    fn sprite_sfx3_with_pan(&mut self, ram: &mut Ram<'_>, slot: usize, sfx: u8) {
        crate::misc::sprite_sfx3_with_pan(ram, self, slot, sfx);
    }

    fn link_receive_item(&mut self, ram: &mut Ram<'_>, item: u8, method: u8) {
        ram.write_u8(0x2e9, method);
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_receive_item(item, -1);
    }

    fn palette_load_shield(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.palette_load_shield());
    }

    fn sprite_transmute_to_bomb(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_transmute_to_bomb(slot);
    }

    fn sprite_draw_water_ripple(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_draw_water_ripple(slot);
    }

    fn dungeon_update_tilemap_with_common_tile(
        &mut self,
        ram: &mut Ram<'_>,
        x: u16,
        y: u16,
        tile: u8,
    ) {
        with_dungeon(self, ram, |dungeon| {
            dungeon.dungeon_update_tile_map_with_common_tile(x, y, tile);
        });
    }

    fn overworld_sprite_data(&self, ram: &Ram<'_>, area: u16) -> &[u8] {
        let stage = match ram.read_u8(0xf3c5) {
            3 => 2,
            2 => 1,
            _ => 0,
        };
        let offset = usize::from(self.assets.words(159)[usize::from(area) + stage * 144]);
        &self.assets.bytes(160)[offset..]
    }

    fn dungeon_sprite_data(&self, room: u16) -> &[u8] {
        let offset = usize::from(self.assets.words(59)[usize::from(room)]);
        &self.assets.bytes(58)[offset..]
    }

    fn sprite_prep_big_key_load_graphics(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_prep_big_key_load_graphics(slot);
    }

    fn sprite_prep_key_set_item_drop(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_prep_key_set_item_drop(slot);
    }

    fn throwable_scenery_scatter_into_debris(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .throwable_scenery_scatter_into_debris(slot);
    }

    fn flame_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }.flame_draw(slot);
    }

    fn prepare_dungeon_exit_from_boss_fight(&mut self, ram: &mut Ram<'_>) {
        with_dungeon(self, ram, |dungeon| {
            dungeon.prepare_dungeon_exit_from_boss_fight();
        });
    }

    fn sprite_spawn_poof_garnish(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_poof_garnish(slot);
    }

    fn handle_screen_flash(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.handle_screen_flash());
    }

    fn follower_main(&mut self, ram: &mut Ram<'_>) {
        crate::tagalong::main(ram, self);
    }

    fn ancilla_main(&mut self, ram: &mut Ram<'_>) {
        crate::ancilla::Ancilla { ram, runtime: self }.main();
    }

    fn overlord_main(&mut self, ram: &mut Ram<'_>) {
        crate::overlord::main(ram, self);
    }
}

impl SpriteMainBank0Runtime for NativeGameServices<'_> {
    fn sprite_active_cross_bank(&mut self, ram: &mut Ram<'_>, slot: usize, kind: u8) {
        if (crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self })
            .dispatch_active_local(slot, kind)
        {
            return;
        }
        if (crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self })
            .dispatch_active(slot, kind)
        {
            return;
        }
        let _ = (crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self })
            .dispatch_active(slot, kind);
    }
    fn sprite_prep_cross_bank(&mut self, ram: &mut Ram<'_>, slot: usize, kind: u8) {
        let _ = kind;
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_module_initialize_after_properties(slot);
    }
    fn draw_moldorm_segment(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        pieces: &[DrawMultipleData; 4],
    ) {
        self.draw_multiple(ram, slot, pieces);
    }
    fn show_solicited_message(&mut self, ram: &mut Ram<'_>, slot: usize, message: u16) -> u16 {
        Sprite::new(ram.bytes_mut(), self).sprite_show_solicited_message(slot, message)
    }
    fn show_message_unconditional(&mut self, ram: &mut Ram<'_>, message: u16) {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_unconditional(message);
    }
    fn fortune_teller_perform_pseudo_science(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .fortune_teller_perform_pseudo_science(slot);
    }
    fn show_message_on_contact(&mut self, ram: &mut Ram<'_>, slot: usize, message: u16) -> u16 {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_on_contact(slot, message)
    }
    fn link_reset_properties_a(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_reset_properties_a();
    }
    fn add_tossed_pond_item(&mut self, ram: &mut Ram<'_>, item: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_tossed_pond_item(0x28, item, 5);
    }
    fn hud_refresh_icon(&mut self, ram: &mut Ram<'_>) {
        crate::hud::Hud { ram, runtime: self }.refresh_icon();
    }
    fn palette_assert_translucency_swap(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_assert_translucency_swap());
    }
    fn palette_filter_wish_ponds(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_filter_wish_ponds());
    }
    fn palette_filter_sp5f(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_filter_sp5f());
    }
    fn palette_revert_translucency_swap(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_revert_translucency_swap());
    }
    fn palette_filter_restore_sp5f(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.palette_filter_restore_sp5f());
    }
    fn link_receive_item(&mut self, ram: &mut Ram<'_>, item: u8, arg: u8) {
        ram.write_u8(0x2e9, arg);
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_receive_item(item, -1);
    }
    fn queue_sfx1_with_pan(&mut self, ram: &mut Ram<'_>, slot: usize, sound: u8) {
        crate::misc::sprite_sfx1_with_pan(ram, self, slot, sound);
    }
    fn movable_mantle_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.movable_mantle_draw(slot);
    }
    fn sprite_repel_dash(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .sprite_repel_dash();
    }
    fn draw_guard_head(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        info: &PrepOamCoordsRet,
        oam_offset: usize,
    ) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_draw_guard_head(slot, info, oam_offset);
    }
    fn draw_bnc_body(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        info: &PrepOamCoordsRet,
        oam_offset: usize,
    ) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_draw_bnc_body(slot, info, oam_offset);
    }
    fn draw_bnc_flail(&mut self, ram: &mut Ram<'_>, slot: usize, info: &PrepOamCoordsRet) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_draw_bnc_flail(slot, info);
    }
    fn draw_shadow_custom(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        info: &PrepOamCoordsRet,
        size: u8,
    ) {
        let mut info = *info;
        Sprite::new(ram.bytes_mut(), self).sprite_draw_shadow_custom(slot, &mut info, size);
    }
    fn sprite_cannonball(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }.sprite_cannonball(slot);
    }
    fn bee_handle_interactions(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .bee_handle_interactions(slot);
    }
    fn sprite_find_empty_bottle(&mut self, ram: &mut Ram<'_>) -> i32 {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .sprite_find_empty_bottle()
    }
    fn wizzbeam_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.wizzbeam_draw(slot);
    }
    fn load_follower_graphics(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |g| g.load_follower_graphics());
    }
    fn follower_initialize(&mut self, ram: &mut Ram<'_>) {
        crate::tagalong::initialize(ram, self);
    }
    fn sprite_sidenexx(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.sprite_sidenexx(slot);
    }
    fn trinexx_fire_add_fire_garnish(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_trinexx_fire_add_fire_garnish(slot);
    }
    fn sprite_cc_cd_common(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_cc_cd_common(slot);
    }
    fn sprite_cd_spawn_garnish(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_cd_spawn_garnish(slot);
    }
    fn draw_multiple_player_deferred(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        pieces: &[DrawMultipleData; 4],
    ) {
        let mut info = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(slot);
        sprite.sprite_draw_multiple_player_deferred(slot, pieces, Some(&mut info));
    }
    fn helmasaur_fireball_tri_split(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .helmasaur_fireball_tri_split(slot);
    }
    fn helmasaur_fireball_quad_split(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .helmasaur_fireball_quad_split(slot);
    }
    fn archery_game_guy_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .archery_game_guy_draw(slot);
    }
    fn link_cancel_dash(&mut self, ram: &mut Ram<'_>) {
        crate::sprite_main_bank2a::SpriteMainBank2aRuntime::link_cancel_dash(self, ram);
    }
    fn sprite_show_message_minimal(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).sprite_show_message_minimal();
    }
    fn sprite_initialize_secondary_item_minigame(&mut self, ram: &mut Ram<'_>, kind: u8) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_initialize_secondary_item_minigame(kind);
    }
    fn sprite_direction_and_deltas(&mut self, ram: &mut Ram<'_>, slot: usize) -> (u8, u8, u8) {
        let result = Sprite::new(ram.bytes_mut(), self).sprite_direction_to_face_link(slot);
        (result.0, result.1.a, result.1.b)
    }
    fn sprite_check_damage_to_and_from_link_result(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
    ) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_and_from_link(slot)
    }
    fn sprite_spawn_fireball(&mut self, ram: &mut Ram<'_>, slot: usize) {
        let _ = crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_spawn_fireball(slot);
    }
    fn sprite_spawn_probe_always(&mut self, ram: &mut Ram<'_>, slot: usize, angle: u8) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_probe_always(slot, angle);
    }
    fn sprite_book_of_mudora(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_book_of_mudora(slot);
    }
    fn sprite_lumberjack_tree(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .sprite_lumberjack_tree(slot);
    }
    fn fireball_spawn_trail_garnish(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .fireball_spawn_trail_garnish(slot);
    }
    fn sprite_place_repulse_spark_2(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_place_rupulse_spark_2(slot);
    }
    fn sprite_zora_regurgitate_flippers(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_zora_regurgitate_flippers(slot);
    }
    fn toppo_flustered(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }.toppo_flustered(slot);
    }
    fn probe_check_tile_solidity(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .probe_check_tile_solidity(slot)
    }
    fn priest_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.priest_draw(slot);
    }
    fn uncle_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.uncle_draw(slot);
    }
    fn draw_multiple_player_deferred_slice(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        pieces: &[DrawMultipleData],
    ) -> PrepOamCoordsRet {
        let mut info = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(slot);
        sprite.sprite_draw_multiple_player_deferred(slot, pieces, Some(&mut info));
        info
    }
    fn lady_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.lady_draw(slot);
    }
    fn sprite_chicken_lady(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.sprite_chicken_lady(slot);
    }
    fn snitch_spawn_guard(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.snitch_spawn_guard(slot);
    }
    fn overworld_draw_wooden_door(&mut self, ram: &mut Ram<'_>, position: u16, open: bool) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .overworld_draw_wooden_door(position, open);
    }
    fn running_boy_spawn_dust_garnish(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .running_boy_spawn_dust_garnish(slot);
    }
    fn bottle_merchant_detect_fish(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .bottle_merchant_detect_fish(slot);
    }
    fn bottle_merchant_buy_bee(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .bottle_merchant_buy_bee(slot);
    }
    fn bottle_merchant_buy_fish(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .bottle_merchant_buy_fish(slot);
    }
    fn crystal_maiden_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.crystal_maiden_draw(slot);
    }
    fn save_palace_deaths(&mut self, ram: &mut Ram<'_>) {
        with_dungeon(self, ram, |d| d.save_palace_deaths());
    }
    fn dungeon_flag_room_data_quadrants(&mut self, ram: &mut Ram<'_>) {
        with_dungeon(self, ram, |d| d.dungeon_flag_room_data_quadrants());
    }
    fn sprite_become_follower(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::tagalong::sprite_become_follower(ram, self, slot);
    }
    fn sprite_load_graphics_properties_light_world_only(&mut self, ram: &mut Ram<'_>) {
        crate::overworld::sprite_load_graphics_properties_light_world_only(ram, self.assets);
    }
    fn decode_animated_sprite_tile_variable(&mut self, ram: &mut Ram<'_>, index: u8) {
        with_gfx(self, ram, |g| g.decode_animated_sprite_tile_variable(index));
    }
    fn sprite_draw_water_ripple_with_oam_adjust(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_draw_water_ripple_with_oam_adjust(slot);
    }
    fn sprite_spawn_water_splash(&mut self, ram: &mut Ram<'_>, slot: usize) {
        let _ = crate::sprite_main_bank2a::SpriteMainBank2a { ram, runtime: self }
            .sprite_spawn_water_splash(slot);
    }
    fn dust_cloud_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.dust_cloud_draw(slot);
    }
    fn medallion_tablet_draw(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
            .medallion_tablet_draw(slot);
    }
    fn bombos_tablet_start_cutscene(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .bombos_tablet_start_cutscene();
    }
    fn ether_tablet_start_cutscene(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .ether_tablet_start_cutscene();
    }
    fn landmine_hitboxes_overlap(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        let mut hitbox = crate::sprite::SpriteHitBox::default();
        let sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.player_setup_action_hit_box(&mut hitbox);
        sprite.sprite_setup_hit_box(slot, &mut hitbox);
        Sprite::<Self>::check_if_hit_boxes_overlap(&hitbox)
    }
    fn zora_fireball_shield_overlap(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        x: u16,
        y: u16,
        width: u8,
        height: u8,
    ) -> bool {
        let mut hitbox = crate::sprite::SpriteHitBox::default();
        Sprite::new(ram.bytes_mut(), self).sprite_setup_hit_box(slot, &mut hitbox);
        [hitbox.r0_xlo, hitbox.r8_xhi] = x.to_le_bytes();
        [hitbox.r1_ylo, hitbox.r9_yhi] = y.to_le_bytes();
        hitbox.r2 = width;
        hitbox.r3 = height;
        Sprite::<Self>::check_if_hit_boxes_overlap(&hitbox)
    }
    fn sprite_bomb_hitbox_overlaps(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        x: u16,
        y: u16,
        size: u8,
    ) -> bool {
        let mut hitbox = crate::sprite::SpriteHitBox::default();
        [hitbox.r0_xlo, hitbox.r8_xhi] = x.to_le_bytes();
        [hitbox.r1_ylo, hitbox.r9_yhi] = y.to_le_bytes();
        hitbox.r2 = size;
        hitbox.r3 = size;
        Sprite::new(ram.bytes_mut(), self).sprite_setup_hit_box(slot, &mut hitbox);
        Sprite::<Self>::check_if_hit_boxes_overlap(&hitbox)
    }
    fn sprite_return_if_lifted(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_lifted(slot);
    }

    fn draw_single_large(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_draw_single_large(slot);
    }

    fn draw_single_small(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_draw_single_small(slot);
    }

    fn queue_sfx2_with_pan(&mut self, ram: &mut Ram<'_>, slot: usize, sound: u8) {
        let x = u16::from(ram.read_u8(0xd10 + slot)) | (u16::from(ram.read_u8(0xd30 + slot)) << 8);
        if ram.read_u8(0x12e) == 0 {
            ram.write_u8(0x12e, sound | crate::misc::calculate_sfx_pan(ram, x));
        }
    }

    fn queue_sfx3_with_pan(&mut self, ram: &mut Ram<'_>, slot: usize, sound: u8) {
        let x = u16::from(ram.read_u8(0xd10 + slot)) | (u16::from(ram.read_u8(0xd30 + slot)) << 8);
        if ram.read_u8(0x12f) == 0 {
            ram.write_u8(0x12f, sound | crate::misc::calculate_sfx_pan(ram, x));
        }
    }

    fn sprite_return_if_inactive(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_inactive(slot)
    }

    fn sprite_return_if_paused(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_paused(slot)
    }

    fn sprite_return_if_recoiling(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_return_if_recoiling(slot)
    }

    fn sprite_check_if_screen_is_clear(&mut self, ram: &mut Ram<'_>) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_if_screen_is_clear()
    }

    fn sprite_check_if_link_is_busy(&mut self, ram: &mut Ram<'_>) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_if_link_is_busy()
    }

    fn sprite_move_x(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_x(slot);
    }

    fn sprite_move_y(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_y(slot);
    }

    fn sprite_move_xy(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_xy(slot);
    }

    fn sprite_move_z(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_move_z(slot);
    }

    fn sprite_attempt_damage_to_link_plus_recoil(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_attempt_damage_to_link_plus_recoil(slot);
    }

    fn sprite_nullify_hookshot_drag(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).sprite_nullify_hookshot_drag();
    }

    fn sprite_check_damage_to_link_same_layer(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link_same_layer(slot)
    }

    fn sprite_check_damage_to_link(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_link(slot)
    }

    fn sprite_check_damage_from_link(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_from_link(slot);
    }

    fn sprite_check_damage_to_and_from_link(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_check_damage_to_and_from_link(slot);
    }

    fn sprite_check_tile_collision(&mut self, ram: &mut Ram<'_>, slot: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self).sprite_check_tile_collision(slot)
    }

    fn sprite_kill_self(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_kill_self(slot);
    }

    fn thrown_sprite_tile_and_sprite_interaction(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).thrown_sprite_tile_and_sprite_interaction(slot);
    }

    fn sprite_apply_speed_towards_link(&mut self, ram: &mut Ram<'_>, slot: usize, speed: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_apply_speed_towards_link(slot, speed);
    }

    fn sprite_project_speed_towards_location(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        x: u16,
        y: u16,
        speed: u8,
    ) -> ProjectSpeedRet {
        Sprite::new(ram.bytes_mut(), self).sprite_project_speed_towards_location(slot, x, y, speed)
    }

    fn sprite_get_16_bit_coords(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_get_16_bit_coords(slot);
    }

    fn sprite_prep_load_palette(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_prep_load_palette(slot);
    }

    fn sprite_correct_oam_entries(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        last_entry: usize,
        big: u8,
    ) {
        Sprite::new(ram.bytes_mut(), self).sprite_correct_oam_entries(slot, last_entry, big);
    }

    fn sprite_draw_distress_custom(&mut self, ram: &mut Ram<'_>, x: u16, y: u16, frame: u8) {
        Sprite::new(ram.bytes_mut(), self).sprite_draw_distress_custom(x, y, frame);
    }

    fn sprite_draw_thin_and_tall(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_draw_thin_and_tall(slot);
    }

    fn sprite_draw_shadow(&mut self, ram: &mut Ram<'_>, slot: usize, info: &PrepOamCoordsRet) {
        let mut info = *info;
        Sprite::new(ram.bytes_mut(), self).sprite_draw_shadow(slot, &mut info);
    }

    fn sprite_direction_to_face_link(&mut self, ram: &mut Ram<'_>, slot: usize) -> (u8, u8) {
        let result = Sprite::new(ram.bytes_mut(), self).sprite_direction_to_face_link(slot);
        (result.0, result.1.b)
    }

    fn sprite_direction_to_face_link_direction(&mut self, ram: &mut Ram<'_>, slot: usize) -> u8 {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_direction_to_face_link(slot)
            .0
    }

    fn sprite_is_below_link(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_is_below_link(slot)
            .a
            != 0
    }

    fn sprite_is_right_of_link(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self)
            .sprite_is_right_of_link(slot)
            .a
            != 0
    }

    fn sprite_add_xy(&mut self, ram: &mut Ram<'_>, slot: usize, x: i8, y: i8) {
        Sprite::new(ram.bytes_mut(), self).sprite_add_xy(slot, i16::from(x), i16::from(y));
    }

    fn sprite_behave_as_barrier(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).sprite_behave_as_barrier(slot);
    }

    fn sprite_track_body_to_head(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        Sprite::new(ram.bytes_mut(), self).sprite_track_body_to_head(slot)
    }

    fn guard_parry_sword_attacks(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).guard_parry_sword_attacks(slot);
    }

    fn get_tile_attribute(&mut self, ram: &mut Ram<'_>, floor: u8, x: &mut u16, y: u16) -> u8 {
        Sprite::new(ram.bytes_mut(), self).get_tile_attribute(floor, x, y)
    }

    fn ancilla_check_damage_to_sprite_preset(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
        damage: u8,
    ) {
        Sprite::new(ram.bytes_mut(), self).ancilla_check_damage_to_sprite_preset(slot, damage);
    }

    fn prep_oam_coord(&mut self, ram: &mut Ram<'_>, slot: usize) -> PrepOamCoordsRet {
        let mut result = PrepOamCoordsRet::default();
        Sprite::new(ram.bytes_mut(), self).sprite_prep_oam_coord(slot, &mut result);
        result
    }

    fn prep_oam_coord_or_double_ret(
        &mut self,
        ram: &mut Ram<'_>,
        slot: usize,
    ) -> Option<PrepOamCoordsRet> {
        let mut result = PrepOamCoordsRet::default();
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        (!sprite.sprite_prep_oam_coord_or_double_ret(slot, &mut result)).then_some(result)
    }

    fn oam_allocate_from_region_a(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_a(bytes);
    }

    fn oam_allocate_from_region_b(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_b(bytes);
    }

    fn oam_allocate_from_region_c(&mut self, ram: &mut Ram<'_>, priority: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_c(priority);
    }

    fn oam_allocate_defer_to_player(&mut self, ram: &mut Ram<'_>, slot: usize) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_defer_to_player(slot);
    }

    fn draw_multiple(&mut self, ram: &mut Ram<'_>, slot: usize, pieces: &[DrawMultipleData]) {
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(slot);
        sprite.sprite_draw_multiple(slot, pieces, None);
    }

    fn sprite_spawn_dynamically(
        &mut self,
        ram: &mut Ram<'_>,
        parent: usize,
        kind: u8,
    ) -> Option<(usize, SpriteSpawnInfo)> {
        let mut info = SpriteSpawnInfo::default();
        let slot =
            Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically(parent, kind, &mut info);
        (slot >= 0).then_some((slot as usize, info))
    }

    fn sprite_spawn_dynamically_ex(
        &mut self,
        ram: &mut Ram<'_>,
        parent: usize,
        kind: u8,
        limit: usize,
    ) -> Option<(usize, SpriteSpawnInfo)> {
        let mut info = SpriteSpawnInfo::default();
        let slot = Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically_ex(
            parent,
            kind,
            &mut info,
            limit as i32,
        );
        (slot >= 0).then_some((slot as usize, info))
    }

    fn sprite_convert_velocity_to_angle(&mut self, x_velocity: u8, y_velocity: u8) -> u8 {
        Sprite::<Self>::sprite_convert_velocity_to_angle(x_velocity, y_velocity)
    }
}

impl AncillaRuntime for NativeGameServices<'_> {
    fn allocate_oam(&mut self, ram: &mut Ram<'_>, k: usize, size: u8) -> u8 {
        let sorted = ram.read_u8(0xfb3) != 0;
        let floor = ram.read_u8(0xc7c + k);
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        if sorted {
            if floor != 0 {
                sprite.oam_allocate_from_region_f(size)
            } else {
                sprite.oam_allocate_from_region_d(size)
            }
        } else {
            sprite.oam_allocate_from_region_a(size)
        }
    }

    fn hud_refresh_icon(&mut self, ram: &mut Ram<'_>) {
        crate::hud::Hud { ram, runtime: self }.refresh_icon();
    }

    fn load_gear_palettes_bunny(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.load_gear_palettes_bunny());
    }

    fn palette_load_sprite_environment_dungeon(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| {
            gfx.palette_load_sprite_environment_dungeon()
        });
    }

    fn load_actual_gear_palettes(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.load_actual_gear_palettes());
    }

    fn terminate_sparkle_objects(&mut self, ram: &mut Ram<'_>) {
        for slot in 0..5 {
            if matches!(
                ram.read_u8(0xc4a + slot),
                0x2a | 0x2b | 0x30 | 0x31 | 0x18 | 0x19 | 0x0c
            ) {
                ram.write_u8(0xc4a + slot, 0);
            }
        }
    }

    fn link_receive_item(&mut self, ram: &mut Ram<'_>, item: u8) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_receive_item(item, 0);
    }

    fn follower_initialize(&mut self, ram: &mut Ram<'_>) {
        crate::tagalong::initialize(ram, self);
    }

    fn link_reset_properties_a(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_reset_properties_a();
    }

    fn palette_load_link_armor_and_gloves(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.palette_load_link_armor_and_gloves());
    }

    fn prepare_dungeon_exit_from_boss_fight(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self).prepare_dungeon_exit_from_boss_fight();
    }

    fn main_show_text_message(&mut self, ram: &mut Ram<'_>) {
        crate::misc::main_show_text_message(ram);
    }

    fn write_to_4bpp_buffer(&mut self, ram: &mut Ram<'_>, tile: u8) {
        with_gfx(self, ram, |gfx| gfx.write_to_4bpp_buffer_at_7f4000(tile));
    }

    fn decode_animated_sprite_tile(&mut self, ram: &mut Ram<'_>, graphics: u8) {
        with_gfx(self, ram, |gfx| {
            gfx.decode_animated_sprite_tile_variable(graphics)
        });
    }

    fn decode_animated_sprite_tile_variable(&mut self, ram: &mut Ram<'_>, tile: u8) {
        with_gfx(self, ram, |gfx| {
            gfx.decode_animated_sprite_tile_variable(tile)
        });
    }

    fn player_tile_detect_nearby(&mut self, ram: &mut Ram<'_>) {
        crate::tile_detect::detect_nearby(ram, self);
    }

    fn overworld_alter_weathervane(&mut self, ram: &mut Ram<'_>) {
        crate::overworld::alter_weathervane(ram, self.assets, |ram| {
            crate::overworld::overworld_do_map_update32x32_b(ram, self.assets);
        });
    }

    fn palette_restore_bg_and_hud(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.palette_restore_bg_and_hud());
    }

    fn palette_electro_themed_gear(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.palette_electro_themed_gear());
    }

    fn filter_majorly_whiten_bg(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.filter_majorly_whiten_bg());
    }

    fn palette_restore_bg_from_flash(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.palette_restore_bg_from_flash());
    }

    fn sprite_project_speed_towards_link(
        &mut self,
        ram: &mut Ram<'_>,
        sprite: usize,
        velocity: u8,
    ) -> crate::ancilla::ProjectSpeed {
        let value =
            Sprite::new(ram.bytes_mut(), self).sprite_project_speed_towards_link(sprite, velocity);
        crate::ancilla::ProjectSpeed {
            x: value.x,
            y: value.y,
            xdiff: value.xdiff,
            ydiff: value.ydiff,
        }
    }

    fn sprite_project_speed_towards_location(
        &mut self,
        ram: &mut Ram<'_>,
        sprite: usize,
        x: u16,
        y: u16,
        velocity: u8,
    ) -> crate::ancilla::ProjectSpeed {
        let value = Sprite::new(ram.bytes_mut(), self)
            .sprite_project_speed_towards_location(sprite, x, y, velocity);
        crate::ancilla::ProjectSpeed {
            x: value.x,
            y: value.y,
            xdiff: value.xdiff,
            ydiff: value.ydiff,
        }
    }

    fn sprite_hitbox_overlaps(
        &mut self,
        ram: &mut Ram<'_>,
        sprite: usize,
        ancilla: AncillaHitBox,
    ) -> bool {
        let mut hitbox = SpriteHitBox {
            r0_xlo: ancilla.x as u8,
            r8_xhi: (ancilla.x >> 8) as u8,
            r1_ylo: ancilla.y as u8,
            r9_yhi: (ancilla.y >> 8) as u8,
            r2: ancilla.width,
            r3: ancilla.height,
            ..SpriteHitBox::default()
        };
        let sprites = Sprite::new(ram.bytes_mut(), self);
        sprites.sprite_setup_hit_box(sprite, &mut hitbox);
        Sprite::<Self>::check_if_hit_boxes_overlap(&hitbox)
    }

    fn sprite_create_deflected_arrow(&mut self, ram: &mut Ram<'_>, ancilla: usize) {
        ram.write_u8(0xc4a + ancilla, 0);
        let mut info = SpriteSpawnInfo::default();
        let slot =
            Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically(ancilla, 0x1b, &mut info);
        if slot < 0 {
            return;
        }
        let slot = slot as usize;
        for (dst, src) in [
            (0xd10, 0xc04),
            (0xd30, 0xc18),
            (0xd00, 0xbfa),
            (0xd20, 0xc0e),
            (0xd50, 0xc2c),
            (0xd40, 0xc22),
        ] {
            ram.write_u8(dst + slot, ram.read_u8(src + ancilla));
        }
        ram.write_u8(0xdd0 + slot, 6);
        ram.write_u8(0xdf0 + slot, 31);
        ram.write_u8(0xf20 + slot, ram.read_u8(0xee));
        Sprite::new(ram.bytes_mut(), self).sprite_place_weapon_tink(slot);
    }

    fn check_damage_to_sprite_preset(&mut self, ram: &mut Ram<'_>, sprite: usize, damage: u8) {
        Sprite::new(ram.bytes_mut(), self).ancilla_check_damage_to_sprite_preset(sprite, damage);
    }

    fn refund_magic(&mut self, ram: &mut Ram<'_>, amount: u8) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .refund_magic(amount);
    }

    fn dungeon_light_torch(&mut self, ram: &mut Ram<'_>) {
        crate::misc::dungeon_light_torch(ram, self);
    }

    fn check_ability_to_swim(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .check_ability_to_swim();
    }

    fn bomb_check_for_destructibles(&mut self, ram: &mut Ram<'_>, x: u16, y: u16, ancilla: usize) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self).bomb_check_for_destructibles(
            x,
            y,
            ancilla as u8,
        );
    }

    fn generated_bombos_position(&mut self, frame: u8) -> u8 {
        self.assets.bytes(72)[usize::from(frame)]
    }

    fn generated_wish_pond_item(&mut self, item: u8) -> u8 {
        self.assets.bytes(71)[usize::from(item)]
    }

    fn sprite_spawn_poof_garnish(&mut self, ram: &mut Ram<'_>, sprite: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_poof_garnish(sprite);
    }

    fn hookshot_check_tile_collision(&mut self, ram: &mut Ram<'_>, ancilla: usize) {
        crate::tile_detect::hookshot_collision(ram, self, ancilla);
    }

    fn overworld_offset_base(&mut self, area: u8) -> (u16, u16) {
        crate::overworld::offset_base(usize::from(area))
    }

    fn prepare_apply_rumble_to_sprites(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).prepare_apply_rumble_to_sprites();
    }

    fn receive_item(&mut self, ram: &mut Ram<'_>, item: u8) {
        AncillaRuntime::link_receive_item(self, ram, item);
    }

    fn audio_port0(&mut self) -> u8 {
        self.audio_port0
    }

    fn spawn_sprite_dynamically(
        &mut self,
        ram: &mut Ram<'_>,
        source: usize,
        kind: u8,
    ) -> Option<usize> {
        let mut info = SpriteSpawnInfo::default();
        let slot =
            Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically(source, kind, &mut info);
        (slot >= 0).then_some(slot as usize)
    }

    fn decompress_shield_graphics(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.decompress_shield_graphics());
    }

    fn decompress_sword_graphics(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.decompress_sword_graphics());
    }

    fn overworld_do_map_update_32x32_b(&mut self, ram: &mut Ram<'_>) {
        crate::overworld::overworld_do_map_update32x32_b(ram, self.assets);
    }

    fn overworld_tile_attribute(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8 {
        crate::tile_detect::overworld_tile_attribute(ram, self, x, y)
    }

    fn dungeon_tile_attribute(
        &mut self,
        ram: &mut Ram<'_>,
        floor: u8,
        mut x: u16,
        y: u16,
    ) -> (u8, u16) {
        let attribute = Sprite::new(ram.bytes_mut(), self).get_tile_attribute(floor, &mut x, y);
        (attribute, x)
    }

    fn sloped_tile_collision(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> bool {
        Sprite::new(ram.bytes_mut(), self).entity_check_sloped_tile_collision(x, y)
    }

    fn allocate_oam_region(&mut self, ram: &mut Ram<'_>, region: u8, size: u8) {
        let mut sprite = Sprite::new(ram.bytes_mut(), self);
        match region {
            b'A' => {
                sprite.oam_allocate_from_region_a(size);
            }
            b'B' => {
                sprite.oam_allocate_from_region_b(size);
            }
            b'C' => {
                sprite.oam_allocate_from_region_c(size);
            }
            b'D' => {
                sprite.oam_allocate_from_region_d(size);
            }
            b'E' => {
                sprite.oam_allocate_from_region_e(size);
            }
            b'F' => {
                sprite.oam_allocate_from_region_f(size);
            }
            _ => unreachable!("ancilla OAM region is A through F"),
        }
    }
}

impl PlayerRuntime for NativeGameServices<'_> {
    fn ancilla_check_link_collision(&mut self, ram: &mut Ram<'_>, slot: usize, class: u8) -> bool {
        crate::ancilla::Ancilla { ram, runtime: self }
            .check_link_collision_class(slot, usize::from(class))
    }

    fn overworld_do_map_update_32x32_b(&mut self, ram: &mut Ram<'_>) {
        crate::overworld::overworld_do_map_update32x32_b(ram, self.assets);
    }

    fn handle_item_tile_action_overworld(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8 {
        crate::misc::handle_item_tile_action_overworld(ram, self, x, y)
    }

    fn handle_nudging_in_a_door(&mut self, ram: &mut Ram<'_>, speed: i8) {
        crate::tile_detect::nudge_in_door(ram, self, speed);
    }

    fn set_and_save_visited_quadrant_flags(&mut self, ram: &mut Ram<'_>) {
        with_dungeon(self, ram, |dungeon| {
            dungeon.set_and_save_visited_quadrant_flags()
        });
    }

    fn dungeon_flag_room_data_quadrants(&mut self, ram: &mut Ram<'_>) {
        with_dungeon(self, ram, |dungeon| {
            dungeon.dungeon_flag_room_data_quadrants()
        });
    }

    fn overworld_get_pit_destination(&mut self, ram: &mut Ram<'_>) {
        crate::overworld::get_pit_destination(ram, self.assets);
    }

    fn take_damage_from_pit(&mut self, ram: &mut Ram<'_>) {
        crate::overworld::take_damage_from_pit(ram);
    }

    fn link_oam_main(&mut self, ram: &mut Ram<'_>) {
        crate::player_oam::main(ram, self);
    }

    fn follower_initialize(&mut self, ram: &mut Ram<'_>) {
        crate::tagalong::initialize(ram, self);
    }

    fn dungeon_start_inter_room_transition(&mut self, ram: &mut Ram<'_>, direction: u8) {
        with_dungeon(self, ram, |dungeon| match direction {
            0 => dungeon.handle_edge_transition_movement_east_right_by8(),
            1 => dungeon.dung_start_inter_room_trans_left_plus(),
            2 => dungeon.handle_edge_transition_movement_south_down_by16(),
            3 => dungeon.dungeon_start_inter_room_trans_up(),
            _ => unreachable!("dungeon transition direction is two bits"),
        });
    }

    fn adjust_quadrant_and_camera(&mut self, ram: &mut Ram<'_>, direction: u8) {
        with_dungeon(self, ram, |dungeon| match direction {
            0 => dungeon.adjust_quadrant_and_camera_right(),
            1 => dungeon.adjust_quadrant_and_camera_left(),
            2 => dungeon.adjust_quadrant_and_camera_down(),
            3 => dungeon.adjust_quadrant_and_camera_up(),
            _ => unreachable!("camera direction is two bits"),
        });
    }

    fn load_gear_palettes_bunny(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.load_gear_palettes_bunny());
    }

    fn ancilla_add_gravestone(&mut self, ram: &mut Ram<'_>) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_gravestone(0x24, 1);
    }

    fn ancilla_add_dash_tremor(&mut self, ram: &mut Ram<'_>) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_dash_tremor(29, 1);
    }

    fn prepare_apply_rumble_to_sprites(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).prepare_apply_rumble_to_sprites();
    }

    fn mirror_save_room_data(&mut self, ram: &mut Ram<'_>) {
        with_dungeon(self, ram, |dungeon| dungeon.mirror_save_room_data());
    }

    fn ancilla_move_y(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::ancilla::Ancilla { ram, runtime: self }.move_y(slot);
    }

    fn ancilla_move_x(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::ancilla::Ancilla { ram, runtime: self }.move_x(slot);
    }

    fn ancilla_move_z(&mut self, ram: &mut Ram<'_>, slot: usize) {
        crate::ancilla::Ancilla { ram, runtime: self }.move_z(slot);
    }

    fn ancilla_check_tile_collision_class2(&mut self, ram: &mut Ram<'_>, slot: usize) -> bool {
        crate::ancilla::Ancilla { ram, runtime: self }.check_tile_collision_class2(slot)
    }

    fn ancilla_sfx2_pan(&mut self, ram: &mut Ram<'_>, slot: usize, sound: u8) {
        crate::ancilla::Ancilla { ram, runtime: self }.sfx2_pan(slot, sound);
    }

    fn ancilla_sfx3_pan(&mut self, ram: &mut Ram<'_>, slot: usize, sound: u8) {
        crate::ancilla::Ancilla { ram, runtime: self }.sfx3_pan(slot, sound);
    }

    fn sprite_nullify_hookshot_drag(&mut self, ram: &mut Ram<'_>) {
        Sprite::new(ram.bytes_mut(), self).sprite_nullify_hookshot_drag();
    }

    fn tile_detect_movement_vertical_slopes(&mut self, ram: &mut Ram<'_>, direction: u8) {
        crate::tile_detect::movement_vertical_slopes(ram, self, usize::from(direction));
    }

    fn tile_detect_movement_horizontal_slopes(&mut self, ram: &mut Ram<'_>, direction: u8) {
        crate::tile_detect::movement_horizontal_slopes(ram, self, usize::from(direction));
    }

    fn tile_detect_movement_x(&mut self, ram: &mut Ram<'_>, direction: u8) {
        crate::tile_detect::movement_x(ram, self, usize::from(direction));
    }

    fn tile_detect_movement_y(&mut self, ram: &mut Ram<'_>, direction: u8) {
        crate::tile_detect::movement_y(ram, self, usize::from(direction));
    }

    fn player_tile_detect_nearby(&mut self, ram: &mut Ram<'_>) {
        crate::tile_detect::detect_nearby(ram, self);
    }

    fn tile_detect_reset_state(&mut self, ram: &mut Ram<'_>) {
        crate::tile_detect::reset_state(ram);
    }

    fn tile_detection_execute(&mut self, ram: &mut Ram<'_>, x: u16, y: u16, bits: u8) {
        crate::tile_detect::execute(ram, self, x, y, u16::from(bits));
    }

    fn tile_detect_execute_inner(
        &mut self,
        ram: &mut Ram<'_>,
        tile: u8,
        offset: u8,
        bits: u8,
        indoors: bool,
    ) {
        crate::tile_detect::execute_inner(ram, tile, usize::from(offset), u16::from(bits), indoors);
    }

    fn enhanced_features0(&self) -> u32 {
        self.enhanced_features0
    }

    fn ancilla_add_splash(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) -> bool {
        crate::ancilla::Ancilla { ram, runtime: self }.add_splash(kind, limit)
    }

    fn dungeon_delete_rupee_tile(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) {
        with_dungeon(self, ram, |dungeon| dungeon.dungeon_delete_rupee_tile(x, y));
    }

    fn open_chest_for_item(&mut self, ram: &mut Ram<'_>, tile: u16) {
        with_dungeon(self, ram, |dungeon| {
            let mut chest_position = 0;
            let _ = dungeon.open_chest_for_item(tile as u8, &mut chest_position);
        });
    }

    fn draw_push_block_oam(&mut self, ram: &mut Ram<'_>, x: i16, y: i16, character: u8) {
        Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_b(4);
        let entry = (usize::from(ram.read_u16(0x90)) - crate::oam::OAM_BUFFER) / 4;
        crate::oam::set_helper0(ram, entry, x as u16, y as u16, character, 0x20, 2);
    }

    fn overworld_smash_rock_pile(
        &mut self,
        ram: &mut Ram<'_>,
        second: bool,
    ) -> Option<(u8, u16, u16)> {
        let tile = crate::overworld::smash_rock_pile(ram, self.assets, second, |ram| {
            crate::overworld::update_map_32x32(ram, self.assets)
        })?;
        Some((tile, ram.read_u16(2), ram.read_u16(0)))
    }

    fn sprite_spawn_immediately_smashed_terrain(
        &mut self,
        ram: &mut Ram<'_>,
        kind: u8,
        x: u16,
        y: u16,
    ) {
        Sprite::new(ram.bytes_mut(), self).sprite_spawn_immediately_smashed_terrain(kind, x, y);
    }

    fn ancilla_sfx2_near(&mut self, ram: &mut Ram<'_>, sound: u8) -> u8 {
        crate::misc::ancilla_sfx2_near(ram, sound)
    }

    fn ancilla_sfx3_near(&mut self, ram: &mut Ram<'_>, sound: u8) {
        crate::misc::ancilla_sfx3_near(ram, sound);
    }

    fn ancilla_add_ether_spell(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_ether_spell(kind, limit);
    }

    fn ancilla_add_bombos_spell(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_bombos_spell(kind, limit);
    }

    fn ancilla_add_quake_spell(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_quake_spell(kind, limit);
    }

    fn ancilla_add_cape_poof(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_cape_poof(kind, limit);
    }

    fn ancilla_add_item_receipt(
        &mut self,
        ram: &mut Ram<'_>,
        kind: u8,
        limit: u8,
        chest_position: i32,
    ) {
        crate::misc::ancilla_add_item_receipt(ram, self, kind, limit, chest_position);
    }

    fn link_zap_handle_mosaic(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.link_zap_handle_mosaic());
    }

    fn palette_electro_themed_gear(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.palette_electro_themed_gear());
    }

    fn player_set_custom_mosaic_level(&mut self, ram: &mut Ram<'_>, level: u8) {
        with_gfx(self, ram, |gfx| gfx.player_set_custom_mosaic_level(level));
    }

    fn ancilla_add_blanket(&mut self, ram: &mut Ram<'_>, kind: u8) {
        crate::ancilla::Ancilla { ram, runtime: self }.add_blanket(kind);
    }

    fn ancilla_add_snoring(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_snoring(kind, limit);
    }

    fn ancilla_add(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) -> Option<usize> {
        crate::ancilla::Ancilla { ram, runtime: self }.ancilla_add_add_ancilla_bank08(kind, limit)
    }

    fn decode_animated_sprite_tile_variable(&mut self, ram: &mut Ram<'_>, tile: u8) {
        with_gfx(self, ram, |gfx| {
            gfx.decode_animated_sprite_tile_variable(tile)
        });
    }

    fn ancilla_set_xy(&mut self, ram: &mut Ram<'_>, slot: usize, x: u16, y: u16) {
        let mut ancilla = crate::ancilla::Ancilla { ram, runtime: self };
        ancilla.set_x(slot, x);
        ancilla.set_y(slot, y);
    }

    fn ancilla_terminate_select_interactives(&mut self, ram: &mut Ram<'_>, selected: u8) {
        let _ =
            crate::ancilla::Ancilla { ram, runtime: self }.terminate_select_interactives(selected);
    }

    fn ancilla_add_bomb(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_bomb(kind, limit);
    }

    fn release_fairy(&mut self, ram: &mut Ram<'_>) -> bool {
        Sprite::new(ram.bytes_mut(), self).release_fairy() >= 0
    }

    fn release_bee_from_bottle(&mut self, ram: &mut Ram<'_>, index: usize) -> bool {
        crate::sprite_main_bank2b::SpriteMainBank2b { ram, runtime: self }
            .release_bee_from_bottle(index)
            >= 0
    }

    fn hud_refresh_icon(&mut self, ram: &mut Ram<'_>) {
        crate::hud::Hud { ram, runtime: self }.refresh_icon();
    }

    fn hud_rebuild(&mut self, ram: &mut Ram<'_>) {
        crate::hud::Hud { ram, runtime: self }.rebuild();
    }

    fn dungeon_light_torch(&mut self, ram: &mut Ram<'_>) {
        crate::misc::dungeon_light_torch(ram, self);
    }

    fn dungeon_get_tele_msg(&mut self, ram: &mut Ram<'_>, room: u16) -> u16 {
        let mut result = 0;
        with_dungeon(self, ram, |dungeon| {
            result = dungeon.dungeon_get_tele_msg(usize::from(room));
        });
        result
    }

    fn overworld_get_sign_text(&mut self, _: &mut Ram<'_>, screen: u16) -> u16 {
        crate::overworld::get_sign_text(self.assets, usize::from(screen))
    }

    fn load_follower_graphics(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.load_follower_graphics());
    }

    fn tile_check_for_mirror_bonk(&mut self, ram: &mut Ram<'_>) {
        crate::tile_detect::check_mirror_bonk(ram, self);
    }

    fn ancilla_add_magic_powder(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_magic_powder(kind, limit);
    }

    fn ancilla_add_lamp_flame(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_lamp_flame(kind, limit);
    }

    fn ancilla_add_exploding_weather_vane(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ =
            crate::ancilla::Ancilla { ram, runtime: self }.add_exploding_weathervane(kind, limit);
    }

    fn ancilla_add_duck_take_off(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_duck_take_off(kind, limit);
    }

    fn ancilla_add_dwarf_poof(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_dwarf_poof(kind, limit);
    }

    fn ancilla_add_bunny_poof(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_bunny_poof(kind, limit);
    }

    fn ancilla_add_fire_rod_shot(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_fire_rod_shot(kind, limit);
    }

    fn ancilla_add_ice_rod_shot(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_ice_rod_shot(kind, limit);
    }

    fn ancilla_add_arrow(
        &mut self,
        ram: &mut Ram<'_>,
        kind: u8,
        direction: u8,
        limit: u8,
        x: u16,
        y: u16,
    ) -> Option<usize> {
        crate::ancilla::Ancilla { ram, runtime: self }.add_arrow(kind, direction, limit, x, y)
    }

    fn ancilla_add_boomerang(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) -> i32 {
        i32::from(crate::ancilla::Ancilla { ram, runtime: self }.add_boomerang(kind, limit))
    }

    fn ancilla_add_hit_stars(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_hit_stars(kind, limit);
    }

    fn random_number(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::get_random_number(ram)
    }

    fn overworld_read_tile_attribute(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8 {
        crate::overworld::read_tile_attribute(ram, self.assets.bytes(164), x, y)
    }

    fn sprite_spawn_small_splash(&mut self, ram: &mut Ram<'_>, slot: u8) -> Option<usize> {
        let spawned = crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .sprite_spawn_small_splash(usize::from(slot));
        (spawned >= 0).then_some(spawned as usize)
    }

    fn sprite_set_x(&mut self, ram: &mut Ram<'_>, slot: usize, x: u16) {
        Sprite::new(ram.bytes_mut(), self).sprite_set_x(slot, x);
    }

    fn sprite_set_y(&mut self, ram: &mut Ram<'_>, slot: usize, y: u16) {
        Sprite::new(ram.bytes_mut(), self).sprite_set_y(slot, y);
    }

    fn ancilla_add_somaria_block(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) -> bool {
        crate::ancilla::Ancilla { ram, runtime: self }
            .ancilla_add_somaria_block(kind, limit)
            .is_some()
    }

    fn current_item_button_override(&mut self, _: &mut Ram<'_>) -> Option<(u8, Option<u8>, bool)> {
        None
    }

    fn sprite_spawn_dynamically(
        &mut self,
        ram: &mut Ram<'_>,
        source: usize,
        kind: u8,
    ) -> Option<usize> {
        let mut info = SpriteSpawnInfo::default();
        let slot =
            Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically(source, kind, &mut info);
        (slot >= 0).then_some(slot as usize)
    }

    fn sprite_sfx3_with_pan(&mut self, ram: &mut Ram<'_>, slot: usize, sound: u8) {
        crate::misc::sprite_sfx3_with_pan(ram, self, slot, sound);
    }

    fn calculate_sfx_pan(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::link_calculate_sfx_pan(ram)
    }

    fn calculate_sword_hit_box(&mut self, ram: &mut Ram<'_>) {
        crate::player_oam::calculate_sword_hit_box(ram);
    }

    fn add_sword_beam(&mut self, ram: &mut Ram<'_>, limit: u8) {
        crate::ancilla::Ancilla { ram, runtime: self }.spawn_sword_beam(limit);
    }

    fn tile_detect_sword_swing_deep_in_door(&mut self, ram: &mut Ram<'_>, doorway: u8) {
        crate::tile_detect::sword_swing_deep_in_door(ram, self, doorway);
    }

    fn ancilla_spawn_sword_charge_sparkle(&mut self, ram: &mut Ram<'_>) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.spawn_sword_charge_sparkle();
    }

    fn ancilla_add_charged_spin_attack_sparkle(&mut self, ram: &mut Ram<'_>) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_charged_spin_attack_sparkle();
    }

    fn ancilla_add_wall_tap_spark(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_wall_tap_spark(kind, limit);
    }

    fn ancilla_add_sword_swing_sparkle(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_sword_swing_sparkle(kind, limit);
    }

    fn ancilla_add_spin_attack_init_spark(&mut self, ram: &mut Ram<'_>, kind: u8, step: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }
            .add_spin_attack_init_spark(kind, step, 0);
    }

    fn ancilla_add_dash_dust(&mut self, ram: &mut Ram<'_>, charging: bool, kind: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_dashing_dust_ex(
            kind,
            limit,
            u8::from(charging),
        );
    }

    fn ancilla_add_falling_prize(&mut self, ram: &mut Ram<'_>, kind: u8, arg: u8, limit: u8) {
        let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_falling_prize(kind, arg, limit);
    }

    fn dungeon_is_pit_that_hurts_player(&mut self, ram: &mut Ram<'_>) -> bool {
        let mut result = false;
        with_dungeon(self, ram, |dungeon| {
            result = dungeon.dungeon_is_pit_that_hurts_player();
        });
        result
    }

    fn dungeon_check_for_liftable_tile(&mut self, ram: &mut Ram<'_>) -> u8 {
        let mut result = 0xff;
        with_dungeon(self, ram, |dungeon| {
            result = dungeon.dungeon_check_for_and_id_liftable_tile() as u8;
        });
        result
    }

    fn liftable_probe_offset(&self, direction: usize) -> (i8, i8) {
        (
            crate::dungeon::DUNGEON_QUERY_IF_TILE_LIFTABLE_X[direction],
            crate::dungeon::DUNGEON_QUERY_IF_TILE_LIFTABLE_Y[direction],
        )
    }

    fn open_chest(&mut self, ram: &mut Ram<'_>, tile: u8) -> Option<(u8, i32)> {
        let mut item = 0xff;
        let mut chest_position = u16::MAX;
        with_dungeon(self, ram, |dungeon| {
            item = dungeon.open_chest_for_item(tile, &mut chest_position);
        });
        (item as i8 >= 0).then_some((item, i32::from(chest_position)))
    }

    fn item_memory_location(&self, item: u8) -> usize {
        crate::misc::ITEM_TARGETS[usize::from(item)]
    }

    fn lift_and_replace_tile(&mut self, ram: &mut Ram<'_>, indoors: bool) -> (u8, u16, u16) {
        let mut point = (0, 0);
        let attribute = if indoors {
            let mut result = 0xff;
            with_dungeon(self, ram, |dungeon| {
                result = dungeon.dungeon_lift_and_replace_liftable(&mut point);
            });
            result
        } else {
            crate::overworld::handle_liftable_tiles(ram, self.assets, |ram| {
                crate::overworld::update_map_32x32(ram, self.assets)
            })
        };
        (attribute, point.0, point.1)
    }

    fn sprite_spawn_throwable_terrain(&mut self, ram: &mut Ram<'_>, kind: usize, x: u16, y: u16) {
        Sprite::new(ram.bytes_mut(), self).sprite_spawn_throwable_terrain(kind as u8, x, y);
    }

    fn load_actual_gear_palettes(&mut self, ram: &mut Ram<'_>) {
        with_gfx(self, ram, |gfx| gfx.load_actual_gear_palettes());
    }

    fn main_show_text_message(&mut self, ram: &mut Ram<'_>) {
        crate::misc::main_show_text_message(ram);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::assets::RuntimeAssets;
    use z3_core::WRAM_SIZE;

    fn with_native_services(test: impl FnOnce(&mut NativeGameServices<'_>)) {
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .expect("compiled assets");
        let assets = RuntimeAssets::parse(&asset_bytes).expect("valid compiled assets");
        test(&mut NativeGameServices::new(&assets));
    }

    fn method_names(source: &str, marker: &str) -> BTreeSet<String> {
        let start = source.find(marker).expect("runtime block marker");
        let block = &source[start..];
        let mut depth = 0usize;
        let mut entered = false;
        let mut body = String::new();
        for character in block.chars() {
            if character == '{' {
                depth += 1;
                entered = true;
            }
            if entered {
                body.push(character);
            }
            if character == '}' {
                depth -= 1;
                if entered && depth == 0 {
                    break;
                }
            }
        }
        body.lines()
            .filter_map(|line| {
                line.trim_start()
                    .strip_prefix("fn ")
                    .and_then(|rest| rest.split_once('('))
                    .map(|(name, _)| name.trim().to_owned())
            })
            .collect()
    }

    #[test]
    fn production_sprite_bank_runtimes_override_every_trait_method() {
        let production = include_str!("native_gameplay_services.rs");
        for (trait_source, trait_marker, impl_marker) in [
            (
                include_str!("sprite.rs"),
                "pub trait SpriteHooks",
                "impl SpriteHooks for NativeGameServices",
            ),
            (
                include_str!("sprite_main_bank0.rs"),
                "pub trait SpriteMainBank0Runtime",
                "impl SpriteMainBank0Runtime for NativeGameServices",
            ),
            (
                include_str!("sprite_main_bank1.rs"),
                "pub trait SpriteMainBank1Runtime",
                "impl crate::sprite_main_bank1::SpriteMainBank1Runtime for NativeGameServices",
            ),
            (
                include_str!("sprite_main_bank2a.rs"),
                "pub trait SpriteMainBank2aRuntime",
                "impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for NativeGameServices",
            ),
            (
                include_str!("sprite_main_bank2b.rs"),
                "pub trait SpriteMainBank2bRuntime",
                "impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for NativeGameServices",
            ),
            (
                include_str!("ancilla.rs"),
                "pub trait AncillaRuntime",
                "impl AncillaRuntime for NativeGameServices",
            ),
            (
                include_str!("player.rs"),
                "pub trait PlayerRuntime",
                "impl PlayerRuntime for NativeGameServices",
            ),
            (
                include_str!("tile_detect.rs"),
                "pub trait TileDetectHooks",
                "impl crate::tile_detect::TileDetectHooks for NativeGameServices",
            ),
            (
                include_str!("tagalong.rs"),
                "pub trait TagalongHooks",
                "impl crate::tagalong::TagalongHooks for NativeGameServices",
            ),
            (
                include_str!("overlord.rs"),
                "pub trait OverlordHooks",
                "impl crate::overlord::OverlordHooks for NativeGameServices",
            ),
        ] {
            let required = method_names(trait_source, trait_marker);
            let implemented = method_names(production, impl_marker);
            let missing = required.difference(&implemented).collect::<Vec<_>>();
            assert!(
                missing.is_empty(),
                "inherited production defaults: {missing:?}"
            );
        }
    }

    #[test]
    fn production_graphics_hooks_are_concrete_in_their_resource_owners() {
        let trait_source = include_str!("load_gfx.rs");
        let production = include_str!("native_services.rs");
        let mut gameplay_required = method_names(trait_source, "pub trait LoadGfxHooks");
        // Save-slot equipment lives in SRAM, which NativeGameServices does not
        // own. FileSelectGfxHooks is the concrete resource-aware owner.
        gameplay_required.remove("file_select_gear");
        let gameplay = method_names(production, "impl LoadGfxHooks for NativeGameServices");
        assert_eq!(gameplay, gameplay_required);

        let file_select = method_names(production, "impl LoadGfxHooks for FileSelectGfxHooks");
        assert_eq!(file_select, BTreeSet::from(["file_select_gear".to_owned()]));
    }

    #[test]
    fn bank1_ganon_idle_matches_the_c_state_writes() {
        with_native_services(|services| {
            let mut bytes = [0; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            ram.write_u8(0xde0 + 3, 1);
            crate::sprite_main_bank1::SpriteMainBank1Runtime::ganon_animation_idle(
                services, &mut ram, 3,
            );
            assert_eq!(ram.read_u8(0xed0 + 3), 10);
            assert_eq!(ram.read_u8(0xdc0 + 3), 10);
        });
    }

    #[test]
    fn bank1_sized_oam_emit_matches_c_buffers_and_pointer_advance() {
        with_native_services(|services| {
            let mut bytes = [0; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            ram.write_u16(0x90, crate::oam::OAM_BUFFER as u16);
            ram.write_u16(0x92, crate::oam::BYTEWISE_EXTENDED_OAM as u16);
            crate::sprite_main_bank1::SpriteMainBank1Runtime::emit_oam_sized(
                services,
                &mut ram,
                0,
                (0x81, 0x72, 0x53, 0x34, 2),
            );
            assert_eq!(crate::oam::read(&ram, 0).x, 0x81);
            assert_eq!(crate::oam::read(&ram, 0).y, 0x72);
            assert_eq!(crate::oam::read(&ram, 0).charnum, 0x53);
            assert_eq!(crate::oam::read(&ram, 0).flags, 0x34);
            assert_eq!(ram.read_u8(crate::oam::BYTEWISE_EXTENDED_OAM), 2);
            assert_eq!(ram.read_u16(0x90), crate::oam::OAM_BUFFER as u16 + 4);
            assert_eq!(
                ram.read_u16(0x92),
                crate::oam::BYTEWISE_EXTENDED_OAM as u16 + 1
            );
        });
    }

    #[test]
    fn bank1_oam_retile_preserves_the_c_high_flag_nibble() {
        with_native_services(|services| {
            let mut bytes = [0; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            crate::oam::set_plain(&mut ram, 7, 1, 2, 3, 0xb6, 0);
            crate::sprite_main_bank1::SpriteMainBank1Runtime::set_oam_piece_character_and_low_flags(
                services, &mut ram, 7, 0x9a, 5,
            );
            let entry = crate::oam::read(&ram, 7);
            assert_eq!(entry.charnum, 0x9a);
            assert_eq!(entry.flags, 0xb5);
        });
    }

    #[test]
    fn sprite_hooks_random_route_updates_the_exact_c_rng_byte() {
        with_native_services(|services| {
            let mut bytes = [0; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            ram.write_u8(0xfa1, 0xfe);
            ram.write_u8(0x1a, 3);
            assert_eq!(SpriteHooks::random_u8(services, &mut ram), 0);
            assert_eq!(ram.read_u8(0xfa1), 0);
            ram.write_u8(0x1a, 0);
            assert_eq!(SpriteHooks::random_u8(services, &mut ram), 0xb8);
            assert_eq!(ram.read_u8(0xfa1), 0xb8);
        });
    }

    #[test]
    fn sprite_hooks_ancilla_main_runs_weapon_tink_before_ancilla_slots() {
        with_native_services(|services| {
            let mut bytes = [0; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            // Region A starts at byte 0x30 after Oam_ResetRegionBases.  Weapon
            // tink reserves four OAM entries even when no ancilla slots exist.
            ram.write_u16(0xfe0, 0x30);
            ram.write_u8(0xfac, 3); // repulsespark_timer
            ram.write_u8(0xfaf, 1); // repulsespark_anim_delay
            ram.write_u8(0xfad, 0x40); // repulsespark_x_lo
            ram.write_u8(0xfae, 0x50); // repulsespark_y_lo

            SpriteHooks::ancilla_main(services, &mut ram);

            assert_eq!(ram.read_u16(0xfe0), 0x40);
            assert_eq!(ram.read_u16(0x90), crate::oam::OAM_BUFFER as u16 + 0x30);
            assert_eq!(crate::oam::read(&ram, 12).charnum, 0x92);
        });
    }
}
