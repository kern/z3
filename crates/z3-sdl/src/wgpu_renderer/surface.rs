//! Surface configuration/acquisition and platform-specific window handles.

use super::{Error, Window};

pub(super) struct SurfaceOwner<'window> {
    surface: wgpu::Surface<'window>,
    config: wgpu::SurfaceConfiguration,
}

impl<'window> SurfaceOwner<'window> {
    pub(super) fn configure(
        surface: wgpu::Surface<'window>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        window: &Window,
    ) -> Result<Self, Error> {
        let (width, height) = drawable_size(window);
        let config = surface
            .get_default_config(adapter, width.max(1), height.max(1))
            .ok_or_else(|| Error::message("wgpu surface is unsupported by the selected adapter"))?;
        surface.configure(device, &config);
        Ok(Self { surface, config })
    }

    pub(super) fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub(super) fn reconfigure(&mut self, device: &wgpu::Device, window: &Window) {
        let (width, height) = drawable_size(window);
        if width != self.config.width || height != self.config.height {
            self.config.width = width.max(1);
            self.config.height = height.max(1);
            self.surface.configure(device, &self.config);
        }
    }

    pub(super) fn acquire(
        &self,
        device: &wgpu::Device,
    ) -> Result<Option<wgpu::SurfaceTexture>, Error> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(device, &self.config);
                match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                    wgpu::CurrentSurfaceTexture::Timeout
                    | wgpu::CurrentSurfaceTexture::Occluded => return Ok(None),
                    status => return Err(surface_error(&status)),
                }
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(None);
            }
            status => return Err(surface_error(&status)),
        };
        Ok(Some(frame))
    }

    pub(super) fn viewport(&self, source: (u32, u32), keep_aspect: bool) -> (f32, f32, f32, f32) {
        viewport((self.config.width, self.config.height), source, keep_aspect)
    }
}

fn viewport(surface: (u32, u32), source: (u32, u32), keep_aspect: bool) -> (f32, f32, f32, f32) {
    let surface_width = surface.0 as f32;
    let surface_height = surface.1 as f32;
    if !keep_aspect {
        return (0.0, 0.0, surface_width, surface_height);
    }
    let source_ratio = source.0 as f32 / source.1 as f32;
    let surface_ratio = surface_width / surface_height;
    let (width, height) = if surface_ratio > source_ratio {
        (surface_height * source_ratio, surface_height)
    } else {
        (surface_width, surface_width / source_ratio)
    };
    (
        (surface_width - width) * 0.5,
        (surface_height - height) * 0.5,
        width,
        height,
    )
}

fn surface_error(status: &wgpu::CurrentSurfaceTexture) -> Error {
    Error::message(format!("wgpu surface error: {status:?}"))
}

fn drawable_size(window: &Window) -> (u32, u32) {
    #[cfg(target_os = "macos")]
    let size = window.metal_drawable_size();
    #[cfg(not(target_os = "macos"))]
    let size = window.size();
    drawable_extent(size)
}

fn drawable_extent(size: (i32, i32)) -> (u32, u32) {
    (size.0.max(1) as u32, size.1.max(1) as u32)
}

#[cfg(target_os = "macos")]
pub(super) struct MetalView(*mut std::ffi::c_void);

#[cfg(target_os = "macos")]
impl Drop for MetalView {
    fn drop(&mut self) {
        // SAFETY: this wrapper uniquely owns the SDL metal view.
        unsafe { super::super::SDL_Metal_DestroyView(self.0) };
    }
}

#[cfg(target_os = "macos")]
pub(super) fn create_surface<'window>(
    instance: &wgpu::Instance,
    window: &'window Window,
) -> Result<(wgpu::Surface<'window>, MetalView), Error> {
    // SAFETY: the SDL window is live and remains borrowed for the surface's
    // lifetime. The owned MetalView below retains the CAMetalLayer.
    let view = unsafe { super::super::SDL_Metal_CreateView(window.0.as_ptr()) };
    if view.is_null() {
        return Err(Error::last());
    }
    let metal_view = MetalView(view);
    // SAFETY: SDL documents this as the live CAMetalLayer backing metal_view.
    let layer = unsafe { super::super::SDL_Metal_GetLayer(view) };
    if layer.is_null() {
        return Err(Error::message("SDL returned a null CAMetalLayer"));
    }
    // SAFETY: layer remains valid because metal_view is owned by the renderer,
    // and window is borrowed for the returned surface lifetime.
    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer))
    }
    .map_err(|error| Error::message(format!("failed to create wgpu surface: {error}")))?;
    Ok((surface, metal_view))
}

#[cfg(target_os = "windows")]
pub(super) fn create_surface<'window>(
    instance: &wgpu::Instance,
    window: &'window Window,
) -> Result<wgpu::Surface<'window>, Error> {
    use std::num::NonZeroIsize;

    let info = syswm_info(window)?;
    if info.subsystem != SYSWM_WINDOWS {
        return Err(Error::message("SDL window is not using the Win32 backend"));
    }
    // SAFETY: the active union member is selected by subsystem above.
    let win = unsafe { info.info.windows };
    let hwnd = NonZeroIsize::new(win.window as isize)
        .ok_or_else(|| Error::message("SDL returned a null HWND"))?;
    let mut handle = wgpu::rwh::Win32WindowHandle::new(hwnd);
    handle.hinstance = NonZeroIsize::new(win.hinstance as isize);
    let target = wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: Some(wgpu::rwh::WindowsDisplayHandle::new().into()),
        raw_window_handle: handle.into(),
    };
    // SAFETY: SDL owns the HWND for the lifetime represented by window.
    unsafe { instance.create_surface_unsafe(target) }
        .map_err(|error| Error::message(format!("failed to create wgpu surface: {error}")))
}

