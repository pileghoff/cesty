#include <stdint.h>

void store_int(uint32_t val, volatile uint32_t *dst) { *dst = val; }

uint32_t load_int(volatile uint32_t *src) { return *src; }

void store_byte(uint8_t val, volatile uint8_t *dst, int offset) {
  dst[offset] = val;
}

uint8_t load_byte(volatile uint8_t *src, int offset) { return src[offset]; }
