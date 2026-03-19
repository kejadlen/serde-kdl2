//! Deserializer implementation for KDL.
//!
//! # Mapping Rules
//!
//! - **Struct/Map** ↔ KDL document (or children block). Each node name is a key.
//! - **Primitives** ↔ A node's first argument: `name "value"` or `count 42`.
//! - **Nested struct** ↔ A node's children block: `nested { field "val" }`.
//! - **Vec/sequence** ↔ Either multiple nodes with the same name, or a single
//!   node with multiple arguments, or `-` children convention.
//! - **Option** ↔ Absent node = `None`, present node = `Some`.
//! - **Enum (unit variant)** ↔ String argument: `color "Red"`.
//! - **Enum (newtype/tuple/struct)** ↔ Child node named after variant:
//!   `shape { Circle { radius 5.0 } }`.

use kdl::{KdlDocument, KdlNode, KdlValue};
use serde::Deserialize;
use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};

use crate::error::{Error, Result};

/// Deserialize a type from a KDL string.
pub fn from_str<'de, T: Deserialize<'de>>(s: &'de str) -> Result<T> {
    let doc: KdlDocument = s.parse()?;
    from_doc(&doc)
}

/// Deserialize a type from a [`KdlDocument`].
pub fn from_doc<'de, T: Deserialize<'de>>(doc: &KdlDocument) -> Result<T> {
    let de = DocumentDeserializer::new(doc);
    T::deserialize(de)
}

// ---------------------------------------------------------------------------
// DocumentDeserializer: deserializes a KdlDocument as a struct/map
// ---------------------------------------------------------------------------

struct DocumentDeserializer<'a> {
    doc: &'a KdlDocument,
}

impl<'a> DocumentDeserializer<'a> {
    fn new(doc: &'a KdlDocument) -> Self {
        Self { doc }
    }
}

impl<'de, 'a> de::Deserializer<'de> for DocumentDeserializer<'a> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_map(DocumentMapAccess::new(self.doc))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_map(DocumentMapAccess::new(self.doc))
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_unit()
    }

    // cov-excl-start
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf option
        seq tuple tuple_struct enum identifier
    }
    // cov-excl-stop
}

// ---------------------------------------------------------------------------
// DocumentMapAccess: iterates over unique node names in a document
// ---------------------------------------------------------------------------

struct DocumentMapAccess<'a> {
    doc: &'a KdlDocument,
    /// Unique node names, in order of first appearance
    keys: Vec<&'a str>,
    index: usize,
}

impl<'a> DocumentMapAccess<'a> {
    fn new(doc: &'a KdlDocument) -> Self {
        let mut keys = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for node in doc.nodes() {
            let name = node.name().value();
            if seen.insert(name) {
                keys.push(name);
            }
        }
        Self {
            doc,
            keys,
            index: 0,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for DocumentMapAccess<'a> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.index >= self.keys.len() {
            return Ok(None);
        }
        let key = self.keys[self.index];
        seed.deserialize(key.into_deserializer()).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let key = self.keys[self.index];
        self.index += 1;

        // Collect all nodes with this name
        let nodes: Vec<&KdlNode> = self
            .doc
            .nodes()
            .iter()
            .filter(|n| n.name().value() == key)
            .collect();

        seed.deserialize(FieldDeserializer::new(nodes))
    }
}

// ---------------------------------------------------------------------------
// FieldDeserializer: deserializes all nodes matching a field name
// ---------------------------------------------------------------------------

struct FieldDeserializer<'a> {
    nodes: Vec<&'a KdlNode>,
}

impl<'a> FieldDeserializer<'a> {
    fn new(nodes: Vec<&'a KdlNode>) -> Self {
        Self { nodes }
    }

    fn first_node(&self) -> Result<&'a KdlNode> {
        self.nodes
            .first()
            .copied()
            .ok_or_else(|| Error::Message("expected at least one node".into()))
    }

    /// Returns the single node, or errors if there are duplicates.
    /// Used by scalar deserialize methods to reject ambiguous duplicate nodes.
    fn only_node(&self) -> Result<&'a KdlNode> {
        if self.nodes.len() > 1 {
            return Err(Error::Message(format!(
                "duplicate node '{}': expected a single node for scalar field, found {}",
                self.nodes[0].name().value(),
                self.nodes.len()
            )));
        }
        self.first_node()
    }

    fn first_arg(&self) -> Result<&'a KdlValue> {
        let node = self.only_node()?;
        first_arg_of(node)
    }
}

