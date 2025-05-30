//! This crate provides the syntax definitions and parsing utilities for the mCRL3 language.
//!
//! The crate is organized into several modules:
//!
//! - `ast`: Defines the abstract syntax tree (AST) for the mCRL3 language.
//! - `display`: Implements display and formatting traits for the AST nodes.
//! - `grammar`: Contains the grammar definitions for the mCRL3 language.
//! - `parse`: Provides parsing utilities to convert source code into AST nodes.
//! - `precedence`: Manages operator precedence rules for the mCRL3 language.
//!
//! # Features
//!
//! - Forbids the use of unsafe code to ensure memory safety.
//! - Re-exports key modules and types for easier access.
//!
//! # Examples
//!
//! ```rust
//! use syntax::parse::parse_expression;
//! use syntax::ast::Expression;
//!
//! let source = "a + b * c";
//! let expr: Expression = parse_expression(source).expect("Failed to parse expression");
//! println!("{:?}", expr);
//! ```
//!
//! This crate contains no unsafe code.
#![forbid(unsafe_code)]

mod ast;
mod display_ast;
mod display_pair;
mod grammar;
mod parse;
mod precedence;

pub use ast::*;
pub use display_ast::*;
pub use display_pair::*;
pub use grammar::*;
pub use parse::*;
pub use precedence::*;
