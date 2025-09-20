// f90nmlrs/src/namelist/validation.rs

//! Validation logic for namelists and groups.

use crate::error::{F90nmlError, Result};
use crate::fortran_types::FortranValue;
use std::collections::HashMap;

/// Validate a namelist for consistency.
pub fn validate_namelist(groups: &HashMap<String, super::NamelistGroup>) -> Result<()> {
    for (group_name, group) in groups {
        group.validate(group_name)?;
    }
    Ok(())
}

/// Validate a group's variables for consistency.
pub fn validate_group_variables(
    variables: &HashMap<String, FortranValue>,
    group_name: &str,
) -> Result<()> {
    // Check for any obvious inconsistencies
    for (var_name, value) in variables {
        match value {
            FortranValue::Array(arr) => {
                if arr.is_empty() {
                    continue; // Empty arrays are okay
                }
                
                // Check that all elements have compatible types
                let first_type = arr[0].type_name();
                for (i, elem) in arr.iter().enumerate().skip(1) {
                    if elem.type_name() != first_type && !elem.is_null() && !arr[0].is_null() {
                        return Err(F90nmlError::InvalidValue {
                            variable: format!("{}%{}", group_name, var_name),
                            value: format!("element {} has type {}", i, elem.type_name()),
                            expected_type: first_type.to_string(),
                        });
                    }
                }
            }
            FortranValue::MultiArray { values, dimensions, .. } => {
                let expected_size: usize = dimensions.iter().product();
                if values.len() != expected_size {
                    return Err(F90nmlError::InvalidValue {
                        variable: format!("{}%{}", group_name, var_name),
                        value: format!("array has {} elements", values.len()),
                        expected_type: format!("array with {} elements", expected_size),
                    });
                }
            }
            _ => {}
        }
    }
    
    Ok(())
}