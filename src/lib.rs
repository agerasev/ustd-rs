#![no_std]

#[cfg(not(feature = "freertos"))]
pub use backend_std::*;

#[cfg(feature = "freertos")]
pub use backend_freertos::*;
