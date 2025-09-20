// f90nmlrs/src/lib.rs

//! A Rust-native library for parsing and generating Fortran 90 namelists.
//!
//! This library provides functionality to:
//! - Parse Fortran namelist files into native Rust data structures
//! - Generate Fortran namelist files from Rust data structures
//! - Handle Fortran types including arrays, derived types, and complex numbers
//! - Support for indexing and multi-dimensional arrays
//! - Convert between different formats (JSON, YAML, namelist)
//! - Advanced streaming template-based patching that preserves formatting and comments

pub mod error;
pub mod findex;
pub mod fortran_types;
pub mod namelist;
pub mod parser;
pub mod scanner;

#[cfg(feature = "cli")]
pub mod cli;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub use error::{F90nmlError, Result};
pub use fortran_types::FortranValue;
pub use namelist::{Namelist, NamelistGroup};
pub use parser::StreamingParser;

/// Parse a Fortran namelist from a file path.
///
/// # Examples
///
/// ```no_run
/// fn main() -> Result<(), f90nmlrs::F90nmlError> {
///     let nml = f90nmlrs::read("data.nml")?;
///     println!("{:#?}", nml);
///     Ok(())
/// }
/// ```
pub fn read<P: AsRef<Path>>(path: P) -> Result<Namelist> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    reads(&contents)
}

/// Parse a Fortran namelist from a string.
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), f90nmlrs::F90nmlError> {
///     let nml_str = "&data_nml x=1 y=2.0 z=.true. /";
///     let nml = f90nmlrs::reads(nml_str)?;
///     Ok(())
/// }
/// ```
pub fn reads(content: &str) -> Result<Namelist> {
    let mut parser = StreamingParser::new(content)?;
    parser.parse()
}

/// Write a namelist to a file.
///
/// # Examples
///
/// ```no_run
/// # use f90nmlrs::{Namelist, WriteOptions};
/// # fn main() -> Result<(), f90nmlrs::F90nmlError> {
/// let mut nml = Namelist::new();
/// nml.insert_group("data_nml")
///    .insert("x", 1i32)
///    .insert("y", 2.0f64)
///    .insert("enabled", true);
///
/// f90nmlrs::write(&nml, "output.nml")?;
/// # Ok(())
/// # }
/// ```
pub fn write<P: AsRef<Path>>(nml: &Namelist, path: P) -> Result<()> {
    write_with_options(nml, path, &WriteOptions::default())
}

/// Write a namelist to a file with specific options.
pub fn write_with_options<P: AsRef<Path>>(
    nml: &Namelist,
    path: P,
    options: &WriteOptions,
) -> Result<()> {
    let path = path.as_ref();

    if !options.force && path.exists() {
        return Err(F90nmlError::FileAlreadyExists(path.to_path_buf()));
    }

    let mut file = File::create(path)?;
    write_to_writer(nml, &mut file, options)
}

/// Write a namelist to any writer implementing the Write trait.
pub fn write_to_writer<W: Write>(
    nml: &Namelist,
    writer: &mut W,
    options: &WriteOptions,
) -> Result<()> {
    let formatted = nml.to_fortran_string(options)?;
    writer.write_all(formatted.as_bytes())?;
    Ok(())
}

/// Options for controlling namelist output formatting.
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// Force overwrite existing files
    pub force: bool,
    /// Column width for output formatting
    pub column_width: usize,
    /// Indentation string (spaces or tabs)
    pub indent: String,
    /// Whether to add commas at the end of lines
    pub end_comma: bool,
    /// Whether to use uppercase for group and variable names
    pub uppercase: bool,
    /// Float formatting precision
    pub float_precision: Option<usize>,
    /// Whether to sort namelist groups alphabetically
    pub sort_groups: bool,
    /// Whether to sort variables within groups alphabetically
    pub sort_variables: bool,
    /// Starting index for arrays (default: 1 for Fortran convention)
    pub default_start_index: i32,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            force: false,
            column_width: 72,
            indent: "    ".to_string(), // 4 spaces
            end_comma: false,
            uppercase: false,
            float_precision: None,
            sort_groups: false,
            sort_variables: false,
            default_start_index: 1,
        }
    }
}

