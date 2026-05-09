use std::ffi::c_int;

extern "C" {
    fn driver_set_led(pin: c_int, enabled: c_int);
    fn driver_read_button(pin: c_int) -> c_int;
}

pub fn set_led(pin: c_int, enabled: bool) {
    unsafe {
        driver_set_led(pin, c_int::from(enabled));
    }
}

pub fn read_button(pin: c_int) -> c_int {
    unsafe { driver_read_button(pin) }
}
