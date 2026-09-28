//! Versioned pointer-free state format used by the C/Rust differential runner.
//!
//! The wire format deliberately contains no native-width integers, pointers,
//! struct padding, or implicit byte order. Regions must be supplied in strictly
//! increasing ID order so one logical state has exactly one representation.

use std::error::Error;
use std::fmt;

/// Eight-byte marker at the beginning of every canonical state stream.
pub const MAGIC: [u8; 8] = *b"Z3STATE\0";
/// The only stream version understood by this implementation.
pub const VERSION: u32 = 1;
/// Size of the fixed stream header in bytes.
pub const HEADER_LEN: usize = 48;
/// Size of a region header in bytes.
pub const REGION_HEADER_LEN: usize = 16;
/// Defensive upper bound for untrusted region counts.
pub const MAX_REGIONS: u32 = 1_024;

/// Stable region IDs shared by the C oracle and Rust implementation.
pub mod region_id {
    pub const WRAM: u32 = 1;
    pub const SRAM: u32 = 2;
    pub const VRAM: u32 = 3;
    pub const CGRAM: u32 = 4;
    pub const OAM: u32 = 5;
    pub const PPU_REGS: u32 = 6;
    pub const DMA: u32 = 7;
    pub const SPC_RAM: u32 = 8;
    pub const SPC_PLAYER: u32 = 9;
    pub const DSP: u32 = 10;
    pub const APU_QUEUE: u32 = 11;
    pub const DRIVER: u32 = 12;
}

/// Metadata common to a complete canonical state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    /// Hash of the ordered region schema and field definitions.
    pub schema_hash: u64,
    /// Hash of determinism-affecting runtime configuration.
    pub configuration_hash: u64,
    /// Logical frame number at which the snapshot was taken.
    pub frame_number: u64,
}

/// One region to be encoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionRef<'a> {
    pub id: u32,
    pub bytes: &'a [u8],
}

/// A decoded region borrowing its bytes from the source stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region<'a> {
    pub id: u32,
    pub bytes: &'a [u8],
}

/// A fully validated canonical state stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document<'a> {
    pub header: Header,
    pub regions: Vec<Region<'a>>,
}

impl<'a> Document<'a> {
    /// Returns a region by its stable ID.
    #[must_use]
    pub fn region(&self, id: u32) -> Option<&'a [u8]> {
        self.regions
            .binary_search_by_key(&id, |region| region.id)
            .ok()
            .map(|index| self.regions[index].bytes)
    }
}

/// Failure while constructing a canonical stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncodeError {
    TooManyRegions(usize),
    ZeroRegionId,
    RegionsNotStrictlyOrdered { previous: u32, current: u32 },
    RegionTooLarge { id: u32, len: usize },
    StreamTooLarge,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyRegions(count) => write!(f, "too many canonical regions: {count}"),
            Self::ZeroRegionId => f.write_str("canonical region ID zero is reserved"),
            Self::RegionsNotStrictlyOrdered { previous, current } => write!(
                f,
                "canonical regions are not strictly ordered: {previous} then {current}"
            ),
            Self::RegionTooLarge { id, len } => {
                write!(f, "canonical region {id} is too large: {len} bytes")
            }
            Self::StreamTooLarge => f.write_str("canonical stream size overflow"),
        }
    }
}

impl Error for EncodeError {}

