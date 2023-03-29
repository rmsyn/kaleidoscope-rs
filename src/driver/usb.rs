use crate::device::{EPDIR, EPTYPE0, EPTYPE1};

#[cfg(feature = "atreus")]
mod atmega32u4;
mod cdc;

#[cfg(feature = "atreus")]
pub use atmega32u4::*;

/// Standard requests
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StandardRequest {
    GetStatus = 0,
    ClearFeature = 1,
    SetFeature = 3,
    SetAddress = 5,
    GetDescriptor = 6,
    SetDescriptor = 7,
    GetConfiguration = 8,
    SetConfiguration = 9,
    GetInterface = 10,
    SetInterface = 11,
    Unknown = 0xff,
}

impl From<u8> for StandardRequest {
    fn from(b: u8) -> Self {
        match b {
            0 => Self::GetStatus,
            1 => Self::ClearFeature,
            3 => Self::SetFeature,
            5 => Self::SetAddress,
            6 => Self::GetDescriptor,
            7 => Self::SetDescriptor,
            8 => Self::GetConfiguration,
            9 => Self::SetConfiguration,
            10 => Self::GetInterface,
            11 => Self::SetInterface,
            _ => Self::Unknown,
        }
    }
}

pub const EP_TYPE_CONTROL: u8 = 0x00;
pub const EP_TYPE_BULK_IN: u8 = (1 << EPTYPE1) | (1 << EPDIR);
pub const EP_TYPE_BULK_OUT: u8 = 1 << EPTYPE1;
pub const EP_TYPE_INTERRUPT_IN: u8 = (1 << EPTYPE1) | (1 << EPTYPE0) | (1 << EPDIR);
pub const EP_TYPE_INTERRUPT_OUT: u8 = (1 << EPTYPE1) | (1 << EPTYPE0);
pub const EP_TYPE_ISOCHRONOUS_IN: u8 = (1 << EPTYPE0) | (1 << EPDIR);
pub const EP_TYPE_ISOCHRONOUS_OUT: u8 = 1 << EPTYPE0;

pub const MAGIC_KEY: u16 = 0x7777;
pub const MAGIC_KEY_POS: u16 = 0x0800;

/// This definitions is useful if you want to reduce the EP_SIZE to 16
/// at the moment only 64 and 16 as EP_SIZE for all EPs are supported except the control endpoint
pub const USB_EP_SIZE: u8 = 64;

/// From ArduinoCore-avr/cores/arduino/USBDesc.h for AtmegaXXu4 devices
pub const USB_ENDPOINTS: usize = 7;

// Set the following line or pass --features cdc_disabled to the compiler
// to disable CDC (serial console via USB).
// That's useful if you want to create an USB device (like an USB Boot Keyboard)
// that works even with problematic devices (like KVM switches).
// Keep in mind that with this change you'll have to use the Arduino's
// reset button to be able to flash it.
#[cfg(feature = "cdc_disabled")]
pub const CDC_ENABLED: bool = false;
#[cfg(not(feature = "cdc_disabled"))]
pub const CDC_ENABLED: bool = true;

pub const CDC_ACM_INTERFACE: u8 = 0;

pub const TRANSFER_PGM: u8 = 0x80;
pub const TRANSFER_RELEASE: u8 = 0x40;
pub const TRANSFER_ZERO: u8 = 0x20;

pub const USB_CONFIG_POWERED_MASK: u8 = 0x40;
pub const USB_CONFIG_BUS_POWERED: u8 = 0x80;
pub const USB_CONFIG_SELF_POWERED: u8 = 0xc0;
pub const USB_CONFIG_REMOTE_WAKEUP: u8 = 0x20;

pub const USB_CONFIG_POWER: u16 = 500;

#[cfg(feature = "arduino")]
pub const USB_VID: u16 = 0x2341;
#[cfg(feature = "sparkfun")]
pub const USB_VID: u16 = 0x1b4f;
#[cfg(not(any(feature = "arduino", feature = "sparkfun")))]
pub const USB_VID: u16 = 0x0000;

