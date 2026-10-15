//! Fixed-point positions and their compatibility storage.
//!
//! Positions have eight fractional bits; movement velocities have four. Keeping
//! those units explicit here preserves carries, negative movement, and wrapping
//! without making gameplay callers assemble split bytes themselves.

use crate::ram::Ram;

/// Integer coordinates in separate low/high arrays. Controllers and follower
/// history use these without allocating nonexistent fractional position bytes.
#[derive(Clone, Copy)]
pub(crate) struct SplitCoordinate {
    low: usize,
    high: usize,
}

impl SplitCoordinate {
    pub(crate) const fn new(low: usize, high: usize) -> Self {
        Self { low, high }
    }

    pub(crate) fn pixels(self, ram: &Ram<'_>, slot: usize) -> u16 {
        u16::from_le_bytes([ram.read_u8(self.low + slot), ram.read_u8(self.high + slot)])
    }

    pub(crate) fn set_pixels(self, ram: &mut Ram<'_>, slot: usize, pixels: u16) {
        let [low, high] = pixels.to_le_bytes();
        ram.write_u8(self.low + slot, low);
        ram.write_u8(self.high + slot, high);
    }
}

/// An unsigned wrapping coordinate with eight fractional bits.
///
/// `INTEGER_BITS` is 8 for height or 16 for world coordinates. The integer part
/// is a bit pattern: callers may interpret it as signed for height comparisons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FixedPosition<const INTEGER_BITS: u32>(u32);

pub type WorldPosition = FixedPosition<16>;
pub type Height = FixedPosition<8>;

impl<const INTEGER_BITS: u32> FixedPosition<INTEGER_BITS> {
    const MASK: u32 = {
        assert!(INTEGER_BITS == 8 || INTEGER_BITS == 16);
        (1 << (INTEGER_BITS + 8)) - 1
    };

    #[must_use]
    pub const fn from_parts(pixels: u16, fraction: u8) -> Self {
        Self(((pixels as u32) << 8 | fraction as u32) & Self::MASK)
    }

    #[must_use]
    pub const fn pixels(self) -> u16 {
        (self.0 >> 8) as u16
    }

    #[must_use]
    pub const fn fraction(self) -> u8 {
        self.0 as u8
    }

    /// Advances by a signed velocity measured in sixteenths of a pixel.
    ///
    /// Conversion to the position's 256ths happens before wrapping. In
    /// particular, -1 moves back 16 fractional units, borrowing one pixel when
    /// necessary; the most negative velocity (-128) moves back eight pixels.
    #[must_use]
    pub const fn advanced(self, velocity: i8) -> Self {
        let delta = (velocity as i32) * 16;
        Self(self.0.wrapping_add(delta as u32) & Self::MASK)
    }
}

/// Layout of parallel coordinate arrays in the legacy image.
///
/// This describes storage, not a cached position. Every operation accesses the
/// live image so callbacks and legacy replay patches observe changes at once.
#[derive(Clone, Copy)]
pub(crate) struct SplitPosition<const INTEGER_BITS: u32> {
    fraction: usize,
    low: usize,
    high: Option<usize>,
}

impl SplitPosition<16> {
    pub(crate) const fn world(fraction: usize, low: usize, high: usize) -> Self {
        Self {
            fraction,
            low,
            high: Some(high),
        }
    }
}

impl SplitPosition<8> {
    pub(crate) const fn height(fraction: usize, pixels: usize) -> Self {
        Self {
            fraction,
            low: pixels,
            high: None,
        }
    }
}

impl<const INTEGER_BITS: u32> SplitPosition<INTEGER_BITS> {
    pub(crate) fn pixels(self, ram: &Ram<'_>, slot: usize) -> u16 {
        let low = ram.read_u8(self.low + slot);
        let high = self.high.map_or(0, |base| ram.read_u8(base + slot));
        u16::from_le_bytes([low, high])
    }

    /// Changes only whole pixels, retaining the existing fractional byte.
    pub(crate) fn set_pixels(self, ram: &mut Ram<'_>, slot: usize, pixels: u16) {
        let [low, high] = pixels.to_le_bytes();
        ram.write_u8(self.low + slot, low);
        if let Some(base) = self.high {
            ram.write_u8(base + slot, high);
        }
    }

    pub(crate) fn load(self, ram: &Ram<'_>, slot: usize) -> FixedPosition<INTEGER_BITS> {
        let fraction = ram.read_u8(self.fraction + slot);
        FixedPosition::from_parts(self.pixels(ram, slot), fraction)
    }

    pub(crate) fn store(
        self,
        ram: &mut Ram<'_>,
        slot: usize,
        position: FixedPosition<INTEGER_BITS>,
    ) {
        // C writes the fractional byte before the integer low and high bytes.
        ram.write_u8(self.fraction + slot, position.fraction());
        self.set_pixels(ram, slot, position.pixels());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carries_borrows_and_wraps_at_each_coordinate_width() {
        assert_eq!(
            WorldPosition::from_parts(0x12ff, 0xf8).advanced(1),
            WorldPosition::from_parts(0x1300, 8)
        );
        assert_eq!(
            WorldPosition::from_parts(0xffff, 0xf0).advanced(1),
            WorldPosition::from_parts(0, 0)
        );
        assert_eq!(
            WorldPosition::from_parts(0, 0).advanced(-1),
            WorldPosition::from_parts(0xffff, 0xf0)
        );
        assert_eq!(
            Height::from_parts(0xff, 0xf0).advanced(1),
            Height::from_parts(0, 0)
        );
        assert_eq!(
            Height::from_parts(0, 0).advanced(-1),
            Height::from_parts(0xff, 0xf0)
        );
    }

    #[test]
    fn every_velocity_matches_c_integer_promotion_at_fractional_boundaries() {
        for integer in [0, 1, 0xff, 0x100, 0x7fff, 0xffff] {
            for fraction in [0, 1, 15, 16, 127, 240, 255] {
                for velocity in i8::MIN..=i8::MAX {
                    let c_sum = u32::from(fraction)
                        .wrapping_add(u32::from(integer) << 8)
                        .wrapping_add((i32::from(velocity) << 4) as u32);
                    let world = WorldPosition::from_parts(integer, fraction).advanced(velocity);
                    assert_eq!(world.pixels(), (c_sum >> 8) as u16);
                    assert_eq!(world.fraction(), c_sum as u8);
                    let height = Height::from_parts(integer, fraction).advanced(velocity);
                    assert_eq!(height.pixels(), u16::from((c_sum >> 8) as u8));
                    assert_eq!(height.fraction(), c_sum as u8);
                }
            }
        }
    }
}
