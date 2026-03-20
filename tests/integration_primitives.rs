//! Integration tests for primitive types (integers, floats, booleans, chars, tuples).

use indoc::indoc;
use serde::{Deserialize, Serialize};

// Test macros
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

macro_rules! deser_ok {
    ($name:ident, $ty:ty, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let val: $ty = serde_kdl2::from_str($input).unwrap();
            assert_eq!(val, $expected);
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

// ── Boolean Defaults ───────────────────────────────────────────────────

#[derive(Debug, Deserialize, PartialEq)]
struct BooleanDefaults {
    #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_true")]
    enabled: bool,
    
    #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_false")]
    disabled: bool,
}

// Test that bare node names use the configured defaults
deser_ok!(
    boolean_defaults_bare,
    BooleanDefaults,
    indoc! {"
        enabled
        disabled
    "},
    BooleanDefaults {
        enabled: true,   // bare_true default
        disabled: false, // bare_false default
    }
);

// Test that explicit values override defaults
deser_ok!(
    boolean_defaults_explicit_override,
    BooleanDefaults,
    indoc! {"
        enabled #false
        disabled #true
    "},
    BooleanDefaults {
        enabled: false,  // overrides bare_true default
        disabled: true,  // overrides bare_false default
    }
);

// Test mixed bare and explicit values
deser_ok!(
    boolean_defaults_mixed,
    BooleanDefaults,
    indoc! {"
        enabled
        disabled #true
    "},
    BooleanDefaults {
        enabled: true,  // bare_true default
        disabled: true, // explicit value
    }
);

// Test single field with bare_true default
#[test]
fn boolean_defaults_single_bare_true() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct W {
        #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_true")]
        flag: bool,
    }
    let val: W = serde_kdl2::from_str("flag").unwrap();
    assert_eq!(val.flag, true);
}

// Test single field with bare_false default
#[test]
fn boolean_defaults_single_bare_false() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct W {
        #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_false")]
        flag: bool,
    }
    let val: W = serde_kdl2::from_str("flag").unwrap();
    assert_eq!(val.flag, false);
}

// Test that explicit false works with bare_true default
#[test]
fn boolean_defaults_explicit_false_with_bare_true() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct W {
        #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_true")]
        flag: bool,
    }
    let val: W = serde_kdl2::from_str("flag #false").unwrap();
    assert_eq!(val.flag, false);
}

// ── Characters ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WCh {
    letter: char,
}

roundtrip!(roundtrip_char, WCh, WCh { letter: 'X' });
roundtrip!(roundtrip_char_multibyte, WCh, WCh { letter: 'é' });
roundtrip!(roundtrip_char_cjk, WCh, WCh { letter: '中' });
roundtrip!(roundtrip_char_emoji, WCh, WCh { letter: '🦀' });

// ── Floats ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WF32 {
    value: f32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WF64 {
    value: f64,
}

roundtrip!(roundtrip_f32, WF32, WF32 { value: 3.125 });
roundtrip!(roundtrip_f64, WF64, WF64 { value: 3.125 });