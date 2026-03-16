//! # serde-kdl
//!
//! [Serde](https://serde.rs) integration for [KDL](https://kdl.dev), the
//! KDL Document Language.
//!
//! This crate provides `serialize` and `deserialize` support for KDL documents
//! using the [`kdl`](https://crates.io/crates/kdl) crate (v6, KDL v2 spec).
//!
//! ## Quick Start
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct Config {
//!     title: String,
//!     count: i32,
//!     enabled: bool,
//! }
//!
//! let kdl_input = r#"
//! title "My App"
//! count 42
//! enabled #true
//! "#;
//!
//! // Deserialize
//! let config: Config = serde_kdl::from_str(kdl_input).unwrap();
//! assert_eq!(config.title, "My App");
//! assert_eq!(config.count, 42);
//! assert_eq!(config.enabled, true);
//!
//! // Serialize
//! let output = serde_kdl::to_string(&config).unwrap();
//! let roundtrip: Config = serde_kdl::from_str(&output).unwrap();
//! assert_eq!(config, roundtrip);
//! ```
//!
//! ## Mapping Rules
//!
//! ### Structs and Maps
//!
//! Struct fields map to node names. Each field becomes a node whose name is
//! the field name and whose first argument is the value.
//!
//! ```kdl
//! title "My App"
//! count 42
//! enabled #true
//! ```
//!
//! ### Nested Structs
//!
//! Nested structs use children blocks:
//!
//! ```kdl
//! server {
//!     host "localhost"
//!     port 8080
//! }
//! ```
//!
//! ### Sequences
//!
//! Sequences of primitives use multiple arguments on a single node:
//!
//! ```kdl
//! tags "web" "rust" "config"
//! ```
//!
//! Sequences of structs use repeated nodes with the same name:
//!
//! ```kdl
//! server {
//!     host "localhost"
//!     port 8080
//! }
//! server {
//!     host "example.com"
//!     port 443
//! }
//! ```
//!
//! The `-` (dash) children convention is also supported for deserialization:
//!
//! ```kdl
//! items {
//!     - 1
//!     - 2
//!     - 3
//! }
//! ```
//!
//! ### Option
//!
//! `None` is represented by the absence of a node. `Some(value)` serializes
//! the inner value normally. `#null` arguments also deserialize as `None`.
//!
//! ### Enums
//!
//! Unit variants serialize as strings:
//!
//! ```kdl
//! color "Red"
//! ```
//!
//! Newtype, tuple, and struct variants use the variant name as a child node:
//!
//! ```kdl
//! shape {
//!     Circle {
//!         radius 5.0
//!     }
//! }
//! ```

pub mod de;
pub mod error;
pub mod ser;

pub use de::{from_doc, from_str};
pub use error::Error;
pub use ser::{to_doc, to_string, to_string_pretty};
