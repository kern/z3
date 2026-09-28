//! NMI and graphics-upload port from `src/nmi.c`.
//!
//! The C implementation performs DMA as completed copies into the PPU arrays.
//! This module deliberately does the same instead of inventing transient
//! [`z3_snes::dma::Dma`] state that does not occur in the reference port.

use crate::ram::Ram;
use core::fmt;
use z3_snes::ppu::Ppu;

const NMI_VRAM_ADDRS: [u8; 35] = [
    0, 0, 4, 8, 12, 8, 12, 0, 4, 0, 8, 4, 12, 4, 12, 0, 8, 16, 20, 24, 28, 24, 28, 16, 20, 16, 24,
    20, 28, 20, 28, 16, 24, 96, 104,
];

pub struct NmiAssets<'a> {
    pub link_graphics: &'a [u8],
    pub bg_tilemaps: [&'a [u8]; 6],
    pub light_overworld_tilemap: &'a [u8],
}

pub trait NmiAudio {
    fn is_bugged_music_track(&self, track: u8) -> bool;
    fn play_music_track(&mut self, track: u8);
    fn read_apu_port(&mut self, port: u8) -> u8;
    fn write_apu_port(&mut self, port: u8, value: u8);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NmiError {
    InvalidSubroutine(u8),
    InvalidBackgroundSource(u8),
    InvalidVramTarget(usize),
    TruncatedData,
    UnsupportedVmain(u8),
    AssetRange {
        asset: &'static str,
        start: usize,
        len: usize,
    },
}
impl fmt::Display for NmiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for NmiError {}

fn checked<'a>(
    data: &'a [u8],
    start: usize,
    len: usize,
    asset: &'static str,
) -> Result<&'a [u8], NmiError> {
    data.get(start..start.saturating_add(len))
        .ok_or(NmiError::AssetRange { asset, start, len })
}

fn write_vram_bytes(ppu: &mut Ppu, word_addr: usize, src: &[u8]) -> Result<(), NmiError> {
    let byte_addr = word_addr
        .checked_mul(2)
        .ok_or(NmiError::InvalidVramTarget(word_addr))?;
    if byte_addr.saturating_add(src.len()) > ppu.vram.len() * 2 {
        return Err(NmiError::InvalidVramTarget(word_addr));
    }
    for (i, &value) in src.iter().enumerate() {
        let absolute = byte_addr + i;
        let word = &mut ppu.vram[absolute >> 1];
        if absolute & 1 == 0 {
            *word = (*word & 0xff00) | u16::from(value);
        } else {
            *word = (*word & 0xff) | (u16::from(value) << 8);
        }
    }
    Ok(())
}

fn write_vertical(ppu: &mut Ppu, word_addr: usize, src: &[u8]) -> Result<(), NmiError> {
    if src.len() & 1 != 0 {
        return Err(NmiError::TruncatedData);
    }
    for (i, pair) in src.chunks_exact(2).enumerate() {
        let address = word_addr + i * 32;
        *ppu.vram
            .get_mut(address)
            .ok_or(NmiError::InvalidVramTarget(address))? = u16::from_le_bytes([pair[0], pair[1]]);
    }
    Ok(())
}

fn write_low(ppu: &mut Ppu, word_addr: usize, src: &[u8]) -> Result<(), NmiError> {
    let dst = ppu
        .vram
        .get_mut(word_addr..word_addr.saturating_add(src.len()))
        .ok_or(NmiError::InvalidVramTarget(word_addr))?;
    for (word, &value) in dst.iter_mut().zip(src) {
        *word = (*word & 0xff00) | u16::from(value);
    }
    Ok(())
}

fn copy_ram_words(ram: &Ram<'_>, address: usize, dst: &mut [u16]) {
    let byte_len = dst.len() * 2;
    for (word, pair) in dst
        .iter_mut()
        .zip(ram.slice(address, byte_len).chunks_exact(2))
    {
        *word = u16::from_le_bytes([pair[0], pair[1]]);
    }
}

