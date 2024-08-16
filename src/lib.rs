#![no_std]

#[cfg(not(any(feature = "backend-std", feature = "backend-freertos")))]
compile_error!("A backend must be selected");

#[cfg(all(feature = "backend-std", feature = "backend-freertos"))]
compile_error!("Only one backend at a time allowed");

#[cfg(feature = "backend-freertos")]
pub use freertos;

mod backend;
pub use backend::*;

#[cfg(all(not(feature = "test-freertos"), test))]
mod tests;
