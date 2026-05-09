#include "uart_buffer.h"

#define UART_BUFFER_CAPACITY 8

static int rx_buffer[UART_BUFFER_CAPACITY];
static int rx_head;
static int rx_tail;
static int rx_len;

static int tx_buffer[UART_BUFFER_CAPACITY];
static int tx_head;
static int tx_tail;
static int tx_len;

void uart_buffer_init(void) {
  rx_head = 0;
  rx_tail = 0;
  rx_len = 0;
  tx_head = 0;
  tx_tail = 0;
  tx_len = 0;
}

void uart_buffer_poll_rx(void) {
  while (rx_len < UART_BUFFER_CAPACITY) {
    int byte = low_uart_try_read();

    if (byte < 0) {
      return;
    }

    rx_buffer[rx_tail] = byte & 0xff;
    rx_tail = (rx_tail + 1) % UART_BUFFER_CAPACITY;
    rx_len++;
  }
}

int uart_buffer_read(void) {
  if (rx_len == 0) {
    return -1;
  }

  int byte = rx_buffer[rx_head];
  rx_head = (rx_head + 1) % UART_BUFFER_CAPACITY;
  rx_len--;
  return byte;
}

int uart_buffer_write(int byte) {
  if (tx_len == UART_BUFFER_CAPACITY) {
    return 0;
  }

  tx_buffer[tx_tail] = byte & 0xff;
  tx_tail = (tx_tail + 1) % UART_BUFFER_CAPACITY;
  tx_len++;
  return 1;
}

void uart_buffer_flush_tx(void) {
  while (tx_len > 0) {
    low_uart_write_byte(tx_buffer[tx_head]);
    tx_head = (tx_head + 1) % UART_BUFFER_CAPACITY;
    tx_len--;
  }
}
