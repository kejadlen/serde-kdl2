use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Basic struct ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SimpleConfig {
    title: String,
    count: i32,
    enabled: bool,
    ratio: f64,
}

#[test]
fn deserialize_simple_struct() {
    let input = r#"
title "My App"
count 42
enabled #true
ratio 3.14
"#;
    let config: SimpleConfig = serde_kdl::from_str(input).unwrap();
    assert_eq!(config.title, "My App");
    assert_eq!(config.count, 42);
    assert_eq!(config.enabled, true);
    assert_eq!(config.ratio, 3.14);
}

#[test]
fn serialize_simple_struct() {
    let config = SimpleConfig {
        title: "My App".into(),
        count: 42,
        enabled: true,
        ratio: 3.14,
    };
    let output = serde_kdl::to_string(&config).unwrap();
    let roundtrip: SimpleConfig = serde_kdl::from_str(&output).unwrap();
    assert_eq!(config, roundtrip);
}

// ── Nested struct ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Server {
    host: String,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct AppConfig {
    name: String,
    server: Server,
}

#[test]
fn deserialize_nested_struct() {
    let input = r#"
name "webapp"
server {
    host "localhost"
    port 8080
}
"#;
    let config: AppConfig = serde_kdl::from_str(input).unwrap();
    assert_eq!(config.name, "webapp");
    assert_eq!(config.server.host, "localhost");
    assert_eq!(config.server.port, 8080);
}

#[test]
fn serialize_nested_struct() {
    let config = AppConfig {
        name: "webapp".into(),
        server: Server {
            host: "localhost".into(),
            port: 8080,
        },
    };
    let output = serde_kdl::to_string(&config).unwrap();
    let roundtrip: AppConfig = serde_kdl::from_str(&output).unwrap();
    assert_eq!(config, roundtrip);
}

// ── Vec of primitives ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Tagged {
    name: String,
    tags: Vec<String>,
}

#[test]
fn deserialize_vec_primitives() {
    let input = r#"
name "project"
tags "web" "rust" "config"
"#;
    let tagged: Tagged = serde_kdl::from_str(input).unwrap();
    assert_eq!(tagged.name, "project");
    assert_eq!(tagged.tags, vec!["web", "rust", "config"]);
}

#[test]
fn serialize_vec_primitives() {
    let tagged = Tagged {
        name: "project".into(),
        tags: vec!["web".into(), "rust".into(), "config".into()],
    };
    let output = serde_kdl::to_string(&tagged).unwrap();
    let roundtrip: Tagged = serde_kdl::from_str(&output).unwrap();
    assert_eq!(tagged, roundtrip);
}

// ── Vec of structs (repeated nodes) ────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Cluster {
    server: Vec<Server>,
}

#[test]
fn deserialize_vec_structs() {
    let input = r#"
server {
    host "localhost"
    port 8080
}
server {
    host "example.com"
    port 443
}
"#;
    let cluster: Cluster = serde_kdl::from_str(input).unwrap();
    assert_eq!(cluster.server.len(), 2);
    assert_eq!(cluster.server[0].host, "localhost");
    assert_eq!(cluster.server[0].port, 8080);
    assert_eq!(cluster.server[1].host, "example.com");
    assert_eq!(cluster.server[1].port, 443);
}

#[test]
fn serialize_vec_structs() {
    let cluster = Cluster {
        server: vec![
            Server {
                host: "localhost".into(),
                port: 8080,
            },
            Server {
                host: "example.com".into(),
                port: 443,
            },
        ],
    };
    let output = serde_kdl::to_string(&cluster).unwrap();
    let roundtrip: Cluster = serde_kdl::from_str(&output).unwrap();
    assert_eq!(cluster, roundtrip);
}

// ── Dash children convention ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DashList {
    items: Vec<i32>,
}

#[test]
fn deserialize_dash_children() {
    let input = r#"
items {
    - 1
    - 2
    - 3
}
"#;
    let list: DashList = serde_kdl::from_str(input).unwrap();
    assert_eq!(list.items, vec![1, 2, 3]);
}

// ── Option fields ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct OptionalFields {
    required: String,
    optional: Option<String>,
}

#[test]
fn deserialize_option_present() {
    let input = r#"
required "hello"
optional "world"
"#;
    let val: OptionalFields = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.required, "hello");
    assert_eq!(val.optional, Some("world".into()));
}

#[test]
fn deserialize_option_absent() {
    let input = r#"
required "hello"
"#;
    let val: OptionalFields = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.required, "hello");
    assert_eq!(val.optional, None);
}

#[test]
fn deserialize_option_null() {
    let input = r#"
required "hello"
optional #null
"#;
    let val: OptionalFields = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.required, "hello");
    assert_eq!(val.optional, None);
}

#[test]
fn serialize_option() {
    let with = OptionalFields {
        required: "hello".into(),
        optional: Some("world".into()),
    };
    let output = serde_kdl::to_string(&with).unwrap();
    assert!(output.contains("optional"));
    let roundtrip: OptionalFields = serde_kdl::from_str(&output).unwrap();
    assert_eq!(with, roundtrip);

    let without = OptionalFields {
        required: "hello".into(),
        optional: None,
    };
    let output = serde_kdl::to_string(&without).unwrap();
    // None fields should be omitted
    assert!(!output.contains("optional"));
    let roundtrip: OptionalFields = serde_kdl::from_str(&output).unwrap();
    assert_eq!(without, roundtrip);
}

