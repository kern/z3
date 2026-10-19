//! Valid-file differential against the original C recorder FILE APIs.
//! Invalid/truncated acceptance is checked against frozen Rust, not C aborts.
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
use z3_game::runtime::{LEGACY_SNAPSHOT_LEN, StateRecorder};
use z3_verify::legacy_snapshot;

type Error = Box<dyn std::error::Error>;
const WRAM_MARKER: usize = 144_389 + 0x1fff0;

fn tokens(source: &str) -> String {
    // The C typedef contains only declarations and comments, no strings.
    let mut cleaned = String::new();
    for line in source.lines() {
        cleaned.push_str(line.split("//").next().unwrap());
    }
    cleaned.chars().filter(|c| !c.is_whitespace()).collect()
}

fn declaration<'a>(source: &'a str, name: &str) -> Result<&'a str, Error> {
    let start = source
        .find(&format!("typedef struct {name} {{"))
        .ok_or("missing C typedef")?;
    let end = source[start..]
        .find(&format!("}} {name};"))
        .ok_or("unterminated C typedef")?;
    Ok(&source[start..start + end + name.len() + 3])
}

fn audit_private_abi(root: &Path) -> Result<(), Error> {
    let original = std::fs::read_to_string(root.join("src/zelda_rtl.c"))?;
    let harness = std::fs::read_to_string(root.join("oracle/recorder_file_oracle.c"))?;
    if tokens(declaration(&original, "StateRecorder")?)
        != tokens(declaration(&harness, "StateRecorder")?)
    {
        return Err("recorder oracle private ABI no longer matches original C typedef".into());
    }
    let util = std::fs::read_to_string(root.join("src/util.h"))?;
    if tokens(declaration(&util, "ByteArray")?)
        != "typedefstructByteArray{uint8*data;size_tsize,capacity;}ByteArray;"
    {
        return Err("ByteArray ABI changed; re-ground the recorder harness".into());
    }
    Ok(())
}

fn header(bytes: &[u8]) -> [u32; 8] {
    std::array::from_fn(|index| {
        u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
    })
}

fn current(bytes: &[u8]) -> &[u8] {
    let h = header(bytes);
    let start = 32 + h[2] as usize + if h[5] & 1 != 0 { h[6] as usize } else { 0 };
    &bytes[start..start + h[6] as usize]
}

fn fixture(h: [u32; 8], log: &[u8], base: &[u8], snapshot: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in h {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(log);
    bytes.extend_from_slice(base);
    bytes.extend_from_slice(snapshot);
    bytes
}

struct Oracle {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
}

impl Oracle {
    fn spawn(executable: &str, assets: &str) -> Result<Self, Error> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let executable = std::fs::canonicalize(executable)?;
        let assets = std::fs::canonicalize(assets)?;
        let mut child = Command::new(executable)
            .arg(assets)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        Ok(Self {
            input: BufWriter::new(child.stdin.take().ok_or("oracle stdin")?),
            output: BufReader::new(child.stdout.take().ok_or("oracle stdout")?),
            child,
        })
    }

    fn receive(&mut self) -> Result<(u32, bool, Vec<u8>), Error> {
        let mut prefix = [0; 12];
        self.output.read_exact(&mut prefix)?;
        let marker = u32::from_le_bytes(prefix[..4].try_into()?);
        let replay = u32::from_le_bytes(prefix[4..8].try_into()?) != 0;
        let length = u32::from_le_bytes(prefix[8..].try_into()?) as usize;
        let mut bytes = vec![0; length];
        self.output.read_exact(&mut bytes)?;
        Ok((marker, replay, bytes))
    }

    fn save(&mut self, fields: [u32; 9], log: &[u8], base: &[u8]) -> Result<Vec<u8>, Error> {
        self.input.write_all(&[0])?;
        for field in fields {
            self.input.write_all(&field.to_le_bytes())?;
        }
        self.input.write_all(log)?;
        self.input.write_all(base)?;
        self.input.flush()?;
        let (marker, replay, bytes) = self.receive()?;
        assert_eq!(marker, fields[8]);
        assert_eq!(replay, fields[5] != 0);
        Ok(bytes)
    }

    fn load(&mut self, bytes: &[u8], replay: bool) -> Result<(u32, bool, Vec<u8>), Error> {
        self.input.write_all(&[1, u8::from(replay)])?;
        self.input.write_all(&(bytes.len() as u32).to_le_bytes())?;
        self.input.write_all(bytes)?;
        self.input.flush()?;
        self.receive()
    }
}

