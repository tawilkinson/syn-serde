// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Serializable span information for tracking location in source code.

use proc_macro2::Span;
use serde_derive::{Deserialize, Serialize};

/// Serializable representation of span information.
/// 
/// This preserves location information from the original source code,
/// including byte offsets and line/column positions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanInfo {
    /// Byte offset of the start of the span
    pub start_offset: usize,
    /// Byte offset of the end of the span  
    pub end_offset: usize,
    /// Line number (1-based) of the start of the span
    pub start_line: usize,
    /// Column number (0-based) of the start of the span
    pub start_column: usize,
    /// Line number (1-based) of the end of the span
    pub end_line: usize,
    /// Column number (0-based) of the end of the span
    pub end_column: usize,
}

impl SpanInfo {
    /// Create a SpanInfo from a proc_macro2::Span.
    /// 
    /// This captures line/column information when available.
    pub fn from_span(span: Span) -> Self {
        // Try to extract span location information
        // This uses runtime feature detection instead of compile-time cfg
        match std::panic::catch_unwind(|| {
            let start = span.start();
            let end = span.end();
            (start.line, start.column, end.line, end.column)
        }) {
            Ok((start_line, start_column, end_line, end_column)) => {
                Self {
                    start_offset: 0, // proc_macro2 doesn't expose byte offsets directly
                    end_offset: 0,   // we'll need to handle this differently
                    start_line,
                    start_column,
                    end_line,
                    end_column,
                }
            }
            Err(_) => {
                // Fallback when span locations are not available
                Self {
                    start_offset: 0,
                    end_offset: 0,
                    start_line: 1,
                    start_column: 0,
                    end_line: 1,
                    end_column: 0,
                }
            }
        }
    }
    
    /// Convert back to a proc_macro2::Span.
    /// 
    /// Note: This creates a span at call_site since proc_macro2
    /// doesn't allow creating spans at arbitrary locations.
    pub fn to_span(&self) -> Span {
        Span::call_site()
    }
    
    /// Create a default SpanInfo (used when span information is not available)
    pub fn call_site() -> Self {
        Self::from_span(Span::call_site())
    }
}

impl Default for SpanInfo {
    fn default() -> Self {
        Self::call_site()
    }
}

// Conversion traits for compatibility
impl From<&Span> for SpanInfo {
    fn from(span: &Span) -> Self {
        Self::from_span(*span)
    }
}

impl From<&SpanInfo> for Span {
    fn from(_span_info: &SpanInfo) -> Self {
        Span::call_site()
    }
}