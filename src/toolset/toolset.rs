use crate::plugins::PluginName;
use crate::toolset::tool_version_list::ToolVersionList;
use crate::toolset::{ToolSource, ToolVersion};
use indexmap::IndexMap;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

/// a toolset is a collection of tools for various plugins
///
/// one example is a .tool-versions file
/// the idea is that we start with an empty toolset, then
/// merge in other toolsets from various sources
#[derive(Debug)]
pub struct Toolset {
    versions: IndexMap<PluginName, ToolVersionList>,
    source: ToolSource,
}

impl Toolset {
    pub fn new(source: ToolSource) -> Self {
        Self {
            versions: IndexMap::new(),
            source,
        }
    }
    pub fn add_version(&mut self, plugin: PluginName, version: ToolVersion) {
        let versions = self
            .versions
            .entry(plugin)
            .or_insert_with(|| ToolVersionList::new(self.source.clone()));
        versions.add_version(version);
    }
    pub fn merge(&mut self, other: &Toolset) {
        for (plugin, versions) in &other.versions {
            self.versions.insert(plugin.clone(), versions.clone());
        }
    }
}

impl Display for Toolset {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let plugins = &self
            .versions
            .iter()
            .map(|(p, v)| {
                format!(
                    "{} {}",
                    p,
                    v.versions.iter().map(|v| v.to_string()).join(" ")
                )
            })
            .collect_vec();
        write!(f, "Toolset: {}", plugins.join(", "))
    }
}
