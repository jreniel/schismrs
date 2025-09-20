// f90nmlrs/src/parser/streaming.rs

//! Streaming parser that writes while parsing, following the Python f90nml approach.
//!
//! This parser processes tokens one at a time and can simultaneously write
//! to an output stream, making token-by-token decisions about whether to
//! preserve the original token or substitute a patched value.

use crate::error::{F90nmlError, Result};
use crate::fortran_types::{parse_fortran_value, FortranValue};
use crate::namelist::{Namelist, NamelistGroup};
use crate::scanner::{Scanner, Token, TokenType};
use std::io::Write;

/// A streaming parser that can parse and patch simultaneously.
pub struct StreamingParser {
    tokens: Vec<Token>,
    current: usize,
}

impl StreamingParser {
    /// Create a new streaming parser for the given input.
    pub fn new(input: &str) -> Result<Self> {
        let scanner = Scanner::new(input);
        let mut tokens = scanner.scan_all()?;

        // Remove whitespace and comment tokens for parsing, but we'll handle them separately for output
        tokens.retain(|t| !matches!(t.token_type, TokenType::Whitespace | TokenType::Comment));

        let parser = Self { tokens, current: 0 };

        Ok(parser)
    }

    /// Parse the input and return a namelist.
    pub fn parse(&mut self) -> Result<Namelist> {
        let mut namelist = Namelist::new();

        // Skip any initial tokens until we find a group start
        while !self.is_at_end() {
            if matches!(
                self.current_token_type(),
                Some(TokenType::GroupStart | TokenType::GroupStartAlt)
            ) {
                let (group_name, group) = self.parse_group()?;
                namelist.insert_group_object(&group_name, group);
            } else {
                self.advance();
            }
        }

        Ok(namelist)
    }

    /// Parse and patch simultaneously, writing output to the writer.
    pub fn parse_and_patch<W: Write>(
        &mut self,
        writer: &mut W,
        patch: &Namelist,
        original_input: &str,
    ) -> Result<Namelist> {
        // For the streaming approach, we need to re-scan with whitespace preservation
        let scanner = Scanner::new(original_input);
        let all_tokens = scanner.scan_all_including_whitespace()?;

        let mut namelist = Namelist::new();
        let mut token_idx = 0;

        while token_idx < all_tokens.len() {
            let token = &all_tokens[token_idx];

            match token.token_type {
                TokenType::GroupStart | TokenType::GroupStartAlt => {
                    let (group_name, group, new_idx) =
                        self.parse_and_patch_group(&all_tokens, token_idx, writer, patch)?;

                    namelist.insert_group_object(&group_name, group);
                    token_idx = new_idx;
                }
                TokenType::Eof => break,
                _ => {
                    // Copy any tokens outside of groups (shouldn't happen in well-formed input)
                    write!(writer, "{}", token.lexeme)?;
                    token_idx += 1;
                }
            }
        }

        // Add any groups from patch that weren't in the original
        for (patch_group_name, patch_group) in patch.groups() {
            if !namelist.has_group(patch_group_name) {
                // Write new group to output
                writeln!(writer, "")?;
                write!(writer, "&{}", patch_group_name)?;

                // Write all variables in the new group
                for (var_name, var_value) in patch_group.variables() {
                    let formatted_value = var_value.to_fortran_string(false);
                    writeln!(writer, "")?;
                    write!(writer, "    {} = {}", var_name, formatted_value)?;
                }
                writeln!(writer, "")?;
                writeln!(writer, "/")?;

                // Add to namelist
                namelist.insert_group_object(patch_group_name, patch_group.clone());
            }
        }

        Ok(namelist)
    }

