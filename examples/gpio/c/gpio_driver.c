#include "gpio_driver.h"
#include "arch/types.h"

#ifndef GPIO_MODULE
#error "Gpio module not defined"
#endif

extern void hal_gpio_write(int pin, int value);
extern int hal_gpio_read(int pin);

void driver_set_led(int pin, int enabled) {
  hal_gpio_write(pin, enabled ? 1 : 0);
}

int driver_read_button(int pin) {
  if (pin == 0xdeadbeef) {
    driver_undefined();
  }
  return hal_gpio_read(pin);
}
