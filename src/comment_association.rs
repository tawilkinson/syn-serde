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
/// The algorithm works as follows:
/// 1. If the comment is on the same line as a node and within its bounds, associate it with that node
/// 2. If the comment is immediately before a node (line before), associate it with that node
/// 3. If the comment is inside a block or immediately after a node, associate it with that node
/// 4. Otherwise, find the nearest enclosing node
fn find_best_node_for_comment(comment: &Comment, node_spans: &[(String, SpanInfo)]) -> Option<String> {
    let comment_line = comment.span.start_line;
    let comment_column = comment.span.start_column;
    
    // First pass: look for nodes on the same line
    for (node_id, node_span) in node_spans {
        if comment_line == node_span.start_line {
            // Comment is on the same line as the node start
            // Only associate if comment is after the node start (not before)
            if comment_column >= node_span.start_column {
                return Some(node_id.clone());
            }
        }
        if comment_line == node_span.end_line {
            // Comment is on the same line as the node end
            // Only associate if comment is before the node end (inside the node)
            if comment_column < node_span.end_column {
                return Some(node_id.clone());
            }
        }
    }
    
    // Second pass: look for nodes that start immediately after the comment
    for (node_id, node_span) in node_spans {
        if comment_line + 1 == node_span.start_line {
            // Comment is immediately before this node
            return Some(node_id.clone());
        }
    }
    
    // Third pass: look for nodes that contain the comment
    for (node_id, node_span) in node_spans {
        if is_comment_inside_node(comment, node_span) {
            return Some(node_id.clone());
        }
    }
    
    // Fourth pass: find the nearest enclosing node
    find_nearest_enclosing_node(comment, node_spans)
}

/// Check if a comment is inside a node's span.
fn is_comment_inside_node(comment: &Comment, node_span: &SpanInfo) -> bool {
    let comment_line = comment.span.start_line;
    let comment_column = comment.span.start_column;
    
    // Check if comment is within the node's line range
    if comment_line < node_span.start_line || comment_line > node_span.end_line {
        return false;
    }
    
    // If comment is on the same line as node start, check column
    if comment_line == node_span.start_line && comment_column < node_span.start_column {
        return false;
    }
    
    // If comment is on the same line as node end, check column
    if comment_line == node_span.end_line && comment_column > node_span.end_column {
        return false;
    }
    
    true
}

/// Find the nearest enclosing node for a comment.
fn find_nearest_enclosing_node(comment: &Comment, node_spans: &[(String, SpanInfo)]) -> Option<String> {
    let mut best_node: Option<String> = None;
    let mut best_span_size = usize::MAX;
    
    for (node_id, node_span) in node_spans {
        if is_comment_inside_node(comment, node_span) {
            let span_size = ((node_span.end_line - node_span.start_line) as usize) * 1000 + 
                           ((node_span.end_column - node_span.start_column) as usize);
            if span_size < best_span_size {
                best_span_size = span_size;
                best_node = Some(node_id.clone());
            }
        }
    }
    
    best_node
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
        assert_eq!(associations.len(), 1);
        assert!(associations.contains_key("fn_foo"));
        assert_eq!(associations["fn_foo"].len(), 1);
        assert_eq!(associations["fn_foo"][0].text, "Line 2");
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
        assert_eq!(associations.len(), 1);
        assert!(associations.contains_key("fn_foo"));
        assert_eq!(associations["fn_foo"].len(), 1);
        assert_eq!(associations["fn_foo"][0].text, "white space");
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
            ("block_body".to_string(), SpanInfo {
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
        assert!(associations.contains_key("block_body"));
        assert_eq!(associations["block_body"].len(), 1);
        assert_eq!(associations["block_body"][0].text, "Line 4, Column 10");
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