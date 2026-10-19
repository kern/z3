//! Whole-state valid-program differential against actual exported C recorder
//! functions. Malformed/overflow domains belong to the frozen Rust tests.
use super::*;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

const WRAM_LEN: usize = 0x20000;

fn state(recorder: &StateRecorder) -> [u32; 9] {
    let tape = &recorder.tape;
    let replay = &tape.replay;
    [
        u32::from(tape.last_inputs),
        tape.frames_since_last,
        tape.total_frames,
        replay.position.0 as u32,
        replay.last_complete.0 as u32,
        replay.frame_counter,
        replay.next_command_at,
        u32::from(replay.command.0),
        u32::from(replay.active),
    ]
}

struct Response {
    returned: u32,
    state: [u32; 9],
    ram: Box<[u8; WRAM_LEN]>,
    log: Vec<u8>,
    base: Vec<u8>,
}

struct Oracle {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
    operations: usize,
    cases: usize,
    clears: usize,
}

impl Oracle {
    fn spawn() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut child = Command::new(root.join("build/c-oracle/recorder_replay_oracle"))
            .arg(root.join("zelda3_assets.dat"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("build original C recorder replay oracle first");
        Self {
            input: BufWriter::new(child.stdin.take().unwrap()),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            operations: 0,
            cases: 0,
            clears: 0,
        }
    }

    fn word(&mut self, value: u32) {
        self.input.write_all(&value.to_le_bytes()).unwrap();
    }

    fn receive(&mut self) -> Response {
        self.input.flush().unwrap();
        let mut words = [0; 48];
        self.output.read_exact(&mut words).unwrap();
        let words: [u32; 12] = std::array::from_fn(|index| {
            u32::from_le_bytes(words[index * 4..index * 4 + 4].try_into().unwrap())
        });
        let mut response = Response {
            returned: words[0],
            state: words[1..10].try_into().unwrap(),
            ram: Box::new([0; WRAM_LEN]),
            log: vec![0; words[10] as usize],
            base: vec![0; words[11] as usize],
        };
        self.output.read_exact(response.ram.as_mut_slice()).unwrap();
        self.output.read_exact(&mut response.log).unwrap();
        self.output.read_exact(&mut response.base).unwrap();
        self.operations += 1;
        response
    }

    fn compare(
        &self,
        response: &Response,
        recorder: &StateRecorder,
        ram: &[u8; WRAM_LEN],
        returned: u32,
        context: &str,
    ) {
        assert_eq!(response.returned, returned, "{context}: returned input");
        assert_eq!(
            response.state,
            state(recorder),
            "{context}: all nine C recorder scalars"
        );
        assert_eq!(response.log, recorder.log(), "{context}: exact log");
        assert_eq!(
            response.base,
            recorder.base_snapshot(),
            "{context}: exact base snapshot"
        );
        if response.ram.as_slice() != ram {
            let first = response
                .ram
                .iter()
                .zip(ram)
                .position(|(c, rust)| c != rust)
                .unwrap();
            panic!(
                "{context}: full WRAM mismatch @{first:#x}: C={} Rust={}",
                response.ram[first], ram[first]
            );
        }
    }

    fn begin(&mut self, recorder: &StateRecorder) -> Box<[u8; WRAM_LEN]> {
        self.input.write_all(&[0]).unwrap();
        for value in state(recorder) {
            self.word(value);
        }
        self.word(recorder.log().len() as u32);
        self.input.write_all(recorder.log()).unwrap();
        let seed = std::array::from_fn::<_, WRAM_LEN, _>(|index| (index as u8).wrapping_mul(37));
        self.input.write_all(&seed).unwrap();
        let response = self.receive();
        // Init explicitly executes original Save once to canonicalize HDMA,
        // SPC mapping, MSU volume/resume bytes. This is the baseline, not an
        // ignored mutation from a tested recorder operation. Every later Clear
        // is required by the C harness to preserve all WRAM and SPC RAM.
        self.compare(
            &response,
            recorder,
            &response.ram,
            u32::MAX,
            "canonical initialization",
        );
        self.cases += 1;
        response.ram
    }

    fn record(&mut self, recorder: &mut StateRecorder, ram: &[u8; WRAM_LEN], inputs: u16) {
        self.input.write_all(&[1]).unwrap();
        self.word(u32::from(inputs));
        recorder.record(inputs);
        let response = self.receive();
        self.compare(&response, recorder, ram, u32::MAX, "Record");
    }

    fn patch(
        &mut self,
        recorder: &mut StateRecorder,
        ram: &[u8; WRAM_LEN],
        address: u32,
        bytes: &[u8],
    ) {
        self.input.write_all(&[2]).unwrap();
        self.word(address);
        self.word(bytes.len() as u32);
        self.input.write_all(bytes).unwrap();
        recorder.record_patch(address, bytes);
        let response = self.receive();
        self.compare(&response, recorder, ram, u32::MAX, "RecordPatchByte");
    }

    fn read(&mut self, recorder: &mut StateRecorder, ram: &mut [u8; WRAM_LEN]) {
        self.input.write_all(&[3]).unwrap();
        let returned = recorder.read_next_replay_state(&mut Ram::new(ram)).unwrap();
        let response = self.receive();
        self.compare(
            &response,
            recorder,
            ram,
            u32::from(returned),
            "ReadNextReplayState",
        );
    }

    fn stop(&mut self, recorder: &mut StateRecorder, ram: &[u8; WRAM_LEN]) {
        self.input.write_all(&[4]).unwrap();
        recorder.stop_replay();
        let response = self.receive();
        self.compare(&response, recorder, ram, u32::MAX, "StopReplay");
    }

    fn clear(&mut self, recorder: &mut StateRecorder, ram: &[u8; WRAM_LEN]) {
        self.input.write_all(&[5]).unwrap();
        let response = self.receive();
        recorder.clear_key_log(&response.base);
        self.compare(
            &response,
            recorder,
            ram,
            u32::MAX,
            "ClearKeyLog at canonical snapshot boundary",
        );
        self.clears += 1;
    }
}

impl Drop for Oracle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn recorder(total: u32, elapsed: u32, inputs: u16, log: &[u8], replay_frame: u32) -> StateRecorder {
    let header = [
        1,
        total,
        log.len() as u32,
        u32::from(inputs),
        elapsed,
        0,
        0,
        replay_frame,
    ];
    let mut bytes: Vec<_> = header.into_iter().flat_map(u32::to_le_bytes).collect();
    bytes.extend_from_slice(log);
    StateRecorder::decode_state_file(&bytes, false).unwrap().0
}

fn vl(bytes: &mut Vec<u8>, mut value: u32) {
    while value >= 255 {
        bytes.push(255);
        value -= 255;
    }
    bytes.push(value as u8);
}

fn toggle(log: &mut Vec<u8>, bit: u8, delay: u32) {
    log.push(bit << 4 | delay.min(15) as u8);
    if delay >= 15 {
        vl(log, delay - 15);
    }
}

fn patch(log: &mut Vec<u8>, address: u32, bytes: &[u8], delay: u32) {
    let size = (bytes.len() - 1).min(3);
    log.push(0xc0 | u8::from(address & 0x10000 != 0) << 1 | (size as u8) << 2 | delay.min(1) as u8);
    if delay >= 1 {
        vl(log, delay - 1);
    }
    if size == 3 {
        vl(log, (bytes.len() - 4) as u32);
    }
    log.extend_from_slice(&[(address >> 8) as u8, address as u8]);
    log.extend_from_slice(bytes);
}

#[test]
#[ignore = "requires original C recorder replay oracle and local assets"]
fn recorder_scheduler_matches_original_c_whole_state() {
    let mut oracle = Oracle::spawn();
    // Record every physical SNES bit, upper input-mask bits which deliberately
    // emit no log event, and exact inline/VL delay boundaries.
    for elapsed in [
        0, 1, 14, 15, 16, 254, 255, 256, 269, 270, 271, 524, 525, 65535,
    ] {
        for bit in 0..16 {
            let mut sr = recorder(u32::MAX, elapsed, 0, &[], 0);
            let ram = oracle.begin(&sr);
            oracle.record(&mut sr, &ram, 1 << bit);
            oracle.record(&mut sr, &ram, 1 << bit);
            oracle.record(&mut sr, &ram, 0);
        }
    }
    // Encoder patch lengths and both address-bank bits, including real 17-bit
    // wrap when replaying later. All count/position/delay C ints stay bounded.
    for address in [0, 0xffff, 0x10000, 0x1fffe, 0x1ffff] {
        for length in [1, 2, 3, 4, 5, 258, 259, 260, 1024] {
            let bytes: Vec<_> = (0..length)
                .map(|index| (index as u8).wrapping_mul(53).wrapping_add(7))
                .collect();
            for delay in [0, 1, 2, 255, 256, 511] {
                let mut sr = recorder(100, delay, 0, &[], 0);
                let ram = oracle.begin(&sr);
                oracle.patch(&mut sr, &ram, address, &bytes);
                let log = sr.log().to_vec();
                let mut replay = recorder(u32::MAX, delay.saturating_sub(1), 0, &log, 1);
                let mut ram = oracle.begin(&replay);
                oracle.read(&mut replay, &mut ram);
                oracle.read(&mut replay, &mut ram);
                oracle.stop(&mut replay, &ram);
                oracle.stop(&mut replay, &ram);
            }
        }
    }
    // All twelve toggles and every inline delay, including the mandatory zero
    // VL terminator for delay 15; delay header remains pending until due.
    for bit in 0..12 {
        for delay in 0u32..=16 {
            let mut log = vec![];
            toggle(&mut log, bit, delay);
            let mut sr = recorder(1000, delay.saturating_sub(1), 0, &log, 1);
            let mut ram = oracle.begin(&sr);
            oracle.read(&mut sr, &mut ram);
            oracle.read(&mut sr, &mut ram);
        }
    }
    // A Stop before the pending payload commits discards that entire command
    // but retains prior complete commands and all inactive scheduler fields.
    for pending_patch in [false, true] {
        let mut log = vec![];
        toggle(&mut log, 0, 0);
        if pending_patch {
            patch(&mut log, 0x1ffff, &[0x17, 0x28, 0x39, 0x4a], 5);
        } else {
            toggle(&mut log, 2, 5);
        }
        let mut sr = recorder(100, 0, 0, &log, 1);
        let mut ram = oracle.begin(&sr);
        oracle.read(&mut sr, &mut ram);
        oracle.stop(&mut sr, &ram);
        oracle.stop(&mut sr, &ram);
        oracle.record(&mut sr, &ram, 3);
        oracle.clear(&mut sr, &ram);
    }
    // These unsigned counter edges do not enter C's signed-int encoder path:
    // identical inputs emit no command, while the empty EOF sentinel consumes
    // no delay bytes. Signed conversion/accumulation overflow is not tested.
    for elapsed in [i32::MAX as u32, u32::MAX] {
        let mut sr = recorder(u32::MAX, elapsed, 0x1234, &[], 0);
        let ram = oracle.begin(&sr);
        oracle.record(&mut sr, &ram, 0x1234);
        oracle.stop(&mut sr, &ram);
    }
    let mut sr = recorder(u32::MAX, u32::MAX, 0, &[], u32::MAX);
    sr.tape.replay.next_command_at = u32::MAX;
    let mut ram = oracle.begin(&sr);
    oracle.read(&mut sr, &mut ram);
    oracle.stop(&mut sr, &ram);
    for total in [0, 1, 2, 3] {
        let mut sr = recorder(total, 0, 0, &[], 0);
        sr.start_replay();
        let mut ram = oracle.begin(&sr);
        oracle.read(&mut sr, &mut ram);
        if sr.is_replaying() {
            oracle.read(&mut sr, &mut ram);
        }
        oracle.stop(&mut sr, &ram);
    }
    // Zero-time chains commit all complete commands in order in one call.
    let mut log = vec![];
    for bit in 0..12 {
        toggle(&mut log, bit, 0);
    }
    patch(&mut log, 0x1ffff, &[1, 2, 3, 4, 5], 0);
    toggle(&mut log, 0, 0);
    let mut sr = recorder(100, 0, 0, &log, 1);
    let mut ram = oracle.begin(&sr);
    oracle.read(&mut sr, &mut ram);
    oracle.stop(&mut sr, &ram);
    oracle.record(&mut sr, &ram, 0x123);
    // Clear ordinary recording, then repeated clear. Full native snapshots
    // are supplied by original C, not synthetic or ROM-derived test fixtures.
    for inputs in [0, 1, 0xfff, 0xf000, 0xffff] {
        let mut sr = recorder(99, 12, inputs, &[0x00, 0x20], 0);
        let ram = oracle.begin(&sr);
        oracle.clear(&mut sr, &ram);
        oracle.clear(&mut sr, &ram);
        oracle.record(&mut sr, &ram, inputs ^ 0x123);
    }
    // Preserve the entire pending command byte when rebasing: raw delay bits
    // are ORed with the adjusted delay rather than canonicalized away.
    for pending_patch in [false, true] {
        for delay in [2, 5, 15, 16, 270] {
            let mut log = vec![];
            toggle(&mut log, 0, 0);
            if pending_patch {
                patch(&mut log, 0x1ffff, &[7, 8, 9, 10, 11], delay);
            } else {
                toggle(&mut log, 2, delay);
            }
            toggle(&mut log, 3, 0);
            let mut sr = recorder(1000, 0, 0, &log, 1);
            let mut ram = oracle.begin(&sr);
            oracle.read(&mut sr, &mut ram);
            oracle.clear(&mut sr, &ram);
            oracle.clear(&mut sr, &ram);
            // Clearing can OR the old inline bits back into the adjusted
            // schedule; follow the actual newly emitted log, not originaltime.
            for _ in 0..delay + 20 {
                oracle.read(&mut sr, &mut ram);
            }
            oracle.clear(&mut sr, &ram); // Active EOF sentinel branch.
            // The original EOF-clear branch retains its old committed cursor
            // even if the new active-input log is shorter. Calling C Stop at
            // that point may enlarge the buffer's advertised size past its
            // allocation. Observe the exact retained state, but do not claim
            // that subsequent unsafe C sequence as a differential oracle.
            if sr.tape.replay.last_complete.0 <= sr.log().len() {
                oracle.stop(&mut sr, &ram);
            }
        }
    }
    println!(
        "recorder scheduler C differential: {} cases, {} full-WRAM/all-nine-scalars/log/base operations, {} idempotent native-boundary Clear calls; no masks or malformed-C claims",
        oracle.cases, oracle.operations, oracle.clears
    );
}
