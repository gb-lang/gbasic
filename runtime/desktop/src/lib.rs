//! G-Basic desktop runtime — extern "C" stubs for the LLVM-compiled programs.

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::time::Instant;

// ─── Object System ───

#[derive(Debug, Clone, Copy, PartialEq)]
enum ObjectKind {
    Rect,
    Circle,
}

#[derive(Debug, Clone)]
struct GameObject {
    kind: ObjectKind,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color_r: u8,
    color_g: u8,
    color_b: u8,
    visible: bool,
    layer: i64,
    // Physics
    vx: f64,
    vy: f64,
    gravity: f64,
    solid: bool,
    bounces: bool,
    // State
    alive: bool,
}

impl GameObject {
    fn new(kind: ObjectKind, w: f64, h: f64) -> Self {
        Self {
            kind,
            x: 0.0,
            y: 0.0,
            w,
            h,
            color_r: 255,
            color_g: 255,
            color_b: 255,
            visible: true,
            layer: 0,
            vx: 0.0,
            vy: 0.0,
            gravity: 0.0,
            solid: false,
            bounces: false,
            alive: true,
        }
    }
}

thread_local! {
    static SDL_STATE: RefCell<Option<SdlState>> = const { RefCell::new(None) };
    static KEY_STATE: RefCell<HashMap<String, bool>> = RefCell::new(HashMap::new());
    static MOUSE_STATE: RefCell<(i64, i64)> = const { RefCell::new((0, 0)) };
    static MEMORY_STORE: RefCell<HashMap<String, i64>> = RefCell::new(HashMap::new());
    static RNG_STATE: RefCell<u64> = const { RefCell::new(12345) };
    static SPRITE_HANDLES: RefCell<Vec<SpriteInfo>> = RefCell::new(Vec::new());
    static OBJECTS: RefCell<Vec<GameObject>> = RefCell::new(Vec::new());
    static SCREEN_AUTO_INIT: Cell<bool> = const { Cell::new(false) };
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

// ─── Auto-init ───

#[unsafe(no_mangle)]
pub extern "C" fn ensure_screen_init() {
    SCREEN_AUTO_INIT.with(|init| {
        if !init.get() {
            init.set(true);
            SDL_STATE.with(|state| {
                if state.borrow().is_none() {
                    runtime_screen_init(800, 600);
                }
            });
        }
    });
}

// ─── Object constructors ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_create_rect(w: f64, h: f64) -> i64 {
    ensure_screen_init();
    OBJECTS.with(|objs| {
        let mut objs = objs.borrow_mut();
        let handle = objs.len() as i64;
        objs.push(GameObject::new(ObjectKind::Rect, w, h));
        handle
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_create_circle(r: f64) -> i64 {
    ensure_screen_init();
    OBJECTS.with(|objs| {
        let mut objs = objs.borrow_mut();
        let handle = objs.len() as i64;
        // For circles, w=h=diameter, but we store radius in w
        objs.push(GameObject::new(ObjectKind::Circle, r, r));
        handle
    })
}

// ─── Property setters ───

fn with_object_mut(handle: i64, f: impl FnOnce(&mut GameObject)) {
    OBJECTS.with(|objs| {
        let mut objs = objs.borrow_mut();
        if let Some(obj) = objs.get_mut(handle as usize) {
            if obj.alive {
                f(obj);
            }
        }
    });
}

fn with_object<R: Default>(handle: i64, f: impl FnOnce(&GameObject) -> R) -> R {
    OBJECTS.with(|objs| {
        let objs = objs.borrow();
        objs.get(handle as usize)
            .filter(|o| o.alive)
            .map(|o| f(o))
            .unwrap_or_default()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_position(handle: i64, x: f64, y: f64) {
    with_object_mut(handle, |o| { o.x = x; o.y = y; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_position_x(handle: i64, x: f64) {
    with_object_mut(handle, |o| { o.x = x; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_position_y(handle: i64, y: f64) {
    with_object_mut(handle, |o| { o.y = y; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_color(handle: i64, r: i64, g: i64, b: i64) {
    with_object_mut(handle, |o| {
        o.color_r = r as u8;
        o.color_g = g as u8;
        o.color_b = b as u8;
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_visible(handle: i64, v: i64) {
    with_object_mut(handle, |o| { o.visible = v != 0; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_velocity(handle: i64, vx: f64, vy: f64) {
    with_object_mut(handle, |o| { o.vx = vx; o.vy = vy; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_velocity_x(handle: i64, vx: f64) {
    with_object_mut(handle, |o| { o.vx = vx; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_velocity_y(handle: i64, vy: f64) {
    with_object_mut(handle, |o| { o.vy = vy; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_gravity(handle: i64, g: f64) {
    with_object_mut(handle, |o| { o.gravity = g; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_solid(handle: i64, v: i64) {
    with_object_mut(handle, |o| { o.solid = v != 0; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_bounces(handle: i64, v: i64) {
    with_object_mut(handle, |o| { o.bounces = v != 0; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_set_layer(handle: i64, l: i64) {
    with_object_mut(handle, |o| { o.layer = l; });
}

// ─── Property getters ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_get_position_x(handle: i64) -> f64 {
    with_object(handle, |o| o.x)
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_get_position_y(handle: i64) -> f64 {
    with_object(handle, |o| o.y)
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_get_velocity_x(handle: i64) -> f64 {
    with_object(handle, |o| o.vx)
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_get_velocity_y(handle: i64) -> f64 {
    with_object(handle, |o| o.vy)
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_get_size_width(handle: i64) -> f64 {
    with_object(handle, |o| o.w)
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_get_size_height(handle: i64) -> f64 {
    with_object(handle, |o| o.h)
}

// ─── Object methods ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_object_move(handle: i64, dx: f64, dy: f64) {
    with_object_mut(handle, |o| { o.x += dx; o.y += dy; });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_object_collides(h1: i64, h2: i64) -> i64 {
    OBJECTS.with(|objs| {
        let objs = objs.borrow();
        let a = match objs.get(h1 as usize) {
            Some(o) if o.alive => o,
            _ => return 0,
        };
        let b = match objs.get(h2 as usize) {
            Some(o) if o.alive => o,
            _ => return 0,
        };
        // AABB collision
        let (ax1, ay1, ax2, ay2) = obj_bounds(a);
        let (bx1, by1, bx2, by2) = obj_bounds(b);
        if ax1 < bx2 && ax2 > bx1 && ay1 < by2 && ay2 > by1 { 1 } else { 0 }
    })
}

fn obj_bounds(o: &GameObject) -> (f64, f64, f64, f64) {
    match o.kind {
        ObjectKind::Rect => (o.x, o.y, o.x + o.w, o.y + o.h),
        ObjectKind::Circle => {
            let r = o.w; // radius stored in w
            (o.x - r, o.y - r, o.x + r, o.y + r)
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_object_contains(handle: i64, x: f64, y: f64) -> i64 {
    with_object(handle, |o| {
        let (x1, y1, x2, y2) = obj_bounds(o);
        if x >= x1 && x <= x2 && y >= y1 && y <= y2 { 1 } else { 0 }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_object_remove(handle: i64) {
    with_object_mut(handle, |o| { o.alive = false; });
}

// ─── Physics step ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_physics_step() {
    let (screen_w, screen_h) = SDL_STATE.with(|state| {
        state.borrow().as_ref().map(|s| (s.width as f64, s.height as f64)).unwrap_or((800.0, 600.0))
    });

    OBJECTS.with(|objs| {
        let mut objs = objs.borrow_mut();
        for obj in objs.iter_mut() {
            if !obj.alive || (!obj.visible) {
                continue;
            }
            // Apply gravity
            obj.vy += obj.gravity;
            // Apply velocity
            obj.x += obj.vx;
            obj.y += obj.vy;
            // Bouncing off screen edges
            if obj.bounces {
                let (x1, y1, x2, y2) = match obj.kind {
                    ObjectKind::Rect => (obj.x, obj.y, obj.x + obj.w, obj.y + obj.h),
                    ObjectKind::Circle => {
                        let r = obj.w;
                        (obj.x - r, obj.y - r, obj.x + r, obj.y + r)
                    }
                };
                if x1 <= 0.0 || x2 >= screen_w {
                    obj.vx = -obj.vx;
                    // Clamp back inside
                    if x1 <= 0.0 {
                        obj.x -= x1;
                    }
                    if x2 >= screen_w {
                        obj.x -= x2 - screen_w;
                    }
                }
                if y1 <= 0.0 || y2 >= screen_h {
                    obj.vy = -obj.vy;
                    if y1 <= 0.0 {
                        obj.y -= y1;
                    }
                    if y2 >= screen_h {
                        obj.y -= y2 - screen_h;
                    }
                }
            }
        }

        // Bounce off solid objects
        let len = objs.len();
        for i in 0..len {
            if !objs[i].alive || !objs[i].bounces {
                continue;
            }
            for j in 0..len {
                if i == j || !objs[j].alive || !objs[j].solid {
                    continue;
                }
                let (ax1, ay1, ax2, ay2) = obj_bounds(&objs[i]);
                let (bx1, by1, bx2, by2) = obj_bounds(&objs[j]);
                if ax1 < bx2 && ax2 > bx1 && ay1 < by2 && ay2 > by1 {
                    // Compute overlap on each axis to determine bounce direction
                    let overlap_x = (ax2.min(bx2) - ax1.max(bx1)).min(ax2 - ax1);
                    let overlap_y = (ay2.min(by2) - ay1.max(by1)).min(ay2 - ay1);
                    if overlap_x < overlap_y {
                        objs[i].vx = -objs[i].vx;
                        if objs[i].x < objs[j].x {
                            objs[i].x -= overlap_x;
                        } else {
                            objs[i].x += overlap_x;
                        }
                    } else {
                        objs[i].vy = -objs[i].vy;
                        if objs[i].y < objs[j].y {
                            objs[i].y -= overlap_y;
                        } else {
                            objs[i].y += overlap_y;
                        }
                    }
                }
            }
        }
    });
}

// ─── Auto-draw ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_auto_draw() {
    // Collect objects sorted by layer, then draw
    OBJECTS.with(|objs| {
        let objs = objs.borrow();
        // Build sorted index list by layer then creation order
        let mut indices: Vec<usize> = (0..objs.len())
            .filter(|&i| objs[i].alive && objs[i].visible)
            .collect();
        indices.sort_by_key(|&i| objs[i].layer);

        for &i in &indices {
            let o = &objs[i];
            let c = Color::RGB(o.color_r, o.color_g, o.color_b);
            match o.kind {
                ObjectKind::Rect => {
                    with_sdl_mut(|s| {
                        s.canvas.set_draw_color(c);
                        let _ = s.canvas.fill_rect(Rect::new(
                            o.x as i32,
                            o.y as i32,
                            o.w as u32,
                            o.h as u32,
                        ));
                    });
                }
                ObjectKind::Circle => {
                    let r = o.w as i64;
                    with_sdl_mut(|s| {
                        s.canvas.set_draw_color(c);
                        let cx = o.x as i32;
                        let cy = o.y as i32;
                        let mut px = r as i32;
                        let mut py = 0i32;
                        let mut d = 1 - px;
                        while px >= py {
                            let _ = s.canvas.draw_line(Point::new(cx - px, cy + py), Point::new(cx + px, cy + py));
                            let _ = s.canvas.draw_line(Point::new(cx - px, cy - py), Point::new(cx + px, cy - py));
                            let _ = s.canvas.draw_line(Point::new(cx - py, cy + px), Point::new(cx + py, cy + px));
                            let _ = s.canvas.draw_line(Point::new(cx - py, cy - px), Point::new(cx + py, cy - px));
                            py += 1;
                            if d <= 0 {
                                d += 2 * py + 1;
                            } else {
                                px -= 1;
                                d += 2 * (py - px) + 1;
                            }
                        }
                    });
                }
            }
        }
    });
}

// ─── Frame auto (implicit game loop) ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_frame_auto() {
    // 1. Poll input
    runtime_input_poll();
    // 2. Check for quit
    let should_quit = SDL_STATE.with(|state| {
        state.borrow().as_ref().map(|s| s.should_quit).unwrap_or(false)
    });
    if should_quit {
        std::process::exit(0);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_frame_auto_end() {
    // 1. Physics step
    runtime_physics_step();
    // 2. Auto-draw all objects
    runtime_auto_draw();
    // 3. Present
    runtime_screen_present();
    // 4. Frame timing (60 FPS)
    with_sdl_mut(|s| {
        let elapsed = s.frame_start.elapsed();
        s.delta_time = elapsed.as_secs_f64();
        let target = std::time::Duration::from_micros(16667);
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
        s.frame_start = Instant::now();
    });
}

// ─── Screen center properties ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_center_x() -> f64 {
    SDL_STATE.with(|state| {
        state.borrow().as_ref().map(|s| s.width as f64 / 2.0).unwrap_or(400.0)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_center_y() -> f64 {
    SDL_STATE.with(|state| {
        state.borrow().as_ref().map(|s| s.height as f64 / 2.0).unwrap_or(300.0)
    })
}

// ─── Random range ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_math_random_range(min: i64, max: i64) -> i64 {
    if min >= max {
        return min;
    }
    RNG_STATE.with(|rng| {
        let mut state = rng.borrow_mut();
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        min + ((x as i64).abs() % (max - min + 1))
    })
}

// ─── Screen clear with named color support ───

#[unsafe(no_mangle)]
pub extern "C" fn runtime_screen_clear_color(r: i64, g: i64, b: i64) {
    ensure_screen_init();
    runtime_screen_clear(r, g, b);
}
