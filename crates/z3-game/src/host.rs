//! Deterministic host-side behavior translated from `src/main.c`.
//!
//! Window, renderer, controller discovery, and audio-device calls belong in a
//! thin platform adapter.  The state transitions and byte transformations in
//! this module are the portable part of the original host.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use z3_core::config::Config;

/// Loads the compiled asset bundle, falling back to the distributed BPS patch
/// plus the original ROM exactly like the C host's `LoadAssets` path.
pub fn load_asset_bundle(
    asset_path: &Path,
    patch_path: &Path,
    rom_path: &Path,
) -> io::Result<Vec<u8>> {
    match fs::read(asset_path) {
        Ok(bytes) => Ok(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let patch = fs::read(patch_path)?;
            let rom = fs::read(rom_path)?;
            z3_core::util::apply_bps(&rom, &patch).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "asset BPS patch does not match the supplied ROM",
                )
            })
        }
        Err(error) => Err(error),
    }
}

pub const COMMAND_CONTROLS: u16 = 1;
pub const COMMAND_CONTROLS_LAST: u16 = 12;
pub const COMMAND_LOAD: u16 = 13;
pub const COMMAND_LOAD_LAST: u16 = 32;
pub const COMMAND_SAVE: u16 = 33;
pub const COMMAND_SAVE_LAST: u16 = 52;
pub const COMMAND_REPLAY: u16 = 53;
pub const COMMAND_REPLAY_LAST: u16 = 72;
pub const COMMAND_LOAD_REF: u16 = 73;
pub const COMMAND_LOAD_REF_LAST: u16 = 92;
pub const COMMAND_REPLAY_REF: u16 = 93;
pub const COMMAND_REPLAY_REF_LAST: u16 = 112;

/// Host policy decoded from the C configuration command numbers. Unknown
/// values are retained so decoding never changes the legacy wire format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostCommand {
    CheatLife,
    CheatKeys,
    CheatEquipment,
    CheatWalkThroughWalls,
    ClearKeyLog,
    StopReplay,
    Fullscreen,
    Reset,
    Pause,
    PauseDimmed,
    Turbo,
    ReplayTurbo,
    WindowBigger,
    WindowSmaller,
    DisplayPerf,
    ToggleRenderer,
    VolumeUp,
    VolumeDown,
    Unknown(u16),
}

impl HostCommand {
    /// Encodes the unchanged configuration command number.
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            Self::CheatLife => 113,
            Self::CheatKeys => 114,
            Self::CheatEquipment => 115,
            Self::CheatWalkThroughWalls => 116,
            Self::ClearKeyLog => 117,
            Self::StopReplay => 118,
            Self::Fullscreen => 119,
            Self::Reset => 120,
            Self::Pause => 121,
            Self::PauseDimmed => 122,
            Self::Turbo => 123,
            Self::ReplayTurbo => 124,
            Self::WindowBigger => 125,
            Self::WindowSmaller => 126,
            Self::DisplayPerf => 127,
            Self::ToggleRenderer => 128,
            Self::VolumeUp => 129,
            Self::VolumeDown => 130,
            Self::Unknown(code) => code,
        }
    }

    /// Legacy recorder patch key, encoded only at the runtime boundary.
    #[must_use]
    pub const fn patch_key(self) -> Option<char> {
        match self {
            Self::CheatLife => Some('w'),
            Self::CheatKeys => Some('o'),
            Self::CheatEquipment => Some('W'),
            Self::CheatWalkThroughWalls => Some('E'),
            Self::ClearKeyLog => Some('k'),
            Self::StopReplay => Some('l'),
            _ => None,
        }
    }
}

impl From<u16> for HostCommand {
    fn from(code: u16) -> Self {
        match code {
            113 => Self::CheatLife,
            114 => Self::CheatKeys,
            115 => Self::CheatEquipment,
            116 => Self::CheatWalkThroughWalls,
            117 => Self::ClearKeyLog,
            118 => Self::StopReplay,
            119 => Self::Fullscreen,
            120 => Self::Reset,
            121 => Self::Pause,
            122 => Self::PauseDimmed,
            123 => Self::Turbo,
            124 => Self::ReplayTurbo,
            125 => Self::WindowBigger,
            126 => Self::WindowSmaller,
            127 => Self::DisplayPerf,
            128 => Self::ToggleRenderer,
            129 => Self::VolumeUp,
            130 => Self::VolumeDown,
            _ => Self::Unknown(code),
        }
    }
}