// FIXME: define based on conditional compilation
pub const USB_PID: u16 = 0x0000;

pub const USB_VERSION: u16 = 0x200;

pub const MANUFACTURER: u8 = 1;
pub const PRODUCT: u8 = 2;
pub const SERIAL: u8 = 3;

pub const USB_DEVICE_DESCRIPTOR_TYPE: u8 = 1;
pub const USB_CONFIGURATION_DESCRIPTOR_TYPE: u8 = 2;
pub const USB_STRING_DESCRIPTOR_TYPE: u8 = 3;
pub const USB_INTERFACE_DESCRIPTOR_TYPE: u8 = 4;
pub const USB_ENDPOINT_DESCRIPTOR_TYPE: u8 = 5;

// Register bits
pub const SUSPI: u8 = 1 << 0;
pub const WAKEUPI: u8 = 1 << 4;

pub const fn usb_config_power_ma(ma: u16) -> u16 {
    ma / 2
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct USBSetup {
    inner: [u8; Self::LEN],
}

impl USBSetup {
    pub const LEN: usize = 8;

    const IDX_REQUEST_TYPE: usize = 0;
    const IDX_REQUEST: usize = 1;
    const IDX_VALUE_L: usize = 2;
    const IDX_VALUE_H: usize = 3;
    const IDX_INDEX_L: usize = 4;
    const IDX_INDEX_H: usize = 5;
    const IDX_LENGTH_L: usize = 6;
    const IDX_LENGTH_H: usize = 7;

    /// Creates a new [USBSetup].
    pub const fn new() -> Self {
        Self { inner: [0u8; 8] }
    }

    /// Gets the request type.
    pub const fn request_type(&self) -> u8 {
        self.inner[Self::IDX_REQUEST_TYPE]
    }

    /// Gets the request.
    pub const fn request(&self) -> u8 {
        self.inner[Self::IDX_REQUEST]
    }

    /// Gets the value (lower byte).
    pub const fn value_l(&self) -> u8 {
        self.inner[Self::IDX_VALUE_L]
    }

    /// Gets the value (higher byte).
    pub const fn value_h(&self) -> u8 {
        self.inner[Self::IDX_VALUE_H]
    }

    pub fn value(&self) -> u16 {
        ((self.value_h() as u16) << 8) | self.value_l() as u16
    }

    /// Gets the index (lower byte).
    pub fn index(&self) -> u16 {
        u16::from_le_bytes([self.inner[Self::IDX_INDEX_L], self.inner[Self::IDX_INDEX_H]])
    }

    /// Gets the index (higher byte).
    pub fn length(&self) -> u16 {
        u16::from_le_bytes([
            self.inner[Self::IDX_LENGTH_L],
            self.inner[Self::IDX_LENGTH_H],
        ])
    }

    /// Gets the [USBSetup] as byte buffer.
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// Gets the [USBSetup] as a mutable byte buffer.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.inner.as_mut()
    }
}

impl From<&[u8]> for USBSetup {
    fn from(bytes: &[u8]) -> Self {
        let mut inner = [0u8; Self::LEN];

        let len = core::cmp::min(bytes.len(), inner.len());

        for (i, b) in inner[..len].iter_mut().enumerate() {
            *b = bytes[i];
        }

        Self { inner }
    }
}

impl From<[u8; USBSetup::LEN]> for USBSetup {
    fn from(inner: [u8; Self::LEN]) -> Self {
        Self { inner }
    }
}

/// Configuration descriptor
#[repr(C)]
pub struct ConfigDescriptor {
    inner: [u8; Self::LEN],
}

impl ConfigDescriptor {
    pub const LEN: usize = 9;

