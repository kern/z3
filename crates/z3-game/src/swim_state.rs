//! Live swimming/momentum axis state in canonical WRAM.
//!
//! Words, byte aliases, gaps and unknown values remain lossless.

use crate::ram::Ram;
use z3_core::WRAM_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwimAxis {
    Vertical,
    Horizontal,
}

impl SwimAxis {
    pub const FORWARD: [Self; 2] = [Self::Vertical, Self::Horizontal];
    pub const REVERSE: [Self; 2] = [Self::Horizontal, Self::Vertical];
    pub const fn index(self) -> usize {
        match self {
            Self::Vertical => 0,
            Self::Horizontal => 1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SwimAxisRef<'a> {
    bytes: &'a [u8],
    axis: SwimAxis,
}
pub struct SwimAxisMut<'a> {
    bytes: &'a mut [u8],
    axis: SwimAxis,
}

impl SwimAxisRef<'_> {
    fn word(self, base: usize) -> u16 {
        let address = base + self.axis.index() * 2;
        u16::from_le_bytes([self.bytes[address], self.bytes[address + 1]])
    }
    pub fn mode_low(self) -> u8 {
        self.bytes[0x32b + self.axis.index() * 2]
    }
}

impl SwimAxisMut<'_> {
    fn set_word(&mut self, base: usize, value: u16) {
        let address = base + self.axis.index() * 2;
        self.bytes[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }
    pub fn set_mode_low(&mut self, value: u8) {
        self.bytes[0x32b + self.axis.index() * 2] = value;
    }
}

macro_rules! fields {
    ($($get:ident, $set:ident, $ram_set:ident => $base:literal;)*) => {
        impl SwimAxisRef<'_> { $(pub fn $get(self) -> u16 { self.word($base) })* }
        impl SwimAxisMut<'_> { $(pub fn $set(&mut self, value: u16) { self.set_word($base, value); })* }
        impl Ram<'_> { $(pub fn $ram_set(&mut self, axis: SwimAxis, value: u16) {
            self.swim_axis_mut(axis).$set(value);
        })* }

        #[cfg(test)]
        fn check_every_field_codec() {
            let mut bytes = [0xa5; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            $(for axis in SwimAxis::FORWARD {
                let address = $base + axis.index() * 2;
                let mut expected = ram.slice(0, WRAM_SIZE).to_vec();
                for value in 0..=u16::MAX {
                    ram.$ram_set(axis, value);
                    assert_eq!(ram.swim_axis(axis).$get(), value);
                    assert_eq!(ram.read_u8(address), value as u8);
                    assert_eq!(ram.read_u8(address + 1), (value >> 8) as u8);
                    // Independent byte writes remain live through each word view.
                    ram.write_u8(address, 0x5a);
                    assert_eq!(ram.swim_axis(axis).$get(), (value & 0xff00) | 0x5a);
                    ram.write_u8(address + 1, 0xa5);
                    assert_eq!(ram.swim_axis(axis).$get(), 0xa55a);
                }
                expected[address..address + 2].copy_from_slice(&0xa55au16.to_le_bytes());
                assert_eq!(ram.slice(0, WRAM_SIZE), expected);
            })*
        }
    };
}

fields! {
    acceleration_latch, set_acceleration_latch, set_swim_acceleration_latch => 0x32f;
    hold_timer, set_hold_timer, set_swim_hold_timer => 0x326;
    acceleration_mode, set_acceleration_mode, set_swim_acceleration_mode => 0x32b;
    speed, set_speed, set_swim_speed => 0x33c;
    speed_limit, set_speed_limit, set_swim_speed_limit => 0x334;
    direction_selector, set_direction_selector, set_swim_direction_selector => 0x338;
}

