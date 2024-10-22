use crate::task::{Context, InterruptContext, TaskContext};
use core::time::Duration;
use freertos::{Duration as FreeRtosDuration, DurationTicks, FreeRtosTickType, FreeRtosUtils};

pub(crate) fn duration_into_freertos(native: Option<Duration>) -> FreeRtosDuration {
    match native {
        Some(t) => {
            assert!(t.as_millis() < u32::MAX as u128);
            FreeRtosDuration::ms(t.as_millis() as u32)
        }
        None => FreeRtosDuration::infinite(),
    }
}
pub(crate) fn duration_from_freertos(freertos: FreeRtosDuration) -> Option<Duration> {
    if freertos.to_ticks() < FreeRtosDuration::infinite().to_ticks() {
        Some(Duration::from_millis(freertos.to_ms() as u64))
    } else {
        None
    }
}

mod sealed {
    use freertos::FreeRtosTickType;

    pub trait TimeContext {
        fn get_tick_count(&mut self) -> FreeRtosTickType;
    }
}
pub(crate) use sealed::TimeContext;

impl TimeContext for TaskContext {
    fn get_tick_count(&mut self) -> FreeRtosTickType {
        FreeRtosUtils::get_tick_count()
    }
}
impl TimeContext for InterruptContext {
    fn get_tick_count(&mut self) -> FreeRtosTickType {
        // TODO: Impl get_tick_count from ISR in FreeRTOS-Rust
        unimplemented!()
    }
}

pub struct Instant {
    start: FreeRtosTickType,
}

impl Instant {
    pub fn now(cx: &mut impl Context) -> Self {
        Self {
            start: cx.get_tick_count(),
        }
    }
    pub fn elapsed(&self, cx: &mut impl Context) -> Duration {
        let now = cx.get_tick_count();
        let ticks = now.wrapping_sub(self.start);
        duration_from_freertos(FreeRtosDuration::ticks(ticks)).unwrap()
    }
}
