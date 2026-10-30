//! Live, non-compacting Garnish effect-pool storage from variables.h.
//! Nominal slots are 0..30, but legacy raw indices intentionally remain
//! physical-WRAM indexed: overwrite cursor 0x80 yields raw slot 127.
use crate::ram::Ram;
use z3_core::WRAM_SIZE;

pub const SLOT_COUNT: usize = 30;
pub const LOWER_SLOT_COUNT: usize = 15;
pub const ACTIVE_MARKER_ADDRESS: usize = 0xfb4;
pub const SHARED_OVERWRITE_CURSOR_ADDRESS: usize = 0xff8;

#[derive(Clone, Copy)]
pub struct GarnishRef<'a> {
    bytes: &'a [u8],
    slot: usize,
}
pub struct GarnishMut<'a> {
    bytes: &'a mut [u8; WRAM_SIZE],
    slot: usize,
}
impl<'a> GarnishRef<'a> {
    pub(crate) fn new(bytes: &'a [u8], slot: usize) -> Self {
        Self { bytes, slot }
    }
    pub fn x(&self) -> u16 {
        u16::from(self.x_low()) | u16::from(self.x_high()) << 8
    }
    pub fn y(&self) -> u16 {
        u16::from(self.y_low()) | u16::from(self.y_high()) << 8
    }
}
impl<'a> GarnishMut<'a> {
    pub(crate) fn new(bytes: &'a mut [u8; WRAM_SIZE], slot: usize) -> Self {
        Self { bytes, slot }
    }
    pub fn as_read_only(&self) -> GarnishRef<'_> {
        GarnishRef::new(self.bytes, self.slot)
    }
    pub fn set_x(&mut self, value: u16) {
        self.set_x_low(value as u8);
        self.set_x_high((value >> 8) as u8);
    }
    pub fn set_y(&mut self, value: u16) {
        self.set_y_low(value as u8);
        self.set_y_high((value >> 8) as u8);
    }

    /// A zero lifetime stays live. Expiry clears only the kind byte; inactive
    /// payload and coordinate bytes must survive for subsequent allocations.
    /// Pause/kind policy belongs to the caller, before this live mutation.
    pub fn tick_lifetime(&mut self) -> bool {
        if self.countdown() == 0 {
            return false;
        }
        let remaining = self.countdown().wrapping_sub(1);
        self.set_countdown(remaining);
        if remaining == 0 {
            self.set_kind(0);
            true
        } else {
            false
        }
    }
}

/// A live reverse-allocation view. Scanning never reserves or initializes a
/// slot, so callbacks and later scans still observe canonical WRAM directly.
#[derive(Clone, Copy)]
pub struct GarnishPoolRef<'a> {
    bytes: &'a [u8],
}

impl<'a> GarnishPoolRef<'a> {
    pub fn slot(&self, index: usize) -> GarnishRef<'a> {
        GarnishRef::new(self.bytes, index)
    }
    pub fn find_free(&self, limit: i32) -> i32 {
        // Original C returns a negative limit unchanged after reading the byte
        // before the nominal pool. This interval remains inside the backing
        // WRAM allocation. Keep the prior Rust fallback outside C's domain.
        // Explicit C-parity correction, separately proved by check_garnish.
        if (-0x1f800..=-1).contains(&limit) {
            return limit;
        }
        (0..=limit)
            .rev()
            .find(|&slot| self.slot(slot as usize).kind() == 0)
            .unwrap_or(-1)
    }
    pub fn find_free_force(&self) -> usize {
        (0..SLOT_COUNT)
            .rev()
            .find(|&slot| self.slot(slot).kind() == 0)
            .unwrap_or(0)
    }
    pub fn find_free_slot(&self) -> Option<usize> {
        (0..SLOT_COUNT)
            .rev()
            .find(|&slot| self.slot(slot).kind() == 0)
    }
}
macro_rules! byte_fields {
    ($($get:ident, $set:ident => $address:literal;)*) => {
        impl GarnishRef<'_> { $(pub fn $get(&self) -> u8 { self.bytes[$address + self.slot] })* }
        impl GarnishMut<'_> {
            $(pub fn $get(&self) -> u8 { self.as_read_only().$get() }
            pub fn $set(&mut self, value: u8) { self.bytes[$address + self.slot] = value; })*
        }
    };
}
byte_fields! {
    kind, set_kind => 0x1f800;
    y_low, set_y_low => 0x1f81e;
    x_low, set_x_low => 0x1f83c;
    y_high, set_y_high => 0x1f85a;
    x_high, set_x_high => 0x1f878;
    y_velocity, set_y_velocity => 0x1f896;
    x_velocity, set_x_velocity => 0x1f8b4;
    y_fraction, set_y_fraction => 0x1f8d2;
    x_fraction, set_x_fraction => 0x1f8f0;
    countdown, set_countdown => 0x1f90e;
    source_sprite, set_source_sprite => 0x1f92c;
    animation5, set_animation5 => 0x1f94a;
    floor, set_floor => 0x1f968;
    oam_flags, set_oam_flags => 0x1f9fe;
}

