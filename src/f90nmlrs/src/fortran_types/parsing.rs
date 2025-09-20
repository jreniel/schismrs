// src/fortran_types/parsing.rs

//! String parsing functions for Fortran values.

use super::value::FortranValue;
use crate::error::{F90nmlError, Result};

/// Parse a string value as a specific Fortran type.
pub fn parse_fortran_value(value: &str, type_hint: Option<&str>) -> Result<FortranValue> {
    let trimmed = value.trim();

    // Handle null/empty values
    if trimmed.is_empty() {
        return Ok(FortranValue::Null);
    }

    // Try to parse based on type hint first
    if let Some(hint) = type_hint {
        match hint {
            "integer" => return parse_integer(trimmed),
            "real" => return parse_real(trimmed),
            "complex" => return parse_complex(trimmed),
            "logical" => return parse_logical(trimmed),
            "character" => return Ok(parse_character(trimmed)),
            _ => {}
        }
    }

    // Auto-detect type - order matters here!
    // Check for logical values first (they can contain alphabetic chars)
    if let Ok(val) = parse_logical(trimmed) {
        return Ok(val);
    }

    // Check for complex numbers (parentheses format)
    if let Ok(val) = parse_complex(trimmed) {
        return Ok(val);
    }

    // Check for real numbers (includes double precision notation)
    if let Ok(val) = parse_real(trimmed) {
        return Ok(val);
    }

    // Check for integers
    if let Ok(val) = parse_integer(trimmed) {
        return Ok(val);
    }

    // Default to character string
    Ok(parse_character(trimmed))
}

/// Parse an integer value.
pub fn parse_integer(value: &str) -> Result<FortranValue> {
    // Handle potential kind specifiers
    let clean_value = if let Some(underscore_pos) = value.find('_') {
        &value[..underscore_pos]
    } else {
        value
    };

    clean_value
        .parse::<i64>()
        .map(FortranValue::Integer)
        .map_err(|_| F90nmlError::invalid_value("", value, "integer"))
}

/// Parse a real value with enhanced Fortran double precision support.
pub fn parse_real(value: &str) -> Result<FortranValue> {
    let mut normalized = value.trim().to_string();

    // First check if this looks like a pure integer - if so, reject it for real parsing
    if looks_like_integer(&normalized) {
        return Err(F90nmlError::invalid_value("", value, "real"));
    }

    // Handle Fortran-style double precision notation (D instead of E)
    // This handles cases like: 1.0d0, 4184.d0, 1d-5, -2.5D+3
    if normalized.to_lowercase().contains('d') {
        // Replace 'd' with 'e' for Rust parsing, but preserve case for the exponent marker
        let lower = normalized.to_lowercase();
        if let Some(d_pos) = lower.find('d') {
            // Replace the 'd' or 'D' with 'e'
            normalized.replace_range(d_pos..d_pos + 1, "e");
        }
    }

    // Handle kind specifiers (remove them for parsing)
    let clean_value = if let Some(underscore_pos) = normalized.find('_') {
        &normalized[..underscore_pos]
    } else {
        &normalized
    };

    // Handle special Fortran real values
    match clean_value.to_lowercase().as_str() {
        "+inf" | "inf" | "+infinity" | "infinity" => return Ok(FortranValue::Real(f64::INFINITY)),
        "-inf" | "-infinity" => return Ok(FortranValue::Real(f64::NEG_INFINITY)),
        "nan" | "+nan" | "-nan" => return Ok(FortranValue::Real(f64::NAN)),
        _ => {}
    }

    // Try to parse as a floating point number
    clean_value
        .parse::<f64>()
        .map(FortranValue::Real)
        .map_err(|_| F90nmlError::invalid_value("", value, "real"))
}

/// Parse a complex value.
pub fn parse_complex(value: &str) -> Result<FortranValue> {
    let trimmed = value.trim();
    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
        return Err(F90nmlError::invalid_value("", value, "complex"));
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let parts: Vec<&str> = inner.split(',').collect();

    if parts.len() != 2 {
        return Err(F90nmlError::invalid_value("", value, "complex"));
    }

    let real = parse_real(parts[0].trim())?.as_real()?;
    let imag = parse_real(parts[1].trim())?.as_real()?;

    Ok(FortranValue::Complex(real, imag))
}

