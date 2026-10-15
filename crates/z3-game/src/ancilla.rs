//! Ancilla (projectile/effect) runtime translated from `src/ancilla.c`.
//!
//! Named live effect views retain the original structure-of-arrays WRAM layout.
//! World and height movement keep their respective 24-bit and 16-bit wrapping
//! fixed-point behavior.

use crate::{oam, position::SplitPosition, ram::Ram};

#[path = "ancilla_handlers_0.rs"]
mod handlers_0;

pub const ANCILLA_COUNT: usize = 10;
#[cfg(test)]
const Z_VEL: usize = 0x294;
const Z: usize = 0x29e;
const Z_SUBPIXEL: usize = 0x2a8;
const Y_LO: usize = 0xbfa;
const X_LO: usize = 0xc04;
const Y_HI: usize = 0xc0e;
const X_HI: usize = 0xc18;
#[cfg(test)]
const Y_VEL: usize = 0xc22;
#[cfg(test)]
const X_VEL: usize = 0xc2c;
const Y_SUBPIXEL: usize = 0xc36;
const X_SUBPIXEL: usize = 0xc40;
const X_POSITION: SplitPosition<16> = SplitPosition::world(X_SUBPIXEL, X_LO, X_HI);
const Y_POSITION: SplitPosition<16> = SplitPosition::world(Y_SUBPIXEL, Y_LO, Y_HI);
const HEIGHT: SplitPosition<8> = SplitPosition::height(Z_SUBPIXEL, Z);
#[cfg(test)]
const TYPE: usize = 0xc4a;
const FEATURES0: usize = 0x64c;
const FEATURE_EXTEND_SCREEN_64: u32 = 1;

