# Research: Site Discovery And Recovery

## Decision 1: Pair current migration guidance with exact-query search pinning

**Decision**: Add one concise migration section to the current command reference that maps retired `fragcap run` to `fragcap capture` and retired `fragcap tap` to `fragcap capture --process <image>`. Configure the static advanced search index with two exact-query pinning rules that promote the stable `/docs/reference/cli` page document to position zero for those retired queries.

**Rationale**: Production-equivalent measurement returned 60 results for each query and placed `/docs/changelog/0-5-0/highlights` first. Adding realistic current prose, headings, repeated notes, or separate per-command notes left the historical page's relevance score substantially higher. The search engine's public pinning API survives static serialization and client loading, matches exact queries case-insensitively after trimming surrounding whitespace, and promotes a stable page identifier while leaving all historical matches in the result set.

**Alternatives considered**:

- Rely on authored current prose and ordinary relevance scoring. Rejected because direct measurement showed it remained below the dense historical page and would make correctness depend on incidental score changes.
- Remove retired terms or changelog pages from the index. Rejected because issue #266 requires historical context to remain searchable.
- Globally demote changelog results. Rejected because history may be the best result for unrelated searches and a global rule would exceed the two-query defect.
- Build a custom search dialog and reorder client results. Rejected because the index already exposes a query-specific promotion primitive and duplicating the dialog would expand the UI surface.

## Decision 2: Promote the installed search engine to a direct dependency

**Decision**: Add exact `zbsearch` 3.3.4 to the site's production dependencies and import its public `insertPin` API from the static index route.

**Rationale**: Fumadocs already resolves and ships exactly this package and version, so the direct edge adds no package to the lock graph. Depending explicitly on the API used by first-party source avoids reaching through a package manager's transitive layout. The package is Apache-2.0 and its Node.js 20 minimum is below the pinned Node.js 24 site build.

**Alternatives considered**:

- Import an internal Fumadocs or package-manager path. Rejected because neither is a public ownership boundary.
- Reimplement pinning against serialized index bytes. Rejected because it duplicates search-engine behavior and risks corrupting the index.
- Use a version range. Rejected because the direct edge must stay aligned with the version Fumadocs currently resolves and the site already exact-pins framework dependencies.

## Decision 3: Use the stable root not-found convention inside the existing layout

**Decision**: Add `site/app/not-found.tsx` as a server component with one `main#main-content`, existing brand assets, one explanatory heading, and a labelled recovery navigation containing only `/` and `/docs/getting-started`.

**Rationale**: Root `app/not-found.tsx` handles unmatched URLs and renders inside the existing root layout, which already supplies global styles, theming, the first-focus shared skip link, and the root provider. A direct primary element avoids nested landmarks. Static links and absolute asset paths work for deeply nested missing URLs without client state or hydration variance.

**Alternatives considered**:

- Use the experimental global not-found convention. Rejected because it bypasses the root layout and would duplicate document, theme, provider, and skip-link setup.
- Wrap the page in the home or documentation layout. Rejected because those components own route-group chrome and complicate the exactly-one-primary invariant.
- Add the shared footer or extra navigation. Rejected because the recovery contract calls for two clear destinations and unrelated links dilute that surface.

## Decision 4: Keep HTTP status ownership in the static host

**Decision**: Leave `site/scripts/serve-export.mjs` and Next export configuration unchanged. The custom page replaces exported `404.html` content; the host continues to return status 404 only after exact file, payload, `.html`, and index candidates fail.

**Rationale**: The existing loopback server already models the deployed static-host contract and returns the custom body with 404 for shallow and deeply nested absent paths. A page component cannot and should not convert an absent static resource into a successful response.

**Alternatives considered**:

- Redirect absent paths to the homepage. Rejected because it erases the missing-resource signal.
- Return a successful branded fallback. Rejected because it creates a soft 404.
- Change server candidate resolution. Rejected because current resolution is correct and S096 changes only fallback content.

## Decision 5: Extend the existing hydrated production regression

**Decision**: Add table-driven search and not-found cases to `site/tests/production-accessibility.spec.mjs`. Exercise the real search dialog, require nonempty result populations, activate the leading result, locate preserved changelog groups, verify current query leaders, request shallow and nested missing paths, assert the response status, inspect semantics and overflow, activate the shared skip link, and follow both recovery links.

**Rationale**: The existing Playwright suite already runs after the production export in continuous integration and fails on page or console errors. Source inspection cannot prove serialized search pinning, result order, client activation, host status, hydration, or computed overflow.

**Alternatives considered**:

- Add only unit tests around rule objects. Rejected because a correct object can still fail to serialize, load, group, or activate through the deployed client.
- Create a second browser suite. Rejected because it would duplicate server startup, browser installation, and error handling.
- Change the documentation workflow. Rejected because it already runs the extended suite after the export build.

## Decision 6: Ignore only Chromium's expected main-document 404 console diagnostic

**Decision**: The browser error fixture ignores a console error only when its source URL equals the current page URL and its text is Chromium's failed-resource message for status 404. Every route expected to succeed still asserts status 200, every missing route asserts status 404, and all other console and page errors remain fatal.

**Rationale**: Chromium emits a console error for the intentionally missing main document even when the custom page and host behavior are correct. Treating that expected transport diagnostic as an application defect would make valid 404 testing impossible. Matching both the main-document URL and exact 404 condition prevents missing assets or unrelated application errors from being hidden.

**Alternatives considered**:

- Disable console error capture for not-found tests. Rejected because hydration and asset failures must remain visible.
- Accept every failed-resource console message. Rejected because it would hide broken assets.
- Assert the page without navigating to an absent URL. Rejected because that would not prove HTTP status behavior.

## Implementation evidence: failing baseline

The pre-implementation production accessibility run retained all eight S095
checks and failed all four new S096 checks. Exact `fragcap run` search still put
the 0.5.0 Highlights page first, both missing-path widths lacked
`main#main-content`, and the current-query harness exposed a brittle search
button selector after navigation. The selector was tightened to Fumadocs'
stable search data attributes before production code changed. The remaining
failures therefore describe product behavior rather than test interaction.
