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

//! The goal of this library, a.k.a. crate, is to make the compiler
//! configuration at the time of building a project with [Cargo] available to
//! [third-party] [Cargo custom subcommands] by running the `cargo rustc --
//! --print cfg` command and parsing its output. This library is _not_ recommended
//! for [build scripts] as the compiler configuration information is available
//! via [Cargo environment variables] that are passed to build scripts at run
//! time.
//!
//! If the Rust compiler (rustc) target is `x86_64-pc-windows-msvc`, then the
//! output from the `cargo rustc -- --print cfg` command will look similar to
//! this:
//!
//! ```powershell
//! PS C:\Path\to\Rust\Project> cargo rustc -- --print cfg
//!   Compiling <PACKAGE> vX.X.X (<PACKAGE_PATH>)
//! debug_assertions
//! target_arch="x86_64"
//! target_endian="little"
//! target_env="msvc"
//! target_family="windows"
//! target_feature="fxsr"
//! target_feature="sse"
//! target_feature="sse2"
//! target_os="windows"
//! target_pointer_width="64"
//! target_vendor="pc"
//! windows
//!   Finished dev [unoptimized + debuginfo] target(s) in 0.10s
//! ```
//!
//! where `<PACKAGE>` is replaced with the name of the Rust package, the
//! `vX.X.X` is replaced with the Semantic Version number defined in the
//! package's manifest (Cargo.toml), and the `<PACKAGE_PATH>` is replaced with
//! the absolute path to the package's root directory. The output may vary
//! depending on the rustc target and development environment.
//!
//! This crate parses the above output and provides the [`Cfg`] and [`Target`]
//! types for accessing the various values from the output. The values for any lines containing
//! a key-value pair and prepended by the `target_` string are available in the
//! [`Target`] type with the double quotes, `"`, removed. Any lines that are not
//! recognized and/or not a target key-value pair are stored, unaltered and can
//! be obtained with the [`Cfg::extras`] method.
//!
//! # Examples
//!
//! Get the configuration for the default Rust compiler (rustc) target within
//! the Cargo "environment" if the rustc target is `x86_64-pc-windows-msvc`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc", target_vendor = "pc"))]
//! # mod x86_64_pc_windows_msvc {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::new()?;
//! assert_eq!(cfg.target().arch(), "x86_64");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), Some("msvc"));
//! assert_eq!(cfg.target().family(), Some("windows"));
//! assert_eq!(cfg.target().os(), "windows");
//! assert_eq!(cfg.target().pointer_width(), "64");
//! assert_eq!(cfg.target().vendor(), Some("pc"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! If the default rustc target is `x86_64-pc-windows-gnu`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "gnu", target_vendor = "pc"))]
//! # mod x86_64_pc_windows_gnu {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::new()?;
//! assert_eq!(cfg.target().arch(), "x86_64");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), Some("gnu"));
//! assert_eq!(cfg.target().family(), Some("windows"));
//! assert_eq!(cfg.target().os(), "windows");
//! assert_eq!(cfg.target().pointer_width(), "64");
//! assert_eq!(cfg.target().vendor(), Some("pc"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! If the default rustc target is `x86_64-unknown-linux-gnu`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
//! # mod x86_64_unknown_linux_gnu {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::new()?;
//! assert_eq!(cfg.target().arch(), "x86_64");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), None);
//! assert_eq!(cfg.target().family(), Some("unix"));
//! assert_eq!(cfg.target().os(), "os");
//! assert_eq!(cfg.target().pointer_width(), "64");
//! assert_eq!(cfg.target().vendor(), Some("unknown"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! If the default rustc target is `x86_64-apple-darwin`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
//! # mod x86_64_apple_darwin {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::new()?;
//! assert_eq!(cfg.target().arch(), "x86_64");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), None);
//! assert_eq!(cfg.target().family(), Some("unix"));
//! assert_eq!(cfg.target().os(), "os");
//! assert_eq!(cfg.target().pointer_width(), "64");
//! assert_eq!(cfg.target().vendor(), Some("apple"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! If the default rustc target is `i686-pc-windows-msvc`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86", target_os = "windows", target_env = "msvc", target_vendor = "pc"))]
//! # mod i686_pc_windows_msvc {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::new()?;
//! assert_eq!(cfg.target().arch(), "x86");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), Some("msvc"));
//! assert_eq!(cfg.target().family(), Some("windows"));
//! assert_eq!(cfg.target().os(), "windows");
//! assert_eq!(cfg.target().pointer_width(), "32");
//! assert_eq!(cfg.target().vendor(), Some("pc"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! If the default rustc target is `i686-pc-windows-gnu`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86", target_os = "windows", target_env = "gnu", target_vendor = "pc"))]
//! # mod i686_pc_windows_gnu {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::new()?;
//! assert_eq!(cfg.target().arch(), "x86_64");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), Some("gnu"));
//! assert_eq!(cfg.target().family(), Some("windows"));
//! assert_eq!(cfg.target().os(), "windows");
//! assert_eq!(cfg.target().pointer_width(), "32");
//! assert_eq!(cfg.target().vendor(), Some("pc"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! If the rustc target is `i686-unknown-linux-gnu`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86", target_os = "linux"))]
//! # mod i686_unknown_linux_gnu {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::new()?;
//! assert_eq!(cfg.target().arch(), "x86");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), None);
//! assert_eq!(cfg.target().family(), Some("unix"));
//! assert_eq!(cfg.target().os(), "os");
//! assert_eq!(cfg.target().pointer_width(), "32");
//! assert_eq!(cfg.target().vendor(), Some("unknown"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! If the rustc target is `i686-apple-darwin`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86", target_os = "macos"))]
//! # mod i686_apple_darwin {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::new()?;
//! assert_eq!(cfg.target().arch(), "x86");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), None);
//! assert_eq!(cfg.target().family(), Some("unix"));
//! assert_eq!(cfg.target().os(), "os");
//! assert_eq!(cfg.target().pointer_width(), "32");
//! assert_eq!(cfg.target().vendor(), Some("apple"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! Get the configuration for a specific Rust compiler (rustc) target triple
//! within the Cargo "environment" but using the [`Cfg::with_args`] with the
//! `--target <TRIPLE>` option for the `cargo rustc` subcommand:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::with_args(&["--target", "i686-pc-windows-msvc"], std::iter::empty::<&str>())?;
//! assert_eq!(cfg.target().arch(), "x86");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), Some("msvc"));
//! assert_eq!(cfg.target().family(), Some("windows"));
//! assert_eq!(cfg.target().os(), "windows");
//! assert_eq!(cfg.target().pointer_width(), "32");
//! assert_eq!(cfg.target().vendor(), Some("pc"));
//! # Ok(())
//! # }
//! ```
//!
//! The above use-case is relatively common, but it would be tedious to
//! routinely use the [`Cfg::with_args`] method. The [`Cfg::with_triple`]
//! method is available as a shorthand for the previous example:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::with_triple("i686-pc-windows-msvc")?;
//! assert_eq!(cfg.target().arch(), "x86");
//! assert_eq!(cfg.target().endian(), "little");
//! assert_eq!(cfg.target().env(), Some("msvc"));
//! assert_eq!(cfg.target().family(), Some("windows"));
//! assert_eq!(cfg.target().os(), "windows");
//! assert_eq!(cfg.target().pointer_width(), "32");
//! assert_eq!(cfg.target().vendor(), Some("pc"));
//! # Ok(())
//! # }
//! ```
//!
//! [`Cfg`]: struct.Cfg.html
//! [`Target`]: struct.Target.html
//! [`Cfg::extras`]: struct.Cfg.html#method.extras
//! [`Cfg::with_args`]: #method.with_args
//! [`Cfg::with_triple`]: #method_with_triple
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
    /// configuration for a specific Rust compiler (rustc) target triple is
    /// desired, then the [`Cfg::with_triple`] should be used. The
    /// [`Cfg::with_triple`] implementation executes the `cargo rustc --target
    /// <TRIPLE> -- --print cfg` command.
    ///
    /// If additional flags or options need to be included as arguments to the
    /// `cargo rustc -- --print cfg` command, then use the [`Cfg::with_args`]
    /// method.
    ///
    /// [`Cfg::with_triple`]: #method.with_triple
    /// [`Cfg::with_args`]: #method.with_args
    pub fn new() -> Result<Self, Error> {
        Self::with_args(std::iter::empty::<&str>(), std::iter::empty::<&str>())
    }

    /// Creates a new configuration for a specific target triple string.
    ///
    /// This executes the `cargo rustc --target <TRIPLE> -- --print cfg`
    /// command, where `<TRIPLE>` is a Rust compiler target. A list of available
    /// recognized target triples can be obtained using the `rustc --print
    /// target-list` command.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// let cfg = Cfg::with_triple("i686-pc-windows-gnu")?;
    /// assert_eq!(cfg.target().arch(), "x86");
    /// assert_eq!(cfg.target().endian(), "little");
    /// assert_eq!(cfg.target().env(), Some("gnu"));
    /// assert_eq!(cfg.target().family(), Some("windows"));
    /// assert_eq!(cfg.target().os(), "windows");
    /// assert_eq!(cfg.target().pointer_width(), "32");
    /// assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn with_triple(s: &str) -> Result<Self, Error> {
        Self::with_args(&["--target", s], std::iter::empty::<&str>())
    }

    /// Creates a new configuration but allows customization of the `cargo rustc
    /// -- --print cfg` command by adding command line arguments.
    ///
    /// This executes the `cargo rustc <CARGO_ARGS> -- <RUSTC_ARGS> --print cfg`
    /// command, where `<CARGO_ARGS>` is replaced with `cargo rustc` subcommand
    /// options and flags and the `<RUSTC_ARGS>` is replaced with options and
    /// flags that are passed to the Rust compiler command line interface after
    /// the `--` argument but before the `--print cfg` option. See the
    /// [`std::process::Command::args`] method for more information about adding
    /// arguments to commands.
    ///
    /// # Examples
    ///
    /// This can be used to add the `--target <TRIPLE>` option,
    ///
    /// ```
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// let cfg = Cfg::with_args(&["--target", "i686-pc-windows-msvc"], std::iter::empty::<&str>())?;
    /// assert_eq!(cfg.target().arch(), "x86");
    /// assert_eq!(cfg.target().endian(), "little");
    /// assert_eq!(cfg.target().env(), Some("msvc"));
    /// assert_eq!(cfg.target().family(), Some("windows"));
    /// assert_eq!(cfg.target().os(), "windows");
    /// assert_eq!(cfg.target().pointer_width(), "32");
    /// assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// but this is a common enough use-case that a specific method for
    /// obtaining the configuration of a target triple is provided with the
    /// [`Cfg::with_triple`] method,
    ///
    /// ```
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// let cfg = Cfg::with_triple("i686-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().arch(), "x86");
    /// assert_eq!(cfg.target().endian(), "little");
    /// assert_eq!(cfg.target().env(), Some("msvc"));
    /// assert_eq!(cfg.target().family(), Some("windows"));
    /// assert_eq!(cfg.target().os(), "windows");
    /// assert_eq!(cfg.target().pointer_width(), "32");
    /// assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// [`Cfg::with_triple`]: #method.with_triple
    /// [`std::process::Command::args`]: https://doc.rust-lang.org/std/process/struct.Command.html#method.args
    pub fn with_args<C, R, A, U>(cargo_args: C, rustc_args: R) -> Result<Self, Error>
    where
        C: IntoIterator<Item = A>,
        R: IntoIterator<Item = U>,
        A: AsRef<OsStr>,
        U: AsRef<OsStr>,
    {
        let output = Command::new(
            env::var(CARGO_VARIABLE)
                .map(PathBuf::from)
                .ok()
                .unwrap_or_else(|| PathBuf::from(CARGO)),
        )
        .arg(RUSTC)
        .args(cargo_args)
        .arg("--")
        .args(rustc_args)
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
                    "target_env" => {
                        env = {
                            let env = value.trim_matches('"').to_string();
                            if env.is_empty() {
                                None
                            } else {
                                Some(env)
                            }
                        }
                    }
                    "target_family" => {
                        family = {
                            let family = value.trim_matches('"').to_string();
                            if family.is_empty() {
                                None
                            } else {
                                Some(family)
                            }
                        }
                    }
                    "target_feature" => features.push(value.trim_matches('"').to_string()),
                    "target_os" => os = Some(value.trim_matches('"').to_string()),
                    "target_pointer_width" => {
                        pointer_width = Some(value.trim_matches('"').to_string())
                    }
                    "target_vendor" => {
                        vendor = {
                            let vendor = value.trim_matches('"').to_string();
                            if vendor.is_empty() {
                                None
                            } else {
                                Some(vendor)
                            }
                        }
                    }
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
                pointer_width: pointer_width
                    .ok_or_else(|| Error::MissingOutput("target_pointer_width"))?,
                vendor,
            },
        })
    }

    /// Any and all additional lines from the `cargo rustc -- --print cfg`
    /// output that are not recognized as target key-value pairs.
    ///
    /// These are any lines that were not recognized as target key-value lines,
    /// i.e. `key="value"`. Unlike the target key-value lines, any double
    /// quotes, `"`, are _not_ removed.
    pub fn extras(&self) -> &Vec<String> {
        &self.extras
    }

    /// All output that is prepended by the `target_` string.
    ///
    /// These are all the recognized target key-value lines, i.e.
    /// `target_<key>="<value>"`. The double quotes, `"` are removed for the values.
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// Consumes this configuration and converts it into the target
    /// configuration.
    ///
    /// The target configuration is all recognized key-value lines prepended
    /// with the `target_` string.
    pub fn into_target(self) -> Target {
        self.target
    }
}

