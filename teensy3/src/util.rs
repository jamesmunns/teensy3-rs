use bindings;

pub fn delay(ms: u32) {
    unsafe {
        bindings::delay(ms);
    }
}

pub fn digital_write(pin: u8, val: bool)
{
    unsafe {
        bindings::digitalWrite(pin,
            if val {bindings::HIGH as u8} else {bindings::LOW as u8});
    }
}

pub fn digital_read(pin: u8) -> bool {
    unsafe {
        if bindings::digitalRead(pin) == 0u8 {false} else {true}
    }
}

#[derive(Debug)]
pub enum PinMode {
    Input,
    Output,
    InputPullup,
    InputPulldown,
    OutputOpenDrain,
}

pub fn pin_mode(pin: u8, mode: PinMode) {
    unsafe {
        bindings::pinMode(pin, match mode {
            PinMode::Input => bindings::INPUT,
            PinMode::Output => bindings::OUTPUT,
            PinMode::InputPullup => bindings::INPUT_PULLUP,
            PinMode::InputPulldown => bindings::INPUT_PULLDOWN,
            PinMode::OutputOpenDrain => bindings::OUTPUT_OPENDRAIN,
        } as u8);
    }
}
