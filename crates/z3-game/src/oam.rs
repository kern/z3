//! Safe access to the C port's WRAM-backed OAM staging buffers.

use crate::ram::Ram;
use z3_core::c_types::OamEnt;

pub const OAM_BUFFER: usize = 0x800;
pub const OAM_ENTRY_COUNT: usize = 128;
pub const EXTENDED_OAM: usize = 0xa00;
pub const BYTEWISE_EXTENDED_OAM: usize = 0xa20;

const CURRENT_ENTRY: usize = 0x90;
const CURRENT_EXTENDED: usize = 0x92;
const REGION_BASES: usize = 0xfe0;
const FALLBACK_COUNTERS: usize = 0xfec;

pub const REGION_LIMITS: [u16; 6] = [0x171, 0x201, 0x31, 0xc1, 0x141, 0x1d1];
pub const REGION_FALLBACKS: [u16; 48] = [
    0x30, 0x50, 0x80, 0xb0, 0xe0, 0x110, 0x140, 0x170, 0x1d0, 0x1d4, 0x1dc, 0x1e0, 0x1e4, 0x1ec,
    0x1f0, 0x1f8, 0, 4, 8, 0xc, 0x10, 0x14, 0x18, 0x1c, 0x30, 0x38, 0x50, 0x68, 0x80, 0x98, 0xb0,
    0xc8, 0x120, 0x124, 0x128, 0x12c, 0x130, 0x134, 0x138, 0x13c, 0x140, 0x150, 0x160, 0x170,
    0x180, 0x190, 0x1a0, 0x1b8,
];
pub const REGION_INITIAL_BASES: [u16; 6] = [0x30, 0x1d0, 0, 0x30, 0x120, 0x140];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OamRegion {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
}

impl OamRegion {
    /// Legacy selectors use twice the region index; odd selectors intentionally
    /// name the same region as the preceding even selector.
    #[must_use]
    pub fn from_legacy_selector(selector: u8) -> Self {
        match selector >> 1 {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            _ => panic!("invalid OAM region selector {selector}"),
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// Independent raw addresses retained by the C drawing interface. In
/// particular, the extended pointer is not rederived when loading a snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OamPointers {
    pub entry: u16,
    pub extended: u16,
}

impl OamPointers {
    #[must_use]
    pub fn load(ram: &Ram<'_>) -> Self {
        Self {
            entry: ram.read_u16(CURRENT_ENTRY),
            extended: ram.read_u16(CURRENT_EXTENDED),
        }
    }

    pub fn publish_entry_first(self, ram: &mut Ram<'_>) {
        ram.write_u16(CURRENT_ENTRY, self.entry);
        ram.write_u16(CURRENT_EXTENDED, self.extended);
    }

    pub fn publish_extended_first(self, ram: &mut Ram<'_>) {
        ram.write_u16(CURRENT_EXTENDED, self.extended);
        ram.write_u16(CURRENT_ENTRY, self.entry);
    }

    /// Reads only the primary pointer and retains the existing draw-time
    /// validation. Allocating does not itself require a valid entry cursor.
    #[must_use]
    pub fn current_cursor(ram: &Ram<'_>) -> OamCursor {
        let address = usize::from(ram.read_u16(CURRENT_ENTRY));
        assert!((OAM_BUFFER..EXTENDED_OAM).contains(&address));
        assert_eq!((address - OAM_BUFFER) & 3, 0);
        OamCursor::from_byte_offset(address - OAM_BUFFER)
    }

    /// Advances both live pointers in primary-then-extended order, retaining
    /// their existing relationship (including deliberately mismatched values).
    pub fn advance_entries(ram: &mut Ram<'_>, entries: u16) {
        ram.write_u16(
            CURRENT_ENTRY,
            ram.read_u16(CURRENT_ENTRY)
                .wrapping_add(entries.wrapping_mul(4)),
        );
        ram.write_u16(
            CURRENT_EXTENDED,
            ram.read_u16(CURRENT_EXTENDED).wrapping_add(entries),
        );
    }
}

/// Result of a byte-count allocation. Its addresses may be unaligned; the
/// existing renderer checks entry alignment when it actually draws.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OamAllocation(OamPointers);

impl OamAllocation {
    #[must_use]
    pub const fn legacy_low_byte(self) -> u8 {
        self.0.entry as u8
    }

    #[must_use]
    pub const fn pointers(self) -> OamPointers {
        self.0
    }
}

/// Live region allocation state. Resetting region bases never resets fallback
/// rotation counters, global pointers, or inactive OAM staging bytes.
pub struct OamAllocator<'a, 'r> {
    ram: &'a mut Ram<'r>,
}

impl<'a, 'r> OamAllocator<'a, 'r> {
    pub fn new(ram: &'a mut Ram<'r>) -> Self {
        Self { ram }
    }

    pub fn reset_region_bases(&mut self) {
        for (index, base) in REGION_INITIAL_BASES.into_iter().enumerate() {
            self.ram.write_u16(REGION_BASES + index * 2, base);
        }
    }

    pub fn allocate_bytes(&mut self, region: OamRegion, bytes: u8) -> OamAllocation {
        let index = region.index();
        let base_address = REGION_BASES + index * 2;
        let mut start = self.ram.read_u16(base_address);
        let end = start.wrapping_add(u16::from(bytes));
        if end >= REGION_LIMITS[index] {
            let counter_address = FALLBACK_COUNTERS + index * 2;
            let counter = self.ram.read_u16(counter_address);
            self.ram.write_u16(counter_address, counter.wrapping_add(1));
            start = REGION_FALLBACKS[index * 8 + usize::from(counter & 7)];
        } else {
            self.ram.write_u16(base_address, end);
        }
        let pointers = OamPointers {
            entry: (OAM_BUFFER as u16).wrapping_add(start),
            extended: (BYTEWISE_EXTENDED_OAM as u16).wrapping_add(start >> 2),
        };
        pointers.publish_extended_first(self.ram);
        OamAllocation(pointers)
    }
}

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
    fn region_reset_preserves_counters_global_pointers_and_staging_bytes() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut expected = bytes.to_vec();
        let mut ram = Ram::new(&mut bytes);
        OamAllocator::new(&mut ram).reset_region_bases();
        for (index, value) in REGION_INITIAL_BASES.into_iter().enumerate() {
            expected[0xfe0 + index * 2..0xfe2 + index * 2].copy_from_slice(&value.to_le_bytes());
        }
        assert_eq!(ram.slice(0, WRAM_SIZE), expected);
    }

