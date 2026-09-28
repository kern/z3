use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::CanonicalState;
use z3_game::{ram::Ram, select_file};
use z3_verify::first_difference;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_select_file SELECT_FILE_ORACLE ASSETS")?;
    let assets = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .ok_or("usage: check_select_file SELECT_FILE_ORACLE ASSETS")?;
    let mut child = Command::new(oracle)
        .arg(assets)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing file-select oracle stdout")?
        .read_to_end(&mut bytes)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("file-select oracle exited with {status}").into());
    }
    if bytes.len() != CanonicalState::BYTE_LEN * 2 {
        return Err(format!("unexpected file-select oracle length: {}", bytes.len()).into());
    }
    let (initial, expected) = bytes.split_at(CanonicalState::BYTE_LEN);
    let mut actual = CanonicalState::from_bytes(initial).ok_or("invalid initial oracle state")?;
    let expected = CanonicalState::from_bytes(expected).ok_or("invalid final oracle state")?;
    {
        let mut ram = Ram::new(&mut actual.wram);
        select_file::validate_sram(&mut ram, &mut actual.sram[..]);
    }
    if let Some(difference) = first_difference(&expected, &actual) {
        return Err(format!("file-select SRAM validation mismatch: {difference:?}").into());
    }
    println!("file-select SRAM repair matches compiled C across the full canonical state");
    Ok(())
}
