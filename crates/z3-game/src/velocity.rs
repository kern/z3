//! Lossless signed-byte movement arithmetic, independent of storage layout.
//!
//! The C game promotes a signed byte to `int` before negating and truncates
//! back to a byte on assignment. Rust's direct `-i8::MIN` instead panics in
//! checked builds. These operations specify the wrapping bit-pattern result.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VelocityByte(u8);

impl VelocityByte {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn signed(self) -> i8 {
        self.0 as i8
    }

    pub const fn negated(self) -> Self {
        Self(self.0.wrapping_neg())
    }

    /// Arithmetic halving, unlike the logical shift used by some rebound paths.
    pub const fn arithmetic_half(self) -> Self {
        Self((self.signed() >> 1) as u8)
    }

    /// Move one byte step toward the target according to the sign of the
    /// *wrapped difference*. This is deliberately not ordinary signed ordering.
    pub const fn approaching(self, target: Self) -> Self {
        if self.0 == target.0 {
            self
        } else if (self.0.wrapping_sub(target.0) as i8) < 0 {
            Self(self.0.wrapping_add(1))
        } else {
            Self(self.0.wrapping_sub(1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_byte_patterns_preserve_c_promotion_negation_and_signed_halving() {
        for bits in 0..=u8::MAX {
            let velocity = VelocityByte::from_bits(bits);
            assert_eq!(velocity.bits(), bits);
            assert_eq!(velocity.signed(), bits as i8);
            assert_eq!(velocity.negated().bits(), (-i16::from(bits as i8)) as u8);
            assert_eq!(
                velocity.arithmetic_half().bits(),
                (i16::from(bits as i8) >> 1) as u8
            );
        }
        assert_eq!(VelocityByte::from_bits(0x80).negated().bits(), 0x80);
        assert_eq!(VelocityByte::from_bits(0x80).arithmetic_half().bits(), 0xc0);
    }

    #[test]
    fn all_current_target_pairs_keep_wrapped_difference_direction() {
        for current in 0..=u8::MAX {
            for target in 0..=u8::MAX {
                let expected = if current == target {
                    current
                } else if ((i16::from(current) - i16::from(target)) as u8 as i8) < 0 {
                    current.wrapping_add(1)
                } else {
                    current.wrapping_sub(1)
                };
                assert_eq!(
                    VelocityByte::from_bits(current)
                        .approaching(VelocityByte::from_bits(target))
                        .bits(),
                    expected
                );
            }
        }
        assert_eq!(
            VelocityByte::from_bits(127)
                .approaching(VelocityByte::from_bits(128))
                .bits(),
            128
        );
        assert_eq!(
            VelocityByte::from_bits(0)
                .approaching(VelocityByte::from_bits(255))
                .bits(),
            255
        );
    }
}
