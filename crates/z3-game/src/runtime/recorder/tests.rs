use super::frozen::FrozenRecorder;
use super::*;
use z3_core::WRAM_SIZE;

fn from_frozen(old: FrozenRecorder) -> StateRecorder {
    StateRecorder {
        base_snapshot: old.base_snapshot,
        tape: RecordedInputs {
            bytes: old.log,
            last_inputs: old.last_inputs,
            frames_since_last: old.frames_since_last,
            total_frames: old.total_frames,
            replay: ReplayState {
                position: LogPosition(old.replay_position),
                last_complete: LogPosition(old.replay_position_last_complete),
                frame_counter: old.replay_frame_counter,
                next_command_at: old.replay_next_command_at,
                command: CommandCode(old.replay_command),
                active: old.replay_mode,
            },
        },
    }
}

fn same_state(new: &StateRecorder, old: &FrozenRecorder) {
    assert_eq!(new, &from_frozen(old.clone()));
}

fn compare_steps(mut old: FrozenRecorder, steps: usize) {
    let mut new = from_frozen(old.clone());
    let mut old_ram = vec![0x5a; WRAM_SIZE];
    let mut new_ram = old_ram.clone();
    for _ in 0..steps {
        if !old.replay_mode {
            break;
        }
        let expected =
            old.read_next_replay_state(&mut Ram::new(old_ram.as_mut_slice().try_into().unwrap()));
        let actual =
            new.read_next_replay_state(&mut Ram::new(new_ram.as_mut_slice().try_into().unwrap()));
        assert_eq!(
            actual.map_err(|e| format!("{e:?}")),
            expected.map_err(|e| format!("{e:?}"))
        );
        same_state(&new, &old);
        assert_eq!(new_ram, old_ram);
        // Repeat errors as well: partial patches must replay the same prefix.
    }
}

#[test]
fn frozen_scheduler_matches_every_raw_command_and_payload_prefix() {
    for command in 0u8..=255 {
        let payload = [0, 0xff, 0xfe, 1, 2, 3, 4, 5];
        for length in 0..=payload.len() {
            compare_steps(
                FrozenRecorder {
                    last_inputs: 0xa55a,
                    total_frames: 100,
                    frames_since_last: 7,
                    log: payload[..length].to_vec(),
                    replay_position: 0,
                    replay_position_last_complete: usize::MAX,
                    replay_next_command_at: 7,
                    replay_command: command,
                    replay_mode: true,
                    ..FrozenRecorder::default()
                },
                3,
            );
        }
        for extension in [vec![], vec![0], vec![255], vec![255, 0], vec![255, 1]] {
            let mut log = vec![0x00, command];
            log.extend_from_slice(&extension);
            let mut old = FrozenRecorder {
                log,
                total_frames: 50,
                ..FrozenRecorder::default()
            };
            old.start_replay();
            compare_steps(old, 4);
        }
    }
}

#[test]
fn frozen_scheduler_matches_mixed_log_truncation_and_wrapped_patch_prefixes() {
    let mut source = FrozenRecorder::default();
    source.record(0x801);
    source.record_patch(0x1fffe, &[1, 2, 3, 4, 5]);
    source.record(0x123);
    source.record_patch(0xffff, &[99, 100]);
    source.record(0);
    for length in 0..=source.log.len() {
        let mut old = source.clone();
        old.log.truncate(length);
        old.total_frames = 20;
        old.start_replay();
        compare_steps(old, 10);
    }
}

