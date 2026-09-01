# Data Model: Cold Platform-Client Ownership

## Platform Adapter

A side-effect-free producer of one prepared platform plan.

| Field | Meaning | Validation |
| --- | --- | --- |
| `kind` | Stable platform token | Closed set; `steam` is supported in S112 |
| `root` | Current local platform root | Absolute, canonical directory |
| `executable` | Exact platform client image | Absolute canonical file beneath `root` |
| `application_id` | Selected platform application | Non-empty identifier from the stored target/profile |

The adapter does not own session state, launch effects, compatibility storage, or target resolution.

## Platform Launch Plan

An immutable pre-effect value retained inside the existing managed-launch configuration.

| Field | Meaning | Validation |
| --- | --- | --- |
| `platform` | Adapter identity | Matches the prepared adapter |
| `root_launch` | Exact platform executable, working directory, arguments, environment | Same direct-launch validation as S109 and S111 |
| `dispatch` | Exact application dispatch action | Closed platform-specific variant with non-empty application identifier |
| `stages` | Platform and terminal-client ownership declarations | One platform root; one terminal client; ancestry reaches the root |
| `deadline` | Bound until terminal client acquisition | Finite and shown to the operator |

The session environment is attached only to `root_launch`. Dispatch invokes the retained executable and application identifier without re-resolving either.

## Platform Launch Receipt

| Field | Meaning | Validation |
| --- | --- | --- |
| `root_process_id` | Identifier returned by exact child creation | Present for an owned platform root |
| `dispatch_state` | `pending`, `issued`, or `failed` | Monotonic transition |

The receipt is corroborating evidence. The process watcher event and exact profile match establish session role ownership.

## Platform Ownership State

```text
Prepared -> RootStarted -> RootObserved -> TitleDispatched -> ClientAcquired -> Complete
                     \-> PlatformExited
                     \-> WatcherLost
RootObserved -> DispatchFailed
TitleDispatched -> EscapedClient
TitleDispatched -> AmbiguousClient
TitleDispatched -> ClientMissing
Any active state -> Interrupted
```

Only `RootObserved` authorizes title dispatch. Only a client that satisfies exact executable identity and creation-time ancestry may reach `ClientAcquired`.

## Platform Evidence

| Dimension | Positive outcome | Non-positive outcomes |
| --- | --- | --- |
| Root ownership | exact platform role observed | warm, missing, ambiguous, lost, failed |
| Client ownership | exact terminal descendant observed | escaped, ambiguous, missing, lost |
| Routing | final-client traffic reached proxy | launcher-only, platform-only, no traffic, inconclusive |
| Propagation | final-client proxy connection beneath owned root | not-confirmed, not-tested, inconclusive |
| Lifecycle | exact starts and exits by role | unlocalized count and watcher loss |

Routing and propagation are never collapsed. Existing compatibility fact rows retain both dimensions independently.

## Storage

No new database table or schema version is introduced. Existing target rows remain authoritative, compatibility facts remain append-only, and existing session artifacts receive the additional typed observations and terminal reasons through their current versioning rules.