pub fn write_ppu_registers(ram: &Ram<'_>, ppu: &mut Ppu) {
    for (register, address) in [
        (0x23, 0x96),
        (0x24, 0x97),
        (0x25, 0x98),
        (0x30, 0x99),
        (0x31, 0x9a),
        (0x32, 0x9c),
        (0x32, 0x9d),
        (0x32, 0x9e),
        (0x2c, 0x1c),
        (0x2d, 0x1d),
        (0x2e, 0x1e),
        (0x2f, 0x1f),
    ] {
        ppu.write(register, ram.read_u8(address));
    }
    for (register, address) in [
        (0x0d, 0x120),
        (0x0e, 0x124),
        (0x0f, 0x11e),
        (0x10, 0x122),
        (0x11, 0xe4),
        (0x12, 0xea),
    ] {
        let value = ram.read_u16(address);
        ppu.write(register, value as u8);
        ppu.write(register, (value >> 8) as u8);
    }
    ppu.write(0, ram.read_u8(0x13));
    ppu.write(6, ram.read_u8(0x95));
    let mode = ram.read_u8(0x94);
    ppu.write(5, mode);
    if mode & 7 == 7 {
        ppu.write(0x1c, 0);
        ppu.write(0x1c, 0);
        ppu.write(0x1d, 0);
        ppu.write(0x1d, 0);
        for (register, address) in [(0x1f, 0x638), (0x20, 0x63a)] {
            let value = ram.read_u16(address);
            ppu.write(register, value as u8);
            ppu.write(register, (value >> 8) as u8);
        }
    }
    ppu.write(0x0b, 0x22);
    ppu.write(0x0c, 7);
}

fn update_audio(ram: &mut Ram<'_>, audio: &mut impl NmiAudio) {
    let music = ram.read_u8(0x12c);
    if music != 0 && !audio.is_bugged_music_track(music) {
        ram.write_u8(0x133, music);
        audio.play_music_track(music);
        if music < 0xf2 {
            ram.write_u8(0x130, music);
        }
        ram.write_u8(0x12c, 0);
    }
    let ambient = ram.read_u8(0x12d);
    if ambient == 0 {
        if audio.read_apu_port(1) == ram.read_u8(0x131) {
            audio.write_apu_port(1, 0);
        }
    } else {
        ram.write_u8(0x131, ambient);
        audio.write_apu_port(1, ambient);
        ram.write_u8(0x12d, 0);
    }
    audio.write_apu_port(2, ram.read_u8(0x12e));
    audio.write_apu_port(3, ram.read_u8(0x12f));
    ram.write_u8(0x12e, 0);
    ram.write_u8(0x12f, 0);
}

pub fn interrupt_nmi(
    ram: &mut Ram<'_>,
    ppu: &mut Ppu,
    assets: &NmiAssets<'_>,
    audio: &mut impl NmiAudio,
    joypad: u16,
) -> Result<(), NmiError> {
    update_audio(ram, audio);
    if ram.read_u8(0x12) == 0 {
        ram.write_u8(0x12, 1);
        do_updates(ram, ppu, assets)?;
        read_joypads(ram, joypad);
    }
    if ram.read_u8(0x12a) != 0 {
        update_irq_gfx(ram, ppu)?;
        let stack = if ram.read_u16(0x1f0a) != 0x1f31 {
            0x1f31
        } else {
            0x1f2
        };
        ram.write_u16(0x1f0a, stack);
    }
    write_ppu_registers(ram, ppu);
    Ok(())
}

pub fn read_joypads(ram: &mut Ram<'_>, input: u16) {
    let reversed = input.reverse_bits();
    let low = reversed as u8;
    let high = (reversed >> 8) as u8;
    let old_low = ram.read_u8(0xfa);
    let old_high = ram.read_u8(0xf8);
    ram.write_u8(0xf2, low);
    ram.write_u8(0xf6, (low ^ old_low) & low);
    ram.write_u8(0xfa, low);
    ram.write_u8(0xf0, high);
    ram.write_u8(0xf4, (high ^ old_high) & high);
    ram.write_u8(0xf8, high);
}

