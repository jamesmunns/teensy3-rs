extern crate bindgen;
extern crate gcc;

use std::env;
use std::fs::{self, File};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::PathBuf;

static COMPILER_FLAGS: &[&str] = &[
    "-mthumb",
    "-mcpu=cortex-m4",
    "-DLAYOUT_US_ENGLISH",
    "-DUSB_SERIAL",
];

static CPP_FLAGS: &[&str] = &[
    "-std=gnu++14",
    "-felide-constructors",
    // "-fno-exceptions",
    "-fno-rtti",
    "-fkeep-inline-functions",
];

struct Config {
    mcu: &'static str,
    cpu: &'static str,
    compiler_flags: Vec<&'static str>,
}

fn get_config() -> Config {
    let features = vec![
        cfg!(feature = "teensy_3_0"),
        cfg!(feature = "teensy_3_1"),
        cfg!(feature = "teensy_3_2"),
        cfg!(feature = "teensy_3_5"),
        cfg!(feature = "teensy_3_6"),
    ];

    if features.iter().filter(|&f| *f).count() != 1 {
        panic!("Bad features!");
    }

    if cfg!(feature = "teensy_3_0") {
        Config {
            mcu: "MK20DX128",
            cpu: "48000000",
            compiler_flags: vec![],
        }
    } else if cfg!(feature = "teensy_3_1") || cfg!(feature = "teensy_3_2") {
        Config {
            mcu: "MK20DX256",
            cpu: "48000000",
            compiler_flags: vec![],
        }
    } else if cfg!(feature = "teensy_3_5") {
        Config {
            mcu: "MK64FX512",
            cpu: "120000000",
            compiler_flags: vec!["-mfloat-abi=hard", "-mfpu=fpv4-sp-d16"],
        }
    } else if cfg!(feature = "teensy_3_6") {
        Config {
            mcu: "MK66FX1M0",
            cpu: "180000000",
            compiler_flags: vec!["-mfloat-abi=hard", "-mfpu=fpv4-sp-d16"],
        }
    } else {
        panic!("uh oh");
    }
}

fn src_files(path: &PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let ignore_files = vec![
        Some(OsStr::new("new.cpp")), // so we can avoid -fno-exceptions
    ];

    let (c_ext, cpp_ext) = (Some(OsStr::new("c")), Some(OsStr::new("cpp")));

    path.read_dir()
        .expect("Unable to read teensy3 directory")
        .filter_map(|entry| {
            let entry = entry.expect("Unable to read a file from teensy3 directory");

            let path = entry.path();

            // Ignore directories
            if path.is_dir() {
                return None;
            }

            // Ignore Files
            if ignore_files.contains(&path.file_name()) {
                return None;
            }

            // We only care about .c and .cpp
            let ext = path.extension();
            if ext == c_ext || ext == cpp_ext {
                Some(path.clone())
            } else {
                None
            }
        })
        .partition(|ref path| path.extension() == c_ext)
}

fn first_dir(path: PathBuf) -> PathBuf {
    path.read_dir()
        .expect("Unable to read directory")
        .next()
        .expect("Expected a directory")
        .expect("Expected a directory")
        .path()
}

fn compile(config: &Config) {
    let teensy3 = ["cores", "teensy3"].iter().collect();

    let (c_files, cpp_files) = src_files(&teensy3);

    let mut builder = gcc::Build::new();

    // Shared Builder
    builder
        .archiver("arm-none-eabi-ar")
        .include(&teensy3)
        .opt_level_str("s")
        .pic(false)
        .warnings(false)
        .define(&format!("__{}__", config.mcu), None)
        .define("F_CPU", config.cpu);

    for flag in COMPILER_FLAGS {
        builder.flag(flag);
    }

    for flag in &config.compiler_flags {
        builder.flag(flag);
    }

    // Compile C Files
    builder
        .clone()
        .compiler("arm-none-eabi-gcc")
        .cpp(false)
        .files(c_files)
        .compile("libteensyduino_c");

    // Compile C++ Files
    let mut cpp = builder.clone();

    for flag in CPP_FLAGS {
        cpp.flag(flag);
    }

    cpp.compiler("arm-none-eabi-g++")
        .cpp(true)
        .define("NEW_H", None) // Ignore new.h, to avoid -fno-exceptions
        .files(cpp_files)
        .compile("libteensyduino_cpp");
}

fn generate_bindings(config: &Config) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // FIXME (https://github.com/jamesmunns/teensy3-rs/issues/17) Remove this hack
    let modified_wprogram_h = out_dir.join("WProgram.h");
    let mut wprogram_h = String::new();
    File::open("cores/teensy3/WProgram.h")
        .expect("failed to open header")
        .read_to_string(&mut wprogram_h)
        .expect("failed to read program header");
    File::create(&modified_wprogram_h)
        .expect("failed to create program header")
        .write_all(
            wprogram_h
                .replace("int32_t random(void);", "long random(void);")
                .as_bytes(),
        )
        .expect("failed to write to program header");

    let mut flags: Vec<String> = CPP_FLAGS
        .iter()
        .chain(COMPILER_FLAGS.iter())
        .chain(config.compiler_flags.iter())
        .map(|&flag| String::from(flag))
        .collect();

    flags.push(format!("-D__{}__", config.mcu));
    flags.push(format!("-DF_CPU={}", config.cpu));

    flags.push(String::from("-D__GNUCLIKE_BUILTIN_VARARGS")); // Fix for duplicate __va_list

    let includes: Vec<String> = vec![["cores", "teensy3"].iter().collect()]
        .iter()
        .map(|path| format!("-I{}", path.to_str().unwrap()))
        .collect();

    let bindings = bindgen::Builder::default()
        .header("cores/teensy3/Arduino.h")
        .generate_inline_functions(true)
        .use_core()
        .blacklist_type("__cxxabiv1")
        .blacklist_type("__gnu_cxx")
        .blacklist_type("std")
        .ctypes_prefix("c_types")
        .clang_args(&flags)
        .clang_args(&includes)
        .clang_arg("-xc++")
        .clang_arg(format!("--target={}", env::var("TARGET").unwrap()))
        .clang_arg("-include")
        .clang_arg(modified_wprogram_h.to_str().unwrap())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    let config = get_config();

    compile(&config);

    generate_bindings(&config);

    // Put the linker script somewhere the top crate can find it
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy(
        [
            "cores",
            "teensy3",
            &format!("{}.ld", config.mcu.to_lowercase()),
        ].iter()
            .collect::<PathBuf>(),
        out_dir.join("teensy3-sys.ld"),
    ).expect("Failed to write to linkerfile");
    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=nosys");
    println!("cargo:rustc-link-lib=c");
    println!("cargo:rustc-link-lib=gcc");
}
