use indoc::indoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ════════════════════════════════════════════════════════════════════════
// Macros for data-driven tests
// ════════════════════════════════════════════════════════════════════════

/// Test that deserializing KDL input produces the expected value.
macro_rules! deser_ok {
    ($name:ident, $ty:ty, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let val: $ty = serde_kdl2::from_str($input).unwrap();
            assert_eq!(val, $expected);
        }
    };
}

/// Test that deserializing KDL input into `T` fails.
macro_rules! deser_err {
    ($name:ident, $ty:ty, $input:expr) => {
        #[test]
        fn $name() {
            assert!(serde_kdl2::from_str::<$ty>($input).is_err());
        }
    };
}

/// Test that a value roundtrips through serialize → deserialize.
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

/// Test that serializing a value fails.
macro_rules! ser_err {
    ($name:ident, $val:expr) => {
        #[test]
        fn $name() {
            assert!(serde_kdl2::to_string(&$val).is_err());
        }
    };
}

/// Test deserializing repeated nodes into Vec<T>.
/// Defines a wrapper struct with the given field name internally.
macro_rules! deser_repeated_vec {
    ($name:ident, $field:ident: Vec<$ty:ty>, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            #[derive(Debug, Deserialize, PartialEq)]
            struct W {
                $field: Vec<$ty>,
            }
            let val: W = serde_kdl2::from_str($input).unwrap();
            assert_eq!(val.$field, $expected);
        }
    };
}

/// Test that deserializing repeated nodes into Vec<T> fails.
macro_rules! deser_repeated_vec_err {
    ($name:ident, $field:ident: Vec<$ty:ty>, $input:expr) => {
        #[test]
        fn $name() {
            #[derive(Debug, Deserialize, PartialEq)]
            struct W {
                $field: Vec<$ty>,
            }
            assert!(serde_kdl2::from_str::<W>($input).is_err());
        }
    };
}

/// Test deserializing into a wrapper with a single typed field.
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

/// Test that deserializing into a wrapper with a single typed field fails.
macro_rules! deser_field_err {
    ($name:ident, $field:ident: $ty:ty, $input:expr) => {
        #[test]
        fn $name() {
            #[derive(Debug, Deserialize)]
            struct W {
                #[allow(dead_code)]
                $field: $ty,
            }
            assert!(serde_kdl2::from_str::<W>($input).is_err());
        }
    };
}

// ════════════════════════════════════════════════════════════════════════
// Shared types
// ════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SimpleConfig {
    title: String,
    count: i32,
    enabled: bool,
    ratio: f64,
}

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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UnitStruct;

// ════════════════════════════════════════════════════════════════════════
// Basic feature tests
// ════════════════════════════════════════════════════════════════════════

