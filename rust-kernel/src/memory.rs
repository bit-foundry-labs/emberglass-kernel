//! Fixed-capacity, no_std frame accounting; does not dereference physical memory.
pub const PAGE_BYTES: u64 = 4096;
pub const MIN_PHYSICAL_ADDRESS: u64 = 0x0010_0000;
pub const USABLE_KIND: u64 = 0;
pub const MAX_BOOT_REGIONS: usize = 128;
pub const REQUIRED_BOOT_PAGES: u64 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub base: u64,
    pub length: u64,
    pub kind: u64,
}

impl Region {
    pub const EMPTY: Self = Self {
        base: 0,
        length: 0,
        kind: 0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapError {
    EmptyMap,
    ZeroLength,
    AddressOverflow,
    OverlappingRegions,
    InvalidMinimum,
    NoUsableSpan,
}

/// Monotonic bookkeeping for one BSP; not hardware ownership or a page table.
#[derive(Debug, PartialEq, Eq)]
pub struct FramePool {
    next: u64,
    end: u64,
}

impl FramePool {
    pub fn claim(&mut self) -> Option<u64> {
        let next = self.next.checked_add(PAGE_BYTES)?;
        if next > self.end {
            return None;
        }
        let physical_address = self.next;
        self.next = next;
        Some(physical_address)
    }

    pub const fn remaining(&self) -> u64 {
        (self.end - self.next) / PAGE_BYTES
    }
}

fn page_align_up(address: u64) -> Option<u64> {
    address.checked_add(PAGE_BYTES - 1).map(|value| value & !(PAGE_BYTES - 1))
}

/// Check the entire boot memory map, including reserved regions, before any
/// claim. Fail closed on overlap or overflow rather than repairing ambiguity.
pub fn choose_pool(regions: &[Region], min_pages: u64) -> Result<FramePool, MapError> {
    if regions.is_empty() {
        return Err(MapError::EmptyMap);
    }
    if min_pages == 0 {
        return Err(MapError::InvalidMinimum);
    }

    let mut choice: Option<FramePool> = None;
    for (index, region) in regions.iter().enumerate() {
        if region.length == 0 {
            return Err(MapError::ZeroLength);
        }
        let end = region.base.checked_add(region.length).ok_or(MapError::AddressOverflow)?;
        for earlier in &regions[..index] {
            let earlier_end = earlier.base.checked_add(earlier.length).ok_or(MapError::AddressOverflow)?;
            if region.base < earlier_end && earlier.base < end {
                return Err(MapError::OverlappingRegions);
            }
        }

        if region.kind != USABLE_KIND {
            continue;
        }
        let first = region.base.max(MIN_PHYSICAL_ADDRESS);
        let start = page_align_up(first).ok_or(MapError::AddressOverflow)?;
        let aligned_end = end & !(PAGE_BYTES - 1);
        if start >= aligned_end || (aligned_end - start) / PAGE_BYTES < min_pages {
            continue;
        }
        if choice.as_ref().is_none_or(|best| start < best.next) {
            choice = Some(FramePool { next: start, end: aligned_end });
        }
    }
    choice.ok_or(MapError::NoUsableSpan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(base: u64, length: u64, kind: u64) -> Region {
        Region { base, length, kind }
    }

    #[test]
    fn empty_map_and_zero_minimum_fail() {
        assert_eq!(choose_pool(&[], 1), Err(MapError::EmptyMap));
        assert_eq!(choose_pool(&[region(0x100000, PAGE_BYTES, 0)], 0), Err(MapError::InvalidMinimum));
    }

    #[test]
    fn deterministic_selection_independent_of_entry_order() {
        let low = region(0x100000, 64 * PAGE_BYTES, 0);
        let high = region(0x400000, 32 * PAGE_BYTES, 0);
        let reserved = region(0x300000, PAGE_BYTES, 2);
        for map in [[high, reserved, low], [low, high, reserved]] {
            assert_eq!(choose_pool(&map, 16).unwrap().claim(), Some(0x100000));
        }
    }

    #[test]
    fn reserved_or_low_memory_is_never_claimed() {
        let map = [
            region(0, 0xa0000, 0),
            region(0x100000, 64 * PAGE_BYTES, 1),
            region(0x200000, 32 * PAGE_BYTES, 0),
        ];
        assert_eq!(choose_pool(&map, 16).unwrap().claim(), Some(0x200000));
    }

    #[test]
    fn alignment_rounds_inward() {
        let map = [region(0x100003, 17 * PAGE_BYTES + 10, 0)];
        let mut pool = choose_pool(&map, 16).unwrap();
        assert_eq!(pool.claim(), Some(0x101000));
        assert_eq!(pool.remaining(), 15);
    }

    #[test]
    fn zero_length_and_address_overflow_fail() {
        assert_eq!(choose_pool(&[region(0x100000, 0, 0)], 1), Err(MapError::ZeroLength));
        assert_eq!(choose_pool(&[region(u64::MAX - 5, 10, 2)], 1), Err(MapError::AddressOverflow));
    }

    #[test]
    fn reserved_overlap_rejected_not_silently_skipped() {
        let map = [
            region(0x100000, 64 * PAGE_BYTES, 0),
            region(0x102000, PAGE_BYTES, 1),
        ];
        assert_eq!(choose_pool(&map, 16), Err(MapError::OverlappingRegions));
    }

    #[test]
    fn adjacent_regions_are_not_overlapping() {
        let map = [
            region(0x100000, 16 * PAGE_BYTES, 0),
            region(0x110000, PAGE_BYTES, 1),
        ];
        assert!(choose_pool(&map, 16).is_ok());
    }

    #[test]
    fn too_short_or_entirely_reserved_fails() {
        assert_eq!(choose_pool(&[region(0x100000, 15 * PAGE_BYTES, 0)], 16), Err(MapError::NoUsableSpan));
        assert_eq!(choose_pool(&[region(0x100000, 100 * PAGE_BYTES, 2)], 16), Err(MapError::NoUsableSpan));
    }

    #[test]
    fn claimed_frames_are_unique_and_exhaustion_does_not_change_state() {
        let mut pool = choose_pool(&[region(0x100000, 16 * PAGE_BYTES, 0)], 16).unwrap();
        for index in 0..16 {
            assert_eq!(pool.claim(), Some(0x100000 + index * PAGE_BYTES));
        }
        assert_eq!(pool.remaining(), 0);
        assert_eq!(pool.claim(), None);
        assert_eq!(pool.claim(), None);
    }

    #[test]
    fn near_u64_max_does_not_wrap() {
        assert_eq!(choose_pool(&[region(u64::MAX - 4095, 4095, 0)], 1), Err(MapError::NoUsableSpan));
    }
}
