//! Live schemas over the one canonical SPC RAM image.
//!
//! Instrument and DSP-preset rows overlap. These short-lived views must never
//! become decoded table caches; uploads, imports and DSP cycles can alter RAM.
use super::{PatternCursor, SfxPosition};
pub(super) const EFFECT_BYTE_LENGTH: [u8; 27] = [
    1, 1, 2, 3, 0, 1, 2, 1, 2, 1, 1, 3, 0, 1, 2, 3, 1, 3, 3, 0, 1, 3, 0, 3, 3, 3, 1,
];
pub(super) const FIR: [i8; 32] = [
    127, 0, 0, 0, 0, 0, 0, 0, 88, -65, -37, -16, -2, 7, 12, 12, 12, 33, 43, 43, 19, -2, -13, -7,
    52, 51, 0, -39, -27, 1, -4, -21,
];

pub(super) const BASE_NOTE_FREQS: [u16; 13] = [
    2143, 2270, 2405, 2548, 2700, 2860, 3030, 3211, 3402, 3604, 3818, 4045, 4286,
];
const PAN_CURVE: [u8; 22] = [
    0, 1, 3, 7, 13, 21, 30, 41, 52, 66, 81, 94, 103, 110, 115, 119, 122, 124, 125, 126, 127, 127,
];
const NOTE_VOLUME: [u8; 16] = [
    25, 50, 76, 101, 114, 127, 140, 152, 165, 178, 191, 203, 216, 229, 242, 252,
];
const NOTE_GATE: [u8; 8] = [50, 101, 127, 152, 178, 203, 229, 252];

pub(super) struct Instrument {
    pub source: u8,
    pub adsr1: u8,
    pub adsr2: u8,
    pub gain: u8,
    pub pitch_base: u16,
}
pub(super) struct DspPreset {
    pub voice_registers: [u8; 8],
    pub pitch_base: u16,
}
pub(super) struct NoteParameters {
    pub gate: u8,
    pub volume: u8,
}

/// Lossless address of an eight-channel phrase table. Entries are read and
/// published one at a time, not preloaded before voice initialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelPatternTable(u16);
impl ChannelPatternTable {
    pub const fn at(address: u16) -> Self {
        Self(address)
    }
    pub const fn address(self) -> u16 {
        self.0
    }
    pub(super) fn channel(self, ram: &[u8; 65536], channel: usize) -> PatternCursor {
        PatternCursor::at(word(
            ram,
            self.0.wrapping_add((channel as u16).wrapping_mul(2)),
        ))
    }
}

pub(super) fn word(ram: &[u8; 65536], address: u16) -> u16 {
    u16::from_le_bytes([
        ram[usize::from(address)],
        ram[usize::from(address.wrapping_add(1))],
    ])
}

