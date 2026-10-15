//! Runtime boundary for the C `InternalSaveLoad` byte image.
//!
//! This is a wire format, not an owned gameplay-state cache. Unknown historical
//! blocks stay byte-for-byte in the template; only live device regions are
//! overlaid when saving. The verifier intentionally keeps an independent codec.

use z3_core::{SRAM_SIZE, WRAM_SIZE};
use z3_snes::saveload::SaveLoadError;

pub const LEGACY_SNAPSHOT_LEN: usize = 275_465;

/// The ordered regions of the pinned C snapshot ABI, including opaque gaps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotRegion {
    Prefix,
    SpcRam,
    BeforeDsp,
    Dsp,
    BeforeDma,
    Dma,
    Ppu,
    Sram,
    BeforeWram,
    Wram,
    Suffix,
}

impl SnapshotRegion {
    pub const ALL: [Self; 11] = [
        Self::Prefix,
        Self::SpcRam,
        Self::BeforeDsp,
        Self::Dsp,
        Self::BeforeDma,
        Self::Dma,
        Self::Ppu,
        Self::Sram,
        Self::BeforeWram,
        Self::Wram,
        Self::Suffix,
    ];

    const fn range(self) -> std::ops::Range<usize> {
        match self {
            Self::Prefix => 0..27,
            Self::SpcRam => 27..65_563,
            Self::BeforeDsp => 65_563..65_603,
            Self::Dsp => 65_603..68_627,
            Self::BeforeDma => 68_627..68_642,
            Self::Dma => 68_642..68_834,
            Self::Ppu => 68_834..136_139,
            Self::Sram => 136_139..144_331,
            Self::BeforeWram => 144_331..144_389,
            Self::Wram => 144_389..275_461,
            Self::Suffix => 275_461..275_465,
        }
    }
}

/// Validates length only, retaining the runtime's existing acceptance policy.
/// Device codecs still interpret their own bytes in the original transfer order.
#[derive(Clone, Copy, Debug)]
pub struct Snapshot<'a> {
    bytes: &'a [u8; LEGACY_SNAPSHOT_LEN],
}

impl<'a> Snapshot<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, LegacySnapshotError> {
        let length = bytes.len();
        Ok(Self {
            bytes: bytes
                .try_into()
                .map_err(|_| LegacySnapshotError::WrongLength(length))?,
        })
    }

    pub fn region(self, region: SnapshotRegion) -> &'a [u8] {
        &self.bytes[region.range()]
    }

    pub fn spc_ram(self) -> &'a [u8; 65_536] {
        self.region(SnapshotRegion::SpcRam)
            .try_into()
            .expect("fixed SPC RAM schema")
    }

    pub fn sram(self) -> &'a [u8; SRAM_SIZE] {
        self.region(SnapshotRegion::Sram)
            .try_into()
            .expect("fixed SRAM schema")
    }

    pub fn wram(self) -> &'a [u8; WRAM_SIZE] {
        self.region(SnapshotRegion::Wram)
            .try_into()
            .expect("fixed WRAM schema")
    }
}

/// A validated destination. Borrowing a region neither clears nor stages it.
pub struct SnapshotMut<'a> {
    bytes: &'a mut [u8; LEGACY_SNAPSHOT_LEN],
}

impl<'a> SnapshotMut<'a> {
    pub fn parse(bytes: &'a mut [u8]) -> Result<Self, LegacySnapshotError> {
        let length = bytes.len();
        Ok(Self {
            bytes: bytes
                .try_into()
                .map_err(|_| LegacySnapshotError::WrongLength(length))?,
        })
    }

    pub fn region_mut(&mut self, region: SnapshotRegion) -> &mut [u8] {
        &mut self.bytes[region.range()]
    }
}

/// Retained historical bytes, not another authority for live machine state.
#[derive(Debug)]
pub(crate) struct SnapshotTemplate {
    bytes: Box<[u8]>,
}

impl Default for SnapshotTemplate {
    fn default() -> Self {
        Self {
            bytes: vec![0; LEGACY_SNAPSHOT_LEN].into_boxed_slice(),
        }
    }
}

impl SnapshotTemplate {
    pub fn capture(&mut self, snapshot: Snapshot<'_>) {
        self.bytes.copy_from_slice(snapshot.bytes);
    }

    pub fn restore_into(&self, destination: &mut SnapshotMut<'_>) {
        destination.bytes.copy_from_slice(&self.bytes);
    }
}

#[derive(Debug)]
pub enum LegacySnapshotError {
    WrongLength(usize),
    Device(SaveLoadError),
}

impl std::fmt::Display for LegacySnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongLength(actual) => write!(
                f,
                "legacy snapshot has {actual} bytes; expected {LEGACY_SNAPSHOT_LEN}"
            ),
            Self::Device(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for LegacySnapshotError {}

impl From<SaveLoadError> for LegacySnapshotError {
    fn from(value: SaveLoadError) -> Self {
        Self::Device(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_covers_the_image_once_with_the_original_region_boundaries() {
        let bytes = (0..LEGACY_SNAPSHOT_LEN)
            .map(|index| (index as u8).wrapping_mul(37))
            .collect::<Vec<_>>();
        let snapshot = Snapshot::parse(&bytes).unwrap();
        let mut joined = Vec::new();
        for region in SnapshotRegion::ALL {
            joined.extend_from_slice(snapshot.region(region));
        }
        assert_eq!(joined, bytes);
        assert_eq!(snapshot.spc_ram(), &bytes[27..65_563]);
        assert_eq!(snapshot.region(SnapshotRegion::Dsp), &bytes[65_603..68_627]);
        assert_eq!(snapshot.region(SnapshotRegion::Dma), &bytes[68_642..68_834]);
        assert_eq!(
            snapshot.region(SnapshotRegion::Ppu),
            &bytes[68_834..136_139]
        );
        assert_eq!(snapshot.sram(), &bytes[136_139..144_331]);
        assert_eq!(snapshot.wram(), &bytes[144_389..275_461]);
    }

    #[test]
    fn template_retains_every_opaque_byte_and_mutation_is_immediate_and_local() {
        let original = (0..LEGACY_SNAPSHOT_LEN)
            .map(|index| (index as u8).wrapping_add((index >> 8) as u8))
            .collect::<Vec<_>>();
        let mut template = SnapshotTemplate::default();
        template.capture(Snapshot::parse(&original).unwrap());
        let mut output = vec![0xaa; LEGACY_SNAPSHOT_LEN];
        let mut destination = SnapshotMut::parse(&mut output).unwrap();
        template.restore_into(&mut destination);
        destination.region_mut(SnapshotRegion::Sram).fill(0x77);
        let mut expected = original;
        expected[136_139..144_331].fill(0x77);
        assert_eq!(output, expected);
        // Saving never writes the template back from live regions.
        template.restore_into(&mut SnapshotMut::parse(&mut output).unwrap());
        assert_eq!(
            output[136_139],
            136_139usize as u8 + (136_139usize >> 8) as u8
        );
    }

    #[test]
    fn wrong_lengths_are_rejected_without_touching_the_destination() {
        for length in [0, 3, LEGACY_SNAPSHOT_LEN - 1, LEGACY_SNAPSHOT_LEN + 1] {
            let mut bytes = vec![0xa5; length];
            assert!(
                matches!(Snapshot::parse(&bytes), Err(LegacySnapshotError::WrongLength(n)) if n == length)
            );
            assert!(
                matches!(SnapshotMut::parse(&mut bytes), Err(LegacySnapshotError::WrongLength(n)) if n == length)
            );
            assert!(bytes.iter().all(|&value| value == 0xa5));
        }
    }
}
