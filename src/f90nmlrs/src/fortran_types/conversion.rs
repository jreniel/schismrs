// f90nmlrs/src/fortran_types/conversion.rs

//! Type conversion implementations for FortranValue.

use super::value::FortranValue;
use std::collections::HashMap;

// Convenient From implementations for common Rust types
impl From<i32> for FortranValue {
    fn from(value: i32) -> Self {
        FortranValue::Integer(value as i64)
    }
}

impl From<i64> for FortranValue {
    fn from(value: i64) -> Self {
        FortranValue::Integer(value)
    }
}

impl From<f32> for FortranValue {
    fn from(value: f32) -> Self {
        FortranValue::Real(value as f64)
    }
}

impl From<f64> for FortranValue {
    fn from(value: f64) -> Self {
        FortranValue::Real(value)
    }
}

impl From<bool> for FortranValue {
    fn from(value: bool) -> Self {
        FortranValue::Logical(value)
    }
}

impl From<String> for FortranValue {
    fn from(value: String) -> Self {
        FortranValue::Character(value)
    }
}

impl From<&str> for FortranValue {
    fn from(value: &str) -> Self {
        FortranValue::Character(value.to_string())
    }
}

impl From<Vec<FortranValue>> for FortranValue {
    fn from(value: Vec<FortranValue>) -> Self {
        FortranValue::Array(value)
    }
}

impl From<HashMap<String, FortranValue>> for FortranValue {
    fn from(value: HashMap<String, FortranValue>) -> Self {
        FortranValue::DerivedType(value)
    }
}

impl From<(f64, f64)> for FortranValue {
    fn from((real, imag): (f64, f64)) -> Self {
        FortranValue::Complex(real, imag)
    }
}

// Additional convenience conversions for arrays of common types
impl From<Vec<i32>> for FortranValue {
    fn from(values: Vec<i32>) -> Self {
        let fortran_values: Vec<FortranValue> = values.into_iter().map(FortranValue::from).collect();
        FortranValue::Array(fortran_values)
    }
}

impl From<Vec<i64>> for FortranValue {
    fn from(values: Vec<i64>) -> Self {
        let fortran_values: Vec<FortranValue> = values.into_iter().map(FortranValue::from).collect();
        FortranValue::Array(fortran_values)
    }
}

impl From<Vec<f32>> for FortranValue {
    fn from(values: Vec<f32>) -> Self {
        let fortran_values: Vec<FortranValue> = values.into_iter().map(FortranValue::from).collect();
        FortranValue::Array(fortran_values)
    }
}

impl From<Vec<f64>> for FortranValue {
    fn from(values: Vec<f64>) -> Self {
        let fortran_values: Vec<FortranValue> = values.into_iter().map(FortranValue::from).collect();
        FortranValue::Array(fortran_values)
    }
}

impl From<Vec<bool>> for FortranValue {
    fn from(values: Vec<bool>) -> Self {
        let fortran_values: Vec<FortranValue> = values.into_iter().map(FortranValue::from).collect();
        FortranValue::Array(fortran_values)
    }
}

impl From<Vec<String>> for FortranValue {
    fn from(values: Vec<String>) -> Self {
        let fortran_values: Vec<FortranValue> = values.into_iter().map(FortranValue::from).collect();
        FortranValue::Array(fortran_values)
    }
}

impl From<Vec<&str>> for FortranValue {
    fn from(values: Vec<&str>) -> Self {
        let fortran_values: Vec<FortranValue> = values.into_iter().map(FortranValue::from).collect();
        FortranValue::Array(fortran_values)
    }
}

// Conversion from Option types (None becomes Null)
impl From<Option<i32>> for FortranValue {
    fn from(value: Option<i32>) -> Self {
        match value {
            Some(v) => FortranValue::from(v),
            None => FortranValue::Null,
        }
    }
}

impl From<Option<i64>> for FortranValue {
    fn from(value: Option<i64>) -> Self {
        match value {
            Some(v) => FortranValue::from(v),
            None => FortranValue::Null,
        }
    }
}

impl From<Option<f32>> for FortranValue {
    fn from(value: Option<f32>) -> Self {
        match value {
            Some(v) => FortranValue::from(v),
            None => FortranValue::Null,
        }
    }
}

impl From<Option<f64>> for FortranValue {
    fn from(value: Option<f64>) -> Self {
        match value {
            Some(v) => FortranValue::from(v),
            None => FortranValue::Null,
        }
    }
}

impl From<Option<bool>> for FortranValue {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(v) => FortranValue::from(v),
            None => FortranValue::Null,
        }
    }
}

impl From<Option<String>> for FortranValue {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(v) => FortranValue::from(v),
            None => FortranValue::Null,
        }
    }
}

impl From<Option<&str>> for FortranValue {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(v) => FortranValue::from(v),
            None => FortranValue::Null,
        }
    }
}

// Conversion to common Rust types (fallible)
impl TryFrom<FortranValue> for i32 {
    type Error = crate::error::F90nmlError;
    
    fn try_from(value: FortranValue) -> Result<Self, Self::Error> {
        let i64_val = value.as_integer()?;
        if i64_val >= i32::MIN as i64 && i64_val <= i32::MAX as i64 {
            Ok(i64_val as i32)
        } else {
            Err(crate::error::F90nmlError::TypeConversion {
                from: value.type_name().to_string(),
                to: "i32".to_string(),
                value: value.to_string(),
            })
        }
    }
}

impl TryFrom<FortranValue> for i64 {
    type Error = crate::error::F90nmlError;
    
    fn try_from(value: FortranValue) -> Result<Self, Self::Error> {
        value.as_integer()
    }
}

impl TryFrom<FortranValue> for f32 {
    type Error = crate::error::F90nmlError;
    
    fn try_from(value: FortranValue) -> Result<Self, Self::Error> {
        let f64_val = value.as_real()?;
        if f64_val.is_finite() {
            Ok(f64_val as f32)
        } else {
            Ok(f64_val as f32) // Allow infinities and NaN
        }
    }
}

impl TryFrom<FortranValue> for f64 {
    type Error = crate::error::F90nmlError;
    
    fn try_from(value: FortranValue) -> Result<Self, Self::Error> {
        value.as_real()
    }
}

impl TryFrom<FortranValue> for bool {
    type Error = crate::error::F90nmlError;
    
    fn try_from(value: FortranValue) -> Result<Self, Self::Error> {
        value.as_logical()
    }
}

impl TryFrom<FortranValue> for String {
    type Error = crate::error::F90nmlError;
    
    fn try_from(value: FortranValue) -> Result<Self, Self::Error> {
        match value {
            FortranValue::Character(s) => Ok(s),
            _ => Err(crate::error::F90nmlError::TypeConversion {
                from: value.type_name().to_string(),
                to: "String".to_string(),
                value: value.to_string(),
            }),
        }
    }
}

impl TryFrom<FortranValue> for (f64, f64) {
    type Error = crate::error::F90nmlError;
    
    fn try_from(value: FortranValue) -> Result<Self, Self::Error> {
        value.as_complex()
    }
}

impl TryFrom<FortranValue> for Vec<FortranValue> {
    type Error = crate::error::F90nmlError;
    
    fn try_from(value: FortranValue) -> Result<Self, Self::Error> {
        match value {
            FortranValue::Array(arr) => Ok(arr),
            FortranValue::MultiArray { values, .. } => Ok(values),
            _ => Err(crate::error::F90nmlError::TypeConversion {
                from: value.type_name().to_string(),
                to: "Vec<FortranValue>".to_string(),
                value: value.to_string(),
            }),
        }
    }
}
