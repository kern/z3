//! Parser and encoder for the C port's version-1 `.sav` container.
//!
//! The format contains an input log, an optional replay base snapshot, and one
//! current snapshot. Snapshot blocks which historically contain C struct bytes
//! or placeholders remain opaque, while stable byte arrays and the known PPU
//! layer tuple are exposed explicitly.

use std::error::Error;
use std::fmt;

pub const CONTAINER_HEADER_LEN: usize = 8 * 4;
pub const CONTAINER_VERSION: u32 = 1;
pub const SNAPSHOT_LEN: usize = 275_465;

pub const SPC_RAM_LEN: usize = 0x1_0000;
pub const DSP_BLOCK_LEN: usize = 3_024;
pub const DMA_BLOCK_LEN: usize = 192;
pub const PPU_BLOCK_LEN: usize = 67_305;
pub const VRAM_LEN: usize = 0x1_0000;
pub const CGRAM_LEN: usize = 0x200;
pub const SRAM_LEN: usize = 0x2000;
pub const WRAM_LEN: usize = 0x2_0000;

const PREFIX_OPAQUE: std::ops::Range<usize> = 0..27;
const SPC_RAM: std::ops::Range<usize> = 27..65_563;
const PRE_DSP_OPAQUE: std::ops::Range<usize> = 65_563..65_603;
const DSP: std::ops::Range<usize> = 65_603..68_627;
const PRE_DMA_OPAQUE: std::ops::Range<usize> = 68_627..68_642;
const DMA: std::ops::Range<usize> = 68_642..68_834;
const PPU: std::ops::Range<usize> = 68_834..136_139;
const SRAM: std::ops::Range<usize> = 136_139..144_331;
const PRE_WRAM_OPAQUE: std::ops::Range<usize> = 144_331..144_389;
const WRAM: std::ops::Range<usize> = 144_389..275_461;
const SUFFIX_OPAQUE: std::ops::Range<usize> = 275_461..275_465;

const PPU_VRAM: std::ops::Range<usize> = 0..65_536;
const PPU_POST_VRAM_OPAQUE: std::ops::Range<usize> = 65_536..65_546;
const PPU_CGRAM: std::ops::Range<usize> = 65_546..66_058;
const PPU_POST_CGRAM_OPAQUE: std::ops::Range<usize> = 66_058..66_614;
const PPU_PRE_LAYERS_OPAQUE: std::ops::Range<usize> = 66_614..67_134;
const PPU_LAYERS_START: usize = 67_134;
const PPU_LAYER_LEN: usize = 12;
const PPU_LAYER_COUNT: usize = 4;
const PPU_SUFFIX_OPAQUE: std::ops::Range<usize> = 67_182..67_305;

/// Non-redundant fields in the eight-word save header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Header {
    pub total_frames: u32,
    pub last_inputs: u32,
    pub frames_since_last: u32,
    /// Byte position in the replay log, encoded in header word five above bit 0.
    pub replay_position: u32,
    pub replay_frame_counter: u32,
}

/// A validated view of a complete save container.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Container<'a> {
    pub header: Header,
    pub log: &'a [u8],
    pub base_snapshot: Option<Snapshot<'a>>,
    pub current_snapshot: Snapshot<'a>,
}

impl Container<'_> {
    /// Re-encodes this container without changing opaque compatibility bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        encode(
            self.header,
            self.log,
            self.base_snapshot.as_ref().map(Snapshot::as_bytes),
            self.current_snapshot.as_bytes(),
        )
        .expect("a previously validated container must remain encodable")
    }
}

/// A validated view of the fixed-size `InternalSaveLoad` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot<'a> {
    bytes: &'a [u8],
}