pub(super) struct SpcData<'a> {
    ram: &'a [u8; 65536],
}
impl<'a> SpcData<'a> {
    pub fn new(ram: &'a [u8; 65536]) -> Self {
        Self { ram }
    }
    pub fn instrument(&self, resolved_id: u8) -> Instrument {
        let at = 0x3d00 + usize::from(resolved_id) * 6;
        let bytes = &self.ram[at..at + 6];
        Instrument {
            source: bytes[0],
            adsr1: bytes[1],
            adsr2: bytes[2],
            gain: bytes[3],
            pitch_base: u16::from_be_bytes([bytes[4], bytes[5]]),
        }
    }
    pub fn dsp_preset(&self, id: u8) -> DspPreset {
        let at = 0x3e00 + usize::from(id) * 9;
        DspPreset {
            voice_registers: self.ram[at..at + 8].try_into().unwrap(),
            pitch_base: u16::from(self.ram[at + 8]) << 8,
        }
    }
    pub fn pan_segment(&self, index: usize) -> (u8, u8) {
        if index >= 21 {
            (self.ram[index + 0x1178], self.ram[index + 0x1179])
        } else {
            (PAN_CURVE[index], PAN_CURVE[index + 1])
        }
    }
    pub fn note_parameters(command: u8) -> NoteParameters {
        NoteParameters {
            gate: NOTE_GATE[usize::from((command >> 4) & 7)],
            volume: NOTE_VOLUME[usize::from(command & 15)],
        }
    }
    pub fn song_entry(&self, zero_based_index: u8) -> u16 {
        word(
            self.ram,
            0xd000u16.wrapping_add(u16::from(zero_based_index) * 2),
        )
    }
    pub fn port1_entry(&self, sound: u8) -> SfxPosition {
        SfxPosition::at(word(self.ram, 0x17beu16 + u16::from(sound) * 2))
    }
    pub fn paired_sound(&self, sound: u8) -> u8 {
        self.ram[0x17ff + usize::from(sound)]
    }
    pub fn secondary_entry(&self, port: usize, sound: u8) -> SfxPosition {
        let base = if port == 2 { 0x181e } else { 0x191a };
        SfxPosition::at(word(self.ram, (base + usize::from(sound) * 2) as u16))
    }
    pub fn echo_flag(&self, port: usize, command: u8) -> u8 {
        let base = if port == 2 { 0x18dc } else { 0x19d8 };
        self.ram[base + usize::from(command & 0x3f)]
    }
    pub fn chained_sound(&self, port: usize, sound: u8) -> u8 {
        let base = if port == 2 { 0x189d } else { 0x1999 };
        self.ram[base + usize::from(sound)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_channel_table_address_keeps_wrapping_live_word_reads() {
        let ram =
            std::array::from_fn(|at| (at as u8).wrapping_mul(37).wrapping_add((at >> 8) as u8));
        for address in 0..=u16::MAX {
            let table = ChannelPatternTable::at(address);
            assert_eq!(table.address(), address);
            let mut expected_address = address;
            for channel in 0..8 {
                let at = expected_address;
                let expected = u16::from_le_bytes([
                    ram[usize::from(at)],
                    ram[usize::from(at.wrapping_add(1))],
                ]);
                assert_eq!(table.channel(&ram, channel).address(), expected);
                expected_address = expected_address.wrapping_add(2);
            }
        }
        let mut ram = [0; 65536];
        let table = ChannelPatternTable::at(0xffff);
        ram[0xffff] = 0x34;
        ram[0] = 0x12;
        assert_eq!(table.channel(&ram, 0).address(), 0x1234);
        ram[0] = 0xab;
        assert_eq!(table.channel(&ram, 0).address(), 0xab34);
    }

    #[test]
    fn instrument_and_preset_schemas_share_live_overlapping_bytes() {
        let mut ram = std::array::from_fn(|at| (at as u8).wrapping_mul(19));
        for id in 0..=255u8 {
            let row = 0x3d00 + usize::from(id) * 6;
            let record = SpcData::new(&ram).instrument(id);
            assert_eq!(
                [record.source, record.adsr1, record.adsr2, record.gain],
                ram[row..row + 4]
            );
            assert_eq!(
                record.pitch_base,
                u16::from_be_bytes([ram[row + 4], ram[row + 5]])
            );
            let row = 0x3e00 + usize::from(id) * 9;
            let record = SpcData::new(&ram).dsp_preset(id);
            assert_eq!(record.voice_registers, ram[row..row + 8]);
            assert_eq!(record.pitch_base, u16::from(ram[row + 8]) << 8);
        }
        ram[0x3e06] = 0x12;
        ram[0x3e07] = 0x34;
        assert_eq!(SpcData::new(&ram).instrument(43).pitch_base, 0x1234);
        assert_eq!(
            &SpcData::new(&ram).dsp_preset(0).voice_registers[6..],
            &[0x12, 0x34]
        );
        ram[0x3e06] = 0xab;
        assert_eq!(SpcData::new(&ram).instrument(43).pitch_base, 0xab34);
        assert_eq!(SpcData::new(&ram).dsp_preset(0).voice_registers[6], 0xab);
    }

    #[test]
    fn pan_and_note_decoders_keep_static_boundary_and_raw_bit_masks() {
        const OLD_CURVE: [u8; 22] = [
            0, 1, 3, 7, 13, 21, 30, 41, 52, 66, 81, 94, 103, 110, 115, 119, 122, 124, 125, 126,
            127, 127,
        ];
        const OLD_GATE: [u8; 8] = [50, 101, 127, 152, 178, 203, 229, 252];
        const OLD_VOLUME: [u8; 16] = [
            25, 50, 76, 101, 114, 127, 140, 152, 165, 178, 191, 203, 216, 229, 242, 252,
        ];
        let mut ram = [0xa5; 65536];
        for index in 0..256 {
            ram[0x1178 + index] = (index as u8).wrapping_mul(17);
        }
        for index in 0..256 {
            let expected = if index < 21 {
                (OLD_CURVE[index], OLD_CURVE[index + 1])
            } else {
                (ram[0x1178 + index], ram[0x1179 + index])
            };
            assert_eq!(SpcData::new(&ram).pan_segment(index), expected);
        }
        for command in 0..=255u8 {
            let parameters = SpcData::note_parameters(command);
            assert_eq!(parameters.gate, OLD_GATE[usize::from((command >> 4) & 7)]);
            assert_eq!(parameters.volume, OLD_VOLUME[usize::from(command & 15)]);
        }
    }
}
