use crate::{
    task::{Context, InterruptContext, TaskContext},
    Error,
};
use core::{marker::PhantomData, ops::ControlFlow, time::Duration};
use freertos::{self, DurationTicks, FreeRtosTickType, FreeRtosUtils};

pub(crate) fn duration_into_freertos(native: Option<Duration>) -> freertos::Duration {
    match native {
        Some(t) => {
            assert!(t.as_millis() < u32::MAX as u128);
            freertos::Duration::ms(t.as_millis() as u32)
        }
        None => freertos::Duration::infinite(),
    }
}
pub(crate) fn duration_from_freertos(freertos: freertos::Duration) -> Option<Duration> {
    if freertos.to_ticks() < freertos::Duration::infinite().to_ticks() {
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
impl TimeContext for TimerContext<'_> {
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
        duration_from_freertos(freertos::Duration::ticks(ticks)).unwrap()
    }
}

pub struct TimerContext<'a> {
    _timer: &'a freertos::Timer,
    /// To ensure `!Sync + !Send`
    _p: PhantomData<*const ()>,
}

impl Context for TimerContext<'_> {}

pub struct TimerBuilder(freertos::TimerBuilder<freertos::Duration>);

impl TimerBuilder {
    pub fn new(period: Duration) -> Self {
        let mut builder = freertos::Timer::new(duration_into_freertos(Some(period)));
        builder.set_auto_reload(true);
        Self(builder)
    }
    pub fn name(mut self, name: &str) -> Self {
        self.0.set_name(name);
        self
    }
    pub fn spawn<F>(self, f: F) -> Result<Timer, Error>
    where
        F: Fn(&mut TimerContext) -> ControlFlow<(), Option<Duration>> + Send + 'static,
    {
        let inner = self.0.create(move |timer| {
            match f(&mut TimerContext {
                _timer: timer,
                _p: PhantomData,
            }) {
                ControlFlow::Break(()) => timer.stop(freertos::Duration::zero()).unwrap(),
                ControlFlow::Continue(new_period_or_same) => match new_period_or_same {
                    None => (),
                    Some(new_period) => timer
                        .change_period(
                            freertos::Duration::zero(),
                            duration_into_freertos(Some(new_period)),
                        )
                        .unwrap(),
                },
            }
        })?;
        inner.start(freertos::Duration::infinite()).unwrap();
        Ok(Timer { inner })
    }
}

pub struct Timer {
    inner: freertos::Timer,
}

impl Timer {
    pub fn stop(&self) {
        // Should not return an error
        self.inner.stop(freertos::Duration::infinite()).unwrap();
    }
}
