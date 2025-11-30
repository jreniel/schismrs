// schismrs/src/sync/dependencies.rs

use crate::config::ModelConfig;
use std::collections::{HashMap, HashSet};

/// Represents a conceptual SCHISM group that generates one or more files
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchismGroup {
    Param,        // Generates param.nml
    Bctides,      // Generates bctides.in
    Station,      // Generates station.in
    Atmospheric,  // Generates atmospheric forcing (sflux/, wind.th, etc. based on nws)
    // Future extensions:
    // Sources,      // Point sources/sinks
    // Hotstart,     // Initial conditions
    // Nudging,      // Nudging files
}

impl SchismGroup {
    /// Get the primary output path (file or directory)
    pub fn output_path(&self) -> &str {
        match self {
            SchismGroup::Param => "param.nml",
            SchismGroup::Bctides => "bctides.in",
            SchismGroup::Station => "station.in",
            SchismGroup::Atmospheric => "sflux/", // Default, may vary by nws type
        }
    }

    /// Check if this group generates a directory (multiple files)
    pub fn is_directory(&self) -> bool {
        matches!(self, SchismGroup::Atmospheric)
    }

    /// Get the generator crate name for this group
    pub fn generator_crate(&self) -> &str {
        match self {
            SchismGroup::Param => "schismrs-param",
            SchismGroup::Bctides => "schismrs-bctides",
            SchismGroup::Station => "schismrs-station",
            SchismGroup::Atmospheric => "schismrs-atmospheric",
        }
    }

    /// Get the identifier used in state.json
    pub fn state_key(&self) -> &str {
        match self {
            SchismGroup::Param => "param",
            SchismGroup::Bctides => "bctides",
            SchismGroup::Station => "station",
            SchismGroup::Atmospheric => "atmospheric",
        }
    }

    /// Get config sections this generator depends on
    ///
    /// Returns the names of config sections that affect this generator's output.
    /// This enables granular change detection - only regenerate if relevant config changed.
    ///
    /// Note: This is for CONFIG parameters only, not source files. Source files
    /// like hgrid, vgrid are tracked via source_dependencies().
    pub fn config_sections(&self) -> Vec<&'static str> {
        match self {
            // param.nml depends on: timestep, forcings, outputs, stratification, transport, coldstart
            SchismGroup::Param => {
                vec!["timestep", "forcings", "outputs", "stratification", "transport", "coldstart"]
                // TODO: Update when these sections are added to ModelConfig
            }
            
            // bctides.in depends on: forcings only (hgrid/vgrid are source files, not config)
            SchismGroup::Bctides => {
                vec!["forcings"]
                // TODO: Update when forcings section is added to ModelConfig
            }
            
            // station.in depends on: outputs only (hgrid is a source file, not config)
            SchismGroup::Station => {
                vec!["outputs"]
                // TODO: Update when outputs section is added to ModelConfig
            }
            
            // atmospheric forcing depends on: forcings only
            SchismGroup::Atmospheric => {
                vec!["forcings"]
                // TODO: Update when forcings section is added to ModelConfig
            }
        }
    }

    /// Compute fingerprint for this generator based on its config dependencies
    ///
    /// Only hashes the config sections that affect this generator's output.
    /// This enables granular change detection.
    pub fn config_fingerprint(&self, config: &ModelConfig) -> String {
        let sections = self.config_sections();
        config.fingerprint_sections(&sections)
    }

    /// Get list of source files this generator depends on
    ///
    /// Returns source file keys (e.g., "hgrid", "vgrid") that must be tracked.
    pub fn source_dependencies(&self) -> Vec<&'static str> {
        match self {
            // param.nml needs vgrid for vertical grid setup, drag for bottom friction
            SchismGroup::Param => vec!["vgrid", "drag"],
            
            // bctides.in needs hgrid for boundary node locations, vgrid for vertical structure
            SchismGroup::Bctides => vec!["hgrid", "vgrid"],
            
            // station.in needs hgrid for station coordinates validation
            SchismGroup::Station => vec!["hgrid"],
            
            // atmospheric forcing has no source file dependencies
            SchismGroup::Atmospheric => vec![],
        }
    }
}

/// Dependency graph defining which config sections affect which groups
pub struct DependencyGraph {
    dependencies: HashMap<SchismGroup, HashSet<String>>,
}

