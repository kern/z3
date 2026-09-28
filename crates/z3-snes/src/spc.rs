//! SPC700 CPU, behaviorally matching `snes/spc.c`.
use crate::bus::ApuBus;
use crate::saveload::{SaveLoad, SaveLoadError, bool_byte, get_u16, put_u16};

pub const SPC_SAVE_BLOCK_LEN: usize = 15;

pub const CYCLES_PER_OPCODE: [u8; 256] = [
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 4, 6, 8, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 6, 5, 2, 2, 4, 6,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 4, 5, 4, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 6, 5, 2, 2, 3, 8,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 4, 6, 6, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 4, 5, 2, 2, 4, 3,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 4, 5, 5, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 3, 6,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 2, 4, 5, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 12,
    5, 2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 2, 4, 4, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 3,
    4, 2, 8, 4, 5, 4, 5, 4, 7, 2, 5, 6, 4, 5, 2, 4, 9, 2, 8, 4, 5, 5, 6, 6, 7, 4, 5, 5, 5, 2, 2, 6,
    3, 2, 8, 4, 5, 3, 4, 3, 6, 2, 4, 5, 3, 4, 3, 4, 3, 2, 8, 4, 5, 4, 5, 5, 6, 3, 4, 5, 4, 2, 2, 4,
    3,
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Spc {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub h: bool,
    pub p: bool,
    pub b: bool,
    pub stopped: bool,
    pub cycles_used: u8,
}
#[derive(Clone, Copy)]
enum Alu {
    Or,
    And,
    Eor,
    Cmp,
    Adc,
    Sbc,
}

impl Spc {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn saveload(&mut self, io: &mut impl SaveLoad) -> Result<(), SaveLoadError> {
        let mut block = [0; SPC_SAVE_BLOCK_LEN];
        if !io.is_loading() {
            block[0] = self.a;
            block[1] = self.x;
            block[2] = self.y;
            block[3] = self.sp;
            put_u16(&mut block, 4, self.pc);
            let flags = [
                self.c,
                self.z,
                self.v,
                self.n,
                self.i,
                self.h,
                self.p,
                self.b,
                self.stopped,
            ];
            for (index, value) in flags.into_iter().enumerate() {
                block[6 + index] = bool_byte(value);
            }
        }
        io.transfer(&mut block)?;
        if io.is_loading() {
            self.a = block[0];
            self.x = block[1];
            self.y = block[2];
            self.sp = block[3];
            self.pc = get_u16(&block, 4);
            self.c = block[6] != 0;
            self.z = block[7] != 0;
            self.v = block[8] != 0;
            self.n = block[9] != 0;
            self.i = block[10] != 0;
            self.h = block[11] != 0;
            self.p = block[12] != 0;
            self.b = block[13] != 0;
            self.stopped = block[14] != 0;
        }
        Ok(())
    }
    pub fn reset(&mut self, b: &mut impl ApuBus) {
        *self = Self::default();
        self.pc = self.rw(b, 0xfffe, 0xffff)
    }
    pub fn run_opcode(&mut self, b: &mut impl ApuBus) -> u8 {
        self.cycles_used = 0;
        if self.stopped {
            return 1;
        }
        let o = self.ro(b);
        self.cycles_used = CYCLES_PER_OPCODE[o as usize];
        self.exec(b, o);
        self.cycles_used
    }
    pub fn flags(&self) -> u8 {
        self.c as u8
            | (self.z as u8) << 1
            | (self.i as u8) << 2
            | (self.h as u8) << 3
            | (self.b as u8) << 4
            | (self.p as u8) << 5
            | (self.v as u8) << 6
            | (self.n as u8) << 7
    }
    pub fn set_flags(&mut self, q: u8) {
        self.c = q & 1 != 0;
        self.z = q & 2 != 0;
        self.i = q & 4 != 0;
        self.h = q & 8 != 0;
        self.b = q & 16 != 0;
        self.p = q & 32 != 0;
        self.v = q & 64 != 0;
        self.n = q & 128 != 0
    }
    fn zn(&mut self, q: u8) {
        self.z = q == 0;
        self.n = q & 128 != 0
    }
    fn ro(&mut self, b: &mut impl ApuBus) -> u8 {
        let q = b.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        q
    }
    fn row(&mut self, b: &mut impl ApuBus) -> u16 {
        let l = self.ro(b);
        u16::from_le_bytes([l, self.ro(b)])
    }
    fn rw(&mut self, b: &mut impl ApuBus, l: u16, h: u16) -> u16 {
        let l = b.read(l);
        u16::from_le_bytes([l, b.read(h)])
    }
    fn ww(&mut self, b: &mut impl ApuBus, l: u16, h: u16, q: u16) {
        let [a, z] = q.to_le_bytes();
        b.write(l, a);
        b.write(h, z)
    }
    fn push(&mut self, b: &mut impl ApuBus, q: u8) {
        b.write(0x100 | self.sp as u16, q);
        self.sp = self.sp.wrapping_sub(1)
    }
    fn pull(&mut self, b: &mut impl ApuBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        b.read(0x100 | self.sp as u16)
    }
    fn pushw(&mut self, b: &mut impl ApuBus, q: u16) {
        let [l, h] = q.to_le_bytes();
        self.push(b, h);
        self.push(b, l)
    }
    fn pullw(&mut self, b: &mut impl ApuBus) -> u16 {
        let l = self.pull(b);
        u16::from_le_bytes([l, self.pull(b)])
    }
    fn branch(&mut self, q: u8, t: bool) {
        if t {
            self.cycles_used = self.cycles_used.wrapping_add(2);
            self.pc = self.pc.wrapping_add_signed(q as i8 as i16)
        }
    }
    fn base(&self) -> u16 {
        (self.p as u16) << 8
    }
    fn dp(&mut self, b: &mut impl ApuBus) -> u16 {
        self.ro(b) as u16 | self.base()
    }
    fn abs(&mut self, b: &mut impl ApuBus) -> u16 {
        self.row(b)
    }
    fn ind(&self) -> u16 {
        self.x as u16 | self.base()
    }
    fn idx(&mut self, b: &mut impl ApuBus) -> u16 {
        let q = self.ro(b).wrapping_add(self.x);
        let d = self.base();
        self.rw(b, q as u16 | d, q.wrapping_add(1) as u16 | d)
    }
    fn imm(&mut self) -> u16 {
        let q = self.pc;
        self.pc = self.pc.wrapping_add(1);
        q
    }
    fn dpx(&mut self, b: &mut impl ApuBus) -> u16 {
        self.ro(b).wrapping_add(self.x) as u16 | self.base()
    }
    fn dpy(&mut self, b: &mut impl ApuBus) -> u16 {
        self.ro(b).wrapping_add(self.y) as u16 | self.base()
    }
    fn abx(&mut self, b: &mut impl ApuBus) -> u16 {
        let q = self.row(b);
        q.wrapping_add(self.x as u16)
    }
    fn aby(&mut self, b: &mut impl ApuBus) -> u16 {
        let q = self.row(b);
        q.wrapping_add(self.y as u16)
    }
    fn idy(&mut self, b: &mut impl ApuBus) -> u16 {
        let q = self.ro(b);
        let d = self.base();
        let a = self.rw(b, q as u16 | d, q.wrapping_add(1) as u16 | d);
        a.wrapping_add(self.y as u16)
    }
    fn dpdp(&mut self, b: &mut impl ApuBus) -> (u16, u16) {
        let s = self.ro(b) as u16 | self.base();
        let d = self.ro(b) as u16 | self.base();
        (d, s)
    }
    fn dpimm(&mut self, b: &mut impl ApuBus) -> (u16, u16) {
        let s = self.imm();
        let d = self.ro(b) as u16 | self.base();
        (d, s)
    }
    fn ii(&self) -> (u16, u16) {
        (self.x as u16 | self.base(), self.y as u16 | self.base())
    }
    fn abit(&mut self, b: &mut impl ApuBus) -> (u16, u8) {
        let q = self.row(b);
        (q & 0x1fff, (q >> 13) as u8)
    }
    fn dpw(&mut self, b: &mut impl ApuBus) -> (u16, u16) {
        let q = self.ro(b);
        let d = self.base();
        (q as u16 | d, q.wrapping_add(1) as u16 | d)
    }
    fn indp(&mut self) -> u16 {
        let q = self.ind();
        self.x = self.x.wrapping_add(1);
        q
    }
    fn add(&mut self, l: u8, r: u8) -> u8 {
        let c = self.c as u16;
        let q = l as u16 + r as u16 + c;
        self.v = (l & 128) == (r & 128) && (r & 128) != (q as u8 & 128);
        self.h = (l & 15) as u16 + (r & 15) as u16 + c > 15;
        self.c = q > 255;
        q as u8
    }
    fn cmpreg(&mut self, l: u8, r: u8) {
        let q = l as u16 + (r ^ 255) as u16 + 1;
        self.c = q > 255;
        self.zn(q as u8)
    }
    fn alua(&mut self, b: &mut impl ApuBus, o: Alu, a: u16) {
        let r = b.read(a);
        match o {
            Alu::Or => self.a |= r,
            Alu::And => self.a &= r,
            Alu::Eor => self.a ^= r,
            Alu::Cmp => {
                self.cmpreg(self.a, r);
                return;
            }
            Alu::Adc => self.a = self.add(self.a, r),
            Alu::Sbc => self.a = self.add(self.a, r ^ 255),
        }
        self.zn(self.a)
    }
    fn alum(&mut self, b: &mut impl ApuBus, o: Alu, d: u16, s: u16) {
        let r = b.read(s);
        let l = b.read(d);
        let q = match o {
            Alu::Or => l | r,
            Alu::And => l & r,
            Alu::Eor => l ^ r,
            Alu::Cmp => {
                self.cmpreg(l, r);
                return;
            }
            Alu::Adc => self.add(l, r),
            Alu::Sbc => self.add(l, r ^ 255),
        };
        b.write(d, q);
        self.zn(q)
    }
    fn shift(&mut self, b: &mut impl ApuBus, a: u16, k: u8) {
        let q = b.read(a);
        let (r, c) = match k {
            0 => (q.wrapping_shl(1), q & 128 != 0),
            1 => (q.wrapping_shl(1) | self.c as u8, q & 128 != 0),
            2 => (q >> 1, q & 1 != 0),
            _ => (q >> 1 | (self.c as u8) << 7, q & 1 != 0),
        };
        self.c = c;
        b.write(a, r);
        self.zn(r)
    }
    fn incm(&mut self, b: &mut impl ApuBus, a: u16, inc: bool) {
        let q = if inc {
            b.read(a).wrapping_add(1)
        } else {
            b.read(a).wrapping_sub(1)
        };
        b.write(a, q);
        self.zn(q)
    }
    fn load(&mut self, b: &mut impl ApuBus, a: u16, r: u8) {
        let q = b.read(a);
        match r {
            0 => self.a = q,
            1 => self.x = q,
            _ => self.y = q,
        };
        self.zn(q)
    }
    fn store(&mut self, b: &mut impl ApuBus, a: u16, q: u8) {
        let _ = b.read(a);
        b.write(a, q)
    }
    fn alug(&mut self, b: &mut impl ApuBus, o: u8) -> bool {
        let h = o >> 4;
        let l = o & 15;
        let op = match h & 14 {
            0 => Alu::Or,
            2 => Alu::And,
            4 => Alu::Eor,
            6 => Alu::Cmp,
            8 => Alu::Adc,
            10 => Alu::Sbc,
            _ => return false,
        };
        if !(4..=9).contains(&l) {
            return false;
        }
        match l {
            4..=7 => {
                let a = match (h & 1, l) {
                    (0, 4) => self.dp(b),
                    (0, 5) => self.abs(b),
                    (0, 6) => self.ind(),
                    (0, _) => self.idx(b),
                    (_, 4) => self.dpx(b),
                    (_, 5) => self.abx(b),
                    (_, 6) => self.aby(b),
                    (_, _) => self.idy(b),
                };
                self.alua(b, op, a);
            }
            8 if h & 1 == 0 => {
                let a = self.imm();
                self.alua(b, op, a);
            }
            8 => {
                let (d, s) = self.dpimm(b);
                self.alum(b, op, d, s);
            }
            9 => {
                let (d, s) = if h & 1 == 0 { self.dpdp(b) } else { self.ii() };
                self.alum(b, op, d, s);
            }
            _ => return false,
        }
        true
    }
    #[allow(clippy::too_many_lines)]
    fn exec(&mut self, b: &mut impl ApuBus, o: u8) {
        if o & 15 == 1 {
            self.pushw(b, self.pc);
            let a = 0xffdeu16.wrapping_sub(2 * (o >> 4) as u16);
            self.pc = self.rw(b, a, a + 1);
            return;
        }
        if o & 31 == 2 {
            let a = self.dp(b);
            let q = b.read(a) | (1 << (o >> 5));
            b.write(a, q);
            return;
        }
        if o & 31 == 18 {
            let a = self.dp(b);
            let q = b.read(a) & !(1 << (o >> 5));
            b.write(a, q);
            return;
        }
        if o & 31 == 3 || o & 31 == 19 {
            let a = self.dp(b);
            let q = b.read(a);
            let r = self.ro(b);
            let s = q & (1 << (o >> 5)) != 0;
            self.branch(r, if o & 16 == 0 { s } else { !s });
            return;
        }
        if self.alug(b, o) {
            return;
        }
        match o {
            0x00 => {}
            0x0a | 0x2a | 0x4a | 0x6a | 0x8a | 0xaa => {
                let (a, n) = self.abit(b);
                let q = (b.read(a) >> n) & 1 != 0;
                self.c = match o {
                    0x0a => self.c | q,
                    0x2a => self.c | !q,
                    0x4a => self.c & q,
                    0x6a => self.c & !q,
                    0x8a => self.c ^ q,
                    _ => q,
                }
            }
            0x0b | 0x2b | 0x4b | 0x6b => {
                let a = self.dp(b);
                self.shift(b, a, o >> 5)
            }
            0x0c | 0x2c | 0x4c | 0x6c => {
                let a = self.abs(b);
                self.shift(b, a, o >> 5)
            }
            0x1b | 0x3b | 0x5b | 0x7b => {
                let a = self.dpx(b);
                self.shift(b, a, o >> 5)
            }
            0x0d => self.push(b, self.flags()),
            0x2d => self.push(b, self.a),
            0x4d => self.push(b, self.x),
            0x6d => self.push(b, self.y),
            0x0e | 0x4e => {
                let a = self.abs(b);
                let q = b.read(a);
                self.zn(self.a.wrapping_sub(q));
                b.write(a, if o == 0x0e { q | self.a } else { q & !self.a })
            }
            0x0f => {
                self.pushw(b, self.pc);
                self.push(b, self.flags());
                self.i = false;
                self.b = true;
                self.pc = self.rw(b, 0xffde, 0xffdf)
            }
            0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xb0 | 0xd0 | 0xf0 => {
                let r = self.ro(b);
                let t = match o {
                    0x10 => !self.n,
                    0x30 => self.n,
                    0x50 => !self.v,
                    0x70 => self.v,
                    0x90 => !self.c,
                    0xb0 => self.c,
                    0xd0 => !self.z,
                    _ => self.z,
                };
                self.branch(r, t)
            }
            0x1a | 0x3a => {
                let (l, h) = self.dpw(b);
                let q = self.rw(b, l, h);
                let r = if o == 0x1a {
                    q.wrapping_sub(1)
                } else {
                    q.wrapping_add(1)
                };
                self.z = r == 0;
                self.n = r & 0x8000 != 0;
                self.ww(b, l, h, r)
            }
            0x1c | 0x3c | 0x5c | 0x7c => {
                let q = self.a;
                let (r, c) = match o {
                    0x1c => (q.wrapping_shl(1), q & 128 != 0),
                    0x3c => (q.wrapping_shl(1) | self.c as u8, q & 128 != 0),
                    0x5c => (q >> 1, q & 1 != 0),
                    _ => (q >> 1 | (self.c as u8) << 7, q & 1 != 0),
                };
                self.a = r;
                self.c = c;
                self.zn(r)
            }
            0x1d | 0x3d => {
                self.x = if o == 0x1d {
                    self.x.wrapping_sub(1)
                } else {
                    self.x.wrapping_add(1)
                };
                self.zn(self.x)
            }
            0x1e | 0x3e | 0xc8 => {
                let a = match o {
                    0x1e => self.abs(b),
                    0x3e => self.dp(b),
                    _ => self.imm(),
                };
                let q = b.read(a);
                self.cmpreg(self.x, q)
            }
            0x1f => {
                let a = self.row(b).wrapping_add(self.x as u16);
                self.pc = self.rw(b, a, a.wrapping_add(1))
            }
            0x20 => self.p = false,
            0x40 => self.p = true,
            0x60 => self.c = false,
            0x80 => self.c = true,
            0xa0 => self.i = true,
            0xc0 => self.i = false,
            0xe0 => {
                self.v = false;
                self.h = false
            }
            0x2e | 0xde => {
                let a = if o == 0x2e { self.dp(b) } else { self.dpx(b) };
                let q = b.read(a);
                let r = self.ro(b);
                self.branch(r, self.a != q)
            }
            0x2f => {
                let q = self.ro(b);
                self.pc = self.pc.wrapping_add_signed(q as i8 as i16)
            }
            0x3f => {
                let q = self.row(b);
                self.pushw(b, self.pc);
                self.pc = q
            }
            0x4f => {
                let q = self.ro(b);
                self.pushw(b, self.pc);
                self.pc = 0xff00 | q as u16
            }
            0x5a => {
                let (l, h) = self.dpw(b);
                let q = self.rw(b, l, h);
                let ya = u16::from_le_bytes([self.a, self.y]);
                let r = ya as u32 + (q ^ 0xffff) as u32 + 1;
                self.c = r > 0xffff;
                self.z = r as u16 == 0;
                self.n = r & 0x8000 != 0
            }
            0x5d => {
                self.x = self.a;
                self.zn(self.x)
            }
            0x5e | 0x7e | 0xad => {
                let a = match o {
                    0x5e => self.abs(b),
                    0x7e => self.dp(b),
                    _ => self.imm(),
                };
                let q = b.read(a);
                self.cmpreg(self.y, q)
            }
            0x5f => self.pc = self.row(b),
            0x6e => {
                let a = self.dp(b);
                let q = b.read(a).wrapping_sub(1);
                b.write(a, q);
                let r = self.ro(b);
                self.branch(r, q != 0)
            }
            0x6f => self.pc = self.pullw(b),
            0x7a | 0x9a => {
                let (l, h) = self.dpw(b);
                let m = self.rw(b, l, h);
                let q = if o == 0x9a { m ^ 0xffff } else { m };
                let ya = u16::from_le_bytes([self.a, self.y]);
                let r = ya as u32 + q as u32 + (o == 0x9a) as u32;
                self.v = (ya & 0x8000) == (q & 0x8000) && (q & 0x8000) != (r as u16 & 0x8000);
                self.h = (ya & 0xfff) as u32 + (q & 0xfff) as u32 + 1 > 0xfff;
                self.c = r > 0xffff;
                let r = r as u16;
                self.z = r == 0;
                self.n = r & 0x8000 != 0;
                [self.a, self.y] = r.to_le_bytes()
            }
            0x7d => {
                self.a = self.x;
                self.zn(self.a)
            }
            0x7f => {
                let q = self.pull(b);
                self.set_flags(q);
                self.pc = self.pullw(b)
            }
            0x8b | 0x9b | 0xab | 0xbb => {
                let a = if o & 16 == 0 { self.dp(b) } else { self.dpx(b) };
                self.incm(b, a, o & 32 != 0)
            }
            0x8c | 0xac => {
                let a = self.abs(b);
                self.incm(b, a, o == 0xac)
            }
            0x8d => {
                let a = self.imm();
                self.load(b, a, 2)
            }
            0x8e => {
                let q = self.pull(b);
                self.set_flags(q)
            }
            0x8f => {
                let (d, s) = self.dpimm(b);
                let q = b.read(s);
                self.store(b, d, q)
            }
            0x9c | 0xbc => {
                self.a = if o == 0x9c {
                    self.a.wrapping_sub(1)
                } else {
                    self.a.wrapping_add(1)
                };
                self.zn(self.a)
            }
            0x9d => {
                self.x = self.sp;
                self.zn(self.x)
            }
            0x9e => {
                let q = u16::from_le_bytes([self.a, self.y]);
                let (r, m) = if self.x == 0 {
                    (0xffff, self.a as u16)
                } else {
                    (q / self.x as u16, q % self.x as u16)
                };
                self.v = r > 255;
                self.h = self.x & 15 <= self.y & 15;
                self.a = r as u8;
                self.y = m as u8;
                self.zn(self.a)
            }
            0x9f => {
                self.a = self.a.rotate_left(4);
                self.zn(self.a)
            }
            0xae => self.a = self.pull(b),
            0xce => self.x = self.pull(b),
            0xee => self.y = self.pull(b),
            0xaf => {
                let a = self.indp();
                b.write(a, self.a)
            }
            0xba => {
                let (l, h) = self.dpw(b);
                let q = self.rw(b, l, h);
                [self.a, self.y] = q.to_le_bytes();
                self.z = q == 0;
                self.n = q & 0x8000 != 0
            }
            0xbd => self.sp = self.x,
            0xbe => {
                if self.a > 0x99 || !self.c {
                    self.a = self.a.wrapping_sub(0x60);
                    self.c = false
                }
                if self.a & 15 > 9 || !self.h {
                    self.a = self.a.wrapping_sub(6)
                }
                self.zn(self.a)
            }
            0xbf => {
                let a = self.indp();
                self.load(b, a, 0)
            }
            0xc4 | 0xc5 | 0xc6 | 0xc7 | 0xd4 | 0xd5 | 0xd6 | 0xd7 => {
                let a = match o {
                    0xc4 => self.dp(b),
                    0xc5 => self.abs(b),
                    0xc6 => self.ind(),
                    0xc7 => self.idx(b),
                    0xd4 => self.dpx(b),
                    0xd5 => self.abx(b),
                    0xd6 => self.aby(b),
                    _ => self.idy(b),
                };
                self.store(b, a, self.a)
            }
            0xc9 => {
                let a = self.abs(b);
                self.store(b, a, self.x)
            }
            0xca => {
                let (a, n) = self.abit(b);
                let q = b.read(a) & !(1 << n) | (self.c as u8) << n;
                b.write(a, q)
            }
            0xcb | 0xcc | 0xdb => {
                let a = match o {
                    0xcb => self.dp(b),
                    0xcc => self.abs(b),
                    _ => self.dpx(b),
                };
                self.store(b, a, self.y)
            }
            0xcd => {
                let a = self.imm();
                self.load(b, a, 1)
            }
            0xcf => {
                let q = self.a as u16 * self.y as u16;
                [self.a, self.y] = q.to_le_bytes();
                self.zn(self.y)
            }
            0xd8 | 0xd9 => {
                let a = if o == 0xd8 { self.dp(b) } else { self.dpy(b) };
                self.store(b, a, self.x)
            }
            0xda => {
                let (l, h) = self.dpw(b);
                let _ = b.read(l);
                b.write(l, self.a);
                b.write(h, self.y)
            }
            0xdc => {
                self.y = self.y.wrapping_sub(1);
                self.zn(self.y)
            }
            0xdd => {
                self.a = self.y;
                self.zn(self.a)
            }
            0xdf => {
                if self.a > 0x99 || self.c {
                    self.a = self.a.wrapping_add(0x60);
                    self.c = true
                }
                if self.a & 15 > 9 || self.h {
                    self.a = self.a.wrapping_add(6)
                }
                self.zn(self.a)
            }
            0xe4 | 0xe5 | 0xe6 | 0xe7 | 0xe8 | 0xf4 | 0xf5 | 0xf6 | 0xf7 => {
                let a = match o {
                    0xe4 => self.dp(b),
                    0xe5 => self.abs(b),
                    0xe6 => self.ind(),
                    0xe7 => self.idx(b),
                    0xe8 => self.imm(),
                    0xf4 => self.dpx(b),
                    0xf5 => self.abx(b),
                    0xf6 => self.aby(b),
                    _ => self.idy(b),
                };
                self.load(b, a, 0)
            }
            0xe9 | 0xf8 | 0xf9 => {
                let a = match o {
                    0xe9 => self.abs(b),
                    0xf8 => self.dp(b),
                    _ => self.dpy(b),
                };
                self.load(b, a, 1)
            }
            0xea => {
                let (a, n) = self.abit(b);
                let q = b.read(a) ^ (1 << n);
                b.write(a, q)
            }
            0xeb | 0xec | 0xfb => {
                let a = match o {
                    0xeb => self.dp(b),
                    0xec => self.abs(b),
                    _ => self.dpx(b),
                };
                self.load(b, a, 2)
            }
            0xed => self.c = !self.c,
            0xef | 0xff => self.stopped = true,
            0xfa => {
                let (d, s) = self.dpdp(b);
                let q = b.read(s);
                b.write(d, q)
            }
            0xfc => {
                self.y = self.y.wrapping_add(1);
                self.zn(self.y)
            }
            0xfd => {
                self.y = self.a;
                self.zn(self.y)
            }
            0xfe => {
                self.y = self.y.wrapping_sub(1);
                let q = self.ro(b);
                self.branch(q, self.y != 0)
            }
            _ => unreachable!("SPC opcode {o:02x}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mem(Box<[u8; 65536]>);
    impl Default for Mem {
        fn default() -> Self {
            Self(Box::new([0; 65536]))
        }
    }
    impl ApuBus for Mem {
        fn read(&mut self, a: u16) -> u8 {
            self.0[a as usize]
        }
        fn write(&mut self, a: u16, q: u8) {
            self.0[a as usize] = q
        }
    }
    fn run(code: &[u8], f: impl FnOnce(&mut Spc, &mut Mem)) -> (Spc, Mem, u8) {
        let mut m = Mem::default();
        m.0[..code.len()].copy_from_slice(code);
        let mut s = Spc::new();
        f(&mut s, &mut m);
        let c = s.run_opcode(&mut m);
        (s, m, c)
    }

    #[test]
    fn all_opcodes_dispatch() {
        for o in 0u8..=255 {
            let (s, _, _) = run(&[o, 0, 0], |s, _| s.x = 1);
            assert!(s.cycles_used > 0, "{o:02x}")
        }
    }
    #[test]
    fn adc_edges() {
        let cases = [
            (0x7f, 1, false, 0x80, false, true, true),
            (0xff, 0, true, 0, true, false, true),
            (0x0f, 0, true, 0x10, false, false, true),
        ];
        for (l, r, ci, out, c, v, h) in cases {
            let (s, _, cy) = run(&[0x88, r], |s, _| {
                s.a = l;
                s.c = ci
            });
            assert_eq!((s.a, s.c, s.v, s.h), (out, c, v, h));
            assert_eq!(cy, CYCLES_PER_OPCODE[0x88])
        }
    }
    #[test]
    fn direct_page_pointer_wrap() {
        let (s, _, _) = run(&[0xe7, 0xff], |s, m| {
            s.p = true;
            s.x = 1;
            m.0[0x100] = 0x34;
            m.0[0x101] = 0x12;
            m.0[0x1234] = 0x80
        });
        assert_eq!(s.a, 0x80);
        assert!(s.n)
    }
    #[test]
    fn call_and_branch_wrap() {
        let (s, m, c) = run(&[0x3f, 0x34, 0x12], |s, _| s.sp = 0);
        assert_eq!((s.pc, s.sp, c), (0x1234, 0xfe, 8));
        assert_eq!((m.0[0x100], m.0[0x1ff]), (0, 3));
        let (s, _, c) = run(&[0x2f, 0xfe], |_, _| {});
        assert_eq!((s.pc, c), (0, 4))
    }
    #[test]
    fn decimal_adjust_wrap() {
        let (s, _, _) = run(&[0xdf], |s, _| {
            s.a = 0xfa;
            s.c = true;
            s.h = true
        });
        assert_eq!(s.a, 0x60);
        assert!(s.c);
        let (s, _, _) = run(&[0xbe], |s, _| {
            s.a = 0;
            s.c = false;
            s.h = false
        });
        assert_eq!(s.a, 0x9a);
        assert!(!s.c)
    }
    #[test]
    fn flags_round_trip() {
        for q in 0u8..=255 {
            let mut s = Spc::new();
            s.set_flags(q);
            assert_eq!(s.flags(), q)
        }
    }
}
