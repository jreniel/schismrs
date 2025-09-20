// f90nmlrs/src/namelist/group.rs

//! Namelist group data structure and operations.

use super::formatting::GroupFormattingHints;
use super::patching::{append_values, merge_values, MergeStrategy};
use super::validation;
use crate::error::Result;
use crate::fortran_types::FortranValue;
use crate::WriteOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A single namelist group containing variables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamelistGroup {
    /// Variables in the group
    variables: HashMap<String, FortranValue>,
    /// Order of variables (to preserve original order)
    variable_order: Vec<String>,
    /// Starting indices for arrays
    start_indices: HashMap<String, Vec<i32>>,
    /// Comments associated with variables
    #[serde(skip)]
    variable_comments: HashMap<String, String>,
    /// Formatting hints for this group
    #[serde(skip)]
    formatting_hints: GroupFormattingHints,
}

impl NamelistGroup {
    /// Create a new empty namelist group.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            variable_order: Vec::new(),
            start_indices: HashMap::new(),
            variable_comments: HashMap::new(),
            formatting_hints: GroupFormattingHints::default(),
        }
    }

    /// Insert a variable with automatic type conversion.
    pub fn insert<T: Into<FortranValue>>(&mut self, name: &str, value: T) -> &mut Self {
        let name = name.to_lowercase();
        if !self.variables.contains_key(&name) {
            self.variable_order.push(name.clone());
        }
        self.variables.insert(name, value.into());
        self
    }

    /// Insert a variable with explicit FortranValue.
    pub fn insert_value(&mut self, name: &str, value: FortranValue) -> &mut Self {
        let name = name.to_lowercase();
        if !self.variables.contains_key(&name) {
            self.variable_order.push(name.clone());
        }
        self.variables.insert(name, value);
        self
    }

    /// Insert a variable with a comment.
    pub fn insert_with_comment<T: Into<FortranValue>>(
        &mut self,
        name: &str,
        value: T,
        comment: &str,
    ) -> &mut Self {
        let name = name.to_lowercase();
        if !self.variables.contains_key(&name) {
            self.variable_order.push(name.clone());
        }
        self.variables.insert(name.clone(), value.into());
        self.variable_comments.insert(name, comment.to_string());
        self
    }

    /// Get a variable by name.
    pub fn get(&self, name: &str) -> Option<&FortranValue> {
        self.variables.get(&name.to_lowercase())
    }

    /// Get a variable (alias for get method for compatibility).
    pub fn get_variable(&self, name: &str) -> Option<&FortranValue> {
        self.get(name)
    }

    /// Get a mutable reference to a variable by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut FortranValue> {
        self.variables.get_mut(&name.to_lowercase())
    }

    /// Check if a variable exists.
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(&name.to_lowercase())
    }

    /// Remove a variable by name.
    pub fn remove(&mut self, name: &str) -> Option<FortranValue> {
        let name = name.to_lowercase();
        if let Some(value) = self.variables.remove(&name) {
            self.variable_order.retain(|v| v != &name);
            self.start_indices.remove(&name);
            self.variable_comments.remove(&name);
            Some(value)
        } else {
            None
        }
    }

    /// Get all variable names in order.
    pub fn variable_names(&self) -> &[String] {
        &self.variable_order
    }

    /// Get an iterator over all variables.
    pub fn variables(&self) -> impl Iterator<Item = (&String, &FortranValue)> {
        self.variable_order
            .iter()
            .filter_map(move |name| self.variables.get(name).map(|value| (name, value)))
    }

    /// Get a mutable iterator over all variables.
    pub fn variables_mut(&mut self) -> Vec<(&String, &mut FortranValue)> {
        let mut result = Vec::new();
        for name in &self.variable_order {
            if let Some(value) = self.variables.get_mut(name) {
                // SAFETY: Similar to groups_mut, we collect into Vec to avoid lifetime issues
                let name_ref = unsafe { &*(name as *const String) };
                let value_ref = unsafe { &mut *(value as *mut FortranValue) };
                result.push((name_ref, value_ref));
            }
        }
        result
    }

    /// Set the starting indices for an array variable.
    pub fn set_start_indices(&mut self, name: &str, indices: Vec<i32>) {
        self.start_indices.insert(name.to_lowercase(), indices);
    }

    /// Get the starting indices for an array variable.
    pub fn get_start_indices(&self, name: &str) -> Option<&[i32]> {
        self.start_indices
            .get(&name.to_lowercase())
            .map(|v| v.as_slice())
    }

    /// Set a comment for a variable.
    pub fn set_comment(&mut self, name: &str, comment: &str) {
        self.variable_comments
            .insert(name.to_lowercase(), comment.to_string());
    }

    /// Get the comment for a variable.
    pub fn get_comment(&self, name: &str) -> Option<&str> {
        self.variable_comments
            .get(&name.to_lowercase())
            .map(|s| s.as_str())
    }

    /// Apply a patch to this group with intelligent merging.
    pub fn apply_patch(&mut self, patch: &NamelistGroup) -> Result<()> {
        for (var_name, patch_value) in patch.variables() {
            if let Some(existing_value) = self.get_mut(var_name) {
                // Try to merge intelligently
                *existing_value = merge_values(existing_value, patch_value)?;
            } else {
                // Add new variable
                self.insert_value(var_name, patch_value.clone());
            }

            // Copy start indices if present
            if let Some(indices) = patch.get_start_indices(var_name) {
                self.set_start_indices(var_name, indices.to_vec());
            }

            // Copy comments if present
            if let Some(comment) = patch.get_comment(var_name) {
                self.set_comment(var_name, comment);
            }
        }
        Ok(())
    }

    /// Merge with another group using a specific strategy.
    pub fn merge_with_strategy(
        &mut self,
        other: &NamelistGroup,
        strategy: MergeStrategy,
    ) -> Result<()> {
        for (var_name, other_value) in other.variables() {
            match strategy {
                MergeStrategy::Replace => {
                    self.insert_value(var_name, other_value.clone());
                }
                MergeStrategy::Update => {
                    self.insert_value(var_name, other_value.clone());
                }
                MergeStrategy::Append => {
                    if let Some(existing_value) = self.get_mut(var_name) {
                        *existing_value = append_values(existing_value, other_value)?;
                    } else {
                        self.insert_value(var_name, other_value.clone());
                    }
                }
                MergeStrategy::SkipExisting => {
                    if !self.has_variable(var_name) {
                        self.insert_value(var_name, other_value.clone());
                    }
                }
            }

            // Always copy metadata for non-skipped variables
            if strategy != MergeStrategy::SkipExisting || !self.has_variable(var_name) {
                if let Some(indices) = other.get_start_indices(var_name) {
                    self.set_start_indices(var_name, indices.to_vec());
                }

                if let Some(comment) = other.get_comment(var_name) {
                    self.set_comment(var_name, comment);
                }
            }
        }
        Ok(())
    }

    /// Create a patch representing the difference from another group.
    pub fn create_patch_from(&self, other: &NamelistGroup) -> NamelistGroup {
        let mut patch = NamelistGroup::new();

        for (var_name, other_value) in other.variables() {
            if let Some(self_value) = self.get(var_name) {
                if self_value != other_value {
                    patch.insert_value(var_name, other_value.clone());

                    // Copy metadata
                    if let Some(indices) = other.get_start_indices(var_name) {
                        patch.set_start_indices(var_name, indices.to_vec());
                    }
                    if let Some(comment) = other.get_comment(var_name) {
                        patch.set_comment(var_name, comment);
                    }
                }
            } else {
                // New variable
                patch.insert_value(var_name, other_value.clone());

                // Copy metadata
                if let Some(indices) = other.get_start_indices(var_name) {
                    patch.set_start_indices(var_name, indices.to_vec());
                }
                if let Some(comment) = other.get_comment(var_name) {
                    patch.set_comment(var_name, comment);
                }
            }
        }

        patch
    }

    /// Convert this group to a Fortran string representation.
    pub fn to_fortran_string(&self, options: &WriteOptions) -> Result<String> {
        let mut output = String::new();

        let variables: Vec<_> = if options.sort_variables {
            let mut sorted: Vec<_> = self.variables().collect();
            sorted.sort_by_key(|(name, _)| name.to_lowercase());
            sorted
        } else {
            self.variables().collect()
        };

        for (var_name, var_value) in variables {
            let name = if options.uppercase {
                var_name.to_uppercase()
            } else {
                var_name.clone()
            };

            let assignment_str = self.format_variable_assignment(&name, var_value, options)?;

            for line in assignment_str {
                output.push_str(&options.indent);
                output.push_str(&line);

                // Add comment if present
                if let Some(comment) = self.get_comment(var_name) {
                    if !comment.trim().is_empty() {
                        if !comment.trim().starts_with('!') {
                            output.push_str("  ! ");
                        } else {
                            output.push_str("  ");
                        }
                        output.push_str(comment.trim());
                    }
                }

                output.push('\n');
            }
        }

        Ok(output)
    }

    fn format_variable_assignment(
        &self,
        name: &str,
        value: &FortranValue,
        options: &WriteOptions,
    ) -> Result<Vec<String>> {
        let mut lines = Vec::new();

        match value {
            FortranValue::Array(arr) => {
                self.format_array_assignment(name, arr, options, &mut lines)?;
            }
            FortranValue::MultiArray {
                values,
                dimensions,
                start_indices,
            } => {
                self.format_multi_array_assignment(
                    name,
                    values,
                    dimensions,
                    start_indices,
                    options,
                    &mut lines,
                )?;
            }
            FortranValue::DerivedType(fields) => {
                self.format_derived_type_assignment(name, fields, options, &mut lines)?;
            }
            FortranValue::DerivedTypeArray(arr) => {
                self.format_derived_type_array_assignment(name, arr, options, &mut lines)?;
            }
            _ => {
                let assignment = self.format_simple_assignment(name, value, options)?;
                lines.push(assignment);
            }
        }

        Ok(lines)
    }

    fn format_simple_assignment(
        &self,
        name: &str,
        value: &FortranValue,
        options: &WriteOptions,
    ) -> Result<String> {
        let mut line = String::new();

        // Add indices if present
        if let Some(indices) = self.get_start_indices(name) {
            if !indices.is_empty() {
                line.push_str(name);
                line.push('(');
                for (i, &idx) in indices.iter().enumerate() {
                    if i > 0 {
                        line.push(',');
                        if options.column_width > 0 {
                            line.push(' ');
                        }
                    }
                    line.push_str(&idx.to_string());
                }
                line.push(')');
            } else {
                line.push_str(name);
            }
        } else {
            line.push_str(name);
        }

        // Add assignment operator
        line.push_str(" = ");

        // Add value
        line.push_str(&value.to_fortran_string(options.uppercase));

        // Add comma if requested
        if options.end_comma {
            line.push(',');
        }

        Ok(line)
    }

    fn format_array_assignment(
        &self,
        name: &str,
        values: &[FortranValue],
        options: &WriteOptions,
        lines: &mut Vec<String>,
    ) -> Result<()> {
        if values.is_empty() {
            lines.push(format!("{} =", name));
            return Ok(());
        }

        let start_indices = self.get_start_indices(name);
        let start_idx = start_indices
            .and_then(|indices| indices.first())
            .copied()
            .unwrap_or(options.default_start_index);

        let end_idx = start_idx + values.len() as i32 - 1;

        let mut line = format!(
            "{}({}) = ",
            name,
            if values.len() == 1 {
                start_idx.to_string()
            } else {
                format!("{}:{}", start_idx, end_idx)
            }
        );

        let header_len = line.len();

        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                line.push_str(", ");
            }

            let value_str = value.to_fortran_string(options.uppercase);

            // Check if we need to wrap to next line
            if options.column_width > 0
                && line.len() + value_str.len() > options.column_width
                && line.len() > header_len
            {
                // End current line
                lines.push(line);

                // Start new line with proper indentation
                line = " ".repeat(header_len);
            }

            line.push_str(&value_str);
        }

        if options.end_comma {
            line.push(',');
        }

        lines.push(line);
        Ok(())
    }

    fn format_multi_array_assignment(
        &self,
        name: &str,
        values: &[FortranValue],
        _dimensions: &[usize],
        _start_indices: &[i32],
        options: &WriteOptions,
        lines: &mut Vec<String>,
    ) -> Result<()> {
        // For multi-dimensional arrays, format as a simple list for now
        // TODO: Implement proper multi-dimensional formatting
        let mut line = format!("{}(:,:) = ", name);

        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                line.push_str(", ");
            }
            line.push_str(&value.to_fortran_string(options.uppercase));
        }

        if options.end_comma {
            line.push(',');
        }

        lines.push(line);
        Ok(())
    }

    fn format_derived_type_assignment(
        &self,
        name: &str,
        fields: &HashMap<String, FortranValue>,
        options: &WriteOptions,
        lines: &mut Vec<String>,
    ) -> Result<()> {
        for (field_name, field_value) in fields {
            let full_name = format!("{}%{}", name, field_name);
            let assignment = self.format_simple_assignment(&full_name, field_value, options)?;
            lines.push(assignment);
        }
        Ok(())
    }

    fn format_derived_type_array_assignment(
        &self,
        name: &str,
        array: &[HashMap<String, FortranValue>],
        options: &WriteOptions,
        lines: &mut Vec<String>,
    ) -> Result<()> {
        for (i, element) in array.iter().enumerate() {
            let index = i as i32 + options.default_start_index;
            for (field_name, field_value) in element {
                let full_name = format!("{}({})%{}", name, index, field_name);
                let assignment = self.format_simple_assignment(&full_name, field_value, options)?;
                lines.push(assignment);
            }
        }
        Ok(())
    }

    /// Validate this group for consistency.
    pub fn validate(&self, group_name: &str) -> Result<()> {
        validation::validate_group_variables(&self.variables, group_name)
    }

    /// Convenience methods for getting typed values
    pub fn get_i32(&self, name: &str) -> Option<i32> {
        self.get(name)?.as_integer().ok().map(|i| i as i32)
    }

    pub fn get_i64(&self, name: &str) -> Option<i64> {
        self.get(name)?.as_integer().ok()
    }

    pub fn get_f32(&self, name: &str) -> Option<f32> {
        self.get(name)?.as_real().ok().map(|f| f as f32)
    }

    pub fn get_f64(&self, name: &str) -> Option<f64> {
        self.get(name)?.as_real().ok()
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name)?.as_logical().ok()
    }

    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.get(name)?.as_character().ok()
    }

    /// Check if the group is empty.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Get the number of variables.
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Get formatting hints for this group.
    pub fn formatting_hints(&self) -> &GroupFormattingHints {
        &self.formatting_hints
    }

    /// Set formatting hints for this group.
    pub fn set_formatting_hints(&mut self, hints: GroupFormattingHints) {
        self.formatting_hints = hints;
    }
}

impl Default for NamelistGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NamelistGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_fortran_string(&crate::WriteOptions::default()) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<invalid group>"),
        }
    }
}

