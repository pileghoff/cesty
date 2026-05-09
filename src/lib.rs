#![doc = include_str!("../README.md")]
pub use cesty_macro::define_mock;
pub use cesty_macro::mock;
pub use lazy_static::lazy_static;

unsafe impl<Tin, Tout> Send for FunctionMockInner<Tin, Tout>
where
    Tin: Sized + 'static + Clone,
    Tout: Sized + 'static + Clone,
{
}
pub struct FunctionMockInner<Tin: Sized + 'static + Clone, Tout: Sized + 'static + Clone> {
    pub call_history: Vec<Tin>,
    pub return_val: std::collections::VecDeque<Tout>,
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
            return_val: std::collections::VecDeque::new(),
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
