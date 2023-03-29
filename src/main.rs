#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;

use kaleidoscope::{return_on_err, hid_mut, usb_device_mut};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().expect("failed to get peripherals");

    let pins = arduino_hal::pins!(dp);

    let _serial = arduino_hal::default_serial!(dp, pins, 9600);

    kaleidoscope::init_cpu(dp.CPU);

    kaleidoscope::init_millis(dp.TC0);
    kaleidoscope::init_tc1(dp.TC1);

    kaleidoscope::init_wdt(dp.WDT);

    kaleidoscope::init_usb(dp.USB_DEVICE);

    let usb = kaleidoscope::usb().expect("null USB");

    kaleidoscope::init_hid(usb);

    kaleidoscope::init_usb_device(usb);

    kaleidoscope::RUNTIME.write().setup().expect("failed to setup runtime");

    loop{
        kaleidoscope::RUNTIME.write().main_loop();
    }
}

#[avr_device::interrupt(atmega32u4)]
fn USB_GEN() {
    return_on_err!(usb_device_mut()).poll(&mut [
                    return_on_err!(hid_mut()).boot_keyboard.hid_class_mut(),
                    return_on_err!(hid_mut()).nkro_keyboard.hid_class_mut(),
                    return_on_err!(hid_mut()).media_keyboard.hid_class_mut(),
                    return_on_err!(hid_mut()).system_control_keyboard.hid_class_mut(),
    ]);
}

#[avr_device::interrupt(atmega32u4)]
fn USB_COM() {
    return_on_err!(usb_device_mut()).poll(&mut [
                    return_on_err!(hid_mut()).boot_keyboard.hid_class_mut(),
                    return_on_err!(hid_mut()).nkro_keyboard.hid_class_mut(),
                    return_on_err!(hid_mut()).media_keyboard.hid_class_mut(),
                    return_on_err!(hid_mut()).system_control_keyboard.hid_class_mut(),
    ]);
}
