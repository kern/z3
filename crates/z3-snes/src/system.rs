//! SNES device integration, address decoding, and ROM loading.
//!
//! This is a direct port of `snes/snes.c` and `snes/snes_other.c`. Device
//! ownership is safe and non-self-referential: the CPU and DMA are temporarily
//! moved out while they call back through the system bus.

use core::fmt;

use crate::apu::Apu;
use crate::bus::Bus;
use crate::cart::{CART_HIROM, CART_LOROM, Cart, CartLoadError};
use crate::cpu::Cpu;
use crate::dma::Dma;
use crate::input::Input;
use crate::ppu::Ppu;
use crate::saveload::{SaveLoad, SaveLoadError, bool_byte, get_u16, get_u32, put_u16, put_u32};

pub const SNES_CORE_SAVE_BLOCK_LEN: usize = 58;

pub const WRAM_SIZE: usize = 0x2_0000;

pub struct Snes {
    pub cpu: Cpu,
    pub apu: Apu,
    pub ppu: Ppu,
    pub dma: Dma,
    pub cart: Cart,
    pub debug_cycles: bool,
    pub disable_hpos: bool,
    pub input1: Input,
    pub input2: Input,
    pub h_pos: u16,
    pub v_pos: u16,
    pub frames: u32,
    pub cpu_cycles_left: u8,
    pub cpu_mem_ops: u8,
    pub apu_catchup_cycles: f64,
    pub h_irq_enabled: bool,
    pub v_irq_enabled: bool,
    pub nmi_enabled: bool,
    pub h_timer: u16,
    pub v_timer: u16,
    pub in_nmi: bool,
    pub in_irq: bool,
    pub in_vblank: bool,
    pub port_auto_read: [u16; 4],
    pub auto_joy_read: bool,
    pub auto_joy_timer: u16,
    pub ppu_latch: bool,
    pub multiply_a: u8,
    pub multiply_result: u16,
    pub divide_a: u16,
    pub divide_result: u16,
    pub fast_mem: bool,
    pub open_bus: u8,
    pub ram: Box<[u8; WRAM_SIZE]>,
    pub ram_address: u32,
    cpu_detached: bool,
    cpu_irq_clear_latch: bool,
}

impl Default for Snes {
    fn default() -> Self {
        Self {
            cpu: Cpu::new(),
            apu: Apu::new(),
            ppu: Ppu::new(),
            dma: Dma::new(),
            cart: Cart::new(),
            debug_cycles: false,
            disable_hpos: false,
            input1: Input::new(),
            input2: Input::new(),
            h_pos: 0,
            v_pos: 0,
            frames: 0,
            cpu_cycles_left: 0,
            cpu_mem_ops: 0,
            apu_catchup_cycles: 0.0,
            h_irq_enabled: false,
            v_irq_enabled: false,
            nmi_enabled: false,
            h_timer: 0,
            v_timer: 0,
            in_nmi: false,
            in_irq: false,
            in_vblank: false,
            port_auto_read: [0; 4],
            auto_joy_read: false,
            auto_joy_timer: 0,
            ppu_latch: false,
            multiply_a: 0,
            multiply_result: 0,
            divide_a: 0,
            divide_result: 0,
            fast_mem: false,
            open_bus: 0,
            ram: Box::new([0; WRAM_SIZE]),
            ram_address: 0,
            cpu_detached: false,
            cpu_irq_clear_latch: false,
        }
    }
}