#[test]
fn frozen_tape_recording_matches_thresholds_high_bits_and_counter_wrap() {
    for frames in [0, 1, 14, 15, 16, 254, 255, 256, 269, 270, 510, 511] {
        for length in [1, 2, 3, 4, 5, 258, 259] {
            let mut old = FrozenRecorder {
                frames_since_last: frames,
                total_frames: u32::MAX,
                ..FrozenRecorder::default()
            };
            let mut new = from_frozen(old.clone());
            for input in [0xf000, 0xf801, 0x0001, 0xfff, 0] {
                old.record(input);
                new.record(input);
                same_state(&new, &old);
            }
            old.frames_since_last = frames;
            new.tape.frames_since_last = frames;
            let bytes = vec![0xa5; length];
            old.record_patch(0x1ffff, &bytes);
            new.record_patch(0x1ffff, &bytes);
            same_state(&new, &old);
        }
    }
    let mut old = FrozenRecorder {
        last_inputs: 5,
        frames_since_last: u32::MAX,
        total_frames: u32::MAX,
        ..FrozenRecorder::default()
    };
    let mut new = from_frozen(old.clone());
    old.record(5);
    new.record(5);
    same_state(&new, &old);
    for frames in [0x8000_0000, u32::MAX] {
        let mut old = FrozenRecorder {
            frames_since_last: frames,
            ..FrozenRecorder::default()
        };
        let mut new = from_frozen(old.clone());
        old.record(1);
        new.record(1);
        same_state(&new, &old);
    }
}

#[test]
fn frozen_replay_eof_counter_and_stop_boundaries() {
    for frames in [0, 1, u32::MAX - 1, u32::MAX] {
        for counter in [0, u32::MAX - 1, u32::MAX] {
            for total in [0, 1, u32::MAX] {
                for position in [0, 1, usize::MAX] {
                    let old = FrozenRecorder {
                        frames_since_last: frames,
                        total_frames: total,
                        replay_mode: true,
                        replay_position: position,
                        replay_position_last_complete: position,
                        replay_next_command_at: u32::MAX,
                        replay_frame_counter: counter,
                        ..FrozenRecorder::default()
                    };
                    compare_steps(old.clone(), 3);
                    let mut new = from_frozen(old.clone());
                    let mut old = old;
                    old.stop_replay();
                    new.stop_replay();
                    same_state(&new, &old);
                    old.stop_replay();
                    new.stop_replay();
                    same_state(&new, &old);
                }
            }
        }
    }
}

#[test]
fn frozen_clear_history_preserves_raw_header_bits_tail_and_panic_mutations() {
    for active in [false, true] {
        for delay in [0, 2, 7, u32::MAX] {
            for position in [0, 2, 3, 4] {
                let mut old = FrozenRecorder {
                    last_inputs: 0xf005,
                    frames_since_last: 1,
                    total_frames: 1,
                    log: vec![9, 8, 7],
                    base_snapshot: vec![1],
                    replay_mode: active,
                    replay_next_command_at: delay,
                    replay_command: 0x05,
                    replay_position: position,
                    replay_frame_counter: 2,
                    ..FrozenRecorder::default()
                };
                let mut new = from_frozen(old.clone());
                let expected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    old.clear_key_log(&[3, 2, 1])
                }));
                let actual = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    new.clear_key_log(&[3, 2, 1])
                }));
                assert_eq!(actual.is_err(), expected.is_err());
                same_state(&new, &old);
            }
        }
    }
}

#[test]
fn frozen_variable_length_wrap_and_lazy_error_after_large_patch_count() {
    // 16,843,009 * 255 == u32::MAX; one further 255 wraps to254.
    let mut extension = vec![255; (u32::MAX / 255) as usize + 1];
    extension.push(0);
    for command in [0x0f, 0xcd] {
        let mut log = vec![command];
        log.extend_from_slice(&extension);
        let mut old = FrozenRecorder {
            log,
            total_frames: 50,
            ..FrozenRecorder::default()
        };
        old.start_replay();
        compare_steps(old, 1);
    }
    // The same extension in a pending payload is a count, not a delay.
    extension.extend_from_slice(&[0x1f, 0xff, 0xaa]);
    compare_steps(
        FrozenRecorder {
            log: extension,
            total_frames: 100,
            replay_mode: true,
            replay_command: 0xcc,
            replay_position_last_complete: usize::MAX,
            ..FrozenRecorder::default()
        },
        2,
    );
    // A maximum decoded count remains a usize addition on this platform;
    // payload truncation occurs after each available byte, not on count size.
    #[cfg(target_pointer_width = "64")]
    {
        let mut log = vec![255; (u32::MAX / 255) as usize];
        log.extend_from_slice(&[0, 0xff, 0xff, 0x44, 0x55]);
        compare_steps(
            FrozenRecorder {
                log,
                total_frames: 100,
                replay_mode: true,
                replay_command: 0xce,
                replay_position_last_complete: usize::MAX,
                ..FrozenRecorder::default()
            },
            2,
        );
    }
}

