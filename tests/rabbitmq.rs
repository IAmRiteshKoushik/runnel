use lapin::{
    BasicProperties, Connection, ConnectionProperties, options::*,
    types::FieldTable,
};
use std::{
    process::{Command, Stdio},
    time::Duration,
};

// Requires the local Compose broker. Run explicitly with RUNNEL_TEST_URL set.
#[tokio::test]
#[ignore = "requires a running RabbitMQ broker and RUNNEL_TEST_URL"]
async fn running_pipeline_delivers_only_redacted_events() {
    let url = std::env::var("RUNNEL_TEST_URL").unwrap();
    let connection = Connection::connect(&url, ConnectionProperties::default())
        .await
        .unwrap();
    let channel = connection.create_channel().await.unwrap();
    let suffix = std::process::id();
    let input = format!("runnel.test.{suffix}.input");
    let output = format!("runnel.test.{suffix}.output");
    for queue in [&input, &output] {
        channel
            .queue_declare(
                queue.clone().into(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .unwrap();
    }
    let config_path =
        std::env::temp_dir().join(format!("runnel-test-{suffix}.toml"));
    std::fs::write(&config_path, format!("[source]\nurl_env='RUNNEL_TEST_URL'\nexchange=''\nrouting_key='{input}'\n[sink]\nurl_env='RUNNEL_TEST_URL'\nexchange=''\nrouting_key='{output}'\n[redaction]\npath=['request','token']\n")).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_runnel"))
        .arg(&config_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    channel
        .basic_publish(
            "".into(),
            input.clone().into(),
            BasicPublishOptions::default(),
            br#"{"request":{"token":"sensitive","method":"GET"}}"#,
            BasicProperties::default(),
        )
        .await
        .unwrap()
        .await
        .unwrap();
    let delivery = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            if let Some(message) = channel
                .basic_get(output.clone().into(), BasicGetOptions::default())
                .await
                .unwrap()
            {
                break message;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await;
    channel
        .basic_publish(
            "".into(),
            input.clone().into(),
            BasicPublishOptions::default(),
            br#"{"request":"private-invalid-value"}"#,
            BasicProperties::default(),
        )
        .await
        .unwrap()
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(
        channel
            .basic_get(output.clone().into(), BasicGetOptions::default())
            .await
            .unwrap()
            .is_none()
    );
    child.kill().unwrap();
    let result = child.wait_with_output().unwrap();
    let _ = std::fs::remove_file(config_path);
    let message = delivery.expect("pipeline did not deliver event");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&message.data).unwrap(),
        serde_json::json!({"request":{"token":"[REDACTED]","method":"GET"}})
    );
    let diagnostics = String::from_utf8_lossy(&result.stderr);
    assert!(!diagnostics.contains("sensitive"));
    assert!(!diagnostics.contains("private-invalid-value"));
    let retained = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(message) = channel
                .basic_get(input.clone().into(), BasicGetOptions::default())
                .await
                .unwrap()
            {
                break message;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("invalid event was not retained after stopping the pipeline");
    assert_eq!(retained.data, br#"{"request":"private-invalid-value"}"#);
    retained.ack(BasicAckOptions::default()).await.unwrap();
    message.ack(BasicAckOptions::default()).await.unwrap();
    for queue in [&input, &output] {
        channel
            .queue_delete(queue.clone().into(), QueueDeleteOptions::default())
            .await
            .unwrap();
    }
}
