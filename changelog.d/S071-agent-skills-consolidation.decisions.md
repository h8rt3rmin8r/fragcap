<!-- spec-impact: none -->
**2026-08-22** Slice S071 consolidated the vendored agent skills, closing issue
#197. The decisions worth keeping:

- **`.github/workflows/ci.yml` gained a step**, `cargo run --package xtask --
  skills`, scoped to the ubuntu leg beside the existing wrapper gate. That file
  is a pinned artifact, which is why this entry is mandatory rather than
  optional. The step is what makes the new check binding rather than advisory;
  a gate that only runs when someone remembers is the state that produced this
  slice.

- **One upstream, and an admission test.** `.agents/skills/` now carries the
  ShruggieTech house standards this constitution binds, from
  <https://github.com/shruggietech/skills> (Apache-2.0), and nothing else. A
  skill is admitted only if a named principle binds this repository to it or a
  repository gate executes it. Before this, 35 of 36 entries carried
  `source: "h8rt3rmin8r/eso-weave"`, an unrelated personal repository, and 33 of
  the 36 had no reference anywhere outside their own directory. The prior
  selection rule was constitution P-1, which is an admission filter: it
  establishes that a skill teaches nothing denylisted and says nothing about
  whether this project has any use for it. That is how a capture-the-flag
  forensics playbook, twelve skills whose names collide with an agent's own
  built-ins, and a 283-file third-party Rust rulebook came to be presented as
  vetted house standards.

- **`traffic-analysis-pcap` was dropped despite looking on-topic**, and the
  initial keep set was wrong to admit it. It is a security-forensics playbook
  (credential harvesting, TLS decryption, covert-channel detection,
  DNS-tunneling heuristics) that routes to four skills never vendored here, so
  four of its own cross-references were broken. fragcap writes pcapng and
  attributes flows; it has no capture-analysis surface this serves. Name
  adjacency is not domain relevance.

- **`shruggie-docs`, `shruggie-graph-memory`, and `shruggie-html` were declined**
  though they are in the same upstream and the same brand as the four kept. No
  principle binds them and no gate runs them. Admitting on brand alone would
  have restated the failure this slice corrects, with a different brand than
  last time.

- **Vendored content is copied in unmodified, and that is not fussiness.** The
  2026-08-06 vendoring hand-edited its copies to satisfy the text hygiene rules
  in `CONVENTIONS.md`, and three lock hashes (`rust-skills`, `shruggie-html`,
  `shruggie-powershell`) never reproduced afterwards. Because nothing verified
  the file, that sat unnoticed for the life of the project. An edited vendored
  copy is no longer the standard it claims to be. Current upstream ships clean
  under `CONVENTIONS.md` already, so the edit bought nothing even at the time.
  All three divergences are now resolved: two of those skills were dropped, and
  the third was re-vendored from canonical bytes. Recording this because a
  reader comparing lock files across the slice will see three anomalies vanish
  at once and should know it was a consequence rather than a tidy-up.

- **The lock's entry schema was left alone.** The release tag is carried inside
  the existing `source` string (`shruggietech/skills@v1.11.0`) rather than in a
  new field, because the four-field schema is written by an external tool this
  repository does not contain and cannot test against.

- **The hash algorithm is reproduced empirically, and the gate does not use it.**
  `computedHash` is SHA-256 over every file in the skill directory sorted by
  relative path, hashing the path bytes then the content bytes, with CRLF
  normalized to LF for text and binary content hashed raw. That was derived by
  probing against known-good entries, not from a specification, because the tool
  that writes the file is absent here. This slice found the first independent
  confirmation it was right: `shruggie-speckit`'s content was already identical
  to upstream, and recomputing its hash reproduced the committed value exactly.
  Three things still bound the risk. The gate asserts structure and never reads
  hashes, so a wrong value cannot produce a false pass. The integrity anchor for
  what was vendored is the publisher's own `SHA256SUMS.txt`, verified before
  extraction: `shruggie-bash` `0351b062c0453f7f514ce41afc066accdea3e84e08738fa2ade934dc860d8071`,
  `shruggie-markdown` `a68a25bf1f6287726d47cdcef5195bf2eef960172f7c48295e9e30f63426fa17`,
  `shruggie-powershell` `1e043b5ebbfe0573aedfae0eb1bdaad96ca75d47ad6dc7a0a83623a7c577e246`,
  `shruggie-speckit` `9c00c1db77930721ae37cc00ad0389beea968c062ded8caa151407b9e7ecb22c`.
  And the falsifier is cheap: if the external skills CLI ever regenerates the
  file and disagrees, the CLI is right and these values are replaced.

