// SPDX-License-Identifier: Apache-2.0 OR MIT

/*!
<!-- Note: Document from sync-markdown-to-rustdoc:start through sync-markdown-to-rustdoc:end
     is synchronized from README.md. Any changes to that range are not preserved. -->
<!-- tidy:sync-markdown-to-rustdoc:start -->

Library to serialize and deserialize [Syn] syntax trees.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
syn-serde = "0.3"
```

## Examples

```toml
[dependencies]
syn-serde = { version = "0.3", features = ["json"] }
syn = { version = "2", features = ["full"] }
```

```
use syn_serde::json;

let syn_file: syn::File = syn::parse_quote! {
    fn main() {
        println!("Hello, world!");
    }
};

println!("{}", json::to_string_pretty(&syn_file));
```

This prints the following JSON:

```json
{
  "items": [
    {
      "fn": {
        "ident": "main",
        "inputs": [],
        "output": null,
        "stmts": [
          {
            "semi": {
              "macro": {
                "path": {
                  "segments": [
                    {
                      "ident": "println"
                    }
                  ]
                },
                "delimiter": "paren",
                "tokens": [
                  {
                    "lit": "\"Hello, world!\""
                  }
                ]
              }
            }
          }
        ]
      }
    }
  ]
}
```

### Rust source file -> JSON representation of the syntax tree

The [`rust2json`] example parse a Rust source file into a `syn_serde::File`
and print out a JSON representation of the syntax tree.

### JSON file -> Rust syntax tree

The [`json2rust`] example parse a JSON file into a `syn_serde::File` and
print out a Rust syntax tree.

## Optional features

- **`json`** — Provides functions for JSON <-> Rust serializing and
  deserializing.

## Relationship to Syn

syn-serde is a fork of [Syn], and syn-serde provides a set of data structures
similar but not identical to [Syn]. All data structures provided by syn-serde
can be converted to the data structures of [Syn] and [proc-macro2].

The data structures of syn-serde 0.3 is compatible with the data structures of
[Syn] 2.x.

[Syn]: https://github.com/dtolnay/syn
[proc-macro2]: https://github.com/alexcrichton/proc-macro2
[`rust2json`]: https://github.com/taiki-e/syn-serde/tree/HEAD/examples/rust2json
[`json2rust`]: https://github.com/taiki-e/syn-serde/tree/HEAD/examples/json2rust

<!-- tidy:sync-markdown-to-rustdoc:end -->
*/

#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_variables)
    )
))]
#![forbid(unsafe_code)]
#![warn(
    // Lints that may help when writing public library.
    // missing_debug_implementations,
    // missing_docs,
    clippy::alloc_instead_of_core,
    // clippy::exhaustive_enums, // TODO
    // clippy::exhaustive_structs, // TODO
    clippy::impl_trait_in_params,
    // clippy::missing_inline_in_public_items,
    // clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
)]
#![allow(
    clippy::enum_glob_use,
    clippy::needless_doctest_main,
    clippy::used_underscore_binding,
    clippy::wildcard_imports
)]
// docs.rs only (cfg is enabled by docs.rs, not build script)
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
mod macros;

#[path = "gen/ast_struct.rs"]
mod ast_struct;

#[path = "gen/ast_enum.rs"]
mod ast_enum;

#[path = "gen/convert.rs"]
mod convert;

mod attr {
    pub use crate::{
        ast_enum::{AttrStyle, Meta},
        ast_struct::{Attribute, MetaList, MetaNameValue},
    };
}
#[doc(hidden)]
pub use crate::attr::{AttrStyle, Attribute, Meta, MetaList, MetaNameValue};

mod data;
pub(crate) use crate::data::assert_struct_semi;
#[doc(hidden)]
pub use crate::data::{Field, Fields, FieldsNamed, FieldsUnnamed, Variant};

mod expr;
#[doc(hidden)]
pub use crate::expr::{
    Arm, Expr, ExprArray, ExprAssign, ExprAsync, ExprAwait, ExprBinary, ExprBlock, ExprBreak,
    ExprCall, ExprCast, ExprClosure, ExprConst, ExprContinue, ExprField, ExprForLoop, ExprGroup,
    ExprIf, ExprIndex, ExprInfer, ExprLet, ExprLit, ExprLoop, ExprMacro, ExprMatch, ExprMethodCall,
    ExprParen, ExprPath, ExprRange, ExprReference, ExprRepeat, ExprReturn, ExprStruct, ExprTry,
    ExprTryBlock, ExprTuple, ExprUnary, ExprUnsafe, ExprWhile, ExprYield, FieldValue, Index, Label,
    Member, RangeLimits,
};

mod file {
    use std::collections::HashMap;
    pub use crate::ast_struct::File;
    
