//! Behavior-preserving port of `snes/ppu.c`.

pub const EXTRA_LEFT_RIGHT: usize = 96;
pub const X_PIXELS: usize = 256 + EXTRA_LEFT_RIGHT * 2;
pub const RENDER_NEW: u8 = 1;
pub const RENDER_MODE7_4X: u8 = 2;
pub const RENDER_HEIGHT_240: u8 = 4;
pub const RENDER_NO_SPRITE_LIMITS: u8 = 8;
pub const PPU_SAVE_BLOCK_LEN: usize = 67_305;

const SPRITE_SIZES: [[i32; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];
const BIT_DEPTHS: [[i32; 4]; 10] = [
    [2, 2, 2, 2],
    [4, 4, 2, 5],
    [4, 4, 5, 5],
    [8, 4, 5, 5],
    [8, 2, 5, 5],
    [4, 2, 5, 5],
    [4, 5, 5, 5],
    [8, 5, 5, 5],
    [4, 4, 2, 5],
    [8, 7, 5, 5],
];
const LAYERS: [[u8; 12]; 10] = [
    [4, 0, 1, 4, 0, 1, 4, 2, 3, 4, 2, 3],
    [4, 0, 1, 4, 0, 1, 4, 2, 4, 2, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 4, 0, 4, 5, 5, 5, 5, 5, 5],
    [4, 4, 4, 0, 4, 5, 5, 5, 5, 5, 5, 5],
    [2, 4, 0, 1, 4, 0, 1, 4, 4, 2, 5, 5],
    [4, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5, 5],
];
const PRIORITIES: [[u8; 12]; 10] = [
    [3, 1, 1, 2, 0, 0, 1, 1, 1, 0, 0, 0],
    [3, 1, 1, 2, 0, 0, 1, 1, 0, 0, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 0, 0, 5, 5, 5, 5, 5, 5],
    [3, 2, 1, 0, 0, 5, 5, 5, 5, 5, 5, 5],
    [1, 3, 1, 1, 2, 0, 0, 1, 0, 0, 5, 5],
    [3, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5, 5],
];
const LAYER_COUNTS: [usize; 10] = [12, 10, 8, 8, 8, 8, 6, 5, 10, 7];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BgLayer {
    pub h_scroll: u16,
    pub v_scroll: u16,
    pub tilemap_wider: bool,
    pub tilemap_higher: bool,
    pub tilemap_address: u16,
    pub tile_address: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ppu {
    pub line_has_sprites: bool,
    pub last_brightness_mult: u8,
    pub last_mosaic_modulo: u8,
    pub render_flags: u8,
    pub render_pitch: usize,
    pub render_buffer: Vec<u8>,
    pub extra_left_cur: u8,
    pub extra_right_cur: u8,
    pub extra_left_right: u8,
    pub extra_bottom_cur: u8,
    pub mode7_perspective_low: f32,
    pub mode7_perspective_high: f32,
    pub screen_enabled: [u8; 2],
    pub screen_windowed: [u8; 2],
    pub mosaic_enabled: u8,
    pub mosaic_size: u8,
    pub obj_tile_address_1: u16,
    pub obj_tile_address_2: u16,
    pub obj_size: u8,
    pub window1_left: u8,
    pub window1_right: u8,
    pub window2_left: u8,
    pub window2_right: u8,
    pub window_select: u32,
    pub clip_mode: u8,
    pub prevent_math_mode: u8,
    pub add_subscreen: bool,
    pub subtract_color: bool,
    pub half_color: bool,
    pub math_enabled: u8,
    pub fixed_color_r: u8,
    pub fixed_color_g: u8,
    pub fixed_color_b: u8,
    pub forced_blank: bool,
    pub brightness: u8,
    pub mode: u8,
    pub vram_pointer: u16,
    pub vram_increment: u16,
    pub vram_increment_on_high: bool,
    pub cgram_pointer: u8,
    pub cgram_second_write: bool,
    pub cgram_buffer: u8,
    pub oam_address: u16,
    pub oam_second_write: bool,
    pub oam_buffer: u8,
    pub bg_layer: [BgLayer; 4],
    pub scroll_previous: u8,
    pub scroll_previous_2: u8,
    pub mode7_matrix: [i16; 8],
    pub mode7_previous: u8,
    pub mode7_large_field: bool,
    pub mode7_character_fill: bool,
    pub mode7_x_flip: bool,
    pub mode7_y_flip: bool,
    pub mode7_ext_bg: bool,
    pub mode7_start_x: i32,
    pub mode7_start_y: i32,
    pub oam: [u16; 0x110],
    pub brightness_mult: [u8; 63],
    pub brightness_mult_half: [u8; 64],
    pub cgram: [u16; 0x100],
    pub mosaic_modulo: [u8; X_PIXELS],
    pub color_map_rgb: [u32; 256],
    pub bg_buffers: [[u16; X_PIXELS]; 2],
    pub obj_buffer: [u16; X_PIXELS],
    pub vram: [u16; 0x8000],
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            line_has_sprites: false,
            last_brightness_mult: 0xff,
            last_mosaic_modulo: 0xff,
            render_flags: 0,
            render_pitch: X_PIXELS * 4,
            render_buffer: Vec::new(),
            extra_left_cur: 0,
            extra_right_cur: 0,
            extra_left_right: EXTRA_LEFT_RIGHT as u8,
            extra_bottom_cur: 0,
            mode7_perspective_low: 0.0,
            mode7_perspective_high: 0.0,
            screen_enabled: [0; 2],
            screen_windowed: [0; 2],
            mosaic_enabled: 0,
            mosaic_size: 1,
            obj_tile_address_1: 0x4000,
            obj_tile_address_2: 0x5000,
            obj_size: 0,
            window1_left: 0,
            window1_right: 0,
            window2_left: 0,
            window2_right: 0,
            window_select: 0,
            clip_mode: 0,
            prevent_math_mode: 0,
            add_subscreen: false,
            subtract_color: false,
            half_color: false,
            math_enabled: 0,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            forced_blank: true,
            brightness: 0,
            mode: 0,
            vram_pointer: 0,
            vram_increment: 1,
            vram_increment_on_high: false,
            cgram_pointer: 0,
            cgram_second_write: false,
            cgram_buffer: 0,
            oam_address: 0,
            oam_second_write: false,
            oam_buffer: 0,
            bg_layer: [BgLayer::default(); 4],
            scroll_previous: 0,
            scroll_previous_2: 0,
            mode7_matrix: [0; 8],
            mode7_previous: 0,
            mode7_large_field: true,
            mode7_character_fill: false,
            mode7_x_flip: false,
            mode7_y_flip: false,
            mode7_ext_bg: false,
            mode7_start_x: 0,
            mode7_start_y: 0,
            oam: [0; 0x110],
            brightness_mult: [0; 63],
            brightness_mult_half: [0; 64],
            cgram: [0; 0x100],
            mosaic_modulo: [0; X_PIXELS],
            color_map_rgb: [0; 256],
            bg_buffers: [[0; X_PIXELS]; 2],
            obj_buffer: [0; X_PIXELS],
            vram: [0; 0x8000],
        }
    }
}

