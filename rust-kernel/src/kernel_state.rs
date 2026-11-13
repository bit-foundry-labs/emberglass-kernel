//! Bootstrap-owned kernel state. This is not global physical-memory authority.
use crate::frame_registry::{FrameReservation, TransactionError, TransactionalFrameAllocator};

pub struct KernelState<const N: usize> {
    frames: TransactionalFrameAllocator<N>,
}

impl<const N: usize> KernelState<N> {
    pub fn new(frames: TransactionalFrameAllocator<N>) -> Self {
        Self { frames }
    }

    pub fn reservation(&self, index: usize) -> Option<FrameReservation> {
        self.frames.reservation(index)
    }

    #[allow(dead_code)]
    pub fn reserve(&mut self, pages: u64, owner: u32) -> Result<FrameReservation, TransactionError> {
        self.frames.reserve(pages, owner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{choose_pool, Region, MIN_PHYSICAL_ADDRESS, PAGE_BYTES, USABLE_KIND};

    fn initialize() -> KernelState<2> {
        let pool = choose_pool(
            &[Region {
                base: MIN_PHYSICAL_ADDRESS,
                length: 4 * PAGE_BYTES,
                kind: USABLE_KIND,
            }],
            1,
        )
        .unwrap();
        KernelState::new(TransactionalFrameAllocator::with_boot_reservation(pool).unwrap())
    }

    #[test]
    fn moved_state_retains_boot_record_and_allocates_disjoint_next_run() {
        let mut state = initialize();
        let first = state.reservation(0).unwrap();
        let next = state.reserve(1, 2).unwrap();
        assert_eq!(next.start, first.start + 2 * PAGE_BYTES);
        assert_eq!(state.reservation(0), Some(first));
        assert_eq!(state.reservation(1), Some(next));
    }
}