    impl File {
        /// Create a File from a syn::File and source code, distributing comments to appropriate AST nodes.
        pub fn from_syn_with_comments(syn_file: &syn::File, source: &str) -> Self {
            // First, create the basic file structure
            let mut file = Self::from(syn_file);
            
            // Extract comments from the source code
            let comments = crate::comment::extract_comments(source);
            
            // Associate comments with AST nodes
            let comment_associations = associate_comments_with_file_nodes(&comments, &file);
            
            // Apply the comment associations to the file structure
            apply_comment_associations(&mut file, comment_associations);
            
            file
        }
    }
    
    /// Associate comments with AST nodes in a file
    fn associate_comments_with_file_nodes(
        comments: &[crate::Comment],
        file: &File,
    ) -> HashMap<String, Vec<crate::Comment>> {
        let mut node_spans = Vec::new();
        
        // Collect spans from all items in the file
        for (i, item) in file.items.iter().enumerate() {
            collect_item_spans(item, &format!("item_{}", i), &mut node_spans);
        }
        
        // Associate comments with nodes
        crate::comment_association::associate_comments_with_nodes(comments, &node_spans)
    }
    
    /// Collect span information from an item and its children
    fn collect_item_spans(item: &crate::Item, item_id: &str, spans: &mut Vec<(String, crate::SpanInfo)>) {
        // Add span information for different item types
        match item {
            crate::Item::Fn(item_fn) => {
                if let Some(span) = &item_fn.span {
                    spans.push((item_id.to_string(), span.clone()));
                }
                // Add block span if present
                if let Some(block_span) = &item_fn.block.span {
                    spans.push((format!("{}_block", item_id), block_span.clone()));
                }
            }
            crate::Item::Enum(item_enum) => {
                if let Some(span) = &item_enum.span {
                    spans.push((item_id.to_string(), span.clone()));
                }
            }
            crate::Item::Struct(_item_struct) => {
                // ItemStruct doesn't have span in the current implementation
            }
            crate::Item::Trait(item_trait) => {
                if let Some(span) = &item_trait.span {
                    spans.push((item_id.to_string(), span.clone()));
                }
            }
            crate::Item::Impl(item_impl) => {
                if let Some(span) = &item_impl.span {
                    spans.push((item_id.to_string(), span.clone()));
                }
            }
            crate::Item::Use(item_use) => {
                if let Some(span) = &item_use.span {
                    spans.push((item_id.to_string(), span.clone()));
                }
            }
            crate::Item::Const(item_const) => {
                if let Some(span) = &item_const.span {
                    spans.push((item_id.to_string(), span.clone()));
                }
            }
            crate::Item::Static(item_static) => {
                if let Some(span) = &item_static.span {
                    spans.push((item_id.to_string(), span.clone()));
                }
            }
            crate::Item::Type(item_type) => {
                if let Some(span) = &item_type.span {
                    spans.push((item_id.to_string(), span.clone()));
                }
            }
            crate::Item::Union(_item_union) => {
                // ItemUnion doesn't have span in the current implementation
            }
            crate::Item::Mod(_item_mod) => {
                // ItemMod doesn't have span in the current implementation
            }
            crate::Item::ForeignMod(_item_foreign_mod) => {
                // ItemForeignMod doesn't have span in the current implementation
            }
            crate::Item::TraitAlias(_item_trait_alias) => {
                // ItemTraitAlias doesn't have span in the current implementation
            }
            crate::Item::Macro(_item_macro) => {
                // ItemMacro doesn't have span in the current implementation
            }
            crate::Item::ExternCrate(_item_extern_crate) => {
                // ItemExternCrate doesn't have span in the current implementation
            }
            crate::Item::Verbatim(_) => {
                // Verbatim items don't have spans
            }
        }
    }
    
    /// Apply comment associations to the file structure
    fn apply_comment_associations(file: &mut File, associations: HashMap<String, Vec<crate::Comment>>) {
        for (i, item) in file.items.iter_mut().enumerate() {
            let item_id = format!("item_{}", i);
            
            // Apply comments to the item
            if let Some(comments) = associations.get(&item_id) {
                apply_comments_to_item(item, comments.clone());
            }
            
            // Apply comments to the item's block if it's a function
            if let crate::Item::Fn(item_fn) = item {
                let block_id = format!("{}_block", item_id);
                if let Some(comments) = associations.get(&block_id) {
                    item_fn.block.comments = comments.clone();
                }
            }
        }
    }
    
