// f90nmlrs/src/scanner/formatting.rs

//! Formatting preservation logic for template-based patching.

use crate::error::Result;
use super::token::{Token, TokenType, FormattingToken};
use super::lexer::Lexer;

/// Handles formatting preservation during lexical analysis.
pub struct FormattingPreserver {
    lexer: Lexer,
    comment_tokens: Vec<char>,
}

impl FormattingPreserver {
    /// Create a new formatting preserver.
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
            comment_tokens: vec!['!', '#'],
        }
    }
    
    /// Set comment tokens.
    pub fn with_comment_tokens(mut self, tokens: Vec<char>) -> Self {
        self.comment_tokens = tokens.clone();
        self.lexer = self.lexer.with_comment_tokens(tokens);
        self
    }
    
    /// Scan all tokens with formatting information preserved.
    pub fn scan_all_with_formatting(&mut self) -> Result<Vec<FormattingToken>> {
        let mut tokens = Vec::new();
        let mut current_indentation = String::new();
        let mut line_start = true;
        
        loop {
            let token = self.lexer.scan_token()?;
            let is_eof = token.token_type == TokenType::Eof;
            
            // Track indentation at line start
            if line_start && token.token_type == TokenType::Whitespace {
                // Extract indentation from whitespace token
                let whitespace_content = &token.lexeme;
                if whitespace_content.contains('\n') {
                    // Find the last newline and get everything after it as indentation
                    if let Some(last_newline_pos) = whitespace_content.rfind('\n') {
                        current_indentation = whitespace_content[last_newline_pos + 1..].to_string();
                    }
                } else {
                    // Accumulate whitespace at the start of line
                    current_indentation.push_str(whitespace_content);
                }
            } else if token.token_type != TokenType::Whitespace && token.token_type != TokenType::Comment {
                line_start = false;
            }
            
            // Create formatting token
            let mut formatting_token = FormattingToken::new(token);
            
            // Set indentation for non-whitespace tokens at line start
            if !line_start && !current_indentation.is_empty() && 
               !matches!(formatting_token.token.token_type, TokenType::Whitespace | TokenType::Comment) {
                formatting_token = formatting_token.with_indentation(current_indentation.clone());
            }
            
            // Collect trailing whitespace and comments
            let mut trailing_whitespace = String::new();
            let mut found_newline = false;
            
            // Use a separate method to collect trailing content
            self.collect_trailing_content(&mut trailing_whitespace, &mut found_newline, &mut line_start, &mut current_indentation)?;
            
            formatting_token = formatting_token.with_trailing_whitespace(trailing_whitespace);
            
            if found_newline {
                formatting_token = formatting_token.with_new_line(true);
            }
            
            tokens.push(formatting_token);
            
            if is_eof {
                break;
            }
        }
        
        // Post-process to properly assign indentation
        self.post_process_indentation(&mut tokens);
        
        Ok(tokens)
    }
    
    /// Collect trailing whitespace and comments after a token.
    fn collect_trailing_content(
        &mut self,
        trailing_whitespace: &mut String,
        found_newline: &mut bool,
        line_start: &mut bool,
        current_indentation: &mut String,
    ) -> Result<()> {
        loop {
            // Peek at the next token to see if it's whitespace or comment
            let next_token = self.lexer.scan_token()?;
            
            match next_token.token_type {
                TokenType::Whitespace => {
                    trailing_whitespace.push_str(&next_token.lexeme);
                    if next_token.lexeme.contains('\n') {
                        *found_newline = true;
                        *line_start = true;
                        current_indentation.clear();
                    }
                }
                TokenType::Comment => {
                    trailing_whitespace.push_str(&next_token.lexeme);
                    // Comments typically end at line boundary, so check for following newline
                    let ws_token = self.lexer.scan_token()?;
                    if ws_token.token_type == TokenType::Whitespace && ws_token.lexeme.contains('\n') {
                        trailing_whitespace.push_str(&ws_token.lexeme);
                        *found_newline = true;
                        *line_start = true;
                        current_indentation.clear();
                    } else {
                        // Put the token back by rewinding (this is tricky with the current design)
                        // For now, we'll handle this case differently
                        break;
                    }
                }
                TokenType::Eof => {
                    break;
                }
                _ => {
                    // Put the token back and break
                    // This is where the borrowing issue comes from - we need to restructure
                    break;
                }
            }
        }
        Ok(())
    }
    
    /// Post-process tokens to properly assign indentation.
    fn post_process_indentation(&self, tokens: &mut [FormattingToken]) {
        let mut line_groups = Vec::new();
        let mut current_line_tokens = Vec::new();
        let mut current_line_indent = String::new();
        
        for (i, token) in tokens.iter().enumerate() {
            if token.starts_new_line || i == 0 {
                // Start of a new line - save previous line
                if !current_line_tokens.is_empty() {
                    line_groups.push((current_line_tokens.clone(), current_line_indent.clone()));
                }
                
                current_line_tokens.clear();
                current_line_indent.clear();
                
                // Extract indentation from the beginning of this line
                if token.token.token_type == TokenType::Whitespace {
                    current_line_indent = token.token.lexeme.clone();
                }
            }
            
            current_line_tokens.push(i);
        }
        
        // Save the last line
        if !current_line_tokens.is_empty() {
            line_groups.push((current_line_tokens, current_line_indent));
        }
        
        // Now apply indentation to each line group
        for (line_token_indices, line_indent) in line_groups {
            self.assign_indentation_to_line(tokens, &line_token_indices, &line_indent);
        }
    }
    
    /// Assign indentation to tokens on a line.
    fn assign_indentation_to_line(
        &self, 
        tokens: &mut [FormattingToken], 
        line_token_indices: &[usize], 
        line_indent: &str
    ) {
        for &token_idx in line_token_indices {
            if !matches!(tokens[token_idx].token.token_type, 
                TokenType::Whitespace | TokenType::Comment) {
                tokens[token_idx].indentation = line_indent.to_string();
                break; // Only assign to the first non-whitespace token
            }
        }
    }
}

