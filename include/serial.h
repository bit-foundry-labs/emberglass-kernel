#ifndef EMBER_SERIAL_H
#define EMBER_SERIAL_H
#include <stdint.h>
void serial_init(void);
void serial_puts(const char *text);
void serial_hex64(uint64_t value);
void serial_dec64(uint64_t value);
#endif
