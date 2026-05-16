# cesty

Cesty is a tool for testing C code using Rust, including building and mock generation.

## Build tool

The goal of Cesty is to make it simpler to compile C sources, outside their native environment (both build environment and compile target)

You declare a Cesty test in you toml file, and tell Cesty which C file to compile and which include folders to include.

```toml
[[test]]
name = "test_foo"
path = "tests/test_foo.rs"

[cesty.test_foo]
sources = ["src/foo.c", "src/bar.c"]
includes = ["include/"]
```

You can also tell Cesty to replace certain headers with your fake headers:

```toml
[cesty.test_foo]
sources = ["src/foo.c", "src/bar.c"]
includes = ["include/"]
replace = {"arch/types.h" = "much_simpler_types.h"}
```


You can even tell Cesty to just replace certain headers with empty ones, if you dont need anything defined in them anyway:

```toml
[cesty.test_foo]
sources = ["src/foo.c", "src/bar.c"]
includes = ["include/"]
ignore = ["arch/panic_handler.h"]
```

In your `build.rs` you need to call the Cesty build function:

```rust,ignore
fn main() {
    cesty_build::build_c_tests();
}
```

## Auto stub

If your file under test references a bunch of functions, that you dont want to build, you can enable auto-stub.

This will automatically stub any undefined symbol, meaning you wont get any errors when trying to link.

If you call any of these missing functions, you will simply get a panic.

# TODO

## Cleanup build lib

The build lib has a bunch of WIP.
Bad error handling etc.

This needs to be cleaned up, and i need to make a way for setting build flags etc.

## Bindgen integration

It would be nice if these tricks (auto-stubbing, header replace, etc.) could be used when generating bindings used for testing.

## Real world test

In the examples folder i really want an example of testing a driver from an open source project using proptest.
