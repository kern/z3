//! The legacy recorder container, separate from replay scheduling and snapshots.
//!
//! Snapshot payloads are deliberately opaque here. In particular, this codec
//! accepts zero-sized and non-machine-sized snapshots and ignores file tails.

use super::{RecordedInputs, StateRecorder};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateFileError {
    Truncated,
    UnsupportedVersion(u32),
    SizeOverflow,
}

#[derive(Clone, Copy)]
enum LoadMode {
    Resume,
    ReplayFromStart,
}

impl From<bool> for LoadMode {
    fn from(replay_from_start: bool) -> Self {
        if replay_from_start {
            Self::ReplayFromStart
        } else {
            Self::Resume
        }
    }
}

/// Word order is part of the on-disk C ABI. Keep raw words losslessly until
/// adapting them to the recorder (whose input mask, for example, is only u16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Header {
    version: u32,
    total_frames: u32,
    log_size: u32,
    last_inputs: u32,
    frames_since_last: u32,
    base_and_position: u32,
    snapshot_size: u32,
    replay_frame_counter: u32,
}

impl Header {
    const BYTE_LEN: usize = 32;

    fn decode(bytes: &[u8]) -> Result<Self, StateFileError> {
        if bytes.len() < Self::BYTE_LEN {
            return Err(StateFileError::Truncated);
        }
        let mut words = bytes[..Self::BYTE_LEN]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("four-byte chunk")));
        let header = Self {
            version: words.next().unwrap(),
            total_frames: words.next().unwrap(),
            log_size: words.next().unwrap(),
            last_inputs: words.next().unwrap(),
            frames_since_last: words.next().unwrap(),
            base_and_position: words.next().unwrap(),
            snapshot_size: words.next().unwrap(),
            replay_frame_counter: words.next().unwrap(),
        };
        if header.version != 1 {
            return Err(StateFileError::UnsupportedVersion(header.version));
        }
        Ok(header)
    }

    fn append_to(self, bytes: &mut Vec<u8>) {
        for word in [
            self.version,
            self.total_frames,
            self.log_size,
            self.last_inputs,
            self.frames_since_last,
            self.base_and_position,
            self.snapshot_size,
            self.replay_frame_counter,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
    }

    fn has_base_snapshot(self) -> bool {
        self.base_and_position & 1 != 0
    }
    fn resume_position(self) -> usize {
        (self.base_and_position >> 1) as usize
    }
}

/// Borrowed, permissive view of all sections, including bytes the legacy load
/// adapter ignores. Parsing always validates the current snapshot, even when
/// the caller later chooses to replay from the base snapshot.
struct RecordingFileView<'a> {
    header: Header,
    log: &'a [u8],
    base_snapshot: &'a [u8],
    current_snapshot: &'a [u8],
    tail: &'a [u8],
}

impl<'a> RecordingFileView<'a> {
    fn decode(bytes: &'a [u8]) -> Result<Self, StateFileError> {
        let header = Header::decode(bytes)?;
        let log_size =
            usize::try_from(header.log_size).map_err(|_| StateFileError::SizeOverflow)?;
        let snapshot_size =
            usize::try_from(header.snapshot_size).map_err(|_| StateFileError::SizeOverflow)?;
        let base_size = if header.has_base_snapshot() {
            snapshot_size
        } else {
            0
        };
        let required = Header::BYTE_LEN
            .checked_add(log_size)
            .and_then(|size| size.checked_add(base_size))
            .and_then(|size| size.checked_add(snapshot_size))
            .ok_or(StateFileError::SizeOverflow)?;
        if bytes.len() < required {
            return Err(StateFileError::Truncated);
        }
        let mut position = Header::BYTE_LEN;
        let log = &bytes[position..position + log_size];
        position += log_size;
        let base_snapshot = &bytes[position..position + base_size];
        position += base_size;
        let current_snapshot = &bytes[position..position + snapshot_size];
        Ok(Self {
            header,
            log,
            base_snapshot,
            current_snapshot,
            tail: &bytes[required..],
        })
    }
}

fn encode_sections(header: Header, log: &[u8], base: &[u8], current: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(Header::BYTE_LEN + log.len() + base.len() + current.len());
    header.append_to(&mut bytes);
    bytes.extend_from_slice(log);
    bytes.extend_from_slice(base);
    bytes.extend_from_slice(current);
    bytes
}

impl StateRecorder {
    /// Encodes the exact eight-word state-recorder header followed by the key
    /// log, optional replay base snapshot, and current machine snapshot.
    #[must_use]
    pub fn encode_state_file(&self, current_snapshot: &[u8]) -> Vec<u8> {
        assert!(
            self.base_snapshot.is_empty() || self.base_snapshot.len() == current_snapshot.len()
        );
        let mut header = Header {
            version: 1,
            total_frames: self.tape.total_frames,
            log_size: self
                .tape
                .bytes
                .len()
                .try_into()
                .expect("key log fits C u32"),
            last_inputs: u32::from(self.tape.last_inputs),
            frames_since_last: self.tape.frames_since_last,
            base_and_position: u32::from(!self.base_snapshot.is_empty()),
            snapshot_size: current_snapshot
                .len()
                .try_into()
                .expect("snapshot fits C u32"),
            replay_frame_counter: 0,
        };
        if self.tape.replay.active {
            header.base_and_position |= (self.tape.replay.last_complete.0 as u32) << 1;
            header.replay_frame_counter = self.tape.replay.frame_counter;
        }
        encode_sections(
            header,
            &self.tape.bytes,
            &self.base_snapshot,
            current_snapshot,
        )
    }

