//! Differential comparison support for the pinned C oracle and Rust port.

pub mod c_oracle;
pub mod legacy_replay;
pub mod legacy_snapshot;
pub mod schema;

use z3_core::{CanonicalState, SRAM_SIZE, VRAM_WORDS, WRAM_SIZE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Wram,
    Sram,
    Vram,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Difference {
    pub region: Region,
    /// Byte offset within the named region. VRAM words are compared as
    /// little-endian byte pairs so diagnostics remain byte-precise.
    pub offset: usize,
    pub reference: u8,
    pub rust: u8,
}

#[must_use]
pub fn first_difference(reference: &CanonicalState, rust: &CanonicalState) -> Option<Difference> {
    if let Some(difference) = compare_bytes(Region::Wram, &reference.wram[..], &rust.wram[..]) {
        return Some(difference);
    }
    if let Some(difference) = compare_bytes(Region::Sram, &reference.sram[..], &rust.sram[..]) {
        return Some(difference);
    }

    for index in 0..VRAM_WORDS {
        let reference_bytes = reference.vram[index].to_le_bytes();
        let rust_bytes = rust.vram[index].to_le_bytes();
        if let Some(mut difference) = compare_bytes(Region::Vram, &reference_bytes, &rust_bytes) {
            difference.offset += index * 2;
            return Some(difference);
        }
    }
    None
}

fn compare_bytes(region: Region, reference: &[u8], rust: &[u8]) -> Option<Difference> {
    reference
        .iter()
        .zip(rust)
        .enumerate()
        .find_map(|(offset, (&reference, &rust))| {
            (reference != rust).then_some(Difference {
                region,
                offset,
                reference,
                rust,
            })
        })
}

const _: () = assert!(CanonicalState::BYTE_LEN == WRAM_SIZE + SRAM_SIZE + VRAM_WORDS * 2);

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::LegacyMemory;

    #[test]
    fn equal_states_have_no_difference() {
        let state = LegacyMemory::default().canonical_state();
        assert_eq!(first_difference(&state, &state), None);
    }

    #[test]
    fn reports_first_region_and_byte_offset() {
        let reference = LegacyMemory::default().canonical_state();
        let mut rust_memory = LegacyMemory::default();
        rust_memory.vram_mut()[7] = 0x0100;
        let rust = rust_memory.canonical_state();

        assert_eq!(
            first_difference(&reference, &rust),
            Some(Difference {
                region: Region::Vram,
                offset: 15,
                reference: 0,
                rust: 1,
            })
        );
    }
}
