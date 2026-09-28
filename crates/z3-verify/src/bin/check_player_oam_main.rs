use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::{LegacyMemory, WRAM_SIZE};
use z3_game::{player_oam, ram::Ram};

struct Hooks;
impl player_oam::PlayerOamHooks for Hooks {}

fn setup(ram: &mut Ram<'_>, scenario: u8) {
    ram.write_u16(0x22, 0x80);
    ram.write_u16(0x20, 0x70);
    ram.write_u8(0xf359, 0xff);
    match scenario {
        1 => {
            ram.write_u8(0x5d, 22);
            ram.write_u8(0x37d, 1);
        }
        2 => {
            ram.write_u8(0x3ef, 1);
            ram.write_u8(0x2f, 6);
            ram.write_u8(0xf359, 2);
        }
        3 => {
            ram.write_u8(0x2e0, 1);
            ram.write_u8(0x2e, 3);
            ram.write_u8(0x2f, 4);
        }
        4 => {
            ram.write_u8(0x5d, 4);
            ram.write_u8(0xf0, 1);
            ram.write_u8(0x2cc, 2);
        }
        5 => {
            ram.write_u8(0x376, 1);
            ram.write_u8(0x2f, 6);
            ram.write_u8(0x2e, 2);
        }
        6 => {
            ram.write_u8(0x5d, 5);
            ram.write_u8(0x4d, 1);
            ram.write_u8(0x34e, 0);
            ram.write_u8(0x308, 2);
        }
        7 => ram.write_u8(0x345, 1),
        8 => {
            ram.write_u8(0x11, 14);
            ram.write_u8(0x10, 1);
            ram.write_u8(0x28, 0);
        }
        9 => {
            ram.write_u8(0x11, 18);
            ram.write_u8(0x462, 4);
            ram.write_u8(0x2e, 5);
        }
        _ => {}
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_player_oam_main PLAYER_OAM_MAIN_ORACLE")?;
    let mut child = Command::new(oracle).stdout(Stdio::piped()).spawn()?;
    let mut output = BufReader::new(child.stdout.take().ok_or("missing oracle stdout")?);
    let mut memory = LegacyMemory::default();
    for scenario in 0..10_u8 {
        memory.wram_mut().fill(0);
        {
            let mut ram = Ram::new(memory.wram_mut());
            setup(&mut ram, scenario);
            player_oam::main(&mut ram, &Hooks);
        }
        let mut reference = vec![0; WRAM_SIZE];
        output.read_exact(&mut reference)?;
        if reference != memory.wram()[..] {
            let offset = reference
                .iter()
                .zip(memory.wram())
                .position(|(c, rust)| c != rust)
                .unwrap();
            return Err(format!(
                "LinkOam_Main scenario {scenario} differs at WRAM {offset:#07x}: C={:02x}, Rust={:02x}",
                reference[offset], memory.wram()[offset]
            )
            .into());
        }
    }
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("player OAM main oracle exited with {status}").into());
    }
    println!("LinkOam_Main matches compiled C across ten control-flow scenarios");
    Ok(())
}
