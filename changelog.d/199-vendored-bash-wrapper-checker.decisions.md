<!-- spec-impact: 18.4, 24.3 -->
Decision: `cargo xtask wrappers` uses the vendored `shruggie-bash` checker as the single Bash compliance authority, and keeps fragcap-specific syntax, help, and dry-run seam checks in `xtask`. `scripts/lint-docs.sh` keeps the standard `log_info` and `safe_run` fixtures with narrow ShellCheck suppressions because the house Bash standard requires those fixtures even though this linter does not currently call them.

Rationale: S071 vendored the Bash standard checker after `xtask` already carried a Rust structural checker. Keeping both would let the repository gate drift from the standard agents are instructed to follow. The direct checker still warns when ShellCheck is absent, but the CI gate now preflights ShellCheck from inside Bash and exits 2 if it cannot run static analysis. Deleting the unused fixtures from `lint-docs.sh` would make the script less compliant with the standard this slice is enforcing, so the suppression is the narrower pinned-script change.

Alternatives considered: Keeping `check_bash` would preserve the duplicate authority issue; running both checkers would hide rather than remove the drift risk; modifying the vendored checker would exceed this slice and change shared skill bytes.

Applies-To: 0.6.0
