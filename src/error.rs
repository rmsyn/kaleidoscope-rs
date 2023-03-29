use keyboardio_hid::usb_device::UsbError;

use crate::event_handler::EventHandlerError;

pub type Result<T> = core::result::Result<T, Error>;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Peripherals = 1,
    USB = 2,
    CPU,
    HID,
    TC1,
    WDT,
    Layer,
    EventConsumed,
    EventAbort,
    EventError,
}

impl Into<&'static str> for Error {
    fn into(self) -> &'static str {
        match self {
            Self::Peripherals => "unable to acquire peripherals",
            Self::USB => "USB error",
            Self::HID => "HID error",
            Self::CPU => "CPU error",
            Self::TC1 => "TC1 error",
            Self::WDT => "WDT error",
            Self::Layer => "Layer error",
            Self::EventConsumed => "Event handler consumed the event",
            Self::EventAbort => "Event handler aborted",
            Self::EventError => "Event handler raised an unknown error",
        }
    }
}

impl From<EventHandlerError> for Error {
    fn from(event: EventHandlerError) -> Self {
        match event {
            EventHandlerError::EventConsumed => Self::EventConsumed,
            EventHandlerError::Abort => Self::EventAbort,
            EventHandlerError::Error => Self::EventError,
        }
    }
}

impl From<UsbError> for Error {
    fn from(_err: UsbError) -> Self {
        Self::USB
    }
}
