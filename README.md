# serde-kdl

[Serde](https://serde.rs) integration for [KDL](https://kdl.dev) (KDL Document Language).

Built on top of [`kdl`](https://crates.io/crates/kdl) v6.5 (KDL v2 spec).

## Quick Start

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    title: String,
    count: i32,
    enabled: bool,
}

let kdl_input = r#"
title "My App"
count 42
enabled #true
"#;

// Deserialize
let config: Config = serde_kdl::from_str(kdl_input).unwrap();

// Serialize
let output = serde_kdl::to_string(&config).unwrap();

// Roundtrip
let roundtrip: Config = serde_kdl::from_str(&output).unwrap();
assert_eq!(config, roundtrip);
```

## Mapping Rules

### Structs → Nodes

Each struct field becomes a KDL node. The node name is the field name, and the
value is the first argument.

```kdl
title "My App"
count 42
enabled #true
```

### Nested Structs → Children Blocks

```kdl
server {
    host "localhost"
    port 8080
}
```

### Vec of Primitives → Multiple Arguments

```kdl
tags "web" "rust" "config"
```

### Vec of Structs → Repeated Nodes

```kdl
server {
    host "localhost"
    port 8080
}
server {
    host "example.com"
    port 443
}
```

### Dash Children Convention

For deserialization, the `-` (dash) node name convention is supported:

```kdl
items {
    - 1
    - 2
    - 3
}
```

### Tuples → Multiple Arguments

```kdl
point 1.0 2.0 3.0
```

### Option

`None` is represented by the absence of a node. Serialization omits `None`
fields entirely. A `#null` argument also deserializes as `None`.

### Enums

**Unit variants** serialize as strings:

```kdl
color "Red"
```

**Newtype variants** use the variant name as a child node name:

```kdl
value {
    Text "hello"
}
```

**Struct variants** use the variant name as a child node with a children block:

```kdl
shape {
    Circle {
        radius 5.0
    }
}
```

**Tuple variants** use the variant name as a child node with multiple arguments:

```kdl
data {
    Point 1.0 2.0 3.0
}
```

### HashMap / BTreeMap

Maps serialize identically to structs — each key becomes a node name:

```kdl
settings {
    key1 "value1"
    key2 "value2"
}
```

## API

```rust
// Deserialize from string
let config: Config = serde_kdl::from_str(kdl_str)?;

// Deserialize from KdlDocument
let config: Config = serde_kdl::from_doc(&doc)?;

// Serialize to string
let s: String = serde_kdl::to_string(&config)?;

// Serialize to string (auto-formatted)
let s: String = serde_kdl::to_string_pretty(&config)?;

// Serialize to KdlDocument
let doc: kdl::KdlDocument = serde_kdl::to_doc(&config)?;
```

## License

MIT OR Apache-2.0
