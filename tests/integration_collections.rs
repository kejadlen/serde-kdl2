//! Integration tests for collection types (Vec, HashMap, etc.).

use indoc::indoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Test macros
macro_rules! deser_ok {
    ($name:ident, $ty:ty, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let val: $ty = serde_kdl2::from_str($input).unwrap();
            assert_eq!(val, $expected);
        }
    };
}

macro_rules! roundtrip {
    ($name:ident, $ty:ty, $val:expr) => {
        #[test]
        fn $name() {
            let val: $ty = $val;
            let output = serde_kdl2::to_string(&val).unwrap();
            let rt: $ty = serde_kdl2::from_str(&output).unwrap();
            assert_eq!(val, rt);
        }
    };
}

macro_rules! deser_field {
    ($name:ident, $field:ident: $ty:ty, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            #[derive(Debug, Deserialize, PartialEq)]
            struct W {
                $field: $ty,
            }
            let val: W = serde_kdl2::from_str($input).unwrap();
            assert_eq!(val.$field, $expected);
        }
    };
}

// Shared types
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Server {
    host: String,
    port: u16,
}

// ── Vec of primitives ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Tagged {
    name: String,
    tags: Vec<String>,
}

deser_ok!(
    deserialize_vec_primitives,
    Tagged,
    indoc! {r#"
        name "project"
        tags "web" "rust" "config"
    "#},
    Tagged {
        name: "project".into(),
        tags: vec!["web".into(), "rust".into(), "config".into()],
    }
);

roundtrip!(
    serialize_vec_primitives,
    Tagged,
    Tagged {
        name: "project".into(),
        tags: vec!["web".into(), "rust".into(), "config".into()],
    }
);

// ── Vec of structs ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Cluster {
    server: Vec<Server>,
}

deser_ok!(
    deserialize_vec_structs,
    Cluster,
    indoc! {r#"
        server {
            host "localhost"
            port 8080
        }
        server {
            host "example.com"
            port 443
        }
    "#},
    Cluster {
        server: vec![
            Server {
                host: "localhost".into(),
                port: 8080
            },
            Server {
                host: "example.com".into(),
                port: 443
            },
        ],
    }
);

roundtrip!(
    serialize_vec_structs,
    Cluster,
    Cluster {
        server: vec![
            Server {
                host: "localhost".into(),
                port: 8080
            },
            Server {
                host: "example.com".into(),
                port: 443
            },
        ],
    }
);

// ── Single-element vec of structs ──────────────────────────────────────

deser_ok!(
    deserialize_single_element_vec_structs,
    Cluster,
    indoc! {r#"
        server {
            host "localhost"
            port 8080
        }
    "#},
    Cluster {
        server: vec![Server {
            host: "localhost".into(),
            port: 8080,
        }],
    }
);

roundtrip!(
    roundtrip_single_element_vec_structs,
    Cluster,
    Cluster {
        server: vec![Server {
            host: "localhost".into(),
            port: 8080,
        }],
    }
);

// ── Dash children ──────────────────────────────────────────────────────

deser_field!(
    deserialize_dash_children,
    items: Vec<i32>,
    indoc! {"
        items {
            - 1
            - 2
            - 3
        }
    "},
    vec![1, 2, 3]
);

// ── HashMap ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WithMap {
    settings: HashMap<String, String>,
}

#[test]
fn deserialize_hashmap() {
    let val: WithMap = serde_kdl2::from_str(indoc! {r#"
        settings {
            key1 "value1"
            key2 "value2"
        }
    "#})
    .unwrap();
    assert_eq!(val.settings.get("key1"), Some(&"value1".into()));
    assert_eq!(val.settings.get("key2"), Some(&"value2".into()));
}

#[test]
fn serialize_hashmap() {
    let mut settings = HashMap::new();
    settings.insert("key1".into(), "value1".into());
    let val = WithMap { settings };
    let output = serde_kdl2::to_string(&val).unwrap();
    let rt: WithMap = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(val, rt);
}

// ── Empty vec ──────────────────────────────────────────────────────────

#[test]
fn serialize_empty_vec() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct W {
        items: Vec<String>,
    }
    let val = W { items: vec![] };
    let output = serde_kdl2::to_string(&val).unwrap();
    let rt: W = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(rt.items, Vec::<String>::new());
}
