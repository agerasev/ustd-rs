use crate::println;
use core::panic::PanicInfo;

extern "C" {
    static __ustd_panicked: u8;
    fn __ustd_panic() -> !;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = println!("PANIC: {}", info);
    unsafe { __ustd_panic() }
}
