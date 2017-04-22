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

static COMMON_COMPILER_ARGS: &'static [&'static str] = &[
    "-mthumb",
    "-DUSB_SERIAL",
    "-DLAYOUT_US_ENGLISH",
    "-DTEENSYDUINO=121",
    "-g",  // TODO:
    "-Os", // TODO: Debug/Release split
];

static COMMON_C_ARGS: &'static [&'static str] = &[];

static COMMON_CPP_ARGS: &'static [&'static str] = &[
    "-std=gnu++0x", // TODO: Guarantee this matches the bindgen invokation
    "-felide-constructors",
    "-fno-exceptions",
    "-fno-rtti",
    "-fkeep-inline-functions",
];

fn flag_sanity_check() -> Result<CompilerOpts, ()> {
    let uc_3_0 = cfg!(feature = "teensy_3_0");
    let uc_3_1 = cfg!(feature = "teensy_3_1");
    let uc_3_2 = cfg!(feature = "teensy_3_2");
    let uc_3_5 = cfg!(feature = "teensy_3_5");
    let uc_3_6 = cfg!(feature = "teensy_3_6");

    let mut base_args = CompilerOpts {
        compiler_args: COMMON_COMPILER_ARGS.to_vec(),
        c_args: COMMON_C_ARGS.to_vec(),
        cpp_args: COMMON_CPP_ARGS.to_vec(),
    };

    match (uc_3_0, uc_3_1, uc_3_2, uc_3_5, uc_3_6) {
        // Teensy 3.0
        (true, false, false, false, false) => {
            generate_linkerfile(include_bytes!("cores/teensy3/mk20dx128.ld"));

            base_args.compiler_args.append(&mut vec![
                "-mcpu=cortex-m4",
                "-D__MK20DX128__",
                "-DF_CPU=48000000",
            ]);
            Ok(base_args)
        }
        // Teensy 3.1
        (false, true, false, false, false) => {
            generate_linkerfile(include_bytes!("cores/teensy3/mk20dx256.ld"));

            base_args.compiler_args.append(&mut vec![
                "-mcpu=cortex-m4",
                "-D__MK20DX256__",
                "-DF_CPU=48000000",
            ]);
            Ok(base_args)
        }
        // Teensy 3.2
        (false, false, true, false, false) => {
            generate_linkerfile(include_bytes!("cores/teensy3/mk20dx256.ld"));

            base_args.compiler_args.append(&mut vec![
                "-mcpu=cortex-m4",
                "-D__MK20DX256__",
                "-DF_CPU=48000000",
            ]);
            Ok(base_args)
        }
        // Teensy 3.5
        (false, false, false, true, false) => { // Teensy 3.5
            generate_linkerfile(include_bytes!("cores/teensy3/mk64fx512.ld"));

            base_args.compiler_args.append(&mut vec![
                // TODO: -m4 -> -m4f
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
            ]);
            Ok(base_args)
        }

        // Either none or multiple flags were selected
        // TODO: more descriptive failures
        _ => Err(()),
    }
}

fn generate_linkerfile(linker_bytes: &[u8]) {
        // Put the linker script somewhere the top crate can find it
        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
        File::create(out.join("teensy3-sys.ld"))
            .unwrap()
            .write_all(linker_bytes)
            .unwrap();
        println!("cargo:rustc-link-search={}", out.display());
}

fn main() {

    let flags = flag_sanity_check().expect("Bad Feature Flags!");

    // TODO: Assert `teensy3-sys.ld` exists

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

    // TODO: Consider rolling all of the C based deps into one static lib?
    // http://stackoverflow.com/questions/3821916/how-to-merge-two-ar-static-libraries-into-one
    //   "-C", "link-arg=-lm",
    //   "-C", "link-arg=-lnosys",
    //   "-C", "link-arg=-lc",
    //   "-C", "link-arg=-lgcc",

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