pub const DEFAULT_AUDIO_FREQUENCY: u16 = 44_100;
pub const DEFAULT_AUDIO_CHANNELS: u8 = 2;
pub const DEFAULT_AUDIO_SAMPLES: u16 = 2_048;
pub const MAX_WINDOW_SCALE: u8 = 10;

const KEYBOARD_REMAP: [u8; 13] = [0, 4, 5, 6, 7, 2, 3, 8, 0, 9, 1, 10, 11];
const SEGMENT_TO_BUTTONS: [u16; 8] = [
    1 << 4,
    1 << 4 | 1 << 7,
    1 << 7,
    1 << 7 | 1 << 5,
    1 << 5,
    1 << 5 | 1 << 6,
    1 << 6,
    1 << 6 | 1 << 4,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveLoadKind {
    Load,
    Save,
    Replay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostAction {
    SaveLoad { kind: SaveLoadKind, slot: u16 },
    Command(HostCommand),
}

/// Applies the exact startup range checks from `main.c` after configuration
/// parsing and returns the initial integer window scale.
pub fn normalize_host_config(config: &mut Config) -> u8 {
    if !(11_025..=48_000).contains(&config.audio_freq) {
        config.audio_freq = DEFAULT_AUDIO_FREQUENCY;
    }
    if !(1..=2).contains(&config.audio_channels) {
        config.audio_channels = DEFAULT_AUDIO_CHANNELS;
    }
    if config.audio_samples == 0 || !config.audio_samples.is_power_of_two() {
        config.audio_samples = DEFAULT_AUDIO_SAMPLES;
    }
    if config.window_scale == 0 {
        2
    } else {
        config.window_scale.min(MAX_WINDOW_SCALE)
    }
}

const REFERENCE_SAVES: [&str; 13] = [
    "Chapter 1 - Zelda's Rescue.sav",
    "Chapter 2 - After Eastern Palace.sav",
    "Chapter 3 - After Desert Palace.sav",
    "Chapter 4 - After Tower of Hera.sav",
    "Chapter 5 - After Hyrule Castle Tower.sav",
    "Chapter 6 - After Dark Palace.sav",
    "Chapter 7 - After Swamp Palace.sav",
    "Chapter 8 - After Skull Woods.sav",
    "Chapter 9 - After Gargoyle's Domain.sav",
    "Chapter 10 - After Ice Palace.sav",
    "Chapter 11 - After Misery Mire.sav",
    "Chapter 12 - After Turtle Rock.sav",
    "Chapter 13 - After Ganon's Tower.sav",
];

#[must_use]
pub fn save_slot_path(kind: SaveLoadKind, slot: u16) -> Option<PathBuf> {
    if slot & 256 != 0 {
        if kind == SaveLoadKind::Save {
            return None;
        }
        let name = REFERENCE_SAVES.get(usize::from(slot - 256))?;
        Some(Path::new("saves/ref").join(name))
    } else {
        Some(PathBuf::from(format!("saves/save{slot}.sav")))
    }
}

/// Reads the fixed 8192-byte SRAM image. A missing file is the C host's normal
/// fresh-start condition and leaves the supplied SRAM untouched.
pub fn read_sram(path: &Path, sram: &mut [u8; 0x2000]) -> io::Result<bool> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    file.read_exact(sram)?;
    Ok(true)
}

/// Mirrors the C host's recoverable write: first rename the old image to
/// `.bak` (ignoring failure), then replace the primary file.
pub fn write_sram(path: &Path, sram: &[u8; 0x2000]) -> io::Result<()> {
    let backup = path.with_extension("bak");
    let _ = fs::rename(path, backup);
    let mut file = File::create(path)?;
    file.write_all(sram)
}

/// Finds `zelda3.ini` in the current directory or either of its first two
/// ancestors, matching the bounded startup search in `SwitchDirectory`.
#[must_use]
pub fn find_project_directory(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .take(3)
        .find(|directory| directory.join("zelda3.ini").is_file())
        .map(Path::to_path_buf)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputState {
    pub keyboard_buttons: u16,
    pub gamepad_buttons: u16,
    pub turbo: bool,
    gamepad_modifiers: u32,
    gamepad_last_command: [u16; 17],
    last_gamepad_id: i32,
    last_axis_x: i32,
    last_axis_y: i32,
}

impl InputState {
    /// Applies the twelve direct SNES controls or returns a host command which
    /// the platform adapter must execute while holding the audio lock.
    pub fn handle_command(&mut self, command: u16, pressed: bool) -> Option<HostAction> {
        if command <= COMMAND_CONTROLS_LAST {
            let bit = KEYBOARD_REMAP[usize::from(command)];
            if pressed {
                self.keyboard_buttons |= 1 << bit;
            } else {
                self.keyboard_buttons &= !(1 << bit);
            }
            return None;
        }
        if HostCommand::from(command) == HostCommand::Turbo {
            self.turbo = pressed;
            return None;
        }
        if !pressed {
            return None;
        }
        Some(command_action(command))
    }

    /// C clears analog/d-pad direction input whenever keyboard direction input
    /// is active, then ORs both sources together.
    #[must_use]
    pub fn combined_buttons(&mut self) -> u16 {
        if self.keyboard_buttons & 0xf0 != 0 {
            self.gamepad_buttons = 0;
        }
        self.keyboard_buttons | self.gamepad_buttons
    }

    /// Updates one logical gamepad button. `find_command` is the exact
    /// modifier-aware map lookup from the configuration layer.
    pub fn handle_gamepad_button(
        &mut self,
        button: usize,
        pressed: bool,
        mut find_command: impl FnMut(usize, u32) -> u16,
    ) -> Option<HostAction> {
        assert!(button < self.gamepad_last_command.len());
        let mask = 1_u32 << button;
        if (self.gamepad_modifiers & mask != 0) == pressed {
            return None;
        }
        self.gamepad_modifiers ^= mask;
        if pressed {
            self.gamepad_last_command[button] = find_command(button, self.gamepad_modifiers);
        }
        let command = self.gamepad_last_command[button];
        (command != 0)
            .then(|| self.handle_command(command, pressed))
            .flatten()
    }

    /// Handles SDL's left-stick axes. Other gamepads are ignored until they
    /// cross the original 16000 takeover threshold.
    pub fn handle_stick_axis(&mut self, gamepad_id: i32, is_x: bool, value: i32) {
        if self.last_gamepad_id != gamepad_id {
            if (-15_999..=15_999).contains(&value) {
                return;
            }
            self.last_gamepad_id = gamepad_id;
            self.last_axis_x = 0;
            self.last_axis_y = 0;
        }
        if is_x {
            self.last_axis_x = value;
        } else {
            self.last_axis_y = value;
        }
        let squared = i64::from(self.last_axis_x) * i64::from(self.last_axis_x)
            + i64::from(self.last_axis_y) * i64::from(self.last_axis_y);
        self.gamepad_buttons = if squared >= 100_000_000 {
            let angle = (approximate_atan2(self.last_axis_y as f32, self.last_axis_x as f32) * 64.0
                + 0.5) as i32 as u8;
            SEGMENT_TO_BUTTONS[usize::from(angle.wrapping_add(80) >> 5)]
        } else {
            0
        };
    }
}

#[must_use]
pub fn command_action(command: u16) -> HostAction {
    match command {
        COMMAND_LOAD..=COMMAND_LOAD_LAST => HostAction::SaveLoad {
            kind: SaveLoadKind::Load,
            slot: command - COMMAND_LOAD,
        },
        COMMAND_SAVE..=COMMAND_SAVE_LAST => HostAction::SaveLoad {
            kind: SaveLoadKind::Save,
            slot: command - COMMAND_SAVE,
        },
        COMMAND_REPLAY..=COMMAND_REPLAY_LAST => HostAction::SaveLoad {
            kind: SaveLoadKind::Replay,
            slot: command - COMMAND_REPLAY,
        },
        COMMAND_LOAD_REF..=COMMAND_LOAD_REF_LAST => HostAction::SaveLoad {
            kind: SaveLoadKind::Load,
            slot: 256 + command - COMMAND_LOAD_REF,
        },
        COMMAND_REPLAY_REF..=COMMAND_REPLAY_REF_LAST => HostAction::SaveLoad {
            kind: SaveLoadKind::Replay,
            slot: 256 + command - COMMAND_REPLAY_REF,
        },
        _ => HostAction::Command(command.into()),
    }
}

/// The C host's fast atan2 approximation, normalized to `[0, 4)`.
#[must_use]
pub fn approximate_atan2(y: f32, x: f32) -> f32 {
    const SIGN_MASK: u32 = 0x8000_0000;
    let x_sign = SIGN_MASK & x.to_bits();
    let y_sign = SIGN_MASK & y.to_bits();
    let quadrant = (((!x_sign & y_sign) >> 29) | (x_sign >> 30)) as f32;
    let mut product = 0.596_227_f32 * x * y;
    if product < 0.0 {
        product = -product;
    }
    let numerator = product + y * y;
    let first_quadrant = numerator / (x * x + product + numerator + 0.000_001);
    quadrant + f32::from_bits((x_sign ^ y_sign) | first_quadrant.to_bits())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZsprError {
    InvalidHeader,
    Truncated,
    InvalidPixelLength,
    InvalidTargetLengths,
}

/// Applies a ZSPR file to the mutable Link graphics/palette assets, preserving
/// the C parser's optional palette suffix behavior.
pub fn parse_link_graphics(
    file: &[u8],
    graphics: &mut [u8],
    armor_and_gloves: &mut [u8],
    gloves_color: &mut [u8],
) -> Result<(), ZsprError> {
    if file.len() < 27 || file.get(..4) != Some(b"ZSPR") {
        return Err(ZsprError::InvalidHeader);
    }
    if graphics.len() != 0x7000 || armor_and_gloves.len() != 150 || gloves_color.len() != 4 {
        return Err(ZsprError::InvalidTargetLengths);
    }
    let pixel_offset = read_u32(file, 9).ok_or(ZsprError::Truncated)? as usize;
    let pixel_length = usize::from(read_u16(file, 13).ok_or(ZsprError::Truncated)?);
    let palette_offset = read_u32(file, 15).ok_or(ZsprError::Truncated)? as usize;
    let palette_length = usize::from(read_u16(file, 19).ok_or(ZsprError::Truncated)?);
    if pixel_length != 0x7000 {
        return Err(ZsprError::InvalidPixelLength);
    }
    let pixels = file
        .get(
            pixel_offset
                ..pixel_offset
                    .checked_add(pixel_length)
                    .ok_or(ZsprError::Truncated)?,
        )
        .ok_or(ZsprError::Truncated)?;
    let palette = file
        .get(
            palette_offset
                ..palette_offset
                    .checked_add(palette_length)
                    .ok_or(ZsprError::Truncated)?,
        )
        .ok_or(ZsprError::Truncated)?;
    graphics.copy_from_slice(pixels);
    if palette_length >= 120 {
        armor_and_gloves[..120].copy_from_slice(&palette[..120]);
    }
    if palette_length >= 124 {
        gloves_color.copy_from_slice(&palette[120..124]);
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

const PERF_FONT: [u8; 100] = [
    0x1c, 0x36, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x36, 0x1c, 0x18, 0x1c, 0x1e, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x7e, 0x3e, 0x63, 0x60, 0x30, 0x18, 0x0c, 0x06, 0x03, 0x63, 0x7f, 0x3e, 0x63,
    0x60, 0x60, 0x3c, 0x60, 0x60, 0x60, 0x63, 0x3e, 0x30, 0x38, 0x3c, 0x36, 0x33, 0x7f, 0x30, 0x30,
    0x30, 0x78, 0x7f, 0x03, 0x03, 0x03, 0x3f, 0x60, 0x60, 0x60, 0x63, 0x3e, 0x1c, 0x06, 0x03, 0x03,
    0x3f, 0x63, 0x63, 0x63, 0x63, 0x3e, 0x7f, 0x63, 0x60, 0x60, 0x30, 0x18, 0x0c, 0x0c, 0x0c, 0x0c,
    0x3e, 0x63, 0x63, 0x63, 0x3e, 0x63, 0x63, 0x63, 0x63, 0x3e, 0x3e, 0x63, 0x63, 0x63, 0x7e, 0x60,
    0x60, 0x60, 0x30, 0x1e,
];

/// Draws the C host's performance counter into an ARGB8888 framebuffer.
pub fn render_number(pixels: &mut [u8], pitch: usize, number: i32, big: bool) {
    let text = number.to_string();
    for (index, byte) in text.bytes().enumerate() {
        render_digit(
            pixels,
            pitch,
            byte - b'0',
            0x0040_4040,
            big,
            (pitch + 12 + index * 32) << usize::from(big),
        );
    }
    for (index, byte) in text.bytes().enumerate() {
        render_digit(
            pixels,
            pitch,
            byte - b'0',
            0x00ff_ffff,
            big,
            (8 + index * 32) << usize::from(big),
        );
    }
}

fn render_digit(pixels: &mut [u8], pitch: usize, digit: u8, color: u32, big: bool, start: usize) {
    let scale = if big { 2 } else { 1 };
    for y in 0..10 {
        let mut bits = PERF_FONT[usize::from(digit) * 10 + y];
        let mut x = 0;
        while bits != 0 {
            if bits & 1 != 0 {
                for yy in 0..scale {
                    for xx in 0..scale {
                        let offset = start + (y * scale + yy) * pitch + (x * scale + xx) * 4;
                        if let Some(target) = pixels.get_mut(offset..offset + 4) {
                            target.copy_from_slice(&color.to_ne_bytes());
                        }
                    }
                }
            }
            bits >>= 1;
            x += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_controls_use_the_c_snes_bit_permutation() {
        let mut input = InputState::default();
        input.handle_command(1, true);
        input.handle_command(7, true);
        assert_eq!(input.keyboard_buttons, (1 << 4) | (1 << 8));
        input.handle_command(1, false);
        assert_eq!(input.keyboard_buttons, 1 << 8);
    }

    #[test]
    fn host_command_codec_preserves_all_legacy_values() {
        let commands = [
            HostCommand::CheatLife,
            HostCommand::CheatKeys,
            HostCommand::CheatEquipment,
            HostCommand::CheatWalkThroughWalls,
            HostCommand::ClearKeyLog,
            HostCommand::StopReplay,
            HostCommand::Fullscreen,
            HostCommand::Reset,
            HostCommand::Pause,
            HostCommand::PauseDimmed,
            HostCommand::Turbo,
            HostCommand::ReplayTurbo,
            HostCommand::WindowBigger,
            HostCommand::WindowSmaller,
            HostCommand::DisplayPerf,
            HostCommand::ToggleRenderer,
            HostCommand::VolumeUp,
            HostCommand::VolumeDown,
        ];
        for (code, command) in (113..=130).zip(commands) {
            assert_eq!(HostCommand::from(code), command);
            assert_eq!(command.code(), code);
        }
        for code in 0..=u16::MAX {
            let command = HostCommand::from(code);
            assert_eq!(command.code(), code);
            if !(113..=130).contains(&code) {
                assert_eq!(command, HostCommand::Unknown(code));
            }
        }
    }

    #[test]
    fn patch_keys_keep_the_recording_command_protocol() {
        for (command, key) in [
            (HostCommand::CheatLife, 'w'),
            (HostCommand::CheatKeys, 'o'),
            (HostCommand::CheatEquipment, 'W'),
            (HostCommand::CheatWalkThroughWalls, 'E'),
            (HostCommand::ClearKeyLog, 'k'),
            (HostCommand::StopReplay, 'l'),
        ] {
            assert_eq!(command.patch_key(), Some(key));
        }
        for code in 119..=u16::MAX {
            assert_eq!(HostCommand::from(code).patch_key(), None);
        }
    }

    #[test]
    fn commands_fire_on_press_while_turbo_tracks_both_edges() {
        let mut input = InputState::default();
        assert_eq!(
            input.handle_command(121, true),
            Some(HostAction::Command(HostCommand::Pause))
        );
        assert_eq!(input.handle_command(121, false), None);
        assert_eq!(input.handle_command(123, true), None);
        assert!(input.turbo);
        assert_eq!(input.handle_command(123, false), None);
        assert!(!input.turbo);
        assert_eq!(
            input.handle_command(u16::MAX, true),
            Some(HostAction::Command(HostCommand::Unknown(u16::MAX)))
        );
        assert_eq!(input.handle_command(u16::MAX, false), None);
    }

    #[test]
    fn gamepad_release_uses_the_command_selected_on_press() {
        let mut input = InputState::default();
        assert_eq!(input.handle_gamepad_button(0, true, |_, _| 123), None);
        assert!(input.turbo);
        assert_eq!(
            input.handle_gamepad_button(0, false, |_, _| {
                panic!("release must retain the pressed command")
            }),
            None
        );
        assert!(!input.turbo);
    }

    #[test]
    fn host_configuration_defaults_match_main() {
        let mut config = Config::default();
        assert_eq!(normalize_host_config(&mut config), 2);
        assert_eq!(config.audio_freq, 44_100);
        assert_eq!(config.audio_channels, 2);
        assert_eq!(config.audio_samples, 2_048);
        config.window_scale = 255;
        config.audio_freq = 48_000;
        config.audio_channels = 1;
        config.audio_samples = 1_024;
        assert_eq!(normalize_host_config(&mut config), 10);
        assert_eq!(config.audio_freq, 48_000);
        assert_eq!(config.audio_channels, 1);
        assert_eq!(config.audio_samples, 1_024);
    }

    #[test]
    fn reference_save_paths_and_save_prohibition_match_c() {
        assert_eq!(
            save_slot_path(SaveLoadKind::Load, 256),
            Some(PathBuf::from("saves/ref/Chapter 1 - Zelda's Rescue.sav"))
        );
        assert_eq!(save_slot_path(SaveLoadKind::Save, 256), None);
        assert_eq!(
            save_slot_path(SaveLoadKind::Replay, 7),
            Some(PathBuf::from("saves/save7.sav"))
        );
    }

    #[test]
    fn project_directory_search_is_bounded_to_two_parents() {
        let root = std::env::temp_dir().join(format!(
            "zrust-host-search-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let nested = root.join("one/two/three");
        fs::create_dir_all(&nested).unwrap();
        File::create(root.join("zelda3.ini")).unwrap();
        assert_eq!(
            find_project_directory(&root.join("one/two")),
            Some(root.clone())
        );
        assert_eq!(find_project_directory(&nested), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn compiled_asset_file_takes_precedence_over_patch_fallback() {
        let root = std::env::temp_dir().join(format!(
            "zrust-host-assets-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::create_dir_all(&root).unwrap();
        let compiled = root.join("zelda3_assets.dat");
        fs::write(&compiled, b"compiled").unwrap();
        assert_eq!(
            load_asset_bundle(
                &compiled,
                &root.join("missing.bps"),
                &root.join("missing.sfc")
            )
            .unwrap(),
            b"compiled"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn save_load_ranges_preserve_reference_slot_encoding() {
        assert_eq!(
            command_action(73),
            HostAction::SaveLoad {
                kind: SaveLoadKind::Load,
                slot: 256
            }
        );
        assert_eq!(
            command_action(112),
            HostAction::SaveLoad {
                kind: SaveLoadKind::Replay,
                slot: 275
            }
        );
    }

    #[test]
    fn stick_takeover_deadzone_and_octants_match_c() {
        let mut input = InputState::default();
        input.handle_stick_axis(1, true, 10_001);
        assert_eq!(input.gamepad_buttons, 0); // ignored: inactive controller
        input.handle_stick_axis(1, true, 16_001);
        assert_eq!(input.gamepad_buttons, 1 << 7);
        input.handle_stick_axis(1, false, -16_001);
        assert_eq!(input.gamepad_buttons, (1 << 7) | (1 << 4));
    }

    #[test]
    fn atan_approximation_hits_cardinal_quadrants() {
        assert!((approximate_atan2(0.0, 1.0) - 0.0).abs() < 0.0001);
        assert!((approximate_atan2(1.0, 0.0) - 1.0).abs() < 0.0001);
        assert!((approximate_atan2(0.0, -1.0) - 2.0).abs() < 0.0001);
        assert!((approximate_atan2(-1.0, 0.0) - 3.0).abs() < 0.0001);
    }

    #[test]
    fn zspr_parser_applies_optional_palette_ranges() {
        let mut file = vec![0; 27 + 0x7000 + 124];
        file[..4].copy_from_slice(b"ZSPR");
        file[9..13].copy_from_slice(&27_u32.to_le_bytes());
        file[13..15].copy_from_slice(&0x7000_u16.to_le_bytes());
        file[15..19].copy_from_slice(&(27_u32 + 0x7000).to_le_bytes());
        file[19..21].copy_from_slice(&124_u16.to_le_bytes());
        file[27..27 + 0x7000].fill(0x5a);
        file[27 + 0x7000..].fill(0xa5);
        let mut graphics = [0; 0x7000];
        let mut palettes = [0; 150];
        let mut gloves = [0; 4];
        parse_link_graphics(&file, &mut graphics, &mut palettes, &mut gloves).unwrap();
        assert!(graphics.iter().all(|&byte| byte == 0x5a));
        assert!(palettes[..120].iter().all(|&byte| byte == 0xa5));
        assert_eq!(gloves, [0xa5; 4]);
    }

    #[test]
    fn number_renderer_draws_shadow_and_foreground() {
        let pitch = 128;
        let mut pixels = vec![0; pitch * 24];
        render_number(&mut pixels, pitch, 8, false);
        assert!(
            pixels
                .chunks_exact(4)
                .any(|pixel| pixel == 0x00ff_ffff_u32.to_ne_bytes())
        );
        assert!(
            pixels
                .chunks_exact(4)
                .any(|pixel| pixel == 0x0040_4040_u32.to_ne_bytes())
        );
    }
}
