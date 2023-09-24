extern crate alloc;

use super::sync::{SemaphoreBlockingContext, SemaphoreContext};
use crate::{
    error::Error,
    time::{duration_into_freertos, TimeContext},
};
use alloc::sync::Arc;
use core::{marker::PhantomData, time::Duration};
use freertos::FreeRtosTaskHandle;

pub trait Context: SemaphoreContext + TimeContext {}

pub trait BlockingContext: Context + SemaphoreBlockingContext {
    fn sleep(&mut self, duration: Option<Duration>);
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct TaskId(FreeRtosTaskHandle);

pub type Priority = u8;

#[derive(Clone, Debug)]
pub struct Task(freertos::Task);

impl Task {
    pub fn id(&self) -> TaskId {
        TaskId(self.0.raw_handle())
    }
}

pub struct Handle {
    task: freertos::Task,
    done: Arc<freertos::Semaphore>,
}

pub struct TaskContext {
    task: freertos::Task,
    done: Arc<freertos::Semaphore>,
    /// To ensure `!Sync + !Send`
    _p: PhantomData<*const ()>,
}

impl Handle {
    pub fn task(&self) -> Task {
        Task(self.task.clone())
    }
    pub fn join<C: BlockingContext>(&self, _cx: &mut C, timeout: Option<Duration>) -> bool {
        let done = self.done.take(duration_into_freertos(timeout)).is_ok();
        if done {
            self.done.give();
        }
        done
    }
}

impl TaskContext {
    pub fn task(&mut self) -> Task {
        Task(self.task.clone())
    }
}

impl Context for TaskContext {}

impl BlockingContext for TaskContext {
    fn sleep(&mut self, duration: Option<Duration>) {
        freertos::CurrentTask::delay(duration_into_freertos(duration))
    }
}

pub struct InterruptContext {
    pub(crate) inner: freertos::InterruptContext,
    /// To ensure `!Sync + !Send`
    _p: PhantomData<*const ()>,
}

impl InterruptContext {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: freertos::InterruptContext::new(),
            _p: PhantomData,
        }
    }
}

impl Context for InterruptContext {}

pub struct Builder(freertos::TaskBuilder);

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
    pub fn spawn<F: FnOnce(&mut TaskContext) + Send + 'static>(self, func: F) -> Result<Handle, Error> {
        let done = Arc::new(freertos::Semaphore::new_binary().unwrap());
        self.0
            .start({
                let done = done.clone();
                move |task| {
                    let mut cx = TaskContext {
                        task,
                        done,
                        _p: PhantomData,
                    };
                    func(&mut cx);
                    cx.done.give();
                }
            })
            .map(|task| Handle { task, done })
    }
}

pub fn spawn<F: FnOnce(&mut TaskContext) + Send + 'static>(func: F) -> Result<Handle, Error> {
    Builder::new().spawn(func)
}
