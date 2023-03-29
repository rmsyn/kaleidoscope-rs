#![no_std]
#![feature(abi_avr_interrupt)]
#![cfg_attr(target_arch = "avr", feature(asm_experimental_arch))]

use arduino_hal::pac;
use avr_device::interrupt::{CriticalSection, Mutex};
use keyboardio_hid::{KeyboardUsbBus, KeyboardUsbBusAllocator};
use keyboardio_hid::usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};

#[macro_use(bitfield)]
extern crate bitfield;

// Re-exports
#[macro_use(lshift, M, ML, MO, TG)]
extern crate kaleidoscope_internal;

pub use kaleidoscope_internal::{device, hid_tables, key_addr, key_defs, matrix_addr};

/// Atomic helper structs for atomic integral types larger than 16-bits
pub mod atomic;
/// Bootloader utilities
pub mod bootloader;
/// Driver definitions
pub mod driver;
/// Library error types
pub mod error;
/// C FFI functions for creating an Arduino sketch
pub mod ffi;
/// Event handler trait definition
pub mod event_handler;
/// Event hook definitions
pub mod hooks;
/// Key address map definitions
pub mod key_addr_map;
/// Key event definitions
pub mod key_event;
/// Key map definitions
pub mod key_map;
/// Keyswitch state definitions
pub mod keyswitch_state;
/// Layer definitions and helper functions
pub mod layers;
/// Collection of live key states
pub mod live_keys;
/// Lock definitions
pub mod lock;
mod macros;
/// millis implementation based on the [avr-hal/uno-millis](https://github.com/Rahix/avr-hal/blob/main/examples/arduino-uno/src/bin/uno-millis.rs) example
pub mod millis;
/// Board-specific plugins
pub mod plugins;
/// Runtime definitions
pub mod runtime;
/// Various utilities
pub mod util;

pub use device::*;
pub use event_handler::*;
pub use ffi::*;
pub use hooks::*;
pub use key_addr::*;
pub use key_defs::*;
pub use key_event::*;
pub use key_map::*;
pub use layers::*;
pub use live_keys::*;
pub use millis::*;
pub use runtime::Runtime;

use driver::hid::{ActiveKeyboard, HIDKeyboard};
pub use error::{Error, Result};

pub static mut CPU: Option<Mutex<pac::CPU>> = None;
pub static mut TC1: Option<Mutex<pac::TC1>> = None;
pub static mut WDT: Option<Mutex<pac::WDT>> = None;

pub static mut HID: Option<HIDKeyboard> = None;
pub static mut USB: Option<KeyboardUsbBusAllocator> = None;
pub static mut USB_DEVICE: Option<UsbDevice<'static, KeyboardUsbBus>> = None;

pub static RUNTIME: lock::Spinlock<Runtime> = lock::Spinlock::new(Runtime::new());
pub static LIVE_KEYS: lock::Spinlock<LiveKeys> = lock::Spinlock::new(LiveKeys::new());
pub static LAYER: lock::Spinlock<Layer> = lock::Spinlock::new(Layer::new());

#[allow(dead_code)]
type RX = atmega_hal::port::Pin<atmega_hal::port::mode::Input, atmega_hal::port::PD2>;
#[allow(dead_code)]
type TX = atmega_hal::port::Pin<atmega_hal::port::mode::Output, atmega_hal::port::PD3>;
#[allow(dead_code)]
type Clock = arduino_hal::DefaultClock;
#[allow(dead_code)]
type Serial = atmega_hal::usart::Usart<atmega_hal::pac::USART1, RX, TX, Clock>;

pub fn init_cpu(cpu: pac::CPU) {
    unsafe { CPU.replace(Mutex::new(cpu)); }
}

pub fn cpu() -> Result<&'static Mutex<pac::CPU>> {
    unsafe { CPU.as_ref().ok_or(Error::CPU) }
}

pub fn init_usb(usb: pac::USB_DEVICE) {
    unsafe { USB.replace(KeyboardUsbBus::new(usb)); }
}

pub fn usb() -> Result<&'static KeyboardUsbBusAllocator> {
    unsafe { USB.as_ref().ok_or(Error::USB) }
}

pub fn init_usb_device(usb_bus: &'static KeyboardUsbBusAllocator) {
    let usb_device = attach_to_host(usb_bus);

    unsafe { USB_DEVICE.replace(usb_device); }
}

pub fn usb_device() -> Result<&'static UsbDevice<'static, KeyboardUsbBus>> {
    unsafe { USB_DEVICE.as_ref().ok_or(Error::USB) }
}

pub fn usb_device_mut() -> Result<&'static mut UsbDevice<'static, KeyboardUsbBus>> {
    unsafe { USB_DEVICE.as_mut().ok_or(Error::USB) }
}

/// Attaches the device to the host.
pub fn attach_to_host(
    usb_bus: &'static KeyboardUsbBusAllocator,
) -> UsbDevice<'static, KeyboardUsbBus> {
    use driver::hid::settings;

    // Creating the UsbDevice freezes allocation, and calls UsbBus::enable.
    // UsbBus::enable clears the UDCON::detach bit.
    let usb_vid_pid = UsbVidPid(settings::USB_VID, settings::USB_PID);

    UsbDeviceBuilder::new(usb_bus, usb_vid_pid)
        .manufacturer(settings::MANUFACTURER)
        .product(settings::PRODUCT)
        .build()
}

/// Detaches the device from the host.
///
/// After re-attaching, all state is reset the originally configured values.
pub fn detach_from_host() -> Result<()> {
    unsafe {
        USB_DEVICE
            .as_mut()
            .ok_or(Error::USB)?
            .force_reset()?;
    }

    Ok(())
}

pub fn init_hid(usb_bus: &'static KeyboardUsbBusAllocator) {
    unsafe { HID.replace(HIDKeyboard::new(usb_bus, ActiveKeyboard::Boot)); }
}

pub fn hid() -> Result<&'static HIDKeyboard<'static>> {
    unsafe { HID.as_ref().ok_or(Error::HID) }
}

pub fn hid_mut() -> Result<&'static mut HIDKeyboard<'static>> {
    unsafe { HID.as_mut().ok_or(Error::HID) }
}

pub fn init_tc1(tc1: pac::TC1) {
    unsafe { TC1.replace(Mutex::new(tc1)); }
}

pub fn tc1() -> Result<&'static Mutex<pac::TC1>> {
    unsafe { TC1.as_ref().ok_or(Error::TC1) }
}

pub fn init_wdt(wdt: pac::WDT) {
    unsafe { WDT.replace(Mutex::new(wdt)); }
}

pub fn wdt() -> Result<&'static Mutex<pac::WDT>> {
    unsafe { WDT.as_ref().ok_or(Error::WDT) }
}

// SAFETY: this function should only be called after disabling interrupts
//
// Needed for manually implementing memory barriers without constantly disabling/enabling
// interrupts.
#[allow(dead_code)]
pub(crate) unsafe fn cs<'a>() -> CriticalSection<'a> {
    CriticalSection::new()
}
