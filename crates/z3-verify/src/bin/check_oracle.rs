use std::env;
use std::path::Path;

use z3_verify::{c_oracle::COracle, first_difference};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let executable = args.next().ok_or("missing stream_oracle path")?;
    let assets = args.next().ok_or("missing zelda3_assets.dat path")?;
    if args.next().is_some() {
        return Err("usage: check_oracle STREAM_ORACLE ASSETS".into());
    }

    let mut first = COracle::spawn(Path::new(&executable), Path::new(&assets))?;
    let mut second = COracle::spawn(Path::new(&executable), Path::new(&assets))?;
    for frame in 1..=600_u32 {
        let input = if (180..184).contains(&frame) {
            0x1000
        } else {
            0
        };
        let reference = first.step(input, 1)?;
        let repeated = second.step(input, 1)?;
        if let Some(difference) = first_difference(&reference, &repeated) {
            return Err(format!("oracle diverged at frame {frame}: {difference:?}").into());
        }
    }
    println!("C oracle stream is deterministic through frame 600");
    Ok(())
}
