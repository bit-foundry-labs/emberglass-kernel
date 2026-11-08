#include "serial.h"
#include <stdint.h>
#define COM1 0x3f8
static inline void outb(uint16_t port, uint8_t value) {
    __asm__ volatile ("outb %0, %1" : : "a"(value), "Nd"(port));
}
static inline uint8_t inb(uint16_t port) {
    uint8_t r;
    __asm__ volatile ("inb %1, %0" : "=a"(r) : "Nd"(port));
    return r;
}
void serial_init(void) {
    outb(COM1+1, 0); outb(COM1+3, 0x80);
    outb(COM1+0, 3); outb(COM1+1, 0);
    outb(COM1+3, 3); outb(COM1+2, 0xC7); outb(COM1+4, 0x0B);
}
static void serial_char(char c) {
    unsigned int tries = 100000;
    while (!(inb(COM1+5) & 0x20) && tries--) { }
    if (tries) outb(COM1, (uint8_t)c);
}
void serial_puts(const char *text) {
    while (*text) { if (*text == '\n') serial_char('\r'); serial_char(*text++); }
}
void serial_hex64(uint64_t value) {
    static const char hex[] = "0123456789ABCDEF";
    serial_puts("0x");
    for (int shift = 60; shift >= 0; shift -= 4) serial_char(hex[(value >> shift) & 0xF]);
}
void serial_dec64(uint64_t n) {
    char digits[21]; unsigned int count = 0;
    do { digits[count++] = (char)('0' + n % 10); n /= 10; } while(n);
    while (count) serial_char(digits[--count]);
}
