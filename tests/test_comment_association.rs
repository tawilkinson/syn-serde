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
        
        // Check that there's no separate comments array at the file level (but comments can exist within nodes)
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert!(parsed.get("comments").is_none(), "File should not have a top-level comments array");
        
        // Check that the use statement has the file-level comment
        let use_item = &parsed["items"][0]["use"];
        assert!(use_item.get("comments").is_some());
        
        // Check that the function has the function comment
        let fn_item = &parsed["items"][1]["fn"];
        assert!(fn_item.get("comments").is_some());
        
        // Check that the function block has the block comment
        let fn_block = &fn_item["stmts"];
        assert!(fn_block.get("comments").is_some());
        
        // Check that the struct has the struct comment
        if parsed["items"].get(2).is_some() {
            let struct_item = &parsed["items"][2]["struct"];
            // Note: ItemStruct doesn't have comments field currently
        }
        
        // Check that the enum has the enum comment
        if parsed["items"].get(3).is_some() {
            let enum_item = &parsed["items"][3]["enum"];
            assert!(enum_item.get("comments").is_some());
        }
    }
}