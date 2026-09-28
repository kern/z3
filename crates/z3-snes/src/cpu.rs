//! Behavior-preserving 65C816 CPU port from `snes/cpu.c`.
//!
//! The CPU state is deliberately independent of the owning SNES. Memory is
//! supplied for each reset/step, avoiding a self-referential Rust structure.

use crate::bus::Bus;
use crate::saveload::{SaveLoad, SaveLoadError, bool_byte, get_u16, put_u16};

pub const CPU_SAVE_BLOCK_LEN: usize = 27;

pub const CYCLES_PER_OPCODE: [u8; 256] = [
    7, 6, 7, 4, 5, 3, 5, 6, 3, 2, 2, 4, 6, 4, 6, 5, 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 2, 2, 6, 4, 7, 5,
    6, 6, 8, 4, 3, 3, 5, 6, 4, 2, 2, 5, 4, 4, 6, 5, 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 2, 2, 4, 4, 7, 5,
    6, 6, 2, 4, 7, 3, 5, 6, 3, 2, 2, 3, 3, 4, 6, 5, 2, 5, 5, 7, 7, 4, 6, 6, 2, 4, 3, 2, 4, 4, 7, 5,
    6, 6, 6, 4, 3, 3, 5, 6, 4, 2, 2, 6, 5, 4, 6, 5, 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 4, 2, 6, 4, 7, 5,
    3, 6, 4, 4, 3, 3, 3, 6, 2, 2, 2, 3, 4, 4, 4, 5, 2, 6, 5, 7, 4, 4, 4, 6, 2, 5, 2, 2, 4, 5, 5, 5,
    2, 6, 2, 4, 3, 3, 3, 6, 2, 2, 2, 4, 4, 4, 4, 5, 2, 5, 5, 7, 4, 4, 4, 6, 2, 4, 2, 2, 4, 4, 4, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, 2, 5, 5, 7, 6, 4, 6, 6, 2, 4, 3, 3, 6, 4, 7, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 4, 2, 8, 4, 7, 5,
];

#[derive(Clone)]
pub struct Cpu {
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub sp: u16,
    pub pc: u16,
    pub dp: u16,
    pub k: u8,
    pub db: u8,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub d: bool,
    pub xf: bool,
    pub mf: bool,
    pub e: bool,
    pub irq_wanted: bool,
    pub nmi_wanted: bool,
    pub waiting: bool,
    pub stopped: bool,
    pub cycles_used: u8,
    pub sp_breakpoint: u16,
    pub in_emu: bool,
    pub hooked_function_rts: Option<fn(bool)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBus {
        mem: Vec<u8>,
    }

    impl TestBus {
        fn new() -> Self {
            Self {
                mem: vec![0; 1 << 24],
            }
        }
    }

    impl Bus for TestBus {
        fn open_bus(&self) -> u8 {
            0
        }
        fn read(&mut self, address: u32) -> u8 {
            self.mem[(address & 0xff_ffff) as usize]
        }
        fn write(&mut self, address: u32, value: u8) {
            self.mem[(address & 0xff_ffff) as usize] = value;
        }
        fn read_b_bus(&mut self, address: u8) -> u8 {
            self.mem[0x2100 | usize::from(address)]
        }
        fn write_b_bus(&mut self, address: u8, value: u8) {
            self.mem[0x2100 | usize::from(address)] = value;
        }
    }

    #[test]
    fn reset_and_flags_match_c_core() {
        let mut bus = TestBus::new();
        bus.mem[0xfffc] = 0x34;
        bus.mem[0xfffd] = 0x12;
        let mut cpu = Cpu::new();
        cpu.reset(&mut bus);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0x100);
        assert_eq!(cpu.flags(), 0x34);
        cpu.e = false;
        cpu.set_flags(0xcb);
        assert_eq!(cpu.flags(), 0xcb);
    }

    #[test]
    fn native_width_and_decimal_arithmetic() {
        let mut bus = TestBus::new();
        let mut cpu = Cpu::new();
        cpu.e = false;
        cpu.set_flags(0x08); // decimal, 16-bit A/X
        cpu.a = 0x9999;
        bus.mem[0x100] = 1;
        bus.mem[0x101] = 0;
        cpu.adc(&mut bus, 0x100, 0x101);
        assert_eq!(cpu.a, 0);
        assert!(cpu.c);
        assert!(cpu.z);
        cpu.sbc(&mut bus, 0x100, 0x101);
        assert_eq!(cpu.a, 0x9999);
    }

    #[test]
    fn stack_wraps_within_emulation_page() {
        let mut bus = TestBus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0x100;
        cpu.push_byte(&mut bus, 0xaa);
        assert_eq!(cpu.sp, 0x1ff);
        assert_eq!(cpu.pull_byte(&mut bus), 0xaa);
        assert_eq!(cpu.sp, 0x100);
    }

    #[test]
    fn every_opcode_has_a_dispatch_path() {
        let mut bus = TestBus::new();
        for opcode in 0u8..=u8::MAX {
            let mut cpu = Cpu::new();
            cpu.e = false;
            cpu.mf = true;
            cpu.xf = true;
            cpu.sp = 0x1f0;
            cpu.pc = 0x200;
            if opcode == 0 {
                cpu.k = 0x07;
                cpu.pc = 0xb26a; // Zelda's intentional 0x7b269 BRK patch.
            }
            cpu.execute(&mut bus, opcode);
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0x100,
            pc: 0,
            dp: 0,
            k: 0,
            db: 0,
            c: false,
            z: false,
            v: false,
            n: false,
            i: true,
            d: false,
            xf: true,
            mf: true,
            e: true,
            irq_wanted: false,
            nmi_wanted: false,
            waiting: false,
            stopped: false,
            cycles_used: 0,
            sp_breakpoint: 0,
            in_emu: false,
            hooked_function_rts: None,
        }
    }
}

