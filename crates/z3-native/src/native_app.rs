//! Interactive application loop. This becomes the `z3` binary once the
//! production `NativeGameServices` hook implementations are linked.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use z3_core::config::{ConfigState, OutputMethod, parse_config_file};
use z3_game::assets::RuntimeAssets;
use z3_game::host::{HostAction, InputState, SaveLoadKind, normalize_host_config};
use z3_game::integration::{NativeGameServices, NativeRuntimeIntegratedExt};
use z3_game::runtime::NativeRuntime;
use z3_sdl::{Event, Renderer, Sdl, Window};

use crate::{
    AudioCallbackBuffer, WINDOW_TITLE, apply_runtime_command, change_window_scale, draw_ppu_frame,
    handle_gamepad_axis_input, handle_gamepad_input, handle_input, open_one_gamepad,
    save_load_slot, sdl_renderer_init, zelda_apu_lock, zelda_apu_unlock,
};

const COMMAND_FULLSCREEN: u16 = 119;
const COMMAND_PAUSE: u16 = 121;
const COMMAND_PAUSE_DIMMED: u16 = 122;
const COMMAND_REPLAY_TURBO: u16 = 124;
const COMMAND_WINDOW_BIGGER: u16 = 125;
const COMMAND_WINDOW_SMALLER: u16 = 126;
const COMMAND_DISPLAY_PERF: u16 = 127;
const COMMAND_TOGGLE_RENDERER: u16 = 128;
const COMMAND_VOLUME_UP: u16 = 129;
const COMMAND_VOLUME_DOWN: u16 = 130;

#[derive(Debug)]
struct Options {
    config: Option<PathBuf>,
    rom: PathBuf,
    frames: Option<u32>,
}

fn options() -> Result<Options, Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let mut config = None;
    let mut rom = None;
    let mut frames = None;
    while let Some(argument) = arguments.next() {
        if argument == "--config" {
            config = Some(PathBuf::from(
                arguments.next().ok_or("--config requires a path")?,
            ));
        } else if argument == "--frames" {
            let count = arguments
                .next()
                .ok_or("--frames requires a count")?
                .to_string_lossy()
                .parse()?;
            if count == 0 {
                return Err("--frames must be greater than zero".into());
            }
            frames = Some(count);
        } else if rom.replace(PathBuf::from(argument)).is_some() {
            return Err("usage: z3 [--config FILE] [--frames COUNT] [ROM]".into());
        }
    }
    Ok(Options {
        config,
        rom: rom.unwrap_or_else(|| PathBuf::from("zelda3.sfc")),
        frames,
    })
}

#[derive(Debug)]
struct UiState {
    paused: bool,
    replay_turbo: bool,
    cursor: bool,
    borderless: bool,
    display_perf: bool,
    render_flags: u8,
    window_scale: u8,
}

#[allow(clippy::too_many_arguments)]
fn execute_action(
    action: HostAction,
    runtime: &Arc<Mutex<NativeRuntime>>,
    audio_buffer: &Arc<Mutex<Option<AudioCallbackBuffer>>>,
    window: &Window,
    state: &mut UiState,
    snes_width: i32,
    snes_height: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        HostAction::SaveLoad { kind, slot } => {
            let mut runtime = zelda_apu_lock(runtime);
            let result = save_load_slot(&mut runtime, kind, slot);
            zelda_apu_unlock(runtime);
            result?;
        }
        HostAction::Command(command) => match command {
            COMMAND_FULLSCREEN => {
                let fullscreen = window.flags() & z3_sdl::WINDOW_FULLSCREEN_DESKTOP == 0;
                window.set_fullscreen_desktop(fullscreen)?;
                state.cursor = !state.cursor;
                window.show_cursor(state.cursor);
            }
            COMMAND_PAUSE | COMMAND_PAUSE_DIMMED => state.paused = !state.paused,
            COMMAND_REPLAY_TURBO => state.replay_turbo = !state.replay_turbo,
            COMMAND_WINDOW_BIGGER => {
                change_window_scale(window, &mut state.window_scale, 1, snes_width, snes_height)
            }
            COMMAND_WINDOW_SMALLER => {
                change_window_scale(window, &mut state.window_scale, -1, snes_width, snes_height)
            }
            COMMAND_DISPLAY_PERF => state.display_perf = !state.display_perf,
            COMMAND_TOGGLE_RENDERER => state.render_flags ^= z3_snes::ppu::RENDER_NEW,
            COMMAND_VOLUME_UP | COMMAND_VOLUME_DOWN => {
                if let Some(buffer) = audio_buffer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .as_mut()
                {
                    let direction = if command == COMMAND_VOLUME_UP { 1 } else { -1 };
                    let volume = buffer.handle_volume_adjustment(direction);
                    println!("[SDL mixer volume]={volume}");
                }
            }
            _ => {
                let mut runtime = zelda_apu_lock(runtime);
                let result = apply_runtime_command(&mut runtime, command);
                zelda_apu_unlock(runtime);
                result?;
            }
        },
    }
    Ok(())
}