/// Failure while validating a canonical stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    Truncated { offset: usize, needed: usize },
    BadMagic([u8; 8]),
    UnsupportedVersion(u32),
    NonZeroHeaderFlags(u32),
    TooManyRegions(u32),
    ZeroRegionId { index: u32 },
    NonZeroRegionFlags { id: u32, flags: u32 },
    RegionsNotStrictlyOrdered { previous: u32, current: u32 },
    RegionLengthTooLarge { id: u32, len: u64 },
    TrailingBytes(usize),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { offset, needed } => {
                write!(
                    f,
                    "truncated canonical stream at {offset}; need {needed} bytes"
                )
            }
            Self::BadMagic(magic) => write!(f, "invalid canonical stream magic: {magic:02x?}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported canonical stream version {version}")
            }
            Self::NonZeroHeaderFlags(flags) => {
                write!(f, "unsupported canonical header flags 0x{flags:08x}")
            }
            Self::TooManyRegions(count) => write!(f, "too many canonical regions: {count}"),
            Self::ZeroRegionId { index } => write!(f, "region {index} uses reserved ID zero"),
            Self::NonZeroRegionFlags { id, flags } => {
                write!(f, "region {id} uses unsupported flags 0x{flags:08x}")
            }
            Self::RegionsNotStrictlyOrdered { previous, current } => write!(
                f,
                "canonical regions are not strictly ordered: {previous} then {current}"
            ),
            Self::RegionLengthTooLarge { id, len } => {
                write!(
                    f,
                    "canonical region {id} length does not fit this host: {len}"
                )
            }
            Self::TrailingBytes(count) => {
                write!(f, "canonical stream has {count} trailing bytes")
            }
        }
    }
}

impl Error for DecodeError {}

