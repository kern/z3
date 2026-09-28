use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::{LegacyMemory, WRAM_SIZE};
use z3_game::{poly, ram::Ram};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_poly POLY_ORACLE")?;
    let mut child = Command::new(oracle).stdout(Stdio::piped()).spawn()?;

    let mut memory = LegacyMemory::default();
    memory.wram_mut().fill(0xaa);
    {
        let mut ram = Ram::new(memory.wram_mut());
        ram.write_u8(0x1f02, 0);
        ram.write_u8(0x1f03, 0);
        ram.write_u8(0x1f04, 0);
        ram.write_u8(0x1f05, 0);
        ram.write_u8(0x1f06, 64);
        ram.write_u8(0x1f07, 64);
        poly::run_frame(&mut ram);
    }

    let mut reference = vec![0; WRAM_SIZE];
    child
        .stdout
        .take()
        .ok_or("missing poly oracle stdout")?
        .read_exact(&mut reference)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("poly oracle exited with {status}").into());
    }
    if reference != memory.wram()[..] {
        let offset = reference
            .iter()
            .zip(memory.wram().iter())
            .position(|(c, rust)| c != rust)
            .unwrap();
        return Err(format!(
            "poly state mismatch at WRAM {offset:#07x}: C={:02x}, Rust={:02x}",
            reference[offset],
            memory.wram()[offset]
        )
        .into());
    }
    println!("polyhedral frame matches compiled C across full WRAM");
    Ok(())
}
