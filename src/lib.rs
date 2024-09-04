#![no_std]

#[cfg(not(any(feature = "std", feature = "freertos")))]
compile_error!("A backend must be selected");

//#[cfg(all(feature = "std", feature = "freertos"))]
//compile_error!("Only one backend at a time allowed");

#[cfg(all(feature = "std", not(feature = "freertos")))]
pub use backend_std::*;

#[cfg(feature = "freertos")]
pub use backend_freertos::*;
