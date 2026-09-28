//! Narrow safe SDL2 boundary for the native Zelda host.
//!
//! The game and deterministic host logic remain safe Rust. Raw pointers are
//! confined to this crate and wrapped in owning RAII types.

mod wgpu_renderer;
pub use wgpu_renderer::WgpuRenderer;

use std::error::Error as StdError;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::fmt;
use std::ptr::NonNull;

pub const INIT_AUDIO: u32 = 0x10;
pub const INIT_VIDEO: u32 = 0x20;
pub const INIT_GAME_CONTROLLER: u32 = 0x2000;
pub const WINDOW_FULLSCREEN: u32 = 0x1;
pub const WINDOW_OPENGL: u32 = 0x2;
pub const WINDOW_HIDDEN: u32 = 0x8;
pub const WINDOW_BORDERLESS: u32 = 0x10;
pub const WINDOW_RESIZABLE: u32 = 0x20;
pub const WINDOW_MINIMIZED: u32 = 0x40;
pub const WINDOW_MAXIMIZED: u32 = 0x80;
pub const WINDOW_FULLSCREEN_DESKTOP: u32 = WINDOW_FULLSCREEN | 0x1000;
pub const WINDOW_METAL: u32 = 0x2000_0000;
pub const WINDOWPOS_UNDEFINED: i32 = 0x1fff_0000;
pub const WINDOWPOS_CENTERED: i32 = 0x2fff_0000;
pub const RENDERER_SOFTWARE: u32 = 0x1;
pub const RENDERER_ACCELERATED: u32 = 0x2;
pub const RENDERER_PRESENT_VSYNC: u32 = 0x4;
pub const BLENDMODE_BLEND: i32 = 1;
pub const PIXELFORMAT_ARGB8888: u32 = 0x1636_2004;
pub const TEXTUREACCESS_STREAMING: i32 = 1;
pub const AUDIO_S16: u16 = 0x8010;
pub const MIX_MAX_VOLUME: i32 = 128;
pub const KMOD_SHIFT: u16 = 0x0003;
pub const KMOD_CTRL: u16 = 0x00c0;
pub const KMOD_ALT: u16 = 0x0300;
pub const EVENT_QUIT: u32 = 0x100;
pub const EVENT_KEY_DOWN: u32 = 0x300;
pub const EVENT_KEY_UP: u32 = 0x301;
pub const EVENT_MOUSE_BUTTON_DOWN: u32 = 0x401;
pub const EVENT_MOUSE_WHEEL: u32 = 0x403;
pub const EVENT_CONTROLLER_AXIS: u32 = 0x650;
pub const EVENT_CONTROLLER_BUTTON_DOWN: u32 = 0x651;
pub const EVENT_CONTROLLER_BUTTON_UP: u32 = 0x652;
pub const EVENT_CONTROLLER_ADDED: u32 = 0x653;
pub const BUTTON_LEFT: u8 = 1;
pub const PRESSED: u8 = 1;
pub const CONTROLLER_AXIS_LEFT_X: u8 = 0;
pub const CONTROLLER_AXIS_LEFT_Y: u8 = 1;
pub const CONTROLLER_AXIS_TRIGGER_LEFT: u8 = 4;
pub const CONTROLLER_AXIS_TRIGGER_RIGHT: u8 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

impl Error {
    fn last() -> Self {
        // SAFETY: SDL_GetError returns a process-owned NUL-terminated string.
        let text = unsafe { CStr::from_ptr(SDL_GetError()) };
        Self(text.to_string_lossy().into_owned())
    }

