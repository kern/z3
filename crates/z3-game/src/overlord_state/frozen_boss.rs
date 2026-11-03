//! Frozen original dc325d5 raw boss helpers; kept independent of domain APIs.
use crate::{ram::Ram, sprite_main_bank2a::SpriteMainBank2aRuntime};
const OVERLORD_X_LO: usize = 0xb08;
const OVERLORD_GEN1: usize = 0xb28;
const OVERLORD_GEN2: usize = 0xb30;
const FRAME_COUNTER: usize = 0x1a;
const SOUND_EFFECT_2: usize = 0x12f;
pub(super) struct FrozenBoss<'a, 'r, R> {
    pub ram: &'a mut Ram<'r>,
    pub runtime: &'a mut R,
}
impl<R: SpriteMainBank2aRuntime> FrozenBoss<'_, '_, R> {
    pub fn helmasaur_king_initialize(&mut self, k: usize) {
        self.ram.write_u8(OVERLORD_GEN1 + 7, 0x30);
        self.ram.write_u8(OVERLORD_GEN1 + 5, 0x80);
        self.ram.write_u8(OVERLORD_GEN1 + 6, 0);
        self.ram.write_u8(OVERLORD_GEN2, 0);
        self.ram.write_u8(OVERLORD_GEN2 + 3, 0);
        self.ram.write_u8(OVERLORD_GEN2 + 1, 0);
        self.ram.write_u8(OVERLORD_GEN2 + 2, 0);
        self.helmasaur_king_reinitialize(k);
    }

    pub fn helmasaur_king_reinitialize(&mut self, k: usize) {
        const T: [u8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 6, 5, 4,
            3, 2, 1,
        ];
        let t = self.ram.sprite(k).subtype2() as usize;
        for i in (0..4).rev() {
            self.ram.write_u8(OVERLORD_X_LO + i, T[(t + i * 8) & 31]);
        }
    }

    pub fn helmasaur_king_swing_tail(&mut self, k: usize) {
        self.ram.write_u8(
            OVERLORD_X_LO + 4,
            self.ram.read_u8(OVERLORD_X_LO + 4).wrapping_add(1),
        );
        self.helmasaur_king_reinitialize(k);
        let mask = if self.ram.sprite(k).animation_clock() != 0 {
            0
        } else {
            1
        };
        if self.ram.read_u8(FRAME_COUNTER) & mask == 0 {
            let j = self.ram.sprite(k).scratch_d() & 1;
            let v = self
                .ram
                .read_u8(OVERLORD_GEN2)
                .wrapping_add(if j != 0 { 255 } else { 1 });
            self.ram.write_u8(OVERLORD_GEN2, v);
            if v == [16u8, 240][j as usize] {
                self.ram
                    .set_sprite_scratch_d(k, self.ram.sprite(k).scratch_d().wrapping_add(1));
            }
            self.ram.write_u16(
                OVERLORD_GEN1 + 5,
                self.ram
                    .read_u16(OVERLORD_GEN1 + 5)
                    .wrapping_add_signed(v as i8 as i16),
            );
        }
        let a = self.ram.sprite(k).animation_clock();
        if a == 0 {
            return;
        }
        if self.ram.read_u8(OVERLORD_GEN2) == 0 {
            self.runtime.sfx3_with_pan(self.ram, k, 6);
        }
        if a == 2 {
            let j = self.ram.sprite(k).head_direction();
            let v = self
                .ram
                .read_u16(OVERLORD_GEN2 + 1)
                .wrapping_add_signed(if j != 0 { -4 } else { 4 });
            self.ram.write_u16(OVERLORD_GEN2 + 1, v);
            if v as u8 == if j != 0 { 132 } else { 124 } {
                self.ram.set_sprite_animation_clock(k, 3);
            }
            self.ram.write_u8(
                OVERLORD_GEN1 + 7,
                self.ram.read_u8(OVERLORD_GEN1 + 7).wrapping_add(3),
            );
        } else if a == 3 {
            let j = self.ram.sprite(k).head_direction() ^ 1;
            let v = self
                .ram
                .read_u16(OVERLORD_GEN2 + 1)
                .wrapping_add_signed(if j != 0 { -4 } else { 4 });
            self.ram.write_u16(OVERLORD_GEN2 + 1, v);
            if v == 0 {
                self.ram.set_sprite_animation_clock(k, 0);
            }
            self.ram.write_u8(
                OVERLORD_GEN1 + 7,
                self.ram.read_u8(OVERLORD_GEN1 + 7).wrapping_sub(3),
            );
        } else if self.ram.read_u8(OVERLORD_GEN2) | self.ram.sprite(k).aux3_delay() == 0 {
            let hd = self.ram.read_u8(OVERLORD_GEN1 + 6) & 1;
            self.ram.set_sprite_head_direction(k, hd);
            let (r, _) = self.runtime.is_right_of_link(self.ram, k);
            if (r ^ 1) == hd {
                self.ram.set_sprite_animation_clock(k, 2);
                let p = self.runtime.calculate_sfx_pan(self.ram, k);
                self.ram.write_u8(SOUND_EFFECT_2, p | 0x26);
            }
        }
    }

    pub fn trinexx_wag_tail(&mut self, _k: usize) {
        if self.ram.read_u8(OVERLORD_X_LO + 5) == 0 {
            let v = self.ram.read_u8(OVERLORD_X_LO + 4).wrapping_add(1);
            self.ram.write_u8(OVERLORD_X_LO + 4, v);
            if v & 3 == 0 {
                let j = self.ram.read_u8(OVERLORD_X_LO + 3) & 1;
                let p =
                    self.ram
                        .read_u8(OVERLORD_X_LO + 2)
                        .wrapping_add(if j != 0 { 255 } else { 1 });
                self.ram.write_u8(OVERLORD_X_LO + 2, p);
                if p == if j != 0 { 0 } else { 6 } {
                    self.ram.write_u8(
                        OVERLORD_X_LO + 3,
                        self.ram.read_u8(OVERLORD_X_LO + 3).wrapping_add(1),
                    );
                    self.ram.write_u8(OVERLORD_X_LO + 5, 8);
                }
            }
        } else {
            self.ram.write_u8(
                OVERLORD_X_LO + 5,
                self.ram.read_u8(OVERLORD_X_LO + 5).wrapping_sub(1),
            );
        }
    }
}
