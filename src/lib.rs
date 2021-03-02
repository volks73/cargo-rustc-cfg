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
//! [Cargo custom subcommands] by running the `cargo rustc --print cfg`
//! command and parsing its output. This library is _not_ recommended for [build
//! scripts] as the compiler configuration information is available via [Cargo
//! environment variables] that are passed to build scripts at run time.
//!
//! # Important
//!
//! This crate currently only works with the nightly toolchain. The nightly
//! toolchain can be installed using [`rustup`]:
//!
//! ```bash
//! $ rustup toolchain install nightly
//! ```
//!
//! or
//!
//! ```pwsh
//! PS C:\rustup toolchain install nightly
//! ```
//!
//! The default toolchain can be changed with `rustup` as well or the `+nightly`
//! toolchain argument can be added to the command invocation using the
//! [`cargo_toolchain`] method of the [`CargoRustcPrintCfg`] builder.
//!
//! # Background
//!
//! If the Rust compiler (rustc) target is `x86_64-pc-windows-msvc`, then the
//! output from the `cargo rustc --print cfg` command will look similar to
//! this:
//!
//! ```pwsh
//! PS C:\Path\to\Rust\Project> cargo rustc --print cfg
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
//! ```
//!
//! The output may vary depending on the rustc host and development
//! environment.
//!
//! This crate parses the above output and provides name or key-value pair
//! compiler configurations as the [`Cfg`] enum for each target rustc
//! configuration, [`RustcTargetCfg`].
//!
//! The [`CargoRustcPrintCfg`] type can be used to customize the `cargo rustc
//! --print cfg` command, but the [`host`], [`target`], and [`targets`]
//! functions should meet the majority of use cases and needs.
//!
//! [`Cfg`]: struct.Cfg.html
//! [`RustcTargetCfg`]: struct.RustcTargetCfg.html
//! [`CargoRustcPrintCfg`]: struct.CargoRustcPrintCfg.html
//! [`cargo_toolchain`]: struct.CargoRustcPrintCfg.html#cargo_toolchain
//! [`host`]: fn.host.html
//! [`target`]: fn.target.html
//! [`targets`]: fn.targets.html
//! [Cargo]: https://doc.rust-lang.org/cargo/index.html
//! [third-party]: https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands
//! [Cargo custom subcommands]: https://doc.rust-lang.org/1.30.0/cargo/reference/external-tools.html#custom-subcommands
//! [build scripts]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
//! [Cargo environment variables]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
//! [`rustup`]: https://doc.rust-lang.org/nightly/edition-guide/rust-2018/rustup-for-managing-rust-versions.html
//! [rustup]: https://rust-lang.github.io/rustup/

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::slice::Iter;
use std::{env, str::FromStr};

/// The command line name of the Cargo application.
pub const CARGO: &str = "cargo";

/// The environment variable name for the Cargo appplication.
pub const CARGO_VARIABLE: &str = "CARGO";

/// The command line name of the Rust compiler subcommand for Cargo.
pub const RUSTC: &str = "rustc";

/// Gets the compiler (rustc) configurations for the host.
///
/// # Examples
///
/// ```
/// # extern crate cargo_rustc_cfg;
/// # use cargo_rustc_cfg::Error;
/// # fn main() -> std::result::Result<(), Error> {
/// let host = cargo_rustc_cfg::host()?;
/// assert!(host.iter().count() > 0);
/// # Ok(())
/// # }
/// ```
pub fn host() -> Result<RustcTargetCfg, Error> {
    CargoRustcPrintCfg::default()
        .execute()?
        .pop()
        .ok_or_else(|| Error::from("The host compiler configuration does not exist"))
}

/// Gets the compiler (rustc) configurations for a specific target.
///
/// A compiler target's "triple" from the `rustc --print target-list` should be
/// used.
///
/// # Examples
///
/// ```
/// # extern crate cargo_rustc_cfg;
/// # use cargo_rustc_cfg::Error;
/// # fn main() -> std::result::Result<(), Error> {
/// let target = cargo_rustc_cfg::target("i686-unknown-linux-gnu")?;
/// assert_eq!(target.get("debug_assertions"), Some("debug_assertions"));
/// assert_eq!(target.get("target_arch"), Some("x86"));
/// assert_eq!(target.get("target_endian"), Some("little"));
/// assert_eq!(target.get("target_env"), Some("gnu"));
/// assert_eq!(target.get("target_family"), Some("unix"));
/// assert_eq!(target.get("target_os"), Some("linux"));
/// assert_eq!(target.get("target_pointer_width"), Some("32"));
/// assert_eq!(target.get("target_vendor"), Some("unknown"));
/// assert_eq!(target.get("unix"), Some("unix"));
/// # Ok(())
/// # }
/// ```
pub fn target<T>(triple: T) -> Result<RustcTargetCfg, Error>
where
    T: AsRef<OsStr>,
{
    CargoRustcPrintCfg::default()
        .rustc_target(triple)
        .execute()?
        .pop()
        .ok_or_else(|| Error::from("The target compiler configuration does not exist"))
}

