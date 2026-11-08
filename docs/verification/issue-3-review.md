# Issue 3 review verification

On 2026-11-08, `cargo test --all-features` passed (the broker test is ignored by default). The connector-substitution test delivered a redacted event using alternative source and sink implementations. Startup tests covered invalid URL rejection and a source connection failure without printing the supplied password.

A separate Compose project was started with `RUNNEL_AMQP_PORT=25672 RUNNEL_MANAGEMENT_PORT=25673 docker-compose -p runnel-review-1cca8081 up -d`. The container was created, but its health check reported that RabbitMQ had not fully booted. Its logs stopped at `Importing concurrently 1 users...` during definitions import; the management endpoint reset connections. Therefore the configured queues and their durability could not be verified against that broker.

`RUNNEL_TEST_URL='amqp://guest:guest@127.0.0.1:25672/%2F' cargo test --test rabbitmq -- --ignored --nocapture` failed at the initial connection with `Connection reset by peer`; it did not exercise delivery, redaction, retention, or acknowledgements. A pre-existing broker container was also unhealthy and was left untouched. Repeat the integration test and inspect the provisioned queues when a healthy broker is available.
