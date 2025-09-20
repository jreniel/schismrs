// f90nmlrs/src/error.rs

//! Error types for the f90nml library with enhanced template and patching support.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type alias for f90nml operations.
pub type Result<T> = std::result::Result<T, F90nmlError>;

/// Errors that can occur when parsing, writing, or patching Fortran namelists.
#[derive(Debug, Clone, PartialEq)]
pub enum F90nmlError {
    /// I/O error when reading or writing files
    Io(String),

    /// Parse error with position and message
    Parse {
        message: String,
        line: usize,
        column: usize,
    },

    /// Invalid syntax in the namelist
    InvalidSyntax { message: String, position: usize },

    /// Unexpected end of file
    UnexpectedEof,

    /// Invalid token encountered
    InvalidToken {
        token: String,
        expected: Vec<String>,
        position: usize,
    },

    /// Invalid value for a variable
    InvalidValue {
        variable: String,
        value: String,
        expected_type: String,
    },

    /// Invalid array index
    InvalidIndex {
        variable: String,
        index: String,
        message: String,
    },

    /// Duplicate group or variable name
    Duplicate {
        name: String,
        item_type: String, // "group" or "variable"
    },

    /// Variable not found
    VariableNotFound { variable: String, group: String },

    /// Group not found
    GroupNotFound { group: String },

    /// Type conversion error
    TypeConversion {
        from: String,
        to: String,
        value: String,
    },

    /// File already exists (when force=false)
    FileAlreadyExists(PathBuf),

    /// Invalid format specification
    InvalidFormat { format: String, message: String },

    /// Template-related errors
    Template {
        message: String,
        template_position: Option<usize>,
    },

    /// Patch application errors
    PatchError {
        message: String,
        group: Option<String>,
        variable: Option<String>,
    },

    /// Incompatible patch (e.g., trying to patch array with scalar)
    IncompatiblePatch {
        variable: String,
        original_type: String,
        patch_type: String,
    },

    /// Missing template information required for patching
    MissingTemplateInfo { operation: String },

    /// Array dimension mismatch
    DimensionMismatch {
        variable: String,
        expected: Vec<usize>,
        actual: Vec<usize>,
    },

    /// Validation error
    ValidationError {
        message: String,
        group: Option<String>,
        variable: Option<String>,
    },

    /// Circular reference in derived types
    CircularReference { path: Vec<String> },

    /// Maximum nesting depth exceeded
    MaxDepthExceeded {
        max_depth: usize,
        current_depth: usize,
    },

    /// Encoding error when reading/writing files
    EncodingError { message: String, encoding: String },

    /// Serialization/deserialization error
    #[cfg(feature = "json")]
    Json(String),

    /// YAML serialization/deserialization error
    #[cfg(feature = "yaml")]
    Yaml(String),

    /// Custom error message
    Custom(String),
}

