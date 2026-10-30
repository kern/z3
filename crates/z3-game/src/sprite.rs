//! Sprite, garnish, collision, spawning, and sprite-loading logic.
//!
//! This module is a bounds-checked Rust port of `src/sprite.c`. The original
//! WRAM layout is kept explicit so behavior can be compared address-for-address.

use crate::sound::{SoundPort, SoundRequest};
#[cfg(test)]
use crate::sprite_state::layout::*;
use crate::sprite_state::{SpriteField, SpriteInitialProperties, SpriteSlot};
use crate::velocity::VelocityByte;
use crate::{oam, ram::Ram};
use z3_core::WRAM_SIZE;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrepOamCoordsRet {
    pub x: u16,
    pub y: u16,
    pub r4: u8,
    pub flags: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpriteHitBox {
    pub r0_xlo: u8,
    pub r8_xhi: u8,
    pub r1_ylo: u8,
    pub r9_yhi: u8,
    pub r2: u8,
    pub r3: u8,
    pub r4_spr_xlo: u8,
    pub r10_spr_xhi: u8,
    pub r5_spr_ylo: u8,
    pub r11_spr_yhi: u8,
    pub r6_spr_xsize: u8,
    pub r7_spr_ysize: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpriteSpawnInfo {
    pub r0_x: u16,
    pub r2_y: u16,
    pub r4_z: u8,
    pub r5_overlord_x: u16,
    pub r7_overlord_y: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawMultipleData {
    pub x: i8,
    pub y: i8,
    pub char_flags: u16,
    pub ext: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectSpeedRet {
    pub x: u8,
    pub y: u8,
    pub xdiff: u8,
    pub ydiff: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PairU8 {
    pub a: u8,
    pub b: u8,
}

pub trait SpriteHooks:
    crate::sprite_main_bank0::SpriteMainBank0Runtime
    + crate::sprite_main_bank1::SpriteMainBank1Runtime
    + crate::sprite_main_bank2a::SpriteMainBank2aRuntime
    + crate::sprite_main_bank2b::SpriteMainBank2bRuntime
{
    fn garnish_alloc_limit(&mut self, _ram: &mut Ram<'_>, _limit: usize) -> Option<usize> {
        None
    }
    fn random_u8(&mut self, _ram: &mut Ram<'_>) -> u8 {
        0
    }
    fn ancilla_spawn_falling_prize(&mut self, _ram: &mut Ram<'_>, _item: u8) -> i32 {
        -1
    }
    fn tile_attribute(&mut self, _ram: &mut Ram<'_>, _floor: u8, _x: &mut u16, _y: u16) -> u8 {
        0
    }
    fn handle_indoor_camera_and_doors(&mut self, _ram: &mut Ram<'_>) {}
    fn link_cancel_dash(&mut self, _ram: &mut Ram<'_>) {}
    fn enhanced_features0(&self) -> u32 {
        0
    }
    fn sprite_prep_small_key(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn throwable_scenery_flags(&self, _kind: u8) -> u8 {
        0
    }
    fn link_calculate_sfx_pan(&self, _ram: &Ram<'_>) -> u8 {
        0
    }
    fn sprite_sfx2_with_pan(&mut self, _ram: &mut Ram<'_>, _slot: usize, _sfx: u8) {}
    fn sprite_sfx3_with_pan(&mut self, _ram: &mut Ram<'_>, _slot: usize, _sfx: u8) {}
    fn link_receive_item(&mut self, _ram: &mut Ram<'_>, _item: u8, _method: u8) {}
    fn palette_load_shield(&mut self, _ram: &mut Ram<'_>) {}
    fn sprite_transmute_to_bomb(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn sprite_draw_water_ripple(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn dungeon_update_tilemap_with_common_tile(
        &mut self,
        _ram: &mut Ram<'_>,
        _x: u16,
        _y: u16,
        _tile: u8,
    ) {
    }
    fn overworld_sprite_data(&self, _ram: &Ram<'_>, _area: u16) -> &[u8] {
        &[]
    }
    fn dungeon_sprite_data(&self, _room: u16) -> &[u8] {
        &[]
    }
    fn sprite_prep_big_key_load_graphics(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn sprite_prep_key_set_item_drop(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn throwable_scenery_scatter_into_debris(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn flame_draw(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn prepare_dungeon_exit_from_boss_fight(&mut self, _ram: &mut Ram<'_>) {}
    fn sprite_spawn_poof_garnish(&mut self, _ram: &mut Ram<'_>, _slot: usize) {}
    fn handle_screen_flash(&mut self, _ram: &mut Ram<'_>) {}
    fn follower_main(&mut self, _ram: &mut Ram<'_>) {}
    fn ancilla_main(&mut self, _ram: &mut Ram<'_>) {}
    fn overlord_main(&mut self, _ram: &mut Ram<'_>) {}
}

pub struct Sprite<'a, 'h, H: SpriteHooks> {
    pub ram: Ram<'a>,
    pub hooks: &'h mut H,
}

impl<'a, 'h, H: SpriteHooks> Sprite<'a, 'h, H> {
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
    fn arr(&self, base: usize, slot: usize) -> u8 {
        self.r8(base + slot)
    }
    fn set_arr(&mut self, base: usize, slot: usize, value: u8) {
        self.w8(base + slot, value);
    }
    fn run_sprite_active_main(&mut self, slot: usize) {
        let kind = self.ram.sprite(slot).kind();
        crate::sprite_main_bank0::SpriteMainBank0 {
            ram: &mut self.ram,
            runtime: self.hooks,
        }
        .dispatch_active(slot, kind);
    }

    fn run_sprite_module_initialize(&mut self, slot: usize) {
        crate::sprite_main_bank1::SpriteMainBank1Runtime::load_properties(
            self.hooks,
            &mut self.ram,
            slot,
        );
        self.ram
            .set_sprite_state(slot, self.ram.sprite(slot).state().wrapping_add(1));
        let kind = self.ram.sprite(slot).kind();
        crate::sprite_main_bank0::SpriteMainBank0 {
            ram: &mut self.ram,
            runtime: self.hooks,
        }
        .dispatch_prep(slot, kind);
    }
    fn current_oam_index(&self) -> usize {
        oam::OamPointers::current_cursor(&self.ram).index()
    }
    fn set_oam_plain(&mut self, index: usize, x: u16, y: u16, charnum: u8, flags: u8, big: u8) {
        oam::set_plain(&mut self.ram, index, x as u8, y as u8, charnum, flags, big);
    }

    #[must_use]
    pub fn sprite_get_x(&self, slot: usize) -> u16 {
        self.ram.sprite(slot).x()
    }
    #[must_use]
    pub fn sprite_get_y(&self, slot: usize) -> u16 {
        self.ram.sprite(slot).y()
    }
    pub fn sprite_set_x(&mut self, slot: usize, value: u16) {
        self.ram.set_sprite_x(slot, value);
    }
    pub fn sprite_set_y(&mut self, slot: usize, value: u16) {
        self.ram.set_sprite_y(slot, value);
    }
    pub fn sprite_approach_target_speed(&mut self, slot: usize, x: u8, y: u8) {
        self.ram.sprite_mut(slot).approach_velocity(x, y);
    }
    pub fn sprite_add_xy(&mut self, slot: usize, x: i16, y: i16) {
        self.sprite_set_x(slot, self.sprite_get_x(slot).wrapping_add_signed(x));
        self.sprite_set_y(slot, self.sprite_get_y(slot).wrapping_add_signed(y));
    }
    pub fn sprite_invert_xy_speeds(&mut self, slot: usize) {
        self.ram.set_sprite_velocity_x(
            slot,
            VelocityByte::from_bits(self.ram.sprite(slot).velocity_x())
                .negated()
                .bits(),
        );
        self.ram.set_sprite_velocity_y(
            slot,
            VelocityByte::from_bits(self.ram.sprite(slot).velocity_y())
                .negated()
                .bits(),
        );
    }
    pub fn sprite_invert_speed_xy(&mut self, slot: usize) {
        self.sprite_invert_xy_speeds(slot);
    }
    pub fn sprite_move_x(&mut self, slot: usize) {
        self.ram.move_sprite_x(slot);
    }
    pub fn sprite_move_y(&mut self, slot: usize) {
        self.ram.move_sprite_y(slot);
    }
    pub fn sprite_move_z(&mut self, slot: usize) {
        self.ram.move_sprite_z(slot);
    }
    pub fn sprite_move_xy(&mut self, slot: usize) {
        self.sprite_move_x(slot);
        self.sprite_move_y(slot);
    }
    pub fn sprite_move_xyz(&mut self, slot: usize) {
        self.sprite_move_z(slot);
        self.sprite_move_x(slot);
        self.sprite_move_y(slot);
    }
    pub fn sprite_zero_velocity_xy(&mut self, slot: usize) {
        self.ram.set_sprite_velocity_x(slot, 0);
        self.ram.set_sprite_velocity_y(slot, 0);
    }
    pub fn sprite_halve_speed_xy(&mut self, slot: usize) {
        self.ram.set_sprite_velocity_x(
            slot,
            VelocityByte::from_bits(self.ram.sprite(slot).velocity_x())
                .arithmetic_half()
                .bits(),
        );
        self.ram.set_sprite_velocity_y(
            slot,
            VelocityByte::from_bits(self.ram.sprite(slot).velocity_y())
                .arithmetic_half()
                .bits(),
        );
    }
    pub fn sprite_bounce_off_wall(&mut self, slot: usize) {
        let collision = self.ram.sprite(slot).wall_collision();
        if collision & 3 != 0 {
            self.ram.set_sprite_velocity_x(
                slot,
                VelocityByte::from_bits(self.ram.sprite(slot).velocity_x())
                    .negated()
                    .bits(),
            );
        }
        if collision & 12 != 0 {
            self.ram.set_sprite_velocity_y(
                slot,
                VelocityByte::from_bits(self.ram.sprite(slot).velocity_y())
                    .negated()
                    .bits(),
            );
        }
    }
    fn project_speed(right: PairU8, below: PairU8, mut velocity: u8) -> ProjectSpeedRet {
        if velocity == 0 {
            return ProjectSpeedRet::default();
        }
        let mut minor = (below.b as i8).unsigned_abs();
        let mut major = (right.b as i8).unsigned_abs();
        let mut swapped = false;
        if major < minor {
            core::mem::swap(&mut minor, &mut major);
            swapped = true;
        }
        let mut xvel = velocity;
        let mut yvel = 0u8;
        let mut accumulator = 0u8;
        loop {
            accumulator = accumulator.wrapping_add(minor);
            if accumulator >= major {
                accumulator = accumulator.wrapping_sub(major);
                yvel = yvel.wrapping_add(1);
            }
            velocity = velocity.wrapping_sub(1);
            if velocity == 0 {
                break;
            }
        }
        if swapped {
            core::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right.a != 0 {
                VelocityByte::from_bits(xvel).negated().bits()
            } else {
                xvel
            },
            y: if below.a != 0 {
                VelocityByte::from_bits(yvel).negated().bits()
            } else {
                yvel
            },
            xdiff: right.b,
            ydiff: below.b,
        }
    }
    pub fn sprite_is_right_of_location(&self, slot: usize, x: u16) -> PairU8 {
        let delta = x.wrapping_sub(self.sprite_get_x(slot));
        PairU8 {
            a: u8::from((delta as i16) < 0),
            b: delta as u8,
        }
    }
    pub fn sprite_is_below_location(&self, slot: usize, y: u16) -> PairU8 {
        let delta = y.wrapping_sub(self.sprite_get_y(slot));
        PairU8 {
            a: u8::from((delta as i16) < 0),
            b: delta as u8,
        }
    }
    pub fn sprite_is_right_of_link(&self, slot: usize) -> PairU8 {
        self.sprite_is_right_of_location(slot, self.r16(0x22))
    }
    pub fn sprite_is_below_link(&self, slot: usize) -> PairU8 {
        let t = i32::from(self.r8(0x20)) + 8;
        let u = (t & 0xff) + i32::from(self.ram.sprite(slot).z());
        let v = (u & 0xff) - i32::from(self.ram.sprite(slot).y_low());
        let w =
            i32::from(self.r8(0x21)) - i32::from(self.ram.sprite(slot).y_high()) - i32::from(v < 0);
        let high = (w & 0xff) + (t >> 8) + (u >> 8);
        PairU8 {
            a: u8::from(((high as u8) as i8) < 0),
            b: v as u8,
        }
    }
    pub fn sprite_project_speed_towards_link(&self, slot: usize, velocity: u8) -> ProjectSpeedRet {
        Self::project_speed(
            self.sprite_is_right_of_link(slot),
            self.sprite_is_below_link(slot),
            velocity,
        )
    }
    pub fn sprite_project_speed_towards_location(
        &self,
        slot: usize,
        x: u16,
        y: u16,
        velocity: u8,
    ) -> ProjectSpeedRet {
        Self::project_speed(
            self.sprite_is_right_of_location(slot, x),
            self.sprite_is_below_location(slot, y),
            velocity,
        )
    }
    pub fn sprite_apply_speed_towards_link(&mut self, slot: usize, velocity: u8) {
        let speed = self.sprite_project_speed_towards_link(slot, velocity);
        self.ram.set_sprite_velocity_x(slot, speed.x);
        self.ram.set_sprite_velocity_y(slot, speed.y);
    }
    pub fn sprite_direction_to_face_location(&mut self, slot: usize, x: u16, y: u16) -> u8 {
        let below = self.sprite_is_below_location(slot, y);
        let right = self.sprite_is_right_of_location(slot, x);
        let ym = (below.b as i8).unsigned_abs();
        self.w8(0xfb5, ym);
        let xm = (right.b as i8).unsigned_abs();
        if xm >= ym { right.a } else { below.a + 2 }
    }
    pub fn sprite_direction_to_face_link(&mut self, slot: usize) -> (u8, PairU8) {
        let below = self.sprite_is_below_link(slot);
        let right = self.sprite_is_right_of_link(slot);
        let ym = (below.b as i8).unsigned_abs();
        self.w8(0xfb5, ym);
        let xm = (right.b as i8).unsigned_abs();
        (
            if xm >= ym { right.a } else { below.a + 2 },
            PairU8 {
                a: right.b,
                b: below.b,
            },
        )
    }
    pub fn sprite_return_if_inactive(&self, slot: usize) -> bool {
        self.ram.sprite(slot).state() != 9
            || self.r8(0xfc1) != 0
            || self.r8(0x11) != 0
            || self.ram.sprite(slot).deflection_bits() & 0x80 == 0
                && self.ram.sprite(slot).pause() != 0
    }
    pub fn sprite_return_if_paused(&self, slot: usize) -> bool {
        self.r8(0xfc1) != 0
            || self.r8(0x11) != 0
            || self.ram.sprite(slot).deflection_bits() & 0x80 == 0
                && self.ram.sprite(slot).pause() != 0
    }
    pub fn sprite_set_spawned_coordinates(&mut self, slot: usize, info: SpriteSpawnInfo) {
        self.sprite_set_x(slot, info.r0_x);
        self.sprite_set_y(slot, info.r2_y);
        self.ram.set_sprite_z(slot, info.r4_z);
    }
    pub fn sprite_check_if_overlords_clear(&self) -> bool {
        (0..8).all(|i| !matches!(self.arr(OVERLORD_TYPE, i), 0x14 | 0x18))
    }
    pub fn sprite_check_if_room_is_clear(&self) -> bool {
        (0..16).all(|i| self.ram.sprite(i).state() == 0 || self.ram.sprite(i).flags4() & 0x40 != 0)
            && self.sprite_check_if_overlords_clear()
    }
    pub fn sprite_check_if_screen_is_clear(&self) -> bool {
        (0..16).all(|i| {
            self.ram.sprite(i).state() == 0
                || self.ram.sprite(i).flags4() & 0x40 != 0
                || self.sprite_get_x(i).wrapping_sub(self.r16(0xe2)) >= 256
                || self.sprite_get_y(i).wrapping_sub(self.r16(0xe8)) >= 256
        }) && self.sprite_check_if_overlords_clear()
    }
    pub fn sprite_check_if_link_is_busy(&self) -> bool {
        if self.r8(0x4d) | self.r8(0x2da) | (self.r8(0x308) & 0x80) != 0 {
            return true;
        }
        (0..5).rev().any(|slot| self.arr(0xc4a, slot) == 0x27)
    }
    pub fn sprite_track_body_to_head(&mut self, slot: usize) -> bool {
        let head = self.ram.sprite(slot).head_direction();
        if head != self.ram.sprite(slot).scratch_d() {
            if self.r8(0x1a) & 0x1f != 0 {
                return false;
            }
            if (head ^ self.ram.sprite(slot).scratch_d()) & 2 == 0 {
                let direction = ((((slot as u8 ^ self.r8(0x1a)) >> 5) | 2) & 3) ^ (head & 2);
                self.ram.set_sprite_scratch_d(slot, direction);
                return false;
            }
        }
        self.ram.set_sprite_scratch_d(slot, head);
        true
    }

    pub fn sprite_show_message_minimal(&mut self) {
        self.w8(0x223, 0);
        self.w8(0x1cd8, 0);
        self.w8(0x11, 2);
        self.w8(0x10c, self.r8(0x10));
        self.w8(0x10, 14);
    }

    pub fn sprite_show_message_unconditional(&mut self, message: u16) {
        self.w16(0x1cf0, message);
        self.sprite_show_message_minimal();
        self.sprite_nullify_hookshot_drag();
        self.w8(0x5e, 0);
        SpriteHooks::link_cancel_dash(self.hooks, &mut self.ram);
        self.w8(0x4d, 0);
        self.w8(0x46, 0);
        if self.r8(0x5d) == 2 {
            self.w8(0x5d, 0);
        }
    }
    pub fn sprite_show_solicited_message(&mut self, slot: usize, message: u16) -> u16 {
        const FACING: [u8; 4] = [4, 6, 0, 2];
        self.w16(0x1cf0, message);
        if !self.sprite_check_damage_to_link_same_layer(slot)
            || self.sprite_check_if_link_is_busy()
            || self.r8(0xf6) & 0x80 == 0
            || self.ram.sprite(slot).aux4_delay() != 0
            || self.r8(0x4d) == 2
        {
            return u16::from(self.ram.sprite(slot).scratch_d());
        }
        let direction = self.sprite_direction_to_face_link(slot).0;
        if self.r8(0x2f) != FACING[usize::from(direction)] {
            return u16::from(self.ram.sprite(slot).scratch_d());
        }
        self.sprite_show_message_unconditional(message);
        self.ram.set_sprite_aux4_delay(slot, 64);
        u16::from(direction) ^ 0x103
    }

    pub fn sprite_show_message_on_contact(&mut self, slot: usize, message: u16) -> u16 {
        self.w16(0x1cf0, message);
        if !self.sprite_check_damage_to_link_same_layer(slot) || self.r8(0x4d) == 2 {
            return u16::from(self.ram.sprite(slot).scratch_d());
        }
        self.sprite_show_message_unconditional(message);
        u16::from(self.sprite_direction_to_face_link(slot).0) ^ 0x103
    }

    pub fn sprite_tutorial_guard_show_message_on_contact(
        &mut self,
        slot: usize,
        message: u16,
    ) -> bool {
        self.w16(0x1cf0, message);
        let flags2 = self.ram.sprite(slot).flags2();
        let flags4 = self.ram.sprite(slot).flags4();
        self.ram.set_sprite_flags2(slot, 0x80);
        self.ram.set_sprite_flags4(slot, 7);
        let result = self.sprite_check_damage_to_link_same_layer(slot);
        self.ram.set_sprite_flags2(slot, flags2);
        self.ram.set_sprite_flags4(slot, flags4);
        if result {
            self.sprite_nullify_hookshot_drag();
            self.w8(0x372, 0);
            self.w8(0x5e, 0);
            if self.r8(0x4d) == 0 {
                self.sprite_show_message_minimal();
            }
        }
        result
    }
    pub fn sprite_get_16_bit_coords(&mut self, slot: usize) {
        self.w16(0xfd8, self.sprite_get_x(slot));
        self.w16(0xfda, self.sprite_get_y(slot));
    }
    pub fn sprite_prep_oam_coord_or_double_ret(
        &mut self,
        slot: usize,
        result: &mut PrepOamCoordsRet,
    ) -> bool {
        self.ram.set_sprite_pause(slot, 0);
        let x = self.r16(0xfd8).wrapping_sub(self.r16(0xe2));
        let y = self.r16(0xfda).wrapping_sub(self.r16(0xe8));
        self.w16(0, x);
        self.w16(2, y.wrapping_sub(u16::from(self.ram.sprite(slot).z())));
        result.flags = self.ram.sprite(slot).oam_flags() ^ self.ram.sprite(slot).object_priority();
        result.r4 = 0;
        let extension = if self.hooks.enhanced_features0() & 1 != 0 {
            0x40
        } else {
            0
        };
        let out_of_bounds = x.wrapping_add(0x40 + extension) >= 0x170 + extension * 2
            || y.wrapping_add(0x40) >= 0x170 && self.ram.sprite(slot).flags4() & 0x20 == 0;
        if out_of_bounds {
            self.ram
                .set_sprite_pause(slot, self.ram.sprite(slot).pause().wrapping_add(1));
            if self.ram.sprite(slot).deflection_bits() & 0x80 == 0 {
                self.sprite_kill_self(slot);
            }
        }
        result.x = self.r16(0);
        result.y = self.r16(2);
        self.w8(0xfa8, result.x as u8);
        self.w8(0xfa9, result.y as u8);
        out_of_bounds
    }

    pub fn sprite_prep_oam_coord(&mut self, slot: usize, result: &mut PrepOamCoordsRet) {
        self.sprite_prep_oam_coord_or_double_ret(slot, result);
    }
    pub fn sprite_inactive_sprite(&mut self, slot: usize) {
        if self.r8(0x1b) == 0 {
            self.ram.set_sprite_scratch_n_word(slot, 0xffff);
        } else {
            self.ram.set_sprite_scratch_n(slot, 0xff);
        }
    }
    pub fn sprite_func3(&mut self, slot: usize) {
        self.ram.set_sprite_state(slot, 6);
        self.ram.set_sprite_main_delay(slot, 31);
        self.ram.set_sprite_flags2(slot, 3);
    }
    pub fn sprite_func18(&mut self, slot: usize, new_type: u8) {
        self.ram.set_sprite_kind(slot, new_type);
        self.sprite_prep_load_properties(slot);
        self.hooks.sprite_spawn_poof_garnish(&mut self.ram, slot);
        self.w8(0x12f, 0);
        self.hooks.sprite_sfx3_with_pan(&mut self.ram, slot, 0x32);
        self.ram.set_sprite_hit_timer(slot, 0);
        self.ram.set_sprite_damage(slot, 0);
    }
    pub fn sprite_hit_timer31(&mut self, slot: usize) {
        if self.ram.sprite(slot).kind() == 0x7a
            && self.r8(0xfff) == 0
            && self.ram.sprite(slot).health() <= self.ram.sprite(slot).damage()
        {
            self.w16(0x1cf0, 0x140);
            self.sprite_show_message_minimal();
        }
    }
    pub fn sprite_timers_and_oam(&mut self, slot: usize) {
        self.sprite_get_16_bit_coords(slot);
        let bytes = (self.ram.sprite(slot).flags2() & 0x1f).wrapping_add(1) * 4;
        if self.r8(0xfb3) != 0 {
            if self.ram.sprite(slot).floor() != 0 {
                self.oam_allocate_from_region_f(bytes);
            } else {
                self.oam_allocate_from_region_d(bytes);
            }
        } else {
            self.oam_allocate_from_region_a(bytes);
        }
        if self.r8(0x11) | self.r8(0xfc1) == 0 {
            self.ram.sprite_mut(slot).tick_action_delays();
            let timer = self.ram.sprite(slot).hit_timer() & 0x7f;
            if timer != 0 {
                if self.ram.sprite(slot).state() >= 9 && timer == 31 {
                    self.sprite_hit_timer31(slot);
                } else if self.ram.sprite(slot).state() >= 9 && timer == 24 {
                    self.sprite_mini_moldorm_recoil(slot);
                }
                if self.ram.sprite(slot).damage() < 251 {
                    self.ram.set_sprite_object_priority(
                        slot,
                        self.ram.sprite(slot).hit_timer().wrapping_mul(2) & 0xe,
                    );
                }
                self.ram
                    .set_sprite_hit_timer(slot, self.ram.sprite(slot).hit_timer().wrapping_sub(1));
            } else {
                self.ram.set_sprite_hit_timer(slot, 0);
                self.ram.set_sprite_object_priority(slot, 0);
            }
            let value = self.ram.sprite(slot).aux4_delay();
            if value != 0 {
                self.ram.set_sprite_aux4_delay(slot, value.wrapping_sub(1));
            }
        }
        const PRIORITIES: [u8; 4] = [0x20, 0x10, 0x30, 0x30];
        let mut floor = self.r8(0xee);
        if floor != 3 {
            floor = self.ram.sprite(slot).floor();
        }
        self.ram.set_sprite_object_priority(
            slot,
            self.ram.sprite(slot).object_priority() & 0xcf | PRIORITIES[usize::from(floor)],
        );
    }
    pub fn sprite_execute_single(&mut self, slot: usize) {
        let state = self.ram.sprite(slot).state();
        if state != 0 {
            self.sprite_timers_and_oam(slot);
        }
        match SPRITE_EXECUTE_SINGLE[usize::from(state)] {
            0 => self.sprite_inactive_sprite(slot),
            1 => self.sprite_module_fall1(slot),
            2 => self.sprite_module_poof(slot),
            3 => self.sprite_module_drown(slot),
            4 => self.sprite_module_explode(slot),
            5 => self.sprite_module_fall2(slot),
            6 => self.sprite_module_die(slot),
            7 => self.sprite_module_burn(slot),
            8 => self.run_sprite_module_initialize(slot),
            9 => self.run_sprite_active_main(slot),
            10 => self.sprite_module_carried(slot),
            11 => self.sprite_module_stunned(slot),
            _ => unreachable!("invalid sprite state"),
        }
    }

    pub fn execute_cached_sprites(&mut self) {
        if self.r8(0x1b) == 0 || self.r8(0x11) == 0 || self.r8(0x11) == 14 || self.r8(0xffa) == 0 {
            self.w8(0xffa, 0);
            return;
        }
        for slot in SpriteSlot::all().rev() {
            let slot = slot.index();
            self.w8(0xfa0, slot as u8);
            if self.ram.cached_sprite_present(slot) {
                self.uncache_and_execute_sprite(slot);
            }
        }
    }

    pub fn sprite_main(&mut self) {
        if self.r8(0x1b) == 0 {
            self.ram.slice_mut(0xc7c, 5).fill(0);
            self.sprite_proximity_activation();
        }
        self.w8(0xfff, u8::from(self.r8(0xf3ca) != 0));
        if self.r8(0x11) == 0 {
            self.w16(0xb7c, 0);
            self.w16(0xb7e, 0);
        }
        self.oam_reset_region_bases();
        self.garnish_execute_upper_slots();
        self.hooks.follower_main(&mut self.ram);
        self.w8(0xfb2, self.r8(0x314));
        self.w8(0x314, 0);
        self.w8(0xfab, 0x80);
        let damage_timer = self.r8(0x47);
        self.w8(
            0x47,
            if damage_timer & 0x7f != 0 {
                damage_timer.wrapping_sub(1)
            } else {
                0
            },
        );
        self.w8(0x379, 0);
        self.w8(0x377, 0);
        self.w8(0xb7b, 0);
        if self.r8(0xfdc) != 0 {
            self.w8(0xfdc, self.r8(0xfdc).wrapping_sub(1));
        }
        self.hooks.ancilla_main(&mut self.ram);
        self.hooks.overlord_main(&mut self.ram);
        self.w8(0xb9a, 0);
        for slot in SpriteSlot::all().rev() {
            let slot = slot.index();
            self.w8(0xfa0, slot as u8);
            self.sprite_execute_single(slot);
        }
        self.garnish_execute_lower_slots();
        self.w8(0x69e, 0);
        self.w8(0x69f, 0);
        self.execute_cached_sprites();
        if self.r8(0xaaa) != 0 {
            self.w8(0xfc6, self.r8(0xaaa));
        }
    }

    pub fn uncache_and_execute_sprite(&mut self, slot: usize) {
        let backup = self.ram.activate_cached_sprite(slot);
        self.sprite_execute_single(slot);
        if self.ram.sprite(slot).pause() != 0 {
            self.ram.clear_cached_sprite(slot);
        }
        self.ram.restore_sprite_transient(slot, backup);
    }
    pub fn sprite_prep_reset_properties(&mut self, slot: usize) {
        self.ram.sprite_mut(slot).reset_transient();
    }
    pub fn sprite_prep_load_properties(&mut self, slot: usize) {
        self.sprite_prep_reset_properties(slot);
        let kind = usize::from(self.ram.sprite(slot).kind());
        self.ram
            .sprite_mut(slot)
            .apply_initial_properties(SpriteInitialProperties {
                flags2: SPRITE_INIT_FLAGS2[kind],
                health: SPRITE_INIT_HEALTH[kind],
                flags4: SPRITE_INIT_FLAGS4[kind],
                flags5: SPRITE_INIT_FLAGS5[kind],
                deflection_bits: SPRITE_INIT_DEFL_BITS[kind],
                bump_damage: SPRITE_INIT_BUMP_DAMAGE[kind],
                flags: SPRITE_INIT_FLAGS[kind],
                flags3: SPRITE_INIT_FLAGS3[kind],
                oam_flags: SPRITE_INIT_FLAGS3[kind] & 15,
            });
        let room = if self.r8(0x1b) != 0 {
            self.r8(0x48e)
        } else {
            self.r8(0x40a)
        };
        self.ram.set_sprite_room(slot, room);
    }
    pub fn sprite_prep_load_palette(&mut self, slot: usize) {
        let flags = SPRITE_INIT_FLAGS3[usize::from(self.ram.sprite(slot).kind())];
        self.ram.set_sprite_flags3(slot, flags);
        self.ram.set_sprite_oam_flags(slot, flags & 15);
    }
    pub fn oam_get_buffer_position(&mut self, number: u8, region: u8) -> u8 {
        self.allocate_oam_bytes(number, oam::OamRegion::from_legacy_selector(region))
    }
    fn allocate_oam_bytes(&mut self, number: u8, region: oam::OamRegion) -> u8 {
        oam::OamAllocator::new(&mut self.ram)
            .allocate_bytes(region, number)
            .legacy_low_byte()
    }
    pub fn oam_allocate_from_region_a(&mut self, n: u8) -> u8 {
        self.allocate_oam_bytes(n, oam::OamRegion::A)
    }
    pub fn oam_allocate_from_region_b(&mut self, n: u8) -> u8 {
        self.allocate_oam_bytes(n, oam::OamRegion::B)
    }
    pub fn oam_allocate_from_region_c(&mut self, n: u8) -> u8 {
        self.allocate_oam_bytes(n, oam::OamRegion::C)
    }
    pub fn oam_allocate_from_region_d(&mut self, n: u8) -> u8 {
        self.allocate_oam_bytes(n, oam::OamRegion::D)
    }
    pub fn oam_allocate_from_region_e(&mut self, n: u8) -> u8 {
        self.allocate_oam_bytes(n, oam::OamRegion::E)
    }
    pub fn oam_allocate_from_region_f(&mut self, n: u8) -> u8 {
        self.allocate_oam_bytes(n, oam::OamRegion::F)
    }
    pub fn oam_reset_region_bases(&mut self) {
        oam::OamAllocator::new(&mut self.ram).reset_region_bases();
    }
    pub fn sprite_convert_velocity_to_angle(mut x: u8, mut y: u8) -> u8 {
        const A: [u8; 32] = [
            0, 0, 1, 1, 1, 2, 2, 2, 0, 0, 15, 15, 15, 14, 14, 14, 8, 8, 7, 7, 7, 6, 6, 6, 8, 8, 9,
            9, 9, 10, 10, 10,
        ];
        const B: [u8; 32] = [
            4, 4, 3, 3, 3, 2, 2, 2, 12, 12, 13, 13, 13, 14, 14, 14, 4, 4, 5, 5, 5, 6, 6, 6, 12, 12,
            11, 11, 11, 10, 10, 10,
        ];
        let sign = usize::from((y >> 7) + (x >> 7) * 2) * 8;
        if (x as i8) < 0 {
            x = VelocityByte::from_bits(x).negated().bits();
        }
        if (y as i8) < 0 {
            y = VelocityByte::from_bits(y).negated().bits();
        }
        if x >= y {
            A[usize::from(y >> 2) + sign]
        } else {
            B[usize::from(x >> 2) + sign]
        }
    }
    pub fn sprite_apply_conveyor(&mut self, slot: usize, tile: u8) {
        if self.r8(0x1a) & 1 == 0 {
            return;
        }
        let i = usize::from(tile - 0x68);
        self.sprite_add_xy(slot, [0, 0, -1, 1][i], [-1, 1, 0, 0][i]);
    }
    pub fn sprite_func8(&mut self, slot: usize) {
        self.ram.set_sprite_state(slot, 1);
        self.ram.set_sprite_main_delay(slot, 0x1f);
        self.w8(0x12e, 0);
        self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 0x20);
    }

    pub fn sprite_func22(&mut self, slot: usize) {
        let sound = SoundRequest::at_world_x(&self.ram, 0x28, self.sprite_get_x(slot));
        SoundPort::EffectsA.overwrite(&mut self.ram, sound);
        self.ram.set_sprite_state(slot, 3);
        self.ram.set_sprite_main_delay(slot, 15);
        self.ram.set_sprite_ai_state(slot, 0);
        self.hooks.random_u8(&mut self.ram);
        self.ram.set_sprite_flags2(slot, 3);
    }
    pub fn sprite_spawn_dynamically_ex(
        &mut self,
        parent: usize,
        kind: u8,
        info: &mut SpriteSpawnInfo,
        mut candidate: i32,
    ) -> i32 {
        while candidate >= 0 {
            let slot = candidate as usize;
            if self.ram.sprite(slot).state() == 0 {
                self.ram.set_sprite_kind(slot, kind);
                self.ram.set_sprite_state(slot, 9);
                info.r0_x = self.sprite_get_x(parent);
                info.r2_y = self.sprite_get_y(parent);
                info.r4_z = self.ram.sprite(parent).z();
                info.r5_overlord_x =
                    u16::from(self.arr(0xb08, parent)) | u16::from(self.arr(0xb10, parent)) << 8;
                info.r7_overlord_y =
                    u16::from(self.arr(0xb18, parent)) | u16::from(self.arr(0xb20, parent)) << 8;
                self.sprite_prep_load_properties(slot);
                if self.r8(0x1b) == 0 {
                    self.ram.set_sprite_scratch_n_word(slot, 0xffff);
                } else {
                    self.ram.set_sprite_scratch_n(slot, 0xff);
                }
                self.ram
                    .set_sprite_floor(slot, self.ram.sprite(parent).floor());
                self.ram
                    .set_sprite_scratch_d(slot, self.ram.sprite(parent).scratch_d());
                self.ram.set_sprite_death_action(slot, 0);
                self.ram.set_sprite_subtype(slot, 0);
                return candidate;
            }
            candidate -= 1;
        }
        -1
    }
    pub fn sprite_spawn_dynamically(
        &mut self,
        parent: usize,
        kind: u8,
        info: &mut SpriteSpawnInfo,
    ) -> i32 {
        self.sprite_spawn_dynamically_ex(parent, kind, info, 15)
    }
    pub fn sprite_kill_self(&mut self, slot: usize) {
        if self.ram.sprite(slot).deflection_bits() & 0x40 == 0 && self.r8(0x1b) != 0 {
            return;
        }
        self.ram.set_sprite_state(slot, 0);
        let block = self.ram.sprite(slot).scratch_n_word();
        self.w8(0, block as u8);
        self.w16(1, (block >> 3).wrapping_add(0xef80));
        if block != 0xffff {
            // The C port narrows this sum to uint16 before adding the WRAM bank.
            let address = 0x10000 + usize::from(0xef80u16.wrapping_add(block >> 3));
            self.w8(address, self.r8(address) & !(0x80 >> (block & 7)));
        }
        if self.r8(0x1b) == 0 {
            self.ram.set_sprite_scratch_n_word(slot, 0xffff);
        } else {
            self.ram.set_sprite_scratch_n(slot, 0xff);
        }
    }
    pub fn sprite_initialize_slots(&mut self) {
        for slot in SpriteSlot::all().rev() {
            let slot = slot.index();
            let state = self.ram.sprite(slot).state();
            let kind = self.ram.sprite(slot).kind();
            if state == 10 && !matches!(kind, 0xec | 0xd2) {
                self.w8(0x309, 0);
                self.w8(0x308, 0);
                self.ram.set_sprite_state(slot, 0);
            } else if state != 0 && kind != 0x6c && self.ram.sprite(slot).room() != self.r8(0x40a) {
                self.ram.set_sprite_state(slot, 0);
            }
        }
        for slot in (0..8).rev() {
            if self.arr(OVERLORD_TYPE, slot) != 0 && self.arr(0xcca, slot) != self.r8(0x40a) {
                self.set_arr(OVERLORD_TYPE, slot, 0);
            }
        }
    }
    pub fn sprite_initialize_mirror_portal(&mut self) {
        for slot in SpriteSlot::all().rev() {
            let slot = slot.index();
            if self.ram.sprite(slot).state() != 0 && self.ram.sprite(slot).kind() == 0x6c {
                self.ram.set_sprite_state(slot, 0);
            }
        }
        let mut info = SpriteSpawnInfo::default();
        let spawned = self.sprite_spawn_dynamically(0xff, 0x6c, &mut info);
        let slot = if spawned < 0 { 0 } else { spawned as usize };
        self.sprite_set_x(
            slot,
            u16::from(self.arr(0x1ab0, 15)) | u16::from(self.arr(0x1ac0, 15)) << 8,
        );
        self.sprite_set_y(
            slot,
            (u16::from(self.arr(0x1ad0, 15)) | u16::from(self.arr(0x1ae0, 15)) << 8)
                .wrapping_add(8),
        );
        self.ram.set_sprite_floor(slot, 0);
        self.ram.set_sprite_ignore_projectile(slot, 1);
    }
    pub fn sprite_disable_all(&mut self) {
        for slot in SpriteSlot::all().rev() {
            let slot = slot.index();
            if self.ram.sprite(slot).state() != 0
                && (self.r8(0x1b) != 0 || self.ram.sprite(slot).kind() != 0x6c)
            {
                self.ram.set_sprite_state(slot, 0);
            }
        }
        for slot in (0..10).rev() {
            self.set_arr(0xc4a, slot, 0);
        }
        for address in [
            0x2ec,
            0xb6a,
            0xb9b,
            0xb88,
            0xb99,
            crate::garnish_state::ACTIVE_MARKER_ADDRESS,
            0xb9e,
            0xcf4,
            0xff9,
            crate::garnish_state::SHARED_OVERWRITE_CURSOR_ADDRESS,
            0xffb,
            0xffc,
            0xffd,
            0xfc6,
            0x3fc,
        ] {
            self.w8(address, 0);
        }
        for slot in (0..8).rev() {
            self.set_arr(OVERLORD_TYPE, slot, 0);
        }
        for slot in (0..30).rev() {
            self.ram.set_garnish_kind(slot, 0);
        }
    }
    pub fn sprite_reset_all_no_disable(&mut self) {
        for address in [0xfdd, 0xfdc, 0xffd, 0x2f0, 0xfc6, 0xb6a, 0xfb3] {
            self.w8(address, 0);
        }
        if self.r8(0xf3cc) != 13 {
            self.w8(0x4b4, 0xfe);
        }
        self.ram.slice_mut(0x1df80, 0x1000).fill(0);
        self.ram.slice_mut(0x1ef80, 0x200).fill(0);
        self.ram.slice_mut(0xb80, 8).fill(0xff);
    }
    pub fn sprite_reset_all(&mut self) {
        self.sprite_disable_all();
        self.sprite_reset_all_no_disable();
    }
    pub fn sprite_manually_set_death_flag_uw(&mut self, slot: usize) {
        if self.r8(0x1b) == 0
            || self.ram.sprite(slot).deflection_bits() & 1 != 0
            || (self.ram.sprite(slot).scratch_n() as i8) < 0
        {
            return;
        }
        let address = 0x1df80 + usize::from(self.r16(0x48e)) * 2;
        self.w16(
            address,
            self.r16(address) | 1u16 << self.ram.sprite(slot).scratch_n(),
        );
    }
    pub fn dungeon_load_single_sprite(&mut self, slot: usize, source: &[u8]) -> usize {
        let y = source[0];
        let x = source[1];
        let kind = source[2];
        if kind == 0xe4 && matches!(y, 0xfe | 0xfd) {
            let previous = slot.wrapping_sub(1);
            self.ram
                .set_sprite_death_action(previous, if y == 0xfe { 1 } else { 2 });
            return previous;
        }
        if kind != 0xe4 && x >= 0xe0 {
            self.dungeon_load_single_overlord(source);
            return slot.wrapping_sub(1);
        }
        let room = usize::from(self.r16(0x48e));
        if SPRITE_INIT_DEFL_BITS[usize::from(kind)] & 1 == 0
            && self.r16(0x1df80 + room * 2) & (1u16 << slot) != 0
        {
            return slot;
        }
        self.ram.set_sprite_state(slot, 8);
        self.w8(0xfb5, y);
        self.ram.set_sprite_floor(slot, y >> 7);
        self.sprite_set_y(
            slot,
            (u16::from(y) << 4 & 0x1ff).wrapping_add(u16::from(self.r8(0xfb1)) << 8),
        );
        self.w8(0xfb6, x);
        self.sprite_set_x(
            slot,
            (u16::from(x) << 4 & 0x1ff).wrapping_add(u16::from(self.r8(0xfb0)) << 8),
        );
        self.ram.set_sprite_kind(slot, kind);
        self.w8(0xfb5, (self.r8(0xfb5) & 0x60) >> 2);
        self.ram
            .set_sprite_subtype(slot, self.r8(0xfb5) | self.r8(0xfb6) >> 5);
        self.ram.set_sprite_scratch_n(slot, slot as u8);
        self.ram.set_sprite_death_action(slot, 0);
        slot
    }

    pub fn dungeon_load_single_overlord(&mut self, source: &[u8]) {
        let Some(slot) = self.alloc_overlord() else {
            return;
        };
        let y = source[0];
        let x = source[1];
        let kind = source[2];
        self.set_arr(OVERLORD_TYPE, slot, kind);
        self.set_arr(0xb40, slot, y >> 7);
        let ypos = (u16::from(y) << 4 & 0x1ff).wrapping_add(u16::from(self.r8(0xfb1)) << 8);
        self.set_arr(0xb18, slot, ypos as u8);
        self.set_arr(0xb20, slot, (ypos >> 8) as u8);
        let xpos = (u16::from(x) << 4 & 0x1ff).wrapping_add(u16::from(self.r8(0xfb0)) << 8);
        self.set_arr(0xb08, slot, xpos as u8);
        self.set_arr(0xb10, slot, (xpos >> 8) as u8);
        self.set_arr(0xcca, slot, self.r8(0x40a));
        self.set_arr(0xb28, slot, 0);
        self.set_arr(0xb30, slot, 0);
        self.set_arr(0xb38, slot, 0);
        if matches!(kind, 10 | 11) {
            self.set_arr(0xb30, slot, 160);
        } else if kind == 3 {
            self.set_arr(0xb30, slot, 255);
            self.set_arr(0xb08, slot, self.arr(0xb08, slot).wrapping_sub(8));
        }
    }

    pub fn dungeon_cache_trans_sprites(&mut self) {
        if self.r8(0x1b) == 0 {
            return;
        }
        self.w8(0xffa, self.r8(0x1b));
        for slot in SpriteSlot::all().rev() {
            self.ram.cache_sprite_for_transition(slot.index());
        }
    }

    pub fn dungeon_load_sprites(&mut self) {
        let room = self.r16(0x48e);
        self.w8(0xfb1, ((room >> 3) as u8) & 0xfe);
        self.w8(0xfb0, ((room as u8) & 0x0f) << 1);
        let source = self.hooks.dungeon_sprite_data(room).to_vec();
        if source.is_empty() {
            return;
        }
        self.w8(0xfb3, source[0]);
        let mut slot = 0usize;
        for entry in source[1..].chunks_exact(3) {
            if entry[0] == 0xff {
                break;
            }
            slot = self.dungeon_load_single_sprite(slot, entry).wrapping_add(1);
        }
    }

    pub fn dungeon_reset_sprites(&mut self) {
        self.dungeon_cache_trans_sprites();
        self.w8(0x309, 0);
        self.w8(0x308, 0);
        self.sprite_disable_all();
        self.w16(0xfb8, 0xffff);
        self.w16(0xfba, 0xffff);
        let room = self.r16(0x48e);
        let found = (0..4).any(|index| self.r16(0xb80 + index * 2) == room);
        if !found {
            let evicted = self.r16(0xb86);
            self.w16(0xb86, self.r16(0xb84));
            self.w16(0xb84, self.r16(0xb82));
            self.w16(0xb82, self.r16(0xb80));
            self.w16(0xb80, room);
            if evicted != 0xffff {
                self.w16(0x1df80 + usize::from(evicted) * 2, 0);
            }
        }
        self.dungeon_load_sprites();
    }

    pub fn sprite_spawn_secret(&mut self, slot: usize) {
        if self.r8(0x1b) == 0 && self.hooks.random_u8(&mut self.ram) & 8 != 0 {
            return;
        }
        let mut secret = self.r8(0xb9c);
        if secret == 0 {
            return;
        }
        if secret == 4 {
            secret = 19 + (self.hooks.random_u8(&mut self.ram) & 3);
        }
        let index = usize::from(secret - 1);
        let kind = SPAWN_SECRET_ITEMS[index];
        if kind == 0 {
            return;
        }
        let mut info = SpriteSpawnInfo::default();
        let spawned = self.sprite_spawn_dynamically(slot, kind, &mut info);
        if spawned < 0 {
            return;
        }
        let spawned = spawned as usize;
        self.ram
            .set_sprite_ai_state(spawned, SPAWN_SECRET_ITEM_SPAWN_FLAG[index]);
        self.ram
            .set_sprite_ignore_projectile(spawned, SPAWN_SECRET_ITEM_IGNORE_PROJ[index]);
        self.ram
            .set_sprite_velocity_z(spawned, SPAWN_SECRET_ITEM_Z_VEL[index]);
        self.sprite_set_x(
            spawned,
            info.r0_x
                .wrapping_add(u16::from(SPAWN_SECRET_ITEM_X_LO[index])),
        );
        self.sprite_set_y(spawned, info.r2_y);
        self.ram.set_sprite_z(spawned, info.r4_z);
        self.ram.set_sprite_graphics(spawned, 0);
        self.ram.set_sprite_aux4_delay(spawned, 32);
        self.ram.set_sprite_aux2_delay(spawned, 48);
        match kind {
            0xe4 => {
                self.hooks.sprite_prep_small_key(&mut self.ram, spawned);
                self.ram.set_sprite_stunned(spawned, 0xff);
            }
            0x0b => {
                self.w8(0x12e, 0x30);
                if self.r8(0x48e) == 1 {
                    self.ram.set_sprite_subtype(spawned, 1);
                }
                self.ram.set_sprite_stunned(spawned, 0xff);
            }
            0x41 | 0x42 => {
                self.w8(0x12f, 4);
                self.ram.set_sprite_damage(spawned, 0);
                self.ram.set_sprite_hit_timer(spawned, 160);
            }
            0x3e => self.ram.set_sprite_oam_flags(spawned, 9),
            _ => {
                self.ram.set_sprite_stunned(spawned, 0xff);
                if kind == 0x79 {
                    self.ram.set_sprite_scratch_a(spawned, 32);
                }
            }
        }
    }

    pub fn sprite_spawn_throwable_terrain_silently(&mut self, what: u8, x: u16, y: u16) -> i32 {
        let Some(slot) = self
            .ram
            .sprites()
            .find_inactive_rev()
            .map(SpriteSlot::index)
        else {
            return -1;
        };
        self.ram.set_sprite_state(slot, 10);
        self.ram.set_sprite_kind(slot, 0xec);
        self.sprite_set_x(slot, x);
        self.sprite_set_y(slot, y);
        self.sprite_prep_load_properties(slot);
        self.ram.set_sprite_floor(slot, self.r8(0xee));
        self.ram.set_sprite_scratch_c(slot, what);
        if what >= 6 {
            self.ram.set_sprite_flags2(slot, 0xa6);
        }
        let mut flags = self.hooks.throwable_scenery_flags(what);
        if what == 2 && self.r8(0x1b) != 0 {
            self.ram.set_sprite_oam_flags(slot, 0x80);
            flags = 0x50;
        }
        self.ram.set_sprite_oam_flags(slot, flags);
        self.ram.set_sprite_unknown4(slot, 9);
        self.w8(0x314, 2);
        self.w8(0xfb2, 2);
        self.ram.set_sprite_main_delay(slot, 16);
        self.ram.set_sprite_floor(slot, self.r8(0xee));
        self.ram.set_sprite_graphics(slot, 0);
        if self.r8(0xb9c) != 0xff {
            if self.r8(0xb9c) | self.r8(0x1b) == 0
                && self.ram.sprite(slot).scratch_c().wrapping_sub(2) < 2
            {
                self.overworld_substitute_alternate_secret();
            }
            if self.r16(0xb9c) & 0x80 != 0 {
                self.ram.set_sprite_graphics(slot, self.r8(0xb9c) & 0x7f);
                self.w8(0xb9c, 0);
            }
            self.sprite_spawn_secret(slot);
        }
        slot as i32
    }

    pub fn sprite_spawn_throwable_terrain(&mut self, what: u8, x: u16, y: u16) {
        let sound = self.hooks.link_calculate_sfx_pan(&self.ram) | 29;
        self.w8(0x12e, sound);
        self.sprite_spawn_throwable_terrain_silently(what, x, y);
    }

    pub fn sprite_spawn_immediately_smashed_terrain(&mut self, what: u8, x: u16, y: u16) {
        let pickup = self.r8(0x314);
        let cached_pickup = self.r8(0xfb2);
        let slot = self.sprite_spawn_throwable_terrain_silently(what, x, y);
        if slot >= 0 {
            self.throwable_scenery_transmute_to_debris(slot as usize);
        }
        self.w8(0xfb2, cached_pickup);
        self.w8(0x314, pickup);
    }

    pub fn sprite_schedule_for_breakage(&mut self, slot: usize) {
        self.ram.set_sprite_main_delay(slot, 31);
        self.ram.set_sprite_state(slot, 6);
        self.ram
            .set_sprite_flags2(slot, self.ram.sprite(slot).flags2().wrapping_add(4));
    }

    pub fn throwable_scenery_transmute_to_debris(&mut self, slot: usize) {
        let graphics = self.ram.sprite(slot).graphics();
        if graphics != 0 {
            self.w8(0xb9c, graphics);
            self.sprite_spawn_secret(slot);
            self.w8(0xb9c, 0);
        }
        let kind = if self.r8(0x1b) != 0 {
            0
        } else {
            self.ram.sprite(slot).scratch_c()
        };
        self.w8(0x12e, 0);
        self.hooks
            .sprite_sfx2_with_pan(&mut self.ram, slot, SPRITE_FUNC21_SFX[usize::from(kind)]);
        self.sprite_schedule_for_breakage(slot);
    }

    pub fn throwable_scenery_transmute_if_valid(&mut self, slot: usize) {
        if self.ram.sprite(slot).kind() == 0xec {
            self.w8(0xfac, 0);
            self.throwable_scenery_transmute_to_debris(slot);
        }
    }

    pub fn sprite_apply_ricochet(&mut self, slot: usize) {
        self.sprite_invert_xy_speeds(slot);
        self.sprite_halve_speed_xy(slot);
        self.throwable_scenery_transmute_if_valid(slot);
    }

    pub fn sprite_spawn_leaping_fish(&mut self, slot: usize) {
        let mut info = SpriteSpawnInfo::default();
        let spawned = self.sprite_spawn_dynamically(slot, 0xd2, &mut info);
        if spawned < 0 {
            return;
        }
        let spawned = spawned as usize;
        self.sprite_set_spawned_coordinates(spawned, info);
        self.ram.set_sprite_ai_state(spawned, 2);
        self.ram.set_sprite_main_delay(spawned, 48);
        if self.ram.sprite(slot).kind() == 0xd2 {
            self.ram.set_sprite_scratch_a(spawned, 0xd2);
        }
    }

    pub fn sprite_garnish_spawn_sparkle_limited(&mut self, slot: usize, x: u16, y: u16) {
        self.sprite_spawn_simple_sparkle_garnish_ex(slot, x, y, 14);
    }

    pub fn sprite_garnish_spawn_sparkle(&mut self, slot: usize, x: u16, y: u16) -> i32 {
        self.sprite_spawn_simple_sparkle_garnish_ex(slot, x, y, 29)
    }

    pub fn sprite_stunned_main_func1(&mut self, slot: usize) {
        self.run_sprite_active_main(slot);
        if self.ram.sprite(slot).unknown5() != 0 {
            if self.ram.sprite(slot).main_delay() < 32 {
                self.ram
                    .set_sprite_oam_flags(slot, self.ram.sprite(slot).oam_flags() & 0xf1 | 4);
            }
            let timer = self.ram.sprite(slot).main_delay();
            let t = (((slot as u8) << 4) ^ self.r8(0x1a)) | self.r8(0x11);
            if t & SPRITE_STUNNED_MAIN_FUNC1_MASKS[usize::from(timer >> 4)] != 0 {
                return;
            }
            let x = SPARKLE_GARNISH_XY[usize::from(self.hooks.random_u8(&mut self.ram) & 3)] as i16
                as u16;
            let y = SPARKLE_GARNISH_XY[usize::from(self.hooks.random_u8(&mut self.ram) & 3)] as i16
                as u16;
            self.sprite_garnish_spawn_sparkle(slot, x, y);
        } else {
            if self.r8(0x1a) & 1 != 0 || self.r8(0x11) != 0 || self.r8(0xfc1) != 0 {
                return;
            }
            let timer = self.ram.sprite(slot).stunned();
            if timer != 0 {
                self.ram.set_sprite_stunned(slot, timer.wrapping_sub(1));
                if timer < 0x38 {
                    self.ram
                        .set_sprite_velocity_x(slot, if timer & 1 != 0 { 0xf8 } else { 8 });
                    self.sprite_move_x(slot);
                }
                return;
            }
            self.ram.set_sprite_state(slot, 9);
            self.ram.set_sprite_recoil_x(slot, 0);
            self.ram.set_sprite_recoil_y(slot, 0);
        }
    }

    pub fn sprite_module_carried(&mut self, slot: usize) {
        self.ram.set_sprite_room(slot, self.r16(0x40a) as u8);
        if self.ram.sprite(slot).unknown3() != 3 {
            if self.ram.sprite(slot).main_delay() == 0 {
                self.ram.set_sprite_main_delay(
                    slot,
                    if self.ram.sprite(slot).scratch_c() == 6 {
                        8
                    } else {
                        4
                    },
                );
                self.ram
                    .set_sprite_unknown3(slot, self.ram.sprite(slot).unknown3().wrapping_add(1));
            }
        } else {
            self.ram
                .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() & !0x10);
        }
        let timer = self.ram.sprite(slot).aux4_delay().wrapping_sub(1);
        let shake = u8::from(timer < 63 && timer & 2 != 0);
        let index = usize::from(self.r8(0x2f)) * 2 + usize::from(self.ram.sprite(slot).unknown3());
        let offset = SPRITE_HELD_X[index] as u8;
        let t0 = u16::from(self.r16(0x22) as u8) + u16::from(offset);
        let t1 = u16::from(t0 as u8) + ((t0 >> 8) & 1) + u16::from(shake);
        let t2 = u16::from((self.r16(0x22) >> 8) as u8)
            + ((t1 >> 8) & 1)
            + ((t0 >> 8) & 1)
            + u16::from((i16::from(SPRITE_HELD_X[index]) >> 8) as u8);
        self.ram.set_sprite_x_low(slot, t1 as u8);
        self.ram.set_sprite_x_high(slot, t2 as u8);
        self.ram.set_sprite_z(slot, SPRITE_HELD_Z[index]);
        let animation = usize::from(if self.r8(0x2e) < 6 { self.r8(0x2e) } else { 0 });
        let z = self
            .r16(0x24)
            .wrapping_add(1)
            .wrapping_add(u16::from(SPRITE_HELD_Z_FOR_FRAME[animation]));
        self.sprite_set_y(slot, self.r16(0x20).wrapping_add(8).wrapping_sub(z));
        self.ram.set_sprite_floor(slot, self.r8(0xee) & 1);
        self.carried_sprite_check_for_throw(slot);
        self.sprite_get_16_bit_coords(slot);
        if self.ram.sprite(slot).unknown4() != 11 {
            self.run_sprite_active_main(slot);
            if self.ram.sprite(slot).aux4_delay() == 1 {
                self.ram.set_sprite_state(slot, 9);
                self.ram.set_sprite_scratch_b(slot, 0);
                self.ram.set_sprite_aux4_delay(slot, 96);
                self.ram.set_sprite_velocity_z(slot, 32);
                self.ram
                    .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() | 0x10);
                self.w8(0x309, 2);
            }
        } else {
            self.sprite_stunned_main_func1(slot);
        }
    }

    pub fn carried_sprite_check_for_throw(&mut self, slot: usize) {
        if self.r8(0x10) == 14 {
            return;
        }
        if self.r8(0x5b) != 2 {
            let busy = (self.r8(0x4d) & 1)
                | self.r8(0x345)
                | self.r8(0x2e0)
                | self.r8(0x2da)
                | if self.r8(0x37b) != 0 {
                    0
                } else {
                    self.r8(0x46)
                };
            if busy == 0 {
                if self.ram.sprite(slot).unknown3() != 3
                    || (self.r8(0xf4) | self.r8(0xf6)) & 0x80 == 0
                {
                    return;
                }
                self.w8(0xf6, self.r8(0xf6) & 0x7f);
            }
        }
        self.hooks.sprite_sfx3_with_pan(&mut self.ram, slot, 0x13);
        self.w8(0x309, 2);
        self.ram
            .set_sprite_state(slot, self.ram.sprite(slot).unknown4());
        self.ram.set_sprite_velocity_z(slot, 0);
        self.ram.set_sprite_unknown3(slot, 0);
        let kind = usize::from(self.ram.sprite(slot).kind());
        self.ram.set_sprite_flags3(
            slot,
            (self.ram.sprite(slot).flags3() & !0x10) | (SPRITE_INIT_FLAGS3[kind] & 0x10),
        );
        let direction = usize::from(self.r8(0x2f) >> 1);
        self.ram
            .set_sprite_velocity_x(slot, SPRITE_HELD_THROW_X_VEL[direction] as u8);
        self.ram
            .set_sprite_velocity_y(slot, SPRITE_HELD_THROW_Y_VEL[direction] as u8);
        self.ram
            .set_sprite_velocity_z(slot, SPRITE_HELD_THROW_Z_VEL[direction]);
        self.ram.set_sprite_aux4_delay(slot, 0);
    }

    pub fn sprite_module_stunned(&mut self, slot: usize) {
        self.sprite_stunned_main_ex(slot, false);
    }

    pub fn thrown_sprite_tile_and_sprite_interaction(&mut self, slot: usize) {
        self.sprite_stunned_main_ex(slot, true);
    }

    pub fn throwable_scenery_interact_with_sprites_and_tiles(&mut self, slot: usize) {
        self.sprite_move_xy(slot);
        if self.ram.sprite(slot).scratch_e() == 0 {
            self.sprite_check_tile_collision(slot);
        }
        self.thrown_sprite_tile_and_sprite_interaction(slot);
    }

    pub fn thrown_sprite_check_damage_to_sprites(&mut self, slot: usize) {
        if self.ram.sprite(slot).aux4_delay() != 0
            || self.ram.sprite(slot).velocity_x() | self.ram.sprite(slot).velocity_y() == 0
        {
            return;
        }
        for other in (0..16).rev() {
            if other != self.r8(0xfa0) as usize
                && self.ram.sprite(slot).kind() != 0xd2
                && self.ram.sprite(other).state() >= 9
                && (((other as u8 ^ self.r8(0x1a)) & 3)
                    | self.ram.sprite(other).ignore_projectile()
                    | self.ram.sprite(other).hit_timer())
                    == 0
                && self.ram.sprite(slot).floor() == self.ram.sprite(other).floor()
            {
                self.thrown_sprite_check_damage_to_single_sprite(slot, other);
            }
        }
    }

    pub fn thrown_sprite_check_damage_to_single_sprite(&mut self, slot: usize, other: usize) {
        let mut hitbox = SpriteHitBox {
            r0_xlo: self.ram.sprite(slot).x_low(),
            r8_xhi: self.ram.sprite(slot).x_high(),
            r2: 15,
            ..SpriteHitBox::default()
        };
        let y = i16::from(self.ram.sprite(slot).y_low()) - i16::from(self.ram.sprite(slot).z());
        let adjusted = u16::from(y as u8) + 8;
        hitbox.r1_ylo = adjusted as u8;
        hitbox.r9_yhi = self
            .ram
            .sprite(slot)
            .y_high()
            .wrapping_add((adjusted >> 8) as u8)
            .wrapping_sub(u8::from(y < 0));
        hitbox.r3 = 8;
        self.sprite_setup_hit_box(other, &mut hitbox);
        if !Self::check_if_hit_boxes_overlap(&hitbox) {
            return;
        }
        if self.ram.sprite(other).kind() == 0x3f {
            self.sprite_place_weapon_tink(slot);
        } else {
            let damage = if self.ram.sprite(slot).kind() == 0xec
                && self.ram.sprite(slot).scratch_c() == 2
                && self.r8(0x1b) == 0
            {
                1
            } else {
                3
            };
            self.ancilla_check_damage_to_sprite_preset(other, damage);
            self.ram
                .set_sprite_recoil_x(other, self.ram.sprite(slot).velocity_x().wrapping_mul(2));
            self.ram
                .set_sprite_recoil_y(other, self.ram.sprite(slot).velocity_y().wrapping_mul(2));
            self.ram.set_sprite_aux4_delay(slot, 16);
        }
        self.sprite_apply_ricochet(slot);
    }

    pub fn sprite_check_tile_collision(&mut self, slot: usize) -> u8 {
        self.sprite_check_tile_collision2(slot);
        self.ram.sprite(slot).wall_collision()
    }

    pub fn sprite_stunned_main_ex(&mut self, slot: usize, second_entry: bool) {
        if !second_entry {
            self.sprite_draw_ripple_if_in_water(slot);
            self.sprite_stunned_main_func1(slot);
            if self.sprite_return_if_paused(slot) {
                return;
            }
            if self.ram.sprite(slot).scratch_f() != 0 {
                if (self.ram.sprite(slot).scratch_f() as i8) < 0 {
                    self.ram.set_sprite_scratch_f(slot, 0);
                }
                self.sprite_zero_velocity_xy(slot);
            }
            if self.ram.sprite(slot).main_delay() < 0x20 {
                self.sprite_check_damage_from_link(slot);
            }
            if self.sprite_return_if_recoiling(slot) {
                return;
            }
            self.sprite_move_xy(slot);
            if self.ram.sprite(slot).scratch_e() == 0 {
                self.sprite_check_tile_collision(slot);
                if self.ram.sprite(slot).state() == 0 {
                    return;
                }
            }
        }
        if self.ram.sprite(slot).wall_collision() & 0x0f != 0 {
            self.sprite_apply_ricochet(slot);
            if self.ram.sprite(slot).state() == 11 {
                self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 5);
            }
        }
        self.sprite_check_tile_property(slot, 0x68);
        let kind = usize::from(self.ram.sprite(slot).kind());
        if SPRITE_INIT_FLAGS3[kind] & 0x10 != 0 {
            self.ram
                .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() | 0x10);
            if self.r8(0xfa5) == 32 {
                self.ram
                    .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() & !0x10);
            }
        }
        self.sprite_move_z(slot);
        self.ram
            .set_sprite_velocity_z(slot, self.ram.sprite(slot).velocity_z().wrapping_sub(2));
        let z_test = self.ram.sprite(slot).z().wrapping_sub(1);
        if z_test >= 0xf0 {
            self.ram.set_sprite_z(slot, 0);
            if self.ram.sprite(slot).kind() == 0xe8
                && (self.ram.sprite(slot).velocity_z().wrapping_sub(0xe8) as i8) < 0
            {
                self.ram.set_sprite_state(slot, 6);
                self.ram.set_sprite_main_delay(slot, 8);
                self.ram.set_sprite_flags2(slot, 3);
                return;
            }
            self.throwable_scenery_transmute_if_valid(slot);
            let mut tile = self.r8(0xfa5);
            if tile == 32 {
                tile = self.ram.sprite(slot).flags() >> 1;
                if self.ram.sprite(slot).flags() & 1 == 0 {
                    self.sprite_func8(slot);
                    return;
                }
            }
            if tile == 9 {
                let velocity = self.ram.sprite(slot).velocity_z();
                self.ram.set_sprite_velocity_z(slot, 0);
                if (velocity.wrapping_sub(0xf0) as i8) < 0 {
                    let mut info = SpriteSpawnInfo::default();
                    let spawned = self.sprite_spawn_dynamically(slot, 0xec, &mut info);
                    if spawned >= 0 {
                        let spawned = spawned as usize;
                        self.sprite_set_spawned_coordinates(spawned, info);
                        self.sprite_func22(spawned);
                    }
                }
            } else if tile == 8 {
                if self.ram.sprite(slot).kind() == 0xd2
                    || self.hooks.random_u8(&mut self.ram) & 1 != 0
                {
                    self.sprite_spawn_leaping_fish(slot);
                }
                self.sprite_func22(slot);
                return;
            }
            let velocity = self.ram.sprite(slot).velocity_z();
            if (velocity as i8) < 0 {
                let bounce = (0u8).wrapping_sub(velocity) >> 1;
                self.ram
                    .set_sprite_velocity_z(slot, if bounce < 9 { 0 } else { bounce });
            }
            for field in [SpriteField::VelocityX, SpriteField::VelocityY] {
                let velocity = VelocityByte::from_bits(self.ram.sprite(slot).field(field))
                    .arithmetic_half()
                    .bits();
                self.ram
                    .set_sprite_field(slot, field, if velocity == 0xff { 0 } else { velocity });
            }
        }
        if (self.ram.sprite(slot).state() != 11 || self.ram.sprite(slot).unknown5() != 0)
            && !self.sprite_return_if_lifted(slot)
            && self.ram.sprite(slot).kind() != 0x4a
        {
            self.thrown_sprite_check_damage_to_sprites(slot);
        }
    }

    pub fn sprite_handle_dragging_by_ancilla(&mut self, slot: usize) -> bool {
        let encoded = self.ram.sprite(slot).scratch_b();
        if encoded == 0 {
            return false;
        }
        let ancilla = usize::from(encoded - 1);
        if self.arr(0xc4a, ancilla) == 0 {
            self.sprite_handle_absorption_by_player(slot);
        } else {
            self.ram.set_sprite_x_low(slot, self.arr(0xc04, ancilla));
            self.ram.set_sprite_x_high(slot, self.arr(0xc18, ancilla));
            self.ram.set_sprite_y_low(slot, self.arr(0xbfa, ancilla));
            self.ram.set_sprite_y_high(slot, self.arr(0xc0e, ancilla));
            self.ram.set_sprite_z(slot, 0);
        }
        true
    }

    pub fn sprite_return_if_phasing_out(&mut self, slot: usize) -> bool {
        if self.ram.sprite(slot).stunned() == 0 || self.r8(0x11) | self.r8(0xfc1) != 0 {
            return false;
        }
        if self.r8(0x1a) & 1 == 0 {
            self.ram
                .set_sprite_stunned(slot, self.ram.sprite(slot).stunned().wrapping_sub(1));
        }
        let timer = self.ram.sprite(slot).stunned();
        if timer == 0 {
            self.ram.set_sprite_state(slot, 0);
        } else if timer >= 0x28 || timer & 1 != 0 {
            return false;
        }
        let mut info = PrepOamCoordsRet::default();
        self.sprite_prep_oam_coord_or_double_ret(slot, &mut info);
        true
    }

    pub fn sprite_handle_absorption_by_player(&mut self, slot: usize) {
        self.ram.set_sprite_state(slot, 0);
        let item = self.ram.sprite(slot).kind().wrapping_sub(0xd8);
        self.hooks
            .sprite_sfx3_with_pan(&mut self.ram, slot, ABSORPTION_SFX[usize::from(item)]);
        match item {
            0 => self.w8(0xf372, self.r8(0xf372).wrapping_add(8)),
            1..=3 => self.w16(
                0xf360,
                self.r16(0xf360)
                    .wrapping_add(u16::from(RUPEES_ABSORPTION[usize::from(item - 1)])),
            ),
            4..=6 => self.w8(
                0xf375,
                self.r8(0xf375)
                    .wrapping_add(BOMBS_ABSORPTION[usize::from(item - 4)]),
            ),
            7 => self.w8(0xf373, self.r8(0xf373).wrapping_add(0x10)),
            8 => self.w8(0xf373, 0x80),
            9 => self.w8(
                0xf376,
                self.r8(0xf376)
                    .wrapping_add(if self.ram.sprite(slot).head_direction() == 0 {
                        5
                    } else {
                        self.ram.sprite(slot).head_direction()
                    }),
            ),
            10 => self.w8(0xf376, self.r8(0xf376).wrapping_add(10)),
            11 => {
                self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 0x31);
                self.w8(0xf372, self.r8(0xf372).wrapping_add(56));
            }
            12 | 13 => {
                if item == 12 {
                    self.w8(0xf36f, self.r8(0xf36f).wrapping_add(1));
                } else {
                    self.w8(0x2e9, 0);
                    SpriteHooks::link_receive_item(self.hooks, &mut self.ram, 0x32, 0);
                }
                self.ram
                    .set_sprite_scratch_n(slot, self.ram.sprite(slot).subtype());
                let flag =
                    u16::from(ABSORB_BIG_KEY[usize::from(self.ram.sprite(slot).death_action())])
                        << 8;
                self.w16(0x402, self.r16(0x402) | flag);
                self.sprite_manually_set_death_flag_uw(slot);
            }
            14 => {
                self.w8(0xf35a, self.ram.sprite(slot).subtype());
                if self.hooks.enhanced_features0() & 4096 != 0 {
                    self.hooks.palette_load_shield(&mut self.ram);
                }
            }
            _ => unreachable!("invalid absorbable sprite type"),
        }
    }

    pub fn sprite_check_absorption_by_player(&mut self, slot: usize) {
        if self.ram.sprite(slot).aux4_delay() == 0 && self.sprite_check_damage_to_player_1(slot) {
            self.sprite_handle_absorption_by_player(slot);
        }
    }

    pub fn sprite_draw_absorbable_transient(&mut self, slot: usize, transient: bool) -> bool {
        if transient && self.sprite_return_if_phasing_out(slot) {
            return false;
        }
        if self.r8(0xfb3) == 0 && self.r8(0x1b) != 0 {
            self.ram.set_sprite_object_priority(slot, 0x30);
        }
        if self.r8(0xfc6) >= 3 {
            return false;
        }
        if self.ram.sprite(slot).aux2_delay() != 0 {
            self.oam_allocate_from_region_c(12);
        }
        if self.ram.sprite(slot).scratch_e() != 0 {
            if self.hooks.enhanced_features0() & 4096 != 0 {
                self.ram.set_sprite_scratch_b(slot, 0);
            }
            return true;
        }
        let kind = self.ram.sprite(slot).kind();
        assert!((0xd8..0xd8 + 19).contains(&kind));
        let numbered = ABSORBABLE_TAB2[usize::from(kind - 0xd8)];
        if numbered != 0 {
            self.sprite_draw_numbered_absorbable(slot, numbered);
            return false;
        }
        match ABSORBABLE_TAB1[usize::from(kind - 0xd8)] {
            0 => self.sprite_draw_single_small(slot),
            2 if kind == 0xe6 && self.ram.sprite(slot).subtype() == 1 => {
                self.sprite_draw_thin_and_tall(slot)
            }
            2 => {
                if kind == 0xe6 {
                    self.ram.set_sprite_graphics(slot, 1);
                }
                self.sprite_draw_single_large(slot);
            }
            _ => self.sprite_draw_thin_and_tall(slot),
        }
        false
    }

    pub fn sprite_draw_numbered_absorbable(&mut self, slot: usize, number: u8) {
        let base = usize::from(number - 1) * 3;
        let mut info = PrepOamCoordsRet::default();
        if self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
            return;
        }
        let count = if self.ram.sprite(slot).head_direction() < 1 {
            3
        } else {
            2
        };
        let oam = self.current_oam_index();
        for output in 0..count {
            let i = count - 1 - output;
            let j = base + i;
            oam::set_helper0(
                &mut self.ram,
                oam + output,
                info.x.wrapping_add_signed(NUMBERED_ABSORBABLE_X[j]),
                info.y.wrapping_add_signed(NUMBERED_ABSORBABLE_Y[j]),
                NUMBERED_ABSORBABLE_CHAR[j],
                info.flags,
                NUMBERED_ABSORBABLE_EXT[j],
            );
        }
        self.sprite_draw_shadow(slot, &mut info);
    }

    pub fn prepare_apply_rumble_to_sprites(&mut self) {
        const X: [i8; 4] = [-32, -32, -32, 16];
        const Y: [i8; 4] = [-32, 32, -24, -24];
        const WH: [u8; 6] = [0x50, 0x50, 0x20, 0x20, 0x50, 0x50];
        let direction = usize::from(self.r8(0x2f) >> 1);
        let x = self.r16(0x22).wrapping_add(X[direction] as i16 as u16);
        let y = self.r16(0x20).wrapping_add(Y[direction] as i16 as u16);
        let mut hitbox = SpriteHitBox {
            r0_xlo: x as u8,
            r8_xhi: (x >> 8) as u8,
            r1_ylo: y as u8,
            r9_yhi: (y >> 8) as u8,
            r2: WH[direction],
            r3: WH[direction + 2],
            ..SpriteHitBox::default()
        };
        self.entity_apply_rumble_to_sprites(&mut hitbox);
    }

    pub fn entity_apply_rumble_to_sprites(&mut self, hitbox: &mut SpriteHitBox) {
        for slot in SpriteSlot::all().rev() {
            let slot = slot.index();
            if self.ram.sprite(slot).deflection_bits() & 2 == 0
                || self.ram.sprite(slot).scratch_e() == 0
            {
                continue;
            }
            if self.r8(0xfc6) != 0x0e {
                self.sprite_setup_hit_box(slot, hitbox);
                if !Self::check_if_hit_boxes_overlap(hitbox) {
                    continue;
                }
            }
            self.ram.set_sprite_scratch_e(slot, 0);
            self.w8(0x12f, 0x30);
            self.ram.set_sprite_velocity_z(slot, 0x30);
            self.ram.set_sprite_velocity_x(slot, 0x10);
            self.ram.set_sprite_aux3_delay(slot, 0x30);
            self.ram.set_sprite_stunned(slot, 0xff);
            if self.ram.sprite(slot).kind() == 0xd8 {
                self.hooks.sprite_transmute_to_bomb(&mut self.ram, slot);
            }
        }
    }

    pub fn sprite_explode_spawn_ea(&mut self, slot: usize) {
        let kind = self.ram.sprite(slot).kind();
        self.w8(0xfb5, kind);
        let mut info = SpriteSpawnInfo::default();
        let spawned = self.sprite_spawn_dynamically_ex(slot, 0xea, &mut info, 14);
        if spawned < 0 {
            return;
        }
        let spawned = spawned as usize;
        self.sprite_set_spawned_coordinates(spawned, info);
        self.ram.set_sprite_velocity_z(spawned, 32);
        self.ram.set_sprite_floor(spawned, self.r8(0xee));
        self.ram
            .set_sprite_scratch_a(spawned, if spawned == 9 { 2 } else { 6 });
        self.sprite_set_y(spawned, info.r2_y.wrapping_add(3));
        if kind == 0xce {
            self.sprite_set_y(spawned, info.r2_y.wrapping_add(16));
        } else if kind == 0xcb {
            self.ram.set_sprite_x_low(spawned, 0x78);
            self.ram.set_sprite_y_low(spawned, 0x78);
            self.ram
                .set_sprite_x_high(spawned, (self.r16(0x22) >> 8) as u8);
            self.ram
                .set_sprite_y_high(spawned, (self.r16(0x20) >> 8) as u8);
        }
    }

    pub fn sprite_kill_friends(&mut self) {
        let current = usize::from(self.r8(0xfa0));
        for slot in SpriteSlot::all().rev() {
            let slot = slot.index();
            if slot != current
                && self.ram.sprite(slot).state() != 0
                && self.ram.sprite(slot).deflection_bits() & 2 == 0
                && self.ram.sprite(slot).kind() != 0x7a
            {
                self.ram.set_sprite_state(slot, 6);
                self.ram.set_sprite_main_delay(slot, 15);
                self.ram.set_sprite_flags3(slot, 0);
                self.ram.set_sprite_flags5(slot, 0);
                self.ram.set_sprite_flags2(slot, 3);
            }
        }
    }

    pub fn release_fairy(&mut self) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let spawned = self.sprite_spawn_dynamically(0, 0xe3, &mut info);
        if spawned >= 0 {
            let slot = spawned as usize;
            self.ram.set_sprite_floor(slot, self.r8(0xee));
            self.sprite_set_x(slot, self.r16(0x22).wrapping_add(8));
            self.sprite_set_y(slot, self.r16(0x20).wrapping_add(16));
            self.ram.set_sprite_scratch_d(slot, 0);
            self.ram.set_sprite_aux4_delay(slot, 96);
        }
        spawned
    }

    pub fn sprite_draw_ripple_if_in_water(&mut self, slot: usize) {
        if !matches!(self.ram.sprite(slot).scratch_i(), 8 | 9) {
            return;
        }
        if self.ram.sprite(slot).flags3() & 0x20 != 0 {
            self.w16(0xfd8, self.r16(0xfd8).wrapping_sub(4));
            if self.ram.sprite(slot).kind() == 0xdf {
                self.w16(0xfda, self.r16(0xfda).wrapping_sub(7));
            }
        }
        self.hooks.sprite_draw_water_ripple(&mut self.ram, slot);
        self.sprite_get_16_bit_coords(slot);
        self.oam_allocate_from_region_a(
            (self.ram.sprite(slot).flags2() & 0x1f).wrapping_add(1) * 4,
        );
    }

    pub fn sprite_fall_draw(&mut self, slot: usize, info: &PrepOamCoordsRet) {
        const CHAR: [u8; 8] = [0x83, 0x83, 0x83, 0x80, 0x80, 0x80, 0xb7, 0xb7];
        let index = self.current_oam_index();
        let charnum = CHAR[usize::from(self.ram.sprite(slot).main_delay() >> 2)];
        oam::write(
            &mut self.ram,
            index,
            z3_core::c_types::OamEnt {
                x: info.x.wrapping_add(4) as u8,
                y: info.y.wrapping_add(4) as u8,
                charnum,
                flags: info.flags & 0x30 | 4,
            },
        );
        self.sprite_correct_oam_entries(slot, 0, 0);
    }

    pub fn sprite_module_fall1(&mut self, slot: usize) {
        if self.ram.sprite(slot).main_delay() == 0 {
            self.ram.set_sprite_state(slot, 0);
            self.sprite_manually_set_death_flag_uw(slot);
            return;
        }
        let mut info = PrepOamCoordsRet::default();
        if !self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
            self.sprite_fall_draw(slot, &info);
        }
    }

    pub fn sprite_module_fall2(&mut self, slot: usize) {
        let mut delay = self.ram.sprite(slot).main_delay();
        if delay == 0 {
            self.ram.set_sprite_state(slot, 0);
            self.sprite_manually_set_death_flag_uw(slot);
            return;
        }
        if delay >= 0x40 {
            if self.ram.sprite(slot).oam_flags() != 5 {
                if delay & 7 | self.r8(0x11) | self.r8(0xfc1) == 0 {
                    self.hooks.sprite_sfx3_with_pan(&mut self.ram, slot, 0x31);
                }
                self.run_sprite_active_main(slot);
                let mut info = PrepOamCoordsRet::default();
                if self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
                    return;
                }
                self.sprite_draw_distress_custom(
                    info.x,
                    info.y.wrapping_sub(8),
                    delay.wrapping_add(20),
                );
                return;
            }
            delay = 63;
            self.ram.set_sprite_main_delay(slot, delay);
        }
        if delay == 61 {
            self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 0x20);
        }
        let frame = usize::from(delay >> 1);
        if matches!(self.ram.sprite(slot).kind(), 0x26 | 0x13) {
            self.ram.set_sprite_graphics(slot, SPRITE_FALL_TAB2[frame]);
            self.sprite_draw_falling_helma_beetle(slot);
        } else {
            let mut graphics = SPRITE_FALL_TAB1[frame];
            if graphics < 12 {
                graphics = graphics.wrapping_add(
                    SPRITE_FALL_TAB4[usize::from(self.ram.sprite(slot).scratch_d())] as u8,
                );
            }
            self.ram.set_sprite_graphics(slot, graphics);
            self.sprite_draw_falling_humanoid(slot);
        }
        if self.r8(0x1a) & SPRITE_FALL_TAB3[usize::from(self.ram.sprite(slot).main_delay() >> 3)]
            | self.r8(0x11)
            != 0
        {
            return;
        }
        self.sprite_check_tile_property(slot, 0x68);
        if self.r8(0xfa5) != 0x20 {
            self.ram.set_sprite_recoil_y(slot, 0);
            self.ram.set_sprite_recoil_x(slot, 0);
        }
        self.ram
            .set_sprite_velocity_y(slot, ((self.ram.sprite(slot).recoil_y() as i8) >> 2) as u8);
        self.ram
            .set_sprite_velocity_x(slot, ((self.ram.sprite(slot).recoil_x() as i8) >> 2) as u8);
        self.sprite_move_xy(slot);
    }

    pub fn sprite_mini_moldorm_recoil(&mut self, slot: usize) {
        if self.ram.sprite(slot).state() < 9 {
            return;
        }
        let previous_state = self.ram.sprite(slot).state();
        self.w8(0xfb5, previous_state);
        let damage = self.ram.sprite(slot).damage();
        if damage == 253 {
            self.ram.set_sprite_damage(slot, 0);
            self.hooks.sprite_sfx3_with_pan(&mut self.ram, slot, 9);
            self.ram.set_sprite_state(slot, 7);
            self.ram.set_sprite_main_delay(slot, 0x70);
            self.ram
                .set_sprite_flags2(slot, self.ram.sprite(slot).flags2().wrapping_add(2));
            self.ram.set_sprite_damage(slot, 0);
            return;
        }
        if damage >= 251 {
            self.ram.set_sprite_damage(slot, 0);
            if previous_state == 11 {
                return;
            }
            let frozen = u8::from(damage == 254);
            self.ram.set_sprite_unknown5(slot, frozen);
            if frozen != 0 {
                self.ram
                    .set_sprite_deflection_bits(slot, self.ram.sprite(slot).deflection_bits() | 8);
                self.ram
                    .set_sprite_flags5(slot, self.ram.sprite(slot).flags5() & !0x80);
                self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 15);
                self.ram.set_sprite_velocity_z(slot, 24);
                self.ram
                    .set_sprite_bump_damage(slot, self.ram.sprite(slot).bump_damage() & !0x80);
                self.sprite_zero_velocity_xy(slot);
            }
            self.ram.set_sprite_state(slot, 11);
            self.ram.set_sprite_main_delay(slot, 64);
            const STUN: [u8; 5] = [0x20, 0x80, 0, 0, 0xff];
            self.ram
                .set_sprite_stunned(slot, STUN[usize::from(damage.wrapping_add(5))]);
            if self.ram.sprite(slot).kind() == 0x23 {
                self.ram.set_sprite_kind(slot, 0x24);
            }
            return;
        }
        let health = i16::from(self.ram.sprite(slot).health()) - i16::from(damage);
        self.ram.set_sprite_health(slot, health as u8);
        self.ram.set_sprite_damage(slot, 0);
        if health > 0 {
            return;
        }
        if self.ram.sprite(slot).death_action() == 0 {
            if previous_state == 11 {
                self.ram.set_sprite_death_action(slot, 3);
            }
            if self.ram.sprite(slot).unknown1() != 0 {
                self.ram.set_sprite_unknown1(slot, 0);
                self.ram.set_sprite_flags5(slot, 0);
            }
        }
        let kind = self.ram.sprite(slot).kind();
        if kind != 0x1b {
            self.hooks.sprite_sfx3_with_pan(&mut self.ram, slot, 9);
        }
        if kind == 0x40 {
            let address = 0xf280 + usize::from(self.r8(0x8a));
            self.w8(address, self.r8(address) | 0x40);
        } else if kind == 0xec {
            if self.ram.sprite(slot).scratch_c() == 2 {
                self.throwable_scenery_transmute_to_debris(slot);
            }
            return;
        }
        if previous_state == 10 {
            self.w8(0x308, 0);
            self.w8(0x309, 0);
        }
        self.ram.set_sprite_state(slot, 6);
        let mut common = false;
        let mut clear_sound = false;
        match kind {
            0x0c => self.sprite_func3(slot),
            0x92 => {
                self.sprite_kill_friends();
                self.ram.set_sprite_main_delay(slot, 0xff);
                common = true;
            }
            0xcb => {
                self.ram.set_sprite_ai_state(slot, 128);
                self.ram.set_sprite_main_delay(slot, 128);
                self.ram.set_sprite_state(slot, 9);
                common = true;
            }
            0xcc | 0xcd => {
                self.ram.set_sprite_ai_state(slot, 128);
                self.ram.set_sprite_main_delay(slot, 96);
                self.ram.set_sprite_state(slot, 9);
                common = true;
            }
            0x53 => {
                self.ram.set_sprite_main_delay(slot, 35);
                self.ram.set_sprite_hit_timer(slot, 0);
                clear_sound = true;
            }
            0x54 => {
                self.ram.set_sprite_ai_state(slot, 5);
                self.ram.set_sprite_main_delay(slot, 0xc0);
                self.ram.set_sprite_hit_timer(slot, 0xc0);
                common = true;
            }
            0x09 => {
                self.ram.set_sprite_ai_state(slot, 3);
                self.ram.set_sprite_aux4_delay(slot, 160);
                self.ram.set_sprite_state(slot, 9);
                common = true;
            }
            0x7a => {
                self.sprite_kill_friends();
                self.ram.set_sprite_state(slot, 9);
                self.ram.set_sprite_ignore_projectile(slot, 9);
                if self.r8(0xfff) == 0 {
                    self.ram.set_sprite_ai_state(slot, 10);
                    self.ram.set_sprite_main_delay(slot, 0xff);
                    self.ram.set_sprite_velocity_z(slot, 32);
                } else {
                    self.ram.set_sprite_main_delay(slot, 0xff);
                    self.ram.set_sprite_ai_state(slot, 8);
                    self.ram.set_sprite_ai_state(1, 9);
                    self.ram.set_sprite_ai_state(2, 9);
                    self.ram.set_sprite_graphics(1, 0);
                    self.ram.set_sprite_graphics(2, 0);
                }
                common = true;
            }
            0x23 if self.ram.sprite(slot).scratch_c() == 0 => {
                self.ram.set_sprite_ai_state(slot, 2);
                self.ram.set_sprite_main_delay(slot, 32);
                self.ram.set_sprite_state(slot, 9);
                self.ram.set_sprite_hit_timer(slot, 0);
            }
            0x0f => {
                self.ram.set_sprite_hit_timer(slot, 0);
                self.ram.set_sprite_main_delay(slot, 15);
            }
            _ if self.ram.sprite(slot).flags() & 2 == 0 => {
                self.ram.set_sprite_main_delay(
                    slot,
                    if self.ram.sprite(slot).hit_timer() & 0x80 != 0 {
                        31
                    } else {
                        15
                    },
                );
                self.ram
                    .set_sprite_flags2(slot, self.ram.sprite(slot).flags2().wrapping_add(4));
                if previous_state == 11 {
                    self.ram.set_sprite_flags5(slot, 1);
                }
            }
            _ => {
                if kind != 0xa2 {
                    self.sprite_kill_friends();
                }
                self.ram.set_sprite_state(slot, 4);
                self.ram.set_sprite_scratch_a(slot, 0);
                self.ram.set_sprite_main_delay(slot, 0xff);
                self.ram.set_sprite_hit_timer(slot, 0xff);
                common = true;
            }
        }
        if common {
            self.w8(0xffc, self.r8(0xffc).wrapping_add(1));
            clear_sound = true;
        }
        if clear_sound {
            self.w8(0x12f, 0);
            self.hooks.sprite_sfx3_with_pan(&mut self.ram, slot, 0x22);
        }
    }

    pub fn force_prize_drop(&mut self, slot: usize, prize: u8, sequence: usize) {
        let index = prize.wrapping_mul(8) | self.arr(0xfc7, sequence);
        self.set_arr(
            0xfc7,
            sequence,
            self.arr(0xfc7, sequence).wrapping_add(1) & 7,
        );
        self.prepare_enemy_drop(slot, PRIZE_ITEMS[usize::from(index)]);
    }

    pub fn prepare_enemy_drop(&mut self, slot: usize, item: u8) {
        self.ram.set_sprite_kind(slot, item);
        if item == 0xe5 {
            self.hooks
                .sprite_prep_big_key_load_graphics(&mut self.ram, slot);
        } else if item == 0xe4 {
            self.hooks
                .sprite_prep_key_set_item_drop(&mut self.ram, slot);
        }
        self.ram.set_sprite_state(slot, 9);
        let saved_z = self.ram.sprite(slot).z();
        self.sprite_prep_load_properties(slot);
        self.ram.set_sprite_ignore_projectile(
            slot,
            self.ram.sprite(slot).ignore_projectile().wrapping_add(1),
        );
        let prize_index = usize::from(self.ram.sprite(slot).kind().wrapping_sub(0xd8));
        let z = if prize_index < PRIZE_Z.len() {
            PRIZE_Z[prize_index]
        } else if prize_index < PRIZE_Z.len() + PERISH_OVERLAY_X.len() {
            PERISH_OVERLAY_X[prize_index - PRIZE_Z.len()] as u8
        } else {
            // Preserve the C port's intentional out-of-bounds table read used
            // by the pulverized Buzz Blob path. These arrays are adjacent.
            PERISH_OVERLAY_Y[prize_index - PRIZE_Z.len() - PERISH_OVERLAY_X.len()] as u8
        };
        self.ram.set_sprite_velocity_z(slot, z & 0xf0);
        self.sprite_set_x(
            slot,
            self.sprite_get_x(slot).wrapping_add(u16::from(z & 0x0f)),
        );
        self.ram.set_sprite_z(slot, saved_z);
        self.ram.set_sprite_aux4_delay(slot, 21);
        self.ram.set_sprite_stunned(slot, 0xff);
        self.sprite_death_func4(slot);
    }

    pub fn sprite_death_func4(&mut self, slot: usize) {
        if self.ram.sprite(slot).kind() == 0xa2 && self.sprite_check_if_screen_is_clear() {
            self.hooks.ancilla_spawn_falling_prize(&mut self.ram, 4);
        }
        self.sprite_manually_set_death_flag_uw(slot);
        self.w8(0xcfb, self.r8(0xcfb).wrapping_add(1));
        if self.ram.sprite(slot).kind() == 0x40 {
            self.ram.set_sprite_state(slot, 9);
            self.ram.set_sprite_graphics(slot, 4);
            // The second-entry death animation is invoked by SpriteDeath_MainEx.
            self.sprite_death_main_ex(slot, true);
        }
    }

    pub fn sprite_do_the_death(&mut self, slot: usize) {
        let kind = self.ram.sprite(slot).kind();
        if kind == 0xbe {
            self.ram
                .set_sprite_scratch_g(0, self.ram.sprite(0).scratch_g().wrapping_sub(1));
        }
        if kind == 0xaa && self.ram.sprite(slot).scratch_e() != 0 {
            let subtype = self.ram.sprite(slot).subtype();
            self.prepare_enemy_drop(
                slot,
                PIKIT_DROP_ITEMS[usize::from(self.ram.sprite(slot).scratch_e().wrapping_sub(1))],
            );
            self.ram.set_sprite_subtype(slot, subtype);
            if subtype == 1 {
                self.ram.set_sprite_oam_flags(slot, 9);
                self.ram.set_sprite_flags3(slot, 0xf0);
            }
            self.ram.set_sprite_head_direction(
                slot,
                self.ram.sprite(slot).head_direction().wrapping_add(1),
            );
            return;
        }
        if kind == 0x45 && self.r8(0xf3c5) == 2 && self.r8(0x40a) == 0x18 {
            self.w8(0x12c, 7);
        }
        let drop = self.ram.sprite(slot).death_action();
        if drop != 0 {
            self.ram
                .set_sprite_subtype(slot, self.ram.sprite(slot).scratch_n());
            self.ram.set_sprite_scratch_n(slot, 0xff);
            self.prepare_enemy_drop(
                slot,
                if drop == 1 {
                    0xe4
                } else if drop == 3 {
                    0xd9
                } else {
                    0xe5
                },
            );
            return;
        }
        let raw_prize = self.ram.sprite(slot).flags5() & 0x0f;
        if raw_prize != 0 {
            let prize = raw_prize.wrapping_sub(1);
            let luck = self.r8(0xcf9);
            if luck != 0 {
                let counter = self.r8(0xcfa).wrapping_add(1);
                self.w8(0xcfa, counter);
                if counter >= 10 {
                    self.w8(0xcf9, 0);
                }
                if luck == 1 {
                    self.force_prize_drop(slot, prize, 1);
                    return;
                }
            } else if self.hooks.random_u8(&mut self.ram) & PRIZE_MASKS[usize::from(prize)] == 0 {
                self.force_prize_drop(slot, prize, usize::from(prize));
                return;
            }
        }
        self.ram.set_sprite_state(slot, 0);
        self.sprite_death_func4(slot);
    }

    pub fn sprite_death_draw_poof(&mut self, slot: usize) {
        if self.r8(0x46c) == 4 {
            self.ram.set_sprite_object_priority(slot, 0x30);
        }
        let mut info = PrepOamCoordsRet::default();
        if self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
            return;
        }
        let offset = (self.ram.sprite(slot).flags3() & 0x20) >> 3;
        let mut frame = usize::from((self.ram.sprite(slot).main_delay() & 0x1c) ^ 0x1c) + 3;
        let first = self.current_oam_index();
        for output in 0..4 {
            if PERISH_OVERLAY_CHAR[frame] != 0 {
                self.set_oam_plain(
                    first + output,
                    u16::from(
                        self.r8(0xfa8)
                            .wrapping_sub(offset)
                            .wrapping_add(PERISH_OVERLAY_X[frame] as u8),
                    ),
                    u16::from(
                        self.r8(0xfa9)
                            .wrapping_sub(offset)
                            .wrapping_add(PERISH_OVERLAY_Y[frame] as u8),
                    ),
                    PERISH_OVERLAY_CHAR[frame],
                    info.flags & 0x30 | PERISH_OVERLAY_FLAGS[frame],
                    self.r8(oam::BYTEWISE_EXTENDED_OAM + first + output),
                );
            }
            frame -= 1;
        }
        self.sprite_correct_oam_entries(slot, 3, 0);
    }

    pub fn sprite_death_main_ex(&mut self, slot: usize, second_entry: bool) {
        if !second_entry {
            let kind = self.ram.sprite(slot).kind();
            if kind == 0xec {
                self.hooks
                    .throwable_scenery_scatter_into_debris(&mut self.ram, slot);
                return;
            }
            if matches!(kind, 0x53 | 0x54 | 0x92)
                || kind == 0x4a && self.ram.sprite(slot).scratch_c() >= 2
            {
                self.run_sprite_active_main(slot);
                return;
            }
            if self.ram.sprite(slot).main_delay() == 0 {
                self.sprite_do_the_death(slot);
                return;
            }
        }
        if (self.ram.sprite(slot).flags3() as i8) < 0 {
            self.run_sprite_active_main(slot);
            return;
        }
        if (self.r8(0x1a) & 3) | self.r8(0x11) | self.r8(0xfc1) == 0 {
            self.ram
                .set_sprite_main_delay(slot, self.ram.sprite(slot).main_delay().wrapping_add(1));
        }
        self.sprite_death_draw_poof(slot);
        if self.ram.sprite(slot).kind() != 0x40 && self.ram.sprite(slot).main_delay() < 10 {
            return;
        }
        oam::OamPointers::advance_entries(&mut self.ram, 4);
        let flags = self.ram.sprite(slot).flags2();
        self.ram.set_sprite_flags2(slot, flags.wrapping_sub(4));
        self.run_sprite_active_main(slot);
        self.ram.set_sprite_flags2(slot, flags);
    }

    pub fn sprite_module_die(&mut self, slot: usize) {
        self.sprite_death_main_ex(slot, false);
    }

    pub fn sprite_module_burn(&mut self, slot: usize) {
        const FLAME_GRAPHICS: [u8; 32] = [
            5, 4, 3, 1, 2, 0, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1,
            2, 3, 0,
        ];
        self.ram.set_sprite_hit_timer(slot, 0);
        let frame = self.ram.sprite(slot).main_delay().wrapping_sub(1);
        if frame == 0 {
            self.sprite_do_the_death(slot);
            return;
        }
        let graphics = self.ram.sprite(slot).graphics();
        let oam_flags = self.ram.sprite(slot).oam_flags();
        self.ram
            .set_sprite_graphics(slot, FLAME_GRAPHICS[usize::from(frame >> 3)]);
        self.ram.set_sprite_oam_flags(slot, 3);
        self.hooks.flame_draw(&mut self.ram, slot);
        self.ram.set_sprite_oam_flags(slot, oam_flags);
        self.ram.set_sprite_graphics(slot, graphics);
        oam::OamPointers::advance_entries(&mut self.ram, 2);
        if self.ram.sprite(slot).main_delay() >= 0x10 {
            let flags = self.ram.sprite(slot).flags2();
            self.ram.set_sprite_flags2(slot, flags.wrapping_sub(2));
            self.run_sprite_active_main(slot);
            self.ram.set_sprite_flags2(slot, flags);
        }
    }

    pub fn sprite_draw_multiple(
        &mut self,
        slot: usize,
        source: &[DrawMultipleData],
        result: Option<&mut PrepOamCoordsRet>,
    ) {
        let mut local = PrepOamCoordsRet::default();
        let info = result.unwrap_or(&mut local);
        if self.sprite_prep_oam_coord_or_double_ret(slot, info) {
            return;
        }
        self.w16(0xcfe, 0);
        let mut state = self.ram.sprite(slot).state();
        if state == 10 {
            state = self.ram.sprite(slot).unknown4();
        }
        if state == 11 {
            self.w8(0xcfe, self.ram.sprite(slot).unknown5());
        }
        let first = self.current_oam_index();
        for (offset, entry) in source.iter().enumerate() {
            let mut data = entry.char_flags ^ (u16::from(info.flags) << 8 | u16::from(info.r4));
            if self.r16(0xcfe) >= 1 {
                data = data & !0x0e00 | 0x0400;
            }
            oam::set_helper0(
                &mut self.ram,
                first + offset,
                info.x.wrapping_add(entry.x as i16 as u16),
                info.y.wrapping_add(entry.y as i16 as u16),
                data as u8,
                (data >> 8) as u8,
                entry.ext,
            );
        }
    }

    pub fn sprite_draw_multiple_player_deferred(
        &mut self,
        slot: usize,
        source: &[DrawMultipleData],
        result: Option<&mut PrepOamCoordsRet>,
    ) {
        self.oam_allocate_defer_to_player(slot);
        self.sprite_draw_multiple(slot, source, result);
    }

    pub fn sprite_prep_and_draw_single_large_no_prep(
        &mut self,
        slot: usize,
        info: &PrepOamCoordsRet,
    ) {
        let index = self.current_oam_index();
        let mut entry = oam::read(&self.ram, index);
        entry.x = info.x as u8;
        if info.y.wrapping_add(0x10) < 0x100 {
            entry.y = info.y as u8;
            let table_index = usize::from(
                SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1
                    .get(usize::from(self.ram.sprite(slot).kind()))
                    .copied()
                    .unwrap_or_else(|| {
                        SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2[usize::from(
                            self.ram.sprite(slot).kind(),
                        )
                            - SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1.len()]
                    })
                    .wrapping_add(self.ram.sprite(slot).graphics()),
            );
            entry.charnum = SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2[table_index];
            entry.flags = info.flags;
        }
        oam::write(&mut self.ram, index, entry);
        self.w8(
            oam::BYTEWISE_EXTENDED_OAM + index,
            2 | u8::from(info.x >= 256),
        );
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            let mut shadow_info = *info;
            self.sprite_draw_shadow(slot, &mut shadow_info);
        }
    }

    pub fn sprite_draw_single_large(&mut self, slot: usize) {
        let mut info = PrepOamCoordsRet::default();
        if !self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
            self.sprite_prep_and_draw_single_large_no_prep(slot, &info);
        }
    }

    pub fn sprite_draw_shadow_custom(
        &mut self,
        slot: usize,
        info: &mut PrepOamCoordsRet,
        amount: u8,
    ) {
        let world_y = self.sprite_get_y(slot).wrapping_add(u16::from(amount));
        info.y = world_y;
        if self.ram.sprite(slot).pause() != 0
            || self.ram.sprite(slot).state() == 10 && self.ram.sprite(slot).unknown3() == 3
        {
            return;
        }
        let y = world_y.wrapping_sub(self.r16(0xe8));
        info.y = y;
        if y.wrapping_add(0x10) >= 0x100 {
            return;
        }
        let index = self.current_oam_index() + usize::from(self.ram.sprite(slot).flags2() & 0x1f);
        if self.ram.sprite(slot).flags3() & 0x20 != 0 {
            oam::set_helper1(
                &mut self.ram,
                index,
                info.x,
                y.wrapping_add(1) as u8,
                0x38,
                info.flags & 0x30 | 8,
                0,
            );
        } else {
            oam::set_helper1(
                &mut self.ram,
                index,
                info.x,
                y as u8,
                0x6c,
                info.flags & 0x30 | 8,
                2,
            );
        }
    }

    pub fn sprite_draw_shadow(&mut self, slot: usize, info: &mut PrepOamCoordsRet) {
        self.sprite_draw_shadow_custom(slot, info, 10);
    }

    pub fn sprite_draw_single_small(&mut self, slot: usize) {
        let mut info = PrepOamCoordsRet::default();
        if self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
            return;
        }
        let index = self.current_oam_index();
        let mut entry = oam::read(&self.ram, index);
        entry.x = info.x as u8;
        if info.y.wrapping_add(0x10) < 0x100 {
            entry.y = info.y as u8;
            let table_index = usize::from(
                SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1
                    .get(usize::from(self.ram.sprite(slot).kind()))
                    .copied()
                    .unwrap_or_else(|| {
                        SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2[usize::from(
                            self.ram.sprite(slot).kind(),
                        )
                            - SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1.len()]
                    })
                    .wrapping_add(self.ram.sprite(slot).graphics()),
            );
            entry.charnum = SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2[table_index];
            entry.flags = info.flags;
        }
        oam::write(&mut self.ram, index, entry);
        self.w8(oam::BYTEWISE_EXTENDED_OAM + index, u8::from(info.x >= 256));
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            self.sprite_draw_shadow_custom(slot, &mut info, 2);
        }
    }

    pub fn sprite_draw_thin_and_tall(&mut self, slot: usize) {
        let mut info = PrepOamCoordsRet::default();
        if self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
            return;
        }
        let table_index = usize::from(
            SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1
                .get(usize::from(self.ram.sprite(slot).kind()))
                .copied()
                .unwrap_or_else(|| {
                    SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2[usize::from(
                        self.ram.sprite(slot).kind(),
                    )
                        - SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1.len()]
                })
                .wrapping_add(self.ram.sprite(slot).graphics()),
        );
        let charnum = SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2[table_index];
        let index = self.current_oam_index();
        oam::set_helper0(&mut self.ram, index, info.x, info.y, charnum, info.flags, 0);
        oam::set_helper0(
            &mut self.ram,
            index + 1,
            info.x,
            info.y.wrapping_add(8),
            charnum.wrapping_add(0x10),
            info.flags,
            0,
        );
        if self.ram.sprite(slot).flags3() & 0x10 != 0 {
            self.sprite_draw_shadow(slot, &mut info);
        }
    }

    pub fn sprite_module_explode(&mut self, slot: usize) {
        const DRAW: [DrawMultipleData; 32] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0060,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0060,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0060,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0060,
                ext: 2,
            },
            DrawMultipleData {
                x: -5,
                y: -5,
                char_flags: 0x0062,
                ext: 2,
            },
            DrawMultipleData {
                x: 5,
                y: -5,
                char_flags: 0x4062,
                ext: 2,
            },
            DrawMultipleData {
                x: -5,
                y: 5,
                char_flags: 0x8062,
                ext: 2,
            },
            DrawMultipleData {
                x: 5,
                y: 5,
                char_flags: 0xc062,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x0062,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x4062,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x8062,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc062,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x0064,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x4064,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x8064,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc064,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x0066,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x4066,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x8066,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc066,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x0068,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x0068,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x0068,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0x0068,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x006a,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x406a,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x806a,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc06a,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x004e,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x404e,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x804e,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc04e,
                ext: 2,
            },
        ];
        if self.ram.sprite(slot).scratch_a() != 0 {
            if self.ram.sprite(slot).main_delay() == 0 {
                self.ram.set_sprite_state(slot, 0);
                if (0..16)
                    .rev()
                    .any(|other| self.ram.sprite(other).state() == 4)
                {
                    return;
                }
                self.w8(0xaaa, 1);
                if !self.sprite_check_if_screen_is_clear() {
                    self.w8(0xffc, 0);
                }
            } else {
                let frame = usize::from((self.ram.sprite(slot).main_delay() >> 2) ^ 7) * 4;
                self.sprite_draw_multiple(slot, &DRAW[frame..frame + 4], None);
            }
            return;
        }
        self.ram.set_sprite_floor(slot, 2);
        let delay = self.ram.sprite(slot).main_delay();
        if delay == 32 {
            self.ram.set_sprite_state(slot, 0);
            self.w8(0x2e4, 0);
            if self.r8(0x5b) != 2 && self.sprite_check_if_screen_is_clear() {
                let kind = self.ram.sprite(slot).kind();
                if kind >= 0xd6 {
                    self.w8(0x12c, 0x13);
                } else if kind == 0x7a {
                    self.hooks
                        .prepare_dungeon_exit_from_boss_fight(&mut self.ram);
                } else {
                    self.sprite_explode_spawn_ea(slot);
                    return;
                }
            }
        }
        if delay >= 64 && (delay >= 0x70 || delay & 1 == 0) {
            self.run_sprite_active_main(slot);
        }
        let kind = self.ram.sprite(slot).kind();
        if delay >= 0xc0 {
            return;
        }
        if delay & 3 == 0 {
            self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 0x0c);
        }
        if delay & if kind == 0x92 { 3 } else { 7 } != 0 {
            return;
        }
        let mut info = SpriteSpawnInfo::default();
        let spawned = self.sprite_spawn_dynamically(slot, 0x1c, &mut info);
        if spawned < 0 {
            return;
        }
        let spawned = spawned as usize;
        self.w8(0xaaa, 11);
        self.ram.set_sprite_state(spawned, 4);
        self.ram.set_sprite_flags2(spawned, 3);
        self.ram.set_sprite_oam_flags(spawned, 0x0c);
        const RANDOM_XY: [i8; 16] = [0, 4, 8, 12, -4, -8, -12, 0, 0, 8, 16, 24, -24, -16, -8, 0];
        let bank = if kind == 0x92 { 8 } else { 0 };
        let x = RANDOM_XY[usize::from(self.hooks.random_u8(&mut self.ram) & 7) | bank];
        let y = RANDOM_XY[usize::from(self.hooks.random_u8(&mut self.ram) & 7) | bank];
        self.sprite_set_x(spawned, info.r0_x.wrapping_add(x as i16 as u16));
        self.sprite_set_y(
            spawned,
            info.r2_y
                .wrapping_add(y as i16 as u16)
                .wrapping_sub(u16::from(info.r4_z)),
        );
        self.ram.set_sprite_main_delay(spawned, 31);
        self.ram.set_sprite_scratch_a(spawned, 31);
    }

    pub fn sprite_module_drown(&mut self, slot: usize) {
        const DRAW: [DrawMultipleData; 8] = [
            DrawMultipleData {
                x: -7,
                y: -7,
                char_flags: 0x0480,
                ext: 0,
            },
            DrawMultipleData {
                x: 14,
                y: -6,
                char_flags: 0x0483,
                ext: 0,
            },
            DrawMultipleData {
                x: -6,
                y: -6,
                char_flags: 0x04cf,
                ext: 0,
            },
            DrawMultipleData {
                x: 13,
                y: -5,
                char_flags: 0x04df,
                ext: 0,
            },
            DrawMultipleData {
                x: -4,
                y: -4,
                char_flags: 0x04ae,
                ext: 0,
            },
            DrawMultipleData {
                x: 12,
                y: -4,
                char_flags: 0x44af,
                ext: 0,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x04e7,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x04e7,
                ext: 2,
            },
        ];
        const FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        const CHAR: [u8; 11] = [
            0xc0, 0xc0, 0xc0, 0xc0, 0xcd, 0xcd, 0xcd, 0xcb, 0xcb, 0xcb, 0xcb,
        ];
        if self.ram.sprite(slot).ai_state() != 0 {
            if self.ram.sprite(slot).scratch_a() == 6 {
                self.oam_allocate_from_region_c(8);
            }
            self.ram
                .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() ^ 16);
            self.sprite_draw_single_large(slot);
            let index = self.current_oam_index();
            let delay = self.ram.sprite(slot).main_delay();
            if delay == 1 {
                self.ram.set_sprite_state(slot, 0);
            }
            let mut entry = oam::read(&self.ram, index);
            if delay != 0 {
                entry.charnum = CHAR[usize::from(delay >> 1)];
                entry.flags = 0x24;
                oam::write(&mut self.ram, index, entry);
                return;
            }
            entry.charnum = 0x8a;
            entry.flags = FLAGS[usize::from((self.ram.sprite(slot).subtype2() >> 2) & 3)] | 0x24;
            oam::write(&mut self.ram, index, entry);
            if self.sprite_return_if_paused(slot) {
                return;
            }
            self.ram
                .set_sprite_subtype2(slot, self.ram.sprite(slot).subtype2().wrapping_add(1));
            self.sprite_move_xy(slot);
            self.sprite_move_z(slot);
            self.ram
                .set_sprite_velocity_z(slot, self.ram.sprite(slot).velocity_z().wrapping_sub(2));
            if (self.ram.sprite(slot).z() as i8) < 0 {
                self.ram.set_sprite_z(slot, 0);
                self.ram.set_sprite_main_delay(slot, 18);
                self.ram
                    .set_sprite_flags3(slot, self.ram.sprite(slot).flags3() & !0x10);
            }
        } else {
            if self.sprite_return_if_paused(slot) {
                return;
            }
            if self.r8(0x1a) & 1 == 0 {
                self.ram.set_sprite_main_delay(
                    slot,
                    self.ram.sprite(slot).main_delay().wrapping_add(1),
                );
            }
            self.ram.set_sprite_oam_flags(slot, 0);
            self.ram.set_sprite_hit_timer(slot, 0);
            if self.ram.sprite(slot).main_delay() == 0 {
                self.ram.set_sprite_state(slot, 0);
            }
            let frame = usize::from((self.ram.sprite(slot).main_delay() << 1 & 0xf8) >> 2);
            self.sprite_draw_multiple(slot, &DRAW[frame..frame + 2], None);
        }
    }

    pub fn sprite_module_poof(&mut self, slot: usize) {
        const X: [i8; 16] = [-6, 10, 1, 13, -6, 10, 1, 13, -7, 4, -5, 6, -1, 1, -2, 0];
        const Y: [i8; 16] = [-6, -4, 10, 9, -6, -4, 10, 9, -8, -10, 4, 3, -1, -2, 0, 1];
        const CHAR: [u8; 16] = [
            0x9b, 0x9b, 0x9b, 0x9b, 0xb3, 0xb3, 0xb3, 0xb3, 0x8a, 0x8a, 0x8a, 0x8a, 0x8a, 0x8a,
            0x8a, 0x8a,
        ];
        const FLAGS: [u8; 16] = [
            0x24, 0xa4, 0x24, 0xa4, 0xe4, 0x64, 0xa4, 0x24, 0x24, 0xe4, 0xe4, 0xe4, 0x24, 0xe4,
            0xe4, 0xe4,
        ];
        const EXT: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];
        if self.ram.sprite(slot).main_delay() == 0 {
            if self.ram.sprite(slot).kind() == 0x0d && self.ram.sprite(slot).head_direction() != 0 {
                let x = self.sprite_get_x(slot);
                self.prepare_enemy_drop(slot, 0x0d);
                self.sprite_set_x(slot, x);
                self.ram.set_sprite_velocity_z(slot, 0);
                self.ram.set_sprite_ignore_projectile(slot, 0);
            } else if self.ram.sprite(slot).death_action() == 0 {
                self.force_prize_drop(slot, 2, 2);
            } else {
                self.sprite_do_the_death(slot);
            }
            return;
        }
        let mut info = PrepOamCoordsRet::default();
        if self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
            return;
        }
        let mut frame = usize::from((self.ram.sprite(slot).main_delay() >> 1) & !3) + 3;
        let first = self.current_oam_index();
        for output in 0..4 {
            self.set_oam_plain(
                first + output,
                u16::from(self.r8(0xfa8)).wrapping_add(X[frame] as i16 as u16),
                u16::from(self.r8(0xfa9)).wrapping_add(Y[frame] as i16 as u16),
                CHAR[frame],
                FLAGS[frame],
                EXT[frame],
            );
            frame -= 1;
        }
        self.sprite_correct_oam_entries(slot, 3, 0xff);
    }

    pub fn sprite_draw_distress_custom(&mut self, x: u16, y: u16, time: u8) {
        self.oam_allocate_from_region_a(0x10);
        if time & 0x18 == 0 {
            return;
        }
        let first = self.current_oam_index();
        for output in 0..4 {
            let frame = 3 - output;
            oam::set_helper0(
                &mut self.ram,
                first + output,
                x.wrapping_add(SPRITE_DISTRESS_X[frame] as i16 as u16),
                y.wrapping_add(SPRITE_DISTRESS_Y[frame] as i16 as u16),
                0x83,
                0x22,
                0,
            );
        }
    }

    pub fn sprite_draw_falling_helma_beetle(&mut self, slot: usize) {
        const DATA: [DrawMultipleData; 12] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0146,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0148,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x014a,
                ext: 2,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x014c,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x00b7,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x0080,
                ext: 0,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x016c,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x016e,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x014e,
                ext: 2,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x015c,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x00b7,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x0080,
                ext: 0,
            },
        ];
        let mut index = usize::from(self.ram.sprite(slot).graphics());
        if self.ram.sprite(slot).kind() == 0x13 {
            index += 6;
        }
        let mut info = PrepOamCoordsRet::default();
        self.sprite_draw_multiple(slot, &DATA[index..index + 1], Some(&mut info));
    }

    pub fn sprite_draw_falling_humanoid(&mut self, slot: usize) {
        let mut info = PrepOamCoordsRet::default();
        if self.sprite_prep_oam_coord_or_double_ret(slot, &mut info) {
            return;
        }
        let graphics = usize::from(self.ram.sprite(slot).graphics());
        let count = if graphics < 12 && graphics & 3 == 0 {
            4
        } else {
            1
        };
        let first = self.current_oam_index();
        for output in 0..count {
            let part = count - 1 - output;
            let frame = graphics * 4 + part;
            self.set_oam_plain(
                first + output,
                info.x
                    .wrapping_add(SPRITE_DRAW_FALL1_X[frame] as i16 as u16),
                info.y
                    .wrapping_add(SPRITE_DRAW_FALL1_Y[frame] as i16 as u16),
                SPRITE_DRAW_FALL1_CHAR[frame],
                info.flags ^ SPRITE_DRAW_FALL1_FLAGS[frame],
                SPRITE_DRAW_FALL1_EXT[frame],
            );
        }
        self.sprite_correct_oam_entries(slot, count - 1, 0xff);
    }

    pub fn sprite_correct_oam_entries(&mut self, slot: usize, count: usize, is_large: u8) {
        let first = self.current_oam_index();
        let first_ext = usize::from(self.r16(0x92));
        let sprite_x = self.sprite_get_x(slot);
        let sprite_y = self.sprite_get_y(slot);
        let scroll_x = sprite_x.wrapping_sub(self.r16(0xe2)) as u8;
        let scroll_y = sprite_y.wrapping_sub(self.r16(0xe8)) as u8;
        for offset in 0..=count {
            let index = first + offset;
            let mut entry = oam::read(&self.ram, index);
            let x = sprite_x.wrapping_add((entry.x.wrapping_sub(scroll_x) as i8) as i16 as u16);
            let y = sprite_y.wrapping_add((entry.y.wrapping_sub(scroll_y) as i8) as i16 as u16);
            let ext_address = first_ext + offset;
            let ext = if (is_large as i8) < 0 {
                self.r8(ext_address) & 2
            } else {
                is_large
            };
            self.w8(
                ext_address,
                ext.wrapping_add(u8::from(x.wrapping_sub(self.r16(0xe2)) >= 0x100)),
            );
            if y.wrapping_add(0x10).wrapping_sub(self.r16(0xe8)) >= 0x100 {
                entry.y = 0xf0;
                oam::write(&mut self.ram, index, entry);
            }
        }
    }

    pub fn overworld_substitute_alternate_secret(&mut self) {
        const AREA_MASKS: [u8; 64] = [
            0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 6, 4, 4, 6, 0, 0, 15, 15, 4, 5,
            5, 4, 6, 6, 15, 15, 4, 5, 5, 7, 6, 6, 31, 31, 4, 7, 7, 4, 6, 6, 6, 7, 2, 0, 0, 0, 0, 0,
            6, 6, 2, 0, 0, 0, 0, 0,
        ];
        const SECRET_VALUES: [u8; 16] = [1, 1, 1, 1, 15, 1, 1, 18, 16, 1, 1, 1, 17, 1, 1, 3];
        const SLOT_MASKS: [u8; 16] = [0, 0, 0, 0, 2, 0, 0, 8, 16, 0, 0, 0, 1, 0, 0, 0];
        if self.hooks.random_u8(&mut self.ram) & 1 != 0 {
            return;
        }
        let active = (0..16)
            .filter(|&slot| {
                self.ram.sprite(slot).state() != 0 && self.ram.sprite(slot).kind() != 0x6c
            })
            .count();
        if active >= 4 || self.r8(0xf3c5) < 2 {
            return;
        }
        let counter = self.r8(0xcf7);
        self.w8(0xcf7, counter.wrapping_add(1));
        let index = usize::from((counter & 7) + if self.r8(0xfff) != 0 { 8 } else { 0 });
        if AREA_MASKS[usize::from(self.r8(0x40a) & 0x3f)] & SLOT_MASKS[index] == 0 {
            self.w8(0xb9c, SECRET_VALUES[index]);
        }
    }
    pub fn garnish_return_if_prep_fails(&mut self, slot: usize) -> Option<(u16, u16)> {
        let x = self.garnish_get_x(slot).wrapping_sub(self.r16(0xe2));
        let y = self.garnish_get_y(slot).wrapping_sub(self.r16(0xe8));
        if x >= 256 || y >= 256 {
            self.ram.set_garnish_kind(slot, 0);
            None
        } else {
            Some((x, y.wrapping_sub(16)))
        }
    }

    pub fn garnish_check_player_collision(&mut self, slot: usize, x: u8, y: u8) {
        if ((slot as u8 ^ self.r8(0x1a)) & 7) != 0 || self.r8(0x31f) != 0 || self.r8(0x37b) != 0 {
            return;
        }
        let link_x = self.r16(0x22).wrapping_sub(self.r16(0xe2)) as u8;
        let link_y = self.r16(0x20).wrapping_sub(self.r16(0xe8)) as u8;
        if link_x.wrapping_sub(x).wrapping_add(12) < 24
            && link_y.wrapping_sub(y).wrapping_add(22) < 28
        {
            self.w8(0x4d, 1);
            self.w8(0x46, 16);
            self.w8(0x373, 16);
            self.w8(0x28, self.r8(0x28) ^ 0xff);
            self.w8(0x27, self.r8(0x27) ^ 0xff);
        }
    }

    pub fn sprite_bounce_from_tile_collision(&mut self, slot: usize) -> u8 {
        let collision = self.sprite_check_tile_collision(slot);
        if collision & 3 != 0 {
            self.ram.set_sprite_velocity_x(
                slot,
                (0u8).wrapping_sub(self.ram.sprite(slot).velocity_x()),
            );
            self.ram
                .set_sprite_scratch_g(slot, self.ram.sprite(slot).scratch_g().wrapping_add(1));
        }
        if collision & 12 != 0 {
            self.ram.set_sprite_velocity_y(
                slot,
                (0u8).wrapping_sub(self.ram.sprite(slot).velocity_y()),
            );
            self.ram
                .set_sprite_scratch_g(slot, self.ram.sprite(slot).scratch_g().wrapping_add(1));
            return self.ram.sprite(slot).scratch_g();
        }
        0
    }

    pub fn get_tile_attribute(&mut self, floor: u8, x: &mut u16, y: u16) -> u8 {
        let tile = if self.r8(0x1b) != 0 {
            let mut index = if floor >= 1 { 0x1000 } else { 0 };
            index += usize::from((*x & 0x1f8) >> 3);
            index += usize::from((y & 0x1f8) << 3);
            self.r8(0x12000 + index)
        } else {
            *x >>= 3;
            self.hooks.tile_attribute(&mut self.ram, floor, x, y)
        };
        self.w8(0xfa5, tile);
        tile
    }

    pub fn sprite_get_tile_attribute(&mut self, slot: usize, x: &mut u16, y: u16) -> u8 {
        self.get_tile_attribute(self.ram.sprite(slot).floor(), x, y)
    }

    pub fn sprite_check_tile_collision2(&mut self, slot: usize) {
        self.ram.set_sprite_wall_collision(slot, 0);
        if (self.ram.sprite(slot).flags4() as i8) < 0 || self.r8(0x46c) == 0 {
            self.sprite_check_tile_collision_single_layer(slot);
            return;
        }
        self.w8(0xfb6, self.ram.sprite(slot).floor());
        self.ram.set_sprite_floor(slot, 1);
        self.sprite_check_tile_collision_single_layer(slot);
        if self.r8(0x46c) == 4 {
            self.ram.set_sprite_floor(slot, self.r8(0xfb6));
            return;
        }
        self.ram.set_sprite_floor(slot, 0);
        self.sprite_check_tile_collision_single_layer(slot);
        self.set_arr(0x1fabc, slot, self.r8(0xfa5));
    }

    pub fn sprite_check_tile_collision_single_layer(&mut self, slot: usize) {
        if self.ram.sprite(slot).flags2() & 0x20 != 0 {
            if self.sprite_check_tile_property(slot, 0x6a) {
                self.ram.set_sprite_wall_collision(
                    slot,
                    self.ram.sprite(slot).wall_collision().wrapping_add(1),
                );
            }
            return;
        }
        if (self.ram.sprite(slot).flags4() as i8) < 0 || self.r8(0x46c) == 0 {
            let y_velocity = self.ram.sprite(slot).velocity_y();
            if y_velocity != 0 {
                self.sprite_check_for_tile_in_direction_vertical(
                    slot,
                    usize::from(y_velocity as i8 >= 0),
                );
            }
            let x_velocity = self.ram.sprite(slot).velocity_x();
            if x_velocity != 0 {
                self.sprite_check_for_tile_in_direction_horizontal(
                    slot,
                    if (x_velocity as i8) < 0 { 2 } else { 3 },
                );
            }
        } else {
            for direction in [1, 0] {
                self.sprite_check_for_tile_in_direction_vertical(slot, direction);
            }
            for direction in [3, 2] {
                self.sprite_check_for_tile_in_direction_horizontal(slot, direction);
            }
        }
        if (self.ram.sprite(slot).flags5() as i8) < 0 || self.ram.sprite(slot).z() != 0 {
            return;
        }
        self.sprite_check_tile_property(slot, 0x68);
        let tile = self.r8(0xfa5);
        self.ram.set_sprite_scratch_i(slot, tile);
        match tile {
            0x1c if self.r8(0xfb3) != 0 && self.ram.sprite(slot).state() == 11 => {
                self.ram.set_sprite_floor(slot, 1);
            }
            0x20 if self.ram.sprite(slot).flags() & 1 != 0 => {
                if self.r8(0x1b) == 0 {
                    self.sprite_func8(slot);
                } else {
                    self.ram.set_sprite_state(slot, 5);
                    if matches!(self.ram.sprite(slot).kind(), 0x13 | 0x26) {
                        self.ram
                            .set_sprite_oam_flags(slot, self.ram.sprite(slot).oam_flags() & !1);
                        self.ram.set_sprite_main_delay(slot, 63);
                    } else {
                        self.ram.set_sprite_main_delay(slot, 95);
                    }
                }
            }
            0x0c if self.arr(0x1fabc, slot) == 0x1c => {
                self.sprite_fall_adjust_position(slot);
                self.ram
                    .set_sprite_wall_collision(slot, self.ram.sprite(slot).wall_collision() | 0x20);
            }
            0x68..=0x6b => self.sprite_apply_conveyor(slot, tile),
            8 if self.r8(0x46c) == 4 => self.sprite_apply_conveyor(slot, 0x6a),
            _ => {}
        }
    }

    pub fn sprite_check_for_tile_in_direction_horizontal(&mut self, slot: usize, direction: usize) {
        if !self.sprite_check_tile_in_direction(slot, direction) {
            return;
        }
        self.ram.set_sprite_wall_collision(
            slot,
            self.ram.sprite(slot).wall_collision() | SPRITE_FUNC7_TAB[direction],
        );
        if self.ram.sprite(slot).subtype() & 7 < 5 {
            let displacement = if self.ram.sprite(slot).scratch_f() != 0 {
                3
            } else {
                1
            };
            self.sprite_add_xy(
                slot,
                if direction & 1 != 0 {
                    -displacement
                } else {
                    displacement
                },
                0,
            );
        }
    }

    pub fn sprite_check_for_tile_in_direction_vertical(&mut self, slot: usize, direction: usize) {
        if !self.sprite_check_tile_in_direction(slot, direction) {
            return;
        }
        self.ram.set_sprite_wall_collision(
            slot,
            self.ram.sprite(slot).wall_collision() | SPRITE_FUNC7_TAB[direction],
        );
        if self.ram.sprite(slot).subtype() & 7 < 5 {
            let displacement = if self.ram.sprite(slot).scratch_f() != 0 {
                3
            } else {
                1
            };
            self.sprite_add_xy(
                slot,
                0,
                if direction & 1 != 0 {
                    -displacement
                } else {
                    displacement
                },
            );
        }
    }

    pub fn sprite_check_tile_in_direction(&mut self, slot: usize, direction: usize) -> bool {
        let flags = self.ram.sprite(slot).flags() & 0xf0;
        self.sprite_check_tile_property(slot, 2 * (usize::from(flags >> 2) + direction))
    }

    pub fn sprite_check_tile_property(&mut self, slot: usize, index: usize) -> bool {
        let index = index >> 1;
        let (mut x, y, in_bounds) = if self.r8(0x1b) != 0 {
            let x = (self.r16(0xfd8).wrapping_add(8) & 0x1ff)
                .wrapping_add_signed(i16::from(SPRITE_FUNC5_X[index]))
                .wrapping_sub(8);
            let y = (self.r16(0xfda).wrapping_add(8) & 0x1ff)
                .wrapping_add_signed(i16::from(SPRITE_FUNC5_Y[index]))
                .wrapping_sub(8);
            (x, y, x < 0x200 && y < 0x200)
        } else {
            let x = self
                .r16(0xfd8)
                .wrapping_add_signed(i16::from(SPRITE_FUNC5_X[index]));
            let y = self
                .r16(0xfda)
                .wrapping_add_signed(i16::from(SPRITE_FUNC5_Y[index]));
            let valid = x.wrapping_sub(self.r16(0xfbc)) < self.r16(0xfb8)
                && y.wrapping_sub(self.r16(0xfbe)) < self.r16(0xfba);
            (x, y, valid)
        };
        if !in_bounds {
            if self.ram.sprite(slot).flags2() & 0x40 != 0 {
                self.ram.set_sprite_state(slot, 0);
                return false;
            }
            return true;
        }
        let tile = self.sprite_get_tile_attribute(slot, &mut x, y);
        if self.ram.sprite(slot).deflection_bits() & 8 != 0 {
            let simplified = SPRITE_SIMPLIFIED_TILE_ATTR[usize::from(tile)];
            if simplified == 4 {
                if self.r8(0x1b) == 0 {
                    self.ram.set_sprite_scratch_e(slot, 4);
                }
            } else if simplified >= 1 {
                return if (0x10..0x14).contains(&self.r8(0xfa5)) {
                    self.entity_check_sloped_tile_collision(x, y)
                } else {
                    true
                };
            }
            return false;
        }
        if self.ram.sprite(slot).flags5() & 0x40 != 0 {
            let kind = self.ram.sprite(slot).kind();
            if matches!(kind, 0xd2 | 0x8a) && tile == 9 {
                return false;
            }
            if kind == 0x94 && self.ram.sprite(slot).scratch_e() == 0
                || matches!(kind, 0xe3 | 0x8c | 0x9a | 0x81)
            {
                return !matches!(tile, 8 | 9);
            }
        }
        if SPRITE_FUNC5_TAB3[usize::from(tile)] == 0 {
            return false;
        }
        if (0x10..0x14).contains(&self.r8(0xfa5)) {
            return self.entity_check_sloped_tile_collision(x, y);
        }
        if self.r8(0xfa5) == 0x44 {
            if self.ram.sprite(slot).scratch_f() != 0 && self.ram.sprite(slot).damage() as i8 >= 0 {
                let damage = if self.ram.sprite(slot).kind() == 0x88
                    && self.hooks.enhanced_features0() & 4096 != 0
                {
                    if self.ram.sprite(slot).hit_timer() != 0 {
                        return true;
                    }
                    6
                } else {
                    4
                };
                self.ancilla_check_damage_to_sprite_preset(slot, damage);
                if self.ram.sprite(slot).hit_timer() != 0 {
                    self.ram.set_sprite_hit_timer(slot, 153);
                    self.ram.set_sprite_scratch_f(slot, 0);
                }
            }
        } else if self.r8(0xfa5) == 0x20 {
            return self.ram.sprite(slot).flags() & 1 == 0
                || self.ram.sprite(slot).scratch_f() == 0;
        }
        true
    }

    pub fn entity_check_sloped_tile_collision(&self, x: u16, y: u16) -> bool {
        let a = (y & 7) as u8;
        let slope = self.r8(0xfa5).wrapping_sub(0x10);
        let b = SLOPED_TILE[usize::from(slope) * 8 + usize::from(x & 7)] as u8;
        if slope < 2 { b >= a } else { a >= b }
    }

    pub fn sprite_fall_adjust_position(&mut self, slot: usize) {
        self.sprite_add_xy(slot, self.r16(0x312) as i16, self.r16(0x310) as i16);
    }

    pub fn oam_allocate_defer_to_player(&mut self, slot: usize) {
        if self.ram.sprite(slot).floor() != self.r8(0xee) {
            return;
        }
        let right = self.sprite_is_right_of_link(slot);
        if right.b.wrapping_add(0x10) >= 0x20 {
            return;
        }
        let below = self.sprite_is_below_link(slot);
        if below.b.wrapping_add(0x20) >= 0x48 {
            return;
        }
        let bytes = (self.ram.sprite(slot).flags2() & 0x1f).wrapping_add(1) << 2;
        if below.a != 0 {
            self.oam_allocate_from_region_c(bytes);
        } else {
            self.oam_allocate_from_region_b(bytes);
        }
    }

    pub fn sprite_return_if_recoiling(&mut self, slot: usize) -> bool {
        let mut timer = self.ram.sprite(slot).scratch_f();
        if timer == 0 {
            return false;
        }
        if timer & 0x7f == 0 {
            self.ram.set_sprite_scratch_f(slot, 0);
            return false;
        }
        let old_y = self.ram.sprite(slot).velocity_y();
        let old_x = self.ram.sprite(slot).velocity_x();
        timer = timer.wrapping_sub(1);
        self.ram.set_sprite_scratch_f(slot, timer);
        if timer == 0
            && (self.ram.sprite(slot).recoil_x().wrapping_add(0x20) >= 0x40
                || self.ram.sprite(slot).recoil_y().wrapping_add(0x20) >= 0x40)
        {
            timer = 144;
            self.ram.set_sprite_scratch_f(slot, timer);
        }
        if (timer as i8) >= 0
            && self.r8(0x1a) & SPRITE2_RETURN_IF_RECOILING_MASKS[usize::from(timer >> 2)] == 0
        {
            self.ram
                .set_sprite_velocity_y(slot, self.ram.sprite(slot).recoil_y());
            self.ram
                .set_sprite_velocity_x(slot, self.ram.sprite(slot).recoil_x());
            let collision = if (self.ram.sprite(slot).bump_damage() as i8) >= 0 {
                self.sprite_check_tile_collision(slot) & 0xf
            } else {
                0
            };
            if collision != 0 {
                if collision < 4 {
                    self.ram.set_sprite_recoil_x(slot, 0);
                    self.ram.set_sprite_velocity_x(slot, 0);
                } else {
                    self.ram.set_sprite_recoil_y(slot, 0);
                    self.ram.set_sprite_velocity_y(slot, 0);
                }
            } else {
                self.sprite_move_xy(slot);
            }
        }
        self.ram.set_sprite_velocity_y(slot, old_y);
        self.ram.set_sprite_velocity_x(slot, old_x);
        self.ram.sprite(slot).kind() != 0x7a
    }

    pub fn sprite_nullify_hookshot_drag(&mut self) {
        for slot in (0..5).rev() {
            if self.arr(0xc4a, slot) & 0x1f == 0 && self.r8(0x37e) != 0 {
                self.w8(0x37e, 0);
                break;
            }
        }
        self.w8(0x41, (self.r16(0x22) >> 8) as u8);
        self.w8(0x40, (self.r16(0x20) >> 8) as u8);
        self.w16(0x22, self.r16(0xfc2));
        self.w16(0x20, self.r16(0xfc4));
        SpriteHooks::handle_indoor_camera_and_doors(self.hooks, &mut self.ram);
    }

    pub fn sprite_halt_all_movement(&mut self) {
        self.sprite_nullify_hookshot_drag();
        self.w8(0x5e, 0);
        SpriteHooks::link_cancel_dash(self.hooks, &mut self.ram);
    }

    pub fn sprite_behave_as_barrier(&mut self, slot: usize) {
        let flags = self.ram.sprite(slot).flags4();
        self.ram.set_sprite_flags4(slot, 0);
        if self.sprite_check_damage_to_link_same_layer(slot) {
            self.sprite_halt_all_movement();
        }
        self.ram.set_sprite_flags4(slot, flags);
    }

    pub fn sprite_overworld_proximity_motivated_load(&mut self, x: u16, y: u16) {
        let x_relative = x.wrapping_sub(self.r16(0xfbc));
        let y_relative = y.wrapping_sub(self.r16(0xfbe));
        if x_relative >= self.r16(0xfb8) || y_relative >= self.r16(0xfba) {
            return;
        }
        let page = (((y_relative >> 8) as u8) * 4) | (x_relative >> 8) as u8;
        let cell = (y as u8 & 0xf0) | ((x >> 4) as u8 & 0x0f);
        self.overworld_load_proxima_sprite_if_alive(u16::from(page) << 8 | u16::from(cell));
    }

    pub fn overworld_load_proxima_sprite_if_alive(&mut self, block: u16) {
        let sprite_to_spawn = self.r8(0x1df80 + usize::from(block));
        if sprite_to_spawn == 0 {
            return;
        }
        let mask = 0x80 >> (block & 7);
        let loaded_address = 0x1ef80 + usize::from(block >> 3);
        if self.r8(loaded_address) & mask != 0 {
            return;
        }
        if sprite_to_spawn >= 0xf4 {
            let Some(slot) = self.alloc_overlord() else {
                return;
            };
            self.w8(loaded_address, self.r8(loaded_address) | mask);
            self.w16(0xb48 + slot * 2, block);
            let kind = sprite_to_spawn.wrapping_sub(0xf3);
            self.set_arr(OVERLORD_TYPE, slot, kind);
            self.set_arr(
                0xb08,
                slot,
                ((block << 4) as u8).wrapping_add(if kind == 1 { 8 } else { 0 }),
            );
            self.set_arr(0xb18, slot, block as u8 & 0xf0);
            self.set_arr(
                0xb10,
                slot,
                ((block >> 8) as u8 & 3).wrapping_add((self.r16(0xfbc) >> 8) as u8),
            );
            self.set_arr(
                0xb20,
                slot,
                ((block >> 10) as u8).wrapping_add((self.r16(0xfbe) >> 8) as u8),
            );
            self.set_arr(0xb40, slot, 0);
            self.set_arr(0xcca, slot, self.r8(0x40a));
            self.set_arr(0xb28, slot, 0);
            self.set_arr(0xb30, slot, 0);
            self.set_arr(0xb38, slot, 0);
        } else {
            let Some(slot) = self.overworld_alloc_sprite(sprite_to_spawn) else {
                return;
            };
            self.w8(loaded_address, self.r8(loaded_address) | mask);
            self.ram.set_sprite_scratch_n_word(slot, block);
            self.ram
                .set_sprite_kind(slot, sprite_to_spawn.wrapping_sub(1));
            self.ram.set_sprite_state(slot, 8);
            self.ram.set_sprite_x_low(slot, (block << 4) as u8 & 0xf0);
            self.ram.set_sprite_y_low(slot, block as u8 & 0xf0);
            self.ram.set_sprite_x_high(
                slot,
                ((block >> 8) as u8 & 3).wrapping_add((self.r16(0xfbc) >> 8) as u8),
            );
            self.ram.set_sprite_y_high(
                slot,
                ((block >> 10) as u8).wrapping_add((self.r16(0xfbe) >> 8) as u8),
            );
            self.ram.set_sprite_floor(slot, 0);
            self.ram.set_sprite_subtype(slot, 0);
            self.ram.set_sprite_death_action(slot, 0);
        }
    }

    pub fn sprite_activate_when_proximal(&mut self) {
        let direction = self.r8(0x69f);
        if direction == 0 {
            return;
        }
        let extension = if self.hooks.enhanced_features0() & 1 != 0 {
            0x40
        } else {
            0
        };
        let x = self.r16(0xe2).wrapping_add(if (direction as i8) < 0 {
            (-(0x10i16 + extension)) as u16
        } else {
            (0x110 + extension) as u16
        });
        let mut y = self.r16(0xe8).wrapping_sub(0x30);
        for _ in 0..22 {
            self.sprite_overworld_proximity_motivated_load(x, y);
            y = y.wrapping_add(16);
        }
    }

    pub fn sprite_activate_when_proximal_big(&mut self) {
        let direction = self.r8(0x69e);
        if direction == 0 {
            return;
        }
        let extension = if self.hooks.enhanced_features0() & 1 != 0 {
            0x40
        } else {
            0
        };
        let mut x = self.r16(0xe2).wrapping_sub(0x30 + extension);
        let y = self
            .r16(0xe8)
            .wrapping_add(if (direction as i8) < 0 { 0xfff0 } else { 0x110 });
        for _ in 0..=(21 + (extension >> 3)) {
            self.sprite_overworld_proximity_motivated_load(x, y);
            x = x.wrapping_add(16);
        }
    }

    pub fn sprite_proximity_activation(&mut self) {
        if self.r8(0x11) != 0 {
            self.sprite_activate_when_proximal();
            self.sprite_activate_when_proximal_big();
        } else {
            if self.r8(0xfb7) & 1 == 0 {
                self.sprite_activate_when_proximal();
            } else {
                self.sprite_activate_when_proximal_big();
            }
            self.w8(0xfb7, self.r8(0xfb7).wrapping_add(1));
        }
    }

    pub fn sprite_activate_all_proxima(&mut self) {
        let saved_scroll = self.r16(0xe2);
        let saved_direction = self.r8(0x69f);
        self.w8(0x69f, 0xff);
        let extension = if self.hooks.enhanced_features0() & 1 != 0 {
            0x40
        } else {
            0
        };
        self.w16(0xe2, saved_scroll.wrapping_sub(extension));
        for _ in 0..=(21 + (extension >> 3)) {
            self.sprite_activate_when_proximal();
            self.w16(0xe2, self.r16(0xe2).wrapping_add(16));
        }
        self.w8(0x69f, saved_direction);
        self.w16(0xe2, saved_scroll);
    }

    pub fn overworld_load_sprites(&mut self) {
        let area = self.r16(0x40a);
        self.w16(0xfbc, (area & 7) << 9);
        self.w16(0xfbe, ((area & 0x3f) >> 2 & 0x0e) << 8);
        let size = u16::from(OVERWORLD_AREA_SPRCOLL_SIZES[usize::from(area as u8)]) << 8;
        self.w16(0xfb8, size);
        self.w16(0xfba, size);
        let source = self.hooks.overworld_sprite_data(&self.ram, area).to_vec();
        for entry in source.chunks_exact(3) {
            if entry[0] == 0xff {
                break;
            }
            if entry[2] == 0xf4 {
                self.w8(0xffd, self.r8(0xffd).wrapping_add(1));
                continue;
            }
            let page_y = (entry[0] >> 4) << 2;
            let page = (entry[1] >> 4).wrapping_add(page_y);
            let cell = (entry[1] & 0x0f) | entry[0] << 4;
            self.w8(
                0x1df80 + usize::from(cell) + (usize::from(page) << 8),
                entry[2].wrapping_add(1),
            );
        }
    }

    pub fn sprite_overworld_reload_all_just_load(&mut self) {
        self.sprite_reset_all_no_disable();
        self.overworld_load_sprites();
        self.sprite_activate_all_proxima();
    }

    pub fn sprite_reload_all_overworld(&mut self) {
        self.sprite_disable_all();
        self.sprite_overworld_reload_all_just_load();
    }
    pub fn link_setup_hit_box(&self, hitbox: &mut SpriteHitBox) {
        hitbox.r2 = 8;
        hitbox.r3 = 8;
        let x = self.r16(0x22).wrapping_add(4);
        hitbox.r0_xlo = x as u8;
        hitbox.r8_xhi = (x >> 8) as u8;
        let y = self.r16(0x20).wrapping_add(8);
        hitbox.r1_ylo = y as u8;
        hitbox.r9_yhi = (y >> 8) as u8;
    }
    pub fn link_setup_hit_box_conditional(&self, hitbox: &mut SpriteHitBox) {
        if self.r8(0x37b) != 0 {
            hitbox.r9_yhi = 0x80;
        } else {
            self.link_setup_hit_box(hitbox);
        }
    }
    pub fn sprite_setup_hit_box(&self, slot: usize, hitbox: &mut SpriteHitBox) {
        if (self.ram.sprite(slot).z() as i8) < 0 {
            hitbox.r10_spr_xhi = 0x80;
            return;
        }
        let index = usize::from(self.ram.sprite(slot).flags4() & 0x1f);
        let xlow = u16::from(self.ram.sprite(slot).x_low())
            .wrapping_add(u16::from(SPRITE_HITBOX_X_LO[index] as u8));
        hitbox.r4_spr_xlo = xlow as u8;
        hitbox.r10_spr_xhi = self
            .ram
            .sprite(slot)
            .x_high()
            .wrapping_add(SPRITE_HITBOX_X_HI[index] as u8)
            .wrapping_add((xlow >> 8) as u8);
        let ylow = u16::from(self.ram.sprite(slot).y_low())
            .wrapping_add(u16::from(SPRITE_HITBOX_Y_LO[index] as u8));
        let borrow_source = (ylow as u8).wrapping_sub(self.ram.sprite(slot).z());
        hitbox.r5_spr_ylo = borrow_source;
        hitbox.r11_spr_yhi = self
            .ram
            .sprite(slot)
            .y_high()
            .wrapping_sub(u8::from((ylow as u8) < self.ram.sprite(slot).z()))
            .wrapping_add((ylow >> 8) as u8)
            .wrapping_add(SPRITE_HITBOX_Y_HI[index] as u8);
        hitbox.r6_spr_xsize = SPRITE_HITBOX_X_SIZE[index];
        hitbox.r7_spr_ysize = SPRITE_HITBOX_Y_SIZE[index];
    }
    pub fn sprite_setup_hit_box00(&self, slot: usize) -> bool {
        self.r16(0x22)
            .wrapping_sub(self.r16(0xfd8))
            .wrapping_add(11)
            < 23
            && self
                .r16(0x20)
                .wrapping_sub(self.r16(0xfda))
                .wrapping_add(u16::from(self.ram.sprite(slot).z()))
                .wrapping_add(16)
                < 24
    }

    pub fn player_setup_action_hit_box(&self, hitbox: &mut SpriteHitBox) {
        if self.r8(0x372) != 0 {
            let direction = usize::from(self.r8(0x2f) >> 1);
            let x_offset = u16::from(PLAYER_ACTION_BOX_RUN_X_LO[direction])
                | u16::from(PLAYER_ACTION_BOX_RUN_X_HI[direction]) << 8;
            let y_offset = u16::from(PLAYER_ACTION_BOX_RUN_Y_LO[direction])
                | u16::from(PLAYER_ACTION_BOX_RUN_Y_HI[direction]) << 8;
            let x = self.r16(0x22).wrapping_add(x_offset);
            let y = self.r16(0x20).wrapping_add(y_offset);
            hitbox.r0_xlo = x as u8;
            hitbox.r8_xhi = (x >> 8) as u8;
            hitbox.r1_ylo = y as u8;
            hitbox.r9_yhi = (y >> 8) as u8;
            hitbox.r2 = 16;
            hitbox.r3 = 16;
            return;
        }
        let mut frame = 0usize;
        if self.r8(0x301) & 10 == 0 && self.r8(0x37a) & 0x10 == 0 {
            let sword_frame = self.r8(0x3c);
            if (sword_frame as i8) < 0 {
                let x = self.r16(0x22).wrapping_sub(14);
                let y = self.r16(0x20).wrapping_sub(10);
                hitbox.r0_xlo = x as u8;
                hitbox.r8_xhi = (x >> 8) as u8;
                hitbox.r1_ylo = y as u8;
                hitbox.r9_yhi = (y >> 8) as u8;
                hitbox.r2 = 44;
                hitbox.r3 = 45;
                return;
            }
            if PLAYER_SETUP_ACTION_HIT_BOX_TAB4[usize::from(sword_frame)] != 0 {
                hitbox.r8_xhi = 0x80;
                return;
            }
            frame = usize::from(self.r8(0x2f)) * 8 + usize::from(sword_frame) + 1;
        }
        let x_offset =
            PLAYER_SETUP_ACTION_HIT_BOX_X[frame].wrapping_add(self.r8(0x45) as i8) as i16 as u16;
        let y_offset =
            PLAYER_SETUP_ACTION_HIT_BOX_Y[frame].wrapping_add(self.r8(0x44) as i8) as i16 as u16;
        let x = self.r16(0x22).wrapping_add(x_offset);
        let y = self.r16(0x20).wrapping_add(y_offset);
        hitbox.r0_xlo = x as u8;
        hitbox.r8_xhi = (x >> 8) as u8;
        hitbox.r1_ylo = y as u8;
        hitbox.r9_yhi = (y >> 8) as u8;
        hitbox.r2 = PLAYER_SETUP_ACTION_HIT_BOX_W[frame] as u8;
        hitbox.r3 = PLAYER_SETUP_ACTION_HIT_BOX_H[frame] as u8;
    }

    pub fn link_update_hit_box_with_sword(&self, hitbox: &mut SpriteHitBox) {
        let sword_frame = self.r8(0x3c);
        if self.r8(0x79) != 0
            || (sword_frame as i8) < 0
            || PLAYER_SETUP_ACTION_HIT_BOX_TAB4[usize::from(sword_frame)] != 0
        {
            return;
        }
        let frame = usize::from(self.r8(0x2f)) * 8 + usize::from(sword_frame) + 1;
        let mut x = self.r16(0x22).wrapping_add(
            PLAYER_SETUP_ACTION_HIT_BOX_X[frame].wrapping_add(self.r8(0x45) as i8) as i16 as u16,
        );
        let mut y = self.r16(0x20).wrapping_add(
            PLAYER_SETUP_ACTION_HIT_BOX_Y[frame].wrapping_add(self.r8(0x44) as i8) as i16 as u16,
        );
        hitbox.r2 = PLAYER_SETUP_ACTION_HIT_BOX_W[frame] as u8;
        let shrink_x = 6.min(hitbox.r2.saturating_sub(2));
        hitbox.r2 = hitbox.r2.wrapping_sub(shrink_x);
        x = x.wrapping_add(u16::from(shrink_x >> 1));
        hitbox.r3 = PLAYER_SETUP_ACTION_HIT_BOX_H[frame] as u8;
        let shrink_y = 6.min(hitbox.r3.saturating_sub(2));
        hitbox.r3 = hitbox.r3.wrapping_sub(shrink_y);
        y = y.wrapping_add(u16::from(shrink_y >> 1));
        hitbox.r0_xlo = x as u8;
        hitbox.r8_xhi = (x >> 8) as u8;
        hitbox.r1_ylo = y as u8;
        hitbox.r9_yhi = (y >> 8) as u8;
    }

    pub fn sprite_do_hit_boxes_fast(&self, slot: usize, hitbox: &mut SpriteHitBox) {
        if self.r8(0xfab) == 0x80 {
            hitbox.r10_spr_xhi = 0x80;
            return;
        }
        let x = self
            .sprite_get_x(slot)
            .wrapping_add((self.r8(0xfab) as i8) as i16 as u16);
        let y = self
            .sprite_get_y(slot)
            .wrapping_add((self.r8(0xfaa) as i8) as i16 as u16);
        hitbox.r4_spr_xlo = x as u8;
        hitbox.r10_spr_xhi = (x >> 8) as u8;
        hitbox.r5_spr_ylo = y as u8;
        hitbox.r11_spr_yhi = (y >> 8) as u8;
        let size = if self.ram.sprite(slot).kind() == 0x6a {
            16
        } else {
            3
        };
        hitbox.r6_spr_xsize = size;
        hitbox.r7_spr_ysize = size;
    }

    pub fn sprite_apply_recoil_to_link(&mut self, slot: usize, velocity: u8) {
        let projected = self.sprite_project_speed_towards_link(slot, velocity);
        self.w8(0x28, projected.x);
        self.w8(0x27, projected.y);
        self.w8(0x29, velocity >> 1);
        self.w8(0xc7, velocity >> 1);
        self.w16(0x24, 0);
    }

    pub fn sprite_attempt_damage_to_link_plus_recoil(&mut self, slot: usize) {
        if self.r8(0x31f) | self.r8(0x37b) != 0 {
            return;
        }
        self.w8(0x46, 19);
        self.sprite_apply_recoil_to_link(slot, 24);
        self.w8(0x4d, 1);
        let damage_class = self.ram.sprite(slot).bump_damage() & 0x0f;
        let armor = self.r8(0xf35b);
        let damage = PLAYER_DAMAGES[usize::from(damage_class) * 3 + usize::from(armor)];
        self.w8(0x373, damage as u8);
        if self.ram.sprite(slot).kind() == 0x61 && self.ram.sprite(slot).scratch_c() != 0 {
            self.w8(0x28, self.ram.sprite(slot).velocity_x().wrapping_mul(2));
            self.w8(0x27, self.ram.sprite(slot).velocity_y().wrapping_mul(2));
        }
    }

    pub fn sprite_attempt_damage_to_link_with_collision_check(&mut self, slot: usize) {
        if (slot as u8 ^ self.r8(0x1a)) & 1 != 0 {
            return;
        }
        let mut hitbox = SpriteHitBox::default();
        self.sprite_do_hit_boxes_fast(slot, &mut hitbox);
        self.link_setup_hit_box_conditional(&mut hitbox);
        if Self::check_if_hit_boxes_overlap(&hitbox) {
            self.sprite_attempt_damage_to_link_plus_recoil(slot);
        }
    }
    pub fn sprite_check_damage_to_link(&mut self, slot: usize) -> bool {
        self.r8(0x37b) == 0 && self.sprite_check_damage_to_player_1(slot)
    }

    pub fn sprite_check_damage_to_player_1(&mut self, slot: usize) -> bool {
        if ((slot as u8 ^ self.r8(0x1a)) & 3) | self.ram.sprite(slot).hit_timer() != 0 {
            false
        } else {
            self.sprite_check_damage_to_link_same_layer(slot)
        }
    }

    pub fn sprite_check_damage_to_link_same_layer(&mut self, slot: usize) -> bool {
        self.ram.sprite(slot).floor() == self.r8(0xee)
            && self.sprite_check_damage_to_link_ignore_layer(slot)
    }

    pub fn sprite_check_damage_to_link_ignore_layer(&mut self, slot: usize) -> bool {
        let collision = if self.ram.sprite(slot).flags4() != 0 {
            let mut hitbox = SpriteHitBox::default();
            self.link_setup_hit_box(&mut hitbox);
            let kind = self.ram.sprite(slot).kind();
            if (0xd8..=0xe6).contains(&kind) && self.hooks.enhanced_features0() & 16 != 0 {
                self.link_update_hit_box_with_sword(&mut hitbox);
            }
            self.sprite_setup_hit_box(slot, &mut hitbox);
            Self::check_if_hit_boxes_overlap(&hitbox)
        } else {
            self.sprite_setup_hit_box00(slot)
        };
        if (self.ram.sprite(slot).flags2() as i8) < 0 {
            return collision;
        }
        if !collision || self.r8(0x4d) != 0 {
            return false;
        }
        let shield_failure = self.r8(0x2e0) != 0
            || (self.r8(0x308) as i8) < 0
            || self.ram.sprite(slot).flags5() & 0x20 == 0
            || self.r8(0xf35a) == 0;
        if !shield_failure {
            self.ram.set_sprite_state(slot, 0);
            let direction = if self.r8(0x3c) != 0 {
                SPRITE_DAMAGE_TAB2[usize::from(self.r8(0x2f) >> 1)]
            } else {
                self.r8(0x2f)
            };
            if direction == SPRITE_DAMAGE_TAB3[usize::from(self.ram.sprite(slot).scratch_d())] {
                self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 6);
                self.sprite_place_rupulse_spark_2(slot);
                match self.ram.sprite(slot).kind() {
                    0x95 => {
                        self.hooks.sprite_sfx3_with_pan(&mut self.ram, slot, 0x26);
                        return false;
                    }
                    0x9b => {
                        self.sprite_invert_xy_speeds(slot);
                        self.ram
                            .set_sprite_scratch_d(slot, self.ram.sprite(slot).scratch_d() ^ 1);
                        self.ram.set_sprite_ai_state(
                            slot,
                            self.ram.sprite(slot).ai_state().wrapping_add(1),
                        );
                        self.ram.set_sprite_state(slot, 9);
                        return false;
                    }
                    0x1b => {
                        self.sprite_schedule_for_breakage(slot);
                        return false;
                    }
                    0x0c => {
                        self.sprite_func3(slot);
                        return true;
                    }
                    _ => return false,
                }
            }
        }
        self.sprite_attempt_damage_to_link_plus_recoil(slot);
        if self.ram.sprite(slot).kind() == 0x0c {
            self.sprite_func3(slot);
        }
        true
    }

    pub fn sprite_check_damage_from_link(&mut self, slot: usize) -> u8 {
        if self.ram.sprite(slot).hit_timer() & 0x80 != 0
            || self.ram.sprite(slot).floor() != self.r8(0xee)
            || self.r8(0x44) == 0x80
        {
            return 0;
        }
        let mut hitbox = SpriteHitBox::default();
        self.player_setup_action_hit_box(&mut hitbox);
        self.sprite_setup_hit_box(slot, &mut hitbox);
        if !Self::check_if_hit_boxes_overlap(&hitbox) {
            return 0;
        }
        self.w8(0x47, 0);
        if self.r8(0x37a) & 0x10 != 0 {
            return 3;
        }
        if self.r8(0x301) & 10 != 0 {
            if self.ram.sprite(slot).kind() >= 0xd6 {
                return 0;
            }
            if self.ram.sprite(slot).state() == 11 && self.ram.sprite(slot).unknown5() != 0 {
                self.ram.set_sprite_state(slot, 2);
                self.ram.set_sprite_main_delay(slot, 32);
                self.ram
                    .set_sprite_flags2(slot, (self.ram.sprite(slot).flags2() & 0xe0) | 3);
                self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 0x1f);
                return 3;
            }
        }
        let kind = self.ram.sprite(slot).kind();
        if kind == 0x7b {
            if self.r8(0x3c).wrapping_sub(9) as i8 >= 0 {
                return 0;
            }
        } else if kind == 9 {
            if self.ram.sprite(slot).scratch_a() == 0 {
                self.sprite_apply_recoil_to_link(slot, 48);
                self.w8(0x47, 144);
                self.w8(0x46, 16);
                self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 0x21);
                self.ram.set_sprite_aux1_delay(slot, 48);
                let effect = if self.hooks.enhanced_features0() & 4096 != 0 {
                    0x32
                } else {
                    0
                };
                let sound = SoundRequest::at_world_x(&self.ram, effect, self.sprite_get_x(slot));
                SoundPort::EffectsB.overwrite(&mut self.ram, sound);
                self.link_place_weapon_tink();
                return 1;
            }
        } else if kind == 0x92 {
            if self.ram.sprite(slot).scratch_c() < 3 {
                if self.r8(0x47) == 0 {
                    self.sprite_apply_recoil_to_link(slot, 4);
                    self.w8(0x46, 16);
                    self.w8(0x47, 16);
                }
                self.link_place_weapon_tink();
                return 1;
            }
            self.sprite_apply_recoil_to_link(slot, 32);
            self.w8(0x47, 144);
            self.w8(0x46, 16);
        } else if matches!(kind, 0x26 | 0x13 | 2) {
            let parry = kind == 2
                || kind == 0x13
                    && SPRITE_DAMAGE_TAB3[usize::from(self.ram.sprite(slot).scratch_d())]
                        == self.r8(0x2f);
            self.sprite_attempt_zap_damage(slot);
            self.sprite_apply_recoil_to_link(slot, 32);
            self.w8(0x47, 16);
            self.w8(0x46, 16);
            if parry {
                self.ram.set_sprite_hit_timer(slot, 0);
                self.link_place_weapon_tink();
            }
            return 0;
        } else if matches!(kind, 0xcb | 0xcd | 0xcc | 0xd6 | 0xd7 | 0xce | 0x54) {
            self.sprite_apply_recoil_to_link(slot, 32);
            self.w8(0x47, 144);
            self.w8(0x46, 16);
        }
        if self.ram.sprite(slot).deflection_bits() & 4 == 0 {
            self.sprite_attempt_zap_damage(slot);
            return 1;
        }
        if self.r8(0x47) == 0 {
            self.sprite_apply_recoil_to_link(slot, 4);
            self.w8(0x46, 16);
            self.w8(0x47, 16);
        }
        self.link_place_weapon_tink();
        1
    }

    pub fn guard_parry_sword_attacks(&mut self, slot: usize) {
        if self.ram.sprite(slot).floor() != self.r8(0xee)
            || self.r8(0x46) | self.r8(0x4d) != 0
            || (self.ram.sprite(slot).hit_timer() as i8) < 0
        {
            return;
        }
        let mut hitbox = SpriteHitBox::default();
        self.sprite_do_hit_boxes_fast(slot, &mut hitbox);
        if self.r8(0x37a) & 0x10 != 0 || self.r8(0x44) == 0x80 {
            self.sprite_attempt_damage_to_link_with_collision_check(slot);
            return;
        }
        self.player_setup_action_hit_box(&mut hitbox);
        if (self.r8(0x3c) as i8) < 0 || !Self::check_if_hit_boxes_overlap(&hitbox) {
            self.sprite_setup_hit_box(slot, &mut hitbox);
            if !Self::check_if_hit_boxes_overlap(&hitbox) {
                self.sprite_attempt_damage_to_link_with_collision_check(slot);
            } else {
                self.sprite_attempt_zap_damage(slot);
            }
            return;
        }
        if self.ram.sprite(slot).kind() != 0x6a {
            let random = usize::from(self.hooks.random_u8(&mut self.ram) & 7);
            self.ram
                .set_sprite_scratch_f(slot, SPRITE_FUNC1_TAB[random]);
        }
        let random = usize::from(self.hooks.random_u8(&mut self.ram) & 7);
        self.w8(0x46, SPRITE_FUNC1_TAB2[random]);
        let velocity = if (self.r8(0x3c).wrapping_sub(9) as i8) < 0 {
            32
        } else {
            24
        };
        let projected = self.sprite_project_speed_towards_link(slot, velocity);
        self.ram
            .set_sprite_recoil_x(slot, VelocityByte::from_bits(projected.x).negated().bits());
        self.ram
            .set_sprite_recoil_y(slot, VelocityByte::from_bits(projected.y).negated().bits());
        self.sprite_apply_recoil_to_link(
            slot,
            if (self.r8(0x3c).wrapping_sub(9) as i8) < 0 {
                8
            } else {
                16
            },
        );
        self.link_place_weapon_tink();
        self.w8(0x47, 0x90);
    }

    pub fn sprite_attempt_zap_damage(&mut self, slot: usize) {
        let mut kind = self.ram.sprite(slot).kind();
        let zaps = kind == 0x7a
            || kind == 0x0d && {
                kind = self.r8(0xf359);
                kind < 4
            }
            || matches!(kind, 0x24 | 0x23) && self.ram.sprite(slot).main_delay() != 0;
        if zaps && self.ram.sprite(slot).state() == 9 {
            if self.r8(0x31f) == 0 {
                self.ram.set_sprite_aux1_delay(slot, 64);
                self.w8(0x360, 64);
                self.sprite_attempt_damage_to_link_plus_recoil(slot);
            }
        } else {
            let speed = if (self.r8(0x3c).wrapping_sub(9) as i8) < 0 {
                0x50
            } else {
                0x40
            };
            let velocity = self.sprite_project_speed_towards_link(slot, speed);
            self.ram
                .set_sprite_recoil_x(slot, VelocityByte::from_bits(velocity.x).negated().bits());
            self.ram
                .set_sprite_recoil_y(slot, VelocityByte::from_bits(velocity.y).negated().bits());
            self.sprite_calculate_sword_damage(slot);
        }
    }

    pub fn sprite_calculate_sword_damage(&mut self, slot: usize) {
        if self.ram.sprite(slot).flags3() & 0x40 != 0 {
            return;
        }
        self.ram.set_sprite_unknown1(slot, self.r8(0x372));
        let mut index = self.r8(0xf359).wrapping_sub(1);
        if self.r8(0x372) == 0 {
            index |= if (self.r8(0x3c) as i8) < 0 {
                4
            } else if (self.r8(0x3c).wrapping_sub(9) as i8) < 0 {
                0
            } else {
                8
            };
        }
        self.w8(0xcf2, SPRITE_FUNC14_DAMAGE[usize::from(index)]);
        if self.r8(0x301) & 10 != 0 {
            self.w8(0xcf2, 3);
        }
        self.w8(0x2e3, 4);
        self.w8(0x47, 16);
        self.sprite_apply_calculated_damage(slot, 0x9d);
    }

    pub fn ancilla_check_damage_to_sprite_preset(&mut self, slot: usize, damage_type: u8) {
        if damage_type == 15 && self.ram.sprite(slot).z() != 0 {
            return;
        }
        self.sprite_func15(slot, damage_type);
        if damage_type != 0 && damage_type != 7
            || self.ram.sprite(slot).damage() != 0
            || self.r8(0xfac) != 0
        {
            return;
        }
        self.w8(0xfac, 5);
        let ancilla = usize::from(self.r8(0xfb6));
        self.w8(0xfad, self.r8(0xc04 + ancilla).wrapping_add(4));
        self.w8(0xfae, self.r8(0xbfa + ancilla));
        self.w8(0xb68, self.r8(0xee));
        self.w8(0x12e, 0);
        self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 5);
    }

    pub fn sprite_func15(&mut self, slot: usize, damage_type: u8) {
        self.w8(0xcf2, damage_type);
        self.sprite_apply_calculated_damage(slot, if damage_type == 8 { 0x35 } else { 0x20 });
    }

    pub fn sprite_apply_calculated_damage(&mut self, slot: usize, hit_timer: u8) {
        if self.ram.sprite(slot).flags3() & 0x40 != 0 || self.ram.sprite(slot).kind() >= 0xd8 {
            return;
        }
        let damage_type = self.r8(0xcf2);
        let enemy_class = self.r8(0x16000
            + usize::from(self.ram.sprite(slot).kind()) * 16
            + usize::from(damage_type));
        let damage = ENEMY_DAMAGES[(usize::from(damage_type) * 8) | usize::from(enemy_class)];
        self.sprite_give_damage(slot, damage, hit_timer);
    }

    pub fn sprite_give_damage(&mut self, slot: usize, damage: u8, hit_timer: u8) {
        if damage == 249 {
            self.sprite_func18(slot, 0xe3);
            return;
        }
        if damage == 250 {
            self.sprite_func18(slot, 0x8f);
            self.ram.set_sprite_ai_state(slot, 2);
            self.ram.set_sprite_velocity_z(slot, 32);
            self.ram.set_sprite_oam_flags(slot, 8);
            self.ram.set_sprite_scratch_f(slot, 0);
            self.ram.set_sprite_hit_timer(slot, 0);
            self.ram.set_sprite_health(slot, 0);
            self.ram.set_sprite_bump_damage(slot, 1);
            self.ram.set_sprite_flags5(slot, 1);
            return;
        }
        if damage >= self.ram.sprite(slot).damage() {
            self.ram.set_sprite_damage(slot, damage);
        }
        if damage == 0 {
            if self.r8(0xcf2) != 10 {
                if self.ram.sprite(slot).flags() & 4 != 0 {
                    self.finish_sprite_damage(slot);
                    return;
                }
                self.w8(0x2e3, 0);
            }
            self.ram.set_sprite_hit_timer(slot, 0);
            self.ram.set_sprite_damage(slot, 0);
            return;
        }
        if damage >= 254 && self.ram.sprite(slot).state() == 11 {
            self.ram.set_sprite_hit_timer(slot, 0);
            self.ram.set_sprite_damage(slot, 0);
            return;
        }
        if self.ram.sprite(slot).kind() == 0x9a && self.ram.sprite(slot).damage() < 0xf0 {
            self.ram.set_sprite_state(slot, 9);
            self.ram.set_sprite_ai_state(slot, 4);
            self.ram.set_sprite_main_delay(slot, 15);
            self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 0x28);
            return;
        }
        if self.ram.sprite(slot).kind() == 0x1b {
            self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 5);
            self.sprite_schedule_for_breakage(slot);
            self.sprite_place_weapon_tink(slot);
            return;
        }
        self.ram.set_sprite_hit_timer(slot, hit_timer);
        if self.ram.sprite(slot).kind() != 0x92 || self.ram.sprite(slot).scratch_c() >= 3 {
            let effect = if self.ram.sprite(slot).flags() & 2 != 0 {
                0x21
            } else if self.ram.sprite(slot).flags5() & 0x10 != 0 {
                0x1c
            } else {
                8
            };
            let sound = SoundRequest::at_world_x(&self.ram, effect, self.sprite_get_x(slot));
            SoundPort::EffectsB.overwrite(&mut self.ram, sound);
        }
        self.finish_sprite_damage(slot);
    }

    fn finish_sprite_damage(&mut self, slot: usize) {
        let kind = self.ram.sprite(slot).kind();
        let value = if self.r8(0xcf2) >= 13 {
            0
        } else if kind == 9 {
            20
        } else if matches!(kind, 0x53 | 0x18) {
            11
        } else {
            15
        };
        self.ram.set_sprite_scratch_f(slot, value);
    }

    pub fn sprite_check_damage_to_and_from_link(&mut self, slot: usize) -> bool {
        self.sprite_check_damage_from_link(slot);
        self.sprite_check_damage_to_link(slot)
    }

    pub fn link_place_weapon_tink(&mut self) {
        if self.r8(0xfac) != 0 {
            return;
        }
        self.w8(0xfac, 5);
        let x = u16::from(self.r16(0x22) as u8) + u16::from(self.r8(0x45));
        self.w8(0xfad, x as u8);
        let y = u16::from(self.r16(0x20) as u8) + u16::from(self.r8(0x44)) + (x >> 8);
        self.w8(0xfae, y as u8);
        self.w8(0xb68, self.r8(0xee));
        let sound = self.hooks.link_calculate_sfx_pan(&self.ram) | 5;
        self.w8(0x12e, sound);
    }

    pub fn sprite_place_rupulse_spark_2(&mut self, slot: usize) {
        let x = self.sprite_get_x(slot).wrapping_sub(self.r16(0xe2));
        let y = self.sprite_get_y(slot).wrapping_sub(self.r16(0xe8));
        if x & !0xff != 0 || y & !0xff != 0 {
            return;
        }
        self.w8(0xfad, self.ram.sprite(slot).x_low());
        self.w8(0xfae, self.ram.sprite(slot).y_low());
        self.w8(0xfac, 5);
        self.w8(0xb68, self.ram.sprite(slot).floor());
    }

    pub fn sprite_place_weapon_tink(&mut self, slot: usize) {
        if self.r8(0xfac) == 0 {
            self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 5);
            self.sprite_place_rupulse_spark_2(slot);
        }
    }

    pub fn sprite_return_if_lifted_permissive(&mut self, slot: usize) -> bool {
        if self.r8(0x372) != 0 {
            return false;
        }
        if self.r8(0x2f4).wrapping_sub(1) != self.r8(0xfa0) {
            let mut hitbox = SpriteHitBox::default();
            self.link_setup_hit_box_conditional(&mut hitbox);
            self.sprite_setup_hit_box(slot, &mut hitbox);
            if Self::check_if_hit_boxes_overlap(&hitbox) {
                let value = (slot as u8).wrapping_add(1);
                self.w8(0xfb2, value);
                self.w8(0x314, value);
            }
            false
        } else {
            self.w8(0xf6, 0);
            self.ram.set_sprite_scratch_e(slot, 0);
            self.hooks.sprite_sfx2_with_pan(&mut self.ram, slot, 0x1d);
            self.ram
                .set_sprite_unknown4(slot, self.ram.sprite(slot).state());
            self.ram.set_sprite_state(slot, 10);
            self.ram.set_sprite_main_delay(slot, 16);
            self.ram.set_sprite_unknown3(slot, 0);
            self.ram.set_sprite_scratch_i(slot, 0);
            let direction = usize::from(self.sprite_direction_to_face_link(slot).0);
            self.w8(0x2f, SPRITE_RETURN_IF_LIFTED_DIRS[direction]);
            true
        }
    }

    pub fn sprite_return_if_lifted(&mut self, slot: usize) -> bool {
        if self.r8(0x11) | self.r8(0x3c) | self.r8(0xfc1) != 0
            || self.ram.sprite(slot).floor() != self.r8(0xee)
        {
            return false;
        }
        if (0..16)
            .rev()
            .any(|other| self.ram.sprite(other).state() == 10)
        {
            return false;
        }
        if !matches!(self.ram.sprite(slot).kind(), 0x0b | 0x4a)
            && self.ram.sprite(slot).velocity_x() | self.ram.sprite(slot).velocity_y() != 0
        {
            return false;
        }
        if self.r8(0x372) != 0 {
            return false;
        }
        self.sprite_return_if_lifted_permissive(slot)
    }

    pub fn sprite_check_if_lifted_permissive(&mut self, slot: usize) {
        self.sprite_return_if_lifted_permissive(slot);
    }
    pub fn check_if_hit_boxes_overlap(hitbox: &SpriteHitBox) -> bool {
        if hitbox.r8_xhi == 0x80 || hitbox.r10_spr_xhi == 0x80 {
            return false;
        }
        fn axis(a_low: u8, a_high: u8, a_size: u8, b_low: u8, b_high: u8, b_size: u8) -> bool {
            let mut t = i32::from(b_low) - i32::from(a_low);
            let r15 = (t + i32::from(b_size)) as u8;
            let r12 = b_high.wrapping_sub(a_high).wrapping_sub(u8::from(t < 0));
            t = i32::from(r12) + (((t & 0xff) + 0x80) >> 8);
            if t & 0xff != 0 {
                return t >= 0x100;
            }
            a_size.wrapping_add(b_size) >= r15
        }
        axis(
            hitbox.r1_ylo,
            hitbox.r9_yhi,
            hitbox.r3,
            hitbox.r5_spr_ylo,
            hitbox.r11_spr_yhi,
            hitbox.r7_spr_ysize,
        ) && axis(
            hitbox.r0_xlo,
            hitbox.r8_xhi,
            hitbox.r2,
            hitbox.r4_spr_xlo,
            hitbox.r10_spr_xhi,
            hitbox.r6_spr_xsize,
        )
    }
    pub fn garnish_get_x(&self, slot: usize) -> u16 {
        self.ram.garnish(slot).x()
    }
    pub fn garnish_get_y(&self, slot: usize) -> u16 {
        self.ram.garnish(slot).y()
    }
    fn garnish_set_x(&mut self, slot: usize, value: u16) {
        self.ram.garnish_mut(slot).set_x(value);
    }
    fn garnish_set_y(&mut self, slot: usize, value: u16) {
        self.ram.garnish_mut(slot).set_y(value);
    }
    pub fn garnish_sparkle_common(&mut self, slot: usize, shift: u8) {
        const CHAR: [u8; 4] = [0x83, 0xc7, 0x80, 0xb7];
        let frame = self.ram.garnish_countdown(slot) >> shift;
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let sprite = usize::from(self.ram.garnish_source_sprite(slot));
        let flags = (self.ram.sprite(sprite).oam_flags()
            | self.ram.sprite(sprite).object_priority())
            & 0xf0
            | 4;
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[usize::from(frame)],
            flags,
            0,
        );
    }

    pub fn garnish_dust_common(&mut self, slot: usize, shift: u8) {
        const CHAR: [u8; 3] = [0xdf, 0xcf, 0xa9];
        let frame = self.ram.garnish_countdown(slot) >> shift;
        self.w8(0xfb5, frame);
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[usize::from(frame)],
            0x24,
            0,
        );
    }

    pub fn garnish12_sparkle(&mut self, slot: usize) {
        self.garnish_sparkle_common(slot, 2);
    }

    pub fn garnish_simple_sparkle(&mut self, slot: usize) {
        self.garnish_sparkle_common(slot, 3);
    }

    pub fn garnish14_kak_kid_dash_dust(&mut self, slot: usize) {
        self.garnish_dust_common(slot, 2);
    }

    pub fn garnish_water_trail(&mut self, slot: usize) {
        self.garnish_dust_common(slot, 3);
    }

    pub fn garnish01_fire_snake_tail(&mut self, slot: usize) {
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let sprite = usize::from(self.ram.garnish_source_sprite(slot));
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            0x28,
            self.ram.sprite(sprite).oam_flags() | self.ram.sprite(sprite).object_priority(),
            2,
        );
    }

    pub fn garnish02_mothula_beam_trail(&mut self, slot: usize) {
        let sprite = usize::from(self.ram.garnish_source_sprite(slot));
        self.set_oam_plain(
            self.current_oam_index(),
            u16::from(self.ram.garnish_x_low(slot)).wrapping_sub(self.r16(0xe2)),
            u16::from(self.ram.garnish_y_low(slot)).wrapping_sub(self.r16(0xe8)),
            0xaa,
            self.ram.sprite(sprite).oam_flags() | self.ram.sprite(sprite).object_priority(),
            2,
        );
    }

    pub fn garnish04_laser_trail(&mut self, slot: usize) {
        const CHAR: [u8; 2] = [0xd2, 0xf3];
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[usize::from(self.ram.garnish_oam_flags(slot))],
            0x25,
            0,
        );
    }

    pub fn garnish06_zoro_trail(&mut self, slot: usize) {
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let sprite = usize::from(self.ram.garnish_source_sprite(slot));
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            0x75,
            self.ram.sprite(sprite).oam_flags() | self.ram.sprite(sprite).object_priority(),
            0,
        );
    }

    pub fn garnish07_babasu_flash(&mut self, slot: usize) {
        const CHAR: [u8; 4] = [0xa8, 0x8a, 0x86, 0x86];
        const FLAGS: [u8; 4] = [0x2d, 0x2c, 0x2c, 0x2c];
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let frame = usize::from(self.ram.garnish_countdown(slot) >> 3);
        self.set_oam_plain(self.current_oam_index(), x, y, CHAR[frame], FLAGS[frame], 2);
    }

    pub fn garnish08_kholdstare_trail(&mut self, slot: usize) {
        const XY: [i8; 3] = [-1, -1, 0];
        const CHAR: [u8; 3] = [0x9c, 0x9d, 0x8d];
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let frame = usize::from(self.ram.garnish_countdown(slot) >> 2);
        let sprite = usize::from(self.ram.garnish_source_sprite(slot));
        let offset = XY[frame] as i16 as u16;
        self.set_oam_plain(
            self.current_oam_index(),
            x.wrapping_add(offset),
            y.wrapping_add(offset),
            CHAR[frame],
            (self.ram.sprite(sprite).oam_flags() | self.ram.sprite(sprite).object_priority()) & !1,
            0,
        );
    }

    pub fn garnish09_lightning_trail(&mut self, slot: usize) {
        const CHAR: [u8; 8] = [0xcc, 0xec, 0xce, 0xee, 0xcc, 0xec, 0xce, 0xee];
        const FLAGS: [u8; 8] = [0x31, 0x31, 0x31, 0x31, 0x71, 0x71, 0x71, 0x71];
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let frame = usize::from(self.ram.garnish_source_sprite(slot));
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[frame].wrapping_sub(if self.r8(0x48e) == 0x20 { 0x80 } else { 0 }),
            self.r8(0x1a).wrapping_shl(1) & 0xe | FLAGS[frame],
            2,
        );
        self.garnish_check_player_collision(slot, x as u8, y as u8);
    }

    pub fn garnish0a_cannon_smoke(&mut self, slot: usize) {
        const CHAR: [u8; 2] = [0x8a, 0x86];
        const FLAGS: [u8; 4] = [0x20, 0x10, 0x30, 0x30];
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let sprite = usize::from(self.ram.garnish_source_sprite(slot));
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[usize::from(self.ram.garnish_countdown(slot) >> 3)],
            FLAGS[sprite] | 4,
            2,
        );
    }

    pub fn garnish0e_trinexx_fire_breath(&mut self, slot: usize) {
        const CHAR: [u8; 4] = [0x83, 0xc7, 0x80, 0x9d];
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let sprite = usize::from(self.ram.garnish_source_sprite(slot));
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[usize::from(self.ram.garnish_countdown(slot) >> 3)],
            (self.ram.sprite(sprite).oam_flags() | self.ram.sprite(sprite).object_priority())
                & 0xf0
                | 0x0e,
            0,
        );
    }

    pub fn garnish0f_blind_laser_trail(&mut self, slot: usize) {
        const CHAR: [u8; 4] = [0x61, 0x71, 0x70, 0x60];
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let sprite = usize::from(self.ram.garnish_source_sprite(slot));
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[usize::from(self.ram.garnish_oam_flags(slot).wrapping_sub(7))],
            self.ram.sprite(sprite).oam_flags() | self.ram.sprite(sprite).object_priority(),
            0,
        );
    }

    pub fn garnish10_ganon_bat_flame(&mut self, slot: usize) {
        const INDEX: [u8; 32] = [
            7, 6, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5,
            4, 5, 4,
        ];
        const CHAR: [u8; 7] = [0xac, 0xac, 0x66, 0x66, 0x8e, 0xa0, 0xa2];
        const FLAGS: [u8; 7] = [1, 0x41, 1, 0x41, 0, 0, 0];
        if self.ram.garnish_countdown(slot) == 8 {
            self.ram.set_garnish_kind(slot, 0x11);
        }
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let frame = usize::from(INDEX[usize::from(self.ram.garnish_countdown(slot) >> 3)]);
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[frame],
            FLAGS[frame] | 0x22,
            2,
        );
        self.garnish_check_player_collision(slot, x as u8, y as u8);
    }

    pub fn garnish11_withering_ganon_bat_flame(&mut self, slot: usize) {
        if self.r8(0x11) | self.r8(0xfc1) == 0 {
            let y = self.garnish_get_y(slot).wrapping_sub(1);
            self.garnish_set_y(slot, y);
        }
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let index = self.current_oam_index();
        self.set_oam_plain(index, x, y, 0xa4, 0x22, 0);
        self.set_oam_plain(index + 1, x.wrapping_add(8), y, 0xa5, 0x22, 0);
    }

    pub fn garnish15_arrghus_splash(&mut self, slot: usize) {
        const X: [i8; 8] = [-12, 20, -10, 10, -8, 8, -4, 4];
        const Y: [i8; 8] = [-4, -4, -2, -2, 0, 0, 0, 0];
        const CHAR: [u8; 8] = [0xae, 0xae, 0xae, 0xae, 0xae, 0xae, 0xac, 0xac];
        const FLAGS: [u8; 8] = [0x34, 0x74, 0x34, 0x74, 0x34, 0x74, 0x34, 0x74];
        const EXT: [u8; 8] = [0, 0, 2, 2, 2, 2, 2, 2];
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let group = usize::from((self.ram.garnish_countdown(slot) >> 1) & 6);
        let oam_index = self.current_oam_index();
        for output in (0..=1).rev() {
            let frame = output + group;
            self.set_oam_plain(
                oam_index + (1 - output),
                x.wrapping_add(X[frame] as i16 as u16),
                y.wrapping_add(Y[frame] as i16 as u16),
                CHAR[frame],
                FLAGS[frame],
                EXT[frame],
            );
        }
    }

    pub fn garnish03_falling_tile(&mut self, slot: usize) {
        const XY: [u8; 5] = [4, 0, 0, 0, 0];
        const CHAR: [u8; 5] = [0x80, 0xcc, 0xcc, 0xea, 0xca];
        const FLAGS: [u8; 5] = [0x30, 0x31, 0x31, 0x31, 0x31];
        const EXT: [u8; 5] = [0, 2, 2, 2, 2];
        let countdown = self.ram.garnish_countdown(slot);
        let mut value = countdown;
        if countdown == 0x1e {
            value = self.r8(0x11) | self.r8(0xfc1);
            if value == 0 {
                let x = self.garnish_get_x(slot);
                let y = self.garnish_get_y(slot).wrapping_sub(16);
                self.hooks
                    .dungeon_update_tilemap_with_common_tile(&mut self.ram, x, y, 4);
            }
        }
        let frame = usize::from(value >> 3);
        let x = self
            .garnish_get_x(slot)
            .wrapping_add(u16::from(XY[frame]))
            .wrapping_sub(self.r16(0xe2));
        let y = self
            .garnish_get_y(slot)
            .wrapping_add(u16::from(XY[frame]))
            .wrapping_sub(self.r16(0xe8));
        if x < 256 && y < 256 {
            self.set_oam_plain(
                self.current_oam_index(),
                x,
                y.wrapping_sub(16),
                CHAR[frame],
                FLAGS[frame],
                EXT[frame],
            );
        }
    }

    pub fn garnish0c_trinexx_ice_breath(&mut self, slot: usize) {
        const CHAR: [u8; 12] = [
            0xe8, 0xe8, 0xe6, 0xe6, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4,
        ];
        const FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        let countdown = self.ram.garnish_countdown(slot);
        if countdown == 0x50 && self.r8(0x11) | self.r8(0xfc1) == 0 {
            let x = self.garnish_get_x(slot);
            let y = self.garnish_get_y(slot).wrapping_sub(16);
            self.hooks
                .dungeon_update_tilemap_with_common_tile(&mut self.ram, x, y, 18);
        }
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        self.set_oam_plain(
            self.current_oam_index(),
            x,
            y,
            CHAR[usize::from(countdown >> 4)],
            FLAGS[usize::from((countdown >> 2) & 3)] | 0x35,
            2,
        );
    }

    pub fn garnish13_pyramid_debris(&mut self, slot: usize) {
        let y = (i32::from(self.ram.garnish_y_low(slot)) << 8)
            + i32::from(self.ram.garnish_y_fraction(slot))
            + (i32::from(self.ram.garnish_y_velocity(slot) as i8) << 4);
        self.ram.set_garnish_y_fraction(slot, y as u8);
        self.ram.set_garnish_y_low(slot, (y >> 8) as u8);
        let x = (i32::from(self.ram.garnish_x_low(slot)) << 8)
            + i32::from(self.ram.garnish_x_fraction(slot))
            + (i32::from(self.ram.garnish_x_velocity(slot) as i8) << 4);
        self.ram.set_garnish_x_fraction(slot, x as u8);
        self.ram.set_garnish_x_low(slot, (x >> 8) as u8);
        self.ram
            .set_garnish_y_velocity(slot, self.ram.garnish_y_velocity(slot).wrapping_add(3));
        let screen_x = self
            .ram
            .garnish_x_low(slot)
            .wrapping_sub(self.r16(0xe2) as u8);
        if screen_x >= 248 {
            self.ram.set_garnish_kind(slot, 0);
            return;
        }
        // The C routine writes the OAM x byte before testing whether y is visible.
        // Preserve that partial write when the debris exits the bottom of the screen.
        let oam_index = self.current_oam_index();
        self.ram.write_u8(oam::OAM_BUFFER + oam_index * 4, screen_x);
        let screen_y = self
            .ram
            .garnish_y_low(slot)
            .wrapping_sub(self.r16(0xe8) as u8);
        if screen_y >= 240 {
            self.ram.set_garnish_kind(slot, 0);
            return;
        }
        self.set_oam_plain(
            self.current_oam_index(),
            u16::from(screen_x),
            u16::from(screen_y),
            0x5c,
            self.r8(0x1a).wrapping_shl(3) & 0xc0 | 0x34,
            0,
        );
    }

    pub fn garnish_execute_upper_slots(&mut self) {
        self.hooks.handle_screen_flash(&mut self.ram);
        if self.ram.garnish_active_marker() != 0 {
            for slot in (15..30).rev() {
                self.garnish_execute_single(slot);
            }
        }
    }

    pub fn garnish_execute_lower_slots(&mut self) {
        if self.ram.garnish_active_marker() != 0 {
            for slot in (0..15).rev() {
                self.garnish_execute_single(slot);
            }
        }
    }

    pub fn garnish_execute_single(&mut self, slot: usize) {
        self.w8(0xfa0, slot as u8);
        let kind = self.ram.garnish_kind(slot);
        if kind == 0 {
            return;
        }
        if (kind == 5 || self.r8(0x11) | self.r8(0xfc1) == 0)
            && self.ram.garnish_mut(slot).tick_lifetime()
        {
            return;
        }
        let bytes = GARNISH_OAM_MEM_SIZE[usize::from(self.ram.garnish_kind(slot))];
        if self.r8(0xfb3) != 0 {
            if self.ram.garnish_floor(slot) != 0 {
                self.oam_allocate_from_region_f(bytes);
            } else {
                self.oam_allocate_from_region_d(bytes);
            }
        } else {
            self.oam_allocate_from_region_a(bytes);
        }
        match GARNISH_FUNCS[usize::from(self.ram.garnish_kind(slot) - 1)] {
            1 => self.garnish01_fire_snake_tail(slot),
            2 => self.garnish02_mothula_beam_trail(slot),
            3 => self.garnish03_falling_tile(slot),
            4 => self.garnish04_laser_trail(slot),
            5 => self.garnish_simple_sparkle(slot),
            6 => self.garnish06_zoro_trail(slot),
            7 => self.garnish07_babasu_flash(slot),
            8 => self.garnish08_kholdstare_trail(slot),
            9 => self.garnish09_lightning_trail(slot),
            10 => self.garnish0a_cannon_smoke(slot),
            11 => self.garnish_water_trail(slot),
            12 => self.garnish0c_trinexx_ice_breath(slot),
            0 => {}
            14 => self.garnish0e_trinexx_fire_breath(slot),
            15 => self.garnish0f_blind_laser_trail(slot),
            16 => self.garnish10_ganon_bat_flame(slot),
            17 => self.garnish11_withering_ganon_bat_flame(slot),
            18 => self.garnish12_sparkle(slot),
            19 => self.garnish13_pyramid_debris(slot),
            20 => self.garnish14_kak_kid_dash_dust(slot),
            21 => self.garnish15_arrghus_splash(slot),
            22 => self.garnish16_thrown_item_debris(slot),
            _ => unreachable!(),
        }
    }

    pub fn garnish16_thrown_item_debris(&mut self, slot: usize) {
        let Some((x, y)) = self.garnish_return_if_prep_fails(slot) else {
            return;
        };
        let oam_flags = self.ram.garnish_oam_flags(slot);
        if self.r8(0xfc6) >= 3 {
            return;
        }
        let debris = self.ram.garnish_source_sprite(slot);
        if debris == 3 {
            self.scatter_debris_draw(slot, x, y);
            return;
        }
        // The C renderer reuses this WRAM scratch byte for the debris kind.
        self.w8(0xfb5, debris);
        let mut base = usize::from(((self.ram.garnish_countdown(slot) >> 2) ^ 7) << 2);
        if debris == 4 || debris == 2 && self.r8(0x1b) == 0 {
            base += 0x20;
        }
        let oam = self.current_oam_index();
        for i in (0..=3).rev() {
            let j = base + i;
            let character = if debris == 0 {
                0x4e
            } else if debris >= 0x80 {
                0xf2
            } else {
                SCATTER_DEBRIS_DRAW_CHAR[j]
            };
            oam::set_helper1(
                &mut self.ram,
                oam + (3 - i),
                x.wrapping_add_signed(SCATTER_DEBRIS_DRAW_X[j]),
                y.wrapping_add_signed(i16::from(SCATTER_DEBRIS_DRAW_Y[j])) as u8,
                character,
                SCATTER_DEBRIS_DRAW_FLAGS[j] | oam_flags,
                0,
            );
        }
    }

    pub fn scatter_debris_draw(&mut self, slot: usize, x: u16, y: u16) {
        if self.ram.garnish_countdown(slot) == 16 {
            self.ram.set_garnish_kind(slot, 0);
        }
        let base = usize::from((self.ram.garnish_countdown(slot) & 0x0f) >> 2) * 3;
        let oam = self.current_oam_index();
        for i in (0..=2).rev() {
            let j = base + i;
            oam::set_helper1(
                &mut self.ram,
                oam + (2 - i),
                x.wrapping_add_signed(i16::from(SCATTER_DEBRIS_DRAW_X2[j])),
                y.wrapping_add_signed(i16::from(SCATTER_DEBRIS_DRAW_Y2[j])) as u8,
                SCATTER_DEBRIS_DRAW_CHAR2[j],
                SCATTER_DEBRIS_DRAW_FLAGS2[j] | 0x22,
                0,
            );
        }
    }
    pub fn sprite_spawn_simple_sparkle_garnish_ex(
        &mut self,
        slot: usize,
        x: u16,
        y: u16,
        limit: usize,
    ) -> i32 {
        let Some(garnish) = self.hooks.garnish_alloc_limit(&mut self.ram, limit) else {
            self.w8(15, 0xff);
            return -1;
        };
        self.ram.set_garnish_kind(garnish, 5);
        self.ram.set_garnish_active_marker(5);
        self.garnish_set_x(garnish, self.sprite_get_x(slot).wrapping_add(x));
        self.garnish_set_y(
            garnish,
            self.sprite_get_y(slot)
                .wrapping_add(y)
                .wrapping_sub(u16::from(self.ram.sprite(slot).z()))
                .wrapping_add(16),
        );
        self.ram.set_garnish_countdown(garnish, 31);
        self.ram.set_garnish_source_sprite(garnish, slot as u8);
        self.ram
            .set_garnish_floor(garnish, self.ram.sprite(slot).floor());
        self.w8(15, garnish as u8);
        garnish as i32
    }
    fn alloc_overlord(&self) -> Option<usize> {
        (0..8).rev().find(|&i| self.arr(OVERLORD_TYPE, i) == 0)
    }
    fn overworld_alloc_sprite(&self, kind: u8) -> Option<usize> {
        let top = if kind == 0x58 {
            4
        } else if kind == 0xd0 {
            5
        } else if matches!(kind, 0xeb | 0x53 | 0xf3) {
            14
        } else {
            13
        };
        (0..=top).rev().find(|&i| {
            self.ram.sprite(i).state() == 0
                || self.ram.sprite(i).kind() == 0x41 && self.ram.sprite(i).scratch_c() != 0
        })
    }
}

