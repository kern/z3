//! Owned input tape and its lazy playback scheduler.

use crate::ram::Ram;
mod recording_file;
mod replay_log;
pub use recording_file::StateFileError;
use replay_log::{CommandCode, CommandKind, LogPosition, LogReader, LogWriter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayError {
    TruncatedLog,
    InvalidCommand(u8),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ReplayState {
    position: LogPosition,
    last_complete: LogPosition,
    frame_counter: u32,
    next_command_at: u32,
    command: CommandCode,
    active: bool,
}

/// The tape is the single owner of recorded bytes, input state, time, and
/// playback cursors. No decoded event cache or mirrored recorder fields exist.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RecordedInputs {
    bytes: Vec<u8>,
    last_inputs: u16,
    frames_since_last: u32,
    total_frames: u32,
    replay: ReplayState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateRecorder {
    base_snapshot: Vec<u8>,
    tape: RecordedInputs,
}

impl RecordedInputs {
    fn record_command(&mut self, command: CommandCode) {
        let frames = self.frames_since_last;
        self.frames_since_last = 0;
        LogWriter::new(&mut self.bytes).header(command, frames);
    }

    fn record(&mut self, inputs: u16) {
        let difference = inputs ^ self.last_inputs;
        if difference != 0 {
            self.last_inputs = inputs;
            for bit in 0..12 {
                if difference >> bit & 1 != 0 {
                    self.record_command(CommandCode(bit << 4));
                }
            }
        }
        self.frames_since_last = self.frames_since_last.wrapping_add(1);
        self.total_frames = self.total_frames.wrapping_add(1);
    }

    fn record_patch(&mut self, address: u32, bytes: &[u8]) {
        assert!(address < 0x20_000);
        assert!(!bytes.is_empty());
        let length_minus_one = bytes.len() - 1;
        let length_code = length_minus_one.min(3);
        self.record_command(CommandCode(
            0xc0 | u8::from(address & 0x1_0000 != 0) << 1 | (length_code as u8) << 2,
        ));
        LogWriter::new(&mut self.bytes).patch_payload(address, bytes, length_code);
    }

    fn start_replay(&mut self) {
        self.frames_since_last = 0;
        self.last_inputs = 0;
        self.replay = ReplayState {
            active: true,
            ..ReplayState::default()
        };
    }

    fn read_next(&mut self, ram: &mut Ram<'_>) -> Result<u16, ReplayError> {
        assert!(self.replay.active);
        while self.frames_since_last >= self.replay.next_command_at {
            let mut reader = LogReader::new(&self.bytes, self.replay.position);
            if reader.position() != self.replay.last_complete {
                self.frames_since_last = 0;
                match self.replay.command.kind() {
                    CommandKind::Toggle(bit) => self.last_inputs ^= 1 << bit,
                    CommandKind::Patch {
                        initial_count,
                        bank,
                    } => {
                        let patch = reader.patch(initial_count, bank)?;
                        for offset in 0..patch.count {
                            let value = reader.byte()?;
                            ram.write_u8((patch.address as usize + offset) & 0x1_ffff, value);
                        }
                    }
                    CommandKind::Invalid(raw) => return Err(ReplayError::InvalidCommand(raw)),
                }
            }
            self.replay.last_complete = reader.position();
            if reader.at_end() {
                self.replay.position = reader.position();
                self.replay.next_command_at = u32::MAX;
                break;
            }
            // Header publication is atomic, but applying its payload is not:
            // truncated patches retain the already-written WRAM prefix.
            let pending = reader.header()?;
            self.replay.next_command_at = pending.delay;
            self.replay.command = pending.code;
            self.replay.position = pending.payload;
        }
        self.frames_since_last = self.frames_since_last.wrapping_add(1);
        self.replay.frame_counter = self.replay.frame_counter.wrapping_add(1);
        if self.replay.frame_counter >= self.total_frames {
            self.replay.active = false;
        }
        Ok(self.last_inputs)
    }

    fn stop_replay(&mut self) {
        if !self.replay.active {
            return;
        }
        self.replay.active = false;
        self.total_frames = self.replay.frame_counter;
        self.bytes.truncate(self.replay.last_complete.0);
    }

    fn clear_history(&mut self) {
        let old_log = core::mem::take(&mut self.bytes);
        let old_frames_since_last = self.frames_since_last;
        self.frames_since_last = 0;
        if self.last_inputs != 0 {
            for bit in 0..12 {
                if self.last_inputs >> bit & 1 != 0 {
                    self.record_command(CommandCode(bit << 4));
                }
            }
        }
        if self.replay.active {
            if self.replay.next_command_at != u32::MAX {
                self.replay.next_command_at = self
                    .replay
                    .next_command_at
                    .wrapping_sub(old_frames_since_last);
                self.frames_since_last = self.replay.next_command_at;
                self.replay.last_complete = LogPosition(self.bytes.len());
                self.record_command(self.replay.command);
                let old_position = self.replay.position;
                self.replay.position = LogPosition(self.bytes.len());
                self.bytes.extend_from_slice(&old_log[old_position.0..]);
            }
            self.total_frames = self.total_frames.wrapping_sub(self.replay.frame_counter);
            self.replay.frame_counter = 0;
        } else {
            self.total_frames = 0;
        }
        self.frames_since_last = 0;
    }
}

impl StateRecorder {
    #[must_use]
    pub fn last_inputs(&self) -> u16 {
        self.tape.last_inputs
    }
    #[must_use]
    pub fn frames_since_last(&self) -> u32 {
        self.tape.frames_since_last
    }
    #[must_use]
    pub fn total_frames(&self) -> u32 {
        self.tape.total_frames
    }
    #[must_use]
    pub fn log(&self) -> &[u8] {
        &self.tape.bytes
    }
    #[must_use]
    pub fn base_snapshot(&self) -> &[u8] {
        &self.base_snapshot
    }

    pub fn record(&mut self, inputs: u16) {
        self.tape.record(inputs);
    }
    pub fn record_patch(&mut self, address: u32, bytes: &[u8]) {
        self.tape.record_patch(address, bytes);
    }

    /// Starts playback after the caller installs the base snapshot (or resets).
    pub fn start_replay(&mut self) {
        self.tape.start_replay();
    }
    #[must_use]
    pub fn is_replaying(&self) -> bool {
        self.tape.replay.active
    }
    pub fn read_next_replay_state(&mut self, ram: &mut Ram<'_>) -> Result<u16, ReplayError> {
        self.tape.read_next(ram)
    }
    pub fn stop_replay(&mut self) {
        self.tape.stop_replay();
    }

    /// Re-bases input history on the supplied opaque machine snapshot.
    pub fn clear_key_log(&mut self, current_snapshot: &[u8]) {
        self.base_snapshot.clear();
        self.base_snapshot.extend_from_slice(current_snapshot);
        self.tape.clear_history();
    }
}

#[cfg(test)]
mod frozen;
#[cfg(test)]
mod replay_oracle_tests;
#[cfg(test)]
mod tests;
