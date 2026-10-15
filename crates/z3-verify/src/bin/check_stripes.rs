//! Asset-free complete-VRAM differential checks for the legacy stripe codec.
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use z3_game::nmi::handle_stripes14;
use z3_snes::ppu::Ppu;

fn append(stream: &mut Vec<u8>, destination: u16, vertical: bool, fill: bool, len: u16, seed: u8) {
    stream.extend_from_slice(&destination.to_be_bytes());
    let flags = (u16::from(vertical) << 15) | (u16::from(fill) << 14);
    stream.extend_from_slice(&(flags | (len - 1)).to_be_bytes());
    let payload_len = if fill { 2 } else { len };
    stream.extend((0..payload_len).map(|i| seed.wrapping_add((i as u8).wrapping_mul(37))));
}

fn cases() -> Vec<Vec<u8>> {
    let mut cases = Vec::new();
    for seed in 0..=u8::MAX {
        let mut stream = Vec::new();
        let destination = u16::from(seed) * 31;
        let vertical = seed & 1 != 0;
        let fill = seed & 2 != 0;
        let len = if vertical && !fill {
            2 * (u16::from(seed % 31) + 1)
        } else {
            u16::from(seed % 63) + 1
        };
        append(&mut stream, destination, vertical, fill, len, seed);
        // Overwrite part of the previous transfer to check sequence order.
        append(&mut stream, destination + 1, false, false, 3, seed ^ 0xa5);
        stream.push(0x80 | (seed & 0x7f));
        cases.push(stream);
    }
    for destination in [0, 1, 0x7fe0, 0x7ffe, 0x7fff] {
        for vertical in [false, true] {
            for fill in [false, true] {
                let mut stream = Vec::new();
                // One word is in range even at the final VRAM address.
                append(
                    &mut stream,
                    destination,
                    vertical,
                    fill,
                    if fill { 1 } else { 2 },
                    0x81,
                );
                stream.push(0xff);
                cases.push(stream);
            }
        }
    }
    cases
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle = std::env::args_os()
        .nth(1)
        .ok_or("usage: check_stripes STRIPE_ORACLE")?;
    let mut child = Command::new(oracle)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let result = (|| -> Result<usize, Box<dyn std::error::Error>> {
        let mut input = child.stdin.take().ok_or("missing oracle stdin")?;
        let mut output = child.stdout.take().ok_or("missing oracle stdout")?;
        let mut ppu = Ppu::new();
        let mut initial = vec![0; 0x10000];
        let mut expected = vec![0; initial.len()];
        let cases = cases();
        for (index, stream) in cases.iter().enumerate() {
            for (word_index, word) in ppu.vram.iter_mut().enumerate() {
                *word = (word_index as u16)
                    .wrapping_mul(0x125)
                    .wrapping_add(index as u16);
                initial[word_index * 2..word_index * 2 + 2].copy_from_slice(&word.to_le_bytes());
            }
            input.write_all(&(stream.len() as u32).to_le_bytes())?;
            input.write_all(&initial)?;
            input.write_all(stream)?;
            input.flush()?;
            output.read_exact(&mut expected)?;
            handle_stripes14(&mut ppu, stream)?;
            for (word_index, pair) in expected.chunks_exact(2).enumerate() {
                let reference = u16::from_le_bytes([pair[0], pair[1]]);
                if ppu.vram[word_index] != reference {
                    return Err(format!("stripe case {index} differs at VRAM word {word_index:#06x}: C={reference:04x}, Rust={:04x}", ppu.vram[word_index]).into());
                }
            }
        }
        Ok(cases.len())
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait()?;
    let count = result?;
    if !status.success() {
        return Err(format!("stripe oracle exited with {status}").into());
    }
    println!(
        "stripe copy/fill and row/column decoding matches compiled C across {count} complete VRAM images"
    );
    Ok(())
}
