//! Independent calls into unchanged C static handlers. All upload headers are
//! aligned, complete and nonaliased; malformed input belongs to frozen Rust
//! tests, not C's pointer-only upload API. Negative interpolation right shifts
//! follow the pinned target compiler, as in the original implementation.
use super::*;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
use z3_snes::dsp::DSP_SAVE_BLOCK_LEN;

const RAM: usize = 65536;
const RESPONSE: usize = 1 + RAM * 2 + DSP_SAVE_BLOCK_LEN + 4 + 512 + 14 + SAMPLE_FRAMES * 4;

#[derive(Clone, Debug)]
enum Operation {
    Instrument(usize, u8),
    Volume(usize, u16),
    Upload(Vec<u8>),
    Samples(bool),
    Patch(u16, Vec<u8>),
    Preset(usize, u8),
    StartUploadedVoice,
}
impl Operation {
    fn header(&self) -> [u8; 4] {
        let (kind, channel, argument) = match self {
            Self::Instrument(c, i) => (0, *c, u16::from(*i)),
            Self::Volume(c, v) => (1, *c, *v),
            Self::Upload(_) => (2, 0, 0),
            Self::Samples(audible) => (3, 0, u16::from(*audible)),
            Self::Patch(a, _) => (4, 0, *a),
            Self::Preset(c, i) => (5, *c, u16::from(*i)),
            Self::StartUploadedVoice => (6, 0, 0),
        };
        [kind, channel as u8, argument as u8, (argument >> 8) as u8]
    }
    fn apply(&self, player: &mut SpcPlayer) -> (u8, [i16; SAMPLE_FRAMES * 2]) {
        let mut samples = [0; SAMPLE_FRAMES * 2];
        let mut result = 0;
        match self {
            Self::Instrument(c, i) => player.channel_set_instrument(*c, *i),
            Self::Volume(c, v) => player.write_volume_to_dsp(*c, *v),
            Self::Upload(bytes) => {
                player.upload(bytes).unwrap();
                result = 1;
            }
            Self::Samples(audible) => {
                player.generate_samples();
                player.dsp.get_samples(&mut samples, SAMPLE_FRAMES, 2);
                if *audible {
                    assert!(
                        samples.iter().any(|&sample| sample != 0),
                        "uploaded BRR must be audible"
                    );
                }
            }
            Self::Patch(address, bytes) => {
                for (i, &byte) in bytes.iter().enumerate() {
                    player.ram[usize::from(address.wrapping_add(i as u16))] = byte;
                }
            }
            Self::Preset(c, i) => {
                player.ram[0x7000..0x7003].copy_from_slice(&[0xe0, *i, 0xc9]);
                player.sfx_sound_ptr_cur = SfxCursor::at(0x7000);
                player.sfx_channel_tick(*c, false);
            }
            Self::StartUploadedVoice => {
                for (register, value) in [
                    (DIR, 0x3c),
                    (MVOLL, 0x7f),
                    (MVOLR, 0x7f),
                    (V0VOLL, 0x60),
                    (V0VOLR, 0x60),
                    (V0PITCHL, 0),
                    (V0PITCHH, 0x10),
                    (V0SRCN, 0),
                    (V0ADSR1, 0),
                    (V0GAIN, 0x7f),
                    (FLG, 0x20),
                    (KOF, 0),
                    (KON, 1),
                ] {
                    player.dsp_write(register, value);
                }
            }
        }
        (result, samples)
    }
}

