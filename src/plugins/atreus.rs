use kaleidoscope_internal::driver::keyscanner::MatrixScanner;

use crate::device::{pins_and_ports::*, DeviceOps};
use crate::driver::{bootloader::avr::Caterina, keyscanner::{Atmega, KeyScannerProps}};

pub type KeyScanner = Atmega;
pub type Bootloader = Caterina;

impl KeyScannerProps for AtreusProps {
    const ROWS: usize = 4;
    const COLS: usize = 12;

    const KEYSCAN_INTERVAL: u16 = 1500;
}

pub struct AtreusProps;

impl AtreusProps {
    pub const SHORT_NAME: &'static str = "atreus";

    pub const MATRIX_ROW_PINS: [u8; Self::ROWS] = [PIN_F6, PIN_F5, PIN_F4, PIN_F1];
    pub const MATRIX_COL_PINS: [u8; Self::COLS] = [
        PIN_F7, PIN_E2, PIN_C7, PIN_C6, PIN_B6, PIN_B5, PIN_D7, PIN_D6, PIN_D4, PIN_D5, PIN_D3,
        PIN_D2,
    ];
}

pub struct Atreus {
    key_scanner: KeyScanner,
}

impl Atreus {
    const LED_COUNT: usize = 0;

    pub const fn new() -> Self {
        Self {
            key_scanner: KeyScanner::new(),
        }
    }

    pub const fn led_count() -> usize {
        Self::LED_COUNT
    }

    pub fn scan_matrix(&mut self) {
        self.key_scanner.scan_matrix();
    }
}

impl DeviceOps for Atreus {
    type KeyScanner = Atmega;

    fn key_scanner(&self) -> &Self::KeyScanner {
        &self.key_scanner
    }

    fn key_scanner_mut(&mut self) -> &mut Self::KeyScanner {
        &mut self.key_scanner
    }
}

pub type Device = Atreus;
pub type DeviceProps = AtreusProps;

#[avr_device::interrupt(atmega32u4)]
fn TIMER1_OVF() {
    use crate::RUNTIME;

    RUNTIME
        .write()
        .device_mut()
        .key_scanner_mut()
        .set_do_scan(true);
}
