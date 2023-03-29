//! Interrupt handler definitions

#[avr_device::nterrupt(atmega32u4)]
fn TIMER1_OVF() {
    use crate::RUNTIME;

    RUNTIME
        .write()
        .device_mut()
        .key_scanner_mut()
        .set_do_scan(true);
}
