<!-- spec-impact: none -->

Recorded S081 measurement limitations for issue #204. The local shell was not
elevated, so baseline `fragcap doctor --timings` reported the process event
tracing probe as unavailable without entering the expensive elevated
`EtwWatcher::start` path. `logman query -ets | Select-String
fragcap-doctor-probe` returned no matching session after the run.

The implementation replaces doctor's full watcher readiness probe with
`EtwWatcher::probe_session`, which starts and drops only the ETW session. This
proves the consumer thread and startup process snapshot are no longer on the
doctor readiness path by code structure, but the local non-elevated shell still
cannot provide representative elevated before and after timing.

After implementation, `cargo run -p fragcap-cli --features etw -- doctor
--timings` still exited 1 because the local binary lacked the live backend, and
the tracing check still reported unavailable without elevation. A second `logman
query -ets | Select-String fragcap-doctor-probe` returned no matching session.