// This macro expansion deliberately forwards from Ram, preserving two-phase
// borrowing for nested read/modify/write expressions with u8 expected types.
// Explicit methods below are generated rather than concatenating identifiers.
impl Ram<'_> {
    pub fn garnishes(&self) -> GarnishPoolRef<'_> {
        GarnishPoolRef {
            bytes: self.slice(0, WRAM_SIZE),
        }
    }
    pub fn garnish(&self, slot: usize) -> GarnishRef<'_> {
        GarnishRef::new(self.slice(0, WRAM_SIZE), slot)
    }
    pub fn garnish_mut(&mut self, slot: usize) -> GarnishMut<'_> {
        GarnishMut::new(self.bytes_mut(), slot)
    }
    pub fn garnish_kind(&self, slot: usize) -> u8 {
        self.garnish(slot).kind()
    }
    pub fn set_garnish_kind(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_kind(value);
    }
    pub fn garnish_y_low(&self, slot: usize) -> u8 {
        self.garnish(slot).y_low()
    }
    pub fn set_garnish_y_low(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_y_low(value);
    }
    pub fn garnish_x_low(&self, slot: usize) -> u8 {
        self.garnish(slot).x_low()
    }
    pub fn set_garnish_x_low(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_x_low(value);
    }
    pub fn garnish_y_high(&self, slot: usize) -> u8 {
        self.garnish(slot).y_high()
    }
    pub fn set_garnish_y_high(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_y_high(value);
    }
    pub fn garnish_x_high(&self, slot: usize) -> u8 {
        self.garnish(slot).x_high()
    }
    pub fn set_garnish_x_high(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_x_high(value);
    }
    pub fn garnish_y_velocity(&self, slot: usize) -> u8 {
        self.garnish(slot).y_velocity()
    }
    pub fn set_garnish_y_velocity(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_y_velocity(value);
    }
    pub fn garnish_x_velocity(&self, slot: usize) -> u8 {
        self.garnish(slot).x_velocity()
    }
    pub fn set_garnish_x_velocity(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_x_velocity(value);
    }
    pub fn garnish_y_fraction(&self, slot: usize) -> u8 {
        self.garnish(slot).y_fraction()
    }
    pub fn set_garnish_y_fraction(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_y_fraction(value);
    }
    pub fn garnish_x_fraction(&self, slot: usize) -> u8 {
        self.garnish(slot).x_fraction()
    }
    pub fn set_garnish_x_fraction(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_x_fraction(value);
    }
    pub fn garnish_countdown(&self, slot: usize) -> u8 {
        self.garnish(slot).countdown()
    }
    pub fn set_garnish_countdown(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_countdown(value);
    }
    pub fn garnish_source_sprite(&self, slot: usize) -> u8 {
        self.garnish(slot).source_sprite()
    }
    pub fn set_garnish_source_sprite(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_source_sprite(value);
    }
    pub fn garnish_animation5(&self, slot: usize) -> u8 {
        self.garnish(slot).animation5()
    }
    pub fn set_garnish_animation5(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_animation5(value);
    }
    pub fn garnish_floor(&self, slot: usize) -> u8 {
        self.garnish(slot).floor()
    }
    pub fn set_garnish_floor(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_floor(value);
    }
    pub fn garnish_oam_flags(&self, slot: usize) -> u8 {
        self.garnish(slot).oam_flags()
    }
    pub fn set_garnish_oam_flags(&mut self, slot: usize, value: u8) {
        self.garnish_mut(slot).set_oam_flags(value);
    }
    /// Shared 0xFB4 marker: NOT a derived live-slot count or a boolean.
    pub fn garnish_active_marker(&self) -> u8 {
        self.read_u8(ACTIVE_MARKER_ADDRESS)
    }
    pub fn set_garnish_active_marker(&mut self, value: u8) {
        self.write_u8(ACTIVE_MARKER_ADDRESS, value);
    }
    /// Scan only: no reservation, initialization, or active-marker change.
    pub fn garnish_find_free(&self, limit: i32) -> i32 {
        self.garnishes().find_free(limit)
    }
    pub fn garnish_find_free_force(&self) -> usize {
        self.garnishes().find_free_force()
    }
    /// 0xFF8 is a shared WRAM alias. In particular 0x80 -> 0x7F,
    /// while 0x00 -> 0xFF -> reset. No modulo/clamping is permitted.
    pub fn garnish_advance_overwrite_cursor(&mut self, reset: u8) -> usize {
        let cursor = self
            .read_u8(SHARED_OVERWRITE_CURSOR_ADDRESS)
            .wrapping_sub(1);
        let cursor = if (cursor as i8) < 0 { reset } else { cursor };
        self.write_u8(SHARED_OVERWRITE_CURSOR_ADDRESS, cursor);
        usize::from(cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded() -> Box<[u8; WRAM_SIZE]> {
        Box::new(std::array::from_fn(|i| {
            (i.wrapping_mul(37) ^ (i >> 8)) as u8
        }))
    }

    #[test]
    fn countdown_expiration_clears_only_kind_and_keeps_zero_live() {
        for slot in [0, 14, 15, 29, 127] {
            for remaining in [0u8, 1, 2, 255] {
                let mut bytes = seeded();
                bytes[0x1f800 + slot] = 5;
                bytes[0x1f90e + slot] = remaining;
                let mut expected = bytes.clone();
                if remaining != 0 {
                    expected[0x1f90e + slot] = remaining - 1;
                }
                if remaining == 1 {
                    expected[0x1f800 + slot] = 0;
                }
                let mut ram = Ram::new(&mut bytes);
                assert_eq!(ram.garnish_mut(slot).tick_lifetime(), remaining == 1);
                assert_eq!(bytes, expected);
            }
        }
    }

    #[test]
    fn all_fourteen_fields_preserve_aliases_and_every_other_byte() {
        macro_rules! field {
            ($get:ident, $set:ident, $address:literal) => {
                for slot in [0, 14, 15, 29, 30, 127, 255] {
                    let mut bytes = seeded();
                    let mut expected = bytes.clone();
                    let value = bytes[$address + slot] ^ 0xff;
                    expected[$address + slot] = value;
                    let mut ram = Ram::new(&mut bytes);
                    ram.garnish_mut(slot).$set(value);
                    assert_eq!(ram.garnish(slot).$get(), value);
                    assert_eq!(bytes, expected);
                }
            };
        }
        field!(kind, set_kind, 0x1f800);
        field!(y_low, set_y_low, 0x1f81e);
        field!(x_low, set_x_low, 0x1f83c);
        field!(y_high, set_y_high, 0x1f85a);
        field!(x_high, set_x_high, 0x1f878);
        field!(y_velocity, set_y_velocity, 0x1f896);
        field!(x_velocity, set_x_velocity, 0x1f8b4);
        field!(y_fraction, set_y_fraction, 0x1f8d2);
        field!(x_fraction, set_x_fraction, 0x1f8f0);
        field!(countdown, set_countdown, 0x1f90e);
        field!(source_sprite, set_source_sprite, 0x1f92c);
        field!(animation5, set_animation5, 0x1f94a);
        field!(floor, set_floor, 0x1f968);
        field!(oam_flags, set_oam_flags, 0x1f9fe);
        let mut bytes = seeded();
        let mut ram = Ram::new(&mut bytes);
        ram.garnish_mut(30).set_kind(0x42);
        assert_eq!(ram.garnish(0).y_low(), 0x42);
        ram.garnish_mut(0).set_y_low(0xa7);
        assert_eq!(ram.garnish(30).kind(), 0xa7);
    }

    #[test]
    fn raw_slot_and_coordinate_writes_preserve_all_other_memory() {
        for slot in [0, 14, 15, 29, 30, 127, 255] {
            for value in [0, 0xff, 0x100, 0x7fff, 0x8000, 0xffff] {
                let mut actual = seeded();
                let mut expected = actual.clone();
                expected[0x1f83c + slot] = value as u8;
                expected[0x1f878 + slot] = (value >> 8) as u8;
                expected[0x1f81e + slot] = value as u8;
                expected[0x1f85a + slot] = (value >> 8) as u8;
                let mut ram = Ram::new(&mut actual);
                ram.garnish_mut(slot).set_x(value);
                ram.garnish_mut(slot).set_y(value);
                assert_eq!(ram.garnish(slot).x(), value);
                assert_eq!(ram.garnish(slot).y(), value);
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn force_and_signed_limit_scans_do_not_reserve_or_clear_fields() {
        for free in 0..=SLOT_COUNT {
            let mut bytes = seeded();
            bytes[0x1f800..0x1f800 + SLOT_COUNT].fill(1);
            if free < SLOT_COUNT {
                bytes[0x1f800 + free] = 0;
            }
            let original = bytes.clone();
            let ram = Ram::new(&mut bytes);
            for limit in [-0x1f801, -0x1f800, -4, -1, 0, 1, 14, 15, 28, 29] {
                let expected = if (-0x1f800..=-1).contains(&limit) {
                    limit
                } else if free < SLOT_COUNT && free as i32 <= limit {
                    free as i32
                } else {
                    -1
                };
                assert_eq!(ram.garnish_find_free(limit), expected);
            }
            assert_eq!(
                ram.garnish_find_free_force(),
                if free < SLOT_COUNT { free } else { 0 }
            );
            assert_eq!(bytes, original);
        }
    }

    #[test]
    fn overwrite_signed_byte_fallback_and_alias_are_exhaustive() {
        for reset in [14, 29] {
            for initial in 0..=u8::MAX {
                let mut bytes = seeded();
                bytes[0xff8] = initial;
                let mut expected = bytes.clone();
                let decremented = initial.wrapping_sub(1);
                let result = if decremented >= 0x80 {
                    reset
                } else {
                    decremented
                };
                expected[0xff8] = result;
                let mut ram = Ram::new(&mut bytes);
                assert_eq!(
                    ram.garnish_advance_overwrite_cursor(reset),
                    usize::from(result)
                );
                assert_eq!(ram.read_u8(0xff8), result);
                assert_eq!(bytes, expected);
            }
        }
    }

    #[test]
    fn shared_active_marker_is_not_a_slot_count_or_boolean() {
        for marker in [0, 1, 5, 19, 31, 255] {
            let mut bytes = seeded();
            let mut expected = bytes.clone();
            expected[0xfb4] = marker;
            let mut ram = Ram::new(&mut bytes);
            ram.set_garnish_active_marker(marker);
            assert_eq!(ram.read_u8(0xfb4), marker);
            ram.write_u8(0xfb4, marker ^ 0xff);
            assert_eq!(ram.garnish_active_marker(), marker ^ 0xff);
            expected[0xfb4] ^= 0xff;
            assert_eq!(bytes, expected);
        }
    }
}
