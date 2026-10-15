//! Asset-free differential checks for dungeon map addressing and 2x2 updates.

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

use z3_core::WRAM_SIZE;
use z3_game::dungeon::{Dungeon, DungeonHooks};

type Error = Box<dyn std::error::Error>;

struct Hooks;
impl DungeonHooks for Hooks {}

fn check(child: &mut Child) -> Result<usize, Error> {
    let mut input = child.stdin.take().ok_or("missing tile map oracle stdin")?;
    let mut output = child
        .stdout
        .take()
        .ok_or("missing tile map oracle stdout")?;
    let mut conversions = vec![0; 0x10000 * 4];
    output.read_exact(&mut conversions)?;
    for position in 0..=u16::MAX {
        let offset = usize::from(position) * 4;
        let actual = [
            Dungeon::<Hooks>::dungeon_map_vram_addr_no_swap(position),
            Dungeon::<Hooks>::dungeon_map_vram_addr(position),
        ];
        for (variant, actual) in actual.into_iter().enumerate() {
            let start = offset + variant * 2;
            let expected = u16::from_le_bytes(conversions[start..start + 2].try_into()?);
            if actual != expected {
                return Err(format!("VRAM address mismatch at position {position:#06x}, variant {variant}: C={expected:#06x}, Rust={actual:#06x}").into());
            }
        }
    }

    let mut initial = Box::new([0; WRAM_SIZE]);
    let mut actual = Box::new([0; WRAM_SIZE]);
    let mut reference = Box::new([0; WRAM_SIZE]);
    let mut count = 0;
    for position in [
        0u16, 1, 0x1e, 0x1f, 0x3e, 0x3f, 0x40, 0x3ff, 0x7ff, 0xfbe, 0xfff, 0x1000, 0x1fbe, 0x1fff,
        0x7fbe, 0xdfbe,
    ] {
        // All upload buffers end below the map itself. This preserves the C
        // helper's normal disjoint-buffer cases while testing both
        // sides of tile row, map page, layer, and image boundaries.
        for upload in [0u16, 2, 24, 0xfe0] {
            for tiles in [[0u16, 0xffff, 0xab12, 0x80ff], [0x100, 0x200, 0x300, 0x400]] {
                for (address, byte) in initial.iter_mut().enumerate() {
                    *byte = (address as u8).wrapping_mul(37).wrapping_add(count as u8);
                }
                initial[0x1000..0x1002].copy_from_slice(&upload.to_le_bytes());
                actual.copy_from_slice(initial.as_slice());
                let attribute = (count as u8).wrapping_mul(3);
                let mut request = [0; 11];
                request[..2].copy_from_slice(&position.to_le_bytes());
                for (slot, tile) in tiles.iter().enumerate() {
                    request[2 + slot * 2..4 + slot * 2].copy_from_slice(&tile.to_le_bytes());
                }
                request[10] = attribute;
                input.write_all(&request)?;
                input.write_all(initial.as_slice())?;
                input.flush()?;
                output.read_exact(reference.as_mut_slice())?;
                Dungeon::new(&mut actual, &mut Hooks).dungeon_store_2x2(position, tiles, attribute);
                if let Some(address) = actual
                    .iter()
                    .zip(reference.iter())
                    .position(|(r, c)| r != c)
                {
                    return Err(format!("2x2 map mismatch case {count}, position {position:#06x}, upload {upload:#06x} at WRAM {address:#07x}: C={:02x}, Rust={:02x}", reference[address], actual[address]).into());
                }
                count += 1;
            }
        }
    }
    Ok(count)
}

fn main() -> Result<(), Error> {
    let oracle = std::env::args_os()
        .nth(1)
        .ok_or("usage: check_tile_maps TILE_MAP_ORACLE")?;
    let mut child = Command::new(oracle)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let result = check(&mut child);
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait()?;
    let count = result?;
    if !status.success() {
        return Err(format!("tile map oracle exited with {status}").into());
    }
    println!(
        "dungeon tile maps match compiled C: 131072 VRAM address conversions and {count} complete WRAM 2x2 updates"
    );
    Ok(())
}
