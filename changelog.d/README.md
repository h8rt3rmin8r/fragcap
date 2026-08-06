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
| `decisions` | A dated decision to change a pinned artifact |

One change can produce several fragments. A slice that adds a feature and fixes
a defect found along the way writes two files.

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

The `decisions` section is different: it records a dated decision to change a
pinned artifact (`.github/workflows/**`, `rust-toolchain.toml`, `release.toml`,
`scripts/**`, release documentation), which the constitution requires. Lead
with the date.

```markdown
**2026-08-06** Pinned the toolchain to 1.82 to match the minimum supported
version declared in the workspace manifest.
```
