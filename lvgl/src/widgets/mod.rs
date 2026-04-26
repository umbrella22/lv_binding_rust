use crate::lv_core::obj::NativeObject;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub use self::arc::ArcMode;
pub use self::bar::{BarMode, BarOrientation};
pub use self::slider::{SliderMode, SliderOrientation};

mod arc;
mod bar;
mod keyboard;
mod label;
mod slider;
mod table;
