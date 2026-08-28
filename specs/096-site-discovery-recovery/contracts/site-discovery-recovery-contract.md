# Contract: Site Discovery And Recovery

The S096 contract binds the production static search experience and exported not-found response.

## Retired command recovery

| Exact query | Current replacement | Promoted page |
| --- | --- | --- |
| `fragcap run` | `fragcap capture` | `/docs/reference/cli` |
| `fragcap tap` | `fragcap capture --process <image>` | `/docs/reference/cli` |

Each exact query MUST produce a nonempty result set whose first activated destination is the current command reference. The current page MUST state the replacement plainly. At least one v0.5.0 changelog group MUST remain visible later in the same result population.

The engine's ordinary case folding and surrounding-whitespace normalization apply to exact matching. No rule demotes all changelog pages or changes broader query behavior.

## Current query baseline

| Query | Required first destination |
| --- | --- |
| `packet attribution` | `/docs/reference/deep-capture-compatibility` |
| `capture scope` | `/docs/architecture` |
| `Deep Capture` | `/docs/architecture` |
| `proxy-owned TLS key` | `/docs/reference/output-formats` |

The result population for every baseline query MUST be nonempty before its leading destination is evaluated.

## Not-found recovery

Every absent-path response tested through the static host MUST satisfy:

```text
HTTP status: 404
primary landmarks: 1
primary selector: main#main-content
visible H1: Page not found
recovery navigation name: Page recovery
recovery destinations: /, /docs/getting-started
root horizontal overflow: false
```

Both recovery links MUST be visible and keyboard reachable at 320 and 1440 pixels. The shared skip link MUST focus `main-content` without issuing another request. Activating each recovery link MUST reach a successful public route.

## Error boundary

The browser gate may ignore only Chromium's failed-resource console diagnostic for the expected main-document 404 whose source URL equals the current absent page. Missing assets, client exceptions, hydration failures, errors from another URL, and errors on successful public routes remain fatal.

## Preservation boundary

The public route inventory remains 54. Generated changelog source and output remain unchanged. The two not-found export artifacts remain outside public-route arithmetic. Capture and Deep Capture runtime behavior does not change.
