# Writing the first test

Before we can write any tests, we need a test file.

Earlier when we defined the test, we told Cargo that it would live in `tests/gpio.rs`, so lets create that file.

In the first test, we will start by simply calling the `set_led` function, with whatever arguments.

```rust
use cesty::cesty_test;
#[cesty_test]
fn first_test() {
    walkthrough::set_led(13, true);
}
```

Running this test will result in a panic, telling you it called an auto-stubbed function `hal_gpio_write`.
It will also try to explain to you how it got to that.

This will be your hint as to what we should do next.

## Adding the mocks

Mocks acts as the standin for the dependencies to our system-under-test that we do not wish to include.

In Cesty, creating mocks is a two step process. First, we define the mocks globally for our file.

Here we mock `hal_gpio_write`, as this was the function that we were missing an implementation for in the last step.

```rust
use cesty::{define_mock, mock};
use std::ffi::c_int;

define_mock!(fn hal_gpio_write(pin: c_int, value: c_int));
```

These mock signatures need to match the ones in our C code, but we need to use the matching rust types. Int gets replaced with `c_int` etc.

The macro `define_mock`, defines a global mock that, that can be instantiated in each test. Only one instance of a mock can ever exist at a time.

Running the tests now gives us a slightly different panic. It now says that we called an uninstantiated mock, so lets fix that by modifying our test:

```rust
#[cesty_test]
fn first_test() {
    let _hal_gpio_write_mock = mock!(hal_gpio_write);
    walkthrough::set_led(13, true);
}
```

The test now passes.

## Useful tests

The test passes, but currently tests nothing.

Let's write a test, that calls `set_led` twice, and verifies that the mock was called with the correct arguments.

This can be done check using the `calls` method on the mock instance, which returns a `Vec` of all the arguments provided when calling the mock.

This can be used to check both the number of calls to the mocks and the arguments provided when calling the mocks including the order.

```rust
#[cesty_test]
fn first_test() {
    let hal_gpio_write_mock = mock!(hal_gpio_write);
    walkthrough::set_led(13, true);
    walkthrough::set_led(9, false);

    assert_eq!(hal_gpio_write_mock.calls(), vec![(13, 1), (9, 0)]);
}
```
