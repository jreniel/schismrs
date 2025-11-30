// schismrs/src/sync/detector.rs

use crate::config::fingerprint::config_fingerprint;
use crate::config::ModelConfig;
use crate::state::ProjectState;
use crate::sync::changes::{ChangeSet, SourceChange};
use crate::sync::dependencies::{
    // DependencyGraph
    SchismGroup,
};
use anyhow::Result;
use std::path::Path;

/// Detect changes between current config/files and previous state
pub struct ChangeDetector {
    // dependency_graph: DependencyGraph,
}

impl ChangeDetector {
    pub fn new() -> Self {
        Self {
            // dependency_graph: DependencyGraph::new(),
        }
    }

    /// Detect all changes between current state and previous state
    pub fn detect_changes(
        &self,
        project_root: &Path,
        state: &ProjectState,
        config: &ModelConfig,
    ) -> Result<ChangeSet> {
        let mut changeset = ChangeSet::new();

        // 1. Check each generator's config fingerprint
        self.detect_generator_changes(state, config, &mut changeset);

        // 2. Check source file hashes
        self.detect_source_changes(project_root, state, config, &mut changeset)?;

        // 3. Filter out locked groups (if implemented in future)
        changeset.filter_locked();

        Ok(changeset)
    }

    /// Check if any generator's config dependencies have changed
    fn detect_generator_changes(
        &self,
        state: &ProjectState,
        config: &ModelConfig,
        changeset: &mut ChangeSet,
    ) {
        let all_groups = vec![
            SchismGroup::Param,
            SchismGroup::Bctides,
            SchismGroup::Station,
            SchismGroup::Atmospheric,
        ];

        for group in all_groups {
            if self.generator_needs_regeneration(&group, state, config) {
                changeset.add_group_to_regenerate(group);
            }
        }
    }

    /// Check if a specific generator needs regeneration
    ///
    /// Returns true if:
    /// - Generator has never been run (no fingerprint in state)
    /// - Config fingerprint has changed
    fn generator_needs_regeneration(
        &self,
        group: &SchismGroup,
        state: &ProjectState,
        config: &ModelConfig,
    ) -> bool {
        let current_fingerprint = group.config_fingerprint(config);
        let stored_fingerprint = state.get_generator_fingerprint(group.state_key());

        match stored_fingerprint {
            None => {
                // Never generated before
                true
            }
            Some(stored) => {
                // Compare fingerprints
                current_fingerprint != stored
            }
        }
    }

    /// Detect changes in source files that generators depend on
    fn detect_source_changes(
        &self,
        project_root: &Path,
        state: &ProjectState,
        config: &ModelConfig,
        changeset: &mut ChangeSet,
    ) -> Result<()> {
        // Check hgrid if it exists in config
        if let Some(hgrid_path) = config.hgrid().path() {
            self.check_source_file(project_root, "hgrid", hgrid_path, state, changeset)?;
        }

        // TODO: Check other source files (vgrid, drag, etc.) when config supports them

        Ok(())
    }

    /// Check if a single source file has changed
    fn check_source_file(
        &self,
        project_root: &Path,
        source_name: &str,
        relative_path: &Path,
        state: &ProjectState,
        changeset: &mut ChangeSet,
    ) -> Result<()> {
        let full_path = project_root.join(relative_path);

        if !full_path.exists() {
            anyhow::bail!("Source file not found: {}", full_path.display());
        }

        // Compute current hash based on source type
        let current_hash = match source_name {
            "hgrid" => {
                // Use schismrs-hgrid's structural hash
                // TODO: Implement when schismrs-hgrid is available
                // let hgrid = schismrs_hgrid::Hgrid::try_from(&full_path)?;
                // hgrid.calculate_hash()

                // Placeholder: use file content hash
                self.compute_file_hash(&full_path)?
            }
            _ => {
                // For other files, use content hash
                self.compute_file_hash(&full_path)?
            }
        };

        let stored_hash = state.get_source_hash(source_name);

        // Check if hash changed
        if stored_hash.is_none() || stored_hash != Some(&current_hash) {
            changeset.add_source_change(SourceChange {
                name: source_name.to_string(),
                path: full_path,
                old_hash: stored_hash.map(String::from),
                new_hash: current_hash,
            });

            // Mark all generators that depend on this source for regeneration
            self.mark_dependent_generators(source_name, changeset);
        }

        Ok(())
    }

    /// Compute hash of file contents
    fn compute_file_hash(&self, path: &Path) -> Result<String> {
        let content = fs_err::read(path)?;
        Ok(config_fingerprint(&content))
    }

    /// Mark all generators that depend on a source file for regeneration
    fn mark_dependent_generators(&self, source_name: &str, changeset: &mut ChangeSet) {
        let all_groups = vec![
            SchismGroup::Param,
            SchismGroup::Bctides,
            SchismGroup::Station,
            SchismGroup::Atmospheric,
        ];

        for group in all_groups {
            if group.source_dependencies().contains(&source_name) {
                changeset.add_group_to_regenerate(group);
            }
        }
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::config::hgrid::HgridConfig;
//     use std::path::PathBuf;
//     use tempfile::TempDir;

//     fn create_test_config() -> ModelConfig {
//         ModelConfig {
//             hgrid: HgridConfig::SimplePath(PathBuf::from("hgrid.gr3")),
//         }
//     }

//     #[test]
//     fn test_generator_needs_regeneration_never_run() {
//         let detector = ChangeDetector::new();
//         let state = ProjectState::new(PathBuf::from("/tmp"), PathBuf::from("config.yml"));
//         let config = create_test_config();

//         // Should need regeneration if never run before
//         assert!(detector.generator_needs_regeneration(&SchismGroup::Param, &state, &config));
//     }

//     #[test]
//     fn test_generator_needs_regeneration_config_changed() {
//         let detector = ChangeDetector::new();
//         let mut state = ProjectState::new(PathBuf::from("/tmp"), PathBuf::from("config.yml"));
//         let config = create_test_config();

//         // Store old fingerprint
//         state.update_generator(
//             SchismGroup::Param.state_key().to_string(),
//             "old_fingerprint".to_string(),
//         );

//         // Should need regeneration because config changed (fingerprint differs)
//         assert!(detector.generator_needs_regeneration(&SchismGroup::Param, &state, &config));
//     }

//     #[test]
//     fn test_generator_no_regeneration_unchanged() {
//         let detector = ChangeDetector::new();
//         let mut state = ProjectState::new(PathBuf::from("/tmp"), PathBuf::from("config.yml"));
//         let config = create_test_config();

//         // Store current fingerprint
//         let current_fp = SchismGroup::Param.config_fingerprint(&config);
//         state.update_generator(SchismGroup::Param.state_key().to_string(), current_fp);

//         // Should NOT need regeneration
//         assert!(!detector.generator_needs_regeneration(&SchismGroup::Param, &state, &config));
//     }

//     #[test]
//     fn test_compute_file_hash_deterministic() {
//         let detector = ChangeDetector::new();
//         let temp_dir = TempDir::new().unwrap();
//         let file_path = temp_dir.path().join("test.txt");

//         fs_err::write(&file_path, "test content").unwrap();

//         let hash1 = detector.compute_file_hash(&file_path).unwrap();
//         let hash2 = detector.compute_file_hash(&file_path).unwrap();

//         assert_eq!(hash1, hash2, "Same file should produce same hash");
//     }
// }
