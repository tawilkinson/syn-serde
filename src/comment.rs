// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Comment extraction functionality for syn-serde.
//!
//! This module provides utilities to extract comments from Rust source code
//! and preserve their location information alongside the AST.

use crate::SpanInfo;
use serde_derive::{Deserialize, Serialize};

/// Represents a comment found in the source code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    /// The text content of the comment (without the leading // or /* */)
    pub text: String,
    /// The span information for the comment's location
    pub span: SpanInfo,
    /// Whether this is a line comment (//) or block comment (/* */)
    pub kind: CommentKind,
}

/// The kind of comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentKind {
    /// A line comment starting with //
    Line,
    /// A block comment enclosed in /* */
    Block,
}

/// Extract comments from source code.
/// 
/// This function parses the source code line by line to find comments
/// and returns them with their precise location information.
pub(crate) fn extract_comments(source: &str) -> Vec<Comment> {
    let mut comments = Vec::new();
    
    for (line_index, line) in source.lines().enumerate() {
        let line_number = line_index + 1; // 1-based line numbers
        
        // Look for line comments
        if let Some(comment_start) = line.find("//") {
            // Make sure it's not inside a string literal (basic check)
            if !is_inside_string_literal(line, comment_start) {
                let comment_text = line[comment_start + 2..].trim().to_string();
                let span = SpanInfo {
                    start_offset: 0,
                    end_offset: 0,
                    start_line: line_number,
                    start_column: comment_start,
                    end_line: line_number,
                    end_column: line.len(),
                };
                
                comments.push(Comment {
                    text: comment_text,
                    span,
                    kind: CommentKind::Line,
                });
            }
        }
        
        // Look for block comments (simplified - doesn't handle multi-line blocks)
        let mut search_start = 0;
        while let Some(block_start) = line[search_start..].find("/*") {
            let actual_start = search_start + block_start;
            
            if !is_inside_string_literal(line, actual_start) {
                if let Some(block_end) = line[actual_start..].find("*/") {
                    let actual_end = actual_start + block_end;
                    let comment_text = line[actual_start + 2..actual_end].trim().to_string();
                    let span = SpanInfo {
                        start_offset: 0,
                        end_offset: 0,
                        start_line: line_number,
                        start_column: actual_start,
                        end_line: line_number,
                        end_column: actual_end + 2,
                    };
                    
                    comments.push(Comment {
                        text: comment_text,
                        span,
                        kind: CommentKind::Block,
                    });
                    
                    search_start = actual_end + 2;
                } else {
                    // Block comment continues to next line - skip for now
                    break;
                }
            } else {
                search_start = actual_start + 1;
            }
        }
    }
    
    comments
}

/// Simple check to see if a position is inside a string literal.
/// This is a basic implementation that doesn't handle all edge cases.
fn is_inside_string_literal(line: &str, pos: usize) -> bool {
    let mut in_string = false;
    let mut escaped = false;
    let mut quote_char = None;
    
    for (i, c) in line.char_indices() {
        if i >= pos {
            break;
        }
        
        match c {
            '"' | '\'' if !escaped => {
                if let Some(expected_quote) = quote_char {
                    if c == expected_quote {
                        in_string = false;
                        quote_char = None;
                    }
                } else {
                    in_string = true;
                    quote_char = Some(c);
                }
            }
            '\\' if in_string => {
                escaped = !escaped;
                continue;
            }
            _ => {}
        }
        
        escaped = false;
    }
    
    in_string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_line_comments() {
        let source = r#"// white space
fn foo() // Line 2

         { // Line 4, Column 10





         } // Line 10, Column 10"#;
        
        let comments = extract_comments(source);
        assert_eq!(comments.len(), 4);
        
        // Check first comment
        assert_eq!(comments[0].text, "white space");
        assert_eq!(comments[0].span.start_line, 1);
        assert_eq!(comments[0].span.start_column, 0);
        assert_eq!(comments[0].kind, CommentKind::Line);
        
        // Check second comment
        assert_eq!(comments[1].text, "Line 2");
        assert_eq!(comments[1].span.start_line, 2);
        assert_eq!(comments[1].span.start_column, 9);
        assert_eq!(comments[1].kind, CommentKind::Line);
        
        // Check third comment
        assert_eq!(comments[2].text, "Line 4, Column 10");
        assert_eq!(comments[2].span.start_line, 4);
        assert_eq!(comments[2].span.start_column, 11);
        assert_eq!(comments[2].kind, CommentKind::Line);
        
        // Check fourth comment
        assert_eq!(comments[3].text, "Line 10, Column 10");
        assert_eq!(comments[3].span.start_line, 10);
        assert_eq!(comments[3].span.start_column, 11);
        assert_eq!(comments[3].kind, CommentKind::Line);
    }

    #[test]
    fn test_extract_block_comments() {
        let source = "/* block comment */ fn test() {}";
        
        let comments = extract_comments(source);
        assert_eq!(comments.len(), 1);
        
        assert_eq!(comments[0].text, "block comment");
        assert_eq!(comments[0].span.start_line, 1);
        assert_eq!(comments[0].span.start_column, 0);
        assert_eq!(comments[0].span.end_column, 19);
        assert_eq!(comments[0].kind, CommentKind::Block);
    }

    #[test]
    fn test_ignore_comments_in_strings() {
        let source = r#"let s = "// not a comment";"#;
        
        let comments = extract_comments(source);
        assert_eq!(comments.len(), 0);
    }
    
    #[test]
    fn test_comment_extraction_integration() {
        let source = r#"// white space
fn foo() // Line 2

         { // Line 4, Column 10





         } // Line 10, Column 10"#;
        
        let comments = extract_comments(source);
        assert_eq!(comments.len(), 4);
        
        // Verify that all four comments mentioned in the issue are captured
        let expected_comments = vec![
            ("white space", 1, 0),
            ("Line 2", 2, 9),
            ("Line 4, Column 10", 4, 11),
            ("Line 10, Column 10", 10, 11),
        ];
        
        for (i, (expected_text, expected_line, expected_column)) in expected_comments.iter().enumerate() {
            assert_eq!(comments[i].text, *expected_text);
            assert_eq!(comments[i].span.start_line, *expected_line);
            assert_eq!(comments[i].span.start_column, *expected_column);
            assert_eq!(comments[i].kind, CommentKind::Line);
        }
    }
}