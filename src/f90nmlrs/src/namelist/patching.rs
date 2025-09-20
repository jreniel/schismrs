// f90nmlrs/src/namelist/patching.rs

//! Patch application and merging strategies for namelists.

use crate::error::Result;
use crate::fortran_types::FortranValue;

/// Strategy for merging namelists and groups.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeStrategy {
    /// Replace existing values completely
    Replace,
    /// Update existing values, add new ones
    Update,
    /// Append to arrays, update scalars
    Append,
    /// Skip if variable already exists
    SkipExisting,
}

/// Merge two FortranValues intelligently.
pub fn merge_values(existing: &FortranValue, new: &FortranValue) -> Result<FortranValue> {
    match (existing, new) {
        // Scalar replacement
        (_, new_val)
            if !matches!(
                new_val,
                FortranValue::Array(_) | FortranValue::MultiArray { .. }
            ) =>
        {
            Ok(new.clone())
        }

        // Array merging
        (FortranValue::Array(_), FortranValue::Array(new_arr)) => {
            Ok(FortranValue::Array(new_arr.clone()))
        }

        // Convert scalar to array and merge
        (existing_scalar, FortranValue::Array(new_arr)) => {
            let mut result = vec![existing_scalar.clone()];
            result.extend(new_arr.iter().cloned());
            Ok(FortranValue::Array(result))
        }

        // Derived type merging
        (FortranValue::DerivedType(existing_fields), FortranValue::DerivedType(new_fields)) => {
            let mut merged = existing_fields.clone();
            for (key, value) in new_fields {
                merged.insert(key.clone(), value.clone());
            }
            Ok(FortranValue::DerivedType(merged))
        }

        // Default: replace with new value
        _ => Ok(new.clone()),
    }
}

/// Append values together (for append merge strategy).
pub fn append_values(existing: &FortranValue, new: &FortranValue) -> Result<FortranValue> {
    match (existing, new) {
        // Array appending
        (FortranValue::Array(existing_arr), FortranValue::Array(new_arr)) => {
            let mut result = existing_arr.clone();
            result.extend(new_arr.iter().cloned());
            Ok(FortranValue::Array(result))
        }

        // Append scalar to array
        (FortranValue::Array(existing_arr), new_scalar) => {
            let mut result = existing_arr.clone();
            result.push(new_scalar.clone());
            Ok(FortranValue::Array(result))
        }

        // Convert scalars to array
        (existing_scalar, new_scalar) if !matches!(existing_scalar, FortranValue::Array(_)) => {
            Ok(FortranValue::Array(vec![
                existing_scalar.clone(),
                new_scalar.clone(),
            ]))
        }

        // For everything else, just replace
        _ => Ok(new.clone()),
    }
}

