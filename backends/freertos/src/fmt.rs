use core::convert::Infallible;

use crate::io::Stdout;

pub use ufmt::*;

struct LfToCrLf<I: Iterator<Item = u8>> {
    iter: I,
    lf: bool,
}

impl<I: Iterator<Item = u8>> LfToCrLf<I> {
    fn new(iter: I) -> Self {
        Self { iter, lf: false }
    }
}

impl<I: Iterator<Item = u8>> Iterator for LfToCrLf<I> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.lf {
            self.lf = false;
            return Some(b'\n');
        }
        let b = self.iter.next()?;
        if b == b'\n' {
            self.lf = true;
            return Some(b'\r');
        }
        Some(b)
    }
}

impl uWrite for Stdout {
    type Error = Infallible;
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let src = LfToCrLf::new(s.as_bytes().iter().cloned());
        for b in src {
            self.write_byte(b);
        }
        self.flush();
        Ok(())
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
