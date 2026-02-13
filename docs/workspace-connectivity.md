# Workspace Connectivity (Replaces Local vs Remote Mode)

DeliDev uses workspace connectivity profiles instead of a mode switch.

## Why This Change

Previous mode language implied two different application architectures.
The rewrite uses one RPC architecture and two endpoint locality patterns.

## Connectivity Types

### Local Endpoint Workspace

- `workspace.type = LOCAL_ENDPOINT`
- endpoint points to a server process running on the same machine/device
- typical endpoint: `http://127.0.0.1:<port>`

### Remote Endpoint Workspace

- `workspace.type = REMOTE_ENDPOINT`
- endpoint points to a network-hosted shared server

## Shared Behavior

Both types use the same:

1. Connect RPC services
2. Event streaming contracts
3. task/pr/review workflows
4. notification model

## Differences

| Aspect | Local Endpoint Workspace | Remote Endpoint Workspace |
|---|---|---|
| Network | loopback/local | LAN/WAN |
| Auth | optional for solo setup | required in shared setup |
| Latency | lower | environment-dependent |
| Collaboration | typically single user | multi-user friendly |

## Workspace Setup Flow

1. user enters workspace name
2. user selects connectivity type
3. user enters endpoint URL
4. user verifies connection
5. user stores workspace profile

## Mobile Implications

Mobile clients use the same workspace concept.
A mobile app can connect to a local endpoint (same network/tunneled) or remote endpoint.

## Migration Rule

Any old "mode" UI or docs should be replaced with workspace terminology.
