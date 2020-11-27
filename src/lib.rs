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

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The command line name of the Cargo application.
pub const CARGO: &str = "cargo";

/// The environment variable name for the Cargo appplication.
pub const CARGO_VARIABLE: &str = "CARGO";

/// The command line name of the Rust compiler subcommand for Cargo.
pub const RUSTC: &str = "rustc";

/// A container for the parsed output from the `cargo rustc -- --print cfg`
/// command.
#[derive(Clone, Debug, PartialEq)]
pub struct Cfg {
    extras: Vec<String>,
    target: Target,
}

impl Cfg {
    /// Creates a new configuration using the default Rust compiler (rustc) target.
    ///
    /// This executes the `cargo rustc -- --print cfg` command. If the
    /// configuration for a specific target triple is desired, then the
    /// `Cfg::try_from` should be used. The `Cfg::try_from` implementation
    /// executes the `cargo rustc --target <triple> -- --print cfg` command.
    ///
    /// If additional flags or options need to be included as arguments to the
    /// `cargo rustc -- --print cfg` command, then use the `Cfg::with_args`
    /// method.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg_default = Cfg::new()?;
    /// ```
    pub fn new() -> Result<Self, Error> {
        Self::with_args(std::iter::empty::<&str>())
    }

    /// Creates a new configuration for a specific target triple string.
    ///
    /// This executes the `cargo rustc --target <triple> -- --print cfg`
    /// command, where `<triple>` is a Rust compiler target. A list of available
    /// recognized target triples can be obtained using the `rustc --print
    /// target-list` command.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg_for_triple = Cfg::with_triple("i686-pc-windows-gnu")?;
    /// ```
    pub fn with_triple(s: &str) -> Result<Self, Error> {
        Self::with_args(&["--target", s])
    }

    /// Creates a new configuration but allows customization of the `cargo rustc
    /// -- --print cfg` command by adding command line arguments.
    ///
    /// This executes the `cargo rustc <args> -- --print cfg` command, where
    /// `<args>` is replaced with options and flags to included in the execution
    /// of the command. See the `std::process::Command::args` method for more
    /// information about adding arguments to commands.
    ///
    /// # Examples
    ///
    /// This can be used to add the `--target <triple>` option,
    ///
    /// ```
    /// let cfg_from_triple = Cfg::with_args(&["--target", "i686-pc-windows-msvc"])?;
    /// ```
    ///
    /// but this is a common enough use-case that a specific method for
    /// obtaining the configuration of a target triple is provided as the
    /// `with_triple` method,
    ///
    /// ```
    /// let cfg_for_triple = Cfg::with_triple("i686-pc-windows-msvc")?;
    /// ```
    pub fn with_args<I, S>(args: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>

    {
        let output = Command::new(
            env::var(CARGO_VARIABLE)
                .map(PathBuf::from)
                .ok()
                .unwrap_or_else(|| PathBuf::from(CARGO)))
            .arg(RUSTC)
            .args(args)
            .arg("--")
            .arg("--print")
            .arg("cfg")
            .output()?;
        if !output.status.success() {
            return Err(Error::Command(output));
        }
        let mut extras = Vec::new();
        let mut arch = None;
        let mut endian = None;
        let mut env = None;
        let mut family = None;
        let mut features = Vec::new();
        let mut os = None;
        let mut pointer_width = None;
        let mut vendor = None;
        let specification = String::from_utf8(output.stdout)?;
        for entry in specification.lines() {
            let mut parts = entry.split('=');

            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                match key {
                    "target_arch" => arch = Some(value.trim_matches('"').to_string()),
                    "target_endian" => endian = Some(value.trim_matches('"').to_string()),
                    "target_env" => env = {
                        let env = value.trim_matches('"').to_string();
                        if env.is_empty() {
                            None
                        } else {
                            Some(env)
                        }
                    },
                    "target_family" => family = {
                        let family = value.trim_matches('"').to_string();
                        if family.is_empty() {
                            None
                        } else {
                            Some(family)
                        }
                    },
                    "target_feature" => features.push(value.trim_matches('"').to_string()),
                    "target_os" => os = Some(value.trim_matches('"').to_string()),
                    "target_pointer_width" => pointer_width = Some(value.trim_matches('"').to_string()),
                    "target_vendor" => vendor = {
                        let vendor = value.trim_matches('"').to_string();
                        if vendor.is_empty() {
                            None
                        } else {
                            Some(vendor)
                        }
                    },
                    _ => {
                        extras.push(String::from(entry));
                    }
                }
            } else {
                extras.push(String::from(entry));
            }
        }

        Ok(Self {
            extras,
            target: Target {
                arch: arch.ok_or_else(|| Error::MissingOutput("target_arch"))?,
                endian: endian.ok_or_else(|| Error::MissingOutput("target_endian"))?,
                env,
                family,
                features,
                os: os.ok_or_else(|| Error::MissingOutput("target_os"))?,
                pointer_width: pointer_width.ok_or_else(|| Error::MissingOutput("target_pointer_width"))?,
                vendor
            }
        })
    }