    fn message(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl StdError for Error {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HitTestResult {
    Normal = 0,
    Draggable = 1,
    ResizeTopLeft = 2,
    ResizeTop = 3,
    ResizeTopRight = 4,
    ResizeRight = 5,
    ResizeBottomRight = 6,
    ResizeBottom = 7,
    ResizeBottomLeft = 8,
    ResizeLeft = 9,
}

#[must_use]
pub fn hit_test(
    width: i32,
    height: i32,
    point: Point,
    fullscreen: bool,
    ctrl: bool,
) -> HitTestResult {
    const BORDER: i32 = 20;
    if fullscreen {
        return HitTestResult::Normal;
    }
    if ctrl {
        return HitTestResult::Draggable;
    }
    if point.y < BORDER {
        if point.x < BORDER {
            HitTestResult::ResizeTopLeft
        } else if point.x >= width - BORDER {
            HitTestResult::ResizeTopRight
        } else {
            HitTestResult::ResizeTop
        }
    } else if point.y >= height - BORDER {
        if point.x < BORDER {
            HitTestResult::ResizeBottomLeft
        } else if point.x >= width - BORDER {
            HitTestResult::ResizeBottomRight
        } else {
            HitTestResult::ResizeBottom
        }
    } else if point.x < BORDER {
        HitTestResult::ResizeLeft
    } else if point.x >= width - BORDER {
        HitTestResult::ResizeRight
    } else {
        HitTestResult::Normal
    }
}

pub struct Sdl;

impl Sdl {
    pub fn init(flags: u32) -> Result<Self, Error> {
        // SAFETY: SDL global initialization accepts a bitmask and has no aliases.
        if unsafe { SDL_Init(flags) } == 0 {
            Ok(Self)
        } else {
            Err(Error::last())
        }
    }

    #[must_use]
    pub fn ticks(&self) -> u32 {
        // SAFETY: SDL is initialized for the lifetime of self.
        unsafe { SDL_GetTicks() }
    }

    pub fn delay(&self, milliseconds: u32) {
        // SAFETY: SDL is initialized for the lifetime of self.
        unsafe { SDL_Delay(milliseconds) }
    }

    pub fn poll_event(&self) -> Option<Event> {
        let mut raw = RawEvent { padding: [0; 56] };
        // SAFETY: raw is the documented SDL_Event-sized writable union.
        (unsafe { SDL_PollEvent(&mut raw) } != 0).then(|| Event::from_raw(&raw))
    }

    #[must_use]
    pub fn num_joysticks(&self) -> i32 {
        // SAFETY: SDL's controller subsystem is initialized.
        unsafe { SDL_NumJoysticks() }
    }

    pub fn open_game_controller(&self, index: i32) -> Result<Option<GameController>, Error> {
        // SAFETY: query/open accept any enumerated device index.
        if unsafe { SDL_IsGameController(index) } == 0 {
            return Ok(None);
        }
        // SAFETY: SDL returns an owned controller handle or null.
        NonNull::new(unsafe { SDL_GameControllerOpen(index) })
            .map(|raw| Some(GameController(raw)))
            .ok_or_else(Error::last)
    }
}

impl Drop for Sdl {
    fn drop(&mut self) {
        // SAFETY: balances the successful SDL_Init that created self.
        unsafe { SDL_Quit() };
    }
}

pub struct GameController(NonNull<c_void>);

impl Drop for GameController {
    fn drop(&mut self) {
        // SAFETY: self uniquely owns this controller handle.
        unsafe { SDL_GameControllerClose(self.0.as_ptr()) };
    }
}

pub struct Window(NonNull<c_void>);

impl Window {
    pub fn new(title: &str, width: i32, height: i32, flags: u32) -> Result<Self, Error> {
        let title = CString::new(title).map_err(|_| Error("window title contains NUL".into()))?;
        // SAFETY: title lives through the call; SDL returns an owned handle.
        NonNull::new(unsafe {
            SDL_CreateWindow(
                title.as_ptr(),
                WINDOWPOS_UNDEFINED,
                WINDOWPOS_UNDEFINED,
                width,
                height,
                flags,
            )
        })
        .map(Self)
        .ok_or_else(Error::last)
    }

    #[must_use]
    pub fn flags(&self) -> u32 {
        // SAFETY: self owns a live window.
        unsafe { SDL_GetWindowFlags(self.0.as_ptr()) }
    }

    #[must_use]
    pub fn size(&self) -> (i32, i32) {
        let (mut width, mut height) = (0, 0);
        // SAFETY: output pointers are valid and self is live.
        unsafe { SDL_GetWindowSize(self.0.as_ptr(), &mut width, &mut height) };
        (width, height)
    }

    pub fn set_size(&self, width: i32, height: i32) {
        // SAFETY: self is live and dimensions are ordinary SDL inputs.
        unsafe { SDL_SetWindowSize(self.0.as_ptr(), width, height) };
    }

    pub fn set_position(&self, x: i32, y: i32) {
        // SAFETY: self is live and coordinates are ordinary SDL inputs.
        unsafe { SDL_SetWindowPosition(self.0.as_ptr(), x, y) };
    }

    pub fn display_usable_bounds(&self) -> Result<Rect, Error> {
        // SAFETY: self is a live window.
        let display = unsafe { SDL_GetWindowDisplayIndex(self.0.as_ptr()) }.max(0);
        let mut bounds = Rect::default();
        // SAFETY: bounds is a valid writable SDL_Rect.
        if unsafe { SDL_GetDisplayUsableBounds(display, &mut bounds) } == 0 {
            Ok(bounds)
        } else {
            Err(Error::last())
        }
    }

    #[must_use]
    pub fn border_sizes(&self) -> Option<(i32, i32, i32, i32)> {
        let (mut top, mut left, mut bottom, mut right) = (0, 0, 0, 0);
        // SAFETY: outputs are valid and self is live.
        (unsafe {
            SDL_GetWindowBordersSize(
                self.0.as_ptr(),
                &mut top,
                &mut left,
                &mut bottom,
                &mut right,
            )
        } == 0)
            .then_some((top, left, bottom, right))
    }

    #[must_use]
    pub fn global_mouse_position(&self) -> (i32, i32) {
        let (mut x, mut y) = (0, 0);
        // SAFETY: outputs are valid.
        unsafe { SDL_GetGlobalMouseState(&mut x, &mut y) };
        (x, y)
    }

    pub fn install_hit_test(&self) -> Result<(), Error> {
        // SAFETY: callback has SDL's ABI and needs no user data.
        if unsafe {
            SDL_SetWindowHitTest(
                self.0.as_ptr(),
                Some(hit_test_callback),
                std::ptr::null_mut(),
            )
        } == 0
        {
            Ok(())
        } else {
            Err(Error::last())
        }
    }

    pub fn set_fullscreen_desktop(&self, enabled: bool) -> Result<(), Error> {
        // SAFETY: self is live.
        if unsafe {
            SDL_SetWindowFullscreen(
                self.0.as_ptr(),
                if enabled {
                    WINDOW_FULLSCREEN_DESKTOP
                } else {
                    0
                },
            )
        } == 0
        {
            Ok(())
        } else {
            Err(Error::last())
        }
    }

    pub fn set_bordered(&self, bordered: bool) {
        // SAFETY: self is live.
        unsafe { SDL_SetWindowBordered(self.0.as_ptr(), c_int::from(bordered)) };
    }

    pub fn show_cursor(&self, visible: bool) {
        // SAFETY: cursor visibility is global SDL state; self proves SDL is active.
        unsafe { SDL_ShowCursor(c_int::from(visible)) };
    }

    pub fn set_title(&self, title: &str) -> Result<(), Error> {
        let title = CString::new(title).map_err(|_| Error("window title contains NUL".into()))?;
        // SAFETY: title lives through the call and self is live.
        unsafe { SDL_SetWindowTitle(self.0.as_ptr(), title.as_ptr()) };
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn metal_drawable_size(&self) -> (i32, i32) {
        let (mut width, mut height) = (0, 0);
        // SAFETY: outputs and the live SDL window are valid.
        unsafe { SDL_Metal_GetDrawableSize(self.0.as_ptr(), &mut width, &mut height) };
        (width, height)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // SAFETY: self uniquely owns this live window.
        unsafe { SDL_DestroyWindow(self.0.as_ptr()) };
    }
}

unsafe extern "C" fn hit_test_callback(
    window: *mut c_void,
    point: *const Point,
    _: *mut c_void,
) -> c_int {
    let (mut width, mut height) = (0, 0);
    // SAFETY: SDL supplies live callback arguments.
    unsafe { SDL_GetWindowSize(window, &mut width, &mut height) };
    // SAFETY: point is valid for the callback duration.
    let point = unsafe { *point };
    // SAFETY: SDL supplies a live window.
    let flags = unsafe { SDL_GetWindowFlags(window) };
    // SAFETY: global modifier query has no preconditions after SDL init.
    let ctrl = unsafe { SDL_GetModState() } & KMOD_CTRL != 0;
    hit_test(
        width,
        height,
        point,
        flags & (WINDOW_FULLSCREEN | WINDOW_FULLSCREEN_DESKTOP) != 0,
        ctrl,
    ) as c_int
}

pub struct Renderer {
    raw: NonNull<c_void>,
    texture: NonNull<c_void>,
    last_rect: Option<Rect>,
}

impl Renderer {
    pub fn new(
        window: &Window,
        width: i32,
        height: i32,
        software: bool,
        keep_aspect: bool,
        linear: bool,
        texture_scale: i32,
    ) -> Result<Self, Error> {
        let flags = if software {
            RENDERER_SOFTWARE
        } else {
            RENDERER_ACCELERATED | RENDERER_PRESENT_VSYNC
        };
        // SAFETY: window is live; returned renderer is owned.
        let raw = NonNull::new(unsafe { SDL_CreateRenderer(window.0.as_ptr(), -1, flags) })
            .ok_or_else(Error::last)?;
        if keep_aspect {
            // SAFETY: renderer is live.
            unsafe { SDL_RenderSetLogicalSize(raw.as_ptr(), width, height) };
        }
        if linear {
            let name = c"SDL_RENDER_SCALE_QUALITY";
            let value = c"best";
            // SAFETY: static C strings are valid.
            unsafe { SDL_SetHint(name.as_ptr(), value.as_ptr()) };
        }
        // SAFETY: renderer is live; returned texture is owned by this wrapper.
        let texture = NonNull::new(unsafe {
            SDL_CreateTexture(
                raw.as_ptr(),
                PIXELFORMAT_ARGB8888,
                TEXTUREACCESS_STREAMING,
                width * texture_scale,
                height * texture_scale,
            )
        });
        let Some(texture) = texture else {
            // SAFETY: raw was just created and texture creation failed.
            unsafe { SDL_DestroyRenderer(raw.as_ptr()) };
            return Err(Error::last());
        };
        Ok(Self {
            raw,
            texture,
            last_rect: None,
        })
    }

    pub fn begin_draw(&mut self, width: i32, height: i32) -> Result<TextureLock<'_>, Error> {
        let rect = Rect {
            x: 0,
            y: 0,
            w: width,
            h: height,
        };
        let mut pixels = std::ptr::null_mut();
        let mut pitch = 0;
        // SAFETY: rect and outputs are valid; texture is live and not otherwise locked.
        if unsafe { SDL_LockTexture(self.texture.as_ptr(), &rect, &mut pixels, &mut pitch) } != 0 {
            return Err(Error::last());
        }
        let len = usize::try_from(pitch)
            .unwrap_or(0)
            .saturating_mul(usize::try_from(height).unwrap_or(0));
        Ok(TextureLock {
            renderer: self,
            pixels: pixels.cast(),
            len,
            pitch: pitch as usize,
            rect,
        })
    }

    fn end_draw(&mut self, rect: &Rect) -> Result<(), Error> {
        // SAFETY: texture is currently locked by TextureLock.
        unsafe { SDL_UnlockTexture(self.texture.as_ptr()) };
        // SAFETY: renderer/texture are live and rect lies within the texture.
        unsafe { SDL_RenderClear(self.raw.as_ptr()) };
        // SAFETY: same as above; null destination means the full target.
        if unsafe {
            SDL_RenderCopy(
                self.raw.as_ptr(),
                self.texture.as_ptr(),
                rect,
                std::ptr::null(),
            )
        } != 0
        {
            return Err(Error::last());
        }
        // SAFETY: renderer is live.
        unsafe { SDL_RenderPresent(self.raw.as_ptr()) };
        self.last_rect = Some(*rect);
        Ok(())
    }

    /// Re-presents the last game frame beneath the same translucent black
    /// overlay used by the C host's `PauseDimmed` command.
    pub fn present_dimmed(&mut self) -> Result<(), Error> {
        // Back-buffer contents after SDL_RenderPresent are unspecified, so
        // reconstruct the last frame from the streaming texture first.
        unsafe { SDL_RenderClear(self.raw.as_ptr()) };
        if let Some(rect) = self.last_rect
            && unsafe {
                SDL_RenderCopy(
                    self.raw.as_ptr(),
                    self.texture.as_ptr(),
                    &rect,
                    std::ptr::null(),
                )
            } != 0
        {
            return Err(Error::last());
        }
        if unsafe { SDL_SetRenderDrawBlendMode(self.raw.as_ptr(), BLENDMODE_BLEND) } != 0
            || unsafe { SDL_SetRenderDrawColor(self.raw.as_ptr(), 0, 0, 0, 159) } != 0
            || unsafe { SDL_RenderFillRect(self.raw.as_ptr(), std::ptr::null()) } != 0
        {
            return Err(Error::last());
        }
        unsafe { SDL_RenderPresent(self.raw.as_ptr()) };
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // SAFETY: these handles are uniquely owned and destroyed texture-first.
        unsafe {
            SDL_DestroyTexture(self.texture.as_ptr());
            SDL_DestroyRenderer(self.raw.as_ptr());
        }
    }
}

pub struct TextureLock<'a> {
    renderer: &'a mut Renderer,
    pixels: *mut u8,
    len: usize,
    pitch: usize,
    rect: Rect,
}

impl TextureLock<'_> {
    #[must_use]
    pub fn pitch(&self) -> usize {
        self.pitch
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        // SAFETY: SDL supplied this region for the active texture lock.
        unsafe { std::slice::from_raw_parts_mut(self.pixels, self.len) }
    }

    pub fn present(self) -> Result<(), Error> {
        let mut this = std::mem::ManuallyDrop::new(self);
        let rect = this.rect;
        this.renderer.end_draw(&rect)
    }
}

impl Drop for TextureLock<'_> {
    fn drop(&mut self) {
        // SAFETY: the texture remains live and this guard owns its lock.
        unsafe { SDL_UnlockTexture(self.renderer.texture.as_ptr()) };
    }
}