impl Snes {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Exact device and byte-range ordering of C `snes_saveload`.
    pub fn saveload(&mut self, io: &mut impl SaveLoad) -> Result<(), SaveLoadError> {
        self.cpu.saveload(io)?;
        self.apu.saveload(io)?;
        self.dma.saveload(io)?;
        self.ppu.saveload(io)?;
        self.cart.saveload(io)?;

        let mut core = [0; SNES_CORE_SAVE_BLOCK_LEN];
        if !io.is_loading() {
            put_u16(&mut core, 0, self.h_pos);
            put_u16(&mut core, 2, self.v_pos);
            put_u32(&mut core, 4, self.frames);
            core[8] = self.cpu_cycles_left;
            core[9] = self.cpu_mem_ops;
            core[16..24].copy_from_slice(&self.apu_catchup_cycles.to_le_bytes());
            core[24] = bool_byte(self.h_irq_enabled);
            core[25] = bool_byte(self.v_irq_enabled);
            core[26] = bool_byte(self.nmi_enabled);
            put_u16(&mut core, 28, self.h_timer);
            put_u16(&mut core, 30, self.v_timer);
            core[32] = bool_byte(self.in_nmi);
            core[33] = bool_byte(self.in_irq);
            core[34] = bool_byte(self.in_vblank);
            for (index, &value) in self.port_auto_read.iter().enumerate() {
                put_u16(&mut core, 36 + index * 2, value);
            }
            core[44] = bool_byte(self.auto_joy_read);
            put_u16(&mut core, 46, self.auto_joy_timer);
            core[48] = bool_byte(self.ppu_latch);
            core[49] = self.multiply_a;
            put_u16(&mut core, 50, self.multiply_result);
            put_u16(&mut core, 52, self.divide_a);
            put_u16(&mut core, 54, self.divide_result);
            core[56] = bool_byte(self.fast_mem);
            core[57] = self.open_bus;
        }
        io.transfer(&mut core)?;
        if io.is_loading() {
            self.h_pos = get_u16(&core, 0);
            self.v_pos = get_u16(&core, 2);
            self.frames = get_u32(&core, 4);
            self.cpu_cycles_left = core[8];
            self.cpu_mem_ops = core[9];
            self.apu_catchup_cycles = f64::from_le_bytes(
                core[16..24]
                    .try_into()
                    .expect("eight-byte floating point value"),
            );
            self.h_irq_enabled = core[24] != 0;
            self.v_irq_enabled = core[25] != 0;
            self.nmi_enabled = core[26] != 0;
            self.h_timer = get_u16(&core, 28);
            self.v_timer = get_u16(&core, 30);
            self.in_nmi = core[32] != 0;
            self.in_irq = core[33] != 0;
            self.in_vblank = core[34] != 0;
            for (index, value) in self.port_auto_read.iter_mut().enumerate() {
                *value = get_u16(&core, 36 + index * 2);
            }
            self.auto_joy_read = core[44] != 0;
            self.auto_joy_timer = get_u16(&core, 46);
            self.ppu_latch = core[48] != 0;
            self.multiply_a = core[49];
            self.multiply_result = get_u16(&core, 50);
            self.divide_a = get_u16(&core, 52);
            self.divide_result = get_u16(&core, 54);
            self.fast_mem = core[56] != 0;
            self.open_bus = core[57];
        }
        io.transfer(self.ram.as_mut_slice())?;
        let mut address = self.ram_address.to_le_bytes();
        io.transfer(&mut address)?;
        if io.is_loading() {
            self.ram_address = u32::from_le_bytes(address);
        }
        self.disable_hpos = false;
        Ok(())
    }

    pub fn reset(&mut self, hard: bool) {
        self.cart.reset();
        let mut cpu = core::mem::take(&mut self.cpu);
        cpu.reset(&mut CpuBus { snes: self });
        self.cpu = cpu;
        self.apu.reset();
        self.dma.reset();
        self.ppu.reset();
        self.input1.reset();
        self.input2.reset();
        if hard {
            self.ram.fill(0);
        }
        self.ram_address = 0;
        self.h_pos = 0;
        self.v_pos = 0;
        self.frames = 0;
        self.cpu_cycles_left = 52;
        self.cpu_mem_ops = 0;
        self.apu_catchup_cycles = 0.0;
        self.h_irq_enabled = false;
        self.v_irq_enabled = false;
        self.nmi_enabled = false;
        self.h_timer = 0x1ff;
        self.v_timer = 0x1ff;
        self.in_nmi = false;
        self.in_irq = false;
        self.in_vblank = false;
        self.port_auto_read.fill(0);
        self.auto_joy_read = false;
        self.auto_joy_timer = 0;
        self.ppu_latch = false;
        self.multiply_a = 0xff;
        self.multiply_result = 0xfe01;
        self.divide_a = 0xffff;
        self.divide_result = 0x0101;
        self.fast_mem = false;
        self.open_bus = 0;
        self.cpu_detached = false;
        self.cpu_irq_clear_latch = false;
    }