pub fn do_updates(
    ram: &mut Ram<'_>,
    ppu: &mut Ppu,
    assets: &NmiAssets<'_>,
) -> Result<(), NmiError> {
    if ram.read_u8(0x710) == 0 {
        for (dst, address, len) in [
            (0x4100, 0xace, 0x40),
            (0x4120, 0xad2, 0x40),
            (0x4140, 0xad6, 0x20),
            (0x4000, 0xacc, 0x40),
            (0x4020, 0xad0, 0x40),
            (0x4040, 0xad4, 0x20),
        ] {
            let source = usize::from(ram.read_u16(address)).wrapping_sub(0x8000);
            write_vram_bytes(
                ppu,
                dst,
                checked(assets.link_graphics, source, len, "link_graphics")?,
            )?;
        }
        for (dst, address, len) in [
            (0x4050, 0xac0, 0x40),
            (0x4070, 0xac4, 0x40),
            (0x4090, 0xac8, 0x40),
            (0x40b0, 0xae0, 0x20),
            (0x40c0, 0xad8, 0x40),
            (0x4150, 0xac2, 0x40),
            (0x4170, 0xac6, 0x40),
            (0x4190, 0xaca, 0x40),
            (0x41b0, 0xae2, 0x20),
            (0x41c0, 0xada, 0x40),
            (0x4200, 0xaec, 0x40),
            (0x4220, 0xaf0, 0x40),
            (0x4240, 0xbd40, 0x40),
            (0x4300, 0xaee, 0x40),
            (0x4320, 0xaf2, 0x40),
            (0x4340, 0xbd80, 0x40),
        ] {
            let source = if address >= 0xbd00 {
                address
            } else {
                usize::from(ram.read_u16(address))
            };
            let data = ram.slice(source, len).to_vec();
            write_vram_bytes(ppu, dst, &data)?;
        }
        if ram.read_u8(0xaf4) != 0 {
            for (dst, address) in [(0x40e0, 0xaf6), (0x41e0, 0xaf8)] {
                let source = usize::from(ram.read_u16(address));
                let data = ram.slice(source, 0x40).to_vec();
                write_vram_bytes(ppu, dst, &data)?;
            }
        }
        let source = usize::from(ram.read_u16(0xadc));
        let dst = usize::from(ram.read_u16(0x134));
        let data = ram.slice(source, 0x400).to_vec();
        write_vram_bytes(ppu, dst, &data)?;
    }
    if ram.read_u8(0x16) != 0 {
        let dst = usize::from(ram.read_u16(0x219));
        let data = ram.slice(0xc700, 330).to_vec();
        write_vram_bytes(ppu, dst, &data)?;
    }
    if ram.read_u8(0x15) != 0 {
        copy_ram_words(ram, 0xc500, &mut ppu.cgram);
    }
    ram.write_u8(0x16, 0);
    ram.write_u8(0x15, 0);
    copy_ram_words(ram, 0x800, &mut ppu.oam);
    let bg = ram.read_u8(0x14);
    if bg != 0 {
        let owned;
        let source = match bg {
            1 => {
                owned = ram.slice(0x1002, 0xeffe).to_vec();
                &owned[..]
            }
            2 => {
                owned = ram.slice(0x1000, 0xf000).to_vec();
                &owned[..]
            }
            3 => assets.bg_tilemaps[0],
            4 => {
                owned = ram.slice(0x21b, z3_core::WRAM_SIZE - 0x21b).to_vec();
                &owned[..]
            }
            5..=9 => assets.bg_tilemaps[usize::from(bg - 4)],
            other => return Err(NmiError::InvalidBackgroundSource(other)),
        };
        handle_stripes14(ppu, source)?;
        if bg == 1 {
            ram.write_u16(0x1000, 0);
        }
        ram.write_u8(0x14, 0);
    }
    let tile_dst = ram.read_u8(0x19);
    if tile_dst != 0 {
        let source = 0x10000 + usize::from(ram.read_u16(0x118));
        let data = ram.slice(source, 0x200).to_vec();
        write_vram_bytes(ppu, usize::from(tile_dst) * 256, &data)?;
        ram.write_u8(0x19, 0);
    }
    if ram.read_u8(0x18) != 0 {
        copy_packets(ram, ppu)?;
        ram.write_u8(0x18, 0);
        ram.write_u8(0x710, 0);
    }
    let index = ram.read_u8(0x17);
    ram.write_u8(0x17, 0);
    dispatch_update(index, ram, ppu, assets)
}

