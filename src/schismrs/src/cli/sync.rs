// schismrs/src/cli/sync.rs

use crate::cli::init_project;
use crate::config::ModelConfig;
use crate::constants::DEFAULT_CONFIG_NAME;
use crate::state::ProjectState;
use crate::sync::ChangeDetector;
use anyhow::Result;
use std::path::Path;

/// Synchronize configuration changes and regenerate affected files
pub fn sync_project(project_root: &Path) -> Result<()> {
    println!("Synchronizing SCHISM project...");

    if !ProjectState::is_initialized(project_root) {
        init_project(project_root)?;
    }

    // Load current state
    let state = ProjectState::load(project_root)?;
    println!("✓ Loaded project state");

    let config_path = project_root.join(DEFAULT_CONFIG_NAME);

    // Load configuration
    let model_config = ModelConfig::try_from(&config_path)?;

    println!("✓ Loaded configuration");

    // // Detect changes
    let detector = ChangeDetector::new();
    let changeset = detector.detect_changes(project_root, &state, &model_config)?;

    // Display change summary
    if !changeset.has_changes() {
        println!("✓ No changes detected. Everything is up to date.");
        return Ok(());
    }

    // println!("Changes detected:\n");
    // println!("{}\n", changeset.summary());

    // if !changeset.needs_regeneration() {
    //     println!("✓ No files need regeneration.");
    //     return Ok(());
    // }

    // // Confirm with user (in future, add --yes flag to skip)
    // println!("Proceeding with file generation...\n");

    // // Generate files
    // let orchestrator = Orchestrator::new(project_root);
    // orchestrator.generate_files(&changeset, &config_with_hashes.config, &state)?;

    // println!("✓ Generated files successfully\n");

    // // Update state with new hashes
    // state.update_config_state(
    //     config_with_hashes.full_hash,
    //     config_with_hashes.section_hashes,
    // );

    // // Update source file info
    // for source_change in &changeset.changed_sources {
    //     update_source_file_info(&mut state, source_change)?;
    // }

    // // Update generated file info
    // update_generated_file_info(&mut state, &changeset, &orchestrator)?;

    // // Mark synced
    // state.mark_synced();

    // // Save updated state
    // state.save(project_root)?;
    // println!("✓ Updated project state");

    // println!("\n✓ Synchronization complete!");

    Ok(())
}

// /// Update source file info in state after detecting changes
// fn update_source_file_info(
//     state: &mut ProjectState,
//     source_change: &crate::sync::SourceChange,
// ) -> Result<()> {
//     let metadata = fs_err::metadata(&source_change.path)?;

//     let info = crate::state::SourceFileInfo {
//         path: source_change.path.clone(),
//         absolute_path: fs_err::canonicalize(&source_change.path)?,
//         content_hash: source_change.new_hash.clone(),
//         last_checked: chrono::Utc::now(),
//         file_size: metadata.len(),
//         modified_at: metadata
//             .modified()
//             .ok()
//             .and_then(|t| chrono::DateTime::from(t).into())
//             .unwrap_or_else(chrono::Utc::now),
//     };

//     state.source_files.insert(source_change.name.clone(), info);

//     Ok(())
// }

// /// Update generated file info in state after generation
// fn update_generated_file_info(
//     state: &mut ProjectState,
//     changeset: &crate::sync::ChangeSet,
//     orchestrator: &Orchestrator,
// ) -> Result<()> {
//     use crate::config::sections::compute_string_hash;

//     for group in &changeset.groups_to_regenerate {
//         let generated_path = orchestrator.cache_manager().generated_path(group);

//         // Compute hash of generated content
//         let content_hash = if group.is_directory() {
//             // For directories, hash the directory manifest (list of files + their hashes)
//             compute_directory_hash(&generated_path)?
//         } else {
//             // For single files, hash the file content
//             let content = fs_err::read_to_string(&generated_path)?;
//             compute_string_hash(&content)
//         };

//         let dependencies = changeset
//             .changed_sections
//             .iter()
//             .cloned()
//             .collect::<Vec<_>>();

//         let info = crate::state::GeneratedFileInfo {
//             path: generated_path,
//             content_hash,
//             generated_at: chrono::Utc::now(),
//             locked: false,
//             depends_on: dependencies,
//             generator_crate: group.generator_crate().to_string(),
//             source_config_hash: state.config.full_hash.clone(),
//         };

//         state
//             .generated_files
//             .insert(group.state_key().to_string(), info);
//     }

//     Ok(())
// }
