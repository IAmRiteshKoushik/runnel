use runnel::sanitization::redact;
use serde_json::json;

#[test]
fn redacts_a_nested_object_value() {
    let output = redact(
        br#"{"request":{"token":"secret","method":"GET"}}"#,
        &["request", "token"],
    )
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output).unwrap(),
        json!({"request":{"token":"[REDACTED]","method":"GET"}})
    );
}

#[test]
fn rejects_invalid_or_incompatible_events() {
    assert!(redact(b"not json", &["request", "token"]).is_err());
    assert!(redact(br#"{"request":"secret"}"#, &["request", "token"]).is_err());
    assert!(redact(br#"{"request":{}}"#, &["request", "token"]).is_err());
    assert!(redact(br#"{"request":{"token":"secret"}}"#, &[]).is_err());
}
