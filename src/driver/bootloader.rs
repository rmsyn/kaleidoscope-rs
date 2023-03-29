pub mod avr;

pub trait Base {
    fn setup() {}
    fn reboot_bootloader() -> ! {
        loop {}
    }
}
