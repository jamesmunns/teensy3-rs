#![feature(lang_items)]
#![no_std]
#![no_main]

extern crate teensy3_sys as teensy3;

#[macro_use]
mod serial;
use serial::Serial;

#[no_mangle]
pub unsafe extern fn main() {
    // Blink Loop
    teensy3::pinMode(13, teensy3::OUTPUT as u8);
    loop {
        teensy3::digitalWrite(13, teensy3::LOW as u8);
        teensy3::delay(500);
        teensy3::digitalWrite(13, teensy3::HIGH as u8);
        teensy3::delay(200);
    }
}

fn read_int(delimiter: u8) -> u32 {
    Serial.try_read_int_until(delimiter).unwrap()
}

#[lang = "panic_fmt"]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("Panic at {}:{}, {}", file, line, msg);
    loop {}
}

#[lang = "eh_personality"]
pub extern fn rust_eh_personality() {}
