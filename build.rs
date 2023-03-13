fn main() {
    let mut b = freertos_cargo_build::Builder::new();

    b.freertos("freertos-rust/freertos-rust-examples/FreeRTOS-Kernel");
    b.freertos_config("freertos/");
    b.freertos_port("ThirdParty/GCC/Posix/");
    b.heap("heap_3.c");
    b.get_cc().file("freertos/hooks.c");
    b.compile().unwrap_or_else(|e| panic!("Build error: {e}"));
    //println!("cargo:rustc-link-lib=freertos");
    //println!("cargo:rustc-link-arg-bins=-lfreertos");

    println!("cargo:rerun-if-changed=freertos/");
}
