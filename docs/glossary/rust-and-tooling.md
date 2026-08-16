# Rust and Tooling

## Applies-To

A field in the master specification's document-control block naming the released
software version the document describes. It tracks the workspace package version
and is bound to it by `cargo xtask spec`, so the specification and the shipped
artifact cannot silently drift.

{: .matters }
> The specification is the architecture of record agents are told to trust. When
> it described v0.2.0 as the first functional release while v0.4.0 had shipped,
> every decision built on it inherited a false baseline. Applies-To is the
> anchor the lock-step check binds, making constitution principle P-11
> mechanical rather than remembered.

**See also:** [spec-impact](rust-and-tooling.md#spec-impact),
[xtask](rust-and-tooling.md#xtask)

## MSRV

**Also known as:** minimum supported Rust version

The oldest toolchain release a crate compiles with, declared as
`rust-version` in its manifest.

Distinct from the toolchain a project builds with. The first is a compatibility
promise to consumers; the second is a reproducibility control for the project.

{: .matters }
> fragcap declares 1.82 while pinning its build toolchain at 1.96.0. A
> declared minimum that is never exercised is an unverified claim, so a
> dedicated check builds at the minimum. That check is currently vacuous, since
> the workspace has no external dependencies, and it says so in its own output.

**See also:** [xtask](rust-and-tooling.md#xtask)

## spec-impact

A field on each `changelog.d/` changelog fragment, written as a leading
`<!-- spec-impact: ... -->` comment, whose value is either `none` or a list of
specification section numbers the change modified. The release gate reads it to
refuse a release whose fragment claims a specification change the release diff
does not contain.

{: .matters }
> It closes the reverse of specification drift: a change that claims it touched a
> section without the section actually changing. The value is machine readable
> and stripped before the changelog is assembled, so the claim is checked
> without cluttering the published changelog.

**See also:** [Applies-To](rust-and-tooling.md#applies-to),
[xtask](rust-and-tooling.md#xtask)

## Test tier

One of the four levels specification section 25.2 defines, distinguished by
what each needs in order to run.

| Tier | Needs | In continuous integration |
| --- | --- | --- |
| 0, unit | nothing | yes |
| 1, pipeline | nothing | yes |
| 2, platform | privilege and a capture driver | yes, on a Windows runner |
| 3, live | privilege, a driver, and a game | no, manual |

{: .matters }
> Tier 1 is the one the architecture was shaped to make possible. Because a
> [replay source](process-and-attribution.md#replay-source) and a [scripted
> attributor](process-and-attribution.md#scripted-attributor) substitute for the two platform-dependent
> seams, the whole pipeline is testable on any machine with no privilege. That
> is the return on keeping capture and attribution apart.

**See also:** [Fixture corpus](capture-and-networking.md#fixture-corpus),
[Replay source](process-and-attribution.md#replay-source)

## xtask

A Cargo convention in which repository-wide tasks are implemented as a
workspace member invoked through a cargo alias, requiring nothing installed
beyond the language toolchain.

{: .matters }
> fragcap's conventions and dependency-direction checks live here rather than
> in shell scripts, because a check written in the project's own language can
> be unit tested against known-bad input. A linter whose matcher never fires is
> indistinguishable from a clean repository.
