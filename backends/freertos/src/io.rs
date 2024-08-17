use core::{
    ffi::c_char,
    fmt::{Error, Write},
    slice::from_raw_parts_mut,
};

extern "C" {
    static __ustd_io_buffer_size: usize;
    static __ustd_io_buffer: [c_char; 0];

    fn __ustd_print_buffer();
}

struct GlobalStdout;

impl GlobalStdout {
    fn buffer_len() -> usize {
        unsafe { __ustd_io_buffer_size }
    }
    fn buffer(&mut self) -> &mut [u8] {
        unsafe { from_raw_parts_mut(__ustd_io_buffer.as_ptr() as *mut u8, Self::buffer_len()) }
    }
    fn write_buffer(&mut self) {
        unsafe { __ustd_print_buffer() }
    }
}

pub struct Stdout {
    global: GlobalStdout,
    pos: usize,
}

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

impl Stdout {
    unsafe fn push_byte_unchecked(&mut self, b: u8) {
        *self.global.buffer().get_unchecked_mut(self.pos) = b;
        self.pos += 1;
    }
    fn write_byte(&mut self, b: u8) {
        unsafe { self.push_byte_unchecked(b) };
        if self.pos >= GlobalStdout::buffer_len() {
            self.global.write_buffer();
            self.pos = 0;
        }
    }
    fn flush(&mut self) {
        if self.pos > 0 {
            unsafe { self.push_byte_unchecked(0) };
            self.global.write_buffer();
            self.pos = 0;
        }
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        let src = LfToCrLf::new(s.as_bytes().iter().cloned());
        for b in src {
            self.write_byte(b);
        }
        self.flush();
        Ok(())
    }
}

pub fn stdout() -> Stdout {
    Stdout {
        global: GlobalStdout,
        pos: 0,
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::{write, fmt::Write};
        let _ = write!($crate::io::stdout(), $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::print!("\n");
    }};
    ($($arg:tt)*) => {{
        use core::{write, fmt::Write};
        let mut stdout = $crate::io::stdout();
        let _ = write!(stdout, $($arg)*);
        let _ = stdout.write_str("\n");
    }};
}
