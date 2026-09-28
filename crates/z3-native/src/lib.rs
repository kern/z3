//! SDL2 native-host policy translated from `src/main.c`.

pub mod native_app;

use std::time::{Duration, Instant};
use z3_core::config::{ConfigState, GamepadButton, SdlKeyConstants, find_command_for_sdl_key};
use z3_game::host::{HostAction, InputState, render_number};
use z3_game::runtime::{NativeRuntime, StateRecorder, apply_patch_command};
use z3_sdl::{GameController, Rect, Renderer, Sdl, TextureLock, WgpuRenderer, Window};

pub const WINDOW_TITLE: &str = "The Legend of Zelda: A Link to the Past";

pub fn zelda_apu_lock<T>(shared: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn zelda_apu_unlock<T>(guard: std::sync::MutexGuard<'_, T>) {
    drop(guard);
}

#[must_use]
pub fn hit_test_callback(
    width: i32,
    height: i32,
    point: z3_sdl::Point,
    fullscreen: bool,
    ctrl: bool,
) -> z3_sdl::HitTestResult {
    z3_sdl::hit_test(width, height, point, fullscreen, ctrl)
}

pub fn sdl_renderer_init(
    window: &Window,
    width: i32,
    height: i32,
    software: bool,
    keep_aspect: bool,
    linear: bool,
    texture_scale: i32,
) -> Result<Renderer, z3_sdl::Error> {
    Renderer::new(
        window,
        width,
        height,
        software,
        keep_aspect,
        linear,
        texture_scale,
    )
}

pub fn sdl_renderer_destroy(renderer: Renderer) {
    drop(renderer);
}

pub fn sdl_renderer_begin_draw(
    renderer: &mut Renderer,
    width: i32,
    height: i32,
) -> Result<TextureLock<'_>, z3_sdl::Error> {
    renderer.begin_draw(width, height)
}

pub fn sdl_renderer_end_draw(lock: TextureLock<'_>) -> Result<(), z3_sdl::Error> {
    lock.present()
}

pub fn draw_ppu_frame(
    runtime: &mut NativeRuntime,
    renderer: &mut Renderer,
    snes_width: i32,
    snes_height: i32,
    render_flags: u8,
) -> Result<(), z3_sdl::Error> {
    draw_ppu_frame_with_perf(
        runtime,
        renderer,
        snes_width,
        snes_height,
        render_flags,
        None,
        false,
    )
}

#[derive(Debug)]
pub struct RenderPerformance {
    history: [f64; 64],
    average: f64,
    position: usize,
    pub frames_per_second: i32,
}

impl Default for RenderPerformance {
    fn default() -> Self {
        Self {
            history: [0.0; 64],
            average: 0.0,
            position: 0,
            frames_per_second: 0,
        }
    }
}

