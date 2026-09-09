# Contract: Discovery Integrity CLI and Library Behavior

## Automatic Registration

The hero command and Doctor discovery action call one library operation that:

1. evaluates every produced candidate exactly once;
2. retains an exhaustive decision reason;
3. registers only eligible candidates through the existing target-entry path;
4. returns a conserved registration account; and
5. never modifies or removes an existing target.

Authoritative platform identities remain eligible without engine evidence. Path identities require verified engine-category title evidence.

## `fragcap targets discover`

Detailed mode remains non-persistent and renders:

```text
SOURCE  IDENTITY  FIDELITY  AUTO     NAME
steam   steam:42  ...       eligible Example
...
  automatic registration: authoritative platform identity
known-roots ...             refused  ...
  automatic registration: insufficient title evidence

Discovery account:
  considered: ...
  produced: ...
  ...
Automatic registration account:
  eligible: ...
  refused: ...
```

Exact spacing remains under the existing table renderer. Values are never truncated or wrapped.

### `--summary`

`fragcap targets discover --summary` renders counts only. It MUST NOT render:

- catalog or local store paths;
- candidate identities or Steam application ids;
- title or directory names;
- install roots, evidence strings, warning paths, accounts, hostnames, or volume identities.

It exits with the same success/failure semantics as detailed discovery and retains counts for warnings and every existing account outcome.

## `fragcap targets reconcile`

Grammar:

```text
fragcap targets reconcile [--db <local.db>] [--steam-root <path>] [--yes]
```

- `--db` follows the existing local-store precedence when omitted.
- `--steam-root` supplies the exact Steam installation for deterministic testing or non-default installations; otherwise ordinary Steam discovery is used.
- Without `--yes`, the command is preview-only.
- With `--yes`, the command recomputes the preview and atomically removes exactly the displayed removable rows.
- An empty plan is a successful no-op.
- A changed or missing preview row is a refusal and leaves every row unchanged.

Preview rows include stable id, handle, and one stable reason. Preserved rows are summarized by reason. The command never infers ownership from a handle, display name, folder name, or path spelling.

## Library Ownership Boundary

- `fragcap-targets` owns automatic-registration decisions, conserved registration accounts, reconciliation planning, and transactional exact deletion.
- The `fragcap` facade owns construction of platform inventory from Steam metadata and re-exports target-domain values.
- `fragcap-cli` owns flags, human rendering, exit mapping, and confirmation presentation.

No new crate dependency edge is permitted.
