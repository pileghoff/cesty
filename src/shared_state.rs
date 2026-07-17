use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SharedState<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> SharedState<T>
where
    T: Clone,
{
    pub fn new(val: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(val)),
        }
    }

    pub fn get(&self) -> T {
        self.inner.lock().unwrap().clone()
    }

    pub fn set(&self, val: T) {
        *self.inner.lock().unwrap() = val;
    }

    pub fn update<V>(&self, f: impl FnOnce(&mut T) -> V) -> V {
        let mut v = self.inner.lock().unwrap();
        f(&mut v)
    }
}
