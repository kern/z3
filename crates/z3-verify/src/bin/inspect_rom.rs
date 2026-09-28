use std::env;
use std::fs;

use z3_snes::system::Snes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args_os().nth(1).ok_or("usage: inspect_rom ROM")?;
    let bytes = fs::read(path)?;
    let mut snes = Snes::new();
    let info = snes.load_rom(&bytes)?;
    println!(
        "mapping={} name={:?} expanded-rom={} pal={}",
        info.mapping, info.name, info.rom_size, info.pal
    );
    println!(
        "reset-vector={:02x}{:02x} pc={:04x}",
        snes.read(0x00fffd),
        snes.read(0x00fffc),
        snes.cpu.pc
    );
    Ok(())
}
