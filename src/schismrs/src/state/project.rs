// schismrs/src/state/project.rs

use crate::constants::{SCHISMRS_DIR, STATE_FILE_NAME};
use crate::state::models::ProjectState;
use crate::state::models::{GeneratorState, ProjectInfo, SourceFileState};
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

impl ProjectState {
    /// Load project state from .schismrs/state.json
    pub fn load(project_root: &Path) -> Result<Self> {
        let state_path = Self::state_file_path(project_root);

        let content = fs_err::read_to_string(&state_path)?;
        let state: ProjectState = serde_json::from_str(&content).context(format!(
            "Error deserializing state file {}",
            state_path.display()
        ))?;
        // .map_err(|e| StateError::InvalidState(format!("Failed to parse state.json: {}", e)))?;

        Ok(state)
    }

    /// Save project state to .schismrs/state.json
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let state_path = Self::state_file_path(project_root);

        // Ensure .schismrs directory exists
        let schismrs_dir = project_root.join(SCHISMRS_DIR);
        fs_err::create_dir_all(&schismrs_dir)?;

        // Serialize with pretty printing for human readability
        let content = serde_json::to_string_pretty(self)?;
        fs_err::write(&state_path, content)?;

        Ok(())
    }

    /// Check if a project is initialized
    pub fn is_initialized(project_root: &Path) -> bool {
        Self::state_file_path(project_root).exists()
    }

    /// Get the path to state.json
    pub fn state_file_path(project_root: &Path) -> PathBuf {
        project_root.join(SCHISMRS_DIR).join(STATE_FILE_NAME)
    }

    /// Get the .schismrs directory path
    pub fn schismrs_dir(project_root: &Path) -> PathBuf {
        project_root.join(SCHISMRS_DIR)
    }

    /// Update config state with new hashes
    pub fn update_config_state(
        &mut self,
        _full_hash: String,
        _section_hashes: std::collections::HashMap<String, String>,
    ) {
        // self.config.full_hash = full_hash;
        // self.config.section_hashes = section_hashes;
        // self.config.last_modified = Utc::now();
    }
    /// Create a new project state
    pub fn new(root: PathBuf, _config_path: PathBuf) -> Self {
        Self {
            project: ProjectInfo {
                root,
                initialized_at: Utc::now(),
                last_sync_at: None,
            },
            generator_fingerprints: HashMap::new(),
            source_hashes: HashMap::new(),
        }
    }

    /// Update last_sync_at timestamp
    pub fn mark_synced(&mut self) {
        self.project.last_sync_at = Some(Utc::now());
    }

    /// Update generator state after successful generation
    pub fn update_generator(&mut self, state_key: String, fingerprint: String) {
        self.generator_fingerprints.insert(
            state_key,
            GeneratorState {
                fingerprint,
                synced_at: Utc::now(),
            },
        );
    }

    /// Update source file state after checking/hashing
    pub fn update_source(&mut self, name: String, hash: String, path: PathBuf) {
        self.source_hashes.insert(
            name,
            SourceFileState {
                hash,
                path,
                checked_at: Utc::now(),
            },
        );
    }

    /// Get the stored fingerprint for a generator, if any
    pub fn get_generator_fingerprint(&self, state_key: &str) -> Option<&str> {
        self.generator_fingerprints
            .get(state_key)
            .map(|state| state.fingerprint.as_str())
    }

    /// Get the stored hash for a source file, if any
    pub fn get_source_hash(&self, name: &str) -> Option<&str> {
        self.source_hashes
            .get(name)
            .map(|state| state.hash.as_str())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::TempDir;

//     #[test]
//     fn test_state_save_and_load() {
//         let temp_dir = TempDir::new().unwrap();
//         let root = temp_dir.path().to_path_buf();

//         let state = ProjectState::new(root.clone(), PathBuf::from("model-config.yml"));

//         // Save state
//         state.save(&root).unwrap();

//         // Check .schismrs directory was created
//         assert!(root.join(".schismrs").exists());
//         assert!(ProjectState::state_file_path(&root).exists());

//         // Load state back
//         let loaded = ProjectState::load(&root).unwrap();

//         assert_eq!(loaded.version, state.version);
//         assert_eq!(loaded.project.root, state.project.root);
//     }

//     #[test]
//     fn test_is_initialized() {
//         let temp_dir = TempDir::new().unwrap();
//         let root = temp_dir.path().to_path_buf();

//         assert!(!ProjectState::is_initialized(&root));

//         let state = ProjectState::new(root.clone(), PathBuf::from("model-config.yml"));
//         state.save(&root).unwrap();

//         assert!(ProjectState::is_initialized(&root));
//     }

//     #[test]
//     fn test_load_nonexistent() {
//         let temp_dir = TempDir::new().unwrap();
//         let root = temp_dir.path().to_path_buf();

//         let result = ProjectState::load(&root);
//         assert!(matches!(result, Err(SchismError::NotInitialized)));
//     }
// }
