use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::CanonicalState;
use z3_game::{
    hud::{Hud, HudRenderOp, HudRuntime},
    ram::Ram,
};
use z3_verify::first_difference;

struct NoExternalCalls;

impl HudRuntime for NoExternalCalls {
    fn render(&mut self, _operation: HudRenderOp, _ram: &mut Ram<'_>) {
        panic!("Hud_Rebuild unexpectedly requested a VRAM upload");
    }

    fn display_select_menu(&mut self, _ram: &mut Ram<'_>) {
        panic!("Hud_Rebuild unexpectedly opened the select menu");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_hud HUD_ORACLE ASSETS")?;
    let assets = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .ok_or("usage: check_hud HUD_ORACLE ASSETS")?;
    let mut child = Command::new(oracle)
        .arg(assets)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing HUD oracle stdout")?
        .read_to_end(&mut bytes)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("HUD oracle exited with {status}").into());
    }
    if bytes.len() != CanonicalState::BYTE_LEN * 2 {
        return Err(format!("unexpected HUD oracle length: {}", bytes.len()).into());
    }
    let (initial, expected) = bytes.split_at(CanonicalState::BYTE_LEN);
    let mut actual = CanonicalState::from_bytes(initial).ok_or("invalid initial HUD state")?;
    let expected = CanonicalState::from_bytes(expected).ok_or("invalid final HUD state")?;
    let mut runtime = NoExternalCalls;
    {
        let mut ram = Ram::new(&mut actual.wram);
        Hud {
            ram: &mut ram,
            runtime: &mut runtime,
        }
        .rebuild();
    }
    if let Some(difference) = first_difference(&expected, &actual) {
        return Err(format!("HUD rebuild mismatch: {difference:?}").into());
    }
    println!("HUD rebuild matches compiled C across the full canonical state");
    Ok(())
}
