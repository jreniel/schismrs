// schismrs/src/orchestrator/mod.rs

use crate::cache::CacheManager;
use crate::config::ModelConfig;
use crate::error::{Result, SchismError};
use crate::state::ProjectState;
use crate::sync::{ChangeSet, SchismGroup};
use std::path::Path;

/// Orchestrator calls workspace crates to generate SCHISM files
pub struct Orchestrator {
    cache_manager: CacheManager,
}

impl Orchestrator {
    pub fn new(project_root: &Path) -> Self {
        Self {
            cache_manager: CacheManager::new(project_root),
        }
    }

    /// Generate files for all groups in the changeset
    pub fn generate_files(
        &self,
        changeset: &ChangeSet,
        config: &ModelConfig,
        _state: &ProjectState,
    ) -> Result<()> {
        for group in &changeset.groups_to_regenerate {
            self.generate_group(group, config)?;
        }

        Ok(())
    }

    /// Generate files for a specific group by calling the appropriate workspace crate
    fn generate_group(&self, group: &SchismGroup, config: &ModelConfig) -> Result<()> {
        // Prepare directory if needed
        self.cache_manager.prepare_group_directory(group)?;

        match group {
            SchismGroup::Param => self.generate_param(config)?,
            SchismGroup::Bctides => self.generate_bctides(config)?,
            SchismGroup::Station => self.generate_station(config)?,
            SchismGroup::Atmospheric => self.generate_atmospheric(config)?,
        }

        Ok(())
    }

    /// Generate param.nml (calls schismrs-param crate)
    fn generate_param(&self, _config: &ModelConfig) -> Result<()> {
        // TODO: Call schismrs-param crate to generate param.nml

        Err(SchismError::GeneratorFailed(
            "schismrs-param".to_string(),
            "Not yet implemented".to_string(),
        ))
    }

    /// Generate bctides.in (calls schismrs-bctides crate)
    fn generate_bctides(&self, _config: &ModelConfig) -> Result<()> {
        // TODO: Call schismrs-bctides crate to generate bctides.in

        Err(SchismError::GeneratorFailed(
            "schismrs-bctides".to_string(),
            "Not yet implemented".to_string(),
        ))
    }

    /// Generate station.in (calls schismrs-station crate)
    fn generate_station(&self, _config: &ModelConfig) -> Result<()> {
        // TODO: Call schismrs-station crate to generate station.in

        Err(SchismError::GeneratorFailed(
            "schismrs-station".to_string(),
            "Not yet implemented".to_string(),
        ))
    }

    /// Generate atmospheric forcing files (calls schismrs-atmospheric crate)
    fn generate_atmospheric(&self, _config: &ModelConfig) -> Result<()> {
        // TODO: Call schismrs-atmospheric crate to generate sflux/ or wind.th

        Err(SchismError::GeneratorFailed(
            "schismrs-atmospheric".to_string(),
            "Not yet implemented".to_string(),
        ))
    }

    /// Get cache manager (for testing/inspection)
    pub fn cache_manager(&self) -> &CacheManager {
        &self.cache_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_orchestrator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let orchestrator = Orchestrator::new(temp_dir.path());

        assert!(orchestrator.cache_manager().cache_root().to_string_lossy().contains(".schismrs"));
    }
}
