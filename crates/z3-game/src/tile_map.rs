//! Typed access to live tile maps and their collision attributes.
//!
//! Word indices and byte offsets are distinct because legacy map16 callers
//! sometimes deliberately read unaligned words. The dungeon BG2 and overworld
//! map16 layouts alias the same bytes; neither view owns or caches a map.

use crate::ram::Ram;

/// An index measured in tile words, not a WRAM address or byte count.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TileWordIndex(usize);

impl TileWordIndex {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }
}

/// A byte offset relative to a tile map. Odd offsets remain odd unless the
/// caller explicitly requests alignment; arithmetic is not implicitly narrowed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TileByteOffset(usize);

impl TileByteOffset {
    #[must_use]
    pub const fn new(offset: usize) -> Self {
        Self(offset)
    }

    #[must_use]
    pub const fn aligned(self) -> Self {
        Self(self.0 & !1)
    }

    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }
}

/// A live layout view, retaining historical accesses beyond a logical row or
/// layer boundary when they still address the canonical WRAM image.
#[derive(Clone, Copy, Debug)]
pub struct TileMap {
    base: usize,
}

pub const DUNGEON_BG2: TileMap = TileMap { base: 0x2000 };
pub const DUNGEON_BG1: TileMap = TileMap { base: 0x4000 };
pub const OVERWORLD_MAP16: TileMap = DUNGEON_BG2;

impl TileMap {
    #[must_use]
    pub fn read(self, ram: &Ram<'_>, index: TileWordIndex) -> u16 {
        self.read_at_byte(ram, TileByteOffset::new(index.0 * 2))
    }

    pub fn write(self, ram: &mut Ram<'_>, index: TileWordIndex, tile: u16) {
        self.write_at_byte(ram, TileByteOffset::new(index.0 * 2), tile);
    }

    #[must_use]
    pub fn read_at_byte(self, ram: &Ram<'_>, offset: TileByteOffset) -> u16 {
        ram.read_u16(self.base + offset.0)
    }

    pub fn write_at_byte(self, ram: &mut Ram<'_>, offset: TileByteOffset, tile: u16) {
        ram.write_u16(self.base + offset.0, tile);
    }
}

/// Collision bytes use the same cell index as the corresponding tile word.
/// The lower dungeon floor is an explicit index offset, not another allocation.
#[derive(Clone, Copy, Debug)]
pub struct DungeonAttributes;

impl DungeonAttributes {
    #[must_use]
    pub fn read(ram: &Ram<'_>, index: TileWordIndex) -> u8 {
        ram.read_u8(0x12000 + index.0)
    }

    pub fn write(ram: &mut Ram<'_>, index: TileWordIndex, attribute: u8) {
        ram.write_u8(0x12000 + index.0, attribute);
    }

    /// Reads two adjacent attributes without aligning their cell index.
    #[must_use]
    pub fn read_pair(ram: &Ram<'_>, index: TileWordIndex) -> u16 {
        ram.read_u16(0x12000 + index.0)
    }

    /// Preserves legacy paired writes and their overlap with neighboring cells.
    pub fn write_pair(ram: &mut Ram<'_>, index: TileWordIndex, attributes: u16) {
        ram.write_u16(0x12000 + index.0, attributes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[test]
    fn shared_map_views_and_raw_patches_are_live() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        DUNGEON_BG2.write(&mut ram, TileWordIndex::new(63), 0x1234);
        assert_eq!(
            OVERWORLD_MAP16.read_at_byte(&ram, TileByteOffset::new(126)),
            0x1234
        );
        ram.write_u8(0x2000 + 127, 0xab);
        assert_eq!(DUNGEON_BG2.read(&ram, TileWordIndex::new(63)), 0xab34);
        assert_eq!(DUNGEON_BG1.read(&ram, TileWordIndex::new(63)), 0xa5a5);
    }

    #[test]
    fn byte_offsets_preserve_unaligned_reads_and_explicit_alignment() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.slice_mut(0x2000, 4)
            .copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);
        assert_eq!(
            OVERWORLD_MAP16.read_at_byte(&ram, TileByteOffset::new(1)),
            0x3322
        );
        assert_eq!(
            OVERWORLD_MAP16.read_at_byte(&ram, TileByteOffset::new(1).aligned()),
            0x2211
        );
    }

    #[test]
    fn edge_and_lower_floor_writes_preserve_the_entire_remaining_image() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut expected = bytes.to_vec();
        let mut ram = Ram::new(&mut bytes);
        DUNGEON_BG2.write(&mut ram, TileWordIndex::new(0x1000), 0x1234);
        DungeonAttributes::write(&mut ram, TileWordIndex::new(0x1000), 0x56);
        DungeonAttributes::write_pair(&mut ram, TileWordIndex::new(0x1001), 0xbcde);
        OVERWORLD_MAP16.write_at_byte(&mut ram, TileByteOffset::new(1), 0x789a);
        expected[0x4000..0x4002].copy_from_slice(&[0x34, 0x12]);
        expected[0x13000] = 0x56;
        expected[0x13001..0x13003].copy_from_slice(&[0xde, 0xbc]);
        expected[0x2001..0x2003].copy_from_slice(&[0x9a, 0x78]);
        assert_eq!(DUNGEON_BG1.read(&ram, TileWordIndex::new(0)), 0x1234);
        assert_eq!(
            DungeonAttributes::read_pair(&ram, TileWordIndex::new(0x1001)),
            0xbcde
        );
        assert_eq!(ram.slice(0, WRAM_SIZE), expected);
    }
}
