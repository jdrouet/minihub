//! Typed attribute values attached to entities.

use serde::{Deserialize, Serialize};

/// A single typed attribute value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Json(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_string_variant_as_plain_string() {
        let val = AttributeValue::String("hello".to_string());
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"hello\"");
    }

    #[test]
    fn should_serialize_int_variant_as_number() {
        let val = AttributeValue::Int(42);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn should_serialize_float_variant_as_number() {
        let val = AttributeValue::Float(21.5);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "21.5");
    }

    #[test]
    fn should_serialize_bool_variant() {
        let val = AttributeValue::Bool(true);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "true");
    }

    #[test]
    fn should_deserialize_json_object_as_json_variant() {
        let json = r#"{"nested": "value"}"#;
        let val: AttributeValue = serde_json::from_str(json).unwrap();
        assert!(matches!(val, AttributeValue::Json(_)));
    }

    #[test]
    fn should_compare_equal_values() {
        assert_eq!(AttributeValue::Int(10), AttributeValue::Int(10));
        assert_ne!(AttributeValue::Int(10), AttributeValue::Int(20));
    }
}
