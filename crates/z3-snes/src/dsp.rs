//! Exact S-DSP state machine and sample generation from `snes/dsp.c`.
//!
//! The C object borrows APU RAM through a pointer. This safe port preserves the
//! ownership boundary by accepting the shared RAM on methods which access it.

const RATE: [u16; 32] = [
    0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80, 64, 48, 40, 32,
    24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
];
const GAUSS_HEX: &str = "00000000000000000000000000000000000000000000000000100100100100100100100100100100100200200200200200200200300300300300300400400400400400500500500500600600600600700700700800800800900900900a00a00a00b00b00b00c00c00d00d00e00e00f00f00f01001001101101201301301401401501501601701701801801901a01b01b01c01d01d01e01f02002002102202302402402502602702802902a02b02c02d02e02f03003103203303403503603703803a03b03c03d03e04004104204304504604704904a04c04d04e05005105305405605705905a05c05e05f06106306406606806a06b06d06f07107307507607807a07c07e08008208408608908b08d08f09109309609809a09c09f0a10a30a60a80ab0ad0af0b20b40b70ba0bc0bf0c10c40c70c90cc0cf0d20d40d70da0dd0e00e30e60e90ec0ef0f20f50f80fb0fe10110410710b10e11111411811b11e12212512912c13013313713a13e14114514814c15015315715b15f16216616a16e17217617a17d18118518918d19119519a19e1a21a61aa1ae1b21b71bb1bf1c31c81cc1d01d51d91dd1e21e61eb1ef1f31f81fc20120520a20f21321821c22122622a22f23323823d24124624b25025425925e26326726c27127627b28028428928e29329829d2a22a62ab2b02b52ba2bf2c42c92ce2d32d82dc2e12e62eb2f02f52fa2ff30430930e31331831d32232632b33033533a33f34434934e35335735c36136636b37037437937e38338838c39139639b39f3a43a93ad3b23b73bb3c03c53c93ce3d23d73dc3e03e53e93ed3f23f63fb3ff40340840c41041541941d42142542a42e43243643a43e44244644a44e45245545945d46146546846c47047347747a47e48148548848c48f49249649949c49f4a24a64a94ac4af4b24b54b74ba4bd4c04c34c54c84cb4cd4d04d24d54d74d94dc4de4e04e34e54e74e94eb4ed4ef4f14f34f54f64f84fa4fb4fd4ff50050250350450650750850a50b50c50d50e50f510511511512513514514515516516517517517518518518518518519519";
const fn hex(b: u8) -> i32 {
    if b >= b'a' {
        (b - b'a' + 10) as i32
    } else {
        (b - b'0') as i32
    }
}
const fn make_gauss() -> [i32; 512] {
    let b = GAUSS_HEX.as_bytes();
    let mut a = [0; 512];
    let mut i = 0;
    while i < 512 {
        a[i] = (hex(b[i * 3]) << 8) | (hex(b[i * 3 + 1]) << 4) | hex(b[i * 3 + 2]);
        i += 1;
    }
    a
}
const GAUSS: [i32; 512] = make_gauss();

pub const SAMPLE_FRAMES: usize = 534;
const ENDX: usize = 0x7c;
pub const DSP_SAVE_BLOCK_LEN: usize = 3_024;

