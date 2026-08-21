# Changelog fragments

`CHANGELOG.md` is assembled from the fragments in this directory at release
time. **Never edit `CHANGELOG.md` from a feature branch.**

The reason is mechanical rather than stylistic: every pull request that touches
the same few lines at the top of a changelog conflicts with every other
concurrent pull request. One file per change removes the shared line entirely.

## Naming

```text
changelog.d/<key>.<section>.md
```

`<key>` identifies the change. Use the issue number, the slice identifier, or a
short slug, whichever is most recognizable later:

```text
42-loopback-detection.added.md
S06-pcapng-writer.added.md
drop-counter-overflow.fixed.md
```

`<section>` is one of:

| Section | Use for |
| --- | --- |
| `added` | New capability |
| `changed` | A change to existing behavior |
| `deprecated` | Capability slated for removal |
| `removed` | Capability removed |
| `fixed` | A defect corrected |
| `security` | A change with a security consequence |
| `decisions` | A dated decision worth surfacing at release: always a change to a pinned artifact, and also an architecture or dependency call a later reader would otherwise have to reconstruct |

One change can produce several fragments. A slice that adds a feature and fixes
a defect found along the way writes two files.

## Spec impact

Every fragment's first line is a machine-readable `spec-impact` field, an HTML
comment naming which specification sections the change modified:

```text
<!-- spec-impact: none -->
<!-- spec-impact: 3.3, 23.1, 27.3 -->
```

The value is either `none` or a comma-separated list of specification section
numbers. It is required on every fragment, checked by `cargo xtask spec`, and it
is stripped before assembly so it never reaches `CHANGELOG.md`.

It exists so a release cannot claim a specification change that did not happen.
At release assembly, if any fragment names a section, `cargo xtask changelog
--release` requires `docs/fragcap-specification.md` to have changed in the
release diff, and refuses otherwise. A fragment that touches no specification
section carries `none`. See constitution principle P-11.

## Content

The body is the changelog entry itself: one or two sentences, present tense,
written for someone who does not know the implementation. No heading, no
bullet, no trailing blank line beyond the single required newline.

```markdown
Loopback interfaces are now detected and offered by `fragcap doctor`, which
names the required npcap installation option when loopback support is absent.
```

Not this:

```markdown
### Added
- Fixed the thing in `src/detect.rs` where the `Vec` was wrong
```

Write what changed for the user, not what changed in the diff.

## Decisions

The `decisions` section is different: it records a dated decision rather than a
change to behavior. Lead with the date.

A decision to change a **pinned artifact** (`.github/workflows/**`,
`rust-toolchain.toml`, `release.toml`, `scripts/**`, release documentation)
**must** be recorded here; the constitution requires it.

That is the mandatory case, not the only one. An architecture call, a dependency
addition or refusal, or a filed request deliberately declined also belongs here
when a later reader would otherwise have to reconstruct the reasoning from a
diff. The repository already depends on this: the architecture narrative in
`AGENTS.md` sends readers to the S05, S08, and S10 decisions fragments for the
ambiguity rule, a reversal, and two dependency arguments, none of which changed
a pinned artifact.

What does **not** belong here is reasoning that lives fully in the slice's own
`specs/` directory and is not worth a release note. A fragment that only
restates `spec.md` is duplication, and duplication is how two records start
disagreeing.

```markdown
**2026-08-06** Pinned the toolchain to 1.82 to match the minimum supported
version declared in the workspace manifest.
```