impl<'a> Snapshot<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, FormatError> {
        if bytes.len() != SNAPSHOT_LEN {
            return Err(FormatError::InvalidSnapshotLength(bytes.len()));
        }
        let snapshot = Self { bytes };
        // `_Bool` occupies one byte in the pinned C snapshot ABI. Reject other
        // representations instead of silently giving them Rust truthiness.
        for layer in 0..PPU_LAYER_COUNT {
            let view = snapshot.ppu().layer(layer).expect("fixed layer index");
            validate_bool(layer, "tilemap_wider", view.state_bytes()[0])?;
            validate_bool(layer, "tilemap_higher", view.state_bytes()[1])?;
        }
        Ok(snapshot)
    }

    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub fn prefix_opaque(&self) -> &'a [u8] {
        &self.bytes[PREFIX_OPAQUE]
    }

    #[must_use]
    pub fn spc_ram(&self) -> &'a [u8] {
        &self.bytes[SPC_RAM]
    }

    #[must_use]
    pub fn pre_dsp_opaque(&self) -> &'a [u8] {
        &self.bytes[PRE_DSP_OPAQUE]
    }

    #[must_use]
    pub fn dsp_block(&self) -> &'a [u8] {
        &self.bytes[DSP]
    }

    #[must_use]
    pub fn pre_dma_opaque(&self) -> &'a [u8] {
        &self.bytes[PRE_DMA_OPAQUE]
    }

    #[must_use]
    pub fn dma_block(&self) -> &'a [u8] {
        &self.bytes[DMA]
    }

    #[must_use]
    pub fn ppu(&self) -> PpuSnapshot<'a> {
        PpuSnapshot {
            bytes: &self.bytes[PPU],
        }
    }

    #[must_use]
    pub fn sram(&self) -> &'a [u8] {
        &self.bytes[SRAM]
    }

    #[must_use]
    pub fn pre_wram_opaque(&self) -> &'a [u8] {
        &self.bytes[PRE_WRAM_OPAQUE]
    }

    #[must_use]
    pub fn wram(&self) -> &'a [u8] {
        &self.bytes[WRAM]
    }

    #[must_use]
    pub fn suffix_opaque(&self) -> &'a [u8] {
        &self.bytes[SUFFIX_OPAQUE]
    }
}

/// The PPU compatibility block nested in a legacy snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PpuSnapshot<'a> {
    bytes: &'a [u8],
}

impl<'a> PpuSnapshot<'a> {
    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub fn vram(&self) -> &'a [u8] {
        &self.bytes[PPU_VRAM]
    }

    #[must_use]
    pub fn post_vram_opaque(&self) -> &'a [u8] {
        &self.bytes[PPU_POST_VRAM_OPAQUE]
    }

    #[must_use]
    pub fn cgram(&self) -> &'a [u8] {
        &self.bytes[PPU_CGRAM]
    }

    #[must_use]
    pub fn post_cgram_opaque(&self) -> &'a [u8] {
        &self.bytes[PPU_POST_CGRAM_OPAQUE]
    }

    #[must_use]
    pub fn pre_layers_opaque(&self) -> &'a [u8] {
        &self.bytes[PPU_PRE_LAYERS_OPAQUE]
    }

    #[must_use]
    pub fn layer(&self, index: usize) -> Option<PpuLayerSnapshot<'a>> {
        if index >= PPU_LAYER_COUNT {
            return None;
        }
        let start = PPU_LAYERS_START + index * PPU_LAYER_LEN;
        Some(PpuLayerSnapshot {
            bytes: &self.bytes[start..start + PPU_LAYER_LEN],
        })
    }

    #[must_use]
    pub fn suffix_opaque(&self) -> &'a [u8] {
        &self.bytes[PPU_SUFFIX_OPAQUE]
    }

    #[must_use]
    pub fn vram_word(&self, index: usize) -> Option<u16> {
        read_word(self.vram(), index)
    }

    #[must_use]
    pub fn cgram_word(&self, index: usize) -> Option<u16> {
        read_word(self.cgram(), index)
    }
}

/// One 12-byte layer record in the legacy PPU compatibility block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PpuLayerSnapshot<'a> {
    bytes: &'a [u8],
}

