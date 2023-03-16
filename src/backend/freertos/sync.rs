use super::{task::ISR, utils::IntoFreertos};
use crate::error::Error;
use core::time::Duration;

pub struct Semaphore(freertos::Semaphore);

impl Semaphore {
    pub fn new() -> Result<Self, Error> {
        freertos::Semaphore::new_binary().map(Self)
    }

    pub fn give(&self) -> bool {
        if let Some(isr) = ISR.lock().as_mut() {
            return self.0.give_from_isr(isr);
        }
        self.0.give()
    }

    /// Returns `true` on success.
    pub fn take(&self, timeout: Option<Duration>) -> bool {
        if let Some(isr) = ISR.lock().as_mut() {
            assert_eq!(timeout, Some(Duration::ZERO));
            return self.0.take_from_isr(isr);
        }
        match self.0.take(timeout.into_freertos()) {
            Ok(()) => true,
            Err(Error::Timeout) => false,
            Err(_) => unreachable!(),
        }
    }
}
