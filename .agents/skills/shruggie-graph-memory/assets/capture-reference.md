# Capture reference: ShruggieGraph MCP tools

Reference for the ShruggieGraph MCP tools. This skill's primary pair is `create_note` (write) and
`search_knowledge` (recall); the connection also carries `upload_source` (write) and three read
tools (`get_source`, `get_project_context`, `get_index_status`), documented here to match the live
server surface. Read this when you need the exact input shape, the scopes a token must carry, or
how memory targeting and sensitivity work. The ShruggieGraph backend is the sole authority for
access; everything below describes how to call the tools correctly, never how to bypass them.

<!-- SHRUGGIE_MCP_TOOLS: search_knowledge, get_source, get_project_context, create_note, upload_source, get_index_status, list_claims, list_entities, list_relationships, list_events, get_usage, get_insights, get_preferences, list_shares, deescalate_sensitivity, reassign_topic, correct_identity -->

The comment line above is the canonical machine-readable list of the tools this skill documents.
A conformance test in the ShruggieGraph API crate parses it and fails the build if it ever
diverges from the tools the server actually exposes, so this reference cannot silently drift from
the system again. Keep it in sync with the sections below whenever the surface changes.

## `create_note` (write)

Creates a note as a manual-note source in the connected memory, with provenance and an audit
record.

Inputs:

- `title` (string, required): a short, self-contained title.
- `content` (string, required): the note body; self-contained and understandable out of context.
- `sensitive` (boolean, optional): defaults to `false`. Set it `true` ONLY when the user
  explicitly and specifically asks to mark this item sensitive/private/confidential; never set it
  on your own judgment of the content, and never ask in order to prompt one. The flag restricts the
  note's sharing and visibility only (sensitive records are excluded from cross-user and
  cross-tenant sharing). It never affects whether the note is stored and never participates in
  provider routing.
- `visibility` (string, optional): leave default unless the user asks otherwise. With no explicit
  value a note defaults to the user's private visibility.
- `attributed_to` (string, optional): according to whom, that is the author or sender the person
  named, when the fact came from someone other than the user.
- `recorded_by_provider` (string, optional): your AI provider.
- `recorded_by_model` (string, optional): your AI model.
- `author` (string, optional, ≤500 chars): who authored or originated the *material*, when that is
  someone other than the account owner. Declared context about the material itself, distinct from
  `attributed_to` (which records who you are relaying the fact from). Supplying it makes the derived
  knowledge attribute authorship to that person instead of defaulting to the account owner.
- `date_of_origin` (string, optional, ≤500 chars): when the material was written or created, when
  that differs from now. May be fuzzy ("circa 1998", "1998", "spring 2019"); it is stored verbatim
  as a hint and is never turned into a false precise date.
- `about` (string, optional, ≤500 chars): a free-form description of what the material is (for
  example "a letter my father wrote me in 1998"), to frame the extraction.
- `context` (string, optional): which linked memory to write to, when the connection is linked to
  more than one. Omit it when only one memory is linked (the default targets it). A `context`
  that is malformed or names a memory the connection is not linked to is rejected, never silently
  written to the default memory.

