#ifndef GPIO_DRIVER_H
#define GPIO_DRIVER_H

void driver_set_led(int pin, int enabled);
int driver_read_button(int pin);

#endif
