![Cesty logo](logo.svg)

Cesty is a tool for testing C code using Rust, including building and mock generation.

<a href="https://pil.fyi/cesty" ><img align="center" height="30" src="https://img.shields.io/badge/Docs-Docs?style=for-the-badge"></a>

Supported LLVM/Clang versions
| Version | Status                       |
|---------|------------------------------|
| LLVM 16 | ![llvm 16](./ci/llvm-16.svg) |
| LLVM 17 | ![llvm 17](./ci/llvm-17.svg) |
| LLVM 18 | ![llvm 18](./ci/llvm-18.svg) |
| LLVM 19 | ![llvm 19](./ci/llvm-19.svg) |
| LLVM 20 | ![llvm 20](./ci/llvm-20.svg) |
| LLVM 21 | ![llvm 21](./ci/llvm-21.svg) |
| LLVM 22 | ![llvm 22](./ci/llvm-22.svg) |

## Build tool

The goal of Cesty is to make it simpler to compile C sources, outside their native environment (both build environment and compile target)

You declare a Cesty test in you toml file, and tell Cesty which C file to compile and which include folders to include.

## Auto stub

If your file under test references a bunch of functions, that you dont want to build, you can enable auto-stub.

This will automatically stub any undefined symbol, meaning you wont get any errors when trying to link.

If you call any of these missing functions, you will simply get a panic.

# Mocks

Using cest-macro, you can generate powerful mocks for you C functions in Rust.

These mocks respect the Arrange-Act-Assert pattern, by allowing you to setup the return value (or more complex behaviour) of the mocks, and then later verify how many times and with what arguments it was called with.

```rust
// Arrange: Configure mock to return true
let gpio_mock = mock!(hal_gpio_write);
gpio_write.set_default_return(true);

// Act: Call the set_led function
cesty_gpio_example::set_led(13, true);
cesty_gpio_example::set_led(9, false);

// Assert: Verify the calls made to the mock
assert_eq!(gpio_write.calls(), vec![(9, 1), (13, 0)]);
```

# Memory mocking

Oftentimes in embedded systems, functionality is implemented not in software, but as a memory mapped peripheral.
To interact with these, we read from and write to special memory addresses.

This poses two major problems:
1. These memory addresses are often outside the legal range when running on host
2. We cant usually intercept these reads and writes to inspect them like we can function calls.

One solution is to make another layer where functions are used to access these registers, and if your code is already structured like that, you have solved the above problems.

In your test code, you can add the following:

```rust,ignore
let mem_mock = Memmock::new();
mem_mock.set(0x8000, vec![1]);
```

When any C code now attempts to read from this address, 0x8000, it will read a 1.

If it attempts to write into address 0x8000, it will succeed.

What happens is that its never actually accessing any memory at 0x8000. Instead the read and write operations are intercepted and redirected to a hashmap.
This also allows you to read and write to the address using the `get` and `set` members on `Memmock`.
