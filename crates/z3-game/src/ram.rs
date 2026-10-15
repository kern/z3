//! Typed, explicitly little-endian access to the C port's `g_ram` layout.
//!
//! Low-level compatibility boundary for the canonical image. Gameplay should
//! prefer named, live views such as [`Ram::player`] over carrying layout offsets.
//! Remaining translated accesses are traceable to `variables.h` without raw
//! pointers, alignment assumptions, or native-endian aliasing.

use z3_core::WRAM_SIZE;

use crate::player_state::{PlayerMut, PlayerRef};

pub struct Ram<'a> {
    bytes: &'a mut [u8; WRAM_SIZE],
}

impl<'a> Ram<'a> {
    #[must_use]
    pub fn new(bytes: &'a mut [u8; WRAM_SIZE]) -> Self {
        Self { bytes }
    }

    /// Borrows the named player fields directly from the canonical WRAM.
    #[must_use]
    pub fn player(&self) -> PlayerRef<'_> {
        PlayerRef::new(self.bytes)
    }

    /// Exclusively borrows the named player fields without copying state.
    #[must_use]
    pub fn player_mut(&mut self) -> PlayerMut<'_> {
        PlayerMut::new(self.bytes)
    }

    #[must_use]
    pub fn read_u8(&self, address: usize) -> u8 {
        self.bytes[address]
    }

    pub fn write_u8(&mut self, address: usize, value: u8) {
        self.bytes[address] = value;
    }

    #[must_use]
    pub fn read_i8(&self, address: usize) -> i8 {
        self.read_u8(address) as i8
    }

    pub fn write_i8(&mut self, address: usize, value: i8) {
        self.write_u8(address, value as u8);
    }

    #[must_use]
    pub fn read_u16(&self, address: usize) -> u16 {
        u16::from_le_bytes([self.bytes[address], self.bytes[address + 1]])
    }

    pub fn write_u16(&mut self, address: usize, value: u16) {
        self.bytes[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }

    #[must_use]
    pub fn read_i16(&self, address: usize) -> i16 {
        self.read_u16(address) as i16
    }

    pub fn write_i16(&mut self, address: usize, value: i16) {
        self.write_u16(address, value as u16);
    }

    #[must_use]
    pub fn read_u32(&self, address: usize) -> u32 {
        u32::from_le_bytes([
            self.bytes[address],
            self.bytes[address + 1],
            self.bytes[address + 2],
            self.bytes[address + 3],
        ])
    }

    pub fn write_u32(&mut self, address: usize, value: u32) {
        self.bytes[address..address + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[must_use]
    pub fn slice(&self, address: usize, length: usize) -> &[u8] {
        &self.bytes[address..address + length]
    }

    pub fn slice_mut(&mut self, address: usize, length: usize) -> &mut [u8] {
        &mut self.bytes[address..address + length]
    }

    /// Reborrows the complete legacy WRAM image for translated units whose C
    /// entrypoints own their `Ram` wrapper internally.
    pub fn bytes_mut(&mut self) -> &mut [u8; WRAM_SIZE] {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_access_is_little_endian_and_preserves_signed_bits() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(0x20, 0xabcd);
        assert_eq!(ram.slice(0x20, 2), &[0xcd, 0xab]);
        ram.write_i16(0x30, -2);
        assert_eq!(ram.read_u16(0x30), 0xfffe);
        ram.write_i8(0x40, -128);
        assert_eq!(ram.read_u8(0x40), 0x80);
        ram.write_u32(0x50, 0x7856_3412);
        assert_eq!(ram.slice(0x50, 4), &[0x12, 0x34, 0x56, 0x78]);
        assert_eq!(ram.read_u32(0x50), 0x7856_3412);
    }
}
