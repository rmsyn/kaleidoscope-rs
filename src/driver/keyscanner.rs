pub(crate) mod atmega;
pub(crate) mod base;

pub use atmega::Atmega;

pub trait KeyScannerProps {
    const ROWS: usize;
    const COLS: usize;

    const KEYSCAN_INTERVAL: u16;
}
