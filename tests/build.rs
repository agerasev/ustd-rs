#[allow(dead_code)]
#[cfg(feature = "freertos")]
fn build_freertos() {
    let mut b = freertos_cargo_build::Builder::new();

    b.freertos("../freertos-rust/freertos-rust-examples/FreeRTOS-Kernel");
    b.freertos_config("src/freertos");
    b.freertos_port("ThirdParty/GCC/Posix/");
    //b.freertos_port_base("freertos-addons/Linux/portable");
    b.heap("heap_3.c");
    b.get_cc()
        .file("src/freertos/ustd.c")
        .file("src/freertos/hooks.c");
    b.compile().unwrap_or_else(|e| panic!("Build error: {e}"));

    //println!("cargo:rustc-link-lib=freertos");
    println!("cargo:rustc-link-arg-bins=-lfreertos");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/freertos/");
}

fn main() {
    #[cfg(feature = "freertos")]
    build_freertos();
}
