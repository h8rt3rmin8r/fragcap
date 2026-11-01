# Research: S186 Installed Deep Capture truth

## R1: Public CA health and private signer proof

S185 put a read-only health check on every CLI entry. The current `health_at` calls `ordinary_file_presence` and `symlink_metadata` on the protected signer before reading public identity or trust. The required ACL can deny both calls to an ordinary account, yielding `unreadable`, no fingerprint, and repair advice despite successful elevated signing. Decision: derive the public identity, cutover, validity, and machine-root verdict independently of protected signer access. Treat denied signer metadata as unverified, not failed; preserve a confirmed missing or invalid signer when actually observable. The elevated pre-effect path still loads and proves the key.

## R2: HTTPS tunnel authority

`http1::request_destination` currently assigns port 80 to a bare origin-form Host and creates an `http://` URL. It receives no tunnel context. `runtime::serve_http` later requires the parsed authority to equal CONNECT. This is a code-level mechanism for a false `tls-tunnel-authority-mismatch` when an HTTPS tunnel on port 443 carries a normal bare Host. The private five-failure report does not reveal its actual authorities, so controlled tests must establish the mechanism without claiming that every field failure had this cause. Decision: make the parser's scheme and effective default port explicit for tunneled requests. Preserve observed headers and refuse genuine distinct authorities unless independent target-scope and verified-upstream handling is demonstrated.

## R3: Header timeout

`http1::read_head` reports `http-header-timeout` when a bounded header read expires. The field report has one such terminal, which could precede an inner authority. Decision: test it as a separate read-stage outcome, retain the deadline, and add stage-specific context only where the current terminal evidence cannot distinguish the read.

## R4: Acceptance authority

The repository closes implementation issues on scoped controlled and hosted gates; real-title runs are optional operator-owned evidence on already published bytes. This slice can reproduce the six-terminal pattern synthetically and prove honest artifact conclusions. It cannot claim the uninspected private title now succeeds. A patch release is the next owner decision after this PR is reviewed and merged.