    const IDX_LEN: usize = 0;
    const IDX_DEV_TYPE: usize = 1;
    const IDX_CLEN_L: usize = 2;
    const IDX_CLEN_H: usize = 3;
    const IDX_NUM_INT: usize = 4;
    const IDX_CONFIG: usize = 5;
    const IDX_ICONFIG: usize = 6;
    const IDX_ATTRIBUTES: usize = 7;
    const IDX_MAX_POWER: usize = 8;

    /// Creates a new [ConfigDescriptor] from total length (`clen`),
    /// and number of interfaces (`num_interfaces`).
    pub const fn new(clen: u16, num_interfaces: u8) -> Self {
        let clen_bytes = clen.to_le_bytes();

        Self {
            inner: [
                9,
                2,
                clen_bytes[0],
                clen_bytes[1],
                num_interfaces,
                1,
                0,
                USB_CONFIG_BUS_POWERED | USB_CONFIG_REMOTE_WAKEUP,
                usb_config_power_ma(USB_CONFIG_POWER) as u8,
            ],
        }
    }

    /// Gets the [ConfigDescriptor] length.
    pub const fn length(&self) -> u8 {
        self.inner[Self::IDX_LEN]
    }

    /// Gets the [ConfigDescriptor] device type.
    pub const fn device_type(&self) -> u8 {
        self.inner[Self::IDX_DEV_TYPE]
    }

    /// Gets the [ConfigDescriptor] total length (lower byte).
    pub const fn clen_l(&self) -> u8 {
        self.inner[Self::IDX_CLEN_L]
    }

    /// Gets the [ConfigDescriptor] total length (higher byte).
    pub const fn clen_h(&self) -> u8 {
        self.inner[Self::IDX_CLEN_H]
    }

    /// Gets the [ConfigDescriptor] total length (higher byte).
    pub const fn clen(&self) -> u16 {
        ((self.inner[Self::IDX_CLEN_H] as u16) << 8) | self.inner[Self::IDX_CLEN_L] as u16
    }

    /// Gets the [ConfigDescriptor] number of interfaces.
    pub const fn num_interfaces(&self) -> u8 {
        self.inner[Self::IDX_NUM_INT]
    }

    /// Gets the [ConfigDescriptor] configuration.
    pub const fn config(&self) -> u8 {
        self.inner[Self::IDX_CONFIG]
    }

    /// Gets the [ConfigDescriptor] interface configuration.
    pub const fn interface_config(&self) -> u8 {
        self.inner[Self::IDX_ICONFIG]
    }

    /// Gets the [ConfigDescriptor] attributes.
    pub const fn attributes(&self) -> u8 {
        self.inner[Self::IDX_ATTRIBUTES]
    }

    /// Gets the [ConfigDescriptor] max power.
    pub const fn max_power(&self) -> u8 {
        self.inner[Self::IDX_MAX_POWER]
    }

    /// Gets the [ConfigDescriptor] as a byte buffer.
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// Gets the [ConfigDescriptor] as a mutable byte buffer.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.inner.as_mut()
    }
}

impl From<&[u8]> for ConfigDescriptor {
    fn from(bytes: &[u8]) -> Self {
        let mut inner = [0u8; Self::LEN];

        let len = core::cmp::min(bytes.len(), inner.len());

        for (i, b) in inner[..len].iter_mut().enumerate() {
            *b = bytes[i];
        }

        Self { inner }
    }
}

impl From<[u8; ConfigDescriptor::LEN]> for ConfigDescriptor {
    fn from(inner: [u8; Self::LEN]) -> Self {
        Self { inner }
    }
}

#[repr(C)]
pub struct DeviceDescriptor {
    inner: [u8; Self::LEN],
}

impl DeviceDescriptor {
    pub const LEN: usize = 18;

