# Golden Trace Files

Expected trace logs for comparison testing.

## Format

Trace files are in JSON Lines format (one JSON object per line):

```json
{"type":"instruction","pc":"0x10000100","opcode":"0x2242"}
{"type":"mmio_read","addr":"0x40000000","value":"0x00000001"}
{"type":"gpio_change","pin":25,"value":true}
```

## Trace Types

- `instruction` - CPU instruction execution
- `mmio_read` - MMIO register read
- `mmio_write` - MMIO register write
- `gpio_change` - GPIO pin state change
- `uart_tx` - UART transmit
- `uart_rx` - UART receive
- `pio_exec` - PIO instruction execution