deser_ok!(
    deserialize_simple_struct,
    SimpleConfig,
    indoc! {r#"
        title "My App"
        count 42
        enabled #true
        ratio 3.125
    "#},
    SimpleConfig {
        title: "My App".into(),
        count: 42,
        enabled: true,
        ratio: 3.125
    }
);

roundtrip!(
    serialize_simple_struct,
    SimpleConfig,
    SimpleConfig {
        title: "My App".into(),
        count: 42,
        enabled: true,
        ratio: 3.125,
    }
);

deser_ok!(
    deserialize_nested_struct,
    AppConfig,
    indoc! {r#"
        name "webapp"
        server {
            host "localhost"
            port 8080
        }
    "#},
    AppConfig {
        name: "webapp".into(),
        server: Server {
            host: "localhost".into(),
            port: 8080,
        },
    }
);

roundtrip!(
    serialize_nested_struct,
    AppConfig,
    AppConfig {
        name: "webapp".into(),
        server: Server {
            host: "localhost".into(),
            port: 8080,
        },
    }
);

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

// ── Option fields ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct OptionalFields {
    required: String,
    optional: Option<String>,
}

deser_ok!(
    deserialize_option_present,
    OptionalFields,
    indoc! {r#"
        required "hello"
        optional "world"
    "#},
    OptionalFields {
        required: String::from("hello"),
        optional: Some(String::from("world"))
    }
);

deser_ok!(
    deserialize_option_absent,
    OptionalFields,
    r#"required "hello""#,
    OptionalFields {
        required: String::from("hello"),
        optional: None
    }
);

deser_ok!(
    deserialize_option_null,
    OptionalFields,
    indoc! {r#"
        required "hello"
        optional #null
    "#},
    OptionalFields {
        required: String::from("hello"),
        optional: None
    }
);

#[test]
fn serialize_option() {
    let with = OptionalFields {
        required: String::from("hello"),
        optional: Some(String::from("world")),
    };
    let output = serde_kdl2::to_string(&with).unwrap();
    assert!(output.contains("optional"));
    let rt: OptionalFields = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(with, rt);

    let without = OptionalFields {
        required: String::from("hello"),
        optional: None,
    };
    let output = serde_kdl2::to_string(&without).unwrap();
    assert!(!output.contains("optional"));
    let rt: OptionalFields = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(without, rt);
}

// ── Enum variants ──────────────────────────────────────────────────────

deser_ok!(
    deserialize_unit_variant,
    Colored,
    indoc! {r#"
        name "widget"
        color "Red"
    "#},
    Colored {
        name: "widget".into(),
        color: Color::Red
    }
);

roundtrip!(
    serialize_unit_variant,
    Colored,
    Colored {
        name: "widget".into(),
        color: Color::Green
    }
);

// ── Struct variant enum ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Drawing {
    name: String,
    shape: Shape,
}

deser_ok!(
    deserialize_struct_variant,
    Drawing,
    indoc! {r#"
        name "my drawing"
        shape {
            Circle {
                radius 5.0
            }
        }
    "#},
    Drawing {
        name: "my drawing".into(),
        shape: Shape::Circle { radius: 5.0 }
    }
);

roundtrip!(
    serialize_struct_variant,
    Drawing,
    Drawing {
        name: "my drawing".into(),
        shape: Shape::Rectangle {
            width: 10.0,
            height: 20.0
        },
    }
);

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

deser_ok!(
    deserialize_newtype_variant,
    Wrapped,
    indoc! {r#"
        value {
            Text "hello"
        }
    "#},
    Wrapped {
        value: Wrapper::Text(String::from("hello"))
    }
);

roundtrip!(
    serialize_newtype_variant,
    Wrapped,
    Wrapped {
        value: Wrapper::Text(String::from("hello"))
    }
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

roundtrip!(
    roundtrip_int_types,
    IntTypes,
    IntTypes {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: -1,
        f: -2,
        g: -3,
        h: -4
    }
);

// ── Tuple ──────────────────────────────────────────────────────────────

deser_field!(deserialize_tuple, point: (f64, f64, f64), "point 1.0 2.0 3.0", (1.0, 2.0, 3.0));

#[test]
fn roundtrip_tuple() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct W {
        point: (f64, f64, f64),
    }
    let val = W {
        point: (1.0, 2.0, 3.0),
    };
    let output = serde_kdl2::to_string(&val).unwrap();
    let rt: W = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(val, rt);
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
    let input = indoc! {r#"
        middle {
            inner {
                value "deep"
            }
        }
    "#};
    let val: Level1 = serde_kdl2::from_str(input).unwrap();
    assert_eq!(val.middle.inner.value, "deep");
    let output = serde_kdl2::to_string(&val).unwrap();
    let rt: Level1 = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(val, rt);
}

// ── Booleans ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Booleans {
    yes: bool,
    no: bool,
}

deser_ok!(
    booleans_deser,
    Booleans,
    indoc! {"
        yes #true
        no #false
    "},
    Booleans {
        yes: true,
        no: false
    }
);
roundtrip!(
    booleans_roundtrip,
    Booleans,
    Booleans {
        yes: true,
        no: false
    }
);

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
    let pretty = serde_kdl2::to_string_pretty(&config).unwrap();
    let rt: AppConfig = serde_kdl2::from_str(&pretty).unwrap();
    assert_eq!(config, rt);
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
    let doc = serde_kdl2::to_doc(&config).unwrap();
    assert!(doc.get("title").is_some());
    let rt: SimpleConfig = serde_kdl2::from_doc(&doc).unwrap();
    assert_eq!(config, rt);
}

// ════════════════════════════════════════════════════════════════════════
// Roundtrip tests — data-driven
// ════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WVi {
    values: Vec<i32>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WCh {
    letter: char,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WBi {
    big_signed: i128,
    big_unsigned: u128,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WF32 {
    value: f32,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WI128 {
    value: i128,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WU128 {
    value: u128,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Meters(f64);
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WNt {
    length: Meters,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Point3D(f64, f64, f64);
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WTs {
    pos: Point3D,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Data {
    Point(f64, f64, f64),
    Pair(String, i32),
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WData {
    data: Data,
}

roundtrip!(
    roundtrip_vec_ints,
    WVi,
    WVi {
        values: vec![1, 2, 3, 4, 5]
    }
);
roundtrip!(roundtrip_char, WCh, WCh { letter: 'X' });
roundtrip!(roundtrip_char_multibyte, WCh, WCh { letter: 'é' });
roundtrip!(roundtrip_char_cjk, WCh, WCh { letter: '中' });
roundtrip!(roundtrip_char_emoji, WCh, WCh { letter: '🦀' });
roundtrip!(
    roundtrip_i128_u128,
    WBi,
    WBi {
        big_signed: -1_000_000_000_000,
        big_unsigned: 1_000_000_000_000
    }
);
roundtrip!(roundtrip_f32, WF32, WF32 { value: 3.125 });
roundtrip!(
    roundtrip_i128,
    WI128,
    WI128 {
        value: 170_141_183_460_469_231_731_687_303_715_884_105_727i128
    }
);
roundtrip!(roundtrip_u128, WU128, WU128 { value: 1000u128 });
roundtrip!(
    roundtrip_newtype_struct,
    WNt,
    WNt {
        length: Meters(42.5)
    }
);
roundtrip!(
    roundtrip_tuple_struct,
    WTs,
    WTs {
        pos: Point3D(1.0, 2.0, 3.0)
    }
);
roundtrip!(
    roundtrip_tuple_variant,
    WData,
    WData {
        data: Data::Point(1.0, 2.0, 3.0)
    }
);
roundtrip!(
    roundtrip_tuple_variant_pair,
    WData,
    WData {
        data: Data::Pair(String::from("hello"), 42)
    }
);

// ════════════════════════════════════════════════════════════════════════
// Serialization-specific tests
// ════════════════════════════════════════════════════════════════════════

ser_err!(serialize_top_level_not_struct, 42i32);
ser_err!(serialize_top_level_string_err, "hello");
ser_err!(serialize_top_level_bool_err, true);

#[test]
fn serialize_u128_overflow() {
    let val = WU128 { value: u128::MAX };
    let err = serde_kdl2::to_string(&val).unwrap_err();
    assert!(matches!(err, serde_kdl2::Error::IntegerOutOfRange(_)));
}

#[test]
fn serialize_top_level_not_struct_error_type() {
    let err = serde_kdl2::to_string(&42i32).unwrap_err();
    assert!(matches!(err, serde_kdl2::Error::TopLevelNotStruct));
}

#[test]
fn serialize_bytes() {
    mod serde_bytes_helper {
        use serde::{Deserializer, Serializer};
        pub fn serialize<S: Serializer>(data: &[u8], ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_bytes(data)
        }
        pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<u8>, D::Error> {
            use serde::Deserialize;
            Vec::<u8>::deserialize(de)
        }
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct W {
        #[serde(with = "serde_bytes_helper")]
        data: Vec<u8>,
    }
    let val = W {
        data: vec![1, 2, 3],
    };
    let output = serde_kdl2::to_string(&val).unwrap();
    let rt: W = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(val, rt);
}

#[test]
fn roundtrip_unit_field() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        label: String,
        marker: (),
    }
    let val = S {
        label: String::from("test"),
        marker: (),
    };
    let output = serde_kdl2::to_string(&val).unwrap();
    assert!(output.contains("label"));
    assert!(output.contains("marker"));
    let rt: S = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(val, rt);
}

#[test]
fn roundtrip_unit_struct_field() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        label: String,
        marker: UnitStruct,
    }
    let val = S {
        label: String::from("test"),
        marker: UnitStruct,
    };
    let output = serde_kdl2::to_string(&val).unwrap();
    assert!(output.contains("marker"));
    let rt: S = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(val, rt);
}

#[test]
fn serialize_f32_field() {
    let output = serde_kdl2::to_string(&WF32 { value: 2.5 }).unwrap();
    assert!(output.contains("2.5"));
}

#[test]
fn serialize_map_integer_keys() {
    #[derive(Debug, Serialize)]
    struct S {
        lookup: HashMap<i32, String>,
    }
    let mut lookup = HashMap::new();
    lookup.insert(1, "one".into());
    let output = serde_kdl2::to_string(&S { lookup }).unwrap();
    assert!(output.contains("1"));
}

#[test]
fn serialize_map_bool_keys() {
    #[derive(Debug, Serialize)]
    struct S {
        flags: HashMap<bool, String>,
    }
    let mut flags = HashMap::new();
    flags.insert(true, "yes".into());
    let output = serde_kdl2::to_string(&S { flags }).unwrap();
    assert!(output.contains("true"));
}

#[test]
fn serialize_mixed_sequence() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(untagged)]
    enum V {
        Num(i64),
        Str(String),
    }
    #[derive(Debug, Serialize)]
    struct S {
        items: Vec<V>,
    }
    let output = serde_kdl2::to_string(&S {
        items: vec![V::Num(1), V::Num(2)],
    })
    .unwrap();
    assert!(output.contains("items"));
}

#[test]
fn serialize_mixed_primitive_sequence() {
    #[derive(Serialize, Debug)]
    #[serde(untagged)]
    enum Mixed {
        Int(i32),
        Str(String),
    }
    #[derive(Serialize, Debug)]
    struct S {
        items: Vec<Mixed>,
    }
    let output = serde_kdl2::to_string(&S {
        items: vec![Mixed::Int(1), Mixed::Str("two".into()), Mixed::Int(3)],
    })
    .unwrap();
    assert!(output.contains("items"));
}

#[test]
fn serialize_nested_sequence() {
    #[derive(Serialize, Debug)]
    struct S {
        matrix: Vec<Vec<i32>>,
    }
    let output = serde_kdl2::to_string(&S {
        matrix: vec![vec![1, 2], vec![3, 4]],
    })
    .unwrap();
    assert!(output.contains("-"));
}

#[test]
fn serialize_vec_bools() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct S {
        flags: Vec<bool>,
    }
    let val = S {
        flags: vec![true, false, true],
    };
    let output = serde_kdl2::to_string(&val).unwrap();
    let rt: S = serde_kdl2::from_str(&output).unwrap();
    assert_eq!(val, rt);
}

#[test]
fn serialize_vec_option_with_nulls() {
    #[derive(Serialize, Debug)]
    struct S {
        vals: Vec<Option<i32>>,
    }
    let output = serde_kdl2::to_string(&S {
        vals: vec![Some(1), None, Some(3)],
    })
    .unwrap();
    assert!(output.contains("#null"));
}

#[test]
fn serialize_mixed_seq_with_null() {
    #[derive(Serialize, Debug)]
    struct S {
        items: Vec<Option<Vec<i32>>>,
    }
    let output = serde_kdl2::to_string(&S {
        items: vec![Some(vec![1, 2]), None, Some(vec![3])],
    })
    .unwrap();
    assert!(output.contains("#null"));
}

#[test]
fn serialize_option_some_null_nested() {
    #[derive(Serialize)]
    struct S {
        items: HashMap<String, Option<String>>,
    }
    let mut items = HashMap::new();
    items.insert("present".into(), Some("value".into()));
    items.insert("absent".into(), None);
    let output = serde_kdl2::to_string(&S { items }).unwrap();
    assert!(!output.contains("absent"));
    assert!(output.contains("present"));
}

#[test]
fn serialize_unsupported_map_key() {
    let err = serde_kdl2::Error::Unsupported("map key must be a string, got Null".into());
    assert!(err.to_string().contains("map key"));
}

// ════════════════════════════════════════════════════════════════════════
// Error trait tests
// ════════════════════════════════════════════════════════════════════════

#[test]
fn error_display_variants() {
    assert_eq!(
        serde_kdl2::Error::TopLevelNotStruct.to_string(),
        "top-level type must be a struct or map"
    );
    assert_eq!(
        serde_kdl2::Error::Message("custom error".into()).to_string(),
        "custom error"
    );
    assert!(
        serde_kdl2::Error::TypeMismatch {
            expected: "string",
            got: "integer".into()
        }
        .to_string()
        .contains("expected string")
    );
    assert!(
        serde_kdl2::Error::MissingField("name".into())
            .to_string()
            .contains("name")
    );
    assert!(
        serde_kdl2::Error::IntegerOutOfRange(999999)
            .to_string()
            .contains("999999")
    );
    assert!(
        serde_kdl2::Error::UnknownVariant("Foo".into())
            .to_string()
            .contains("Foo")
    );
    assert!(
        serde_kdl2::Error::Unsupported("nope".into())
            .to_string()
            .contains("nope")
    );
}

#[test]
fn serde_error_custom_impls() {
    assert_eq!(
        <serde_kdl2::Error as serde::de::Error>::custom("deser fail").to_string(),
        "deser fail"
    );
    assert_eq!(
        <serde_kdl2::Error as serde::ser::Error>::custom("ser fail").to_string(),
        "ser fail"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Deserialization error tests — data-driven
// ════════════════════════════════════════════════════════════════════════

deser_err!(deserialize_invalid_kdl, SimpleConfig, "{{{{invalid");
deser_field_err!(
    deserialize_duplicate_scalar_node,
    name: String,
    indoc! {r#"
        name "first"
        name "second"
    "#}
);
deser_field_err!(deserialize_bool_type_mismatch, flag: bool, r#"flag "not a bool""#);
deser_field_err!(deserialize_string_type_mismatch, name: String, "name 42");
deser_field_err!(deserialize_int_type_mismatch, value: i32, r#"value "not a number""#);
deser_field_err!(deserialize_float_type_mismatch, value: f64, r#"value "not a float""#);
deser_field_err!(deserialize_char_type_mismatch, ch: char, r#"ch "abc""#);
deser_field_err!(deserialize_char_from_int_mismatch, ch: char, "ch 65");
deser_field_err!(deserialize_integer_overflow, value: i8, "value 999");
deser_field_err!(deserialize_node_no_args, value: String, "value");
deser_err!(deserialize_u128_overflow, WU128, "value -1");
deser_field_err!(deserialize_enum_no_match, color: Color, "color 42");
deser_field_err!(deserialize_enum_non_string, color: Color, "color 42");
deser_repeated_vec_err!(node_content_enum_error_in_seq, color: Vec<Color>, "color 42\n");
deser_field_err!(value_deserializer_unit_mismatch_in_args_err, vals: Vec<()>, "vals 42");

#[test]
fn field_deserialize_enum_multi_children_error() {
    #[derive(Deserialize, Debug)]
    struct S {
        #[allow(dead_code)]
        shape: Shape,
    }
    let input = indoc! {"
        shape {
            Circle {
                radius 5.0
            }
            Rectangle {
                width 10.0
            }
        }
    "};
    assert!(serde_kdl2::from_str::<S>(input).is_err());
}

// ════════════════════════════════════════════════════════════════════════
// Deserialization: single-field tests — data-driven
// ════════════════════════════════════════════════════════════════════════

deser_field!(deserialize_f32_from_integer, value: f32, "value 3", 3.0f32);
deser_field!(deserialize_f64_from_integer, value: f64, "value 42", 42.0f64);
deser_field!(deserialize_int_from_float, count: i32, "count 3.0", 3);
deser_field_err!(deserialize_int_from_fractional_float, count: i32, "count 3.7");
deser_field_err!(deserialize_int_from_negative_frac, count: i32, "count -1.5");
deser_field!(value_deserializer_any_integer, val: i128, "val 42", 42i128);
deser_field!(value_deserializer_any_bool, val: bool, "val #true", true);
deser_field!(value_deserializer_null_option, optional: Option<i32>, "optional #null", None);
deser_field!(value_deserializer_any_null, val: Option<String>, "val #null", None);

#[test]
fn value_deserializer_any_float() {
    #[derive(Deserialize, Debug)]
    struct S {
        val: f64,
    }
    let val: S = serde_kdl2::from_str("val 3.125").unwrap();
    assert!((val.val - 3.125).abs() < 0.001);
}

#[test]
fn value_deserializer_unit_null() {
    #[derive(Deserialize, Debug)]
    struct S {
        #[allow(dead_code)]
        marker: (),
        name: String,
    }
    let val: S = serde_kdl2::from_str(indoc! {r#"
        marker #null
        name "test"
    "#})
    .unwrap();
    assert_eq!(val.name, "test");
}

#[test]
fn value_deserializer_unit_mismatch() {
    #[derive(Deserialize, Debug)]
    struct S {
        marker: (),
    }
    let val: S = serde_kdl2::from_str("marker 42").unwrap();
    assert_eq!(val.marker, ());
}

deser_field!(value_deserializer_newtype_struct, val: (i32,), "val 42", (42,));

// Actually, newtype struct requires a named wrapper:
#[test]
fn value_deserializer_newtype_struct_named() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct W(i32);
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        val: W,
    }
    let val: S = serde_kdl2::from_str("val 42").unwrap();
    assert_eq!(val.val, W(42));
}

// ════════════════════════════════════════════════════════════════════════
// FieldDeserializer::deserialize_any branches — data-driven via untagged
// ════════════════════════════════════════════════════════════════════════

#[test]
fn deserialize_any_with_properties() {
    #[derive(Deserialize, Debug)]
    struct S {
        item: HashMap<String, String>,
    }
    let val: S = serde_kdl2::from_str(r#"item key="value""#).unwrap();
    assert_eq!(val.item.get("key"), Some(&"value".into()));
}

deser_field!(deserialize_any_with_multiple_args, values: Vec<i32>, "values 1 2 3", vec![1, 2, 3]);

#[test]
fn deserialize_any_unit_node() {
    #[derive(Deserialize, Debug)]
    struct S {
        #[allow(dead_code)]
        marker: (),
        name: String,
    }
    let val: S = serde_kdl2::from_str(indoc! {r#"
        marker
        name "test"
    "#})
    .unwrap();
    assert_eq!(val.name, "test");
}

// ── untagged enum branches ─────────────────────────────────────────────

macro_rules! test_untagged_any {
    ($name:ident, $variant_ty:ty, $input:expr, $check:expr) => {
        #[test]
        fn $name() {
            #[derive(Deserialize, Debug, PartialEq)]
            #[serde(untagged)]
            enum DynVal {
                V($variant_ty),
            }
            #[derive(Deserialize, Debug)]
            struct S {
                data: DynVal,
            }
            let val: S = serde_kdl2::from_str($input).unwrap();
            let DynVal::V(inner) = val.data;
            let check: $variant_ty = $check;
            assert_eq!(inner, check);
        }
    };
}

test_untagged_any!(
    field_deserialize_any_single_arg,
    String,
    r#"data "hello""#,
    String::from("hello")
);
test_untagged_any!(
    field_deserialize_any_multi_arg,
    Vec<String>,
    r#"data "a" "b" "c""#,
    vec!["a".into(), "b".into(), "c".into()]
);
test_untagged_any!(
    field_deserialize_any_children,
    HashMap<String, String>,
    indoc! {r#"
        data {
            key "value"
        }
    "#},
    {
        let mut m = HashMap::new();
        m.insert("key".into(), "value".into());
        m
    }
);
test_untagged_any!(
    field_deserialize_any_props,
    HashMap<String, String>,
    r#"data key="value""#,
    {
        let mut m = HashMap::new();
        m.insert("key".into(), "value".into());
        m
    }
);

#[test]
fn field_deserialize_any_no_args() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum DynVal {
        Unit,
    }
    #[derive(Deserialize, Debug)]
    struct S {
        data: DynVal,
    }
    let val: S = serde_kdl2::from_str("data").unwrap();
    assert_eq!(val.data, DynVal::Unit);
}

#[test]
fn field_deserialize_any_float() {
    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum DynVal {
        Float(f64),
    }
    #[derive(Deserialize, Debug)]
    struct S {
        data: DynVal,
    }
    let val: S = serde_kdl2::from_str("data 3.125").unwrap();
    let DynVal::Float(f) = val.data;
    assert!((f - 3.125).abs() < 0.001);
}

#[test]
fn field_deserialize_any_bool() {
    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum DynVal {
        Bool(bool),
    }
    #[derive(Deserialize, Debug)]
    struct S {
        data: DynVal,
    }
    let val: S = serde_kdl2::from_str("data #true").unwrap();
    let DynVal::Bool(b) = val.data;
    assert!(b);
}

#[test]
fn field_deserialize_any_null() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum DynVal {
        Nothing,
        Str(String),
    }
    #[derive(Deserialize, Debug)]
    struct S {
        data: Option<DynVal>,
    }
    let val: S = serde_kdl2::from_str("data #null").unwrap();
    assert_eq!(val.data, None);
}

#[test]
fn field_deserialize_any_integer_limitation() {
    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum DynVal {
        #[allow(dead_code)]
        Num(i64),
    }
    #[derive(Deserialize, Debug)]
    struct S {
        #[allow(dead_code)]
        data: DynVal,
    }
    assert!(serde_kdl2::from_str::<S>("data 42").is_err());
}

// ════════════════════════════════════════════════════════════════════════
// DocumentDeserializer extra paths
// ════════════════════════════════════════════════════════════════════════

#[test]
fn document_deserialize_any_as_map() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum TopLevel {
        Config { name: String, label: String },
    }
    let val: TopLevel = serde_kdl2::from_str(indoc! {r#"
        name "test"
        label "hello"
    "#})
    .unwrap();
    assert_eq!(
        val,
        TopLevel::Config {
            name: String::from("test"),
            label: String::from("hello")
        }
    );
}

#[test]
fn document_deserialize_unit() {
    let _: () = serde_kdl2::from_str("").unwrap();
}

#[test]
fn document_deserialize_unit_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Empty;
    let _: Empty = serde_kdl2::from_str("").unwrap();
}

#[test]
fn document_deserialize_newtype_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Inner {
        name: String,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct Outer(Inner);
    let val: Outer = serde_kdl2::from_str(r#"name "test""#).unwrap();
    assert_eq!(val.0.name, "test");
}

// ── Extra field handling (ignored_any) ─────────────────────────────────

deser_field!(
    deserialize_with_extra_fields,
    name: String,
    indoc! {r#"
        name "test"
        extra "ignored"
        another 42
    "#},
    String::from("test")
);

deser_field!(
    document_deserialize_ignored_any,
    name: String,
    indoc! {r#"
        name "test"
        unknown "ignored"
    "#},
    String::from("test")
);

deser_field!(
    field_ignored_any_with_children,
    name: String,
    indoc! {r#"
        name "test"
        complex {
            nested "value"
            deep {
                x 1
            }
        }
    "#},
    String::from("test")
);

// ════════════════════════════════════════════════════════════════════════
// Properties-based struct/map deserialization — data-driven
// ════════════════════════════════════════════════════════════════════════

#[test]
fn deserialize_struct_from_properties() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        point: Point,
    }
    let val: S = serde_kdl2::from_str(r#"point x=1.0 y=2.0"#).unwrap();
    assert_eq!(val.point, Point { x: 1.0, y: 2.0 });
}

#[test]
fn deserialize_map_from_properties() {
    #[derive(Deserialize, Debug)]
    struct S {
        meta: HashMap<String, String>,
    }
    let val: S = serde_kdl2::from_str(r#"meta author="Alice" version="1.0""#).unwrap();
    assert_eq!(val.meta.get("author"), Some(&String::from("Alice")));
    assert_eq!(val.meta.get("version"), Some(&"1.0".into()));
}

deser_field!(
    deserialize_empty_map_from_node,
    meta: HashMap<String, String>,
    "meta",
    HashMap::new()
);

// ── Non-dash children as sequence ──────────────────────────────────────

deser_field!(
    deserialize_children_as_sequence,
    items: Vec<i32>,
    indoc! {"
        items {
            item 1
            item 2
            item 3
        }
    "},
    vec![1, 2, 3]
);

// ════════════════════════════════════════════════════════════════════════
// FieldDeserializer misc paths — data-driven
// ════════════════════════════════════════════════════════════════════════

#[test]
fn field_deserialize_unit_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Marker;
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        tag: Marker,
    }
    let val: S = serde_kdl2::from_str("tag").unwrap();
    assert_eq!(val.tag, Marker);
}

#[test]
fn field_deserialize_newtype_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Inner {
        x: i32,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct W(Inner);
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        data: W,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        data {
            x 42
        }
    "})
    .unwrap();
    assert_eq!(val.data, W(Inner { x: 42 }));
}

#[test]
fn field_deserialize_struct_from_properties() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        pos: Point,
    }
    let val: S = serde_kdl2::from_str("pos x=1.0 y=2.0").unwrap();
    assert_eq!(val.pos, Point { x: 1.0, y: 2.0 });
}

#[test]
fn field_deserialize_struct_single_arg() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct W {
        value: i32,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        item: W,
    }
    let val: S = serde_kdl2::from_str("item 42").unwrap();
    assert_eq!(val.item, W { value: 42 });
}

#[test]
fn field_deserialize_struct_empty() {
    #[derive(Deserialize, Debug, PartialEq, Default)]
    struct Empty {
        #[serde(default)]
        a: Option<i32>,
        #[serde(default)]
        b: Option<String>,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        data: Empty,
    }
    let val: S = serde_kdl2::from_str("data").unwrap();
    assert_eq!(val.data, Empty { a: None, b: None });
}

deser_field!(
    field_deserializer_bytes_as_seq,
    data: Vec<u8>,
    "data 72 101 108",
    vec![72u8, 101, 108]
);

deser_ok!(
    deserialize_identifier_field,
    Colored,
    indoc! {r#"
        name "widget"
        color "Blue"
    "#},
    Colored {
        name: "widget".into(),
        color: Color::Blue
    }
);

// ════════════════════════════════════════════════════════════════════════
// Enum access paths — data-driven
// ════════════════════════════════════════════════════════════════════════

#[test]
fn enum_newtype_variant_via_arg() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Val {
        Number(i64),
        Text(String),
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        value: Val,
    }
    let val: S = serde_kdl2::from_str(r#"value "Number" 42"#).unwrap();
    assert_eq!(val.value, Val::Number(42));
}

#[test]
fn enum_tuple_variant_via_args() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Val {
        Point(f64, f64),
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        value: Val,
    }
    let val: S = serde_kdl2::from_str(r#"value "Point" 1.0 2.0"#).unwrap();
    assert_eq!(val.value, Val::Point(1.0, 2.0));
}

#[test]
fn enum_struct_variant_via_props() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Val {
        Circle { radius: f64 },
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        value: Val,
    }
    let val: S = serde_kdl2::from_str(r#"value "Circle" radius=5.0"#).unwrap();
    assert_eq!(val.value, Val::Circle { radius: 5.0 });
}

#[test]
fn enum_complex_unit_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Status {
        Active,
        Inactive,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        status: Status,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        status {
            Active
        }
    "})
    .unwrap();
    assert_eq!(val.status, Status::Active);
}

#[test]
fn enum_complex_tuple_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Val {
        Point(f64, f64, f64),
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        data: Val,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        data {
            Point 1.0 2.0 3.0
        }
    "})
    .unwrap();
    assert_eq!(val.data, Val::Point(1.0, 2.0, 3.0));
}

#[test]
fn enum_complex_struct_variant_from_props() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Val {
        Circle { radius: f64 },
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        shape: Val,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        shape {
            Circle radius=5.0
        }
    "})
    .unwrap();
    assert_eq!(val.shape, Val::Circle { radius: 5.0 });
}

// ════════════════════════════════════════════════════════════════════════
// Repeated-node Vec<T> tests — data-driven
//
// NodeContentDeserializer is exercised when repeated nodes are
// deserialized into Vec<T>. Each entry tests a different element type.
// ════════════════════════════════════════════════════════════════════════

deser_repeated_vec!(
    node_content_bool_in_seq,
    flag: Vec<bool>,
    indoc! {"
        flag #true
        flag #false
    "},
    vec![true, false]
);

deser_repeated_vec!(
    node_content_i8_in_seq,
    val: Vec<i8>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1i8, 2]
);

deser_repeated_vec!(
    node_content_i16_in_seq,
    val: Vec<i16>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1i16, 2]
);

deser_repeated_vec!(
    node_content_i32_in_seq,
    val: Vec<i32>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1i32, 2]
);

deser_repeated_vec!(
    node_content_i64_in_seq,
    val: Vec<i64>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1i64, 2]
);

deser_repeated_vec!(
    node_content_i128_in_seq,
    val: Vec<i128>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1i128, 2]
);

deser_repeated_vec!(
    node_content_u8_in_seq,
    val: Vec<u8>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1u8, 2]
);

deser_repeated_vec!(
    node_content_u16_in_seq,
    val: Vec<u16>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1u16, 2]
);

deser_repeated_vec!(
    node_content_u32_in_seq,
    val: Vec<u32>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1u32, 2]
);

deser_repeated_vec!(
    node_content_u64_in_seq,
    val: Vec<u64>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1u64, 2]
);

deser_repeated_vec!(
    node_content_u128_in_seq,
    val: Vec<u128>,
    indoc! {"
        val 1
        val 2
    "},
    vec![1u128, 2]
);

deser_repeated_vec!(
    node_content_f32_in_seq,
    val: Vec<f32>,
    indoc! {"
        val 1.5
        val 2.5
    "},
    vec![1.5f32, 2.5]
);

deser_repeated_vec!(
    node_content_f64_in_seq,
    val: Vec<f64>,
    indoc! {"
        val 1.5
        val 2.5
    "},
    vec![1.5f64, 2.5]
);

deser_repeated_vec!(
    node_content_char_in_seq,
    ch: Vec<char>,
    indoc! {r#"
        ch "A"
        ch "B"
    "#},
    vec!['A', 'B']
);

deser_repeated_vec!(
    node_content_string_in_seq,
    name: Vec<String>,
    indoc! {r#"
        name "Alice"
        name "Bob"
    "#},
    vec![String::from("Alice"), String::from("Bob")]
);

deser_repeated_vec!(
    node_content_unit_in_seq,
    marker: Vec<()>,
    indoc! {"
        marker
        marker
    "},
    vec![(), ()]
);

deser_repeated_vec!(
    node_content_option_in_seq,
    val: Vec<Option<i32>>,
    indoc! {"
        val 1
        val #null
        val 3
    "},
    vec![Some(1), None, Some(3)]
);

deser_repeated_vec!(
    node_content_tuple_in_seq,
    coords: Vec<(f64, f64)>,
    indoc! {"
        coords 1.0 2.0
        coords 3.0 4.0
    "},
    vec![(1.0, 2.0), (3.0, 4.0)]
);

deser_repeated_vec!(
    node_content_enum_in_seq,
    color: Vec<Color>,
    indoc! {r#"
        color "Red"
        color "Blue"
    "#},
    vec![Color::Red, Color::Blue]
);

deser_repeated_vec!(
    node_content_multi_arg_as_seq,
    coords: Vec<Vec<f64>>,
    indoc! {"
        coords 1.0 2.0 3.0
        coords 4.0 5.0 6.0
    "},
    vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]
);

deser_repeated_vec!(
    node_content_bytes_in_seq,
    data: Vec<Vec<u8>>,
    indoc! {"
        data 1 2 3
        data 4 5 6
    "},
    vec![vec![1u8, 2, 3], vec![4, 5, 6]]
);

deser_repeated_vec!(
    node_content_dash_children_in_seq,
    group: Vec<Vec<i32>>,
    indoc! {"
        group {
            - 1
            - 2
        }
        group {
            - 3
            - 4
        }
    "},
    vec![vec![1, 2], vec![3, 4]]
);

deser_repeated_vec!(
    node_content_non_dash_children_seq,
    group: Vec<Vec<i32>>,
    indoc! {"
        group {
            item 1
            item 2
        }
        group {
            item 3
        }
    "},
    vec![vec![1, 2], vec![3]]
);

deser_repeated_vec!(
    node_content_seq_empty_children_fallthrough,
    vals: Vec<Vec<i32>>,
    indoc! {"
        vals 1 2 3
        vals 4 5 6
    "},
    vec![vec![1, 2, 3], vec![4, 5, 6]]
);

deser_repeated_vec!(
    node_content_seq_args_fallthrough_empty_children,
    vals: Vec<Vec<i32>>,
    indoc! {"
        vals {
        }
        vals {
        }
    "},
    vec![Vec::<i32>::new(), Vec::<i32>::new()]
);

// ── Newtype/tuple struct in repeated nodes ─────────────────────────────

#[test]
fn node_content_newtype_in_seq() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Meters(f64);
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        dist: Vec<Meters>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        dist 1.0
        dist 2.0
    "})
    .unwrap();
    assert_eq!(val.dist, vec![Meters(1.0), Meters(2.0)]);
}

#[test]
fn node_content_tuple_struct_in_seq() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Pair(f64, f64);
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        pair: Vec<Pair>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        pair 1.0 2.0
        pair 3.0 4.0
    "})
    .unwrap();
    assert_eq!(val.pair, vec![Pair(1.0, 2.0), Pair(3.0, 4.0)]);
}

#[test]
fn node_content_unit_struct_in_seq() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Marker;
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        tag: Vec<Marker>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        tag
        tag
    "})
    .unwrap();
    assert_eq!(val.tag, vec![Marker, Marker]);
}

// ── Struct from properties in repeated nodes ───────────────────────────

#[test]
fn node_content_struct_from_properties_in_seq() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        point: Vec<Point>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        point x=1.0 y=2.0
        point x=3.0 y=4.0
    "})
    .unwrap();
    assert_eq!(
        val.point,
        vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }]
    );
}

#[test]
fn node_content_single_arg_struct_in_seq() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct W {
        value: i32,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        item: Vec<W>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        item 10
        item 20
    "})
    .unwrap();
    assert_eq!(val.item, vec![W { value: 10 }, W { value: 20 }]);
}

#[test]
fn node_content_struct_empty_node_in_seq() {
    #[derive(Deserialize, Debug, PartialEq, Default)]
    struct Item {
        #[serde(default)]
        a: Option<i32>,
        #[serde(default)]
        b: Option<String>,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        item: Vec<Item>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        item
        item
    "})
    .unwrap();
    assert_eq!(
        val.item,
        vec![Item { a: None, b: None }, Item { a: None, b: None }]
    );
}

// ── Map from properties/children in repeated nodes ─────────────────────

#[test]
fn node_content_map_from_properties_in_seq() {
    #[derive(Deserialize, Debug)]
    struct S {
        entry: Vec<HashMap<String, String>>,
    }
    let val: S = serde_kdl2::from_str(indoc! {r#"
        entry a="1" b="2"
        entry c="3"
    "#})
    .unwrap();
    assert_eq!(val.entry.len(), 2);
    assert_eq!(val.entry[0].get("a"), Some(&"1".into()));
}

#[test]
fn node_content_map_from_children() {
    #[derive(Deserialize, Debug)]
    struct S {
        entry: Vec<HashMap<String, String>>,
    }
    let val: S = serde_kdl2::from_str(indoc! {r#"
        entry {
            a "1"
            b "2"
        }
        entry {
            c "3"
        }
    "#})
    .unwrap();
    assert_eq!(val.entry[0].get("a"), Some(&"1".into()));
    assert_eq!(val.entry[1].get("c"), Some(&"3".into()));
}

#[test]
fn node_content_map_from_props_no_children() {
    #[derive(Deserialize, Debug)]
    struct S {
        entry: Vec<HashMap<String, String>>,
    }
    let val: S = serde_kdl2::from_str(indoc! {r#"
        entry a="1" b="2"
        entry c="3"
    "#})
    .unwrap();
    assert_eq!(val.entry[0].get("a"), Some(&"1".into()));
    assert_eq!(val.entry[1].get("c"), Some(&"3".into()));
}

// ── Complex enum in repeated nodes ─────────────────────────────────────

#[test]
fn node_content_complex_enum_in_seq() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        shape: Vec<Shape>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        shape {
            Circle {
                radius 5.0
            }
        }
        shape {
            Rectangle {
                width 10.0
                height 20.0
            }
        }
    "})
    .unwrap();
    assert_eq!(val.shape[0], Shape::Circle { radius: 5.0 });
    assert_eq!(
        val.shape[1],
        Shape::Rectangle {
            width: 10.0,
            height: 20.0
        }
    );
}

// ── Extra fields in repeated nodes (ignored_any) ───────────────────────

#[test]
fn node_content_ignored_any() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Item {
        name: String,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        item: Vec<Item>,
    }
    let val: S = serde_kdl2::from_str(indoc! {r#"
        item {
            name "test"
            extra "ignored"
        }
        item {
            name "test2"
            bonus 99
        }
    "#})
    .unwrap();
    assert_eq!(val.item.len(), 2);
    assert_eq!(val.item[0].name, "test");
}

// ── Repeated-node Vec for same-type via MultiNodeSeqAccess ─────────────

#[test]
fn node_content_with_children() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        server: Vec<Server>,
    }
    let val: S = serde_kdl2::from_str(indoc! {r#"
        server {
            host "a"
            port 1
        }
        server {
            host "b"
            port 2
        }
    "#})
    .unwrap();
    assert_eq!(val.server.len(), 2);
}

deser_repeated_vec!(
    node_content_identifier_in_enum_seq,
    color: Vec<Color>,
    indoc! {r#"
        color "Red"
        color "Green"
    "#},
    vec![Color::Red, Color::Green]
);

deser_repeated_vec!(
    node_content_string_in_repeated_nodes,
    val: Vec<String>,
    indoc! {r#"
        val "hello"
        val "world"
    "#},
    vec![String::from("hello"), String::from("world")]
);

// ════════════════════════════════════════════════════════════════════════
// Multi-arg Vec<T> tests — data-driven
//
// ArgsSeqAccess is exercised when a single node has multiple positional
// arguments deserialized into Vec<T>.
// ════════════════════════════════════════════════════════════════════════

deser_field!(
    args_seq_bool_values,
    flags: Vec<bool>,
    "flags #true #false #true",
    vec![true, false, true]
);
deser_field!(
    args_seq_string_values,
    names: Vec<String>,
    r#"names "Alice" "Bob""#,
    vec![String::from("Alice"), String::from("Bob")]
);
deser_field!(
    args_seq_char_values,
    letters: Vec<char>,
    r#"letters "A" "B" "C""#,
    vec!['A', 'B', 'C']
);
deser_field!(
    args_seq_i128_values,
    vals: Vec<i128>,
    "vals 100 200 300",
    vec![100i128, 200, 300]
);
deser_field!(
    args_seq_u128_values,
    vals: Vec<u128>,
    "vals 100 200 300",
    vec![100u128, 200, 300]
);
deser_field!(
    args_seq_f32_values,
    vals: Vec<f32>,
    "vals 1.5 2.5 3.5",
    vec![1.5f32, 2.5, 3.5]
);
deser_field!(
    args_seq_f64_values,
    vals: Vec<f64>,
    "vals 1.5 2.5 3.5",
    vec![1.5f64, 2.5, 3.5]
);
deser_field!(
    args_seq_enum_values,
    colors: Vec<Color>,
    r#"colors "Red" "Blue" "Green""#,
    vec![Color::Red, Color::Blue, Color::Green]
);
deser_field!(
    args_seq_option_with_null,
    vals: Vec<Option<String>>,
    r#"vals "hello" #null "world""#,
    vec![
        Some(String::from("hello")),
        None,
        Some(String::from("world"))
    ]
);
deser_field!(
    value_deserializer_bytes_from_string,
    data: Vec<u8>,
    "data 104 101 108",
    vec![104u8, 101, 108]
);

deser_field!(
    value_deserializer_unit_null_in_args,
    vals: Vec<()>,
    "vals #null #null",
    vec![(), ()]
);

#[test]
fn args_seq_newtype_values() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Meters(f64);
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        dists: Vec<Meters>,
    }
    let val: S = serde_kdl2::from_str("dists 1.0 2.0 3.0").unwrap();
    assert_eq!(val.dists, vec![Meters(1.0), Meters(2.0), Meters(3.0)]);
}

#[test]
fn value_deserializer_unit_struct_in_args() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Marker;
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        vals: Vec<Marker>,
    }
    let val: S = serde_kdl2::from_str("vals #null #null").unwrap();
    assert_eq!(val.vals, vec![Marker, Marker]);
}

// ════════════════════════════════════════════════════════════════════════
// Custom Deserialize impl tests — data-driven via macro
//
// These test code paths reachable only through custom Deserialize impls
// that call specific deserializer methods (deserialize_str,
// deserialize_bytes, deserialize_byte_buf).
// ════════════════════════════════════════════════════════════════════════

/// Generates a test with a custom Deserialize impl calling `$deser_method`
/// and using `visit_str` to produce a String wrapper.
macro_rules! test_custom_str_deser {
    ($name:ident, $deser_method:ident, field_type: $field_ty:ty, $field:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            #[derive(Debug, PartialEq)]
            struct Custom(String);
            impl<'de> Deserialize<'de> for Custom {
                fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                    struct V;
                    impl<'de> serde::de::Visitor<'de> for V {
                        type Value = Custom;
                        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(f, "custom str")
                        }
                        fn visit_str<E>(self, v: &str) -> Result<Custom, E> {
                            Ok(Custom(v.to_string()))
                        }
                    }
                    de.$deser_method(V)
                }
            }
            #[derive(Deserialize, Debug, PartialEq)]
            struct S {
                $field: $field_ty,
            }
            let val: S = serde_kdl2::from_str($input).unwrap();
            assert_eq!(val.$field, $expected);
        }
    };
}

/// Generates a test with a custom Deserialize impl calling `$deser_method`
/// and using `visit_bytes` to produce a Vec<u8> wrapper.
macro_rules! test_custom_bytes_deser {
    ($name:ident, $deser_method:ident, field_type: $field_ty:ty, $field:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            #[derive(Debug, PartialEq)]
            struct Custom(Vec<u8>);
            impl<'de> Deserialize<'de> for Custom {
                fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                    struct V;
                    impl<'de> serde::de::Visitor<'de> for V {
                        type Value = Custom;
                        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(f, "custom bytes")
                        }
                        fn visit_bytes<E>(self, v: &[u8]) -> Result<Custom, E> {
                            Ok(Custom(v.to_vec()))
                        }
                    }
                    de.$deser_method(V)
                }
            }
            #[derive(Deserialize, Debug, PartialEq)]
            struct S {
                $field: $field_ty,
            }
            let val: S = serde_kdl2::from_str($input).unwrap();
            assert_eq!(val.$field, $expected);
        }
    };
}

/// Generates a test with a custom Deserialize impl calling `$deser_method`
/// and using `visit_seq` to produce a Vec<u8> wrapper.
macro_rules! test_custom_seq_bytes_deser {
    ($name:ident, $deser_method:ident, field_type: $field_ty:ty, $field:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            #[derive(Debug, PartialEq)]
            struct Custom(Vec<u8>);
            impl<'de> Deserialize<'de> for Custom {
                fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                    struct V;
                    impl<'de> serde::de::Visitor<'de> for V {
                        type Value = Custom;
                        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(f, "custom seq bytes")
                        }
                        fn visit_seq<A: serde::de::SeqAccess<'de>>(
                            self,
                            mut seq: A,
                        ) -> Result<Custom, A::Error> {
                            let mut v = Vec::new();
                            while let Some(b) = seq.next_element()? {
                                v.push(b);
                            }
                            Ok(Custom(v))
                        }
                    }
                    de.$deser_method(V)
                }
            }
            #[derive(Deserialize, Debug, PartialEq)]
            struct S {
                $field: $field_ty,
            }
            let val: S = serde_kdl2::from_str($input).unwrap();
            assert_eq!(val.$field, $expected);
        }
    };
}

// -- FieldDeserializer paths --

test_custom_str_deser!(
    field_deserializer_str, deserialize_str,
    field_type: Custom, name,
    r#"name "hello""#,
    Custom(String::from("hello"))
);

test_custom_seq_bytes_deser!(
    field_deserializer_byte_buf, deserialize_byte_buf,
    field_type: Custom, data,
    "data 1 2 3",
    Custom(vec![1, 2, 3])
);

// -- NodeContentDeserializer paths (via Vec<Custom> in repeated nodes) --

test_custom_seq_bytes_deser!(
    node_content_bytes_custom_deser_in_seq, deserialize_bytes,
    field_type: Vec<Custom>, data,
    indoc! {"
        data 1 2 3
        data 4 5
    "},
    vec![Custom(vec![1, 2, 3]), Custom(vec![4, 5])]
);

test_custom_seq_bytes_deser!(
    node_content_byte_buf_custom_deser_in_seq, deserialize_byte_buf,
    field_type: Vec<Custom>, data,
    indoc! {"
        data 1 2 3
        data 4 5
    "},
    vec![Custom(vec![1, 2, 3]), Custom(vec![4, 5])]
);

test_custom_str_deser!(
    node_content_str_custom_deser_in_seq, deserialize_str,
    field_type: Vec<Custom>, name,
    indoc! {r#"
        name "hello"
        name "world"
    "#},
    vec![Custom(String::from("hello")), Custom(String::from("world"))]
);

// -- ValueDeserializer paths (via Vec<Custom> in multi-arg nodes) --

test_custom_bytes_deser!(
    value_deserializer_bytes_from_string_seq, deserialize_bytes,
    field_type: Vec<Custom>, data,
    r#"data "hello" "world""#,
    vec![Custom(b"hello".to_vec()), Custom(b"world".to_vec())]
);

test_custom_bytes_deser!(
    value_deserializer_byte_buf_via_seq, deserialize_byte_buf,
    field_type: Vec<Custom>, data,
    r#"data "hello""#,
    vec![Custom(b"hello".to_vec())]
);

// -- ValueDeserializer error: deserialize_bytes on non-string --

#[test]
fn value_deserializer_bytes_type_mismatch_via_seq() {
    #[derive(Debug)]
    struct Custom(#[allow(dead_code)] Vec<u8>);
    impl<'de> Deserialize<'de> for Custom {
        fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct V;
            impl<'de> serde::de::Visitor<'de> for V {
                type Value = Custom;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "bytes")
                }
                fn visit_bytes<E>(self, v: &[u8]) -> Result<Custom, E> {
                    Ok(Custom(v.to_vec()))
                }
            }
            de.deserialize_bytes(V)
        }
    }
    #[derive(Deserialize, Debug)]
    struct S {
        #[allow(dead_code)]
        data: Vec<Custom>,
    }
    assert!(serde_kdl2::from_str::<S>("data 42").is_err());
}

// -- FieldDeserializer error: deserialize_bytes on non-seq --

#[test]
fn value_deserializer_bytes_type_mismatch() {
    #[derive(Debug)]
    struct ByteString(#[allow(dead_code)] Vec<u8>);
    impl<'de> Deserialize<'de> for ByteString {
        fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct V;
            impl<'de> serde::de::Visitor<'de> for V {
                type Value = ByteString;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "bytes")
                }
                fn visit_bytes<E>(self, v: &[u8]) -> Result<ByteString, E> {
                    Ok(ByteString(v.to_vec()))
                }
            }
            de.deserialize_bytes(V)
        }
    }
    #[derive(Deserialize, Debug)]
    struct S {
        #[allow(dead_code)]
        data: ByteString,
    }
    assert!(serde_kdl2::from_str::<S>("data 42").is_err());
}

// ════════════════════════════════════════════════════════════════════════
// Miscellaneous edge case tests
// ════════════════════════════════════════════════════════════════════════

#[test]
fn value_deserializer_seq_error() {
    let err = serde_kdl2::Error::TypeMismatch {
        expected: "sequence",
        got: "scalar value".into(),
    };
    assert!(err.to_string().contains("sequence"));
}

// ════════════════════════════════════════════════════════════════════════
// Mutation testing — targeted tests for surviving mutants
// ════════════════════════════════════════════════════════════════════════

// Kills mutant: de.rs FieldDeserializer::deserialize_seq
// dash filter `==` → `!=`. With mixed dash and non-dash children,
// the mutant would pick non-dash nodes instead of dash nodes.
#[test]
fn field_deserialize_seq_dash_filter_over_non_dash() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        items: Vec<i32>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        items {
            - 10
            - 20
            extra 99
        }
    "})
    .unwrap();
    assert_eq!(val.items, vec![10, 20]);
}

// Kills mutant: de.rs FieldDeserializer::deserialize_struct
// `fields.len() == 1 && args.len() == 1` → `||`. With 2 fields and 1
// argument, `||` would enter SingleArgStructAccess (wrong), while `&&`
// falls through to the empty-map path.
#[test]
fn field_deserialize_struct_multi_field_single_arg_falls_through() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Multi {
        #[serde(default)]
        a: Option<i32>,
        #[serde(default)]
        b: Option<String>,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        data: Multi,
    }
    // 2 fields, 1 arg. With `&&`: false (2 ≠ 1), falls to empty map →
    // both default to None. With `||`: `args.len() == 1` is true →
    // SingleArgStructAccess assigns 42 to "a", producing Some(42).
    let val: S = serde_kdl2::from_str("data 42").unwrap();
    assert_eq!(val.data, Multi { a: None, b: None });
}

// Kills mutant: de.rs NodeContentDeserializer::deserialize_seq
// dash filter `==` → `!=` in repeated-node context.
#[test]
fn node_content_seq_dash_filter_over_non_dash() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        group: Vec<Vec<i32>>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        group {
            - 10
            - 20
            extra 99
        }
        group {
            - 30
        }
    "})
    .unwrap();
    assert_eq!(val.group, vec![vec![10, 20], vec![30]]);
}

// Kills mutant: de.rs NodeContentDeserializer::deserialize_struct
// `fields.len() == 1 && args.len() == 1` → `||` in repeated-node context.
// With 2 fields and 1 arg per node, `||` enters SingleArgStructAccess
// (wrong), while `&&` falls through to empty map.
#[test]
fn node_content_struct_multi_field_single_arg_falls_through() {
    #[derive(Deserialize, Debug, PartialEq, Default)]
    struct Multi {
        #[serde(default)]
        a: Option<i32>,
        #[serde(default)]
        b: Option<String>,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        item: Vec<Multi>,
    }
    let val: S = serde_kdl2::from_str(indoc! {"
        item 42
        item 99
    "})
    .unwrap();
    assert_eq!(
        val.item,
        vec![Multi { a: None, b: None }, Multi { a: None, b: None }]
    );
}

// Kills mutant: de.rs EnumUnitVariantAccess::newtype_variant_seed
// `arg_offset < args.len()` → `arg_offset <= args.len()`. With the
// mutant, accessing args[args.len()] panics instead of returning an error.
#[test]
fn enum_newtype_variant_missing_value_errors() {
    #[derive(Deserialize, Debug)]
    enum Val {
        #[allow(dead_code)]
        Wrap(String),
    }
    #[derive(Deserialize, Debug)]
    struct S {
        #[allow(dead_code)]
        value: Val,
    }
    // "Wrap" is the only arg → arg_offset=1, args.len()=1.
    // With `<`: 1 < 1 is false → returns error.
    // With `<=`: 1 <= 1 is true → tries args[1] → panic.
    let result = serde_kdl2::from_str::<S>(r#"value "Wrap""#);
    assert!(result.is_err());
}