/// Gets the compiler (rustc) configurations for multiple targets.
///
/// A compiler target's "triple" from the `rustc --print target-list` should be
/// used.
///
/// # Examples
///
/// ```
/// # extern crate cargo_rustc_cfg;
/// # use cargo_rustc_cfg::Error;
/// # fn main() -> std::result::Result<(), Error> {
/// let targets = cargo_rustc_cfg::targets(&["i686-pc-windows-msvc", "i686-pc-windows-gnu"])?;
/// let gnu = targets.get(0).expect("i686-pc-windows-gnu target");
/// let msvc = targets.get(1).expect("i686-pc-windows-msvc target");
///
/// assert_eq!(msvc.get("debug_assertions"), Some("debug_assertions"));
/// assert_eq!(msvc.get("target_arch"), Some("x86"));
/// assert_eq!(msvc.get("target_endian"), Some("little"));
/// assert_eq!(msvc.get("target_env"), Some("msvc"));
/// assert_eq!(msvc.get("target_family"), Some("windows"));
/// assert_eq!(msvc.get("target_os"), Some("windows"));
/// assert_eq!(msvc.get("target_pointer_width"), Some("32"));
/// assert_eq!(msvc.get("target_vendor"), Some("pc"));
/// assert_eq!(msvc.get("windows"), Some("windows"));
///
/// assert_eq!(gnu.get("debug_assertions"), Some("debug_assertions"));
/// assert_eq!(gnu.get("target_arch"), Some("x86"));
/// assert_eq!(gnu.get("target_endian"), Some("little"));
/// assert_eq!(gnu.get("target_env"), Some("gnu"));
/// assert_eq!(gnu.get("target_family"), Some("windows"));
/// assert_eq!(gnu.get("target_os"), Some("windows"));
/// assert_eq!(gnu.get("target_pointer_width"), Some("32"));
/// assert_eq!(gnu.get("target_vendor"), Some("pc"));
/// assert_eq!(gnu.get("windows"), Some("windows"));
/// # Ok(())
/// # }
/// ```
pub fn targets<T>(t: &[T]) -> Result<Vec<RustcTargetCfg>, Error>
where
    T: AsRef<OsStr>,
{
    CargoRustcPrintCfg::default().rustc_targets(t).execute()
}

/// A builder type for the `cargo rustc --print cfg` command.
///
/// For reference, the default command signature is:
///
/// ```text
/// cargo rustc -Z unstable-option --print cfg
/// ```
///
/// and the more generic command signature represented by this type is:
///
/// ```text
/// cargo +<TOOLCHAIN> rustc <CARGO_ARGS> <RUSTC_TARGET> --print cfg -- <RUSTC_ARGS>
/// ```
///
/// where `<TOOLCHAIN>` is replaced with the [`cargo_toolchain`] value, the
/// `<CARGO_ARGS>` is replaced with the [`cargo_args`] value, the
/// `<RUSTC_TARGET>` is replaced with the [`rustc_target`] value, and the
/// `<RUSTC_ARGS>` is replaced with the [`rustc_args`] value.
///
/// [`cargo_toolchain`]: #method.cargo_toolchain
/// [`cargo_args`]: #method.cargo_args
/// [`rustc_target`]: #method.rustc_target
/// [`rustc_args`]: #method.rustc_args
#[derive(Clone, Debug, PartialEq)]
pub struct CargoRustcPrintCfg {
    cargo_args: Vec<OsString>,
    cargo_toolchain: Option<OsString>,
    manifest_path: Option<PathBuf>,
    rustc_args: Vec<OsString>,
    rustc_targets: Vec<OsString>,
}

