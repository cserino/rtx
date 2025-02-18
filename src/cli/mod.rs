use atty::Stream;
use clap::{FromArgMatches, Subcommand};
use color_eyre::Result;
use indoc::{formatdoc, indoc};
use log::LevelFilter;
use once_cell::sync::Lazy;

use crate::cli::command::Command;
use crate::config::Config;
use crate::output::Output;
use crate::ui::color::Color;

mod activate;
mod alias;
pub mod args;
mod asdf;
pub mod command;
mod complete;
mod current;
mod deactivate;
mod direnv;
mod doctor;
mod env;
mod exec;
mod external;
mod global;
mod hook_env;
mod install;
mod latest;
mod local;
mod ls;
mod ls_remote;
mod plugins;
mod settings;
mod uninstall;
pub mod version;
mod r#where;

// render help
#[cfg(debug_assertions)]
mod render_help;

pub struct Cli {
    command: clap::Command,
    external_commands: Vec<clap::Command>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Activate(activate::Activate),
    Alias(alias::Alias),
    Asdf(asdf::Asdf),
    Complete(complete::Complete),
    Current(current::Current),
    Deactivate(deactivate::Deactivate),
    Direnv(direnv::Direnv),
    Doctor(doctor::Doctor),
    Env(env::Env),
    Exec(exec::Exec),
    Global(global::Global),
    HookEnv(hook_env::HookEnv),
    Install(install::Install),
    Latest(latest::Latest),
    Local(local::Local),
    Ls(ls::Ls),
    LsRemote(ls_remote::LsRemote),
    Plugins(plugins::Plugins),
    Settings(settings::Settings),
    Uninstall(uninstall::Uninstall),
    Version(version::Version),
    Where(r#where::Where),

    #[cfg(debug_assertions)]
    RenderHelp(render_help::RenderHelp),
}

impl Commands {
    pub fn run(self, config: Config, out: &mut Output) -> Result<()> {
        match self {
            Self::Activate(cmd) => cmd.run(config, out),
            Self::Alias(cmd) => cmd.run(config, out),
            Self::Asdf(cmd) => cmd.run(config, out),
            Self::Complete(cmd) => cmd.run(config, out),
            Self::Current(cmd) => cmd.run(config, out),
            Self::Deactivate(cmd) => cmd.run(config, out),
            Self::Direnv(cmd) => cmd.run(config, out),
            Self::Doctor(cmd) => cmd.run(config, out),
            Self::Env(cmd) => cmd.run(config, out),
            Self::Exec(cmd) => cmd.run(config, out),
            Self::Global(cmd) => cmd.run(config, out),
            Self::HookEnv(cmd) => cmd.run(config, out),
            Self::Install(cmd) => cmd.run(config, out),
            Self::Latest(cmd) => cmd.run(config, out),
            Self::Ls(cmd) => cmd.run(config, out),
            Self::LsRemote(cmd) => cmd.run(config, out),
            Self::Local(cmd) => cmd.run(config, out),
            Self::Plugins(cmd) => cmd.run(config, out),
            Self::Settings(cmd) => cmd.run(config, out),
            Self::Uninstall(cmd) => cmd.run(config, out),
            Self::Version(cmd) => cmd.run(config, out),
            Self::Where(cmd) => cmd.run(config, out),

            #[cfg(debug_assertions)]
            Self::RenderHelp(cmd) => cmd.run(config, out),
        }
    }
}

impl Cli {
    pub fn new() -> Self {
        Self {
            command: Self::command(),
            external_commands: vec![],
        }
    }

    pub fn new_with_external_commands(config: &Config) -> Result<Self> {
        let external_commands = external::commands(config)?;
        Ok(Self {
            command: Self::command().subcommands(external_commands.clone()),
            external_commands,
        })
    }

    pub fn command() -> clap::Command {
        Commands::augment_subcommands(
            clap::Command::new("rtx")
                .version(version::VERSION.to_string())
                .about(env!("CARGO_PKG_DESCRIPTION"))
                .long_about(LONG_ABOUT)
                .arg_required_else_help(true)
                .subcommand_required(true)
                .after_help(AFTER_HELP.as_str())
                .arg(args::log_level::LogLevel::arg())
                .arg(args::verbose::Verbose::arg()),
        )
    }

