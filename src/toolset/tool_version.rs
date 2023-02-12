use std::fmt::{Display, Formatter};

/// represents a single version of a tool for a particular plugin
#[derive(Debug, Clone)]
pub enum ToolVersion {
    Version(String),
    Prefix(String),
    Ref(String),
    Path(String),
    System,
}

impl Display for ToolVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ToolVersion::Version(v) => write!(f, "{}", v),
            ToolVersion::Prefix(p) => write!(f, "prefix:{}", p),
            ToolVersion::Ref(r) => write!(f, "ref:{}", r),
            ToolVersion::Path(p) => write!(f, "path:{}", p),
            ToolVersion::System => write!(f, "system"),
        }
    }
}
