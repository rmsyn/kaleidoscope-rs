use atmega_hal::wdt::Timeout;

use crate::driver::{bootloader::Base, wdt::wdt_enable};

/// Magic bits to reboot into the bootloader, and stay there.
pub const BOOT_KEY: u16 = 0x7777;
pub const BOOT_KEY_PTR: u16 = 0x0800;

pub struct Caterina;

impl Base for Caterina {
    /// Set the magic bits to get a Caterina-based device
    /// to reboot into the bootloader and stay there, rather
    /// than run move onward
    ///
    /// These values are the same as those defined in
    /// Caterina.c:
    /// https://github.com/arduino/ArduinoCore-avr/blob/5755ddea49fa69d6c505c772ebee5af5078e2ebf/bootloaders/caterina/Caterina.c#L130-L133
    fn reboot_bootloader() -> ! {
        // Stash the magic key
        unsafe {
            core::ptr::write_volatile(BOOT_KEY_PTR as *mut u16, BOOT_KEY);
        }

        // Set a watchdog timer
        if let Err(_err) = wdt_enable(Timeout::Ms125) {
            // FIXME: log error
        }

        loop {
            // This infinite loop ensures nothing else
            avr_device::asm::nop();
        } // happens before the watchdog reboots us
    }
}
