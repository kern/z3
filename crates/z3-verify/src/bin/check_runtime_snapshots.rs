//! Cross-check production snapshot views against the independent verifier codec.
use std::{error::Error, fs};
use z3_game::{
    host::{SaveLoadKind, save_slot_path},
    runtime::NativeRuntime,
    snapshot::{Snapshot, SnapshotRegion},
};
use z3_verify::legacy_snapshot;

fn check(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let reference = legacy_snapshot::Snapshot::parse(bytes)?;
    let view = Snapshot::parse(bytes)?;
    // This parser and its offsets stay independent of the production schema.
    for (region, expected) in [
        (SnapshotRegion::Prefix, reference.prefix_opaque()),
        (SnapshotRegion::SpcRam, reference.spc_ram()),
        (SnapshotRegion::BeforeDsp, reference.pre_dsp_opaque()),
        (SnapshotRegion::Dsp, reference.dsp_block()),
        (SnapshotRegion::BeforeDma, reference.pre_dma_opaque()),
        (SnapshotRegion::Dma, reference.dma_block()),
        (SnapshotRegion::Ppu, reference.ppu().as_bytes()),
        (SnapshotRegion::Sram, reference.sram()),
        (SnapshotRegion::BeforeWram, reference.pre_wram_opaque()),
        (SnapshotRegion::Wram, reference.wram()),
        (SnapshotRegion::Suffix, reference.suffix_opaque()),
    ] {
        if view.region(region) != expected {
            return Err(format!("runtime snapshot schema differs for {region:?}").into());
        }
    }
    let mut runtime = NativeRuntime::new();
    let mut input = bytes.to_vec();
    runtime.internal_save_load(&mut input, true)?;
    if input != bytes {
        return Err("snapshot input mutated during import".into());
    }
    if runtime.wram.as_slice() != reference.wram()
        || runtime.sram.as_slice() != reference.sram()
        || runtime.audio.player.ram.as_slice() != reference.spc_ram()
        || runtime
            .ppu
            .vram
            .iter()
            .enumerate()
            .any(|(index, &word)| Some(word) != reference.ppu().vram_word(index))
    {
        return Err("snapshot imported into incorrect live memory".into());
    }
    let mut output = vec![0xa5; bytes.len()];
    runtime.internal_save_load(&mut output, false)?;
    if let Some(offset) = output.iter().zip(bytes).position(|(a, b)| a != b) {
        return Err(format!("snapshot round trip differs at {offset:#x}").into());
    }
    // A live change must override the template, not re-export stale bytes.
    runtime.wram[0x1234] ^= 0x5a;
    runtime.sram[0x456] ^= 0xa5;
    runtime.ppu.vram[0x2345] ^= 0x55aa;
    runtime.audio.player.ram[0x5678] ^= 0x3c;
    runtime.internal_save_load(&mut output, false)?;
    let changed = legacy_snapshot::Snapshot::parse(&output)?;
    if changed.wram()[0x1234] != reference.wram()[0x1234] ^ 0x5a
        || changed.sram()[0x456] != reference.sram()[0x456] ^ 0xa5
        || changed.spc_ram()[0x5678] != reference.spc_ram()[0x5678] ^ 0x3c
        || changed.ppu().vram_word(0x2345)
            != reference.ppu().vram_word(0x2345).map(|word| word ^ 0x55aa)
    {
        return Err("live snapshot mutation was hidden by the template".into());
    }
    let mut expected = bytes.to_vec();
    expected[144_389 + 0x1234] ^= 0x5a;
    expected[136_139 + 0x456] ^= 0xa5;
    expected[27 + 0x5678] ^= 0x3c;
    expected[68_834 + 0x2345 * 2] ^= 0xaa;
    expected[68_834 + 0x2345 * 2 + 1] ^= 0x55;
    if output != expected {
        return Err("live snapshot mutation changed unrelated or opaque bytes".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for chapter in 1..=13 {
        let path = save_slot_path(SaveLoadKind::Replay, 255 + chapter).ok_or("replay path")?;
        let bytes = fs::read(path)?;
        let container = legacy_snapshot::parse(&bytes)?;
        for image in container
            .base_snapshot
            .iter()
            .chain(std::iter::once(&container.current_snapshot))
        {
            check(image.as_bytes())?;
            let mut patched = image.as_bytes().to_vec();
            // Perturb only historical outer gaps, not interpreted device bytes.
            for (start, end) in [
                (0, 27),
                (65_563, 65_603),
                (68_627, 68_642),
                (144_331, 144_389),
                (275_461, 275_465),
            ] {
                for (index, byte) in patched[start..end].iter_mut().enumerate() {
                    *byte = (index as u8).wrapping_mul(37).wrapping_add(chapter as u8);
                }
            }
            check(&patched)?;
            count += 1;
        }
    }
    println!(
        "runtime snapshot schema, complete round trips, live mutations and opaque-byte variants match the independent codec for {count} reference images"
    );
    Ok(())
}
