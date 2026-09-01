<!-- spec-impact: none -->
Native application-stream validation now waits for accepted records to become readable instead of assuming the asynchronous writer flushes within 20 milliseconds, preventing intermittent Windows CI failures.
