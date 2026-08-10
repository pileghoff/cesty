# Setup

The first step is to create the Rust crate.
This can be done using the following command:
```bash
cargo new --lib walkthrough
```

This will create a new crate in a folder named walkthrough.
The crate will contain a `Cargo.toml` file which defines both meta-data about the crate, such as name and license, as well as information for the build system, such as which files to include in the build.

The crate also contains a folder called `src` with a single file, `lib.rs`. We will use this file later, but for now you can safely ignore it.

## Adding a new test

To add our first test, we need to define it `Cargo.toml`

```toml
[[test]]
name = "gpio"
path = "tests/gpio.rs"
```

This tells cargo that we have a test in a subfolder called `tests`, and that the name of this test is `gpio`.

In addition to this, we also need to add a Cesty section to our toml file, describing how to build the C sources.

This section references the test name defined earlier, in this example `gpio`:

```toml
[cesty.gpio]
sources = ["c/gpio_driver.c"]
includes = ["include"]
```

We have told Cesty where to find the sources and the includes.

# Ignoring headers

Sometimes we have headers that we just want to ignore.
Maybe due to the way that the code is structured, we don't actually need it.
In our example `arch/types.h` is exactly such a header.

This can be achieved in Cesty using the `ignore` option.

```toml
[cesty.gpio]
sources = ["c/gpio_driver.c"]
includes = ["include"]
ignore = ["arch/types.h"]
```

## Defines

If we try to compile now, we will fail due to the missing define `GPIO_MODULE`.
This is easily fixed by adding a custom flag in the config:

```toml
[cesty.gpio]
sources = ["c/gpio_driver.c"]
includes = ["include"]
ignore = ["arch/types.h"]
flags = "-DGPIO_MODULE"
```

## Stubbing

The last thing stopping us from compiling the module is the undefined reference to the function `driver_undefined`, meaning the whole thing will fail during linking.

This is easily fixed in cesty by using auto stub.

<details>
<summary>
As an alternative we can also add a mock, that we simply never use.
</summary>

When you only have a few functions that you want to ignore, adding these unused mocks are preferred, but if you have a large legacy codebase, auto stub can help you ignore a ton of functions at the same time, to get you going immediately.

If the code ever calls a function that is auto stubbed, the code will immediately panic and inform you of exactly what function was called. This can be used to help guide you through the mocking process.
</details>

```toml
[cesty.gpio]
sources = ["c/gpio_driver.c"]
includes = ["include"]
ignore = ["arch/types.h"]
flags = "-DGPIO_MODULE"
auto_stub = true
```