impl<'a> PpuLayerSnapshot<'a> {
    #[must_use]
    pub fn prefix_opaque(&self) -> &'a [u8] {
        &self.bytes[..4]
    }

    #[must_use]
    pub fn state_bytes(&self) -> &'a [u8] {
        &self.bytes[4..8]
    }

    #[must_use]
    pub fn suffix_opaque(&self) -> &'a [u8] {
        &self.bytes[8..12]
    }

    #[must_use]
    pub fn tilemap_wider(&self) -> bool {
        self.bytes[4] != 0
    }

    #[must_use]
    pub fn tilemap_higher(&self) -> bool {
        self.bytes[5] != 0
    }

    #[must_use]
    pub fn tilemap_address(&self) -> u16 {
        u16::from_le_bytes([self.bytes[6], self.bytes[7]])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatError {
    Truncated {
        offset: usize,
        needed: usize,
    },
    UnsupportedVersion(u32),
    InvalidSnapshotLength(usize),
    SnapshotSizeField(u32),
    InvalidPpuBool {
        layer: usize,
        field: &'static str,
        value: u8,
    },
    LastInputsOutOfRange(u32),
    LogTooLarge(usize),
    ReplayPositionTooLarge(u32),
    ReplayPositionOutOfBounds {
        position: u32,
        log_len: usize,
    },
    ReplayFrameOutOfBounds {
        frame: u32,
        total_frames: u32,
    },
    SizeOverflow,
    TrailingBytes(usize),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { offset, needed } => {
                write!(f, "truncated save at {offset}; need {needed} bytes")
            }
            Self::UnsupportedVersion(version) => write!(f, "unsupported save version {version}"),
            Self::InvalidSnapshotLength(len) => {
                write!(f, "snapshot has {len} bytes; expected {SNAPSHOT_LEN}")
            }
            Self::SnapshotSizeField(len) => write!(
                f,
                "save header declares snapshot length {len}; expected {SNAPSHOT_LEN}"
            ),
            Self::InvalidPpuBool {
                layer,
                field,
                value,
            } => write!(
                f,
                "PPU layer {layer} field {field} has invalid boolean byte {value}"
            ),
            Self::LastInputsOutOfRange(inputs) => {
                write!(f, "last input state does not fit 16 bits: {inputs}")
            }
            Self::LogTooLarge(len) => write!(f, "replay log is too large: {len} bytes"),
            Self::ReplayPositionTooLarge(position) => {
                write!(f, "replay position cannot be encoded: {position}")
            }
            Self::ReplayPositionOutOfBounds { position, log_len } => {
                write!(f, "replay position {position} exceeds log length {log_len}")
            }
            Self::ReplayFrameOutOfBounds {
                frame,
                total_frames,
            } => write!(
                f,
                "replay frame {frame} exceeds total frame count {total_frames}"
            ),
            Self::SizeOverflow => f.write_str("save container size overflow"),
            Self::TrailingBytes(len) => write!(f, "save container has {len} trailing bytes"),
        }
    }
}

impl Error for FormatError {}

/// Parses and validates a complete version-1 save container.
pub fn parse(source: &[u8]) -> Result<Container<'_>, FormatError> {
    let mut reader = Reader::new(source);
    let version = reader.u32()?;
    if version != CONTAINER_VERSION {
        return Err(FormatError::UnsupportedVersion(version));
    }
    let total_frames = reader.u32()?;
    let log_len_u32 = reader.u32()?;
    let last_inputs = reader.u32()?;
    let frames_since_last = reader.u32()?;
    let base_and_position = reader.u32()?;
    let snapshot_len = reader.u32()?;
    let replay_frame_counter = reader.u32()?;
    if snapshot_len != SNAPSHOT_LEN as u32 {
        return Err(FormatError::SnapshotSizeField(snapshot_len));
    }
    if last_inputs > u32::from(u16::MAX) {
        return Err(FormatError::LastInputsOutOfRange(last_inputs));
    }
    let replay_position = base_and_position >> 1;
    if replay_position > log_len_u32 {
        return Err(FormatError::ReplayPositionOutOfBounds {
            position: replay_position,
            log_len: log_len_u32 as usize,
        });
    }
    if replay_frame_counter > total_frames {
        return Err(FormatError::ReplayFrameOutOfBounds {
            frame: replay_frame_counter,
            total_frames,
        });
    }

    let log = reader.bytes(log_len_u32 as usize)?;
    let base_snapshot = if base_and_position & 1 != 0 {
        Some(Snapshot::parse(reader.bytes(SNAPSHOT_LEN)?)?)
    } else {
        None
    };
    let current_snapshot = Snapshot::parse(reader.bytes(SNAPSHOT_LEN)?)?;
    if reader.remaining() != 0 {
        return Err(FormatError::TrailingBytes(reader.remaining()));
    }

    Ok(Container {
        header: Header {
            total_frames,
            last_inputs,
            frames_since_last,
            replay_position,
            replay_frame_counter,
        },
        log,
        base_snapshot,
        current_snapshot,
    })
}

