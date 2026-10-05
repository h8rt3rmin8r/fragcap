# Route Owner Release Contract

1. A session with no managed launch attempt records `route-owner: not-needed` and continues cleanup.
2. After an attempted managed launch, the release adapter repeatedly takes complete query-only process inventories until all declared route-owning images are absent or the authorized release budget expires.
3. The proxy remains started and session trust remains installed during this interval. The operator receives a normal-exit instruction at the first observed owner.
4. Snapshot failure never counts as absence. Deadline expiry records `route-owner: failed`, identifies the unresolved condition, and gives normal-exit plus fresh-launch recovery guidance before proxy stop.
5. Proxy shutdown follows the release result. Terminal cleanup is partial if release failed, even if proxy and trust cleanup themselves succeeded.
6. The controlled harness and prelaunch failures do not wait for unrelated running applications.
