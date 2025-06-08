use core::{ffi::c_char, slice::from_raw_parts_mut};

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

impl Stdout {
    unsafe fn push_byte_unchecked(&mut self, b: u8) {
        *self.global.buffer().get_unchecked_mut(self.pos) = b;
        self.pos += 1;
    }
    pub(crate) fn write_byte(&mut self, b: u8) {
        unsafe { self.push_byte_unchecked(b) };
        if self.pos >= GlobalStdout::buffer_len() {
            self.global.write_buffer();
            self.pos = 0;
        }
    }
    pub(crate) fn flush(&mut self) {
        if self.pos > 0 {
            unsafe { self.push_byte_unchecked(0) };
            self.global.write_buffer();
            self.pos = 0;
        }
    }
}

pub fn stdout() -> Stdout {
    Stdout {
        global: GlobalStdout,
        pos: 0,
    }
}
