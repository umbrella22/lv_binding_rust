//! Custom tick source for LVGL 9.
//!
//! LVGL 8 drove the tick through the `LV_TICK_CUSTOM` compile-time hook
//! (`rs_lv_timer`); LVGL 9 replaces it with the runtime callback
//! [`lvgl_sys::lv_tick_set_cb`]. Calling [`update_clock`] periodically
//! registers that callback on first use and then feeds it, so animations
//! and timers advance from your Rust clock.

use core::num::TryFromIntError;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::time::Duration;

static TICK_MS: AtomicU32 = AtomicU32::new(0);
static TICK_REGISTERED: AtomicBool = AtomicBool::new(false);

extern "C" fn tick_cb() -> u32 {
    TICK_MS.load(Ordering::Relaxed)
}

pub trait LvClock {
    fn since_init(&self) -> Duration;
}

pub fn update_clock(clock: &impl LvClock) -> Result<(), TryFromIntError> {
    TICK_MS.store(
        clock.since_init().as_millis().try_into()?,
        Ordering::Relaxed,
    );
    if !TICK_REGISTERED.swap(true, Ordering::Relaxed) {
        unsafe { lvgl_sys::lv_tick_set_cb(Some(tick_cb)) };
    }
    Ok(())
}
