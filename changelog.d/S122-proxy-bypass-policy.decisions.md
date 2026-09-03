<!-- spec-impact: 13.7, 17.2.1, 19, 25, 28.1 -->
Treat bypass as reviewed routing scope rather than proxy loss. Empty policy
inherits nothing, uppercase and lowercase proxy variables are overwritten,
controlled origins stay proxy-routed, and every proxied DNS answer is checked
again by local-destination policy on every attempt. Wildcard-all, malformed,
ambiguous, and listener-colliding rules refuse before effects. S122 adds no
dependency, system proxy mutation, transparent fallback, or Deep Capture
completion claim.
