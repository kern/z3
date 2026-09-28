//! Exact DMA/HDMA behavior from `snes/dma.c`.

use crate::bus::Bus;
use crate::saveload::{SaveLoad, SaveLoadError, bool_byte, get_u16, get_u32, put_u16, put_u32};

pub const DMA_SAVE_BLOCK_LEN: usize = 192;

const B_ADDRESS_OFFSETS: [[u8; 4]; 8] = [
    [0, 0, 0, 0],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
    [0, 1, 2, 3],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
];

const TRANSFER_LENGTH: [usize; 8] = [1, 2, 2, 4, 4, 4, 2, 4];

/// Canonical per-channel state. Every field from the C structure remains
/// visible so save states and the differential verifier can serialize it in a
/// fixed order without depending on Rust's struct layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DmaChannel {
    pub b_address: u8,
    pub a_bank: u8,
    pub indirect_bank: u8,
    pub repeat_count: u8,
    pub a_address: u16,
    /// DMA byte count, or the indirect HDMA address.
    pub size: u16,
    pub table_address: u16,
    pub unused_byte: u8,
    pub dma_active: bool,
    pub hdma_active: bool,
    pub mode: u8,
    pub fixed: bool,
    pub decrement: bool,
    pub indirect: bool,
    pub from_b: bool,
    pub unused_bit: bool,
    pub do_transfer: bool,
    pub terminated: bool,
    pub offset_index: u8,
}

impl Default for DmaChannel {
    fn default() -> Self {
        Self {
            b_address: 0xff,
            a_bank: 0xff,
            indirect_bank: 0xff,
            repeat_count: 0xff,
            a_address: 0xffff,
            size: 0xffff,
            table_address: 0xffff,
            unused_byte: 0xff,
            dma_active: false,
            hdma_active: false,
            mode: 7,
            fixed: true,
            decrement: true,
            indirect: true,
            from_b: true,
            unused_bit: true,
            do_transfer: false,
            terminated: false,
            offset_index: 0,
        }
    }
}

/// The eight SNES DMA channels and their shared timing state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dma {
    pub channels: [DmaChannel; 8],
    pub hdma_timer: u16,
    pub dma_timer: u32,
    pub dma_busy: bool,
}

impl Dma {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Transfers the byte range copied by C `dma_saveload`. Padding is kept
    /// explicit and zeroed for newly-created streams.
    pub fn saveload(&mut self, io: &mut impl SaveLoad) -> Result<(), SaveLoadError> {
        let mut block = [0; DMA_SAVE_BLOCK_LEN];
        if !io.is_loading() {
            self.write_save_block(&mut block)?;
        }
        io.transfer(&mut block)?;
        if io.is_loading() {
            self.read_save_block(&block)?;
        }
        Ok(())
    }

    pub fn write_save_block(&self, block: &mut [u8]) -> Result<(), SaveLoadError> {
        if block.len() != DMA_SAVE_BLOCK_LEN {
            return Err(SaveLoadError::WrongBlockSize {
                expected: DMA_SAVE_BLOCK_LEN,
                actual: block.len(),
            });
        }
        for (index, channel) in self.channels.iter().enumerate() {
            let at = index * 22;
            block[at] = channel.b_address;
            block[at + 1] = channel.a_bank;
            block[at + 2] = channel.indirect_bank;
            block[at + 3] = channel.repeat_count;
            put_u16(block, at + 4, channel.a_address);
            put_u16(block, at + 6, channel.size);
            put_u16(block, at + 8, channel.table_address);
            block[at + 10] = channel.unused_byte;
            block[at + 11] = bool_byte(channel.dma_active);
            block[at + 12] = bool_byte(channel.hdma_active);
            block[at + 13] = channel.mode;
            block[at + 14] = bool_byte(channel.fixed);
            block[at + 15] = bool_byte(channel.decrement);
            block[at + 16] = bool_byte(channel.indirect);
            block[at + 17] = bool_byte(channel.from_b);
            block[at + 18] = bool_byte(channel.unused_bit);
            block[at + 19] = bool_byte(channel.do_transfer);
            block[at + 20] = bool_byte(channel.terminated);
            block[at + 21] = channel.offset_index;
        }
        put_u16(block, 176, self.hdma_timer);
        put_u32(block, 180, self.dma_timer);
        block[184] = bool_byte(self.dma_busy);
        Ok(())
    }