fn first_arg_of(node: &KdlNode) -> Result<&KdlValue> {
    node.get(0).ok_or_else(|| Error::TypeMismatch {
        expected: "a value argument",
        got: format!("node '{}' with no arguments", node.name().value()),
    })
}

/// Helper: get all positional arguments from a node (entries with no name).
fn node_args(node: &KdlNode) -> Vec<&KdlValue> {
    node.entries()
        .iter()
        .filter(|e| e.name().is_none())
        .map(|e| e.value())
        .collect()
}

/// Helper: get all property entries from a node (entries with a name).
fn node_props(node: &KdlNode) -> Vec<(&str, &KdlValue)> {
    node.entries()
        .iter()
        .filter_map(|e| e.name().map(|n| (n.value(), e.value())))
        .collect()
}

impl<'de, 'a> de::Deserializer<'de> for FieldDeserializer<'a> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let node = self.first_node()?;
        let args = node_args(node);
        let props = node_props(node);

        if node.children().is_some() {
            // Has children → treat as struct/map
            self.deserialize_map(visitor)
        } else if !props.is_empty() {
            // Has properties → treat as map
            self.deserialize_map(visitor)
        } else if args.len() == 1 {
            // Single argument → deserialize as value
            ValueDeserializer::new(args[0]).deserialize_any(visitor)
        } else if args.len() > 1 {
            // Multiple arguments → deserialize as seq
            self.deserialize_seq(visitor)
        } else {
            // No args, no children, no props → null/unit
            visitor.visit_unit()
        }
    }

    // -- Primitives: delegate to first argument --

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_bool(visitor)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_i8(visitor)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_i16(visitor)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_i32(visitor)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_i64(visitor)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_i128(visitor)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_u8(visitor)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_u16(visitor)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_u32(visitor)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_u64(visitor)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_u128(visitor)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_f32(visitor)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_f64(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_char(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_str(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(self.first_arg()?).deserialize_string(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // If we got here, the node exists, so it's Some.
        // None is handled at the map level by the absence of a node.
        let node = self.first_node()?;
        let args = node_args(node);
        // If there's a single argument and it's null, treat as None
        if args.len() == 1 && args[0].is_null() && node.children().is_none() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // Strategy:
        // 1. Multiple nodes with same name → each node is an element
        // 2. Single node with `-` children → each `-` child is an element
        // 3. Single node with multiple args → each arg is an element
        // 4. Single node with non-`-` children → children nodes are elements

        if self.nodes.len() > 1 {
            // Multiple nodes → each node contributes one element
            visitor.visit_seq(MultiNodeSeqAccess {
                nodes: self.nodes,
                index: 0,
            })
        } else {
            let node = self.first_node()?;

            // Check for `-` children convention
            if let Some(children) = node.children() {
                let dash_nodes: Vec<&KdlNode> = children
                    .nodes()
                    .iter()
                    .filter(|n| n.name().value() == "-")
                    .collect();
                if !dash_nodes.is_empty() {
                    return visitor.visit_seq(MultiNodeSeqAccess {
                        nodes: dash_nodes,
                        index: 0,
                    });
                }
                // Non-dash children: treat each child as an element
                let child_nodes: Vec<&KdlNode> = children.nodes().iter().collect();
                if !child_nodes.is_empty() {
                    return visitor.visit_seq(ChildNodeSeqAccess {
                        nodes: child_nodes,
                        index: 0,
                    });
                }
            }

            // Arguments as elements
            let args = node_args(node);
            visitor.visit_seq(ArgsSeqAccess { args, index: 0 })
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let node = self.only_node()?;
        // If node has children, deserialize from children document
        if let Some(children) = node.children() {
            return visitor.visit_map(DocumentMapAccess::new(children));
        }
        // If node has properties, deserialize from properties
        let props = node_props(node);
        if !props.is_empty() {
            return visitor.visit_map(PropsMapAccess { props, index: 0 });
        }
        // Empty map
        visitor.visit_map(PropsMapAccess {
            props: vec![],
            index: 0,
        })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let node = self.only_node()?;

        // If node has children, deserialize struct from children
        if let Some(children) = node.children() {
            return visitor.visit_map(DocumentMapAccess::new(children));
        }

        // If node has properties, use them as struct fields
        let props = node_props(node);
        if !props.is_empty() {
            return visitor.visit_map(PropsMapAccess { props, index: 0 });
        }

        // If struct has a single field and node has a single argument,
        // try to match argument to the sole field
        let args = node_args(node);
        if fields.len() == 1 && args.len() == 1 {
            return visitor.visit_map(SingleArgStructAccess {
                field_name: fields[0],
                value: args[0],
                done: false,
            });
        }

        // Empty
        visitor.visit_map(PropsMapAccess {
            props: vec![],
            index: 0,
        })
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let node = self.only_node()?;
        let args = node_args(node);

        // Unit variant: first argument is variant name string
        // e.g., `color "Red"`
        if !args.is_empty()
            && node.children().is_none()
            && let Some(s) = args[0].as_string()
        {
            return visitor.visit_enum(EnumUnitAccess {
                variant: s,
                node,
                arg_offset: 1,
            });
        }

        // Complex variant: child node named after variant
        // e.g., `shape { Circle { radius 5.0 } }`
        if let Some(children) = node.children() {
            let child_nodes = children.nodes();
            if child_nodes.len() == 1 {
                let variant_node = &child_nodes[0];
                return visitor.visit_enum(EnumComplexAccess {
                    variant_name: variant_node.name().value(),
                    variant_node,
                });
            }
        }

        Err(Error::Message(format!(
            "cannot deserialize enum from node '{}'",
            node.name().value()
        )))
    }

    // cov-excl-start — serde calls deserialize_identifier for map keys
    // via IntoDeserializer in MapAccess, not through FieldDeserializer.
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }
    // cov-excl-stop

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }
}

// ---------------------------------------------------------------------------
// SeqAccess implementations
// ---------------------------------------------------------------------------

/// Seq from multiple nodes with the same name. Each node is deserialized as an element.
struct MultiNodeSeqAccess<'a> {
    nodes: Vec<&'a KdlNode>,
    index: usize,
}

