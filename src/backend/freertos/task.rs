extern crate alloc;

use super::utils::IntoFreertos;
use crate::{error::Error, task::Priority};
use alloc::sync::Arc;
use core::time::Duration;
use freertos::FreeRtosTaskHandle;
use lazy_static::lazy_static;
use spin::Mutex as Spin;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct TaskId(FreeRtosTaskHandle);

#[derive(Clone, Debug)]
pub(crate) struct Task(freertos::Task);

impl Task {
    pub fn id(&self) -> TaskId {
        TaskId(self.0.raw_handle())
    }
}

pub(crate) struct Handle {
    task: freertos::Task,
    done: Arc<freertos::Semaphore>,
}

impl Handle {
    pub fn task(&self) -> Task {
        Task(self.task.clone())
    }
    pub fn join(&self, timeout: Option<Duration>) -> bool {
        let done = self.done.take(timeout.into_freertos()).is_ok();
        if done {
            self.done.give();
        }
        done
    }
}

pub(crate) struct Builder(freertos::TaskBuilder);

impl Builder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(freertos::Task::new())
    }
    pub fn name(mut self, name: &str) -> Self {
        self.0.name(name);
        self
    }
    pub fn stack_size(mut self, size: usize) -> Self {
        assert!(size <= u16::MAX as usize);
        self.0.stack_size(size as u16);
        self
    }
    pub fn priority(mut self, priority: Priority) -> Self {
        self.0.priority(freertos::TaskPriority(priority));
        self
    }
    pub fn spawn<F: FnOnce() + Send + 'static>(self, func: F) -> Result<Handle, Error> {
        let done = Arc::new(freertos::Semaphore::new_binary().unwrap());
        self.0
            .start({
                let done = done.clone();
                move |_task| {
                    func();
                    done.give();
                }
            })
            .map(|task| Handle { task, done })
    }
}

pub(crate) fn current() -> Task {
    Task(freertos::Task::current().unwrap())
}

pub(crate) fn sleep(duration: Option<Duration>) {
    freertos::CurrentTask::delay(duration.into_freertos())
}

lazy_static! {
    pub(crate) static ref ISR: Spin<Option<freertos::InterruptContext>> = Spin::new(None);
}

pub(crate) struct Interrupt {
    _unused: [u8; 0],
}

impl Interrupt {
    pub fn new() -> Self {
        let mut guard = ISR.lock();
        assert!(guard.is_none());
        guard.replace(freertos::InterruptContext::new());
        Self { _unused: [] }
    }
}

impl Drop for Interrupt {
    fn drop(&mut self) {
        drop(ISR.lock().take().unwrap());
    }
}
