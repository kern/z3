use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::{LegacyMemory, WRAM_SIZE};
use z3_game::{overlord, ram::Ram};

fn run(ram: &mut Ram<'_>, scenario: u8) {
    match scenario {
        0 => {
            overlord::set_x(ram, 3, 0x1234);
            overlord::set_y(ram, 3, 0xabcd);
            ram.write_u16(0x1ff0, overlord::get_x(ram, 3));
            ram.write_u16(0x1ff2, overlord::get_y(ram, 3));
        }
        1 => overlord::pot_trap(ram, 2),
        2 => overlord::position_target(ram, 3),
        3 => overlord::check_if_active(ram, 1),
        4 => ram.write_u8(
            0x1ff0,
            u8::from(overlord::armos_coordinator_check_knights(ram)),
        ),
        5 => overlord::armos_coordinator_disable_coercion(ram),
        _ => unreachable!(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_overlord OVERLORD_ORACLE")?;
    let mut child = Command::new(oracle).stdout(Stdio::piped()).spawn()?;
    let mut output = BufReader::new(child.stdout.take().ok_or("missing oracle stdout")?);
    let mut memory = LegacyMemory::default();
    for scenario in 0..6_u8 {
        output.read_exact(memory.wram_mut())?;
        let mut expected = vec![0; WRAM_SIZE];
        output.read_exact(&mut expected)?;
        {
            let mut ram = Ram::new(memory.wram_mut());
            run(&mut ram, scenario);
        }
        if expected != memory.wram()[..] {
            let offset = expected
                .iter()
                .zip(memory.wram())
                .position(|(c, rust)| c != rust)
                .unwrap();
            return Err(format!(
                "overlord scenario {scenario} differs at WRAM {offset:#07x}: C={:02x}, Rust={:02x}",
                expected[offset],
                memory.wram()[offset]
            )
            .into());
        }
    }
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("overlord oracle exited with {status}").into());
    }
    println!("overlord state helpers match compiled C across six full-WRAM scenarios");
    Ok(())
}
