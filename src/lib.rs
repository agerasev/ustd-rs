#![no_std]

#[cfg(feature = "backend-freertos")]
pub use freertos;

mod backend;
pub use backend::*;

pub mod task;

pub use io::{print, println};
pub mod prelude {
    pub use super::io::{print, println};
    pub use super::task::{BlockingContext, Context};
}

#[cfg(any(test, feature = "test-freertos"))]
pub mod tests;
