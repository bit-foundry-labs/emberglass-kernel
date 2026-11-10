//! Minimal Embers x86-64 Rust kernel entry. Not a protected microkernel.
#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use core::arch::asm;
use core::panic::PanicInfo;

const COM1: u16 = 0x3f8;
const UART_TRANSMIT_READY: u8 = 1 << 5;
const UART_MAX_POLLS: usize = 100_000;

// Limine protocol base revision 6; matches the independent C boot reference.
// https://github.com/limine-bootloader/limine-protocol/blob/trunk/PROTOCOL.md
#[used]
#[link_section = ".limine_requests_start"]
static REQUESTS_START: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

#[used]
#[link_section = ".limine_requests"]
static mut BASE_REVISION: [u64; 3] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 6];

#[used]
#[link_section = ".limine_requests_end"]
static REQUESTS_END: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];

/// # Safety
/// Requires privileged x86-64 execution with I/O access to COM1.
#[inline]
unsafe fn port_write(port: u16, value: u8) {
    // SAFETY: Caller guarantees privilege and port access.
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
    }
}

/// # Safety
/// Requires privileged x86-64 execution with I/O access to COM1.
#[inline]
unsafe fn port_read(port: u16) -> u8 {
    let value: u8;
    // SAFETY: Caller guarantees privilege and port access.
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nostack, preserves_flags));
    }
    value
}

fn serial_init() {
    // SAFETY: Limine's handoff runs this kernel in ring 0. The QEMU
    // smoke test provides an emulated legacy COM1 UART.
    unsafe {
        port_write(COM1 + 1, 0x00); // Disable UART interrupts.
        port_write(COM1 + 3, 0x80); // Divisor latch access.
        port_write(COM1, 0x03); // 38400 baud, 115200 base clock.
        port_write(COM1 + 1, 0x00);
        port_write(COM1 + 3, 0x03); // 8N1.
        port_write(COM1 + 2, 0xc7); // Enable and clear FIFOs.
        port_write(COM1 + 4, 0x0b); // DTR, RTS, OUT2.
    }
}

fn serial_write_byte(byte: u8) -> bool {
    for _ in 0..UART_MAX_POLLS {
        // SAFETY: Only called from the privileged entry/panic paths.
        if unsafe { port_read(COM1 + 5) } & UART_TRANSMIT_READY != 0 {
            // SAFETY: The UART reported a ready transmit register.
            unsafe { port_write(COM1, byte) };
            return true;
        }
    }
    false
}

fn serial_write(message: &str) {
    for byte in message.bytes() {
        if !serial_write_byte(byte) {
            return;
        }
    }
}

fn limine_revision_supported() -> bool {
    // SAFETY: Limine may rewrite word 2 before handoff; no other writer
    // is permitted after entry. The volatile read observes its response.
    let revision = unsafe {
        core::ptr::read_volatile(core::ptr::addr_of!(BASE_REVISION).cast::<u64>().add(2))
    };
    revision == 0
}

#[no_mangle]
pub extern "C" fn kernel_entry() -> ! {
    serial_init();
    serial_write("EMBERS: Rust no_std / x86_64 / boot M02\n");

    if !limine_revision_supported() {
        serial_write("ERROR: Limine base revision 6 unsupported\n");
        halt_forever();
    }

    serial_write("BOOT: Limine base revision accepted\n");
    serial_write("READY: Embers Rust no_std entry reached.\n");
    halt_forever()
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    serial_write("ERROR: Rust kernel panic\n");
    halt_forever()
}

fn halt_forever() -> ! {
    // SAFETY: Ring 0 and no installed IDT or interrupt handlers yet.
    unsafe { asm!("cli", options(nomem, nostack, preserves_flags)) };
    loop {
        // SAFETY: There is deliberately no scheduling in this experiment.
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}
