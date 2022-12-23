use crate::Error;
use core::time::Duration;

pub struct Semaphore {
    semaphore: freertos::Semaphore,
}

impl Semaphore {
    pub fn new() -> Result<Self, Error> {
        freertos::Semaphore::new_binary().map(|sem| Self { semaphore: sem })
    }

    /// Returns `true` on success.
    pub fn try_give(&self) -> bool {
        self.semaphore.give()
    }

    /// Returns `true` on success.
    pub fn try_take(&self) -> bool {
        match self.semaphore.take(freertos::Duration::zero()) {
            Ok(()) => true,
            Err(Error::Timeout) => false,
            Err(other) => unreachable!("Unexpected error: {:?}", other),
        }
    }

    pub fn take(&self) {
        self.semaphore.take(freertos::Duration::infinite()).unwrap()
    }

    /// Returns `true` on success, `false` when timed out.
    pub fn take_timeout(&self, timeout: Duration) -> bool {
        match self
            .semaphore
            .take(freertos::Duration::ms(timeout.as_millis() as u32))
        {
            Ok(()) => true,
            Err(Error::Timeout) => false,
            Err(other) => unreachable!("Unexpected error: {:?}", other),
        }
    }
}
