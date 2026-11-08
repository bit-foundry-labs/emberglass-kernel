/* Emberglass: original 64-bit higher-half kernel entry point. */
#include "boot_protocol.h"
#include "serial.h"
#include "display.h"
#include "memory.h"
#include <stdint.h>
#include <stdbool.h>
__attribute__((used, section(".limine_requests_start")))
static volatile uint64_t limine_start[] = LIMINE_REQUESTS_START_MARKER;
__attribute__((used, section(".limine_requests")))
static volatile uint64_t limine_revision[] = LIMINE_BASE_REVISION(6);
__attribute__((used, section(".limine_requests")))
static volatile struct limine_framebuffer_request fb_request = {.id = LIMINE_FRAMEBUFFER_REQUEST_ID, .revision = 0};
__attribute__((used, section(".limine_requests")))
static volatile struct limine_memmap_request map_request = {.id = LIMINE_MEMMAP_REQUEST_ID, .revision = 0};
__attribute__((used, section(".limine_requests")))
static volatile struct limine_hhdm_request hhdm_request = {.id = LIMINE_HHDM_REQUEST_ID, .revision = 0};
__attribute__((used, section(".limine_requests_end")))
static volatile uint64_t limine_end[] = LIMINE_REQUESTS_END_MARKER;
volatile uint64_t emberglass_boot_stage;
static __attribute__((noreturn)) void halt_forever(void) {
    for (;;) __asm__ volatile ("cli; hlt");
}
void kernel_entry(void) {
    emberglass_boot_stage = 1;
    serial_init();
    serial_puts("\nEMBERGLASS KERNEL / x86_64 / milestone 01\n");
    if (!LIMINE_BASE_REVISION_SUPPORTED(limine_revision)) {
        serial_puts("ERROR: unsupported Limine base revision.\n"); halt_forever();
    }
    emberglass_boot_stage = 2;
    serial_puts("BOOT: protocol base revision accepted\n");
    serial_puts("DISPLAY: checking framebuffer response\n");
    if (fb_request.response && fb_request.response->framebuffer_count && fb_request.response->framebuffers) {
        struct limine_framebuffer *f = fb_request.response->framebuffers[0];
        serial_puts("DISPLAY: dimensions = ");
        if (f) {
            serial_dec64(f->width); serial_puts("x"); serial_dec64(f->height);
            serial_puts(", pitch = "); serial_dec64(f->pitch); serial_puts("\n");
        } else {
            serial_puts("no framebuffer pointer\n");
        }
        serial_puts("DISPLAY: drawing splash\n");
        display_splash(f);
        serial_puts("DISPLAY: done\n");
    } else {
        serial_puts("WARN: framebuffer unavailable; serial remains active.\n");
    }
    if (!map_request.response || !map_request.response->entries || !hhdm_request.response) {
        serial_puts("ERROR: missing memory map or higher-half direct map.\n"); halt_forever();
    }
    struct frame_pool pool = {0};
    for (uint64_t i = 0; i < map_request.response->entry_count; ++i) {
        const struct limine_memmap_entry *r = map_request.response->entries[i];
        if (!r) continue;
        struct ember_region candidate = {r->base, r->length, r->type};
        if (frame_pool_choose(&pool, &candidate, 1, 16)) break;
    }
    if (!pool.next) { serial_puts("ERROR: no 16-page usable physical-memory region.\n"); halt_forever(); }
    uint64_t frame;
    if (frame_pool_take(&pool, &frame)) {
        serial_puts("MEMORY: first owned physical frame = "); serial_hex64(frame); serial_puts("\n");
        serial_puts("MEMORY: direct-map offset = "); serial_hex64(hhdm_request.response->offset); serial_puts("\n");
        serial_puts("MEMORY: remaining bootstrap frames = "); serial_dec64(frame_pool_remaining(&pool)); serial_puts("\n");
    }
    emberglass_boot_stage = 3;
    serial_puts("READY: original freestanding kernel entered successfully.\n");
    serial_puts("NEXT: exception handlers, paging ownership and scheduler.\n");
    halt_forever();
}
