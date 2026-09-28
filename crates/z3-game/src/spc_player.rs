//! Native music player port from `src/spc_player.c`.

use z3_snes::dsp::{Dsp, DspRegWriteHistory, SAMPLE_FRAMES};

const V0VOLL: u8 = 0x00;
const V0VOLR: u8 = 0x01;
const V0PITCHL: u8 = 0x02;
const V0PITCHH: u8 = 0x03;
const V0SRCN: u8 = 0x04;
const V0ADSR1: u8 = 0x05;
const V0ADSR2: u8 = 0x06;
const V0GAIN: u8 = 0x07;
const MVOLL: u8 = 0x0c;
const EFB: u8 = 0x0d;
const FIR0: u8 = 0x0f;
const MVOLR: u8 = 0x1c;
const EVOLL: u8 = 0x2c;
const PMON: u8 = 0x2d;
const EVOLR: u8 = 0x3c;
const NON: u8 = 0x3d;
const KON: u8 = 0x4c;
const EON: u8 = 0x4d;
const KOF: u8 = 0x5c;
const DIR: u8 = 0x5d;
const FLG: u8 = 0x6c;
const ESA: u8 = 0x6d;
const EDL: u8 = 0x7d;

const EFFECT_BYTE_LENGTH: [u8; 27] = [
    1, 1, 2, 3, 0, 1, 2, 1, 2, 1, 1, 3, 0, 1, 2, 3, 1, 3, 3, 0, 1, 3, 0, 3, 3, 3, 1,
];

fn read16(ram: &[u8; 65536], address: u16) -> u16 {
    u16::from_le_bytes([ram[address as usize], ram[address.wrapping_add(1) as usize]])
}

fn spc_div_helper(a: i32, b: u8) -> u16 {
    let negative = a & 0x100 != 0;
    let a = if negative { -a } else { a };
    let (q, r) = if b == 0 {
        (0xff, a & 0xff)
    } else {
        ((a & 0xff) / i32::from(b), (a & 0xff) % i32::from(b))
    };
    let result = (q << 8)
        + if b == 0 {
            0xff
        } else {
            ((r << 8) / i32::from(b)) & 0xff
        };
    if negative {
        (0i32.wrapping_sub(result)) as u16
    } else {
        result as u16
    }
}