impl<'de, 'a> SeqAccess<'de> for MultiNodeSeqAccess<'a> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.index >= self.nodes.len() {
            return Ok(None);
        }
        let node = self.nodes[self.index];
        self.index += 1;
        seed.deserialize(NodeContentDeserializer::new(node))
            .map(Some)
    }
}

/// Seq from child nodes (non-dash). Each child node is an element deserialized
/// by its content (children or first arg).
struct ChildNodeSeqAccess<'a> {
    nodes: Vec<&'a KdlNode>,
    index: usize,
}

impl<'de, 'a> SeqAccess<'de> for ChildNodeSeqAccess<'a> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.index >= self.nodes.len() {
            return Ok(None);
        }
        let node = self.nodes[self.index];
        self.index += 1;
        seed.deserialize(NodeContentDeserializer::new(node))
            .map(Some)
    }
}

/// Seq from a node's positional arguments.
struct ArgsSeqAccess<'a> {
    args: Vec<&'a KdlValue>,
    index: usize,
}

impl<'de, 'a> SeqAccess<'de> for ArgsSeqAccess<'a> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.index >= self.args.len() {
            return Ok(None);
        }
        let val = self.args[self.index];
        self.index += 1;
        seed.deserialize(ValueDeserializer::new(val)).map(Some)
    }
}

// ---------------------------------------------------------------------------
// PropsMapAccess: iterate over node properties as key-value pairs
// ---------------------------------------------------------------------------

struct PropsMapAccess<'a> {
    props: Vec<(&'a str, &'a KdlValue)>,
    index: usize,
}

impl<'de, 'a> MapAccess<'de> for PropsMapAccess<'a> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.index >= self.props.len() {
            return Ok(None);
        }
        let (key, _) = self.props[self.index];
        seed.deserialize(key.into_deserializer()).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let (_, val) = self.props[self.index];
        self.index += 1;
        seed.deserialize(ValueDeserializer::new(val))
    }
}

// ---------------------------------------------------------------------------
// SingleArgStructAccess: a struct with one field from a node's single arg
// ---------------------------------------------------------------------------

struct SingleArgStructAccess<'a> {
    field_name: &'static str,
    value: &'a KdlValue,
    done: bool,
}