pub struct AudioDevice {
    id: u32,
    callback: NonNull<AudioCallbackState>,
    pub frequency: i32,
    pub channels: u8,
}

struct AudioCallbackState {
    callback: Box<AudioCallback>,
}

type AudioCallback = dyn FnMut(&mut [i16]) + Send;

impl AudioDevice {
    pub fn open(
        frequency: i32,
        channels: u8,
        samples: u16,
        callback: impl FnMut(&mut [i16]) + Send + 'static,
    ) -> Result<Self, Error> {
        let state = Box::new(AudioCallbackState {
            callback: Box::new(callback),
        });
        let callback = NonNull::from(Box::leak(state));
        let want = AudioSpec {
            freq: frequency,
            format: AUDIO_S16,
            channels,
            silence: 0,
            samples,
            padding: 0,
            size: 0,
            callback: Some(audio_callback),
            userdata: callback.as_ptr().cast(),
        };
        let mut have = want;
        // SAFETY: specs and callback userdata stay valid for the device lifetime.
        let id = unsafe { SDL_OpenAudioDevice(std::ptr::null(), 0, &want, &mut have, 0) };
        if id == 0 {
            // SAFETY: reclaim the leaked allocation after failed open.
            drop(unsafe { Box::from_raw(callback.as_ptr()) });
            return Err(Error::last());
        }
        Ok(Self {
            id,
            callback,
            frequency: have.freq,
            channels: have.channels,
        })
    }