impl CargoRustcPrintCfg {
    /// Adds arguments to the Cargo command after the `rustc` subcommand but
    /// before `--print cfg` argument.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc --print cfg
    /// ```
    ///
    /// and this method adds arguments between `rustc` and `--print cfg` to yield:
    ///
    /// ```text
    /// cargo rustc <CARGO_ARGS> --print cfg
    /// ```
    ///
    /// # Examples
    ///
    /// Adding the `-Z unstable-options` argument to the command. Note, the `-Z
    /// unstable-options` is only available with the nightly channel:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let host = CargoRustcPrintCfg::default()
    ///     .cargo_args(&["-Z", "unstable-options"])
    ///     .execute()?
    ///     .pop()
    ///     .expect("Compiler confugiration");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Adding the `-Z multitarget` argument to the command. Note, the `-Z
    /// multitarget` is only available with the nightly channel:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let host = CargoRustcPrintCfg::default()
    ///     .cargo_args(&["-Z", "multitarget"])
    ///     .execute()?
    ///     .pop()
    ///     .expect("Compiler confugiration");
    /// # Ok(())
    /// # }
    /// ```
    pub fn cargo_args<A, S>(&mut self, a: A) -> &mut Self
    where
        A: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cargo_args = a.into_iter().map(|s| s.as_ref().into()).collect();
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
    /// cargo rustc -Z unstable-option --print cfg
    /// ```
    ///
    /// and this method would add `+<TOOLCHAIN>` between `cargo` and `rustc` to
    /// yield:
    ///
    /// ```text
    /// cargo +<TOOLCHAIN> rustc -Z unstable-option --print cfg
    /// ```
    ///
    /// [`rustup`]: https://rust-lang.github.io/rustup/
    pub fn cargo_toolchain<T>(&mut self, t: T) -> &mut Self
    where
        T: AsRef<OsStr>,
    {
        self.cargo_toolchain = Some(t.as_ref().into());
        self
    }

    /// Sets the path to a package's manifest (Cargo.toml) to determine the
    /// compiler configuration.
    ///
    /// The default assumes the current working directory (CWD) contains the
    /// package's manifest, i.e. at the root directory of the Cargo project. Use
    /// this method to override this default and determine the compiler
    /// configuration for a Cargo-based project outside of the CWD.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc -Z unstable-option --print cfg
    /// ```
    ///
    /// and this method adds the `--manifest-path` argument to yield:
    ///
    /// ```text
    /// cargo rustc -Z unstable-option --manifest-path <PATH> --print cfg
    /// ```
    ///
    /// where `<PATH>` is replaced with a path to a package's manifest
    /// (Cargo.toml).
    pub fn manifest_path<P>(&mut self, p: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        self.manifest_path = Some(p.into());
        self
    }