/// Create a new namelist based on an input namelist and a patch.
///
/// This allows for selective updates to namelist values while preserving
/// the original formatting and comments.
///
/// # Examples
///
/// ```
/// # use f90nmlrs::{reads, Namelist, patch};
/// # fn main() -> Result<(), f90nmlrs::F90nmlError> {
/// let original_str = "&data_nml x=1 y=2.0 /";
/// let original = reads(original_str)?;
///
/// let mut patch_nml = Namelist::new();
/// patch_nml.insert_group("data_nml")
///      .insert("x", 42i32);
///
/// let patched = patch(&original, &patch_nml)?;
/// # Ok(())
/// # }
/// ```
pub fn patch(original: &Namelist, patch: &Namelist) -> Result<Namelist> {
    let mut result = original.clone();
    result.apply_patch(patch)?;
    Ok(result)
}

/// Create a patched namelist file based on an input file and patch.
///
/// This function performs streaming template-based patching that preserves the original
/// file's formatting, comments, and structure while updating only the specified values.
///
/// # Arguments
///
/// * `input_path` - Path to the original namelist file
/// * `patch` - Namelist containing the values to update
/// * `output_path` - Path where the patched file will be written
///
/// # Examples
///
/// ```no_run
/// # use f90nmlrs::{patch_file, Namelist};
/// # fn main() -> Result<(), f90nmlrs::F90nmlError> {
/// let mut patch = Namelist::new();
/// patch.insert_group("data_nml")
///      .insert("x", 42i32)
///      .insert("new_var", "hello");
///
/// f90nmlrs::patch_file("input.nml", &patch, "output.nml")?;
/// # Ok(())
/// # }
/// ```
pub fn patch_file<P1, P2>(input_path: P1, patch: &Namelist, output_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // Read original file
    let mut input_file = File::open(input_path)?;
    let mut original_content = String::new();
    input_file.read_to_string(&mut original_content)?;

    // Create output file
    let mut output_file = File::create(output_path)?;

    patch_to_writer(&original_content, patch, &mut output_file)
}

/// Perform streaming template-based patching to a writer.
///
/// This function implements the core streaming patching logic that preserves formatting
/// and comments from the original content while updating values from the patch.
/// It follows the Python f90nml approach of parsing and writing simultaneously.
///
/// # Examples
///
/// ```
/// # use f90nmlrs::{patch_to_writer, Namelist};
/// # fn main() -> Result<(), f90nmlrs::F90nmlError> {
/// let original_content = "&data_nml x=1 y=2.0 /";
/// let mut patch = Namelist::new();
/// patch.insert_group("data_nml").insert("x", 42i32);
///
/// let mut output = Vec::new();
/// f90nmlrs::patch_to_writer(original_content, &patch, &mut output)?;
///
/// let result = String::from_utf8(output).unwrap();
/// assert!(result.contains("42"));
/// assert!(result.contains("2.0"));
/// # Ok(())
/// # }
/// ```
pub fn patch_to_writer<W: Write>(
    original_content: &str,
    patch: &Namelist,
    writer: &mut W,
) -> Result<()> {
    let mut parser = StreamingParser::new(original_content)?;
    parser.parse_and_patch(writer, patch, original_content)?;
    Ok(())
}

/// Enhanced patch function that works with file paths and preserves formatting.
///
/// This is equivalent to the Python f90nml.patch() function but uses streaming processing.
/// If no output path is provided, the function still parses and applies the patch,
/// returning the merged namelist.
///
/// # Examples
///
/// ```no_run
/// # use f90nmlrs::{patch_with_template, Namelist};
/// # fn main() -> Result<(), f90nmlrs::F90nmlError> {
/// let mut patch = Namelist::new();
/// patch.insert_group("data_nml").insert("x", 42i32);
///
/// // Just merge without writing to file
/// let result = f90nmlrs::patch_with_template("input.nml", &patch, None::<&str>)?;
///
/// // Or write to output file
/// let result = f90nmlrs::patch_with_template("input.nml", &patch, Some("output.nml"))?;
/// # Ok(())
/// # }
/// ```
pub fn patch_with_template<P1, P2>(
    input_path: P1,
    patch: &Namelist,
    output_path: Option<P2>,
) -> Result<Namelist>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let input_path = input_path.as_ref();

    // Read original file
    let mut input_file = File::open(input_path)?;
    let mut original_content = String::new();
    input_file.read_to_string(&mut original_content)?;

    // Parse and apply patch using streaming parser
    let mut parser = StreamingParser::new(&original_content)?;

    let result_namelist = if let Some(output_path) = output_path {
        let output_path = output_path.as_ref();
        let mut output_file = File::create(output_path)?;

        // Parse and patch with output to file
        parser.parse_and_patch(&mut output_file, patch, &original_content)?
    } else {
        // Just parse and merge without output file
        let original_namelist = parser.parse()?;
        let mut result = original_namelist;
        result.apply_patch(patch)?;
        result
    };

    Ok(result_namelist)
}

