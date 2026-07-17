#![doc = include_str!("../README.md")]
pub use cesty_macro::define_mock;
pub use cesty_macro::mock;
pub use lazy_static::lazy_static;
pub mod mem_mock;
pub mod shared_state;
use std::ffi::{CStr, c_char};
use std::io::{self, Write};
use std::sync::atomic::{AtomicI8, Ordering};
use yansi::Paint;

#[unsafe(no_mangle)]
extern "C" fn cesty_panic(function: *const c_char) {
    let func = AtomicI8::new(0);
    std::panic::set_hook(Box::new(move |info| {
        let info = info.payload_as_str().unwrap_or("missing payload");
        if func.fetch_add(1, Ordering::Relaxed) == 0 {
            _ = std::io::stderr().write_fmt(format_args!(
                "\n\n{}: Called auto-stubbed function {}\n\n",
                "Panic".bold().red(),
                info.bold(),
            ));
        }
    }));

    let function = unsafe { CStr::from_ptr(function) };
    panic!("{:?}", function);
}

unsafe impl<Tin, Tout> Send for FunctionMockInner<Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone,
{
}
pub struct FunctionMockInner<Tin: Sized + 'static + Clone, Tout: Sized + 'static + Clone> {
    pub call_history: Vec<Tin>,
    pub return_val: std::collections::VecDeque<Tout>,
    pub handler: Option<Box<dyn std::ops::Fn(Tin) -> Tout>>,
    pub default_ret_val: Option<Tout>,
}

impl<Tin, Tout> Default for FunctionMockInner<Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone,
{
    fn default() -> Self {
        Self::new(None)
    }
}

impl<Tin, Tout> FunctionMockInner<Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone,
{
    pub fn new(handler: Option<Box<dyn std::ops::Fn(Tin) -> Tout>>) -> Self {
        FunctionMockInner {
            call_history: Vec::new(),
            return_val: std::collections::VecDeque::new(),
            default_ret_val: None,
            handler,
        }
    }

    pub fn set_handler(&mut self, handler: Option<Box<dyn std::ops::Fn(Tin) -> Tout>>) {
        self.handler = handler;
    }

    pub fn handle(&mut self, input: Tin) -> Tout {
        if let Some(handler) = &self.handler {
            return handler(input);
        }
        self.call_history.push(input);
        self.get_next_return()
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
    inner: &'a std::sync::Mutex<FunctionMockInner<Tin, Tout>>,
}

impl<'a, Tin, Tout> FunctionMock<'a, Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone,
{
    pub fn new(inner: &'a std::sync::Mutex<FunctionMockInner<Tin, Tout>>) -> Self {
        inner.lock().unwrap().call_history.clear();
        inner.lock().unwrap().return_val.clear();
        inner.lock().unwrap().default_ret_val = None;
        inner.lock().unwrap().set_handler(None);
        FunctionMock { inner }
    }

    pub fn handler(&self, handler: Box<dyn std::ops::Fn(Tin) -> Tout>) {
        self.inner.lock().unwrap().set_handler(Some(handler));
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