    /// Adds arguments to the Cargo command after the `--` flag.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc -Z unstable-option --print cfg
    /// ```
    ///
    /// and this method adds arguments after `--` to yield:
    ///
    /// ```text
    /// cargo rustc -Z unstable-option --print cfg -- <RUSTC_ARGS>
    /// ```
    pub fn rustc_args<A, S>(&mut self, a: A) -> &mut Self
    where
        A: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.rustc_args = a.into_iter().map(|s| s.as_ref().into()).collect();
        self
    }

    /// Specify a Rust compiler (rustc) target via a target triple.
    ///
    /// The `--target` argument is prepended automatically. Please do not include it
    /// as part of the target triple value.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc -Z unstable-option --print cfg
    /// ```
    ///
    /// and this method would add `--target <RUSTC_TARGET>` to yield:
    ///
    /// ```text
    /// cargo rustc -Z unstable-option --target <RUSTC_TARGET> --print cfg
    /// ```
    ///
    /// where `<RUSTC_TARGET>` is a target triple from the `rustc --print
    /// target-list` output.
    ///
    /// If more than one rustc target is specified, the `-Z multitarget` option
    /// will automatically be added to the command invocation.
    ///
    /// # Examples
    ///
    /// The compiler configuration for the i686-unknown-linux-gnu target regardless of the host.
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let target = CargoRustcPrintCfg::default()
    ///     .rustc_target("i686-unknown-linux-gnu")
    ///     .execute()?
    ///     .pop()
    ///     .expect("Compiler configuration");
    /// assert_eq!(target.get("debug_assertions"), Some("debug_assertions"));
    /// assert_eq!(target.get("target_arch"), Some("x86"));
    /// assert_eq!(target.get("target_endian"), Some("little"));
    /// assert_eq!(target.get("target_env"), Some("gnu"));
    /// assert_eq!(target.get("target_family"), Some("unix"));
    /// assert_eq!(target.get("target_os"), Some("linux"));
    /// assert_eq!(target.get("target_pointer_width"), Some("32"));
    /// assert_eq!(target.get("target_vendor"), Some("unknown"));
    /// assert_eq!(target.get("unix"), Some("unix"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn rustc_target<T>(&mut self, t: T) -> &mut Self
    where
        T: AsRef<OsStr>,
    {
        self.rustc_targets.push(t.as_ref().into());
        self
    }

    /// Specify multiple Rust compiler (rustc) targets via target triples.
    ///
    /// The `--target` argument is prepended automatically. Please do not include it
    /// as part of the target triple value.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo rustc --print cfg
    /// ```
    ///
    /// and this method would add multiple `--target <RUSTC_TARGET>` to yield:
    ///
    /// ```text
    /// cargo rustc <RUSTC_TARGET_1> --target <RUSTC_TARGET_2> --print cfg
    /// ```
    ///
    /// where `<RUSTC_TARGET>` is a target triple from the `rustc --print
    /// target-list` output.
    ///
    /// If multiple rustc targets are specified, then the `-Z multitarget`
    /// option must be added using the [`cargo_args`] method or specified in the
    /// project's `.cargo/config.toml` file as follows:
    ///
    /// ```toml
    /// [unstable]
    /// multitarget = true
    /// ```
    ///
    /// [`cargo_args`]: #method.cargo_args
    pub fn rustc_targets<T>(&mut self, t: &[T]) -> &mut Self
    where
        T: AsRef<OsStr>,
    {
        self.rustc_targets.append(
            &mut t
                .iter()
                .map(|t| t.as_ref().into())
                .collect::<Vec<OsString>>(),
        );
        self
    }

    /// This executes the `cargo rustc` subcommand with the appropriate options.
    ///
    /// For reference, the generic command signature:
    ///
    /// ```text
    /// `cargo +<TOOLCHAIN> rustc -Z unstable-options <CARGO_ARGS> <RUSTC_TARGETS> --print cfg -- <RUSTC_ARGS>`
    /// ```
    ///
    /// where `<TOOLCHAIN>` is replaced with the [`cargo_toolchain`] value, the
    /// `<CARGO_ARGS>` is replaced with the [`cargo_args`] value, the
    /// `<RUSTC_TARGETS>` is appropriately replaced with `--target
    /// <RUSTC_TARGET> for each specified target from the [`rustc_targets`] or
    /// [`rustc_target`] methods, and the `<RUSTC_ARGS>` is replaced with the
    /// [`rustc_args`] value.
    ///
    /// # Examples
    ///
    /// For a Windows target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let windows = CargoRustcPrintCfg::default()
    ///     .rustc_target("x86_64-pc-windows-msvc")
    ///     .execute()?
    ///     .pop()
    ///     .expect("Compiler configuration");
    /// assert_eq!(windows.get("debug_assertions"), Some("debug_assertions"));
    /// assert_eq!(windows.get("target_arch"), Some("x86_64"));
    /// assert_eq!(windows.get("target_endian"), Some("little"));
    /// assert_eq!(windows.get("target_env"), Some("msvc"));
    /// assert_eq!(windows.get("target_family"), Some("windows"));
    /// assert_eq!(windows.get("target_os"), Some("windows"));
    /// assert_eq!(windows.get("target_pointer_width"), Some("64"));
    /// assert_eq!(windows.get("target_vendor"), Some("pc"));
    /// assert_eq!(windows.get("windows"), Some("windows"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For a Linux target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let linux = CargoRustcPrintCfg::default()
    ///     .rustc_target("x86_64-unknown-linux-gnu")
    ///     .execute()?
    ///     .pop()
    ///     .expect("Compiler configuration");
    /// assert_eq!(linux.get("debug_assertions"), Some("debug_assertions"));
    /// assert_eq!(linux.get("target_arch"), Some("x86_64"));
    /// assert_eq!(linux.get("target_endian"), Some("little"));
    /// assert_eq!(linux.get("target_env"), Some("gnu"));
    /// assert_eq!(linux.get("target_family"), Some("unix"));
    /// assert_eq!(linux.get("target_os"), Some("linux"));
    /// assert_eq!(linux.get("target_pointer_width"), Some("64"));
    /// assert_eq!(linux.get("target_vendor"), Some("unknown"));
    /// assert_eq!(linux.get("unix"), Some("unix"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// If the host is an macOS target:
    ///
    /// ```
    /// # extern crate cargo_rustc_cfg;
    /// # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
    /// # fn main() -> std::result::Result<(), Error> {
    /// let macos = CargoRustcPrintCfg::default()
    ///     .rustc_target("x86_64-apple-darwin")
    ///     .execute()?
    ///     .pop()
    ///     .expect("Compiler configuration");
    /// assert_eq!(macos.get("debug_assertions"), Some("debug_assertions"));
    /// assert_eq!(macos.get("target_arch"), Some("x86_64"));
    /// assert_eq!(macos.get("target_endian"), Some("little"));
    /// assert_eq!(macos.get("target_env"), Some(""));
    /// assert_eq!(macos.get("target_family"), Some("unix"));
    /// assert_eq!(macos.get("target_os"), Some("macos"));
    /// assert_eq!(macos.get("target_pointer_width"), Some("64"));
    /// assert_eq!(macos.get("target_vendor"), Some("apple"));
    /// assert_eq!(macos.get("unix"), Some("unix"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`cargo_toolchain`]: #method.cargo_toolchain
    /// [`cargo_args`]: #method.cargo_args
    /// [`rustc_targets`]: #method.rustc_targets
    /// [`rustc_target`]: #method.rustc_target
    /// [`rustc_args`]: #method.rustc_args
    pub fn execute(&self) -> Result<Vec<RustcTargetCfg>, Error> {
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
        if let Some(manifest_path) = &self.manifest_path {
            cmd.arg("--manifest-path");
            cmd.arg(manifest_path);
        }
        for rustc_target in &self.rustc_targets {
            cmd.arg("--target");
            cmd.arg(rustc_target);
        }
        cmd.arg("--print");
        cmd.arg("cfg");
        if !self.rustc_args.is_empty() {
            cmd.arg("--");
            cmd.args(&self.rustc_args);
        }
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(Error::Command(output));
        }
        let stdout = String::from_utf8(output.stdout)?;
        let mut cfgs = Vec::new();
        let mut targets = Vec::new();
        for line in stdout.lines() {
            if line.is_empty() {
                targets.push(RustcTargetCfg(cfgs.drain(..).collect()));
            } else {
                cfgs.push(line.parse::<Cfg>()?);
            }
        }
        targets.push(RustcTargetCfg(cfgs));
        Ok(targets)
    }
}

