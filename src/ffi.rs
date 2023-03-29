use crate::{init_cpu, init_hid, init_millis, init_tc1, init_usb, init_wdt, usb, RUNTIME};

#[no_mangle]
pub extern "C" fn kaleidoscope_setup() {
    let dp = arduino_hal::Peripherals::take().expect("failed to get peripherals");

    let pins = arduino_hal::pins!(dp);

    let _serial = arduino_hal::default_serial!(dp, pins, 9600);

    init_cpu(dp.CPU);

    init_millis(dp.TC0);
    init_tc1(dp.TC1);

    init_wdt(dp.WDT);

    init_usb(dp.USB_DEVICE);
    init_hid(usb().expect("failed to initialize USB"));

    RUNTIME.write().setup().expect("failed to setup runtime");
}

#[no_mangle]
pub extern "C" fn kaleidoscope_loop() -> ! {
    loop {
        RUNTIME.write().main_loop();
    }
}