impl Ppu {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Transfers the deliberately sparse compatibility image used by the C
    /// `ppu_saveload` routine.
    pub fn saveload(
        &mut self,
        io: &mut impl crate::saveload::SaveLoad,
    ) -> Result<(), crate::saveload::SaveLoadError> {
        let mut block = vec![0; PPU_SAVE_BLOCK_LEN];
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
        use crate::saveload::{SaveLoadError, bool_byte, put_u16};
        if block.len() != PPU_SAVE_BLOCK_LEN {
            return Err(SaveLoadError::WrongBlockSize {
                expected: PPU_SAVE_BLOCK_LEN,
                actual: block.len(),
            });
        }
        for (index, &word) in self.vram.iter().enumerate() {
            put_u16(block, index * 2, word);
        }
        for (index, &word) in self.cgram.iter().enumerate() {
            put_u16(block, 65_546 + index * 2, word);
        }
        for (index, layer) in self.bg_layer.iter().enumerate() {
            let at = 67_134 + index * 12;
            block[at + 4] = bool_byte(layer.tilemap_wider);
            block[at + 5] = bool_byte(layer.tilemap_higher);
            put_u16(block, at + 6, layer.tilemap_address);
        }
        Ok(())
    }

    pub fn read_save_block(&mut self, block: &[u8]) -> Result<(), crate::saveload::SaveLoadError> {
        use crate::saveload::{SaveLoadError, get_u16};
        if block.len() != PPU_SAVE_BLOCK_LEN {
            return Err(SaveLoadError::WrongBlockSize {
                expected: PPU_SAVE_BLOCK_LEN,
                actual: block.len(),
            });
        }
        for (index, word) in self.vram.iter_mut().enumerate() {
            *word = get_u16(block, index * 2);
        }
        for (index, word) in self.cgram.iter_mut().enumerate() {
            *word = get_u16(block, 65_546 + index * 2);
        }
        for (index, layer) in self.bg_layer.iter_mut().enumerate() {
            let at = 67_134 + index * 12;
            layer.tilemap_wider = block[at + 4] != 0;
            layer.tilemap_higher = block[at + 5] != 0;
            layer.tilemap_address = get_u16(block, at + 6);
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        let buffer = core::mem::take(&mut self.render_buffer);
        let pitch = self.render_pitch;
        let extra = self.extra_left_right;
        let line_has_sprites = self.line_has_sprites;
        let render_flags = self.render_flags;
        let perspective_low = self.mode7_perspective_low;
        let perspective_high = self.mode7_perspective_high;
        let mosaic_enabled = self.mosaic_enabled;
        let brightness_mult = self.brightness_mult;
        let brightness_mult_half = self.brightness_mult_half;
        let mosaic_modulo = self.mosaic_modulo;
        let color_map_rgb = self.color_map_rgb;
        let bg_buffers = self.bg_buffers;
        *self = Self::default();
        self.render_buffer = buffer;
        self.render_pitch = pitch;
        self.extra_left_right = extra;
        self.line_has_sprites = line_has_sprites;
        self.render_flags = render_flags;
        self.mode7_perspective_low = perspective_low;
        self.mode7_perspective_high = perspective_high;
        self.mosaic_enabled = mosaic_enabled;
        self.brightness_mult = brightness_mult;
        self.brightness_mult_half = brightness_mult_half;
        self.mosaic_modulo = mosaic_modulo;
        self.color_map_rgb = color_map_rgb;
        self.bg_buffers = bg_buffers;
    }

    pub fn begin_drawing(&mut self, pitch: usize, height: usize, flags: u8) {
        self.render_flags = flags;
        self.render_pitch = pitch;
        let scale = self.current_render_scale(flags);
        self.render_buffer
            .resize(pitch.saturating_mul(height).saturating_mul(scale), 0);
        self.update_brightness();
    }
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.render_buffer
    }
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.render_buffer
    }
    #[must_use]
    pub fn current_render_scale(&self, flags: u8) -> usize {
        usize::from(
            self.mode == 7
                && !self.forced_blank
                && flags & (RENDER_MODE7_4X | RENDER_NEW) == (RENDER_MODE7_4X | RENDER_NEW),
        ) * 3
            + 1
    }
    pub fn set_mode7_perspective_correction(&mut self, low: i32, high: i32) {
        self.mode7_perspective_low = if low == 0 { 0.0 } else { 1.0 / low as f32 };
        self.mode7_perspective_high = 1.0 / high as f32;
    }
    pub fn set_extra_side_space(&mut self, left: usize, right: usize, bottom: usize) {
        let max = usize::from(self.extra_left_right);
        self.extra_left_cur = left.min(max) as u8;
        self.extra_right_cur = right.min(max) as u8;
        self.extra_bottom_cur = bottom.min(16) as u8;
    }
    fn update_brightness(&mut self) {
        if self.brightness != self.last_brightness_mult {
            self.last_brightness_mult = self.brightness;
            for i in 0..32usize {
                let v = ((((i << 3) | (i >> 2)) * usize::from(self.brightness)) / 15) as u8;
                self.brightness_mult[i] = v;
                self.brightness_mult_half[i * 2] = v;
                self.brightness_mult_half[i * 2 + 1] = v;
            }
            let maximum = self.brightness_mult[31];
            self.brightness_mult[32..].fill(maximum);
        }
        if self.current_render_scale(self.render_flags) == 4 {
            for (index, color) in self.cgram.iter().copied().enumerate() {
                self.color_map_rgb[index] =
                    u32::from(self.brightness_mult[usize::from(color & 0x1f)]) << 16
                        | u32::from(self.brightness_mult[usize::from((color >> 5) & 0x1f)]) << 8
                        | u32::from(self.brightness_mult[usize::from((color >> 10) & 0x1f)]);
            }
        }
    }

    #[must_use]
    pub fn read(&self, address: u8) -> u8 {
        match address {
            0x34..=0x36 => {
                let r = i32::from(self.mode7_matrix[0])
                    .wrapping_mul(i32::from(self.mode7_matrix[1]) >> 8);
                (r >> (8 * u32::from(address - 0x34))) as u8
            }
            _ => 0xff,
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x00 => {
                self.brightness = value & 15;
                self.forced_blank = value & 0x80 != 0;
            }
            0x01 => assert_eq!(value, 2),
            0x02 => {
                self.oam_address = (self.oam_address & 0xff00) | u16::from(value);
                self.oam_second_write = false;
            }
            0x03 => {
                assert_eq!(value & 0x80, 0);
                self.oam_address = (self.oam_address & 0xff) | (u16::from(value & 1) << 8);
                self.oam_second_write = false;
            }
            0x04 => {
                if self.oam_second_write {
                    if usize::from(self.oam_address) < 0x110 {
                        self.oam[usize::from(self.oam_address)] =
                            (u16::from(value) << 8) | u16::from(self.oam_buffer);
                        self.oam_address = self.oam_address.wrapping_add(1);
                    }
                } else {
                    self.oam_buffer = value;
                }
                self.oam_second_write = !self.oam_second_write;
            }
            0x05 => {
                self.mode = value & 7;
                assert!(value == 7 || value == 9);
                assert!(self.mode == 1 || self.mode == 7);
                assert_eq!(value & 0xf0, 0);
            }
            0x06 => {
                self.mosaic_size = (value >> 4) + 1;
                self.mosaic_enabled = if self.mosaic_size > 1 { value } else { 0 };
            }
            0x07..=0x0a => {
                let i = usize::from(address - 7);
                self.bg_layer[i].tilemap_wider = value & 1 != 0;
                self.bg_layer[i].tilemap_higher = value & 2 != 0;
                self.bg_layer[i].tilemap_address = u16::from(value & 0xfc) << 8;
            }
            0x0b => {
                self.bg_layer[0].tile_address = u16::from(value & 15) << 12;
                self.bg_layer[1].tile_address = u16::from(value & 0xf0) << 8;
            }
            0x0c => {
                self.bg_layer[2].tile_address = u16::from(value & 15) << 12;
                self.bg_layer[3].tile_address = u16::from(value & 0xf0) << 8;
            }
            0x0d | 0x0f | 0x11 | 0x13 => {
                if address == 0x0d {
                    self.mode7_matrix[6] = (((u16::from(value) << 8)
                        | u16::from(self.mode7_previous))
                        & 0x1fff) as i16;
                    self.mode7_previous = value;
                }
                let i = usize::from((address - 0x0d) / 2);
                self.bg_layer[i].h_scroll = ((u16::from(value) << 8)
                    | u16::from(self.scroll_previous & 0xf8)
                    | u16::from(self.scroll_previous_2 & 7))
                    & 0x3ff;
                self.scroll_previous = value;
                self.scroll_previous_2 = value;
            }
            0x0e | 0x10 | 0x12 | 0x14 => {
                if address == 0x0e {
                    self.mode7_matrix[7] = (((u16::from(value) << 8)
                        | u16::from(self.mode7_previous))
                        & 0x1fff) as i16;
                    self.mode7_previous = value;
                }
                let i = usize::from((address - 0x0e) / 2);
                self.bg_layer[i].v_scroll =
                    ((u16::from(value) << 8) | u16::from(self.scroll_previous)) & 0x3ff;
                self.scroll_previous = value;
            }
            0x15 => {
                self.vram_increment = match value & 3 {
                    0 => 1,
                    1 => 32,
                    _ => 128,
                };
                assert_eq!((value & 12) >> 2, 0);
                self.vram_increment_on_high = value & 0x80 != 0;
            }
            0x16 => self.vram_pointer = (self.vram_pointer & 0xff00) | u16::from(value),
            0x17 => self.vram_pointer = (self.vram_pointer & 0xff) | (u16::from(value) << 8),
            0x18 => {
                let i = usize::from(self.vram_pointer & 0x7fff);
                self.vram[i] = (self.vram[i] & 0xff00) | u16::from(value);
                if !self.vram_increment_on_high {
                    self.vram_pointer = self.vram_pointer.wrapping_add(self.vram_increment);
                }
            }
            0x19 => {
                let i = usize::from(self.vram_pointer & 0x7fff);
                self.vram[i] = (self.vram[i] & 0xff) | (u16::from(value) << 8);
                if self.vram_increment_on_high {
                    self.vram_pointer = self.vram_pointer.wrapping_add(self.vram_increment);
                }
            }
            0x1a => {
                assert_eq!(value, 0x80);
                self.mode7_large_field = value & 0x80 != 0;
                self.mode7_character_fill = value & 0x40 != 0;
                self.mode7_y_flip = value & 2 != 0;
                self.mode7_x_flip = value & 1 != 0;
            }
            0x1b..=0x1e => {
                self.mode7_matrix[usize::from(address - 0x1b)] =
                    i16::from_le_bytes([self.mode7_previous, value]);
                self.mode7_previous = value;
            }
            0x1f..=0x20 => {
                self.mode7_matrix[usize::from(address - 0x1b)] =
                    (((u16::from(value) << 8) | u16::from(self.mode7_previous)) & 0x1fff) as i16;
                self.mode7_previous = value;
            }
            0x21 => {
                self.cgram_pointer = value;
                self.cgram_second_write = false;
            }
            0x22 => {
                if self.cgram_second_write {
                    self.cgram[usize::from(self.cgram_pointer)] =
                        (u16::from(value) << 8) | u16::from(self.cgram_buffer);
                    self.cgram_pointer = self.cgram_pointer.wrapping_add(1);
                } else {
                    self.cgram_buffer = value;
                }
                self.cgram_second_write = !self.cgram_second_write;
            }
            0x23 => self.window_select = (self.window_select & !0xff) | u32::from(value),
            0x24 => self.window_select = (self.window_select & !0xff00) | (u32::from(value) << 8),
            0x25 => {
                self.window_select = (self.window_select & !0xff0000) | (u32::from(value) << 16)
            }
            0x26 => self.window1_left = value,
            0x27 => self.window1_right = value,
            0x28 => self.window2_left = value,
            0x29 => self.window2_right = value,
            0x2a | 0x2b => assert_eq!(value, 0),
            0x2c => self.screen_enabled[0] = value,
            0x2d => self.screen_enabled[1] = value,
            0x2e => self.screen_windowed[0] = value,
            0x2f => self.screen_windowed[1] = value,
            0x30 => {
                assert_eq!(value & 1, 0);
                self.add_subscreen = value & 2 != 0;
                self.prevent_math_mode = (value >> 4) & 3;
                self.clip_mode = (value >> 6) & 3;
            }
            0x31 => {
                self.subtract_color = value & 0x80 != 0;
                self.half_color = value & 0x40 != 0;
                self.math_enabled = value & 0x3f;
            }
            0x32 => {
                if value & 0x80 != 0 {
                    self.fixed_color_b = value & 31;
                }
                if value & 0x40 != 0 {
                    self.fixed_color_g = value & 31;
                }
                if value & 0x20 != 0 {
                    self.fixed_color_r = value & 31;
                }
            }
            0x33 => {
                assert_eq!(value, 0);
                self.mode7_ext_bg = value & 0x40 != 0;
            }
            _ => {}
        }
    }

    fn enabled(&self, sub: usize, layer: usize) -> bool {
        self.screen_enabled[sub] & (1 << layer) != 0
    }
    fn windowed(&self, sub: usize, layer: usize) -> bool {
        self.screen_windowed[sub] & (1 << layer) != 0
    }
    fn mosaic(&self, layer: usize) -> bool {
        self.mosaic_enabled & (1 << layer) != 0
    }
    fn window_state(&self, layer: usize, x: i32) -> bool {
        let f = self.window_select >> (layer * 4);
        let (e1, e2) = (f & 2 != 0, f & 8 != 0);
        let mut w1 = x >= i32::from(self.window1_left) && x <= i32::from(self.window1_right);
        let mut w2 = x >= i32::from(self.window2_left) && x <= i32::from(self.window2_right);
        if f & 1 != 0 {
            w1 = !w1;
        }
        if f & 4 != 0 {
            w2 = !w2;
        }
        match (e1, e2) {
            (false, false) => false,
            (true, false) => w1,
            (false, true) => w2,
            (true, true) => w1 || w2,
        }
    }

    fn calculate_mode7_starts(&mut self, mut y: i32) {
        let sign13 = |v: i16| i32::from(((i32::from(v) << 3) as i16) >> 3);
        let (h, v, xc, yc) = (
            sign13(self.mode7_matrix[6]),
            sign13(self.mode7_matrix[7]),
            sign13(self.mode7_matrix[4]),
            sign13(self.mode7_matrix[5]),
        );
        let mut ch = h - xc;
        let mut cv = v - yc;
        ch = if ch & 0x2000 != 0 {
            ch | !1023
        } else {
            ch & 1023
        };
        cv = if cv & 0x2000 != 0 {
            cv | !1023
        } else {
            cv & 1023
        };
        if self.mosaic(0) {
            y -= (y - 1) % i32::from(self.mosaic_size);
        }
        let ry = if self.mode7_y_flip { 255 - y } else { y };
        self.mode7_start_x = (i32::from(self.mode7_matrix[0]).wrapping_mul(ch) & !63)
            .wrapping_add(i32::from(self.mode7_matrix[1]).wrapping_mul(ry) & !63)
            .wrapping_add(i32::from(self.mode7_matrix[1]).wrapping_mul(cv) & !63)
            .wrapping_add(xc << 8);
        self.mode7_start_y = (i32::from(self.mode7_matrix[2]).wrapping_mul(ch) & !63)
            .wrapping_add(i32::from(self.mode7_matrix[3]).wrapping_mul(ry) & !63)
            .wrapping_add(i32::from(self.mode7_matrix[3]).wrapping_mul(cv) & !63)
            .wrapping_add(yc << 8);
    }

    fn mode7_pixel(&self, mut x: i32, layer: usize, priority: bool) -> u16 {
        if self.mosaic(layer) {
            x -= x % i32::from(self.mosaic_size);
        }
        let rx = if self.mode7_x_flip { 255 - x } else { x };
        let mut xp = self
            .mode7_start_x
            .wrapping_add(i32::from(self.mode7_matrix[0]).wrapping_mul(rx))
            >> 8;
        let mut yp = self
            .mode7_start_y
            .wrapping_add(i32::from(self.mode7_matrix[2]).wrapping_mul(rx))
            >> 8;
        let mut outside = !(0..1024).contains(&xp) || !(0..1024).contains(&yp);
        xp &= 0x3ff;
        yp &= 0x3ff;
        if !self.mode7_large_field {
            outside = false;
        }
        let tile = if outside {
            0
        } else {
            self.vram[((yp >> 3) * 128 + (xp >> 3)) as usize] as u8
        };
        let pixel = if outside && !self.mode7_character_fill {
            0
        } else {
            (self.vram[usize::from(tile) * 64 + ((yp & 7) * 8 + (xp & 7)) as usize] >> 8) as u8
        };
        if layer == 1 {
            if (pixel & 0x80 != 0) != priority {
                0
            } else {
                u16::from(pixel & 0x7f)
            }
        } else {
            u16::from(pixel)
        }
    }

    fn bg_pixel(&self, x: i32, y: i32, layer: usize, priority: bool) -> u16 {
        let l = self.bg_layer[layer];
        let wide = self.mode == 5 || self.mode == 6;
        let bits_x = if wide { 4 } else { 3 };
        let high_x = if wide { 0x200 } else { 0x100 };
        let mut map = i32::from(l.tilemap_address) + (((y >> 3) & 31) << 5 | ((x >> bits_x) & 31));
        if x & high_x != 0 && l.tilemap_wider {
            map += 0x400;
        }
        if y & 0x100 != 0 && l.tilemap_higher {
            map += if l.tilemap_wider { 0x800 } else { 0x400 };
        }
        let tile = self.vram[(map & 0x7fff) as usize];
        if (tile & 0x2000 != 0) != priority {
            return 0;
        }
        let mut palette = i32::from((tile & 0x1c00) >> 10);
        let row = if tile & 0x8000 != 0 {
            7 - (y & 7)
        } else {
            y & 7
        };
        let col = if tile & 0x4000 != 0 {
            x & 7
        } else {
            7 - (x & 7)
        };
        let mut number = i32::from(tile & 0x3ff);
        if wide && ((x & 8 != 0) ^ (tile & 0x4000 != 0)) {
            number += 1;
        }
        let depth = BIT_DEPTHS[usize::from(self.mode)][layer];
        if self.mode == 0 {
            palette += 8 * layer as i32;
        }
        let base = i32::from(l.tile_address) + (number & 0x3ff) * 4 * depth;
        let p1 = self.vram[((base + row) & 0x7fff) as usize];
        let mut pixel = i32::from((p1 >> col) & 1) | (i32::from((p1 >> (8 + col)) & 1) << 1);
        let mut palette_size = 4;
        if depth > 2 {
            palette_size = 16;
            let p2 = self.vram[((base + 8 + row) & 0x7fff) as usize];
            pixel |= i32::from((p2 >> col) & 1) << 2;
            pixel |= i32::from((p2 >> (8 + col)) & 1) << 3;
        }
        if depth > 4 {
            palette_size = 256;
            for p in 0..2 {
                let w = self.vram[((base + 16 + p * 8 + row) & 0x7fff) as usize];
                pixel |= i32::from((w >> col) & 1) << (4 + p * 2);
                pixel |= i32::from((w >> (8 + col)) & 1) << (5 + p * 2);
            }
        }
        if pixel == 0 {
            0
        } else {
            (palette_size * palette + pixel) as u16
        }
    }

    fn pixel(&self, x: i32, y: i32, sub: bool) -> (u8, i32, i32, i32) {
        let mode = if self.mode == 1 {
            8
        } else if self.mode == 7 && self.mode7_ext_bg {
            9
        } else {
            usize::from(self.mode)
        };
        let mut found_layer = 5u8;
        let mut found_pixel = 0u16;
        for i in 0..LAYER_COUNTS[mode] {
            let layer = usize::from(LAYERS[mode][i]);
            let priority = PRIORITIES[mode][i];
            let screen = usize::from(sub);
            // The optimized C renderer deliberately keeps BG3 within the
            // native 256-pixel viewport while extending the other layers.
            if layer == 2 && !(0..256).contains(&x) {
                continue;
            }
            if !self.enabled(screen, layer)
                || (self.windowed(screen, layer) && self.window_state(layer, x))
            {
                continue;
            }
            let p = if layer < 4 {
                let (mut lx, mut ly) = (x, y);
                if self.mosaic(layer) {
                    lx -= lx % i32::from(self.mosaic_size);
                    // The C optimized renderer samples the mosaic table at
                    // the one-based scanline, while its legacy pixel renderer
                    // rounds from scanline - 1. Preserve each renderer's
                    // phase so both match their respective C paths.
                    ly -= if self.render_flags & RENDER_NEW != 0 {
                        ly % i32::from(self.mosaic_size)
                    } else {
                        (ly - 1) % i32::from(self.mosaic_size)
                    };
                }
                if self.mode == 7 {
                    self.mode7_pixel(lx, layer, priority != 0)
                } else {
                    lx += i32::from(self.bg_layer[layer].h_scroll);
                    ly += i32::from(self.bg_layer[layer].v_scroll);
                    self.bg_pixel(lx & 0x3ff, ly & 0x3ff, layer, priority != 0)
                }
            } else {
                let e = self.obj_buffer[(x + EXTRA_LEFT_RIGHT as i32) as usize];
                if e >> 12 == u16::from(priority * 4 + 2) {
                    e & 0xff
                } else {
                    0
                }
            };
            if p != 0 {
                found_layer = layer as u8;
                found_pixel = p;
                break;
            }
        }
        let c = self.cgram[usize::from(found_pixel & 0xff)];
        let layer = if found_layer == 4 && found_pixel < 0xc0 {
            6
        } else {
            found_layer
        };
        (
            layer,
            i32::from(c & 31),
            i32::from((c >> 5) & 31),
            i32::from((c >> 10) & 31),
        )
    }

    fn handle_pixel(&mut self, x: i32, y: i32, row: usize) {
        let (mut r, mut g, mut b) = (0, 0, 0);
        if !self.forced_blank {
            let (layer, mr, mg, mb) = self.pixel(x, y, false);
            (r, g, b) = (mr, mg, mb);
            let cw = self.window_state(5, x);
            if self.clip_mode == 3 || self.clip_mode == 2 && cw || self.clip_mode == 1 && !cw {
                (r, g, b) = (0, 0, 0);
            }
            let math = layer < 6
                && self.math_enabled & (1 << layer) != 0
                && !(self.prevent_math_mode == 3
                    || self.prevent_math_mode == 2 && cw
                    || self.prevent_math_mode == 1 && !cw);
            let (mut sl, mut r2, mut g2, mut b2) = (5, 0, 0, 0);
            if math && self.add_subscreen || self.mode == 5 || self.mode == 6 {
                (sl, r2, g2, b2) = self.pixel(x, y, true);
            }
            if math {
                let cr = if self.add_subscreen && sl != 5 {
                    r2
                } else {
                    i32::from(self.fixed_color_r)
                };
                let cg = if self.add_subscreen && sl != 5 {
                    g2
                } else {
                    i32::from(self.fixed_color_g)
                };
                let cb = if self.add_subscreen && sl != 5 {
                    b2
                } else {
                    i32::from(self.fixed_color_b)
                };
                if self.subtract_color {
                    r -= cr;
                    g -= cg;
                    b -= cb;
                } else {
                    r += cr;
                    g += cg;
                    b += cb;
                }
                if self.half_color && (sl != 5 || !self.add_subscreen) {
                    r >>= 1;
                    g >>= 1;
                    b >>= 1;
                }
                r = r.clamp(0, 31);
                g = g.clamp(0, 31);
                b = b.clamp(0, 31);
            }
        }
        // The z-buffer always has the compile-time overscan margin, but the
        // output surface is only as wide as the configured visible margin.
        // In 4:3 mode that margin is zero, so using EXTRA_LEFT_RIGHT here
        // writes beyond a 256-pixel SDL texture.
        let o = row + (x + i32::from(self.extra_left_right)) as usize * 4;
        let br = i32::from(self.brightness);
        self.render_buffer[o] = ((((b << 3) | (b >> 2)) * br) / 15) as u8;
        self.render_buffer[o + 1] = ((((g << 3) | (g >> 2)) * br) / 15) as u8;
        self.render_buffer[o + 2] = ((((r << 3) | (r >> 2)) * br) / 15) as u8;
        self.render_buffer[o + 3] = 0;
    }

    fn evaluate_sprites(&mut self, line: i32) -> bool {
        let unlimited = self.render_flags & RENDER_NO_SPRITE_LIMITS != 0;
        let mut sprites = if unlimited { 1024 } else { 33 };
        let mut tiles = if unlimited { 1024 } else { 35 };
        let original = tiles;
        let sizes = SPRITE_SIZES[usize::from(self.obj_size)];
        let extra = i32::from(self.extra_left_right);
        for index in (0..=0xfeusize).step_by(2) {
            let yy = i32::from(self.oam[index] >> 8);
            if yy == 0xf0 {
                continue;
            }
            let mut row = (line - yy) & 0xff;
            let high = i32::from(self.oam[0x100 + (index >> 4)] >> (index & 15));
            let size = sizes[((high >> 1) & 1) as usize];
            if row >= size {
                continue;
            }
            let mut x = i32::from(self.oam[index] & 0xff) + (high & 1) * 256;
            if x >= 256 + extra {
                x -= 512;
            }
            if x <= -(size + extra) {
                continue;
            }
            sprites -= 1;
            if sprites == 0 {
                break;
            }
            let oam = i32::from(self.oam[index + 1]);
            let address = if oam & 0x100 != 0 {
                self.obj_tile_address_2
            } else {
                self.obj_tile_address_1
            };
            if oam & 0x8000 != 0 {
                row = size - 1 - row;
            }
            let palette = 0x80 + 16 * ((oam & 0xe00) >> 9);
            let prio =
                (((oam & 0x3000) >> 12) * 4 + 2) * 16 + 4 + if oam & 0x800 == 0 { 2 } else { 0 };
            let z = (palette + (prio << 8)) as u16;
            for col in (0..size).step_by(8) {
                if col + x <= -8 - extra || col + x >= 256 + extra {
                    continue;
                }
                tiles -= 1;
                if tiles == 0 {
                    return true;
                }
                let used = if oam & 0x4000 != 0 {
                    size - 1 - col
                } else {
                    col
                };
                let tile =
                    ((((oam & 0xff) >> 4) + (row >> 3)) << 4) | (((oam & 15) + (used >> 3)) & 15);
                let a = (i32::from(address) + tile * 16 + (row & 7)) & 0x7fff;
                let plane = u32::from(self.vram[a as usize])
                    | (u32::from(self.vram[((a + 8) & 0x7fff) as usize]) << 16);
                let left = (-(col + x + EXTRA_LEFT_RIGHT as i32)).max(0);
                let right = (256 + EXTRA_LEFT_RIGHT as i32 - (col + x)).min(8);
                for px in left..right {
                    let shift = if oam & 0x4000 != 0 { px } else { 7 - px };
                    let bits = plane >> shift;
                    let p =
                        (bits & 1) | ((bits >> 7) & 2) | ((bits >> 14) & 4) | ((bits >> 21) & 8);
                    let d = (col + x + px + EXTRA_LEFT_RIGHT as i32) as usize;
                    if p != 0 && self.obj_buffer[d] & 0xff == 0 {
                        self.obj_buffer[d] = z.wrapping_add(p as u16);
                    }
                }
            }
        }
        tiles != original
    }

    /// Zelda's high-resolution Mode 7 path. This intentionally mirrors the
    /// C renderer's fixed-point arithmetic and its limited feature set.
    fn draw_mode7_upsampled(&mut self, y: usize) {
        let sign13 = |value: i16| i32::from(((i32::from(value) << 3) as i16) >> 3);
        let x_center = sign13(self.mode7_matrix[4]) as u32;
        let y_center = sign13(self.mode7_matrix[5]) as u32;
        let clipped_h = (sign13(self.mode7_matrix[6]) as u32).wrapping_sub(x_center);
        let clipped_v = (sign13(self.mode7_matrix[7]) as u32).wrapping_sub(y_center);
        let mut m0_values = [0u32; 4];
        if self.mode7_perspective_low.to_bits() == 0 {
            m0_values.fill((i32::from(self.mode7_matrix[0]) << 12) as u32);
        } else {
            const OFFSETS: [f32; 4] = [-1.0, -0.75, -0.5, -0.25];
            for (index, offset) in OFFSETS.into_iter().enumerate() {
                let position = y as f32 + offset;
                let interpolated = self.mode7_perspective_low
                    + (self.mode7_perspective_high - self.mode7_perspective_low)
                        * position
                        * (1.0 / 223.0);
                m0_values[index] = (4096.0 / interpolated) as i32 as u32;
            }
        }

        let pitch = self.render_pitch;
        let row_base = (y - 1) * 4 * pitch;
        let left = usize::from(self.extra_left_cur);
        let right = usize::from(self.extra_right_cur);
        let extra = usize::from(self.extra_left_right);
        let destination_start = row_base + (extra - left) * 16;
        let draw_width = 256 + left + right;
        let m1 = (i32::from(self.mode7_matrix[1]) << 12) as u32;
        let m2 = (i32::from(self.mode7_matrix[2]) << 12) as u32;

        for (subline, &m0) in m0_values.iter().enumerate() {
            let m3 = m0;
            let line = y as u32;
            let mut x_position = m0
                .wrapping_mul(clipped_h)
                .wrapping_add(m1.wrapping_mul(clipped_v.wrapping_add(line)))
                .wrapping_add(x_center << 20);
            let mut y_position = m2
                .wrapping_mul(clipped_h)
                .wrapping_add(m3.wrapping_mul(clipped_v.wrapping_add(line)))
                .wrapping_add(y_center << 20);
            x_position = x_position.wrapping_sub(m0.wrapping_add(m1) >> 1);
            y_position = y_position.wrapping_sub(m2.wrapping_add(m3) >> 1);
            let mut x_current = (x_position << 2).wrapping_add((subline as u32).wrapping_mul(m1));
            let mut y_current = (y_position << 2).wrapping_add((subline as u32).wrapping_mul(m3));
            x_current = x_current.wrapping_sub((left as u32).wrapping_mul(4).wrapping_mul(m0));
            y_current = y_current.wrapping_sub((left as u32).wrapping_mul(4).wrapping_mul(m2));

            for pixel_index in 0..draw_width * 4 {
                let tile_index =
                    (((y_current >> 25) & 0x7f) * 128 + ((x_current >> 25) & 0x7f)) as usize;
                let tile = usize::from(self.vram[tile_index] as u8);
                let color_index = usize::from(
                    (self.vram[tile * 64
                        + (((y_current >> 22) & 7) * 8 + ((x_current >> 22) & 7)) as usize]
                        >> 8) as u8,
                );
                let color = if x_current & 0x8000_0000 != 0 {
                    0
                } else if self.half_color {
                    (self.color_map_rgb[color_index] & 0xfe_fe_fe) >> 1
                } else {
                    self.color_map_rgb[color_index]
                };
                let offset = destination_start + subline * pitch + pixel_index * 4;
                self.render_buffer[offset..offset + 4].copy_from_slice(&color.to_le_bytes());
                x_current = x_current.wrapping_add(m0);
                y_current = y_current.wrapping_add(m2);
            }
        }

        if self.line_has_sprites {
            for index in 0..draw_width {
                let pixel = usize::from(self.obj_buffer[EXTRA_LEFT_RIGHT - left + index] & 0xff);
                if pixel == 0 {
                    continue;
                }
                let bytes = self.color_map_rgb[pixel].to_le_bytes();
                for subline in 0..4 {
                    let start = destination_start + subline * pitch + index * 16;
                    for sample in 0..4 {
                        let offset = start + sample * 4;
                        self.render_buffer[offset..offset + 4].copy_from_slice(&bytes);
                    }
                }
            }
        }

        let left_clear = (extra - left) * 16;
        let right_clear = (extra - right) * 16;
        let right_start = (256 + extra * 2 - (extra - right)) * 16;
        for subline in 0..4 {
            let row = row_base + subline * pitch;
            self.render_buffer[row..row + left_clear].fill(0);
            self.render_buffer[row + right_start..row + right_start + right_clear].fill(0);
        }
    }

    pub fn run_line(&mut self, line: usize) {
        if line == 0 {
            return;
        }
        if self.mosaic_size != self.last_mosaic_modulo {
            self.last_mosaic_modulo = self.mosaic_size;
            let mut phase = 0usize;
            for (index, value) in self.mosaic_modulo.iter_mut().enumerate() {
                *value = index.wrapping_sub(phase) as u8;
                phase = if phase + 1 == usize::from(self.mosaic_size) {
                    0
                } else {
                    phase + 1
                };
            }
        }
        self.obj_buffer.fill(0x0500);
        self.line_has_sprites = !self.forced_blank && self.evaluate_sprites(line as i32 - 1);
        let scale = self.current_render_scale(self.render_flags);
        let row = (line - 1)
            .saturating_mul(self.render_pitch)
            .saturating_mul(scale);
        let needed = row.saturating_add(self.render_pitch.saturating_mul(scale));
        if self.render_buffer.len() < needed {
            return;
        }
        if line >= 225 + usize::from(self.extra_bottom_cur) || self.forced_blank {
            self.render_buffer[row..needed].fill(0);
            return;
        }
        if self.render_flags & RENDER_NEW != 0
            && self.mode == 7
            && self.render_flags & RENDER_MODE7_4X != 0
        {
            self.draw_mode7_upsampled(line);
            return;
        }
        if self.mode == 7 {
            self.calculate_mode7_starts(line as i32);
        }
        let (first_x, last_x) = if self.render_flags & RENDER_NEW != 0 {
            (
                -i32::from(self.extra_left_cur),
                256 + i32::from(self.extra_right_cur),
            )
        } else {
            (0, 256)
        };
        for x in first_x..last_x {
            self.handle_pixel(x, line as i32, row);
        }
        let side = usize::from(self.extra_left_right);
        let left_clear = side
            - usize::from(self.extra_left_cur) * usize::from(self.render_flags & RENDER_NEW != 0);
        let right_clear = side
            - usize::from(self.extra_right_cur) * usize::from(self.render_flags & RENDER_NEW != 0);
        self.render_buffer[row..row + left_clear * 4].fill(0);
        let visible_width = 256 + side * 2;
        let right = row + (visible_width - right_clear) * 4;
        self.render_buffer[right..right + right_clear * 4].fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reset_state() {
        let p = Ppu::new();
        assert!(p.forced_blank);
        assert_eq!(p.vram_increment, 1);
        assert_eq!(p.obj_tile_address_1, 0x4000);
        assert!(p.mode7_large_field);
    }
    #[test]
    fn vram_increment() {
        let mut p = Ppu::new();
        p.write(0x15, 1);
        p.write(0x16, 3);
        p.write(0x18, 0x12);
        assert_eq!(p.vram[3], 0x12);
        assert_eq!(p.vram_pointer, 35);
        p.write(0x15, 0x81);
        p.write(0x19, 0x34);
        assert_eq!(p.vram[35], 0x3400);
        assert_eq!(p.vram_pointer, 67);
    }
    #[test]
    fn paired_writes() {
        let mut p = Ppu::new();
        p.write(0x21, 9);
        p.write(0x22, 0x34);
        p.write(0x22, 0x12);
        assert_eq!(p.cgram[9], 0x1234);
        p.write(0x02, 7);
        p.write(0x04, 0x78);
        p.write(0x04, 0x56);
        assert_eq!(p.oam[7], 0x5678);
    }
    #[test]
    fn multiply_reads() {
        let mut p = Ppu::new();
        p.mode7_matrix[0] = -300;
        p.mode7_matrix[1] = 0x1200;
        let v = -300i32 * 0x12;
        assert_eq!(p.read(0x34), v as u8);
        assert_eq!(p.read(0x35), (v >> 8) as u8);
        assert_eq!(p.read(0x36), (v >> 16) as u8);
    }
    #[test]
    fn forced_blank() {
        let mut p = Ppu::new();
        p.begin_drawing(X_PIXELS * 4, 1, 0);
        p.render_buffer.fill(0xaa);
        p.run_line(1);
        assert!(p.render_buffer.iter().all(|&x| x == 0));
    }
    #[test]
    fn backdrop() {
        let mut p = Ppu::new();
        p.write(0, 15);
        p.cgram[0] = 31 | (16 << 5) | (1 << 10);
        p.begin_drawing(X_PIXELS * 4, 1, 0);
        p.run_line(1);
        let o = EXTRA_LEFT_RIGHT * 4;
        assert_eq!(&p.render_buffer[o..o + 4], &[8, 132, 255, 0]);
    }

    #[test]
    fn four_three_output_uses_a_256_pixel_surface() {
        let mut p = Ppu::new();
        p.extra_left_right = 0;
        p.write(0, 15);
        p.begin_drawing(256 * 4, 1, 0);
        p.run_line(1);
        assert_eq!(p.render_buffer.len(), 256 * 4);
    }

    #[test]
    fn mosaic_table_stores_each_block_start_with_byte_wrapping() {
        let mut p = Ppu::new();
        p.write(0x06, 0x21);
        p.begin_drawing(X_PIXELS * 4, 1, 0);
        p.run_line(1);
        assert_eq!(&p.mosaic_modulo[..7], &[0, 0, 0, 3, 3, 3, 6]);
        assert_eq!(p.mosaic_modulo[258], 2);
    }

    #[test]
    fn reset_preserves_fields_omitted_by_the_c_reset() {
        let mut p = Ppu::new();
        p.render_flags = RENDER_NEW;
        p.mosaic_enabled = 0x0f;
        p.mode7_perspective_low = 0.25;
        p.bg_buffers[0][7] = 0xbeef;
        p.vram[7] = 0xbeef;
        p.reset();
        assert_eq!(p.render_flags, RENDER_NEW);
        assert_eq!(p.mosaic_enabled, 0x0f);
        assert_eq!(p.mode7_perspective_low, 0.25);
        assert_eq!(p.bg_buffers[0][7], 0xbeef);
        assert_eq!(p.vram[7], 0);
    }

    #[test]
    fn new_renderer_expands_active_side_space() {
        let mut p = Ppu::new();
        p.write(0, 15);
        p.cgram[0] = 31;
        p.set_extra_side_space(5, 7, 0);
        p.begin_drawing(X_PIXELS * 4, 1, RENDER_NEW);
        p.run_line(1);
        let first = (EXTRA_LEFT_RIGHT - 5) * 4;
        let last = (EXTRA_LEFT_RIGHT + 256 + 7 - 1) * 4;
        assert_eq!(&p.render_buffer[first..first + 4], &[0, 0, 255, 0]);
        assert_eq!(&p.render_buffer[last..last + 4], &[0, 0, 255, 0]);
        assert!(p.render_buffer[..first].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn new_and_reference_renderers_match_native_viewport() {
        let mut reference = Ppu::new();
        reference.write(0, 12);
        reference.cgram[0] = 7 | (19 << 5) | (28 << 10);
        reference.fixed_color_r = 3;
        reference.math_enabled = 1 << 5;
        reference.begin_drawing(X_PIXELS * 4, 1, 0);
        let mut optimized = reference.clone();
        optimized.begin_drawing(X_PIXELS * 4, 1, RENDER_NEW);
        reference.run_line(1);
        optimized.run_line(1);
        let start = EXTRA_LEFT_RIGHT * 4;
        assert_eq!(
            &reference.render_buffer[start..start + 256 * 4],
            &optimized.render_buffer[start..start + 256 * 4]
        );
    }

    #[test]
    fn mode7_upsampling_draws_four_by_four_samples() {
        let mut p = Ppu::new();
        p.mode = 7;
        p.forced_blank = false;
        p.brightness = 15;
        p.mode7_matrix[0] = 0x100;
        p.mode7_matrix[3] = 0x100;
        p.vram.fill(0x0201);
        p.cgram[2] = 31;
        let pitch = X_PIXELS * 16;
        p.begin_drawing(pitch, 4, RENDER_NEW | RENDER_MODE7_4X);
        assert_eq!(p.current_render_scale(p.render_flags), 4);
        p.run_line(1);
        let start = EXTRA_LEFT_RIGHT * 16 + 10 * 16;
        for row in 0..4 {
            for sample in 0..4 {
                let offset = row * pitch + start + sample * 4;
                assert_eq!(&p.render_buffer[offset..offset + 4], &[0, 0, 255, 0]);
            }
        }
    }

    #[test]
    fn mode7_upsampling_allocates_and_draws_the_complete_frame() {
        let mut p = Ppu::new();
        p.mode = 7;
        p.forced_blank = false;
        p.brightness = 15;
        p.extra_left_right = 0;
        p.mode7_matrix[0] = 0x100;
        p.mode7_matrix[3] = 0x100;
        p.vram.fill(0x0201);
        p.cgram[2] = 31;

        let pitch = 256 * 4 * 4;
        p.begin_drawing(pitch, 224, RENDER_NEW | RENDER_MODE7_4X);
        for line in 1..=224 {
            p.run_line(line);
        }

        assert_eq!(p.render_buffer.len(), pitch * 224 * 4);
        let final_row = &p.render_buffer[pitch * 895..pitch * 896];
        assert!(final_row.iter().any(|&byte| byte != 0));
    }
}
