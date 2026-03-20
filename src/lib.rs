//! # serde-kdl2
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
//! let config: Config = serde_kdl2::from_str(kdl_input).unwrap();
//! assert_eq!(config.title, "My App");
//! assert_eq!(config.count, 42);
//! assert_eq!(config.enabled, true);
//!
//! // Serialize
//! let output = serde_kdl2::to_string(&config).unwrap();
//! let roundtrip: Config = serde_kdl2::from_str(&output).unwrap();
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
//! ### Bare Node Defaults
//!
//! You can specify custom defaults for bare node names (nodes without arguments)
//! using the provided helper functions:
//!
//! ```rust
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Config {
//!     #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_true")]
//!     enabled: bool,
//!     
//!     #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_false")]
//!     debug: bool,
//! }
//! ```
//!
//! ```kdl
//! enabled          // → true
//! debug            // → false
//! enabled #false   // → false (explicit override)
//! debug #true      // → true (explicit override)
//! ```
//!
//! For other types, you can create custom deserializer functions following
//! the same visitor pattern (see examples for details).
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

/// Serde helpers for custom defaults with bare node names.
pub mod bare_defaults {
    use serde::{de, Deserializer};

    /// Boolean-specific bare default deserializers.
    pub mod bool {
        use super::*;

        /// Deserializes a boolean field where bare node names default to `true`.
        /// 
        /// Use with `#[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_true")]`.
        /// 
        /// # Examples
        /// 
        /// ```kdl
        /// enabled        // → true
        /// enabled #true  // → true
        /// enabled #false // → false
        /// ```
        pub fn bare_true<'de, D>(deserializer: D) -> Result<bool, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize_bool_with_bare_default(deserializer, true)
        }

        /// Deserializes a boolean field where bare node names default to `false`.
        /// 
        /// Use with `#[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_false")]`.
        /// 
        /// # Examples
        /// 
        /// ```kdl
        /// disabled        // → false
        /// disabled #true  // → true
        /// disabled #false // → false
        /// ```
        pub fn bare_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize_bool_with_bare_default(deserializer, false)
        }

        fn deserialize_bool_with_bare_default<'de, D>(
            deserializer: D,
            default_value: bool,
        ) -> Result<bool, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct BareDefaultBoolVisitor {
                default_value: bool,
            }

            impl<'de> de::Visitor<'de> for BareDefaultBoolVisitor {
                type Value = bool;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "a boolean value or bare node name")
                }

                fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(value)
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(self.default_value)
                }
            }

            deserializer.deserialize_any(BareDefaultBoolVisitor { default_value })
        }
    }

    // For other types, users can create custom deserializer functions following
    // the pattern demonstrated in the bool module above.
}


