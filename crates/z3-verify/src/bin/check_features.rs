use std::{env, fs, path::Path};

use z3_core::config::{
    FEATURE_BREAK_POTS_WITH_SWORD, FEATURE_CANCEL_BIRD_TRAVEL, FEATURE_CARRY_MORE_RUPEES,
    FEATURE_COLLECT_ITEMS_WITH_SWORD, FEATURE_DIM_FLASHES, FEATURE_DISABLE_LOW_HEALTH_BEEP,
    FEATURE_EXTEND_SCREEN_64, FEATURE_GAME_CHANGING_BUG_FIXES, FEATURE_MIRROR_TO_DARKWORLD,
    FEATURE_MISC_BUG_FIXES, FEATURE_MORE_ACTIVE_BOMBS, FEATURE_SHOW_MAX_ITEMS_IN_YELLOW,
    FEATURE_SKIP_INTRO_ON_KEYPRESS, FEATURE_SWITCH_LR, FEATURE_SWITCH_LR_LIMIT,
    FEATURE_TURN_WHILE_DASHING, FEATURE_WIDESCREEN_VISUAL_FIXES,
};
use z3_game::{
    assets::RuntimeAssets,
    integration::{NativeGameServices, step_integrated_raw},
    runtime::NativeRuntime,
};
use z3_snes::ppu::{RENDER_HEIGHT_240, RENDER_MODE7_4X, RENDER_NEW, RENDER_NO_SPRITE_LIMITS};
use z3_verify::{c_oracle::COracle, first_difference, legacy_snapshot};

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    features: u32,
    extra: u8,
    height: u16,
    flags: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let oracle_path = args.next().ok_or("missing stream_oracle path")?;
    let assets_path = args.next().ok_or("missing zelda3_assets.dat path")?;
    let frames = args
        .next()
        .map(|value| value.to_string_lossy().parse::<u32>())
        .transpose()?
        .unwrap_or(240);
    if frames == 0 || args.next().is_some() {
        return Err("usage: check_features STREAM_ORACLE ASSETS [FRAMES]".into());
    }

    let bytes = fs::read(&assets_path)?;
    let assets = RuntimeAssets::parse(&bytes)?;
    let messages = assets.messaging(None);
    let messaging_assets = messages.as_assets();
    let feature_bits = [
        ("item-switch-lr", FEATURE_SWITCH_LR),
        (
            "item-switch-lr-limit",
            FEATURE_SWITCH_LR | FEATURE_SWITCH_LR_LIMIT,
        ),
        ("turn-while-dashing", FEATURE_TURN_WHILE_DASHING),
        ("mirror-to-darkworld", FEATURE_MIRROR_TO_DARKWORLD),
        ("collect-items-with-sword", FEATURE_COLLECT_ITEMS_WITH_SWORD),
        ("break-pots-with-sword", FEATURE_BREAK_POTS_WITH_SWORD),
        ("disable-low-health-beep", FEATURE_DISABLE_LOW_HEALTH_BEEP),
        ("skip-intro-on-keypress", FEATURE_SKIP_INTRO_ON_KEYPRESS),
        ("show-max-items-yellow", FEATURE_SHOW_MAX_ITEMS_IN_YELLOW),
        ("more-active-bombs", FEATURE_MORE_ACTIVE_BOMBS),
        ("carry-more-rupees", FEATURE_CARRY_MORE_RUPEES),
        ("misc-bug-fixes", FEATURE_MISC_BUG_FIXES),
        ("game-changing-bug-fixes", FEATURE_GAME_CHANGING_BUG_FIXES),
        ("cancel-bird-travel", FEATURE_CANCEL_BIRD_TRAVEL),
        ("dim-flashes", FEATURE_DIM_FLASHES),
    ];
    let mut scenarios = vec![
        Scenario {
            name: "4:3-legacy",
            features: 0,
            extra: 0,
            height: 224,
            flags: 0,
        },
        Scenario {
            name: "4:3-new",
            features: 0,
            extra: 0,
            height: 224,
            flags: RENDER_NEW,
        },
        Scenario {
            name: "4:3-new-no-sprite-limit",
            features: 0,
            extra: 0,
            height: 224,
            flags: RENDER_NEW | RENDER_NO_SPRITE_LIMITS,
        },
        widescreen("16:10", 51, 224, 0),
        widescreen("16:9", 71, 224, 0),
        widescreen("18:9", 96, 224, 0),
        widescreen("16:10-extend-y", 64, 240, RENDER_HEIGHT_240),
        widescreen("16:9-extend-y", 85, 240, RENDER_HEIGHT_240),
        widescreen("18:9-extend-y", 112, 240, RENDER_HEIGHT_240),
        Scenario {
            name: "enhanced-mode7",
            features: 0,
            extra: 0,
            height: 224,
            flags: RENDER_NEW | RENDER_MODE7_4X,
        },
    ];
    scenarios.extend(feature_bits.map(|(name, features)| Scenario {
        name,
        features,
        extra: 0,
        height: 224,
        flags: RENDER_NEW,
    }));

    for scenario in scenarios {
        verify_scenario(
            scenario,
            frames,
            Path::new(&oracle_path),
            Path::new(&assets_path),
            &assets,
            &messaging_assets,
        )?;
    }
    verify_enhanced_mode7_from_save(
        180,
        Path::new(&oracle_path),
        Path::new(&assets_path),
        &assets,
        &messaging_assets,
    )?;
    println!("all enhanced-feature scenarios match the C oracle ({frames} frames each)");
    Ok(())
}

