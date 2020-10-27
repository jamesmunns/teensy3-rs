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
#[panic_handler]
fn teensy_panic(pi: &core::panic::PanicInfo) -> ! {
    if let Some(s) = pi.payload().downcast_ref::<&str>() {
        println!("Panic occured {:?}", s);
    } else {
        println!("Panic occured");
    }
    for _ in 0..3 {
    }
    loop {
        unsafe {
            bindings::digitalWrite(13, bindings::HIGH as u8);
            bindings::delay(100);
            bindings::digitalWrite(13, bindings::LOW as u8);
            bindings::delay(100);
        }
    };
}
// #[lang = "panic_fmt"]
// pub extern fn rust_begin_panic(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
//     println!("{:?}:{:?} - {:?}\n\r", file, line, msg);
//     loop {}
// }

/// ?
#[lang = "eh_personality"]
pub extern fn rust_eh_personality() {}
