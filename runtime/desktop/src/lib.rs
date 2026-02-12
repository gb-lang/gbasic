//! G-Basic desktop runtime — extern "C" stubs for the LLVM-compiled programs.

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::time::Instant;

thread_local! {
    static SDL_STATE: RefCell<Option<SdlState>> = const { RefCell::new(None) };
    static KEY_STATE: RefCell<HashMap<String, bool>> = RefCell::new(HashMap::new());
    static MOUSE_STATE: RefCell<(i64, i64)> = const { RefCell::new((0, 0)) };
    static MEMORY_STORE: RefCell<HashMap<String, i64>> = RefCell::new(HashMap::new());
    static RNG_STATE: RefCell<u64> = const { RefCell::new(12345) };
    static SPRITE_HANDLES: RefCell<Vec<SpriteInfo>> = RefCell::new(Vec::new());
}

struct SpriteInfo {
    surface_data: Vec<u8>,
    width: u32,
    height: u32,
    pitch: u32,
    x: f64,
    y: f64,
    scale: f64,
}

struct SdlState {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    should_quit: bool,
    width: i64,
    height: i64,
    frame_start: Instant,
    delta_time: f64,
}

// ─── DRY helpers ───

/// Read a C string pointer into a &str, returning None if null or invalid UTF-8.
unsafe fn read_cstr<'a>(ptr: *const std::ffi::c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

/// Access the SDL state mutably, returning None if not initialized.
fn with_sdl_mut<R>(f: impl FnOnce(&mut SdlState) -> R) -> Option<R> {
    SDL_STATE.with(|state| {
        let mut borrow = state.borrow_mut();
        borrow.as_mut().map(|s| f(s))
    })
}

/// Convert i64 RGB components to an SDL Color.
fn rgb(r: i64, g: i64, b: i64) -> Color {
    Color::RGB(r as u8, g as u8, b as u8)
}

