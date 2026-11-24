# Set up and backfill a project

Use this guide when the operator asks to bring a project into Workflowy or recover its history. Start a new project with its brief, first question, or working notes. For an existing project, recover the parts of its history worth thinking with today, including what the operator tried, which choices they made and why, and what they still want to investigate.

## Establish the project and destination

1. Resolve the intended directory, Git root if present, and the account and workspace the operator uses for this project. A nested folder can be the intended scope even when Git finds a parent repo. If multiple checkouts or worktrees represent the same project, use their canonical repo identity to find the shared branch. Use the actual repo or folder name as the default title. For a title collision, establish a distinct destination before writing project material.
2. Look for the matching branch directly under **AI Agent Workspace → PROJECTS**. Preserve meaningful organization and stable node IDs. Insert a new project branch in case-insensitive alphabetical order. When the operator asks to establish a missing workspace, verify the account and use the [workspace layout and ABOUT bootstrap](workspace-layout.md). Use the [Workflowy MCP reference](workflowy-mcp.md) to resolve links, verify the destination, and check writes. Respect the private collaboration boundaries in the skill.
3. Confirm what the operator wants recovered from the given project, existing instructions, and any named archives or dates. For a request to backfill an existing project, start with the working baseline and sources clearly tied to that project. Ask only if a missing identifier, restricted destination, or ambiguous archive changes what can be recovered. Keep personal and corporate records linked to their own project and authorized destination.

## Choose sources by project type

### New Git repo

Read the initial brief, README, and any starting artifacts. Use the intended outcome, first action, and major decisions or questions already established. Paste the skill's `## Workflowy` opt-in block into the project instructions your agent reads. For a new project without meaningful history, start with the present and record the first major work session when it happens.

### Existing Git repo

