//! File selection, copy/erase, and player-name entry.
//!
//! Addresses intentionally match `select_file.c`: the original aliases this
//! scratch storage with variables used by other modules.

use crate::{oam, ram::Ram};

pub const SAVE_SIZE: usize = 0x500;
pub const SAVE_COUNT: usize = 3;
pub const BACKUP_OFFSET: usize = 0xf00;
pub const SRAM_SIZE: usize = 0x2000;
pub const SRAM_SWORD: usize = 0x359;
pub const SRAM_SHIELD: usize = 0x35a;
pub const SRAM_HEALTH: usize = 0x36c;
pub const SRAM_NAME: usize = 0x3d9;
pub const SRAM_VALID: usize = 0x3e5;
pub const SRAM_DIED_COUNTER: usize = 0x405;

const MAIN_MODULE: usize = 0x10;
const SUBMODULE: usize = 0x11;
const NMI_LOAD_BG: usize = 0x14;
const FRAME_COUNTER: usize = 0x1a;
const SUBSUBMODULE: usize = 0xb0;
const SELECTFILE_ARR1: usize = 0xbf;
const R16: usize = 0xc8;
const R17: usize = 0xc9;
const R18: usize = 0xca;
const R20: usize = 0xcc;
const JOYPAD_H_LAST: usize = 0xf0;
const FILTERED_JOYPAD_H: usize = 0xf4;
const FILTERED_JOYPAD_L: usize = 0xf6;
const LINK_DMA_GRAPHICS_INDEX: usize = 0x100;
const IRQ_FLAG: usize = 0x128;
const MUSIC_CONTROL: usize = 0x12c;
const SOUND_EFFECT_1: usize = 0x12e;
const SOUND_EFFECT_2: usize = 0x12f;
const NAME_SAVE_OFFSET: usize = 0x200;
const NAME_SCROLL_X: usize = 0x630;
const NAME_COLUMN: usize = 0xb10;
const NAME_CURSOR_Y: usize = 0xb11;
const NAME_POSITION: usize = 0xb12;
const NAME_X_MOTION: usize = 0xb13;
const NAME_Y_MOTION: usize = 0xb14;
const NAME_ROW: usize = 0xb15;
const NAME_X_DIRECTION: usize = 0xb16;
const PREVIOUS_SELECTION: usize = 0xb9d;
const VRAM_UPLOAD_OFFSET: usize = 0x1000;
const VRAM_UPLOAD_DATA: usize = 0x1002;
const INIDISP_COPY: usize = 0x13;
const FLAG_UPDATE_CGRAM_IN_NMI: usize = 0x15;
const BG3HOFS_COPY2: usize = 0xe4;
const BG3VOFS_COPY2: usize = 0xea;
const IS_NMI_THREAD_ACTIVE: usize = 0x12a;
const NMI_DISABLE_CORE_UPDATES: usize = 0x710;
const NMI_FLAG_UPDATE_POLYHEDRAL: usize = 0x1f0c;
const SRAM_VAR1: usize = 0x1ffe;

