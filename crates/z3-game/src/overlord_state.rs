//! Live views of controller storage and its shared boss workspaces.
//!
//! Eight is the controller allocation/traversal limit, not an address bound:
//! boss algorithms intentionally use raw indices crossing those byte lanes.
//! These views never initialize or claim exclusive phase ownership of B00..B57.
//! Select-file's B10..B16 aliases and player storage beginning at B58 remain
//! canonical WRAM. Never retain a view across a runtime callback.
use crate::ram::Ram;
use z3_core::WRAM_SIZE;

pub const SLOT_COUNT: usize = 8;
#[derive(Clone, Copy)]
pub struct ControllerRef<'a> {
    bytes: &'a [u8],
    index: usize,
}
pub struct ControllerMut<'a> {
    bytes: &'a mut [u8; WRAM_SIZE],
    index: usize,
}
impl ControllerRef<'_> {
    pub fn x(&self) -> u16 {
        u16::from(self.x_low()) | u16::from(self.x_high()) << 8
    }
    pub fn y(&self) -> u16 {
        u16::from(self.y_low()) | u16::from(self.y_high()) << 8
    }
    pub fn source_position(&self) -> u16 {
        let address = 0xb48 + self.index * 2;
        u16::from_le_bytes([self.bytes[address], self.bytes[address + 1]])
    }
}
impl ControllerMut<'_> {
    pub fn as_read_only(&self) -> ControllerRef<'_> {
        ControllerRef {
            bytes: self.bytes,
            index: self.index,
        }
    }
    pub fn set_x(&mut self, value: u16) {
        self.set_x_low(value as u8);
        self.set_x_high((value >> 8) as u8);
    }
    pub fn set_y(&mut self, value: u16) {
        self.set_y_low(value as u8);
        self.set_y_high((value >> 8) as u8);
    }
    pub fn set_source_position(&mut self, value: u16) {
        let address = 0xb48 + self.index * 2;
        self.bytes[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }
}
#[derive(Clone, Copy)]
pub struct ControllerPoolRef<'a> {
    bytes: &'a [u8],
}
impl<'a> ControllerPoolRef<'a> {
    pub fn slot(&self, index: usize) -> ControllerRef<'a> {
        ControllerRef {
            bytes: self.bytes,
            index,
        }
    }
    /// A reverse scan only: the free slot is not reserved or initialized.
    pub fn find_free(&self) -> Option<usize> {
        (0..SLOT_COUNT).rev().find(|&i| self.slot(i).kind() == 0)
    }
    pub fn room_clear(&self) -> bool {
        (0..SLOT_COUNT).all(|i| !matches!(self.slot(i).kind(), 20 | 24))
    }
}
impl Ram<'_> {
    pub fn controller(&self, index: usize) -> ControllerRef<'_> {
        ControllerRef {
            bytes: self.slice(0, WRAM_SIZE),
            index,
        }
    }
    pub fn controller_mut(&mut self, index: usize) -> ControllerMut<'_> {
        ControllerMut {
            bytes: self.bytes_mut(),
            index,
        }
    }
    pub fn controllers(&self) -> ControllerPoolRef<'_> {
        ControllerPoolRef {
            bytes: self.slice(0, WRAM_SIZE),
        }
    }
    pub fn controller_source_position(&self, index: usize) -> u16 {
        self.controller(index).source_position()
    }
    pub fn set_controller_source_position(&mut self, index: usize, value: u16) {
        self.controller_mut(index).set_source_position(value);
    }
    /// Clear ONLY kinds, in original reverse order; boss and inactive payload
    /// bytes, spawned-area records and source words are not pool-owned resets.
    pub fn clear_controller_kinds_reverse(&mut self) {
        for i in (0..SLOT_COUNT).rev() {
            self.set_controller_kind(i, 0);
        }
    }
}

macro_rules! controller_fields {
    ($($get:ident, $set:ident, $read:ident, $write:ident => $address:literal;)*) => {
        impl ControllerRef<'_> { $(pub fn $get(&self) -> u8 { self.bytes[$address + self.index] })* }
        impl ControllerMut<'_> { $(pub fn $get(&self) -> u8 { self.as_read_only().$get() }
            pub fn $set(&mut self, value: u8) { self.bytes[$address + self.index] = value; })* }
        // Receiver forwarding preserves u8 expected types and two-phase
        // borrowing for nested read/modify/write expressions.
        impl Ram<'_> { $(pub fn $read(&self, index: usize) -> u8 { self.controller(index).$get() }
            pub fn $write(&mut self, index: usize, value: u8) { self.controller_mut(index).$set(value); })* }
    };
}
controller_fields! {
    kind, set_kind, controller_kind, set_controller_kind => 0xb00;
    x_low, set_x_low, controller_x_low, set_controller_x_low => 0xb08;
    x_high, set_x_high, controller_x_high, set_controller_x_high => 0xb10;
    y_low, set_y_low, controller_y_low, set_controller_y_low => 0xb18;
    y_high, set_y_high, controller_y_high, set_controller_y_high => 0xb20;
    scratch_a, set_scratch_a, controller_scratch_a, set_controller_scratch_a => 0xb28;
    timer, set_timer, controller_timer, set_controller_timer => 0xb30;
    scratch_c, set_scratch_c, controller_scratch_c, set_controller_scratch_c => 0xb38;
    floor, set_floor, controller_floor, set_controller_floor => 0xb40;
    spawned_area, set_spawned_area, controller_spawned_area, set_controller_spawned_area => 0xcca;
}