    /// Runs one CPU instruction using the timed CPU bus wrapper.
    pub fn run_cpu_opcode(&mut self) -> u8 {
        let mut cpu = core::mem::take(&mut self.cpu);
        self.cpu_detached = true;
        self.cpu_irq_clear_latch = false;
        let cycles = cpu.run_opcode(&mut CpuBus { snes: self });
        if self.cpu_irq_clear_latch {
            cpu.irq_wanted = false;
        }
        self.cpu_detached = false;
        self.cpu = cpu;
        cycles
    }

    /// Runs one two-master-cycle DMA tick.
    pub fn cycle_dma(&mut self) -> bool {
        let mut dma = core::mem::take(&mut self.dma);
        let active = dma.cycle(self);
        self.dma = dma;
        active
    }

    pub fn init_hdma(&mut self) {
        let mut dma = core::mem::take(&mut self.dma);
        dma.init_hdma(self);
        self.dma = dma;
    }

    pub fn do_hdma(&mut self) {
        let mut dma = core::mem::take(&mut self.dma);
        dma.do_hdma(self);
        self.dma = dma;
    }

    fn catch_up_apu(&mut self) {
        let cycles = self.apu_catchup_cycles as u32;
        for _ in 0..cycles {
            self.apu.cycle();
        }
        self.apu_catchup_cycles -= f64::from(cycles);
    }

    pub fn do_auto_joypad(&mut self) {
        self.port_auto_read.fill(0);
        self.input1.set_latch_line(true);
        self.input2.set_latch_line(true);
        self.input1.cycle();
        self.input2.cycle();
        self.input1.set_latch_line(false);
        self.input2.set_latch_line(false);
        for bit in 0..16 {
            let input1 = self.input1.read();
            self.port_auto_read[0] |= u16::from(input1 & 1) << (15 - bit);
            self.port_auto_read[2] |= u16::from(input1 >> 1 & 1) << (15 - bit);
            let input2 = self.input2.read();
            self.port_auto_read[1] |= u16::from(input2 & 1) << (15 - bit);
            self.port_auto_read[3] |= u16::from(input2 >> 1 & 1) << (15 - bit);
        }
    }

    pub fn read_b_bus(&mut self, address: u8) -> u8 {
        match address {
            0x00..=0x3f => self.ppu.read(address),
            0x40..=0x7f => self.apu.output_ports[usize::from(address & 3)],
            0x80 => {
                let value = self.ram[self.ram_address as usize];
                self.ram_address = (self.ram_address + 1) & 0x1ffff;
                value
            }
            _ => self.open_bus,
        }
    }

    pub fn write_b_bus(&mut self, address: u8, value: u8) {
        match address {
            0x00..=0x3f => self.ppu.write(address, value),
            0x40..=0x7f => {
                self.catch_up_apu();
                self.apu.input_ports[usize::from(address & 3)] = value;
            }
            0x80 => {
                self.ram[self.ram_address as usize] = value;
                self.ram_address = (self.ram_address + 1) & 0x1ffff;
            }
            0x81 => self.ram_address = self.ram_address & 0x1ff00 | u32::from(value),
            0x82 => self.ram_address = self.ram_address & 0x100ff | u32::from(value) << 8,
            0x83 => self.ram_address = self.ram_address & 0x0ffff | u32::from(value & 1) << 16,
            _ => {}
        }
    }

