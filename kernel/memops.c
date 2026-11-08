#include <stddef.h>
#include <stdint.h>
/* Freestanding primitives; compilers may emit calls to these. */
void *memcpy(void *restrict dest, const void *restrict src, size_t count) {
    uint8_t *d=dest; const uint8_t *s=src;
    for(size_t i=0;i<count;i++)d[i]=s[i];
    return dest;
}
void *memset(void *dest, int byte, size_t count) {
    uint8_t *d=dest;
    for(size_t i=0;i<count;i++)d[i]=(uint8_t)byte;
    return dest;
}
void *memmove(void *dest, const void *src, size_t count) {
    uint8_t *d=dest; const uint8_t *s=src;
    if(d<s) for(size_t i=0;i<count;i++)d[i]=s[i];
    else if(d>s) while(count--)d[count]=s[count];
    return dest;
}
int memcmp(const void *a, const void *b, size_t count) {
    const uint8_t *x=a, *y=b;
    for(size_t i=0;i<count;i++)if(x[i]!=y[i])return x[i]<y[i]?-1:1;
    return 0;
}