    pub fn read_save_block(&mut self, block: &[u8]) -> Result<(), SaveLoadError> {
        if block.len() != DMA_SAVE_BLOCK_LEN {
            return Err(SaveLoadError::WrongBlockSize {
                expected: DMA_SAVE_BLOCK_LEN,
                actual: block.len(),
            });
        }
        for (index, channel) in self.channels.iter_mut().enumerate() {
            let at = index * 22;
            channel.b_address = block[at];
            channel.a_bank = block[at + 1];
            channel.indirect_bank = block[at + 2];
            channel.repeat_count = block[at + 3];
            channel.a_address = get_u16(block, at + 4);
            channel.size = get_u16(block, at + 6);
            channel.table_address = get_u16(block, at + 8);
            channel.unused_byte = block[at + 10];
            channel.dma_active = block[at + 11] != 0;
            channel.hdma_active = block[at + 12] != 0;
            channel.mode = block[at + 13];
            channel.fixed = block[at + 14] != 0;
            channel.decrement = block[at + 15] != 0;
            channel.indirect = block[at + 16] != 0;
            channel.from_b = block[at + 17] != 0;
            channel.unused_bit = block[at + 18] != 0;
            channel.do_transfer = block[at + 19] != 0;
            channel.terminated = block[at + 20] != 0;
            channel.offset_index = block[at + 21];
        }
        self.hdma_timer = get_u16(block, 176);
        self.dma_timer = get_u32(block, 180);
        self.dma_busy = block[184] != 0;
        Ok(())
    }

    /// Reads one of the `$43x0-$43xF` channel registers.
    #[must_use]
    pub fn read_register(&self, bus: &impl Bus, address: u16) -> u8 {
        let channel = usize::from((address & 0x70) >> 4);
        let state = &self.channels[channel];
        match address & 0x0f {
            0x0 => {
                state.mode
                    | (u8::from(state.fixed) << 3)
                    | (u8::from(state.decrement) << 4)
                    | (u8::from(state.unused_bit) << 5)
                    | (u8::from(state.indirect) << 6)
                    | (u8::from(state.from_b) << 7)
            }
            0x1 => state.b_address,
            0x2 => state.a_address as u8,
            0x3 => (state.a_address >> 8) as u8,
            0x4 => state.a_bank,
            0x5 => state.size as u8,
            0x6 => (state.size >> 8) as u8,
            0x7 => state.indirect_bank,
            0x8 => state.table_address as u8,
            0x9 => (state.table_address >> 8) as u8,
            0xa => state.repeat_count,
            0xb | 0xf => state.unused_byte,
            _ => bus.open_bus(),
        }
    }

    /// Writes one of the `$43x0-$43xF` channel registers.
    pub fn write_register(&mut self, address: u16, value: u8) {
        let channel = usize::from((address & 0x70) >> 4);
        let state = &mut self.channels[channel];
        match address & 0x0f {
            0x0 => {
                state.mode = value & 7;
                state.fixed = value & 0x08 != 0;
                state.decrement = value & 0x10 != 0;
                state.unused_bit = value & 0x20 != 0;
                state.indirect = value & 0x40 != 0;
                state.from_b = value & 0x80 != 0;
            }
            0x1 => state.b_address = value,
            0x2 => state.a_address = (state.a_address & 0xff00) | u16::from(value),
            0x3 => state.a_address = (state.a_address & 0x00ff) | (u16::from(value) << 8),
            0x4 => state.a_bank = value,
            0x5 => state.size = (state.size & 0xff00) | u16::from(value),
            0x6 => state.size = (state.size & 0x00ff) | (u16::from(value) << 8),
            0x7 => state.indirect_bank = value,
            0x8 => state.table_address = (state.table_address & 0xff00) | u16::from(value),
            0x9 => {
                state.table_address = (state.table_address & 0x00ff) | (u16::from(value) << 8);
            }
            0xa => state.repeat_count = value,
            0xb | 0xf => state.unused_byte = value,
            _ => {}
        }
    }

