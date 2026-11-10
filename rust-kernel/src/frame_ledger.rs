//! Fixed-capacity, single-writer physical-frame ownership *bookkeeping*.
//!
//! This is not a capability system, a page-table manager or proof of DMA
//! quiescence. Once quarantined, a frame can NEVER be recycled by this type.
//! The trusted kernel must separately enforce CPU/IOMMU mapping permissions.
use crate::memory::FramePool;

pub const MAX_LEDGER_FRAMES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerId(u32);

impl OwnerId {
    /// Zero is reserved for the absence of an owner.
    pub const fn new(value: u32) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }
}

/// An internal accounting identifier, NOT an unforgeable capability.
/// Its private fields alone do not provide a security boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameToken {
    ledger_id: u64,
    slot: usize,
    generation: u64,
    owner: OwnerId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotState {
    Available,
    Owned(OwnerId),
    Quarantined,
}

#[derive(Clone, Copy, Debug)]
struct Slot {
    physical: u64,
    generation: u64,
    state: SlotState,
}

impl Slot {
    const EMPTY: Self = Self {
        physical: 0,
        generation: 0,
        state: SlotState::Available,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerError {
    InvalidCapacity,
    InsufficientFrames,
    Exhausted,
    InvalidToken,
    WrongOwner,
    Quarantined,
    GenerationExhausted,
}

/// A fixed array of *distinct* frames claimed from M04's monotonic pool.
///
/// One mutable owner at a time, no concurrency, no dynamic allocation, and
/// no physical frame reads or writes. Neither FramePool nor FrameToken can
/// establish kernel/user memory isolation.
pub struct FrameLedger<const N: usize> {
    // The first claimed frame identifies the ledger within one unique FramePool.
    ledger_id: u64,
    slots: [Slot; N],
}

impl<const N: usize> FrameLedger<N> {
    /// Claim N frames from the monotonically advancing pool. Check capacity
    /// and remaining frames first, avoiding partial consumption on failures.
    /// Identity is distinct only when the source pool never reissues frames.
    pub fn from_pool(pool: &mut FramePool) -> Result<Self, LedgerError> {
        if N == 0 || N > MAX_LEDGER_FRAMES {
            return Err(LedgerError::InvalidCapacity);
        }
        if pool.remaining() < N as u64 {
            return Err(LedgerError::InsufficientFrames);
        }

        let mut slots = [Slot::EMPTY; N];
        for slot in &mut slots {
            // Remaining capacity was checked above; pool is single-writer.
            slot.physical = pool.claim().ok_or(LedgerError::InsufficientFrames)?;
        }
        // N is nonzero, and all frames originate from one monotonic pool.
        // This identity is NOT globally unique across overlapping FramePools.
        let ledger_id = slots[0].physical;
        Ok(Self { ledger_id, slots })
    }

    /// Assign the lowest available slot deterministically to an owner.
    pub fn acquire(&mut self, owner: OwnerId) -> Result<FrameToken, LedgerError> {
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if slot.state == SlotState::Available {
                let generation = slot
                    .generation
                    .checked_add(1)
                    .ok_or(LedgerError::GenerationExhausted)?;
                slot.generation = generation;
                slot.state = SlotState::Owned(owner);
                return Ok(FrameToken {
                    ledger_id: self.ledger_id,
                    slot: index,
                    generation,
                    owner,
                });
            }
        }
        Err(LedgerError::Exhausted)
    }

    /// Read the physical *number* only if the caller matches the active owner.
    /// This does not map or dereference physical memory.
    pub fn physical_address(&self, token: FrameToken, caller: OwnerId) -> Result<u64, LedgerError> {
        let slot = self.require_owned(token, caller)?;
        Ok(slot.physical)
    }

    /// Logically withdraw the owner's frame from future claims. A quarantined
    /// frame stays unavailable forever in M05, even if no CPU work remains.
    pub fn quarantine(&mut self, token: FrameToken, caller: OwnerId) -> Result<(), LedgerError> {
        self.require_owned(token, caller)?;
        self.slots[token.slot].state = SlotState::Quarantined;
        Ok(())
    }

    pub fn available_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.state == SlotState::Available)
            .count()
    }

    pub fn quarantined_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.state == SlotState::Quarantined)
            .count()
    }

    fn require_owned(&self, token: FrameToken, caller: OwnerId) -> Result<&Slot, LedgerError> {
        // A slot, generation, and owner can coincide in separate ledgers.
        // A token from another ledger must never authorize this ledger.
        if token.ledger_id != self.ledger_id {
            return Err(LedgerError::InvalidToken);
        }
        if caller != token.owner {
            return Err(LedgerError::WrongOwner);
        }
        let slot = self
            .slots
            .get(token.slot)
            .ok_or(LedgerError::InvalidToken)?;
        if slot.generation != token.generation {
            return Err(LedgerError::InvalidToken);
        }
        match slot.state {
            SlotState::Owned(owner) if owner == caller => Ok(slot),
            SlotState::Owned(_) => Err(LedgerError::WrongOwner),
            SlotState::Quarantined => Err(LedgerError::Quarantined),
            SlotState::Available => Err(LedgerError::InvalidToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{choose_pool, Region, PAGE_BYTES, USABLE_KIND};

    fn pool(pages: u64) -> FramePool {
        choose_pool(
            &[Region {
                base: 0x100000,
                length: PAGE_BYTES * pages,
                kind: USABLE_KIND,
            }],
            1,
        )
        .unwrap()
    }

    fn owner(value: u32) -> OwnerId {
        OwnerId::new(value).unwrap()
    }

    #[test]
    fn rejects_zero_and_oversized_capacity_without_claims() {
        let mut frames = pool(64);
        assert!(OwnerId::new(0).is_none());
        assert!(matches!(
            FrameLedger::<0>::from_pool(&mut frames),
            Err(LedgerError::InvalidCapacity)
        ));
        assert!(matches!(
            FrameLedger::<33>::from_pool(&mut frames),
            Err(LedgerError::InvalidCapacity)
        ));
        assert_eq!(frames.remaining(), 64);
    }

    #[test]
    fn capacity_shortage_does_not_consume_frames() {
        let mut frames = pool(2);
        assert!(matches!(
            FrameLedger::<3>::from_pool(&mut frames),
            Err(LedgerError::InsufficientFrames)
        ));
        assert_eq!(frames.remaining(), 2);
        assert_eq!(frames.claim(), Some(0x100000));
    }

    #[test]
    fn frames_are_unique_aligned_and_assigned_in_slot_order() {
        let mut frames = pool(4);
        let mut ledger = FrameLedger::<3>::from_pool(&mut frames).unwrap();
        let a = ledger.acquire(owner(1)).unwrap();
        let b = ledger.acquire(owner(2)).unwrap();
        let c = ledger.acquire(owner(1)).unwrap();
        assert_eq!(ledger.physical_address(a, owner(1)), Ok(0x100000));
        assert_eq!(ledger.physical_address(b, owner(2)), Ok(0x101000));
        assert_eq!(ledger.physical_address(c, owner(1)), Ok(0x102000));
        assert_eq!(frames.remaining(), 1);
        assert_eq!(ledger.available_count(), 0);
        assert_eq!(ledger.acquire(owner(2)), Err(LedgerError::Exhausted));
    }

    #[test]
    fn wrong_owner_cannot_observe_or_quarantine_a_frame() {
        let mut frames = pool(2);
        let mut ledger = FrameLedger::<2>::from_pool(&mut frames).unwrap();
        let token = ledger.acquire(owner(1)).unwrap();
        assert_eq!(
            ledger.physical_address(token, owner(2)),
            Err(LedgerError::WrongOwner)
        );
        assert_eq!(
            ledger.quarantine(token, owner(2)),
            Err(LedgerError::WrongOwner)
        );
        assert_eq!(ledger.physical_address(token, owner(1)), Ok(0x100000));
        assert_eq!(ledger.quarantined_count(), 0);
    }

    #[test]
    fn quarantined_frames_are_never_reallocated() {
        let mut frames = pool(2);
        let mut ledger = FrameLedger::<2>::from_pool(&mut frames).unwrap();
        let first = ledger.acquire(owner(1)).unwrap();
        ledger.quarantine(first, owner(1)).unwrap();
        assert_eq!(
            ledger.physical_address(first, owner(1)),
            Err(LedgerError::Quarantined)
        );
        assert_eq!(
            ledger.quarantine(first, owner(1)),
            Err(LedgerError::Quarantined)
        );
        let second = ledger.acquire(owner(2)).unwrap();
        assert_eq!(ledger.physical_address(second, owner(2)), Ok(0x101000));
        assert_eq!(ledger.acquire(owner(1)), Err(LedgerError::Exhausted));
        assert_eq!(ledger.available_count(), 0);
        assert_eq!(ledger.quarantined_count(), 1);
    }

    #[test]
    fn fabricated_indices_and_generations_are_rejected() {
        let mut frames = pool(2);
        let mut ledger = FrameLedger::<2>::from_pool(&mut frames).unwrap();
        let valid = ledger.acquire(owner(1)).unwrap();
        assert_eq!(
            ledger.physical_address(FrameToken { slot: 4, ..valid }, owner(1)),
            Err(LedgerError::InvalidToken)
        );
        assert_eq!(
            ledger.physical_address(
                FrameToken {
                    generation: valid.generation + 1,
                    ..valid
                },
                owner(1),
            ),
            Err(LedgerError::InvalidToken)
        );
        assert_eq!(
            ledger.physical_address(
                FrameToken {
                    owner: owner(2),
                    ..valid
                },
                owner(2),
            ),
            Err(LedgerError::WrongOwner)
        );
    }

    #[test]
    fn distinct_ledgers_from_one_pool_cannot_reissue_the_same_frame() {
        let mut frames = pool(4);
        let mut first = FrameLedger::<2>::from_pool(&mut frames).unwrap();
        let mut second = FrameLedger::<2>::from_pool(&mut frames).unwrap();
        let a = first.acquire(owner(1)).unwrap();
        let b = second.acquire(owner(1)).unwrap();
        assert_eq!(first.physical_address(a, owner(1)), Ok(0x100000));
        assert_eq!(second.physical_address(b, owner(1)), Ok(0x102000));
        assert_eq!(frames.remaining(), 0);
    }

    #[test]
    fn token_cannot_cross_ledgers_even_if_slot_generation_and_owner_match() {
        let mut frames = pool(4);
        let mut first = FrameLedger::<2>::from_pool(&mut frames).unwrap();
        let mut second = FrameLedger::<2>::from_pool(&mut frames).unwrap();
        let token_a = first.acquire(owner(1)).unwrap();
        let token_b = second.acquire(owner(1)).unwrap();

        assert_eq!(token_a.slot, token_b.slot);
        assert_eq!(token_a.generation, token_b.generation);
        assert_eq!(token_a.owner, token_b.owner);
        assert_ne!(token_a.ledger_id, token_b.ledger_id);

        assert_eq!(
            second.physical_address(token_a, owner(1)),
            Err(LedgerError::InvalidToken)
        );
        assert_eq!(
            first.quarantine(token_b, owner(1)),
            Err(LedgerError::InvalidToken)
        );
        assert_eq!(first.quarantined_count(), 0);
        assert_eq!(second.quarantined_count(), 0);
        assert_eq!(first.physical_address(token_a, owner(1)), Ok(0x100000));
        assert_eq!(second.physical_address(token_b, owner(1)), Ok(0x102000));
    }

    #[test]
    fn invalid_ledger_identity_does_not_mutate_owned_frame() {
        let mut frames = pool(2);
        let mut ledger = FrameLedger::<2>::from_pool(&mut frames).unwrap();
        let valid = ledger.acquire(owner(7)).unwrap();
        let forged = FrameToken {
            ledger_id: valid.ledger_id + PAGE_BYTES,
            ..valid
        };
        assert_eq!(
            ledger.quarantine(forged, owner(7)),
            Err(LedgerError::InvalidToken)
        );
        assert_eq!(ledger.physical_address(valid, owner(7)), Ok(0x100000));
    }

    #[test]
    fn exhausted_generation_never_wraps_or_reassigns() {
        let mut frames = pool(1);
        let mut ledger = FrameLedger::<1>::from_pool(&mut frames).unwrap();
        ledger.slots[0].generation = u64::MAX;
        assert_eq!(
            ledger.acquire(owner(1)),
            Err(LedgerError::GenerationExhausted)
        );
        assert_eq!(ledger.available_count(), 1);
    }
}
