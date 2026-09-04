<!-- spec-impact: 26.3, 28.1 -->
Replace PID-only native session ownership with a generation-specific Windows
synchronization lease held for the exact adapter lifetime. A reused PID cannot
make abandoned residue active, and Doctor opens no target process handle.

Narrow confirmed Deep Capture repair to the existing exact journal recovery
plan and exact abandoned owner-record retirement. Recognized sidecars and
manifests are not deletion authority, so healthy retained evidence remains.
Wrong-store, mismatched, malformed, active, and ambiguous state stays visible
instead of triggering broad cleanup. Packaging and archive validation remain
issue #329.

Legacy owner records have no generation lease. A complete terminal journal is
retired automatically; any other legacy record remains unproven at startup and
requires confirmed Doctor repair before its exact journal plan and exact owner
record can be retired. A confirmed header-only crash prefix is terminalized
before that owner authority is retired. Inventory canonicalizes every scan root
and bundle identity before ownership matching. Native check names use session
and resource identity, with a stable bundle-derived fallback when an unsupported
journal version cannot supply a session identity, rather than list position.
