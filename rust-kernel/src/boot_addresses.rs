//! Limine HHDM and executable-address handoff; BSP-only, before paging changes.
//! Response-pointer validity is a bootloader trust assumption, not a proof.
use crate::address_provenance::{AddressProvenance, ProvenanceError};
use core::ptr::{addr_of, read_volatile};

#[repr(C)]
struct LimineHhdmRequest {
    id: [u64; 4],
    revision: u64,
    response: *const LimineHhdmResponse,
}

#[repr(C)]
struct LimineHhdmResponse {
    revision: u64,
    offset: u64,
}

#[repr(C)]
struct LimineExecutableAddressRequest {
    id: [u64; 4],
    revision: u64,
    response: *const LimineExecutableAddressResponse,
}

#[repr(C)]
struct LimineExecutableAddressResponse {
    revision: u64,
    physical_base: u64,
    virtual_base: u64,
}

// Limine protocol feature IDs, request revision 0:
// https://github.com/limine-bootloader/limine-protocol/blob/trunk/PROTOCOL.md
#[used]
#[link_section = ".limine_requests"]
static mut HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ],
    revision: 0,
    response: core::ptr::null(),
};

#[used]
#[link_section = ".limine_requests"]
static mut EXECUTABLE_ADDRESS_REQUEST: LimineExecutableAddressRequest =
    LimineExecutableAddressRequest {
        id: [
            0xc7b1dd30df4c8b88,
            0x0a82e883a194f07b,
            0x71ba76863cc55f63,
            0xb2644a48c516a487,
        ],
        revision: 0,
        response: core::ptr::null(),
    };

#[derive(Debug)]
pub enum BootAddressError {
    MissingHhdmResponse,
    MissingExecutableResponse,
    InvalidMetadata(ProvenanceError),
}

/// Copy the required Limine address metadata while its handoff mappings
/// remain live. No derived physical/virtual pointer is dereferenced.
///
/// The two nonnull response pointers must actually reference readable,
/// aligned, initialized, stable Limine response structs. This cannot be
/// established by checking pointer values alone.
pub fn load() -> Result<AddressProvenance, BootAddressError> {
    // SAFETY: Limine is the sole writer before BSP handoff; these request
    // fields remain mapped, and no concurrent writer exists after entry.
    let hhdm_response = unsafe { read_volatile(addr_of!(HHDM_REQUEST.response)) };
    if hhdm_response.is_null() {
        return Err(BootAddressError::MissingHhdmResponse);
    }
    // SAFETY: Limine's response pointer must reference a valid, aligned,
    // stable response in bootloader-reclaimable memory.
    let hhdm = unsafe { read_volatile(hhdm_response) };

    // SAFETY: Same exclusive BSP handoff contract as the HHDM request.
    let executable_response =
        unsafe { read_volatile(addr_of!(EXECUTABLE_ADDRESS_REQUEST.response)) };
    if executable_response.is_null() {
        return Err(BootAddressError::MissingExecutableResponse);
    }
    // SAFETY: Same Limine response pointer validity requirement as above.
    let executable = unsafe { read_volatile(executable_response) };

    AddressProvenance::new(
        hhdm.offset,
        executable.physical_base,
        executable.virtual_base,
    )
    .map_err(BootAddressError::InvalidMetadata)
}
