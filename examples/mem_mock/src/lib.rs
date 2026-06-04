use std::ffi::c_int;

extern "C" {
    pub fn example_set(val: c_int);
    pub fn example_get() -> c_int;
}
