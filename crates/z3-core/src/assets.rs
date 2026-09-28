//! Reader for the exact `zelda3_assets.dat` format produced by Python.
//!
//! The format is intentionally kept compatible with `LoadAssets` in the C
//! port. It consists of an 88-byte header, one little-endian `u32` size per
//! asset, an informational NUL-separated key table, and 4-byte-aligned asset
//! payloads. Assets are addressed by their stable numeric index.

use core::fmt;

/// Number of entries in the asset table generated for this source revision.
pub const ASSET_COUNT: usize = 165;

/// Size of the fixed portion of a `zelda3_assets.dat` header.
pub const HEADER_SIZE: usize = 88;

/// Format/version signature generated from the ordered asset keys.
///
/// The first 16 bytes are `Zelda3_v0     \n\0`; the remaining bytes are the
/// SHA-256 digest of the ordered, NUL-terminated asset key table. This is the
/// same constant as `kAssets_Sig` in `src/assets.h`.
pub const ASSET_SIGNATURE: [u8; 48] = [
    90, 101, 108, 100, 97, 51, 95, 118, 48, 32, 32, 32, 32, 32, 10, 0, 27, 174, 233, 45, 74, 174,
    252, 50, 49, 27, 153, 197, 27, 43, 216, 197, 132, 101, 173, 169, 36, 108, 15, 155, 176, 169,
    57, 131, 174, 101, 51, 207,
];

/// A validated view of an asset container.
#[derive(Debug)]
pub struct AssetFile<'a> {
    bytes: &'a [u8],
    keys: &'a [u8],
    assets: Vec<&'a [u8]>,
}

impl<'a> AssetFile<'a> {
    /// Parses and validates an asset container without copying its payloads.
    ///
    /// Like the C loader, this accepts trailing bytes and treats the key table
    /// as informational. Its digest is already part of [`ASSET_SIGNATURE`];
    /// game code uses stable numeric indices rather than key lookup.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AssetError> {
        let size_table_end = HEADER_SIZE
            .checked_add(
                ASSET_COUNT
                    .checked_mul(4)
                    .ok_or(AssetError::OffsetOverflow)?,
            )
            .ok_or(AssetError::OffsetOverflow)?;

        if bytes.len() < size_table_end {
            return Err(AssetError::TooShort {
                minimum: size_table_end,
                actual: bytes.len(),
            });
        }
        if bytes.get(..ASSET_SIGNATURE.len()) != Some(ASSET_SIGNATURE.as_slice()) {
            return Err(AssetError::InvalidSignature);
        }

        let count = read_u32(bytes, 80).ok_or(AssetError::OffsetOverflow)? as usize;
        if count != ASSET_COUNT {
            return Err(AssetError::WrongAssetCount {
                expected: ASSET_COUNT,
                actual: count,
            });
        }

        let keys_len = read_u32(bytes, 84).ok_or(AssetError::OffsetOverflow)? as usize;
        let keys_end = size_table_end
            .checked_add(keys_len)
            .ok_or(AssetError::OffsetOverflow)?;
        let keys = bytes
            .get(size_table_end..keys_end)
            .ok_or(AssetError::KeyTableOutOfBounds {
                offset: size_table_end,
                size: keys_len,
                file_size: bytes.len(),
            })?;

        let mut assets = Vec::with_capacity(ASSET_COUNT);
        let mut offset = keys_end;
        for index in 0..ASSET_COUNT {
            offset = align_four(offset).ok_or(AssetError::OffsetOverflow)?;
            let size_offset = HEADER_SIZE + index * 4;
            let size = read_u32(bytes, size_offset).ok_or(AssetError::OffsetOverflow)? as usize;
            let end = offset.checked_add(size).ok_or(AssetError::OffsetOverflow)?;
            let data = bytes.get(offset..end).ok_or(AssetError::AssetOutOfBounds {
                index,
                offset,
                size,
                file_size: bytes.len(),
            })?;
            assets.push(data);
            offset = end;
        }