fn copy_packets(ram: &Ram<'_>, ppu: &mut Ppu) -> Result<(), NmiError> {
    // `uvram.data` follows the 0x100-byte pad in UploadVram_3.
    let mut cursor = 0x1100;
    loop {
        if ram.read_u16(cursor) == 0xffff {
            return Ok(());
        }
        let dst = usize::from(ram.read_u16(cursor));
        let mode = ram.read_u8(cursor + 2);
        let len = usize::from(ram.read_u8(cursor + 3));
        cursor += 4;
        let data = ram.slice(cursor, len);
        match mode {
            0x80 => write_vram_bytes(ppu, dst, data)?,
            0x81 => write_vertical(ppu, dst, data)?,
            other => return Err(NmiError::UnsupportedVmain(other)),
        }
        cursor += len;
    }
}

pub fn dispatch_update(
    index: u8,
    ram: &mut Ram<'_>,
    ppu: &mut Ppu,
    assets: &NmiAssets<'_>,
) -> Result<(), NmiError> {
    match index {
        0 | 6 => Ok(()),
        1 => upload_tilemap(ram, ppu),
        2 => {
            ram_copy(ram, ppu, 0x7c00, 0x10000, 0x7e0)?;
            ram.write_u8(0x710, 0);
            Ok(())
        }
        3 => update_ow_scroll(ram, ppu),
        4 => arbitrary(ram, ppu, 0x12000, 0, 0x80),
        5 => {
            let dst = usize::from(ram.read_u16(0x116));
            vertical_ram(ram, ppu, dst, 0xc880, 0x40)?;
            vertical_ram(ram, ppu, dst + 0x800, 0xc8c0, 0x40)
        }
        7 => light_map(ppu, assets.light_overworld_tilemap),
        8 => {
            ram_copy(ram, ppu, 0, 0x10000, 0x800)?;
            ram_copy(ram, ppu, 0x800, 0x10800, 0x800)
        }
        9 => copy_and_enable(ram, ppu, 0x2c00, 0x10000, 0x1000),
        10 => copy_and_enable(ram, ppu, 0x3400, 0x11000, 0x1000),
        11 => ram_copy(
            ram,
            ppu,
            usize::from(ram.read_u8(0x116)) * 256,
            0x11000,
            0x400,
        ),
        12 => arbitrary(ram, ppu, 0x13000, 0x40, 0x80),
        13 => arbitrary(ram, ppu, 0x12000, 0, 0x40),
        14..=17 => copy_and_enable(
            ram,
            ppu,
            0x2000 + usize::from(index - 14) * 0x800,
            0x10000,
            0x1000,
        ),
        18 => copy_and_enable(ram, ppu, 0x4400, 0x10000, 0x800),
        19 => copy_and_enable(ram, ppu, 0x5000, 0x10000, 0x1000),
        20 => copy_and_enable(ram, ppu, 0x5800, 0x10000, 0x1000),
        21 => dark_map(ram, ppu),
        22 => {
            ram_copy(ram, ppu, 0x7800, 0x2000, 0x800)?;
            ram_copy(ram, ppu, 0x7d00, 0x3400, 0x600)
        }
        23 => ram_copy(ram, ppu, 0x3d00, 0x10000, 0x100),
        24 => ram_copy(ram, ppu, 0x3ed0, 0x10000, 0x40),
        other => Err(NmiError::InvalidSubroutine(other)),
    }
}

fn ram_copy(
    ram: &Ram<'_>,
    ppu: &mut Ppu,
    dst: usize,
    src: usize,
    len: usize,
) -> Result<(), NmiError> {
    write_vram_bytes(ppu, dst, ram.slice(src, len))
}
fn vertical_ram(
    ram: &Ram<'_>,
    ppu: &mut Ppu,
    dst: usize,
    src: usize,
    len: usize,
) -> Result<(), NmiError> {
    write_vertical(ppu, dst, ram.slice(src, len))
}
fn copy_and_enable(
    ram: &mut Ram<'_>,
    ppu: &mut Ppu,
    dst: usize,
    src: usize,
    len: usize,
) -> Result<(), NmiError> {
    ram_copy(ram, ppu, dst, src, len)?;
    ram.write_u8(0x710, 0);
    Ok(())
}

