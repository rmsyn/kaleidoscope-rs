use crate::error::Result;

mod atreus;

pub trait Mcu {
    const DISABLE_JTAG: bool;
    const DISABLE_CLOCK_DIVISION: bool;

    /// Detaching from the host.
    ///
    /// The methods themselves implement detaching from / attaching to the host,
    /// without rebooting the device, and remaining powered in between.
    ///
    /// Intended to be used in cases where we want to change some settings between
    /// detach and attach.
    fn detach_from_host() -> Result<()>;

    /// Attaching to the host.
    ///
    /// The methods themselves implement detaching from / attaching to the host,
    /// without rebooting the device, and remaining powered in between.
    ///
    /// Intended to be used in cases where we want to change some settings between
    /// detach and attach.
    fn attach_to_host() -> Result<()>;

    /// Poll the USB device for a bus reset.
    ///
    /// This default implementation uses a change in USBConfigured() as a proxy
    /// for actually detecting a bus reset.
    fn poll_usb_reset() -> bool;

    /// These two lines here are the result of many hours spent chasing ghosts.
    /// These are great lines, and we love them dearly, for they make a set of
    /// pins that would otherwise be reserved for JTAG accessible from the
    /// firmware.
    ///
    /// Most AVR chips that get put into keyboards have the JTAG port disabled in
    /// fuses, but some do not. When they're used for JTAG, then no matter what
    /// we do in the firmware, they will not be affected. So in case JTAG is not
    /// disabled in fuses, we want to do that in the firmware. Luckily for us,
    /// that's doable, we just have to write the JTD bit into MCUCR twice within
    /// four cycles. These two lines do just that.
    ///
    /// For more information, see the ATmega16U4/ATmega32U4 datasheet, the
    /// following sections:
    ///  - 2.2.7 (PIN Descriptions; PIN F)
    ///  - 7.8.7 (On-chip Debug System)
    ///  - 26.5.1 (MCU Control Register – MCUCR)
    fn disable_jtag() -> Result<()>;

    /// Disable clock division.
    fn disable_clock_division() -> Result<()>;

    fn setup();

    fn usb_configured() -> bool {
        true
    }
}
