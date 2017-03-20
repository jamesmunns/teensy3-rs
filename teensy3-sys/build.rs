extern crate bindgen;

use std::env;
use std::fs::read_dir;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let c_compiler = "arm-none-eabi-gcc";
    let cpp_compiler = "arm-none-eabi-g++";
    let archive = "arm-none-eabi-ar";

    // Both C and C++
    let compiler_args = [
        "-mthumb",
        "-mcpu=cortex-m4",
        "-D__MK20DX256__",
        "-DF_CPU=48000000",

        "-DUSB_SERIAL",
        "-DLAYOUT_US_ENGLISH",
        "-DTEENSYDUINO=121",
        "-g",
        "-Os",
    ];

    // C only
    let c_args = [
    ];

    // C++ only
    let cpp_args = [
        "-std=gnu++0x",
        "-felide-constructors",
        "-fno-exceptions",
        "-fno-rtti",
        "-fkeep-inline-functions",
    ];

    let srcdir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("teensy3-core");
    let outdir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut objs = Vec::new();
    for entry in read_dir(&srcdir).unwrap() {
        let path = entry.unwrap().path();
        let (compiler, extra_args) = match path.extension() {
            Some(e) if e == "c" => (c_compiler, &c_args[..]),
            Some(e) if e == "cpp" => (cpp_compiler, &cpp_args[..]),
            _ => continue,
        };
        let obj = PathBuf::from(path.with_extension("o").file_name().unwrap());
        check(
            Command::new(compiler)
            .args(&compiler_args)
            .args(extra_args)
            .arg("-I").arg(&srcdir)
            .arg("-c").arg(&path)
            .arg("-o").arg(&obj)
            .current_dir(&outdir)
        );
        objs.push(obj);
    }
    check(
        Command::new(archive)
        .arg("crus")
        .arg(outdir.join("libteensyduino.a"))
        .args(objs)
        .current_dir(&outdir)
    );
    println!("cargo:rustc-link-search=native={}", outdir.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=teensyduino");

    bindgen::Builder::default()
        .no_unstable_rust()
        .use_core()
        .generate_inline_functions(true)
        .header("bindings.h")
        .ctypes_prefix("c_types")
        .clang_args(&compiler_args)
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=gnu++0x")
        .clang_arg("-target")
        .clang_arg(env::var("TARGET").unwrap())
        .generate()
        .expect("error when generating bindings")
        .write_to_file(outdir.join("bindings.rs"))
        .expect("error when writing bindings");

}

fn check(command: &mut Command) {
    match command.status() {
        Ok(ref status) if status.success() => {}
        Ok(ref status) => panic!("command `{:?}` exited with {}.", command, status),
        Err(ref error) => panic!("could not start command `{:?}`: {}.", command, error),
    }
}
