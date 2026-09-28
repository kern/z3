//! Deterministic audio-control translation from `src/audio.c`.
//!
//! File-backed MSU decoding is a host concern; the game-visible track
//! remapping, SPC-port queue, and sample mixing live here so their ordering and
//! arithmetic can be tested independently of an audio callback.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::spc_player::SpcPlayer;

pub const MSU_ENABLED_MSU: u8 = 1;
pub const MSU_ENABLED_DELUXE: u8 = 2;
pub const MSU_ENABLED_OPUZ: u8 = 4;

pub const MSU_STATE_IDLE: u8 = 0;
pub const MSU_STATE_FINISHED_PLAYING: u8 = 1;
pub const MSU_STATE_RESUMING: u8 = 2;
pub const MSU_STATE_PLAYING: u8 = 3;

/// Byte layout copied into WRAM by `ZeldaSaveMusicStateToRam_Locked`.
///
/// The C structure is eight-byte aligned: its meaningful fields occupy bytes
/// 0..26 and six bytes of tail padding bring `sizeof(MsuPlayerResumeInfo)` to
/// 32.  Save states contain those 32 bytes verbatim, so this explicit codec is
/// preferable to depending on Rust's structure layout.
pub const MSU_RESUME_INFO_SIZE: usize = 32;
pub const MSU_RESUME_INFO_OFFSET: usize = 0x1db60;
pub const MSU_RESUME_INFO_ALT_OFFSET: usize = 0x1db20;
pub const MSU_VOLUME_OFFSET: usize = 0x654;
pub const MUSIC_UNK1_OFFSET: usize = 0x130;
pub const LAST_MUSIC_CONTROL_OFFSET: usize = 0x133;
pub const APUI00_MIRROR_OFFSET: usize = 0x648;

pub const VOLUME_TRANSITION_TARGET: [u8; 4] = [0, 64, 255, 255];
pub const VOLUME_TRANSITION_STEP: [u8; 4] = [7, 3, 3, 24];