impl RenderPerformance {
    fn record(&mut self, elapsed: Duration) {
        let sample = 1.0 / elapsed.as_secs_f64().max(f64::EPSILON);
        self.average += sample - self.history[self.position];
        self.history[self.position] = sample;
        self.position = (self.position + 1) & 63;
        self.frames_per_second = (self.average * (1.0 / 64.0)) as i32;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ppu_frame_with_perf(
    runtime: &mut NativeRuntime,
    renderer: &mut Renderer,
    snes_width: i32,
    snes_height: i32,
    render_flags: u8,
    mut performance: Option<&mut RenderPerformance>,
    overlay: bool,
) -> Result<(), z3_sdl::Error> {
    let scale = runtime.ppu.current_render_scale(render_flags) as i32;
    let mut texture = sdl_renderer_begin_draw(renderer, snes_width * scale, snes_height * scale)?;
    let before = Instant::now();
    runtime.draw(texture.pitch(), render_flags, |_| {});
    if let Some(performance) = performance.as_mut() {
        performance.record(before.elapsed());
        if overlay {
            let pitch = texture.pitch();
            let offset = pitch * scale as usize;
            render_number(
                &mut runtime.ppu.pixels_mut()[offset..],
                pitch,
                performance.frames_per_second,
                scale == 4,
            );
        }
    }
    let source = runtime.ppu.pixels();
    let destination = texture.pixels_mut();
    destination[..source.len()].copy_from_slice(source);
    sdl_renderer_end_draw(texture)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ppu_frame_wgpu_with_perf(
    runtime: &mut NativeRuntime,
    renderer: &mut WgpuRenderer<'_>,
    window: &Window,
    snes_width: i32,
    snes_height: i32,
    render_flags: u8,
    mut performance: Option<&mut RenderPerformance>,
    overlay: bool,
) -> Result<(), z3_sdl::Error> {
    let scale = runtime.ppu.current_render_scale(render_flags) as i32;
    let pitch = renderer.begin_draw((snes_width * scale) as u32, (snes_height * scale) as u32);
    let before = Instant::now();
    runtime.draw(pitch, render_flags, |_| {});
    if let Some(performance) = performance.as_mut() {
        performance.record(before.elapsed());
        if overlay {
            let offset = pitch * scale as usize;
            render_number(
                &mut runtime.ppu.pixels_mut()[offset..],
                pitch,
                performance.frames_per_second,
                scale == 4,
            );
        }
    }
    renderer.pixels_mut().copy_from_slice(runtime.ppu.pixels());
    renderer.present(window)
}

pub const SDL_KEY_CONSTANTS: SdlKeyConstants = SdlKeyConstants {
    alt_mask: z3_sdl::KMOD_ALT,
    ctrl_mask: z3_sdl::KMOD_CTRL,
    shift_mask: z3_sdl::KMOD_SHIFT,
    left_alt: 0x4000_00e2,
    right_alt: 0x4000_00e6,
    left_ctrl: 0x4000_00e0,
    right_ctrl: 0x4000_00e4,
    left_shift: 0x4000_00e1,
    right_shift: 0x4000_00e5,
};

pub fn handle_input(
    input: &mut InputState,
    config: &ConfigState,
    key_code: i32,
    key_modifiers: u16,
    pressed: bool,
) -> Option<HostAction> {
    let command =
        find_command_for_sdl_key(&config.keyboard, key_code, key_modifiers, SDL_KEY_CONSTANTS);
    (command != 0)
        .then(|| input.handle_command(command, pressed))
        .flatten()
}

pub fn handle_gamepad_input(
    input: &mut InputState,
    config: &ConfigState,
    button: usize,
    pressed: bool,
) -> Option<HostAction> {
    let logical = gamepad_button(button)?;
    input.handle_gamepad_button(button, pressed, |_, modifiers| {
        config.gamepad.find(logical, modifiers)
    })
}

pub fn handle_gamepad_axis_input(
    input: &mut InputState,
    config: &ConfigState,
    gamepad_id: i32,
    axis: u8,
    value: i16,
) -> Option<HostAction> {
    match axis {
        z3_sdl::CONTROLLER_AXIS_LEFT_X => {
            input.handle_stick_axis(gamepad_id, true, i32::from(value));
            None
        }
        z3_sdl::CONTROLLER_AXIS_LEFT_Y => {
            input.handle_stick_axis(gamepad_id, false, i32::from(value));
            None
        }
        z3_sdl::CONTROLLER_AXIS_TRIGGER_LEFT | z3_sdl::CONTROLLER_AXIS_TRIGGER_RIGHT
            if !(12_000..16_000).contains(&value) =>
        {
            let button = if axis == z3_sdl::CONTROLLER_AXIS_TRIGGER_LEFT {
                15
            } else {
                16
            };
            handle_gamepad_input(input, config, button, value >= 12_000)
        }
        _ => None,
    }
}

pub fn open_one_gamepad(
    sdl: &Sdl,
    controllers: &mut Vec<GameController>,
    index: i32,
) -> Result<bool, z3_sdl::Error> {
    if let Some(controller) = sdl.open_game_controller(index)? {
        controllers.push(controller);
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn save_load_slot(
    runtime: &mut NativeRuntime,
    kind: z3_game::host::SaveLoadKind,
    slot: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = z3_game::host::save_slot_path(kind, slot).ok_or("reference saves are read-only")?;
    match kind {
        z3_game::host::SaveLoadKind::Save => {
            let snapshot = runtime.save_snes_state()?;
            let file = runtime.recorder.encode_state_file(&snapshot);
            std::fs::write(path, file)?;
        }
        z3_game::host::SaveLoadKind::Load | z3_game::host::SaveLoadKind::Replay => {
            let file = std::fs::read(path)?;
            let replay = kind == z3_game::host::SaveLoadKind::Replay;
            let (recorder, snapshot) = StateRecorder::decode_state_file(&file, replay)
                .map_err(|error| format!("invalid state file: {error:?}"))?;
            runtime.recorder = recorder;
            if snapshot.is_empty() {
                runtime.reset(false);
            } else {
                runtime.load_snes_state(&snapshot)?;
            }
        }
    }
    Ok(())
}

pub fn apply_runtime_command(
    runtime: &mut NativeRuntime,
    command: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch = match command {
        113 => Some('w'),
        114 => Some('o'),
        115 => Some('W'),
        116 => Some('E'),
        117 => Some('k'),
        118 => Some('l'),
        _ => None,
    };
    if let Some(patch) = patch {
        let snapshot = runtime.save_snes_state()?;
        apply_patch_command(
            &mut z3_game::ram::Ram::new(&mut runtime.wram),
            &mut runtime.recorder,
            patch,
            &snapshot,
        );
    } else if command == 120 {
        runtime.reset(true);
    }
    Ok(())
}

fn gamepad_button(button: usize) -> Option<GamepadButton> {
    Some(match button {
        0 => GamepadButton::A,
        1 => GamepadButton::B,
        2 => GamepadButton::X,
        3 => GamepadButton::Y,
        4 => GamepadButton::Back,
        5 => GamepadButton::Guide,
        6 => GamepadButton::Start,
        7 => GamepadButton::L3,
        8 => GamepadButton::R3,
        9 => GamepadButton::L1,
        10 => GamepadButton::R1,
        11 => GamepadButton::DpadUp,
        12 => GamepadButton::DpadDown,
        13 => GamepadButton::DpadLeft,
        14 => GamepadButton::DpadRight,
        15 => GamepadButton::L2,
        16 => GamepadButton::R2,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowPlacement {
    pub scale: u8,
    pub width: i32,
    pub height: i32,
    pub x: Option<i32>,
    pub y: Option<i32>,
}

/// Exact arithmetic from C `ChangeWindowScale`, separated for deterministic tests.
#[must_use]
pub fn change_window_scale_geometry(
    current_scale: u8,
    scale_step: i32,
    snes_width: i32,
    snes_height: i32,
    usable: Option<Rect>,
    borders: (i32, i32, i32, i32),
    mouse: (i32, i32),
) -> WindowPlacement {
    let (top, left, bottom, right) = borders;
    let max_scale = usable.map_or(10, |bounds| {
        let horizontal = (bounds.w - left - right + snes_width / 4) / snes_width;
        let vertical = (bounds.h - top - bottom + snes_height / 4) / snes_height;
        horizontal.min(vertical)
    });
    let scale = (i32::from(current_scale) + scale_step)
        .min(max_scale)
        .max(1) as u8;
    let width = i32::from(scale) * snes_width;
    let height = i32::from(scale) * snes_height;
    let (x, y) = usable.map_or((None, None), |bounds| {
        (
            Some(
                (mouse.0 - width / 2)
                    .min(bounds.x + bounds.w - left - right - width)
                    .max(bounds.x + left),
            ),
            Some(
                (mouse.1 - height / 2)
                    .min(bounds.y + bounds.h - top - bottom - height)
                    .max(bounds.y + top),
            ),
        )
    });
    WindowPlacement {
        scale,
        width,
        height,
        x,
        y,
    }
}

pub fn change_window_scale(
    window: &Window,
    current_scale: &mut u8,
    scale_step: i32,
    snes_width: i32,
    snes_height: i32,
) {
    let flags = window.flags();
    if flags
        & (z3_sdl::WINDOW_FULLSCREEN_DESKTOP
            | z3_sdl::WINDOW_FULLSCREEN
            | z3_sdl::WINDOW_MINIMIZED
            | z3_sdl::WINDOW_MAXIMIZED)
        != 0
    {
        return;
    }
    let usable = window.display_usable_bounds().ok();
    let borders = window.border_sizes().unwrap_or((31, 1, 1, 1));
    let placement = change_window_scale_geometry(
        *current_scale,
        scale_step,
        snes_width,
        snes_height,
        usable,
        borders,
        window.global_mouse_position(),
    );
    *current_scale = placement.scale;
    window.set_size(placement.width, placement.height);
    window.set_position(
        placement.x.unwrap_or(z3_sdl::WINDOWPOS_CENTERED),
        placement.y.unwrap_or(z3_sdl::WINDOWPOS_CENTERED),
    );
}

#[must_use]
pub fn audio_frames_per_block(frequency: i32) -> usize {
    (534 * frequency as usize) / 32_000
}

#[derive(Debug)]
pub struct AudioCallbackBuffer {
    samples: Vec<i16>,
    position: usize,
    channels: usize,
    frames_per_block: usize,
    mixer_volume: i32,
}

impl AudioCallbackBuffer {
    #[must_use]
    pub fn new(frequency: i32, channels: usize) -> Self {
        let frames_per_block = audio_frames_per_block(frequency);
        Self {
            samples: vec![0; frames_per_block * channels],
            position: frames_per_block * channels,
            channels,
            frames_per_block,
            mixer_volume: z3_sdl::MIX_MAX_VOLUME,
        }
    }

    pub fn set_mixer_volume(&mut self, volume: i32) {
        self.mixer_volume = volume.clamp(0, z3_sdl::MIX_MAX_VOLUME);
    }

    pub fn handle_volume_adjustment(&mut self, adjustment: i32) -> i32 {
        self.mixer_volume = z3_sdl::adjusted_mixer_volume(self.mixer_volume, adjustment);
        self.mixer_volume
    }

    pub fn audio_callback(
        &mut self,
        output: &mut [i16],
        mut render: impl FnMut(&mut [i16], usize, usize),
        mut discard: impl FnMut(),
    ) {
        let mut written = 0;
        while written < output.len() {
            if self.position == self.samples.len() {
                render(&mut self.samples, self.frames_per_block, self.channels);
                self.position = 0;
            }
            let count = (output.len() - written).min(self.samples.len() - self.position);
            let destination = &mut output[written..written + count];
            destination.fill(0);
            z3_sdl::mix_audio_s16(
                destination,
                &self.samples[self.position..self.position + count],
                self.mixer_volume,
            );
            self.position += count;
            written += count;
        }
        discard();
    }

    pub fn render_game_audio(
        &mut self,
        output: &mut [i16],
        audio: &mut z3_game::audio::AudioRuntime,
    ) {
        self.audio_callback(
            output,
            |samples, frames, channels| audio.render_audio(samples, frames, channels),
            || {},
        );
        audio.discard_unused_audio_frames();
    }
}

pub fn die(error: &str) -> ! {
    eprintln!("Error: {error}");
    std::process::exit(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_scale_uses_screen_fit_slack_and_mouse_centering() {
        let placement = change_window_scale_geometry(
            2,
            10,
            256,
            224,
            Some(Rect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1080,
            }),
            (31, 1, 1, 1),
            (1000, 500),
        );
        assert_eq!(placement.scale, 4);
        assert_eq!((placement.width, placement.height), (1024, 896));
        assert_eq!((placement.x, placement.y), (Some(488), Some(52)));

        let tiny = change_window_scale_geometry(
            2,
            1,
            256,
            224,
            Some(Rect {
                x: 0,
                y: 0,
                w: 100,
                h: 100,
            }),
            (31, 1, 1, 1),
            (0, 0),
        );
        assert_eq!(tiny.scale, 1);
        assert_eq!((tiny.x, tiny.y), (Some(1), Some(31)));
    }

    #[test]
    fn callback_refills_blocks_and_keeps_partial_tail() {
        let mut callback = AudioCallbackBuffer::new(32_000, 2);
        assert_eq!(callback.frames_per_block, 534);
        let mut renders = 0;
        let mut discarded = 0;
        let mut output = vec![0; 1_100];
        callback.audio_callback(
            &mut output,
            |samples, _, _| {
                renders += 1;
                samples.fill(renders);
            },
            || discarded += 1,
        );
        assert_eq!(renders, 2);
        assert!(output[..1_068].iter().all(|&sample| sample == 1));
        assert!(output[1_068..].iter().all(|&sample| sample == 2));
        assert_eq!(discarded, 1);
    }

    #[test]
    fn native_runtime_can_be_owned_by_the_sdl_audio_mutex() {
        fn assert_send<T: Send>() {}
        assert_send::<NativeRuntime>();
    }

    #[test]
    fn trigger_axis_preserves_the_c_hysteresis_band() {
        let mut config = ConfigState::default();
        config.gamepad.add(GamepadButton::L2, 0, 1);
        let mut input = InputState::default();
        handle_gamepad_axis_input(
            &mut input,
            &config,
            7,
            z3_sdl::CONTROLLER_AXIS_TRIGGER_LEFT,
            14_000,
        );
        assert_eq!(input.keyboard_buttons, 0);
        handle_gamepad_axis_input(
            &mut input,
            &config,
            7,
            z3_sdl::CONTROLLER_AXIS_TRIGGER_LEFT,
            16_000,
        );
        assert_eq!(input.keyboard_buttons, 1 << 4);
        handle_gamepad_axis_input(
            &mut input,
            &config,
            7,
            z3_sdl::CONTROLLER_AXIS_TRIGGER_LEFT,
            13_000,
        );
        assert_eq!(input.keyboard_buttons, 1 << 4);
        handle_gamepad_axis_input(
            &mut input,
            &config,
            7,
            z3_sdl::CONTROLLER_AXIS_TRIGGER_LEFT,
            11_999,
        );
        assert_eq!(input.keyboard_buttons, 0);
    }
}
