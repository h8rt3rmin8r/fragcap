# Contract: Session Owner Lease

1. A session creates one opaque bounded lease identity before native effects.
2. On Windows it creates and retains one current-session named synchronization
   object for the exact adapter lifetime.
3. The owner record is atomically written beneath `session-owners` with schema
   version, canonical bundle, diagnostic PID, and lease identity.
4. Doctor validates record size, schema, identity grammar, absolute canonical
   bundle reachability, duplicates, and the live named object. The exact record
   is the authority for an operator-selected custom bundle root.
5. A live matching object proves the session generation active. A missing
   object proves only that the registered generation is no longer active.
6. PID liveness or image name cannot upgrade a missing lease to active.
7. Read-only probing may open and close the synchronization object. It creates
   no object and opens no process.
8. Non-Windows runtime reports native Deep Capture unsupported. Controlled
   tests inject lease state without a platform side effect.
9. The lease identity is never a capability credential and is not rendered in
   normal Doctor output.
