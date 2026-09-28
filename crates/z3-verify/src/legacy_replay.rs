//! Compatibility decoder for the C reference's legacy replay event stream.
//!
//! This implements the command layout used by `StateRecorder_RecordCmd` and
//! `StateRecorder_ReadNextReplayState` in `src/zelda_rtl.c`. Delay fields use a
//! continuation encoding where each `0xff` contributes 255 and the first byte
//! below 255 terminates the value.

use std::error::Error;
use std::fmt;

pub const WRAM_SIZE: usize = 0x20_000;
pub const INPUT_BITS: u8 = 12;

/// A decoded replay command and the frame on which it takes effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event<'a> {
    /// Zero-based frame relative to the replay's base snapshot.
    pub frame: u32,
    /// Delay encoded on this command, relative to the previous command.
    pub frames_since_previous: u32,
    pub command: Command<'a>,
}

/// Semantic content of one replay command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command<'a> {
    /// Toggle one of the 12 SNES input bits.
    ToggleInput { bit: u8, resulting_inputs: u16 },
    /// Replace bytes in WRAM, wrapping at the 128 KiB boundary like the C code.
    PatchWram { address: u32, bytes: &'a [u8] },
}

/// The reason a replay stream could not be decoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Truncated,
    UnterminatedVariableLength,
    NumericOverflow,
    UnsupportedCommand(u8),
}

/// A replay error with the byte offset at which it was detected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayError {
    pub offset: usize,
    pub kind: ErrorKind,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "legacy replay error at byte {}: ", self.offset)?;
        match self.kind {
            ErrorKind::Truncated => f.write_str("truncated command"),
            ErrorKind::UnterminatedVariableLength => {
                f.write_str("unterminated variable-length integer")
            }
            ErrorKind::NumericOverflow => f.write_str("numeric overflow"),
            ErrorKind::UnsupportedCommand(command) => {
                write!(f, "unsupported command 0x{command:02x}")
            }
        }
    }
}

impl Error for ReplayError {}

/// Incremental decoder for a legacy replay command stream.
#[derive(Clone, Debug)]
pub struct Decoder<'a> {
    source: &'a [u8],
    position: usize,
    frame: u32,
    inputs: u16,
}

impl<'a> Decoder<'a> {
    #[must_use]
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            position: 0,
            frame: 0,
            inputs: 0,
        }
    }

    #[must_use]
    pub fn position(&self) -> usize {
        self.position
    }

    #[must_use]
    pub fn inputs(&self) -> u16 {
        self.inputs
    }

    /// Decodes the next command. The decoder is exhausted after returning
    /// `Ok(None)`.
    pub fn next_event(&mut self) -> Result<Option<Event<'a>>, ReplayError> {
        if self.position == self.source.len() {
            return Ok(None);
        }

        let command_offset = self.position;
        let encoded = self.take_byte()?;
        let delay_mask = if encoded < 0xc0 { 0x0f } else { 0x01 };
        let mut delay = u32::from(encoded & delay_mask);
        if encoded & delay_mask == delay_mask {
            delay = delay
                .checked_add(self.variable_length()?)
                .ok_or_else(|| self.error(ErrorKind::NumericOverflow))?;
        }
        self.frame = self
            .frame
            .checked_add(delay)
            .ok_or_else(|| self.error(ErrorKind::NumericOverflow))?;

        let command = if encoded < 0xc0 {
            let bit = encoded >> 4;
            debug_assert!(bit < INPUT_BITS);
            self.inputs ^= 1_u16 << bit;
            Command::ToggleInput {
                bit,
                resulting_inputs: self.inputs,
            }
        } else if encoded < 0xd0 {
            let length_code = (encoded >> 2) & 3;
            let mut length = usize::from(length_code) + 1;
            if length_code == 3 {
                let extra = self.variable_length()?;
                length = length
                    .checked_add(
                        usize::try_from(extra)
                            .map_err(|_| self.error(ErrorKind::NumericOverflow))?,
                    )
                    .ok_or_else(|| self.error(ErrorKind::NumericOverflow))?;
            }
            let address_high = u32::from((encoded >> 1) & 1) << 16;
            let address =
                address_high | (u32::from(self.take_byte()?) << 8) | u32::from(self.take_byte()?);
            let bytes = self.take_bytes(length)?;
            Command::PatchWram { address, bytes }
        } else {
            return Err(ReplayError {
                offset: command_offset,
                kind: ErrorKind::UnsupportedCommand(encoded),
            });
        };

        Ok(Some(Event {
            frame: self.frame,
            frames_since_previous: delay,
            command,
        }))
    }

    fn variable_length(&mut self) -> Result<u32, ReplayError> {
        let start = self.position;
        let mut value = 0_u32;
        loop {
            let Some(&byte) = self.source.get(self.position) else {
                return Err(ReplayError {
                    offset: start,
                    kind: ErrorKind::UnterminatedVariableLength,
                });
            };
            self.position += 1;
            value = value
                .checked_add(u32::from(byte))
                .ok_or_else(|| self.error(ErrorKind::NumericOverflow))?;
            if byte != 0xff {
                return Ok(value);
            }
        }
    }

    fn take_byte(&mut self) -> Result<u8, ReplayError> {
        let byte = self
            .source
            .get(self.position)
            .copied()
            .ok_or_else(|| self.error(ErrorKind::Truncated))?;
        self.position += 1;
        Ok(byte)
    }

    fn take_bytes(&mut self, len: usize) -> Result<&'a [u8], ReplayError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or_else(|| self.error(ErrorKind::NumericOverflow))?;
        let bytes = self
            .source
            .get(self.position..end)
            .ok_or_else(|| self.error(ErrorKind::Truncated))?;
        self.position = end;
        Ok(bytes)
    }

    fn error(&self, kind: ErrorKind) -> ReplayError {
        ReplayError {
            offset: self.position,
            kind,
        }
    }
}