fn upload_tilemap(ram: &mut Ram<'_>, ppu: &mut Ppu) -> Result<(), NmiError> {
    let index = usize::from(ram.read_u8(0x116));
    let dst = *NMI_VRAM_ADDRS
        .get(index)
        .ok_or(NmiError::InvalidVramTarget(index))?;
    ram_copy(ram, ppu, usize::from(dst) << 8, 0x1000, 0x800)?;
    ram.write_u16(0x1000, 0);
    ram.write_u8(0x710, 0);
    Ok(())
}

fn update_ow_scroll(ram: &mut Ram<'_>, ppu: &mut Ppu) -> Result<(), NmiError> {
    // C starts at `uvram.data`, not at the containing struct's base.
    let mut cursor = 0x1100;
    let header = ram.read_u16(cursor);
    let vertical = header & 0x8000 != 0;
    let len = usize::from(header & 0x3fff);
    let copy_len = len & !1;
    cursor += 2;
    loop {
        let dst = usize::from(ram.read_u16(cursor));
        cursor += 2;
        let data = ram.slice(cursor, copy_len);
        if vertical {
            write_vertical(ppu, dst, data)?
        } else {
            write_vram_bytes(ppu, dst, data)?
        }
        cursor += copy_len;
        if ram.read_u8(cursor + 1) & 0x80 != 0 {
            break;
        }
    }
    ram.write_u8(0x710, 0);
    Ok(())
}

fn arbitrary(
    ram: &mut Ram<'_>,
    ppu: &mut Ppu,
    mut src: usize,
    mut i: usize,
    end: usize,
) -> Result<(), NmiError> {
    loop {
        let dst = usize::from(ram.read_u16(0x14000 + i));
        ram_copy(ram, ppu, dst, src, 0x80)?;
        src += 0x80;
        i += 2;
        if i == end {
            break;
        }
    }
    ram.write_u8(0x710, 0);
    Ok(())
}

fn light_map(ppu: &mut Ppu, source: &[u8]) -> Result<(), NmiError> {
    let mut cursor = 0;
    for base in [0, 0x20, 0x1000, 0x1020] {
        let mut dst = base;
        for _ in 0..0x20 {
            write_low(
                ppu,
                dst,
                checked(source, cursor, 0x20, "light_overworld_tilemap")?,
            )?;
            cursor += 0x20;
            dst += 0x80;
        }
    }
    Ok(())
}
fn dark_map(ram: &Ram<'_>, ppu: &mut Ppu) -> Result<(), NmiError> {
    let (mut src, mut dst) = (0x1000, 0x810);
    for _ in 0..0x20 {
        write_low(ppu, dst, ram.slice(src, 0x20))?;
        src += 0x20;
        dst += 0x80;
    }
    Ok(())
}

pub fn handle_stripes14(ppu: &mut Ppu, data: &[u8]) -> Result<(), NmiError> {
    let mut cursor = 0;
    loop {
        if data.get(cursor).ok_or(NmiError::TruncatedData)? & 0x80 != 0 {
            return Ok(());
        }
        let h = data
            .get(cursor..cursor + 4)
            .ok_or(NmiError::TruncatedData)?;
        let dst = usize::from(u16::from_be_bytes([h[0], h[1]]));
        let vertical = h[2] & 0x80 != 0;
        let fill = h[2] & 0x40 != 0;
        let len = usize::from(u16::from_be_bytes([h[2], h[3]]) & 0x3fff) + 1;
        cursor += 4;
        if fill {
            let pair = data
                .get(cursor..cursor + 2)
                .ok_or(NmiError::TruncatedData)?;
            let repeated = pair.repeat((len + 1) >> 1);
            if vertical {
                write_vertical(ppu, dst, &repeated)?
            } else {
                write_vram_bytes(ppu, dst, &repeated)?
            }
            cursor += 2;
        } else {
            let source = data
                .get(cursor..cursor + len)
                .ok_or(NmiError::TruncatedData)?;
            if vertical {
                write_vertical(ppu, dst, source)?
            } else {
                write_vram_bytes(ppu, dst, source)?
            }
            cursor += len;
        }
    }
}

