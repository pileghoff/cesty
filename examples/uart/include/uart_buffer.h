#ifndef UART_BUFFER_H
#define UART_BUFFER_H

int low_uart_try_read(void);
void low_uart_write_byte(int byte);

void uart_buffer_init(void);
void uart_buffer_poll_rx(void);
int uart_buffer_read(void);
int uart_buffer_write(int byte);
void uart_buffer_flush_tx(void);

#endif
