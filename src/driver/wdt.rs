#[cfg(feature = "atmega32u4")]
use atmega_hal::{pac::WDT, wdt::Timeout};
use avr_device::interrupt;

use crate::{error::Result, wdt};

/// Taken from [atmega-hal] implementation.
#[cfg(feature = "atmega32u4")]
fn set_timeout(wdt: &WDT, timeout: Timeout) {
    match timeout {
        Timeout::Ms16 => wdt.wdtcsr.write(|w| w.wdpl().cycles_2k_512k()),
        Timeout::Ms32 => wdt.wdtcsr.write(|w| w.wdpl().cycles_4k_1024k()),
        Timeout::Ms64 => wdt.wdtcsr.write(|w| w.wdpl().cycles_8k()),
        Timeout::Ms125 => wdt.wdtcsr.write(|w| w.wdpl().cycles_16k()),
        Timeout::Ms250 => wdt.wdtcsr.write(|w| w.wdpl().cycles_32k()),
        Timeout::Ms500 => wdt.wdtcsr.write(|w| w.wdpl().cycles_64k()),
        Timeout::Ms1000 => wdt.wdtcsr.write(|w| w.wdpl().cycles_128k()),
        Timeout::Ms2000 => wdt.wdtcsr.write(|w| w.wdpl().cycles_256k()),
        Timeout::Ms4000 => wdt
            .wdtcsr
            .write(|w| w.wdph().set_bit().wdpl().cycles_2k_512k()),
        Timeout::Ms8000 => wdt
            .wdtcsr
            .write(|w| w.wdph().set_bit().wdpl().cycles_4k_1024k()),
    };
}

/// Enable the watchdog timer.
///
/// Taken from [avr-hal-generic].
#[inline]
pub fn wdt_enable(timeout: Timeout) -> Result<()> {
    let wdt_lock = wdt()?;

    interrupt::free(|cs| {
        // Reset the watchdog timer.
        wdt_reset();

        // Enable watchdog configuration mode.
        let wdt = wdt_lock.borrow(cs);

        wdt.wdtcsr
            .modify(|_, w| w.wdce().set_bit().wde().set_bit());

        // Set the timeout in milliseconds.
        set_timeout(wdt, timeout);

        // Disable watchdog configuration mode.
        wdt.wdtcsr
            .modify(|_, w| w.wde().set_bit().wdce().clear_bit());
    });

    Ok(())
}

/// Disable the watchdog timer.
///
/// Taken from [avr-hal-generic].
#[inline]
pub fn wdt_disable() -> Result<()> {
    let wdt_lock = wdt()?;

    // The sequence for clearing WDE is as follows:
    //
    //     1. In the same operation, write a logic one to the Watchdog change enable bit
    //        (WDCE) and WDE. A logic one must be written to WDE regardless of the
    //        previous value of the WDE bit.
    //     2. Within the next four clock cycles, clear the WDE and WDCE bits.
    //        This must be done in one operation.
    avr_device::interrupt::free(|cs| {
        // Reset the watchdog timer.
        wdt_reset();

        let wdt = wdt_lock.borrow(cs);

        // Enable watchdog configuration mode.
        wdt.wdtcsr
            .modify(|_, w| w.wdce().set_bit().wde().set_bit());

        // Disable watchdog.
        wdt.wdtcsr.reset();
    });

    Ok(())
}

/// Reset the watchdog timer.
///
/// Taken from [avr-hal-generic](https://github.com/Rahix/avr-hal).
#[inline]
pub fn wdt_reset() {
    avr_device::asm::wdr();
}