#[cfg(target_os = "linux")]
pub(super) fn create_surface<'window>(
    instance: &wgpu::Instance,
    window: &'window Window,
) -> Result<wgpu::Surface<'window>, Error> {
    use std::ptr::NonNull;

    let info = syswm_info(window)?;
    let target = match info.subsystem {
        SYSWM_X11 => {
            // SAFETY: the active union member is selected by subsystem.
            let x11 = unsafe { info.info.x11 };
            let display = NonNull::new(x11.display)
                .ok_or_else(|| Error::message("SDL returned a null X11 display"))?;
            wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(
                    wgpu::rwh::XlibDisplayHandle::new(Some(display), 0).into(),
                ),
                raw_window_handle: wgpu::rwh::XlibWindowHandle::new(x11.window).into(),
            }
        }
        SYSWM_WAYLAND => {
            // SAFETY: the active union member is selected by subsystem.
            let wayland = unsafe { info.info.wayland };
            let display = NonNull::new(wayland.display)
                .ok_or_else(|| Error::message("SDL returned a null Wayland display"))?;
            let surface = NonNull::new(wayland.surface)
                .ok_or_else(|| Error::message("SDL returned a null Wayland surface"))?;
            wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(wgpu::rwh::WaylandDisplayHandle::new(display).into()),
                raw_window_handle: wgpu::rwh::WaylandWindowHandle::new(surface).into(),
            }
        }
        subsystem => {
            return Err(Error::message(format!(
                "unsupported SDL window subsystem {subsystem} for wgpu"
            )));
        }
    };
    // SAFETY: SDL owns the native display/window handles for window's lifetime.
    unsafe { instance.create_surface_unsafe(target) }
        .map_err(|error| Error::message(format!("failed to create wgpu surface: {error}")))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub(super) fn create_surface<'window>(
    _instance: &wgpu::Instance,
    _window: &'window Window,
) -> Result<wgpu::Surface<'window>, Error> {
    Err(Error::message("SDL/wgpu is unsupported on this platform"))
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SdlVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

#[cfg(target_os = "windows")]
const SYSWM_WINDOWS: i32 = 1;
#[cfg(target_os = "linux")]
const SYSWM_X11: i32 = 2;
#[cfg(target_os = "linux")]
const SYSWM_WAYLAND: i32 = 6;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct WindowsInfo {
    window: *mut std::ffi::c_void,
    hdc: *mut std::ffi::c_void,
    hinstance: *mut std::ffi::c_void,
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Clone, Copy)]
struct X11Info {
    display: *mut std::ffi::c_void,
    window: std::ffi::c_ulong,
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Clone, Copy)]
struct WaylandInfo {
    display: *mut std::ffi::c_void,
    surface: *mut std::ffi::c_void,
    shell_surface: *mut std::ffi::c_void,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[repr(C)]
#[derive(Clone, Copy)]
union SysWmUnion {
    #[cfg(target_os = "windows")]
    windows: WindowsInfo,
    #[cfg(target_os = "linux")]
    x11: X11Info,
    #[cfg(target_os = "linux")]
    wayland: WaylandInfo,
    padding: [usize; 8],
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[repr(C)]
struct SysWmInfo {
    version: SdlVersion,
    subsystem: i32,
    info: SysWmUnion,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn syswm_info(window: &Window) -> Result<SysWmInfo, Error> {
    let mut info = SysWmInfo {
        version: SdlVersion::default(),
        subsystem: 0,
        info: SysWmUnion { padding: [0; 8] },
    };
    // SAFETY: SDL fills its version structure and then the ABI-compatible
    // SDL_SysWMinfo storage while window remains live.
    unsafe {
        SDL_GetVersion(&mut info.version);
        if SDL_GetWindowWMInfo(window.0.as_ptr(), &mut info) == 0 {
            return Err(Error::last());
        }
    }
    Ok(info)
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[link(name = "SDL2")]
unsafe extern "C" {
    fn SDL_GetVersion(version: *mut SdlVersion);
    fn SDL_GetWindowWMInfo(window: *mut std::ffi::c_void, info: *mut SysWmInfo) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resized_viewports_keep_the_original_aspect_and_stretch_rules() {
        assert_eq!(
            viewport((1000, 600), (4, 3), true),
            (100.0, 0.0, 800.0, 600.0)
        );
        assert_eq!(
            viewport((600, 1000), (4, 3), true),
            (0.0, 275.0, 600.0, 450.0)
        );
        assert_eq!(
            viewport((1024, 896), (256, 224), true),
            // Preserve the original f32 rounding even at a nominally exact fit.
            (0.0, 3.0517578e-5, 1024.0, 895.99994)
        );
        assert_eq!(
            viewport((600, 1000), (256, 224), false),
            (0.0, 0.0, 600.0, 1000.0)
        );
    }

    #[test]
    fn zero_or_negative_drawable_dimensions_keep_the_one_pixel_surface_minimum() {
        for extent in [(0, 0), (-1, -20), (1, 1)] {
            assert_eq!(drawable_extent(extent), (1, 1));
        }
        assert_eq!(drawable_extent((0, 640)), (1, 640));
        assert_eq!(drawable_extent((640, 0)), (640, 1));
        assert_eq!(
            viewport(drawable_extent((0, 0)), (1, 1), true),
            (0.0, 0.0, 1.0, 1.0)
        );
    }
}
