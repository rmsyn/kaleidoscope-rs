use crate::device::FLASHEND;

pub const NEW_LUFA_SIGNATURE: u16 = 0xdcfb;

pub fn is_lufa_bootloader() -> bool {
    let addr = (FLASHEND - 1) as u32;
    let sig = unsafe { core::ptr::read_volatile(addr as *const u32) };

    sig == (NEW_LUFA_SIGNATURE as u32)
}
