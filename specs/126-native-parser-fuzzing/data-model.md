# Data Model: Native Parser Fuzzing

## Surface Registry

- `schema_version`: closed integer version
- `product_scope`: reviewed shipped boundary
- `max_input_bytes`: global hard cap
- `toolchain`: exact nightly channel
- `cargo_fuzz_version`: exact installed runner version
- `libfuzzer_sys_version`: exact harness dependency version
- `targets`: unique target records
- `dependency_boundaries`: third-party parser ownership records

## Target Record

- `id`: exact fuzz binary name
- `owner`: `fragcap-proxy` or `fragcap`
- `corpus`: repository-relative corpus directory
- `dictionary`: optional repository-relative libFuzzer dictionary
- `surfaces`: nonempty list of surface records
- `properties`: required transition, bound, and round-trip behavior
- `ci_runs`: fixed smoke iteration budget
- `timeout_seconds`: fixed per-input timeout

## Surface Record

- `id`: unique stable kebab-case identifier
- `authority`: source module and production parser or state machine
- `input_kind`: bytes, text, JSONL stream, JSON document, or transition sequence
- `max_retained_bytes`: finite allocation or retained-evidence cap
- `states`: state or outcome vocabulary exercised by the target

## Dependency Boundary

- `crate`: exact direct dependency name
- `owns`: wire or syntax parsing explicitly not claimed by S126
- `fragcap_surface`: surrounding metadata or state boundary S126 does exercise

## Seed Case

- Stored under the target's corpus directory.
- Filename is descriptive, unique, and stable.
- Contents are nonempty, synthetic, no larger than the global cap, and tracked.
- Identical content may not appear twice within one corpus.

## Validation Invariants

- Target, surface, corpus, and dependency identifiers are unique and nonempty.
- Every target source, corpus, optional dictionary, stable replay dispatch, and
  CI matrix entry exists exactly once.
- Every corpus is nonempty, all seeds are tracked, and forbidden content is absent.
- Every cap and campaign bound is positive and no larger than its global policy.
- Exact tool versions agree across registry, fuzz manifest, workflow, and guide.
