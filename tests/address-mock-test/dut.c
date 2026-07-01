#include <stdint.h>
#include <stdlib.h>
#include <string.h>

void store_int(uint32_t val, uint32_t *dst) { *dst = val; }

uint32_t load_int(uint32_t *src) { return *src; }

void store_byte(uint8_t val, uint8_t *dst, int offset) { dst[offset] = val; }

uint8_t load_byte(uint8_t *src, int offset) { return src[offset]; }

void memcpy_proxy(uint8_t *dst, uint8_t *src, int len) {
  memcpy(dst, src, len);
}

void memmove_proxy(uint8_t *dst, uint8_t *src, int len) {
  memmove(dst, src, len);
}

void memset_proxy(uint8_t *dst, uint8_t v, int len) { memset(dst, v, len); }

int memcmp_proxy(uint8_t *a, uint8_t *b, int len) { return memcmp(a, b, len); }
