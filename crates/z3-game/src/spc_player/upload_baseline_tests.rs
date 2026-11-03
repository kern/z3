//! Independent verbatim upload algorithm from accepted dc325d5.
//! C has no source-length boundary, so malformed prefixes are Rust-baseline
//! comparisons, never C out-of-bounds fixtures. Compare the ENTIRE player.
use super::*;

trait FrozenUpload {
    fn frozen_upload(&mut self, data: &[u8]) -> Result<(), UploadError>;
}
impl FrozenUpload for SpcPlayer {
    fn frozen_upload(&mut self, data: &[u8]) -> Result<(), UploadError> {
        self.dsp_write(0x2c, 0);
        self.dsp_write(0x3c, 0);
        self.dsp_write(0x5c, 255);
        let mut p = 0;
        loop {
            if p + 2 > data.len() {
                return Err(UploadError::Truncated);
            }
            let n = u16::from_le_bytes([data[p], data[p + 1]]) as usize;
            p += 2;
            if n == 0 {
                break;
            }
            if p + 2 + n > data.len() {
                return Err(UploadError::Truncated);
            }
            let mut a = u16::from_le_bytes([data[p], data[p + 1]]);
            p += 2;
            for &v in &data[p..p + n] {
                self.ram[a as usize] = v;
                a = a.wrapping_add(1)
            }
            p += n
        }
        self.pause_music_ctr = 0;
        self.port_to_snes[0] = 0;
        self.port1_active = 0;
        self.port2_active = 0;
        self.port3_active = 0;
        self.is_chan_on = 0;
        self.input_ports = [0; 4];
        Ok(())
    }
}

fn seed(history_count: u32) -> SpcPlayer {
    let mut player = SpcPlayer::new();
    player.initialize();
    for (at, byte) in player.ram.iter_mut().enumerate() {
        *byte = (at as u8).wrapping_mul(37).wrapping_add((at >> 8) as u8);
    }
    player.copy_variables_from_ram();
    player.input_ports = [11, 22, 33, 44];
    player.timer_cycles = 73; // Upload must not invent GenerateSamples validation.
    player.history = Some(DspRegWriteHistory {
        count: history_count,
        addr: [0x35; 256],
        val: [0xc7; 256],
    });
    player
}
fn compare(base: &SpcPlayer, data: &[u8]) {
    let mut actual = base.clone();
    let mut expected = base.clone();
    let expected_result = expected.frozen_upload(data);
    assert_eq!(
        actual.upload(data),
        expected_result,
        "prefix length {}",
        data.len()
    );
    assert!(
        actual == expected,
        "complete player mismatch at input length {}",
        data.len()
    );
}
fn block(output: &mut Vec<u8>, address: u16, payload: &[u8]) {
    output.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    output.extend_from_slice(&address.to_le_bytes());
    output.extend_from_slice(payload);
}

#[test]
fn every_malformed_prefix_preserves_frozen_upload_publication() {
    let mut stream = Vec::new();
    block(&mut stream, 0xffff, &[1, 2, 3]);
    block(&mut stream, 0x3d00, &(0..=255u8).collect::<Vec<_>>());
    block(&mut stream, 0x3e00, &[9, 8, 7, 6, 5, 4, 3, 2, 1]);
    block(&mut stream, 0, &[0xff; 17]); // overlaps/writes imported state without importing it.
    stream.extend_from_slice(&[0, 0, 0x99, 0x88, 0x77]);
    for history_count in [0, 254, 255, 256] {
        let base = seed(history_count);
        for length in 0..=stream.len() {
            compare(&base, &stream[..length]);
        }
    }
}

#[test]
fn maximum_upload_wrap_and_incomplete_block_match_frozen_algorithm() {
    let base = seed(0);
    let mut stream = Vec::new();
    let payload: Vec<_> = (0..65535usize)
        .map(|i| (i as u8).wrapping_mul(19))
        .collect();
    block(&mut stream, 0xffff, &payload);
    stream.extend_from_slice(&[0, 0, 0x55]);
    for length in [0, 1, 2, 3, 4, 65538, 65539, 65540, 65541, 65542] {
        compare(&base, &stream[..length]);
    }
}

#[test]
fn errors_mute_dsp_but_never_reset_or_partially_commit_current_block() {
    let base = seed(0);
    let mut actual = base.clone();
    assert_eq!(
        actual.upload(&[3, 0, 0xff, 0xff, 1, 2]),
        Err(UploadError::Truncated)
    );
    assert!(
        actual.ram == base.ram,
        "truncated current block changed RAM"
    );
    assert_eq!(actual.input_ports, base.input_ports);
    assert_eq!(actual.port1_active, base.port1_active);
    let history = actual.history.as_ref().unwrap();
    assert_eq!(&history.addr[..3], &[0x2c, 0x3c, 0x5c]);
    assert_eq!(&history.val[..3], &[0, 0, 255]);
    assert_eq!(history.count, 3);
    let mut input = Vec::new();
    block(&mut input, 0xffff, &[0x12, 0x34]);
    input.extend_from_slice(&[3, 0, 0, 0, 0xab]);
    let mut actual = base.clone();
    assert_eq!(actual.upload(&input), Err(UploadError::Truncated));
    let mut expected = base.ram.clone();
    expected[0xffff] = 0x12;
    expected[0] = 0x34;
    assert!(
        actual.ram == expected,
        "prior block/current block commit differs"
    );
    assert_eq!(actual.input_ports, base.input_ports);
}
