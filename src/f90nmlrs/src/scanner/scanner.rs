// f90nmlrs/src/scanner/scanner.rs

//! Core scanner implementation for Fortran namelist files with streaming support.

use crate::error::Result;
use super::token::{Token, TokenType};
use super::lexer::Lexer;

/// Lexical scanner for Fortran namelist files.
pub struct Scanner {
    input: String,
    comment_tokens: Vec<char>,
    non_delimited_strings: bool,
}

impl Scanner {
    /// Create a new scanner for the given input.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            comment_tokens: vec!['!', '#'],
            non_delimited_strings: true,
        }
    }
    
    /// Set comment tokens (default: ['!', '#']).
    pub fn with_comment_tokens(mut self, tokens: Vec<char>) -> Self {
        self.comment_tokens = tokens;
        self
    }
    
    /// Enable or disable non-delimited strings.
    pub fn with_non_delimited_strings(mut self, enabled: bool) -> Self {
        self.non_delimited_strings = enabled;
        self
    }
    
    /// Scan all tokens from the input, filtering out whitespace.
    pub fn scan_all(&self) -> Result<Vec<Token>> {
        let mut lexer = Lexer::new(&self.input)
            .with_comment_tokens(self.comment_tokens.clone())
            .with_non_delimited_strings(self.non_delimited_strings);
        
        let mut tokens = Vec::new();
        
        loop {
            let token = lexer.scan_token()?;
            let is_eof = token.token_type == TokenType::Eof;
            
            // Filter out whitespace tokens for normal parsing
            if !matches!(token.token_type, TokenType::Whitespace) {
                tokens.push(token);
            }
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
    
    /// Scan all tokens including whitespace and comments.
    /// This is used for streaming/template-based patching where we need to preserve everything.
    pub fn scan_all_including_whitespace(&self) -> Result<Vec<Token>> {
        let mut lexer = Lexer::new(&self.input)
            .with_comment_tokens(self.comment_tokens.clone())
            .with_non_delimited_strings(self.non_delimited_strings);
        
        let mut tokens = Vec::new();
        
        loop {
            let token = lexer.scan_token()?;
            let is_eof = token.token_type == TokenType::Eof;
            
            // Include ALL tokens, including whitespace
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
    
    /// Scan the next token (for streaming use).
    pub fn scan_token(&self) -> Result<Token> {
        let mut lexer = Lexer::new(&self.input)
            .with_comment_tokens(self.comment_tokens.clone())
            .with_non_delimited_strings(self.non_delimited_strings);
        
        lexer.scan_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_simple_namelist() {
        let input = "&data_nml x=1 y=2.0 z=.true. /";
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_all().unwrap();
        
        let expected_types = vec![
            TokenType::GroupStart,     // &
            TokenType::Identifier,     // data_nml
            TokenType::Identifier,     // x
            TokenType::Assign,         // =
            TokenType::Integer,        // 1
            TokenType::Identifier,     // y
            TokenType::Assign,         // =
            TokenType::Real,           // 2.0
            TokenType::Identifier,     // z
            TokenType::Assign,         // =
            TokenType::Logical,        // .true.
            TokenType::GroupEnd,       // /
            TokenType::Eof,
        ];
        
        assert_eq!(tokens.len(), expected_types.len());
        for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
            assert_eq!(token.token_type, *expected_type);
        }
    }
    
    #[test]
    fn test_scan_with_whitespace_preservation() {
        let input = "&data_nml  ! comment\n    x = 1\n/";
        
        let scanner = Scanner::new(input);
        let tokens_filtered = scanner.scan_all().unwrap();
        let tokens_all = scanner.scan_all_including_whitespace().unwrap();
        
        // Filtered version should have fewer tokens
        assert!(tokens_all.len() > tokens_filtered.len());
        
        // All version should include whitespace and comments
        let has_whitespace = tokens_all.iter()
            .any(|t| t.token_type == TokenType::Whitespace);
        let has_comment = tokens_all.iter()
            .any(|t| t.token_type == TokenType::Comment);
        
        assert!(has_whitespace, "Should preserve whitespace tokens");
        assert!(has_comment, "Should preserve comment tokens");
        
        // Filtered version should not have whitespace
        let filtered_has_whitespace = tokens_filtered.iter()
            .any(|t| t.token_type == TokenType::Whitespace);
        
        assert!(!filtered_has_whitespace, "Filtered tokens should not contain whitespace");
    }
    
    #[test]
    fn test_scan_numbers() {
        let input = "42 3.14 1.23e4 1.23d-5 2_real64";
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_all().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Integer);
        assert_eq!(tokens[0].lexeme, "42");
        
        assert_eq!(tokens[1].token_type, TokenType::Real);
        assert_eq!(tokens[1].lexeme, "3.14");
        
        assert_eq!(tokens[2].token_type, TokenType::Real);
        assert_eq!(tokens[2].lexeme, "1.23e4");
        
        assert_eq!(tokens[3].token_type, TokenType::Real);
        assert_eq!(tokens[3].lexeme, "1.23d-5");
        
        assert_eq!(tokens[4].token_type, TokenType::Real);
        assert_eq!(tokens[4].lexeme, "2_real64");
    }
    
    #[test]
    fn test_scan_strings() {
        let input = r#"'hello' "world" 'don''t'"#;
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_all().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].lexeme, "'hello'");
        
        assert_eq!(tokens[1].token_type, TokenType::String);
        assert_eq!(tokens[1].lexeme, "\"world\"");
        
        assert_eq!(tokens[2].token_type, TokenType::String);
        assert_eq!(tokens[2].lexeme, "'don''t'");
    }
    
    #[test]
    fn test_scan_logicals() {
        let input = ".true. .false. .T. .F. true false";
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_all().unwrap();
        
        for i in 0..6 {
            assert_eq!(tokens[i].token_type, TokenType::Logical);
        }
    }
    
    #[test]
    fn test_scan_arrays() {
        let input = "arr(1:10) = 1, 2, 3";
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_all().unwrap();
        
        let expected_types = vec![
            TokenType::Identifier,     // arr
            TokenType::LeftParen,      // (
            TokenType::Integer,        // 1
            TokenType::Colon,          // :
            TokenType::Integer,        // 10
            TokenType::RightParen,     // )
            TokenType::Assign,         // =
            TokenType::Integer,        // 1
            TokenType::Comma,          // ,
            TokenType::Integer,        // 2
            TokenType::Comma,          // ,
            TokenType::Integer,        // 3
            TokenType::Eof,
        ];
        
        for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
            assert_eq!(token.token_type, *expected_type);
        }
    }
    
    #[test]
    fn test_scan_comments() {
        let input = "x=1 ! This is a comment\ny=2";
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_all_including_whitespace().unwrap();
        
        // Should include comment token when preserving everything
        let comment_token = tokens.iter()
            .find(|t| t.token_type == TokenType::Comment)
            .expect("Should find comment token");
        assert_eq!(comment_token.lexeme, "! This is a comment");
    }
    
    #[test]
    fn test_line_column_tracking() {
        let input = "x=1\ny=2";
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_all().unwrap();
        
        assert_eq!(tokens[0].line, 1); // x
        assert_eq!(tokens[0].column, 1);
        
        // Find the 'y' token
        let y_token = tokens.iter()
            .find(|t| t.token_type == TokenType::Identifier && t.lexeme == "y")
            .expect("Should find y token");
        assert_eq!(y_token.line, 2);
        assert_eq!(y_token.column, 1);
    }
}