    // TODO: use this
    pub fn _parse_log_level(self, args: &Vec<String>) -> LevelFilter {
        let matches = self.command.get_matches_from(args);
        *matches.get_one::<LevelFilter>("log-level").unwrap()
    }

    pub fn run(self, mut config: Config, args: &Vec<String>, out: &mut Output) -> Result<()> {
        debug!("{}", &args.join(" "));
        if args[1..] == ["-v"] {
            // normally this would be considered --verbose
            return version::Version {}.run(config, out);
        }
        let matches = self.command.get_matches_from(args);
        if *matches.get_one::<u8>("verbose").unwrap() > 0 {
            config.settings.verbose = true;
        }
        if let Some((command, sub_m)) = matches.subcommand() {
            external::execute(&config, command, sub_m, self.external_commands)?;
        }
        Commands::from_arg_matches(&matches)?.run(config, out)
    }
}

static COLOR: Lazy<Color> = Lazy::new(|| Color::new(Stream::Stdout));

const LONG_ABOUT: &str = indoc! {"
rtx is a tool for managing runtime versions.

Use this to install a particular version of node or ruby for a project. Using `rtx activate`,
you can also have your shell automatically switch to the correct node and ruby versions when you
`cd` into the project's directory.

It's a replacement for tools like `nvm`, `nodenv`, `rbenv`, `rvm`, `chruby`, `pyenv`, etc. that
works for any language. It's also great for managing linters/tools like `jq` and `shellcheck`.

It is inspired by asdf and uses asdf's plugin ecosystem under the hood: https://asdf-vm.com/
"};

static AFTER_HELP: Lazy<String> = Lazy::new(|| {
    formatdoc! { "
    {}
      rtx install nodejs@20.0.0       Install a specific node version
      rtx install nodejs@20.0         Install a version matching a prefix
      rtx local nodejs@20             Use node-20.x in current project
      rtx global nodejs@20            Use node-20.x as default

      rtx install nodejs              Install the version specified in .tool-versions
      rtx local nodejs                Use latest node in current directory
      rtx global system               Use system node everywhere unless overridden

      rtx x nodejs@20 -- node app.js  Run `node app.js` with PATH pointing to node-20.x
", COLOR.header("Examples:")  }
});

#[cfg(test)]
pub mod tests {
    use crate::config::MissingRuntimeBehavior::AutoInstall;
    use crate::config::Settings;
    use crate::dirs;
    use crate::plugins::{Plugin, PluginName};

    use super::*;

    pub fn cli_run(args: &Vec<String>) -> Result<Output> {
        let config = Config::load()?;
        let mut out = Output::tracked();
        Cli::new_with_external_commands(&config)?.run(config, args, &mut out)?;

        Ok(out)
    }

    #[macro_export]
    macro_rules! assert_cli {
        ($($args:expr),+) => {{
            let args = &vec!["rtx".into(), $($args.into()),+];
            $crate::cli::tests::cli_run(args).unwrap().stdout.content
        }};
    }

    #[macro_export]
    macro_rules! assert_cli_err {
        ($($args:expr),+) => {{
            let args = &vec!["rtx".into(), $($args.into()),+];
            $crate::cli::tests::cli_run(args).unwrap_err()
        }};
    }

    pub fn ensure_plugin_installed(name: &str) {
        let settings = Settings {
            missing_runtime_behavior: AutoInstall,
            ..Settings::default()
        };
        Plugin::load_ensure_installed(&PluginName::from(name), &settings).unwrap();
    }

    pub fn grep(output: String, pattern: &str) -> String {
        output
            .split('\n')
            .find(|line| line.contains(pattern))
            .map(|line| line.to_string())
            .unwrap_or_default()
            .replace(dirs::HOME.to_string_lossy().as_ref(), "~")
    }
}
