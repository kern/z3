use std::{env, fs, process::ExitCode};

use z3_core::assets::AssetFile;
use z3_game::{assets::RuntimeAssets, overworld::OverworldAssets};

fn main() -> ExitCode {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "zelda3_assets.dat".to_owned());
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let assets = match AssetFile::parse(&bytes) {
        Ok(assets) => assets,
        Err(error) => {
            eprintln!("invalid asset bundle {path}: {error:?}");
            return ExitCode::FAILURE;
        }
    };
    let runtime = match RuntimeAssets::parse(&bytes) {
        Ok(assets) => assets,
        Err(error) => {
            eprintln!("invalid typed runtime assets {path}: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    // Exercise both byte and decoded signed/unsigned word views used by the
    // native gameplay translation, not just the container index itself.
    let messaging = runtime.messaging(None);
    let messaging = messaging.as_assets();
    let typed_entries = runtime.map16_to_map8().len()
        + runtime.entrance_areas().len()
        + runtime.exit_rooms().len()
        + runtime.special_exit_up_target().len()
        + runtime.dungeon_object_data(0).len()
        + runtime.dungeon_room_door_info(0).len()
        + runtime.dungeon_room_header(0).len()
        + runtime.dungeon_room_layout(0).len()
        + runtime.default_room_layout(0).len()
        + runtime.dungeon_room_chests().len()
        + runtime.dungeon_secrets().len()
        + messaging.dictionary.len()
        + messaging.messages.len()
        + messaging.font_tiles.len()
        + messaging.font_widths.len();
    let _ = runtime.dungeon_movable_block(0);
    let _ = runtime.dungeon_torch_data(0);
    let _ = runtime.dungeon_entrance(false, 0);
    let _ = runtime.dungeon_entrance(true, 0);
    if typed_entries == 0 {
        eprintln!("runtime asset views are unexpectedly empty");
        return ExitCode::FAILURE;
    }

    let payload_bytes: usize = assets.iter().map(|asset| asset.len()).sum();
    println!(
        "{path}: {} assets, {} key bytes, {payload_bytes} payload bytes, {typed_entries} checked typed entries",
        assets.len(),
        assets.key_table().len()
    );
    ExitCode::SUCCESS
}
