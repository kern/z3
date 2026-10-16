//! Owned position in the music phrase-control stream.
//!
//! The stream is read from live SPC RAM, not cached or borrowed by the cursor.
//! Every 16-bit position survives snapshot encoding, including low-memory
//! addresses and wrapped word reads at the end of RAM.

/// One phrase-control word. All 65,536 word values have defined meanings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhraseEntry {
    Stop,
    FastForwardOn,
    FastForwardOff,
    Repeat(u8),
    PatternTable(u16),
}

/// The persistent position of the next unread phrase-control word.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhraseCursor {
    next: u16,
}

impl PhraseCursor {
    /// Starts a phrase stream at an address supplied by a song table or save.
    #[must_use]
    pub const fn at(address: u16) -> Self {
        Self { next: address }
    }

    /// The lossless address used by the legacy snapshot codec and diagnostics.
    #[must_use]
    pub const fn address(self) -> u16 {
        self.next
    }

    pub fn jump_to(&mut self, destination: u16) {
        self.next = destination;
    }

    /// Reads only the control word, then advances the persistent position.
    ///
    /// A repeat destination remains unread until the player updates its repeat
    /// counter and explicitly calls [`Self::read_repeat_destination`].
    pub fn next_entry(&mut self, ram: &[u8; 65536]) -> PhraseEntry {
        let word = self.take_word(ram);
        if word >> 8 != 0 {
            return PhraseEntry::PatternTable(word);
        }
        match word {
            0 => PhraseEntry::Stop,
            0x80 => PhraseEntry::FastForwardOn,
            0x81 => PhraseEntry::FastForwardOff,
            _ => PhraseEntry::Repeat(word as u8),
        }
    }

    /// Reads a repeat destination after the repeat-counter update.
    pub fn read_repeat_destination(&mut self, ram: &[u8; 65536]) -> u16 {
        self.take_word(ram)
    }

    fn take_word(&mut self, ram: &[u8; 65536]) -> u16 {
        let address = self.next;
        let word = u16::from_le_bytes([
            ram[usize::from(address)],
            ram[usize::from(address.wrapping_add(1))],
        ]);
        self.next = address.wrapping_add(2);
        word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_control_words_keep_the_original_high_byte_classification() {
        let mut ram = [0; 65536];
        let mut categories = [0; 5];
        for word in 0..=u16::MAX {
            ram[0x100..0x102].copy_from_slice(&word.to_le_bytes());
            let mut cursor = PhraseCursor::at(0x100);
            let actual = cursor.next_entry(&ram);
            let expected = if word >= 0x100 {
                categories[4] += 1;
                PhraseEntry::PatternTable(word)
            } else {
                match word {
                    0 => {
                        categories[0] += 1;
                        PhraseEntry::Stop
                    }
                    0x80 => {
                        categories[1] += 1;
                        PhraseEntry::FastForwardOn
                    }
                    0x81 => {
                        categories[2] += 1;
                        PhraseEntry::FastForwardOff
                    }
                    _ => {
                        categories[3] += 1;
                        PhraseEntry::Repeat(word as u8)
                    }
                }
            };
            assert_eq!(actual, expected, "control word {word:#06x}");
            assert_eq!(cursor.address(), 0x102);
        }
        assert_eq!(categories, [1, 1, 1, 253, 65_280]);
    }

    #[test]
    fn every_position_reads_a_wrapping_word_and_advances_by_two() {
        let ram = std::array::from_fn(|index| (index ^ (index >> 8)) as u8);
        for address in 0..=u16::MAX {
            let mut cursor = PhraseCursor::at(address);
            let expected = u16::from_le_bytes([
                ram[usize::from(address)],
                ram[usize::from(address.wrapping_add(1))],
            ]);
            assert_eq!(cursor.read_repeat_destination(&ram), expected);
            assert_eq!(cursor.address(), address.wrapping_add(2));
        }
        let mut cursor = PhraseCursor::at(0xfffe);
        cursor.read_repeat_destination(&ram);
        assert_eq!(cursor.address(), 0);
        cursor.jump_to(0xffff);
        cursor.read_repeat_destination(&ram);
        assert_eq!(cursor.address(), 1);
    }

    #[test]
    fn repeat_destination_is_not_read_or_advanced_ahead_of_its_phase() {
        let mut ram = [0; 65536];
        ram[0xfffe..].copy_from_slice(&[0xff, 0]);
        ram[..2].copy_from_slice(&[0x34, 0x12]);
        let mut cursor = PhraseCursor::at(0xfffe);
        assert_eq!(cursor.next_entry(&ram), PhraseEntry::Repeat(0xff));
        assert_eq!(cursor.address(), 0);
        // Destination reads must observe live RAM after the control-word phase.
        ram[..2].copy_from_slice(&[0x78, 0x56]);
        let destination = cursor.read_repeat_destination(&ram);
        assert_eq!(destination, 0x5678);
        assert_eq!(cursor.address(), 2);
        cursor.jump_to(destination);
        assert_eq!(cursor.address(), 0x5678);
    }
}
