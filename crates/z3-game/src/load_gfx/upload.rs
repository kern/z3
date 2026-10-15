//! Hook-free graphics uploads with disjoint WRAM, VRAM, and asset borrows.

use super::*;

/// Uploads never borrow the gameplay callback owner.
pub struct GraphicsUpload<'a, A: GfxAssets> {
    pub wram: &'a mut [u8; z3_core::WRAM_SIZE],
    pub vram: &'a mut [u16],
    pub assets: &'a A,
}

impl<A: GfxAssets> GraphicsUpload<'_, A> {
    fn read8(&self, address: usize) -> u8 {
        self.wram[address]
    }

    fn write8(&mut self, address: usize, value: u8) {
        self.wram[address] = value;
    }

    fn read16(&self, address: usize) -> u16 {
        u16::from_le_bytes([self.wram[address], self.wram[address + 1]])
    }

    fn decomp_spr_to_wram(&mut self, offset: usize, gfx: usize) -> usize {
        decode_sprite(&mut self.wram[offset..], self.assets, gfx)
    }

    fn decomp_bg_to_wram(&mut self, offset: usize, gfx: usize) -> usize {
        decompress(&mut self.wram[offset..], self.assets.background(gfx))
    }

    pub fn erase_tile_maps(&mut self, high: u16, low: u16) {
        self.vram[..0x2000].fill(low);
        self.vram[0x6000..0x6800].fill(high);
    }

    pub fn erase_tile_maps_normal(&mut self) {
        self.erase_tile_maps(0x7f, 0x1ec);
    }

    pub fn erase_tile_maps_triforce(&mut self) {
        self.erase_tile_maps(0xa9, 0x7f);
    }

    pub fn erase_tile_maps_dungeonmap(&mut self) {
        self.erase_tile_maps(0x7f, 0x300);
    }

    fn convert_to_vram(&mut self, vram_offset: usize, source: &[u8], high: bool) {
        convert_words(
            &mut self.vram[vram_offset..vram_offset + 0x400],
            source,
            64,
            high,
        );
        if high {
            // C's `Do3To4High` uses this WRAM range as an eight-word
            // temporary, so the final converted tile remains observable.
            self.wram[0xbf..0xcf].copy_from_slice(&high_plane_scratch(source));
        }
    }

    fn convert_wram_to_vram(&mut self, vram_offset: usize, source: usize, high: bool) {
        // These are separate memory owners, so neither conversion input nor
        // output needs an intermediate buffer. Release the immutable WRAM
        // source before committing the final low-WRAM scratch result.
        let input = &self.wram[source..source + 0x600];
        convert_words(
            &mut self.vram[vram_offset..vram_offset + 0x400],
            input,
            64,
            high,
        );
        if high {
            let scratch = high_plane_scratch(input);
            self.wram[0xbf..0xcf].copy_from_slice(&scratch);
        }
    }

    pub fn load_sprite_graphics(&mut self, vram_offset: usize, pack: usize, temp: usize) {
        self.decomp_spr_to_wram(temp, pack);
        self.convert_wram_to_vram(
            vram_offset,
            temp,
            matches!(pack, 0x52 | 0x53 | 0x5a | 0x5b | 0x5c | 0x5e | 0x5f),
        );
    }

    pub fn load_background_graphics(
        &mut self,
        vram_offset: usize,
        pack: usize,
        slot: usize,
        temp: usize,
    ) {
        self.decomp_bg_to_wram(temp, pack);
        let high = if self.read8(0xaa1) >= 0x20 {
            matches!(slot, 7 | 2 | 3 | 4)
        } else {
            slot >= 4
        };
        self.convert_wram_to_vram(vram_offset, temp, high);
    }

    pub fn load_common_sprites(&mut self) {
        let misc = usize::from(self.read8(0xaa4));
        let assets = self.assets;
        self.convert_to_vram(0x4400, assets.sprite(misc), true);
        if self.read8(0x10) != 1 {
            for (vram, pack) in [(0x4800, 6), (0x4c00, 7)] {
                self.convert_to_vram(vram, assets.sprite(pack), false);
            }
        } else {
            self.load_sprite_graphics(0x4800, 94, 0x14000);
            self.load_sprite_graphics(0x4c00, 95, 0x14000);
        }
    }

    pub fn initialize_tilesets(&mut self) {
        self.load_common_sprites();
        let sprites = SPRITE_TILESETS[usize::from(self.read8(0xaa3))];
        for (i, selected) in sprites.into_iter().enumerate() {
            if selected != 0 {
                self.write8(0xc2fc + i, selected);
            }
            let pack = usize::from(self.read8(0xc2fc + i));
            self.load_sprite_graphics(0x5000 + i * 0x400, pack, 0x7800 + i * 0x600);
        }
        let main = MAIN_TILESETS[usize::from(self.read8(0xaa1))];
        let aux = AUX_TILESETS[usize::from(self.read8(0xaa2))];
        for i in 0..4 {
            self.write8(0xc2f8 + i, if aux[i] != 0 { aux[i] } else { main[i + 3] });
        }
        for (vram, pack, slot, temp) in [
            (0x2000, main[0], 7, 0x14000),
            (0x2400, main[1], 6, 0x14000),
            (0x2800, main[2], 5, 0x14000),
            (0x2c00, self.read8(0xc2f8), 4, 0x6000),
            (0x3000, self.read8(0xc2f9), 3, 0x6600),
            (0x3400, self.read8(0xc2fa), 2, 0x6c00),
            (0x3800, self.read8(0xc2fb), 1, 0x7200),
            (0x3c00, main[7], 0, 0x14000),
        ] {
            self.load_background_graphics(vram, usize::from(pack), slot, temp);
        }
    }

    pub fn load_default_graphics(&mut self) {
        let assets = self.assets;
        let source = assets.sprite(0);
        self.convert_to_vram(0x4000, source, true);
        // The C converter uses g_ram[0xbf..0xcf] as eight 16-bit scratch words
        // array. It processes each tile's first 16 bytes in reverse-index
        // order and leaves the final tile's plane OR masks observable in
        // WRAM, so canonical state parity must preserve that residue.
        leave_default_graphics_scratch(self.wram, source);
        decomp_and_upload_2bpp(self.wram, self.vram, self.assets, 0x7000, 0x6a);
        decomp_and_upload_2bpp(self.wram, self.vram, self.assets, 0x7400, 0x6b);
        decomp_and_upload_2bpp(self.wram, self.vram, self.assets, 0x7800, 0x69);
    }

    pub fn attract_load_bg3_gfx(&mut self) {
        decomp_and_upload_2bpp(self.wram, self.vram, self.assets, 0x7800, 0x67);
    }

    pub fn transfer_font_to_vram(&mut self) {
        let font = self.assets.dialogue_font();
        for (i, pair) in font[..0x1000].chunks_exact(2).enumerate() {
            self.vram[0x7000 + i] = u16::from_le_bytes([pair[0], pair[1]]);
        }
    }

    pub fn load_file_select_graphics(&mut self) {
        self.decomp_spr_to_wram(0x14000, 0x5e);
        self.convert_wram_to_vram(0x5000, 0x14000, true);

        self.decomp_spr_to_wram(0x14000, 0x5f);
        self.convert_wram_to_vram(0x5400, 0x14000, true);

        self.transfer_font_to_vram();

        self.decomp_spr_to_wram(0x14000, 0x6b);
        for index in 0..0x300 {
            self.vram[0x7800 + index] = self.read16(0x14000 + index * 2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RawAssets(Vec<u8>);
    impl GfxAssets for RawAssets {
        fn sprite(&self, _: usize) -> &[u8] {
            &self.0
        }
        fn background(&self, _: usize) -> &[u8] {
            unreachable!()
        }
        fn raw(&self, _: usize) -> &[u8] {
            unreachable!()
        }
        fn dialogue_font(&self) -> &[u8] {
            unreachable!()
        }
    }

    #[test]
    fn uploads_preserve_exact_planes_and_only_the_observable_high_scratch() {
        let assets = RawAssets(
            (0..0x600)
                .map(|index| (index * 37 + index / 17) as u8)
                .collect(),
        );
        for (pack, high) in [(12, false), (0x52, true)] {
            for source in [0xbf, 0x7800, 0x14000] {
                let mut wram = Box::new([0xa5; z3_core::WRAM_SIZE]);
                let mut expected_wram = wram.to_vec();
                expected_wram[source..source + 0x600].copy_from_slice(&assets.0);
                if high {
                    for row in 0..8 {
                        expected_wram[0xbf + (7 - row) * 2] =
                            assets.0[63 * 24 + row * 2] | assets.0[63 * 24 + row * 2 + 1];
                        expected_wram[0xc0 + (7 - row) * 2] = 0;
                    }
                }
                let mut vram = vec![0xbeef; z3_core::VRAM_WORDS];
                GraphicsUpload {
                    wram: &mut wram,
                    vram: &mut vram,
                    assets: &assets,
                }
                .load_sprite_graphics(0x4400, pack, source);
                let mut expected_planes = [0; 0x800];
                convert_3_to_4(&mut expected_planes, &assets.0, 64, high, true);
                let expected_words = expected_planes
                    .chunks_exact(2)
                    .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                    .collect::<Vec<_>>();
                assert_eq!(&vram[0x4400..0x4800], expected_words.as_slice());
                assert!(
                    vram[..0x4400]
                        .iter()
                        .chain(&vram[0x4800..])
                        .all(|&word| word == 0xbeef)
                );
                assert_eq!(wram.as_slice(), expected_wram);
            }
        }
    }
}