/// Alternative implementation that avoids borrowing issues by buffering tokens.
pub struct BufferedFormattingPreserver {
    input: String,
    comment_tokens: Vec<char>,
}

impl BufferedFormattingPreserver {
    /// Create a new buffered formatting preserver.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            comment_tokens: vec!['!', '#'],
        }
    }
    
    /// Set comment tokens.
    pub fn with_comment_tokens(mut self, tokens: Vec<char>) -> Self {
        self.comment_tokens = tokens;
        self
    }
    
    /// Scan all tokens with formatting information preserved.
    pub fn scan_all_with_formatting(&self) -> Result<Vec<FormattingToken>> {
        // First pass: collect all raw tokens
        let mut lexer = Lexer::new(&self.input).with_comment_tokens(self.comment_tokens.clone());
        let mut raw_tokens = Vec::new();
        
        loop {
            let token = lexer.scan_token()?;
            let is_eof = token.token_type == TokenType::Eof;
            raw_tokens.push(token);
            if is_eof {
                break;
            }
        }
        
        // Second pass: group tokens and assign trailing whitespace/comments
        let mut formatting_tokens = Vec::new();
        let mut i = 0;
        
        while i < raw_tokens.len() {
            let token = &raw_tokens[i];
            let mut formatting_token = FormattingToken::new(token.clone());
            
            // Collect trailing whitespace and comments
            let mut trailing_whitespace = String::new();
            let mut found_newline = false;
            let mut j = i + 1;
            
            // Collect consecutive whitespace and comments as trailing content
            while j < raw_tokens.len() {
                match raw_tokens[j].token_type {
                    TokenType::Whitespace | TokenType::Comment => {
                        trailing_whitespace.push_str(&raw_tokens[j].lexeme);
                        if raw_tokens[j].lexeme.contains('\n') {
                            found_newline = true;
                        }
                        j += 1;
                    }
                    _ => break,
                }
            }
            
            formatting_token = formatting_token
                .with_trailing_whitespace(trailing_whitespace)
                .with_new_line(found_newline);
            
            formatting_tokens.push(formatting_token);
            
            // Skip the tokens we consumed as trailing content
            i = j.max(i + 1);
        }
        
        // Third pass: assign indentation by analyzing whitespace patterns
        self.assign_indentation_from_patterns(&mut formatting_tokens);
        
        Ok(formatting_tokens)
    }
    
    /// Assign indentation by analyzing whitespace patterns in trailing content.
    fn assign_indentation_from_patterns(&self, tokens: &mut [FormattingToken]) {
        for i in 0..tokens.len() {
            // Look for tokens that have trailing whitespace containing newlines
            if tokens[i].trailing_whitespace.contains('\n') {
                // Extract indentation from the trailing whitespace
                let trailing = &tokens[i].trailing_whitespace;
                if let Some(last_newline) = trailing.rfind('\n') {
                    let indentation = trailing[last_newline + 1..].to_string();
                    
                    // Find the next meaningful token and assign indentation
                    for j in (i + 1)..tokens.len() {
                        if !matches!(tokens[j].token.token_type, TokenType::Whitespace | TokenType::Comment) {
                            tokens[j].indentation = indentation.clone();
                            break;
                        }
                    }
                }
            }
        }
        
        // Handle the first line separately (before any newlines)
        if !tokens.is_empty() {
            // Check if the first token needs indentation
            let mut first_meaningful_idx = None;
            for (idx, token) in tokens.iter().enumerate() {
                if !matches!(token.token.token_type, TokenType::Whitespace | TokenType::Comment) {
                    first_meaningful_idx = Some(idx);
                    break;
                }
            }
            
            // If the first token is whitespace, use it as indentation for the first meaningful token
            if tokens[0].token.token_type == TokenType::Whitespace && !tokens[0].token.lexeme.contains('\n') {
                if let Some(idx) = first_meaningful_idx {
                    if tokens[idx].indentation.is_empty() {
                        tokens[idx].indentation = tokens[0].token.lexeme.clone();
                    }
                }
            }
        }
    }
}