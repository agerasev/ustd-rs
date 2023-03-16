use super::{
    task::{InterruptContext, TaskContext},
    utils::IntoFreertos,
};
use crate::{
    error::Error,
    task::{BlockingContext, Context},
};
use core::time::Duration;
use freertos::Duration as FreeRtosDuration;

pub trait SemaphoreContext {
    fn semaphore_try_give(&mut self, sem: &Semaphore) -> bool;
    fn semaphore_try_take(&mut self, sem: &Semaphore) -> bool;
}

pub trait SemaphoreBlockingContext: SemaphoreContext {
    fn semaphore_take(&mut self, sem: &Semaphore, timeout: Option<Duration>) -> bool;
}

impl SemaphoreContext for TaskContext {
    fn semaphore_try_give(&mut self, sem: &Semaphore) -> bool {
        sem.0.give()
    }
    fn semaphore_try_take(&mut self, sem: &Semaphore) -> bool {
        match sem.0.take(FreeRtosDuration::zero()) {
            Ok(()) => true,
            Err(Error::Timeout) => false,
            Err(_) => unreachable!(),
        }
    }
}
impl SemaphoreBlockingContext for TaskContext {
    fn semaphore_take(&mut self, sem: &Semaphore, timeout: Option<Duration>) -> bool {
        match sem.0.take(timeout.into_freertos()) {
            Ok(()) => true,
            Err(Error::Timeout) => false,
            Err(_) => unreachable!(),
        }
    }
}
impl SemaphoreContext for InterruptContext {
    fn semaphore_try_give(&mut self, sem: &Semaphore) -> bool {
        sem.0.give_from_isr(&mut self.inner)
    }
    fn semaphore_try_take(&mut self, sem: &Semaphore) -> bool {
        sem.0.take_from_isr(&mut self.inner)
    }
}

pub struct Semaphore(freertos::Semaphore);

impl Semaphore {
    pub fn new() -> Result<Self, Error> {
        freertos::Semaphore::new_binary().map(Self)
    }

    pub fn try_give<C: Context>(&self, cx: &mut C) -> bool {
        cx.semaphore_try_give(self)
    }

    pub fn try_take<C: Context>(&self, cx: &mut C) -> bool {
        cx.semaphore_try_take(self)
    }
    pub fn take<C: BlockingContext>(&self, cx: &mut C, timeout: Option<Duration>) -> bool {
        cx.semaphore_take(self, timeout)
    }
}