impl Default for CargoRustcPrintCfg {
    fn default() -> Self {
        Self {
            cargo_args: Vec::new(),
            cargo_toolchain: None,
            manifest_path: None,
            rustc_args: Vec::new(),
            rustc_targets: Vec::new(),
        }
    }
}

/// A container for the compiler (rustc) configurations for a specific compiler
/// target.
#[derive(Clone, Debug, PartialEq)]
pub struct RustcTargetCfg(Vec<Cfg>);

impl RustcTargetCfg {
    /// An iterator visiting all compiler configurations for the compiler
    /// (rustc) target.
    pub fn iter(&self) -> Iter<Cfg> {
        self.0.iter()
    }

    /// Returns a reference to the compiler configuration value with the corresponding identifier (ID).
    ///
    /// In the case of a name compiler configuration, the name is the value. If
    /// the compiler configuration is a key-value pair, the value will be
    /// returned if the key matches the ID.
    pub fn get(&self, id: &str) -> Option<&str> {
        self.0.iter().find_map(|c| match c {
            Cfg::Name(n) => {
                if n == id {
                    Some(n.as_ref())
                } else {
                    None
                }
            }
            Cfg::KeyPair(k, v) => {
                if k == id {
                    Some(v.as_ref())
                } else {
                    None
                }
            }
        })
    }

    /// Returns `true` if a compiler configuration matches the corresponding identifier (ID).
    ///
    /// In the case of a name compiler configuration, the name is the ID. If the
    /// compiler configuration is a key-value pair, then this will return `true`
    /// if either the key or the value match the ID.
    pub fn has(&self, id: &str) -> bool {
        self.0.iter().any(|c| match c {
            Cfg::Name(n) => n == id,
            Cfg::KeyPair(k, v) => k == id || v == id,
        })
    }
}

