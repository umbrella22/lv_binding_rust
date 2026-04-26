pub fn pct(x: i32) -> i32 {
    unsafe { lvgl_sys::lv_pct(x) }
}
