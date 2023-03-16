use crate::error::Error;
use core::time::Duration;

pub use crate::backend::task::*;

/// Basic execution context.
pub trait Context {}

/// Context that allows to make blocking calls.
pub trait BlockingContext: Context {
    fn sleep(&mut self, duration: Option<Duration>);
}

/// Task priority.
pub type Priority = u8;

/// Spawn a new task.
pub fn spawn<F: FnOnce(&mut TaskContext) + Send + 'static>(func: F) -> Result<Handle, Error> {
    Builder::new().spawn(func)
}
