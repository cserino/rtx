use std::fs::{create_dir_all, File};
use std::io::Write;
use std::ops::Deref;

use color_eyre::eyre::Result;

use crate::cli::command::Command;
use crate::config::Config;
use crate::config::MissingRuntimeBehavior::{Prompt, Warn};
use crate::hash::hash_to_str;
use crate::output::Output;
use crate::{dirs, env};

/// [internal] This is an internal command that writes an envrc file
/// for direnv to consume.
#[derive(Debug, clap::Args)]
#[clap(verbatim_doc_comment, hide = true)]
pub struct Envrc {}

impl Command for Envrc {
    fn run(self, mut config: Config, out: &mut Output) -> Result<()> {
        if config.settings.missing_runtime_behavior == Prompt {
            config.settings.missing_runtime_behavior = Warn;
        }
        config.ensure_installed()?;
        let envrc_path = env::RTX_TMP_DIR
            .join("direnv")
            .join(hash_to_str(dirs::CURRENT.deref()) + ".envrc");

        // TODO: exit early if envrc_path exists and is up to date
        create_dir_all(envrc_path.parent().unwrap())?;
        let mut file = File::create(&envrc_path)?;

        writeln!(
            file,
            "### Do not edit. This was autogenerated by 'asdf direnv envrc' ###"
        )?;
        for cf in &config.config_files {
            writeln!(file, "watch_file {}", cf.to_string_lossy())?;
        }
        for (k, v) in config.env()? {
            writeln!(
                file,
                "export {}={}",
                shell_escape::unix::escape(k.into()),
                shell_escape::unix::escape(v.into()),
            )?;
        }
        for path in &config.list_paths()? {
            writeln!(file, "PATH_add {}", path.to_string_lossy())?;
        }

        rtxprintln!(out, "{}", envrc_path.to_string_lossy());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use insta::assert_display_snapshot;

    use crate::assert_cli;

    use super::*;

    #[test]
    fn test_direnv_envrc() {
        assert_cli!("install");
        let stdout = assert_cli!("direnv", "envrc");
        let envrc = fs::read_to_string(stdout.trim()).unwrap();
        let envrc = envrc.replace(dirs::HOME.to_string_lossy().as_ref(), "~");
        assert_display_snapshot!(envrc);
    }
}