    /// Any and all additional lines from the `cargo rustc -- --print cfg` output.
    ///
    /// These are any lines that were not recognized as target key-value lines,
    /// i.e. `key="value"`. Unlike the target key-value lines, any double
    /// quotes, `"`, are _not_ removed.
    pub fn extras(&self) -> &Vec<String> {
        &self.extras
    }

    /// All output that is prepended by the `target_` string.
    ///
    /// These are all recognized target key-value lines, i.e.
    /// `target_<key>="<value>"`. The double quotes are removed for the values.
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
    /// `x86`, `x86_64`, `arm`, and `arm64`. The surrounding double quotes, `"`,
    /// of the raw output of the `cargo rustc -- --print cfg` command are removed.
    pub fn arch(&self) -> &str {
        &self.arch
    }

    /// The endianness for the target configuration.
    ///
    /// This is the `target_endian` line in the output. Typical values included
    /// either `little` or `big`. The surrounding double quotes, `"`, of the raw
    /// output of the `cargo rustc -- --print cfg` command are removed.
    pub fn endian(&self) -> &str {
        &self.endian
    }

    /// The environment for the target configuration.
    ///
    /// This is the `target_env` line in the output. Typical values include
    /// `gnu`, `msvc`, `musl`, etc. If an environment is used or provided by a
    /// target, then this will be `None`. The surrounding double quotes, `"` of
    /// the raw output from the `cargo rustc -- --print cfg` command are
    /// removed.
    pub fn env(&self) -> Option<&String> {
        self.env.as_ref()
    }

    /// The family for the target configuration.
    ///
    /// This is the `target_family` line in the output. Example values include:
    /// `windows` or `unix`. If a family is not provided, then this will be
    /// `None`. The surrounding double quotes of the raw output from the `cargo
    /// rustc -- --print cfg` command are removed.
    pub fn family(&self) -> Option<&String> {
        self.family.as_ref()
    }

    /// A list of all features enabled for the target configuration.
    ///
    /// If a feature is _not_ enabled, then it will not be in the Vector.
    ///
    /// The surrounding double quotes of the raw output from the `cargo rustc --
    /// --print cfg` command are removed.
    pub fn features(&self) -> &Vec<String> {
        &self.features
    }

    /// The operating system (OS) for the target configuration.
    ///
    /// This is the `target_os` line in the output. Example values include:
    /// `linux`, `macOS`, and `windows`. For Windows, this is the same as the
    /// target's family. The surrounding double quotes, `"`, of the raw output
    /// from the `cargo rustc -- --print cfg` command are removed.
    pub fn os(&self) -> &str {
        &self.os
    }

    /// The pointer width for the target configuration.
    ///
    /// This is the `target_pointer_width` line in the output. Example values
    /// include: `32` or `64`. The surrounding double quotes, `"`, of the raw
    /// output from the `cargo rustc -- --print cfg` command are removed.
    pub fn pointer_width(&self) -> &str {
        &self.pointer_width
    }

    /// The vendor for the target configuration.
    ///
    /// This is the `target_vendor` line in the output. Example values include:
    /// `apple`, `unknown`, and `pc`. If no vendor is provided for a target,
    /// then this is `None`. The surrounding double quotes, `"`, of the raw
    /// output from the `cargo rustc -- --print cfg` command are removed.
    pub fn vendor(&self) -> Option<&String> {
        self.vendor.as_ref()
    }
}

/// The error type for cargo-rustc-cfg operations and associated traits.
///
/// Errors mostly originate from the dependencies and executing the `cargo rustc
/// -- --print cfg` command, i.e. Input/Output (IO) errors, but custom instances
/// of `Error` can be created with the `Generic` variant and a message.
#[derive(Debug)]
pub enum Error {
    /// A command operation failed. Any content in the STDERR stream is used as
    /// part of the error message.
    Command(Output),
    /// UTF-8 string conversion failed.
    FromUtf8(std::string::FromUtf8Error),
    /// A generic, or custom, error occurred. The message should contain the detailed information.
    Generic(String),
    /// An I/O operation failed.
    Io(std::io::Error),
    /// An expected output from the `cargo rustc -- --print cfg` command is missing.
    MissingOutput(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(output) => write!(f, "{:?}: {}", output, String::from_utf8_lossy(&output.stderr)),
            Self::FromUtf8(err) => write!(f, "{}", err),
            Self::Generic(msg) => write!(f, "{}", msg),
            Self::Io(err) => write!(f, "{}", err),
            Self::MissingOutput(key) => write!(f, "The '{}' is missing from the output", key),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Command(..) => None,
            Self::FromUtf8(err) => Some(err),
            Self::Generic(..) => None,
            Self::Io(err) => Some(err),
            Self::MissingOutput(..) => None,
        }
    }
}

impl From<&'static str> for Error {
    fn from(e: &str) -> Self {
        Error::Generic(String::from(e))
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Generic(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8(e)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