    /// Parse a group with patching support.
    fn parse_and_patch_group<W: Write>(
        &self,
        tokens: &[Token],
        start_idx: usize,
        writer: &mut W,
        patch: &Namelist,
    ) -> Result<(String, NamelistGroup, usize)> {
        if start_idx >= tokens.len() {
            return Err(F90nmlError::UnexpectedEof);
        }

        // Write the group start token
        write!(writer, "{}", tokens[start_idx].lexeme)?;
        let mut idx = start_idx + 1;

        // Skip whitespace and write it
        while idx < tokens.len() && tokens[idx].token_type == TokenType::Whitespace {
            write!(writer, "{}", tokens[idx].lexeme)?;
            idx += 1;
        }

        // Get group name
        if idx >= tokens.len() || tokens[idx].token_type != TokenType::Identifier {
            return Err(F90nmlError::parse_error(
                "Expected group name after &",
                0,
                0,
            ));
        }

        let group_name = tokens[idx].lexeme.clone();
        write!(writer, "{}", group_name)?;
        idx += 1;

        let mut group = NamelistGroup::new();
        let patch_group = patch.get_group(&group_name);
        let mut patch_vars_used = std::collections::HashSet::new();

        // Parse variables in the group
        while idx < tokens.len() {
            let token = &tokens[idx];

            match token.token_type {
                TokenType::GroupEnd => {
                    // Before closing, add any new variables from patch
                    if let Some(patch_group) = patch_group {
                        for (var_name, var_value) in patch_group.variables() {
                            if !patch_vars_used.contains(var_name) {
                                let formatted_value = var_value.to_fortran_string(false);
                                writeln!(writer, "")?;
                                write!(writer, "    {} = {}", var_name, formatted_value)?;
                                group.insert(var_name, var_value.clone());
                            }
                        }
                    }

                    // Write the group end token
                    write!(writer, "{}", token.lexeme)?;
                    idx += 1;
                    break;
                }
                TokenType::Identifier => {
                    // Check if this is a variable assignment
                    let mut look_idx = idx + 1;
                    while look_idx < tokens.len()
                        && tokens[look_idx].token_type == TokenType::Whitespace
                    {
                        look_idx += 1;
                    }

                    if look_idx < tokens.len()
                        && matches!(
                            tokens[look_idx].token_type,
                            TokenType::Assign | TokenType::LeftParen
                        )
                    {
                        // Parse variable assignment with patching
                        if let Some((var_name, value, new_idx)) =
                            self.parse_and_patch_variable(tokens, idx, writer, patch_group)?
                        {
                            group.insert(&var_name, value);
                            patch_vars_used.insert(var_name);
                            idx = new_idx;
                        } else {
                            idx += 1;
                        }
                    } else {
                        // Not a variable, just copy
                        write!(writer, "{}", token.lexeme)?;
                        idx += 1;
                    }
                }
                _ => {
                    // Copy other tokens (whitespace, comments, etc.)
                    write!(writer, "{}", token.lexeme)?;
                    idx += 1;
                }
            }
        }

        Ok((group_name, group, idx))
    }

    /// Parse a variable assignment with patching support.
    fn parse_and_patch_variable<W: Write>(
        &self,
        tokens: &[Token],
        start_idx: usize,
        writer: &mut W,
        patch_group: Option<&NamelistGroup>,
    ) -> Result<Option<(String, FortranValue, usize)>> {
        if start_idx >= tokens.len() || tokens[start_idx].token_type != TokenType::Identifier {
            return Ok(None);
        }

        let var_name = tokens[start_idx].lexeme.clone();
        write!(writer, "{}", var_name)?;
        let mut idx = start_idx + 1;

        // Skip whitespace and write it
        while idx < tokens.len() && tokens[idx].token_type == TokenType::Whitespace {
            write!(writer, "{}", tokens[idx].lexeme)?;
            idx += 1;
        }

        // Handle optional array indexing
        if idx < tokens.len() && tokens[idx].token_type == TokenType::LeftParen {
            // For now, copy array indexing as-is (TODO: handle array patching)
            let mut paren_depth = 1;
            write!(writer, "{}", tokens[idx].lexeme)?;
            idx += 1;

            while idx < tokens.len() && paren_depth > 0 {
                match tokens[idx].token_type {
                    TokenType::LeftParen => paren_depth += 1,
                    TokenType::RightParen => paren_depth -= 1,
                    _ => {}
                }
                write!(writer, "{}", tokens[idx].lexeme)?;
                idx += 1;
            }
        }

        // Skip whitespace before assignment
        while idx < tokens.len() && tokens[idx].token_type == TokenType::Whitespace {
            write!(writer, "{}", tokens[idx].lexeme)?;
            idx += 1;
        }

        // Expect assignment operator
        if idx >= tokens.len() || tokens[idx].token_type != TokenType::Assign {
            return Err(F90nmlError::parse_error(
                "Expected '=' in variable assignment",
                0,
                0,
            ));
        }

        write!(writer, "{}", tokens[idx].lexeme)?; // Write '='
        idx += 1;

        // Skip whitespace after assignment
        while idx < tokens.len() && tokens[idx].token_type == TokenType::Whitespace {
            write!(writer, "{}", tokens[idx].lexeme)?;
            idx += 1;
        }

        // Check if we have a patch value for this variable
        let (value, new_idx) = if let Some(patch_group) = patch_group {
            if let Some(patch_val) = patch_group.get(&var_name) {
                // Write the patched value
                let formatted_value = patch_val.to_fortran_string(false);
                write!(writer, "{}", formatted_value)?;

                // Skip over the original value tokens
                let skip_idx = self.skip_value_tokens(tokens, idx)?;
                (patch_val.clone(), skip_idx)
            } else {
                // Parse and copy the original value
                self.parse_and_copy_value(tokens, idx, writer)?
            }
        } else {
            // Parse and copy the original value
            self.parse_and_copy_value(tokens, idx, writer)?
        };

        Ok(Some((var_name, value, new_idx)))
    }

