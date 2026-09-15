# S151 Data Model

## Probe Observation

Fixed ProbeName identifies coarse phase or readiness leaf. Begin carries actual monotonic start instant; completion carries measured duration. One serial worker emits at most sixteen queued fixed-size events. Two active state slots cover parent plus leaf; leaf completion resumes the parent. Waiting is a caller-thread derived diagnostic, not a readiness verdict or persistent report field.

## Timing

One second is anomaly threshold and minimum repeat interval. Active start instants, not event receipt times, supply elapsed milliseconds. Queued completions are processed before a timeout diagnostic. Explicit timings cover all completions; automatic timing covers slow ones. No deadline is introduced.

## Published Identity

Schema version 1, semantic release version, exact forty-character lowercase hexadecimal source commit, UTC publication date and canonical official release URL define actual publication. A finite source-defined inventory of thirteen unique relative contained readable current surfaces defines required baseline markers (not configurable by the publication record). Candidate workspace version is deliberately independent.

## Failure Transitions

Work starts, remains active, completes and joins. External pending stays active. Failed diagnostics preserve facts. Worker panic disconnects, joins and propagates. Coordinator panic drops receiver before scoped joining, so sends fail without deadlocking. Identity or applicability mismatch fails the currency gate and never silently rewrites authority.