    fn read_register(&mut self, address: u16) -> u8 {
        match address {
            0x4210 => {
                let value = 2 | u8::from(self.in_nmi) << 7;
                self.in_nmi = false;
                value | self.open_bus & 0x70
            }
            0x4211 => {
                let value = u8::from(self.in_irq) << 7;
                self.in_irq = false;
                self.clear_cpu_irq();
                value | self.open_bus & 0x7f
            }
            0x4212 => {
                let value = u8::from(self.auto_joy_timer > 0)
                    | u8::from(self.h_pos >= 1024) << 6
                    | u8::from(self.in_vblank) << 7;
                value | self.open_bus & 0x3e
            }
            0x4213 => u8::from(self.ppu_latch) << 7,
            0x4214 => self.divide_result as u8,
            0x4215 => (self.divide_result >> 8) as u8,
            0x4216 => self.multiply_result as u8,
            0x4217 => (self.multiply_result >> 8) as u8,
            0x4218 | 0x421a | 0x421c | 0x421e => {
                self.port_auto_read[usize::from((address - 0x4218) / 2)] as u8
            }
            0x4219 | 0x421b | 0x421d | 0x421f => {
                (self.port_auto_read[usize::from((address - 0x4219) / 2)] >> 8) as u8
            }
            _ => self.open_bus,
        }
    }

    fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x4200 => {
                self.auto_joy_read = value & 1 != 0;
                if !self.auto_joy_read {
                    self.auto_joy_timer = 0;
                }
                self.h_irq_enabled = value & 0x10 != 0;
                self.v_irq_enabled = value & 0x20 != 0;
                self.nmi_enabled = value & 0x80 != 0;
                if !self.h_irq_enabled && !self.v_irq_enabled {
                    self.in_irq = false;
                    self.clear_cpu_irq();
                }
            }
            0x4201 => {
                if value & 0x80 == 0 && self.ppu_latch {
                    let _ = self.ppu.read(0x37);
                }
                self.ppu_latch = value & 0x80 != 0;
            }
            0x4202 => self.multiply_a = value,
            0x4203 => self.multiply_result = u16::from(self.multiply_a) * u16::from(value),
            0x4204 => self.divide_a = self.divide_a & 0xff00 | u16::from(value),
            0x4205 => self.divide_a = self.divide_a & 0x00ff | u16::from(value) << 8,
            0x4206 => {
                if value == 0 {
                    self.divide_result = 0xffff;
                    self.multiply_result = self.divide_a;
                } else {
                    self.divide_result = self.divide_a / u16::from(value);
                    self.multiply_result = self.divide_a % u16::from(value);
                }
            }
            0x4207 => self.h_timer = self.h_timer & 0x100 | u16::from(value),
            0x4208 => self.h_timer = self.h_timer & 0x0ff | u16::from(value & 1) << 8,
            0x4209 => self.v_timer = self.v_timer & 0x100 | u16::from(value),
            0x420a => self.v_timer = self.v_timer & 0x0ff | u16::from(value & 1) << 8,
            0x420b => self.dma.start_dma(value, false),
            0x420c => self.dma.start_dma(value, true),
            0x420d => self.fast_mem = value & 1 != 0,
            _ => {}
        }
    }

    fn read_raw(&mut self, address: u32) -> u8 {
        let bank = (address >> 16) as u8;
        let address = address as u16;
        if bank & 0x7f < 0x40 && address < 0x4380 {
            if address < 0x2000 {
                return self.ram[usize::from(address)];
            }
            if (0x2100..0x2200).contains(&address) {
                return self.read_b_bus(address as u8);
            }
            if address == 0x4016 {
                return self.input1.read() | self.open_bus & 0xfc;
            }
            if address == 0x4017 {
                return self.input2.read() | self.open_bus & 0xe0 | 0x1c;
            }
            if (0x4200..0x4220).contains(&address) {
                return self.read_register(address);
            }
            if (0x4300..0x4380).contains(&address) {
                return self.dma.read_register(self, address);
            }
        } else if bank & !1 == 0x7e {
            return self.ram[(usize::from(bank & 1) << 16) | usize::from(address)];
        }
        self.cart.read(bank, address, self.open_bus)
    }

    fn clear_cpu_irq(&mut self) {
        if self.cpu_detached {
            self.cpu_irq_clear_latch = true;
        } else {
            self.cpu.irq_wanted = false;
        }
    }

    pub fn read(&mut self, address: u32) -> u8 {
        let value = self.read_raw(address);
        self.open_bus = value;
        value
    }

    pub fn write(&mut self, address: u32, value: u8) {
        self.open_bus = value;
        let bank = (address >> 16) as u8;
        let address = address as u16;
        if bank == 0x7e || bank == 0x7f {
            self.ram[(usize::from(bank & 1) << 16) | usize::from(address)] = value;
        } else if bank < 0x40 || (0x80..0xc0).contains(&bank) {
            if address < 0x2000 {
                self.ram[usize::from(address)] = value;
            } else if (0x2100..0x2200).contains(&address) {
                self.write_b_bus(address as u8, value);
            } else if address == 0x4016 {
                let high = value & 1 != 0;
                self.input1.set_latch_line(high);
                self.input2.set_latch_line(high);
            } else if (0x4200..0x4220).contains(&address) {
                self.write_register(address, value);
            } else if (0x4300..0x4380).contains(&address) {
                self.dma.write_register(address, value);
            }
        }
        self.cart.write(bank, address, value);
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<RomInfo, LoadRomError> {
        if data.len() < 0x8000 {
            return Err(LoadRomError::TooSmall(data.len()));
        }
        let locations = [0x7fc0, 0x81c0, 0xffc0, 0x101c0];
        let mut headers = [CartHeader::default(); 4];
        for header in &mut headers {
            header.score = -50;
        }
        for (index, &location) in locations.iter().enumerate() {
            if data.len() >= location + 0x40 {
                headers[index] = CartHeader::read(data, location);
            }
        }
        let mut used = 0;
        let mut max = 0;
        for (index, header) in headers.iter().enumerate() {
            if header.score > max {
                max = header.score;
                used = index;
            }
        }
        let source = if used & 1 != 0 { &data[0x200..] } else { data };
        let header = headers[used];
        if header.cart_type > CART_HIROM {
            return Err(LoadRomError::UnsupportedType(header.cart_type));
        }
        let mut size = 0x8000usize;
        while source.len() > size {
            size *= 2;
        }
        let mut rom = vec![0; size];
        rom[..source.len()].copy_from_slice(source);
        let mut length = source.len();
        let mut test = 1;
        while length != size {
            if length & test != 0 {
                rom.copy_within(length - test..length, length);
                length += test;
            }
            test *= 2;
        }
        let ram_size = if header.chips > 0 {
            header.ram_size as usize
        } else {
            0
        };
        self.cart.load(header.cart_type, &rom, ram_size)?;
        self.reset(true);
        Ok(RomInfo {
            mapping: header.cart_type,
            name: String::from_utf8_lossy(&header.name).into_owned(),
            rom_size: size,
            pal: header.pal,
        })
    }
}

impl Bus for Snes {
    fn open_bus(&self) -> u8 {
        self.open_bus
    }
    fn read(&mut self, address: u32) -> u8 {
        Snes::read(self, address)
    }
    fn write(&mut self, address: u32, value: u8) {
        Snes::write(self, address, value)
    }
    fn read_b_bus(&mut self, address: u8) -> u8 {
        Snes::read_b_bus(self, address)
    }
    fn write_b_bus(&mut self, address: u8, value: u8) {
        Snes::write_b_bus(self, address, value)
    }
}

