//! Safe process boundary for the C reference oracle.
//!
//! Keeping C outside the Rust address space means the actual port remains
//! `unsafe`-free. The protocol is deliberately tiny: a three-byte frame
//! command goes in and the canonical WRAM/SRAM/VRAM byte image comes back.

use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use z3_core::CanonicalState;

pub struct COracle {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayFrameComparison {
    pub state_difference: Option<usize>,
    pub pixel_difference: Option<usize>,
    pub replaying: bool,
}

pub const REPLAY_AUDIO_FRAMES: usize = 534;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayAudioComparison {
    pub first_sample_difference: Option<usize>,
    pub expected_sample: i16,
    pub actual_sample: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayAudioStepComparison {
    pub state_difference: Option<usize>,
    pub audio: ReplayAudioComparison,
    pub replaying: bool,
}

pub const AUDIO_STATE_RAM_LEN: usize = 65536;
pub const AUDIO_STATE_DSP_LEN: usize = 3024;

pub struct OracleAudioState {
    pub ram: Vec<u8>,
    pub dsp: Vec<u8>,
}

impl COracle {
    pub fn spawn(executable: &Path, assets: &Path) -> io::Result<Self> {
        let mut child = Command::new(executable)
            .arg(assets)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("C oracle stdin was not piped"))?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("C oracle stdout was not piped"))?;
        Ok(Self {
            child,
            input: BufWriter::new(input),
            output: BufReader::new(output),
        })
    }

