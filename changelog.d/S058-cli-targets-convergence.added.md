<!-- spec-impact: 14.5, 17 -->

### CLI targets convergence: default `--db` and extcap by target selection (slice S058)

Two parts of the `fragcap-cli` surface now speak the targets model the rest of the
tool adopted in S054/S055.

The explicit `targets` subcommands (`add`, `show`, `remove`, `export`, `import`,
`list`) no longer require an explicit `--db`. When it is omitted they resolve the
same default local store the bare `fragcap targets` hero command uses (the
`FRAGCAP_LOCAL_DB` override, else the per-user default), so `fragcap targets add
--steam <app_id>` and its siblings run with no store path, exactly as the listing
does. An explicit `--db` still overrides, and a subcommand that must open a store
with no resolvable location fails with a named error rather than a panic. This
resolves the manual-registration path the S057 getting-started guide had to defer to
the reference (issue #157).

The Wireshark extcap capture path now selects a stored **target** instead of
resolving a retired profile file. The stored-target resolution that `capture` uses
is extracted into one shared implementation (`commands/target_resolve.rs`), and
`extcap` calls it: its configuration dialog presents a target selector (a handle, a
name, or a row index), resolved against the local store exactly as `capture
--target` resolves it, so the analyzer dialog and the command line select capture
identically. The extcap control grammar is otherwise unchanged (interfaces, link
types, the four-option config block, and FIFO streaming), so unmodified Wireshark
still drives fragcap; only the meaning of the number=0 selection option changes from
a profile reference to a target selector. No code path in the extcap capture handler
resolves a profile file through the search-path or bundled-set cascade any longer.
The S057 CLI-reference "legacy" callout is removed and the converged options are
documented (issue #156).

The extracted `target_resolve` seam takes its inputs explicitly rather than from the
`capture` argument struct, so both commands share one resolution body; it is also
the single place the follow-up launch-and-observe slice (S059, issue #152) extends.

`fragcap-cli` only: no change to `fragcap-core`, the pipeline, attribution, or the
capture orchestrator's behavior, and no new dependency or `Cargo.lock` delta. The
extcap capture golden is regenerated to reflect the synthesized stored-target
profile (a single `target`-role stage, as `capture` synthesizes), with the
conservation identity and packet count unchanged.
