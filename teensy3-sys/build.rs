extern crate bindgen;
extern crate cc;

use std::env;
use std::fs::{self, File};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;

// Both c and c++
static COMPILER_FLAGS: &[&str] = &[
    "-mthumb",
    "-mcpu=cortex-m4",
    "-ffunction-sections",  // unused function removal, use linker flag "link-arg=-Wl,--gc-sections
    "-fdata-sections",      // unused data removal, use linker flag "link-arg=-Wl,--gc-sections
    "-DLAYOUT_US_ENGLISH",
    // You can enable different teensy features by changing following "-D____" flag.
    // For example, disable keyboard/mouse functionality by replacing it with "-DUSB_SERIAL".
    // Here's full list of possibilities from cores/teensy3/usb_dec.h (not tested!)
    //     USB_SERIAL, USB_DUAL_SERIAL, USB_TRIPLE_SERIAL, USB_KEYBOARDONLY, USB_HID,
    //     USB_SERIAL_HID, USB_TOUCHSCREEN, USB_HID_TOUCHSCREEN, USB_MIDI, USB_MIDI4
    //     USB_MIDI16, USB_MIDI_SERIAL, USB_MIDI4_SERIAL, USB_MIDI16_SERIAL, USB_RAWHID
    //     USB_FLIGHTSIM, USB_FLIGHTSIM_JOYSTICK, USB_MTPDISK, USB_AUDIO,
    //     USB_MIDI_AUDIO_SERIAL, USB_MIDI16_AUDIO_SERIAL, USB_EVERYTHING
    "-DUSB_SERIAL_HID",  // usb serial and hid functionality (i.e. print output & mouse/keyboard)
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
fn get_src_paths() -> [PathBuf;2] {
    // How on earth you make a globally accessible Path in rust? Is it even possible?
    // I'll make a function that returns a constant pathbuf then
    [
        PathBuf::from("cores/teensy3"),
        PathBuf::from("SPI"),
        //PathBuf::from("Wire"),
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

    let target = std::env::var("TARGET").unwrap();
    let fpu; // FPU: hardware based Floating Point Unit
    let compiler_flags;
    if &target == "thumbv7em-none-eabi" {
        fpu = false;
        compiler_flags = vec![];
    } else if &target == "thumbv7em-none-eabihf" {
        if !(cfg!(feature = "teensy_3_5") || cfg!(feature = "teensy_3_6")) {
            panic!("Hardware floating point not supported on this device. Use thumbv7em-none-eabi");
        }
        // Useful and general terminology: Options for -mfloat-abi are:
        //     soft: processor has no Floating Point Unit (FPU)
        //     hard: processor has FPU, floating point operations are hardware instructions.
        //     softfp: processor has FPU, but still uses soft calling convention
        fpu = true;
        compiler_flags = vec![
            "-mfloat-abi=hard",
            "-mfpu=fpv4-sp-d16",
        ]
    } else {
        panic!("Unknown target triple {}, use 'thumbv7em-none-eabi'", target);
    };

    let (mcu, cpu, ) = if cfg!(feature = "teensy_3_0") {
        ("MK20DX128", "48000000")
    } else if cfg!(feature = "teensy_3_1") || cfg!(feature = "teensy_3_2") {
        ("MK20DX256", "48000000")
    } else if cfg!(feature = "teensy_3_5") {
        ("MK64FX512", "120000000")
    } else if cfg!(feature = "teensy_3_6") {
        ("MK66FX1M0", "180000000")
    } else {
        panic!("This panic is never thrown.");
    };

    // Find out newlib path, which is something like "/usr/include/newlib/c++/4.9.3"
    // Figure out version number by taking first item in directory
    let p = PathBuf::from("/usr/include/newlib/c++/");
    if !p.is_dir() {
        panic!("Newlib not found from {:?}. \nIt is either not installed or system is windows.", p)}
    let mut entry_iter = fs::read_dir(p).unwrap();
    let first_entry = entry_iter.next().unwrap().unwrap();
    let newlib_path = first_entry.path();

    // newlib bits path is differs on diffenrent versions of newlib
    let newlib_bits_path = match fpu {
        true  => {
            let v9 = newlib_path.join("arm-none-eabi/thumb/v7e-m+fp/hard/");
            //let v4 = newlib_path.join("arm-none-eabi/armv7e-m/fpu/");
            let v4 = newlib_path.join("arm-none-eabi/thumb/armv7-ar/fpu/vfpv3-d16/be/");
            if v9.is_dir() {  // newlib version 8 path
                v9
            } else if v4.is_dir() {  // newlib version 4 path
                v4
            } else {
                panic!("Newlib library path not found automatically. Please configure it manually \
                in build.rs.")
            }
        },
        false => {
            let v9 = newlib_path.join("arm-none-eabi/thumb/v7e-m/nofp/");
            //let v4 = newlib_path.join("arm-none-eabi/armv7e-m");
            let v4 = newlib_path.join("arm-none-eabi/thumb/");
            if v9.is_dir() {  // newlib version 8 path
                v9
            } else if v4.is_dir() {  // newlib version 4 path
                v4
            } else {
                panic!("Newlib library path not found automatically. Please configure it manually \
                in build.rs.")
            }
        },
    };

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

    for flag in COMPILER_FLAGS.iter().chain(&config.compiler_flags) {
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
        .no_default_flags(true)
        .files(c_files)
        .compile("libteensyduino_c");
    // Compile C++ Files
    cpp.compiler("arm-none-eabi-g++")
        .cpp(true)
        .no_default_flags(true)
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

    let mut flags: Vec<String> = COMPILER_FLAGS
        .iter()
        .chain(C_FLAGS.iter())
        .chain(CPP_FLAGS.iter())
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
        //.blacklist_type("__cxxabiv1")  // something
        //.blacklist_type("__gnu_cxx")  // something
        .blacklist_item("_Tp")  // get bindings to compile
        .blacklist_item("FP_.*")  // get bindings to compile
        .blacklist_item("std")  // Ignore private items of stdlib or something
        .blacklist_item("std_.*")  // drops anything that stars with std_
        .opaque_type("std::.*")  // Do not try to translate std types if any left
        .ctypes_prefix("c_types")
        .clang_args(&flags)
        .clang_args(&includes)
        .clang_arg("-v")
        .clang_arg("-H")
        .clang_arg("-xc++")
        .clang_arg(format!("--target={}", env::var("TARGET").unwrap()))
        .clang_arg("-I/usr/include/newlib/")
        .clang_arg(format!("-I{}", config.newlib_path.to_str().unwrap()))
        .clang_arg(format!("-I{}", config.newlib_bits_path.to_str().unwrap()))
        .clang_arg("-include")
        .clang_arg(modified_wprogram_h.to_str().unwrap())
        .clang_arg("-fretain-comments-from-system-headers")  // It does not still generate comments?
        .generate_comments(true)  // It does not still generate comments?
        //.clang_arg("-L/usr/include/newlib/c++/9.2.1/stdlib.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    // Testing, can be removed
    if false {  // Figure out what's inside docker container
        // execute arbitrary shell commands
        let commands = [
            "find /usr/ -name 'type_traits'",
            "find /usr/ -name 'c++config.h'",
            "find /usr/ -name 'stdlib.h'",
            "find /usr/ -name 'cstdlib'",
        ];
        for (i, &command) in commands.iter().enumerate() {
            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .expect("Failed to execute command.");
            let out_string = String::from_utf8_lossy(output.stdout.as_slice()) ;
            println!("--{}--\n{}\n-----", i, out_string);
        }
        println!("Target: {}", std::env::var("TARGET").unwrap());
        println!("PATH: {}", std::env::var("PATH").unwrap());
        //panic!("Stop");
    }

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

    println!("cargo:rerun-if-changed=build.rs");

}