impl fmt::Display for F90nmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            F90nmlError::Io(msg) => write!(f, "I/O error: {}", msg),

            F90nmlError::Parse {
                message,
                line,
                column,
            } => {
                write!(
                    f,
                    "Parse error at line {}, column {}: {}",
                    line, column, message
                )
            }

            F90nmlError::InvalidSyntax { message, position } => {
                write!(f, "Invalid syntax at position {}: {}", position, message)
            }

            F90nmlError::UnexpectedEof => {
                write!(f, "Unexpected end of file")
            }

            F90nmlError::InvalidToken {
                token,
                expected,
                position,
            } => {
                write!(
                    f,
                    "Invalid token '{}' at position {}. Expected one of: {}",
                    token,
                    position,
                    expected.join(", ")
                )
            }

            F90nmlError::InvalidValue {
                variable,
                value,
                expected_type,
            } => {
                write!(
                    f,
                    "Invalid value '{}' for variable '{}'. Expected type: {}",
                    value, variable, expected_type
                )
            }

            F90nmlError::InvalidIndex {
                variable,
                index,
                message,
            } => {
                write!(
                    f,
                    "Invalid index '{}' for variable '{}': {}",
                    index, variable, message
                )
            }

            F90nmlError::Duplicate { name, item_type } => {
                write!(f, "Duplicate {} name: '{}'", item_type, name)
            }

            F90nmlError::VariableNotFound { variable, group } => {
                write!(f, "Variable '{}' not found in group '{}'", variable, group)
            }

            F90nmlError::GroupNotFound { group } => {
                write!(f, "Group '{}' not found", group)
            }

            F90nmlError::TypeConversion { from, to, value } => {
                write!(f, "Cannot convert '{}' from {} to {}", value, from, to)
            }

            F90nmlError::FileAlreadyExists(path) => {
                write!(f, "File already exists: {}", path.display())
            }

            F90nmlError::InvalidFormat { format, message } => {
                write!(f, "Invalid format '{}': {}", format, message)
            }

            F90nmlError::Template {
                message,
                template_position,
            } => {
                if let Some(pos) = template_position {
                    write!(f, "Template error at position {}: {}", pos, message)
                } else {
                    write!(f, "Template error: {}", message)
                }
            }

            F90nmlError::PatchError {
                message,
                group,
                variable,
            } => match (group, variable) {
                (Some(g), Some(v)) => write!(
                    f,
                    "Patch error in group '{}', variable '{}': {}",
                    g, v, message
                ),
                (Some(g), None) => write!(f, "Patch error in group '{}': {}", g, message),
                (None, Some(v)) => write!(f, "Patch error with variable '{}': {}", v, message),
                (None, None) => write!(f, "Patch error: {}", message),
            },

            F90nmlError::IncompatiblePatch {
                variable,
                original_type,
                patch_type,
            } => {
                write!(
                    f,
                    "Cannot patch variable '{}': incompatible types ({} vs {})",
                    variable, original_type, patch_type
                )
            }

            F90nmlError::MissingTemplateInfo { operation } => {
                write!(
                    f,
                    "Missing template information required for operation: {}",
                    operation
                )
            }

            F90nmlError::DimensionMismatch {
                variable,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Dimension mismatch for variable '{}': expected {:?}, got {:?}",
                    variable, expected, actual
                )
            }

            F90nmlError::ValidationError {
                message,
                group,
                variable,
            } => match (group, variable) {
                (Some(g), Some(v)) => write!(
                    f,
                    "Validation error in group '{}', variable '{}': {}",
                    g, v, message
                ),
                (Some(g), None) => write!(f, "Validation error in group '{}': {}", g, message),
                (None, Some(v)) => write!(f, "Validation error with variable '{}': {}", v, message),
                (None, None) => write!(f, "Validation error: {}", message),
            },

            F90nmlError::CircularReference { path } => {
                write!(f, "Circular reference detected: {}", path.join(" -> "))
            }

            F90nmlError::MaxDepthExceeded {
                max_depth,
                current_depth,
            } => {
                write!(
                    f,
                    "Maximum nesting depth exceeded: {} > {}",
                    current_depth, max_depth
                )
            }

            F90nmlError::EncodingError { message, encoding } => {
                write!(f, "Encoding error ({}): {}", encoding, message)
            }

            #[cfg(feature = "json")]
            F90nmlError::Json(msg) => write!(f, "JSON error: {}", msg),

            #[cfg(feature = "yaml")]
            F90nmlError::Yaml(msg) => write!(f, "YAML error: {}", msg),

            F90nmlError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for F90nmlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Most of our errors don't have a source, but we could add
        // source tracking for I/O errors, etc.
        None
    }
}