    #[test]
    fn global_pointer_publication_and_advance_retain_independent_raw_addresses() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut expected = bytes.to_vec();
        let mut ram = Ram::new(&mut bytes);
        OamPointers {
            entry: 0xfffc,
            extended: 0x1234,
        }
        .publish_entry_first(&mut ram);
        OamPointers::advance_entries(&mut ram, 1);
        assert_eq!(
            OamPointers::load(&ram),
            OamPointers {
                entry: 0,
                extended: 0x1235
            }
        );
        OamPointers {
            entry: 0x900,
            extended: 0xffff,
        }
        .publish_extended_first(&mut ram);
        OamPointers::advance_entries(&mut ram, 2);
        assert_eq!(OamPointers::current_cursor(&ram).index(), 66);
        expected[0x90..0x94].copy_from_slice(&[8, 9, 1, 0]);
        assert_eq!(ram.slice(0, WRAM_SIZE), expected);
    }

    #[test]
    fn fallback_rotates_from_old_counter_then_wraps_without_changing_region_base() {
        for (index, region) in [
            OamRegion::A,
            OamRegion::B,
            OamRegion::C,
            OamRegion::D,
            OamRegion::E,
            OamRegion::F,
        ]
        .into_iter()
        .enumerate()
        {
            let mut bytes = [0xa5; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            ram.write_u16(0xfe0 + index * 2, REGION_LIMITS[index]);
            ram.write_u16(0xfec + index * 2, 0xffff);
            let mut expected = ram.slice(0, WRAM_SIZE).to_vec();
            let start = REGION_FALLBACKS[index * 8 + 7];
            let result = OamAllocator::new(&mut ram).allocate_bytes(region, 0);
            let pointers = OamPointers {
                entry: 0x800 + start,
                extended: 0xa20 + (start >> 2),
            };
            assert_eq!(result.pointers(), pointers);
            assert_eq!(result.legacy_low_byte(), pointers.entry as u8);
            expected[0x90..0x92].copy_from_slice(&pointers.entry.to_le_bytes());
            expected[0x92..0x94].copy_from_slice(&pointers.extended.to_le_bytes());
            expected[0xfec + index * 2..0xfee + index * 2].fill(0);
            assert_eq!(ram.slice(0, WRAM_SIZE), expected);
            assert_eq!(
                OamRegion::from_legacy_selector((index * 2 + 1) as u8),
                region
            );
        }
    }

    #[test]
    fn allocation_keeps_unaligned_starts_and_wraps_u16_base_before_threshold() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(0xfe0, 0x0e);
        let result = OamAllocator::new(&mut ram).allocate_bytes(OamRegion::A, 3);
        assert_eq!(
            result.pointers(),
            OamPointers {
                entry: 0x80e,
                extended: 0xa23
            }
        );
        assert_eq!(ram.read_u16(0xfe0), 0x11);
        assert_eq!(ram.read_u16(0xfec), 0xa5a5);
        ram.write_u16(0xfe0, 0xffff);
        let result = OamAllocator::new(&mut ram).allocate_bytes(OamRegion::A, 1);
        assert_eq!(
            result.pointers(),
            OamPointers {
                entry: 0x7ff,
                extended: 0x4a1f
            }
        );
        assert_eq!(ram.read_u16(0xfe0), 0);
        assert_eq!(ram.read_u16(0xfec), 0xa5a5);
    }

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
