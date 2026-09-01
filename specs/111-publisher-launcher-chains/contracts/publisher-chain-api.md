# Contract: Publisher-Chain Managed Launch

## Stored value contract

A publisher chain uses the existing target entry's ordered `launch_entries` array. Each Windows-applicable member provides:

```json
{
  "executable": "Publisher/Launcher.exe",
  "arguments": "optional stored argument fragment",
  "os": "windows",
  "role": "launcher"
}
```

Required role sequence:

1. `launcher`
2. Zero or more unique intermediate role names
3. `client`

The existing single-entry `client` shape remains a direct launch. A multi-entry value becomes a publisher chain only when the complete role contract is valid. Unknown, missing, duplicate, or reordered roles are refused rather than reinterpreted.

An exact absolute executable path is canonicalized as stored and may name a publisher installation outside the selected game's install root. A relative executable path is resolved only beneath the canonical game install root and refuses any escape. No publisher-directory search or fallback is permitted.

## Public preparation contract

The facade exposes side-effect-free preparation that returns the existing `ManagedLaunch` value:

```rust
pub fn prepare_managed_launch(
    target: &TargetEntry,
) -> Result<ManagedLaunch, LaunchConfigError>;
```

The result is either:

- `ManagedLaunch::Direct` for one exact client entry;
- `ManagedLaunch::Publisher` for one exact launcher-rooted chain; or
- a complete preparation error with no process or routing effect.

`ManagedLaunch::with_environment` applies values only to a direct process creation owned by the value. For a publisher chain it modifies the root launch and leaves the declared descendant identities unchanged.

`ManagedLaunch::execute` starts only the exact root and returns its process identifier when the platform supplies one. It never invokes or builds a shell command.

## Shared Capture profile contract

Stored target resolution returns one validated profile for both Capture and Deep Capture. For a publisher chain:

- every declared role becomes one profile stage;
- the launcher stage has no ancestry predicate;
- each later stage names the prior role through `descends_from`;
- only the client is terminal;
- every stage retains exact executable matching.

## Deep Capture policy contract

`publisher-launcher-cold` joins the supported launch cases only after the exact chain and current compatibility facts both pass. `publisher-launcher-warm` and `publisher-launcher-game-start-clean-warm` remain stable refusals. A generic `publisher-launcher` value remains unsupported because it does not state the observed cold or warm condition.

## Error contract

Preparation errors remain side-effect-free and distinguish at least:

- missing install root;
- missing publisher launcher;
- missing terminal client;
- duplicate role;
- unknown or empty role;
- multiple roots or terminal clients;
- invalid stage order;
- relative executable escaping the install root;
- invalid stored arguments;
- already-running launcher;
- already-running descendant;
- platform unsupported.

Errors name the stored target and competing roles or executable images where safe. They never choose one candidate implicitly.
