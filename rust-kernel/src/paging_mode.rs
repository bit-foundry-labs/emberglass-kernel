//! Read-only x86-64 paging preflight for M06b; no table writes or CR3 changes.
//! This validates selected control-register bits, not page-table contents.
use core::arch::asm;

const CR0_PG: u64 = 1 << 31;
const CR4_PAE: u64 = 1 << 5;
const CR4_LA57: u64 = 1 << 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PagingModeError {
    PagingDisabled,
    PhysicalAddressExtensionDisabled,
    FiveLevelPagingUnsupported,
}

/// Pure bit-level validation of the current four-level paging prerequisites.
///
/// The Limine x86-64 long-mode handoff is a separate trust assumption.
/// This check does not inspect EFER, CR3, translations, or page permissions.
pub fn validate(cr0: u64, cr4: u64) -> Result<(), PagingModeError> {
    if cr0 & CR0_PG == 0 {
        return Err(PagingModeError::PagingDisabled);
    }
    if cr4 & CR4_PAE == 0 {
        return Err(PagingModeError::PhysicalAddressExtensionDisabled);
    }
    if cr4 & CR4_LA57 != 0 {
        return Err(PagingModeError::FiveLevelPagingUnsupported);
    }
    Ok(())
}

/// Sample the BSP's current paging bits without modifying CPU state.
///
/// # Safety boundary
/// This function is only valid at CPL0 after Limine's x86-64 handoff, on
/// the bootstrap processor. CR reads would fault in an unprivileged caller.
pub fn check_active() -> Result<(), PagingModeError> {
    let cr0: u64;
    let cr4: u64;
    // SAFETY: Called only from the single privileged early-boot BSP path.
    // These instructions read control registers and do not change mappings.
    unsafe {
        asm!(
            "mov {value}, cr0",
            value = out(reg) cr0,
            options(nomem, nostack, preserves_flags)
        );
        asm!(
            "mov {value}, cr4",
            value = out(reg) cr4,
            options(nomem, nostack, preserves_flags)
        );
    }
    validate(cr0, cr4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_four_level_paging_and_unrelated_control_bits() {
        assert_eq!(validate(CR0_PG, CR4_PAE), Ok(()));
        assert_eq!(validate(CR0_PG | 1, CR4_PAE | (1 << 7)), Ok(()));
    }

    #[test]
    fn rejects_disabled_paging() {
        assert_eq!(validate(0, CR4_PAE), Err(PagingModeError::PagingDisabled));
    }

    #[test]
    fn rejects_missing_pae() {
        assert_eq!(
            validate(CR0_PG, 0),
            Err(PagingModeError::PhysicalAddressExtensionDisabled)
        );
    }

    #[test]
    fn rejects_five_level_paging() {
        assert_eq!(
            validate(CR0_PG, CR4_PAE | CR4_LA57),
            Err(PagingModeError::FiveLevelPagingUnsupported)
        );
    }

    #[test]
    fn rejection_priority_is_deterministic() {
        assert_eq!(validate(0, CR4_LA57), Err(PagingModeError::PagingDisabled));
        assert_eq!(
            validate(CR0_PG, CR4_LA57),
            Err(PagingModeError::PhysicalAddressExtensionDisabled)
        );
    }
}