fn fade(value: &mut u16, add: u16, target: u8, continue_fade: bool) {
    *value = if continue_fade {
        value.wrapping_add(add)
    } else {
        u16::from(target) << 8
    };
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub pattern_order_ptr_for_chan: u16,
    pub note_ticks_left: u8,
    pub note_keyoff_ticks_left: u8,
    pub subroutine_num_loops: u8,
    pub volume_fade_ticks: u8,
    pub pan_num_ticks: u8,
    pub pitch_slide_length: u8,
    pub pitch_slide_delay_left: u8,
    pub vibrato_hold_count: u8,
    pub vib_depth: u8,
    pub tremolo_hold_count: u8,
    pub tremolo_depth: u8,
    pub vibrato_change_count: u8,
    pub note_length: u8,
    pub note_gate_off_fixedpt: u8,
    pub channel_volume_master: u8,
    pub instrument_id: u8,
    pub instrument_pitch_base: u16,
    pub saved_pattern_ptr: u16,
    pub pattern_start_ptr: u16,
    pub pitch_envelope_num_ticks: u8,
    pub pitch_envelope_delay: u8,
    pub pitch_envelope_direction: u8,
    pub pitch_envelope_slide_value: u8,
    pub vibrato_count: u8,
    pub vibrato_rate: u8,
    pub vibrato_delay_ticks: u8,
    pub vibrato_fade_num_ticks: u8,
    pub vibrato_fade_add_per_tick: u8,
    pub vibrato_depth_target: u8,
    pub tremolo_count: u8,
    pub tremolo_rate: u8,
    pub tremolo_delay_ticks: u8,
    pub channel_transposition: u8,
    pub channel_volume: u16,
    pub volume_fade_addpertick: u16,
    pub volume_fade_target: u8,
    pub final_volume: u8,
    pub pan_value: u16,
    pub pan_add_per_tick: u16,
    pub pan_target_value: u8,
    pub pan_flag_with_phase_invert: u8,
    pub pitch: u16,
    pub pitch_add_per_tick: u16,
    pub pitch_target: u8,
    pub fine_tune: u8,
    pub sfx_sound_ptr: u16,
    pub sfx_which_sound: u8,
    pub sfx_arr_countdown: u8,
    pub sfx_note_length_left: u8,
    pub sfx_note_length: u8,
    pub sfx_pan: u8,
    pub index: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpcPlayer {
    pub history: Option<DspRegWriteHistory>,
    pub timer_cycles: u8,
    pub dsp: Dsp,
    pub new_value_from_snes: [u8; 4],
    pub port_to_snes: [u8; 4],
    pub last_value_from_snes: [u8; 4],
    pub counter_sf0c: u8,
    pub always_zero: u16,
    pub temp_accum: u16,
    pub ttt: u8,
    pub did_affect_volumepitch_flag: u8,
    pub addr0: u16,
    pub addr1: u16,
    pub lfsr_value: u16,
    pub is_chan_on: u8,
    pub fast_forward: u8,
    pub sfx_start_arg_pan: u8,
    pub sfx_sound_ptr_cur: u16,
    pub music_ptr_toplevel: u16,
    pub block_count: u8,
    pub sfx_timer_accum: u8,
    pub chn: u8,
    pub key_on: u8,
    pub key_off: u8,
    pub cur_chan_bit: u8,
    pub reg_flg: u8,
    pub reg_non: u8,
    pub reg_eon: u8,
    pub reg_pmon: u8,
    pub echo_stored_time: u8,
    pub echo_parameter_edl: u8,
    pub reg_efb: u8,
    pub global_transposition: u8,
    pub main_tempo_accum: u8,
    pub tempo: u16,
    pub tempo_fade_num_ticks: u8,
    pub tempo_fade_final: u8,
    pub tempo_fade_add: u16,
    pub master_volume: u16,
    pub master_volume_fade_ticks: u8,
    pub master_volume_fade_target: u8,
    pub master_volume_fade_add_per_tick: u16,
    pub vol_dirty: u8,
    pub percussion_base_id: u8,
    pub echo_volume_left: u16,
    pub echo_volume_right: u16,
    pub echo_volume_fade_add_left: u16,
    pub echo_volume_fade_add_right: u16,
    pub echo_volume_fade_ticks: u8,
    pub echo_volume_fade_target_left: u8,
    pub echo_volume_fade_target_right: u8,
    pub sfx_channel_index: u8,
    pub current_bit: u8,
    pub dsp_register_index: u8,
    pub echo_channels: u8,
    pub byte_3c4: u8,
    pub byte_3c5: u8,
    pub echo_fract_incr: u8,
    pub sfx_channel_index2: u8,
    pub sfx_channel_bit: u8,
    pub pause_music_ctr: u8,
    pub port2_active: u8,
    pub port2_current_bit: u8,
    pub port3_active: u8,
    pub port3_current_bit: u8,
    pub port1_active: u8,
    pub port1_current_bit: u8,
    pub byte_3e1: u8,
    pub sfx_play_echo_flag: u8,
    pub sfx_channels_echo_mask2: u8,
    pub port1_counter: u8,
    pub channel_67_volume: u8,
    pub cutk_always_zero: u8,
    pub last_written_edl: u8,
    pub input_ports: [u8; 4],
    pub channels: [Channel; 8],
    pub ram: Box<[u8; 65536]>,
}

impl Default for SpcPlayer {
    fn default() -> Self {
        Self {
            history: None,
            timer_cycles: 0,
            dsp: Dsp::new(),
            new_value_from_snes: [0; 4],
            port_to_snes: [0; 4],
            last_value_from_snes: [0; 4],
            counter_sf0c: 0,
            always_zero: 0,
            temp_accum: 0,
            ttt: 0,
            did_affect_volumepitch_flag: 0,
            addr0: 0,
            addr1: 0,
            lfsr_value: 0,
            is_chan_on: 0,
            fast_forward: 0,
            sfx_start_arg_pan: 0,
            sfx_sound_ptr_cur: 0,
            music_ptr_toplevel: 0,
            block_count: 0,
            sfx_timer_accum: 0,
            chn: 0,
            key_on: 0,
            key_off: 0,
            cur_chan_bit: 0,
            reg_flg: 0,
            reg_non: 0,
            reg_eon: 0,
            reg_pmon: 0,
            echo_stored_time: 0,
            echo_parameter_edl: 0,
            reg_efb: 0,
            global_transposition: 0,
            main_tempo_accum: 0,
            tempo: 0,
            tempo_fade_num_ticks: 0,
            tempo_fade_final: 0,
            tempo_fade_add: 0,
            master_volume: 0,
            master_volume_fade_ticks: 0,
            master_volume_fade_target: 0,
            master_volume_fade_add_per_tick: 0,
            vol_dirty: 0,
            percussion_base_id: 0,
            echo_volume_left: 0,
            echo_volume_right: 0,
            echo_volume_fade_add_left: 0,
            echo_volume_fade_add_right: 0,
            echo_volume_fade_ticks: 0,
            echo_volume_fade_target_left: 0,
            echo_volume_fade_target_right: 0,
            sfx_channel_index: 0,
            current_bit: 0,
            dsp_register_index: 0,
            echo_channels: 0,
            byte_3c4: 0,
            byte_3c5: 0,
            echo_fract_incr: 0,
            sfx_channel_index2: 0,
            sfx_channel_bit: 0,
            pause_music_ctr: 0,
            port2_active: 0,
            port2_current_bit: 0,
            port3_active: 0,
            port3_current_bit: 0,
            port1_active: 0,
            port1_current_bit: 0,
            byte_3e1: 0,
            sfx_play_echo_flag: 0,
            sfx_channels_echo_mask2: 0,
            port1_counter: 0,
            channel_67_volume: 0,
            cutk_always_zero: 0,
            last_written_edl: 0,
            input_ports: [0; 4],
            channels: core::array::from_fn(|i| Channel {
                index: i as u8,
                ..Channel::default()
            }),
            ram: Box::new([0; 65536]),
        }
    }
}

impl SpcPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    fn dsp_write(&mut self, r: u8, v: u8) {
        if let Some(h) = &mut self.history
            && h.count < 256
        {
            h.addr[h.count as usize] = r;
            h.val[h.count as usize] = v;
            h.count += 1
        }
        self.dsp.write(&mut self.ram, r, v)
    }

    fn setup_echo_parameter_edl(&mut self, mut value: u8) {
        self.echo_parameter_edl = value;
        if value != self.last_written_edl {
            value = (self.last_written_edl & 0x0f) ^ 0xff;
            if self.echo_stored_time & 0x80 != 0 {
                value = value.wrapping_add(self.echo_stored_time);
            }
            self.echo_stored_time = value;
            self.dsp_write(EON, 0);
            self.dsp_write(EFB, 0);
            self.dsp_write(EVOLR, 0);
            self.dsp_write(EVOLL, 0);
            self.dsp_write(FLG, self.reg_flg | 0x20);
            self.last_written_edl = self.echo_parameter_edl;
            self.dsp_write(EDL, self.echo_parameter_edl);
        }
        self.dsp_write(
            ESA,
            (self.echo_parameter_edl.wrapping_mul(8) ^ 0xff).wrapping_add(0xd1),
        );
    }

    fn write_volume_to_dsp(&mut self, ci: usize, mut volume: u16) {
        const VOLUME_TABLE: [u8; 22] = [
            0, 1, 3, 7, 13, 21, 30, 41, 52, 66, 81, 94, 103, 110, 115, 119, 122, 124, 125, 126,
            127, 127,
        ];
        if self.is_chan_on & self.cur_chan_bit != 0 {
            return;
        }
        let final_volume = self.channels[ci].final_volume;
        let phase = self.channels[ci].pan_flag_with_phase_invert;
        for side in 0..2u8 {
            let j = usize::from(volume >> 8);
            let lo = if j >= 21 {
                self.ram[j + 0x1178]
            } else {
                VOLUME_TABLE[j]
            };
            let hi = if j >= 21 {
                self.ram[j + 0x1179]
            } else {
                VOLUME_TABLE[j + 1]
            };
            let interpolation = i16::from(hi) - i16::from(lo);
            let mut result =
                u8::try_from(i16::from(lo) + ((interpolation * i16::from(volume as u8)) >> 8))
                    .unwrap_or_else(|_| {
                        (i16::from(lo) + ((interpolation * i16::from(volume as u8)) >> 8)) as u8
                    });
            result = ((u16::from(result) * u16::from(final_volume)) >> 8) as u8;
            if (phase.wrapping_shl(u32::from(side))) & 0x80 != 0 {
                result = result.wrapping_neg();
            }
            self.dsp_write(V0VOLL + side + self.channels[ci].index * 16, result);
            volume = 0x1400u16.wrapping_sub(volume);
        }
    }

    fn write_pitch(&mut self, ci: usize, mut pitch: u16) {
        const BASE_NOTE_FREQS: [u16; 13] = [
            2143, 2270, 2405, 2548, 2700, 2860, 3030, 3211, 3402, 3604, 3818, 4045, 4286,
        ];
        let high = (pitch >> 8) as u8;
        if high >= 0x34 {
            pitch = pitch.wrapping_add(u16::from(high - 0x34));
        } else if high < 0x13 {
            let correction = high.wrapping_sub(0x13).wrapping_mul(2);
            pitch = pitch.wrapping_add(u16::from(correction).wrapping_sub(256));
        }
        let pp = ((pitch >> 8) as u8) & 0x7f;
        let mut octave = pp / 12;
        let note = usize::from(pp % 12);
        let delta = BASE_NOTE_FREQS[note + 1] - BASE_NOTE_FREQS[note];
        let mut result = BASE_NOTE_FREQS[note]
            .wrapping_add((u16::from(delta as u8) * u16::from(pitch as u8)) >> 8)
            .wrapping_mul(2);
        while octave != 6 {
            result >>= 1;
            octave = octave.wrapping_add(1);
        }
        result =
            ((u32::from(self.channels[ci].instrument_pitch_base) * u32::from(result)) >> 8) as u16;
        if self.cur_chan_bit & self.is_chan_on == 0 {
            let reg = self.channels[ci].index * 16;
            self.dsp_write(reg + V0PITCHL, result as u8);
            self.dsp_write(reg + V0PITCHH, (result >> 8) as u8);
        }
    }

    fn music_reset_chan(&mut self) {
        self.cur_chan_bit = 0x80;
        for ci in (0..8).rev() {
            let c = &mut self.channels[ci];
            c.channel_volume = (c.channel_volume & 0xff) | 0xff00;
            c.pan_flag_with_phase_invert = 10;
            c.pan_value = 10 << 8;
            c.instrument_id = 0;
            c.fine_tune = 0;
            c.channel_transposition = 0;
            c.pitch_envelope_num_ticks = 0;
            c.vib_depth = 0;
            c.tremolo_depth = 0;
            self.cur_chan_bit >>= 1;
        }
        self.master_volume_fade_ticks = 0;
        self.echo_volume_fade_ticks = 0;
        self.tempo_fade_num_ticks = 0;
        self.global_transposition = 0;
        self.block_count = 0;
        self.percussion_base_id = 0;
        self.master_volume = (self.master_volume & 0xff) | 0xc000;
        self.tempo = (self.tempo & 0xff) | 0x2000;
    }

    fn channel_set_instrument(&mut self, ci: usize, mut instrument: u8) {
        self.channels[ci].instrument_id = instrument;
        if instrument & 0x80 != 0 {
            instrument = instrument
                .wrapping_add(54)
                .wrapping_add(self.percussion_base_id);
        }
        let address = usize::from(instrument) * 6 + 0x3d00;
        let params = <[u8; 6]>::try_from(&self.ram[address..address + 6]).unwrap();
        if self.is_chan_on & self.cur_chan_bit != 0 {
            return;
        }
        let reg = self.channels[ci].index * 16;
        if params[0] & 0x80 != 0 {
            self.reg_flg = (self.reg_flg & 0x20) | (params[0] & 0x1f);
            self.reg_non |= self.cur_chan_bit;
            self.dsp_write(reg + V0SRCN, 0);
        } else {
            self.dsp_write(reg + V0SRCN, params[0]);
        }
        self.dsp_write(reg + V0ADSR1, params[1]);
        self.dsp_write(reg + V0ADSR2, params[2]);
        self.dsp_write(reg + V0GAIN, params[3]);
        self.channels[ci].instrument_pitch_base = u16::from_be_bytes([params[4], params[5]]);
    }

    fn compute_pitch_add(&mut self, ci: usize, pitch: u8) {
        let c = &mut self.channels[ci];
        c.pitch_target = pitch & 0x7f;
        c.pitch_add_per_tick = spc_div_helper(
            i32::from(c.pitch_target) - i32::from((c.pitch >> 8) as u8),
            c.pitch_slide_length,
        );
    }

    fn pitch_slide_to_note_check(&mut self, ci: usize) {
        let ptr = self.channels[ci].pattern_order_ptr_for_chan;
        if self.channels[ci].pitch_slide_length != 0 || self.ram[ptr as usize] != 0xf9 {
            return;
        }
        if self.cur_chan_bit & self.is_chan_on != 0 {
            self.channels[ci].pattern_order_ptr_for_chan = ptr.wrapping_add(4);
            return;
        }
        let delay = self.ram[ptr.wrapping_add(1) as usize];
        let length = self.ram[ptr.wrapping_add(2) as usize];
        let note = self.ram[ptr.wrapping_add(3) as usize]
            .wrapping_add(self.global_transposition)
            .wrapping_add(self.channels[ci].channel_transposition);
        self.channels[ci].pattern_order_ptr_for_chan = ptr.wrapping_add(4);
        self.channels[ci].pitch_slide_delay_left = delay;
        self.channels[ci].pitch_slide_length = length;
        self.compute_pitch_add(ci, note);
    }

    fn next_pattern_byte(&mut self, ci: usize) -> u8 {
        let address = self.channels[ci].pattern_order_ptr_for_chan;
        self.channels[ci].pattern_order_ptr_for_chan = address.wrapping_add(1);
        self.ram[address as usize]
    }

    fn handle_effect(&mut self, ci: usize, effect: u8) {
        let arg = if EFFECT_BYTE_LENGTH[usize::from(effect - 0xe0)] != 0 {
            self.next_pattern_byte(ci)
        } else {
            0
        };
        match effect {
            0xe0 => self.channel_set_instrument(ci, arg),
            0xe1 => {
                self.channels[ci].pan_flag_with_phase_invert = arg;
                self.channels[ci].pan_value = u16::from(arg & 0x1f) << 8;
            }
            0xe2 => {
                let target = self.next_pattern_byte(ci);
                let c = &mut self.channels[ci];
                c.pan_num_ticks = arg;
                c.pan_target_value = target;
                c.pan_add_per_tick =
                    spc_div_helper(i32::from(target) - i32::from((c.pan_value >> 8) as u8), arg);
            }
            0xe3 => {
                let rate = self.next_pattern_byte(ci);
                let depth = self.next_pattern_byte(ci);
                let c = &mut self.channels[ci];
                c.vibrato_delay_ticks = arg;
                c.vibrato_rate = rate;
                c.vibrato_depth_target = depth;
                c.vib_depth = depth;
                c.vibrato_fade_num_ticks = 0;
            }
            0xe4 => {
                self.channels[ci].vibrato_depth_target = 0;
                self.channels[ci].vib_depth = 0;
                self.channels[ci].vibrato_fade_num_ticks = 0;
            }
            0xe5 => {
                if self.pause_music_ctr == 0 && self.byte_3e1 == 0 {
                    self.master_volume = u16::from(arg) << 8;
                }
            }
            0xe6 => {
                let target = self.next_pattern_byte(ci);
                self.master_volume_fade_ticks = arg;
                self.master_volume_fade_target = target;
                self.master_volume_fade_add_per_tick = spc_div_helper(
                    i32::from(target) - i32::from((self.master_volume >> 8) as u8),
                    arg,
                );
            }
            0xe7 => self.tempo = u16::from(arg) << 8,
            0xe8 => {
                let target = self.next_pattern_byte(ci);
                self.tempo_fade_num_ticks = arg;
                self.tempo_fade_final = target;
                self.tempo_fade_add =
                    spc_div_helper(i32::from(target) - i32::from((self.tempo >> 8) as u8), arg);
            }
            0xe9 => self.global_transposition = arg,
            0xea => self.channels[ci].channel_transposition = arg,
            0xeb => {
                let rate = self.next_pattern_byte(ci);
                let depth = self.next_pattern_byte(ci);
                self.channels[ci].tremolo_delay_ticks = arg;
                self.channels[ci].tremolo_rate = rate;
                self.channels[ci].tremolo_depth = depth;
            }
            0xec => self.channels[ci].tremolo_depth = 0,
            0xed => self.channels[ci].channel_volume = u16::from(arg) << 8,
            0xee => {
                let target = self.next_pattern_byte(ci);
                let c = &mut self.channels[ci];
                c.volume_fade_ticks = arg;
                c.volume_fade_target = target;
                c.volume_fade_addpertick = spc_div_helper(
                    i32::from(target) - i32::from((c.channel_volume >> 8) as u8),
                    arg,
                );
            }
            0xef => {
                let high = self.next_pattern_byte(ci);
                let loops = self.next_pattern_byte(ci);
                let c = &mut self.channels[ci];
                c.pattern_start_ptr = u16::from_le_bytes([arg, high]);
                c.subroutine_num_loops = loops;
                c.saved_pattern_ptr = c.pattern_order_ptr_for_chan;
                c.pattern_order_ptr_for_chan = c.pattern_start_ptr;
            }
            0xf0 => {
                let c = &mut self.channels[ci];
                c.vibrato_fade_num_ticks = arg;
                c.vibrato_fade_add_per_tick = if arg == 0 { 0xff } else { c.vib_depth / arg };
            }
            0xf4 => self.channels[ci].fine_tune = arg,
            0xf5 => {
                let left = self.next_pattern_byte(ci);
                let right = self.next_pattern_byte(ci);
                self.reg_eon = arg;
                self.echo_channels = arg;
                self.echo_volume_left = u16::from(left) << 8;
                self.echo_volume_right = u16::from(right) << 8;
                self.reg_flg &= !0x20;
            }
            0xf6 => {
                self.echo_volume_left = 0;
                self.echo_volume_right = 0;
                self.reg_flg |= 0x20;
            }
            0xf7 => {
                const FIR: [i8; 32] = [
                    127, 0, 0, 0, 0, 0, 0, 0, 88, -65, -37, -16, -2, 7, 12, 12, 12, 33, 43, 43, 19,
                    -2, -13, -7, 52, 51, 0, -39, -27, 1, -4, -21,
                ];
                self.setup_echo_parameter_edl(arg);
                self.reg_efb = self.next_pattern_byte(ci);
                let preset = usize::from(self.next_pattern_byte(ci)) * 8;
                for i in 0..8u8 {
                    self.dsp_write(FIR0 + i * 16, FIR[preset + usize::from(i)] as u8);
                }
            }
            0xf8 => {
                let left = self.next_pattern_byte(ci);
                let right = self.next_pattern_byte(ci);
                self.echo_volume_fade_ticks = arg;
                self.echo_volume_fade_target_left = left;
                self.echo_volume_fade_target_right = right;
                self.echo_volume_fade_add_left = spc_div_helper(
                    i32::from(left) - i32::from((self.echo_volume_left >> 8) as u8),
                    arg,
                );
                self.echo_volume_fade_add_right = spc_div_helper(
                    i32::from(right) - i32::from((self.echo_volume_right >> 8) as u8),
                    arg,
                );
            }
            0xf9 => {
                let length = self.next_pattern_byte(ci);
                let pitch = self
                    .next_pattern_byte(ci)
                    .wrapping_add(self.global_transposition)
                    .wrapping_add(self.channels[ci].channel_transposition);
                self.channels[ci].pitch_slide_delay_left = arg;
                self.channels[ci].pitch_slide_length = length;
                self.compute_pitch_add(ci, pitch);
            }
            0xfa => self.percussion_base_id = arg,
            _ => panic!("C SPC player reached unimplemented effect {effect:#04x}"),
        }
    }

    fn want_write_kof(&self, ci: usize) -> bool {
        let c = &self.channels[ci];
        let mut loops = c.subroutine_num_loops;
        let mut ptr = c.pattern_order_ptr_for_chan;
        loop {
            let mut cmd = self.ram[ptr as usize];
            ptr = ptr.wrapping_add(1);
            if cmd == 0 {
                if loops == 0 {
                    return true;
                }
                loops = loops.wrapping_sub(1);
                ptr = if loops == 0 {
                    c.saved_pattern_ptr
                } else {
                    c.pattern_start_ptr
                };
            } else {
                while cmd & 0x80 == 0 {
                    cmd = self.ram[ptr as usize];
                    ptr = ptr.wrapping_add(1);
                }
                if cmd == 0xc8 {
                    return false;
                }
                if cmd == 0xef {
                    ptr = read16(&self.ram, ptr);
                } else if cmd >= 0xe0 {
                    ptr = ptr.wrapping_add(u16::from(EFFECT_BYTE_LENGTH[usize::from(cmd - 0xe0)]));
                } else {
                    return true;
                }
            }
        }
    }

    fn calc_vibrato_add_pitch(&mut self, ci: usize, pitch: u16, value: u8) {
        let mut t = u16::from(value) << 2;
        if t & 0x100 != 0 {
            t ^= 0xff;
        }
        let depth = self.channels[ci].vib_depth;
        let r = if depth >= 0xf1 {
            u16::from(t as u8) * u16::from(depth & 0x0f)
        } else {
            (u16::from(t as u8) * u16::from(depth)) >> 8
        };
        let adjusted = if value & 0x80 != 0 {
            pitch.wrapping_sub(r)
        } else {
            pitch.wrapping_add(r)
        };
        self.write_pitch(ci, adjusted);
    }

    fn handle_pan_and_sweep(&mut self, ci: usize) {
        self.did_affect_volumepitch_flag = 0;
        if self.channels[ci].tremolo_depth != 0 {
            self.channels[ci].tremolo_hold_count = self.channels[ci].tremolo_delay_ticks;
            panic!("C SPC player HandleTremolo is unimplemented");
        }
        let mut volume = self.channels[ci].pan_value;
        if self.channels[ci].pan_num_ticks != 0 {
            self.did_affect_volumepitch_flag = 0x80;
            let delta = (i32::from(self.main_tempo_accum)
                * i32::from(self.channels[ci].pan_add_per_tick as i16))
                / 256;
            volume = volume.wrapping_add(delta as u16);
        }
        if self.did_affect_volumepitch_flag != 0 {
            self.write_volume_to_dsp(ci, volume);
        }
        self.did_affect_volumepitch_flag = 0;
        let mut pitch = self.channels[ci].pitch;
        if self.channels[ci].pitch_slide_length != 0
            && self.channels[ci].pitch_slide_delay_left == 0
        {
            self.did_affect_volumepitch_flag = 0x80;
            let delta = (i32::from(self.main_tempo_accum)
                * i32::from(self.channels[ci].pitch_add_per_tick as i16))
                / 256;
            pitch = pitch.wrapping_add(delta as u16);
        }
        if self.channels[ci].vib_depth != 0
            && self.channels[ci].vibrato_delay_ticks == self.channels[ci].vibrato_hold_count
        {
            let value = (((u16::from(self.main_tempo_accum)
                * u16::from(self.channels[ci].vibrato_rate))
                >> 8) as u8)
                .wrapping_add(self.channels[ci].vibrato_count);
            self.calc_vibrato_add_pitch(ci, pitch, value);
        } else if self.did_affect_volumepitch_flag != 0 {
            self.write_pitch(ci, pitch);
        }
    }

    fn handle_note_tick(&mut self, ci: usize) {
        if self.channels[ci].note_keyoff_ticks_left != 0 {
            self.channels[ci].note_keyoff_ticks_left =
                self.channels[ci].note_keyoff_ticks_left.wrapping_sub(1);
            if (self.channels[ci].note_keyoff_ticks_left == 0
                || self.channels[ci].note_ticks_left == 2)
                && self.want_write_kof(ci)
                && self.cur_chan_bit & self.is_chan_on == 0
            {
                self.dsp_write(KOF, self.cur_chan_bit);
            }
        }
        self.did_affect_volumepitch_flag = 0;
        if self.channels[ci].pitch_slide_length != 0 {
            if self.channels[ci].pitch_slide_delay_left != 0 {
                self.channels[ci].pitch_slide_delay_left =
                    self.channels[ci].pitch_slide_delay_left.wrapping_sub(1);
            } else if self.is_chan_on & self.cur_chan_bit == 0 {
                self.did_affect_volumepitch_flag = 0x80;
                let c = &mut self.channels[ci];
                c.pitch_slide_length = c.pitch_slide_length.wrapping_sub(1);
                fade(
                    &mut c.pitch,
                    c.pitch_add_per_tick,
                    c.pitch_target,
                    c.pitch_slide_length != 0,
                );
            }
        }
        let pitch = self.channels[ci].pitch;
        if self.channels[ci].vib_depth != 0 {
            if self.channels[ci].vibrato_delay_ticks == self.channels[ci].vibrato_hold_count {
                let c = &mut self.channels[ci];
                if c.vibrato_change_count == c.vibrato_fade_num_ticks {
                    c.vib_depth = c.vibrato_depth_target;
                } else {
                    let old = c.vibrato_change_count;
                    c.vibrato_change_count = c.vibrato_change_count.wrapping_add(1);
                    c.vib_depth = (if old == 0 { 0 } else { c.vib_depth })
                        .wrapping_add(c.vibrato_fade_add_per_tick);
                }
                c.vibrato_count = c.vibrato_count.wrapping_add(c.vibrato_rate);
                let value = c.vibrato_count;
                self.calc_vibrato_add_pitch(ci, pitch, value);
                return;
            }
            self.channels[ci].vibrato_hold_count =
                self.channels[ci].vibrato_hold_count.wrapping_add(1);
        }
        if self.did_affect_volumepitch_flag != 0 {
            self.write_pitch(ci, pitch);
        }
    }

    fn calc_final_volume(&mut self, ci: usize, volume: u8) {
        let c = &mut self.channels[ci];
        let mut result = (u32::from(self.master_volume >> 8) * u32::from(volume)) >> 8;
        result = (result * u32::from(c.channel_volume_master)) >> 8;
        result = (result * u32::from(c.channel_volume >> 8)) >> 8;
        c.final_volume = ((result * result) >> 8) as u8;
    }

    fn chan_handle_tick(&mut self, ci: usize) {
        if self.channels[ci].volume_fade_ticks != 0 {
            let c = &mut self.channels[ci];
            c.volume_fade_ticks = c.volume_fade_ticks.wrapping_sub(1);
            self.vol_dirty |= self.cur_chan_bit;
            fade(
                &mut c.channel_volume,
                c.volume_fade_addpertick,
                c.volume_fade_target,
                true,
            );
        }
        if self.channels[ci].tremolo_depth != 0 {
            if self.channels[ci].tremolo_delay_ticks == self.channels[ci].tremolo_hold_count {
                self.vol_dirty |= self.cur_chan_bit;
                let c = &mut self.channels[ci];
                if c.tremolo_count & 0x80 != 0 && c.tremolo_depth == 0xff {
                    c.tremolo_count = 0x80;
                } else {
                    c.tremolo_count = c.tremolo_count.wrapping_add(c.tremolo_rate);
                }
                panic!("C SPC player CalcTremolo is unimplemented");
            }
            self.channels[ci].tremolo_hold_count =
                self.channels[ci].tremolo_hold_count.wrapping_add(1);
        }
        self.calc_final_volume(ci, 0xff);
        if self.channels[ci].pan_num_ticks != 0 {
            let c = &mut self.channels[ci];
            c.pan_num_ticks = c.pan_num_ticks.wrapping_sub(1);
            self.vol_dirty |= self.cur_chan_bit;
            fade(
                &mut c.pan_value,
                c.pan_add_per_tick,
                c.pan_target_value,
                true,
            );
        }
        if self.vol_dirty & self.cur_chan_bit != 0 {
            self.write_volume_to_dsp(ci, self.channels[ci].pan_value);
        }
    }

    fn play_note(&mut self, ci: usize, mut note: u8) {
        if note >= 0xca {
            self.channel_set_instrument(ci, note);
            note = 0xa4;
        }
        if note >= 0xc8 || self.is_chan_on & self.cur_chan_bit != 0 {
            return;
        }
        let c = &mut self.channels[ci];
        c.pitch = u16::from(
            (note & 0x7f)
                .wrapping_add(self.global_transposition)
                .wrapping_add(c.channel_transposition),
        ) << 8
            | u16::from(c.fine_tune);
        c.vibrato_count = c.vibrato_fade_num_ticks << 7;
        c.vibrato_hold_count = 0;
        c.vibrato_change_count = 0;
        c.tremolo_count = 0;
        c.tremolo_hold_count = 0;
        self.vol_dirty |= self.cur_chan_bit;
        self.key_on |= self.cur_chan_bit;
        c.pitch_slide_length = c.pitch_envelope_num_ticks;
        if c.pitch_slide_length != 0 {
            c.pitch_slide_delay_left = c.pitch_envelope_delay;
            if c.pitch_envelope_direction == 0 {
                c.pitch = c
                    .pitch
                    .wrapping_sub(u16::from(c.pitch_envelope_slide_value) << 8);
            }
            let target = (c.pitch >> 8) as u8;
            let target = target.wrapping_add(c.pitch_envelope_slide_value);
            self.compute_pitch_add(ci, target);
        }
        self.write_pitch(ci, self.channels[ci].pitch);
    }

    fn port0_handle_music(&mut self) {
        let command = self.new_value_from_snes[0];
        if command == 0xff {
            panic!("C SPC player music upload command 0xff is unimplemented");
        }
        if command == 0xf1 {
            self.master_volume_fade_ticks = 0x80;
            self.pause_music_ctr = 0x80;
            self.master_volume_fade_target = 0;
            self.master_volume_fade_add_per_tick =
                spc_div_helper(-i32::from((self.master_volume >> 8) as u8), 0x80);
        } else if command == 0xf2 {
            if self.byte_3e1 != 0 {
                return;
            }
            self.byte_3e1 = (self.master_volume >> 8) as u8;
            self.master_volume = (self.master_volume & 0xff) | 0x7000;
        } else if command == 0xf3 {
            if self.byte_3e1 == 0 {
                return;
            }
            self.master_volume = (self.master_volume & 0xff) | (u16::from(self.byte_3e1) << 8);
            self.byte_3e1 = 0;
        } else if command == 0xf0 {
            self.pause_music();
            return;
        } else if command != 0 {
            self.pause_music_ctr = 0;
            self.byte_3e1 = 0;
            self.port_to_snes[0] = command;
            self.music_ptr_toplevel = read16(
                &self.ram,
                0xd000u16.wrapping_add(u16::from(command - 1) * 2),
            );
            self.counter_sf0c = 2;
            self.key_off |= self.is_chan_on ^ 0xff;
            return;
        }

        if self.port_to_snes[0] == 0 {
            return;
        }
        if self.pause_music_ctr != 0 {
            self.pause_music_ctr = self.pause_music_ctr.wrapping_sub(1);
            if self.pause_music_ctr == 0 {
                self.pause_music();
                return;
            }
        }
        if self.counter_sf0c != 0 {
            self.counter_sf0c = self.counter_sf0c.wrapping_sub(1);
            if self.counter_sf0c != 0 {
                self.music_reset_chan();
                return;
            }
            if !self.load_next_phrase() {
                return;
            }
        }

        'restart_phrase: loop {
            self.vol_dirty = 0;
            self.cur_chan_bit = 1;
            for ci in 0..8 {
                if self.channels[ci].pattern_order_ptr_for_chan >> 8 != 0 {
                    self.channels[ci].note_ticks_left =
                        self.channels[ci].note_ticks_left.wrapping_sub(1);
                    if self.channels[ci].note_ticks_left == 0 {
                        loop {
                            let mut cmd = self.next_pattern_byte(ci);
                            if cmd == 0 {
                                if self.channels[ci].subroutine_num_loops == 0 {
                                    if !self.load_next_phrase() {
                                        return;
                                    }
                                    continue 'restart_phrase;
                                }
                                let c = &mut self.channels[ci];
                                c.subroutine_num_loops = c.subroutine_num_loops.wrapping_sub(1);
                                c.pattern_order_ptr_for_chan = if c.subroutine_num_loops == 0 {
                                    c.saved_pattern_ptr
                                } else {
                                    c.pattern_start_ptr
                                };
                                continue;
                            }
                            if cmd & 0x80 == 0 {
                                const NOTE_VOL: [u8; 16] = [
                                    25, 50, 76, 101, 114, 127, 140, 152, 165, 178, 191, 203, 216,
                                    229, 242, 252,
                                ];
                                const GATE: [u8; 8] = [50, 101, 127, 152, 178, 203, 229, 252];
                                self.channels[ci].note_length = cmd;
                                cmd = self.next_pattern_byte(ci);
                                if cmd & 0x80 == 0 {
                                    self.channels[ci].note_gate_off_fixedpt =
                                        GATE[usize::from((cmd >> 4) & 7)];
                                    self.channels[ci].channel_volume_master =
                                        NOTE_VOL[usize::from(cmd & 0xf)];
                                    cmd = self.next_pattern_byte(ci);
                                }
                            }
                            if cmd >= 0xe0 {
                                self.handle_effect(ci, cmd);
                                continue;
                            }
                            if self.fast_forward == 0 && self.is_chan_on & self.cur_chan_bit == 0 {
                                self.play_note(ci, cmd);
                            }
                            let c = &mut self.channels[ci];
                            c.note_ticks_left = c.note_length;
                            let gate = (u16::from(c.note_ticks_left)
                                * u16::from(c.note_gate_off_fixedpt))
                                >> 8;
                            c.note_keyoff_ticks_left = if gate == 0 { 1 } else { gate as u8 };
                            self.pitch_slide_to_note_check(ci);
                            break;
                        }
                    } else if self.fast_forward == 0 {
                        self.handle_note_tick(ci);
                        self.pitch_slide_to_note_check(ci);
                    }
                }
                self.cur_chan_bit = self.cur_chan_bit.wrapping_shl(1);
            }
            break;
        }
        if self.tempo_fade_num_ticks != 0 {
            self.tempo_fade_num_ticks = self.tempo_fade_num_ticks.wrapping_sub(1);
            self.tempo = if self.tempo_fade_num_ticks == 0 {
                u16::from(self.tempo_fade_final) << 8
            } else {
                self.tempo.wrapping_add(self.tempo_fade_add)
            };
        }
        if self.echo_volume_fade_ticks != 0 {
            self.echo_volume_left = self
                .echo_volume_left
                .wrapping_add(self.echo_volume_fade_add_left);
            self.echo_volume_right = self
                .echo_volume_right
                .wrapping_add(self.echo_volume_fade_add_right);
            self.echo_volume_fade_ticks = self.echo_volume_fade_ticks.wrapping_sub(1);
            if self.echo_volume_fade_ticks == 0 {
                self.echo_volume_left = u16::from(self.echo_volume_fade_target_left) << 8;
                self.echo_volume_right = u16::from(self.echo_volume_fade_target_right) << 8;
            }
        }
        if self.master_volume_fade_ticks != 0 {
            self.master_volume_fade_ticks = self.master_volume_fade_ticks.wrapping_sub(1);
            self.master_volume = if self.master_volume_fade_ticks == 0 {
                u16::from(self.master_volume_fade_target) << 8
            } else {
                self.master_volume
                    .wrapping_add(self.master_volume_fade_add_per_tick)
            };
            self.vol_dirty = 0xff;
        }
        self.cur_chan_bit = 1;
        for ci in 0..8 {
            if self.channels[ci].pattern_order_ptr_for_chan >> 8 != 0 {
                self.chan_handle_tick(ci);
            }
            self.cur_chan_bit = self.cur_chan_bit.wrapping_shl(1);
        }
    }

    fn pause_music(&mut self) {
        self.key_off = self.is_chan_on ^ 0xff;
        self.port_to_snes[0] = 0;
        self.cur_chan_bit = 0;
    }

    fn load_next_phrase(&mut self) -> bool {
        loop {
            let value = read16(&self.ram, self.music_ptr_toplevel);
            self.music_ptr_toplevel = self.music_ptr_toplevel.wrapping_add(2);
            if value >> 8 != 0 {
                let mut address = value;
                for ci in 0..8 {
                    self.channels[ci].pattern_order_ptr_for_chan = read16(&self.ram, address);
                    address = address.wrapping_add(2);
                }
                self.cur_chan_bit = 1;
                for ci in 0..8 {
                    if self.channels[ci].pattern_order_ptr_for_chan >> 8 != 0
                        && self.channels[ci].instrument_id == 0
                    {
                        self.channel_set_instrument(ci, 0);
                    }
                    let c = &mut self.channels[ci];
                    c.subroutine_num_loops = 0;
                    c.volume_fade_ticks = 0;
                    c.pan_num_ticks = 0;
                    c.note_ticks_left = 1;
                    self.cur_chan_bit = self.cur_chan_bit.wrapping_shl(1);
                }
                return true;
            }
            match value {
                0 => {
                    self.pause_music();
                    return false;
                }
                0x80 => self.fast_forward = 0x80,
                0x81 => self.fast_forward = 0,
                _ => {
                    self.block_count = self.block_count.wrapping_sub(1);
                    if (self.block_count as i8) < 0 {
                        self.block_count = value as u8;
                    }
                    let destination = read16(&self.ram, self.music_ptr_toplevel);
                    self.music_ptr_toplevel = self.music_ptr_toplevel.wrapping_add(2);
                    if self.block_count != 0 {
                        self.music_ptr_toplevel = destination;
                    }
                }
            }
        }
    }

    fn sfx_turn_off_channel(&mut self, ci: usize) {
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

    fn write_key_on(&mut self, bit: u8) {
        self.dsp_write(KOF, 0);
        self.dsp_write(KON, bit);
    }

    fn sfx_maybe_disable_echo(&mut self) {
        if (self.port_to_snes[0] & 0x10 == 0
            || self.current_bit & self.sfx_channels_echo_mask2 != 0)
            && self.current_bit & self.reg_eon != 0
        {
            self.reg_eon ^= self.current_bit;
            self.dsp_write(EON, self.reg_eon);
        }
    }

    fn sfx_channel_tick(&mut self, ci: usize, is_continue: bool) {
        if is_continue {
            self.sfx_maybe_disable_echo();
            self.sfx_channel_index = self.channels[ci].index * 2;
            self.sfx_sound_ptr_cur = self.channels[ci].sfx_sound_ptr;
            self.channels[ci].sfx_note_length_left =
                self.channels[ci].sfx_note_length_left.wrapping_sub(1);
            if self.channels[ci].sfx_note_length_left != 0 {
                self.sfx_note_continue(ci);
                self.channels[ci].sfx_sound_ptr = self.sfx_sound_ptr_cur;
                return;
            }
            self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
        }
        loop {
            self.dsp_register_index = self.sfx_channel_index * 8;
            let mut cmd = self.ram[self.sfx_sound_ptr_cur as usize];
            if cmd == 0 {
                self.sfx_turn_off_channel(ci);
                return;
            }
            if cmd & 0x80 == 0 {
                self.channels[ci].sfx_note_length = cmd;
                self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                cmd = self.ram[self.sfx_sound_ptr_cur as usize];
                if cmd & 0x80 == 0 {
                    if self.port1_active & self.current_bit != 0 {
                        if cmd == 0 || self.channel_67_volume == 0 {
                            let volume = cmd;
                            self.dsp_write(self.dsp_register_index + V0VOLL, cmd);
                            self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                            cmd = self.ram[self.sfx_sound_ptr_cur as usize];
                            if cmd & 0x80 != 0 {
                                self.dsp_write(self.dsp_register_index + V0VOLR, volume);
                            } else {
                                self.dsp_write(self.dsp_register_index + V0VOLR, cmd);
                                self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                                cmd = self.ram[self.sfx_sound_ptr_cur as usize];
                            }
                        } else {
                            self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                            cmd = self.ram[self.sfx_sound_ptr_cur as usize];
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
                        self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                        cmd = self.ram[self.sfx_sound_ptr_cur as usize];
                    }
                }
            }
            if cmd == 0xe0 {
                self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                let preset = usize::from(self.ram[self.sfx_sound_ptr_cur as usize]);
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
                self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
            } else if cmd == 0xf9 || cmd == 0xf1 {
                if cmd == 0xf9 {
                    self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                    let note = self.ram[self.sfx_sound_ptr_cur as usize];
                    self.play_note(ci, note);
                    self.write_key_on(self.current_bit);
                }
                self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                self.channels[ci].pitch_slide_delay_left =
                    self.ram[self.sfx_sound_ptr_cur as usize];
                self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                self.channels[ci].pitch_slide_length = self.ram[self.sfx_sound_ptr_cur as usize];
                self.sfx_sound_ptr_cur = self.sfx_sound_ptr_cur.wrapping_add(1);
                let target = self.ram[self.sfx_sound_ptr_cur as usize];
                self.compute_pitch_add(ci, target);
                self.channels[ci].sfx_note_length_left = self.channels[ci].sfx_note_length;
                self.sfx_note_continue(ci);
                break;
            } else if cmd == 0xff {
                let table = 0x17c0u16
                    .wrapping_add(u16::from(self.channels[ci].sfx_which_sound.wrapping_sub(1)) * 2);
                let pointer = read16(&self.ram, table);
                self.sfx_sound_ptr_cur = pointer;
                self.channels[ci].sfx_sound_ptr = pointer;
            } else {
                self.play_note(ci, cmd);
                self.write_key_on(self.current_bit);
                self.channels[ci].sfx_note_length_left = self.channels[ci].sfx_note_length;
                self.sfx_note_continue(ci);
                break;
            }
        }
        self.channels[ci].sfx_sound_ptr = self.sfx_sound_ptr_cur;
    }

    fn sfx_note_continue(&mut self, ci: usize) {
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

    fn port1_play_inner(&mut self) {
        self.port1_counter = 0;
        let sound = self.new_value_from_snes[1];
        self.channels[7].sfx_which_sound = sound;
        self.channels[7].sfx_arr_countdown = 3;
        self.channels[7].pitch_envelope_num_ticks = 0;
        self.port1_active = 0x80;
        self.is_chan_on |= 0x80;
        self.dsp_write(KOF, 0x80);
        self.new_value_from_snes[1] = self.ram[0x1800 + usize::from(sound.wrapping_sub(1))];
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

    fn port1_start_new_sound(&mut self) {
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
                        let address = 0x17c0u16.wrapping_add(
                            u16::from(self.channels[ci].sfx_which_sound.wrapping_sub(1)) * 2,
                        );
                        let pointer = read16(&self.ram, address);
                        self.sfx_sound_ptr_cur = pointer;
                        self.channels[ci].sfx_sound_ptr = pointer;
                        self.sfx_channel_tick(ci, false);
                    }
                }
            }
            self.current_bit >>= 1;
        }
    }

    fn port1_handle_cmd(&mut self) {
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

    fn port_start_new_sound(&mut self, port: usize) {
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
                        let base = if port == 2 { 0x1820 } else { 0x191c };
                        let address = base
                            + usize::from(self.channels[ci].sfx_which_sound.wrapping_sub(1)) * 2;
                        let pointer = read16(&self.ram, address as u16);
                        self.sfx_sound_ptr_cur = pointer;
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

    fn allocate_sfx_channel(&mut self, port: usize) -> usize {
        let command = self.new_value_from_snes[port];
        let flag_address = if port == 2 {
            0x18dd + usize::from((command & 0x3f).wrapping_sub(1))
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

    fn port_handle_cmd(&mut self, port: usize) {
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
            let base = if port == 2 { 0x189e } else { 0x199a };
            self.new_value_from_snes[port] =
                self.ram[base + usize::from(self.channels[ci].sfx_which_sound.wrapping_sub(1))];
        }
    }

    fn read_port_from_snes(&mut self, port: usize) {
        let old = self.last_value_from_snes[port];
        self.last_value_from_snes[port] = self.input_ports[port];
        self.new_value_from_snes[port] = if self.input_ports[port] != old {
            self.input_ports[port]
        } else {
            0
        };
    }

    fn loop_part1(&mut self) {
        self.dsp_write(KOF, self.key_off);
        self.dsp_write(PMON, self.reg_pmon);
        self.dsp_write(NON, self.reg_non);
        self.dsp_write(KOF, 0);
        self.dsp_write(KON, self.key_on);
        if self.echo_stored_time & 0x80 == 0 {
            self.dsp_write(FLG, self.reg_flg);
            if self.echo_stored_time == self.echo_parameter_edl {
                self.dsp_write(EON, self.reg_eon);
                self.dsp_write(EFB, self.reg_efb);
                self.dsp_write(EVOLR, (self.echo_volume_right >> 8) as u8);
                self.dsp_write(EVOLL, (self.echo_volume_left >> 8) as u8);
            }
        }
        self.key_off = 0;
        self.key_on = 0;
    }

    fn loop_part2(&mut self, ticks: u8) {
        let total = u16::from(self.sfx_timer_accum) + u16::from(ticks.wrapping_mul(0x38));
        self.sfx_timer_accum = total as u8;
        if total >= 256 {
            self.port1_start_new_sound();
            self.port1_handle_cmd();
            self.read_port_from_snes(1);
            self.port_start_new_sound(2);
            self.port_handle_cmd(2);
            self.read_port_from_snes(2);
            self.port_start_new_sound(3);
            self.port_handle_cmd(3);
            self.read_port_from_snes(3);
            if self.echo_stored_time != self.echo_parameter_edl {
                self.echo_fract_incr = self.echo_fract_incr.wrapping_add(1);
                if self.echo_fract_incr & 1 == 0 {
                    self.echo_stored_time = self.echo_stored_time.wrapping_add(1);
                }
            }
        }
        let total = u16::from(self.main_tempo_accum)
            + u16::from(ticks.wrapping_mul((self.tempo >> 8) as u8));
        self.main_tempo_accum = total as u8;
        if total >= 256 {
            self.port0_handle_music();
            self.read_port_from_snes(0);
        } else if self.port_to_snes[0] != 0 {
            self.cur_chan_bit = 1;
            for ci in 0..8 {
                if self.channels[ci].pattern_order_ptr_for_chan >> 8 != 0 {
                    self.handle_pan_and_sweep(ci);
                }
                self.cur_chan_bit = self.cur_chan_bit.wrapping_shl(1);
            }
        }
    }

    pub fn initialize(&mut self) {
        self.dsp.reset();
        let history = self.history.take();
        let dsp = core::mem::take(&mut self.dsp);
        *self = Self::default();
        self.history = history;
        self.dsp = dsp;
        self.setup_echo_parameter_edl(1);
        self.reg_flg |= 0x20;
        self.dsp_write(MVOLL, 0x60);
        self.dsp_write(MVOLR, 0x60);
        self.dsp_write(DIR, 0x3c);
        self.tempo = (self.tempo & 0xff) | 0x1000;
        self.timer_cycles = 0;
        self.loop_part1();
    }

    pub fn copy_variables_to_ram(&mut self) {
        for i in 0..8 {
            let c = &self.channels[i];
            macro_rules! p8 {
                ($f:ident, $a:expr) => {
                    self.ram[$a + i * 2] = c.$f;
                };
            }
            macro_rules! p16 {
                ($f:ident, $a:expr) => {{
                    let b = c.$f.to_le_bytes();
                    self.ram[$a + i * 2] = b[0];
                    self.ram[$a + i * 2 + 1] = b[1];
                }};
            }
            p16!(pattern_order_ptr_for_chan, 0x30);
            p8!(note_ticks_left, 0x70);
            p8!(note_keyoff_ticks_left, 0x71);
            p8!(subroutine_num_loops, 0x80);
            p8!(volume_fade_ticks, 0x90);
            p8!(pan_num_ticks, 0x91);
            p8!(pitch_slide_length, 0xa0);
            p8!(pitch_slide_delay_left, 0xa1);
            p8!(vibrato_hold_count, 0xb0);
            p8!(vib_depth, 0xb1);
            p8!(tremolo_hold_count, 0xc0);
            p8!(tremolo_depth, 0xc1);
            p8!(vibrato_change_count, 0x100);
            p8!(note_length, 0x200);
            p8!(note_gate_off_fixedpt, 0x201);
            p8!(channel_volume_master, 0x210);
            p8!(instrument_id, 0x211);
            p16!(instrument_pitch_base, 0x220);
            p16!(saved_pattern_ptr, 0x230);
            p16!(pattern_start_ptr, 0x240);
            p8!(pitch_envelope_num_ticks, 0x280);
            p8!(pitch_envelope_delay, 0x281);
            p8!(pitch_envelope_direction, 0x290);
            p8!(pitch_envelope_slide_value, 0x291);
            p8!(vibrato_count, 0x2a0);
            p8!(vibrato_rate, 0x2a1);
            p8!(vibrato_delay_ticks, 0x2b0);
            p8!(vibrato_fade_num_ticks, 0x2b1);
            p8!(vibrato_fade_add_per_tick, 0x2c0);
            p8!(vibrato_depth_target, 0x2c1);
            p8!(tremolo_count, 0x2d0);
            p8!(tremolo_rate, 0x2d1);
            p8!(tremolo_delay_ticks, 0x2e0);
            p8!(channel_transposition, 0x2f0);
            p16!(channel_volume, 0x300);
            p16!(volume_fade_addpertick, 0x310);
            p8!(volume_fade_target, 0x320);
            p8!(final_volume, 0x321);
            p16!(pan_value, 0x330);
            p16!(pan_add_per_tick, 0x340);
            p8!(pan_target_value, 0x350);
            p8!(pan_flag_with_phase_invert, 0x351);
            p16!(pitch, 0x360);
            p16!(pitch_add_per_tick, 0x370);
            p8!(pitch_target, 0x380);
            p8!(fine_tune, 0x381);
            p16!(sfx_sound_ptr, 0x390);
            p8!(sfx_which_sound, 0x3a0);
            p8!(sfx_arr_countdown, 0x3a1);
            p8!(sfx_note_length_left, 0x3b0);
            p8!(sfx_note_length, 0x3b1);
            p8!(sfx_pan, 0x3d0);
        }
        self.copy_player_to_ram();
    }

    fn copy_player_to_ram(&mut self) {
        self.ram[0..4].copy_from_slice(&self.new_value_from_snes);
        self.ram[4..8].copy_from_slice(&self.port_to_snes);
        self.ram[8..12].copy_from_slice(&self.last_value_from_snes);
        macro_rules! p8 {
            ($f:ident, $a:expr) => {
                self.ram[$a] = self.$f;
            };
        }
        macro_rules! p16 {
            ($f:ident, $a:expr) => {
                self.ram[$a..$a + 2].copy_from_slice(&self.$f.to_le_bytes());
            };
        }
        p8!(counter_sf0c, 0xc);
        p16!(always_zero, 0xe);
        p16!(temp_accum, 0x10);
        p8!(ttt, 0x12);
        p8!(did_affect_volumepitch_flag, 0x13);
        p16!(addr0, 0x14);
        p16!(addr1, 0x16);
        p16!(lfsr_value, 0x18);
        p8!(is_chan_on, 0x1a);
        p8!(fast_forward, 0x1b);
        p8!(sfx_start_arg_pan, 0x20);
        p16!(sfx_sound_ptr_cur, 0x2c);
        p16!(music_ptr_toplevel, 0x40);
        p8!(block_count, 0x42);
        p8!(sfx_timer_accum, 0x43);
        p8!(chn, 0x44);
        p8!(key_on, 0x45);
        p8!(key_off, 0x46);
        p8!(cur_chan_bit, 0x47);
        p8!(reg_flg, 0x48);
        p8!(reg_non, 0x49);
        p8!(reg_eon, 0x4a);
        p8!(reg_pmon, 0x4b);
        p8!(echo_stored_time, 0x4c);
        p8!(echo_parameter_edl, 0x4d);
        p8!(reg_efb, 0x4e);
        p8!(global_transposition, 0x50);
        p8!(main_tempo_accum, 0x51);
        p16!(tempo, 0x52);
        p8!(tempo_fade_num_ticks, 0x54);
        p8!(tempo_fade_final, 0x55);
        p16!(tempo_fade_add, 0x56);
        p16!(master_volume, 0x58);
        p8!(master_volume_fade_ticks, 0x5a);
        p8!(master_volume_fade_target, 0x5b);
        p16!(master_volume_fade_add_per_tick, 0x5c);
        p8!(vol_dirty, 0x5e);
        p8!(percussion_base_id, 0x5f);
        p16!(echo_volume_left, 0x60);
        p16!(echo_volume_right, 0x62);
        p16!(echo_volume_fade_add_left, 0x64);
        p16!(echo_volume_fade_add_right, 0x66);
        p8!(echo_volume_fade_ticks, 0x68);
        p8!(echo_volume_fade_target_left, 0x69);
        p8!(echo_volume_fade_target_right, 0x6a);
        p8!(sfx_channel_index, 0x3c0);
        p8!(current_bit, 0x3c1);
        p8!(dsp_register_index, 0x3c2);
        p8!(echo_channels, 0x3c3);
        p8!(byte_3c4, 0x3c4);
        p8!(byte_3c5, 0x3c5);
        p8!(echo_fract_incr, 0x3c7);
        p8!(sfx_channel_index2, 0x3c8);
        p8!(sfx_channel_bit, 0x3c9);
        p8!(pause_music_ctr, 0x3ca);
        p8!(port2_active, 0x3cb);
        p8!(port2_current_bit, 0x3cc);
        p8!(port3_active, 0x3cd);
        p8!(port3_current_bit, 0x3ce);
        p8!(port1_active, 0x3cf);
        p8!(port1_current_bit, 0x3e0);
        p8!(byte_3e1, 0x3e1);
        p8!(sfx_play_echo_flag, 0x3e2);
        p8!(sfx_channels_echo_mask2, 0x3e3);
        p8!(port1_counter, 0x3e4);
        p8!(channel_67_volume, 0x3e5);
        p8!(cutk_always_zero, 0x3ff);
    }

    pub fn copy_variables_from_ram(&mut self) {
        for i in 0..8 {
            macro_rules! g8 {
                ($f:ident, $a:expr) => {
                    self.channels[i].$f = self.ram[$a + i * 2];
                };
            }
            macro_rules! g16 {
                ($f:ident, $a:expr) => {
                    self.channels[i].$f =
                        u16::from_le_bytes([self.ram[$a + i * 2], self.ram[$a + i * 2 + 1]]);
                };
            }
            g16!(pattern_order_ptr_for_chan, 0x30);
            g8!(note_ticks_left, 0x70);
            g8!(note_keyoff_ticks_left, 0x71);
            g8!(subroutine_num_loops, 0x80);
            g8!(volume_fade_ticks, 0x90);
            g8!(pan_num_ticks, 0x91);
            g8!(pitch_slide_length, 0xa0);
            g8!(pitch_slide_delay_left, 0xa1);
            g8!(vibrato_hold_count, 0xb0);
            g8!(vib_depth, 0xb1);
            g8!(tremolo_hold_count, 0xc0);
            g8!(tremolo_depth, 0xc1);
            g8!(vibrato_change_count, 0x100);
            g8!(note_length, 0x200);
            g8!(note_gate_off_fixedpt, 0x201);
            g8!(channel_volume_master, 0x210);
            g8!(instrument_id, 0x211);
            g16!(instrument_pitch_base, 0x220);
            g16!(saved_pattern_ptr, 0x230);
            g16!(pattern_start_ptr, 0x240);
            g8!(pitch_envelope_num_ticks, 0x280);
            g8!(pitch_envelope_delay, 0x281);
            g8!(pitch_envelope_direction, 0x290);
            g8!(pitch_envelope_slide_value, 0x291);
            g8!(vibrato_count, 0x2a0);
            g8!(vibrato_rate, 0x2a1);
            g8!(vibrato_delay_ticks, 0x2b0);
            g8!(vibrato_fade_num_ticks, 0x2b1);
            g8!(vibrato_fade_add_per_tick, 0x2c0);
            g8!(vibrato_depth_target, 0x2c1);
            g8!(tremolo_count, 0x2d0);
            g8!(tremolo_rate, 0x2d1);
            g8!(tremolo_delay_ticks, 0x2e0);
            g8!(channel_transposition, 0x2f0);
            g16!(channel_volume, 0x300);
            g16!(volume_fade_addpertick, 0x310);
            g8!(volume_fade_target, 0x320);
            g8!(final_volume, 0x321);
            g16!(pan_value, 0x330);
            g16!(pan_add_per_tick, 0x340);
            g8!(pan_target_value, 0x350);
            g8!(pan_flag_with_phase_invert, 0x351);
            g16!(pitch, 0x360);
            g16!(pitch_add_per_tick, 0x370);
            g8!(pitch_target, 0x380);
            g8!(fine_tune, 0x381);
            g16!(sfx_sound_ptr, 0x390);
            g8!(sfx_which_sound, 0x3a0);
            g8!(sfx_arr_countdown, 0x3a1);
            g8!(sfx_note_length_left, 0x3b0);
            g8!(sfx_note_length, 0x3b1);
            g8!(sfx_pan, 0x3d0);
        }
        self.copy_player_from_ram();
    }

    fn copy_player_from_ram(&mut self) {
        self.new_value_from_snes.copy_from_slice(&self.ram[0..4]);
        self.port_to_snes.copy_from_slice(&self.ram[4..8]);
        self.last_value_from_snes.copy_from_slice(&self.ram[8..12]);
        macro_rules! g8 {
            ($f:ident, $a:expr) => {
                self.$f = self.ram[$a];
            };
        }
        macro_rules! g16 {
            ($f:ident, $a:expr) => {
                self.$f = u16::from_le_bytes([self.ram[$a], self.ram[$a + 1]]);
            };
        }
        g8!(counter_sf0c, 0xc);
        g16!(always_zero, 0xe);
        g16!(temp_accum, 0x10);
        g8!(ttt, 0x12);
        g8!(did_affect_volumepitch_flag, 0x13);
        g16!(addr0, 0x14);
        g16!(addr1, 0x16);
        g16!(lfsr_value, 0x18);
        g8!(is_chan_on, 0x1a);
        g8!(fast_forward, 0x1b);
        g8!(sfx_start_arg_pan, 0x20);
        g16!(sfx_sound_ptr_cur, 0x2c);
        g16!(music_ptr_toplevel, 0x40);
        g8!(block_count, 0x42);
        g8!(sfx_timer_accum, 0x43);
        g8!(chn, 0x44);
        g8!(key_on, 0x45);
        g8!(key_off, 0x46);
        g8!(cur_chan_bit, 0x47);
        g8!(reg_flg, 0x48);
        g8!(reg_non, 0x49);
        g8!(reg_eon, 0x4a);
        g8!(reg_pmon, 0x4b);
        g8!(echo_stored_time, 0x4c);
        g8!(echo_parameter_edl, 0x4d);
        g8!(reg_efb, 0x4e);
        g8!(global_transposition, 0x50);
        g8!(main_tempo_accum, 0x51);
        g16!(tempo, 0x52);
        g8!(tempo_fade_num_ticks, 0x54);
        g8!(tempo_fade_final, 0x55);
        g16!(tempo_fade_add, 0x56);
        g16!(master_volume, 0x58);
        g8!(master_volume_fade_ticks, 0x5a);
        g8!(master_volume_fade_target, 0x5b);
        g16!(master_volume_fade_add_per_tick, 0x5c);
        g8!(vol_dirty, 0x5e);
        g8!(percussion_base_id, 0x5f);
        g16!(echo_volume_left, 0x60);
        g16!(echo_volume_right, 0x62);
        g16!(echo_volume_fade_add_left, 0x64);
        g16!(echo_volume_fade_add_right, 0x66);
        g8!(echo_volume_fade_ticks, 0x68);
        g8!(echo_volume_fade_target_left, 0x69);
        g8!(echo_volume_fade_target_right, 0x6a);
        g8!(sfx_channel_index, 0x3c0);
        g8!(current_bit, 0x3c1);
        g8!(dsp_register_index, 0x3c2);
        g8!(echo_channels, 0x3c3);
        g8!(byte_3c4, 0x3c4);
        g8!(byte_3c5, 0x3c5);
        g8!(echo_fract_incr, 0x3c7);
        g8!(sfx_channel_index2, 0x3c8);
        g8!(sfx_channel_bit, 0x3c9);
        g8!(pause_music_ctr, 0x3ca);
        g8!(port2_active, 0x3cb);
        g8!(port2_current_bit, 0x3cc);
        g8!(port3_active, 0x3cd);
        g8!(port3_current_bit, 0x3ce);
        g8!(port1_active, 0x3cf);
        g8!(port1_current_bit, 0x3e0);
        g8!(byte_3e1, 0x3e1);
        g8!(sfx_play_echo_flag, 0x3e2);
        g8!(sfx_channels_echo_mask2, 0x3e3);
        g8!(port1_counter, 0x3e4);
        g8!(channel_67_volume, 0x3e5);
        g8!(cutk_always_zero, 0x3ff);
    }
    pub fn upload(&mut self, data: &[u8]) -> Result<(), UploadError> {
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
    pub fn generate_samples(&mut self) {
        assert!(self.timer_cycles <= 64);
        assert!(self.dsp.sample_offset <= SAMPLE_FRAMES as u16);
        while self.dsp.sample_offset < SAMPLE_FRAMES as u16 {
            if self.timer_cycles >= 64 {
                self.loop_part2(self.timer_cycles >> 6);
                self.loop_part1();
                self.timer_cycles &= 63
            }
            let n = (SAMPLE_FRAMES as u16 - self.dsp.sample_offset)
                .min(u16::from(64 - self.timer_cycles)) as u8;
            self.timer_cycles += n;
            for _ in 0..n {
                self.dsp.cycle(&mut self.ram)
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadError {
    Truncated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_wraps() {
        let mut p = SpcPlayer::new();
        p.upload(&[3, 0, 0xff, 0xff, 1, 2, 3, 0, 0]).unwrap();
        assert_eq!(p.ram[0xffff], 1);
        assert_eq!(p.ram[0], 2);
        assert_eq!(p.ram[1], 3)
    }
    #[test]
    fn generate_frame() {
        let mut p = SpcPlayer::new();
        p.generate_samples();
        assert_eq!(p.dsp.sample_offset, 534);
        assert_eq!(p.timer_cycles, 22);
    }

    #[test]
    fn initialize_matches_c_reset_sequence() {
        let mut p = SpcPlayer::new();
        p.ram[0x1234] = 0xaa;
        p.initialize();
        assert_eq!(p.ram[0x1234], 0);
        assert_eq!(p.echo_parameter_edl, 1);
        assert_eq!(p.echo_stored_time, 0xff);
        assert_eq!(p.last_written_edl, 1);
        assert_eq!(p.reg_flg, 0x20);
        assert_eq!(p.tempo, 0x1000);
        assert_eq!(p.dsp.read(MVOLL), 0x60);
        assert_eq!(p.dsp.read(MVOLR), 0x60);
        assert_eq!(p.dsp.read(DIR), 0x3c);
        assert_eq!(p.dsp.read(EDL), 1);
        assert_eq!(p.dsp.read(ESA), 0xc8);
    }

    #[test]
    fn ram_mirror_round_trips_all_mapped_fields() {
        let mut p = SpcPlayer::new();
        for (i, byte) in p.ram[..0x400].iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(73).wrapping_add(19);
        }
        p.copy_variables_from_ram();
        p.ram[..0x400].fill(0);
        p.copy_variables_to_ram();
        let first = p.ram[..0x400].to_vec();

        let mut q = SpcPlayer::new();
        q.ram[..0x400].copy_from_slice(&first);
        q.copy_variables_from_ram();
        q.ram[..0x400].fill(0);
        q.copy_variables_to_ram();
        assert_eq!(&q.ram[..0x400], first.as_slice());
        assert_eq!(q.channels, p.channels);
    }

    #[test]
    fn signed_fixed_point_division_matches_spc_arithmetic() {
        assert_eq!(spc_div_helper(7, 2), 0x0380);
        assert_eq!(spc_div_helper(-7, 2), 0xfc80);
        assert_eq!(spc_div_helper(5, 0), 0xffff);
    }
}
