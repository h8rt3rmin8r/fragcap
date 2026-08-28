# Production UX And Accessibility Audit

## 1. Scope And Production Artifact Identity

S094 audits the post-v0.7.0 public site described by issue #249. It records the
production artifact and files corrections separately; it changes no site
behavior. The audited artifact was built from commit
`ceeec93242da399e365ec9b5046a80a54ed74df5` on branch
`codex/094-production-ux-audit`.

The source inventory contains 50 documentation routes and four public home or
information routes. The export also contains the expected `404` and
`_not-found` implementation artifacts. Those two files are not public routes.

## 2. Environment And Exact Commands

| Item | Value |
| --- | --- |
| Date | 2026-08-28 |
| Operating system | Windows NT 10.0.26200.0 |
| Browser | Codex in-app browser, Chromium engine |
| Desktop viewport | 1440 by 900 CSS pixels |
| Narrow viewports | 768 by 900 and 320 by 900 CSS pixels |
| Node.js | v24.11.0 |
| pnpm | 9.15.4 |
| Cargo | 1.96.0 |
| rustc | 1.96.0 |

Commands used from the repository root unless a directory is shown:

```text
cd site
pnpm install --frozen-lockfile
pnpm build

node site/scripts/serve-export.mjs site/out 4174

cargo xtask docs check
cargo xtask docs build
bash scripts/lint-docs.sh link
bash -lc 'shopt -s expand_aliases; alias curl=/mnt/c/Windows/System32/curl.exe; cd /mnt/a/Code/fragcap; set -- link; source scripts/lint-docs.sh'
cargo xtask ci
```

The committed loopback server resolved an exact file, then `<path>.html`, then
`<path>/index.html`, and returned the exported `404.html` otherwise. This avoids
directory-index behavior that the deployed clean-route site does not have.
`pnpm install --frozen-lockfile` and `pnpm build` passed. The lockfile did not
change. The export identified itself as Next.js 16.3.0 and contained both
`.nojekyll` and `CNAME`; `CNAME` contains `fragcap.com`.

## 3. Reconciled Route Inventory

Source pages, generated documentation metadata, exported HTML, and visible
navigation were reconciled. The result is:

```text
expected routes = generated routes = observed routes = 54
documentation routes = 50
home and information routes = 4
not-found probes = 1 (excluded from public route arithmetic)
```

No source-only, export-only, or navigation-only public route was found.

## 4. Route Coverage Matrix

Legend: `P` passed, `F01` through `F06` refer to findings below, and `N/A`
means that the route is outside the documentation-only narrow matrix. `Sem`
is the heading and landmark inspection. `Rules` is the deterministic DOM rule
pass described in section 6. Every route was opened from the production export.

| Route | 1440 | 768 | 320 | Sem | Rules |
| --- | --- | --- | --- | --- | --- |
| `/` | F01 | P | P | F01 | N/A |
| `/brand` | P | N/A | N/A | P | N/A |
| `/disclaimer` | P | N/A | N/A | P | N/A |
| `/license` | P | N/A | N/A | P | N/A |
| `/docs` | F01 | P | P | F01 | F01 |
| `/docs/architecture` | F01, F06 | P | P | F01, F06 | F01, F06 |
| `/docs/changelog` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-2-0/added` | F01, F02 | P | P | F01, F02 | F01, F02 |
| `/docs/changelog/0-2-0/changed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-2-0/decisions` | F01, F02 | P | P | F01, F02 | F01, F02 |
| `/docs/changelog/0-2-0/fixed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-2-0/highlights` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-3-0/added` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-3-0/changed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-3-0/decisions` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-3-0/fixed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-3-0/highlights` | F01, F02 | P | P | F01, F02 | F01, F02 |
| `/docs/changelog/0-4-0/added` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-4-0/changed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-4-0/decisions` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-4-0/fixed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-4-0/highlights` | F01, F02 | P | P | F01, F02 | F01, F02 |
| `/docs/changelog/0-5-0/added` | F01, F02 | P | P | F01, F02 | F01, F02 |
| `/docs/changelog/0-5-0/changed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-5-0/decisions` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-5-0/fixed` | F01, F02 | P | P | F01, F02 | F01, F02 |
| `/docs/changelog/0-5-0/highlights` | F01, F02 | P | P | F01, F02 | F01, F02 |
| `/docs/changelog/0-5-1/fixed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-6-0/added` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-6-0/changed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-6-0/decisions` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-6-0/fixed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-6-0/highlights` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-6-0/removed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-7-0/added` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-7-0/decisions` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-7-0/fixed` | F01 | P | P | F01 | F01 |
| `/docs/changelog/0-7-0/highlights` | F01 | P | P | F01 | F01 |
| `/docs/contributing` | F01 | P | P | F01 | F01 |
| `/docs/getting-started` | F01 | P | P | F01 | F01 |
| `/docs/glossary` | F01 | P | P | F01 | F01 |
| `/docs/glossary/anti-cheat-and-security` | F01 | P | P | F01 | F01 |
| `/docs/glossary/capture-and-networking` | F01 | P | P | F01 | F01 |
| `/docs/glossary/command-line-and-diagnostics` | F01 | P | P | F01 | F01 |
| `/docs/glossary/file-and-wire-formats` | F01 | P | P | F01 | F01 |
| `/docs/glossary/platform-and-distribution` | F01 | P | P | F01 | F01 |
| `/docs/glossary/process-and-attribution` | F01 | P | P | F01 | F01 |
| `/docs/glossary/rust-and-tooling` | F01 | P | P | F01 | F01 |
| `/docs/glossary/windows-internals` | F01 | P | P | F01 | F01 |
| `/docs/guides/capture-modes` | F01 | P | P | F01 | F01 |
| `/docs/reference/cli` | F01 | P | P | F01 | F01 |
| `/docs/reference/deep-capture-compatibility` | F01 | P | P | F01 | F01 |
| `/docs/reference/output-formats` | F01 | P | P | F01 | F01 |
| `/docs/reference/target-schema` | F01 | P | P | F01 | F01 |