        Ok(Self {
            bytes,
            keys,
            assets,
        })
    }

    /// Returns the complete validated container bytes.
    #[must_use]
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the informational NUL-separated key table.
    #[must_use]
    pub fn key_table(&self) -> &'a [u8] {
        self.keys
    }

    /// Returns the number of assets in this format revision.
    #[must_use]
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Returns whether the container has no assets.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Returns an asset by the stable numeric index used by `src/assets.h`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&'a [u8]> {
        self.assets.get(index).copied()
    }

    /// Iterates over asset payloads in their stable numeric-index order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &'a [u8]> + '_ {
        self.assets.iter().copied()
    }
}

/// A structural error in a `zelda3_assets.dat` container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetError {
    /// The file does not contain the complete fixed header and size table.
    TooShort { minimum: usize, actual: usize },
    /// The version or ordered-asset-key signature differs from this revision.
    InvalidSignature,
    /// The file was produced for a different asset index table.
    WrongAssetCount { expected: usize, actual: usize },
    /// An offset calculation could not be represented safely.
    OffsetOverflow,
    /// The declared asset-key table extends past the input.
    KeyTableOutOfBounds {
        offset: usize,
        size: usize,
        file_size: usize,
    },
    /// A declared asset payload extends past the input.
    AssetOutOfBounds {
        index: usize,
        offset: usize,
        size: usize,
        file_size: usize,
    },
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort { minimum, actual } => {
                write!(
                    f,
                    "asset file is too short: need {minimum} bytes, got {actual}"
                )
            }
            Self::InvalidSignature => write!(f, "invalid asset format/version signature"),
            Self::WrongAssetCount { expected, actual } => {
                write!(f, "wrong asset count: expected {expected}, got {actual}")
            }
            Self::OffsetOverflow => write!(f, "asset offset arithmetic overflowed"),
            Self::KeyTableOutOfBounds {
                offset,
                size,
                file_size,
            } => write!(
                f,
                "asset key table at {offset} with size {size} exceeds file size {file_size}"
            ),
            Self::AssetOutOfBounds {
                index,
                offset,
                size,
                file_size,
            } => write!(
                f,
                "asset {index} at {offset} with size {size} exceeds file size {file_size}"
            ),
        }
    }
}

impl std::error::Error for AssetError {}

/// Reads one entry from the packed-array representation used by
/// `FindIndexInMemblk` in the C port.
///
/// The final little-endian `u16` stores the highest valid index. Values below
/// 8192 select a `u16` offset table; values at or above 8192 select a `u32`
/// table after subtracting 8192. Malformed data and out-of-range indices return
/// `None` rather than creating an invalid slice.
#[must_use]
pub fn packed_array_entry(data: &[u8], index: usize) -> Option<&[u8]> {
    let end = data.len().checked_sub(2)?;
    let encoded_max = read_u16(data, end)? as usize;
    let (max_index, offset_width) = if encoded_max < 8192 {
        (encoded_max, 2usize)
    } else {
        (encoded_max - 8192, 4usize)
    };

    if index > max_index {
        return None;
    }
    let table_size = max_index.checked_mul(offset_width)?;
    if table_size > end {
        return None;
    }

    let left = if index == 0 {
        table_size
    } else {
        table_size.checked_add(read_offset(data, (index - 1) * offset_width, offset_width)?)?
    };
    let right = if index == max_index {
        end
    } else {
        table_size.checked_add(read_offset(data, index * offset_width, offset_width)?)?
    };

    if left > right || right > end {
        return None;
    }
    data.get(left..right)
}

