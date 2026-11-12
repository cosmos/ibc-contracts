# Logging and best practices

## Key principles
Keep logs simple, structured, and valuable for on-call debugging. This page captures only the essentials so teams can adopt a consistent approach quickly.


- Purpose: logs exist to explain what happened and why, especially when something fails. Optimize for on-call readability and machine parsing.
- Think in events: one log captures a single event with context (including correlation fields)
- Never log secrets: just don't
- Practical levels: prefer INFO; use DEBUG sparingly; only ERROR for failures that need attention.

## Log levels
- DEBUG: development-time details and deep diagnosis. Usually disabled or sampled in prod.
- INFO: normal operations and key state transitions (startup, readiness, completed request, retries).
  - Prefer INFO over DEBUG, because that is what someone on-call will be able to read.
- ERROR: errors that caused failure, dropped work, or user-visible impact. Include actionable context.

## Correlation
Correlation fields are what we use to find related information. 
It can be any information that identifies an area we care about, such as a specific request, module, environment, etc.
It enables us to compile logs and metrics for one or more attributes, helping us narrow down the search for information or a signal.

### Traces vs spans
A trace is the entire journey of a request as it flows through multiple services, components, and operations.
Typically identified by an initiating request ID.

Spans are sub-sections _in_ a span is more like a single unit of work — it represents one operation in your system.
Example: A database query, an HTTP request to another service, or a function execution.
Super helpful if you want to figure out where time is being spent in a request to find performance bottlenecks or other issues related to timing.

### Correlation fields

To make this possible, we need always to include any available correlation fields.
To the extent possible, they should be either set up on launch of the service (such as environment, service name, service versions, etc) or automated by the framework/language.

Minimum list of correlation fields
- trace_id: the request ID (e.g., a specific gRPC request's ID)
- timestamp: RFC 3339 UTC
- environment: dev/prod
- service_name: name of the current service (e.g., proof-api)
- service_version: the deployed version of the code
- span_id: when available

## Logging errors

It is important to only handle errors once. Errors can become overly distracting if they are handled multiple times (logging an error and returning an error both count as handling the error).
Typically an error should be logged only when it is decided that the error will either be returned to the client (client here means an external api client, not returning to a calling function kind of client), or when it is decided that the error will be discarded (e.g. a goroutine that does not return errors to the parent function).

## Rules
- Use structured logging (JSON) to stdout/stderr; avoid multiline messages.
- Use OpenTelemetry semantic conventions where applicable and keep field names consistent across services.
- Include correlation fields and basic resource fields (service.name, service.version, environment).
- Use RFC 3339 UTC timestamps and disable ANSI colors in production logs.

## Language-specific guides

### Rust

TODO: IBC-145 Add Rust-specific instructions for how to do logging correctly in our repo
