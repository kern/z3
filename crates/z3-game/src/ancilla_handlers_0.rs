//! Local one-for-one translation of `src/ancilla.c` handlers and constructors.
//!
//! Genuine sprite, tile, graphics, and overworld calls cross the explicit
//! [`AncillaRuntime`] boundary; ancilla-owned state transitions remain local.

use super::{Ancilla, AncillaHitBox, AncillaRuntime, PFLAGS, ProjectSpeed};
use crate::sound::{SoundPort, SoundRequest};
use crate::{misc, oam};
use z3_core::c_types::PairU8;

const ANCILLA_ALLOC_ROTATE: usize = 0x3c4;

const RECEIVE_ITEM_SIZE: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
const RECEIVE_ITEM_FLAGS: [u8; 76] = [
    5, 0xff, 5, 5, 5, 5, 5, 1, 2, 1, 1, 1, 2, 2, 2, 4, 4, 4, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 4, 4, 2,
    1, 6, 1, 2, 1, 2, 2, 1, 2, 2, 4, 1, 1, 4, 2, 1, 4, 2, 2, 4, 4, 4, 2, 1, 4, 1, 2, 2, 1, 2, 2, 1,
    1, 4, 4, 1, 2, 2, 4, 4, 4, 2, 5, 2, 1,
];
const RECEIVE_ITEM_GFX: [u8; 76] = [
    6, 0x18, 0x18, 0x18, 0x2d, 0x20, 0x2e, 9, 9, 0x0a, 8, 5, 0x10, 0x0b, 0x2c, 0x1b, 0x1a, 0x1c,
    0x14, 0x19, 0x0c, 7, 0x1d, 0x2f, 7, 0x15, 0x12, 0x0d, 0x0d, 0x0e, 0x11, 0x17, 0x28, 0x27, 4, 4,
    0x0f, 0x16, 3, 0x13, 1, 0x1e, 0x10, 0, 0, 0, 0, 0, 0, 0x30, 0x22, 0x21, 0x24, 0x24, 0x24, 0x23,
    0x23, 0x23, 0x29, 0x2a, 0x2c, 0x2b, 3, 3, 0x34, 0x35, 0x31, 0x33, 2, 0x32, 0x36, 0x37, 0x2c, 6,
    0x0c, 0x38,
];

const TILE_COLL0_ATTRS: [u8; 256] = [
    0, 1, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 3, 3,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];
const TILE_COLL_ATTRS: [u8; 256] = [
    0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 3, 3,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

const SPIN_CHAR: [u8; 32] = [
    0x92, 0xff, 0xff, 0xff, 0x8c, 0x8c, 0x8c, 0x8c, 0xd6, 0xd6, 0xd6, 0xd6, 0x93, 0x93, 0x93, 0x93,
    0xd6, 0xd6, 0xd6, 0xd6, 0xd7, 0xff, 0xff, 0xff, 0x80, 0xff, 0xff, 0xff, 0x22, 0xff, 0xff, 0xff,
];
const SPIN_FLAGS: [u8; 29] = [
    0x22, 0xff, 0xff, 0xff, 0x22, 0x62, 0xa2, 0xe2, 0x24, 0x64, 0xa4, 0xe4, 0x22, 0x62, 0xa2, 0xe2,
    0x22, 0x62, 0xa2, 0xe2, 0x22, 0xff, 0xff, 0xff, 0x22, 0xff, 0xff, 0xff, 0xfc,
];
const SPIN_Y: [i8; 29] = [
    -4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -4, 0, 0, 0, -4, 0, 0, 0,
    -4,
];
const SPIN_X: [i16; 29] = [
    -4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -4, 0, 0, 0, -4, 0, 0, 0,
    0x11a5,
];

const QUAKE_ITEMS: [(i8, i8, u8); 151] = [
    (0, -16, 0x00),
    (0, -16, 0x01),
    (0, -16, 0x02),
    (0, -16, 0x03),
    (0, -16, 0x43),
    (0, -16, 0x42),
    (0, -16, 0x41),
    (0, -16, 0x40),
    (0, -16, 0x40),
    (14, -8, 0x84),
    (29, -8, 0x44),
    (13, -7, 0x84),
    (31, -7, 0x44),
    (47, -4, 0x84),
    (49, -11, 0x06),
    (63, -5, 0x44),
    (47, -4, 0x84),
    (36, -17, 0x08),
    (49, -11, 0x06),
    (63, -5, 0x44),
    (78, 4, 0x08),
    (22, -31, 0x08),
    (36, -17, 0x08),
    (78, 4, 0x08),
    (93, 20, 0x08),
    (7, -46, 0x08),
    (23, -45, 0x48),
    (22, -31, 0x08),
    (93, 20, 0x08),
    (93, 36, 0x48),
    (-7, -61, 0x08),
    (37, -59, 0x48),
    (7, -46, 0x08),
    (23, -45, 0x48),
    (93, 36, 0x48),
    (93, 52, 0x08),
    (-22, -75, 0x08),
    (47, -74, 0x01),
    (-8, -61, 0x08),
    (36, -60, 0x48),
    (93, 52, 0x08),
    (108, 67, 0x08),
    (-37, -90, 0x08),
    (-22, -75, 0x08),
    (47, -74, 0x01),
    (59, -62, 0x81),
    (108, 67, 0x08),
    (121, 80, 0x08),
    (-44, -104, 0xc9),
    (-37, -90, 0x08),
    (73, -74, 0x48),
    (59, -62, 0x81),
    (121, 80, 0x08),
    (-44, -120, 0x09),
    (-44, -104, 0xc9),
    (87, -89, 0x48),
    (73, -74, 0x48),
    (-44, -120, 0x09),
    (102, -104, 0x48),
    (87, -89, 0x48),
    (102, -104, 0x48),
    (87, -89, 0x48),
    (112, -116, 0x48),
    (102, -104, 0x48),
    (112, -116, 0x48),
    (-13, -16, 0x00),
    (-13, -16, 0x01),
    (-13, -16, 0x02),
    (-13, -16, 0x03),
    (-11, -16, 0x43),
    (-11, -16, 0x42),
    (-11, -16, 0x41),
    (-11, -16, 0x40),
    (-24, -10, 0x04),
    (-38, -18, 0x08),
    (-24, -10, 0x04),
    (-40, -7, 0xc4),
    (-45, -33, 0xc9),
    (-38, -18, 0x08),
    (-57, -7, 0x04),
    (-40, -7, 0xc4),
    (-48, -45, 0x07),
    (-45, -33, 0xc9),
    (-57, -7, 0x04),
    (-71, 2, 0x48),
    (-48, -45, 0x06),
    (-71, 2, 0x48),
    (-70, 18, 0x08),
    (-48, -45, 0x05),
    (-70, 18, 0x08),
    (-56, 33, 0x08),
    (-48, -45, 0x07),
    (-54, 34, 0x08),
    (-54, 49, 0x88),
    (-48, -45, 0x06),
    (-54, 49, 0x88),
    (-69, 64, 0x88),
    (-48, -45, 0x07),
    (-69, 64, 0x88),
    (-85, 73, 0xc4),
    (-48, -45, 0x05),
    (-101, 73, 0x04),
    (-85, 73, 0xc4),
    (-60, -53, 0x08),
    (-48, -45, 0x06),
    (-101, 73, 0x04),
    (-116, 77, 0xc4),
    (-75, -67, 0x08),
    (-60, -53, 0x08),
    (-128, 76, 0x04),
    (-116, 77, 0xc4),
    (-90, -82, 0x08),
    (-75, -67, 0x08),
    (-128, 76, 0x04),
    (-105, -97, 0x08),
    (-90, -82, 0x08),
    (-120, -111, 0x08),
    (-105, -97, 0x08),
    (-120, -111, 0x08),
    (0, -5, 0x0a),
    (0, -5, 0x0b),
    (2, -3, 0x0c),
    (1, -3, 0x0d),
    (0, -3, 0x8d),
    (1, -3, 0x8c),
    (1, -3, 0x8b),
    (1, -3, 0x8a),
    (-6, 12, 0x89),
    (-6, 12, 0x89),
    (-10, 28, 0xc9),
    (-10, 28, 0x49),
    (-8, 44, 0x89),
    (-8, 44, 0x89),
    (-10, 56, 0x02),
    (-10, 56, 0x02),
    (-23, 70, 0x48),
    (5, 70, 0x08),
    (-23, 70, 0x48),
    (5, 70, 0x08),
    (-38, 85, 0x48),
    (19, 85, 0x08),
    (-38, 85, 0x48),
    (19, 85, 0x08),
    (-52, 99, 0x48),
    (33, 101, 0x08),
    (-52, 99, 0x48),
    (33, 101, 0x08),
    (-66, 113, 0x48),
    (47, 115, 0x08),
    (-66, 113, 0x48),
    (47, 115, 0x08),
];
const QUAKE_ITEM_POS: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 17, 21, 25, 30, 36, 42, 48, 53, 57, 60, 62, 64, 65, 66,
    67, 68, 69, 70, 71, 72, 74, 77, 81, 85, 88, 91, 94, 97, 100, 103, 107, 111, 114, 116, 118, 119,
    120, 121, 122, 123, 124, 125, 126, 128, 130, 132, 134, 137, 141, 145, 149, 151,
];
const QUAKE_ITEMS2: [(i8, i8, u8); 104] = [
    (-96, 112, 0x20),
    (-96, 112, 0x21),
    (-96, 112, 0x66),
    (-96, 112, 0x22),
    (-96, 112, 0x23),
    (-96, 112, 0x63),
    (-96, 112, 0x62),
    (-96, 112, 0x26),
    (-96, 112, 0x27),
    (-86, 124, 0x28),
    (-86, 124, 0x28),
    (-72, -117, 0x28),
    (-72, -117, 0x28),
    (-59, -102, 0xa1),
    (-59, -102, 0xa1),
    (-44, -116, 0x68),
    (-44, -116, 0x68),
    (-29, 126, 0x68),
    (-29, 126, 0x68),
    (-19, 125, 0xc5),
    (-112, 96, 0x2a),
    (-112, 96, 0x2b),
    (-112, 96, 0x2c),
    (-112, 96, 0x2d),
    (-119, 82, 0x29),
    (-112, 96, 0x2a),
    (-123, 66, 0xe9),
    (-119, 82, 0x29),
    (-121, 50, 0x29),
    (-123, 66, 0xe9),
    (126, 34, 0x28),
    (-115, 34, 0x68),
    (-121, 50, 0x29),
    (-106, 18, 0xa9),
    (111, 19, 0x28),
    (126, 34, 0x28),
    (-115, 34, 0x68),
    (-100, 2, 0x68),
    (102, 4, 0xe9),
    (-106, 18, 0xa9),
    (111, 19, 0x28),
    (-91, -14, 0xa9),
    (95, -11, 0x28),
    (-100, 2, 0x68),
    (102, 4, 0xe9),
    (96, 112, 0x60),
    (96, 112, 0x61),
    (96, 112, 0x26),
    (96, 112, 0x62),
    (96, 112, 0x63),
    (96, 112, 0x23),
    (96, 112, 0x22),
    (96, 112, 0x66),
    (85, 111, 0xe8),
    (96, 112, 0x67),
    (70, 104, 0x24),
    (85, 111, 0xe8),
    (70, 104, 0x24),
    (54, 108, 0xe4),
    (40, 100, 0x28),
    (38, 107, 0x24),
    (54, 108, 0xe4),
    (25, 85, 0x28),
    (40, 100, 0x28),
    (38, 107, 0x24),
    (22, 110, 0xe4),
    (11, 70, 0x28),
    (25, 85, 0x28),
    (7, 108, 0x24),
    (22, 110, 0xe4),
    (11, 70, 0x28),
    (7, 108, 0x24),
    (112, 112, 0x2a),
    (112, 112, 0x2b),
    (112, 112, 0x2c),
    (112, 112, 0x2d),
    (112, 112, 0x2a),
    (108, 125, 0x29),
    (108, 125, 0x29),
    (114, -116, 0x28),
    (114, -116, 0x28),
    (124, -100, 0x29),
    (124, -100, 0x29),
    (123, -84, 0xe9),
    (123, -84, 0xe9),
    (117, -74, 0xe4),
    (-124, -69, 0x28),
    (117, -74, 0xe4),
    (-124, -69, 0x28),
    (103, -67, 0x68),
    (-110, -54, 0x28),
    (103, -67, 0x68),
    (-110, -54, 0x28),
    (95, -52, 0x69),
    (-102, -39, 0x29),
    (95, -52, 0x69),
    (-102, -39, 0x29),
    (96, -36, 0xe9),
    (-102, -24, 0xe9),
    (96, -36, 0xe9),
    (-102, -24, 0xe9),
    (-123, -14, 0x29),
    (-115, -14, 0x2e),
    (49, -12, 0x28),
];
const QUAKE_ITEM_POS2: [u8; 56] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 18, 19, 20, 21, 22, 23, 24, 26, 28, 30, 33, 37, 41,
    45, 46, 47, 48, 49, 50, 51, 52, 53, 55, 57, 59, 62, 66, 70, 72, 73, 74, 75, 76, 78, 80, 82, 84,
    87, 91, 95, 99, 101, 104,
];

impl<R: AncillaRuntime> Ancilla<'_, '_, R> {
    // C-spelling entry points below intentionally retain the translation-unit
    // ownership in their names.  They delegate only to concrete local ports;
    // none of them crosses `AncillaRuntime` unless the C function itself does.
    pub fn add_bird_travel_something(&mut self, kind: u8, limit: u8) {
        let _ = self.add_bird_travel(kind, limit);
    }

    pub fn ancilla07_bomb(&mut self, k: usize) {
        self.bomb(k);
    }

    pub fn ancilla0a_arrow_in_the_wall(&mut self, k: usize) {
        self.arrow_in_wall(k);
    }

    pub fn ancilla27_duck(&mut self, k: usize) {
        self.duck(k);
    }

    pub fn ancilla_add_add_ancilla_bank08(&mut self, kind: u8, limit: u8) -> Option<usize> {
        self.add_ancilla(kind, limit)
    }

    pub fn ancilla_add_dash_dust_charging(&mut self, kind: u8, limit: u8) -> Option<usize> {
        self.add_dashing_dust_ex(kind, limit, 0)
    }

    pub fn ancilla_add_gt_cutscene(&mut self) -> Option<usize> {
        self.add_ganons_tower_cutscene()
    }

    pub fn ancilla_add_ms_cutscene(&mut self, kind: u8, limit: u8) -> Option<usize> {
        self.add_master_sword_cutscene(kind, limit)
    }

    pub fn ancilla_draw_gt_cutscene_crystal(&mut self, index: usize, x: u16, y: u16) {
        self.gt_cutscene_draw_crystal(index, x, y);
    }

    pub fn ancilla_draw_weathervane_explosion_wood_debris(&mut self, k: usize) {
        self.draw_weathervane_debris(k);
    }

    pub fn ancilla_allocate_oam_from_region_a_or_d_or_f(&mut self, k: usize, size: u8) -> usize {
        self.allocate_oam_from_region_a_d_f(k, size)
    }

    pub fn ancilla_allocate_oam_from_region_b_or_e(&mut self, size: u8) {
        let region = if self.read(0xfb3) == 0 { b'B' } else { b'E' };
        self.runtime.allocate_oam_region(self.ram, region, size);
    }

    #[must_use]
    pub fn ancilla_calculate_sfx_pan(&self, k: usize) -> u8 {
        misc::calculate_sfx_pan(self.ram, self.get_x(k))
    }

    pub fn bombos_spell_control_blasting(&mut self, k: usize) {
        self.control_bombos_blasting(k);
    }

    pub fn bombos_spell_control_fire_columns(&mut self, k: usize) {
        self.control_bombos_fire_columns(k);
    }

    pub fn bombos_spell_finish_fire_columns(&mut self, k: usize) {
        self.finish_bombos_fire_columns(k);
    }

    pub fn boomerang_cheat_when_no_ones_looking(&self, k: usize, speed: &mut ProjectSpeed) {
        self.boomerang_cheat(k, speed);
    }

    pub fn byrna_windup_spark_transmute_to_normal(&mut self, k: usize) {
        self.byrna_windup_transmute(k);
    }

    pub fn ether_spell_handle_lightning_stroke(&mut self, k: usize) {
        self.ether_lightning_stroke(k);
    }

    pub fn ether_spell_handle_orb_pulse(&mut self, k: usize) {
        self.ether_orb_pulse(k);
    }

    pub fn ether_spell_handle_radial_spin(&mut self, k: usize) {
        self.ether_radial_spin(k);
    }

    pub fn hapiness_pond_rupees_execute_rupee(&mut self, k: usize, pond: usize) {
        self.happiness_pond_execute_rupee(k, pond);
    }

    pub fn hapiness_pond_rupees_get_state(&mut self, ancilla: usize, pond: usize) {
        self.happiness_pond_get_state(ancilla, pond);
    }

    pub fn hapiness_pond_rupees_save_state(&mut self, pond: usize, ancilla: usize) {
        self.happiness_pond_save_state(pond, ancilla);
    }

    #[must_use]
    pub fn hookshot_should_i_even_bother_with_tiles(&mut self, k: usize) -> bool {
        self.hookshot_should_bother_with_tiles(k)
    }

    pub fn quake_spell_control_bolts(&mut self, k: usize) {
        self.quake_control_bolts(k);
    }

    pub fn quake_spell_shake_screen(&mut self, _k: usize) {
        self.quake_shake_screen();
    }

    pub fn quake_spell_spread_bolts(&mut self, k: usize) {
        self.quake_spread_bolts(k);
    }

    pub fn somarian_blast_draw(&mut self, k: usize) {
        self.somaria_blast_draw(k);
    }

    pub fn spin_attack_sparkle_a_transmute_to_next_spark(&mut self, k: usize) {
        self.spin_attack_sparkle_a_transmute(k);
    }

    /// Exact port of `SomariaBlock_CheckForTransitTile`. The reverse scan is
    /// observable when more than one probe touches a transit line.
    pub fn somaria_block_check_for_transit_tile(&mut self, k: usize) {
        const X: [i8; 12] = [-8, 0, 8, -8, 0, 8, -16, -16, -16, 16, 16, 16];
        const Y: [i8; 12] = [-16, -16, -16, 16, 16, 16, -8, 0, 8, -8, 0, 8];
        if self.read(0x3f4) == 0 {
            return;
        }
        for j in (0..12).rev() {
            let x = self.get_x(k).wrapping_add(i16::from(X[j]) as u16);
            let y = self.get_y(k).wrapping_add(i16::from(Y[j]) as u16);
            let priority = self.object_priority(k);
            self.check_tile_collision_targeted(k, x, y);
            self.set_object_priority(k, priority);
            if matches!(self.tile_attribute(k), 0xb6 | 0xbc) {
                self.set_xy(k, x, y);
                self.add_somaria_platform_poof(k);
                return;
            }
        }
    }

    fn set_xy(&mut self, k: usize, x: u16, y: u16) {
        self.set_x(k, x);
        self.set_y(k, y);
    }

    fn prep_oam_coord(&mut self, k: usize) -> (u16, u16) {
        const LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
        self.ram
            .write_u16(0x64, u16::from(LAYER_BITS[usize::from(self.floor(k))]) << 8);
        (
            self.get_x(k).wrapping_sub(self.ram.read_u16(0xe2)),
            self.get_y(k).wrapping_sub(self.ram.read_u16(0xe8)),
        )
    }

    fn prep_adjusted_oam_coord(&mut self, k: usize) -> (u16, u16) {
        const LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
        self.ram
            .write_u16(0x64, u16::from(LAYER_BITS[usize::from(self.floor(k))]) << 8);
        (
            self.get_x(k).wrapping_sub(self.ram.read_u16(0x11e)),
            self.get_y(k).wrapping_sub(self.ram.read_u16(0x122)),
        )
    }

    /// Exact port of `SpinSpark_Draw`, including its deliberately mismatched
    /// 32-entry character table and 29-entry companion tables. The `0xff`
    /// character sentinel prevents the three out-of-bounds companion reads.
    pub fn spin_spark_draw(&mut self, k: usize, offs: i32) {
        let (x, y) = self.prep_oam_coord(k);
        let mut oam_index = self.current_oam_index();
        let base = (i32::from(self.item_to_link(k)) + offs) * 4;
        assert!((0..32).contains(&base));
        for i in 0..4 {
            let t = (base + i) as usize;
            let charnum = SPIN_CHAR[t];
            if charnum != 0xff {
                let flags = (SPIN_FLAGS[t] & !0x30) | (self.ram.read_u16(0x64) >> 8) as u8;
                self.set_oam(
                    oam_index,
                    x.wrapping_add(SPIN_X[t] as u16),
                    y.wrapping_add(i16::from(SPIN_Y[t]) as u16),
                    charnum,
                    flags,
                    0,
                );
                oam_index += 1;
            }
        }
    }

    fn alloc_init(&mut self, kind: u8, limit: u8) -> Option<usize> {
        // BYTE(R14) is the low byte at WRAM $0e.
        if self.read(0x64a) >= 1 {
            self.write(0x0e, limit.wrapping_add(1));
        }
        let count = (0..5).filter(|&i| self.kind(i) == kind).count() as u8;
        if limit.wrapping_add(1) == count {
            return None;
        }
        let first = if kind == 7 || kind == 8 {
            usize::from(limit)
        } else {
            4
        };
        if let Some(k) = (0..=first).rev().find(|&k| self.kind(k) == 0) {
            return Some(k);
        }
        let mut k = i32::from(self.read(ANCILLA_ALLOC_ROTATE));
        loop {
            k -= 1;
            if k < 0 {
                k = i32::from(limit);
            }
            if matches!(self.kind(k as usize), 0x3c | 0x13 | 0x0a) {
                self.write(ANCILLA_ALLOC_ROTATE, k as u8);
                return Some(k as usize);
            }
            if k == 0 {
                break;
            }
        }
        self.write(ANCILLA_ALLOC_ROTATE, 0);
        None
    }

    pub fn add_ancilla(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.alloc_init(kind, limit)?;
        self.set_kind(k, kind);
        self.set_floor(k, self.read(0xee));
        self.set_floor2(k, self.read(0x476));
        self.set_y_velocity(k, 0);
        self.set_x_velocity(k, 0);
        self.set_object_priority(k, 0);
        self.set_state_u(k, 0);
        self.set_sprite_count(k, PFLAGS[usize::from(kind)]);
        Some(k)
    }

    /// Exact port of `AddDashingDustEx`.
    pub fn add_dashing_dust_ex(&mut self, kind: u8, limit: u8, flag: u8) -> Option<usize> {
        const X: [i8; 4] = [4, 4, 6, 0];
        const Y: [i8; 4] = [20, 4, 16, 16];
        let k = self.add_ancilla(kind, limit)?;
        self.set_step(k, flag);
        self.set_item_to_link(k, 0);
        self.set_timer(k, 3);
        let direction = usize::from(self.read(0x2f) >> 1);
        self.set_direction(k, direction as u8);
        let link_x = self.ram.read_u16(0x22);
        let link_y = self.ram.read_u16(0x20);
        if flag == 0 {
            self.set_xy(k, link_x, link_y.wrapping_add(20));
        } else {
            self.set_xy(
                k,
                link_x.wrapping_add(i16::from(X[direction]) as u16),
                link_y.wrapping_add(i16::from(Y[direction]) as u16),
            );
        }
        Some(k)
    }

    /// Exact port of `Bomb_ProjectSpeedTowardsPlayer`. Sprite slot zero is
    /// restored after the genuine cross-unit projection call.
    pub fn bomb_project_speed_towards_player(
        &mut self,
        _k: usize,
        x: u16,
        y: u16,
        velocity: u8,
    ) -> ProjectSpeed {
        const Y_LO: usize = 0xd00;
        const X_LO: usize = 0xd10;
        const Y_HI: usize = 0xd20;
        const X_HI: usize = 0xd30;
        const Z: usize = 0xf70;
        let old_x = u16::from_le_bytes([self.read(X_LO), self.read(X_HI)]);
        let old_y = u16::from_le_bytes([self.read(Y_LO), self.read(Y_HI)]);
        let old_z = self.read(Z);
        self.write(X_LO, x as u8);
        self.write(X_HI, (x >> 8) as u8);
        self.write(Y_LO, y as u8);
        self.write(Y_HI, (y >> 8) as u8);
        self.write(Z, 0);
        let result = self
            .runtime
            .sprite_project_speed_towards_link(self.ram, 0, velocity);
        self.write(Z, old_z);
        self.write(X_LO, old_x as u8);
        self.write(X_HI, (old_x >> 8) as u8);
        self.write(Y_LO, old_y as u8);
        self.write(Y_HI, (old_y >> 8) as u8);
        result
    }

    pub fn add_bird_common(&mut self, k: usize) {
        add_bird_common(self, k);
    }

    pub fn boomerang_cheat(&self, k: usize, speed: &mut ProjectSpeed) {
        boomerang_cheat(self, k, speed);
    }

    #[must_use]
    pub fn somarian_block_check_empty(&self, first: usize) -> bool {
        somarian_block_check_empty(self, first)
    }

    pub fn check_damage_to_sprite(&mut self, sprite: usize, kind: u8) {
        if self.read(0xef0 + sprite) as i8 >= 0 {
            self.check_damage_to_sprite_aggressive(sprite, kind);
        }
    }

    pub fn check_damage_to_sprite_aggressive(&mut self, sprite: usize, kind: u8) {
        const DAMAGE: [u8; 57] = [
            6, 1, 11, 0, 0, 0, 0, 8, 0, 6, 0, 12, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 14, 13, 0, 0,
            15, 0, 0, 7, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 11, 0, 1, 1, 1, 1, 1, 1, 1,
            1,
        ];
        let mut damage = DAMAGE[usize::from(kind)];
        if damage == 6 && self.read(0xf340) >= 3 {
            if self.read(0xe20 + sprite) == 0xd7 {
                self.write(0xf10 + sprite, 32);
            }
            damage = 9;
        }
        self.runtime
            .check_damage_to_sprite_preset(self.ram, sprite, damage);
    }

    pub fn medallion_check_sprite_damage(&mut self, k: usize) {
        let kind = self.kind(k);
        self.write(0xfb5, kind);
        for sprite in (0..16).rev() {
            if self.read(0xdd0 + sprite) >= 9
                && (self.read(0xba0 + sprite) | self.read(0xf00 + sprite)) == 0
            {
                self.check_damage_to_sprite_aggressive(sprite, kind);
            }
        }
    }

    pub fn call_for_duck_indoors(&mut self) {
        misc::ancilla_sfx2_near(self.ram, 0x13);
        if (0..=5).any(|k| self.kind(k) == 0x27) {
            return;
        }
        if let Some(k) = self.add_ancilla(0x27, 4) {
            self.set_timer(k, 0x78);
            self.set_state_l(k, 0);
            self.set_z_velocity(k, 0);
            self.set_height(k, 0);
            self.set_step(k, 0);
            self.add_bird_common(k);
        }
    }

    pub fn sfx1_pan(&mut self, k: usize, value: u8) {
        self.write(0xcf8, value);
        self.write(
            0x12d,
            value | misc::calculate_sfx_pan(self.ram, self.get_x(k)),
        );
    }

    pub fn sfx2_pan(&mut self, k: usize, value: u8) {
        self.write(0xcf8, value);
        self.write(
            0x12e,
            value | misc::calculate_sfx_pan(self.ram, self.get_x(k)),
        );
    }

    pub fn sfx3_pan(&mut self, k: usize, value: u8) {
        self.write(0xcf8, value);
        self.write(
            0x12f,
            value | misc::calculate_sfx_pan(self.ram, self.get_x(k)),
        );
    }

    pub fn project_reflexive_speed_onto_sprite(
        &mut self,
        sprite: usize,
        x: u16,
        y: u16,
        velocity: u8,
    ) -> ProjectSpeed {
        let old_x = self.ram.read_u16(0x22);
        let old_y = self.ram.read_u16(0x20);
        self.ram.write_u16(0x22, x);
        self.ram.write_u16(0x20, y);
        let result = self
            .runtime
            .sprite_project_speed_towards_link(self.ram, sprite, velocity);
        self.ram.write_u16(0x22, old_x);
        self.ram.write_u16(0x20, old_y);
        result
    }

    #[must_use]
    pub fn is_right_of_link(&self, k: usize) -> PairU8 {
        let difference = self.ram.read_u16(0x22).wrapping_sub(self.get_x(k));
        PairU8 {
            a: u8::from((difference as i16) < 0),
            b: difference as u8,
        }
    }

    #[must_use]
    pub fn is_below_link(&self, k: usize) -> PairU8 {
        let difference = self.ram.read_u16(0x20).wrapping_sub(self.get_y(k));
        PairU8 {
            a: u8::from((difference as i16) < 0),
            b: difference as u8,
        }
    }

    #[must_use]
    pub fn project_speed_towards_player(&self, k: usize, mut velocity: u8) -> ProjectSpeed {
        if velocity == 0 {
            return ProjectSpeed::default();
        }
        let below = self.is_below_link(k);
        let mut short = if (below.b as i8) < 0 {
            below.b.wrapping_neg()
        } else {
            below.b
        };
        let right = self.is_right_of_link(k);
        let mut long = if (right.b as i8) < 0 {
            right.b.wrapping_neg()
        } else {
            right.b
        };
        let mut swapped = false;
        if long < short {
            swapped = true;
            core::mem::swap(&mut short, &mut long);
        }
        let mut x_velocity = velocity;
        let mut y_velocity = 0_u8;
        let mut accumulator = 0_u8;
        loop {
            accumulator = accumulator.wrapping_add(short);
            if accumulator >= long {
                accumulator = accumulator.wrapping_sub(long);
                y_velocity = y_velocity.wrapping_add(1);
            }
            velocity = velocity.wrapping_sub(1);
            if velocity == 0 {
                break;
            }
        }
        if swapped {
            core::mem::swap(&mut x_velocity, &mut y_velocity);
        }
        ProjectSpeed {
            x: if right.a != 0 {
                x_velocity.wrapping_neg()
            } else {
                x_velocity
            },
            y: if below.a != 0 {
                y_velocity.wrapping_neg()
            } else {
                y_velocity
            },
            xdiff: right.b,
            ydiff: below.b,
        }
    }

    pub fn bomb_check_sprite_damage(&mut self, k: usize) {
        for sprite in (0..16).rev() {
            if ((((sprite as u8) ^ self.read(0x1a)) & 3)
                | self.read(0xef0 + sprite)
                | self.read(0xba0 + sprite))
                != 0
            {
                continue;
            }
            if self.read(0xf20 + sprite) != self.floor(k) || self.read(0xdd0 + sprite) < 9 {
                continue;
            }
            let hitbox = AncillaHitBox {
                x: self.get_x(k).wrapping_sub(24),
                y: self
                    .get_y(k)
                    .wrapping_sub(24)
                    .wrapping_sub(u16::from(self.height(k))),
                width: 48,
                height: 48,
            };
            if !self
                .runtime
                .sprite_hitbox_overlaps(self.ram, sprite, hitbox)
            {
                continue;
            }
            if self.read(0xe20 + sprite) == 0x92 && self.read(0xdb0 + sprite) >= 3 {
                continue;
            }
            self.check_damage_to_sprite(sprite, self.kind(k));
            let speed =
                self.project_reflexive_speed_onto_sprite(sprite, self.get_x(k), self.get_y(k), 64);
            self.write(0xf40 + sprite, speed.x.wrapping_neg());
            self.write(0xf30 + sprite, speed.y.wrapping_neg());
        }
    }

    pub fn check_tile_collision_targeted(&mut self, k: usize, mut x: u16, y: u16) -> bool {
        if y.wrapping_sub(self.ram.read_u16(0xe8)) >= 224
            || x.wrapping_sub(self.ram.read_u16(0xe2)) >= 256
        {
            return false;
        }
        let tile_attr = if self.read(0x1b) == 0 {
            x >>= 3;
            self.runtime.overworld_tile_attribute(self.ram, x, y)
        } else {
            let (attribute, adjusted_x) =
                self.runtime
                    .dungeon_tile_attribute(self.ram, self.floor(k), x, y);
            x = adjusted_x;
            attribute
        };
        self.set_tile_attribute(k, tile_attr);
        if tile_attr == 3 && self.floor2(k) != 0 {
            return false;
        }
        let mut collision = TILE_COLL0_ATTRS[usize::from(tile_attr)];
        if self.kind(k) == 2 && tile_attr & 0xf0 == 0xc0 {
            collision = 0;
        }
        if self.object_priority(k) == 0 {
            match collision {
                0 => return false,
                1 => {
                    self.write(0xfdc, 3);
                    return true;
                }
                2 => return self.runtime.sloped_tile_collision(self.ram, x, y),
                3 => {
                    if self.floor2(k) != 0 {
                        self.write(0xfdc, 3);
                        return true;
                    }
                    return false;
                }
                _ => {}
            }
        }
        let cooldown = self.state_u(k).wrapping_sub(1);
        self.set_state_u(k, cooldown);
        if (cooldown as i8) < 0 {
            self.set_state_u(k, 0);
            if collision == 4 {
                self.set_state_u(k, 6);
                self.set_object_priority(k, self.object_priority(k) ^ 1);
            }
        }
        false
    }

    pub fn check_tile_collision_one_floor(&mut self, k: usize) -> bool {
        const X: [i8; 20] = [
            8, 8, 0, 16, 4, 4, 0, 16, 4, 4, 4, 12, 12, 12, 4, 12, 0, 0, 0, 0,
        ];
        const Y: [i8; 20] = [
            0, 16, 5, 5, 0, 16, 4, 4, 4, 12, 5, 5, 4, 12, 12, 12, 0, 0, 0, 0,
        ];
        let direction = usize::from(self.direction(k));
        self.check_tile_collision_targeted(
            k,
            self.get_x(k).wrapping_add(i16::from(X[direction]) as u16),
            self.get_y(k).wrapping_add(i16::from(Y[direction]) as u16),
        )
    }

    pub fn check_tile_collision(&mut self, k: usize) -> u8 {
        if self.read(0x1b) == 0 && self.object_priority(k) != 0 {
            self.set_tile_attribute(k, 0);
            return 0;
        }
        let dungeon_collision = self.read(0x46c);
        if dungeon_collision == 0 {
            return self.check_tile_collision_one_floor(k) as u8;
        }
        let (x_offset, y_offset) = if dungeon_collision < 3 {
            (
                self.ram
                    .read_u16(0xe0)
                    .wrapping_sub(self.ram.read_u16(0xe2)),
                self.ram
                    .read_u16(0xe6)
                    .wrapping_sub(self.ram.read_u16(0xe8)),
            )
        } else {
            (0, 0)
        };
        let old_x = self.get_x(k);
        let old_y = self.get_y(k);
        self.set_xy(
            k,
            old_x.wrapping_add(x_offset),
            old_y.wrapping_add(y_offset),
        );
        self.set_floor(k, 1);
        let upper = self.check_tile_collision_one_floor(k) as u8;
        self.set_floor(k, 0);
        self.set_xy(k, old_x, old_y);
        (upper << 1) | self.check_tile_collision_one_floor(k) as u8
    }

    pub fn check_tile_collision_staggered(&mut self, k: usize) -> u8 {
        if (self.read(0x1a) ^ k as u8) & 1 != 0 {
            self.check_tile_collision(k)
        } else {
            0
        }
    }

    pub fn check_tile_collision_class2(&mut self, k: usize) -> bool {
        let dungeon_collision = self.read(0x46c);
        if dungeon_collision == 0 {
            return self.check_tile_collision_class2_inner(k);
        }
        let (x_offset, y_offset) = if dungeon_collision < 3 {
            (
                self.ram
                    .read_u16(0xe0)
                    .wrapping_sub(self.ram.read_u16(0xe2)),
                self.ram
                    .read_u16(0xe6)
                    .wrapping_sub(self.ram.read_u16(0xe8)),
            )
        } else {
            (0, 0)
        };
        let old_x = self.get_x(k);
        let old_y = self.get_y(k);
        self.set_xy(
            k,
            old_x.wrapping_add(x_offset),
            old_y.wrapping_add(y_offset),
        );
        self.set_floor(k, 1);
        let upper = self.check_tile_collision_class2_inner(k);
        self.set_floor(k, 0);
        self.set_xy(k, old_x, old_y);
        upper | self.check_tile_collision_class2_inner(k)
    }

    pub fn check_tile_collision_class2_inner(&mut self, k: usize) -> bool {
        const Y: [i8; 4] = [-8, 8, 0, 0];
        const X: [i8; 4] = [0, 0, -8, 8];
        let direction = usize::from(self.direction(k));
        let mut x = self.get_x(k).wrapping_add(i16::from(X[direction]) as u16);
        let y = self.get_y(k).wrapping_add(i16::from(Y[direction]) as u16);
        if y.wrapping_sub(self.ram.read_u16(0xe8)) >= 224
            || x.wrapping_sub(self.ram.read_u16(0xe2)) >= 256
        {
            return false;
        }
        let tile_attr = if self.read(0x1b) == 0 {
            x >>= 3;
            self.runtime.overworld_tile_attribute(self.ram, x, y)
        } else {
            let (attribute, adjusted_x) =
                self.runtime
                    .dungeon_tile_attribute(self.ram, self.floor(k), x, y);
            x = adjusted_x;
            attribute
        };
        self.set_tile_attribute(k, tile_attr);
        if tile_attr == 3 && self.floor2(k) != 0 {
            return false;
        }
        match TILE_COLL_ATTRS[usize::from(tile_attr)] {
            0 => false,
            2 => self.runtime.sloped_tile_collision(self.ram, x, y),
            4 => {
                if self.floor2(k) != 0 {
                    true
                } else {
                    self.set_object_priority(k, 1);
                    false
                }
            }
            3 => self.floor2(k) != 0,
            _ => true,
        }
    }

    fn check_initial_tile_a(&mut self, k: usize) -> i8 {
        const Y: [i8; 12] = [8, 0, -8, 8, 16, 24, 8, 8, 8, 8, 8, 8];
        const X: [i8; 12] = [0, 0, 0, 0, 0, 0, 0, -8, -16, 0, 8, 16];
        let mut j = usize::from(self.direction(k)) * 3;
        for i in (0_i8..=2).rev() {
            self.set_xy(
                k,
                self.ram.read_u16(0x22).wrapping_add(i16::from(X[j]) as u16),
                self.ram.read_u16(0x20).wrapping_add(i16::from(Y[j]) as u16),
            );
            if self.check_tile_collision(k) != 0 {
                return i;
            }
            j += 1;
        }
        -1
    }

    fn check_initial_tile_collision_class2(&mut self, k: usize) -> bool {
        const Y: [i16; 9] = [15, 16, 28, 24, 12, 12, 12, 12, 8];
        const X: [i16; 9] = [8, 8, 8, 8, -1, 0, 17, 16, 0x4b8b];
        let mut j = usize::from(self.direction(k)) * 2;
        for _ in 0..3 {
            self.set_xy(
                k,
                self.ram.read_u16(0x22).wrapping_add(X[j] as u16),
                self.ram.read_u16(0x20).wrapping_add(Y[j] as u16),
            );
            if self.check_tile_collision_class2(k) {
                return true;
            }
            j += 1;
        }
        false
    }

    pub fn add_hit_stars(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const XY: [i8; 12] = [21, -11, 21, 11, 3, -6, 21, 5, 16, -14, 16, 14];
        const X2: [i8; 6] = [-3, 19, 2, 13, -6, 22];
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 0);
        self.set_aux_timer(k, 2);
        self.set_arr3(k, 1);
        self.set_y_velocity(k, 0);
        self.set_x_velocity(k, 0);
        let step = if self.read(0x301) != 0 {
            usize::from(self.read(0x2f) >> 1) + 2
        } else if self.read(0x37a) != 0 {
            usize::from(u8::from(self.read(0x2f) != 4))
        } else {
            usize::from(kind)
        };
        self.set_step(k, step as u8);
        let x2 = self
            .ram
            .read_u16(0x22)
            .wrapping_add(i16::from(X2[step]) as u16);
        self.set_state_a(k, x2 as u8);
        self.set_state_b(k, (x2 >> 8) as u8);
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(i16::from(XY[step * 2 + 1]) as u16),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(XY[step * 2]) as u16),
        );
        Some(k)
    }

    #[must_use]
    pub fn check_for_presence(&self, kind: u8) -> bool {
        (0..=5).rev().any(|k| self.kind(k) == kind)
    }

    /// Exact port of `AncillaAdd_ArrowFindSlot`.
    pub fn arrow_find_slot(&mut self, kind: u8, arrow_limit: u8) -> Option<usize> {
        let arrow_count = (0..=4).rev().filter(|&k| self.kind(k) == 10).count() as u8;
        let k = if arrow_count != arrow_limit.wrapping_add(1) {
            (0..=4).rev().find(|&k| self.kind(k) == 0)?
        } else {
            loop {
                let mut rotate = self.read(ANCILLA_ALLOC_ROTATE).wrapping_sub(1);
                if (rotate as i8) < 0 {
                    rotate = 4;
                }
                self.write(ANCILLA_ALLOC_ROTATE, rotate);
                let slot = usize::from(rotate);
                if self.kind(slot) == 10 {
                    break slot;
                }
            }
        };
        self.set_kind(k, kind);
        self.set_floor(k, self.read(0xee));
        self.set_floor2(k, self.read(0x476));
        self.set_y_velocity(k, 0);
        self.set_x_velocity(k, 0);
        self.set_object_priority(k, 0);
        self.set_state_u(k, 0);
        self.set_sprite_count(k, PFLAGS[usize::from(kind)]);
        Some(k)
    }

    /// Exact port of `AncillaAdd_Arrow`.
    pub fn add_arrow(
        &mut self,
        kind: u8,
        direction: u8,
        arrow_limit: u8,
        x: u16,
        y: u16,
    ) -> Option<usize> {
        const X: [i8; 4] = [4, 4, 0, 4];
        const Y: [i8; 4] = [-4, 3, 4, 4];
        const X_VEL: [i8; 4] = [0, 0, -48, 48];
        const Y_VEL: [i8; 4] = [-48, 48, 0, 0];

        self.ram.write_u16(0x72, y);
        self.ram.write_u16(0x74, x);
        self.write(0x76, direction);
        if self.check_for_presence(kind) {
            return None;
        }
        let k = self.arrow_find_slot(kind, arrow_limit)?;
        self.write(0x12e, misc::link_calculate_sfx_pan(self.ram) | 7);
        self.set_state_h(k, 0);
        self.set_item_to_link(k, 8);
        let j = usize::from(direction >> 1);
        self.set_direction(k, j as u8 | 4);
        self.set_y_velocity(k, Y_VEL[j] as u8);
        self.set_x_velocity(k, X_VEL[j] as u8);
        self.set_xy(
            k,
            x.wrapping_add(i16::from(X[j]) as u16),
            y.wrapping_add(8).wrapping_add(i16::from(Y[j]) as u16),
        );
        Some(k)
    }

    /// Exact port of `AncillaAdd_ChargedSpinAttackSparkle`.
    pub fn add_charged_spin_attack_sparkle(&mut self) -> Option<usize> {
        let k = (0..10)
            .rev()
            .find(|&slot| matches!(self.kind(slot), 0 | 0x3c))?;
        self.set_kind(k, 13);
        self.set_floor(k, self.read(0xee));
        self.set_timer(k, 6);
        Some(k)
    }

    /// Exact port of `AncillaAdd_ExplodingWeatherVane`.
    pub fn add_exploding_weathervane(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const ARR4: [i8; 12] = [8, 10, 9, 4, 11, 12, -10, -8, 4, -6, -10, -4];
        const ARR5: [u8; 12] = [20, 22, 20, 20, 22, 20, 20, 22, 20, 22, 20, 20];
        const ARR6: [u8; 12] = [
            0xb0, 0xa3, 0xa0, 0xa2, 0xa0, 0xa8, 0xa0, 0xa0, 0xa8, 0xa1, 0xb0, 0xa0,
        ];
        const ARR8: [u8; 12] = [0, 2, 4, 6, 3, 8, 14, 8, 12, 7, 10, 8];
        const ARR10: [u8; 12] = [48, 18, 32, 20, 22, 24, 32, 20, 24, 22, 20, 32];

        let k = self.add_ancilla(kind, limit)?;
        self.set_aux_timer(k, 10);
        self.set_state_g(k, 128);
        self.set_step(k, 0);
        self.set_arr3(k, 0);
        self.write(0x12e, 0);
        self.write(0x12c, 0xf2);
        self.write(0x12d, 0x17);
        self.write(0x158b8, 0);
        self.ram.write_u16(0x158b6, 0x280);
        for i in (0..12).rev() {
            self.write(0x15800 + i, 0);
            self.write(0x1580c + i, ARR4[i] as u8);
            self.write(0x15818 + i, ARR5[i]);
            self.write(0x15824 + i, ARR6[i]);
            self.write(0x15830 + i, 7);
            self.write(0x1583c + i, ARR8[i]);
            self.write(0x15848 + i, 2);
            self.write(0x15854 + i, ARR10[i]);
            self.write(0x15860 + i, 1);
            self.write(0x1586c + i, (i & 1) as u8);
        }
        Some(k)
    }

    /// Exact port of `AncillaAdd_TossedPondItem`.
    pub fn add_tossed_pond_item(&mut self, kind: u8, item: u8, limit: u8) -> Option<usize> {
        const X: [u8; 76] = [
            4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 2, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
        ];
        const Y: [i8; 76] = [
            -13, -13, -13, -13, -13, -12, -12, -13, -13, -12, -12, -12, -10, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -13, -12, -12, -12, -12, -12, -12, -10, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -13, -12, -12,
        ];
        self.write(0x2d8, item);
        let k = self.add_ancilla(kind, limit)?;
        self.write(0x12f, misc::link_calculate_sfx_pan(self.ram) | 0x13);
        let graphics = RECEIVE_ITEM_GFX[usize::from(item)];
        if graphics != 0xff {
            if graphics == 0x20 {
                self.runtime.decompress_shield_graphics(self.ram);
            }
            self.runtime.decode_animated_sprite_tile(self.ram, graphics);
        } else {
            self.runtime.decode_animated_sprite_tile(self.ram, 0);
        }
        if graphics == 6 {
            self.runtime.decompress_sword_graphics(self.ram);
        }
        self.write(0x308, 0x80);
        self.write(0x309, 0);
        self.write(0x2f, 0);
        self.write(0x2e, 0);
        self.set_z_velocity(k, 20);
        self.set_y_velocity(k, (-40_i8) as u8);
        self.set_x_velocity(k, 0);
        self.set_height(k, 0);
        self.set_timer(k, 16);
        self.set_item_to_link(k, item);
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(u16::from(X[usize::from(item)])),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(Y[usize::from(item)]) as u16),
        );
        Some(k)
    }

    /// Exact port of `AddHappinessPondRupees`.
    pub fn add_happiness_pond_rupees(&mut self, arg: u8) -> Option<usize> {
        const START: [i8; 4] = [0, 4, 4, 9];
        const END: [i8; 4] = [-1, 0, -1, -1];
        const X_VEL: [i8; 10] = [0, -12, -6, 6, 12, -9, -5, 0, 5, 9];
        const Y_VEL: [i8; 10] = [-40, -40, -40, -40, -40, -32, -32, -32, -32, -32];
        const Z_VEL: [u8; 10] = [20, 20, 20, 20, 20, 16, 16, 16, 16, 16];

        let ancilla = self.add_ancilla(0x42, 9)?;
        self.write(0x12f, misc::link_calculate_sfx_pan(self.ram) | 0x13);
        self.runtime
            .decode_animated_sprite_tile(self.ram, RECEIVE_ITEM_GFX[0x35]);
        self.write(0x308, 0x80);
        self.write(0x309, 0);
        self.write(0x2f, 0);
        self.write(0x2e, 0);
        self.ram.slice_mut(0x1586c, 10).fill(0);
        let x = self.ram.read_u16(0x22).wrapping_add(4);
        let y = self.ram.read_u16(0x20).wrapping_sub(12);
        let mut item = START[usize::from(arg)];
        let end = END[usize::from(arg)];
        let mut slot = 9_usize;
        loop {
            let j = item as usize;
            self.write(0x1586c + slot, 1);
            self.write(0x15818 + slot, Z_VEL[j]);
            self.write(0x15800 + slot, Y_VEL[j] as u8);
            self.write(0x1580c + slot, X_VEL[j] as u8);
            self.write(0x15854 + slot, 0);
            self.write(0x158aa + slot, 0);
            self.write(0x15860 + slot, 16);
            self.write(0x1587a + slot, 53);
            self.write(0x1583c + slot, x as u8);
            self.write(0x15848 + slot, (x >> 8) as u8);
            self.write(0x15824 + slot, y as u8);
            self.write(0x15830 + slot, (y >> 8) as u8);
            slot = slot.wrapping_sub(1);
            item -= 1;
            if item == end {
                break;
            }
        }
        Some(ancilla)
    }

    /// Exact port of `AncillaAdd_FallingPrize`.
    pub fn add_falling_prize(&mut self, kind: u8, prize_index: u8, limit: u8) -> Option<usize> {
        const ITEM: [u8; 7] = [0x10, 0x37, 0x39, 0x38, 0x26, 0x0f, 0x20];
        const G: [i8; 7] = [0x40, 0, 0, 0, 0, -1, 0];
        const X: [u16; 7] = [0x78, 0x78, 0x78, 0x78, 0x78, 0x80, 0x78];
        const Y: [u16; 7] = [0x48, 0x78, 0x78, 0x78, 0x78, 0x68, 0x78];
        const Z: [u8; 7] = [0x60, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        let prize = usize::from(prize_index);
        self.write(0x2d8, prize_index);
        let k = self.add_ancilla(kind, limit)?;
        let item = ITEM[prize];
        self.set_item_to_link(k, item);
        if matches!(item, 0x10 | 0x0f) {
            self.runtime
                .decode_animated_sprite_tile(self.ram, RECEIVE_ITEM_GFX[usize::from(item)]);
        }
        self.set_z_velocity(k, (-48_i8) as u8);
        self.set_y_velocity(k, 0);
        self.set_x_velocity(k, 0);
        self.set_step(k, 0);
        self.set_height(k, Z[prize]);
        self.set_aux_timer(k, 9);
        self.set_arr3(k, 0);
        self.set_state_l(k, 0);
        self.set_state_g(k, G[prize] as u8);
        self.write(0x2d8, item);
        let (x, y) = if !matches!(prize_index, 0 | 5) {
            if self.read(0x40c) == 20 {
                (
                    self.ram.read_u16(0x22) & 0xff00 | 0x100,
                    self.ram.read_u16(0x20) & 0xff00 | 0x100,
                )
            } else {
                (
                    X[prize].wrapping_add(self.ram.read_u16(0xe2)),
                    Y[prize].wrapping_add(self.ram.read_u16(0xe8)),
                )
            }
        } else {
            (
                self.ram.read_u16(0x22),
                Y[prize].wrapping_add(self.ram.read_u16(0xe8)),
            )
        };
        self.set_xy(k, x, y);
        Some(k)
    }

    pub fn add_sword_charge_sparkle_from(&mut self, source: usize) {
        self.add_sword_charge_sparkle(source);
    }

    pub fn add_silver_arrow_sparkle_from(&mut self, source: usize) {
        self.add_silver_arrow_sparkle(source);
    }

    /// Exact port of `AncillaSpawn_SwordChargeSparkle`.
    pub fn spawn_sword_charge_sparkle(&mut self) -> Option<usize> {
        const A: [u8; 4] = [0, 0, 7, 7];
        const B: [u8; 4] = [0x70, 0x70, 0, 0];
        const X: [u8; 4] = [0, 3, 4, 5];
        const Y: [u8; 4] = [5, 12, 8, 8];
        let k = self.alloc_high()?;
        self.set_kind(k, 0x3c);
        self.set_item_to_link(k, 0);
        self.set_timer(k, 4);
        self.set_floor(k, self.read(0xee));
        let direction = usize::from(self.read(0x2f) >> 1);
        let mut x = 0_i8;
        let mut y = 0_i8;
        let y_mask = A[direction];
        if y_mask == 0 {
            y = (self.read(0x79) >> 2) as i8;
            if direction == 0 {
                y = y.wrapping_neg();
            }
        }
        let x_mask = B[direction];
        if x_mask == 0 {
            x = (self.read(0x79) >> 2) as i8;
            if direction == 2 {
                x = x.wrapping_neg();
            }
        }
        let random = misc::get_random_number(self.ram);
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(i16::from(x) as u16)
                .wrapping_add(u16::from(X[direction]))
                .wrapping_add(u16::from((random & x_mask) >> 4)),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(y) as u16)
                .wrapping_add(u16::from(Y[direction]))
                .wrapping_add(u16::from(random & y_mask)),
        );
        Some(k)
    }

    pub fn add_blanket(&mut self, kind: u8) {
        self.set_kind(0, kind);
        self.set_sprite_count(0, super::PFLAGS[usize::from(kind)]);
        self.set_floor(0, self.read(0xee));
        self.set_floor2(0, self.read(0x476));
        self.set_object_priority(0, 0);
        self.set_xy(0, 0x938, 0x2162);
    }

    pub fn add_snoring(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 0);
        self.set_y_velocity(k, (-8_i8) as u8);
        self.set_aux_timer(k, 7);
        self.set_x_velocity(k, 8);
        self.set_step(k, 0xff);
        self.set_xy(
            k,
            self.ram.read_u16(0x22).wrapping_add(16),
            self.ram.read_u16(0x20).wrapping_add(4),
        );
        Some(k)
    }

    pub fn add_bomb(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const X0: [i8; 4] = [8, 8, 0, 16];
        const Y0: [i8; 4] = [0, 24, 12, 12];
        const X1: [i8; 4] = [8, 8, -6, 22];
        const Y1: [i8; 4] = [4, 28, 12, 12];
        let k = self.add_ancilla(kind, limit)?;
        let bombs = self.read(0xf343);
        if bombs == 0 {
            self.clear(k);
            return Some(k);
        }
        self.write(0xf343, bombs - 1);
        if bombs == 1 {
            self.runtime.hud_refresh_icon(self.ram);
        }
        self.set_state_r(k, 0);
        self.set_step(k, 0);
        self.set_item_to_link(k, 0);
        self.set_state_l(k, 0);
        self.set_arr3(k, super::BOMB_FRAME_DELAYS[0]);
        self.set_arr25(k, 0);
        self.set_arr26(k, 7);
        self.set_height(k, 0);
        self.set_timer(k, 8);
        let direction = usize::from(self.read(0x2f) >> 1);
        self.set_direction(k, direction as u8);
        self.set_state_t(k, 0);
        self.set_arr23(k, 0);
        self.set_arr22(k, 0);
        let collision = self.check_initial_tile_collision_class2(k);
        let (x, y) = if collision {
            (X0[direction], Y0[direction])
        } else {
            (X1[direction], Y1[direction])
        };
        self.set_xy(
            k,
            self.ram.read_u16(0x22).wrapping_add(i16::from(x) as u16),
            self.ram.read_u16(0x20).wrapping_add(i16::from(y) as u16),
        );
        self.write(
            0x12e,
            misc::calculate_sfx_pan(self.ram, self.ram.read_u16(0x22)) | 0x0b,
        );
        Some(k)
    }

    /// Exact port of `AncillaAdd_Boomerang`. The returned byte intentionally
    /// preserves the C function's `-1` to `uint8` conversion on a clear path.
    pub fn add_boomerang(&mut self, kind: u8, limit: u8) -> u8 {
        const SPEED: [u8; 4] = [0x20, 0x18, 0x30, 0x28];
        const STEP: [u8; 2] = [0x20, 0x60];
        const ARR3: [u8; 2] = [3, 2];
        const DIRECTION_MASK: [u8; 4] = [8, 4, 2, 1];
        const INPUT: [u8; 8] = [8, 4, 2, 1, 9, 5, 10, 6];
        const ARR1: [u8; 8] = [2, 3, 3, 2, 2, 3, 3, 3];
        const Y_NEAR: [i8; 8] = [-10, -8, -9, -9, -10, -8, -9, -9];
        const X_NEAR: [i8; 8] = [-10, 11, 8, -8, -10, 11, 8, -8];
        const Y_FAR: [i8; 8] = [-16, 6, 0, 0, -8, 8, -8, 8];
        const X_FAR: [i8; 8] = [0, 0, -8, 8, 8, 8, -8, -8];

        let Some(k) = self.add_ancilla(kind, limit) else {
            return 0;
        };
        self.set_aux_timer(k, 0);
        self.set_item_to_link(k, 0);
        self.set_state_k(k, 0);
        self.set_height(k, 0);
        self.set_state_l(k, self.sprite_count(k));
        self.write(0x35f, 1);
        let boomerang = self.read(0xf341).wrapping_sub(1);
        self.set_state_g(k, boomerang);
        self.set_step(k, STEP[usize::from(boomerang)]);
        self.set_arr3(k, ARR3[usize::from(boomerang)]);

        let joy = self.read(0xf0);
        let diagonal = (joy & 0x0c != 0 && joy & 3 != 0) as usize;
        let speed_index = usize::from(boomerang) * 2 + diagonal;
        let speed = SPEED[speed_index];
        self.set_state_h(k, speed);
        let input = if joy & 0x0f != 0 {
            joy & 0x0f
        } else {
            DIRECTION_MASK[usize::from(self.read(0x2f) >> 1)]
        };
        self.write(0x39d, 0);
        if input & 0x0c != 0 {
            let velocity = if input & 8 != 0 {
                speed.wrapping_neg()
            } else {
                speed
            };
            self.set_y_velocity(k, velocity);
            let direction = usize::from((velocity as i8) >= 0);
            self.set_direction(k, direction as u8);
            self.write(0x39d, DIRECTION_MASK[direction]);
        }
        self.set_state_s(k, 0);
        if input & 3 != 0 {
            if input & 2 == 0 {
                self.set_state_s(k, 1);
            }
            let velocity = if input & 2 != 0 {
                speed.wrapping_neg()
            } else {
                speed
            };
            self.set_x_velocity(k, velocity);
            let direction = if (velocity as i8) < 0 { 2 } else { 3 };
            self.set_direction(k, direction as u8);
            self.write(0x39d, self.read(0x39d) | DIRECTION_MASK[direction]);
        }

        let mut offset = INPUT.iter().position(|&value| value == input).unwrap_or(0);
        self.set_arr1(k, ARR1[offset]);
        self.set_arr23(k, (offset * 2) as u8);
        if self.read(0x3c) >= 9 {
            self.set_aux_timer(k, self.aux_timer(k).wrapping_add(1));
        } else if speed_index != 0 || joy & 0x0f == 0 {
            offset = usize::from(self.read(0x2f) >> 1);
        }

        let collision = self.check_initial_tile_a(k);
        if collision < 0 {
            let (x_offset, y_offset) = if self.aux_timer(k) != 0 {
                (X_FAR[offset], Y_FAR[offset])
            } else {
                (X_NEAR[offset], Y_NEAR[offset])
            };
            self.set_xy(
                k,
                self.ram
                    .read_u16(0x22)
                    .wrapping_add(i16::from(x_offset) as u16),
                self.ram
                    .read_u16(0x20)
                    .wrapping_add(8)
                    .wrapping_add(i16::from(y_offset) as u16),
            );
        } else {
            self.clear(k);
            self.write(0x35f, 0);
            let sfx = if self.tile_attribute(k) != 0xf0 { 5 } else { 6 };
            self.write(
                0x12e,
                misc::calculate_sfx_pan(self.ram, self.get_x(k)) | sfx,
            );
            self.add_boomerang_wall_clink(k);
        }
        collision as u8
    }

    fn revival_fairy_dust(&mut self) {
        const POWDER_FRAME: [u8; 40] = [
            13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 10, 11, 12, 0, 1, 2, 3, 4, 5, 6, 16, 17, 18, 0, 1, 2,
            3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6,
        ];
        let k = 2;
        if self.step(0) == 0 || self.step(k) == 2 {
            return;
        }
        let timer = self.arr3(k).wrapping_sub(1);
        self.set_arr3(k, timer);
        if (timer as i8) >= 0 {
            return;
        }
        self.set_arr3(k, 0);
        self.runtime.allocate_oam_region(
            self.ram,
            if self.read(0xfb3) == 0 { b'A' } else { b'D' },
            16,
        );
        let auxiliary = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, auxiliary);
        if (auxiliary as i8) < 0 {
            self.set_aux_timer(k, 3);
            if self.item_to_link(k) == 9 {
                self.set_arr3(k, 32);
                self.set_step(k, self.step(k).wrapping_add(1));
                self.set_item_to_link(k, 2);
                return;
            }
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            self.set_arr25(k, POWDER_FRAME[30 + usize::from(frame)]);
        }
        self.magic_powder_draw(k);
    }

    fn revival_fairy_monitor_hp(&mut self) {
        if matches!(self.read(0xf36d), value if value == self.read(0xf36c) || value == 0x38)
            && self.read(0x20a) == 0
        {
            if self.read(0x345) != 0 {
                self.write(0x340, 4);
                self.write(0x5d, 4);
            } else if self.read(0x56) != 0 {
                self.write(0x5d, 23);
                self.write(0x2e0, 1);
                let features = u32::from_le_bytes(self.ram.slice(0x64c, 4).try_into().unwrap());
                if features & 4096 != 0 {
                    self.runtime.load_gear_palettes_bunny(self.ram);
                }
            } else {
                self.write(0x5d, 0);
            }
            self.write(0x4d, 0);
            self.write(0x36b, 0);
            self.write(0x30d, 0);
            self.write(0x30a, 0);
            self.write(0x24, 0);
            self.write(0x46, 0);
            for k in 0..5 {
                self.clear(k);
            }
            return;
        }
        let k = 1;
        if self.step(k) == 0 {
            let timer = self.arr3(k).wrapping_sub(1);
            self.set_arr3(k, timer);
            if timer == 0 {
                self.set_arr3(k, timer.wrapping_add(1));
                self.set_z_velocity(k, 4);
                self.move_z(k);
                if self.height(k) >= 16 {
                    self.set_step(k, self.step(k).wrapping_add(1));
                    self.set_z_velocity(k, 2);
                }
            }
        } else {
            let timer = self.state_k(k).wrapping_sub(1);
            self.set_state_k(k, timer);
            if (timer as i8) < 0 {
                self.set_state_k(k, 32);
                self.set_z_velocity(k, self.z_velocity(k).wrapping_neg());
            }
            self.move_z(k);
        }
        self.write(0x24, self.height(k));
    }

    pub fn revival_fairy_main(&mut self) {
        const DELAY: [u8; 2] = [0, 0x90];
        const CHAR: [u8; 5] = [0x4b, 0x4d, 0x49, 0x47, 0x49];
        let k = 0;
        let mut draw = true;
        match self.step(k) {
            0 => {
                let timer = self.arr3(k).wrapping_sub(1);
                self.set_arr3(k, timer);
                if timer == 0 {
                    let step = self.step(k).wrapping_add(1);
                    self.set_step(k, step);
                    self.set_arr3(k, DELAY[usize::from(step)]);
                    self.set_state_k(k, 0);
                    self.set_z_velocity(k, 0);
                } else {
                    self.move_z(k);
                }
            }
            1 => {
                let timer = self.arr3(k).wrapping_sub(1);
                self.set_arr3(k, timer);
                if timer == 0 {
                    self.set_step(k, self.step(k).wrapping_add(1));
                    self.set_z_velocity(k, 0);
                    self.set_x_velocity(k, 0);
                } else {
                    if matches!(timer, 0x4f | 0x8f) {
                        self.set_state_l(k, self.state_l(k).wrapping_add(1));
                        self.sfx2_pan(k, 0x31);
                    }
                    if self.state_l(k) != 0 {
                        let animation = self.state_g(k).wrapping_sub(1);
                        self.set_state_g(k, animation);
                        if (animation as i8) < 0 {
                            self.set_state_g(k, 5);
                            let frame = self.item_to_link(k).wrapping_add(1);
                            if frame == 3 {
                                self.set_item_to_link(k, 0);
                                self.set_state_l(k, 0);
                            } else {
                                self.set_item_to_link(k, frame);
                            }
                        }
                    }
                    self.set_z_velocity(
                        k,
                        if self.state_k(k) != 0 {
                            self.z_velocity(k).wrapping_add(1)
                        } else {
                            self.z_velocity(k).wrapping_sub(1)
                        },
                    );
                    if (self.z_velocity(k) as i8).unsigned_abs() == 8 {
                        self.set_state_k(k, self.state_k(k) ^ 1);
                    }
                    self.move_z(k);
                }
            }
            2 => {
                if self.z_velocity(k) < 24 {
                    self.set_z_velocity(k, self.z_velocity(k).wrapping_add(1));
                }
                if self.x_velocity(k) < 16 {
                    self.set_x_velocity(k, self.x_velocity(k).wrapping_add(1));
                }
                self.move_x(k);
                self.move_z(k);
            }
            3 => draw = false,
            _ => unreachable!(),
        }
        if draw {
            self.runtime.allocate_oam_region(self.ram, b'C', 12);
            let (x, y) = self.prep_oam_coord(k);
            let mut frame = if self.step(k) == 1 && self.state_l(k) != 0 {
                self.item_to_link(k).wrapping_add(1)
            } else {
                0
            };
            if frame != 0 {
                frame = frame.wrapping_add(1);
            } else {
                frame = self.read(0x1a) >> 2 & 1;
            }
            let index = self.current_oam_index();
            self.set_oam(
                index,
                x,
                y.wrapping_sub(i16::from(self.height(k) as i8) as u16),
                CHAR[usize::from(frame)],
                0x74,
                2,
            );
            if oam::read(self.ram, index).y == 0xf0 {
                self.set_step(k, 3);
                self.write(0x11, self.read(0x11).wrapping_add(1));
                self.write(0x1c, self.read(0xc211));
            }
        }
        self.revival_fairy_dust();
        self.revival_fairy_monitor_hp();
    }

    pub fn game_over_text_draw(&mut self) {
        const CHAR: [u8; 16] = [
            0x40, 0x50, 0x41, 0x51, 0x42, 0x52, 0x43, 0x53, 0x44, 0x54, 0x45, 0x55, 0x43, 0x53,
            0x46, 0x56,
        ];
        oam::OamPointers::for_cursor(oam::OamCursor::from_index(0)).publish_entry_first(self.ram);
        let mut index = self.current_oam_index();
        let mut k = self.read(0x35f) as i8;
        loop {
            let slot = k as usize;
            self.set_oam(index, self.get_x(slot), 0x57, CHAR[slot * 2], 0x3c, 0);
            self.set_oam(
                index + 1,
                self.get_x(slot),
                0x5f,
                CHAR[slot * 2 + 1],
                0x3c,
                0,
            );
            index += 2;
            k -= 1;
            if k < 0 {
                break;
            }
        }
    }

    pub fn add_super_bomb_explosion(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.set_state_r(k, 0);
        self.set_step(k, 0);
        self.set_arr25(k, 0);
        self.set_state_l(k, 0);
        self.set_arr3(k, super::BOMB_FRAME_DELAYS[1]);
        self.set_item_to_link(k, 1);
        let history = usize::from(self.ram.read_u16(0x2cf));
        let y = u16::from_le_bytes([self.read(0x1a00 + history), self.read(0x1a14 + history)]);
        let x = u16::from_le_bytes([self.read(0x1a28 + history), self.read(0x1a3c + history)]);
        self.set_xy(k, x.wrapping_add(8), y.wrapping_add(16));
        Some(k)
    }

    pub fn configure_revival_ancillae(&mut self) {
        const POWDER_FRAME: [u8; 40] = [
            13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 10, 11, 12, 0, 1, 2, 3, 4, 5, 6, 16, 17, 18, 0, 1, 2,
            3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6,
        ];
        self.write(0x109, 80);
        self.set_arr3(0, 64);
        self.set_step(0, 0);
        self.set_z_velocity(0, 8);
        self.set_state_l(0, 0);
        self.set_state_g(0, 5);
        self.set_item_to_link(0, 0);
        self.set_state_k(0, 0);
        self.set_xy(0, self.ram.read_u16(0x22), self.ram.read_u16(0x20));
        self.set_height(0, 0);

        self.set_height(1, 0);
        self.set_arr3(1, 240);
        self.set_step(1, 0);
        self.set_state_k(1, 0);

        self.set_item_to_link(2, 2);
        self.set_aux_timer(2, 3);
        self.set_arr3(2, 8);
        self.set_step(2, 0);
        self.set_direction(2, 3);
        self.set_arr25(2, POWDER_FRAME[32]);
        self.set_xy(
            2,
            self.ram.read_u16(0x22).wrapping_add(20),
            self.ram.read_u16(0x20).wrapping_add(2),
        );
    }

    pub fn add_lamp_flame(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const X: [i8; 4] = [0, 0, -20, 18];
        const Y: [i8; 4] = [-16, 24, 4, 4];
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 0);
        self.set_aux_timer(k, 0);
        self.set_timer(k, 23);
        let direction = usize::from(self.read(0x2f) >> 1);
        self.set_direction(k, direction as u8);
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(i16::from(X[direction]) as u16),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(Y[direction]) as u16),
        );
        self.write(
            0x12e,
            misc::calculate_sfx_pan(self.ram, self.get_x(k)) | 0x2a,
        );
        Some(k)
    }

    pub fn add_master_sword_cutscene(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 0);
        self.set_aux_timer(k, 2);
        self.set_timer(k, 64);
        self.set_xy(
            k,
            self.ram.read_u16(0x22).wrapping_add(8),
            self.ram.read_u16(0x20).wrapping_sub(8),
        );
        Some(k)
    }

    pub fn add_blast_wall_fireball_from(&mut self, source: usize) {
        self.add_blast_wall_fireball(source);
    }

    pub fn add_bunny_poof(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.write(0x4b, 0x0c);
        self.set_step(k, 0);
        self.write(
            0x12e,
            misc::calculate_sfx_pan(self.ram, self.ram.read_u16(0x22))
                | if self.read(0x2e0) == 0 { 0x14 } else { 0x15 },
        );
        self.set_item_to_link(k, 0);
        self.set_aux_timer(k, 7);
        self.set_xy(
            k,
            self.ram.read_u16(0x22),
            self.ram.read_u16(0x20).wrapping_add(4),
        );
        Some(k)
    }

    pub fn add_cape_poof(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.set_step(k, 1);
        self.write(0x2e1, 1);
        self.write(0x50, self.read(0x50) | 1);
        self.write(0x67, 0);
        self.write(0x26, 0);
        self.set_item_to_link(k, 0);
        self.set_aux_timer(k, 7);
        self.set_xy(
            k,
            self.ram.read_u16(0x22),
            self.ram.read_u16(0x20).wrapping_add(4),
        );
        Some(k)
    }

    pub fn add_dwarf_poof(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.write(
            0x12e,
            misc::calculate_sfx_pan(self.ram, self.ram.read_u16(0x22))
                | if self.read(0xf3cc) == 8 { 0x14 } else { 0x15 },
        );
        self.set_item_to_link(k, 0);
        self.set_step(k, 0);
        self.set_aux_timer(k, 7);
        self.write(0x2f9, 1);
        let history = usize::from(self.read(0x2cf));
        let x = u16::from_le_bytes([self.read(0x1a28 + history), self.read(0x1a3c + history)]);
        let y = u16::from_le_bytes([self.read(0x1a00 + history), self.read(0x1a14 + history)]);
        self.set_xy(k, x, y.wrapping_add(4));
        Some(k)
    }

    pub fn add_bush_poof(&mut self, x: u16, y: u16) -> Option<usize> {
        if self.read(0x301) & 0x40 == 0 {
            return None;
        }
        let k = self.add_ancilla(0x3f, 4)?;
        self.set_item_to_link(k, 0);
        self.set_timer(k, 7);
        self.write(
            0x12e,
            misc::calculate_sfx_pan(self.ram, self.ram.read_u16(0x22)) | 21,
        );
        self.set_xy(k, x, y.wrapping_sub(2));
        Some(k)
    }

    pub fn add_ether_spell(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 0);
        self.set_arr25(k, 0);
        self.set_step(k, 0);
        self.write(0x112, 1);
        self.set_aux_timer(k, 2);
        self.set_arr3(k, 3);
        self.set_y_velocity(k, 127);
        self.write(0x15808, 40);
        self.write(0xaaa, 9);
        self.write(0x15812, 0x40);
        let sound = SoundRequest::at_player(self.ram, 0x26);
        SoundPort::EffectsB.overwrite(self.ram, sound);
        for i in 0..8 {
            self.write(0x15800 + i, (i * 8) as u8);
        }
        let link_y = self.ram.read_u16(0x20);
        let link_x = self.ram.read_u16(0x22);
        self.ram.write_u16(0x15813, link_y);
        let top = self.ram.read_u16(0xe8).wrapping_sub(16);
        self.ram.write_u16(0x1580c, top & 0x00f0);
        self.ram.write_u16(0x15815, link_x);
        self.ram.write_u16(0x1580e, link_x.wrapping_add(8));
        self.ram.write_u16(0x1580a, link_y.wrapping_sub(16));
        self.ram
            .write_u16(0x15810, link_y.wrapping_sub(16).wrapping_add(0x24));
        self.set_xy(k, link_x, top);
        Some(k)
    }

    pub fn add_victory_spin(&mut self) -> Option<usize> {
        if self.read(0xf359).wrapping_add(1) & 0xfe == 0 {
            return None;
        }
        let k = self.add_ancilla(0x3b, 0)?;
        self.set_item_to_link(k, 0);
        self.set_arr3(k, 1);
        self.set_aux_timer(k, 34);
        Some(k)
    }

    pub fn add_magic_powder(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const X: [i8; 4] = [-2, -2, -12, 12];
        const Y: [i8; 4] = [0, 20, 16, 16];
        const X1: [i8; 4] = [10, 10, -8, 28];
        const Y1: [i8; 4] = [1, 40, 22, 22];
        const FRAME: [u8; 4] = [13, 10, 16, 7];
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 0);
        self.set_height(k, 0);
        self.set_aux_timer(k, 1);
        self.write(0x109, 80);
        let direction = usize::from(self.read(0x2f) >> 1);
        self.set_direction(k, direction as u8);
        self.set_arr25(k, FRAME[direction]);
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(i16::from(X[direction]) as u16),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(Y[direction]) as u16),
        );
        self.check_tile_collision(k);
        self.write(0x333, self.tile_attribute(k));
        if self.read(0x304) == 9 {
            self.clear(k);
            return Some(k);
        }
        self.write(
            0x12e,
            misc::calculate_sfx_pan(self.ram, self.ram.read_u16(0x22)) | 0x0d,
        );
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(i16::from(X1[direction]) as u16),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(Y1[direction]) as u16),
        );
        Some(k)
    }

    pub fn add_wall_tap_spark(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const X: [i8; 4] = [11, 10, -12, 29];
        const Y: [i8; 4] = [-4, 32, 17, 17];
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 5);
        self.set_aux_timer(k, 1);
        let direction = usize::from(self.read(0x2f) >> 1);
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(i16::from(X[direction]) as u16),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(Y[direction]) as u16),
        );
        Some(k)
    }

    pub fn add_sword_swing_sparkle(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 0);
        self.set_aux_timer(k, 1);
        self.set_direction(k, self.read(0x2f) >> 1);
        self.set_xy(k, self.ram.read_u16(0x22), self.ram.read_u16(0x20));
        Some(k)
    }

    pub fn add_dash_tremor(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const DIRECTION: [u8; 4] = [2, 2, 0, 0];
        const EDGE: [u8; 2] = [0x80, 0x78];
        if (0..10).any(|slot| self.kind(slot) == kind) {
            return None;
        }
        let k = self.add_ancilla(kind, limit)?;
        self.set_item_to_link(k, 16);
        self.set_state_l(k, 0);
        let direction = DIRECTION[usize::from(self.read(0x2f) >> 1)];
        self.set_direction(k, direction);
        let screen_y = self
            .ram
            .read_u16(0x20)
            .wrapping_sub(self.ram.read_u16(0xe8)) as u8;
        let screen_x = self
            .ram
            .read_u16(0x22)
            .wrapping_sub(self.ram.read_u16(0xe2)) as u8;
        self.set_y(
            k,
            if (if direction != 0 { screen_y } else { screen_x })
                < EDGE[usize::from(direction >> 1)]
            {
                3
            } else {
                (-3_i16) as u16
            },
        );
        Some(k)
    }

    pub fn add_boomerang_wall_clink_from(&mut self, source: usize) {
        self.add_boomerang_wall_clink(source);
    }

    pub fn add_hookshot_wall_clink_from(&mut self, source: usize) {
        self.add_hookshot_wall_clink(source);
    }

    pub fn add_duck_take_off(&mut self, kind: u8, limit: u8) -> Option<usize> {
        if (0..=5).any(|slot| self.kind(slot) == kind) {
            return None;
        }
        let k = self.add_ancilla(kind, limit)?;
        self.set_timer(k, 0x78);
        self.set_state_l(k, 0);
        self.set_z_velocity(k, 0);
        self.set_height(k, 0);
        self.set_step(k, 0);
        self.add_bird_common(k);
        Some(k)
    }

    pub fn add_bird_travel(&mut self, kind: u8, limit: u8) -> Option<usize> {
        if (0..=5).any(|slot| self.kind(slot) == kind) {
            return None;
        }
        let k = self.add_ancilla(kind, limit)?;
        self.write(0x5d, 0);
        self.write(0x5e, 0);
        self.write(0x3a, self.read(0x3a) & !0x81);
        self.write(0x3c, 0);
        self.write(0x3d, 0);
        self.write(0x50, self.read(0x50) & !1);
        self.set_state_l(k, 1);
        let features = u32::from_le_bytes(self.ram.slice(0x64c, 4).try_into().unwrap());
        if features & 1 != 0 {
            self.set_z_velocity(k, 58);
            self.set_height(k, (-105_i8) as u8);
        } else {
            self.set_z_velocity(k, 40);
            self.set_height(k, (-51_i8) as u8);
        }
        self.set_step(k, 2);
        self.add_bird_common(k);
        Some(k)
    }

    pub fn add_quake_spell(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        self.set_step(k, 0);
        self.set_item_to_link(k, 0);
        self.write(0xaaa, 13);
        self.write(0x12e, 0x35);
        for i in 0..5 {
            self.write(0x15805 + i, 0);
            self.write(0x15800 + i, 1);
        }
        self.write(0x1580a, 0);
        self.write(0x112, 1);
        self.set_timer(k, 2);
        self.ram
            .write_u16(0x1580b, self.ram.read_u16(0x20).wrapping_add(26));
        self.ram
            .write_u16(0x1580d, self.ram.read_u16(0x22).wrapping_add(8));
        self.ram.write_u16(0x1581e, 3);
        Some(k)
    }

    pub fn add_spin_attack_init_spark(&mut self, kind: u8, step: u8, limit: u8) -> Option<usize> {
        const Y: [i8; 4] = [32, -8, 10, 20];
        const X: [i8; 4] = [10, 7, 28, -10];
        let k = self.add_ancilla(kind, limit)?;
        for slot in (0..=4).rev() {
            if self.kind(slot) == 0x31 {
                self.clear(slot);
            }
        }
        self.set_item_to_link(k, 0);
        self.set_step(k, step);
        self.set_timer(k, 4);
        self.set_aux_timer(k, 3);
        let direction = usize::from(self.read(0x2f) >> 1);
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(i16::from(X[direction]) as u16),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(Y[direction]) as u16),
        );
        Some(k)
    }

    pub fn add_fire_rod_shot(&mut self, kind: u8, _limit: u8) -> Option<usize> {
        const X: [i8; 4] = [0, 0, -8, 16];
        const Y: [i8; 4] = [-8, 16, 3, 3];
        const X_VEL: [i8; 4] = [0, 0, -64, 64];
        const Y_VEL: [i8; 4] = [-64, 64, 0, 0];
        const X_VEL2: [i8; 12] = [0, 0, -40, 40, 0, 0, -48, 48, 0, 0, -64, 64];
        const Y_VEL2: [i8; 12] = [-40, 40, 0, 0, -48, 48, 0, 0, -64, 64, 0, 0];

        let Some(mut k) = self.alloc_init(kind, 1) else {
            if kind != 1 {
                self.runtime.refund_magic(self.ram, 0);
            }
            return None;
        };
        let allocated_k = k;
        if kind != 1 {
            misc::ancilla_sfx2_near(self.ram, 0x0e);
        }
        self.set_kind(k, kind);
        self.set_sprite_count(k, PFLAGS[usize::from(kind)]);
        self.set_timer(k, 3);
        self.set_step(k, 0);
        self.set_item_to_link(k, 0);
        self.set_object_priority(k, 0);
        self.set_state_u(k, 0);
        let mut direction = usize::from(self.read(0x2f) >> 1);
        self.set_direction(k, direction as u8);
        if self.check_initial_tile_a(k) < 0 {
            self.set_xy(
                k,
                self.ram
                    .read_u16(0x22)
                    .wrapping_add(i16::from(X[direction]) as u16),
                self.ram
                    .read_u16(0x20)
                    .wrapping_add(i16::from(Y[direction]) as u16),
            );
            if kind != 1 {
                self.set_x_velocity(k, X_VEL[direction] as u8);
                self.set_y_velocity(k, Y_VEL[direction] as u8);
            } else {
                direction += usize::from(self.read(0xf359).wrapping_sub(2)) * 4;
                self.set_x_velocity(k, X_VEL2[direction] as u8);
                self.set_y_velocity(k, Y_VEL2[direction] as u8);
            }
            self.set_floor(k, self.read(0xee));
            self.set_floor2(k, self.read(0x476));
        } else if kind == 1 {
            self.set_kind(k, 4);
            self.set_timer(k, 7);
            self.set_sprite_count(k, 16);
        } else {
            self.set_step(k, 1);
            self.set_timer(k, 31);
            self.set_sprite_count(k, 8);
            k = usize::from(self.read(0x2f) >> 1);
            self.sfx2_pan(k, 0x2a);
        }
        Some(allocated_k)
    }

    /// Exact port of `AncillaAdd_IceRodShot`.
    pub fn add_ice_rod_shot(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const X: [i8; 4] = [0, 0, -20, 20];
        const Y: [i8; 4] = [-16, 24, 8, 8];
        const X_VEL: [i8; 4] = [0, 0, -48, 48];
        const Y_VEL: [i8; 4] = [-48, 48, 0, 0];

        let Some(k) = self.add_ancilla(kind, limit) else {
            self.runtime.refund_magic(self.ram, 0);
            return None;
        };
        self.write(0x12e, misc::link_calculate_sfx_pan(self.ram) | 15);
        self.set_step(k, 0);
        self.set_arr25(k, 0);
        self.set_item_to_link(k, 0xff);
        self.set_state_l(k, 1);
        self.set_aux_timer(k, 3);
        self.set_arr3(k, 6);
        let direction = usize::from(self.read(0x2f) >> 1);
        self.set_direction(k, direction as u8);
        self.set_y_velocity(k, Y_VEL[direction] as u8);
        self.set_x_velocity(k, X_VEL[direction] as u8);

        if self.check_initial_tile_a(k) < 0 {
            let x = self
                .ram
                .read_u16(0x22)
                .wrapping_add(i16::from(X[direction]) as u16);
            let y = self
                .ram
                .read_u16(0x20)
                .wrapping_add(i16::from(Y[direction]) as u16);
            if x.wrapping_sub(self.ram.read_u16(0xe2)) & 0xff00 != 0
                || y.wrapping_sub(self.ram.read_u16(0xe8)) & 0xff00 != 0
            {
                self.clear(k);
                return None;
            }
            self.set_xy(k, x, y);
        } else {
            self.set_kind(k, 0x11);
            self.set_sprite_count(k, PFLAGS[0x11]);
            self.set_item_to_link(k, 0);
            self.set_aux_timer(k, 4);
        }
        Some(k)
    }

    /// Exact port of `AncillaAdd_Splash`; the return value is true only when
    /// allocation failed, matching the inverted C result.
    pub fn add_splash(&mut self, kind: u8, limit: u8) -> bool {
        let Some(k) = self.add_ancilla(kind, limit) else {
            return true;
        };
        self.write(0x12e, misc::link_calculate_sfx_pan(self.ram) | 0x24);
        self.set_item_to_link(k, 0);
        self.set_aux_timer(k, 2);
        if self.read(0x1b) != 0 && self.read(0x345) == 0 {
            self.write(0xee, 0);
        }
        self.set_xy(
            k,
            self.ram.read_u16(0x22).wrapping_sub(11),
            self.ram.read_u16(0x20).wrapping_add(8),
        );
        false
    }

    /// Exact port of `AncillaAdd_GraveStone`.
    pub fn add_gravestone(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const ROW_Y: [u16; 8] = [0x550, 0x540, 0x530, 0x520, 0x500, 0x4e0, 0x4c0, 0x4b0];
        const X: [u16; 15] = [
            0x8b0, 0x8f0, 0x910, 0x950, 0x970, 0x9a0, 0x850, 0x870, 0x8b0, 0x8f0, 0x920, 0x950,
            0x880, 0x990, 0x840,
        ];
        const Y1: [u16; 15] = [
            0x540, 0x530, 0x530, 0x530, 0x520, 0x520, 0x510, 0x510, 0x4f0, 0x4f0, 0x4f0, 0x4f0,
            0x4d0, 0x4b0, 0x4a0,
        ];
        const X1: [u16; 15] = X;
        const POSITION: [u16; 15] = [
            0xa16, 0x99e, 0x9a2, 0x9aa, 0x92e, 0x934, 0x88a, 0x88e, 0x796, 0x79e, 0x7a4, 0x7aa,
            0x690, 0x5b2, 0x508,
        ];
        const COUNTER: [u8; 15] = [
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x38,
            0x58,
        ];
        const ROW_INDEX: [u8; 9] = [0, 1, 4, 6, 8, 12, 13, 14, 15];

        let k = self.add_ancilla(kind, limit)?;
        let link_y = self.ram.read_u16(0x20);
        let row_y = (if link_y & 0x0f < 7 {
            link_y
        } else {
            link_y.wrapping_add(16)
        }) & !0x0f;
        let Some(row) = (0..8).rev().find(|&i| ROW_Y[i] == row_y) else {
            self.clear(k);
            return None;
        };
        let mut j = usize::from(ROW_INDEX[row]);
        let end = usize::from(ROW_INDEX[row + 1]);
        loop {
            let x = X[j];
            if x < self.ram.read_u16(0x22) && x.wrapping_add(15) >= self.ram.read_u16(0x22) {
                let rejected = if j == 13 {
                    self.read(0x372) == 0
                } else {
                    self.read(0x372) != 0
                };
                if rejected {
                    break;
                }
                let position = POSITION[j];
                self.ram.write_u16(0x698, position);
                self.ram.write_u16(0x692, u16::from(COUNTER[j]));
                if COUNTER[j] == 0x58 {
                    self.write(0x12f, misc::link_calculate_sfx_pan(self.ram) | 0x1b);
                } else if COUNTER[j] == 0x38 {
                    let screen = usize::from(self.read(0x8a));
                    self.write(0xf280 + screen, self.read(0xf280 + screen) | 0x20);
                    self.write(0x12f, misc::link_calculate_sfx_pan(self.ram) | 0x1b);
                }
                let debris = position.wrapping_sub(0x80);
                self.write(0x732 + k, debris as u8);
                self.write(0x728 + k, (debris >> 8) as u8);
                self.runtime.overworld_do_map_update_32x32_b(self.ram);
                if self.read(0x12f) & 0x3f != 0x1b {
                    self.write(0x12e, misc::link_calculate_sfx_pan(self.ram) | 0x22);
                }
                let y = Y1[j];
                self.write(0x48, 4);
                self.write(0x3e9, 1);
                let lifted_y = y.wrapping_sub(18);
                self.set_state_a(k, lifted_y as u8);
                self.set_state_b(k, (lifted_y >> 8) as u8);
                self.set_xy(k, X1[j], y.wrapping_sub(2));
                return Some(k);
            }
            j += 1;
            if j == end {
                break;
            }
        }
        self.clear(k);
        None
    }

    /// Exact port of `AncillaAdd_WaterfallSplash`.
    pub fn add_waterfall_splash(&mut self) -> Option<usize> {
        if (0..=5).any(|k| self.kind(k) == 0x41) {
            return None;
        }
        let k = self.add_ancilla(0x41, 4)?;
        self.set_timer(k, 2);
        self.set_item_to_link(k, 0);
        Some(k)
    }

    /// Exact port of `AncillaAdd_GTCutscene`.
    pub fn add_ganons_tower_cutscene(&mut self) -> Option<usize> {
        if ((self.read(0x308) & 0x80) | self.read(0x4d)) != 0
            || self.read(0xf37a) & 0x7f != 0x7f
            || self.read(0xf280 + 0x43) & 0x20 != 0
        {
            return None;
        }

        self.runtime.terminate_sparkle_objects(self.ram);
        if (0..=5).any(|slot| self.kind(slot) == 0x43) {
            return None;
        }
        let k = self.add_ancilla(0x43, 4)?;

        for i in (0..16).rev() {
            if self.read(0xe20 + i) == 0x37 {
                self.write(0xdd0 + i, 0);
            }
        }
        for i in (0..=0x17).rev() {
            self.write(0x15837 + i, 0xff);
        }
        self.runtime.decode_animated_sprite_tile(self.ram, 0x28);
        self.write(0xab1, 4);
        self.ram.write_u16(0xaa8, 0x200);
        self.runtime
            .palette_load_sprite_environment_dungeon(self.ram);
        self.write(0x15, self.read(0x15).wrapping_add(1));
        self.write(0x2e4, 1);
        self.set_y_fraction(k, 0);
        self.set_x_fraction(k, 0);
        self.set_step(k, 0);
        self.write(0x15812, 240);
        self.write(0x15808, 0);
        for (i, angle) in [0, 10, 22, 32, 42, 54].into_iter().enumerate() {
            self.write(0x15800 + i, angle);
        }
        self.set_xy(
            k,
            self.ram.read_u16(0x22),
            self.ram.read_u16(0x20).wrapping_sub(16),
        );
        Some(k)
    }

    /// Exact port of `AncillaAdd_BlastWall`.
    pub fn add_blast_wall(&mut self) {
        const DELTA_X: [i8; 4] = [-16, 16, 0, 0];
        const DELTA_Y: [i8; 4] = [0, 0, -16, 16];
        const OFFSETS: [i8; 16] = [-8, 0, -8, 16, 16, 0, 16, 16, 0, -8, 16, -8, 0, 16, 16, 16];
        const SFX_PAN: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
        self.set_kind(0, 0x33);
        self.set_kind(1, 0x33);
        for k in 2..=5 {
            self.clear(k);
        }
        self.set_item_to_link(0, 0);
        self.write(0x2ec, 0);
        self.write(0x308, 0);
        self.write(0x50, 0);
        self.set_state_k(0, 0);
        self.set_floor(0, self.read(0xee));
        self.write(0xc7d, self.read(0xee));
        self.set_floor2(0, self.read(0x476));
        self.write(0x10010, 0);
        self.write(0x10009, 0);
        self.write(0x10001, 0);
        self.write(0x10011, 0);
        self.write(0x10000, 1);
        self.write(0x112, 1);
        self.write(0x10008, 3);

        let direction = usize::from(self.read(0x1001c));
        self.ram.write_u16(
            0x10018,
            self.ram
                .read_u16(0x10018)
                .wrapping_add(i16::from(DELTA_X[direction]) as u16),
        );
        self.ram.write_u16(
            0x1001a,
            self.ram
                .read_u16(0x1001a)
                .wrapping_add(i16::from(DELTA_Y[direction]) as u16),
        );
        let mut table = if direction < 4 { 4 } else { 0 };
        for k in (0..=3).rev() {
            let y = self
                .ram
                .read_u16(0x10018)
                .wrapping_add(i16::from(OFFSETS[table * 2]) as u16);
            let x = self
                .ram
                .read_u16(0x1001a)
                .wrapping_add(i16::from(OFFSETS[table * 2 + 1]) as u16);
            self.ram.write_u16(0x10020 + k * 2, y);
            self.ram.write_u16(0x10030 + k * 2, x);
            let screen_x = x.wrapping_sub(self.ram.read_u16(0xe2));
            if screen_x < 256 {
                self.write(0x12e, SFX_PAN[usize::from(screen_x >> 5)] | 0x0c);
            }
            table += 1;
        }
    }

    /// Exact port of `AncillaAdd_DoorDebris`.
    pub fn add_door_debris(&mut self) -> Option<usize> {
        let k = self.add_ancilla(8, 1)?;
        self.set_arr25(k, 0);
        self.set_arr26(k, 7);
        Some(k)
    }

    /// Exact port of `Ancilla_TerminateSelectInteractives`.
    pub fn terminate_select_interactives(&mut self, mut selected: u8) -> u8 {
        for i in (0..=5).rev() {
            if self.kind(i) == 0x3e {
                selected = i as u8;
            } else if self.kind(i) == 0x2c {
                self.write(0x646, 0);
                if self.read(0x48) & 0x80 != 0 {
                    self.write(0x48, 0);
                    self.write(0x5e, 0);
                }
            }
            if (self.read(0x308) as i8) < 0 {
                if i + 1 != usize::from(self.read(0x2ec)) {
                    self.clear(i);
                }
            } else {
                if i + 1 == usize::from(self.read(0x2ec)) {
                    self.write(0x2ec, 0);
                }
                self.clear(i);
            }
        }
        if self.read(0x37a) & 0x10 != 0 {
            self.write(0x46, 0);
            self.write(0x37a, 0);
        }
        for address in [0x3f0, 0x2f2, 0x2f3, 0x35f, 0x3fc, 0x37b, 0x3fd, 0x360] {
            self.write(address, 0);
        }
        if self.read(0x5d) == 19 {
            self.write(0x5d, 0);
            self.write(0x3a, self.read(0x3a) & !0x40);
            self.write(0x50, self.read(0x50) & !1);
            self.write(0x37a, self.read(0x37a) & !4);
            self.write(0x37e, 0);
        }
        selected
    }

    /// Exact port of `AncillaAdd_ExplodingSomariaBlock`.
    pub fn add_exploding_somaria_block(&mut self, k: usize) {
        self.set_kind(k, 0x2e);
        self.set_sprite_count(k, PFLAGS[0x2e]);
        self.set_aux_timer(k, 3);
        self.set_step(k, 0);
        self.set_item_to_link(k, 0);
        self.set_arr3(k, 0);
        self.set_arr1(k, 0);
        self.set_state_r(k, 0);
        self.set_object_priority(k, 0);
        self.write(0x646, 0);
        self.write(0x12f, misc::calculate_sfx_pan(self.ram, self.get_x(k)) | 1);
    }

    /// Exact port of `AncillaAdd_SomariaBlock`. This constructor was hidden by
    /// the old suffix-based audit because the exploding-block constructor had
    /// a similar name.
    pub fn ancilla_add_somaria_block(&mut self, kind: u8, limit: u8) -> Option<usize> {
        const X: [i8; 4] = [8, 8, -8, 23];
        const Y: [i8; 4] = [-8, 31, 17, 17];
        let k = self.add_ancilla(kind, limit)?;
        for j in (0..=4).rev() {
            if j == k || self.kind(j) != 0x2c {
                continue;
            }
            if usize::from(self.read(0x2ec)) == j + 1 {
                self.write(0x2ec, 0);
            }
            self.add_exploding_somaria_block(j);
            self.clear(k);
            self.write(0x646, 0);
            if self.read(0x5e) == 0x12 {
                self.write(0x48, 0);
                self.write(0x5e, 0);
            }
            return Some(k);
        }

        misc::ancilla_sfx3_near(self.ram, 0x2a);
        self.set_step(k, 0);
        self.set_y_velocity(k, 0);
        self.set_x_velocity(k, 0);
        self.set_item_to_link(k, 0);
        self.set_aux_timer(k, 0);
        self.set_arr3(k, 0);
        self.set_arr1(k, 0);
        self.set_state_h(k, 0);
        self.set_state_g(k, 12);
        self.set_timer(k, 18);
        self.set_state_l(k, 0);
        self.set_height(k, 0);
        self.set_state_k(k, 0);
        self.set_state_r(k, 0);
        self.set_arr4(k, 0);
        self.set_state_s(k, 9);
        self.set_state_t(k, 0);
        let direction = usize::from(self.read(0x2f) >> 1);
        self.set_direction(k, direction as u8);
        let link_x = self.ram.read_u16(0x22);
        let link_y = self.ram.read_u16(0x20);
        if self.check_initial_tile_collision_class2(k) {
            self.set_xy(k, link_x.wrapping_add(8), link_y.wrapping_add(16));
        } else {
            self.set_xy(
                k,
                link_x.wrapping_add(i16::from(X[direction]) as u16),
                link_y.wrapping_add(i16::from(Y[direction]) as u16),
            );
            self.somaria_block_check_for_transit_tile(k);
        }
        Some(k)
    }

    pub fn add_rupees(&mut self, k: usize) -> bool {
        self.item_receipt_add_rupees(k)
    }

    pub fn terminate_if_offscreen_from(&mut self, k: usize) {
        self.terminate_if_offscreen(k);
    }

    pub fn dash_tremor_twiddle_offset_from(&mut self, k: usize) -> u16 {
        self.dash_tremor_twiddle_offset(k)
    }

    fn terminate_if_offscreen(&mut self, k: usize) {
        let features = u32::from_le_bytes(self.ram.slice(0x64c, 4).try_into().unwrap());
        let xt = if features & 1 != 0 { 0x40 } else { 0 };
        let x = self
            .get_x(k)
            .wrapping_sub(self.ram.read_u16(0xe2))
            .wrapping_add(xt);
        let y = self.get_y(k).wrapping_sub(self.ram.read_u16(0xe8));
        if x >= 244 + xt * 2 || y >= 240 {
            self.clear(k);
        }
    }

    pub fn somaria_block_spawn_bullets(&mut self, k: usize) {
        const X: [i8; 4] = [-8, -8, -9, -4];
        const Y: [i8; 4] = [-15, -4, -8, -8];
        const X_VEL: [i8; 4] = [0, 0, -40, 40];
        const Y_VEL: [i8; 4] = [-40, 40, 0, 0];
        let z = match self.height(k) {
            0xff => 0,
            z => z,
        };
        let x = self.get_x(k);
        let y = self.get_y(k).wrapping_sub(u16::from(z));
        for direction in (0..4).rev() {
            if let Some(j) = self.alloc_init(1, 4) {
                self.set_kind(j, 1);
                self.set_sprite_count(j, PFLAGS[1]);
                self.set_step(j, 4);
                self.set_item_to_link(j, 0);
                self.set_object_priority(j, 0);
                self.set_direction(j, direction as u8);
                self.set_xy(
                    j,
                    x.wrapping_add(i16::from(X[direction]) as u16),
                    y.wrapping_add(i16::from(Y[direction]) as u16),
                );
                self.terminate_if_offscreen(j);
                self.set_x_velocity(j, X_VEL[direction] as u8);
                self.set_y_velocity(j, Y_VEL[direction] as u8);
                self.set_floor(j, self.floor(k));
                self.set_floor2(j, self.read(0x476));
            }
        }
        self.write(0xfb5, 0xff);
    }

    pub fn weapon_tink(&mut self) {
        if self.read(0xfac) == 0 {
            return;
        }
        self.write(0xfdc, 2);
        let delay = self.read(0xfaf).wrapping_sub(1);
        self.write(0xfaf, delay);
        if (delay as i8) < 0 {
            self.write(0xfac, self.read(0xfac).wrapping_sub(1));
            self.write(0xfaf, 1);
        }
        let region = if self.read(0xfb3) != 0 {
            if self.read(0xb68) != 0 { b'F' } else { b'D' }
        } else {
            b'A'
        };
        self.runtime.allocate_oam_region(self.ram, region, 0x10);
        let x = self.read(0xfad).wrapping_sub(self.read(0xe2));
        let y = self.read(0xfae).wrapping_sub(self.read(0xe8));
        if x >= 0xf8 || y >= 0xf0 {
            self.write(0xfac, 0);
            return;
        }
        let index = self.current_oam_index();
        let flags = [0x22, 0x12, 0x22, 0x22][usize::from(self.read(0xb68))];
        let timer = self.read(0xfac);
        if timer >= 3 {
            oam::set_plain(
                self.ram,
                index,
                x,
                y,
                if timer < 9 { 0x92 } else { 0x80 },
                flags,
                0,
            );
            return;
        }
        let charnum = [0x93, 0x82, 0x81][usize::from(timer)];
        for (i, (dx, dy, flip)) in [
            (-4_i8, -4_i8, 0x00),
            (4, -4, 0x40),
            (-4, 4, 0x80),
            (4, 4, 0xc0),
        ]
        .into_iter()
        .enumerate()
        {
            oam::set_plain(
                self.ram,
                index + i,
                x.wrapping_add(dx as u8),
                y.wrapping_add(dy as u8),
                charnum,
                flags | flip,
                0,
            );
        }
    }

    pub fn main(&mut self) {
        self.weapon_tink();
        self.execute_all();
    }

    fn return_if_outside_bounds(&mut self, k: usize) -> Option<super::AncillaOamInfo> {
        let x = self.x_low(k).wrapping_sub(self.read(0xe2));
        let y = self.y_low(k).wrapping_sub(self.read(0xe8));
        if x >= 0xf4 || y >= 0xf0 {
            self.clear(k);
            None
        } else {
            Some(super::AncillaOamInfo {
                x,
                y,
                flags: [0x20, 0x10][usize::from(self.floor(k))],
            })
        }
    }

    pub fn ice_rod_sparkle(&mut self, k: usize) {
        const X: [u8; 16] = [2, 7, 6, 1, 1, 7, 7, 1, 0, 7, 8, 1, 4, 9, 4, 0xff];
        const Y: [u8; 16] = [2, 3, 8, 7, 1, 1, 7, 7, 1, 0, 7, 8, 0xff, 4, 9, 4];
        const CHAR: [u8; 16] = [
            0x83, 0x83, 0x83, 0x83, 0xb6, 0x80, 0xb6, 0x80, 0xb7, 0xb6, 0xb7, 0xb6, 0xb7, 0xb6,
            0xb7, 0xb6,
        ];
        if self.timer(k) == 0 {
            self.clear(k);
        }
        if self.read(0x11) == 0 {
            self.move_x(k);
            self.move_y(k);
        }
        let Some(mut info) = self.return_if_outside_bounds(k) else {
            return;
        };
        if let Some(j) = (0..=4).rev().find(|&j| self.kind(j) == 0x0b)
            && self.object_priority(j) != 0
        {
            info.flags = 0x30;
        }
        let region = if self.read(0xfb3) != 0 {
            if self.floor(k) != 0 { b'E' } else { b'D' }
        } else {
            b'A'
        };
        self.runtime.allocate_oam_region(self.ram, region, 0x10);
        let index = self.current_oam_index();
        let frame = usize::from(self.timer(k) & 0x1c);
        for (oam_offset, i) in (0..4).rev().enumerate() {
            oam::set_plain(
                self.ram,
                index + oam_offset,
                info.x.wrapping_add(X[frame + i]),
                info.y.wrapping_add(Y[frame + i]),
                CHAR[frame + i],
                info.flags | 4,
                0,
            );
        }
    }

    pub fn add_ice_rod_sparkle(&mut self, k: usize) {
        const X_VEL: [i8; 4] = [0, 0, -4, 4];
        const Y_VEL: [i8; 4] = [-4, 4, 0, 0];
        if self.read(0x11) != 0 {
            return;
        }
        let counter = self.arr4(k).wrapping_sub(1);
        self.set_arr4(k, counter);
        if (counter as i8) >= 0 {
            return;
        }
        self.set_arr4(k, 5);
        if let Some(j) = self.alloc_high() {
            self.set_kind(j, 0x13);
            self.set_timer(j, 15);
            let direction = usize::from(self.direction(k));
            self.set_x_velocity(j, X_VEL[direction] as u8);
            self.set_y_velocity(j, Y_VEL[direction] as u8);
            self.set_x_low(j, self.x_low(k));
            self.set_y_low(j, self.y_low(k));
            self.set_floor(j, self.floor(k));
            self.set_sprite_count(j, 0);
        }
    }

    pub fn setup_hitbox(&self, k: usize) -> AncillaHitBox {
        const X: [i8; 12] = [4, 4, 4, 4, 3, 3, 2, 11, -16, -16, -1, -8];
        const Y: [i8; 12] = [4, 4, 4, 4, 2, 11, 3, 3, -1, -8, -16, -16];
        const W: [u8; 12] = [8, 8, 8, 8, 1, 1, 1, 1, 32, 32, 8, 8];
        const H: [u8; 12] = [8, 8, 8, 8, 1, 1, 1, 1, 8, 8, 32, 32];
        let mut direction = usize::from(self.direction(k));
        if self.kind(k) == 0x0c {
            direction |= 8;
        }
        AncillaHitBox {
            x: self.get_x(k).wrapping_add(i16::from(X[direction]) as u16),
            y: self.get_y(k).wrapping_add(i16::from(Y[direction]) as u16),
            width: W[direction],
            height: H[direction],
        }
    }

    pub fn check_sprite_collision_single(&mut self, k: usize, sprite: usize) -> bool {
        let hitbox = self.setup_hitbox(k);
        if !self
            .runtime
            .sprite_hitbox_overlaps(self.ram, sprite, hitbox)
        {
            return false;
        }
        let mut return_value = true;
        if self.read(0xb6b + sprite) & 8 != 0 && self.kind(k) == 9 {
            if self.read(0xe20 + sprite) != 0x1b {
                self.runtime.sprite_create_deflected_arrow(self.ram, k);
                return false;
            }
            if self.read(0xf340) < 3 {
                self.runtime.sprite_create_deflected_arrow(self.ram, k);
            } else {
                return_value = false;
            }
        }
        if self.read(0xcaa + sprite) & 0x10 != 0 {
            const OPPOSITE: [u8; 4] = [2, 3, 0, 1];
            let direction = self.direction(k) & 3;
            self.set_direction(k, direction);
            if direction == OPPOSITE[usize::from(direction)] {
                self.write(0xbb0 + sprite, self.kind(k));
                self.write(0xfdc, 3);
                return return_value;
            }
        }
        if self.kind(k) == 5 || self.kind(k) == 0x1f {
            let hookshot_exception = self.kind(k) == 0x1f && self.read(0xe20 + sprite) == 0x8d;
            if !hookshot_exception && self.read(0xef0 + sprite) != 0 {
                self.write(0xbb0 + sprite, self.kind(k));
                self.write(0xfdc, 3);
                return return_value;
            }
            if hookshot_exception || self.read(0xcaa + sprite) & 2 != 0 {
                self.write(0xda0 + sprite, k as u8 + 1);
                self.write(0xbb0 + sprite, self.kind(k));
                self.write(0xfdc, 3);
                return return_value;
            }
        }
        if self.read(0xba0 + sprite) == 0 {
            if self.read(0xe20 + sprite) == 0x92 && self.read(0xdb0 + sprite) < 3 {
                self.write(0xbb0 + sprite, self.kind(k));
                self.write(0xfdc, 3);
                return return_value;
            }
            let direction = usize::from(self.direction(k) & 3);
            self.write(0xf40 + sprite, [0, 0, 0xc0, 0x40][direction]);
            self.write(0xf30 + sprite, [0xc0, 0x40, 0, 0][direction]);
            self.write(0xfb6, k as u8);
            self.check_damage_to_sprite(sprite, self.kind(k));
            self.write(0xbb0 + sprite, self.kind(k));
            self.write(0xfdc, 3);
            return return_value;
        }
        false
    }

    pub fn check_sprite_collision(&mut self, k: usize) -> Option<usize> {
        for sprite in (0..16).rev() {
            let scheduled = matches!(self.kind(k), 9 | 0x1f)
                || ((((sprite as u8) ^ self.read(0x1a)) & 3) | self.read(0xf00 + sprite)) == 0;
            if scheduled
                && self.read(0xdd0 + sprite) >= 9
                && (self.read(0xcaa + sprite) & 2 != 0 || self.object_priority(k) == 0)
                && self.floor(k) == self.read(0xf20 + sprite)
                && self.check_sprite_collision_single(k, sprite)
            {
                return Some(sprite);
            }
        }
        None
    }

    pub fn setup_basic_hitbox(&self, k: usize) -> AncillaHitBox {
        AncillaHitBox {
            x: self.get_x(k).wrapping_sub(8),
            y: self
                .get_y(k)
                .wrapping_sub(8)
                .wrapping_sub(u16::from(self.height(k))),
            width: 15,
            height: 15,
        }
    }

    pub fn check_basic_sprite_collision_single(&mut self, k: usize, sprite: usize) -> bool {
        let hitbox = self.setup_basic_hitbox(k);
        if !self
            .runtime
            .sprite_hitbox_overlaps(self.ram, sprite, hitbox)
        {
            return false;
        }
        if self.read(0xe20 + sprite) == 0x92 && self.read(0xdb0 + sprite) < 3 {
            return true;
        }
        if self.read(0xe20 + sprite) == 0x80 && self.read(0xf10 + sprite) == 0 {
            self.write(0xf10 + sprite, 24);
            self.write(0xde0 + sprite, self.read(0xde0 + sprite) ^ 1);
        }
        if self.read(0xba0 + sprite) != 0 {
            return false;
        }
        let x = self.get_x(k).wrapping_sub(8);
        let y = self
            .get_y(k)
            .wrapping_sub(8)
            .wrapping_sub(u16::from(self.height(k)));
        let speed = self
            .runtime
            .sprite_project_speed_towards_location(self.ram, sprite, x, y, 80);
        self.write(0xf30 + sprite, !speed.y);
        self.write(0xf40 + sprite, !speed.x);
        self.check_damage_to_sprite(sprite, self.kind(k));
        true
    }

    pub fn check_basic_sprite_collision(&mut self, k: usize) -> Option<usize> {
        for sprite in (0..16).rev() {
            if ((((sprite as u8) ^ self.read(0x1a)) & 3)
                | self.read(0xf00 + sprite)
                | self.read(0xef0 + sprite))
                != 0
            {
                continue;
            }
            if self.read(0xdd0 + sprite) < 9
                || (self.read(0xcaa + sprite) & 2 == 0 && self.object_priority(k) != 0)
                || self.floor(k) != self.read(0xf20 + sprite)
                || (self.kind(k) == 0x2c && matches!(self.read(0xe20 + sprite), 0x1e | 0x90))
            {
                continue;
            }
            if self.check_basic_sprite_collision_single(k, sprite) {
                return Some(sprite);
            }
        }
        None
    }

    pub fn beam_hit(&mut self, k: usize) {
        const X: [i8; 16] = [-12, 20, -12, 20, -8, 16, -8, 16, -4, 12, -4, 12, 0, 8, 0, 8];
        const Y: [i8; 16] = [-12, -12, 20, 20, -8, -8, 16, 16, -4, -4, 12, 12, 0, 0, 8, 8];
        const CHAR: [u8; 16] = [
            0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x54, 0x54,
            0x54, 0x54,
        ];
        const FLAGS: [u8; 16] = [
            0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0, 0x40, 0x80, 0xc0,
        ];
        let Some(info) = self.return_if_outside_bounds(k) else {
            return;
        };
        if self.timer(k) == 0 {
            self.clear(k);
            return;
        }
        let index = self.current_oam_index();
        let frame = usize::from(self.timer(k) >> 1) * 4;
        let ancilla_x = self.get_x(k);
        let ancilla_y = self.get_y(k);
        let r7 = ancilla_x.wrapping_sub(self.ram.read_u16(0xe2)) as u8;
        let r6 = ancilla_y.wrapping_sub(self.ram.read_u16(0xe8)) as u8;
        for (oam_offset, i) in (0..4).rev().enumerate() {
            let m = frame + i;
            let x = info.x.wrapping_add(X[m] as u8);
            let y = info.y.wrapping_add(Y[m] as u8);
            let x_adjusted = ancilla_x
                .wrapping_add(i16::from(x.wrapping_sub(r7) as i8) as u16)
                .wrapping_sub(self.ram.read_u16(0xe2));
            let y_adjusted = ancilla_y
                .wrapping_add(i16::from(y.wrapping_sub(r6) as i8) as u16)
                .wrapping_sub(self.ram.read_u16(0xe8))
                .wrapping_add(0x10);
            oam::set_plain(
                self.ram,
                index + oam_offset,
                x,
                if y_adjusted >= 0x100 { 0xf0 } else { y },
                CHAR[m].wrapping_add(0x82),
                FLAGS[m] | 2 | info.flags,
                u8::from(x_adjusted >= 0x100),
            );
        }
    }

    fn fire_rod_shot_become_skull_woods_fire(&mut self) {
        if self.read(0x1b) != 0 || self.read(0x8a) & 0x40 == 0 {
            return;
        }
        self.set_kind(0, 0x34);
        for slot in 1..=5 {
            self.clear(slot);
        }
        self.write(0x35f, 0);
        self.set_sprite_count(0, PFLAGS[0x34]);
        for (i, value) in [253, 254, 255, 0].into_iter().enumerate() {
            self.write(0x10000 + i, value);
            self.write(0x10008 + i, 5);
        }
        self.write(0x10010, 0);
        self.set_aux_timer(0, 5);
        self.ram.write_u16(0x10018, 0x100);
        self.ram.write_u16(0x10026, 0x100);
        self.ram.write_u16(0x1001a, 0x98);
        self.ram.write_u16(0x10036, 0x98);
        self.write(0x4c6, 2);
        self.write(0xb0, 0);
        self.write(0xc8, 0);
        self.set_floor(0, self.read(0xee));
        self.set_floor2(0, self.read(0x476));
        self.set_item_to_link(0, 0);
        self.set_step(0, 0);
    }

    pub fn fire_shot_draw(&mut self, k: usize) {
        const X: [u8; 16] = [7, 0, 8, 0, 8, 4, 0, 0, 2, 8, 0, 0, 1, 4, 9, 0];
        const Y: [u8; 16] = [1, 4, 9, 0, 7, 0, 8, 0, 8, 4, 0, 0, 2, 8, 0, 0];
        const CHAR: [u8; 3] = [0x8d, 0x9d, 0x9c];
        let Some(mut info) = self.return_if_outside_bounds(k) else {
            return;
        };
        if self.object_priority(k) != 0 {
            info.flags |= 0x30;
        }
        let index = self.current_oam_index();
        let frame = usize::from(self.item_to_link(k) & 0x0c);
        for (oam_offset, i) in (0..3).rev().enumerate() {
            oam::set_plain(
                self.ram,
                index + oam_offset,
                info.x.wrapping_add(X[frame + i]),
                info.y.wrapping_add(Y[frame + i]),
                CHAR[i],
                info.flags | 2,
                0,
            );
        }
    }

    pub fn fire_rod_shot(&mut self, k: usize) {
        if self.step(k) == 0 {
            if self.read(0x11) == 0 {
                self.set_state_l(k, 0);
                self.move_x(k);
                self.move_y(k);
                let mut collided = self.check_sprite_collision(k).is_some();
                if !collided {
                    self.set_direction(k, self.direction(k) | 8);
                    collided = self.check_tile_collision(k) != 0;
                    self.set_state_l(k, self.tile_attribute(k));
                    if !collided {
                        self.set_direction(k, self.direction(k) | 12);
                        let saved = self.state_u(k);
                        collided = self.check_tile_collision(k) != 0;
                        self.set_state_u(k, saved);
                    }
                }
                if collided {
                    self.set_step(k, self.step(k).wrapping_add(1));
                    self.set_timer(k, 31);
                    self.set_sprite_count(k, 8);
                    self.sfx2_pan(k, 0x2a);
                }
                self.set_item_to_link(k, self.item_to_link(k).wrapping_add(1));
                self.set_direction(k, self.direction(k) & !0x0c);
                let attr_l = self.state_l(k);
                self.write(0x333, attr_l);
                let mut torch = attr_l & 0xf0 == 0xc0;
                if !torch {
                    let attr = self.tile_attribute(k);
                    self.write(0x333, attr);
                    torch = attr & 0xf0 == 0xc0;
                }
                if torch {
                    self.runtime.dungeon_light_torch(self.ram);
                }
            }
            self.fire_shot_draw(k);
            return;
        }
        self.check_basic_sprite_collision(k);
        let Some(info) = self.return_if_outside_bounds(k) else {
            return;
        };
        let index = self.current_oam_index();
        if self.timer(k) == 0 {
            let old_type = self.kind(k);
            self.clear(k);
            if old_type != 0x2f && self.read(0x8a) == 64 && self.tile_attribute(k) == 0x43 {
                self.fire_rod_shot_become_skull_woods_fire();
            }
            return;
        }
        let frame = self.timer(k) >> 3;
        if frame != 0 {
            oam::set_plain(
                self.ram,
                index,
                info.x,
                info.y,
                [0xa2, 0xa0, 0x8e][usize::from(frame - 1)],
                info.flags | 2,
                2,
            );
        } else {
            oam::set_plain(
                self.ram,
                index,
                info.x,
                info.y.wrapping_sub(3),
                0xa4,
                info.flags | 2,
                0,
            );
            oam::set_plain(
                self.ram,
                index + 1,
                info.x.wrapping_add(8),
                info.y.wrapping_sub(3),
                0xa5,
                info.flags | 2,
                0,
            );
        }
    }

    /// Compatibility entry point for `Ancilla_AllocateOamFromCustomRegion`.
    pub fn allocate_oam_from_custom_region(&mut self, index: usize) -> usize {
        self.wrap_custom_oam_cursor(oam::OamCursor::from_index(index))
            .index()
    }

    fn wrap_custom_oam_cursor(&mut self, cursor: oam::OamCursor) -> oam::OamCursor {
        oam::EffectOamLayout::from_sort_setting(self.read(0xfb3))
            .wrap_effect_cursor(self.ram, cursor)
    }

    fn add_boomerang_wall_clink(&mut self, source: usize) {
        const X: [i8; 8] = [8, 8, 0, 10, 12, 8, 4, 0];
        const Y: [i8; 8] = [0, 8, 8, 8, 4, 8, 12, 8];
        const OFFSET: [u8; 16] = [0, 6, 4, 0, 2, 10, 12, 0, 0, 8, 14, 0, 0, 0, 0, 0];
        let x = self.get_x(source);
        let y = self.get_y(source);
        self.ram.write_u16(0x39b, x);
        self.ram.write_u16(0x399, y);
        if let Some(k) = self.add_ancilla(6, 1) {
            self.set_item_to_link(k, 0);
            self.set_arr3(k, 1);
            let direction = usize::from(OFFSET[usize::from(self.read(0x39d))] >> 1);
            self.set_xy(
                k,
                x.wrapping_add(i16::from(X[direction]) as u16),
                y.wrapping_add(i16::from(Y[direction]) as u16),
            );
        }
    }

    fn add_sword_charge_sparkle(&mut self, source: usize) {
        let Some(k) = (0..10).rev().find(|&slot| self.kind(slot) == 0) else {
            return;
        };
        self.set_kind(k, 60);
        self.set_floor(k, self.read(0xee));
        self.set_item_to_link(k, 0);
        self.set_timer(k, 4);
        let random = misc::get_random_number(self.ram);
        let mut z = self.height(source);
        if z >= 0xf8 {
            z = 0;
        }
        self.set_xy(
            k,
            self.get_x(source).wrapping_add(2 + u16::from(random >> 5)),
            self.get_y(source)
                .wrapping_sub(2)
                .wrapping_sub(u16::from(z))
                .wrapping_add(u16::from(random & 0x0f)),
        );
    }

    #[must_use]
    pub fn boomerang_screen_edge(&self, k: usize) -> bool {
        let x = self.get_x(k);
        let y = self.get_y(k);
        let effect = self.read(0x39d);
        if effect & 3 != 0 {
            let adjusted = x
                .wrapping_add(if effect & 1 != 0 { 16 } else { 0 })
                .wrapping_sub(self.ram.read_u16(0xe2));
            if adjusted >= 0x100 {
                return true;
            }
        }
        if effect & 12 != 0 {
            let adjusted = y
                .wrapping_add(if effect & 4 != 0 { 16 } else { 0 })
                .wrapping_sub(self.ram.read_u16(0xe8));
            if adjusted >= 0xe2 {
                return true;
            }
        }
        false
    }

    pub fn boomerang_terminate(&mut self, k: usize) {
        self.clear(k);
        self.write(0x35f, 0);
        if self.read(0x301) & 0x80 != 0 {
            self.write(0x301, 0);
            self.write(0x3a, self.read(0x3a) & !0x40);
            if self.read(0x3a) & 0x80 == 0 {
                self.write(0x50, self.read(0x50) & !1);
            }
        }
    }

    pub fn boomerang_stop_offscreen(&mut self, k: usize) {
        let x = self.get_x(k).wrapping_add(8);
        let y = self.get_y(k).wrapping_add(8);
        let link_x = self.ram.read_u16(0x22);
        let link_y = self.ram.read_u16(0x20);
        if x >= link_x && x < link_x.wrapping_add(16) && y >= link_y && y < link_y.wrapping_add(24)
        {
            self.boomerang_terminate(k);
        }
    }

    pub fn boomerang_draw(&mut self, k: usize) {
        const FLAGS: [u8; 8] = [0xa4, 0xe4, 0x64, 0x24, 0xa2, 0xe2, 0x62, 0x22];
        const XY: [i8; 8] = [2, -2, 2, 2, -2, 2, -2, -2];
        let (x0, y0) = self.prep_oam_coord(k);
        if self.item_to_link(k) != 0 {
            let floor = self.read(0xee);
            self.set_floor(k, floor);
            self.ram.write_u16(
                0x64,
                u16::from([0x20_u8, 0x10, 0x30, 0x20][usize::from(floor)]) << 8,
            );
        }
        if self.object_priority(k) != 0 {
            self.ram.write_u16(0x64, 0x3000);
        }
        if self.read(0x11) == 0 && self.aux_timer(k) != 0 {
            let countdown = self.arr3(k).wrapping_sub(1);
            self.set_arr3(k, countdown);
            if (countdown as i8) < 0 {
                self.set_arr3(k, [3, 2][usize::from(self.state_g(k))]);
                let delta = if self.state_s(k) != 0 { 0xff } else { 1 };
                self.set_arr1(k, self.arr1(k).wrapping_add(delta) & 3);
            }
        }
        let frame = usize::from(self.arr1(k));
        let x = x0.wrapping_add(i16::from(XY[frame * 2 + 1]) as u16);
        let y = y0.wrapping_add(i16::from(XY[frame * 2]) as u16);
        if self.aux_timer(k) == 0 {
            let offset = [0x180_u16, 0xd0][usize::from(self.read(0xfb3))];
            oam::OamPointers::for_cursor(oam::OamCursor::from_byte_offset(usize::from(offset)))
                .publish_extended_first(self.ram);
        }
        self.set_oam_safe(
            self.current_oam_index(),
            x,
            y,
            0x26,
            (FLAGS[usize::from(self.state_g(k)) * 4 + frame] & !0x30)
                | (self.ram.read_u16(0x64) >> 8) as u8,
            2,
        );
    }

    pub fn boomerang(&mut self, k: usize) {
        const X: [i8; 8] = [0, 0, -8, 8, 8, 8, -8, -8];
        const Y: [i8; 8] = [-16, 6, 0, 0, -8, 8, -8, 8];
        if (0..=4).any(|slot| self.kind(slot) == 0x22) || self.read(0x11) != 0 {
            self.boomerang_draw(k);
            return;
        }
        if self.read(0x1a) & 7 == 0 {
            self.sfx2_pan(k, 9);
        }
        if self.aux_timer(k) == 0 {
            if self.read(0x3c) < 9 && self.read(0x300) == 0 {
                let features = u32::from_le_bytes(self.ram.slice(0x64c, 4).try_into().unwrap());
                if self.read(0x2e0) != 0
                    || self.read(0x4d) != 0
                    || (self.read(0x301) == 0 && features & 4096 != 0)
                {
                    self.boomerang_terminate(k);
                    return;
                }
                self.boomerang_draw(k);
                return;
            }
            let direction = usize::from(self.arr23(k) >> 1);
            self.set_xy(
                k,
                self.ram
                    .read_u16(0x22)
                    .wrapping_add(i16::from(X[direction]) as u16),
                self.ram
                    .read_u16(0x20)
                    .wrapping_add(8)
                    .wrapping_add(i16::from(Y[direction]) as u16),
            );
            self.set_aux_timer(k, 1);
        }
        if self.state_g(k) != 0 && self.read(0x1a) & 1 == 0 {
            self.add_sword_charge_sparkle(k);
        }
        if self.item_to_link(k) != 0 {
            if self.state_k(k) != 0 {
                self.set_state_k(k, self.state_k(k).wrapping_add(1));
            }
            let old_link_y = self.ram.read_u16(0x20);
            self.set_carried_player_y(k, old_link_y);
            self.ram.write_u16(0x20, old_link_y.wrapping_add(8));
            let mut speed = self.project_speed_towards_player(k, self.state_h(k));
            self.boomerang_cheat(k, &mut speed);
            self.set_x_velocity(k, speed.x);
            self.set_y_velocity(k, speed.y);
            self.ram.write_u16(0x20, self.carried_player_y(k));
        }
        if self.y_velocity(k) != 0 {
            self.set_y_velocity(k, self.y_velocity(k).wrapping_add(self.state_k(k)));
        }
        self.move_y(k);
        if self.x_velocity(k) != 0 {
            self.set_x_velocity(k, self.x_velocity(k).wrapping_add(self.state_k(k)));
        }
        self.move_x(k);
        let hit_sprite = self.check_sprite_collision(k).is_some();
        if self.item_to_link(k) == 0 {
            if hit_sprite {
                self.set_item_to_link(k, 1);
            } else if self.check_tile_collision(k) != 0 {
                self.add_boomerang_wall_clink(k);
                self.sfx2_pan(k, if self.tile_attribute(k) == 0xf0 { 6 } else { 5 });
                self.set_item_to_link(k, 1);
            } else if self.boomerang_screen_edge(k) {
                self.set_item_to_link(k, 1);
            } else {
                let step = self.step(k).wrapping_sub(1);
                self.set_step(k, step);
                if step == 0 {
                    self.set_item_to_link(k, 1);
                } else if step < 5 {
                    self.set_state_k(k, self.state_k(k).wrapping_sub(1));
                }
            }
        } else {
            let priority = self.object_priority(k);
            let floor = self.floor(k);
            self.set_floor(k, 0);
            self.check_tile_collision(k);
            self.set_floor(k, floor);
            self.set_object_priority(k, priority);
            self.boomerang_stop_offscreen(k);
        }
        self.boomerang_draw(k);
    }

    pub fn wall_hit_draw(&mut self, k: usize) {
        const X: [i8; 32] = [
            -4, 0, 0, 0, -4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -4, 0, 0, 0, -4, 0,
            0, 0, -8, 0, 0, 0,
        ];
        const Y: [i8; 32] = [
            -4, 0, 0, 0, -4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -4, 0, 0, 0, -4, 0,
            0, 0, -8, 0, 0, 0,
        ];
        const CHAR: [u8; 32] = [
            0x80, 0, 0, 0, 0x92, 0, 0, 0, 0x81, 0x81, 0x81, 0x81, 0x82, 0x82, 0x82, 0x82, 0x93,
            0x93, 0x93, 0x93, 0x92, 0, 0, 0, 0xb9, 0, 0, 0, 0x90, 0x90, 0, 0,
        ];
        const FLAGS: [u8; 32] = [
            0x32, 0, 0, 0, 0x32, 0, 0, 0, 0x32, 0x72, 0xb2, 0xf2, 0x32, 0x72, 0xb2, 0xf2, 0x32,
            0x72, 0xb2, 0xf2, 0x32, 0, 0, 0, 0x72, 0, 0, 0, 0x32, 0xf2, 0, 0,
        ];
        let (x, y) = self.prep_oam_coord(k);
        let mut frame = usize::from(self.item_to_link(k)) * 4;
        let mut index = self.current_oam_index();
        for _ in (0..4).rev() {
            if CHAR[frame] != 0 {
                self.set_oam(
                    index,
                    x.wrapping_add(i16::from(X[frame]) as u16),
                    y.wrapping_add(i16::from(Y[frame]) as u16),
                    CHAR[frame],
                    (FLAGS[frame] & !0x30) | (self.ram.read_u16(0x64) >> 8) as u8,
                    0,
                );
                index += 1;
            }
            index = self.allocate_oam_from_custom_region(index);
            frame += 1;
        }
    }

    pub fn wall_hit(&mut self, k: usize) {
        let countdown = self.arr3(k).wrapping_sub(1);
        self.set_arr3(k, countdown);
        if (countdown as i8) < 0 {
            let frame = self.item_to_link(k).wrapping_add(1);
            if frame == 5 {
                self.clear(k);
                return;
            }
            self.set_item_to_link(k, frame);
            self.set_arr3(k, 1);
        }
        self.wall_hit_draw(k);
    }

    pub fn sword_wall_hit(&mut self, k: usize) {
        self.write(0xfdc, 3);
        let countdown = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, countdown);
        if (countdown as i8) < 0 {
            let frame = self.item_to_link(k).wrapping_add(1);
            if frame == 8 {
                self.clear(k);
                return;
            }
            self.set_item_to_link(k, frame);
            self.set_aux_timer(k, 1);
        }
        self.wall_hit_draw(k);
    }

    pub fn door_debris_draw(&mut self, k: usize) {
        const XY: [u16; 64] = [
            4, 7, 3, 17, 8, 8, 7, 17, 11, 7, 10, 16, 16, 7, 17, 17, 20, 7, 21, 17, 16, 8, 17, 17,
            13, 7, 14, 16, 8, 7, 7, 17, 7, 4, 17, 3, 8, 8, 17, 7, 7, 11, 16, 10, 7, 16, 17, 17, 7,
            20, 17, 21, 8, 16, 17, 17, 7, 13, 16, 14, 7, 8, 17, 7,
        ];
        const CHAR_FLAGS: [u16; 32] = [
            0x205e, 0xe05e, 0xa05e, 0x605e, 0x204f, 0x204f, 0x204f, 0x204f, 0x605e, 0x605e, 0x205e,
            0xe05e, 0x604f, 0x604f, 0x604f, 0x604f, 0x205e, 0xe05e, 0xa05e, 0x605e, 0x204f, 0xe04f,
            0x204f, 0x204f, 0x605e, 0x605e, 0x205e, 0xe05e, 0x604f, 0x604f, 0x604f, 0x604f,
        ];
        self.prep_adjusted_oam_coord(k);
        let y = self
            .ram
            .read_u16(0x732 + k * 2)
            .wrapping_sub(self.ram.read_u16(0xe8));
        let x = self
            .ram
            .read_u16(0x728 + k * 2)
            .wrapping_sub(self.ram.read_u16(0xe2));
        let frame = usize::from(self.arr25(k)) + usize::from(self.read(0x73c + k)) * 4;
        let mut index = self.current_oam_index();
        for i in 0..2 {
            let t = frame * 2 + i;
            let data = CHAR_FLAGS[t];
            self.set_oam(
                index,
                x.wrapping_add(XY[t * 2 + 1]),
                y.wrapping_add(XY[t * 2]),
                data as u8,
                ((data >> 8) as u8 & 0xc0) | (self.ram.read_u16(0x64) >> 8) as u8,
                0,
            );
            index = self.allocate_oam_from_custom_region(index + 1);
        }
    }

    pub fn door_debris(&mut self, k: usize) {
        self.door_debris_draw(k);
        let countdown = self.arr26(k).wrapping_sub(1);
        self.set_arr26(k, countdown);
        if (countdown as i8) < 0 {
            self.set_arr26(k, 7);
            let frame = self.arr25(k).wrapping_add(1);
            self.set_arr25(k, frame);
            if frame == 4 {
                self.clear(k);
            }
        }
    }

    fn add_silver_arrow_sparkle(&mut self, source: usize) {
        const X: [i8; 4] = [-4, -4, 0, 2];
        const Y: [i8; 4] = [0, 2, -4, -4];
        let Some(k) = self.alloc_high() else {
            return;
        };
        self.set_kind(k, 0x3c);
        self.set_item_to_link(k, 0);
        self.set_timer(k, 4);
        self.set_floor(k, self.read(0xee));
        let random = misc::get_random_number(self.ram);
        let direction = usize::from(self.direction(source) & 3);
        self.set_xy(
            k,
            self.get_x(source)
                .wrapping_add(i16::from(X[direction]) as u16)
                .wrapping_add(u16::from(random >> 4 & 7)),
            self.get_y(source)
                .wrapping_add(i16::from(Y[direction]) as u16)
                .wrapping_add(u16::from(random & 7)),
        );
    }

    pub fn arrow_draw(&mut self, k: usize) {
        const CHAR: [u8; 48] = [
            0x2b, 0x2a, 0x2a, 0x2b, 0x3d, 0x3a, 0x3a, 0x3d, 0x2b, 0xff, 0x2b, 0xff, 0x3d, 0xff,
            0x3d, 0xff, 0x3c, 0x2c, 0x3c, 0x2a, 0x3c, 0x2c, 0x3c, 0x2a, 0x2c, 0x3c, 0x2a, 0x3c,
            0x2c, 0x3c, 0x2a, 0x3c, 0x3b, 0x2d, 0x3b, 0x3a, 0x3b, 0x2d, 0x3b, 0x3a, 0x2d, 0x3b,
            0x3a, 0x3b, 0x2d, 0x3b, 0x3a, 0x3b,
        ];
        const FLAGS: [u8; 48] = [
            0xa4, 0xa4, 0x24, 0x24, 0x64, 0x64, 0x24, 0x24, 0xa4, 0xff, 0x24, 0xff, 0x64, 0xff,
            0x24, 0xff, 0xa4, 0xa4, 0xa4, 0xa4, 0xa4, 0xe4, 0xa4, 0xa4, 0x24, 0x24, 0x24, 0x24,
            0x64, 0x24, 0x24, 0x24, 0x64, 0x64, 0x64, 0xe4, 0x64, 0xe4, 0x64, 0xe4, 0x24, 0x24,
            0x24, 0xa4, 0xa4, 0x24, 0x24, 0xa4,
        ];
        const Y: [i8; 48] = [
            0, 8, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0,
            8, 0, 8, -1, -1, 0, 0, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 0, 0,
        ];
        const X: [i8; 48] = [
            0, 0, 0, 0, 0, 8, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, -2, 0, 0, 1, 1, 0, 0,
            -2, -1, 0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8,
        ];
        let (mut x, mut y) = self.prep_adjusted_oam_coord(k);
        if self.object_priority(k) != 0 {
            self.ram.write_u16(0x64, 0x3000);
        }
        if self.state_h(k) != 0 {
            x = x.wrapping_add(
                self.ram
                    .read_u16(0xe8)
                    .wrapping_sub(self.ram.read_u16(0xe6)),
            );
            y = y.wrapping_add(
                self.ram
                    .read_u16(0xe2)
                    .wrapping_sub(self.ram.read_u16(0xe0)),
            );
        }
        let animation = self.item_to_link(k);
        let mut frame = usize::from(self.direction(k) & !4);
        if self.kind(k) == 0x0a {
            frame = frame * 4 + 8 + usize::from(if animation & 8 != 0 { 1 } else { animation & 3 });
        } else if (animation as i8) >= 0 {
            frame |= 4;
        }
        frame *= 2;
        let original = self.current_oam_index();
        let mut index = original;
        let palette = if self.read(0xf340) & 4 != 0 { 2 } else { 4 };
        for i in 0..2 {
            let t = frame + i;
            if CHAR[t] != 0xff {
                self.set_oam(
                    index,
                    x.wrapping_add(i16::from(X[t]) as u16),
                    y.wrapping_add(i16::from(Y[t]) as u16),
                    CHAR[t],
                    (FLAGS[t] & !0x3e) | palette | (self.ram.read_u16(0x64) >> 8) as u8,
                    0,
                );
                index += 1;
            }
        }
        if oam::read(self.ram, original).y == 0xf0 && oam::read(self.ram, original + 1).y == 0xf0 {
            self.clear(k);
        }
    }

    pub fn arrow(&mut self, k: usize) {
        const Y: [i8; 4] = [-4, 2, 0, 0];
        const X: [i8; 4] = [0, 0, -4, 4];
        if self.read(0x11) != 0 {
            self.arrow_draw(k);
            return;
        }
        let animation = self.item_to_link(k).wrapping_sub(1);
        self.set_item_to_link(k, animation);
        if (animation as i8) >= 0 {
            if animation >= 4 {
                return;
            }
        } else {
            self.set_item_to_link(k, 0xff);
        }
        self.move_y(k);
        self.move_x(k);
        if self.read(0xf340) & 4 != 0 && self.read(0x1a) & 1 == 0 {
            self.add_silver_arrow_sparkle(k);
        }
        self.set_state_s(k, 0xff);
        let mut j;
        if let Some(sprite) = self.check_sprite_collision(k) {
            self.set_x_velocity(k, self.x_low(k).wrapping_sub(self.read(0xd10 + sprite)));
            self.set_y_velocity(
                k,
                self.y_low(k)
                    .wrapping_sub(self.read(0xd00 + sprite))
                    .wrapping_add(self.read(0xf70 + sprite)),
            );
            self.set_state_s(k, sprite as u8);
            if self.read(0xe20 + sprite) == 0x65 {
                if self.read(0xd90 + sprite) == 1 {
                    self.write(0x12f, 0x2d);
                    self.write(0xe10 + sprite, 0x80);
                    self.write(0xf10, 128);
                    if self.read(0xb88) < 9 {
                        self.write(0xb88, self.read(0xb88).wrapping_add(1));
                    }
                    self.write(0xda0 + sprite, self.read(0xb88));
                    self.write(0xed0 + sprite, self.read(0xed0 + sprite).wrapping_add(1));
                } else {
                    self.write(0xee0 + sprite, 4);
                    self.write(0xb88, 0);
                }
            } else {
                self.write(0xb88, 0);
            }
            j = sprite;
        } else {
            let collision = self.check_tile_collision(k);
            if collision == 0 {
                self.arrow_draw(k);
                return;
            }
            self.set_state_h(k, collision >> 1);
            j = usize::from(self.direction(k) & 3);
            self.set_xy(
                k,
                self.get_x(k).wrapping_add(i16::from(X[j]) as u16),
                self.get_y(k).wrapping_add(i16::from(Y[j]) as u16),
            );
            self.write(0xb88, 0);
            j = usize::from(collision);
        }
        if self.read(0xe20 + j) != 0x1b {
            self.sfx2_pan(k, 8);
        }
        self.set_item_to_link(k, 0);
        self.set_kind(k, 10);
        self.set_aux_timer(k, 1);
        if self.state_h(k) != 0 {
            self.set_x_low(
                k,
                self.x_low(k)
                    .wrapping_add(self.read(0xe0).wrapping_sub(self.read(0xe2))),
            );
            self.set_y_low(
                k,
                self.y_low(k)
                    .wrapping_add(self.read(0xe6).wrapping_sub(self.read(0xe8))),
            );
        }
        self.arrow_draw(k);
    }

    pub fn arrow_in_wall(&mut self, k: usize) {
        let sprite = self.state_s(k);
        if (sprite as i8) >= 0 {
            let sprite = usize::from(sprite);
            if self.read(0xdd0 + sprite) < 9
                || (self.read(0xf70 + sprite) as i8) < 0
                || self.read(0xba0 + sprite) != 0
                || self.read(0xcaa + sprite) & 2 != 0
            {
                self.clear(k);
                return;
            }
            let sprite_x =
                u16::from_le_bytes([self.read(0xd10 + sprite), self.read(0xd30 + sprite)]);
            let sprite_y =
                u16::from_le_bytes([self.read(0xd00 + sprite), self.read(0xd20 + sprite)]);
            self.set_xy(
                k,
                sprite_x.wrapping_add(i16::from(self.x_velocity(k) as i8) as u16),
                sprite_y
                    .wrapping_add(i16::from(self.y_velocity(k) as i8) as u16)
                    .wrapping_sub(u16::from(self.read(0xf70 + sprite))),
            );
        }
        if self.read(0x11) == 0 {
            let countdown = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, countdown);
            if countdown == 0 {
                self.set_aux_timer(k, 2);
                let frame = self.item_to_link(k).wrapping_add(1);
                self.set_item_to_link(k, frame);
                if frame == 9 {
                    self.clear(k);
                    return;
                }
                if frame & 8 != 0 {
                    self.set_aux_timer(k, 0x80);
                }
            }
        }
        self.arrow_draw(k);
    }

    pub fn ice_rod_shot(&mut self, k: usize) {
        if self.read(0x11) == 0 {
            let countdown = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, countdown);
            if (countdown as i8) < 0 {
                let animation = self.item_to_link(k).wrapping_add(1);
                self.set_item_to_link(k, animation);
                if animation & !1 != 0 {
                    self.set_step(k, 1);
                    self.set_item_to_link(k, animation & 7 | 4);
                }
                self.set_aux_timer(k, 3);
            }
            if self.step(k) != 0 {
                if self.return_if_outside_bounds(k).is_none() {
                    return;
                }
                self.move_y(k);
                self.move_x(k);
                if self.check_sprite_collision(k).is_some() || self.check_tile_collision(k) != 0 {
                    self.set_kind(k, 0x11);
                    self.set_sprite_count(k, PFLAGS[0x11]);
                    self.set_item_to_link(k, 0);
                    self.set_aux_timer(k, 4);
                }
            }
        }
        self.add_ice_rod_sparkle(k);
    }

    pub fn ice_shot_spread_draw(&mut self, k: usize) {
        const CHAR_FLAGS: [u8; 16] = [
            0xcf, 0x24, 0xcf, 0x24, 0xcf, 0x24, 0xcf, 0x24, 0xdf, 0x24, 0xdf, 0x24, 0xdf, 0x24,
            0xdf, 0x24,
        ];
        const XY: [u8; 16] = [
            0, 0, 0, 8, 8, 0, 8, 8, 0xf8, 0xf8, 0xf8, 0x10, 0x10, 0xf8, 0x10, 0x10,
        ];
        let (x0, y0) = self.prep_oam_coord(k);
        self.runtime.allocate_oam(self.ram, k, self.sprite_count(k));
        let original = self.current_oam_index();
        let mut index = original;
        let mut frame = usize::from(self.item_to_link(k)) * 4;
        for _ in 0..4 {
            let y = y0.wrapping_add(i16::from(XY[frame * 2] as i8) as u16);
            let x = x0.wrapping_add(i16::from(XY[frame * 2 + 1] as i8) as u16);
            let mut entry = oam::read(self.ram, index);
            let mut visible_y = 0xf0;
            if x < 256 && y < 256 {
                entry.x = x as u8;
                if y < 224 {
                    visible_y = y as u8;
                }
            }
            entry.y = visible_y;
            entry.charnum = CHAR_FLAGS[frame * 2];
            entry.flags =
                (CHAR_FLAGS[frame * 2 + 1] & !0x30) | (self.ram.read_u16(0x64) >> 8) as u8;
            oam::write(self.ram, index, entry);
            self.write(oam::BYTEWISE_EXTENDED_OAM + index, 0);
            index = self.allocate_oam_from_custom_region(index + 1);
            frame += 1;
        }
        if oam::read(self.ram, original).y == 0xf0 && oam::read(self.ram, original + 1).y == 0xf0 {
            self.clear(k);
        }
    }

    pub fn ice_rod_wall_hit(&mut self, k: usize) {
        let countdown = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, countdown);
        if (countdown as i8) < 0 {
            self.set_aux_timer(k, 7);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 2 {
                self.clear(k);
                return;
            }
        }
        self.ice_shot_spread_draw(k);
    }

    fn check_link_collision(&self, k: usize, class: usize) -> Option<super::CheckPlayerCollOut> {
        const Y_OFFSET: [i16; 5] = [0, 8, 8, 8, 0];
        const X_OFFSET: [i16; 5] = [0, 8, 8, 8, 0];
        const HEIGHT: [u16; 5] = [20, 20, 8, 28, 14];
        const WIDTH: [u16; 5] = [20, 3, 8, 24, 14];
        const LINK_Y_OFFSET: [u16; 5] = [12, 12, 12, 12, 12];
        const LINK_X_OFFSET: [u16; 5] = [8, 8, 8, 12, 8];
        let y = self
            .get_y(k)
            .wrapping_add(Y_OFFSET[class] as u16)
            .wrapping_add(i16::from(self.height(k) as i8) as u16);
        let x = self.get_x(k).wrapping_add(X_OFFSET[class] as u16);
        let r4 = self
            .ram
            .read_u16(0x20)
            .wrapping_add(LINK_Y_OFFSET[class])
            .wrapping_sub(y);
        let r6 = self
            .ram
            .read_u16(0x22)
            .wrapping_add(LINK_X_OFFSET[class])
            .wrapping_sub(x);
        let r8 = r4.cast_signed().unsigned_abs();
        let r10 = r6.cast_signed().unsigned_abs();
        (r8 < HEIGHT[class] && r10 < WIDTH[class]).then_some(super::CheckPlayerCollOut {
            r4,
            r6,
            r8,
            r10,
        })
    }

    pub fn check_link_collision_class(&self, k: usize, class: usize) -> bool {
        self.check_link_collision(k, class).is_some()
    }

    fn transmute_to_splash(&mut self, k: usize) {
        self.set_kind(k, 0x3d);
        self.set_item_to_link(k, 0);
        self.set_timer(k, 6);
        self.set_xy(
            k,
            self.get_x(k).wrapping_sub(8),
            self.get_y(k).wrapping_add(12),
        );
        self.sfx2_pan(k, 0x28);
        self.item_splash(k);
    }

    fn object_splash_draw(&mut self, k: usize) {
        const X: [i8; 10] = [0, 0, 0, 0, 11, -3, 15, -7, 15, -7];
        const Y: [i8; 10] = [0, 0, -6, 0, -13, -8, -17, -4, -17, -4];
        const CHAR: [u8; 10] = [0xc0, 0xff, 0xe7, 0xff, 0xaf, 0xbf, 0x80, 0x80, 0x83, 0x83];
        const FLAGS: [u8; 10] = [0, 0xff, 0, 0xff, 0x40, 0, 0x40, 0, 0xc0, 0x80];
        const EXT: [u8; 10] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0];
        let (x, y) = self.prep_oam_coord(k);
        let mut frame = usize::from(self.item_to_link(k)) * 2;
        let mut index = self.current_oam_index();
        for _ in 0..2 {
            if CHAR[frame] != 0xff {
                self.set_oam(
                    index,
                    x.wrapping_add(i16::from(X[frame]) as u16),
                    y.wrapping_add(i16::from(Y[frame]) as u16),
                    CHAR[frame],
                    FLAGS[frame] | 0x24,
                    EXT[frame],
                );
                index += 1;
            }
            frame += 1;
        }
    }

    pub fn item_splash(&mut self, k: usize) {
        self.allocate_oam_from_region_a_d_f(k, 8);
        if self.read(0x11) == 0 && self.timer(k) == 0 {
            self.set_timer(k, 6);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 5 {
                self.clear(k);
                return;
            }
        }
        self.object_splash_draw(k);
    }

    fn receive_item_draw(&mut self, k: usize, x: u16, y: u16) -> usize {
        let item = usize::from(self.item_to_link(k));
        let mut flags = RECEIVE_ITEM_FLAGS[item];
        if (flags as i8) < 0 {
            flags = self.arr4(k);
        }
        let mut index = self.current_oam_index();
        self.set_oam(
            index,
            x,
            y,
            0x24,
            flags.wrapping_mul(2) | 0x30,
            RECEIVE_ITEM_SIZE[item],
        );
        index += 1;
        if RECEIVE_ITEM_SIZE[item] == 0 {
            self.set_oam(
                index,
                x,
                y.wrapping_add(8),
                0x34,
                flags.wrapping_mul(2) | 0x30,
                0,
            );
            index += 1;
        }
        index
    }

    fn wish_pond_item_draw(&mut self, k: usize) {
        let (x, y) = self.prep_adjusted_oam_coord(k);
        if self.item_to_link(k) == 1 {
            self.set_arr4(k, 5);
        }
        let index =
            self.receive_item_draw(k, x, y.wrapping_sub(i16::from(self.height(k) as i8) as u16));
        let z_velocity = self.z_velocity(k);
        if self.read(0x309) != 2 || (z_velocity as i8) >= 0 && z_velocity >= 2 {
            return;
        }
        let kind = self.runtime.generated_wish_pond_item(self.item_to_link(k));
        self.draw_shadow(
            index,
            if kind == 2 { 1 } else { 2 },
            x.wrapping_sub(if kind == 2 { 0 } else { 4 }),
            y.wrapping_add(40),
            (self.ram.read_u16(0x64) >> 8) as u8,
        );
    }

    pub fn wish_pond_item(&mut self, k: usize) {
        self.allocate_oam_from_region_a_d_f(k, 0x10);
        if self.read(0x11) == 0 && self.timer(k) == 0 {
            self.write(0x309, 2);
            self.write(0x308, 0);
            self.set_z_velocity(k, self.z_velocity(k).wrapping_sub(2));
            self.move_z(k);
            self.move_y(k);
            self.move_x(k);
            let z = self.height(k);
            if (z as i8) < 0 && z < 228 {
                self.set_height(k, 228);
                let generated = self.runtime.generated_wish_pond_item(self.item_to_link(k));
                self.set_xy(
                    k,
                    self.get_x(k)
                        .wrapping_add(if generated != 0 { 8 } else { 4 }),
                    self.get_y(k).wrapping_add(18),
                );
                self.transmute_to_splash(k);
                return;
            }
        }
        self.wish_pond_item_draw(k);
    }

    fn happiness_pond_get_state(&mut self, ancilla: usize, pond: usize) {
        const SOURCE: [usize; 14] = [
            0x15824, 0x15830, 0x1583c, 0x15848, 0x15854, 0x15800, 0x1580c, 0x15818, 0x15886,
            0x15892, 0x1589e, 0x1587a, 0x158aa, 0x15860,
        ];
        const DESTINATION: [usize; 14] = [
            0xbfa, 0xc0e, 0xc04, 0xc18, 0x29e, 0xc22, 0xc2c, 0x294, 0xc36, 0xc40, 0x2a8, 0xc5e,
            0xc54, 0xc68,
        ];
        for (source, destination) in SOURCE.into_iter().zip(DESTINATION) {
            let mut value = self.read(source + pond);
            if source == 0x15860 && value != 0 {
                value -= 1;
            }
            self.write(destination + ancilla, value);
        }
    }

    fn happiness_pond_save_state(&mut self, pond: usize, ancilla: usize) {
        const DESTINATION: [usize; 14] = [
            0x15824, 0x15830, 0x1583c, 0x15848, 0x15854, 0x15800, 0x1580c, 0x15818, 0x15886,
            0x15892, 0x1589e, 0x1587a, 0x15860, 0x158aa,
        ];
        const SOURCE: [usize; 14] = [
            0xbfa, 0xc0e, 0xc04, 0xc18, 0x29e, 0xc22, 0xc2c, 0x294, 0xc36, 0xc40, 0x2a8, 0xc5e,
            0xc68, 0xc54,
        ];
        for (destination, source) in DESTINATION.into_iter().zip(SOURCE) {
            self.write(destination + pond, self.read(source + ancilla));
        }
    }

    fn happiness_pond_execute_rupee(&mut self, k: usize, pond: usize) {
        self.allocate_oam_from_region_a_d_f(k, 0x10);
        self.happiness_pond_get_state(k, pond);
        if self.step(k) != 0 {
            if self.read(0x11) == 0 && self.timer(k) == 0 {
                self.set_timer(k, 6);
                let frame = self.item_to_link(k).wrapping_add(1);
                self.set_item_to_link(k, frame);
                if frame == 5 {
                    self.set_step(k, self.step(k).wrapping_add(1));
                } else {
                    self.object_splash_draw(k);
                }
            } else {
                self.object_splash_draw(k);
            }
        } else if self.read(0x11) == 0 && self.timer(k) == 0 {
            self.set_z_velocity(k, self.z_velocity(k).wrapping_sub(2));
            self.move_y(k);
            self.move_x(k);
            self.move_z(k);
            let z = self.height(k);
            if (z as i8) < 0 && z < 0xe4 {
                self.set_height(k, 0xe4);
                self.set_xy(
                    k,
                    self.get_x(k).wrapping_sub(4),
                    self.get_y(k).wrapping_add(30),
                );
                self.set_item_to_link(k, 0);
                self.set_timer(k, 6);
                self.sfx2_pan(k, 0x28);
                self.set_step(k, self.step(k).wrapping_add(1));
                self.object_splash_draw(k);
            } else {
                self.set_arr4(k, 2);
                self.set_floor(k, 0);
                self.wish_pond_item_draw(k);
            }
        } else {
            self.set_arr4(k, 2);
            self.set_floor(k, 0);
            self.wish_pond_item_draw(k);
        }
        self.happiness_pond_save_state(pond, k);
    }

    pub fn happiness_pond_rupees(&mut self, k: usize) {
        self.write(0x309, 2);
        self.write(0x308, 0);
        for pond in (0..10).rev() {
            if self.read(0x1586c + pond) != 0 {
                self.happiness_pond_execute_rupee(k, pond);
                if self.read(0x158aa + pond) == 2 {
                    self.write(0x1586c + pond, 0);
                }
            }
        }
        if (0..10).all(|pond| self.read(0x1586c + pond) == 0) {
            self.clear(k);
        }
    }

    fn morph_poof_draw(&mut self, k: usize) {
        const X: [i8; 12] = [0, 0, 0, 0, 0, 8, 0, 8, -4, 12, -4, 12];
        const Y: [i8; 12] = [0, 0, 0, 0, 0, 0, 8, 8, -4, -4, 12, 12];
        const FLAGS: [u8; 12] = [
            0, 0xff, 0xff, 0xff, 0x40, 0, 0xc0, 0x80, 0, 0x40, 0x80, 0xc0,
        ];
        const CHAR: [u8; 3] = [0x86, 0xa9, 0x9b];
        const EXT: [u8; 3] = [2, 0, 0];
        if self.read(0xfb3) != 0
            && self.floor(k) != 0
            && (self.read(0x35f) == 0 || self.read(0x1a) & 1 != 0)
        {
            oam::OamPointers::for_cursor(oam::OamCursor::from_byte_offset(0xd0))
                .publish_entry_first(self.ram);
        }
        let (x, y) = self.prep_oam_coord(k);
        let frame = usize::from(self.item_to_link(k));
        let ext = EXT[frame];
        let mut index = self.current_oam_index();
        for i in 0..4 {
            let j = frame * 4 + i;
            self.set_oam(
                index,
                x.wrapping_add(i16::from(X[j]) as u16),
                y.wrapping_add(i16::from(Y[j]) as u16),
                CHAR[frame],
                FLAGS[j] | 4 | (self.ram.read_u16(0x64) >> 8) as u8,
                ext,
            );
            index += 1;
            if ext == 2 {
                break;
            }
        }
    }

    pub fn link_poof(&mut self, k: usize) {
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) < 0 {
            self.set_aux_timer(k, 7);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 3 {
                self.clear(k);
                self.write(0x2e1, 0);
                self.write(0x50, 0);
                if self.step(k) == 0 {
                    self.write(0x2e, 0);
                    self.write(0x4b, 0);
                    let bunny = u8::from(self.ram.read_u16(0x8a) as u8 & 0x40 != 0);
                    self.write(0x56, bunny);
                    self.write(0x2e0, bunny);
                    if bunny != 0 {
                        self.runtime.load_gear_palettes_bunny(self.ram);
                    } else {
                        self.runtime.load_actual_gear_palettes(self.ram);
                    }
                }
                return;
            }
        }
        self.morph_poof_draw(k);
    }

    pub fn dwarf_poof(&mut self, k: usize) {
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) < 0 {
            self.set_aux_timer(k, 7);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 3 {
                self.clear(k);
                self.write(0x2f9, 0);
                return;
            }
        }
        self.morph_poof_draw(k);
    }

    pub fn bush_poof(&mut self, k: usize) {
        const X: [i8; 16] = [0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -2, 10, -2, 10];
        const Y: [i8; 16] = [0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -2, -2, 10, 10];
        const CHAR: [u8; 16] = [
            0x86, 0x87, 0x96, 0x97, 0xa9, 0xa9, 0xa9, 0xa9, 0x8a, 0x8b, 0x9a, 0x9b, 0x9b, 0x9b,
            0x9b, 0x9b,
        ];
        const FLAGS: [u8; 16] = [
            0, 0, 0, 0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0, 0xc0, 0x80, 0x40, 0,
        ];
        if self.timer(k) == 0 {
            self.set_timer(k, 7);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 4 {
                self.clear(k);
                return;
            }
        }
        self.runtime.allocate_oam_region(self.ram, b'C', 0x10);
        let (x, y) = self.prep_oam_coord(k);
        let mut index = self.current_oam_index();
        let mut j = usize::from(self.item_to_link(k)) * 4;
        for _ in 0..4 {
            self.set_oam(
                index,
                x.wrapping_add(i16::from(X[j]) as u16),
                y.wrapping_add(i16::from(Y[j]) as u16),
                CHAR[j],
                FLAGS[j] | 4 | (self.ram.read_u16(0x64) >> 8) as u8,
                0,
            );
            index += 1;
            j += 1;
        }
    }

    pub fn sword_swing_sparkle(&mut self, k: usize) {
        const X: [i8; 48] = [
            5, 10, -1, 5, 10, -4, 5, 10, -4, -4, -1, -1, 0, 5, -1, 0, 5, 14, 0, 5, 14, 14, -1, -1,
            -23, -27, -1, -23, -27, -22, -23, -27, -22, -22, -1, -1, 32, 35, -1, 32, 35, 30, 32,
            35, 30, 30, -1, -1,
        ];
        const Y: [i8; 48] = [
            -22, -18, -1, -22, -18, -17, -22, -18, -17, -17, -1, -1, 35, 40, -1, 35, 40, 37, 35,
            40, 37, 37, -1, -1, 2, 7, -1, 2, 7, 19, 2, 7, 19, 19, -1, -1, 2, 7, -1, 2, 7, 19, 2, 7,
            19, 19, -1, -1,
        ];
        const CHAR: [u8; 48] = [
            0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7,
            0xff, 0x80, 0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80,
            0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7,
            0x83, 0x83, 0x80, 0x83, 0xff, 0xff,
        ];
        const FLAGS: [u8; 48] = [
            0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80, 0x80,
            0, 0x80, 0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff, 0, 0, 0xff,
            0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff,
        ];
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) < 0 {
            self.set_aux_timer(k, 0);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 4 {
                self.clear(k);
                return;
            }
        }
        self.set_xy(k, self.ram.read_u16(0x22), self.ram.read_u16(0x20));
        let (x, y) = self.prep_oam_coord(k);
        let mut table_index =
            usize::from(self.item_to_link(k)) * 3 + usize::from(self.direction(k)) * 12;
        let mut oam_index = self.current_oam_index();
        for _ in 0..3 {
            if CHAR[table_index] != 0xff {
                self.set_oam(
                    oam_index,
                    x.wrapping_add(i16::from(X[table_index]) as u16),
                    y.wrapping_add(i16::from(Y[table_index]) as u16),
                    CHAR[table_index],
                    FLAGS[table_index] | 4 | (self.ram.read_u16(0x64) >> 8) as u8,
                    0,
                );
            }
            table_index += 1;
            oam_index += 1;
        }
    }

    pub fn spin_attack_full_charge_spark(&mut self, k: usize) {
        const Y: [i8; 4] = [-8, 27, 12, 12];
        const X: [i8; 4] = [4, 4, -13, 20];
        const PRIORITY: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
        self.allocate_oam_from_region_a_d_f(k, 4);
        self.set_oam_index(k, oam::OamPointers::legacy_primary_low_byte(self.ram));
        if self.timer(k) == 0 {
            self.clear(k);
            return;
        }
        let direction = usize::from(self.read(0x2f) >> 1);
        let x = self
            .ram
            .read_u16(0x22)
            .wrapping_add(i16::from(X[direction]) as u16)
            .wrapping_sub(self.ram.read_u16(0xe2));
        let y = self
            .ram
            .read_u16(0x20)
            .wrapping_add(i16::from(Y[direction]) as u16)
            .wrapping_sub(self.ram.read_u16(0xe8));
        let priority = PRIORITY[usize::from(self.floor(k))];
        self.ram.write_u16(0x64, u16::from(priority) << 8);
        self.set_oam(self.current_oam_index(), x, y, 0xd7, priority | 2, 0);
    }

    pub fn flute(&mut self, k: usize) {
        const Z_VELOCITY: [u8; 4] = [0x18, 0x10, 0x0a, 0];
        if self.read(0x11) == 0 {
            if self.step(k) != 3 {
                self.set_z_velocity(k, self.z_velocity(k).wrapping_sub(2));
                self.move_x(k);
                self.move_z(k);
                let z = self.height(k);
                if (z as i8) < 0 || z >= 0xf0 {
                    let step = self.step(k).wrapping_add(1);
                    self.set_step(k, step);
                    self.set_z_velocity(k, Z_VELOCITY[usize::from(step)]);
                    self.set_height(k, 0);
                }
            } else if self.check_link_collision(k, 2).is_some()
                && self.read(0x37e) == 0
                && self.read(0x4d) == 0
            {
                self.clear(k);
                self.write(0x2e9, 0);
                self.runtime.link_receive_item(self.ram, 0x14);
                return;
            }
        }
        let (x, y) = self.prep_adjusted_oam_coord(k);
        let index = self.current_oam_index();
        self.set_oam(
            index,
            x,
            y.wrapping_sub(i16::from(self.height(k) as i8) as u16),
            0x24,
            (self.ram.read_u16(0x64) >> 8) as u8 | 4,
            2,
        );
        if oam::read(self.ram, index).y == 0xf0 {
            self.clear(k);
        }
    }

    pub fn lamp_flame(&mut self, k: usize) {
        const CHAR: [u8; 12] = [
            0x9c, 0x9c, 0xff, 0xff, 0xa4, 0xa5, 0xb2, 0xb3, 0xe3, 0xf3, 0xff, 0xff,
        ];
        const Y: [i8; 12] = [-3, 0, 0, 0, 0, 0, 8, 8, 0, 8, 0, 0];
        const X: [i8; 12] = [4, 10, 0, 0, 1, 9, 2, 7, 4, 4, 0, 0];
        let (x, y) = self.prep_adjusted_oam_coord(k);
        let timer = self.timer(k);
        if timer == 0 {
            self.clear(k);
            return;
        }
        let mut j = usize::from((timer & 0xf8) >> 1);
        let mut index = self.current_oam_index();
        loop {
            if CHAR[j] != 0xff {
                self.set_oam(
                    index,
                    x.wrapping_add(i16::from(X[j]) as u16),
                    y.wrapping_add(i16::from(Y[j]) as u16),
                    CHAR[j],
                    (self.ram.read_u16(0x64) >> 8) as u8 | 2,
                    0,
                );
                index += 1;
            }
            j += 1;
            if j & 3 == 0 {
                break;
            }
        }
    }

    pub fn waterfall_splash(&mut self, k: usize) {
        const X: [i8; 8] = [0, 0, -4, 4, -7, 7, -9, 17];
        const Y: [i8; 8] = [-4, 0, -5, -5, -3, -3, 12, 12];
        const CHAR: [u8; 8] = [0xc0, 0xff, 0xac, 0xac, 0xae, 0xae, 0xbf, 0xbf];
        const FLAGS: [u8; 8] = [0x84, 0xff, 0x84, 0xc4, 0x84, 0xc4, 0x84, 0xc4];
        const EXT: [u8; 8] = [2, 0xff, 2, 2, 2, 2, 0, 0];
        let indoors = self.read(0x1b) != 0;
        if !self.check_for_entrance_trigger(if indoors { 0 } else { 1 }) {
            self.clear(k);
            return;
        }
        if self.read(0x11) == 0 && self.read(0x1a) & 7 == 0 {
            misc::ancilla_sfx2_near(self.ram, 0x1c);
        }
        self.write(0x351, 1);
        let animation = self.read(0x2e);
        if (animation.wrapping_sub(6) as i8) >= 0 {
            self.write(0x2e, animation - 6);
        }
        if self.timer(k) == 0 {
            self.set_timer(k, 2);
            self.set_item_to_link(k, self.item_to_link(k).wrapping_add(1) & 3);
        }
        let link_y = self.ram.read_u16(0x20);
        self.set_y(
            k,
            if indoors && (link_y as u8) < 0x38 {
                0xd38
            } else {
                link_y
            },
        );
        self.set_x(k, self.ram.read_u16(0x22));
        let (x, mut y) = self.prep_adjusted_oam_coord(k);
        let link_z = self.read(0x24);
        if (link_z as i8) >= 0 {
            y = y.wrapping_sub(u16::from(link_z));
        }
        let mut j = usize::from(self.item_to_link(k)) * 2;
        let mut index = self.current_oam_index();
        for _ in 0..2 {
            if CHAR[j] != 0xff {
                self.set_oam(
                    index,
                    x.wrapping_add(i16::from(X[j]) as u16),
                    y.wrapping_add(i16::from(Y[j]) as u16),
                    CHAR[j],
                    FLAGS[j] | 0x30,
                    EXT[j],
                );
            }
            index += 1;
            j += 1;
        }
    }

    pub fn gravestone(&mut self, k: usize) {
        const CHAR: [u8; 4] = [0xc8, 0xc8, 0xd8, 0xd8];
        const FLAGS: [u8; 4] = [0, 0x40, 0, 0x40];
        let (mut x, mut y) = self.prep_adjusted_oam_coord(k);
        self.runtime.allocate_oam_region(self.ram, b'B', 16);
        let mut index = self.current_oam_index();
        for i in 0..4 {
            self.set_oam(index, x, y, CHAR[i], FLAGS[i] | 0x3d, 2);
            index += 1;
            x = x.wrapping_add(16);
            if i == 1 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(8);
            }
        }
    }

    pub fn somaria_block_fizz(&mut self, k: usize) {
        const X: [i8; 6] = [-4, -1, -8, 0, -6, -2];
        const Y: [i8; 6] = [-4, -1, -4, -4, -4, -4];
        const CHAR: [u8; 6] = [0x92, 0xff, 0xf9, 0xf9, 0xf9, 0xf9];
        const FLAGS: [u8; 6] = [6, 0xff, 0x86, 0xc6, 0x86, 0xc6];
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) < 0 {
            self.set_aux_timer(k, 3);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 3 {
                self.clear(k);
                return;
            }
        }
        let (x, y) = self.prep_adjusted_oam_coord(k);
        let mut z = self.height(k);
        if z == 0xff {
            z = 0;
        }
        let y = y.wrapping_sub(i16::from(z as i8) as u16);
        let mut j = usize::from(self.item_to_link(k)) * 2;
        let mut index = self.current_oam_index();
        for _ in 0..2 {
            if CHAR[j] != 0xff {
                self.set_oam(
                    index,
                    x.wrapping_add(i16::from(X[j]) as u16),
                    y.wrapping_add(i16::from(Y[j]) as u16),
                    CHAR[j],
                    FLAGS[j] & !0x30 | (self.ram.read_u16(0x64) >> 8) as u8,
                    0,
                );
            }
            index += 1;
            j += 1;
        }
    }

    pub fn somaria_block_fission(&mut self, k: usize) {
        const X: [i8; 16] = [-8, 0, -8, 0, -10, -10, 2, 2, -8, 0, -8, 0, -12, -12, 4, 4];
        const Y: [i8; 16] = [-10, -10, 2, 2, -8, 0, -8, 0, -12, -12, 4, 4, -8, 0, -8, 0];
        const CHAR: [u8; 16] = [
            0xc6, 0xc6, 0xc6, 0xc6, 0xc4, 0xc4, 0xc4, 0xc4, 0xd2, 0xd2, 0xd2, 0xd2, 0xc5, 0xc5,
            0xc5, 0xc5,
        ];
        const FLAGS: [u8; 16] = [
            0xc6, 0x86, 0x46, 6, 0x46, 0xc6, 6, 0x86, 0xc6, 0x86, 0x46, 6, 0x46, 0xc6, 6, 0x86,
        ];
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) < 0 {
            self.set_aux_timer(k, 3);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 2 {
                self.clear(k);
                self.somaria_block_spawn_bullets(k);
                return;
            }
        }
        let (x, y) = self.prep_adjusted_oam_coord(k);
        let mut z = self.height(k);
        if self.state_k(k) == 3 && self.read(0x24) != 0xff {
            z = z.wrapping_add(self.read(0x24));
        }
        let mut j = usize::from(self.item_to_link(k)) * 8;
        let mut index = self.current_oam_index();
        for _ in 0..8 {
            self.set_oam(
                index,
                x.wrapping_add(i16::from(X[j]) as u16),
                y.wrapping_add(i16::from(Y[j]) as u16)
                    .wrapping_sub(i16::from(z as i8) as u16),
                CHAR[j],
                FLAGS[j] & !0x30 | (self.ram.read_u16(0x64) >> 8) as u8,
                0,
            );
            index += 1;
            j += 1;
        }
    }

    fn sparkle_prep_oam_from_radial(&self, radial: super::AncillaRadialProjection) -> (u16, u16) {
        let y_offset = if radial.r2 != 0 {
            -i16::from(radial.r0)
        } else {
            i16::from(radial.r0)
        };
        let x_offset = if radial.r6 != 0 {
            -i16::from(radial.r4)
        } else {
            i16::from(radial.r4)
        };
        (
            self.ram
                .read_u16(0x1580e)
                .wrapping_add(x_offset as u16)
                .wrapping_sub(4)
                .wrapping_sub(self.ram.read_u16(0xe2)),
            self.ram
                .read_u16(0x15810)
                .wrapping_add(y_offset as u16)
                .wrapping_sub(4)
                .wrapping_sub(self.ram.read_u16(0xe8)),
        )
    }

    pub fn sword_beam(&mut self, k: usize) {
        const CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];
        const TRAIL_CHAR: [u8; 3] = [0xb7, 0x80, 0x83];
        let mut flags = 2;
        if self.read(0x11) == 0 {
            self.set_xy(k, self.ram.read_u16(0x1580e), self.ram.read_u16(0x15810));
            self.move_x(k);
            self.move_y(k);
            self.ram.write_u16(0x1580e, self.get_x(k));
            self.ram.write_u16(0x15810, self.get_y(k));
            let cycle = self.state_g(k);
            self.set_state_g(k, cycle.wrapping_add(1));
            if cycle & 0x0f == 0 {
                let sound = SoundRequest::at_world_x(self.ram, 1, self.get_x(k));
                SoundPort::EffectsB.overwrite(self.ram, sound);
            }
            if self.check_sprite_collision(k).is_some() || self.check_tile_collision(k) != 0 {
                const Y_OFFSET: [i8; 4] = [0, 0, -6, -6];
                const X_OFFSET: [i8; 4] = [-8, -10, 0, 0];
                let direction = usize::from(self.direction(k));
                self.set_xy(
                    k,
                    self.get_x(k)
                        .wrapping_add(i16::from(X_OFFSET[direction]) as u16),
                    self.get_y(k)
                        .wrapping_add(i16::from(Y_OFFSET[direction]) as u16),
                );
                self.set_kind(k, 4);
                self.set_timer(k, 7);
                self.set_sprite_count(k, 0x10);
                return;
            }
            let timer = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, timer);
            if (timer as i8) < 0 {
                flags = 4;
                self.set_aux_timer(k, 2);
            }
        }
        let original = self.current_oam_index();
        let spin = self.state_s(k);
        let priority = (self.ram.read_u16(0x64) >> 8) as u8;
        for i in (0..4).rev() {
            if self.read(0x11) == 0 {
                self.write(
                    0x15800 + i,
                    self.read(0x15800 + i).wrapping_add(spin) & 0x3f,
                );
            }
            let radial = Self::radial_projection(self.read(0x15800 + i), self.read(0x15808));
            let (x, y) = self.sparkle_prep_oam_from_radial(radial);
            self.set_oam(original + (3 - i), x, y, CHAR[i], flags | priority, 0);
        }
        let mut draw_trail = true;
        if self.read(0x11) == 0 {
            let timer = self.arr3(k).wrapping_sub(1);
            self.set_arr3(k, timer);
            if (timer as i8) < 0 {
                self.set_arr3(k, 0);
                let trail = self.arr1(k).wrapping_add(1) & 3;
                self.set_arr1(k, trail);
                if trail == 3 {
                    self.write(0x15804, self.read(0x15804).wrapping_add(spin) & 0x3f);
                }
            } else {
                draw_trail = false;
            }
        }
        let trail = self.arr1(k);
        if draw_trail && trail != 3 {
            let radial = Self::radial_projection(self.read(0x15804), self.read(0x15808));
            let (x, y) = self.sparkle_prep_oam_from_radial(radial);
            self.set_oam(
                original + 4,
                x,
                y,
                TRAIL_CHAR[usize::from(trail)],
                4 | priority,
                0,
            );
        }
        if (0..4).all(|i| oam::read(self.ram, original + i).y == 0xf0) {
            self.clear(k);
        }
    }

    fn add_sword_beam(&mut self, limit: u8) {
        const X: [i8; 4] = [-8, -10, -22, 4];
        const Y: [i8; 4] = [-24, 8, -6, -6];
        const SPIN: [i8; 4] = [-8, -8, -8, 8];
        const ANGLES: [u8; 16] = [
            0x21, 0x1d, 0x19, 0x15, 3, 0x3e, 0x3a, 0x36, 0x12, 0x0e, 0x0a, 6, 0x31, 0x2d, 0x29,
            0x25,
        ];
        const Y_VELOCITY: [i8; 4] = [-64, 64, 0, 0];
        const X_VELOCITY: [i8; 4] = [0, 0, -64, 64];
        let Some(k) = self.add_ancilla(0x0c, limit) else {
            return;
        };
        let facing = usize::from(self.read(0x2f));
        for i in 0..4 {
            self.write(0x15800 + i, ANGLES[facing * 2 + i]);
        }
        self.write(0x15804, ANGLES[facing * 2 + 3]);
        self.set_aux_timer(k, 2);
        self.set_item_to_link(k, 0x4c);
        self.set_arr3(k, 8);
        self.set_step(k, 0);
        self.set_state_l(k, 0);
        self.set_state_g(k, 0);
        self.set_arr1(k, 0);
        self.write(0x15808, 14);
        let direction = facing >> 1;
        self.set_direction(k, direction as u8);
        self.set_y_velocity(k, Y_VELOCITY[direction] as u8);
        self.set_x_velocity(k, X_VELOCITY[direction] as u8);
        self.set_state_s(k, SPIN[direction] as u8);
        self.ram
            .write_u16(0x15810, self.ram.read_u16(0x20).wrapping_add(12));
        self.ram
            .write_u16(0x1580e, self.ram.read_u16(0x22).wrapping_add(8));
        if self.check_initial_tile_a(k) >= 0 {
            self.set_xy(
                k,
                self.ram
                    .read_u16(0x1580e)
                    .wrapping_add(i16::from(X[direction]) as u16),
                self.ram
                    .read_u16(0x15810)
                    .wrapping_add(i16::from(Y[direction]) as u16),
            );
            let sound = SoundRequest::at_world_x(self.ram, 1, self.get_x(k));
            SoundPort::EffectsB.overwrite(self.ram, sound);
            self.set_kind(k, 4);
            self.set_timer(k, 7);
            self.set_sprite_count(k, 16);
        }
    }

    pub fn spawn_sword_beam(&mut self, limit: u8) {
        self.add_sword_beam(limit);
    }

    fn spin_attack_sparkle_b_closer(&mut self, k: usize) {
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) < 0 {
            self.set_aux_timer(k, 1);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 3 {
                self.clear(k);
            }
        }
        self.spin_spark_draw(k, 4);
    }

    fn spin_attack_sparkle_a_transmute(&mut self, k: usize) {
        const ANGLES: [u8; 16] = [
            0x21, 0x20, 0x1f, 0x1e, 3, 2, 1, 0, 0x12, 0x11, 0x10, 0x0f, 0x31, 0x30, 0x2f, 0x2e,
        ];
        const X: [i8; 4] = [-3, 21, 25, -8];
        const Y: [i8; 4] = [28, -2, 24, 6];
        self.set_kind(k, 0x2b);
        let facing = usize::from(self.read(0x2f));
        for i in 0..4 {
            self.write(0x15800 + i, ANGLES[facing * 2 + i]);
        }
        self.write(0x15804, ANGLES[facing * 2 + 3]);
        self.set_aux_timer(k, 2);
        self.set_item_to_link(k, 0x4c);
        self.set_arr3(k, 8);
        self.set_step(k, 0);
        self.set_state_l(k, 0);
        self.set_arr1(k, 0xff);
        self.write(0x15808, 20);
        self.ram
            .write_u16(0x1580e, self.ram.read_u16(0x22).wrapping_add(8));
        self.ram
            .write_u16(0x15810, self.ram.read_u16(0x20).wrapping_add(12));
        let direction = facing >> 1;
        self.set_xy(
            k,
            self.ram
                .read_u16(0x22)
                .wrapping_add(i16::from(X[direction]) as u16),
            self.ram
                .read_u16(0x20)
                .wrapping_add(i16::from(Y[direction]) as u16),
        );
        self.spin_attack_sparkle_b(k);
    }

    pub fn spin_attack_sparkle_a(&mut self, k: usize) {
        const TIMER: [u8; 6] = [4, 2, 3, 3, 2, 1];
        if self.read(0x11) == 0 {
            let auxiliary = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, auxiliary);
            if (auxiliary as i8) < 0 {
                self.set_aux_timer(k, 0);
                if self.timer(k) == 0 {
                    let frame = self.item_to_link(k).wrapping_add(1);
                    self.set_item_to_link(k, frame);
                    self.set_timer(k, TIMER[usize::from(frame)]);
                    if frame == 5 {
                        if self.step(k) != 0 {
                            self.add_sword_beam(frame);
                        } else {
                            self.spin_attack_sparkle_a_transmute(k);
                        }
                        return;
                    }
                }
            }
        }
        if self.item_to_link(k) != 0 {
            self.spin_spark_draw(k, -1);
        }
    }

    pub fn spin_attack_sparkle_b(&mut self, k: usize) {
        const CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];
        const TRAIL_CHAR: [u8; 3] = [0xb7, 0x80, 0x83];
        if self.state_l(k) != 0 {
            self.spin_attack_sparkle_b_closer(k);
            return;
        }
        let mut flags = 2;
        if self.read(0x11) == 0 {
            let radius = self.item_to_link(k).wrapping_sub(3);
            self.set_item_to_link(k, radius);
            if radius < 13 {
                self.set_aux_timer(k, 1);
                self.set_state_l(k, 1);
                self.set_item_to_link(k, 0);
                self.spin_attack_sparkle_b_closer(k);
                return;
            }
            self.set_step(
                k,
                if radius < 0x42 {
                    3
                } else if radius == 0x46 {
                    1
                } else if radius == 0x43 {
                    2
                } else {
                    0
                },
            );
            let timer = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, timer);
            if (timer as i8) < 0 {
                flags = 4;
                self.set_aux_timer(k, 2);
            }
        }
        let original = self.current_oam_index();
        let step = usize::from(self.step(k));
        let priority = (self.ram.read_u16(0x64) >> 8) as u8;
        let mut index = original;
        for i in (0..=step).rev() {
            if self.read(0x11) == 0 {
                self.write(0x15800 + i, self.read(0x15800 + i).wrapping_add(4) & 0x3f);
            }
            let radial = Self::radial_projection(self.read(0x15800 + i), self.read(0x15808));
            let (x, y) = self.sparkle_prep_oam_from_radial(radial);
            self.set_oam(index, x, y, CHAR[i], flags | priority, 0);
            index += 1;
        }
        let mut draw_trail = true;
        if self.read(0x11) == 0 {
            let timer = self.arr3(k).wrapping_sub(1);
            self.set_arr3(k, timer);
            if (timer as i8) < 0 {
                self.set_arr3(k, 0);
                let trail = self.arr1(k).wrapping_add(1) & 3;
                self.set_arr1(k, trail);
                if trail == 3 {
                    self.write(0x15804, self.read(0x15804).wrapping_add(9) & 0x3f);
                }
            } else {
                draw_trail = false;
            }
        }
        let trail = self.arr1(k);
        if draw_trail && trail != 3 {
            let radial = Self::radial_projection(self.read(0x15804), self.read(0x15808));
            let (x, y) = self.sparkle_prep_oam_from_radial(radial);
            self.set_oam(index, x, y, TRAIL_CHAR[usize::from(trail)], 4 | priority, 0);
        }
        if self.item_to_link(k) == 7 {
            self.write(oam::BYTEWISE_EXTENDED_OAM + original + 3, 1);
        }
    }

    fn byrna_windup_transmute(&mut self, k: usize) {
        const ANGLES: [u8; 16] = [
            0x34, 0x33, 0x32, 0x31, 0x16, 0x15, 0x14, 0x13, 0x2a, 0x29, 0x28, 0x27, 0x10, 0x0f,
            0x0e, 0x0d,
        ];
        self.set_kind(k, 0x31);
        let facing = usize::from(self.read(0x2f));
        for i in 0..4 {
            self.write(0x15800 + i, ANGLES[facing * 2 + i]);
        }
        self.set_aux_timer(k, 0x17);
        self.set_state_g(k, 0);
        self.set_item_to_link(k, 0);
        self.set_arr3(k, 8);
        self.set_step(k, 0);
        self.set_state_l(k, 0);
        self.set_arr1(k, 2);
        self.set_timer(k, 21);
        self.write(0x15808, 20);
        misc::ancilla_sfx3_near(self.ram, 0x30);
        self.byrna_spark(k);
    }

    pub fn byrna_windup_spark(&mut self, k: usize) {
        const X: [i8; 16] = [3, 1, 0, 0, 13, 16, 12, 12, 24, 7, -4, -10, -8, 9, 22, 26];
        const Y: [i8; 16] = [5, 0, -3, -6, -8, -3, 12, 28, 5, 0, 8, 16, 5, 0, 8, 16];
        const DRAW_X: [i8; 16] = [-4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0];
        const DRAW_Y: [i8; 16] = [-4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0];
        const CHAR: [u8; 16] = [
            0x92, 0xff, 0xff, 0xff, 0x8c, 0x8c, 0x8c, 0x8c, 0xd6, 0xd6, 0xd6, 0xd6, 0x93, 0x93,
            0x93, 0x93,
        ];
        const FLAGS: [u8; 16] = [
            0x22, 0xff, 0xff, 0xff, 0x22, 0x62, 0xa2, 0xe2, 0x24, 0x64, 0xa4, 0xe4, 0x22, 0x62,
            0xa2, 0xe2,
        ];
        if self.read(0x11) == 0 {
            let timer = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, timer);
            if (timer as i8) < 0 {
                self.set_aux_timer(k, 1);
                let frame = self.item_to_link(k).wrapping_add(1);
                self.set_item_to_link(k, frame);
                if frame == 17 {
                    self.byrna_windup_transmute(k);
                    return;
                }
            }
        }
        if self.item_to_link(k) == 0 {
            return;
        }
        let mut pose = self.read(0x300);
        if pose == 2 {
            let value = self.arr3(k).wrapping_sub(1);
            if (value as i8) < 0 {
                self.set_arr3(k, 0);
                pose = 3;
            } else {
                self.set_arr3(k, value);
            }
        }
        let j = usize::from(pose) + usize::from(self.read(0x2f)) * 2;
        self.set_xy(
            k,
            self.ram.read_u16(0x22).wrapping_add(i16::from(X[j]) as u16),
            self.ram.read_u16(0x20).wrapping_add(i16::from(Y[j]) as u16),
        );
        let (x, y) = self.prep_oam_coord(k);
        let frame = self.item_to_link(k).wrapping_sub(1) & 0x0f;
        let mut table = if frame == 0 {
            0
        } else if frame == 15 {
            12
        } else {
            usize::from((frame & 1) + 1) * 4
        };
        let priority = (self.ram.read_u16(0x64) >> 8) as u8;
        let mut index = self.current_oam_index();
        for _ in 0..4 {
            if CHAR[table] != 0xff {
                self.set_oam(
                    index,
                    x.wrapping_add(i16::from(DRAW_X[table]) as u16),
                    y.wrapping_add(i16::from(DRAW_Y[table]) as u16),
                    CHAR[table],
                    FLAGS[table] & !0x30 | priority,
                    0,
                );
                index += 1;
            }
            table += 1;
        }
    }

    pub fn byrna_spark(&mut self, k: usize) {
        const MAGIC: [u8; 3] = [4, 2, 1];
        const CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];
        let mut flags = 2;
        if self.read(0x11) == 0 {
            if self.read(0x303) != 13 {
                self.write(0x37b, 0);
                self.clear(k);
                self.write(0x373, 0);
                return;
            }
            self.write(0x37b, 1);
            let timer = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, timer);
            if timer == 0 {
                self.set_aux_timer(k, 1);
                let cost = MAGIC[usize::from(self.read(0xf37b))];
                let power = self.read(0xf36e);
                let remaining = power.wrapping_sub(cost);
                if power == 0 || remaining >= 0x80 {
                    self.write(0x37b, 0);
                    self.clear(k);
                    self.write(0x373, 0);
                    return;
                }
                let magic_timer = self.state_g(k).wrapping_sub(1);
                self.set_state_g(k, magic_timer);
                if (magic_timer as i8) < 0 {
                    self.set_state_g(k, 0x17);
                    self.write(0xf36e, remaining);
                }
                if self.read(0xf4) & 0x40 != 0 {
                    self.write(0x37b, 0);
                    self.clear(k);
                    self.write(0x373, 0);
                    return;
                }
            }
            if self.step(k) != 3 {
                let frame = self.item_to_link(k).wrapping_add(1);
                self.set_item_to_link(k, frame);
                self.set_step(
                    k,
                    if frame >= 4 {
                        3
                    } else if frame == 2 {
                        1
                    } else if frame == 3 {
                        2
                    } else {
                        0
                    },
                );
            }
            let draw_timer = self.arr1(k).wrapping_sub(1);
            self.set_arr1(k, draw_timer);
            if (draw_timer as i8) < 0 {
                self.set_arr1(k, 2);
                flags = 4;
            }
        }
        let mut z = i16::from(self.read(0x24) as i8);
        if z == -1 {
            z = 0;
        }
        self.ram.write_u16(
            0x15810,
            self.ram
                .read_u16(0x20)
                .wrapping_add(12)
                .wrapping_sub(z as u16),
        );
        self.ram
            .write_u16(0x1580e, self.ram.read_u16(0x22).wrapping_add(8));
        if self.timer(k) == 0 {
            self.set_timer(k, 21);
            misc::ancilla_sfx3_near(self.ram, 0x30);
        }
        let priority = (self.ram.read_u16(0x64) >> 8) as u8;
        let mut index = self.current_oam_index();
        for i in (0..=usize::from(self.step(k))).rev() {
            if self.read(0x11) == 0 {
                self.write(0x15800 + i, self.read(0x15800 + i).wrapping_add(3) & 0x3f);
            }
            let radial = Self::radial_projection(self.read(0x15800 + i), self.read(0x15808));
            let (x, y) = self.sparkle_prep_oam_from_radial(radial);
            self.set_oam(index, x, y, CHAR[i], flags | priority, 0);
            self.set_xy(
                k,
                x.wrapping_add(self.ram.read_u16(0xe2)),
                y.wrapping_add(self.ram.read_u16(0xe8)),
            );
            self.set_direction(k, 0);
            self.check_sprite_collision(k);
            index += 1;
        }
    }

    pub fn duck(&mut self, k: usize) {
        const DMA: [u8; 4] = [0, 0x20, 0x40, 0xe0];
        const X: [i8; 3] = [0, -9, -9];
        const Y: [i8; 3] = [0, 12, 20];
        const CHAR: [u8; 3] = [0x0e, 0, 2];
        const FLAGS: [u8; 3] = [0x22, 0x2e, 0x2e];
        let mut forced_frame = None;
        if self.read(0x11) == 0 {
            if self.timer(k) != 0 {
                let features = u32::from_le_bytes(self.ram.slice(0x64c, 4).try_into().unwrap());
                self.set_xy(
                    k,
                    self.ram
                        .read_u16(0xe2)
                        .wrapping_sub(16 + if features & 1 != 0 { 0x40 } else { 0 }),
                    self.ram.read_u16(0x20).wrapping_sub(8),
                );
                return;
            }
            let timer = self.state_g(k).wrapping_sub(1);
            self.set_state_g(k, timer);
            if (timer as i8) < 0 {
                self.set_state_g(k, 0x28);
                self.sfx3_pan(k, 0x1e);
            }
            if self.state_l(k) != 0 {
                self.set_z_velocity(k, self.z_velocity(k).wrapping_sub(1));
                self.move_z(k);
            } else if self.step(k) != 0 {
                self.write(0xfc1, self.read(0xfc1).wrapping_add(1));
                self.set_z_velocity(k, self.z_velocity(k).wrapping_sub(1));
                self.move_z(k);
            }
            self.move_x(k);
            if self.state_l(k) != 0 {
                let x = self.get_x(k);
                if self.step(k) != 0 {
                    self.write(0xfc1, self.read(0xfc1).wrapping_add(1));
                }
                if (x as i16) >= 0 && x >= self.ram.read_u16(0x22) {
                    if self.step(k) != 0 {
                        self.set_step(k, 0);
                        self.write(0x4b, 0);
                        self.write(0x2f9, 0);
                        self.write(0x2da, 0);
                        self.set_y_velocity(k, 0);
                        self.write(0x2e4, 0);
                        self.write(0x37b, 0);
                        self.write(0x3fd, 0);
                        self.write(0x31f, 144);
                        if !matches!(self.read(0xf3cc), 12 | 13) || self.read(0xf3d3) == 0 {
                            self.runtime.follower_initialize(self.ram);
                        }
                    }
                } else if self.ram.read_u16(0x22).wrapping_sub(x) < 48 {
                    forced_frame = Some(3);
                }
            } else if self.check_link_collision(k, 1).is_some() && self.read(0x10) != 15 {
                let indoors = self.read(0x1b) != 0;
                let blocked_outdoors = !indoors
                    && (matches!(self.read(0x5d), 8..=10)
                        || self.read(0x5b) == 2
                        || (self.read(0x2da)
                            | self.read(0x37e)
                            | self.read(0x3ef)
                            | self.read(0x37b))
                            != 0
                        || self.read(0x308) & 0x80 != 0);
                if !blocked_outdoors {
                    if !indoors {
                        for slot in (0..=4).rev() {
                            if matches!(self.kind(slot), 0x2a | 0x1f | 0x30 | 0x31 | 0x41) {
                                self.clear(slot);
                            }
                        }
                        if self.read(0xf3cc) == 9 {
                            self.write(0xf3cc, 0);
                            self.write(0x2f9, 0);
                        }
                    }
                    self.write(0x308, 0);
                    self.write(0x309, 0);
                    self.ram.write_u16(0x11a, 0);
                    self.ram.write_u16(0x11c, 0);
                    self.runtime.link_reset_properties_a(self.ram);
                    self.write(0x345, 0);
                    self.write(0x3f8, 0);
                    self.write(0x4b, 12);
                    self.write(0x5d, 0);
                    self.write(0x2da, 1);
                    self.write(0x2e4, 1);
                    self.write(0x37b, 1);
                    self.write(0x2f9, 1);
                    self.set_step(k, 2);
                    self.write(0xfc1, self.read(0xfc1).wrapping_add(1));
                    self.write(0x373, 0);
                    if indoors {
                        self.write(0x3fd, self.read(0x1b));
                    }
                }
            }
        }
        let frame = if let Some(frame) = forced_frame {
            frame
        } else {
            let timer = self.arr3(k).wrapping_sub(1);
            self.set_arr3(k, timer);
            if (timer as i8) < 0 {
                self.set_arr3(k, 3);
                let frame = self.state_k(k).wrapping_add(1);
                self.set_state_k(k, if frame == 3 { 0 } else { frame });
            }
            usize::from(self.state_k(k))
        };
        self.write(0xaf4, DMA[frame]);
        let (x, y) = self.prep_oam_coord(k);
        let z = if self.height(k) != 0 {
            i16::from(self.height(k)) - 256
        } else {
            0
        };
        let original = self.current_oam_index();
        for i in 0..=usize::from(self.step(k)) {
            self.set_oam(
                original + i,
                x.wrapping_add(i16::from(X[i]) as u16),
                y.wrapping_add(z as u16)
                    .wrapping_add(i16::from(Y[i]) as u16),
                CHAR[i],
                FLAGS[i] | 0x30,
                2,
            );
        }
        let shadow = original + usize::from(self.step(k)) + 1;
        self.draw_shadow(shadow, 1, x, y.wrapping_add(28), 0x30);
        if self.step(k) != 0 {
            self.draw_shadow(shadow + 2, 1, x.wrapping_sub(7), y.wrapping_add(28), 0x30);
        }
        if (x as i16) >= 0 && x >= 0x130 {
            self.clear(k);
            if self.state_l(k) == 0 && self.step(k) != 0 {
                self.write(0x11, 10);
                self.write(0x10c, self.read(0x10));
                self.write(0x10, 14);
            }
        }
    }

    pub fn cutscene_duck(&mut self, k: usize) {
        const DIRECTION_FLAGS: [u8; 2] = [0x40, 0];
        const LIMIT: [u8; 2] = [28, 60];
        const DMA: [u8; 4] = [0, 0x20, 0x40, 0xe0];
        if self.read(0x1a) & 31 == 0 {
            self.sfx3_pan(k, 0x1e);
        }
        let animation_timer = self.arr3(k).wrapping_sub(1);
        self.set_arr3(k, animation_timer);
        if (animation_timer as i8) < 0 {
            self.set_arr3(k, 3);
            self.set_state_k(k, self.state_k(k) ^ 1);
        }
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if timer == 0 {
            self.set_aux_timer(k, 1);
            if self.state_l(k) == 0 {
                let phase = self.item_to_link(k).wrapping_sub(1);
                self.set_item_to_link(k, phase);
                if (phase as i8) >= 0 {
                    let velocity = if self.step(k) != 0 {
                        self.z_velocity(k).wrapping_add(1)
                    } else {
                        self.z_velocity(k).wrapping_sub(1)
                    };
                    self.set_z_velocity(k, velocity);
                    if (velocity as i8).unsigned_abs() >= 12 {
                        self.set_step(k, self.step(k) ^ 1);
                    }
                } else {
                    self.set_item_to_link(k, 0);
                    self.set_step(k, 0);
                    self.set_x_velocity(k, LIMIT[0]);
                    self.set_z_velocity(k, (-16_i8) as u8);
                    self.set_state_l(k, 1);
                    self.set_step(k, 3);
                }
            }
            if self.state_l(k) != 0 {
                let x_velocity = if self.step(k) & 1 == 0 {
                    self.x_velocity(k).wrapping_add(1)
                } else {
                    self.x_velocity(k).wrapping_sub(1)
                };
                self.set_x_velocity(k, x_velocity);
                let abs_x = (x_velocity as i8).unsigned_abs();
                if abs_x == 0 {
                    let phase = self.state_l(k).wrapping_add(1);
                    self.set_state_l(k, phase);
                    if phase == 7 {
                        self.set_state_s(k, 1);
                    }
                }
                let speed_index = usize::from(self.state_s(k));
                if abs_x >= LIMIT[speed_index] {
                    self.set_step(k, self.step(k) ^ 3);
                }
                self.set_direction(k, if (x_velocity as i8) < 0 { 2 } else { 3 });
                let vertical = LIMIT[speed_index].wrapping_sub(abs_x) >> 1;
                self.set_z_velocity(
                    k,
                    if self.step(k) & 2 != 0 {
                        vertical.wrapping_neg()
                    } else {
                        vertical
                    },
                );
            }
        }
        self.move_x(k);
        self.move_z(k);
        self.write(0xaf4, DMA[usize::from(self.state_k(k)) + 1]);
        let (x, y) = self.prep_oam_coord(k);
        let index = self.current_oam_index();
        self.set_oam(
            index,
            x,
            y.wrapping_add(i16::from(self.height(k) as i8) as u16),
            0x0e,
            0x22 | 0x30 | DIRECTION_FLAGS[usize::from(self.direction(k) & 1)],
            2,
        );
        self.draw_shadow(index + 1, 1, x, y.wrapping_add(48), 0x30);
        if (x as i16) >= 0 && x >= 248 {
            self.clear(k);
            self.write(0x11, 0);
            self.write(0xf34c, 3);
        }
    }

    fn add_occasional_sparkle(&mut self, k: usize) {
        if self.read(0x1a) & 7 == 0 {
            self.add_sword_charge_sparkle(k);
        }
    }

    fn item_receipt_add_rupees(&mut self, k: usize) -> bool {
        const VALUE: [u16; 5] = [1, 5, 20, 100, 50];
        let item = self.item_to_link(k);
        let amount = if (0x34..=0x36).contains(&item) {
            VALUE[usize::from(item - 0x34)]
        } else if matches!(item, 0x40 | 0x41) {
            VALUE[usize::from(item - 0x40 + 3)]
        } else if item == 0x46 {
            300
        } else if item == 0x47 {
            20
        } else {
            return false;
        };
        self.ram
            .write_u16(0xf360, self.ram.read_u16(0xf360).wrapping_add(amount));
        true
    }

    fn item_receipt_finish(&mut self, k: usize, reset_player: bool) {
        let item = self.item_to_link(k);
        if reset_player {
            if item == 1 && self.step(k) == 0 {
                self.write(0x12d, 5);
                self.write(0x12c, 2);
            }
            self.write(0x5d, if self.read(0x345) != 0 { 4 } else { 0 });
            self.write(0x2d8, 0);
            self.write(0x2da, 0);
            self.write(0x37b, 0);
            self.item_receipt_add_rupees(k);
        }
        self.write(0x2e9, 0);
        if item == 0x17 && self.read(0xf36b) == 0 {
            self.runtime.receive_item(self.ram, 0x26);
            self.clear(k);
            self.write(0xfc1, 0);
            return;
        }
        if matches!(item, 0x26 | 0x3f) {
            if self.read(0xf36c) != 0xa0 {
                let capacity = self.read(0xf36c).wrapping_add(8);
                self.write(0xf36c, capacity);
                self.write(
                    0xf372,
                    self.read(0xf372)
                        .wrapping_add(capacity.wrapping_sub(self.read(0xf36d))),
                );
                misc::ancilla_sfx3_near(self.ram, 0x0d);
            }
        } else if item == 0x3e {
            self.write(0x2e4, 0);
            if self.read(0xf36c) != 0xa0 {
                self.write(0xf36c, self.read(0xf36c).wrapping_add(8));
                self.write(0xf372, self.read(0xf372).wrapping_add(8));
                misc::ancilla_sfx3_near(self.ram, 0x0d);
            }
        } else if item == 0x42 {
            self.write(0xf372, self.read(0xf372).wrapping_add(8));
        } else if item == 0x45 {
            self.write(0xf373, self.read(0xf373).wrapping_add(16));
        } else if matches!(item, 0x22 | 0x23) {
            self.runtime.palette_load_link_armor_and_gloves(self.ram);
        }
        self.clear(k);
        self.write(0xfc1, 0);
        if self.step(k) == 3 && !matches!(item, 0x10 | 0x26 | 0x0f | 0x20) {
            self.runtime.prepare_dungeon_exit_from_boss_fight(self.ram);
        }
        if self.step(k) != 2 {
            self.write(0x2e4, 0);
        }
    }

    fn item_receipt_show_message(&mut self, k: usize) {
        const MESSAGE: [i16; 76] = [
            -1, 0x70, 0x77, 0x52, -1, 0x78, 0x78, 0x62, 0x61, 0x66, 0x69, 0x53, 0x52, 0x56, -1,
            0x64, 0x63, 0x65, 0x51, 0x54, 0x67, 0x68, 0x6b, 0x77, 0x79, 0x55, 0x6e, 0x58, 0x6d,
            0x5d, 0x57, 0x5e, -1, 0x74, 0x75, 0x76, -1, 0x5f, 0x158, -1, 0x6a, 0x5c, 0x8f, 0x71,
            0x72, 0x73, 0x71, 0x72, 0x73, 0x6a, 0x6c, 0x60, -1, -1, -1, 0x59, 0x84, 0x5a, -1, -1,
            -1, -1, -1, 0x159, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0xdb, 0x67, 0x7c,
        ];
        if self.read(0x1b) != 0
            && matches!(
                self.ram.read_u16(0xa0),
                0xff | 0x10f | 0x110 | 0x112 | 0x11f
            )
        {
            return;
        }
        let item = self.item_to_link(k);
        let message = if matches!(item, 0x38 | 0x39) {
            if self.read(0xf374) & 7 == 7 {
                [0x5b_i16, 0x83][usize::from(item - 0x38)]
            } else {
                MESSAGE[usize::from(item)]
            }
        } else if self.step(k) != 2 {
            if item == 0x17 {
                [-1_i16, 0x155, 0x156, 0x157][usize::from(self.read(0xf36b))]
            } else {
                MESSAGE[usize::from(item)]
            }
        } else {
            -1
        };
        if message >= 0 {
            self.ram.write_u16(0x1cf0, message as u16);
            if message == 0x70 {
                self.write(0x12d, 9);
            }
            self.runtime.main_show_text_message(self.ram);
        }
    }

    pub fn item_receipt(&mut self, k: usize) {
        enum Flow {
            Draw,
            Move,
            Message,
            Finish(bool),
        }
        let mut flow = Flow::Draw;
        if self.read(0x2e4) != 2 {
            let submodule = self.read(0x11);
            if submodule != 0 && !matches!(submodule, 43 | 9) {
                if submodule == 2 {
                    self.set_timer(k, 16);
                }
            } else {
                self.write(0xfc1, self.read(0xfc1).wrapping_add(1));
                if self.step(k) != 0 && self.step(k) != 3 {
                    let timer = self.aux_timer(k).wrapping_sub(1);
                    self.set_aux_timer(k, timer);
                    if (timer as i8) < 0 {
                        flow = Flow::Finish(false);
                    } else if timer == 0 {
                        flow = Flow::Message;
                    } else {
                        if timer == 40
                            && self.step(k) != 2
                            && (self.item_receipt_add_rupees(k) || self.item_to_link(k) != 0x17)
                        {
                            misc::ancilla_sfx3_near(self.ram, 0x0f);
                        }
                        flow = Flow::Move;
                    }
                } else if self.item_to_link(k) == 1 && self.step(k) != 2 {
                    let timer = self.timer(k);
                    if timer == 0 {
                        flow = Flow::Finish(true);
                    } else if timer == 17 {
                        self.ram.write_u16(0x2cd, 0xdf3);
                        self.write(0xf3cc, 0x0e);
                        flow = Flow::Message;
                    }
                } else {
                    let timer = self.aux_timer(k).wrapping_sub(1);
                    self.set_aux_timer(k, timer);
                    if timer == 0 {
                        flow = Flow::Finish(true);
                    } else if timer == 1 {
                        if matches!(self.item_to_link(k), 0x37..=0x39)
                            && self.runtime.audio_port0() != 0
                        {
                            self.set_aux_timer(k, timer.wrapping_add(1));
                        } else {
                            flow = Flow::Message;
                        }
                    }
                }
            }
        }
        match flow {
            Flow::Finish(reset) => {
                self.item_receipt_finish(k, reset);
                return;
            }
            Flow::Message => self.item_receipt_show_message(k),
            Flow::Move => {
                if self.aux_timer(k) >= 24 {
                    let velocity = self.y_velocity(k).wrapping_sub(1);
                    if velocity >= 248 {
                        self.set_y_velocity(k, velocity);
                    }
                    self.move_y(k);
                }
            }
            Flow::Draw => {}
        }
        let item = self.item_to_link(k);
        if item == 0x20 {
            self.set_height(k, 0);
            self.add_occasional_sparkle(k);
            if self.runtime.audio_port0() == 0 {
                self.write(0x12c, 0x1a);
                self.transmute_to_rising_crystal(k);
                return;
            }
        } else if item == 1 {
            const FLAGS: [u8; 3] = [5, 1, 4];
            self.set_arr4(k, FLAGS[0]);
            if self.step(k) != 2 {
                if self.timer(k) < 16 {
                    self.set_arr1(k, 0);
                    self.set_arr4(k, FLAGS[0]);
                } else {
                    let timer = self.arr3(k).wrapping_sub(1);
                    self.set_arr3(k, timer);
                    if (timer as i8) < 0 {
                        self.set_arr3(k, 2);
                        let frame = self.arr1(k).wrapping_add(1);
                        let frame = if frame == 3 { 0 } else { frame };
                        self.set_arr1(k, frame);
                        self.set_arr4(k, FLAGS[usize::from(frame)]);
                    }
                }
            }
        }
        if matches!(item, 0x34..=0x36) {
            let timer = self.arr3(k).wrapping_sub(1);
            self.set_arr3(k, timer);
            if (timer as i8) < 0 {
                const DELAY: [u8; 3] = [9, 5, 5];
                const TILE: [u8; 3] = [0x24, 0x25, 0x26];
                let frame = self.arr1(k).wrapping_add(1);
                let frame = if frame == 3 { 0 } else { frame };
                self.set_arr1(k, frame);
                self.set_arr3(k, DELAY[usize::from(frame)]);
                self.runtime
                    .write_to_4bpp_buffer(self.ram, TILE[usize::from(frame)]);
            }
        }
        let (x, y) = self.prep_adjusted_oam_coord(k);
        self.receive_item_draw(k, x, y);
    }

    fn transmute_to_rising_crystal(&mut self, k: usize) {
        self.set_kind(k, 0x3e);
        self.set_y_velocity(k, 0);
        self.set_x_velocity(k, 0);
        self.set_y_fraction(k, 0);
        self.rising_crystal(k);
    }

    pub fn milestone_item_receipt(&mut self, k: usize) {
        let item = self.item_to_link(k);
        if !matches!(item, 0x10 | 0x0f) {
            let room_state = self.ram.read_u16(0x402);
            if room_state & 0x4000 != 0 {
                self.clear(k);
                return;
            }
            if room_state & 0x8000 == 0 {
                return;
            }
            let decode_timer = self.read(0x4c2);
            if decode_timer != 0 {
                if decode_timer == 1 {
                    if item == 0x20 {
                        self.write(0x12d, 0x0f);
                        self.runtime
                            .decode_animated_sprite_tile_variable(self.ram, 0x28);
                    } else {
                        self.runtime
                            .decode_animated_sprite_tile_variable(self.ram, 0x23);
                    }
                }
                self.write(0x4c2, decode_timer - 1);
                return;
            }
            if self.arr3(k) == 0 && item == 0x20 {
                self.set_arr3(k, 1);
                self.write(0xab1, 4);
                self.ram.write_u16(0xaa8, 0x200);
                self.runtime
                    .palette_load_sprite_environment_dungeon(self.ram);
                self.write(0x15, self.read(0x15).wrapping_add(1));
            }
        } else if self.state_g(k) != 0 {
            self.set_state_g(k, self.state_g(k) - 1);
            return;
        }
        if item == 0x20 {
            self.add_occasional_sparkle(k);
        }
        if self.read(0x11) == 0 {
            if self.height(k) < 24
                && self.check_link_collision(k, 2).is_some()
                && self.read(0x37e) == 0
                && self.read(0x4d) == 0
            {
                self.clear(k);
                if matches!(self.read(0x5d), 25 | 26) {
                    self.write(0x112, 0);
                    self.write(0x3ef, 0);
                    self.write(0x5d, 0);
                }
                self.write(0x2e9, 3);
                self.runtime.receive_item(self.ram, item);
                return;
            }
            if self.step(k) != 2 {
                if self.step(k) != 0 {
                    self.set_z_velocity(k, self.z_velocity(k).wrapping_sub(1));
                }
                self.move_z(k);
                if self.height(k) >= 0xf8 {
                    self.set_step(k, self.step(k).wrapping_add(1));
                    self.set_z_velocity(k, 0x18);
                    self.set_height(k, 0);
                }
            }
        }
        let (x, y) = self.prep_adjusted_oam_coord(k);
        let index = self.receive_item_draw(k, x, y.wrapping_sub(u16::from(self.height(k))));
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) < 0 {
            self.set_aux_timer(k, 9);
            let frame = self.state_l(k).wrapping_add(1);
            self.set_state_l(k, if frame == 3 { 0 } else { frame });
        }
        let shadow = if self.height(k) == 0 {
            if self.ram.read_u16(0xa0) == 6 {
                usize::from(self.state_l(k)) + 4
            } else {
                0
            }
        } else if self.height(k) < 0x20 {
            1
        } else {
            2
        };
        self.draw_shadow(index, shadow, x, y.wrapping_add(12), 0x20);
    }

    pub fn rising_crystal(&mut self, k: usize) {
        const CRYSTAL_BITS: [u8; 13] = [0, 0, 4, 2, 0, 16, 2, 1, 64, 4, 1, 32, 8];
        self.set_height(k, 0);
        self.add_occasional_sparkle(k);
        let mut velocity = self.y_velocity(k).wrapping_sub(1);
        if velocity < 0xf0 {
            velocity = 0xf0;
        }
        self.set_y_velocity(k, velocity);
        self.move_y(k);
        let scroll = self.ram.read_u16(0xe8);
        if self.get_y(k).wrapping_sub(scroll) < 0x49 {
            self.set_y(k, scroll.wrapping_add(0x49));
            if self.read(0x11) == 0 {
                let palace = usize::from(self.read(0x40c) >> 1);
                self.write(0xf37a, self.read(0xf37a) | CRYSTAL_BITS[palace]);
                self.write(0x11, 0x18);
                self.write(0xb0, 0);
                for address in 0xc340..0xc400 {
                    self.write(address, 0);
                }
                self.ram.write_u16(0xc007, 0);
                self.ram.write_u16(0xc009, 0);
            }
        }
        let (x, y) = self.prep_adjusted_oam_coord(k);
        self.receive_item_draw(k, x, y);
    }

    pub fn big_bomb_explosion(&mut self, k: usize) {
        const X: [i8; 9] = [0, -16, 0, 16, -24, 24, -16, 0, 16];
        const Y: [i8; 9] = [0, -16, -24, -16, 0, 0, 16, 24, 16];
        const DRAW_FRAME: [u8; 12] = [0, 1, 2, 3, 2, 3, 4, 5, 6, 7, 8, 9];
        const DRAW_COUNT: [u8; 11] = [1, 4, 4, 4, 4, 4, 5, 4, 6, 6, 6];
        if self.read(0x11) == 0 {
            let timer = self.arr3(k).wrapping_sub(1);
            self.set_arr3(k, timer);
            if timer == 0 {
                let state = self.item_to_link(k).wrapping_add(1);
                self.set_item_to_link(k, state);
                if state == 2 {
                    self.sfx2_pan(k, 0x0c);
                }
                if state == 11 {
                    self.clear(k);
                    return;
                }
                self.set_arr3(k, super::BOMB_FRAME_DELAYS[usize::from(state)]);
            }
        }
        self.ram.write_u16(0x64, 0x3000);
        let state = usize::from(self.item_to_link(k));
        let count = usize::from(DRAW_COUNT[state]);
        let frame = usize::from(DRAW_FRAME[state]) * 6;
        self.set_step(k, (frame * 2) as u8);
        let mut drawn = 0;
        for i in (0..9).rev() {
            let x = self
                .get_x(k)
                .wrapping_add(i16::from(X[i]) as u16)
                .wrapping_sub(self.ram.read_u16(0xe2));
            let y = self
                .get_y(k)
                .wrapping_add(i16::from(Y[i]) as u16)
                .wrapping_sub(self.ram.read_u16(0xe8));
            if x < 256 && y < 256 {
                self.allocate_oam_from_region_a_d_f(frame * 2, 0x18);
                let start = self.current_oam_index() + drawn;
                let end = self.draw_explosion(start, frame, 0, count, 0x32, (x, y));
                drawn += end - start;
            }
        }
        if self.item_to_link(k) == 3 && self.arr3(k) == 1 {
            let features = u32::from_le_bytes(self.ram.slice(0x64c, 4).try_into().unwrap());
            let old = if features & 4096 != 0 {
                self.read(0xf3cc)
            } else {
                0
            };
            self.write(0xf3cc, 13);
            self.runtime
                .bomb_check_for_destructibles(self.ram, self.get_x(k), self.get_y(k), 0);
            self.write(0xf3cc, old);
        }
    }

    pub fn add_somaria_platform_poof(&mut self, k: usize) {
        self.set_kind(k, 0x39);
        self.set_aux_timer(k, 7);
        for sprite in (0..16).rev() {
            if self.read(0xe20 + sprite) == 0xed {
                self.write(0xdd0 + sprite, 0);
                self.write(0x2f5, 0);
            }
        }
        self.runtime.player_tile_detect_nearby(self.ram);
    }

    fn somaria_block_check_for_switch(&mut self, k: usize) -> bool {
        const X: [i8; 4] = [0, 0, -4, 4];
        const Y: [i8; 4] = [-4, 4, 0, 0];
        self.write(0x646, 0);
        self.set_arr24(k, 0);
        for j in (0..4).rev() {
            let x = self.get_x(k).wrapping_add(i16::from(X[j]) as u16);
            let y = self.get_y(k).wrapping_add(i16::from(Y[j]) as u16);
            let priority = self.object_priority(k);
            self.check_tile_collision_targeted(k, x, y);
            self.set_object_priority(k, priority);
            if matches!(self.tile_attribute(k), 0x23 | 0x24 | 0x25 | 0x3b) {
                self.set_arr24(k, self.arr24(k).wrapping_add(1));
            }
        }
        self.arr24(k) != 4
    }

    fn somaria_block_fizzle_away(&mut self, k: usize) {
        if self.read(0x5e) == 18 {
            self.write(0x48, 0);
            self.write(0x5e, 0);
        }
        self.write(0x646, 0);
        self.set_kind(k, 0x2d);
        self.set_aux_timer(k, 0);
        self.set_step(k, 0);
        self.set_item_to_link(k, 0);
        self.set_arr3(k, 0);
        self.set_arr1(k, 0);
        self.set_state_r(k, 0);
        if usize::from(self.read(0x2ec)) == k + 1 {
            self.write(0x2ec, 0);
            self.write(0x308, self.read(0x308) & 0x80);
        }
        self.somaria_block_fizz(k);
    }

    fn somaria_block_draw(&mut self, k: usize) {
        const X: [i8; 12] = [-8, 0, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        const Y: [i8; 12] = [-8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        const FLAGS: [u8; 12] = [
            0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
        ];
        let picked = usize::from(self.read(0x2ec)) == k + 1;
        if picked && self.read(0x308) & 0x80 != 0 && self.state_k(k) != 3 && self.read(0x2f) == 0 {
            self.runtime.allocate_oam_region(
                self.ram,
                if self.read(0xfb3) == 0 { b'B' } else { b'E' },
                self.sprite_count(k),
            );
        } else if self.read(0xfb3) != 0
            && self.floor(k) != 0
            && (self.state_l(k) != 0 || picked && self.read(0x308) & 0x80 != 0)
        {
            oam::OamPointers::for_cursor(oam::OamCursor::from_byte_offset(0xd0))
                .publish_entry_first(self.ram);
        }
        let (x, mut y) = self.prep_adjusted_oam_coord(k);
        let z = self.height(k) as i8;
        if z != 0 && z != -1 && self.state_k(k) != 3 && self.object_priority(k) != 0 {
            self.ram.write_u16(0x64, 0x3000);
        }
        y = y.wrapping_sub(i16::from(z) as u16);
        let original = self.current_oam_index();
        let mut table = usize::from(self.arr1(k)) * 4;
        let priority = (self.ram.read_u16(0x64) >> 8) as u8;
        for i in 0..4 {
            self.set_oam_safe(
                original + i,
                x.wrapping_add(i16::from(X[table]) as u16),
                y.wrapping_add(i16::from(Y[table]) as u16),
                0xe9,
                FLAGS[table] & !0x30 | 2 | priority,
                0,
            );
            table += 1;
        }
        if self.somarian_block_check_empty(original) {
            self.write(0x646, 0);
            self.clear(k);
            if picked {
                self.write(0x2ec, 0);
                if self.read(0x308) & 0x80 != 0 {
                    self.write(0x308, 0);
                }
            }
        }
    }

    pub fn somaria_block(&mut self, k: usize) {
        const COLLISION_X: [i8; 12] = [0, 0, -8, 8, 0, 0, 0, 0, 8, -8, -8, 8];
        const COLLISION_Y: [i8; 12] = [-8, 8, 0, 0, 0, 0, 0, 0, -8, 8, -8, 8];
        let cycle = self.state_g(k).wrapping_sub(1);
        self.set_state_g(k, cycle);
        if (cycle as i8) >= 0 {
            return;
        }
        self.set_state_g(k, 0);
        let mut picked = usize::from(self.read(0x2ec)) == k + 1;
        if self.state_h(k) == 0 {
            if matches!(self.read(0x11), 0 | 8 | 16) {
                self.handle_lift_logic(k);
                picked = usize::from(self.read(0x2ec)) == k + 1;
            } else if picked && self.state_k(k) != 0 {
                if self.state_k(k) != 3 {
                    self.latch_link_coordinates(k, 3);
                    self.latch_altitude_above_link(k);
                    self.set_state_k(k, 3);
                }
                self.latch_carried_position(k);
            }
            if self.read(0x1b) != 0 {
                if self.state_k(k) == 0
                    && self.read(0x308) & 0x80 == 0
                    && matches!(self.height(k), 0 | 0xff)
                {
                    if self.read(0x3f4) != 0 {
                        let mut j = usize::from(self.read(0x1a) & 3);
                        while j < 12 {
                            let priority = self.object_priority(k);
                            let x = self.get_x(k).wrapping_add(i16::from(COLLISION_X[j]) as u16);
                            let y = self.get_y(k).wrapping_add(i16::from(COLLISION_Y[j]) as u16);
                            self.check_tile_collision_targeted(k, x, y);
                            self.set_object_priority(k, priority);
                            if matches!(self.tile_attribute(k), 0xb6 | 0xbc) {
                                self.set_xy(k, x, y);
                                self.add_somaria_platform_poof(k);
                                if picked {
                                    self.write(0x2ec, 0);
                                }
                                return;
                            }
                            j += 4;
                        }
                    } else if !self.somaria_block_check_for_switch(k)
                        && matches!(self.height(k), 0 | 0xff)
                    {
                        self.write(0x646, self.read(0x646).wrapping_add(1));
                    }
                } else if picked {
                    self.write(0x646, 0);
                }
            }
        } else if picked {
            self.write(0x646, 0);
        }
        let mut old_y = self.latch_y_coord_to_z(k);
        let old_direction = self.direction(k);
        let mut old_priority = self.object_priority(k);
        self.set_object_priority(k, 0);
        let collision = self.check_tile_collision_class2(k);
        if self.read(0x1b) != 0 && self.state_l(k) != 0 && self.tile_attribute(k) == 0x1c {
            self.set_state_t(k, 1);
        }
        if collision && (self.read(0x308) & 0x80 == 0 || self.read(0x309) != 0) {
            if old_priority == 0 && self.arr4(k) == 0 && self.height(k) != 0 {
                self.set_arr4(k, 1);
                let y_speed = if self.direction(k) == 1 { 16 } else { 4 };
                if self.y_velocity(k) != 0 {
                    self.set_y_velocity(
                        k,
                        if (self.y_velocity(k) as i8) < 0 {
                            y_speed
                        } else {
                            (-(y_speed as i8)) as u8
                        },
                    );
                }
                if self.x_velocity(k) != 0 {
                    self.set_x_velocity(
                        k,
                        if (self.x_velocity(k) as i8) < 0 {
                            4
                        } else {
                            (-4_i8) as u8
                        },
                    );
                }
                if self.direction(k) == 1 {
                    self.set_y_velocity(k, (-4_i8) as u8);
                    self.set_state_l(k, 2);
                }
            }
        } else if self.read(0x308) & 0x80 == 0 && matches!(self.height(k), 0 | 0xff) {
            self.set_direction(k, 16);
            let priority = self.object_priority(k);
            self.check_tile_collision(k);
            self.set_object_priority(k, priority);
            let attribute = self.tile_attribute(k);
            if attribute == 0x26 {
                if old_priority == 0 && self.arr4(k) == 0 && self.height(k) != 0 {
                    self.set_arr4(k, 1);
                    let y_speed = if self.direction(k) == 1 { 16 } else { 4 };
                    if self.y_velocity(k) != 0 {
                        self.set_y_velocity(
                            k,
                            if (self.y_velocity(k) as i8) < 0 {
                                y_speed
                            } else {
                                (-(y_speed as i8)) as u8
                            },
                        );
                    }
                    if self.x_velocity(k) != 0 {
                        self.set_x_velocity(
                            k,
                            if (self.x_velocity(k) as i8) < 0 {
                                4
                            } else {
                                (-4_i8) as u8
                            },
                        );
                    }
                    if self.direction(k) == 1 {
                        self.set_y_velocity(k, (-4_i8) as u8);
                        self.set_state_l(k, 2);
                    }
                }
            } else if matches!(attribute, 0x0c | 0x1c) {
                if self.read(0x46c) != 3 {
                    if self.floor(k) == 0 && !matches!(self.height(k), 0 | 0xff) {
                        self.set_floor(k, 1);
                    }
                } else {
                    old_y = self.get_y(k).wrapping_add(self.ram.read_u16(0x310));
                    self.set_x(k, self.get_x(k).wrapping_add(self.ram.read_u16(0x312)));
                }
            } else if attribute == 0x20
                || attribute & 0xf0 == 0xb0 && !matches!(attribute, 0xb6 | 0xbc)
            {
                if self.read(0x308) & 0x80 == 0 {
                    if picked {
                        self.write(0x2ec, 0);
                    }
                    if self.timer(k) == 0 {
                        if self.read(0x5e) == 18 {
                            self.write(0x5e, 0);
                            self.write(0x48, 0);
                        }
                        self.clear(k);
                        return;
                    }
                }
            } else if attribute == 8 {
                if picked {
                    self.write(0x2ec, 0);
                }
                if self.timer(k) == 0 {
                    self.set_y(k, self.get_y(k).wrapping_sub(24));
                    self.transmute_to_splash(k);
                    return;
                }
            } else if matches!(attribute, 0x68..=0x6b) {
                self.apply_conveyor(k);
                old_y = self.get_y(k);
            } else {
                self.set_timer(
                    k,
                    if self.state_l(k) | self.state_h(k) != 0 {
                        0
                    } else {
                        2
                    },
                );
            }
        }
        old_priority |= self.object_priority(k);
        if self.read(0x308) & 0x80 == 0 {
            let timer = self.state_s(k).wrapping_sub(1);
            self.set_state_s(k, timer);
            if timer == 0 {
                self.set_state_s(k, 1);
                self.set_object_priority(k, 0);
                if self.check_basic_sprite_collision(k).is_some() {
                    self.set_state_s(k, 7);
                    let step = self.step(k).wrapping_add(1);
                    self.set_step(k, step);
                    if step == 5 {
                        self.somaria_block_fizzle_away(k);
                        return;
                    }
                }
            }
        }
        self.set_y(k, old_y);
        self.set_direction(k, old_direction);
        self.set_object_priority(k, old_priority);
        self.somaria_block_draw(k);
    }

    pub fn somaria_platform_poof(&mut self, k: usize) {
        const DIRECTION: [u8; 4] = [1, 0, 3, 2];
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) >= 0 {
            return;
        }
        self.clear(k);
        let x = self.get_x(k) & !7 | 4;
        let y = self.get_y(k) & !7 | 4;
        let floor = self.floor(k);
        if let Some(sprite) = self.runtime.spawn_sprite_dynamically(self.ram, k, 0xed) {
            self.write(0x2f5, 0);
            let [x_lo, x_hi] = x.to_le_bytes();
            let [y_lo, y_hi] = y.to_le_bytes();
            self.write(0xd10 + sprite, x_lo);
            self.write(0xd30 + sprite, x_hi);
            self.write(0xd00 + sprite, y_lo);
            self.write(0xd20 + sprite, y_hi);
            let position = usize::from((x & 0x1f8) >> 3)
                + usize::from((y & 0x1f8) << 3)
                + if floor >= 1 { 0x1000 } else { 0 };
            let mut direction = 0;
            if self.read(0x12000 + position - 64) & 0xf0 != 0xb0 {
                direction += 1;
                if self.read(0x12000 + position + 64) & 0xf0 != 0xb0 {
                    direction += 1;
                    if self.read(0x12000 + position - 1) & 0xf0 != 0xb0 {
                        direction += 1;
                    }
                }
            }
            self.write(0xde0 + sprite, DIRECTION[direction]);
            self.write(0xf20 + sprite, 0);
        } else {
            self.somaria_block_draw(k);
        }
    }

    pub fn add_cutscene_duck(&mut self, kind: u8, limit: u8) {
        if (0..=5).any(|slot| self.kind(slot) == kind) {
            return;
        }
        if let Some(k) = self.add_ancilla(kind, limit) {
            self.set_direction(k, 2);
            self.set_arr3(k, 3);
            self.set_step(k, 0);
            self.set_aux_timer(k, 32);
            self.set_item_to_link(k, 116);
            self.set_z_velocity(k, 0);
            self.set_state_l(k, 0);
            self.set_height(k, 0);
            self.set_state_s(k, 0);
            self.set_xy(k, 0x200, 0x788);
        }
    }

    fn draw_weathervane_debris(&mut self, k: usize) {
        const CHAR: [u8; 2] = [0x4e, 0x4f];
        let (x, y) = self.prep_oam_coord(k);
        let y = y.wrapping_sub(i16::from(self.height(k) as i8) as u16);
        let frame = self.item_to_link(k);
        if (frame as i8) < 0 {
            return;
        }
        self.set_oam(
            self.current_oam_index() + usize::from(self.read(0x15879) >> 2),
            x,
            y,
            CHAR[usize::from(frame)],
            0x3c,
            0,
        );
        self.write(0x15879, self.read(0x15879).wrapping_add(4));
    }

    pub fn weathervane_explosion(&mut self, k: usize) {
        let master_timer = self.ram.read_u16(0x158b6).wrapping_sub(1);
        self.ram.write_u16(0x158b6, master_timer);
        if master_timer != 0 {
            return;
        }
        self.ram.write_u16(0x158b6, 1);
        if self.read(0x158b8) == 0 {
            self.write(0x158b8, 1);
            self.write(0x12c, 0xf3);
        }
        let cycle = self.state_g(k).wrapping_sub(1);
        self.set_state_g(k, cycle);
        if cycle != 0 {
            return;
        }
        self.set_state_g(k, 1);
        if self.arr3(k) == 0 {
            self.set_arr3(k, 1);
            misc::ancilla_sfx2_near(self.ram, 0x0c);
        }
        if self.step(k) == 0 {
            let timer = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, timer);
            if (timer as i8) < 0 {
                self.set_step(k, 1);
                self.runtime.overworld_alter_weathervane(self.ram);
                self.add_cutscene_duck(0x38, 0);
            }
        }
        self.write(0x15878, k as u8);
        self.write(0x15879, 0);
        for i in (0..12).rev() {
            let state = self.read(0x1586c + i);
            if state == 0xff {
                continue;
            }
            let timer = self.read(0x15860 + i).wrapping_sub(1);
            self.write(0x15860 + i, timer);
            if (timer as i8) < 0 {
                self.write(0x15860 + i, 1);
                self.write(0x1586c + i, state ^ 1);
            }
            self.set_item_to_link(k, self.read(0x1586c + i));
            self.set_y_low(k, self.read(0x15824 + i));
            self.set_y_high(k, self.read(0x15830 + i));
            self.set_x_low(k, self.read(0x1583c + i));
            self.set_x_high(k, self.read(0x15848 + i));
            self.set_height(k, self.read(0x15854 + i));
            self.set_y_velocity(k, self.read(0x15800 + i));
            self.set_x_velocity(k, self.read(0x1580c + i));
            let z_velocity = self.read(0x15818 + i).wrapping_sub(1);
            self.write(0x15818 + i, z_velocity);
            self.set_z_velocity(k, z_velocity);
            self.move_y(k);
            self.move_x(k);
            self.move_z(k);
            let dead = self.height(k) >= 0xf0;
            self.draw_weathervane_debris(k);
            if dead {
                self.write(0x1586c + i, 0xff);
            }
            self.write(0x15824 + i, self.y_low(k));
            self.write(0x15830 + i, self.y_high(k));
            self.write(0x1583c + i, self.x_low(k));
            self.write(0x15848 + i, self.x_high(k));
            self.write(0x15854 + i, self.height(k));
        }
        if (0..12).all(|i| self.read(0x1586c + i) == 0xff) {
            self.clear(k);
        }
    }

    pub fn skull_woods_fire(&mut self, k: usize) {
        const PAN: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
        const Y: [i8; 4] = [0, 0, 0, -3];
        const CHAR: [u8; 4] = [0x8e, 0xa0, 0xa2, 0xa4];
        const EXT: [u8; 4] = [2, 2, 2, 0];
        const OVERLAY_X: [i8; 24] = [
            -13, -21, -10, -1, -1, -1, -16, -27, -4, -16, -6, -25, -16, -27, -4, -16, -6, -25, -13,
            -5, -27, -11, -22, -3,
        ];
        const OVERLAY_Y: [i8; 24] = [
            -31, -24, -22, -1, -1, -1, -37, -32, -32, -23, -16, -14, -37, -32, -32, -23, -16, -14,
            -35, -29, -28, -20, -13, -11,
        ];
        const OVERLAY_CHAR: [u8; 24] = [
            0x86, 0x86, 0x86, 0xff, 0xff, 0xff, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x8a, 0x8a,
            0x8a, 0x8a, 0x8a, 0x8a, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b,
        ];
        const OVERLAY_FLAGS: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x40, 0x40, 0x80, 0x40, 0,
        ];
        const OVERLAY_EXT: [u8; 24] = [
            2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
        ];
        if self.read(0x10010) != 0 && self.item_to_link(k) != 4 {
            let timer = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, timer);
            if (timer as i8) < 0 {
                self.set_aux_timer(k, 5);
                self.set_item_to_link(k, self.item_to_link(k).wrapping_add(1));
            }
        }
        let mut index = self.current_oam_index();
        for slot in (0..4).rev() {
            let timer = self.read(0x10008 + slot).wrapping_sub(1);
            self.write(0x10008 + slot, timer);
            if (timer as i8) < 0 {
                self.write(0x10008 + slot, 5);
                let mut frame = self.read(0x10000 + slot);
                if frame != 128 {
                    frame = frame.wrapping_add(1);
                    self.write(0x10000 + slot, frame);
                    if frame == 0 || frame == 4 {
                        if frame == 4 {
                            self.write(0x10000 + slot, 0);
                        }
                        let y = self.ram.read_u16(0x10018).wrapping_sub(8);
                        self.ram.write_u16(0x10018, y);
                        if y < 200 && self.read(0x10010) != 1 {
                            self.write(0x10010, 1);
                            let pan = usize::from(0x98_u8.wrapping_sub(self.read(0xe2)) >> 5);
                            self.write(0x12e, PAN[pan] | 0x0c);
                        }
                        if y < 168 {
                            self.write(0x10000 + slot, 128);
                        }
                        let x = self.ram.read_u16(0x1001a);
                        self.ram.write_u16(0x10030 + slot * 2, x);
                        self.ram.write_u16(0x10020 + slot * 2, y);
                        if self.read(0x12e) == 0 {
                            let pan =
                                usize::from(x.wrapping_sub(self.ram.read_u16(0xe2)) as u8 >> 5);
                            self.write(0x12e, PAN[pan] | 0x2a);
                        }
                    }
                }
            }
            let frame = self.read(0x10000 + slot);
            if (frame as i8) >= 0 {
                let frame = usize::from(frame);
                let x = self
                    .ram
                    .read_u16(0x10030 + slot * 2)
                    .wrapping_sub(self.ram.read_u16(0xe2));
                let y = self
                    .ram
                    .read_u16(0x10020 + slot * 2)
                    .wrapping_sub(self.ram.read_u16(0xe8))
                    .wrapping_add(i16::from(Y[frame]) as u16);
                self.set_oam(index, x, y, CHAR[frame], 0x32, EXT[frame]);
                index += 1;
                if EXT[frame] != 2 {
                    self.set_oam(
                        index,
                        x.wrapping_add(8),
                        y,
                        CHAR[frame] + 1,
                        0x32,
                        EXT[frame],
                    );
                    index += 1;
                }
            }
        }
        if (0..4).all(|slot| (self.read(0x10000 + slot) as i8) < 0) {
            self.clear(k);
            return;
        }
        if self.read(0x10010) == 0 || self.item_to_link(k) == 4 {
            return;
        }
        let mut table = usize::from(self.item_to_link(k)) * 6;
        for _ in 0..6 {
            if OVERLAY_CHAR[table] != 0xff {
                self.set_oam(
                    index,
                    168_u16
                        .wrapping_sub(self.ram.read_u16(0xe2))
                        .wrapping_add(i16::from(OVERLAY_X[table]) as u16),
                    200_u16
                        .wrapping_sub(self.ram.read_u16(0xe8))
                        .wrapping_add(i16::from(OVERLAY_Y[table]) as u16),
                    OVERLAY_CHAR[table],
                    OVERLAY_FLAGS[table] | 0x32,
                    OVERLAY_EXT[table],
                );
                index += 1;
            }
            table += 1;
        }
    }

    fn gt_cutscene_draw_crystal(&mut self, index: usize, x: u16, y: u16) {
        self.set_oam_safe(index, x, y, 0x24, 0x3c, 2);
    }

    fn gt_cutscene_activate_sparkle(&mut self) {
        for slot in (0..24).rev() {
            if self.read(0x15837 + slot) == 0xff {
                self.write(0x15837 + slot, 0);
                self.write(0x158af + slot, 4);
                let random = misc::get_random_number(self.ram);
                let base = slot & 7;
                let x = u16::from_le_bytes([self.read(0x15827 + base), self.read(0x1582f + base)])
                    .wrapping_add(u16::from(random >> 4));
                let y = u16::from_le_bytes([self.read(0x15817 + base), self.read(0x1581f + base)])
                    .wrapping_add(u16::from(random & 0x0f));
                let [x_lo, x_hi] = x.to_le_bytes();
                let [y_lo, y_hi] = y.to_le_bytes();
                self.write(0x1587f + slot, x_lo);
                self.write(0x15897 + slot, x_hi);
                self.write(0x1584f + slot, y_lo);
                self.write(0x15867 + slot, y_hi);
                return;
            }
        }
    }

    fn gt_cutscene_sparkle_a_lot(&mut self, mut index: usize) -> usize {
        const CHAR: [u8; 3] = [0xb7, 0x80, 0x83];
        const FLAGS: [u8; 3] = [4, 4, 0x84];
        for slot in (0..24).rev() {
            let frame = self.read(0x15837 + slot);
            if frame == 0xff {
                continue;
            }
            let timer = self.read(0x158af + slot).wrapping_sub(1);
            self.write(0x158af + slot, timer);
            let frame = if (timer as i8) < 0 {
                self.write(0x158af + slot, 4);
                let frame = frame.wrapping_add(1);
                self.write(0x15837 + slot, frame);
                if frame == 3 {
                    self.write(0x15837 + slot, 0xff);
                    continue;
                }
                frame
            } else {
                frame
            };
            let x = u16::from_le_bytes([self.read(0x1587f + slot), self.read(0x15897 + slot)]);
            let y = u16::from_le_bytes([self.read(0x1584f + slot), self.read(0x15867 + slot)]);
            self.set_oam(
                index,
                x,
                y,
                CHAR[usize::from(frame)],
                FLAGS[usize::from(frame)] | 0x30,
                0,
            );
            index += 1;
        }
        index
    }

    fn gt_cutscene_advance_radius(&mut self, k: usize, velocity: u8) -> u8 {
        let old_x = self.get_x(k);
        self.set_x_velocity(k, velocity);
        self.set_x(k, u16::from(self.read(0x15808)));
        self.move_x(k);
        let radius = self.get_x(k) as u8;
        self.write(0x15808, radius);
        self.set_x(k, old_x);
        radius
    }

    pub fn ganons_tower_cutscene(&mut self, k: usize) {
        let mut index = self.current_oam_index();
        let mut central_only = false;
        if self.step(k) == 0 {
            let velocity = self.y_velocity(k).wrapping_sub(1);
            self.set_y_velocity(k, if velocity < 0xf0 { 0xf0 } else { velocity });
            self.move_y(k);
            let x = self.get_x(k);
            let y = self.get_y(k);
            if y.wrapping_sub(self.ram.read_u16(0xe8)) < 0x38 {
                self.ram
                    .write_u16(0x15810, self.ram.read_u16(0xe8).wrapping_add(0x40));
                self.ram.write_u16(0x1580e, x.wrapping_add(8));
                self.set_y(k, self.ram.read_u16(0xe8).wrapping_add(0x38));
                self.set_step(k, self.step(k).wrapping_add(1));
                self.write(0x12d, 5);
                self.write(0x12c, 0xf1);
                self.ram.write_u16(0x1cf0, 0x13b);
                self.runtime.main_show_text_message(self.ram);
                central_only = true;
            }
        }
        if !central_only {
            if self.step(k) == 1 && self.read(0x11) == 0 {
                let radius = self.gt_cutscene_advance_radius(k, 16);
                if radius >= 48 {
                    self.write(0x15808, 48);
                    self.set_step(k, self.step(k).wrapping_add(1));
                }
            }
            if self.read(0x11) == 0 {
                match self.step(k) {
                    0 => central_only = true,
                    1 => {}
                    2 => {
                        let timer = self.read(0x15812).wrapping_sub(1);
                        self.write(0x15812, timer);
                        if timer == 0 {
                            self.write(0x4c6, 5);
                            self.write(0xb0, 0);
                            self.write(0xc8, 0);
                            self.set_step(k, self.step(k).wrapping_add(1));
                        }
                    }
                    _ => {
                        let radius = self.gt_cutscene_advance_radius(k, 48);
                        if radius >= 240 {
                            self.write(0xab1, 0);
                            self.ram.write_u16(0xaa8, 0x200);
                            self.runtime
                                .palette_load_sprite_environment_dungeon(self.ram);
                            self.write(0x15, self.read(0x15).wrapping_add(1));
                            self.clear(k);
                            return;
                        }
                    }
                }
            }
            if !central_only {
                let step = self.step(k);
                if step != 0 {
                    index = self.gt_cutscene_sparkle_a_lot(index);
                }
                for slot in (0..7).rev() {
                    if self.read(0x11) == 0 && step != 1 && self.read(0x1a) & 1 == 0 {
                        self.write(
                            0x15800 + slot,
                            self.read(0x15800 + slot).wrapping_add(1) & 0x3f,
                        );
                    }
                    let radial =
                        Self::radial_projection(self.read(0x15800 + slot), self.read(0x15808));
                    let x_offset = if radial.r6 != 0 {
                        -i16::from(radial.r4)
                    } else {
                        i16::from(radial.r4)
                    };
                    let y_offset = if radial.r2 != 0 {
                        -i16::from(radial.r0)
                    } else {
                        i16::from(radial.r0)
                    };
                    let x = self
                        .ram
                        .read_u16(0x1580e)
                        .wrapping_add(x_offset as u16)
                        .wrapping_sub(8)
                        .wrapping_sub(self.ram.read_u16(0xe2));
                    let y = self
                        .ram
                        .read_u16(0x15810)
                        .wrapping_add(y_offset as u16)
                        .wrapping_sub(8)
                        .wrapping_sub(self.ram.read_u16(0xe8));
                    let [x_lo, x_hi] = x.to_le_bytes();
                    let [y_lo, y_hi] = y.to_le_bytes();
                    self.write(0x15827 + slot, x_lo);
                    self.write(0x1582f + slot, x_hi);
                    self.write(0x15817 + slot, y_lo);
                    self.write(0x1581f + slot, y_hi);
                    self.gt_cutscene_draw_crystal(index, x, y);
                    index += 1;
                }
            }
        }
        let (x, y) = self.prep_adjusted_oam_coord(k);
        let [x_lo, x_hi] = x.to_le_bytes();
        let [y_lo, y_hi] = y.to_le_bytes();
        self.write(0x15827 + 7, x_lo);
        self.write(0x1582f + 7, x_hi);
        self.write(0x15817 + 7, y_lo);
        self.write(0x1581f + 7, y_hi);
        self.gt_cutscene_draw_crystal(index, x, y);
        if self.step(k) == 0 {
            self.add_occasional_sparkle(k);
        } else if self.read(0x11) == 0 {
            self.gt_cutscene_activate_sparkle();
        }
    }

    fn bomb_get_displacement_from_link(&self, k: usize) -> usize {
        let x = self
            .ram
            .read_u16(0x22)
            .wrapping_add(8)
            .wrapping_sub(self.get_x(k));
        let y = self
            .ram
            .read_u16(0x20)
            .wrapping_add(12)
            .wrapping_sub(self.get_y(k));
        usize::from(
            (x.cast_signed()
                .unsigned_abs()
                .wrapping_add(y.cast_signed().unsigned_abs())
                & 0xfc)
                >> 2,
        )
    }

    fn apply_conveyor(&mut self, k: usize) {
        const X_VEL: [i8; 4] = [0, 0, -8, 8];
        const Y_VEL: [i8; 4] = [-8, 8, 0, 0];
        let belt = usize::from(self.tile_attribute(k) - 0x68);
        self.set_y_velocity(k, Y_VEL[belt] as u8);
        self.set_x_velocity(k, X_VEL[belt] as u8);
        self.move_y(k);
        self.move_x(k);
    }

    fn bomb_check_sprite_and_player_damage(&mut self, k: usize) {
        const SPEED: [u8; 16] = [
            32, 32, 32, 32, 32, 32, 28, 28, 28, 28, 28, 28, 24, 24, 24, 24,
        ];
        const Z_VEL: [u8; 16] = [16, 16, 16, 16, 16, 16, 12, 12, 12, 12, 8, 8, 8, 8, 8, 8];
        const DELAY: [u8; 16] = [
            32, 32, 32, 32, 32, 32, 24, 24, 24, 24, 24, 24, 16, 16, 16, 16,
        ];
        const DAMAGE: [u8; 3] = [8, 4, 2];
        let state = self.item_to_link(k);
        if state == 0 || state >= 9 {
            return;
        }
        self.bomb_check_sprite_damage(k);
        if self.read(0x37b) != 0 {
            if k + 1 == usize::from(self.read(0x2ec)) && self.read(0x308) & 0x80 != 0 {
                self.write(0x308, self.read(0x308) & !0x80);
                self.write(0x50, 0);
            }
            return;
        }
        if self.read(0x4d) != 0 || self.read(0x46) != 0 || self.floor(k) != self.read(0xee) {
            return;
        }
        let link_x = self.ram.read_u16(0x22);
        let link_y = self.ram.read_u16(0x20);
        let bomb_x = self.get_x(k).wrapping_sub(16);
        let bomb_y = self.get_y(k).wrapping_sub(16);
        fn axis(a: u16, a_size: u8, b: u16, b_size: u8) -> bool {
            let [a_low, a_high] = a.to_le_bytes();
            let [b_low, b_high] = b.to_le_bytes();
            if a_high == 0x80 || b_high == 0x80 {
                return false;
            }
            let mut t = i32::from(b_low) - i32::from(a_low);
            let extent = (t + i32::from(b_size)) as u8;
            let page = b_high.wrapping_sub(a_high).wrapping_sub(u8::from(t < 0));
            t = i32::from(page) + (((t & 0xff) + 0x80) >> 8);
            if t & 0xff != 0 {
                return t >= 0x100;
            }
            a_size.wrapping_add(b_size) >= extent
        }
        if !axis(link_x, 16, bomb_x, 32) || !axis(link_y, 24, bomb_y, 32) {
            return;
        }
        let x = self.get_x(k).wrapping_sub(8);
        let y = self.get_y(k).wrapping_sub(12);
        let displacement = self.bomb_get_displacement_from_link(k);
        let speed = self.bomb_project_speed_towards_player(k, x, y, SPEED[displacement]);
        if self.read(0x31f) != 0 || self.read(0xffc) == 2 {
            return;
        }
        self.write(0x28, speed.x);
        self.write(0x27, speed.y);
        self.write(0x29, Z_VEL[displacement]);
        self.write(0x2c7, Z_VEL[displacement]);
        self.write(0x46, DELAY[displacement]);
        self.write(0x4d, 1);
        self.write(0x31f, 58);
        if self.ram.read_u16(0x402) & 0x8000 == 0 {
            self.write(0x373, DAMAGE[usize::from(self.read(0xf35b))]);
        }
    }

    fn latch_altitude_above_link(&mut self, k: usize) {
        self.set_height(k, 17);
        self.set_y(k, self.get_y(k).wrapping_add(17));
        self.set_object_priority(k, 0);
    }

    fn latch_link_coordinates(&mut self, k: usize, phase: usize) {
        const X: [i8; 12] = [8, 8, -4, 20, 8, 8, 8, 8, 8, 8, 8, 8];
        const Y: [i8; 12] = [16, 8, 4, 4, 8, 2, -1, -1, 2, 2, -1, -1];
        let i = phase * 4 + usize::from(self.read(0x2f) >> 1);
        self.set_xy(
            k,
            self.ram.read_u16(0x22).wrapping_add(i16::from(X[i]) as u16),
            self.ram.read_u16(0x20).wrapping_add(i16::from(Y[i]) as u16),
        );
    }

    fn latch_carried_position(&mut self, k: usize) {
        const Y: [i8; 6] = [-2, -1, 0, -2, -1, 0];
        self.write(0x5e, 12);
        self.set_floor(k, self.read(0xee));
        self.set_floor2(k, self.read(0x476));
        let mut z = self.ram.read_u16(0x24);
        if z == 0xffff {
            z = 0;
        }
        self.set_xy(
            k,
            self.ram.read_u16(0x22).wrapping_add(8),
            self.ram
                .read_u16(0x20)
                .wrapping_sub(z)
                .wrapping_add(18)
                .wrapping_add(i16::from(Y[usize::from(self.read(0x2e))]) as u16),
        );
    }

    fn latch_y_coord_to_z(&mut self, k: usize) -> u16 {
        let y = self.get_y(k);
        let z = self.height(k) as i8;
        if self.direction(k) == 1 && z != -1 {
            self.set_y(k, y.wrapping_sub(i16::from(z) as u16));
        }
        y
    }

    fn clear_pickup_item_and_check(&mut self, k: usize) {
        self.write(0x2ec, 0);
        if self.item_to_link(k) != 0 || self.read(0x308) != 0 || self.floor(k) != self.read(0xee) {
            return;
        }
        let Some(collision) = self.check_link_collision(k, 0) else {
            return;
        };
        if collision.r8 >= 16 || collision.r10 >= 12 {
            let facing = if collision.r8 >= collision.r10 {
                if (collision.r4 as i16) < 0 { 1 } else { 0 }
            } else if (collision.r6 as i16) < 0 {
                3
            } else {
                2
            };
            if facing * 2 != usize::from(self.read(0x2f)) {
                return;
            }
        }
        self.write(0x2ec, (k + 1) as u8);
        self.set_state_k(k, 0);
        self.set_aux_timer(k, 16);
        self.set_state_l(k, 0);
        self.set_height(k, 0);
    }

    fn handle_lift_logic(&mut self, k: usize) {
        const DELAY: [u8; 3] = [16, 8, 9];
        if self.state_r(k) != 0 {
            if self.item_to_link(k) != 0 {
                return;
            }
            if self.state_k(k) == 3 {
                self.set_z_velocity(k, self.z_velocity(k).wrapping_sub(2));
                self.move_z(k);
                let z = self.height(k);
                if z != 0 && z < 252 {
                    return;
                }
                self.set_height(k, 0);
                let bounce = self.state_r(k).wrapping_add(1);
                self.set_state_r(k, bounce);
                if bounce != 3 {
                    self.set_z_velocity(k, 24);
                    return;
                }
                self.set_state_k(k, 0);
            }
            self.set_state_r(k, 0);
            self.write(0x5e, 0);
            return;
        }
        if self.state_l(k) == 0 {
            if self.read(0x2ec) == 0 {
                self.clear_pickup_item_and_check(k);
                return;
            }
            if usize::from(self.read(0x2ec)) != k + 1 {
                return;
            }
            if (self.read(0x37b) == 0 && self.read(0x46) != 0)
                || self.read(0x3fd) != 0
                || self.read(0x4d) == 1
            {
                self.set_state_r(k, 1);
                self.set_z_velocity(k, 0);
                self.write(0x2ec, 0);
                self.set_arr4(k, 0);
                self.handle_lift_logic(k);
                return;
            }
            if self.read(0x308) & 0x80 == 0 {
                self.clear_pickup_item_and_check(k);
                return;
            }
            let mut phase = usize::from(self.state_k(k));
            if self.read(0x309) != 2 && self.read(0x2ec) != 0 && phase != 3 {
                if phase == 0 && self.aux_timer(k) == 16 {
                    self.sfx2_pan(k, 0x1d);
                }
                let timer = self.aux_timer(k).wrapping_sub(1);
                self.set_aux_timer(k, timer);
                if (timer as i8) < 0 {
                    phase += 1;
                    self.set_state_k(k, phase as u8);
                    self.set_aux_timer(k, if phase == 3 { 0xfe } else { DELAY[phase] });
                    if phase == 3 {
                        self.latch_altitude_above_link(k);
                        return;
                    }
                }
                self.latch_link_coordinates(k, phase);
                return;
            }
            if phase != 3 {
                return;
            }
            if self.read(0x309) != 2
                && (self.read(0x11) != 0 || (self.read(0xf6) | self.read(0xf4)) & 0x80 == 0)
            {
                if self.item_to_link(k) != 0 {
                    return;
                }
                if self.read(0x5b) >= 2 {
                    self.write(0x5e, 0);
                    if usize::from(self.read(0x2ec)) == k + 1 {
                        self.write(0x2ec, 0);
                        self.clear(k);
                    }
                    return;
                }
                if (self.read(0x345) | self.read(0x2e0)) == 0 {
                    self.latch_carried_position(k);
                    return;
                }
                self.write(0x308, 0);
            }
            const Y_VEL: [i8; 4] = [-32, 32, 0, 0];
            const X_VEL: [i8; 4] = [0, 0, -32, 32];
            let direction = usize::from(self.read(0x2f) >> 1);
            self.set_direction(k, direction as u8);
            self.set_z_velocity(k, 24);
            self.set_y_velocity(k, Y_VEL[direction] as u8);
            self.set_x_velocity(k, X_VEL[direction] as u8);
            self.write(0x309, 2);
            self.set_state_l(k, 1);
            self.write(0x2ec, 0);
            self.set_arr4(k, 0);
            self.set_state_k(k, 0);
            self.set_object_priority(k, 0);
            self.sfx3_pan(k, 0x13);
        }
        if self.item_to_link(k) == 0 {
            self.set_z_velocity(k, self.z_velocity(k).wrapping_sub(2));
            self.move_y(k);
            self.move_x(k);
            let old_z = self.height(k);
            self.move_z(k);
            let z = self.height(k);
            if self.arr4(k) != 0 && self.direction(k) == 1 && (z as i8) >= 0 {
                self.set_y(
                    k,
                    self.get_y(k)
                        .wrapping_add(i16::from(z.wrapping_sub(old_z) as i8) as u16),
                );
            }
            if (z as i8) >= 0 || z == 0xff {
                return;
            }
            self.set_height(k, 0);
            self.sfx2_pan(k, 0x21);
            let bounce = self.state_l(k).wrapping_add(1);
            self.set_state_l(k, bounce);
            if bounce != 3 {
                self.set_y_velocity(k, ((self.y_velocity(k) as i8) / 2) as u8);
                self.set_x_velocity(k, ((self.x_velocity(k) as i8) / 2) as u8);
                self.set_z_velocity(k, 16);
                self.set_arr4(k, 0);
            } else {
                self.set_height(k, 0);
                self.set_state_l(k, 0);
                self.set_arr4(k, 0);
                self.write(0x5e, 0);
                self.set_y_velocity(k, 0);
                self.set_x_velocity(k, 0);
                self.set_z_velocity(k, 0);
                if self.state_t(k) != 0 {
                    self.set_floor(k, self.state_t(k));
                    self.set_state_t(k, 0);
                }
            }
        }
    }

    fn draw_shadow(&mut self, index: usize, kind: usize, mut x: u16, y: u16, palette: u8) {
        const CHAR: [u8; 14] = [
            0x6c, 0x6c, 0x28, 0x28, 0x38, 0xff, 0xc8, 0xc8, 0xd8, 0xd8, 0xd9, 0xd9, 0xda, 0xda,
        ];
        const FLAGS: [u8; 14] = [
            0x28, 0x68, 0x28, 0x68, 0x28, 0xff, 0x22, 0x22, 0x24, 0x64, 0x24, 0x64, 0x24, 0x64,
        ];
        if kind == 2 {
            x = x.wrapping_add(4);
        }
        self.set_oam_safe(
            index,
            x,
            y,
            CHAR[kind * 2],
            (FLAGS[kind * 2] & !0x30) | palette,
            0,
        );
        if CHAR[kind * 2 + 1] != 0xff {
            self.set_oam_safe(
                index + 1,
                x.wrapping_add(8),
                y,
                CHAR[kind * 2 + 1],
                (FLAGS[kind * 2 + 1] & !0x30) | palette,
                0,
            );
        }
    }

    fn bomb_check_underside_sprite_status(
        &mut self,
        k: usize,
        position: &mut (u16, u16),
    ) -> Option<usize> {
        if self.item_to_link(k) != 0 {
            return None;
        }
        let mut shadow = 0;
        if self.tile_attribute(k) == 9 {
            let timer = self.arr22(k).wrapping_sub(1);
            self.set_arr22(k, timer);
            if (timer as i8) < 0 {
                self.set_arr22(k, 3);
                let mut frame = self.arr23(k).wrapping_add(1);
                if frame == 3 {
                    frame = 0;
                }
                self.set_arr23(k, frame);
            }
            shadow = usize::from(self.arr23(k) + 4);
            if matches!(self.read(0x12d) & 0x3f, 0x0b | 0x21) {
                let x = self.get_x(k).wrapping_sub(self.ram.read_u16(0xe2));
                let pan = if x < 0x80 {
                    0x80
                } else if x < 0x100 {
                    0
                } else {
                    0x40
                };
                self.write(0x12d, pan | 0x28);
            }
        } else if self.tile_attribute(k) == 0x40 {
            shadow = 3;
        }
        let z = self.height(k);
        if (2..252).contains(&z) {
            shadow = 2;
        }
        if k + 1 == usize::from(self.read(0x2ec)) && self.read(0x308) & 0x80 != 0 {
            return None;
        }
        position.1 = position
            .1
            .wrapping_add(i16::from(z as i8) as u16)
            .wrapping_add(2);
        position.0 = position.0.wrapping_sub(8);
        Some(shadow)
    }

    pub fn bomb_draw(&mut self, k: usize) {
        const DRAW_FRAME: [u8; 12] = [0, 1, 2, 3, 2, 3, 4, 5, 6, 7, 8, 9];
        const DRAW_COUNT: [u8; 11] = [1, 4, 4, 4, 4, 4, 5, 4, 6, 6, 6];
        let mut position = self.prep_adjusted_oam_coord(k);
        let z = self.height(k) as i8;
        if z != 0 && z != -1 && self.state_k(k) != 3 && self.object_priority(k) != 0 {
            self.ram.write_u16(0x64, 0x3000);
        }
        position.1 = position.1.wrapping_sub(i16::from(z) as u16);
        let state = usize::from(self.item_to_link(k));
        let frame = usize::from(DRAW_FRAME[state]) * 6;
        let mut palette = 2;
        if state == 0 {
            palette = if self.arr3(k) < 0x20 {
                self.arr3(k) & 0x0e
            } else {
                4
            };
            if self.state_l(k) == 0
                && (self.read(0xe20) == 0x92 || k + 1 == usize::from(self.read(0x2ec)))
                && (self.read(0x308) & 0x80 == 0 || self.state_k(k) != 3 && self.read(0x2f) == 0)
            {
                self.runtime.allocate_oam_region(
                    self.ram,
                    if self.read(0xfb3) == 0 { b'B' } else { b'E' },
                    12,
                );
            } else if self.read(0xfb3) != 0
                && self.floor(k) != 0
                && (self.state_l(k) != 0
                    || k + 1 == usize::from(self.read(0x2ec)) && self.read(0x308) & 0x80 != 0)
            {
                oam::OamPointers::for_cursor(oam::OamCursor::from_byte_offset(0xd0))
                    .publish_entry_first(self.ram);
            }
        }
        let original = self.current_oam_index();
        let count = usize::from(DRAW_COUNT[state]);
        let underwater = state == 0 && matches!(self.tile_attribute(k), 9 | 0x40);
        let explosion_index = original + if underwater { 2 } else { 0 };
        self.draw_explosion(explosion_index, frame, 0, count, palette, position);
        let shadow_index = explosion_index + count;
        if let Some(shadow) = self.bomb_check_underside_sprite_status(k, &mut position) {
            let shadow_index = if shadow_index != original + 1 {
                original
            } else {
                shadow_index
            };
            self.draw_shadow(
                shadow_index,
                shadow,
                position.0,
                position.1,
                (self.ram.read_u16(0x64) >> 8) as u8,
            );
        }
    }

    pub fn bomb(&mut self, k: usize) {
        if self.read(0x11) != 0 {
            if matches!(self.read(0x11), 8 | 16) {
                self.handle_lift_logic(k);
            } else if k + 1 == usize::from(self.read(0x2ec)) && self.state_k(k) != 0 {
                if self.state_k(k) != 3 {
                    self.latch_link_coordinates(k, 3);
                    self.latch_altitude_above_link(k);
                    self.set_state_k(k, 3);
                }
                self.latch_carried_position(k);
            }
            self.bomb_draw(k);
            return;
        }
        self.handle_lift_logic(k);
        let mut old_y = self.latch_y_coord_to_z(k);
        let old_direction = self.direction(k);
        let old_priority = self.object_priority(k);
        self.set_object_priority(k, 0);
        let mut collision = self.check_tile_collision_class2(k);
        if self.read(0x1b) != 0 && self.state_l(k) != 0 && self.tile_attribute(k) == 0x1c {
            self.set_state_t(k, 1);
        }
        if collision && (self.read(0x308) & 0x80 == 0 || self.read(0x309) != 0) {
            if old_priority == 0 && self.arr4(k) == 0 {
                self.set_arr4(k, 1);
                let y_speed = if self.direction(k) == 1 { 16 } else { 4 };
                if self.y_velocity(k) != 0 {
                    self.set_y_velocity(
                        k,
                        if (self.y_velocity(k) as i8) < 0 {
                            y_speed
                        } else {
                            (-(y_speed as i8)) as u8
                        },
                    );
                }
                if self.x_velocity(k) != 0 {
                    self.set_x_velocity(
                        k,
                        if (self.x_velocity(k) as i8) < 0 {
                            4
                        } else {
                            (-4_i8) as u8
                        },
                    );
                }
                if self.direction(k) == 1 && self.height(k) != 0 {
                    self.set_y_velocity(k, (-4_i8) as u8);
                    self.set_state_l(k, 2);
                }
            }
        } else if !(k + 1 == usize::from(self.read(0x2ec)) && self.read(0x308) & 0x80 != 0)
            && matches!(self.height(k), 0 | 0xff)
        {
            self.set_direction(k, 16);
            let priority = self.object_priority(k);
            self.check_tile_collision(k);
            self.set_object_priority(k, priority);
            let attribute = self.tile_attribute(k);
            if attribute == 0x26 {
                collision = true;
                if old_priority == 0 && self.arr4(k) == 0 {
                    self.set_arr4(k, 1);
                    let y_speed = if self.direction(k) == 1 { 16 } else { 4 };
                    if self.y_velocity(k) != 0 {
                        self.set_y_velocity(
                            k,
                            if (self.y_velocity(k) as i8) < 0 {
                                y_speed
                            } else {
                                (-(y_speed as i8)) as u8
                            },
                        );
                    }
                    if self.x_velocity(k) != 0 {
                        self.set_x_velocity(
                            k,
                            if (self.x_velocity(k) as i8) < 0 {
                                4
                            } else {
                                (-4_i8) as u8
                            },
                        );
                    }
                }
            } else if matches!(attribute, 0x0c | 0x1c) {
                if self.read(0x46c) != 3 {
                    if self.floor(k) == 0 && !matches!(self.height(k), 0 | 0xff) {
                        self.set_floor(k, 1);
                    }
                } else {
                    old_y = self.get_y(k).wrapping_add(self.ram.read_u16(0x310));
                    self.set_x(k, self.get_x(k).wrapping_add(self.ram.read_u16(0x312)));
                }
            } else if attribute == 0x20
                || attribute & 0xf0 == 0xb0 && !matches!(attribute, 0xb6 | 0xbc)
            {
                if self.read(0x308) & 0x80 == 0 {
                    if k + 1 == usize::from(self.read(0x2ec)) {
                        self.write(0x2ec, 0);
                    }
                    if self.timer(k) == 0 {
                        self.clear(k);
                        return;
                    }
                }
            } else if attribute == 8 {
                if k + 1 == usize::from(self.read(0x2ec)) {
                    self.write(0x2ec, 0);
                }
                if self.timer(k) == 0 {
                    self.set_y(k, self.get_y(k).wrapping_sub(24));
                    self.transmute_to_splash(k);
                    return;
                }
            } else if (0x68..=0x6b).contains(&attribute) {
                self.apply_conveyor(k);
                old_y = self.get_y(k);
            } else {
                self.set_timer(k, if self.state_l(k) != 0 { 0 } else { 2 });
            }
        }
        let _ = collision;
        self.set_y(k, old_y);
        self.set_direction(k, old_direction);
        self.set_object_priority(k, self.object_priority(k) | old_priority);
        self.bomb_check_sprite_and_player_damage(k);
        let timer = self.arr3(k).wrapping_sub(1);
        self.set_arr3(k, timer);
        if timer == 0 {
            let state = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, state);
            if state == 1 {
                self.sfx2_pan(k, 0x0c);
                if k + 1 == usize::from(self.read(0x2ec)) {
                    self.write(0x2ec, 0);
                    if self.read(0x308) & 0x80 != 0 {
                        self.write(0x308, 0);
                        self.write(0x50, 0);
                    }
                }
            }
            if state == 11 {
                self.set_kind(k, if self.step(k) != 0 { 8 } else { 0 });
                return;
            }
            self.set_arr3(k, super::BOMB_FRAME_DELAYS[usize::from(state)]);
        }
        if self.item_to_link(k) == 7 && self.arr3(k) == 2 {
            self.ram.write_u16(0x728 + k * 2, 0);
            self.runtime
                .bomb_check_for_destructibles(self.ram, self.get_x(k), self.get_y(k), k);
            if self.ram.read_u16(0x728 + k * 2) != 0 {
                self.set_step(k, 1);
            }
        }
        self.bomb_draw(k);
    }

    fn draw_explosion(
        &mut self,
        mut index: usize,
        mut frame: usize,
        mut item: usize,
        item_end: usize,
        palette: u8,
        position: (u16, u16),
    ) -> usize {
        const XY: [i8; 108] = [
            -8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -8, -8, -8, 0, 0, -8, 0, 0, 0, 0, 0, 0, -16, -16,
            -16, 0, 0, -16, 0, 0, 0, 0, 0, 0, -16, -16, -16, 0, 0, -16, 0, 0, 0, 0, 0, 0, -8, -8,
            -21, -22, -21, 8, 9, -22, 9, 8, 0, 0, -6, -15, 0, -1, -16, -2, -8, -7, 0, 0, 0, 0, -9,
            -4, -21, -5, -12, -18, -11, 7, 0, -15, 4, -2, -9, -4, -22, -5, -13, -20, -11, 8, 1,
            -16, 5, -2, -20, 4, -12, -19, -9, 16, -5, -2, 2, -9, 10, 6,
        ];
        let (x, y) = position;
        const CHAR_FLAGS: [u8; 108] = [
            0x6e, 0x26, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x8c, 0x22,
            0x8c, 0x62, 0x8c, 0xa2, 0x8c, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x84, 0x22, 0x84, 0x62,
            0x84, 0xa2, 0x84, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x88, 0x22, 0x88, 0x62, 0x88, 0xa2,
            0x88, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x86, 0x22, 0x88, 0x22, 0x88, 0x62, 0x88, 0xa2,
            0x88, 0xe2, 0xff, 0xff, 0x86, 0x22, 0x86, 0x62, 0x86, 0xe2, 0x86, 0xe2, 0xff, 0xff,
            0xff, 0xff, 0x86, 0xe2, 0x86, 0x22, 0x86, 0x22, 0x86, 0x62, 0x86, 0xa2, 0x86, 0xa2,
            0x8a, 0xa2, 0x8a, 0x62, 0x8a, 0x22, 0x8a, 0x62, 0x8a, 0x62, 0x8a, 0xe2, 0x9b, 0x22,
            0x9b, 0xa2, 0x9b, 0x62, 0x9b, 0xe2, 0x9b, 0xa2, 0x9b, 0x22,
        ];
        const EXT: [u8; 54] = [
            2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2,
            1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
        ];
        let base_frame = frame;
        loop {
            if CHAR_FLAGS[frame * 2] != 0xff {
                let i = item + base_frame;
                self.set_oam_safe(
                    index,
                    x.wrapping_add(i16::from(XY[i * 2 + 1]) as u16),
                    y.wrapping_add(i16::from(XY[i * 2]) as u16),
                    CHAR_FLAGS[frame * 2],
                    (CHAR_FLAGS[frame * 2 + 1] & !0x3e)
                        | (self.ram.read_u16(0x64) >> 8) as u8
                        | palette,
                    EXT[frame],
                );
                index += 1;
            }
            frame += 1;
            item += 1;
            if item == item_end {
                break;
            }
        }
        index
    }

    fn add_blast_wall_fireball(&mut self, source_offset: usize) {
        const XY: [i8; 32] = [
            -64, 0, -22, 42, -38, 38, -42, 22, 0, 64, 22, 42, 38, 38, 42, 22, 64, 0, 22, -42, 38,
            -38, 42, -22, 0, -64, -22, -42, -38, -38, -42, -22,
        ];
        for k in (5..=10).rev() {
            if self.kind(k) == 0 {
                self.set_kind(k, 0x32);
                self.set_floor(k, self.read(0xee));
                self.write(0x10040 + k, 16);
                let j = usize::from(self.read(0x1a) & 15);
                self.set_y_velocity(k, XY[j * 2] as u8);
                self.set_x_velocity(k, XY[j * 2 + 1] as u8);
                self.set_xy(
                    k,
                    self.ram
                        .read_u16(0x10030 + source_offset * 2)
                        .wrapping_add(16),
                    self.ram
                        .read_u16(0x10020 + source_offset * 2)
                        .wrapping_add(8),
                );
                return;
            }
        }
    }

    fn draw_blast_wall_blast(&mut self, k: usize, x: u16, y: u16) {
        const DRAW_FRAME: [u8; 12] = [0, 1, 2, 3, 2, 3, 4, 5, 6, 7, 8, 9];
        const DRAW_COUNT: [u8; 11] = [1, 4, 4, 4, 4, 4, 5, 4, 6, 6, 6];
        self.ram.write_u16(0x64, 0x3000);
        self.runtime.allocate_oam_region(
            self.ram,
            if self.read(0xfb3) != 0 { b'D' } else { b'A' },
            0x18,
        );
        let state = usize::from(self.read(0x10000 + k));
        self.draw_explosion(
            self.current_oam_index(),
            usize::from(DRAW_FRAME[state]) * 6,
            0,
            usize::from(DRAW_COUNT[state]),
            0x32,
            (
                x.wrapping_sub(self.ram.read_u16(0xe2)),
                y.wrapping_sub(self.ram.read_u16(0xe8)),
            ),
        );
    }

    pub fn blast_wall_explosion(&mut self, mut k: usize) {
        const SFX_PAN: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
        if self.read(0x11) == 0 {
            if self.read(0x10000 + k) != 0 {
                let delay = self.read(0x10008 + k).wrapping_sub(1);
                self.write(0x10008 + k, delay);
                if delay == 0 {
                    let state = self.read(0x10000 + k).wrapping_add(1);
                    self.write(0x10000 + k, state);
                    if state != 0 && state < 9 {
                        self.add_blast_wall_fireball(k * 4);
                    }
                    if state == 11 {
                        self.write(0x10000 + k, 0);
                        self.write(0x10008 + k, 0);
                    } else {
                        self.write(0x10008 + k, 3);
                    }
                }
            } else {
                k ^= 1;
                if self.read(0x10000 + k) == 6
                    && self.read(0x10008 + k) == 2
                    && self.item_to_link(0).wrapping_add(1) < 7
                {
                    self.set_item_to_link(0, self.item_to_link(0).wrapping_add(1));
                    self.write(0x10000 + k, 1);
                    self.write(0x10008 + k, 3);
                    for i in (0..4).rev() {
                        let vertical = self.read(0x1001c) < 4;
                        let delta = if i & 2 != 0 { -13_i16 } else { 13 };
                        let j = k * 4 + i;
                        let address = if vertical { 0x10030 } else { 0x10020 };
                        self.ram.write_u16(
                            address + j * 2,
                            self.ram
                                .read_u16(address + j * 2)
                                .wrapping_add(delta as u16),
                        );
                        let x = self
                            .ram
                            .read_u16(0x10030 + j * 2)
                            .wrapping_sub(self.ram.read_u16(0xe2));
                        if x < 256 {
                            self.write(0x12e, SFX_PAN[usize::from(x >> 5)] | 0x0c);
                        }
                    }
                }
            }
        }
        let active = usize::from(self.state_k(0));
        if self.read(0x10000 + active) != 0 {
            let mut i = if active == 1 { 7 } else { 3 };
            loop {
                self.draw_blast_wall_blast(
                    active,
                    self.ram.read_u16(0x10030 + i * 2),
                    self.ram.read_u16(0x10020 + i * 2),
                );
                i = i.wrapping_sub(1);
                if i & 3 == 3 {
                    break;
                }
            }
        }
        if self.item_to_link(0) == 6 && self.read(0x10000) == 0 && self.read(0x10001) == 0 {
            self.clear(0);
            self.clear(1);
            self.write(0x112, 0);
        }
    }

    pub fn jump_splash(&mut self, k: usize) {
        const CHAR: [u8; 2] = [0xac, 0xae];
        if self.read(0x11) == 0 {
            let delay = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, delay);
            if (delay as i8) < 0 {
                self.set_aux_timer(k, 0);
                self.set_item_to_link(k, 1);
            }
            if self.item_to_link(k) != 0 {
                let velocity = self.y_velocity(k).wrapping_sub(4);
                self.set_y_velocity(k, velocity);
                self.set_x_velocity(k, velocity);
                if velocity < 232 {
                    self.clear(k);
                    if (self.read(0x2e0) != 0 || self.read(0x5d) == 4) && self.read(0x345) != 0 {
                        self.runtime.check_ability_to_swim(self.ram);
                    }
                    return;
                }
                self.move_x(k);
                self.move_y(k);
            }
        }
        let (mut x, y) = self.prep_oam_coord(k);
        let reflected_x = self
            .ram
            .read_u16(0x22)
            .wrapping_mul(2)
            .wrapping_sub(self.get_x(k))
            .wrapping_sub(self.ram.read_u16(0xe2));
        let third_x = self
            .get_x(k)
            .wrapping_add(12)
            .wrapping_sub(self.ram.read_u16(0xe2));
        let frame = usize::from(self.item_to_link(k));
        let mut flags = 0;
        let mut index = self.current_oam_index();
        for _ in 0..2 {
            self.set_oam(index, x, y, CHAR[frame], 0x24 | flags, 2);
            index = self.allocate_oam_from_custom_region(index + 1);
            x = reflected_x;
            flags = 0x40;
        }
        self.set_oam(
            index,
            third_x,
            y,
            0xc0,
            0x24,
            if frame == 1 { 1 } else { 2 },
        );
    }

    /// Compatibility entry point for `HitStars_UpdateOamBufferPosition`.
    pub fn hit_stars_update_oam_buffer_position(&mut self, index: usize) -> usize {
        oam::EffectOamLayout::from_sort_setting(self.read(0xfb3))
            .wrap_hit_stars_cursor(self.ram, oam::OamCursor::from_index(index))
            .index()
    }

    pub fn hit_stars(&mut self, k: usize) {
        const CHAR: [u8; 2] = [0x90, 0x91];
        let divider = self.arr3(k).wrapping_sub(1);
        self.set_arr3(k, divider);
        if (divider as i8) >= 0 {
            return;
        }
        self.set_arr3(k, 0);
        if self.read(0x11) == 0 {
            let delay = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, delay);
            if (delay as i8) < 0 {
                self.set_aux_timer(k, 0);
                self.set_item_to_link(k, 1);
            }
            if self.item_to_link(k) != 0 {
                let velocity = self.y_velocity(k).wrapping_sub(4);
                self.set_y_velocity(k, velocity);
                self.set_x_velocity(k, velocity);
                if velocity < 232 {
                    self.clear(k);
                    return;
                }
                self.move_y(k);
                self.move_x(k);
            }
        }
        let (mut x, y) = self.prep_oam_coord(k);
        let anchor = u16::from_le_bytes([self.state_a(k), self.state_b(k)]);
        let reflected_x = anchor
            .wrapping_mul(2)
            .wrapping_sub(self.get_x(k))
            .wrapping_sub(8)
            .wrapping_sub(self.ram.read_u16(0xe2));
        if self.step(k) == 2 {
            self.runtime.allocate_oam_region(
                self.ram,
                if self.read(0xfb3) == 0 { b'B' } else { b'E' },
                8,
            );
        }
        let mut index = self.current_oam_index();
        let mut flags = 0;
        for _ in (0..2).rev() {
            self.set_oam(
                index,
                x,
                y,
                CHAR[usize::from(self.item_to_link(k))],
                (self.ram.read_u16(0x64) >> 8) as u8 | 4 | flags,
                0,
            );
            flags = 0x40;
            x = (x & 0xff00) | (reflected_x & 0x00ff);
            index = self.hit_stars_update_oam_buffer_position(index + 1);
        }
    }

    pub fn shovel_dirt(&mut self, k: usize) {
        const XY: [i8; 8] = [18, -13, -9, 4, 18, 13, -9, -11];
        const CHAR: [u8; 2] = [0x40, 0x50];
        let (mut x, mut y) = self.prep_oam_coord(k);
        if self.timer(k) == 0 {
            self.set_timer(k, 8);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 2 {
                self.clear(k);
                return;
            }
        }
        let frame = usize::from(self.item_to_link(k));
        let direction = frame + if self.read(0x2f) == 4 { 0 } else { 2 };
        x = x.wrapping_add(i16::from(XY[direction * 2 + 1]) as u16);
        y = y.wrapping_add(i16::from(XY[direction * 2]) as u16);
        let mut index = self.current_oam_index();
        for i in 0..2 {
            self.set_oam(
                index,
                x.wrapping_add(i * 8),
                y,
                CHAR[frame].wrapping_add(i as u8),
                4 | (self.ram.read_u16(0x64) >> 8) as u8,
                0,
            );
            index = self.allocate_oam_from_custom_region(index + 1);
        }
    }

    pub fn blast_wall_fireball(&mut self, k: usize) {
        const CHAR: [u8; 3] = [0x9d, 0x9c, 0x8d];
        if self.read(0x11) == 0 {
            let frame = self.item_to_link(k).wrapping_add(2);
            self.set_item_to_link(k, frame);
            self.set_y_velocity(k, self.y_velocity(k).wrapping_add(frame));
            self.move_y(k);
            self.move_x(k);
            let timer = self.read(0x10040 + k).wrapping_sub(1);
            self.write(0x10040 + k, timer);
            if (timer as i8) < 0 {
                self.clear(k);
                return;
            }
        }
        self.runtime.allocate_oam_region(
            self.ram,
            if self.read(0xfb3) != 0 { b'D' } else { b'A' },
            4,
        );
        let (x, y) = self.prep_oam_coord(k);
        let timer = self.read(0x10040 + k);
        let frame = if timer & 8 != 0 {
            0
        } else if timer & 4 != 0 {
            1
        } else {
            2
        };
        self.set_oam(self.current_oam_index(), x, y, CHAR[frame], 0x22, 0);
    }

    fn radial_projection(angle: u8, radius: u8) -> super::AncillaRadialProjection {
        const COSINE: [u8; 64] = [
            255, 254, 251, 244, 236, 225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49,
            74, 97, 120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244, 236,
            225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97, 120, 142, 162,
            181, 197, 212, 225, 236, 244, 251, 254,
        ];
        const SINE: [u8; 64] = [
            0, 25, 49, 74, 97, 120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254,
            251, 244, 236, 225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97,
            120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244, 236, 225,
            212, 197, 181, 162, 142, 120, 97, 74, 49, 25,
        ];
        let i = usize::from(angle);
        let cosine = u16::from(COSINE[i]) * u16::from(radius);
        let sine = u16::from(SINE[i]) * u16::from(radius);
        super::AncillaRadialProjection {
            r0: ((cosine >> 8) + ((cosine >> 7) & 1)) as u8,
            r2: u8::from(!(16..48).contains(&i)),
            r4: ((sine >> 8) + ((sine >> 7) & 1)) as u8,
            r6: u8::from(i >= 33),
        }
    }

    fn draw_ether_orb(&mut self, k: usize, mut index: usize) {
        const CHAR: [u8; 8] = [0x48, 0x48, 0x4a, 0x4a, 0x4c, 0x4c, 0x4e, 0x4e];
        const FLAGS: [u8; 8] = [0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c];
        let mut y = self
            .ram
            .read_u16(0x15813)
            .wrapping_sub(1)
            .wrapping_sub(self.ram.read_u16(0xe8));
        let mut x = self
            .ram
            .read_u16(0x15815)
            .wrapping_sub(8)
            .wrapping_sub(self.ram.read_u16(0xe2));
        let frame = usize::from(self.item_to_link(k)) * 4;
        for i in 0..4 {
            self.set_oam(index, x, y, CHAR[frame + i], FLAGS[frame + i], 2);
            index = self.allocate_oam_from_custom_region(index + 1);
            x = x.wrapping_add(16);
            if i == 1 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(16);
            }
        }
    }

    fn draw_ether_blitz(&mut self, k: usize) {
        const SEGMENT_CHAR: [u8; 4] = [0x40, 0x42, 0x44, 0x46];
        let (x, mut y) = self.prep_oam_coord(k);
        let mut index = self.current_oam_index();
        let frame = usize::from(self.item_to_link(k));
        let mut alternating = 0;
        let mut remaining = self.arr25(k);
        loop {
            self.set_oam(
                index,
                x,
                y,
                SEGMENT_CHAR[frame * 2 + alternating],
                0x3c | (self.ram.read_u16(0x64) >> 8) as u8,
                2,
            );
            y = y.wrapping_sub(16);
            index += 1;
            alternating ^= 1;
            if remaining == 0 {
                break;
            }
            remaining -= 1;
        }
        if self.step(k) == 1 {
            self.draw_ether_orb(k, index);
        }
    }

    fn draw_ether_blitz_ball(
        &mut self,
        index: usize,
        radial: super::AncillaRadialProjection,
        frame: usize,
    ) -> usize {
        const CHAR: [u8; 2] = [0x68, 0x6a];
        let x_offset = if radial.r6 != 0 {
            -i16::from(radial.r4)
        } else {
            i16::from(radial.r4)
        };
        let y_offset = if radial.r2 != 0 {
            -i16::from(radial.r0)
        } else {
            i16::from(radial.r0)
        };
        let x = self
            .ram
            .read_u16(0x1580e)
            .wrapping_add(x_offset as u16)
            .wrapping_sub(8)
            .wrapping_sub(self.ram.read_u16(0xe2));
        let y = self
            .ram
            .read_u16(0x15810)
            .wrapping_add(y_offset as u16)
            .wrapping_sub(8)
            .wrapping_sub(self.ram.read_u16(0xe8));
        self.set_oam(index, x, y, CHAR[frame], 0x3c, 2);
        self.allocate_oam_from_custom_region(index + 1)
    }

    fn draw_ether_blitz_segment(
        &mut self,
        index: usize,
        radial: super::AncillaRadialProjection,
        frame: usize,
        spoke: usize,
    ) -> usize {
        const X: [i8; 16] = [
            -8, -16, -24, -16, -8, 0, 8, -16, -8, -16, -24, -16, -8, 0, 8, 0,
        ];
        const Y: [i8; 16] = [
            8, 0, -8, -16, -24, -16, -8, -16, 8, 0, -8, -16, -24, -16, -8, 0,
        ];
        const CHAR: [u8; 32] = [
            0x40, 0x42, 0x66, 0x64, 0x62, 0x60, 0x64, 0x66, 0x42, 0x40, 0x66, 0x64, 0x60, 0x62,
            0x64, 0x66, 0x68, 0x42, 0x68, 0x64, 0x68, 0x60, 0x68, 0x64, 0x68, 0x40, 0x68, 0x66,
            0x68, 0x62, 0x68, 0x64,
        ];
        const FLAGS: [u8; 32] = [
            0x3c, 0x3c, 0xfc, 0xfc, 0x3c, 0x3c, 0xbc, 0xbc, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c,
            0x7c, 0x7c, 0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0xbc, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0xfc,
            0x3c, 0xbc, 0x3c, 0xbc,
        ];
        let x_offset = if radial.r6 != 0 {
            -i16::from(radial.r4)
        } else {
            i16::from(radial.r4)
        };
        let y_offset = if radial.r2 != 0 {
            -i16::from(radial.r0)
        } else {
            i16::from(radial.r0)
        };
        let base_x = self
            .ram
            .read_u16(0x1580e)
            .wrapping_add(x_offset as u16)
            .wrapping_sub(self.ram.read_u16(0xe2));
        let base_y = self
            .ram
            .read_u16(0x15810)
            .wrapping_add(y_offset as u16)
            .wrapping_sub(self.ram.read_u16(0xe8));
        let t = frame * 8 + spoke;
        self.set_oam(
            index,
            base_x.wrapping_sub(8),
            base_y.wrapping_sub(8),
            CHAR[t * 2],
            FLAGS[t * 2],
            2,
        );
        self.set_oam(
            index + 1,
            base_x.wrapping_add(i16::from(X[t]) as u16),
            base_y.wrapping_add(i16::from(Y[t]) as u16),
            CHAR[t * 2 + 1],
            FLAGS[t * 2 + 1],
            2,
        );
        self.allocate_oam_from_custom_region(index + 2)
    }

    fn check_for_entrance_trigger(&self, kind: usize) -> bool {
        const BASE_Y: [u16; 4] = [0xd40, 0x210, 0xcfc, 0x100];
        const BASE_X: [u16; 4] = [0xd80, 0xe68, 0x130, 0xf10];
        const SIZE_Y: [u16; 4] = [11, 32, 16, 12];
        const SIZE_X: [u16; 4] = [16, 16, 16, 16];
        self.ram
            .read_u16(0x20)
            .wrapping_add(12)
            .wrapping_sub(BASE_Y[kind])
            .cast_signed()
            .unsigned_abs()
            < SIZE_Y[kind]
            && self
                .ram
                .read_u16(0x22)
                .wrapping_add(8)
                .wrapping_sub(BASE_X[kind])
                .cast_signed()
                .unsigned_abs()
                < SIZE_X[kind]
    }

    fn ether_radial_spin(&mut self, k: usize) {
        if self.step(k) == 4 {
            match self.read(0x1a) & 7 {
                0 => self.write(0x12f, 0x2a),
                4 => self.write(0x12f, 0xaa),
                7 => self.write(0x12f, 0x6a),
                _ => {}
            }
        } else {
            self.set_x_low(k, self.read(0x15808));
            self.set_x_high(k, 0);
            self.move_x(k);
            self.write(0x15808, self.x_low(k));
            if self.read(0x15808) == 0x40 {
                self.set_step(k, 4);
            }
        }
        let step = self.step(k);
        let frame = usize::from(self.item_to_link(k));
        let mut index = self.current_oam_index();
        for spoke in (0..8).rev() {
            if step != 2 && step != 5 {
                self.write(
                    0x15800 + spoke,
                    self.read(0x15800 + spoke).wrapping_add(1) & 0x3f,
                );
            }
            let radial = Self::radial_projection(self.read(0x15800 + spoke), self.read(0x15808));
            index = if step != 2 {
                self.draw_ether_blitz_ball(index, radial, frame)
            } else {
                self.draw_ether_blitz_segment(index, radial, frame, spoke)
            };
        }
        if self.read(0x15808) < 0xf0
            && (0..8).any(|i| oam::read(self.ram, self.current_oam_index() + i).y != 0xf0)
        {
            return;
        }
        self.clear(k);
        self.write(0xaaa, 1);
        self.write(0x324, 0);
        self.write(0x31c, 0);
        self.write(0x31d, 0);
        self.write(0x50, 0);
        self.write(0xfc1, 0);
        if self.read(0x8a) == 0x70
            && self.read(0xf280 + 0x70) & 0x20 == 0
            && self.check_for_entrance_trigger(2)
        {
            self.write(0x4c6, 3);
            self.write(0xb0, 0);
            self.write(0xc8, 0);
        }
        if self.read(0x5d) != 25 {
            self.write(0x5d, 0);
            self.write(0x3d, 0);
            self.write(
                0x3a,
                if self.read(0x3c) != 0 {
                    self.read(0xf0) & 0x80
                } else {
                    0
                },
            );
        }
        self.write(0x5e, 0);
        self.write(0x325, 0);
        self.runtime.load_actual_gear_palettes(self.ram);
        self.runtime.palette_restore_bg_and_hud(self.ram);
    }

    fn ether_orb_pulse(&mut self, k: usize) {
        if (self.arr25(k) as i8) >= 0 {
            let timer = self.arr3(k).wrapping_sub(1);
            self.set_arr3(k, timer);
            if (timer as i8) >= 0 {
                self.draw_ether_blitz(k);
                return;
            }
            self.set_arr3(k, 3);
            let segments = self.arr25(k).wrapping_sub(1);
            self.set_arr25(k, segments);
            if (segments as i8) >= 0 {
                self.draw_ether_blitz(k);
                return;
            }
            self.set_arr3(k, 9);
        }
        let timer = self.arr3(k).wrapping_sub(1);
        self.set_arr3(k, timer);
        if (timer as i8) < 0 {
            self.set_step(k, 2);
            self.set_y_velocity(k, 0);
            self.set_x_velocity(k, 16);
            self.set_item_to_link(k, 0);
            self.set_aux_timer(k, 2);
            if self.read(0x31d) != 0 {
                self.medallion_check_sprite_damage(k);
            }
        }
        self.draw_ether_orb(k, self.current_oam_index());
    }

    fn ether_lightning_stroke(&mut self, k: usize) {
        self.move_y(k);
        let y = self.get_y(k);
        if self.read(0x1580c) != (y as u8 & 0xf0) {
            self.write(0x1580c, y as u8 & 0xf0);
            self.set_arr25(k, self.arr25(k).wrapping_add(1));
        }
        if y < 0xe000 && self.ram.read_u16(0x1580a) < 0xe000 && self.ram.read_u16(0x1580a) <= y {
            self.set_step(k, 1);
        }
        self.draw_ether_blitz(k);
    }

    pub fn ether_spell(&mut self, k: usize) {
        if self.read(0x11) != 0 {
            return;
        }
        if self.step(k) != 0 {
            let flash = if self.read(0x31d) == 0 {
                let value = self.arr4(k).wrapping_add(1);
                self.set_arr4(k, value);
                value & 4 == 0
            } else {
                self.read(0x31d) == 11
            };
            if flash {
                self.runtime.palette_electro_themed_gear(self.ram);
                self.runtime.filter_majorly_whiten_bg(self.ram);
            } else {
                self.runtime.load_actual_gear_palettes(self.ram);
                self.runtime.palette_restore_bg_from_flash(self.ram);
            }
        }
        if self.step(k) == 2 {
            let timer = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, timer);
            if (timer as i8) < 0 {
                self.set_aux_timer(k, 2);
                let frame = self.item_to_link(k).wrapping_add(1);
                self.set_item_to_link(k, frame);
                if frame == 2 {
                    self.set_item_to_link(k, frame - 1);
                    self.set_x_velocity(k, 16);
                    self.set_step(k, 3);
                }
            }
            self.set_x_velocity(k, self.x_velocity(k).wrapping_add(1));
            self.ether_radial_spin(k);
            return;
        }
        let timer = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, timer);
        if (timer as i8) < 0 {
            self.set_aux_timer(k, 2);
            self.set_item_to_link(k, self.item_to_link(k) ^ 1);
        }
        match self.step(k) {
            0 => self.ether_lightning_stroke(k),
            1 => self.ether_orb_pulse(k),
            3 => self.ether_radial_spin(k),
            4 => {
                let duration = self.read(0x15812).wrapping_sub(1);
                self.write(0x15812, duration);
                if duration == 0 {
                    self.set_step(k, 5);
                }
                self.ether_radial_spin(k);
            }
            _ => {
                let velocity = self.x_velocity(k).wrapping_add(0x10);
                self.set_x_velocity(k, if (velocity as i8) < 0 { 0x7f } else { velocity });
                self.ether_radial_spin(k);
            }
        }
    }

    fn allocate_oam_from_region_a_d_f(&mut self, k: usize, size: u8) -> usize {
        let region = if self.read(0xfb3) != 0 {
            if self.floor(k) != 0 { b'F' } else { b'D' }
        } else {
            b'A'
        };
        self.runtime.allocate_oam_region(self.ram, region, size);
        self.current_oam_index()
    }

    pub fn add_bombos_spell(&mut self, kind: u8, limit: u8) -> Option<usize> {
        let k = self.add_ancilla(kind, limit)?;
        for i in 0..10 {
            self.write(0x15810 + i, 0);
            self.write(0x15800 + i, 3);
        }
        for i in 0..8 {
            self.write(0x15935 + i, 0);
            self.write(0x15945 + i, 3);
        }
        self.write(0x15934, 0);
        self.write(0x15a56, 0);
        self.write(0x15a55, 0x80);
        self.write(0x15820, 0x10);
        self.write(0xaaa, 11);
        self.write(0x112, 1);
        self.set_step(k, 0);
        self.set_item_to_link(k, 0);
        misc::ancilla_sfx2_near(self.ram, 0x2a);
        let mut generated = self.runtime.generated_bombos_position(self.read(0x1a));
        if generated >= 0xe0 {
            generated &= 0x7f;
        }
        self.ram.write_u16(
            0x159d5,
            (self.ram.read_u16(0x22) & 0xff00) | u16::from(generated),
        );
        self.ram.write_u16(
            0x15955,
            (self.ram.read_u16(0x20) & 0xff00) | u16::from(generated),
        );
        self.ram
            .write_u16(0x1592c, self.ram.read_u16(0x22).wrapping_sub(16));
        self.ram
            .write_u16(0x15924, self.ram.read_u16(0x20).wrapping_add(16));
        self.write(0x15a57, 16);
        let radial = Self::radial_projection(self.read(0x15820), 16);
        let x_offset = if radial.r6 != 0 {
            -i16::from(radial.r4)
        } else {
            i16::from(radial.r4)
        };
        let y_offset = if radial.r2 != 0 {
            -i16::from(radial.r0)
        } else {
            i16::from(radial.r0)
        };
        let x = self.ram.read_u16(0x1592c).wrapping_add(x_offset as u16);
        let y = self.ram.read_u16(0x15924).wrapping_add(y_offset as u16);
        self.write(0x158a4, x as u8);
        self.write(0x158e4, (x >> 8) as u8);
        self.write(0x15824, y as u8);
        self.write(0x15864, (y >> 8) as u8);
        Some(k)
    }

    fn draw_bombos_fire_column(&mut self, slot: usize) {
        const X: [i8; 39] = [
            0, -1, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, -1, 1, -1, -1, 2, -1, -1,
        ];
        const Y: [i8; 39] = [
            0, -1, -1, 0, -4, -1, 0, -8, -1, 0, -12, -1, 0, -16, -1, 0, -4, -20, 0, -8, -24, 0,
            -12, -28, 0, -16, -32, 0, -16, -32, -18, -34, -1, -35, -1, -1, -36, -1, -1,
        ];
        const FLAGS: [u8; 39] = [
            0x3c, 0xff, 0xff, 0x3c, 0x3c, 0xff, 0x3c, 0x3c, 0xff, 0x7c, 0x7c, 0xff, 0x3c, 0x7c,
            0xff, 0x3c, 0x3c, 0x3c, 0xbc, 0x3c, 0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0x3c, 0x7c, 0x3c,
            0x3c, 0x3c, 0x3c, 0x3c, 0xff, 0x3c, 0xff, 0xff, 0x3c, 0xff, 0xff,
        ];
        const CHAR: [u8; 39] = [
            0x40, 0xff, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44,
            0xff, 0x40, 0x46, 0x44, 0x4a, 0x4a, 0x48, 0x4c, 0x4c, 0x4a, 0x4e, 0x4c, 0x4a, 0x4e,
            0x6a, 0x4c, 0x4e, 0x68, 0xff, 0x6a, 0xff, 0xff, 0x4e, 0xff, 0xff,
        ];
        self.allocate_oam_from_region_a_d_f(slot, 0x10);
        let state = self.read(0x15810 + slot);
        if state == 13 {
            return;
        }
        let mut frame = usize::from(state) * 3 + 2;
        let mut index = self.current_oam_index();
        for _ in 0..3 {
            if CHAR[frame] != 0xff {
                let x = u16::from_le_bytes([self.read(0x158a4 + slot), self.read(0x158e4 + slot)])
                    .wrapping_add(i16::from(X[frame]) as u16)
                    .wrapping_sub(self.ram.read_u16(0xe2));
                let y = u16::from_le_bytes([self.read(0x15824 + slot), self.read(0x15864 + slot)])
                    .wrapping_add(i16::from(Y[frame]) as u16)
                    .wrapping_sub(self.ram.read_u16(0xe8));
                self.set_oam(index, x, y, CHAR[frame], FLAGS[frame], 2);
                index += 1;
            }
            index = self.allocate_oam_from_custom_region(index);
            frame -= 1;
        }
    }

    fn control_bombos_fire_columns(&mut self, k: usize) {
        const SFX_PAN: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
        let suppress_spawn = self.item_to_link(k) != 0;
        let mut last = usize::from(self.step(k));
        let mut slot = last;
        loop {
            let mut skip_draw = self.read(0x15810 + slot) == 13;
            if !skip_draw {
                let delay = self.read(0x15800 + slot).wrapping_sub(1);
                self.write(0x15800 + slot, delay);
                if (delay as i8) < 0 {
                    self.write(0x15800 + slot, 3);
                    let state = self.read(0x15810 + slot).wrapping_add(1);
                    self.write(0x15810 + slot, state);
                    if state == 13 || state == 2 && suppress_spawn {
                        skip_draw = true;
                    } else if state == 2 {
                        let new_slot = if last == 9 {
                            let found = (0..=9).rev().find(|&i| self.read(0x15810 + i) == 13);
                            if let Some(found) = found {
                                self.write(0x15810 + found, 0);
                                found
                            } else {
                                9
                            }
                        } else {
                            last += 1;
                            last
                        };
                        let radius = self.read(0x15a57).saturating_add(3).min(207);
                        self.write(0x15a57, radius);
                        self.write(0x15820, self.read(0x15820).wrapping_add(6));
                        let radial = Self::radial_projection(self.read(0x15820) & 0x3f, radius);
                        let x_offset = if radial.r6 != 0 {
                            -i16::from(radial.r4)
                        } else {
                            i16::from(radial.r4)
                        };
                        let y_offset = if radial.r2 != 0 {
                            -i16::from(radial.r0)
                        } else {
                            i16::from(radial.r0)
                        };
                        let x = self.ram.read_u16(0x1592c).wrapping_add(x_offset as u16);
                        let y = self.ram.read_u16(0x15924).wrapping_add(y_offset as u16);
                        self.write(0x158a4 + new_slot, x as u8);
                        self.write(0x158e4 + new_slot, (x >> 8) as u8);
                        self.write(0x15824 + new_slot, y as u8);
                        self.write(0x15864 + new_slot, (y >> 8) as u8);
                        let screen_x = x.wrapping_sub(self.ram.read_u16(0xe2)).wrapping_add(8);
                        if screen_x < 256 {
                            self.write(0x12e, SFX_PAN[usize::from(screen_x >> 5)] | 0x2a);
                        }
                    }
                }
                if !skip_draw {
                    self.draw_bombos_fire_column(slot);
                }
            }
            if slot == 0 {
                break;
            }
            slot -= 1;
        }
        if self.read(0x15820) >= 0x80 {
            self.write(0x15934, 1);
        }
        self.set_step(k, last as u8);
    }

    fn finish_bombos_fire_columns(&mut self, k: usize) {
        let mut slot = usize::from(self.step(k));
        loop {
            let delay = self.read(0x15800 + slot).wrapping_sub(1);
            self.write(0x15800 + slot, delay);
            if (delay as i8) < 0 {
                self.write(0x15800 + slot, 3);
                self.write(
                    0x15810 + slot,
                    self.read(0x15810 + slot).wrapping_add(1).min(13),
                );
            }
            self.draw_bombos_fire_column(slot);
            if slot == 0 {
                break;
            }
            slot -= 1;
        }
        if (0..10).any(|i| self.read(0x15810 + i) != 13) {
            return;
        }
        self.write(0x15934, 2);
        self.medallion_check_sprite_damage(k);
        self.set_step(k, 0);
    }

    fn draw_bombos_blast(&mut self, k: usize) {
        const X: [i8; 32] = [
            -8, -1, -1, -1, -12, -4, -12, -4, -16, 0, -16, 0, -16, 0, -16, 0, -17, 1, -17, 1, -19,
            3, -19, 3, -19, 3, -19, 3, -19, 3, -19, 3,
        ];
        const Y: [i8; 32] = [
            -8, -1, -1, -1, -12, -12, -4, -4, -16, -16, 0, 0, -16, -16, 0, 0, -17, -17, 1, 1, -19,
            -19, 3, 3, -19, -19, 3, 3, -19, -19, 3, 3,
        ];
        const FLAGS: [u8; 32] = [
            0x3c, 0xff, 0xff, 0xff, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c,
            0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc,
            0x3c, 0x7c, 0xbc, 0xfc,
        ];
        const CHAR: [u8; 32] = [
            0x60, 0xff, 0xff, 0xff, 0x62, 0x62, 0x62, 0x62, 0x64, 0x64, 0x64, 0x64, 0x66, 0x66,
            0x66, 0x66, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x6a, 0x6a, 0x6a, 0x6a,
            0x4e, 0x4e, 0x4e, 0x4e,
        ];
        let state = self.read(0x15935 + k);
        if state == 8 {
            return;
        }
        self.allocate_oam_from_region_a_d_f(k, 0x10);
        let x0 = self.ram.read_u16(0x159d5 + k * 2);
        let y0 = self.ram.read_u16(0x15955 + k * 2);
        let mut frame = usize::from(state) * 4 + 3;
        let mut index = self.current_oam_index();
        for _ in 0..4 {
            if CHAR[frame] != 0xff {
                self.set_oam(
                    index,
                    x0.wrapping_add(i16::from(X[frame]) as u16)
                        .wrapping_sub(self.ram.read_u16(0xe2)),
                    y0.wrapping_add(i16::from(Y[frame]) as u16)
                        .wrapping_sub(self.ram.read_u16(0xe8)),
                    CHAR[frame],
                    FLAGS[frame],
                    2,
                );
                index += 1;
            }
            index = self.allocate_oam_from_custom_region(index);
            frame -= 1;
        }
    }

    fn control_bombos_blasting(&mut self, k: usize) {
        const RANDOM: [u8; 72] = [
            0xb6, 0x5d, 0xa1, 0x30, 0x69, 0xb5, 0xa3, 0x24, 0x96, 0xac, 0x73, 0x5f, 0x92, 0x48,
            0x52, 0x81, 0x39, 0x95, 0x7f, 0x20, 0x88, 0x5d, 0x34, 0x98, 0xbc, 0xd2, 0x51, 0x77,
            0xa2, 0x47, 0x94, 0xb2, 0x34, 0xda, 0x30, 0x62, 0x9f, 0x76, 0x51, 0x46, 0x98, 0x5c,
            0x9b, 0x61, 0x58, 0x95, 0x4c, 0xba, 0x7e, 0xcb, 0x12, 0xd0, 0x70, 0xa6, 0x46, 0xbf,
            0x40, 0x50, 0x7e, 0x8c, 0x2d, 0x61, 0xac, 0x88, 0x20, 0x6a, 0x72, 0x5f, 0xd2, 0x28,
            0x52, 0x80,
        ];
        const SFX_PAN: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
        let mut last = usize::from(self.step(k));
        for slot in (0..=last).rev() {
            if self.read(0x15935 + slot) != 8 {
                let delay = self.read(0x15945 + slot).wrapping_sub(1);
                self.write(0x15945 + slot, delay);
                if (delay as i8) < 0 {
                    self.write(0x15945 + slot, 3);
                    let state = self.read(0x15935 + slot).wrapping_add(1);
                    self.write(0x15935 + slot, state);
                    if state == 1 && self.read(0x15a56) == 0 {
                        let new_slot = if last != 15 {
                            last += 1;
                            last
                        } else {
                            (0..=15)
                                .rev()
                                .find(|&i| self.read(0x15935 + i) == 8)
                                .unwrap_or(0)
                        };
                        self.write(0x15935 + new_slot, 0);
                        self.write(0x15945 + new_slot, 3);
                        let frame = usize::from(self.read(0x1a) & 0x3f);
                        self.ram.write_u16(
                            0x15955 + new_slot * 2,
                            u16::from(RANDOM[frame]).wrapping_add(self.ram.read_u16(0xe8)),
                        );
                        self.ram.write_u16(
                            0x159d5 + new_slot * 2,
                            u16::from(RANDOM[frame + 3]).wrapping_add(self.ram.read_u16(0xe2)),
                        );
                        self.write(
                            0x12e,
                            0x0c | SFX_PAN
                                [usize::from(self.ram.read_u16(0x159d5 + new_slot * 2) >> 5 & 7)],
                        );
                    }
                }
            }
            self.draw_bombos_blast(slot);
        }
        if (0..16).any(|slot| self.read(0x15935 + slot) != 8) {
            self.set_step(k, last as u8);
        } else {
            self.clear(k);
            self.write(0xaaa, 1);
            self.write(0x324, 0);
            self.write(0x31c, 0);
            self.write(0x31d, 0);
            self.write(0x50, 0);
            self.write(0xfc1, 0);
            if self.read(0x5d) != 26 {
                self.write(0x5d, 0);
                self.write(0x3d, 0);
                self.write(
                    0x3a,
                    if self.read(0x3c) != 0 {
                        self.read(0xf0) & 0x80
                    } else {
                        0
                    },
                );
            }
            self.write(0x5e, 0);
            self.write(0x325, 0);
        }
        let timer = self.read(0x15a55).wrapping_sub(1);
        self.write(0x15a55, timer);
        if timer == 0 {
            self.write(0x15a55, 1);
            self.write(0x15a56, 1);
        }
    }

    pub fn bombos_spell(&mut self, k: usize) {
        match self.read(0x15934) {
            0 => {
                if self.read(0x11) == 0 {
                    self.control_bombos_fire_columns(k);
                } else {
                    for slot in (0..10).rev() {
                        self.draw_bombos_fire_column(slot);
                    }
                }
            }
            2 => {
                if self.read(0x11) == 0 {
                    self.control_bombos_blasting(k);
                } else {
                    let mut slot = usize::from(self.step(k));
                    loop {
                        self.draw_bombos_blast(slot);
                        if slot == 0 {
                            break;
                        }
                        slot -= 1;
                    }
                }
            }
            _ => {
                if self.read(0x11) == 0 {
                    self.finish_bombos_fire_columns(k);
                } else {
                    for slot in (0..10).rev() {
                        self.draw_bombos_fire_column(slot);
                    }
                }
            }
        }
    }

    /// Compatibility entry point for `AncillaDraw_QuakeInitialBolts`.
    pub fn draw_quake_initial_bolts(&mut self, k: usize) {
        const OFFSET: [usize; 5] = [0, 0x18, 0, 0x18, 0x2f];
        const CHAR: [u8; 15] = [
            0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 0x60, 0x62, 0x64, 0x66, 0x68, 0x6a,
            0x63,
        ];
        let table = usize::from(self.read(0x15805 + k)) + OFFSET[k];
        let start = usize::from(QUAKE_ITEM_POS[table]);
        let end = usize::from(QUAKE_ITEM_POS[table + 1]);
        let mut cursor = oam::OamPointers::current_cursor(self.ram);
        for &(x_offset, y_offset, flags) in &QUAKE_ITEMS[start..end] {
            let x = self
                .ram
                .read_u16(0x1580d)
                .wrapping_add(i16::from(x_offset) as u16)
                .wrapping_sub(self.ram.read_u16(0xe2));
            let y = self
                .ram
                .read_u16(0x1580b)
                .wrapping_add(i16::from(y_offset) as u16)
                .wrapping_sub(self.ram.read_u16(0xe8));
            let old_x = cursor.read(self.ram).x;
            let mut visible_x = old_x;
            let mut visible_y = 0xf0;
            if x < 256 && y < 256 {
                visible_x = x as u8;
                if y < 0xf0 {
                    visible_y = y as u8;
                }
            }
            oam::set_plain(
                self.ram,
                cursor.index(),
                visible_x,
                visible_y,
                CHAR[usize::from(flags & 0x0f)],
                flags & 0xc0 | 0x3c,
                2,
            );
            cursor.advance();
            oam::OamPointers::advance_entries(self.ram, 1);
        }
    }

    fn quake_shake_screen(&mut self) {
        let offset = self.ram.read_u16(0x1581e);
        self.ram.write_u16(0x11c, offset);
        self.ram.write_u16(0x1581e, offset.wrapping_neg());
        self.write(0x30, self.read(0x30).wrapping_add(offset as u8));
    }

    fn quake_control_bolts(&mut self, k: usize) {
        const LIMIT: [u8; 5] = [0x17, 0x16, 0x17, 0x16, 0x10];
        let mut next_step = self.step(k);
        self.write(0x1580f, next_step);
        let mut slot = usize::from(self.read(0x1580a));
        loop {
            if self.read(0x15805 + slot) != LIMIT[slot] {
                let delay = self.read(0x15800 + slot).wrapping_sub(1);
                self.write(0x15800 + slot, delay);
                let mut skip_draw = false;
                if (delay as i8) < 0 {
                    self.write(0x15800 + slot, 1);
                    let state = self.read(0x15805 + slot).wrapping_add(1);
                    self.write(0x15805 + slot, state);
                    if state == LIMIT[slot] {
                        skip_draw = true;
                    } else if slot == 0 && state == 2 {
                        misc::ancilla_sfx2_near(self.ram, 0x0c);
                        self.write(0x1580a, 1);
                    } else if slot == 1 && state == 2 {
                        self.write(0x1580a, 4);
                    } else if slot == 4 && state == 7 {
                        next_step = 1;
                        self.write(0x1580f, next_step);
                    }
                }
                if !skip_draw {
                    self.draw_quake_initial_bolts(slot);
                }
            }
            if slot == 0 {
                break;
            }
            slot -= 1;
        }
        self.set_step(k, next_step);
    }

    fn quake_spread_bolts(&mut self, k: usize) {
        const CHAR: [u8; 15] = [
            0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 0x60, 0x62, 0x64, 0x66, 0x68, 0x6a,
            0x63,
        ];
        if self.step(k) != 1 {
            return;
        }
        if self.timer(k) == 0 {
            self.set_timer(k, 2);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 55 {
                self.set_step(k, 2);
                return;
            }
        }
        let frame = usize::from(self.item_to_link(k));
        let start = usize::from(QUAKE_ITEM_POS2[frame]);
        let end = usize::from(QUAKE_ITEM_POS2[frame + 1]);
        let mut cursor = oam::OamPointers::current_cursor(self.ram);
        for &(x, y, flags) in &QUAKE_ITEMS2[start..end] {
            oam::set_plain(
                self.ram,
                cursor.index(),
                x as u8,
                y as u8,
                CHAR[usize::from(flags & 0x0f)],
                flags & 0xc0 | 0x3c,
                flags >> 4 & 3,
            );
            oam::OamPointers::advance_entries(self.ram, 1);
            cursor = self.wrap_custom_oam_cursor(cursor.next());
        }
    }

    pub fn quake_spell(&mut self, k: usize) {
        const LIMIT: [u8; 5] = [0x17, 0x16, 0x17, 0x16, 0x10];
        if self.read(0x11) != 0 {
            if self.read(0x15805 + 4) != LIMIT[4] {
                self.draw_quake_initial_bolts(k);
            }
            return;
        }
        if self.step(k) != 2 {
            self.quake_shake_screen();
            self.quake_control_bolts(k);
            self.quake_spread_bolts(k);
            return;
        }
        self.medallion_check_sprite_damage(k);
        self.runtime.prepare_apply_rumble_to_sprites(self.ram);
        self.clear(k);
        self.write(0x5d, 0);
        self.write(0xaaa, 1);
        self.write(0x324, 0);
        self.write(0x31c, 0);
        self.write(0x31d, 0);
        self.write(0x50, 0);
        self.write(0x3d, 0);
        self.write(0xfc1, 0);
        self.ram.write_u16(0x11a, 0);
        self.ram.write_u16(0x11c, 0);
        if self.read(0x8a) == 0x47
            && self.read(0xf280 + 0x47) & 0x20 == 0
            && self.check_for_entrance_trigger(3)
        {
            self.write(0x4c6, 4);
            self.write(0xb0, 0);
            self.write(0xc8, 0);
        }
        self.write(
            0x3a,
            if self.read(0x3c) != 0 {
                self.read(0xf0) & 0x80
            } else {
                0
            },
        );
        self.write(0x5e, 0);
        self.write(0x325, 0);
    }

    fn magic_powder_draw(&mut self, k: usize) {
        const X: [i8; 76] = [
            -5, -12, 2, -9, -7, -10, -6, -2, -6, -12, 1, -6, -6, -12, 1, -6, -6, -12, 1, -6, -6,
            -12, 1, -6, -6, -12, 1, -6, -17, -23, -14, -19, -11, -18, -9, -13, -4, -13, -1, -8, -3,
            -9, 0, -5, -3, -10, -1, -5, -4, -13, -1, -8, -3, -9, 0, -5, -3, -10, -1, -5, -3, -13,
            -1, -8, 9, 15, 6, 11, 3, 10, 1, 5, -4, 5, -7, 0,
        ];
        const Y: [i8; 76] = [
            -20, -15, -13, -7, -18, -13, -13, -13, -20, -13, -13, -8, -20, -13, -13, -8, -19, -12,
            -12, -7, -18, -11, -11, -6, -17, -10, -10, -5, -16, -14, -12, -9, -17, -14, -12, -8,
            -18, -14, -13, -6, -33, -31, -29, -26, -28, -25, -23, -19, -22, -18, -17, -10, -2, 0,
            2, 5, -9, -6, -4, 0, -16, -12, -11, -4, -16, -14, -12, -9, -17, -14, -12, -8, -18, -14,
            -13, -6,
        ];
        const CHAR: [u8; 19] = [9, 10, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9];
        const FLAGS: [u8; 76] = [
            0x68, 0x24, 0xa2, 0x28, 0x68, 0xe2, 0x28, 0xa4, 0x68, 0xe2, 0xa4, 0x28, 0x22, 0xa4,
            0xe8, 0x62, 0x24, 0xa8, 0xe2, 0x64, 0x28, 0xa2, 0xe4, 0x68, 0x22, 0xa4, 0xe8, 0x62,
            0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4,
            0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4, 0xe8, 0x64,
            0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8,
            0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68,
        ];
        let (x, y) = self.prep_oam_coord(k);
        let frame = usize::from(self.arr25(k));
        let index = self.current_oam_index();
        for i in 0..4 {
            let table = frame * 4 + i;
            self.set_oam(
                index + i,
                x.wrapping_add(i16::from(X[table]) as u16),
                y.wrapping_add(i16::from(Y[table]) as u16),
                CHAR[frame],
                (FLAGS[table] & !0x30) | (self.ram.read_u16(0x64) >> 8) as u8,
                0,
            );
        }
    }

    fn powder_apply_damage_to_sprites(&mut self, k: usize) {
        for sprite in (0..16).rev() {
            if ((self.read(0x1a) ^ sprite as u8) & 3) != 0
                || self.read(0xdd0 + sprite) != 9
                || self.read(0xcd2 + sprite) & 0x20 != 0
            {
                continue;
            }
            let hitbox = self.setup_basic_hitbox(k);
            if !self
                .runtime
                .sprite_hitbox_overlaps(self.ram, sprite, hitbox)
            {
                continue;
            }
            let mut value = self.read(0xe20 + sprite);
            let special = if value != 0x0b {
                true
            } else {
                value = self.read(0x1b);
                if value == 0 {
                    true
                } else {
                    value = self.ram.read_u16(0x48e).wrapping_sub(1) as u8;
                    value != 0
                }
            };
            if special {
                if value != 0x0d {
                    self.runtime
                        .check_damage_to_sprite_preset(self.ram, sprite, 10);
                    continue;
                }
                if self.read(0xeb0 + sprite) != 0 {
                    continue;
                }
            }
            self.write(0xeb0 + sprite, 1);
            self.runtime.sprite_spawn_poof_garnish(self.ram, sprite);
        }
    }

    pub fn powder_dust(&mut self, k: usize) {
        const FRAMES: [u8; 40] = [
            13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 10, 11, 12, 0, 1, 2, 3, 4, 5, 6, 16, 17, 18, 0, 1, 2,
            3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6,
        ];
        if self.read(0x11) == 0 {
            self.powder_apply_damage_to_sprites(k);
            let delay = self.aux_timer(k).wrapping_sub(1);
            self.set_aux_timer(k, delay);
            if (delay as i8) < 0 {
                self.set_aux_timer(k, 1);
                let direction = usize::from(self.direction(k));
                if self.item_to_link(k) == 9 {
                    self.clear(k);
                    self.write(0x333, 0);
                    return;
                }
                let frame = self.item_to_link(k).wrapping_add(1);
                self.set_item_to_link(k, frame);
                self.set_arr25(k, FRAMES[usize::from(frame) + direction * 10]);
            }
        }
        self.runtime.allocate_oam_region(
            self.ram,
            if self.read(0xfb3) == 0 { b'B' } else { b'E' },
            self.sprite_count(k),
        );
        self.magic_powder_draw(k);
    }

    fn dash_tremor_twiddle_offset(&mut self, k: usize) -> u16 {
        let direction = self.direction(k);
        let value = self.get_y(k).wrapping_neg();
        self.set_y(k, value);
        if self.read(0x1b) != 0 {
            return value;
        }
        let (start, end, scroll) = if direction == 2 {
            (
                self.ram.read_u16(0x600).wrapping_add(1),
                self.ram.read_u16(0x602).wrapping_sub(1),
                self.ram.read_u16(0xe8),
            )
        } else {
            (
                self.ram.read_u16(0x604).wrapping_add(1),
                self.ram.read_u16(0x606).wrapping_sub(1),
                self.ram.read_u16(0xe2),
            )
        };
        let absolute = value.wrapping_add(scroll);
        if absolute <= start || absolute >= end {
            0
        } else {
            value
        }
    }

    pub fn screen_shake(&mut self, k: usize) {
        if self.read(0x11) == 0 {
            let timer = self.item_to_link(k).wrapping_sub(1);
            self.set_item_to_link(k, timer);
            if (timer as i8) < 0 {
                self.ram.write_u16(0x11a, 0);
                self.ram.write_u16(0x11c, 0);
                self.clear(k);
                return;
            }
            let offset = self.dash_tremor_twiddle_offset(k);
            if self.direction(k) == 0 {
                self.ram.write_u16(0x11a, offset);
                self.write(0x31, self.read(0x31).wrapping_add(offset as u8));
            } else {
                self.ram.write_u16(0x11c, offset);
                self.write(0x30, self.read(0x30).wrapping_add(offset as u8));
            }
        }
        self.write(0xfdc, 3);
    }

    fn dash_dust_motive(&mut self, k: usize) {
        const CHAR: [u8; 3] = [0xa9, 0xcf, 0xdf];
        if self.timer(k) == 0 {
            self.set_timer(k, 3);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 3 {
                self.clear(k);
                return;
            }
        }
        if self.read(0x2f) == 2 {
            self.runtime.allocate_oam_region(self.ram, b'B', 4);
        }
        let (x, y) = self.prep_oam_coord(k);
        self.set_oam(
            self.current_oam_index(),
            x,
            y,
            CHAR[usize::from(self.item_to_link(k))],
            4 | (self.ram.read_u16(0x64) >> 8) as u8,
            0,
        );
    }

    pub fn dash_dust(&mut self, k: usize) {
        const X_ADJUST: [i8; 4] = [0, 0, 4, -4];
        const X: [i16; 30] = [
            10, 5, -1, 0, 10, 5, 0, 5, -1, 0, -1, -1, 9, -1, -1, 10, 5, -1, 0, 10, 5, 0, 5, -1, 0,
            -1, -1, 9, -1, -1,
        ];
        const Y: [i16; 30] = [
            -2, 0, -1, -3, -2, 0, -3, 0, -1, -3, -1, -1, -2, -1, -1, -2, 0, -1, -3, -2, 0, -3, 0,
            -1, -3, -1, -1, -2, -1, -1,
        ];
        const CHAR: [u8; 30] = [
            0xcf, 0xa9, 0xff, 0xa9, 0xdf, 0xcf, 0xcf, 0xdf, 0xff, 0xdf, 0xff, 0xff, 0xa9, 0xff,
            0xff, 0xcf, 0xcf, 0xff, 0xcf, 0xdf, 0xcf, 0xcf, 0xdf, 0xff, 0xdf, 0xff, 0xff, 0xcf,
            0xff, 0xff,
        ];
        if self.step(k) != 0 {
            self.dash_dust_motive(k);
            return;
        }
        if self.timer(k) == 0 {
            self.set_timer(k, 3);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 5 {
                return;
            }
            if frame == 6 {
                self.clear(k);
                return;
            }
        }
        if self.item_to_link(k) == 5 {
            return;
        }
        let (x, y) = self.prep_oam_coord(k);
        let adjustment = i16::from(X_ADJUST[usize::from(self.read(0x2f) >> 1)]);
        let mut frame =
            3 * (usize::from(self.item_to_link(k)) + if self.read(0x351) == 1 { 5 } else { 0 });
        let mut index = self.current_oam_index();
        for _ in (0..3).rev() {
            if CHAR[frame] != 0xff {
                self.set_oam(
                    index,
                    x.wrapping_add(adjustment.wrapping_add(X[frame]) as u16),
                    y.wrapping_add(Y[frame] as u16),
                    CHAR[frame],
                    4 | (self.ram.read_u16(0x64) >> 8) as u8,
                    0,
                );
                index += 1;
            }
            frame += 1;
        }
    }

    fn hookshot_should_bother_with_tiles(&mut self, k: usize) -> bool {
        let x = self.get_x(k);
        let y = self.get_y(k);
        if self.read(0x1b) == 0 {
            let area = self.read(0x700) >> 1;
            let (base_x, base_y) = self.runtime.overworld_offset_base(area);
            if self.direction(k) & 2 == 0 {
                let offset = y.wrapping_sub(base_y);
                return offset < 4 || offset >= self.ram.read_u16(0x716);
            }
            let offset = x.wrapping_sub(base_x);
            return offset < 6 || offset >= self.ram.read_u16(0x716);
        }
        if self.direction(k) & 2 == 0 {
            let offset = y & 0x1ff;
            !(4..0x1e8).contains(&offset) || y & 0x200 != self.ram.read_u16(0x20) & 0x200
        } else {
            let offset = x & 0x1ff;
            !(4..0x1f0).contains(&offset) || x & 0x200 != self.ram.read_u16(0x22) & 0x200
        }
    }

    fn hookshot_check_proximity_to_link(&self, x: u16, y: u16) -> bool {
        self.ram
            .read_u16(0x20)
            .wrapping_sub(self.ram.read_u16(0xe8))
            .wrapping_add(8)
            .wrapping_sub(y)
            .cast_signed()
            .unsigned_abs()
            < 12
            && self
                .ram
                .read_u16(0x22)
                .wrapping_sub(self.ram.read_u16(0xe2))
                .wrapping_add(4)
                .wrapping_sub(x)
                .cast_signed()
                .unsigned_abs()
                < 12
    }

    fn add_hookshot_wall_clink(&mut self, source: usize) {
        const X: [i8; 8] = [8, 8, 0, 10, 12, 8, 4, 0];
        const Y: [i8; 8] = [0, 8, 8, 8, 4, 8, 12, 8];
        let Some(k) = self.add_ancilla(6, 1) else {
            return;
        };
        self.set_item_to_link(k, 0);
        self.set_arr3(k, 1);
        let direction = usize::from(self.direction(source));
        self.set_xy(
            k,
            self.get_x(source)
                .wrapping_add(i16::from(X[direction]) as u16),
            self.get_y(source)
                .wrapping_add(i16::from(Y[direction]) as u16),
        );
    }

    fn hookshot_draw(&mut self, k: usize) {
        const MOVE_X: [i8; 4] = [0, 0, 8, -8];
        const MOVE_Y: [i8; 4] = [8, -9, 0, 0];
        const FLAGS: [u8; 12] = [0, 0, 0xff, 0x80, 0x80, 0xff, 0x40, 0xff, 0x40, 0, 0xff, 0];
        const CHAR: [u8; 12] = [9, 0x0a, 0xff, 9, 0x0a, 0xff, 9, 0xff, 0x0a, 9, 0xff, 0x0a];
        let (x0, y0) = self.prep_oam_coord(k);
        if self.state_l(k) != 0 {
            self.ram.write_u16(0x64, 0x3000);
        }
        let mut index = self.current_oam_index();
        let direction = usize::from(self.direction(k));
        let mut frame = direction * 3;
        let mut x = x0;
        let mut y = y0;
        for i in (0..3).rev() {
            if CHAR[frame] != 0xff {
                self.set_oam(
                    index,
                    x,
                    y,
                    CHAR[frame],
                    FLAGS[frame] | 2 | (self.ram.read_u16(0x64) >> 8) as u8,
                    0,
                );
                index += 1;
            }
            if i == 1 {
                x = x.wrapping_sub(8);
                y = y.wrapping_add(8);
            } else {
                x = x.wrapping_add(8);
            }
            frame += 1;
        }
        let mut remainder = 0_i16;
        let mut count = usize::from(self.item_to_link(k) >> 1);
        if count >= 7 {
            remainder = (count - 7) as i16;
            count = 6;
        }
        if count == 0 {
            return;
        }
        if self.direction(k) & 1 != 0 {
            remainder = -remainder;
        }
        x = x0;
        y = y0;
        if MOVE_Y[direction] == 0 {
            y = y.wrapping_add(4);
        }
        if MOVE_X[direction] == 0 {
            x = x.wrapping_add(4);
        }
        loop {
            if MOVE_Y[direction] != 0 {
                y = y.wrapping_add((i16::from(MOVE_Y[direction]) + remainder) as u16);
            }
            if MOVE_X[direction] != 0 {
                x = x.wrapping_add((i16::from(MOVE_X[direction]) + remainder) as u16);
            }
            if !self.hookshot_check_proximity_to_link(x, y) {
                self.set_oam(
                    index,
                    x,
                    y,
                    0x19,
                    (self.read(0x1a) & 2) << 6 | 2 | (self.ram.read_u16(0x64) >> 8) as u8,
                    0,
                );
                index += 1;
            }
            if count == 0 {
                break;
            }
            count -= 1;
        }
    }

    pub fn hookshot(&mut self, k: usize) {
        if self.read(0x11) != 0 {
            self.hookshot_draw(k);
            return;
        }
        if self.timer(k) == 0 {
            self.set_timer(k, 7);
            self.sfx2_pan(k, 0x0a);
        }
        if self.read(0x37e) != 0 {
            self.hookshot_draw(k);
            return;
        }
        self.move_y(k);
        self.move_x(k);
        if self.step(k) != 0 {
            let length = self.item_to_link(k).wrapping_sub(1);
            self.set_item_to_link(k, length);
            if (length as i8) < 0 {
                self.clear(k);
                return;
            }
            self.hookshot_draw(k);
            return;
        }
        let length = self.item_to_link(k).wrapping_add(1);
        self.set_item_to_link(k, length);
        if length == 32 {
            self.set_step(k, 1);
            self.set_x_velocity(k, self.x_velocity(k).wrapping_neg());
            self.set_y_velocity(k, self.y_velocity(k).wrapping_neg());
        }
        if self.hookshot_should_bother_with_tiles(k) {
            self.hookshot_draw(k);
            return;
        }
        if self.state_l(k) == 0
            && self.step(k) == 0
            && self.check_sprite_collision(k).is_some()
            && self.step(k) == 0
        {
            self.set_step(k, 1);
            self.set_y_velocity(k, self.y_velocity(k).wrapping_neg());
            self.set_x_velocity(k, self.x_velocity(k).wrapping_neg());
        }
        self.runtime.hookshot_check_tile_collision(self.ram, k);
        let ledge = if self.read(0x1b) != 0 {
            if self.direction(k) & 2 == 0 {
                (self.read(0x36d) | (self.read(0x36d) >> 4)) & 3
            } else {
                self.read(0x36e) & 3
            }
        } else {
            0
        };
        let detected = if self.read(0x1b) != 0 {
            ledge != 0
        } else {
            ((self.read(0x36e) & 3 | self.read(0x36d) | self.read(0x370)) & 0x33) != 0
        };
        if detected {
            let delay = self.state_g(k).wrapping_sub(1);
            self.set_state_g(k, delay);
            if (delay as i8) < 0 {
                if self.state_k(k) != 0 && (ledge & 3 != 0 || self.state_k(k) != self.read(0x76)) {
                    self.set_state_g(k, 2);
                    let l = self.state_l(k).wrapping_sub(1);
                    self.set_state_l(k, l);
                    if (l as i8) < 0 {
                        self.set_state_l(k, 0);
                    }
                } else {
                    self.set_state_l(k, self.state_l(k).wrapping_add(1));
                    self.set_state_k(k, self.read(0x76));
                    self.set_state_g(k, 1);
                }
            }
        }
        if self.state_l(k) != 0 {
            self.hookshot_draw(k);
            return;
        }
        if (self.state_g(k) as i8) >= 0 {
            self.set_state_g(k, self.state_g(k).wrapping_sub(1));
            self.hookshot_draw(k);
            return;
        }
        let obstacle = ((self.ram.read_u16(0x0e) >> 4)
            | self.ram.read_u16(0x0e)
            | u16::from(self.read(0x58))
            | self.ram.read_u16(0x0c))
            & 3
            != 0;
        if obstacle && self.step(k) == 0 {
            self.set_step(k, 1);
            self.set_y_velocity(k, self.y_velocity(k).wrapping_neg());
            self.set_x_velocity(k, self.x_velocity(k).wrapping_neg());
            if self.ram.read_u16(0x2f6) & 3 == 0 {
                self.add_hookshot_wall_clink(k);
                self.sfx2_pan(
                    k,
                    if self.ram.read_u16(0x2f6) & 0x30 != 0 {
                        6
                    } else {
                        5
                    },
                );
            }
        }
        if self.ram.read_u16(0x2f6) & 3 != 0 {
            if self.item_to_link(k) < 4 {
                self.clear(k);
                return;
            }
            self.write(0x37e, 1);
            self.write(0x39d, k as u8);
        }
        self.hookshot_draw(k);
    }

    pub fn blanket(&mut self, k: usize) {
        const CHAR: [u8; 8] = [0x0a, 0x0a, 0x0a, 0x0a, 0x0c, 0x0c, 0x0a, 0x0a];
        const FLAGS: [u8; 8] = [0, 0x60, 0xa0, 0xe0, 0, 0x60, 0xa0, 0xe0];
        let (mut x, mut y) = self.prep_oam_coord(k);
        let opening_pose = self.read(0x37d) != 0;
        self.runtime
            .allocate_oam_region(self.ram, if opening_pose { b'A' } else { b'B' }, 0x10);
        let mut frame = if opening_pose { 4 } else { 0 };
        let index = self.current_oam_index();
        for i in (0..4).rev() {
            self.set_oam(
                index + (3 - i),
                x,
                y,
                CHAR[frame],
                FLAGS[frame] | 0x0d | (self.ram.read_u16(0x64) >> 8) as u8,
                2,
            );
            x = x.wrapping_add(16);
            if i == 2 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(8);
            }
            frame += 1;
        }
    }

    pub fn snore(&mut self, k: usize) {
        const DMA: [u8; 3] = [0x44, 0x43, 0x42];
        let delay = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, delay);
        if (delay as i8) < 0 {
            if self.item_to_link(k) != 2 {
                self.set_item_to_link(k, self.item_to_link(k).wrapping_add(1));
            }
            self.set_aux_timer(k, 7);
        }
        let velocity = self.x_velocity(k).wrapping_add(self.step(k));
        self.set_x_velocity(k, velocity);
        if (velocity as i8).unsigned_abs() >= 8 {
            self.set_step(k, self.step(k).wrapping_neg());
        }
        self.move_y(k);
        self.move_x(k);
        if self.get_y(k) <= self.ram.read_u16(0x20).wrapping_sub(24) {
            self.clear(k);
        }
        self.write(0x109, DMA[usize::from(self.item_to_link(k))]);
        let (x, y) = self.prep_oam_coord(k);
        self.set_oam(self.current_oam_index(), x, y, 9, 0x24, 0);
    }

    pub fn sword_up_sparkle(&mut self, k: usize) {
        const X: [i8; 16] = [16, 0, 0, 0, 8, 16, 8, 16, 9, 15, 0, 0, 12, 0, 0, 0];
        const Y: [i8; 16] = [-7, 0, 0, 0, -11, -11, -3, -3, -7, -7, 0, 0, -7, 0, 0, 0];
        const CHAR: [u8; 16] = [
            0x92, 0xff, 0xff, 0xff, 0x93, 0x93, 0x93, 0x93, 0xf9, 0xf9, 0xff, 0xff, 0x80, 0xff,
            0xff, 0xff,
        ];
        const FLAGS: [u8; 16] = [
            0, 0xff, 0xff, 0xff, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0xff, 0xff, 0, 0xff, 0xff, 0xff,
        ];
        if self.aux_timer(k) != 0 {
            self.set_aux_timer(k, self.aux_timer(k).wrapping_sub(1));
            return;
        }
        let delay = self.arr3(k).wrapping_sub(1);
        self.set_arr3(k, delay);
        if (delay as i8) < 0 {
            self.set_arr3(k, 1);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 4 {
                self.clear(k);
                self.set_aux_timer(k, self.aux_timer(k).wrapping_sub(1));
                return;
            }
        }
        self.prep_oam_coord(k);
        let mut frame = usize::from(self.item_to_link(k)) * 4;
        let mut index = self.current_oam_index();
        for _ in 0..4 {
            if CHAR[frame] != 0xff {
                self.set_oam(
                    index,
                    self.ram
                        .read_u16(0x22)
                        .wrapping_add(i16::from(X[frame]) as u16)
                        .wrapping_sub(self.ram.read_u16(0xe2)),
                    self.ram
                        .read_u16(0x20)
                        .wrapping_add(i16::from(Y[frame]) as u16)
                        .wrapping_sub(self.ram.read_u16(0xe8)),
                    CHAR[frame],
                    FLAGS[frame] | 4 | (self.ram.read_u16(0x64) >> 8) as u8,
                    0,
                );
                index += 1;
            }
            frame += 1;
        }
    }

    pub fn spin_attack_charge_sparkle(&mut self, k: usize) {
        const CHAR: [u8; 3] = [0xb7, 0x80, 0x83];
        const FLAGS: [u8; 3] = [4, 4, 0x84];
        if self.read(0x11) == 0 && self.timer(k) == 0 {
            self.set_timer(k, 4);
            let frame = self.item_to_link(k).wrapping_add(1);
            self.set_item_to_link(k, frame);
            if frame == 3 {
                self.clear(k);
                return;
            }
        }
        self.allocate_oam_from_region_a_d_f(k, 4);
        self.set_oam_index(k, oam::OamPointers::legacy_primary_low_byte(self.ram));
        let (x, y) = self.prep_oam_coord(k);
        let frame = usize::from(self.item_to_link(k));
        self.set_oam(
            self.current_oam_index(),
            x,
            y,
            CHAR[frame],
            FLAGS[frame] | (self.ram.read_u16(0x64) >> 8) as u8,
            0,
        );
    }

    pub fn master_sword_receipt(&mut self, k: usize) {
        const X: [i8; 8] = [-1, 8, -1, 8, 0, 7, 0, 7];
        const Y: [i8; 8] = [1, 1, 9, 9, 1, 1, 9, 9];
        const CHAR: [u8; 8] = [0x86, 0x86, 0x96, 0x96, 0x87, 0x87, 0x97, 0x97];
        const FLAGS: [u8; 8] = [1, 0x41, 1, 0x41, 1, 0x41, 1, 0x41];
        if self.timer(k) == 0 {
            self.clear(k);
            return;
        }
        let delay = self.aux_timer(k).wrapping_sub(1);
        self.set_aux_timer(k, delay);
        if (delay as i8) < 0 {
            self.set_item_to_link(
                k,
                if self.item_to_link(k) == 2 {
                    0
                } else {
                    self.item_to_link(k).wrapping_add(1)
                },
            );
        }
        let (x, y) = self.prep_oam_coord(k);
        let frame = i16::from(self.item_to_link(k)) - 1;
        if frame < 0 {
            return;
        }
        let mut frame = frame as usize * 4;
        let index = self.current_oam_index();
        for i in 0..4 {
            self.set_oam(
                index + i,
                x.wrapping_add(i16::from(X[frame]) as u16),
                y.wrapping_add(i16::from(Y[frame]) as u16),
                CHAR[frame],
                (FLAGS[frame] & !0x30) | 4 | (self.ram.read_u16(0x64) >> 8) as u8,
                0,
            );
            frame += 1;
        }
    }

    pub fn somaria_bullet(&mut self, k: usize) {
        const MASK: [u8; 6] = [7, 3, 1, 0, 0, 0];
        if self.read(0x11) == 0 {
            if self.read(0x1a) & MASK[usize::from(self.step(k))] == 0 {
                self.move_x(k);
                self.move_y(k);
            }
            if self.timer(k) == 0 {
                self.set_timer(k, 3);
                let mut step = self.step(k).wrapping_add(1);
                if step >= 6 {
                    step = 4;
                }
                self.set_step(k, step);
            }
            let sprite_hit = self.check_sprite_collision(k).is_some();
            let tile_hit =
                ((self.read(0x1a) ^ k as u8) & 1) != 0 && self.check_tile_collision(k) != 0;
            if sprite_hit || tile_hit {
                self.set_kind(k, 4);
                self.set_timer(k, 7);
                self.set_sprite_count(k, 16);
            }
        }
        self.somaria_blast_draw(k);
    }

    fn somaria_blast_draw(&mut self, k: usize) {
        const X0: [i8; 24] = [
            0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const X1: [i8; 24] = [
            8, 8, 8, 8, 4, 4, 8, 8, 8, 8, 4, 4, 0, 0, 0, 0, 8, 8, 0, 0, 0, 0, 8, 8,
        ];
        const Y0: [u8; 24] = [
            0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4,
        ];
        const Y1: [u8; 24] = [
            0, 0, 0, 0, 8, 8, 0x80, 0, 0, 0, 8, 8, 0x80, 8, 8, 8, 4, 4, 0x80, 8, 8, 8, 4, 4,
        ];
        const FLAGS0: [u8; 24] = [
            0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0xc0, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0xc0, 0, 0, 0, 0, 0, 0x80,
        ];
        const FLAGS1: [u8; 24] = [
            0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0, 0, 0, 0, 0, 0x40, 0xc0, 0xc0, 0xc0, 0xc0, 0x40,
            0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0x80,
        ];
        const CHAR0: [u8; 24] = [
            0x50, 0x50, 0x44, 0x44, 0x52, 0x52, 0x50, 0x50, 0x44, 0x44, 0x51, 0x51, 0x43, 0x43,
            0x42, 0x42, 0x41, 0x41, 0x43, 0x43, 0x42, 0x42, 0x40, 0x40,
        ];
        const CHAR1: [u8; 24] = [
            0x50, 0x50, 0x44, 0x44, 0x51, 0x51, 0x50, 0x50, 0x44, 0x44, 0x52, 0x52, 0x43, 0x43,
            0x42, 0x42, 0x40, 0x40, 0x43, 0x43, 0x42, 0x42, 0x41, 0x41,
        ];
        let Some(mut info) = self.return_if_outside_bounds(k) else {
            return;
        };
        info.flags |= [2, 6][usize::from(self.item_to_link(k))];
        if self.object_priority(k) != 0 {
            info.flags |= 0x30;
        }
        let frame = usize::from(self.direction(k)) * 6 + usize::from(self.step(k));
        let index = self.current_oam_index();
        oam::set_plain(
            self.ram,
            index,
            info.x.wrapping_add(X0[frame] as u8),
            if (Y0[frame] as i8) < 0 {
                0xf0
            } else {
                info.y.wrapping_add(Y0[frame])
            },
            0x82_u8.wrapping_add(CHAR0[frame]),
            info.flags | FLAGS0[frame],
            0,
        );
        oam::set_plain(
            self.ram,
            index + 1,
            info.x.wrapping_add(X1[frame] as u8),
            if (Y1[frame] as i8) < 0 {
                0xf0
            } else {
                info.y.wrapping_add(Y1[frame])
            },
            0x82_u8.wrapping_add(CHAR1[frame]),
            info.flags | FLAGS1[frame],
            0,
        );
    }
}

pub(crate) fn add_bird_common<R: AncillaRuntime>(ctx: &mut Ancilla<'_, '_, R>, k: usize) {
    ctx.set_y_velocity(k, 0);
    ctx.set_item_to_link(k, 0);
    ctx.set_aux_timer(k, 1);
    ctx.set_x_velocity(k, 56);
    ctx.set_arr3(k, 3);
    ctx.set_state_k(k, 0);
    ctx.set_state_g(k, 0);
    let features = u32::from_le_bytes(ctx.ram.slice(0x64c, 4).try_into().unwrap());
    let extend = if features & 1 != 0 { 0x40 } else { 0 };
    let x = ctx.ram.read_u16(0xe2).wrapping_sub(16 + extend);
    let y = ctx.ram.read_u16(0x20).wrapping_sub(8);
    ctx.set_x(k, x);
    ctx.set_y(k, y);
}

pub(crate) fn boomerang_cheat<R: AncillaRuntime>(
    ctx: &Ancilla<'_, '_, R>,
    k: usize,
    speed: &mut ProjectSpeed,
) {
    let x = ctx
        .ram
        .read_u16(0x22)
        .wrapping_sub(ctx.get_x(k))
        .wrapping_add(0xf0);
    let y = ctx
        .ram
        .read_u16(0x20)
        .wrapping_sub(ctx.get_y(k))
        .wrapping_add(0xf0);
    if x >= 0x1e0 {
        speed.x = if (x.wrapping_sub(0x1e0) as i16) < 0 {
            0x90
        } else {
            0x70
        };
    } else if y >= 0x1e0 {
        speed.y = if (y.wrapping_sub(0x1e0) as i16) < 0 {
            0x90
        } else {
            0x70
        };
    }
}

/// `SomarianBlock_CheckEmpty`: visible entries are empty only when every
/// member of the four-entry group has its X high bit set.
pub(crate) fn somarian_block_check_empty<R: AncillaRuntime>(
    ctx: &Ancilla<'_, '_, R>,
    first: usize,
) -> bool {
    let any_visible = (0..4).any(|i| oam::read(ctx.ram, first + i).y != 0xf0);
    !any_visible || (0..4).all(|i| ctx.ram.read_u8(oam::BYTEWISE_EXTENDED_OAM + first + i) & 1 != 0)
}

pub(super) fn dispatch<R: AncillaRuntime>(
    ctx: &mut Ancilla<'_, '_, R>,
    kind: u8,
    k: usize,
) -> bool {
    match kind {
        1 => ctx.somaria_bullet(k),
        2 => ctx.fire_rod_shot(k),
        // Ancilla_Empty (type 3) is the only deliberately inert entry.
        3 => ctx.empty(k),
        4 => ctx.beam_hit(k),
        5 => ctx.boomerang(k),
        6 => ctx.wall_hit(k),
        7 => ctx.bomb(k),
        8 => ctx.door_debris(k),
        9 => ctx.arrow(k),
        0x0a => ctx.arrow_in_wall(k),
        0x0b => ctx.ice_rod_shot(k),
        0x0c => ctx.sword_beam(k),
        0x0d => ctx.spin_attack_full_charge_spark(k),
        0x0e..=0x10 | 0x12 | 0x33 => ctx.blast_wall_explosion(k),
        0x11 => ctx.ice_rod_wall_hit(k),
        0x13 => ctx.ice_rod_sparkle(k),
        0x15 => ctx.jump_splash(k),
        0x16 => ctx.hit_stars(k),
        0x17 => ctx.shovel_dirt(k),
        0x18 => ctx.ether_spell(k),
        0x19 => ctx.bombos_spell(k),
        0x1a => ctx.powder_dust(k),
        0x1b => ctx.sword_wall_hit(k),
        0x1c => ctx.quake_spell(k),
        0x1d => ctx.screen_shake(k),
        0x1e => ctx.dash_dust(k),
        0x1f => ctx.hookshot(k),
        0x20 => ctx.blanket(k),
        0x21 => ctx.snore(k),
        0x22 => ctx.item_receipt(k),
        0x23 => ctx.link_poof(k),
        0x24 => ctx.gravestone(k),
        0x26 => ctx.sword_swing_sparkle(k),
        0x27 => ctx.duck(k),
        0x28 => ctx.wish_pond_item(k),
        0x29 => ctx.milestone_item_receipt(k),
        0x2a => ctx.spin_attack_sparkle_a(k),
        0x2b => ctx.spin_attack_sparkle_b(k),
        0x2c => ctx.somaria_block(k),
        0x2d => ctx.somaria_block_fizz(k),
        0x2e => ctx.somaria_block_fission(k),
        0x2f => ctx.lamp_flame(k),
        0x30 => ctx.byrna_windup_spark(k),
        0x31 => ctx.byrna_spark(k),
        0x32 => ctx.blast_wall_fireball(k),
        0x34 => ctx.skull_woods_fire(k),
        0x35 => ctx.master_sword_receipt(k),
        0x36 => ctx.flute(k),
        0x37 => ctx.weathervane_explosion(k),
        0x38 => ctx.cutscene_duck(k),
        0x39 => ctx.somaria_platform_poof(k),
        0x3a => ctx.big_bomb_explosion(k),
        0x3b => ctx.sword_up_sparkle(k),
        0x3c => ctx.spin_attack_charge_sparkle(k),
        0x3d => ctx.item_splash(k),
        0x3e => ctx.rising_crystal(k),
        0x3f => ctx.bush_poof(k),
        0x40 => ctx.dwarf_poof(k),
        0x41 => ctx.waterfall_splash(k),
        0x42 => ctx.happiness_pond_rupees(k),
        0x43 => ctx.ganons_tower_cutscene(k),
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::Ram;
    use z3_core::WRAM_SIZE;

    #[derive(Default)]
    struct Runtime {
        projected_from: Option<(u16, u16, u8, u8)>,
        damages: Vec<(usize, u8)>,
        swim_checks: usize,
        interactions: Vec<&'static str>,
        refunds: Vec<u8>,
        decoded_graphics: Vec<u8>,
        map_updates: usize,
        dungeon_probes: Vec<(u8, u16, u16)>,
        transit_at: Option<(u16, u16)>,
    }
    impl AncillaRuntime for Runtime {
        fn sprite_project_speed_towards_link(
            &mut self,
            ram: &mut Ram<'_>,
            k: usize,
            velocity: u8,
        ) -> ProjectSpeed {
            self.projected_from = Some((
                u16::from_le_bytes([ram.read_u8(0xd10 + k), ram.read_u8(0xd30 + k)]),
                u16::from_le_bytes([ram.read_u8(0xd00 + k), ram.read_u8(0xd20 + k)]),
                ram.read_u8(0xf70 + k),
                velocity,
            ));
            ProjectSpeed {
                x: 0x22,
                y: 0xdd,
                xdiff: 1,
                ydiff: 2,
            }
        }

        fn check_damage_to_sprite_preset(&mut self, _ram: &mut Ram<'_>, sprite: usize, damage: u8) {
            self.damages.push((sprite, damage));
        }

        fn check_ability_to_swim(&mut self, _ram: &mut Ram<'_>) {
            self.swim_checks += 1;
        }

        fn terminate_sparkle_objects(&mut self, ram: &mut Ram<'_>) {
            self.interactions.push("Ancilla_TerminateSparkleObjects");
            for slot in 0..5 {
                if matches!(
                    ram.read_u8(0xc4a + slot),
                    0x2a | 0x2b | 0x30 | 0x31 | 0x18 | 0x19 | 0x0c
                ) {
                    ram.write_u8(0xc4a + slot, 0);
                }
            }
        }

        fn player_tile_detect_nearby(&mut self, _ram: &mut Ram<'_>) {
            self.interactions.push("Player_TileDetectNearby");
        }

        fn refund_magic(&mut self, _ram: &mut Ram<'_>, amount: u8) {
            self.refunds.push(amount);
        }

        fn decode_animated_sprite_tile(&mut self, _ram: &mut Ram<'_>, graphics: u8) {
            self.decoded_graphics.push(graphics);
        }

        fn overworld_do_map_update_32x32_b(&mut self, _ram: &mut Ram<'_>) {
            self.map_updates += 1;
        }

        fn dungeon_tile_attribute(
            &mut self,
            _ram: &mut Ram<'_>,
            floor: u8,
            x: u16,
            y: u16,
        ) -> (u8, u16) {
            self.dungeon_probes.push((floor, x, y));
            (
                if self.transit_at == Some((x, y)) {
                    0xb6
                } else {
                    0
                },
                x,
            )
        }
    }

    fn fixture(f: impl FnOnce(&mut Ancilla<'_, '_, Runtime>)) {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(0x90, 0x800);
        let mut runtime = Runtime::default();
        f(&mut Ancilla {
            ram: &mut ram,
            runtime: &mut runtime,
        });
    }

    #[test]
    fn beam_hit_expires_when_timer_reaches_zero() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = Runtime::default();
        let mut ctx = Ancilla {
            ram: &mut ram,
            runtime: &mut runtime,
        };
        ctx.write(Ancilla::<Runtime>::type_address(0), 4);
        assert!(dispatch(&mut ctx, 4, 0));
        assert_eq!(ctx.kind(0), 0);
    }

    #[test]
    fn somaria_empty_check_preserves_group_scan_quirk() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = Runtime::default();
        for i in 0..4 {
            oam::set_plain(&mut ram, i, 0, 0xf0, 0, 0, 0);
        }
        let ctx = Ancilla {
            ram: &mut ram,
            runtime: &mut runtime,
        };
        assert!(somarian_block_check_empty(&ctx, 0));
        oam::set_plain(ctx.ram, 0, 0, 10, 0, 0, 0);
        assert!(!somarian_block_check_empty(&ctx, 0));
        for i in 0..4 {
            ctx.ram.write_u8(oam::BYTEWISE_EXTENDED_OAM + i, 1);
        }
        assert!(somarian_block_check_empty(&ctx, 0));
    }

    #[test]
    fn bomb_projection_restores_sprite_zero_after_callback() {
        fixture(|ctx| {
            ctx.write(0xd10, 0x34);
            ctx.write(0xd30, 0x12);
            ctx.write(0xd00, 0x78);
            ctx.write(0xd20, 0x56);
            ctx.write(0xf70, 9);
            let speed = ctx.bomb_project_speed_towards_player(7, 0xabcd, 0xef01, 64);
            assert_eq!(speed.x, 0x22);
            assert_eq!(ctx.runtime.projected_from, Some((0xabcd, 0xef01, 0, 64)));
            assert_eq!(ctx.read(0xd10), 0x34);
            assert_eq!(ctx.read(0xd30), 0x12);
            assert_eq!(ctx.read(0xd00), 0x78);
            assert_eq!(ctx.read(0xd20), 0x56);
            assert_eq!(ctx.read(0xf70), 9);
        });
    }

    #[test]
    fn dash_dust_uses_facing_offsets_and_initializes_lifetime() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x22, 0x1000);
            ctx.ram.write_u16(0x20, 0x2000);
            ctx.write(0x2f, 4);
            let k = ctx.add_dashing_dust_ex(0x1e, 1, 1).unwrap();
            assert_eq!(k, 4);
            assert_eq!(ctx.get_x(k), 0x1006);
            assert_eq!(ctx.get_y(k), 0x2010);
            assert_eq!(ctx.step(k), 1);
            assert_eq!(ctx.read(0xc68 + k), 3);
        });
    }

    #[test]
    fn spin_draw_skips_ff_entries_without_consuming_oam() {
        fixture(|ctx| {
            ctx.set_xy(0, 0x1010, 0x2020);
            ctx.ram.write_u16(0xe2, 0x1000);
            ctx.ram.write_u16(0xe8, 0x2000);
            ctx.write(0xc5e, 1);
            ctx.spin_spark_draw(0, -1);
            assert_eq!(oam::read(ctx.ram, 0).charnum, 0x92);
            assert_eq!(oam::read(ctx.ram, 1).charnum, 0);
        });
    }

    #[test]
    fn upgraded_bow_remaps_arrow_damage_and_marks_d7_sprite() {
        fixture(|ctx| {
            ctx.write(0xf340, 3);
            ctx.write(0xe20 + 6, 0xd7);
            ctx.check_damage_to_sprite_aggressive(6, 0);
            assert_eq!(ctx.read(0xf10 + 6), 32);
            assert_eq!(ctx.runtime.damages, vec![(6, 9)]);
        });
    }

    #[test]
    fn fire_rod_creation_uses_initial_probe_then_facing_velocity() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x22, 0x100);
            ctx.ram.write_u16(0x20, 0x100);
            ctx.ram.write_u16(0xe2, 0x80);
            ctx.ram.write_u16(0xe8, 0x80);
            ctx.write(0x2f, 0);
            let k = ctx.add_fire_rod_shot(2, 9).unwrap();
            assert_eq!(k, 4);
            assert_eq!(ctx.get_x(k), 0x100);
            assert_eq!(ctx.get_y(k), 0xf8);
            assert_eq!(ctx.read(0xc2c + k), 0);
            assert_eq!(ctx.read(0xc22 + k), (-64_i8) as u8);
            assert_eq!(ctx.read(0xc68 + k), 3);
        });
    }

    #[test]
    fn ice_rod_creation_preserves_directional_offsets_and_state() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x22, 0x180);
            ctx.ram.write_u16(0x20, 0x190);
            ctx.ram.write_u16(0xe2, 0x100);
            ctx.ram.write_u16(0xe8, 0x100);
            ctx.write(0x2f, 4);
            let k = ctx.add_ice_rod_shot(2, 1).unwrap();
            assert_eq!(ctx.get_x(k), 0x16c);
            assert_eq!(ctx.get_y(k), 0x198);
            assert_eq!(ctx.direction(k), 2);
            assert_eq!(ctx.read(0xc2c + k), (-48_i8) as u8);
            assert_eq!(ctx.read(0xc22 + k), 0);
            assert_eq!(ctx.item_to_link(k), 0xff);
        });
    }

    #[test]
    fn splash_preserves_inverted_result_and_indoor_floor_side_effect() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x22, 0x100);
            ctx.ram.write_u16(0x20, 0x200);
            ctx.write(0x1b, 1);
            ctx.write(0x345, 0);
            ctx.write(0xee, 1);
            assert!(!ctx.add_splash(0x17, 4));
            assert_eq!(ctx.get_x(4), 0xf5);
            assert_eq!(ctx.get_y(4), 0x208);
            assert_eq!(ctx.read(0xee), 0);
        });
    }

    #[test]
    fn boomerang_constructor_preserves_clear_path_uint8_result() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x22, 0x180);
            ctx.ram.write_u16(0x20, 0x190);
            ctx.write(0xf341, 1);
            ctx.write(0x2f, 0);
            assert_eq!(ctx.add_boomerang(5, 4), 0xff);
            assert_eq!(ctx.get_x(4), 0x176);
            assert_eq!(ctx.get_y(4), 0x18e);
            assert_eq!(ctx.read(0xc22 + 4), (-32_i8) as u8);
            assert_eq!(ctx.read(0x35f), 1);
        });
    }

    #[test]
    fn happiness_pond_ten_rupees_populates_all_parallel_slots() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x22, 0x180);
            ctx.ram.write_u16(0x20, 0x190);
            ctx.add_happiness_pond_rupees(3).unwrap();
            assert_eq!(ctx.ram.slice(0x1586c, 10), &[1; 10]);
            assert_eq!(ctx.read(0x1580c), 0);
            assert_eq!(ctx.read(0x1580c + 5), (-9_i8) as u8);
            assert_eq!(ctx.runtime.decoded_graphics, vec![0x24]);
        });
    }

    #[test]
    fn gravestone_constructor_updates_map_and_latches_drag_state() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x22, 0x8f1);
            ctx.ram.write_u16(0x20, 0x540);
            let k = ctx.add_gravestone(0x24, 4).unwrap();
            assert_eq!(ctx.ram.read_u16(0x698), 0x99e);
            assert_eq!(ctx.ram.read_u16(0x692), 0x30);
            assert_eq!(ctx.get_x(k), 0x8f0);
            assert_eq!(ctx.get_y(k), 0x52e);
            assert_eq!(ctx.read(0x48), 4);
            assert_eq!(ctx.runtime.map_updates, 1);
        });
    }

    #[test]
    fn blast_wall_constructor_initializes_paired_explosions() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x10018, 0x500);
            ctx.ram.write_u16(0x1001a, 0x600);
            ctx.write(0x1001c, 0);
            ctx.add_blast_wall();
            assert_eq!(ctx.kind(0), 0x33);
            assert_eq!(ctx.kind(1), 0x33);
            assert_eq!(ctx.ram.read_u16(0x10018), 0x4f0);
            assert_eq!(ctx.ram.read_u16(0x10020), 0x500);
            assert_eq!(ctx.ram.read_u16(0x10030), 0x610);
            assert_eq!(ctx.read(0x10000), 1);
            assert_eq!(ctx.read(0x10008), 3);
        });
    }

    #[test]
    fn ganons_tower_constructor_clears_agahnim_and_sets_seal_state() {
        fixture(|ctx| {
            ctx.write(0xf37a, 0x7f);
            ctx.write(0xe20 + 6, 0x37);
            ctx.write(0xdd0 + 6, 9);
            ctx.ram.write_u16(0x22, 0x1234);
            ctx.ram.write_u16(0x20, 0x5678);
            let k = ctx.add_ganons_tower_cutscene().unwrap();
            assert_eq!(ctx.read(0xdd0 + 6), 0);
            assert_eq!(ctx.read(0x15812), 240);
            assert_eq!(ctx.ram.slice(0x15800, 6), &[0, 10, 22, 32, 42, 54]);
            assert_eq!(ctx.get_x(k), 0x1234);
            assert_eq!(ctx.get_y(k), 0x5668);
            assert_eq!(
                ctx.runtime.interactions[0],
                "Ancilla_TerminateSparkleObjects"
            );
        });
    }

    #[test]
    fn somaria_block_spawns_quad_in_descending_free_slots() {
        fixture(|ctx| {
            ctx.set_xy(5, 0x180, 0x190);
            ctx.write(0xc7c + 5, 1);
            ctx.ram.write_u16(0xe2, 0x100);
            ctx.ram.write_u16(0xe8, 0x100);
            ctx.somaria_block_spawn_bullets(5);
            for k in 1..=4 {
                assert_eq!(ctx.kind(k), 1);
                assert_eq!(ctx.step(k), 4);
                assert_eq!(ctx.floor(k), 1);
            }
            assert_eq!(ctx.direction(4), 3);
            assert_eq!(ctx.direction(1), 0);
            assert_eq!(ctx.read(0xfb5), 0xff);
        });
    }

    #[test]
    fn boomerang_screen_edge_short_circuits_step_decrement() {
        fixture(|ctx| {
            ctx.write(Ancilla::<Runtime>::type_address(0), 5);
            ctx.write(0x3b1, 1);
            ctx.write(0xc54, 7);
            ctx.write(0x39d, 1);
            ctx.set_xy(0, 0x200, 0x80);
            ctx.boomerang(0);
            assert_eq!(ctx.item_to_link(0), 1);
            assert_eq!(ctx.step(0), 7);
        });
    }

    #[test]
    fn radial_projection_preserves_c_rounding_and_quadrant_bits() {
        assert_eq!(
            Ancilla::<Runtime>::radial_projection(0, 40),
            super::super::AncillaRadialProjection {
                r0: 40,
                r2: 1,
                r4: 0,
                r6: 0
            }
        );
        assert_eq!(
            Ancilla::<Runtime>::radial_projection(16, 40),
            super::super::AncillaRadialProjection {
                r0: 0,
                r2: 0,
                r4: 40,
                r6: 0
            }
        );
        assert_eq!(
            Ancilla::<Runtime>::radial_projection(48, 40),
            super::super::AncillaRadialProjection {
                r0: 0,
                r2: 1,
                r4: 40,
                r6: 1
            }
        );
    }

    #[test]
    fn jump_splash_runs_swim_recovery_on_terminal_velocity() {
        fixture(|ctx| {
            ctx.write(Ancilla::<Runtime>::type_address(0), 0x15);
            ctx.write(0xc5e, 1);
            ctx.write(0xc22, 232);
            ctx.write(0x2e0, 1);
            ctx.write(0x345, 1);
            ctx.jump_splash(0);
            assert_eq!(ctx.kind(0), 0);
            assert_eq!(ctx.runtime.swim_checks, 1);
        });
    }

    #[test]
    fn ether_step_two_transitions_to_radial_spin() {
        fixture(|ctx| {
            ctx.write(Ancilla::<Runtime>::type_address(0), 0x18);
            ctx.write(0xc54, 2);
            ctx.write(0xc5e, 1);
            ctx.write(0x3b1, 0);
            ctx.write(0xc2c, 1);
            ctx.write(0x15808, 40);
            for i in 0..8 {
                ctx.write(0x15800 + i, (i * 8) as u8);
            }
            ctx.ether_spell(0);
            assert_eq!(ctx.step(0), 3);
            assert_eq!(ctx.item_to_link(0), 1);
            assert_eq!(ctx.read(0xc2c), 17);
        });
    }

    #[test]
    fn quake_hidden_piece_retains_x_and_advances_independent_extended_pointer() {
        fixture(|ctx| {
            oam::OamPointers {
                entry: 0x80c,
                extended: 0xffff,
            }
            .publish_entry_first(ctx.ram);
            ctx.write(0x15805, 0);
            ctx.ram.write_u16(0x1580b, 0);
            ctx.ram.write_u16(0x1580d, 0x100);
            ctx.write(0x80c, 0x47);
            let mut expected = ctx.ram.slice(0, z3_core::WRAM_SIZE).to_vec();
            ctx.draw_quake_initial_bolts(0);
            expected[0x80c..0x810].copy_from_slice(&[0x47, 0xf0, 0x40, 0x3c]);
            expected[0xa23] = 2;
            expected[0x90..0x94].copy_from_slice(&[0x10, 8, 0, 0]);
            assert_eq!(ctx.ram.slice(0, z3_core::WRAM_SIZE), expected);
        });
    }

    #[test]
    fn quake_spread_advances_globals_then_publishes_only_its_actual_local_wrap() {
        fixture(|ctx| {
            oam::OamPointers {
                entry: 0x98c,
                extended: 0xffff,
            }
            .publish_entry_first(ctx.ram);
            ctx.set_step(2, 1);
            ctx.set_timer(2, 1);
            ctx.set_item_to_link(2, 0);
            let mut expected = ctx.ram.slice(0, z3_core::WRAM_SIZE).to_vec();
            ctx.quake_spell_spread_bolts(2);
            expected[0x98c..0x990].copy_from_slice(&[160, 112, 0x40, 0x3c]);
            expected[0xa83] = 2;
            expected[0x90..0x94].copy_from_slice(&[0x20, 8, 0x28, 0xa]);
            assert_eq!(ctx.ram.slice(0, z3_core::WRAM_SIZE), expected);
        });
    }

    #[test]
    fn ether_orbit_updates_workspace_angles_without_treating_them_as_pool_slots() {
        fixture(|ctx| {
            ctx.set_kind(2, 0x18);
            ctx.set_step(2, 4);
            ctx.set_aux_timer(2, 2);
            ctx.set_item_to_link(2, 0);
            ctx.set_x_velocity(2, 0x44);
            ctx.write(0x15808, 40);
            ctx.write(0x15812, 10);
            ctx.ram.slice_mut(0x1642c, 8).fill(0xa5);
            for i in 0..8 {
                ctx.write(0x15800 + i, 0x3f - i as u8);
            }
            ctx.ether_spell(2);
            assert_eq!(ctx.ram.slice(0x15800, 8), &[0, 63, 62, 61, 60, 59, 58, 57]);
            assert_eq!(ctx.x_velocity(2), 0x44);
            assert_eq!(ctx.ram.slice(0x1642c, 8), &[0xa5; 8]);
        });
    }

    #[test]
    fn ether_expansion_updates_slot_velocity_without_writing_low_wram_scratch() {
        fixture(|ctx| {
            ctx.set_kind(2, 0x18);
            ctx.set_step(2, 5);
            ctx.set_aux_timer(2, 2);
            ctx.set_item_to_link(2, 0);
            ctx.set_x_velocity(2, 0x78);
            ctx.write(0x15808, 40);
            let scratch = ctx.read(2);
            ctx.ether_spell(2);
            assert_eq!(ctx.x_velocity(2), 0x7f);
            assert_eq!(ctx.read(2), scratch);
            assert_eq!(ctx.ram.slice(0x15800, 8), &[0; 8]);
        });
    }

    #[test]
    fn item_splash_clears_on_fifth_frame() {
        fixture(|ctx| {
            ctx.write(Ancilla::<Runtime>::type_address(0), 0x3d);
            ctx.write(0xc5e, 4);
            ctx.write(0xc68, 0);
            ctx.item_splash(0);
            assert_eq!(ctx.kind(0), 0);
            assert_eq!(ctx.item_to_link(0), 5);
            assert_eq!(ctx.read(0xc68), 6);
        });
    }

    #[test]
    fn receive_item_draw_uses_dynamic_palette_for_negative_table_entry() {
        fixture(|ctx| {
            ctx.write(0xc5e, 1);
            ctx.write(0xbf0, 5);
            let end = ctx.receive_item_draw(0, 0x20, 0x30);
            assert_eq!(end, 2);
            let top = oam::read(ctx.ram, 0);
            let bottom = oam::read(ctx.ram, 1);
            assert_eq!((top.charnum, top.flags), (0x24, 0x3a));
            assert_eq!((bottom.charnum, bottom.flags), (0x34, 0x3a));
        });
    }

    #[test]
    fn big_bomb_terminal_state_clears_before_drawing() {
        fixture(|ctx| {
            ctx.write(Ancilla::<Runtime>::type_address(0), 0x3a);
            ctx.write(0xc5e, 10);
            ctx.write(0x39f, 1);
            ctx.big_bomb_explosion(0);
            assert_eq!(ctx.kind(0), 0);
            assert_eq!(ctx.item_to_link(0), 11);
            assert_eq!(oam::read(ctx.ram, 0).charnum, 0);
        });
    }

    #[test]
    fn somaria_transit_probe_scans_backwards_and_transmutes_on_first_match() {
        fixture(|ctx| {
            ctx.write(0x1b, 1);
            ctx.write(0x3f4, 1);
            ctx.write(Ancilla::<Runtime>::type_address(2), 0x2c);
            ctx.write(0xc7c + 2, 1);
            ctx.write(0x280 + 2, 7);
            ctx.set_xy(2, 100, 100);
            ctx.runtime.transit_at = Some((116, 108));

            ctx.somaria_block_check_for_transit_tile(2);

            assert_eq!(ctx.runtime.dungeon_probes, vec![(1, 116, 108)]);
            assert_eq!((ctx.get_x(2), ctx.get_y(2)), (116, 108));
            assert_eq!(ctx.kind(2), 0x39);
            assert_eq!(ctx.object_priority(2), 7);
            assert_eq!(ctx.runtime.interactions, vec!["Player_TileDetectNearby"]);
        });
    }

    #[test]
    fn somaria_constructor_initializes_every_parallel_state_array() {
        fixture(|ctx| {
            ctx.ram.write_u16(0x22, 0x120);
            ctx.ram.write_u16(0x20, 0x180);
            ctx.write(0x2f, 4);

            let k = ctx.ancilla_add_somaria_block(0x2c, 1).unwrap();

            assert_eq!(ctx.kind(k), 0x2c);
            assert_eq!((ctx.get_x(k), ctx.get_y(k)), (0x118, 0x191));
            assert_eq!(ctx.direction(k), 2);
            assert_eq!(ctx.read(0x394 + k), 12);
            assert_eq!(ctx.read(0xc68 + k), 18);
            assert_eq!(ctx.read(0x3a9 + k), 9);
            assert_eq!(ctx.read(0x12f) & 0x3f, 0x2a);
        });
    }

    #[test]
    fn somaria_constructor_replaces_existing_block_and_releases_drag_state() {
        fixture(|ctx| {
            ctx.write(Ancilla::<Runtime>::type_address(3), 0x2c);
            ctx.write(0x2ec, 4);
            ctx.write(0x5e, 0x12);
            ctx.write(0x48, 0xff);

            let new_slot = ctx.ancilla_add_somaria_block(0x2c, 1).unwrap();

            assert_eq!(new_slot, 4);
            assert_eq!(ctx.kind(3), 0x2e);
            assert_eq!(ctx.kind(4), 0);
            assert_eq!(ctx.read(0x2ec), 0);
            assert_eq!(ctx.read(0x5e), 0);
            assert_eq!(ctx.read(0x48), 0);
        });
    }

    #[test]
    fn bird_presence_check_matches_the_six_c_visible_slots() {
        fixture(|ctx| {
            ctx.write(Ancilla::<Runtime>::type_address(9), 0x27);
            assert!(ctx.add_bird_travel(0x27, 4).is_some());
        });
    }

    #[test]
    fn sword_beam_sound_preserves_the_simultaneous_sword_swing_channel() {
        fixture(|ctx| {
            ctx.write(Ancilla::<Runtime>::type_address(0), 0x0c);
            ctx.write(0x12e, 0x82);
            ctx.ram.write_u16(0x1580e, 0x80);
            ctx.ram.write_u16(0x15810, 0x80);

            ctx.sword_beam(0);

            assert_eq!(ctx.read(0x12e), 0x82);
            assert_eq!(ctx.read(0x12f) & 0x3f, 1);
        });
    }

    #[test]
    fn ether_sound_preserves_the_other_sound_effect_channel() {
        fixture(|ctx| {
            ctx.write(0x12e, 0x82);
            assert!(ctx.add_ether_spell(0x18, 4).is_some());
            assert_eq!(ctx.read(0x12e), 0x82);
            assert_eq!(ctx.read(0x12f) & 0x3f, 0x26);
        });
    }
}
