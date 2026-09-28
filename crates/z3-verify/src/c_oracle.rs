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
        if run_what == 0xff {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "0xff is reserved for canonical-state import",
            ));
        }
        let [low, high] = input.to_le_bytes();
        self.input.write_all(&[low, high, run_what])?;
        self.input.flush()?;

        self.read_state()
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
