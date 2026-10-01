//! LVGL desktop-simulator demo driving LVGL's native SDL2 backend
//! (`lv_sdl_window`) from the vendored LVGL 9.6 tree.
//!
//! Run with:
//! ```text
//! cargo run -p lvgl --example sdl_demo --features sdl
//! ```
//!
//! Opens a 480x320 window, animates a bar + counter label for ~2.5 s,
//! verifies real pixel output with `lv_snapshot_take`, then exits.

use cstr_core::CString;
use std::ptr::NonNull;
use std::thread::sleep;
use std::time::Duration;

use lvgl::style::Style;
use lvgl::widgets::{Bar, Label};
use lvgl::{AnimationState, Color, LvError, NativeObject, Part, Screen, Widget};
use lvgl_sys as sys;

const HOR_RES: i32 = 480;
const VER_RES: i32 = 320;
const FRAMES: u32 = 400;

fn main() -> Result<(), LvError> {
    unsafe {
        sys::lv_init();

        if sys::lv_sdl_window_create(HOR_RES, VER_RES).is_null() {
            panic!("lv_sdl_window_create failed");
        }
        if sys::lv_sdl_mouse_create().is_null() {
            panic!("lv_sdl_mouse_create failed");
        }
    }
    println!("SDL window created: {HOR_RES}x{VER_RES} (mouse input attached)");

    // The SDL backend created the display itself; wrap its active screen so
    // the safe widget API can build the UI on it.
    let raw_screen = unsafe { sys::lv_screen_active() };
    let mut screen = unsafe {
        Screen::from_raw(NonNull::new(raw_screen).expect("SDL window has no active screen"))
    }
    .expect("failed to wrap the SDL active screen");

    let mut bg = Style::default();
    bg.set_bg_color(Color::from_rgb((0x10, 0x18, 0x28)));
    screen.add_style(Part::Main, &mut bg);

    let mut title = Label::create(&mut screen)?;
    title.set_text(CString::new("LVGL 9.6.0 x SDL2 simulator").unwrap().as_c_str());
    let mut title_style = Style::default();
    title_style.set_text_color(Color::from_rgb((0xE8, 0xE8, 0xE8)));
    title.add_style(Part::Main, &mut title_style);

    let mut bar = Bar::create(&mut screen)?;
    bar.set_size(300, 24);
    bar.set_range(0, 100);

    let mut status = Label::create(&mut screen)?;
    status.set_text(CString::new("frame 0").unwrap().as_c_str());

    // If the UI never produces a single non-black pixel, rendering is broken;
    // snapshot the screen mid-run and count non-zero bytes.
    let mut snapshot_checked = false;

    for frame in 0..FRAMES {
        unsafe { sys::lv_timer_handler() };
        sleep(Duration::from_millis(5));

        if frame % 20 == 0 {
            bar.set_value((frame * 100 / FRAMES) as i32, AnimationState::OFF);
        }
        if frame % 50 == 0 {
            status.set_text(CString::new(format!("frame {frame}")).unwrap().as_c_str());
        }
        if frame == 150 && !snapshot_checked {
            snapshot_checked = true;
            let buf = unsafe {
                sys::lv_snapshot_take(
                    screen.raw().as_mut(),
                    sys::lv_color_format_t_LV_COLOR_FORMAT_ARGB8888,
                )
            };
            assert!(!buf.is_null(), "lv_snapshot_take returned null");
            let (non_zero, total) = unsafe {
                let size = (*buf).data_size as usize;
                let data = std::slice::from_raw_parts((*buf).data, size);
                let nz = data.iter().filter(|b| **b != 0).count();
                sys::lv_draw_buf_destroy(buf);
                (nz, size)
            };
            println!("snapshot: {non_zero}/{total} non-zero bytes (ARGB8888)");
            assert!(non_zero > 0, "screen rendered no pixels");
        }
    }

    println!("sdl_demo finished: {FRAMES} frames pumped, rendering verified");
    Ok(())
}
