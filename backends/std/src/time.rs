extern crate std;

pub use std::time::Instant;

use std::{
    marker::PhantomData,
    ops::ControlFlow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, sleep, Thread},
    time::Duration,
};

use crate::{task::Context, Error};

#[derive(Default)]
struct TimerState {
    stopped: AtomicBool,
}

pub struct TimerContext<'a> {
    _timer: &'a Timer,
    /// To ensure `!Sync + !Send`
    _p: PhantomData<*const ()>,
}

impl Context for TimerContext<'_> {}

pub struct TimerBuilder {
    inner: thread::Builder,
    period: Duration,
}

impl TimerBuilder {
    pub fn new(period: Duration) -> Self {
        Self {
            inner: thread::Builder::new(),
            period,
        }
    }
    pub fn name(self, name: &str) -> Self {
        Self {
            inner: self.inner.name(name.to_string()),
            ..self
        }
    }
    pub fn spawn<F>(self, f: F) -> Result<Timer, Error>
    where
        F: Fn(&mut TimerContext) -> ControlFlow<(), Option<Duration>> + Send + 'static,
    {
        let state = Arc::<TimerState>::default();
        let thread = self
            .inner
            .spawn({
                let state = state.clone();
                move || {
                    let timer = Timer {
                        thread: thread::current(),
                        state: state.clone(),
                    };
                    let mut cx = TimerContext {
                        _timer: &timer,
                        _p: PhantomData,
                    };
                    let mut period = self.period;
                    loop {
                        sleep(period);
                        if state.stopped.load(Ordering::Acquire) {
                            break;
                        }
                        match f(&mut cx) {
                            ControlFlow::Break(()) => break,
                            ControlFlow::Continue(new_period_or_same) => match new_period_or_same {
                                None => (),
                                Some(new_period) => {
                                    period = new_period;
                                }
                            },
                        }
                    }
                }
            })?
            .thread()
            .clone();
        Ok(Timer { thread, state })
    }
}

pub struct Timer {
    #[allow(dead_code)]
    thread: Thread,
    state: Arc<TimerState>,
}

impl Timer {
    pub fn stop(&self) {
        self.state.stopped.store(true, Ordering::Release);
    }
}
