/* Minimal Limine protocol declarations for our independent kernel.
 * Protocol material: https://github.com/limine-bootloader/limine-protocol
 * Validate compatibility with a pinned bootloader release before distribution.
 */
#ifndef EMBER_BOOT_PROTOCOL_H
#define EMBER_BOOT_PROTOCOL_H
#include <stdint.h>
#define LIMINE_COMMON_MAGIC 0xc7b1dd30df4c8b88ULL, 0x0a82e883a194f07bULL
#define LIMINE_BASE_REVISION(N) {0xf9562b2d5c95a6c8ULL, 0x6a7b384944536bdcULL, (N)}
#define LIMINE_BASE_REVISION_SUPPORTED(VAR) ((VAR)[2] == 0)
#define LIMINE_REQUESTS_START_MARKER {0xf6b8f4b39de7d1aeULL, 0xfab91a6940fcb9cfULL, 0x785c6ed015d3e316ULL, 0x181e920a7852b9d9ULL}
#define LIMINE_REQUESTS_END_MARKER {0xadc0e0531bb10d03ULL, 0x9572709f31764c62ULL}
#define LIMINE_FRAMEBUFFER_REQUEST_ID {LIMINE_COMMON_MAGIC, 0x9d5827dcd881dd75ULL, 0xa3148604f6fab11bULL}
#define LIMINE_MEMMAP_REQUEST_ID {LIMINE_COMMON_MAGIC, 0x67cf3d9d378a806fULL, 0xe304acdfc50c3c62ULL}
#define LIMINE_HHDM_REQUEST_ID {LIMINE_COMMON_MAGIC, 0x48dcf1cb8ad2b852ULL, 0x63984e959a98244bULL}
#define LIMINE_MEMMAP_USABLE 0ULL
#define LIMINE_FRAMEBUFFER_RGB 1
struct limine_framebuffer {
    void *address;
    uint64_t width, height, pitch;
    uint16_t bpp;
    uint8_t memory_model;
    uint8_t red_mask_size, red_mask_shift;
    uint8_t green_mask_size, green_mask_shift;
    uint8_t blue_mask_size, blue_mask_shift;
    uint8_t unused[7];
    uint64_t edid_size;
    void *edid;
    uint64_t mode_count;
    void **modes;
};
struct limine_framebuffer_response {
    uint64_t revision, framebuffer_count;
    struct limine_framebuffer **framebuffers;
};
struct limine_framebuffer_request {
    uint64_t id[4], revision;
    volatile struct limine_framebuffer_response *response;
};
struct limine_memmap_entry { uint64_t base, length, type; };
struct limine_memmap_response {
    uint64_t revision, entry_count;
    struct limine_memmap_entry **entries;
};
struct limine_memmap_request {
    uint64_t id[4], revision;
    volatile struct limine_memmap_response *response;
};
struct limine_hhdm_response { uint64_t revision, offset; };
struct limine_hhdm_request {
    uint64_t id[4], revision;
    volatile struct limine_hhdm_response *response;
};
#endif
