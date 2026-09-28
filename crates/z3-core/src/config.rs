//! Behavior-preserving host configuration primitives translated from
//! `src/config.c`.
//!
//! SDL's name-to-keycode lookup remains a host concern.  The parser accepts a
//! resolver callback, which keeps this crate dependency-free while preserving
//! the C key encoding and duplicate/precedence rules exactly.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::util::{
    next_delim, next_line_strip_comments, next_possibly_quoted_string,
    replace_filename_with_new_path, split_key_value,
};

pub const KEY_MOD_SCANCODE: u16 = 0x200;
pub const KEY_MOD_ALT: u16 = 0x400;
pub const KEY_MOD_SHIFT: u16 = 0x800;
pub const KEY_MOD_CTRL: u16 = 0x1000;
pub const SDL_SCANCODE_MASK: i32 = 1 << 30;

pub const FEATURE_EXTEND_SCREEN_64: u32 = 1;
pub const FEATURE_SWITCH_LR: u32 = 2;
pub const FEATURE_TURN_WHILE_DASHING: u32 = 4;
pub const FEATURE_MIRROR_TO_DARKWORLD: u32 = 8;
pub const FEATURE_COLLECT_ITEMS_WITH_SWORD: u32 = 16;
pub const FEATURE_BREAK_POTS_WITH_SWORD: u32 = 32;
pub const FEATURE_DISABLE_LOW_HEALTH_BEEP: u32 = 64;
pub const FEATURE_SKIP_INTRO_ON_KEYPRESS: u32 = 128;
pub const FEATURE_SHOW_MAX_ITEMS_IN_YELLOW: u32 = 256;
pub const FEATURE_MORE_ACTIVE_BOMBS: u32 = 512;
pub const FEATURE_WIDESCREEN_VISUAL_FIXES: u32 = 1024;
pub const FEATURE_CARRY_MORE_RUPEES: u32 = 2048;
pub const FEATURE_MISC_BUG_FIXES: u32 = 4096;
pub const FEATURE_CANCEL_BIRD_TRAVEL: u32 = 8192;
pub const FEATURE_GAME_CHANGING_BUG_FIXES: u32 = 16384;
pub const FEATURE_SWITCH_LR_LIMIT: u32 = 32768;
pub const FEATURE_DIM_FLASHES: u32 = 65536;

pub const MSU_ENABLED_MSU: u8 = 1;
pub const MSU_ENABLED_DELUXE: u8 = 2;
pub const MSU_ENABLED_OPUZ: u8 = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum OutputMethod {
    #[default]
    Sdl,
    SdlSoftware,
    OpenGl,
    OpenGlEs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub window_width: i32,
    pub window_height: i32,
    pub enhanced_mode7: bool,
    pub new_renderer: bool,
    pub ignore_aspect_ratio: bool,
    pub fullscreen: u8,
    pub window_scale: u8,
    pub enable_audio: bool,
    pub linear_filtering: bool,
    pub output_method: OutputMethod,
    pub audio_freq: u16,
    pub audio_channels: u8,
    pub audio_samples: u16,
    pub autosave: bool,
    pub extended_aspect_ratio: u8,
    pub extend_y: bool,
    pub no_sprite_limits: bool,
    pub display_perf_title: bool,
    pub enable_msu: u8,
    pub resume_msu: bool,
    pub disable_frame_delay: bool,
    pub msu_volume: u8,
    pub features0: u32,
    pub link_graphics: Option<String>,
    pub shader: Option<String>,
    pub msu_path: Option<String>,
    pub language: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_width: 0,
            window_height: 0,
            enhanced_mode7: false,
            new_renderer: false,
            ignore_aspect_ratio: false,
            fullscreen: 0,
            window_scale: 0,
            enable_audio: false,
            linear_filtering: false,
            output_method: OutputMethod::Sdl,
            audio_freq: 0,
            audio_channels: 0,
            audio_samples: 0,
            autosave: false,
            extended_aspect_ratio: 0,
            extend_y: false,
            no_sprite_limits: false,
            display_perf_title: false,
            enable_msu: 0,
            resume_msu: false,
            disable_frame_delay: false,
            msu_volume: 100,
            features0: 0,
            link_graphics: None,
            shader: None,
            msu_path: None,
            language: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    Back,
    Guide,
    Start,
    L3,
    R3,
    L1,
    R1,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    L2,
    R2,
}

pub const GAMEPAD_BUTTON_COUNT: usize = 17;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct KeyEntry {
    key: u16,
    command: u16,
    next: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyMap {
    first: [u16; 255],
    entries: Vec<KeyEntry>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            first: [0; 255],
            entries: Vec::new(),
        }
    }
}