struct Oracle {
    child: Child,
    input: Option<BufWriter<ChildStdin>>,
    output: BufReader<ChildStdout>,
    response: Vec<u8>,
    cases: usize,
    snapshots: usize,
}
impl Oracle {
    fn spawn() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut child = Command::new(root.join("build/c-oracle/spc_data_oracle"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("build focused SPC data oracle first");
        Self {
            input: Some(BufWriter::new(child.stdin.take().unwrap())),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            response: vec![0; RESPONSE],
            cases: 0,
            snapshots: 0,
        }
    }
    fn check(
        &mut self,
        mut player: SpcPlayer,
        operations: &[Operation],
        context: impl std::fmt::Debug,
    ) {
        assert!(!operations.is_empty() && operations.len() <= 32);
        let input = self.input.as_mut().unwrap();
        input.write_all(&[operations.len() as u8]).unwrap();
        input.write_all(player.ram.as_slice()).unwrap();
        input
            .write_all(player.mapped_ram_snapshot().as_slice())
            .unwrap();
        input
            .write_all(&[player.timer_cycles, player.last_written_edl])
            .unwrap();
        input.write_all(&player.input_ports).unwrap();
        for (step, operation) in operations.iter().enumerate() {
            input.write_all(&operation.header()).unwrap();
            if let Operation::Upload(bytes) | Operation::Patch(_, bytes) = operation {
                input
                    .write_all(&(bytes.len() as u32).to_le_bytes())
                    .unwrap();
                input.write_all(bytes).unwrap();
            }
            input.flush().unwrap();
            // Read C first: even a Rust debug panic must not hide whether the
            // original C accepted and completed the same bounded operation.
            self.output.read_exact(&mut self.response).unwrap();
            let (result, samples) = operation.apply(&mut player);
            let mut offset = 0;
            let mut section = |name: &str, actual: &[u8]| {
                let expected = &self.response[offset..offset + actual.len()];
                if expected != actual {
                    let first = expected
                        .iter()
                        .zip(actual)
                        .position(|(a, b)| a != b)
                        .unwrap();
                    panic!(
                        "{context:?} step{step} {operation:?}: {name}[{first:#x}] C={:#x} Rust={:#x}",
                        expected[first], actual[first]
                    );
                }
                offset += actual.len();
            };
            section("return", &[result]);
            section("raw RAM", player.ram.as_slice());
            section(
                "passive mapped RAM",
                player.mapped_ram_snapshot().as_slice(),
            );
            let mut dsp = [0; DSP_SAVE_BLOCK_LEN];
            player.dsp.write_save_block(&mut dsp).unwrap();
            section("complete DSP", &dsp);
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
            section("nonmapped fields", &scalars);
            let pcm: Vec<_> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
            section("public PCM", &pcm);
            assert_eq!(offset, RESPONSE);
            self.snapshots += 1;
        }
        self.cases += 1;
    }
    fn finish(mut self) {
        drop(self.input.take());
        assert!(
            self.child.wait().unwrap().success(),
            "C oracle must exit successfully"
        );
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
        cur_chan_bit: 1 << ci,
        current_bit: 1 << ci,
        sfx_channel_index: (ci * 2) as u8,
        reg_flg: 0xe7,
        reg_non: 0x55,
        last_written_edl: 0x91,
        input_ports: [1, 2, 3, 4],
        ..SpcPlayer::default()
    };
    for (i, byte) in player.ram.iter_mut().enumerate() {
        *byte = (i as u8).wrapping_mul(73).wrapping_add((i >> 8) as u8);
    }
    player.channels[ci].final_volume = 255;
    player.channels[ci].instrument_pitch_base = 0xa75c;
    player.channels[ci].pitch = 0x3456;
    player.channels[ci].sfx_note_length = 4;
    player
}

#[test]
#[ignore = "requires built original C SPC data oracle"]
fn pan_interpolation_full_byte_delta_matches_original_c() {
    let mut oracle = Oracle::spawn();
    let mut player = base(0);
    player.ram[0x1178 + 21] = 0;
    player.ram[0x1179 + 21] = 255;
    oracle.check(
        player,
        &[Operation::Volume(0, 0x15ff)],
        "full byte pan delta",
    );
    oracle.finish();
}

fn upload(blocks: &[(u16, Vec<u8>)], tail: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (destination, payload) in blocks {
        assert!(!payload.is_empty() && payload.len() <= 65535 && payload.len().is_multiple_of(2));
        bytes.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&destination.to_le_bytes());
        bytes.extend_from_slice(payload);
    }
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(tail);
    bytes
}

