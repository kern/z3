//! Graphics transfer operations above the PPU's legacy word storage.
//!
//! Address units, byte lanes, column stride and clipping policy are explicit.
//! The compatibility renderer remains authoritative; these operations neither
//! reorder transfers nor defer their effects until the end of a frame.

use crate::nmi::NmiError;

/// Legacy UVRAM DMA packets (little-endian destination, mode and byte count).
/// The caller supplies the `uvram.data` suffix, excluding its containing pad.
pub struct UploadPacketStream<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> UploadPacketStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn next_upload(&mut self) -> Result<Option<VramUpload<'a>>, NmiError> {
        let destination = u16::from_le_bytes([self.data[self.cursor], self.data[self.cursor + 1]]);
        if destination == 0xffff {
            return Ok(None);
        }
        let mode = self.data[self.cursor + 2];
        let len = usize::from(self.data[self.cursor + 3]);
        self.cursor += 4;
        let bytes = &self.data[self.cursor..self.cursor + len];
        let layout = match mode {
            0x80 => TransferLayout::PackedBytes,
            0x81 => TransferLayout::ColumnWords,
            other => return Err(NmiError::UnsupportedVmain(other)),
        };
        self.cursor += len;
        Ok(Some(VramUpload {
            destination: VramWordAddress::new(usize::from(destination)),
            layout,
            bounds: TransferBounds::Strict,
            bytes,
        }))
    }
}

/// Legacy overworld scroll packets share a length/direction header. The byte
/// count is rounded down to whole words, and columns clip instead of wrapping.
pub struct ScrollUploadStream<'a> {
    data: &'a [u8],
    cursor: usize,
    len: usize,
    vertical: bool,
    started: bool,
}

impl<'a> ScrollUploadStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let header = u16::from_le_bytes([data[0], data[1]]);
        Self {
            data,
            cursor: 2,
            len: usize::from(header & 0x3fff) & !1,
            vertical: header & 0x8000 != 0,
            started: false,
        }
    }

    pub fn next_upload(&mut self) -> Option<VramUpload<'a>> {
        // Check the next marker only after the preceding upload was applied.
        // The first destination is unconditional, as in the C do/while loop.
        if self.started && self.data[self.cursor + 1] & 0x80 != 0 {
            return None;
        }
        self.started = true;
        let destination = u16::from_le_bytes([self.data[self.cursor], self.data[self.cursor + 1]]);
        self.cursor += 2;
        let bytes = &self.data[self.cursor..self.cursor + self.len];
        self.cursor += self.len;
        Some(VramUpload {
            destination: VramWordAddress::new(usize::from(destination)),
            layout: if self.vertical {
                TransferLayout::ColumnWords
            } else {
                TransferLayout::PackedBytes
            },
            bounds: if self.vertical {
                TransferBounds::ClipOutsideVram
            } else {
                TransferBounds::Strict
            },
            bytes,
        })
    }
}

/// An index into VRAM words, not a byte offset or a WRAM address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VramWordAddress(usize);

