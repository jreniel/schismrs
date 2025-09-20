// f90nmlrs/src/namelist/mod.rs

//! Fortran namelist data structures with advanced patching support.
//!
//! This module provides the core namelist and group structures with
//! sophisticated patch application that can preserve formatting and
//! handle complex merge scenarios.

pub mod core;
pub mod group;
pub mod formatting;
pub mod patching;
pub mod validation;

// Re-export the main types
pub use core::Namelist;
pub use group::NamelistGroup;
pub use formatting::{FormattingHints, GroupFormattingHints, VariableFormatting, CaseStyle};
pub use patching::MergeStrategy;

// Re-export helper functions
pub use patching::{merge_values, append_values};