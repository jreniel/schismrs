// f90nmlrs/src/scanner/lexer.rs

//! Low-level lexical analysis for Fortran namelist tokens.

use crate::error::{F90nmlError, Result};
use super::token::{Token, TokenType};

/// Low-level lexer for Fortran namelist tokens.
pub struct Lexer {
    input: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    comment_tokens: Vec<char>,
    non_delimited_strings: bool,
}

impl Lexer {
    /// Create a new lexer for the given input.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
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
    
    /// Scan the next token.
    pub fn scan_token(&mut self) -> Result<Token> {
        // Handle whitespace separately and return as token
        if let Some(c) = self.peek() {
            if c.is_whitespace() {
                return self.scan_whitespace();
            }
        }
        
        let start_line = self.line;
        let start_column = self.column;
        
        if self.is_at_end() {
            return Ok(Token::new(TokenType::Eof, String::new(), start_line, start_column));
        }
        
        let start = self.current;
        let c = self.advance();
        
        let token_type = match c {
            '&' => TokenType::GroupStart,
            '$' => TokenType::GroupStartAlt,
            '/' => TokenType::GroupEnd,
            '=' => TokenType::Assign,
            ',' => TokenType::Comma,
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            ':' => TokenType::Colon,
            '%' => TokenType::Percent,
            '+' => return self.scan_plus_or_number(start_line, start_column),
            '-' => return self.scan_minus_or_number(start_line, start_column),
            '*' => TokenType::Star,
            '\'' => return self.scan_string_single(start_line, start_column),
            '"' => return self.scan_string_double(start_line, start_column),
            '.' => return self.scan_decimal_or_logical(start_line, start_column),
            _ if c.is_ascii_alphabetic() || c == '_' => {
                return self.scan_identifier(start_line, start_column);
            }
            _ if c.is_ascii_digit() => {
                return self.scan_number(start_line, start_column);
            }
            _ if self.comment_tokens.contains(&c) => {
                return self.scan_comment(start_line, start_column);
            }
            _ => TokenType::Invalid,
        };
        
        let lexeme: String = self.input[start..self.current].iter().collect();
        Ok(Token::new(token_type, lexeme, start_line, start_column))
    }
    
    fn scan_whitespace(&mut self) -> Result<Token> {
        let start_line = self.line;
        let start_column = self.column;
        let start = self.current;
        
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
        
        let lexeme: String = self.input[start..self.current].iter().collect();
        Ok(Token::new(TokenType::Whitespace, lexeme, start_line, start_column))
    }
    
