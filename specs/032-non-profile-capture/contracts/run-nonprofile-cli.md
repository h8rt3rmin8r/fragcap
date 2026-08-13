# CLI Contract: `run` non-profile inputs

## Command

```text
fragcap run (--profile <REF> | --install-dir <PATH> | --steam <APP_ID>) [capture options...]
```

- Exactly one of `--profile`, `--install-dir`, `--steam` is required (a clap
  `ArgGroup`). Supplying more than one, or none, is a usage error (exit 2).
- All existing `run` capture options (`--out`, `--sink`, `--mode`, `--duration`,
  `--wait`, `--max-packets`, `--max-bytes`, `--roles`, `--direction`,
  `--interface`, `--loopback`, `--no-payload`, offline flags) apply to every
  input unchanged.

## Behavior by input

- `--profile <REF>`: unchanged. Resolves the reference through the cascade; the
  profile provider answers; captures with the backing profile. Output is
  byte-identical to before this slice.
- `--install-dir <PATH>`: resolves the cascade over `PATH` as the request's
  install root. The engine rule / walker name the socket-holding client (or
  runtime observation resolves it). On a non-profile answer, `run` synthesizes a
  one-stage `heuristic-unverified` identity from the resolved target and captures
  it through the shared launch-agnostic engine.
- `--steam <APP_ID>`: resolves `APP_ID` to its install directory via the Steam
  library lookup, then behaves exactly as `--install-dir` over that directory. A
  not-installed app id fails (exit 1) naming the missing title.

## Failure contract

- Usage misuse (zero or multiple of the three inputs): exit 2, clap usage
  message, before any resolution.
- A resolved-but-declined install location (unrecognized layout, ambiguous
  layout, unreadable tree): exit 1, message naming the reason from the resolver's
  notes; nothing captured.
- A `--steam` app id not installed, or a Steam lookup error: exit 1, surfaced
  message; nothing captured.
- An install directory path that does not exist / is not a directory: exit 1,
  surfaced message naming the path.
- A synthesized identity that fails profile validation (an empty or malformed
  predicate set): exit 2 (a profile diagnostic), the same as an authored profile.

## Fidelity

- The synthesized identity is `heuristic-unverified`, never `authored`.
- The synthesized game identity is a generic placeholder; `--steam` additionally
  records the app id as `game.app_id`.

## Constraints

- No process handle opened, no process memory read; capture reaches the target
  through the same launch-agnostic, attribution-from-outside engine `watch` uses
  (P-1).
- The already-running target is attached via the shared startup-snapshot fold, no
  new acquisition mechanism (P-1, FR-010).