impl Cpu {
    /// Transfers the C CPU range from `a` up to (but excluding)
    /// `cyclesUsed`; the debugger stack breakpoint is cleared afterwards.
    pub fn saveload(&mut self, io: &mut impl SaveLoad) -> Result<(), SaveLoadError> {
        let mut block = [0; CPU_SAVE_BLOCK_LEN];
        if !io.is_loading() {
            put_u16(&mut block, 0, self.a);
            put_u16(&mut block, 2, self.x);
            put_u16(&mut block, 4, self.y);
            put_u16(&mut block, 6, self.sp);
            put_u16(&mut block, 8, self.pc);
            put_u16(&mut block, 10, self.dp);
            block[12] = self.k;
            block[13] = self.db;
            let flags = [
                self.c,
                self.z,
                self.v,
                self.n,
                self.i,
                self.d,
                self.xf,
                self.mf,
                self.e,
                self.irq_wanted,
                self.nmi_wanted,
                self.waiting,
                self.stopped,
            ];
            for (index, value) in flags.into_iter().enumerate() {
                block[14 + index] = bool_byte(value);
            }
        }
        io.transfer(&mut block)?;
        if io.is_loading() {
            self.a = get_u16(&block, 0);
            self.x = get_u16(&block, 2);
            self.y = get_u16(&block, 4);
            self.sp = get_u16(&block, 6);
            self.pc = get_u16(&block, 8);
            self.dp = get_u16(&block, 10);
            self.k = block[12];
            self.db = block[13];
            self.c = block[14] != 0;
            self.z = block[15] != 0;
            self.v = block[16] != 0;
            self.n = block[17] != 0;
            self.i = block[18] != 0;
            self.d = block[19] != 0;
            self.xf = block[20] != 0;
            self.mf = block[21] != 0;
            self.e = block[22] != 0;
            self.irq_wanted = block[23] != 0;
            self.nmi_wanted = block[24] != 0;
            self.waiting = block[25] != 0;
            self.stopped = block[26] != 0;
        }
        self.sp_breakpoint = 0;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Idx,
    Sr,
    Dp,
    Idl,
    ImmM,
    ImmX,
    Abs,
    Abl,
    Idy,
    Idp,
    Isy,
    Dpx,
    Dpy,
    Ily,
    Aby,
    Abx,
    Alx,
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self, bus: &mut impl Bus) {
        let hook = self.hooked_function_rts;
        *self = Self::default();
        self.hooked_function_rts = hook;
        self.pc = self.read_word(bus, 0xfffc, 0xfffd);
    }

    pub fn flags(&self) -> u8 {
        u8::from(self.n) << 7
            | u8::from(self.v) << 6
            | u8::from(self.mf) << 5
            | u8::from(self.xf) << 4
            | u8::from(self.d) << 3
            | u8::from(self.i) << 2
            | u8::from(self.z) << 1
            | u8::from(self.c)
    }

    pub fn set_flags(&mut self, val: u8) {
        self.n = val & 0x80 != 0;
        self.v = val & 0x40 != 0;
        self.mf = val & 0x20 != 0;
        self.xf = val & 0x10 != 0;
        self.d = val & 8 != 0;
        self.i = val & 4 != 0;
        self.z = val & 2 != 0;
        self.c = val & 1 != 0;
        if self.e {
            self.mf = true;
            self.xf = true;
            self.sp = self.sp & 0xff | 0x100;
        }
        if self.xf {
            self.x &= 0xff;
            self.y &= 0xff;
        }
    }

    pub fn run_opcode(&mut self, bus: &mut impl Bus) -> u8 {
        self.cycles_used = 0;
        if self.stopped {
            return 1;
        }
        if self.waiting {
            if self.irq_wanted || self.nmi_wanted {
                self.waiting = false;
            }
            return 1;
        }
        if (!self.i && self.irq_wanted) || self.nmi_wanted {
            self.cycles_used = 7;
            let irq = !self.nmi_wanted;
            self.nmi_wanted = false;
            self.interrupt(bus, irq);
        } else {
            let opcode = self.read_opcode(bus);
            self.cycles_used = CYCLES_PER_OPCODE[opcode as usize];
            self.execute(bus, opcode);
        }
        self.cycles_used
    }

    fn read_opcode(&mut self, bus: &mut impl Bus) -> u8 {
        let value = bus.read(u32::from(self.k) << 16 | u32::from(self.pc));
        self.pc = self.pc.wrapping_add(1);
        value
    }
    fn read_opcode_word(&mut self, bus: &mut impl Bus) -> u16 {
        let lo = self.read_opcode(bus);
        u16::from_le_bytes([lo, self.read_opcode(bus)])
    }
    fn read_word(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) -> u16 {
        u16::from_le_bytes([bus.read(lo), bus.read(hi)])
    }
    fn write_word(&mut self, bus: &mut impl Bus, lo: u32, hi: u32, value: u16, reversed: bool) {
        let [l, h] = value.to_le_bytes();
        if reversed {
            bus.write(hi, h);
            bus.write(lo, l);
        } else {
            bus.write(lo, l);
            bus.write(hi, h);
        }
    }
    fn push_byte(&mut self, bus: &mut impl Bus, value: u8) {
        bus.write(u32::from(self.sp), value);
        self.sp = self.sp.wrapping_sub(1);
        if self.e {
            self.sp = self.sp & 0xff | 0x100;
        }
    }
    fn pull_byte(&mut self, bus: &mut impl Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        if self.e {
            self.sp = self.sp & 0xff | 0x100;
        }
        bus.read(u32::from(self.sp))
    }
    fn push_word(&mut self, bus: &mut impl Bus, value: u16) {
        self.push_byte(bus, (value >> 8) as u8);
        self.push_byte(bus, value as u8);
    }
    fn pull_word(&mut self, bus: &mut impl Bus) -> u16 {
        let lo = self.pull_byte(bus);
        u16::from_le_bytes([lo, self.pull_byte(bus)])
    }
    fn set_zn(&mut self, value: u16, byte: bool) {
        if byte {
            self.z = value as u8 == 0;
            self.n = value & 0x80 != 0;
        } else {
            self.z = value == 0;
            self.n = value & 0x8000 != 0;
        }
    }
    fn branch(&mut self, displacement: u8, check: bool) {
        if check {
            self.cycles_used = self.cycles_used.wrapping_add(1);
            self.pc = self.pc.wrapping_add_signed(i16::from(displacement as i8));
        }
    }
    fn interrupt(&mut self, bus: &mut impl Bus, irq: bool) {
        self.push_byte(bus, self.k);
        self.push_word(bus, self.pc);
        self.push_byte(bus, self.flags());
        self.cycles_used = self.cycles_used.wrapping_add(1);
        self.i = true;
        self.d = false;
        self.k = 0;
        self.pc = if irq {
            self.read_word(bus, 0xffee, 0xffef)
        } else {
            self.read_word(bus, 0xffea, 0xffeb)
        };
    }

