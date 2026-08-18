# Quickstart / Validation Guide: S057

Runnable checks that prove the slice is done. Local build/test uses the GNU
toolchain; CI runs the MSVC gate.

## Prerequisites

- Repo on branch `057-landing-getting-started`.
- Rust GNU toolchain `1.96.0-x86_64-pc-windows-gnu` for local build/test.
- `bash` available (for `cargo xtask docs check` and grep). `pnpm` optional (for
  `cargo xtask docs build`).

## 1. Doctor companion change (US4)

```bash
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli
```

Expected: green. The doctor tests assert the identity rows are `version, binary,
catalog db, local db` (no `profile dir`) and that no `Profiles` section exists.

Optional manual check (Windows, any privilege - doctor captures nothing):

```bash
cargo +1.96.0-x86_64-pc-windows-gnu run -p fragcap-cli -- doctor
```

Expected: the Identity section has no `profile dir` row; there is no `Profiles`
section; every other section and the exit status are unchanged.

## 2. Retired-token grep (SC-002)

```bash
grep -rniE "fragcap (run|tap|watch)|steam profile|profile validate|--profile-dir|writing-a-profile|profile-schema" site/
grep -rn "\-\-profile\b" site/            # only the internal extcap config option may remain in prose about extcap, not as a capture selector
grep -rniE "run --profile|steam profile <" site/
```

Expected: no matches for retired usages. Any surviving `profile` hit is a schema
`kind` or internal-type reference on `target-schema.mdx` / `architecture.mdx`, or
the word in ordinary prose about the schema, never a retired command/selector.

## 3. No dangling links to deleted pages

```bash
grep -rn "writing-a-profile\|profile-schema" site/
```

Expected: no matches (nav, inbound links, and the landing page all rerouted).

## 4. Getting-started endpoint (SC-001, manual read)

Read `site/content/docs/getting-started.mdx` as a literal first-time reader.
Confirm:

- Every command is runnable at the point it appears (binary installed, releases
  link present, Administrator instruction present before `doctor`).
- The prerequisites (npcap/Wireshark) are acquired in "Before you begin," framed
  conditionally.
- "Find a target" uses `fragcap targets`, defines a Steam App ID inline, and offers
  a non-Steam path.
- The last step yields `capture.fcapng` on disk.
- The doctor sample matches the corrected binary output.

## 5. Landing page (US2, manual read / build)

Confirm the settled opening sentence and two paragraphs appear verbatim, the worked
example is a `fragcap targets` listing, the dependency diagram is present, there is
one primary CTA, and none of the section-23.1 prohibitions appear.

## 6. Full gate

```bash
cargo +1.96.0-x86_64-pc-windows-gnu xtask docs check     # glossary / P-6 linter
# CI (MSVC) runs the authoritative gate:
cargo xtask ci
```

Expected: docs check passes; `cargo xtask ci` is green (fmt, clippy, workspace
tests, lint, deps, license, docs check). If `pnpm` is available, `cargo xtask docs
build` produces the static export with no broken internal links.

## 7. QA issue closure (SC-006)

Confirm each of #130, #131, #132, #133 (docs half), #134, #135 is resolved by a
concrete change in the guide (see the mapping in tasks.md), so they can be closed
as the slice lands.
