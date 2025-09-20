// f90nmlrs/src/namelist/core.rs

//! Core Namelist struct and basic operations.

use super::formatting::FormattingHints;
use super::group::NamelistGroup;
use super::patching::MergeStrategy;
use super::validation::validate_namelist;
use crate::error::Result;
use crate::WriteOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A complete Fortran namelist containing multiple groups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Namelist {
    /// Groups in the namelist, keyed by group name
    groups: HashMap<String, NamelistGroup>,
    /// Order of groups (to preserve original order)
    group_order: Vec<String>,
    /// Formatting hints for template-based output
    #[serde(skip)]
    formatting_hints: FormattingHints,
}

impl Namelist {
    /// Create a new empty namelist.
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            group_order: Vec::new(),
            formatting_hints: FormattingHints::default(),
        }
    }

    /// Create a new namelist with formatting hints.
    pub fn with_formatting_hints(mut self, hints: FormattingHints) -> Self {
        self.formatting_hints = hints;
        self
    }

    /// Insert a new group and return a mutable reference to it.
    pub fn insert_group(&mut self, name: &str) -> &mut NamelistGroup {
        let name = name.to_lowercase();
        if !self.groups.contains_key(&name) {
            self.group_order.push(name.clone());
            self.groups.insert(name.clone(), NamelistGroup::new());
        }
        self.groups.get_mut(&name).unwrap()
    }

    /// Insert a group object directly.
    pub fn insert_group_object(&mut self, name: &str, group: NamelistGroup) {
        let name = name.to_lowercase();
        if !self.groups.contains_key(&name) {
            self.group_order.push(name.clone());
        }
        self.groups.insert(name, group);
    }

    /// Get a group by name.
    pub fn get_group(&self, name: &str) -> Option<&NamelistGroup> {
        self.groups.get(&name.to_lowercase())
    }

    /// Get a mutable reference to a group by name.
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut NamelistGroup> {
        self.groups.get_mut(&name.to_lowercase())
    }

    /// Check if a group exists.
    pub fn has_group(&self, name: &str) -> bool {
        self.groups.contains_key(&name.to_lowercase())
    }

    /// Remove a group by name.
    pub fn remove_group(&mut self, name: &str) -> Option<NamelistGroup> {
        let name = name.to_lowercase();
        if let Some(group) = self.groups.remove(&name) {
            self.group_order.retain(|g| g != &name);
            Some(group)
        } else {
            None
        }
    }

    /// Get all group names in order.
    pub fn group_names(&self) -> &[String] {
        &self.group_order
    }

    /// Get an iterator over all groups.
    pub fn groups(&self) -> impl Iterator<Item = (&String, &NamelistGroup)> {
        self.group_order
            .iter()
            .filter_map(move |name| self.groups.get(name).map(|group| (name, group)))
    }

    /// Get a mutable iterator over all groups.
    pub fn groups_mut(&mut self) -> Vec<(&String, &mut NamelistGroup)> {
        let mut result = Vec::new();
        for name in &self.group_order {
            if let Some(group) = self.groups.get_mut(name) {
                // SAFETY: We're manually ensuring that each name is unique
                // and we collect into a Vec instead of returning an iterator
                // to avoid lifetime issues
                let name_ref = unsafe { &*(name as *const String) };
                let group_ref = unsafe { &mut *(group as *mut NamelistGroup) };
                result.push((name_ref, group_ref));
            }
        }
        result
    }

    /// Apply a patch to this namelist with sophisticated merging.
    ///
    /// This method implements the core patching logic that handles:
    /// - Adding new groups and variables
    /// - Updating existing values
    /// - Merging array values intelligently
    /// - Preserving formatting hints where possible
    pub fn apply_patch(&mut self, patch: &Namelist) -> Result<()> {
        for (group_name, patch_group) in patch.groups() {
            if let Some(existing_group) = self.get_group_mut(group_name) {
                existing_group.apply_patch(patch_group)?;
            } else {
                // Create new group from patch
                self.groups.insert(group_name.clone(), patch_group.clone());
                self.group_order.push(group_name.clone());
            }
        }
        Ok(())
    }

    /// Apply a selective patch that only updates specified variables.
    ///
    /// This is useful for template-based patching where you want to update
    /// only certain values while preserving everything else.
    pub fn apply_selective_patch(
        &mut self,
        patch: &Namelist,
        include_groups: Option<&[&str]>,
        exclude_groups: Option<&[&str]>,
    ) -> Result<()> {
        for (group_name, patch_group) in patch.groups() {
            // Check inclusion/exclusion filters
            if let Some(include) = include_groups {
                if !include.iter().any(|&g| g.eq_ignore_ascii_case(group_name)) {
                    continue;
                }
            }

            if let Some(exclude) = exclude_groups {
                if exclude.iter().any(|&g| g.eq_ignore_ascii_case(group_name)) {
                    continue;
                }
            }

            if let Some(existing_group) = self.get_group_mut(group_name) {
                existing_group.apply_patch(patch_group)?;
            } else {
                // Create new group from patch
                self.groups.insert(group_name.clone(), patch_group.clone());
                self.group_order.push(group_name.clone());
            }
        }
        Ok(())
    }

    /// Create a patch that represents the difference between this namelist and another.
    ///
    /// This is useful for generating minimal patches or understanding what changed.
    pub fn create_patch_from(&self, other: &Namelist) -> Namelist {
        let mut patch = Namelist::new();

        for (group_name, other_group) in other.groups() {
            if let Some(self_group) = self.get_group(group_name) {
                let group_patch = self_group.create_patch_from(other_group);
                if !group_patch.is_empty() {
                    patch.groups.insert(group_name.clone(), group_patch);
                    patch.group_order.push(group_name.clone());
                }
            } else {
                // Entire group is new
                patch.groups.insert(group_name.clone(), other_group.clone());
                patch.group_order.push(group_name.clone());
            }
        }

        patch
    }

    /// Convert this namelist to a Fortran string representation.
    pub fn to_fortran_string(&self, options: &WriteOptions) -> Result<String> {
        let mut output = String::new();
        let mut first_group = true;

        let groups: Vec<_> = if options.sort_groups {
            let mut sorted: Vec<_> = self.groups().collect();
            sorted.sort_by_key(|(name, _)| name.to_lowercase());
            sorted
        } else {
            self.groups().collect()
        };

        for (group_name, group) in groups {
            if !first_group {
                output.push('\n');
            }
            first_group = false;

            let name = if options.uppercase {
                group_name.to_uppercase()
            } else {
                group_name.clone()
            };

            output.push_str(&format!("&{}\n", name));
            output.push_str(&group.to_fortran_string(options)?);
            output.push_str("/\n");
        }

        Ok(output)
    }

    /// Get formatting hints.
    pub fn formatting_hints(&self) -> &FormattingHints {
        &self.formatting_hints
    }

    /// Set formatting hints.
    pub fn set_formatting_hints(&mut self, hints: FormattingHints) {
        self.formatting_hints = hints;
    }

    /// Check if the namelist is empty.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Get the number of groups.
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    /// Validate the namelist for consistency.
    pub fn validate(&self) -> Result<()> {
        validate_namelist(&self.groups)
    }

    /// Merge another namelist into this one using specific merge strategies.
    pub fn merge_with_strategy(&mut self, other: &Namelist, strategy: MergeStrategy) -> Result<()> {
        for (group_name, other_group) in other.groups() {
            if let Some(existing_group) = self.get_group_mut(group_name) {
                existing_group.merge_with_strategy(other_group, strategy)?;
            } else {
                match strategy {
                    MergeStrategy::Replace | MergeStrategy::Update | MergeStrategy::Append => {
                        self.groups.insert(group_name.clone(), other_group.clone());
                        self.group_order.push(group_name.clone());
                    }
                    MergeStrategy::SkipExisting => {
                        // Don't add new groups when skipping existing
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for Namelist {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Namelist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_fortran_string(&crate::WriteOptions::default()) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<invalid namelist>"),
        }
    }
}