impl From<io::Error> for F90nmlError {
    fn from(err: io::Error) -> Self {
        F90nmlError::Io(err.to_string())
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for F90nmlError {
    fn from(err: serde_json::Error) -> Self {
        F90nmlError::Json(err.to_string())
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for F90nmlError {
    fn from(err: serde_yaml::Error) -> Self {
        F90nmlError::Yaml(err.to_string())
    }
}

impl F90nmlError {
    /// Create a new parse error.
    pub fn parse_error<S: Into<String>>(message: S, line: usize, column: usize) -> Self {
        F90nmlError::Parse {
            message: message.into(),
            line,
            column,
        }
    }

    /// Create a new invalid syntax error.
    pub fn invalid_syntax<S: Into<String>>(message: S, position: usize) -> Self {
        F90nmlError::InvalidSyntax {
            message: message.into(),
            position,
        }
    }

    /// Create a new invalid token error.
    pub fn invalid_token<S: Into<String>>(
        token: S,
        expected: Vec<String>,
        position: usize,
    ) -> Self {
        F90nmlError::InvalidToken {
            token: token.into(),
            expected,
            position,
        }
    }

    /// Create a new invalid value error.
    pub fn invalid_value<S: Into<String>>(variable: S, value: S, expected_type: S) -> Self {
        F90nmlError::InvalidValue {
            variable: variable.into(),
            value: value.into(),
            expected_type: expected_type.into(),
        }
    }

    /// Create a new invalid index error.
    pub fn invalid_index<S: Into<String>>(variable: S, index: S, message: S) -> Self {
        F90nmlError::InvalidIndex {
            variable: variable.into(),
            index: index.into(),
            message: message.into(),
        }
    }

    /// Create a new template error.
    pub fn template_error<S: Into<String>>(message: S, position: Option<usize>) -> Self {
        F90nmlError::Template {
            message: message.into(),
            template_position: position,
        }
    }

    /// Create a new patch error.
    pub fn patch_error<S: Into<String>>(
        message: S,
        group: Option<String>,
        variable: Option<String>,
    ) -> Self {
        F90nmlError::PatchError {
            message: message.into(),
            group,
            variable,
        }
    }

    /// Create a new incompatible patch error.
    pub fn incompatible_patch<S: Into<String>>(
        variable: S,
        original_type: S,
        patch_type: S,
    ) -> Self {
        F90nmlError::IncompatiblePatch {
            variable: variable.into(),
            original_type: original_type.into(),
            patch_type: patch_type.into(),
        }
    }

    /// Create a new missing template info error.
    pub fn missing_template_info<S: Into<String>>(operation: S) -> Self {
        F90nmlError::MissingTemplateInfo {
            operation: operation.into(),
        }
    }

    /// Create a new dimension mismatch error.
    pub fn dimension_mismatch<S: Into<String>>(
        variable: S,
        expected: Vec<usize>,
        actual: Vec<usize>,
    ) -> Self {
        F90nmlError::DimensionMismatch {
            variable: variable.into(),
            expected,
            actual,
        }
    }

    /// Create a new validation error.
    pub fn validation_error<S: Into<String>>(
        message: S,
        group: Option<String>,
        variable: Option<String>,
    ) -> Self {
        F90nmlError::ValidationError {
            message: message.into(),
            group,
            variable,
        }
    }

    /// Create a new circular reference error.
    pub fn circular_reference(path: Vec<String>) -> Self {
        F90nmlError::CircularReference { path }
    }

    /// Create a new max depth exceeded error.
    pub fn max_depth_exceeded(max_depth: usize, current_depth: usize) -> Self {
        F90nmlError::MaxDepthExceeded {
            max_depth,
            current_depth,
        }
    }

    /// Create a new encoding error.
    pub fn encoding_error<S: Into<String>>(message: S, encoding: S) -> Self {
        F90nmlError::EncodingError {
            message: message.into(),
            encoding: encoding.into(),
        }
    }

    /// Create a new custom error.
    pub fn custom<S: Into<String>>(message: S) -> Self {
        F90nmlError::Custom(message.into())
    }

    /// Get the error category for logging/metrics purposes.
    pub fn category(&self) -> &'static str {
        match self {
            F90nmlError::Io(_) => "io",
            F90nmlError::Parse { .. } => "parse",
            F90nmlError::InvalidSyntax { .. } => "syntax",
            F90nmlError::UnexpectedEof => "eof",
            F90nmlError::InvalidToken { .. } => "token",
            F90nmlError::InvalidValue { .. } => "value",
            F90nmlError::InvalidIndex { .. } => "index",
            F90nmlError::Duplicate { .. } => "duplicate",
            F90nmlError::VariableNotFound { .. } => "not_found",
            F90nmlError::GroupNotFound { .. } => "not_found",
            F90nmlError::TypeConversion { .. } => "conversion",
            F90nmlError::FileAlreadyExists(_) => "file_exists",
            F90nmlError::InvalidFormat { .. } => "format",
            F90nmlError::Template { .. } => "template",
            F90nmlError::PatchError { .. } => "patch",
            F90nmlError::IncompatiblePatch { .. } => "patch",
            F90nmlError::MissingTemplateInfo { .. } => "template",
            F90nmlError::DimensionMismatch { .. } => "dimension",
            F90nmlError::ValidationError { .. } => "validation",
            F90nmlError::CircularReference { .. } => "circular",
            F90nmlError::MaxDepthExceeded { .. } => "depth",
            F90nmlError::EncodingError { .. } => "encoding",
            #[cfg(feature = "json")]
            F90nmlError::Json(_) => "json",
            #[cfg(feature = "yaml")]
            F90nmlError::Yaml(_) => "yaml",
            F90nmlError::Custom(_) => "custom",
        }
    }

    /// Check if this is a recoverable error.
    pub fn is_recoverable(&self) -> bool {
        match self {
            // I/O errors are usually not recoverable
            F90nmlError::Io(_) => false,
            F90nmlError::FileAlreadyExists(_) => true, // Can use force=true
            F90nmlError::EncodingError { .. } => false,

            // Parse errors are generally not recoverable
            F90nmlError::Parse { .. } => false,
            F90nmlError::InvalidSyntax { .. } => false,
            F90nmlError::UnexpectedEof => false,
            F90nmlError::InvalidToken { .. } => false,

            // Value errors might be recoverable with different input
            F90nmlError::InvalidValue { .. } => true,
            F90nmlError::InvalidIndex { .. } => true,
            F90nmlError::TypeConversion { .. } => true,
            F90nmlError::InvalidFormat { .. } => true,

            // Structural errors
            F90nmlError::Duplicate { .. } => true,
            F90nmlError::VariableNotFound { .. } => true,
            F90nmlError::GroupNotFound { .. } => true,
            F90nmlError::DimensionMismatch { .. } => true,
            F90nmlError::ValidationError { .. } => true,
            F90nmlError::CircularReference { .. } => false,
            F90nmlError::MaxDepthExceeded { .. } => false,

            // Template and patch errors
            F90nmlError::Template { .. } => true,
            F90nmlError::PatchError { .. } => true,
            F90nmlError::IncompatiblePatch { .. } => true,
            F90nmlError::MissingTemplateInfo { .. } => false,

            // Serialization errors
            #[cfg(feature = "json")]
            F90nmlError::Json(_) => true,
            #[cfg(feature = "yaml")]
            F90nmlError::Yaml(_) => true,

            F90nmlError::Custom(_) => true,
        }
    }

    /// Get contextual information about where this error occurred.
    pub fn context(&self) -> ErrorContext {
        match self {
            F90nmlError::Parse { line, column, .. } => ErrorContext {
                line: Some(*line),
                column: Some(*column),
                group: None,
                variable: None,
            },
            F90nmlError::InvalidSyntax { position: _, .. } => ErrorContext {
                line: None,
                column: None,
                group: None,
                variable: None,
            },
            F90nmlError::InvalidValue { variable, .. } => ErrorContext {
                line: None,
                column: None,
                group: None,
                variable: Some(variable.clone()),
            },
            F90nmlError::VariableNotFound { variable, group } => ErrorContext {
                line: None,
                column: None,
                group: Some(group.clone()),
                variable: Some(variable.clone()),
            },
            F90nmlError::GroupNotFound { group } => ErrorContext {
                line: None,
                column: None,
                group: Some(group.clone()),
                variable: None,
            },
            F90nmlError::PatchError {
                group, variable, ..
            } => ErrorContext {
                line: None,
                column: None,
                group: group.clone(),
                variable: variable.clone(),
            },
            F90nmlError::ValidationError {
                group, variable, ..
            } => ErrorContext {
                line: None,
                column: None,
                group: group.clone(),
                variable: variable.clone(),
            },
            _ => ErrorContext::empty(),
        }
    }

    /// Create a detailed error report for debugging.
    pub fn detailed_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("Error Category: {}\n", self.category()));
        report.push_str(&format!("Recoverable: {}\n", self.is_recoverable()));
        report.push_str(&format!("Message: {}\n", self));

        let context = self.context();
        if !context.is_empty() {
            report.push_str("\nContext:\n");
            if let Some(line) = context.line {
                report.push_str(&format!("  Line: {}\n", line));
            }
            if let Some(column) = context.column {
                report.push_str(&format!("  Column: {}\n", column));
            }
            if let Some(ref group) = context.group {
                report.push_str(&format!("  Group: {}\n", group));
            }
            if let Some(ref variable) = context.variable {
                report.push_str(&format!("  Variable: {}\n", variable));
            }
        }

        report
    }
}

/// Context information about where an error occurred.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub group: Option<String>,
    pub variable: Option<String>,
}