fn verify_enhanced_mode7_from_save(
    frames: u32,
    oracle_path: &Path,
    assets_path: &Path,
    assets: &RuntimeAssets<'_>,
    messaging_assets: &z3_game::messaging::MessagingAssets<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let save_path = Path::new("saves/ref/Chapter 2 - After Eastern Palace.sav");
    let save_bytes = fs::read(save_path)?;
    let save = legacy_snapshot::parse(&save_bytes)?;
    let snapshot = save.current_snapshot.as_bytes();
    let mut oracle = COracle::spawn(oracle_path, assets_path)?;
    let mut native = NativeRuntime::new();
    native.ppu.extra_left_right = 0;
    native.load_snes_state(snapshot)?;
    let imported = oracle.import_snapshot(snapshot)?;
    if let Some(difference) = first_difference(&imported, &native.canonical_state()) {
        return Err(format!("enhanced Mode7 snapshot import differs: {difference:?}").into());
    }
    let _ = oracle.configure_features(0, 0)?;
    let mut services = NativeGameServices::new(assets);
    let flags = RENDER_NEW | RENDER_MODE7_4X;
    for frame in 1..=frames {
        let input = if frame <= 4 { 0x0200 } else { 0 };
        let reference = oracle.step(input, 1)?;
        step_integrated_raw(
            &mut native,
            input,
            1,
            assets.nmi_assets(),
            assets.bytes(0),
            assets,
            messaging_assets,
            &mut services,
            |_| None,
        );
        if let Some(difference) = first_difference(&reference, &native.canonical_state()) {
            let actual = native.canonical_state();
            let nearby = reference
                .wram
                .iter()
                .zip(actual.wram.iter())
                .enumerate()
                .filter(|(_, (left, right))| left != right)
                .take(12)
                .map(|(offset, (left, right))| format!("{offset:#x}:{left:02x}/{right:02x}"))
                .collect::<Vec<_>>()
                .join(",");
            return Err(format!(
                "enhanced Mode7 save diverged from C at frame {frame}: {difference:?}; module={:02x}/{:02x}, sub={:02x}/{:02x}, map={:02x}/{:02x}, filtered={:02x}/{:02x}, item_x={:02x}/{:02x}, diffs=[{nearby}]",
                reference.wram[0x10], native.wram[0x10], reference.wram[0x11], native.wram[0x11], reference.wram[0x200], native.wram[0x200], reference.wram[0xf6], native.wram[0xf6], reference.wram[0x656], native.wram[0x656]
            )
            .into());
        }
        let scale = native.ppu.current_render_scale(flags) as u8;
        if scale == 4 {
            let expected = oracle.render(256, 224, flags, scale)?;
            native.draw(256 * usize::from(scale) * 4, flags, |_| {});
            if expected != native.ppu.pixels() {
                let offset = expected
                    .iter()
                    .zip(native.ppu.pixels())
                    .position(|(left, right)| left != right)
                    .unwrap_or(expected.len().min(native.ppu.pixels().len()));
                return Err(format!(
                    "enhanced Mode7 4x framebuffer differs at frame {frame}, byte {offset}"
                )
                .into());
            }
            println!("feature scenario enhanced-mode7-map: 4x pixels match");
            return Ok(());
        }
    }
    Err(format!("world map did not enter enhanced Mode7 within {frames} frames").into())
}

