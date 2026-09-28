//! Top-level deterministic runtime control translated from `zelda_rtl.c`.

use crate::oam::{OAM_BUFFER, OAM_ENTRY_COUNT};
use crate::ram::Ram;
use z3_core::{CanonicalState, SRAM_SIZE, WRAM_SIZE};
use z3_snes::{
    dma::{Dma, DmaChannel},
    ppu::{Ppu, RENDER_HEIGHT_240, RENDER_MODE7_4X},
    saveload::SaveLoadError,
};

pub const LEGACY_SNAPSHOT_LEN: usize = 275_465;
const SNAPSHOT_SPC_RAM: std::ops::Range<usize> = 27..65_563;
const SNAPSHOT_DSP: std::ops::Range<usize> = 65_603..68_627;
const SNAPSHOT_DMA: std::ops::Range<usize> = 68_642..68_834;
const SNAPSHOT_PPU: std::ops::Range<usize> = 68_834..136_139;
const SNAPSHOT_SRAM: std::ops::Range<usize> = 136_139..144_331;
const SNAPSHOT_WRAM: std::ops::Range<usize> = 144_389..275_461;
const HDMA_TABLE_OLD: usize = 0x1b00;
const HDMA_TABLE_NEW: usize = 0x1dba0;
const HDMA_TABLE_LEN: usize = 224 * 2;

const INIDISP_COPY: usize = 0x13;
const FLAG_UPDATE_CGRAM: usize = 0x15;
const FRAME_COUNTER: usize = 0x1a;
const CRYSTAL_ROTATE_COUNTER: usize = 0x649;
const BUGS_FIXED: usize = 0x64a;
const APUI00_MIRROR: usize = 0x648;
const ENHANCED_FEATURES0: usize = 0x64c;
const ANIMATED_TILE_DATA_SRC: usize = 0xadc;
const NMI_BOOLEAN: usize = 0x12;
const DMA_SOURCE_ADDR_9: usize = 0xae0;
const DMA_SOURCE_ADDR_14: usize = 0xae2;
const NMI_THREAD_ACTIVE: usize = 0x12a;
const THREAD_OTHER_STACK: usize = 0x1f0a;
const MAIN_PALETTE_BUFFER: usize = 0xc500;

pub const BUG_FIX_POLY_RENDERER: u8 = 1;

pub fn hdma_setup(
    dma: &mut Dma,
    address6: u32,
    address7: u32,
    transfer_unit: u8,
    register6: u8,
    register7: u8,
    indirect_bank: u8,
) {
    fn configure(
        dma: &mut Dma,
        base: u16,
        address: u32,
        transfer_unit: u8,
        register: u8,
        indirect_bank: u8,
    ) {
        dma.write_register(base, transfer_unit);
        dma.write_register(base + 1, register);
        dma.write_register(base + 2, address as u8);
        dma.write_register(base + 3, (address >> 8) as u8);
        dma.write_register(base + 4, (address >> 16) as u8);
        dma.write_register(base + 7, indirect_bank);
    }
    if address6 != 0 {
        configure(dma, 0x60, address6, transfer_unit, register6, indirect_bank);
    }
    configure(dma, 0x70, address7, transfer_unit, register7, indirect_bank);
}

/// Owned native machine state corresponding to `ZeldaEnv` plus `g_ram`.
/// VRAM has one source of truth in the PPU; canonical exports copy it in the
/// same little-endian order as the C oracle process.
#[derive(Debug)]
pub struct NativeRuntime {
    pub wram: Box<[u8; WRAM_SIZE]>,
    pub sram: Box<[u8; SRAM_SIZE]>,
    pub dma: Dma,
    pub ppu: Ppu,
    pub audio: crate::audio::AudioRuntime,
    pub recorder: StateRecorder,
    pub wanted_features: u32,
    pub debug_frame_counter: u32,
    legacy_snapshot_template: Vec<u8>,
}

impl Default for NativeRuntime {
    fn default() -> Self {
        Self {
            wram: Box::new([0; WRAM_SIZE]),
            sram: Box::new([0; SRAM_SIZE]),
            dma: Dma::new(),
            ppu: Ppu::new(),
            audio: crate::audio::AudioRuntime::default(),
            recorder: StateRecorder::default(),
            wanted_features: 0,
            debug_frame_counter: 0,
            legacy_snapshot_template: vec![0; LEGACY_SNAPSHOT_LEN],
        }
    }
}

