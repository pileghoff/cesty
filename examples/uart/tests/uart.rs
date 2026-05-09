use std::ffi::c_int;

use cesty::{define_mock, mock};
use proptest::prelude::*;

define_mock!(fn low_uart_try_read() -> c_int);
define_mock!(fn low_uart_write_byte(byte: c_int));

const UART_BUFFER_CAPACITY: usize = 8;

proptest! {
    // We need proptest to fork, since the C module is
    // using static globals.
    // If we do not enable fork, multiple tests are going to touch
    // the same data.
    #![proptest_config(ProptestConfig {
        fork: true,
        .. ProptestConfig::default()
    })]

    #[test]
    fn poll_rx_buffers_available_bytes_from_low_level_uart(
        bytes in proptest::collection::vec(0u8..=127, 0..16)
    ) {
        let low_read = mock!(low_uart_try_read);

        for byte in &bytes {
            low_read.add_return((*byte).into());
        }

        low_read.add_return(-1);

        cesty_uart_example::init();
        cesty_uart_example::poll_rx();

        let expected: Vec<_> = bytes
            .iter()
            .copied()
            .take(UART_BUFFER_CAPACITY)
            .map(Some)
            .collect();
        let actual: Vec<_> = (0..expected.len())
            .map(|_| cesty_uart_example::read())
            .collect();

        prop_assert_eq!(actual, expected);
        prop_assert_eq!(cesty_uart_example::read(), None);

        let expected_low_read_calls = if bytes.len() < UART_BUFFER_CAPACITY {
            bytes.len() + 1
        } else {
            UART_BUFFER_CAPACITY
        };
        prop_assert_eq!(low_read.calls(), vec![(); expected_low_read_calls]);
    }

    #[test]
    fn writes_are_buffered_until_flush(
        bytes in proptest::collection::vec(0u8..=127, 0..16)
    ) {
        let low_write = mock!(low_uart_write_byte);
        low_write.set_default_return(());

        cesty_uart_example::init();

        for (index, byte) in bytes.iter().enumerate() {
            let accepted = cesty_uart_example::write(*byte);
            prop_assert_eq!(accepted, index < UART_BUFFER_CAPACITY);
        }

        cesty_uart_example::flush_tx();

        let expected: Vec<c_int> = bytes
            .iter()
            .copied()
            .take(UART_BUFFER_CAPACITY)
            .map(Into::into)
            .collect();

        prop_assert_eq!(low_write.calls(), expected);
    }
}