    const IDX_LEN: usize = 0;
    const IDX_DEV_TYPE: usize = 1;
    const IDX_USB_VER_L: usize = 2;
    const IDX_USB_VER_H: usize = 3;
    const IDX_DEV_CLASS: usize = 4;
    const IDX_DEV_SUB_CLASS: usize = 5;
    const IDX_DEV_PROTOCOL: usize = 6;
    const IDX_PACKET_SIZE0: usize = 7;
    const IDX_ID_VENDOR_L: usize = 8;
    const IDX_ID_VENDOR_H: usize = 9;
    const IDX_ID_PRODUCT_L: usize = 10;
    const IDX_ID_PRODUCT_H: usize = 11;
    const IDX_DEV_VER_L: usize = 12;
    const IDX_DEV_VER_H: usize = 13;
    const IDX_MANUFACTURER: usize = 14;
    const IDX_PRODUCT: usize = 15;
    const IDX_SERIAL_NUMBER: usize = 16;
    const IDX_NUM_CONFIGS: usize = 17;

    pub const fn new(
        dev_class: u8,
        dev_sub_class: u8,
        dev_protocol: u8,
        packet_size0: u8,
        id_vendor: u16,
        id_product: u16,
        version: u16,
        manufacturer: u8,
        product: u8,
        serial_number: u8,
        num_configs: u8,
    ) -> Self {
        let usb_version = USB_VERSION.to_le_bytes();
        let vendor = id_vendor.to_le_bytes();
        let prod = id_product.to_le_bytes();
        let ver = version.to_le_bytes();

        Self {
            inner: [
                18,
                1,
                usb_version[0],
                usb_version[1],
                dev_class,
                dev_sub_class,
                dev_protocol,
                packet_size0,
                vendor[0],
                vendor[1],
                prod[0],
                prod[1],
                ver[0],
                ver[1],
                manufacturer,
                product,
                serial_number,
                num_configs,
            ],
        }
    }

    pub const fn len(&self) -> u8 {
        self.inner[Self::IDX_LEN]
    }

    pub const fn device_type(&self) -> u8 {
        self.inner[Self::IDX_DEV_TYPE]
    }

    pub const fn usb_version(&self) -> u16 {
        u16::from_le_bytes([
            self.inner[Self::IDX_USB_VER_L],
            self.inner[Self::IDX_USB_VER_H],
        ])
    }

    pub const fn device_class(&self) -> u8 {
        self.inner[Self::IDX_DEV_CLASS]
    }

    pub const fn device_sub_class(&self) -> u8 {
        self.inner[Self::IDX_DEV_SUB_CLASS]
    }

    pub const fn device_protocol(&self) -> u8 {
        self.inner[Self::IDX_DEV_PROTOCOL]
    }

    pub const fn packet_size0(&self) -> u8 {
        self.inner[Self::IDX_PACKET_SIZE0]
    }

    pub const fn id_vendor(&self) -> u16 {
        u16::from_le_bytes([
            self.inner[Self::IDX_ID_VENDOR_L],
            self.inner[Self::IDX_ID_VENDOR_H],
        ])
    }

    pub const fn id_product(&self) -> u16 {
        u16::from_le_bytes([
            self.inner[Self::IDX_ID_PRODUCT_L],
            self.inner[Self::IDX_ID_PRODUCT_H],
        ])
    }

    pub const fn device_version(&self) -> u16 {
        u16::from_le_bytes([
            self.inner[Self::IDX_DEV_VER_L],
            self.inner[Self::IDX_DEV_VER_H],
        ])
    }

    pub const fn manufacturer(&self) -> u8 {
        self.inner[Self::IDX_MANUFACTURER]
    }

    pub const fn product(&self) -> u8 {
        self.inner[Self::IDX_PRODUCT]
    }

    pub const fn serial_number(&self) -> u8 {
        self.inner[Self::IDX_SERIAL_NUMBER]
    }

    pub const fn num_configurations(&self) -> u8 {
        self.inner[Self::IDX_NUM_CONFIGS]
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_ref()
    }
}
