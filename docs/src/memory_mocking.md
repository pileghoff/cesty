# Memory mocking

Memory mocking allows you to intercept memory access in your C code.

This is usefull when testing code that writes directly to registers at fixed addresses.

Setting up the memory mocking means that the memory access is redirected into a legal address range, and it allows you to inspect the access to those addresses.

## Example

Assuming you have some C code that reads and writes to a register at address 0x8000.

This address is not legal to read and write to when using a desktop OS, and it would cause a segault.

We setup the memory mocking like so:

```rust
let mem_mock = Memmock::new();
mem_mock.set(0x8000, vec![0,0,0,0]);
```

Here we are instantiating a new memory mock object, and telling it to set the 4 bytes, starting at address 0x8000 to zeroes.

This means that if the C code attempts to read any of those 4 bytes, the access will be intercepted, and zeroes will be returned.

If the C code attempsts to read beyond those 4 bytes, e.g. if it attempted to read a 64bit number, we would read outside the memory mock region and trigger the segault.

If the C code wrote to the address, we could use the memory mock object to see the new value.


```rust
let mem_mock = Memmock::new();
mem_mock.set(0x8000, vec![0,0,0,0]);
unsafe { dut(); }
assert_eq!(mem_mock.get(0x8000, 4).unwrap(), vec![0xde,0xad,0xbe,0xef]);
```

## How

This works using a clang plugin that intercepts all memory access, and redirects it to a custom Rust handler.

The handler will check the memory access byte-wise, and check if the current memory access is in the list of mocked addresses.

If it is, the access will be redirected to a fake memory block.

Currently memory reads and writes, including memmove, memset, memcmp and memcpy are intercepted.
