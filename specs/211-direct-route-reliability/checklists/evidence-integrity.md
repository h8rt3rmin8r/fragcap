# Evidence Integrity Checklist: Direct Route Reliability

**Purpose**: Check that route and packet evidence requirements do not confuse their authorities.
**Feature**: [spec.md](../spec.md)

- [x] Does the route contract require one-use receipt, exact process generation, tuple, and demonstrated client transfer for a positive result?
- [x] Does it state how original-present, original-absent, and incomplete packet history differ?
- [x] Does it prohibit inferring client delivery from SOCKS CONNECT or packet consumption alone?
- [x] Does it specify how refused, zero-byte, reused, ambiguous, and lost-event cases remain non-positive?
- [x] Does the helper diagnostic exclude credentials and unrelated payloads while retaining the actual error kind?
- [x] Does it require compatibility, route, terminal result, and manifest to reconcile from the same candidate run?
