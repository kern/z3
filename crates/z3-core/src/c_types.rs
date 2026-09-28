//! Exact-width equivalents of the aliases and small value types in `src/types.h`.
//!
//! The parity port uses Rust's fixed-width integer names at call sites.  The
//! helpers here spell out the behavior of the C macros which depended on
//! unsigned wraparound or the little-endian target architecture.

/// The C build enables the extended side area unconditionally.
pub const ENABLE_LARGE_SCREEN: bool = true;
/// Number of extra PPU pixels on each horizontal side in the C build.
pub const PPU_EXTRA_LEFT_RIGHT: usize = if ENABLE_LARGE_SCREEN { 96 } else { 0 };

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Point16U {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct PointU8 {
    pub x: u8,
    pub y: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Pair16U {
    pub a: u16,
    pub b: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct PairU8 {
    pub a: u8,
    pub b: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct ProjectSpeedRet {
    pub x: u8,
    pub y: u8,
    pub xdiff: u8,
    pub ydiff: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct OamEnt {
    pub x: u8,
    pub y: u8,
    pub charnum: u8,
    pub flags: u8,
}

/// Safe borrowed equivalent of the C `MemBlk { ptr, size }` pair.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemBlk<'a> {
    pub bytes: &'a [u8],
}

impl<'a> MemBlk<'a> {
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.bytes.len()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }
}

#[must_use]
pub const fn sign8(value: u8) -> bool {
    value & 0x80 != 0
}

#[must_use]
pub const fn sign16(value: u16) -> bool {
    value & 0x8000 != 0
}

/// C's `abs8` treats its argument as unsigned two's-complement data.
#[must_use]
pub const fn abs8(value: u8) -> u8 {
    if sign8(value) {
        value.wrapping_neg()
    } else {
        value
    }
}

/// C's `abs16` treats its argument as unsigned two's-complement data.
#[must_use]
pub const fn abs16(value: u16) -> u16 {
    if sign16(value) {
        value.wrapping_neg()
    } else {
        value
    }
}

#[must_use]
pub const fn int_min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

#[must_use]
pub const fn int_max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

#[must_use]
pub const fn uint_min(a: u32, b: u32) -> u32 {
    if a < b { a } else { b }
}

#[must_use]
pub const fn uint_max(a: u32, b: u32) -> u32 {
    if a > b { a } else { b }
}

#[must_use]
pub const fn low_byte(value: u32) -> u8 {
    value as u8
}

#[must_use]
pub const fn high_byte(value: u32) -> u8 {
    (value >> 8) as u8
}

#[must_use]
pub const fn low_word(value: u32) -> u16 {
    value as u16
}

#[must_use]
pub const fn swap16(value: u16) -> u16 {
    value.swap_bytes()
}

/// Equivalent to the C `XY(x, y)` macro, including unsigned wraparound.
#[must_use]
pub const fn xy(x: u16, y: u16) -> u16 {
    y.wrapping_mul(64).wrapping_add(x)
}

/// Loads a 24-bit little-endian integer, matching the supported C targets.
#[must_use]
pub const fn load24(bytes: [u8; 3]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])
}

/// Rust equivalent of the reference program's fatal `Die` helper.
#[cold]
#[track_caller]
pub fn die(error: &str) -> ! {
    panic!("{error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_absolute_value_matches_c_wraparound() {
        assert_eq!(abs8(0x7f), 0x7f);
        assert_eq!(abs8(0xff), 1);
        assert_eq!(abs8(0x80), 0x80);
        assert_eq!(abs16(0xffff), 1);
        assert_eq!(abs16(0x8000), 0x8000);
        assert_eq!(int_min(-4, 3), -4);
        assert_eq!(int_max(-4, 3), 3);
        assert_eq!(uint_min(4, 3), 3);
        assert_eq!(uint_max(4, 3), 4);
    }

    #[test]
    fn endian_and_coordinate_helpers_match_macros() {
        assert_eq!(low_byte(0x1234_abcd), 0xcd);
        assert_eq!(high_byte(0x1234_abcd), 0xab);
        assert_eq!(low_word(0x1234_abcd), 0xabcd);
        assert_eq!(swap16(0x1234), 0x3412);
        assert_eq!(load24([0x12, 0x34, 0x56]), 0x56_3412);
        assert_eq!(xy(5, 3), 197);
        assert_eq!(xy(u16::MAX, 1), 63);
    }

    #[test]
    fn translated_value_types_have_c_layout_sizes() {
        assert_eq!(size_of::<Point16U>(), 4);
        assert_eq!(size_of::<PointU8>(), 2);
        assert_eq!(size_of::<Pair16U>(), 4);
        assert_eq!(size_of::<PairU8>(), 2);
        assert_eq!(size_of::<ProjectSpeedRet>(), 4);
        assert_eq!(size_of::<OamEnt>(), 4);
    }
}
