// SPDX-License-Identifier: Apache-2.0 OR MIT

use syn_serde::json;

#[test]
fn test_compact_json_excludes_spans() {
    let syn_file: syn::File = syn::parse_quote! {
        fn hello() {
            println!("world");
        }
    };

    // Regular JSON should contain spans
    let regular_json = json::to_string_pretty(&syn_file);
    assert!(regular_json.contains("span"));
    assert!(regular_json.contains("start_line"));
    assert!(regular_json.contains("end_line"));

    // Compact JSON should not contain spans
    let compact_json = json::to_string_compact_pretty(&syn_file);
    assert!(!compact_json.contains("span"));
    assert!(!compact_json.contains("start_line"));
    assert!(!compact_json.contains("end_line"));

    // Should still contain the actual content
    assert!(compact_json.contains("hello"));
    assert!(compact_json.contains("println"));
}

#[test]
fn test_compact_json_preserves_structure() {
    let syn_file: syn::File = syn::parse_quote! {
        struct Point {
            x: i32,
            y: i32,
        }
    };

    let compact_json = json::to_string_compact(&syn_file);
    
    // Should contain the struct information
    assert!(compact_json.contains("Point"));
    assert!(compact_json.contains("struct"));
    
    // Should not contain span information
    assert!(!compact_json.contains("span"));
}

#[test]
fn test_compact_json_functions_consistency() {
    let syn_file: syn::File = syn::parse_quote! {
        fn test() {}
    };

    // Test that all compact functions produce span-free output
    let string_compact = json::to_string_compact(&syn_file);
    let string_compact_pretty = json::to_string_compact_pretty(&syn_file);
    let vec_compact = json::to_vec_compact(&syn_file);
    let vec_compact_pretty = json::to_vec_compact_pretty(&syn_file);

    // None should contain spans
    assert!(!string_compact.contains("span"));
    assert!(!string_compact_pretty.contains("span"));
    assert!(!String::from_utf8(vec_compact).unwrap().contains("span"));
    assert!(!String::from_utf8(vec_compact_pretty).unwrap().contains("span"));

    // Test writer functions
    let mut output = Vec::new();
    json::to_writer_compact(&mut output, &syn_file).unwrap();
    assert!(!String::from_utf8(output).unwrap().contains("span"));

    let mut output_pretty = Vec::new();
    json::to_writer_compact_pretty(&mut output_pretty, &syn_file).unwrap();
    assert!(!String::from_utf8(output_pretty).unwrap().contains("span"));
}

#[test]
fn test_compact_json_size_reduction() {
    let syn_file: syn::File = syn::parse_quote! {
        fn calculate(a: i32, b: i32) -> i32 {
            a + b
        }
    };

    let regular_json = json::to_string(&syn_file);
    let compact_json = json::to_string_compact(&syn_file);

    // Compact JSON should be significantly smaller
    assert!(compact_json.len() < regular_json.len());
    
    // Should still contain the function content
    assert!(compact_json.contains("calculate"));
    assert!(compact_json.contains("i32"));
}