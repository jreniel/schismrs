// f90nmlrs/src/scanner/token.rs

//! Token types and structures for Fortran namelist lexical analysis.

use std::fmt;

/// A token in the Fortran namelist.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The type of token
    pub token_type: TokenType,
    /// The raw text of the token
    pub lexeme: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({})", self.token_type, self.lexeme)
    }
}

/// A token with associated formatting information for template preservation.
#[derive(Debug, Clone, PartialEq)]
pub struct FormattingToken {
    /// The actual token
    pub token: Token,
    /// Whitespace and comments that follow this token
    pub trailing_whitespace: String,
    /// Whether this token starts a new line
    pub starts_new_line: bool,
    /// Indentation level for this token
    pub indentation: String,
}

impl FormattingToken {
    pub fn new(token: Token) -> Self {
        Self {
            token,
            trailing_whitespace: String::new(),
            starts_new_line: false,
            indentation: String::new(),
        }
    }
    
    pub fn with_trailing_whitespace(mut self, whitespace: String) -> Self {
        self.trailing_whitespace = whitespace;
        self
    }
    
    pub fn with_indentation(mut self, indentation: String) -> Self {
        self.indentation = indentation;
        self
    }
    
    pub fn with_new_line(mut self, starts_new_line: bool) -> Self {
        self.starts_new_line = starts_new_line;
        self
    }
}

/// Types of tokens that can appear in a Fortran namelist.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// Namelist group start (&)
    GroupStart,
    /// Namelist group end (/)
    GroupEnd,
    /// Alternative group start ($)
    GroupStartAlt,
    /// Alternative group end ($)
    GroupEndAlt,
    /// Assignment operator (=)
    Assign,
    /// Comma separator (,)
    Comma,
    /// Left parenthesis (
    LeftParen,
    /// Right parenthesis )
    RightParen,
    /// Colon (:)
    Colon,
    /// Percent sign (%)
    Percent,
    /// Plus operator (+)
    Plus,
    /// Minus operator (-)
    Minus,
    /// Multiplication operator (*)
    Star,
    /// Identifier (variable names, group names)
    Identifier,
    /// Integer literal
    Integer,
    /// Real number literal
    Real,
    /// Complex number literal
    Complex,
    /// Logical literal (.true., .false.)
    Logical,
    /// String literal
    String,
    /// Comment
    Comment,
    /// Whitespace
    Whitespace,
    /// End of file
    Eof,
    /// Invalid token
    Invalid,
}