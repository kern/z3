//! Ordered, typed mapping of native music state to the SPC RAM image.
//!
//! The schemas below follow the C MemMap tables in source order. A channel
//! advances two RAM bytes per voice even for its one-byte fields. Only these
//! fields are mirrored: DSP state, input ports, channel indices, and the rest
//! of RAM retain their independent live values.

use super::{Channel, PatternAddress, PatternCursor, PhraseCursor, SpcPlayer};

type SpcRam = [u8; 65536];

trait MappedValue: Copy {
    const WIDTH: usize;
    fn read(ram: &SpcRam, address: usize) -> Self;
    fn write(self, ram: &mut SpcRam, address: usize);
}

impl MappedValue for u8 {
    const WIDTH: usize = 1;
    fn read(ram: &SpcRam, address: usize) -> Self {
        ram[address]
    }
    fn write(self, ram: &mut SpcRam, address: usize) {
        ram[address] = self;
    }
}

impl MappedValue for u16 {
    const WIDTH: usize = 2;
    fn read(ram: &SpcRam, address: usize) -> Self {
        Self::from_le_bytes([ram[address], ram[address + 1]])
    }
    fn write(self, ram: &mut SpcRam, address: usize) {
        ram[address..address + Self::WIDTH].copy_from_slice(&self.to_le_bytes());
    }
}

impl MappedValue for PhraseCursor {
    const WIDTH: usize = <u16 as MappedValue>::WIDTH;
    fn read(ram: &SpcRam, address: usize) -> Self {
        Self::at(<u16 as MappedValue>::read(ram, address))
    }
    fn write(self, ram: &mut SpcRam, address: usize) {
        self.address().write(ram, address);
    }
}

impl MappedValue for PatternCursor {
    const WIDTH: usize = <u16 as MappedValue>::WIDTH;
    fn read(ram: &SpcRam, address: usize) -> Self {
        Self::at(<u16 as MappedValue>::read(ram, address))
    }
    fn write(self, ram: &mut SpcRam, address: usize) {
        self.address().write(ram, address);
    }
}

impl MappedValue for PatternAddress {
    const WIDTH: usize = <u16 as MappedValue>::WIDTH;
    fn read(ram: &SpcRam, address: usize) -> Self {
        Self::at(<u16 as MappedValue>::read(ram, address))
    }
    fn write(self, ram: &mut SpcRam, address: usize) {
        self.address().write(ram, address);
    }
}

impl MappedValue for [u8; 4] {
    const WIDTH: usize = 4;
    fn read(ram: &SpcRam, address: usize) -> Self {
        [
            ram[address],
            ram[address + 1],
            ram[address + 2],
            ram[address + 3],
        ]
    }
    fn write(self, ram: &mut SpcRam, address: usize) {
        ram[address..address + Self::WIDTH].copy_from_slice(&self);
    }
}

#[cfg(test)]
pub(super) struct MappedField {
    pub name: &'static str,
    pub address: usize,
    pub width: usize,
}

macro_rules! channel_mapping {
    ($($field:ident: $value:ty => $address:literal;)*) => {
        impl Channel {
            fn write_mapped_ram(&self, ram: &mut SpcRam, channel_offset: usize) {
                $(<$value as MappedValue>::write(self.$field, ram, $address + channel_offset);)*
            }

            fn read_mapped_ram(&mut self, ram: &SpcRam, channel_offset: usize) {
                $(self.$field = <$value as MappedValue>::read(ram, $address + channel_offset);)*
            }
        }

        #[cfg(test)]
        pub(super) const CHANNEL_FIELDS: &[MappedField] = &[
            $(MappedField { name: stringify!($field), address: $address,
                width: <$value as MappedValue>::WIDTH },)*
        ];
    };
}

