// f90nmlrs/src/namelist/formatting.rs

//! Formatting hints and preservation for namelist output.
//!
//! This module provides structures for preserving original formatting
//! when reading and writing namelists, similar to how the Python f90nml
//! handles template-based patching.

use std::collections::HashMap;

/// Formatting hints for the entire namelist.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FormattingHints {
    /// Hints for individual groups
    pub groups: HashMap<String, GroupFormattingHints>,
    /// Global case style preference
    pub case_style: CaseStyle,
    /// Global indentation style
    pub indentation: String,
    /// Whether to preserve original spacing
    pub preserve_spacing: bool,
}

impl FormattingHints {
    /// Create new formatting hints.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set case style for all elements.
    pub fn with_case_style(mut self, case_style: CaseStyle) -> Self {
        self.case_style = case_style;
        self
    }
    
    /// Set global indentation.
    pub fn with_indentation<S: Into<String>>(mut self, indent: S) -> Self {
        self.indentation = indent.into();
        self
    }
    
    /// Enable or disable spacing preservation.
    pub fn with_spacing_preservation(mut self, preserve: bool) -> Self {
        self.preserve_spacing = preserve;
        self
    }
    
    /// Get or create formatting hints for a group.
    pub fn group_hints(&mut self, group_name: &str) -> &mut GroupFormattingHints {
        self.groups.entry(group_name.to_lowercase()).or_insert_with(GroupFormattingHints::default)
    }
}

/// Formatting hints for a single namelist group.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GroupFormattingHints {
    /// Variable-specific formatting
    pub variables: HashMap<String, VariableFormatting>,
    /// Group-level indentation override
    pub indentation: Option<String>,
    /// Comments associated with this group
    pub comments: Vec<String>,
    /// Original ordering of variables
    pub variable_order: Vec<String>,
}

impl GroupFormattingHints {
    /// Create new group formatting hints.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set variable formatting.
    pub fn set_variable_formatting(&mut self, var_name: &str, formatting: VariableFormatting) {
        self.variables.insert(var_name.to_lowercase(), formatting);
    }
    
    /// Get variable formatting.
    pub fn get_variable_formatting(&self, var_name: &str) -> Option<&VariableFormatting> {
        self.variables.get(&var_name.to_lowercase())
    }
    
    /// Add a comment.
    pub fn add_comment<S: Into<String>>(&mut self, comment: S) {
        self.comments.push(comment.into());
    }
    
    /// Set variable order.
    pub fn set_variable_order(&mut self, order: Vec<String>) {
        self.variable_order = order;
    }
}

/// Formatting options for individual variables.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VariableFormatting {
    /// Inline comment for this variable
    pub inline_comment: Option<String>,
    /// Custom indentation for this variable
    pub indentation: Option<String>,
    /// Whether this variable should have trailing comma
    pub trailing_comma: bool,
    /// Custom spacing around the assignment operator
    pub assignment_spacing: Option<AssignmentSpacing>,
    /// Array formatting style
    pub array_style: ArrayStyle,
}

impl VariableFormatting {
    /// Create new variable formatting.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set inline comment.
    pub fn with_comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.inline_comment = Some(comment.into());
        self
    }
    
    /// Set custom indentation.
    pub fn with_indentation<S: Into<String>>(mut self, indent: S) -> Self {
        self.indentation = Some(indent.into());
        self
    }
    
    /// Enable trailing comma.
    pub fn with_trailing_comma(mut self, trailing: bool) -> Self {
        self.trailing_comma = trailing;
        self
    }
    
    /// Set assignment spacing.
    pub fn with_assignment_spacing(mut self, spacing: AssignmentSpacing) -> Self {
        self.assignment_spacing = Some(spacing);
        self
    }
    
    /// Set array formatting style.
    pub fn with_array_style(mut self, style: ArrayStyle) -> Self {
        self.array_style = style;
        self
    }
}

/// Case style for identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStyle {
    /// Keep original case
    Preserve,
    /// Convert to lowercase
    Lower,
    /// Convert to uppercase
    Upper,
}

impl Default for CaseStyle {
    fn default() -> Self {
        CaseStyle::Preserve
    }
}

/// Spacing around assignment operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssignmentSpacing {
    /// Spaces before the '=' operator
    pub before: usize,
    /// Spaces after the '=' operator
    pub after: usize,
}

impl Default for AssignmentSpacing {
    fn default() -> Self {
        Self { before: 1, after: 1 }
    }
}

impl AssignmentSpacing {
    /// Create assignment spacing with equal padding.
    pub fn equal(spaces: usize) -> Self {
        Self { before: spaces, after: spaces }
    }
    
    /// Create compact assignment spacing (no spaces).
    pub fn compact() -> Self {
        Self { before: 0, after: 0 }
    }
    
    /// Create loose assignment spacing (2 spaces each side).
    pub fn loose() -> Self {
        Self { before: 2, after: 2 }
    }
}

/// Array formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayStyle {
    /// Inline array: arr = 1, 2, 3
    Inline,
    /// Multi-line array with elements on separate lines
    MultiLine,
    /// Compact array with minimal spacing
    Compact,
}

impl Default for ArrayStyle {
    fn default() -> Self {
        ArrayStyle::Inline
    }
}