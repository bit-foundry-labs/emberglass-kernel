//! BSP boot-time one-shot gate for constructing the initial FramePool.
//! This is not a global allocator or proof of hardware frame ownership.
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateError {
    AlreadyIssued,
}

/// A permanently consumed issuance permit. No reset or release path exists.
pub struct BootPoolGate {
    issued: AtomicBool,
}

impl BootPoolGate {
    pub const fn new() -> Self {
        Self {
            issued: AtomicBool::new(false),
        }
    }

    /// Fail closed; even if subsequent bootloader parsing fails, retrying
    /// cannot manufacture another overlapping pool from the same map.
    pub fn try_issue(&self) -> Result<(), GateError> {
        self.issued
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| GateError::AlreadyIssued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_issuance_succeeds_and_all_later_requests_fail() {
        let gate = BootPoolGate::new();
        assert_eq!(gate.try_issue(), Ok(()));
        assert_eq!(gate.try_issue(), Err(GateError::AlreadyIssued));
        assert_eq!(gate.try_issue(), Err(GateError::AlreadyIssued));
    }

    #[test]
    fn independent_gates_do_not_share_state() {
        let first = BootPoolGate::new();
        let second = BootPoolGate::new();
        assert_eq!(first.try_issue(), Ok(()));
        assert_eq!(second.try_issue(), Ok(()));
        assert_eq!(first.try_issue(), Err(GateError::AlreadyIssued));
    }

    #[test]
    fn concurrent_attempts_have_exactly_one_winner() {
        use std::sync::Arc;
        use std::thread;

        let gate = Arc::new(BootPoolGate::new());
        let mut threads = Vec::new();
        for _ in 0..8 {
            let shared = Arc::clone(&gate);
            threads.push(thread::spawn(move || shared.try_issue()));
        }
        let successes = threads
            .into_iter()
            .filter(|thread| thread.join().unwrap().is_ok())
            .count();
        assert_eq!(successes, 1);
        assert_eq!(gate.try_issue(), Err(GateError::AlreadyIssued));
    }
}
