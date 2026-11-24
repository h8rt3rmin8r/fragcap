---
name: workflowy
description: Work in a private Workflowy notebook using Desktop MCP. Use to brainstorm, review a discussion, record a project session or Spec Kit slice, backfill a project, or maintain the shared glossary, entities, and references.
disable-model-invocation: false
metadata:
  version: "1.1.0"
---

# Workflowy

Use Workflowy to think with the operator and their authorized agents. Develop ideas, drafts, questions, and disagreements in the relevant discussion. Keep useful reasoning and changes of mind beside the decision they explain. Revise existing prose when asked to edit it. The operator decides what is published. Use the intended signed-in account and destination for private material and keep credentials in their secret store. [Workflowy sharing](https://workflowy.com/help/share-collaborate)

## Route the work

- Discover the MCP tools and confirm access with a scoped read. Desktop MCP requires the Workflowy desktop app running and signed in with Local MCP enabled, plus an agent session connected to its Streamable HTTP connection. The [MCP reference](assets/workflowy-mcp.md) shows scoped seeks, link markup, native dates, and readback checks. [Official setup](https://community.workflowy.com/t/workflowy-desktop-mcp/1695)
- Read **AI Agent Workspace → ABOUT** and follow the operator's branch link when given. Keep one branch per named project under **PROJECTS**, in case-insensitive name order after `_example` and `_general`. Use `_general` for cross-project or non-project discussions. Organize it by topic when useful. Check ancestry before edits.
- If the connector is unavailable, continue the task and tell the operator what remains to be recorded. Provide a draft when the conversation is suitable for the material.

## Project records

- Named projects use `info`, `notes`, and `spec-kit` when applicable. Keep `info` to the purpose, latest supported state, next action, active blockers, and links. Git and the established tracker govern official progress where used. Otherwise use the project artifacts.
- Keep `notes` as one oldest-first timeline. Use one local ISO 8601 timestamp with its real offset, followed by a short fragment, for a major session or substantive update. Structural labels are exempt. Nest updates and replies with the session or discussion. Outcomes, reasoning, questions, links, and tags belong in children.
- Keep all formal slices as direct children of `spec-kit` in code order. Use the exact formal slice code and a short title, such as `S094 Check source order`. Put dates, tags, scope, outcome, and authoritative links in children. Require a `Date` child and, for a slice belonging to an epic, a linked `Epic` child. Use the verified integration or authoritative closeout date, labeled by event, or mark it Pending or Unknown.
- Use one native Workflowy date, `#timeline`, and the primary project hashtag on the same metadata child for indexable records. Add `#slice` for slices. Save a scoped date-range search or export a chronological view using [the date and link methods](assets/workflowy-mcp.md).
- Use the [project setup and backfill guide](assets/project-backfill.md) for Git, folder-only work, or recovery from local agent records. Copy the [project outline template](assets/templates/project-outline.md) as needed.

## Shared catalogs

- **GLOSSARY** owns reusable meanings, methods, protocols, and formats. **ENTITIES** owns named people, organizations, tools, services, models, systems, and datasets. **REFERENCES** owns citable sources, editions, specs, releases, and artifacts. Follow the [catalog rules and worked examples](assets/shared-catalogs.md).
- Resolve identity across all three catalogs before adding a record. Use a single record per referent and give aliases to the preferred record. A matching title or URL is a review cue. Preserve source versions and actual distinctions.
- Keep catalog entries alphabetized by name, ignoring case and hashtags. Put fields under their entry as named child bullets. Use a link to the existing source record when it fits, and retain the exact section or version that supports the claim. Reuse hashtags for browsing and direct node links for specific relationships.

## Verify and continue

Before writing, find the canonical record by meaning and evidence. Update it in place or link to it and record only the change. Preserve stable node IDs. After each batch, read back the affected nodes and verify titles, hierarchy, ordering, link targets, native dates, duplicates, and current state where relevant. Re-read before repeating an uncertain write. A rerun with the same source evidence should add zero records.

## Activate in agent instructions

Paste the following block into the active project `AGENTS.md` or equivalent to opt that project in. The live copy is under **ABOUT → Enable Workflowy for a project**. The [global agent context template](assets/templates/global-agent-context.md) is for user-level instructions covering unrelated or cross-project work.

```markdown
## Workflowy

This project opts into the `workflowy` skill and Workflowy Desktop MCP for
substantive work. Load the skill when available. A running, signed-in desktop
app with Local MCP enabled and a connected agent session is required.

- Read **AI Agent Workspace → ABOUT** and locate the project under **PROJECTS**.
  Reuse its branch or insert it alphabetically. Use **PROJECTS → _general** for
  work without a named project. Confirm the destination before writing.
- Keep named projects in `info`, `notes`, and `spec-kit` when the project uses
  Spec Kit. Keep `info` current and link its claims to evidence. Use `notes` for
  candid drafts, decisions, questions, and sessions in chronological order.
  Start each major session and substantive update title with one local
  ISO 8601 timestamp and its offset, followed by a short fragment. Keep
  updates and replies with their session or discussion.
- Put each formal slice directly under `spec-kit` in code order. Its title
  contains only the formal code and short title, like `S094 Check source order`.
  Add a `Date` child and an `Epic` child when applicable. Put scope, outcome, and
  links in children. Use the native Workflowy date and project hashtag
  together on the timeline child for dated records.
- Find the existing record by meaning and source before writing. Link to
  the exact finding or authoritative Git, tracker, or project artifact.
  Add new evidence or a delta in place and preserve the original node ID.
  Follow ABOUT for shared catalog fields, alphabetical order, and link roles.
- Keep private work with the operator and their authorized agents. The
  operator decides what is published. Read back MCP edits and check titles,
  parents, order, links, dates, and duplicates. If MCP is unavailable,
  continue the work and report what still needs to be saved.
```
