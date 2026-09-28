use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::LegacyMemory;
use z3_game::{misc, ram::Ram};

fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_misc MISC_ORACLE ASSETS")?;
    let assets = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .ok_or("usage: check_misc MISC_ORACLE ASSETS")?;
    let mut child = Command::new(oracle)
        .arg(assets)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut reference = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing misc oracle stdout")?
        .read_to_end(&mut reference)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("misc oracle exited with {status}").into());
    }

    let mut memory = LegacyMemory::default();
    let mut actual = Vec::with_capacity(256 * 8);
    for scenario in 0..=u8::MAX {
        memory.wram_mut().fill(0);
        memory.sram_mut().fill(0);
        memory.vram_mut().fill(0);
        {
            let mut ram = Ram::new(memory.wram_mut());
            ram.write_u8(0x1a, scenario.wrapping_mul(29).wrapping_add(7));
            ram.write_u8(0xfa1, scenario.wrapping_mul(53).wrapping_add(11));
            ram.write_u16(
                0xe2,
                u16::from(scenario.wrapping_mul(17)) | (u16::from(scenario >> 3) << 8),
            );
            let random = misc::get_random_number(&mut ram);
            ram.write_u8(0x1ff0, random);
            let pan = misc::calculate_sfx_pan_arbitrary(&ram, scenario);
            ram.write_u8(0x1ff1, pan);
        }
        actual.extend_from_slice(&hash(&memory.canonical_state().to_bytes()).to_le_bytes());
    }
    if reference != actual {
        let offset = reference
            .iter()
            .zip(&actual)
            .position(|(c, rust)| c != rust)
            .unwrap_or(reference.len().min(actual.len()));
        return Err(format!(
            "misc mismatch at result byte {offset:#x} (scenario {}): C={:02x?}, Rust={:02x?}",
            offset / 8,
            reference.get(offset),
            actual.get(offset)
        )
        .into());
    }
    println!("misc RNG and arbitrary pan match compiled C across all 256 byte inputs");
    Ok(())
}
