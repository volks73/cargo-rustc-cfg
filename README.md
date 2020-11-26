# cargo-rustc-cfg: A Rust library (crate) for running the `cargo rustc -- --print cfg` command and parsing the output

A Rust library (crate) that runs the `cargo rustc -- --print cfg` command and parses the output. This is inspired by the [rustc-cfg](https://crates.io/crates/rustc-cfg) crate, which runs the `rustc --print cfg` command and parses the output, but this does not take into account any flags or configurations passed from cargo to the Rust compiler (rustc) when building a project using [Cargo](http://doc.crates.io/). For example, if the `RUSTFLAGS` environment variable is used to add a target feature, i.e. `RUSTFLAGS="-C target-feature=+crt-static`, then the `rustc --print cfg` command will not list the added target feature its output because the `RUSTFLAGS` environment variable is managed by Cargo. However, the `cargo rustc -- --print cfg` will list the added target feature its output. This crate is useful for developing [third-party](https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands) [Cargo custom subcommands](https://doc.rust-lang.org/1.30.0/cargo/reference/external-tools.html#custom-subcommands) that need compiler configuration information. This crate is not recommeded for [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html).

[![Crates.io](https://img.shields.io/crates/v/cargo-rustc-cfg.svg)](https://crates.io/crates/cargo-rustc-cfg)
[![GitHub release](https://img.shields.io/github/release/volks73/cargo-rustc-cfg.svg)](https://github.com/volks73/cargo-rustc-cfg/releases)
[![Crates.io](https://img.shields.io/crates/l/cargo-rustc-cfg.svg)](https://github.com/volks73/cargo-wix#license)
[![Build Status](https://github.com/volks73/cargo-rustc-cfg/workflows/CI/badge.svg?branch=master)](https://github.com/volks73/cargo-rustc_cfg/actions?query=branch%3main)

## Quick Start


## Installation


## Usage


## Tests


## License

The `cargo-rustc-cfg` project is licensed under either the [MIT license](https://opensource.org/licenses/MIT) or [Apache 2.0 license](http://www.apache.org/licenses/LICENSE-2.0). See the [LICENSE-MIT](https://github.com/volks73/cargo-rustc-cfg/blob/master/LICENSE-MIT) or [LICENSE-APACHE](https://github.com/volks73/cargo-rustc-cfg/blob/master/LICENSE-APACHE) files for more information about licensing and copyright.

