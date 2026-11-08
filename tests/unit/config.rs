use super::*;

#[test]
fn source_parses_without_deserializing_sink() {
    let config: SourceConfig = toml::from_str(
            "[source]\nurl_env = 'SOURCE_URL'\nexchange = ''\nrouting_key = 'input'\n[sink]\nurl_env = 42\n",
        ).unwrap();
    assert_eq!(config.source.routing_key, "input");
    assert_eq!(config.source.url_env, "SOURCE_URL");
}

#[test]
fn missing_source_fields_fail() {
    assert!(
        toml::from_str::<SourceConfig>("[source]\nurl_env = 'URL'").is_err()
    );
}

#[test]
fn checked_in_config_loads_both_endpoints() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.toml");
    let config = load(&path).unwrap();
    assert_eq!(config.source.routing_key, "runnel.input");
    assert_eq!(config.sink.routing_key, "runnel.output");
}

#[test]
fn pipeline_rejects_missing_or_empty_redaction_path() {
    let text = "[source]\nurl_env='URL'\nexchange=''\nrouting_key='in'\n[sink]\nurl_env='URL'\nexchange=''\nrouting_key='out'\n";
    assert!(parse(text).is_err());
    assert!(parse(&format!("{text}\n[redaction]\npath=[]\n")).is_err());
    assert!(
        parse(&format!("{text}\n[redaction]\npath=['request', '']\n")).is_err()
    );
}
