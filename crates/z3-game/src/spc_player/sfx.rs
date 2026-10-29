//! SFX command grammar, voice allocation and current-token publication.
//!
//! This module uses the same shared voice, pitch, instrument and DSP state as
//! music playback; it neither caches SPC RAM nor maintains a second voice copy.

use super::*;

impl SpcPlayer {
    pub(super) fn sfx_turn_off_channel(&mut self, ci: usize) {
        self.channels[ci].sfx_which_sound = 0;
        self.is_chan_on &= !self.current_bit;
        self.port1_active &= !self.current_bit;
        self.port2_active &= !self.current_bit;
        self.port3_active &= !self.current_bit;
        let instrument = self.channels[ci].instrument_id;
        self.channel_set_instrument(ci, instrument);
        if self.echo_channels & self.current_bit != 0 && self.reg_eon & self.current_bit == 0 {
            self.reg_eon |= self.current_bit;
            self.dsp_write(EON, self.reg_eon);
            self.sfx_channels_echo_mask2 &= !self.current_bit;
        }
    }

    pub(super) fn write_key_on(&mut self, bit: u8) {
        self.dsp_write(KOF, 0);
        self.dsp_write(KON, bit);
    }

    pub(super) fn sfx_maybe_disable_echo(&mut self) {
        if (self.port_to_snes[0] & 0x10 == 0
            || self.current_bit & self.sfx_channels_echo_mask2 != 0)
            && self.current_bit & self.reg_eon != 0
        {
            self.reg_eon ^= self.current_bit;
            self.dsp_write(EON, self.reg_eon);
        }
    }

    pub(super) fn sfx_channel_tick(&mut self, ci: usize, is_continue: bool) {
        if is_continue {
            self.sfx_maybe_disable_echo();
            self.sfx_channel_index = self.channels[ci].index * 2;
            self.sfx_sound_ptr_cur
                .jump_to(self.channels[ci].sfx_sound_ptr);
            self.channels[ci].sfx_note_length_left =
                self.channels[ci].sfx_note_length_left.wrapping_sub(1);
            if self.channels[ci].sfx_note_length_left != 0 {
                self.sfx_note_continue(ci);
                self.channels[ci].sfx_sound_ptr = self.sfx_sound_ptr_cur.position();
                return;
            }
            self.sfx_sound_ptr_cur.advance();
        }
        loop {
            self.dsp_register_index = self.sfx_channel_index * 8;
            let mut cmd = self.sfx_sound_ptr_cur.current(&self.ram);
            if cmd == 0 {
                self.sfx_turn_off_channel(ci);
                return;
            }
            if cmd & 0x80 == 0 {
                self.channels[ci].sfx_note_length = cmd;
                cmd = self.sfx_sound_ptr_cur.advance_read(&self.ram);
                if cmd & 0x80 == 0 {
                    if self.port1_active & self.current_bit != 0 {
                        if cmd == 0 || self.channel_67_volume == 0 {
                            let volume = cmd;
                            self.dsp_write(self.dsp_register_index + V0VOLL, cmd);
                            cmd = self.sfx_sound_ptr_cur.advance_read(&self.ram);
                            if cmd & 0x80 != 0 {
                                self.dsp_write(self.dsp_register_index + V0VOLR, volume);
                            } else {
                                self.dsp_write(self.dsp_register_index + V0VOLR, cmd);
                                cmd = self.sfx_sound_ptr_cur.advance_read(&self.ram);
                            }
                        } else {
                            cmd = self.sfx_sound_ptr_cur.advance_read(&self.ram);
                        }
                    } else {
                        self.channels[ci].final_volume = cmd.wrapping_mul(2);
                        self.channels[ci].pan_flag_with_phase_invert = 10;
                        let pan = if self.sfx_start_arg_pan & 0x80 != 0 {
                            16
                        } else if self.sfx_start_arg_pan & 0x40 != 0 {
                            4
                        } else {
                            10
                        };
                        self.write_volume_to_dsp(ci, pan << 8);
                        cmd = self.sfx_sound_ptr_cur.advance_read(&self.ram);
                    }
                }
            }
            if cmd == 0xe0 {
                let preset = usize::from(self.sfx_sound_ptr_cur.advance_read(&self.ram));
                let address = 0x3e00 + preset * 9;
                let params = <[u8; 9]>::try_from(&self.ram[address..address + 9]).unwrap();
                let reg = self.channels[ci].index * 16;
                self.dsp_write(reg + V0VOLL, params[0]);
                self.dsp_write(reg + V0VOLR, params[1]);
                self.dsp_write(reg + V0PITCHL, params[2]);
                self.dsp_write(reg + V0PITCHH, params[3]);
                self.dsp_write(reg + V0SRCN, params[4]);
                self.dsp_write(reg + V0ADSR1, params[5]);
                self.dsp_write(reg + V0ADSR2, params[6]);
                self.dsp_write(reg + V0GAIN, params[7]);
                self.channels[ci].instrument_pitch_base = u16::from(params[8]) << 8;
                self.sfx_sound_ptr_cur.advance();
            } else if cmd == 0xf9 || cmd == 0xf1 {
                if cmd == 0xf9 {
                    let note = self.sfx_sound_ptr_cur.advance_read(&self.ram);
                    self.play_note(ci, note);
                    self.write_key_on(self.current_bit);
                }
                self.sfx_sound_ptr_cur.advance();
                self.channels[ci].pitch_slide_delay_left =
                    self.sfx_sound_ptr_cur.current(&self.ram);
                self.sfx_sound_ptr_cur.advance();
                self.channels[ci].pitch_slide_length = self.sfx_sound_ptr_cur.current(&self.ram);
                self.sfx_sound_ptr_cur.advance();
                let target = self.sfx_sound_ptr_cur.current(&self.ram);
                self.compute_pitch_add(ci, target);
                self.channels[ci].sfx_note_length_left = self.channels[ci].sfx_note_length;
                self.sfx_note_continue(ci);
                break;
            } else if cmd == 0xff {
                let table = 0x17beu16 + u16::from(self.channels[ci].sfx_which_sound) * 2;
                let pointer = SfxPosition::at(read16(&self.ram, table));
                self.sfx_sound_ptr_cur.jump_to(pointer);
                self.channels[ci].sfx_sound_ptr = pointer;
            } else {
                self.play_note(ci, cmd);
                self.write_key_on(self.current_bit);
                self.channels[ci].sfx_note_length_left = self.channels[ci].sfx_note_length;
                self.sfx_note_continue(ci);
                break;
            }
        }
        self.channels[ci].sfx_sound_ptr = self.sfx_sound_ptr_cur.position();
    }

