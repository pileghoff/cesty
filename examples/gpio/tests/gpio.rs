use cesty::{define_mock, mock};
use std::ffi::c_int;

define_mock!(fn hal_gpio_write(pin: c_int, value: c_int));
define_mock!(fn hal_gpio_read(pin: c_int) -> c_int);

#[test]
fn c_driver_forwards_led_writes_to_mocked_hal() {
    let gpio_write = mock!(hal_gpio_write);
    gpio_write.set_default_return(());

    cesty_gpio_example::set_led(13, true);
    cesty_gpio_example::set_led(13, false);

    assert_eq!(gpio_write.calls(), vec![(13, 1), (13, 0)]);
}

#[test]
fn c_driver_reads_button_from_mocked_hal() {
    let gpio_read = mock!(hal_gpio_read);
    gpio_read.add_return(1);
    gpio_read.add_return(0);

    let first = cesty_gpio_example::read_button(4);
    let second = cesty_gpio_example::read_button(4);

    assert_eq!((first, second), (1, 0));
    assert_eq!(gpio_read.calls(), vec![4, 4]);
}

#[test]
fn c_driver_cutsom_handler() {
    let gpio_read = mock!(hal_gpio_read);
    gpio_read.handler(Box::new(|i| i * 2));

    assert_eq!(cesty_gpio_example::read_button(4), 8);
}
