//! Safe access to the C port's WRAM-backed OAM staging buffers.

use crate::ram::Ram;
use z3_core::c_types::OamEnt;

pub const OAM_BUFFER: usize = 0x800;
pub const OAM_ENTRY_COUNT: usize = 128;
pub const EXTENDED_OAM: usize = 0xa00;
pub const BYTEWISE_EXTENDED_OAM: usize = 0xa20;

#[must_use]
pub fn read(ram: &Ram<'_>, index: usize) -> OamEnt {
    assert!(index < OAM_ENTRY_COUNT);
    let address = OAM_BUFFER + index * 4;
    OamEnt {
        x: ram.read_u8(address),
        y: ram.read_u8(address + 1),
        charnum: ram.read_u8(address + 2),
        flags: ram.read_u8(address + 3),
    }
}

pub fn write(ram: &mut Ram<'_>, index: usize, entry: OamEnt) {
    assert!(index < OAM_ENTRY_COUNT);
    let address = OAM_BUFFER + index * 4;
    ram.write_u8(address, entry.x);
    ram.write_u8(address + 1, entry.y);
    ram.write_u8(address + 2, entry.charnum);
    ram.write_u8(address + 3, entry.flags);
}

pub fn set_helper0(
    ram: &mut Ram<'_>,
    index: usize,
    x: u16,
    y: u16,
    charnum: u8,
    flags: u8,
    big: u8,
) {
    let visible_y = if y.wrapping_add(0x10) < 0x100 {
        y as u8
    } else {
        0xf0
    };
    write(
        ram,
        index,
        OamEnt {
            x: x as u8,
            y: visible_y,
            charnum,
            flags,
        },
    );
    ram.write_u8(BYTEWISE_EXTENDED_OAM + index, big | ((x >> 8) as u8 & 1));
}

pub fn set_helper1(
    ram: &mut Ram<'_>,
    index: usize,
    x: u16,
    y: u8,
    charnum: u8,
    flags: u8,
    big: u8,
) {
    write(
        ram,
        index,
        OamEnt {
            x: x as u8,
            y,
            charnum,
            flags,
        },
    );
    ram.write_u8(BYTEWISE_EXTENDED_OAM + index, big | ((x >> 8) as u8 & 1));
}

pub fn set_plain(ram: &mut Ram<'_>, index: usize, x: u8, y: u8, charnum: u8, flags: u8, big: u8) {
    write(
        ram,
        index,
        OamEnt {
            x,
            y,
            charnum,
            flags,
        },
    );
    ram.write_u8(BYTEWISE_EXTENDED_OAM + index, big);
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[test]
    fn helper0_matches_visibility_and_x_high_bit_rules() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        set_helper0(&mut ram, 3, 0x1ab, 0xef, 0x42, 0x31, 2);
        assert_eq!(
            read(&ram, 3),
            OamEnt {
                x: 0xab,
                y: 0xef,
                charnum: 0x42,
                flags: 0x31
            }
        );
        assert_eq!(ram.read_u8(BYTEWISE_EXTENDED_OAM + 3), 3);
        set_helper0(&mut ram, 4, 0, 0xf0, 0, 0, 0);
        assert_eq!(read(&ram, 4).y, 0xf0);
    }
}
