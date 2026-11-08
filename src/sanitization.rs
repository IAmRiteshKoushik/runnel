use serde_json::Value;

pub fn redact(input: &[u8], path: &[&str]) -> Result<Vec<u8>, &'static str> {
    if path.is_empty() || path.iter().any(|segment| segment.is_empty()) {
        return Err("invalid path");
    }
    let mut value: Value =
        serde_json::from_slice(input).map_err(|_| "invalid JSON")?;
    let mut current = &mut value;
    for segment in path {
        current = current
            .as_object_mut()
            .and_then(|object| object.get_mut(*segment))
            .ok_or("path does not match")?;
    }
    *current = Value::String("[REDACTED]".into());
    serde_json::to_vec(&value).map_err(|_| "cannot encode event")
}