All 54 desktop loads had a title, one visible H1, usable shared navigation, and
no console warning or error. All 50 documentation pages kept their footer and
navigation available at both narrow widths. The unknown-route probe returned
HTTP 404, but failed recovery behavior as F05.

## 5. Keyboard And Focus Journey

The browser exposed the desktop and mobile controls as native links and
buttons in a coherent DOM order. The desktop sequence included the brand,
sidebar controls, search, documentation links, current-page table of contents,
theme control, content links, and footer links. There were no positive
`tabindex` values, unnamed visible controls, or disabled controls in the
journey. Search opened as a named dialog with a textbox and a named close
control, and mouse operation showed no trap. Focus styles resolve to the
browser or component outline rules.

The journey fails at its first required surface because there is no skip link,
and documentation pages have no main landmark to receive a skip target (F01).
The in-app browser's native key injection did not advance focus from `body` or
activate a focused native button, so an end-to-end hardware-equivalent Tab,
Enter, Space, and Escape traversal is **not run**. The semantic order and native
element types passed inspection, but keyboard event handling and visible focus
during actual sequential traversal remain unverified. This limitation applies
to desktop and 320 px mobile navigation.

## 6. Semantic And Automated Accessibility Results

Every documentation route received a deterministic, read-only DOM rule pass.
The exact predicates are preserved in `site/scripts/audit-export-dom.mjs`.
With a Playwright-compatible page, the invocation is:

```js
const { auditDocument } = await import('./site/scripts/audit-export-dom.mjs');
const result = await page.evaluate(auditDocument);
```

The run set the page viewport to 1440, 768, or 320 CSS pixels before evaluation
and retained the returned object for the route matrix. The function may also be
serialized into another read-only page evaluator. It checked document language,
visible H1 count, content heading steps, main landmarks, accessible names for
visible buttons and form controls, image `alt` attributes, duplicate IDs, root
horizontal overflow, silently clipped article content, complex-content scroll
containment, and footer visibility. This was a declared local rule set, not axe,
Lighthouse, or a conformance certification.

Passed results:

- All pages declare document language and expose one visible H1.
- No visible button or form control lacks an accessible name.
- No visible content image lacks `alt` text. The two Npcap screenshots have
  specific alternatives.
- No duplicate IDs were found.
- Link text and accessible names identify their destinations in the inspected
  navigation, content, and footer surfaces.
- No root overflow or silent clipping was found at 320 or 768 px.

Failed results:

- All 50 documentation routes lack a main landmark; the homepage nests two
  main elements (F01).
- Seven generated changelog pages jump from H1 directly to H4 (F02).
- Both architecture Mermaid SVGs have a graphics-document role but no
  accessible name (F06).

No native screen reader, speech output, accessibility inspector, axe, or
Lighthouse run was performed. Those omissions limit confidence about
announcements and rules outside the declared DOM checks.

## 7. Responsive, Zoom, Theme, Contrast, And Complex Content

Every documentation page was opened at 320, 768, and 1440 px. Long code lines,
tables, and matrices may exceed their content column, but their immediate
containers use horizontal scrolling. The information remains reachable and the
root page does not overflow. Both architecture diagrams reflow to the 320 px
content width. The Getting Started images scale to the content width, retain
their alternatives, and do not clip. Footers remain reachable on every route.

A 720 CSS-pixel layout was used as the reflow equivalent of 1440 px at 200
percent zoom. The stricter 320 px route matrix also passed reflow and content
reachability. Native browser zoom itself is **not run** because the in-app
browser exposes viewport control but not browser zoom state. This gives strong
reflow evidence but does not verify browser-specific text rasterization or a
zoom-only overlay defect.

