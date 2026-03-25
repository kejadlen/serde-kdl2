use hegel::TestCase;
use hegel::generators::{
    Generator, booleans, floats, from_regex, integers, optional, sampled_from, text, vecs,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Helpers ──────────────────────────────────────────────────────────────

/// Generator for f64 values that can roundtrip through KDL.
///
/// NaN is excluded because `NaN != NaN`, so roundtrip equality assertions
/// always fail. Infinity is excluded because KDL has no infinity literal —
/// the serializer would need to encode it as a string, which changes the type.
fn finite_f64() -> impl Generator<f64> {
    floats::<f64>().allow_nan(false).allow_infinity(false)
}

/// Generator for valid KDL node-name identifiers (non-empty, starts with a letter).
fn kdl_identifier() -> impl Generator<String> {
    from_regex("[a-zA-Z][a-zA-Z0-9_-]{0,15}").fullmatch(true)
}

// ── Flat struct roundtrip ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct FlatStruct {
    name: String,
    count: i32,
    enabled: bool,
    ratio: f64,
}

#[hegel::test]
fn flat_struct_roundtrip(tc: TestCase) {
    let val = FlatStruct {
        name: tc.draw(text()),
        count: tc.draw(integers()),
        enabled: tc.draw(booleans()),
        ratio: tc.draw(finite_f64()),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: FlatStruct = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn nested_struct_roundtrip(tc: TestCase) {
    let val = Outer {
        label: tc.draw(text()),
        inner: Inner {
            host: tc.draw(text()),
            port: tc.draw(integers()),
        },
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: Outer = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── Vec of primitives roundtrip ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithVecStrings {
    label: String,
    tags: Vec<String>,
}

#[hegel::test]
fn vec_strings_roundtrip(tc: TestCase) {
    let val = WithVecStrings {
        label: tc.draw(text()),
        tags: tc.draw(vecs(text())),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithVecStrings = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithVecInts {
    label: String,
    numbers: Vec<i64>,
}

#[hegel::test]
fn vec_ints_roundtrip(tc: TestCase) {
    let val = WithVecInts {
        label: tc.draw(text()),
        numbers: tc.draw(vecs(integers::<i64>())),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithVecInts = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn vec_structs_roundtrip(tc: TestCase) {
    let count = tc.draw(integers::<usize>().max_value(10));
    let mut items = Vec::new();
    for _ in 0..count {
        items.push(Item {
            name: tc.draw(text()),
            value: tc.draw(integers()),
        });
    }
    let val = WithItems {
        title: tc.draw(text()),
        item: items,
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithItems = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn option_fields_roundtrip(tc: TestCase) {
    let val = WithOptions {
        required: tc.draw(text()),
        maybe_str: tc.draw(optional(text())),
        maybe_num: tc.draw(optional(integers::<i64>())),
        maybe_bool: tc.draw(optional(booleans())),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithOptions = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn unit_enum_roundtrip(tc: TestCase) {
    let val = WithEnum {
        label: tc.draw(text()),
        color: tc.draw(sampled_from(vec![Color::Red, Color::Green, Color::Blue])),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithEnum = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn complex_enum_roundtrip(tc: TestCase) {
    let variant = tc.draw(integers::<u8>().min_value(0).max_value(2));
    let shape = match variant {
        0 => Shape::Circle {
            radius: tc.draw(integers()),
        },
        1 => Shape::Rectangle {
            width: tc.draw(integers()),
            height: tc.draw(integers()),
        },
        _ => Shape::Point,
    };
    let val = WithShape {
        name: tc.draw(text()),
        shape,
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithShape = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── BTreeMap roundtrip ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithMap {
    title: String,
    metadata: BTreeMap<String, String>,
}

#[hegel::test]
fn btreemap_roundtrip(tc: TestCase) {
    let keys = tc.draw(vecs(kdl_identifier()).unique(true));
    let mut metadata = BTreeMap::new();
    for key in keys {
        metadata.insert(key, tc.draw(text()));
    }
    let val = WithMap {
        title: tc.draw(text()),
        metadata,
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithMap = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn integer_types_roundtrip(tc: TestCase) {
    let val = IntegerTypes {
        a: tc.draw(integers()),
        b: tc.draw(integers()),
        c: tc.draw(integers()),
        d: tc.draw(integers()),
        e: tc.draw(integers()),
        f: tc.draw(integers()),
        g: tc.draw(integers()),
        h: tc.draw(integers()),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: IntegerTypes = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── Bool roundtrip ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Flags {
    a: bool,
    b: bool,
    c: bool,
}

#[hegel::test]
fn bool_roundtrip(tc: TestCase) {
    let val = Flags {
        a: tc.draw(booleans()),
        b: tc.draw(booleans()),
        c: tc.draw(booleans()),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: Flags = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── Tuple roundtrip ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithTuple {
    label: String,
    pair: (i32, i32),
}

#[hegel::test]
fn tuple_roundtrip(tc: TestCase) {
    let val = WithTuple {
        label: tc.draw(text()),
        pair: (tc.draw(integers()), tc.draw(integers())),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithTuple = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn deeply_nested_roundtrip(tc: TestCase) {
    let val = Level1 {
        name: tc.draw(text()),
        level2: Level2 {
            tag: tc.draw(text()),
            level3: Level3 {
                value: tc.draw(integers()),
            },
        },
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: Level1 = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn kitchen_sink_roundtrip(tc: TestCase) {
    let val = KitchenSink {
        s: tc.draw(text()),
        i: tc.draw(integers()),
        b: tc.draw(booleans()),
        f: tc.draw(finite_f64()),
        tags: tc.draw(vecs(text())),
        opt: tc.draw(optional(integers::<i32>())),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: KitchenSink = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── Pretty printing roundtrip ────────────────────────────────────────────

#[hegel::test]
fn pretty_print_roundtrip(tc: TestCase) {
    let val = FlatStruct {
        name: tc.draw(text()),
        count: tc.draw(integers()),
        enabled: tc.draw(booleans()),
        ratio: tc.draw(finite_f64()),
    };
    let serialized = serde_kdl2::to_string_pretty(&val).unwrap();
    let deserialized: FlatStruct = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
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

#[hegel::test]
fn newtype_enum_roundtrip(tc: TestCase) {
    let variant = tc.draw(booleans());
    let wrapped = if variant {
        Wrapper::Text(tc.draw(text()))
    } else {
        Wrapper::Number(tc.draw(integers()))
    };
    let val = WithNewtype {
        label: tc.draw(text()),
        wrapped,
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithNewtype = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── Parse robustness ─────────────────────────────────────────────────────

/// Parsing arbitrary text must never panic — it should return Ok or Err.
#[hegel::test]
fn from_str_never_panics(tc: TestCase) {
    let input: String = tc.draw(text());
    let _ = serde_kdl2::from_str::<FlatStruct>(&input);
}

// ── Consistency: to_string vs to_string_pretty ───────────────────────────

/// Both serialization paths must produce output that deserializes to the
/// same value.
#[hegel::test]
fn to_string_and_pretty_agree(tc: TestCase) {
    let val = FlatStruct {
        name: tc.draw(text()),
        count: tc.draw(integers()),
        enabled: tc.draw(booleans()),
        ratio: tc.draw(finite_f64()),
    };
    let compact = serde_kdl2::to_string(&val).unwrap();
    let pretty = serde_kdl2::to_string_pretty(&val).unwrap();
    let from_compact: FlatStruct = serde_kdl2::from_str(&compact).unwrap();
    let from_pretty: FlatStruct = serde_kdl2::from_str(&pretty).unwrap();
    assert_eq!(from_compact, from_pretty);
}

// ── Consistency: to_doc/from_doc vs to_string/from_str ───────────────────

/// The doc API and the string API must agree.
#[hegel::test]
fn doc_api_matches_string_api(tc: TestCase) {
    let val = FlatStruct {
        name: tc.draw(text()),
        count: tc.draw(integers()),
        enabled: tc.draw(booleans()),
        ratio: tc.draw(finite_f64()),
    };
    let from_string: FlatStruct =
        serde_kdl2::from_str(&serde_kdl2::to_string(&val).unwrap()).unwrap();
    let from_doc: FlatStruct = serde_kdl2::from_doc(&serde_kdl2::to_doc(&val).unwrap()).unwrap();
    assert_eq!(from_string, from_doc);
}

// ── Char roundtrip ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithChar {
    label: String,
    letter: char,
}

#[hegel::test]
fn char_roundtrip(tc: TestCase) {
    let val = WithChar {
        label: tc.draw(text()),
        letter: tc.draw(
            integers::<u32>()
                .min_value(0x20)
                .max_value(0x10FFFF)
                .filter(|&cp| !(0xD800..=0xDFFF).contains(&cp))
                .map(|cp| char::from_u32(cp).unwrap()),
        ),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithChar = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── i128/u128 roundtrip ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Wide {
    signed: i128,
    unsigned: u128,
}

/// i128 and u128 roundtrip within the range that KDL can represent.
///
/// The kdl crate's parser can't roundtrip `i128::MIN` — it parses the
/// sign and magnitude separately, and the magnitude of `i128::MIN`
/// (`i128::MAX + 1`) overflows during parsing. u128 values above
/// `i128::MAX` overflow the crate's i128 storage.
#[hegel::test]
fn i128_u128_roundtrip(tc: TestCase) {
    let val = Wide {
        signed: tc.draw(integers::<i128>().min_value(i128::MIN + 1)),
        unsigned: tc.draw(integers::<u128>().max_value(i128::MAX as u128)),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: Wide = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── f32 roundtrip ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithF32 {
    value: f32,
}

#[hegel::test]
fn f32_roundtrip(tc: TestCase) {
    let val = WithF32 {
        value: tc.draw(floats::<f32>().allow_nan(false).allow_infinity(false)),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithF32 = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── Newtype struct roundtrip ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Meters(f64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithNewtype2 {
    length: Meters,
}

#[hegel::test]
fn newtype_struct_roundtrip(tc: TestCase) {
    let val = WithNewtype2 {
        length: Meters(tc.draw(finite_f64())),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithNewtype2 = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── Tuple struct roundtrip ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Point3D(f64, f64, f64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithTupleStruct {
    pos: Point3D,
}

#[hegel::test]
fn tuple_struct_roundtrip(tc: TestCase) {
    let val = WithTupleStruct {
        pos: Point3D(
            tc.draw(finite_f64()),
            tc.draw(finite_f64()),
            tc.draw(finite_f64()),
        ),
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithTupleStruct = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── Tuple enum variant roundtrip ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Data {
    Point(f64, f64, f64),
    Pair(String, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithData {
    data: Data,
}

#[hegel::test]
fn tuple_enum_variant_roundtrip(tc: TestCase) {
    let variant = tc.draw(booleans());
    let data = if variant {
        Data::Point(
            tc.draw(finite_f64()),
            tc.draw(finite_f64()),
            tc.draw(finite_f64()),
        )
    } else {
        Data::Pair(tc.draw(text()), tc.draw(integers()))
    };
    let val = WithData { data };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithData = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}

// ── HashMap roundtrip ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithHashMap {
    title: String,
    metadata: std::collections::HashMap<String, String>,
}

#[hegel::test]
fn hashmap_roundtrip(tc: TestCase) {
    let keys = tc.draw(vecs(kdl_identifier()).unique(true));
    let mut metadata = std::collections::HashMap::new();
    for key in keys {
        metadata.insert(key, tc.draw(text()));
    }
    let val = WithHashMap {
        title: tc.draw(text()),
        metadata,
    };
    let serialized = serde_kdl2::to_string(&val).unwrap();
    let deserialized: WithHashMap = serde_kdl2::from_str(&serialized).unwrap();
    assert_eq!(val, deserialized);
}
