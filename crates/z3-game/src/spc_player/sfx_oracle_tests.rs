//! Full-state SFX oracles, including accepted panned commands whose ID is zero.
use super::*;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
use z3_snes::dsp::DSP_SAVE_BLOCK_LEN;

const RAM_LEN: usize = 65536;
const PCM_SAMPLES: usize = 534 * 2;
const RESPONSE_LEN: usize = 1 + RAM_LEN * 2 + DSP_SAVE_BLOCK_LEN + 4 + 512 + 14 + PCM_SAMPLES * 2;

fn executable() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/c-oracle/spc_sfx_oracle")
}

fn compare(
    response: &[u8],
    player: &SpcPlayer,
    result: u8,
    pcm: &[i16; PCM_SAMPLES],
    context: impl std::fmt::Debug,
) {
    let mut offset = 0;
    let mut section = |name: &str, actual: &[u8]| {
        let expected = &response[offset..offset + actual.len()];
        if expected != actual {
            let first = expected
                .iter()
                .zip(actual)
                .position(|(a, b)| a != b)
                .unwrap();
            panic!(
                "{context:?}: {name}[{first:#06x}] C={:#04x} Rust={:#04x}",
                expected[first], actual[first]
            );
        }
        offset += actual.len();
    };
    section("return", &[result]);
    section("raw RAM", player.ram.as_slice());
    section("mapped RAM", player.mapped_ram_snapshot().as_slice());
    let mut dsp = [0; DSP_SAVE_BLOCK_LEN];
    player.dsp.write_save_block(&mut dsp).unwrap();
    section("DSP", &dsp);
    let history = player.history.as_ref().unwrap();
    section("history count", &history.count.to_le_bytes());
    section("history addresses", &history.addr);
    section("history values", &history.val);
    let mut scalars = [0; 14];
    scalars[0] = player.timer_cycles;
    scalars[1] = player.last_written_edl;
    scalars[2..6].copy_from_slice(&player.input_ports);
    for (index, channel) in player.channels.iter().enumerate() {
        scalars[6 + index] = channel.index;
    }
    section("nonmapped scalars", &scalars);
    let bytes: Vec<_> = pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
    section("PCM", &bytes);
    assert_eq!(offset, RESPONSE_LEN);
}

struct Oracle {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
    response: Vec<u8>,
    cases: usize,
}

impl Oracle {
    fn spawn() -> Self {
        let mut child = Command::new(executable())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
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
        op: u8,
        ci: usize,
        arg: u8,
        context: impl std::fmt::Debug,
    ) {
        self.input.write_all(&[op, ci as u8, arg]).unwrap();
        self.input.write_all(player.ram.as_slice()).unwrap();
        self.input
            .write_all(player.mapped_ram_snapshot().as_slice())
            .unwrap();
        self.input.flush().unwrap();
        let result = match op {
            0 => {
                player.sfx_channel_tick(ci, arg != 0);
                0
            }
            1 => {
                player.port1_start_new_sound();
                0
            }
            2 | 3 => {
                player.port_start_new_sound(usize::from(op));
                0
            }
            4 => {
                player.port1_play_inner();
                0
            }
            5 | 6 => player.allocate_sfx_channel(usize::from(op - 3)) as u8,
            7 | 8 => {
                player.port_handle_cmd(usize::from(op - 5));
                0
            }
            9 => {
                player.port1_handle_cmd();
                0
            }
            _ => unreachable!(),
        };
        self.output.read_exact(&mut self.response).unwrap();
        compare(
            &self.response,
            &player,
            result,
            &[0; PCM_SAMPLES],
            (&context, op, ci, arg),
        );
        self.cases += 1;
    }
}

impl Drop for Oracle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn base(ci: usize) -> SpcPlayer {
    let mut player = SpcPlayer {
        history: Some(DspRegWriteHistory::default()),
        ..SpcPlayer::default()
    };
    player.current_bit = 1 << ci;
    player.cur_chan_bit = 1 << ci;
    player.is_chan_on = 1 << ci;
    player.sfx_channel_index = (ci * 2) as u8;
    player.sfx_sound_ptr_cur = SfxCursor::at(0x4000);
    let voice = &mut player.channels[ci];
    voice.sfx_sound_ptr = SfxPosition::at(0x6000);
    voice.sfx_which_sound = 1;
    voice.sfx_note_length = 4;
    voice.sfx_note_length_left = 1;
    voice.instrument_id = 1;
    voice.instrument_pitch_base = 0x0200;
    voice.pitch = 0x3456;
    voice.final_volume = 120;
    voice.pan_value = 0x0a00;
    player.ram[0x3d06..0x3d0c].copy_from_slice(&[2, 0x8f, 0xe0, 0x7f, 2, 0]);
    player
}

