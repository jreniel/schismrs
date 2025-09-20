// f90nmlrs/src/fortran_types/mod.rs

//! Fortran data types and their Rust representations with formatting preservation.

pub mod conversion;
pub mod formatting;
pub mod parsing;
pub mod value;

#[cfg(test)]
mod tests;

// Re-export the main types and functions
pub use formatting::{ComplexFormat, FormatOptions, QuoteStyle};
pub use parsing::{
    infer_fortran_type, parse_character, parse_complex, parse_fortran_value, parse_integer,
    parse_logical, parse_real, parse_repeat_expression, parse_value_list, validate_parsed_value,
    ValueConstraints,
};
pub use value::FortranValue;