#[cfg(feature = "json")]
/// Convert a namelist to JSON string.
pub fn to_json(nml: &Namelist) -> Result<String> {
    serde_json::to_string_pretty(nml).map_err(F90nmlError::from)
}

#[cfg(feature = "json")]
/// Parse a namelist from JSON string.
pub fn from_json(json: &str) -> Result<Namelist> {
    serde_json::from_str(json).map_err(F90nmlError::from)
}

#[cfg(feature = "yaml")]
/// Convert a namelist to YAML string.
pub fn to_yaml(nml: &Namelist) -> Result<String> {
    serde_yaml::to_string(nml).map_err(F90nmlError::from)
}

#[cfg(feature = "yaml")]
/// Parse a namelist from YAML string.
pub fn from_yaml(yaml: &str) -> Result<Namelist> {
    serde_yaml::from_str(yaml).map_err(F90nmlError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reads_simple() {
        let nml_str = "&data_nml x=1 y=2.0 z=.true. /";
        let nml = reads(nml_str).unwrap();

        let group = nml.get_group("data_nml").unwrap();
        assert_eq!(group.get_i32("x"), Some(1));
        assert_eq!(group.get_f64("y"), Some(2.0));
        assert_eq!(group.get_bool("z"), Some(true));
    }

    #[test]
    fn test_write_simple() {
        let mut nml = Namelist::new();
        nml.insert_group("data_nml")
            .insert("x", 1i32)
            .insert("y", 2.0f64)
            .insert("enabled", true);

        let output = nml.to_fortran_string(&WriteOptions::default()).unwrap();
        assert!(output.contains("&data_nml"));
        assert!(output.contains("x = 1"));
        assert!(output.contains("y = 2"));
        assert!(output.contains("enabled = .true."));
        assert!(output.contains("/"));
    }

    #[test]
    fn test_patch() {
        let original_str = "&data_nml x=1 y=2.0 /";
        let original = reads(original_str).unwrap();

        let mut patch_nml = Namelist::new();
        patch_nml.insert_group("data_nml").insert("x", 42i32);

        let patched = patch(&original, &patch_nml).unwrap();
        let group = patched.get_group("data_nml").unwrap();

        assert_eq!(group.get_i32("x"), Some(42)); // Updated
        assert_eq!(group.get_f64("y"), Some(2.0)); // Preserved
    }

    #[test]
    fn test_streaming_patch_to_writer() {
        let original_content = "&data_nml x=1 y=2.0 /";

        let mut patch = Namelist::new();
        patch.insert_group("data_nml").insert("x", 42i32);

        let mut output = Vec::new();
        patch_to_writer(original_content, &patch, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();

        // Should update x value
        assert!(result.contains("x = 42") || result.contains("x=42"));
        // Should preserve y value
        assert!(result.contains("y = 2.0") || result.contains("y=2.0"));
    }

    #[test]
    fn test_streaming_patch_with_comments() {
        let original_content = r#"&data_nml  ! group comment
    x = 1,  ! inline comment
    y = 2.0
/"#;

        let mut patch = Namelist::new();
        patch
            .insert_group("data_nml")
            .insert("x", 42i32)
            .insert("new_var", "hello");

        let mut output = Vec::new();
        patch_to_writer(original_content, &patch, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();

        // Should preserve comments
        assert!(result.contains("! group comment"));
        assert!(result.contains("! inline comment"));

        // Should update values
        assert!(result.contains("42")); // x updated
        assert!(result.contains("2.0")); // y preserved

        println!("{}", &result);

        // Should add new variables
        assert!(result.contains("new_var"));
        assert!(result.contains("hello"));
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_json_roundtrip() {
        let nml_str = "&data_nml x=1 y=2.0 z=.true. /";
        let nml = reads(nml_str).unwrap();

        let json = to_json(&nml).unwrap();
        let nml_from_json = from_json(&json).unwrap();

        assert_eq!(nml, nml_from_json);
    }
}