impl Drop for Oracle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn check_reference_files(root: &Path) -> Result<usize, Error> {
    let c_source = std::fs::read_to_string(root.join("src/zelda_rtl.c"))?;
    let list = c_source
        .split("kReferenceSaves[] = {")
        .nth(1)
        .and_then(|tail| tail.split("};").next())
        .ok_or("missing original reference-save list")?;
    let names: Vec<_> = list
        .split('"')
        .enumerate()
        .filter_map(|(index, value)| (index % 2 == 1).then_some(value))
        .collect();
    assert_eq!(names.len(), 13);
    let mut comparisons = 0;
    for name in names {
        let bytes = std::fs::read(root.join("saves/ref").join(name))?;
        let independent = legacy_snapshot::parse(&bytes)?;
        assert_eq!(
            independent.to_bytes(),
            bytes,
            "{name}: independent canonical file"
        );
        let h = header(&bytes);
        assert_eq!(
            h,
            [
                1,
                independent.header.total_frames,
                independent.log.len() as u32,
                independent.header.last_inputs,
                independent.header.frames_since_last,
                independent.header.replay_position << 1
                    | u32::from(independent.base_snapshot.is_some()),
                legacy_snapshot::SNAPSHOT_LEN as u32,
                independent.header.replay_frame_counter
            ]
        );
        let base = independent
            .base_snapshot
            .as_ref()
            .map(|snapshot| snapshot.as_bytes());
        for replay_from_start in [false, true] {
            let (recorder, selected) = StateRecorder::decode_state_file(&bytes, replay_from_start)
                .map_err(|error| format!("{name}: production rejected reference: {error:?}"))?;
            assert_eq!(
                recorder.total_frames(),
                independent.header.total_frames,
                "{name}"
            );
            assert_eq!(recorder.log(), independent.log, "{name}: log");
            assert_eq!(
                recorder.base_snapshot(),
                base.unwrap_or_default(),
                "{name}: base"
            );
            let mut expected_header = independent.header;
            if replay_from_start {
                assert_eq!(
                    selected,
                    base.unwrap_or_default(),
                    "{name}: selected base/reset"
                );
                assert_eq!(recorder.last_inputs(), 0);
                assert_eq!(recorder.frames_since_last(), 0);
                assert!(recorder.is_replaying());
                expected_header.last_inputs = 0;
                expected_header.frames_since_last = 0;
                expected_header.replay_position = 0;
                expected_header.replay_frame_counter = 0;
            } else {
                assert_eq!(
                    selected,
                    independent.current_snapshot.as_bytes(),
                    "{name}: selected current"
                );
                assert_eq!(
                    u32::from(recorder.last_inputs()),
                    independent.header.last_inputs
                );
                assert_eq!(
                    recorder.frames_since_last(),
                    independent.header.frames_since_last
                );
                assert_eq!(
                    recorder.is_replaying(),
                    independent.header.replay_frame_counter != 0
                );
                if !recorder.is_replaying() {
                    expected_header.replay_position = 0;
                }
            }
            let expected = legacy_snapshot::encode(
                expected_header,
                independent.log,
                base,
                independent.current_snapshot.as_bytes(),
            )?;
            let encoded = recorder.encode_state_file(independent.current_snapshot.as_bytes());
            assert_eq!(
                encoded, expected,
                "{name}: public codec vs independent encoder"
            );
            if expected_header == independent.header {
                assert_eq!(encoded, bytes, "{name}: canonical byte-identical re-encode");
            }
            comparisons += 1;
        }
    }
    Ok(comparisons)
}