- Establish the baseline from the canonical repo and the checked-out branch or commit. Check the worktree for staged and unstaged changes. Distinguish locally changed, proposed, merged, released, and deployed work where relevant. Confirm the upstream or default branch when it matters. Record a compatibility or testing claim to the extent supported by a check or a dated verification record.
- Start with the README, changelogs, release notes and tags, current specs, decision records, and the active issue or PR queue. Use changelogs to find releases, turning points, migrations, fixes that changed the plan, and recurring decisions. Follow referenced issues and PRs for the disposition and rationale. Verify the publication or deployment record appropriate to a shipping claim.
- Use targeted `git log`, `git show`, or relevant file history to answer a specific question. Start near the last documented release, an unresolved issue, or the commits touching the affected paths. Expand backward along links when an older decision still explains current work. Write a concise human-readable milestone, decision, or handoff with the evidence. Skip mechanical churn and commit-by-commit narration.
- Use absolute, clickable URLs to the actual remote forge, including enterprise or self-hosted hosts. Prefer stable links to the specific issue, PR, release, commit, exact version of a file, or test evidence. Use the forge or its connector to resolve URLs when available. Remove embedded credentials and tokens from any source URL. Use local references sparingly. A commit SHA or a plain-text repo-relative path can locate evidence when an external URL is unavailable. [GitHub guide to permanent file links](https://docs.github.com/en/repositories/working-with-files/using-files/getting-permanent-links-to-files)

### Project folder without Git

- Read the owner's brief, current deliverables, README or notes, and files that explain how the work evolved. Inventory the folder before opening large or unrelated material. Use dated drafts, document headings, decisions, and attributable session records to recover the sequence. File modification time can help locate candidates. Use a source date or mark the event date unknown.
- Treat the owner and current artifacts as the authority for state. Capture the leading question or objective, what has been tried, the most useful files and findings, and the next concrete move. Use project-relative paths, document titles, or existing private links as references. A short brainstorming folder may need a single outline with a few dated entries. Git is optional for this workflow.

## Review local agent records

Start with project-specific memory indexes and handoff summaries. Move to archived chats or transcripts to recover a missing decision, find a source link, or verify what the operator actually said. Use read-only discovery. Search the known store for an exact repo path, canonical remote or issue link, previous project name, or known session ID. Account for different worktree paths and operating systems. Inspect metadata and narrow matches before opening transcripts. Require a defensible project match and skip unrelated material in mixed archives. Consult an earlier transcript when a memory summary's wording or confidence matters.

### GPT and Codex

- Start with an operator-supplied ChatGPT export or saved project chats. Look for `conversations.json` or numbered conversation JSON files. Use the export viewer or HTML where supplied. Narrow by known chat title or ID, date, project identifier, or referenced artifact. Recover relevant user decisions and useful handoffs. Some organizational workspaces use owner-managed access. If the required record is unavailable, report that gap. [ChatGPT history and data export](https://help.openai.com/en/articles/7260999-exporting-your-chatgpt-history-and-data) · [ChatGPT export file names](https://help.openai.com/en/articles/9106926-transfer-exported-conversations-between-chatgpt-accounts)
- For local Codex records, resolve the configured `CODEX_HOME` or check the default `~/.codex`. Look for `memories/` or `memories_v2/` and their project-relevant summaries or rollout summaries, if present. Use any session index to narrow the search, then inspect matching records under `sessions/` or `archived_sessions/` if present. Prefer the agent client's own project or session lookup when exposed. Verify the installed layout and use working-directory metadata and session IDs to confirm attribution. Treat version-specific filenames and formats as discovery hints. [Codex configuration reference](https://developers.openai.com/codex/config-reference)

### Claude

- For Claude Code, check `CLAUDE_CONFIG_DIR` or `~/.claude`. Locate the matching project under `projects/`. Read its `memory/MEMORY.md` index and relevant topic files, or use the configured `autoMemoryDirectory`. Read only matching `<session-id>.jsonl` transcripts when more evidence is needed. Worktrees can share a memory directory while sessions are stored by working directory, so confirm which worktree a record concerns. Transcript retention and custom settings can affect coverage. [Claude Code memory](https://code.claude.com/docs/en/memory) · [Claude Code sessions](https://code.claude.com/docs/en/sessions)
- For Claude Desktop or web exports, use an operator-supplied archive or saved project notes and confirm the project, chat, and date. Claude Code's local directory layout applies to Claude Code.

### Ollama

- Identify the client or script that ran the conversation. If Ollama served a coding agent or chat frontend, recover project records from that client or its export. For a custom script, use the project's saved message history or output files when they exist. The chat API accepts message history supplied by the caller. [Ollama chat API](https://docs.ollama.com/api/chat)
- For interactive `ollama run`, `~/.ollama/history` can supply clues from saved input. Search it for project matches. Use project artifacts or a saved client transcript to recover replies and substantiate decisions or results. [Ollama CLI history implementation](https://github.com/ollama/ollama/blob/main/readline/history.go)
- If the operator identifies an Ollama session saved with `/save`, inspect that named model with `ollama show --modelfile` and look for its `MESSAGE` history. Confirm the messages belong to the project. A generic model's example messages may be template data. [Ollama interactive implementation](https://github.com/ollama/ollama/blob/main/cmd/interactive.go) · [Ollama Modelfile reference](https://docs.ollama.com/modelfile)

## Reconcile, write, and resume

1. Use only `tags`, `links`, and `details` as direct children of `info`. Put the present goal, supported state, next actions, blockers, and supporting evidence under `details`. Keep historical reasoning in its original record and distinguish firsthand verification from reported results.
2. Reconcile all formal slices against their canonical spec or tracker record. In `spec-kit`, place each slice directly under the section in code order with the formal code and short title. Preserve exact historical code spelling. Where the same code has multiple formal slices, distinguish them by source path and integration record. Record verified dates and any documented epic in child fields. Use the [project outline template](templates/project-outline.md).
3. Record significant sessions and distinct release or decision events as flat, oldest-first children of `notes`. Use the most precise date alone as each title, and put the description, tags, and prose under `title`, `tags`, and `details`. Recover dates from evidence and retain any verified capture timestamp separately. Follow the [record contract](record-format.md) and use [legacy remediation](legacy-remediation.md) for existing records.
4. Match records by meaning and evidence anchors such as exact issue, PR, spec, release, commit, session ID, or project-relative document and section. Preserve the original node ID. Merge verified copies, keep unique evidence, and use direct links from other sessions for the findings they reuse. Re-read just before each patch and verify the text, parent, order, date metadata, and links afterward.
5. Record the backfill session with its sources, coverage, and gaps. Keep changed or new context with the relevant record. A rerun with identical evidence adds zero records. Shared reusable knowledge follows the [catalog guidance](shared-catalogs.md).

## Install and verify the project instruction

- Inspect the project-level instructions and any applicable overrides. To enable Workflowy, paste the `## Workflowy` opt-in block from `SKILL.md` into the `AGENTS.md` or equivalent file the agent reads for that project. Replace an existing Workflowy opt-in or earlier `## Project continuity` block in place. Preserve surrounding guidance. A folder-only project can use its own `AGENTS.md` if the agent loads it there. [Claude Code instruction loading](https://code.claude.com/docs/en/memory)
- Put any project-specific note outside the verbatim block. Keep private node URLs in private project instructions only when useful. Preserve the local diff for the owner to review. Committing and publishing repo changes are separate actions.
- Verify the opt-in appears once in the active project instructions. For Git, check the intended diff and account for an untracked new file. Verify the Workflowy branch and any backfilled sample entries through the MCP. If a fresh session is part of the request and supported, give it a neutral project task and check whether it loads the skill when available and reads the correct branch. Report which parts you verified.
