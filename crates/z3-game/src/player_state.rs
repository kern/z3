//! Named, live views of player state in the canonical WRAM image.
//!
//! These views borrow the existing bytes; they never cache or duplicate state.
//! Byte accessors intentionally overlap the coordinate words, matching the
//! aliases in `src/variables.h`. Values retain their original bit patterns:
//! callers choose the signed interpretation or wrapping arithmetic they need.

use z3_core::WRAM_SIZE;

use crate::ram::Ram;

/// Read-only access to the current player state.
#[derive(Clone, Copy)]
pub struct PlayerRef<'a> {
    bytes: &'a [u8; WRAM_SIZE],
}

impl<'a> PlayerRef<'a> {
    pub(crate) fn new(bytes: &'a [u8; WRAM_SIZE]) -> Self {
        Self { bytes }
    }
}

/// Exclusive access to the current player state, backed by the same WRAM.
pub struct PlayerMut<'a> {
    bytes: &'a mut [u8; WRAM_SIZE],
}

impl<'a> PlayerMut<'a> {
    pub(crate) fn new(bytes: &'a mut [u8; WRAM_SIZE]) -> Self {
        Self { bytes }
    }

    #[must_use]
    pub fn as_read_only(&self) -> PlayerRef<'_> {
        PlayerRef::new(self.bytes)
    }
}

// Keep each field's legacy location and width in one table. The Ram setter
// conveniences preserve two-phase borrowing for read/modify/write expressions
// such as `ram.set_player_x(ram.player().x().wrapping_add(1))`.
macro_rules! byte_fields {
    ($($get:ident, $set:ident, $ram_set:ident => $address:literal;)*) => {
        impl PlayerRef<'_> {
            $(#[must_use]
            pub fn $get(&self) -> u8 {
                self.bytes[$address]
            })*
        }

        impl PlayerMut<'_> {
            $(#[must_use]
            pub fn $get(&self) -> u8 {
                self.as_read_only().$get()
            }

            pub fn $set(&mut self, value: u8) {
                self.bytes[$address] = value;
            })*
        }

        impl Ram<'_> {
            $(pub fn $ram_set(&mut self, value: u8) {
                self.player_mut().$set(value);
            })*
        }
    };
}

macro_rules! word_fields {
    ($($get:ident, $set:ident, $ram_set:ident => $address:literal;)*) => {
        impl PlayerRef<'_> {
            $(#[must_use]
            pub fn $get(&self) -> u16 {
                u16::from_le_bytes([self.bytes[$address], self.bytes[$address + 1]])
            })*
        }

        impl PlayerMut<'_> {
            $(#[must_use]
            pub fn $get(&self) -> u16 {
                self.as_read_only().$get()
            }

            pub fn $set(&mut self, value: u16) {
                self.bytes[$address..$address + 2].copy_from_slice(&value.to_le_bytes());
            })*
        }

        impl Ram<'_> {
            $(pub fn $ram_set(&mut self, value: u16) {
                self.player_mut().$set(value);
            })*
        }
    };
}

// Coordinates and inventory word aliases from variables.h.
word_fields! {
    y, set_y, set_player_y => 0x20;
    x, set_x, set_player_x => 0x22;
    z, set_z, set_player_z => 0x24;
    rupees_goal, set_rupees_goal, set_player_rupees_goal => 0xf360;
    rupees, set_rupees, set_player_rupees => 0xf362;
}

