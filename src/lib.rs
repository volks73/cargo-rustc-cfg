// Copyright (C) 2020 Christopher R. Field.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # `cargo-rustc-cfg` libraryhttps://doc.rust-lang.org/cargo/index.html
//!
//! The goal of this library, a.k.a. crate, is to make the compiler
//! configuration at the time of building a project with [Cargo] available to
//! [third-party] [Cargo custom subcommands] by running the `cargo rustc --
//! --print cfg` command and parse its output. This library is _not_ recommended
//! for [build scripts] as the compiler configuration information is available
//! via [Cargo environment variables] that are passed to build scripts at run
//! time.
//!
//! [Cargo]: https://doc.rust-lang.org/cargo/index.html
//! [third-party]: https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands
//! [Cargo custom subcommands]: https://doc.rust-lang.org/1.30.0/cargo/reference/external-tools.html#custom-subcommands
//! [build scripts]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
//! [Cargo environment variables]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts

/// A container for the parsed output from the `cargo rustc -- --print cfg`
/// command.
#[derive(Clone, Debug, PartialEq)]
pub struct Cfg {
    debug_assertions: bool,
    target: Target,
    family: Option<String>,
}

impl Cfg {
    /// Returns `true` if the Debug profile is used; otherwise it is `false` for
    /// the Release profile.
    pub fn has_debug_assertions(&self) -> bool {
        self.debug_assertions
    }

    /// Some targets will have a trailing line in the output that appears to be
    /// the same as the target's family.
    pub fn family(&self) -> Option<&String> {
        self.family.as_ref()
    }

    /// All output that is prepended by the `target_` string in the output.
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// Consumes this configuration and converts it into the target
    /// configuration.
    pub fn into_target(self) -> Target {
        self.target
    }
}

/// A container for all lines from the output that are prefixed with `target_`.
///
/// Note, this is the majority of the lines from the output.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    arch: String,
    endian: String,
    env: Option<String>,
    family: Option<String>,
    features: Vec<String>,
    os: String,
    pointer_width: String,
    vendor: Option<String>,
}

impl Target {
    /// The architecture for the target configuration.
    ///
    /// This is the `target_arch` line in the output. Example values include:
    /// `x86`, `x86_64`, `arm`, and `arm64`.
    pub fn arch(&self) -> &str {
        &self.arch
    }

    /// The endianness for the target configuration.
    ///
    /// This is the `target_endian` line in the output. Typical values included
    /// either `little` or `big`.
    pub fn endian(&self) -> &str {
        &self.endian
    }

    /// The environment for the target configuration.
    ///
    /// This is the `target_env` line in the output. Typical values include
    /// `gnu`, `msvc`, `musl`, etc. If an environment is used or provided by a
    /// target, then this will be `None`.
    pub fn env(&self) -> Option<&String> {
        self.env.as_ref()
    }

    /// The family for the target configuration.
    ///
    /// This is the `target_family` line in the output. Example values include:
    /// `windows` or `unix`. If a family is not provided, then this will be
    /// `None`.
    pub fn family(&self) -> Option<&String> {
        self.family.as_ref()
    }

    /// A list of all features enabled for the target configuration.
    ///
    /// If a feature is _not_ enabled, then it will not be in the Vector.
    pub fn features(&self) -> &Vec<String> {
        &self.features
    }

    /// The operating system (OS) for the target configuration.
    ///
    /// This is the `target_os` line in the output. Example values include:
    /// `linux`, `macOS`, and `windows`. For Windows, this is the same as the
    /// target's family.
    pub fn os(&self) -> &str {
        &self.os
    }

    /// The pointer width for the target configuration.
    ///
    /// This is the `target_pointer_width` line in the output. Example values
    /// include: `32` or `64`.
    pub fn pointer_width(&self) -> &str {
        &self.pointer_width
    }

    /// The vendor for the target configuration.
    ///
    /// This is the `target_vendor` line in the output. Example values include:
    /// `apple`, `unknown`, and `pc`. If no vendor is provided for a target,
    /// then this is `None`.
    pub fn vendor(&self) -> Option<&String> {
        self.vendor.as_ref()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
