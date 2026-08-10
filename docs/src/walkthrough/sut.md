# System under test

Before we write any Rust code, we need to examine the C code that we will be testing.

The walkthrough is based off of the gpio example code from the Cesty repo.

The main C module looks as follows:

```c
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
```

The two external functions `hal_gpio_write` and `hal_gpio_read` are the main dependencies, that we need to mock.

We also have the header `gpio_driver.h`:

```c
#ifndef GPIO_DRIVER_H
#define GPIO_DRIVER_H

void driver_set_led(int pin, int enabled);
int driver_read_button(int pin);

// This function is called from the gpio driver,
// but is not implemented anywhere.
int driver_undefined();

#endif
```

Looking at these files, we notice the following aspects as well:
- We have an additional header, `arch/types.h` that we need to handle.
- We need to define `GPIO_MODULE`, or else we will  get a compile error.
- There is a function `driver_undefined` that is only used in an edgecase, that we will not be using.