    fn addr(&mut self, bus: &mut impl Bus, mode: Mode, write: bool) -> (u32, u32) {
        match mode {
            Mode::ImmM | Mode::ImmX => {
                let lo = u32::from(self.k) << 16 | u32::from(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let byte = if matches!(mode, Mode::ImmM) {
                    self.mf
                } else {
                    self.xf
                };
                let hi = if byte {
                    0
                } else {
                    let h = u32::from(self.k) << 16 | u32::from(self.pc);
                    self.pc = self.pc.wrapping_add(1);
                    h
                };
                (lo, hi)
            }
            Mode::Dp | Mode::Dpx | Mode::Dpy | Mode::Sr => {
                let a = u16::from(self.read_opcode(bus));
                let base = match mode {
                    Mode::Dpx => self.dp.wrapping_add(a).wrapping_add(self.x),
                    Mode::Dpy => self.dp.wrapping_add(a).wrapping_add(self.y),
                    Mode::Sr => self.sp.wrapping_add(a),
                    _ => self.dp.wrapping_add(a),
                };
                if !matches!(mode, Mode::Sr) && self.dp & 0xff != 0 {
                    self.cycles_used = self.cycles_used.wrapping_add(1);
                }
                (u32::from(base), u32::from(base.wrapping_add(1)))
            }
            Mode::Idx => {
                let a = u16::from(self.read_opcode(bus));
                if self.dp & 0xff != 0 {
                    self.cycles_used += 1;
                }
                let p = self.dp.wrapping_add(a).wrapping_add(self.x);
                let q = self.read_word(bus, u32::from(p), u32::from(p.wrapping_add(1)));
                self.bank_pair_index(q, 0)
            }
            Mode::Idp | Mode::Idy => {
                let a = u16::from(self.read_opcode(bus));
                if self.dp & 0xff != 0 {
                    self.cycles_used += 1;
                }
                let p = self.dp.wrapping_add(a);
                let q = self.read_word(bus, u32::from(p), u32::from(p.wrapping_add(1)));
                let idx = if matches!(mode, Mode::Idy) { self.y } else { 0 };
                if matches!(mode, Mode::Idy)
                    && write
                    && (!self.xf || (u32::from(q) >> 8 != (u32::from(q) + u32::from(idx)) >> 8))
                {
                    self.cycles_used += 1;
                }
                self.bank_pair_index(q, idx)
            }
            Mode::Idl | Mode::Ily => {
                let a = u16::from(self.read_opcode(bus));
                if self.dp & 0xff != 0 {
                    self.cycles_used += 1;
                }
                let p = self.dp.wrapping_add(a);
                let mut q =
                    u32::from(self.read_word(bus, u32::from(p), u32::from(p.wrapping_add(1))));
                q |= u32::from(bus.read(u32::from(p.wrapping_add(2)))) << 16;
                if matches!(mode, Mode::Ily) {
                    q = q.wrapping_add(u32::from(self.y)) & 0xffffff;
                }
                (q, (q + 1) & 0xffffff)
            }
            Mode::Isy => {
                let a = u16::from(self.read_opcode(bus));
                let p = self.sp.wrapping_add(a);
                let q = self
                    .read_word(bus, u32::from(p), u32::from(p.wrapping_add(1)))
                    .wrapping_add(self.y);
                self.bank_pair_index(q, 0)
            }
            Mode::Abs | Mode::Abx | Mode::Aby => {
                let a = self.read_opcode_word(bus);
                let idx = if matches!(mode, Mode::Abx) {
                    self.x
                } else if matches!(mode, Mode::Aby) {
                    self.y
                } else {
                    0
                };
                if !matches!(mode, Mode::Abs)
                    && write
                    && (!self.xf || (u32::from(a) >> 8 != (u32::from(a) + u32::from(idx)) >> 8))
                {
                    self.cycles_used += 1;
                }
                self.bank_pair_index(a, idx)
            }
            Mode::Abl | Mode::Alx => {
                let mut a =
                    u32::from(self.read_opcode_word(bus)) | u32::from(self.read_opcode(bus)) << 16;
                if matches!(mode, Mode::Alx) {
                    a = a.wrapping_add(u32::from(self.x)) & 0xffffff;
                }
                (a, (a + 1) & 0xffffff)
            }
        }
    }
    fn bank_pair_index(&self, address: u16, index: u16) -> (u32, u32) {
        let a = ((u32::from(self.db) << 16) + u32::from(address) + u32::from(index)) & 0xff_ffff;
        (a, (a + 1) & 0xffffff)
    }

    fn family_mode(op: u8) -> Option<Mode> {
        Some(match op & 0x1f {
            0x01 => Mode::Idx,
            0x03 => Mode::Sr,
            0x05 => Mode::Dp,
            0x07 => Mode::Idl,
            0x09 => Mode::ImmM,
            0x0d => Mode::Abs,
            0x0f => Mode::Abl,
            0x11 => Mode::Idy,
            0x12 => Mode::Idp,
            0x13 => Mode::Isy,
            0x15 => Mode::Dpx,
            0x17 => Mode::Ily,
            0x19 => Mode::Aby,
            0x1d => Mode::Abx,
            0x1f => Mode::Alx,
            _ => return None,
        })
    }

    fn execute(&mut self, bus: &mut impl Bus, opcode: u8) {
        if let Some(mode) = Self::family_mode(opcode) {
            let family = opcode & 0xe0;
            if !(family == 0x80 && opcode == 0x89) {
                let (lo, hi) = self.addr(bus, mode, family == 0x80);
                match family {
                    0x00 => self.ora(bus, lo, hi),
                    0x20 => self.and(bus, lo, hi),
                    0x40 => self.eor(bus, lo, hi),
                    0x60 => self.adc(bus, lo, hi),
                    0x80 => self.sta(bus, lo, hi),
                    0xa0 => self.lda(bus, lo, hi),
                    0xc0 => self.cmp(bus, lo, hi),
                    0xe0 => self.sbc(bus, lo, hi),
                    _ => unreachable!(),
                }
                return;
            }
        }
        self.execute_special(bus, opcode);
    }

    fn logic_value(&mut self, bus: &mut impl Bus, lo: u32, hi: u32, f: impl Fn(u16, u16) -> u16) {
        if self.mf {
            let a = self.a & 0xff00;
            self.a = a | f(self.a & 0xff, u16::from(bus.read(lo))) & 0xff;
        } else {
            self.cycles_used += 1;
            let value = self.read_word(bus, lo, hi);
            self.a = f(self.a, value);
        }
        self.set_zn(self.a, self.mf);
    }
    fn and(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        self.logic_value(bus, lo, hi, |a, b| a & b)
    }
    fn ora(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        self.logic_value(bus, lo, hi, |a, b| a | b)
    }
    fn eor(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        self.logic_value(bus, lo, hi, |a, b| a ^ b)
    }
    fn lda(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        if self.mf {
            self.a = self.a & 0xff00 | u16::from(bus.read(lo));
        } else {
            self.cycles_used += 1;
            self.a = self.read_word(bus, lo, hi);
        }
        self.set_zn(self.a, self.mf)
    }
    fn ldx(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        if self.xf {
            self.x = u16::from(bus.read(lo));
        } else {
            self.cycles_used += 1;
            self.x = self.read_word(bus, lo, hi);
        }
        self.set_zn(self.x, self.xf)
    }
    fn ldy(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        if self.xf {
            self.y = u16::from(bus.read(lo));
        } else {
            self.cycles_used += 1;
            self.y = self.read_word(bus, lo, hi);
        }
        self.set_zn(self.y, self.xf)
    }
    fn sta(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        if self.mf {
            bus.write(lo, self.a as u8)
        } else {
            self.cycles_used += 1;
            self.write_word(bus, lo, hi, self.a, false)
        }
    }
    fn stx(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        if self.xf {
            bus.write(lo, self.x as u8)
        } else {
            self.cycles_used += 1;
            self.write_word(bus, lo, hi, self.x, false)
        }
    }
    fn sty(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        if self.xf {
            bus.write(lo, self.y as u8)
        } else {
            self.cycles_used += 1;
            self.write_word(bus, lo, hi, self.y, false)
        }
    }
    fn stz(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        if self.mf {
            bus.write(lo, 0)
        } else {
            self.cycles_used += 1;
            self.write_word(bus, lo, hi, 0, false)
        }
    }

    fn adc(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        let value = if self.mf {
            u32::from(bus.read(lo))
        } else {
            self.cycles_used += 1;
            u32::from(self.read_word(bus, lo, hi))
        };
        let a = if self.mf {
            u32::from(self.a & 0xff)
        } else {
            u32::from(self.a)
        };
        let mask = if self.mf { 0xff } else { 0xffff };
        let sign = if self.mf { 0x80 } else { 0x8000 };
        let mut result;
        if self.d {
            result = (a & 0xf) + (value & 0xf) + u32::from(self.c);
            if result > 9 {
                result = ((result + 6) & 0xf) + 0x10;
            }
            result += (a & 0xf0) + (value & 0xf0);
            if !self.mf && result > 0x9f {
                result = ((result + 0x60) & 0xff) + 0x100;
            }
            if !self.mf {
                result += (a & 0xf00) + (value & 0xf00);
                if result > 0x9ff {
                    result = ((result + 0x600) & 0xfff) + 0x1000;
                }
                result += (a & 0xf000) + (value & 0xf000);
            }
        } else {
            result = a + value + u32::from(self.c);
        }
        self.v = (a & sign) == (value & sign) && (value & sign) != (result & sign);
        if self.d && result > if self.mf { 0x9f } else { 0x9fff } {
            result += if self.mf { 0x60 } else { 0x6000 };
        }
        self.c = result > mask;
        let r = result as u16;
        self.a = if self.mf {
            self.a & 0xff00 | r & 0xff
        } else {
            r
        };
        self.set_zn(self.a, self.mf);
    }
    fn sbc(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        let raw = if self.mf {
            u32::from(bus.read(lo))
        } else {
            self.cycles_used += 1;
            u32::from(self.read_word(bus, lo, hi))
        };
        let mask = if self.mf { 0xff } else { 0xffff };
        let value = raw ^ mask;
        let a = if self.mf {
            u32::from(self.a & 0xff)
        } else {
            u32::from(self.a)
        };
        let sign = if self.mf { 0x80 } else { 0x8000 };
        let mut result: i32;
        if self.d {
            result = ((a & 0xf) + (value & 0xf) + u32::from(self.c)) as i32;
            if result < 0x10 {
                let t = result - 6;
                result = t & (if t < 0 { 0xf } else { 0x1f });
            }
            result += ((a & 0xf0) + (value & 0xf0)) as i32;
            if !self.mf && result < 0x100 {
                let t = result - 0x60;
                result = t & (if t < 0 { 0xff } else { 0x1ff });
            }
            if !self.mf {
                result += ((a & 0xf00) + (value & 0xf00)) as i32;
                if result < 0x1000 {
                    let t = result - 0x600;
                    result = t & (if t < 0 { 0xfff } else { 0x1fff });
                }
                result += ((a & 0xf000) + (value & 0xf000)) as i32;
            }
        } else {
            result = (a + value + u32::from(self.c)) as i32;
        }
        self.v = (a & sign) == (value & sign) && (value & sign) != ((result as u32) & sign);
        if self.d && result < (mask + 1) as i32 {
            result -= if self.mf { 0x60 } else { 0x6000 };
        }
        self.c = result > mask as i32;
        let r = result as u16;
        self.a = if self.mf {
            self.a & 0xff00 | r & 0xff
        } else {
            r
        };
        self.set_zn(self.a, self.mf);
    }
    fn compare(&mut self, bus: &mut impl Bus, lo: u32, hi: u32, reg: u16, index: bool) {
        let byte = if index { self.xf } else { self.mf };
        let mask = if byte { 0xff } else { 0xffff };
        if !byte {
            self.cycles_used += 1;
        }
        let value = if byte {
            u16::from(bus.read(lo))
        } else {
            self.read_word(bus, lo, hi)
        };
        let result = u32::from(reg & mask) + u32::from(value ^ mask) + 1;
        self.c = result > u32::from(mask);
        self.set_zn(result as u16, byte)
    }
    fn cmp(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        self.compare(bus, lo, hi, self.a, false)
    }
    fn cpx(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        self.compare(bus, lo, hi, self.x, true)
    }
    fn cpy(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        self.compare(bus, lo, hi, self.y, true)
    }
    fn bit(&mut self, bus: &mut impl Bus, lo: u32, hi: u32) {
        if self.mf {
            let v = bus.read(lo);
            self.z = (self.a as u8 & v) == 0;
            self.n = v & 0x80 != 0;
            self.v = v & 0x40 != 0;
        } else {
            self.cycles_used += 1;
            let v = self.read_word(bus, lo, hi);
            self.z = self.a & v == 0;
            self.n = v & 0x8000 != 0;
            self.v = v & 0x4000 != 0;
        }
    }

    fn rmw(&mut self, bus: &mut impl Bus, lo: u32, hi: u32, kind: u8) {
        let value = if self.mf {
            u32::from(bus.read(lo))
        } else {
            self.cycles_used += 2;
            u32::from(self.read_word(bus, lo, hi))
        };
        let bits = if self.mf { 8 } else { 16 };
        let mask = if self.mf { 0xff } else { 0xffff };
        let result = match kind {
            0 => {
                let r = value << 1 | u32::from(self.c);
                self.c = r > mask;
                r
            }
            1 => {
                self.c = value & 1 != 0;
                value >> 1
            }
            2 => {
                let c = value & 1 != 0;
                let r = value >> 1 | u32::from(self.c) << (bits - 1);
                self.c = c;
                r
            }
            3 => {
                let r = value << 1;
                self.c = r > mask;
                r
            }
            4 => value.wrapping_add(1),
            5 => value.wrapping_sub(1),
            _ => unreachable!(),
        } & mask;
        if self.mf {
            bus.write(lo, result as u8)
        } else {
            self.write_word(bus, lo, hi, result as u16, true)
        }
        self.set_zn(result as u16, self.mf);
    }
    fn tsb_trb(&mut self, bus: &mut impl Bus, lo: u32, hi: u32, set: bool) {
        if self.mf {
            let v = bus.read(lo);
            self.z = (self.a as u8 & v) == 0;
            bus.write(
                lo,
                if set {
                    v | self.a as u8
                } else {
                    v & !(self.a as u8)
                },
            );
        } else {
            self.cycles_used += 2;
            let v = self.read_word(bus, lo, hi);
            self.z = self.a & v == 0;
            self.write_word(
                bus,
                lo,
                hi,
                if set { v | self.a } else { v & !self.a },
                true,
            )
        }
    }

    fn indexed_indirect(&mut self, bus: &mut impl Bus) -> u16 {
        let a = self.read_opcode_word(bus);
        let p = (u32::from(self.k) << 16) | u32::from(a.wrapping_add(self.x));
        self.read_word(
            bus,
            p,
            (u32::from(self.k) << 16) | u32::from(a.wrapping_add(self.x).wrapping_add(1)),
        )
    }
    fn mem_op(&mut self, bus: &mut impl Bus, mode: Mode, write: bool, op: u8) {
        let (lo, hi) = self.addr(bus, mode, write);
        match op {
            0 => self.rmw(bus, lo, hi, 0),
            1 => self.rmw(bus, lo, hi, 1),
            2 => self.rmw(bus, lo, hi, 2),
            3 => self.rmw(bus, lo, hi, 3),
            4 => self.rmw(bus, lo, hi, 4),
            5 => self.rmw(bus, lo, hi, 5),
            6 => self.bit(bus, lo, hi),
            7 => self.tsb_trb(bus, lo, hi, true),
            8 => self.tsb_trb(bus, lo, hi, false),
            9 => self.stz(bus, lo, hi),
            10 => self.stx(bus, lo, hi),
            11 => self.sty(bus, lo, hi),
            12 => self.ldx(bus, lo, hi),
            13 => self.ldy(bus, lo, hi),
            14 => self.cpx(bus, lo, hi),
            15 => self.cpy(bus, lo, hi),
            _ => unreachable!(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn execute_special(&mut self, bus: &mut impl Bus, opcode: u8) {
        match opcode {
            0x00 => self.brk_patch(bus),
            0x02 => {
                self.read_opcode(bus);
                self.push_byte(bus, self.k);
                self.push_word(bus, self.pc);
                self.push_byte(bus, self.flags());
                self.cycles_used += 1;
                self.i = true;
                self.d = false;
                self.k = 0;
                self.pc = self.read_word(bus, 0xffe4, 0xffe5);
            }
            0x04 => self.mem_op(bus, Mode::Dp, false, 7),
            0x0c => self.mem_op(bus, Mode::Abs, false, 7),
            0x14 => self.mem_op(bus, Mode::Dp, false, 8),
            0x1c => self.mem_op(bus, Mode::Abs, false, 8),
            0x06 => self.mem_op(bus, Mode::Dp, true, 3),
            0x0e => self.mem_op(bus, Mode::Abs, true, 3),
            0x16 => self.mem_op(bus, Mode::Dpx, true, 3),
            0x1e => self.mem_op(bus, Mode::Abx, true, 3),
            0x26 => self.mem_op(bus, Mode::Dp, true, 0),
            0x2e => self.mem_op(bus, Mode::Abs, true, 0),
            0x36 => self.mem_op(bus, Mode::Dpx, true, 0),
            0x3e => self.mem_op(bus, Mode::Abx, true, 0),
            0x46 => self.mem_op(bus, Mode::Dp, true, 1),
            0x4e => self.mem_op(bus, Mode::Abs, true, 1),
            0x56 => self.mem_op(bus, Mode::Dpx, true, 1),
            0x5e => self.mem_op(bus, Mode::Abx, true, 1),
            0x66 => self.mem_op(bus, Mode::Dp, true, 2),
            0x6e => self.mem_op(bus, Mode::Abs, true, 2),
            0x76 => self.mem_op(bus, Mode::Dpx, true, 2),
            0x7e => self.mem_op(bus, Mode::Abx, true, 2),
            0xc6 => self.mem_op(bus, Mode::Dp, true, 5),
            0xce => self.mem_op(bus, Mode::Abs, true, 5),
            0xd6 => self.mem_op(bus, Mode::Dpx, true, 5),
            0xde => self.mem_op(bus, Mode::Abx, true, 5),
            0xe6 => self.mem_op(bus, Mode::Dp, true, 4),
            0xee => self.mem_op(bus, Mode::Abs, true, 4),
            0xf6 => self.mem_op(bus, Mode::Dpx, true, 4),
            0xfe => self.mem_op(bus, Mode::Abx, true, 4),
            0x24 => self.mem_op(bus, Mode::Dp, false, 6),
            0x2c => self.mem_op(bus, Mode::Abs, false, 6),
            0x34 => self.mem_op(bus, Mode::Dpx, false, 6),
            0x3c => self.mem_op(bus, Mode::Abx, false, 6),
            0x64 => self.mem_op(bus, Mode::Dp, false, 9),
            0x74 => self.mem_op(bus, Mode::Dpx, false, 9),
            0x9c => self.mem_op(bus, Mode::Abs, false, 9),
            0x9e => self.mem_op(bus, Mode::Abx, true, 9),
            0x84 => self.mem_op(bus, Mode::Dp, false, 11),
            0x8c => self.mem_op(bus, Mode::Abs, false, 11),
            0x94 => self.mem_op(bus, Mode::Dpx, false, 11),
            0x86 => self.mem_op(bus, Mode::Dp, false, 10),
            0x8e => self.mem_op(bus, Mode::Abs, false, 10),
            0x96 => self.mem_op(bus, Mode::Dpy, false, 10),
            0xa0 => self.mem_op(bus, Mode::ImmX, false, 13),
            0xa4 => self.mem_op(bus, Mode::Dp, false, 13),
            0xac => self.mem_op(bus, Mode::Abs, false, 13),
            0xb4 => self.mem_op(bus, Mode::Dpx, false, 13),
            0xbc => self.mem_op(bus, Mode::Abx, false, 13),
            0xa2 => self.mem_op(bus, Mode::ImmX, false, 12),
            0xa6 => self.mem_op(bus, Mode::Dp, false, 12),
            0xae => self.mem_op(bus, Mode::Abs, false, 12),
            0xb6 => self.mem_op(bus, Mode::Dpy, false, 12),
            0xbe => self.mem_op(bus, Mode::Aby, false, 12),
            0xc0 => self.mem_op(bus, Mode::ImmX, false, 15),
            0xc4 => self.mem_op(bus, Mode::Dp, false, 15),
            0xcc => self.mem_op(bus, Mode::Abs, false, 15),
            0xe0 => self.mem_op(bus, Mode::ImmX, false, 14),
            0xe4 => self.mem_op(bus, Mode::Dp, false, 14),
            0xec => self.mem_op(bus, Mode::Abs, false, 14),
            0x08 => {
                let f = self.flags();
                self.push_byte(bus, f)
            }
            0x0b => self.push_word(bus, self.dp),
            0x28 => {
                let f = self.pull_byte(bus);
                self.set_flags(f)
            }
            0x2b => {
                self.dp = self.pull_word(bus);
                self.set_zn(self.dp, false)
            }
            0x48 => {
                if self.mf {
                    self.push_byte(bus, self.a as u8)
                } else {
                    self.cycles_used += 1;
                    self.push_word(bus, self.a)
                }
            }
            0x4b => self.push_byte(bus, self.k),
            0x5a => {
                if self.xf {
                    self.push_byte(bus, self.y as u8)
                } else {
                    self.cycles_used += 1;
                    self.push_word(bus, self.y)
                }
            }
            0x68 => {
                if self.mf {
                    let v = self.pull_byte(bus);
                    self.a = self.a & 0xff00 | u16::from(v)
                } else {
                    self.cycles_used += 1;
                    self.a = self.pull_word(bus)
                }
                self.set_zn(self.a, self.mf)
            }
            0x7a => {
                if self.xf {
                    self.y = u16::from(self.pull_byte(bus))
                } else {
                    self.cycles_used += 1;
                    self.y = self.pull_word(bus)
                }
                self.set_zn(self.y, self.xf)
            }
            0x8b => self.push_byte(bus, self.db),
            0xab => {
                self.db = self.pull_byte(bus);
                self.set_zn(u16::from(self.db), true)
            }
            0xda => {
                if self.xf {
                    self.push_byte(bus, self.x as u8)
                } else {
                    self.cycles_used += 1;
                    self.push_word(bus, self.x)
                }
            }
            0xfa => {
                if self.xf {
                    self.x = u16::from(self.pull_byte(bus))
                } else {
                    self.cycles_used += 1;
                    self.x = self.pull_word(bus)
                }
                self.set_zn(self.x, self.xf)
            }
            0xf4 => {
                let v = self.read_opcode_word(bus);
                self.push_word(bus, v)
            }
            0xd4 => {
                let (lo, hi) = self.addr(bus, Mode::Dp, false);
                let v = self.read_word(bus, lo, hi);
                self.push_word(bus, v)
            }
            0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xb0 | 0xd0 | 0xf0 => {
                let d = self.read_opcode(bus);
                let take = match opcode {
                    0x10 => !self.n,
                    0x30 => self.n,
                    0x50 => !self.v,
                    0x70 => self.v,
                    0x90 => !self.c,
                    0xb0 => self.c,
                    0xd0 => !self.z,
                    _ => self.z,
                };
                self.branch(d, take)
            }
            0x18 => self.c = false,
            0x38 => self.c = true,
            0x58 => self.i = false,
            0x78 => self.i = true,
            0xb8 => self.v = false,
            0xd8 => self.d = false,
            0xf8 => self.d = true,
            0x80 => {
                let d = self.read_opcode(bus);
                self.pc = self.pc.wrapping_add_signed(i16::from(d as i8))
            }
            0x82 => {
                let d = self.read_opcode_word(bus) as i16;
                self.pc = self.pc.wrapping_add_signed(d)
            }
            0x20 => {
                let v = self.read_opcode_word(bus);
                self.push_word(bus, self.pc.wrapping_sub(1));
                self.pc = v
            }
            0x22 => {
                let v = self.read_opcode_word(bus);
                let k = self.read_opcode(bus);
                self.push_byte(bus, self.k);
                self.push_word(bus, self.pc.wrapping_sub(1));
                self.pc = v;
                self.k = k
            }
            0x4c => self.pc = self.read_opcode_word(bus),
            0x5c => {
                let pc = self.read_opcode_word(bus);
                self.k = self.read_opcode(bus);
                self.pc = pc
            }
            0x6c => {
                let a = self.read_opcode_word(bus);
                self.pc = self.read_word(bus, u32::from(a), u32::from(a.wrapping_add(1)))
            }
            0x7c => self.pc = self.indexed_indirect(bus),
            0xdc => {
                let a = self.read_opcode_word(bus);
                self.pc = self.read_word(bus, u32::from(a), u32::from(a.wrapping_add(1)));
                self.k = bus.read(u32::from(a.wrapping_add(2)))
            }
            0xfc => {
                let v = self.indexed_indirect(bus);
                self.push_word(bus, self.pc.wrapping_sub(1));
                self.pc = v
            }
            0x40 => {
                let f = self.pull_byte(bus);
                self.set_flags(f);
                self.cycles_used += 1;
                self.pc = self.pull_word(bus);
                self.k = self.pull_byte(bus)
            }
            0x60 => {
                self.check_rts_hook(false);
                self.pc = self.pull_word(bus).wrapping_add(1)
            }
            0x6b => {
                self.check_rts_hook(true);
                self.pc = self.pull_word(bus).wrapping_add(1);
                self.k = self.pull_byte(bus)
            }
            0x62 => {
                let d = self.read_opcode_word(bus) as i16;
                let v = self.pc.wrapping_add_signed(d);
                self.push_word(bus, v)
            }
            0x42 => {
                self.read_opcode(bus);
            }
            0xea => {}
            0xcb => self.waiting = true,
            0xdb => self.stopped = true,
            0xc2 => {
                let mask = self.read_opcode(bus);
                self.set_flags(self.flags() & !mask)
            }
            0xe2 => {
                let mask = self.read_opcode(bus);
                self.set_flags(self.flags() | mask)
            }
            0x44 | 0x54 => {
                let dest = self.read_opcode(bus);
                let src = self.read_opcode(bus);
                self.db = dest;
                let value = bus.read(u32::from(src) << 16 | u32::from(self.x));
                bus.write(u32::from(dest) << 16 | u32::from(self.y), value);
                self.a = self.a.wrapping_sub(1);
                if opcode == 0x44 {
                    self.x = self.x.wrapping_sub(1);
                    self.y = self.y.wrapping_sub(1)
                } else {
                    self.x = self.x.wrapping_add(1);
                    self.y = self.y.wrapping_add(1)
                }
                if self.a != 0xffff {
                    self.pc = self.pc.wrapping_sub(3)
                }
                if self.xf {
                    self.x &= 0xff;
                    self.y &= 0xff
                }
            }
            0x0a | 0x2a | 0x4a | 0x6a => self.acc_shift(opcode),
            0x1a => {
                self.a = if self.mf {
                    self.a & 0xff00 | self.a.wrapping_add(1) & 0xff
                } else {
                    self.a.wrapping_add(1)
                };
                self.set_zn(self.a, self.mf)
            }
            0x3a => {
                self.a = if self.mf {
                    self.a & 0xff00 | self.a.wrapping_sub(1) & 0xff
                } else {
                    self.a.wrapping_sub(1)
                };
                self.set_zn(self.a, self.mf)
            }
            0x88 => {
                self.y = if self.xf {
                    self.y.wrapping_sub(1) & 0xff
                } else {
                    self.y.wrapping_sub(1)
                };
                self.set_zn(self.y, self.xf)
            }
            0xc8 => {
                self.y = if self.xf {
                    self.y.wrapping_add(1) & 0xff
                } else {
                    self.y.wrapping_add(1)
                };
                self.set_zn(self.y, self.xf)
            }
            0xca => {
                self.x = if self.xf {
                    self.x.wrapping_sub(1) & 0xff
                } else {
                    self.x.wrapping_sub(1)
                };
                self.set_zn(self.x, self.xf)
            }
            0xe8 => {
                self.x = if self.xf {
                    self.x.wrapping_add(1) & 0xff
                } else {
                    self.x.wrapping_add(1)
                };
                self.set_zn(self.x, self.xf)
            }
            0x1b => self.sp = self.a,
            0x3b => {
                self.a = self.sp;
                self.set_zn(self.a, false)
            }
            0x5b => {
                self.dp = self.a;
                self.set_zn(self.dp, false)
            }
            0x7b => {
                self.a = self.dp;
                self.set_zn(self.a, false)
            }
            0x8a => {
                self.a = if self.mf {
                    self.a & 0xff00 | self.x & 0xff
                } else {
                    self.x
                };
                self.set_zn(self.a, self.mf)
            }
            0x98 => {
                self.a = if self.mf {
                    self.a & 0xff00 | self.y & 0xff
                } else {
                    self.y
                };
                self.set_zn(self.a, self.mf)
            }
            0xa8 => {
                self.y = if self.xf { self.a & 0xff } else { self.a };
                self.set_zn(self.y, self.xf)
            }
            0xaa => {
                self.x = if self.xf { self.a & 0xff } else { self.a };
                self.set_zn(self.x, self.xf)
            }
            0x9a => self.sp = self.x,
            0xba => {
                self.x = if self.xf { self.sp & 0xff } else { self.sp };
                self.set_zn(self.x, self.xf)
            }
            0x9b => {
                self.y = if self.xf { self.x & 0xff } else { self.x };
                self.set_zn(self.y, self.xf)
            }
            0xbb => {
                self.x = if self.xf { self.y & 0xff } else { self.y };
                self.set_zn(self.x, self.xf)
            }
            0x89 => {
                if self.mf {
                    let v = self.read_opcode(bus);
                    self.z = (self.a as u8 & v) == 0
                } else {
                    self.cycles_used += 1;
                    let v = self.read_opcode_word(bus);
                    self.z = self.a & v == 0
                }
            }
            0xeb => {
                let [lo, hi] = self.a.to_le_bytes();
                self.a = u16::from_le_bytes([hi, lo]);
                self.set_zn(u16::from(hi), true)
            }
            0xfb => {
                core::mem::swap(&mut self.c, &mut self.e);
                let f = self.flags();
                self.set_flags(f)
            }
            _ => unreachable!("all 256 opcodes are decoded: {opcode:02x}"),
        }
    }

    fn acc_shift(&mut self, opcode: u8) {
        let mask = if self.mf { 0xff } else { 0xffff };
        let old = u32::from(self.a & mask);
        let result = match opcode {
            0x0a => {
                self.c = old & (if self.mf { 0x80 } else { 0x8000 }) != 0;
                old << 1
            }
            0x2a => {
                let r = old << 1 | u32::from(self.c);
                self.c = r > u32::from(mask);
                r
            }
            0x4a => {
                self.c = old & 1 != 0;
                old >> 1
            }
            _ => {
                let c = old & 1 != 0;
                let r = old >> 1 | u32::from(self.c) << (if self.mf { 7 } else { 15 });
                self.c = c;
                r
            }
        } as u16
            & mask;
        self.a = if self.mf {
            self.a & 0xff00 | result
        } else {
            result
        };
        self.set_zn(self.a, self.mf)
    }
    fn check_rts_hook(&mut self, long: bool) {
        if self.sp_breakpoint != 0 && self.sp >= self.sp_breakpoint {
            assert_eq!(self.sp, self.sp_breakpoint);
            self.sp_breakpoint = 0;
            if let Some(hook) = self.hooked_function_rts {
                hook(long)
            }
        }
    }

    fn brk_patch(&mut self, bus: &mut impl Bus) {
        let address = (u32::from(self.k) << 16 | u32::from(self.pc)).wrapping_sub(1);
        match address {
            0x7b269 => {
                if self.x as u8 >= 28 {
                    self.pc = 0xb280
                }
                self.execute(bus, 0xe8)
            }
            0x5dec7 => {
                self.a = self.a & 0xff00 | u16::from(bus.read(0x5deb0 + u32::from(self.y as u8)));
                if bus.read(0xd90 + u32::from(self.x as u8)) == 2 {
                    self.pc = 0xdeea
                } else {
                    self.pc = self.pc.wrapping_add(2)
                }
            }
            0x9be5e => {
                self.a = self.a & 0xff00 | 224;
                bus.write(0xde0 + u32::from(self.y as u8), 0);
                self.pc = self.pc.wrapping_add(1)
            }
            0x1af9a4 => {
                self.a = self.a & 0xff00 | u16::from((self.a as u8).wrapping_add(4));
                self.c = false;
                self.pc = self.pc.wrapping_add(1)
            }
            0x1e8a46 => {
                self.a = self
                    .a
                    .wrapping_sub(u16::from(bus.read(0xe2)))
                    .wrapping_sub(u16::from(bus.read(8)))
                    .wrapping_add(12);
                self.pc = self.pc.wrapping_add(5)
            }
            0x1e8a52 => {
                self.a = self
                    .a
                    .wrapping_sub(u16::from(bus.read(0xe8)))
                    .wrapping_add(8)
                    .wrapping_sub(u16::from(bus.read(9)))
                    .wrapping_add(8);
                self.pc = self.pc.wrapping_add(7)
            }
            0x9a966 => {
                for i in 0..4 {
                    bus.write(0x72 + i, 0)
                }
                self.pc = self.pc.wrapping_add(1)
            }
            0x8f708 => {
                bus.write(0x75, 0);
                self.execute(bus, 0xc8)
            }
            0x1de0e5 => {
                if self.a as u8 >= 160 {
                    self.pc = 0xe164
                } else {
                    self.pc = self.pc.wrapping_add(1)
                }
            }
            0x6d0b6 | 0x6d0c6 => {
                self.c = self.a as u8 >= 4;
                self.a = self.a.wrapping_sub(4);
                self.pc = self.pc.wrapping_add(1)
            }
            0x1d8f29 | 0x1dc812 | 0x1ddbd3 | 0x1df856 | 0x6ed0b | 0x9b478 | 0x9b46c => {
                self.c = false;
                self.execute(bus, 0x69)
            }
            0x1e88da => {
                self.c = false;
                self.execute(bus, 0x65)
            }
            0x9b468 | 0x9b46a | 0x9b474 | 0x9b476 => {
                self.c = true;
                self.execute(bus, 0xe5)
            }
            0x9b60c => {
                self.c = true;
                self.execute(bus, 0xe9)
            }
            0x1dcdeb => {
                self.y = u16::from(bus.read(0xeb0 + u32::from(self.x as u8)));
                self.a = self.x
            }
            _ => panic!("unexpected BRK at {address:06x}"),
        }
    }
}