fn align_four(value: usize) -> Option<usize> {
    value.checked_add(3).map(|value| value & !3)
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    let bytes: [u8; 2] = data.get(offset..offset.checked_add(2)?)?.try_into().ok()?;
    Some(u16::from_le_bytes(bytes))
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    let bytes: [u8; 4] = data.get(offset..offset.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn read_offset(data: &[u8], offset: usize, width: usize) -> Option<usize> {
    match width {
        2 => Some(read_u16(data, offset)? as usize),
        4 => Some(read_u32(data, offset)? as usize),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_file(payloads: &[Vec<u8>]) -> Vec<u8> {
        assert!(payloads.len() <= ASSET_COUNT);

        let mut all_payloads = payloads.to_vec();
        all_payloads.resize_with(ASSET_COUNT, Vec::new);
        let keys = (0..ASSET_COUNT)
            .flat_map(|index| format!("asset_{index}\0").into_bytes())
            .collect::<Vec<_>>();

        let mut file = Vec::new();
        file.extend_from_slice(&ASSET_SIGNATURE);
        file.extend_from_slice(&[0; 32]);
        file.extend_from_slice(&(ASSET_COUNT as u32).to_le_bytes());
        file.extend_from_slice(&(keys.len() as u32).to_le_bytes());
        for payload in &all_payloads {
            file.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        }
        file.extend_from_slice(&keys);
        for payload in &all_payloads {
            while file.len() & 3 != 0 {
                file.push(0);
            }
            file.extend_from_slice(payload);
        }
        file
    }

    #[test]
    fn parses_assets_with_python_alignment_rules() {
        let bytes = synthetic_file(&[vec![1], vec![2, 3, 4], vec![5, 6]]);
        let file = AssetFile::parse(&bytes).unwrap();

        assert_eq!(file.len(), ASSET_COUNT);
        assert!(!file.is_empty());
        assert_eq!(file.get(0), Some(&[1][..]));
        assert_eq!(file.get(1), Some(&[2, 3, 4][..]));
        assert_eq!(file.get(2), Some(&[5, 6][..]));
        assert_eq!(file.get(ASSET_COUNT), None);
        assert_eq!(file.iter().count(), ASSET_COUNT);
        assert!(file.key_table().starts_with(b"asset_0\0asset_1\0"));
        assert_eq!(file.bytes(), bytes);
    }

    #[test]
    fn rejects_wrong_signature_and_asset_count() {
        let mut bad_signature = synthetic_file(&[]);
        bad_signature[0] ^= 0xff;
        assert_eq!(
            AssetFile::parse(&bad_signature).unwrap_err(),
            AssetError::InvalidSignature
        );

        let mut bad_count = synthetic_file(&[]);
        bad_count[80..84].copy_from_slice(&164u32.to_le_bytes());
        assert_eq!(
            AssetFile::parse(&bad_count).unwrap_err(),
            AssetError::WrongAssetCount {
                expected: ASSET_COUNT,
                actual: 164,
            }
        );
    }

    #[test]
    fn rejects_truncated_key_table_and_payload() {
        let mut bad_keys = synthetic_file(&[]);
        bad_keys[84..88].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            AssetFile::parse(&bad_keys),
            Err(AssetError::KeyTableOutOfBounds { .. })
        ));

        let good = synthetic_file(&[vec![1, 2, 3, 4]]);
        let truncated = &good[..good.len() - 1];
        assert!(matches!(
            AssetFile::parse(truncated),
            Err(AssetError::AssetOutOfBounds { index: 0, .. })
        ));
    }

    #[test]
    fn reads_sixteen_bit_packed_arrays() {
        // Offsets [2, 5], payloads "ab", "cde", "f", highest index 2.
        let data = [2, 0, 5, 0, b'a', b'b', b'c', b'd', b'e', b'f', 2, 0];
        assert_eq!(packed_array_entry(&data, 0), Some(&b"ab"[..]));
        assert_eq!(packed_array_entry(&data, 1), Some(&b"cde"[..]));
        assert_eq!(packed_array_entry(&data, 2), Some(&b"f"[..]));
        assert_eq!(packed_array_entry(&data, 3), None);
    }

    #[test]
    fn reads_thirty_two_bit_packed_array_offsets() {
        // 8,193 empty entries select the wide-offset representation. There
        // are 8,192 zero offsets followed by the encoded highest index.
        let mut data = vec![0; 8192 * 4];
        data.extend_from_slice(&(8192u16 + 8192).to_le_bytes());

        assert_eq!(packed_array_entry(&data, 0), Some(&[][..]));
        assert_eq!(packed_array_entry(&data, 8192), Some(&[][..]));
        assert_eq!(packed_array_entry(&data, 8193), None);
    }

    #[test]
    fn rejects_malformed_packed_arrays() {
        assert_eq!(packed_array_entry(&[], 0), None);
        // The first end offset points beyond the packed payload.
        assert_eq!(packed_array_entry(&[0xff, 0xff, b'x', 1, 0], 0), None);
    }
}
