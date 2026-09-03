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
