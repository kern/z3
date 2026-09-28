//! Exact cartridge address-decoding behavior from `snes/cart.c`.
//!
//! The address calculations intentionally use masks rather than modulo. That
//! preserves the mirroring behavior of the C implementation, including for
//! oddly sized images. Open bus is supplied by the caller because the Rust
//! device does not retain a raw pointer to the owning SNES.

use crate::saveload::{SaveLoad, SaveLoadError};
use core::fmt;

/// No cartridge is mapped.
pub const CART_NONE: u8 = 0;
/// LoROM address decoding is active.
pub const CART_LOROM: u8 = 1;
/// HiROM address decoding is active.
pub const CART_HIROM: u8 = 2;

/// The SRAM size allocated by `cart_init` in the C implementation.
pub const DEFAULT_RAM_SIZE: usize = 0x2000;

/// Cartridge ROM, SRAM, and mapping type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cart {
    cart_type: u8,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl Default for Cart {
    fn default() -> Self {
        Self::new()
    }
}

impl Cart {
    /// Creates the same empty cartridge state as `cart_init`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cart_type: CART_NONE,
            rom: Vec::new(),
            ram: vec![0; DEFAULT_RAM_SIZE],
        }
    }

    pub fn saveload(&mut self, io: &mut impl SaveLoad) -> Result<(), SaveLoadError> {
        io.transfer(&mut self.ram)
    }

    /// Loads a copied ROM image and clears SRAM.
    ///
    /// The C port asserts that `ram_size` equals the already allocated SRAM
    /// size. Returning an error keeps that invariant without process aborts.
    pub fn load(
        &mut self,
        cart_type: u8,
        rom: &[u8],
        ram_size: usize,
    ) -> Result<(), CartLoadError> {
        if ram_size != self.ram.len() {
            return Err(CartLoadError::RamSizeMismatch {
                allocated: self.ram.len(),
                requested: ram_size,
            });
        }

        self.cart_type = cart_type;
        self.rom.clear();
        self.rom.extend_from_slice(rom);
        self.ram.fill(0);
        Ok(())
    }

    /// Clears all SRAM bytes. ROM and mapping type are unchanged.
    pub fn reset(&mut self) {
        self.ram.fill(0);
    }

    /// Returns the raw mapping type used by the C switch statements.
    #[must_use]
    pub fn cart_type(&self) -> u8 {
        self.cart_type
    }

    /// Returns the copied ROM image.
    #[must_use]
    pub fn rom(&self) -> &[u8] {
        &self.rom
    }

    /// Returns cartridge SRAM for save-state serialization.
    #[must_use]
    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    /// Returns mutable cartridge SRAM for save-state restoration.
    pub fn ram_mut(&mut self) -> &mut [u8] {
        &mut self.ram
    }

    /// Reads one byte using the selected mapping, returning `open_bus` for an
    /// unmapped address or unknown cartridge type.
    #[must_use]
    pub fn read(&self, bank: u8, address: u16, open_bus: u8) -> u8 {
        // This early return is present in the C implementation. Its `case 1`
        // in the following switch is consequently unreachable.
        if self.cart_type == CART_LOROM {
            return self.read_lorom(bank, address, open_bus);
        }

        match self.cart_type {
            CART_HIROM => self.read_hirom(bank, address, open_bus),
            _ => open_bus,
        }
    }

    /// Writes one byte using the selected mapping. ROM and unmapped addresses
    /// ignore writes.
    pub fn write(&mut self, bank: u8, address: u16, value: u8) {
        match self.cart_type {
            CART_LOROM => self.write_lorom(bank, address, value),
            CART_HIROM => self.write_hirom(bank, address, value),
            _ => {}
        }
    }

    fn read_lorom(&self, bank: u8, address: u16, open_bus: u8) -> u8 {
        if address >= 0x8000 {
            return self.read_lorom_rom(bank, address, open_bus);
        }
        if ((0x70..0x7e).contains(&bank) || bank >= 0xf0) && !self.ram.is_empty() {
            let offset = (((bank as usize & 0xf) << 15) | address as usize) & (self.ram.len() - 1);
            return self.ram[offset];
        }
        if bank & 0x40 != 0 {
            return self.read_lorom_rom(bank, address, open_bus);
        }
        open_bus
    }

    fn read_lorom_rom(&self, bank: u8, address: u16, open_bus: u8) -> u8 {
        if self.rom.is_empty() {
            return open_bus;
        }
        let offset = ((bank as usize) << 15 | (address as usize & 0x7fff)) & (self.rom.len() - 1);
        self.rom[offset]
    }

    fn write_lorom(&mut self, bank: u8, address: u16, value: u8) {
        // Preserve the original `bank > 0xf0` condition. Bank f0 SRAM is
        // readable but, unusually, not writable through this function.
        if ((0x70..0x7e).contains(&bank) || bank > 0xf0) && address < 0x8000 && !self.ram.is_empty()
        {
            let offset = (((bank as usize & 0xf) << 15) | address as usize) & (self.ram.len() - 1);
            self.ram[offset] = value;
        }
    }

    fn read_hirom(&self, bank: u8, address: u16, open_bus: u8) -> u8 {
        let bank = bank & 0x7f;
        if bank < 0x40 && (0x6000..0x8000).contains(&address) && !self.ram.is_empty() {
            let offset = (((bank as usize & 0x3f) << 13) | (address as usize & 0x1fff))
                & (self.ram.len() - 1);
            return self.ram[offset];
        }
        if address >= 0x8000 || bank >= 0x40 {
            if self.rom.is_empty() {
                return open_bus;
            }
            let offset = (((bank as usize & 0x3f) << 16) | address as usize) & (self.rom.len() - 1);
            return self.rom[offset];
        }
        open_bus
    }

    fn write_hirom(&mut self, bank: u8, address: u16, value: u8) {
        let bank = bank & 0x7f;
        if bank < 0x40 && (0x6000..0x8000).contains(&address) && !self.ram.is_empty() {
            let offset = (((bank as usize & 0x3f) << 13) | (address as usize & 0x1fff))
                & (self.ram.len() - 1);
            self.ram[offset] = value;
        }
    }
}

