//! Comparison utilities.

use crate::DiffResult;

/// Compare two values and return the difference.
pub trait Compare {
    /// Compare self with another value.
    fn compare(&self, other: &Self) -> DiffResult;
}

impl Compare for u8 {
    fn compare(&self, other: &Self) -> DiffResult {
        if self == other {
            DiffResult::Match
        } else {
            DiffResult::Mismatch {
                expected: *self as u64,
                actual: *other as u64,
            }
        }
    }
}

impl Compare for u16 {
    fn compare(&self, other: &Self) -> DiffResult {
        if self == other {
            DiffResult::Match
        } else {
            DiffResult::Mismatch {
                expected: *self as u64,
                actual: *other as u64,
            }
        }
    }
}

impl Compare for u32 {
    fn compare(&self, other: &Self) -> DiffResult {
        if self == other {
            DiffResult::Match
        } else {
            DiffResult::Mismatch {
                expected: *self as u64,
                actual: *other as u64,
            }
        }
    }
}

impl Compare for u64 {
    fn compare(&self, other: &Self) -> DiffResult {
        if self == other {
            DiffResult::Match
        } else {
            DiffResult::Mismatch {
                expected: *self,
                actual: *other,
            }
        }
    }
}

impl<T: Compare> Compare for [T] {
    fn compare(&self, other: &Self) -> DiffResult {
        if self.len() != other.len() {
            return DiffResult::LengthMismatch {
                expected: self.len(),
                actual: other.len(),
            };
        }
        
        for (i, (a, b)) in self.iter().zip(other.iter()).enumerate() {
            match a.compare(b) {
                DiffResult::Match => continue,
                result => return DiffResult::ElementMismatch { index: i, inner: Box::new(result) },
            }
        }
        
        DiffResult::Match
    }
}