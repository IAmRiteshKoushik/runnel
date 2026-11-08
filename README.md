# Runnel: A High-Performance Log Shipper



## Synthetic HTTP workloads

`http-workload` is a separate Rust binary. Its dependencies are enabled only
by the `workload` Cargo feature. Start a RabbitMQ broker with an existing input queue. The pipeline's Compose topology
is still tracked in issue #3. Until it exists, this creates a temporary broker
and a durable `runnel.input` queue for a local run:

```sh
docker run -d --name runnel-rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:4-management
# Wait for RabbitMQ to start, then declare the queue through its management API.
curl -u guest:guest -H 'content-type: application/json' -X PUT \
  http://localhost:15672/api/queues/%2F/runnel.input \
  -d '{"durable":true,"auto_delete":false,"arguments":{}}'
cargo run --release --features workload --bin http-workload -- --count 1000 --rate 100 --sizes 1024 8192 \
  --malformed-every 20 --incompatible-every 25
```

Use `docker rm -f runnel-rabbitmq` to remove this temporary broker and its
messages. For an existing topology, pass `--url`, `--exchange`, and
`--routing-key` as needed. The defaults use the default exchange and the
`runnel.input` queue on localhost. The tool does not declare or modify broker
topology. It publishes persistent messages with publisher confirms and
mandatory routing, and exits with an error if publication or routing fails.
Credentials in a command-line URL can be visible to other local processes.

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

Run the generator tests with `cargo test --features workload --bin http-workload`.
