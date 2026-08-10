# Introduction

Cesty is a suite of tools, to enable testing of C code in Rust.

First Cesty helps you __build__ the parts of the C code you care about testing, while __stubbing__ out the parts you want to ignore.
Next, Cesty helps you __mock__ both __functions__ calls and __memory__ access.

<a href="https://github.com/pileghoff/cesty" ><img align="center" height="30" src="https://img.shields.io/badge/Repo-grey?style=for-the-badge&logo=github"></a>
<img align="center" height="30" src="https://img.shields.io/badge/MIT-blue?style=for-the-badge&label=License">
<a href="https://crates.io/crates/cesty" ><img align="center" height="30" alt="Crates.io Version" src="https://img.shields.io/crates/v/cesty?style=for-the-badge"></a>

## Quick start

## Installing dependencies

To get going, first you need to isntall clang and llvm, including the dev packages.

<details>

<summary>Ubuntu</summary>

```sh
sudo apt install clang lld llvm-dev
```
</details>

<details>

<summary>Arch Linux</summary>

```sh
sudo pacman -S clang llvm
```
</details>

### Setting up the test

Cesty is based on cargo tests. This means that to start out, you need to create a new Rust crate using `cargo new --lib cesty_quickstart`.

Next, add the dependencies by navigating to the newly created folder and running `cargo add cesty` and `cargo add --build cesty-build`.

Now, add the test folder and file, which is where all your first tests will live. You can add this anywhere, but we will create them in `./tests/cesty_quickstart.rs`

```rust
use cesty::cesty_test;

#[cesty_test]
fn cesty_quickstart() {
    assert_eq!(true, true);
}
```

We use the `cesty_test` attribute instead of the regular `test` attribute to ensure that tests are run single threaded and to inject additional information that makes Cesty able to show nicer error messages.

Next, cesty needs to be configured in your `Cargo.toml`.
In the config file you tell cesty where the test file lives, as well as what C sources needs to be compiled and how to compile them.

Assuming we have our C source file in `src/dut.c` and includes in `includes`, we will add the following to our config file:

```toml
[[test]]
name = "cesty_quickstart"
path = "tests/cesty_quickstart.rs"

[cesty.cesty_quickstart]
sources = ["c/dut.c"]
includes = ["include"]
```

Finally, you need to create a `build.rs` in the root of your crate. This is a build script that is run when you compile your Rust crate. Inside this, you call the cesty build, to trigger the build of your C sources.

```rust
fn main() {
    cesty_build::build_c_tests();
}
```

Now, you have a functional Cesty test, that does absolutly noting.

You can run the tests using `cargo test`.

## Why Rust when testing C?

C is still the lingua franca of many fields. Especially in embedded software, where both C++ and Rust struggle to gain much ground, due to the many years of dominance of C.

This is not a bad thing. C is still an excellent language, especially when it comes to solving the unique problems faced in embedded development. But it does have its warts and rough edges.

When it comes to writing tests, C often leads to verbose error prone tests.
Often times, this has been "solved" by writing the tests in a mix of C++ and C. This can remove some of the boilerplate, but we propose to take this a step further.

When writing your tests in Rust, you can take advantage of all the niceties rust offers.

- You get all the nice data structures offered by Rust
- The powerful macros reduces the boilerplate of much of mocking
- Crates such as proptest allows you more thoroughly test the code, in less lines of code.
