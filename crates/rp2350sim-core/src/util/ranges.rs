//! Range utilities for memory regions.

use serde::{Deserialize, Serialize};

/// A memory range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryRange {
    pub start: u32,
    pub end: u32,
}

impl MemoryRange {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub const fn from_base_size(base: u32, size: u32) -> Self {
        Self {
            start: base,
            end: base + size - 1,
        }
    }

    pub const fn contains(&self, addr: u32) -> bool {
        addr >= self.start && addr <= self.end
    }

    pub const fn size(&self) -> u32 {
        self.end - self.start + 1
    }

    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    pub const fn is_before(&self, addr: u32) -> bool {
        addr < self.start
    }

    pub const fn is_after(&self, addr: u32) -> bool {
        addr > self.end
    }

    pub const fn offset(&self, addr: u32) -> Option<u32> {
        if self.contains(addr) {
            Some(addr - self.start)
        } else {
            None
        }
    }
}

/// A collection of non-overlapping memory ranges.
#[derive(Debug, Clone, Default)]
pub struct RangeSet {
    ranges: Vec<MemoryRange>,
}

impl RangeSet {
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    pub fn insert(&mut self, range: MemoryRange) -> bool {
        // Check for overlaps
        for existing in &self.ranges {
            if existing.overlaps(&range) {
                return false;
            }
        }

        // Find insertion point
        let pos = self.ranges.iter().position(|r| r.start > range.start).unwrap_or(self.ranges.len());
        self.ranges.insert(pos, range);
        true
    }

    pub fn remove(&mut self, range: MemoryRange) -> bool {
        let pos = self.ranges.iter().position(|r| *r == range);
        if let Some(pos) = pos {
            self.ranges.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, addr: u32) -> bool {
        self.ranges.iter().any(|r| r.contains(addr))
    }

    pub fn find(&self, addr: u32) -> Option<&MemoryRange> {
        self.ranges.iter().find(|r| r.contains(addr))
    }

    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn clear(&mut self) {
        self.ranges.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &MemoryRange> {
        self.ranges.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_range() {
        let range = MemoryRange::from_base_size(0x1000, 0x100);
        assert_eq!(range.start, 0x1000);
        assert_eq!(range.end, 0x10FF);
        assert_eq!(range.size(), 0x100);

        assert!(range.contains(0x1000));
        assert!(range.contains(0x1050));
        assert!(range.contains(0x10FF));
        assert!(!range.contains(0x0FFF));
        assert!(!range.contains(0x1100));

        assert_eq!(range.offset(0x1050), Some(0x50));
        assert_eq!(range.offset(0x2000), None);
    }

    #[test]
    fn test_range_overlaps() {
        let r1 = MemoryRange::from_base_size(0x1000, 0x100);
        let r2 = MemoryRange::from_base_size(0x1050, 0x100);
        let r3 = MemoryRange::from_base_size(0x2000, 0x100);

        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }
}