    /// Apply comments to a specific item
    fn apply_comments_to_item(item: &mut crate::Item, comments: Vec<crate::Comment>) {
        match item {
            crate::Item::Fn(item_fn) => {
                item_fn.comments = comments;
            }
            crate::Item::Enum(item_enum) => {
                item_enum.comments = comments;
            }
            crate::Item::Struct(_item_struct) => {
                // ItemStruct doesn't have comments field
            }
            crate::Item::Trait(item_trait) => {
                item_trait.comments = comments;
            }
            crate::Item::Impl(item_impl) => {
                item_impl.comments = comments;
            }
            crate::Item::Use(item_use) => {
                item_use.comments = comments;
            }
            crate::Item::Const(item_const) => {
                item_const.comments = comments;
            }
            crate::Item::Static(item_static) => {
                item_static.comments = comments;
            }
            crate::Item::Type(item_type) => {
                item_type.comments = comments;
            }
            crate::Item::Union(item_union) => {
                item_union.comments = comments;
            }
            crate::Item::Mod(_item_mod) => {
                // ItemMod doesn't have comments field
            }
            crate::Item::ForeignMod(item_foreign_mod) => {
                item_foreign_mod.comments = comments;
            }
            crate::Item::TraitAlias(_item_trait_alias) => {
                // ItemTraitAlias doesn't have comments field
            }
            crate::Item::Macro(_item_macro) => {
                // ItemMacro doesn't have comments field
            }
            crate::Item::ExternCrate(_item_extern_crate) => {
                // ItemExternCrate doesn't have comments field
            }
            crate::Item::Verbatim(_) => {
                // Can't attach comments to verbatim items
            }
        }
    }
}
#[doc(hidden)]
pub use crate::file::File;

mod generics;
#[doc(hidden)]
pub use crate::generics::{
    BoundLifetimes, ConstParam, GenericParam, Generics, LifetimeParam, PredicateLifetime,
    PredicateType, TraitBound, TraitBoundModifier, TypeParam, TypeParamBound, WhereClause,
    WherePredicate,
};

mod item;
#[doc(hidden)]
pub use crate::item::{
    FnArg, ForeignItem, ForeignItemFn, ForeignItemMacro, ForeignItemStatic, ForeignItemType,
    ImplItem, ImplItemConst, ImplItemFn, ImplItemMacro, ImplItemType, ImplRestriction, Item,
    ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Receiver,
    Signature, StaticMutability, TraitItem, TraitItemConst, TraitItemFn, TraitItemMacro,
    TraitItemType, UseGroup, UseName, UsePath, UseRename, UseTree, Variadic,
};

mod lifetime {
    pub use crate::ast_struct::Lifetime;
}
#[doc(hidden)]
pub use crate::lifetime::Lifetime;

mod lit;
#[doc(hidden)]
pub use crate::lit::{
    Lit, LitBool, LitByte, LitByteStr, LitChar, LitFloat, LitInt, LitStr, StrStyle,
};

mod mac {
    pub use crate::{ast_enum::MacroDelimiter, ast_struct::Macro};
}
#[doc(hidden)]
pub use crate::mac::{Macro, MacroDelimiter};

mod op {
    pub use crate::ast_enum::{BinOp, UnOp};
}
#[doc(hidden)]
pub use crate::op::{BinOp, UnOp};

mod pat;
#[doc(hidden)]
pub use crate::expr::{
    ExprConst as PatConst, ExprLit as PatLit, ExprMacro as PatMacro, ExprPath as PatPath,
    ExprRange as PatRange,
};
#[doc(hidden)]
pub use crate::pat::{
    FieldPat, Pat, PatIdent, PatOr, PatParen, PatReference, PatRest, PatSlice, PatStruct, PatTuple,
    PatTupleStruct, PatType, PatWild,
};

mod path;
#[doc(hidden)]
pub use crate::path::{
    AngleBracketedGenericArguments, AssocConst, AssocType, Constraint, GenericArgument,
    ParenthesizedGenericArguments, Path, PathArguments, PathSegment, QSelf,
};

mod restriction;
#[doc(hidden)]
pub use crate::restriction::{FieldMutability, VisRestricted, Visibility};

mod stmt;
#[doc(hidden)]
pub use crate::stmt::{Block, Local, LocalInit, Stmt, StmtMacro};

mod ty;
#[doc(hidden)]
pub use crate::ty::{
    Abi, BareFnArg, BareVariadic, ReturnType, Type, TypeArray, TypeBareFn, TypeGroup,
    TypeImplTrait, TypeMacro, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice,
    TypeTraitObject, TypeTuple,
};

mod token_stream;
#[doc(hidden)]
pub use crate::token_stream::{
    Delimiter, Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree,
};

mod span;
#[doc(hidden)]
pub use crate::span::SpanInfo;

mod comment;
#[doc(hidden)]
pub use crate::comment::{Comment, CommentKind};