Requires the connection to carry the `mcp:note.create` scope. A write without the scope, or to a
memory the connection is not linked to, fails closed. `author` / `date_of_origin` / `about` are the
optional **declared context** (GitHub #191): all optional, none blocks the write, each trimmed and
capped at 500 characters (an over-length value is rejected, never truncated).

## `upload_source` (write)

Uploads an entire file (document, image, PDF, or text) as a source. The whole original file is
stored and enters ingestion, and derived knowledge links back to it. Use this when the user hands
you a file to remember rather than a fact to note.

Inputs:

- `filename` (string, required): the original filename, including its extension.
- `content_type` (string, required): the MIME type, for example `application/pdf`.
- `data_base64` (string, required): the file's bytes, base64-encoded.
- `sensitive` (boolean, optional): as for `create_note` — set only on an explicit user instruction,
  never on your own judgment; sharing and visibility only.
- `visibility` (string, optional): leave default unless the user asks otherwise.
- `note` (string, optional): a short note of what you derived from the file. Recorded as context;
  it does not replace the stored file.
- `attributed_to` (string, optional): according to whom.
- `recorded_by_provider` (string, optional): your AI provider.
- `recorded_by_model` (string, optional): your AI model.
- `author` (string, optional, ≤500 chars): who authored or originated the *file*, when that is
  someone other than the account owner — declared context about the material, distinct from
  `attributed_to`. Makes derived knowledge attribute authorship to that person.
- `date_of_origin` (string, optional, ≤500 chars): when the file was written or created, distinct
  from the upload date. May be fuzzy; stored verbatim as a hint, never fabricated into a precise
  date.
- `about` (string, optional, ≤500 chars): a free-form description of what the file is. Distinct from
  `note` (what *you* derived); `about` describes the material itself.
- `context` (string, optional): which linked memory to write to, when more than one is connected.

Requires the `mcp:source.upload` scope and a linked memory. Fails closed without them. `author` /
`date_of_origin` / `about` are the same optional **declared context** as on `create_note` (GitHub
#191): all optional, none blocks the write, each trimmed and capped at 500 characters.

## `search_knowledge` (recall)

Returns permission-filtered, cited context for a query. It calls no language model and generates
no answer; you reason over what it returns and cite the notes you used.

Inputs:

- `query` (string, required): a focused natural-language query.
- `context` (string, optional): the memory to search when the connection is linked to more than
  one. Omit it when only one memory is linked.
- `require_citations` (boolean, optional): leave default unless the user asks.

Requires `mcp:search` (or `mcp:read`). Results are filtered to what the connection is authorized
for; content from other people's memories is never returned.

## Read tools (source, context, and index state)

These support recall and are available on the connection; reach for them when a `search_knowledge`
result needs to be followed to its source, when the user asks for project context directly, or to
check the memory's index state before a write.

- `get_source` (read): retrieve a source's metadata and its allowed text or spans, including source
  availability state. Input: `source_id` (string, required). Requires `mcp:source.read`.
- `get_project_context` (read): return source-backed context for the connected memory. Inputs:
  `query` (string, required), `context` (string, optional). Requires `mcp:read`.

### `other_memories`: matches in the person's other connected memories

`search_knowledge` and `get_project_context` search **one** memory per call. When the connection is
linked to more than one and another of them holds records matching the query, the reply carries an
`other_memories` array: one entry per memory, each `{ "context": "<memory id>", "matches": <count> }`.
It is absent when there is nothing to report, so its absence genuinely means "no matches elsewhere",
not "not checked".

It deliberately carries **no content** — a count and an identifier, nothing else. The whole point is
that the person has not yet decided to look there.

**Say it out loud rather than ignoring it.** If the search you ran came back thin and
`other_memories` is present, do not answer as though the knowledge does not exist. Tell the person
what is there and offer to look:

> I didn't find anything about that in your work memory, but there are 3 matching notes in your other
> connected memory. Want me to check there?

Only look if they say yes, by re-running the same query with `context` set to that memory's id. Never
cross to another memory silently, and never present a count as though you had read the records.

(A one-time admit that turns this into a single confirmed step, instead of a second query, is
planned; until it ships, re-running with the `context` is how you look.)
- `get_index_status` (read): report whether the connected memory is mid-reindex and list its
  active background jobs (uploads, embedding, maintenance) with progress. Input: `context`
  (string, optional). Requires `mcp:read`. Use it before a write to avoid acting on a graph that
  is being rebuilt.

## Derived-knowledge list tools (read)

These list the connected memory's derived knowledge so you can browse the graph without the web
console. Each is read-only, requires `mcp:read`, returns only what the connection is authorized to
see, and takes optional `q` (a case-insensitive text filter), `limit` (page size, one of 25, 50,
100, 250, 500, default 25), and `context` (which linked memory, when more than one is connected).
None has a required input.

- `list_claims` (read): the memory's claims, the derived factual statements in the graph.
- `list_entities` (read): the memory's entities, the people, organizations, projects, and other
  nodes.
- `list_relationships` (read): the memory's relationships, the edges between entities.
- `list_events` (read): the memory's events, the dated occurrences in the graph.

Reach for these when the user asks what is in their graph, or to browse a kind of knowledge before
a focused `search_knowledge`. For a specific record's full detail or source, follow up with
`get_source` on the source a result cites.

## Account read tools (usage, insights, preferences)

These read the connected account's own context so you can orient yourself without the web console.
Each is read-only, requires `mcp:read`, and takes only optional `context` (which linked memory, when
more than one is connected). None writes anything.

- `get_usage` (read): the account's own current-period usage (input and output tokens, embeddings
  created) and plan standing (tier and monthly limits). Use it to pace work, for example before a
  large `upload_source` or a burst of `search_knowledge`, so you can warn the user if they are near a
  limit. It reports only this account's own usage, never another account's and never cost or
  admin-only counters.
- `get_insights` (read): a summary of the memory's top current-interest topics and the total
  interaction count over a recent window. Takes an optional `days` (one of the supported ranges;
  defaults to the standard window). Use it when the user asks how their knowledge is trending.
- `get_preferences` (read): the connection owner's UI preferences (currently whether the
  delete-confirmation prompt is suppressed). These are per-user, so the result is the same across a
  user's linked memories. Read-only; changing preferences stays in the web console.

