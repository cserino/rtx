use color_eyre::eyre::Result;
use std::ffi::{OsStr, OsString};
use std::fmt::Display;

use clap::{Arg, Command, Error};
use regex::Regex;

use crate::plugins::PluginName;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RuntimeArg {
    pub plugin: PluginName,
    pub version: RuntimeArgVersion,
}

/// The type of runtime argument
/// Generally, these are in the form of `plugin@version` that's "Version"
/// but there are some alternatives like `plugin@ref:sha` or `plugin@path:/path/to/runtime`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum RuntimeArgVersion {
    /// Nothing was specified, e.g.: `nodejs`
    None,
    /// references a version, version prefix, or alias
    /// e.g.: `nodejs@20`, `nodejs@latest`, `nodejs@lts`
    Version(String),
    /// use the system runtime already on PATH
    /// e.g.: `nodejs@system`
    System,
    // /// build runtime from source at this VCS sha
    // rtx currently does not support this, see https://github.com/jdxcode/rtx/issues/98
    // Ref,
    // /// runtime is in a local directory, not managed by rtx
    // Path,
}

impl RuntimeArg {
    pub fn parse(input: &str) -> Self {
        match input.split_once('@') {
            Some((plugin, "system")) => Self {
                plugin: plugin.into(),
                version: RuntimeArgVersion::System,
            },
            Some((plugin, version)) => Self {
                plugin: plugin.into(),
                version: RuntimeArgVersion::Version(version.into()),
            },
            None => Self {
                plugin: input.into(),
                version: RuntimeArgVersion::None,
            },
        }
    }

    /// this handles the case where the user typed in:
    /// rtx local nodejs 20.0.0
    /// instead of
    /// rtx local nodejs@20.0.0
    ///
    /// We can detect this, and we know what they meant, so make it work the way
    /// they expected.
    pub fn double_runtime_condition(runtimes: &[RuntimeArg]) -> Vec<RuntimeArg> {
        let mut runtimes = runtimes.to_vec();
        if runtimes.len() == 2 {
            let re: &Regex = regex!(r"^\d+(\.\d+)?(\.\d+)?$");
            let a = runtimes[0].clone();
            let b = runtimes[1].clone();
            if matches!(a.version, RuntimeArgVersion::None)
                && matches!(b.version, RuntimeArgVersion::None)
                && re.is_match(&b.plugin)
            {
                runtimes[1].version = RuntimeArgVersion::Version(b.plugin);
                runtimes[1].plugin = a.plugin;
                runtimes.remove(0);
            }
        }
        runtimes
    }
}

impl Display for RuntimeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.version {
            RuntimeArgVersion::System => write!(f, "{}@system", self.plugin),
            RuntimeArgVersion::Version(version) => write!(f, "{}@{}", self.plugin, version),
            RuntimeArgVersion::None => write!(f, "{}", self.plugin),
        }
    }
}

impl Display for RuntimeArgVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeArgVersion::System => write!(f, "system"),
            RuntimeArgVersion::Version(version) => write!(f, "{version}"),
            RuntimeArgVersion::None => write!(f, "current"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeArgParser;

impl clap::builder::TypedValueParser for RuntimeArgParser {
    type Value = RuntimeArg;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        self.parse(cmd, arg, value.to_os_string())
    }

    fn parse(
        &self,
        _cmd: &Command,
        _arg: Option<&Arg>,
        value: OsString,
    ) -> Result<Self::Value, Error> {
        Ok(RuntimeArg::parse(&value.to_string_lossy()))
    }
}
