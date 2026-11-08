#ifndef EMBER_MEMORY_H
#define EMBER_MEMORY_H
#include <stdint.h>
#include <stdbool.h>
#define EMBER_PAGE_SIZE 4096ULL
#define EMBER_USABLE_MEMORY 0ULL
#define EMBER_MIN_PHYS 0x100000ULL
struct ember_region { uint64_t base, length, type; };
struct frame_pool { uint64_t next, end; };
bool frame_pool_choose(struct frame_pool *pool, const struct ember_region *regions, uint64_t count, uint64_t min_pages);
bool frame_pool_take(struct frame_pool *pool, uint64_t *physical_address);
uint64_t frame_pool_remaining(const struct frame_pool *pool);
#endif