struct CpuBus<'a> {
    snes: &'a mut Snes,
}
impl Bus for CpuBus<'_> {
    fn open_bus(&self) -> u8 {
        self.snes.open_bus
    }
    fn read(&mut self, address: u32) -> u8 {
        self.snes.cpu_mem_ops = self.snes.cpu_mem_ops.wrapping_add(1);
        self.snes.cpu_cycles_left = self.snes.cpu_cycles_left.wrapping_add(6);
        self.snes.read(address)
    }
    fn write(&mut self, address: u32, value: u8) {
        self.snes.cpu_mem_ops = self.snes.cpu_mem_ops.wrapping_add(1);
        self.snes.cpu_cycles_left = self.snes.cpu_cycles_left.wrapping_add(6);
        self.snes.write(address, value)
    }
    fn read_b_bus(&mut self, address: u8) -> u8 {
        self.snes.read_b_bus(address)
    }
    fn write_b_bus(&mut self, address: u8, value: u8) {
        self.snes.write_b_bus(address, value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RomInfo {
    pub mapping: u8,
    pub name: String,
    pub rom_size: usize,
    pub pal: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoadRomError {
    TooSmall(usize),
    UnsupportedType(u8),
    Cart(CartLoadError),
}
impl From<CartLoadError> for LoadRomError {
    fn from(value: CartLoadError) -> Self {
        Self::Cart(value)
    }
}
impl fmt::Display for LoadRomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall(n) => write!(f, "ROM is too small ({n} bytes)"),
            Self::UnsupportedType(t) => write!(f, "unsupported cartridge type {t}"),
            Self::Cart(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for LoadRomError {}

#[derive(Clone, Copy)]
struct CartHeader {
    score: i16,
    name: [u8; 21],
    chips: u8,
    ram_size: u32,
    pal: bool,
    cart_type: u8,
}
impl Default for CartHeader {
    fn default() -> Self {
        Self {
            score: 0,
            name: [b'.'; 21],
            chips: 0,
            ram_size: 0,
            pal: false,
            cart_type: 0,
        }
    }
}
impl CartHeader {
    fn read(data: &[u8], location: usize) -> Self {
        let mut h = Self::default();
        h.name.copy_from_slice(&data[location..location + 21]);
        for c in &mut h.name {
            if !(0x20..0x7f).contains(c) {
                *c = b'.';
            }
        }
        let speed = data[location + 0x15] >> 4;
        let kind = data[location + 0x15] & 0xf;
        let coprocessor = data[location + 0x16] >> 4;
        h.chips = data[location + 0x16] & 0xf;
        h.ram_size = 0x400u32
            .checked_shl(u32::from(data[location + 0x18]))
            .unwrap_or(0);
        let region = data[location + 0x19];
        let complement = u16::from_le_bytes([data[location + 0x1c], data[location + 0x1d]]);
        let checksum = u16::from_le_bytes([data[location + 0x1e], data[location + 0x1f]]);
        h.pal = (2..=0xc).contains(&region) || region == 0x11;
        h.cart_type = if location < 0x9000 {
            CART_LOROM
        } else {
            CART_HIROM
        };
        let mut score = 0;
        score += if speed == 2 || speed == 3 { 5 } else { -4 };
        score += if kind <= 3 || kind == 5 { 5 } else { -2 };
        score += if coprocessor <= 5 || coprocessor >= 0xe {
            5
        } else {
            -2
        };
        score += if h.chips <= 6 || h.chips == 9 || h.chips == 0xa {
            5
        } else {
            -2
        };
        score += if region <= 0x14 { 5 } else { -2 };
        score += if checksum.wrapping_add(complement) == 0xffff {
            8
        } else {
            -6
        };
        let reset = u16::from_le_bytes([data[location + 0x3c], data[location + 0x3d]]);
        score += if reset >= 0x8000 { 8 } else { -20 };
        let opcode = data[location + 0x40 - 0x8000 + usize::from(reset & 0x7fff)];
        if opcode == 0x78 || opcode == 0x18 {
            score += 6;
        }
        if opcode == 0x4c || opcode == 0x5c || opcode == 0x9c {
            score += 3;
        }
        if opcode == 0 || opcode == 0xff || opcode == 0xdb {
            score -= 6;
        }
        h.score = score;
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saveload::{LoadState, SaveState};

    #[test]
    fn reset_matches_reference_register_defaults() {
        let mut snes = Snes::new();
        snes.ram[0x1234] = 0xaa;
        snes.reset(false);
        assert_eq!(snes.ram[0x1234], 0xaa);
        assert_eq!(snes.cpu_cycles_left, 52);
        assert_eq!((snes.h_timer, snes.v_timer), (0x1ff, 0x1ff));
        assert_eq!(snes.multiply_result, 0xfe01);
        assert_eq!(snes.divide_result, 0x0101);
        snes.reset(true);
        assert_eq!(snes.ram[0x1234], 0);
    }

    #[test]
    fn wram_mirrors_and_stream_port_wrap_exactly() {
        let mut snes = Snes::new();
        snes.write(0x001234, 0x56);
        assert_eq!(snes.read(0x7e1234), 0x56);
        snes.write_b_bus(0x81, 0xff);
        snes.write_b_bus(0x82, 0xff);
        snes.write_b_bus(0x83, 1);
        snes.write_b_bus(0x80, 0xaa);
        snes.write_b_bus(0x80, 0xbb);
        assert_eq!(snes.ram[0x1ffff], 0xaa);
        assert_eq!(snes.ram[0], 0xbb);
        assert_eq!(snes.ram_address, 1);
    }

    #[test]
    fn multiply_divide_and_divide_by_zero_match_c() {
        let mut snes = Snes::new();
        snes.write_register(0x4202, 13);
        snes.write_register(0x4203, 17);
        assert_eq!(snes.multiply_result, 221);
        snes.write_register(0x4204, 0x34);
        snes.write_register(0x4205, 0x12);
        snes.write_register(0x4206, 0x11);
        assert_eq!(snes.divide_result, 0x1234 / 0x11);
        assert_eq!(snes.multiply_result, 0x1234 % 0x11);
        snes.write_register(0x4206, 0);
        assert_eq!(snes.divide_result, 0xffff);
        assert_eq!(snes.multiply_result, 0x1234);
    }

    #[test]
    fn auto_joypad_places_serial_low_bit_first() {
        let mut snes = Snes::new();
        snes.input1.set_current_state(0xa55a);
        snes.do_auto_joypad();
        assert_eq!(snes.port_auto_read[0], 0x5aa5);
        assert_eq!(snes.port_auto_read[2], 0);
    }

    #[test]
    fn full_snes_saveload_stream_round_trips_exactly() {
        let mut source = Snes::new();
        source.h_pos = 777;
        source.apu_catchup_cycles = 12.75;
        source.cpu.a = 0x1234;
        source.apu.ram[0x4567] = 0x89;
        source.dma.channels[3].table_address = 0xabcd;
        source.ppu.vram[42] = 0xbeef;
        source.cart.ram_mut()[7] = 0x55;
        source.ram[0x1_2345] = 0xaa;

        let mut writer = SaveState::new();
        source.saveload(&mut writer).unwrap();
        let first = writer.into_bytes();

        let mut restored = Snes::new();
        let mut reader = LoadState::new(&first);
        restored.saveload(&mut reader).unwrap();
        reader.finish().unwrap();
        let mut writer = SaveState::new();
        restored.saveload(&mut writer).unwrap();
        assert_eq!(writer.into_bytes(), first);
        assert_eq!(restored.cpu.a, 0x1234);
        assert_eq!(restored.ppu.vram[42], 0xbeef);
        assert_eq!(restored.ram[0x1_2345], 0xaa);
    }
}
