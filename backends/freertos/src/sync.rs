use super::task::{InterruptContext, TaskContext};
use crate::{
    error::Error,
    task::{BlockingContext, Context},
    time::duration_into_freertos,
};
use core::time::Duration;
use freertos::Duration as FreeRtosDuration;

mod sealed {
    use freertos::{Duration as FreeRtosDuration, Semaphore};

    pub trait SemaphoreContext {
        fn semaphore_try_give(&mut self, sem: &Semaphore) -> bool;
        fn semaphore_try_take(&mut self, sem: &Semaphore) -> bool;
    }

    pub trait SemaphoreBlockingContext: SemaphoreContext {
        fn semaphore_take(&mut self, sem: &Semaphore, timeout: FreeRtosDuration) -> bool;
    }
}

pub(crate) use sealed::{SemaphoreBlockingContext, SemaphoreContext};

impl SemaphoreContext for TaskContext {
    fn semaphore_try_give(&mut self, sem: &freertos::Semaphore) -> bool {
        sem.give()
    }
    fn semaphore_try_take(&mut self, sem: &freertos::Semaphore) -> bool {
        match sem.take(FreeRtosDuration::zero()) {
            Ok(()) => true,
            Err(Error::Timeout) => false,
            Err(_) => unreachable!(),
        }
    }
}
impl SemaphoreBlockingContext for TaskContext {
    fn semaphore_take(&mut self, sem: &freertos::Semaphore, timeout: FreeRtosDuration) -> bool {
        match sem.take(timeout) {
            Ok(()) => true,
            Err(Error::Timeout) => false,
            Err(_) => unreachable!(),
        }
    }
}
impl SemaphoreContext for InterruptContext {
    fn semaphore_try_give(&mut self, sem: &freertos::Semaphore) -> bool {
        sem.give_from_isr(&mut self.inner)
    }
    fn semaphore_try_take(&mut self, sem: &freertos::Semaphore) -> bool {
        sem.take_from_isr(&mut self.inner)
    }
}

pub struct Semaphore(freertos::Semaphore);

impl Semaphore {
    pub fn new() -> Result<Self, Error> {
        freertos::Semaphore::new_binary().map(Self)
    }

    pub fn try_give<C: Context>(&self, cx: &mut C) -> bool {
        cx.semaphore_try_give(&self.0)
    }

    pub fn try_take<C: Context>(&self, cx: &mut C) -> bool {
        cx.semaphore_try_take(&self.0)
    }
    pub fn take<C: BlockingContext>(&self, cx: &mut C, timeout: Option<Duration>) -> bool {
        cx.semaphore_take(&self.0, duration_into_freertos(timeout))
    }
}
