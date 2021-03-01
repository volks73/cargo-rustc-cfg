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
//! The output may vary depending on the rustc target and development
//! environment.
//!
//! This crate parses the above output and provides the [`Cfg`] and [`Target`]
//! types for accessing the various values from the output. The values for any
//! lines containing a key-value pair and prepended by the `target_` string are
//! available in the [`Target`] type with the double quotes, `"`, removed. Any
//! lines that are not recognized and/or not a target key-value pair are stored
//! (unaltered) and can be obtained with the [`Cfg::extras`] method.
//!
//! The [`CargoRustcPrintCfg`] type can be used to customize the `cargo rustc
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
//! # use cargo_rustc_cfg::{CargoRustcPrintCfg, Error};
//! # fn main() -> std::result::Result<(), Error> {
//! let cfg = CargoRustcPrintCfg::default()
//!     .rustc_target("i686-pc-windows-msvc")
//!     .execute()?;
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

use std::{env, str::FromStr};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The command line name of the Cargo application.
pub const CARGO: &str = "cargo";

/// The environment variable name for the Cargo appplication.
pub const CARGO_VARIABLE: &str = "CARGO";

/// The command line name of the Rust compiler subcommand for Cargo.
pub const RUSTC: &str = "rustc";

/// A builder type for the `cargo rustc --print cfg` command.
///
/// For reference, the default command signature is:
///
/// ```text
/// cargo +nightly rustc -Z unstable-option -Z multitarget --print cfg
/// ```
///
/// and the more generic command signature represented by this type is:
///
/// ```text
/// cargo <TOOLCHAIN> rustc <CARGO_ARGS> <CARGO_TARGET> <RUSTC_TARGET> --print cfg -- <RUSTC_ARGS>
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
    cargo_args: Vec<OsString>,
    cargo_toolchain: Option<OsString>,
    manifest_path: Option<PathBuf>,
    rustc_args: Vec<OsString>,
    rustc_targets: Vec<OsString>,
}

impl CargoRustcPrintCfg {
    /// Adds arguments to the Cargo command after the `rustc` subcommand but
    /// before the `<CARGO_TARGET>` and `<RUSTC_TARGET>` arguments.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo +nightly rustc -Z unstable-options -Z multitarget --print cfg
    /// ```
    ///
    /// and this method adds arguments between `rustc` and `--print cfg` to yield:
    ///
    /// ```text
    /// cargo +nightly rustc -Z unstable-options -Z multitarget <CARGO_ARGS> --print cfg
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
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --print cfg
    /// ```
    ///
    /// and this method would add `+<TOOLCHAIN>` between `cargo` and `rustc` to yield:
    ///
    /// ```text
    /// cargo +<TOOLCHAIN> rustc -Z unstable-option -Z multitarget --print cfg
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
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --print cfg
    /// ```
    ///
    /// and this method adds the `--manifest-path` argument to yield:
    ///
    /// ```text
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --manifest-path <PATH> --print cfg
    /// ```
    ///
    /// where `<PATH>` is replaced with a path to a package's manifest
    /// (Cargo.toml).
    pub fn manifest_path<P>(&mut self, p: P) -> &mut Self
    where
        P: Into<PathBuf>
    {
        self.manifest_path = Some(p.into());
        self
    }

