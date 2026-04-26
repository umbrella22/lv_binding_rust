use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl;
use lvgl::style::Style;
use lvgl::widgets::Scale;
use lvgl::{Align, Color, Display, LvError, Part, Widget};
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

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Scale Example", &output_settings);

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
    screen_style.set_bg_color(Color::from_rgb((0, 0, 0)));
    screen.add_style(Part::Main, &mut screen_style);

    let mut scale_style = Style::default();
    scale_style.set_radius(5);
    scale_style.set_bg_color(Color::from_rgb((192, 192, 192)));
    scale_style.set_pad_top(20);
    scale_style.set_pad_left(5);
    scale_style.set_pad_right(5);
    scale_style.set_line_color(Color::from_rgb((255, 255, 255)));
    scale_style.set_line_width(2);

    let mut scale = Scale::create(&mut screen)?;
    scale.add_style(Part::Main, &mut scale_style);
    scale.set_align(Align::Center, 0, 0);
    scale.set_range(0, 100);

    let mut i = 0;
    'running: loop {
        let start = Instant::now();

        lvgl::task_handler();
        {
            let d = sim_display.borrow();
            window.update(&*d);
        }

        for event in window.events() {
            match event {
                SimulatorEvent::MouseButtonUp {
                    mouse_btn: _,
                    point,
                } => {
                    println!("Clicked on: {:?}", point);
                }
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }

        if i > 99 {
            i = 0;
        } else {
            i += 1;
        }

        sleep(Duration::from_millis(16));
        lvgl::tick_inc(Instant::now().duration_since(start));
    }

    Ok(())
}