    pub fn pause(&self, paused: bool) {
        // SAFETY: id is a live audio device.
        unsafe { SDL_PauseAudioDevice(self.id, c_int::from(paused)) };
    }
}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        // SAFETY: pausing/closing synchronizes with the callback; userdata may then be freed.
        unsafe {
            SDL_PauseAudioDevice(self.id, 1);
            SDL_CloseAudioDevice(self.id);
            drop(Box::from_raw(self.callback.as_ptr()));
        }
    }
}

unsafe extern "C" fn audio_callback(userdata: *mut c_void, stream: *mut u8, len: c_int) {
    if userdata.is_null() || stream.is_null() || len <= 0 {
        return;
    }
    // SAFETY: SDL passes the AudioCallbackState installed by AudioDevice::open.
    let state = unsafe { &mut *userdata.cast::<AudioCallbackState>() };
    // SAFETY: AUDIO_S16 guarantees a writable aligned i16 stream of len bytes.
    let output = unsafe { std::slice::from_raw_parts_mut(stream.cast::<i16>(), len as usize / 2) };
    (state.callback)(output);
}

#[must_use]
pub fn adjusted_mixer_volume(current: i32, adjustment: i32) -> i32 {
    (current + adjustment * (MIX_MAX_VOLUME >> 4)).clamp(0, MIX_MAX_VOLUME)
}