impl VramWordAddress {
    pub const fn new(words: usize) -> Self {
        Self(words)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

/// The reference has both strict transfers and raw-pointer transfers whose
/// off-buffer suffix never enters observable VRAM. Neither policy wraps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferBounds {
    Strict,
    ClipOutsideVram,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferLayout {
    PackedBytes,
    ColumnWords,
    LowBytes,
}

/// A borrowed transfer, applied immediately in source order.
#[derive(Clone, Copy, Debug)]
pub struct VramUpload<'a> {
    pub destination: VramWordAddress,
    pub layout: TransferLayout,
    pub bounds: TransferBounds,
    pub bytes: &'a [u8],
}

impl VramUpload<'_> {
    #[track_caller]
    pub fn apply(self, vram: &mut [u16]) -> Result<(), NmiError> {
        let destination = self.destination.index();
        match self.layout {
            TransferLayout::PackedBytes => {
                let start = destination
                    .checked_mul(2)
                    .ok_or(NmiError::InvalidVramTarget(destination))?;
                if self.bounds == TransferBounds::Strict
                    && start.saturating_add(self.bytes.len()) > vram.len() * 2
                {
                    eprintln!(
                        "invalid VRAM DMA caller={} target={destination:04x} len={:x}",
                        core::panic::Location::caller(),
                        self.bytes.len()
                    );
                    return Err(NmiError::InvalidVramTarget(destination));
                }
                for (offset, &value) in self.bytes.iter().enumerate() {
                    let address = start + offset;
                    if let Some(word) = vram.get_mut(address >> 1) {
                        if address & 1 == 0 {
                            *word = (*word & 0xff00) | u16::from(value);
                        } else {
                            *word = (*word & 0xff) | (u16::from(value) << 8);
                        }
                    }
                }
            }
            TransferLayout::ColumnWords => {
                if self.bytes.len() & 1 != 0 {
                    return Err(NmiError::TruncatedData);
                }
                for (row, pair) in self.bytes.chunks_exact(2).enumerate() {
                    let address = destination + row * 32;
                    if let Some(word) = vram.get_mut(address) {
                        *word = u16::from_le_bytes([pair[0], pair[1]]);
                    } else if self.bounds == TransferBounds::Strict {
                        // Earlier rows have already been written in the C/Rust
                        // compatibility path; do not make this transactional.
                        return Err(NmiError::InvalidVramTarget(address));
                    }
                }
            }
            TransferLayout::LowBytes => {
                // Preserve the strict path's all-or-nothing range validation.
                if self.bounds == TransferBounds::Strict {
                    let words = vram
                        .get_mut(destination..destination.saturating_add(self.bytes.len()))
                        .ok_or(NmiError::InvalidVramTarget(destination))?;
                    for (word, &value) in words.iter_mut().zip(self.bytes) {
                        *word = (*word & 0xff00) | u16::from(value);
                    }
                } else {
                    for (offset, &value) in self.bytes.iter().enumerate() {
                        if let Some(word) = vram.get_mut(destination + offset) {
                            *word = (*word & 0xff00) | u16::from(value);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Decoder for the legacy big-endian stripe stream. Header layout and cursor
/// arithmetic live here rather than in the NMI orchestration code. Decoding
/// stays lazy: a malformed later packet does not suppress earlier transfers.
pub struct StripeStream<'a> {
    data: &'a [u8],
    cursor: usize,
}

pub struct Stripe<'a> {
    destination: VramWordAddress,
    layout: TransferLayout,
    payload: StripePayload<'a>,
}

enum StripePayload<'a> {
    Bytes(&'a [u8]),
    RepeatedWord { pair: [u8; 2], words: usize },
}

impl<'a> StripeStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn next_stripe(&mut self) -> Result<Option<Stripe<'a>>, NmiError> {
        if self.data.get(self.cursor).ok_or(NmiError::TruncatedData)? & 0x80 != 0 {
            return Ok(None);
        }
        let header = self
            .data
            .get(self.cursor..self.cursor + 4)
            .ok_or(NmiError::TruncatedData)?;
        let destination =
            VramWordAddress::new(usize::from(u16::from_be_bytes([header[0], header[1]])));
        let layout = if header[2] & 0x80 != 0 {
            TransferLayout::ColumnWords
        } else {
            TransferLayout::PackedBytes
        };
        let fill = header[2] & 0x40 != 0;
        let len = usize::from(u16::from_be_bytes([header[2], header[3]]) & 0x3fff) + 1;
        self.cursor += 4;
        let payload = if fill {
            let pair = self
                .data
                .get(self.cursor..self.cursor + 2)
                .ok_or(NmiError::TruncatedData)?;
            self.cursor += 2;
            // The count is in bytes; an odd fill still emits a complete pair.
            StripePayload::RepeatedWord {
                pair: [pair[0], pair[1]],
                words: (len + 1) >> 1,
            }
        } else {
            let bytes = self
                .data
                .get(self.cursor..self.cursor + len)
                .ok_or(NmiError::TruncatedData)?;
            self.cursor += len;
            StripePayload::Bytes(bytes)
        };
        Ok(Some(Stripe {
            destination,
            layout,
            payload,
        }))
    }
}

impl Stripe<'_> {
    pub fn apply(self, vram: &mut [u16]) -> Result<(), NmiError> {
        let repeated;
        let bytes = match self.payload {
            StripePayload::Bytes(bytes) => bytes,
            StripePayload::RepeatedWord { pair, words } => {
                repeated = pair.repeat(words);
                &repeated
            }
        };
        VramUpload {
            destination: self.destination,
            layout: self.layout,
            bounds: TransferBounds::Strict,
            bytes,
        }
        .apply(vram)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upload(bytes: &[u8], layout: TransferLayout, bounds: TransferBounds) -> VramUpload<'_> {
        VramUpload {
            destination: VramWordAddress::new(1),
            layout,
            bounds,
            bytes,
        }
    }

    #[test]
    fn byte_lanes_and_unpaired_low_byte_are_preserved() {
        let mut words = [0xabcd; 4];
        upload(
            &[1, 2, 3],
            TransferLayout::PackedBytes,
            TransferBounds::Strict,
        )
        .apply(&mut words)
        .unwrap();
        assert_eq!(words, [0xabcd, 0x0201, 0xab03, 0xabcd]);
        upload(&[4, 5], TransferLayout::LowBytes, TransferBounds::Strict)
            .apply(&mut words)
            .unwrap();
        assert_eq!(words, [0xabcd, 0x0204, 0xab05, 0xabcd]);
    }

    #[test]
    fn strict_columns_preserve_prefix_writes_but_reject_odd_data_first() {
        let mut words = [0xabcd; 34];
        assert_eq!(
            upload(
                &[1, 2, 3],
                TransferLayout::ColumnWords,
                TransferBounds::Strict
            )
            .apply(&mut words),
            Err(NmiError::TruncatedData)
        );
        assert_eq!(words, [0xabcd; 34]);
        assert_eq!(
            upload(
                &[1, 2, 3, 4, 5, 6],
                TransferLayout::ColumnWords,
                TransferBounds::Strict
            )
            .apply(&mut words),
            Err(NmiError::InvalidVramTarget(65))
        );
        assert_eq!(words[1], 0x0201);
        assert_eq!(words[33], 0x0403);
    }

    #[test]
    fn clipped_transfers_do_not_wrap_and_strict_bytes_do_not_partially_write() {
        let mut words = [0xabcd; 2];
        let bytes = [1, 2, 3, 4];
        let transfer = upload(&bytes, TransferLayout::PackedBytes, TransferBounds::Strict);
        assert_eq!(
            transfer.apply(&mut words),
            Err(NmiError::InvalidVramTarget(1))
        );
        assert_eq!(words, [0xabcd; 2]);
        VramUpload {
            bounds: TransferBounds::ClipOutsideVram,
            ..transfer
        }
        .apply(&mut words)
        .unwrap();
        assert_eq!(words, [0xabcd, 0x0201]);
        VramUpload {
            layout: TransferLayout::ColumnWords,
            bounds: TransferBounds::ClipOutsideVram,
            ..transfer
        }
        .apply(&mut words)
        .unwrap();
        assert_eq!(words, [0xabcd, 0x0201]);
    }

    #[test]
    fn stripe_cursor_preserves_odd_fill_rounding_and_late_failure_visibility() {
        // One-byte fill still repeats a complete word, then a vertical copy,
        // then a deliberately incomplete header.
        let bytes = [0, 1, 0x40, 0, 0x34, 0x12, 0, 2, 0x80, 1, 0x78, 0x56, 0, 0];
        let mut stream = StripeStream::new(&bytes);
        let mut words = [0xabcd; 40];
        stream
            .next_stripe()
            .unwrap()
            .unwrap()
            .apply(&mut words)
            .unwrap();
        assert_eq!(words[1], 0x1234);
        stream
            .next_stripe()
            .unwrap()
            .unwrap()
            .apply(&mut words)
            .unwrap();
        assert_eq!(words[2], 0x5678);
        assert!(matches!(stream.next_stripe(), Err(NmiError::TruncatedData)));
        assert_eq!(words[1], 0x1234);
        assert_eq!(words[2], 0x5678);
        let mut end = StripeStream::new(&[0xff]);
        assert!(end.next_stripe().unwrap().is_none());
    }

    #[test]
    fn upload_packets_keep_layout_units_and_stop_at_a_two_byte_marker() {
        let bytes = [
            1, 0, 0x80, 3, 0x34, 0x12, 0x56, 2, 0, 0x81, 4, 0x78, 0x56, 0xbc, 0x9a, 0xff, 0xff,
        ];
        let mut stream = UploadPacketStream::new(&bytes);
        let mut words = [0xabcd; 40];
        while let Some(upload) = stream.next_upload().unwrap() {
            upload.apply(&mut words).unwrap();
        }
        assert_eq!(words[1], 0x1234);
        assert_eq!(words[2], 0x5678);
        assert_eq!(words[34], 0x9abc);
        assert_eq!(words[3], 0xabcd);
        let mut invalid = UploadPacketStream::new(&[0, 0, 0x82, 0]);
        assert!(matches!(
            invalid.next_upload(),
            Err(NmiError::UnsupportedVmain(0x82))
        ));
    }

    #[test]
    fn scroll_packets_truncate_odd_lengths_and_clip_columns_without_wrapping() {
        let bytes = [5, 0x80, 1, 0, 0x34, 0x12, 0x78, 0x56, 0, 0x80];
        let mut stream = ScrollUploadStream::new(&bytes);
        let mut words = [0xabcd; 2];
        while let Some(upload) = stream.next_upload() {
            upload.apply(&mut words).unwrap();
        }
        assert_eq!(words, [0xabcd, 0x1234]);
    }
}