/// Encodes a save container, preserving all supplied snapshot bytes.
pub fn encode(
    header: Header,
    log: &[u8],
    base_snapshot: Option<&[u8]>,
    current_snapshot: &[u8],
) -> Result<Vec<u8>, FormatError> {
    let log_len = u32::try_from(log.len()).map_err(|_| FormatError::LogTooLarge(log.len()))?;
    if header.last_inputs > u32::from(u16::MAX) {
        return Err(FormatError::LastInputsOutOfRange(header.last_inputs));
    }
    if header.replay_position > u32::MAX >> 1 {
        return Err(FormatError::ReplayPositionTooLarge(header.replay_position));
    }
    if header.replay_position > log_len {
        return Err(FormatError::ReplayPositionOutOfBounds {
            position: header.replay_position,
            log_len: log.len(),
        });
    }
    if header.replay_frame_counter > header.total_frames {
        return Err(FormatError::ReplayFrameOutOfBounds {
            frame: header.replay_frame_counter,
            total_frames: header.total_frames,
        });
    }
    let current_snapshot = Snapshot::parse(current_snapshot)?;
    let base_snapshot = base_snapshot.map(Snapshot::parse).transpose()?;
    let base_and_position = (header.replay_position << 1) | u32::from(base_snapshot.is_some());
    let total_len = CONTAINER_HEADER_LEN
        .checked_add(log.len())
        .and_then(|len| len.checked_add(SNAPSHOT_LEN))
        .and_then(|len| {
            len.checked_add(if base_snapshot.is_some() {
                SNAPSHOT_LEN
            } else {
                0
            })
        })
        .ok_or(FormatError::SizeOverflow)?;

    let mut output = Vec::with_capacity(total_len);
    push_u32(&mut output, CONTAINER_VERSION);
    push_u32(&mut output, header.total_frames);
    push_u32(&mut output, log_len);
    push_u32(&mut output, header.last_inputs);
    push_u32(&mut output, header.frames_since_last);
    push_u32(&mut output, base_and_position);
    push_u32(&mut output, SNAPSHOT_LEN as u32);
    push_u32(&mut output, header.replay_frame_counter);
    output.extend_from_slice(log);
    if let Some(snapshot) = base_snapshot {
        output.extend_from_slice(snapshot.as_bytes());
    }
    output.extend_from_slice(current_snapshot.as_bytes());
    debug_assert_eq!(output.len(), total_len);
    Ok(output)
}

fn validate_bool(layer: usize, field: &'static str, value: u8) -> Result<(), FormatError> {
    if value <= 1 {
        Ok(())
    } else {
        Err(FormatError::InvalidPpuBool {
            layer,
            field,
            value,
        })
    }
}

fn read_word(bytes: &[u8], index: usize) -> Option<u16> {
    let offset = index.checked_mul(2)?;
    let pair = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([pair[0], pair[1]]))
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

