# shruggie-graph-memory

A companion skill that makes an AI assistant capture durable knowledge from ordinary conversation
into [ShruggieGraph](https://graph.shruggie.tech), a permission-scoped, source-backed AI memory you
own and carry across AI providers, and recall it later. When the ShruggieGraph MCP tools are
connected, the assistant proactively saves lasting facts (decisions, preferences, commitments,
stable details) as notes and probes your memory for relevant context on its own when a
conversation touches something you may have stored.

This README is human-facing setup. The skill behavior itself lives in `SKILL.md`, and the tool
reference in `assets/capture-reference.md`.

This is a single, provider-agnostic skill: the same `SKILL.md` works for Claude (claude.ai and
Claude Desktop) and for ChatGPT's uploadable Skills. Its instructions drive only the ShruggieGraph
MCP tools, which are the same everywhere, so there is nothing provider-specific to reconcile. The
frontmatter carries fields each platform reads and each other ignores harmlessly (`icon` for
ChatGPT's skill list; `disable-model-invocation` for Claude's automatic invocation).

## What it does

- Writes durable facts to your ShruggieGraph memory with the `create_note` MCP tool, and whole
  files with `upload_source`. Each becomes a cited source with an audit trail; the backend
  enforces all access.
- Recalls with `search_knowledge` (and the read tools) proactively, returning permission-filtered,
  cited context.
- Always stores what is durable. Genuinely sensitive facts are stored and marked `sensitive`,
  which restricts their sharing and visibility only. The only reason a write is withheld is an
  explicit instruction from you not to save a specific thing.
- Stays quiet on trivia and transient chatter.

The skill makes no permission decisions. ShruggieGraph is the sole authority for what a
connection may read or write, and the connection itself determines which memory notes land in;
there is nothing to configure in the skill.

## Prerequisites

- A ShruggieGraph account.
- The ShruggieGraph MCP tools connected to your AI client (either path below). The console's
  **Connect** guide (linked from the Connected apps page) documents both paths end to end.

## Connect claude.ai (web)

Add a custom connector in claude.ai with the URL `https://graph.shruggie.tech/api/mcp`. claude.ai
discovers the ShruggieGraph OAuth server, registers, and sends you to the console consent page,
where you sign in, pick which memory to link, and approve the scopes. Disconnecting in claude.ai
revokes the connection server-side; you can also sever any connection yourself on the console's
**Connected apps** page.

## Connect Claude Desktop

ShruggieGraph's MCP endpoint is request/response JSON-RPC, so Claude Desktop connects through the
bundled `mcp-stdio` adapter from the ShruggieGraph CLI (which speaks HTTPS).

1. In the console, open **Connected apps** (in the account menu) and create a token:
   - Client name: `Claude Desktop` (or your client).
   - Scopes: `mcp:note.create`, `mcp:search`, `mcp:read` for capture and recall; add
     `mcp:source.read` to follow results to their source, and `mcp:source.upload` if you want to
     upload whole files.
   - Memory: pick the memory this connection writes to and reads from.
2. Copy the `sgmcp_...` secret (shown once).
3. In `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "shruggie-graph": {
      "command": "/path/to/shruggiegraph-cli",
      "args": ["mcp-stdio", "--endpoint", "https://graph.shruggie.tech/api/mcp"],
      "env": { "SHRUGGIE_MCP_TOKEN": "sgmcp_..." }
    }
  }
}
```

For a more secure setup that keeps the secret out of the config file, mint the token with the CLI
flag `--store-account <name>` (which stores it in the OS keyring), then replace the `env` block
with `"--account", "<name>"` in `args`.

## Connect ChatGPT

ChatGPT needs two separate things: the ShruggieGraph MCP **tools** connected, and this **skill**
uploaded. They are independent — the tools are what the skill calls; the skill is the instructions
that tell ChatGPT to call them proactively.

1. Connect the tools: add a custom connector pointing at `https://graph.shruggie.tech/api/mcp`.
   ChatGPT discovers the ShruggieGraph OAuth server, registers, and sends you to the console consent
   page, where you sign in, pick which memory to link, and approve the scopes.
2. Upload the skill: in ChatGPT open **Skills**, then **+ → Upload from your computer**, and upload a
   zip of this directory (top-level folder `shruggie-graph-memory/` containing `SKILL.md`, `icon.png`,
   and `assets/`). The `icon.png` is the ShruggieGraph shrug mark and becomes the skill's icon.

## Where notes land

The connection carries the targeting: notes are written to the memory you linked at consent or
token-mint time, and searches read from it. If you link several memories to one connection, the
skill passes an optional `context` describing which one you mean. There is no workspace id or any
other identifier to configure.

## Install

Install the skill through your AI client's skill mechanism: a per-skill zip upload for Claude
Desktop or ChatGPT (**Skills → Upload from your computer**), or a symlink for a coding agent. This
one skill is provider-agnostic — the same `SKILL.md` serves Claude and ChatGPT — and is maintained
in the ShruggieGraph repository as its canonical source; install the copy under
`skills/shruggie-graph-memory/`. The bundled `icon.png` (the shrug mark) is used where the client
shows a skill icon.

## A note on automatic invocation

This skill sets `disable-model-invocation: false` on purpose. Automatic capture and proactive
recall during ordinary conversation are the whole point, so the assistant must be able to pick the
skill up by context rather than waiting for an explicit `/shruggie-graph-memory` command. The
actual write is performed by the `create_note` MCP tool, which you enabled by connecting with the
`mcp:note.create` scope, and which the ShruggieGraph backend independently authorizes. This is a
deliberate departure from the usual guideline that side-effecting skills are explicit-invocation
only, recorded here so it is not mistaken for an oversight.
