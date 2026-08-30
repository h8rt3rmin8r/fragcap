<!-- spec-impact: none -->
Native controlled-proxy validation now waits for connection workers to finish before checking their final HTTP and HTTPS observations, preventing intermittent Windows CI failures.
