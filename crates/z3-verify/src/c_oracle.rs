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
        if matches!(run_what, 0xfc..=0xff) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "0xfc..=0xff are reserved for oracle control commands",
            ));
        }
        let [low, high] = input.to_le_bytes();
        self.input.write_all(&[low, high, run_what])?;
        self.input.flush()?;

        self.read_state()
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

impl Drop for COracle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
