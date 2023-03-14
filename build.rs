#[allow(dead_code)]
#[cfg(feature = "backend-freertos")]
fn build_freertos() {
    let mut b = freertos_cargo_build::Builder::new();

    b.freertos("freertos-rust/freertos-rust-examples/FreeRTOS-Kernel");
    b.freertos_config("src/tests/freertos");
    b.freertos_port("ThirdParty/GCC/Posix/");
    b.heap("heap_3.c");
    b.get_cc().file("src/tests/freertos/hooks.c");
    b.compile().unwrap_or_else(|e| panic!("Build error: {e}"));
    //println!("cargo:rustc-link-lib=freertos");
    //println!("cargo:rustc-link-arg-bins=-lfreertos");

    println!("cargo:rerun-if-changed=src/tests/freertos/");
}

fn main() {
    #[cfg(all(feature = "backend-freertos", test))]
    build_freertos();
}
