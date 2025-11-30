// src/config/hgrid.rs

use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)] // Allows both string and object syntax
pub enum HgridConfig {
    /// Simple path-only syntax: `hgrid: "hgrid.gr3"`
    SimplePath(PathBuf),

    /// Extended syntax with optional CRS: `hgrid: { path: "hgrid.gr3", crs: "..." }`
    ExtendedPath {
        path: PathBuf,

        #[serde(skip_serializing_if = "Option::is_none")]
        crs: Option<String>, // WKT string
    },
    // Generator {
    //     generator: String,

    //     #[serde(default)]
    //     #[serde(skip_serializing_if = "Option::is_none")]
    //     crs: Option<String>, // WKT string for output

    //     #[serde(default)]
    //     #[serde(skip_serializing_if = "HashMap::is_empty")]
    //     params: HashMap<String, serde_json::Value>,
    // },
}

impl HgridConfig {
    /// Get the path if this is a SimplePath or ExtendedPath variant
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            HgridConfig::SimplePath(path) => Some(path),
            HgridConfig::ExtendedPath { path, .. } => Some(path),
            // HgridConfig::Generator { .. } => None,
        }
    }

    /// Get the CRS if specified
    pub fn crs(&self) -> Option<&str> {
        match self {
            HgridConfig::SimplePath(_) => None,
            HgridConfig::ExtendedPath { crs, .. } => crs.as_deref(),
            // HgridConfig::Generator { crs, .. } => crs.as_deref(),
        }
    }

    // /// Check if this config requires a generator to run
    // pub fn is_generator(&self) -> bool {
    //     matches!(self, HgridConfig::Generator { .. })
    // }
}