    fn scan_plus_or_number(&mut self, line: usize, column: usize) -> Result<Token> {
        let start = self.current - 1;
        
        if self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.scan_number_continuation()?;
            let lexeme: String = self.input[start..self.current].iter().collect();
            let token_type = self.determine_number_type(&lexeme);
            Ok(Token::new(token_type, lexeme, line, column))
        } else if self.peek() == Some('.') {
            self.advance(); // consume '.'
            if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.scan_number_continuation()?;
                let lexeme: String = self.input[start..self.current].iter().collect();
                Ok(Token::new(TokenType::Real, lexeme, line, column))
            } else {
                // Just a '+.' - back up the '.'
                self.current -= 1;
                let lexeme: String = self.input[start..self.current].iter().collect();
                Ok(Token::new(TokenType::Plus, lexeme, line, column))
            }
        } else if self.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
            self.scan_identifier_continuation();
            let lexeme: String = self.input[start..self.current].iter().collect();
            Ok(Token::new(TokenType::Identifier, lexeme, line, column))
        } else {
            let lexeme: String = self.input[start..self.current].iter().collect();
            Ok(Token::new(TokenType::Plus, lexeme, line, column))
        }
    }
    
    fn scan_minus_or_number(&mut self, line: usize, column: usize) -> Result<Token> {
        let start = self.current - 1;
        
        if self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.scan_number_continuation()?;
            let lexeme: String = self.input[start..self.current].iter().collect();
            let token_type = self.determine_number_type(&lexeme);
            Ok(Token::new(token_type, lexeme, line, column))
        } else if self.peek() == Some('.') {
            self.advance(); // consume '.'
            if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.scan_number_continuation()?;
                let lexeme: String = self.input[start..self.current].iter().collect();
                Ok(Token::new(TokenType::Real, lexeme, line, column))
            } else {
                // Just a '-.' - back up the '.'
                self.current -= 1;
                let lexeme: String = self.input[start..self.current].iter().collect();
                Ok(Token::new(TokenType::Minus, lexeme, line, column))
            }
        } else if self.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
            self.scan_identifier_continuation();
            let lexeme: String = self.input[start..self.current].iter().collect();
            Ok(Token::new(TokenType::Identifier, lexeme, line, column))
        } else {
            let lexeme: String = self.input[start..self.current].iter().collect();
            Ok(Token::new(TokenType::Minus, lexeme, line, column))
        }
    }
    
    fn scan_identifier(&mut self, line: usize, column: usize) -> Result<Token> {
        let start = self.current - 1;
        self.scan_identifier_continuation();
        
        let lexeme: String = self.input[start..self.current].iter().collect();
        
        let token_type = match lexeme.to_lowercase().as_str() {
            ".true." | ".t." | "true" | "t" => TokenType::Logical,
            ".false." | ".f." | "false" | "f" => TokenType::Logical,
            _ => TokenType::Identifier,
        };
        
        Ok(Token::new(token_type, lexeme, line, column))
    }
    
    fn scan_identifier_continuation(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else if self.non_delimited_strings && (c == '\'' || c == '"') {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn scan_number(&mut self, line: usize, column: usize) -> Result<Token> {
        let start = self.current - 1;
        
        // Scan integer part
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }
        
        let mut has_decimal = false;
        let mut has_exponent = false;
        let mut has_kind = false;
        
        // Check for decimal point
        if self.peek() == Some('.') {
            // Look ahead to see if it's actually a decimal or just a period
            if self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit() || matches!(c, 'e' | 'E' | 'd' | 'D')) {
                has_decimal = true;
                self.advance(); // consume '.'
                
                // Scan fractional part
                while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    self.advance();
                }
            }
        }
        
        // Check for exponent
        if let Some(c) = self.peek() {
            if c == 'e' || c == 'E' || c == 'd' || c == 'D' {
                has_exponent = true;
                self.advance(); // consume exponent marker
                
                // Optional sign
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        self.advance();
                    }
                }
                
                // Exponent digits
                if !self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    return Err(F90nmlError::invalid_syntax(
                        "Invalid exponent in number",
                        self.current,
                    ));
                }
                
                while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    self.advance();
                }
            }
        }
        
        // Check for kind specifier
        if self.peek() == Some('_') {
            has_kind = true;
            self.advance(); // consume '_'
            
            if self.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
                // Named kind
                while self.peek().map_or(false, |c| c.is_ascii_alphanumeric() || c == '_') {
                    self.advance();
                }
            } else if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                // Numeric kind
                while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    self.advance();
                }
            }
        }
        
        let lexeme: String = self.input[start..self.current].iter().collect();
        
        // Determine token type based on what we found
        let token_type = if has_decimal || has_exponent || has_kind {
            TokenType::Real
        } else {
            TokenType::Integer
        };
        
        Ok(Token::new(token_type, lexeme, line, column))
    }
    
    fn scan_number_continuation(&mut self) -> Result<()> {
        // This is called when we already consumed a +/- and need to continue scanning a number
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }
        
        // Check for decimal point
        if self.peek() == Some('.') {
            self.advance();
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        
        // Check for exponent
        if let Some(c) = self.peek() {
            if c == 'e' || c == 'E' || c == 'd' || c == 'D' {
                self.advance();
                
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        self.advance();
                    }
                }
                
                while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    self.advance();
                }
            }
        }
        
        // Check for kind specifier
        if self.peek() == Some('_') {
            self.advance(); // consume '_'
            
            if self.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
                // Named kind
                while self.peek().map_or(false, |c| c.is_ascii_alphanumeric() || c == '_') {
                    self.advance();
                }
            } else if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                // Numeric kind
                while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    self.advance();
                }
            }
        }
        
        Ok(())
    }
    
    /// Determine the number type from a lexeme
    fn determine_number_type(&self, lexeme: &str) -> TokenType {
        if lexeme.contains('.') || 
           lexeme.contains('e') || lexeme.contains('E') || 
           lexeme.contains('d') || lexeme.contains('D') ||
           lexeme.contains('_') {
            TokenType::Real
        } else {
            TokenType::Integer
        }
    }
    
    fn scan_decimal_or_logical(&mut self, line: usize, column: usize) -> Result<Token> {
        let start = self.current - 1;
        
        if self.peek().map_or(false, |c| c.is_ascii_digit()) {
            // It's a decimal number
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
            
            // Check for exponent
            if let Some(c) = self.peek() {
                if c == 'e' || c == 'E' || c == 'd' || c == 'D' {
                    self.advance();
                    
                    if let Some(sign) = self.peek() {
                        if sign == '+' || sign == '-' {
                            self.advance();
                        }
                    }
                    
                    while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                        self.advance();
                    }
                }
            }
            
            let lexeme: String = self.input[start..self.current].iter().collect();
            return Ok(Token::new(TokenType::Real, lexeme, line, column));
        }
        
        // Check if it's a logical value
        if self.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
            while self.peek().map_or(false, |c| c.is_ascii_alphanumeric() || c == '_') {
                self.advance();
            }
            
            if self.peek() == Some('.') {
                self.advance(); // consume closing '.'
            }
            
            let lexeme: String = self.input[start..self.current].iter().collect();
            let lower = lexeme.to_lowercase();
            
            if lower.starts_with(".t") || lower.starts_with(".f") {
                return Ok(Token::new(TokenType::Logical, lexeme, line, column));
            }
            
            return Ok(Token::new(TokenType::Identifier, lexeme, line, column));
        }
        
        // Just a decimal point - this is invalid in Fortran namelists
        let lexeme: String = self.input[start..self.current].iter().collect();
        Ok(Token::new(TokenType::Invalid, lexeme, line, column))
    }
    
    fn scan_string_single(&mut self, line: usize, column: usize) -> Result<Token> {
        let start = self.current - 1;
        
        while !self.is_at_end() {
            let c = self.advance();
            if c == '\n' {
                return Err(F90nmlError::invalid_syntax(
                    "Unterminated string literal",
                    self.current,
                ));
            }
            
            if c == '\'' {
                // Check for escaped quote
                if self.peek() == Some('\'') {
                    self.advance(); // consume the second quote
                } else {
                    // End of string
                    break;
                }
            }
        }
        
        let lexeme: String = self.input[start..self.current].iter().collect();
        Ok(Token::new(TokenType::String, lexeme, line, column))
    }
    
    fn scan_string_double(&mut self, line: usize, column: usize) -> Result<Token> {
        let start = self.current - 1;
        
        while !self.is_at_end() {
            let c = self.advance();
            if c == '\n' {
                return Err(F90nmlError::invalid_syntax(
                    "Unterminated string literal",
                    self.current,
                ));
            }
            
            if c == '"' {
                // Check for escaped quote
                if self.peek() == Some('"') {
                    self.advance(); // consume the second quote
                } else {
                    // End of string
                    break;
                }
            }
        }
        
        let lexeme: String = self.input[start..self.current].iter().collect();
        Ok(Token::new(TokenType::String, lexeme, line, column))
    }
    
    fn scan_comment(&mut self, line: usize, column: usize) -> Result<Token> {
        let start = self.current - 1;
        
        // Consume until end of line
        while self.peek() != Some('\n') && !self.is_at_end() {
            self.advance();
        }
        
        let lexeme: String = self.input[start..self.current].iter().collect();
        Ok(Token::new(TokenType::Comment, lexeme, line, column))
    }
    
    fn advance(&mut self) -> char {
        if !self.is_at_end() {
            let c = self.input[self.current];
            self.current += 1;
            
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            
            c
        } else {
            '\0'
        }
    }
    
    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.input[self.current])
        }
    }
    
    fn peek_ahead(&self, distance: usize) -> Option<char> {
        let pos = self.current + distance;
        if pos >= self.input.len() {
            None
        } else {
            Some(self.input[pos])
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }
}