// f90nmlrs/src/fortran_types/formatting.rs

//! Formatting options and output logic for Fortran values.

use super::value::FortranValue;

/// Formatting options for Fortran value output.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Whether to use uppercase for logical values
    pub uppercase: bool,
    /// Precision for floating-point numbers
    pub float_precision: Option<usize>,
    /// Whether to use exponential notation for small/large numbers
    pub exponential_threshold: Option<(f64, f64)>, // (min, max)
    /// Format for complex numbers: either "(re,im)" or "re+im*i"
    pub complex_format: ComplexFormat,
    /// How to handle string quoting
    pub string_quote_style: QuoteStyle,
    /// Whether to use Fortran-style double precision notation (D instead of E)
    pub use_fortran_double: bool,
    /// Maximum width for array elements before wrapping
    pub array_element_width: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComplexFormat {
    /// Standard Fortran format: (real, imag)
    Parentheses,
    /// Mathematical format: real + imag*i
    Mathematical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuoteStyle {
    /// Use single quotes (default Fortran style)
    Single,
    /// Use double quotes
    Double,
    /// Preserve original quote style if known
    Preserve,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            uppercase: false,
            float_precision: None,
            exponential_threshold: None,
            complex_format: ComplexFormat::Parentheses,
            string_quote_style: QuoteStyle::Single,
            use_fortran_double: false,
            array_element_width: None,
        }
    }
}

impl FortranValue {
    /// Format this value as it would appear in a Fortran namelist with basic options.
    pub fn to_fortran_string(&self, uppercase: bool) -> String {
        let options = FormatOptions {
            uppercase,
            ..Default::default()
        };
        self.to_fortran_string_with_options(&options)
    }

    /// Format this value with detailed formatting options.
    pub fn to_fortran_string_with_options(&self, options: &FormatOptions) -> String {
        match self {
            FortranValue::Integer(i) => i.to_string(),
            FortranValue::Real(f) => self.format_real(*f, options),
            FortranValue::Complex(r, i) => self.format_complex(*r, *i, options),
            FortranValue::Logical(b) => self.format_logical(*b, options),
            FortranValue::Character(s) => self.format_string(s, options),
            FortranValue::Array(arr) => self.format_array(arr, options),
            FortranValue::MultiArray { values, .. } => self.format_array(values, options),
            FortranValue::DerivedType(_) => {
                // Derived types are handled specially during output
                "<derived_type>".to_string()
            }
            FortranValue::DerivedTypeArray(_) => "<derived_type_array>".to_string(),
            FortranValue::Null => "".to_string(),
        }
    }

    fn format_real(&self, value: f64, options: &FormatOptions) -> String {
        if value.is_infinite() {
            if value > 0.0 {
                "+inf".to_string()
            } else {
                "-inf".to_string()
            }
        } else if value.is_nan() {
            "nan".to_string()
        } else {
            // Check if we should use exponential notation
            let use_exponential =
                if let Some((min_threshold, max_threshold)) = options.exponential_threshold {
                    let abs_val = value.abs();
                    abs_val != 0.0 && (abs_val < min_threshold || abs_val > max_threshold)
                } else {
                    false
                };

            if use_exponential {
                if options.use_fortran_double {
                    if let Some(precision) = options.float_precision {
                        format!("{:.precision$e}", value, precision = precision).replace('e', "d")
                    } else {
                        format!("{:e}", value).replace('e', "d")
                    }
                } else {
                    if let Some(precision) = options.float_precision {
                        format!("{:.precision$e}", value, precision = precision)
                    } else {
                        format!("{:e}", value)
                    }
                }
            } else {
                if let Some(precision) = options.float_precision {
                    format!("{:.precision$}", value, precision = precision)
                } else {
                    // Ensure real numbers always have a decimal point for roundtrip compatibility
                    let s = value.to_string();
                    if s.contains('.') || s.contains('e') || s.contains('E') {
                        s
                    } else {
                        format!("{}.0", s)
                    }
                }
            }
        }
    }

    fn format_complex(&self, real: f64, imag: f64, options: &FormatOptions) -> String {
        let real_str = FortranValue::Real(real).format_real(real, options);
        let imag_str = FortranValue::Real(imag).format_real(imag, options);

        match options.complex_format {
            ComplexFormat::Parentheses => format!("({}, {})", real_str, imag_str),
            ComplexFormat::Mathematical => {
                if imag >= 0.0 {
                    format!("{}+{}*i", real_str, imag_str)
                } else {
                    format!("{}{}*i", real_str, imag_str)
                }
            }
        }
    }

    fn format_logical(&self, value: bool, options: &FormatOptions) -> String {
        let base = if value { ".true." } else { ".false." };
        if options.uppercase {
            base.to_uppercase()
        } else {
            base.to_string()
        }
    }

    fn format_string(&self, value: &str, options: &FormatOptions) -> String {
        let quote_char = match options.string_quote_style {
            QuoteStyle::Single => '\'',
            QuoteStyle::Double => '"',
            QuoteStyle::Preserve => '\'', // Default to single if no preserved style
        };

        // Escape quotes by doubling them
        let escaped = if quote_char == '\'' {
            value.replace('\'', "''")
        } else {
            value.replace('"', "\"\"")
        };

        format!("{}{}{}", quote_char, escaped, quote_char)
    }

    fn format_array(&self, values: &[FortranValue], options: &FormatOptions) -> String {
        if values.is_empty() {
            return String::new();
        }

        let formatted_values: Vec<String> = values
            .iter()
            .map(|v| v.to_fortran_string_with_options(options))
            .collect();

        if let Some(max_width) = options.array_element_width {
            // Try to fit elements within specified width
            let mut result = String::new();
            let mut current_line_len = 0;

            for (i, val_str) in formatted_values.iter().enumerate() {
                if i > 0 {
                    if current_line_len + val_str.len() + 2 > max_width {
                        result.push_str(",\n    "); // New line with indentation
                        current_line_len = 4;
                    } else {
                        result.push_str(", ");
                        current_line_len += 2;
                    }
                }

                result.push_str(val_str);
                current_line_len += val_str.len();
            }

            result
        } else {
            formatted_values.join(", ")
        }
    }

    /// Create a value with repeat notation (for compact array representation).
    pub fn with_repeat_count(&self, count: usize) -> String {
        if count <= 1 {
            self.to_fortran_string(false)
        } else {
            format!("{}*{}", count, self.to_fortran_string(false))
        }
    }

    /// Try to detect repeated values in an array and use repeat notation.
    pub fn format_array_with_repeats(values: &[FortranValue], options: &FormatOptions) -> String {
        if values.is_empty() {
            return String::new();
        }

        let mut result = Vec::new();
        let mut current_value = &values[0];
        let mut count = 1;

        for value in values.iter().skip(1) {
            if value == current_value {
                count += 1;
            } else {
                // Output the current run
                if count == 1 {
                    result.push(current_value.to_fortran_string_with_options(options));
                } else {
                    result.push(format!(
                        "{}*{}",
                        count,
                        current_value.to_fortran_string_with_options(options)
                    ));
                }

                current_value = value;
                count = 1;
            }
        }

        // Output the final run
        if count == 1 {
            result.push(current_value.to_fortran_string_with_options(options));
        } else {
            result.push(format!(
                "{}*{}",
                count,
                current_value.to_fortran_string_with_options(options)
            ));
        }

        result.join(", ")
    }
}
