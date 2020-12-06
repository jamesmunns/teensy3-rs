//! GPIO pin utilities. API is designed to be safe, and it panics if there are multiple references
//! to same pin.
//!
//! If API feels too restricting for some purpose, then sorry, author is not sure
//! what operations are undefined behaviour in original API. For example writing to input pin

use bindings;
use core::convert::TryInto;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PinMode {
    /// Read digital signal, by determining whether voltage is 0V or more
    Input,
    /// High current output (~40mA). This current can e.g. light up leds.
    ///
    /// If interest is to just check whether two pins are connected, use `InputPullup`
    /// voltage source for that. Do not connect `Output` pin directly to `Input` pin,
    /// as output may produce too high current for input.
    Output,
    /// `InputPullup` is similar to `Input`, and it is made for implementing push buttons. Voltage
    /// is raised for pin, but it is done so that maximum output current output is very low.
    /// If this pin is not connected to anything, then voltage is high. If pin is connected to
    /// ground, then voltage is low on the pin. High and low values are measured with
    /// `digital_read`. Alternatively to connecting other end to ground is to connect it to
    /// pin which is set to mode `OutputOpenDrain`.
    InputPullup,
    InputPulldown,
    /// This pin is connected to ground, in default LOW state. However, if HIGH
    /// is written to it, then this pin is not connected to anything.
    OutputOpenDrain,
}

/// Object corresponding to physical pin. There is only one instance per pin.
/// To prevent duplication, fields are private, and Copy and Clone are not derived.
#[derive(Debug)]
pub struct Pin {
    num: u8,    // there are strictly less than 256 ports on teensy board
    mode: PinMode,
}

static mut PINROW_AVAILABLE: bool = true;

pub const NUM_PINS: usize = bindings::CORE_NUM_TOTAL_PINS as usize;
pub const LED_PIN: usize = bindings::LED_BUILTIN as usize;

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
#[allow(clippy::needless_doctest_main)]
pub struct PinRow([bool; NUM_PINS]);

impl PinRow {
    /// Returns singleton struct that can be used to control GPIO pins. It can be called
    /// only once. It is intended to be called in initialization function of program.
    ///
    /// # Safety
    /// Caller verifies that it is called only once in program's life time. In most cases this
    /// function panics on second call. If there are race cases with threads or interruptions,
    /// then this may not panic on second call. So this could also marked as unsafe function.
    pub fn new_once() -> PinRow {
        let state = unsafe { core::mem::replace(&mut PINROW_AVAILABLE, false) };
        assert!(state, "Singleton creation called second time");
        PinRow([false; NUM_PINS])
    }

    /// Checks if pin has been already reserved. If false, then `get_pin()` can be called for that
    pub fn is_used(&self, num: usize) -> bool {
        return self.0[num]
    }

    /// Reserve pin for usage
    pub fn get_pin(&mut self, num: usize, mode: PinMode) -> Pin {
        // If value in arrays is true, then that pin can not be "taken out".
        assert!(!self.0[num], "Pin {} already reserved", num);
        let mut pin = Pin{num: num.try_into().unwrap(), mode};
        pin.set_mode(mode);
        if pin.mode == PinMode::Output {
            pin.digital_write(false);  // By default output is off
        }
        self.0[num as usize] = true;
        return pin;
    }

    /// Return led pin and set it to output
    pub fn get_led(&mut self) -> Pin{
        let mut led = self.get_pin(LED_PIN, PinMode::Output);
        led.digital_write(false);
        return led;
    }

    /// Give pin back to pool (consumes Pin)
    pub fn return_pin(&mut self, mut pin: Pin) {
        assert!(self.0[pin.num as usize], "Internal error!");
        if pin.mode == PinMode::Output {
            pin.digital_write(false);
        }
        //pin.set_mode(PinMode::Input);
        self.0[pin.num as usize] = false;
    }
}

impl Pin {
    /// Set pin mode
    /// If setting to `INPUT`, then pullup resistor is disabled.
    /// If setting to `INPUT_PULLUP`, it may take time to rise voltage. Call e.g. `util::delay(10)`
    pub fn set_mode(&mut self, mode: PinMode) {
        let mode_ = match mode {
            PinMode::Input => bindings::INPUT,
            PinMode::Output => bindings::OUTPUT,
            PinMode::InputPullup => bindings::INPUT_PULLUP,
            PinMode::InputPulldown => bindings::INPUT_PULLDOWN,     // What is this?
            PinMode::OutputOpenDrain => bindings::OUTPUT_OPENDRAIN,
        } as u8;
        unsafe {
            bindings::pinMode(self.num, mode_);
        }
        self.mode = mode;

        if mode == PinMode::Input{  // Disable pullup in input mode
            unsafe{bindings::digitalWrite(self.num, bindings::LOW as u8)};
        }
    }

    /// Set pin to high (true) or low (false)
    #[allow(unused_mut)]
    pub fn digital_write(&mut self, val: bool)
    {
        match self.mode {
            PinMode::Output | PinMode::OutputOpenDrain => {  // Correct path
                let value = if val {bindings::HIGH as u8} else {bindings::LOW as u8};
                unsafe {
                    bindings::digitalWrite(self.num, value);
                }
            },
            PinMode::Input => {
                panic!("Tried to write to pin while it was not in `INPUT` mode. \
                \nSet pin to pullup mode by using `pin.set_mode(PinMode::InputPullup)`");
            },
            _ => {
                panic!("Tried to write to pin while it was not in `INPUT` mode. \
                \nPin must be set to `OUTPUT` for it to be written.");
            },
        }
    }

    /// Read high (true) or low (false) state from pin
    pub fn digital_read(&self) -> bool {
        unsafe {
            return bindings::digitalRead(self.num) != 0u8;
        }
    }

    /// Return pin number
    pub fn get_num(&self) -> usize {
        return self.num as usize;
    }

    /// Return pin mode
    pub fn get_mode(&self) -> PinMode {
        return self.mode;
    }
}
