# Build configuration

The Cesty build config is done primarily in your `Cargo.toml`.
To catch this configuration, you also need a `build.rs`, that triggers the build

```rust
fn main() {
    cesty_build::build_c_tests();
}
```

Next, you need to add a test to your toml file. This test needs a name and a test file

```toml
[[test]]
name = "test_foo"
path = "tests/test_foo.rs"
```

This is just a regular Rust test. In order to instruct Cesty to compile a C source as part of this test, you need to add a custom Cesty section to your toml file. This is done using the header `[cesty.{TEST NAME}]`.
To this section you can add sources you wish to compile and where to find includes.

```toml
[cesty.test_foo]
sources = ["src/foo.c"]
includes = ["include/"]
```

In case there are some headers you want to replace with stubs, you can use the `replace` option:

```toml
replace = {"arch/types.h" = "stubs/much_simpler_types.h"}
```

This will copy the file located at `./stubs/much_simpler_types.h` into the include path as `arch/types.h`, such that the build will pick up this file instead of the original.

Sometimes there are include files that you don't actually care about, and you want to replace it with an empty file. For this Cesty offers the `ignore` option.

```toml
ignore = [ "arch/clock_tree.h" ]
```

When compiling, you often need to set specific flags to configure your defines or other compile options. Here Cesty provides the `flags` options. Flags expects a list of strings, each corresponding to a single flag.

```toml
flags = ["-DFOO=2", "-Wall"]
```

Cesty also offers the `auto_stub` option. This is disabled by default, but enabling it allows you to stub out any function that are not used in your test path.

In other words, if you only wish to test a specific part of the included sources, but the source files relies on functions defined in other files, you can tell Cesty to ignore those.
Auto stub will compile the source files, and find any undefined functions. Those undefined sources will be replaced by stubs that panic when called.
They are defined weakly, such that they can be replaced in Rust as mocks or fakes.

Here is a complete example of a test config:

Basic example:
```toml
[[test]]
name = "test_foo"
path = "tests/test_foo.rs"

[cesty.test_foo]
sources = ["src/foo.c", "src/bar.c"]
includes = ["include/"]
replace = {"arch/types.h" = "much_simpler_types.h"}
ignore = [ "arch/clock_tree.h" ]
flags = ["-DFOO=2", "-Wall"]
auto_stub = true
```

## Options reference

|  Name   |  Description        |  Type    |
|---------|--------------------|-----------|
| sources | Path to the source files to include in the build | String or list of strings |
| include | Path to the folders containing header files | String or list of strings |
| replace | Pairs, where the first is the header to replace, and the second is the header it will be replaced with | Map of strings to strings |
| ignore | Headers that will be ignored | List of strings |
| flags | Build flags | List of strings |
| auto_stub | Flag to enable or disable auto_stub (Disabled by default) | Bool |
