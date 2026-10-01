use crate::display::DisplayError;
use crate::Widget;
use core::convert::{TryFrom, TryInto};
#[cfg(feature = "nightly")]
use core::error::Error;
use core::fmt;
use core::ptr::NonNull;
#[cfg(feature = "embedded_graphics")]
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};

pub type LvResult<T> = Result<T, LvError>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum LvError {
    InvalidReference,
    Uninitialized,
    LvOOMemory,
    AlreadyInUse,
}

impl fmt::Display for LvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LvError::InvalidReference => "Accessed invalid reference or ptr",
                LvError::Uninitialized => "LVGL uninitialized",
                LvError::LvOOMemory => "LVGL out of memory",
                LvError::AlreadyInUse => "Resource already in use",
            }
        )
    }
}

#[cfg(feature = "nightly")]
impl Error for LvError {}

impl From<DisplayError> for LvError {
    fn from(err: DisplayError) -> Self {
        use LvError::*;
        match err {
            DisplayError::NotAvailable => Uninitialized,
            DisplayError::FailedToRegister => InvalidReference,
            DisplayError::NotRegistered => Uninitialized,
        }
    }
}

impl From<LvError> for DisplayError {
    fn from(err: LvError) -> Self {
        use DisplayError::*;
        match err {
            LvError::InvalidReference => FailedToRegister,
            LvError::Uninitialized => NotAvailable,
            LvError::LvOOMemory => FailedToRegister,
            LvError::AlreadyInUse => FailedToRegister,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Color {
    pub(crate) raw: lvgl_sys::lv_color_t,
}

impl Color {
    pub fn from_rgb((r, g, b): (u8, u8, u8)) -> Self {
        let raw = unsafe { lvgl_sys::lv_color_make(r, g, b) };
        Self { raw }
    }

    pub fn from_raw(raw: lvgl_sys::lv_color_t) -> Self {
        Self { raw }
    }

    pub fn r(&self) -> u8 {
        self.raw.red
    }

    pub fn g(&self) -> u8 {
        self.raw.green
    }

    pub fn b(&self) -> u8 {
        self.raw.blue
    }
}

#[cfg(feature = "embedded_graphics")]
impl From<Color> for Rgb888 {
    fn from(color: Color) -> Self {
        Rgb888::new(color.r(), color.g(), color.b())
    }
}

#[cfg(feature = "embedded_graphics")]
impl From<Color> for Rgb565 {
    fn from(color: Color) -> Self {
        Rgb565::new(color.r(), color.g(), color.b())
    }
}

impl From<Color> for lvgl_sys::lv_color_t {
    fn from(val: Color) -> Self {
        val.raw
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Event<T> {
    Pressed,
    Pressing,
    PressLost,
    ShortClicked,
    Clicked,
    LongPressed,
    LongPressedRepeat,
    Released,
    ValueChanged,
    DrawMain,
    DrawMainBegin,
    DrawMainEnd,
    DrawPost,
    DrawPostBegin,
    DrawPostEnd,
    Focused,
    Defocused,
    Pointer(PointerEvent),
    Special(T),
}

impl<S> TryFrom<lvgl_sys::lv_event_code_t> for Event<S> {
    type Error = ();

    fn try_from(value: lvgl_sys::lv_event_code_t) -> Result<Self, Self::Error> {
        match value {
            lvgl_sys::lv_event_code_t_LV_EVENT_PRESSED => Ok(Event::Pressed),
            lvgl_sys::lv_event_code_t_LV_EVENT_PRESSING => Ok(Event::Pressing),
            lvgl_sys::lv_event_code_t_LV_EVENT_PRESS_LOST => Ok(Event::PressLost),
            lvgl_sys::lv_event_code_t_LV_EVENT_SHORT_CLICKED => Ok(Event::ShortClicked),
            lvgl_sys::lv_event_code_t_LV_EVENT_CLICKED => Ok(Event::Clicked),
            lvgl_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED => Ok(Event::LongPressed),
            lvgl_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED_REPEAT => Ok(Event::LongPressedRepeat),
            lvgl_sys::lv_event_code_t_LV_EVENT_RELEASED => Ok(Event::Released),
            lvgl_sys::lv_event_code_t_LV_EVENT_VALUE_CHANGED => Ok(Event::ValueChanged),
            lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN => Ok(Event::DrawMain),
            lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN_BEGIN => Ok(Event::DrawMainBegin),
            lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN_END => Ok(Event::DrawMainEnd),
            lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST => Ok(Event::DrawPost),
            lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST_BEGIN => Ok(Event::DrawPostBegin),
            lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST_END => Ok(Event::DrawPostEnd),
            lvgl_sys::lv_event_code_t_LV_EVENT_FOCUSED => Ok(Event::Focused),
            lvgl_sys::lv_event_code_t_LV_EVENT_DEFOCUSED => Ok(Event::Defocused),
            _ => Err(()),
        }
    }
}

impl<S> From<Event<S>> for lvgl_sys::lv_event_code_t {
    fn from(event: Event<S>) -> Self {
        let native_event = match event {
            Event::Pressed => lvgl_sys::lv_event_code_t_LV_EVENT_PRESSED,
            Event::Pressing => lvgl_sys::lv_event_code_t_LV_EVENT_PRESSING,
            Event::PressLost => lvgl_sys::lv_event_code_t_LV_EVENT_PRESS_LOST,
            Event::ShortClicked => lvgl_sys::lv_event_code_t_LV_EVENT_SHORT_CLICKED,
            Event::Clicked => lvgl_sys::lv_event_code_t_LV_EVENT_CLICKED,
            Event::LongPressed => lvgl_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED,
            Event::LongPressedRepeat => lvgl_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED_REPEAT,
            Event::Released => lvgl_sys::lv_event_code_t_LV_EVENT_RELEASED,
            Event::ValueChanged => lvgl_sys::lv_event_code_t_LV_EVENT_VALUE_CHANGED,
            Event::DrawMain => lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN,
            Event::DrawMainBegin => lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN_BEGIN,
            Event::DrawMainEnd => lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN_END,
            Event::DrawPost => lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST,
            Event::DrawPostBegin => lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST_BEGIN,
            Event::DrawPostEnd => lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST_END,
            Event::Focused => lvgl_sys::lv_event_code_t_LV_EVENT_FOCUSED,
            Event::Defocused => lvgl_sys::lv_event_code_t_LV_EVENT_DEFOCUSED,
            _ => lvgl_sys::lv_event_code_t_LV_EVENT_CLICKED,
        };
        native_event as lvgl_sys::lv_event_code_t
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PointerEvent {
    DragBegin,
    DragEnd,
    DragThrowBegin,
}

pub(crate) unsafe extern "C" fn event_callback<'a, T, F>(event: *mut lvgl_sys::lv_event_t)
where
    T: Widget<'a> + Sized,
    F: FnMut(T, Event<<T as Widget<'a>>::SpecialEvent>),
{
    let code = lvgl_sys::lv_event_get_code(event);
    let obj = lvgl_sys::lv_event_get_target_obj(event);
    if let Ok(code) = code.try_into() {
        if let Some(obj_ptr) = NonNull::new(obj) {
            let object = T::from_raw(obj_ptr).unwrap();
            let user_closure = &mut *(lvgl_sys::lv_event_get_user_data(event) as *mut F);
            user_closure(object, code);
        }
    }
}

pub(crate) unsafe extern "C" fn drop_event_callback<F>(event: *mut lvgl_sys::lv_event_t)
{
    let user_data = lvgl_sys::lv_event_get_user_data(event);
    if !user_data.is_null() {
        drop(crate::Box::<F>::from_raw(user_data as *mut F));
        let target = lvgl_sys::lv_event_get_target_obj(event);
        if !target.is_null() {
            lvgl_sys::lv_obj_set_user_data(target, core::ptr::null_mut());
        }
    }
}

pub enum Align {
    Center,
    TopLeft,
    TopMid,
    TopRight,
    BottomLeft,
    BottomMid,
    BottomRight,
    LeftMid,
    RightMid,
    OutTopLeft,
    OutTopMid,
    OutTopRight,
    OutBottomLeft,
    OutBottomMid,
    OutBottomRight,
    OutLeftTop,
    OutLeftMid,
    OutLeftBottom,
    OutRightTop,
    OutRightMid,
    OutRightBottom,
}

impl From<Align> for lvgl_sys::lv_align_t {
    fn from(value: Align) -> Self {
        match value {
            Align::Center => lvgl_sys::lv_align_t_LV_ALIGN_CENTER,
            Align::TopLeft => lvgl_sys::lv_align_t_LV_ALIGN_TOP_LEFT,
            Align::TopMid => lvgl_sys::lv_align_t_LV_ALIGN_TOP_MID,
            Align::TopRight => lvgl_sys::lv_align_t_LV_ALIGN_TOP_RIGHT,
            Align::BottomLeft => lvgl_sys::lv_align_t_LV_ALIGN_BOTTOM_LEFT,
            Align::BottomMid => lvgl_sys::lv_align_t_LV_ALIGN_BOTTOM_MID,
            Align::BottomRight => lvgl_sys::lv_align_t_LV_ALIGN_BOTTOM_RIGHT,
            Align::LeftMid => lvgl_sys::lv_align_t_LV_ALIGN_LEFT_MID,
            Align::RightMid => lvgl_sys::lv_align_t_LV_ALIGN_RIGHT_MID,
            Align::OutTopLeft => lvgl_sys::lv_align_t_LV_ALIGN_OUT_TOP_LEFT,
            Align::OutTopMid => lvgl_sys::lv_align_t_LV_ALIGN_OUT_TOP_MID,
            Align::OutTopRight => lvgl_sys::lv_align_t_LV_ALIGN_OUT_TOP_RIGHT,
            Align::OutBottomLeft => lvgl_sys::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
            Align::OutBottomMid => lvgl_sys::lv_align_t_LV_ALIGN_OUT_BOTTOM_MID,
            Align::OutBottomRight => lvgl_sys::lv_align_t_LV_ALIGN_OUT_BOTTOM_RIGHT,
            Align::OutLeftTop => lvgl_sys::lv_align_t_LV_ALIGN_OUT_LEFT_TOP,
            Align::OutLeftMid => lvgl_sys::lv_align_t_LV_ALIGN_OUT_LEFT_MID,
            Align::OutLeftBottom => lvgl_sys::lv_align_t_LV_ALIGN_OUT_LEFT_BOTTOM,
            Align::OutRightTop => lvgl_sys::lv_align_t_LV_ALIGN_OUT_RIGHT_TOP,
            Align::OutRightMid => lvgl_sys::lv_align_t_LV_ALIGN_OUT_RIGHT_MID,
            Align::OutRightBottom => lvgl_sys::lv_align_t_LV_ALIGN_OUT_RIGHT_BOTTOM,
        }
    }
}

pub enum TextAlign {
    Auto,
    Center,
    Left,
    Right,
}

impl From<TextAlign> for lvgl_sys::lv_text_align_t {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Auto => lvgl_sys::lv_text_align_t_LV_TEXT_ALIGN_AUTO,
            TextAlign::Center => lvgl_sys::lv_text_align_t_LV_TEXT_ALIGN_CENTER,
            TextAlign::Left => lvgl_sys::lv_text_align_t_LV_TEXT_ALIGN_LEFT,
            TextAlign::Right => lvgl_sys::lv_text_align_t_LV_TEXT_ALIGN_RIGHT,
        }
    }
}

pub enum AnimationState {
    ON,
    OFF,
}

impl From<AnimationState> for bool {
    fn from(anim: AnimationState) -> Self {
        match anim {
            AnimationState::ON => true,
            AnimationState::OFF => false,
        }
    }
}

#[repr(u32)]
pub enum LabelLongMode {
    Wrap = lvgl_sys::lv_label_long_mode_t_LV_LABEL_LONG_MODE_WRAP,
    Dots = lvgl_sys::lv_label_long_mode_t_LV_LABEL_LONG_MODE_DOTS,
    Scroll = lvgl_sys::lv_label_long_mode_t_LV_LABEL_LONG_MODE_SCROLL,
    ScrollCircular = lvgl_sys::lv_label_long_mode_t_LV_LABEL_LONG_MODE_SCROLL_CIRCULAR,
    Clip = lvgl_sys::lv_label_long_mode_t_LV_LABEL_LONG_MODE_CLIP,
}

impl From<LabelLongMode> for lvgl_sys::lv_label_long_mode_t {
    fn from(value: LabelLongMode) -> Self {
        value as lvgl_sys::lv_label_long_mode_t
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn color_properties_accessible() {
        let color = Color::from_rgb((206, 51, 255));

        assert_eq!(color.r(), 206);
        assert_eq!(color.g(), 51);
        assert_eq!(color.b(), 255);
    }
}
