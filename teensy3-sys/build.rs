extern crate bindgen;
extern crate cc;

use std::env;
use std::fs::{self, File};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::PathBuf;
//use std::process::Command;

// Both c and c++
static COMPILER_FLAGS: &[&str] = &[
    "-mthumb",
    "-mcpu=cortex-m4",
    "-DLAYOUT_US_ENGLISH",
    "-DUSB_SERIAL",
    "-DTEENSYDUINO",
];

static C_FLAGS: &[&str] = &[
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
    newlib_path: PathBuf,
    newlib_bits_path: PathBuf,
}
/// Get paths to compile
fn get_src_paths() -> [PathBuf;1] {
    // How on earth you make a globally accessible path in rust?!? Is it even possible?
    [
        PathBuf::from("cores/teensy3"),
    ]
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
        panic!("Invalid features! Define one board for teensy3. E.g. add feature 'teensy_3_6'");
    }

    let (mcu, cpu, compiler_flags) = if cfg!(feature = "teensy_3_0") {
        ("MK20DX128", "48000000", vec![])
    } else if cfg!(feature = "teensy_3_1") || cfg!(feature = "teensy_3_2") {
        ("MK20DX256", "48000000", vec![])
    } else if cfg!(feature = "teensy_3_5") {
        // If you want to fight errors, you can try enabling hardware floats
        // vec!["-mfloat-abi=hard", "-mfpu=fpv4-sp-d16"]
        ("MK64FX512", "120000000", vec![])
    } else if cfg!(feature = "teensy_3_6") {
        // If you want to fight errors, you can try enabling hardware floats
        // vec!["-mfloat-abi=hard", "-mfpu=fpv4-sp-d16"]
        ("MK66FX1M0", "180000000", vec![])
    } else {
        panic!("Uh oh");
    };

    // Find out newlib path, which is something like "/usr/include/newlib/c++/4.9.3"
    // Figure out version number by taking first item in directory
    let mut entry_iter = fs::read_dir("/usr/include/newlib/c++/").unwrap();
    let first_entry = entry_iter.next().unwrap().unwrap();
    let newlib_path = first_entry.path();

    let newlib_bits_path = newlib_path.join("arm-none-eabi/armv7e-m");
    // If you want to fight errors, you can try enabling hardware floats
    //let fpu = compiler_flags.contains(&"-mfloat-abi=hard");
    // let newlib_bits_path = match fpu {
    //     true  => newlib_path.join("arm-none-eabi/armv7e-m/fpu"),
    //     false => newlib_path.join("arm-none-eabi/armv7e-m/softfp"),
    // };

    return Config{mcu, cpu, compiler_flags, newlib_path, newlib_bits_path}
}

fn src_files(path: &PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let ignore_files = vec![
        Some(OsStr::new("new.cpp")), // so we can avoid -fno-exceptions
    ];

    let (c_ext, cpp_ext) = (Some(OsStr::new("c")), Some(OsStr::new("cpp")));

    if path.is_file() {  // Single files can be compiled too, though idk why someone wants that
        let ext = path.extension();
        match ext {
            e if e==c_ext => {return (vec![path.clone()], Vec::new());},
            e if e==cpp_ext => {return (Vec::new(), vec![path.clone()]);},
            _ => panic!("Invalid file extension on source file."),
        }
    }
    else if path.is_dir() {
        path.read_dir()
            .expect(&format!("Unable to read directory: {}", path.to_str().unwrap()))
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
    else {panic!("Uh oh")}
}

fn compile(config: &Config) {
    let mut c_files = Vec::new();
    let mut cpp_files = Vec::new();
    let mut builder = cc::Build::new();  // Shared Builder both for c & c++
    for path in get_src_paths().iter() {
        let (c, cpp) = src_files(path);
        c_files.extend(c);
        cpp_files.extend(cpp);
        builder.include(&path);
    }

    builder.archiver("arm-none-eabi-ar")
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


    let mut c = builder.clone();
    let mut cpp = builder.clone();

    for flag in C_FLAGS {
        c.flag(flag);
    }
    for flag in CPP_FLAGS {
        cpp.flag(flag);
    }

    // Compile C Files
    c.compiler("arm-none-eabi-gcc")
        .cpp(false)
        //.no_default_flags(true)  // Default flags seem to do something that fixes linker errrors
        .files(c_files)
        .compile("libteensyduino_c");
    // Compile C++ Files
    cpp.compiler("arm-none-eabi-g++")
        .cpp(true)
        //.no_default_flags(true)
        .cpp_link_stdlib(None)
        .define("NEW_H", None) // Ignore new.h, to avoid -fno-exceptions
        .files(cpp_files)
        .compile("libteensyduino_cpp");
}

fn generate_bindings(config: &Config) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Hack to overcome non-matching function overload in teensy libraries
    // (https://github.com/jamesmunns/teensy3-rs/issues/17)
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

    let includes: Vec<String> = get_src_paths().iter()
        .map(|path| format!("-I{}", path.to_str().unwrap()))
        .collect();

    bindgen::Builder::default()
        .header("bindings.h")
        .generate_inline_functions(true)
        .use_core()
        .blacklist_type("__cxxabiv1")  // something
        .blacklist_type("__gnu_cxx")  // something
        .blacklist_item("_Tp")  // get bindings to compile
        .blacklist_item("FP_.*")  // get bindings to compile
        .blacklist_item("std")  // Ignore private items of stdlib or something
        .blacklist_item("std_.*")  // drops anything that stars with std_
        .opaque_type("std::.*")  // Do not try to translate std types
        .ctypes_prefix("c_types")
        .clang_args(&flags)
        .clang_args(&includes)
        .clang_arg("-xc++")
        .clang_arg(format!("--target={}", env::var("TARGET").unwrap()))
        // I have no knowledge what is proper way of telling bindgen about these paths
        .clang_arg(format!("-I{}", config.newlib_path.to_str().unwrap()))
        .clang_arg(format!("-I{}", config.newlib_bits_path.to_str().unwrap()))
        .clang_arg("-include")
        .clang_arg(modified_wprogram_h.to_str().unwrap())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    let config = get_config();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    compile(&config);

    generate_bindings(&config);

    // Take linker script from teensy libraries and put it somewhere the top crate can find it
    let mcu = config.mcu.to_lowercase();
    fs::copy(
        PathBuf::from(format!("cores/teensy3/{}.ld", mcu)),
        out_dir.join("linker_script.ld"),
    ).expect("Failed to write to linkerfile");
    println!("cargo:rustc-link-search={}", out_dir.display());  // grapped linker file

    File::create(&out_dir.join("mcu_info.txt"))
        .expect("failed to create cpu info file")
        .write_all(mcu.as_bytes())
        .expect("failed to write cpu info file");
}
