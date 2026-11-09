#![no_std]
#![forbid(unsafe_code)]
//! Research model, NOT an implemented kernel capability table or scheduler.

pub mod capabilities;

use crate::capabilities::{CapError, CapTable, Handle, SpaceId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tenant(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct Proposal {
    pub tenant: Tenant,
    pub epoch: u64,
    pub weight: u16,
    pub pages: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidatedPolicy {
    tenant: Tenant,
    weight: u16,
    pages: u32,
}
impl ValidatedPolicy {
    pub const fn tenant(self) -> Tenant {
        self.tenant
    }
    pub const fn weight(self) -> u16 {
        self.weight
    }
    pub const fn pages(self) -> u32 {
        self.pages
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    Missing,
    Unauthorized,
    WrongTenant,
    Expired,
    ReplayOrStale,
    InvalidWeight,
    MemoryLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Advisory(ValidatedPolicy),
    Baseline(Rejection),
}

/// An asynchronous advisory-policy gate. Invalid suggestions never change state.
pub struct PolicyGate {
    last_epoch: Option<u64>,
}

impl Default for PolicyGate {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyGate {
    pub const fn new() -> Self {
        Self { last_epoch: None }
    }

    pub fn evaluate<const N: usize>(
        &mut self,
        table: &CapTable<N>,
        caller: SpaceId,
        authority_handle: Handle,
        candidate: Option<Proposal>,
        now: u64,
    ) -> Decision {
        let authority = match table.authorize_update(caller, authority_handle, now) {
            Ok(authority) => authority,
            Err(CapError::Expired) => return Decision::Baseline(Rejection::Expired),
            Err(_) => return Decision::Baseline(Rejection::Unauthorized),
        };
        let limits = authority.limits();
        let Some(p) = candidate else {
            return Decision::Baseline(Rejection::Missing);
        };
        let problem = if p.tenant != authority.tenant() {
            Some(Rejection::WrongTenant)
        } else if p.epoch != now || self.last_epoch.is_some_and(|e| p.epoch <= e) {
            Some(Rejection::ReplayOrStale)
        } else if p.weight == 0 || p.weight > limits.max_weight {
            Some(Rejection::InvalidWeight)
        } else if p.pages > limits.max_pages {
            Some(Rejection::MemoryLimit)
        } else {
            None
        };
        if let Some(error) = problem {
            return Decision::Baseline(error);
        }
        self.last_epoch = Some(p.epoch);
        Decision::Advisory(ValidatedPolicy {
            tenant: p.tenant,
            weight: p.weight,
            pages: p.pages,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinimumCpu {
    pub tenant: Tenant,
    pub ticks: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionError {
    InvalidEpoch,
    InvalidShare,
    Overflow,
    Overbooked,
}

/// Necessary capacity condition in a simplified single-CPU epoch.
/// This does NOT establish fairness or enforce runtime execution.
pub fn admit_minimums(
    epoch: u32,
    reserved_system: u32,
    requests: &[MinimumCpu],
) -> Result<u32, AdmissionError> {
    if epoch == 0 || reserved_system >= epoch {
        return Err(AdmissionError::InvalidEpoch);
    }
    let capacity = epoch - reserved_system;
    let mut total = 0_u32;
    for request in requests {
        if request.ticks == 0 {
            return Err(AdmissionError::InvalidShare);
        }
        total = total
            .checked_add(request.ticks)
            .ok_or(AdmissionError::Overflow)?;
        if total > capacity {
            return Err(AdmissionError::Overbooked);
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::{Limits, Rights};

    const A: Tenant = Tenant(1);
    const B: Tenant = Tenant(2);
    const CALLER: SpaceId = SpaceId(101);
    const OTHER: SpaceId = SpaceId(202);

    fn authority_bounds() -> Limits {
        Limits {
            max_weight: 150,
            max_pages: 64,
            expires_at_epoch: 10,
        }
    }

    fn root_cap(table: &mut CapTable<4>) -> Handle {
        table
            .mint_root(
                CALLER,
                A,
                Rights::UPDATE.union(Rights::DELEGATE).union(Rights::REVOKE),
                authority_bounds(),
                1,
            )
            .unwrap()
    }

    fn gate() -> (PolicyGate, CapTable<4>, Handle) {
        let mut table = CapTable::<4>::new();
        let handle = root_cap(&mut table);
        (PolicyGate::new(), table, handle)
    }

    fn good(epoch: u64) -> Proposal {
        Proposal {
            tenant: A,
            epoch,
            weight: 100,
            pages: 16,
        }
    }
    #[test]
    fn valid_advice_is_authorized_not_executed() {
        let (mut g, table, handle) = gate();
        assert!(matches!(
            g.evaluate(&table, CALLER, handle, Some(good(1)), 1),
            Decision::Advisory(_)
        ));
    }
    #[test]
    fn model_absence_falls_back() {
        let (mut g, table, handle) = gate();
        assert_eq!(
            g.evaluate(&table, CALLER, handle, None, 1),
            Decision::Baseline(Rejection::Missing)
        );
    }
    #[test]
    fn foreign_tenant_is_denied() {
        let (mut g, table, handle) = gate();
        let mut p = good(1);
        p.tenant = B;
        assert_eq!(
            g.evaluate(&table, CALLER, handle, Some(p), 1),
            Decision::Baseline(Rejection::WrongTenant)
        );
    }
    #[test]
    fn priority_clamping_is_not_substituted_for_rejection() {
        let (mut g, table, handle) = gate();
        let mut p = good(1);
        p.weight = 250;
        assert_eq!(
            g.evaluate(&table, CALLER, handle, Some(p), 1),
            Decision::Baseline(Rejection::InvalidWeight)
        );
        assert!(matches!(
            g.evaluate(&table, CALLER, handle, Some(good(1)), 1),
            Decision::Advisory(_)
        ));
    }
    #[test]
    fn budgets_are_bounded() {
        let (mut g, table, handle) = gate();
        let mut p = good(1);
        p.pages = 65;
        assert_eq!(
            g.evaluate(&table, CALLER, handle, Some(p), 1),
            Decision::Baseline(Rejection::MemoryLimit)
        );
    }
    #[test]
    fn expired_or_replayed_advice_is_denied() {
        let (mut g, table, handle) = gate();
        assert!(matches!(
            g.evaluate(&table, CALLER, handle, Some(good(2)), 2),
            Decision::Advisory(_)
        ));
        assert_eq!(
            g.evaluate(&table, CALLER, handle, Some(good(2)), 2),
            Decision::Baseline(Rejection::ReplayOrStale)
        );
        assert_eq!(
            g.evaluate(&table, CALLER, handle, Some(good(11)), 11),
            Decision::Baseline(Rejection::Expired)
        );
    }

    #[test]
    fn forged_or_wrong_owner_handle_is_rejected() {
        let (mut g, table, handle) = gate();
        assert_eq!(
            g.evaluate(&table, OTHER, handle, Some(good(1)), 1),
            Decision::Baseline(Rejection::Unauthorized)
        );
        assert_eq!(
            g.evaluate(&table, CALLER, Handle::from_raw(999), Some(good(1)), 1),
            Decision::Baseline(Rejection::Unauthorized)
        );
    }
    #[test]
    fn cpu_admission_respects_capacity() {
        let yes = [
            MinimumCpu {
                tenant: A,
                ticks: 40,
            },
            MinimumCpu {
                tenant: B,
                ticks: 40,
            },
        ];
        let no = [
            MinimumCpu {
                tenant: A,
                ticks: 50,
            },
            MinimumCpu {
                tenant: B,
                ticks: 40,
            },
        ];
        assert_eq!(admit_minimums(100, 20, &yes), Ok(80));
        assert_eq!(
            admit_minimums(100, 20, &no),
            Err(AdmissionError::Overbooked)
        );
    }
    #[test]
    fn rejects_invalid_admission_inputs() {
        assert_eq!(admit_minimums(0, 0, &[]), Err(AdmissionError::InvalidEpoch));
        assert_eq!(
            admit_minimums(100, 100, &[]),
            Err(AdmissionError::InvalidEpoch)
        );
        assert_eq!(
            admit_minimums(
                100,
                0,
                &[MinimumCpu {
                    tenant: A,
                    ticks: 0
                }]
            ),
            Err(AdmissionError::InvalidShare)
        );
    }
}
