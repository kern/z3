use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_core::{LegacyMemory, WRAM_SIZE};
use z3_game::player_oam::{
    SwordResult, calculate_sword_hit_box, find_most_significant_bit, set_equipment_vram_offsets,
    set_weapon_vram_offsets, want_invoke_sword,
};
use z3_game::ram::Ram;

fn push_offsets(actual: &mut Vec<u8>, ram: &Ram<'_>) {
    actual.push(ram.read_u8(0x45));
    actual.push(ram.read_u8(0x44));
}

fn push_vram_result(actual: &mut Vec<u8>, done: bool, result: SwordResult, ram: &Ram<'_>) {
    actual.extend_from_slice(&[
        u8::from(done),
        result.r6,
        result.r12,
        ram.read_u8(0x107),
        ram.read_u8(0x108),
        ram.read_u8(0x109),
    ]);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_player_oam PLAYER_OAM_ORACLE")?;
    let mut child = Command::new(oracle).stdout(Stdio::piped()).spawn()?;
    let mut reference = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing player OAM oracle stdout")?
        .read_to_end(&mut reference)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("player OAM oracle exited with {status}").into());
    }

    let mut actual = Vec::with_capacity(reference.len());
    actual.extend((0..=u8::MAX).map(find_most_significant_bit));
    let mut memory = LegacyMemory::default();
    for sword in 0..5_u8 {
        for direction in (0..8_u8).step_by(2) {
            for frame in 0..21_u8 {
                memory.wram_mut().fill(0);
                let mut ram = Ram::new(memory.wram_mut());
                ram.write_u8(0xf359, sword);
                ram.write_u8(0x2f, direction);
                ram.write_u8(0x3c, frame);
                ram.write_u8(0x45, 0xa5);
                ram.write_u8(0x44, 0x5a);
                calculate_sword_hit_box(&mut ram);
                push_offsets(&mut actual, &ram);
            }
        }
    }

    for item in [0_u8, 5] {
        for index in 0..511 {
            memory.wram_mut().fill(0);
            let mut ram = Ram::new(memory.wram_mut());
            ram.write_u8(0x301, item);
            ram.write_u8(0x107, 0x31);
            ram.write_u8(0x108, 0x42);
            ram.write_u8(0x109, 0x53);
            let mut result = SwordResult {
                r6: 0xa5,
                r12: 0x5a,
            };
            if item == 0 {
                let done = set_weapon_vram_offsets(&mut ram, index, &mut result);
                push_vram_result(&mut actual, done, result, &ram);
            }

            ram.write_u8(0x107, 0x31);
            ram.write_u8(0x108, 0x42);
            ram.write_u8(0x109, 0x53);
            result = SwordResult {
                r6: 0xa5,
                r12: 0x5a,
            };
            let done = set_equipment_vram_offsets(&mut ram, index, &mut result);
            push_vram_result(&mut actual, done, result, &ram);
        }
    }

    for scenario in 0..1024_u16 {
        memory.wram_mut().fill(0);
        let mut ram = Ram::new(memory.wram_mut());
        ram.write_u8(0x5d, (scenario & 7) as u8);
        ram.write_u8(0x301, ((scenario >> 1) & 0xff) as u8);
        ram.write_u8(0x37a, ((scenario >> 3) & 0x3f) as u8);
        ram.write_u8(0x3a, if scenario & 1 != 0 { 0x80 } else { 0 });
        ram.write_u8(0xf359, ((scenario >> 4) & 3) as u8);
        ram.write_u8(0x308, ((scenario >> 7) & 1) as u8);
        ram.write_u8(0x3ef, ((scenario >> 8) & 1) as u8);
        ram.write_u8(0x360, ((scenario >> 9) & 1) as u8);
        actual.push(u8::from(want_invoke_sword(&ram)));
    }

    if reference != actual {
        let offset = reference
            .iter()
            .zip(&actual)
            .position(|(c, rust)| c != rust)
            .unwrap_or(reference.len().min(actual.len()));
        let c = reference.get(offset).copied();
        let rust = actual.get(offset).copied();
        return Err(format!(
            "player OAM mismatch at byte {offset:#x}: C={c:02x?}, Rust={rust:02x?} (C len {}, Rust len {})",
            reference.len(),
            actual.len()
        )
        .into());
    }
    assert_eq!(memory.wram().len(), WRAM_SIZE);
    println!(
        "player OAM helpers match compiled C across {} result bytes",
        actual.len()
    );
    Ok(())
}
