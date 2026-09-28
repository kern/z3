use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::LegacyMemory;
use z3_game::{ram::Ram, tile_detect};

fn hash_ram(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_tile_detect TILE_DETECT_ORACLE")?;
    let mut child = Command::new(oracle).stdout(Stdio::piped()).spawn()?;
    let mut reference = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing tile detection oracle stdout")?
        .read_to_end(&mut reference)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("tile detection oracle exited with {status}").into());
    }

    let mut memory = LegacyMemory::default();
    let mut actual = Vec::with_capacity(256 * 4 * 8);
    for tile in 0..=u8::MAX {
        for scenario in 0..4_u8 {
            memory.wram_mut().fill(0);
            {
                let mut ram = Ram::new(memory.wram_mut());
                ram.write_u8(0xffc, scenario & 1);
                ram.write_u16(0x402, if scenario & 2 != 0 { 0x8000 } else { 0 });
                ram.write_u8(0x2f5, u8::from(scenario & 1 != 0));
                ram.write_u8(0xef, u8::from(scenario & 2 != 0));
                tile_detect::execute_inner(
                    &mut ram,
                    tile,
                    0x100 + usize::from(tile) * 2,
                    1_u16 << scenario,
                    scenario & 1 != 0,
                );
            }
            actual.extend_from_slice(&hash_ram(memory.wram()).to_le_bytes());
        }
    }

    if reference != actual {
        let offset = reference
            .iter()
            .zip(&actual)
            .position(|(c, rust)| c != rust)
            .unwrap_or(reference.len().min(actual.len()));
        return Err(format!(
            "tile detection mismatch at result byte {offset:#x} (tile {:#04x}, scenario {}): C={:02x?}, Rust={:02x?}",
            offset / 32,
            (offset % 32) / 8,
            reference.get(offset),
            actual.get(offset)
        )
        .into());
    }
    println!("all 256 tile behaviors match compiled C across four state scenarios");
    Ok(())
}