/// Failure to satisfy an invariant enforced by the C loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartLoadError {
    /// `cart_load` requires the requested SRAM size to equal its allocation.
    RamSizeMismatch { allocated: usize, requested: usize },
}

impl fmt::Display for CartLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RamSizeMismatch {
                allocated,
                requested,
            } => write!(
                f,
                "cartridge RAM size mismatch: allocated {allocated}, requested {requested}"
            ),
        }
    }
}

impl std::error::Error for CartLoadError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterned_rom(size: usize) -> Vec<u8> {
        (0..size).map(|index| (index >> 8) as u8).collect()
    }

    fn loaded(kind: u8, rom_size: usize) -> Cart {
        let mut cart = Cart::new();
        cart.load(kind, &patterned_rom(rom_size), DEFAULT_RAM_SIZE)
            .unwrap();
        cart
    }

    #[test]
    fn initial_and_unknown_types_are_open_bus() {
        let mut cart = Cart::new();
        assert_eq!(cart.read(0xff, 0xffff, 0xa5), 0xa5);
        cart.write(0x70, 0x1234, 9);
        assert_eq!(cart.ram()[0x1234], 0);

        cart.load(99, &[1, 2, 3, 4], DEFAULT_RAM_SIZE).unwrap();
        assert_eq!(cart.read(0, 0x8000, 0x5a), 0x5a);
    }

    #[test]
    fn load_and_reset_clear_ram_and_copy_rom() {
        let mut cart = Cart::new();
        cart.ram_mut()[7] = 3;
        cart.load(CART_LOROM, &[1, 2, 3], DEFAULT_RAM_SIZE).unwrap();
        assert_eq!(cart.rom(), &[1, 2, 3]);
        assert!(cart.ram().iter().all(|&byte| byte == 0));

        cart.ram_mut()[7] = 4;
        cart.reset();
        assert_eq!(cart.ram()[7], 0);
        assert_eq!(cart.rom(), &[1, 2, 3]);

        assert_eq!(
            cart.load(CART_HIROM, &[], 0),
            Err(CartLoadError::RamSizeMismatch {
                allocated: DEFAULT_RAM_SIZE,
                requested: 0,
            })
        );
    }

    #[test]
    fn lorom_maps_upper_halves_and_full_high_banks() {
        let cart = loaded(CART_LOROM, 0x20_000);
        assert_eq!(cart.read(0x00, 0x8123, 0xee), 0x01);
        assert_eq!(cart.read(0x01, 0x8123, 0xee), 0x81);
        assert_eq!(cart.read(0x40, 0x0123, 0xee), 0x01);
        assert_eq!(cart.read(0xc1, 0x0123, 0xee), 0x81);
        assert_eq!(cart.read(0x00, 0x1234, 0xee), 0xee);

        // The C mask mirrors bank 4 onto bank 0 for this 128 KiB image.
        assert_eq!(cart.read(0x04, 0x8123, 0xee), 0x01);
    }

    #[test]
    fn lorom_sram_banks_and_f0_write_quirk_match_c() {
        let mut cart = loaded(CART_LOROM, 0x1_0000);
        cart.write(0x70, 0x0123, 0x11);
        assert_eq!(cart.read(0x70, 0x0123, 0xee), 0x11);
        assert_eq!(cart.read(0xf0, 0x0123, 0xee), 0x11);

        cart.write(0xf0, 0x0123, 0x22);
        assert_eq!(cart.read(0xf0, 0x0123, 0xee), 0x11);
        cart.write(0xf1, 0x0123, 0x33);
        assert_eq!(cart.read(0x71, 0x0123, 0xee), 0x33);

        cart.write(0x7e, 0x0123, 0x44);
        assert_eq!(cart.read(0x70, 0x0123, 0xee), 0x33);
        cart.write(0x70, 0x8000, 0x55);
        assert_eq!(cart.read(0x70, 0x0123, 0xee), 0x33);
    }

    #[test]
    fn hirom_maps_rom_ram_and_bank_mirrors() {
        let mut cart = loaded(CART_HIROM, 0x4_0000);

        cart.write(0x00, 0x6123, 0x41);
        assert_eq!(cart.read(0x00, 0x6123, 0xee), 0x41);
        // Bit 7 is discarded before all HiROM decoding.
        assert_eq!(cart.read(0x80, 0x6123, 0xee), 0x41);
        // With 8 KiB SRAM, all banks in the SRAM window mirror.
        assert_eq!(cart.read(0x01, 0x6123, 0xee), 0x41);

        assert_eq!(cart.read(0x00, 0x1234, 0xee), 0xee);
        assert_eq!(cart.read(0x00, 0x8123, 0xee), 0x81);
        assert_eq!(cart.read(0x40, 0x0123, 0xee), 0x01);
        assert_eq!(cart.read(0xc1, 0x0123, 0xee), 0x01);

        let before = cart.rom().to_vec();
        cart.write(0x40, 0x1234, 0xff);
        assert_eq!(cart.rom(), before);
    }

    #[test]
    fn empty_rom_reads_safely_as_open_bus() {
        let mut cart = Cart::new();
        cart.load(CART_LOROM, &[], DEFAULT_RAM_SIZE).unwrap();
        assert_eq!(cart.read(0, 0x8000, 0x9a), 0x9a);
        cart.load(CART_HIROM, &[], DEFAULT_RAM_SIZE).unwrap();
        assert_eq!(cart.read(0x40, 0, 0x9b), 0x9b);
    }
}
