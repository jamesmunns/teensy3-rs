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