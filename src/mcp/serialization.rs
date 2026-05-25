use serde::Serialize;
use serde_json::{json, Value};

pub(crate) fn to_value_or_error<T: Serialize>(target: &str, value: T) -> Value {
    serde_json::to_value(value).unwrap_or_else(|err| {
        json!({
            "status": "serialization_error",
            "serialization_target": target,
            "error": err.to_string(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::to_value_or_error;
    use serde::ser::{Error, Serialize, Serializer};

    struct BrokenValue;

    impl Serialize for BrokenValue {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("forced serialization failure"))
        }
    }

    #[test]
    fn to_value_or_error_returns_structured_failure_payload() {
        let value = to_value_or_error("BrokenValue", BrokenValue);

        assert_eq!(value["status"], "serialization_error");
        assert_eq!(value["serialization_target"], "BrokenValue");
        assert!(value["error"]
            .as_str()
            .is_some_and(|error| error.contains("forced serialization failure")));
    }

    #[test]
    fn to_value_or_error_returns_json_on_success() {
        #[derive(serde::Serialize)]
        struct Simple {
            name: String,
            value: i64,
        }
        let input = Simple { name: "test".into(), value: 42 };
        let result = to_value_or_error("Simple", input);
        assert_eq!(result["name"], "test");
        assert_eq!(result["value"], 42);
        assert!(!result.as_object().unwrap().contains_key("status"));
    }
}
