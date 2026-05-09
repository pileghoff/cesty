pub use ffi_mock_macro::mock;

use core::panic;
pub use lazy_static::lazy_static;
use std::{collections::VecDeque, sync::Mutex};

unsafe impl<Tin, Tout> Send for FunctionMockInner<Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone,
{
}
pub struct FunctionMockInner<Tin: Sized + 'static + Clone, Tout: Sized + 'static + Clone> {
    pub call_history: Vec<Tin>,
    pub return_val: VecDeque<Tout>,
    pub default_ret_val: Option<Tout>,
}

impl<Tin, Tout> FunctionMockInner<Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone,
{
    pub fn new() -> Self {
        FunctionMockInner {
            call_history: Vec::new(),
            return_val: VecDeque::new(),
            default_ret_val: None,
        }
    }

    pub fn get_next_return(&mut self) -> Tout {
        match self.return_val.pop_front() {
            Some(v) => v,
            None => match &self.default_ret_val {
                Some(v) => v.clone(),
                None => panic!("Unexpected call"),
            },
        }
    }
}

pub struct FunctionMock<'a, Tin: Sized + 'static + Clone, Tout: Sized + 'static + Clone> {
    inner: &'a Mutex<FunctionMockInner<Tin, Tout>>,
}

impl<'a, Tin, Tout> FunctionMock<'a, Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone,
{
    pub fn new(inner: &'a Mutex<FunctionMockInner<Tin, Tout>>) -> Self {
        inner.lock().unwrap().call_history.clear();
        inner.lock().unwrap().return_val.clear();
        inner.lock().unwrap().default_ret_val = None;
        FunctionMock { inner }
    }

    pub fn calls(self) -> Vec<Tin> {
        self.inner.lock().unwrap().call_history.clone()
    }

    pub fn add_return(&self, val: Tout) {
        self.inner.lock().unwrap().return_val.push_back(val);
    }

    pub fn set_default_return(&self, val: Tout) {
        let mut inner = self.inner.lock().unwrap();
        inner.default_ret_val = Some(val);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::null;

    use super::*;
    use crate as ffi_mock;

    #[derive(Clone)]
    struct TestStruct {
        a: u32,
        b: *const std::ffi::c_void,
        c: usize,
    }
    extern "C" {
        fn test_fun(a: u32, b: *const std::ffi::c_void, c: TestStruct) -> TestStruct;
    }

    fn test_indirection() -> u32 {
        let res = unsafe {
            test_fun(
                10,
                null(),
                TestStruct {
                    a: 0,
                    b: null(),
                    c: 0,
                },
            )
        };

        res.a
    }

    #[test]
    fn basic_setup() {
        let mock = mock!(
            fn test_fun(a: u32, b: *const std::ffi::c_void, c: TestStruct) -> TestStruct
        );

        mock.add_return(TestStruct {
            a: 0xbeef,
            b: null(),
            c: 0,
        });

        let res = test_indirection();
        assert_eq!(res, 0xbeef);
    }
}