/// One borrowed shared image, with explicitly overlapping named interpretations.
/// Orbit outputs, compact Arrghi positions, Helmasaur tail/leg/control bytes
/// and Trinexx words are aliases, not separately owned fields or eight-slot
/// buffers. All indexed boss lanes retain unrestricted physical WRAM indices.
#[derive(Clone, Copy)]
pub struct BossWorkspaceRef<'a> {
    bytes: &'a [u8],
}
pub struct BossWorkspaceMut<'a> {
    bytes: &'a mut [u8; WRAM_SIZE],
}
impl BossWorkspaceMut<'_> {
    pub fn as_read_only(&self) -> BossWorkspaceRef<'_> {
        BossWorkspaceRef { bytes: self.bytes }
    }
}
impl Ram<'_> {
    pub fn boss_workspace(&self) -> BossWorkspaceRef<'_> {
        BossWorkspaceRef {
            bytes: self.slice(0, WRAM_SIZE),
        }
    }
    pub fn boss_workspace_mut(&mut self) -> BossWorkspaceMut<'_> {
        BossWorkspaceMut {
            bytes: self.bytes_mut(),
        }
    }
}
macro_rules! boss_indexed_fields {
    ($($get:ident, $set:ident, $read:ident, $write:ident => $address:literal;)*) => {
        impl BossWorkspaceRef<'_> { $(pub fn $get(&self, index: usize) -> u8 { self.bytes[$address + index] })* }
        impl BossWorkspaceMut<'_> { $(pub fn $get(&self, index: usize) -> u8 { self.as_read_only().$get(index) }
            pub fn $set(&mut self, index: usize, value: u8) { self.bytes[$address + index] = value; })* }
        impl Ram<'_> { $(pub fn $read(&self, index: usize) -> u8 { self.boss_workspace().$get(index) }
            pub fn $write(&mut self, index: usize, value: u8) { self.boss_workspace_mut().$set(index, value); })* }
    };
}
boss_indexed_fields! {
    orbit_x_low, set_orbit_x_low, boss_orbit_x_low, set_boss_orbit_x_low => 0xb10;
    orbit_x_high, set_orbit_x_high, boss_orbit_x_high, set_boss_orbit_x_high => 0xb20;
    orbit_y_low, set_orbit_y_low, boss_orbit_y_low, set_boss_orbit_y_low => 0xb30;
    orbit_y_high, set_orbit_y_high, boss_orbit_y_high, set_boss_orbit_y_high => 0xb40;
    arrghi_x_low, set_arrghi_x_low, boss_arrghi_x_low, set_boss_arrghi_x_low => 0xb08;
    arrghi_x_high, set_arrghi_x_high, boss_arrghi_x_high, set_boss_arrghi_x_high => 0xb18;
    arrghi_y_low, set_arrghi_y_low, boss_arrghi_y_low, set_boss_arrghi_y_low => 0xb28;
    arrghi_y_high, set_arrghi_y_high, boss_arrghi_y_high, set_boss_arrghi_y_high => 0xb38;
    helmasaur_leg_frame, set_helmasaur_leg_frame, boss_helmasaur_leg_frame, set_boss_helmasaur_leg_frame => 0xb08;
    helmasaur_tail_x, set_helmasaur_tail_x, boss_helmasaur_tail_x, set_boss_helmasaur_tail_x => 0xb0d;
    helmasaur_tail_y, set_helmasaur_tail_y, boss_helmasaur_tail_y, set_boss_helmasaur_tail_y => 0xb1d;
    trinexx_control, set_trinexx_control, boss_trinexx_control, set_boss_trinexx_control => 0xb08;
}
macro_rules! boss_byte_fields {
    ($($get:ident, $set:ident, $read:ident, $write:ident => $address:literal;)*) => {
        impl BossWorkspaceRef<'_> { $(pub fn $get(&self) -> u8 { self.bytes[$address] })* }
        impl BossWorkspaceMut<'_> { $(pub fn $get(&self) -> u8 { self.as_read_only().$get() }
            pub fn $set(&mut self, value: u8) { self.bytes[$address] = value; })* }
        impl Ram<'_> { $(pub fn $read(&self) -> u8 { self.boss_workspace().$get() }
            pub fn $write(&mut self, value: u8) { self.boss_workspace_mut().$set(value); })* }
    };
}
boss_byte_fields! {
    orbit_phase_low, set_orbit_phase_low, boss_orbit_phase_low, set_boss_orbit_phase_low => 0xb08;
    orbit_phase_high, set_orbit_phase_high, boss_orbit_phase_high, set_boss_orbit_phase_high => 0xb09;
    orbit_radius, set_orbit_radius, boss_orbit_radius, set_boss_orbit_radius => 0xb0a;
    orbit_count, set_orbit_count, boss_orbit_count, set_boss_orbit_count => 0xb0b;
    orbit_speed, set_orbit_speed, boss_orbit_speed, set_boss_orbit_speed => 0xb0c;
    helmasaur_animation_clock, set_helmasaur_animation_clock, boss_helmasaur_animation_clock, set_boss_helmasaur_animation_clock => 0xb0c;
    helmasaur_phase_low, set_helmasaur_phase_low, boss_helmasaur_phase_low, set_boss_helmasaur_phase_low => 0xb2d;
    helmasaur_turn, set_helmasaur_turn, boss_helmasaur_turn, set_boss_helmasaur_turn => 0xb2e;
    helmasaur_radius, set_helmasaur_radius, boss_helmasaur_radius, set_boss_helmasaur_radius => 0xb2f;
    helmasaur_swing_state, set_helmasaur_swing_state, boss_helmasaur_swing_state, set_boss_helmasaur_swing_state => 0xb30;
    helmasaur_velocity_low, set_helmasaur_velocity_low, boss_helmasaur_velocity_low, set_boss_helmasaur_velocity_low => 0xb31;
    helmasaur_velocity_high, set_helmasaur_velocity_high, boss_helmasaur_velocity_high, set_boss_helmasaur_velocity_high => 0xb32;
    helmasaur_next_tail, set_helmasaur_next_tail, boss_helmasaur_next_tail, set_boss_helmasaur_next_tail => 0xb33;
    trinexx_target_x_low, set_trinexx_target_x_low, boss_trinexx_target_x_low, set_boss_trinexx_target_x_low => 0xb08;
    trinexx_target_y_low, set_trinexx_target_y_low, boss_trinexx_target_y_low, set_boss_trinexx_target_y_low => 0xb09;
    trinexx_bend, set_trinexx_bend, boss_trinexx_bend, set_boss_trinexx_bend => 0xb0a;
    trinexx_direction, set_trinexx_direction, boss_trinexx_direction, set_boss_trinexx_direction => 0xb0b;
    trinexx_clock, set_trinexx_clock, boss_trinexx_clock, set_boss_trinexx_clock => 0xb0c;
    trinexx_delay, set_trinexx_delay, boss_trinexx_delay, set_boss_trinexx_delay => 0xb0d;
    trinexx_startup_delay, set_trinexx_startup_delay, boss_trinexx_startup_delay, set_boss_trinexx_startup_delay => 0xb0e;
    trinexx_tail_phase_low, set_trinexx_tail_phase_low, boss_trinexx_tail_phase_low, set_boss_trinexx_tail_phase_low => 0xb0f;
    trinexx_ready, set_trinexx_ready, boss_trinexx_ready, set_boss_trinexx_ready => 0xb10;
}
macro_rules! boss_word_fields {
    ($($get:ident, $set:ident, $read:ident, $write:ident => $address:literal;)*) => {
        impl BossWorkspaceRef<'_> { $(pub fn $get(&self) -> u16 {
            u16::from_le_bytes([self.bytes[$address], self.bytes[$address + 1]]) })* }
        impl BossWorkspaceMut<'_> { $(pub fn $get(&self) -> u16 { self.as_read_only().$get() }
            pub fn $set(&mut self, value: u16) { self.bytes[$address..$address + 2].copy_from_slice(&value.to_le_bytes()); })* }
        impl Ram<'_> { $(pub fn $read(&self) -> u16 { self.boss_workspace().$get() }
            pub fn $write(&mut self, value: u16) { self.boss_workspace_mut().$set(value); })* }
    };
}
// The odd words below intentionally include neighboring byte fields. In
// particular Trinexx's phase HIGH byte IS the ready flag, never a private copy.
boss_word_fields! {
    orbit_phase, set_orbit_phase, boss_orbit_phase, set_boss_orbit_phase => 0xb08;
    helmasaur_phase, set_helmasaur_phase, boss_helmasaur_phase, set_boss_helmasaur_phase => 0xb2d;
    helmasaur_velocity, set_helmasaur_velocity, boss_helmasaur_velocity, set_boss_helmasaur_velocity => 0xb31;
    trinexx_tail_phase, set_trinexx_tail_phase, boss_trinexx_tail_phase, set_boss_trinexx_tail_phase => 0xb0f;
}

#[cfg(test)]
mod frozen_boss;
#[cfg(test)]
mod frozen_tests;
#[cfg(test)]
mod tests;