/// Parse a logical value with enhanced recognition.
pub fn parse_logical(value: &str) -> Result<FortranValue> {
    let lower = value.to_lowercase().trim().to_string();

    // Standard Fortran logical constants
    match lower.as_str() {
        ".true." | ".t." | "true" | "t" => Ok(FortranValue::Logical(true)),
        ".false." | ".f." | "false" | "f" => Ok(FortranValue::Logical(false)),
        _ => {
            // Try more flexible parsing for partial logical values
            if lower.starts_with(".t") {
                Ok(FortranValue::Logical(true))
            } else if lower.starts_with(".f") {
                Ok(FortranValue::Logical(false))
            } else {
                Err(F90nmlError::invalid_value("", value, "logical"))
            }
        }
    }
}

/// Parse a character string, handling quotes and escaping.
pub fn parse_character(value: &str) -> FortranValue {
    let trimmed = value.trim();

    // Handle quoted strings
    if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        || (trimmed.starts_with('"') && trimmed.ends_with('"'))
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        // Un-escape doubled quotes
        let unescaped = if trimmed.starts_with('\'') {
            inner.replace("''", "'")
        } else {
            inner.replace("\"\"", "\"")
        };
        FortranValue::Character(unescaped)
    } else {
        // Unquoted string
        FortranValue::Character(trimmed.to_string())
    }
}

