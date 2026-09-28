use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::{LegacyMemory, WRAM_SIZE};
use z3_game::{ram::Ram, tagalong};

fn run(ram: &mut Ram<'_>, scenario: u8) {
    match scenario {
        0 => ram.write_u8(0x1ff0, u8::from(tagalong::is_following(ram))),
        1 => ram.write_u8(0x1ff0, u8::from(tagalong::validate_message_freedom(ram))),
        2 => ram.write_u8(0x1ff0, u8::from(tagalong::check_blind_trigger(ram))),
        3 | 4 => {
            let result = tagalong::check_proximity_to_link(ram);
            ram.write_u8(0x1ff0, u8::from(result));
        }
        5 => tagalong::disable(ram),
        _ => unreachable!(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_tagalong TAGALONG_ORACLE")?;
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
                "tagalong scenario {scenario} differs at WRAM {offset:#07x}: C={:02x}, Rust={:02x}",
                expected[offset],
                memory.wram()[offset]
            )
            .into());
        }
    }
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("tagalong oracle exited with {status}").into());
    }
    println!("tagalong state helpers match compiled C across six full-WRAM scenarios");
    Ok(())
}
