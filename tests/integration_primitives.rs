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
        enabled: false, // overrides bare_true default
        disabled: true, // overrides bare_false default
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

// Test error handling for the macro-generated deserializers
serde_kdl2::bare_default!(error_test_deser, bool, true);
serde_kdl2::bare_default!(string_test_deser, String, "default".to_string());

#[test]
fn macro_generated_deserializer_error_handling() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct W {
        #[serde(default, deserialize_with = "error_test_deser")]
        flag: bool,
    }
    
    // Test that invalid types produce errors
    let result: Result<W, _> = serde_kdl2::from_str(r#"flag "not_a_bool""#);
    assert!(result.is_err());
}

#[test]
fn macro_generic_type_support() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct W {
        #[serde(default, deserialize_with = "string_test_deser")]
        name: String,
    }
    
    // Test bare node gets default
    let val: W = serde_kdl2::from_str("name").unwrap();
    assert_eq!(val.name, "default");
    
    // Test missing field gets default (from serde's default)
    let val: W = serde_kdl2::from_str("").unwrap();
    assert_eq!(val.name, "");
    
    // Test that non-unit values cause errors (to trigger expecting method)
    let result: Result<W, _> = serde_kdl2::from_str(r#"name 123"#);
    assert!(result.is_err());
}

// Test error case - invalid type for boolean defaults
#[test]
fn boolean_defaults_type_error() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct W {
        #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_true")]
        flag: bool,
    }
    let result: Result<W, _> = serde_kdl2::from_str(r#"flag "not_a_bool""#);
    assert!(result.is_err());
}



// ── Custom Boolean Defaults with Macro ─────────────────────────────────

// Generate custom deserializers using the macro
serde_kdl2::bare_default!(writable_deser, bool, true);
serde_kdl2::bare_default!(readonly_deser, bool, false);

// Custom missing default function
fn enabled_missing() -> bool {
    true
}

// Test the clean macro-based API
#[derive(Debug, Deserialize, PartialEq)]
struct CustomBoolDefaults {
    // missing → false (default), bare → true
    #[serde(default, deserialize_with = "writable_deser")]
    writable: bool,

    // missing → true (custom), bare → false
    #[serde(default = "enabled_missing", deserialize_with = "readonly_deser")]
    readonly: bool,

    // missing → false (default), bare → false
    #[serde(default, deserialize_with = "readonly_deser")]
    disabled: bool,

    // missing → true (custom), bare → true
    #[serde(default = "enabled_missing", deserialize_with = "writable_deser")]
    enabled: bool,
}

// Test missing fields use the correct defaults
deser_ok!(
    custom_bool_missing_fields,
    CustomBoolDefaults,
    "",
    CustomBoolDefaults {
        writable: false, // default (false)
        readonly: true,  // enabled_missing() → true
        disabled: false, // default (false)
        enabled: true,   // enabled_missing() → true
    }
);

// Test bare fields use the correct defaults
deser_ok!(
    custom_bool_bare_fields,
    CustomBoolDefaults,
    indoc! {"
        writable
        readonly
        disabled
        enabled
    "},
    CustomBoolDefaults {
        writable: true,  // writable_deser → true
        readonly: false, // readonly_deser → false
        disabled: false, // readonly_deser → false
        enabled: true,   // writable_deser → true
    }
);

// Test explicit values override all defaults
deser_ok!(
    custom_bool_explicit_values,
    CustomBoolDefaults,
    indoc! {"
        writable #false
        readonly #true
        disabled #true
        enabled #false
    "},
    CustomBoolDefaults {
        writable: false, // explicit
        readonly: true,  // explicit
        disabled: true,  // explicit
        enabled: false,  // explicit
    }
);

// Test the mount use case with clean macro API
serde_kdl2::bare_default!(mount_writable_deser, bool, true);

#[derive(Debug, Deserialize, PartialEq)]
struct CleanMount {
    source: String,
    #[serde(default, deserialize_with = "mount_writable_deser")]
    writable: bool,
}

deser_ok!(
    clean_mount_missing_writable,
    CleanMount,
    indoc! {"
        source \"/host/path\"
    "},
    CleanMount {
        source: "/host/path".to_string(),
        writable: false, // missing → false (default)
    }
);

deser_ok!(
    clean_mount_bare_writable,
    CleanMount,
    indoc! {"
        source \"/host/path\"
        writable
    "},
    CleanMount {
        source: "/host/path".to_string(),
        writable: true, // bare → true (macro)
    }
);

deser_ok!(
    clean_mount_explicit_writable,
    CleanMount,
    indoc! {"
        source \"/host/path\"
        writable #false
    "},
    CleanMount {
        source: "/host/path".to_string(),
        writable: false, // explicit
    }
);

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