// ── Enum variants ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Colored {
    name: String,
    color: Color,
}

#[test]
fn deserialize_unit_variant() {
    let input = r#"
name "widget"
color "Red"
"#;
    let val: Colored = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.color, Color::Red);
}

#[test]
fn serialize_unit_variant() {
    let val = Colored {
        name: "widget".into(),
        color: Color::Green,
    };
    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: Colored = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── Struct variant enum ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Drawing {
    name: String,
    shape: Shape,
}

#[test]
fn deserialize_struct_variant() {
    let input = r#"
name "my drawing"
shape {
    Circle {
        radius 5.0
    }
}
"#;
    let val: Drawing = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.name, "my drawing");
    assert_eq!(val.shape, Shape::Circle { radius: 5.0 });
}

#[test]
fn serialize_struct_variant() {
    let val = Drawing {
        name: "my drawing".into(),
        shape: Shape::Rectangle {
            width: 10.0,
            height: 20.0,
        },
    };
    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: Drawing = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── Newtype variant enum ───────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Wrapper {
    Text(String),
    Number(i64),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wrapped {
    value: Wrapper,
}

#[test]
fn deserialize_newtype_variant() {
    let input = r#"
value {
    Text "hello"
}
"#;
    let val: Wrapped = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.value, Wrapper::Text("hello".into()));
}

#[test]
fn serialize_newtype_variant() {
    let val = Wrapped {
        value: Wrapper::Text("hello".into()),
    };
    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: Wrapped = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── HashMap ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WithMap {
    settings: HashMap<String, String>,
}

#[test]
fn deserialize_hashmap() {
    let input = r#"
settings {
    key1 "value1"
    key2 "value2"
}
"#;
    let val: WithMap = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.settings.get("key1"), Some(&"value1".into()));
    assert_eq!(val.settings.get("key2"), Some(&"value2".into()));
}

#[test]
fn serialize_hashmap() {
    let mut settings = HashMap::new();
    settings.insert("key1".into(), "value1".into());
    let val = WithMap { settings };
    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: WithMap = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── Various integer types ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct IntTypes {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: i8,
    f: i16,
    g: i32,
    h: i64,
}

#[test]
fn roundtrip_int_types() {
    let val = IntTypes {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: -1,
        f: -2,
        g: -3,
        h: -4,
    };
    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: IntTypes = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── Tuple ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WithTuple {
    point: (f64, f64, f64),
}

#[test]
fn deserialize_tuple() {
    let input = r#"
point 1.0 2.0 3.0
"#;
    let val: WithTuple = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.point, (1.0, 2.0, 3.0));
}

#[test]
fn roundtrip_tuple() {
    let val = WithTuple {
        point: (1.0, 2.0, 3.0),
    };
    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: WithTuple = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── Deeply nested ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Level3 {
    value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Level2 {
    inner: Level3,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Level1 {
    middle: Level2,
}

#[test]
fn deeply_nested() {
    let input = r#"
middle {
    inner {
        value "deep"
    }
}
"#;
    let val: Level1 = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.middle.inner.value, "deep");

    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: Level1 = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── Boolean values ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Booleans {
    yes: bool,
    no: bool,
}

#[test]
fn booleans() {
    let input = r#"
yes #true
no #false
"#;
    let val: Booleans = serde_kdl::from_str(input).unwrap();
    assert_eq!(val.yes, true);
    assert_eq!(val.no, false);

    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: Booleans = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── Empty vec ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WithVec {
    items: Vec<String>,
}

#[test]
fn serialize_empty_vec() {
    let val = WithVec { items: vec![] };
    let output = serde_kdl::to_string(&val).unwrap();
    // Empty vec gets an empty children block
    let roundtrip: WithVec = serde_kdl::from_str(&output).unwrap();
    assert_eq!(roundtrip.items, Vec::<String>::new());
}

// ── to_string_pretty ───────────────────────────────────────────────────

#[test]
fn pretty_print() {
    let config = AppConfig {
        name: "webapp".into(),
        server: Server {
            host: "localhost".into(),
            port: 8080,
        },
    };
    let pretty = serde_kdl::to_string_pretty(&config).unwrap();
    // Should still roundtrip
    let roundtrip: AppConfig = serde_kdl::from_str(&pretty).unwrap();
    assert_eq!(config, roundtrip);
}

// ── to_doc / from_doc ──────────────────────────────────────────────────

#[test]
fn doc_api() {
    let config = SimpleConfig {
        title: "Test".into(),
        count: 1,
        enabled: false,
        ratio: 0.5,
    };
    let doc = serde_kdl::to_doc(&config).unwrap();
    assert!(doc.get("title").is_some());
    let roundtrip: SimpleConfig = serde_kdl::from_doc(&doc).unwrap();
    assert_eq!(config, roundtrip);
}

// ── Vec of integers ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Numbers {
    values: Vec<i32>,
}

#[test]
fn roundtrip_vec_ints() {
    let val = Numbers {
        values: vec![1, 2, 3, 4, 5],
    };
    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: Numbers = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}

// ── Char field ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WithChar {
    letter: char,
}

#[test]
fn roundtrip_char() {
    let val = WithChar { letter: 'X' };
    let output = serde_kdl::to_string(&val).unwrap();
    let roundtrip: WithChar = serde_kdl::from_str(&output).unwrap();
    assert_eq!(val, roundtrip);
}