impl ErrorContext {
    /// Create an empty context.
    pub fn empty() -> Self {
        Self {
            line: None,
            column: None,
            group: None,
            variable: None,
        }
    }

    /// Check if this context is empty.
    pub fn is_empty(&self) -> bool {
        self.line.is_none()
            && self.column.is_none()
            && self.group.is_none()
            && self.variable.is_none()
    }

    /// Create a context with position information.
    pub fn with_position(line: usize, column: usize) -> Self {
        Self {
            line: Some(line),
            column: Some(column),
            group: None,
            variable: None,
        }
    }

    /// Create a context with group and variable information.
    pub fn with_location(group: Option<String>, variable: Option<String>) -> Self {
        Self {
            line: None,
            column: None,
            group,
            variable,
        }
    }
}

/// Error severity levels for different types of issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Warning - operation can continue but there may be issues
    Warning,
    /// Error - operation failed but may be retryable
    Error,
    /// Fatal - operation failed and cannot be retried
    Fatal,
}

impl F90nmlError {
    /// Get the severity level of this error.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Fatal errors that cannot be recovered from
            F90nmlError::UnexpectedEof => ErrorSeverity::Fatal,
            F90nmlError::CircularReference { .. } => ErrorSeverity::Fatal,
            F90nmlError::MaxDepthExceeded { .. } => ErrorSeverity::Fatal,
            F90nmlError::MissingTemplateInfo { .. } => ErrorSeverity::Fatal,

