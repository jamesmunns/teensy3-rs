//! Convenience library for Rust development on a Teensy 3.x.
//!
//! Contains safe and unsafe utilities, bootstrapped by the C++
//! Teensyduino libraries provided by the manufacturer

#![no_std]
#![feature(lang_items)]
#![allow(dead_code)]

/// Unsafe C++ Teensyduino bindings from the teensy3-sys crate
pub extern crate teensy3_sys as bindings;

/// "Safe" wrappers
#[macro_use]
pub mod serial;
pub mod spi;
pub mod util;

/// Processor panic
#[lang = "panic_fmt"]
pub extern "C" fn rust_begin_panic(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("{:?}:{:?} - {:?}\n\r", file, line, msg);
    loop {}
}

/// ?
#[lang = "eh_personality"]
pub extern "C" fn rust_eh_personality() {}
