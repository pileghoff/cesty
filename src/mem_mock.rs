use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ptr;
use std::sync::{Mutex, MutexGuard, OnceLock};

static MEM_MAP: OnceLock<Mutex<HashMap<usize, u8>>> = OnceLock::new();

static MEM_MOCK_INSTANCE: OnceLock<Mutex<()>> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn cesty_store(dst: *mut u8, src: *const u8, size: usize) {
    let bytes = unsafe { std::slice::from_raw_parts(src, size).to_vec() };
    println!("Store {:?} @ {:08x}", bytes, dst.addr());
    let mut map = MEM_MAP
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    if map.contains_key(&dst.addr()) {
        let bytes = unsafe { std::slice::from_raw_parts(src, size) };
        for (offset, byte) in bytes.iter().enumerate() {
            map.insert(dst.addr() + offset, *byte);
        }
    } else {
        unsafe {
            ptr::copy_nonoverlapping(src, dst, size);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cesty_load(src: *const u8, dst: *mut u8, size: usize) {
    let mut map = MEM_MAP
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    let mut bytes = Vec::new();
    for offset in 0..size {
        let e = map.entry(src.addr() + offset);
        match e {
            Entry::Vacant(_) => {
                unsafe {
                    ptr::copy_nonoverlapping(src, dst, size);
                }
                return;
            }
            Entry::Occupied(e) => {
                bytes.push(*e.get());
            }
        }
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), dst, size);
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

        for (offset, val) in value.iter().enumerate() {
            map.insert(addr + offset, *val);
        }
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
        let mut map = MEM_MAP
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();

        let mut bytes = Vec::new();
        let mut offset = 0;
        loop {
            match map.entry(addr + offset) {
                Entry::Vacant(_) => break,
                Entry::Occupied(e) => bytes.push(*e.get()),
            }

            offset += 1;
        }

        if bytes.is_empty() {
            return None;
        }

        Some(bytes)
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
