//! LVGL font handling logic and helper structures.
//!
//! Fonts can be built into LVGL or custom-generated with the [official online
//! converter].
//!
//! # Built-in fonts
//! LVGL offers several fonts for Latin (only up to ASCII), Arabic, Persian,
//! and Hebrew, along with CJK glyphs. These can be enabled in `lv_conf.h` and
//! once enabled will be usable as-is:
//! ```ignore
//! use lvgl::font::Font;
//! use lvgl::style::Style;
//!
//! fn main() {
//!     let mut my_style = Style::default();
//!     my_style.set_text_font(Font::montserrat_48());
//!     // Use the style
//! }
//! ```
//!
//! Built-in fonts are *only* available if the `nightly` feature is enabled for
//! the `lvgl` crate.
//!
//! # Custom fonts
//! Fonts converted to a C source file (e.g. with the [official online
//! converter]) can be compiled into `lvgl-sys` by dropping the file into a
//! directory included by its build script, or linked in by the final
//! application. The symbol can then unsafely be converted into a `Font`
//! struct. Built-in fonts enabled in `lv_conf.h` (e.g. `montserrat_14`,
//! which this crate enables by default) work the same way:
//! ```
//! use lvgl::font::Font;
//! use lvgl::style::Style;
//!
//! let montserrat_14 = unsafe {
//!     Font::new_raw(lvgl_sys::lv_font_montserrat_14)
//! };
//! let mut my_style = Style::default();
//! my_style.set_text_font(montserrat_14);
//! // Use the style
//! ```
//! This operation is inherently unsafe as it instantiates and uses arbitrary
//! data structures that the Rust compiler can't check.
//!
//! [official online converter]: https://lvgl.io/tools/fontconverter

mod generic;
pub use generic::*;

#[cfg(feature = "nightly")]
mod builtin;