impl DependencyGraph {
    /// Create the dependency graph
    pub fn new() -> Self {
        let mut dependencies = HashMap::new();

        // param.nml depends on: timestep, forcings, outputs, stratification, transport, coldstart
        dependencies.insert(
            SchismGroup::Param,
            vec![
                "timestep".to_string(),
                "forcings".to_string(),
                "outputs".to_string(),
                "stratification".to_string(),
                "transport".to_string(),
                "coldstart".to_string(),
            ]
            .into_iter()
            .collect(),
        );

        // bctides.in depends on: hgrid, forcings.bctides
        dependencies.insert(
            SchismGroup::Bctides,
            vec!["hgrid".to_string(), "forcings".to_string()]
                .into_iter()
                .collect(),
        );

        // station.in depends on: outputs
        dependencies.insert(
            SchismGroup::Station,
            vec!["outputs".to_string()].into_iter().collect(),
        );

        // atmospheric forcing depends on: forcings (nws type and configuration)
        dependencies.insert(
            SchismGroup::Atmospheric,
            vec!["forcings".to_string()].into_iter().collect(),
        );

        Self { dependencies }
    }

    /// Get all groups that depend on a given config section
    pub fn affected_groups(&self, section: &str) -> Vec<SchismGroup> {
        self.dependencies
            .iter()
            .filter_map(|(group, deps)| {
                if deps.contains(section) {
                    Some(group.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all groups that depend on any of the given config sections
    pub fn affected_groups_by_sections(&self, sections: &[String]) -> Vec<SchismGroup> {
        let mut affected = HashSet::new();

        for section in sections {
            for group in self.affected_groups(section) {
                affected.insert(group);
            }
        }

        affected.into_iter().collect()
    }

    /// Get dependencies for a specific group
    pub fn dependencies_for(&self, group: &SchismGroup) -> Option<&HashSet<String>> {
        self.dependencies.get(group)
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_dependencies() {
        let graph = DependencyGraph::new();

        let affected = graph.affected_groups("timestep");
        assert!(affected.contains(&SchismGroup::Param));
    }

    #[test]
    fn test_bctides_dependencies() {
        let graph = DependencyGraph::new();

        let affected = graph.affected_groups("hgrid");
        assert!(affected.contains(&SchismGroup::Bctides));

        let affected = graph.affected_groups("forcings");
        assert!(affected.contains(&SchismGroup::Bctides));
        assert!(affected.contains(&SchismGroup::Param));
        assert!(affected.contains(&SchismGroup::Atmospheric));
    }

    #[test]
    fn test_affected_by_multiple_sections() {
        let graph = DependencyGraph::new();

        let sections = vec!["timestep".to_string(), "outputs".to_string()];
        let affected = graph.affected_groups_by_sections(&sections);

        assert!(affected.contains(&SchismGroup::Param));
    }

    #[test]
    fn test_generator_crate_names() {
        assert_eq!(SchismGroup::Param.generator_crate(), "schismrs-param");
        assert_eq!(SchismGroup::Bctides.generator_crate(), "schismrs-bctides");
        assert_eq!(SchismGroup::Atmospheric.generator_crate(), "schismrs-atmospheric");
    }

    #[test]
    fn test_atmospheric_is_directory() {
        assert!(SchismGroup::Atmospheric.is_directory());
        assert!(!SchismGroup::Param.is_directory());
    }

    #[test]
    fn test_source_dependencies() {
        assert_eq!(SchismGroup::Param.source_dependencies(), vec!["vgrid", "drag"]);
        assert_eq!(SchismGroup::Bctides.source_dependencies(), vec!["hgrid", "vgrid"]);
        assert_eq!(SchismGroup::Station.source_dependencies(), vec!["hgrid"]);
        assert!(SchismGroup::Atmospheric.source_dependencies().is_empty());
    }

    #[test]
    fn test_config_sections() {
        let param_sections = SchismGroup::Param.config_sections();
        assert!(param_sections.contains(&"timestep"));
        assert!(param_sections.contains(&"forcings"));
        
        let bctides_sections = SchismGroup::Bctides.config_sections();
        assert!(bctides_sections.contains(&"forcings"));
        assert_eq!(bctides_sections.len(), 1);
    }
}