impl<'a> Iterator for Decoder<'a> {
    type Item = Result<Event<'a>, ReplayError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_event().transpose()
    }
}

/// Applies a decoded WRAM patch with the same 17-bit address wrapping as C.
/// Input events are ignored by this helper.
pub fn apply_wram_patch(event: &Event<'_>, wram: &mut [u8; WRAM_SIZE]) {
    let Command::PatchWram { address, bytes } = &event.command else {
        return;
    };
    for (index, &byte) in bytes.iter().enumerate() {
        wram[((*address as usize).wrapping_add(index)) & (WRAM_SIZE - 1)] = byte;
    }
}

/// Decodes an entire command stream, stopping at the first malformed command.
pub fn decode_all(source: &[u8]) -> Result<Vec<Event<'_>>, ReplayError> {
    Decoder::new(source).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_input_toggles_and_cumulative_frames() {
        // Bit 0 at frame 2, bit 11 at the same frame, then bit 0 after 16 frames.
        let events = decode_all(&[0x02, 0xb0, 0x0f, 0x01]).unwrap();
        assert_eq!(
            events,
            vec![
                Event {
                    frame: 2,
                    frames_since_previous: 2,
                    command: Command::ToggleInput {
                        bit: 0,
                        resulting_inputs: 1,
                    },
                },
                Event {
                    frame: 2,
                    frames_since_previous: 0,
                    command: Command::ToggleInput {
                        bit: 11,
                        resulting_inputs: 0x801,
                    },
                },
                Event {
                    frame: 18,
                    frames_since_previous: 16,
                    command: Command::ToggleInput {
                        bit: 0,
                        resulting_inputs: 0x800,
                    },
                },
            ]
        );
    }

    #[test]
    fn decodes_short_and_extended_wram_patches() {
        // 0xc8: three-byte patch, zero-frame delay, low WRAM bank.
        // 0xce: extended-length patch, zero-frame delay, high WRAM bank.
        let stream = [
            0xc8, 0x12, 0x34, 1, 2, 3, 0xce, 0x01, 0xff, 0xfe, 4, 5, 6, 7, 8,
        ];
        let events = decode_all(&stream).unwrap();
        assert_eq!(
            events[0].command,
            Command::PatchWram {
                address: 0x1234,
                bytes: &[1, 2, 3],
            }
        );
        assert_eq!(
            events[1].command,
            Command::PatchWram {
                address: 0x1fffe,
                bytes: &[4, 5, 6, 7, 8],
            }
        );
    }

    #[test]
    fn applies_patch_with_128k_wraparound() {
        let events = decode_all(&[0xc6, 0xff, 0xff, 0xaa, 0xbb]).unwrap();
        let mut wram = [0_u8; WRAM_SIZE];
        apply_wram_patch(&events[0], &mut wram);
        assert_eq!(wram[0x1ffff], 0xaa);
        assert_eq!(wram[0], 0xbb);
    }

    #[test]
    fn rejects_unsupported_and_truncated_commands() {
        assert_eq!(
            decode_all(&[0xd0]),
            Err(ReplayError {
                offset: 0,
                kind: ErrorKind::UnsupportedCommand(0xd0),
            })
        );
        assert!(matches!(
            decode_all(&[0xc8, 0x12]),
            Err(ReplayError {
                kind: ErrorKind::Truncated,
                ..
            })
        ));
        assert!(matches!(
            decode_all(&[0x0f, 0xff]),
            Err(ReplayError {
                kind: ErrorKind::UnterminatedVariableLength,
                ..
            })
        ));
    }
}
