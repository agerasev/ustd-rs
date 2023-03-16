use crate::{backend::task as backend, error::Error};
use core::{marker::PhantomData, time::Duration};

/// Unique task identifier.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct TaskId(backend::TaskId);

/// Task priority.
pub type Priority = u8;

/// Task itself.
#[derive(Clone)]
pub struct Task(backend::Task);

impl Task {
    pub fn id(&self) -> TaskId {
        TaskId(self.0.id())
    }
}

/// Task handle.
pub struct Handle(backend::Handle);

impl Handle {
    pub fn task(&self) -> Task {
        Task(self.0.task())
    }
    /// Wait for task to finish.
    pub fn join(&self, timeout: Option<Duration>) -> bool {
        self.0.join(timeout)
    }
}

/// Task builder.
pub struct Builder(backend::Builder);

impl Builder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(backend::Builder::new())
    }
    pub fn name(self, name: &str) -> Self {
        Self(self.0.name(name))
    }
    pub fn stack_size(self, size: usize) -> Self {
        Self(self.0.stack_size(size))
    }
    pub fn priority(self, priority: Priority) -> Self {
        Self(self.0.priority(priority))
    }
    pub fn spawn<F: FnOnce() + Send + 'static>(self, func: F) -> Result<Handle, Error> {
        self.0.spawn(func).map(Handle)
    }
}

/// Spawn a new task.
pub fn spawn<F: FnOnce() + Send + 'static>(func: F) -> Result<Handle, Error> {
    Builder::new().spawn(func)
}

/// Get current task.
///
/// *Panics if called in ISR.*
///
/// *Allowed to call in execution units created only by [`spawn`], [`Builder::spawn`], `#[ustd::main]` or `#[ustd::test]`.*
pub fn current() -> Task {
    Task(backend::current())
}

/// Sleep for specified `duration`.
///
/// If `None` then sleep infinetely.
pub fn sleep(duration: Option<Duration>) {
    backend::sleep(duration);
}

/// Interrupt context.
///
/// While exists marks current execution unit as interrupt.
///
/// *Panic caused if more than one interrupt context exists at the same time in the same execution unit.*
pub struct Interrupt {
    #[allow(dead_code)]
    inner: backend::Interrupt,
    /// To ensure `!Sync + !Send`
    _p: PhantomData<*const ()>,
}

impl Interrupt {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: backend::Interrupt::new(),
            _p: PhantomData,
        }
    }
}
