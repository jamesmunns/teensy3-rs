extern crate bindgen;

use std::env;
use std::fs::{File, read_dir};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
struct CompilerOpts {
    compiler_args: Vec<&'static str>,
    c_args: Vec<&'static str>,
    cpp_args: Vec<&'static str>,
}

fn flag_sanity_check() -> Result<CompilerOpts, ()> {
    let uc_3_0 = cfg!(feature = "teensy_3_0");
    let uc_3_1 = cfg!(feature = "teensy_3_1");
    let uc_3_2 = cfg!(feature = "teensy_3_2");
    let uc_3_5 = cfg!(feature = "teensy_3_5");
    let uc_3_6 = cfg!(feature = "teensy_3_6");

    // TODO: de-dupe
    match (uc_3_0, uc_3_1, uc_3_2, uc_3_5, uc_3_6) {
        (false, true, false, false, false) => { // Teensy 3.1
            Ok(CompilerOpts {
                compiler_args: vec![
                    "-mthumb",
                    "-mcpu=cortex-m4",
                    "-D__MK20DX256__",
                    "-DF_CPU=48000000",

                    "-DUSB_SERIAL",
                    "-DLAYOUT_US_ENGLISH",
                    "-DTEENSYDUINO=121",
                    "-g",
                    "-Os",
                ],
                c_args: vec![],
                cpp_args: vec![
                    "-std=gnu++0x",
                    "-felide-constructors",
                    "-fno-exceptions",
                    "-fno-rtti",
                    "-fkeep-inline-functions",
                ]
            })
        }
        (false, false, true, false, false) => { // Teensy 3.2
            Ok(CompilerOpts {
                compiler_args: vec![
                    "-mthumb",
                    "-mcpu=cortex-m4",
                    "-D__MK20DX256__",
                    "-DF_CPU=48000000",

                    "-DUSB_SERIAL",
                    "-DLAYOUT_US_ENGLISH",
                    "-DTEENSYDUINO=121",
                    "-g",
                    "-Os",
                ],
                c_args: vec![],
                cpp_args: vec![
                    "-std=gnu++0x",
                    "-felide-constructors",
                    "-fno-exceptions",
                    "-fno-rtti",
                    "-fkeep-inline-functions",
                ]
            })
        }
        (false, false, false, true, false) => { // Teensy 3.5
            Ok(CompilerOpts {
                compiler_args: vec![
                    "-mthumb",
                    "-mcpu=cortex-m4",
                    "-D__MK64FX512__",
                    "-DF_CPU=120000000",

                    // AJM - hm. need to figure out hard float
                    //   probably relevant to 3.1+ as well.
                    //
                    // "-mfloat-abi=hard",
                    // "-mfpu=fpv4-sp-d16",
                    // "-fsingle-precision-constant",
                    // AJM

                    "-DUSB_SERIAL",
                    "-DLAYOUT_US_ENGLISH",
                    "-DTEENSYDUINO=121",
                    "-g",
                    "-Os",
                ],
                c_args: vec![],
                cpp_args: vec![
                    "-std=gnu++0x",
                    "-felide-constructors",
                    "-fno-exceptions",
                    "-fno-rtti",
                    "-fkeep-inline-functions",
                ]
            })
        }
        _ => Err(()),
    }
}

fn main() {

    let flags = flag_sanity_check().expect("Bad Feature Flags!");

    let source_dirs = [
        "cores/teensy3",
        "SPI",
        "Wire",
    ];

    let c_compiler = "arm-none-eabi-gcc";
    let cpp_compiler = "arm-none-eabi-g++";
    let archive = "arm-none-eabi-ar";

    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let includes = source_dirs.iter().flat_map(|dir| vec!["-I", dir]).collect::<Vec<_>>();
    let mut objs = Vec::new();

    for source_dir in &source_dirs {
        for entry in read_dir(crate_dir.join(source_dir)).unwrap() {
            let path = entry.unwrap().path();
            let (compiler, extra_args) = match path.extension() {
                Some(e) if e == "c" => (c_compiler, &flags.c_args[..]),
                Some(e) if e == "cpp" => (cpp_compiler, &flags.cpp_args[..]),
                _ => continue,
            };
            let obj = path.with_extension("o").file_name().unwrap().to_owned();
            check(
                Command::new(compiler)
                .args(&flags.compiler_args)
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
        .clang_args(&flags.compiler_args)
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
