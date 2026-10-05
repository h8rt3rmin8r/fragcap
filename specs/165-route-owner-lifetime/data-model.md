# Data Model: S165 Route Owner Lifetime

## Route owner set

A derived, in-memory set of case-insensitive image names from the exact authorized managed launch: Steam root plus declared title entries, one direct client, or every declared publisher stage. No target process environment or executable image is read. The set is discarded after session cleanup.

## Route-owner release result

One cleanup result precedes proxy stop. `Released` means a complete query-only inventory contained none of the names after launch was attempted. `NotNeeded` means managed launch was never attempted or the controlled harness used no routed child. `Failed` means a process remained or inventory could not establish absence before the finite release deadline. A failed result includes normal-exit and fresh-launch recovery guidance and prevents a complete terminal outcome.

## Deadline

The route-owner release duration is an immutable authorized session-plan field in milliseconds, separate from launch, observation, shutdown, and cleanup. It has a finite maximum and a two-minute default. The proxy shutdown clock starts after the release attempt returns.
