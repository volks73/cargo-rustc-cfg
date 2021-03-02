# cargo-rustc-cfg: A Rust library (crate) for running the `cargo rustc --print cfg` command and parsing the output

A Rust library (crate) that runs the `cargo rustc --print cfg` command and parses the output. This is inspired by the [rustc-cfg] crate, which runs the `rustc --print cfg` command and parses the output, but it does not take into account any flags or configurations passed from [Cargo] to the [Rust compiler (rustc)] when building a project using Cargo. For example, if the `RUSTFLAGS` environment variable is used to add a target feature, i.e. `RUSTFLAGS="-C target-feature=+crt-static`, then the `rustc --print cfg` command will not list the added target feature in its output because the `RUSTFLAGS` environment variable is managed by Cargo. However, the `cargo rustc --print cfg` will list the added target feature in its output. This crate is useful for developing [third-party] [Cargo custom subcommands] that need compiler configuration information. This crate is _not_ recommeded for [build scripts].

[![Crates.io](https://img.shields.io/crates/v/cargo-rustc-cfg.svg)](https://crates.io/crates/cargo-rustc-cfg)
[![GitHub release](https://img.shields.io/github/release/volks73/cargo-rustc-cfg.svg)](https://github.com/volks73/cargo-rustc-cfg/releases)
[![Crates.io](https://img.shields.io/crates/l/cargo-rustc-cfg.svg)](https://github.com/volks73/cargo-rustc-cfg#license)
[![Build Status](https://github.com/volks73/cargo-rustc-cfg/workflows/CI/badge.svg)](https://github.com/volks73/cargo-rustc-cfg/actions)

[rustc-cfg]: https://crates.io/crates/rustc-cfg
[Cargo]: http://doc.crates.io
[Rust compiler (rustc)]: https://doc.rust-lang.org/rustc/index.html
[third-party]: https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands
[Cargo custom subcommands]: https://doc.rust-lang.org/1.30.0/cargo/reference/external-tools.html#custom-subcommands
[build scripts]: https://doc.rust-lang.org/cargo/reference/build-scripts.html

## Quick Start

**Note**, this crate currently needs the nightly toolchain. 

```rust
use cargo_rustc_cfg;

let host = cargo_rustc_cfg::host()?;
println("{:?}", host);
```

## Installation

Add the following to a package's manifest (Cargo.toml):

```toml
cargo-rustc-cfg = "0.2"
```

If using the [Rust 2015 Edition], then also add the following to the `lib.rs` or `main.rs` source file:

```rust
extern crate cargo_rustc_cfg;
```

[Rust 2015 Edition]: https://doc.rust-lang.org/stable/edition-guide/rust-2015/index.html

## Tests

Tests are run using the `cargo test` command. Currently, only [documentation tests] are implemented because for a relatively simple, small library these provide enough coverage. If the default toolchain is `stable` but the `nightly` toolchain is installed, the `cargo +nightly test` command can be used without having to switch the default toolchain.

[Documentation tests]: https://doc.rust-lang.org/rustdoc/documentation-tests.html

## License

The `cargo-rustc-cfg` project is licensed under either the [MIT license](https://opensource.org/licenses/MIT) or [Apache 2.0 license](http://www.apache.org/licenses/LICENSE-2.0). See the [LICENSE-MIT](https://github.com/volks73/cargo-rustc-cfg/blob/master/LICENSE-MIT) or [LICENSE-APACHE](https://github.com/volks73/cargo-rustc-cfg/blob/master/LICENSE-APACHE) files for more information about licensing and copyright.
