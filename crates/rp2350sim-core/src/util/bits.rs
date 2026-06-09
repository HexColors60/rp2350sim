//! Bit manipulation utilities.

/// Extract bits from a value.
#[inline]
pub const fn extract_bits(value: u32, start: u32, len: u32) -> u32 {
    (value >> start) & ((1 << len) - 1)
}

/// Insert bits into a value.
#[inline]
pub const fn insert_bits(value: u32, start: u32, len: u32, bits: u32) -> u32 {
    let mask = ((1 << len) - 1) << start;
    (value & !mask) | ((bits << start) & mask)
}

/// Check if a bit is set.
#[inline]
pub const fn bit_is_set(value: u32, bit: u32) -> bool {
    (value & (1 << bit)) != 0
}

/// Set a bit.
#[inline]
pub const fn set_bit(value: u32, bit: u32) -> u32 {
    value | (1 << bit)
}

/// Clear a bit.
#[inline]
pub const fn clear_bit(value: u32, bit: u32) -> u32 {
    value & !(1 << bit)
}

/// Toggle a bit.
#[inline]
pub const fn toggle_bit(value: u32, bit: u32) -> u32 {
    value ^ (1 << bit)
}

/// Sign extend a value.
#[inline]
pub const fn sign_extend(value: u32, bits: u32) -> i32 {
    let sign_bit = 1 << (bits - 1);
    if value & sign_bit != 0 {
        (value | !((1 << bits) - 1)) as i32
    } else {
        value as i32
    }
}

/// Sign extend a 16-bit value to 32-bit.
#[inline]
pub const fn sign_extend_16(value: u16) -> i32 {
    sign_extend(value as u32, 16)
}

/// Sign extend an 8-bit value to 32-bit.
#[inline]
pub const fn sign_extend_8(value: u8) -> i32 {
    sign_extend(value as u32, 8)
}

/// Rotate right.
#[inline]
pub const fn ror(value: u32, shift: u32) -> u32 {
    let shift = shift & 31;
    (value >> shift) | (value << (32 - shift))
}

/// Rotate left.
#[inline]
pub const fn rol(value: u32, shift: u32) -> u32 {
    let shift = shift & 31;
    (value << shift) | (value >> (32 - shift))
}

/// Count leading zeros.
#[inline]
pub fn clz(value: u32) -> u32 {
    value.leading_zeros()
}

/// Count trailing zeros.
#[inline]
pub fn ctz(value: u32) -> u32 {
    value.trailing_zeros()
}

/// Population count (count set bits).
#[inline]
pub fn popcount(value: u32) -> u32 {
    value.count_ones()
}

/// Reverse bits.
#[inline]
pub fn reverse_bits(value: u32) -> u32 {
    value.reverse_bits()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bits() {
        assert_eq!(extract_bits(0b11110000, 4, 4), 0b1111);
        assert_eq!(extract_bits(0b11110000, 0, 4), 0b0000);
        assert_eq!(extract_bits(0b10101010, 1, 3), 0b101);
    }

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b0111, 4), 7);
        assert_eq!(sign_extend(0b1111, 4), -1);
        assert_eq!(sign_extend(0b1000, 4), -8);
    }

    #[test]
    fn test_bit_operations() {
        assert!(bit_is_set(0b1000, 3));
        assert!(!bit_is_set(0b1000, 2));
        assert_eq!(set_bit(0b0000, 2), 0b0100);
        assert_eq!(clear_bit(0b0100, 2), 0b0000);
        assert_eq!(toggle_bit(0b0100, 2), 0b0000);
        assert_eq!(toggle_bit(0b0000, 2), 0b0100);
    }
}