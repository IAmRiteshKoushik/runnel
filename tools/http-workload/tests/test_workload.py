import json
from types import SimpleNamespace
from unittest.mock import patch

import pytest
from faker import Faker

import workload


@pytest.mark.parametrize("dataset", workload.DATASETS)
def test_dataset_contains_http_request_and_response(dataset):
    message = workload.make_message(Faker(), dataset, 2048, "valid")
    record = json.loads(message)
    assert record["request"]["headers"] and record["request"]["body"]
    assert record["response"]["headers"] and record["response"]["body"]
    assert len(message) >= 2048


def test_invalid_messages_exercise_different_failures():
    fake = Faker()
    with pytest.raises(json.JSONDecodeError):
        json.loads(workload.make_message(fake, "checkout", 1000, "malformed"))
    record = json.loads(workload.make_message(fake, "checkout", 1000, "incompatible"))
    assert isinstance(record["request"], str)


def test_publishes_mixed_messages_with_confirms_and_routing():
    args = workload.parse_args(["--count", "4", "--rate", "100000", "--sizes", "1024", "4096", "--malformed-every", "3", "--incompatible-every", "2"])
    messages = []
    class Channel:
        def confirm_delivery(self):
            self.confirmed = True
        def basic_publish(self, **kwargs):
            messages.append(kwargs)
    channel = Channel()
    connection = SimpleNamespace(channel=lambda: channel, close=lambda: None)
    with patch.object(workload.pika, "BlockingConnection", return_value=connection):
        sent, _ = workload.publish(args)
    assert sent == 4 and channel.confirmed
    assert all(m["mandatory"] and m["routing_key"] == "runnel.input" for m in messages)
    assert len(messages[0]["body"]) >= 1024 and len(messages[1]["body"]) >= 4096
    assert isinstance(json.loads(messages[1]["body"])["request"], str)
    with pytest.raises(json.JSONDecodeError):
        json.loads(messages[2]["body"])


@pytest.mark.parametrize("option", ["--rate", "--duration", "--count", "--sizes", "--malformed-every"])
def test_rejects_nonpositive_options(option):
    args = [option, "0"] if option in ("--count", "--duration") else ["--count", "1", option, "0"]
    with pytest.raises(SystemExit):
        workload.parse_args(args)
