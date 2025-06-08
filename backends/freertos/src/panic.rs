use crate::println;
use core::panic::PanicInfo;

extern "C" {
    static __ustd_panicked: u8;
    fn __ustd_panic() -> !;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let (file, line, column) = info
        .location()
        .map(|loc| (loc.file(), loc.line(), loc.column()))
        .unwrap_or(("", 0, 0));
    let _ = println!(
        "PANIC: {} at {}:{}:{}",
        info.message().as_str().unwrap_or(""),
        file,
        line,
        column,
    );
    unsafe { __ustd_panic() }
}
