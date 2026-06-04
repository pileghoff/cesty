use cesty::mem_mock::Memmock;
use cesty_mem_mock_example::{example_get, example_set};

#[test]
fn example_read_init() {
    let mem_mock = Memmock::new();
    mem_mock.set(0x8000, (0xbeefu32).to_ne_bytes().into_iter().collect());

    assert_eq!(unsafe { example_get() }, 0xbeef);
}

#[test]
fn example_write() {
    let mem_mock = Memmock::new();
    mem_mock.set(0x8000, (0u32).to_ne_bytes().into_iter().collect());

    unsafe { example_set(0xdead) };

    assert_eq!(
        mem_mock.get(0x8000).unwrap(),
        (0xdeadu32).to_ne_bytes().into_iter().collect::<Vec<u8>>()
    );
}

#[test]
fn example_write_and_read() {
    let mem_mock = Memmock::new();
    mem_mock.set(0x8000, (0u32).to_ne_bytes().into_iter().collect());

    unsafe { example_set(0xdead) };

    assert_eq!(unsafe { example_get() }, 0xdead);
}
