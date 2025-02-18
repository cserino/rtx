use atty::Stream;
use color_eyre::eyre::Result;
use indoc::formatdoc;
use once_cell::sync::Lazy;

use crate::cli::args::runtime::{RuntimeArg, RuntimeArgParser};
use crate::cli::command::Command;
use crate::config::{config_file, Config};
use crate::output::Output;
use crate::plugins::PluginName;
use crate::ui::color::Color;
use crate::{dirs, env};

/// sets global .tool-versions to include a specified runtime
///
/// then displays the contents of ~/.tool-versions
/// this file is `$HOME/.tool-versions` by default
/// use `rtx local` to set a runtime version locally in the current directory
#[derive(Debug, clap::Args)]
#[clap(verbatim_doc_comment, visible_alias = "g", after_long_help = AFTER_LONG_HELP.as_str())]
pub struct Global {
    /// runtime(s) to add to .tool-versions
    ///
    /// e.g.: nodejs@20
    /// if this is a single runtime with no version, the current value of the global
    /// .tool-versions will be displayed
    #[clap(value_parser = RuntimeArgParser, verbatim_doc_comment)]
    runtime: Option<Vec<RuntimeArg>>,

    /// save fuzzy match to .tool-versions
    /// e.g.: `rtx global --fuzzy nodejs@20` will save `nodejs 20` to .tool-versions,
    /// by default, it would save the exact version, e.g.: `nodejs 20.0.0`
    #[clap(long)]
    fuzzy: bool,

    /// remove the plugin(s) from ~/.tool-versions
    #[clap(long, value_name = "PLUGIN")]
    remove: Option<Vec<PluginName>>,
}

impl Command for Global {
    fn run(self, mut config: Config, out: &mut Output) -> Result<()> {
        let cf_path = dirs::HOME.join(env::RTX_DEFAULT_TOOL_VERSIONS_FILENAME.as_str());

        let mut cf = match cf_path.exists() {
            true => config_file::parse(&cf_path)?,
            false => config_file::init(&cf_path),
        };

        if let Some(plugins) = &self.remove {
            for plugin in plugins {
                cf.remove_plugin(plugin);
            }
        }
        if let Some(runtimes) = &self.runtime {
            let runtimes = RuntimeArg::double_runtime_condition(&runtimes.clone());
            if cf.display_runtime(out, &runtimes)? {
                return Ok(());
            }
            cf.add_runtimes(&mut config, &runtimes, self.fuzzy)?;
        }

        if self.runtime.is_some() || self.remove.is_some() {
            cf.save()?;
        }

        rtxprint!(out, "{}", cf.dump());

        Ok(())
    }
}

static COLOR: Lazy<Color> = Lazy::new(|| Color::new(Stream::Stdout));
static AFTER_LONG_HELP: Lazy<String> = Lazy::new(|| {
    formatdoc! {r#"
    {}
      # set the current version of nodejs to 20.x
      # will use a precise version (e.g.: 20.0.0) in .tool-versions file
      $ rtx global nodejs@20

      # set the current version of nodejs to 20.x
      # will use a fuzzy version (e.g.: 20) in .tool-versions file
      $ rtx global --fuzzy nodejs@20

      # show the current version of nodejs in ~/.tool-versions
      $ rtx global nodejs
      20.0.0
    "#, COLOR.header("Examples:")}
});

#[cfg(test)]
mod tests {
    use std::fs;

    use insta::assert_snapshot;
    use pretty_assertions::assert_str_eq;

    use crate::{assert_cli, assert_cli_err, dirs};

    #[test]
    fn test_global() {
        let cf_path = dirs::HOME.join(".tool-versions");
        let orig = fs::read_to_string(&cf_path).ok();
        let _ = fs::remove_file(&cf_path);

        assert_cli!("install", "shfmt@2");
        let stdout = assert_cli!("global", "shfmt@2");
        assert_snapshot!(stdout);
        let stdout = assert_cli!("global", "--fuzzy", "shfmt@2");
        assert_snapshot!(stdout);
        let stdout = assert_cli!("global", "--remove", "nodejs");
        assert_snapshot!(stdout);
        let stdout = assert_cli!("global", "tiny", "2");
        assert_snapshot!(stdout);

        // will output the current version(s)
        let stdout = assert_cli!("global", "tiny");
        assert_str_eq!(stdout, "2.1.0\n");

        // this plugin isn't installed
        let err = assert_cli_err!("global", "invalid-plugin");
        assert_str_eq!(
            err.to_string(),
            "no version set for invalid-plugin in ~/.tool-versions"
        );

        // can only request a version one plugin at a time
        let err = assert_cli_err!("global", "tiny", "dummy");
        assert_str_eq!(err.to_string(), "invalid input, specify a version for each runtime. Or just specify one runtime to print the current version");

        // this is just invalid
        let err = assert_cli_err!("global", "tiny", "dummy@latest");
        assert_str_eq!(err.to_string(), "invalid input, specify a version for each runtime. Or just specify one runtime to print the current version");

        if let Some(orig) = orig {
            fs::write(cf_path, orig).unwrap();
        }
    }
}