fn bytes(player: &mut SpcPlayer, start: u16, values: &[u8]) {
    for (index, &value) in values.iter().enumerate() {
        player.ram[usize::from(start.wrapping_add(index as u16))] = value;
    }
}

fn word(player: &mut SpcPlayer, at: usize, value: u16) {
    assert!(at.is_multiple_of(2) && at <= 0xfffe);
    player.ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

#[test]
#[ignore = "requires built C SFX oracle; full static-handler differential"]
fn sfx_cursors_match_static_c_handlers() {
    let mut oracle = Oracle::spawn();
    for ci in 0..8 {
        let template = base(ci);
        for address in 0..=u16::MAX {
            let mut player = template.clone();
            player.sfx_sound_ptr_cur = SfxCursor::at(address);
            bytes(&mut player, address, &[0xf1, 0, 0, 0x56]);
            oracle.check(player, 0, ci, 0, ("all-position f1", address));
        }
    }
    let all_positions = oracle.cases;
    assert_eq!(all_positions, 8 * 65536);
    for ci in 0..8 {
        for left in 0..=255u8 {
            for slide in [0, 1, 2, 0xff] {
                let mut player = base(ci);
                player.channels[ci].sfx_note_length_left = left;
                player.channels[ci].pitch_slide_length = slide;
                player.channels[ci].pitch_add_per_tick = 0x123;
                player.channels[ci].pitch_target = 0x56;
                bytes(&mut player, 0x6000, &[0xc9, 0xc9]);
                oracle.check(player, 0, ci, 1, ("continuation count", left, slide));
            }
        }
        for duration in 1..=127u8 {
            for volume in [0, 1, 0x7f] {
                for stereo in [false, true] {
                    for port1 in [false, true] {
                        for fixed_volume in [0, 1] {
                            let mut player = base(ci);
                            player.port1_active = if port1 { 1 << ci } else { 0 };
                            player.channel_67_volume = fixed_volume;
                            let stereo_program = [duration, volume, 0x23, 0xc9];
                            let mono_program = [duration, volume, 0xc9];
                            let program: &[u8] = if stereo {
                                &stereo_program
                            } else {
                                &mono_program
                            };
                            bytes(&mut player, 0x4000, program);
                            oracle.check(
                                player,
                                0,
                                ci,
                                0,
                                (
                                    "duration/volume",
                                    duration,
                                    volume,
                                    stereo,
                                    port1,
                                    fixed_volume,
                                ),
                            );
                        }
                    }
                }
            }
        }
        for address in [0, 0xfffd, 0xfffe, 0xffff] {
            for opcode in [0xf1, 0xf9] {
                for pan in [0, 0x40, 0x80, 0xc0] {
                    let mut player = base(ci);
                    player.sfx_sound_ptr_cur = SfxCursor::at(address);
                    player.sfx_start_arg_pan = pan;
                    if opcode == 0xf9 {
                        bytes(&mut player, address, &[opcode, 0xc9, 2, 1, 0x56]);
                    } else {
                        bytes(&mut player, address, &[opcode, 2, 1, 0x56]);
                    }
                    oracle.check(
                        player,
                        0,
                        ci,
                        0,
                        ("slide prefix wrap", address, opcode, pan),
                    );
                }
            }
        }
        for preset in 0..=255u8 {
            for tail in [0, 0xc9] {
                let mut player = base(ci);
                player.sfx_sound_ptr_cur = SfxCursor::at(0x7000);
                let row = 0x3e00 + usize::from(preset) * 9;
                player.ram[row..row + 9]
                    .copy_from_slice(&[0x12, 0x34, 0x56, 0x27, 9, 0x8f, 0x3c, 0x7f, 2]);
                bytes(&mut player, 0x7000, &[0xe0, preset, tail]);
                oracle.check(player, 0, ci, 0, ("preset", preset, tail));
            }
        }
        let mut player = base(ci);
        bytes(&mut player, 0x4000, &[0xe0, 2, 0xe0, 3, 0xc9]);
        oracle.check(player, 0, ci, 0, "preset chain");
        for sound in 0..=255u8 {
            let mut player = base(ci);
            player.channels[ci].sfx_which_sound = sound;
            bytes(&mut player, 0x4000, &[0xff]);
            word(&mut player, 0x17be + usize::from(sound) * 2, 0x5000);
            bytes(&mut player, 0x5000, &[0]);
            oracle.check(player, 0, ci, 0, ("FF loop/End", sound));
        }
        for music in [0, 0x10] {
            for echo in [0, 1 << ci] {
                for mask in [0, 1 << ci] {
                    let mut player = base(ci);
                    player.port_to_snes[0] = music;
                    player.reg_eon = echo;
                    player.echo_channels = 1 << ci;
                    player.sfx_channels_echo_mask2 = mask;
                    player.channels[ci].sfx_note_length_left = 2;
                    oracle.check(player, 0, ci, 1, ("echo continuation", music, echo, mask));
                }
            }
        }
        for count in [0, 1, 2, 0xff] {
            for sound in [0, 1] {
                for op in [1, 2, 3] {
                    if op == 1 && ci < 5 {
                        continue;
                    }
                    let mut player = base(ci);
                    player.channels[ci].sfx_arr_countdown = count;
                    player.channels[ci].sfx_which_sound = sound;
                    if op == 1 {
                        player.port1_active = 1 << ci;
                    }
                    if op == 2 {
                        player.port2_active = 1 << ci;
                    }
                    if op == 3 {
                        player.port3_active = 1 << ci;
                    }
                    let table = match op {
                        1 => 0x17be,
                        2 => 0x181e,
                        _ => 0x191a,
                    };
                    word(&mut player, table + usize::from(sound) * 2, 0x5000);
                    oracle.check(player, op, ci, 0, ("start count guard", count, sound));
                }
            }
        }
    }
    for sound in 0..=255u8 {
        let mut player = base(7);
        player.new_value_from_snes[1] = sound;
        player.ram[0x17ff + usize::from(sound)] = 0;
        oracle.check(
            player,
            4,
            7,
            0,
            ("Port1 pair; zero/high IDs helper-only", sound),
        );
        for op in [1, 2, 3] {
            let mut player = base(7);
            player.channels[7].sfx_arr_countdown = 1;
            player.channels[7].sfx_which_sound = sound;
            if op == 1 {
                player.port1_active = 0x80;
            }
            if op == 2 {
                player.port2_active = 0x80;
            }
            if op == 3 {
                player.port3_active = 0x80;
            }
            let table = match op {
                1 => 0x17be,
                2 => 0x181e,
                _ => 0x191a,
            };
            word(&mut player, table + usize::from(sound) * 2, 0x5000);
            oracle.check(player, op, 7, 0, ("all sound IDs start", sound));
        }
    }
    for command in 0..=255u8 {
        for active in [0, 0x80, 0xc0] {
            for paired_sound in [0, 1] {
                let mut player = base(7);
                player.new_value_from_snes[1] = command;
                player.port1_active = active;
                player.port2_active = 0xff;
                player.port3_active = 0xff;
                player.ram[0x17ff + usize::from(command)] = paired_sound;
                oracle.check(
                    player,
                    9,
                    7,
                    0,
                    ("Port1 command", command, active, paired_sound),
                );
            }
        }
    }
    for counter in 0..=255u8 {
        for active in [0, 0x80, 0xc0] {
            let mut player = base(7);
            player.port1_counter = counter;
            player.port1_active = active;
            oracle.check(player, 1, 7, 0, ("Port1 counter", counter, active));
        }
    }
    for primary in [0, 1, 5, 0x7f, 0x80, 0xff] {
        for secondary in [1, 0x7f, 0xff] {
            let mut player = base(7);
            player.new_value_from_snes[1] = primary;
            player.port2_active = 0xff;
            player.port3_active = 0xff;
            player.ram[0x17ff + usize::from(primary)] = secondary;
            oracle.check(player, 4, 7, 0, ("Port1 paired helper", primary, secondary));
        }
    }
    for mask in 0..=255u8 {
        for op in [1, 2, 3] {
            let mut player = base(7);
            player.port1_active = if op == 1 { mask } else { 0 };
            player.port2_active = if op == 2 { mask } else { 0 };
            player.port3_active = if op == 3 { mask } else { 0 };
            oracle.check(player, op, 7, 0, ("all active masks", mask));
        }
    }
    for command in 1..=255u8 {
        for port in [2, 3] {
            for reuse in [false, true] {
                let mut player = base(7);
                player.new_value_from_snes[port] = command;
                player.is_chan_on = if reuse { 0x80 } else { 0 };
                player.channels[7].sfx_which_sound = command & 0x3f;
                player.channels[7].sfx_pan = command & 0xc0;
                if port == 2 {
                    player.port2_active = if reuse { 0x80 } else { 0 };
                } else {
                    player.port3_active = if reuse { 0x80 } else { 0 };
                }
                player.ram[if port == 2 {
                    0x18dc + usize::from(command & 0x3f)
                } else {
                    0x19d8 + usize::from(command & 0x3f)
                }] = 1;
                oracle.check(
                    player,
                    (port + 3) as u8,
                    7,
                    0,
                    ("allocate accepted byte", port, command, reuse),
                );
            }
            let mut player = base(7);
            player.is_chan_on = 0;
            player.new_value_from_snes[port] = command;
            oracle.check(
                player,
                (port + 5) as u8,
                7,
                0,
                ("command accepted byte", port, command),
            );
        }
    }
    for port in [2, 3] {
        for old_wrapped_nonzero in [false, true] {
            let mut player = base(7);
            player.is_chan_on = 0;
            player.new_value_from_snes[port] = 0x40;
            let (predecessor, old_wrapped) = if port == 2 {
                (0x189d, 0x199d)
            } else {
                (0x1999, 0x1a99)
            };
            player.ram[predecessor] = u8::from(!old_wrapped_nonzero);
            player.ram[old_wrapped] = u8::from(old_wrapped_nonzero);
            oracle.check(
                player,
                (port + 5) as u8,
                7,
                0,
                ("zero-ID chain predecessor", port, old_wrapped_nonzero),
            );
        }
    }
    println!(
        "SFX static C differential: {} cases, {} all-position cases, {} bytes/snapshot, {} complete compared bytes; no masks",
        oracle.cases,
        all_positions,
        RESPONSE_LEN,
        oracle.cases * RESPONSE_LEN
    );
}

#[test]
#[ignore = "requires built C SFX oracle; public initialize/generate-samples parity"]
fn sfx_public_samples_match_c_for_zero_id_commands() {
    let mut frames = 0;
    for port in [2, 3] {
        for command in [0x40, 0x80, 0xc0] {
            let mut child = Command::new(executable())
                .args([port.to_string(), command.to_string(), "12".to_owned()])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let mut reference = BufReader::new(child.stdout.take().unwrap());
            let mut player = SpcPlayer::new();
            player.initialize();
            player.history = Some(DspRegWriteHistory::default());
            player.ram.fill(0);
            let base = if port == 2 { 0x1820 } else { 0x191c };
            player.ram[base - 1] = 0x40;
            player.ram[base + 511] = 0x50;
            if port == 2 {
                player.ram[0x18dc] = 1;
            }
            player.input_ports[port] = command;
            let mut response = vec![0; RESPONSE_LEN];
            for frame in 1..=12 {
                player.generate_samples();
                let mut pcm = [0; PCM_SAMPLES];
                player.dsp.get_samples(&mut pcm, 534, 2);
                reference.read_exact(&mut response).unwrap();
                compare(
                    &response,
                    &player,
                    0,
                    &pcm,
                    ("public zero ID", port, command, frame),
                );
                frames += 1;
            }
            assert!(child.wait().unwrap().success());
        }
    }
    assert_eq!(frames, 72);
    println!(
        "SFX public API: 6 accepted commands, 72 complete raw/mapped RAM, DSP/history/PCM frames; no masks"
    );
}
