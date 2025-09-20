// f90nmlrs/tests/schism_param_test.rs

//! Integration test using a real-world Fortran namelist from the SCHISM project.
//!
//! This test fetches the param.nml file from the SCHISM repository and validates
//! our parser against it. The file is cached locally to avoid repeated downloads.

use f90nmlrs::reads;
use reqwest;
use std::fs;
use std::path::Path;

const SCHISM_PARAM_URL: &str =
    "https://raw.githubusercontent.com/schism-dev/schism/refs/heads/master/sample_inputs/param.nml";
const CACHE_FILE: &str = "tests/fixtures/schism_param.nml";

/// Fetch the SCHISM param.nml file, either from cache or download it.
fn get_schism_param_content() -> Result<String, Box<dyn std::error::Error>> {
    // First try to read from cache
    if Path::new(CACHE_FILE).exists() {
        println!("Reading SCHISM param.nml from cache: {}", CACHE_FILE);
        return Ok(fs::read_to_string(CACHE_FILE)?);
    }

    // Create fixtures directory if it doesn't exist
    if let Some(parent) = Path::new(CACHE_FILE).parent() {
        fs::create_dir_all(parent)?;
    }

    // Download the file
    println!("Downloading SCHISM param.nml from: {}", SCHISM_PARAM_URL);

    let response = reqwest::blocking::get(SCHISM_PARAM_URL)?;
    let content = response.text()?;

    // Cache it for future runs
    fs::write(CACHE_FILE, &content)?;
    println!("Cached SCHISM param.nml to: {}", CACHE_FILE);

    Ok(content)
}

#[test]
fn test_parse_schism_param_nml() {
    let content = get_schism_param_content().expect("Failed to get SCHISM param.nml content");

    // Parse the namelist
    let namelist = reads(&content).expect("Failed to parse SCHISM param.nml");

    // Basic validation - check that we parsed something meaningful
    assert!(!namelist.is_empty(), "Parsed namelist should not be empty");

    // Print some basic info about what we parsed
    println!("Successfully parsed SCHISM param.nml!");
    println!("Found {} groups:", namelist.len());
    for group_name in namelist.group_names() {
        let group = namelist.get_group(group_name).unwrap();
        println!("  - {}: {} variables", group_name, group.len());
    }
}

#[test]
fn test_schism_param_specific_values() {
    let content = get_schism_param_content().expect("Failed to get SCHISM param.nml content");

    let namelist = reads(&content).expect("Failed to parse SCHISM param.nml");

    // Test some specific values that we expect to find
    // Note: These are based on inspecting the actual file content

    // Look for common SCHISM namelist groups
    let expected_groups = ["CORE", "OPT", "SCHOUT"];
    let mut found_groups = 0;

    for expected_group in &expected_groups {
        if namelist.has_group(expected_group) {
            found_groups += 1;
            println!("✓ Found expected group: {}", expected_group);

            let group = namelist.get_group(expected_group).unwrap();
            if !group.is_empty() {
                println!("  Group {} has {} variables", expected_group, group.len());

                // Print first few variables as samples
                for (i, (var_name, _)) in group.variables().enumerate() {
                    if i >= 3 {
                        break;
                    } // Only show first 3
                    println!("    - {}", var_name);
                }
            }
        } else {
            println!("✗ Expected group not found: {}", expected_group);
        }
    }

    // We should find at least some expected groups
    assert!(
        found_groups > 0,
        "Should find at least one expected SCHISM group"
    );
}

#[test]
fn test_schism_param_roundtrip() {
    let content = get_schism_param_content().expect("Failed to get SCHISM param.nml content");

    // Parse the namelist
    let namelist = reads(&content).expect("Failed to parse SCHISM param.nml");

    // Convert back to string
    let options = f90nmlrs::WriteOptions::default();
    let regenerated = namelist
        .to_fortran_string(&options)
        .expect("Failed to convert namelist back to string");

    // Parse the regenerated content
    let reparsed_namelist = reads(&regenerated).expect("Failed to reparse regenerated namelist");

    // Basic validation - should have same number of groups
    assert_eq!(
        namelist.len(),
        reparsed_namelist.len(),
        "Roundtrip should preserve number of groups"
    );

    // Check that all original groups are present
    for group_name in namelist.group_names() {
        assert!(
            reparsed_namelist.has_group(group_name),
            "Roundtrip should preserve group: {}",
            group_name
        );

        let original_group = namelist.get_group(group_name).unwrap();
        let reparsed_group = reparsed_namelist.get_group(group_name).unwrap();

        // Should have same number of variables (basic check)
        assert_eq!(
            original_group.len(),
            reparsed_group.len(),
            "Group {} should have same number of variables after roundtrip",
            group_name
        );
    }

    println!("✓ Roundtrip test passed!");
}

#[test]
#[cfg(feature = "json")]
fn test_schism_param_json_conversion() {
    let content = get_schism_param_content().expect("Failed to get SCHISM param.nml content");

    let namelist = reads(&content).expect("Failed to parse SCHISM param.nml");

    // Convert to JSON
    let json_content = f90nmlrs::to_json(&namelist).expect("Failed to convert to JSON");

    // Should be valid JSON
    assert!(!json_content.is_empty());
    assert!(json_content.contains("{"));
    assert!(json_content.contains("}"));

    // Convert back from JSON
    let from_json =
        f90nmlrs::from_json(&json_content).expect("Failed to parse JSON back to namelist");

    // Basic validation
    assert_eq!(namelist.len(), from_json.len());

    println!("✓ JSON conversion test passed!");
}

/// Helper function to manually inspect the SCHISM param.nml content
#[test]
// #[ignore] // Use `cargo test -- --ignored` to run this
fn inspect_schism_param_content() {
    let content = get_schism_param_content().expect("Failed to get SCHISM param.nml content");

    println!("SCHISM param.nml content (first 2000 chars):");
    println!("{}", &content[..content.len().min(2000)]);

    if content.len() > 2000 {
        println!("... (truncated, total length: {} chars)", content.len());
    }

    // Try to parse and show detailed structure
    match reads(&content) {
        Ok(namelist) => {
            println!("\nSuccessfully parsed! Structure:");
            for group_name in namelist.group_names() {
                let group = namelist.get_group(group_name).unwrap();
                println!("\nGroup '{}' ({} variables):", group_name, group.len());

                for (var_name, var_value) in group.variables() {
                    println!(
                        "  {} = {} (type: {})",
                        var_name,
                        var_value,
                        var_value.type_name()
                    );
                }
            }
        }
        Err(e) => {
            println!("\nFailed to parse: {}", e);
        }
    }
}
