#include <stdint.h>

#define REG_ADDR (0x8000)

void example_set(int val) { *((int *)REG_ADDR) = val; }

int example_get() { return *((int *)REG_ADDR); }
