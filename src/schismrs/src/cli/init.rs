// schismrs/src/cli/init.rs

use crate::constants::DEFAULT_CONFIG_NAME;
use crate::state::ProjectState;
use anyhow::Context;
use std::path::{Path, PathBuf};

/// Initialize a SCHISM project in the current directory
pub fn init_project(project_root: &Path) -> anyhow::Result<()> {
    // Check if already initialized
    if ProjectState::is_initialized(project_root) {
        anyhow::bail!(
            "The project at {} is already initialized",
            project_root.display()
        );
    }

    println!("Initializing SCHISM project in: {}", project_root.display());

    // Check if config file exists
    let config_path = project_root.join(DEFAULT_CONFIG_NAME);
    if !config_path.exists() {
        anyhow::bail!(
            "Expected a configuration file: {} but it doesn't exist.",
            config_path.display()
        );
    }

    // Create .schismrs directory structure
    let schismrs_dir = ProjectState::schismrs_dir(project_root);
    fs_err::create_dir_all(&schismrs_dir).context(format!(
        "Failed to create directory: {}",
        schismrs_dir.display()
    ))?;
    println!("  ✓ Created {}", schismrs_dir.display());

    // Initialize cache structure
    // let cache_manager = CacheManager::new(project_root);
    // cache_manager.initialize().context();
    // println!("  ✓ Initialized cache directory");

    // Create initial state
    let state = ProjectState::new(
        project_root.to_path_buf(),
        PathBuf::from(DEFAULT_CONFIG_NAME),
    );

    // Save state
    state.save(project_root)?;
    println!("  ✓ Created state file");

    // Create .gitignore in .schismrs directory
    // create_schismrs_gitignore(project_root)?;
    // println!("  ✓ Created .gitignore");

    // println!("\n✓ Project initialized successfully!");
    // println!("\nNext steps:");
    // println!("  1. Edit {} to configure your model", DEFAULT_CONFIG_NAME);
    // println!("  2. Run 'schismrs sync' to generate SCHISM input files");

    Ok(())
}

/// Create .gitignore in .schismrs directory
fn _create_schismrs_gitignore(project_root: &Path) -> anyhow::Result<()> {
    let gitignore_path = ProjectState::schismrs_dir(project_root).join(".gitignore");

    let gitignore_content = r#"# Ignore cache directory but keep state
cache/

# Keep state.json
!state.json
"#;

    fs_err::write(&gitignore_path, gitignore_content)
        .context(format!("Error writting file {}", gitignore_path.display()))?;

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Write;
//     use tempfile::TempDir;

//     #[test]
//     fn test_init_project_success() {
//         let temp_dir = TempDir::new().unwrap();
//         let root = temp_dir.path();

//         // Create a config file
//         let config_path = root.join(DEFAULT_CONFIG_NAME);
//         let mut file = fs_err::File::create(&config_path).unwrap();
//         file.write_all(b"hgrid: hgrid.gr3\ntimestep: 150.0")
//             .unwrap();

//         // Initialize project
//         init_project(root).unwrap();

//         // Check directories created
//         assert!(ProjectState::schismrs_dir(root).exists());
//         assert!(ProjectState::state_file_path(root).exists());

//         // Check cache structure
//         let cache_manager = CacheManager::new(root);
//         assert!(cache_manager.cache_root().exists());
//         assert!(cache_manager.sources_dir().exists());
//         assert!(cache_manager.generated_dir().exists());

//         // Check .gitignore
//         assert!(ProjectState::schismrs_dir(root).join(".gitignore").exists());
//     }

//     #[test]
//     fn test_init_already_initialized() {
//         let temp_dir = TempDir::new().unwrap();
//         let root = temp_dir.path();

//         // Create config file
//         let config_path = root.join(DEFAULT_CONFIG_NAME);
//         fs_err::write(&config_path, "hgrid: hgrid.gr3").unwrap();

//         // Initialize once
//         init_project(root).unwrap();

//         // Try to initialize again
//         let result = init_project(root);
//         assert!(matches!(result, Err(CliError::AlreadyInitialized(_))));
//     }

//     #[test]
//     fn test_init_no_config_file() {
//         let temp_dir = TempDir::new().unwrap();
//         let root = temp_dir.path();

//         // Try to initialize without config file
//         let result = init_project(root);
//         assert!(matches!(result, Err(CliError::ConfigNotFound(_))));
//     }
// }
