# Workspace Connectivity

DexDex uses workspace connectivity profiles.

## Connectivity Types

### Local Endpoint Workspace

- `workspace.type = LOCAL_ENDPOINT`
- endpoint points to a server process on the same machine or device
- typical endpoint: `http://127.0.0.1:<port>`

### Remote Endpoint Workspace

- `workspace.type = REMOTE_ENDPOINT`
- endpoint points to a network-hosted server

## Shared Behavior

Both types use the same:

1. Connect RPC services
2. event streaming contracts
3. task, PR, and review workflows
4. notification model

## Differences

| Aspect | Local Endpoint Workspace | Remote Endpoint Workspace |
|---|---|---|
| Network | loopback/local | LAN/WAN |
| Auth | optional for solo setup | required in shared setup |
| Latency | lower | environment-dependent |
| Collaboration | typically single user | multi-user friendly |

## Workspace Setup Flow

1. enter workspace name
2. choose connectivity type
3. enter endpoint URL
4. verify connection
5. save workspace profile

## Mobile Connectivity

Mobile clients use the same workspace concept.
A mobile app can connect to local endpoints (same network or tunneled) and remote endpoints.

Capability rollout is phased:

1. baseline mobile flow supports monitoring, logs, plan actions, stop actions, and core remediation triggers
2. expanded mobile flow adds broader remediation and review interactions
3. rollout phase reflects interaction constraints, not lower product priority
4. connectivity contract stays identical across desktop and mobile