    /// Decodes a C-compatible recorder file and returns the machine snapshot
    /// which the caller must install before advancing another frame.
    pub fn decode_state_file(
        bytes: &[u8],
        replay_from_start: bool,
    ) -> Result<(Self, Vec<u8>), StateFileError> {
        Self::decode_recording(bytes, replay_from_start.into())
    }

    fn decode_recording(bytes: &[u8], mode: LoadMode) -> Result<(Self, Vec<u8>), StateFileError> {
        let file = RecordingFileView::decode(bytes)?;
        let header = file.header;
        let log = file.log.to_vec();
        let base_snapshot = file.base_snapshot.to_vec();
        let current_snapshot = file.current_snapshot.to_vec();
        let _ignored_trailing_bytes = file.tail;
        let mut recorder = Self {
            base_snapshot,
            tape: RecordedInputs {
                last_inputs: header.last_inputs as u16,
                frames_since_last: header.frames_since_last,
                total_frames: header.total_frames,
                bytes: log,
                ..RecordedInputs::default()
            },
        };
        let snapshot_to_load = match mode {
            LoadMode::ReplayFromStart => {
                let snapshot = recorder.base_snapshot.clone();
                recorder.start_replay();
                snapshot
            }
            LoadMode::Resume => {
                recorder.tape.replay.position.0 = header.resume_position();
                recorder.tape.replay.last_complete.0 = recorder.tape.replay.position.0;
                recorder.tape.replay.frame_counter = header.replay_frame_counter;
                recorder.tape.replay.active = recorder.tape.replay.frame_counter != 0;
                current_snapshot
            }
        };
        Ok((recorder, snapshot_to_load))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent word-array fixture: deliberately not encoded through Header.
    fn fixture(words: [u32; 8], payload: &[u8]) -> Vec<u8> {
        words
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .chain(payload.iter().copied())
            .collect()
    }

    #[test]
    fn borrowed_sections_preserve_raw_words_opaque_bytes_and_tail() {
        let words = [
            1, 0x12345678, 2, 0xabcd1234, 0xfedcba98, 0x76543211, 3, 0x87654321,
        ];
        let bytes = fixture(words, &[0xff, 0xc0, 1, 0, 255, 9, 8, 7, 0x55, 0xaa]);
        let view = RecordingFileView::decode(&bytes).unwrap();
        assert_eq!(view.log, [0xff, 0xc0]);
        assert_eq!(view.base_snapshot, [1, 0, 255]);
        assert_eq!(view.current_snapshot, [9, 8, 7]);
        assert_eq!(view.tail, [0x55, 0xaa]);
        assert_eq!(view.log.as_ptr(), bytes[32..].as_ptr());
        assert_eq!(view.base_snapshot.as_ptr(), bytes[34..].as_ptr());
        assert_eq!(view.current_snapshot.as_ptr(), bytes[37..].as_ptr());
        let mut encoded_header = Vec::new();
        view.header.append_to(&mut encoded_header);
        assert_eq!(encoded_header, bytes[..32]);
        let (recorder, snapshot) = StateRecorder::decode_state_file(&bytes, false).unwrap();
        assert_eq!(snapshot, [9, 8, 7]);
        assert_eq!(recorder.tape.last_inputs, 0x1234);
        assert_eq!(recorder.tape.replay.position.0, (words[5] >> 1) as usize);
        assert_eq!(recorder.tape.replay.frame_counter, words[7]);
        assert!(recorder.tape.replay.active);
        assert_eq!(recorder.tape.replay.next_command_at, 0);
        assert_eq!(recorder.tape.replay.command.0, 0);
        let mut canonical_words = words;
        canonical_words[3] = 0x1234;
        assert_eq!(
            recorder.encode_state_file(&snapshot),
            fixture(canonical_words, &[0xff, 0xc0, 1, 0, 255, 9, 8, 7]),
            "recorder re-encoding narrows inputs and discards the tail"
        );
    }

    #[test]
    fn every_payload_prefix_is_truncated_in_both_load_modes() {
        let bytes = fixture([1, 0, 2, 0, 0, 1, 3, 0], &[1, 2, 3, 4, 5, 6, 7, 8]);
        for mode in [false, true] {
            for length in 0..bytes.len() {
                assert_eq!(
                    StateRecorder::decode_state_file(&bytes[..length], mode),
                    Err(StateFileError::Truncated),
                    "length={length}, replay={mode}"
                );
            }
            assert!(StateRecorder::decode_state_file(&bytes, mode).is_ok());
        }
    }

    #[test]
    fn header_length_then_version_then_payload_is_the_error_precedence() {
        let bytes = fixture([0x87654321, 0, u32::MAX, 0, 0, 1, u32::MAX, 0], &[]);
        for mode in [false, true] {
            for length in 0..32 {
                assert_eq!(
                    StateRecorder::decode_state_file(&bytes[..length], mode),
                    Err(StateFileError::Truncated)
                );
            }
            assert_eq!(
                StateRecorder::decode_state_file(&bytes, mode),
                Err(StateFileError::UnsupportedVersion(0x87654321))
            );
        }
        let bytes = fixture([1, 0, u32::MAX, 0, 0, 1, u32::MAX, 0], &[]);
        let expected = if usize::BITS == 32 {
            StateFileError::SizeOverflow
        } else {
            StateFileError::Truncated
        };
        assert_eq!(
            StateRecorder::decode_state_file(&bytes, false),
            Err(expected)
        );
    }

    #[test]
    fn opaque_sizes_zero_base_flag_and_out_of_range_cursors_remain_accepted() {
        for size in [0u32, 1, 3, 257] {
            for base in [false, true] {
                let current: Vec<_> = (0..size).map(|i| i as u8).collect();
                let mut payload = Vec::new();
                if base {
                    payload.extend_from_slice(&current);
                }
                payload.extend_from_slice(&current);
                payload.extend_from_slice(&[0xde, 0xad]);
                let bytes = fixture(
                    [
                        1,
                        1,
                        0,
                        u32::MAX,
                        99,
                        0xfffffffe | u32::from(base),
                        size,
                        50,
                    ],
                    &payload,
                );
                let (resume, snapshot) = StateRecorder::decode_state_file(&bytes, false).unwrap();
                assert_eq!(snapshot, current);
                assert_eq!(resume.tape.replay.position.0, 0x7fffffff);
                assert_eq!(resume.tape.last_inputs, u16::MAX);
                assert_eq!(
                    resume.encode_state_file(&snapshot),
                    fixture(
                        [
                            1,
                            1,
                            0,
                            0xffff,
                            99,
                            0xfffffffe | u32::from(base && size != 0),
                            size,
                            50,
                        ],
                        &payload[..payload.len() - 2],
                    ),
                    "zero-sized bases lose the base-present bit on re-encoding"
                );
                let (start, snapshot) = StateRecorder::decode_state_file(&bytes, true).unwrap();
                assert_eq!(snapshot, if base { current.clone() } else { Vec::new() });
                assert_eq!(start.tape.last_inputs, 0);
                assert_eq!(start.tape.frames_since_last, 0);
                assert_eq!(start.tape.replay.position.0, 0);
                assert_eq!(start.tape.replay.frame_counter, 0);
                assert!(start.tape.replay.active);
            }
        }
    }

    #[test]
    fn resume_mode_depends_only_on_saved_frame_and_encoding_uses_last_complete() {
        let mut recorder = StateRecorder {
            base_snapshot: vec![1, 2, 3],
            ..StateRecorder::default()
        };
        recorder.tape.last_inputs = 0x1234;
        recorder.tape.total_frames = 90;
        recorder.tape.frames_since_last = 7;
        recorder.tape.bytes = vec![0x81, 0xff];
        recorder.tape.replay.position.0 = 30;
        recorder.tape.replay.last_complete.0 = 4;
        recorder.tape.replay.frame_counter = 8;
        recorder.tape.replay.next_command_at = 50;
        recorder.tape.replay.command.0 = 0xc1;
        recorder.tape.replay.active = true;
        let bytes = recorder.encode_state_file(&[4, 5, 6]);
        assert_eq!(
            bytes,
            fixture(
                [1, 90, 2, 0x1234, 7, 9, 3, 8],
                &[0x81, 0xff, 1, 2, 3, 4, 5, 6]
            )
        );
        recorder.tape.replay.last_complete.0 = usize::MAX;
        let header = Header::decode(&recorder.encode_state_file(&[4, 5, 6])).unwrap();
        assert_eq!(header.base_and_position, u32::MAX);
        recorder.tape.replay.active = false;
        assert_eq!(
            recorder.encode_state_file(&[4, 5, 6]),
            fixture(
                [1, 90, 2, 0x1234, 7, 1, 3, 0],
                &[0x81, 0xff, 1, 2, 3, 4, 5, 6]
            )
        );
        let bytes = fixture([1, 0, 0, 0, 0, 0xfffffffe, 0, 0], &[]);
        let (resume, _) = StateRecorder::decode_state_file(&bytes, false).unwrap();
        assert!(!resume.tape.replay.active);
        assert_eq!(resume.tape.replay.position.0, 0x7fffffff);
    }

    #[test]
    #[should_panic]
    fn mismatched_base_and_current_snapshot_sizes_still_panic() {
        let _ = StateRecorder {
            base_snapshot: vec![1],
            ..StateRecorder::default()
        }
        .encode_state_file(&[]);
    }
}
