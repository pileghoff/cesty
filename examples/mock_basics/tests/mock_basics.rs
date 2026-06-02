use std::ffi::c_int;

use cesty::{define_mock, mock};

define_mock!(fn foo(pin: c_int) -> c_int);

#[test]
fn basic() {
    let foo_mock = mock!(foo);
    // set the default return value
    foo_mock.set_default_return(1);
    assert_eq!(foo(10), 1);

    // set the next return value
    foo_mock.add_return(2);
    assert_eq!(foo(11), 2);

    // queue up multiple return values
    foo_mock.add_return(3);
    foo_mock.add_return(4);
    assert_eq!(foo(12), 3);
    assert_eq!(foo(13), 4);
    assert_eq!(foo(14), 1); // at the end, you will then get back the default value you previously set.

    // you can also get the call history as a vec
    assert_eq!(foo_mock.calls(), vec![10, 11, 12, 13, 14]);
}
