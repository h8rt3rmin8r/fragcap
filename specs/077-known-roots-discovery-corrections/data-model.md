# Data Model: Known-Roots Discovery Corrections

## `ClassifierVerdict`

| Variant | Meaning | Walker action |
| --- | --- | --- |
| `Hit` | One target directory | Emit one candidate and stop descent. |
| `Container` | More than one distinct engine product observed | Emit no candidate; descend when depth permits. |
| `Miss` | No target classification | Emit no candidate; retain existing miss accounting and descent behavior. |

`Container` is ephemeral discovery control. It is not a target classification, candidate identity, or stored row.

## `DiscoveryAccount`

Two outcomes are added:

| Field | Meaning |
| --- | --- |
| `container_descended` | A classified container was not emitted and its immediate children were enumerated because depth permitted. |
| `container_descent_truncated` | A classified container was not emitted and the depth bound prevented enumerating its children. |

The conservation equation becomes:

```text
produced + parse_failed + declined_by_user + considered_not_a_game
  + container_descended + container_descent_truncated
  + volume_skipped + access_error == considered
```

The aggregate discovery driver sums both fields exactly as it sums every existing outcome. The CLI account line renders both by stable names.

## Container Classification

Input is the complete finding vector already produced for one bounded directory scan. Filter to engine-category findings, collect unique product values, and choose:

```text
distinct engine products >= 2 -> Container
distinct engine products == 1 -> Hit at detected engine fidelity
distinct engine products == 0 and known-root prior -> Hit at heuristic fidelity
distinct engine products == 0 and no prior -> Miss
```

Coverage state and warnings remain separate from the verdict. A partial scan can return `Container` from what was observed while still reporting that coverage was incomplete.

## Canonical Candidate Path

`KNOWN_ROOTS` remains a list of slash-separated relative paths. The generic walker joins it in the existing fixture-compatible form. On Windows, `FsDirectoryLister` converts that input to native separators exactly before `read_dir`; children returned from that lister are used unchanged for both `CandidateIdentity::Path` and `install_root`. Non-Windows hosts pass the slash-native input through unchanged.

No case folding, absolute-path resolution, link resolution, or historical-row rewrite occurs.
