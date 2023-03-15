use crate::{backend::sync as backend, error::Error};

/// Binary semaphore.
pub struct Semaphore(backend::Semaphore);

impl Semaphore {
    /// Create semaphore in empty state.
    pub fn new() -> Result<Self, Error> {
        backend::Semaphore::new().map(Self)
    }

    /// Release semaphore in non-blocking manner.
    ///
    /// Returns `true` on success, `false` when already given.
    pub fn give(&self) -> bool {
        self.0.give()
    }

    /// Acquire semaphore.
    ///
    /// When `timeout` is `None` then wait infinitely.
    ///
    /// Returns `true` on success, `false` when timed out.
    ///
    /// *When called from interrupt then only non-blocking mode allowed: panics if `timeout` is not zero.*
    pub fn take(&self, timeout: Option<core::time::Duration>) -> bool {
        self.0.take(timeout)
    }
}
