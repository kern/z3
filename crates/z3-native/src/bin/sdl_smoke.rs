use z3_sdl::{
    AudioDevice, INIT_AUDIO, INIT_GAME_CONTROLLER, INIT_VIDEO, Renderer, Sdl, WINDOW_HIDDEN, Window,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl = Sdl::init(INIT_VIDEO | INIT_AUDIO | INIT_GAME_CONTROLLER)?;
    let window = Window::new("z3 SDL smoke", 256, 224, WINDOW_HIDDEN)?;
    let mut renderer = Renderer::new(&window, 256, 224, true, true, false, 1)?;
    let mut frame = renderer.begin_draw(256, 224)?;
    frame.pixels_mut().fill(0x55);
    frame.present()?;
    let mut controllers = Vec::new();
    for index in 0..sdl.num_joysticks() {
        let _ = z3_native::open_one_gamepad(&sdl, &mut controllers, index);
    }
    let callbacks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let callback_count = std::sync::Arc::clone(&callbacks);
    let audio = AudioDevice::open(32_000, 2, 64, move |samples| {
        samples.fill(0);
        callback_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    })?;
    audio.pause(false);
    sdl.delay(20);
    audio.pause(true);
    if callbacks.load(std::sync::atomic::Ordering::Relaxed) == 0 {
        return Err("SDL dummy audio callback did not run".into());
    }
    Ok(())
}
