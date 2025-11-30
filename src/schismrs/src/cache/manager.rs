// schismrs/src/cache/manager.rs

use crate::error::{Result, SchismError};
use crate::sync::SchismGroup;
use std::path::{Path, PathBuf};

const CACHE_DIR: &str = "cache";
const SOURCES_DIR: &str = "sources";
const GENERATED_DIR: &str = "generated";

/// Manages the .schismrs/cache directory structure
pub struct CacheManager {
    cache_root: PathBuf,
}

impl CacheManager {
    /// Create a new cache manager for the given project root
    pub fn new(project_root: &Path) -> Self {
        let cache_root = project_root.join(".schismrs").join(CACHE_DIR);
        Self { cache_root }
    }

    /// Initialize the cache directory structure
    pub fn initialize(&self) -> Result<()> {
        fs_err::create_dir_all(self.sources_dir())?;
        fs_err::create_dir_all(self.generated_dir())?;

        // Create .gitignore in cache directory
        let gitignore_path = self.cache_root.join(".gitignore");
        let gitignore_content = "# Ignore all cache contents\n*\n!.gitignore\n";
        fs_err::write(gitignore_path, gitignore_content)?;

        Ok(())
    }

    /// Get the cache root directory
    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    /// Get the sources directory (.schismrs/cache/sources/)
    pub fn sources_dir(&self) -> PathBuf {
        self.cache_root.join(SOURCES_DIR)
    }

    /// Get the generated directory (.schismrs/cache/generated/)
    pub fn generated_dir(&self) -> PathBuf {
        self.cache_root.join(GENERATED_DIR)
    }

    /// Get the path for a cached source file
    pub fn source_path(&self, name: &str) -> PathBuf {
        self.sources_dir().join(name)
    }

    /// Get the path for a generated group's output
    pub fn generated_path(&self, group: &SchismGroup) -> PathBuf {
        self.generated_dir().join(group.output_path())
    }

    /// Copy a source file into the cache
    pub fn cache_source_file(&self, name: &str, source_path: &Path) -> Result<()> {
        if !source_path.exists() {
            return Err(SchismError::SourceFileNotFound(source_path.to_path_buf()));
        }

        let dest_path = self.source_path(name);

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            fs_err::create_dir_all(parent)?;
        }

        fs_err::copy(source_path, &dest_path)?;

        Ok(())
    }

    /// Check if a source file exists in cache
    pub fn has_cached_source(&self, name: &str) -> bool {
        self.source_path(name).exists()
    }

    /// Check if a generated group output exists
    pub fn has_generated(&self, group: &SchismGroup) -> bool {
        self.generated_path(group).exists()
    }

    /// Remove a generated group's output
    pub fn remove_generated(&self, group: &SchismGroup) -> Result<()> {
        let path = self.generated_path(group);

        if !path.exists() {
            return Ok(());
        }

        if group.is_directory() {
            fs_err::remove_dir_all(&path)?;
        } else {
            fs_err::remove_file(&path)?;
        }

        Ok(())
    }

    /// Create directory for a group if it's a directory type
    pub fn prepare_group_directory(&self, group: &SchismGroup) -> Result<()> {
        if group.is_directory() {
            let path = self.generated_path(group);
            fs_err::create_dir_all(&path)?;
        }
        Ok(())
    }

    /// Clean the entire cache directory
    pub fn clean(&self) -> Result<()> {
        if self.cache_root.exists() {
            fs_err::remove_dir_all(&self.cache_root)?;
        }
        self.initialize()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_cache_manager_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CacheManager::new(temp_dir.path());

        manager.initialize().unwrap();

        assert!(manager.cache_root().exists());
        assert!(manager.sources_dir().exists());
        assert!(manager.generated_dir().exists());
        assert!(manager.cache_root().join(".gitignore").exists());
    }

    #[test]
    fn test_cache_source_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CacheManager::new(temp_dir.path());
        manager.initialize().unwrap();

        // Create a test source file
        let source_path = temp_dir.path().join("test.gr3");
        let mut file = fs_err::File::create(&source_path).unwrap();
        file.write_all(b"test content").unwrap();

        // Cache it
        manager.cache_source_file("test.gr3", &source_path).unwrap();

        assert!(manager.has_cached_source("test.gr3"));

        let cached_content = fs_err::read_to_string(manager.source_path("test.gr3")).unwrap();
        assert_eq!(cached_content, "test content");
    }

    #[test]
    fn test_generated_paths() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CacheManager::new(temp_dir.path());
        manager.initialize().unwrap();

        let param_path = manager.generated_path(&SchismGroup::Param);
        assert!(param_path.ends_with("param.nml"));

        let sflux_path = manager.generated_path(&SchismGroup::Atmospheric);
        assert!(sflux_path.ends_with("sflux"));
    }

    #[test]
    fn test_clean_cache() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CacheManager::new(temp_dir.path());
        manager.initialize().unwrap();

        // Create some files
        let test_file = manager.sources_dir().join("test.txt");
        fs_err::write(&test_file, "content").unwrap();

        assert!(test_file.exists());

        // Clean cache
        manager.clean().unwrap();

        // Check structure recreated but file is gone
        assert!(manager.cache_root().exists());
        assert!(manager.sources_dir().exists());
        assert!(!test_file.exists());
    }
}
