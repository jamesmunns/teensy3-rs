//! Convenience library for Rust development on a Teensy 3.x.
//!
//! Contains safe and unsafe utilities, bootstrapped by the C++
//! Teensyduino libraries provided by the manufacturer

#![no_std]
#![feature(lang_items)]

/// Unsafe C++ Teensyduino bindings from the teensy3-sys crate
pub extern crate teensy3_sys as bindings;

/// Safe wrapper for the USB Serial Port
pub mod serial;

/// Processor panic
#[lang = "panic_fmt"]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    loop {}
}

/// ?
#[lang = "eh_personality"]
pub extern fn rust_eh_personality() {}
