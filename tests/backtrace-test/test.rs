use cesty::{cesty_test, define_mock};

unsafe extern "C" {
    pub fn foo(a: i32) -> i32;
}

define_mock!(fn bar() -> i32);

#[cesty_test]
fn test_autostubbed() {
    unsafe { foo(1) };
}

#[cesty_test]
fn test_mocked_simple() {
    unsafe { foo(0) };
}

mod nested {
    use super::*;

    #[cesty_test]
    fn test_mocked_nested_func() {
        fn inner() {
            unsafe { foo(0) };
        }

        inner();
    }

    #[cesty_test]
    fn test_mocked_nested_closure() {
        fn inner(f: impl FnOnce()) {
            f();
        }

        inner(|| unsafe {
            foo(0);
        });
    }
}
