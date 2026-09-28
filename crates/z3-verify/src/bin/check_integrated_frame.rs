use std::{env, fs, path::Path};

use z3_core::CanonicalState;
use z3_game::{
    assets::RuntimeAssets,
    integration::{NativeGameServices, step_integrated_raw},
    runtime::NativeRuntime,
};
use z3_verify::{c_oracle::COracle, first_difference, legacy_snapshot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let oracle_path = args.next().ok_or("missing stream_oracle path")?;
    let assets_path = args.next().ok_or("missing zelda3_assets.dat path")?;
    let third = args.next();
    let (frames, save_path, run_what) = match third.as_deref() {
        Some(value) if value == "--save" => {
            let path = args.next().ok_or("--save requires a .sav path")?;
            let frames = args
                .next()
                .map(|value| value.to_string_lossy().parse::<u32>())
                .transpose()?
                .unwrap_or(1);
            let run_what = args
                .next()
                .map(|value| value.to_string_lossy().parse::<u8>())
                .transpose()?
                .unwrap_or(1);
            (frames, Some(path), run_what)
        }
        value => (
            value
                .map(|value| value.to_string_lossy().parse::<u32>())
                .transpose()?
                .unwrap_or(1),
            None,
            1,
        ),
    };
    if frames == 0 || args.next().is_some() {
        return Err(
            "usage: check_integrated_frame STREAM_ORACLE ASSETS [FRAMES]\n       check_integrated_frame STREAM_ORACLE ASSETS --save PATH [FRAMES] [RUN_WHAT]".into(),
        );
    }
    let bytes = fs::read(&assets_path)?;
    let assets = RuntimeAssets::parse(&bytes)?;
    let messages = assets.messaging(None);
    let messaging_assets = messages.as_assets();
    let mut oracle = COracle::spawn(Path::new(&oracle_path), Path::new(&assets_path))?;
    let mut native = NativeRuntime::new();
    if let Some(save_path) = save_path.as_deref() {
        let save_bytes = fs::read(save_path)?;
        let save = legacy_snapshot::parse(&save_bytes)?;
        let initial = canonical_from_snapshot(save.current_snapshot);
        let imported = oracle.import_state(&initial)?;
        if let Some(difference) = first_difference(&initial, &imported) {
            return Err(format!("C oracle changed imported save state: {difference:?}").into());
        }
        native.import_canonical_state(&initial);
    }
    // Differential parity must exercise the same concrete service composition
    // as the native executable.  The old `FirstFrameServices` harness was a
    // useful narrow bring-up tool, but most of its trait implementations used
    // default no-op callbacks and therefore could not validate production
    // cross-module routing.
    let mut services = NativeGameServices::new(&assets);
    let mut previous = native.canonical_state();
    for frame in 1..=frames {
        let input = if save_path.is_none() && (180..184).contains(&frame) {
            0x1000
        } else {
            0
        };
        let reference = oracle.step(input, run_what)?;
        step_integrated_raw(
            &mut native,
            input,
            run_what,
            assets.nmi_assets(),
            assets.bytes(0),
            &assets,
            &messaging_assets,
            &mut services,
            |_| None,
        );
        let actual = native.canonical_state();
        if let Some(difference) = first_difference(&reference, &actual) {
            let nearby = reference
                .wram
                .iter()
                .zip(actual.wram.iter())
                .enumerate()
                .filter(|(_, (left, right))| left != right)
                .take(24)
                .map(|(offset, (&left, &right))| format!("{offset:#07x}:{left:02x}/{right:02x}"))
                .collect::<Vec<_>>()
                .join(", ");
            let oam_context = (0..128usize)
                .filter(|&entry| {
                    let offset = entry * 4;
                    reference.wram[0x800 + offset..0x800 + offset + 4]
                        != actual.wram[0x800 + offset..0x800 + offset + 4]
                        || reference.wram[0xa00 + entry / 4] != actual.wram[0xa00 + entry / 4]
                })
                .take(24)
                .map(|entry| {
                    let offset = 0x800 + entry * 4;
                    format!(
                        "#{entry}: {:02x?}/{:02x?},ext={:02x}/{:02x}",
                        &reference.wram[offset..offset + 4],
                        &actual.wram[offset..offset + 4],
                        reference.wram[0xa00 + entry / 4],
                        actual.wram[0xa00 + entry / 4],
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let sprites = (0..16usize)
                .filter_map(|slot| {
                    let state = actual.wram[0xdd0 + slot];
                    let reference_state = reference.wram[0xdd0 + slot];
                    (state != 0 || reference_state != 0 || previous.wram[0xdd0 + slot] != 0).then(|| {
                        let previous_x = u16::from_le_bytes([
                            previous.wram[0xd10 + slot],
                            previous.wram[0xd30 + slot],
                        ]);
                        let previous_y = u16::from_le_bytes([
                            previous.wram[0xd00 + slot],
                            previous.wram[0xd20 + slot],
                        ]);
                        let floor = previous.wram[0xf20 + slot];
                        let tile_index = usize::from((previous_x & 0x1f8) >> 3)
                            + usize::from((previous_y & 0x1f8) << 3)
                            + if floor >= 1 { 0x1000 } else { 0 };
                        format!(
                            "{slot}:state={state:02x}/{reference_state:02x},type={:02x}/{:02x},ai={:02x}/{:02x},delay={:02x}/{:02x},z={:02x}/{:02x},xy={:02x}{:02x}/{:02x}{:02x}|{:02x}{:02x}/{:02x}{:02x},pre_state={:02x},pre_xy={previous_x:04x}/{previous_y:04x},floor={floor:02x},g={:02x},gfx={:02x},sub={:02x},sub2={:02x},flags2={:02x},flags3={:02x}/{:02x}(pre={:02x}),objprio={:02x},tile[{tile_index:04x}]={:02x}",
                            actual.wram[0xe20 + slot],
                            reference.wram[0xe20 + slot],
                            actual.wram[0xd80 + slot],
                            reference.wram[0xd80 + slot],
                            actual.wram[0xdf0 + slot],
                            reference.wram[0xdf0 + slot],
                            actual.wram[0xf70 + slot],
                            reference.wram[0xf70 + slot],
                            actual.wram[0xd30 + slot],
                            actual.wram[0xd10 + slot],
                            actual.wram[0xd20 + slot],
                            actual.wram[0xd00 + slot],
                            reference.wram[0xd30 + slot],
                            reference.wram[0xd10 + slot],
                            reference.wram[0xd20 + slot],
                            reference.wram[0xd00 + slot],
                            previous.wram[0xdd0 + slot],
                            previous.wram[0xed0 + slot],
                            previous.wram[0xdc0 + slot],
                            previous.wram[0xe30 + slot],
                            previous.wram[0xe90 + slot],
                            previous.wram[0xe40 + slot],
                            actual.wram[0xe60 + slot],
                            reference.wram[0xe60 + slot],
                            previous.wram[0xe60 + slot],
                            previous.wram[0xf60 + slot],
                            previous.wram[0x12000 + tile_index],
                        )
                    })
                })
                .collect::<Vec<_>>()
                .join(", ");
            let garnishes = (0..30usize)
                .filter_map(|slot| {
                    let kind = actual.wram[0x1f800 + slot];
                    let reference_kind = reference.wram[0x1f800 + slot];
                    (kind != 0 || reference_kind != 0).then(|| {
                        format!(
                            "{slot}:type={kind:02x}/{reference_kind:02x},count={:02x}/{:02x},xy={:02x}/{:02x},sprite={:02x},floor={:02x},flags={:02x}",
                            actual.wram[0x1f90e + slot],
                            reference.wram[0x1f90e + slot],
                            actual.wram[0x1f83c + slot],
                            actual.wram[0x1f81e + slot],
                            actual.wram[0x1f92c + slot],
                            actual.wram[0x1f968 + slot],
                            actual.wram[0x1f9fe + slot],
                        )
                    })
                })
                .collect::<Vec<_>>()
                .join(", ");
            let ancillas = (0..10usize)
                .filter_map(|slot| {
                    let kind = actual.wram[0xc4a + slot];
                    let reference_kind = reference.wram[0xc4a + slot];
                    let previous_kind = previous.wram[0xc4a + slot];
                    (kind != 0 || reference_kind != 0 || previous_kind != 0).then(|| {
                        format!(
                            "{slot}:type={kind:02x}/{reference_kind:02x}(pre={previous_kind:02x})"
                        )
                    })
                })
                .collect::<Vec<_>>()
                .join(", ");
            let overlords = (0..8usize)
                .filter_map(|slot| {
                    let kind = actual.wram[0xb00 + slot];
                    let reference_kind = reference.wram[0xb00 + slot];
                    let previous_kind = previous.wram[0xb00 + slot];
                    (kind != 0 || reference_kind != 0 || previous_kind != 0).then(|| {
                        format!(
                            "{slot}:type={kind:02x}/{reference_kind:02x}(pre={previous_kind:02x})"
                        )
                    })
                })
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "integrated Rust frame {frame} diverged: {difference:?}; WRAM diffs [{nearby}]; OAM [{oam_context}]; regions pre={:04x?}/{:04x?},ref={:04x?}/{:04x?},rust={:04x?}/{:04x?}; sprites [{sprites}]; garnishes [{garnishes}]; ancillas [{ancillas}]; overlords [{overlords}]",
                (0..6).map(|i| u16::from_le_bytes([previous.wram[0xfe0 + i * 2], previous.wram[0xfe1 + i * 2]])).collect::<Vec<_>>(),
                (0..6).map(|i| u16::from_le_bytes([previous.wram[0xfec + i * 2], previous.wram[0xfed + i * 2]])).collect::<Vec<_>>(),
                (0..6).map(|i| u16::from_le_bytes([reference.wram[0xfe0 + i * 2], reference.wram[0xfe1 + i * 2]])).collect::<Vec<_>>(),
                (0..6).map(|i| u16::from_le_bytes([reference.wram[0xfec + i * 2], reference.wram[0xfed + i * 2]])).collect::<Vec<_>>(),
                (0..6).map(|i| u16::from_le_bytes([actual.wram[0xfe0 + i * 2], actual.wram[0xfe1 + i * 2]])).collect::<Vec<_>>(),
                (0..6).map(|i| u16::from_le_bytes([actual.wram[0xfec + i * 2], actual.wram[0xfed + i * 2]])).collect::<Vec<_>>(),
            )
            .into());
        }
        previous = actual;
    }
    if let Some(path) = save_path {
        println!(
            "integrated Rust matches the C oracle from {} through {frames} frame(s)",
            Path::new(&path).display()
        );
    } else {
        println!("integrated Rust matches the C oracle byte-for-byte through frame {frames}");
    }
    Ok(())
}

fn canonical_from_snapshot(snapshot: legacy_snapshot::Snapshot<'_>) -> CanonicalState {
    let mut bytes = Vec::with_capacity(CanonicalState::BYTE_LEN);
    bytes.extend_from_slice(snapshot.wram());
    bytes.extend_from_slice(snapshot.sram());
    bytes.extend_from_slice(snapshot.ppu().vram());
    CanonicalState::from_bytes(&bytes).expect("validated legacy snapshot has canonical state sizes")
}
