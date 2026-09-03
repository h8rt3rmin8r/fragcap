<!-- spec-impact: 13.7, 17.2.1, 19, 25, 28.1 -->
Treat bypass as reviewed routing scope rather than proxy loss. Empty policy
inherits nothing, uppercase and lowercase proxy variables are overwritten,
controlled origins stay proxy-routed, and every proxied DNS answer is checked
again by local-destination policy on every attempt. Wildcard-all, malformed,
ambiguous, and listener-colliding rules refuse before effects. S122 adds no
dependency, system proxy mutation, transparent fallback, or Deep Capture
completion claim.

The initial exact-versus-suffix DNS distinction was removed after review.
Conventional `NO_PROXY` consumers can treat a bare domain as including its
descendants, so the typed policy now assigns that same domain-boundary meaning
to bare and leading-dot inputs instead of authorizing a narrower rule than the
child can enforce.

Proxy-retained evidence cannot see traffic that went direct. Its bypass count
is therefore null with an explicit unavailable state, while successful,
refused, infrastructure, and undetermined proxy outcomes reconcile from each
retained terminal observation. Proxy silence is never converted into zero.
