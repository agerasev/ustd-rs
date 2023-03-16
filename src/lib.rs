#![no_std]

#[cfg(feature = "backend-freertos")]
pub use freertos;

pub mod backend;

pub mod context;
pub mod error;
pub mod io;
pub mod sync;
pub mod task;

pub use io::{print, println};

#[cfg(any(test, feature = "test-freertos"))]
pub mod tests;