#[test]
#[ignore = "requires built original C SPC data oracle"]
fn spc_data_and_valid_uploads_match_original_c() {
    let mut oracle = Oracle::spawn();
    for ci in 0..8 {
        for percussion in [0, 1, 73, 255] {
            for owned in [false, true] {
                for instrument in 0..=255 {
                    let mut player = base(ci);
                    player.percussion_base_id = percussion;
                    player.is_chan_on = if owned { 1 << ci } else { !(1 << ci) };
                    oracle.check(
                        player,
                        &[Operation::Instrument(ci, instrument)],
                        ("instrument", ci, percussion, owned, instrument),
                    );
                }
            }
        }
    }
    // Every pan word, with descending and nonmonotonic *full-byte* fallback
    // segments, both channel sides and wrapped 0x1400 - pan interpolation.
    for pan in 0..=65535u16 {
        let ci = usize::from(pan & 7);
        let mut player = base(ci);
        player.channels[ci].pan_flag_with_phase_invert = ((pan >> 8) as u8) & 0xc0;
        oracle.check(
            player,
            &[Operation::Volume(ci, pan)],
            ("all pan words", pan, ci),
        );
    }
    for ci in 0..8 {
        for owned in [false, true] {
            for phase in [0, 0x40, 0x80, 0xc0, 0xff] {
                for final_volume in [0, 1, 127, 128, 254, 255] {
                    for pan in [
                        0, 1, 0x13ff, 0x1400, 0x1401, 0x14ff, 0x1500, 0x15ff, 0xff00, 0xffff,
                    ] {
                        let mut player = base(ci);
                        player.is_chan_on = if owned { 1 << ci } else { !(1 << ci) };
                        player.channels[ci].pan_flag_with_phase_invert = phase;
                        player.channels[ci].final_volume = final_volume;
                        oracle.check(
                            player,
                            &[Operation::Volume(ci, pan)],
                            ("pan boundaries", ci, owned, phase, final_volume, pan),
                        );
                    }
                }
            }
        }
    }
    // Each byte of the nine-byte preset independently influences real DSP
    // writes. Six-byte instruments overlap these same live bytes.
    for ci in 0..8 {
        for preset in 0..=255u8 {
            let mut player = base(ci);
            player.sfx_sound_ptr_cur = SfxCursor::at(0x7000);
            for byte in 0..9 {
                let row = 0x3e00 + usize::from(preset) * 9;
                player.ram[row + byte] = (byte as u8).wrapping_mul(37).wrapping_add(preset);
            }
            oracle.check(
                player,
                &[Operation::Preset(ci, preset)],
                ("preset nine bytes", ci, preset),
            );
        }
    }
    for ci in 0..8 {
        for preset in [0, 1, 85, 255] {
            for byte in 0..9 {
                for value in [0, 0x7f, 0x80, 255] {
                    let mut player = base(ci);
                    let row = 0x3e00 + usize::from(preset) * 9;
                    player.ram[row..row + 9]
                        .copy_from_slice(&[0x12, 0x34, 0x56, 0x27, 9, 0x8f, 0x3c, 0x7f, 2]);
                    player.ram[row + byte] = value;
                    oracle.check(
                        player,
                        &[Operation::Preset(ci, preset)],
                        ("independent preset byte", ci, preset, byte, value),
                    );
                }
            }
        }
    }
    for ci in 0..8 {
        let player = base(ci);
        oracle.check(
            player,
            &[
                Operation::Instrument(ci, 43), // 3e02 overlaps preset0.
                Operation::Patch(
                    0x3e00,
                    vec![0x90, 0x17, 0x21, 0x42, 0x03, 0x8f, 0x15, 0x7f, 0x02],
                ),
                Operation::Instrument(ci, 43),
                Operation::Preset(ci, 0),
                Operation::Patch(
                    0x3e00,
                    vec![0x11, 0x27, 0x83, 0x62, 0x05, 0x9f, 0x35, 0x6f, 0x04],
                ),
                Operation::Preset(ci, 0),
                Operation::Instrument(ci, 43),
            ],
            ("live shared record mutation", ci),
        );
    }
    let fixtures = [
        upload(&[], &[]),
        upload(&[], &[0xff, 0x78, 0x56, 0x34, 0x12]),
        upload(
            &[
                (0, vec![0xa5; 8]),
                (0x3d00, vec![2, 0x8f, 0xe0, 0x7f, 2, 0]),
            ],
            &[],
        ),
        upload(
            &[
                (0x3c00, vec![0x00, 0x50, 0x00, 0x50]),
                (0x5000, vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            ],
            &[],
        ),
        upload(
            &[
                (0xffff, vec![1, 2, 3, 4]),
                (0x1178, vec![0x01, 0xff, 0x82, 0x03]),
            ],
            &[],
        ),
        upload(
            &[
                (0x3d00, vec![0xff; 16]),
                (0x3d04, vec![0; 8]),
                (0x3d00, vec![2, 0x8f, 0x21, 0x7f, 3, 0x81]),
            ],
            &[],
        ),
        upload(
            &[(
                0x8001,
                (0..65534).map(|i| (i as u8).wrapping_mul(29)).collect(),
            )],
            &[0xde, 0xad],
        ),
    ];
    for (index, bytes) in fixtures.into_iter().enumerate() {
        let mut player = base(0);
        player.pause_music_ctr = 0xa5;
        player.port_to_snes[0] = 0x17;
        player.port1_active = 1;
        player.port2_active = 2;
        player.port3_active = 4;
        player.is_chan_on = 0xff;
        oracle.check(
            player,
            &[
                Operation::Upload(bytes),
                Operation::Instrument(0, 0),
                Operation::Preset(0, 0),
            ],
            ("upload + live lookups", index),
        );
    }
    // Quiet player state avoids invalid uploaded music/SFX programs. Public
    // GenerateSamples and get_samples still run real DSP/echo cycles, and all
    // generated PCM, internal DSP state and unmapped bytes are compared.
    for destination in [0, 0x1178, 0x3c00, 0x3d00, 0x3e00, 0xffff] {
        let mut player = SpcPlayer {
            history: Some(DspRegWriteHistory::default()),
            ..SpcPlayer::default()
        };
        player.new_value_from_snes = [0; 4];
        oracle.check(
            player,
            &[
                Operation::Upload(upload(&[(destination, vec![0x17, 0x21, 0x35, 0x49])], &[])),
                Operation::Samples(false),
                Operation::Samples(false),
                Operation::Samples(false),
            ],
            ("public samples after upload", destination),
        );
    }
    for nybble in [0x11, 0x37, 0x55, 0x77] {
        let player = SpcPlayer {
            history: Some(DspRegWriteHistory::default()),
            ..SpcPlayer::default()
        };
        let mut brr = vec![0x63]; // End+loop, no filter, range6: audible positive samples.
        brr.extend_from_slice(&[nybble; 8]);
        brr.push(0); // Even source size preserves aligned next upload header.
        oracle.check(
            player,
            &[
                Operation::Upload(upload(
                    &[(0x3c00, vec![0, 0x50, 0, 0x50]), (0x5000, brr)],
                    &[],
                )),
                Operation::StartUploadedVoice,
                Operation::Samples(true),
                Operation::Samples(true),
                Operation::Samples(true),
            ],
            ("audible uploaded BRR", nybble),
        );
    }
    eprintln!(
        "SPC data original C: {} cases, {} full snapshots, {} compared bytes",
        oracle.cases,
        oracle.snapshots,
        oracle.snapshots * RESPONSE
    );
    oracle.finish();
}
