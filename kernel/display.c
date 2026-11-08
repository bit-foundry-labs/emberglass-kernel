#include "display.h"
#include <stdint.h>
#include <stddef.h>
/* Native framebuffer drawing; no graphics libraries. */
static uint32_t channel(uint8_t value, uint8_t bits, uint8_t shift) {
    if (bits == 0 || bits > 8 || shift > 24) return 0;
    return ((uint32_t)(value * ((1u << bits) - 1u) / 255u)) << shift;
}
static uint32_t pixel(struct limine_framebuffer *f, uint8_t r, uint8_t g, uint8_t b) {
    return channel(r, f->red_mask_size, f->red_mask_shift) |
           channel(g, f->green_mask_size, f->green_mask_shift) |
           channel(b, f->blue_mask_size, f->blue_mask_shift);
}
static void rect(struct limine_framebuffer *f, uint64_t x, uint64_t y, uint64_t w, uint64_t h, uint32_t color) {
    if (x >= f->width || y >= f->height) return;
    if (w > f->width-x) w = f->width-x;
    if (h > f->height-y) h = f->height-y;
    for (uint64_t row=0; row<h; ++row) {
        volatile uint32_t *out = (volatile uint32_t *)((uintptr_t)f->address + (y+row)*f->pitch);
        for (uint64_t col=0; col<w; ++col) out[x+col] = color;
    }
}
void display_splash(struct limine_framebuffer *f) {
    if (!f || !f->address || f->memory_model != LIMINE_FRAMEBUFFER_RGB || f->bpp != 32 || f->width > UINT64_MAX/4 || f->pitch < f->width * 4 || f->height < 160 || f->width < 200) return;
    const uint32_t ink = pixel(f, 29, 33, 35);
    const uint32_t flame = pixel(f, 215, 135, 106);
    const uint32_t stone = pixel(f, 231, 225, 210);
    const uint32_t low = pixel(f, 85, 86, 85);
    rect(f, 0, 0, f->width, f->height, ink);
    uint64_t left = f->width / 8, top = f->height / 4;
    uint64_t unit = f->height / 16;
    rect(f, left, top, unit, unit * 4, flame);
    rect(f, left + unit + unit / 2, top, unit / 3, unit * 4, stone);
    rect(f, left + unit * 2, top, unit, unit / 3, stone);
    rect(f, left + unit * 2, top + unit * 2, unit, unit / 3, stone);
    rect(f, left + unit * 2, top + unit * 4 - unit / 3, unit, unit / 3, stone);
    rect(f, left, f->height - top, f->width / 2, 2, low);
    rect(f, left, f->height - top + 14, f->width / 4, 3, flame);
}
