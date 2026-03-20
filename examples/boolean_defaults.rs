use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

// Custom deserializer for integer with default value
fn retry_count_default<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    struct RetryCountDefaultVisitor;

    impl<'de> Visitor<'de> for RetryCountDefaultVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "an integer or bare node name")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as i32)
        }

        fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as i32)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as i32)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(3) // Default retry count for bare nodes
        }
    }

    deserializer.deserialize_any(RetryCountDefaultVisitor)
}

// Custom deserializer for string with default value
fn name_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct NameDefaultVisitor;

    impl<'de> Visitor<'de> for NameDefaultVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a string or bare node name")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok("unknown".to_string()) // Default name for bare nodes
        }
    }

    deserializer.deserialize_any(NameDefaultVisitor)
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_true")]
    enabled: bool,
    
    #[serde(deserialize_with = "serde_kdl2::bare_defaults::bool::bare_false")]
    debug: bool,
    
    #[serde(deserialize_with = "retry_count_default")]
    retry_count: i32,
    
    #[serde(deserialize_with = "name_default")]
    username: String,
    
    // Regular field (requires explicit value)
    port: u16,
}

fn main() {
    let kdl_input = r#"
        enabled          // defaults to true
        debug            // defaults to false  
        retry_count      // defaults to 3
        username         // defaults to "unknown"
        port 8080        // explicit value required
    "#;
    
    let config: Config = serde_kdl2::from_str(kdl_input).unwrap();
    
    println!("{:#?}", config);
    // Output:
    // Config {
    //     enabled: true,           // from bare_true default
    //     debug: false,            // from bare_false default
    //     retry_count: 3,          // from custom default
    //     username: "unknown",     // from custom default
    //     port: 8080,             // from explicit value
    // }
    
    // You can still override the defaults with explicit values
    let kdl_override = r#"
        enabled #false      // overrides bare_true default
        debug #true         // overrides bare_false default
        retry_count 10      // overrides default
        username "alice"    // overrides default
        port 3000
    "#;
    
    let config2: Config = serde_kdl2::from_str(kdl_override).unwrap();
    println!("{:#?}", config2);
    // Output:
    // Config {
    //     enabled: false,     // explicit override
    //     debug: true,        // explicit override
    //     retry_count: 10,    // explicit override
    //     username: "alice",  // explicit override
    //     port: 3000,
    // }
}