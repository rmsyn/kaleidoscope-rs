pub const fn bit_read(value: u8, bit: u8) -> u8 {
    (value >> bit) & 0x01
}

pub const fn bit_set(value: u8, bit: u8) -> u8 {
    value | (1 << bit)
}

pub const fn bit_clear(value: u8, bit: u8) -> u8 {
    value & !(1 << bit)
}

pub const fn bit_write(value: u8, bit: u8, bit_value: u8) -> u8 {
    if bit_value != 0 {
        bit_set(value, bit)
    } else {
        bit_clear(value, bit)
    }
}