    pub fn step(&mut self, input: u16, run_what: u8) -> io::Result<CanonicalState> {
        if matches!(run_what, 0xf8..=0xff) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "0xf8..=0xff are reserved for oracle control commands",
            ));
        }
        let [low, high] = input.to_le_bytes();
        self.input.write_all(&[low, high, run_what])?;
        self.input.flush()?;

        self.read_state()
    }

    pub fn load_reference_replay(&mut self, chapter: u16) -> io::Result<CanonicalState> {
        let [low, high] = chapter.to_le_bytes();
        self.input.write_all(&[low, high, 0xfb])?;
        self.input.flush()?;
        self.read_state()
    }

    pub fn export_state(&mut self) -> io::Result<CanonicalState> {
        self.input.write_all(&[0, 0, 0xf8])?;
        self.input.flush()?;
        self.read_state()
    }

    pub fn export_cgram(&mut self) -> io::Result<[u16; 256]> {
        self.input.write_all(&[0, 0, 0xf7])?;
        self.input.flush()?;
        let mut bytes = [0u8; 512];
        self.output.read_exact(&mut bytes)?;
        Ok(std::array::from_fn(|index| {
            u16::from_le_bytes([bytes[index * 2], bytes[index * 2 + 1]])
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_replay_frame(
        &mut self,
        state: &[u8],
        pixels: &[u8],
        width: u16,
        height: u16,
        flags: u8,
        scale: u8,
    ) -> io::Result<ReplayFrameComparison> {
        let expected_pixels =
            usize::from(width) * usize::from(height) * usize::from(scale) * usize::from(scale) * 4;
        if state.len() != CanonicalState::BYTE_LEN
            || pixels.len() != expected_pixels
            || !matches!(scale, 1 | 4)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid replay frame dimensions",
            ));
        }
        let [width_low, width_high] = width.to_le_bytes();
        let [height_low, height_high] = height.to_le_bytes();
        self.input.write_all(&[width_low, width_high, 0xfa])?;
        self.input
            .write_all(&[height_low, height_high, flags, scale])?;
        self.input.write_all(state)?;
        self.input.write_all(pixels)?;
        self.input.flush()?;
        let mut response = [0; 12];
        self.output.read_exact(&mut response)?;
        let difference = |offset| {
            let value = u32::from_le_bytes(response[offset..offset + 4].try_into().unwrap());
            (value != u32::MAX).then_some(value as usize)
        };
        Ok(ReplayFrameComparison {
            state_difference: difference(0),
            pixel_difference: difference(4),
            replaying: u32::from_le_bytes(response[8..12].try_into().unwrap()) != 0,
        })
    }

    /// Advances the C APU by one game frame and compares every interleaved
    /// stereo PCM sample with the native Rust audio output.
    pub fn compare_replay_audio(&mut self, samples: &[i16]) -> io::Result<ReplayAudioComparison> {
        validate_replay_audio(samples)?;
        self.input.write_all(&[0, 0, 0xf9])?;
        write_replay_audio(&mut self.input, samples)?;
        self.input.flush()?;
        let mut response = [0; 8];
        self.output.read_exact(&mut response)?;
        Ok(decode_replay_audio(&response))
    }

    /// Steps the recorded C game frame and compares state plus exact PCM,
    /// avoiding the expensive pixel render used by the full video gate.
    pub fn compare_replay_audio_step(
        &mut self,
        state: &[u8],
        samples: &[i16],
    ) -> io::Result<ReplayAudioStepComparison> {
        if state.len() != CanonicalState::BYTE_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid replay state length",
            ));
        }
        validate_replay_audio(samples)?;
        self.input.write_all(&[0, 0, 0xf6])?;
        self.input.write_all(state)?;
        write_replay_audio(&mut self.input, samples)?;
        self.input.flush()?;
        let mut response = [0; 16];
        self.output.read_exact(&mut response)?;
        let difference = u32::from_le_bytes(response[..4].try_into().unwrap());
        Ok(ReplayAudioStepComparison {
            state_difference: (difference != u32::MAX).then_some(difference as usize),
            audio: decode_replay_audio(&response[4..12]),
            replaying: u32::from_le_bytes(response[12..16].try_into().unwrap()) != 0,
        })
    }

    /// Captures the C player's mapped SPC RAM and raw DSP save block for
    /// diagnosing a PCM mismatch. The DSP layout matches legacy `.sav` data.
    pub fn export_audio_state(&mut self) -> io::Result<OracleAudioState> {
        self.input.write_all(&[0, 0, 0xf5])?;
        self.input.flush()?;
        let mut bytes = vec![0; AUDIO_STATE_RAM_LEN + AUDIO_STATE_DSP_LEN];
        self.output.read_exact(&mut bytes)?;
        Ok(OracleAudioState {
            ram: bytes[..AUDIO_STATE_RAM_LEN].to_vec(),
            dsp: bytes[AUDIO_STATE_RAM_LEN..].to_vec(),
        })
    }

    /// Current C audio write queue: 4-byte pending ports, 16 four-byte ring
    /// entries, position/count/total, then four current SPC input ports.
    pub fn export_audio_queue(&mut self) -> io::Result<[u8; 75]> {
        self.input.write_all(&[0, 0, 0xf4])?;
        self.input.flush()?;
        let mut snapshot = [0; 75];
        self.output.read_exact(&mut snapshot)?;
        Ok(snapshot)
    }

    /// Imports the complete C-compatible device snapshot embedded in a `.sav`
    /// file, including PPU registers/CGRAM/OAM needed for render comparison.
    pub fn import_snapshot(&mut self, snapshot: &[u8]) -> io::Result<CanonicalState> {
        let length = u32::try_from(snapshot.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "snapshot is too large"))?;
        self.input.write_all(&[0, 0, 0xfc])?;
        self.input.write_all(&length.to_le_bytes())?;
        self.input.write_all(snapshot)?;
        self.input.flush()?;
        self.read_state()
    }

    /// Selects the same enhanced gameplay and widescreen settings used by the
    /// native hosts. The oracle returns its unchanged canonical state as an
    /// acknowledgement so protocol errors cannot go unnoticed.
    pub fn configure_features(
        &mut self,
        features: u32,
        extra_left_right: u8,
    ) -> io::Result<CanonicalState> {
        self.input.write_all(&[0, 0, 0xfe])?;
        self.input.write_all(&features.to_le_bytes())?;
        self.input.write_all(&[extra_left_right])?;
        self.input.flush()?;
        self.read_state()
    }

    /// Renders the current C PPU state to the same tightly packed ARGB byte
    /// layout consumed by the SDL/OpenGL frontends.
    pub fn render(&mut self, width: u16, height: u16, flags: u8, scale: u8) -> io::Result<Vec<u8>> {
        if !matches!(scale, 1 | 4) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "render scale must be one or four",
            ));
        }
        let [width_low, width_high] = width.to_le_bytes();
        let [height_low, height_high] = height.to_le_bytes();
        self.input.write_all(&[width_low, width_high, 0xfd])?;
        self.input
            .write_all(&[height_low, height_high, flags, scale])?;
        self.input.flush()?;
        let mut pixels = vec![
            0;
            usize::from(width)
                * usize::from(height)
                * usize::from(scale)
                * usize::from(scale)
                * 4
        ];
        self.output.read_exact(&mut pixels)?;
        Ok(pixels)
    }

    /// Replaces the oracle's pointer-free game state without restarting it.
    /// The returned state is the C side's immediate round trip of the import.
    pub fn import_state(&mut self, state: &CanonicalState) -> io::Result<CanonicalState> {
        self.input.write_all(&[0, 0, 0xff])?;
        self.input.write_all(&state.to_bytes())?;
        self.input.flush()?;

        self.read_state()
    }

    fn read_state(&mut self) -> io::Result<CanonicalState> {
        let mut bytes = vec![0; CanonicalState::BYTE_LEN];
        self.output.read_exact(&mut bytes)?;
        CanonicalState::from_bytes(&bytes)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid C oracle state"))
    }
}

fn validate_replay_audio(samples: &[i16]) -> io::Result<()> {
    if samples.len() != REPLAY_AUDIO_FRAMES * 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "replay audio must be exactly 534 stereo frames",
        ));
    }
    Ok(())
}

fn write_replay_audio(output: &mut impl Write, samples: &[i16]) -> io::Result<()> {
    for sample in samples {
        output.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}

fn decode_replay_audio(response: &[u8]) -> ReplayAudioComparison {
    let offset = u32::from_le_bytes(response[..4].try_into().unwrap());
    ReplayAudioComparison {
        first_sample_difference: (offset != u32::MAX).then_some(offset as usize),
        expected_sample: i16::from_le_bytes(response[4..6].try_into().unwrap()),
        actual_sample: i16::from_le_bytes(response[6..8].try_into().unwrap()),
    }
}

impl Drop for COracle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
