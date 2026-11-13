//! Host-only state model. Completion and quiescence are simulated inputs.
//! This module does not implement DMA, device reset, or memory isolation.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Active,
    Draining,
    Quarantined,
    Reusable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ticket {
    generation: u64,
    sequence: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Closed,
    Busy,
    Stale,
    NotDraining,
    NotQuiescent,
    Exhausted,
}

/// One buffer and at most one admitted operation, under a single trusted owner.
/// An abstract device-access flag is evidence in the model, not hardware proof.
pub struct Buffer {
    state: State,
    generation: u64,
    next_sequence: u64,
    inflight: Option<Ticket>,
    device_may_access: bool,
    synced_for_cpu: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            state: State::Active,
            generation: 1,
            next_sequence: 0,
            inflight: None,
            device_may_access: false,
            synced_for_cpu: true,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn submit(&mut self) -> Result<Ticket, Error> {
        if self.state != State::Active {
            return Err(Error::Closed);
        }
        if self.inflight.is_some() {
            return Err(Error::Busy);
        }
        let next = self.next_sequence.checked_add(1).ok_or(Error::Exhausted)?;
        let ticket = Ticket {
            generation: self.generation,
            sequence: next,
        };
        self.next_sequence = next;
        self.inflight = Some(ticket);
        self.device_may_access = true;
        self.synced_for_cpu = false;
        Ok(ticket)
    }

    pub fn revoke(&mut self) -> Result<(), Error> {
        if self.state != State::Active {
            return Err(Error::Closed);
        }
        self.state = State::Draining;
        Ok(())
    }

    /// Assume a trusted adapter has validated the completion for this ticket.
    pub fn complete(&mut self, ticket: Ticket) -> Result<(), Error> {
        if self.inflight != Some(ticket) {
            return Err(Error::Stale);
        }
        self.inflight = None;
        self.device_may_access = false;
        Ok(())
    }

    /// Abstract synchronization event; real requirements are device-specific.
    pub fn sync_for_cpu(&mut self) -> Result<(), Error> {
        if self.device_may_access {
            return Err(Error::NotQuiescent);
        }
        self.synced_for_cpu = true;
        Ok(())
    }

    pub fn quarantine(&mut self) -> Result<(), Error> {
        if self.state != State::Draining {
            return Err(Error::NotDraining);
        }
        self.state = State::Quarantined;
        Ok(())
    }

    /// This model has no reset or mapping-unmap implementation.
    /// Quarantine is terminal; reclamation is possible only from draining.
    pub fn reclaim(&mut self) -> Result<(), Error> {
        if self.state != State::Draining {
            return Err(Error::NotDraining);
        }
        if self.inflight.is_some() || self.device_may_access || !self.synced_for_cpu {
            return Err(Error::NotQuiescent);
        }
        self.state = State::Reusable;
        Ok(())
    }

    pub fn reissue(&mut self) -> Result<(), Error> {
        if self.state != State::Reusable {
            return Err(Error::NotQuiescent);
        }
        let next = self.generation.checked_add(1).ok_or(Error::Exhausted)?;
        self.generation = next;
        self.next_sequence = 0;
        self.state = State::Active;
        Ok(())
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoke_drains_then_requires_sync_before_reclaim() {
        let mut b = Buffer::new();
        let t = b.submit().unwrap();
        b.revoke().unwrap();
        assert_eq!(b.submit(), Err(Error::Closed));
        assert_eq!(b.reclaim(), Err(Error::NotQuiescent));
        assert_eq!(b.sync_for_cpu(), Err(Error::NotQuiescent));
        b.complete(t).unwrap();
        assert_eq!(b.reclaim(), Err(Error::NotQuiescent));
        b.sync_for_cpu().unwrap();
        b.reclaim().unwrap();
        assert_eq!(b.state(), State::Reusable);
    }

    #[test]
    fn stale_completion_cannot_release_a_new_operation() {
        let mut b = Buffer::new();
        let old = b.submit().unwrap();
        b.revoke().unwrap();
        b.complete(old).unwrap();
        b.sync_for_cpu().unwrap();
        b.reclaim().unwrap();
        b.reissue().unwrap();
        let current = b.submit().unwrap();
        assert_eq!(b.complete(old), Err(Error::Stale));
        b.revoke().unwrap();
        assert_eq!(b.reclaim(), Err(Error::NotQuiescent));
        b.complete(current).unwrap();
        b.sync_for_cpu().unwrap();
        assert_eq!(b.reclaim(), Ok(()));
    }

    #[test]
    fn hung_device_remains_quarantined() {
        let mut b = Buffer::new();
        b.submit().unwrap();
        b.revoke().unwrap();
        b.quarantine().unwrap();
        assert_eq!(b.reclaim(), Err(Error::NotDraining));
        assert_eq!(b.reissue(), Err(Error::NotQuiescent));
        assert_eq!(b.state(), State::Quarantined);
    }

    #[test]
    fn immediate_reuse_after_revoke_has_a_counterexample() {
        let mut b = Buffer::new();
        b.submit().unwrap();
        b.revoke().unwrap();
        // A hypothetical implementation that reuses immediately is unsafe:
        // the simulated device may still access the buffer.
        let flawed_reusable = b.state == State::Draining;
        assert!(flawed_reusable && b.device_may_access);
        assert_eq!(b.reclaim(), Err(Error::NotQuiescent));
    }

    #[test]
    fn bounded_event_sequences_never_reuse_with_outstanding_access() {
        // Enumerate all 7^6 event sequences, including rejected actions.
        let mut reached_reusable = false;
        let mut reached_quarantine = false;
        for mut sequence in 0..7usize.pow(6) {
            let mut b = Buffer::new();
            let mut ticket = None;
            for _ in 0..6 {
                match sequence % 7 {
                    0 => {
                        if let Ok(t) = b.submit() {
                            ticket = Some(t);
                        }
                    }
                    1 => {
                        let _ = b.revoke();
                    }
                    2 => {
                        if let Some(t) = ticket {
                            let _ = b.complete(t);
                        }
                    }
                    3 => {
                        let _ = b.sync_for_cpu();
                    }
                    4 => {
                        let _ = b.reclaim();
                    }
                    5 => {
                        let _ = b.reissue();
                    }
                    _ => {
                        let _ = b.quarantine();
                    }
                }
                sequence /= 7;
                if b.state == State::Reusable {
                    reached_reusable = true;
                    assert!(!b.device_may_access);
                    assert!(b.inflight.is_none());
                    assert!(b.synced_for_cpu);
                }
                reached_quarantine |= b.state == State::Quarantined;
            }
        }
        assert!(reached_reusable);
        assert!(reached_quarantine);
    }

    #[test]
    fn generation_exhaustion_fails_closed() {
        let mut b = Buffer::new();
        b.generation = u64::MAX;
        b.revoke().unwrap();
        b.reclaim().unwrap();
        assert_eq!(b.reissue(), Err(Error::Exhausted));
        assert_eq!(b.state(), State::Reusable);
        assert_eq!(b.submit(), Err(Error::Closed));
    }
}
