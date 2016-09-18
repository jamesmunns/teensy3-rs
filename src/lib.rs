#![no_std]
#![feature(lang_items)]

pub extern crate teensy3_sys as bindings;
pub mod serial;

#[lang = "panic_fmt"]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    loop {}
}

#[lang = "eh_personality"]
pub extern fn rust_eh_personality() {}
