#include "serial.h"

#include <assert.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define COM1 0x3f8
#define TEST_POLL_LIMIT 8u

/* Port I/O is emulated; the real kernel port-instruction path is untouched. */
static unsigned int poll_count;
static unsigned int ready_on_poll;
static unsigned int writes;
static char written[64];

uint8_t ember_serial_test_inb(uint16_t port) {
    assert(port == COM1 + 5);
    ++poll_count;
    return poll_count >= ready_on_poll ? 0x20 : 0;
}

void ember_serial_test_outb(uint16_t port, uint8_t value) {
    assert(port == COM1);
    assert(writes < sizeof(written));
    written[writes++] = (char)value;
}

static void reset_uart(unsigned int ready_after) {
    poll_count = 0;
    ready_on_poll = ready_after;
    writes = 0;
    memset(written, 0, sizeof(written));
}

int main(void) {
    reset_uart(1);
    serial_puts("A");
    assert(poll_count == 1);
    assert(writes == 1 && written[0] == 'A');

    /* A ready signal on the last allowed poll must still transmit. */
    reset_uart(TEST_POLL_LIMIT);
    serial_puts("B");
    assert(poll_count == TEST_POLL_LIMIT);
    assert(writes == 1 && written[0] == 'B');

    /* A permanently busy device must not transmit after counter wrap. */
    reset_uart(UINT_MAX);
    serial_puts("C");
    assert(poll_count == TEST_POLL_LIMIT);
    assert(writes == 0);

    reset_uart(1);
    serial_puts("\n");
    assert(poll_count == 2);
    assert(writes == 2);
    assert(memcmp(written, "\r\n", 2) == 0);

    reset_uart(1);
    serial_hex64(0xABCD);
    assert(writes == 18 && memcmp(written, "0x000000000000ABCD", 18) == 0);

    reset_uart(1);
    serial_dec64(0);
    assert(writes == 1 && written[0] == '0');

    puts("PASS: UART immediate, final-poll, timeout, CRLF and integer formatting");
    return 0;
}
