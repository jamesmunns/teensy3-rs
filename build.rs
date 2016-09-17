use std::env;
use std::process::Command;

fn main() {
    let outdir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", outdir);

    // TODO(@jamesmunns): How do I pipe stdout here so we dont just silently wait?
    let mut cmd = Command::new("make")
        .args(&["--no-print-directory", "-C", "teensy3", "NO_ARDUINO=1"])
        .output()
        .expect("failed to build Teensyduino libs");

    let mut cmd2 = Command::new("arm-none-eabi-objcopy")
        .args(&["-O",
                "ihex",
                "-R",
                ".eeprom",
                "teensy3/main.elf",
                "teensy3.hex"])
        .output()
        .expect("Linker Error?");

// arm-none-eabi-objcopy

    println!("It worked?");
}
