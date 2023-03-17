#![no_std]

#[cfg(feature = "backend-freertos")]
pub use freertos;

mod backend;
pub use backend::*;

pub mod prelude {
    pub use super::task::{BlockingContext, Context};
}

#[cfg_attr(not(feature = "test-freertos"), cfg(test))]
pub mod tests;