fn main() -> Result<(), Error> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: check_recording_file oracle-executable zelda3_assets.dat".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    audit_private_abi(&root)?;
    let reference_comparisons = check_reference_files(&root)?;
    let mut oracle = Oracle::spawn(&args[1], &args[2])?;
    let (_, initial_replay, initial) = oracle.receive()?;
    assert!(!initial_replay);
    let template = current(&initial).to_vec();
    assert_eq!(template.len(), LEGACY_SNAPSHOT_LEN);
    assert_eq!(
        initial,
        StateRecorder::default().encode_state_file(&template)
    );
    let mut cases = 1;
    // Header boundaries, independent optional-base and replay flags, and empty
    // versus nonempty opaque logs. No replay log is executed by this harness.
    for base_present in [false, true] {
        for replay in [false, true] {
            for total in [0, 1, 0x7fff_ffff, u32::MAX] {
                for (elapsed, inputs, position, frame) in [
                    (0, 0, 0, 0),
                    (1, 0x1234, 1, 1),
                    (u32::MAX, 0xffff, 0x7fff_ffff, u32::MAX),
                ] {
                    for log in [vec![], vec![0x11, 0xcf, 0xff, 0, 0x27, 0x80, 0xff]] {
                        let mut base = if base_present {
                            template.clone()
                        } else {
                            vec![]
                        };
                        if base_present {
                            base[WRAM_MARKER..WRAM_MARKER + 4]
                                .copy_from_slice(&0xbace_0123u32.to_le_bytes());
                        }
                        let mut snapshot = template.clone();
                        snapshot[WRAM_MARKER..WRAM_MARKER + 4]
                            .copy_from_slice(&0xc011_4567u32.to_le_bytes());
                        let h = [
                            1,
                            total,
                            log.len() as u32,
                            inputs,
                            elapsed,
                            u32::from(base_present) | if replay { position << 1 } else { 0 },
                            LEGACY_SNAPSHOT_LEN as u32,
                            if replay { frame } else { 0 },
                        ];
                        let source = fixture(h, &log, &base, &snapshot);
                        let (mut rust, _) = StateRecorder::decode_state_file(&source, false)
                            .map_err(|error| format!("valid C fixture rejected: {error:?}"))?;
                        // A live replay at frame zero is representable on Save
                        // even though resume derives its mode from hdr[7].
                        if replay && frame == 0 {
                            rust.start_replay();
                            assert_eq!(rust.last_inputs(), inputs as u16);
                            assert_eq!(rust.frames_since_last(), elapsed);
                        }
                        let fields = [
                            total,
                            inputs,
                            elapsed,
                            position,
                            frame,
                            u32::from(replay),
                            log.len() as u32,
                            base.len() as u32,
                            0xc011_4567,
                        ];
                        let saved = oracle.save(fields, &log, &base)?;
                        assert_eq!(header(&saved), h, "original C Save header");
                        assert_eq!(
                            rust.encode_state_file(current(&saved)),
                            saved,
                            "C Save vs Rust encode"
                        );
                        cases += 1;
                        for replay_from_start in [false, true] {
                            let (decoded, selected) =
                                StateRecorder::decode_state_file(&source, replay_from_start)
                                    .map_err(|error| {
                                        format!("valid C fixture rejected: {error:?}")
                                    })?;
                            let (marker, c_replaying, resaved) =
                                oracle.load(&source, replay_from_start)?;
                            assert_eq!(decoded.is_replaying(), c_replaying);
                            let expected_marker = if selected.is_empty() {
                                0
                            } else {
                                u32::from_le_bytes(
                                    selected[WRAM_MARKER..WRAM_MARKER + 4].try_into()?,
                                )
                            };
                            assert_eq!(
                                marker, expected_marker,
                                "C Load selected wrong machine snapshot"
                            );
                            assert_eq!(
                                decoded.encode_state_file(current(&resaved)),
                                resaved,
                                "C Load/Save vs Rust decode/encode"
                            );
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    // C Load narrows the raw 32-bit input word into uint16 and accepts packed
    // resume positions independently of the replay-frame word. Trailing file
    // bytes are also deliberately ignored by both public adapters.
    for inputs in [0x1234_5678, u32::MAX] {
        for packed in [0, 1, 0x8000_0000, u32::MAX] {
            for replay_frame in [0, u32::MAX] {
                let mut snapshot = template.clone();
                snapshot[WRAM_MARKER..WRAM_MARKER + 4]
                    .copy_from_slice(&0xc011_4567u32.to_le_bytes());
                let mut base = if packed & 1 != 0 {
                    template.clone()
                } else {
                    vec![]
                };
                if !base.is_empty() {
                    base[WRAM_MARKER..WRAM_MARKER + 4]
                        .copy_from_slice(&0xbace_0123u32.to_le_bytes());
                }
                let h = [
                    1,
                    u32::MAX,
                    0,
                    inputs,
                    0x8000_0000,
                    packed,
                    LEGACY_SNAPSHOT_LEN as u32,
                    replay_frame,
                ];
                let mut source = fixture(h, &[], &base, &snapshot);
                source.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
                for replay_from_start in [false, true] {
                    let (decoded, selected) =
                        StateRecorder::decode_state_file(&source, replay_from_start)
                            .map_err(|error| format!("valid C fixture rejected: {error:?}"))?;
                    let (marker, c_replaying, resaved) = oracle.load(&source, replay_from_start)?;
                    assert_eq!(decoded.is_replaying(), c_replaying);
                    let expected_marker = if selected.is_empty() {
                        0
                    } else {
                        u32::from_le_bytes(selected[WRAM_MARKER..WRAM_MARKER + 4].try_into()?)
                    };
                    assert_eq!(marker, expected_marker);
                    assert_eq!(
                        decoded.encode_state_file(current(&resaved)),
                        resaved,
                        "raw input/position words and ignored tail"
                    );
                    cases += 1;
                }
            }
        }
    }
    println!(
        "recording container C differential: {cases} complete valid-file Save/Load comparisons; private StateRecorder/ByteArray source ABI audited; no malformed C claims"
    );
    println!(
        "recording container independent fixture codec: all 13 reference files, {reference_comparisons} resume/replay comparisons, complete header/log/base/current bytes"
    );
    Ok(())
}
