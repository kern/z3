//! SPC700/DSP integration and memory-mapped APU I/O from `snes/apu.c`.

use crate::bus::ApuBus;
use crate::dsp::Dsp;
use crate::saveload::{SaveLoad, SaveLoadError, bool_byte, get_u32, put_u32};
use crate::spc::Spc;

pub const APU_CORE_SAVE_BLOCK_LEN: usize = 65_576;

const BOOT_ROM: [u8; 0x40] = [
    0xcd, 0xef, 0xbd, 0xe8, 0x00, 0xc6, 0x1d, 0xd0, 0xfc, 0x8f, 0xaa, 0xf4, 0x8f, 0xbb, 0xf5, 0x78,
    0xcc, 0xf4, 0xd0, 0xfb, 0x2f, 0x19, 0xeb, 0xf4, 0xd0, 0xfc, 0x7e, 0xf4, 0xd0, 0x0b, 0xe4, 0xf5,
    0xcb, 0xf4, 0xd7, 0x00, 0xfc, 0xd0, 0xf3, 0xab, 0x01, 0x10, 0xef, 0x7e, 0xf4, 0x10, 0xeb, 0xba,
    0xf6, 0xda, 0x00, 0xba, 0xf4, 0xc4, 0xf4, 0xdd, 0x5d, 0xd0, 0xdb, 0x1f, 0x00, 0x00, 0xc0, 0xff,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Timer {
    pub cycles: u8,
    pub divider: u8,
    pub target: u8,
    pub counter: u8,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DspWriteHistory {
    pub count: usize,
    pub addresses: [u8; 256],
    pub values: [u8; 256],
}

impl Default for DspWriteHistory {
    fn default() -> Self {
        Self {
            count: 0,
            addresses: [0; 256],
            values: [0; 256],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Apu {
    pub spc: Spc,
    pub dsp: Dsp,
    pub ram: Box<[u8; 0x1_0000]>,
    pub rom_readable: bool,
    pub dsp_address: u8,
    pub cycles: u32,
    pub input_ports: [u8; 6],
    pub output_ports: [u8; 4],
    pub timers: [Timer; 3],
    pub cpu_cycles_left: u8,
    pub history: DspWriteHistory,
}

impl Default for Apu {
    fn default() -> Self {
        let mut apu = Self {
            spc: Spc::new(),
            dsp: Dsp::new(),
            ram: Box::new([0; 0x1_0000]),
            rom_readable: false,
            dsp_address: 0,
            cycles: 0,
            input_ports: [0; 6],
            output_ports: [0; 4],
            timers: [Timer::default(); 3],
            cpu_cycles_left: 0,
            history: DspWriteHistory::default(),
        };
        apu.reset();
        apu
    }
}

impl Apu {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Matches `apu_saveload`: APU RAM/core bytes, then DSP, then SPC.
    pub fn saveload(&mut self, io: &mut impl SaveLoad) -> Result<(), SaveLoadError> {
        let mut block = vec![0; APU_CORE_SAVE_BLOCK_LEN];
        if !io.is_loading() {
            block[..0x1_0000].copy_from_slice(self.ram.as_slice());
            block[65_536] = bool_byte(self.rom_readable);
            block[65_537] = self.dsp_address;
            put_u32(&mut block, 65_540, self.cycles);
            block[65_544..65_550].copy_from_slice(&self.input_ports);
            block[65_550..65_554].copy_from_slice(&self.output_ports);
            for (index, timer) in self.timers.iter().enumerate() {
                let at = 65_554 + index * 5;
                block[at] = timer.cycles;
                block[at + 1] = timer.divider;
                block[at + 2] = timer.target;
                block[at + 3] = timer.counter;
                block[at + 4] = bool_byte(timer.enabled);
            }
            block[65_569] = self.cpu_cycles_left;
        }
        io.transfer(&mut block)?;
        if io.is_loading() {
            self.ram.copy_from_slice(&block[..0x1_0000]);
            self.rom_readable = block[65_536] != 0;
            self.dsp_address = block[65_537];
            self.cycles = get_u32(&block, 65_540);
            self.input_ports.copy_from_slice(&block[65_544..65_550]);
            self.output_ports.copy_from_slice(&block[65_550..65_554]);
            for (index, timer) in self.timers.iter_mut().enumerate() {
                let at = 65_554 + index * 5;
                timer.cycles = block[at];
                timer.divider = block[at + 1];
                timer.target = block[at + 2];
                timer.counter = block[at + 3];
                timer.enabled = block[at + 4] != 0;
            }
            self.cpu_cycles_left = block[65_569];
        }
        self.dsp.saveload(io)?;
        self.spc.saveload(io)
    }

    pub fn reset(&mut self) {
        // This must precede the SPC reset: its reset vector is in the boot ROM.
        self.rom_readable = true;
        let mut spc = core::mem::take(&mut self.spc);
        spc.reset(self);
        self.spc = spc;
        self.dsp.reset();
        self.ram.fill(0);
        self.dsp_address = 0;
        self.cycles = 0;
        self.input_ports.fill(0);
        self.output_ports.fill(0);
        self.timers.fill(Timer::default());
        self.cpu_cycles_left = 7;
        self.history.count = 0;
    }

    pub fn cycle(&mut self) {
        if self.cpu_cycles_left == 0 {
            let mut spc = core::mem::take(&mut self.spc);
            self.cpu_cycles_left = spc.run_opcode(self);
            self.spc = spc;
        }
        self.cpu_cycles_left = self.cpu_cycles_left.wrapping_sub(1);

        if self.cycles & 0x1f == 0 {
            self.dsp.cycle(&mut self.ram);
        }

        for (index, timer) in self.timers.iter_mut().enumerate() {
            if timer.cycles == 0 {
                timer.cycles = if index == 2 { 16 } else { 128 };
                if timer.enabled {
                    timer.divider = timer.divider.wrapping_add(1);
                    if timer.divider == timer.target {
                        timer.divider = 0;
                        timer.counter = timer.counter.wrapping_add(1) & 0x0f;
                    }
                }
            }
            timer.cycles = timer.cycles.wrapping_sub(1);
        }
        self.cycles = self.cycles.wrapping_add(1);
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0xf0 | 0xf1 | 0xfa..=0xfc => 0,
            0xf2 => self.dsp_address,
            0xf3 => self.dsp.read(self.dsp_address & 0x7f),
            0xf4..=0xf9 => self.input_ports[usize::from(address - 0xf4)],
            0xfd..=0xff => {
                let timer = &mut self.timers[usize::from(address - 0xfd)];
                let value = timer.counter;
                timer.counter = 0;
                value
            }
            0xffc0..=0xffff if self.rom_readable => BOOT_ROM[usize::from(address - 0xffc0)],
            _ => self.ram[usize::from(address)],
        }
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            0xf0 => {}
            0xf1 => {
                for (index, timer) in self.timers.iter_mut().enumerate() {
                    let enabled = value & (1 << index) != 0;
                    if !timer.enabled && enabled {
                        timer.divider = 0;
                        timer.counter = 0;
                    }
                    timer.enabled = enabled;
                }
                if value & 0x10 != 0 {
                    self.input_ports[0..2].fill(0);
                }
                if value & 0x20 != 0 {
                    self.input_ports[2..4].fill(0);
                }
                self.rom_readable = value & 0x80 != 0;
            }
            0xf2 => self.dsp_address = value,
            0xf3 => {
                if self.history.count != 256 {
                    let index = self.history.count;
                    self.history.count += 1;
                    self.history.addresses[index] = self.dsp_address;
                    self.history.values[index] = value;
                }
                if self.dsp_address < 0x80 {
                    self.dsp.write(&mut self.ram, self.dsp_address, value);
                }
            }
            0xf4..=0xf7 => self.output_ports[usize::from(address - 0xf4)] = value,
            0xf8..=0xf9 => self.input_ports[usize::from(address - 0xf4)] = value,
            0xfa..=0xfc => self.timers[usize::from(address - 0xfa)].target = value,
            _ => {}
        }
        // The C implementation mirrors every I/O write into APU RAM.
        self.ram[usize::from(address)] = value;
    }
}

impl ApuBus for Apu {
    fn read(&mut self, address: u16) -> u8 {
        self.cpu_read(address)
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cpu_write(address, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_reads_boot_vector_before_clearing_ram() {
        let apu = Apu::new();
        assert_eq!(apu.spc.pc, 0xffc0);
        assert!(apu.rom_readable);
        assert_eq!(apu.cpu_cycles_left, 7);
        assert!(apu.ram.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn control_register_and_ports_match_c_side_effects() {
        let mut apu = Apu::new();
        apu.input_ports[..4].copy_from_slice(&[1, 2, 3, 4]);
        apu.timers[0].counter = 9;
        apu.cpu_write(0xf1, 0x91);
        assert_eq!(&apu.input_ports[..2], &[0, 0]);
        assert_eq!(&apu.input_ports[2..4], &[3, 4]);
        assert!(apu.timers[0].enabled);
        assert_eq!(apu.timers[0].counter, 0);
        assert!(apu.rom_readable);
        assert_eq!(apu.ram[0xf1], 0x91);
    }

    #[test]
    fn timer_counter_read_clears_and_wraps_at_four_bits() {
        let mut apu = Apu::new();
        apu.timers[2] = Timer {
            cycles: 0,
            divider: 0,
            target: 1,
            counter: 15,
            enabled: true,
        };
        apu.cycle();
        assert_eq!(apu.timers[2].counter, 0);
        apu.timers[2].counter = 7;
        assert_eq!(apu.cpu_read(0xff), 7);
        assert_eq!(apu.cpu_read(0xff), 0);
    }

    #[test]
    fn dsp_writes_are_bounded_and_recorded() {
        let mut apu = Apu::new();
        apu.cpu_write(0xf2, 0x0c);
        apu.cpu_write(0xf3, 0x55);
        assert_eq!(apu.history.count, 1);
        assert_eq!(apu.history.addresses[0], 0x0c);
        assert_eq!(apu.history.values[0], 0x55);
        assert_eq!(apu.dsp.read(0x0c), 0x55);

        apu.cpu_write(0xf2, 0x80);
        apu.cpu_write(0xf3, 0xaa);
        assert_eq!(apu.history.count, 2);
    }
}
