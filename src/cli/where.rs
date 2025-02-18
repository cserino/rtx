use atty::Stream;
use color_eyre::eyre::Result;
use indoc::formatdoc;
use once_cell::sync::Lazy;

use crate::cli::args::runtime::{RuntimeArg, RuntimeArgParser};
use crate::cli::command::Command;
use crate::config::Config;
use crate::errors::Error::VersionNotInstalled;
use crate::output::Output;
use crate::ui::color::Color;

/// Display the installation path for a runtime
///
/// Must be installed.
#[derive(Debug, clap::Args)]
#[clap(verbatim_doc_comment, after_long_help = AFTER_LONG_HELP.as_str(), hide = true)]
pub struct Where {
    /// runtime(s) to look up
    /// if "@<PREFIX>" is specified, it will show the latest installed version that matches the prefix
    /// otherwise, it will show the current, active installed version
    #[clap(required = true, value_parser = RuntimeArgParser)]
    runtime: RuntimeArg,

    /// the version prefix to use when querying the latest version
    /// same as the first argument after the "@"
    /// used for asdf compatibility
    #[clap(hide = true)]
    asdf_version: Option<String>,
}

impl Command for Where {
    fn run(self, mut config: Config, out: &mut Output) -> Result<()> {
        let version = config.resolve_runtime_arg(&self.runtime)?;
        let rtv = config.ts.list_installed_versions().into_iter().find(|rtv| {
            rtv.plugin.name == self.runtime.plugin && version.eq(&Some(rtv.version.clone()))
        });

        match rtv {
            Some(rtv) => {
                rtxprintln!(out, "{}", rtv.install_path.to_string_lossy());
                Ok(())
            }
            None => Err(VersionNotInstalled(
                self.runtime.plugin.to_string(),
                self.runtime.version.to_string(),
            ))?,
        }
    }
}

static COLOR: Lazy<Color> = Lazy::new(|| Color::new(Stream::Stdout));
static AFTER_LONG_HELP: Lazy<String> = Lazy::new(|| {
    formatdoc! {r#"
    {}
      # Show the latest installed version of nodejs
      # If it is is not installed, errors
      $ rtx where nodejs@20
      /Users/jdx/.local/share/rtx/installs/nodejs/20.0.0

      # Show the current, active install directory of nodejs
      # Errors if nodejs is not referenced in any .tool-version file
      $ rtx where nodejs
      /Users/jdx/.local/share/rtx/installs/nodejs/20.0.0
    "#, COLOR.header("Examples:")}
});

#[cfg(test)]
mod tests {
    use insta::assert_display_snapshot;
    use pretty_assertions::assert_str_eq;

    use crate::dirs;
    use crate::{assert_cli, assert_cli_err};

    #[test]
    fn test_where() {
        assert_cli!("plugin", "add", "shfmt");
        assert_cli!("install");
        let stdout = assert_cli!("where", "shfmt");
        assert_str_eq!(
            stdout.trim(),
            dirs::ROOT.join("installs/shfmt/3.5.1").to_string_lossy()
        );
    }

    #[test]
    fn test_where_alias() {
        assert_cli!("plugin", "add", "shfmt");
        assert_cli!("install", "shfmt@my/alias");
        let stdout = assert_cli!("where", "shfmt@my/alias");
        assert_str_eq!(
            stdout.trim(),
            dirs::ROOT.join("installs/shfmt/3.0.2").to_string_lossy()
        );
    }

    #[test]
    fn test_where_not_found() {
        let err = assert_cli_err!("where", "shfmt@1111");
        assert_display_snapshot!(err, @"[shfmt] version 1111 not installed");
    }
}