    /// Adds arguments to the Cargo command after the `--` flag.
    ///
    /// For reference, the default command is:
    ///
    /// ```text
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --print cfg
    /// ```
    ///
    /// and this method adds arguments after `--` to yield:
    ///
    /// ```text
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --print cfg -- <RUSTC_ARGS>
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
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --print cfg
    /// ```
    ///
    /// and this method would add `--target <RUSTC_TARGET>` to yield:
    ///
    /// ```text
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --target <RUSTC_TARGET> --print cfg
    /// ```
    ///
    /// where `<RUSTC_TARGET>` is a target triple from the `rustc --print
    /// target-list` output.
    ///
    /// [`rustup`]: https://rust-lang.github.io/rustup/
    pub fn rustc_target<T>(&mut self, t: T) -> &mut Self
    where
        T: AsRef<OsStr>,
    {
        self.rustc_targets.push(t.as_ref().into());
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
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --print cfg
    /// ```
    ///
    /// and this method would add `--target <RUSTC_TARGET>` to yield:
    ///
    /// ```text
    /// cargo +nightly rustc -Z unstable-option -Z multitarget --target <RUSTC_TARGET> --print cfg
    /// ```
    ///
    /// where `<RUSTC_TARGET>` is a target triple from the `rustc --print
    /// target-list` output.
    ///
    /// [`rustup`]: https://rust-lang.github.io/rustup/
    pub fn rustc_targets<T>(&mut self, t: &[T]) -> &mut Self
    where
        T: AsRef<OsStr>,
    {
        self.rustc_targets.append(&mut t.iter().map(|t| t.as_ref().into()).collect::<Vec<OsString>>());
        self
    }

    /// This executes the `cargo rustc` subcommand with the appropriate options.
    ///
    /// For reference, the generic command signature:
    ///
    /// ```text
    /// `cargo +<TOOLCHAIN> rustc -Z unstable-options -Z multitarget <CARGO_ARGS> <RUSTC_TARGETS> --print cfg -- <RUSTC_ARGS>`
    /// ```
    ///
    /// where `<TOOLCHAIN>` is replaced
    /// with the [`cargo_toolchain`] value, the `<CARGO_ARGS>` is replaced with
    /// the [`cargo_args`] value, the `<RUSTC_TARGETS>` is appropriately
    /// replaced with `--target <RUSTC_TARGET> for each specified target from
    /// the [`rustc_targets`] or [`rustc_target`] methods, and the
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
    /// [`rustc_targets`]: #method.rustc_targets
    /// [`rustc_target`]: #method.rustc_target
    /// [`rustc_args`]: #method.rustc_args
    pub fn execute(&self) -> Result<Vec<TargetRustcCfg>, Error> {
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
        } else {
            cmd.arg("+nightly");
        }
        cmd.arg(RUSTC);
        cmd.arg("-Z");
        cmd.arg("unstable-options");
        cmd.arg("-Z");
        cmd.arg("multitarget");
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
        String::from_utf8(output.stdout)?.split("").map(TargetRustcCfg::from_str).collect()
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

#[derive(Clone, Debug, PartialEq)]
pub struct TargetRustcCfg {
    cfgs: Vec<Cfg>
}

impl TargetRustcCfg {
    pub fn cfgs(&self) -> &[Cfg] {
        &self.cfgs
    }

    pub fn name_cfgs(&self) -> Vec<&Cfg> {
        self.cfgs.iter().filter(|c| c.is_name()).collect()
    }

    pub fn key_pair_cfgs(&self) -> Vec<&Cfg> {
        self.cfgs.iter().filter(|c| c.is_key_pair()).collect()
    }

    pub fn into_cfgs(self) -> Vec<Cfg> {
        self.cfgs
    }
}

impl FromStr for TargetRustcCfg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            cfgs: s.lines().map(|line| line.parse::<Cfg>()).collect::<Result<Vec<Cfg>, Error>>()?
        })
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
    /// sign , `=`, like `target_arch=x86_64`, etc.
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
            Cfg::KeyPair(..) => None
        }
    }

    /// Gets the key-value pair configuration.
    ///
    /// This will return `None` if the configuration is not a key-value pair
    /// configuration.
    pub fn key_pair(&self) -> Option<(&str, &str)> {
        match self {
            Cfg::Name(..) => None,
            Cfg::KeyPair(k, v) => Some((k, v))
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
            Cfg::KeyPair(..) => false
        }
    }

    /// Checks if this is a key-value pair configuration.
    ///
    /// Returns `true` if this is a key-value pair configuration; otherwise,
    /// this returns `false`.
    pub fn is_key_pair(&self) -> bool {
         match self {
            Cfg::Name(..) => false,
            Cfg::KeyPair(..) => true
        }
    }

    /// Gets the name configuration.
    ///
    /// This will return `None` if this is not a name configuration. Regardless,
    /// it will consume this configuration.
    pub fn into_name(self) -> Option<String> {
        match self {
            Cfg::Name(n) => Some(n),
            Cfg::KeyPair(..) => None
        }
    }

    /// Gets the key-value pair configuration.
    ///
    /// This will return `None` if this is not a key-value pair configuration.
    /// Regardless, it will consume this configuration.
    pub fn into_key_pair(self) -> Option<(String, String)> {
        match self {
            Cfg::Name(..) => None,
            Cfg::KeyPair(k, v) => Some((k, v))
        }
    }
}

impl FromStr for Cfg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("=") {
            let mut parts = s.split("=");
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                Ok(Cfg::KeyPair(String::from(key), value.trim_matches('"').to_string()))
            } else {
                Err(Error::Generic(format!("Could not parse '{}' into a key-value configuration pair", s)))
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