byte_fields! {
    y_low, set_y_low, set_player_y_low => 0x20;
    y_high, set_y_high, set_player_y_high => 0x21;
    x_low, set_x_low, set_player_x_low => 0x22;
    x_high, set_x_high, set_player_x_high => 0x23;
    z_low, set_z_low, set_player_z_low => 0x24;
    z_high, set_z_high, set_player_z_high => 0x25;
    actual_velocity_y, set_actual_velocity_y, set_player_actual_velocity_y => 0x27;
    actual_velocity_x, set_actual_velocity_x, set_player_actual_velocity_x => 0x28;
    actual_velocity_z, set_actual_velocity_z, set_player_actual_velocity_z => 0x29;
    subpixel_y, set_subpixel_y, set_player_subpixel_y => 0x2a;
    subpixel_x, set_subpixel_x, set_player_subpixel_x => 0x2b;
    subpixel_z, set_subpixel_z, set_player_subpixel_z => 0x2c;
    facing, set_facing, set_player_facing => 0x2f;
    velocity_y, set_velocity_y, set_player_velocity_y => 0x30;
    velocity_x, set_velocity_x, set_player_velocity_x => 0x31;
    action_state, set_action_state, set_player_action_state => 0x5d;
    bow, set_bow, set_player_bow => 0xf340;
    boomerang, set_boomerang, set_player_boomerang => 0xf341;
    bombs, set_bombs, set_player_bombs => 0xf343;
    gloves, set_gloves, set_player_gloves => 0xf354;
    boots, set_boots, set_player_boots => 0xf355;
    flippers, set_flippers, set_player_flippers => 0xf356;
    moon_pearl, set_moon_pearl, set_player_moon_pearl => 0xf357;
    sword, set_sword, set_player_sword => 0xf359;
    shield, set_shield, set_player_shield => 0xf35a;
    armor, set_armor, set_player_armor => 0xf35b;
    max_health, set_max_health, set_player_max_health => 0xf36c;
    health, set_health, set_player_health => 0xf36d;
    magic, set_magic, set_player_magic => 0xf36e;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_words_and_bytes_are_live_aliases() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.player_mut().set_x(0xabcd);
        assert_eq!(ram.slice(0x22, 2), &[0xcd, 0xab]);
        assert_eq!(ram.player().x_low(), 0xcd);
        ram.player_mut().set_x_high(0x12);
        assert_eq!(ram.player().x(), 0x12cd);
        ram.write_u8(0x22, 0x34);
        assert_eq!(ram.player().x(), 0x1234);
        ram.write_u16(0x20, 0x5678);
        assert_eq!(ram.player_mut().y(), 0x5678);
        ram.player_mut().set_y_low(0xff);
        assert_eq!(ram.read_u16(0x20), 0x56ff);
    }

    #[test]
    fn subpixels_and_velocity_preserve_wrapping_and_signed_bits() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.set_player_x(0xffff);
        ram.player_mut().set_subpixel_x(0xfe);
        ram.player_mut().set_velocity_x((-2i8) as u8);
        let (subpixel, carry) = ram.player().subpixel_x().overflowing_add(3);
        ram.set_player_subpixel_x(subpixel);
        ram.set_player_x(ram.player().x().wrapping_add(u16::from(carry)));
        assert_eq!(ram.player().x(), 0);
        assert_eq!(ram.player().subpixel_x(), 1);
        assert_eq!(ram.player().velocity_x() as i8, -2);
        // Adjacent subpixel and velocity fields retain their word aliases.
        ram.write_u16(0x2a, 0x1234);
        assert_eq!(ram.player().subpixel_y(), 0x34);
        assert_eq!(ram.player().subpixel_x(), 0x12);
        ram.write_u16(0x30, 0x80ff);
        assert_eq!(ram.player().velocity_y(), 0xff);
        assert_eq!(ram.player().velocity_x(), 0x80);
    }

    #[test]
    fn named_writes_match_the_complete_legacy_memory_image() {
        let mut expected = std::array::from_fn(|index| index as u8);
        let mut actual = expected;
        {
            let mut ram = Ram::new(&mut expected);
            ram.write_u16(0x20, 0xfffe);
            ram.write_u16(0x22, 0xabcd);
            ram.write_u8(0x23, 0x12);
            ram.write_u16(0x24, 0x8000);
            ram.write_u8(0x2a, 0x78);
            ram.write_u8(0x2b, 0x56);
            ram.write_u8(0x2c, 0x34);
            ram.write_u8(0x2f, 6);
            ram.write_u8(0x30, 0xfe);
            ram.write_u8(0x31, 2);
            ram.write_u8(0x5d, 0xff);
            ram.write_u8(0xf359, 4);
            ram.write_u8(0xf35a, 3);
            ram.write_u8(0xf35b, 2);
            ram.write_u8(0xf36c, 160);
            ram.write_u8(0xf36d, 152);
            ram.write_u8(0xf36e, 128);
            ram.write_u16(0xf362, 999);
        }
        {
            let mut ram = Ram::new(&mut actual);
            let mut player = ram.player_mut();
            player.set_y(0xfffe);
            player.set_x(0xabcd);
            player.set_x_high(0x12);
            player.set_z(0x8000);
            player.set_subpixel_y(0x78);
            player.set_subpixel_x(0x56);
            player.set_subpixel_z(0x34);
            player.set_facing(6);
            player.set_velocity_y(0xfe);
            player.set_velocity_x(2);
            player.set_action_state(0xff);
            player.set_sword(4);
            player.set_shield(3);
            player.set_armor(2);
            player.set_max_health(160);
            player.set_health(152);
            player.set_magic(128);
            player.set_rupees(999);
        }
        assert_eq!(actual.as_slice(), expected.as_slice());
    }
}