impl KeyMap {
    /// Returns false for a duplicate, after preserving the C implementation's
    /// otherwise-unlinked duplicate allocation.
    pub fn add(&mut self, key: u16, command: u16) -> bool {
        assert!(self.entries.len() <= 10_000, "too many keys");
        let index = self.entries.len();
        self.entries.push(KeyEntry {
            key,
            command,
            next: 0,
        });
        let bucket = usize::from(key % 255);
        let mut link = self.first[bucket];
        if link == 0 {
            self.first[bucket] = (index + 1) as u16;
            return true;
        }
        loop {
            let entry_index = usize::from(link - 1);
            if self.entries[entry_index].key == key {
                return false;
            }
            let next = self.entries[entry_index].next;
            if next == 0 {
                self.entries[entry_index].next = (index + 1) as u16;
                return true;
            }
            link = next;
        }
    }

    #[must_use]
    pub fn find(&self, key: u16) -> u16 {
        let mut link = self.first[usize::from(key % 255)];
        while link != 0 {
            let entry = self.entries[usize::from(link - 1)];
            if entry.key == key {
                return entry.command;
            }
            link = entry.next;
        }
        0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GamepadEntry {
    modifiers: u32,
    command: u16,
    next: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GamepadMap {
    first: [u16; GAMEPAD_BUTTON_COUNT],
    entries: Vec<GamepadEntry>,
}

impl Default for GamepadMap {
    fn default() -> Self {
        Self {
            first: [0; GAMEPAD_BUTTON_COUNT],
            entries: Vec::new(),
        }
    }
}

impl GamepadMap {
    pub fn add(&mut self, button: GamepadButton, modifiers: u32, command: u16) {
        assert!(self.entries.len() <= 1_000, "too many joypad keys");
        let button = button as usize;
        let bits = count_bits32(modifiers);
        let mut previous: Option<usize> = None;
        let mut link = self.first[button];
        while link != 0 && bits < count_bits32(self.entries[usize::from(link - 1)].modifiers) {
            previous = Some(usize::from(link - 1));
            link = self.entries[usize::from(link - 1)].next;
        }
        let index = self.entries.len();
        self.entries.push(GamepadEntry {
            modifiers,
            command,
            next: link,
        });
        let encoded = (index + 1) as u16;
        if let Some(previous) = previous {
            self.entries[previous].next = encoded;
        } else {
            self.first[button] = encoded;
        }
    }

    #[must_use]
    pub fn find(&self, button: GamepadButton, modifiers: u32) -> u16 {
        let mut link = self.first[button as usize];
        while link != 0 {
            let entry = self.entries[usize::from(link - 1)];
            if modifiers & entry.modifiers == entry.modifiers {
                return entry.command;
            }
            link = entry.next;
        }
        0
    }
}

/// C's `CountBits32`, kept explicit because ordering equally-specific gamepad
/// chords is observable in the input map.
#[must_use]
pub const fn count_bits32(mut value: u32) -> u32 {
    let mut count = 0;
    while value != 0 {
        value &= value - 1;
        count += 1;
    }
    count
}

#[must_use]
pub const fn remap_sdl_keycode(key: i32) -> u16 {
    let scancode = if key & SDL_SCANCODE_MASK != 0 {
        KEY_MOD_SCANCODE
    } else {
        0
    };
    scancode | (key as u16 & (KEY_MOD_SCANCODE - 1))
}

/// SDL modifier bit masks are supplied by the host so this helper can remain
/// independent of a specific SDL binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SdlKeyConstants {
    pub alt_mask: u16,
    pub ctrl_mask: u16,
    pub shift_mask: u16,
    pub left_alt: i32,
    pub right_alt: i32,
    pub left_ctrl: i32,
    pub right_ctrl: i32,
    pub left_shift: i32,
    pub right_shift: i32,
}

#[must_use]
pub fn find_command_for_sdl_key(
    map: &KeyMap,
    code: i32,
    modifiers: u16,
    constants: SdlKeyConstants,
) -> u16 {
    if code & !(SDL_SCANCODE_MASK | 0x1ff) != 0 {
        return 0;
    }
    let mut key = remap_sdl_keycode(code);
    if code != constants.left_alt
        && code != constants.right_alt
        && modifiers & constants.alt_mask != 0
    {
        key |= KEY_MOD_ALT;
    }
    if code != constants.left_ctrl
        && code != constants.right_ctrl
        && modifiers & constants.ctrl_mask != 0
    {
        key |= KEY_MOD_CTRL;
    }
    if code != constants.left_shift
        && code != constants.right_shift
        && modifiers & constants.shift_mask != 0
    {
        key |= KEY_MOD_SHIFT;
    }
    map.find(key)
}

#[must_use]
pub fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "0" | "false" | "no" | "off" => Some(false),
        "1" | "true" | "yes" | "on" => Some(true),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigState {
    pub config: Config,
    pub keyboard: KeyMap,
    pub gamepad: GamepadMap,
    configured_key_groups: HashSet<usize>,
    has_gamepad_controls: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigWarning {
    pub path: PathBuf,
    pub line: usize,
    pub message: String,
}

#[derive(Clone, Copy)]
struct KeyGroup {
    name: &'static str,
    command: u16,
    size: usize,
}

// Values follow config.h exactly.
const CONTROLS: u16 = 1;
const LOAD: u16 = 13;
const SAVE: u16 = 33;
const REPLAY: u16 = 53;
const LOAD_REF: u16 = 73;
const REPLAY_REF: u16 = 93;
const CHEAT_LIFE: u16 = 113;

const KEY_GROUPS: [KeyGroup; 25] = [
    KeyGroup {
        name: "Null",
        command: 0,
        size: 65535,
    },
    KeyGroup {
        name: "Controls",
        command: CONTROLS,
        size: 12,
    },
    KeyGroup {
        name: "Load",
        command: LOAD,
        size: 20,
    },
    KeyGroup {
        name: "Save",
        command: SAVE,
        size: 20,
    },
    KeyGroup {
        name: "Replay",
        command: REPLAY,
        size: 20,
    },
    KeyGroup {
        name: "LoadRef",
        command: LOAD_REF,
        size: 20,
    },
    KeyGroup {
        name: "ReplayRef",
        command: REPLAY_REF,
        size: 20,
    },
    KeyGroup {
        name: "CheatLife",
        command: CHEAT_LIFE,
        size: 1,
    },
    KeyGroup {
        name: "CheatKeys",
        command: 114,
        size: 1,
    },
    KeyGroup {
        name: "CheatEquipment",
        command: 115,
        size: 1,
    },
    KeyGroup {
        name: "CheatWalkThroughWalls",
        command: 116,
        size: 1,
    },
    KeyGroup {
        name: "ClearKeyLog",
        command: 117,
        size: 1,
    },
    KeyGroup {
        name: "StopReplay",
        command: 118,
        size: 1,
    },
    KeyGroup {
        name: "Fullscreen",
        command: 119,
        size: 1,
    },
    KeyGroup {
        name: "Reset",
        command: 120,
        size: 1,
    },
    KeyGroup {
        name: "Pause",
        command: 121,
        size: 1,
    },
    KeyGroup {
        name: "PauseDimmed",
        command: 122,
        size: 1,
    },
    KeyGroup {
        name: "Turbo",
        command: 123,
        size: 1,
    },
    KeyGroup {
        name: "ReplayTurbo",
        command: 124,
        size: 1,
    },
    KeyGroup {
        name: "WindowBigger",
        command: 125,
        size: 1,
    },
    KeyGroup {
        name: "WindowSmaller",
        command: 126,
        size: 1,
    },
    KeyGroup {
        name: "VolumeUp",
        command: 129,
        size: 1,
    },
    KeyGroup {
        name: "VolumeDown",
        command: 130,
        size: 1,
    },
    KeyGroup {
        name: "DisplayPerf",
        command: 127,
        size: 1,
    },
    KeyGroup {
        name: "ToggleRenderer",
        command: 128,
        size: 1,
    },
];

impl ConfigState {
    /// Parses one in-memory configuration file. Includes are retrieved through
    /// `load`, mirroring the C parser's recursive relative-path behavior.
    pub fn parse_text<F, R>(
        &mut self,
        path: &Path,
        text: &str,
        resolve_key: &mut R,
        load: &mut F,
    ) -> Vec<ConfigWarning>
    where
        F: FnMut(&Path) -> io::Result<String>,
        R: FnMut(&str) -> Option<i32>,
    {
        let mut warnings = Vec::new();
        self.parse_text_inner(path, text, 0, resolve_key, load, &mut warnings);
        warnings
    }

    fn parse_text_inner<F, R>(
        &mut self,
        path: &Path,
        text: &str,
        depth: usize,
        resolve_key: &mut R,
        load: &mut F,
        warnings: &mut Vec<ConfigWarning>,
    ) where
        F: FnMut(&Path) -> io::Result<String>,
        R: FnMut(&str) -> Option<i32>,
    {
        let mut section = -2_i8;
        let mut remaining = Some(text);
        let mut line = 0;
        while let Some(text) = next_line_strip_comments(&mut remaining) {
            line += 1;
            if text.is_empty() {
                continue;
            }
            if text.starts_with('[') {
                section = section_id(text).unwrap_or_else(|| {
                    warnings.push(warning(path, line, format!("invalid .ini section {text}")));
                    -1
                });
                continue;
            }
            if let Some(include) = text.strip_prefix("!include ") {
                let mut include = include;
                let name = next_possibly_quoted_string(&mut include);
                let include_path = PathBuf::from(replace_filename_with_new_path(
                    &path.to_string_lossy(),
                    name,
                ));
                if depth > 10 {
                    warnings.push(warning(
                        path,
                        line,
                        format!("unable to read {}", include_path.display()),
                    ));
                } else if let Ok(included) = load(&include_path) {
                    self.parse_text_inner(
                        &include_path,
                        &included,
                        depth + 1,
                        resolve_key,
                        load,
                        warnings,
                    );
                } else {
                    warnings.push(warning(
                        path,
                        line,
                        format!("unable to read {}", include_path.display()),
                    ));
                }
                continue;
            }
            if section == -2 {
                warnings.push(warning(path, line, "expecting [section]".into()));
                continue;
            }
            let Some((key, value)) = split_key_value(text) else {
                warnings.push(warning(path, line, "expecting 'key=value'".into()));
                continue;
            };
            if section >= 0 && !self.handle(section as u8, key, value, resolve_key) {
                warnings.push(warning(path, line, format!("can't parse '{key}'")));
            }
        }
    }

    fn handle<R>(&mut self, section: u8, key: &str, value: &str, resolve_key: &mut R) -> bool
    where
        R: FnMut(&str) -> Option<i32>,
    {
        match section {
            0 | 5 => {
                let Some((index, group)) = KEY_GROUPS
                    .iter()
                    .enumerate()
                    .find(|(_, group)| key.eq_ignore_ascii_case(group.name))
                else {
                    return false;
                };
                if section == 0 {
                    self.configured_key_groups.insert(index);
                    self.parse_key_array(value, *group, resolve_key);
                } else {
                    if index == 1 {
                        self.has_gamepad_controls = true;
                    }
                    self.parse_gamepad_array(value, *group);
                }
                true
            }
            1 => self.handle_graphics(key, value),
            2 => self.handle_sound(key, value),
            3 => self.handle_general(key, value),
            4 => self.handle_feature(key, value),
            _ => false,
        }
    }

    fn parse_key_array<R>(&mut self, value: &str, group: KeyGroup, resolve_key: &mut R)
    where
        R: FnMut(&str) -> Option<i32>,
    {
        let mut remaining = Some(value);
        let mut command = group.command;
        for _ in 0..group.size {
            let Some(mut name) = next_delim(&mut remaining, ',') else {
                break;
            };
            if !name.is_empty() {
                let mut modifiers = 0;
                loop {
                    if let Some(rest) = strip_ascii_prefix(name, "Shift+") {
                        modifiers |= KEY_MOD_SHIFT;
                        name = rest;
                    } else if let Some(rest) = strip_ascii_prefix(name, "Ctrl+") {
                        modifiers |= KEY_MOD_CTRL;
                        name = rest;
                    } else if let Some(rest) = strip_ascii_prefix(name, "Alt+") {
                        modifiers |= KEY_MOD_ALT;
                        name = rest;
                    } else {
                        break;
                    }
                }
                if let Some(key) = resolve_key(name) {
                    self.keyboard
                        .add(modifiers | remap_sdl_keycode(key), command);
                }
            }
            if command != 0 {
                command = command.wrapping_add(1);
            }
        }
    }

    fn parse_gamepad_array(&mut self, value: &str, group: KeyGroup) {
        let mut remaining = Some(value);
        let mut command = group.command;
        for _ in 0..group.size {
            let Some(text) = next_delim(&mut remaining, ',') else {
                break;
            };
            if !text.is_empty()
                && let Some((button, modifiers)) = parse_gamepad_combo(text)
            {
                self.gamepad.add(button, modifiers, command);
            }
            if command != 0 {
                command = command.wrapping_add(1);
            }
        }
    }

    fn handle_graphics(&mut self, key: &str, value: &str) -> bool {
        match_ascii!(key,
            "WindowSize" => {
                if value.eq_ignore_ascii_case("Auto") {
                    self.config.window_width = 0;
                    self.config.window_height = 0;
                } else {
                    for part in value.split('x') {
                        if self.config.window_width == 0 {
                            self.config.window_width = c_atoi(part);
                        } else {
                            self.config.window_height = c_atoi(part);
                            break;
                        }
                    }
                }
                true
            },
            "EnhancedMode7" => set_bool(value, &mut self.config.enhanced_mode7),
            "NewRenderer" => set_bool(value, &mut self.config.new_renderer),
            "IgnoreAspectRatio" => set_bool(value, &mut self.config.ignore_aspect_ratio),
            "Fullscreen" => { self.config.fullscreen = c_atoi(value) as u8; true },
            "WindowScale" => { self.config.window_scale = c_atoi(value) as u8; true },
            "OutputMethod" => {
                self.config.output_method = if value.eq_ignore_ascii_case("SDL-Software") {
                    OutputMethod::SdlSoftware
                } else if value.eq_ignore_ascii_case("OpenGL") {
                    OutputMethod::OpenGl
                } else if value.eq_ignore_ascii_case("OpenGL ES") {
                    OutputMethod::OpenGlEs
                } else { OutputMethod::Sdl };
                true
            },
            "LinearFiltering" => set_bool(value, &mut self.config.linear_filtering),
            "NoSpriteLimits" => set_bool(value, &mut self.config.no_sprite_limits),
            "LinkGraphics" => { self.config.link_graphics = Some(value.into()); true },
            "Shader" => { self.config.shader = (!value.is_empty()).then(|| value.into()); true },
            "DimFlashes" => set_bool_bit(value, &mut self.config.features0, FEATURE_DIM_FLASHES),
            _ => false
        )
    }

    fn handle_sound(&mut self, key: &str, value: &str) -> bool {
        match_ascii!(key,
            "EnableAudio" => set_bool(value, &mut self.config.enable_audio),
            "AudioFreq" => { self.config.audio_freq = c_atoi(value) as u16; true },
            "AudioChannels" => { self.config.audio_channels = c_atoi(value) as u8; true },
            "AudioSamples" => { self.config.audio_samples = c_atoi(value) as u16; true },
            "EnableMSU" => {
                if value.eq_ignore_ascii_case("opuz") { self.config.enable_msu = MSU_ENABLED_OPUZ; }
                else if value.eq_ignore_ascii_case("deluxe") { self.config.enable_msu = MSU_ENABLED_DELUXE; }
                else if value.eq_ignore_ascii_case("deluxe-opuz") { self.config.enable_msu = MSU_ENABLED_DELUXE | MSU_ENABLED_OPUZ; }
                else if let Some(enabled) = parse_bool(value) { self.config.enable_msu = u8::from(enabled); }
                else { return false; }
                true
            },
            "MSUPath" => { self.config.msu_path = Some(value.into()); true },
            "MSUVolume" => { self.config.msu_volume = c_atoi(value) as u8; true },
            "ResumeMSU" => set_bool(value, &mut self.config.resume_msu),
            _ => false
        )
    }

    fn handle_general(&mut self, key: &str, value: &str) -> bool {
        match_ascii!(key,
            "Autosave" => { self.config.autosave = c_atoi(value) != 0; true },
            "ExtendedAspectRatio" => self.parse_aspect_ratio(value),
            "DisplayPerfInTitle" => set_bool(value, &mut self.config.display_perf_title),
            "DisableFrameDelay" => set_bool(value, &mut self.config.disable_frame_delay),
            "Language" => { self.config.language = Some(value.into()); true },
            _ => false
        )
    }

    fn parse_aspect_ratio(&mut self, value: &str) -> bool {
        let mut height = 224_i32;
        let mut unchanged_sprites = false;
        let mut no_visual_fixes = false;
        let mut remaining = Some(value);
        while let Some(part) = next_delim(&mut remaining, ',') {
            match part {
                "extend_y" => {
                    height = 240;
                    self.config.extend_y = true;
                }
                "16:9" => self.config.extended_aspect_ratio = ((height * 16 / 9 - 256) / 2) as u8,
                "16:10" => self.config.extended_aspect_ratio = ((height * 16 / 10 - 256) / 2) as u8,
                "18:9" => self.config.extended_aspect_ratio = ((height * 18 / 9 - 256) / 2) as u8,
                "4:3" => self.config.extended_aspect_ratio = 0,
                "unchanged_sprites" => unchanged_sprites = true,
                "no_visual_fixes" => no_visual_fixes = true,
                _ => return false,
            }
        }
        if self.config.extended_aspect_ratio != 0 && !unchanged_sprites {
            self.config.features0 |= FEATURE_EXTEND_SCREEN_64;
        }
        if self.config.extended_aspect_ratio != 0 && !no_visual_fixes {
            self.config.features0 |= FEATURE_WIDESCREEN_VISUAL_FIXES;
        }
        true
    }

    fn handle_feature(&mut self, key: &str, value: &str) -> bool {
        let mask = match key.to_ascii_lowercase().as_str() {
            "itemswitchlr" => FEATURE_SWITCH_LR,
            "itemswitchlrlimit" => FEATURE_SWITCH_LR_LIMIT,
            "turnwhiledashing" => FEATURE_TURN_WHILE_DASHING,
            "mirrortodarkworld" => FEATURE_MIRROR_TO_DARKWORLD,
            "collectitemswithsword" => FEATURE_COLLECT_ITEMS_WITH_SWORD,
            "breakpotswithsword" => FEATURE_BREAK_POTS_WITH_SWORD,
            "disablelowhealthbeep" => FEATURE_DISABLE_LOW_HEALTH_BEEP,
            "skipintroonkeypress" => FEATURE_SKIP_INTRO_ON_KEYPRESS,
            "showmaxitemsinyellow" => FEATURE_SHOW_MAX_ITEMS_IN_YELLOW,
            "moreactivebombs" => FEATURE_MORE_ACTIVE_BOMBS,
            "carrymorerupees" => FEATURE_CARRY_MORE_RUPEES,
            "miscbugfixes" => FEATURE_MISC_BUG_FIXES,
            "gamechangingbugfixes" => FEATURE_GAME_CHANGING_BUG_FIXES,
            "cancelbirdtravel" => FEATURE_CANCEL_BIRD_TRAVEL,
            _ => return false,
        };
        set_bool_bit(value, &mut self.config.features0, mask)
    }

    /// Adds the exact C defaults for every key group omitted by the parsed
    /// configuration and for the twelve basic gamepad controls.
    pub fn register_defaults(&mut self) {
        for (index, group) in KEY_GROUPS.iter().enumerate().skip(1) {
            if self.configured_key_groups.contains(&index) {
                continue;
            }
            let mut command = group.command;
            for offset in 0..group.size {
                let key = default_keyboard_control(usize::from(command));
                if key != 0 {
                    self.keyboard.add(key, command);
                }
                if command != 0 {
                    command = command.wrapping_add(1);
                }
                if offset + 1 == group.size {
                    break;
                }
            }
        }
        if !self.has_gamepad_controls {
            let defaults = [
                GamepadButton::DpadUp,
                GamepadButton::DpadDown,
                GamepadButton::DpadLeft,
                GamepadButton::DpadRight,
                GamepadButton::Back,
                GamepadButton::Start,
                GamepadButton::B,
                GamepadButton::A,
                GamepadButton::Y,
                GamepadButton::X,
                GamepadButton::L1,
                GamepadButton::R1,
            ];
            for (offset, button) in defaults.into_iter().enumerate() {
                self.gamepad.add(button, 0, CONTROLS + offset as u16);
            }
        }
    }
}

/// C's public file-level fallback: explicit paths are tried once; the default
/// path first tries `zelda3.user.ini`, then `zelda3.ini`.
pub fn parse_config_file<R>(
    path: Option<&Path>,
    mut resolve_key: R,
) -> (ConfigState, Vec<ConfigWarning>)
where
    R: FnMut(&str) -> Option<i32>,
{
    let mut state = ConfigState::default();
    let mut warnings = Vec::new();
    let (selected, contents) = if let Some(path) = path {
        (path.to_path_buf(), fs::read_to_string(path))
    } else {
        let user = PathBuf::from("zelda3.user.ini");
        match fs::read_to_string(&user) {
            Ok(contents) => (user, Ok(contents)),
            Err(_) => {
                let base = PathBuf::from("zelda3.ini");
                let contents = fs::read_to_string(&base);
                (base, contents)
            }
        }
    };
    match contents {
        Ok(text) => {
            let mut loader = |path: &Path| fs::read_to_string(path);
            warnings.extend(state.parse_text(&selected, &text, &mut resolve_key, &mut loader));
        }
        Err(_) => warnings.push(warning(
            &selected,
            0,
            format!("unable to read config file {}", selected.display()),
        )),
    }
    state.register_defaults();
    (state, warnings)
}

macro_rules! match_ascii {
    ($value:expr, $($pattern:literal => $result:expr,)* _ => $fallback:expr) => {{
        $(if $value.eq_ignore_ascii_case($pattern) { $result } else)* { $fallback }
    }};
}
use match_ascii;

fn warning(path: &Path, line: usize, message: String) -> ConfigWarning {
    ConfigWarning {
        path: path.into(),
        line,
        message,
    }
}

fn strip_ascii_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let head = value.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}

fn set_bool(value: &str, output: &mut bool) -> bool {
    if let Some(parsed) = parse_bool(value) {
        *output = parsed;
        true
    } else {
        false
    }
}

fn set_bool_bit(value: &str, output: &mut u32, mask: u32) -> bool {
    if let Some(parsed) = parse_bool(value) {
        *output = (*output & !mask) | if parsed { mask } else { 0 };
        true
    } else {
        false
    }
}

fn c_atoi(value: &str) -> i32 {
    let bytes = value.as_bytes();
    let mut index = 0;
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    let negative = bytes.get(index) == Some(&b'-');
    if negative || bytes.get(index) == Some(&b'+') {
        index += 1;
    }
    let mut result = 0_i32;
    while let Some(digit) = bytes
        .get(index)
        .and_then(|byte| byte.is_ascii_digit().then(|| byte - b'0'))
    {
        result = result.wrapping_mul(10).wrapping_add(i32::from(digit));
        index += 1;
    }
    if negative {
        result.wrapping_neg()
    } else {
        result
    }
}

fn parse_gamepad_button(value: &str) -> Option<(GamepadButton, &str)> {
    // Longest substrings first, exactly as the C table.
    const NAMES: [(&str, GamepadButton); 19] = [
        ("Back", GamepadButton::Back),
        ("Guide", GamepadButton::Guide),
        ("Start", GamepadButton::Start),
        ("L3", GamepadButton::L3),
        ("R3", GamepadButton::R3),
        ("L1", GamepadButton::L1),
        ("R1", GamepadButton::R1),
        ("DpadUp", GamepadButton::DpadUp),
        ("DpadDown", GamepadButton::DpadDown),
        ("DpadLeft", GamepadButton::DpadLeft),
        ("DpadRight", GamepadButton::DpadRight),
        ("L2", GamepadButton::L2),
        ("R2", GamepadButton::R2),
        ("Lb", GamepadButton::L1),
        ("Rb", GamepadButton::R1),
        ("A", GamepadButton::A),
        ("B", GamepadButton::B),
        ("X", GamepadButton::X),
        ("Y", GamepadButton::Y),
    ];
    NAMES.iter().find_map(|&(name, button)| {
        strip_ascii_prefix(value, name).map(|remaining| (button, remaining))
    })
}

fn parse_gamepad_combo(value: &str) -> Option<(GamepadButton, u32)> {
    let mut value = value;
    let mut modifiers = 0_u32;
    loop {
        let (button, rest) = parse_gamepad_button(value)?;
        let rest = rest.trim_start_matches([' ', '\t']);
        if let Some(rest) = rest.strip_prefix('+') {
            modifiers |= 1 << button as u8;
            value = rest;
        } else if rest.is_empty() {
            return Some((button, modifiers));
        } else {
            return None;
        }
    }
}

fn default_keyboard_control(command: usize) -> u16 {
    const SC: u16 = KEY_MOD_SCANCODE;
    const SHIFT: u16 = KEY_MOD_SHIFT;
    const ALT: u16 = KEY_MOD_ALT;
    const CTRL: u16 = KEY_MOD_CTRL;
    match command {
        1..=4 => SC + [82, 81, 80, 79][command - 1],
        5 => SC + 229, // right shift
        6 => 13,
        7 => b'x' as u16,
        8 => b'z' as u16,
        9 => b's' as u16,
        10 => b'a' as u16,
        11 => b'c' as u16,
        12 => b'v' as u16,
        13..=22 => SC + 58 + (command - 13) as u16,
        33..=42 => SHIFT | (SC + 58 + (command - 33) as u16),
        53..=62 => CTRL | (SC + 58 + (command - 53) as u16),
        113 => b'w' as u16,
        114 => b'o' as u16,
        115 => SHIFT | b'w' as u16,
        116 => CTRL | b'e' as u16,
        117 => b'k' as u16,
        118 => b'l' as u16,
        119 => ALT | 13,
        120 => CTRL | b'r' as u16,
        121 => SHIFT | b'p' as u16,
        122 => b'p' as u16,
        123 => b'\t' as u16,
        124 => b't' as u16,
        127 => b'f' as u16,
        128 => b'r' as u16,
        _ => 0,
    }
}

fn section_id(value: &str) -> Option<i8> {
    [
        "[KeyMap]",
        "[Graphics]",
        "[Sound]",
        "[General]",
        "[Features]",
        "[GamepadMap]",
    ]
    .iter()
    .position(|section| value.eq_ignore_ascii_case(section))
    .map(|index| index as i8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn booleans_match_all_c_spellings() {
        for value in ["0", "false", "NO", "Off"] {
            assert_eq!(parse_bool(value), Some(false));
        }
        for value in ["1", "true", "YES", "On"] {
            assert_eq!(parse_bool(value), Some(true));
        }
        assert_eq!(parse_bool("2"), None);
        assert_eq!(parse_bool("true "), None);
    }

    #[test]
    fn atoi_stops_at_the_first_non_digit_without_evaluating_it() {
        assert_eq!(c_atoi("42,rest"), 42);
        assert_eq!(c_atoi(" -17px"), -17);
        assert_eq!(c_atoi("word"), 0);
    }

    #[test]
    fn gamepad_specific_modifiers_precede_generic_entries() {
        let mut map = GamepadMap::default();
        map.add(GamepadButton::A, 0, 1);
        map.add(GamepadButton::A, 1 << GamepadButton::L1 as u8, 2);
        assert_eq!(map.find(GamepadButton::A, 0), 1);
        assert_eq!(map.find(GamepadButton::A, 1 << GamepadButton::L1 as u8), 2);
    }

    #[test]
    fn parser_preserves_order_dependent_extended_aspect_ratio() {
        let mut state = ConfigState::default();
        let mut resolver = |_: &str| None;
        let mut loader = |_: &Path| -> io::Result<String> {
            Err(io::Error::new(io::ErrorKind::NotFound, "test"))
        };
        let warnings = state.parse_text(
            Path::new("zelda3.ini"),
            "[General]\nExtendedAspectRatio=extend_y,16:9\n[Features]\nItemSwitchLR=yes\n",
            &mut resolver,
            &mut loader,
        );
        assert!(warnings.is_empty());
        assert_eq!(state.config.extended_aspect_ratio, 85);
        assert!(state.config.extend_y);
        assert_eq!(
            state.config.features0,
            FEATURE_EXTEND_SCREEN_64 | FEATURE_WIDESCREEN_VISUAL_FIXES | FEATURE_SWITCH_LR
        );
    }

    #[test]
    fn key_arrays_apply_modifiers_and_skip_empty_slots() {
        let mut state = ConfigState::default();
        let mut resolver = |name: &str| match name {
            "X" => Some(b'x' as i32),
            _ => None,
        };
        let mut loader = |_: &Path| -> io::Result<String> {
            Err(io::Error::new(io::ErrorKind::NotFound, "test"))
        };
        state.parse_text(
            Path::new("z.ini"),
            "[KeyMap]\nControls=Shift+X,,X\n",
            &mut resolver,
            &mut loader,
        );
        assert_eq!(state.keyboard.find(KEY_MOD_SHIFT | b'x' as u16), 1);
        assert_eq!(state.keyboard.find(b'x' as u16), 3);
    }

    #[test]
    fn include_path_and_gamepad_combo_match_c_parser() {
        let mut state = ConfigState::default();
        let mut resolver = |_: &str| None;
        let mut loader = |path: &Path| {
            assert_eq!(path, Path::new("cfg/keys.ini"));
            Ok("[GamepadMap]\nControls=L1+A,B\n".into())
        };
        let warnings = state.parse_text(
            Path::new("cfg/main.ini"),
            "!include \"keys.ini\"\n",
            &mut resolver,
            &mut loader,
        );
        assert!(warnings.is_empty());
        assert_eq!(
            state
                .gamepad
                .find(GamepadButton::A, 1 << GamepadButton::L1 as u8),
            1
        );
        assert_eq!(state.gamepad.find(GamepadButton::B, 0), 2);
    }

    #[test]
    fn defaults_keep_exact_command_indices_and_sdl_scancodes() {
        let mut state = ConfigState::default();
        state.register_defaults();
        assert_eq!(state.keyboard.find(KEY_MOD_SCANCODE | 82), 1);
        assert_eq!(
            state.keyboard.find(KEY_MOD_SHIFT | KEY_MOD_SCANCODE | 58),
            33
        );
        assert_eq!(state.keyboard.find(KEY_MOD_CTRL | b'r' as u16), 120);
        assert_eq!(state.gamepad.find(GamepadButton::B, 0), 7);
    }
}
