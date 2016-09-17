#![feature(lang_items)]
#![no_std]

#[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, improper_ctypes)]
#[path = "bindings.rs"]
mod teensy3;

#[macro_use]
mod serial;
use serial::Serial;

#[no_mangle]
pub unsafe extern fn main() {
    // Blink Loop
    teensy3::pinMode(13, teensy3::OUTPUT as u8);
    loop {
        teensy3::digitalWrite(13, teensy3::LOW as u8);
        teensy3::delay(1000);
        teensy3::digitalWrite(13, teensy3::HIGH as u8);
        teensy3::delay(2000);
    }
}

fn read_int(delimiter: u8) -> u32 {
    Serial.try_read_int_until(delimiter).unwrap()
}

mod std {
    pub use core::*;
    pub mod os {
        #[allow(non_camel_case_types)]
        pub mod raw {
            pub enum c_void {}
            pub type c_uchar = u8;
            pub type c_short = i16;
            pub type c_ushort = u16;
            pub type c_int = i32;
            pub type c_uint = u32;
            pub type c_long = i32;
            pub type c_ulong = u32;
            pub type c_longlong = i64;
            pub type c_ulonglong = u64;
        }
    }
}

#[lang = "panic_fmt"]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("Panic at {}:{}, {}", file, line, msg);
    loop {}
}

#[lang = "eh_personality"]
pub extern fn rust_eh_personality() {}