Dark-theme direct text on `/docs/getting-started` passed the computed contrast
sample. The home and documentation theme controls switched both themes and
kept controls available. Light-theme normal text failed in shared tokens (F03):

- `rgb(115,115,115)` on `rgb(241,241,241)` is 4.20:1.
- `rgb(115,115,115)` on `rgb(245,245,245)` is 4.35:1.
- `rgb(215,58,73)` on `rgb(241,241,241)` is 4.05:1.

The required normal-text threshold is 4.5:1. No visible button or input was
smaller than 24 by 24 CSS pixels in the sampled routes. A simple computed-style
sample cannot establish all non-text graphical contrast, so forced-colors and
pixel-level diagram contrast are not claimed.

## 8. Search, Internal Anchors, And External Links

| Query | Classification | First useful result | Result |
| --- | --- | --- | --- |
| `packet attribution` | current Capture | Deep Capture compatibility | Pass |
| `capture scope` | current Capture | Architecture | Pass |
| `Deep Capture` | current Deep Capture | Architecture | Pass |
| `proxy-owned TLS key` | current Deep Capture | Output formats | Pass |
| `fragcap run` | retired | v0.5.0 highlights | Fail, F04 |
| `fragcap tap` | retired | v0.5.0 highlights | Fail, F04 |

The export check examined 3,062 internal route references, including 1,789
fragment references. It found zero missing routes and zero missing anchors.
Shared navigation and representative in-content links resolved in the browser.
The repository's external-link surface is `bash scripts/lint-docs.sh link`.
Its first run used WSL `/usr/bin/curl` and failed because that environment could
not resolve `npcap.com`. A direct Windows `curl.exe` request resolved the host
and returned HTTP 200. The same repository link mode was then sourced with
Windows curl substituted for the unavailable WSL network path:

```text
bash -lc 'shopt -s expand_aliases; alias curl=/mnt/c/Windows/System32/curl.exe; cd /mnt/a/Code/fragcap; set -- link; source scripts/lint-docs.sh'
```

That run passed with `OK all external references responded`. The initial DNS
failure is retained here because it was an environment failure observed by the
required check, not a passing result.

## 9. Findings And Dispositions

Before filing, open and closed issues were searched with the phrases `main
landmark skip link`, `nested main homepage`, `heading hierarchy changelog`,
`light theme contrast`, `retired commands search`, `404 recovery navigation`,
and `Mermaid accessible label`. No existing issue owned any finding. Issue #249
and epic #255 were the only broad audit results returned for retired search
terms. Each new issue is in the `Post-v0.7.0 documentation` milestone.

### F01: Primary Landmark And Skip Target Are Inconsistent

- **Severity**: High
- **Route or shared surface**: `/` and all 50 documentation routes
- **Viewport or access mode**: 320, 768, 1440 px; keyboard and landmark navigation
- **Reproduction**: Inspect `main,[role="main"]` and traverse from the first
  focusable element.
- **Observed evidence**: Documentation routes expose zero primary landmarks
  and no skip link. The homepage contains two nested `main` elements.
- **User impact**: Landmark navigation cannot identify documentation content,
  and keyboard users cannot bypass persistent navigation.