impl NativeRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self, preserve_sram: bool) {
        let mut ram = Ram::new(&mut self.wram);
        reset_runtime(
            &mut ram,
            &mut self.sram,
            &mut self.dma,
            &mut self.ppu,
            &mut self.audio,
            preserve_sram,
        );
    }

    pub fn initialize(
        &mut self,
        dummy_snes_write: impl FnMut(u16, u8),
        load_intro_song_bank: impl FnMut(),
    ) {
        initialization_code(
            &mut Ram::new(&mut self.wram),
            &mut self.sram,
            dummy_snes_write,
            load_intro_song_bank,
        );
    }

    pub fn step(&mut self, hooks: &mut impl FrameHooks, input: u16, run_what: u8) {
        run_frame_internal(&mut Ram::new(&mut self.wram), hooks, input, run_what);
    }

    /// Host-facing frame entry corresponding to `ZeldaRunFrame`: normalizes
    /// impossible input pairs, records/replays input and deterministic patches,
    /// selects the game/poly schedule, advances the core, then queues APU state.
    pub fn run_frame(
        &mut self,
        hooks: &mut impl FrameHooks,
        input: u16,
    ) -> Result<bool, ReplayError> {
        let input = normalize_inputs(input);
        self.debug_frame_counter = self.debug_frame_counter.wrapping_add(1);
        let replaying = self.recorder.is_replaying();
        let mut ram = Ram::new(&mut self.wram);
        let input = if replaying {
            self.recorder.read_next_replay_state(&mut ram)?
        } else {
            self.recorder.record(input);
            let music_playing = u8::from(self.audio.is_music_playing());
            if music_playing != ram.read_u8(APUI00_MIRROR) {
                ram.write_u8(APUI00_MIRROR, music_playing);
                self.recorder
                    .record_patch(APUI00_MIRROR as u32, &[music_playing]);
            }
            if ram.read_u16(ANIMATED_TILE_DATA_SRC) != 0 {
                if ram.read_u8(BUGS_FIXED) < 1 {
                    ram.write_u8(BUGS_FIXED, 1);
                    self.recorder.record_patch(BUGS_FIXED as u32, &[1]);
                }
                if ram.read_u32(ENHANCED_FEATURES0) != self.wanted_features {
                    ram.write_u32(ENHANCED_FEATURES0, self.wanted_features);
                    self.recorder.record_patch(
                        ENHANCED_FEATURES0 as u32,
                        &self.wanted_features.to_le_bytes(),
                    );
                }
            }
            input
        };
        let vertical_irq_increment = ram.read_u8(0xff);
        let run_what = determine_run_what(&mut ram, vertical_irq_increment);
        run_frame_internal(&mut ram, hooks, input, run_what);
        self.audio.push_apu_state();
        Ok(replaying)
    }

    pub fn run_core_frame<'assets, Route, LoadTrack>(
        &mut self,
        input: u16,
        nmi_assets: crate::nmi::NmiAssets<'assets>,
        intro_song_bank: &'assets [u8],
        route_main_module: Route,
        load_pcm_track: LoadTrack,
    ) -> Result<bool, ReplayError>
    where
        Route: FnMut(&mut Ram<'_>),
        LoadTrack: FnMut(u8) -> Option<crate::audio::PcmTrack>,
    {
        let input = normalize_inputs(input);
        self.debug_frame_counter = self.debug_frame_counter.wrapping_add(1);
        let replaying = self.recorder.is_replaying();
        let mut ram = Ram::new(&mut self.wram);
        let input = if replaying {
            self.recorder.read_next_replay_state(&mut ram)?
        } else {
            self.recorder.record(input);
            let music_playing = u8::from(self.audio.is_music_playing());
            if music_playing != ram.read_u8(APUI00_MIRROR) {
                ram.write_u8(APUI00_MIRROR, music_playing);
                self.recorder
                    .record_patch(APUI00_MIRROR as u32, &[music_playing]);
            }
            if ram.read_u16(ANIMATED_TILE_DATA_SRC) != 0 {
                if ram.read_u8(BUGS_FIXED) < 1 {
                    ram.write_u8(BUGS_FIXED, 1);
                    self.recorder.record_patch(BUGS_FIXED as u32, &[1]);
                }
                if ram.read_u32(ENHANCED_FEATURES0) != self.wanted_features {
                    ram.write_u32(ENHANCED_FEATURES0, self.wanted_features);
                    self.recorder.record_patch(
                        ENHANCED_FEATURES0 as u32,
                        &self.wanted_features.to_le_bytes(),
                    );
                }
            }
            input
        };
        let vertical_irq_increment = ram.read_u8(0xff);
        let run_what = determine_run_what(&mut ram, vertical_irq_increment);
        let mut hooks = CoreFrameHooks {
            sram: &mut self.sram,
            ppu: &mut self.ppu,
            audio: &mut self.audio,
            nmi_assets,
            intro_song_bank,
            route_main_module,
            load_pcm_track,
        };
        run_frame_internal(&mut ram, &mut hooks, input, run_what);
        drop(hooks);
        self.audio.push_apu_state();
        Ok(replaying)
    }

    pub fn draw(&mut self, pitch: usize, render_flags: u8, dummy_nmitimen_write: impl FnMut(u8)) {
        draw_ppu_frame(
            &mut Ram::new(&mut self.wram),
            &mut self.dma,
            &mut self.ppu,
            pitch,
            render_flags,
            dummy_nmitimen_write,
        );
    }

    #[must_use]
    pub fn canonical_state(&self) -> CanonicalState {
        CanonicalState {
            wram: self.wram.clone(),
            sram: self.sram.clone(),
            vram: Box::new(self.ppu.vram),
        }
    }

    pub fn import_canonical_state(&mut self, state: &CanonicalState) {
        self.wram.copy_from_slice(state.wram.as_slice());
        self.sram.copy_from_slice(state.sram.as_slice());
        self.ppu.vram.copy_from_slice(state.vram.as_slice());
    }

    /// Implements the exact transfer order of C `InternalSaveLoad`. On load,
    /// opaque historical compatibility regions are retained as the template
    /// for the next save; on save, live fields are overlaid on that template.
    pub fn internal_save_load(
        &mut self,
        snapshot: &mut [u8],
        loading: bool,
    ) -> Result<(), LegacySnapshotError> {
        if snapshot.len() != LEGACY_SNAPSHOT_LEN {
            return Err(LegacySnapshotError::WrongLength(snapshot.len()));
        }
        if loading {
            self.audio
                .player
                .ram
                .copy_from_slice(&snapshot[SNAPSHOT_SPC_RAM]);
            self.audio
                .player
                .dsp
                .read_save_block(&snapshot[SNAPSHOT_DSP])?;
            self.dma.read_save_block(&snapshot[SNAPSHOT_DMA])?;
            self.ppu.read_save_block(&snapshot[SNAPSHOT_PPU])?;
            self.sram.copy_from_slice(&snapshot[SNAPSHOT_SRAM]);
            self.wram.copy_from_slice(&snapshot[SNAPSHOT_WRAM]);
            self.legacy_snapshot_template.copy_from_slice(snapshot);
        } else {
            snapshot.copy_from_slice(&self.legacy_snapshot_template);
            snapshot[SNAPSHOT_SPC_RAM].copy_from_slice(self.audio.player.ram.as_slice());
            self.audio
                .player
                .dsp
                .write_save_block(&mut snapshot[SNAPSHOT_DSP])?;
            self.dma.write_save_block(&mut snapshot[SNAPSHOT_DMA])?;
            self.ppu.write_save_block(&mut snapshot[SNAPSHOT_PPU])?;
            snapshot[SNAPSHOT_SRAM].copy_from_slice(self.sram.as_slice());
            snapshot[SNAPSHOT_WRAM].copy_from_slice(self.wram.as_slice());
        }
        Ok(())
    }

    /// C `LoadSnesState`: restore every serialized device, repair the moved
    /// HDMA table, then reconstruct translated SPC state from its RAM image.
    pub fn load_snes_state(&mut self, snapshot: &[u8]) -> Result<(), LegacySnapshotError> {
        let mut owned = snapshot.to_vec();
        self.internal_save_load(&mut owned, true)?;
        self.wram.copy_within(
            HDMA_TABLE_OLD..HDMA_TABLE_OLD + HDMA_TABLE_LEN,
            HDMA_TABLE_NEW,
        );
        self.audio.restore_music_after_load(false);
        Ok(())
    }

    /// C `SaveSnesState`: place the relocated HDMA table and translated SPC
    /// variables in their compatibility locations before serializing.
    pub fn save_snes_state(&mut self) -> Result<Vec<u8>, LegacySnapshotError> {
        self.wram.copy_within(
            HDMA_TABLE_NEW..HDMA_TABLE_NEW + HDMA_TABLE_LEN,
            HDMA_TABLE_OLD,
        );
        self.audio
            .zelda_save_music_state_to_ram_locked(self.wram.as_mut_slice());
        let mut snapshot = vec![0; LEGACY_SNAPSHOT_LEN];
        self.internal_save_load(&mut snapshot, false)?;
        Ok(snapshot)
    }
}

#[derive(Debug)]
pub enum LegacySnapshotError {
    WrongLength(usize),
    Device(SaveLoadError),
}

