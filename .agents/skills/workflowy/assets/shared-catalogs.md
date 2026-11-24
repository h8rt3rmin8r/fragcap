# Shared catalogs and links

All three catalogs live under **REFERENCE**.

## Choose a record

| Catalog | Record | Required fields | Optional fields |
| --- | --- | --- | --- |
| Glossary | Reusable concept, protocol, method, or format | `definition` | `aliases`, `scope`, `source`, `broader` |
| Entities | Named person, organization, software, service, model, dataset, or system | `type`, `summary` | `role`, `aliases`, `homepage`, `external_id`, `source`, `maintained_by` |
| Remote Data | Citable source, edition, specification, release, or artifact | `links`, `use_when` | `type`, `publisher`, `version`, `about`, `supersedes` |

A named language or platform ecosystem can have an entity record when it has its own tools, maintainers, and identity. Put abstract semantics in Glossary and independently citable editions in Remote Data. Use `Person`, `Organization`, `Software`, `Service`, `Platform`, `Language`, `Model`, `Dataset`, or `System` as entity types. Put detail such as *Go library* or *build tool* in `role`. For existing references, retain meaningful source-kind and authority detail.

Keep direct records alphabetized by preferred name, ignoring case. Use a clean title. For each tagged record, put a single `tags` field directly beneath the record and give each native hashtag its own bullet beneath it. Use aliases on the preferred record. Search all three catalogs by name, alias, authoritative ID, and source link before creating anything. Shared titles and URLs are leads to check against meaning. Preserve distinct versions, sections, and real name collisions. Keep entity homepages and evidence links in their own fields. In Remote Data, use `links` for absolute external URLs, with one distinct target per child. A source-specific identifier may appear in the label of a link it identifies. Keep imported codes out of separate fields. Keep machine-local evidence with its owning project.

## Link with purpose

Use the Workflowy bullet format for each standalone field label and every nested value. Nest one value or target per child. Use stable Workflowy node links where the target exists. Let Workflowy generate backlinks and use hashtags for topic discovery. Put multiple values under the same field label.

- `source` connects a definition or description to supporting material. Reuse the existing reference when it fits. Keep the precise section or commit as a locator when it carries the evidence.
- `about` connects a reference to its subject. Put multiple targets in separate bullets beneath one `about` field.
- `maintained_by` connects software or a service to its documented maintainer.
- `broader` connects a glossary concept to a more general concept. Use it for an actual conceptual hierarchy.
- `supersedes` connects a source to the record it actually replaces. Editions may coexist for citation.

For example, **WebVTT** is a glossary format concept. Its `source` points to the **W3C WebVTT specification** in Remote Data. **Cueson** is an entity. **Cueson WebVTT format contract** is a separate reference with `about` links to WebVTT and Cueson. The standard and implementation contract serve different lookup needs. Use the [copyable record templates](templates/shared-records.md).

## Reconcile records

Update the canonical bullet in place. When merging verified duplicates, preserve meaningful aliases, unique evidence, and inbound navigation. Relink affected sources to the canonical record before removing verified duplicates. Workflowy-generated backlink excerpts are views of incoming links. Exclude them from record counts and exports.

Preserve exact editions and source locators. Start graph integration as a projection of the outline. Keep Workflowy node IDs as origin keys and persist a portable identifier for each canonical record when first syncing. Export labels, types, source URLs, and typed links independently. Reuse the persisted IDs across operator-directed exports and derive reciprocal views from the original relation. Keep the source-of-truth and audience boundaries explicit.

- [SKOS for labels, scope and concept relationships](https://www.w3.org/TR/skos-reference/)
- [Dublin Core Metadata Terms for source, subject, publisher and version](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/)
- [W3C persistent IDs and versioning guidance](https://www.w3.org/TR/dwbp/#DataIdentifiers)
- [Wikidata guidance on merging identities](https://www.wikidata.org/wiki/Help:Merge)