pub fn update_irq_gfx(ram: &mut Ram<'_>, ppu: &mut Ppu) -> Result<(), NmiError> {
    if ram.read_u8(0x1f0c) != 0 {
        ram_copy(ram, ppu, 0x5800, 0xe800, 0x800)?;
        ram.write_u8(0x1f0c, 0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;
    fn assets() -> NmiAssets<'static> {
        static EMPTY: [u8; 0] = [];
        NmiAssets {
            link_graphics: &EMPTY,
            bg_tilemaps: [&EMPTY; 6],
            light_overworld_tilemap: &EMPTY,
        }
    }
    #[test]
    fn joypad_reversal_and_edges() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        read_joypads(&mut ram, 0x8001);
        assert_eq!((ram.read_u8(0xf0), ram.read_u8(0xf2)), (0x80, 1));
        assert_eq!((ram.read_u8(0xf4), ram.read_u8(0xf6)), (0x80, 1));
        read_joypads(&mut ram, 0x8001);
        assert_eq!((ram.read_u8(0xf4), ram.read_u8(0xf6)), (0, 0));
    }
    #[test]
    fn dispatch_character_upload() {
        let mut bytes = [0; WRAM_SIZE];
        for (i, b) in bytes[0x10000..0x11000].iter_mut().enumerate() {
            *b = i as u8;
        }
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(0x710, 1);
        let mut ppu = Ppu::new();
        dispatch_update(14, &mut ram, &mut ppu, &assets()).unwrap();
        assert_eq!(ppu.vram[0x2000], 0x0100);
        assert_eq!(ppu.vram[0x27ff], 0xfffe);
        assert_eq!(ram.read_u8(0x710), 0);
    }
    #[test]
    fn packet_plain_and_vertical_copies() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let packet = [
            0, 1, 0x80, 4, 1, 2, 3, 4, 0x10, 1, 0x81, 4, 5, 6, 7, 8, 0xff, 0xff,
        ];
        ram.slice_mut(0x1100, packet.len()).copy_from_slice(&packet);
        let mut ppu = Ppu::new();
        copy_packets(&ram, &mut ppu).unwrap();
        assert_eq!(ppu.vram[0x100..0x102], [0x0201, 0x0403]);
        assert_eq!(ppu.vram[0x110], 0x0605);
        assert_eq!(ppu.vram[0x130], 0x0807);
    }
    #[test]
    fn overworld_scroll_reads_uvram_data_after_struct_padding() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        // Horizontal, two-byte rows; one row ending with the high-bit sentinel.
        ram.slice_mut(0x1100, 8)
            .copy_from_slice(&[2, 0, 0x20, 0x04, 0x34, 0x12, 0xff, 0xff]);
        let mut ppu = Ppu::new();
        update_ow_scroll(&mut ram, &mut ppu).unwrap();
        assert_eq!(ppu.vram[0x420], 0x1234);
    }
    #[test]
    fn overworld_scroll_truncates_odd_lengths_to_whole_words() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.slice_mut(0x1100, 8)
            .copy_from_slice(&[3, 0, 0x20, 0x04, 0x34, 0x12, 0xff, 0xff]);
        let mut ppu = Ppu::new();
        update_ow_scroll(&mut ram, &mut ppu).unwrap();
        assert_eq!(ppu.vram[0x420], 0x1234);
        assert_eq!(ppu.vram[0x421], 0);
    }
    #[test]
    fn stripe_fill_header_is_big_endian() {
        let mut ppu = Ppu::new();
        handle_stripes14(&mut ppu, &[1, 0x20, 0x40, 3, 0x34, 0x12, 0x80, 0, 0, 0]).unwrap();
        assert_eq!(ppu.vram[0x120..0x122], [0x1234; 2]);
    }

    #[test]
    fn background_source_four_reads_the_wram_stripe_stream() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(0x710, 1);
        ram.write_u8(0x14, 4);
        ram.slice_mut(0x21b, 7)
            .copy_from_slice(&[0x01, 0x20, 0x00, 0x01, 0x34, 0x12, 0x80]);
        let mut ppu = Ppu::new();
        do_updates(&mut ram, &mut ppu, &assets()).unwrap();
        assert_eq!(ppu.vram[0x120], 0x1234);
        assert_eq!(ram.read_u8(0x14), 0);
    }
}
