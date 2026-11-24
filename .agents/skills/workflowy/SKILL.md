---
name: workflowy
description: Work with the operator and their agents in their private Workflowy workspace. Use for candid collaboration, project notes, Spec Kit records, personal notes, applicable company standards, shared references, project onboarding, backfills, or repair of legacy Workflowy records.
disable-model-invocation: false
metadata:
  version: "1.3.1"
---

# Workflowy

Use the operator's private workspace to develop ideas and drafts, share candid feedback, and work through decisions with the operator and their authorized agents. The operator decides what is published. Verify the account and destination before writing. Keep credentials in their secret store. [Workflowy sharing](https://workflowy.com/help/share-collaborate)

## Find the workspace

- Discover an authorized connector and confirm access with a scoped read. Workflowy Desktop MCP requires a running, signed-in desktop app, Local MCP enabled, and a connected client. A cloud agent can use a separately configured, operator-authorized connector if available. When access is unavailable, continue under the available instructions and supply an appropriately private, ready-to-paste nested outline with its intended destination. Mark it unsynced. [Official Desktop MCP setup](https://community.workflowy.com/t/workflowy-desktop-mcp/1695)
- Use the layout and exact ABOUT bootstrap in [Workspace layout](assets/workspace-layout.md). Find a named project by its owner-confirmed link or under **PROJECTS**, in case-insensitive alphabetical order. Route generic non-project notes to **PERSONAL > notes**. Search the flat **STANDARDS** list by title or topic tag before governed work. Read each applicable standard's `rule` or `default` and any `scope`.
- Use normal bullets for discussion, drafts, evidence, and updates. Never save records as Workflowy comments or comment drafts. Check the existing record first and add a link or a delta when the finding already has a home.

## Write records

- Named projects use `info`, `notes`, and `spec-kit` when applicable. The only direct children of `info` are `tags`, `links`, and `details`. Under `details`, keep purpose, current state, next actions, blockers, and supporting evidence.
- Use lowercase letters and underscores for reusable field names, such as `current_state` and `date_basis`. Keep established section names and distinct record titles. Format exact technical literals as inline code. Use the [MCP example](assets/workflowy-mcp.md#write-inline-code) to write native code formatting.
- Put dated records directly under `notes`, in oldest-first order. The title contains only its ISO 8601 date or timestamp. Put substantive follow-ups at the same level and link to the prior record. Require `title`, `tags`, and `details`. Write field labels on their own bullets with values as children. Use the explicit Workflowy bullet format for every field and value. Only content inside `details` may use another format. Use one tag, link, or distinct item per child. The [record contract](assets/record-format.md) defines precision, optional fields, ordering, and the slice format.
- Place one formal slice directly under `spec-kit` in code order. Use the formal code and short title. Put the verified date and documented epic in child fields. Use the [project outline template](assets/templates/project-outline.md).
- Keep record and project names free of inline tags. Store hashtags as individual bullet values under `tags`. For a named project, put its project-level tags in `info > tags`. Record source dates and useful scope tags. The operator owns any timeline construction. Routine recording excludes timeline fields or tags, saved searches, views, and exports.
- Use [catalog guidance](assets/shared-catalogs.md) for **REFERENCE > Entities**, **Glossary**, and **Remote Data**. Look up identity first, sort by name, and use direct links for specific relationships. Use absolute external URLs and local evidence references sparingly.

## Backfill, repair, and verify

- Use [project setup and backfill](assets/project-backfill.md) for new, existing Git, and folder-only projects, including recovery from local GPT, Claude, and Ollama records.
- Use [legacy remediation](assets/legacy-remediation.md) to upgrade older note layouts, slice metadata, and comment-based records. Read [Workflowy MCP methods](assets/workflowy-mcp.md) for scoped seeks, patches, links, and date handling.
- Preserve stable node IDs whenever possible. Read back each patch and check titles, ancestry, order, precision, links, and duplicates. Re-read before retrying an uncertain write. The same evidence should produce zero new records when reapplied.

## Activate in project instructions

Paste this block into the active project `AGENTS.md` or equivalent to enable the Workflowy workflow for that project. Put any project-specific destination link outside the block. Use the [global context prompt](assets/templates/global-agent-context.md) for user-level opt-in.

```markdown
## Workflowy

This project opts into the `workflowy` skill and its private agent workspace.
Load the skill when available. Use the operator's project link or find this
repo or folder under **AI Agent Workspace > PROJECTS**. Reuse its branch or
insert it alphabetically. Search **STANDARDS** by title or topic tag. Read the
applicable rules, defaults, and scopes before governed work.

Use normal bullets for candid discussion, drafts, decisions, and evidence.
Never store work in Workflowy comments or comment drafts. Give `info` exactly
three direct children: `tags`, `links`, and `details`. Put purpose, current state,
next actions, blockers, and supporting evidence under `details`.
Use lowercase letters and underscores for reusable field names. Use inline
code for exact technical literals. Link to the exact canonical finding or evidence.
Git and the established tracker govern official progress where used.
Otherwise use the project artifacts. The operator decides what is published.

In `notes`, keep major sessions and substantive updates as a flat, oldest-first
list. Use only the ISO 8601 date or timestamp as the title. Retain the finest
supported precision and offset. Put the description, scope tags, and prose
under `title`, `tags`, and `details`. Use label-only field bullets with each
value or link nested in its own bullet. Require the Workflowy bullet format
for fields and values. Only the contents of `details` may use other formats.
Keep tags out of names and other fields. Use one hashtag per `tags` value.
Put any project-level tags in `info > tags`. Link related updates. The operator
owns timeline construction. Agents record dates without creating timeline
metadata, views, saved searches, or exports.

For Spec Kit, use one direct `spec-kit` bullet per formal slice in code order.
Use only its code and short title. Nest the verified date under `date` and
the documented epic under `epic` when applicable. Use the skill for shared
reference records, migration, and precise field rules.

Desktop MCP requires a running, signed-in Workflowy app with Local MCP
enabled and a connected client. A cloud or other nonlocal agent uses a
separately configured, operator-authorized connector when available. Confirm
the destination and read back writes. If no connector is available, continue
under the available instructions and provide an appropriately private,
ready-to-paste nested outline with its exact destination. Mark it unsynced.
```
