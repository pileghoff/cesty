use std::sync::{Mutex, OnceLock};

static CESTY_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn cesty_run_test(test_body: impl FnOnce()) {
    let _lock = CESTY_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    test_body()
}
