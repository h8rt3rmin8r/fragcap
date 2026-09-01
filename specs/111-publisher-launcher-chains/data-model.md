# Data Model: Managed Publisher-Launcher Chains

## PublisherChainLaunch

An immutable managed-launch variant prepared from one exact stored target.

| Field | Meaning | Validation |
| --- | --- | --- |
| `root` | Exact direct launch for the publisher launcher | Canonical explicitly stored absolute path, or canonical relative path contained beneath the game install root |
| `stages` | Ordered declared stages | At least launcher and client; unique role per stage |

The root receives the authorized child environment. Descendants are created by the publisher software and inherit according to ordinary operating-system process semantics.

## PublisherStage

One exact declared role in the chain.

| Field | Meaning | Validation |
| --- | --- | --- |
| `role` | Stable profile role | Non-empty, unique, first is `launcher`, last is `client` |
| `executable` | Stored executable identity | Non-empty Windows launch entry; exact absolute paths may name external publisher installs, while relative paths must remain beneath the game install root |
| `arguments` | Root arguments when applicable | Parsed once during preparation; descendant arguments remain match evidence only when already declared in profile vocabulary |
| `parent_role` | Required prior ancestor | Absent for root; otherwise names the immediately prior declared role |
| `terminal` | Whether exit ends the session | True only for `client` |

## PublisherChainProfile

The validated Capture profile synthesized from `PublisherStage` values.

- The root stage matches its exact canonical executable path and image.
- Each later stage matches its exact canonical executable path and image, and descends from the prior role.
- Matching precedence is descendant-first even though the immutable launch plan remains root-to-client, so several roles may reuse one executable.
- The client is the sole terminal stage.
- All stages use session lifecycle so transient intermediates remain observable without becoming the stop authority.
- In a publisher session, one process may bind each role. A competitor produces explicit ambiguity and is not promoted.
- In a publisher session, the acquisition deadline stays active until the terminal stage binds. An omitted publisher `--wait` defaults to two minutes.
- Ordinary profiles retain multi-process roles and watching-only acquisition timeout semantics.

## PublisherChainState

Preflight classification derived from the stored declaration and query-only running-process inventory.

| State | Meaning | S111 action |
| --- | --- | --- |
| `cold` | No declared stage image is already running | Permit preparation as `publisher-launcher-cold` |
| `launcher-warm` | Declared launcher is already running and a declared client is also present or ownership is otherwise not clean | Side-effect-free refusal |
| `game-start-clean-warm` | Launcher is running and client is absent | Side-effect-free refusal, retained distinctly for issue #309 |
| `ambiguous` | More than one structural interpretation or running candidate exists | Side-effect-free refusal |

## Runtime stage dispositions

The existing process tree and CaptureSession produce runtime evidence.

| Disposition | Meaning |
| --- | --- |
| `matched` | One observed creation-time process satisfies one declared stage |
| `absent` | No observation satisfied the stage before the deadline |
| `ambiguous` | Competing observed candidates could not be reduced exactly |
| `escaped-tree` | A candidate appeared outside the session root's creation-time ancestry |
| `unmatched` | An observed descendant belongs to no declared stage |

No runtime disposition rewrites the stored chain automatically in S111.

## State transitions

```text
stored target
  -> structural validation
  -> cold inventory check
  -> immutable Capture and publisher launch preparation
  -> authorization
  -> journaled proxy, trust, and route effects
  -> arm process observation and Capture
  -> execute exact root
  -> bind declared descendants by creation-time ancestry
  -> correlate terminal client sockets
  -> bounded stop, cleanup, and bundle reconciliation
```

Any validation or cold-inventory failure stops before authorization and effects. Any runtime ambiguity or escape remains inconclusive and cannot become compatibility success.