pub const PFLAGS: [u8; 68] = [
    0, 8, 0xc, 0x10, 0x10, 4, 0x10, 0x18, 8, 8, 8, 0, 0x14, 0, 0x10, 0x28, 0x18, 0x10, 0x10, 0x10,
    0x10, 0xc, 8, 8, 0x50, 0, 0x10, 8, 0x40, 0, 0xc, 0x24, 0x10, 0xc, 8, 0x10, 0x10, 4, 0xc, 0x1c,
    0, 0x10, 0x14, 0x14, 0x10, 8, 0x20, 0x10, 0x10, 0x10, 4, 0, 0x80, 0x10, 4, 0x30, 0x14, 0x10, 0,
    0x10, 0, 0, 8, 0, 0x10, 8, 0x78, 0x80,
];
pub const BOMB_FRAME_DELAYS: [u8; 11] = [0xa0, 6, 4, 4, 4, 4, 4, 6, 6, 6, 6];
pub const FIRE_ROD_X_VEL: [i8; 12] = [0, 0, -40, 40, 0, 0, -48, 48, 0, 0, -64, 64];
pub const FIRE_ROD_Y_VEL: [i8; 12] = [-40, 40, 0, 0, -48, 48, 0, 0, -64, 64, 0, 0];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AncillaOamInfo {
    pub x: u8,
    pub y: u8,
    pub flags: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AncillaRadialProjection {
    pub r0: u8,
    pub r2: u8,
    pub r4: u8,
    pub r6: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProjectSpeed {
    pub x: u8,
    pub y: u8,
    pub xdiff: u8,
    pub ydiff: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AncillaHitBox {
    pub x: u16,
    pub y: u16,
    pub width: u8,
    pub height: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CheckPlayerCollOut {
    pub r4: u16,
    pub r6: u16,
    pub r8: u16,
    pub r10: u16,
}

pub trait AncillaRuntime {
    /// OAM-region allocation is owned by the OAM subsystem.
    fn allocate_oam(&mut self, _ram: &mut Ram<'_>, _k: usize, _size: u8) -> u8 {
        0
    }
    fn hud_refresh_icon(&mut self, _ram: &mut Ram<'_>) {}
    fn load_gear_palettes_bunny(&mut self, _ram: &mut Ram<'_>) {}
    fn terminate_sparkle_objects(&mut self, ram: &mut Ram<'_>) {
        for slot in 0..5 {
            if matches!(
                ram.ancilla(slot).kind(),
                0x2a | 0x2b | 0x30 | 0x31 | 0x18 | 0x19 | 0x0c
            ) {
                ram.ancilla_mut(slot).set_kind(0);
            }
        }
    }
    fn palette_load_sprite_environment_dungeon(&mut self, _ram: &mut Ram<'_>) {}
    fn load_actual_gear_palettes(&mut self, _ram: &mut Ram<'_>) {}
    fn link_receive_item(&mut self, ram: &mut Ram<'_>, item: u8) {
        self.receive_item(ram, item);
    }
    fn follower_initialize(&mut self, _ram: &mut Ram<'_>) {}
    fn link_reset_properties_a(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_load_link_armor_and_gloves(&mut self, _ram: &mut Ram<'_>) {}
    fn prepare_dungeon_exit_from_boss_fight(&mut self, _ram: &mut Ram<'_>) {}
    fn main_show_text_message(&mut self, ram: &mut Ram<'_>) {
        crate::misc::main_show_text_message(ram);
    }
    fn write_to_4bpp_buffer(&mut self, _ram: &mut Ram<'_>, _tile: u8) {}
    fn decode_animated_sprite_tile_variable(&mut self, ram: &mut Ram<'_>, tile: u8) {
        self.decode_animated_sprite_tile(ram, tile);
    }
    fn player_tile_detect_nearby(&mut self, _ram: &mut Ram<'_>) {}
    fn overworld_alter_weathervane(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_restore_bg_and_hud(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_electro_themed_gear(&mut self, _ram: &mut Ram<'_>) {}
    fn filter_majorly_whiten_bg(&mut self, _ram: &mut Ram<'_>) {}
    fn palette_restore_bg_from_flash(&mut self, _ram: &mut Ram<'_>) {}

    /// Sprite velocity projection lives in `sprite.c`.
    fn sprite_project_speed_towards_link(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _velocity: u8,
    ) -> ProjectSpeed {
        ProjectSpeed::default()
    }
    fn sprite_project_speed_towards_location(
        &mut self,
        _ram: &mut Ram<'_>,
        _k: usize,
        _x: u16,
        _y: u16,
        _velocity: u8,
    ) -> ProjectSpeed {
        ProjectSpeed::default()
    }

    /// Completes a sprite hit box and compares it with the supplied ancilla box.
    fn sprite_hitbox_overlaps(
        &mut self,
        _ram: &mut Ram<'_>,
        _sprite: usize,
        _ancilla: AncillaHitBox,
    ) -> bool {
        false
    }

    fn sprite_create_deflected_arrow(&mut self, _ram: &mut Ram<'_>, _ancilla: usize) {}
    fn check_damage_to_sprite_preset(&mut self, _ram: &mut Ram<'_>, _sprite: usize, _damage: u8) {}
    fn refund_magic(&mut self, _ram: &mut Ram<'_>, _amount: u8) {}
    fn dungeon_light_torch(&mut self, _ram: &mut Ram<'_>) {}
    fn check_ability_to_swim(&mut self, _ram: &mut Ram<'_>) {}
    fn bomb_check_for_destructibles(
        &mut self,
        _ram: &mut Ram<'_>,
        _x: u16,
        _y: u16,
        _ancilla: usize,
    ) {
    }
    fn generated_bombos_position(&mut self, _frame: u8) -> u8 {
        0
    }
    /// Generated wish-pond item metadata lives in the asset bundle.
    fn generated_wish_pond_item(&mut self, _item: u8) -> u8 {
        0
    }
    fn sprite_spawn_poof_garnish(&mut self, _ram: &mut Ram<'_>, _sprite: usize) {}
    fn hookshot_check_tile_collision(&mut self, _ram: &mut Ram<'_>, _ancilla: usize) {}
    fn overworld_offset_base(&mut self, _area: u8) -> (u16, u16) {
        (0, 0)
    }
    fn prepare_apply_rumble_to_sprites(&mut self, _ram: &mut Ram<'_>) {}
    fn receive_item(&mut self, _ram: &mut Ram<'_>, _item: u8) {}
    fn audio_port0(&mut self) -> u8 {
        0
    }
    fn spawn_sprite_dynamically(
        &mut self,
        _ram: &mut Ram<'_>,
        _source: usize,
        _kind: u8,
    ) -> Option<usize> {
        None
    }
    fn decode_animated_sprite_tile(&mut self, _ram: &mut Ram<'_>, _graphics: u8) {}
    fn decompress_shield_graphics(&mut self, _ram: &mut Ram<'_>) {}
    fn decompress_sword_graphics(&mut self, _ram: &mut Ram<'_>) {}
    fn overworld_do_map_update_32x32_b(&mut self, _ram: &mut Ram<'_>) {}

    /// Attribute lookup and slope resolution are owned by the overworld/dungeon
    /// and tile-detection translation units. The ancilla collision policy stays
    /// local to this module.
    fn overworld_tile_attribute(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) -> u8 {
        0
    }
    fn dungeon_tile_attribute(
        &mut self,
        _ram: &mut Ram<'_>,
        _floor: u8,
        x: u16,
        _y: u16,
    ) -> (u8, u16) {
        (0, x)
    }
    fn sloped_tile_collision(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) -> bool {
        false
    }

    /// Region allocation is implemented by the sprite/OAM subsystem. The
    /// callback must update the canonical `oam_cur_ptr` just like the C helper.
    fn allocate_oam_region(&mut self, _ram: &mut Ram<'_>, _region: u8, _size: u8) {}
}

pub struct Ancilla<'a, 'r, R: AncillaRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
}

impl<R: AncillaRuntime> Ancilla<'_, '_, R> {
    /// One-for-one entry point for `Ancilla_GetX`.
    #[must_use]
    pub fn ancilla_get_x(&self, k: usize) -> u16 {
        self.get_x(k)
    }

    /// One-for-one entry point for `Ancilla_GetY`.
    #[must_use]
    pub fn ancilla_get_y(&self, k: usize) -> u16 {
        self.get_y(k)
    }

    /// One-for-one entry point for `Ancilla_SetX`.
    pub fn ancilla_set_x(&mut self, k: usize, value: u16) {
        self.set_x(k, value);
    }

    /// One-for-one entry point for `Ancilla_SetY`.
    pub fn ancilla_set_y(&mut self, k: usize, value: u16) {
        self.set_y(k, value);
    }

    #[must_use]
    pub fn get_x(&self, k: usize) -> u16 {
        X_POSITION.pixels(self.ram, k)
    }

    #[must_use]
    pub fn get_y(&self, k: usize) -> u16 {
        Y_POSITION.pixels(self.ram, k)
    }

    pub fn set_x(&mut self, k: usize, value: u16) {
        X_POSITION.set_pixels(self.ram, k, value);
    }

    pub fn set_y(&mut self, k: usize, value: u16) {
        Y_POSITION.set_pixels(self.ram, k, value);
    }

    #[must_use]
    pub fn alloc_high(&self) -> Option<usize> {
        (0..ANCILLA_COUNT).rev().find(|&k| self.kind(k) == 0)
    }

    pub fn move_x(&mut self, k: usize) {
        let position = X_POSITION.load(self.ram, k);
        let velocity = self.x_velocity(k) as i8;
        X_POSITION.store(self.ram, k, position.advanced(velocity));
    }

    pub fn move_y(&mut self, k: usize) {
        let position = Y_POSITION.load(self.ram, k);
        let velocity = self.y_velocity(k) as i8;
        Y_POSITION.store(self.ram, k, position.advanced(velocity));
    }

    pub fn move_z(&mut self, k: usize) {
        let position = HEIGHT.load(self.ram, k);
        let velocity = self.z_velocity(k) as i8;
        HEIGHT.store(self.ram, k, position.advanced(velocity));
    }

    pub fn set_oam(&mut self, index: usize, x: u16, y: u16, charnum: u8, flags: u8, mut big: u8) {
        let mut visible_y = 0xf0;
        let feature_bytes = self.ram.slice(FEATURES0, 4);
        let features = u32::from_le_bytes(feature_bytes.try_into().expect("four feature bytes"));
        let xt = if features & FEATURE_EXTEND_SCREEN_64 != 0 {
            0x40
        } else {
            0
        };
        let visible_x = x.wrapping_add(xt) < 0x100 + xt * 2;
        let mut entry = oam::read(self.ram, index);
        if visible_x && y < 256 {
            big |= (x >> 8) as u8 & 1;
            entry.x = x as u8;
            if y < 0xf0 {
                visible_y = y as u8
            }
        }
        entry.y = visible_y;
        entry.charnum = charnum;
        entry.flags = flags;
        oam::write(self.ram, index, entry);
        self.ram.write_u8(oam::BYTEWISE_EXTENDED_OAM + index, big);
    }

    pub fn set_oam_safe(
        &mut self,
        index: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        mut big: u8,
    ) {
        let mut visible_y = 0xf0;
        let feature_bytes = self.ram.slice(FEATURES0, 4);
        let features = u32::from_le_bytes(feature_bytes.try_into().expect("four feature bytes"));
        let xt = if features & FEATURE_EXTEND_SCREEN_64 != 0 {
            0x48
        } else {
            0
        };
        if x.wrapping_add(0x80) < 0x180 + xt {
            big |= (x >> 8) as u8 & 1;
            if y.wrapping_add(0x10) < 0x100 {
                visible_y = y as u8
            }
        }
        oam::set_plain(self.ram, index, x as u8, visible_y, charnum, flags, big);
    }

    pub fn execute_all(&mut self) {
        for k in (0..ANCILLA_COUNT).rev() {
            self.ram.write_u8(0xfa0, k as u8);
            let kind = self.kind(k);
            if kind != 0 {
                self.execute_one(kind, k)
            }
        }
    }

    /// One-for-one entry point for `Ancilla_Main`.
    pub fn ancilla_main(&mut self) {
        self.main();
    }

    /// The original entries deliberately assert because types $14 and $25 are
    /// unused. Keep that behavior visible instead of silently treating them as
    /// empty handlers.
    pub fn ancilla_unused_14(&mut self, _k: usize) {
        panic!("unused ancilla type 0x14");
    }

    pub fn ancilla_unused_25(&mut self, _k: usize) {
        panic!("unused ancilla type 0x25");
    }

    pub fn execute_one(&mut self, kind: u8, k: usize) {
        assert!((1..=67).contains(&kind));
        if k < 6 {
            let size = self.sprite_count(k);
            let index = self.runtime.allocate_oam(self.ram, k, size);
            self.set_oam_index(k, index);
        }
        if self.ram.read_u8(0x11) == 0 {
            let timer = self.timer(k);
            if timer != 0 {
                self.set_timer(k, timer - 1)
            }
        }
        self.dispatch(kind, k);
    }

    fn dispatch(&mut self, kind: u8, k: usize) {
        if handlers_0::dispatch(self, kind, k) {
            return;
        }
        match kind {
            20 | 37 => panic!("unused ancilla type {kind:#x}"),
            // Never disguise local ancilla logic as an external dependency.
            // Each remaining arm is replaced with its local port in source order.
            _ => panic!("unported local ancilla handler {kind:#x} in slot {k}"),
        }
    }

    #[must_use]
    pub(crate) fn current_oam_index(&self) -> usize {
        oam::OamPointers::current_cursor(self.ram).index()
    }

    pub fn empty(&mut self, _k: usize) {}

    pub(crate) fn read(&self, address: usize) -> u8 {
        self.ram.read_u8(address)
    }
    pub(crate) fn write(&mut self, address: usize, value: u8) {
        self.ram.write_u8(address, value)
    }
    #[cfg(test)]
    pub(crate) fn type_address(k: usize) -> usize {
        TYPE + k
    }
    pub fn clear(&mut self, k: usize) {
        self.set_kind(k, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    struct Runtime;
    impl AncillaRuntime for Runtime {}

    fn fixture(f: impl FnOnce(&mut Ancilla<'_, '_, Runtime>)) {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = Runtime;
        f(&mut Ancilla {
            ram: &mut ram,
            runtime: &mut runtime,
        });
    }

    #[test]
    fn coordinates_use_parallel_low_and_high_arrays() {
        fixture(|a| {
            a.set_x(4, 0xabcd);
            a.set_y(4, 0x1234);
            assert_eq!(a.get_x(4), 0xabcd);
            assert_eq!(a.get_y(4), 0x1234);
            assert_eq!(a.ram.read_u8(X_LO + 4), 0xcd);
            assert_eq!(a.ram.read_u8(X_HI + 4), 0xab);
        });
    }

    #[test]
    fn movement_matches_signed_24_bit_fixed_point_math() {
        fixture(|a| {
            a.set_x(0, 0x1000);
            a.ram.write_u8(X_SUBPIXEL, 8);
            a.ram.write_i8(X_VEL, -1);
            a.move_x(0);
            assert_eq!(a.get_x(0), 0x0fff);
            assert_eq!(a.ram.read_u8(X_SUBPIXEL), 0xf8);
            a.ram.write_u8(Z, 0);
            a.ram.write_u8(Z_SUBPIXEL, 0);
            a.ram.write_i8(Z_VEL, -1);
            a.move_z(0);
            assert_eq!(a.ram.read_u8(Z), 0xff);
            assert_eq!(a.ram.read_u8(Z_SUBPIXEL), 0xf0);
        });
    }

    #[test]
    fn coordinate_setters_preserve_fractions_and_other_slots() {
        fixture(|a| {
            a.ram.bytes_mut().fill(0xa5);
            let mut expected = a.ram.slice(0, WRAM_SIZE).to_vec();
            a.set_x(4, 0xabcd);
            a.set_y(4, 0x1234);
            expected[X_LO + 4] = 0xcd;
            expected[X_HI + 4] = 0xab;
            expected[Y_LO + 4] = 0x34;
            expected[Y_HI + 4] = 0x12;
            assert_eq!(a.ram.slice(0, WRAM_SIZE), expected);
        });
    }

    #[test]
    fn movement_touches_only_the_selected_split_coordinate() {
        fixture(|a| {
            for axis in 0..3 {
                let (fraction, low, high, velocity) = match axis {
                    0 => (X_SUBPIXEL, X_LO, Some(X_HI), X_VEL),
                    1 => (Y_SUBPIXEL, Y_LO, Some(Y_HI), Y_VEL),
                    _ => (Z_SUBPIXEL, Z, None, Z_VEL),
                };
                for speed in [-128i8, -1, 0, 1, 127] {
                    for pixels in [0u16, 0xff, 0xffff] {
                        a.ram.bytes_mut().fill(0xa5);
                        let slot = 4;
                        a.ram.write_u8(fraction + slot, 0xf8);
                        a.ram.write_u8(low + slot, pixels as u8);
                        if let Some(high) = high {
                            a.ram.write_u8(high + slot, (pixels >> 8) as u8);
                        }
                        a.ram.write_i8(velocity + slot, speed);
                        let mut expected = a.ram.slice(0, WRAM_SIZE).to_vec();
                        let integer = if high.is_some() {
                            pixels
                        } else {
                            pixels & 0xff
                        };
                        let c_sum = (u32::from(integer) << 8 | 0xf8)
                            .wrapping_add((i32::from(speed) << 4) as u32);
                        expected[fraction + slot] = c_sum as u8;
                        expected[low + slot] = (c_sum >> 8) as u8;
                        if let Some(high) = high {
                            expected[high + slot] = (c_sum >> 16) as u8;
                        }
                        match axis {
                            0 => a.move_x(slot),
                            1 => a.move_y(slot),
                            _ => a.move_z(slot),
                        }
                        assert_eq!(a.ram.slice(0, WRAM_SIZE), expected);
                    }
                }
            }
        });
    }

    #[test]
    fn allocation_scans_from_high_slot() {
        fixture(|a| {
            for k in 0..ANCILLA_COUNT {
                a.ram.write_u8(TYPE + k, 1);
            }
            a.ram.write_u8(TYPE + 3, 0);
            a.ram.write_u8(TYPE + 8, 0);
            assert_eq!(a.alloc_high(), Some(8));
        });
    }

    #[test]
    fn execute_all_updates_cur_object_index_even_for_empty_slots() {
        fixture(|a| {
            a.ram.write_u8(0xfa0, 0xff);
            a.execute_all();
            assert_eq!(a.ram.read_u8(0xfa0), 0);
        });
    }

    #[test]
    fn terminate_sparkles_matches_the_c_slot_and_type_filter() {
        fixture(|a| {
            for (slot, kind) in [0x2a, 0x2b, 0x30, 0x31, 0x18, 0x19, 0x0c, 0x43]
                .into_iter()
                .enumerate()
            {
                a.ram.write_u8(TYPE + slot, kind);
            }
            a.runtime.terminate_sparkle_objects(a.ram);
            assert_eq!(a.ram.slice(TYPE, 8), &[0, 0, 0, 0, 0, 0x19, 0x0c, 0x43]);
        });
    }

    #[test]
    fn show_text_message_uses_the_translated_misc_state_transition() {
        fixture(|a| {
            a.ram.write_u8(0x10, 9);
            a.ram.write_u8(0x11, 0xaa);
            a.ram.write_u8(0x1cd8, 0xbb);
            a.runtime.main_show_text_message(a.ram);
            assert_eq!(a.ram.read_u8(0x10), 14);
            assert_eq!(a.ram.read_u8(0x11), 2);
            assert_eq!(a.ram.read_u8(0x010c), 9);
            assert_eq!(a.ram.read_u8(0x1cd8), 0);
        });
    }
}