pub const MSU_TRACKS_WITH_REPEAT: [u8; 48] = [
    1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1,
    1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

const IS_MUSIC_OW_OR_DUNGEON: [u8; 32] = [
    0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 2, 2, 2, 0, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 0, 0,
];

const MSU_DELUXE_OW_SONGS: [u8; 160] = [
    37, 37, 42, 38, 38, 38, 38, 39, 37, 37, 42, 38, 38, 38, 38, 41, 42, 42, 42, 42, 42, 42, 40, 40,
    43, 43, 42, 47, 47, 42, 45, 45, 43, 43, 43, 47, 47, 42, 45, 45, 112, 112, 48, 42, 42, 42, 42,
    45, 44, 44, 48, 48, 48, 46, 46, 46, 44, 44, 44, 48, 48, 46, 46, 46, 49, 49, 51, 50, 50, 50, 50,
    50, 49, 49, 51, 50, 50, 50, 50, 51, 51, 51, 51, 51, 51, 51, 51, 51, 52, 52, 51, 56, 56, 51, 54,
    54, 52, 52, 52, 56, 56, 51, 54, 54, 58, 52, 57, 51, 51, 51, 51, 54, 53, 53, 57, 57, 57, 55, 55,
    110, 53, 53, 57, 57, 57, 55, 55, 110, 37, 41, 41, 42, 42, 42, 42, 42, 42, 41, 41, 42, 42, 42,
    42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42,
];

const MSU_DELUXE_ENTRANCE_SONGS: [u8; 133] = [
    59, 59, 60, 61, 61, 61, 62, 62, 63, 64, 64, 64, 105, 65, 65, 66, 66, 62, 67, 62, 62, 68, 62,
    62, 68, 68, 62, 62, 62, 62, 62, 62, 62, 62, 62, 62, 69, 70, 71, 72, 73, 73, 73, 106, 102, 74,
    62, 62, 75, 75, 76, 77, 78, 68, 79, 80, 81, 62, 62, 62, 82, 75, 242, 59, 59, 76, 242, 242, 242,
    96, 83, 99, 59, 242, 242, 242, 84, 95, 104, 62, 85, 62, 62, 86, 242, 67, 103, 83, 83, 87, 76,
    88, 81, 98, 81, 88, 83, 89, 75, 97, 90, 91, 91, 100, 92, 93, 92, 242, 93, 107, 62, 75, 62, 67,
    62, 242, 242, 242, 73, 73, 73, 73, 102, 114, 81, 76, 62, 67, 62, 61, 94, 62, 103,
];

#[must_use]
pub fn remap_msu_deluxe_track(enabled: u8, track: u8, area: u8, entrance: u16) -> u8 {
    if enabled & MSU_ENABLED_DELUXE == 0 || usize::from(track) >= IS_MUSIC_OW_OR_DUNGEON.len() {
        return track;
    }
    match IS_MUSIC_OW_OR_DUNGEON[usize::from(track)] {
        1 => MSU_DELUXE_OW_SONGS
            .get(usize::from(area))
            .copied()
            .unwrap_or(track),
        2 => MSU_DELUXE_ENTRANCE_SONGS
            .get(usize::from(entrance))
            .copied()
            .filter(|&mapped| mapped != 242)
            .unwrap_or(track),
        _ => track,
    }
}

#[must_use]
pub fn is_playing_music_track(
    state: u8,
    enabled: u8,
    actual_track: u8,
    music_unk1: u8,
    requested: u8,
    area: u8,
    entrance: u16,
) -> bool {
    if state != MSU_STATE_IDLE && enabled & MSU_ENABLED_DELUXE != 0 {
        remap_msu_deluxe_track(enabled, requested, area, entrance) == actual_track
    } else {
        requested == music_unk1
    }
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn is_playing_music_track_with_bug(
    state: u8,
    enabled: u8,
    actual_track: u8,
    music_unk1: u8,
    last_music_control: u8,
    misc_bug_fixes: bool,
    requested: u8,
    area: u8,
    entrance: u16,
) -> bool {
    if state != MSU_STATE_IDLE && enabled & MSU_ENABLED_DELUXE != 0 {
        remap_msu_deluxe_track(enabled, requested, area, entrance) == actual_track
    } else {
        requested
            == if misc_bug_fixes {
                music_unk1
            } else {
                last_music_control
            }
    }
}

#[must_use]
pub fn entrance_music_track(state: u8, enabled: u8, entrance: u16, original_track: u8) -> u8 {
    if state != MSU_STATE_IDLE
        && enabled & MSU_ENABLED_DELUXE != 0
        && original_track == 242
        && MSU_DELUXE_ENTRANCE_SONGS
            .get(usize::from(entrance))
            .is_some_and(|&track| track != 242)
    {
        16
    } else {
        original_track
    }
}

#[must_use]
pub fn read_apui00(wram: &[u8]) -> u8 {
    wram[APUI00_MIRROR_OFFSET]
}

#[must_use]
pub fn zelda_read_apui00(wram: &[u8]) -> u8 {
    read_apui00(wram)
}

/// Sixteen-frame SPC command queue used to bridge the game and audio clocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApuWriteQueue {
    entries: [[u8; 4]; 16],
    current: [u8; 4],
    position: u8,
    count: u8,
    total_writes: u8,
}

impl ApuWriteQueue {
    pub fn write(&mut self, address: u32, value: u8) {
        self.current[(address & 3) as usize] = value;
    }

    pub fn push(&mut self) {
        self.entries[usize::from(self.position & 0x0f)] = self.current;
        self.position = self.position.wrapping_add(1);
        if self.count < 16 {
            self.count += 1;
        }
        self.total_writes = self.total_writes.wrapping_add(1);
    }

    pub fn pop_into(&mut self, input_ports: &mut [u8; 4]) {
        if self.count != 0 {
            let index = self.position.wrapping_sub(self.count) & 0x0f;
            *input_ports = self.entries[usize::from(index)];
            self.count -= 1;
        }
    }

    pub fn discard_unused_frames(&mut self, input_ports: &[u8; 4]) {
        if self.count != 0 {
            let index = self.position.wrapping_sub(self.count) & 0x0f;
            if input_ports == &self.entries[usize::from(index)] {
                if self.total_writes >= 16 {
                    self.total_writes = 14;
                    self.count -= 1;
                }
                return;
            }
        }
        self.total_writes = 0;
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.count = 0;
        self.total_writes = 0;
    }

    #[must_use]
    pub fn current(&self) -> [u8; 4] {
        self.current
    }

    #[must_use]
    pub fn queued(&self) -> u8 {
        self.count
    }
}

pub fn mix_to_buffer_with_volume(dst: &mut [i16], src: &[i16], frames: usize, volume: f32) {
    assert!(dst.len() >= frames * 2 && src.len() >= frames * 2);
    let volume = (65_536.0 * volume) as i32;
    for index in 0..frames * 2 {
        let sample = if volume == 65_536 {
            i32::from(src[index])
        } else {
            (i32::from(src[index]) * volume) >> 16
        };
        dst[index] = dst[index].wrapping_add(sample as i16);
    }
}

pub fn mix_to_buffer_with_volume_ramp(
    dst: &mut [i16],
    src: &[i16],
    frames: usize,
    volume: f32,
    volume_step: f32,
) {
    assert!(dst.len() >= frames * 2 && src.len() >= frames * 2);
    let mut fixed_volume = (volume * 281_474_976_710_656.0) as i64;
    let fixed_step = (volume_step * 281_474_976_710_656.0) as i64;
    for frame in 0..frames {
        let scale = (fixed_volume >> 32) as i32;
        for channel in 0..2 {
            let index = frame * 2 + channel;
            let sample = (i32::from(src[index]) * scale) >> 16;
            dst[index] = dst[index].wrapping_add(sample as i16);
        }
        fixed_volume = fixed_volume.wrapping_add(fixed_step);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VolumeTransition {
    pub volume: f32,
    pub target: f32,
    pub step: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MsuResumeInfo {
    pub tag: u32,
    pub offset: u32,
    pub samples_until_repeat: u32,
    pub range_cur: u16,
    pub range_repeat: u16,
    pub initial_packet_bytes: u64,
    pub original_track: u8,
    pub actual_track: u8,
}

impl MsuResumeInfo {
    #[must_use]
    pub fn to_c_bytes(self) -> [u8; MSU_RESUME_INFO_SIZE] {
        let mut bytes = [0; MSU_RESUME_INFO_SIZE];
        bytes[0..4].copy_from_slice(&self.tag.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.offset.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.samples_until_repeat.to_le_bytes());
        bytes[12..14].copy_from_slice(&self.range_cur.to_le_bytes());
        bytes[14..16].copy_from_slice(&self.range_repeat.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.initial_packet_bytes.to_le_bytes());
        bytes[24] = self.original_track;
        bytes[25] = self.actual_track;
        bytes
    }

    #[must_use]
    pub fn from_c_bytes(bytes: &[u8]) -> Option<Self> {
        let bytes = bytes.get(..MSU_RESUME_INFO_SIZE)?;
        Some(Self {
            tag: u32::from_le_bytes(bytes[0..4].try_into().expect("fixed field")),
            offset: u32::from_le_bytes(bytes[4..8].try_into().expect("fixed field")),
            samples_until_repeat: u32::from_le_bytes(bytes[8..12].try_into().expect("fixed field")),
            range_cur: u16::from_le_bytes(bytes[12..14].try_into().expect("fixed field")),
            range_repeat: u16::from_le_bytes(bytes[14..16].try_into().expect("fixed field")),
            initial_packet_bytes: u64::from_le_bytes(
                bytes[16..24].try_into().expect("fixed field"),
            ),
            original_track: bytes[24],
            actual_track: bytes[25],
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PcmTrackError {
    TooShort,
    WrongTag([u8; 4]),
    RepeatAfterEnd { repeat: u32, samples: u32 },
    OddPayload(usize),
}

/// Decoded view of an MSU1 PCM file. Samples remain interleaved stereo, just
/// as they are read by `MsuPlayer_Mix` in the reference runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PcmTrack {
    samples: Vec<i16>,
    repeat_position: u32,
}

impl PcmTrack {
    pub fn parse(bytes: &[u8]) -> Result<Self, PcmTrackError> {
        if bytes.len() < 8 {
            return Err(PcmTrackError::TooShort);
        }
        let tag: [u8; 4] = bytes[..4].try_into().expect("fixed tag length");
        if &tag != b"MSU1" {
            return Err(PcmTrackError::WrongTag(tag));
        }
        let payload = &bytes[8..];
        if payload.len() & 3 != 0 {
            return Err(PcmTrackError::OddPayload(payload.len()));
        }
        let repeat_position = u32::from_le_bytes(bytes[4..8].try_into().expect("fixed word"));
        let frame_count = (payload.len() / 4) as u32;
        if repeat_position > frame_count {
            return Err(PcmTrackError::RepeatAfterEnd {
                repeat: repeat_position,
                samples: frame_count,
            });
        }
        let samples = payload
            .chunks_exact(2)
            .map(|sample| i16::from_le_bytes([sample[0], sample[1]]))
            .collect();
        Ok(Self {
            samples,
            repeat_position,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, PcmFileError> {
        let bytes = std::fs::read(path).map_err(PcmFileError::Io)?;
        Self::parse(&bytes).map_err(PcmFileError::Format)
    }

    #[must_use]
    pub fn frames(&self) -> u32 {
        (self.samples.len() / 2) as u32
    }
}

#[derive(Debug)]
pub enum PcmFileError {
    Io(io::Error),
    Format(PcmTrackError),
}

impl PartialEq for PcmFileError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(left), Self::Io(right)) => left.kind() == right.kind(),
            (Self::Format(left), Self::Format(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for PcmFileError {}

#[derive(Debug)]
pub enum MsuFileError {
    Pcm(PcmFileError),
    Opuz(OpuzErrorKind),
}

#[must_use]
pub fn msu_track_path(prefix: impl AsRef<Path>, actual_track: u8, opuz: bool) -> PathBuf {
    let mut path = prefix.as_ref().as_os_str().to_owned();
    path.push(actual_track.to_string());
    path.push(if opuz { ".opuz" } else { ".pcm" });
    PathBuf::from(path)
}

fn read_msu_file_tag(path: &Path) -> Result<[u8; 4], MsuFileError> {
    let mut file = File::open(path).map_err(|error| MsuFileError::Pcm(PcmFileError::Io(error)))?;
    let mut tag = [0; 4];
    file.read_exact(&mut tag)
        .map_err(|error| MsuFileError::Pcm(PcmFileError::Io(error)))?;
    Ok(tag)
}

#[derive(Clone, Debug, PartialEq)]
pub struct PcmPlayback {
    pub track: PcmTrack,
    pub resume: MsuResumeInfo,
    pub sample_position: u32,
    pub samples_until_repeat: u32,
    pub volume: VolumeTransition,
    pub state: u8,
    buffer_start: u32,
    buffer_position: u32,
    buffer_size: u32,
}

impl PcmPlayback {
    #[must_use]
    pub fn new(track: PcmTrack, original_track: u8, actual_track: u8) -> Self {
        let samples_until_repeat = track.frames();
        Self {
            track,
            resume: MsuResumeInfo {
                // Little-endian value read from the bytes `MSU1` by C.
                tag: u32::from_le_bytes(*b"MSU1"),
                range_cur: 8,
                original_track,
                actual_track,
                ..MsuResumeInfo::default()
            },
            sample_position: 0,
            samples_until_repeat,
            volume: VolumeTransition {
                volume: 1.0,
                target: 1.0,
                step: 0.0,
            },
            state: MSU_STATE_PLAYING,
            buffer_start: 0,
            buffer_position: 0,
            buffer_size: 0,
        }
    }

    pub fn restore(&mut self, resume: MsuResumeInfo) {
        if resume.tag == u32::from_le_bytes(*b"MSU1")
            && resume.actual_track == self.resume.actual_track
            && resume.offset <= self.track.frames()
        {
            self.sample_position = resume.offset;
            self.samples_until_repeat = self.track.frames().saturating_sub(resume.offset);
            self.resume = resume;
            self.state = MSU_STATE_RESUMING;
            self.buffer_position = 0;
            self.buffer_size = 0;
        }
    }

    /// Mixes as many requested frames as remain available. A non-repeating
    /// track can finish before filling the destination, matching the C early
    /// return from `MsuPlayer_Mix`.
    pub fn mix(&mut self, destination: &mut [i16], repeat: bool) -> usize {
        let requested = destination.len() / 2;
        let mut mixed = 0usize;
        while mixed < requested {
            if self.buffer_position == self.buffer_size {
                if self.samples_until_repeat == 0 {
                    if !repeat {
                        self.state = MSU_STATE_FINISHED_PLAYING;
                        // `MsuPlayer_CloseFile` preserves the finished state but
                        // clears the resume structure in the C runtime.
                        self.resume = MsuResumeInfo::default();
                        break;
                    }
                    self.sample_position = self.track.repeat_position;
                    self.samples_until_repeat = self
                        .track
                        .frames()
                        .saturating_sub(self.track.repeat_position);
                    if self.samples_until_repeat == 0 {
                        self.state = MSU_STATE_FINISHED_PLAYING;
                        break;
                    }
                }
                let loaded = self.samples_until_repeat.min(960);
                self.resume.offset = self.sample_position;
                self.buffer_start = self.sample_position;
                self.buffer_position = 0;
                self.buffer_size = loaded;
                self.sample_position += loaded;
                self.samples_until_repeat -= loaded;
            }
            let frames =
                (requested - mixed).min((self.buffer_size - self.buffer_position) as usize);
            let source_start = (self.buffer_start + self.buffer_position) as usize * 2;
            let source_end = source_start + frames * 2;
            self.volume.mix(
                &mut destination[mixed * 2..(mixed + frames) * 2],
                &self.track.samples[source_start..source_end],
                frames,
            );
            self.buffer_position += frames as u32;
            mixed += frames;
        }
        mixed
    }
}

pub trait OpusPacketDecoder {
    type Error: std::fmt::Debug;

    fn reset(&mut self) -> Result<(), Self::Error>;
    fn decode(&mut self, packet: &[u8], output: &mut [i16]) -> Result<usize, Self::Error>;
}

impl OpusPacketDecoder for z3_opus::Decoder {
    type Error = z3_opus::Error;

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reset()
    }

    fn decode(&mut self, packet: &[u8], output: &mut [i16]) -> Result<usize, Self::Error> {
        self.decode(packet, output)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpuzErrorKind {
    Io(io::ErrorKind),
    WrongTag([u8; 4]),
    InvalidRangeOffset(u32),
    PacketTooLarge(u16),
    Decoder,
    ResumePacketMismatch,
    NoProgress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpuzMixError {
    pub kind: OpuzErrorKind,
    /// The SPC track that the C runtime writes back to APUI00 on failure.
    pub fallback_track: u8,
}

/// File-backed playback for the compact OPUZ container used by the C port.
///
/// Range records, packet framing, preskip, looping, and snapshot verification
/// intentionally follow `MsuPlayer_Mix` rather than a general media format.
#[derive(Debug)]
pub struct OpuzPlayback<D: OpusPacketDecoder> {
    file: Option<File>,
    decoder: Option<D>,
    pub resume: MsuResumeInfo,
    pub samples_until_repeat: u32,
    pub range_cur: u16,
    pub range_repeat: u16,
    pub current_file_offset: u32,
    pub state: u8,
    pub volume: VolumeTransition,
    preskip: usize,
    buffer: [i16; 960 * 2],
    buffer_position: usize,
    buffer_size: usize,
}

impl OpuzPlayback<z3_opus::Decoder> {
    pub fn open(
        path: impl AsRef<Path>,
        original_track: u8,
        actual_track: u8,
        resume: Option<MsuResumeInfo>,
    ) -> Result<Self, OpuzErrorKind> {
        let decoder = z3_opus::Decoder::new_stereo_48khz().map_err(|_| OpuzErrorKind::Decoder)?;
        Self::open_with_decoder(path, original_track, actual_track, resume, decoder)
    }
}

impl<D: OpusPacketDecoder> OpuzPlayback<D> {
    pub fn open_with_decoder(
        path: impl AsRef<Path>,
        original_track: u8,
        actual_track: u8,
        resume: Option<MsuResumeInfo>,
        decoder: D,
    ) -> Result<Self, OpuzErrorKind> {
        let mut file = File::open(path).map_err(|error| OpuzErrorKind::Io(error.kind()))?;
        let mut header = [0; 8];
        file.read_exact(&mut header)
            .map_err(|error| OpuzErrorKind::Io(error.kind()))?;
        let tag_bytes: [u8; 4] = header[..4].try_into().expect("fixed tag");
        if &tag_bytes != b"OPUZ" {
            return Err(OpuzErrorKind::WrongTag(tag_bytes));
        }
        let tag = u32::from_le_bytes(tag_bytes);
        let matching_resume =
            resume.filter(|saved| saved.actual_track == actual_track && saved.tag == tag);
        let (resume, state) = matching_resume.map_or_else(
            || {
                (
                    MsuResumeInfo {
                        tag,
                        range_cur: 8,
                        original_track,
                        actual_track,
                        ..MsuResumeInfo::default()
                    },
                    MSU_STATE_PLAYING,
                )
            },
            |saved| (saved, MSU_STATE_RESUMING),
        );
        if state == MSU_STATE_RESUMING {
            file.seek(SeekFrom::Start(u64::from(resume.offset)))
                .map_err(|error| OpuzErrorKind::Io(error.kind()))?;
        }
        Ok(Self {
            file: Some(file),
            decoder: Some(decoder),
            samples_until_repeat: resume.samples_until_repeat,
            range_cur: resume.range_cur,
            range_repeat: resume.range_repeat,
            current_file_offset: resume.offset,
            resume,
            state,
            volume: VolumeTransition {
                volume: 1.0,
                target: 1.0,
                step: 0.0,
            },
            preskip: 0,
            buffer: [0; 960 * 2],
            buffer_position: 0,
            buffer_size: 0,
        })
    }

    fn fail(&mut self, kind: OpuzErrorKind) -> OpuzMixError {
        let fallback_track = self.resume.original_track;
        self.file = None;
        self.decoder = None;
        self.state = MSU_STATE_IDLE;
        self.resume = MsuResumeInfo::default();
        OpuzMixError {
            kind,
            fallback_track,
        }
    }

    fn finish(&mut self) {
        self.state = MSU_STATE_FINISHED_PLAYING;
        self.file = None;
        self.decoder = None;
        self.resume = MsuResumeInfo::default();
    }

    fn read_range(&mut self) -> Result<bool, OpuzErrorKind> {
        if self.range_cur == 0 {
            return Ok(false);
        }
        self.decoder
            .as_mut()
            .expect("open decoder")
            .reset()
            .map_err(|_| OpuzErrorKind::Decoder)?;
        let file = self.file.as_mut().expect("open OPUZ file");
        file.seek(SeekFrom::Start(u64::from(self.range_cur)))
            .map_err(|error| OpuzErrorKind::Io(error.kind()))?;
        let mut range = [0; 10];
        file.read_exact(&mut range)
            .map_err(|error| OpuzErrorKind::Io(error.kind()))?;
        let file_offset = u32::from_le_bytes(range[0..4].try_into().expect("fixed word"));
        if file_offset & 0xf000_0000 != 0 {
            return Err(OpuzErrorKind::InvalidRangeOffset(file_offset));
        }
        self.samples_until_repeat = u32::from_le_bytes(range[4..8].try_into().expect("fixed word"));
        let preskip_flags = u16::from_le_bytes(range[8..10].try_into().expect("fixed word"));
        self.preskip = usize::from(preskip_flags & 0x3fff);
        if preskip_flags & 0x4000 != 0 {
            self.range_repeat = self.range_cur;
        }
        self.range_cur = if preskip_flags & 0x8000 != 0 {
            self.range_repeat
        } else {
            self.range_cur.wrapping_add(10)
        };
        self.current_file_offset = file_offset;
        self.resume.range_repeat = self.range_repeat;
        self.resume.range_cur = self.range_cur;
        file.seek(SeekFrom::Start(u64::from(file_offset)))
            .map_err(|error| OpuzErrorKind::Io(error.kind()))?;
        Ok(true)
    }

    fn decode_packet(&mut self) -> Result<(), OpuzErrorKind> {
        loop {
            let file = self.file.as_mut().expect("open OPUZ file");
            let mut packet_header = [0; 2];
            file.read_exact(&mut packet_header)
                .map_err(|error| OpuzErrorKind::Io(error.kind()))?;
            let framing = u16::from_le_bytes(packet_header);
            let packet_size = framing & 0x7fff;
            if packet_size > 1275 {
                return Err(OpuzErrorKind::PacketTooLarge(packet_size));
            }
            let prefix = usize::from(framing >> 15);
            let mut framed_packet = vec![0; usize::from(packet_size) + 2];
            framed_packet[..2].copy_from_slice(&packet_header);
            file.read_exact(&mut framed_packet[2..])
                .map_err(|error| OpuzErrorKind::Io(error.kind()))?;
            let mut initial = [0; 8];
            let initial_len = initial.len().min(framed_packet.len());
            initial[..initial_len].copy_from_slice(&framed_packet[..initial_len]);
            let initial_packet_bytes = u64::from_le_bytes(initial);
            if self.state == MSU_STATE_RESUMING {
                self.state = MSU_STATE_PLAYING;
                if self.resume.initial_packet_bytes != initial_packet_bytes {
                    return Err(OpuzErrorKind::ResumePacketMismatch);
                }
            }
            self.resume.initial_packet_bytes = initial_packet_bytes;
            self.resume.samples_until_repeat =
                self.samples_until_repeat.wrapping_add(self.preskip as u32);
            self.resume.offset = self.current_file_offset;
            self.current_file_offset = self
                .current_file_offset
                .wrapping_add(2 + u32::from(packet_size));
            framed_packet[1] = 0xfc;
            let packet = &framed_packet[2 - prefix..2 + usize::from(packet_size)];
            let decoded = self
                .decoder
                .as_mut()
                .expect("open decoder")
                .decode(packet, &mut self.buffer)
                .map_err(|_| OpuzErrorKind::Decoder)?;
            if decoded == 0 || decoded > 960 {
                return Err(OpuzErrorKind::Decoder);
            }
            if decoded > self.preskip {
                let available = decoded - self.preskip;
                let buffered = available.min(self.samples_until_repeat as usize);
                self.samples_until_repeat -= buffered as u32;
                self.buffer_position = self.preskip;
                self.buffer_size = self.preskip + buffered;
                self.preskip = 0;
                return Ok(());
            }
            self.preskip -= decoded;
        }
    }

    /// Mixes OPUZ frames into an interleaved stereo destination.
    ///
    /// Decode/read failures return the original SPC track so the caller can
    /// perform the exact C fallback write.
    pub fn mix(&mut self, destination: &mut [i16]) -> Result<usize, OpuzMixError> {
        let requested = destination.len() / 2;
        let mut mixed = 0;
        while mixed < requested {
            if self.buffer_position == self.buffer_size {
                if self.samples_until_repeat == 0 {
                    match self.read_range() {
                        Ok(true) => {}
                        Ok(false) => {
                            self.finish();
                            return Ok(mixed);
                        }
                        Err(kind) => return Err(self.fail(kind)),
                    }
                }
                if self.samples_until_repeat == 0 {
                    return Err(self.fail(OpuzErrorKind::NoProgress));
                }
                if let Err(kind) = self.decode_packet() {
                    return Err(self.fail(kind));
                }
            }
            let frames = (requested - mixed).min(self.buffer_size - self.buffer_position);
            let source_start = self.buffer_position * 2;
            let source_end = source_start + frames * 2;
            self.volume.mix(
                &mut destination[mixed * 2..(mixed + frames) * 2],
                &self.buffer[source_start..source_end],
                frames,
            );
            self.buffer_position += frames;
            mixed += frames;
        }
        Ok(mixed)
    }
}

#[derive(Debug)]
pub struct AudioRuntime {
    pub player: SpcPlayer,
    pub queue: ApuWriteQueue,
    pub pcm: Option<PcmPlayback>,
    pub opuz: Option<OpuzPlayback<z3_opus::Decoder>>,
    pub msu_enabled: u8,
    transition_targets: [f32; 4],
    transition_steps: [f32; 4],
}

impl Default for AudioRuntime {
    fn default() -> Self {
        let mut player = SpcPlayer::new();
        player.initialize();
        Self {
            player,
            queue: ApuWriteQueue::default(),
            pcm: None,
            opuz: None,
            msu_enabled: 0,
            transition_targets: [0.0; 4],
            transition_steps: [0.0; 4],
        }
    }
}

impl AudioRuntime {
    /// Snapshot the C-visible MSU playback state used by
    /// `ZeldaIsPlayingMusicTrack` without exposing the playback owners.
    #[must_use]
    pub fn music_playback_state(&self) -> (u8, u8, u8) {
        let (state, actual_track) = if let Some(opuz) = &self.opuz {
            (opuz.state, opuz.resume.actual_track)
        } else {
            self.pcm.as_ref().map_or((MSU_STATE_IDLE, 0), |pcm| {
                (pcm.state, pcm.resume.actual_track)
            })
        };
        (state, self.msu_enabled, actual_track)
    }

    #[must_use]
    pub fn read_apu(&self, address: u32) -> u8 {
        self.player.port_to_snes[(address & 3) as usize]
    }

    #[must_use]
    pub fn zelda_apu_read(&self, address: u32) -> u8 {
        self.read_apu(address)
    }

    pub fn enable_msu(&mut self, enabled: u8, volume_percent: u8, audio_frequency: u32) {
        self.msu_enabled = enabled;
        (self.transition_targets, self.transition_steps) =
            VolumeTransition::tables(volume_percent, audio_frequency);
        if let Some(pcm) = &mut self.pcm {
            pcm.volume.volume = 1.0;
        }
        if let Some(opuz) = &mut self.opuz {
            opuz.volume.volume = 1.0;
        }
    }

    fn open_file_track(
        &mut self,
        path_prefix: &Path,
        original_track: u8,
        actual_track: u8,
        resume: Option<MsuResumeInfo>,
    ) -> Result<(), MsuFileError> {
        self.pcm = None;
        self.opuz = None;
        if actual_track == 0 {
            return Ok(());
        }
        let extension_is_opuz = self.msu_enabled & MSU_ENABLED_OPUZ != 0;
        let path = msu_track_path(path_prefix, actual_track, extension_is_opuz);
        match read_msu_file_tag(&path) {
            Ok(tag) if &tag == b"OPUZ" => {
                let mut playback = OpuzPlayback::open(&path, original_track, actual_track, resume)
                    .map_err(MsuFileError::Opuz)?;
                playback.volume.target = self.transition_targets[3];
                playback.volume.step = self.transition_steps[3];
                self.opuz = Some(playback);
                Ok(())
            }
            Ok(_) => {
                let track = PcmTrack::open(&path).map_err(MsuFileError::Pcm)?;
                let mut playback = PcmPlayback::new(track, original_track, actual_track);
                playback.volume.target = self.transition_targets[3];
                playback.volume.step = self.transition_steps[3];
                if let Some(resume) = resume {
                    playback.restore(resume);
                }
                self.pcm = Some(playback);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Applies one game music-control write. File lookup remains a host
    /// callback; successful PCM tracks are owned and decoded by this runtime.
    pub fn play_music_control(
        &mut self,
        music_control: u8,
        area: u8,
        entrance: u16,
        mut load_pcm_track: impl FnMut(u8) -> Option<PcmTrack>,
    ) {
        if self.msu_enabled == 0 {
            self.write_apu(0, music_control);
            return;
        }
        if music_control & 0xf0 != 0xf0 {
            self.opuz = None;
            let actual = remap_msu_deluxe_track(self.msu_enabled, music_control, area, entrance);
            self.pcm = if actual == 0 {
                None
            } else {
                load_pcm_track(actual).map(|track| {
                    let mut playback = PcmPlayback::new(track, music_control, actual);
                    playback.volume.target = self.transition_targets[3];
                    playback.volume.step = self.transition_steps[3];
                    playback
                })
            };
        } else if (0xf1..=0xf3).contains(&music_control)
            && let Some(playback) = &mut self.pcm
        {
            let index = usize::from(music_control - 0xf1);
            playback.volume.target = self.transition_targets[index];
            playback.volume.step = self.transition_steps[index];
        }

        if (0xf1..=0xf3).contains(&music_control)
            && let Some(playback) = &mut self.opuz
        {
            let index = usize::from(music_control - 0xf1);
            playback.volume.target = self.transition_targets[index];
            playback.volume.step = self.transition_steps[index];
        }

        self.write_apu(
            0,
            if self.pcm.is_some() || self.opuz.is_some() {
                0xf0
            } else {
                music_control
            },
        );
    }

    /// Opens the same `<prefix><track>.pcm/.opuz` path as the C runtime and
    /// installs a real file-backed decoder.
    pub fn play_music_control_from_files(
        &mut self,
        music_control: u8,
        area: u8,
        entrance: u16,
        path_prefix: impl AsRef<Path>,
    ) -> Result<(), MsuFileError> {
        if self.msu_enabled == 0 {
            self.write_apu(0, music_control);
            return Ok(());
        }
        if (0xf1..=0xf3).contains(&music_control) {
            let index = usize::from(music_control - 0xf1);
            if let Some(playback) = &mut self.pcm {
                playback.volume.target = self.transition_targets[index];
                playback.volume.step = self.transition_steps[index];
            }
            if let Some(playback) = &mut self.opuz {
                playback.volume.target = self.transition_targets[index];
                playback.volume.step = self.transition_steps[index];
            }
        } else if music_control & 0xf0 != 0xf0 {
            let actual = remap_msu_deluxe_track(self.msu_enabled, music_control, area, entrance);
            if let Err(error) =
                self.open_file_track(path_prefix.as_ref(), music_control, actual, None)
            {
                self.write_apu(0, music_control);
                return Err(error);
            }
        }
        self.write_apu(
            0,
            if self.pcm.is_some() || self.opuz.is_some() {
                0xf0
            } else {
                music_control
            },
        );
        Ok(())
    }

    pub fn zelda_play_msu_audio_track(
        &mut self,
        music_control: u8,
        area: u8,
        entrance: u16,
        path_prefix: impl AsRef<Path>,
    ) -> Result<(), MsuFileError> {
        self.play_music_control_from_files(music_control, area, entrance, path_prefix)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn play_music_control_from_files_with_resume_ram(
        &mut self,
        music_control: u8,
        area: u8,
        entrance: u16,
        main_module: u8,
        resume_msu: bool,
        wram: &mut [u8],
        path_prefix: impl AsRef<Path>,
    ) -> Result<(), MsuFileError> {
        if self.msu_enabled == 0 || music_control & 0xf0 == 0xf0 {
            return self.play_music_control_from_files(music_control, area, entrance, path_prefix);
        }
        let actual = remap_msu_deluxe_track(self.msu_enabled, music_control, area, entrance);
        let alternate = MsuResumeInfo::from_c_bytes(
            wram.get(MSU_RESUME_INFO_ALT_OFFSET..MSU_RESUME_INFO_ALT_OFFSET + MSU_RESUME_INFO_SIZE)
                .expect("WRAM must contain the alternate MSU resume area"),
        )
        .expect("fixed resume structure");
        let resume = (main_module == 9 && resume_msu && alternate.actual_track == actual)
            .then_some(alternate);
        let previous = self
            .opuz
            .as_ref()
            .map(|playback| (playback.state, playback.resume))
            .or_else(|| {
                self.pcm
                    .as_ref()
                    .map(|playback| (playback.state, playback.resume))
            });
        if let Some((state, previous)) = previous
            && state >= MSU_STATE_RESUMING
        {
            wram[MSU_RESUME_INFO_ALT_OFFSET..MSU_RESUME_INFO_ALT_OFFSET + MSU_RESUME_INFO_SIZE]
                .copy_from_slice(&previous.to_c_bytes());
        }
        if let Err(error) =
            self.open_file_track(path_prefix.as_ref(), music_control, actual, resume)
        {
            self.write_apu(0, music_control);
            return Err(error);
        }
        self.write_apu(
            0,
            if self.pcm.is_some() || self.opuz.is_some() {
                0xf0
            } else {
                music_control
            },
        );
        Ok(())
    }

    /// Variant of [`Self::play_music_control`] that also reproduces the C
    /// runtime's alternate-resume slot used when returning to the overworld.
    #[allow(clippy::too_many_arguments)]
    pub fn play_music_control_with_resume_ram(
        &mut self,
        music_control: u8,
        area: u8,
        entrance: u16,
        main_module: u8,
        resume_msu: bool,
        wram: &mut [u8],
        mut load_pcm_track: impl FnMut(u8) -> Option<PcmTrack>,
    ) {
        if self.msu_enabled == 0 {
            self.write_apu(0, music_control);
            return;
        }

        if music_control & 0xf0 != 0xf0 {
            self.opuz = None;
            let actual = remap_msu_deluxe_track(self.msu_enabled, music_control, area, entrance);
            let alternate = MsuResumeInfo::from_c_bytes(
                wram.get(
                    MSU_RESUME_INFO_ALT_OFFSET..MSU_RESUME_INFO_ALT_OFFSET + MSU_RESUME_INFO_SIZE,
                )
                .expect("WRAM must contain the alternate MSU resume area"),
            )
            .expect("fixed resume structure");
            let resume = (main_module == 9 && resume_msu && alternate.actual_track == actual)
                .then_some(alternate);

            if let Some(previous) = &self.pcm
                && previous.state >= MSU_STATE_RESUMING
            {
                wram[MSU_RESUME_INFO_ALT_OFFSET..MSU_RESUME_INFO_ALT_OFFSET + MSU_RESUME_INFO_SIZE]
                    .copy_from_slice(&previous.resume.to_c_bytes());
            }

            self.pcm = if actual == 0 {
                None
            } else {
                load_pcm_track(actual).map(|track| {
                    let mut playback = PcmPlayback::new(track, music_control, actual);
                    playback.volume.target = self.transition_targets[3];
                    playback.volume.step = self.transition_steps[3];
                    if let Some(resume) = resume {
                        playback.restore(resume);
                    }
                    playback
                })
            };
        } else if (0xf1..=0xf3).contains(&music_control)
            && let Some(playback) = &mut self.pcm
        {
            let index = usize::from(music_control - 0xf1);
            playback.volume.target = self.transition_targets[index];
            playback.volume.step = self.transition_steps[index];
        }

        self.write_apu(
            0,
            if self.pcm.is_some() {
                0xf0
            } else {
                music_control
            },
        );
    }

    pub fn write_apu(&mut self, address: u32, value: u8) {
        self.queue.write(address, value);
    }

    pub fn push_apu_state(&mut self) {
        self.queue.push();
    }

    pub fn discard_unused_audio_frames(&mut self) {
        self.queue.discard_unused_frames(&self.player.input_ports);
    }

    pub fn render_audio(&mut self, output: &mut [i16], frames: usize, channels: usize) {
        assert!(output.len() >= frames * channels);
        self.queue.pop_into(&mut self.player.input_ports);
        self.player.generate_samples();
        self.player.dsp.get_samples(output, frames, channels);
        if channels == 2
            && let Some(pcm) = &mut self.pcm
            && pcm.state != MSU_STATE_FINISHED_PLAYING
        {
            let actual_track = usize::from(pcm.resume.actual_track);
            let repeat = MSU_TRACKS_WITH_REPEAT
                .get(actual_track)
                .is_none_or(|&value| value != 0);
            pcm.mix(&mut output[..frames * 2], repeat);
        }
        if channels == 2
            && let Some(opuz) = &mut self.opuz
            && opuz.state != MSU_STATE_FINISHED_PLAYING
            && let Err(error) = opuz.mix(&mut output[..frames * 2])
        {
            self.queue.write(0, error.fallback_track);
        }
    }

    #[must_use]
    pub fn is_music_playing(&self) -> bool {
        if let Some(opuz) = &self.opuz {
            opuz.state != MSU_STATE_FINISHED_PLAYING
        } else {
            self.pcm
                .as_ref()
                .map_or(self.player.port_to_snes[0] != 0, |pcm| {
                    pcm.state != MSU_STATE_FINISHED_PLAYING
                })
        }
    }

    pub fn save_music_state_to_ram(&mut self) {
        self.player.copy_variables_to_ram();
        self.player.ram[0x410..0x414].copy_from_slice(&self.queue.current());
    }

    /// Saves the MSU fields that the C runtime embeds in its WRAM snapshot.
    /// SPC state is saved by [`Self::save_music_state_to_ram`] because that
    /// state belongs to the emulated SPC RAM rather than SNES WRAM.
    pub fn save_msu_state_to_wram(&self, wram: &mut [u8]) {
        let (volume, resume) = if let Some(playback) = &self.opuz {
            ((playback.volume.volume * 255.0) as u8, playback.resume)
        } else {
            self.pcm
                .as_ref()
                .map_or((0, MsuResumeInfo::default()), |playback| {
                    ((playback.volume.volume * 255.0) as u8, playback.resume)
                })
        };
        wram[MSU_VOLUME_OFFSET] = volume;
        let resume_range = MSU_RESUME_INFO_OFFSET..MSU_RESUME_INFO_OFFSET + MSU_RESUME_INFO_SIZE;
        let destination = wram
            .get_mut(resume_range)
            .expect("WRAM must contain the MSU resume area");
        destination.copy_from_slice(&resume.to_c_bytes());
    }

    /// Restores PCM playback from the exact game-visible snapshot fields.
    /// File lookup stays injectable so deterministic tests and non-SDL hosts
    /// can use the same state transition logic as the C runtime.
    pub fn restore_msu_from_wram(
        &mut self,
        wram: &[u8],
        area: u8,
        entrance: u16,
        mut load_pcm_track: impl FnMut(u8) -> Option<PcmTrack>,
    ) {
        if self.msu_enabled == 0 {
            return;
        }
        let resume = MsuResumeInfo::from_c_bytes(
            wram.get(MSU_RESUME_INFO_OFFSET..MSU_RESUME_INFO_OFFSET + MSU_RESUME_INFO_SIZE)
                .expect("WRAM must contain the MSU resume area"),
        )
        .expect("fixed resume structure");
        let music = wram[MUSIC_UNK1_OFFSET];
        let original_track = if music == 0xf1 {
            resume.original_track
        } else {
            music
        };
        let actual_track = remap_msu_deluxe_track(self.msu_enabled, original_track, area, entrance);

        self.pcm = if actual_track == 0 {
            None
        } else {
            load_pcm_track(actual_track).map(|track| {
                let mut playback = PcmPlayback::new(track, original_track, actual_track);
                playback.volume.volume = 0.0;
                playback.volume.target = self.transition_targets[3];
                playback.volume.step = self.transition_steps[3];
                playback.restore(resume);

                let last_control = wram[LAST_MUSIC_CONTROL_OFFSET];
                if (0xf1..=0xf3).contains(&last_control) {
                    let index = usize::from(last_control - 0xf1);
                    let target = VOLUME_TRANSITION_TARGET[index];
                    let saved_volume = wram[MSU_VOLUME_OFFSET];
                    if target != saved_volume {
                        let scale = self.transition_targets[3] * (1.0 / 255.0);
                        playback.volume.volume = f32::from(saved_volume) * scale;
                        playback.volume.target = f32::from(target) * scale;
                        playback.volume.step = self.transition_steps[index];
                    }
                }
                playback
            })
        };
        if self.pcm.is_some() {
            self.write_apu(0, 0xf0);
        }
    }

    /// File-backed counterpart of [`Self::restore_msu_from_wram`], including
    /// OPUZ packet-boundary verification against the saved resume structure.
    pub fn restore_msu_from_wram_from_files(
        &mut self,
        wram: &[u8],
        area: u8,
        entrance: u16,
        path_prefix: impl AsRef<Path>,
    ) -> Result<(), MsuFileError> {
        if self.msu_enabled == 0 {
            return Ok(());
        }
        let resume = MsuResumeInfo::from_c_bytes(
            wram.get(MSU_RESUME_INFO_OFFSET..MSU_RESUME_INFO_OFFSET + MSU_RESUME_INFO_SIZE)
                .expect("WRAM must contain the MSU resume area"),
        )
        .expect("fixed resume structure");
        let music = wram[MUSIC_UNK1_OFFSET];
        let original_track = if music == 0xf1 {
            resume.original_track
        } else {
            music
        };
        let actual_track = remap_msu_deluxe_track(self.msu_enabled, original_track, area, entrance);
        self.pcm = None;
        self.opuz = None;
        if actual_track == 0 {
            return Ok(());
        }

        let use_opuz = self.msu_enabled & MSU_ENABLED_OPUZ != 0;
        let path = msu_track_path(path_prefix, actual_track, use_opuz);
        let tag = read_msu_file_tag(&path)?;
        if &tag == b"OPUZ" {
            let mut playback = OpuzPlayback::open(path, original_track, actual_track, Some(resume))
                .map_err(MsuFileError::Opuz)?;
            playback.volume.volume = 0.0;
            playback.volume.target = self.transition_targets[3];
            playback.volume.step = self.transition_steps[3];
            restore_saved_volume_transition(
                &mut playback.volume,
                wram,
                self.transition_targets,
                self.transition_steps,
            );
            self.opuz = Some(playback);
        } else {
            let track = PcmTrack::open(path).map_err(MsuFileError::Pcm)?;
            let mut playback = PcmPlayback::new(track, original_track, actual_track);
            playback.volume.volume = 0.0;
            playback.volume.target = self.transition_targets[3];
            playback.volume.step = self.transition_steps[3];
            playback.restore(resume);
            restore_saved_volume_transition(
                &mut playback.volume,
                wram,
                self.transition_targets,
                self.transition_steps,
            );
            self.pcm = Some(playback);
        }
        self.write_apu(0, 0xf0);
        Ok(())
    }

    pub fn restore_music_after_load(&mut self, reset: bool) {
        self.player.copy_variables_from_ram();
        self.player.timer_cycles = 0;
        let ports: [u8; 4] = self.player.ram[0x410..0x414]
            .try_into()
            .expect("fixed saved port range");
        self.player.input_ports = ports;
        for (address, value) in ports.into_iter().enumerate() {
            self.queue.write(address as u32, value);
        }
        if reset {
            self.player.initialize();
        }
        self.queue.reset();
    }

    pub fn zelda_restore_music_after_load_locked(
        &mut self,
        reset: bool,
        wram: &[u8],
        area: u8,
        entrance: u16,
        path_prefix: impl AsRef<Path>,
    ) -> Result<(), MsuFileError> {
        self.restore_music_after_load(reset);
        self.restore_msu_from_wram_from_files(wram, area, entrance, path_prefix)
    }

    pub fn zelda_save_music_state_to_ram_locked(&mut self, wram: &mut [u8]) {
        self.save_music_state_to_ram();
        self.save_msu_state_to_wram(wram);
    }

    pub fn load_song_bank(&mut self, bank: &[u8]) -> Result<(), crate::spc_player::UploadError> {
        self.player.upload(bank)
    }
}

/// Per-frame adapter used by NMI. It captures the game-visible fields needed
/// for deluxe remapping while allowing the host to provide PCM files lazily.
pub struct NmiAudioAdapter<'a, F> {
    pub runtime: &'a mut AudioRuntime,
    pub area: u8,
    pub entrance: u16,
    pub music_unk1: u8,
    pub last_music_control: u8,
    pub misc_bug_fixes: bool,
    pub load_pcm_track: F,
}

impl<F> crate::nmi::NmiAudio for NmiAudioAdapter<'_, F>
where
    F: FnMut(u8) -> Option<PcmTrack>,
{
    fn is_bugged_music_track(&self, track: u8) -> bool {
        let (state, actual_track) = if let Some(opuz) = &self.runtime.opuz {
            (opuz.state, opuz.resume.actual_track)
        } else {
            self.runtime
                .pcm
                .as_ref()
                .map_or((MSU_STATE_IDLE, 0), |pcm| {
                    (pcm.state, pcm.resume.actual_track)
                })
        };
        is_playing_music_track_with_bug(
            state,
            self.runtime.msu_enabled,
            actual_track,
            self.music_unk1,
            self.last_music_control,
            self.misc_bug_fixes,
            track,
            self.area,
            self.entrance,
        )
    }

    fn play_music_track(&mut self, track: u8) {
        self.runtime
            .play_music_control(track, self.area, self.entrance, &mut self.load_pcm_track);
    }

    fn read_apu_port(&mut self, port: u8) -> u8 {
        self.runtime.player.port_to_snes[usize::from(port & 3)]
    }

    fn write_apu_port(&mut self, port: u8, value: u8) {
        self.runtime.write_apu(u32::from(port), value);
    }
}

fn restore_saved_volume_transition(
    volume: &mut VolumeTransition,
    wram: &[u8],
    targets: [f32; 4],
    steps: [f32; 4],
) {
    let last_control = wram[LAST_MUSIC_CONTROL_OFFSET];
    if (0xf1..=0xf3).contains(&last_control) {
        let index = usize::from(last_control - 0xf1);
        let target = VOLUME_TRANSITION_TARGET[index];
        let saved_volume = wram[MSU_VOLUME_OFFSET];
        if target != saved_volume {
            let scale = targets[3] * (1.0 / 255.0);
            volume.volume = f32::from(saved_volume) * scale;
            volume.target = f32::from(target) * scale;
            volume.step = steps[index];
        }
    }
}

impl VolumeTransition {
    #[must_use]
    pub fn tables(msu_volume_percent: u8, audio_frequency: u32) -> ([f32; 4], [f32; 4]) {
        let volume_scale = f32::from(msu_volume_percent) * (1.0 / 255.0 / 100.0);
        let step_scale =
            f32::from(msu_volume_percent) * (60.0 / 256.0 / 100.0) / audio_frequency as f32;
        (
            VOLUME_TRANSITION_TARGET.map(|value| f32::from(value) * volume_scale),
            VOLUME_TRANSITION_STEP.map(|value| f32::from(value) * step_scale),
        )
    }

    pub fn mix(&mut self, dst: &mut [i16], src: &[i16], frames: usize) {
        assert!(dst.len() >= frames * 2 && src.len() >= frames * 2);
        let mut ramp_frames = 0;
        if self.volume != self.target {
            let step = if self.volume < self.target {
                self.step
            } else {
                -self.step
            };
            let candidate = self.volume + step * frames as f32;
            ramp_frames = frames;
            let crossed = if step >= 0.0 {
                candidate >= self.target
            } else {
                candidate < self.target
            };
            let new_volume = if crossed {
                ramp_frames = (((self.target - self.volume) / step) as u32 as usize).min(frames);
                self.target
            } else {
                candidate
            };
            mix_to_buffer_with_volume_ramp(dst, src, ramp_frames, self.volume, step);
            self.volume = new_volume;
        }
        mix_to_buffer_with_volume(
            &mut dst[ramp_frames * 2..],
            &src[ramp_frames * 2..],
            frames - ramp_frames,
            self.volume,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temp_file(extension: &str, bytes: &[u8]) -> PathBuf {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "z3-audio-test-{}-{sequence}.{extension}",
            std::process::id()
        ));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[derive(Debug)]
    struct FakeOpus {
        packets: Rc<RefCell<Vec<Vec<u8>>>>,
        resets: Rc<RefCell<usize>>,
    }

    impl OpusPacketDecoder for FakeOpus {
        type Error = ();

        fn reset(&mut self) -> Result<(), Self::Error> {
            *self.resets.borrow_mut() += 1;
            Ok(())
        }

        fn decode(&mut self, packet: &[u8], output: &mut [i16]) -> Result<usize, Self::Error> {
            self.packets.borrow_mut().push(packet.to_vec());
            for (index, frame) in output.chunks_exact_mut(2).take(3).enumerate() {
                frame[0] = 100 + index as i16;
                frame[1] = 200 + index as i16;
            }
            Ok(3)
        }
    }

    fn opuz_file(range_flags: u16, packet_framing: u16) -> Vec<u8> {
        let mut bytes = b"OPUZ".to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&18u32.to_le_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&range_flags.to_le_bytes());
        bytes.extend_from_slice(&packet_framing.to_le_bytes());
        bytes.extend_from_slice(&[0x11, 0x22]);
        bytes
    }

    #[test]
    fn deluxe_mapping_preserves_world_entrance_and_242_rules() {
        assert_eq!(remap_msu_deluxe_track(MSU_ENABLED_DELUXE, 2, 0, 0), 37);
        assert_eq!(remap_msu_deluxe_track(MSU_ENABLED_DELUXE, 16, 0, 2), 60);
        assert_eq!(remap_msu_deluxe_track(MSU_ENABLED_DELUXE, 16, 0, 62), 16);
        assert_eq!(remap_msu_deluxe_track(0, 2, 0, 0), 2);
        assert_eq!(entrance_music_track(3, MSU_ENABLED_DELUXE, 2, 242), 16);
        assert_eq!(entrance_music_track(3, MSU_ENABLED_DELUXE, 62, 242), 242);
    }

    #[test]
    fn apu_queue_keeps_last_sixteen_frames_and_pops_in_order() {
        let mut queue = ApuWriteQueue::default();
        for value in 0..20u8 {
            queue.write(0, value);
            queue.write(3, value ^ 0x55);
            queue.push();
        }
        assert_eq!(queue.queued(), 16);
        let mut ports = [0; 4];
        queue.pop_into(&mut ports);
        assert_eq!(ports, [4, 0, 0, 4 ^ 0x55]);
        queue.pop_into(&mut ports);
        assert_eq!(ports, [5, 0, 0, 5 ^ 0x55]);
    }

    #[test]
    fn discard_backpressure_matches_reference_threshold() {
        let mut queue = ApuWriteQueue::default();
        for _ in 0..16 {
            queue.push();
        }
        assert_eq!(queue.queued(), 16);
        queue.discard_unused_frames(&[0; 4]);
        assert_eq!(queue.queued(), 15);
        queue.discard_unused_frames(&[1, 0, 0, 0]);
        queue.discard_unused_frames(&[0; 4]);
        assert_eq!(queue.queued(), 15);
    }

    #[test]
    fn volume_mixer_uses_wrapping_i16_accumulation() {
        let mut dst = [30_000, -30_000, 10, 20];
        let src = [10_000, -10_000, 100, 200];
        mix_to_buffer_with_volume(&mut dst, &src, 2, 0.5);
        assert_eq!(dst, [35_000i32 as i16, (-35_000i32) as i16, 60, 120]);
    }

    #[test]
    fn transition_mixer_reaches_target_then_uses_constant_volume() {
        let mut transition = VolumeTransition {
            volume: 0.0,
            target: 0.5,
            step: 0.25,
        };
        let src = [1000; 8];
        let mut dst = [0; 8];
        transition.mix(&mut dst, &src, 4);
        assert_eq!(transition.volume, 0.5);
        assert_eq!(dst, [0, 0, 250, 250, 500, 500, 500, 500]);
        let (targets, steps) = VolumeTransition::tables(100, 48_000);
        assert_eq!(targets[0], 0.0);
        assert!(targets[3] > 0.99 && targets[3] < 1.01);
        assert!(steps[3] > steps[0]);
    }

    #[test]
    fn pcm_track_repeats_at_header_position_and_saves_frame_offset() {
        let mut file = b"MSU1".to_vec();
        file.extend_from_slice(&1u32.to_le_bytes());
        for sample in [10i16, 20, 30, 40, 50, 60] {
            file.extend_from_slice(&sample.to_le_bytes());
        }
        let track = PcmTrack::parse(&file).unwrap();
        assert_eq!(track.frames(), 3);
        let mut playback = PcmPlayback::new(track, 2, 2);
        let mut output = [0i16; 10];
        assert_eq!(playback.mix(&mut output, true), 5);
        assert_eq!(output, [10, 20, 30, 40, 50, 60, 30, 40, 50, 60]);
        assert_eq!(playback.sample_position, 3);
        assert_eq!(playback.resume.offset, 1);
    }

    #[test]
    fn pcm_resume_offset_stays_at_the_loaded_960_frame_chunk() {
        let mut file = b"MSU1".to_vec();
        file.extend_from_slice(&0u32.to_le_bytes());
        for frame in 0..1_000i16 {
            file.extend_from_slice(&frame.to_le_bytes());
            file.extend_from_slice(&(-frame).to_le_bytes());
        }
        let mut playback = PcmPlayback::new(PcmTrack::parse(&file).unwrap(), 2, 2);
        assert_eq!(playback.mix(&mut [0; 200], true), 100);
        assert_eq!(playback.sample_position, 960);
        assert_eq!(playback.resume.offset, 0);
        assert_eq!(playback.resume.samples_until_repeat, 0);
        assert_eq!(playback.mix(&mut [0; 200], true), 100);
        assert_eq!(playback.resume.offset, 0);
        assert_eq!(playback.mix(&mut [0; 1_600], true), 800);
        assert_eq!(playback.resume.offset, 960);
        assert_eq!(playback.sample_position, 1_000);
    }

    #[test]
    fn finished_pcm_clears_resume_and_never_restarts_on_later_callbacks() {
        let mut file = b"MSU1".to_vec();
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&10i16.to_le_bytes());
        file.extend_from_slice(&20i16.to_le_bytes());
        let track = PcmTrack::parse(&file).unwrap();
        let mut playback = PcmPlayback::new(track, 8, 8);
        let mut output = [0; 4];
        assert_eq!(playback.mix(&mut output, false), 1);
        assert_eq!(playback.state, MSU_STATE_FINISHED_PLAYING);
        assert_eq!(playback.resume, MsuResumeInfo::default());

        let mut runtime = AudioRuntime {
            pcm: Some(playback),
            ..AudioRuntime::default()
        };
        let mut later = [0; 2];
        runtime.render_audio(&mut later, 1, 2);
        assert_eq!(later, [0, 0]);
        assert_eq!(
            runtime.pcm.as_ref().unwrap().state,
            MSU_STATE_FINISHED_PLAYING
        );
    }

    #[test]
    fn pcm_track_rejects_non_msu_and_invalid_repeat_headers() {
        assert_eq!(PcmTrack::parse(b"bad"), Err(PcmTrackError::TooShort));
        let mut wrong = b"NOPE".to_vec();
        wrong.extend_from_slice(&0u32.to_le_bytes());
        assert!(matches!(
            PcmTrack::parse(&wrong),
            Err(PcmTrackError::WrongTag(_))
        ));
        let mut repeat = b"MSU1".to_vec();
        repeat.extend_from_slice(&1u32.to_le_bytes());
        assert_eq!(
            PcmTrack::parse(&repeat),
            Err(PcmTrackError::RepeatAfterEnd {
                repeat: 1,
                samples: 0
            })
        );
    }

    #[test]
    fn file_backed_pcm_open_and_c_style_track_name() {
        let mut bytes = b"MSU1".to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&1i16.to_le_bytes());
        bytes.extend_from_slice(&2i16.to_le_bytes());
        let path = temp_file("pcm", &bytes);
        assert_eq!(PcmTrack::open(&path).unwrap().frames(), 1);
        std::fs::remove_file(path).unwrap();

        assert_eq!(
            msu_track_path("/music/alttp_msu-", 37, false),
            PathBuf::from("/music/alttp_msu-37.pcm")
        );
        assert_eq!(
            msu_track_path("/music/alttp_msu-", 37, true),
            PathBuf::from("/music/alttp_msu-37.opuz")
        );
    }

    #[test]
    fn opuz_reads_range_packet_and_finishes_with_exact_framing() {
        let path = temp_file("opuz", &opuz_file(0x8000, 0x8002));
        let packets = Rc::new(RefCell::new(Vec::new()));
        let resets = Rc::new(RefCell::new(0));
        let decoder = FakeOpus {
            packets: Rc::clone(&packets),
            resets: Rc::clone(&resets),
        };
        let mut playback = OpuzPlayback::open_with_decoder(&path, 2, 37, None, decoder).unwrap();
        let mut output = [0; 8];
        assert_eq!(playback.mix(&mut output).unwrap(), 2);
        assert_eq!(&output[..4], &[100, 200, 101, 201]);
        // A high packet-framing bit prepends the 0xfc Opus self-delimiting byte.
        assert_eq!(&*packets.borrow(), &[vec![0xfc, 0x11, 0x22]]);
        assert_eq!(*resets.borrow(), 1);
        assert_eq!(playback.mix(&mut output).unwrap(), 0);
        assert_eq!(playback.state, MSU_STATE_FINISHED_PLAYING);
        assert_eq!(playback.resume, MsuResumeInfo::default());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn opuz_range_repeat_and_snapshot_packet_verification_match_c() {
        let path = temp_file("opuz", &opuz_file(0xc000, 2));
        let packets = Rc::new(RefCell::new(Vec::new()));
        let resets = Rc::new(RefCell::new(0));
        let mut first = OpuzPlayback::open_with_decoder(
            &path,
            2,
            2,
            None,
            FakeOpus {
                packets: Rc::clone(&packets),
                resets: Rc::clone(&resets),
            },
        )
        .unwrap();
        let mut initial = [0; 2];
        assert_eq!(first.mix(&mut initial).unwrap(), 1);
        let saved = first.resume;
        assert_eq!(saved.offset, 18);
        assert_eq!(saved.samples_until_repeat, 2);

        let mut resumed = OpuzPlayback::open_with_decoder(
            &path,
            2,
            2,
            Some(saved),
            FakeOpus {
                packets: Rc::clone(&packets),
                resets: Rc::clone(&resets),
            },
        )
        .unwrap();
        assert_eq!(resumed.state, MSU_STATE_RESUMING);
        let mut after_load = [0; 6];
        assert_eq!(resumed.mix(&mut after_load).unwrap(), 3);
        assert_eq!(resumed.state, MSU_STATE_PLAYING);
        assert_eq!(&after_load, &[100, 200, 101, 201, 100, 200]);
        assert_eq!(*resets.borrow(), 2);

        let mut corrupt = saved;
        corrupt.initial_packet_bytes ^= 1;
        let mut mismatch = OpuzPlayback::open_with_decoder(
            &path,
            2,
            2,
            Some(corrupt),
            FakeOpus { packets, resets },
        )
        .unwrap();
        let error = mismatch.mix(&mut [0; 2]).unwrap_err();
        assert_eq!(error.kind, OpuzErrorKind::ResumePacketMismatch);
        assert_eq!(error.fallback_track, 2);
        assert_eq!(mismatch.state, MSU_STATE_IDLE);
        assert_eq!(mismatch.resume, MsuResumeInfo::default());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn production_opus_decoder_is_available() {
        let mut decoder = z3_opus::Decoder::new_stereo_48khz().unwrap();
        decoder.reset().unwrap();
        let mut output = [1; 960 * 2];
        assert_eq!(
            decoder.decode(&[0xf8, 0xff, 0xfe], &mut output).unwrap(),
            960
        );
        assert!(output.iter().all(|&sample| sample == 0));
    }

    #[test]
    fn runtime_opens_pcm_by_c_filename_and_falls_back_when_missing() {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let prefix =
            std::env::temp_dir().join(format!("z3-msu-prefix-{}-{sequence}-", std::process::id()));
        let track_path = msu_track_path(&prefix, 2, false);
        let mut bytes = b"MSU1".to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&11i16.to_le_bytes());
        bytes.extend_from_slice(&22i16.to_le_bytes());
        std::fs::write(&track_path, bytes).unwrap();

        let mut runtime = AudioRuntime::default();
        runtime.enable_msu(MSU_ENABLED_MSU, 100, 44_100);
        runtime
            .play_music_control_from_files(2, 0, 0, &prefix)
            .unwrap();
        assert!(runtime.pcm.is_some());
        assert_eq!(runtime.queue.current()[0], 0xf0);
        assert_eq!(runtime.read_apu(7), runtime.player.port_to_snes[3]);
        let mut wram = vec![0; 0x20_000];
        wram[APUI00_MIRROR_OFFSET] = 0x5a;
        assert_eq!(read_apui00(&wram), 0x5a);

        let error = runtime
            .play_music_control_from_files(3, 0, 0, &prefix)
            .unwrap_err();
        assert!(matches!(error, MsuFileError::Pcm(PcmFileError::Io(_))));
        assert!(runtime.pcm.is_none());
        assert_eq!(runtime.queue.current()[0], 3);
        std::fs::remove_file(track_path).unwrap();
    }

    #[test]
    fn runtime_opens_and_decodes_a_real_opuz_file() {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let prefix =
            std::env::temp_dir().join(format!("z3-opuz-prefix-{}-{sequence}-", std::process::id()));
        let track_path = msu_track_path(&prefix, 2, true);
        let mut bytes = b"OPUZ".to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&18u32.to_le_bytes());
        bytes.extend_from_slice(&960u32.to_le_bytes());
        bytes.extend_from_slice(&0x8000u16.to_le_bytes());
        bytes.extend_from_slice(&3u16.to_le_bytes());
        bytes.extend_from_slice(&[0xf8, 0xff, 0xfe]);
        std::fs::write(&track_path, bytes).unwrap();

        let mut runtime = AudioRuntime::default();
        runtime.enable_msu(MSU_ENABLED_MSU | MSU_ENABLED_OPUZ, 100, 48_000);
        runtime
            .play_music_control_from_files(2, 0, 0, &prefix)
            .unwrap();
        assert!(runtime.opuz.is_some());
        assert_eq!(runtime.queue.current()[0], 0xf0);
        let mut output = [0; 960 * 2];
        assert_eq!(
            runtime.opuz.as_mut().unwrap().mix(&mut output).unwrap(),
            960
        );
        assert!(output.iter().all(|&sample| sample == 0));
        std::fs::remove_file(track_path).unwrap();
    }

    #[test]
    fn file_backed_opuz_rotates_and_resumes_the_alternate_slot() {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let prefix =
            std::env::temp_dir().join(format!("z3-opuz-resume-{}-{sequence}-", std::process::id()));
        let mut bytes = b"OPUZ".to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&18u32.to_le_bytes());
        bytes.extend_from_slice(&960u32.to_le_bytes());
        bytes.extend_from_slice(&0x8000u16.to_le_bytes());
        bytes.extend_from_slice(&3u16.to_le_bytes());
        bytes.extend_from_slice(&[0xf8, 0xff, 0xfe]);
        let path2 = msu_track_path(&prefix, 2, true);
        let path3 = msu_track_path(&prefix, 3, true);
        std::fs::write(&path2, &bytes).unwrap();
        std::fs::write(&path3, &bytes).unwrap();

        let mut runtime = AudioRuntime::default();
        runtime.enable_msu(MSU_ENABLED_MSU | MSU_ENABLED_OPUZ, 100, 48_000);
        runtime
            .play_music_control_from_files(3, 0, 0, &prefix)
            .unwrap();
        runtime.opuz.as_mut().unwrap().mix(&mut [0; 2]).unwrap();
        let previous = runtime.opuz.as_ref().unwrap().resume;

        let alternate = MsuResumeInfo {
            tag: u32::from_le_bytes(*b"OPUZ"),
            offset: 18,
            samples_until_repeat: 960,
            initial_packet_bytes: u64::from_le_bytes([3, 0, 0xf8, 0xff, 0xfe, 0, 0, 0]),
            original_track: 2,
            actual_track: 2,
            ..MsuResumeInfo::default()
        };
        let mut wram = vec![0; 0x20_000];
        wram[MSU_RESUME_INFO_ALT_OFFSET..MSU_RESUME_INFO_ALT_OFFSET + MSU_RESUME_INFO_SIZE]
            .copy_from_slice(&alternate.to_c_bytes());
        runtime
            .play_music_control_from_files_with_resume_ram(2, 0, 0, 9, true, &mut wram, &prefix)
            .unwrap();
        let resumed = runtime.opuz.as_mut().unwrap();
        assert_eq!(resumed.state, MSU_STATE_RESUMING);
        assert_eq!(resumed.resume.actual_track, 2);
        assert_eq!(resumed.mix(&mut [0; 2]).unwrap(), 1);
        assert_eq!(resumed.state, MSU_STATE_PLAYING);
        assert_eq!(
            MsuResumeInfo::from_c_bytes(
                &wram
                    [MSU_RESUME_INFO_ALT_OFFSET..MSU_RESUME_INFO_ALT_OFFSET + MSU_RESUME_INFO_SIZE]
            ),
            Some(previous)
        );
        std::fs::remove_file(path2).unwrap();
        std::fs::remove_file(path3).unwrap();
    }

    #[test]
    fn runtime_saves_most_recent_ports_in_spc_scratch_ram() {
        let mut runtime = AudioRuntime::default();
        for (address, value) in [1, 2, 3, 4].into_iter().enumerate() {
            runtime.write_apu(address as u32, value);
        }
        runtime.save_music_state_to_ram();
        assert_eq!(&runtime.player.ram[0x410..0x414], &[1, 2, 3, 4]);
        runtime.player.input_ports = [0; 4];
        runtime.restore_music_after_load(false);
        assert_eq!(runtime.player.input_ports, [1, 2, 3, 4]);
        assert_eq!(runtime.queue.queued(), 0);
    }

    #[test]
    fn resume_info_codec_matches_the_32_byte_c_layout() {
        let info = MsuResumeInfo {
            tag: 0x0403_0201,
            offset: 0x0807_0605,
            samples_until_repeat: 0x0c0b_0a09,
            range_cur: 0x0e0d,
            range_repeat: 0x100f,
            initial_packet_bytes: 0x1817_1615_1413_1211,
            original_track: 0x19,
            actual_track: 0x1a,
        };
        let bytes = info.to_c_bytes();
        assert_eq!(&bytes[..26], &(1u8..=26).collect::<Vec<_>>());
        assert_eq!(&bytes[26..], &[0; 6]);
        assert_eq!(MsuResumeInfo::from_c_bytes(&bytes), Some(info));
        assert_eq!(MsuResumeInfo::from_c_bytes(&bytes[..31]), None);
    }

    #[test]
    fn runtime_round_trips_pcm_resume_and_transition_through_wram() {
        let mut file = b"MSU1".to_vec();
        file.extend_from_slice(&0u32.to_le_bytes());
        for sample in [1i16, 2, 3, 4, 5, 6] {
            file.extend_from_slice(&sample.to_le_bytes());
        }
        let track = PcmTrack::parse(&file).unwrap();
        let mut source = AudioRuntime::default();
        source.enable_msu(MSU_ENABLED_MSU, 100, 44_100);
        source.play_music_control(2, 0, 0, |_| Some(track.clone()));
        source.pcm.as_mut().unwrap().mix(&mut [0; 2], true);
        source.pcm.as_mut().unwrap().volume.volume = 0.5;

        let mut wram = vec![0; 0x20_000];
        wram[MUSIC_UNK1_OFFSET] = 2;
        wram[LAST_MUSIC_CONTROL_OFFSET] = 0xf2;
        source.save_msu_state_to_wram(&mut wram);
        assert_eq!(wram[MSU_VOLUME_OFFSET], 127);

        let mut restored = AudioRuntime::default();
        restored.enable_msu(MSU_ENABLED_MSU, 100, 44_100);
        restored.restore_msu_from_wram(&wram, 0, 0, |_| Some(track.clone()));
        let c_scale = restored.transition_targets[3] * (1.0 / 255.0);
        let playback = restored.pcm.unwrap();
        assert_eq!(playback.sample_position, 0);
        assert_eq!(playback.state, MSU_STATE_RESUMING);
        assert_eq!(playback.resume.original_track, 2);
        assert_eq!(playback.volume.volume, 127.0 * c_scale);
        assert_eq!(playback.volume.target, 64.0 * c_scale);
        assert_eq!(restored.queue.current()[0], 0xf0);
    }

    #[test]
    fn returning_to_overworld_uses_and_rotates_alternate_resume_slot() {
        let mut file = b"MSU1".to_vec();
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&[0; 8]);
        let track = PcmTrack::parse(&file).unwrap();
        let alternate = MsuResumeInfo {
            tag: u32::from_le_bytes(*b"MSU1"),
            offset: 1,
            original_track: 2,
            actual_track: 2,
            ..MsuResumeInfo::default()
        };
        let mut wram = vec![0; 0x20_000];
        wram[MSU_RESUME_INFO_ALT_OFFSET..MSU_RESUME_INFO_ALT_OFFSET + MSU_RESUME_INFO_SIZE]
            .copy_from_slice(&alternate.to_c_bytes());

        let mut runtime = AudioRuntime::default();
        runtime.enable_msu(MSU_ENABLED_MSU, 100, 44_100);
        runtime.play_music_control(3, 0, 0, |_| Some(track.clone()));
        let previous = runtime.pcm.as_ref().unwrap().resume;
        runtime.play_music_control_with_resume_ram(2, 0, 0, 9, true, &mut wram, |_| {
            Some(track.clone())
        });

        let playback = runtime.pcm.as_ref().unwrap();
        assert_eq!(playback.sample_position, 1);
        assert_eq!(playback.state, MSU_STATE_RESUMING);
        assert_eq!(
            MsuResumeInfo::from_c_bytes(
                &wram
                    [MSU_RESUME_INFO_ALT_OFFSET..MSU_RESUME_INFO_ALT_OFFSET + MSU_RESUME_INFO_SIZE]
            ),
            Some(previous)
        );
    }

    #[test]
    fn music_control_loads_deluxe_pcm_and_pauses_the_spc_player() {
        let mut file = b"MSU1".to_vec();
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&[0; 4]);
        let track = PcmTrack::parse(&file).unwrap();
        let mut requested = None;
        let mut runtime = AudioRuntime::default();
        runtime.enable_msu(MSU_ENABLED_MSU | MSU_ENABLED_DELUXE, 100, 44_100);
        runtime.play_music_control(2, 0, 0, |actual| {
            requested = Some(actual);
            Some(track.clone())
        });
        assert_eq!(requested, Some(37));
        assert_eq!(runtime.queue.current()[0], 0xf0);
        assert_eq!(runtime.pcm.as_ref().unwrap().resume.original_track, 2);
        assert_eq!(runtime.pcm.as_ref().unwrap().resume.actual_track, 37);

        runtime.play_music_control(0xf1, 0, 0, |_| None);
        assert_eq!(runtime.pcm.as_ref().unwrap().volume.target, 0.0);
    }

    #[test]
    fn nmi_adapter_routes_ports_and_track_queries_through_runtime_state() {
        use crate::nmi::NmiAudio;

        let mut runtime = AudioRuntime::default();
        runtime.player.port_to_snes[2] = 0x77;
        let mut adapter = NmiAudioAdapter {
            runtime: &mut runtime,
            area: 0,
            entrance: 0,
            music_unk1: 5,
            last_music_control: 4,
            misc_bug_fixes: true,
            load_pcm_track: |_| None,
        };
        assert!(adapter.is_bugged_music_track(5));
        assert_eq!(adapter.read_apu_port(2), 0x77);
        adapter.write_apu_port(3, 0x66);
        assert_eq!(adapter.runtime.queue.current()[3], 0x66);
    }
}