fn load_pcm(prefix: &Path, track: u8) -> Option<z3_game::audio::PcmTrack> {
    z3_game::audio::PcmTrack::open(z3_game::audio::msu_track_path(prefix, track, false)).ok()
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = options()?;
    if options.config.is_none()
        && let Some(directory) = z3_game::host::find_project_directory(&std::env::current_dir()?)
    {
        std::env::set_current_dir(directory)?;
    }

    let sdl = Sdl::init(z3_sdl::INIT_VIDEO | z3_sdl::INIT_AUDIO | z3_sdl::INIT_GAME_CONTROLLER)?;
    let (mut config_state, warnings): (ConfigState, _) =
        parse_config_file(options.config.as_deref(), z3_sdl::key_from_name);
    for warning in warnings {
        eprintln!(
            "{}:{}: {}",
            warning.path.display(),
            warning.line,
            warning.message
        );
    }
    let window_scale = normalize_host_config(&mut config_state.config);
    let config = &config_state.config;
    if matches!(
        config.output_method,
        OutputMethod::OpenGl | OutputMethod::OpenGlEs
    ) {
        return Err("this Rust native binary currently provides the SDL renderer backend".into());
    }
    if config.shader.is_some() {
        eprintln!("Warning: Shaders are supported only with the OpenGL backend");
    }

    let mut asset_bytes = z3_game::host::load_asset_bundle(
        Path::new("zelda3_assets.dat"),
        Path::new("zelda3_assets.bps"),
        &options.rom,
    )?;
    let mut gloves_color = z3_game::load_gfx::GLOVES_COLOR;
    if let Some(link_graphics) = config.link_graphics.as_deref() {
        eprintln!("Loading Link Graphics: {link_graphics}");
        let (graphics_range, palette_range) = {
            let file = z3_core::assets::AssetFile::parse(&asset_bytes)?;
            let base = asset_bytes.as_ptr() as usize;
            let range = |index| {
                let asset = file.get(index).expect("validated runtime asset index");
                let start = asset.as_ptr() as usize - base;
                start..start + asset.len()
            };
            (range(57), range(81))
        };
        let zspr = std::fs::read(link_graphics)?;
        let mut graphics = asset_bytes[graphics_range.clone()].to_vec();
        let mut palette = asset_bytes[palette_range.clone()].to_vec();
        let mut glove_bytes = [
            gloves_color[0] as u8,
            (gloves_color[0] >> 8) as u8,
            gloves_color[1] as u8,
            (gloves_color[1] >> 8) as u8,
        ];
        z3_game::host::parse_link_graphics(&zspr, &mut graphics, &mut palette, &mut glove_bytes)
            .map_err(|error| std::io::Error::other(format!("invalid Link graphics: {error:?}")))?;
        asset_bytes[graphics_range].copy_from_slice(&graphics);
        asset_bytes[palette_range].copy_from_slice(&palette);
        gloves_color = [
            u16::from_le_bytes([glove_bytes[0], glove_bytes[1]]),
            u16::from_le_bytes([glove_bytes[2], glove_bytes[3]]),
        ];
    }
    let assets = RuntimeAssets::parse_with_gloves(&asset_bytes, gloves_color)?;
    let messages = assets.messaging(config.language.as_deref());
    let messaging_assets = messages.as_assets();
    let snes_width = i32::from(config.extended_aspect_ratio) * 2 + 256;
    let snes_height = if config.extend_y { 240 } else { 224 };
    let custom_size = config.window_width != 0 && config.window_height != 0;
    let window_width = if custom_size {
        config.window_width
    } else {
        i32::from(window_scale) * snes_width
    };
    let window_height = if custom_size {
        config.window_height
    } else {
        i32::from(window_scale) * snes_height
    };
    let mut window_flags = z3_sdl::WINDOW_RESIZABLE;
    window_flags |= match config.fullscreen {
        1 => z3_sdl::WINDOW_FULLSCREEN_DESKTOP,
        2 => z3_sdl::WINDOW_FULLSCREEN,
        _ => 0,
    };
    let window = Window::new(WINDOW_TITLE, window_width, window_height, window_flags)?;
    // SDL's dummy video driver (used for deterministic/headless verification)
    // reports this optional desktop affordance as unsupported.  The C host also
    // deliberately ignores SDL_SetWindowHitTest's return value.
    let _ = window.install_hit_test();
    let texture_scale = if config.enhanced_mode7 { 4 } else { 1 };
    let mut renderer: Renderer = sdl_renderer_init(
        &window,
        snes_width,
        snes_height,
        config.output_method == OutputMethod::SdlSoftware,
        !config.ignore_aspect_ratio,
        config.linear_filtering,
        texture_scale,
    )?;

    let render_flags = (u8::from(config.new_renderer) * z3_snes::ppu::RENDER_NEW)
        | (u8::from(config.enhanced_mode7) * z3_snes::ppu::RENDER_MODE7_4X)
        | (u8::from(config.extend_y) * z3_snes::ppu::RENDER_HEIGHT_240)
        | (u8::from(config.no_sprite_limits) * z3_snes::ppu::RENDER_NO_SPRITE_LIMITS);
    let mut native = NativeRuntime::new();
    native.ppu.extra_left_right = config
        .extended_aspect_ratio
        .min(z3_snes::ppu::EXTRA_LEFT_RIGHT as u8);
    native.wanted_features = config.features0;
    native.audio.enable_msu(
        config.enable_msu,
        config.msu_volume,
        u32::from(config.audio_freq),
    );
    std::fs::create_dir_all("saves")?;
    z3_game::host::read_sram(Path::new("saves/sram.dat"), &mut native.sram)?;
    let runtime = Arc::new(Mutex::new(native));

    let audio_buffer: Arc<Mutex<Option<AudioCallbackBuffer>>> = Arc::new(Mutex::new(None));
    let audio_runtime = Arc::clone(&runtime);
    let callback_buffer = Arc::clone(&audio_buffer);
    let audio_device = config
        .enable_audio
        .then(|| {
            z3_sdl::AudioDevice::open(
                i32::from(config.audio_freq),
                config.audio_channels,
                config.audio_samples,
                move |output| {
                    let mut buffer = callback_buffer
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let mut native = zelda_apu_lock(&audio_runtime);
                    if let Some(buffer) = buffer.as_mut() {
                        buffer.render_game_audio(output, &mut native.audio);
                    } else {
                        output.fill(0);
                    }
                    zelda_apu_unlock(native);
                },
            )
        })
        .transpose()?;
    if let Some(device) = &audio_device {
        *audio_buffer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(AudioCallbackBuffer::new(
            device.frequency,
            usize::from(device.channels),
        ));
    }

    let mut controllers = Vec::new();
    for index in 0..sdl.num_joysticks() {
        if let Err(error) = open_one_gamepad(&sdl, &mut controllers, index) {
            eprintln!("Could not open gamepad {index}: {error}");
        }
    }
    let mut services = NativeGameServices::new(&assets);
    services.enhanced_features0 = config.features0;
    let mut input = InputState::default();
    let mut ui = UiState {
        paused: false,
        replay_turbo: true,
        cursor: true,
        borderless: false,
        display_perf: false,
        render_flags,
        window_scale,
    };
    let msu_prefix = PathBuf::from(config.msu_path.as_deref().unwrap_or(""));
    let mut running = true;
    let mut audio_paused = true;
    let mut last_tick = sdl.ticks();
    let mut frame_counter = 0_u32;
    if config.autosave {
        execute_action(
            HostAction::SaveLoad {
                kind: SaveLoadKind::Load,
                slot: 0,
            },
            &runtime,
            &audio_buffer,
            &window,
            &mut ui,
            snes_width,
            snes_height,
        )?;
    }

    while running {
        while let Some(event) = sdl.poll_event() {
            let action = match event {
                Event::Quit => {
                    running = false;
                    None
                }
                Event::Key {
                    code,
                    modifiers,
                    pressed,
                } => handle_input(&mut input, &config_state, code, modifiers, pressed),
                Event::ControllerAdded { index } => {
                    if let Err(error) = open_one_gamepad(&sdl, &mut controllers, index) {
                        eprintln!("Could not open gamepad {index}: {error}");
                    }
                    None
                }
                Event::ControllerAxis { id, axis, value } => {
                    handle_gamepad_axis_input(&mut input, &config_state, id, axis, value)
                }
                Event::ControllerButton { button, pressed } => {
                    z3_sdl::remap_controller_button(button).and_then(|button| {
                        handle_gamepad_input(&mut input, &config_state, button, pressed)
                    })
                }
                Event::MouseWheel { y }
                    if z3_sdl::modifier_state() & z3_sdl::KMOD_CTRL != 0 && y != 0 =>
                {
                    change_window_scale(
                        &window,
                        &mut ui.window_scale,
                        if y > 0 { 1 } else { -1 },
                        snes_width,
                        snes_height,
                    );
                    None
                }
                Event::MouseButton {
                    button: z3_sdl::BUTTON_LEFT,
                    state: z3_sdl::PRESSED,
                    clicks: 2,
                } if z3_sdl::modifier_state() & z3_sdl::KMOD_SHIFT != 0
                    && window.flags()
                        & (z3_sdl::WINDOW_FULLSCREEN | z3_sdl::WINDOW_FULLSCREEN_DESKTOP)
                        == 0 =>
                {
                    ui.borderless = !ui.borderless;
                    window.set_bordered(!ui.borderless);
                    None
                }
                _ => None,
            };
            if let Some(action) = action {
                execute_action(
                    action,
                    &runtime,
                    &audio_buffer,
                    &window,
                    &mut ui,
                    snes_width,
                    snes_height,
                )?;
            }
        }

        if ui.paused != audio_paused {
            audio_paused = ui.paused;
            if let Some(device) = &audio_device {
                device.pause(audio_paused);
            }
        }
        if ui.paused {
            sdl.delay(16);
            continue;
        }

        let buttons = input.combined_buttons();
        let is_replay = {
            let mut native = zelda_apu_lock(&runtime);
            let replay = native
                .run_integrated_frame(
                    buttons,
                    assets.nmi_assets(),
                    assets.bytes(0),
                    &assets,
                    &messaging_assets,
                    &mut services,
                    |track| load_pcm(&msu_prefix, track),
                )
                .map_err(|error| format!("replay failed: {error:?}"))?;
            zelda_apu_unlock(native);
            replay
        };
        frame_counter = frame_counter.wrapping_add(1);
        if options.frames == Some(frame_counter) {
            running = false;
        }
        if (input.turbo ^ (is_replay && ui.replay_turbo))
            && frame_counter & if input.turbo { 0x0f } else { 0x7f } != 0
        {
            continue;
        }
        {
            let mut native = zelda_apu_lock(&runtime);
            draw_ppu_frame(
                &mut native,
                &mut renderer,
                snes_width,
                snes_height,
                ui.render_flags,
            )?;
            zelda_apu_unlock(native);
        }
        if !config.disable_frame_delay {
            const DELAYS: [u32; 3] = [17, 17, 16];
            last_tick = last_tick.wrapping_add(DELAYS[frame_counter as usize % 3]);
            let now = sdl.ticks();
            if last_tick > now {
                let mut delay = last_tick - now;
                if delay > 500 {
                    last_tick = now.wrapping_sub(500);
                    delay = 500;
                }
                sdl.delay(delay);
            } else if now.wrapping_sub(last_tick) > 500 {
                last_tick = now;
            }
        }
    }

    if config.autosave {
        let mut native = zelda_apu_lock(&runtime);
        save_load_slot(&mut native, SaveLoadKind::Save, 0)?;
        zelda_apu_unlock(native);
    }
    let native = zelda_apu_lock(&runtime);
    z3_game::host::write_sram(Path::new("saves/sram.dat"), &native.sram)?;
    zelda_apu_unlock(native);
    Ok(())
}

pub fn main_entry() {
    if let Err(error) = run() {
        crate::die(&error.to_string());
    }
}
