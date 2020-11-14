//! GPIO pin utilities. API is designed to be safe, and it panics if there are multiple references
//! to same pin.
//!
//! If API feels too restricting for some purpose, then sorry, author is not sure
//! what operations are undefined behaviour in original API. For example writing to input pin

use bindings;
use util;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PinMode {
    Input,
    Output,
    InputPullup,
    InputPulldown,
    OutputOpenDrain,
}

/// Object corresponding to physical pin. There is only one instance per pin.
/// To prevent duplication, fields are private, and Copy and Clone are not derived.
pub struct Pin {
    num: u8,
    mode: PinMode,
}

static mut PINROW_AVAILABLE: bool = true;

const NUM_PINS: usize =
    if core::cfg!(any(feature = "teensy_3_0", feature = "teensy_3_1", feature = "teensy_3_2")) {
    34
} else if core::cfg!{any(feature = "teensy_3_5", feature = "teensy_3_6")}{
    58
} else {
    0  // This is never reached, because build script panics if some of above feature is unspecified
};

/// PinRow keeps book what GPIO pins are used and what are free. There is only one
/// pin-object per physical pin, so pin can be "taken out" from PinRow, and then "returned"
/// if it is not used anymore.
///
/// # Example
/// ```
/// fn setup() -> PinRow {
///     // It's unsafe because caller verifies that it's called only once
///     unsafe{PinRow::new_once()}
/// }
/// fn main() {
///     let mut pinrow = setup();
///     let pin = pinrow.get_pin(12, PinMode::input);
///     // Do stuff with pin
///     loop{}
/// }
/// ```
pub struct PinRow([bool; NUM_PINS as usize]);

impl PinRow {
    /// Returns singleton struct that can be used to control GPIO pins.
    /// The calling of this function should happen in program's initialization function.
    ///
    /// It's unsafe, because caller verifies that it is called only once in program's life time.
    /// In most cases this function panics on second call. If there are race cases with threads or
    /// interruptions, then this may not panic on second call.
    pub unsafe fn new_once() -> PinRow {
        let state = core::mem::replace(&mut PINROW_AVAILABLE, false);
        assert!(state, "Singleton creation called second time");
        PinRow([false; NUM_PINS])
    }

    /// If value in arrays is true, then that pin is in use, and can not be "taken out".
    pub fn check_pin(&self, num: u8) -> bool {
        return self.0[num as usize]
    }

    /// Reserve pin for usage
    pub fn get_pin(&mut self, num: u8, mode: PinMode) -> Pin {
        if self.0[num as usize] {
            panic!("Pin already reserved")
        }
        let mut pin = Pin{num, mode};
        pin.set_mode(mode);
        if pin.mode == PinMode::Output {
            pin.digital_write(false);  // By default output is off
        }
        self.0[num as usize] = true;
        return pin;
    }

    /// Return led pin and set it to output
    pub fn get_led(&mut self) -> Pin{
        return self.get_pin(13, PinMode::Output);
    }

    /// Give pin back to pool (consumes Pin)
    pub fn return_pin(&mut self, mut pin: Pin) {
        if !self.0[pin.num as usize] {
            panic!("Internal error!")
        }
        if pin.mode == PinMode::Output {
            pin.digital_write(false);
        }
        //pin.set_mode(PinMode::Input);
        self.0[pin.num as usize] = false;
    }
}

impl Pin {
    /// Set pin mode
    fn set_mode(&mut self, mode: PinMode) {
        let mode_ = match mode {
            PinMode::Input => bindings::INPUT,
            PinMode::Output => bindings::OUTPUT,
            PinMode::InputPullup => bindings::INPUT_PULLUP,
            PinMode::InputPulldown => bindings::INPUT_PULLDOWN,     // What is this?
            PinMode::OutputOpenDrain => bindings::OUTPUT_OPENDRAIN, // What is this?
        } as u8;
        unsafe {
            bindings::pinMode(self.num, mode_);
        }
        self.mode = mode;
        // There is slight delay for pin to raise it's voltage to its maximum
        if mode == PinMode::InputPullup{
            util::delay(1);  // wait 1ms
        }
    }

    /// Set pin to high (true) or low (false)
    #[allow(unused_mut)]
    pub fn digital_write(&mut self, val: bool)
    {
        if self.mode == PinMode::Input {
            panic!("Please set pin to pullup mode by using pin.set_mode(PinMode::InputPullup)")
        } else if (self.mode != PinMode::Output) || (self.mode != PinMode::OutputOpenDrain) {
            panic!("Pin must be set to OUTPUT for it to be written.")
        }
        let value = if val {bindings::HIGH as u8} else {bindings::LOW as u8};
        unsafe {
            bindings::digitalWrite(self.num, value);
        }
    }

    /// Read high (true) or low (false) state from pin
    fn digital_read(&self) -> bool {
        unsafe {
            return bindings::digitalRead(self.num) != 0u8;
        }
    }
}