/// Encodes an ordered set of regions into the canonical wire format.
pub fn encode(header: Header, regions: &[RegionRef<'_>]) -> Result<Vec<u8>, EncodeError> {
    if regions.len() > MAX_REGIONS as usize {
        return Err(EncodeError::TooManyRegions(regions.len()));
    }

    let mut previous = None;
    let mut total_len = HEADER_LEN;
    for region in regions {
        if region.id == 0 {
            return Err(EncodeError::ZeroRegionId);
        }
        if let Some(previous) = previous
            && region.id <= previous
        {
            return Err(EncodeError::RegionsNotStrictlyOrdered {
                previous,
                current: region.id,
            });
        }
        previous = Some(region.id);
        let _ = u64::try_from(region.bytes.len()).map_err(|_| EncodeError::RegionTooLarge {
            id: region.id,
            len: region.bytes.len(),
        })?;
        total_len = total_len
            .checked_add(REGION_HEADER_LEN)
            .and_then(|len| len.checked_add(region.bytes.len()))
            .ok_or(EncodeError::StreamTooLarge)?;
    }

    let mut output = Vec::with_capacity(total_len);
    output.extend_from_slice(&MAGIC);
    push_u32(&mut output, VERSION);
    push_u32(&mut output, 0); // Reserved header flags.
    push_u64(&mut output, header.schema_hash);
    push_u64(&mut output, header.configuration_hash);
    push_u64(&mut output, header.frame_number);
    push_u32(&mut output, regions.len() as u32);
    push_u32(&mut output, 0); // Reserved for future header expansion.

    for region in regions {
        push_u32(&mut output, region.id);
        push_u32(&mut output, 0); // Reserved region flags.
        push_u64(&mut output, region.bytes.len() as u64);
        output.extend_from_slice(region.bytes);
    }
    debug_assert_eq!(output.len(), total_len);
    Ok(output)
}

/// Decodes and strictly validates a canonical wire stream.
pub fn decode(source: &[u8]) -> Result<Document<'_>, DecodeError> {
    let mut reader = Reader::new(source);
    let magic = reader.array::<8>()?;
    if magic != MAGIC {
        return Err(DecodeError::BadMagic(magic));
    }
    let version = reader.u32()?;
    if version != VERSION {
        return Err(DecodeError::UnsupportedVersion(version));
    }
    let flags = reader.u32()?;
    if flags != 0 {
        return Err(DecodeError::NonZeroHeaderFlags(flags));
    }
    let header = Header {
        schema_hash: reader.u64()?,
        configuration_hash: reader.u64()?,
        frame_number: reader.u64()?,
    };
    let count = reader.u32()?;
    if count > MAX_REGIONS {
        return Err(DecodeError::TooManyRegions(count));
    }
    let reserved = reader.u32()?;
    if reserved != 0 {
        return Err(DecodeError::NonZeroHeaderFlags(reserved));
    }

    let mut regions = Vec::with_capacity(count as usize);
    let mut previous = None;
    for index in 0..count {
        let id = reader.u32()?;
        if id == 0 {
            return Err(DecodeError::ZeroRegionId { index });
        }
        let flags = reader.u32()?;
        if flags != 0 {
            return Err(DecodeError::NonZeroRegionFlags { id, flags });
        }
        let len_u64 = reader.u64()?;
        let len = usize::try_from(len_u64)
            .map_err(|_| DecodeError::RegionLengthTooLarge { id, len: len_u64 })?;
        if let Some(previous) = previous
            && id <= previous
        {
            return Err(DecodeError::RegionsNotStrictlyOrdered {
                previous,
                current: id,
            });
        }
        previous = Some(id);
        let bytes = reader.bytes(len)?;
        regions.push(Region { id, bytes });
    }
    if reader.remaining() != 0 {
        return Err(DecodeError::TrailingBytes(reader.remaining()));
    }
    Ok(Document { header, regions })
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

struct Reader<'a> {
    source: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    fn remaining(&self) -> usize {
        self.source.len() - self.position
    }

    fn bytes(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(DecodeError::Truncated {
                offset: self.position,
                needed: len,
            })?;
        let bytes = self
            .source
            .get(self.position..end)
            .ok_or(DecodeError::Truncated {
                offset: self.position,
                needed: len,
            })?;
        self.position = end;
        Ok(bytes)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        Ok(self.bytes(N)?.try_into().expect("slice length was checked"))
    }

    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.array()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: Header = Header {
        schema_hash: 0x0102_0304_0506_0708,
        configuration_hash: 0x1112_1314_1516_1718,
        frame_number: 0x2122_2324_2526_2728,
    };

    #[test]
    fn round_trip_is_explicitly_little_endian() {
        let encoded = encode(
            HEADER,
            &[
                RegionRef {
                    id: region_id::WRAM,
                    bytes: &[0xaa, 0xbb],
                },
                RegionRef {
                    id: region_id::VRAM,
                    bytes: &[0x34, 0x12],
                },
            ],
        )
        .unwrap();

        assert_eq!(&encoded[0..8], b"Z3STATE\0");
        assert_eq!(&encoded[8..12], &[1, 0, 0, 0]);
        assert_eq!(&encoded[16..24], &HEADER.schema_hash.to_le_bytes());
        assert_eq!(&encoded[48..52], &region_id::WRAM.to_le_bytes());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.header, HEADER);
        assert_eq!(decoded.region(region_id::WRAM), Some(&[0xaa, 0xbb][..]));
        assert_eq!(decoded.region(region_id::VRAM), Some(&[0x34, 0x12][..]));
        assert_eq!(decoded.region(region_id::SRAM), None);
    }

    #[test]
    fn encoding_rejects_duplicate_or_unsorted_regions() {
        let error = encode(
            HEADER,
            &[
                RegionRef { id: 2, bytes: &[] },
                RegionRef { id: 2, bytes: &[] },
            ],
        )
        .unwrap_err();
        assert_eq!(
            error,
            EncodeError::RegionsNotStrictlyOrdered {
                previous: 2,
                current: 2
            }
        );
    }

    #[test]
    fn decoding_rejects_truncation_and_trailing_bytes() {
        let encoded = encode(HEADER, &[RegionRef { id: 1, bytes: &[7] }]).unwrap();
        assert!(matches!(
            decode(&encoded[..encoded.len() - 1]),
            Err(DecodeError::Truncated { .. })
        ));

        let mut with_trailer = encoded;
        with_trailer.push(0);
        assert_eq!(decode(&with_trailer), Err(DecodeError::TrailingBytes(1)));
    }

    #[test]
    fn decoding_rejects_noncanonical_region_order() {
        let mut encoded = encode(
            HEADER,
            &[
                RegionRef { id: 1, bytes: &[] },
                RegionRef { id: 2, bytes: &[] },
            ],
        )
        .unwrap();
        encoded[64..68].copy_from_slice(&1_u32.to_le_bytes());
        assert_eq!(
            decode(&encoded),
            Err(DecodeError::RegionsNotStrictlyOrdered {
                previous: 1,
                current: 1
            })
        );
    }
}