    pub(super) fn sfx_note_continue(&mut self, ci: usize) {
        self.did_affect_volumepitch_flag = 0;
        if self.channels[ci].pitch_slide_length != 0 {
            self.did_affect_volumepitch_flag = 0x80;
            let c = &mut self.channels[ci];
            c.pitch_slide_length = c.pitch_slide_length.wrapping_sub(1);
            fade(
                &mut c.pitch,
                c.pitch_add_per_tick,
                c.pitch_target,
                c.pitch_slide_length != 0,
            );
            self.cur_chan_bit = 0;
            self.write_pitch(ci, self.channels[ci].pitch);
        } else if self.channels[ci].sfx_note_length_left == 2 {
            self.dsp_write(KOF, self.current_bit);
        }
    }

    pub(super) fn port1_play_inner(&mut self) {
        self.port1_counter = 0;
        let sound = self.new_value_from_snes[1];
        self.channels[7].sfx_which_sound = sound;
        self.channels[7].sfx_arr_countdown = 3;
        self.channels[7].pitch_envelope_num_ticks = 0;
        self.port1_active = 0x80;
        self.is_chan_on |= 0x80;
        self.dsp_write(KOF, 0x80);
        self.new_value_from_snes[1] = self.ram[0x17ff + usize::from(sound)];
        if self.new_value_from_snes[1] == 0 {
            return;
        }
        let sound = self.new_value_from_snes[1];
        self.channels[6].sfx_which_sound = sound;
        self.channels[6].sfx_arr_countdown = 3;
        self.channels[6].pitch_envelope_num_ticks = 0;
        self.port1_active = 0x40;
        self.is_chan_on |= 0x40;
        self.dsp_write(KOF, 0x40);
        self.port1_active = 0xc0;
        self.sfx_channels_echo_mask2 |= 0xc0;
        self.port2_active &= 0x3f;
        self.port3_active &= 0x3f;
    }

    pub(super) fn port1_start_new_sound(&mut self) {
        if self.port1_counter != 0 {
            self.port1_counter = self.port1_counter.wrapping_sub(1);
            if self.port1_counter == 0 {
                self.new_value_from_snes[1] = 5;
                self.port1_play_inner();
                self.new_value_from_snes[1] = 0;
                return;
            }
            self.channel_67_volume = self.port1_counter >> 1;
            for reg in [0x70, 0x71, 0x60, 0x61] {
                self.dsp_write(reg, self.channel_67_volume);
            }
        }
        self.port1_current_bit = self.port1_active;
        if self.port1_current_bit == 0 {
            return;
        }
        self.current_bit = 0x80;
        for ci in (5..=7).rev() {
            let carry = self.port1_current_bit >> 7;
            self.port1_current_bit <<= 1;
            if carry != 0 {
                self.sfx_channel_index = self.channels[ci].index * 2;
                self.dsp_register_index = self.channels[ci].index * 16;
                self.sfx_start_arg_pan = self.channels[ci].sfx_pan;
                if self.channels[ci].sfx_arr_countdown == 0 {
                    if self.channels[ci].sfx_which_sound != 0 {
                        self.sfx_channel_tick(ci, true);
                    }
                } else {
                    self.channels[ci].sfx_arr_countdown =
                        self.channels[ci].sfx_arr_countdown.wrapping_sub(1);
                    if self.channels[ci].sfx_arr_countdown == 0 {
                        let address = 0x17beu16 + u16::from(self.channels[ci].sfx_which_sound) * 2;
                        let pointer = SfxPosition::at(read16(&self.ram, address));
                        self.sfx_sound_ptr_cur.jump_to(pointer);
                        self.channels[ci].sfx_sound_ptr = pointer;
                        self.sfx_channel_tick(ci, false);
                    }
                }
            }
            self.current_bit >>= 1;
        }
    }