    /// Performs one byte of the first active general-DMA channel.
    pub fn do_dma(&mut self, bus: &mut impl Bus) {
        let Some(index) = self.channels.iter().position(|channel| channel.dma_active) else {
            self.dma_busy = false;
            return;
        };

        let channel = &mut self.channels[index];
        let b_offset =
            B_ADDRESS_OFFSETS[usize::from(channel.mode)][usize::from(channel.offset_index)];
        transfer_byte(
            bus,
            channel.a_address,
            channel.a_bank,
            channel.b_address.wrapping_add(b_offset),
            channel.from_b,
        );
        channel.offset_index = channel.offset_index.wrapping_add(1) & 3;
        self.dma_timer = self.dma_timer.wrapping_add(6);

        if !channel.fixed {
            channel.a_address = if channel.decrement {
                channel.a_address.wrapping_sub(1)
            } else {
                channel.a_address.wrapping_add(1)
            };
        }
        channel.size = channel.size.wrapping_sub(1);
        if channel.size == 0 {
            channel.offset_index = 0;
            channel.dma_active = false;
            self.dma_timer = self.dma_timer.wrapping_add(8);
        }
    }

    /// Initializes active HDMA channels at the start of a frame.
    pub fn init_hdma(&mut self, bus: &mut impl Bus) {
        self.hdma_timer = 0;
        let mut hdma_happened = false;

        for channel in &mut self.channels {
            if channel.hdma_active {
                hdma_happened = true;
                channel.dma_active = false;
                channel.offset_index = 0;
                channel.table_address = channel.a_address;
                channel.repeat_count = read_table_byte(bus, channel);
                self.hdma_timer = self.hdma_timer.wrapping_add(8);
                if channel.indirect {
                    let low = read_table_byte(bus, channel);
                    let high = read_table_byte(bus, channel);
                    channel.size = u16::from_le_bytes([low, high]);
                    self.hdma_timer = self.hdma_timer.wrapping_add(16);
                }
                channel.do_transfer = true;
            } else {
                channel.do_transfer = false;
            }
            channel.terminated = false;
        }

        if hdma_happened {
            self.hdma_timer = self.hdma_timer.wrapping_add(16);
        }
    }

    /// Performs the active channels' work for one HDMA scanline.
    pub fn do_hdma(&mut self, bus: &mut impl Bus) {
        self.hdma_timer = 0;
        let mut hdma_happened = false;

        for channel in &mut self.channels {
            if !channel.hdma_active || channel.terminated {
                continue;
            }
            hdma_happened = true;
            channel.dma_active = false;
            channel.offset_index = 0;
            self.hdma_timer = self.hdma_timer.wrapping_add(8);

            if channel.do_transfer {
                let mode = usize::from(channel.mode);
                let offsets = &B_ADDRESS_OFFSETS[mode][..TRANSFER_LENGTH[mode]];
                for &b_offset in offsets {
                    self.hdma_timer = self.hdma_timer.wrapping_add(8);
                    let b_address = channel.b_address.wrapping_add(b_offset);
                    if channel.indirect {
                        transfer_byte(
                            bus,
                            channel.size,
                            channel.indirect_bank,
                            b_address,
                            channel.from_b,
                        );
                        channel.size = channel.size.wrapping_add(1);
                    } else {
                        transfer_byte(
                            bus,
                            channel.table_address,
                            channel.a_bank,
                            b_address,
                            channel.from_b,
                        );
                        channel.table_address = channel.table_address.wrapping_add(1);
                    }
                }
            }

            channel.repeat_count = channel.repeat_count.wrapping_sub(1);
            channel.do_transfer = channel.repeat_count & 0x80 != 0;
            if channel.repeat_count & 0x7f == 0 {
                channel.repeat_count = read_table_byte(bus, channel);
                if channel.indirect {
                    let low = read_table_byte(bus, channel);
                    let high = read_table_byte(bus, channel);
                    channel.size = u16::from_le_bytes([low, high]);
                    self.hdma_timer = self.hdma_timer.wrapping_add(16);
                }
                if channel.repeat_count == 0 {
                    channel.terminated = true;
                }
                channel.do_transfer = true;
            }
        }

        if hdma_happened {
            self.hdma_timer = self.hdma_timer.wrapping_add(16);
        }
    }

