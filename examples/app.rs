use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use lvgl;
use lvgl::Display;

type ColorSpace = Rgb565;

#[allow(unused_mut)]
#[allow(unused_variables)]
fn main() {
    const HOR_RES: u32 = 240;
    const VER_RES: u32 = 240;

    let mut embedded_graphics_display: SimulatorDisplay<ColorSpace> =
        SimulatorDisplay::new(Size::new(HOR_RES, VER_RES));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("App Example", &output_settings);

    let _display = Display::register::<_, { (HOR_RES * VER_RES) as usize }>(
        HOR_RES,
        VER_RES,
        move |refresh| {
            embedded_graphics_display
                .draw_iter(refresh.as_pixels())
                .unwrap();
        },
    )
    .unwrap();
}
