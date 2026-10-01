use cstr_core::CString;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl;
use lvgl::font::Font;
use lvgl::style::Style;
use lvgl::widgets::Label;
use lvgl::{Align, Color, Display, LvError, Part, TextAlign, Widget};
use lvgl_sys;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

fn main() -> Result<(), LvError> {
    const HOR_RES: u32 = 240;
    const VER_RES: u32 = 240;

    let sim_display: Rc<RefCell<SimulatorDisplay<Rgb565>>> = Rc::new(RefCell::new(
        SimulatorDisplay::new(Size::new(HOR_RES, VER_RES)),
    ));
    let output_settings = OutputSettingsBuilder::new().scale(1).build();
    let mut window = Window::new("PineTime", &output_settings);

    let display = Display::register::<_, { (HOR_RES * VER_RES) as usize }>(HOR_RES, VER_RES, {
        let sim_display = Rc::clone(&sim_display);
        move |refresh| {
            sim_display
                .borrow_mut()
                .draw_iter(refresh.as_pixels())
                .unwrap();
        }
    })?;

    let mut screen = display.get_scr_act()?;

    println!("Before all widgets: {:?}", mem_info());

    let mut screen_style = Style::default();
    screen_style.set_bg_color(Color::from_rgb((0, 0, 0)));
    screen_style.set_radius(0);
    screen.add_style(Part::Main, &mut screen_style);

    let mut time = Label::from("20:46");
    let mut style_time = Style::default();
    style_time.set_text_color(Color::from_rgb((255, 255, 255)));
    style_time.set_text_align(TextAlign::Center);

    style_time.set_text_font(unsafe { Font::new_raw(lvgl_sys::lv_font_montserrat_14) });

    time.add_style(Part::Main, &mut style_time);
    time.set_align(Align::Center, 0, 90);
    time.set_width(240);
    time.set_height(240);

    let mut bt = Label::from("#5794f2 \u{F293}#");
    bt.set_width(50);
    bt.set_height(80);
    bt.set_recolor(true);
    bt.set_align(Align::TopLeft, 0, 0);

    let mut power: Label = "#fade2a 20%#".into();
    power.set_recolor(true);
    power.set_width(80);
    power.set_height(20);
    power.set_align(Align::TopRight, 40, 0);

    let mut i = 0;
    'running: loop {
        let start = Instant::now();
        if i > 59 {
            i = 0;
        }
        let val = CString::new(format!("21:{:02}", i)).unwrap();
        time.set_text(val.as_c_str());
        i = 1 + i;

        lvgl::task_handler();
        {
            let d = sim_display.borrow();
            window.update(&*d);
        }

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }

        sleep(Duration::from_secs(1));
        lvgl::tick_inc(Instant::now().duration_since(start));
    }

    println!("Final part of demo app: {:?}", mem_info());

    Ok(())
}

fn mem_info() -> lvgl_sys::lv_mem_monitor_t {
    let mut info = lvgl_sys::lv_mem_monitor_t {
        total_size: 0,
        free_cnt: 0,
        free_size: 0,
        free_biggest_size: 0,
        used_cnt: 0,
        max_used: 0,
        used_pct: 0,
        frag_pct: 0,
    };
    unsafe {
        lvgl_sys::lv_mem_monitor(&mut info as *mut _);
    }
    info
}