## Sharing, visibility, and topic tools

These let a connection see what others shared, correct an over-cautious sensitivity flag, and file a
record under the right topic, without the web console.

- `list_shares` (read): list the knowledge records other people have shared with the connection
  owner (the shared-with-me surface). Input: `context` (optional). Requires `mcp:read`. Returns only
  the owner's own shares, never anyone else's.
- `deescalate_sensitivity` (write): reverse an automatic sensitivity escalation on a knowledge
  record, clearing its `sensitive` flag so it can be shared again. Inputs: `record_id` (required),
  `reason` (optional, recorded in the audit log), `context` (optional). Requires the
  `mcp:visibility.write` scope and edit access to the record. It only reverses a flag an automatic
  de-duplication step raised; a sensitivity the user declared is never reversed. Sensitivity governs
  sharing and visibility only, so this never changes whether content is stored, extracted, or
  embedded. Audited.
- `reassign_topic` (write): assign a knowledge record to a topic, a deterministic override of the
  automatic tagging. Inputs: `record_id` (required), `topic_id` (required), `context` (optional).
  Requires the `mcp:topic.write` scope and write access.

## Entity identity correction

- `correct_identity` (write): correct entity identity in the graph. Operations: `merge` two
  entities, `split` a prior merge apart, `mark_distinct` (record that two entities are genuinely
  different, blocking future automatic merges), and `unmark_distinct`. Requires the
  `mcp:graph.correct` scope (not granted by default) and owner/admin graph-write access.
  **Two-call safety**: call once WITHOUT `confirmation_token` to preview (inputs: `operation`,
  `entity_ref_a` (id or name), `entity_ref_b` for merge/mark/unmark, and optionally `merge_id` for
  split — omit it and the entity's own most recent merge is used). The preview returns a `detail`
  describing exactly what will change (for merge: the `survivor` and the `folded` entities plus the
  resulting visibility/sensitivity; for split: the members it `restores`; for mark/unmark: the
  `pair`) plus a short-lived `confirmation_token`, OR a `needs_disambiguation` list when a name
  matches more than one entity (no token). Then call again with
  `{ confirmation_token, confirm: true }` to apply. Nothing changes on a single call; a token is
  bound to its workspace and expires. Tenant-confined and audited. A split of an entity with no
  prior merge is a safe no-op reported at preview; a `merge_id` that does not involve the named
  entity is refused rather than confirmed.

## Memory targeting

There is no id to resolve and nothing to ask the user for. The credential itself carries the
targeting: an `sgmcp_` token or an OAuth connection is linked to one or more memories when it is
minted or consented, and every tool defaults to the only linked memory. Only when the user has
linked several memories to one connection does the optional `context` string matter; describe
which memory you mean (for example the user's personal memory versus a shared group memory) and
the backend resolves it. Never invent a `context` value: one that is malformed is refused as bad
input, and one that does not match a linked memory is denied, so a targeting mistake never lands
in the wrong memory. If a call fails on targeting, ask the user which memory they intend.

## Scopes a memory connection should carry

For this skill the token minted in the ShruggieGraph console (Connected apps page), or the OAuth
consent granted at connect time, should carry the scopes for the tools you use:

- `mcp:note.create` (to write notes with `create_note`)
- `mcp:search` (to recall with `search_knowledge`)
- `mcp:read` (to read project context, index state, derived knowledge, usage, and shares, and as
  the search fallback)
- `mcp:source.read` (to follow a result to its source with `get_source`)
- `mcp:source.upload` (to upload whole files with `upload_source`)
- `mcp:visibility.write` (to reverse an automatic sensitivity escalation with
  `deescalate_sensitivity`)
- `mcp:topic.write` (to file a record under a topic with `reassign_topic`)
- `mcp:graph.correct` (to correct entity identity with `correct_identity`; not granted by default)

These eight are the scopes the server advertises for this surface. A connection's effective
authority is the live intersection of its scopes, its linked memories, and the owner's current
permissions. If a write or search starts failing, the connection may have been revoked or its
permissions changed; surface the error rather than retrying blindly.
