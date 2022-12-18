#![no_std]

#[cfg(feature = "backend-std")]
mod std;
#[cfg(feature = "backend-std")]
pub use std::*;
