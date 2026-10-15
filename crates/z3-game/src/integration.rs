//! Composition of the translated game modules.
//!
//! The individual translation units deliberately expose their cross-unit calls
//! as traits.  This module owns the original `kMainRouting` ordering and gives
//! the host one concrete place to connect those units without duplicating (or
//! subtly reordering) the C dispatch table.

use std::cell::RefCell;

use crate::{
    assets::RuntimeAssets,
    attract::{Attract, AttractRuntime},
    dungeon::{Dungeon, DungeonHooks},
    ending::{Ending, EndingCall, EndingRuntime},
    messaging::{Messaging, MessagingAssets, MessagingRuntime},
    misc::{self, MiscHooks},
    overworld,
    ram::Ram,
    select_file::{FileSelect, FileSelectRuntime},
};
use z3_core::SRAM_SIZE;
use z3_snes::ppu::Ppu;

const MAIN_MODULE: usize = 0x10;

/// The 28 entries of C's `kMainRouting`, in their original numeric order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MainModule {
    Intro = 0x00,
    FileSelect = 0x01,
    CopyFile = 0x02,
    EraseFile = 0x03,
    NameFile = 0x04,
    LoadFile = 0x05,
    PreDungeon = 0x06,
    Dungeon = 0x07,
    OverworldLoad = 0x08,
    Overworld = 0x09,
    SpecialOverworldLoad = 0x0a,
    SpecialOverworld = 0x0b,
    Unknown0 = 0x0c,
    Unknown1 = 0x0d,
    Interface = 0x0e,
    SpotlightClose = 0x0f,
    SpotlightOpen = 0x10,
    DungeonFallingEntrance = 0x11,
    GameOver = 0x12,
    BossVictoryPendant = 0x13,
    Attract = 0x14,
    MirrorWarpFromAga = 0x15,
    BossVictoryCrystal = 0x16,
    SaveAndQuit = 0x17,
    GanonEmerges = 0x18,
    TriforceRoom = 0x19,
    Credits = 0x1a,
    SpawnSelect = 0x1b,
}

impl MainModule {
    #[must_use]
    pub const fn from_index(index: u8) -> Option<Self> {
        Some(match index {
            0x00 => Self::Intro,
            0x01 => Self::FileSelect,
            0x02 => Self::CopyFile,
            0x03 => Self::EraseFile,
            0x04 => Self::NameFile,
            0x05 => Self::LoadFile,
            0x06 => Self::PreDungeon,
            0x07 => Self::Dungeon,
            0x08 => Self::OverworldLoad,
            0x09 => Self::Overworld,
            0x0a => Self::SpecialOverworldLoad,
            0x0b => Self::SpecialOverworld,
            0x0c => Self::Unknown0,
            0x0d => Self::Unknown1,
            0x0e => Self::Interface,
            0x0f => Self::SpotlightClose,
            0x10 => Self::SpotlightOpen,
            0x11 => Self::DungeonFallingEntrance,
            0x12 => Self::GameOver,
            0x13 => Self::BossVictoryPendant,
            0x14 => Self::Attract,
            0x15 => Self::MirrorWarpFromAga,
            0x16 => Self::BossVictoryCrystal,
            0x17 => Self::SaveAndQuit,
            0x18 => Self::GanonEmerges,
            0x19 => Self::TriforceRoom,
            0x1a => Self::Credits,
            0x1b => Self::SpawnSelect,
            _ => return None,
        })
    }
}

