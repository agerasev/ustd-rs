#![no_std]

pub(crate) mod backend;

pub mod error;
pub mod io;
pub mod sync;
pub mod task;

pub mod prelude {
    pub use super::io::{print, println};
}

#[cfg(test)]
mod tests;