struct Reader<'a> {
    source: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    fn remaining(&self) -> usize {
        self.source.len() - self.position
    }

    fn bytes(&mut self, len: usize) -> Result<&'a [u8], FormatError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(FormatError::SizeOverflow)?;
        let bytes = self
            .source
            .get(self.position..end)
            .ok_or(FormatError::Truncated {
                offset: self.position,
                needed: len,
            })?;
        self.position = end;
        Ok(bytes)
    }

    fn u32(&mut self) -> Result<u32, FormatError> {
        let bytes: [u8; 4] = self.bytes(4)?.try_into().expect("slice length was checked");
        Ok(u32::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_snapshot(seed: u8) -> Vec<u8> {
        let mut bytes = vec![seed; SNAPSHOT_LEN];
        for layer in 0..PPU_LAYER_COUNT {
            let state = PPU.start + PPU_LAYERS_START + layer * PPU_LAYER_LEN + 4;
            bytes[state] = u8::from(layer & 1 != 0);
            bytes[state + 1] = u8::from(layer & 2 != 0);
            bytes[state + 2..state + 4].copy_from_slice(&(0x1200_u16 + layer as u16).to_le_bytes());
        }
        bytes
    }

    #[test]
    fn round_trips_container_and_preserves_opaque_bytes() {
        let current = synthetic_snapshot(0xa5);
        let base = synthetic_snapshot(0x5a);
        let header = Header {
            total_frames: 99,
            last_inputs: 0x812,
            frames_since_last: 7,
            replay_position: 3,
            replay_frame_counter: 42,
        };
        let encoded = encode(header, &[1, 2, 3, 4], Some(&base), &current).unwrap();
        assert_eq!(&encoded[..4], &CONTAINER_VERSION.to_le_bytes());
        assert_eq!(&encoded[8..12], &4_u32.to_le_bytes());

        let decoded = parse(&encoded).unwrap();
        assert_eq!(decoded.header, header);
        assert_eq!(decoded.log, &[1, 2, 3, 4]);
        assert_eq!(decoded.base_snapshot.unwrap().as_bytes(), base);
        assert_eq!(decoded.current_snapshot.as_bytes(), current);
        assert_eq!(decoded.to_bytes(), encoded);
    }

    #[test]
    fn exposes_named_snapshot_regions_and_little_endian_ppu_fields() {
        let bytes = synthetic_snapshot(0);
        let snapshot = Snapshot::parse(&bytes).unwrap();
        assert_eq!(snapshot.spc_ram().len(), SPC_RAM_LEN);
        assert_eq!(snapshot.dsp_block().len(), DSP_BLOCK_LEN);
        assert_eq!(snapshot.dma_block().len(), DMA_BLOCK_LEN);
        assert_eq!(snapshot.ppu().as_bytes().len(), PPU_BLOCK_LEN);
        assert_eq!(snapshot.ppu().vram().len(), VRAM_LEN);
        assert_eq!(snapshot.ppu().cgram().len(), CGRAM_LEN);
        assert_eq!(snapshot.sram().len(), SRAM_LEN);
        assert_eq!(snapshot.wram().len(), WRAM_LEN);
        assert_eq!(snapshot.ppu().layer(3).unwrap().tilemap_address(), 0x1203);
        assert!(snapshot.ppu().layer(3).unwrap().tilemap_wider());
        assert!(snapshot.ppu().layer(3).unwrap().tilemap_higher());
        assert!(snapshot.ppu().layer(4).is_none());
    }

    #[test]
    fn rejects_bad_sizes_truncation_and_trailing_data() {
        assert_eq!(
            Snapshot::parse(&[0; 7]),
            Err(FormatError::InvalidSnapshotLength(7))
        );
        let current = synthetic_snapshot(0);
        let encoded = encode(Header::default(), &[], None, &current).unwrap();
        assert!(matches!(
            parse(&encoded[..encoded.len() - 1]),
            Err(FormatError::Truncated { .. })
        ));
        let mut trailing = encoded;
        trailing.push(0);
        assert_eq!(parse(&trailing), Err(FormatError::TrailingBytes(1)));
    }

    #[test]
    fn rejects_invalid_header_and_ppu_bool() {
        let current = synthetic_snapshot(0);
        let mut encoded = encode(Header::default(), &[], None, &current).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(parse(&encoded), Err(FormatError::UnsupportedVersion(2)));

        let mut invalid = current;
        let bool_offset = PPU.start + PPU_LAYERS_START + 4;
        invalid[bool_offset] = 2;
        assert!(matches!(
            Snapshot::parse(&invalid),
            Err(FormatError::InvalidPpuBool {
                layer: 0,
                field: "tilemap_wider",
                value: 2
            })
        ));
    }
}