macro_rules! player_mapping {
    ($($field:ident: $value:ty => $address:literal;)*) => {
        impl SpcPlayer {
            /// Writes the native music variables into live SPC RAM for saving.
            pub fn copy_variables_to_ram(&mut self) {
                for (index, channel) in self.channels.iter().enumerate() {
                    channel.write_mapped_ram(&mut self.ram, index * 2);
                }
                $(<$value as MappedValue>::write(self.$field, &mut self.ram, $address);)*
            }

            /// Restores only the mapped native music variables from live RAM.
            pub fn copy_variables_from_ram(&mut self) {
                for (index, channel) in self.channels.iter_mut().enumerate() {
                    channel.read_mapped_ram(&self.ram, index * 2);
                }
                $(self.$field = <$value as MappedValue>::read(&self.ram, $address);)*
            }

            /// Returns the complete RAM image with native variables overlaid.
            ///
            /// Unlike the live save operation, observing this snapshot never
            /// changes SPC memory, echo data, DSP state, ports, or write history.
            #[must_use]
            pub fn mapped_ram_snapshot(&self) -> Box<[u8; 65536]> {
                let mut ram = self.ram.clone();
                for (index, channel) in self.channels.iter().enumerate() {
                    channel.write_mapped_ram(&mut ram, index * 2);
                }
                $(<$value as MappedValue>::write(self.$field, &mut ram, $address);)*
                ram
            }
        }

        #[cfg(test)]
        pub(super) const PLAYER_FIELDS: &[MappedField] = &[
            $(MappedField { name: stringify!($field), address: $address,
                width: <$value as MappedValue>::WIDTH },)*
        ];
    };
}

channel_mapping! {
    pattern_order_ptr_for_chan: PatternCursor => 0x30;
    note_ticks_left: u8 => 0x70;
    note_keyoff_ticks_left: u8 => 0x71;
    subroutine_num_loops: u8 => 0x80;
    volume_fade_ticks: u8 => 0x90;
    pan_num_ticks: u8 => 0x91;
    pitch_slide_length: u8 => 0xa0;
    pitch_slide_delay_left: u8 => 0xa1;
    vibrato_hold_count: u8 => 0xb0;
    vib_depth: u8 => 0xb1;
    tremolo_hold_count: u8 => 0xc0;
    tremolo_depth: u8 => 0xc1;
    vibrato_change_count: u8 => 0x100;
    note_length: u8 => 0x200;
    note_gate_off_fixedpt: u8 => 0x201;
    channel_volume_master: u8 => 0x210;
    instrument_id: u8 => 0x211;
    instrument_pitch_base: u16 => 0x220;
    saved_pattern_ptr: PatternAddress => 0x230;
    pattern_start_ptr: PatternAddress => 0x240;
    pitch_envelope_num_ticks: u8 => 0x280;
    pitch_envelope_delay: u8 => 0x281;
    pitch_envelope_direction: u8 => 0x290;
    pitch_envelope_slide_value: u8 => 0x291;
    vibrato_count: u8 => 0x2a0;
    vibrato_rate: u8 => 0x2a1;
    vibrato_delay_ticks: u8 => 0x2b0;
    vibrato_fade_num_ticks: u8 => 0x2b1;
    vibrato_fade_add_per_tick: u8 => 0x2c0;
    vibrato_depth_target: u8 => 0x2c1;
    tremolo_count: u8 => 0x2d0;
    tremolo_rate: u8 => 0x2d1;
    tremolo_delay_ticks: u8 => 0x2e0;
    channel_transposition: u8 => 0x2f0;
    channel_volume: u16 => 0x300;
    volume_fade_addpertick: u16 => 0x310;
    volume_fade_target: u8 => 0x320;
    final_volume: u8 => 0x321;
    pan_value: u16 => 0x330;
    pan_add_per_tick: u16 => 0x340;
    pan_target_value: u8 => 0x350;
    pan_flag_with_phase_invert: u8 => 0x351;
    pitch: u16 => 0x360;
    pitch_add_per_tick: u16 => 0x370;
    pitch_target: u8 => 0x380;
    fine_tune: u8 => 0x381;
    sfx_sound_ptr: u16 => 0x390;
    sfx_which_sound: u8 => 0x3a0;
    sfx_arr_countdown: u8 => 0x3a1;
    sfx_note_length_left: u8 => 0x3b0;
    sfx_note_length: u8 => 0x3b1;
    sfx_pan: u8 => 0x3d0;
}

