use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use runnel::config::{load, load_source};

static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

fn with_config(contents: &str, test: impl FnOnce(&Path)) {
    let path = std::env::temp_dir().join(format!(
        "runnel-config-{}-{}.toml",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, contents).unwrap();
    test(&path);
    fs::remove_file(path).unwrap();
}

#[test]
fn source_loads_without_deserializing_sink() {
    with_config(
        "[source]\nurl_env = 'SOURCE_URL'\nexchange = ''\nrouting_key = 'input'\n[sink]\nurl_env = 42\n",
        |path| {
            let source = load_source(path).unwrap();
            assert_eq!(source.routing_key, "input");
            assert_eq!(source.url_env, "SOURCE_URL");
        },
    );
}

#[test]
fn missing_source_fields_fail() {
    with_config("[source]\nurl_env = 'URL'", |path| {
        assert!(load_source(path).is_err());
    });
}

#[test]
fn invalid_toml_and_empty_required_fields_fail() {
    with_config("[source", |path| assert!(load_source(path).is_err()));
    with_config(
        "[source]\nurl_env = '  '\nexchange = ''\nrouting_key = 'input'",
        |path| assert!(load_source(path).is_err()),
    );
}

#[test]
fn checked_in_config_loads_both_endpoints() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.toml");
    let config = load(&path).unwrap();
    assert_eq!(config.source.routing_key, "runnel.input");
    assert_eq!(config.sink.routing_key, "runnel.output");
}