/// A container for all lines from the output that are prefixed with the
/// `target_` string.
///
/// For more information about possible values and recognized key-value pairs,
/// see the [Rust Reference book] on [Conditional Compilation].
///
/// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
/// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
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
    /// The CPU architecture for the target configuration.
    ///
    /// This is the `target_arch` line in the output. See the [`target_arch`]
    /// section on [Conditional Compilation] for example values. The surrounding
    /// double quotes, `"`, of the raw output of the `cargo rustc -- --print
    /// cfg` command are removed.
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::with_triple("i686-pc-windows-gnu")?;
    /// assert_eq!(cfg.target().arch(), "x86");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`target_arch`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
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
    pub fn env(&self) -> Option<&str> {
        self.env.as_deref()
    }

    /// The family for the target configuration.
    ///
    /// This is the `target_family` line in the output. Example values include:
    /// `windows` or `unix`. If a family is not provided, then this will be
    /// `None`. The surrounding double quotes of the raw output from the `cargo
    /// rustc -- --print cfg` command are removed.
    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    /// A list of all features enabled for the target configuration.
    ///
    /// If a feature is _not_ enabled, then it will not be in the Vector.
    ///
    /// The surrounding double quotes of the raw output from the `cargo rustc --
    /// --print cfg` command are removed.
    pub fn features(&self) -> Vec<&str> {
        self.features.iter().map(std::ops::Deref::deref).collect()
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
    pub fn vendor(&self) -> Option<&str> {
        self.vendor.as_deref()
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
            Self::Command(output) => write!(
                f,
                "{:?}: {}",
                output,
                String::from_utf8_lossy(&output.stderr)
            ),
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
    use super::*;

    mod target {
        use super::*;

        #[test]
        fn x86_64_pc_windows_msvc_target_is_correct() {
            let target = Cfg::with_triple("x86_64-pc-windows-msvc").unwrap().into_target();
            assert_eq!(target.arch(), "x86_64");
            assert_eq!(target.endian(), "little");
            assert_eq!(target.env(), Some("msvc"));
            assert_eq!(target.family(), Some("windows"));
            assert_eq!(target.os(), "windows");
            assert_eq!(target.pointer_width(), "64");
            assert_eq!(target.vendor(), Some("pc"));
        }

        #[test]
        fn x86_64_pc_windows_gnu_target_is_correct() {
            let target = Cfg::with_triple("x86_64-pc-windows-gnu").unwrap().into_target();
            assert_eq!(target.arch(), "x86_64");
            assert_eq!(target.endian(), "little");
            assert_eq!(target.env(), Some("gnu"));
            assert_eq!(target.family(), Some("windows"));
            assert_eq!(target.os(), "windows");
            assert_eq!(target.pointer_width(), "64");
            assert_eq!(target.vendor(), Some("pc"));
        }

        #[test]
        fn i686_pc_windows_msvc_target_is_correct() {
            let target = Cfg::with_triple("i686-pc-windows-msvc").unwrap().into_target();
            assert_eq!(target.arch(), "x86");
            assert_eq!(target.endian(), "little");
            assert_eq!(target.env(), Some("msvc"));
            assert_eq!(target.family(), Some("windows"));
            assert_eq!(target.os(), "windows");
            assert_eq!(target.pointer_width(), "32");
            assert_eq!(target.vendor(), Some("pc"));
        }

        #[test]
        fn i686_pc_windows_gnu_target_is_correct() {
            let target = Cfg::with_triple("i686-pc-windows-gnu").unwrap().into_target();
            assert_eq!(target.arch(), "x86");
            assert_eq!(target.endian(), "little");
            assert_eq!(target.env(), Some("gnu"));
            assert_eq!(target.family(), Some("windows"));
            assert_eq!(target.os(), "windows");
            assert_eq!(target.pointer_width(), "32");
            assert_eq!(target.vendor(), Some("pc"));
        }

        #[test]
        fn x86_64_unknown_linux_gnu_target_is_correct() {
            let target = Cfg::with_triple("x86_64-unknown-linux-gnu").unwrap().into_target();
            assert_eq!(target.arch(), "x86_64");
            assert_eq!(target.endian(), "little");
            assert_eq!(target.env(), Some("gnu"));
            assert_eq!(target.family(), Some("unix"));
            assert_eq!(target.os(), "linux");
            assert_eq!(target.pointer_width(), "64");
            assert_eq!(target.vendor(), Some("unknown"));
        }

        #[test]
        fn i686_unknown_linux_gnu_target_is_correct() {
            let target = Cfg::with_triple("i686-unknown-linux-gnu").unwrap().into_target();
            assert_eq!(target.arch(), "x86");
            assert_eq!(target.endian(), "little");
            assert_eq!(target.env(), Some("gnu"));
            assert_eq!(target.family(), Some("unix"));
            assert_eq!(target.os(), "linux");
            assert_eq!(target.pointer_width(), "32");
            assert_eq!(target.vendor(), Some("unknown"));
        }

        #[test]
        fn x86_64_apple_darwin_target_is_correct() {
            let target = Cfg::with_triple("x86_64-apple-darwin").unwrap().into_target();
            assert_eq!(target.arch(), "x86_64");
            assert_eq!(target.endian(), "little");
            assert_eq!(target.env(), None);
            assert_eq!(target.family(), Some("unix"));
            assert_eq!(target.os(), "macos");
            assert_eq!(target.pointer_width(), "64");
            assert_eq!(target.vendor(), Some("apple"));
        }

        #[test]
        fn i686_apple_darwin_target_is_correct() {
            let target = Cfg::with_triple("i686-apple-darwin").unwrap().into_target();
            assert_eq!(target.arch(), "x86");
            assert_eq!(target.endian(), "little");
            assert_eq!(target.env(), None);
            assert_eq!(target.family(), Some("unix"));
            assert_eq!(target.os(), "macos");
            assert_eq!(target.pointer_width(), "32");
            assert_eq!(target.vendor(), Some("apple"));
        }
    }
}
