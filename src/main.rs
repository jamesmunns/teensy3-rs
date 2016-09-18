#![feature(lang_items)]
#![no_std]
#![no_main]

#[macro_use]
extern crate teensy3;

use teensy3::bindings;

#[no_mangle]
pub unsafe extern fn main() {
    // Blink Loop
    bindings::pinMode(13, bindings::OUTPUT as u8);
    loop {
        bindings::digitalWrite(13, bindings::LOW as u8);
        bindings::delay(500);
        bindings::digitalWrite(13, bindings::HIGH as u8);
        bindings::delay(200);
    }
}

#[lang = "panic_fmt"]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("Panic at {}:{}, {}", file, line, msg);
    loop {}
}

#[lang = "eh_personality"]
pub extern fn rust_eh_personality() {}