mod comment_association;

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub mod json;

mod sealed {
    #[allow(unknown_lints, unnameable_types)] // Not public API. unnameable_types is available on Rust 1.79+
    pub trait Sealed {}
}

// -----------------------------------------------------------------------------
// Syn trait

/// A trait for the data structures of [Syn] and [proc-macro2].
///
/// [Syn]: https://github.com/dtolnay/syn
/// [proc-macro2]: https://github.com/alexcrichton/proc-macro2
pub trait Syn: Sized + sealed::Sealed {
    type Adapter: Serialize + for<'de> Deserialize<'de>;

    /// Converts a `Syn` type into an adapter.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "json")]
    /// # fn dox() {
    /// use syn_serde::Syn;
    ///
    /// let syn_file: syn::File = syn::parse_quote! {
    ///     fn main() {
    ///         println!("Hello, world!");
    ///     }
    /// };
    ///
    /// let serializable_file = syn_file.to_adapter();
    /// println!("{}", serde_json::to_string_pretty(&serializable_file).unwrap());
    /// # }
    /// # fn main() {} // rustdoc bug: https://github.com/rust-lang/rust/issues/131893
    /// ```
    fn to_adapter(&self) -> Self::Adapter;

    /// Converts an adapter into a `Syn` type.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "json")]
    /// # fn dox() -> Result<(), Box<dyn std::error::Error>> {
    /// use syn_serde::Syn;
    ///
    /// // `struct Unit;`
    /// let json = r#"{
    ///   "struct": {
    ///     "ident": "Unit",
    ///     "fields": "unit"
    ///   }
    /// }"#;
    ///
    /// let serializable_file: <syn::File as Syn>::Adapter = serde_json::from_str(json)?;
    /// let syn_file = syn::File::from_adapter(&serializable_file);
    /// # Ok(())
    /// # }
    /// ```
    fn from_adapter(adapter: &Self::Adapter) -> Self;
}

// -----------------------------------------------------------------------------

use core::ops;

use proc_macro2::Span;
use serde::{de::Deserialize, ser::Serialize};
use serde_derive::{Deserialize, Serialize};

type Punctuated<T> = Vec<T>;

fn default<T>() -> T
where
    T: Default,
{
    T::default()
}

fn default_or_none<T>(x: bool) -> Option<T>
where
    T: Default,
{
    if x { Some(T::default()) } else { None }
}

fn not<T>(x: T) -> T::Output
where
    T: ops::Not,
{
    !x
}

// https://github.com/rust-lang/rust/issues/51443
trait RefInto<U>: Sized {
    fn ref_into<'a>(&'a self) -> U
    where
        &'a Self: Into<U>,
    {
        self.into()
    }
}

impl<T, U> RefInto<U> for T {}

trait MapInto<U, M> {
    type T;

    fn ref_map<'a, F>(&'a self, f: F) -> M
    where
        Self::T: 'a,
        F: FnMut(&'a Self::T) -> U;

    fn map_into<'a>(&'a self) -> M
    where
        Self::T: 'a,
        &'a Self::T: Into<U>,
    {
        self.ref_map(Into::into)
    }
}

impl<T, U> MapInto<U, Vec<U>> for Vec<T> {
    type T = T;

    fn ref_map<'a, F>(&'a self, f: F) -> Vec<U>
    where
        F: FnMut(&'a Self::T) -> U,
    {
        self.iter().map(f).collect()
    }
}

impl<T, U, P> MapInto<U, syn::punctuated::Punctuated<U, P>> for Vec<T>
where
    P: Default,
{
    type T = T;

    fn ref_map<'a, F>(&'a self, f: F) -> syn::punctuated::Punctuated<U, P>
    where
        F: FnMut(&'a Self::T) -> U,
    {
        self.iter().map(f).collect()
    }
}

impl<T, U, P> MapInto<U, Vec<U>> for syn::punctuated::Punctuated<T, P>
where
    P: Default,
{
    type T = T;

    fn ref_map<'a, F>(&'a self, f: F) -> Vec<U>
    where
        F: FnMut(&'a Self::T) -> U,
    {
        self.iter().map(f).collect()
    }
}

impl<T, U> MapInto<U, Option<U>> for Option<T> {
    type T = T;

    fn ref_map<'a, F>(&'a self, f: F) -> Option<U>
    where
        F: FnMut(&'a Self::T) -> U,
    {
        self.as_ref().map(f)
    }
}

impl<T, U> MapInto<U, Box<U>> for Box<T> {
    type T = T;

    fn ref_map<'a, F>(&'a self, mut f: F) -> Box<U>
    where
        F: FnMut(&'a Self::T) -> U,
    {
        Box::new(f(self))
    }
}
