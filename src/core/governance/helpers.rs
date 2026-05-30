pub(super) fn now_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs().to_string()
}

pub(super) fn json_to_string(val: &Option<serde_json::Value>) -> Option<String> {
    val.as_ref().map(|v| v.to_string())
}

pub(super) fn string_list_to_json(list: &Option<Vec<String>>) -> Option<String> {
    list.as_ref()
        .map(|items| serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string()))
}
