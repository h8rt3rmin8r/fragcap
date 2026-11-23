# Shared catalogs and links

## Choose a record

| Catalog | Record | Required fields | Optional fields |
| --- | --- | --- | --- |
| GLOSSARY | Reusable concept, protocol, method, or format | Definition | Aliases, Scope, Source, Broader |
| ENTITIES | Named person, organization, software, service, model, dataset, or system | Type, Summary | Role, Aliases, Homepage, External ID, Source, Maintained by |
| REFERENCES | Citable source, edition, specification, release, or artifact | URL, Use when | Type, Publisher, Source ID, Version, About, Supersedes |

A named language or platform ecosystem can have an entity record when it has its own tools, maintainers, and identity. Put abstract semantics in GLOSSARY and independently citable editions in REFERENCES. Use `Person`, `Organization`, `Software`, `Service`, `Platform`, `Language`, `Model`, `Dataset`, or `System` as entity types. Put detail such as *Go library* or *build tool* in `Role`. For existing references, retain meaningful source-kind and authority detail.

Keep direct records alphabetized by name, ignoring case and trailing native hashtags. Use aliases on the preferred record. Search all three catalogs by name, alias, authoritative ID, and source link before creating anything. Shared titles and URLs are leads to check against meaning. Preserve distinct versions, sections, and real name collisions. Keep entity homepages and evidence links in their own fields. Use absolute external URLs in REFERENCES. Keep machine-local evidence with its owning project.

## Link with purpose

Use one target per named child, with a stable Workflowy node link where the target exists. Let Workflowy generate the backlink. Use hashtags for topic discovery. Keep repeatable fields as separate child bullets.

- `Source` connects a definition or description to supporting material. Reuse the existing reference when it fits. Keep the precise section or commit as a locator when it carries the evidence.
- `About` connects a reference to its subject. A single reference can have multiple About children.
- `Maintained by` connects software or a service to its documented maintainer.
- `Broader` connects a glossary concept to a more general concept. Use it for an actual conceptual hierarchy.
- `Supersedes` connects a source to the record it actually replaces. Editions may coexist for citation.

For example, **WebVTT** is a glossary format concept. Its `Source` points to the **W3C WebVTT specification** in REFERENCES. **Cueson** is an entity. **Cueson WebVTT format contract** is a separate reference with `About` links to WebVTT and Cueson. The standard and implementation contract serve different lookup needs. Use the [copyable record templates](templates/shared-records.md).

## Reconcile and export

Update the canonical bullet in place. When merging verified duplicates, preserve meaningful aliases, unique evidence, and inbound navigation. Keep retired node IDs discoverable through a redirect or an old-to-canonical mapping. If an existing project links to an old node outside the current edit scope, preserve that old node as a forwarding record outside the catalog. Workflowy-generated backlink excerpts are views of incoming links. Exclude them from record counts and exports.

Keep publisher-issued IDs in `Source ID`, scoped by `Publisher` or known issuing authority. Preserve exact editions and source locators. Start graph integration as a projection of the outline. Keep Workflowy node IDs as origin keys and persist a portable identifier for each canonical record when first syncing. Export labels, types, source IDs, and typed links independently. Reuse the persisted IDs across exports and derive reciprocal views from the original relation. Keep the source-of-truth and audience boundaries explicit.

- [SKOS for labels, scope and concept relationships](https://www.w3.org/TR/skos-reference/)
- [Dublin Core Metadata Terms for source, subject, publisher and version](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/)
- [W3C persistent IDs and versioning guidance](https://www.w3.org/TR/dwbp/#DataIdentifiers)
- [Wikidata guidance on merging identities](https://www.wikidata.org/wiki/Help:Merge)
