#![doc = include_str!("../README.md")]
pub use cesty_macro::cesty_test;
pub use cesty_macro::define_mock;
pub use cesty_macro::mock;
pub use lazy_static::lazy_static;

use crate::format_backtrace::format_backtrace;
mod format_backtrace;
pub mod mem_mock;
pub mod shared_state;
pub mod test_runner;

pub mod cesty_panic {
    use crate::format_backtrace::format_backtrace;
    use std::ffi::{CStr, c_char};
    use std::io::Write;
    use std::sync::atomic::Ordering;
    use yansi::Paint;

    #[unsafe(no_mangle)]
    extern "C" fn cesty_panic(function: *const c_char) {
        let function = unsafe { CStr::from_ptr(function) }.to_str().unwrap();
        let funct = std::sync::atomic::AtomicI8::new(0);
        let stack_trace = format_backtrace(function);

        std::panic::set_hook(Box::new(move |_info| {
            if funct.fetch_add(1, Ordering::Relaxed) == 0 {
                _ = std::io::stderr().write_fmt(format_args!(
                    "\n\n{}: Called auto-stubbed function {}\nFrom: \n  {}\n\n",
                    "Panic".bold().red(),
                    function.bold(),
                    stack_trace
                ));
            }
        }));

        panic!();
    }
}

unsafe impl<Tin, Tout> Send for FunctionMockInner<Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone + Default,
{
}
pub struct FunctionMockInner<Tin: Sized + 'static + Clone, Tout: Sized + 'static + Clone + Default>
{
    pub call_history: Vec<Tin>,
    pub return_val: std::collections::VecDeque<Tout>,
    pub handler: Option<Box<dyn std::ops::FnMut(Tin) -> Tout>>,
    pub default_ret_val: Option<Tout>,
    pub instantiated: bool,
    pub name: String,
}

impl<Tin, Tout> FunctionMockInner<Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone + Default,
{
    pub fn new(name: String) -> Self {
        FunctionMockInner {
            call_history: Vec::new(),
            return_val: std::collections::VecDeque::new(),
            default_ret_val: None,
            instantiated: false,
            handler: None,
            name,
        }
    }

    pub fn set_handler(&mut self, handler: Option<Box<dyn std::ops::FnMut(Tin) -> Tout>>) {
        self.handler = handler;
    }

    pub fn handle(&mut self, input: Tin) -> Tout {
        if !self.instantiated {
            let mock_name = self.name.clone();
            let funct = std::sync::atomic::AtomicI8::new(0);
            let stack_trace = format_backtrace(&mock_name);

            std::panic::set_hook(Box::new(move |_info| {
                use yansi::Paint;
                if funct.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0 {
                    _ = std::io::Write::write_fmt(
                        &mut std::io::stderr(),
                        format_args!(
                            concat!(
                                "\n---------------------------------------\n",
                                "Called uninstatied mock {} \n",
                                "From: \n  {}",
                                "\n-----------------------------------------\n",
                            ),
                            mock_name.bold().red(),
                            stack_trace,
                        ),
                    );
                }
            }));

            panic!();
        }
        if let Some(handler) = &mut self.handler {
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
                None => Tout::default(),
            },
        }
    }
}

pub struct FunctionMock<'a, Tin: Sized + 'static + Clone, Tout: Sized + 'static + Clone + Default> {
    inner: &'a std::sync::Mutex<FunctionMockInner<Tin, Tout>>,
}

impl<'a, Tin, Tout> FunctionMock<'a, Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone + Default,
{
    pub fn new(inner: &'a std::sync::Mutex<FunctionMockInner<Tin, Tout>>) -> Self {
        inner.lock().unwrap().call_history.clear();
        inner.lock().unwrap().return_val.clear();
        inner.lock().unwrap().default_ret_val = None;
        inner.lock().unwrap().instantiated = true;
        inner.lock().unwrap().set_handler(None);

        FunctionMock { inner }
    }

    pub fn handler(&self, handler: Box<dyn std::ops::FnMut(Tin) -> Tout>) {
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

impl<'a, Tin, Tout> Drop for FunctionMock<'a, Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone + Default,
{
    fn drop(&mut self) {
        self.inner.lock().unwrap().instantiated = false;
    }
}
