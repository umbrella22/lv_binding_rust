use core::num::TryFromIntError;
use core::time::Duration;

static mut RET_VAL: u32 = 0;

pub trait LvClock {
    fn since_init(&self) -> Duration;
}

pub fn update_clock(clock: &impl LvClock) -> Result<(), TryFromIntError> {
    unsafe { RET_VAL = clock.since_init().as_millis().try_into()? }
    Ok(())
}

#[no_mangle]
unsafe extern "C" fn rs_lv_timer() -> u32 {
    RET_VAL
}