impl FromStr for RustcTargetCfg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.lines()
                .map(|line| line.parse::<Cfg>())
                .collect::<Result<Vec<Cfg>, Error>>()?,
        ))
    }
}

impl IntoIterator for RustcTargetCfg {
    type Item = Cfg;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for RustcTargetCfg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for cfg in &self.0 {
            cfg.fmt(f)?;
        }
        Ok(())
    }
}

/// A compiler (rustc) configuration statement, or line, from the output of the
/// `cargo rustc --print cfg`.
///
/// A compiler configuration is either a `Name` configuration, like "unix" or
/// "debug_assertions", or a `KeyPair` configuration, like `target_os="windows"`.
#[derive(Clone, Debug, PartialEq)]
pub enum Cfg {
    /// A compiler configuration like `unix`, `windows`, `debug_assertions`, etc.
    Name(String),
    /// A compiler configuration with a key and value separated by the equal
    /// sign , `=`, like `target_arch="x86_64"`, etc. The surrounding double
    /// quotes are removed.
    KeyPair(String, String),
}

impl Cfg {
    /// Gets the name configuration.
    ///
    /// This will return `None` if the configuration is not a name
    /// configuration.
    pub fn name(&self) -> Option<&str> {
        match self {
            Cfg::Name(n) => Some(n),
            Cfg::KeyPair(..) => None,
        }
    }

    /// Gets the key-value pair configuration.
    ///
    /// This will return `None` if the configuration is not a key-value pair
    /// configuration.
    pub fn key_pair(&self) -> Option<(&str, &str)> {
        match self {
            Cfg::Name(..) => None,
            Cfg::KeyPair(k, v) => Some((k, v)),
        }
    }

    /// Gets the key part of the key-value pair configuration.
    ///
    /// This will return `None` if the configuration is not a key-value pair
    /// configuration.
    pub fn key(&self) -> Option<&str> {
        match self {
            Cfg::Name(..) => None,
            Cfg::KeyPair(k, ..) => Some(k),
        }
    }

    /// Gets the value part of the key-value pair configuration.
    ///
    /// This will return `None` if the configuration is not a key-value pair
    /// configuration.
    pub fn value(&self) -> Option<&str> {
        match self {
            Cfg::Name(..) => None,
            Cfg::KeyPair(.., v) => Some(v),
        }
    }

    /// Checks if this is a name configuration.
    ///
    /// Returns `true` if this is a name configuration; otherwise, this returns
    /// `false`.
    pub fn is_name(&self) -> bool {
        match self {
            Cfg::Name(..) => true,
            Cfg::KeyPair(..) => false,
        }
    }

    /// Checks if this is a key-value pair configuration.
    ///
    /// Returns `true` if this is a key-value pair configuration; otherwise,
    /// this returns `false`.
    pub fn is_key_pair(&self) -> bool {
        match self {
            Cfg::Name(..) => false,
            Cfg::KeyPair(..) => true,
        }
    }

    /// Gets the name configuration.
    ///
    /// This will return `None` if this is not a name configuration. Regardless,
    /// it will consume this configuration.
    pub fn into_name(self) -> Option<String> {
        match self {
            Cfg::Name(n) => Some(n),
            Cfg::KeyPair(..) => None,
        }
    }

    /// Gets the key-value pair configuration.
    ///
    /// This will return `None` if this is not a key-value pair configuration.
    /// Regardless, it will consume this configuration.
    pub fn into_key_pair(self) -> Option<(String, String)> {
        match self {
            Cfg::Name(..) => None,
            Cfg::KeyPair(k, v) => Some((k, v)),
        }
    }
}

impl FromStr for Cfg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('=') {
            let mut parts = s.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                Ok(Cfg::KeyPair(
                    String::from(key),
                    value.trim_matches('"').to_string(),
                ))
            } else {
                Err(Error::Generic(format!(
                    "Could not parse '{}' into a key-value configuration pair",
                    s
                )))
            }
        } else {
            Ok(Cfg::Name(String::from(s)))
        }
    }
}

impl fmt::Display for Cfg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Cfg::Name(ref s) => s.fmt(f),
            Cfg::KeyPair(ref k, ref v) => write!(f, "{} = \"{}\"", k, v),
        }
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
            Self::FromUtf8(err) => err.fmt(f),
            Self::Generic(msg) => write!(f, "{}", msg),
            Self::Io(err) => err.fmt(f),
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
