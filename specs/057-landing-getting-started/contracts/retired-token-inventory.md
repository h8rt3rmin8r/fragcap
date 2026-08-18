# Contract: Retired-token inventory (SC-002 gate)

The exact tokens that MUST NOT appear anywhere under `site/` after the slice, and
the files that carry them today. Verified by grep over `site/` plus a site build.

## Forbidden tokens (site-wide)

| Token | What it was |
| --- | --- |
| `fragcap run` | retired capture verb (now `capture`) |
| `fragcap tap` | retired ad-hoc capture verb (now `capture --process`) |
| `fragcap watch` | retired disambiguating verb (now `capture --path`/`--path-regex`) |
| `steam profile` | retired scaffold command (now `targets add --steam`) |
| `profile validate` | retired command (now `schema validate`) |
| `--profile` (as a capture selector) | retired selector (now `--target`) |
| `--profile-dir` | retired flag |
| the AppData `fragcap\profiles` directory | retired profile directory |
| `eso` / `<game-id>` used as a `--profile` arg | profile slug that does not exist |

Note: `--profile` survives ONLY as an internal extcap config option in `cli.rs`;
the CLI reference must not present it as a user selector. The word "profile" as a
concept (schema `kind: "profile"`, the internal capture-config) is not forbidden;
the retired user-facing commands/selector/directory/slug are.

## Carrier files today (from grep)

- `site/app/(home)/page.tsx` - `fragcap run --profile eso` demo; `writing-a-profile`
  link.
- `site/app/(home)/brand/page.tsx` - `fragcap run --profile eso` specimen line.
- `site/content/docs/getting-started.mdx` - `profile dir` doctor row; `fragcap
  steam profile <APP_ID>`; `fragcap run --profile <game-id>`; `writing-a-profile`
  link; "Get a profile" framing.
- `site/content/docs/guides/capture-modes.mdx` - three `fragcap run --profile
  <game-id>` examples; `[capture] mode` profile reference.
- `site/content/docs/guides/writing-a-profile.mdx` - DELETED.
- `site/content/docs/reference/cli.mdx` - `run`, `tap`, `profile`, `steam profile`
  sections; `--profile` flag; `run --profile eso`.
- `site/content/docs/reference/profile-schema.mdx` - DELETED.
- `site/content/docs/index.mdx` - `writing-a-profile` and `profile-schema` links.
- `site/content/docs/architecture.mdx` - `writing-a-profile` link.
- `site/content/docs/meta.json` - `guides/writing-a-profile`,
  `reference/profile-schema` nav entries.

## Acceptable residue (must remain reachable / correct)

- `reference/target-schema.mdx` legitimately uses `profile` as a schema `kind` and
  documents the master schema; it stays and is not a violation.
- The word "profile" in prose describing the schema kind or the internal
  capture-config is acceptable; only the retired user-facing surface is forbidden.

## Verification command (illustrative)

```bash
grep -rniE "fragcap (run|tap|watch)|steam profile|profile validate|--profile[^-]|--profile-dir|writing-a-profile|profile-schema|\\\\fragcap\\\\profiles" site/
# expect: no matches for retired usages; any `profile` hit must be a schema kind
# or internal-type reference on target-schema.mdx / architecture.mdx.
```
