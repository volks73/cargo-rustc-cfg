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

//! The goal of this library, a.k.a. crate, is to provide access to the compiler
//! configuration at Cargo build time of a project for use with [third-party]
//! [Cargo custom subcommands] by running the `cargo rustc --lib -- --print cfg`
//! command and parsing its output. This library is _not_ recommended for [build
//! scripts] as the compiler configuration information is available via [Cargo
//! environment variables] that are passed to build scripts at run
//! time.
//!
//! If the Rust compiler (rustc) target is `x86_64-pc-windows-msvc`, then the
//! output from the `cargo rustc --lib -- --print cfg` command will look similar to
//! this:
//!
//! ```powershell
//! PS C:\Path\to\Rust\Project> cargo rustc --lib -- --print cfg
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
//! types for accessing the various values from the output. The values for any
//! lines containing a key-value pair and prepended by the `target_` string are
//! available in the [`Target`] type with the double quotes, `"`, removed. Any
//! lines that are not recognized and/or not a target key-value pair are stored
//! (unaltered) and can be obtained with the [`Cfg::extras`] method.
//!
//! The [`CargoRustcPrintCfg`] type can be used to customize the `cargo rustc --
//! --print cfg` command.
//!
//! # Examples
//!
//! Get the host configuration with Cargo modifications if the host is
//! `x86_64-pc-windows-msvc`:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc", target_vendor = "pc"))]
//! # mod x86_64_pc_windows_msvc {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::host()?;
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
//! If the host is `x86_64-pc-windows-gnu`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "gnu", target_vendor = "pc"))]
//! # mod x86_64_pc_windows_gnu {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::host()?;
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
//! If the host is `x86_64-unknown-linux-gnu`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
//! # mod x86_64_unknown_linux_gnu {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::host()?;
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
//! If the host is `x86_64-apple-darwin`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
//! # mod x86_64_apple_darwin {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::host()?;
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
//! If the host is `i686-pc-windows-msvc`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86", target_os = "windows", target_env = "msvc", target_vendor = "pc"))]
//! # mod i686_pc_windows_msvc {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::host()?;
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
//! If the host is `i686-pc-windows-gnu`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86", target_os = "windows", target_env = "gnu", target_vendor = "pc"))]
//! # mod i686_pc_windows_gnu {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::host()?;
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
//! If the host is `i686-unknown-linux-gnu`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86", target_os = "linux"))]
//! # mod i686_unknown_linux_gnu {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::host()?;
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
//! If the host is `i686-apple-darwin`, then:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # #[cfg(all(target_arch = "x86", target_os = "macos"))]
//! # mod i686_apple_darwin {
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::host()?;
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
//! Get the configuration for a rustc target that is not the host, i.e.
//! cross-compilation, using the [`CargoRustcPrintCfg`] type and the
//! [`rustc_target`] method:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = CargoRustcPrintCfg::default()
//!     .rustc_target("i686-pc-windows-msvc")
//!     .execute()?
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
//! The above use-case is relatively common, but it is tedious to
//! routinely use the [`CargoRustcPrintCfg`] builder. The [`Cfg::rustc_target`]
//! method is available as a shorthand for the previous example:
//!
//! ```
//! # extern crate cargo_rustc_cfg;
//! # use cargo_rustc_cfg::{Cfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = Cfg::rustc_target("i686-pc-windows-msvc")?;
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
//! Regardless of using the long or short form to specify a rustc target, the
//! rustc target must still be installed and available on the host system. It is
//! recommended to use [rustup] and the `rustc target add <TRIPLE>` command to
//! install rustc targets.
//!
//! [`Cfg`]: struct.Cfg.html
//! [`Target`]: struct.Target.html
//! [`Cfg::extras`]: struct.Cfg.html#method.extras
//! [`CargoRustcPrintCfg`]: struct.CargoRustcPrintCfg.html
//! [`rustc_target`]: struct.CargoRustcPrintCfg.html#rustc_target
//! [`Cfg::rustc_target`]: struct.Cfg.html#method.rustc_target
//! [Cargo]: https://doc.rust-lang.org/cargo/index.html
//! [third-party]: https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands
//! [Cargo custom subcommands]: https://doc.rust-lang.org/1.30.0/cargo/reference/external-tools.html#custom-subcommands
//! [build scripts]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
//! [Cargo environment variables]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
//! [rustup]: https://rust-lang.github.io/rustup/

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::{Command, Output};

