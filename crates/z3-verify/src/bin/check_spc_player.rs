use std::fs;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::assets::AssetFile;
use z3_game::spc_player::SpcPlayer;
use z3_snes::dsp::{DspRegWriteHistory, SAMPLE_FRAMES};

const RAM_COMPARE_SIZE: usize = 0xc000;
const SAMPLE_COUNT: usize = SAMPLE_FRAMES * 2;
const RESPONSE_SIZE: usize = RAM_COMPARE_SIZE + SAMPLE_COUNT * 2 + 4 + 256 + 256;

fn set_frame_command(player: &mut SpcPlayer, frame: usize) {
    match frame {
        1 => player.input_ports[0] = 1,
        3 => player.input_ports[1] = 1,
        5 => player.input_ports[2] = 1,
        7 => player.input_ports[3] = 1,
        _ => {}
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_spc_player ORACLE ASSETS")?;
    let assets_path = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .ok_or("usage: check_spc_player ORACLE ASSETS")?;
    let assets_bytes = fs::read(&assets_path)?;
    let assets = AssetFile::parse(&assets_bytes)?;
    let intro = assets.get(0).ok_or("missing intro sound bank")?;

    let mut child = Command::new(oracle)
        .arg(&assets_path)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut output = BufReader::new(child.stdout.take().ok_or("missing oracle stdout")?);

    let mut player = SpcPlayer::new();
    player.history = Some(DspRegWriteHistory::default());
    player
        .upload(intro)
        .map_err(|error| format!("intro sound bank upload failed: {error:?}"))?;
    player.initialize();
    player.history.as_mut().unwrap().count = 0;

    for frame in 0..10 {
        set_frame_command(&mut player, frame);
        player.generate_samples();
        let mut samples = vec![0_i16; SAMPLE_COUNT];
        player.dsp.get_samples(&mut samples, SAMPLE_FRAMES, 2);
        player.copy_variables_to_ram();

        let mut actual = Vec::with_capacity(RESPONSE_SIZE);
        actual.extend_from_slice(&player.ram[..RAM_COMPARE_SIZE]);
        for sample in samples {
            actual.extend_from_slice(&sample.to_le_bytes());
        }
        let history = player.history.as_ref().unwrap();
        actual.extend_from_slice(&history.count.to_le_bytes());
        actual.extend_from_slice(&history.addr);
        actual.extend_from_slice(&history.val);

        let mut reference = vec![0; RESPONSE_SIZE];
        output.read_exact(&mut reference)?;
        if reference != actual {
            let offset = reference
                .iter()
                .zip(&actual)
                .position(|(c, rust)| c != rust)
                .unwrap();
            return Err(format!(
                "SPC player mismatch at frame {frame}, byte {offset:#x}: C={:02x}, Rust={:02x}",
                reference[offset], actual[offset]
            )
            .into());
        }
        player.history.as_mut().unwrap().count = 0;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(format!("SPC player oracle exited with {status}").into());
    }
    println!("native SPC player matches compiled C for 10 real-bank audio frames");
    Ok(())
}
