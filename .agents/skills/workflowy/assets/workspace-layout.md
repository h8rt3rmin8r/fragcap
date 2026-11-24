# Workspace layout

Use this structure when the operator asks to establish or upgrade an instance. Reuse the existing root and move retained records in place.

```text
- AI Agent Workspace
  - ABOUT
  - PERSONAL
    - notes
  - PROJECTS
  - REFERENCE
    - Entities
    - Glossary
    - Remote Data
  - STANDARDS
```

## ABOUT

Use these two child bullets. The second is a live external link. This skill is the source of the workspace rules and templates.

- Private workspace for the operator and their authorized agents.
- [Get the latest Workflowy skill](https://github.com/shruggietech/skills/releases/latest)

## PERSONAL

Use `notes` for generic non-project work. Follow the [dated record contract](record-format.md). On an existing installation, an operator-maintained `assets` branch covers personal holdings and dealings such as property, real estate, and money. Preserve operator-maintained branches during upgrades. Start an empty instance with `notes`.

## PROJECTS

Use one branch per named project. Sort by name, ignoring case. Put project-level hashtags under `info > tags` as individual bullet values. Put `info`, `notes`, and where applicable `spec-kit` directly under it. Give `info` exactly three direct children: `tags`, `links`, and `details`. Put primary repository, tracker, documentation, or other verified entry points under `links`. Put all other project context under `details`. Use the [project template](templates/project-outline.md) and [backfill guide](project-backfill.md).

## REFERENCE

Use [the catalog contracts](shared-catalogs.md) for Entities, Glossary, and Remote Data. Sort records within each catalog by preferred name, ignoring case. Use `tags` for each record's hashtags.

## STANDARDS

This is the operator's durable guide to company and cross-project practices. Keep one standard per direct child of **STANDARDS**, sorted alphabetically by title, ignoring case. Use normal Workflowy bullets and reuse the existing record when revising a standard.

Use a `tags` field with one native hashtag per child. Assign the most relevant topic and add another when people would reasonably look under either topic.

- `#standards-collaboration` covers operator intent, coordination, and handoffs.
- `#standards-delivery` covers work scope, specifications, testing gates, and release authority.
- `#standards-evidence` covers sources, verification, attribution, and uncertainty.
- `#standards-tooling` covers tools, storage, artifacts, and data control.
- `#standards-writing` covers prose, presentation, and formatting.

Keep the directive under `rule` for a requirement or `default` for a usual practice. Retain any `scope`, `source`, and `recorded` fields. Keep cited sources and related standards as direct links.

Before governed work, search within **STANDARDS** by title or topic tag. Follow a tag match to its owning standard and read its directive and any scope or linked authority. Apply the standards that govern the task. Record session history under the relevant project or Personal notes. [Workflowy search](https://workflowy.com/help/search-your-notes/)
