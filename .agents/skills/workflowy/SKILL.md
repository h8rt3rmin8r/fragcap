---
name: workflowy
description: Read, discuss, capture, and organize the operator's private working notes through the Workflowy MCP. Use when asked to brainstorm or review ideas in Workflowy, connect research and references, maintain an outline, or work with a project that opts in.
disable-model-invocation: false
metadata:
  version: "1.0.1"
---

# Workflowy

Workflowy is the operator's private working notebook, shared with the agents they choose to involve. Use it to think together while an idea is still forming, work through a disagreement, keep a useful discovery, or develop something the operator wants to return to. Some of that work belongs to a project. Some starts with a question, an observation, or a subject the operator wants to explore.

## Work in the notebook

- Read the note, its surrounding discussion, and any source that matters to the question. When asked to contribute, put your response beneath the relevant idea so the operator's words and your reply can be read together. Revise existing prose when the request is to edit it.
- If the operator asks what they're missing, state the strongest concern you see and what makes it consequential. If the evidence is thin, say what would change your mind. During a brainstorm, develop the promising options and keep unresolved questions visible. Preserve disagreements, abandoned approaches, rough drafts, and private context when they matter to the discussion.
- Give the material the shape it needs. An observation can be a single bullet. A discussion can grow through replies, links, and revisions. A draft can live in full. When a decision changes, connect it to the earlier choice so the reasons remain legible. Keep exploratory ideas distinct from decisions the operator has made.
- Date major sessions and substantive updates, including research and brainstorming. Use a timezone-aware clock to generate an ISO 8601 timestamp in the operator's local time zone with the UTC offset in effect.

Write candidly for the operator and their authorized agents. This is an appropriate place for a frank assessment or unpublished context that would be out of place in a public issue or PR. Workflowy is hosted and node sharing can include descendants. Use the intended account and private branch, and resolve any ambiguity about who can read restricted material before adding it. Keep credentials in their existing secret store. Publication is a separate decision for the operator. [Workflowy sharing guide](https://workflowy.com/help/share-collaborate) · [Workflowy privacy policy](https://workflowy.com/privacy-policy/)

## Find the right place

The desktop MCP requires a workstation with Workflowy installed, running, signed in, and Local MCP enabled. The agent session needs a connection to that server. Discover the available tools, then confirm access with a scoped read. [Workflowy MCP reference](assets/workflowy-mcp.md) · [Official Workflowy Desktop MCP documentation](https://community.workflowy.com/t/workflowy-desktop-mcp/1695)

Start with the link, branch, or topic the operator named. In **AI Agent Workspace**, read **ABOUT** for operating guidance. Find opted-in repos and folders under **PROJECTS**. Follow an existing conversation or subject branch when it fits. Use the reserved **PROJECTS → _general** branch for discussions, ideas, and notes without a named project or an established destination. A new topic can start with the question itself. Check title and ancestry before writing. If the destination is ambiguous, show the candidates and ask which one to use. Read back edits through the MCP.

If the connection is unavailable, carry on with what can be done in the conversation. State which note or change still needs to be saved, and put a useful draft or handoff in the reply when that conversation is appropriate for the material.

## Build shared knowledge

Use the shared collections to make unfamiliar terms, people, tools, and subjects easier to look up. Check for an existing entry before adding one, and keep local discussion with the topic or project it came from. Each shared entry takes its name and native Workflowy hashtags in the title. Give each field a child bullet in `Field: value` form. Find and edit fields by label.

Keep entries in **GLOSSARY**, **REFERENCES**, and **ENTITIES** in alphabetical order by name. Ignore case and topic hashtags when sorting, and insert each new entry in its sorted position.

| Collection | Fields | Optional fields |
| --- | --- | --- |
| **GLOSSARY** | `Definition` | `Aliases`, `Source` |
| **REFERENCES** | `URL`, `Use when` | `Type`, `Source ID` |
| **ENTITIES** | `Type`, `Summary` | `URL` |

Use absolute URLs to the actual external source. Preserve a source's published identifier in `Source ID` when it helps trace the entry. Name a local path or document sparingly when it adds something, preferably in the branch that uses it. Reuse the same hashtag across related entries. For example, `#mcp` can connect a definition, a specification, and a discussion. Click or search the tag from **AI Agent Workspace** for matches across its branches. Via MCP, search for the literal tag and check the matched bullets. [Workflowy tags and mentions](https://workflowy.com/help/tags-mentions) · [Workflowy search scope](https://workflowy.com/help/search-your-notes/)

## Use Workflowy for a project

Use the project's own outline and adapt it as the work changes. Keep conversations, open questions, and reasons for decisions where the work happens. Update the current state and next action in place when they change. A major session can record an outcome, what you checked, and any blocker or follow-up. Use Git, the issue or PR tracker, or the relevant project artifacts to verify official progress. Link directly to the evidence.

Keep direct children of **PROJECTS** in case-insensitive alphabetical order by displayed name. The reserved branches `_example` and `_general` come first in that order. Insert a new project in its sorted position.

For a new repo, an existing Git project, or a folder-only effort, use the [project setup and backfill guide](assets/project-backfill.md). It covers changelogs and Git evidence, local GPT/Codex, Claude, and Ollama records, and recovering useful material in batches.

### Enable Workflowy for a project

Paste the Markdown inside the code block into the project's active `AGENTS.md`, or the equivalent instruction file your agent reads in a standalone folder. This opts that project into using Workflowy. Agents use the `workflowy` skill and Desktop MCP available to their session. The live copy is under **ABOUT → Enable Workflowy for a project**.

```markdown
## Workflowy

This project opts into the `workflowy` skill and Workflowy Desktop MCP for
substantive work. Load the skill when available and use the MCP to follow the
steps below. These instructions also stand on their own.

- **Find the project.** Read **ABOUT** in **AI Agent Workspace**. Find this repo
  or folder under **PROJECTS** by name or use the operator's project link.
  Read the current state and relevant discussion. Check the destination before
  writing. Reuse the existing branch or create it in alphabetical order.
  **PROJECTS → _general** is reserved for work without a named project.
- **Work in the notebook.** Use the private branch to brainstorm, develop
  drafts, give candid feedback, and work through disagreements with the operator
  and their authorized agents. Put replies beneath the discussion they address.
  Shape the outline around the work. Publication is an operator decision.
- **Keep the record useful.** Start each major session entry and substantive
  update with an ISO 8601 timestamp in the operator's local time zone, using
  the offset in effect. Record outcomes, evidence, and open questions or next
  actions. Update the current state in place. Git and the established tracker
  are authoritative when used. Otherwise, verify against project artifacts.
  Link to the evidence. For a requested backfill, bring forward useful past
  decisions and context with their sources.
- **Connect related notes.** Reuse native hashtags. Follow **ABOUT** for the
  named fields in **GLOSSARY**, **REFERENCES**, and **ENTITIES**. Sort entries
  alphabetically by name, ignoring case and hashtags. Use absolute URLs for
  external links and name local sources sparingly.
- **Verify and continue.** Read back MCP edits. Desktop MCP requires a running,
  signed-in Workflowy app with Local MCP enabled and a connected agent session.
  If MCP is unavailable, continue the work and put any useful draft or handoff
  in the final reply when its audience is appropriate.
```
