<!-- spec-impact: none -->

Fixed `fragcap doctor` doing full ETW watcher startup for one process-event
tracing readiness boolean by using a session-only runtime probe.