            // Regular errors that might be fixable
            F90nmlError::Io(_) => ErrorSeverity::Error,
            F90nmlError::Parse { .. } => ErrorSeverity::Error,
            F90nmlError::InvalidSyntax { .. } => ErrorSeverity::Error,
            F90nmlError::InvalidToken { .. } => ErrorSeverity::Error,
            F90nmlError::InvalidValue { .. } => ErrorSeverity::Error,
            F90nmlError::InvalidIndex { .. } => ErrorSeverity::Error,
            F90nmlError::TypeConversion { .. } => ErrorSeverity::Error,
            F90nmlError::InvalidFormat { .. } => ErrorSeverity::Error,
            F90nmlError::Template { .. } => ErrorSeverity::Error,
            F90nmlError::PatchError { .. } => ErrorSeverity::Error,
            F90nmlError::IncompatiblePatch { .. } => ErrorSeverity::Error,
            F90nmlError::DimensionMismatch { .. } => ErrorSeverity::Error,
            F90nmlError::ValidationError { .. } => ErrorSeverity::Error,
            F90nmlError::EncodingError { .. } => ErrorSeverity::Error,

            // Warnings for issues that don't prevent operation
            F90nmlError::Duplicate { .. } => ErrorSeverity::Warning,
            F90nmlError::VariableNotFound { .. } => ErrorSeverity::Warning,
            F90nmlError::GroupNotFound { .. } => ErrorSeverity::Warning,
            F90nmlError::FileAlreadyExists(_) => ErrorSeverity::Warning,

            #[cfg(feature = "json")]
            F90nmlError::Json(_) => ErrorSeverity::Error,
            #[cfg(feature = "yaml")]
            F90nmlError::Yaml(_) => ErrorSeverity::Error,

            F90nmlError::Custom(_) => ErrorSeverity::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_error_display() {
        let err = F90nmlError::parse_error("Invalid token", 5, 10);
        assert_eq!(
            err.to_string(),
            "Parse error at line 5, column 10: Invalid token"
        );

        let err = F90nmlError::GroupNotFound {
            group: "missing".to_string(),
        };
        assert_eq!(err.to_string(), "Group 'missing' not found");

        let err = F90nmlError::FileAlreadyExists(Path::new("/tmp/test.nml").to_path_buf());
        assert_eq!(err.to_string(), "File already exists: /tmp/test.nml");
    }

