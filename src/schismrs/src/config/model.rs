// src/config/model.rs

use crate::config::fingerprint::config_fingerprint;
use crate::config::hgrid::HgridConfig;
use crate::config::timestep::TimestepConfig;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure parsed from model-config.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    hgrid: HgridConfig,
    timestep: TimestepConfig,
}

impl ModelConfig {
    /// Get a reference to the hgrid configuration
    pub fn hgrid(&self) -> &HgridConfig {
        &self.hgrid
    }

    /// Compute a fingerprint for the specified config sections
    ///
    /// This allows generators to hash only the config sections they depend on,
    /// enabling granular change detection.
    ///
    /// Note: This only hashes configuration parameters, not source files.
    /// Source files like hgrid, vgrid are tracked separately via source file hashing.
    ///
    /// # Arguments
    /// * `sections` - Names of config sections to include in fingerprint
    ///
    /// # Returns
    /// A deterministic hash string representing the combined state of requested sections
    pub fn fingerprint_sections(&self, sections: &[&str]) -> String {
        let mut parts: Vec<String> = Vec::new();

        for section in sections {
            match *section {
                // TODO: Add config sections as they're implemented:
                "timestep" => parts.push(config_fingerprint(&self.timestep)),
                // "forcings" => parts.push(config_fingerprint(&self.forcings)),
                // "outputs" => parts.push(config_fingerprint(&self.outputs)),
                // "stratification" => parts.push(config_fingerprint(&self.stratification)),
                // "transport" => parts.push(config_fingerprint(&self.transport)),
                // "coldstart" => parts.push(config_fingerprint(&self.coldstart)),
                this_section => {
                    unimplemented!("section {} is not implemented", this_section)
                }
            }
        }

        // Sort for deterministic ordering
        parts.sort();

        // Hash the combined fingerprints
        // If no sections matched, return a consistent empty hash
        if parts.is_empty() {
            config_fingerprint(&"")
        } else {
            config_fingerprint(&parts.join("-"))
        }
    }
}

impl TryFrom<&Path> for ModelConfig {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> anyhow::Result<Self> {
        let content = fs_err::read_to_string(path)
            .context(format!("Error reading {} to string.", path.display()))?;

        // Deserialize directly into ModelConfig
        serde_saphyr::from_str::<ModelConfig>(&content)
            .context(format!("Error parsing YAML file: {}", path.display()))
    }
}

impl TryFrom<&PathBuf> for ModelConfig {
    type Error = anyhow::Error;

    fn try_from(path: &PathBuf) -> anyhow::Result<Self> {
        Self::try_from(path.as_path())
    }
}
