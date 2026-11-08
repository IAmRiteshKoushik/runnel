use std::process::Command;

#[test]
fn malformed_broker_url_is_rejected_without_disclosing_it() {
    use std::{
        process::Stdio,
        thread,
        time::{Duration, Instant},
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_runnel"))
        .env("RUNNEL_URL", "not-a-url-secret")
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("invalid URL did not fail at startup");
        }
        thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("invalid AMQP URL"), "{diagnostic}");
    assert!(!diagnostic.contains("not-a-url-secret"));
}

#[test]
fn connection_failures_identify_operation_without_credentials() {
    use std::{process::Stdio, thread, time::Duration};
    let mut child = Command::new(env!("CARGO_BIN_EXE_runnel"))
        .env(
            "RUNNEL_URL",
            "amqp://guest:private-password@127.0.0.1:1/%2F",
        )
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(750));
    child.kill().unwrap();
    let output = child.wait_with_output().unwrap();
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostic.contains("source connection failed"),
        "{diagnostic}"
    );
    assert!(!diagnostic.contains("private-password"));
}
