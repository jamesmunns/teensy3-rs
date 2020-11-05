//! Basic Arduino utilities for the Teensy

use bindings;

/// Delay at least `ms` milliseconds
pub fn delay(ms: u32) {
    unsafe {
        bindings::delay(ms);
    }
}

