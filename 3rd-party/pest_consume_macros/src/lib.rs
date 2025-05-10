#![doc(html_root_url = "https://docs.rs/pest_consume_macros/1.1.0")]

//! This crate contains the code-generation primitives for the [pest_consume](https://docs.rs/pest_consume) crate.
//! See there for documentation.
//!
//! It provides two main macro functionalities:
//! - `parser`: Generates the implementation for a pest_consume parser
//! - `match_nodes`: Provides pattern matching capabilities for parsing nodes

extern crate proc_macro;

mod make_parser;
mod match_nodes;

use proc_macro::TokenStream;

/// Attribute macro for generating a pest_consume parser implementation.
#[proc_macro_attribute]
pub fn parser(attrs: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(match make_parser::make_parser(attrs, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    })
}

/// Procedural macro for pattern matching against parse nodes.
///
/// Provides a pattern matching syntax for working with parse trees,
/// supporting complex patterns with rule matching and binding.
#[proc_macro]
pub fn match_nodes(input: TokenStream) -> TokenStream {
    TokenStream::from(match match_nodes::match_nodes(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    })
}