/// The command line name of the Cargo application.
pub const CARGO: &str = "cargo";

/// The environment variable name for the Cargo appplication.
pub const CARGO_VARIABLE: &str = "CARGO";

/// The command line name of the Rust compiler subcommand for Cargo.
pub const RUSTC: &str = "rustc";

/// The various types of Cargo targets.
///
/// The default variant is [`Library`].
///
/// This should _not_ be confused with Rust compiler (rustc) targets, which are
/// typically represented by a "triple" string. Cargo targets are defined and
/// listed in a package's manifest (Cargo.toml).
///
/// [`Library`]: #enum.CargoTarget.Library
#[derive(Clone, Debug, PartialEq)]
pub enum CargoTarget {
    /// A `--bench <NAME>` Cargo target as defined in the package's manifest
    /// (Cargo.toml).
    Benchmark(OsString),
    /// A `--bin <NAME>` Cargo target as defined in the package's manifest
    /// (Cargo.toml).
    Binary(OsString),
    /// An `--example <NAME>` Cargo target as defined in the package's manifest
    /// (Cargo.toml).
    Example(OsString),
    /// The default Cargo target.
    Library,
    /// A `--test <NAME>` Cargo target as defined in the package's manifest
    /// (Cargo.toml).
    Test(OsString),
}

impl CargoTarget {
    /// Converts the Cargo target into the command line arguments for Cargo.
    pub fn to_args(&self) -> Vec<&OsStr> {
        match self {
            Self::Benchmark(name) => vec![OsStr::new("--bench"), name],
            Self::Binary(name) => vec![OsStr::new("--bin"), name],
            Self::Example(name) => vec![OsStr::new("--example"), name],
            Self::Library => vec![OsStr::new("--lib")],
            Self::Test(name) => vec![OsStr::new("--test"), name],
        }
    }

    /// Consumes the Cargo target to convert into the command line arguments for
    /// Cargo.
    pub fn into_args(self) -> Vec<OsString> {
        match self {
            Self::Benchmark(name) => vec![OsString::from("--bench"), name],
            Self::Binary(name) => vec![OsString::from("--bin"), name],
            Self::Example(name) => vec![OsString::from("--example"), name],
            Self::Library => vec![OsString::from("--lib")],
            Self::Test(name) => vec![OsString::from("--test"), name],
        }
    }
}

impl Default for CargoTarget {
    fn default() -> Self {
        Self::Library
    }
}

/// A builder type for the `cargo rustc -- --print cfg` command.
///
/// For reference, the default command signature:
///
/// ```text
/// cargo rustc --lib -- --print cfg
/// ```
///
/// and the more generic command signature represented by this type:
///
/// ```text
/// cargo <TOOLCHAIN> rustc <CARGO_ARGS> <CARGO_TARGET> <RUSTC_TARGET> -- <RUSTC_ARGS> --print cfg
/// ```
///
/// where `<TOOLCHAIN>` is replaced with the [`cargo_toolchain`] value, the
/// `<CARGO_ARGS>` is replaced with the [`cargo_args`] value, the
/// `<CARGO_TARGET>` is replaced with the [`cargo_target`] value, the
/// `<RUSTC_TARGET>` is replaced with the [`rustc_target`] value, and the
/// `<RUSTC_ARGS>` is replaced with the [`rustc_args`] value.
///
/// [`cargo_toolchain`]: #method.cargo_toolchain
/// [`cargo_args`]: #method.cargo_args
/// [`cargo_target`]: #method.cargo_target
/// [`rustc_target`]: #method.rustc_target
/// [`rustc_args`]: #method.rustc_args
#[derive(Clone, Debug, PartialEq)]
pub struct CargoRustcPrintCfg {
    cargo_args: Option<Vec<OsString>>,
    cargo_target: CargoTarget,
    cargo_toolchain: Option<OsString>,
    rustc_args: Option<Vec<OsString>>,
    rustc_target: Option<OsString>,
}

