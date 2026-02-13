# Event Streaming (To-Be)

DeliDev uses server-streamed workspace events for near-real-time synchronization.

## Goals

1. minimize polling in client UI for task/PR state
2. provide consistent event payloads across desktop and mobile
3. support replay and reconnection without data loss

## Stream Endpoint

- RPC: `EventStreamService.StreamWorkspaceEvents`
- Request: `workspace_id`, optional `from_sequence`
- Response: stream of `WorkspaceEventEnvelope`

## Envelope Contract

| Field | Type | Description |
|---|---|---|
| sequence | uint64 | Monotonic per workspace |
| event_type | StreamEventType | Typed event category |
| emitted_at | timestamp | Server emission time |
| payload | oneof | Event-specific payload |

## Event Types

1. `TASK_UPDATED`
2. `SUBTASK_UPDATED`
3. `SESSION_OUTPUT`
4. `SESSION_STATE_CHANGED`
5. `PR_UPDATED`
6. `REVIEW_ASSIST_UPDATED`
7. `NOTIFICATION_CREATED`

## Replay and Resume

1. client stores last applied sequence
2. on reconnect, client sends `from_sequence = last + 1`
3. server replays retained events from that point
4. if sequence is too old for retention, server returns explicit resync-required error

## Ordering and Idempotency

1. ordering guarantee is per workspace sequence
2. client reducers must be idempotent by sequence
3. duplicate envelopes may arrive during reconnect edges and must be ignored

## Backpressure and Health

1. server may batch high-frequency session output events
2. heartbeat envelopes may be emitted to keep connection alive
3. clients should auto-reconnect with bounded exponential backoff

## Security

1. workspace authorization on stream open
2. stream termination on token expiry or permission change
3. no secret payloads in stream body

## Operational Metrics

1. active stream connections
2. average stream lag (`now - emitted_at`)
3. reconnect rate
4. replay volume
5. dropped connection count
