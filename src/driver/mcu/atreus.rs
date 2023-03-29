use core::sync::atomic::{AtomicBool, Ordering};

use avr_device::interrupt;
use keyboardio_hid::usb_device::device::UsbDeviceState;

use super::Mcu;
use crate::{cpu, detach_from_host, init_usb_device, error::Result, plugins::atreus::Atreus, return_on_err, usb, usb_device};

static WAS_CONFIGURED: AtomicBool = AtomicBool::new(false);

impl Mcu for Atreus {
    const DISABLE_JTAG: bool = false;
    const DISABLE_CLOCK_DIVISION: bool = false;

    fn detach_from_host() -> Result<()> {
        detach_from_host()
    }

    fn attach_to_host() -> Result<()> {
        init_usb_device(usb()?);
        Ok(())
    }

    fn poll_usb_reset() -> bool {
        let mut ret = false;

        if WAS_CONFIGURED.load(Ordering::Relaxed) {
            if !Self::usb_configured() {
                WAS_CONFIGURED.store(false, Ordering::SeqCst);
                ret = true;
            }
        } else {
            if Self::usb_configured() {
                WAS_CONFIGURED.store(true, Ordering::SeqCst);
            }
        }

        ret
    }

    fn usb_configured() -> bool {
        if let Ok(usb) = usb_device() {
            usb.state() == UsbDeviceState::Configured
        } else {
            false
        }
    }

    fn disable_jtag() -> Result<()> {
        interrupt::free(|cs| {
            cpu()?
                .borrow(cs)
                .mcucr
                .modify(|_, w| w.jtd().set_bit().jtd().set_bit());

            Ok(())
        })
    }

    fn disable_clock_division() -> Result<()> {
        interrupt::free(|cs| {
            cpu()?
                .borrow(cs)
                .clkpr
                .modify(|_, w| {
                    // Enable writing the CLKPS bits.
                    //
                    // See CLKPR in the Microchip documentation.
                    w.clkpce().set_bit();

                    // Setting CLKPS to 0b0000 sets clock division to 1.
                    //
                    // See CLKPR in the Microchip documentation.
                    w.clkps().val_0x00()
                });

            Ok(())
        })
    }

    fn setup() {
        if Self::DISABLE_JTAG {
            return_on_err!(Self::disable_jtag());
        }

        if Self::DISABLE_CLOCK_DIVISION {
            return_on_err!(Self::disable_clock_division());
        }
    }
}
