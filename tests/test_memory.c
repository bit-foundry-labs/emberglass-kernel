#include <stdio.h>
#include <assert.h>
#include <stdint.h>
#include "memory.h"
int main(void) {
    struct frame_pool p = {0}; uint64_t addr=0;
    const struct ember_region regions[] = {
        {0x4000, 0x10000, 0},
        {0x200000, 0x20000, 1},
        {0x201123, 0x10000, 0}
    };
    assert(frame_pool_choose(&p, regions, 3, 15));
    assert(p.next == 0x202000 && p.end == 0x211000);
    assert(frame_pool_remaining(&p) == 15);
    assert(frame_pool_take(&p, &addr) && addr == 0x202000);
    assert(frame_pool_remaining(&p) == 14);
    for (int i=0;i<14;i++) assert(frame_pool_take(&p,&addr));
    assert(!frame_pool_take(&p,&addr));
    assert(!frame_pool_choose(&p, regions, 3, 16));
    assert(!frame_pool_choose(&p, regions, 3, 0));
    struct ember_region overflow[] = {{UINT64_MAX-32, 2048, 0}};
    assert(!frame_pool_choose(&p, overflow, 1, 1));
    assert(!frame_pool_choose(NULL, regions, 3, 1));
    assert(!frame_pool_choose(&p, NULL, 1, 1));
    struct ember_region high[] = {{0x400000, 4096, 0}};
    assert(frame_pool_choose(&p, high, 1, 1));
    assert(frame_pool_take(&p, &addr) && addr==0x400000);
    assert(!frame_pool_take(&p, &addr));
    puts("PASS: alignment, reserved exclusion, exhaustion, insufficient capacity, overflow and null inputs");
}
