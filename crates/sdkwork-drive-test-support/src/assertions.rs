/// Assert that a JSON value has the expected field.
pub fn assert_has_field(value: &serde_json::Value, field: &str) {
    assert!(
        value.get(field).is_some(),
        "Expected field '{}' to exist in {:?}",
        field,
        value
    );
}

/// Assert that a JSON value has the expected string field value.
pub fn assert_string_field(value: &serde_json::Value, field: &str, expected: &str) {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("Field '{}' is not a string in {:?}", field, value));
    assert_eq!(
        actual, expected,
        "Field '{}' expected '{}', got '{}'",
        field, expected, actual
    );
}

/// Assert that a JSON value has the expected numeric field value.
pub fn assert_numeric_field(value: &serde_json::Value, field: &str, expected: i64) {
    let actual = value
        .get(field)
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| panic!("Field '{}' is not a number in {:?}", field, value));
    assert_eq!(
        actual, expected,
        "Field '{}' expected {}, got {}",
        field, expected, actual
    );
}

/// Assert that a JSON value has the expected boolean field value.
pub fn assert_bool_field(value: &serde_json::Value, field: &str, expected: bool) {
    let actual = value
        .get(field)
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| panic!("Field '{}' is not a boolean in {:?}", field, value));
    assert_eq!(
        actual, expected,
        "Field '{}' expected {}, got {}",
        field, expected, actual
    );
}

/// Assert that a JSON array has the expected length.
pub fn assert_array_length(value: &serde_json::Value, field: &str, expected: usize) {
    let array = value
        .get(field)
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("Field '{}' is not an array in {:?}", field, value));
    assert_eq!(
        array.len(),
        expected,
        "Field '{}' expected {} items, got {}",
        field,
        expected,
        array.len()
    );
}

/// Assert that a result is an error with the expected message.
pub fn assert_error_contains<T: std::fmt::Debug>(
    result: Result<T, impl std::error::Error>,
    expected: &str,
) {
    match result {
        Ok(val) => panic!("Expected error, got Ok({:?})", val),
        Err(e) => assert!(
            e.to_string().contains(expected),
            "Error '{}' does not contain '{}'",
            e,
            expected
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_assert_has_field() {
        let value = json!({"name": "test"});
        assert_has_field(&value, "name");
    }

    #[test]
    #[should_panic(expected = "Expected field 'missing'")]
    fn test_assert_has_field_missing() {
        let value = json!({"name": "test"});
        assert_has_field(&value, "missing");
    }

    #[test]
    fn test_assert_string_field() {
        let value = json!({"status": "active"});
        assert_string_field(&value, "status", "active");
    }

    #[test]
    fn test_assert_numeric_field() {
        let value = json!({"count": 42});
        assert_numeric_field(&value, "count", 42);
    }

    #[test]
    fn test_assert_bool_field() {
        let value = json!({"enabled": true});
        assert_bool_field(&value, "enabled", true);
    }

    #[test]
    fn test_assert_array_length() {
        let value = json!({"items": [1, 2, 3]});
        assert_array_length(&value, "items", 3);
    }
}
