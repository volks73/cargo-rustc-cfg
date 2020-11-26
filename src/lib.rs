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

use std::convert::TryFrom;
use std::process::{Command, Output};

/// The command line name of Cargo application.
pub const CARGO: &str = "cargo";

/// The command line name of the Rust compiler subcommand for Cargo.
pub const RUSTC: &str = "rustc";

/// The `cargo rustc` command.
///
/// Creates a `std::process:Command` for the `cargo rustc` subcommand, but does
/// _not_ include any of the flags or options to execute the `cargo rustc --
/// --print cfg` command. The `-- --print cfg` will need to be manually added by
/// using the `PrintCfg` type.
///
/// Use this type to create a "base" command for the `cargo rustc` subcommand,
/// add any arguments, and then add the `-- --print cfg` suffix before
/// executing using the `PrintCfg` type. This allows for customization of the
/// `cargo rustc` command without having to re-implement all arguments.
#[derive(Debug)]
pub struct CargoRustc(Command);

impl Default for CargoRustc {
    fn default() -> Self {
        let mut cmd = Command::new(CARGO);
        cmd.arg(RUSTC);
        Self(cmd)
    }
}

impl From<CargoRustc> for Command {
    fn from(c: CargoRustc) -> Command {
        c.0
    }
}

/// The `-- --print cfg` suffix arguments for the `cargo rustc -- --print cfg` command.
///
/// Adds the necessary arguments to the command to execute the `cargo rustc --
/// -- print cfg`. This is separated out so additional arguments can be added to
/// the command because "terminating" and output parsing without having to
/// re-implement all arguments as methods.
#[derive(Debug)]
pub struct PrintCfg(Command);

impl From<Command> for PrintCfg {
    fn from(mut c: Command) -> Self {
        c.arg("--");
        c.arg("--print");
        c.arg("cfg");
        Self(c)
    }
}

impl From<CargoRustc> for PrintCfg {
    fn from(c: CargoRustc) -> Self {
        Self::from(c.0)
    }
}

impl From<PrintCfg> for Command {
    fn from(c: PrintCfg) -> Command {
        c.0
    }
}

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
    pub fn new() -> Result<Self, Error> {
        Self::try_from(PrintCfg::from(CargoRustc::default()))
    }

    /// Any and all additional lines from the `cargo rustc -- --print cfg` output.
    ///
    /// These are any lines that were not recognized as target key-value lines.
    pub fn extras(&self) -> &Vec<String> {
        &self.extras
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

impl TryFrom<String> for Cfg {
    type Error = Error;

    fn try_from(t: String) -> Result<Self, Self::Error> {
        let mut cmd: Command = CargoRustc::default().into();
        cmd.arg("--target").arg(t);
        Self::try_from(PrintCfg::from(cmd))
    }
}

impl TryFrom<PrintCfg> for Cfg {
    type Error = Error;

    fn try_from(p: PrintCfg) -> Result<Self, Self::Error> {
        Self::try_from(p.0)
    }
}

impl TryFrom<Command> for Cfg {
    type Error = Error;

    fn try_from(mut cmd: Command) -> Result<Self, Self::Error> {
        let output = cmd.output()?;
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

#[derive(Debug)]
pub enum Error {
    Command(Output),
    FromUtf8(std::string::FromUtf8Error),
    Generic(String),
    Io(std::io::Error),
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
