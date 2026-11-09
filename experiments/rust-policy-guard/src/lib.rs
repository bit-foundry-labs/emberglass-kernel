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

/// Policy session bound explicitly to one capability and caller.
///
/// Binding is validated when the gate is created and rechecked on every
/// request. Successful advice updates this session's replay epoch.
#[derive(Debug)]
pub struct PolicyGate {
    bound_authority: BoundAuthority,
    last_epoch: Option<u64>,
}

#[derive(Clone, Copy, Debug)]
struct BoundAuthority {
    caller: SpaceId,
    handle: Handle,
}

impl PolicyGate {
    /// The real kernel must derive caller from a protected IPC/thread context.
    /// The host simulator receives caller from its trusted test harness.
    pub fn try_bind<const N: usize>(
        table: &CapTable<N>,
        caller: SpaceId,
        handle: Handle,
        now: u64,
    ) -> Result<Self, CapError> {
        table.authorize_update(caller, handle, now)?;
        Ok(Self {
            bound_authority: BoundAuthority { caller, handle },
            last_epoch: None,
        })
    }

    /// Validate advice and record a successful epoch. This method does NOT
    /// apply the policy to hardware; the kernel must protect that later step
    /// with an admission hold or fresh authorization.
    pub fn validate_and_record<const N: usize>(
        &mut self,
        table: &CapTable<N>,
        candidate: Option<Proposal>,
        now: u64,
    ) -> Decision {
        let authority = match table.authorize_update(
            self.bound_authority.caller,
            self.bound_authority.handle,
            now,
        ) {
            Ok(authority) => authority,
            Err(CapError::Expired) => return Decision::Baseline(Rejection::Expired),
            Err(_) => return Decision::Baseline(Rejection::Unauthorized),
        };
        let limits = authority.limits();
        let Some(proposal) = candidate else {
            return Decision::Baseline(Rejection::Missing);
        };
        let problem = if proposal.tenant != authority.tenant() {
            Some(Rejection::WrongTenant)
        } else if proposal.epoch != now
            || self.last_epoch.is_some_and(|previous| proposal.epoch <= previous)
        {
            Some(Rejection::ReplayOrStale)
        } else if proposal.weight == 0 || proposal.weight > limits.max_weight {
            Some(Rejection::InvalidWeight)
        } else if proposal.pages > limits.max_pages {
            Some(Rejection::MemoryLimit)
        } else {
            None
        };
        if let Some(error) = problem {
            return Decision::Baseline(error);
        }
        self.last_epoch = Some(proposal.epoch);
        Decision::Advisory(ValidatedPolicy {
            tenant: proposal.tenant,
            weight: proposal.weight,
            pages: proposal.pages,
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
    use crate::capabilities::{Limits, Rights, RootCapabilitySpec};

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
                RootCapabilitySpec {
                    owner: CALLER,
                    tenant: A,
                    rights: Rights::UPDATE.union(Rights::DELEGATE).union(Rights::REVOKE),
                    limits: authority_bounds(),
                },
                1,
            )
            .unwrap()
    }

    fn gate() -> (PolicyGate, CapTable<4>, Handle) {
        let mut table = CapTable::<4>::new();
        let handle = root_cap(&mut table);
        let policy_gate = PolicyGate::try_bind(&table, CALLER, handle, 1).unwrap();
        (policy_gate, table, handle)
    }

    fn other_root_cap(table: &mut CapTable<4>) -> Handle {
        table
            .mint_root(
                RootCapabilitySpec {
                    owner: OTHER,
                    tenant: B,
                    rights: Rights::UPDATE.union(Rights::DELEGATE).union(Rights::REVOKE),
                    limits: authority_bounds(),
                },
                1,
            )
            .unwrap()
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
        let (mut g, table, _) = gate();
        assert!(matches!(
            g.validate_and_record(&table, Some(good(1)), 1),
            Decision::Advisory(_)
        ));
    }
    #[test]
    fn model_absence_falls_back() {
        let (mut g, table, _) = gate();
        assert_eq!(
            g.validate_and_record(&table, None, 1),
            Decision::Baseline(Rejection::Missing)
        );
    }
    #[test]
    fn foreign_tenant_is_denied() {
        let (mut g, table, _) = gate();
        let mut p = good(1);
        p.tenant = B;
        assert_eq!(
            g.validate_and_record(&table, Some(p), 1),
            Decision::Baseline(Rejection::WrongTenant)
        );
    }
    #[test]
    fn priority_clamping_is_not_substituted_for_rejection() {
        let (mut g, table, _) = gate();
        let mut p = good(1);
        p.weight = 250;
        assert_eq!(
            g.validate_and_record(&table, Some(p), 1),
            Decision::Baseline(Rejection::InvalidWeight)
        );
        assert!(matches!(
            g.validate_and_record(&table, Some(good(1)), 1),
            Decision::Advisory(_)
        ));
    }
    #[test]
    fn budgets_are_bounded() {
        let (mut g, table, _) = gate();
        let mut p = good(1);
        p.pages = 65;
        assert_eq!(
            g.validate_and_record(&table, Some(p), 1),
            Decision::Baseline(Rejection::MemoryLimit)
        );
    }
    #[test]
    fn expired_or_replayed_advice_is_denied() {
        let (mut g, table, _) = gate();
        assert!(matches!(
            g.validate_and_record(&table, Some(good(2)), 2),
            Decision::Advisory(_)
        ));
        assert_eq!(
            g.validate_and_record(&table, Some(good(2)), 2),
            Decision::Baseline(Rejection::ReplayOrStale)
        );
        assert_eq!(
            g.validate_and_record(&table, Some(good(11)), 11),
            Decision::Baseline(Rejection::Expired)
        );
    }

    #[test]
    fn forged_or_wrong_owner_handle_is_rejected_at_binding() {
        let (_, table, handle) = gate();
        assert!(matches!(
            PolicyGate::try_bind(&table, OTHER, handle, 1),
            Err(CapError::AccessDenied)
        ));
        assert!(matches!(
            PolicyGate::try_bind(&table, CALLER, Handle::from_raw(999), 1),
            Err(CapError::AccessDenied)
        ));
    }

    #[test]
    fn rejected_proposal_does_not_advance_replay_epoch() {
        let (mut policy_gate, table, _) = gate();
        let mut invalid = good(3);
        invalid.weight = 0;
        assert_eq!(
            policy_gate.validate_and_record(&table, Some(invalid), 3),
            Decision::Baseline(Rejection::InvalidWeight)
        );
        assert!(matches!(
            policy_gate.validate_and_record(&table, Some(good(3)), 3),
            Decision::Advisory(_)
        ));
        assert_eq!(
            policy_gate.validate_and_record(&table, Some(good(3)), 3),
            Decision::Baseline(Rejection::ReplayOrStale)
        );
    }

    #[test]
    fn separate_gates_can_authorize_same_epoch_for_different_capabilities() {
        let mut table = CapTable::<4>::new();
        let caller_handle = root_cap(&mut table);
        let other_handle = other_root_cap(&mut table);
        let mut gate_a = PolicyGate::try_bind(&table, CALLER, caller_handle, 1).unwrap();
        let mut gate_b = PolicyGate::try_bind(&table, OTHER, other_handle, 1).unwrap();
        let proposal_b = Proposal {
            tenant: B,
            epoch: 3,
            weight: 100,
            pages: 16,
        };
        assert!(matches!(
            gate_a.validate_and_record(&table, Some(good(3)), 3),
            Decision::Advisory(_)
        ));
        assert!(matches!(
            gate_b.validate_and_record(&table, Some(proposal_b), 3),
            Decision::Advisory(_)
        ));
    }

    #[test]
    fn revoked_and_reissued_handle_cannot_rebind_gate() {
        let mut table = CapTable::<1>::new();
        let original_handle = table
            .mint_root(
                RootCapabilitySpec {
                    owner: CALLER,
                    tenant: A,
                    rights: Rights::UPDATE.union(Rights::DELEGATE).union(Rights::REVOKE),
                    limits: authority_bounds(),
                },
                1,
            )
            .unwrap();
        let mut original_gate =
            PolicyGate::try_bind(&table, CALLER, original_handle, 1).unwrap();
        assert!(matches!(
            original_gate.validate_and_record(&table, Some(good(1)), 1),
            Decision::Advisory(_)
        ));
        table.revoke(CALLER, original_handle, 1).unwrap();
        let reissued_handle = table
            .mint_root(
                RootCapabilitySpec {
                    owner: CALLER,
                    tenant: A,
                    rights: Rights::UPDATE.union(Rights::DELEGATE).union(Rights::REVOKE),
                    limits: authority_bounds(),
                },
                1,
            )
            .unwrap();
        assert_ne!(original_handle.raw(), reissued_handle.raw());
        assert_eq!(
            original_gate.validate_and_record(&table, Some(good(2)), 2),
            Decision::Baseline(Rejection::Unauthorized)
        );
        let mut reissued_gate =
            PolicyGate::try_bind(&table, CALLER, reissued_handle, 2).unwrap();
        assert!(matches!(
            reissued_gate.validate_and_record(&table, Some(good(2)), 2),
            Decision::Advisory(_)
        ));
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
