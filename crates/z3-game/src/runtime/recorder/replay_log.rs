//! Lossless legacy input-tape instructions. Headers are decoded separately
//! from payloads: a scheduled patch must not touch RAM until it becomes due.

use super::ReplayError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct LogPosition(pub(super) usize);

/// Retain the entire byte, including inline delay bits. Clearing history
/// re-emits this byte with OR (not with replacement of its delay bits).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CommandCode(pub(super) u8);

pub(super) enum CommandKind {
    Toggle(u8),
    Patch { initial_count: usize, bank: u32 },
    Invalid(u8),
}

impl CommandCode {
    fn delay_mask(self) -> u8 {
        if self.0 < 0xc0 { 15 } else { 1 }
    }

    pub(super) fn kind(self) -> CommandKind {
        match self.0 {
            0x00..=0xbf => CommandKind::Toggle(self.0 >> 4),
            0xc0..=0xcf => CommandKind::Patch {
                initial_count: 1 + usize::from((self.0 >> 2) & 3),
                bank: u32::from((self.0 >> 1) & 1) << 16,
            },
            raw => CommandKind::Invalid(raw),
        }
    }
}

pub(super) struct PendingHeader {
    pub(super) code: CommandCode,
    pub(super) delay: u32,
    pub(super) payload: LogPosition,
}

/// A speculative cursor: the scheduler explicitly publishes it only at the
/// original commit boundaries. A failed read never advances this cursor.
pub(super) struct LogReader<'a> {
    bytes: &'a [u8],
    position: LogPosition,
}

impl<'a> LogReader<'a> {
    pub(super) fn new(bytes: &'a [u8], position: LogPosition) -> Self {
        Self { bytes, position }
    }
    pub(super) fn position(&self) -> LogPosition {
        self.position
    }
    pub(super) fn at_end(&self) -> bool {
        self.position.0 >= self.bytes.len()
    }

    pub(super) fn byte(&mut self) -> Result<u8, ReplayError> {
        let value = self
            .bytes
            .get(self.position.0)
            .copied()
            .ok_or(ReplayError::TruncatedLog)?;
        self.position.0 += 1;
        Ok(value)
    }

    pub(super) fn variable_length(&mut self) -> Result<u32, ReplayError> {
        let mut value = 0u32;
        loop {
            let byte = self.byte()?;
            value = value.wrapping_add(u32::from(byte));
            if byte != 255 {
                return Ok(value);
            }
        }
    }

    pub(super) fn header(&mut self) -> Result<PendingHeader, ReplayError> {
        let code = CommandCode(self.byte()?);
        let mask = code.delay_mask();
        let mut delay = u32::from(code.0 & mask);
        if delay == u32::from(mask) {
            delay = delay.wrapping_add(self.variable_length()?);
        }
        Ok(PendingHeader {
            code,
            delay,
            payload: self.position,
        })
    }

    pub(super) fn patch(
        &mut self,
        initial_count: usize,
        bank: u32,
    ) -> Result<PatchRun, ReplayError> {
        let mut count = initial_count;
        if count == 4 {
            count += self.variable_length()? as usize;
        }
        let mut address = bank;
        address |= u32::from(self.byte()?) << 8;
        address |= u32::from(self.byte()?);
        Ok(PatchRun { address, count })
    }
}

/// Metadata only: payload bytes remain lazy and are applied one at a time.
pub(super) struct PatchRun {
    pub(super) address: u32,
    pub(super) count: usize,
}

pub(super) struct LogWriter<'a> {
    bytes: &'a mut Vec<u8>,
}

impl<'a> LogWriter<'a> {
    pub(super) fn new(bytes: &'a mut Vec<u8>) -> Self {
        Self { bytes }
    }

    fn variable_length(&mut self, mut value: u32) {
        while value >= 255 {
            self.bytes.push(255);
            value -= 255;
        }
        self.bytes.push(value as u8);
    }

    pub(super) fn header(&mut self, code: CommandCode, frames: u32) {
        let inline_limit = u32::from(code.delay_mask());
        self.bytes.push(code.0 | frames.min(inline_limit) as u8);
        if frames >= inline_limit {
            self.variable_length(frames - inline_limit);
        }
    }

    pub(super) fn patch_payload(&mut self, address: u32, bytes: &[u8], length_code: usize) {
        if length_code == 3 {
            self.variable_length((bytes.len() - 1 - 3) as u32);
        }
        self.bytes.push((address >> 8) as u8);
        self.bytes.push(address as u8);
        self.bytes.extend_from_slice(bytes);
    }
}
