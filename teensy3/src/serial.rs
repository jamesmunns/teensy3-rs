use bindings;
use core::fmt;

#[derive(Copy, Clone)]
pub struct Serial;

/// Struct for the onboard USB port. Should be treated as a singleton
impl Serial {
    pub fn readable(self) -> bool {
        unsafe {
            bindings::usb_serial_available() > 0
        }
    }

    pub fn read_byte(self) -> u8 {
        self.try_read_byte().unwrap()
    }

    pub fn try_read_byte(self) -> Result<u8, &'static str> {
        match unsafe { bindings::usb_serial_getchar() } {
            -1 => Err("usb_serial_getchar returned -1"),
            byte => Ok(byte as u8)
        }
    }

    pub fn write_bytes(self, bytes: &[u8]) -> Result<(), ()> {
        unsafe {
            if bindings::usb_serial_write(bytes.as_ptr() as *const _, bytes.len() as u32) >= 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

/// print!() using the onboard USB serial port
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        ::core::fmt::Write::write_fmt(&mut $crate::serial::Serial, format_args!($($arg)*)).ok();
    }
}

/// println!() using the onboard USB serial port
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        ::core::fmt::Write::write_fmt(&mut $crate::serial::Serial, format_args!($($arg)*)).ok();
    }
}
