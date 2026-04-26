use crate::functions::CoreError;
use crate::Box;
use crate::Color;
use crate::Screen;
use crate::{disp_get_default, get_str_act, NativeObject};
#[cfg(feature = "nightly")]
use core::error::Error;
use core::fmt;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::ptr::NonNull;
use core::{ptr, result};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DisplayError {
    NotAvailable,
    FailedToRegister,
    NotRegistered,
}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Display {}",
            match self {
                DisplayError::NotAvailable => "not available",
                DisplayError::FailedToRegister => "failed to register",
                DisplayError::NotRegistered => "not registered",
            }
        )
    }
}

#[cfg(feature = "nightly")]
impl Error for DisplayError {}

type Result<T> = result::Result<T, DisplayError>;

pub struct Display {
    pub(crate) disp: NonNull<lvgl_sys::lv_display_t>,
    drop: Option<unsafe extern "C" fn()>,
    owned: bool,
    user_data_drop: Option<unsafe fn(*mut cty::c_void)>,
}

impl Display {
    pub(crate) fn from_raw(
        disp: NonNull<lvgl_sys::lv_display_t>,
        drop: Option<unsafe extern "C" fn()>,
        owned: bool,
        user_data_drop: Option<unsafe fn(*mut cty::c_void)>,
    ) -> Self {
        Self {
            disp,
            drop,
            owned,
            user_data_drop,
        }
    }

    pub fn register<F, const N: usize>(
        hor_res: u32,
        ver_res: u32,
        display_update: F,
    ) -> Result<Self>
    where
        F: FnMut(&DisplayRefresh<N>) + 'static,
    {
        let display_driver = DisplayDriver::new(hor_res, ver_res, display_update)?;
        Ok(display_driver.into_display())
    }

    pub fn get_scr_act(&self) -> Result<Screen<'_>> {
        Ok(get_str_act(Some(self))?.try_into()?)
    }

    pub fn set_scr_act(&self, screen: &mut Screen) {
        let scr_ptr = unsafe { screen.raw().as_mut() };
        unsafe { lvgl_sys::lv_screen_load(scr_ptr) }
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn register_raw<const N: usize>(
        hor_res: u32,
        ver_res: u32,
        flush_cb: Option<
            unsafe extern "C" fn(*mut lvgl_sys::lv_display_t, *const lvgl_sys::lv_area_t, *mut u8),
        >,
        drop: Option<unsafe extern "C" fn()>,
    ) -> Result<Self> {
        let display_driver = DisplayDriver::<N>::new_raw(hor_res, ver_res, flush_cb)?;
        Ok(display_driver.into_display_with_drop(drop))
    }
}

impl Default for Display {
    fn default() -> Self {
        disp_get_default().expect("LVGL must be INITIALIZED")
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                if let Some(drop_user_data) = self.user_data_drop {
                    let user_data = lvgl_sys::lv_display_get_user_data(self.disp.as_ptr());
                    if !user_data.is_null() {
                        drop_user_data(user_data);
                        lvgl_sys::lv_display_set_user_data(self.disp.as_ptr(), ptr::null_mut());
                    }
                }
                lvgl_sys::lv_display_delete(self.disp.as_ptr());
            }
        }

        if let Some(drop) = self.drop {
            unsafe { drop() }
        }
    }
}

pub(crate) fn get_scr_act() -> Result<Screen<'static>> {
    Ok(get_str_act(None)?.try_into()?)
}

pub struct DrawBuffer<const N: usize> {
    _refresh_buffer: Pin<Box<[MaybeUninit<lvgl_sys::lv_color_t>; N]>>,
}

impl<const N: usize> Default for DrawBuffer<N> {
    fn default() -> Self {
        let buf = Box::pin([MaybeUninit::uninit(); N]);
        Self {
            _refresh_buffer: buf,
        }
    }
}

pub(crate) struct DisplayDriver<const N: usize> {
    disp: NonNull<lvgl_sys::lv_display_t>,
    _buffer: DrawBuffer<N>,
    user_data_drop: Option<unsafe fn(*mut cty::c_void)>,
}

impl<const N: usize> DisplayDriver<N> {
    pub fn new<F>(hor_res: u32, ver_res: u32, display_update_callback: F) -> Result<Self>
    where
        F: FnMut(&DisplayRefresh<N>) + 'static,
    {
        let mut draw_buffer = DrawBuffer::default();

        let disp = unsafe {
            let ptr = lvgl_sys::lv_display_create(hor_res as i32, ver_res as i32);
            NonNull::new(ptr).ok_or(DisplayError::FailedToRegister)?
        };

        unsafe {
            let buf_ptr =
                draw_buffer._refresh_buffer.as_mut().get_mut() as *mut _ as *mut cty::c_void;
            lvgl_sys::lv_display_set_buffers(
                disp.as_ptr(),
                buf_ptr,
                ptr::null_mut(),
                (N * core::mem::size_of::<lvgl_sys::lv_color_t>()) as u32,
                lvgl_sys::lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
            );

            lvgl_sys::lv_display_set_flush_cb(disp.as_ptr(), Some(disp_flush_trampoline::<F, N>));
        }

        let user_data = Box::<F>::into_raw(Box::new(display_update_callback)) as *mut cty::c_void;
        unsafe {
            lvgl_sys::lv_display_set_user_data(disp.as_ptr(), user_data);
        }

        Ok(Self {
            disp,
            _buffer: draw_buffer,
            user_data_drop: Some(drop_display_user_data::<F>),
        })
    }

