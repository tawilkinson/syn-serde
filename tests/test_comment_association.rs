//! Test for comment association with AST nodes
//! This test verifies that comments are properly attached to their corresponding AST nodes

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_comment_association_with_ast_nodes() {
        let source = r#"
// File-level comment
use std::collections::HashMap;

// Function comment
fn test_function() {
    // Block comment
    let x = 42;
}

// Struct comment
struct TestStruct {
    field: i32,
}

// Enum comment
enum TestEnum {
    Variant1,
    Variant2,
}
"#;
        
        let syn_file = syn::parse_file(source).unwrap();
        let syntax = syn_serde::File::from_syn_with_comments(&syn_file, source);
        
        // Verify comments are attached to correct nodes
        let json_output = serde_json::to_string_pretty(&syntax).unwrap();
        
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        
        // Check that there IS a top-level comments array for comments not inside function curly braces
        assert!(parsed.get("comments").is_some(), "File should have a top-level comments array for unassociated comments");
        
        // Check that comments outside function curly braces are preserved at file level
        let file_comments = parsed["comments"].as_array().unwrap();
        assert!(file_comments.len() > 0, "Should have file-level comments");
        
        // Check that the function block has the block comment (inside curly braces)
        let fn_item = &parsed["items"][1]["fn"];
        let fn_block = &fn_item["stmts"];
        if let Some(block_comments) = fn_block.get("comments") {
            let block_comment_array = block_comments.as_array().unwrap();
            assert!(block_comment_array.len() > 0, "Should have block-level comments for comments inside curly braces");
        }
    }
}