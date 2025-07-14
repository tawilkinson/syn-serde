// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Serializable span information for tracking location in source code.
//!
//! This module provides the [`SpanInfo`] struct which captures location information
//! from [`proc_macro2::Span`] in a serializable format. This enables preserving
//! source code location data when serializing/deserializing syn AST nodes.
//!
//! # Example
//!
//! ```rust
//! use syn_serde::SpanInfo;
//! use proc_macro2::Span;
//!
//! let span = Span::call_site();
//! let span_info = SpanInfo::from_span(span);
//! 
//! // Serialize to JSON
//! let json = serde_json::to_string(&span_info).unwrap();
//! println!("Span: {}", json);
//!
//! // Deserialize back
//! let restored: SpanInfo = serde_json::from_str(&json).unwrap();
//! assert_eq!(span_info, restored);
//! ```

use proc_macro2::Span;
use serde_derive::{Deserialize, Serialize};

/// Serializable representation of span information.
/// 
/// This preserves location information from the original source code,
/// including byte offsets and line/column positions. When the `span-locations`
/// feature is enabled in `proc-macro2`, this captures accurate line and column
/// information. Otherwise, it provides default values.
///
/// # Note on byte offsets
/// 
/// The `start_offset` and `end_offset` fields are currently set to 0 because
/// `proc_macro2::Span` doesn't expose byte offset information directly. These
/// fields are reserved for future use or can be populated by external tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanInfo {
    /// Byte offset of the start of the span (currently always 0)
    pub start_offset: usize,
    /// Byte offset of the end of the span (currently always 0)
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
    /// This captures line/column information when available (requires the
    /// `span-locations` feature in `proc-macro2`). If span location information
    /// is not available, fallback values are used.
    ///
    /// # Example
    ///
    /// ```rust
    /// use syn_serde::SpanInfo;
    /// use proc_macro2::Span;
    ///
    /// let span = Span::call_site();
    /// let span_info = SpanInfo::from_span(span);
    /// assert!(span_info.start_line >= 1);
    /// ```
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
    /// Note: This creates a span at `call_site()` since `proc_macro2`
    /// doesn't allow creating spans at arbitrary locations. The location
    /// information in `SpanInfo` is preserved for other uses.
    ///
    /// # Example
    ///
    /// ```rust
    /// use syn_serde::SpanInfo;
    /// use proc_macro2::Span;
    ///
    /// let original = Span::call_site();
    /// let span_info = SpanInfo::from_span(original);
    /// let restored = span_info.to_span();
    /// 
    /// // The restored span will be call_site(), but span_info retains the location data
    /// ```
    pub fn to_span(&self) -> Span {
        Span::call_site()
    }
    
    /// Create a default SpanInfo (used when span information is not available).
    ///
    /// This is equivalent to `SpanInfo::from_span(Span::call_site())`.
    pub fn call_site() -> Self {
        Self::from_span(Span::call_site())
    }

    /// Check if this span represents a single point (start == end).
    pub fn is_point(&self) -> bool {
        self.start_line == self.end_line && self.start_column == self.end_column
    }

    /// Get the length in columns (for single-line spans).
    /// 
    /// Returns `None` if the span crosses multiple lines.
    pub fn column_length(&self) -> Option<usize> {
        if self.start_line == self.end_line {
            Some(self.end_column.saturating_sub(self.start_column))
        } else {
            None
        }
    }
}

impl Default for SpanInfo {
    fn default() -> Self {
        Self::call_site()
    }
}

// Conversion traits for compatibility with proc_macro2::Span
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