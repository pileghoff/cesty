use std::collections::HashMap;
use std::ptr;
use std::sync::{Mutex, MutexGuard, OnceLock};

static MEM_MAP: OnceLock<Mutex<HashMap<usize, Vec<u8>>>> = OnceLock::new();

static MEM_MOCK_INSTANCE: OnceLock<Mutex<()>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn cesty_store(dst: *mut u8, src: *const u8, size: usize) {
    let mut map = MEM_MAP
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    if let std::collections::hash_map::Entry::Occupied(mut e) = map.entry(dst.addr()) {
        let bytes = unsafe { std::slice::from_raw_parts(src, size).to_vec() };
        e.insert(bytes);
    } else {
        unsafe {
            ptr::copy_nonoverlapping(src, dst, size);
        }
    }
}

#[no_mangle]
pub extern "C" fn cesty_load(src: *const u8, dst: *mut u8, size: usize) {
    let mut map = MEM_MAP
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    if let std::collections::hash_map::Entry::Occupied(e) = map.entry(src.addr()) {
        let bytes = e.get();
        if bytes.len() >= size {
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), dst, size);
            }
            return;
        }
    }

    unsafe {
        ptr::copy_nonoverlapping(src, dst, size);
    }
}

pub struct Memmock<'a> {
    _instance: MutexGuard<'a, ()>,
}

impl<'a> Memmock<'a> {
    pub fn set(&self, addr: usize, value: Vec<u8>) {
        let mut map = MEM_MAP
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();

        map.insert(addr, value);
    }

    pub fn new() -> Memmock<'a> {
        let instance = MEM_MOCK_INSTANCE
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        let mut map = MEM_MAP
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();

        map.clear();
        Memmock {
            _instance: instance,
        }
    }

    pub fn get(&self, addr: usize) -> Option<Vec<u8>> {
        let map = MEM_MAP
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();

        map.get(&addr).cloned()
    }
}

impl<'a> Default for Memmock<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Memmock<'_> {
    fn drop(&mut self) {
        let mut map = MEM_MAP
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();

        map.clear();
    }
}
