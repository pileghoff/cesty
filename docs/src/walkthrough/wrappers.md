# Wrappers

Due to the way that linking works in Cesty, we need to define all the external references that we use in our tests, in our main crate (in our case that would be `src/lib.rs`), instead of in each individual test.

This also gives us a perfect excuse to define wrappers as the same time.
Wrappers are not strictly required, but do them anyway.

A wrapper that we create a Rust function that calls the C function. This function can contain logic to make the interface smoother, such as turning nice Rust types into C compatible types etc.

In our case this simply includes turning a bool into an int.

```rust
use std::ffi::c_int;

unsafe extern "C" {
    fn driver_set_led(pin: c_int, enabled: c_int);
}

pub fn set_led(pin: c_int, enabled: bool) {
    unsafe {
        driver_set_led(pin, c_int::from(enabled));
    }
}
```
