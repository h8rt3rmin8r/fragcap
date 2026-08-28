# Data Model: Site Discovery And Recovery

S096 adds no runtime persistence. Its model consists of serialized search rules, ordered browser observations, and static-host recovery observations.

## Search Promotion Rule

| Field | Meaning | Validation |
| --- | --- | --- |
| `id` | Stable identity for one retired-command rule | Unique and tied to one command |
| `pattern` | Exact retired query | `fragcap run` or `fragcap tap` |
| `anchoring` | Query match mode | Exact equality after engine normalization |
| `document` | Current page promoted for the query | Stable page id `/docs/reference/cli` |
| `position` | Requested result position | Zero |

## Search Observation

| Field | Meaning | Validation |
| --- | --- | --- |
| `query` | Phrase entered into the production dialog | One retired or baseline current query |
| `results` | Ordered visible search buttons | Population is nonempty |
| `first_destination` | Path reached by activating the first result | Matches the expected current documentation route |
| `historical_index` | Position of a matching changelog group | Present and greater than zero for retired queries |
| `replacement` | Current instruction on the promoted page | Names the supported replacement command |

## Not-Found Observation

| Field | Meaning | Validation |
| --- | --- | --- |
| `requested_path` | Shallow or nested absent path | Does not collide with an exported artifact |
| `status` | Static-host response status | 404 |
| `viewport` | Browser width | 320 or 1440 pixels |
| `primary` | Recovery content landmark | Exactly one `main#main-content` |
| `heading` | Visible error identity | Exactly one `Page not found` heading |
| `recovery_links` | Ordered destinations inside labelled recovery navigation | Exactly `/` and `/docs/getting-started` |
| `overflow` | Root content width beyond viewport | False |
| `browser_errors` | Uncaught errors other than the expected main-document 404 diagnostic | Empty |

## State Transitions

```text
documentation source -> static search index -> exact query -> pinned current group -> preserved historical groups

absent request -> host candidate miss -> HTTP 404 plus exported recovery body -> homepage or getting-started navigation
```

An empty result population, absent historical group, successful missing response, missing recovery link, or unobserved browser subject cannot transition to pass.
