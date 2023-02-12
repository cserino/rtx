use std::path::PathBuf;
use indexmap::IndexMap;
use itertools::Itertools;
use rayon::prelude::*;
use crate::{dirs, env, file};
use crate::config::{Config, Settings};
use crate::plugins::PluginName;
use crate::toolset::Toolset;

pub fn load_toolset(config: &Config) -> Toolset {
    let legacy_filenames = load_legacy_filenames(config)?;
    let config_paths = find_all_config_files(&legacy_filenames);
}

fn load_legacy_filenames(
    config: &Config
) -> IndexMap<String, PluginName> {
    if !config.settings.legacy_version_file {
        return Ok(IndexMap::new());
    }
    config
        .list_plugins()
        .into_par_iter()
        .filter_map::<_, Vec<(String, PluginName)>>(|plugin| {
            match plugin.legacy_filenames() {
                Ok(filenames) => Some(filenames.map(|f| (f, plugin.name.clone())).collect_vec()),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    return None;
                }
            }
        })
        .flatten()
        .collect::<IndexMap<String, PluginName>>();
}

fn find_all_config_files(legacy_filenames: &IndexMap<String, PluginName>) -> Vec<PathBuf> {
    let mut filenames = vec![
        // ".rtxrc.toml",
        // ".rtxrc",
        env::RTX_DEFAULT_TOOL_VERSIONS_FILENAME.as_str(),
    ];
    for filename in legacy_filenames.keys() {
        filenames.push(filename.as_str());
    }
    filenames.reverse();

    let mut config_files = file::FindUp::new(&dirs::CURRENT, &filenames).collect::<Vec<_>>();

    let home_config = dirs::HOME.join(env::RTX_DEFAULT_TOOL_VERSIONS_FILENAME.as_str());
    if home_config.is_file() {
        config_files.push(home_config);
    }

    config_files.into_iter().unique().collect()
}
