# Helpdesk of Just
list:
  @just --list

# Generate and insert logs into RabbitMQ
loggen *args:
  @cargo run --features workload --bin loggen -- {{args}}