/// Register-write capture used by the translated SPC music player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DspRegWriteHistory {
    pub count: u32,
    pub addr: [u8; 256],
    pub val: [u8; 256],
}
impl Default for DspRegWriteHistory {
    fn default() -> Self {
        Self {
            count: 0,
            addr: [0; 256],
            val: [0; 256],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DspChannel {
    pub pitch: u16,
    pub pitch_counter: u16,
    pub pitch_modulation: bool,
    pub decode_buffer: [i16; 19],
    pub srcn: u8,
    pub decode_offset: u16,
    pub previous_flags: u8,
    pub old: i16,
    pub older: i16,
    pub use_noise: bool,
    pub adsr_rates: [u16; 4],
    pub rate_counter: u16,
    pub adsr_state: u8,
    pub sustain_level: u16,
    pub use_gain: bool,
    pub gain_mode: u8,
    pub direct_gain: bool,
    pub gain_value: u16,
    pub gain: u16,
    pub key_on: bool,
    pub key_off: bool,
    pub sample_out: i16,
    pub volume_l: i8,
    pub volume_r: i8,
    pub echo_enable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dsp {
    pub ram: [u8; 0x80],
    pub channels: [DspChannel; 8],
    pub dir_page: u16,
    pub even_cycle: bool,
    pub mute: bool,
    pub reset_flag: bool,
    pub master_volume_l: i8,
    pub master_volume_r: i8,
    pub noise_sample: i16,
    pub noise_rate: u16,
    pub noise_counter: u16,
    pub echo_writes: bool,
    pub echo_volume_l: i8,
    pub echo_volume_r: i8,
    pub feedback_volume: i8,
    pub echo_buffer_address: u16,
    pub echo_delay: u16,
    pub echo_remain: u16,
    pub echo_buffer_index: u16,
    pub fir_buffer_index: u8,
    pub fir_values: [i8; 8],
    pub fir_buffer_l: [i16; 8],
    pub fir_buffer_r: [i16; 8],
    pub sample_buffer: [i16; SAMPLE_FRAMES * 2],
    pub sample_offset: u16,
}
impl Default for Dsp {
    fn default() -> Self {
        let mut ram = [0; 0x80];
        ram[ENDX] = 0xff;
        Self {
            ram,
            channels: core::array::from_fn(|_| DspChannel::default()),
            dir_page: 0,
            even_cycle: false,
            mute: true,
            reset_flag: true,
            master_volume_l: 0,
            master_volume_r: 0,
            noise_sample: -0x4000,
            noise_rate: 0,
            noise_counter: 0,
            echo_writes: false,
            echo_volume_l: 0,
            echo_volume_r: 0,
            feedback_volume: 0,
            echo_buffer_address: 0,
            echo_delay: 1,
            echo_remain: 1,
            echo_buffer_index: 0,
            fir_buffer_index: 0,
            fir_values: [0; 8],
            fir_buffer_l: [0; 8],
            fir_buffer_r: [0; 8],
            sample_buffer: [0; SAMPLE_FRAMES * 2],
            sample_offset: 0,
        }
    }
}

impl Dsp {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn reset(&mut self) {
        *self = Self::default()
    }

    /// Transfers the exact C `Dsp` range beginning at `ram` and ending at the
    /// structure boundary. C ABI padding is represented by bytes in `block`.
    pub fn saveload(
        &mut self,
        io: &mut impl crate::saveload::SaveLoad,
    ) -> Result<(), crate::saveload::SaveLoadError> {
        let mut block = [0; DSP_SAVE_BLOCK_LEN];
        if !io.is_loading() {
            self.write_save_block(&mut block)?;
        }
        io.transfer(&mut block)?;
        if io.is_loading() {
            self.read_save_block(&block)?;
        }
        Ok(())
    }

    pub fn write_save_block(&self, block: &mut [u8]) -> Result<(), crate::saveload::SaveLoadError> {
        use crate::saveload::{SaveLoadError, bool_byte, put_i16, put_u16};
        if block.len() != DSP_SAVE_BLOCK_LEN {
            return Err(SaveLoadError::WrongBlockSize {
                expected: DSP_SAVE_BLOCK_LEN,
                actual: block.len(),
            });
        }
        block[..128].copy_from_slice(&self.ram);
        for (index, channel) in self.channels.iter().enumerate() {
            let at = 128 + index * 86;
            put_u16(block, at, channel.pitch);
            put_u16(block, at + 2, channel.pitch_counter);
            block[at + 4] = bool_byte(channel.pitch_modulation);
            for (sample, &value) in channel.decode_buffer.iter().enumerate() {
                put_i16(block, at + 6 + sample * 2, value);
            }
            block[at + 44] = channel.srcn;
            put_u16(block, at + 46, channel.decode_offset);
            block[at + 48] = channel.previous_flags;
            put_i16(block, at + 50, channel.old);
            put_i16(block, at + 52, channel.older);
            block[at + 54] = bool_byte(channel.use_noise);
            for (rate, &value) in channel.adsr_rates.iter().enumerate() {
                put_u16(block, at + 56 + rate * 2, value);
            }
            put_u16(block, at + 64, channel.rate_counter);
            block[at + 66] = channel.adsr_state;
            put_u16(block, at + 68, channel.sustain_level);
            block[at + 70] = bool_byte(channel.use_gain);
            block[at + 71] = channel.gain_mode;
            block[at + 72] = bool_byte(channel.direct_gain);
            put_u16(block, at + 74, channel.gain_value);
            put_u16(block, at + 76, channel.gain);
            block[at + 78] = bool_byte(channel.key_on);
            block[at + 79] = bool_byte(channel.key_off);
            put_i16(block, at + 80, channel.sample_out);
            block[at + 82] = channel.volume_l as u8;
            block[at + 83] = channel.volume_r as u8;
            block[at + 84] = bool_byte(channel.echo_enable);
        }
        put_u16(block, 816, self.dir_page);
        block[818] = bool_byte(self.even_cycle);
        block[819] = bool_byte(self.mute);
        block[820] = bool_byte(self.reset_flag);
        block[821] = self.master_volume_l as u8;
        block[822] = self.master_volume_r as u8;
        put_i16(block, 824, self.noise_sample);
        put_u16(block, 826, self.noise_rate);
        put_u16(block, 828, self.noise_counter);
        block[830] = bool_byte(self.echo_writes);
        block[831] = self.echo_volume_l as u8;
        block[832] = self.echo_volume_r as u8;
        block[833] = self.feedback_volume as u8;
        put_u16(block, 834, self.echo_buffer_address);
        put_u16(block, 836, self.echo_delay);
        put_u16(block, 838, self.echo_remain);
        put_u16(block, 840, self.echo_buffer_index);
        block[842] = self.fir_buffer_index;
        for (index, &value) in self.fir_values.iter().enumerate() {
            block[843 + index] = value as u8;
        }
        for (index, &value) in self.fir_buffer_l.iter().enumerate() {
            put_i16(block, 852 + index * 2, value);
        }
        for (index, &value) in self.fir_buffer_r.iter().enumerate() {
            put_i16(block, 868 + index * 2, value);
        }
        for (index, &value) in self.sample_buffer.iter().enumerate() {
            put_i16(block, 884 + index * 2, value);
        }
        put_u16(block, 3020, self.sample_offset);
        Ok(())
    }

    pub fn read_save_block(&mut self, block: &[u8]) -> Result<(), crate::saveload::SaveLoadError> {
        use crate::saveload::{SaveLoadError, get_i16, get_u16};
        if block.len() != DSP_SAVE_BLOCK_LEN {
            return Err(SaveLoadError::WrongBlockSize {
                expected: DSP_SAVE_BLOCK_LEN,
                actual: block.len(),
            });
        }
        self.ram.copy_from_slice(&block[..128]);
        for (index, channel) in self.channels.iter_mut().enumerate() {
            let at = 128 + index * 86;
            channel.pitch = get_u16(block, at);
            channel.pitch_counter = get_u16(block, at + 2);
            channel.pitch_modulation = block[at + 4] != 0;
            for (sample, value) in channel.decode_buffer.iter_mut().enumerate() {
                *value = get_i16(block, at + 6 + sample * 2);
            }
            channel.srcn = block[at + 44];
            channel.decode_offset = get_u16(block, at + 46);
            channel.previous_flags = block[at + 48];
            channel.old = get_i16(block, at + 50);
            channel.older = get_i16(block, at + 52);
            channel.use_noise = block[at + 54] != 0;
            for (rate, value) in channel.adsr_rates.iter_mut().enumerate() {
                *value = get_u16(block, at + 56 + rate * 2);
            }
            channel.rate_counter = get_u16(block, at + 64);
            channel.adsr_state = block[at + 66];
            channel.sustain_level = get_u16(block, at + 68);
            channel.use_gain = block[at + 70] != 0;
            channel.gain_mode = block[at + 71];
            channel.direct_gain = block[at + 72] != 0;
            channel.gain_value = get_u16(block, at + 74);
            channel.gain = get_u16(block, at + 76);
            channel.key_on = block[at + 78] != 0;
            channel.key_off = block[at + 79] != 0;
            channel.sample_out = get_i16(block, at + 80);
            channel.volume_l = block[at + 82] as i8;
            channel.volume_r = block[at + 83] as i8;
            channel.echo_enable = block[at + 84] != 0;
        }
        self.dir_page = get_u16(block, 816);
        self.even_cycle = block[818] != 0;
        self.mute = block[819] != 0;
        self.reset_flag = block[820] != 0;
        self.master_volume_l = block[821] as i8;
        self.master_volume_r = block[822] as i8;
        self.noise_sample = get_i16(block, 824);
        self.noise_rate = get_u16(block, 826);
        self.noise_counter = get_u16(block, 828);
        self.echo_writes = block[830] != 0;
        self.echo_volume_l = block[831] as i8;
        self.echo_volume_r = block[832] as i8;
        self.feedback_volume = block[833] as i8;
        self.echo_buffer_address = get_u16(block, 834);
        self.echo_delay = get_u16(block, 836);
        self.echo_remain = get_u16(block, 838);
        self.echo_buffer_index = get_u16(block, 840);
        self.fir_buffer_index = block[842];
        for (index, value) in self.fir_values.iter_mut().enumerate() {
            *value = block[843 + index] as i8;
        }
        for (index, value) in self.fir_buffer_l.iter_mut().enumerate() {
            *value = get_i16(block, 852 + index * 2);
        }
        for (index, value) in self.fir_buffer_r.iter_mut().enumerate() {
            *value = get_i16(block, 868 + index * 2);
        }
        for (index, value) in self.sample_buffer.iter_mut().enumerate() {
            *value = get_i16(block, 884 + index * 2);
        }
        self.sample_offset = get_u16(block, 3020);
        Ok(())
    }
    #[must_use]
    pub fn read(&self, a: u8) -> u8 {
        self.ram[a as usize]
    }
    pub fn write(&mut self, m: &mut [u8; 65536], a: u8, mut v: u8) {
        let c = (a >> 4) as usize;
        match a {
            0x0c => self.master_volume_l = v as i8,
            0x1c => self.master_volume_r = v as i8,
            0x2c => self.echo_volume_l = v as i8,
            0x3c => self.echo_volume_r = v as i8,
            0x4c => {
                for i in 0..8 {
                    self.channels[i].key_on = v & (1 << i) != 0;
                    if self.channels[i].key_on {
                        self.key_on(m, i)
                    }
                }
            }
            0x5c => {
                for i in 0..8 {
                    self.channels[i].key_off = v & (1 << i) != 0;
                    if self.channels[i].key_off {
                        self.channels[i].adsr_state = 4
                    }
                }
            }
            0x6c => {
                self.reset_flag = v & 0x80 != 0;
                self.mute = v & 0x40 != 0;
                self.echo_writes = v & 0x20 == 0;
                self.noise_rate = RATE[(v & 31) as usize]
            }
            0x7c => v = 0,
            0x0d => self.feedback_volume = v as i8,
            0x2d => {
                for i in 0..8 {
                    self.channels[i].pitch_modulation = v & (1 << i) != 0
                }
            }
            0x3d => {
                for i in 0..8 {
                    self.channels[i].use_noise = v & (1 << i) != 0
                }
            }
            0x4d => {
                for i in 0..8 {
                    self.channels[i].echo_enable = v & (1 << i) != 0
                }
            }
            0x5d => self.dir_page = (v as u16) << 8,
            0x6d => self.echo_buffer_address = (v as u16) << 8,
            0x7d => {
                self.echo_delay = ((v & 15) as u16) * 512;
                if self.echo_delay == 0 {
                    self.echo_delay = 1
                }
            }
            _ if a & 15 == 15 => self.fir_values[c] = v as i8,
            _ if a & 15 == 0 => self.channels[c].volume_l = v as i8,
            _ if a & 15 == 1 => self.channels[c].volume_r = v as i8,
            _ if a & 15 == 2 => {
                self.channels[c].pitch = (self.channels[c].pitch & 0x3f00) | v as u16
            }
            _ if a & 15 == 3 => {
                self.channels[c].pitch =
                    ((self.channels[c].pitch & 255) | ((v as u16) << 8)) & 0x3fff
            }
            _ if a & 15 == 4 => self.channels[c].srcn = v,
            _ if a & 15 == 5 => {
                self.channels[c].adsr_rates[0] = RATE[((v & 15) * 2 + 1) as usize];
                self.channels[c].adsr_rates[1] = RATE[(((v & 0x70) >> 4) * 2 + 16) as usize];
                self.channels[c].use_gain = v & 0x80 == 0
            }
            _ if a & 15 == 6 => {
                self.channels[c].adsr_rates[2] = RATE[(v & 31) as usize];
                self.channels[c].sustain_level = (((v & 0xe0) >> 5) as u16 + 1) * 256
            }
            _ if a & 15 == 7 => {
                self.channels[c].direct_gain = v & 0x80 == 0;
                if v & 0x80 != 0 {
                    self.channels[c].gain_mode = (v & 0x60) >> 5;
                    self.channels[c].adsr_rates[3] = RATE[(v & 31) as usize]
                } else {
                    self.channels[c].gain_value = ((v & 0x7f) as u16) * 16
                }
            }
            _ => {}
        }
        self.ram[a as usize] = v
    }
    fn key_on(&mut self, m: &[u8; 65536], c: usize) {
        let s = &mut self.channels[c];
        s.key_on = false;
        s.previous_flags = 0;
        let p = self.dir_page.wrapping_add(4 * s.srcn as u16);
        s.decode_offset = rd(m, p);
        s.decode_buffer = [0; 19];
        s.gain = 0;
        s.adsr_state = if s.use_gain { 3 } else { 0 }
    }
    pub fn cycle(&mut self, m: &mut [u8; 65536]) {
        let (mut l, mut r) = (0, 0);
        for c in 0..8 {
            self.cycle_channel(m, c);
            let s = &self.channels[c];
            l = clamp(l + ((s.sample_out as i32 * s.volume_l as i32) >> 6));
            r = clamp(r + ((s.sample_out as i32 * s.volume_r as i32) >> 6))
        }
        l = clamp((l * self.master_volume_l as i32) >> 7);
        r = clamp((r * self.master_volume_r as i32) >> 7);
        self.echo(m, &mut l, &mut r);
        if self.mute {
            l = 0;
            r = 0
        }
        self.noise();
        if self.sample_offset < 534 {
            let i = self.sample_offset as usize * 2;
            self.sample_buffer[i] = l as i16;
            self.sample_buffer[i + 1] = r as i16;
            self.sample_offset += 1
        }
        self.even_cycle = !self.even_cycle
    }
    fn cycle_channel(&mut self, m: &[u8; 65536], c: usize) {
        let mut p = self.channels[c].pitch;
        if c > 0 && self.channels[c].pitch_modulation {
            let f = (self.channels[c - 1].sample_out as i32 >> 4) + 0x400;
            // The C assignment converts the possibly-negative product to
            // uint16_t before applying the upper clamp.
            p = ((p as i32 * f) >> 10) as u16;
            if p > 0x3fff {
                p = 0x3fff;
            }
        }
        let n = self.channels[c].pitch_counter as u32 + p as u32;
        if n > 0xffff {
            self.brr(m, c)
        }
        self.channels[c].pitch_counter = n as u16;
        let s = if self.channels[c].use_noise {
            self.noise_sample
        } else {
            self.sample(
                c,
                (self.channels[c].pitch_counter >> 12) as usize,
                ((self.channels[c].pitch_counter >> 4) & 255) as usize,
            )
        };
        if self.reset_flag {
            self.channels[c].adsr_state = 4;
            self.channels[c].gain = 0
        }
        let st = self.channels[c].adsr_state;
        let direct = st != 4 && self.channels[c].use_gain && self.channels[c].direct_gain;
        let rate = if st == 4 {
            0
        } else {
            self.channels[c].adsr_rates[st as usize]
        };
        if st != 4 && !direct && rate != 0 {
            self.channels[c].rate_counter = self.channels[c].rate_counter.wrapping_add(1)
        }
        if st == 4 || (!direct && rate != 0 && self.channels[c].rate_counter >= rate) {
            if st != 4 {
                self.channels[c].rate_counter = 0
            }
            self.gain(c)
        }
        if direct {
            self.channels[c].gain = self.channels[c].gain_value
        }
        self.ram[c * 16 + 8] = (self.channels[c].gain >> 4) as u8;
        let o = ((s as i32 * self.channels[c].gain as i32) >> 11) as i16;
        self.ram[c * 16 + 9] = (o >> 7) as u8;
        self.channels[c].sample_out = o
    }
    fn gain(&mut self, c: usize) {
        let s = &mut self.channels[c];
        match s.adsr_state {
            0 => {
                s.gain = s
                    .gain
                    .wrapping_add(if s.adsr_rates[0] == 1 { 1024 } else { 32 });
                if s.gain >= 0x7e0 {
                    s.adsr_state = 1
                }
                if s.gain > 0x7ff {
                    s.gain = 0x7ff
                }
            }
            1 | 2 => {
                // `uint16_t` promotes to signed `int` here in the C source.
                // In particular, a zero gain produces a zero decrement.
                let decrement = (((i32::from(s.gain) - 1) >> 8) + 1) as u16;
                s.gain = s.gain.wrapping_sub(decrement);
                if s.adsr_state == 1 && s.gain < s.sustain_level {
                    s.adsr_state = 2
                }
            }
            3 => match s.gain_mode {
                0 => {
                    s.gain = s.gain.wrapping_sub(32);
                    if s.gain > 0x7ff {
                        s.gain = 0
                    }
                }
                1 => {
                    let decrement = (((i32::from(s.gain) - 1) >> 8) + 1) as u16;
                    s.gain = s.gain.wrapping_sub(decrement)
                }
                2 => {
                    s.gain = s.gain.wrapping_add(32);
                    if s.gain > 0x7ff {
                        s.gain = 0
                    }
                }
                3 => {
                    s.gain = s.gain.wrapping_add(if s.gain < 0x600 { 32 } else { 8 });
                    if s.gain > 0x7ff {
                        s.gain = 0
                    }
                }
                _ => unreachable!(),
            },
            4 => {
                s.gain = s.gain.wrapping_sub(8);
                if s.gain > 0x7ff {
                    s.gain = 0
                }
            }
            _ => unreachable!(),
        }
    }
    fn sample(&self, c: usize, n: usize, o: usize) -> i16 {
        let b = &self.channels[c].decode_buffer;
        let mut x = (GAUSS[255 - o] * b[n] as i32) >> 10;
        x += (GAUSS[511 - o] * b[n + 1] as i32) >> 10;
        x += (GAUSS[256 + o] * b[n + 2] as i32) >> 10;
        x = x as i16 as i32;
        x += (GAUSS[o] * b[n + 3] as i32) >> 10;
        (clamp(x) >> 1) as i16
    }
    fn brr(&mut self, m: &[u8; 65536], c: usize) {
        let s = &mut self.channels[c];
        s.decode_buffer.copy_within(16..19, 0);
        if s.previous_flags == 1 || s.previous_flags == 3 {
            let p = self.dir_page.wrapping_add(4 * s.srcn as u16);
            s.decode_offset = rd(m, p.wrapping_add(2));
            if s.previous_flags == 1 {
                s.adsr_state = 4;
                s.gain = 0
            }
            self.ram[ENDX] |= 1 << c
        }
        let h = m[s.decode_offset as usize];
        s.decode_offset = s.decode_offset.wrapping_add(1);
        let (sh, f) = (h >> 4, (h & 12) >> 2);
        s.previous_flags = h & 3;
        let (mut byte, mut old, mut older) = (0, s.old as i32, s.older as i32);
        for i in 0..16 {
            let mut x = if i & 1 != 0 {
                (byte & 15) as i32
            } else {
                byte = m[s.decode_offset as usize];
                s.decode_offset = s.decode_offset.wrapping_add(1);
                (byte >> 4) as i32
            };
            if x > 7 {
                x -= 16
            }
            x = if sh <= 12 { x << sh >> 1 } else { x >> 3 << 12 };
            x += match f {
                0 => 0,
                1 => old + (-old >> 4),
                2 => 2 * old + ((-3 * old) >> 5) - older + (older >> 4),
                3 => 2 * old + ((-13 * old) >> 6) - older + ((3 * older) >> 4),
                _ => unreachable!(),
            };
            x = clamp(x);
            x = (((x & 0x7fff) << 1) as i16 >> 1) as i32;
            older = old;
            old = x;
            s.decode_buffer[i + 3] = x as i16
        }
        s.older = older as i16;
        s.old = old as i16
    }
    fn noise(&mut self) {
        if self.noise_rate != 0 {
            self.noise_counter = self.noise_counter.wrapping_add(1)
        }
        if self.noise_rate != 0 && self.noise_counter >= self.noise_rate {
            let bit = (self.noise_sample & 1) ^ ((self.noise_sample >> 1) & 1);
            let x = ((self.noise_sample >> 1) & 0x3fff) | (bit << 14);
            self.noise_sample = (((x as i32 & 0x7fff) << 1) as i16) >> 1;
            self.noise_counter = 0
        }
    }
    fn echo(&mut self, m: &mut [u8; 65536], l: &mut i32, r: &mut i32) {
        let a = self
            .echo_buffer_address
            .wrapping_add(self.echo_buffer_index.wrapping_mul(4));
        let fi = self.fir_buffer_index as usize;
        self.fir_buffer_l[fi] = (rd(m, a) as i16) >> 1;
        self.fir_buffer_r[fi] = (rd(m, a.wrapping_add(2)) as i16) >> 1;
        let (mut sl, mut sr) = (0, 0);
        for i in 0..8 {
            let j = (fi + i + 1) & 7;
            sl += (self.fir_buffer_l[j] as i32 * self.fir_values[i] as i32) >> 6;
            sr += (self.fir_buffer_r[j] as i32 * self.fir_values[i] as i32) >> 6;
            if i == 6 {
                sl = sl as i16 as i32;
                sr = sr as i16 as i32
            }
        }
        sl = clamp(sl);
        sr = clamp(sr);
        *l = clamp(*l + ((sl * self.echo_volume_l as i32) >> 7));
        *r = clamp(*r + ((sr * self.echo_volume_r as i32) >> 7));
        let (mut il, mut ir) = (0, 0);
        for s in &self.channels {
            if s.echo_enable {
                il = clamp(il + ((s.sample_out as i32 * s.volume_l as i32) >> 6));
                ir = clamp(ir + ((s.sample_out as i32 * s.volume_r as i32) >> 6))
            }
        }
        il = clamp(il + ((sl * self.feedback_volume as i32) >> 7)) & !1;
        ir = clamp(ir + ((sr * self.feedback_volume as i32) >> 7)) & !1;
        if self.echo_writes {
            wr(m, a, il as u16);
            wr(m, a.wrapping_add(2), ir as u16)
        }
        self.fir_buffer_index = self.fir_buffer_index.wrapping_add(1) & 7;
        self.echo_buffer_index = self.echo_buffer_index.wrapping_add(1);
        self.echo_remain = self.echo_remain.wrapping_sub(1);
        if self.echo_remain == 0 {
            self.echo_remain = self.echo_delay;
            self.echo_buffer_index = 0
        }
    }
    pub fn get_samples(&mut self, out: &mut [i16], count: usize, channels: usize) {
        let output_channels = if channels == 1 { 1 } else { 2 };
        assert!(out.len() >= count * output_channels);
        let add = 534.0f32 / count as f32;
        let mut loc = 0.0f32;
        for i in 0..count {
            let j = loc as usize * 2;
            let (l, r) = (self.sample_buffer[j], self.sample_buffer[j + 1]);
            if channels == 1 {
                out[i] = ((l as i32 + r as i32) >> 1) as i16
            } else {
                out[i * 2] = l;
                out[i * 2 + 1] = r
            }
            loc += add
        }
        self.sample_offset = 0
    }
}
fn clamp(x: i32) -> i32 {
    x.clamp(-0x8000, 0x7fff)
}
fn rd(m: &[u8; 65536], a: u16) -> u16 {
    u16::from_le_bytes([m[a as usize], m[a.wrapping_add(1) as usize]])
}
fn wr(m: &mut [u8; 65536], a: u16, v: u16) {
    let b = v.to_le_bytes();
    m[a as usize] = b[0];
    m[a.wrapping_add(1) as usize] = b[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reset_state() {
        let d = Dsp::new();
        assert_eq!(d.ram[ENDX], 255);
        assert!(d.mute && d.reset_flag);
        assert_eq!(d.noise_sample, -0x4000);
        assert_eq!((d.echo_delay, d.echo_remain), (1, 1))
    }
    #[test]
    fn registers() {
        let (mut d, mut m) = (Dsp::new(), [0; 65536]);
        d.write(&mut m, 0x22, 0xcd);
        d.write(&mut m, 0x23, 0xff);
        assert_eq!(d.channels[2].pitch, 0x3fcd);
        d.write(&mut m, 0x25, 0xff);
        d.write(&mut m, 0x26, 0xe1);
        assert_eq!(d.channels[2].adsr_rates, [1, 2, 2048, 0]);
        assert_eq!(d.channels[2].sustain_level, 0x800);
        d.write(&mut m, 0x7c, 9);
        assert_eq!(d.read(0x7c), 0)
    }
    #[test]
    fn key_on_wraps_directory() {
        let (mut d, mut m) = (Dsp::new(), [0; 65536]);
        d.write(&mut m, 0x5d, 0xff);
        d.write(&mut m, 0x74, 0x40);
        m[0] = 0x34;
        m[1] = 0x12;
        d.write(&mut m, 0x4c, 0x80);
        assert_eq!(d.channels[7].decode_offset, 0x1234)
    }
    #[test]
    fn resampling() {
        let mut d = Dsp::new();
        for i in 0..534 {
            d.sample_buffer[i * 2] = i as i16;
            d.sample_buffer[i * 2 + 1] = -(i as i16)
        }
        let mut o = [0; 8];
        d.get_samples(&mut o, 4, 2);
        assert_eq!(o, [0, 0, 133, -133, 267, -267, 400, -400])
    }
    #[test]
    fn exponential_gain_at_zero_matches_c_integer_promotion() {
        let mut d = Dsp::new();
        d.channels[0].adsr_state = 2;
        d.channels[0].gain = 0;
        d.gain(0);
        assert_eq!(d.channels[0].gain, 0);
        d.channels[0].adsr_state = 3;
        d.channels[0].gain_mode = 1;
        d.gain(0);
        assert_eq!(d.channels[0].gain, 0);
    }
}
