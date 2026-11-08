# Runnel: A High-Performance Log Shipper



## Synthetic HTTP workloads

`loggen` is a separate Rust binary. Its dependencies are enabled by the
`workload` Cargo feature. The broker in `compose.yml` does not yet provision
queues, so create the input queue before running the generator:

```sh
podman compose up -d
# Wait for RabbitMQ to start, then declare the queue through its management API.
curl -u guest:guest -H 'content-type: application/json' -X PUT \
  http://localhost:15672/api/queues/%2F/runnel.input \
  -d '{"durable":true,"auto_delete":false,"arguments":{}}'
export RUNNEL_SOURCE_URL='amqp://guest:guest@localhost:5672/%2F'
cargo run --release --features workload --bin loggen -- --count 1000 --rate 100 --sizes 1024 8192 \
  --malformed-every 20 --incompatible-every 25
```

The tool reads the `[source]` section of `config.toml` from the current
directory. Use `--config PATH` to select another file. Set `url_env` to the
name of an environment variable containing the AMQP URL, and set `exchange`
and `routing_key` to the existing input topology. The `[sink]` section is for
the future pipeline and is not read by the generator. Do not put credentials in
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
