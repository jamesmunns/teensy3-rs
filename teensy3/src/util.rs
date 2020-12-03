//! Basic Arduino utilities for the Teensy

#![allow(clippy::new_without_default)]

use bindings;

/// Delay at least `ms` milliseconds
pub fn delay(ms: u32) {
    unsafe {
        bindings::delay(ms);
    }
}

/// Measure time differences in millisecond.
/// # Examples
/// ```
/// use teensy3::util::{delay, MillisTimer};
/// let now = MillisTimer::new();
/// delay(200);
/// println!("About {:?} ~ 200 ms elapsed", now.elapsed());
/// delay(300);
/// println!("About {:?} ~ 500 ms elapsed", now.elapsed());
/// ```
pub struct MillisTimer {
    pub init_time: u32
}

impl MillisTimer {
    /// Create timer with internal milliseconds set to value, where time 0 corresponds to
    /// first call of `MillisTimer::new()` in program's lifetime.
    pub fn new() -> Self {
        let now = unsafe{bindings::elapsedMillis::new()};
        MillisTimer {init_time: now.ms}
    }

    /// Elapsed milliseconds from the creation of this object. Maximum measurable time difference
    /// is about 49 days due to the 32 bit integer limits. Then it loops over.
    pub fn elapsed(&self) -> u32 {
        let now = unsafe{bindings::elapsedMillis::new()};
        return if now.ms > self.init_time {
            now.ms - self.init_time
        } else {
            // Timer overflowed
            (u32::MAX - self.init_time) + now.ms + 1
        }
    }
}


/// Measure time differences in microseconds.
/// # Examples
/// ```
/// use teensy3::util::{delay, MicrosTimer};
/// let now = MicrosTimer::new();
/// delay(2);
/// println!("About {:?} ~ 2000 us elapsed", now.elapsed());
/// delay(3);
/// println!("About {:?} ~ 5000 us elapsed", now.elapsed());
/// ```
pub struct MicrosTimer {
    pub init_time: u32
}

impl MicrosTimer {
    /// Create timer with internal microseconds set to value, where time 0 corresponds to
    /// first call of `MicrosTimer::new()` in program's lifetime.
    pub fn new() -> Self {
        let now = unsafe{bindings::elapsedMicros::new()};
        MicrosTimer {init_time: now.us}
    }

    /// Elapsed microseconds from the creation of this object. Maximum measurable time difference
    /// is about 71 minutes due to the 32 bit integer limits. Then it loops over.
    pub fn elapsed(&self) -> u32 {
        let now = unsafe{bindings::elapsedMicros::new()};
        return if now.us > self.init_time {
            now.us - self.init_time
        } else {
            // Timer overflowed
            (u32::MAX - self.init_time) + now.us + 1
        }
    }
}
