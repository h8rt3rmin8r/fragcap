# Data Model: Filter Manager Install Acknowledgement

**Slice**: 016 | **Date**: 2026-08-10

No stored data. One field added to a private struct, one method added to
`FilterManager`, and one message type on a thread channel.

## `HandleState` (changed, `fragcap-core::filter`, private)

```text
struct HandleState {
    installed: Installed,          // committed only on a success ack now
    last_install: Option<Instant>, // set when an install is issued (gates reinstalls and retries)
    gapped: BTreeSet<Endpoint>,    // cleared only on a success ack now
    retired: bool,
    pending: Option<BTreeSet<Endpoint>>,  // NEW: an issued, unacknowledged program
}
```

- `pending` is `Some` from the moment `poll` issues an install until an
  acknowledgement arrives. While `Some`, `poll` issues no new install for the
  handle (one in flight at a time), which makes a bare `(handle, ok)`
  acknowledgement unambiguous.

## `FilterManager::poll` (changed behavior, same signature)

`poll(&mut self, wanted, now) -> Vec<Install>` no longer sets `installed`,
`last_install` is set at issue, and `gapped` is not cleared here. Per handle, when
settled and wanted is non-empty and the handle is not retired:

1. If `installed == Narrowed(wanted)`: skip (idempotent).
2. If `pending.is_some()`: skip (one install in flight).
3. If within `min_reinstall_interval` of `last_install`: skip (rate limit / retry
   spacing).
4. Otherwise: set `pending = Some(wanted)`, `last_install = Some(now)`, and push
   the `Install`.

The gap-accounting loop is unchanged; it reads `installed`, which is now the truly
installed program.

## `FilterManager::acknowledge` (new)

```text
pub fn acknowledge(&mut self, handle: usize, installed_ok: bool)
```

- Takes the handle's `pending` (returns if none: a stale or duplicate ack is
  ignored).
- On `installed_ok`: `installed = Narrowed(pending)`, `gapped.clear()`.
- On failure: leaves `installed`, `last_install`, and `gapped` unchanged, so the
  handle keeps its prior program and the next eligible poll retries.

## `FilterManager::retire` (changed)

Also clears `pending` (a handle whose capture thread ended has no install in
flight to await).

## Install acknowledgement message (new, `fragcap-core::pipeline`, private)

`(usize, bool)` = `(handle_index, installed_ok)`, sent by a capture thread over a
shared `std::sync::mpsc::Sender` after each `set_filter`, drained by the control
thread each poll. Not a public type; it lives on the channel between the two
threads inside `Pipeline::run`.

## `acquire` (changed signature, `fragcap-core::pipeline`, private)

Gains the handle index and a clone of the acknowledgement sender, and after
installing the latest filter sends `(handle, result.is_ok())`.
