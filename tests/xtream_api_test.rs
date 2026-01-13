// Test the flexible deserialization for Xtream API responses
// Run with: cargo test --test xtream_api_test

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

// Custom deserializer for fields that can be either string or integer
fn deserialize_string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Null => Ok(String::new()),
        _ => Err(Error::custom("expected string or number")),
    }
}

// Custom deserializer for integer fields that can be either int or string
fn deserialize_int_or_string<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;
    
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(n) => n.as_i64().map(|i| Some(i as i32)).ok_or_else(|| Error::custom("invalid number")),
        Value::String(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i32>().map(Some).map_err(|_| Error::custom("expected integer or numeric string"))
            }
        }
        Value::Null => Ok(None),
        _ => Err(Error::custom("expected integer or string")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestChannel {
    #[serde(deserialize_with = "deserialize_string_or_int", default)]
    pub num: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub stream_id: String,
    #[serde(deserialize_with = "deserialize_string_or_int", default)]
    pub category_id: String,
    #[serde(deserialize_with = "deserialize_int_or_string", default)]
    pub tv_archive: Option<i32>,
}

#[test]
fn test_deserialize_integer_as_string() {
    // Test case 1: num as integer (like real Xtream API)
    let json = r#"{
        "num": 1,
        "name": "PPV| BOXING 00: PAUL Vs JOSHUA 8PM ET",
        "stream_id": 1237348,
        "category_id": 42,
        "tv_archive": 1
    }"#;
    
    let channel: TestChannel = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(channel.num, "1");
    assert_eq!(channel.name, "PPV| BOXING 00: PAUL Vs JOSHUA 8PM ET");
    assert_eq!(channel.stream_id, "1237348");
    assert_eq!(channel.category_id, "42");
    assert_eq!(channel.tv_archive, Some(1));
}

#[test]
fn test_deserialize_string_as_string() {
    // Test case 2: num as string
    let json = r#"{
        "num": "2",
        "name": "Test Channel",
        "stream_id": "9999",
        "category_id": "10",
        "tv_archive": "2"
    }"#;
    
    let channel: TestChannel = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(channel.num, "2");
    assert_eq!(channel.stream_id, "9999");
    assert_eq!(channel.category_id, "10");
    assert_eq!(channel.tv_archive, Some(2));
}

#[test]
fn test_deserialize_missing_optional_fields() {
    // Test case 3: missing optional fields
    let json = r#"{
        "name": "Minimal Channel",
        "stream_id": 123
    }"#;
    
    let channel: TestChannel = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(channel.num, "");
    assert_eq!(channel.stream_id, "123");
    assert_eq!(channel.category_id, "");
}

#[test]
fn test_deserialize_real_xtream_response() {
    // Test case 4: Real Xtream API response format
    let json = r#"[
        {
            "num": 1,
            "name": "PPV| BOXING 00: PAUL Vs JOSHUA 8PM ET",
            "stream_type": "live",
            "stream_id": 1237348,
            "stream_icon": "https://example.com/logo.png",
            "category_id": 5,
            "tv_archive": 1
        },
        {
            "num": 2,
            "name": "News Channel",
            "stream_type": "live",
            "stream_id": 1237349,
            "stream_icon": "",
            "category_id": 3,
            "tv_archive": "2"
        }
    ]"#;
    
    let channels: Vec<TestChannel> = serde_json::from_str(json).expect("Failed to deserialize array");
    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].num, "1");
    assert_eq!(channels[0].stream_id, "1237348");
    assert_eq!(channels[0].tv_archive, Some(1));
    assert_eq!(channels[1].num, "2");
    assert_eq!(channels[1].tv_archive, Some(2));
}

#[test]
fn test_deserialize_float_as_string() {
    // Test case 5: Float numbers should also work
    let json = r#"{
        "num": 1.5,
        "name": "Test",
        "stream_id": 123.456,
        "category_id": 1
    }"#;
    
    let channel: TestChannel = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(channel.num, "1.5");
    assert_eq!(channel.stream_id, "123.456");
}

#[test]
fn test_deserialize_null_as_empty_string() {
    // Test case 6: Null values should become empty strings
    let json = r#"{
        "num": null,
        "name": "Test",
        "stream_id": 123,
        "category_id": null
    }"#;
    
    let channel: TestChannel = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(channel.num, "");
    assert_eq!(channel.category_id, "");
}
