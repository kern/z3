//! Music pattern addresses, distinct from the SFX current-token convention.
//! Positions own no RAM borrow: reads always observe the supplied live image.

/// A lossless subroutine return or loop-start bookmark, including inactive
/// low-page addresses and all otherwise unusual snapshot representations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PatternAddress(u16);

impl PatternAddress {
    #[must_use]
    pub const fn at(address: u16) -> Self {
        Self(address)
    }

    #[must_use]
    pub const fn address(self) -> u16 {
        self.0
    }
}

/// Persistent position of the next unread music-pattern byte.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PatternCursor {
    next: PatternAddress,
}

impl PatternCursor {
    #[must_use]
    pub const fn at(address: u16) -> Self {
        Self {
            next: PatternAddress::at(address),
        }
    }

    #[must_use]
    pub const fn address(self) -> u16 {
        self.next.address()
    }

    /// The music scheduler uses a nonzero high byte, not a nonzero address.
    #[must_use]
    pub const fn music_active(self) -> bool {
        self.address() >> 8 != 0
    }

    #[must_use]
    pub const fn bookmark(self) -> PatternAddress {
        self.next
    }

    pub fn jump_to(&mut self, address: PatternAddress) {
        self.next = address;
    }

    pub fn advance(&mut self, bytes: u16) {
        self.next = PatternAddress::at(self.address().wrapping_add(bytes));
    }

    #[must_use]
    pub fn peek(self, ram: &[u8; 65536]) -> u8 {
        self.peek_offset(ram, 0)
    }

    #[must_use]
    pub fn peek_offset(self, ram: &[u8; 65536], bytes: u16) -> u8 {
        ram[usize::from(self.address().wrapping_add(bytes))]
    }

    pub fn read_byte(&mut self, ram: &[u8; 65536]) -> u8 {
        let address = self.address();
        self.advance(1);
        ram[usize::from(address)]
    }

    /// KOF scans a separate position and never publishes it to the channel.
    #[must_use]
    pub const fn fork(self) -> PatternProbe {
        PatternProbe { cursor: self }
    }
}

/// Independent lookahead. A call probe follows only the two-byte destination;
/// it must not consume the live call's loop operand or alter its return state.
pub struct PatternProbe {
    cursor: PatternCursor,
}

impl PatternProbe {
    pub fn read_byte(&mut self, ram: &[u8; 65536]) -> u8 {
        self.cursor.read_byte(ram)
    }

    pub fn advance(&mut self, bytes: u16) {
        self.cursor.advance(bytes);
    }

    pub fn jump_to(&mut self, address: PatternAddress) {
        self.cursor.jump_to(address);
    }

    #[must_use]
    pub fn call_destination(&self, ram: &[u8; 65536]) -> PatternAddress {
        PatternAddress::at(u16::from_le_bytes([
            self.cursor.peek(ram),
            self.cursor.peek_offset(ram, 1),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_raw_position_keeps_activity_read_wrap_and_bookmark_semantics() {
        let ram = std::array::from_fn(|i| (i ^ (i >> 8)) as u8);
        for address in 0..=u16::MAX {
            let mut cursor = PatternCursor::at(address);
            assert_eq!(cursor.music_active(), address >= 0x100);
            let bookmark = cursor.bookmark();
            assert_eq!(bookmark.address(), address);
            assert_eq!(cursor.read_byte(&ram), ram[usize::from(address)]);
            assert_eq!(cursor.address(), address.wrapping_add(1));
            cursor.jump_to(bookmark);
            cursor.advance(4);
            assert_eq!(cursor.address(), address.wrapping_add(4));
        }
    }

    #[test]
    fn fork_observes_live_wrapping_operands_without_publishing_or_consuming_loops() {
        let mut ram = [0; 65536];
        ram[0xffff] = 0x34;
        ram[0] = 0x12;
        let live = PatternCursor::at(0xffff);
        let mut probe = live.fork();
        assert_eq!(probe.call_destination(&ram), PatternAddress::at(0x1234));
        ram[0] = 0x56;
        let target = probe.call_destination(&ram);
        assert_eq!(target, PatternAddress::at(0x5634));
        probe.jump_to(target);
        assert_eq!(live.address(), 0xffff);
        ram[0x5634] = 0xc8;
        assert_eq!(probe.read_byte(&ram), 0xc8);
        assert_eq!(live.address(), 0xffff);
    }
}
