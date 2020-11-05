//! Convenience library for Rust development on a Teensy 3.x.
//!
//! Contains safe and unsafe utilities, bootstrapped by the C++
//! Teensyduino libraries provided by the manufacturer

#![no_std]
//#![feature(lang_items)]
#![allow(dead_code)]

/// Unsafe C++ Teensyduino bindings from the teensy3-sys crate
pub extern crate teensy3_sys as bindings;

/// "Safe" wrappers
#[macro_use]
pub mod serial;
pub mod spi;
pub mod util;
pub mod pins;

/// Processor panic: enter infinite loop. Blink monotonically and keep writing reason
/// for panic every 10 seconds.
#[panic_handler]
fn teensy_panic(pi: &core::panic::PanicInfo) -> ! {
    println!("{}", pi);
    loop {
        // Keep writing the reason for panic over and over again
        for _ in 0..50 {
            unsafe {
                bindings::digitalWrite(13, bindings::HIGH as u8);
                bindings::delay(100);
                bindings::digitalWrite(13, bindings::LOW as u8);
                bindings::delay(100);
            }
        }
        println!("-------------------");
        println!("-------------------");
        println!("{}", pi);
    };
}

///// Something related to error unwinding
//#[lang = "eh_personality"]
//pub extern fn rust_eh_personality() {}
