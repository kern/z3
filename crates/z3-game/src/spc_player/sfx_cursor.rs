//! SFX positions refer to the current token or last consumed operand, unlike
//! music patterns' next-unread cursors. Neither type caches or borrows RAM.

/// Saved current-token position of an SFX voice, losslessly serialized as u16.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SfxPosition(u16);

impl SfxPosition {
    #[must_use]
    pub const fn at(address: u16) -> Self {
        Self(address)
    }

    #[must_use]
    pub const fn address(self) -> u16 {
        self.0
    }
}

/// Persistent shared SFX decoder position. Channel save/restore is explicit so
/// stop-token early returns never accidentally publish a different bookmark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SfxCursor {
    current: SfxPosition,
}

impl SfxCursor {
    #[must_use]
    pub const fn at(address: u16) -> Self {
        Self {
            current: SfxPosition::at(address),
        }
    }

    #[must_use]
    pub const fn address(self) -> u16 {
        self.current.address()
    }

    #[must_use]
    pub const fn position(self) -> SfxPosition {
        self.current
    }

    pub fn jump_to(&mut self, position: SfxPosition) {
        self.current = position;
    }

    pub fn advance(&mut self) {
        self.current = SfxPosition::at(self.address().wrapping_add(1));
    }

    #[must_use]
    pub fn current(self, ram: &[u8; 65536]) -> u8 {
        ram[usize::from(self.address())]
    }

    /// Prefix-increment before reading an operand, publishing each increment
    /// immediately. No prefetched argument block crosses intervening DSP writes.
    pub fn advance_read(&mut self, ram: &[u8; 65536]) -> u8 {
        self.advance();
        self.current(ram)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_current_position_preserves_prefix_reads_wrap_and_saved_position() {
        let mut ram = std::array::from_fn(|i| (i ^ (i >> 8)) as u8);
        for address in 0..=u16::MAX {
            let mut cursor = SfxCursor::at(address);
            let saved = cursor.position();
            assert_eq!(saved.address(), address);
            assert_eq!(cursor.current(&ram), ram[usize::from(address)]);
            let next = address.wrapping_add(1);
            ram[usize::from(next)] = 0x51;
            assert_eq!(cursor.advance_read(&ram), 0x51);
            assert_eq!(cursor.address(), next);
            ram[usize::from(next)] = 0xa3;
            assert_eq!(cursor.current(&ram), 0xa3);
            cursor.jump_to(saved);
            assert_eq!(cursor.address(), address);
        }
    }
}
