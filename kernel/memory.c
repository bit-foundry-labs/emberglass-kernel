/* Bootstrap allocator: aligned contiguous usable frames, no freeing yet. */
#include "memory.h"
#include <stdint.h>
#include <stdbool.h>
static uint64_t align_up(uint64_t n, uint64_t a) {
    uint64_t add = a - 1;
    if (n > UINT64_MAX - add) return 0;
    return (n + add) & ~add;
}
static uint64_t align_down(uint64_t n, uint64_t a) { return n & ~(a - 1); }
bool frame_pool_choose(struct frame_pool *pool, const struct ember_region *regions, uint64_t count, uint64_t min_pages) {
    if (!pool) return false;
    pool->next = pool->end = 0;
    if (!regions || min_pages == 0 || min_pages > UINT64_MAX / EMBER_PAGE_SIZE) return false;
    for (uint64_t i = 0; i < count; i++) {
        const struct ember_region *r = &regions[i];
        if (r->type != EMBER_USABLE_MEMORY || r->length < EMBER_PAGE_SIZE || r->base > UINT64_MAX - r->length) continue;
        uint64_t begin = r->base < EMBER_MIN_PHYS ? EMBER_MIN_PHYS : r->base;
        begin = align_up(begin, EMBER_PAGE_SIZE);
        uint64_t end = align_down(r->base + r->length, EMBER_PAGE_SIZE);
        if (begin == 0 || end <= begin || (end - begin) / EMBER_PAGE_SIZE < min_pages) continue;
        pool->next = begin; pool->end = end;
        return true;
    }
    return false;
}
bool frame_pool_take(struct frame_pool *pool, uint64_t *physical_address) {
    if (!pool || !physical_address || pool->next == 0 || pool->next >= pool->end || pool->end - pool->next < EMBER_PAGE_SIZE) return false;
    *physical_address = pool->next;
    pool->next += EMBER_PAGE_SIZE;
    return true;
}
uint64_t frame_pool_remaining(const struct frame_pool *pool) {
    return pool && pool->next <= pool->end ? (pool->end - pool->next) / EMBER_PAGE_SIZE : 0;
}