player_mapping! {
    new_value_from_snes: [u8; 4] => 0x0;
    port_to_snes: [u8; 4] => 0x4;
    last_value_from_snes: [u8; 4] => 0x8;
    counter_sf0c: u8 => 0xc;
    always_zero: u16 => 0xe;
    temp_accum: u16 => 0x10;
    ttt: u8 => 0x12;
    did_affect_volumepitch_flag: u8 => 0x13;
    addr0: u16 => 0x14;
    addr1: u16 => 0x16;
    lfsr_value: u16 => 0x18;
    is_chan_on: u8 => 0x1a;
    fast_forward: u8 => 0x1b;
    sfx_start_arg_pan: u8 => 0x20;
    sfx_sound_ptr_cur: u16 => 0x2c;
    music_ptr_toplevel: PhraseCursor => 0x40;
    block_count: u8 => 0x42;
    sfx_timer_accum: u8 => 0x43;
    chn: u8 => 0x44;
    key_on: u8 => 0x45;
    key_off: u8 => 0x46;
    cur_chan_bit: u8 => 0x47;
    reg_flg: u8 => 0x48;
    reg_non: u8 => 0x49;
    reg_eon: u8 => 0x4a;
    reg_pmon: u8 => 0x4b;
    echo_stored_time: u8 => 0x4c;
    echo_parameter_edl: u8 => 0x4d;
    reg_efb: u8 => 0x4e;
    global_transposition: u8 => 0x50;
    main_tempo_accum: u8 => 0x51;
    tempo: u16 => 0x52;
    tempo_fade_num_ticks: u8 => 0x54;
    tempo_fade_final: u8 => 0x55;
    tempo_fade_add: u16 => 0x56;
    master_volume: u16 => 0x58;
    master_volume_fade_ticks: u8 => 0x5a;
    master_volume_fade_target: u8 => 0x5b;
    master_volume_fade_add_per_tick: u16 => 0x5c;
    vol_dirty: u8 => 0x5e;
    percussion_base_id: u8 => 0x5f;
    echo_volume_left: u16 => 0x60;
    echo_volume_right: u16 => 0x62;
    echo_volume_fade_add_left: u16 => 0x64;
    echo_volume_fade_add_right: u16 => 0x66;
    echo_volume_fade_ticks: u8 => 0x68;
    echo_volume_fade_target_left: u8 => 0x69;
    echo_volume_fade_target_right: u8 => 0x6a;
    sfx_channel_index: u8 => 0x3c0;
    current_bit: u8 => 0x3c1;
    dsp_register_index: u8 => 0x3c2;
    echo_channels: u8 => 0x3c3;
    byte_3c4: u8 => 0x3c4;
    byte_3c5: u8 => 0x3c5;
    echo_fract_incr: u8 => 0x3c7;
    sfx_channel_index2: u8 => 0x3c8;
    sfx_channel_bit: u8 => 0x3c9;
    pause_music_ctr: u8 => 0x3ca;
    port2_active: u8 => 0x3cb;
    port2_current_bit: u8 => 0x3cc;
    port3_active: u8 => 0x3cd;
    port3_current_bit: u8 => 0x3ce;
    port1_active: u8 => 0x3cf;
    port1_current_bit: u8 => 0x3e0;
    byte_3e1: u8 => 0x3e1;
    sfx_play_echo_flag: u8 => 0x3e2;
    sfx_channels_echo_mask2: u8 => 0x3e3;
    port1_counter: u8 => 0x3e4;
    channel_67_volume: u8 => 0x3e5;
    cutk_always_zero: u8 => 0x3ff;
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_snes::dsp::{DspRegWriteHistory, SAMPLE_FRAMES};

    fn mapped_bytes() -> [bool; 65536] {
        let mut mapped = [false; 65536];
        for offset in (0..8).map(|index| index * 2) {
            for field in CHANNEL_FIELDS {
                assert!(!field.name.is_empty());
                for byte in
                    &mut mapped[field.address + offset..field.address + offset + field.width]
                {
                    assert!(!*byte, "overlapping channel field {}", field.name);
                    *byte = true;
                }
            }
        }
        for field in PLAYER_FIELDS {
            assert!(!field.name.is_empty());
            for byte in &mut mapped[field.address..field.address + field.width] {
                assert!(!*byte, "overlapping player field {}", field.name);
                *byte = true;
            }
        }
        mapped
    }

    fn seeded_player() -> SpcPlayer {
        let mut player = SpcPlayer::new();
        player.initialize();
        for (index, byte) in player.ram.iter_mut().enumerate() {
            *byte = (index.wrapping_mul(37) ^ (index >> 8)) as u8;
        }
        player.tempo = 0x1234;
        player.key_on = 0x81;
        player.new_value_from_snes = [0x11, 0x22, 0x33, 0x44];
        player.port_to_snes = [0x55, 0x66, 0x77, 0x88];
        player.last_value_from_snes = [0x99, 0xaa, 0xbb, 0xcc];
        player.input_ports = [0xdd, 0xee, 0xff, 0x80];
        player.timer_cycles = 63;
        player.last_written_edl = 13;
        player.history = Some(DspRegWriteHistory::default());
        for (index, channel) in player.channels.iter_mut().enumerate() {
            channel.pattern_order_ptr_for_chan = PatternCursor::at(0xbe00 | index as u16);
            channel.note_ticks_left = 0x10 + index as u8;
            channel.note_keyoff_ticks_left = 0x90 + index as u8;
            channel.pitch = 0xa000 | index as u16;
            channel.sfx_pan = 0xf0 + index as u8;
            channel.index = 0xe0 + index as u8;
        }
        player
    }

    #[test]
    fn descriptors_cover_only_the_598_reviewed_bytes() {
        assert_eq!(CHANNEL_FIELDS.len(), 52);
        assert_eq!(PLAYER_FIELDS.len(), 70);
        let mapped = mapped_bytes();
        assert_eq!(mapped.iter().filter(|&&byte| byte).count(), 598);
        assert!(!mapped[0x0d]);
        assert!(!mapped[0x3c6]);
        assert!(mapped[0x3ff]);
        assert!(mapped[0x400..].iter().all(|&byte| !byte));
    }

    #[test]
    fn pattern_cursor_and_bookmark_codecs_preserve_every_raw_u16() {
        let mut ram = [0xa5; 65536];
        assert_eq!(<PatternCursor as MappedValue>::WIDTH, 2);
        assert_eq!(<PatternAddress as MappedValue>::WIDTH, 2);
        for value in 0..=u16::MAX {
            PatternCursor::at(value).write(&mut ram, 0x30);
            assert_eq!(&ram[0x30..0x32], &value.to_le_bytes());
            assert_eq!(
                <PatternCursor as MappedValue>::read(&ram, 0x30).address(),
                value
            );
            PatternAddress::at(value).write(&mut ram, 0x230);
            assert_eq!(&ram[0x230..0x232], &value.to_le_bytes());
            assert_eq!(
                <PatternAddress as MappedValue>::read(&ram, 0x230).address(),
                value
            );
        }
        for address in [0x2f, 0x32, 0x22f, 0x232] {
            assert_eq!(ram[address], 0xa5);
        }
    }

    #[test]
    fn every_phrase_position_round_trips_as_only_two_little_endian_bytes() {
        let mut ram = std::array::from_fn(|index| (index.wrapping_mul(37) ^ (index >> 8)) as u8);
        let before = ram;
        for address in 0..=u16::MAX {
            let cursor = PhraseCursor::at(address);
            cursor.write(&mut ram, 0x40);
            assert_eq!(&ram[0x40..0x42], &address.to_le_bytes());
            assert_eq!(PhraseCursor::read(&ram, 0x40), cursor);
            assert_eq!(&ram[..0x40], &before[..0x40]);
            assert_eq!(&ram[0x42..], &before[0x42..]);
        }
    }

    #[test]
    fn passive_snapshot_is_pure_and_matches_live_export_on_a_clone() {
        let player = seeded_player();
        let before = player.clone();
        let snapshot = player.mapped_ram_snapshot();
        assert_eq!(player, before);

        let mut saved = player.clone();
        saved.copy_variables_to_ram();
        assert_eq!(snapshot, saved.ram);
        assert_eq!(
            &snapshot[..12],
            &[
                0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc
            ]
        );
        assert_eq!(&snapshot[0x52..0x54], &[0x34, 0x12]);
        assert_eq!(snapshot[0x45], 0x81);
        for index in 0..8 {
            assert_eq!(
                &snapshot[0x30 + index * 2..0x32 + index * 2],
                &[index as u8, 0xbe]
            );
            assert_eq!(snapshot[0x70 + index * 2], 0x10 + index as u8);
            assert_eq!(snapshot[0x71 + index * 2], 0x90 + index as u8);
            assert_eq!(
                &snapshot[0x360 + index * 2..0x362 + index * 2],
                &[index as u8, 0xa0]
            );
            assert_eq!(snapshot[0x3d0 + index * 2], 0xf0 + index as u8);
        }
        for (address, mapped) in mapped_bytes().into_iter().enumerate() {
            if !mapped {
                assert_eq!(
                    snapshot[address], before.ram[address],
                    "unmapped RAM {address:#x}"
                );
            }
        }
    }

    #[test]
    fn importing_preserves_nonmapped_state_and_every_ram_byte() {
        let mut player = seeded_player();
        let before = player.clone();
        player.copy_variables_from_ram();
        assert_eq!(player.ram, before.ram);
        assert_eq!(player.history, before.history);
        assert_eq!(player.dsp, before.dsp);
        assert_eq!(player.timer_cycles, before.timer_cycles);
        assert_eq!(player.last_written_edl, before.last_written_edl);
        assert_eq!(player.input_ports, before.input_ports);
        for (channel, previous) in player.channels.iter().zip(&before.channels) {
            assert_eq!(channel.index, previous.index);
        }
        // Re-exporting decoded fields must reproduce both mapped data and holes.
        assert_eq!(player.mapped_ram_snapshot(), before.ram);
    }

    #[test]
    fn repeated_passive_observation_preserves_future_pcm_dsp_and_echo_state() {
        let mut observed = SpcPlayer::new();
        observed.initialize();
        observed.history = Some(DspRegWriteHistory::default());
        for (index, byte) in observed.ram[..0x400].iter_mut().enumerate() {
            *byte = (index as u8).wrapping_mul(73).wrapping_add(19);
        }
        // Exercise live RAM that differs from its native variable overlay,
        // including an echo buffer overlapping the mapped low-memory region.
        observed.reg_flg = 0;
        observed.dsp.write(&mut observed.ram, super::super::FLG, 0);
        observed.dsp.write(&mut observed.ram, super::super::ESA, 0);
        observed.dsp.write(&mut observed.ram, super::super::EDL, 1);
        let mut unobserved = observed.clone();
        for _ in 0..3 {
            for _ in 0..3 {
                let snapshot = observed.mapped_ram_snapshot();
                assert_ne!(snapshot, observed.ram);
            }
            assert_eq!(observed, unobserved);
            observed.generate_samples();
            unobserved.generate_samples();
            let mut actual = vec![0; SAMPLE_FRAMES * 2];
            let mut expected = vec![0; SAMPLE_FRAMES * 2];
            observed.dsp.get_samples(&mut actual, SAMPLE_FRAMES, 2);
            unobserved.dsp.get_samples(&mut expected, SAMPLE_FRAMES, 2);
            assert_eq!(actual, expected);
            assert_eq!(observed, unobserved);
        }
    }
}
