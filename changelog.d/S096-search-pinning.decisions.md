<!-- spec-impact: none -->

## 2026-08-28 - Pin exact retired-command searches to current guidance

The static search index now promotes the stable current command-reference page
for the exact retired queries `fragcap run` and `fragcap tap`. ZBSearch's `is`
condition already normalizes case and outer whitespace, so two narrow rules
cover the required variants without changing ranking for current terminology.
Historical results stay in the index and follow the promoted guidance.

`zbsearch` 3.3.4 is now a direct site dependency because first-party route code
imports its public pinning API. The package was already present at that exact
version through Fumadocs, so the lockfile gains no package or version. It is
Apache-2.0 licensed, requires Node.js 20 or newer, and is built under the site's
pinned Node.js 24 workflow.
