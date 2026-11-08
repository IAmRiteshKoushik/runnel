# Runnel

Runnel moves log events between queues, applying configurable sanitization before delivery.

## Language

**Log event**:
A JSON record describing an HTTP request and response, including any captured bodies, headers, and other details.

**Sanitization**:
Transformation of a log event to redact, mask, or remove selected values before delivery.

**Source**:
The system from which Runnel receives log events.

**Sink**:
The system to which Runnel delivers sanitized log events.
