//! Alignment utilities.

/// Align a value up to the given alignment.
#[inline]
pub const fn align_up(value: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (value + mask) & !mask
}

/// Align a value down to the given alignment.
#[inline]
pub const fn align_down(value: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    value & !mask
}

/// Check if a value is aligned.
#[inline]
pub const fn is_aligned(value: usize, alignment: usize) -> bool {
    (value & (alignment - 1)) == 0
}

/// Align a 32-bit address up.
#[inline]
pub const fn align_up_u32(value: u32, alignment: u32) -> u32 {
    let mask = alignment - 1;
    (value + mask) & !mask
}

/// Align a 32-bit address down.
#[inline]
pub const fn align_down_u32(value: u32, alignment: u32) -> u32 {
    let mask = alignment - 1;
    value & !mask
}

/// Check if a 32-bit address is aligned.
#[inline]
pub const fn is_aligned_u32(value: u32, alignment: u32) -> bool {
    (value & (alignment - 1)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(3, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
    }

    #[test]
    fn test_align_down() {
        assert_eq!(align_down(0, 4), 0);
        assert_eq!(align_down(1, 4), 0);
        assert_eq!(align_down(3, 4), 0);
        assert_eq!(align_down(4, 4), 4);
        assert_eq!(align_down(5, 4), 4);
    }

    #[test]
    fn test_is_aligned() {
        assert!(is_aligned(0, 4));
        assert!(!is_aligned(1, 4));
        assert!(!is_aligned(3, 4));
        assert!(is_aligned(4, 4));
        assert!(!is_aligned(5, 4));
    }
}