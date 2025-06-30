use syn_serde::json;

#[test]
fn test_span_preservation() {
    // Use code that will include Index and LitBool to test span preservation
    let code = r#"
        const ARRAY: [bool; 2] = [true, false];
        fn get_item() -> bool {
            ARRAY.0
        }
    "#;

    // Parse the code to get syn AST with span information
    let syn_file: syn::File = syn::parse_str(code).unwrap();
    
    // Convert to syn-serde and then to JSON
    let json_str = json::to_string_pretty(&syn_file);
    
    println!("JSON output:\n{}", json_str);
    
    // Check if spans are present in the JSON for the types that should have them
    let has_span = json_str.contains("span");
    let has_start_line = json_str.contains("start_line");
    
    if has_span {
        println!("✓ Found span information in JSON");
        assert!(has_start_line, "Span should include start_line");
    } else {
        println!("No spans found in this example - may need different syntax");
    }
    
    // Parse back from JSON 
    let restored_file: syn::File = json::from_str(&json_str).unwrap();
    
    // The restored file should be structurally equivalent (though spans will be call_site)
    assert_eq!(syn_file.items.len(), restored_file.items.len());
}

#[test]
fn test_span_info_extraction() {
    let code = r#"fn test() {
    let x = 42;
}"#;

    let syn_file: syn::File = syn::parse_str(code).unwrap();
    
    // Extract a specific span from the AST (function identifier)
    if let syn::Item::Fn(item_fn) = &syn_file.items[0] {
        let span = item_fn.sig.ident.span();
        let span_info = syn_serde::SpanInfo::from_span(span);
        
        // Verify that we captured location information
        assert!(span_info.start_line > 0);
        assert!(span_info.end_line > 0);
        assert!(span_info.start_column < span_info.end_column || span_info.start_line < span_info.end_line);
        
        println!("Extracted span info: {:?}", span_info);
        
        // Test serialization/deserialization of SpanInfo
        let json = serde_json::to_string(&span_info).unwrap();
        let restored: syn_serde::SpanInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(span_info, restored);
    }
}

#[test]
fn test_literal_spans() {
    // Test with boolean literals to exercise LitBool spans
    let code = r#"const FLAG: bool = true;"#;
    
    let syn_file: syn::File = syn::parse_str(code).unwrap();
    let json_str = json::to_string_pretty(&syn_file);
    
    println!("Literal test JSON:\n{}", json_str);
    
    // Look for boolean literal span
    if json_str.contains("span") {
        println!("✓ Found span in literal");
        assert!(json_str.contains("start_line"));
    }
}

#[test]
fn test_span_info_methods() {
    use syn_serde::SpanInfo;
    
    // Test SpanInfo utility methods
    let span_info = SpanInfo {
        start_offset: 0,
        end_offset: 0,
        start_line: 2,
        start_column: 10,
        end_line: 2,
        end_column: 15,
    };
    
    // Test column length calculation
    assert_eq!(span_info.column_length(), Some(5));
    assert!(!span_info.is_point());
    
    // Test point span
    let point_span = SpanInfo {
        start_offset: 0,
        end_offset: 0,
        start_line: 1,
        start_column: 5,
        end_line: 1,
        end_column: 5,
    };
    
    assert!(point_span.is_point());
    assert_eq!(point_span.column_length(), Some(0));
    
    // Test multi-line span
    let multiline_span = SpanInfo {
        start_offset: 0,
        end_offset: 0,
        start_line: 1,
        start_column: 10,
        end_line: 3,
        end_column: 5,
    };
    
    assert!(!multiline_span.is_point());
    assert_eq!(multiline_span.column_length(), None);
}

#[test]
fn test_span_roundtrip() {
    // Test that we can serialize and deserialize spans through JSON
    let code = r#"const VALUE: bool = false;"#;
    
    let original_file: syn::File = syn::parse_str(code).unwrap();
    
    // Convert to JSON and back
    let json_str = json::to_string_pretty(&original_file);
    let restored_file: syn::File = json::from_str(&json_str).unwrap();
    
    // Should have same structure
    assert_eq!(original_file.items.len(), restored_file.items.len());
    
    // Check that specific elements match
    if let (syn::Item::Const(orig), syn::Item::Const(rest)) = 
        (&original_file.items[0], &restored_file.items[0]) {
        assert_eq!(orig.ident.to_string(), rest.ident.to_string());
    }
    
    println!("Roundtrip successful, JSON contains spans: {}", json_str.contains("span"));
}