fn widescreen(name: &'static str, extra: u8, height: u16, height_flag: u8) -> Scenario {
    Scenario {
        name,
        features: FEATURE_EXTEND_SCREEN_64 | FEATURE_WIDESCREEN_VISUAL_FIXES,
        extra,
        height,
        flags: RENDER_NEW | height_flag,
    }
}

fn verify_scenario(
    scenario: Scenario,
    frames: u32,
    oracle_path: &Path,
    assets_path: &Path,
    assets: &RuntimeAssets<'_>,
    messaging_assets: &z3_game::messaging::MessagingAssets<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut oracle = COracle::spawn(oracle_path, assets_path)?;
    let mut native = NativeRuntime::new();
    native.wanted_features = scenario.features;
    native.ppu.extra_left_right = scenario.extra.min(z3_snes::ppu::EXTRA_LEFT_RIGHT as u8);
    let acknowledged = oracle.configure_features(scenario.features, native.ppu.extra_left_right)?;
    if let Some(difference) = first_difference(&acknowledged, &native.canonical_state()) {
        return Err(format!(
            "{} configuration changed state: {difference:?}",
            scenario.name
        )
        .into());
    }
    let mut services = NativeGameServices::new(assets);
    services.enhanced_features0 = scenario.features;
    for frame in 1..=frames {
        let input = if (180..184).contains(&frame) {
            0x1000
        } else {
            0
        };
        let reference = oracle.step(input, 1)?;
        step_integrated_raw(
            &mut native,
            input,
            1,
            assets.nmi_assets(),
            assets.bytes(0),
            assets,
            messaging_assets,
            &mut services,
            |_| None,
        );
        if let Some(difference) = first_difference(&reference, &native.canonical_state()) {
            return Err(format!(
                "{} diverged from C at frame {frame}: {difference:?}",
                scenario.name
            )
            .into());
        }
    }

    let width = 256_u16 + u16::from(scenario.extra) * 2;
    let scale = native.ppu.current_render_scale(scenario.flags) as u8;
    let expected = oracle.render(width, scenario.height, scenario.flags, scale)?;
    native.draw(
        usize::from(width) * usize::from(scale) * 4,
        scenario.flags,
        |_| {},
    );
    let actual = native.ppu.pixels();
    if expected != actual {
        let offset = expected
            .iter()
            .zip(actual)
            .position(|(left, right)| left != right)
            .unwrap_or(expected.len().min(actual.len()));
        return Err(format!(
            "{} rendered framebuffer differs at byte {offset} (C={}, Rust={})",
            scenario.name,
            expected.get(offset).copied().unwrap_or_default(),
            actual.get(offset).copied().unwrap_or_default(),
        )
        .into());
    }
    println!("feature scenario {}: state + pixels match", scenario.name);
    Ok(())
}
