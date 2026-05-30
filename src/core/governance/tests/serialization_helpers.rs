use super::*;

#[test]
fn json_to_string_with_some_object() {
    use serde_json::json;
    let result = json_to_string(&Some(json!({"key": "value"})));
    assert!(result.is_some());
    assert!(
        result.as_ref().unwrap().contains("\"key\""),
        "result should contain key: {:?}",
        result
    );
    assert!(
        result.as_ref().unwrap().contains("\"value\""),
        "result should contain value: {:?}",
        result
    );
}

#[test]
fn json_to_string_with_some_string() {
    use serde_json::json;
    let result = json_to_string(&Some(json!("hello")));
    assert_eq!(result, Some("\"hello\"".to_string()));
}

#[test]
fn json_to_string_with_none() {
    let result: Option<String> = json_to_string(&None);
    assert!(result.is_none());
}

#[test]
fn json_to_string_with_some_number() {
    use serde_json::json;
    let result = json_to_string(&Some(json!(42)));
    assert_eq!(result, Some("42".to_string()));
}

#[test]
fn string_list_to_json_with_some_list() {
    let input = Some(vec!["a".to_string(), "b".to_string()]);
    let result = string_list_to_json(&input);
    assert!(result.is_some());
    let parsed: Vec<String> = serde_json::from_str(result.as_ref().unwrap()).unwrap();
    assert_eq!(parsed, vec!["a", "b"]);
}

#[test]
fn string_list_to_json_with_empty_list() {
    let input: Option<Vec<String>> = Some(vec![]);
    let result = string_list_to_json(&input);
    assert_eq!(result, Some("[]".to_string()));
}

#[test]
fn string_list_to_json_with_none() {
    let input: Option<Vec<String>> = None;
    let result = string_list_to_json(&input);
    assert!(result.is_none());
}

#[test]
fn string_list_to_json_with_strings_containing_special_chars() {
    let input = Some(vec!["hello world".to_string(), "a\"b".to_string()]);
    let result = string_list_to_json(&input);
    assert!(result.is_some());
    let parsed: Vec<String> = serde_json::from_str(result.as_ref().unwrap()).unwrap();
    assert_eq!(parsed, vec!["hello world", "a\"b"]);
}
