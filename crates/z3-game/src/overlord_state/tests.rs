use super::*;
fn image() -> Box<[u8; WRAM_SIZE]> {
    Box::new(std::array::from_fn(|i| {
        (i.wrapping_mul(37) ^ (i >> 8)) as u8
    }))
}
#[test]
fn every_controller_and_indexed_boss_byte_preserves_full_memory() {
    macro_rules! field {
        ($get:ident,$set:ident,$a:literal) => {
            for i in [0, 7, 8, 15, 20, 31, 255] {
                let mut bytes = image();
                let mut expected = bytes.clone();
                let v = expected[$a + i] ^ 0xff;
                expected[$a + i] = v;
                let mut ram = Ram::new(&mut bytes);
                ram.$set(i, v);
                assert_eq!(ram.$get(i), v);
                assert_eq!(bytes, expected);
            }
        };
    }
    field!(controller_kind, set_controller_kind, 0xb00);
    field!(controller_x_low, set_controller_x_low, 0xb08);
    field!(controller_x_high, set_controller_x_high, 0xb10);
    field!(controller_y_low, set_controller_y_low, 0xb18);
    field!(controller_y_high, set_controller_y_high, 0xb20);
    field!(controller_scratch_a, set_controller_scratch_a, 0xb28);
    field!(controller_timer, set_controller_timer, 0xb30);
    field!(controller_scratch_c, set_controller_scratch_c, 0xb38);
    field!(controller_floor, set_controller_floor, 0xb40);
    field!(controller_spawned_area, set_controller_spawned_area, 0xcca);
    field!(boss_orbit_x_low, set_boss_orbit_x_low, 0xb10);
    field!(boss_orbit_x_high, set_boss_orbit_x_high, 0xb20);
    field!(boss_orbit_y_low, set_boss_orbit_y_low, 0xb30);
    field!(boss_orbit_y_high, set_boss_orbit_y_high, 0xb40);
    field!(boss_arrghi_x_low, set_boss_arrghi_x_low, 0xb08);
    field!(boss_arrghi_x_high, set_boss_arrghi_x_high, 0xb18);
    field!(boss_arrghi_y_low, set_boss_arrghi_y_low, 0xb28);
    field!(boss_arrghi_y_high, set_boss_arrghi_y_high, 0xb38);
    field!(
        boss_helmasaur_leg_frame,
        set_boss_helmasaur_leg_frame,
        0xb08
    );
    field!(boss_helmasaur_tail_x, set_boss_helmasaur_tail_x, 0xb0d);
    field!(boss_helmasaur_tail_y, set_boss_helmasaur_tail_y, 0xb1d);
    field!(boss_trinexx_control, set_boss_trinexx_control, 0xb08);
}
#[test]
fn every_boss_scalar_byte_preserves_full_memory() {
    macro_rules! field {
        ($get:ident,$set:ident,$a:literal) => {
            for v in [0, 1, 0x7f, 0x80, 0xff] {
                let mut bytes = image();
                let mut expected = bytes.clone();
                expected[$a] = v;
                let mut ram = Ram::new(&mut bytes);
                ram.$set(v);
                assert_eq!(ram.$get(), v);
                assert_eq!(bytes, expected);
            }
        };
    }
    field!(boss_orbit_phase_low, set_boss_orbit_phase_low, 0xb08);
    field!(boss_orbit_phase_high, set_boss_orbit_phase_high, 0xb09);
    field!(boss_orbit_radius, set_boss_orbit_radius, 0xb0a);
    field!(boss_orbit_count, set_boss_orbit_count, 0xb0b);
    field!(boss_orbit_speed, set_boss_orbit_speed, 0xb0c);
    field!(
        boss_helmasaur_animation_clock,
        set_boss_helmasaur_animation_clock,
        0xb0c
    );
    field!(
        boss_helmasaur_phase_low,
        set_boss_helmasaur_phase_low,
        0xb2d
    );
    field!(boss_helmasaur_turn, set_boss_helmasaur_turn, 0xb2e);
    field!(boss_helmasaur_radius, set_boss_helmasaur_radius, 0xb2f);
    field!(
        boss_helmasaur_swing_state,
        set_boss_helmasaur_swing_state,
        0xb30
    );
    field!(
        boss_helmasaur_velocity_low,
        set_boss_helmasaur_velocity_low,
        0xb31
    );
    field!(
        boss_helmasaur_velocity_high,
        set_boss_helmasaur_velocity_high,
        0xb32
    );
    field!(
        boss_helmasaur_next_tail,
        set_boss_helmasaur_next_tail,
        0xb33
    );
    field!(
        boss_trinexx_target_x_low,
        set_boss_trinexx_target_x_low,
        0xb08
    );
    field!(
        boss_trinexx_target_y_low,
        set_boss_trinexx_target_y_low,
        0xb09
    );
    field!(boss_trinexx_bend, set_boss_trinexx_bend, 0xb0a);
    field!(boss_trinexx_direction, set_boss_trinexx_direction, 0xb0b);
    field!(boss_trinexx_clock, set_boss_trinexx_clock, 0xb0c);
    field!(boss_trinexx_delay, set_boss_trinexx_delay, 0xb0d);
    field!(
        boss_trinexx_startup_delay,
        set_boss_trinexx_startup_delay,
        0xb0e
    );
    field!(
        boss_trinexx_tail_phase_low,
        set_boss_trinexx_tail_phase_low,
        0xb0f
    );
    field!(boss_trinexx_ready, set_boss_trinexx_ready, 0xb10);
}
#[test]
fn all_boss_words_keep_endian_odd_aliases_and_every_other_byte() {
    macro_rules! field {
        ($get:ident,$set:ident,$a:literal) => {
            for b in 0..=255u16 {
                for v in [b, b << 8, b | ((255 - b) << 8)] {
                    let mut bytes = image();
                    let mut expected = bytes.clone();
                    expected[$a..$a + 2].copy_from_slice(&v.to_le_bytes());
                    let mut ram = Ram::new(&mut bytes);
                    ram.$set(v);
                    assert_eq!(ram.$get(), v);
                    assert_eq!(bytes, expected);
                }
            }
        };
    }
    field!(boss_orbit_phase, set_boss_orbit_phase, 0xb08);
    field!(boss_helmasaur_phase, set_boss_helmasaur_phase, 0xb2d);
    field!(boss_helmasaur_velocity, set_boss_helmasaur_velocity, 0xb31);
    field!(boss_trinexx_tail_phase, set_boss_trinexx_tail_phase, 0xb0f);
}
#[test]
fn coordinate_and_source_stride_writes_preserve_full_memory_including_b58() {
    for i in [0, 7, 8, 15, 31, 255] {
        for v in [0u16, 0xff, 0x100, 0x7fff, 0x8000, 0xffff] {
            let mut bytes = image();
            let mut expected = bytes.clone();
            expected[0xb08 + i] = v as u8;
            expected[0xb10 + i] = (v >> 8) as u8;
            expected[0xb18 + i] = v as u8;
            expected[0xb20 + i] = (v >> 8) as u8;
            expected[0xb48 + i * 2..0xb48 + i * 2 + 2].copy_from_slice(&v.to_le_bytes());
            let mut ram = Ram::new(&mut bytes);
            ram.controller_mut(i).set_x(v);
            ram.controller_mut(i).set_y(v);
            ram.set_controller_source_position(i, v);
            assert_eq!(ram.controller(i).x(), v);
            assert_eq!(ram.controller(i).y(), v);
            assert_eq!(ram.controller_source_position(i), v);
            assert_eq!(bytes, expected);
        }
    }
}
#[test]
fn nominal_pool_scans_and_kind_only_reset_do_not_own_boss_payload() {
    for mask in 0u16..256 {
        let mut bytes = image();
        let mut ram = Ram::new(&mut bytes);
        for i in 0..8 {
            ram.set_controller_kind(i, u8::from(mask & (1 << i) != 0));
        }
        let mut expected = Box::new(*ram.bytes_mut());
        assert_eq!(
            ram.controllers().find_free(),
            (0..8).rev().find(|&i| mask & (1 << i) == 0)
        );
        assert!(ram.controllers().room_clear());
        assert_eq!(ram.bytes_mut(), &*expected);
        expected[0xb00..0xb08].fill(0);
        ram.clear_controller_kinds_reverse();
        assert_eq!(ram.bytes_mut(), &*expected);
    }
    for kind in 0..=255 {
        for i in 0..8 {
            let mut bytes = Box::new([0; WRAM_SIZE]);
            let mut ram = Ram::new(&mut bytes);
            ram.set_controller_kind(i, kind);
            assert_eq!(ram.controllers().room_clear(), !matches!(kind, 20 | 24));
        }
    }
}
#[test]
fn controller_boss_and_select_file_aliases_remain_live() {
    let mut bytes = image();
    let mut ram = Ram::new(&mut bytes);
    ram.set_boss_helmasaur_phase(0xa1b2);
    assert_eq!(ram.controller_scratch_a(5), 0xb2);
    assert_eq!(ram.controller_scratch_a(6), 0xa1);
    assert_eq!(ram.boss_helmasaur_turn(), 0xa1);
    ram.set_boss_helmasaur_velocity(0xc3d4);
    assert_eq!(ram.controller_timer(1), 0xd4);
    assert_eq!(ram.boss_helmasaur_velocity_high(), 0xc3);
    ram.set_boss_trinexx_tail_phase(0xe5f6);
    assert_eq!(ram.controller_x_low(7), 0xf6);
    assert_eq!(ram.controller_x_high(0), 0xe5);
    assert_eq!(ram.boss_trinexx_ready(), 0xe5);
    ram.write_u8(0xb10, 0x42); // Select-file NAME_COLUMN aliases the same byte.
    assert_eq!(ram.boss_trinexx_tail_phase(), 0x42f6);
    ram.set_boss_orbit_x_low(8, 0x51);
    assert_eq!(ram.controller_y_low(0), 0x51);
    assert_eq!(ram.boss_arrghi_x_high(0), 0x51);
    ram.set_boss_arrghi_x_low(8, 0x62);
    assert_eq!(ram.boss_orbit_x_low(0), 0x62);
    ram.set_boss_helmasaur_tail_x(3, 0x73);
    assert_eq!(ram.boss_trinexx_ready(), 0x73);
}