    pub(super) fn port1_handle_cmd(&mut self) {
        let command = self.new_value_from_snes[1];
        if command & 0x80 == 0 {
            if command != 0 {
                self.port_to_snes[1] = command;
                if command != 5 || self.port1_active != 0 {
                    self.port1_play_inner();
                }
            }
        } else {
            self.port_to_snes[1] = command;
            if self.port1_active != 0 {
                self.port1_counter = 0x78;
            }
        }
    }

    pub(super) fn port_start_new_sound(&mut self, port: usize) {
        let active = if port == 2 {
            self.port2_active
        } else {
            self.port3_active
        };
        if port == 2 {
            self.port2_current_bit = active;
        } else {
            self.port3_current_bit = active;
        }
        if active == 0 {
            return;
        }
        let mut shifted = active;
        self.current_bit = 0x80;
        for ci in (0..8).rev() {
            let carry = shifted >> 7;
            shifted <<= 1;
            if carry != 0 {
                self.sfx_channel_index = self.channels[ci].index * 2;
                self.dsp_register_index = self.channels[ci].index * 16;
                self.sfx_start_arg_pan = self.channels[ci].sfx_pan;
                if self.channels[ci].sfx_arr_countdown == 0 {
                    if self.channels[ci].sfx_which_sound != 0 {
                        self.sfx_channel_tick(ci, true);
                    }
                } else {
                    self.channels[ci].sfx_arr_countdown =
                        self.channels[ci].sfx_arr_countdown.wrapping_sub(1);
                    if self.channels[ci].sfx_arr_countdown == 0 {
                        let base = if port == 2 { 0x181e } else { 0x191a };
                        let address = base + usize::from(self.channels[ci].sfx_which_sound) * 2;
                        let pointer = SfxPosition::at(read16(&self.ram, address as u16));
                        self.sfx_sound_ptr_cur.jump_to(pointer);
                        self.channels[ci].sfx_sound_ptr = pointer;
                        self.sfx_channel_tick(ci, false);
                    }
                }
            }
            self.current_bit >>= 1;
        }
        if port == 2 {
            self.port2_current_bit = shifted;
        } else {
            self.port3_current_bit = shifted;
        }
    }

    pub(super) fn allocate_sfx_channel(&mut self, port: usize) -> usize {
        let command = self.new_value_from_snes[port];
        let flag_address = if port == 2 {
            0x18dc + usize::from(command & 0x3f)
        } else {
            0x19d8 + usize::from(command & 0x3f)
        };
        self.sfx_play_echo_flag = self.ram[flag_address];
        let active = if port == 2 {
            self.port2_active
        } else {
            self.port3_active
        };
        self.current_bit = 0x80;
        let mut selected = None;
        for ci in (0..8).rev() {
            if active & self.current_bit != 0
                && self.channels[ci]
                    .sfx_which_sound
                    .wrapping_add(self.channels[ci].sfx_pan)
                    == command
            {
                selected = Some(ci);
                break;
            }
            self.current_bit >>= 1;
        }
        if selected.is_none() {
            self.current_bit = 0x80;
            for ci in (0..8).rev() {
                if self.is_chan_on & self.current_bit == 0 {
                    selected = Some(ci);
                    break;
                }
                self.current_bit >>= 1;
            }
        }
        let ci = selected.expect("C SPC player expected a free SFX channel");
        self.sfx_channel_index = self.channels[ci].index * 2;
        self.sfx_channel_index2 = self.sfx_channel_index;
        self.sfx_channel_bit = self.current_bit;
        self.is_chan_on |= self.current_bit;
        if self.sfx_play_echo_flag != 0 {
            self.sfx_channels_echo_mask2 |= self.current_bit;
        }
        self.sfx_maybe_disable_echo();
        ci
    }

    pub(super) fn port_handle_cmd(&mut self, port: usize) {
        while self.new_value_from_snes[port] != 0 && self.is_chan_on != 0xff {
            let ci = self.allocate_sfx_channel(port);
            let command = self.new_value_from_snes[port];
            self.channels[ci].sfx_pan = command & 0xc0;
            self.channels[ci].sfx_which_sound = command & 0x3f;
            self.channels[ci].sfx_arr_countdown = 3;
            self.channels[ci].pitch_envelope_num_ticks = 0;
            if port == 2 {
                self.port2_active |= self.current_bit;
            } else {
                self.port3_active |= self.current_bit;
            }
            self.dsp_write(KOF, self.current_bit);
            let base = if port == 2 { 0x189d } else { 0x1999 };
            self.new_value_from_snes[port] =
                self.ram[base + usize::from(self.channels[ci].sfx_which_sound)];
        }
    }
}
