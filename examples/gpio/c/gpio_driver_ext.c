#include "arch/types.h"
#include "gpio_driver.h"

extern int hal_gpio_read(int pin);

int driver_read_ext(int pin) { return hal_gpio_read(pin); }