    /// Skip over value tokens in the original input.
    fn skip_value_tokens(&self, tokens: &[Token], start_idx: usize) -> Result<usize> {
        let mut idx = start_idx;
        let mut paren_depth = 0;

        while idx < tokens.len() {
            match tokens[idx].token_type {
                TokenType::LeftParen => paren_depth += 1,
                TokenType::RightParen => paren_depth -= 1,
                TokenType::Comma if paren_depth == 0 => break,
                TokenType::GroupEnd | TokenType::Identifier if paren_depth == 0 => {
                    // Check if this identifier is followed by '=' (next variable)
                    let mut look_idx = idx + 1;
                    while look_idx < tokens.len()
                        && tokens[look_idx].token_type == TokenType::Whitespace
                    {
                        look_idx += 1;
                    }
                    if look_idx < tokens.len()
                        && matches!(
                            tokens[look_idx].token_type,
                            TokenType::Assign | TokenType::LeftParen
                        )
                    {
                        break;
                    }
                }
                _ => {}
            }
            idx += 1;
        }

        Ok(idx)
    }

    /// Parse and copy a value, returning the parsed value and new index.
    fn parse_and_copy_value<W: Write>(
        &self,
        tokens: &[Token],
        start_idx: usize,
        writer: &mut W,
    ) -> Result<(FortranValue, usize)> {
        let mut idx = start_idx;
        let mut value_tokens = Vec::new();
        let mut paren_depth = 0;

        // Collect value tokens
        while idx < tokens.len() {
            let token = &tokens[idx];

            match token.token_type {
                TokenType::LeftParen => {
                    paren_depth += 1;
                    write!(writer, "{}", token.lexeme)?;
                    value_tokens.push(token.clone());
                }
                TokenType::RightParen => {
                    paren_depth -= 1;
                    write!(writer, "{}", token.lexeme)?;
                    value_tokens.push(token.clone());
                }
                TokenType::Comma if paren_depth == 0 => {
                    write!(writer, "{}", token.lexeme)?;
                    break;
                }
                TokenType::GroupEnd | TokenType::Identifier if paren_depth == 0 => {
                    // Check if this is the start of the next variable
                    if token.token_type == TokenType::Identifier {
                        let mut look_idx = idx + 1;
                        while look_idx < tokens.len()
                            && tokens[look_idx].token_type == TokenType::Whitespace
                        {
                            look_idx += 1;
                        }
                        if look_idx < tokens.len()
                            && matches!(
                                tokens[look_idx].token_type,
                                TokenType::Assign | TokenType::LeftParen
                            )
                        {
                            break;
                        }
                    } else {
                        break;
                    }
                    write!(writer, "{}", token.lexeme)?;
                    value_tokens.push(token.clone());
                }
                _ => {
                    write!(writer, "{}", token.lexeme)?;
                    value_tokens.push(token.clone());
                }
            }
            idx += 1;
        }

        // Parse the collected tokens into a value
        let value = if value_tokens.is_empty() {
            FortranValue::Null
        } else {
            // Join lexemes and parse as a single value
            let value_str = value_tokens
                .iter()
                .filter(|t| !matches!(t.token_type, TokenType::Whitespace | TokenType::Comment))
                .map(|t| t.lexeme.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            parse_fortran_value(&value_str, None)?
        };

        Ok((value, idx))
    }

    // Helper methods from the original implementation
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.tokens[self.current].token_type == TokenType::Eof
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Option<&Token> {
        if self.current > 0 {
            Some(&self.tokens[self.current - 1])
        } else {
            None
        }
    }

    fn current_token_type(&self) -> Option<TokenType> {
        if self.is_at_end() {
            None
        } else {
            Some(self.tokens[self.current].token_type.clone())
        }
    }

    fn parse_group(&mut self) -> Result<(String, NamelistGroup)> {
        // Skip group start token
        self.advance();

        // Get group name
        let group_name = if let Some(token) = self.advance() {
            if token.token_type == TokenType::Identifier {
                token.lexeme.clone()
            } else {
                return Err(F90nmlError::parse_error(
                    "Expected group name after &",
                    token.line,
                    token.column,
                ));
            }
        } else {
            return Err(F90nmlError::UnexpectedEof);
        };

        let mut group = NamelistGroup::new();

        // Parse variables until group end
        while !self.is_at_end() {
            if let Some(current) = self.peek() {
                match current.token_type {
                    TokenType::GroupEnd => {
                        self.advance(); // consume '/'
                        break;
                    }
                    TokenType::Identifier => {
                        let (var_name, value) = self.parse_variable()?;
                        group.insert(&var_name, value);
                    }
                    _ => {
                        self.advance(); // skip unknown tokens
                    }
                }
            } else {
                break;
            }
        }

        Ok((group_name, group))
    }

    fn parse_variable(&mut self) -> Result<(String, FortranValue)> {
        let var_name = if let Some(token) = self.advance() {
            if token.token_type == TokenType::Identifier {
                token.lexeme.clone()
            } else {
                return Err(F90nmlError::parse_error(
                    "Expected variable name",
                    token.line,
                    token.column,
                ));
            }
        } else {
            return Err(F90nmlError::UnexpectedEof);
        };

        // Skip optional array indexing for now
        if let Some(current) = self.peek() {
            if current.token_type == TokenType::LeftParen {
                self.skip_array_indexing()?;
            }
        }

        // Expect assignment operator
        if let Some(token) = self.advance() {
            if token.token_type != TokenType::Assign {
                return Err(F90nmlError::parse_error(
                    "Expected '=' after variable name",
                    token.line,
                    token.column,
                ));
            }
        } else {
            return Err(F90nmlError::UnexpectedEof);
        }

        // Parse value
        let value = self.parse_value()?;

        Ok((var_name, value))
    }

    fn parse_value(&mut self) -> Result<FortranValue> {
        if let Some(token) = self.advance() {
            let value_str = token.lexeme.clone();
            parse_fortran_value(&value_str, None)
        } else {
            Err(F90nmlError::UnexpectedEof)
        }
    }

    fn skip_array_indexing(&mut self) -> Result<()> {
        let mut paren_count = 0;
        while !self.is_at_end() {
            if let Some(token) = self.advance() {
                match token.token_type {
                    TokenType::LeftParen => paren_count += 1,
                    TokenType::RightParen => {
                        paren_count -= 1;
                        if paren_count == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.current])
        }
    }

    // fn expect(&mut self, expected: TokenType) -> Result<&Token> {
    //     if let Some(current_type) = self.current_token_type() {
    //         if current_type == expected {
    //             Ok(self.advance().unwrap())
    //         } else {
    //             Err(F90nmlError::parse_error(
    //                 &format!("Expected {:?}", expected),
    //                 0,
    //                 0,
    //             ))
    //         }
    //     } else {
    //         Err(F90nmlError::UnexpectedEof)
    //     }
    // }

    // fn skip_until_token(&mut self, target: TokenType) -> Result<()> {
    //     while !self.is_at_end() {
    //         if let Some(current_type) = self.current_token_type() {
    //             if current_type == target {
    //                 return Ok(());
    //             }
    //         }
    //         self.advance();
    //     }
    //     Err(F90nmlError::UnexpectedEof)
    // }
}

