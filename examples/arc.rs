use cstr_core::CString;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl;
use lvgl::style::Style;
use lvgl::widgets::{Arc, Label};
use lvgl::{Align, Color, Display, LvError, Part, Widget};
use lvgl_sys;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

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

fn main() -> Result<(), LvError> {
    const HOR_RES: u32 = 240;
    const VER_RES: u32 = 240;

    println!("meminfo init: {:?}", mem_info());
    let sim_display: Rc<RefCell<SimulatorDisplay<Rgb565>>> = Rc::new(RefCell::new(
        SimulatorDisplay::new(Size::new(HOR_RES, VER_RES)),
    ));

    let output_settings = OutputSettingsBuilder::new().scale(1).build();
    let mut window = Window::new("Arc Example", &output_settings);

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

    let mut screen_style = Style::default();
    screen_style.set_bg_color(Color::from_rgb((255, 255, 255)));
    screen_style.set_radius(0);
    screen.add_style(Part::Main, &mut screen_style);

    let mut arc = Arc::create(&mut screen)?;
    arc.set_size(150, 150);
    arc.set_align(Align::Center, 0, 10);
    arc.set_range(0, 270);
    arc.set_value(0);

    let mut loading_lbl = Label::create(&mut screen)?;
    loading_lbl.set_text(CString::new("Loading...").unwrap().as_c_str());
    loading_lbl.set_align(Align::OutTopMid, 0, 0);

    let mut loading_style = Style::default();
    loading_style.set_text_color(Color::from_rgb((0, 0, 0)));
    loading_lbl.add_style(Part::Main, &mut loading_style);

    let mut i = 0;
    let mut forward = true;

    'running: loop {
        let start = Instant::now();
        if i > 270 {
            forward = !forward;
            i = 1;
            println!("mem info running: {:?}", mem_info());
        }
        arc.set_value(if forward { i } else { 270 - i });
        i += 1;

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
        sleep(Duration::from_millis(15));
        lvgl::tick_inc(Instant::now().duration_since(start));
    }
    println!("meminfo end: {:?}", mem_info());

    Ok(())
}