impl<'de, 'a> MapAccess<'de> for SingleArgStructAccess<'a> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.done {
            return Ok(None);
        }
        seed.deserialize(self.field_name.into_deserializer())
            .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        self.done = true;
        seed.deserialize(ValueDeserializer::new(self.value))
    }
}

// ---------------------------------------------------------------------------
// NodeContentDeserializer: deserializes from a single node's content
// ---------------------------------------------------------------------------

/// Deserializes a single node's "content" - its arguments, properties, and children.
/// Used when a node represents a single value in a sequence.
struct NodeContentDeserializer<'a> {
    node: &'a KdlNode,
}

impl<'a> NodeContentDeserializer<'a> {
    fn new(node: &'a KdlNode) -> Self {
        Self { node }
    }
}

impl<'de, 'a> de::Deserializer<'de> for NodeContentDeserializer<'a> {
    type Error = Error;

    // cov-excl-start — serde's Content buffering can't round-trip through
    // tree-structured deserializers, making this unreachable via public API.
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let args = node_args(self.node);
        let props = node_props(self.node);

        if self.node.children().is_some() || !props.is_empty() {
            self.deserialize_map(visitor)
        } else if args.len() == 1 {
            ValueDeserializer::new(args[0]).deserialize_any(visitor)
        } else if args.len() > 1 {
            self.deserialize_seq(visitor)
        } else {
            visitor.visit_unit()
        }
    }
    // cov-excl-stop

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_bool(visitor)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_i8(visitor)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_i16(visitor)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_i32(visitor)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_i64(visitor)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_i128(visitor)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_u8(visitor)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_u16(visitor)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_u32(visitor)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_u64(visitor)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_u128(visitor)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_f32(visitor)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_f64(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_char(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_str(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        ValueDeserializer::new(first_arg_of(self.node)?).deserialize_string(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let args = node_args(self.node);
        if args.len() == 1 && args[0].is_null() && self.node.children().is_none() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // Check for `-` children convention first
        if let Some(children) = self.node.children() {
            let dash_nodes: Vec<&KdlNode> = children
                .nodes()
                .iter()
                .filter(|n| n.name().value() == "-")
                .collect();
            if !dash_nodes.is_empty() {
                return visitor.visit_seq(MultiNodeSeqAccess {
                    nodes: dash_nodes,
                    index: 0,
                });
            }
            // Non-dash children
            let child_nodes: Vec<&KdlNode> = children.nodes().iter().collect();
            if !child_nodes.is_empty() {
                return visitor.visit_seq(ChildNodeSeqAccess {
                    nodes: child_nodes,
                    index: 0,
                });
            }
        }
        let args = node_args(self.node);
        visitor.visit_seq(ArgsSeqAccess { args, index: 0 })
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(children) = self.node.children() {
            return visitor.visit_map(DocumentMapAccess::new(children));
        }
        let props = node_props(self.node);
        visitor.visit_map(PropsMapAccess { props, index: 0 })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        if let Some(children) = self.node.children() {
            return visitor.visit_map(DocumentMapAccess::new(children));
        }
        let props = node_props(self.node);
        if !props.is_empty() {
            return visitor.visit_map(PropsMapAccess { props, index: 0 });
        }
        // Single-arg struct
        let args = node_args(self.node);
        if fields.len() == 1 && args.len() == 1 {
            return visitor.visit_map(SingleArgStructAccess {
                field_name: fields[0],
                value: args[0],
                done: false,
            });
        }
        visitor.visit_map(PropsMapAccess {
            props: vec![],
            index: 0,
        })
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let args = node_args(self.node);

        // Unit variant: first argument is variant name
        if !args.is_empty()
            && self.node.children().is_none()
            && let Some(s) = args[0].as_string()
        {
            return visitor.visit_enum(EnumUnitAccess {
                variant: s,
                node: self.node,
                arg_offset: 1,
            });
        }

        // cov-excl-start — complex variant through NodeContentDeserializer
        // requires a repeated-node element with a single child node naming
        // a variant. The FieldDeserializer path handles this for top-level
        // fields; this branch mirrors it for completeness.
        // Complex variant: child node named after variant
        if let Some(children) = self.node.children() {
            let child_nodes = children.nodes();
            if child_nodes.len() == 1 {
                let variant_node = &child_nodes[0];
                return visitor.visit_enum(EnumComplexAccess {
                    variant_name: variant_node.name().value(),
                    variant_node,
                });
            }
        }

        Err(Error::Message(format!(
            "cannot deserialize enum from node '{}'",
            self.node.name().value()
        )))
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }
    // cov-excl-stop
}

// ---------------------------------------------------------------------------
// ValueDeserializer: deserializes a KdlValue as a primitive
// ---------------------------------------------------------------------------

struct ValueDeserializer<'a> {
    value: &'a KdlValue,
}

impl<'a> ValueDeserializer<'a> {
    fn new(value: &'a KdlValue) -> Self {
        Self { value }
    }

    fn to_i128(&self) -> Result<i128> {
        match self.value {
            KdlValue::Integer(i) => Ok(*i),
            KdlValue::Float(f) => {
                if f.is_finite() && f.fract() == 0.0 {
                    Ok(*f as i128)
                } else {
                    Err(Error::TypeMismatch {
                        expected: "integer (float has fractional part or is non-finite)",
                        got: format!("{f}"),
                    })
                }
            }
            other => Err(Error::TypeMismatch {
                expected: "integer",
                got: format!("{other:?}"),
            }),
        }
    }

    fn to_f64(&self) -> Result<f64> {
        match self.value {
            KdlValue::Float(f) => Ok(*f),
            KdlValue::Integer(i) => Ok(*i as f64),
            other => Err(Error::TypeMismatch {
                expected: "float",
                got: format!("{other:?}"),
            }),
        }
    }
}

macro_rules! deserialize_integer {
    ($method:ident, $visit:ident, $ty:ty) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let i = self.to_i128()?;
            let val: $ty = i.try_into().map_err(|_| Error::IntegerOutOfRange(i))?;
            visitor.$visit(val)
        }
    };
}

