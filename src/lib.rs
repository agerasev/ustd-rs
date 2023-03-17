#![no_std]

#[cfg(feature = "backend-freertos")]
pub use freertos;

mod backend;
pub use backend::*;

#[cfg(all(not(feature = "test-freertos"), test))]
mod tests;