impl Ram<'_> {
    pub fn swim_axis(&self, axis: SwimAxis) -> SwimAxisRef<'_> {
        SwimAxisRef {
            bytes: self.slice(0, WRAM_SIZE),
            axis,
        }
    }
    pub fn swim_axis_mut(&mut self, axis: SwimAxis) -> SwimAxisMut<'_> {
        SwimAxisMut {
            bytes: self.bytes_mut(),
            axis,
        }
    }
    pub fn set_swim_mode_low(&mut self, axis: SwimAxis, value: u8) {
        self.swim_axis_mut(axis).set_mode_low(value);
    }

    /// Exact ResetAllAcceleration order; directions and gaps intentionally live.
    pub fn reset_swim_acceleration(&mut self) {
        for axis in SwimAxis::FORWARD {
            self.set_swim_acceleration_latch(axis, 0);
        }
        for axis in SwimAxis::FORWARD {
            self.set_swim_hold_timer(axis, 0);
        }
        for axis in SwimAxis::FORWARD {
            self.set_swim_acceleration_mode(axis, 0);
        }
        for axis in SwimAxis::FORWARD {
            self.set_swim_speed(axis, 0);
        }
        for axis in SwimAxis::FORWARD {
            self.set_swim_speed_limit(axis, 0);
        }
    }

    /// Collision reset is a different four-field operation, not an axis default.
    pub fn clear_blocked_swim_axis(&mut self, axis: SwimAxis) {
        self.set_swim_acceleration_latch(axis, 0);
        self.set_swim_acceleration_mode(axis, 0);
        self.set_swim_speed(axis, 0);
        self.set_swim_speed_limit(axis, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_six_fields_exhaustive_little_endian_codec_and_live_byte_aliases() {
        check_every_field_codec();
    }
    #[test]
    fn exhaustive_mode_codec_and_low_byte_are_live_without_gap_writes() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for axis in SwimAxis::FORWARD {
            let address = 0x32b + axis.index() * 2;
            for value in 0..=u16::MAX {
                ram.set_swim_acceleration_mode(axis, value);
                assert_eq!(ram.swim_axis(axis).acceleration_mode(), value);
                assert_eq!(ram.read_u16(address), value);
                assert_eq!(ram.swim_axis(axis).mode_low(), value as u8);
                ram.set_swim_mode_low(axis, 0x5a);
                assert_eq!(
                    ram.swim_axis(axis).acceleration_mode(),
                    (value & 0xff00) | 0x5a
                );
            }
        }
        assert_eq!(ram.read_u8(0x32a), 0xa5);
        assert_eq!(ram.read_u8(0x32e), 0xff);
        assert_eq!(ram.read_u8(0x333), 0xa5);
    }

    #[test]
    fn reset_preserves_every_other_wram_byte_including_both_direction_words() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut expected = bytes;
        for address in [
            0x32f, 0x331, 0x326, 0x328, 0x32b, 0x32d, 0x33c, 0x33e, 0x334, 0x336,
        ] {
            expected[address..address + 2].fill(0);
        }
        Ram::new(&mut bytes).reset_swim_acceleration();
        assert_eq!(bytes, expected);
    }

    #[test]
    fn blocked_axis_reset_changes_only_four_words_and_retains_timer_and_selector() {
        for axis in SwimAxis::FORWARD {
            let mut bytes = [0x5a; WRAM_SIZE];
            let mut expected = bytes;
            for base in [0x32f, 0x32b, 0x33c, 0x334] {
                let address = base + axis.index() * 2;
                expected[address..address + 2].fill(0);
            }
            Ram::new(&mut bytes).clear_blocked_swim_axis(axis);
            assert_eq!(bytes, expected);
        }
    }

    #[test]
    fn raw_selector_mode_and_latch_are_never_normalized_or_cached() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for axis in SwimAxis::FORWARD {
            ram.set_swim_direction_selector(axis, 0xffff);
            ram.set_swim_acceleration_mode(axis, 0xabcd);
            ram.set_swim_acceleration_latch(axis, 0x8000);
            assert_eq!(ram.swim_axis(axis).direction_selector(), 0xffff);
            assert_eq!(ram.swim_axis(axis).acceleration_latch(), 0x8000);
            ram.set_swim_mode_low(axis, 0x12);
            assert_eq!(ram.swim_axis(axis).acceleration_mode(), 0xab12);
        }
    }
}

#[cfg(test)]
mod frozen;
#[cfg(test)]
mod oracle_tests;
