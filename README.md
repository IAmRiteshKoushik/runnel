# Runnel: A High-Performance Log Shipper



## Run the pipeline

Start the local broker with `docker compose up -d` (or `docker-compose up -d`), and wait for it to become healthy. It provisions durable `runnel.input` and `runnel.output` queues. The default guest credentials are for local development only. Ports can be changed with `RUNNEL_AMQP_PORT` and `RUNNEL_MANAGEMENT_PORT`.

```sh
export RUNNEL_URL='amqp://guest:guest@localhost:5672/%2F'
cargo run --bin runnel
```

In another terminal, publish a synthetic example and read the result through the management API:

```sh
curl -u guest:guest -H 'content-type: application/json' \
  -X POST http://localhost:15672/api/exchanges/%2F/amq.default/publish \
  -d '{"properties":{"delivery_mode":2},"routing_key":"runnel.input","payload":"{\"request\":{\"headers\":{\"authorization\":\"example-secret\"}}}","payload_encoding":"string"}'
curl -u guest:guest -H 'content-type: application/json' \
  -X POST http://localhost:15672/api/queues/%2F/runnel.output/get \
  -d '{"count":1,"ackmode":"ack_requeue_false","encoding":"auto"}'
```

The output contains `"authorization":"[REDACTED]"`. Set `redaction.path` to a nonempty array of exact object keys. The target must exist; missing keys, non-object intermediate values, and invalid JSON fail closed. Arrays and wildcards are not supported in this ticket. Configuration and both URL environment variables are checked before consumption; pass a config file path as the first argument to select another file. The source `routing_key` is the input queue name and its `exchange` must be empty. The sink exchange and routing key must already exist and route to a durable queue.

Runnel holds one delivery at a time and acknowledges it only after a persistent, mandatory publication is confirmed and routed. A connection or publication failure causes a reconnect and redelivery; the window between confirmation and acknowledgement can produce duplicates. Durability depends on broker topology and persistent input messages. If sanitization fails, Runnel pauses with the original message unacknowledged; stop the process to return it to the input queue and inspect it through an appropriately restricted process. This blocks later events. There is no quarantine or automatic recovery yet, and no unsanitized message is deliberately sent to the output. Avoid input queue expiry, delivery limits, or dead-letter policies that could discard or expose retained messages. Protect broker access and retained data.

Run the broker integration test explicitly against a development broker:

```sh
RUNNEL_TEST_URL="$RUNNEL_URL" cargo test --test rabbitmq -- --ignored
```

## Synthetic HTTP workloads

`loggen` is a separate Rust binary. Its dependencies are enabled by the
`workload` Cargo feature. The broker in `compose.yml` provisions the input queue:

```sh
podman compose up -d
export RUNNEL_URL='amqp://guest:guest@localhost:5672/%2F'
cargo run --release --features workload --bin loggen -- --count 1000 --rate 100 --sizes 1024 8192 \
  --malformed-every 20 --incompatible-every 25
```

The tool reads the `[source]` section of `config.toml` from the current
directory. Use `--config PATH` to select another file. Set `url_env` to the
name of an environment variable containing the AMQP URL, and set `exchange`
and `routing_key` to the existing input topology. The `[sink]` and `[redaction]` sections are not read by the generator. Do not put credentials in
the TOML file. The generator does not declare or modify broker topology. It
publishes persistent messages with publisher confirms and mandatory routing,
and exits with an error if publication or routing fails.

Choose `--count` or `--duration` in seconds. `--rate` is a target messages per
second; generation and synchronous publisher confirms can limit the achieved
rate. The tool prints the number confirmed and elapsed time. `--sizes` cycles
through minimum encoded byte sizes; the actual event can be larger than the
requested size. Malformed JSON is a short fixed payload and does not use the
size setting. `--datasets` selects any of `checkout`, `profile`, and `search`;
they cycle in order, as do sizes. `--seed` controls the fake data's random sequence; timestamps use the current clock.
Every Nth message can be malformed or have a string in place of the `request`
object. When both intervals coincide, malformed takes precedence. An
incompatible event exercises a rule addressing a field under `request`; its
failure depends on the pipeline's configured rules. All fields are synthetic.

Run the tests with `cargo test --all-features`.
