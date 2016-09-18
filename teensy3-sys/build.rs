use std::env;
use std::process::Command;

fn main() {
    let outdir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", outdir);

    // TODO(@jamesmunns): How do I pipe stdout here so we dont just silently wait?
    assert!(Command::new("make")
        .args(&["--no-print-directory", "-C", "teensy3-core", "NO_ARDUINO=1"])
        .status()
        .expect("failed to build Teensyduino libs")
        .success());

    println!("cargo:rustc-link-search={}/teensy3-core", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:libdir=teensy");
    println!("cargo:rustc-link-lib=static=teensyduino");
}
