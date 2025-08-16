use super::task::{InterruptContext, TaskContext};
use crate::{
    error::Error,
    task::{BlockingContext, Context},
    time::{duration_into_freertos, TimerContext},
};
use core::{
    ops::{Deref, DerefMut},
    time::Duration,
};
use freertos::{Duration as FreeRtosDuration, FreeRtosError};

mod sealed {
    use freertos::{Duration as FreeRtosDuration, Semaphore};

    pub trait SyncContext {
        fn semaphore_try_give(&mut self, sem: &Semaphore) -> bool;
        fn semaphore_try_take(&mut self, sem: &Semaphore) -> bool;
    }

    pub trait SyncBlockingContext: SyncContext {
        fn semaphore_take(&mut self, sem: &Semaphore, timeout: FreeRtosDuration) -> bool;
    }
}

pub(crate) use sealed::{SyncBlockingContext, SyncContext};

impl SyncContext for TaskContext {
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
impl SyncContext for TimerContext<'_> {
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
impl SyncBlockingContext for TaskContext {
    fn semaphore_take(&mut self, sem: &freertos::Semaphore, timeout: FreeRtosDuration) -> bool {
        match sem.take(timeout) {
            Ok(()) => true,
            Err(Error::Timeout) => false,
            Err(_) => unreachable!(),
        }
    }
}
impl SyncContext for InterruptContext {
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

pub struct Mutex<T>(freertos::Mutex<T>);

impl<T> Mutex<T> {
    pub fn new(value: T) -> Result<Self, Error> {
        freertos::Mutex::new(value).map(Self)
    }

    pub fn try_lock(&self, _cx: &mut TaskContext) -> Result<Option<MutexGuard<'_, T>>, Error> {
        match self.0.lock(freertos::Duration::zero()) {
            Ok(guard) => Ok(Some(MutexGuard(guard))),
            Err(FreeRtosError::Timeout | FreeRtosError::MutexTimeout) => Ok(None),
            Err(other) => Err(other),
        }
    }
    pub fn lock(
        &self,
        _cx: &mut TaskContext,
        timeout: Option<Duration>,
    ) -> Result<MutexGuard<'_, T>, Error> {
        self.0.lock(duration_into_freertos(timeout)).map(MutexGuard)
    }
}

pub struct MutexGuard<'a, T>(freertos::MutexGuard<'a, T, freertos::MutexNormal>);

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