/// Cross-unit module entries which cannot be called by `misc.rs` itself.
///
/// The five main-module implementations that live in `misc.c` are invoked
/// directly by [`module_main_routing`]. Implementations of this trait should
/// use the typed adapters below for the already translated state machines.
pub trait MainModuleRuntime: MiscHooks {
    fn module00_intro(&mut self, ram: &mut Ram<'_>);
    fn module01_file_select(&mut self, ram: &mut Ram<'_>);
    fn module02_copy_file(&mut self, ram: &mut Ram<'_>);
    fn module03_erase_file(&mut self, ram: &mut Ram<'_>);
    fn module04_name_file(&mut self, ram: &mut Ram<'_>);
    fn module06_pre_dungeon(&mut self, ram: &mut Ram<'_>);
    fn module07_dungeon(&mut self, ram: &mut Ram<'_>);
    fn module08_overworld_load(&mut self, ram: &mut Ram<'_>);
    fn module09_overworld(&mut self, ram: &mut Ram<'_>);
    fn module0a_special_overworld_load(&mut self, ram: &mut Ram<'_>);
    fn module0b_special_overworld(&mut self, ram: &mut Ram<'_>);
    fn module0e_interface(&mut self, ram: &mut Ram<'_>);
    fn module0f_spotlight_close(&mut self, ram: &mut Ram<'_>);
    fn module10_spotlight_open(&mut self, ram: &mut Ram<'_>);
    fn module11_dungeon_falling_entrance(&mut self, ram: &mut Ram<'_>);
    fn module12_game_over(&mut self, ram: &mut Ram<'_>);
    fn module14_attract(&mut self, ram: &mut Ram<'_>);
    fn module18_ganon_emerges(&mut self, ram: &mut Ram<'_>);
    fn module19_triforce_room(&mut self, ram: &mut Ram<'_>);
    fn module1a_credits(&mut self, ram: &mut Ram<'_>);
    fn module1b_spawn_select(&mut self, ram: &mut Ram<'_>);
}

/// Cross-unit operations used by the two spotlight main modules.
pub trait SpotlightServices {
    fn prep_exit_with_spotlight(&mut self, ram: &mut Ram<'_>);
    fn configure_spotlight_table(&mut self, ram: &mut Ram<'_>);
    fn open_spotlight_next(&mut self, ram: &mut Ram<'_>);
    fn spotlight_open(&mut self, ram: &mut Ram<'_>);
    fn link_handle_velocity(&mut self, ram: &mut Ram<'_>);
    fn link_handle_moving_animation_full_long_entry(&mut self, ram: &mut Ram<'_>);
}

pub trait UnifiedGameplayServices:
    crate::sprite::SpriteHooks
    + crate::ancilla::AncillaRuntime
    + crate::overlord::OverlordHooks
    + crate::hud::HudRuntime
    + crate::player_oam::PlayerOamHooks
{
    fn run_sprite_main(&mut self, ram: &mut Ram<'_>)
    where
        Self: Sized,
    {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_main();
    }

    fn run_ancilla_main(&mut self, ram: &mut Ram<'_>)
    where
        Self: Sized,
    {
        crate::ancilla::Ancilla { ram, runtime: self }.main();
    }

    fn run_overlord_main(&mut self, ram: &mut Ram<'_>)
    where
        Self: Sized,
    {
        crate::overlord::main(ram, self);
    }

    fn run_hud_module(&mut self, ram: &mut Ram<'_>)
    where
        Self: Sized,
    {
        crate::hud::Hud { ram, runtime: self }.run_module();
    }

    fn run_link_oam_main(&mut self, ram: &mut Ram<'_>)
    where
        Self: Sized,
    {
        crate::player_oam::main(ram, self);
    }
}

impl<T> UnifiedGameplayServices for T where
    T: crate::sprite::SpriteHooks
        + crate::ancilla::AncillaRuntime
        + crate::overlord::OverlordHooks
        + crate::hud::HudRuntime
        + crate::player_oam::PlayerOamHooks
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreOverworldSubmodule {
    LoadProperties,
    LoadOverlays,
    LoadAndAdvance,
}

impl PreOverworldSubmodule {
    const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::LoadProperties),
            1 => Some(Self::LoadOverlays),
            2 => Some(Self::LoadAndAdvance),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverworldSubmodule {
    PlayerControl,
    LoadAuxGfx,
    FinishTransitionGfx,
    LoadNewMapAndGfx,
    LoadNewSprites,
    StartScrollTransition,
    RunScrollTransition,
    EaseOffScrollTransition,
    FinalizeEntry,
    OpenBigDoorFromExit,
    WalkFromExitFacingDown,
    WalkFromExitFacingUp,
    OpenBigDoor,
    StartMosaicTransition,
    PreOverworldLoadOverlays,
    FadeBackInFromMosaic,
    EnterSpecialArea,
    LoadSpecialAreaAndAdvance,
    MosaicTransition,
    Assertion1d,
    Assertion1e,
    SpecialExit,
    LoadOverlays2,
    LoadAmbientOverlayFalse,
    Brighten,
    MirrorWarp,
    LoadOverlays,
    LoadAndBuildScreen,
    RecoverFromDrowning,
    MasterSwordPalette,
    WeathervaneExplosion,
    Whirlpool,
    FinishPyramidHole,
}

impl OverworldSubmodule {
    const fn from_index(index: u8) -> Option<Self> {
        use OverworldSubmodule as O;
        Some(match index {
            0 => O::PlayerControl,
            1 | 15 | 26 | 38 => O::LoadAuxGfx,
            2 | 16 | 27 | 39 => O::FinishTransitionGfx,
            3 | 17 => O::LoadNewMapAndGfx,
            4 | 18 => O::LoadNewSprites,
            5 | 19 => O::StartScrollTransition,
            6 | 20 => O::RunScrollTransition,
            7 | 21 => O::EaseOffScrollTransition,
            8 => O::FinalizeEntry,
            9 => O::OpenBigDoorFromExit,
            10 => O::WalkFromExitFacingDown,
            11 => O::WalkFromExitFacingUp,
            12 => O::OpenBigDoor,
            13 | 23 | 36 => O::StartMosaicTransition,
            14 => O::PreOverworldLoadOverlays,
            22 | 41 => O::FadeBackInFromMosaic,
            24 => O::EnterSpecialArea,
            25 => O::LoadSpecialAreaAndAdvance,
            28 => O::MosaicTransition,
            29 => O::Assertion1d,
            30 => O::Assertion1e,
            31 => O::SpecialExit,
            32 => O::LoadOverlays2,
            33 => O::LoadAmbientOverlayFalse,
            34 => O::Brighten,
            35 | 44 => O::MirrorWarp,
            37 => O::LoadOverlays,
            40 => O::LoadAndBuildScreen,
            42 => O::RecoverFromDrowning,
            43 => O::MasterSwordPalette,
            45 => O::WeathervaneExplosion,
            46 => O::Whirlpool,
            47 => O::FinishPyramidHole,
            _ => return None,
        })
    }
}

/// Operations surrounding the already-translated overworld top-level loops.
/// The two dispatch methods are the remaining seam for the 48-entry overworld
/// and three-entry pre-overworld submodule tables.
pub trait OverworldServices: SpotlightServices {
    fn overworld_call(&mut self, ram: &mut Ram<'_>, call: OverworldCall);
    fn is_playing_music_track(&mut self, track: u8) -> bool;
    fn setup_mirror_hdma(&mut self, setup: overworld::MirrorHdmaSetup);
}

pub enum OverworldCall {
    LoadTransitionAuxGfx,
    PrepareTransitionAuxGfx,
    LoadEventOverlay,
    TriggerMapLoadStripe(overworld::MapStripeAxis, u8),
    LoadNewSpriteGfxSet,
    ReloadOverworldSprites,
    ReloadOverworldSpritesJustLoad,
    HandleMovingAnimation,
    IncrementalVramUpload,
    InitializeSpriteSlots,
    DisableFollower,
    ResetTransitionPropsAndInterface,
    ApplyPaletteFilterBounce,
    DecodeAnimatedSpriteTile(u8),
    LoadPalettes(u8, u8),
    ApplySpecialOverworldPalette,
    ResetSwimmingState,
    DrawQuadrantsAndOverlays,
    InitializeWhiteFilter,
    RunAnimationSubmodules,
    ResetHdmaAndIris,
    ReloadPreviouslyLoadedSheets,
    CacheQuadrantVisits,
    ResetLinkState,
    InitializeFollower,
    PaletteFilterBlindingWhite,
    PaletteFilterWhirlpoolBlue,
    PaletteFilterIsolateWhirlpoolBlue,
    FindPartnerWhirlpoolExit,
    LoadOverlayAndMap,
    WhirlpoolReloadPalettes,
    WhirlpoolRestoreRedGreen,
    WhirlpoolRestoreBlue,
}

/// Services needed by the translated main-module state machines.
pub trait GameServices:
    FrameResourceHooks
    + MiscHooks
    + crate::load_gfx::LoadGfxHooks
    + FileSelectRuntime
    + DungeonHooks
    + MessagingRuntime
    + AttractRuntime
    + EndingRuntime
    + OverworldServices
    + UnifiedGameplayServices
    + overworld::PreOverworldHooks
    + overworld::PlayerControlHooks
{
}

impl<T> GameServices for T where
    T: FrameResourceHooks
        + MiscHooks
        + crate::load_gfx::LoadGfxHooks
        + FileSelectRuntime
        + DungeonHooks
        + MessagingRuntime
        + AttractRuntime
        + EndingRuntime
        + OverworldServices
        + UnifiedGameplayServices
        + overworld::PreOverworldHooks
        + overworld::PlayerControlHooks
{
}

/// Per-frame values read from host-owned devices by translated gameplay code.
pub trait FrameResourceHooks {
    fn sync_vram(&mut self, vram: &[u16; z3_core::VRAM_WORDS]);
    fn set_audio_port0(&mut self, value: u8);
    fn set_music_playback_state(
        &mut self,
        state: u8,
        enabled: u8,
        actual_track: u8,
        music_unk1: u8,
        area: u8,
        entrance: u16,
    );
    fn take_mirror_hdma_setup(&mut self) -> Option<overworld::MirrorHdmaSetup>;
    fn take_load_gfx_device_commands(&mut self) -> LoadGfxDeviceCommands;
    fn take_host_actions(&mut self) -> Vec<NativeHostAction>;
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct LoadGfxDeviceCommands {
    pub ppu_writes: Vec<(u8, u8)>,
    pub hdma_setups: Vec<(u16, u8, u8)>,
    pub vram_spans: Vec<VramWriteSpan>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VramWriteSpan {
    pub start: usize,
    pub words: Vec<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeHostAction {
    HdmaSetup {
        addr6: u32,
        addr7: u32,
        transfer_unit: u8,
        reg6: u8,
        reg7: u8,
        indirect_bank: u8,
    },
    RenderText,
    DeathFunc15(bool),
    RouteMainModule,
    LoadSongBank(u8),
    EndingExternal(EndingCall),
    WorldMapLoadLight,
    TransferMode7Characters,
    WorldMapSetupHdma,
}

/// Resource owner shared by native hosts and differential verification.
///
/// Gameplay hook implementations are deliberately attached to this type so
/// the verifier and the interactive host exercise the same composition.
pub struct NativeGameServices<'assets> {
    pub assets: &'assets RuntimeAssets<'assets>,
    pub scratch_vram: Box<[u16; z3_core::VRAM_WORDS]>,
    vram_snapshot: Box<[u16; z3_core::VRAM_WORDS]>,
    pub last_sprite_oam: crate::sprite::PrepOamCoordsRet,
    pub ending_sprite_oam: crate::sprite::PrepOamCoordsRet,
    pub last_bank1_spawn: Option<(usize, crate::sprite::SpriteSpawnInfo)>,
    pub last_bank2a_spawn: Option<(usize, crate::sprite::SpriteSpawnInfo)>,
    pub enhanced_features0: u32,
    pub audio_port0: u8,
    pub pending_mirror_hdma: Option<overworld::MirrorHdmaSetup>,
    pub music_playback_state: (u8, u8, u8),
    pub music_unk1: u8,
    pub music_area: u8,
    pub music_entrance: u16,
    pub pending_ppu_writes: Vec<(u8, u8)>,
    pub pending_load_gfx_hdma: Vec<(u16, u8, u8)>,
    pub pending_host_actions: Vec<NativeHostAction>,
}

impl<'assets> NativeGameServices<'assets> {
    #[must_use]
    pub fn new(assets: &'assets RuntimeAssets<'assets>) -> Self {
        Self {
            assets,
            scratch_vram: Box::new([0; z3_core::VRAM_WORDS]),
            vram_snapshot: Box::new([0; z3_core::VRAM_WORDS]),
            last_sprite_oam: crate::sprite::PrepOamCoordsRet::default(),
            ending_sprite_oam: crate::sprite::PrepOamCoordsRet::default(),
            last_bank1_spawn: None,
            last_bank2a_spawn: None,
            enhanced_features0: 0,
            audio_port0: 0,
            pending_mirror_hdma: None,
            music_playback_state: (crate::audio::MSU_STATE_IDLE, 0, 0),
            music_unk1: 0,
            music_area: 0,
            music_entrance: 0,
            pending_ppu_writes: Vec::new(),
            pending_load_gfx_hdma: Vec::new(),
            pending_host_actions: Vec::new(),
        }
    }
}

impl FrameResourceHooks for NativeGameServices<'_> {
    fn sync_vram(&mut self, vram: &[u16; z3_core::VRAM_WORDS]) {
        self.scratch_vram.copy_from_slice(vram);
        self.vram_snapshot.copy_from_slice(vram);
    }

    fn set_audio_port0(&mut self, value: u8) {
        self.audio_port0 = value;
    }

    fn take_mirror_hdma_setup(&mut self) -> Option<overworld::MirrorHdmaSetup> {
        self.pending_mirror_hdma.take()
    }

    fn set_music_playback_state(
        &mut self,
        state: u8,
        enabled: u8,
        actual_track: u8,
        music_unk1: u8,
        area: u8,
        entrance: u16,
    ) {
        self.music_playback_state = (state, enabled, actual_track);
        self.music_unk1 = music_unk1;
        self.music_area = area;
        self.music_entrance = entrance;
    }

    fn take_load_gfx_device_commands(&mut self) -> LoadGfxDeviceCommands {
        let mut vram_spans = Vec::new();
        let mut index = 0;
        while index < z3_core::VRAM_WORDS {
            if self.scratch_vram[index] == self.vram_snapshot[index] {
                index += 1;
                continue;
            }
            let start = index;
            while index < z3_core::VRAM_WORDS
                && self.scratch_vram[index] != self.vram_snapshot[index]
            {
                index += 1;
            }
            vram_spans.push(VramWriteSpan {
                start,
                words: self.scratch_vram[start..index].to_vec(),
            });
        }
        self.vram_snapshot
            .copy_from_slice(self.scratch_vram.as_slice());
        LoadGfxDeviceCommands {
            ppu_writes: std::mem::take(&mut self.pending_ppu_writes),
            hdma_setups: std::mem::take(&mut self.pending_load_gfx_hdma),
            vram_spans,
        }
    }

    fn take_host_actions(&mut self) -> Vec<NativeHostAction> {
        std::mem::take(&mut self.pending_host_actions)
    }
}

/// Concrete owner of the resources shared by the top-level module state
/// machines. This implements [`MainModuleRuntime`], so it can be passed
/// directly to [`module_main_routing`] from the frame loop.
pub struct IntegratedGame<'a, 'assets, S> {
    pub sram: &'a mut [u8],
    pub dma: &'a mut z3_snes::dma::Dma,
    pub ppu: &'a mut Ppu,
    pub audio: &'a mut crate::audio::AudioRuntime,
    pub messaging_assets: &'assets MessagingAssets<'assets>,
    pub overworld_assets: &'assets RuntimeAssets<'assets>,
    pub enhanced_features0: u32,
    pub services: &'a mut S,
}

struct FileSelectAdapter<'a, 'assets, S> {
    ppu: &'a mut Ppu,
    assets: &'assets RuntimeAssets<'assets>,
    services: &'a mut S,
}

struct FileSelectAdapterGfxHooks<'a> {
    sram: &'a [u8],
}

impl crate::load_gfx::LoadGfxHooks for FileSelectAdapterGfxHooks<'_> {
    fn file_select_gear(&mut self, slot: usize) -> (u8, u8, u8, u8) {
        let save = slot * 0x500;
        (
            self.sram[save + 0x35b],
            self.sram[save + 0x354],
            self.sram[save + 0x359],
            self.sram[save + 0x35a],
        )
    }
}

impl<S: GameServices> FileSelectRuntime for FileSelectAdapter<'_, '_, S> {
    fn force_blank(&mut self, ram: &mut Ram<'_>) {
        crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: self.services,
        }
        .enable_force_blank();
    }

    fn erase_triforce_tilemaps(&mut self, ram: &mut Ram<'_>) {
        crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: self.services,
        }
        .erase_tile_maps_triforce();
    }

    fn load_file_select_palettes(&mut self, ram: &mut Ram<'_>, sram: &[u8]) {
        crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: &mut FileSelectAdapterGfxHooks { sram },
        }
        .palette_load_for_file_select();
    }

    fn load_file_select_graphics(&mut self, ram: &mut Ram<'_>) {
        crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: self.services,
        }
        .load_file_select_graphics();
        crate::overworld::decompress_enemy_damage_subclasses(ram, self.assets);
    }

    fn copy_selected_save_to_wram(&mut self, ram: &mut Ram<'_>, sram: &[u8], slot: usize) {
        self.services.copy_selected_save_to_wram(ram, sram, slot);
    }

    fn persist_sram(&mut self, sram: &[u8]) {
        self.services.persist_sram(sram);
    }

    fn initialize_file_select_graphics(&mut self, ram: &mut Ram<'_>) {
        let mut gfx = crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: self.services,
        };
        gfx.palette_load_dungeon_set();
        gfx.palette_load_owbg3();
        gfx.palette_load_hud();
        gfx.load_default_graphics();
        gfx.initialize_tilesets();
    }
}

struct EndingAdapter<'a, 'assets, 'messages, S> {
    sram: &'a mut [u8],
    dma: &'a mut z3_snes::dma::Dma,
    ppu: &'a mut Ppu,
    assets: &'assets RuntimeAssets<'assets>,
    messaging_assets: &'messages MessagingAssets<'messages>,
    services: &'a mut S,
}

impl<S: GameServices> EndingRuntime for EndingAdapter<'_, '_, '_, S> {
    fn call(&mut self, call: EndingCall, ram: &mut Ram<'_>) {
        match call {
            EndingCall::EnableForceBlank
            | EndingCall::EraseTileMapsNormal
            | EndingCall::EraseTileMapsTriforce
            | EndingCall::GraphicsLoadChrHalfSlot
            | EndingCall::InitializeTilesets
            | EndingCall::LoadCommonSprites
            | EndingCall::LoadDefaultGraphics
            | EndingCall::LoadFollowerGraphics
            | EndingCall::LoadItemGfxIntoWram4BppBuffer
            | EndingCall::OverworldLoadAllPalettes
            | EndingCall::OverworldLoadScreensPaletteSet
            | EndingCall::OverworldCopyPalettesToCache
            | EndingCall::SpecialOverworldCopyPalettesToCache
            | EndingCall::OverworldLoadPalettesInner
            | EndingCall::PaletteFilterBlindingWhiteTriforce
            | EndingCall::PaletteFilterSp5F
            | EndingCall::PaletteFilterWishPondsInner
            | EndingCall::PaletteFadeIntro2
            | EndingCall::PaletteFadeIntroOneStep
            | EndingCall::PaletteLoadHud
            | EndingCall::PaletteLoadOwbg2
            | EndingCall::PaletteRevertTranslucencySwap
            | EndingCall::PaletteSetOwBgColor
            | EndingCall::SetBackdropcolorBlack
            | EndingCall::TransferFontToVram => {
                let mut gfx = crate::load_gfx::LoadGfx {
                    wram: ram.bytes_mut(),
                    vram: &mut self.ppu.vram,
                    assets: self.assets,
                    hooks: self.services,
                };
                match call {
                    EndingCall::EnableForceBlank => gfx.enable_force_blank(),
                    EndingCall::EraseTileMapsNormal => gfx.erase_tile_maps_normal(),
                    EndingCall::EraseTileMapsTriforce => gfx.erase_tile_maps_triforce(),
                    EndingCall::GraphicsLoadChrHalfSlot => gfx.graphics_load_chr_half_slot(),
                    EndingCall::InitializeTilesets => gfx.initialize_tilesets(),
                    EndingCall::LoadCommonSprites => gfx.load_common_sprites(),
                    EndingCall::LoadDefaultGraphics => gfx.load_default_graphics(),
                    EndingCall::LoadFollowerGraphics => gfx.load_follower_graphics(),
                    EndingCall::LoadItemGfxIntoWram4BppBuffer => {
                        gfx.load_item_gfx_into_wram_4bpp_buffer();
                    }
                    EndingCall::OverworldLoadAllPalettes => gfx.overworld_load_all_palettes(),
                    EndingCall::OverworldLoadScreensPaletteSet => {
                        gfx.overworld_load_screens_palette_set();
                    }
                    EndingCall::OverworldCopyPalettesToCache => {
                        gfx.overworld_copy_palettes_to_cache()
                    }
                    EndingCall::SpecialOverworldCopyPalettesToCache => {
                        gfx.special_overworld_copy_palettes_to_cache();
                    }
                    EndingCall::OverworldLoadPalettesInner => gfx.overworld_load_palettes_inner(),
                    EndingCall::PaletteFilterBlindingWhiteTriforce => {
                        gfx.palette_filter_blinding_white_triforce();
                    }
                    EndingCall::PaletteFilterSp5F => gfx.palette_filter_sp5f(),
                    EndingCall::PaletteFilterWishPondsInner => {
                        gfx.palette_filter_wish_ponds_inner()
                    }
                    EndingCall::PaletteFadeIntro2 => gfx.palette_fade_intro_2(),
                    EndingCall::PaletteFadeIntroOneStep => gfx.palette_fade_intro_one_step(),
                    EndingCall::PaletteLoadHud => gfx.palette_load_hud(),
                    EndingCall::PaletteLoadOwbg2 => gfx.palette_load_owbg2(),
                    EndingCall::PaletteRevertTranslucencySwap => {
                        gfx.palette_revert_translucency_swap()
                    }
                    EndingCall::PaletteSetOwBgColor => gfx.palette_set_ow_bg_color(),
                    EndingCall::SetBackdropcolorBlack => gfx.palette_bg_and_fixed_color_black(),
                    EndingCall::TransferFontToVram => gfx.transfer_font_to_vram(),
                    _ => unreachable!(),
                }
                if call == EndingCall::EraseTileMapsNormal {
                    // Dungeon loading can write a tile equal to the pre-erase
                    // scratch value. Rebase its diff on the erased live VRAM.
                    self.services.sync_vram(&self.ppu.vram);
                }
            }
            EndingCall::LoadOwMusicIfNeeded => {
                Dungeon::new(ram.bytes_mut(), self.services).load_ow_music_if_needed();
            }
            EndingCall::TextGenerateMessagePointers => {
                let mut pointer = 0x1c_8000_u32;
                for index in 0..398 {
                    if index == 359 {
                        pointer = 0x0e_df40;
                    }
                    let destination = 0x171c0 + index * 3;
                    ram.write_u16(destination, pointer as u16);
                    ram.write_u8(destination + 2, (pointer >> 16) as u8);
                    if let Some(message) = self.messaging_assets.messages.get(index) {
                        pointer = pointer.wrapping_add(message.len() as u32 + 1);
                    } else {
                        assert_eq!(index, 397, "dialogue asset missing pointer source");
                    }
                }
            }
            EndingCall::RenderText => Messaging {
                ram,
                sram: self.sram,
                assets: self.messaging_assets,
                runtime: self.services,
            }
            .render_text(),
            EndingCall::BirdTravelFinishDoit => Messaging {
                ram,
                sram: self.sram,
                assets: self.messaging_assets,
                runtime: self.services,
            }
            .bird_travel_finish_doit(),
            EndingCall::FluteMenuLoadSelectedScreen => Messaging {
                ram,
                sram: self.sram,
                assets: self.messaging_assets,
                runtime: self.services,
            }
            .flute_menu_load_selected_screen(),
            EndingCall::LoadCreditsSongs => crate::misc::load_credits_songs(ram, self.services),
            EndingCall::LoadOverworldFromDungeon => {
                crate::overworld::load_overworld_from_dungeon(ram, self.assets, |ram| {
                    crate::overworld::load_gfx_and_screen_size(ram, self.assets)
                });
            }
            EndingCall::OverworldEnterSpecialArea => {
                let ppu = std::cell::RefCell::new(&mut *self.ppu);
                let services = std::cell::RefCell::new(&mut *self.services);
                crate::overworld::enter_special_area(
                    ram,
                    self.assets,
                    |ram| {
                        crate::overworld::load_overworld_from_dungeon(ram, self.assets, |ram| {
                            crate::overworld::load_gfx_and_screen_size(ram, self.assets)
                        });
                    },
                    |ram, bg, sprites| {
                        let mut ppu = ppu.borrow_mut();
                        let mut services = services.borrow_mut();
                        crate::load_gfx::LoadGfx {
                            wram: ram.bytes_mut(),
                            vram: &mut ppu.vram,
                            assets: self.assets,
                            hooks: &mut **services,
                        }
                        .overworld_load_palettes(bg, sprites);
                    },
                    |ram| {
                        let mut ppu = ppu.borrow_mut();
                        let mut services = services.borrow_mut();
                        crate::load_gfx::LoadGfx {
                            wram: ram.bytes_mut(),
                            vram: &mut ppu.vram,
                            assets: self.assets,
                            hooks: &mut **services,
                        }
                        .palette_special_ow();
                    },
                );
            }
            EndingCall::SpriteMain => self.services.run_sprite_main(ram),
            EndingCall::LinkOamMain => self.services.run_link_oam_main(ram),
            EndingCall::ApplyPaletteFilterBounce
            | EndingCall::CallForDuckIndoors
            | EndingCall::ConditionalMosaicControl
            | EndingCall::DungeonHandleLayerEffect
            | EndingCall::DungeonLoadAndDrawRoom
            | EndingCall::DungeonLoadEntrance
            | EndingCall::DungeonLoadPalettes
            | EndingCall::HudRebuildIndoor
            | EndingCall::LinkHandleMovingAnimationFullLongEntry
            | EndingCall::LinkHandleVelocity
            | EndingCall::LinkResetPropertiesA
            | EndingCall::MainShowTextMessage
            | EndingCall::Module0802LoadAndAdvance
            | EndingCall::OverworldHandleMapScroll
            | EndingCall::OverworldLoadAndBuildScreen
            | EndingCall::OverworldLoadOverlayAndMap
            | EndingCall::OverworldLoadOverlays2
            | EndingCall::OverworldSetFixedColAndScroll
            | EndingCall::PaletteAnimGetMasterSword2
            | EndingCall::ResetTransitionPropsAndAdvanceResetInterface
            | EndingCall::SaveDungeonKeys
            | EndingCall::SpriteSpawnBatCrashCutscene
            | EndingCall::TriforceRoomLinkApproachTriforce => {
                EndingRuntime::call(self.services, call, ram)
            }
        }
    }
    fn ppu_write(&mut self, register: u16, value: u8) {
        self.ppu.write(register as u8, value);
    }
    fn sprite_reset(&mut self, ram: &mut Ram<'_>, k: usize) {
        EndingRuntime::sprite_reset(self.services, ram, k);
    }
    fn sprite_set_x(&mut self, ram: &mut Ram<'_>, k: usize, x: u16) {
        EndingRuntime::sprite_set_x(self.services, ram, k, x);
    }
    fn sprite_set_y(&mut self, ram: &mut Ram<'_>, k: usize, y: u16) {
        EndingRuntime::sprite_set_y(self.services, ram, k, y);
    }
    fn sprite_move_xy(&mut self, ram: &mut Ram<'_>, k: usize) {
        EndingRuntime::sprite_move_xy(self.services, ram, k);
    }
    fn sprite_active_main(&mut self, ram: &mut Ram<'_>, k: usize) {
        EndingRuntime::sprite_active_main(self.services, ram, k);
    }
    fn sprite_draw_shadow(&mut self, ram: &mut Ram<'_>, k: usize) {
        EndingRuntime::sprite_draw_shadow(self.services, ram, k);
    }
    fn oam_allocate_region_a(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        EndingRuntime::oam_allocate_region_a(self.services, ram, bytes);
    }
    fn sprite_get_16_bit_coords(&mut self, ram: &mut Ram<'_>, k: usize) {
        EndingRuntime::sprite_get_16_bit_coords(self.services, ram, k);
    }
    fn sprite_draw_multiple(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        entries: &[crate::ending::DrawMultipleData],
    ) {
        EndingRuntime::sprite_draw_multiple(self.services, ram, k, entries);
    }
    fn ending_map_data(&self) -> &[u16] {
        self.services.ending_map_data()
    }
    fn ending_credits_text(&self) -> &[u8] {
        self.services.ending_credits_text()
    }
    fn ending_credits_offsets(&self) -> &[u16] {
        self.services.ending_credits_offsets()
    }
    fn ending_scene_text(&self) -> &[u8] {
        self.services.ending_scene_text()
    }
    fn ending_scene_offsets(&self) -> &[u16] {
        self.services.ending_scene_offsets()
    }
    fn random_number(&mut self, ram: &mut Ram<'_>) -> u8 {
        EndingRuntime::random_number(self.services, ram)
    }
    fn generated_end_sequence_15(&self, frame: u8) -> u8 {
        self.services.generated_end_sequence_15(frame)
    }
    fn decompress_animated_overworld_tiles(&mut self, ram: &mut Ram<'_>, set: u8) {
        crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: self.services,
        }
        .decompress_animated_overworld_tiles(set);
    }
    fn get_overworld_bg_palette(&mut self, screen: u8) -> u8 {
        overworld::get_overworld_bg_palette(self.assets, usize::from(screen))
    }
    fn overworld_load_palettes(&mut self, ram: &mut Ram<'_>, bg: u8, sprites: u8) {
        crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: self.services,
        }
        .overworld_load_palettes(bg, sprites);
    }
    fn decompress_animated_dungeon_tiles(&mut self, ram: &mut Ram<'_>, set: u8) {
        crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: self.services,
        }
        .decompress_animated_dungeon_tiles(set);
    }
    fn dungeon_palette_info(&mut self, index: u8) -> (u8, u8) {
        self.services.dungeon_palette_info(index)
    }
    fn health_after_death(&self, capacity_index: usize) -> u8 {
        self.services.health_after_death(capacity_index)
    }
    fn save_game_file(&mut self, ram: &mut Ram<'_>) {
        Messaging {
            ram,
            sram: self.sram,
            assets: self.messaging_assets,
            runtime: self.services,
        }
        .save_game_file();
    }
    fn credits_hdma_setup(&mut self, ram: &mut Ram<'_>) {
        crate::runtime::hdma_setup(self.dma, 0, 0x0e_bd53, 0x42, 0, 0x0f, 0);
        ram.write_u8(0x9b, 0x80);
    }
    fn vram_write_word(&mut self, word_address: usize, value: u16) {
        self.ppu.vram[word_address] = value;
    }
    fn dungeon_animated_tile_for_theme(&self, theme: u8) -> u8 {
        self.services.dungeon_animated_tile_for_theme(theme)
    }
    fn overworld_load_area_palettes_ex(&mut self, ram: &mut Ram<'_>, index: u8) {
        crate::load_gfx::LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut self.ppu.vram,
            assets: self.assets,
            hooks: self.services,
        }
        .overworld_load_area_palettes_ex(index);
    }
}

impl<S: GameServices> MiscHooks for IntegratedGame<'_, '_, S> {
    fn call(&mut self, ram: &mut Ram<'_>, call: misc::MiscCall) {
        match call {
            misc::MiscCall::HdmaSetup {
                addr6,
                addr7,
                transfer_unit,
                reg6,
                reg7,
                indirect_bank,
            } => crate::runtime::hdma_setup(
                self.dma,
                addr6,
                addr7,
                transfer_unit,
                reg6,
                reg7,
                indirect_bank,
            ),
            misc::MiscCall::RenderText => Messaging {
                ram,
                sram: self.sram,
                assets: self.messaging_assets,
                runtime: self.services,
            }
            .render_text(),
            misc::MiscCall::DeathFunc15(count_as_death) => Messaging {
                ram,
                sram: self.sram,
                assets: self.messaging_assets,
                runtime: self.services,
            }
            .death_func15(count_as_death),
            _ => MiscHooks::call(self.services, ram, call),
        }
    }

    fn route_main_module(&mut self, ram: &mut Ram<'_>, _module: u8) {
        module_main_routing(ram, self);
    }

    fn load_song_bank(&mut self, _ram: &mut Ram<'_>, bank: u8) {
        self.audio
            .load_song_bank(self.overworld_assets.bytes(usize::from(bank)))
            .expect("valid compiled song bank");
    }

    fn predefined_tile_word(&self, word_index: usize) -> u16 {
        self.services.predefined_tile_word(word_index)
    }

    fn refill_magic(&mut self, ram: &mut Ram<'_>) -> bool {
        self.services.refill_magic(ram)
    }

    fn refill_health(&mut self, ram: &mut Ram<'_>) -> bool {
        self.services.refill_health(ram)
    }

    fn add_ancilla(&mut self, ram: &mut Ram<'_>, kind: u8, arg: u8) -> i32 {
        self.services.add_ancilla(ram, kind, arg)
    }

    fn set_ancilla_xy(&mut self, ram: &mut Ram<'_>, ancilla: usize, x: u16, y: u16) {
        self.services.set_ancilla_xy(ram, ancilla, x, y);
    }

    fn sprite_x(&self, ram: &Ram<'_>, sprite: usize) -> u16 {
        self.services.sprite_x(ram, sprite)
    }

    fn dungeon_item_tile_action(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8 {
        self.services.dungeon_item_tile_action(ram, x, y)
    }

    fn overworld_item_tile_action(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8 {
        self.services.overworld_item_tile_action(ram, x, y)
    }
}

impl<S: GameServices> MainModuleRuntime for IntegratedGame<'_, '_, S> {
    fn module00_intro(&mut self, ram: &mut Ram<'_>) {
        let mut ending = EndingAdapter {
            sram: self.sram,
            dma: self.dma,
            ppu: self.ppu,
            assets: self.overworld_assets,
            messaging_assets: self.messaging_assets,
            services: self.services,
        };
        run_ending_module(ram, &mut ending, MainModule::Intro, self.enhanced_features0);
    }

    fn module01_file_select(&mut self, ram: &mut Ram<'_>) {
        let mut adapter = FileSelectAdapter {
            ppu: self.ppu,
            assets: self.overworld_assets,
            services: self.services,
        };
        run_file_select_module(ram, self.sram, &mut adapter, MainModule::FileSelect);
    }

    fn module02_copy_file(&mut self, ram: &mut Ram<'_>) {
        let mut adapter = FileSelectAdapter {
            ppu: self.ppu,
            assets: self.overworld_assets,
            services: self.services,
        };
        run_file_select_module(ram, self.sram, &mut adapter, MainModule::CopyFile);
    }

    fn module03_erase_file(&mut self, ram: &mut Ram<'_>) {
        let mut adapter = FileSelectAdapter {
            ppu: self.ppu,
            assets: self.overworld_assets,
            services: self.services,
        };
        run_file_select_module(ram, self.sram, &mut adapter, MainModule::EraseFile);
    }

    fn module04_name_file(&mut self, ram: &mut Ram<'_>) {
        let mut adapter = FileSelectAdapter {
            ppu: self.ppu,
            assets: self.overworld_assets,
            services: self.services,
        };
        run_file_select_module(ram, self.sram, &mut adapter, MainModule::NameFile);
    }

    fn module06_pre_dungeon(&mut self, ram: &mut Ram<'_>) {
        run_dungeon_module(ram, self.services, MainModule::PreDungeon);
    }

    fn module07_dungeon(&mut self, ram: &mut Ram<'_>) {
        run_dungeon_module(ram, self.services, MainModule::Dungeon);
    }

    fn module08_overworld_load(&mut self, ram: &mut Ram<'_>) {
        run_overworld_load(ram, self.overworld_assets, self.services);
    }

    fn module09_overworld(&mut self, ram: &mut Ram<'_>) {
        run_overworld(ram, self.overworld_assets, self.services);
    }

    fn module0a_special_overworld_load(&mut self, ram: &mut Ram<'_>) {
        run_overworld_load(ram, self.overworld_assets, self.services);
    }

    fn module0b_special_overworld(&mut self, ram: &mut Ram<'_>) {
        run_overworld(ram, self.overworld_assets, self.services);
    }

    fn module0e_interface(&mut self, ram: &mut Ram<'_>) {
        run_messaging_module(
            ram,
            self.sram,
            self.messaging_assets,
            self.services,
            MainModule::Interface,
        );
    }

    fn module0f_spotlight_close(&mut self, ram: &mut Ram<'_>) {
        run_spotlight_close(ram, self.services);
    }

    fn module10_spotlight_open(&mut self, ram: &mut Ram<'_>) {
        run_spotlight_open(ram, self.services);
    }

    fn module11_dungeon_falling_entrance(&mut self, ram: &mut Ram<'_>) {
        run_dungeon_module(ram, self.services, MainModule::DungeonFallingEntrance);
    }

    fn module12_game_over(&mut self, ram: &mut Ram<'_>) {
        run_messaging_module(
            ram,
            self.sram,
            self.messaging_assets,
            self.services,
            MainModule::GameOver,
        );
    }

    fn module14_attract(&mut self, ram: &mut Ram<'_>) {
        run_attract_module(
            ram,
            self.sram,
            self.dma,
            self.ppu,
            self.messaging_assets,
            self.services,
        );
    }

    fn module18_ganon_emerges(&mut self, ram: &mut Ram<'_>) {
        let mut ending = EndingAdapter {
            sram: self.sram,
            dma: self.dma,
            ppu: self.ppu,
            assets: self.overworld_assets,
            messaging_assets: self.messaging_assets,
            services: self.services,
        };
        run_ending_module(
            ram,
            &mut ending,
            MainModule::GanonEmerges,
            self.enhanced_features0,
        );
    }

    fn module19_triforce_room(&mut self, ram: &mut Ram<'_>) {
        let mut ending = EndingAdapter {
            sram: self.sram,
            dma: self.dma,
            ppu: self.ppu,
            assets: self.overworld_assets,
            messaging_assets: self.messaging_assets,
            services: self.services,
        };
        run_ending_module(
            ram,
            &mut ending,
            MainModule::TriforceRoom,
            self.enhanced_features0,
        );
    }

    fn module1a_credits(&mut self, ram: &mut Ram<'_>) {
        let mut ending = EndingAdapter {
            sram: self.sram,
            dma: self.dma,
            ppu: self.ppu,
            assets: self.overworld_assets,
            messaging_assets: self.messaging_assets,
            services: self.services,
        };
        run_ending_module(
            ram,
            &mut ending,
            MainModule::Credits,
            self.enhanced_features0,
        );
    }

    fn module1b_spawn_select(&mut self, ram: &mut Ram<'_>) {
        run_spawn_select(ram, self.services);
    }
}

/// Exact replacement for the external routing closure used by
/// `NativeRuntime::run_core_frame`.
pub fn module_main_routing<R: MainModuleRuntime>(ram: &mut Ram<'_>, runtime: &mut R) {
    let index = ram.read_u8(MAIN_MODULE);
    let module = MainModule::from_index(index)
        .unwrap_or_else(|| panic!("main module index {index:#04x} outside C routing table"));
    match module {
        MainModule::Intro => runtime.module00_intro(ram),
        MainModule::FileSelect => runtime.module01_file_select(ram),
        MainModule::CopyFile => runtime.module02_copy_file(ram),
        MainModule::EraseFile => runtime.module03_erase_file(ram),
        MainModule::NameFile => runtime.module04_name_file(ram),
        MainModule::LoadFile => misc::module05_load_file(ram, runtime),
        MainModule::PreDungeon => runtime.module06_pre_dungeon(ram),
        MainModule::Dungeon => runtime.module07_dungeon(ram),
        MainModule::OverworldLoad => runtime.module08_overworld_load(ram),
        MainModule::Overworld => runtime.module09_overworld(ram),
        MainModule::SpecialOverworldLoad => runtime.module0a_special_overworld_load(ram),
        MainModule::SpecialOverworld => runtime.module0b_special_overworld(ram),
        MainModule::Unknown0 => misc::module_unknown0(),
        MainModule::Unknown1 => misc::module_unknown1(),
        MainModule::Interface => runtime.module0e_interface(ram),
        MainModule::SpotlightClose => runtime.module0f_spotlight_close(ram),
        MainModule::SpotlightOpen => runtime.module10_spotlight_open(ram),
        MainModule::DungeonFallingEntrance => runtime.module11_dungeon_falling_entrance(ram),
        MainModule::GameOver => runtime.module12_game_over(ram),
        MainModule::BossVictoryPendant => misc::module13_boss_victory_pendant(ram, runtime),
        MainModule::Attract => runtime.module14_attract(ram),
        MainModule::MirrorWarpFromAga => misc::module15_mirror_warp_from_aga(ram, runtime),
        MainModule::BossVictoryCrystal => misc::module16_boss_victory_crystal(ram, runtime),
        MainModule::SaveAndQuit => misc::module17_save_and_quit(ram, runtime),
        MainModule::GanonEmerges => runtime.module18_ganon_emerges(ram),
        MainModule::TriforceRoom => runtime.module19_triforce_room(ram),
        MainModule::Credits => runtime.module1a_credits(ram),
        MainModule::SpawnSelect => runtime.module1b_spawn_select(ram),
    }
}

/// Invoke one of the four file-selection entries using its translated state
/// machine. Passing any other module is a caller error.
pub fn run_file_select_module<R: FileSelectRuntime>(
    ram: &mut Ram<'_>,
    sram: &mut [u8],
    runtime: &mut R,
    module: MainModule,
) {
    let mut select = FileSelect { ram, sram, runtime };
    match module {
        MainModule::FileSelect => select.run_file_select_module(),
        MainModule::CopyFile => select.run_copy_module(),
        MainModule::EraseFile => select.run_erase_module(),
        MainModule::NameFile => select.run_name_module(),
        _ => panic!("{module:?} is not a file-selection module"),
    }
}

/// Invoke a translated dungeon main-module entry using a safe reborrow of the
/// complete WRAM image.
pub fn run_dungeon_module<H: DungeonHooks>(ram: &mut Ram<'_>, hooks: &mut H, module: MainModule) {
    let mut dungeon = Dungeon::new(ram.bytes_mut(), hooks);
    match module {
        MainModule::PreDungeon => dungeon.module_pre_dungeon(),
        MainModule::Dungeon => dungeon.module07_dungeon(),
        MainModule::DungeonFallingEntrance => dungeon.module11_dungeon_falling_entrance(),
        _ => panic!("{module:?} is not a dungeon module"),
    }
}

/// Invoke the translated interface or game-over state machine.
pub fn run_messaging_module<R: MessagingRuntime>(
    ram: &mut Ram<'_>,
    sram: &mut [u8],
    assets: &MessagingAssets<'_>,
    runtime: &mut R,
    module: MainModule,
) {
    let mut messaging = Messaging {
        ram,
        sram,
        assets,
        runtime,
    };
    match module {
        MainModule::Interface => messaging.module0e_interface(),
        MainModule::GameOver => messaging.module_game_over(),
        _ => panic!("{module:?} is not a translated messaging module"),
    }
}

/// Invoke the translated intro/ending state machines.
pub fn run_ending_module<R: EndingRuntime>(
    ram: &mut Ram<'_>,
    runtime: &mut R,
    module: MainModule,
    enhanced_features0: u32,
) {
    let mut ending = Ending { ram, runtime };
    match module {
        MainModule::Intro => ending.module00_intro(enhanced_features0),
        MainModule::GanonEmerges => ending.module18_ganon_emerges(),
        MainModule::TriforceRoom => ending.module19_triforce_room(),
        MainModule::Credits => ending.module1a_credits(),
        _ => panic!("{module:?} is not an intro/ending module"),
    }
}

/// Invoke `Module14_Attract` through the translated implementation.
pub fn run_attract_module<R: AttractRuntime>(
    ram: &mut Ram<'_>,
    sram: &mut [u8],
    dma: &mut z3_snes::dma::Dma,
    ppu: &mut Ppu,
    messaging_assets: &MessagingAssets<'_>,
    runtime: &mut R,
) {
    Attract {
        ram,
        sram,
        dma,
        ppu,
        messaging_assets,
        runtime,
    }
    .module14_attract();
}

/// `Module1B_SpawnSelect`, retained here until the rest of its translation
/// unit exposes a native method. Its cross-unit calls already have exact
/// `MiscHooks` representations.
pub fn run_spawn_select<H: MiscHooks>(ram: &mut Ram<'_>, hooks: &mut H) {
    const NMI_LOAD_BG_FROM_VRAM: usize = 0x14;
    const SUBMODULE: usize = 0x11;
    const SUBSUBMODULE: usize = 0xb0;
    const CHOICE_IN_MULTISELECT: usize = 0x1ce8;
    const WHICH_STARTING_POINT: usize = 0xf3c8;
    const LOCATION_MENU_START_POS: [u8; 3] = [0, 1, 6];

    hooks.call(ram, misc::MiscCall::RenderText);
    if ram.read_u8(SUBMODULE) != 0 {
        return;
    }
    ram.write_u8(NMI_LOAD_BG_FROM_VRAM, 0);
    hooks.call(ram, misc::MiscCall::EnableForceBlank);
    hooks.call(ram, misc::MiscCall::EraseTileMaps);
    let saved_start = ram.read_u8(WHICH_STARTING_POINT);
    let choice = usize::from(ram.read_u8(CHOICE_IN_MULTISELECT));
    ram.write_u8(
        WHICH_STARTING_POINT,
        *LOCATION_MENU_START_POS
            .get(choice)
            .expect("spawn-select choice outside C table"),
    );
    ram.write_u8(SUBSUBMODULE, 0);
    misc::load_dungeon_room_rebuild_hud(ram, hooks);
    ram.write_u8(WHICH_STARTING_POINT, saved_start);
}

/// Connect `Module0F_SpotlightClose` to its translated overworld entrypoint.
pub fn run_spotlight_close<S: GameServices>(ram: &mut Ram<'_>, services: &mut S) {
    let services = RefCell::new(services);
    overworld::module0f_spotlight_close(
        ram,
        |ram| services.borrow_mut().run_sprite_main(ram),
        |ram| services.borrow_mut().prep_exit_with_spotlight(ram),
        |ram| {
            let mut borrowed = services.borrow_mut();
            let inner = RefCell::new(&mut **borrowed);
            overworld::spotlight_configure_table_and_control(
                ram,
                |ram| inner.borrow_mut().configure_spotlight_table(ram),
                |ram| inner.borrow_mut().open_spotlight_next(ram),
            );
        },
        |ram| SpotlightServices::link_handle_velocity(&mut **services.borrow_mut(), ram),
        |ram| {
            SpotlightServices::link_handle_moving_animation_full_long_entry(
                &mut **services.borrow_mut(),
                ram,
            );
        },
        |ram| services.borrow_mut().run_link_oam_main(ram),
    );
}

/// Connect `Module10_SpotlightOpen` to its translated overworld entrypoint.
pub fn run_spotlight_open<S: GameServices>(ram: &mut Ram<'_>, services: &mut S) {
    let services = RefCell::new(services);
    overworld::module10_spotlight_open(
        ram,
        |ram| services.borrow_mut().run_sprite_main(ram),
        |ram| SpotlightServices::spotlight_open(&mut **services.borrow_mut(), ram),
        |ram| services.borrow_mut().configure_spotlight_table(ram),
        |ram| services.borrow_mut().open_spotlight_next(ram),
        |ram| services.borrow_mut().run_link_oam_main(ram),
    );
}

/// Connect modules 8 and 10 to the translated pre-overworld dispatcher.
pub fn run_overworld_load<S: GameServices>(
    ram: &mut Ram<'_>,
    assets: &RuntimeAssets<'_>,
    services: &mut S,
) {
    overworld::module08_overworld_load(ram, |ram, submodule| {
        let submodule = PreOverworldSubmodule::from_index(submodule)
            .unwrap_or_else(|| panic!("pre-overworld submodule {submodule:#04x} outside C table"));
        match submodule {
            PreOverworldSubmodule::LoadProperties => {
                overworld::pre_overworld_load_properties(ram, assets, services);
            }
            PreOverworldSubmodule::LoadOverlays => {
                overworld::pre_overworld_load_overlays(ram, |ram| {
                    overworld::load_overlays_inner(ram, |ram| {
                        overworld::load_overworld_overlay(ram, assets);
                    });
                });
            }
            PreOverworldSubmodule::LoadAndAdvance => {
                overworld::module08_load_and_advance(ram, assets, |ram| {
                    services.overworld_call(ram, OverworldCall::DrawQuadrantsAndOverlays);
                });
            }
        }
    });
}

/// Connect modules 9 and 11 to the translated overworld frame wrapper. The
/// wrapper preserves its C ordering: submodule, scroll offsets, sprites,
/// restored offsets, Link OAM, HUD refill, then rain overlay.
pub fn run_overworld<S: GameServices>(
    ram: &mut Ram<'_>,
    assets: &RuntimeAssets<'_>,
    services: &mut S,
) {
    let services = RefCell::new(services);
    overworld::module09_overworld(
        ram,
        |ram| {
            let submodule = ram.read_u8(0x11);
            let submodule = OverworldSubmodule::from_index(submodule)
                .unwrap_or_else(|| panic!("overworld submodule {submodule:#04x} outside C table"));
            dispatch_overworld_submodule(ram, assets, &services, submodule);
        },
        |ram| services.borrow_mut().run_sprite_main(ram),
        |ram| services.borrow_mut().run_link_oam_main(ram),
        |ram| overworld::PreOverworldHooks::hud_refill_logic(&mut **services.borrow_mut(), ram),
    );
}

fn dispatch_overworld_submodule<S: GameServices>(
    ram: &mut Ram<'_>,
    assets: &RuntimeAssets<'_>,
    services: &RefCell<&mut S>,
    submodule: OverworldSubmodule,
) {
    use OverworldCall as C;
    use OverworldSubmodule as O;
    macro_rules! call {
        ($ram:expr, $op:expr) => {
            services.borrow_mut().overworld_call($ram, $op)
        };
    }
    match submodule {
        O::PlayerControl => {
            overworld::module09_player_control(ram, &mut **services.borrow_mut());
        }
        O::LoadAuxGfx => overworld::module09_load_aux_gfx(
            ram,
            |r| call!(r, C::LoadTransitionAuxGfx),
            |r| call!(r, C::PrepareTransitionAuxGfx),
        ),
        O::FinishTransitionGfx => overworld::finish_transition_gfx(ram),
        O::LoadNewMapAndGfx => overworld::module09_load_new_map_and_gfx(
            ram,
            assets,
            |r| call!(r, C::LoadEventOverlay),
            |r, axis, step| call!(r, C::TriggerMapLoadStripe(axis, step)),
            |r| call!(r, C::LoadNewSpriteGfxSet),
        ),
        O::LoadNewSprites => overworld::module09_load_new_sprites(ram, assets, |r| {
            call!(r, C::ReloadOverworldSpritesJustLoad);
        }),
        O::StartScrollTransition => overworld::start_scroll_transition(ram, assets),
        O::RunScrollTransition => overworld::run_scroll_transition(
            ram,
            assets,
            |r| call!(r, C::HandleMovingAnimation),
            |r| call!(r, C::IncrementalVramUpload),
            |r| call!(r, C::InitializeSpriteSlots),
        ),
        O::EaseOffScrollTransition => overworld::ease_off_scroll_transition(ram, assets, |r| {
            call!(r, C::DisableFollower);
        }),
        O::FinalizeEntry => overworld::finalize_entry_onto_screen(
            ram,
            assets,
            |r| call!(r, C::HandleMovingAnimation),
            overworld::operate_camera_scroll,
        ),
        O::OpenBigDoorFromExit => overworld::open_big_door_from_exit(ram, |r| {
            overworld::update_map_32x32_conditional(r, |r| {
                overworld::update_map_32x32(r, assets);
            });
        }),
        O::WalkFromExitFacingDown => overworld::walk_from_exit_facing_down(ram, assets, |r| {
            call!(r, C::HandleMovingAnimation);
        }),
        O::WalkFromExitFacingUp => overworld::walk_from_exit_facing_up(ram, |r| {
            call!(r, C::HandleMovingAnimation);
        }),
        O::OpenBigDoor => overworld::open_big_door(ram, |r| {
            overworld::update_map_32x32_conditional(r, |r| {
                overworld::update_map_32x32(r, assets);
            });
        }),
        O::StartMosaicTransition => overworld::start_mosaic_transition(
            ram,
            overworld::conditional_mosaic_control,
            |track| services.borrow_mut().is_playing_music_track(track),
            |r| call!(r, C::ResetTransitionPropsAndInterface),
            |r| call!(r, C::ApplyPaletteFilterBounce),
            |r, tile| call!(r, C::DecodeAnimatedSpriteTile(tile)),
            |r| {
                overworld::load_overworld_from_special_overworld(
                    r,
                    assets,
                    |r, bg, sprites| call!(r, C::LoadPalettes(bg, sprites)),
                    |r| call!(r, C::ApplySpecialOverworldPalette),
                    |r| call!(r, C::ResetSwimmingState),
                    |r| overworld::load_gfx_and_screen_size(r, assets),
                );
            },
        ),
        O::PreOverworldLoadOverlays => overworld::pre_overworld_load_overlays(ram, |r| {
            overworld::load_overlays_inner(r, |r| overworld::load_overworld_overlay(r, assets));
        }),
        O::FadeBackInFromMosaic => overworld::fade_back_in_from_mosaic(
            ram,
            assets,
            |r, bg, sprites| call!(r, C::LoadPalettes(bg, sprites)),
            |r| call!(r, C::LoadNewSpriteGfxSet),
            |r| call!(r, C::IncrementalVramUpload),
            |r| call!(r, C::ApplyPaletteFilterBounce),
            |track| services.borrow_mut().is_playing_music_track(track),
        ),
        O::EnterSpecialArea => overworld::overworld_func18(
            ram,
            |r| {
                overworld::enter_special_area(
                    r,
                    assets,
                    |r| {
                        overworld::load_overworld_from_dungeon(r, assets, |r| {
                            overworld::load_gfx_and_screen_size(r, assets);
                        });
                    },
                    |r, bg, sprites| call!(r, C::LoadPalettes(bg, sprites)),
                    |r| call!(r, C::ApplySpecialOverworldPalette),
                );
            },
            |r| {
                overworld::load_overlays(
                    r,
                    |r| call!(r, C::InitializeSpriteSlots),
                    |r| call!(r, C::ReloadOverworldSprites),
                    |r| {
                        overworld::load_overlays_inner(r, |r| {
                            overworld::load_overworld_overlay(r, assets)
                        })
                    },
                );
            },
        ),
        O::LoadSpecialAreaAndAdvance => overworld::overworld_func19(ram, |r| {
            overworld::module08_load_and_advance(r, assets, |r| {
                call!(r, C::DrawQuadrantsAndOverlays);
            });
        }),
        O::MosaicTransition => overworld::overworld_func1c(
            ram,
            |r| call!(r, C::LoadNewSpriteGfxSet),
            |r| call!(r, C::IncrementalVramUpload),
            |r| call!(r, C::ApplyPaletteFilterBounce),
        ),
        O::Assertion1d => overworld::overworld_func1d(),
        O::Assertion1e => overworld::overworld_func1e(),
        O::SpecialExit => overworld::overworld_func1f(
            ram,
            |r| call!(r, C::HandleMovingAnimation),
            overworld::operate_camera_scroll,
        ),
        O::LoadOverlays2 => {
            overworld::overworld_load_overlays2(ram, assets);
        }
        O::LoadOverlays => overworld::load_overlays(
            ram,
            |r| call!(r, C::InitializeSpriteSlots),
            |r| call!(r, C::ReloadOverworldSprites),
            |r| overworld::load_overlays_inner(r, |r| overworld::load_overworld_overlay(r, assets)),
        ),
        O::LoadAmbientOverlayFalse => {
            overworld::load_ambient_overlay_false(ram, assets, |r| {
                call!(r, C::DrawQuadrantsAndOverlays);
            });
        }
        O::Brighten => overworld::overworld_func22(ram),
        O::MirrorWarp => overworld::module09_mirror_warp(
            ram,
            |r| {
                overworld::initialize_mirror_hdma(r, |setup| {
                    services.borrow_mut().setup_mirror_hdma(setup);
                });
            },
            |r| call!(r, C::InitializeWhiteFilter),
            |r| overworld::load_gfx_and_screen_size(r, assets),
            |r| {
                overworld::mirror_warp_build_waving_hdma_table(r, |r| {
                    call!(r, C::RunAnimationSubmodules);
                });
            },
            |r| {
                overworld::mirror_warp_build_dewaving_hdma_table(
                    r,
                    |r| call!(r, C::RunAnimationSubmodules),
                    overworld::set_fixed_color_and_scroll,
                );
            },
            |r| {
                overworld::mirror_warp_finalize_and_load_destination(
                    r,
                    assets,
                    |r| {
                        // C's MirrorWarp_FinalizeAndLoadDestination reprograms
                        // HDMA channel 7 for the iris window before resetting
                        // its table. The service callback handles only the
                        // latter, so queue the hardware setup here as well.
                        services
                            .borrow_mut()
                            .setup_mirror_hdma(overworld::MirrorHdmaSetup {
                                channel6_address: 0,
                                channel7_address: 0xf2fb,
                                transfer_unit: 0x41,
                                channel6_register: 0,
                                channel7_register: 0x26,
                                indirect_bank: 0,
                            });
                        call!(r, C::ResetHdmaAndIris);
                    },
                    |r| call!(r, C::ReloadPreviouslyLoadedSheets),
                );
            },
        ),
        O::LoadAndBuildScreen => overworld::load_and_build_screen(ram, assets, |r| {
            call!(r, C::DrawQuadrantsAndOverlays);
        }),
        O::RecoverFromDrowning => overworld::module09_recover_from_drowning(
            ram,
            |r| {
                overworld::scroll_to_land(r, overworld::operate_camera_scroll, |r| {
                    overworld::handle_map_scroll(r, assets);
                });
            },
            |r| {
                overworld::recover_position_after_drowning(
                    r,
                    |r| call!(r, C::CacheQuadrantVisits),
                    |r| call!(r, C::ResetLinkState),
                    |r| call!(r, C::InitializeFollower),
                );
            },
        ),
        O::MasterSwordPalette => overworld::overworld_func2b(ram, |r| {
            call!(r, C::PaletteFilterBlindingWhite);
        }),
        O::WeathervaneExplosion => overworld::overworld_weathervane_explosion(),
        O::Whirlpool => {
            if ram.read_u8(0xb0) == 7 {
                overworld::module09_load_aux_gfx(
                    ram,
                    |r| call!(r, C::LoadTransitionAuxGfx),
                    |r| call!(r, C::PrepareTransitionAuxGfx),
                );
                ram.write_u8(0x11, ram.read_u8(0x11).wrapping_sub(1));
                ram.write_u8(0xb0, 8);
            } else if ram.read_u8(0xb0) == 8 {
                overworld::finish_transition_gfx(ram);
                ram.write_u8(0x13, 15);
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                ram.write_u8(0x11, ram.read_u8(0x11).wrapping_sub(1));
                ram.write_u8(0xb0, 9);
            } else if ram.read_u8(0xb0) == 9 {
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                ram.write_u16(0xaa8, 0);
                call!(ram, C::WhirlpoolReloadPalettes);
                call!(ram, C::LoadNewSpriteGfxSet);
                ram.write_u8(0x9e, 0x80);
                ram.write_u8(0x13, 15);
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                ram.write_u8(0xb0, 10);
            } else if ram.read_u8(0xb0) == 10 {
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                call!(ram, C::WhirlpoolRestoreRedGreen);
                if ram.read_u8(0xc007) != 0 {
                    call!(ram, C::WhirlpoolRestoreRedGreen);
                }
            } else if ram.read_u8(0xb0) == 11 {
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                call!(ram, C::IncrementalVramUpload);
                call!(ram, C::WhirlpoolRestoreBlue);
            } else if ram.read_u8(0xb0) == 12 {
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                ram.write_u8(0x31f, 144);
                call!(ram, C::ReloadPreviouslyLoadedSheets);
                ram.write_u8(0x9b, 0x80);
                ram.write_u8(
                    0x12d,
                    ram.read_u8(0x15b00 + usize::from(ram.read_u8(0x8a))) >> 4,
                );
                ram.write_u8(0x12c, if ram.read_u8(0xf3ca) != 0 { 9 } else { 2 });
                ram.write_u8(0x11, 0);
                ram.write_u8(0xb0, 0);
                ram.write_u8(0x200, 0);
                ram.write_u8(0x710, 0);
            } else if ram.read_u8(0xb0) == 5 {
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                call!(ram, C::LoadOverlayAndMap);
                ram.write_u8(0x17, 12);
                ram.write_u8(0x13, 15);
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                ram.write_u8(0xb0, 6);
            } else if ram.read_u8(0xb0) == 3 {
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                ram.write_u8(0x9e, 0x9f);
                ram.write_u16(0xaa8, 0);
                ram.write_u8(0xab2, 0);
                call!(ram, C::FindPartnerWhirlpoolExit);
                ram.write_u8(0xb2, 0);
                overworld::overworld_load_overlays2(ram, assets);
                ram.write_u8(0x11, ram.read_u8(0x11).wrapping_sub(1));
                ram.write_u8(0x17, 12);
                ram.write_u8(0x15, 0);
                ram.write_u8(0x9e, 0x80);
                ram.write_u8(0x13, 15);
                ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
                ram.write_u8(0xb0, 4);
            } else {
                overworld::module09_whirlpool(
                    ram,
                    |r| call!(r, C::PaletteFilterWhirlpoolBlue),
                    |r| call!(r, C::PaletteFilterIsolateWhirlpoolBlue),
                );
            }
        }
        O::FinishPyramidHole => overworld::overworld_func2f(ram, assets),
    }
}

/// Frame hooks that keep the PPU/SRAM borrows available to both module routing
/// and NMI. This removes the aliasing problem of capturing those resources in
/// `NativeRuntime::run_core_frame`'s external closure.
pub struct IntegratedFrameHooks<'a, 'assets, 'messages, S, LoadTrack> {
    pub sram: &'a mut [u8; SRAM_SIZE],
    pub dma: &'a mut z3_snes::dma::Dma,
    pub ppu: &'a mut Ppu,
    pub audio: &'a mut crate::audio::AudioRuntime,
    pub nmi_assets: crate::nmi::NmiAssets<'assets>,
    pub intro_song_bank: &'assets [u8],
    pub overworld_assets: &'assets RuntimeAssets<'assets>,
    pub messaging_assets: &'messages MessagingAssets<'messages>,
    pub enhanced_features0: u32,
    pub services: &'a mut S,
    pub load_pcm_track: LoadTrack,
}

impl<S: FrameResourceHooks, LoadTrack> IntegratedFrameHooks<'_, '_, '_, S, LoadTrack> {
    fn drain_service_device_commands(&mut self) {
        if let Some(setup) = self.services.take_mirror_hdma_setup() {
            crate::runtime::hdma_setup(
                self.dma,
                setup.channel6_address,
                setup.channel7_address,
                setup.transfer_unit,
                setup.channel6_register,
                setup.channel7_register,
                setup.indirect_bank,
            );
        }
        let commands = self.services.take_load_gfx_device_commands();
        for span in commands.vram_spans {
            self.ppu.vram[span.start..span.start + span.words.len()].copy_from_slice(&span.words);
        }
        for (register, value) in commands.ppu_writes {
            self.ppu.write(register, value);
        }
        for (table, table_bank, bbad) in commands.hdma_setups {
            let address = u32::from(table) | (u32::from(table_bank) << 16);
            crate::runtime::hdma_setup(self.dma, address, address, 0x41, bbad, bbad, 0);
        }
    }
}

impl<S: GameServices, LoadTrack> crate::runtime::FrameHooks
    for IntegratedFrameHooks<'_, '_, '_, S, LoadTrack>
where
    LoadTrack: FnMut(u8) -> Option<crate::audio::PcmTrack>,
{
    fn initialize(&mut self, ram: &mut Ram<'_>) {
        self.audio
            .load_song_bank(self.intro_song_bank)
            .expect("valid compiled intro song bank");
        crate::runtime::startup_initialize_memory(ram, self.sram);
        ram.write_u16(0xadc, 0xa680);
        ram.write_u16(0xae0, 0xb280);
        ram.write_u16(0xae2, 0xb2e0);
    }

    fn run_poly_loop(&mut self, ram: &mut Ram<'_>) {
        if ram.read_u8(0x1f00) != 0 && ram.read_u8(0x1f0c) == 0 {
            crate::poly::run_frame(ram);
            ram.write_u8(0x1f00, 0);
            ram.write_u8(0x1f0c, 0xff);
        }
    }

    fn module_main_routing(&mut self, ram: &mut Ram<'_>) {
        self.services.sync_vram(&self.ppu.vram);
        // The C gameplay code reads the recorder's APUI00 WRAM mirror, not the
        // live APU port. During replay the mirror is patched by the recording.
        self.services.set_audio_port0(ram.read_u8(0x648));
        let (state, enabled, actual_track) = self.audio.music_playback_state();
        self.services.set_music_playback_state(
            state,
            enabled,
            actual_track,
            ram.read_u8(0x130),
            ram.read_u16(0x40a) as u8,
            u16::from(ram.read_u8(0x10e)),
        );
        {
            let mut game = IntegratedGame {
                sram: self.sram,
                dma: self.dma,
                ppu: self.ppu,
                audio: self.audio,
                messaging_assets: self.messaging_assets,
                overworld_assets: self.overworld_assets,
                enhanced_features0: self.enhanced_features0,
                services: self.services,
            };
            module_main_routing(ram, &mut game);
        }
        self.drain_service_device_commands();
        let mut host_action_rounds = 0;
        loop {
            let actions = self.services.take_host_actions();
            if actions.is_empty() {
                break;
            }
            host_action_rounds += 1;
            assert!(
                host_action_rounds < 256,
                "host action callback cycle: {actions:?}"
            );
            for action in actions {
                // Host actions may call back into production services. Rebase
                // their scratch view on the latest live PPU, then commit every
                // queued write before the next action can observe the device.
                self.services.sync_vram(&self.ppu.vram);
                match action {
                    NativeHostAction::HdmaSetup {
                        addr6,
                        addr7,
                        transfer_unit,
                        reg6,
                        reg7,
                        indirect_bank,
                    } => crate::runtime::hdma_setup(
                        self.dma,
                        addr6,
                        addr7,
                        transfer_unit,
                        reg6,
                        reg7,
                        indirect_bank,
                    ),
                    NativeHostAction::RenderText => Messaging {
                        ram,
                        sram: self.sram,
                        assets: self.messaging_assets,
                        runtime: self.services,
                    }
                    .render_text(),
                    NativeHostAction::DeathFunc15(count_as_death) => Messaging {
                        ram,
                        sram: self.sram,
                        assets: self.messaging_assets,
                        runtime: self.services,
                    }
                    .death_func15(count_as_death),
                    NativeHostAction::RouteMainModule => {
                        let mut game = IntegratedGame {
                            sram: self.sram,
                            dma: self.dma,
                            ppu: self.ppu,
                            audio: self.audio,
                            messaging_assets: self.messaging_assets,
                            overworld_assets: self.overworld_assets,
                            enhanced_features0: self.enhanced_features0,
                            services: self.services,
                        };
                        module_main_routing(ram, &mut game);
                    }
                    NativeHostAction::LoadSongBank(bank) => {
                        self.audio
                            .load_song_bank(self.overworld_assets.bytes(usize::from(bank)))
                            .expect("valid compiled song bank");
                    }
                    NativeHostAction::EndingExternal(call) => {
                        let mut ending = EndingAdapter {
                            sram: self.sram,
                            dma: self.dma,
                            ppu: self.ppu,
                            assets: self.overworld_assets,
                            messaging_assets: self.messaging_assets,
                            services: self.services,
                        };
                        ending.call(call, ram);
                    }
                    NativeHostAction::WorldMapLoadLight => {
                        for word in &mut self.ppu.vram[..0x4000] {
                            *word = (*word & 0xff00) | 0x00ef;
                        }
                        let map_graphics = self.overworld_assets.bytes(66);
                        for (word, &high) in self.ppu.vram[..0x4000].iter_mut().zip(map_graphics) {
                            *word = (*word & 0x00ff) | (u16::from(high) << 8);
                        }
                        ram.write_u8(0x1c, 0x11);
                        ram.write_u8(0x1d, 0);
                        setup_world_map_hdma(ram, self.dma);
                        crate::load_gfx::LoadGfx {
                            wram: ram.bytes_mut(),
                            vram: &mut self.ppu.vram,
                            assets: self.overworld_assets,
                            hooks: self.services,
                        }
                        .load_overworld_map_palette();
                        crate::load_gfx::LoadGfx {
                            wram: ram.bytes_mut(),
                            vram: &mut self.ppu.vram,
                            assets: self.overworld_assets,
                            hooks: self.services,
                        }
                        .load_actual_gear_palettes();
                        ram.write_u8(0x15, ram.read_u8(0x15).wrapping_add(1));
                        ram.write_u8(0x17, 7);
                        ram.write_u8(0x13, 0);
                        ram.write_u8(0x0710, ram.read_u8(0x0710).wrapping_add(1));
                        ram.write_u8(0x0200, ram.read_u8(0x0200).wrapping_add(1));
                    }
                    NativeHostAction::TransferMode7Characters => {
                        let map_graphics = self.overworld_assets.bytes(66);
                        for (word, &high) in self.ppu.vram[..0x4000].iter_mut().zip(map_graphics) {
                            *word = (*word & 0x00ff) | (u16::from(high) << 8);
                        }
                    }
                    NativeHostAction::WorldMapSetupHdma => {
                        setup_world_map_hdma_transfer(ram, self.dma)
                    }
                }
                self.drain_service_device_commands();
            }
        }
    }

    fn prepare_sprites_for_nmi(&mut self, ram: &mut Ram<'_>) {
        misc::nmi_prepare_sprites(ram);
    }

    fn interrupt_nmi(&mut self, ram: &mut Ram<'_>, input: u16) {
        let mut audio = crate::audio::NmiAudioAdapter {
            runtime: self.audio,
            area: ram.read_u16(0x40a) as u8,
            entrance: u16::from(ram.read_u8(0x10e)),
            music_unk1: ram.read_u8(0x130),
            last_music_control: ram.read_u8(0x133),
            misc_bug_fixes: ram.read_u32(0x64c) & 4096 != 0,
            load_pcm_track: &mut self.load_pcm_track,
        };
        crate::nmi::interrupt_nmi(ram, self.ppu, &self.nmi_assets, &mut audio, input)
            .expect("valid native NMI state");
    }
}

pub(crate) fn setup_world_map_registers(ram: &mut Ram<'_>) {
    ram.write_u16(0xe0, 0x80);
    ram.write_u16(0xe6, 0xc8);
    ram.write_u16(0x63a, 0x1c9);
    ram.write_u16(0x638, 0x100);
    for address in [0x96, 0x97, 0x98, 0x1e, 0x1f] {
        ram.write_u8(address, 0);
    }
    if ram.read_u8(0x10) != 20 && ram.read_u8(0x11) != 10 {
        ram.write_u8(0x635, 4);
        ram.write_u8(0x637, 12);
        ram.write_u8(0x636, 1);
        let y = ((ram.read_u16(0xc108) >> 4).wrapping_sub(0x48)) & !1;
        ram.write_u16(0xe6, y);
        ram.write_u16(0x63a, y.wrapping_add(0x100));
        let x = (ram.read_u16(0xc10a) >> 4).wrapping_sub(0x80) as i16;
        let scaled = ((5 * x.unsigned_abs()) >> 1) as i16;
        ram.write_u16(
            0xe0,
            (if x < 0 { -scaled } else { scaled } as u16).wrapping_add(0x80) & !1,
        );
    } else if ram.read_u8(0x10) != 20 {
        ram.write_u8(0x635, 4);
        ram.write_u8(0x637, 33);
        ram.write_u8(0x636, 0);
    }
    ram.write_u8(0x9b, 0xc0);
}

pub(crate) fn setup_world_map_hdma_transfer(ram: &Ram<'_>, dma: &mut z3_snes::dma::Dma) {
    let address = if ram.read_u8(0x10) == 20 {
        0x0a_bddd
    } else if ram.read_u8(0x11) == 10 || ram.read_u8(0x636) == 0 {
        0x0a_bdcf
    } else {
        0x0a_bdd6
    };
    let flags = if ram.read_u8(0x10) == 20 { 0 } else { 10 };
    crate::runtime::hdma_setup(dma, address, address, 0x42, 0x1b, 0x1e, flags);
}

fn setup_world_map_hdma(ram: &mut Ram<'_>, dma: &mut z3_snes::dma::Dma) {
    setup_world_map_registers(ram);
    setup_world_map_hdma_transfer(ram, dma);
}

/// Host-facing native frame entry with the concrete game router installed.
/// Its recorder, feature-patch, scheduler, and audio ordering matches
/// `NativeRuntime::run_core_frame`.
#[allow(clippy::too_many_arguments)]
pub fn run_integrated_core_frame<'assets, 'messages, S, LoadTrack>(
    native: &mut crate::runtime::NativeRuntime,
    input: u16,
    nmi_assets: crate::nmi::NmiAssets<'assets>,
    intro_song_bank: &'assets [u8],
    overworld_assets: &'assets RuntimeAssets<'assets>,
    messaging_assets: &'messages MessagingAssets<'messages>,
    services: &mut S,
    load_pcm_track: LoadTrack,
) -> Result<bool, crate::runtime::ReplayError>
where
    S: GameServices,
    LoadTrack: FnMut(u8) -> Option<crate::audio::PcmTrack>,
{
    const APUI00_MIRROR: usize = 0x648;
    const BUGS_FIXED: usize = 0x64a;
    const ENHANCED_FEATURES0: usize = 0x64c;
    const ANIMATED_TILE_DATA_SRC: usize = 0xadc;

    let input = crate::runtime::normalize_inputs(input);
    native.debug_frame_counter = native.debug_frame_counter.wrapping_add(1);
    let replaying = native.recorder.is_replaying();
    let mut ram = Ram::new(&mut native.wram);
    let input = if replaying {
        native.recorder.read_next_replay_state(&mut ram)?
    } else {
        native.recorder.record(input);
        let music_playing = u8::from(native.audio.is_music_playing());
        if music_playing != ram.read_u8(APUI00_MIRROR) {
            ram.write_u8(APUI00_MIRROR, music_playing);
            native
                .recorder
                .record_patch(APUI00_MIRROR as u32, &[music_playing]);
        }
        if ram.read_u16(ANIMATED_TILE_DATA_SRC) != 0 {
            if ram.read_u8(BUGS_FIXED) < 1 {
                ram.write_u8(BUGS_FIXED, 1);
                native.recorder.record_patch(BUGS_FIXED as u32, &[1]);
            }
            if ram.read_u32(ENHANCED_FEATURES0) != native.wanted_features {
                ram.write_u32(ENHANCED_FEATURES0, native.wanted_features);
                native.recorder.record_patch(
                    ENHANCED_FEATURES0 as u32,
                    &native.wanted_features.to_le_bytes(),
                );
            }
        }
        input
    };
    let vertical_irq_increment = ram.read_u8(0xff);
    let run_what = crate::runtime::determine_run_what(&mut ram, vertical_irq_increment);
    let mut hooks = IntegratedFrameHooks {
        sram: &mut native.sram,
        dma: &mut native.dma,
        ppu: &mut native.ppu,
        audio: &mut native.audio,
        nmi_assets,
        intro_song_bank,
        overworld_assets,
        messaging_assets,
        enhanced_features0: native.wanted_features,
        services,
        load_pcm_track,
    };
    crate::runtime::run_frame_internal(&mut ram, &mut hooks, input, run_what);
    drop(hooks);
    native.audio.push_apu_state();
    Ok(replaying)
}

/// Oracle-facing equivalent of C `ZeldaRunFrameInternal`: run exactly the
/// requested scheduler bits without the host recorder's feature patches.
/// This is the correct entrypoint for byte-level differential streams.
#[allow(clippy::too_many_arguments)]
pub fn step_integrated_raw<'assets, 'messages, S, LoadTrack>(
    native: &mut crate::runtime::NativeRuntime,
    input: u16,
    run_what: u8,
    nmi_assets: crate::nmi::NmiAssets<'assets>,
    intro_song_bank: &'assets [u8],
    overworld_assets: &'assets RuntimeAssets<'assets>,
    messaging_assets: &'messages MessagingAssets<'messages>,
    services: &mut S,
    load_pcm_track: LoadTrack,
) where
    S: GameServices,
    LoadTrack: FnMut(u8) -> Option<crate::audio::PcmTrack>,
{
    let mut hooks = IntegratedFrameHooks {
        sram: &mut native.sram,
        dma: &mut native.dma,
        ppu: &mut native.ppu,
        audio: &mut native.audio,
        nmi_assets,
        intro_song_bank,
        overworld_assets,
        messaging_assets,
        enhanced_features0: native.wanted_features,
        services,
        load_pcm_track,
    };
    crate::runtime::run_frame_internal(
        &mut Ram::new(&mut native.wram),
        &mut hooks,
        crate::runtime::normalize_inputs(input),
        run_what,
    );
}

/// Runner-facing extension that installs the complete Rust module router on a
/// [`NativeRuntime`](crate::runtime::NativeRuntime).
pub trait NativeRuntimeIntegratedExt {
    #[allow(clippy::too_many_arguments)]
    fn run_integrated_frame<'assets, 'messages, S, LoadTrack>(
        &mut self,
        input: u16,
        nmi_assets: crate::nmi::NmiAssets<'assets>,
        intro_song_bank: &'assets [u8],
        overworld_assets: &'assets RuntimeAssets<'assets>,
        messaging_assets: &'messages MessagingAssets<'messages>,
        services: &mut S,
        load_pcm_track: LoadTrack,
    ) -> Result<bool, crate::runtime::ReplayError>
    where
        S: GameServices,
        LoadTrack: FnMut(u8) -> Option<crate::audio::PcmTrack>;
}

impl NativeRuntimeIntegratedExt for crate::runtime::NativeRuntime {
    fn run_integrated_frame<'assets, 'messages, S, LoadTrack>(
        &mut self,
        input: u16,
        nmi_assets: crate::nmi::NmiAssets<'assets>,
        intro_song_bank: &'assets [u8],
        overworld_assets: &'assets RuntimeAssets<'assets>,
        messaging_assets: &'messages MessagingAssets<'messages>,
        services: &mut S,
        load_pcm_track: LoadTrack,
    ) -> Result<bool, crate::runtime::ReplayError>
    where
        S: GameServices,
        LoadTrack: FnMut(u8) -> Option<crate::audio::PcmTrack>,
    {
        run_integrated_core_frame(
            self,
            input,
            nmi_assets,
            intro_song_bank,
            overworld_assets,
            messaging_assets,
            services,
            load_pcm_track,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[test]
    fn main_module_indices_match_the_c_table() {
        for index in 0..28 {
            assert_eq!(MainModule::from_index(index).map(|v| v as u8), Some(index));
        }
        assert_eq!(MainModule::from_index(28), None);
        assert_eq!(MainModule::from_index(u8::MAX), None);
        assert_eq!(MainModule::OverworldLoad as u8, 8);
        assert_eq!(MainModule::SpecialOverworldLoad as u8, 10);
        assert_eq!(MainModule::Overworld as u8, 9);
        assert_eq!(MainModule::SpecialOverworld as u8, 11);
    }

    #[test]
    fn overworld_submodule_indices_match_the_c_table() {
        use OverworldSubmodule as O;
        let expected = [
            O::PlayerControl,
            O::LoadAuxGfx,
            O::FinishTransitionGfx,
            O::LoadNewMapAndGfx,
            O::LoadNewSprites,
            O::StartScrollTransition,
            O::RunScrollTransition,
            O::EaseOffScrollTransition,
            O::FinalizeEntry,
            O::OpenBigDoorFromExit,
            O::WalkFromExitFacingDown,
            O::WalkFromExitFacingUp,
            O::OpenBigDoor,
            O::StartMosaicTransition,
            O::PreOverworldLoadOverlays,
            O::LoadAuxGfx,
            O::FinishTransitionGfx,
            O::LoadNewMapAndGfx,
            O::LoadNewSprites,
            O::StartScrollTransition,
            O::RunScrollTransition,
            O::EaseOffScrollTransition,
            O::FadeBackInFromMosaic,
            O::StartMosaicTransition,
            O::EnterSpecialArea,
            O::LoadSpecialAreaAndAdvance,
            O::LoadAuxGfx,
            O::FinishTransitionGfx,
            O::MosaicTransition,
            O::Assertion1d,
            O::Assertion1e,
            O::SpecialExit,
            O::LoadOverlays2,
            O::LoadAmbientOverlayFalse,
            O::Brighten,
            O::MirrorWarp,
            O::StartMosaicTransition,
            O::LoadOverlays,
            O::LoadAuxGfx,
            O::FinishTransitionGfx,
            O::LoadAndBuildScreen,
            O::FadeBackInFromMosaic,
            O::RecoverFromDrowning,
            O::MasterSwordPalette,
            O::MirrorWarp,
            O::WeathervaneExplosion,
            O::Whirlpool,
            O::FinishPyramidHole,
        ];
        for (index, expected) in expected.into_iter().enumerate() {
            assert_eq!(OverworldSubmodule::from_index(index as u8), Some(expected));
        }
        assert_eq!(OverworldSubmodule::from_index(48), None);
        assert_eq!(
            PreOverworldSubmodule::from_index(0),
            Some(PreOverworldSubmodule::LoadProperties)
        );
        assert_eq!(
            PreOverworldSubmodule::from_index(1),
            Some(PreOverworldSubmodule::LoadOverlays)
        );
        assert_eq!(
            PreOverworldSubmodule::from_index(2),
            Some(PreOverworldSubmodule::LoadAndAdvance)
        );
        assert_eq!(PreOverworldSubmodule::from_index(3), None);
    }

    #[derive(Default)]
    struct FileRuntime {
        initialized: usize,
    }

    impl FileSelectRuntime for FileRuntime {
        fn force_blank(&mut self, _ram: &mut Ram<'_>) {}
        fn erase_triforce_tilemaps(&mut self, _ram: &mut Ram<'_>) {}
        fn load_file_select_palettes(&mut self, _ram: &mut Ram<'_>, _sram: &[u8]) {}
        fn load_file_select_graphics(&mut self, _ram: &mut Ram<'_>) {}
        fn copy_selected_save_to_wram(&mut self, _ram: &mut Ram<'_>, _sram: &[u8], _slot: usize) {}
        fn persist_sram(&mut self, _sram: &[u8]) {}
        fn initialize_file_select_graphics(&mut self, _ram: &mut Ram<'_>) {
            self.initialized += 1;
        }
    }

    #[test]
    fn file_select_adapter_calls_the_real_state_machine() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut sram = vec![0; 0x2000];
        let mut runtime = FileRuntime::default();
        run_file_select_module(&mut ram, &mut sram, &mut runtime, MainModule::FileSelect);
        assert_eq!(runtime.initialized, 1);
    }

    #[derive(Default)]
    struct DungeonRuntime;

    impl DungeonHooks for DungeonRuntime {}

    #[test]
    fn dungeon_adapter_reborrows_ram_and_runs_the_real_entrypoint() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut runtime = DungeonRuntime;
        run_dungeon_module(&mut ram, &mut runtime, MainModule::DungeonFallingEntrance);
        assert_eq!(ram.read_u8(0xb0), 1);
    }

    #[test]
    fn typed_ending_calls_preserve_immediate_and_deferred_service_routing() {
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .unwrap();
        let assets = RuntimeAssets::parse(&asset_bytes).unwrap();
        let mut services = NativeGameServices::new(&assets);
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(MAIN_MODULE, 26);
        for call in [
            EndingCall::EnableForceBlank,
            EndingCall::EraseTileMapsNormal,
        ] {
            EndingRuntime::call(&mut services, call, &mut ram);
        }
        EndingRuntime::call(&mut services, EndingCall::MainShowTextMessage, &mut ram);
        assert_eq!(ram.read_u8(MAIN_MODULE), 14);
        assert_eq!(ram.read_u8(0x11), 2);
        EndingRuntime::call(&mut services, EndingCall::RenderText, &mut ram);
        assert_eq!(
            services.take_host_actions(),
            [
                NativeHostAction::EndingExternal(EndingCall::EnableForceBlank),
                NativeHostAction::EndingExternal(EndingCall::EraseTileMapsNormal),
                NativeHostAction::EndingExternal(EndingCall::RenderText),
            ]
        );
    }

    #[test]
    fn native_vram_bridge_emits_exact_dungeon_tilemap_span_with_latest_overlap() {
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .unwrap();
        let assets = RuntimeAssets::parse(&asset_bytes).unwrap();
        let mut services = NativeGameServices::new(&assets);
        let live_vram = [0x1111; z3_core::VRAM_WORDS];
        services.sync_vram(&live_vram);

        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(0x116, 3);
        for index in 0..0x400 {
            ram.write_u16(0x1000 + index * 2, 0x4000 | index as u16);
        }
        DungeonHooks::dungeon_nmi_upload_tilemap(&mut services, &mut ram);

        // A later production write overlapping the upload must win in the
        // transaction that is eventually drained to the live PPU.
        services.vram_write_word(0x800 + 0x155, 0xface);

        let commands = services.take_load_gfx_device_commands();
        assert_eq!(commands.vram_spans.len(), 1);
        assert_eq!(commands.vram_spans[0].start, 0x800);
        assert_eq!(commands.vram_spans[0].words.len(), 0x400);
        for (index, &word) in commands.vram_spans[0].words.iter().enumerate() {
            let expected = if index == 0x155 {
                0xface
            } else {
                0x4000 | index as u16
            };
            assert_eq!(word, expected, "VRAM word {index:#x}");
        }
        assert_eq!(ram.read_u16(0x1000), 0);
        assert_eq!(ram.read_u8(0x710), 0);
    }

    #[test]
    fn native_vram_bridge_drain_preserves_unrelated_live_ppu_write() {
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .unwrap();
        let assets = RuntimeAssets::parse(&asset_bytes).unwrap();
        let mut services = NativeGameServices::new(&assets);
        let mut ppu = Ppu::new();
        ppu.vram.fill(0x1111);
        services.sync_vram(&ppu.vram);

        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(0x116, 3);
        let expected: Vec<u16> = (0..0x400).map(|index| 0x4000 | index).collect();
        for (index, &word) in expected.iter().enumerate() {
            ram.write_u16(0x1000 + index * 2, word);
        }
        DungeonHooks::dungeon_nmi_upload_tilemap(&mut services, &mut ram);

        const SENTINEL: usize = 0x4567;
        ppu.vram[SENTINEL] = 0xbeef;
        let empty_bg: [&[u8]; 6] = [&[], &[], &[], &[], &[], &[]];
        let nmi_assets = crate::nmi::NmiAssets {
            link_graphics: &[],
            bg_tilemaps: empty_bg,
            light_overworld_tilemap: &[],
        };
        let messages: [&[u8]; 0] = [];
        let dictionary: [&[u8]; 0] = [];
        let messaging_assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dictionary,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let mut sram = [0; SRAM_SIZE];
        let mut dma = z3_snes::dma::Dma::new();
        let mut audio = crate::audio::AudioRuntime::default();
        {
            let mut hooks = IntegratedFrameHooks {
                sram: &mut sram,
                dma: &mut dma,
                ppu: &mut ppu,
                audio: &mut audio,
                nmi_assets,
                intro_song_bank: &[],
                overworld_assets: &assets,
                messaging_assets: &messaging_assets,
                enhanced_features0: 0,
                services: &mut services,
                load_pcm_track: |_: u8| None::<crate::audio::PcmTrack>,
            };
            hooks.drain_service_device_commands();
        }

        assert_eq!(&ppu.vram[0x800..0xc00], expected.as_slice());
        assert_eq!(ppu.vram[SENTINEL], 0xbeef);
    }

    #[derive(Default)]
    struct Services {
        initialized: usize,
        seen_starting_points: Vec<u8>,
        interface_steps: Vec<bool>,
    }

    impl FrameResourceHooks for Services {
        fn sync_vram(&mut self, _vram: &[u16; z3_core::VRAM_WORDS]) {}
        fn set_audio_port0(&mut self, _value: u8) {}
        fn set_music_playback_state(
            &mut self,
            _state: u8,
            _enabled: u8,
            _actual_track: u8,
            _music_unk1: u8,
            _area: u8,
            _entrance: u16,
        ) {
        }
        fn take_mirror_hdma_setup(&mut self) -> Option<overworld::MirrorHdmaSetup> {
            None
        }
        fn take_load_gfx_device_commands(&mut self) -> LoadGfxDeviceCommands {
            LoadGfxDeviceCommands::default()
        }
        fn take_host_actions(&mut self) -> Vec<NativeHostAction> {
            Vec::new()
        }
    }

    impl FileSelectRuntime for Services {
        fn force_blank(&mut self, _ram: &mut Ram<'_>) {}
        fn erase_triforce_tilemaps(&mut self, _ram: &mut Ram<'_>) {}
        fn load_file_select_palettes(&mut self, _ram: &mut Ram<'_>, _sram: &[u8]) {}
        fn load_file_select_graphics(&mut self, _ram: &mut Ram<'_>) {}
        fn copy_selected_save_to_wram(&mut self, _ram: &mut Ram<'_>, _sram: &[u8], _slot: usize) {}
        fn persist_sram(&mut self, _sram: &[u8]) {}
        fn initialize_file_select_graphics(&mut self, _ram: &mut Ram<'_>) {
            self.initialized += 1;
        }
    }

    impl MiscHooks for Services {
        fn call(&mut self, ram: &mut Ram<'_>, call: misc::MiscCall) {
            if call == misc::MiscCall::HudSearchEquipped {
                self.seen_starting_points.push(ram.read_u8(0xf3c8));
            }
        }
        fn route_main_module(&mut self, _ram: &mut Ram<'_>, _module: u8) {}
        fn load_song_bank(&mut self, _ram: &mut Ram<'_>, _bank: u8) {}
        fn predefined_tile_word(&self, _word_index: usize) -> u16 {
            0
        }
        fn refill_magic(&mut self, _ram: &mut Ram<'_>) -> bool {
            false
        }
        fn refill_health(&mut self, _ram: &mut Ram<'_>) -> bool {
            false
        }
        fn add_ancilla(&mut self, _ram: &mut Ram<'_>, _kind: u8, _arg: u8) -> i32 {
            -1
        }
        fn set_ancilla_xy(&mut self, _ram: &mut Ram<'_>, _ancilla: usize, _x: u16, _y: u16) {}
        fn sprite_x(&self, _ram: &Ram<'_>, _sprite: usize) -> u16 {
            0
        }
        fn dungeon_item_tile_action(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) -> u8 {
            0
        }
        fn overworld_item_tile_action(&mut self, _ram: &mut Ram<'_>, _x: u16, _y: u16) -> u8 {
            0
        }
    }

    impl crate::load_gfx::LoadGfxHooks for Services {}
    impl DungeonHooks for Services {}
    impl MessagingRuntime for Services {
        fn interface_gameplay_step(&mut self, _ram: &mut Ram<'_>, run_world: bool) {
            self.interface_steps.push(run_world);
        }
    }
    impl AttractRuntime for Services {}
    impl EndingRuntime for Services {}
    impl crate::sprite_main_bank0::SpriteMainBank0Runtime for Services {}
    impl crate::sprite_main_bank2b::SpriteMainBank2bRuntime for Services {}
    impl crate::sprite_main_bank1::SpriteMainBank1Runtime for Services {}
    impl crate::sprite::SpriteHooks for Services {}
    impl crate::sprite_main_bank2a::SpriteMainBank2aRuntime for Services {}
    impl crate::ancilla::AncillaRuntime for Services {}
    impl crate::player_oam::PlayerOamHooks for Services {}

    impl crate::hud::HudRuntime for Services {
        fn render(&mut self, _operation: crate::hud::HudRenderOp, _ram: &mut Ram<'_>) {}
        fn display_select_menu(&mut self, _ram: &mut Ram<'_>) {}
    }

    impl crate::overlord::OverlordHooks for Services {
        fn random_number(&mut self, _ram: &mut Ram<'_>) -> u8 {
            0
        }
        fn spawn_sprite(
            &mut self,
            _ram: &mut Ram<'_>,
            _source: usize,
            _sprite_type: u8,
            _highest_slot: usize,
        ) -> Option<(usize, crate::overlord::SpawnInfo)> {
            None
        }
        fn tile_attribute(&mut self, _ram: &mut Ram<'_>, _floor: u8, _x: &mut u16, _y: u16) -> u8 {
            0
        }
        fn garnish_alloc(&mut self, _ram: &mut Ram<'_>) -> Option<usize> {
            None
        }
        fn calculate_sfx_pan(&mut self, _ram: &mut Ram<'_>, _x: u8) -> u8 {
            0
        }
        fn queue_sfx2_with_pan(&mut self, _ram: &mut Ram<'_>, _sprite: usize, _sfx: u8) {}
        fn queue_sfx3_with_pan(&mut self, _ram: &mut Ram<'_>, _sprite: usize, _sfx: u8) {}
        fn transmute_to_bomb(&mut self, _ram: &mut Ram<'_>, _sprite: usize) {}
        fn sinus_lookup(&self, _angle: u8) -> u16 {
            0
        }
    }

    impl SpotlightServices for Services {
        fn prep_exit_with_spotlight(&mut self, _ram: &mut Ram<'_>) {}
        fn configure_spotlight_table(&mut self, _ram: &mut Ram<'_>) {}
        fn open_spotlight_next(&mut self, _ram: &mut Ram<'_>) {}
        fn spotlight_open(&mut self, _ram: &mut Ram<'_>) {}
        fn link_handle_velocity(&mut self, _ram: &mut Ram<'_>) {}
        fn link_handle_moving_animation_full_long_entry(&mut self, _ram: &mut Ram<'_>) {}
    }

    impl OverworldServices for Services {
        fn overworld_call(&mut self, _ram: &mut Ram<'_>, _call: OverworldCall) {}
        fn is_playing_music_track(&mut self, _track: u8) -> bool {
            false
        }
        fn setup_mirror_hdma(&mut self, _setup: overworld::MirrorHdmaSetup) {}
    }

    impl overworld::PreOverworldHooks for Services {
        fn load_from_dungeon(&mut self, _ram: &mut Ram<'_>) {}
        fn load_from_special_overworld(&mut self, _ram: &mut Ram<'_>) {}
        fn hud_refill_logic(&mut self, _ram: &mut Ram<'_>) {}
        fn decompress_animated_tiles(&mut self, _ram: &mut Ram<'_>, _tiles: u8) {}
        fn initialize_tilesets(&mut self, _ram: &mut Ram<'_>) {}
        fn load_screen_palette_set(&mut self, _ram: &mut Ram<'_>) {}
        fn load_palettes(&mut self, _ram: &mut Ram<'_>, _background: u8, _sprites: u8) {}
        fn set_overworld_background_color(&mut self, _ram: &mut Ram<'_>) {}
        fn load_palettes_inner(&mut self, _ram: &mut Ram<'_>) {}
        fn copy_special_palettes_to_cache(&mut self, _ram: &mut Ram<'_>) {}
        fn initialize_follower(&mut self, _ram: &mut Ram<'_>) {}
        fn decode_animated_sprite_tile(&mut self, _ram: &mut Ram<'_>, _tile: u8) {}
        fn reload_overworld_sprites(&mut self, _ram: &mut Ram<'_>) {}
        fn initialize_mirror_portal(&mut self, _ram: &mut Ram<'_>) {}
        fn reset_torch_background_and_player(&mut self, _ram: &mut Ram<'_>) {}
        fn load_bunny_palettes(&mut self, _ram: &mut Ram<'_>) {}
        fn load_overworld_music_if_needed(&mut self, _ram: &mut Ram<'_>) {}
    }

    impl overworld::PlayerControlHooks for Services {
        fn did_press_map(&mut self, _ram: &mut Ram<'_>) -> bool {
            false
        }
        fn display_select_menu(&mut self, _ram: &mut Ram<'_>) {}
        fn handle_item_switch_inputs(&mut self, _ram: &mut Ram<'_>) {}
        fn animate_entrance(&mut self, _ram: &mut Ram<'_>) {}
        fn link_main(&mut self, _ram: &mut Ram<'_>) {}
        fn super_bomb_indicator(&mut self, _ram: &mut Ram<'_>) {}
        fn load_chr_half_slot(&mut self, _ram: &mut Ram<'_>) {}
        fn operate_camera_scroll(&mut self, _ram: &mut Ram<'_>) {}
        fn use_entrance(&mut self, _ram: &mut Ram<'_>) {}
        fn animate_dark_world_palette(&mut self, _ram: &mut Ram<'_>) {}
        fn handle_transitions(&mut self, _ram: &mut Ram<'_>) {}
        fn scroll_and_check_special_exit(&mut self, _ram: &mut Ram<'_>) {}
    }

    #[test]
    fn integrated_game_is_a_core_frame_route_closure_replacement() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(MAIN_MODULE, MainModule::FileSelect as u8);
        let mut sram = vec![0; 0x2000];
        let mut dma = z3_snes::dma::Dma::new();
        let mut ppu = Ppu::new();
        let messages: [&[u8]; 0] = [];
        let dictionary: [&[u8]; 0] = [];
        let assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dictionary,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .unwrap();
        let overworld_assets = RuntimeAssets::parse(&asset_bytes).unwrap();
        let mut services = Services::default();
        let mut game = IntegratedGame {
            sram: &mut sram,
            dma: &mut dma,
            ppu: &mut ppu,
            audio: &mut crate::audio::AudioRuntime::default(),
            messaging_assets: &assets,
            overworld_assets: &overworld_assets,
            enhanced_features0: 0,
            services: &mut services,
        };
        module_main_routing(&mut ram, &mut game);
        assert_eq!(ram.read_u8(0x13), 0x80);
        assert!(ppu.vram.iter().any(|&word| word != 0));
        assert_eq!(ram.read_u8(0x11), 1);
    }

    #[test]
    fn intro_first_frame_runs_concrete_graphics_and_force_blank_services() {
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .unwrap();
        let runtime_assets = RuntimeAssets::parse(&asset_bytes).unwrap();
        let messages: [&[u8]; 0] = [];
        let dictionary: [&[u8]; 0] = [];
        let messaging_assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dictionary,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut sram = vec![0; 0x2000];
        let mut dma = z3_snes::dma::Dma::new();
        let mut ppu = Ppu::new();
        let mut services = Services::default();
        ram.write_u8(MAIN_MODULE, MainModule::Intro as u8);
        let mut game = IntegratedGame {
            sram: &mut sram,
            dma: &mut dma,
            ppu: &mut ppu,
            audio: &mut crate::audio::AudioRuntime::default(),
            messaging_assets: &messaging_assets,
            overworld_assets: &runtime_assets,
            enhanced_features0: 0,
            services: &mut services,
        };
        module_main_routing(&mut ram, &mut game);
        assert_eq!(ram.read_u8(0x13), 15);
        assert_eq!(ram.read_u8(0x9b), 0);
        assert_eq!(ram.read_u8(0x11), 1);
        assert_eq!(ram.read_u8(0xb0), 1);
        assert!(ram.slice(0x11000, 0x400).iter().any(|&byte| byte != 0));
    }

    #[test]
    fn spawn_select_uses_selected_start_then_restores_the_saved_start() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(0x1ce8, 2);
        ram.write_u8(0xf3c8, 4);
        ram.write_u8(0xb0, 9);
        let mut services = Services::default();
        run_spawn_select(&mut ram, &mut services);
        assert_eq!(services.seen_starting_points, [6]);
        assert_eq!(ram.read_u8(0xf3c8), 4);
        assert_eq!(ram.read_u8(0xb0), 0);
    }

    #[test]
    fn integrated_core_frame_routes_a_real_main_module() {
        let mut native = crate::runtime::NativeRuntime::new();
        native.wram[0xadc] = 1;
        native.wram[MAIN_MODULE] = MainModule::FileSelect as u8;
        native.wram[0x94] = 7;
        let empty_bg: [&[u8]; 6] = [&[], &[], &[], &[], &[], &[]];
        let nmi_assets = crate::nmi::NmiAssets {
            link_graphics: &[],
            bg_tilemaps: empty_bg,
            light_overworld_tilemap: &[],
        };
        let messages: [&[u8]; 0] = [];
        let dictionary: [&[u8]; 0] = [];
        let assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dictionary,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .unwrap();
        let overworld_assets = RuntimeAssets::parse(&asset_bytes).unwrap();
        let mut services = Services::default();
        let replaying = native
            .run_integrated_frame(
                0,
                nmi_assets,
                &[],
                &overworld_assets,
                &assets,
                &mut services,
                |_| None,
            )
            .unwrap();
        assert!(!replaying);
        assert_eq!(native.wram[0x13], 0x80);
        assert!(native.ppu.vram.iter().any(|&word| word != 0));
        assert_eq!(native.wram[0x11], 1);
        assert_eq!(native.debug_frame_counter, 1);
    }

    #[test]
    fn overworld_wrapper_directly_runs_the_selected_translated_submodule() {
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .unwrap();
        let assets = RuntimeAssets::parse(&asset_bytes).unwrap();
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(0x10, MainModule::Overworld as u8);
        ram.write_u8(0x11, 2);
        let mut services = Services::default();
        run_overworld(&mut ram, &assets, &mut services);
        assert_eq!(ram.read_u8(0x11), 3);
        assert_eq!(ram.read_u8(0x17), 10);
        assert_eq!(ram.read_u8(0x710), 10);
    }

    #[test]
    fn interface_wrapper_runs_gameplay_before_the_selected_submodule() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(0x11, 7);
        ram.write_u8(0x13, 5);
        ram.write_u16(0xe0, 0x1000);
        ram.write_u16(0xe2, 0x2000);
        ram.write_u16(0xe6, 0x3000);
        ram.write_u16(0xe8, 0x4000);
        ram.write_u16(0x11a, 0x12);
        ram.write_u16(0x11c, 0x34);
        let dictionary: [&[u8]; 0] = [];
        let messages: [&[u8]; 0] = [];
        let assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dictionary,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let mut sram = [0; SRAM_SIZE];
        let mut services = Services::default();

        run_messaging_module(
            &mut ram,
            &mut sram,
            &assets,
            &mut services,
            MainModule::Interface,
        );

        assert_eq!(services.interface_steps, [true]);
        assert_eq!(ram.read_u8(0x13), 4);
        assert_eq!(ram.read_u16(0x11e), 0x2012);
        assert_eq!(ram.read_u16(0x120), 0x1012);
        assert_eq!(ram.read_u16(0x122), 0x4034);
        assert_eq!(ram.read_u16(0x124), 0x3034);
    }

    #[test]
    fn ending_adapter_owns_ppu_and_vram_writes_without_aliasing_services() {
        let asset_bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../zelda3_assets.dat"
        ))
        .unwrap();
        let assets = RuntimeAssets::parse(&asset_bytes).unwrap();
        let messages: [&[u8]; 0] = [];
        let dictionary: [&[u8]; 0] = [];
        let messaging_assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dictionary,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        let mut sram = [0; SRAM_SIZE];
        let mut dma = z3_snes::dma::Dma::new();
        let mut ppu = Ppu::new();
        let mut services = Services::default();
        let mut adapter = EndingAdapter {
            sram: &mut sram,
            dma: &mut dma,
            ppu: &mut ppu,
            assets: &assets,
            messaging_assets: &messaging_assets,
            services: &mut services,
        };
        adapter.vram_write_word(0x27f0, 0x1234);
        adapter.ppu_write(0x07, 0x13);
        assert_eq!(ppu.vram[0x27f0], 0x1234);
    }

    #[test]
    fn unified_gameplay_services_reuse_one_safe_ram_borrow() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        let mut services = Services::default();
        services.run_ancilla_main(&mut ram);
        services.run_overlord_main(&mut ram);
        UnifiedGameplayServices::run_hud_module(&mut services, &mut ram);
        services.run_link_oam_main(&mut ram);
        services.run_sprite_main(&mut ram);
    }
}