#[test]
fn frozen_clear_reemits_every_raw_pending_command_without_normalization() {
    for raw in 0u8..=255 {
        let mut old = FrozenRecorder {
            log: vec![0x12, 0x34, 0x56],
            replay_mode: true,
            replay_command: raw,
            replay_position: 1,
            replay_next_command_at: 2,
            ..FrozenRecorder::default()
        };
        let mut new = from_frozen(old.clone());
        old.clear_key_log(&[]);
        new.clear_key_log(&[]);
        same_state(&new, &old);
    }
}

#[test]
fn recorder_matches_toggle_delay_and_extended_patch_encoding() {
    let mut recorder = StateRecorder::default();
    recorder.record(0);
    recorder.record(1);
    for _ in 0..20 {
        recorder.record(1);
    }
    recorder.record(0x801);
    recorder.record_patch(0x1fffe, &[1, 2, 3, 4, 5]);
    assert_eq!(
        recorder.log(),
        [0x01, 0xbf, 6, 0xcf, 0, 1, 0xff, 0xfe, 1, 2, 3, 4, 5]
    );
}

#[test]
fn replay_applies_input_toggles_and_patches_on_the_recorded_frames() {
    let mut bytes = [0; WRAM_SIZE];
    let mut ram = Ram::new(&mut bytes);
    let mut recorder = from_frozen(FrozenRecorder {
        total_frames: 2,
        // Toggle input bit 0 immediately, then patch $0123 one frame later.
        log: vec![0x00, 0xc1, 0x00, 0x01, 0x23, 0xaa],
        ..FrozenRecorder::default()
    });
    recorder.start_replay();
    assert_eq!(recorder.read_next_replay_state(&mut ram), Ok(1));
    assert_eq!(ram.read_u8(0x123), 0);
    assert_eq!(recorder.read_next_replay_state(&mut ram), Ok(1));
    assert_eq!(ram.read_u8(0x123), 0xaa);
    assert!(!recorder.is_replaying());
}

#[test]
fn recorder_state_file_round_trips_c_header_and_snapshot_selection() {
    let recorder = from_frozen(FrozenRecorder {
        last_inputs: 0x123,
        frames_since_last: 7,
        total_frames: 99,
        log: vec![0x01, 0x42],
        base_snapshot: vec![1, 2, 3],
        ..FrozenRecorder::default()
    });
    let encoded = recorder.encode_state_file(&[4, 5, 6]);
    assert_eq!(&encoded[..4], &1u32.to_le_bytes());
    let (resumed, snapshot) =
        StateRecorder::decode_state_file(&encoded, false).expect("valid recorder file");
    assert_eq!(snapshot, [4, 5, 6]);
    assert_eq!(resumed.last_inputs(), 0x123);
    assert_eq!(resumed.log(), [0x01, 0x42]);

    let (replayed, snapshot) =
        StateRecorder::decode_state_file(&encoded, true).expect("valid recorder file");
    assert_eq!(snapshot, [1, 2, 3]);
    assert!(replayed.is_replaying());
    assert_eq!(replayed.last_inputs(), 0);
}

#[test]
fn clearing_key_log_rebases_active_inputs_on_current_snapshot() {
    let mut recorder = from_frozen(FrozenRecorder {
        last_inputs: 0x0005,
        frames_since_last: 12,
        total_frames: 30,
        log: vec![0xaa, 0xbb],
        ..FrozenRecorder::default()
    });
    recorder.clear_key_log(&[9, 8, 7]);
    assert_eq!(recorder.base_snapshot(), [9, 8, 7]);
    assert_eq!(recorder.log(), [0x00, 0x20]);
    assert_eq!(recorder.frames_since_last(), 0);
    assert_eq!(recorder.total_frames(), 0);
}