// ─── Screen namespace ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_init(width: i64, height: i64) {
    let sdl = sdl2::init().expect("Failed to init SDL2");
    let video = sdl.video().expect("Failed to init SDL2 video");
    let window = video
        .window("G-Basic", width as u32, height as u32)
        .position_centered()
        .build()
        .expect("Failed to create window");
    let canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("Failed to create canvas");
    let event_pump = sdl.event_pump().expect("Failed to get event pump");

    SDL_STATE.with(|state| {
        *state.borrow_mut() = Some(SdlState {
            canvas,
            event_pump,
            should_quit: false,
            width,
            height,
            frame_start: Instant::now(),
            delta_time: 0.0,
        });
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_clear(r: i64, g: i64, b: i64) {
    with_sdl_mut(|s| {
        s.canvas.set_draw_color(rgb(r, g, b));
        s.canvas.clear();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_set_pixel(x: i64, y: i64, r: i64, g: i64, b: i64) {
    with_sdl_mut(|s| {
        s.canvas.set_draw_color(rgb(r, g, b));
        let _ = s.canvas.draw_point(Point::new(x as i32, y as i32));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_draw_rect(x: i64, y: i64, w: i64, h: i64, r: i64, g: i64, b: i64) {
    with_sdl_mut(|s| {
        s.canvas.set_draw_color(rgb(r, g, b));
        let _ = s.canvas.fill_rect(Rect::new(x as i32, y as i32, w as u32, h as u32));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_draw_line(x1: i64, y1: i64, x2: i64, y2: i64, r: i64, g: i64, b: i64) {
    with_sdl_mut(|s| {
        s.canvas.set_draw_color(rgb(r, g, b));
        let _ = s.canvas.draw_line(
            Point::new(x1 as i32, y1 as i32),
            Point::new(x2 as i32, y2 as i32),
        );
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_present() {
    with_sdl_mut(|s| {
        s.canvas.present();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_width() -> i64 {
    SDL_STATE.with(|state| {
        state.borrow().as_ref().map(|s| s.width).unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_height() -> i64 {
    SDL_STATE.with(|state| {
        state.borrow().as_ref().map(|s| s.height).unwrap_or(0)
    })
}

// ─── Input namespace ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_input_poll() {
    with_sdl_mut(|s| {
        for event in s.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    s.should_quit = true;
                }
                Event::KeyDown { keycode: Some(k), .. } => {
                    KEY_STATE.with(|ks| {
                        ks.borrow_mut().insert(k.name().to_lowercase(), true);
                    });
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    KEY_STATE.with(|ks| {
                        ks.borrow_mut().insert(k.name().to_lowercase(), false);
                    });
                }
                Event::MouseMotion { x, y, .. } => {
                    MOUSE_STATE.with(|ms| {
                        *ms.borrow_mut() = (x as i64, y as i64);
                    });
                }
                _ => {}
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_input_key_pressed(key: *const std::ffi::c_char) -> i64 {
    let name = match unsafe { read_cstr(key) } {
        Some(s) => s.to_lowercase(),
        None => return 0,
    };
    KEY_STATE.with(|ks| {
        if *ks.borrow().get(&name).unwrap_or(&false) { 1 } else { 0 }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_input_mouse_x() -> i64 {
    MOUSE_STATE.with(|ms| ms.borrow().0)
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_input_mouse_y() -> i64 {
    MOUSE_STATE.with(|ms| ms.borrow().1)
}

// ─── Math namespace ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_sin(x: f64) -> f64 { x.sin() }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_cos(x: f64) -> f64 { x.cos() }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_sqrt(x: f64) -> f64 { x.sqrt() }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_abs(x: f64) -> f64 { x.abs() }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_floor(x: f64) -> f64 { x.floor() }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_ceil(x: f64) -> f64 { x.ceil() }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_pow(x: f64, y: f64) -> f64 { x.powf(y) }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_max(a: f64, b: f64) -> f64 { a.max(b) }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_min(a: f64, b: f64) -> f64 { a.min(b) }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_random() -> f64 {
    RNG_STATE.with(|rng| {
        let mut state = rng.borrow_mut();
        // xorshift64
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        (x as f64) / (u64::MAX as f64)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_pi() -> f64 { std::f64::consts::PI }

// ─── System namespace ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_system_time() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_system_sleep(ms: i64) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_system_exit(code: i64) {
    std::process::exit(code as i32);
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_system_frame_begin() {
    with_sdl_mut(|s| {
        s.frame_start = Instant::now();
    });
    // Also poll events
    runtime_input_poll();
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_system_frame_end() {
    with_sdl_mut(|s| {
        let elapsed = s.frame_start.elapsed();
        s.delta_time = elapsed.as_secs_f64();
        // Target ~60 FPS (16.67ms per frame)
        let target = std::time::Duration::from_micros(16667);
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_system_frame_time() -> f64 {
    SDL_STATE.with(|state| {
        state.borrow().as_ref().map(|s| s.delta_time).unwrap_or(0.0)
    })
}

// ─── Sprite functions ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_sprite_load(path: *const std::ffi::c_char) -> i64 {
    let p = match unsafe { read_cstr(path) } {
        Some(s) => s,
        None => return -1,
    };
    let surface = match sdl2::surface::Surface::load_bmp(p) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let width = surface.width();
    let height = surface.height();
    let pitch = surface.pitch();
    let data = surface.without_lock().unwrap_or(&[]).to_vec();
    SPRITE_HANDLES.with(|sprites| {
        let mut sprites = sprites.borrow_mut();
        let handle = sprites.len() as i64;
        sprites.push(SpriteInfo {
            surface_data: data,
            width,
            height,
            pitch,
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        });
        handle
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_sprite_at(handle: i64, x: f64, y: f64) -> i64 {
    SPRITE_HANDLES.with(|sprites| {
        let mut sprites = sprites.borrow_mut();
        if let Some(s) = sprites.get_mut(handle as usize) {
            s.x = x;
            s.y = y;
        }
    });
    handle
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_sprite_scale(handle: i64, scale: f64) -> i64 {
    SPRITE_HANDLES.with(|sprites| {
        let mut sprites = sprites.borrow_mut();
        if let Some(s) = sprites.get_mut(handle as usize) {
            s.scale = scale;
        }
    });
    handle
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_sprite_draw(handle: i64) {
    SPRITE_HANDLES.with(|sprites| {
        let sprites = sprites.borrow();
        if let Some(info) = sprites.get(handle as usize) {
            let w = (info.width as f64 * info.scale) as u32;
            let h = (info.height as f64 * info.scale) as u32;
            let x = info.x as i32;
            let y = info.y as i32;
            let mut data = info.surface_data.clone();
            let width = info.width;
            let height = info.height;
            let pitch = info.pitch;
            with_sdl_mut(move |s| {
                if let Ok(surface) = sdl2::surface::Surface::from_data(
                    &mut data,
                    width,
                    height,
                    pitch,
                    sdl2::pixels::PixelFormatEnum::RGB24,
                ) {
                    let tc = s.canvas.texture_creator();
                    if let Ok(texture) = tc.create_texture_from_surface(&surface) {
                        let _ = s.canvas.copy(&texture, None, Rect::new(x, y, w, h));
                    }
                }
            });
        }
    });
}

// ─── Draw circle (midpoint algorithm) ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_draw_circle(cx: i64, cy: i64, radius: i64, r: i64, g: i64, b: i64) {
    with_sdl_mut(|s| {
        s.canvas.set_draw_color(rgb(r, g, b));
        let cx = cx as i32;
        let cy = cy as i32;
        let mut x = radius as i32;
        let mut y = 0i32;
        let mut d = 1 - x;
        while x >= y {
            let _ = s.canvas.draw_line(Point::new(cx - x, cy + y), Point::new(cx + x, cy + y));
            let _ = s.canvas.draw_line(Point::new(cx - x, cy - y), Point::new(cx + x, cy - y));
            let _ = s.canvas.draw_line(Point::new(cx - y, cy + x), Point::new(cx + y, cy + x));
            let _ = s.canvas.draw_line(Point::new(cx - y, cy - x), Point::new(cx + y, cy - x));
            y += 1;
            if d <= 0 {
                d += 2 * y + 1;
            } else {
                x -= 1;
                d += 2 * (y - x) + 1;
            }
        }
    });
}

// ─── Sound namespace ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_sound_beep(freq: i64, dur: i64) {
    eprintln!("[sound] beep freq={freq} dur={dur}ms (stub — real tone generation not implemented)");
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_sound_effect_load(path: *const std::ffi::c_char) -> i64 {
    let p = unsafe { read_cstr(path) }.unwrap_or("?");
    eprintln!("[sound] effect_load(\"{p}\") (stub — install SDL2_mixer for real audio)");
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_sound_effect_play(path: *const std::ffi::c_char) {
    let p = unsafe { read_cstr(path) }.unwrap_or("?");
    eprintln!("[sound] effect_play(\"{p}\") (stub)");
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_sound_effect_volume(path: *const std::ffi::c_char, volume: f64) {
    let p = unsafe { read_cstr(path) }.unwrap_or("?");
    eprintln!("[sound] effect_volume(\"{p}\", {volume}) (stub)");
}

// ─── Memory namespace ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_memory_set(key: *const std::ffi::c_char, val: i64) {
    if let Some(s) = unsafe { read_cstr(key) } {
        MEMORY_STORE.with(|m| {
            m.borrow_mut().insert(s.to_string(), val);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_memory_get(key: *const std::ffi::c_char) -> i64 {
    match unsafe { read_cstr(key) } {
        Some(s) => MEMORY_STORE.with(|m| *m.borrow().get(s).unwrap_or(&0)),
        None => 0,
    }
}

// ─── IO namespace ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_io_read_file(path: *const std::ffi::c_char) -> *const std::ffi::c_char {
    match unsafe { read_cstr(path) } {
        Some(p) => match std::fs::read_to_string(p) {
            Ok(content) => {
                let c = CString::new(content).unwrap_or_default();
                c.into_raw() as *const _
            }
            Err(_) => std::ptr::null(),
        },
        None => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_io_write_file(path: *const std::ffi::c_char, data: *const std::ffi::c_char) {
    if let (Some(p), Some(d)) = (unsafe { read_cstr(path) }, unsafe { read_cstr(data) }) {
        let _ = std::fs::write(p, d);
    }
}

// ─── String runtime ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_string_concat(
    a: *const std::ffi::c_char,
    b: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let sa = unsafe { read_cstr(a) }.unwrap_or("");
    let sb = unsafe { read_cstr(b) }.unwrap_or("");
    let result = format!("{sa}{sb}");
    // Intentionally leaks — no GC in week 1
    let c = CString::new(result).unwrap_or_default();
    c.into_raw() as *const _
}

// ─── Legacy functions (kept for backward compat) ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_init(width: i32, height: i32) {
    runtime_screen_init(width as i64, height as i64);
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_clear_screen(r: u8, g: u8, b: u8) {
    runtime_screen_clear(r as i64, g as i64, b as i64);
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_present() {
    runtime_screen_present();
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_should_quit() -> i32 {
    SDL_STATE.with(|state| {
        state.borrow().as_ref().map(|s| if s.should_quit { 1 } else { 0 }).unwrap_or(1)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print(s: *const std::ffi::c_char) {
    match unsafe { read_cstr(s) } {
        Some(s) => println!("{s}"),
        None => println!(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print_int(v: i64) {
    println!("{v}");
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print_float(v: f64) {
    println!("{v}");
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print_str_part(s: *const std::ffi::c_char) {
    if let Some(s) = unsafe { read_cstr(s) } {
        print!("{s}");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print_int_part(v: i64) {
    print!("{v}");
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print_float_part(v: f64) {
    print!("{v}");
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print_newline() {
    println!();
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_shutdown() {
    SDL_STATE.with(|state| {
        *state.borrow_mut() = None;
    });
}
