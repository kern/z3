//! Bounded programs for the unchanged static C Port0_HandleMusic entrypoint.
//! Never read a C word at 0xffff or an unaligned word, and never build an
//! unterminated control chain. Rust's separate cursor unit tests cover its
//! stronger wrap semantics without claiming those are defined C accesses.

use super::*;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
use z3_snes::dsp::DSP_SAVE_BLOCK_LEN;

const RAM_LEN: usize = 65536;
const RESPONSE_LEN: usize = RAM_LEN * 2 + DSP_SAVE_BLOCK_LEN + 4 + 256 * 2;
const PHRASE: usize = 0x800;
const TABLE: u16 = 0x3000;

struct Oracle {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
    response: Vec<u8>,
    cases: usize,
    snapshots: usize,
}

impl Oracle {
    fn spawn() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut child = Command::new(root.join("build/c-oracle/spc_phrase_oracle"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("build the focused C oracle first");
        Self {
            input: BufWriter::new(child.stdin.take().unwrap()),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            response: vec![0; RESPONSE_LEN],
            cases: 0,
            snapshots: 0,
        }
    }

    fn check(&mut self, mut player: SpcPlayer, commands: &[u8], context: impl std::fmt::Debug) {
        assert!(!commands.is_empty() && commands.len() <= 8);
        self.input.write_all(&[commands.len() as u8]).unwrap();
        self.input.write_all(commands).unwrap();
        self.input.write_all(player.ram.as_slice()).unwrap();
        self.input
            .write_all(player.mapped_ram_snapshot().as_slice())
            .unwrap();
        self.input.flush().unwrap();
        for (step, &command) in commands.iter().enumerate() {
            player.new_value_from_snes[0] = command;
            player.port0_handle_music();
            self.output.read_exact(&mut self.response).unwrap();
            let mut offset = 0;
            let mut compare = |label: &str, actual: &[u8]| {
                let expected = &self.response[offset..offset + actual.len()];
                // Slice equality is a complete optimized comparison, not an
                // interpreted per-byte loop across the multi-gigabyte sweep.
                if expected != actual {
                    let first = expected
                        .iter()
                        .zip(actual)
                        .position(|(a, b)| a != b)
                        .unwrap();
                    panic!(
                        "{context:?} step {step} command {command:#x}: {label}[{first:#x}] C={:#x} Rust={:#x}",
                        expected[first], actual[first]
                    );
                }
                offset += actual.len();
            };
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
            self.snapshots += 1;
        }
        self.cases += 1;
    }
}

impl Drop for Oracle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn word(player: &mut SpcPlayer, address: usize, value: u16) {
    assert!(address.is_multiple_of(2) && address <= RAM_LEN - 2);
    player.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
}

fn cursor(player: &mut SpcPlayer, address: u16) {
    // Exercise the persistent field through the actual lossless save codec;
    // this stays valid when its owned type changes from u16 to PhraseCursor.
    let raw = player.ram.clone();
    let mut mapped = player.mapped_ram_snapshot();
    mapped[0x40..0x42].copy_from_slice(&address.to_le_bytes());
    player.ram = mapped;
    player.copy_variables_from_ram();
    player.ram = raw;
}

fn base() -> SpcPlayer {
    let mut player = SpcPlayer {
        history: Some(DspRegWriteHistory::default()),
        counter_sf0c: 1,
        ..SpcPlayer::default()
    };
    player.port_to_snes[0] = 1;
    player.ram.fill(0xa5);
    player.ram[usize::from(TABLE)..usize::from(TABLE) + 16].fill(0);
    word(&mut player, PHRASE, 0);
    cursor(&mut player, PHRASE as u16);
    player
}

#[test]
#[ignore = "requires build/c-oracle/spc_phrase_oracle; exhaustive complete-state C differential"]
fn phrase_cursor_matches_static_c_music_handler() {
    let mut oracle = Oracle::spawn();
    let template = base();
    // Every repeat code and every prior count. Both destinations are bounded
    // phrases, including the fallthrough path when the decremented count is 0.
    for code in 1..=255u16 {
        if matches!(code, 0x80 | 0x81) {
            continue;
        }
        for count in 0..=255u8 {
            let mut player = template.clone();
            player.block_count = count;
            word(&mut player, PHRASE, code);
            word(&mut player, PHRASE + 2, 0x820);
            word(&mut player, PHRASE + 4, TABLE);
            word(&mut player, 0x820, TABLE);
            oracle.check(player, &[0], ("repeat", code, count));
        }
    }
    let exhaustive_cases = oracle.cases;
    assert_eq!(exhaustive_cases, 253 * 256);

    // High-byte-zero phrase entries all retain their original meanings.
    for chain in [
        vec![0],
        vec![0x80, TABLE],
        vec![0x81, TABLE],
        vec![0x80, 0x81, 0x80, TABLE],
        vec![0x81, 0x80, 0x81, 0],
    ] {
        let mut player = template.clone();
        for (index, &value) in chain.iter().enumerate() {
            word(&mut player, PHRASE + index * 2, value);
        }
        oracle.check(player, &[0], ("control chain", chain));
    }
    // No channel dereferences are needed to prove exact cursor/count update at
    // the valid C word edge. The increment at 0xfffe wraps, but no word ever
    // starts at 0xffff; the last valid eight-word descriptor starts at 0xfff0.
    for address in [0, 2, 0x7ffe, 0xfffc, 0xfffe] {
        let mut player = template.clone();
        cursor(&mut player, address);
        word(&mut player, usize::from(address), TABLE);
        oracle.check(player, &[0], ("word boundary", address));
    }
    for count in [0, 1, 2, 0x80] {
        let mut player = template.clone();
        cursor(&mut player, 0xfffc);
        player.block_count = count;
        word(&mut player, 0xfffc, 3);
        word(&mut player, 0xfffe, 0x820);
        word(&mut player, 0, TABLE);
        word(&mut player, 0x820, TABLE);
        oracle.check(player, &[0], ("repeat destination wrap", count));
    }
    for toggle in [0x80, 0x81] {
        let mut player = template.clone();
        cursor(&mut player, 0xfffe);
        word(&mut player, 0xfffe, toggle);
        word(&mut player, 0, TABLE);
        oracle.check(player, &[0], ("toggle control wrap", toggle));
    }
    let mut player = template.clone();
    word(&mut player, PHRASE, 0xfff0);
    player.ram[0xfff0..].fill(0);
    oracle.check(player, &[0], "last descriptor");

    // Starting a song initializes the cursor immediately, reset consumes one
    // tick, and phrase loading consumes the next. Include every implemented
    // command which the C handler treats as a song ID, including f4..fe.
    for command in 1..=254u8 {
        if matches!(command, 0xf0..=0xf3) {
            continue;
        }
        let mut player = template.clone();
        word(
            &mut player,
            0xd000 + usize::from(command - 1) * 2,
            PHRASE as u16,
        );
        word(&mut player, PHRASE, TABLE);
        oracle.check(player, &[command, 0, 0], ("song", command));
    }

    for command in [0, 0xf0, 0xf1, 0xf2, 0xf3] {
        for port in [0, 1] {
            for pause in [0, 1, 2, 0x80] {
                for saved_volume in [0, 0x46] {
                    let mut player = template.clone();
                    player.port_to_snes[0] = port;
                    player.pause_music_ctr = pause;
                    player.byte_3e1 = saved_volume;
                    player.master_volume = 0xc034;
                    oracle.check(
                        player,
                        &[command],
                        ("guard", command, port, pause, saved_volume),
                    );
                }
            }
        }
    }

    // Mixed active/disabled channel tables, every active-channel bitmask,
    // instrument setup/skipping, and a zero pattern token which restarts the
    // phrase loader during channel iteration. Keep note/effect streams bounded.
    for mask in 0..=255u8 {
        for instrument in [0, 1] {
            let mut player = template.clone();
            word(&mut player, PHRASE, TABLE);
            for index in 0..8 {
                let active = mask & (1 << index) != 0;
                word(
                    &mut player,
                    usize::from(TABLE) + index * 2,
                    if active { 0x4000 } else { index as u16 },
                );
                player.channels[index].instrument_id = instrument;
            }
            player.ram[0x4000..0x4004].copy_from_slice(&[2, 0xc9, 0, 0]);
            player.ram[0x3d00..0x3d0c].fill(0);
            oracle.check(player, &[0], ("channels", mask, instrument));
        }
    }
    for channel in 0..8 {
        let mut player = template.clone();
        player.counter_sf0c = 0;
        player.channels[channel].pattern_order_ptr_for_chan = PatternCursor::at(0x4000);
        player.channels[channel].note_ticks_left = 1;
        player.ram[0x4000] = 0;
        word(&mut player, PHRASE, TABLE);
        oracle.check(player, &[0], ("mid-channel restart", channel));
    }
    println!(
        "phrase cursor C differential: {} cases ({} exhaustive repeat/count), {} complete {}-byte snapshots, {} compared bytes; no masks",
        oracle.cases,
        exhaustive_cases,
        oracle.snapshots,
        RESPONSE_LEN,
        oracle.snapshots * RESPONSE_LEN
    );
}
