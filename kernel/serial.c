#include "serial.h"

#include <stdint.h>

#define COM1 0x3f8
#define UART_TRANSMIT_READY 0x20

#ifndef SERIAL_READY_POLL_LIMIT
#define SERIAL_READY_POLL_LIMIT 100000u
#endif

#ifdef EMBER_SERIAL_TEST
/* Host-only test injection. Production keeps direct x86 port operations. */
extern void ember_serial_test_outb(uint16_t port, uint8_t value);
extern uint8_t ember_serial_test_inb(uint16_t port);
#define outb ember_serial_test_outb
#define inb ember_serial_test_inb
#else
static inline void outb(uint16_t port, uint8_t value) {
    __asm__ volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t result;
    __asm__ volatile("inb %1, %0" : "=a"(result) : "Nd"(port));
    return result;
}
#endif

void serial_init(void) {
    outb(COM1 + 1, 0);
    outb(COM1 + 3, 0x80);
    outb(COM1 + 0, 3);
    outb(COM1 + 1, 0);
    outb(COM1 + 3, 3);
    outb(COM1 + 2, 0xC7);
    outb(COM1 + 4, 0x0B);
}

static void serial_char(char character) {
    /* A bounded loop avoids unsigned-counter wraparound on timeout. */
    for (unsigned int attempt = 0; attempt < SERIAL_READY_POLL_LIMIT; ++attempt) {
        if ((inb(COM1 + 5) & UART_TRANSMIT_READY) != 0) {
            outb(COM1, (uint8_t)character);
            return;
        }
    }
    /* Preserve boot-time best-effort behavior: drop on UART timeout. */
}

void serial_puts(const char *text) {
    while (*text) {
        if (*text == '\n') {
            serial_char('\r');
        }
        serial_char(*text++);
    }
}

void serial_hex64(uint64_t value) {
    static const char hex[] = "0123456789ABCDEF";

    serial_puts("0x");
    for (int shift = 60; shift >= 0; shift -= 4) {
        serial_char(hex[(value >> shift) & 0xF]);
    }
}

void serial_dec64(uint64_t value) {
    char digits[21];
    unsigned int count = 0;

    do {
        digits[count++] = (char)('0' + value % 10);
        value /= 10;
    } while (value);

    while (count) {
        serial_char(digits[--count]);
    }
}