impl<'de, 'a> de::Deserializer<'de> for ValueDeserializer<'a> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            KdlValue::String(s) => visitor.visit_str(s),
            // visit_i128 is incompatible with serde's Content buffer
            // (used by #[serde(untagged)]), so the Integer branch can't
            // be reached through untagged enums. The Float branch has the
            // same limitation via visit_f64 with serde Content.
            KdlValue::Integer(i) => visitor.visit_i128(*i),
            KdlValue::Float(f) => visitor.visit_f64(*f),
            KdlValue::Bool(b) => visitor.visit_bool(*b),
            KdlValue::Null => visitor.visit_none(), // cov-excl-line
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            KdlValue::Bool(b) => visitor.visit_bool(*b),
            other => Err(Error::TypeMismatch {
                expected: "bool",
                got: format!("{other:?}"),
            }),
        }
    }

    deserialize_integer!(deserialize_i8, visit_i8, i8);
    deserialize_integer!(deserialize_i16, visit_i16, i16);
    deserialize_integer!(deserialize_i32, visit_i32, i32);
    deserialize_integer!(deserialize_i64, visit_i64, i64);
    deserialize_integer!(deserialize_u8, visit_u8, u8);
    deserialize_integer!(deserialize_u16, visit_u16, u16);
    deserialize_integer!(deserialize_u32, visit_u32, u32);
    deserialize_integer!(deserialize_u64, visit_u64, u64);

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i128(self.to_i128()?)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let i = self.to_i128()?;
        let val: u128 = i.try_into().map_err(|_| Error::IntegerOutOfRange(i))?;
        visitor.visit_u128(val)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(self.to_f64()? as f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.to_f64()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            KdlValue::String(s) if s.chars().count() == 1 => {
                visitor.visit_char(s.chars().next().unwrap())
            }
            other => Err(Error::TypeMismatch {
                expected: "single character",
                got: format!("{other:?}"),
            }),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            KdlValue::String(s) => visitor.visit_str(s),
            other => Err(Error::TypeMismatch {
                expected: "string",
                got: format!("{other:?}"),
            }),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            KdlValue::String(s) => visitor.visit_bytes(s.as_bytes()),
            other => Err(Error::TypeMismatch {
                expected: "string (for bytes)",
                got: format!("{other:?}"),
            }),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.value.is_null() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.value.is_null() {
            visitor.visit_unit()
        } else {
            Err(Error::TypeMismatch {
                expected: "null",
                got: format!("{:?}", self.value),
            })
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    // cov-excl-start — a scalar KDL value can't be a sequence, tuple,
    // map, or struct. These error paths exist for serde trait completeness.
    // FieldDeserializer and NodeContentDeserializer handle these types
    // before reaching ValueDeserializer.
    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::TypeMismatch {
            expected: "sequence",
            got: format!("scalar value {:?}", self.value),
        })
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::TypeMismatch {
            expected: "map",
            got: format!("scalar value {:?}", self.value),
        })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::TypeMismatch {
            expected: "struct",
            got: format!("scalar value {:?}", self.value),
        })
    }
    // cov-excl-stop

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        // For a bare value, it must be a string naming a unit variant
        match self.value {
            KdlValue::String(s) => visitor.visit_enum(s.as_str().into_deserializer()),
            other => Err(Error::TypeMismatch {
                expected: "string (enum variant name)",
                got: format!("{other:?}"),
            }),
        }
    }

    // cov-excl-start — serde uses deserialize_identifier for map keys
    // and enum variant names, which route through MapAccess/EnumAccess,
    // not through ValueDeserializer directly.
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }
    // cov-excl-stop
}