    pub unsafe fn new_raw(
        hor_res: u32,
        ver_res: u32,
        flush_cb: Option<
            unsafe extern "C" fn(*mut lvgl_sys::lv_display_t, *const lvgl_sys::lv_area_t, *mut u8),
        >,
    ) -> Result<Self> {
        let mut draw_buffer = DrawBuffer::default();

        let disp = unsafe {
            let ptr = lvgl_sys::lv_display_create(hor_res as i32, ver_res as i32);
            NonNull::new(ptr).ok_or(DisplayError::FailedToRegister)?
        };

        unsafe {
            let buf_ptr =
                draw_buffer._refresh_buffer.as_mut().get_mut() as *mut _ as *mut cty::c_void;
            lvgl_sys::lv_display_set_buffers(
                disp.as_ptr(),
                buf_ptr,
                ptr::null_mut(),
                (N * core::mem::size_of::<lvgl_sys::lv_color_t>()) as u32,
                lvgl_sys::lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
            );

            if let Some(cb) = flush_cb {
                lvgl_sys::lv_display_set_flush_cb(disp.as_ptr(), Some(cb));
            }
        }

        Ok(Self {
            disp,
            _buffer: draw_buffer,
            user_data_drop: None,
        })
    }

    pub fn into_display(self) -> Display {
        let disp = self.disp;
        let user_data_drop = self.user_data_drop;
        core::mem::forget(self);
        Display::from_raw(disp, None, true, user_data_drop)
    }

    pub fn into_display_with_drop(self, drop: Option<unsafe extern "C" fn()>) -> Display {
        let disp = self.disp;
        let user_data_drop = self.user_data_drop;
        core::mem::forget(self);
        Display::from_raw(disp, drop, true, user_data_drop)
    }
}

unsafe fn drop_display_user_data<F>(user_data: *mut cty::c_void) {
    unsafe {
        drop(Box::<F>::from_raw(user_data as *mut F));
    }
}

pub struct Area {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

pub struct DisplayRefresh<const N: usize> {
    pub area: Area,
    pub colors: [Color; N],
}

#[cfg(feature = "embedded_graphics")]
mod embedded_graphics_impl {
    use crate::{Color, DisplayRefresh};
    use embedded_graphics::prelude::*;
    use embedded_graphics::Pixel;

    impl<const N: usize> DisplayRefresh<N> {
        pub fn as_pixels<C>(&self) -> impl IntoIterator<Item = Pixel<C>> + '_
        where
            C: PixelColor + From<Color>,
        {
            let area = &self.area;
            let x1 = area.x1;
            let x2 = area.x2;
            let y1 = area.y1;
            let y2 = area.y2;

            let ys = y1..=y2;
            let xs = (x1..=x2).enumerate();
            let x_len = (x2 - x1 + 1) as usize;

            ys.enumerate().flat_map(move |(iy, y)| {
                xs.clone().map(move |(ix, x)| {
                    let color_len = x_len * iy + ix;
                    let raw_color = self.colors[color_len];
                    Pixel(Point::new(x as i32, y as i32), raw_color.into())
                })
            })
        }
    }
}

unsafe extern "C" fn disp_flush_trampoline<F, const N: usize>(
    disp: *mut lvgl_sys::lv_display_t,
    area: *const lvgl_sys::lv_area_t,
    px_map: *mut u8,
) where
    F: FnMut(&DisplayRefresh<N>),
{
    let user_data = unsafe { lvgl_sys::lv_display_get_user_data(disp) };
    if !user_data.is_null() {
        let callback = &mut *(user_data as *mut F);

        let mut colors = [Color::default(); N];
        let color_ptr = px_map as *const lvgl_sys::lv_color_t;
        for (i, color) in colors.iter_mut().enumerate() {
            let lv_color = unsafe { *color_ptr.add(i) };
            *color = Color::from_raw(lv_color);
        }

        let update = DisplayRefresh {
            area: unsafe {
                Area {
                    x1: (*area).x1,
                    x2: (*area).x2,
                    y1: (*area).y1,
                    y2: (*area).y2,
                }
            },
            colors,
        };
        callback(&update);
    }
    lvgl_sys::lv_display_flush_ready(disp);
}

impl From<CoreError> for DisplayError {
    fn from(err: CoreError) -> Self {
        use DisplayError::*;
        match err {
            CoreError::ResourceNotAvailable => NotAvailable,
            CoreError::OperationFailed => NotAvailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests;

    #[test]
    fn get_scr_act_return_display() {
        tests::initialize_test(true);
        let _screen = get_str_act(None).expect("We can get the active screen");
    }

    #[test]
    fn get_default_display() {
        tests::initialize_test(true);
        let display = Display::default();
        let _screen_direct = display
            .get_scr_act()
            .expect("Return screen directly from the display instance");
        let _screen_default = get_scr_act().expect("Return screen from the default display");
    }

    #[test]
    fn register_display_directly() -> Result<()> {
        crate::tests::initialize_test(true);
        let display = Display::default();
        let _screen = display
            .get_scr_act()
            .expect("Return screen directly from the display instance");
        Ok(())
    }
}
