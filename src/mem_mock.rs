use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ptr;
use std::sync::{Mutex, MutexGuard, OnceLock};

static MEM_MAP: OnceLock<Mutex<HashMap<usize, u8>>> = OnceLock::new();
static MEM_MOCK_INSTANCE: OnceLock<Mutex<()>> = OnceLock::new();

unsafe fn load_mem(src: *const u8, size: usize) -> Vec<u8> {
    let mut map = MEM_MAP
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    let mut src_bytes = Vec::new();
    for offset in 0..size {
        src_bytes.push(match map.entry(src.addr() + offset) {
            Entry::Vacant(_) => unsafe { ptr::read(src.offset(offset.try_into().unwrap())) },
            Entry::Occupied(e) => *e.get(),
        });
    }

    src_bytes
}

unsafe fn write_mem(dst: *mut u8, val: Vec<u8>) {
    let mut map = MEM_MAP
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    for (offset, v) in val.iter().enumerate() {
        match map.entry(dst.addr() + offset) {
            Entry::Vacant(_) => unsafe {
                ptr::write_unaligned(dst.offset(offset.try_into().unwrap()), *v);
            },
            Entry::Occupied(mut e) => {
                e.insert(*v);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cesty_store(dst: *mut u8, src: *const u8, size: usize) {
    let bytes = unsafe { std::slice::from_raw_parts(src, size).to_vec() };
    write_mem(dst, bytes);
}

#[no_mangle]
pub unsafe extern "C" fn cesty_load(src: *const u8, dst: *mut u8, size: usize) {
    let bytes = load_mem(src, size);
    ptr::copy_nonoverlapping(bytes.as_ptr(), dst, size);
}

#[no_mangle]
pub unsafe extern "C" fn cesty_memmove(dst: *mut u8, src: *const u8, size: usize) {
    write_mem(dst, load_mem(src, size));
}

#[no_mangle]
pub unsafe extern "C" fn cesty_memset(dst: *mut u8, value: u8, size: usize) {
    write_mem(dst, vec![value; size]);
}

#[no_mangle]
pub unsafe extern "C" fn cesty_memcmp(a: *const u8, b: *const u8, size: usize) -> std::ffi::c_int {
    let a_val = load_mem(a, size);
    let b_val = load_mem(b, size);

    for (a, b) in a_val.iter().zip(b_val) {
        if *a < b {
            return -1;
        }
        if b < *a {
            return 1;
        }
    }

    0
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
            .unwrap_or_else(|e| e.into_inner());
        let mut map = MEM_MAP
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        map.clear();
        Memmock {
            _instance: instance,
        }
    }

    pub fn get(&self, addr: usize, len: usize) -> Option<Vec<u8>> {
        let mut map = MEM_MAP
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();

        let mut bytes = Vec::new();
        for offset in 0..len {
            match map.entry(addr + offset) {
                Entry::Vacant(_) => break,
                Entry::Occupied(e) => bytes.push(*e.get()),
            }
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
