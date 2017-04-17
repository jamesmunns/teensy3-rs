//! Safe bindings for the primary SPI port of the Teensy 3.1/3.2

use bindings;

/// Bit order for SPI data TX/RX
#[derive(Debug, PartialEq)]
pub enum BitOrder {
    MsbFirst,
    LsbFirst,
}

/// Data TX Mode
#[derive(Debug, PartialEq)]
pub enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3
}

/// Settings structure for the SPI peripheral
#[derive(Debug, PartialEq)]
pub struct SpiSettings {
    max_clock: u32,
    order: BitOrder,
    mode: Mode,
    ctar: RenderedSpi,
}

type RenderedSpi = u32;

impl SpiSettings {
    pub fn new(max_clock: u32, order: BitOrder, mode: Mode) -> SpiSettings {
        let mut new = SpiSettings {
            max_clock: max_clock,
            order: order,
            mode: mode,
            ctar: 0
        };
        new.ctar = new.render();
        new
    }

    fn render(&self) -> RenderedSpi {
        let mut t = unsafe {
            let mut t_out: u32 = bindings::SPISettings_ctar_div_table[0] as u32;
            for t in bindings::SPISettings_ctar_div_table.iter() {
                t_out = *t as u32;
                if self.max_clock > (bindings::F_BUS as u32 / t_out) {
                    break;
                }
            }
            t_out
        };

        // uint32_t c = SPI_CTAR_FMSZ(7);
        let mut c: u32 = (7 & 15) << 27;

        // if (bitOrder == LSBFIRST) c |= SPI_CTAR_LSBFE;
        if self.order == BitOrder::LsbFirst {
            c |= 0x01000000;
        }

        // if (dataMode & 0x04) {
        //     c |= SPI_CTAR_CPHA;
        //     t = (t & 0xFFFF0FFF) | ((t & 0xF000) >> 4);
        // }
        // if (dataMode & 0x08) {
        //     c |= SPI_CTAR_CPOL;
        // }
        match self.mode {
            Mode::Mode0 => {},
            Mode::Mode1 => {
                c |= 0x02000000;
                t = (t & 0xFFFF0FFF) | ((t & 0xF000) >> 4);
            },
            Mode::Mode2 => {
                c |= 0x04000000
            },
            Mode::Mode3 => {
                c |= 0x06000000;
                t = (t & 0xFFFF0FFF) | ((t & 0xF000) >> 4);
            },
        }

        c | t
    }
}

#[derive(Copy, Clone)]
pub struct Spi;
use bindings::SPIClass;

impl Spi {
    /// Initialize the SPI peripheral
    pub fn begin(&self) {
        unsafe {
            SPIClass::begin();
        }
    }

    /// Begin a single SPI data transaction
    pub fn begin_transaction(&self, settings: &SpiSettings) {
        unsafe {
            SPIClass::beginTransaction(bindings::SPISettings{
                ctar: settings.ctar,
            });
        }
    }

    /// Finalize a SPI data transaction
    pub fn end_transaction(&self) {
        unsafe {
            SPIClass::endTransaction();
        }
    }

    /// Send N bytes, replacing each input byte with the associated output byte
    ///
    /// TODO: Improve once https://github.com/rust-lang/rfcs/issues/1038 lands
    pub fn transfer_replace(&self, data: &mut [u8]) {
        for mut byte in data.iter_mut() {
            *byte = unsafe {
                SPIClass::transfer(*byte)
            }
        }
    }
}
