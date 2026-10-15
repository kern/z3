//! Safe access to the C port's WRAM-backed OAM staging buffers.

use crate::ram::Ram;
use z3_core::c_types::OamEnt;

pub const OAM_BUFFER: usize = 0x800;
pub const OAM_ENTRY_COUNT: usize = 128;
pub const EXTENDED_OAM: usize = 0xa00;
pub const BYTEWISE_EXTENDED_OAM: usize = 0xa20;

/// A local position in the OAM staging list, measured in entries rather than bytes.
///
/// Moving this value does not change the global allocation pointer in WRAM.
/// A cursor may point one past the list after emitting its last entry; entry
/// reads/writes retain the bounds checks of [`read`] and [`write`]. Advancing
/// does not wrap back to the first entry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OamCursor {
    index: usize,
}

impl OamCursor {
    #[must_use]
    pub const fn from_index(index: usize) -> Self {
        Self { index }
    }

    /// Decodes a legacy byte offset relative to the beginning of OAM.
    ///
    /// Low two bits are discarded, matching the original offset-to-entry
    /// conversion. Offsets are not implicitly narrowed to an 8-bit value.
    #[must_use]
    pub const fn from_byte_offset(offset: usize) -> Self {
        Self::from_index(offset >> 2)
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn offset(self, entries: usize) -> Self {
        Self::from_index(self.index + entries)
    }

    #[must_use]
    pub const fn next(self) -> Self {
        self.offset(1)
    }

    pub fn advance(&mut self) {
        *self = self.next();
    }

    #[must_use]
    pub fn read(self, ram: &Ram<'_>) -> OamEnt {
        read(ram, self.index)
    }

    pub fn write(self, ram: &mut Ram<'_>, entry: OamEnt) {
        write(ram, self.index, entry);
    }

    /// Replaces tile/flags while retaining the entry's existing position.
    pub fn set_tile(self, ram: &mut Ram<'_>, tile: u16) {
        let mut entry = self.read(ram);
        entry.charnum = tile as u8;
        entry.flags = (tile >> 8) as u8;
        self.write(ram, entry);
    }

    /// Replaces position while retaining the entry's existing tile/flags.
    pub fn set_xy(self, ram: &mut Ram<'_>, x: u8, y: u8) {
        let mut entry = self.read(ram);
        entry.x = x;
        entry.y = y;
        self.write(ram, entry);
    }

    /// Hides an entry without clearing its other bytes or extended data.
    pub fn hide(self, ram: &mut Ram<'_>) {
        let mut entry = self.read(ram);
        entry.y = 0xf0;
        self.write(ram, entry);
    }

    /// Stores the unmasked legacy size/X-high byte.
    ///
    /// Some callers deliberately use bits beyond the hardware's two-bit
    /// format. Preserve the staging value until the existing packing step.
    pub fn set_extended(self, ram: &mut Ram<'_>, value: u8) {
        ram.write_u8(BYTEWISE_EXTENDED_OAM + self.index, value);
    }

    /// Stores two adjacent extended bytes in the original low/high order.
    /// This deliberately retains the historical adjacent-byte write, even at
    /// the last entry, rather than truncating it to the OAM list's bounds.
    pub fn set_extended_pair(self, ram: &mut Ram<'_>, value: u16) {
        ram.write_u16(BYTEWISE_EXTENDED_OAM + self.index, value);
    }
}

#[must_use]
pub fn read(ram: &Ram<'_>, index: usize) -> OamEnt {
    assert!(index < OAM_ENTRY_COUNT);
    let address = OAM_BUFFER + index * 4;
    OamEnt {
        x: ram.read_u8(address),
        y: ram.read_u8(address + 1),
        charnum: ram.read_u8(address + 2),
        flags: ram.read_u8(address + 3),
    }
}

#[track_caller]
pub fn write(ram: &mut Ram<'_>, index: usize, entry: OamEnt) {
    assert!(index < OAM_ENTRY_COUNT);
    let address = OAM_BUFFER + index * 4;
    ram.write_u8(address, entry.x);
    ram.write_u8(address + 1, entry.y);
    ram.write_u8(address + 2, entry.charnum);
    ram.write_u8(address + 3, entry.flags);
}

#[track_caller]
pub fn set_helper0(
    ram: &mut Ram<'_>,
    index: usize,
    x: u16,
    y: u16,
    charnum: u8,
    flags: u8,
    big: u8,
) {
    let visible_y = if y.wrapping_add(0x10) < 0x100 {
        y as u8
    } else {
        0xf0
    };
    write(
        ram,
        index,
        OamEnt {
            x: x as u8,
            y: visible_y,
            charnum,
            flags,
        },
    );
    ram.write_u8(BYTEWISE_EXTENDED_OAM + index, big | ((x >> 8) as u8 & 1));
}

pub fn set_helper1(
    ram: &mut Ram<'_>,
    index: usize,
    x: u16,
    y: u8,
    charnum: u8,
    flags: u8,
    big: u8,
) {
    write(
        ram,
        index,
        OamEnt {
            x: x as u8,
            y,
            charnum,
            flags,
        },
    );
    ram.write_u8(BYTEWISE_EXTENDED_OAM + index, big | ((x >> 8) as u8 & 1));
}

#[track_caller]
pub fn set_plain(ram: &mut Ram<'_>, index: usize, x: u8, y: u8, charnum: u8, flags: u8, big: u8) {
    write(
        ram,
        index,
        OamEnt {
            x,
            y,
            charnum,
            flags,
        },
    );
    ram.write_u8(BYTEWISE_EXTENDED_OAM + index, big);
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[test]
    fn local_cursor_keeps_full_offsets_and_does_not_wrap_at_list_end() {
        for offset in [0, 1, 3, 4, 0x1fc, 0x1ff, 0x200] {
            assert_eq!(OamCursor::from_byte_offset(offset).index(), offset / 4);
        }
        let mut cursor = OamCursor::from_index(127);
        cursor.advance();
        assert_eq!(cursor.index(), 128);
        assert_eq!(cursor.offset(2).index(), 130);
    }

    #[test]
    fn cursor_edits_preserve_hidden_bytes_and_the_global_allocation_pointer() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut expected = bytes.to_vec();
        let mut ram = Ram::new(&mut bytes);
        let cursor = OamCursor::from_index(127);
        cursor.set_tile(&mut ram, 0x1234);
        cursor.set_xy(&mut ram, 0xff, 0x40);
        cursor.hide(&mut ram);
        cursor.set_extended(&mut ram, 0xfd);
        let address = OAM_BUFFER + 127 * 4;
        expected[address..address + 4].copy_from_slice(&[0xff, 0xf0, 0x34, 0x12]);
        expected[BYTEWISE_EXTENDED_OAM + 127] = 0xfd;
        assert_eq!(ram.slice(0, WRAM_SIZE), expected);
    }

    #[test]
    fn paired_extended_write_retains_last_entry_adjacent_byte_effect() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut expected = bytes.to_vec();
        let mut ram = Ram::new(&mut bytes);
        OamCursor::from_index(127).set_extended_pair(&mut ram, 0x1234);
        expected[BYTEWISE_EXTENDED_OAM + 127] = 0x34;
        expected[BYTEWISE_EXTENDED_OAM + 128] = 0x12;
        assert_eq!(ram.slice(0, WRAM_SIZE), expected);
    }

    #[test]
    fn helper0_matches_visibility_and_x_high_bit_rules() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        set_helper0(&mut ram, 3, 0x1ab, 0xef, 0x42, 0x31, 2);
        assert_eq!(
            read(&ram, 3),
            OamEnt {
                x: 0xab,
                y: 0xef,
                charnum: 0x42,
                flags: 0x31
            }
        );
        assert_eq!(ram.read_u8(BYTEWISE_EXTENDED_OAM + 3), 3);
        set_helper0(&mut ram, 4, 0, 0xf0, 0, 0, 0);
        assert_eq!(read(&ram, 4).y, 0xf0);
    }
}
