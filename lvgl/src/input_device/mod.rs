use crate::LvResult;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Data {
    Pointer(PointerInputData),
    Encoder(EncoderInputData),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum InputState {
    Pressed(Data),
    Released(Data),
}

impl InputState {
    pub fn once(self) -> BufferStatus {
        BufferStatus::Once(self)
    }

    pub fn and_continued(self) -> BufferStatus {
        BufferStatus::Buffered(self)
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BufferStatus {
    Once(InputState),
    Buffered(InputState),
}

pub trait InputDriver<D> {
    fn register<F>(handler: F, display: &crate::Display) -> LvResult<D>
    where
        F: Fn() -> BufferStatus;

    fn get_descriptor(&self) -> Option<*mut lvgl_sys::lv_indev_t>;

    unsafe fn set_descriptor(&mut self, descriptor: *mut lvgl_sys::lv_indev_t) -> LvResult<()>;
}

mod encoder;
mod pointer;

pub use encoder::*;
pub use pointer::*;