// ---------------------------------------------------------------------------
// Enum deserialization
// ---------------------------------------------------------------------------

/// For unit and newtype variants where the variant name is a string argument.
/// e.g., `color "Red"` or `wrapper "SomeVariant" 42`
struct EnumUnitAccess<'a> {
    variant: &'a str,
    node: &'a KdlNode,
    arg_offset: usize,
}

impl<'de, 'a> de::EnumAccess<'de> for EnumUnitAccess<'a> {
    type Error = Error;
    type Variant = EnumUnitVariantAccess<'a>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let val = seed.deserialize::<serde::de::value::StrDeserializer<'_, Error>>(
            self.variant.into_deserializer(),
        )?; // cov-excl-line — error requires StrDeserializer to fail on a valid string
        Ok((
            val,
            EnumUnitVariantAccess {
                node: self.node,
                arg_offset: self.arg_offset,
            },
        ))
    }
}

struct EnumUnitVariantAccess<'a> {
    node: &'a KdlNode,
    arg_offset: usize,
}

impl<'de, 'a> de::VariantAccess<'de> for EnumUnitVariantAccess<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let args = node_args(self.node);
        if self.arg_offset < args.len() {
            seed.deserialize(ValueDeserializer::new(args[self.arg_offset]))
        } else {
            // cov-excl-start — requires a newtype variant with no value
            // after the variant name, e.g. `field "Variant"` targeting
            // `Variant(T)`. Serde resolves this as a unit variant first.
            Err(Error::Message(
                "expected a value after variant name for newtype variant".into(),
            ))
            // cov-excl-stop
        }
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        let args = node_args(self.node);
        let remaining: Vec<&KdlValue> = args[self.arg_offset..].to_vec();
        visitor.visit_seq(ArgsSeqAccess {
            args: remaining,
            index: 0,
        })
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        // Properties of the node are the struct fields
        let props = node_props(self.node);
        visitor.visit_map(PropsMapAccess { props, index: 0 })
    }
}

/// For complex variants where a child node is named after the variant.
/// e.g., `shape { Circle { radius 5.0 } }`
struct EnumComplexAccess<'a> {
    variant_name: &'a str,
    variant_node: &'a KdlNode,
}

impl<'de, 'a> de::EnumAccess<'de> for EnumComplexAccess<'a> {
    type Error = Error;
    type Variant = EnumComplexVariantAccess<'a>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let val = seed.deserialize::<serde::de::value::StrDeserializer<'_, Error>>(
            self.variant_name.into_deserializer(),
        )?; // cov-excl-line — error requires StrDeserializer to fail on a valid string
        Ok((
            val,
            EnumComplexVariantAccess {
                node: self.variant_node,
            },
        ))
    }
}

struct EnumComplexVariantAccess<'a> {
    node: &'a KdlNode,
}

impl<'de, 'a> de::VariantAccess<'de> for EnumComplexVariantAccess<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(NodeContentDeserializer::new(self.node))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        let args = node_args(self.node);
        visitor.visit_seq(ArgsSeqAccess { args, index: 0 })
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        if let Some(children) = self.node.children() {
            visitor.visit_map(DocumentMapAccess::new(children))
        } else {
            let props = node_props(self.node);
            visitor.visit_map(PropsMapAccess { props, index: 0 })
        }
    }
}
