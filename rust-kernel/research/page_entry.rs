//! Pure encoding for a present 4 KiB supervisor leaf entry.
//! This does not establish frame ownership, map memory, or modify page tables.

#[derive(Debug, PartialEq, Eq)]
pub enum EntryError {
    InvalidPhysicalWidth,
    UnalignedAddress,
    AddressOutOfRange,
}

/// Encode a supervisor-only leaf entry with an explicitly chosen write bit.
///
/// `physical_bits` is a caller-supplied limit, not a detected CPU feature.
/// The caller must establish the actual CPU width and ownership before use.
/// NX is not set: executable protection and EFER.NXE are outside this model.
pub fn supervisor_leaf(address: u64, physical_bits: u8, writable: bool) -> Result<u64, EntryError> {
    if !(12..=52).contains(&physical_bits) {
        return Err(EntryError::InvalidPhysicalWidth);
    }
    if address & 0xfff != 0 {
        return Err(EntryError::UnalignedAddress);
    }
    if address >= (1u64 << physical_bits) {
        return Err(EntryError::AddressOutOfRange);
    }
    Ok(address | 1 | if writable { 2 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_supervisor_leaf_and_write_permission() {
        assert_eq!(supervisor_leaf(0x12345000, 36, false), Ok(0x12345001));
        assert_eq!(supervisor_leaf(0x12345000, 36, true), Ok(0x12345003));
        assert_eq!(supervisor_leaf(0, 36, false), Ok(1));
    }

    #[test]
    fn rejects_unaligned_and_out_of_width_addresses() {
        assert_eq!(supervisor_leaf(0x1001, 36, true), Err(EntryError::UnalignedAddress));
        assert_eq!(supervisor_leaf(1 << 36, 36, false), Err(EntryError::AddressOutOfRange));
        assert_eq!(supervisor_leaf((1 << 36) - 4096, 36, false), Ok((1 << 36) - 4096 + 1));
        assert_eq!(supervisor_leaf(1 << 52, 52, false), Err(EntryError::AddressOutOfRange));
    }

    #[test]
    fn rejects_invalid_widths() {
        for width in [0, 11, 53, 255] {
            assert_eq!(supervisor_leaf(0, width, false), Err(EntryError::InvalidPhysicalWidth));
        }
    }
}
