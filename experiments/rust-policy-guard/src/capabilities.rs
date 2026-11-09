//! A bounded, kernel-owned capability table for a future Rust microkernel.
//!
//! The trusted kernel must own this table in protected memory. User code
//! receives only opaque Handle values and may submit them through IPC/syscalls.
//! Callers in this host experiment are *supplied by the test harness*;
//! in the actual kernel, SpaceId must be derived from the active thread.
//!
//! This is an implementable no_std mechanism, but NOT hardware isolation.

use crate::Tenant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpaceId(pub u32);

/// A user-visible, process-local handle. Its bit pattern is NOT a secret.
/// Knowledge of another space's handle does not grant its rights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle(u64);

impl Handle {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    fn index(self) -> usize {
        (self.0 as u32) as usize
    }

    fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }

    fn new(index: usize, generation: u32) -> Self {
        Self(((generation as u64) << 32) | index as u64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rights(u8);

impl Rights {
    pub const UPDATE: Self = Self(0b001);
    pub const DELEGATE: Self = Self(0b010);
    pub const REVOKE: Self = Self(0b100);

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Limits can only narrow across a delegation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_weight: u16,
    pub max_pages: u32,
    pub expires_at_epoch: u64,
}

impl Limits {
    fn within(self, parent: Self) -> bool {
        self.max_weight <= parent.max_weight
            && self.max_pages <= parent.max_pages
            && self.expires_at_epoch <= parent.expires_at_epoch
    }

    fn valid(self, now: u64) -> bool {
        self.max_weight > 0 && self.expires_at_epoch >= now
    }
}

/// A checked copy of the narrow authority attached to an UPDATE capability.
/// It is never accepted directly from an IPC message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthorizedPolicy {
    tenant: Tenant,
    limits: Limits,
}

impl AuthorizedPolicy {
    pub const fn tenant(self) -> Tenant {
        self.tenant
    }

    pub const fn limits(self) -> Limits {
        self.limits
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapError {
    AccessDenied,
    Expired,
    InvalidLimits,
    RightsEscalation,
    LimitsEscalation,
    Full,
}

#[derive(Clone, Copy)]
struct Entry {
    owner: SpaceId,
    tenant: Tenant,
    rights: Rights,
    limits: Limits,
    parent: Option<Handle>,
}

#[derive(Clone, Copy)]
struct Slot {
    generation: u32,
    entry: Option<Entry>,
    retired: bool,
}

impl Slot {
    const EMPTY: Self = Self {
        generation: 1,
        entry: None,
        retired: false,
    };

    fn clear(&mut self) {
        self.entry = None;
        // Never let a recycled slot return to the generation of an old handle.
        match self.generation.checked_add(1) {
            Some(next) => self.generation = next,
            None => self.retired = true,
        }
    }
}

/// Fixed-capacity, allocator-free reference design.
///
/// Only trusted kernel code may hold a mutable reference to this object.
/// A caller-provided SpaceId here models a kernel-derived current process.
/// All mutations must be serialized if multiple cores share this table.
pub struct CapTable<const N: usize> {
    slots: [Slot; N],
}

impl<const N: usize> Default for CapTable<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> CapTable<N> {
    pub const fn new() -> Self {
        Self {
            slots: [Slot::EMPTY; N],
        }
    }

    fn entry(&self, handle: Handle) -> Result<Entry, CapError> {
        let slot = self
            .slots
            .get(handle.index())
            .ok_or(CapError::AccessDenied)?;
        if slot.retired || slot.generation != handle.generation() {
            return Err(CapError::AccessDenied);
        }
        slot.entry.ok_or(CapError::AccessDenied)
    }

    // Validate every ancestor. Revoking a parent invalidates its descendants
    // even if a child handle is still held by another process.
    fn intact_lineage(&self, mut handle: Handle, now: u64) -> Result<Entry, CapError> {
        let first = self.entry(handle)?;
        for _ in 0..N {
            let entry = self.entry(handle)?;
            if now > entry.limits.expires_at_epoch {
                return Err(CapError::Expired);
            }
            match entry.parent {
                None => return Ok(first),
                Some(parent_handle) => {
                    let parent = self.entry(parent_handle)?;
                    if parent.tenant != entry.tenant
                        || !parent.rights.contains(Rights::DELEGATE)
                        || !parent.rights.contains(entry.rights)
                        || !entry.limits.within(parent.limits)
                    {
                        return Err(CapError::AccessDenied);
                    }
                    handle = parent_handle;
                }
            }
        }
        Err(CapError::AccessDenied)
    }

    fn allocate(&mut self, entry: Entry) -> Result<Handle, CapError> {
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if slot.entry.is_none() && !slot.retired && u32::try_from(index).is_ok() {
                slot.entry = Some(entry);
                return Ok(Handle::new(index, slot.generation));
            }
        }
        Err(CapError::Full)
    }

    /// KERNEL-ONLY operation. Never expose this through an untrusted syscall.
    /// Root capabilities come from the trusted service manager / boot logic.
    pub fn mint_root(
        &mut self,
        owner: SpaceId,
        tenant: Tenant,
        rights: Rights,
        limits: Limits,
        now: u64,
    ) -> Result<Handle, CapError> {
        if rights.is_empty() || !limits.valid(now) {
            return Err(CapError::InvalidLimits);
        }
        self.allocate(Entry {
            owner,
            tenant,
            rights,
            limits,
            parent: None,
        })
    }

    fn authorize(
        &self,
        caller: SpaceId,
        handle: Handle,
        required: Rights,
        now: u64,
    ) -> Result<Entry, CapError> {
        let entry = self.entry(handle)?;
        // Must be checked BEFORE traversing ancestry, to avoid leaking
        // ancestor information to a holder from a different process.
        if entry.owner != caller || !entry.rights.contains(required) {
            return Err(CapError::AccessDenied);
        }
        self.intact_lineage(handle, now)?;
        Ok(entry)
    }

    pub fn authorize_update(
        &self,
        caller: SpaceId,
        handle: Handle,
        now: u64,
    ) -> Result<AuthorizedPolicy, CapError> {
        let entry = self.authorize(caller, handle, Rights::UPDATE, now)?;
        Ok(AuthorizedPolicy {
            tenant: entry.tenant,
            limits: entry.limits,
        })
    }

    /// Delegation attenuates both permissions and quota ceilings.
    /// The parent stays valid; revoke it to invalidate the entire subtree.
    pub fn delegate(
        &mut self,
        caller: SpaceId,
        parent_handle: Handle,
        recipient: SpaceId,
        rights: Rights,
        limits: Limits,
        now: u64,
    ) -> Result<Handle, CapError> {
        let parent = self.authorize(caller, parent_handle, Rights::DELEGATE, now)?;
        if rights.is_empty() {
            return Err(CapError::InvalidLimits);
        }
        if !parent.rights.contains(rights) {
            return Err(CapError::RightsEscalation);
        }
        if !limits.valid(now) {
            return Err(CapError::InvalidLimits);
        }
        if !limits.within(parent.limits) {
            return Err(CapError::LimitsEscalation);
        }
        self.allocate(Entry {
            owner: recipient,
            tenant: parent.tenant,
            rights,
            limits,
            parent: Some(parent_handle),
        })
    }

    /// KERNEL-MEDIATED revocation requiring REVOKE authority on this handle.
    /// Revocation invalidates descendants; we eagerly reclaim their slots.
    /// This bounded scan is not optimized for large tables.
    pub fn revoke(&mut self, caller: SpaceId, handle: Handle, now: u64) -> Result<(), CapError> {
        self.authorize(caller, handle, Rights::REVOKE, now)?;
        self.slots[handle.index()].clear();

        // Reclaim orphaned descendants by repeatedly detecting references to
        // missing parents. With a fixed table the scan always terminates.
        loop {
            let mut changed = false;
            for index in 0..N {
                let parent = self.slots[index].entry.and_then(|entry| entry.parent);
                if let Some(parent_handle) = parent {
                    if self.entry(parent_handle).is_err() {
                        self.slots[index].clear();
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        Ok(())
    }

    pub fn live_slots(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.entry.is_some())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: SpaceId = SpaceId(101);
    const B: SpaceId = SpaceId(202);
    const T: Tenant = Tenant(7);

    fn bounds() -> Limits {
        Limits {
            max_weight: 150,
            max_pages: 64,
            expires_at_epoch: 10,
        }
    }

    fn root<const N: usize>(table: &mut CapTable<N>) -> Handle {
        table
            .mint_root(
                A,
                T,
                Rights::UPDATE.union(Rights::DELEGATE).union(Rights::REVOKE),
                bounds(),
                1,
            )
            .unwrap()
    }

    #[test]
    fn kernel_mints_and_owner_can_update() {
        let mut table = CapTable::<4>::new();
        let cap = root(&mut table);
        let checked = table.authorize_update(A, cap, 1).unwrap();
        assert_eq!(checked.tenant(), T);
        assert_eq!(checked.limits(), bounds());
    }

    #[test]
    fn possessing_another_space_handle_is_not_authority() {
        let mut table = CapTable::<4>::new();
        let stolen = root(&mut table);
        assert_eq!(
            table.authorize_update(B, stolen, 1),
            Err(CapError::AccessDenied)
        );
        assert_eq!(
            table.authorize_update(A, Handle::from_raw(123), 1),
            Err(CapError::AccessDenied)
        );
    }

    #[test]
    fn delegation_only_attenuates_and_binds_to_recipient() {
        let mut table = CapTable::<4>::new();
        let original = root(&mut table);
        let narrow = Limits {
            max_weight: 60,
            max_pages: 12,
            expires_at_epoch: 7,
        };
        let child = table
            .delegate(A, original, B, Rights::UPDATE, narrow, 1)
            .unwrap();
        assert_eq!(
            table.authorize_update(B, child, 2).unwrap().limits(),
            narrow
        );
        assert_eq!(
            table.authorize_update(A, child, 2),
            Err(CapError::AccessDenied)
        );
        assert_eq!(
            table.delegate(B, child, A, Rights::UPDATE, narrow, 2),
            Err(CapError::AccessDenied)
        );
    }

    #[test]
    fn rights_and_limits_cannot_be_amplified() {
        let mut table = CapTable::<4>::new();
        let original = root(&mut table);
        assert_eq!(
            table.delegate(A, original, B, Rights(0b1000), bounds(), 1),
            Err(CapError::RightsEscalation)
        );
        let too_large = Limits {
            max_weight: 151,
            ..bounds()
        };
        assert_eq!(
            table.delegate(A, original, B, Rights::UPDATE, too_large, 1),
            Err(CapError::LimitsEscalation)
        );
        let too_long = Limits {
            expires_at_epoch: 11,
            ..bounds()
        };
        assert_eq!(
            table.delegate(A, original, B, Rights::UPDATE, too_long, 1),
            Err(CapError::LimitsEscalation)
        );
    }

    #[test]
    fn revoked_handles_remain_stale_after_slot_reuse() {
        let mut table = CapTable::<1>::new();
        let old = root(&mut table);
        table.revoke(A, old, 1).unwrap();
        assert_eq!(
            table.authorize_update(A, old, 1),
            Err(CapError::AccessDenied)
        );
        let new = root(&mut table);
        assert_ne!(old.raw(), new.raw());
        assert_eq!(
            table.authorize_update(A, old, 1),
            Err(CapError::AccessDenied)
        );
        assert!(table.authorize_update(A, new, 1).is_ok());
    }

    #[test]
    fn revoking_parent_invalidates_multiple_generations_of_delegation() {
        let mut table = CapTable::<6>::new();
        let parent = root(&mut table);
        let child = table
            .delegate(
                A,
                parent,
                B,
                Rights::UPDATE.union(Rights::DELEGATE),
                bounds(),
                1,
            )
            .unwrap();
        let grandchild = table
            .delegate(B, child, A, Rights::UPDATE, bounds(), 1)
            .unwrap();
        table.revoke(A, parent, 1).unwrap();
        assert_eq!(table.live_slots(), 0);
        assert_eq!(
            table.authorize_update(B, child, 1),
            Err(CapError::AccessDenied)
        );
        assert_eq!(
            table.authorize_update(A, grandchild, 1),
            Err(CapError::AccessDenied)
        );
    }

    #[test]
    fn another_space_cannot_revoke_capability() {
        let mut table = CapTable::<2>::new();
        let cap = root(&mut table);
        assert_eq!(table.revoke(B, cap, 1), Err(CapError::AccessDenied));
        assert!(table.authorize_update(A, cap, 1).is_ok());
    }

    #[test]
    fn expired_capability_cannot_be_delegated_or_used() {
        let mut table = CapTable::<4>::new();
        let cap = root(&mut table);
        assert_eq!(table.authorize_update(A, cap, 11), Err(CapError::Expired));
        assert_eq!(
            table.delegate(A, cap, B, Rights::UPDATE, bounds(), 11),
            Err(CapError::Expired)
        );
    }

    #[test]
    fn capacity_and_zero_rights_are_rejected() {
        let mut table = CapTable::<1>::new();
        assert_eq!(
            table.mint_root(A, T, Rights(0), bounds(), 1),
            Err(CapError::InvalidLimits)
        );
        let _ = root(&mut table);
        assert_eq!(
            table.mint_root(A, T, Rights::UPDATE, bounds(), 1),
            Err(CapError::Full)
        );
    }

    #[test]
    fn full_table_rejects_additional_delegation_without_mutation() {
        let mut table = CapTable::<2>::new();
        let root = root(&mut table);
        let _child = table
            .delegate(A, root, B, Rights::UPDATE, bounds(), 1)
            .unwrap();
        assert_eq!(
            table.delegate(A, root, B, Rights::UPDATE, bounds(), 1),
            Err(CapError::Full)
        );
        assert_eq!(table.live_slots(), 2);
    }

    #[test]
    fn generational_counter_exhaustion_retires_slot() {
        let mut table = CapTable::<1>::new();
        table.slots[0].generation = u32::MAX;
        let last = root(&mut table);
        table.revoke(A, last, 1).unwrap();
        assert_eq!(
            table.mint_root(A, T, Rights::UPDATE, bounds(), 1),
            Err(CapError::Full)
        );
        assert_eq!(
            table.authorize_update(A, last, 1),
            Err(CapError::AccessDenied)
        );
    }
}