pub fn mix_audio_s16(destination: &mut [i16], source: &[i16], volume: i32) {
    assert_eq!(destination.len(), source.len());
    if volume == MIX_MAX_VOLUME {
        destination.copy_from_slice(source);
        return;
    }
    for (destination, &source) in destination.iter_mut().zip(source) {
        let mixed = i32::from(*destination) + i32::from(source) * volume / MIX_MAX_VOLUME;
        *destination = mixed.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Quit,
    Key {
        code: i32,
        modifiers: u16,
        pressed: bool,
    },
    MouseWheel {
        y: i32,
    },
    MouseButton {
        button: u8,
        state: u8,
        clicks: u8,
    },
    ControllerAdded {
        index: i32,
    },
    ControllerAxis {
        id: i32,
        axis: u8,
        value: i16,
    },
    ControllerButton {
        button: u8,
        pressed: bool,
    },
    Other(u32),
}

impl Event {
    fn from_raw(raw: &RawEvent) -> Self {
        // SAFETY: every SDL_Event starts with its u32 event type.
        let event_type = unsafe { raw.event_type };
        match event_type {
            EVENT_QUIT => Self::Quit,
            EVENT_KEY_DOWN | EVENT_KEY_UP => {
                // SAFETY: the type tag selects SDL_KeyboardEvent.
                let event = unsafe { raw.key };
                Self::Key {
                    code: event.keysym.sym,
                    modifiers: event.keysym.modifiers,
                    pressed: event_type == EVENT_KEY_DOWN,
                }
            }
            EVENT_MOUSE_WHEEL => {
                // SAFETY: the type tag selects SDL_MouseWheelEvent.
                Self::MouseWheel {
                    y: unsafe { raw.wheel }.y,
                }
            }
            EVENT_MOUSE_BUTTON_DOWN => {
                // SAFETY: the type tag selects SDL_MouseButtonEvent.
                let event = unsafe { raw.button };
                Self::MouseButton {
                    button: event.button,
                    state: event.state,
                    clicks: event.clicks,
                }
            }
            EVENT_CONTROLLER_ADDED => {
                // SAFETY: the type tag selects SDL_ControllerDeviceEvent.
                Self::ControllerAdded {
                    index: unsafe { raw.cdevice }.which,
                }
            }
            EVENT_CONTROLLER_AXIS => {
                // SAFETY: the type tag selects SDL_ControllerAxisEvent.
                let event = unsafe { raw.caxis };
                Self::ControllerAxis {
                    id: event.which,
                    axis: event.axis,
                    value: event.value,
                }
            }
            EVENT_CONTROLLER_BUTTON_DOWN | EVENT_CONTROLLER_BUTTON_UP => {
                // SAFETY: the type tag selects SDL_ControllerButtonEvent.
                Self::ControllerButton {
                    button: unsafe { raw.cbutton }.button,
                    pressed: event_type == EVENT_CONTROLLER_BUTTON_DOWN,
                }
            }
            other => Self::Other(other),
        }
    }
}

#[must_use]
pub fn key_from_name(name: &str) -> Option<i32> {
    let name = CString::new(name).ok()?;
    // SAFETY: string lives through the call.
    let key = unsafe { SDL_GetKeyFromName(name.as_ptr()) };
    (key != 0).then_some(key)
}

#[must_use]
pub fn modifier_state() -> u16 {
    // SAFETY: global modifier query has no preconditions after SDL init.
    unsafe { SDL_GetModState() }
}

#[must_use]
pub fn remap_controller_button(button: u8) -> Option<usize> {
    (button <= 14).then_some(usize::from(button))
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Keysym {
    scancode: i32,
    sym: i32,
    modifiers: u16,
    unused: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct KeyboardEvent {
    event_type: u32,
    timestamp: u32,
    window_id: u32,
    state: u8,
    repeat: u8,
    padding2: u8,
    padding3: u8,
    keysym: Keysym,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct MouseWheelEvent {
    event_type: u32,
    timestamp: u32,
    window_id: u32,
    which: u32,
    x: i32,
    y: i32,
    direction: u32,
    precise_x: f32,
    precise_y: f32,
    mouse_x: i32,
    mouse_y: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct MouseButtonEvent {
    event_type: u32,
    timestamp: u32,
    window_id: u32,
    which: u32,
    button: u8,
    state: u8,
    clicks: u8,
    padding1: u8,
    x: i32,
    y: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct ControllerAxisEvent {
    event_type: u32,
    timestamp: u32,
    which: i32,
    axis: u8,
    padding1: u8,
    padding2: u8,
    padding3: u8,
    value: i16,
    padding4: u16,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct ControllerButtonEvent {
    event_type: u32,
    timestamp: u32,
    which: i32,
    button: u8,
    state: u8,
    padding1: u8,
    padding2: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct ControllerDeviceEvent {
    event_type: u32,
    timestamp: u32,
    which: i32,
}

#[repr(C)]
union RawEvent {
    event_type: u32,
    key: KeyboardEvent,
    wheel: MouseWheelEvent,
    button: MouseButtonEvent,
    caxis: ControllerAxisEvent,
    cbutton: ControllerButtonEvent,
    cdevice: ControllerDeviceEvent,
    padding: [u8; 56],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct AudioSpec {
    freq: c_int,
    format: u16,
    channels: u8,
    silence: u8,
    samples: u16,
    padding: u16,
    size: u32,
    callback: Option<unsafe extern "C" fn(*mut c_void, *mut u8, c_int)>,
    userdata: *mut c_void,
}

type HitTestCallback = unsafe extern "C" fn(*mut c_void, *const Point, *mut c_void) -> c_int;

#[link(name = "SDL2")]
unsafe extern "C" {
    fn SDL_Init(flags: u32) -> c_int;
    fn SDL_Quit();
    fn SDL_GetError() -> *const c_char;
    fn SDL_GetTicks() -> u32;
    fn SDL_Delay(milliseconds: u32);
    fn SDL_PollEvent(event: *mut RawEvent) -> c_int;
    fn SDL_CreateWindow(
        title: *const c_char,
        x: c_int,
        y: c_int,
        w: c_int,
        h: c_int,
        flags: u32,
    ) -> *mut c_void;
    fn SDL_DestroyWindow(window: *mut c_void);
    fn SDL_GetWindowFlags(window: *mut c_void) -> u32;
    fn SDL_GetWindowSize(window: *mut c_void, width: *mut c_int, height: *mut c_int);
    fn SDL_SetWindowSize(window: *mut c_void, width: c_int, height: c_int);
    fn SDL_SetWindowPosition(window: *mut c_void, x: c_int, y: c_int);
    fn SDL_GetWindowDisplayIndex(window: *mut c_void) -> c_int;
    fn SDL_GetDisplayUsableBounds(display: c_int, bounds: *mut Rect) -> c_int;
    fn SDL_GetWindowBordersSize(
        window: *mut c_void,
        top: *mut c_int,
        left: *mut c_int,
        bottom: *mut c_int,
        right: *mut c_int,
    ) -> c_int;
    fn SDL_GetGlobalMouseState(x: *mut c_int, y: *mut c_int) -> u32;
    fn SDL_SetWindowHitTest(
        window: *mut c_void,
        callback: Option<HitTestCallback>,
        data: *mut c_void,
    ) -> c_int;
    fn SDL_SetWindowFullscreen(window: *mut c_void, flags: u32) -> c_int;
    fn SDL_SetWindowBordered(window: *mut c_void, bordered: c_int);
    fn SDL_SetWindowTitle(window: *mut c_void, title: *const c_char);
    #[cfg(target_os = "macos")]
    fn SDL_Metal_CreateView(window: *mut c_void) -> *mut c_void;
    #[cfg(target_os = "macos")]
    fn SDL_Metal_DestroyView(view: *mut c_void);
    #[cfg(target_os = "macos")]
    fn SDL_Metal_GetLayer(view: *mut c_void) -> *mut c_void;
    #[cfg(target_os = "macos")]
    fn SDL_Metal_GetDrawableSize(window: *mut c_void, width: *mut c_int, height: *mut c_int);
    fn SDL_GetModState() -> u16;
    fn SDL_ShowCursor(toggle: c_int) -> c_int;
    fn SDL_CreateRenderer(window: *mut c_void, index: c_int, flags: u32) -> *mut c_void;
    fn SDL_DestroyRenderer(renderer: *mut c_void);
    fn SDL_RenderSetLogicalSize(renderer: *mut c_void, width: c_int, height: c_int) -> c_int;
    fn SDL_SetHint(name: *const c_char, value: *const c_char) -> c_int;
    fn SDL_CreateTexture(
        renderer: *mut c_void,
        format: u32,
        access: c_int,
        width: c_int,
        height: c_int,
    ) -> *mut c_void;
    fn SDL_DestroyTexture(texture: *mut c_void);
    fn SDL_LockTexture(
        texture: *mut c_void,
        rect: *const Rect,
        pixels: *mut *mut c_void,
        pitch: *mut c_int,
    ) -> c_int;
    fn SDL_UnlockTexture(texture: *mut c_void);
    fn SDL_RenderClear(renderer: *mut c_void) -> c_int;
    fn SDL_RenderCopy(
        renderer: *mut c_void,
        texture: *mut c_void,
        source: *const Rect,
        destination: *const Rect,
    ) -> c_int;
    fn SDL_RenderPresent(renderer: *mut c_void);
    fn SDL_SetRenderDrawBlendMode(renderer: *mut c_void, blend_mode: c_int) -> c_int;
    fn SDL_SetRenderDrawColor(
        renderer: *mut c_void,
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    ) -> c_int;
    fn SDL_RenderFillRect(renderer: *mut c_void, rect: *const Rect) -> c_int;
    fn SDL_OpenAudioDevice(
        device: *const c_char,
        capture: c_int,
        wanted: *const AudioSpec,
        obtained: *mut AudioSpec,
        changes: c_int,
    ) -> u32;
    fn SDL_PauseAudioDevice(device: u32, pause: c_int);
    fn SDL_CloseAudioDevice(device: u32);
    fn SDL_NumJoysticks() -> c_int;
    fn SDL_IsGameController(index: c_int) -> c_int;
    fn SDL_GameControllerOpen(index: c_int) -> *mut c_void;
    fn SDL_GameControllerClose(controller: *mut c_void);
    fn SDL_GetKeyFromName(name: *const c_char) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_event_union_size_matches_sdl2_abi() {
        assert_eq!(std::mem::size_of::<RawEvent>(), 56);
    }

    #[test]
    fn hit_test_matches_all_c_regions() {
        assert_eq!(
            hit_test(300, 200, Point { x: 0, y: 0 }, false, false),
            HitTestResult::ResizeTopLeft
        );
        assert_eq!(
            hit_test(300, 200, Point { x: 299, y: 199 }, false, false),
            HitTestResult::ResizeBottomRight
        );
        assert_eq!(
            hit_test(300, 200, Point { x: 150, y: 100 }, false, true),
            HitTestResult::Draggable
        );
        assert_eq!(
            hit_test(300, 200, Point { x: 0, y: 0 }, true, true),
            HitTestResult::Normal
        );
    }

    #[test]
    fn volume_adjustment_and_s16_mixing_match_sdl() {
        assert_eq!(adjusted_mixer_volume(128, 1), 128);
        assert_eq!(adjusted_mixer_volume(8, -2), 0);
        assert_eq!(adjusted_mixer_volume(64, 1), 72);
        let mut output = [0_i16, 30_000, -30_000];
        mix_audio_s16(&mut output, &[12_800, 12_800, -12_800], 64);
        assert_eq!(output, [6_400, i16::MAX, i16::MIN]);
    }
}
