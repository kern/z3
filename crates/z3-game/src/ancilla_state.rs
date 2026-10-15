//! Named live fields for the projectile/effect pool.
//!
//! The layout follows `variables.h`, including the short five-byte scratch
//! arrays: accessing their upper effect slots intentionally aliases later
//! fields. Views never copy, clear, normalize, or compact inactive slots.

use z3_core::WRAM_SIZE;

use crate::ancilla::{Ancilla, AncillaRuntime};

#[derive(Clone, Copy)]
pub struct AncillaRef<'a> {
    bytes: &'a [u8; WRAM_SIZE],
    slot: usize,
}

impl<'a> AncillaRef<'a> {
    pub(crate) fn new(bytes: &'a [u8; WRAM_SIZE], slot: usize) -> Self {
        Self { bytes, slot }
    }

    /// Carrying an object temporarily stores a word starting at its A byte.
    /// This intentionally includes the next slot (or the first B byte).
    #[must_use]
    pub fn carried_player_y(&self) -> u16 {
        let address = 0x38a + self.slot;
        u16::from_le_bytes([self.bytes[address], self.bytes[address + 1]])
    }
}

pub struct AncillaMut<'a> {
    bytes: &'a mut [u8; WRAM_SIZE],
    slot: usize,
}

impl<'a> AncillaMut<'a> {
    pub(crate) fn new(bytes: &'a mut [u8; WRAM_SIZE], slot: usize) -> Self {
        Self { bytes, slot }
    }

    #[must_use]
    pub fn as_read_only(&self) -> AncillaRef<'_> {
        AncillaRef::new(self.bytes, self.slot)
    }

    pub fn set_carried_player_y(&mut self, value: u16) {
        let address = 0x38a + self.slot;
        self.bytes[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }
}

impl<R: AncillaRuntime> Ancilla<'_, '_, R> {
    #[must_use]
    pub fn carried_player_y(&self, slot: usize) -> u16 {
        self.ram.ancilla(slot).carried_player_y()
    }

    pub fn set_carried_player_y(&mut self, slot: usize, value: u16) {
        self.ram.ancilla_mut(slot).set_carried_player_y(value);
    }
}

// Each field retains its original byte width and bits. The forwarding methods
// on Ancilla retain Rust's two-phase borrowing for nested read/modify/write
// expressions, without holding a mutable view across callbacks.
macro_rules! byte_fields {
    ($($get:ident, $set:ident => $address:literal;)*) => {
        impl AncillaRef<'_> {
            $(#[must_use]
            pub fn $get(&self) -> u8 { self.bytes[$address + self.slot] })*
        }
        impl AncillaMut<'_> {
            $(#[must_use]
            pub fn $get(&self) -> u8 { self.as_read_only().$get() }
            pub fn $set(&mut self, value: u8) { self.bytes[$address + self.slot] = value; })*
        }
        impl<R: AncillaRuntime> Ancilla<'_, '_, R> {
            $(#[must_use]
            pub fn $get(&self, slot: usize) -> u8 { self.ram.ancilla(slot).$get() }
            pub fn $set(&mut self, slot: usize, value: u8) {
                self.ram.ancilla_mut(slot).$set(value);
            })*
        }
    };
}

byte_fields! {
    object_priority, set_object_priority => 0x280;
    state_u, set_state_u => 0x28a;
    z_velocity, set_z_velocity => 0x294;
    height, set_height => 0x29e;
    z_fraction, set_z_fraction => 0x2a8;
    state_k, set_state_k => 0x380;
    state_l, set_state_l => 0x385;
    state_a, set_state_a => 0x38a;
    state_b, set_state_b => 0x38f;
    state_g, set_state_g => 0x394;
    arr3, set_arr3 => 0x39f;
    arr1, set_arr1 => 0x3a4;
    state_s, set_state_s => 0x3a9;
    aux_timer, set_aux_timer => 0x3b1;
    state_h, set_state_h => 0x3c5;
    floor2, set_floor2 => 0x3ca;
    arr23, set_arr23 => 0x3cf;
    state_t, set_state_t => 0x3d5;
    arr24, set_arr24 => 0x3db;
    tile_attribute, set_tile_attribute => 0x3e4;
    state_r, set_state_r => 0x3ea;
    arr26, set_arr26 => 0x741;
    arr25, set_arr25 => 0x746;
    arr22, set_arr22 => 0x74b;
    arr4, set_arr4 => 0xbf0;
    y_low, set_y_low => 0xbfa;
    x_low, set_x_low => 0xc04;
    y_high, set_y_high => 0xc0e;
    x_high, set_x_high => 0xc18;
    y_velocity, set_y_velocity => 0xc22;
    x_velocity, set_x_velocity => 0xc2c;
    y_fraction, set_y_fraction => 0xc36;
    x_fraction, set_x_fraction => 0xc40;
    kind, set_kind => 0xc4a;
    step, set_step => 0xc54;
    item_to_link, set_item_to_link => 0xc5e;
    timer, set_timer => 0xc68;
    direction, set_direction => 0xc72;
    floor, set_floor => 0xc7c;
    oam_index, set_oam_index => 0xc86;
    sprite_count, set_sprite_count => 0xc90;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::Ram;

    #[test]
    fn short_array_aliases_and_raw_patches_are_immediately_visible() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.ancilla_mut(9).set_state_k(0x34);
        assert_eq!(ram.ancilla(4).state_l(), 0x34);
        ram.write_u8(0x385 + 4, 0x56);
        assert_eq!(ram.ancilla(9).state_k(), 0x56);
        ram.ancilla_mut(5).set_state_a(0x78);
        assert_eq!(ram.ancilla(0).state_b(), 0x78);
        ram.ancilla_mut(4).set_carried_player_y(0xabcd);
        assert_eq!(ram.ancilla(4).state_a(), 0xcd);
        assert_eq!(ram.ancilla(0).state_b(), 0xab);
        assert_eq!(ram.ancilla(4).carried_player_y(), 0xabcd);
    }

    #[test]
    fn writes_retain_inactive_slots_and_all_neighboring_bytes() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut expected = bytes.to_vec();
        let mut ram = Ram::new(&mut bytes);
        for slot in [0, 9] {
            ram.ancilla_mut(slot).set_kind(0);
            ram.ancilla_mut(slot).set_timer(0xff);
            ram.ancilla_mut(slot).set_x_velocity(0x80);
            expected[0xc4a + slot] = 0;
            expected[0xc68 + slot] = 0xff;
            expected[0xc2c + slot] = 0x80;
        }
        assert_eq!(ram.slice(0, WRAM_SIZE), expected);
    }
}
