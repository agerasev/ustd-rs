#[cfg(feature = "backend-std")]
mod std;
#[cfg(feature = "backend-std")]
pub use self::std::*;

#[cfg(feature = "backend-freertos")]
mod freertos;
#[cfg(feature = "backend-freertos")]
pub use self::freertos::*;

pub mod prelude {
    pub use super::{print, println};
}
