//! Safe numeric checks for Limine-provided x86-64 address metadata.
//! These checks do not establish that any candidate virtual address is mapped.
use crate::memory::PAGE_BYTES;

const HIGHER_HALF_START: u64 = 0xffff_8000_0000_0000;
const EXECUTABLE_MIN_VIRTUAL: u64 = 0xffff_ffff_8000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceError {
    InvalidDirectMapOffset,
    InvalidExecutableBase,
    UnalignedFrame,
    AddressOverflow,
    NonCanonicalAddress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddressProvenance {
    pub hhdm_offset: u64,
    pub executable_physical_base: u64,
    pub executable_virtual_base: u64,
}

fn is_canonical_48(address: u64) -> bool {
    let high = address >> 48;
    if address & (1_u64 << 47) != 0 {
        high == 0xffff
    } else {
        high == 0
    }
}

impl AddressProvenance {
    /// Validate metadata for the current four-level x86-64 boot profile.
    /// The bootloader and its response pointers remain trusted external input.
    pub fn new(
        hhdm_offset: u64,
        executable_physical_base: u64,
        executable_virtual_base: u64,
    ) -> Result<Self, ProvenanceError> {
        if hhdm_offset < HIGHER_HALF_START
            || hhdm_offset % PAGE_BYTES != 0
            || !is_canonical_48(hhdm_offset)
        {
            return Err(ProvenanceError::InvalidDirectMapOffset);
        }
        if executable_physical_base % PAGE_BYTES != 0
            || executable_virtual_base < EXECUTABLE_MIN_VIRTUAL
            || executable_virtual_base % PAGE_BYTES != 0
            || !is_canonical_48(executable_virtual_base)
        {
            return Err(ProvenanceError::InvalidExecutableBase);
        }
        Ok(Self {
            hhdm_offset,
            executable_physical_base,
            executable_virtual_base,
        })
    }

    /// Compute only a *candidate* virtual number for a physical frame.
    /// The caller must independently prove the frame belongs to a mapped
    /// Limine memory-map type; this method never dereferences the address.
    pub fn hhdm_candidate(&self, physical_frame: u64) -> Result<u64, ProvenanceError> {
        if physical_frame % PAGE_BYTES != 0 {
            return Err(ProvenanceError::UnalignedFrame);
        }
        let address = self
            .hhdm_offset
            .checked_add(physical_frame)
            .ok_or(ProvenanceError::AddressOverflow)?;
        if !is_canonical_48(address) {
            return Err(ProvenanceError::NonCanonicalAddress);
        }
        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HHDM: u64 = 0xffff_8000_0000_0000;
    const EXEC_VA: u64 = 0xffff_ffff_8000_0000;

    #[test]
    fn accepts_page_aligned_higher_half_provenance() {
        let result = AddressProvenance::new(HHDM, 0x200000, EXEC_VA).unwrap();
        assert_eq!(result.hhdm_candidate(0x100000), Ok(HHDM + 0x100000));
        assert_eq!(result.executable_physical_base, 0x200000);
        assert_eq!(result.executable_virtual_base, EXEC_VA);
    }

    #[test]
    fn rejects_noncanonical_low_and_unaligned_hhdm_offsets() {
        for offset in [0, 0x1000, 0x0000_8000_0000_0000, HHDM + 1] {
            assert_eq!(
                AddressProvenance::new(offset, 0x200000, EXEC_VA),
                Err(ProvenanceError::InvalidDirectMapOffset)
            );
        }
    }

    #[test]
    fn rejects_noncanonical_low_or_unaligned_executable_bases() {
        for (physical, virtual_base) in [
            (0x200001, EXEC_VA),
            (0x200000, EXEC_VA + 1),
            (0x200000, 0x400000),
            (0x200000, 0x0000_8000_0000_0000),
        ] {
            assert_eq!(
                AddressProvenance::new(HHDM, physical, virtual_base),
                Err(ProvenanceError::InvalidExecutableBase)
            );
        }
    }

    #[test]
    fn refuses_unaligned_physical_frame_before_translation() {
        let metadata = AddressProvenance::new(HHDM, 0x200000, EXEC_VA).unwrap();
        assert_eq!(
            metadata.hhdm_candidate(0x100001),
            Err(ProvenanceError::UnalignedFrame)
        );
    }

    #[test]
    fn rejects_overflow_without_wrapping_to_low_addresses() {
        let metadata = AddressProvenance::new(0xffff_ffff_ffff_f000, 0x200000, EXEC_VA).unwrap();
        assert_eq!(
            metadata.hhdm_candidate(0x1000),
            Err(ProvenanceError::AddressOverflow)
        );
    }

    #[test]
    fn candidate_does_not_imply_mapping_or_access() {
        let metadata = AddressProvenance::new(HHDM, 0x200000, EXEC_VA).unwrap();
        // Numeric translation is permitted even for physical addresses
        // that the caller has not established as actually mapped.
        assert!(metadata.hhdm_candidate(0x3000).is_ok());
    }
}