- **Disposition**: [Issue #263](https://github.com/h8rt3rmin8r/fragcap/issues/263).

### F02: Generated Changelog Headings Skip Two Levels

- **Severity**: Medium
- **Route or shared surface**: Seven routes named in the route matrix
- **Viewport or access mode**: Semantic outline at all audited widths
- **Reproduction**: Inspect the first content heading after the page H1.
- **Observed evidence**: The next content heading is H4 on each affected route.
- **User impact**: Heading navigation presents a misleading hierarchy.
- **Disposition**: [Issue #264](https://github.com/h8rt3rmin8r/fragcap/issues/264).

### F03: Shared Light-Theme Text Misses 4.5:1 Contrast

- **Severity**: High
- **Route or shared surface**: Shared documentation chrome and prose tokens,
  represented by `/docs/getting-started`
- **Viewport or access mode**: Light theme at 1440 px, with the same tokens at
  768 and 320 px
- **Reproduction**: Select light theme and calculate computed foreground and
  nearest opaque background contrast for direct normal text.
- **Observed evidence**: Ratios of 4.20:1, 4.35:1, and 4.05:1 occur in sidebar
  text, summaries, table-of-contents text, inline code, and syntax tokens.
- **User impact**: Repeated ordinary text has insufficient contrast.
- **Disposition**: [Issue #265](https://github.com/h8rt3rmin8r/fragcap/issues/265).

### F04: Search Foregrounds Retired Commands

- **Severity**: High
- **Route or shared surface**: Global documentation search
- **Viewport or access mode**: Search dialog at 1440 px
- **Reproduction**: Query `fragcap run` and `fragcap tap`.
- **Observed evidence**: The v0.5.0 highlights page is the first result for
  both exact retired names and displays the retired commands in its excerpt.
- **User impact**: A command-oriented search can direct a user first to an
  obsolete interface.
- **Disposition**: [Issue #266](https://github.com/h8rt3rmin8r/fragcap/issues/266).

### F05: Not-Found Page Has No Recovery Navigation

- **Severity**: Medium
- **Route or shared surface**: Any absent route; probe
  `/definitely-missing-s094`
- **Viewport or access mode**: 320 and 1440 px
- **Reproduction**: Request the absent path from the static export.
- **Observed evidence**: HTTP status is 404, but the body contains only `404`
  and `This page could not be found.` It contains zero links and no main.
- **User impact**: A stale or mistyped link leaves the visitor at a dead end.
- **Disposition**: [Issue #267](https://github.com/h8rt3rmin8r/fragcap/issues/267).

### F06: Architecture Diagrams Have No Accessible Name

- **Severity**: Medium
- **Route or shared surface**: `/docs/architecture`
- **Viewport or access mode**: Semantic inspection at 320, 768, and 1440 px
- **Reproduction**: Inspect the two Mermaid-generated SVG nodes.
- **Observed evidence**: Both have `role="graphics-document document"`, but
  neither has an `aria-label` or child `title`.
- **User impact**: A screen-reader user encounters an unnamed graphic.
- **Disposition**: [Issue #268](https://github.com/h8rt3rmin8r/fragcap/issues/268).

All six material findings have exactly one issue disposition. No correction is
implemented in S094. Epic #255 remains open.

## 10. Checks Not Run And Confidence Limits

| Check | Result | Reason and confidence impact |
| --- | --- | --- |
| Hardware-equivalent keyboard traversal | Not run | The in-app browser did not deliver Tab or activation keys to the page. DOM order and native semantics were inspected, but sequential focus, keyboard event handling, and visible focus remain unverified. |
| Native screen reader | Not run | No screen reader session was available. Accessible names and structure are DOM evidence, not speech-output evidence. |
| Native 200 percent browser zoom | Not run | The browser exposes viewport control, not zoom state. A 720 px reflow equivalent and the stricter 320 px matrix passed, but zoom-only rendering behavior is unverified. |
| Forced-colors or high-contrast mode | Not run | The browser exposed neither operating-system mode. Ordinary light and dark computed colors were checked only. |
| axe or Lighthouse | Not run | No audit dependency was present and S094 does not alter the lockfile. The documented deterministic DOM rules are narrower. |
| External screen-reader or mobile device | Not run | The audit used Chromium CSS viewports on Windows. Device browser and assistive-technology differences remain outside this evidence. |

## 11. Gate Results And Conclusion

| Gate | Result |
| --- | --- |
| `pnpm install --frozen-lockfile` | Pass |
| `pnpm build` | Pass |
| `cargo xtask docs check` | Pass |
| `cargo xtask docs build` | Pass |
| `bash scripts/lint-docs.sh link` | Fail because WSL `/usr/bin/curl` could not resolve `npcap.com` |
| `scripts/lint-docs.sh link` sourced with Windows curl | Pass; all external references responded |
| `cargo xtask ci` | Pass |
| Final encoding and diff hygiene | Pass |

The production export is complete and responsive, its internal route and anchor
graph balances, and complex content remains reachable. Six material
accessibility or navigation defects were reproduced and assigned to narrow
milestone issues. The repository gates pass. The disclosed native keyboard,
screen-reader, zoom, and forced-colors checks must not be interpreted as
passes.

## 12. S095 Remediation Evidence

S095 corrected F01, F02, F03, and F06 together and added a blocking Chromium
regression over the built static export. The regression enumerates all 54
public routes at 320, 768, and 1440 px, requires nonempty subject populations,
and verifies the following outcomes:

- F01: every public route has exactly one `main-content` primary region; the
  first focusable control is a visible-on-focus skip link whose activation
  transfers focus without a network request.
- F02: every generated changelog route has a truthful heading sequence, with
  the known `installing-on-windows` destination and links preserved.
- F03: the corrected light muted foreground and light Shiki red both measure
  at least 4.5:1 against their rendered opaque backgrounds.
- F06: the two hydrated architecture SVGs retain their graphics-document roles
  and expose the distinct names `Capture packet attribution architecture` and
  `Deep Capture session architecture`.

F04 and F05 remain open for S096. The limitations disclosed in section 10 also
remain limitations; S095 adds automated Chromium keyboard activation and
computed-style evidence, not native assistive-technology results.
