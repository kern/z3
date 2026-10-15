//! WRAM graphics scripts and callbacks; this context cannot borrow VRAM.

use super::*;

/// CPU-side graphics work, including callbacks that can re-enter gameplay.
pub struct GraphicsCpu<'a, A: GfxAssets, H: LoadGfxHooks> {
    pub wram: &'a mut [u8; z3_core::WRAM_SIZE],
    pub assets: &'a A,
    pub hooks: &'a mut H,
}

impl<A: GfxAssets, H: LoadGfxHooks> GraphicsCpu<'_, A, H> {
    fn read8(&self, address: usize) -> u8 {
        self.wram[address]
    }

    fn write8(&mut self, address: usize, value: u8) {
        self.wram[address] = value;
    }

    fn read16(&self, address: usize) -> u16 {
        u16::from_le_bytes([self.wram[address], self.wram[address + 1]])
    }

    fn write16(&mut self, address: usize, value: u16) {
        self.wram[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn increment8(&mut self, address: usize) {
        self.write8(address, self.read8(address).wrapping_add(1));
    }

    fn palette(&self, base: usize, index: usize) -> u16 {
        self.read16(base + index * 2)
    }

    fn set_palette(&mut self, base: usize, index: usize, value: u16) {
        self.write16(base + index * 2, value);
    }

    fn asset_words(&self, asset: usize, offset: usize, count: usize) -> Vec<u16> {
        self.assets.raw(asset)[offset * 2..(offset + count) * 2]
            .chunks_exact(2)
            .map(|x| u16::from_le_bytes([x[0], x[1]]))
            .collect()
    }

    pub fn snes_divide(dividend: u16, divisor: u8) -> u16 {
        if divisor == 0 {
            0xffff
        } else {
            dividend / u16::from(divisor)
        }
    }

    pub fn enable_force_blank(&mut self) {
        self.write8(INIDISP_COPY, 0x80);
        self.write8(HDMAEN_COPY, 0);
    }

    pub fn load_overworld_map_palette(&mut self) {
        let offset = if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            0x80
        } else {
            0
        };
        let colors = self.asset_words(ASSET_OW_MAP_PAL, offset, 128);
        for (i, color) in colors.into_iter().enumerate() {
            self.set_palette(MAIN_PALETTE, i, color);
        }
    }

    pub fn load_item_animation_gfx_one(
        &mut self,
        destination: usize,
        count: usize,
        table_index: usize,
        from_temp: bool,
    ) -> usize {
        let base = if from_temp { 0x14000 } else { 0 };
        let offset = INTRO_LOAD_GFX[table_index] * 24;
        if from_temp {
            expand_pair_from_wram(
                self.wram,
                destination,
                base + offset..0x14600,
                base + offset,
                count,
            );
        } else {
            conversion::expand_pair(
                &mut self.wram[destination..],
                &self.assets.sprite(0)[offset..],
                offset,
                count,
            );
        }
        destination + count * 0x40
    }

    pub fn load_item_gfx_into_wram_4bpp_buffer(&mut self) {
        let mut dst = 0x9480;
        for (count, index) in [(7, 0), (7, 1), (3, 2)] {
            dst = self.load_item_animation_gfx_one(dst, count, index, false);
        }
        self.decomp_spr_to_wram(0x14000, 95);
        for (count, index, temp) in [(4, 3, true), (3, 4, true), (1, 5, true), (4, 6, false)] {
            dst = self.load_item_animation_gfx_one(dst, count, index, temp);
        }
        self.decomp_spr_to_wram(0x14000, 96);
        for (count, index) in [(14, 7), (7, 8)] {
            dst = self.load_item_animation_gfx_one(dst, count, index, true);
        }
        self.decomp_spr_to_wram(0x14000, 95);
        self.load_item_animation_gfx_one(dst, 2, 9, true);
        self.decomp_spr_to_wram(0x14000, 84);
        expand_pair_from_wram(self.wram, 0xa480, 0x14000..0x14600, 0x14000, 8);
        self.decomp_spr_to_wram(0x14000, 96);
        expand_pair_from_wram(self.wram, 0xb280, 0x14000..0x14600, 0x14000, 3);
        self.load_item_gfx_auxiliary();
    }

    pub fn decompress_sword_graphics(&mut self) {
        self.decomp_spr_to_wram(0x14600, 0x5f);
        self.decomp_spr_to_wram(0x14000, 0x5e);
        let offset = usize::from(SWORD_TYPE_TO_GFX_OFFS[usize::from(self.read8(0xf359))]);
        expand_pair_from_wram(
            self.wram,
            0x9000,
            0x14000 + offset..0x15200,
            0x14000 + offset,
            12,
        );
    }

    pub fn decompress_shield_graphics(&mut self) {
        self.decomp_spr_to_wram(0x14600, 0x5f);
        self.decomp_spr_to_wram(0x14000, 0x5e);
        let offset = usize::from(SHIELD_TYPE_TO_GFX_OFFS[usize::from(self.read8(0xf35a))]);
        expand_pair_from_wram(
            self.wram,
            0x9300,
            0x14000 + offset..0x15200,
            0x14000 + offset,
            6,
        );
    }

    pub fn decompress_animated_dungeon_tiles(&mut self, pack: u8) {
        self.decomp_bg_to_wram(0x14000, usize::from(pack));
        self.convert_wram(0xa680, 0x14000, 48, false);
        self.decomp_bg_to_wram(0x14000, 0x5c);
        self.convert_wram(0xac80, 0x14000, 48, false);
        for i in (0..512).step_by(2) {
            let a = self.read16(0xa880 + i);
            let b = self.read16(0xac80 + i);
            let c = self.read16(0xae80 + i);
            let d = self.read16(0xaa80 + i);
            self.write16(0xa880 + i, b);
            self.write16(0xac80 + i, c);
            self.write16(0xae80 + i, d);
            self.write16(0xaa80 + i, a);
        }
        self.write16(0x134, 0x3b00);
    }

    pub fn decompress_animated_overworld_tiles(&mut self, pack: u8) {
        self.decomp_bg_to_wram(0x14000, usize::from(pack));
        self.convert_wram(0xa680, 0x14000, 64, false);
        self.decomp_bg_to_wram(0x14000, usize::from(pack.wrapping_add(1)));
        self.convert_wram(0xae80, 0x14000, 32, false);
        self.write16(0x134, 0x3c00);
    }

    pub fn load_item_gfx_auxiliary(&mut self) {
        self.decomp_bg_to_wram(0x14000, 0xf);
        self.convert_wram(0xb340, 0x14000, 16, false);
        self.decomp_spr_to_wram(0x14000, 0x58);
        self.convert_wram(0xb540, 0x14000, 32, false);
        self.decomp_bg_to_wram(0x14000, 5);
        self.convert_wram(0xbdc0, 0x14480, 2, false);
    }

    pub fn load_follower_graphics(&mut self) {
        let follower = usize::from(self.read8(0xf3cc));
        let pack = if follower == 1 {
            0x64
        } else if follower >= 12 {
            0x58
        } else if follower >= 9 {
            0x59
        } else {
            0x66
        };
        self.decomp_spr_to_wram(0x14600, pack);
        self.decomp_spr_to_wram(0x14000, 0x65);
        let source = 0x14000 + usize::from(TAGALONG_WHICH[follower]);
        self.convert_wram(0xb940, source, 0x20, false);
    }

    pub fn write_to_4bpp_buffer_at_7f4000(&mut self, index: u8) {
        let offset = usize::from(DECODE_ANIMATED_SPRITE_TILE[usize::from(index)]);
        expand_pair_from_wram(
            self.wram,
            0xbd40,
            0x14000 + offset..0x15200,
            0x14000 + offset,
            2,
        );
    }

    pub fn decode_animated_sprite_tile_variable(&mut self, index: u8) {
        let pack = if index == 0x23 || index >= 0x37 {
            0x5d
        } else if index == 0xc || index >= 0x24 {
            0x5c
        } else {
            0x5b
        };
        self.decomp_spr_to_wram(0x14600, pack);
        self.decomp_spr_to_wram(0x14000, 0x5a);
        self.write_to_4bpp_buffer_at_7f4000(index);
    }

    pub fn decomp_spr(&self, dst: &mut [u8], gfx: usize) -> usize {
        decode_sprite(dst, self.assets, gfx)
    }

    pub fn decomp_bg(&self, dst: &mut [u8], gfx: usize) -> usize {
        decompress(dst, self.assets.background(gfx))
    }

    pub fn decomp_spr_to_wram(&mut self, offset: usize, gfx: usize) -> usize {
        decode_sprite(&mut self.wram[offset..], self.assets, gfx)
    }

    pub fn decomp_bg_to_wram(&mut self, offset: usize, gfx: usize) -> usize {
        decompress(&mut self.wram[offset..], self.assets.background(gfx))
    }

    pub fn animate_mirror_warp_decompress_new_tile_sets(&mut self) {
        let main = MAIN_TILESETS[usize::from(self.read8(0xaa1))];
        let aux = AUX_TILESETS[usize::from(self.read8(0xaa2))];
        for i in 0..4 {
            self.write8(0xc2f8 + i, if aux[i] != 0 { aux[i] } else { main[i + 3] });
        }
        let sprites = SPRITE_TILESETS[usize::from(self.read8(0xaa3))];
        for (i, value) in sprites.into_iter().enumerate() {
            if value != 0 {
                self.write8(0xc2fc + i, value);
            }
        }
    }

    pub fn gfx_load_sprites_inner(&mut self, destination: usize) {
        let packs = SPRITE_TILESETS[usize::from(self.read8(0xaa3))];
        for (i, selected) in packs.into_iter().enumerate() {
            if selected != 0 {
                self.write8(0xc2fc + i, selected);
            }
            let pack = usize::from(self.read8(0xc2fc + i));
            let length = self.decomp_spr_to_wram(destination + i * 0x600, pack);
            debug_assert_eq!(length, 0x600);
        }
        self.write8(0x412, 0);
    }

    pub fn load_trans_aux_gfx(&mut self) {
        let packs = AUX_TILESETS[usize::from(self.read8(0xaa2))];
        for (i, pack) in packs.into_iter().enumerate() {
            if pack != 0 {
                self.write8(0xc2f8 + i, pack);
                let length = self.decomp_bg_to_wram(0x6000 + i * 0x600, usize::from(pack));
                debug_assert_eq!(length, 0x600);
            }
        }
        self.gfx_load_sprites_inner(0x7800);
    }

    pub fn load_trans_aux_gfx_sprite(&mut self) {
        self.gfx_load_sprites_inner(0x7800);
    }

    pub fn reload_previously_loaded_sheets(&mut self) {
        for i in 0..4 {
            let pack = usize::from(self.read8(0xc2f8 + i));
            self.decomp_bg_to_wram(0x6000 + i * 0x600, pack);
        }
        for i in 0..4 {
            let pack = usize::from(self.read8(0xc2fc + i));
            self.decomp_spr_to_wram(0x7800 + i * 0x600, pack);
        }
        self.write8(0x412, 0);
    }

    pub fn attract_decompress_story_gfx(&mut self) {
        self.decomp_spr_to_wram(0x14000, 0x67);
        self.decomp_spr_to_wram(0x14800, 0x68);
    }

    pub fn animate_mirror_warp(&mut self) {
        let state = self.read8(0x200);
        self.write8(0x200, state.wrapping_add(1));
        let nmi = MIRROR_WARP_NMI_LOAD[usize::from(state)];
        self.write8(0x17, nmi);
        self.write8(0x710, nmi);
        let xt = if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            8
        } else {
            0
        };
        match state {
            0 => {
                let ctr = self.read8(0x6ba).wrapping_add(1);
                self.write8(0x6ba, ctr);
                if ctr != 32 {
                    self.write8(0x200, 0);
                } else {
                    self.hooks.set_target_overworld_warp_to_pyramid(self.wram);
                }
            }
            1 => {
                self.animate_mirror_warp_decompress_new_tile_sets();
                self.decomp_bg_to_wram(0x14000, usize::from(VARIOUS_PACKS[xt]));
                self.decomp_bg_to_wram(0x14600, usize::from(VARIOUS_PACKS[xt + 1]));
                self.convert_wram(0x10000, 0x14000, 64, true);
                self.convert_wram(0x10800, 0x14600, 64, false);
            }
            2 => {
                self.decomp_bg_to_wram(0x14000, usize::from(VARIOUS_PACKS[xt + 2]));
                self.decomp_bg_to_wram(0x14600, usize::from(VARIOUS_PACKS[xt + 3]));
                self.convert_wram(0x10000, 0x14000, 64, false);
                self.convert_wram(0x10800, 0x14600, 64, true);
            }
            3 => {
                self.decomp_bg_to_wram(0x14000, usize::from(self.read8(0xc2f9)));
                self.decomp_bg_to_wram(0x14600, usize::from(self.read8(0xc2fa)));
                self.convert_wram(0x10000, 0x14000, 128, true);
            }
            4 => {
                self.decomp_bg_to_wram(0x14000, usize::from(VARIOUS_PACKS[xt + 4]));
                self.decomp_bg_to_wram(0x14600, usize::from(VARIOUS_PACKS[xt + 5]));
                self.convert_wram(0x10000, 0x14000, 128, false);
            }
            5 => {
                self.hooks.pre_overworld_load_overlays(self.wram);
                if matches!(self.read8(OVERWORLD_SCREEN), 27 | 91) {
                    self.write8(0x1d, 1);
                }
                self.write8(0x11, self.read8(0x11).wrapping_sub(1));
                self.write8(0x17, 12);
                self.write8(0x710, 12);
            }
            6 | 9 => {
                self.write8(0x17, 13);
                self.write8(0x710, 13);
            }
            7 => {
                self.hooks.overworld_draw_mirror_position(self.wram);
                self.increment8(0x710);
            }
            8 => {
                self.hooks.mirror_warp_load_sprites_and_colors(self.wram);
                self.write8(0x17, 12);
                self.write8(0x710, 12);
            }
            10 => {
                let screen = self.read8(OVERWORLD_SCREEN) & 0xbf;
                self.decompress_animated_overworld_tiles(if matches!(screen, 3 | 5 | 7) {
                    0x58
                } else {
                    0x5a
                });
            }
            11 => {
                let screen = self.read8(OVERWORLD_SCREEN);
                self.write8(
                    0x1d,
                    u8::from(matches!(
                        screen,
                        0 | 0x70 | 0x40 | 0x5b | 3 | 5 | 7 | 0x43 | 0x45 | 0x47
                    )),
                );
                let source = self.assets.sprite(usize::from(VARIOUS_PACKS[xt + 6]));
                do_3_to_4_high_16(&mut self.wram[0x10000..0x10800], source, 64);
            }
            12 => {
                self.decomp_spr_to_wram(0x14000, usize::from(self.read8(0xc2fc)));
                self.decomp_spr_to_wram(0x14600, usize::from(self.read8(0xc2fd)));
                let high = matches!(self.read16(0xc2fc), 0x52 | 0x53 | 0x5a | 0x5b);
                self.convert_wram(0x10000, 0x14000, 64, high);
                self.convert_wram(0x10800, 0x14600, 64, false);
            }
            13 => {
                self.decomp_spr_to_wram(0x14000, usize::from(self.read8(0xc2fe)));
                self.decomp_spr_to_wram(0x14600, usize::from(self.read8(0xc2ff)));
                self.convert_wram(0x10000, 0x14000, 128, false);
                self.hooks.handle_followers_after_mirroring(self.wram);
            }
            14 => self.write8(0x200, 14),
            _ => {}
        }
    }

    pub fn graphics_load_chr_half_slot(&mut self) {
        let mut k = usize::from(self.read8(0xaaa));
        if k == 0 {
            return;
        }
        let sp6 = GRAPHICS_LOAD_SP6[k - 1];
        if sp6 >= 0 {
            self.write8(0xab1, sp6 as u8);
            self.write16(0xaa8, 0x200);
            if k == 1 {
                self.write8(0xab1, 10);
                self.palette_load_sprite_environment();
            } else {
                self.palette_load_sprite_environment_dungeon();
            }
            self.increment8(FLAG_UPDATE_CGRAM);
        }
        let mut tilebytes = 0x44;
        let mut bank_offset = 0;
        self.write8(0xaaa, self.read8(0xaaa).wrapping_add(1));
        if self.read8(0xaaa) & 1 != 0 {
            self.write8(0xaaa, 0);
            if k != 18 {
                bank_offset = 0x300;
                tilebytes = 0x46;
                if k == 2 {
                    self.write8(0x112, 0);
                }
            }
        }
        self.write8(0x116, tilebytes);
        self.write8(0x17, 11);
        k = usize::from(GRAPHICS_HALF_SLOT_PACKS[k - 1]);
        if k == 1 {
            k = usize::from(self.read8(0xaa4));
        }
        let source = &self.assets.sprite(k)[bank_offset..bank_offset + 0x300];
        do_3_to_4_high_16(&mut self.wram[0x11000..0x11400], source, 32);
    }

    pub fn prep_trans_aux_gfx(&mut self) {
        self.convert_wram(0x10000, 0x6000, 0x40, true);
        if self.read8(0xaa2) >= 32 {
            self.convert_wram(0x10800, 0x6600, 0x80, true);
            self.convert_wram(0x11800, 0x7200, 0x40, false);
        } else {
            self.convert_wram(0x10800, 0x6600, 0xc0, false);
        }
    }

    pub fn load_new_sprite_gfx_set(&mut self) {
        self.convert_wram(0x10000, 0x7800, 0xc0, false);
        let pack = self.read8(0xc2ff);
        self.convert_wram(
            0x11800,
            0x8a00,
            0x40,
            matches!(pack, 0x52 | 0x53 | 0x5a | 0x5b),
        );
    }

    fn convert_wram(&mut self, destination: usize, source: usize, count: usize, high: bool) {
        planar_wram(self.wram, destination, source, count, high);
    }

    pub fn apply_palette_filter_bounce(&mut self) {
        let countdown = self.read16(PALETTE_FILTER_COUNTDOWN);
        let table_offset = usize::from(countdown >= 0x10);
        let mask = 0x8000u16 >> (countdown & 15);
        let lighten = self.read16(DARKENING_OR_LIGHTENING) != 0;
        let delta = if lighten { 1u16 } else { u16::MAX };
        let mut j = 0usize;
        loop {
            let a = self.palette(AUX_PALETTE, j);
            let mut c = self.palette(MAIN_PALETTE, j);
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x1f) * 2)] & mask == 0 {
                c = c.wrapping_add(delta);
            }
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x3e0) >> 4)] & mask == 0 {
                c = c.wrapping_add(delta << 5);
            }
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x7c00) >> 9)] & mask == 0 {
                c = c.wrapping_add(delta << 10);
            }
            self.set_palette(MAIN_PALETTE, j, c);
            j += 1;
            j = match j {
                1 => 0x20,
                0xd8 => 0xe0,
                _ => j,
            };
            if j == 0xf0 {
                break;
            }
        }
        self.increment8(FLAG_UPDATE_CGRAM);
        let target = u16::from(self.read8(MOSAIC_TARGET_LEVEL));
        let reached = if !lighten {
            let next = countdown.wrapping_add(1);
            self.write16(PALETTE_FILTER_COUNTDOWN, next);
            next == target
        } else {
            self.write16(PALETTE_FILTER_COUNTDOWN, countdown.wrapping_sub(1));
            countdown == target
        };
        if !reached {
            return;
        }
        self.write16(
            DARKENING_OR_LIGHTENING,
            self.read16(DARKENING_OR_LIGHTENING) ^ 2,
        );
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.increment8(SUBSUBMODULE);
    }

    pub fn palette_filter_range(&mut self, from: usize, to: usize) {
        let countdown = self.read16(PALETTE_FILTER_COUNTDOWN);
        let table_offset = usize::from(countdown >= 0x10);
        let mask = 0x8000u16 >> (countdown & 15);
        let delta = if self.read16(DARKENING_OR_LIGHTENING) != 0 {
            1u16
        } else {
            u16::MAX
        };
        for j in from..to {
            let a = self.palette(AUX_PALETTE, j);
            let mut c = self.palette(MAIN_PALETTE, j);
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x1f) * 2)] & mask == 0 {
                c = c.wrapping_add(delta);
            }
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x3e0) >> 4)] & mask == 0 {
                c = c.wrapping_add(delta << 5);
            }
            if PALETTE_FILTERING_BITS[table_offset + usize::from((a & 0x7c00) >> 9)] & mask == 0 {
                c = c.wrapping_add(delta << 10);
            }
            self.set_palette(MAIN_PALETTE, j, c);
        }
    }

    pub fn palette_filter_incr_countdown(&mut self) {
        let next = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, next);
        if next == 0x1f {
            self.write16(PALETTE_FILTER_COUNTDOWN, 0);
            let mode = self.read16(DARKENING_OR_LIGHTENING) ^ 2;
            self.write16(DARKENING_OR_LIGHTENING, mode);
            // The original increments WORD(link_actual_vel_y) here when mode is nonzero.
            if mode != 0 {
                let v = self.read16(0x27);
                self.write16(0x27, v.wrapping_add(1));
            }
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_restore_additive(&mut self, from: usize, to: usize) {
        for i in from / 2..to / 2 {
            let c = self.palette(MAIN_PALETTE, i);
            let d = self.palette(AUX_PALETTE, i);
            let mut out = c;
            if c & 0x1f != d & 0x1f {
                out = out.wrapping_add(1);
            }
            if c & 0x3e0 != d & 0x3e0 {
                out = out.wrapping_add(0x20);
            }
            if c & 0x7c00 != d & 0x7c00 {
                out = out.wrapping_add(0x400);
            }
            self.set_palette(MAIN_PALETTE, i, out);
        }
    }

    pub fn palette_filter_restore_subtractive(&mut self, from: usize, to: usize) {
        for i in from / 2..to / 2 {
            let c = self.palette(MAIN_PALETTE, i);
            let d = self.palette(AUX_PALETTE, i);
            let mut out = c;
            if c & 0x1f != d & 0x1f {
                out = out.wrapping_sub(1);
            }
            if c & 0x3e0 != d & 0x3e0 {
                out = out.wrapping_sub(0x20);
            }
            if c & 0x7c00 != d & 0x7c00 {
                out = out.wrapping_sub(0x400);
            }
            self.set_palette(MAIN_PALETTE, i, out);
        }
    }

    pub fn reset_hud_palettes_4_and_5(&mut self) {
        for i in 16..24 {
            self.set_palette(MAIN_PALETTE, i, 0);
        }
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.write16(DARKENING_OR_LIGHTENING, 2);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_history(&mut self) {
        self.palette_filter_range(0x10, 0x18);
        self.palette_filter_incr_countdown();
    }

    pub fn palette_filter_wish_ponds(&mut self) {
        self.write8(0x1d, 2);
        self.write8(0x9a, 0x30);
        self.palette_filter_wish_ponds_inner();
    }

    pub fn palette_filter_crystal(&mut self) {
        self.write8(0x1d, 1);
        self.palette_filter_wish_ponds_inner();
    }

    pub fn palette_filter_wish_ponds_inner(&mut self) {
        for i in 0xd0..0xd8 {
            self.set_palette(MAIN_PALETTE, i, 0);
        }
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.write16(DARKENING_OR_LIGHTENING, 2);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_restore_sp5f(&mut self) {
        for i in 208..216 {
            self.set_palette(MAIN_PALETTE, i, self.palette(AUX_PALETTE, i));
        }
        self.write8(0x1d, 0);
        self.write8(0x9a, 32);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_sp5f(&mut self) {
        for _ in 0..2 {
            self.palette_filter_range(208, 216);
            self.palette_filter_incr_countdown();
            if self.read16(PALETTE_FILTER_COUNTDOWN) == 0 {
                break;
            }
        }
    }

    pub fn kholdstare_shell_palette_filtering(&mut self) {
        let features = u32::from_le_bytes(self.wram[FEATURES0..FEATURES0 + 4].try_into().unwrap());
        let start = if features & 4096 != 0 { 0x50 } else { 0x40 };
        if self.read8(SUBSUBMODULE) == 0 {
            for i in start..start + 8 {
                self.set_palette(MAIN_PALETTE, i, self.palette(AUX_PALETTE, i));
            }
            self.write16(PALETTE_FILTER_COUNTDOWN, 0);
            self.write16(DARKENING_OR_LIGHTENING, 0);
            self.increment8(FLAG_UPDATE_CGRAM);
            self.write8(SUBSUBMODULE, 1);
            return;
        }
        for _ in 0..2 {
            self.palette_filter_range(start, start + 8);
            self.palette_filter_incr_countdown();
            if self.read16(PALETTE_FILTER_COUNTDOWN) == 0 {
                self.write8(0x1d, 0);
                break;
            }
        }
    }

    pub fn agahnim_warp_shadow_filter(&mut self, index: usize, settings_addr: usize) {
        self.write16(
            PALETTE_FILTER_COUNTDOWN,
            self.read16(settings_addr + index * 2),
        );
        self.write16(
            DARKENING_OR_LIGHTENING,
            self.read16(settings_addr + (index + 3) * 2),
        );
        let start = usize::from(PALETTE_FILTER_AGAHNIM[index] >> 1);
        for _ in 0..2 {
            self.palette_filter_range(start, start + 8);
            let next = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
            self.write16(PALETTE_FILTER_COUNTDOWN, next);
            if next == 0x1f {
                self.write16(PALETTE_FILTER_COUNTDOWN, 0);
                self.write16(
                    DARKENING_OR_LIGHTENING,
                    self.read16(DARKENING_OR_LIGHTENING) ^ 2,
                );
                break;
            }
        }
        self.write16(
            settings_addr + index * 2,
            self.read16(PALETTE_FILTER_COUNTDOWN),
        );
        self.write16(
            settings_addr + (index + 3) * 2,
            self.read16(DARKENING_OR_LIGHTENING),
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_fade_intro_one_step(&mut self) {
        self.palette_filter_restore_additive(0x100, 0x1a0);
        self.palette_filter_restore_additive(0xc0, 0x100);
        self.write8(
            PALETTE_FILTER_COUNTDOWN,
            self.read8(PALETTE_FILTER_COUNTDOWN).wrapping_sub(1),
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_fade_intro_2(&mut self) {
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.write8(
            PALETTE_FILTER_COUNTDOWN,
            self.read8(PALETTE_FILTER_COUNTDOWN).wrapping_sub(1),
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_initialize_white_filter(&mut self) {
        for i in 0..256 {
            self.set_palette(AUX_PALETTE, i, 0x7fff);
        }
        self.set_palette(MAIN_PALETTE, 32, self.palette(MAIN_PALETTE, 0));
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.write16(DARKENING_OR_LIGHTENING, 2);
        if self.read16(OVERWORLD_SCREEN) == 27 {
            for i in [0, 32] {
                self.set_palette(AUX_PALETTE, i, 0);
                self.set_palette(MAIN_PALETTE, i, 0);
            }
        }
        self.write8(0x6bb, 8); // mirror_vars.ctr
        self.write8(0x6ba, 0); // mirror_vars.ctr2
    }

    pub fn mirror_warp_run_animation_submodules(&mut self) {
        let counter = self.read8(0x6bb).wrapping_sub(1);
        self.write8(0x6bb, counter);
        if counter != 0 {
            self.animate_mirror_warp();
            return;
        }
        self.write8(0x6bb, 2);
        self.palette_filter_blinding_white();
    }

    pub fn palette_filter_blinding_white(&mut self) {
        if self.read16(DARKENING_OR_LIGHTENING) == 0xff {
            return;
        }
        if self.read16(DARKENING_OR_LIGHTENING) == 2 {
            self.palette_filter_restore_additive(0x40, 0x1b0);
            self.palette_filter_restore_additive(0x1c0, 0x1e0);
        } else {
            self.palette_filter_restore_subtractive(0x40, 0x1b0);
            self.palette_filter_restore_subtractive(0x1c0, 0x1e0);
        }
        self.palette_filter_start_blinding_white();
    }

    pub fn palette_filter_start_blinding_white(&mut self) {
        self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
        let next = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, next);
        if self.read16(DARKENING_OR_LIGHTENING) == 0 {
            if next == 66 {
                self.write16(DARKENING_OR_LIGHTENING, 0xff);
                self.write8(0x6bb, 32);
            }
        } else if next == 31 {
            self.write16(
                DARKENING_OR_LIGHTENING,
                self.read16(DARKENING_OR_LIGHTENING) ^ 2,
            );
            if self.read8(0x10) != 21 {
                return;
            }
            self.hooks.ppu_write(0x0c, 0);
            self.write8(HDMAEN_COPY, 0);
            for i in 0..240 {
                self.write16(HDMA_TABLE_DYNAMIC + i * 2, 0x778);
            }
            self.write8(HDMAEN_COPY, 0xc0);
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_blinding_white_triforce(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x200);
        self.palette_filter_start_blinding_white();
    }

    pub fn trinexx_flash_shell_palette_red(&mut self) {
        self.trinexx_red(true);
    }

    pub fn trinexx_unflash_shell_palette_red(&mut self) {
        self.trinexx_red(false);
    }

    fn trinexx_red(&mut self, flash: bool) {
        if self.read8(0x4be) == 0 {
            for i in 0x41..0x48 {
                let target = self.palette(AUX_PALETTE, i) & 0x1f;
                let value = self.palette(MAIN_PALETTE, i);
                let component = value & 0x1f;
                let next = if flash {
                    component + u16::from(component != 0x1f)
                } else {
                    component - u16::from(component != target)
                };
                self.set_palette(MAIN_PALETTE, i, value & 0xffe0 | next);
            }
            self.increment8(FLAG_UPDATE_CGRAM);
            let count = self.read8(0x4c0).wrapping_add(1);
            self.write8(0x4c0, count);
            if count >= 12 {
                self.write8(0x4c0, 0);
                self.write8(0x4be, 0);
                return;
            }
            self.write8(0x4be, 3);
        }
        self.write8(0x4be, self.read8(0x4be).wrapping_sub(1));
    }

    pub fn trinexx_flash_shell_palette_blue(&mut self) {
        self.trinexx_blue(true);
    }

    pub fn trinexx_unflash_shell_palette_blue(&mut self) {
        self.trinexx_blue(false);
    }

    fn trinexx_blue(&mut self, flash: bool) {
        if self.read8(0x4bf) == 0 {
            for i in 0x41..0x48 {
                let target = self.palette(AUX_PALETTE, i) & 0x7c00;
                let value = self.palette(MAIN_PALETTE, i);
                let component = value & 0x7c00;
                let next = if flash {
                    component + (u16::from(component != 0x7c00) << 10)
                } else {
                    component - (u16::from(component != target) << 10)
                };
                self.set_palette(MAIN_PALETTE, i, value & !0x7c00 | next);
            }
            self.increment8(FLAG_UPDATE_CGRAM);
            let count = self.read8(0x4c1).wrapping_add(1);
            self.write8(0x4c1, count);
            if count >= 12 {
                self.write8(0x4c1, 0);
                self.write8(0x4bf, 0);
                return;
            }
            self.write8(0x4bf, 3);
        }
        self.write8(0x4bf, self.read8(0x4bf).wrapping_sub(1));
    }

    pub fn palette_filter_restore_bg_subtractive_strict(&mut self) {
        if self.read16(DARKENING_OR_LIGHTENING) == 255 {
            return;
        }
        self.palette_filter_restore_subtractive(0x40, 0x100);
        let next = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, next);
        if next == 0x20 {
            self.write16(DARKENING_OR_LIGHTENING, 255);
            self.write16(0x1d, 0);
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_restore_bg_additive_strict(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x100);
        self.write16(
            PALETTE_FILTER_COUNTDOWN,
            self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1),
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn filter_majorly_whiten_color(&self, color: u16) -> u16 {
        let features = u32::from_le_bytes(self.wram[FEATURES0..FEATURES0 + 4].try_into().unwrap());
        let amount = if features & 65_536 != 0 { 3 } else { 14 };
        let red = (usize::from(color & 0x1f) + amount).min(0x1f) as u16;
        let green = (usize::from(color & 0x3e0) + (amount << 5)).min(0x3e0) as u16;
        let blue = (usize::from(color & 0x7c00) + (amount << 10)).min(0x7c00) as u16;
        red | green | blue
    }

    pub fn filter_majorly_whiten_bg(&mut self) {
        for i in 32..128 {
            let color = self.filter_majorly_whiten_color(self.palette(AUX_PALETTE, i));
            self.set_palette(MAIN_PALETTE, i, color);
        }
        let backdrop = if self.palette(AUX_PALETTE, 0) != 0 {
            self.palette(MAIN_PALETTE, 32)
        } else {
            0
        };
        self.set_palette(MAIN_PALETTE, 0, backdrop);
    }

    pub fn palette_set_bg_and_fixed_color(&mut self, color: u16) {
        for base in [MAIN_PALETTE, AUX_PALETTE] {
            self.set_palette(base, 0, color);
            self.set_palette(base, 32, color);
        }
        self.set_backdrop_color_black();
    }

    pub fn palette_bg_and_fixed_color_black(&mut self) {
        self.palette_set_bg_and_fixed_color(0);
    }

    pub fn set_backdrop_color_black(&mut self) {
        self.write8(0x9c, 0x20);
        self.write8(0x9d, 0x40);
        self.write8(0x9e, 0x80);
    }

    pub fn palette_get_ow_bg_color(&self) -> u16 {
        let screen = self.read16(OVERWORLD_SCREEN);
        if screen < 0x80 {
            return if screen & 0x40 != 0 { 0x2a32 } else { 0x2669 };
        }
        match self.read16(DUNGEON_ROOM) {
            0x180 | 0x182 | 0x183 => 0x19c6,
            _ => 0x2669,
        }
    }

    pub fn palette_set_ow_bg_color(&mut self) {
        let color = self.palette_get_ow_bg_color();
        self.palette_set_bg_and_fixed_color(color);
    }

    pub fn palette_special_ow(&mut self) {
        let color = self.palette_get_ow_bg_color();
        self.set_palette(AUX_PALETTE, 0, color);
        self.set_palette(AUX_PALETTE, 32, color);
        self.set_backdrop_color_black();
    }

    pub fn palette_set_translucency_swap(&mut self, value: bool) {
        self.write8(PALETTE_SWAP_FLAG, u8::from(value));
        for i in 0..8 {
            for (a, b) in [(0x80, 0xf0), (0x88, 0xf8), (0xb8, 0xd8)] {
                let ca = self.palette(AUX_PALETTE, i + a);
                let cb = self.palette(AUX_PALETTE, i + b);
                self.set_palette(AUX_PALETTE, i + b, ca);
                self.set_palette(MAIN_PALETTE, i + b, ca);
                self.set_palette(AUX_PALETTE, i + a, cb);
                self.set_palette(MAIN_PALETTE, i + a, cb);
            }
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_assert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(true);
    }

    pub fn palette_revert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(false);
    }

    pub fn overworld_copy_palettes_to_cache(&mut self) {
        self.wram
            .copy_within(AUX_PALETTE..AUX_PALETTE + 512, MAIN_PALETTE);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn special_overworld_copy_palettes_to_cache(&mut self) {
        for i in 32..256 {
            self.set_palette(MAIN_PALETTE, i, 0);
        }
        for i in 0..8 {
            for offset in [0, 8, 0x10, 0x18, 0xd8, 0xe8, 0xf0, 0xf8] {
                self.set_palette(
                    MAIN_PALETTE,
                    i + offset,
                    self.palette(AUX_PALETTE, i + offset),
                );
            }
        }
        self.write8(MOSAIC_COPY, 0xf7);
        self.write8(MOSAIC_LEVEL, 0xf7);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_restore_coldata(&mut self) {
        if self.read8(0x1b) != 0 {
            return;
        }
        let rgb = match self.read8(OVERWORLD_SCREEN) {
            3 | 5 | 7 => 0x8c4c26u32,
            0x43 | 0x45 | 0x47 => 0x874a26,
            0x5b => 0x894f33,
            _ => 0x804020,
        };
        self.write8(0x9c, rgb as u8);
        self.write8(0x9d, (rgb >> 8) as u8);
        self.write8(0x9e, (rgb >> 16) as u8);
    }

    pub fn palette_restore_bg_from_flash(&mut self) {
        self.wram
            .copy_within(AUX_PALETTE + 64..AUX_PALETTE + 256, MAIN_PALETTE + 64);
        self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
        self.palette_restore_coldata();
    }

    pub fn palette_restore_bg_and_hud(&mut self) {
        self.wram
            .copy_within(AUX_PALETTE..AUX_PALETTE + 256, MAIN_PALETTE);
        self.increment8(FLAG_UPDATE_CGRAM);
        self.palette_restore_coldata();
    }

    pub fn palette_load_single(&mut self, source: &[u16], destination: usize, entries: usize) {
        let destination = (destination + usize::from(self.read16(0xaa8))) / 2;
        for (i, &color) in source[..=entries].iter().enumerate() {
            self.set_palette(AUX_PALETTE, destination + i, color);
        }
    }

    pub fn palette_load_multiple(
        &mut self,
        source: &[u16],
        destination: usize,
        entries: usize,
        palettes: usize,
    ) {
        let width = entries + 1;
        for row in 0..=palettes {
            let destination = (destination + row * 0x20 + usize::from(self.read16(0xaa8))) / 2;
            for (i, &color) in source[row * width..(row + 1) * width].iter().enumerate() {
                self.set_palette(AUX_PALETTE, destination + i, color);
            }
        }
    }

    pub fn palette_load_multiple_arbitrary(
        &mut self,
        source: &[u16],
        destination: usize,
        entries: usize,
    ) {
        self.load_gear_palette(destination, source, entries + 1);
    }

    pub fn load_gear_palette(&mut self, destination: usize, source: &[u16], entries: usize) {
        for (i, &color) in source[..entries].iter().enumerate() {
            self.set_palette(AUX_PALETTE, destination / 2 + i, color);
            self.set_palette(MAIN_PALETTE, destination / 2 + i, color);
        }
    }

    fn load_asset_single(
        &mut self,
        asset: usize,
        offset: usize,
        destination: usize,
        entries: usize,
    ) {
        let colors = self.asset_words(asset, offset, entries + 1);
        self.palette_load_single(&colors, destination, entries);
    }

    fn load_asset_multiple(
        &mut self,
        asset: usize,
        offset: usize,
        destination: usize,
        entries: usize,
        palettes: usize,
    ) {
        let colors = self.asset_words(asset, offset, (entries + 1) * (palettes + 1));
        self.palette_load_multiple(&colors, destination, entries, palettes);
    }

    fn load_asset_arbitrary(
        &mut self,
        asset: usize,
        offset: usize,
        destination: usize,
        entries: usize,
    ) {
        let colors = self.asset_words(asset, offset, entries + 1);
        self.palette_load_multiple_arbitrary(&colors, destination, entries);
    }

    pub fn palette_load_sp0l(&mut self) {
        let offset = usize::from(self.read8(0xaac)) * 7;
        let destination = if self.read8(PALETTE_SWAP_FLAG) != 0 {
            0x1e2
        } else {
            0x102
        };
        self.load_asset_single(ASSET_PAL_SPR_AUX3, offset, destination, 6);
    }

    pub fn palette_load_sprite_main(&mut self) {
        let offset = if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            60
        } else {
            0
        };
        self.load_asset_multiple(ASSET_PAL_MAIN_SPR, offset, 0x122, 14, 3);
    }

    pub fn palette_load_sp5l(&mut self) {
        self.load_asset_single(
            ASSET_PAL_SPR_AUX1,
            usize::from(self.read8(0xaad)) * 7,
            0x1a2,
            6,
        );
    }

    pub fn palette_load_sp6l(&mut self) {
        self.load_asset_single(
            ASSET_PAL_SPR_AUX1,
            usize::from(self.read8(0xaae)) * 7,
            0x1c2,
            6,
        );
    }

    pub fn palette_load_sword(&mut self) {
        let sword = self.read8(0xf359);
        let offset = if sword as i8 > 0 {
            usize::from(sword - 1) * 3
        } else {
            0
        };
        self.load_asset_arbitrary(ASSET_PAL_SWORD, offset, 0x1b2, 2);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_load_shield(&mut self) {
        let shield = self.read8(0xf35a);
        self.load_asset_arbitrary(
            ASSET_PAL_SHIELD,
            usize::from(shield.saturating_sub(1)) * 4,
            0x1b8,
            3,
        );
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_load_sprite_environment(&mut self) {
        if self.read8(0x1b) != 0 {
            self.palette_load_sprite_environment_dungeon();
        } else {
            self.palette_misc_sprite_outdoors();
        }
    }

    pub fn palette_load_sprite_environment_dungeon(&mut self) {
        self.load_asset_single(
            ASSET_PAL_MISC_SPR,
            usize::from(self.read8(0xab1)) * 7,
            0x1d2,
            6,
        );
    }

    pub fn palette_misc_sprite_outdoors(&mut self) {
        let index = if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            9
        } else {
            7
        };
        let destination = if self.read8(PALETTE_SWAP_FLAG) != 0 {
            0x1f2
        } else {
            0x112
        };
        self.load_asset_single(ASSET_PAL_MISC_SPR, index * 7, destination, 6);
        self.load_asset_single(ASSET_PAL_MISC_SPR, (index - 1) * 7, 0x1d2, 6);
    }

    pub fn palette_load_dungeon_map_sprite(&mut self) {
        self.load_asset_multiple(ASSET_PAL_MAP_SPR, 0, 0x182, 6, 2);
    }

    pub fn palette_load_link_armor_and_gloves(&mut self) {
        self.load_asset_arbitrary(
            ASSET_PAL_ARMOR_GLOVES,
            usize::from(self.read8(0xf35b)) * 15,
            0x1e2,
            14,
        );
        self.palette_update_gloves_color();
    }

    pub fn palette_update_gloves_color(&mut self) {
        let gloves = self.read8(0xf354);
        if gloves != 0 {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(
                    base,
                    0xfd,
                    self.assets.gloves_color(usize::from(gloves - 1)),
                );
            }
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_load_dungeon_map_bg(&mut self) {
        self.load_asset_multiple(ASSET_PAL_MAP_BG, 0, 0x40, 15, 5);
    }

    pub fn palette_load_hud(&mut self) {
        self.load_asset_multiple(ASSET_HUD_PAL, usize::from(self.read8(0xab2)) * 32, 0, 15, 1);
    }

    pub fn palette_load_dungeon_set(&mut self) {
        let offset = usize::from(self.read8(0xab6) >> 1) * 90;
        self.load_asset_multiple(ASSET_PAL_DUNG_BG_MAIN, offset, 0x42, 14, 5);
        let destination = if self.read8(PALETTE_SWAP_FLAG) != 0 {
            0x1f2
        } else {
            0x112
        };
        self.load_asset_single(ASSET_PAL_DUNG_BG_MAIN, offset, destination, 6);
    }

    pub fn palette_load_owbg3(&mut self) {
        self.load_asset_single(
            ASSET_PAL_OW_AUX3,
            usize::from(self.read8(0xab8)) * 7,
            0xe2,
            6,
        );
    }

    pub fn palette_load_owbg_main(&mut self) {
        self.load_asset_multiple(
            ASSET_PAL_OW_MAIN,
            usize::from(self.read8(0xab3)) * 35,
            0x42,
            6,
            4,
        );
    }

    pub fn palette_load_owbg1(&mut self) {
        self.load_asset_multiple(
            ASSET_PAL_OW_AUX12,
            usize::from(self.read8(0xab4)) * 21,
            0x52,
            6,
            2,
        );
    }

    pub fn palette_load_owbg2(&mut self) {
        self.load_asset_multiple(
            ASSET_PAL_OW_AUX12,
            usize::from(self.read8(0xab5)) * 21,
            0xb2,
            6,
            2,
        );
    }

    pub fn load_gear_palettes(&mut self, sword: u8, shield: u8, armor: u8) {
        let sword_offset = if sword != 0 && sword != 255 {
            usize::from(sword - 1) * 3
        } else {
            0
        };
        self.load_asset_arbitrary(ASSET_PAL_SWORD, sword_offset, 0x1b2, 2);
        self.load_asset_arbitrary(
            ASSET_PAL_SHIELD,
            usize::from(shield.saturating_sub(1)) * 4,
            0x1b8,
            3,
        );
        self.load_asset_arbitrary(ASSET_PAL_ARMOR_GLOVES, usize::from(armor) * 15, 0x1e2, 14);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn load_actual_gear_palettes(&mut self) {
        self.load_gear_palettes(self.read8(0xf359), self.read8(0xf35a), self.read8(0xf35b));
        let features = u32::from_le_bytes(self.wram[FEATURES0..FEATURES0 + 4].try_into().unwrap());
        if features & 4096 != 0 {
            self.palette_update_gloves_color();
        }
    }

    pub fn palette_electro_themed_gear(&mut self) {
        self.load_gear_palettes(2, 2, 4);
    }

    pub fn load_gear_palettes_bunny(&mut self) {
        self.load_gear_palettes(self.read8(0xf359), self.read8(0xf35a), 3);
    }

    pub fn palette_load_agahnim(&mut self) {
        for destination in [0x162, 0x182, 0x1a2] {
            self.load_asset_arbitrary(ASSET_PAL_SPR_AUX1, 14 * 7, destination, 6);
        }
        self.load_asset_arbitrary(ASSET_PAL_SPR_AUX1, 21 * 7, 0x1c2, 6);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_load_for_file_select(&mut self) {
        for slot in 0..3 {
            let (armor, gloves, sword, shield) = self.hooks.file_select_gear(slot);
            let offset = slot * 0x20;
            self.palette_load_for_file_select_armor(offset, armor, gloves);
            self.palette_load_for_file_select_sword(offset, sword);
            self.palette_load_for_file_select_shield(offset, shield);
        }
        let main = self.asset_words(ASSET_PAL_MAIN_SPR, 0, 30);
        for i in 0..7 {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(base, 0xe8 + i, main[7 + i]);
                self.set_palette(base, 0xf8 + i, main[22 + i]);
            }
        }
    }

    pub fn palette_load_for_file_select_armor(&mut self, offset: usize, armor: u8, gloves: u8) {
        let colors = self.asset_words(ASSET_PAL_ARMOR_GLOVES, usize::from(armor) * 15, 15);
        for (i, color) in colors.into_iter().enumerate() {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(base, offset + 0x81 + i, color);
            }
        }
        if gloves != 0 {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(
                    base,
                    offset + 0x8d,
                    self.assets.gloves_color(usize::from(gloves - 1)),
                );
            }
        }
    }

    pub fn palette_load_for_file_select_sword(&mut self, offset: usize, sword: u8) {
        let colors = self.asset_words(ASSET_PAL_SWORD, usize::from(sword.saturating_sub(1)) * 3, 3);
        for (i, color) in colors.into_iter().enumerate() {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(base, offset + 0x99 + i, color);
            }
        }
    }

    pub fn palette_load_for_file_select_shield(&mut self, offset: usize, shield: u8) {
        let colors = self.asset_words(
            ASSET_PAL_SHIELD,
            usize::from(shield.saturating_sub(1)) * 4,
            4,
        );
        for (i, color) in colors.into_iter().enumerate() {
            for base in [AUX_PALETTE, MAIN_PALETTE] {
                self.set_palette(base, offset + 0x9c + i, color);
            }
        }
    }

    pub fn overworld_load_palettes(&mut self, bg: u8, sprites: u8) {
        self.write16(0xaa8, 0);
        let bg_offset = usize::from(bg) * 3;
        // The credits scene requests palette 167; the C table access runs into
        // adjacent static data. Preserve the observed C oracle bytes here.
        let bg_info = if bg == 167 {
            [0, -49, -67]
        } else {
            [
                OW_BG_PAL_INFO.get(bg_offset).copied().unwrap_or(-1),
                OW_BG_PAL_INFO.get(bg_offset + 1).copied().unwrap_or(-1),
                OW_BG_PAL_INFO.get(bg_offset + 2).copied().unwrap_or(-1),
            ]
        };
        for (address, value) in [
            (0xab4, bg_info[0]),
            (0xab5, bg_info[1]),
            (0xab8, bg_info[2]),
        ] {
            if value >= 0 {
                self.write8(address, value as u8);
            }
        }
        let sprite_offset = usize::from(sprites) * 2;
        for (address, value) in [
            (0xaad, OW_SPR_PAL_INFO[sprite_offset]),
            (0xaae, OW_SPR_PAL_INFO[sprite_offset + 1]),
        ] {
            if value >= 0 {
                self.write8(address, value as u8);
            }
        }
        self.palette_load_owbg1();
        self.palette_load_owbg2();
        self.palette_load_owbg3();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
    }

    pub fn overworld_load_all_palettes(&mut self) {
        self.wram[AUX_PALETTE + 0x180..AUX_PALETTE + 0x200].fill(0);
        self.wram[MAIN_PALETTE..MAIN_PALETTE + 512].fill(0);
        for (address, value) in [
            (0xab3, 5),
            (0xab4, 3),
            (0xab5, 3),
            (0xab8, 0),
            (0xab1, 5),
            (0xaac, 11),
            (PALETTE_SWAP_FLAG, 0),
        ] {
            self.write8(address, value);
        }
        self.write16(0xaa8, 0);
        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_owbg_main();
        self.palette_load_owbg1();
        self.palette_load_owbg2();
        self.palette_load_owbg3();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();
        for i in 0..8 {
            self.set_palette(MAIN_PALETTE, 0xd8 + i, self.palette(AUX_PALETTE, 0xe8 + i));
        }
    }

    pub fn dungeon_load_palettes(&mut self) {
        self.write16(0xaa8, 0);
        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_sprite_environment();
        self.palette_load_link_armor_and_gloves();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
        self.overworld_load_palettes_inner();
    }

    pub fn overworld_load_palettes_inner(&mut self) {
        self.write8(0xc20a, self.read8(0xab6));
        self.write8(0xc20b, self.read8(0xab8));
        self.write8(0xc20c, self.read8(0xab7));
        self.write16(DARKENING_OR_LIGHTENING, 2);
        self.write16(PALETTE_FILTER_COUNTDOWN, 0);
        self.write16(MOSAIC_TARGET_LEVEL, 0);
        self.overworld_copy_palettes_to_cache();
    }

    pub fn overworld_load_screens_palette_set(&mut self) {
        let screen = self.read8(OVERWORLD_SCREEN) & 0x3f;
        let mut index = if matches!(screen, 3 | 5 | 7) { 2 } else { 0 };
        if self.read16(OVERWORLD_SCREEN) & 0x40 != 0 {
            index += 1;
        }
        self.overworld_load_area_palettes_ex(index);
    }

    pub fn overworld_load_area_palettes_ex(&mut self, index: u8) {
        self.write8(0xab3, index);
        self.write16(0xaa8, self.read16(0xaa8) & 0xff);
        self.palette_load_sprite_main();
        self.palette_load_sprite_environment();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_link_armor_and_gloves();
        self.write8(0xaac, if self.read8(0xf3ca) & 0x40 != 0 { 3 } else { 1 });
        self.palette_load_sp0l();
        self.palette_load_hud();
        self.palette_load_owbg_main();
    }

    pub fn dungeon_handle_translucency_and_palette(&mut self) {
        if self.read8(PALETTE_SWAP_FLAG) != 0 {
            self.palette_revert_translucency_swap();
        }
        self.write8(0x99, 2);
        self.write8(0x9a, 0xb3);
        let mut torch = self.read8(0x45a);
        if self.read8(0xc005) == 0 {
            let property = self.read8(0x414);
            let mut color_math = match property {
                7 => 0x32,
                4 => 0x62,
                _ => 0x20,
            };
            if property == 2 {
                self.palette_assert_translucency_swap();
                if self.read8(DUNGEON_ROOM) == 13 {
                    self.wram[0xc019..0xc025].fill(0);
                    self.palette_load_agahnim();
                }
                color_math = 0x70;
            }
            self.write8(0x9a, color_math);
            torch = 3;
        }
        self.write8(0xc017, LIT_TORCHES_COLOR_PLUS[usize::from(torch)]);
        self.write16(PALETTE_FILTER_COUNTDOWN, 31);
        self.write8(MOSAIC_TARGET_LEVEL, 0);
        self.write16(DARKENING_OR_LIGHTENING, 2);
        self.write16(0xaa8, 0);
        self.palette_load_dungeon_set();
        self.palette_load_sp0l();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.increment8(SUBSUBMODULE);
    }

    pub fn handle_screen_flash(&mut self) {
        let count = self.read8(0xff9);
        if count == 0 || self.read8(0x11) != 0 {
            return;
        }
        self.write8(0xff9, count - 1);
        if count == 1 {
            self.palette_restore_bg_and_hud();
            return;
        }
        if count & 1 != 0 {
            self.filter_majorly_whiten_bg();
        } else {
            self.palette_restore_bg_from_flash();
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_whirlpool_blue(&mut self) {
        if self.read8(0x1a) & 1 != 0 {
            for i in 0x20..0x100 {
                let mut color = self.palette(MAIN_PALETTE, i);
                if color & 0x7c00 != 0x7c00 {
                    color = color.wrapping_add(0x400);
                }
                self.set_palette(MAIN_PALETTE, i, color);
            }
            self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
            let countdown = self.read16(PALETTE_FILTER_COUNTDOWN);
            if countdown & 1 == 0 {
                self.write8(MOSAIC_LEVEL, self.read8(MOSAIC_LEVEL).wrapping_add(16));
            }
            self.write16(PALETTE_FILTER_COUNTDOWN, countdown.wrapping_add(1));
            if countdown.wrapping_add(1) == 31 {
                self.write16(PALETTE_FILTER_COUNTDOWN, 0);
                self.increment8(SUBSUBMODULE);
                self.write8(MOSAIC_LEVEL, 0xf0);
            }
        }
        self.write8(BGMODE_COPY, 9);
        self.write8(MOSAIC_COPY, self.read8(MOSAIC_LEVEL) | 3);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_isolate_whirlpool_blue(&mut self) {
        for i in 0x20..0x100 {
            let mut color = self.palette(MAIN_PALETTE, i);
            if color & 0x3e0 != 0 {
                color = color.wrapping_sub(0x20);
            }
            if color & 0x1f != 0 {
                color = color.wrapping_sub(1);
            }
            self.set_palette(MAIN_PALETTE, i, color);
        }
        self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
        let countdown = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, countdown);
        if countdown == 31 {
            self.write16(PALETTE_FILTER_COUNTDOWN, 0);
            self.increment8(SUBSUBMODULE);
            self.write8(MOSAIC_LEVEL, 0xf0);
        }
        self.write8(BGMODE_COPY, 9);
        self.write8(MOSAIC_COPY, self.read8(MOSAIC_LEVEL) | 3);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_whirlpool_restore_blue(&mut self) {
        if self.read8(0x1a) & 1 != 0 {
            for i in 0x20..0x100 {
                let target = self.palette(AUX_PALETTE, i) & 0x7c00;
                let mut color = self.palette(MAIN_PALETTE, i);
                if color & 0x7c00 != target {
                    color = color.wrapping_sub(0x400);
                }
                self.set_palette(MAIN_PALETTE, i, color);
            }
            self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
            let countdown = self.read16(PALETTE_FILTER_COUNTDOWN);
            if countdown & 1 == 0 {
                self.write8(MOSAIC_LEVEL, self.read8(MOSAIC_LEVEL).wrapping_sub(16));
            }
            self.write16(PALETTE_FILTER_COUNTDOWN, countdown.wrapping_add(1));
            if countdown.wrapping_add(1) == 31 {
                self.write16(PALETTE_FILTER_COUNTDOWN, 0);
                self.increment8(SUBSUBMODULE);
                self.write8(MOSAIC_LEVEL, 0);
            }
        }
        self.write8(BGMODE_COPY, 9);
        self.write8(MOSAIC_COPY, self.read8(MOSAIC_LEVEL) | 3);
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn palette_filter_whirlpool_restore_red_green(&mut self) {
        for i in 0x20..0x100 {
            let target = self.palette(AUX_PALETTE, i);
            let mut color = self.palette(MAIN_PALETTE, i);
            if color & 0x3e0 != target & 0x3e0 {
                color = color.wrapping_add(0x20);
            }
            if color & 0x1f != target & 0x1f {
                color = color.wrapping_add(1);
            }
            self.set_palette(MAIN_PALETTE, i, color);
        }
        self.set_palette(MAIN_PALETTE, 0, self.palette(MAIN_PALETTE, 32));
        let countdown = self.read16(PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        self.write16(PALETTE_FILTER_COUNTDOWN, countdown);
        if countdown == 31 {
            self.write16(PALETTE_FILTER_COUNTDOWN, 0);
            self.increment8(SUBSUBMODULE);
        }
        self.increment8(FLAG_UPDATE_CGRAM);
    }

    pub fn iris_spotlight_calculate_circle_value(&self, radius: u8) -> u16 {
        let spotlight_radius = self.read16(0x67c) as u8;
        let index = (Self::snes_divide(u16::from(radius) << 8, spotlight_radius) >> 1) as u8;
        let curve = SPOTLIGHT_CIRCLE[usize::from(index)];
        let scaled = ((u16::from(curve) * u16::from(spotlight_radius)) >> 8) as u8;
        let spread = 2 * u16::from(scaled);
        if curve == 0 {
            return 0xff;
        }
        let center = self.read16(0x670);
        let left = center.wrapping_sub(spread);
        let left = if (left as i16) < 0 { 0 } else { left.min(255) };
        let right = center.wrapping_add(spread).min(255);
        let value = left | right << 8;
        if value == 0xffff { 0xff } else { value }
    }

    pub fn iris_spotlight_reset_table(&mut self) {
        for i in 0..240 {
            self.write16(HDMA_TABLE_DYNAMIC + i * 2, 0xff00);
        }
    }

    pub fn iris_spotlight_close(&mut self) {
        self.spotlight_internal(0x7e, 0);
    }

    pub fn spotlight_open(&mut self) {
        self.spotlight_internal(0, 2);
    }

    pub fn spotlight_internal(&mut self, radius: u8, mode: u8) {
        self.write16(0x67c, u16::from(radius));
        self.write16(0x67e, u16::from(mode));
        self.hooks.ppu_write(0x0c, 0);
        self.hooks.setup_hdma(0xf2fb, 0, 0x26);
        self.write8(0x96, 0x33);
        self.write8(0x97, 3);
        self.write8(0x98, 0x33);
        self.write8(0x1e, self.read8(0x1c));
        self.write8(0x1f, self.read8(0x1d));
        if self.read8(0x1b) == 0 {
            self.set_backdrop_color_black();
        }
        self.iris_spotlight_configure_table();
        self.write8(HDMAEN_COPY, 0x80);
        self.write8(INIDISP_COPY, 0xf);
    }

    pub fn iris_spotlight_configure_table(&mut self) {
        let center_y = self
            .read16(0x20)
            .wrapping_sub(self.read16(0xe8))
            .wrapping_add(12);
        let radius = self.read16(0x67c);
        self.write16(0x674, center_y.wrapping_sub(radius));
        self.write16(0x676, center_y.wrapping_add(radius));
        self.write16(
            0x670,
            self.read16(0x22)
                .wrapping_sub(self.read16(0xe2))
                .wrapping_add(8),
        );
        self.write16(0x67a, radius);
        let mut lower = center_y.wrapping_mul(2);
        if lower < 224 {
            lower = 224;
        }
        let mut upper = center_y.wrapping_mul(2).wrapping_sub(lower);
        loop {
            let mut value = 0xff;
            if lower < self.read16(0x676) {
                let r = self.read8(0x67a);
                if self.read16(0x67a) != 0 {
                    self.write16(0x67a, self.read16(0x67a).wrapping_sub(1));
                }
                value = self.iris_spotlight_calculate_circle_value(r);
            }
            if upper < 240 {
                self.write16(HDMA_TABLE_DYNAMIC + usize::from(upper) * 2, value);
            }
            if lower < 240 {
                self.write16(HDMA_TABLE_DYNAMIC + usize::from(lower) * 2, value);
            }
            if upper == center_y {
                break;
            }
            upper = upper.wrapping_add(1);
            lower = lower.wrapping_sub(1);
        }
        for i in 224..240 {
            self.write16(HDMA_TABLE_DYNAMIC + i * 2, 0);
        }
        self.wram
            .copy_within(HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 448, 0x17000);
        let mode = usize::from(self.read8(0x67e) >> 1);
        self.write16(
            0x67c,
            self.read16(0x67c)
                .wrapping_add_signed(i16::from(SPOTLIGHT_DELTA_SIZE[mode])),
        );
        if self.read16(0x67c) != u16::from(SPOTLIGHT_GOAL[mode]) {
            return;
        }
        if self.read16(0x67e) == 0 {
            self.write8(INIDISP_COPY, 0x80);
        } else {
            self.iris_spotlight_reset_table();
        }
        self.write8(SUBSUBMODULE, 0);
        self.write8(0x11, 0);
        if matches!(self.read8(0x10), 7 | 16) {
            if self.read8(0x1b) == 0 {
                self.write8(
                    0x12d,
                    self.read8(0x15b00 + usize::from(self.read8(OVERWORLD_SCREEN))) >> 4,
                );
            }
            if self.read8(0x132) != 0xff {
                self.write8(0x12c, self.read8(0x132));
            }
        }
        self.write8(0x10, self.read8(0x10c));
        if self.read8(0x10) == 6 {
            self.hooks.sprite_reset_all(self.wram);
        }
    }

    pub fn adjust_water_hdma_window(&mut self) {
        let center = self.read16(0x682).wrapping_sub(self.read16(0xe8));
        self.write16(0x674, center.wrapping_sub(self.read16(0x684)));
        self.write16(0x676, center.wrapping_add(self.read16(0x684)));
        self.adjust_water_hdma_window_x(center);
    }

    pub fn adjust_water_hdma_window_x(&mut self, center_y: u16) {
        let center_x = self.read16(0x680).wrapping_sub(self.read16(0xe2));
        self.write16(0x670, center_x);
        let spread = self.read16(0x686).saturating_sub(1);
        let left = center_x.wrapping_sub(spread).min(255);
        let right = center_x.wrapping_add(spread).min(255);
        let window = left | right << 8;
        let mut lower = center_y.wrapping_mul(2);
        if lower < 0xe0 {
            lower = 0xe0;
        }
        let mut upper = center_y.wrapping_mul(2).wrapping_sub(lower);
        loop {
            if upper as i16 >= 0 {
                let value = if self.read16(0x674) as i16 >= 0 && upper < self.read16(0x674) {
                    0xff
                } else {
                    window
                };
                if upper < 240 {
                    self.write16(
                        HDMA_TABLE_DYNAMIC + usize::from(upper) * 2,
                        if value == 0xffff { 0xff } else { value },
                    );
                }
            }
            let value = if lower >= self.read16(0x676) {
                0xff
            } else {
                if lower >= 225 && self.read16(0x678) != 0 {
                    self.write16(0x678, self.read16(0x678) - 1);
                }
                window
            };
            if lower < 240 {
                self.write16(
                    HDMA_TABLE_DYNAMIC + usize::from(lower) * 2,
                    if value == 0xffff { 0xff } else { value },
                );
            }
            let done = center_y == upper;
            lower = lower.wrapping_sub(1);
            upper = upper.wrapping_add(1);
            if done {
                break;
            }
        }
    }

    pub fn flood_dam_prep_flood_hdma(&mut self) {
        self.write16(0x674, self.read16(0x682).wrapping_sub(self.read16(0xe8)));
        let center_x = self.read16(0x680).wrapping_sub(self.read16(0xe2));
        self.write16(0x670, center_x);
        let radius = self.read16(0x686) ^ 1;
        let mut line = 0usize;
        let upper = usize::from(self.read16(0x676));
        loop {
            self.write16(HDMA_TABLE_DYNAMIC + line * 2, 0xff00);
            line += 1;
            if line == upper {
                break;
            }
        }
        let spread = radius.wrapping_add(1);
        let window =
            center_x.wrapping_add(spread) << 8 | u16::from(center_x.wrapping_sub(spread) as u8);
        let end = self.read16(0x676).wrapping_add(self.read16(0x684)) ^ 1;
        loop {
            if line >= usize::from(end) {
                self.write16(HDMA_TABLE_DYNAMIC + line * 2, 0xff);
            } else {
                let mut mapped = (line as u16).wrapping_mul(2);
                while mapped >= 480 {
                    mapped = mapped.wrapping_mul(2);
                }
                self.write16(
                    HDMA_TABLE_DYNAMIC + usize::from(mapped >> 1) * 2,
                    if window == 0xffff { 0xff } else { window },
                );
            }
            line += 1;
            if line >= 225 {
                break;
            }
        }
    }

    pub fn graphics_incremental_vram_upload(&mut self) {
        let counter = self.read8(0x412);
        if counter == 16 {
            return;
        }
        self.write8(0x19, 0x50 + counter);
        self.write16(0x118, u16::from(counter) << 9);
        self.write8(0x412, counter + 1);
    }

    pub fn link_zap_handle_mosaic(&mut self) {
        let mut level = self.read8(MOSAIC_LEVEL);
        if self.read8(0x647) == 0 {
            level = level.wrapping_add(0x10);
            if level == 0xc0 {
                self.write8(0x647, 1);
            }
        } else {
            level = level.wrapping_sub(0x10);
            if level == 0 {
                self.write8(0x647, 0);
            }
        }
        self.write8(MOSAIC_LEVEL, level);
        self.write8(MOSAIC_COPY, level >> 1 | 3);
        self.write8(BGMODE_COPY, 9);
    }

    pub fn player_set_custom_mosaic_level(&mut self, value: u8) {
        self.write8(0x647, 0);
        self.write8(MOSAIC_LEVEL, value);
        self.write8(MOSAIC_COPY, value >> 1 | 3);
        self.write8(BGMODE_COPY, 9);
    }

    pub fn reset_star_tile_graphics(&mut self) {
        self.write8(0x4bc, 0);
        self.dungeon_restore_star_tile_chr();
    }

    pub fn dungeon_restore_star_tile_chr(&mut self) {
        let (first, second) = if self.read8(0x4bc) != 0 {
            (32, 0)
        } else {
            (0, 32)
        };
        self.wram
            .copy_within(0xbdc0 + first..0xbdc0 + first + 32, 0x10000);
        self.wram
            .copy_within(0xbdc0 + second..0xbdc0 + second + 32, 0x10020);
        self.write8(0x17, 0x18);
    }

    pub fn dungeon_update_peg_gfx_buffer(&mut self, first: usize, second: usize) {
        self.wram
            .copy_within(0xb340 + first..0xb340 + first + 128, 0x10000);
        self.wram
            .copy_within(0xb340 + second..0xb340 + second + 128, 0x10080);
        self.write8(0x17, 23);
    }

    pub fn recover_peg_gfx_from_mapping(&mut self) {
        if self.read8(0xc172) != 0 {
            self.dungeon_update_peg_gfx_buffer(0x180, 0);
        } else {
            self.dungeon_update_peg_gfx_buffer(0, 0x180);
        }
    }

    pub fn update_pegs_step_1(&mut self) {
        if self.read8(0xc172) != 0 {
            self.dungeon_update_peg_gfx_buffer(0x80, 0x100);
        } else {
            self.dungeon_update_peg_gfx_buffer(0x100, 0x80);
        }
    }

    pub fn update_pegs_step_2(&mut self) {
        if self.read8(0xc172) != 0 {
            self.dungeon_update_peg_gfx_buffer(0x100, 0x80);
        } else {
            self.dungeon_update_peg_gfx_buffer(0x80, 0x100);
        }
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;

    struct Assets;
    impl GfxAssets for Assets {
        fn sprite(&self, _: usize) -> &[u8] {
            &[0; 0x600]
        }
        fn background(&self, _: usize) -> &[u8] {
            // Byte fills of 1024 and 512 bytes, followed by end of stream.
            &[0xe7, 0xff, 0x55, 0xe5, 0xff, 0x55, 0xff]
        }
        fn raw(&self, _: usize) -> &[u8] {
            unreachable!()
        }
        fn dialogue_font(&self) -> &[u8] {
            unreachable!()
        }
    }

    struct Hooks {
        vram: Box<[u16; z3_core::VRAM_WORDS]>,
        calls: Vec<u8>,
    }
    impl Hooks {
        fn nested(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE], stage: u8) {
            self.calls.push(stage);
            // A graphics script has no outstanding VRAM borrow or temporary
            // replacement owner when it calls back into gameplay.
            assert_eq!(self.vram[17], 0xface);
            self.vram[18] = u16::from(stage);
            let mut gfx = GraphicsCpu {
                wram,
                assets: &Assets,
                hooks: self,
            };
            gfx.enable_force_blank();
            gfx.decompress_animated_overworld_tiles(0);
        }
    }
    impl LoadGfxHooks for Hooks {
        fn set_target_overworld_warp_to_pyramid(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
            self.nested(wram, 0);
        }
        fn mirror_warp_load_sprites_and_colors(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
            self.nested(wram, 8);
        }
        fn handle_followers_after_mirroring(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
            assert!(wram[0x10000..0x11000].iter().all(|&byte| byte == 0));
            self.nested(wram, 13);
        }
    }

    #[test]
    fn mirror_callbacks_reenter_cpu_graphics_without_replacing_the_vram_owner() {
        for stage in [0, 8, 13] {
            let mut wram = Box::new([0; z3_core::WRAM_SIZE]);
            wram[0x200] = stage;
            wram[0x6ba] = 31;
            wram[0x10] = 21;
            let mut hooks = Hooks {
                vram: Box::new([0xface; z3_core::VRAM_WORDS]),
                calls: Vec::new(),
            };
            let vram_address = hooks.vram.as_ptr();
            GraphicsCpu {
                wram: &mut wram,
                assets: &Assets,
                hooks: &mut hooks,
            }
            .animate_mirror_warp();
            assert_eq!(hooks.calls, [stage]);
            assert_eq!(hooks.vram.as_ptr(), vram_address);
            assert_eq!(hooks.vram[17], 0xface);
            assert_eq!(hooks.vram[18], u16::from(stage));
            assert_eq!(u16::from_le_bytes([wram[0x134], wram[0x135]]), 0x3c00);
            assert_eq!(wram[0x200], stage + 1);
            assert_eq!(wram[INIDISP_COPY], 0x80);
        }
    }
}
