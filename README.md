# cesty

Cesty is a tool for testing C code using Rust, including building and mock generation.

## Quick start

First, add Cesty as a dependency:
```toml
[dependencies]
cesty = "0.1.0"

[build-dependencies]
cesty-build = "0.1.0"
```

Next we need to add the test as a target in the toml file.
Cesty requires that for each C target you want to test, you create a separate test target.
This test target tells the compiler where to find the Rust file that defines the test, the C source files that needs to be tested as well as the header files required for building.

```toml
[[test]]
name = "test_foo"
path = "tests/test_foo.rs"

[cesty.test_foo]
sources = ["src/foo.c", "src/bar.c"]
includes = ["include/"]
```

You also need to add a `build.rs` to your project, that calls the Cesty build function.

```rust,ignore
fn main() {
    cesty_build::build_c_tests();
}
```

For each entry in `[cesty]`, the build helper compiles the
declared C sources with the `cc` crate, emits Cargo link directives for the
matching static library, and tracks the manifest, C sources, and headers under
the declared include paths for rebuilds.

See `examples/gpio` for a complete working crate with C sources, headers,
`build.rs`, test metadata, and Rust integration tests. Run it with:

```sh
cargo test --manifest-path examples/gpio/Cargo.toml
```

See `examples/uart` for a C buffering driver that depends on a lower-level UART
driver mocked from Rust:

```sh
cargo test --manifest-path examples/uart/Cargo.toml
```
