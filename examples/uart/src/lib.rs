use std::ffi::c_int;

extern "C" {
    fn uart_buffer_init();
    fn uart_buffer_poll_rx();
    fn uart_buffer_read() -> c_int;
    fn uart_buffer_write(byte: c_int) -> c_int;
    fn uart_buffer_flush_tx();
}

pub fn init() {
    unsafe {
        uart_buffer_init();
    }
}

pub fn poll_rx() {
    unsafe {
        uart_buffer_poll_rx();
    }
}

pub fn read() -> Option<u8> {
    let byte = unsafe { uart_buffer_read() };

    if byte < 0 {
        None
    } else {
        Some(byte as u8)
    }
}

pub fn write(byte: u8) -> bool {
    unsafe { uart_buffer_write(byte.into()) != 0 }
}

pub fn flush_tx() {
    unsafe {
        uart_buffer_flush_tx();
    }
}
