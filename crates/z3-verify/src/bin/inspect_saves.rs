use std::{env, fs, process::ExitCode};

use z3_verify::{legacy_replay, legacy_snapshot};

fn main() -> ExitCode {
    let paths: Vec<String> = env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: inspect_saves SAVE...");
        return ExitCode::FAILURE;
    }

    let mut failed = false;
    for path in paths {
        let result = fs::read(&path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                let save = legacy_snapshot::parse(&bytes).map_err(|error| error.to_string())?;
                if save.to_bytes() != bytes {
                    return Err("save did not encode back to identical bytes".to_owned());
                }
                let events =
                    legacy_replay::decode_all(save.log).map_err(|error| error.to_string())?;
                println!(
                    "{path}: {} frames, {} log bytes, {} events, base snapshot: {}",
                    save.header.total_frames,
                    save.log.len(),
                    events.len(),
                    save.base_snapshot.is_some()
                );
                Ok(())
            });
        if let Err(error) = result {
            eprintln!("{path}: {error}");
            failed = true;
        }
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
