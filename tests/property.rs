use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Helpers ──────────────────────────────────────────────────────────────

/// Strategy for strings that are safe to roundtrip through KDL.
/// Excludes characters that would break KDL parsing.
fn kdl_safe_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _.,:;!?@#%^&*()+=/<>\\[\\]{}|~'-]{0,64}"
}

/// Strategy for f64 values that can roundtrip (no NaN/infinity).
fn finite_f64() -> impl Strategy<Value = f64> {
    prop::num::f64::ANY.prop_filter("finite floats only", |f| f.is_finite())
}

// ── Flat struct roundtrip ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct FlatStruct {
    name: String,
    count: i32,
    enabled: bool,
    ratio: f64,
}

fn flat_struct_strategy() -> impl Strategy<Value = FlatStruct> {
    (kdl_safe_string(), any::<i32>(), any::<bool>(), finite_f64()).prop_map(
        |(name, count, enabled, ratio)| FlatStruct {
            name,
            count,
            enabled,
            ratio,
        },
    )
}

proptest! {
    #[test]
    fn flat_struct_roundtrip(val in flat_struct_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: FlatStruct = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Nested struct roundtrip ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Inner {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Outer {
    label: String,
    inner: Inner,
}

fn outer_strategy() -> impl Strategy<Value = Outer> {
    (kdl_safe_string(), kdl_safe_string(), any::<u16>()).prop_map(|(label, host, port)| Outer {
        label,
        inner: Inner { host, port },
    })
}

proptest! {
    #[test]
    fn nested_struct_roundtrip(val in outer_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: Outer = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Vec of primitives roundtrip ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithVecStrings {
    label: String,
    tags: Vec<String>,
}

fn with_vec_strings_strategy() -> impl Strategy<Value = WithVecStrings> {
    (
        kdl_safe_string(),
        prop::collection::vec(kdl_safe_string(), 0..10),
    )
        .prop_map(|(label, tags)| WithVecStrings { label, tags })
}

proptest! {
    #[test]
    fn vec_strings_roundtrip(val in with_vec_strings_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithVecStrings = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithVecInts {
    label: String,
    numbers: Vec<i64>,
}

fn with_vec_ints_strategy() -> impl Strategy<Value = WithVecInts> {
    (
        kdl_safe_string(),
        prop::collection::vec(any::<i64>(), 0..10),
    )
        .prop_map(|(label, numbers)| WithVecInts { label, numbers })
}

proptest! {
    #[test]
    fn vec_ints_roundtrip(val in with_vec_ints_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithVecInts = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Vec of structs roundtrip ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Item {
    name: String,
    value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithItems {
    title: String,
    item: Vec<Item>,
}

fn with_items_strategy() -> impl Strategy<Value = WithItems> {
    (
        kdl_safe_string(),
        prop::collection::vec(
            (kdl_safe_string(), any::<i32>()).prop_map(|(name, value)| Item { name, value }),
            2..5,
        ),
    )
        .prop_map(|(title, item)| WithItems { title, item })
}

proptest! {
    #[test]
    fn vec_structs_roundtrip(val in with_items_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithItems = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Option fields roundtrip ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
struct WithOptions {
    required: String,
    #[serde(default)]
    maybe_str: Option<String>,
    #[serde(default)]
    maybe_num: Option<i64>,
    #[serde(default)]
    maybe_bool: Option<bool>,
}

fn with_options_strategy() -> impl Strategy<Value = WithOptions> {
    (
        kdl_safe_string(),
        prop::option::of(kdl_safe_string()),
        prop::option::of(any::<i64>()),
        prop::option::of(any::<bool>()),
    )
        .prop_map(|(required, maybe_str, maybe_num, maybe_bool)| WithOptions {
            required,
            maybe_str,
            maybe_num,
            maybe_bool,
        })
}

proptest! {
    #[test]
    fn option_fields_roundtrip(val in with_options_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithOptions = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Enum roundtrip ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithEnum {
    label: String,
    color: Color,
}

fn color_strategy() -> impl Strategy<Value = Color> {
    prop_oneof![Just(Color::Red), Just(Color::Green), Just(Color::Blue),]
}

fn with_enum_strategy() -> impl Strategy<Value = WithEnum> {
    (kdl_safe_string(), color_strategy()).prop_map(|(label, color)| WithEnum { label, color })
}

proptest! {
    #[test]
    fn unit_enum_roundtrip(val in with_enum_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithEnum = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Complex enum roundtrip ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Shape {
    Circle { radius: i32 },
    Rectangle { width: i32, height: i32 },
    Point,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithShape {
    name: String,
    shape: Shape,
}

fn shape_strategy() -> impl Strategy<Value = Shape> {
    prop_oneof![
        any::<i32>().prop_map(|radius| Shape::Circle { radius }),
        (any::<i32>(), any::<i32>()).prop_map(|(width, height)| Shape::Rectangle { width, height }),
        Just(Shape::Point),
    ]
}

fn with_shape_strategy() -> impl Strategy<Value = WithShape> {
    (kdl_safe_string(), shape_strategy()).prop_map(|(name, shape)| WithShape { name, shape })
}

proptest! {
    #[test]
    fn complex_enum_roundtrip(val in with_shape_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithShape = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── BTreeMap roundtrip ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithMap {
    title: String,
    metadata: BTreeMap<String, String>,
}

/// Strategy for map keys that are valid KDL node names (non-empty identifiers).
fn kdl_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,15}"
}

fn with_map_strategy() -> impl Strategy<Value = WithMap> {
    (
        kdl_safe_string(),
        prop::collection::btree_map(kdl_identifier(), kdl_safe_string(), 0..5),
    )
        .prop_map(|(title, metadata)| WithMap { title, metadata })
}

proptest! {
    #[test]
    fn btreemap_roundtrip(val in with_map_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithMap = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Integer types roundtrip ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct IntegerTypes {
    a: i8,
    b: i16,
    c: i32,
    d: i64,
    e: u8,
    f: u16,
    g: u32,
    h: u64,
}

proptest! {
    #[test]
    fn integer_types_roundtrip(
        a in any::<i8>(),
        b in any::<i16>(),
        c in any::<i32>(),
        d in any::<i64>(),
        e in any::<u8>(),
        f in any::<u16>(),
        g in any::<u32>(),
        h in any::<u64>(),
    ) {
        let val = IntegerTypes { a, b, c, d, e, f, g, h };
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: IntegerTypes = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Bool roundtrip ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Flags {
    a: bool,
    b: bool,
    c: bool,
}

proptest! {
    #[test]
    fn bool_roundtrip(a in any::<bool>(), b in any::<bool>(), c in any::<bool>()) {
        let val = Flags { a, b, c };
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: Flags = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Tuple roundtrip ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithTuple {
    label: String,
    pair: (i32, i32),
}

proptest! {
    #[test]
    fn tuple_roundtrip(label in kdl_safe_string(), a in any::<i32>(), b in any::<i32>()) {
        let val = WithTuple { label, pair: (a, b) };
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithTuple = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Deeply nested roundtrip ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Level3 {
    value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Level2 {
    tag: String,
    level3: Level3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Level1 {
    name: String,
    level2: Level2,
}

proptest! {
    #[test]
    fn deeply_nested_roundtrip(
        name in kdl_safe_string(),
        tag in kdl_safe_string(),
        value in any::<i32>(),
    ) {
        let val = Level1 {
            name,
            level2: Level2 {
                tag,
                level3: Level3 { value },
            },
        };
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: Level1 = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Serialize then deserialize never panics ──────────────────────────────

/// Ensure serialization of arbitrary valid structs never panics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct KitchenSink {
    s: String,
    i: i64,
    b: bool,
    f: f64,
    tags: Vec<String>,
    #[serde(default)]
    opt: Option<i32>,
}

fn kitchen_sink_strategy() -> impl Strategy<Value = KitchenSink> {
    (
        kdl_safe_string(),
        any::<i64>(),
        any::<bool>(),
        finite_f64(),
        prop::collection::vec(kdl_safe_string(), 0..5),
        prop::option::of(any::<i32>()),
    )
        .prop_map(|(s, i, b, f, tags, opt)| KitchenSink {
            s,
            i,
            b,
            f,
            tags,
            opt,
        })
}

proptest! {
    #[test]
    fn kitchen_sink_roundtrip(val in kitchen_sink_strategy()) {
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: KitchenSink = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Pretty printing roundtrip ────────────────────────────────────────────

proptest! {
    #[test]
    fn pretty_print_roundtrip(val in flat_struct_strategy()) {
        let serialized = serde_kdl::to_string_pretty(&val).unwrap();
        let deserialized: FlatStruct = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}

// ── Newtype enum variant roundtrip ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Wrapper {
    Text(String),
    Number(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithNewtype {
    label: String,
    wrapped: Wrapper,
}

fn wrapper_strategy() -> impl Strategy<Value = Wrapper> {
    prop_oneof![
        kdl_safe_string().prop_map(Wrapper::Text),
        any::<i64>().prop_map(Wrapper::Number),
    ]
}

proptest! {
    #[test]
    fn newtype_enum_roundtrip(label in kdl_safe_string(), wrapped in wrapper_strategy()) {
        let val = WithNewtype { label, wrapped };
        let serialized = serde_kdl::to_string(&val).unwrap();
        let deserialized: WithNewtype = serde_kdl::from_str(&serialized).unwrap();
        prop_assert_eq!(val, deserialized);
    }
}
