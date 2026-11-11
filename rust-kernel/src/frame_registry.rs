//! M06d: bounded, single-writer reservation bookkeeping for physical ranges.
//! This module does not authenticate callers or establish mapping authority.
use crate::memory::{MIN_PHYSICAL_ADDRESS, PAGE_BYTES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameReservation {
    pub start: u64,
    pub pages: u64,
    pub owner: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError {
    ZeroPages,
    InvalidOwner,
    InvalidStart,
    AddressOverflow,
    Overlap,
    CapacityExhausted,
}

/// A single-writer, fixed-capacity registry. Reservations never expire, and
/// there is deliberately no release path until safe reclamation is specified.
pub struct FrameRegistry<const N: usize> {
    entries: [Option<FrameReservation>; N],
    count: usize,
}

impl<const N: usize> FrameRegistry<N> {
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            count: 0,
        }
    }

    pub const fn count(&self) -> usize {
        self.count
    }

    /// Record a trusted physical run without touching actual physical memory.
    /// Any rejection leaves all registry entries unchanged.
    ///
    /// Caller must independently establish that the range is USABLE, uniquely
    /// sourced, and not accessible through conflicting CPU or device mappings.
    pub fn reserve(
        &mut self,
        start: u64,
        pages: u64,
        owner: u32,
    ) -> Result<FrameReservation, RegistryError> {
        if owner == 0 {
            return Err(RegistryError::InvalidOwner);
        }
        if pages == 0 {
            return Err(RegistryError::ZeroPages);
        }
        if start < MIN_PHYSICAL_ADDRESS || start % PAGE_BYTES != 0 {
            return Err(RegistryError::InvalidStart);
        }
        let bytes = pages
            .checked_mul(PAGE_BYTES)
            .ok_or(RegistryError::AddressOverflow)?;
        let end = start
            .checked_add(bytes)
            .ok_or(RegistryError::AddressOverflow)?;

        for existing in self.entries[..self.count].iter().flatten() {
            let existing_end = existing.start + existing.pages * PAGE_BYTES;
            if start < existing_end && existing.start < end {
                return Err(RegistryError::Overlap);
            }
        }
        if self.count == N {
            return Err(RegistryError::CapacityExhausted);
        }

        let reservation = FrameReservation {
            start,
            pages,
            owner,
        };
        self.entries[self.count] = Some(reservation);
        self.count += 1;
        Ok(reservation)
    }

    pub fn entry(&self, index: usize) -> Option<FrameReservation> {
        self.entries.get(index).copied().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_ranges_and_distinct_owners_are_allowed() {
        let mut registry = FrameRegistry::<2>::new();
        assert_eq!(registry.reserve(0x100000, 2, 1).unwrap().start, 0x100000);
        assert_eq!(registry.reserve(0x102000, 1, 2).unwrap().owner, 2);
        assert_eq!(registry.count(), 2);
    }

    #[test]
    fn overlaps_are_rejected_across_owners_and_same_owner() {
        let mut registry = FrameRegistry::<4>::new();
        registry.reserve(0x101000, 4, 1).unwrap();
        for (start, pages, owner) in [
            (0x100000, 2, 2),
            (0x102000, 1, 1),
            (0x104000, 2, 2),
            (0x101000, 4, 1),
        ] {
            assert_eq!(
                registry.reserve(start, pages, owner),
                Err(RegistryError::Overlap)
            );
        }
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn invalid_inputs_never_mutate_state() {
        let mut registry = FrameRegistry::<2>::new();
        for (start, pages, owner, expected) in [
            (0x100000, 1, 0, RegistryError::InvalidOwner),
            (0x100000, 0, 1, RegistryError::ZeroPages),
            (0x100001, 1, 1, RegistryError::InvalidStart),
            (0x0, 1, 1, RegistryError::InvalidStart),
            (0x100000, u64::MAX, 1, RegistryError::AddressOverflow),
            (0xffff_ffff_ffff_f000, 2, 1, RegistryError::AddressOverflow),
        ] {
            assert_eq!(registry.reserve(start, pages, owner), Err(expected));
        }
        assert_eq!(registry.count(), 0);
        assert_eq!(registry.entry(0), None);
    }

    #[test]
    fn capacity_exhaustion_does_not_consume_a_range() {
        let mut registry = FrameRegistry::<1>::new();
        registry.reserve(0x100000, 1, 1).unwrap();
        assert_eq!(
            registry.reserve(0x200000, 1, 2),
            Err(RegistryError::CapacityExhausted)
        );
        assert_eq!(registry.entry(0).unwrap().start, 0x100000);
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn zero_capacity_fails_closed() {
        let mut registry = FrameRegistry::<0>::new();
        assert_eq!(
            registry.reserve(0x100000, 1, 1),
            Err(RegistryError::CapacityExhausted)
        );
    }

    #[test]
    fn reservation_records_persist_without_reuse() {
        let mut registry = FrameRegistry::<2>::new();
        let snapshot = registry.reserve(0x100000, 1, 7).unwrap();
        assert_eq!(snapshot, registry.entry(0).unwrap());
        assert_eq!(
            registry.reserve(0x100000, 1, 8),
            Err(RegistryError::Overlap)
        );
        assert_eq!(registry.count(), 1);
        assert_eq!(registry.entry(2), None);
    }
}