/// Utility function to determine if a string looks like a real number.
pub fn looks_like_real(value: &str) -> bool {
    let trimmed = value.trim().to_lowercase();

    // Handle special float values first
    if trimmed.contains("inf") || trimmed.contains("nan") {
        return true;
    }

    // Check for decimal point - but make sure it's a numeric decimal, not logical delimiters
    if trimmed.contains('.') {
        // Exclude logical constants like .true. and .false.
        if trimmed.starts_with('.')
            && (trimmed.contains("true")
                || trimmed.contains("false")
                || trimmed.contains('t')
                || trimmed.contains('f'))
        {
            return false;
        }
        // Also check that there are digits around the decimal point
        if let Some(dot_pos) = trimmed.find('.') {
            let before_dot = &trimmed[..dot_pos];
            let after_dot = &trimmed[dot_pos + 1..];

            // At least one side of the decimal should have digits
            let has_digits_before = before_dot.chars().any(|c| c.is_ascii_digit());
            let has_digits_after = after_dot.chars().any(|c| c.is_ascii_digit());

            if has_digits_before || has_digits_after {
                return true;
            }
        }
    }

    // Check for scientific notation (e or d) - but only if it looks like a number
    if trimmed.contains('e') || trimmed.contains('d') {
        // Make sure it's actually scientific notation, not just a word containing 'e' or 'd'
        // Look for patterns like: 1e5, 2.5e-3, 4d0, etc.
        let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
        if has_digit {
            // Check if 'e' or 'd' is preceded by a digit and possibly followed by +/- and digits
            for (i, ch) in trimmed.char_indices() {
                if ch == 'e' || ch == 'd' {
                    // Check if there's a digit before this position
                    let before_ok = trimmed[..i].chars().any(|c| c.is_ascii_digit());
                    if before_ok {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Utility function to determine if a string looks like an integer.
pub fn looks_like_integer(value: &str) -> bool {
    let trimmed = value.trim();

    // Check for kind specifier
    let clean_value = if let Some(underscore_pos) = trimmed.find('_') {
        &trimmed[..underscore_pos]
    } else {
        trimmed
    };

    // Should be only digits and optional leading sign
    if clean_value.is_empty() {
        return false;
    }

    let mut chars = clean_value.chars();
    if let Some(first) = chars.next() {
        if first == '+' || first == '-' {
            chars.all(|c| c.is_ascii_digit())
        } else if first.is_ascii_digit() {
            chars.all(|c| c.is_ascii_digit())
        } else {
            false
        }
    } else {
        false
    }
}

/// Infer the Fortran type from a string value.
pub fn infer_fortran_type(value: &str) -> &'static str {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return "null";
    }

    // Check for logical values first
    if parse_logical(trimmed).is_ok() {
        return "logical";
    }

    // Check for complex values
    if trimmed.starts_with('(') && trimmed.ends_with(')') && trimmed.contains(',') {
        return "complex";
    }

    // Check for quoted strings
    if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        || (trimmed.starts_with('"') && trimmed.ends_with('"'))
    {
        return "character";
    }

    // Check for real numbers (including double precision)
    if looks_like_real(trimmed) {
        return "real";
    }

    // Check for integers
    if looks_like_integer(trimmed) {
        return "integer";
    }

    // Default to character
    "character"
}

/// Parse a value list like "1, 2, 3" or "1.0, 2.0, 3.0".
pub fn parse_value_list(input: &str, type_hint: Option<&str>) -> Result<Vec<FortranValue>> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut values = Vec::new();
    let mut current_value = String::new();
    let mut in_quotes = false;
    let mut quote_char = None;
    let mut paren_depth = 0;

    for ch in input.chars() {
        match ch {
            '\'' | '"' if !in_quotes => {
                in_quotes = true;
                quote_char = Some(ch);
                current_value.push(ch);
            }
            ch if in_quotes && Some(ch) == quote_char => {
                in_quotes = false;
                quote_char = None;
                current_value.push(ch);
            }
            '(' if !in_quotes => {
                paren_depth += 1;
                current_value.push(ch);
            }
            ')' if !in_quotes => {
                paren_depth -= 1;
                current_value.push(ch);
            }
            ',' if !in_quotes && paren_depth == 0 => {
                let trimmed = current_value.trim();
                if !trimmed.is_empty() {
                    values.push(parse_fortran_value(trimmed, type_hint)?);
                } else {
                    // Empty value (e.g., "1,,3" has an empty middle value)
                    values.push(FortranValue::Null);
                }
                current_value.clear();
            }
            _ => {
                current_value.push(ch);
            }
        }
    }

    // Handle the final value
    let trimmed = current_value.trim();
    if !trimmed.is_empty() {
        values.push(parse_fortran_value(trimmed, type_hint)?);
    } else if !values.is_empty() {
        // Trailing comma case
        values.push(FortranValue::Null);
    }

    Ok(values)
}

/// Parse a repeat count expression like "3*42" or "5*.true.".
pub fn parse_repeat_expression(input: &str) -> Result<(usize, FortranValue)> {
    if let Some(star_pos) = input.find('*') {
        let count_str = input[..star_pos].trim();
        let value_str = input[star_pos + 1..].trim();

        let count = count_str
            .parse::<usize>()
            .map_err(|_| F90nmlError::invalid_value("", count_str, "repeat count"))?;

        let value = if value_str.is_empty() {
            FortranValue::Null
        } else {
            parse_fortran_value(value_str, None)?
        };

        Ok((count, value))
    } else {
        // No repeat, just a single value
        Ok((1, parse_fortran_value(input, None)?))
    }
}

/// Validation constraints for parsed values.
#[derive(Debug, Clone, Default)]
pub struct ValueConstraints {
    pub integer_range: Option<(i64, i64)>,
    pub real_range: Option<(f64, f64)>,
    pub max_string_length: Option<usize>,
    pub max_array_length: Option<usize>,
    pub allowed_logical_formats: Option<Vec<String>>,
}

impl ValueConstraints {
    /// Create a new `ValueConstraints` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set integer range constraints.
    pub fn with_integer_range(mut self, min: i64, max: i64) -> Self {
        self.integer_range = Some((min, max));
        self
    }

    /// Set real range constraints.
    pub fn with_real_range(mut self, min: f64, max: f64) -> Self {
        self.real_range = Some((min, max));
        self
    }

    /// Set maximum string length constraint.
    pub fn with_max_string_length(mut self, max_len: usize) -> Self {
        self.max_string_length = Some(max_len);
        self
    }

    /// Set maximum array length constraint.
    pub fn with_max_array_length(mut self, max_len: usize) -> Self {
        self.max_array_length = Some(max_len);
        self
    }

    /// Set allowed logical format constraints.
    pub fn with_allowed_logical_formats(mut self, formats: Vec<String>) -> Self {
        self.allowed_logical_formats = Some(formats);
        self
    }
}

/// Validate a parsed value against expected constraints.
pub fn validate_parsed_value(value: &FortranValue, constraints: &ValueConstraints) -> Result<()> {
    match value {
        FortranValue::Integer(i) => {
            if let Some((min, max)) = constraints.integer_range {
                if *i < min || *i > max {
                    return Err(F90nmlError::ValidationError {
                        message: format!(
                            "Integer {} is outside allowed range [{}, {}]",
                            i, min, max
                        ),
                        group: None,
                        variable: None,
                    });
                }
            }
        }
        FortranValue::Real(f) => {
            if let Some((min, max)) = constraints.real_range {
                if *f < min || *f > max {
                    return Err(F90nmlError::ValidationError {
                        message: format!("Real {} is outside allowed range [{}, {}]", f, min, max),
                        group: None,
                        variable: None,
                    });
                }
            }
        }
        FortranValue::Character(s) => {
            if let Some(max_len) = constraints.max_string_length {
                if s.len() > max_len {
                    return Err(F90nmlError::ValidationError {
                        message: format!("String length {} exceeds maximum {}", s.len(), max_len),
                        group: None,
                        variable: None,
                    });
                }
            }
        }
        FortranValue::Array(arr) => {
            if let Some(max_len) = constraints.max_array_length {
                if arr.len() > max_len {
                    return Err(F90nmlError::ValidationError {
                        message: format!("Array length {} exceeds maximum {}", arr.len(), max_len),
                        group: None,
                        variable: None,
                    });
                }
            }

            // Recursively validate array elements
            for element in arr {
                validate_parsed_value(element, constraints)?;
            }
        }
        _ => {} // Other types don't have constraints yet
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_double_precision_notation() {
        // Test various double precision formats
        assert_eq!(parse_real("4184.d0").unwrap(), FortranValue::Real(4184.0));
        assert_eq!(parse_real("1.0d0").unwrap(), FortranValue::Real(1.0));
        assert_eq!(parse_real("1d5").unwrap(), FortranValue::Real(1e5));
        assert_eq!(parse_real("2.5D-3").unwrap(), FortranValue::Real(2.5e-3));
        assert_eq!(parse_real("-1.23d+2").unwrap(), FortranValue::Real(-123.0));

        // Test mixed case
        assert_eq!(parse_real("1.5D0").unwrap(), FortranValue::Real(1.5));
        assert_eq!(
            parse_real("3.14159d0").unwrap(),
            FortranValue::Real(3.14159)
        );
    }

    #[test]
    fn test_parse_real_with_kind_specifiers() {
        // Kind specifiers should be ignored for parsing
        assert_eq!(parse_real("1.0_dp").unwrap(), FortranValue::Real(1.0));
        assert_eq!(parse_real("2.5d0_8").unwrap(), FortranValue::Real(2.5));
        assert_eq!(parse_real("1e5_real64").unwrap(), FortranValue::Real(1e5));
    }

    #[test]
    fn test_infer_types() {
        assert_eq!(infer_fortran_type("4184.d0"), "real");
        assert_eq!(infer_fortran_type("42"), "integer");
        assert_eq!(infer_fortran_type(".true."), "logical");
        assert_eq!(infer_fortran_type("'hello'"), "character");
        assert_eq!(infer_fortran_type("(1.0, 2.0)"), "complex");
    }

    #[test]
    fn test_looks_like_functions() {
        assert!(looks_like_real("4184.d0"));
        assert!(looks_like_real("1.5"));
        assert!(looks_like_real("1e5"));
        assert!(looks_like_real("inf"));
        assert!(!looks_like_real("42")); // Pure integers don't look like reals
        assert!(!looks_like_real(".true."));

        assert!(looks_like_integer("42"));
        assert!(looks_like_integer("-123"));
        assert!(looks_like_integer("42_i8"));
        assert!(!looks_like_integer("4.2"));
        assert!(!looks_like_integer(".true."));
    }

    #[test]
    fn test_auto_detection_order() {
        // The auto-detection should prefer real over integer for double precision
        let val = parse_fortran_value("4184.d0", None).unwrap();
        assert!(matches!(val, FortranValue::Real(_)));

        // But regular integers should still be detected as integers
        let val = parse_fortran_value("42", None).unwrap();
        assert!(matches!(val, FortranValue::Integer(_)));
    }
}

