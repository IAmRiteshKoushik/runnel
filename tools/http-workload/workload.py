"""Publish synthetic HTTP log events to an existing RabbitMQ input."""

import argparse
import itertools
import json
import math
import time

import pika
from faker import Faker

DATASETS = ("checkout", "profile", "search")


def event(fake, dataset):
    """Build a synthetic request and response with dataset-specific contents."""
    record = {
        "timestamp": fake.iso8601(),
        "request": {
            "method": "POST", "path": "",
            "headers": {"content-type": "application/json", "authorization": f"Bearer {fake.uuid4()}", "x-request-id": fake.uuid4()},
            "body": {}, "remote_ip": fake.ipv4_public(),
        },
        "response": {"status": 200, "headers": {"content-type": "application/json"}, "body": {}},
        "duration_ms": fake.random_int(min=1, max=1500),
    }
    request, response = record["request"], record["response"]
    if dataset == "checkout":
        request["path"] = "/api/orders"
        request["body"] = {"email": fake.email(), "card": fake.credit_card_number(), "items": [{"sku": fake.bothify("SKU-####"), "quantity": 2}]}
        response["body"] = {"order_id": fake.uuid4(), "total": "49.99"}
        response["status"] = 201
    elif dataset == "profile":
        request["method"], request["path"] = "PUT", "/api/profile"
        request["body"] = {"name": fake.name(), "email": fake.email(), "address": fake.address()}
        response["body"] = {"user_id": fake.uuid4(), "updated": True}
    elif dataset == "search":
        request["path"] = "/api/search"
        request["body"] = {"query": fake.sentence(), "filters": {"category": fake.word()}}
        response["body"] = {"results": [{"id": fake.uuid4(), "title": fake.sentence()} for _ in range(3)]}
    else:
        raise ValueError(f"unknown dataset: {dataset}")
    return record


def make_message(fake, dataset, size, kind):
    if kind == "malformed":
        return b'{"request":'
    record = event(fake, dataset)
    if kind == "incompatible":
        record["request"] = "not an object"
    # Size is a minimum in bytes. Existing fields can already exceed it.
    record["padding"] = ""
    encoded = json.dumps(record, ensure_ascii=True, separators=(",", ":")).encode()
    if len(encoded) < size:
        record["padding"] = "x" * (size - len(encoded))
        encoded = json.dumps(record, ensure_ascii=True, separators=(",", ":")).encode()
    return encoded


def positive_int(value):
    number = int(value)
    if number <= 0:
        raise argparse.ArgumentTypeError("must be greater than zero")
    return number


def positive_float(value):
    number = float(value)
    if not math.isfinite(number) or number <= 0:
        raise argparse.ArgumentTypeError("must be finite and greater than zero")
    return number


def parse_args(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    limit = parser.add_mutually_exclusive_group(required=True)
    limit.add_argument("--count", type=positive_int, help="number of messages")
    limit.add_argument("--duration", type=positive_float, help="run for this many seconds")
    parser.add_argument("--rate", type=positive_float, default=100, help="target messages per second (default: 100)")
    parser.add_argument("--sizes", type=positive_int, nargs="+", default=[1024], help="minimum message sizes in bytes, cycled in order")
    parser.add_argument("--datasets", nargs="+", choices=DATASETS, default=list(DATASETS))
    parser.add_argument("--malformed-every", type=positive_int, help="send malformed JSON at this interval")
    parser.add_argument("--incompatible-every", type=positive_int, help="send an incompatible request at this interval")
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--url", default="amqp://guest:guest@localhost:5672/%2F", help="AMQP URL; can contain credentials")
    parser.add_argument("--exchange", default="", help="existing exchange (default: RabbitMQ default exchange)")
    parser.add_argument("--routing-key", default="runnel.input", help="input queue or exchange routing key")
    return parser.parse_args(argv)


def publish(args):
    fake = Faker()
    fake.seed_instance(args.seed)
    datasets, sizes = itertools.cycle(args.datasets), itertools.cycle(args.sizes)
    connection = pika.BlockingConnection(pika.URLParameters(args.url))
    sent = 0
    started = time.monotonic()
    try:
        channel = connection.channel()
        channel.confirm_delivery()
        while args.count is None or sent < args.count:
            now = time.monotonic()
            if args.duration is not None and now - started >= args.duration:
                break
            deadline = started + sent / args.rate
            if deadline > now:
                if args.duration is not None and deadline >= started + args.duration:
                    break
                time.sleep(deadline - now)
            number = sent + 1
            kind = "valid"
            if args.malformed_every and number % args.malformed_every == 0:
                kind = "malformed"
            elif args.incompatible_every and number % args.incompatible_every == 0:
                kind = "incompatible"
            message = make_message(fake, next(datasets), next(sizes), kind)
            channel.basic_publish(exchange=args.exchange, routing_key=args.routing_key, body=message,
                                  properties=pika.BasicProperties(content_type="application/json", delivery_mode=2), mandatory=True)
            sent += 1
    finally:
        connection.close()
    return sent, time.monotonic() - started


def main():
    args = parse_args()
    try:
        sent, elapsed = publish(args)
    except (pika.exceptions.AMQPError, OSError) as exc:
        # Connection exceptions can contain the URL; avoid printing credentials.
        raise SystemExit(f"RabbitMQ publication failed ({type(exc).__name__})") from None
    print(f"Published {sent} messages in {elapsed:.2f}s")


if __name__ == "__main__":
    main()
