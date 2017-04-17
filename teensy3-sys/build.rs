extern crate bindgen;

use std::env;
use std::fs::{File, read_dir};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let source_dirs = [
        "cores/teensy3",
        "SPI",
        "Wire",
    ];

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

    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let includes = source_dirs.iter().flat_map(|dir| vec!["-I", dir]).collect::<Vec<_>>();
    let mut objs = Vec::new();

    for source_dir in &source_dirs {
        for entry in read_dir(crate_dir.join(source_dir)).unwrap() {
            let path = entry.unwrap().path();
            let (compiler, extra_args) = match path.extension() {
                Some(e) if e == "c" => (c_compiler, &c_args[..]),
                Some(e) if e == "cpp" => (cpp_compiler, &cpp_args[..]),
                _ => continue,
            };
            let obj = path.with_extension("o").file_name().unwrap().to_owned();
            check(
                Command::new(compiler)
                .args(&compiler_args)
                .args(extra_args)
                .args(&includes)
                .arg("-c").arg(Path::new(source_dir).join(path.file_name().unwrap()))
                .arg("-o").arg(out_dir.join(&obj))
            );
            objs.push(obj);
        }
    }
    check(
        Command::new(archive)
        .arg("crus")
        .arg(out_dir.join("libteensyduino.a"))
        .args(objs)
        .current_dir(&out_dir)
    );
    println!("cargo:rustc-link-search=native={}", out_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=teensyduino");

    // FIXME (https://github.com/jamesmunns/teensy3-rs/issues/17) Remove this hack
    let modified_wprogram_h = out_dir.join("WProgram.h");
    let mut wprogram_h = String::new();
    File::open("cores/teensy3/WProgram.h").unwrap().read_to_string(&mut wprogram_h).unwrap();
    File::create(&modified_wprogram_h).unwrap().write_all(wprogram_h.replace(
        "int32_t random(void);",
        "long random(void);",
    ).as_bytes()).unwrap();

    bindgen::Builder::default()
        .no_unstable_rust()
        .use_core()
        .generate_inline_functions(true)
        .header("bindings.h")
        .ctypes_prefix("c_types")
        .clang_args(&compiler_args)
        .clang_args(&includes)
        .clang_arg("-include")
        .clang_arg(modified_wprogram_h.to_str().unwrap())
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=gnu++0x")
        .clang_arg("-target")
        .clang_arg(env::var("TARGET").unwrap())
        .generate()
        .expect("error when generating bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("error when writing bindings");

}

fn check(command: &mut Command) {
    match command.status() {
        Ok(ref status) if status.success() => {}
        Ok(ref status) => panic!("command `{:?}` exited with {}.", command, status),
        Err(ref error) => panic!("could not start command `{:?}`: {}.", command, error),
    }
}