    #[test]
    fn test_error_constructors() {
        let err = F90nmlError::invalid_token("&", vec!["identifier".to_string()], 42);
        match err {
            F90nmlError::InvalidToken {
                token,
                expected,
                position,
            } => {
                assert_eq!(token, "&");
                assert_eq!(expected, vec!["identifier"]);
                assert_eq!(position, 42);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_template_errors() {
        let err = F90nmlError::template_error("Invalid template syntax", Some(100));
        match err {
            F90nmlError::Template {
                message,
                template_position,
            } => {
                assert_eq!(message, "Invalid template syntax");
                assert_eq!(template_position, Some(100));
            }
            _ => panic!("Wrong error type"),
        }

        let err = F90nmlError::missing_template_info("patching");
        assert_eq!(
            err.to_string(),
            "Missing template information required for operation: patching"
        );
    }

    #[test]
    fn test_patch_errors() {
        let err = F90nmlError::patch_error(
            "Value mismatch",
            Some("data_nml".to_string()),
            Some("x".to_string()),
        );
        assert_eq!(
            err.to_string(),
            "Patch error in group 'data_nml', variable 'x': Value mismatch"
        );

        let err = F90nmlError::incompatible_patch("arr", "integer", "real");
        assert_eq!(
            err.to_string(),
            "Cannot patch variable 'arr': incompatible types (integer vs real)"
        );
    }

    #[test]
    fn test_dimension_mismatch() {
        let err = F90nmlError::dimension_mismatch("matrix", vec![3, 3], vec![2, 4]);
        assert_eq!(
            err.to_string(),
            "Dimension mismatch for variable 'matrix': expected [3, 3], got [2, 4]"
        );
    }

    #[test]
    fn test_validation_errors() {
        let err = F90nmlError::validation_error(
            "Array index out of bounds",
            Some("test_nml".to_string()),
            Some("arr".to_string()),
        );
        assert_eq!(
            err.to_string(),
            "Validation error in group 'test_nml', variable 'arr': Array index out of bounds"
        );
    }

    #[test]
    fn test_circular_reference() {
        let path = vec![
            "type_a".to_string(),
            "type_b".to_string(),
            "type_a".to_string(),
        ];
        let err = F90nmlError::circular_reference(path);
        assert_eq!(
            err.to_string(),
            "Circular reference detected: type_a -> type_b -> type_a"
        );
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let f90_err = F90nmlError::from(io_err);

        match f90_err {
            F90nmlError::Io(msg) => assert!(msg.contains("File not found")),
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(F90nmlError::parse_error("test", 1, 1).category(), "parse");
        assert_eq!(
            F90nmlError::template_error("test", None).category(),
            "template"
        );
        assert_eq!(
            F90nmlError::patch_error("test", None, None).category(),
            "patch"
        );
        assert_eq!(
            F90nmlError::validation_error("test", None, None).category(),
            "validation"
        );
    }

    #[test]
    fn test_error_recoverability() {
        assert!(!F90nmlError::UnexpectedEof.is_recoverable());
        assert!(!F90nmlError::circular_reference(vec![]).is_recoverable());
        assert!(F90nmlError::invalid_value("x", "abc", "integer").is_recoverable());
        assert!(F90nmlError::FileAlreadyExists(PathBuf::new()).is_recoverable());
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(F90nmlError::UnexpectedEof.severity(), ErrorSeverity::Fatal);
        assert_eq!(
            F90nmlError::parse_error("test", 1, 1).severity(),
            ErrorSeverity::Error
        );
        assert_eq!(
            F90nmlError::GroupNotFound {
                group: "test".to_string()
            }
            .severity(),
            ErrorSeverity::Warning
        );
    }

    #[test]
    fn test_error_context() {
        let err = F90nmlError::parse_error("test", 10, 5);
        let context = err.context();
        assert_eq!(context.line, Some(10));
        assert_eq!(context.column, Some(5));
        assert!(context.group.is_none());
        assert!(context.variable.is_none());

        let err = F90nmlError::VariableNotFound {
            variable: "x".to_string(),
            group: "data_nml".to_string(),
        };
        let context = err.context();
        assert_eq!(context.group, Some("data_nml".to_string()));
        assert_eq!(context.variable, Some("x".to_string()));
    }

    #[test]
    fn test_detailed_report() {
        let err = F90nmlError::parse_error("Unexpected token", 5, 10);
        let report = err.detailed_report();

        assert!(report.contains("Error Category: parse"));
        assert!(report.contains("Recoverable: false"));
        assert!(report.contains("Message: Parse error"));
        assert!(report.contains("Line: 5"));
        assert!(report.contains("Column: 10"));
    }

    #[test]
    fn test_error_context_helpers() {
        let context = ErrorContext::with_position(5, 10);
        assert_eq!(context.line, Some(5));
        assert_eq!(context.column, Some(10));
        assert!(!context.is_empty());

        let context =
            ErrorContext::with_location(Some("test_nml".to_string()), Some("var".to_string()));
        assert_eq!(context.group, Some("test_nml".to_string()));
        assert_eq!(context.variable, Some("var".to_string()));

        let empty_context = ErrorContext::empty();
        assert!(empty_context.is_empty());
    }
}

