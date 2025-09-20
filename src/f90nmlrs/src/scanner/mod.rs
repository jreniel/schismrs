// f90nmlrs/src/scanner/mod.rs

//! Lexical scanner for Fortran namelist files with streaming support.
//!
//! This module implements tokenization for Fortran namelist input.
//! The scanner can operate in two modes:
//! 1. Normal mode: filters out whitespace for parsing
//! 2. Streaming mode: preserves all tokens including whitespace for template-based patching

pub mod token;
pub mod scanner;
pub mod lexer;

// Re-export main types and functions
pub use token::{Token, TokenType};
pub use scanner::Scanner;
pub use lexer::Lexer;

use crate::error::Result;

/// Convenience function to scan a string into tokens (filters whitespace).
pub fn scan(input: &str) -> Result<Vec<Token>> {
    let scanner = Scanner::new(input);
    scanner.scan_all()
}

/// Convenience function to scan a string preserving all tokens including whitespace.
pub fn scan_with_whitespace(input: &str) -> Result<Vec<Token>> {
    let scanner = Scanner::new(input);
    scanner.scan_all_including_whitespace()
}