/// Side effects owned by graphics, persistence, and the game loader.
pub trait FileSelectRuntime {
    fn force_blank(&mut self, ram: &mut Ram<'_>);
    fn erase_triforce_tilemaps(&mut self, ram: &mut Ram<'_>);
    fn load_file_select_palettes(&mut self, ram: &mut Ram<'_>, sram: &[u8]);
    fn load_file_select_graphics(&mut self, ram: &mut Ram<'_>);
    fn copy_selected_save_to_wram(&mut self, ram: &mut Ram<'_>, sram: &[u8], slot: usize);
    fn persist_sram(&mut self, sram: &[u8]);
    /// Performs the graphics/palette setup calls made by `Module_SelectFile_0`.
    fn initialize_file_select_graphics(&mut self, ram: &mut Ram<'_>);
}

#[must_use]
pub fn check_checksum(save: &[u8]) -> bool {
    assert!(save.len() >= SAVE_SIZE);
    save[..SAVE_SIZE].chunks_exact(2).fold(0_u16, |sum, word| {
        sum.wrapping_add(u16::from_le_bytes([word[0], word[1]]))
    }) == 0x5a5a
}

pub fn fix_checksum(save: &mut [u8]) {
    assert!(save.len() >= SAVE_SIZE);
    let sum = save[..SAVE_SIZE - 2]
        .chunks_exact(2)
        .fold(0_u16, |sum, word| {
            sum.wrapping_add(u16::from_le_bytes([word[0], word[1]]))
        });
    save[SAVE_SIZE - 2..SAVE_SIZE].copy_from_slice(&0x5a5a_u16.wrapping_sub(sum).to_le_bytes());
}

pub fn validate_sram(ram: &mut Ram<'_>, sram: &mut [u8]) {
    assert!(sram.len() >= BACKUP_OFFSET + SAVE_COUNT * SAVE_SIZE);
    for slot in 0..SAVE_COUNT {
        let primary = slot * SAVE_SIZE;
        let backup = primary + BACKUP_OFFSET;
        if !check_checksum(&sram[primary..primary + SAVE_SIZE]) {
            if check_checksum(&sram[backup..backup + SAVE_SIZE]) {
                sram.copy_within(backup..backup + SAVE_SIZE, primary);
            } else {
                sram[primary..primary + SAVE_SIZE].fill(0);
                sram[backup..backup + SAVE_SIZE].fill(0);
            }
        }
    }
    ram.slice_mut(0xd00, 768).fill(0);
}

fn read_u16(bytes: &[u8], address: usize) -> u16 {
    u16::from_le_bytes([bytes[address], bytes[address + 1]])
}
fn write_u16(bytes: &mut [u8], address: usize, value: u16) {
    bytes[address..address + 2].copy_from_slice(&value.to_le_bytes());
}

pub fn copy_save(sram: &mut [u8], source: usize, target: usize) {
    assert!(source < SAVE_COUNT && target < SAVE_COUNT);
    sram.copy_within(
        source * SAVE_SIZE..(source + 1) * SAVE_SIZE,
        target * SAVE_SIZE,
    );
}

pub fn erase_save(sram: &mut [u8], slot: usize) {
    assert!(slot < SAVE_COUNT);
    let primary = slot * SAVE_SIZE;
    sram[primary..primary + SAVE_SIZE].fill(0);
    let backup = primary + BACKUP_OFFSET;
    sram[backup..backup + SAVE_SIZE].fill(0);
}

pub struct FileSelect<'a, 'r, R: FileSelectRuntime> {
    pub ram: &'a mut Ram<'r>,
    pub sram: &'a mut [u8],
    pub runtime: &'a mut R,
}

impl<R: FileSelectRuntime> FileSelect<'_, '_, R> {
    fn write_stripe_hex(&mut self, byte_offset: usize, hex: &str) {
        debug_assert_eq!(hex.len() & 1, 0);
        for (i, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
            let digit = |c: u8| match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                _ => unreachable!("stripe literals contain lowercase hexadecimal only"),
            };
            self.ram.write_u8(
                VRAM_UPLOAD_DATA + byte_offset + i,
                (digit(pair[0]) << 4) | digit(pair[1]),
            );
        }
    }

    /// `SelectFile_Func1`: build the 32x32 checkerboard tilemap stripe and
    /// return the byte offset immediately after it.
    fn build_checkerboard_stripe(&mut self) -> usize {
        self.ram.write_u16(VRAM_UPLOAD_DATA, 0x10);
        self.ram.write_u16(VRAM_UPLOAD_DATA + 2, 0xff07);
        const TILE: [u16; 4] = [0x3581, 0x3582, 0x3591, 0x3592];
        for i in 0..1024 {
            self.ram.write_u16(
                VRAM_UPLOAD_DATA + 4 + i * 2,
                TILE[((i & 0x20) >> 4) + (i & 1)],
            );
        }
        4 + 1024 * 2
    }
    fn arr1(&self, slot: usize) -> u16 {
        self.ram.read_u16(SELECTFILE_ARR1 + slot * 2)
    }
    fn set_arr1(&mut self, slot: usize, value: u16) {
        self.ram.write_u16(SELECTFILE_ARR1 + slot * 2, value);
    }
    fn joy(&self, mask: u8) -> u8 {
        ((self.ram.read_u8(FILTERED_JOYPAD_L) & 0xc0) | self.ram.read_u8(FILTERED_JOYPAD_H)) & mask
    }
    fn sound1(&mut self, value: u8) {
        self.ram.write_u8(SOUND_EFFECT_1, value);
    }
    fn sound2(&mut self, value: u8) {
        self.ram.write_u8(SOUND_EFFECT_2, value);
    }

    /// `Module01_FileSelect`.
    pub fn run_file_select_module(&mut self) {
        self.ram.write_u16(BG3HOFS_COPY2, 0);
        self.ram.write_u16(BG3VOFS_COPY2, 0);
        match self.ram.read_u8(SUBMODULE) {
            0 => self.initialize_file_select(),
            1 => {
                self.ram.slice_mut(SELECTFILE_ARR1, 6).fill(0);
                self.erase_triforce();
            }
            2 => self.build_file_select_screen(),
            3 => self.trigger_file_select_stripes(),
            4 => self.trigger_name_stripes(),
            5 => self.file_select_main(),
            _ => {}
        }
    }

    pub fn initialize_file_select(&mut self) {
        self.runtime.force_blank(self.ram);
        self.ram.write_u8(IS_NMI_THREAD_ACTIVE, 0);
        self.ram.write_u8(NMI_FLAG_UPDATE_POLYHEDRAL, 0);
        self.ram.write_u8(MUSIC_CONTROL, 11);
        self.ram.write_u8(SUBMODULE, 1);
        self.ram.write_u16(0xaa8, 0x200);
        self.ram.write_u8(0xab6, 6);
        self.ram.write_u8(NMI_DISABLE_CORE_UPDATES, 6);
        self.ram.write_u8(0xab2, 0);
        self.ram.write_u8(0x202, 0);
        self.ram.write_u8(0xaa4, 1);
        self.ram.write_u8(0xaa1, 35);
        self.ram.write_u8(0xaa2, 81);
        self.runtime.initialize_file_select_graphics(self.ram);
        self.runtime.load_file_select_graphics(self.ram);
        validate_sram(self.ram, self.sram);
    }

    pub fn build_file_select_screen(&mut self) {
        let mut offset = self.build_checkerboard_stripe();
        self.write_stripe_hex(offset, "1042002789358a358b358c358b358c358b358c358b358c358b358c358b358c358b358c358b358c358a7589751062000399359a351064401e7f34107400039a75997510820003a935aa351084401e7f3410940003aa75a97510a200279d35ad359b359c359b359c359b359c359b359c359b359c359b359c359b359c359b359c35ad759d7510c20027ab35ac35ab35ac35ab35ac35ab35ac35ab35ac35ab35ac35ab35ac35ab35ac35ab35ac35ab75ac7510e20001833510e34032853510fd000184351102c0228635111dc0229635134200019335134340329535135d00019435");
        offset += 224;
        let mut tile = 0x1103_u16;
        for _ in 0..18 {
            self.ram
                .write_u16(VRAM_UPLOAD_DATA + offset, tile.swap_bytes());
            self.ram.write_u16(VRAM_UPLOAD_DATA + offset + 2, 0x3240);
            self.ram.write_u16(VRAM_UPLOAD_DATA + offset + 4, 0x347f);
            tile = tile.wrapping_add(0x20);
            offset += 6;
        }
        self.ram.write_u8(VRAM_UPLOAD_DATA + offset, 0xff);
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
        self.ram.write_u8(NMI_LOAD_BG, 1);
    }

    pub fn trigger_file_select_stripes(&mut self) {
        self.ram.write_u8(R16, self.ram.read_u8(PREVIOUS_SELECTION));
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
        self.ram.write_u8(NMI_LOAD_BG, 6);
    }

    pub fn trigger_name_stripes(&mut self) {
        self.write_stripe_hex(0, "61290025e718a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91861490025f7189118a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91861a90025e818a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91861c90025f8189118a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91862290025e918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91862490025f9189118a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918ff");
        self.ram.write_u8(INIDISP_COPY, 0xf);
        self.ram.write_u8(NMI_DISABLE_CORE_UPDATES, 0);
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
        self.ram.write_u8(NMI_LOAD_BG, 6);
    }

    pub fn return_to_file_select(&mut self) {
        self.ram.write_u8(MAIN_MODULE, 1);
        self.ram.write_u8(SUBMODULE, 1);
        self.ram.write_u8(SUBSUBMODULE, 0);
        self.ram.write_u8(R16, 0);
    }

    pub fn draw_fairy(&mut self, x: u8, y: u8) {
        let chr = if self.ram.read_u8(FRAME_COUNTER) & 8 != 0 {
            0xaa
        } else {
            0xa8
        };
        oam::set_plain(self.ram, 0, x, y, chr, 0x7e, 2);
    }

    pub fn draw_file_oam(&mut self, slot: usize) {
        const OI: [usize; 3] = [10, 15, 20];
        const Y: [u8; 3] = [0x43, 0x63, 0x83];
        const SWORD: [u8; 4] = [0x85, 0xa1, 0xa1, 0xa1];
        const SHIELD: [u8; 3] = [0xc4, 0xca, 0xe0];
        const F1: [u8; 3] = [0x72, 0x76, 0x7a];
        const F2: [u8; 3] = [0x32, 0x36, 0x3a];
        const F3: [u8; 3] = [0x30, 0x34, 0x38];
        self.ram.write_u16(LINK_DMA_GRAPHICS_INDEX, 0x22c);
        let save = slot * SAVE_SIZE;
        let sword = self.sram[save + SRAM_SWORD].wrapping_sub(1);
        let sword_chr = SWORD[if (sword as i8) < 0 {
            0
        } else {
            usize::from(sword.min(3))
        }];
        oam::set_plain(
            self.ram,
            OI[slot],
            0x40,
            Y[slot] - 5,
            sword_chr,
            F1[slot],
            0,
        );
        oam::set_plain(
            self.ram,
            OI[slot] + 1,
            0x40,
            Y[slot] + 3,
            sword_chr + 16,
            F1[slot],
            0,
        );
        if (sword as i8) < 0 {
            for i in 0..2 {
                let mut e = oam::read(self.ram, OI[slot] + i);
                e.y = 0xf0;
                oam::write(self.ram, OI[slot] + i, e);
            }
        }
        let shield = self.sram[save + SRAM_SHIELD].wrapping_sub(1);
        let shield_chr = SHIELD[if (shield as i8) < 0 {
            0
        } else {
            usize::from(shield.min(2))
        }];
        oam::set_plain(
            self.ram,
            OI[slot] + 2,
            0x2f,
            Y[slot] + 10,
            shield_chr,
            F2[slot],
            2,
        );
        if (shield as i8) < 0 {
            let mut e = oam::read(self.ram, OI[slot] + 2);
            e.y = 0xf0;
            oam::write(self.ram, OI[slot] + 2, e);
        }
        oam::set_plain(self.ram, OI[slot] + 3, 0x34, Y[slot], 0, F3[slot], 2);
        oam::set_plain(
            self.ram,
            OI[slot] + 4,
            0x34,
            Y[slot] + 8,
            2,
            F3[slot] | 0x40,
            2,
        );
    }

    pub fn draw_death_counter(&mut self, slot: usize) {
        const CHR: [u8; 10] = [0xd0, 0xac, 0xad, 0xbc, 0xbd, 0xae, 0xaf, 0xbe, 0xbf, 0xc0];
        const OI: [usize; 3] = [1, 4, 7];
        const DX: [i8; 3] = [12, 4, -4];
        const Y: [u8; 3] = [0x43, 0x63, 0x83];
        let mut n = read_u16(self.sram, slot * SAVE_SIZE + SRAM_DIED_COUNTER);
        if n == 0xffff {
            return;
        }
        n = n.min(999);
        let d = [n % 10, n / 10 % 10, n / 100];
        let mut i = if d[2] != 0 {
            2
        } else if d[1] != 0 {
            1
        } else {
            0
        };
        let mut oi = OI[slot];
        loop {
            oam::set_plain(
                self.ram,
                oi,
                0x34_u8.wrapping_add_signed(DX[i]),
                Y[slot] + 0x10,
                CHR[usize::from(d[i])],
                0x3c,
                0,
            );
            if i == 0 {
                break;
            }
            i -= 1;
            oi += 1;
        }
    }

    pub fn draw_name_and_health(&mut self, slot: usize) {
        const ND: [usize; 3] = [8, 0x5c, 0xb0];
        const HD: [usize; 3] = [0x16, 0x6a, 0xbe];
        let save = slot * SAVE_SIZE;
        for i in 0..6 {
            let tile = read_u16(self.sram, save + SRAM_NAME + i * 2) + 0x1800;
            let dst = VRAM_UPLOAD_DATA + ND[slot] + i * 2;
            self.ram.write_u16(dst, tile);
            self.ram.write_u16(dst + 42, tile + 0x10);
        }
        for i in 0..usize::from(self.sram[save + SRAM_HEALTH] >> 3) {
            self.ram.write_u16(
                VRAM_UPLOAD_DATA + HD[slot] + (i / 10) * 42 + (i % 10) * 2,
                0x520,
            );
        }
    }

    pub fn erase_triforce(&mut self) {
        self.ram.write_u8(NMI_DISABLE_CORE_UPDATES, 128);
        self.runtime.force_blank(self.ram);
        self.runtime.erase_triforce_tilemaps(self.ram);
        self.runtime.load_file_select_palettes(self.ram, self.sram);
        self.ram.write_u8(
            FLAG_UPDATE_CGRAM_IN_NMI,
            self.ram.read_u8(FLAG_UPDATE_CGRAM_IN_NMI).wrapping_add(1),
        );
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
    }

    pub fn file_select_main(&mut self) {
        const FY: [u8; 5] = [0x4a, 0x6a, 0x8a, 0xaf, 0xbf];
        let selected = usize::from(self.ram.read_u8(R16));
        if selected < 3 {
            self.ram.write_u8(PREVIOUS_SELECTION, selected as u8);
        }
        for slot in 0..3 {
            if read_u16(self.sram, slot * SAVE_SIZE + SRAM_VALID) == 0x55aa {
                self.set_arr1(slot, 1);
                self.draw_file_oam(slot);
                self.draw_death_counter(slot);
                self.draw_name_and_health(slot);
            }
        }
        let selected = usize::from(self.ram.read_u8(R16));
        self.draw_fairy(0x1c, FY[selected]);
        self.ram.write_u8(NMI_LOAD_BG, 1);
        let a = self.joy(0xfc);
        if a & 0x2c != 0 {
            let k = self.ram.read_u8(R16);
            self.ram.write_u8(
                R16,
                if a & 8 != 0 {
                    if k == 0 { 4 } else { k - 1 }
                } else {
                    (k + 1) % 5
                },
            );
            self.sound2(0x20);
        } else if a != 0 {
            self.sound1(0x2c);
            let selected = usize::from(self.ram.read_u8(R16));
            if selected < 3 {
                self.ram.write_u8(R17, 0);
                if self.arr1(selected) == 0 {
                    self.ram.write_u8(MAIN_MODULE, 4);
                    self.ram.write_u8(SUBMODULE, 0);
                    self.ram.write_u8(SUBSUBMODULE, 0);
                } else {
                    self.ram.write_u8(MUSIC_CONTROL, 0xf1);
                    write_u16(self.sram, SRAM_VAR1, selected as u16 * 2 + 2);
                    self.ram.write_u16(0, selected as u16 * SAVE_SIZE as u16);
                    self.runtime
                        .copy_selected_save_to_wram(self.ram, self.sram, selected);
                }
            } else if (0..3).any(|slot| self.arr1(slot) != 0) {
                self.ram
                    .write_u8(MAIN_MODULE, if selected == 3 { 2 } else { 3 });
                self.ram.write_u8(R16, 0);
                self.ram.write_u8(SUBMODULE, 0);
                self.ram.write_u8(SUBSUBMODULE, 0);
            } else {
                self.sound1(0x3c);
            }
        }
    }

    pub fn choose_copy_source(&mut self) {
        const X: [u8; 4] = [36, 36, 36, 28];
        const Y: [u8; 4] = [87, 111, 135, 191];
        self.write_stripe_hex(0, "61040015851826180718af18021807186f188618a918a918a91861240015951836181718bf18121817187f189618a918a918a9186167000fe718a918a918a918a918a918a918a9186187000ff7189118a918a918a918a918a918a91861c7000fe818a918a918a918a918a918a918a91861e7000ff8189118a918a918a918a918a918a9186227000fe918a918a918a918a918a918a918a9186247000ff9189118a918a918a918a918a918a918ff");
        self.ram.write_u16(VRAM_UPLOAD_OFFSET, 0xac);
        for slot in 0..3 {
            if self.arr1(slot) & 1 != 0 {
                self.draw_picker_name(slot, [0x3c, 0x64, 0x8c][slot]);
            }
        }
        let selected = usize::from(self.ram.read_u8(R16));
        self.draw_fairy(X[selected], Y[selected]);
        let a = self.joy(0xfc);
        if a & 0x2c != 0 {
            let mut k = self.ram.read_u8(R16);
            loop {
                k = if a & 8 == 0 {
                    if k >= 3 { 0 } else { k + 1 }
                } else if k == 0 {
                    3
                } else {
                    k - 1
                };
                if k == 3 || self.arr1(usize::from(k)) != 0 {
                    break;
                }
            }
            self.ram.write_u8(R16, k);
            self.sound2(0x20);
        } else if a != 0 {
            self.sound1(0x2c);
            let k = self.ram.read_u8(R16);
            if k == 3 {
                self.return_to_file_select();
                return;
            }
            self.ram.write_u16(R20, u16::from(k) * 2);
            self.write_stripe_hex(52, "6167400ea9006187400ea90061c7400ea90061e7400ea900113000018335113140148535113c000184351150c00e8635115cc00e9635125000019335125140149535125c00019435ff");
            if k != 2 {
                let offset = usize::from(k) * 12;
                self.ram.write_u16(VRAM_UPLOAD_DATA + offset + 52, 0x2762);
                self.ram.write_u16(VRAM_UPLOAD_DATA + offset + 58, 0x4762);
            }
            self.ram
                .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE) + 1);
            self.ram.write_u8(R16, 0);
        }
    }

    pub fn choose_copy_target(&mut self) {
        let source = self.ram.read_u16(R20);
        let mut targets = [0_u8; 2];
        let mut j = 0;
        for t in [0_u8, 2, 4] {
            if u16::from(t) != source {
                targets[j] = t;
                j += 1;
            }
        }
        self.ram.write_u8(R18, targets[0]);
        self.ram.write_u8(R18 + 1, targets[1]);
        self.write_stripe_hex(0, "61510015851823180e18a91826180718af18021807186f18861861710015951833181e18b91836181718bf18121817187f18961861b4000fa918a918a918a918a918a918a918a91861d4000fa9189118a918a918a918a918a918a9186214000fa918a918a918a918a918a918a918a9186234000fa9189118a918a918a918a918a918a918ff");
        for (j, slot) in (0..3).filter(|&slot| slot as u16 * 2 != source).enumerate() {
            let dst = [0x38, 0x60][j];
            let tile = [0x18e7, 0x18e8, 0x18e9][slot];
            self.ram.write_u16(VRAM_UPLOAD_DATA + dst, tile);
            self.ram.write_u16(VRAM_UPLOAD_DATA + dst + 20, tile + 0x10);
            if self.arr1(slot) != 0 {
                self.draw_picker_name(slot, dst + 4);
            }
        }
        self.ram.write_u16(VRAM_UPLOAD_OFFSET, 132);
        let selected = usize::from(self.ram.read_u8(R16));
        self.draw_fairy([0x8c, 0x8c, 0x1c][selected], [0x67, 0x7f, 0xbf][selected]);
        let a = self.joy(0xfc);
        if a & 0x2c != 0 {
            let k = self.ram.read_u8(R16);
            self.ram.write_u8(
                R16,
                if a & 8 != 0 {
                    if k == 0 { 2 } else { k - 1 }
                } else {
                    (k + 1) % 3
                },
            );
            self.sound2(0x20);
        } else if a != 0 {
            self.sound1(0x2c);
            let k = self.ram.read_u8(R16);
            if k == 2 {
                self.return_to_file_select();
                return;
            }
            self.ram.write_u16(R18, u16::from(targets[usize::from(k)]));
            self.write_stripe_hex(52, "61b4400ea90061d4400ea90062c6000d02180e180f182818a9180e180a1862e6000d12181e181f183818a9181e181a18ff");
            if k == 0 {
                self.ram.write_u16(VRAM_UPLOAD_DATA + 52, 0x1462);
                self.ram.write_u16(VRAM_UPLOAD_DATA + 58, 0x3462);
            }
            self.ram
                .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE) + 1);
            self.ram.write_u8(R16, 0);
        }
    }

    pub fn confirm_copy(&mut self) {
        self.draw_fairy(0x1c, [0xaf, 0xbf][usize::from(self.ram.read_u8(R16))]);
        let a = self.joy(0xfc);
        if a & 0x2c != 0 {
            self.ram.write_u8(R16, self.ram.read_u8(R16) ^ 1);
            self.sound2(0x20);
        } else if a != 0 {
            self.sound1(0x2c);
            if self.ram.read_u8(R16) == 0 {
                let source = usize::from(self.ram.read_u16(R20) >> 1);
                let target = usize::from(self.ram.read_u16(R18) >> 1);
                copy_save(self.sram, source, target);
                self.set_arr1(target, 1);
                self.runtime.persist_sram(self.sram);
            }
            self.return_to_file_select();
        }
    }

    pub fn confirm_erase(&mut self) {
        self.draw_fairy(0x1c, [175, 191][usize::from(self.ram.read_u8(R16))]);
        if self.ram.read_u8(FILTERED_JOYPAD_H) & 0x2c != 0 {
            self.ram.write_u8(R16, self.ram.read_u8(R16) ^ 1);
            self.sound2(0x20);
        }
        if self.joy(0xd0) != 0 {
            self.sound1(0x2c);
            if self.ram.read_u8(R16) == 0 {
                self.sound2(0x22);
                self.sound1(0);
                let slot = usize::from(self.ram.read_u8(SUBSUBMODULE));
                self.set_arr1(slot, 0);
                erase_save(self.sram, slot);
                self.runtime.persist_sram(self.sram);
            }
            self.return_to_file_select();
            self.ram.write_u8(SUBSUBMODULE, 0);
        }
    }

    pub fn choose_erase_target(&mut self) {
        const X: [u8; 4] = [36, 36, 36, 28];
        const Y: [u8; 4] = [103, 127, 151, 191];
        self.write_stripe_hex(0, "61a70025e718a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91861c70025f7189118a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91862070025e818a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91862270025f8189118a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91862670025e918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a91862870025f9189118a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918a918ff");
        for slot in 0..3 {
            if self.arr1(slot) != 0 {
                self.draw_name_and_health(slot);
            }
        }
        let mut k = self.ram.read_u8(R16);
        self.draw_fairy(X[usize::from(k)], Y[usize::from(k)]);
        if self.ram.read_u8(FILTERED_JOYPAD_H) & 0x2c != 0 {
            if self.ram.read_u8(FILTERED_JOYPAD_H) & 0x24 == 0 {
                loop {
                    if k == 0 {
                        k = 3;
                        break;
                    }
                    k -= 1;
                    if self.arr1(usize::from(k)) != 0 {
                        break;
                    }
                }
            } else {
                loop {
                    k = k.wrapping_add(1);
                    if k >= 4 {
                        k = 0;
                    }
                    if k == 3 || self.arr1(usize::from(k)) != 0 {
                        break;
                    }
                }
            }
            self.sound2(0x20);
        }
        self.ram.write_u8(R16, k);
        if self.joy(0xd0) != 0 {
            self.sound1(0x2c);
            if k == 3 {
                self.return_to_file_select();
                return;
            }
            self.ram
                .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
            self.write_stripe_hex(0, "61a74024a90061c74024a90062074024a90062274024a90062c6002104182118001822180418a91823180718af182218a9180f180b18001828180418211862e6002114183118101832181418a91833181718bf183218a9181f181b181018381814183118ff");
            if k != 2 {
                let offset = usize::from(k) * 12;
                self.ram.write_u16(VRAM_UPLOAD_DATA + offset, 0x6762);
                self.ram.write_u16(VRAM_UPLOAD_DATA + offset + 6, 0x8762);
            }
            self.ram.write_u8(SUBSUBMODULE, k);
            self.ram.write_u8(R16, 0);
        }
    }

    pub fn run_copy_module(&mut self) {
        self.ram.write_u8(PREVIOUS_SELECTION, 0);
        let entry_submodule = self.ram.read_u8(SUBMODULE);
        match entry_submodule {
            0 => self.erase_triforce(),
            1 => self.build_file_select_screen(),
            2 => self.setup_picker(7),
            3 => self.choose_copy_source(),
            4 => self.choose_copy_target(),
            5 => self.confirm_copy(),
            _ => {}
        }
        if (3..=5).contains(&entry_submodule) {
            self.ram.write_u8(NMI_LOAD_BG, 1);
        }
        if (entry_submodule == 3 || entry_submodule == 4)
            && self.ram.read_u8(SUBMODULE) == entry_submodule
            && self.ram.read_u8(FRAME_COUNTER) & 0x30 == 0
        {
            self.delete_picker_header_stripe();
        }
    }

    pub fn run_erase_module(&mut self) {
        match self.ram.read_u8(SUBMODULE) {
            0 => self.erase_triforce(),
            1 => self.build_file_select_screen(),
            2 => self.setup_picker(8),
            3 => {
                if self.ram.read_u8(R16) < 3 {
                    self.ram.write_u8(PREVIOUS_SELECTION, self.ram.read_u8(R16));
                }
                self.choose_erase_target();
                self.ram.write_u8(NMI_LOAD_BG, 1);
            }
            4 => {
                self.confirm_erase();
                self.ram.write_u8(NMI_LOAD_BG, 1);
            }
            _ => {}
        }
    }

    fn setup_picker(&mut self, bg_index: u8) {
        self.ram.write_u8(NMI_LOAD_BG, bg_index);
        self.ram
            .write_u8(SUBMODULE, self.ram.read_u8(SUBMODULE).wrapping_add(1));
        self.ram.write_u8(INIDISP_COPY, 0xf);
        self.ram.write_u8(NMI_DISABLE_CORE_UPDATES, 0);
        let slot = (0..3).find(|&slot| self.arr1(slot) != 0).unwrap_or(0);
        self.ram.write_u8(R16, slot as u8);
    }

    pub fn delete_picker_header_stripe(&mut self) {
        for byte_offset in [4, 0x1e] {
            for i in 0..11 {
                self.ram
                    .write_u16(VRAM_UPLOAD_DATA + byte_offset + i * 2, 0xa9);
            }
        }
    }

    fn draw_picker_name(&mut self, slot: usize, byte_offset: usize) {
        for i in 0..6 {
            let tile = read_u16(self.sram, slot * SAVE_SIZE + SRAM_NAME + i * 2) + 0x1800;
            self.ram
                .write_u16(VRAM_UPLOAD_DATA + byte_offset + i * 2, tile);
            self.ram
                .write_u16(VRAM_UPLOAD_DATA + byte_offset + 20 + i * 2, tile + 0x10);
        }
    }

    pub fn erase_save_for_naming(&mut self) {
        self.erase_triforce();
        self.ram.write_u8(IRQ_FLAG, 1);
        self.ram.write_u8(NAME_COLUMN, 0);
        self.ram.write_u8(NAME_POSITION, 0);
        self.ram.write_u8(NAME_ROW, 0);
        self.ram.write_u8(R18, 0);
        self.ram.write_u8(R20, 0);
        self.ram.write_u8(NAME_CURSOR_Y, 0x83);
        self.ram.write_u16(NAME_SCROLL_X, 0x1f0);
        self.ram.write_u16(BG3HOFS_COPY2, 0);
        let offset = usize::from(self.ram.read_u8(R16)) * SAVE_SIZE;
        self.ram.write_u16(NAME_SAVE_OFFSET, offset as u16);
        self.sram[offset..offset + SAVE_SIZE].fill(0);
        for i in 0..6 {
            write_u16(self.sram, offset + SRAM_NAME + i * 2, 0xa9);
        }
    }

    pub fn run_name_module(&mut self) {
        match self.ram.read_u8(SUBMODULE) {
            0 => self.erase_save_for_naming(),
            1 => {
                let offset = self.build_checkerboard_stripe();
                self.ram.write_u16(VRAM_UPLOAD_DATA + offset, 0xffff);
                self.ram.write_u8(NMI_LOAD_BG, 1);
                self.ram.write_u8(SUBMODULE, 2);
            }
            2 => {
                self.ram.write_u8(NMI_LOAD_BG, 5);
                self.ram.write_u8(SUBMODULE, 3);
                self.ram.write_u8(INIDISP_COPY, 15);
                self.ram.write_u8(NMI_DISABLE_CORE_UPDATES, 0);
            }
            3 => self.do_the_naming(),
            _ => {}
        }
    }

    pub fn do_the_naming(&mut self) {
        const X_STEP: [i16; 26] = [
            -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -2, 2, -2, 2, -2, 2, -2, 2, -4,
            4,
        ];
        const ROW_Y: [u8; 4] = [131, 147, 163, 179];
        const CURSOR_X: [u8; 6] = [31, 47, 63, 79, 95, 111];
        const COLUMN_X: [u16; 32] = [
            0x1f0, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0,
            0xe0, 0xf0, 0x100, 0x110, 0x120, 0x130, 0x140, 0x150, 0x160, 0x170, 0x180, 0x190,
            0x1a0, 0x1b0, 0x1c0, 0x1d0, 0x1e0,
        ];
        const CHARACTERS: [u8; 128] = [
            6, 7, 0x5f, 9, 0x59, 0x59, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x60, 0x23,
            0x59, 0x59, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x59, 0x59, 0x59, 0, 1, 2, 3, 4, 5, 0x10,
            0x11, 0x12, 0x13, 0x59, 0x59, 0x24, 0x5f, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c,
            0x2d, 0x59, 0x59, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f, 0x59, 0x59, 0x59, 0x0a, 0x0b, 0x0c,
            0x0d, 0x0e, 0x0f, 0x40, 0x41, 0x42, 0x59, 0x59, 0x59, 0x2e, 0x2f, 0x30, 0x31, 0x32,
            0x33, 0x40, 0x41, 0x42, 0x59, 0x59, 0x59, 0x61, 0x3f, 0x45, 0x46, 0x59, 0x59, 0x59,
            0x59, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x44, 0x59, 0x6f, 0x6f, 0x59, 0x59, 0x59,
            0x59, 0x59, 0x59, 0x59, 0x5a, 0x44, 0x59, 0x6f, 0x6f, 0x59, 0x59, 0x5a, 0x44, 0x59,
            0x6f, 0x6f, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x5a,
        ];

        loop {
            let mut j = self.ram.read_u8(NAME_X_MOTION);
            if j == 0 {
                self.name_scroll_input_x();
                break;
            }
            if j != 0x31 {
                self.ram.write_u8(NAME_X_MOTION, j.wrapping_add(4));
            }
            j = j.wrapping_sub(1);
            let column = usize::from(self.ram.read_u8(NAME_COLUMN));
            if COLUMN_X[column] == self.ram.read_u16(NAME_SCROLL_X) {
                self.ram.write_u8(
                    NAME_X_MOTION,
                    if self.ram.read_u8(JOYPAD_H_LAST) & 3 != 0 {
                        0x30
                    } else {
                        0
                    },
                );
                self.name_scroll_input_x();
                continue;
            }
            if self.ram.read_u8(NAME_X_DIRECTION) == 0 {
                j = j.wrapping_add(2);
            }
            let step = X_STEP[usize::from(j) / 2];
            self.ram.write_u16(
                NAME_SCROLL_X,
                self.ram.read_u16(NAME_SCROLL_X).wrapping_add_signed(step) & 0x1ff,
            );
            break;
        }

        loop {
            if self.ram.read_u8(NAME_Y_MOTION) == 0 {
                self.name_scroll_input_y();
                break;
            }
            let row = usize::from(self.ram.read_u8(NAME_ROW));
            let y = self.ram.read_u8(NAME_CURSOR_Y);
            let diff = y.wrapping_sub(ROW_Y[row]);
            if diff != 0 {
                self.ram.write_u8(
                    NAME_CURSOR_Y,
                    if (diff as i8) < 0 {
                        y.wrapping_add(2)
                    } else {
                        y.wrapping_sub(2)
                    },
                );
                break;
            }
            self.ram.write_u8(NAME_Y_MOTION, 0);
            self.name_scroll_input_y();
        }

        let cursor_y = self.ram.read_u8(NAME_CURSOR_Y);
        for i in 0..26 {
            oam::set_plain(self.ram, i, 0x18 + i as u8 * 8, cursor_y, 0x2e, 0x3c, 0);
        }
        let position = usize::from(self.ram.read_u8(NAME_POSITION));
        oam::set_plain(self.ram, 26, CURSOR_X[position], 0x58, 0x29, 0x0c, 0);

        if self.ram.read_u8(NAME_X_MOTION) | self.ram.read_u8(NAME_Y_MOTION) != 0 {
            return;
        }
        if self.ram.read_u8(FILTERED_JOYPAD_H) & 0x10 == 0 {
            if self.ram.read_u8(FILTERED_JOYPAD_H) & 0xc0 == 0
                && self.ram.read_u8(FILTERED_JOYPAD_L) & 0xc0 == 0
            {
                return;
            }
            self.sound1(0x2b);
            let index = usize::from(self.ram.read_u8(NAME_COLUMN))
                + usize::from(self.ram.read_u8(NAME_ROW)) * 0x20;
            let tile = CHARACTERS[index];
            match tile {
                0x5a => self.ram.write_u8(
                    NAME_POSITION,
                    if position == 0 { 5 } else { position as u8 - 1 },
                ),
                0x44 => self.ram.write_u8(NAME_POSITION, ((position + 1) % 6) as u8),
                0x6f => {
                    self.finish_name();
                }
                _ => {
                    let chr = u16::from(tile & 0xf0) * 2 + u16::from(tile & 0x0f);
                    let offset = usize::from(self.ram.read_u16(NAME_SAVE_OFFSET));
                    write_u16(self.sram, offset + SRAM_NAME + position * 2, chr);
                    self.draw_selected_character(position, chr);
                    self.ram.write_u8(NAME_POSITION, ((position + 1) % 6) as u8);
                }
            }
        } else {
            self.finish_name();
        }
    }

    pub fn name_scroll_input_x(&mut self) {
        let input = self.ram.read_u8(JOYPAD_H_LAST) & 3;
        if input == 0 {
            return;
        }
        let direction = input - 1;
        self.ram.write_u8(NAME_X_DIRECTION, direction);
        self.ram.write_u8(
            NAME_X_MOTION,
            self.ram.read_u8(NAME_X_MOTION).wrapping_add(1),
        );
        let column = self.ram.read_u8(NAME_COLUMN);
        self.ram.write_u8(
            NAME_COLUMN,
            if direction == 0 {
                (column + 1) & 0x1f
            } else {
                column.wrapping_sub(1) & 0x1f
            },
        );
    }

    pub fn name_scroll_input_y(&mut self) {
        let input = self.ram.read_u8(JOYPAD_H_LAST) & 0x0c;
        if input == 0 {
            self.ram.write_u8(R18, 0);
            return;
        }
        let row = self.ram.read_u8(NAME_ROW);
        if ((input * 2) | row) == 0x10 || ((input * 4) | row) == 0x13 {
            self.ram.write_u8(R18 + 1, input);
            return;
        }
        let direction = (input >> 2) - 1;
        self.ram.write_u8(
            NAME_ROW,
            if direction == 0 {
                (row + 1) & 3
            } else {
                row.wrapping_sub(1) & 3
            },
        );
        self.ram.write_u8(
            NAME_Y_MOTION,
            self.ram.read_u8(NAME_Y_MOTION).wrapping_add(1),
        );
        self.ram.write_u8(R18 + 1, direction + 1);
    }

    pub fn draw_selected_character(&mut self, position: usize, chr: u16) {
        let a = [0x84_u16, 0x86, 0x88, 0x8a, 0x8c, 0x8e][position] | 0x6100;
        for (offset, value) in [
            a.swap_bytes(),
            0x100,
            0x1800 | chr,
            (a + 0x20).swap_bytes(),
            0x100,
            (0x1800 | chr) + 0x10,
        ]
        .into_iter()
        .enumerate()
        {
            self.ram.write_u16(VRAM_UPLOAD_DATA + offset * 2, value);
        }
        self.ram.write_u8(VRAM_UPLOAD_DATA + 12, 0xff);
        self.ram.write_u8(NMI_LOAD_BG, 1);
    }

    pub fn finish_name(&mut self) -> bool {
        let slot = usize::from(self.ram.read_u8(R16));
        let save = slot * SAVE_SIZE;
        if (0..6).all(|i| read_u16(self.sram, save + SRAM_NAME + i * 2) == 0xa9) {
            self.sound1(0x3c);
            return false;
        }
        write_u16(self.sram, save + SRAM_VALID, 0x55aa);
        write_u16(self.sram, save + 0x20c, 0xf000);
        write_u16(self.sram, save + 0x20e, 0xf000);
        write_u16(self.sram, save + SRAM_DIED_COUNTER, 0xffff);
        write_u16(self.sram, SRAM_VAR1, slot as u16 * 2 + 2);
        const INIT: [u8; 60] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x18, 0x18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0xf8, 0, 0,
        ];
        self.sram[save + 0x340..save + 0x340 + 60].copy_from_slice(&INIT);
        fix_checksum(&mut self.sram[save..save + SAVE_SIZE]);
        self.runtime.persist_sram(self.sram);
        self.return_to_file_select();
        self.ram.write_u8(IRQ_FLAG, 0xff);
        self.sound1(0x2c);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[derive(Default)]
    struct TestRuntime {
        persisted: usize,
    }
    impl FileSelectRuntime for TestRuntime {
        fn force_blank(&mut self, _ram: &mut Ram<'_>) {}
        fn erase_triforce_tilemaps(&mut self, _ram: &mut Ram<'_>) {}
        fn load_file_select_palettes(&mut self, _ram: &mut Ram<'_>, _sram: &[u8]) {}
        fn load_file_select_graphics(&mut self, _ram: &mut Ram<'_>) {}
        fn copy_selected_save_to_wram(&mut self, _ram: &mut Ram<'_>, _sram: &[u8], _slot: usize) {}
        fn persist_sram(&mut self, _sram: &[u8]) {
            self.persisted += 1;
        }
        fn initialize_file_select_graphics(&mut self, _ram: &mut Ram<'_>) {}
    }
    #[test]
    fn checksum_fix_and_corruption() {
        let mut s = [0xff; SAVE_SIZE];
        fix_checksum(&mut s);
        assert!(check_checksum(&s));
        s[19] ^= 0x80;
        assert!(!check_checksum(&s));
    }
    #[test]
    fn validation_restores_or_clears() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0_u8; SRAM_SIZE];
        s[BACKUP_OFFSET + 7] = 0x42;
        fix_checksum(&mut s[BACKUP_OFFSET..BACKUP_OFFSET + SAVE_SIZE]);
        s[SAVE_SIZE + 8] = 9;
        validate_sram(&mut r, &mut s);
        assert_eq!(s[7], 0x42);
        assert!(check_checksum(&s[..SAVE_SIZE]));
        assert!(s[SAVE_SIZE..2 * SAVE_SIZE].iter().all(|&x| x == 0));
    }
    #[test]
    fn copy_and_erase_match_primary_backup_rules() {
        let mut s = [0_u8; SRAM_SIZE];
        for (i, b) in s[..SAVE_SIZE].iter_mut().enumerate() {
            *b = i as u8;
        }
        copy_save(&mut s, 0, 1);
        assert_eq!(&s[..SAVE_SIZE], &s[SAVE_SIZE..2 * SAVE_SIZE]);
        s[BACKUP_OFFSET + SAVE_SIZE] = 7;
        erase_save(&mut s, 1);
        assert!(s[SAVE_SIZE..2 * SAVE_SIZE].iter().all(|&x| x == 0));
        assert!(
            s[BACKUP_OFFSET + SAVE_SIZE..BACKUP_OFFSET + 2 * SAVE_SIZE]
                .iter()
                .all(|&x| x == 0)
        );
    }
    #[test]
    fn naming_movement_wraps_and_guards_vertical_repeat() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0; SRAM_SIZE];
        let mut rt = TestRuntime::default();
        r.write_u8(NAME_COLUMN, 31);
        r.write_u8(JOYPAD_H_LAST, 1);
        let mut f = FileSelect {
            ram: &mut r,
            sram: &mut s,
            runtime: &mut rt,
        };
        f.name_scroll_input_x();
        assert_eq!(f.ram.read_u8(NAME_COLUMN), 0);
        f.ram.write_u8(NAME_ROW, 3);
        f.ram.write_u8(JOYPAD_H_LAST, 4);
        f.name_scroll_input_y();
        assert_eq!(f.ram.read_u8(NAME_ROW), 3);
        f.ram.write_u8(NAME_ROW, 0);
        f.ram.write_u8(JOYPAD_H_LAST, 8);
        f.name_scroll_input_y();
        assert_eq!(f.ram.read_u8(NAME_ROW), 0);
        assert_eq!(f.ram.read_u8(R18 + 1), 8);
    }

    #[test]
    fn naming_horizontal_animation_reaches_exact_c_target() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0; SRAM_SIZE];
        let mut rt = TestRuntime::default();
        r.write_u8(NAME_COLUMN, 0);
        r.write_u16(NAME_SCROLL_X, 0x1f0);
        r.write_u8(JOYPAD_H_LAST, 1);
        let mut f = FileSelect {
            ram: &mut r,
            sram: &mut s,
            runtime: &mut rt,
        };
        for _ in 0..20 {
            f.do_the_naming();
            f.ram.write_u8(JOYPAD_H_LAST, 0);
        }
        assert_eq!(f.ram.read_u8(NAME_COLUMN), 1);
        assert_eq!(f.ram.read_u16(NAME_SCROLL_X), 0);
        assert_eq!(f.ram.read_u8(NAME_X_MOTION), 0);
    }

    #[test]
    fn naming_character_write_uses_saved_slot_offset() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0; SRAM_SIZE];
        let mut rt = TestRuntime::default();
        r.write_u16(NAME_SAVE_OFFSET, SAVE_SIZE as u16);
        r.write_u8(NAME_COLUMN, 0);
        r.write_u8(NAME_ROW, 0);
        r.write_u8(FILTERED_JOYPAD_H, 0x80);
        let mut f = FileSelect {
            ram: &mut r,
            sram: &mut s,
            runtime: &mut rt,
        };
        f.do_the_naming();
        assert_eq!(read_u16(f.sram, SAVE_SIZE + SRAM_NAME), 6);
        assert_eq!(f.ram.read_u8(NAME_POSITION), 1);
    }

    #[test]
    fn copy_targets_keep_the_c_picker_order() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0; SRAM_SIZE];
        let mut rt = TestRuntime::default();
        r.write_u16(R20, 2);
        let mut f = FileSelect {
            ram: &mut r,
            sram: &mut s,
            runtime: &mut rt,
        };
        f.choose_copy_target();
        assert_eq!(f.ram.read_u8(R18), 0);
        assert_eq!(f.ram.read_u8(R18 + 1), 4);
    }

    #[test]
    fn checkerboard_builder_matches_c_layout() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0; SRAM_SIZE];
        let mut rt = TestRuntime::default();
        let mut f = FileSelect {
            ram: &mut r,
            sram: &mut s,
            runtime: &mut rt,
        };
        let end = f.build_checkerboard_stripe();
        assert_eq!(end, 2052);
        assert_eq!(f.ram.read_u16(VRAM_UPLOAD_DATA), 0x10);
        assert_eq!(f.ram.read_u16(VRAM_UPLOAD_DATA + 2), 0xff07);
        assert_eq!(f.ram.read_u16(VRAM_UPLOAD_DATA + 4), 0x3581);
        assert_eq!(f.ram.read_u16(VRAM_UPLOAD_DATA + 6), 0x3582);
        assert_eq!(f.ram.read_u16(VRAM_UPLOAD_DATA + 68), 0x3591);
        assert_eq!(f.ram.read_u16(VRAM_UPLOAD_DATA + 70), 0x3592);
    }

    #[test]
    fn file_select_screen_contains_exact_static_and_generated_tail() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0; SRAM_SIZE];
        let mut rt = TestRuntime::default();
        let mut f = FileSelect {
            ram: &mut r,
            sram: &mut s,
            runtime: &mut rt,
        };
        f.build_file_select_screen();
        let static_offset = VRAM_UPLOAD_DATA + 2052;
        assert_eq!(&f.ram.slice(static_offset, 4), &[0x10, 0x42, 0, 0x27]);
        let tail = static_offset + 224;
        assert_eq!(f.ram.read_u16(tail), 0x0311);
        assert_eq!(f.ram.read_u16(tail + 2), 0x3240);
        assert_eq!(f.ram.read_u16(tail + 4), 0x347f);
        assert_eq!(f.ram.read_u8(tail + 18 * 6), 0xff);
        assert_eq!(f.ram.read_u8(SUBMODULE), 1);
        assert_eq!(f.ram.read_u8(NMI_LOAD_BG), 1);
    }

    #[test]
    fn name_and_picker_stripes_are_materialized_locally() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0; SRAM_SIZE];
        let mut rt = TestRuntime::default();
        let mut f = FileSelect {
            ram: &mut r,
            sram: &mut s,
            runtime: &mut rt,
        };
        f.trigger_name_stripes();
        assert_eq!(&f.ram.slice(VRAM_UPLOAD_DATA, 4), &[0x61, 0x29, 0, 0x25]);
        assert_eq!(f.ram.read_u8(VRAM_UPLOAD_DATA + 252), 0xff);

        f.ram.write_u16(SELECTFILE_ARR1, 1);
        f.ram.write_u8(R16, 0);
        f.choose_copy_source();
        assert_eq!(&f.ram.slice(VRAM_UPLOAD_DATA, 4), &[0x61, 0x04, 0, 0x15]);
        assert_eq!(f.ram.read_u8(VRAM_UPLOAD_DATA + 172), 0xff);
    }

    #[test]
    fn finishing_name_sets_signature_init_and_checksum() {
        let mut b = [0; WRAM_SIZE];
        let mut r = Ram::new(&mut b);
        let mut s = [0; SRAM_SIZE];
        let mut rt = TestRuntime::default();
        r.write_u8(R16, 2);
        write_u16(&mut s, 2 * SAVE_SIZE + SRAM_NAME, 6);
        let mut f = FileSelect {
            ram: &mut r,
            sram: &mut s,
            runtime: &mut rt,
        };
        assert!(f.finish_name());
        let save = &f.sram[2 * SAVE_SIZE..3 * SAVE_SIZE];
        assert_eq!(read_u16(save, SRAM_VALID), 0x55aa);
        assert_eq!(read_u16(save, SRAM_DIED_COUNTER), 0xffff);
        assert_eq!(read_u16(f.sram, SRAM_VAR1), 6);
        assert!(check_checksum(save));
        assert_eq!(f.runtime.persisted, 1);
    }
}