impl std::fmt::Display for LegacySnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongLength(actual) => write!(
                f,
                "legacy snapshot has {actual} bytes; expected {LEGACY_SNAPSHOT_LEN}"
            ),
            Self::Device(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for LegacySnapshotError {}

impl From<SaveLoadError> for LegacySnapshotError {
    fn from(value: SaveLoadError) -> Self {
        Self::Device(value)
    }
}

/// Exact one-time initialization sequence used by `ZeldaRunFrameInternal`.
/// The callbacks preserve the observable ordering of the three dummy SNES
/// register writes and the intro sound-bank upload without coupling this core
/// module to a host frontend.
pub fn initialization_code(
    ram: &mut Ram<'_>,
    sram: &mut [u8; 0x2000],
    mut dummy_snes_write: impl FnMut(u16, u8),
    mut load_intro_song_bank: impl FnMut(),
) {
    dummy_snes_write(0x4200, 0);
    dummy_snes_write(0x420c, 0);
    dummy_snes_write(0x420b, 0);
    load_intro_song_bank();
    startup_initialize_memory(ram, sram);
    ram.write_u16(ANIMATED_TILE_DATA_SRC, 0xa680);
    ram.write_u16(DMA_SOURCE_ADDR_9, 0xb280);
    ram.write_u16(DMA_SOURCE_ADDR_14, 0xb2e0);
    dummy_snes_write(0x4200, 0x81);
}

/// Resets the native machine components in the same order as `ZeldaReset`.
/// PPU reset owns VRAM clearing, while WRAM and optional SRAM clearing happen
/// explicitly here.
pub fn reset_runtime(
    ram: &mut Ram<'_>,
    sram: &mut [u8; 0x2000],
    dma: &mut Dma,
    ppu: &mut Ppu,
    audio: &mut crate::audio::AudioRuntime,
    preserve_sram: bool,
) {
    dma.reset();
    ppu.reset();
    ram.slice_mut(0, z3_core::WRAM_SIZE).fill(0);
    if !preserve_sram {
        sram.fill(0);
    }
    audio.restore_music_after_load(true);
}

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
const ATTRACT_DMA_TABLE0: [u8; 13] = [
    0x20, 0xff, 0, 0x50, 0x18, 0xe0, 0x50, 0x18, 0xe0, 1, 0xff, 0, 0,
];
const ATTRACT_DMA_TABLE1: [u8; 10] = [0x48, 0xff, 0, 0x30, 0x30, 0xd8, 1, 0xff, 0, 0];
const ENDING_HDMA_TABLE: [u8; 19] = [
    0x52, 0, 6, 8, 0xe2, 0, 8, 2, 6, 5, 4, 6, 0x10, 6, 6, 0x81, 0xe2, 0, 0,
];
const SPOTLIGHT_INDIRECT_HDMA: [u8; 7] = [0xf8, 0, 0x1b, 0xf8, 0xf0, 0x1b, 0];
const MAP_MODE_HDMA0: [u8; 7] = [0xf0, 0x27, 0xdd, 0xf0, 7, 0xde, 0];
const MAP_MODE_HDMA1: [u8; 7] = [0xf0, 0xe7, 0xde, 0xf0, 0xc7, 0xdf, 0];
const ATTRACT_INDIRECT_HDMA: [u8; 7] = [0xf0, 0, 0x1b, 0xf0, 0xe0, 0x1b, 0];
const PRAYING_SCENE_HDMA: [u8; 7] = [0xf8, 0, 0x1b, 0xf8, 0xf0, 0x1b, 0];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SimpleHdma {
    table: Option<u32>,
    indirect: Option<u32>,
    repeat_count: u8,
    mode: u8,
    ppu_address: u8,
    indirect_bank: u8,
}

impl SimpleHdma {
    #[must_use]
    pub fn from_channel(channel: &DmaChannel) -> Self {
        if !channel.hdma_active {
            return Self::default();
        }
        Self {
            table: Some(u32::from(channel.a_address) | u32::from(channel.a_bank) << 16),
            indirect: None,
            repeat_count: 0,
            mode: channel.mode | u8::from(channel.indirect) << 6,
            ppu_address: channel.b_address,
            indirect_bank: channel.indirect_bank,
        }
    }

    pub fn do_line(&mut self, ram: &Ram<'_>, ppu: &mut Ppu) {
        let Some(mut table) = self.table else {
            return;
        };
        let mut transfer = false;
        if self.repeat_count & 0x7f == 0 {
            self.repeat_count = hdma_read(ram, table);
            table = table.wrapping_add(1);
            if self.repeat_count == 0 {
                self.table = None;
                return;
            }
            if self.mode & 0x40 != 0 {
                let low = hdma_read(ram, table);
                let high = hdma_read(ram, table.wrapping_add(1));
                self.indirect = Some(resolve_hdma_pointer(
                    u32::from(self.indirect_bank) << 16 | u32::from(low) | u32::from(high) << 8,
                ));
                table = table.wrapping_add(2);
            }
            transfer = true;
        }
        if transfer || self.repeat_count & 0x80 != 0 {
            let mode = usize::from(self.mode & 7);
            for &register_offset in B_ADDRESS_OFFSETS[mode].iter().take(TRANSFER_LENGTH[mode]) {
                let source = if self.mode & 0x40 != 0 {
                    let source = self.indirect.expect("indirect HDMA pointer");
                    self.indirect = Some(source.wrapping_add(1));
                    source
                } else {
                    let source = table;
                    table = table.wrapping_add(1);
                    source
                };
                let register = self.ppu_address.wrapping_add(register_offset);
                ppu.write(register, hdma_read(ram, source));
            }
        }
        self.table = Some(table);
        self.repeat_count = self.repeat_count.wrapping_sub(1);
    }
}

fn static_byte(base: u32, address: u32, bytes: &[u8]) -> Option<u8> {
    let offset = address.checked_sub(base)? as usize;
    bytes.get(offset).copied()
}

fn zoom_byte(address: u32, base: u32, values: &[u16]) -> Option<u8> {
    let offset = address.checked_sub(base)? as usize;
    let value = *values.get(offset / 2)?;
    Some(if offset & 1 == 0 {
        value as u8
    } else {
        (value >> 8) as u8
    })
}

fn resolve_hdma_pointer(address: u32) -> u32 {
    match address {
        0x0a_dd27 => 0x0100_0000,
        0x0a_de07 => 0x0100_00e0,
        0x0a_dee7 => 0x0110_0000,
        0x0a_dfc7 => 0x0110_00e0,
        _ => address,
    }
}

fn hdma_read(ram: &Ram<'_>, address: u32) -> u8 {
    if (0x0100_0000..0x0100_01e0).contains(&address) {
        return zoom_byte(address, 0x0100_0000, &crate::attract::MAP_MODE_ZOOMS1)
            .expect("mode-7 zoom table 1 byte");
    }
    if (0x0110_0000..0x0110_01e0).contains(&address) {
        return zoom_byte(address, 0x0110_0000, &crate::attract::MAP_MODE_ZOOMS2)
            .expect("mode-7 zoom table 2 byte");
    }
    static_byte(0x0c_fa87, address, &ATTRACT_DMA_TABLE0)
        .or_else(|| static_byte(0x0c_fa94, address, &ATTRACT_DMA_TABLE1))
        .or_else(|| static_byte(0x0e_bd53, address, &ENDING_HDMA_TABLE))
        .or_else(|| static_byte(0x00_f2fb, address, &SPOTLIGHT_INDIRECT_HDMA))
        .or_else(|| static_byte(0x0a_bdcf, address, &MAP_MODE_HDMA0))
        .or_else(|| static_byte(0x0a_bdd6, address, &MAP_MODE_HDMA1))
        .or_else(|| static_byte(0x0a_bddd, address, &ATTRACT_INDIRECT_HDMA))
        .or_else(|| static_byte(0x02_c80c, address, &PRAYING_SCENE_HDMA))
        .unwrap_or_else(|| {
            let low = address & 0xffff;
            let translated = if (0x1b00..0x1ce0).contains(&low) {
                0x1dba0 + (low as usize - 0x1b00)
            } else {
                low as usize
            };
            ram.read_u8(translated)
        })
}

fn configure_ppu_side_space(ram: &Ram<'_>, ppu: &mut Ppu) {
    fn unsigned_delta(left: u16, right: u16) -> usize {
        (i32::from(left) - i32::from(right)) as u32 as usize
    }
    let main_module = ram.read_u8(0x10);
    let module = if main_module == 14 {
        ram.read_u8(0x10c)
    } else {
        main_module
    };
    let mut left = 0usize;
    let mut right = 0usize;
    let mut bottom = 0usize;
    if module == 9 {
        if main_module == 14 && ram.read_u8(0x11) == 7 && ram.read_u8(0x200) >= 4 {
            left = usize::from(ppu.extra_left_right);
            right = left;
            bottom = 16;
        } else {
            let horizontal = ram.read_u16(0xe2);
            let vertical = ram.read_u16(0xe8);
            left = unsigned_delta(horizontal, ram.read_u16(0x604));
            right = unsigned_delta(ram.read_u16(0x606), horizontal);
            bottom = unsigned_delta(ram.read_u16(0x602), vertical);
        }
    } else if module == 7 {
        if ram.read_u8(0x458) == 0 || ram.read_u8(0x1d) == 0 {
            let quadrant = usize::from(ram.read_u8(0xa6) >> 1);
            let horizontal = ram.read_u16(0xe2);
            left = usize::from(horizontal.saturating_sub(ram.read_u16(0x608 + quadrant * 2)));
            right = usize::from(
                ram.read_u16(0x608 + (quadrant + 2) * 2)
                    .saturating_sub(horizontal),
            );
        }
        let quadrant = usize::from(ram.read_u8(0xa7) >> 1);
        bottom = usize::from(
            ram.read_u16(0x600 + (quadrant + 2) * 2)
                .saturating_sub(ram.read_u16(0xe8)),
        );
    } else if matches!(module, 20 | 0 | 1) {
        left = usize::from(ppu.extra_left_right);
        right = left;
        bottom = 16;
    }
    ppu.set_extra_side_space(left, right, bottom);
}

pub fn draw_ppu_frame(
    ram: &mut Ram<'_>,
    dma: &mut Dma,
    ppu: &mut Ppu,
    pitch: usize,
    render_flags: u8,
    mut dummy_nmitimen_write: impl FnMut(u8),
) {
    let height = if render_flags & RENDER_HEIGHT_240 != 0 {
        240
    } else {
        224
    };
    ppu.begin_drawing(pitch, height, render_flags);
    dma.start_dma(ram.read_u8(0x9b), true);
    let mut channels = [
        SimpleHdma::from_channel(&dma.channels[6]),
        SimpleHdma::from_channel(&dma.channels[7]),
    ];

    if render_flags & RENDER_MODE7_4X != 0 && ppu.mode == 7 {
        let source = u32::from(dma.channels[6].a_address) | u32::from(dma.channels[6].a_bank) << 16;
        let (low, high) = match source {
            0x0a_bdcf => (
                i32::from(crate::attract::MAP_MODE_ZOOMS1[0]),
                i32::from(crate::attract::MAP_MODE_ZOOMS1[223]),
            ),
            0x0a_bdd6 => (
                i32::from(crate::attract::MAP_MODE_ZOOMS2[0]),
                i32::from(crate::attract::MAP_MODE_ZOOMS2[223]),
            ),
            0x0a_bddd => (
                i32::from(ram.read_u16(0x1dba0)),
                i32::from(ram.read_u16(0x1dba0 + 223 * 2)),
            ),
            _ => (0, 0),
        };
        ppu.set_mode7_perspective_correction(low, high);
    }
    if ppu.extra_left_right != 0 || render_flags & RENDER_HEIGHT_240 != 0 {
        configure_ppu_side_space(ram, ppu);
    }

    for line in 0..=height {
        if line == 128 && ram.read_u8(0x128) != 0 {
            let scroll = ram.read_u16(0x630);
            ppu.write(0x11, scroll as u8);
            ppu.write(0x11, (scroll >> 8) as u8);
            ppu.write(0x12, 0);
            ppu.write(0x12, 0);
            if ram.read_u8(0x128) & 0x80 != 0 {
                ram.write_u8(0x128, 0);
                dummy_nmitimen_write(0x81);
            }
        }
        ppu.run_line(line);
        channels[0].do_line(ram, ppu);
        channels[1].do_line(ram, ppu);
    }
}

/// C clears only the first 8 KiB of WRAM during startup, validates the three
/// save-slot signatures independently, and leaves every other byte intact.
pub fn startup_initialize_memory(ram: &mut Ram<'_>, sram: &mut [u8; 0x2000]) {
    ram.slice_mut(0, 0x2000).fill(0);
    ram.write_u16(MAIN_PALETTE_BUFFER, 0);
    sram[0x1ffe..0x2000].fill(0);
    for offset in [0x3e5, 0x8e5, 0xde5] {
        if u16::from_le_bytes([sram[offset], sram[offset + 1]]) != 0x55aa {
            sram[offset] = 0;
            sram[offset + 1] = 0;
        }
    }
    ram.write_u8(INIDISP_COPY, 0x80);
    ram.write_u8(
        FLAG_UPDATE_CGRAM,
        ram.read_u8(FLAG_UPDATE_CGRAM).wrapping_add(1),
    );
}

pub fn clear_oam_buffer(ram: &mut Ram<'_>) {
    for index in 0..OAM_ENTRY_COUNT {
        ram.write_u8(OAM_BUFFER + index * 4 + 1, 0xf0);
    }
}

#[must_use]
pub const fn normalize_inputs(mut inputs: u16) -> u16 {
    if inputs & 0x30 == 0x30 {
        inputs ^= 0x30;
    }
    if inputs & 0xc0 == 0xc0 {
        inputs ^= 0xc0;
    }
    inputs
}

/// Determines which native threads execute this frame. The return bits match
/// `ZeldaRunFrameInternal`: bit 0 is the game loop, bit 1 the poly loop.
pub fn determine_run_what(ram: &mut Ram<'_>, vertical_irq_increment: u8) -> u8 {
    if ram.read_u8(BUGS_FIXED) < BUG_FIX_POLY_RENDERER {
        if ram.read_u8(NMI_THREAD_ACTIVE) != 0 && ram.read_u16(THREAD_OTHER_STACK) != 0x1f31 {
            2
        } else {
            1
        }
    } else {
        let total =
            u16::from(ram.read_u8(CRYSTAL_ROTATE_COUNTER)) + u16::from(vertical_irq_increment);
        ram.write_u8(CRYSTAL_ROTATE_COUNTER, total as u8);
        if ram.read_u8(NMI_THREAD_ACTIVE) != 0 && total >> 8 != 0 {
            3
        } else {
            1
        }
    }
}

pub trait FrameHooks {
    fn initialize(&mut self, ram: &mut Ram<'_>);
    fn run_poly_loop(&mut self, ram: &mut Ram<'_>);
    fn module_main_routing(&mut self, ram: &mut Ram<'_>);
    fn prepare_sprites_for_nmi(&mut self, ram: &mut Ram<'_>);
    fn interrupt_nmi(&mut self, ram: &mut Ram<'_>, input: u16);
}

pub struct CoreFrameHooks<'a, 'assets, Route, LoadTrack> {
    pub sram: &'a mut [u8; SRAM_SIZE],
    pub ppu: &'a mut Ppu,
    pub audio: &'a mut crate::audio::AudioRuntime,
    pub nmi_assets: crate::nmi::NmiAssets<'assets>,
    pub intro_song_bank: &'assets [u8],
    pub route_main_module: Route,
    pub load_pcm_track: LoadTrack,
}

impl<Route, LoadTrack> FrameHooks for CoreFrameHooks<'_, '_, Route, LoadTrack>
where
    Route: FnMut(&mut Ram<'_>),
    LoadTrack: FnMut(u8) -> Option<crate::audio::PcmTrack>,
{
    fn initialize(&mut self, ram: &mut Ram<'_>) {
        self.audio
            .load_song_bank(self.intro_song_bank)
            .expect("valid compiled intro song bank");
        startup_initialize_memory(ram, self.sram);
        ram.write_u16(ANIMATED_TILE_DATA_SRC, 0xa680);
        ram.write_u16(DMA_SOURCE_ADDR_9, 0xb280);
        ram.write_u16(DMA_SOURCE_ADDR_14, 0xb2e0);
    }

    fn run_poly_loop(&mut self, ram: &mut Ram<'_>) {
        if ram.read_u8(0x1f00) != 0 && ram.read_u8(0x1f0c) == 0 {
            crate::poly::run_frame(ram);
            ram.write_u8(0x1f00, 0);
            ram.write_u8(0x1f0c, 0xff);
        }
    }

    fn module_main_routing(&mut self, ram: &mut Ram<'_>) {
        (self.route_main_module)(ram);
    }

    fn prepare_sprites_for_nmi(&mut self, ram: &mut Ram<'_>) {
        crate::misc::nmi_prepare_sprites(ram);
    }

    fn interrupt_nmi(&mut self, ram: &mut Ram<'_>, input: u16) {
        let mut audio = crate::audio::NmiAudioAdapter {
            runtime: self.audio,
            area: ram.read_u16(0x40a) as u8,
            entrance: u16::from(ram.read_u8(0x10e)),
            music_unk1: ram.read_u8(0x130),
            last_music_control: ram.read_u8(0x133),
            misc_bug_fixes: ram.read_u32(ENHANCED_FEATURES0) & 4096 != 0,
            load_pcm_track: &mut self.load_pcm_track,
        };
        crate::nmi::interrupt_nmi(ram, self.ppu, &self.nmi_assets, &mut audio, input)
            .expect("valid native NMI state");
    }
}

pub fn run_frame_internal(
    ram: &mut Ram<'_>,
    hooks: &mut impl FrameHooks,
    input: u16,
    run_what: u8,
) {
    if ram.read_u16(ANIMATED_TILE_DATA_SRC) == 0 {
        hooks.initialize(ram);
    }
    if run_what & 2 != 0 {
        hooks.run_poly_loop(ram);
    }
    if run_what & 1 != 0 {
        ram.write_u8(FRAME_COUNTER, ram.read_u8(FRAME_COUNTER).wrapping_add(1));
        clear_oam_buffer(ram);
        hooks.module_main_routing(ram);
        hooks.prepare_sprites_for_nmi(ram);
        ram.write_u8(NMI_BOOLEAN, 0);
    }
    hooks.interrupt_nmi(ram, input);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateRecorder {
    pub last_inputs: u16,
    pub frames_since_last: u32,
    pub total_frames: u32,
    pub log: Vec<u8>,
    pub base_snapshot: Vec<u8>,
    replay_position: usize,
    replay_position_last_complete: usize,
    replay_frame_counter: u32,
    replay_next_command_at: u32,
    replay_command: u8,
    replay_mode: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayError {
    TruncatedLog,
    InvalidCommand(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateFileError {
    Truncated,
    UnsupportedVersion(u32),
    SizeOverflow,
}

impl StateRecorder {
    fn append_variable_length(&mut self, mut value: u32) {
        while value >= 255 {
            self.log.push(255);
            value -= 255;
        }
        self.log.push(value as u8);
    }

    fn record_command(&mut self, command: u8) {
        let frames = self.frames_since_last;
        self.frames_since_last = 0;
        let inline_limit = if command < 0xc0 { 0x0f } else { 1 };
        self.log.push(command | frames.min(inline_limit) as u8);
        if frames >= inline_limit {
            self.append_variable_length(frames - inline_limit);
        }
    }

    pub fn record(&mut self, inputs: u16) {
        let difference = inputs ^ self.last_inputs;
        if difference != 0 {
            self.last_inputs = inputs;
            for bit in 0..12 {
                if difference >> bit & 1 != 0 {
                    self.record_command(bit << 4);
                }
            }
        }
        self.frames_since_last = self.frames_since_last.wrapping_add(1);
        self.total_frames = self.total_frames.wrapping_add(1);
    }

    pub fn record_patch(&mut self, address: u32, bytes: &[u8]) {
        assert!(address < 0x20_000);
        assert!(!bytes.is_empty());
        let length_minus_one = bytes.len() - 1;
        let length_code = length_minus_one.min(3);
        self.record_command(
            0xc0 | u8::from(address & 0x1_0000 != 0) << 1 | (length_code as u8) << 2,
        );
        if length_code == 3 {
            self.append_variable_length((length_minus_one - 3) as u32);
        }
        self.log.push((address >> 8) as u8);
        self.log.push(address as u8);
        self.log.extend_from_slice(bytes);
    }

    /// Starts playback of the current key log. Loading the optional base
    /// snapshot is kept at the runtime boundary, matching the C port's order:
    /// install the snapshot (or reset), then call this method.
    pub fn start_replay(&mut self) {
        self.frames_since_last = 0;
        self.last_inputs = 0;
        self.replay_position = 0;
        self.replay_position_last_complete = 0;
        self.replay_frame_counter = 0;
        self.replay_next_command_at = 0;
        self.replay_command = 0;
        self.replay_mode = true;
    }

    #[must_use]
    pub fn is_replaying(&self) -> bool {
        self.replay_mode
    }

    fn replay_byte(&self, position: &mut usize) -> Result<u8, ReplayError> {
        let value = self
            .log
            .get(*position)
            .copied()
            .ok_or(ReplayError::TruncatedLog)?;
        *position += 1;
        Ok(value)
    }

    fn replay_variable_length(&self, position: &mut usize) -> Result<u32, ReplayError> {
        let mut value = 0u32;
        loop {
            let byte = self.replay_byte(position)?;
            value = value.wrapping_add(u32::from(byte));
            if byte != 255 {
                return Ok(value);
            }
        }
    }

    pub fn read_next_replay_state(&mut self, ram: &mut Ram<'_>) -> Result<u16, ReplayError> {
        assert!(self.replay_mode);
        while self.frames_since_last >= self.replay_next_command_at {
            let mut position = self.replay_position;
            if position != self.replay_position_last_complete {
                self.frames_since_last = 0;
                if self.replay_command < 0xc0 {
                    self.last_inputs ^= 1 << (self.replay_command >> 4);
                } else if self.replay_command < 0xd0 {
                    let mut count = 1 + usize::from((self.replay_command >> 2) & 3);
                    if count == 4 {
                        count += self.replay_variable_length(&mut position)? as usize;
                    }
                    let mut address = u32::from((self.replay_command >> 1) & 1) << 16;
                    address |= u32::from(self.replay_byte(&mut position)?) << 8;
                    address |= u32::from(self.replay_byte(&mut position)?);
                    for offset in 0..count {
                        let value = self.replay_byte(&mut position)?;
                        ram.write_u8((address as usize + offset) & 0x1_ffff, value);
                    }
                } else {
                    return Err(ReplayError::InvalidCommand(self.replay_command));
                }
            }
            self.replay_position_last_complete = position;
            if position >= self.log.len() {
                self.replay_position = position;
                self.replay_next_command_at = u32::MAX;
                break;
            }

            let command = self.replay_byte(&mut position)?;
            let mask = if command < 0xc0 { 0x0f } else { 1 };
            let mut frames = u32::from(command & mask);
            if frames == u32::from(mask) {
                frames = frames.wrapping_add(self.replay_variable_length(&mut position)?);
            }
            self.replay_next_command_at = frames;
            self.replay_command = command;
            self.replay_position = position;
        }
        self.frames_since_last = self.frames_since_last.wrapping_add(1);
        self.replay_frame_counter = self.replay_frame_counter.wrapping_add(1);
        if self.replay_frame_counter >= self.total_frames {
            self.replay_mode = false;
        }
        Ok(self.last_inputs)
    }

    pub fn stop_replay(&mut self) {
        if !self.replay_mode {
            return;
        }
        self.replay_mode = false;
        self.total_frames = self.replay_frame_counter;
        self.log.truncate(self.replay_position_last_complete);
    }

    /// Re-bases the key log on the supplied current machine snapshot. Active
    /// buttons become zero-time toggle commands, exactly as in the C recorder.
    pub fn clear_key_log(&mut self, current_snapshot: &[u8]) {
        self.base_snapshot.clear();
        self.base_snapshot.extend_from_slice(current_snapshot);
        let old_log = core::mem::take(&mut self.log);
        let old_frames_since_last = self.frames_since_last;
        self.frames_since_last = 0;
        if self.last_inputs != 0 {
            for bit in 0..12 {
                if self.last_inputs >> bit & 1 != 0 {
                    self.record_command(bit << 4);
                }
            }
        }
        if self.replay_mode {
            if self.replay_next_command_at != u32::MAX {
                self.replay_next_command_at = self
                    .replay_next_command_at
                    .wrapping_sub(old_frames_since_last);
                self.frames_since_last = self.replay_next_command_at;
                self.replay_position_last_complete = self.log.len();
                self.record_command(self.replay_command);
                let old_position = self.replay_position;
                self.replay_position = self.log.len();
                self.log.extend_from_slice(&old_log[old_position..]);
            }
            self.total_frames = self.total_frames.wrapping_sub(self.replay_frame_counter);
            self.replay_frame_counter = 0;
        } else {
            self.total_frames = 0;
        }
        self.frames_since_last = 0;
    }

    /// Encodes the exact eight-word state-recorder header followed by the key
    /// log, optional replay base snapshot, and current machine snapshot.
    #[must_use]
    pub fn encode_state_file(&self, current_snapshot: &[u8]) -> Vec<u8> {
        assert!(
            self.base_snapshot.is_empty() || self.base_snapshot.len() == current_snapshot.len()
        );
        let mut header = [0u32; 8];
        header[0] = 1;
        header[1] = self.total_frames;
        header[2] = self.log.len().try_into().expect("key log fits C u32");
        header[3] = u32::from(self.last_inputs);
        header[4] = self.frames_since_last;
        header[5] = u32::from(!self.base_snapshot.is_empty());
        header[6] = current_snapshot
            .len()
            .try_into()
            .expect("snapshot fits C u32");
        if self.replay_mode {
            header[5] |= (self.replay_position_last_complete as u32) << 1;
            header[7] = self.replay_frame_counter;
        }
        let mut bytes = Vec::with_capacity(
            32 + self.log.len() + self.base_snapshot.len() + current_snapshot.len(),
        );
        for word in header {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(&self.log);
        bytes.extend_from_slice(&self.base_snapshot);
        bytes.extend_from_slice(current_snapshot);
        bytes
    }

    /// Decodes a C-compatible recorder file and returns the machine snapshot
    /// which the caller must install before advancing another frame.
    pub fn decode_state_file(
        bytes: &[u8],
        replay_from_start: bool,
    ) -> Result<(Self, Vec<u8>), StateFileError> {
        if bytes.len() < 32 {
            return Err(StateFileError::Truncated);
        }
        let mut header = [0u32; 8];
        for (word, chunk) in header.iter_mut().zip(bytes[..32].chunks_exact(4)) {
            *word = u32::from_le_bytes(chunk.try_into().expect("four-byte chunk"));
        }
        if header[0] != 1 {
            return Err(StateFileError::UnsupportedVersion(header[0]));
        }
        let log_size = usize::try_from(header[2]).map_err(|_| StateFileError::SizeOverflow)?;
        let snapshot_size = usize::try_from(header[6]).map_err(|_| StateFileError::SizeOverflow)?;
        let base_size = if header[5] & 1 != 0 { snapshot_size } else { 0 };
        let required = 32usize
            .checked_add(log_size)
            .and_then(|size| size.checked_add(base_size))
            .and_then(|size| size.checked_add(snapshot_size))
            .ok_or(StateFileError::SizeOverflow)?;
        if bytes.len() < required {
            return Err(StateFileError::Truncated);
        }
        let mut position = 32;
        let log = bytes[position..position + log_size].to_vec();
        position += log_size;
        let base_snapshot = bytes[position..position + base_size].to_vec();
        position += base_size;
        let current_snapshot = bytes[position..position + snapshot_size].to_vec();

        let mut recorder = Self {
            last_inputs: header[3] as u16,
            frames_since_last: header[4],
            total_frames: header[1],
            log,
            base_snapshot,
            ..Self::default()
        };
        let snapshot_to_load = if replay_from_start {
            let snapshot = recorder.base_snapshot.clone();
            recorder.start_replay();
            snapshot
        } else {
            recorder.replay_position = (header[5] >> 1) as usize;
            recorder.replay_position_last_complete = recorder.replay_position;
            recorder.replay_frame_counter = header[7];
            recorder.replay_mode = recorder.replay_frame_counter != 0;
            current_snapshot
        };
        Ok((recorder, snapshot_to_load))
    }
}

pub struct MultiPatch<'a> {
    recorder: &'a mut StateRecorder,
    start: u32,
    values: Vec<u8>,
}

impl<'a> MultiPatch<'a> {
    pub fn new(recorder: &'a mut StateRecorder) -> Self {
        Self {
            recorder,
            start: 0,
            values: Vec::with_capacity(256),
        }
    }

    pub fn patch(&mut self, ram: &mut Ram<'_>, address: u32, value: u8) {
        if self.values.len() >= 256 || address != self.start + self.values.len() as u32 {
            self.commit();
            self.start = address;
        }
        self.values.push(value);
        ram.write_u8(address as usize, value);
    }

    pub fn commit(&mut self) {
        if !self.values.is_empty() {
            self.recorder.record_patch(self.start, &self.values);
            self.values.clear();
        }
    }
}

impl Drop for MultiPatch<'_> {
    fn drop(&mut self) {
        self.commit();
    }
}

pub fn apply_patch_command(
    ram: &mut Ram<'_>,
    recorder: &mut StateRecorder,
    command: char,
    current_snapshot: &[u8],
) {
    if command == 'k' {
        recorder.clear_key_log(current_snapshot);
        return;
    }
    if command == 'l' {
        recorder.stop_replay();
        return;
    }
    let mut patch = MultiPatch::new(recorder);
    match command {
        'w' => {
            patch.patch(ram, 0xf372, 80);
            patch.patch(ram, 0xf373, 80);
        }
        'W' => {
            patch.patch(ram, 0xf375, 10);
            patch.patch(ram, 0xf376, 10);
            let rupees = ram.read_u16(0xf360).wrapping_add(100);
            patch.patch(ram, 0xf360, rupees as u8);
            patch.patch(ram, 0xf361, (rupees >> 8) as u8);
        }
        'o' => patch.patch(ram, 0xf36f, 1),
        'E' => {
            let value = ram.read_u8(0x37f) ^ 1;
            patch.patch(ram, 0x37f, value);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[derive(Default)]
    struct Hooks(Vec<&'static str>);
    impl FrameHooks for Hooks {
        fn initialize(&mut self, ram: &mut Ram<'_>) {
            self.0.push("init");
            ram.write_u16(ANIMATED_TILE_DATA_SRC, 1);
        }
        fn run_poly_loop(&mut self, _: &mut Ram<'_>) {
            self.0.push("poly");
        }
        fn module_main_routing(&mut self, _: &mut Ram<'_>) {
            self.0.push("game");
        }
        fn prepare_sprites_for_nmi(&mut self, _: &mut Ram<'_>) {
            self.0.push("sprites");
        }
        fn interrupt_nmi(&mut self, _: &mut Ram<'_>, _: u16) {
            self.0.push("nmi");
        }
    }

    #[test]
    fn startup_preserves_valid_slots_and_clears_invalid_signatures() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut sram = [0xa5; 0x2000];
        sram[0x3e5..0x3e7].copy_from_slice(&0x55aau16.to_le_bytes());
        let mut ram = Ram::new(&mut bytes);
        startup_initialize_memory(&mut ram, &mut sram);
        assert!(
            ram.slice(0, 0x2000)
                .iter()
                .enumerate()
                .all(|(index, &byte)| index == INIDISP_COPY
                    || index == FLAG_UPDATE_CGRAM
                    || byte == 0)
        );
        assert_eq!(ram.read_u8(0x2000), 0xa5);
        assert_eq!(&sram[0x3e5..0x3e7], &0x55aau16.to_le_bytes());
        assert_eq!(&sram[0x8e5..0x8e7], &[0, 0]);
        assert_eq!(ram.read_u8(INIDISP_COPY), 0x80);
        assert_eq!(ram.read_u8(FLAG_UPDATE_CGRAM), 1);
    }

    #[test]
    fn initialization_sequence_sets_native_dma_sources_after_sound_upload() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut sram = [0; 0x2000];
        let mut events = Vec::new();
        let mut sound_loaded = false;
        {
            let mut ram = Ram::new(&mut bytes);
            initialization_code(
                &mut ram,
                &mut sram,
                |address, value| events.push((address, value)),
                || sound_loaded = true,
            );
            assert_eq!(ram.read_u16(ANIMATED_TILE_DATA_SRC), 0xa680);
            assert_eq!(ram.read_u16(DMA_SOURCE_ADDR_9), 0xb280);
            assert_eq!(ram.read_u16(DMA_SOURCE_ADDR_14), 0xb2e0);
        }
        assert!(sound_loaded);
        assert_eq!(
            events,
            [(0x4200, 0), (0x420c, 0), (0x420b, 0), (0x4200, 0x81)]
        );
    }

    #[test]
    fn reset_clears_wram_and_optionally_preserves_sram() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut sram = [0x5a; 0x2000];
        let mut dma = Dma::new();
        let mut ppu = Ppu::new();
        let mut audio = crate::audio::AudioRuntime::default();
        let mut ram = Ram::new(&mut bytes);
        reset_runtime(&mut ram, &mut sram, &mut dma, &mut ppu, &mut audio, true);
        assert!(ram.slice(0, WRAM_SIZE).iter().all(|&byte| byte == 0));
        assert!(sram.iter().all(|&byte| byte == 0x5a));

        ram.write_u8(0, 1);
        reset_runtime(&mut ram, &mut sram, &mut dma, &mut ppu, &mut audio, false);
        assert!(sram.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn owned_runtime_exports_and_imports_the_canonical_oracle_regions() {
        let mut machine = NativeRuntime::new();
        machine.wram[3] = 0x11;
        machine.sram[4] = 0x22;
        machine.ppu.vram[5] = 0x4433;
        let state = machine.canonical_state();
        assert_eq!(state.wram[3], 0x11);
        assert_eq!(state.sram[4], 0x22);
        assert_eq!(state.vram[5], 0x4433);

        machine.wram.fill(0);
        machine.sram.fill(0);
        machine.ppu.vram.fill(0);
        machine.import_canonical_state(&state);
        assert_eq!(machine.canonical_state(), state);
    }

    #[test]
    fn host_frame_entry_records_feature_patches_runs_core_and_queues_audio() {
        let mut machine = NativeRuntime::new();
        machine.wram[ANIMATED_TILE_DATA_SRC] = 1;
        machine.wanted_features = 0x7856_3412;
        let mut hooks = Hooks::default();
        assert_eq!(machine.run_frame(&mut hooks, 0x0031), Ok(false));
        assert_eq!(hooks.0, ["game", "sprites", "nmi"]);
        assert_eq!(machine.wram[BUGS_FIXED], 1);
        assert_eq!(
            u32::from_le_bytes(
                machine.wram[ENHANCED_FEATURES0..ENHANCED_FEATURES0 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x7856_3412
        );
        assert_eq!(machine.audio.queue.queued(), 1);
        // Up+down normalized away, leaving only bit zero.
        assert_eq!(machine.recorder.last_inputs, 1);
    }

    #[test]
    fn concrete_core_hooks_route_module_then_execute_native_nmi() {
        static EMPTY: [u8; 0] = [];
        static LINK_GRAPHICS: [u8; 0x7000] = [0; 0x7000];
        let mut machine = NativeRuntime::new();
        machine.wram[ANIMATED_TILE_DATA_SRC] = 1;
        machine.wram[0x12] = 1;
        machine.wram[0x94] = 7;
        let nmi_assets = crate::nmi::NmiAssets {
            link_graphics: &LINK_GRAPHICS,
            bg_tilemaps: [&EMPTY; 6],
            light_overworld_tilemap: &EMPTY,
        };
        machine
            .run_core_frame(
                0x8000,
                nmi_assets,
                &EMPTY,
                |ram| ram.write_u8(0x700, 0x5a),
                |_| None,
            )
            .unwrap();
        assert_eq!(machine.wram[0x700], 0x5a);
        assert_eq!(machine.audio.queue.queued(), 1);
    }

    #[test]
    fn run_mask_and_frame_order_match_both_poly_schedulers() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(NMI_THREAD_ACTIVE, 1);
        ram.write_u16(THREAD_OTHER_STACK, 0x1234);
        assert_eq!(determine_run_what(&mut ram, 7), 2);
        ram.write_u8(BUGS_FIXED, BUG_FIX_POLY_RENDERER);
        ram.write_u8(CRYSTAL_ROTATE_COUNTER, 250);
        assert_eq!(determine_run_what(&mut ram, 10), 3);
        let mut hooks = Hooks::default();
        run_frame_internal(&mut ram, &mut hooks, 0x1234, 3);
        assert_eq!(hooks.0, ["init", "poly", "game", "sprites", "nmi"]);
        assert_eq!(ram.read_u8(FRAME_COUNTER), 1);
        assert_eq!(ram.read_u8(OAM_BUFFER + 1), 0xf0);
    }

    #[test]
    fn opposite_directions_cancel_as_pairs() {
        assert_eq!(normalize_inputs(0xf0), 0);
        assert_eq!(normalize_inputs(0x31), 1);
        assert_eq!(normalize_inputs(0xc2), 2);
    }

    #[test]
    fn recorder_matches_toggle_delay_and_extended_patch_encoding() {
        let mut recorder = StateRecorder::default();
        recorder.record(0);
        recorder.record(1);
        for _ in 0..20 {
            recorder.record(1);
        }
        recorder.record(0x801);
        recorder.record_patch(0x1fffe, &[1, 2, 3, 4, 5]);
        assert_eq!(
            recorder.log,
            [0x01, 0xbf, 6, 0xcf, 0, 1, 0xff, 0xfe, 1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn replay_applies_input_toggles_and_patches_on_the_recorded_frames() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut recorder = StateRecorder {
            total_frames: 2,
            // Toggle input bit 0 immediately, then patch $0123 one frame later.
            log: vec![0x00, 0xc1, 0x00, 0x01, 0x23, 0xaa],
            ..StateRecorder::default()
        };
        recorder.start_replay();
        assert_eq!(recorder.read_next_replay_state(&mut ram), Ok(1));
        assert_eq!(ram.read_u8(0x123), 0);
        assert_eq!(recorder.read_next_replay_state(&mut ram), Ok(1));
        assert_eq!(ram.read_u8(0x123), 0xaa);
        assert!(!recorder.is_replaying());
    }

    #[test]
    fn recorder_state_file_round_trips_c_header_and_snapshot_selection() {
        let recorder = StateRecorder {
            last_inputs: 0x123,
            frames_since_last: 7,
            total_frames: 99,
            log: vec![0x01, 0x42],
            base_snapshot: vec![1, 2, 3],
            ..StateRecorder::default()
        };
        let encoded = recorder.encode_state_file(&[4, 5, 6]);
        assert_eq!(&encoded[..4], &1u32.to_le_bytes());
        let (resumed, snapshot) =
            StateRecorder::decode_state_file(&encoded, false).expect("valid recorder file");
        assert_eq!(snapshot, [4, 5, 6]);
        assert_eq!(resumed.last_inputs, 0x123);
        assert_eq!(resumed.log, [0x01, 0x42]);

        let (replayed, snapshot) =
            StateRecorder::decode_state_file(&encoded, true).expect("valid recorder file");
        assert_eq!(snapshot, [1, 2, 3]);
        assert!(replayed.is_replaying());
        assert_eq!(replayed.last_inputs, 0);
    }

    #[test]
    fn clearing_key_log_rebases_active_inputs_on_current_snapshot() {
        let mut recorder = StateRecorder {
            last_inputs: 0x0005,
            frames_since_last: 12,
            total_frames: 30,
            log: vec![0xaa, 0xbb],
            ..StateRecorder::default()
        };
        recorder.clear_key_log(&[9, 8, 7]);
        assert_eq!(recorder.base_snapshot, [9, 8, 7]);
        assert_eq!(recorder.log, [0x00, 0x20]);
        assert_eq!(recorder.frames_since_last, 0);
        assert_eq!(recorder.total_frames, 0);
    }

    #[test]
    fn multi_patch_groups_only_contiguous_runs_up_to_256_bytes() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut recorder = StateRecorder::default();
        {
            let mut patch = MultiPatch::new(&mut recorder);
            patch.patch(&mut ram, 0x100, 7);
            patch.patch(&mut ram, 0x101, 8);
            patch.patch(&mut ram, 0x200, 9);
        }
        assert_eq!(ram.slice(0x100, 2), &[7, 8]);
        assert_eq!(ram.read_u8(0x200), 9);
        assert_eq!(recorder.log, [0xc4, 1, 0, 7, 8, 0xc0, 2, 0, 9]);
    }

    #[test]
    fn patch_commands_preserve_c_addresses_widths_and_wrapping() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(0xf360, 0xfff0);
        let mut recorder = StateRecorder::default();
        apply_patch_command(&mut ram, &mut recorder, 'W', &[]);
        assert_eq!(ram.slice(0xf375, 2), &[10, 10]);
        assert_eq!(ram.read_u16(0xf360), 0x0054);
        apply_patch_command(&mut ram, &mut recorder, 'E', &[]);
        assert_eq!(ram.read_u8(0x37f), 1);
        apply_patch_command(&mut ram, &mut recorder, 'E', &[]);
        assert_eq!(ram.read_u8(0x37f), 0);
    }

    #[test]
    fn simple_hdma_runs_direct_and_relocated_indirect_tables() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut ppu = Ppu::new();
        let mut channel = DmaChannel {
            a_address: 0x600,
            a_bank: 0,
            b_address: 0,
            mode: 0,
            indirect: false,
            hdma_active: true,
            ..DmaChannel::default()
        };
        ram.write_u8(0x600, 1);
        ram.write_u8(0x601, 0x0f);
        ram.write_u8(0x602, 0);
        let mut hdma = SimpleHdma::from_channel(&channel);
        hdma.do_line(&ram, &mut ppu);
        assert_eq!(ppu.brightness, 0x0f);
        hdma.do_line(&ram, &mut ppu);
        assert_eq!(hdma.table, None);

        channel.indirect = true;
        ram.write_u8(0x600, 1);
        ram.write_u16(0x601, 0x1b00);
        ram.write_u8(0x603, 0);
        ram.write_u8(0x1dba0, 7);
        let mut hdma = SimpleHdma::from_channel(&channel);
        hdma.do_line(&ram, &mut ppu);
        assert_eq!(ppu.brightness, 7);
    }

    #[test]
    fn hdma_setup_writes_exact_channel_six_and_seven_register_fields() {
        let mut dma = Dma::new();
        hdma_setup(&mut dma, 0x0a_bdcf, 0x02_c80c, 0x42, 0x1b, 0x1c, 0x7e);
        assert_eq!(dma.channels[6].mode, 2);
        assert!(dma.channels[6].indirect);
        assert_eq!(dma.channels[6].b_address, 0x1b);
        assert_eq!(dma.channels[6].a_address, 0xbdcf);
        assert_eq!(dma.channels[6].a_bank, 0x0a);
        assert_eq!(dma.channels[6].indirect_bank, 0x7e);
        assert_eq!(dma.channels[7].b_address, 0x1c);
        assert_eq!(dma.channels[7].a_address, 0xc80c);
        assert_eq!(dma.channels[7].a_bank, 0x02);
    }

    #[test]
    fn ppu_frame_runs_irq_split_and_native_height() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(0x128, 0x80);
        ram.write_u16(0x630, 0x1234);
        let mut dma = Dma::new();
        let mut ppu = Ppu::new();
        let mut nmitimen = Vec::new();
        draw_ppu_frame(&mut ram, &mut dma, &mut ppu, 256 * 4, 0, |value| {
            nmitimen.push(value)
        });
        assert_eq!(ppu.pixels().len(), 256 * 4 * 224);
        assert_eq!(ram.read_u8(0x128), 0);
        assert_eq!(nmitimen, [0x81]);
    }

    #[test]
    fn every_reference_snapshot_decodes_and_reencodes_byte_identically() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/ref");
        let mut count = 0;
        for entry in std::fs::read_dir(directory).expect("reference save directory") {
            let path = entry.expect("reference save entry").path();
            if path.extension().and_then(std::ffi::OsStr::to_str) != Some("sav") {
                continue;
            }
            let file = std::fs::read(&path).expect("reference save");
            let start = file.len() - LEGACY_SNAPSHOT_LEN;
            let original = &file[start..];
            let mut runtime = NativeRuntime::new();
            let mut imported = original.to_vec();
            runtime
                .internal_save_load(&mut imported, true)
                .expect("decode reference snapshot");
            let mut encoded = vec![0; LEGACY_SNAPSHOT_LEN];
            runtime
                .internal_save_load(&mut encoded, false)
                .expect("encode reference snapshot");
            assert_eq!(encoded, original, "{}", path.display());
            count += 1;
        }
        assert_eq!(count, 13);
    }

    #[test]
    fn legacy_snapshot_rejects_wrong_length() {
        let mut runtime = NativeRuntime::new();
        assert!(matches!(
            runtime.load_snes_state(&[0; 3]),
            Err(LegacySnapshotError::WrongLength(3))
        ));
    }

    #[test]
    fn save_and_load_restore_every_live_legacy_device_region() {
        let mut runtime = NativeRuntime::new();
        runtime.wram[0x1234] = 0x12;
        runtime.wram[HDMA_TABLE_NEW + 7] = 0x34;
        runtime.sram[0x456] = 0x56;
        runtime.audio.player.ram[0x5678] = 0x78;
        runtime.audio.player.dsp.channels[2].pitch = 0x2345;
        runtime.dma.channels[3].table_address = 0x3456;
        runtime.ppu.vram[0x4567] = 0x4567;
        runtime.ppu.cgram[17] = 0x5678;
        runtime.ppu.bg_layer[1].tilemap_address = 0x6789;
        let saved = runtime.save_snes_state().unwrap();

        runtime.wram.fill(0);
        runtime.sram.fill(0);
        runtime.audio.player.ram.fill(0);
        runtime.audio.player.dsp.reset();
        runtime.dma.reset();
        runtime.ppu.reset();
        runtime.load_snes_state(&saved).unwrap();

        assert_eq!(runtime.wram[0x1234], 0x12);
        assert_eq!(runtime.wram[HDMA_TABLE_NEW + 7], 0x34);
        assert_eq!(runtime.sram[0x456], 0x56);
        assert_eq!(runtime.audio.player.ram[0x5678], 0x78);
        assert_eq!(runtime.audio.player.dsp.channels[2].pitch, 0x2345);
        assert_eq!(runtime.dma.channels[3].table_address, 0x3456);
        assert_eq!(runtime.ppu.vram[0x4567], 0x4567);
        assert_eq!(runtime.ppu.cgram[17], 0x5678);
        assert_eq!(runtime.ppu.bg_layer[1].tilemap_address, 0x6789);
        assert_eq!(runtime.save_snes_state().unwrap(), saved);
    }
}
