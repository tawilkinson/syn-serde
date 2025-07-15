// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Comment association functionality for syn-serde.
//!
//! This module provides utilities to associate comments with AST nodes
//! based on their position in the source code.

use crate::{Comment, SpanInfo};
use std::collections::HashMap;

/// Associates comments with AST nodes based on their position.
/// 
/// This function takes a list of comments and a list of AST node spans
/// and returns a mapping of AST node identifiers to their associated comments.
pub(crate) fn associate_comments_with_nodes(
    comments: &[Comment],
    node_spans: &[(String, SpanInfo)],
) -> HashMap<String, Vec<Comment>> {
    let mut associations: HashMap<String, Vec<Comment>> = HashMap::new();
    
    for comment in comments {
        let best_node = find_best_node_for_comment(comment, node_spans);
        if let Some(node_id) = best_node {
            associations.entry(node_id).or_default().push(comment.clone());
        }
    }
    
    associations
}

/// Find the best AST node to associate a comment with.
/// 
/// The algorithm is conservative and only associates comments that are truly inside a node's span.
/// Comments are only associated with function blocks if they are wholly within the curly braces.
fn find_best_node_for_comment(comment: &Comment, node_spans: &[(String, SpanInfo)]) -> Option<String> {
    // Only look for block nodes (not function declaration nodes)
    // This ensures comments are only nested within function blocks, not function declarations
    for (node_id, node_span) in node_spans {
        // Only consider block nodes for comment association
        if node_id.ends_with("_block") && is_comment_strictly_inside_node(comment, node_span) {
            return Some(node_id.clone());
        }
    }
    
    // If no block node strictly contains the comment, don't associate it
    None
}

/// Check if a comment is strictly inside a node's span.
/// This is much more conservative than the original logic.
/// A comment is considered "inside" if:
/// 1. It's on a line strictly between start and end lines, OR
/// 2. It's on the start line but after the start column (for cases like "{ // comment"), OR  
/// 3. It's on the end line but before the end column (for cases like "// comment }")
fn is_comment_strictly_inside_node(comment: &Comment, node_span: &SpanInfo) -> bool {
    let comment_line = comment.span.start_line;
    let comment_column = comment.span.start_column;
    
    // Case 1: Comment is strictly between start and end lines
    if comment_line > node_span.start_line && comment_line < node_span.end_line {
        return true;
    }
    
    // Case 2: Comment is on the start line but after the start column (e.g., "{ // comment")
    if comment_line == node_span.start_line && comment_column > node_span.start_column {
        return true;
    }
    
    // Case 3: Comment is on the end line but before the end column (e.g., "// comment }")
    if comment_line == node_span.end_line && comment_column < node_span.end_column {
        return true;
    }
    
    false
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommentKind, SpanInfo};

    #[test]
    fn test_comment_on_same_line_as_node() {
        let comment = Comment {
            text: "Line 2".to_string(),
            span: SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 2,
                start_column: 9,
                end_line: 2,
                end_column: 18,
            },
            kind: CommentKind::Line,
        };
        
        let node_spans = vec![
            ("fn_foo".to_string(), SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 2,
                start_column: 3,
                end_line: 2,
                end_column: 6,
            }),
        ];
        
        let associations = associate_comments_with_nodes(&[comment], &node_spans);
        // Comment should NOT be associated with function declaration
        assert_eq!(associations.len(), 0);
    }
    
    #[test]
    fn test_comment_before_node() {
        let comment = Comment {
            text: "white space".to_string(),
            span: SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 14,
            },
            kind: CommentKind::Line,
        };
        
        let node_spans = vec![
            ("fn_foo".to_string(), SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 2,
                start_column: 3,
                end_line: 2,
                end_column: 6,
            }),
        ];
        
        let associations = associate_comments_with_nodes(&[comment], &node_spans);
        // Comment should NOT be associated with function declaration
        assert_eq!(associations.len(), 0);
    }
    
    #[test]
    fn test_comment_inside_block() {
        let comment = Comment {
            text: "Line 4, Column 10".to_string(),
            span: SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 4,
                start_column: 11,
                end_line: 4,
                end_column: 31,
            },
            kind: CommentKind::Line,
        };
        
        let node_spans = vec![
            ("item_0_block".to_string(), SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 4,
                start_column: 9,
                end_line: 10,
                end_column: 10,
            }),
        ];
        
        let associations = associate_comments_with_nodes(&[comment], &node_spans);
        assert_eq!(associations.len(), 1);
        assert!(associations.contains_key("item_0_block"));
        assert_eq!(associations["item_0_block"].len(), 1);
        assert_eq!(associations["item_0_block"][0].text, "Line 4, Column 10");
    }
    
    #[test]
    fn test_comment_outside_node_scope_not_associated() {
        let comment = Comment {
            text: "Line 10, Column 10 - after function".to_string(),
            span: SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 10,
                start_column: 11, // Comment starts after the closing brace
                end_line: 10,
                end_column: 50,
            },
            kind: CommentKind::Line,
        };
        
        let node_spans = vec![
            ("fn_foo".to_string(), SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 2,
                start_column: 3,
                end_line: 2,
                end_column: 6,
            }),
            ("block_body".to_string(), SpanInfo {
                start_offset: 0,
                end_offset: 0,
                start_line: 4,
                start_column: 9,
                end_line: 10,
                end_column: 10, // Block ends at column 10
            }),
        ];
        
        let associations = associate_comments_with_nodes(&[comment], &node_spans);
        // Comment should not be associated with any node since it's outside their scope
        assert_eq!(associations.len(), 0);
    }
}