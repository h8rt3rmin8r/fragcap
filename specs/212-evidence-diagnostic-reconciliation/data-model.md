# Data Model: Evidence and Diagnostic Debt Reconciliation

## Issue disposition

**Fields**: Public issue identifier, observed evidence class, completed work, remaining product behavior, status, labels, milestone, dependency links, and controlled completion test.

**Validation**: Each retained requirement has a product-controllable outcome. A completed comparison is not a future gate. A field result cannot be claimed from source or controlled tests alone. Public wording contains no private target identity, endpoint, traffic, or research path.

**Relationships**: An epic groups child dispositions; a closed child can still be referenced as completed evidence; an independently reproducible defect stays separately open.

## Source disposition

**Fields**: Original tracked path, substantive hunk group, decision (`extract`, `retain privately`, or `retire`), reason, and public destination when extracted.

**Validation**: Every one of the 20 modified tracked paths is represented. A path may have more than one disposition because unrelated hunks can coexist. The public diff is a subset of reviewed extract decisions. Private research content does not enter the public ledger.

## HTTP-head failure count

**Fields**: Existing connection ID, existing terminal code and boundary, optional `http_head_bytes_before_failure` unsigned count.

**Validation**: A present count equals the number of bytes observed by that bounded head read before read error, partial EOF, or size refusal; it does not exceed the configured head bound. Clean zero-byte EOF, complete head, and unrelated errors have no count. The count contains no partial bytes or inferred client reason.

**State transitions**:

| Head-read outcome | Existing terminal behavior | New count |
| --- | --- | --- |
| Clean EOF with zero bytes | Ordinary no-head result | Absent |
| I/O read error before any byte | Existing read-error terminal | `0` |
| I/O read error after partial bytes | Existing read-error terminal | Exact positive count |
| EOF after partial bytes | Existing early-EOF terminal | Exact positive count |
| Configured head limit reached before completion | Existing limit terminal | Exact bounded count |
| Complete head | Existing success path | Absent |
| Other failure, including timeout outside the synchronous read result | Existing terminal | Absent |

## Projection relationship

The proxy runtime owns the observed count. The facade copies it to the same connection ID. Native `proxy.jsonl` and terminal diagnostics JSON conditionally emit the same numeric key. The separately generated bundle debug report reads native `proxy.jsonl` and emits a bounded `proxy.head_read_failures[]` summary only for valid counted HTTP-head failures, carrying connection ID, allowlisted code and boundary, and count. Neither projection creates a second count or a partial HTTP request. Prior records without the field remain readable.

## Finite audit exception

**Fields**: Nine exact advisory IDs (`RUSTSEC-2026-0574` through `RUSTSEC-2026-0582`), affected package (`heapless 0.8.0`), one allowed direct parent (`smoltcp@0.12.0`), expiry (2026-12-15 UTC), mirrored governed-policy records, bounded source-review rationale, positive audit results, negative canary results, and replacement issue #640.

**Validation**: The locked all-features graph and six isolated graphs pass under `cargo-deny 0.21.0`; `unsound = "all"` remains blocking. The offline governed policy pins the same tool, mirrors the exact exception scope, and rejects missing or divergent records. A new direct affected parent, an expired waiver, and policy drift each fail. This state means exception-qualified audit, not repaired dependency or universal safety. The two isolated spike locks resolve `url 2.5.9` and `form_urlencoded 1.2.3`; the product workspace lockfile is unchanged.

**Relationship**: The finite exception unblocks S212 CI only while #640 independently owns removal of the affected dependency path and all temporary exceptions.
