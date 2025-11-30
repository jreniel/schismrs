// schismrs/src/sync/changes.rs

use crate::sync::dependencies::SchismGroup;
use std::collections::HashSet;
use std::path::PathBuf;

/// Represents detected changes in the project
#[derive(Debug, Clone)]
pub struct ChangeSet {
    /// Config sections that changed
    pub changed_sections: Vec<String>,

    /// Source files that changed (hgrid, vgrid, etc.)
    pub changed_sources: Vec<SourceChange>,

    /// Groups that need regeneration
    pub groups_to_regenerate: Vec<SchismGroup>,

    /// Groups that are locked and cannot be regenerated
    pub locked_groups: Vec<SchismGroup>,
}

/// Represents a change to a source file
#[derive(Debug, Clone)]
pub struct SourceChange {
    pub name: String,
    pub path: PathBuf,
    pub old_hash: Option<String>,
    pub new_hash: String,
}

impl ChangeSet {
    /// Create an empty changeset
    pub fn new() -> Self {
        Self {
            changed_sections: Vec::new(),
            changed_sources: Vec::new(),
            groups_to_regenerate: Vec::new(),
            locked_groups: Vec::new(),
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.changed_sections.is_empty()
            || !self.changed_sources.is_empty()
            || !self.groups_to_regenerate.is_empty()
    }

    /// Check if there are any groups to regenerate
    pub fn needs_regeneration(&self) -> bool {
        !self.groups_to_regenerate.is_empty()
    }

    /// Add a changed config section
    pub fn add_section_change(&mut self, section: String) {
        if !self.changed_sections.contains(&section) {
            self.changed_sections.push(section);
        }
    }

    /// Add a changed source file
    pub fn add_source_change(&mut self, change: SourceChange) {
        self.changed_sources.push(change);
    }

    /// Add a group that needs regeneration
    pub fn add_group_to_regenerate(&mut self, group: SchismGroup) {
        if !self.groups_to_regenerate.contains(&group) {
            self.groups_to_regenerate.push(group);
        }
    }

    /// Mark a group as locked (cannot be regenerated)
    pub fn mark_locked(&mut self, group: SchismGroup) {
        if !self.locked_groups.contains(&group) {
            self.locked_groups.push(group);
        }
    }

    /// Remove locked groups from regeneration list
    pub fn filter_locked(&mut self) {
        let locked_set: HashSet<_> = self.locked_groups.iter().collect();
        self.groups_to_regenerate
            .retain(|group| !locked_set.contains(group));
    }

    /// Get a summary of changes for display
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        if !self.changed_sections.is_empty() {
            lines.push(format!(
                "Config sections changed: {}",
                self.changed_sections.join(", ")
            ));
        }

        if !self.changed_sources.is_empty() {
            lines.push(format!("Source files changed: {}", self.changed_sources.len()));
            for source in &self.changed_sources {
                lines.push(format!("  - {}", source.name));
            }
        }

        if !self.groups_to_regenerate.is_empty() {
            lines.push(format!(
                "Groups to regenerate: {}",
                self.groups_to_regenerate.len()
            ));
            for group in &self.groups_to_regenerate {
                lines.push(format!("  - {} ({})", group.state_key(), group.output_path()));
            }
        }

        if !self.locked_groups.is_empty() {
            lines.push(format!("Locked groups (skipped): {}", self.locked_groups.len()));
            for group in &self.locked_groups {
                lines.push(format!("  - {} ({})", group.state_key(), group.output_path()));
            }
        }

        if lines.is_empty() {
            "No changes detected.".to_string()
        } else {
            lines.join("\n")
        }
    }
}

impl Default for ChangeSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_changeset() {
        let changeset = ChangeSet::new();
        assert!(!changeset.has_changes());
        assert!(!changeset.needs_regeneration());
    }

    #[test]
    fn test_add_section_change() {
        let mut changeset = ChangeSet::new();
        changeset.add_section_change("timestep".to_string());

        assert!(changeset.has_changes());
        assert_eq!(changeset.changed_sections.len(), 1);
        assert_eq!(changeset.changed_sections[0], "timestep");
    }

    #[test]
    fn test_add_group_to_regenerate() {
        let mut changeset = ChangeSet::new();
        changeset.add_group_to_regenerate(SchismGroup::Param);

        assert!(changeset.needs_regeneration());
        assert_eq!(changeset.groups_to_regenerate.len(), 1);
    }

    #[test]
    fn test_filter_locked() {
        let mut changeset = ChangeSet::new();
        changeset.add_group_to_regenerate(SchismGroup::Param);
        changeset.add_group_to_regenerate(SchismGroup::Bctides);
        changeset.mark_locked(SchismGroup::Param);

        changeset.filter_locked();

        assert_eq!(changeset.groups_to_regenerate.len(), 1);
        assert!(changeset.groups_to_regenerate.contains(&SchismGroup::Bctides));
        assert!(!changeset.groups_to_regenerate.contains(&SchismGroup::Param));
    }

    #[test]
    fn test_no_duplicate_groups() {
        let mut changeset = ChangeSet::new();
        changeset.add_group_to_regenerate(SchismGroup::Param);
        changeset.add_group_to_regenerate(SchismGroup::Param);

        assert_eq!(changeset.groups_to_regenerate.len(), 1);
    }
}