const OVERLORD_TYPE: usize = 0xb00;

pub const ABSORB_BIG_KEY: [u8; 2] = [0x40, 0x20];
pub const ABSORPTION_SFX: [u8; 15] = [
    0xb, 0xa, 0xa, 0xa, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0x2f, 0x2f, 0xb,
];

pub use crate::oam::{
    REGION_FALLBACKS as OAM_GET_BUFFER_POS_TAB1, REGION_LIMITS as OAM_GET_BUFFER_POS_TAB0,
};
pub const SPRITE2_RETURN_IF_RECOILING_MASKS: [u8; 6] = [3, 1, 0, 0, 0xc, 3];
pub const SPRITE_HITBOX_X_LO: [i8; 32] = [
    2, 3, 0, -3, -6, 0, 2, -8, 0, -4, -8, 0, -8, -16, 2, 2, 2, 2, 2, -8, 2, 2, -16, -8, -12, 4, -4,
    -12, 5, -32, -2, 4,
];
pub const SPRITE_HITBOX_X_HI: [i8; 32] = [
    0, 0, 0, -1, -1, 0, 0, -1, 0, -1, -1, 0, -1, -1, 0, 0, 0, 0, 0, -1, 0, 0, -1, -1, -1, 0, -1,
    -1, 0, -1, -1, 0,
];
pub const SPRITE_HITBOX_X_SIZE: [u8; 32] = [
    12, 1, 16, 20, 20, 8, 4, 32, 48, 24, 32, 32, 32, 48, 12, 12, 60, 124, 12, 32, 4, 12, 48, 32,
    40, 8, 24, 24, 5, 80, 4, 8,
];
pub const SPRITE_HITBOX_Y_LO: [i8; 32] = [
    0, 3, 4, -4, -8, 2, 0, -16, 12, -4, -8, 0, -10, -16, 2, 2, 2, 2, -3, -12, 2, 10, 0, -12, 16, 4,
    -4, -12, 3, -16, -8, 10,
];
pub const SPRITE_HITBOX_Y_HI: [i8; 32] = [
    0, 0, 0, -1, -1, 0, 0, -1, 0, -1, -1, 0, -1, -1, 0, 0, 0, 0, -1, -1, 0, 0, 0, -1, 0, 0, -1, -1,
    0, -1, -1, 0,
];
pub const SPRITE_HITBOX_Y_SIZE: [u8; 32] = [
    14, 1, 16, 21, 24, 4, 8, 40, 20, 24, 40, 29, 36, 48, 60, 124, 12, 12, 17, 28, 4, 2, 28, 20, 10,
    4, 24, 16, 5, 48, 8, 12,
];
pub const SPRITE_DAMAGE_TAB2: [u8; 4] = [6, 4, 0, 0];
pub const SPRITE_DAMAGE_TAB3: [u8; 4] = [4, 6, 0, 2];
pub const SPRITE_FUNC21_SFX: [u8; 9] = [0x1f, 0x1f, 0x1e, 0x1e, 0x1e, 0x1f, 0x1f, 0x1f, 0x1f];
pub const SPARKLE_GARNISH_XY: [i8; 4] = [-4, 12, 3, 8];
pub const SPRITE_HELD_Z_FOR_FRAME: [u8; 6] = [3, 2, 1, 3, 2, 1];
pub const SPRITE_HELD_X: [i8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, -13, -10, -5, 0, 13, 10, 5, 0];
pub const SPRITE_HELD_Z: [u8; 16] = [13, 14, 15, 16, 0, 10, 22, 16, 8, 11, 14, 16, 8, 11, 14, 16];
pub const SPRITE_HELD_THROW_X_VEL: [i8; 4] = [0, 0, -62, 63];
pub const SPRITE_HELD_THROW_Y_VEL: [i8; 4] = [-62, 63, 0, 0];
pub const SPRITE_HELD_THROW_Z_VEL: [u8; 4] = [4, 4, 4, 4];
pub const SPRITE_STUNNED_MAIN_FUNC1_MASKS: [u8; 7] = [0x7f, 0xf, 3, 1, 0, 0, 0];
pub const SPRITE_FUNC7_TAB: [u8; 4] = [8, 4, 2, 1];
pub const SPRITE_FUNC5_X: [i8; 54] = [
    8, 8, 2, 14, 8, 8, -2, 10, 8, 8, 1, 14, 4, 4, 4, 4, 4, 4, -2, 10, 8, 8, -25, 40, 8, 8, 2, 14,
    8, 8, -8, 23, 8, 8, -20, 36, 8, 8, -1, 16, 8, 8, -1, 16, 8, 8, -8, 24, 8, 8, -8, 24, 8, 3,
];
pub const SPRITE_FUNC5_Y: [i8; 54] = [
    6, 20, 13, 13, 0, 8, 4, 4, 1, 14, 8, 8, 4, 4, 4, 4, -2, 10, 4, 4, -25, 40, 8, 8, 3, 16, 10, 10,
    -8, 25, 8, 8, -20, 36, 8, 8, -1, 16, 8, 8, 14, 3, 8, 8, -8, 24, 8, 8, -8, 32, 8, 8, 12, 4,
];
pub const SPRITE_SIMPLIFIED_TILE_ATTR: [u8; 256] = [
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];
pub const SPRITE_FUNC5_TAB3: [i8; 256] = [
    0, 1, 2, 3, 2, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1,
    1, 1, 1, 0, 0, 0, 1, 2, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1,
    1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1,
];
pub const SLOPED_TILE: [i8; 32] = [
    7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0,
];
pub const SPRITE_FUNC1_TAB: [u8; 8] = [15, 15, 24, 15, 15, 19, 15, 15];
pub const SPRITE_FUNC1_TAB2: [u8; 8] = [6, 6, 6, 12, 6, 6, 6, 15];
pub const SCATTER_DEBRIS_DRAW_X: [i16; 64] = [
    0, 8, 0, 8, -2, 9, -1, 9, -4, 9, -1, 10, -6, 9, -1, 12, -7, 9, -2, 13, -9, 9, -3, 14, -4, -4,
    9, 15, -3, -3, -3, 9, -4, 4, 6, 10, -1, 4, 6, 7, 0, 2, 4, 7, 1, 1, 5, 7, 0, -2, 8, 9, -1, -6,
    9, 10, -2, -7, 12, 11, -3, -9, 4, 6,
];
pub const SCATTER_DEBRIS_DRAW_Y: [i8; 64] = [
    0, 0, 8, 8, 0, -1, 10, 10, 0, -3, 11, 7, 1, -4, 12, 8, 1, -4, 13, 9, 2, -4, 16, 10, 14, 14, -4,
    11, 16, 16, 16, -1, 2, -5, 5, 1, 3, -7, 8, 2, 4, -8, 4, 10, -9, 4, 4, 12, -10, 4, 8, 14, -12,
    4, 8, 15, -15, 3, 8, 17, -17, 1, 18, 15,
];
pub const SCATTER_DEBRIS_DRAW_CHAR: [u8; 64] = [
    0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58,
    0x48, 0x58, 0x58, 0x58, 0x48, 0x58, 0x58, 0x48, 0x48, 0x48, 0x58, 0x48, 0x48, 0x48, 0x48, 0x48,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59,
];
pub const SCATTER_DEBRIS_DRAW_FLAGS: [u8; 64] = [
    0x80, 0, 0x80, 0x40, 0x80, 0x40, 0x80, 0, 0, 0xc0, 0, 0x80, 0x80, 0x40, 0x80, 0, 0x80, 0xc0, 0,
    0x80, 0, 0, 0x80, 0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40,
    0, 0x40, 0x40, 0, 0, 0x80, 0, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0, 0, 0x40, 0, 0, 0,
];
pub const SCATTER_DEBRIS_DRAW_X2: [i8; 12] = [-8, 8, 16, -5, 8, 15, -1, 7, 11, 1, 3, 8];
pub const SCATTER_DEBRIS_DRAW_Y2: [i8; 12] = [7, 2, 12, 9, 2, 10, 11, 2, 11, 7, 3, 8];
pub const SCATTER_DEBRIS_DRAW_CHAR2: [u8; 12] = [
    0xe2, 0xe2, 0xe2, 0xe2, 0xf2, 0xf2, 0xf2, 0xe2, 0xe2, 0xf2, 0xe2, 0xe2,
];
pub const SCATTER_DEBRIS_DRAW_FLAGS2: [u8; 12] = [0, 0, 0, 0, 0x80, 0x40, 0, 0x80, 0x40, 0, 0, 0];
pub const SPRITE_FUNC14_DAMAGE: [u8; 12] = [1, 2, 3, 4, 2, 3, 4, 5, 1, 1, 2, 3];
pub const ENEMY_DAMAGES: [u8; 128] = [
    0, 1, 32, 255, 252, 251, 0, 0, 0, 2, 64, 4, 0, 0, 0, 0, 0, 4, 64, 2, 3, 0, 0, 0, 0, 8, 64, 4,
    0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 4, 64, 16, 0, 0, 0, 0, 0,
    255, 64, 255, 252, 251, 0, 0, 0, 4, 64, 255, 252, 251, 32, 0, 0, 100, 24, 100, 0, 0, 0, 0, 0,
    249, 250, 255, 100, 0, 0, 0, 0, 8, 64, 253, 4, 16, 0, 0, 0, 8, 64, 254, 4, 0, 0, 0, 0, 16, 64,
    253, 0, 0, 0, 0, 0, 254, 64, 16, 0, 0, 0, 0, 0, 32, 64, 255, 0, 0, 0, 250,
];
pub const SPRITE_INIT_FLAGS2: [u8; 243] = [
    1, 2, 1, 0x82, 0x81, 0x84, 0x84, 0x84, 2, 0xf, 2, 1, 0x20, 3, 4, 0x84, 1, 5, 4, 1, 0x80, 4,
    0xa2, 0x83, 4, 2, 0x82, 0x62, 0x82, 0x80, 0x80, 0x85, 1, 0xa5, 3, 4, 4, 0x83, 2, 1, 0x82, 0xa2,
    0xa2, 0xa3, 0xaa, 0xa3, 0xa4, 0x82, 0x82, 0x83, 0x82, 0x80, 0x82, 0x82, 0xa5, 0x80, 0xa4, 0x82,
    0x81, 0x82, 0x82, 0x82, 0x81, 6, 8, 8, 8, 8, 6, 8, 8, 8, 6, 7, 7, 2, 2, 0x22, 1, 1, 0x20, 0x82,
    7, 0x85, 0xf, 0x21, 5, 0x83, 2, 1, 1, 1, 1, 7, 7, 7, 7, 0, 0x85, 0x83, 3, 0xa4, 0, 0, 0, 0, 9,
    4, 0xa0, 0, 1, 0, 0, 3, 0x8b, 0x86, 0xc2, 0x82, 0x81, 4, 0x82, 0x21, 6, 3, 1, 3, 3, 3, 0, 0, 4,
    5, 5, 3, 1, 2, 0, 0, 0, 2, 7, 0, 1, 1, 0x87, 6, 0, 0x83, 2, 0x22, 0x22, 0x22, 0x22, 4, 3, 5, 1,
    1, 4, 1, 2, 8, 8, 0x80, 0x21, 3, 3, 3, 2, 2, 8, 0x8f, 0xa1, 0x81, 0x80, 0x80, 0x80, 0x80, 0xa1,
    0x80, 0x81, 0x81, 0x86, 0x81, 0x82, 0x82, 0x80, 0x80, 0x83, 6, 0, 0, 5, 4, 6, 5, 2, 0, 0, 5, 4,
    4, 7, 0xb, 0xc, 0xc, 6, 6, 3, 0xa4, 4, 0x82, 0x81, 0x83, 0x10, 0x10, 0x81, 0x82, 0x82, 0x82,
    0x83, 0x83, 0x83, 0x81, 0x82, 0x83, 0x83, 0x81, 0x82, 0x81, 0x82, 0xa0, 0xa1, 0xa3, 0xa1, 0xa1,
    0xa1, 0x83, 0x85, 0x83, 0x83, 0x83, 0x83,
];
pub const SPRITE_INIT_HEALTH: [u8; 243] = [
    12, 6, 255, 3, 3, 3, 3, 3, 2, 12, 4, 255, 0, 3, 12, 2, 0, 20, 4, 4, 0, 255, 0, 2, 3, 8, 0, 0,
    0, 0, 0, 0, 8, 3, 8, 2, 2, 0, 3, 255, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 3, 0, 3, 3, 3, 0, 3, 0, 0,
    0, 0, 3, 2, 255, 2, 6, 4, 8, 6, 8, 6, 4, 8, 8, 8, 4, 4, 2, 2, 2, 255, 8, 255, 48, 16, 8, 8,
    255, 2, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 4, 4, 255, 255, 255, 255, 16, 3, 0,
    2, 4, 1, 255, 4, 255, 0, 0, 0, 0, 255, 0, 0, 96, 255, 24, 255, 255, 255, 3, 4, 255, 16, 8, 8,
    0, 255, 32, 32, 32, 32, 32, 8, 8, 4, 8, 64, 48, 255, 2, 255, 255, 255, 255, 16, 4, 2, 4, 4, 8,
    8, 8, 16, 64, 64, 8, 4, 8, 4, 4, 8, 12, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    128, 48, 255, 255, 255, 255, 8, 0, 0, 0, 32, 0, 8, 5, 40, 40, 40, 90, 16, 24, 64, 0, 4, 0, 0,
    255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const SPRITE_INIT_BUMP_DAMAGE: [u8; 243] = [
    0x83, 0x83, 0x81, 2, 2, 2, 2, 2, 1, 0x13, 1, 1, 1, 1, 8, 1, 1, 8, 5, 3, 0x40, 4, 0, 2, 3, 0x85,
    0, 1, 0, 0x40, 0, 0, 6, 0, 5, 3, 1, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
    0, 0, 0, 0, 0, 0, 2, 2, 0, 1, 1, 3, 1, 3, 1, 1, 3, 3, 3, 1, 3, 1, 1, 1, 1, 1, 1, 0x11, 0x14, 1,
    1, 2, 5, 0, 0, 4, 4, 8, 8, 8, 8, 4, 0, 4, 3, 2, 2, 2, 2, 2, 3, 1, 0, 0, 1, 0x80, 5, 1, 0, 0, 0,
    0x40, 0, 4, 0, 0, 0x14, 4, 6, 4, 4, 4, 4, 3, 4, 4, 4, 1, 4, 4, 0x15, 5, 4, 5, 0x15, 0x15, 3, 5,
    0, 5, 0x15, 5, 5, 6, 6, 6, 6, 5, 3, 6, 5, 5, 3, 3, 3, 6, 0x17, 0x15, 0x15, 5, 5, 1, 0x85, 0x83,
    5, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x17, 0x17, 5, 5, 5, 4, 3, 2, 0x10, 0,
    6, 0, 5, 7, 0x17, 0x17, 0x17, 0x15, 7, 6, 0x10, 0, 3, 3, 0, 0x19, 0x19, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const SPRITE_INIT_FLAGS3: [u8; 243] = [
    0x19, 0xb, 0x1b, 0x4b, 0x41, 0x41, 0x41, 0x4d, 0x1d, 1, 0x1d, 0x19, 0x8d, 0x1b, 9, 0x9d, 0x3d,
    1, 9, 0x11, 0x40, 1, 0x4d, 0x19, 7, 0x1d, 0x59, 0x80, 0x4d, 0x40, 1, 0x49, 0x1b, 0x41, 3, 0x13,
    0x15, 0x41, 0x18, 0x1b, 0x41, 0x47, 0xf, 0x49, 0x4b, 0x4d, 0x41, 0x47, 0x49, 0x4d, 0x49, 0x40,
    0x4d, 0x47, 0x49, 0x41, 0x74, 0x47, 0x5b, 0x58, 0x51, 0x49, 0x1d, 0x5d, 3, 0x19, 0x1b, 0x17,
    0x19, 0x17, 0x19, 0x1b, 0x17, 0x17, 0x17, 0x1b, 0xd, 9, 0x19, 0x19, 0x49, 0x5d, 0x5b, 0x49,
    0xd, 3, 0x13, 0x41, 0x1b, 0x5b, 0x5d, 0x43, 0x43, 0x4d, 0x4d, 0x4d, 0x4d, 0x4d, 0x49, 1, 0,
    0x41, 0x4d, 0x4d, 0x4d, 0x4d, 0x1d, 9, 0xc4, 0xd, 0xd, 9, 3, 3, 0x4b, 0x47, 0x47, 0x49, 0x49,
    0x41, 0x47, 0x36, 0x8b, 0x49, 0x1d, 0x49, 0x43, 0x43, 0x43, 0xb, 0x41, 0xd, 7, 0xb, 0x1d, 0x43,
    0xd, 0x43, 0xd, 0x1d, 0x4d, 0x4d, 0x1b, 0x1b, 0xa, 0xb, 0, 5, 0xd, 1, 1, 1, 1, 0xb, 5, 1, 1, 1,
    7, 0x17, 0x19, 0xd, 0xd, 0x80, 0x4d, 0x19, 0x17, 0x19, 0xb, 9, 0xd, 0x4a, 0x12, 0x49, 0xc3,
    0xc3, 0xc3, 0xc3, 0x76, 0x40, 0x59, 0x41, 0x58, 0x4f, 0x73, 0x5b, 0x44, 0x41, 0x51, 0xa, 0xb,
    0xb, 0x4b, 0, 0x40, 0x5b, 0xd, 0, 0, 0xd, 0x4b, 0xb, 0x59, 0x41, 0xb, 0xd, 1, 0xd, 0xd, 0,
    0x50, 0x4c, 0x44, 0x51, 1, 1, 0xf2, 0xf8, 0xf4, 0xf2, 0xd4, 0xd4, 0xd4, 0xf8, 0xf8, 0xf4, 0xf4,
    0xd8, 0xf8, 0xd8, 0xdf, 0xc8, 0x69, 0xc1, 0xd2, 0xd2, 0xdc, 0xc7, 0xc1, 0xc7, 0xc7, 0xc7, 0xc1,
];
pub const SPRITE_INIT_FLAGS4: [u8; 243] = [
    0, 0, 0, 0x43, 0x43, 0x43, 0x43, 0x43, 0, 0, 0, 0, 0x1c, 0, 0, 2, 1, 3, 0, 0, 3, 0xc0, 7, 0, 0,
    0, 7, 0x45, 0x43, 0, 0x40, 0xd, 0, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 0xd, 7, 7, 7, 7, 3,
    7, 7, 7, 0x40, 3, 7, 0xd, 0, 7, 7, 0, 0, 9, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
    0x12, 0x12, 0x12, 0, 0, 0, 0, 0x80, 0x12, 9, 9, 0, 0x40, 0, 0xc, 0, 0, 0, 0x40, 0x40, 0x10,
    0x10, 0x2e, 0x2e, 0x40, 0x1e, 0x53, 0, 0xa, 0, 0, 0, 0, 0x12, 0x12, 0x40, 0, 0, 0x40, 0x19, 0,
    0, 0xa, 0xd, 0xa, 0xa, 0x80, 0xa, 0x41, 0, 0x40, 0, 0x49, 0, 0, 0xc0, 0, 0x40, 0, 0, 0x40, 0,
    0, 9, 0x80, 0xc0, 0, 0x40, 0, 0, 0x80, 0, 0, 0x18, 0x5a, 0, 0xd4, 0xd4, 0xd4, 0xd4, 0, 0x40, 0,
    0x80, 0x80, 0x40, 0x40, 0x40, 0, 9, 0x1d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xa, 0x1b, 0x1b, 0x1b,
    0x1b, 0x41, 0, 3, 7, 7, 3, 0xa, 0, 1, 0xa, 0xa, 9, 0, 0, 0, 0, 9, 0, 0, 0x40, 0x40, 0, 0, 0, 0,
    0x89, 0x80, 0x80, 0, 0x1c, 0, 0x40, 0, 0, 0x1c, 7, 3, 3, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
    0x44, 0x44, 0x44, 0x44, 0x44, 0x43, 0x44, 0x43, 0x40, 0xc0, 0xc0, 0xc7, 0xc3, 0xc3, 0xc0, 0x1b,
    8, 0x1b, 0x1b, 0x1b, 3,
];
pub const SPRITE_INIT_FLAGS: [u8; 243] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0xa, 0, 1, 0x30, 0, 0, 0x20, 0x10, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 8,
    0x20, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0x68, 0x60, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0, 0,
    0, 0, 0, 0, 2, 2, 2, 0, 0, 0x70, 0, 0, 0, 0x90, 0x90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0x60, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0, 0, 2, 0, 0, 0x70, 0, 0, 0, 0, 0, 0, 0, 0,
    0xb0, 0, 0xc2, 0, 0x20, 0, 2, 0, 0, 0, 0, 0, 2, 0, 0xb0, 0, 0, 0, 0, 0, 0, 0, 0xa0, 0xa0, 0, 0,
    0, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xc2, 0,
    0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0xa, 0xa, 0x10, 0x10,
    0x10, 0x10, 0, 0, 0, 0x10, 0x10, 0x10, 0x10, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const SPRITE_INIT_FLAGS5: [u8; 243] = [
    0x83, 0x96, 0x84, 0x80, 0x80, 0x80, 0x80, 0x80, 2, 0, 2, 0x80, 0xa0, 0x83, 0x97, 0x80, 0x80,
    0x94, 0x91, 7, 0, 0x80, 0, 0x80, 0x92, 0x96, 0x80, 0xa0, 0, 0, 0, 0x80, 4, 0x80, 0x82, 6, 6, 0,
    0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    0x80, 0x80, 0x80, 0x80, 0x80, 0, 0, 0x80, 0x80, 0x90, 0x80, 0x91, 0x91, 0x91, 0x97, 0x91, 0x95,
    0x95, 0x93, 0x97, 0x14, 0x91, 0x92, 0x81, 0x82, 0x82, 0x80, 0x85, 0x80, 0x80, 0x80, 4, 4, 0x80,
    0x91, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0x80, 0x80, 0x82, 0x8a, 0x80, 0x80,
    0x80, 0x80, 0x92, 0x91, 0x80, 0x82, 0x81, 0x81, 0x80, 0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    0x80, 0x80, 0x80, 0x80, 0x97, 0x80, 0x80, 0x80, 0x80, 0xc2, 0x80, 0x15, 0x15, 0x17, 6, 0, 0x80,
    0, 0xc0, 0x13, 0x40, 0, 2, 6, 0x10, 0x14, 0, 0, 0x40, 0, 0, 0, 0, 0x13, 0x46, 0x11, 0x80, 0x80,
    0, 0, 0, 0x10, 0, 0, 0, 0x16, 0x16, 0x16, 0x81, 0x87, 0x82, 0, 0x80, 0x80, 0, 0, 0, 0, 0x80,
    0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0, 0, 0, 0x17, 0, 0x12, 0, 0, 0, 0, 0,
    0x10, 0x17, 0, 0x40, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0,
    0, 0x80, 0, 0, 0, 0, 0, 0,
];
pub const SPRITE_INIT_DEFL_BITS: [u8; 243] = [
    0, 0, 0x44, 0x20, 0x20, 0x20, 0x20, 0x20, 0, 0x81, 0, 0, 0x48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0,
    0, 0, 0, 0x48, 0x24, 0x80, 0, 0, 0, 0x20, 0, 0, 0, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x80,
    0, 0, 0, 0, 0, 0, 0x80, 0, 0x80, 0, 2, 0, 0, 0, 4, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0x84, 0, 0x81, 5, 1, 0x40, 8, 0xa0, 0, 0, 0, 0, 0, 0x84, 0x84, 0x84, 0x84, 8, 0x80, 0x80,
    0x80, 0, 0x80, 0x80, 0x80, 0x80, 0, 8, 0x80, 0, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0,
    4, 0, 0, 0, 0, 0x80, 4, 4, 0, 0, 0x48, 0, 0, 4, 0, 1, 1, 0, 0, 0x80, 0, 0, 0, 0x40, 8, 8, 8, 8,
    0, 0, 0, 0x80, 0x80, 0, 0, 0, 4, 1, 5, 0, 0, 0, 0, 0, 0, 0, 4, 2, 0, 0x80, 0x80, 0x80, 0x80,
    0x82, 0x80, 0, 0, 0x80, 0, 0, 0x80, 0x80, 0, 0, 1, 1, 0x40, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 4, 5,
    5, 5, 0x80, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0,
    0x82, 0x82, 8, 0x80, 0x20, 0x80, 0x80, 0x80, 0x20,
];
pub const PLAYER_DAMAGES: [i8; 30] = [
    2, 1, 1, 4, 4, 4, 0, 0, 0, 8, 4, 2, 8, 8, 8, 16, 8, 4, 32, 16, 8, 32, 24, 16, 24, 16, 8, 64,
    48, 24,
];
pub const PLAYER_ACTION_BOX_RUN_Y_HI: [u8; 4] = [0xff, 0, 0, 0];
pub const PLAYER_ACTION_BOX_RUN_Y_LO: [u8; 4] = [0xf8, 16, 8, 8];
pub const PLAYER_ACTION_BOX_RUN_X_HI: [u8; 4] = [0, 0, 0xff, 0];
pub const PLAYER_ACTION_BOX_RUN_X_LO: [u8; 4] = [0, 0, 0xf8, 8];
pub const PLAYER_SETUP_ACTION_HIT_BOX_X: [i8; 65] = [
    0, 2, 0, 0, -8, 0, 2, 0, 2, 2, 1, 1, 0, 0, 0, 0, 0, 2, 4, 4, 0, 0, -4, -4, -6, 2, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 2, 2, 4, 4, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, -4, -4, -10, 0, 2, 2, 0, 0,
    0, 0, 0, 0, 0,
];
pub const PLAYER_SETUP_ACTION_HIT_BOX_W: [i8; 65] = [
    15, 4, 8, 8, 8, 8, 12, 8, 4, 4, 6, 6, 0, 0, 0, 0, 0, 4, 16, 12, 8, 8, 12, 11, 12, 4, 6, 6, 0,
    0, 0, 0, 0, 8, 8, 8, 10, 14, 15, 4, 4, 4, 6, 6, 0, 0, 0, 0, 0, 8, 8, 8, 10, 14, 15, 4, 4, 4, 6,
    6, 0, 0, 0, 0, 0,
];
pub const PLAYER_SETUP_ACTION_HIT_BOX_Y: [i8; 65] = [
    0, 2, 0, 2, 4, 4, 4, 7, 2, 2, 1, 1, 0, 0, 0, 0, 0, 2, 0, 2, -4, -3, -8, 0, 0, 2, 1, 1, 0, 0, 0,
    0, 0, 0, 0, 0, -2, 0, -4, 1, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, -2, 0, -4, 1, 2, 2, 1, 1, 0,
    0, 0, 0, 0,
];
pub const PLAYER_SETUP_ACTION_HIT_BOX_H: [i8; 65] = [
    15, 4, 8, 2, 12, 8, 12, 8, 4, 4, 6, 6, 0, 0, 0, 0, 0, 4, 8, 4, 12, 12, 12, 4, 8, 4, 6, 4, 0, 0,
    0, 0, 0, 8, 8, 8, 8, 8, 12, 4, 4, 4, 6, 6, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 12, 4, 4, 4, 6, 6, 0,
    0, 0, 0, 0,
];
pub const PLAYER_SETUP_ACTION_HIT_BOX_TAB4: [i8; 13] = [1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1];
pub const SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1: [u8; 236] = [
    200, 0, 107, 0, 0, 0, 0, 0, 0, 203, 0, 8, 10, 11, 0, 0, 13, 0, 0, 86, 0, 0, 15, 17, 0, 19, 0,
    0, 0, 0, 20, 0, 21, 27, 0, 42, 42, 248, 0, 182, 0, 0, 0, 170, 0, 0, 28, 0, 0, 0, 0, 0, 0, 0, 0,
    243, 243, 0, 187, 39, 0, 0, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 63, 0, 0, 0, 64, 64, 68, 0,
    0, 0, 0, 71, 70, 0, 0, 72, 74, 101, 101, 0, 0, 0, 0, 0, 143, 0, 0, 76, 78, 78, 78, 78, 0, 48,
    36, 50, 56, 60, 129, 0, 82, 0, 0, 0, 0, 0, 0, 92, 0, 98, 94, 0, 0, 0, 101, 102, 0, 0, 0, 0,
    110, 14, 0, 59, 66, 0, 0, 117, 120, 123, 0, 0, 207, 0, 132, 141, 141, 141, 141, 0, 148, 117,
    160, 0, 0, 162, 166, 0, 0, 0, 177, 0, 181, 0, 189, 0, 0, 0, 105, 0, 0, 0, 0, 0, 92, 0, 214,
    230, 0, 0, 0, 219, 218, 233, 0, 0, 190, 192, 106, 0, 249, 215, 0, 0, 0, 216, 0, 0, 222, 227, 0,
    0, 0, 235, 0, 0, 0, 0, 0, 0, 244, 244, 29, 31, 31, 31, 32, 32, 32, 33, 34, 35, 35, 37, 40, 106,
    246, 41, 0, 0, 205, 206,
];
pub const SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2: [u8; 251] = [
    0xa0, 0xa2, 0xa0, 0xa2, 0x80, 0x82, 0x80, 0x82, 0xea, 0xec, 0x84, 0x4e, 0x61, 0xbd, 0x8c, 0x20,
    0x22, 0xc0, 0xc2, 0xe6, 0xe4, 0x82, 0xaa, 0x84, 0xac, 0x80, 0xa0, 0xca, 0xaf, 0x29, 0x39, 0xb,
    0x6e, 0x60, 0x62, 0x63, 0x4c, 0xea, 0xec, 0x24, 0x6b, 0x24, 0x22, 0x24, 0x26, 0x20, 0x30, 0x21,
    0x2a, 0x24, 0x86, 0x88, 0x8a, 0x8c, 0x8e, 0xa2, 0xa4, 0xa6, 0xa8, 0xaa, 0x84, 0x80, 0x82, 0x6e,
    0x40, 0x42, 0xe6, 0xe8, 0x80, 0x82, 0xc8, 0x8d, 0xe3, 0xe5, 0xc5, 0xe1, 4, 0x24, 0xe, 0x2e,
    0xc, 0xa, 0x9c, 0xc7, 0xb6, 0xb7, 0x60, 0x62, 0x64, 0x66, 0x68, 0x6a, 0xe4, 0xf4, 2, 2, 0, 4,
    0xc6, 0xcc, 0xce, 0x28, 0x84, 0x82, 0x80, 0xe5, 0x24, 0, 2, 4, 0xa0, 0xaa, 0xa4, 0xa6, 0xac,
    0xa2, 0xa8, 0xa6, 0x88, 0x86, 0x8e, 0xae, 0x8a, 0x42, 0x44, 0x42, 0x44, 0x64, 0x66, 0xcc, 0xcc,
    0xca, 0x87, 0x97, 0x8e, 0xae, 0xac, 0x8c, 0x8e, 0xaa, 0xac, 0xd2, 0xf3, 0x84, 0xa2, 0x84, 0xa4,
    0xe7, 0x8a, 0xa8, 0x8a, 0xa8, 0x88, 0xa0, 0xa4, 0xa2, 0xa6, 0xa6, 0xa6, 0xa6, 0x7e, 0x7f, 0x8a,
    0x88, 0x8c, 0xa6, 0x86, 0x8e, 0xac, 0x86, 0xbb, 0xac, 0xa9, 0xb9, 0xaa, 0xba, 0xbc, 0x8a, 0x8e,
    0x8a, 0x86, 0xa, 0xc2, 0xc4, 0xe2, 0xe4, 0xc6, 0xea, 0xec, 0xff, 0xe6, 0xc6, 0xcc, 0xec, 0xce,
    0xee, 0x4c, 0x6c, 0x4e, 0x6e, 0xc8, 0xc4, 0xc6, 0x88, 0x8c, 0x24, 0xe0, 0xae, 0xc0, 0xc8, 0xc4,
    0xc6, 0xe2, 0xe0, 0xee, 0xae, 0xa0, 0x80, 0xee, 0xc0, 0xc2, 0xbf, 0x8c, 0xaa, 0x86, 0xa8, 0xa6,
    0x2c, 0x28, 6, 0xdf, 0xcf, 0xa9, 0x46, 0x46, 0xea, 0xc0, 0xc2, 0xe0, 0xe8, 0xe2, 0xe6, 0xe4,
    0xb, 0x8e, 0xa0, 0xec, 0xea, 0xe9, 0x48, 0x58,
];
pub const OVERWORLD_AREA_SPRCOLL_SIZES: [u8; 192] = [
    4, 4, 2, 4, 4, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 2, 4, 4, 2, 4, 4,
    4, 4, 2, 4, 4, 2, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 2, 2, 2, 4, 4, 2, 4, 4, 2, 2, 2, 4, 4, 2,
    4, 4, 2, 4, 4, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 2, 4, 4, 2, 4, 4,
    4, 4, 2, 4, 4, 2, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 2, 2, 2, 4, 4, 2, 4, 4, 2, 2, 2, 4, 4, 2,
    4, 4, 2, 4, 4, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 2, 4, 4, 2, 4, 4,
    4, 4, 2, 4, 4, 2, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 2, 2, 2, 4, 4, 2, 4, 4, 2, 2, 2, 4, 4, 2,
];
pub use crate::oam::REGION_INITIAL_BASES as OAM_RESET_REGION_BASES;
pub const GARNISH_OAM_MEM_SIZE: [u8; 23] = [
    0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 8, 16,
];
pub const GARNISH_FUNCS: [u8; 22] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0, 14, 15, 16, 17, 18, 19, 20, 21, 22,
];
pub const SPRITE_EXECUTE_SINGLE: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
pub const SPRITE_FALL_TAB1: [u8; 32] = [
    13, 13, 13, 13, 13, 13, 13, 12, 12, 12, 12, 12, 3, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0,
    0, 0, 0, 0,
];
pub const SPRITE_FALL_TAB2: [u8; 32] = [
    5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
];
pub const SPRITE_FALL_TAB3: [u8; 16] = [
    0xff, 0x3f, 0x1f, 0xf, 0xf, 7, 3, 1, 0xff, 0x3f, 0x1f, 0xf, 7, 3, 1, 0,
];
pub const SPRITE_FALL_TAB4: [i8; 4] = [0, 4, 8, 0];
pub const SPRITE_DRAW_FALL1_X: [i8; 56] = [
    -4, 4, -4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, -4, 12, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0,
    0, 0, -4, 12, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0,
];
pub const SPRITE_DRAW_FALL1_Y: [i8; 56] = [
    -4, -4, 4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, -4, -4, 12, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0,
    0, 0, -4, -4, 12, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0,
];
pub const SPRITE_DRAW_FALL1_CHAR: [u8; 56] = [
    0xae, 0xa8, 0xa6, 0xaf, 0xaa, 0, 0, 0, 0xac, 0, 0, 0, 0xbe, 0, 0, 0, 0xa8, 0xae, 0xaf, 0xa6,
    0xaa, 0, 0, 0, 0xac, 0, 0, 0, 0xbe, 0, 0, 0, 0xa6, 0xaf, 0xae, 0xa8, 0xaa, 0, 0, 0, 0xac, 0, 0,
    0, 0xbe, 0, 0, 0, 0xb6, 0, 0, 0, 0x80, 0, 0, 0,
];
pub const SPRITE_DRAW_FALL1_FLAGS: [u8; 56] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0x40, 0,
    0, 0, 0x40, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0x80, 0, 0, 0, 0x80, 0, 0, 0, 1, 0,
    0, 0, 1, 0, 0, 0,
];
pub const SPRITE_DRAW_FALL1_EXT: [u8; 56] = [
    0, 2, 2, 0, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
    2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const SPRITE_DISTRESS_X: [i8; 4] = [-3, 2, 7, 11];
pub const SPRITE_DISTRESS_Y: [i8; 4] = [-5, -7, -7, -5];
pub const PIKIT_DROP_ITEMS: [u8; 4] = [0xdc, 0xe1, 0xd9, 0xe6];
pub const PRIZE_MASKS: [u8; 7] = [1, 1, 1, 0, 1, 1, 1];
pub const PRIZE_ITEMS: [u8; 56] = [
    0xd8, 0xd8, 0xd8, 0xd8, 0xd9, 0xd8, 0xd8, 0xd9, 0xda, 0xd9, 0xda, 0xdb, 0xda, 0xd9, 0xda, 0xda,
    0xe0, 0xdf, 0xdf, 0xda, 0xe0, 0xdf, 0xd8, 0xdf, 0xdc, 0xdc, 0xdc, 0xdd, 0xdc, 0xdc, 0xde, 0xdc,
    0xe1, 0xd8, 0xe1, 0xe2, 0xe1, 0xd8, 0xe1, 0xe2, 0xdf, 0xd9, 0xd8, 0xe1, 0xdf, 0xdc, 0xd9, 0xd8,
    0xd8, 0xe3, 0xe0, 0xdb, 0xde, 0xd8, 0xdb, 0xe2,
];
pub const PRIZE_Z: [u8; 15] = [
    0, 0x24, 0x24, 0x24, 0x20, 0x20, 0x20, 0x24, 0x24, 0x24, 0x24, 0, 0x24, 0x20, 0x20,
];
pub const PERISH_OVERLAY_X: [i8; 32] = [
    0, 0, 0, 8, 0, 8, 0, 8, 8, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -3, 11, -3, 11, -6, 14,
    -6, 14,
];
pub const PERISH_OVERLAY_Y: [i8; 32] = [
    0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -3, -3, 11, 11, -6, -6,
    14, 14,
];
pub const PERISH_OVERLAY_CHAR: [u8; 32] = [
    0, 0xb9, 0, 0, 0xb4, 0xb5, 0xb5, 0xb4, 0xb9, 0, 0, 0, 0xb5, 0xb4, 0xb4, 0xb5, 0xa8, 0xa8, 0xb8,
    0xb8, 0xa8, 0xa8, 0xb8, 0xb8, 0xa9, 0xa9, 0xa9, 0xa9, 0x9b, 0x9b, 0x9b, 0x9b,
];
pub const PERISH_OVERLAY_FLAGS: [u8; 32] = [
    4, 4, 4, 4, 4, 4, 0xc4, 0xc4, 0x44, 4, 4, 4, 0x44, 0x44, 0x84, 0x84, 4, 0x44, 4, 0x44, 4, 0x44,
    4, 0x44, 0x44, 4, 0xc4, 0x84, 4, 0x44, 0x84, 0xc4,
];
pub const SPAWN_SECRET_ITEMS: [u8; 22] = [
    0xd9, 0x3e, 0x79, 0xd9, 0xdc, 0xd8, 0xda, 0xe4, 0xe1, 0xdc, 0xd8, 0xdf, 0xe0, 0xb, 0x42, 0xd3,
    0x41, 0xd4, 0xd9, 0xe3, 0xd8, 0,
];
pub const SPAWN_SECRET_ITEM_SPAWN_FLAG: [u8; 22] = [
    0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const SPAWN_SECRET_ITEM_X_LO: [u8; 22] = [
    4, 0, 4, 4, 0, 4, 4, 4, 4, 0, 4, 4, 4, 0, 0, 0, 0, 0, 4, 0, 4, 4,
];
pub const SPAWN_SECRET_ITEM_IGNORE_PROJ: [u8; 22] = [
    1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1,
];
pub const SPAWN_SECRET_ITEM_Z_VEL: [u8; 22] = [
    16, 0, 0, 16, 0, 0, 16, 16, 16, 16, 0, 16, 10, 16, 0, 0, 0, 0, 16, 0, 0, 0,
];
pub const SPRITE_RETURN_IF_LIFTED_DIRS: [u8; 4] = [4, 6, 0, 2];
pub const ABSORBABLE_TAB1: [u8; 15] = [0, 1, 1, 1, 2, 2, 2, 0, 1, 1, 2, 2, 1, 2, 2];
pub const ABSORBABLE_TAB2: [u8; 19] = [0, 0, 0, 0, 1, 2, 3, 0, 0, 4, 5, 0, 0, 0, 0, 2, 4, 6, 2];
pub const NUMBERED_ABSORBABLE_X: [i16; 18] = [0, 0, 8, 0, 0, 8, 0, 0, 8, 0, 0, 2, 0, 0, 2, 0, 0, 0];
pub const NUMBERED_ABSORBABLE_Y: [i16; 18] = [0, 0, 8, 0, 0, 8, 0, 0, 8, 0, 8, 8, 0, 8, 8, 0, 8, 8];
pub const NUMBERED_ABSORBABLE_CHAR: [u8; 18] = [
    0x6e, 0x6e, 0x68, 0x6e, 0x6e, 0x78, 0x6e, 0x6e, 0x79, 0x63, 0x73, 0x69, 0x63, 0x73, 0x6a, 0x63,
    0x73, 0x73,
];
pub const NUMBERED_ABSORBABLE_EXT: [u8; 18] =
    [2, 2, 0, 2, 2, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const RUPEES_ABSORPTION: [u8; 3] = [1, 5, 20];
pub const BOMBS_ABSORPTION: [u8; 3] = [1, 4, 8];

#[cfg(test)]
mod tests {
    use super::*;

    struct Hooks;
    impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for Hooks {}
    impl SpriteHooks for Hooks {}
    impl crate::sprite_main_bank0::SpriteMainBank0Runtime for Hooks {}
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for Hooks {}
    impl crate::sprite_main_bank1::SpriteMainBank1Runtime for Hooks {}

    #[derive(Default)]
    struct GarnishHooks {
        flash_gate: Option<u8>,
        flashes: usize,
        tile_slots: Vec<u8>,
    }
    impl crate::sprite_main_bank0::SpriteMainBank0Runtime for GarnishHooks {}
    impl crate::sprite_main_bank1::SpriteMainBank1Runtime for GarnishHooks {}
    impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for GarnishHooks {}
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for GarnishHooks {}
    impl SpriteHooks for GarnishHooks {
        fn handle_screen_flash(&mut self, ram: &mut Ram<'_>) {
            self.flashes += 1;
            if let Some(marker) = self.flash_gate {
                ram.set_garnish_active_marker(marker);
                ram.set_garnish_kind(29, 5);
                ram.set_garnish_countdown(29, 1);
            }
        }

        fn dungeon_update_tilemap_with_common_tile(
            &mut self,
            ram: &mut Ram<'_>,
            _x: u16,
            _y: u16,
            _tile: u8,
        ) {
            self.tile_slots.push(ram.read_u8(0xfa0));
            ram.set_garnish_active_marker(0);
            // Slot14 was already visited; slot4 has not yet been visited.
            for slot in [4, 14] {
                ram.set_garnish_kind(slot, 13);
                ram.set_garnish_countdown(slot, 2);
            }
            // FallingTile must read coordinates live after this callback.
            ram.garnish_mut(10).set_x(12);
            ram.garnish_mut(10).set_y(44);
        }
    }

    #[test]
    fn garnish_screen_flash_precedes_gate_and_expiry_keeps_full_memory() {
        for marker in [0, 31] {
            let mut bytes = Box::new([0; WRAM_SIZE]);
            bytes[0xfa0] = 0xa5;
            let mut expected = bytes.clone();
            expected[crate::garnish_state::ACTIVE_MARKER_ADDRESS] = marker;
            expected[0x1f800 + 29] = if marker == 0 { 5 } else { 0 };
            expected[0x1f90e + 29] = if marker == 0 { 1 } else { 0 };
            if marker != 0 {
                expected[0xfa0] = 15;
            }
            let mut hooks = GarnishHooks {
                flash_gate: Some(marker),
                ..GarnishHooks::default()
            };
            Sprite::new(&mut bytes, &mut hooks).garnish_execute_upper_slots();
            assert_eq!(hooks.flashes, 1);
            assert_eq!(bytes, expected);
        }
    }

    #[test]
    fn garnish_lower_pass_keeps_live_hook_state_and_reverse_slot_order() {
        let mut bytes = [0; WRAM_SIZE];
        let mut hooks = GarnishHooks::default();
        let mut sprite = Sprite::new(&mut bytes, &mut hooks);
        sprite.oam_reset_region_bases();
        sprite.ram.set_garnish_active_marker(1);
        sprite.ram.set_garnish_kind(10, 3);
        sprite.ram.set_garnish_countdown(10, 31);
        sprite.garnish_execute_lower_slots();
        assert_eq!(sprite.ram.garnish_active_marker(), 0);
        assert_eq!(sprite.ram.garnish_countdown(4), 1);
        assert_eq!(sprite.ram.garnish_countdown(14), 2);
        assert_eq!(sprite.ram.read_u8(0xfa0), 0);
        assert_eq!(sprite.ram.slice(0x830, 4), &[16, 32, 0x80, 0x30]);
        assert_eq!(hooks.tile_slots, [10]);
        assert_eq!(hooks.flashes, 0);
    }

    #[derive(Default)]
    struct ExecutionHooks {
        random_calls: usize,
    }
    impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for ExecutionHooks {}
    impl SpriteHooks for ExecutionHooks {
        fn random_u8(&mut self, _: &mut Ram<'_>) -> u8 {
            self.random_calls += 1;
            3
        }
    }
    impl crate::sprite_main_bank0::SpriteMainBank0Runtime for ExecutionHooks {}
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for ExecutionHooks {}
    impl crate::sprite_main_bank1::SpriteMainBank1Runtime for ExecutionHooks {}

    struct TerrainHooks;
    impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for TerrainHooks {}
    impl SpriteHooks for TerrainHooks {
        fn throwable_scenery_flags(&self, kind: u8) -> u8 {
            [0x0c, 0x0c, 0x0c, 0, 0, 0, 0xb0, 0x08, 0xb4][usize::from(kind)]
        }
    }
    impl crate::sprite_main_bank0::SpriteMainBank0Runtime for TerrainHooks {}
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for TerrainHooks {}
    impl crate::sprite_main_bank1::SpriteMainBank1Runtime for TerrainHooks {}

    struct LoaderHooks;
    impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for LoaderHooks {}
    impl SpriteHooks for LoaderHooks {
        fn dungeon_sprite_data(&self, _room: u16) -> &[u8] {
            &[1, 0x12, 0x34, 0x21, 0xff, 0, 0]
        }
        fn overworld_sprite_data(&self, _ram: &Ram<'_>, _area: u16) -> &[u8] {
            &[0x23, 0x45, 0x21, 0xff, 0, 0]
        }
    }
    impl crate::sprite_main_bank0::SpriteMainBank0Runtime for LoaderHooks {}
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for LoaderHooks {}
    impl crate::sprite_main_bank1::SpriteMainBank1Runtime for LoaderHooks {}

    #[test]
    fn movement_preserves_signed_4_4_velocity_and_subpixels() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.sprite_set_x(0, 0x1234);
        sprites.ram.set_sprite_subpixel_x(0, 0xf8);
        sprites.ram.set_sprite_velocity_x(0, (-2i8) as u8);
        sprites.sprite_move_x(0);
        assert_eq!(sprites.sprite_get_x(0), 0x1234);
        assert_eq!(sprites.ram.sprite(0).subpixel_x(), 0xd8);
        sprites.ram.set_sprite_velocity_x(0, 8);
        sprites.sprite_move_x(0);
        assert_eq!(sprites.sprite_get_x(0), 0x1235);
        assert_eq!(sprites.ram.sprite(0).subpixel_x(), 0x58);
    }

    #[test]
    fn coordinate_setters_preserve_fractional_and_neighboring_bytes() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut expected = bytes.to_vec();
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.sprite_set_x(15, 0xabcd);
        sprites.sprite_set_y(15, 0x1234);
        expected[SPRITE_X_LO + 15] = 0xcd;
        expected[SPRITE_X_HI + 15] = 0xab;
        expected[SPRITE_Y_LO + 15] = 0x34;
        expected[SPRITE_Y_HI + 15] = 0x12;
        assert_eq!(sprites.sprite_get_x(15), 0xabcd);
        assert_eq!(sprites.sprite_get_y(15), 0x1234);
        assert_eq!(sprites.ram.slice(0, WRAM_SIZE), expected);
    }

    #[test]
    fn movement_preserves_all_bytes_outside_the_selected_coordinate() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        for axis in 0..3 {
            let (fraction, low, high, velocity) = match axis {
                0 => (
                    SPRITE_X_SUBPIXEL,
                    SPRITE_X_LO,
                    Some(SPRITE_X_HI),
                    SPRITE_X_VEL,
                ),
                1 => (
                    SPRITE_Y_SUBPIXEL,
                    SPRITE_Y_LO,
                    Some(SPRITE_Y_HI),
                    SPRITE_Y_VEL,
                ),
                _ => (SPRITE_Z_SUBPOS, SPRITE_Z, None, SPRITE_Z_VEL),
            };
            for slot in [0, 15] {
                for pixels in [0u16, 0xff, 0xffff] {
                    for subpixel in [0u8, 0xf8] {
                        for speed in [-128i8, -1, 0, 1, 127] {
                            sprites.ram.bytes_mut().fill(0xa5);
                            sprites.set_arr(fraction, slot, subpixel);
                            sprites.set_arr(low, slot, pixels as u8);
                            if let Some(high) = high {
                                sprites.set_arr(high, slot, (pixels >> 8) as u8);
                            }
                            sprites.set_arr(velocity, slot, speed as u8);
                            let mut expected = sprites.ram.slice(0, WRAM_SIZE).to_vec();
                            let integer = if high.is_some() {
                                pixels
                            } else {
                                pixels & 0xff
                            };
                            let sum = ((u32::from(integer) << 8) | u32::from(subpixel))
                                .wrapping_add((i32::from(speed) << 4) as u32);
                            expected[fraction + slot] = sum as u8;
                            expected[low + slot] = (sum >> 8) as u8;
                            if let Some(high) = high {
                                expected[high + slot] = (sum >> 16) as u8;
                            }
                            match axis {
                                0 => sprites.sprite_move_x(slot),
                                1 => sprites.sprite_move_y(slot),
                                _ => sprites.sprite_move_z(slot),
                            }
                            assert_eq!(sprites.ram.slice(0, WRAM_SIZE), expected);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn speed_projection_preserves_c_octant_and_signed_bytes() {
        let right = PairU8 { a: 0, b: 40 };
        let below = PairU8 {
            a: 1,
            b: (-20i8) as u8,
        };
        assert_eq!(
            Sprite::<Hooks>::project_speed(right, below, 16),
            ProjectSpeedRet {
                x: 16,
                y: (-8i8) as u8,
                xdiff: 40,
                ydiff: (-20i8) as u8
            }
        );
        assert_eq!(
            Sprite::<Hooks>::project_speed(right, below, 0),
            ProjectSpeedRet::default()
        );
    }

    #[test]
    fn cached_execution_restores_all_twenty_four_live_fields() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = ExecutionHooks::default();
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        let live = [
            SPRITE_STATE,
            SPRITE_TYPE,
            SPRITE_X_LO,
            SPRITE_X_HI,
            SPRITE_Y_LO,
            SPRITE_Y_HI,
            SPRITE_GRAPHICS,
            SPRITE_A,
            SPRITE_HEAD_DIR,
            SPRITE_OAM_FLAGS,
            SPRITE_OBJ_PRIO,
            SPRITE_D,
            SPRITE_FLAGS2,
            SPRITE_FLOOR,
            SPRITE_AI_STATE,
            SPRITE_FLAGS3,
            SPRITE_B,
            SPRITE_C,
            SPRITE_E,
            SPRITE_SUBTYPE2,
            SPRITE_Z,
            SPRITE_DELAY_MAIN,
            SPRITE_I,
            SPRITE_IGNORE_PROJECTILE,
        ];
        for (i, base) in live.into_iter().enumerate() {
            sprites.set_arr(base, 0, i as u8 + 0x40);
        }
        sprites.set_arr(0x1d00, 0, 9);
        sprites.set_arr(0x1d10, 0, 1);
        sprites.uncache_and_execute_sprite(0);
        for (i, base) in live.into_iter().enumerate() {
            assert_eq!(sprites.arr(base, 0), i as u8 + 0x40);
        }
    }

    #[test]
    fn guard_parry_negative_hit_timer_short_circuits_before_randomness() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = ExecutionHooks::default();
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.ram.set_sprite_floor(0, 0);
        sprites.ram.set_sprite_hit_timer(0, 0x80);
        sprites.guard_parry_sword_attacks(0);
        assert_eq!(sprites.hooks.random_calls, 0);
        assert_eq!(sprites.ram.sprite(0).scratch_f(), 0);
    }

    #[test]
    fn dispatch_table_inventory_matches_c_function_pointer_tables() {
        assert_eq!(
            SPRITE_EXECUTE_SINGLE,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
        );
        assert_eq!(GARNISH_FUNCS[12], 0);
        assert_eq!(GARNISH_FUNCS.len(), 22);
        assert_eq!(GARNISH_OAM_MEM_SIZE.len(), 23);
    }

    #[test]
    fn oam_region_allocation_wraps_through_original_eight_positions() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.w16(0xfec, 0x1234);
        sprites.oam_reset_region_bases();
        assert_eq!(sprites.r16(0xfec), 0x1234);
        sprites.w16(0xfec, 0);
        assert_eq!(sprites.oam_allocate_from_region_a(4), 0x30);
        sprites.w16(0xfe0, OAM_GET_BUFFER_POS_TAB0[0] - 1);
        assert_eq!(
            sprites.oam_allocate_from_region_a(4),
            OAM_GET_BUFFER_POS_TAB1[0] as u8
        );
        assert_eq!(sprites.r16(0x92), 0xa20 + (OAM_GET_BUFFER_POS_TAB1[0] >> 2));
    }

    #[test]
    fn hitbox_overlap_matches_c_boundary_and_page_rules() {
        let mut box_ = SpriteHitBox {
            r0_xlo: 0xf8,
            r1_ylo: 0x20,
            r2: 8,
            r3: 8,
            r4_spr_xlo: 0x00,
            r5_spr_ylo: 0x27,
            r6_spr_xsize: 8,
            r7_spr_ysize: 8,
            r8_xhi: 0,
            r9_yhi: 1,
            r10_spr_xhi: 1,
            r11_spr_yhi: 1,
        };
        assert!(Sprite::<Hooks>::check_if_hit_boxes_overlap(&box_));
        box_.r5_spr_ylo = 0x28;
        assert!(Sprite::<Hooks>::check_if_hit_boxes_overlap(&box_));
        box_.r5_spr_ylo = 0x29;
        assert!(!Sprite::<Hooks>::check_if_hit_boxes_overlap(&box_));
        box_.r10_spr_xhi = 0x80;
        assert!(!Sprite::<Hooks>::check_if_hit_boxes_overlap(&box_));
    }

    #[test]
    fn reset_all_uses_canonical_wram_regions_and_sentinel() {
        let mut bytes = [0x55u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.w8(0xf3cc, 0);
        sprites.sprite_reset_all_no_disable();
        assert_eq!(sprites.r8(0x4b4), 0xfe);
        assert!(sprites.ram.slice(0x1df80, 0x1000).iter().all(|&x| x == 0));
        assert!(sprites.ram.slice(0x1ef80, 0x200).iter().all(|&x| x == 0));
        assert!(sprites.ram.slice(0xb80, 8).iter().all(|&x| x == 0xff));
    }

    #[test]
    fn overworld_proxima_load_marks_block_and_preserves_packed_coordinates() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.w16(0xfbc, 0x0200);
        sprites.w16(0xfbe, 0x0400);
        let block = 0x0132u16;
        sprites.w8(0x1df80 + usize::from(block), 0x22);
        sprites.overworld_load_proxima_sprite_if_alive(block);
        assert_eq!(sprites.ram.sprite(13).state(), 8);
        assert_eq!(sprites.ram.sprite(13).kind(), 0x21);
        assert_eq!(sprites.r16(0xbc0 + 13 * 2), block);
        assert_eq!(sprites.ram.sprite(13).x_low(), 0x20);
        assert_eq!(sprites.ram.sprite(13).y_low(), 0x30);
        assert_eq!(sprites.ram.sprite(13).x_high(), 3);
        assert_eq!(sprites.ram.sprite(13).y_high(), 4);
        assert_ne!(
            sprites.r8(0x1ef80 + usize::from(block >> 3)) & (0x80 >> (block & 7)),
            0
        );
    }

    #[test]
    fn dungeon_death_flags_are_sixteen_bit_per_room() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.w8(0x1b, 1);
        sprites.w16(0x48e, 3);
        sprites.ram.set_sprite_scratch_n(12, 12);
        sprites.sprite_manually_set_death_flag_uw(12);
        assert_eq!(sprites.r16(0x1df80 + 6), 0x1000);
    }

    #[test]
    fn throwable_terrain_initialization_preserves_external_flag_table_quirks() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = TerrainHooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.w8(0x1b, 1);
        sprites.w8(0xee, 1);
        assert_eq!(
            sprites.sprite_spawn_throwable_terrain_silently(2, 0x1234, 0x5678),
            15
        );
        assert_eq!(sprites.ram.sprite(15).state(), 10);
        assert_eq!(sprites.ram.sprite(15).kind(), 0xec);
        assert_eq!(sprites.sprite_get_x(15), 0x1234);
        assert_eq!(sprites.sprite_get_y(15), 0x5678);
        assert_eq!(sprites.ram.sprite(15).oam_flags(), 0x50);
        assert_eq!(sprites.ram.sprite(15).unknown4(), 9);
        assert_eq!(sprites.r8(0x314), 2);
        assert_eq!(sprites.r8(0xfb2), 2);
    }

    #[test]
    fn absorbable_rupee_updates_goal_with_wrapping_arithmetic() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.ram.set_sprite_kind(0, 0xda);
        sprites.ram.set_sprite_state(0, 9);
        sprites.w16(0xf360, 0xffff);
        sprites.sprite_handle_absorption_by_player(0);
        assert_eq!(sprites.ram.sprite(0).state(), 0);
        assert_eq!(sprites.r16(0xf360), 4);
    }

    #[test]
    fn moldorm_recoil_damage_253_enters_burn_state_exactly() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.ram.set_sprite_state(3, 9);
        sprites.ram.set_sprite_damage(3, 253);
        sprites.ram.set_sprite_flags2(3, 0x22);
        sprites.sprite_mini_moldorm_recoil(3);
        assert_eq!(sprites.ram.sprite(3).state(), 7);
        assert_eq!(sprites.ram.sprite(3).main_delay(), 0x70);
        assert_eq!(sprites.ram.sprite(3).flags2(), 0x24);
        assert_eq!(sprites.ram.sprite(3).damage(), 0);
    }

    #[test]
    fn fall_draw_uses_current_oam_pointer_and_corrects_extended_entry() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.oam_reset_region_bases();
        sprites.oam_allocate_from_region_a(4);
        sprites.sprite_set_x(0, 0x0120);
        sprites.sprite_set_y(0, 0x0040);
        sprites.w16(0xe2, 0x0100);
        sprites.w16(0xe8, 0);
        sprites.ram.set_sprite_main_delay(0, 12);
        sprites.sprite_fall_draw(
            0,
            &PrepOamCoordsRet {
                x: 0x20,
                y: 0x40,
                flags: 0x35,
                r4: 0,
            },
        );
        let entry = oam::read(&sprites.ram, sprites.current_oam_index());
        assert_eq!(entry.charnum, 0x80);
        assert_eq!(entry.flags, 0x34);
        assert_eq!(sprites.r8(usize::from(sprites.r16(0x92))), 0);
    }

    #[test]
    fn bulk_loaders_decode_packed_dungeon_and_overworld_records() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = LoaderHooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.w16(0x48e, 0x0020);
        sprites.dungeon_load_sprites();
        assert_eq!(sprites.r8(0xfb3), 1);
        assert_eq!(sprites.ram.sprite(0).state(), 8);
        assert_eq!(sprites.ram.sprite(0).kind(), 0x21);
        // The C loader masks the packed local coordinate to nine bits.
        assert_eq!(sprites.sprite_get_x(0), 0x0140);
        assert_eq!(sprites.sprite_get_y(0), 0x0520);

        sprites.w16(0x40a, 0);
        sprites.overworld_load_sprites();
        let page: u8 = ((0x23u8 >> 4) << 2).wrapping_add(0x45 >> 4);
        let cell: u8 = (0x45 & 0x0f) | 0x23 << 4;
        assert_eq!(
            sprites.r8(0x1df80 + usize::from(cell) + (usize::from(page) << 8)),
            0x22
        );
    }

    #[test]
    fn buzz_blob_drop_preserves_adjacent_table_overread_value() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.sprite_set_x(0, 0x1000);
        sprites.prepare_enemy_drop(0, 0x0d);
        assert_eq!(sprites.sprite_get_x(0), 0x1008);
        assert_eq!(sprites.ram.sprite(0).velocity_z(), 0);
    }

    #[test]
    fn enemy_hit_sound_pan_follows_enemy_instead_of_link() {
        let mut bytes = [0u8; WRAM_SIZE];
        let mut hooks = Hooks;
        let mut sprites = Sprite::new(&mut bytes, &mut hooks);
        sprites.w16(0xe2, 0x300);
        sprites.w16(0x22, 0x380);
        sprites.ram.set_sprite_flags5(3, 0x10);
        sprites.sprite_set_x(3, 0x330);
        sprites.sprite_give_damage(3, 1, 0x20);
        assert_eq!(sprites.r8(0x12f), 0x9c);
        sprites.sprite_func22(3);
        assert_eq!(sprites.r8(0x12e), 0xa8);

        sprites.w16(0x22, 0x330);
        sprites.sprite_set_x(3, 0x380);
        sprites.sprite_give_damage(3, 1, 0x20);
        assert_eq!(sprites.r8(0x12f), 0x1c);
        sprites.sprite_func22(3);
        assert_eq!(sprites.r8(0x12e), 0x28);
    }
}
