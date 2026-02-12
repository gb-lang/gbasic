//! G-Basic desktop runtime — extern "C" stubs for the LLVM-compiled programs.

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::cell::RefCell;

thread_local! {
    static SDL_STATE: RefCell<Option<SdlState>> = const { RefCell::new(None) };
}

struct SdlState {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    should_quit: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_init(width: i32, height: i32) {
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
        });
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_clear_screen(r: u8, g: u8, b: u8) {
    SDL_STATE.with(|state| {
        if let Some(ref mut s) = *state.borrow_mut() {
            s.canvas.set_draw_color(Color::RGB(r, g, b));
            s.canvas.clear();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_present() {
    SDL_STATE.with(|state| {
        if let Some(ref mut s) = *state.borrow_mut() {
            s.canvas.present();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_should_quit() -> i32 {
    SDL_STATE.with(|state| {
        if let Some(ref mut s) = *state.borrow_mut() {
            for event in s.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        s.should_quit = true;
                    }
                    _ => {}
                }
            }
            if s.should_quit { 1 } else { 0 }
        } else {
            1
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print(s: *const std::ffi::c_char) {
    if s.is_null() {
        println!();
        return;
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(s) };
    if let Ok(s) = cstr.to_str() {
        println!("{s}");
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

// No-newline variants for string interpolation
#[unsafe(no_mangle)]
pub extern "C" fn runtime_print_str_part(s: *const std::ffi::c_char) {
    if !s.is_null() {
        let cstr = unsafe { std::ffi::CStr::from_ptr(s) };
        if let Ok(s) = cstr.to_str() {
            print!("{s}");
        }
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
