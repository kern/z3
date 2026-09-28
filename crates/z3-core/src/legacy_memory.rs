//! Canonical memory regions used by the C implementation and its verifier.

/// SNES work RAM size compared by `VerifySnapshotsEq` in the C reference.
pub const WRAM_SIZE: usize = 0x20_000;
/// Cartridge save RAM span used by the C reference snapshot verifier.
pub const SRAM_SIZE: usize = 0x2_000;
/// Number of 16-bit VRAM words used by the C reference snapshot verifier.
pub const VRAM_WORDS: usize = 0x8_000;

/// Mutable legacy memory retained during the exact-port phase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyMemory {
    wram: Box<[u8; WRAM_SIZE]>,
    sram: Box<[u8; SRAM_SIZE]>,
    vram: Box<[u16; VRAM_WORDS]>,
}

impl Default for LegacyMemory {
    fn default() -> Self {
        Self {
            wram: Box::new([0; WRAM_SIZE]),
            sram: Box::new([0; SRAM_SIZE]),
            vram: Box::new([0; VRAM_WORDS]),
        }
    }
}

impl LegacyMemory {
    #[must_use]
    pub fn wram(&self) -> &[u8; WRAM_SIZE] {
        &self.wram
    }

    pub fn wram_mut(&mut self) -> &mut [u8; WRAM_SIZE] {
        &mut self.wram
    }

    #[must_use]
    pub fn sram(&self) -> &[u8; SRAM_SIZE] {
        &self.sram
    }

    pub fn sram_mut(&mut self) -> &mut [u8; SRAM_SIZE] {
        &mut self.sram
    }

    #[must_use]
    pub fn vram(&self) -> &[u16; VRAM_WORDS] {
        &self.vram
    }

    pub fn vram_mut(&mut self) -> &mut [u16; VRAM_WORDS] {
        &mut self.vram
    }

    #[must_use]
    pub fn read_u16_le(&self, address: usize) -> u16 {
        u16::from_le_bytes([self.wram[address], self.wram[address + 1]])
    }

    pub fn write_u16_le(&mut self, address: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.wram[address] = bytes[0];
        self.wram[address + 1] = bytes[1];
    }

    #[must_use]
    pub fn canonical_state(&self) -> CanonicalState {
        CanonicalState {
            wram: self.wram.clone(),
            sram: self.sram.clone(),
            vram: self.vram.clone(),
        }
    }
}

/// Pointer-free state suitable for deterministic C/Rust comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalState {
    pub wram: Box<[u8; WRAM_SIZE]>,
    pub sram: Box<[u8; SRAM_SIZE]>,
    pub vram: Box<[u16; VRAM_WORDS]>,
}

impl CanonicalState {
    pub const BYTE_LEN: usize = WRAM_SIZE + SRAM_SIZE + VRAM_WORDS * 2;

    /// Serializes integers explicitly as little-endian bytes. Rust struct
    /// padding, pointers, and allocator layout never enter the comparison.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::BYTE_LEN);
        bytes.extend_from_slice(self.wram.as_slice());
        bytes.extend_from_slice(self.sram.as_slice());
        for word in self.vram.iter() {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// Decodes the exact byte layout emitted by the C oracle.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::BYTE_LEN {
            return None;
        }

        let (wram_bytes, rest) = bytes.split_at(WRAM_SIZE);
        let (sram_bytes, vram_bytes) = rest.split_at(SRAM_SIZE);
        let mut wram = Box::new([0; WRAM_SIZE]);
        let mut sram = Box::new([0; SRAM_SIZE]);
        let mut vram = Box::new([0; VRAM_WORDS]);
        wram.copy_from_slice(wram_bytes);
        sram.copy_from_slice(sram_bytes);
        for (word, pair) in vram.iter_mut().zip(vram_bytes.chunks_exact(2)) {
            *word = u16::from_le_bytes([pair[0], pair[1]]);
        }
        Some(Self { wram, sram, vram })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_access_is_explicitly_little_endian() {
        let mut memory = LegacyMemory::default();
        memory.write_u16_le(0x1234, 0xabcd);
        assert_eq!(&memory.wram()[0x1234..0x1236], &[0xcd, 0xab]);
        assert_eq!(memory.read_u16_le(0x1234), 0xabcd);
    }

    #[test]
    fn canonical_bytes_have_stable_region_order_and_endianness() {
        let mut memory = LegacyMemory::default();
        memory.wram_mut()[0] = 0x11;
        memory.sram_mut()[0] = 0x22;
        memory.vram_mut()[0] = 0x4433;

        let bytes = memory.canonical_state().to_bytes();
        assert_eq!(bytes.len(), CanonicalState::BYTE_LEN);
        assert_eq!(bytes[0], 0x11);
        assert_eq!(bytes[WRAM_SIZE], 0x22);
        assert_eq!(&bytes[WRAM_SIZE + SRAM_SIZE..][..2], &[0x33, 0x44]);
        assert_eq!(
            CanonicalState::from_bytes(&bytes).unwrap(),
            memory.canonical_state()
        );
        assert!(CanonicalState::from_bytes(&bytes[..bytes.len() - 1]).is_none());
    }
}
