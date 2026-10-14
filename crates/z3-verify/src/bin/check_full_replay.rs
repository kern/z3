//! Compare every recorded gameplay state and rendered pixel with the C port.

use std::{env, error::Error, fs, path::Path};

use z3_game::{
    assets::RuntimeAssets,
    host::{SaveLoadKind, save_slot_path},
    integration::{NativeGameServices, run_integrated_core_frame},
    runtime::{NativeRuntime, StateRecorder},
};
use z3_snes::{
    dsp::DSP_SAVE_BLOCK_LEN,
    ppu::{RENDER_MODE7_4X, RENDER_NEW},
};
use z3_verify::{
    c_oracle::{COracle, REPLAY_AUDIO_FRAMES},
    first_difference, legacy_snapshot,
};

fn audio_state_detail(
    oracle: &mut COracle,
    native: &mut NativeRuntime,
) -> Result<Option<String>, Box<dyn Error>> {
    let reference = oracle.export_audio_state()?;
    native.audio.player.copy_variables_to_ram();
    let rust_ram = &native.audio.player.ram;
    let ram_difference = reference
        .ram
        .iter()
        .zip(rust_ram.iter())
        .position(|(left, right)| left != right)
        .map(|index| {
            format!(
                "{index:#x}: C={:02x} Rust={:02x}",
                reference.ram[index], rust_ram[index]
            )
        });
    let mut rust_dsp = [0; DSP_SAVE_BLOCK_LEN];
    native.audio.player.dsp.write_save_block(&mut rust_dsp)?;
    let dsp_difference = reference
        .dsp
        .iter()
        .zip(rust_dsp.iter())
        .position(|(left, right)| left != right)
        .map(|index| {
            format!(
                "{index:#x}: C={:02x} Rust={:02x}",
                reference.dsp[index], rust_dsp[index]
            )
        });
    if ram_difference.is_none() && dsp_difference.is_none() {
        Ok(None)
    } else {
        Ok(Some(format!(
            "first SPC RAM difference={ram_difference:?}; first DSP save-block difference={dsp_difference:?}"
        )))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let oracle_path = args.next().ok_or("missing C oracle path")?;
    let assets_path = args.next().ok_or("missing assets path")?;
    let chapter: u16 = args
        .next()
        .ok_or("missing chapter number (1-13)")?
        .parse()?;
    if !(1..=13).contains(&chapter) || args.next().is_some() {
        return Err("usage: check_full_replay STREAM_ORACLE ASSETS CHAPTER(1-13)".into());
    }
    let save_path = save_slot_path(SaveLoadKind::Replay, 255 + chapter).unwrap();
    let save_bytes = fs::read(&save_path)?;
    let save = legacy_snapshot::parse(&save_bytes)?;
    let (recorder, base_snapshot) = StateRecorder::decode_state_file(&save_bytes, true)
        .map_err(|error| format!("invalid replay file: {error:?}"))?;
    let asset_bytes = fs::read(&assets_path)?;
    let assets = RuntimeAssets::parse(&asset_bytes)?;
    let messages = assets.messaging(None);
    let messaging_assets = messages.as_assets();
    let mut native = NativeRuntime::new();
    if base_snapshot.is_empty() {
        native.reset(false);
    } else {
        native.load_snes_state(&base_snapshot)?;
    }
    native.ppu.extra_left_right = 0;
    native.recorder = recorder;
    let mut oracle = COracle::spawn(Path::new(&oracle_path), Path::new(&assets_path))?;
    let imported = oracle.load_reference_replay(chapter - 1)?;
    let _ = oracle.configure_features(0, 0)?;
    if let Some(difference) = first_difference(&imported, &native.canonical_state()) {
        return Err(format!("chapter {chapter} initial state differs: {difference:?}").into());
    }
    let mut services = NativeGameServices::new(&assets);
    let flags = env::var("Z3_VERIFY_RENDER_FLAGS")
        .ok()
        .map(|value| value.parse::<u8>())
        .transpose()?
        .unwrap_or(RENDER_NEW | RENDER_MODE7_4X);
    let audio_only = env::var_os("Z3_VERIFY_AUDIO_ONLY").is_some();
    let audio_state_check = env::var_os("Z3_VERIFY_AUDIO_STATE").is_some();
    let mut audio_samples = [0i16; REPLAY_AUDIO_FRAMES * 2];
    for frame in 1..=save.header.total_frames {
        let prior_wram = native.wram.clone();
        let replaying = run_integrated_core_frame(
            &mut native,
            0,
            assets.nmi_assets(),
            assets.bytes(0),
            &assets,
            &messaging_assets,
            &mut services,
            |_| None,
        )
        .map_err(|error| format!("chapter {chapter} frame {frame} replay failed: {error:?}"))?;
        if audio_only {
            let state = native.canonical_state().to_bytes();
            native
                .audio
                .render_audio(&mut audio_samples, REPLAY_AUDIO_FRAMES, 2);
            let comparison = oracle.compare_replay_audio_step(&state, &audio_samples)?;
            if comparison.state_difference.is_some() || comparison.replaying != replaying {
                let reference = oracle.export_state()?;
                let difference = first_difference(&reference, &native.canonical_state());
                return Err(format!(
                    "chapter {chapter} frame {frame}: audio replay state={difference:?}, replaying C/Rust={}/{}",
                    comparison.replaying, replaying
                )
                .into());
            }
            if let Some(sample) = comparison.audio.first_sample_difference {
                let audio_state = audio_state_detail(&mut oracle, &mut native)?
                    .unwrap_or_else(|| "SPC and DSP state match".to_owned());
                return Err(format!(
                    "chapter {chapter} frame {frame}: audio sample {sample} differs: C={}, Rust={} (534 stereo samples per recorded frame); {audio_state}",
                    comparison.audio.expected_sample, comparison.audio.actual_sample,
                )
                .into());
            }
            if audio_state_check {
                let c_queue = oracle.export_audio_queue()?;
                let rust_queue = native
                    .audio
                    .queue
                    .oracle_snapshot(&native.audio.player.input_ports);
                if let Some(index) = c_queue
                    .iter()
                    .zip(rust_queue.iter())
                    .position(|(left, right)| left != right)
                {
                    return Err(format!(
                        "chapter {chapter} frame {frame}: APU queue byte {index} differs: C={:02x} Rust={:02x}; prior module={:02x} sub={:02x} room={:04x} player={:04x},{:04x} state={:02x} sword={:02x}; prior sound WRAM={:02x?}, current sound WRAM={:02x?}",
                        c_queue[index],
                        rust_queue[index],
                        prior_wram[0x10],
                        prior_wram[0x11],
                        u16::from_le_bytes([prior_wram[0xa0], prior_wram[0xa1]]),
                        u16::from_le_bytes([prior_wram[0x22], prior_wram[0x23]]),
                        u16::from_le_bytes([prior_wram[0x20], prior_wram[0x21]]),
                        prior_wram[0x5d],
                        prior_wram[0x359],
                        &prior_wram[0x12c..=0x133],
                        &native.wram[0x12c..=0x133],
                    )
                    .into());
                }
                if let Some(audio_state) = audio_state_detail(&mut oracle, &mut native)? {
                    return Err(format!(
                        "chapter {chapter} frame {frame}: APU state differs before PCM output; {audio_state}"
                    )
                    .into());
                }
            }
            if frame % 1_000 == 0 {
                println!(
                    "chapter {chapter}: {frame}/{} exact state and audio frames match",
                    save.header.total_frames
                );
            }
            continue;
        }
        let scale = native.ppu.current_render_scale(flags) as u8;
        native.draw(256 * usize::from(scale) * 4, flags, |_| {});
        let state = native.canonical_state().to_bytes();
        let comparison =
            oracle.compare_replay_frame(&state, native.ppu.pixels(), 256, 224, flags, scale)?;
        if comparison.state_difference.is_some()
            || comparison.pixel_difference.is_some()
            || comparison.replaying != replaying
        {
            let reference = oracle.export_state()?;
            let difference = first_difference(&reference, &native.canonical_state());
            let nearby = reference
                .wram
                .iter()
                .zip(native.wram.iter())
                .enumerate()
                .filter(|(_, (left, right))| left != right)
                .take(24)
                .map(|(offset, (left, right))| format!("{offset:#x}:{left:02x}/{right:02x}"))
                .collect::<Vec<_>>()
                .join(",");
            let sprite_diffs = (0xd00..0xf80)
                .filter(|&offset| reference.wram[offset] != native.wram[offset])
                .map(|offset| {
                    format!(
                        "{offset:#x}:{:02x}/{:02x}",
                        reference.wram[offset], native.wram[offset]
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let expected_pixels = oracle.render(256, 224, flags, scale)?;
            let pixel = expected_pixels
                .iter()
                .zip(native.ppu.pixels())
                .position(|(left, right)| left != right);
            let pixel_detail = pixel.map(|index| {
                let mismatches = expected_pixels.iter().zip(native.ppu.pixels()).enumerate().filter_map(|(i, (left, right))| (left != right).then_some(i)).collect::<Vec<_>>();
                format!(
                    "index={index}, C={:08x}, Rust={:08x}, mismatching_pixels={}, first_coords={:?}",
                    expected_pixels[index],
                    native.ppu.pixels()[index],
                    mismatches.len(),
                    mismatches.iter().take(20).map(|i| (i % (256 * usize::from(scale)), i / (256 * usize::from(scale)))).collect::<Vec<_>>()
                )
            });
            let cgram = oracle.export_cgram()?;
            let sprite_summary = |wram: &[u8]| {
                (0..16)
                    .filter(|&slot| wram[0xdd0 + slot] != 0)
                    .map(|slot| {
                        let x = u16::from_le_bytes([wram[0xd10 + slot], wram[0xd30 + slot]]);
                        let y = u16::from_le_bytes([wram[0xd00 + slot], wram[0xd20 + slot]]);
                        format!(
                            "{slot}:t{:02x}s{:02x}@{x:04x},{y:04x}",
                            wram[0xe20 + slot],
                            wram[0xdd0 + slot]
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            let c_sprites = sprite_summary(&reference.wram[..]);
            let rust_sprites = sprite_summary(&native.wram[..]);
            let ancilla_summary = |wram: &[u8]| {
                (0..5)
                    .filter(|&slot| wram[0xc4a + slot] != 0)
                    .map(|slot| {
                        format!(
                            "{slot}:t{:02x}step{:02x}timer{:02x}",
                            wram[0xc4a + slot],
                            wram[0xc54 + slot],
                            wram[0xc68 + slot]
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            let oam_dump = (0x800..0xa40)
                .step_by(4)
                .filter(|&address| {
                    (0x830..0x838).contains(&address)
                        || reference.wram[address..address + 4] != native.wram[address..address + 4]
                })
                .map(|address| {
                    format!(
                        "{address:#x}:C{:02x?}/R{:02x?}",
                        &reference.wram[address..address + 4],
                        &native.wram[address..address + 4]
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            let cgram_differences = cgram
                .iter()
                .zip(native.ppu.cgram.iter())
                .enumerate()
                .filter(|(_, (left, right))| left != right)
                .take(20)
                .map(|(index, (left, right))| {
                    let offset = 0xc300 + index * 2;
                    let main =
                        u16::from_le_bytes(native.wram[offset..offset + 2].try_into().unwrap());
                    let aux = u16::from_le_bytes(
                        native.wram[offset + 0x200..offset + 0x202]
                            .try_into()
                            .unwrap(),
                    );
                    format!("{index:02x}:{left:04x}/{right:04x} main={main:04x} aux={aux:04x}")
                })
                .collect::<Vec<_>>()
                .join(",");
            let vram_differences = reference
                .vram
                .iter()
                .zip(native.ppu.vram.iter())
                .enumerate()
                .filter(|(_, (left, right))| left != right)
                .take(24)
                .map(|(index, (left, right))| format!("{index:04x}:{left:04x}/{right:04x}"))
                .collect::<Vec<_>>()
                .join(",");
            let dungeon_map_differences = (0x2000..0x6000)
                .filter(|&offset| reference.wram[offset] != native.wram[offset])
                .take(24)
                .map(|offset| {
                    format!(
                        "{offset:#x}:{:02x}/{:02x}",
                        reference.wram[offset], native.wram[offset]
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let dungeon_bg1_differences = (0x4000..0x6000)
                .filter(|&offset| reference.wram[offset] != native.wram[offset])
                .take(12)
                .map(|offset| {
                    format!(
                        "{offset:#x}:{:02x}/{:02x}",
                        reference.wram[offset], native.wram[offset]
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            return Err(format!(
                "chapter {chapter} frame {frame}: state={difference:?}, pixel={pixel_detail:?}, replaying C/Rust={}/{}; prior module={:02x} sub={:02x} subsub={:02x} room={:04x} a5={:02x} pos={:04x},{:04x} vel={:02x},{:02x} old={:04x},{:04x} scratch={:02x} audio={:02x?} ancilla=[{}]; module={:02x} sub={:02x} subsub={:02x} room C/R={:04x}/{:04x} entrance={:04x} player_state={:02x} auxiliary={:02x} z={:04x} z_vel={:02x} input={:04x}; audio C={:02x?} Rust={:02x?} ancilla C=[{}] Rust=[{}]; WRAM diffs=[{nearby}]; dungeon map diffs=[{dungeon_map_differences}] BG1 diffs=[{dungeon_bg1_differences}]; sprite WRAM diffs=[{sprite_diffs}]; OAM dump=[{oam_dump}]; CGRAM diffs=[{cgram_differences}]; VRAM diffs=[{vram_differences}]; sprites C=[{c_sprites}] Rust=[{rust_sprites}]; save={}",
                comparison.replaying,
                replaying,
                prior_wram[0x10],
                prior_wram[0x11],
                prior_wram[0xb0],
                u16::from_le_bytes([prior_wram[0xa0], prior_wram[0xa1]]),
                prior_wram[0xa5],
                u16::from_le_bytes([prior_wram[0x22], prior_wram[0x23]]),
                u16::from_le_bytes([prior_wram[0x20], prior_wram[0x21]]),
                prior_wram[0x31],
                prior_wram[0x30],
                u16::from_le_bytes([prior_wram[0x2dc], prior_wram[0x2dd]]),
                u16::from_le_bytes([prior_wram[0x2de], prior_wram[0x2df]]),
                prior_wram[0x74],
                &prior_wram[0x12c..=0x133],
                ancilla_summary(&prior_wram[..]),
                native.wram[0x10],
                native.wram[0x11],
                native.wram[0xb0],
                u16::from_le_bytes([reference.wram[0xa0], reference.wram[0xa1]]),
                u16::from_le_bytes([native.wram[0xa0], native.wram[0xa1]]),
                u16::from_le_bytes([native.wram[0x10e], native.wram[0x10f]]),
                native.wram[0x5d],
                native.wram[0x4d],
                u16::from_le_bytes([native.wram[0x24], native.wram[0x25]]),
                native.wram[0x29],
                native.recorder.last_inputs,
                &reference.wram[0x12c..=0x133],
                &native.wram[0x12c..=0x133],
                ancilla_summary(&reference.wram[..]),
                ancilla_summary(&native.wram[..]),
                save_path.display()
            )
            .into());
        }
        native
            .audio
            .render_audio(&mut audio_samples, REPLAY_AUDIO_FRAMES, 2);
        let audio_comparison = oracle.compare_replay_audio(&audio_samples)?;
        if let Some(sample) = audio_comparison.first_sample_difference {
            let audio_state = audio_state_detail(&mut oracle, &mut native)?
                .unwrap_or_else(|| "SPC and DSP state match".to_owned());
            return Err(format!(
                "chapter {chapter} frame {frame}: audio sample {sample} differs: C={}, Rust={} (534 stereo samples per recorded frame); {audio_state}",
                audio_comparison.expected_sample, audio_comparison.actual_sample,
            )
            .into());
        }
        if frame % 1_000 == 0 {
            println!(
                "chapter {chapter}: {frame}/{} exact state, pixel, and audio frames match",
                save.header.total_frames
            );
        }
    }
    if audio_only {
        println!(
            "chapter {chapter}: all {} recorded frames match the C state and audio samples exactly",
            save.header.total_frames
        );
    } else {
        println!(
            "chapter {chapter}: all {} recorded frames match the C state, rendered pixels, and audio samples exactly",
            save.header.total_frames
        );
    }
    Ok(())
}
