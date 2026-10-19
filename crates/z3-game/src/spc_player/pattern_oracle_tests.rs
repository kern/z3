//! Complete-state differential for live patterns, bookmarks and KOF probes.
//! C byte postincrements wrap; C KOF's local int does not. Every C lookahead
//! program below terminates before the end of RAM; Rust-only wrap tests follow.

use super::*;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
use z3_snes::dsp::DSP_SAVE_BLOCK_LEN;

const RAM_LEN: usize = 65536;
const RESPONSE_LEN: usize = 1 + RAM_LEN * 2 + DSP_SAVE_BLOCK_LEN + 4 + 512;

struct Oracle {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
    response: Vec<u8>,
    cases: usize,
}

impl Oracle {
    fn spawn() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut child = Command::new(root.join("build/c-oracle/spc_pattern_oracle"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("build spc_pattern_oracle first");
        Self {
            input: BufWriter::new(child.stdin.take().unwrap()),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            response: vec![0; RESPONSE_LEN],
            cases: 0,
        }
    }

    fn check(
        &mut self,
        mut player: SpcPlayer,
        operation: u8,
        ci: usize,
        arg: u8,
        context: impl std::fmt::Debug,
    ) {
        self.input.write_all(&[operation, ci as u8, arg]).unwrap();
        self.input.write_all(player.ram.as_slice()).unwrap();
        self.input
            .write_all(player.mapped_ram_snapshot().as_slice())
            .unwrap();
        self.input.flush().unwrap();
        let result = match operation {
            0 => {
                player.handle_effect(ci, arg);
                0
            }
            1 => {
                player.pitch_slide_to_note_check(ci);
                0
            }
            2 => {
                let before = player.clone();
                let result = u8::from(player.want_write_kof(ci));
                assert_eq!(before, player, "KOF probe mutated state: {context:?}");
                result
            }
            3 => {
                player.new_value_from_snes[0] = arg;
                player.port0_handle_music();
                0
            }
            _ => unreachable!(),
        };
        self.output.read_exact(&mut self.response).unwrap();
        let mut offset = 0;
        let mut compare = |label: &str, actual: &[u8]| {
            let expected = &self.response[offset..offset + actual.len()];
            if expected != actual {
                let first = expected
                    .iter()
                    .zip(actual)
                    .position(|(a, b)| a != b)
                    .unwrap();
                panic!(
                    "{context:?} op={operation} channel={ci}: {label}[{first:#x}] C={:#x} Rust={:#x}",
                    expected[first], actual[first]
                );
            }
            offset += actual.len();
        };
        compare("return", &[result]);
        compare("raw RAM", player.ram.as_slice());
        compare("mapped RAM", player.mapped_ram_snapshot().as_slice());
        let mut dsp = [0; DSP_SAVE_BLOCK_LEN];
        player.dsp.write_save_block(&mut dsp).unwrap();
        compare("DSP", &dsp);
        let history = player.history.as_ref().unwrap();
        compare("history count", &history.count.to_le_bytes());
        compare("history addresses", &history.addr);
        compare("history values", &history.val);
        assert_eq!(offset, RESPONSE_LEN);
        self.cases += 1;
    }
}

impl Drop for Oracle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn base(ci: usize, address: u16) -> SpcPlayer {
    let mut player = SpcPlayer {
        history: Some(DspRegWriteHistory::default()),
        ..SpcPlayer::default()
    };
    player.ram.fill(0xa5);
    player.cur_chan_bit = 1 << ci;
    player.master_volume = 0xc034;
    player.tempo = 0x2345;
    player.port_to_snes[0] = 1;
    let channel = &mut player.channels[ci];
    channel.pattern_order_ptr_for_chan = PatternCursor::at(address);
    channel.saved_pattern_ptr = PatternAddress::at(0x5000);
    channel.pattern_start_ptr = PatternAddress::at(0x6000);
    channel.instrument_id = 1;
    channel.instrument_pitch_base = 0x0200;
    channel.pitch = 0x3456;
    channel.note_ticks_left = 1;
    channel.note_length = 2;
    channel.channel_volume = 0x8045;
    channel.pan_value = 0x0a00;
    player
}

fn bytes(player: &mut SpcPlayer, start: u16, data: &[u8]) {
    for (offset, &byte) in data.iter().enumerate() {
        player.ram[usize::from(start.wrapping_add(offset as u16))] = byte;
    }
}

#[test]
#[ignore = "requires built C oracle; complete-state static handler differential"]
fn pattern_cursors_match_static_c_handlers() {
    let mut oracle = Oracle::spawn();
    // All positions and all voices exercise the live byte-read/advance path,
    // including ff..→00 wrap, independently of the high-byte activation guard.
    for ci in 0..8 {
        let template = base(ci, 0);
        for address in 0..=u16::MAX {
            let mut player = template.clone();
            player.channels[ci].pattern_order_ptr_for_chan = PatternCursor::at(address);
            bytes(&mut player, address, &[0xf9, 0x12, 0x34, 0x56]);
            player.global_transposition = 0x93;
            player.channels[ci].channel_transposition = 0x81;
            oracle.check(player, 1, ci, 0, ("pitch byte wrap", address));
        }
    }
    let all_position_cases = oracle.cases;
    assert_eq!(all_position_cases, 8 * 65536);

    // Every supported effect with safe parameters. Direct calls deliberately
    // avoid executing the unimplemented tremolo tick after effect eb sets it.
    for ci in 0..8 {
        for effect in 0xe0..=0xfau8 {
            if matches!(effect, 0xf1..=0xf3) {
                continue;
            }
            for address in [0, 1, 0x4000, 0xfffd, 0xfffe, 0xffff] {
                for first in [0, 1, 0x7f, 0x80, 0xff] {
                    let mut player = base(ci, address);
                    bytes(
                        &mut player,
                        address,
                        &[first, 0x26, if effect == 0xf7 { 3 } else { 0x43 }],
                    );
                    player.ram[0x3d00..0x4300].fill(0);
                    // Keep cursor operands intact if they overlap the tables.
                    bytes(
                        &mut player,
                        address,
                        &[first, 0x26, if effect == 0xf7 { 3 } else { 0x43 }],
                    );
                    oracle.check(player, 0, ci, effect, ("effect", effect, address, first));
                }
            }
        }
        // All EF loop counts, destinations including low/inactive and ffff,
        // and all operand-end wrap variants. No target dereference in this op.
        for loops in 0..=255u8 {
            for address in [0x4000, 0xfffd, 0xfffe, 0xffff] {
                for target in [0, 0xff, 0x100, 0xffff] {
                    let mut player = base(ci, address);
                    let [low, high] = (target as u16).to_le_bytes();
                    bytes(&mut player, address, &[low, high, loops]);
                    oracle.check(
                        player,
                        0,
                        ci,
                        0xef,
                        ("EF bookmarks", loops, address, target),
                    );
                }
            }
        }
        for address in [0, 0xfffe, 0xffff] {
            for existing_slide in [0, 1, 0xff] {
                for token in [0, 0xf9, 0xff] {
                    for owned in [false, true] {
                        let mut player = base(ci, address);
                        player.channels[ci].pitch_slide_length = existing_slide;
                        player.is_chan_on = if owned { 1 << ci } else { 0 };
                        bytes(&mut player, address, &[token, 1, 0, 0x80]);
                        oracle.check(
                            player,
                            1,
                            ci,
                            0,
                            ("pitch guard", address, existing_slide, token, owned),
                        );
                    }
                }
            }
        }
        // Live zero-token returns use every loop count, including 0→phrase
        // loading and 1→return; the remaining counts restart the loop bookmark.
        for loops in 0..=255u8 {
            let mut player = base(ci, 0x4000);
            player.channels[ci].subroutine_num_loops = loops;
            bytes(&mut player, 0x4000, &[0]);
            bytes(&mut player, 0x5000, &[2, 0xc9, 0]);
            bytes(&mut player, 0x6000, &[3, 0xc9, 0]);
            player.music_ptr_toplevel = PhraseCursor::at(0x800);
            bytes(&mut player, 0x800, &[0, 0]);
            oracle.check(player, 3, ci, 0, ("live return", loops));
        }
        // A nested call intentionally overwrites the one-deep return slot.
        let mut player = base(ci, 0x4000);
        bytes(&mut player, 0x4000, &[0xef, 0, 0x50, 2, 2, 0xc9, 0]);
        bytes(&mut player, 0x5000, &[0xef, 0, 0x60, 1, 2, 0xc9, 0]);
        bytes(&mut player, 0x6000, &[2, 0xc9, 0]);
        oracle.check(player, 3, ci, 0, "nested one-deep EF");

        // Every low-page address is inactive to the music scheduler, not only0.
        for address in 0..=0xff {
            oracle.check(base(ci, address), 3, ci, 0, ("inactive", address));
        }
        // KOF uses local loop state and never installs EF's repeat operand.
        for loops in 0..=255u8 {
            for tail in [0, 0xc8, 0xc9, 0x90] {
                let mut player = base(ci, 0x4000);
                player.channels[ci].subroutine_num_loops = loops;
                bytes(&mut player, 0x4000, &[0]);
                bytes(&mut player, 0x5000, &[tail]);
                bytes(&mut player, 0x6000, &[0xc8]);
                oracle.check(player, 2, ci, 0, ("KOF return", loops, tail));
            }
        }
        for effect in 0xe0..=0xfau8 {
            for terminal in [0xc8, 0xc9] {
                let mut player = base(ci, 0x4000);
                if effect == 0xef {
                    bytes(&mut player, 0x4000, &[0xef, 0, 0x50, 0xff]);
                    bytes(&mut player, 0x5000, &[terminal]);
                } else {
                    let length = usize::from(EFFECT_BYTE_LENGTH[usize::from(effect - 0xe0)]);
                    bytes(&mut player, 0x4000, &[1, 2, effect]);
                    player.ram[0x4003..0x4003 + length].fill(0x24);
                    player.ram[0x4003 + length] = terminal;
                }
                oracle.check(player, 2, ci, 0, ("KOF skip", effect, terminal));
            }
        }
        for start in [0xfffc, 0xfffd, 0xfffe, 0xffff] {
            let mut player = base(ci, start);
            bytes(&mut player, start, &[0xc8]);
            oracle.check(player, 2, ci, 0, ("KOF last byte", start));
        }
    }
    println!(
        "pattern C differential: {} cases ({} all-position pitch cases), {} bytes per complete snapshot, {} compared bytes; no masks",
        oracle.cases,
        all_position_cases,
        RESPONSE_LEN,
        oracle.cases * RESPONSE_LEN
    );
}

#[test]
fn kof_retains_rust_wrapping_scan_beyond_the_defined_c_int_range() {
    let mut player = base(0, 0xffff);
    bytes(&mut player, 0xffff, &[1, 0xc8]);
    let before = player.clone();
    assert!(!player.want_write_kof(0));
    assert_eq!(player, before);
    bytes(&mut player, 0xffff, &[0xef, 0x00, 0x50]);
    player.ram[0x5000] = 0xc9;
    assert!(player.want_write_kof(0));
}
