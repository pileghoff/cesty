use std::sync::{Mutex, OnceLock};

static CESTY_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn cesty_run_test_internal(test_body: impl FnOnce()) {
    let _lock = match CESTY_TEST_LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(m) => m,
        Err(e) => e.into_inner(),
    };
    test_body()
}
