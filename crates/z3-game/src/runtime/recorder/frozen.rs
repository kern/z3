#![allow(dead_code)]
// Frozen from 02d83bb runtime.rs, before owned tape migration.
use crate::ram::Ram;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrozenRecorder {
    pub last_inputs: u16,
    pub frames_since_last: u32,
    pub total_frames: u32,
    pub log: Vec<u8>,
    pub base_snapshot: Vec<u8>,
    pub(super) replay_position: usize,
    pub(super) replay_position_last_complete: usize,
    pub(super) replay_frame_counter: u32,
    pub(super) replay_next_command_at: u32,
    pub(super) replay_command: u8,
    pub(super) replay_mode: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayError {
    TruncatedLog,
    InvalidCommand(u8),
}

impl FrozenRecorder {
    fn append_variable_length(&mut self, mut value: u32) {
        while value >= 255 {
            self.log.push(255);
            value -= 255;
        }
        self.log.push(value as u8);
    }

    fn record_command(&mut self, command: u8) {
        let frames = self.frames_since_last;
        self.frames_since_last = 0;
        let inline_limit = if command < 0xc0 { 0x0f } else { 1 };
        self.log.push(command | frames.min(inline_limit) as u8);
        if frames >= inline_limit {
            self.append_variable_length(frames - inline_limit);
        }
    }

    pub fn record(&mut self, inputs: u16) {
        let difference = inputs ^ self.last_inputs;
        if difference != 0 {
            self.last_inputs = inputs;
            for bit in 0..12 {
                if difference >> bit & 1 != 0 {
                    self.record_command(bit << 4);
                }
            }
        }
        self.frames_since_last = self.frames_since_last.wrapping_add(1);
        self.total_frames = self.total_frames.wrapping_add(1);
    }

    pub fn record_patch(&mut self, address: u32, bytes: &[u8]) {
        assert!(address < 0x20_000);
        assert!(!bytes.is_empty());
        let length_minus_one = bytes.len() - 1;
        let length_code = length_minus_one.min(3);
        self.record_command(
            0xc0 | u8::from(address & 0x1_0000 != 0) << 1 | (length_code as u8) << 2,
        );
        if length_code == 3 {
            self.append_variable_length((length_minus_one - 3) as u32);
        }
        self.log.push((address >> 8) as u8);
        self.log.push(address as u8);
        self.log.extend_from_slice(bytes);
    }

    /// Starts playback of the current key log. Loading the optional base
    /// snapshot is kept at the runtime boundary, matching the C port's order:
    /// install the snapshot (or reset), then call this method.
    pub fn start_replay(&mut self) {
        self.frames_since_last = 0;
        self.last_inputs = 0;
        self.replay_position = 0;
        self.replay_position_last_complete = 0;
        self.replay_frame_counter = 0;
        self.replay_next_command_at = 0;
        self.replay_command = 0;
        self.replay_mode = true;
    }

    #[must_use]
    pub fn is_replaying(&self) -> bool {
        self.replay_mode
    }

    fn replay_byte(&self, position: &mut usize) -> Result<u8, ReplayError> {
        let value = self
            .log
            .get(*position)
            .copied()
            .ok_or(ReplayError::TruncatedLog)?;
        *position += 1;
        Ok(value)
    }

    fn replay_variable_length(&self, position: &mut usize) -> Result<u32, ReplayError> {
        let mut value = 0u32;
        loop {
            let byte = self.replay_byte(position)?;
            value = value.wrapping_add(u32::from(byte));
            if byte != 255 {
                return Ok(value);
            }
        }
    }

    pub fn read_next_replay_state(&mut self, ram: &mut Ram<'_>) -> Result<u16, ReplayError> {
        assert!(self.replay_mode);
        while self.frames_since_last >= self.replay_next_command_at {
            let mut position = self.replay_position;
            if position != self.replay_position_last_complete {
                self.frames_since_last = 0;
                if self.replay_command < 0xc0 {
                    self.last_inputs ^= 1 << (self.replay_command >> 4);
                } else if self.replay_command < 0xd0 {
                    let mut count = 1 + usize::from((self.replay_command >> 2) & 3);
                    if count == 4 {
                        count += self.replay_variable_length(&mut position)? as usize;
                    }
                    let mut address = u32::from((self.replay_command >> 1) & 1) << 16;
                    address |= u32::from(self.replay_byte(&mut position)?) << 8;
                    address |= u32::from(self.replay_byte(&mut position)?);
                    for offset in 0..count {
                        let value = self.replay_byte(&mut position)?;
                        ram.write_u8((address as usize + offset) & 0x1_ffff, value);
                    }
                } else {
                    return Err(ReplayError::InvalidCommand(self.replay_command));
                }
            }
            self.replay_position_last_complete = position;
            if position >= self.log.len() {
                self.replay_position = position;
                self.replay_next_command_at = u32::MAX;
                break;
            }

            let command = self.replay_byte(&mut position)?;
            let mask = if command < 0xc0 { 0x0f } else { 1 };
            let mut frames = u32::from(command & mask);
            if frames == u32::from(mask) {
                frames = frames.wrapping_add(self.replay_variable_length(&mut position)?);
            }
            self.replay_next_command_at = frames;
            self.replay_command = command;
            self.replay_position = position;
        }
        self.frames_since_last = self.frames_since_last.wrapping_add(1);
        self.replay_frame_counter = self.replay_frame_counter.wrapping_add(1);
        if self.replay_frame_counter >= self.total_frames {
            self.replay_mode = false;
        }
        Ok(self.last_inputs)
    }

    pub fn stop_replay(&mut self) {
        if !self.replay_mode {
            return;
        }
        self.replay_mode = false;
        self.total_frames = self.replay_frame_counter;
        self.log.truncate(self.replay_position_last_complete);
    }

    /// Re-bases the key log on the supplied current machine snapshot. Active
    /// buttons become zero-time toggle commands, exactly as in the C recorder.
    pub fn clear_key_log(&mut self, current_snapshot: &[u8]) {
        self.base_snapshot.clear();
        self.base_snapshot.extend_from_slice(current_snapshot);
        let old_log = core::mem::take(&mut self.log);
        let old_frames_since_last = self.frames_since_last;
        self.frames_since_last = 0;
        if self.last_inputs != 0 {
            for bit in 0..12 {
                if self.last_inputs >> bit & 1 != 0 {
                    self.record_command(bit << 4);
                }
            }
        }
        if self.replay_mode {
            if self.replay_next_command_at != u32::MAX {
                self.replay_next_command_at = self
                    .replay_next_command_at
                    .wrapping_sub(old_frames_since_last);
                self.frames_since_last = self.replay_next_command_at;
                self.replay_position_last_complete = self.log.len();
                self.record_command(self.replay_command);
                let old_position = self.replay_position;
                self.replay_position = self.log.len();
                self.log.extend_from_slice(&old_log[old_position..]);
            }
            self.total_frames = self.total_frames.wrapping_sub(self.replay_frame_counter);
            self.replay_frame_counter = 0;
        } else {
            self.total_frames = 0;
        }
        self.frames_since_last = 0;
    }
}