    /// Advances DMA timing by the two master cycles represented by one C
    /// `dma_cycle` call. General DMA's timer gate is intentionally absent: it
    /// is commented out in the reference implementation.
    pub fn cycle(&mut self, bus: &mut impl Bus) -> bool {
        if self.hdma_timer > 0 {
            self.hdma_timer = self.hdma_timer.wrapping_sub(2);
            true
        } else if self.dma_busy {
            self.do_dma(bus);
            true
        } else {
            false
        }
    }

    pub fn start_dma(&mut self, value: u8, hdma: bool) {
        for (index, channel) in self.channels.iter_mut().enumerate() {
            let active = value & (1 << index) != 0;
            if hdma {
                channel.hdma_active = active;
            } else {
                channel.dma_active = active;
            }
        }
        if !hdma {
            self.dma_busy = value != 0;
            if self.dma_busy {
                self.dma_timer = self.dma_timer.wrapping_add(16);
            }
        }
    }
}

fn read_table_byte(bus: &mut impl Bus, channel: &mut DmaChannel) -> u8 {
    let address = (u32::from(channel.a_bank) << 16) | u32::from(channel.table_address);
    let value = bus.read(address);
    channel.table_address = channel.table_address.wrapping_add(1);
    value
}

fn transfer_byte(bus: &mut impl Bus, a_address: u16, a_bank: u8, b_address: u8, from_b: bool) {
    let address = (u32::from(a_bank) << 16) | u32::from(a_address);
    if from_b {
        let value = bus.read_b_bus(b_address);
        bus.write(address, value);
    } else {
        let value = bus.read(address);
        bus.write_b_bus(b_address, value);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[derive(Debug)]
    struct MockBus {
        open_bus: u8,
        a_bus: HashMap<u32, u8>,
        b_bus: [u8; 256],
        a_writes: Vec<(u32, u8)>,
        b_writes: Vec<(u8, u8)>,
    }

    impl Default for MockBus {
        fn default() -> Self {
            Self {
                open_bus: 0,
                a_bus: HashMap::new(),
                b_bus: [0; 256],
                a_writes: Vec::new(),
                b_writes: Vec::new(),
            }
        }
    }

    impl Bus for MockBus {
        fn open_bus(&self) -> u8 {
            self.open_bus
        }

        fn read(&mut self, address: u32) -> u8 {
            self.a_bus.get(&address).copied().unwrap_or(self.open_bus)
        }

        fn write(&mut self, address: u32, value: u8) {
            self.a_bus.insert(address, value);
            self.a_writes.push((address, value));
        }

        fn read_b_bus(&mut self, address: u8) -> u8 {
            self.b_bus[usize::from(address)]
        }

        fn write_b_bus(&mut self, address: u8, value: u8) {
            self.b_bus[usize::from(address)] = value;
            self.b_writes.push((address, value));
        }
    }

    #[test]
    fn reset_and_register_io_match_c_defaults_and_bit_packing() {
        let mut dma = Dma::new();
        let mut bus = MockBus {
            open_bus: 0x5a,
            ..MockBus::default()
        };
        assert_eq!(dma.read_register(&bus, 0x4300), 0xff);
        assert_eq!(dma.read_register(&bus, 0x430c), 0x5a);

        dma.write_register(0x4350, 0xdb);
        dma.write_register(0x4351, 0x80);
        dma.write_register(0x4352, 0x34);
        dma.write_register(0x4353, 0x12);
        dma.write_register(0x4354, 0x7e);
        dma.write_register(0x4355, 0x78);
        dma.write_register(0x4356, 0x56);
        dma.write_register(0x4357, 0x40);
        dma.write_register(0x4358, 0xbc);
        dma.write_register(0x4359, 0x9a);
        dma.write_register(0x435a, 0x81);
        dma.write_register(0x435f, 0xee);
        assert_eq!(dma.read_register(&bus, 0x4350), 0xdb);
        assert_eq!(dma.channels[5].a_address, 0x1234);
        assert_eq!(dma.channels[5].size, 0x5678);
        assert_eq!(dma.channels[5].table_address, 0x9abc);
        assert_eq!(dma.read_register(&bus, 0x435b), 0xee);

        dma.reset();
        assert_eq!(dma, Dma::default());
        // Keep this mutable to ensure the test bus implements the full trait.
        bus.write_b_bus(0, 1);
    }

    #[test]
    fn general_dma_uses_mode_pattern_increment_and_channel_timing() {
        let mut dma = Dma::new();
        let mut bus = MockBus::default();
        bus.a_bus.insert(0x7e_ffff, 0x11);
        bus.a_bus.insert(0x7e_0000, 0x22);
        bus.a_bus.insert(0x7e_0001, 0x33);
        bus.a_bus.insert(0x7e_0002, 0x44);

        let channel = &mut dma.channels[0];
        channel.mode = 1;
        channel.b_address = 0x20;
        channel.a_bank = 0x7e;
        channel.a_address = 0xffff;
        channel.size = 4;
        channel.fixed = false;
        channel.decrement = false;
        channel.from_b = false;
        dma.start_dma(1, false);

        for _ in 0..4 {
            assert!(dma.cycle(&mut bus));
        }
        assert_eq!(
            bus.b_writes,
            vec![(0x20, 0x11), (0x21, 0x22), (0x20, 0x33), (0x21, 0x44)]
        );
        assert_eq!(dma.channels[0].a_address, 3);
        assert_eq!(dma.channels[0].size, 0);
        assert!(!dma.channels[0].dma_active);
        assert_eq!(dma.dma_timer, 16 + 4 * 6 + 8);
        // One final cycle observes that no channels remain and clears busy.
        assert!(dma.cycle(&mut bus));
        assert!(!dma.dma_busy);
        assert!(!dma.cycle(&mut bus));
    }

    #[test]
    fn reverse_fixed_dma_reads_b_bus_without_changing_a_address() {
        let mut dma = Dma::new();
        let mut bus = MockBus::default();
        bus.b_bus[0x40] = 0xab;
        let channel = &mut dma.channels[2];
        channel.mode = 0;
        channel.b_address = 0x40;
        channel.a_bank = 0x12;
        channel.a_address = 0x3456;
        channel.size = 1;
        channel.fixed = true;
        channel.decrement = true;
        channel.from_b = true;
        dma.start_dma(1 << 2, false);
        dma.do_dma(&mut bus);

        assert_eq!(bus.a_writes, vec![(0x12_3456, 0xab)]);
        assert_eq!(dma.channels[2].a_address, 0x3456);
    }

    #[test]
    fn decrementing_dma_wraps_the_sixteen_bit_a_address() {
        let mut dma = Dma::new();
        let mut bus = MockBus::default();
        bus.a_bus.insert(0x34_0000, 0xa0);
        bus.a_bus.insert(0x34_ffff, 0xb0);
        let channel = &mut dma.channels[0];
        channel.mode = 0;
        channel.b_address = 0x10;
        channel.a_bank = 0x34;
        channel.a_address = 0;
        channel.size = 2;
        channel.fixed = false;
        channel.decrement = true;
        channel.from_b = false;
        dma.start_dma(1, false);
        dma.do_dma(&mut bus);
        dma.do_dma(&mut bus);

        assert_eq!(bus.b_writes, vec![(0x10, 0xa0), (0x10, 0xb0)]);
        assert_eq!(dma.channels[0].a_address, 0xfffe);
    }

    #[test]
    fn hdma_direct_transfer_loads_descriptor_and_terminates() {
        let mut dma = Dma::new();
        let mut bus = MockBus::default();
        bus.a_bus.insert(0x12_1000, 0x81);
        bus.a_bus.insert(0x12_1001, 0xaa);
        bus.a_bus.insert(0x12_1002, 0xbb);
        bus.a_bus.insert(0x12_1003, 0x00);

        let channel = &mut dma.channels[0];
        channel.mode = 1;
        channel.b_address = 0x20;
        channel.a_bank = 0x12;
        channel.a_address = 0x1000;
        channel.indirect = false;
        channel.from_b = false;
        dma.start_dma(1, true);
        dma.init_hdma(&mut bus);
        assert_eq!(dma.hdma_timer, 24);
        assert_eq!(dma.channels[0].table_address, 0x1001);

        dma.do_hdma(&mut bus);
        assert_eq!(bus.b_writes, vec![(0x20, 0xaa), (0x21, 0xbb)]);
        assert_eq!(dma.channels[0].table_address, 0x1004);
        assert!(dma.channels[0].terminated);
        assert_eq!(dma.hdma_timer, 40);
    }

    #[test]
    fn hdma_indirect_uses_indirect_bank_and_cycle_drains_timer_first() {
        let mut dma = Dma::new();
        let mut bus = MockBus::default();
        bus.a_bus.insert(0x01_2000, 0x81);
        bus.a_bus.insert(0x01_2001, 0x34);
        bus.a_bus.insert(0x01_2002, 0x12);
        bus.a_bus.insert(0x02_1234, 0x66);
        bus.a_bus.insert(0x01_2003, 0x00);
        bus.a_bus.insert(0x01_2004, 0x00);
        bus.a_bus.insert(0x01_2005, 0x00);

        let channel = &mut dma.channels[0];
        channel.mode = 0;
        channel.b_address = 0x30;
        channel.a_bank = 1;
        channel.a_address = 0x2000;
        channel.indirect = true;
        channel.indirect_bank = 2;
        channel.from_b = false;
        dma.start_dma(1, true);
        dma.init_hdma(&mut bus);
        assert_eq!(dma.hdma_timer, 40);
        assert!(dma.cycle(&mut bus));
        assert_eq!(dma.hdma_timer, 38);
        assert!(bus.b_writes.is_empty());

        dma.do_hdma(&mut bus);
        assert_eq!(bus.b_writes, vec![(0x30, 0x66)]);
        assert_eq!(dma.channels[0].size, 0);
        assert!(dma.channels[0].terminated);
        assert_eq!(dma.hdma_timer, 48);
    }

    #[test]
    fn hdma_repeat_count_without_high_bit_skips_following_scanline() {
        let mut dma = Dma::new();
        let mut bus = MockBus::default();
        bus.a_bus.insert(0x03_4000, 0x02);
        bus.a_bus.insert(0x03_4001, 0x77);
        bus.a_bus.insert(0x03_4002, 0x00);

        let channel = &mut dma.channels[0];
        channel.mode = 0;
        channel.b_address = 0x50;
        channel.a_bank = 3;
        channel.a_address = 0x4000;
        channel.indirect = false;
        channel.from_b = false;
        dma.start_dma(1, true);
        dma.init_hdma(&mut bus);

        dma.do_hdma(&mut bus);
        assert_eq!(bus.b_writes, vec![(0x50, 0x77)]);
        assert_eq!(dma.channels[0].repeat_count, 1);
        assert!(!dma.channels[0].do_transfer);

        dma.do_hdma(&mut bus);
        assert_eq!(bus.b_writes, vec![(0x50, 0x77)]);
        assert!(dma.channels[0].terminated);
        assert_eq!(dma.channels[0].table_address, 0x4003);
    }
}
