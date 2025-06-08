extern crate std;

use std::{io::Error, io::Write};

pub use crate::io::{stdout, Stdout};

pub use ufmt::*;

impl uWrite for Stdout {
    type Error = Error;
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_all(s.as_bytes())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use $crate::fmt::{self as ufmt, uwrite};
        let _ = uwrite!($crate::io::stdout(), $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::print!("\n");
    }};
    ($($arg:tt)*) => {{
        use $crate::fmt::{self as ufmt, uwriteln};
        let mut stdout = $crate::io::stdout();
        let _ = uwriteln!(&mut stdout, $($arg)*);
    }};
}