- **The gate asks git's index, not `.gitignore`.** Assertion three compares a
  directory walk against `git ls-files -z`. Reparsing ignore rules would
  reimplement precedence this repository has already been caught getting wrong,
  and the index is the thing that actually determines what a clone receives.
  When git is absent or the lock cannot be read, the gate exits 2 (could not
  run) and never degrades to a pass, matching `neutral` and `msrv`. This gate
  exists because an unverified file drifted for the life of the project, so a
  gate that cannot read the file must say so.

- **The gate does not verify hashes, deliberately.** Given the algorithm's
  empirical provenance above, a hash check could fail against correct content,
  and a gate that cries wolf is a gate somebody disables. Structure was the
  cheap, deterministic half that catches every drift class actually observed
  here, including the one nothing caught for the life of the project.

- **`xtask` gained no dependency.** The check reads `skills-lock.json` with a
  strict reader for the subset it needs rather than taking `serde_json`, which
  would have added no package to `Cargo.lock` but would have ended xtask's
  zero-dependency state for two string fields. The reader refuses anything it
  does not recognize, and a refusal is exit 2 rather than a guess.

- **Wiring the newly vendored Bash checker into `cargo xtask wrappers`, to
  replace the hand-rolled `check_bash`, was deliberately declined here** and
  filed instead as issue #199. It was measured on 2026-08-22 to pass every Bash
  script in `scripts/` cleanly, ShellCheck included, so the swap is small. It
  changes what
  the gate enforces rather than what this slice consolidates, and a slice whose
  subject is removal should not quietly widen a gate on its way past.

- **The refreshed PowerShell checker was measured before it was adopted.** The
  copy that gates continuous integration had drifted 48 lines (POSIX twin) and
  141 lines (PowerShell original) from upstream, and replacing a live gate's
  checker is the one step in this slice that could have turned CI red. Both the
  old and the new checker were run against `scripts/Invoke-FragCap.ps1` before
  any file was touched; output and exit code were identical. Vendoring was then
  ordered before deletion so the window in which the gate has no checker never
  opened.

**Review round on pull request #200 (Codex, 2 findings, both verified and
fixed).** Both were in the new gate, and both were cases of it failing at
precisely the thing it was written to do.

- **The working-tree/index comparison ran one way only.** It asserted that every
  file on disk is tracked, and never the inverse, so a vendored file deleted
  from the working tree without `git rm` left the index carrying a file the tree
  no longer had, and the gate passed. Reproduced before fixing, and it was
  slightly worse than reported: the gate did not merely stay silent, it printed
  "all 31 vendored file(s) are tracked by git", reporting a silently smaller
  count as agreement, which is the P-9 failure this gate exists to prevent. Now
  checked in both directions. Making it symmetric required making the two views
  cover the same paths: `speckit-*` is now filtered out of the index view as
  well as the disk walk (otherwise all ten would report as absent on every run,
  exactly as the finding anticipated), and a file sitting loose in
  `.agents/skills/` is now collected by the walk so it appears in both.
- **The lock reader accepted trailing content after the document.** `value()`
  stopped at the closing brace and nothing required end of input, so a lock
  followed by a second value or by garbage parsed to the leading object and the
  gate passed. Reproduced before fixing (exit 0 where the module's own contract
  says 2). The reader now requires end of input, and trailing whitespace alone
  is still accepted. This one contradicted the module's documented reason for
  being strict, which was the argument for hand-rolling the reader rather than
  taking `serde_json`; a strict reader that is not strict is a worse answer than
  the dependency would have been.

Both fixes carry tests, and both original defects were reproduced against the
real tree before the fix and confirmed corrected after, along with a regression
check that the original present-but-untracked case is still caught.