impl CargoRustcPrintCfg {
    /// Adds arguments to the Cargo command after the `rustc` subcommand but
    /// before the `<CARGO_TARGET>` and `<RUSTC_TARGET>` arguments.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc --lib -- --print cfg
    /// ```
    ///
    /// and this method adds arguments between `rustc` and `--lib` to yield:
    ///
    /// ```text
    /// cargo rustc <CARGO_ARGS> --lib -- --print cfg
    /// ```
    pub fn cargo_args<A, S>(&mut self, a: A) -> &mut Self
    where
        A: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.cargo_args = Some(a.into_iter().map(|s| s.as_ref().into()).collect());
        self
    }

    /// Specifies a single Cargo target.
    ///
    /// When passing arguments to the Rust compiler (rustc) using the `--` flag,
    /// only one _Cargo_ target can be used. By default, the library target will
    /// be used. Use this method to specify a specific Cargo target other than
    /// the library target.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc --lib -- --print cfg
    /// ```
    ///
    /// and this method replaces the `--lib` with `--bin <NAME>`, `--bench
    /// <NAME>`, `--example <NAME>`, or `--test <NAME>`, respectively.
    pub fn cargo_target(&mut self, t: CargoTarget) -> &mut Self
    {
        self.cargo_target = t;
        self
    }

    /// Specify a toolchain to use.
    ///
    /// The toolchain must be installed on the host system before specifying it
    /// with this method. It is recommended to install and manage various
    /// toolchains using the [`rustup`] application.
    ///
    /// The plus sign, `+`, is prepended automatically. Please do not include it
    /// as part of the toolchain value.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc --lib -- --print cfg
    /// ```
    ///
    /// and this method would add `+<TOOLCHAIN>` between `cargo` and `rustc` to yield:
    ///
    /// ```text
    /// cargo +<TOOLCHAIN> rustc --lib -- --print cfg
    /// ```
    ///
    /// [`rustup`]: https://rust-lang.github.io/rustup/
    pub fn cargo_toolchain<T>(&mut self, t: T) -> &mut Self
    where
        T: AsRef<OsStr>
    {
        self.cargo_toolchain = Some(t.as_ref().into());
        self
    }

    /// Adds arguments to the Cargo command after the `--` flag but
    /// before the `--print cfg` arguments.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc --lib -- --print cfg
    /// ```
    ///
    /// and this method adds arguments between `--` and `--print cfg` to yield:
    ///
    /// ```text
    /// cargo rustc --lib -- <RUSTC_ARGS> --print cfg
    /// ```
    pub fn rustc_args<A, S>(&mut self, a: A) -> &mut Self
    where
        A: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.rustc_args = Some(a.into_iter().map(|s| s.as_ref().into()).collect());
        self
    }

    /// Specify a Rust compiler (rustc) target via a target triple.
    ///
    /// The rustc target must be installed on the host system before specifying
    /// it with this method. It is recommended to install and manage targets for
    /// various toolchains using the [`rustup`] application.
    ///
    /// The `--target` argument is prepended automatically. Please do not include it
    /// as part of the target triple value.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc --lib -- --print cfg
    /// ```
    ///
    /// and this method would add `--target <RUSTC_TARGET>` between `--lib` and `--` to yield:
    ///
    /// ```text
    /// cargo rustc --lib --target <RUSTC_TARGET> -- --print cfg
    /// ```
    ///
    /// where `<RUSTC_TARGET>` is a target triple from the `rustc --print
    /// target-list` output..
    ///
    /// [`rustup`]: https://rust-lang.github.io/rustup/
    pub fn rustc_target<T>(&mut self, t: T) -> &mut Self
        where T: AsRef<OsStr>
    {
        self.rustc_target = Some(t.as_ref().into());
        self
    }

    /// This executes the `cargo rustc` subcommand with the appropriate options.
    ///
    /// For reference, the generic command signature:
    ///
    /// ```text
    /// `cargo <TOOLCHAIN> rustc <CARGO_ARGS> <CARGO_TARGET> <RUSTC_TARGET> -- <RUSTC_ARGS> --print cfg`
    /// ```
    ///
    /// where `<TOOLCHAIN>` is replaced
    /// with the [`cargo_toolchain`] value, the `<CARGO_ARGS>` is replaced with
    /// the [`cargo_args`] value, the `<CARGO_TARGET>` is replaced with the
    /// [`cargo_target`] value, the `<RUSTC_TARGET>` is appropriately replaced
    /// with `--target <RUSTC_TARGET>` from the [`rustc_target`] value, and the
    /// `<RUSTC_ARGS>` is replaced with the [`rustc_args`] value.
    ///
    /// # Examples
    ///
    /// If the host is a Windows target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc", target_vendor = "pc"))]
    /// # mod x86_64_pc_windows_msvc {
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = CargoRustcPrintCfg::default().execute()?;
    /// assert_eq!(cfg.target().arch(), "x86_64");
    /// assert_eq!(cfg.target().endian(), "little");
    /// assert_eq!(cfg.target().env(), Some("msvc"));
    /// assert_eq!(cfg.target().family(), Some("windows"));
    /// assert_eq!(cfg.target().os(), "windows");
    /// assert_eq!(cfg.target().pointer_width(), "64");
    /// assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    ///
    /// If the host is a Linux target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    /// # mod x86_64_unknown_linux_gnu {
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = CargoRustcPrintCfg::default().execute()?;
    /// assert_eq!(cfg.target().arch(), "x86_64");
    /// assert_eq!(cfg.target().endian(), "little");
    /// assert_eq!(cfg.target().env(), None);
    /// assert_eq!(cfg.target().family(), Some("unix"));
    /// assert_eq!(cfg.target().os(), "os");
    /// assert_eq!(cfg.target().pointer_width(), "64");
    /// assert_eq!(cfg.target().vendor(), Some("unknown"));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    ///
    /// If the host is an Apple target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    /// # mod x86_64_apple_darwin {
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = CargoRustcPrintCfg::default().execute()?;
    /// assert_eq!(cfg.target().arch(), "x86_64");
    /// assert_eq!(cfg.target().endian(), "little");
    /// assert_eq!(cfg.target().env(), None);
    /// assert_eq!(cfg.target().family(), Some("unix"));
    /// assert_eq!(cfg.target().os(), "os");
    /// assert_eq!(cfg.target().pointer_width(), "64");
    /// assert_eq!(cfg.target().vendor(), Some("apple"));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    ///
    /// [`cargo_toolchain`]: #method.cargo_toolchain
    /// [`cargo_args`]: #method.cargo_args
    /// [`cargo_target`]: #method.cargo_target
    /// [`rustc_target`]: #method.rustc_target
    /// [`rustc_args`]: #method.rustc_args
    pub fn execute(&self) -> Result<Cfg, Error> {
        let mut cmd = Command::new(
            env::var(CARGO_VARIABLE)
                .map(PathBuf::from)
                .ok()
                .unwrap_or_else(|| PathBuf::from(CARGO)),
        );
        if let Some(toolchain) = &self.cargo_toolchain {
            let mut arg = OsString::from("+");
            arg.push(toolchain);
            cmd.arg(arg);
        }
        cmd.arg(RUSTC);
        if let Some(cargo_args) = &self.cargo_args {
            cmd.args(cargo_args);
        }
        if let Some(rustc_target) = &self.rustc_target {
            cmd.arg("--target");
            cmd.arg(rustc_target);
        }
        cmd.arg("--");
        if let Some(rustc_args) = &self.rustc_args {
            cmd.args(rustc_args);
        }
        cmd.args(self.cargo_target.to_args());
        cmd.arg("--print");
        cmd.arg("cfg");
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

        Ok(Cfg {
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
}

impl Default for CargoRustcPrintCfg {
    fn default() -> Self {
        Self {
            cargo_args: None,
            cargo_target: CargoTarget::default(),
            cargo_toolchain: None,
            rustc_args: None,
            rustc_target: None,
        }
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
    /// Obtains the host configuration.
    ///
    /// This is a helper method for using the [`CargoRustcPrintCfg`], such that:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc", target_vendor = "pc"))]
    /// # mod x86_64_pc_windows_msvc {
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = CargoRustcPrintCfg::default().execute()?;
    /// # assert_eq!(cfg.target().arch(), "x86_64");
    /// # assert_eq!(cfg.target().endian(), "little");
    /// # assert_eq!(cfg.target().env(), Some("msvc"));
    /// # assert_eq!(cfg.target().family(), Some("windows"));
    /// # assert_eq!(cfg.target().os(), "windows");
    /// # assert_eq!(cfg.target().pointer_width(), "64");
    /// # assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    ///
    /// becomes:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc", target_vendor = "pc"))]
    /// # mod x86_64_pc_windows_msvc {
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::host()?;
    /// # assert_eq!(cfg.target().arch(), "x86_64");
    /// # assert_eq!(cfg.target().endian(), "little");
    /// # assert_eq!(cfg.target().env(), Some("msvc"));
    /// # assert_eq!(cfg.target().family(), Some("windows"));
    /// # assert_eq!(cfg.target().os(), "windows");
    /// # assert_eq!(cfg.target().pointer_width(), "64");
    /// # assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    ///
    /// [`CargoRustcPrintCfg`]: #struct.CargoRustcPrintCfg
    pub fn host() -> Result<Self, Error> {
        CargoRustcPrintCfg::default().execute()
    }

    /// Obtains a configuration for a Rust compiler (rustc) target.
    ///
    /// This is a helper method for using the [`CargoRustcPrintCfg`], such that:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = CargoRustcPrintCfg::default().rustc_target("x86_64-pc-windows-msvc").execute()?;
    /// # assert_eq!(cfg.target().arch(), "x86_64");
    /// # assert_eq!(cfg.target().endian(), "little");
    /// # assert_eq!(cfg.target().env(), Some("msvc"));
    /// # assert_eq!(cfg.target().family(), Some("windows"));
    /// # assert_eq!(cfg.target().os(), "windows");
    /// # assert_eq!(cfg.target().pointer_width(), "64");
    /// # assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// becomes:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// # assert_eq!(cfg.target().arch(), "x86_64");
    /// # assert_eq!(cfg.target().endian(), "little");
    /// # assert_eq!(cfg.target().env(), Some("msvc"));
    /// # assert_eq!(cfg.target().family(), Some("windows"));
    /// # assert_eq!(cfg.target().os(), "windows");
    /// # assert_eq!(cfg.target().pointer_width(), "64");
    /// # assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`CargoRustcPrintCfg`]: #struct.CargoRustcPrintCfg
    pub fn rustc_target<S>(t: S) -> Result<Self, Error>
    where
        S: AsRef<OsStr>
    {
        CargoRustcPrintCfg::default().rustc_target(t).execute()
    }

    /// Any and all additional lines from the output of the `cargo rustc --print
    /// cfg` command that are not recognized as target key-value pairs.
    ///
    /// These are any lines that were not recognized as target key-value lines,
    /// i.e. `key="value"`. Unlike the target key-value lines, any double
    /// quotes, `"`, are _not_ removed.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// assert!(cfg.extras().contains(&"debug_assertions"));
    /// assert!(cfg.extras().contains(&"windows"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn extras(&self) -> Vec<&str> {
        self.extras.iter().map(|s| &**s).collect()
    }

    /// All output that is prepended by the `target_` string.
    ///
    /// These are all the recognized target key-value lines, i.e.
    /// `target_<key>="<value>"`. The double quotes, `"` are removed for the
    /// values.
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
    /// The target's CPU architecture.
    ///
    /// This is the `target_arch` line in the output. See the [`target_arch`]
    /// section on [Conditional Compilation] in the [Rust Reference book] for
    /// example values. The surrounding double quotes, `"`, of the raw output of
    /// the `cargo rustc -- --print cfg` command are removed.
    ///
    /// # Examples
    ///
    /// For a 32-bit Intel x86 target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("i686-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().arch(), "x86");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a 64-bit Intel x86 target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().arch(), "x86_64");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a 32-bit ARM target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("thumbv7a-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().arch(), "arm");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a 64-bit ARM target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("aarch64-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().arch(), "aarch64");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`target_arch`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
    /// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
    pub fn arch(&self) -> &str {
        &self.arch
    }

    /// The target's CPU endianness.
    ///
    /// This is the `target_endian` line in the output. See the
    /// [`target_endian`] section on [Conditional Compilation] in the [Rust
    /// Reference book] for example values. The surrounding double quotes, `"`,
    /// of the raw output of the `cargo rustc -- --print cfg` command are
    /// removed.
    ///
    /// # Examples
    ///
    /// For a little endian target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().endian(), "little");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a big endian target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("sparc64-unknown-linux-gnu")?;
    /// assert_eq!(cfg.target().endian(), "big");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`target_endian`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
    /// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
    pub fn endian(&self) -> &str {
        &self.endian
    }

    /// The Application Binary Interface (ABI) or `libc` used by the target.
    ///
    /// This is the `target_env` line in the output. See the
    /// [`target_env`] section on [Conditional Compilation] in the [Rust
    /// Reference book] for example values. The surrounding double quotes, `"`,
    /// of the raw output of the `cargo rustc -- --print cfg` command are
    /// removed.
    ///
    /// This will return `None` if the `target_env` line is missing from the
    /// output or the value is the empty string, `""`.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-gnu")?;
    /// assert_eq!(cfg.target().env(), Some("gnu"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`target_env`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_env
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
    /// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
    pub fn env(&self) -> Option<&str> {
        self.env.as_deref()
    }

    /// The target's operating system family.
    ///
    /// This is the `target_family` line in the output. See the
    /// [`target_family`] section on [Conditional Compilation] in the [Rust
    /// Reference book] for example values. The surrounding double quotes, `"`,
    /// of the raw output of the `cargo rustc -- --print cfg` command are
    /// removed.
    ///
    /// This will return `None` if the `target_family` key-value pair was missing
    /// from the output or the value was the empty string, `""`.
    ///
    /// # Examples
    ///
    /// For a Windows target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().family(), Some("windows"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a Linux target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-unknown-linux-gnu")?;
    /// assert_eq!(cfg.target().family(), Some("unix"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For an Apple target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-apple-darwin")?;
    /// assert_eq!(cfg.target().family(), Some("unix"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`target_family`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_family
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
    /// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    /// The features enabled for a target's compilation.
    ///
    /// This is any `target_feature` line in the output. See the
    /// [`target_feature`] section on [Conditional Compilation] in the [Rust
    /// Reference book] for example values. The surrounding double quotes, `"`,
    /// of the raw output of the `cargo rustc -- --print cfg` command are
    /// removed.
    ///
    /// Compiler features are enabled and disabled using either the Rust
    /// compiler's (rustc) [`-C/--codegen`] command line option, the Cargo
    /// [`rustflags`] key-value [configuration], or the [`RUSTFLAGS`] environment
    /// variable supported by Cargo but not rustc.
    ///
    /// # Examples
    ///
    /// Using the [`RUSTFLAGS`] environment variable to add the static linking
    /// feature to the target's compiler configuration:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// std::env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// std::env::set_var("RUSTFLAGS", "");
    /// assert!(cfg.target().features().contains(&"crt-static"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Using the [`-C/--codegen`] command line option to add the static linking
    /// feature to the target's compiler configuration:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::with_args(
    ///     &["--target", "x86_64-pc-windows-msvc"],
    ///     &["-C", "target-feature=+crt-static"]
    /// )?;
    /// assert!(cfg.target().features().contains(&"crt-static"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`target_feature`]: https://doc.rust-lang.org/reference/conditioal-compilation.html#target_feature
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
    /// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
    /// [`-C/--codegen`]: https://doc.rust-lang.org/rustc/command-line-arguments.html#-c--codegen-code-generation-options
    /// [`rustflags`]: https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags
    /// [configuration]: https://doc.rust-lang.org/cargo/reference/config.html
    /// [`RUSTFLAGS`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html
    pub fn features(&self) -> Vec<&str> {
        self.features.iter().map(|s| &**s).collect()
    }

    /// The target's operating system.
    ///
    /// This is the `target_os` line in the output. See the
    /// [`target_os`] section on [Conditional Compilation] in the [Rust
    /// Reference book] for example values. The surrounding double quotes, `"`,
    /// of the raw output of the `cargo rustc -- --print cfg` command are
    /// removed.
    ///
    /// # Examples
    ///
    /// For a Windows target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().os(), "windows");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a Linux target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-unknown-linux-gnu")?;
    /// assert_eq!(cfg.target().os(), "linux");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For an Apple target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-apple-darwin")?;
    /// assert_eq!(cfg.target().os(), "macos");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note, the target's OS is different from the target's family for Apple
    /// targets.
    ///
    /// [`target_family`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_family
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
    /// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
    pub fn os(&self) -> &str {
        &self.os
    }

    /// The target's pointer width in bits, but as string.
    ///
    /// This is the `target_pointer_width` line in the output. See the
    /// [`target_pointer_width`] section on [Conditional Compilation] in the
    /// [Rust Reference book] for example values. The surrounding double quotes,
    /// `"`, of the raw output of the `cargo rustc -- --print cfg` command are
    /// removed.
    ///
    /// # Examples
    ///
    /// For a 64-bit target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().pointer_width(), "64");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a 32-bit target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("i686-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().pointer_width(), "32");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`target_pointer_width`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_pointer_width
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
    /// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
    pub fn pointer_width(&self) -> &str {
        &self.pointer_width
    }

    /// The target's vendor.
    ///
    /// This is the `target_vendor` line in the output. See the
    /// [`target_vendor`] section on [Conditional Compilation] in the [Rust
    /// Reference book] for example values. The surrounding double quotes, `"`,
    /// of the raw output of the `cargo rustc -- --print cfg` command are
    /// removed.
    ///
    /// This will return `None` if the `target_vendor` line is missing or the
    /// value is the empty string, `""`.
    ///
    /// # Examples
    ///
    /// For a Windows target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-pc-windows-msvc")?;
    /// assert_eq!(cfg.target().vendor(), Some("pc"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a Linux target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-unknown-linux-gnu")?;
    /// assert_eq!(cfg.target().vendor(), Some("unknown"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For an Apple target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{Cfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let cfg = Cfg::rustc_target("x86_64-apple-darwin")?;
    /// assert_eq!(cfg.target().vendor(), Some("apple"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`target_vendor`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_vendor
    /// [Conditional Compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
    /// [Rust Reference book]: https://doc.rust-lang.org/reference/introduction.html
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
