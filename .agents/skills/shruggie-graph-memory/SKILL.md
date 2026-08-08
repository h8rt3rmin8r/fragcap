---
name: shruggie-graph-memory
description: Automatically capture durable knowledge from an ordinary conversation into ShruggieGraph (a permission-scoped, source-backed AI memory) with its create_note MCP tool, and proactively recall it with search_knowledge. Use whenever the ShruggieGraph MCP tools are connected: store any durable, reusable fact worth keeping across sessions and AI providers (a decision, preference, commitment, deadline, or person/project/org detail), and probe the graph on your own for relevant context whenever the conversation opens a new topic, names a person/project/organization, or asks something the user's stored memory could answer. Also trigger on phrasings like "remember this", "save that to my memory", "what do I know about X", "check my memory for". Always store what is durable; the only reason to skip a write is an explicit user instruction not to save. Skip only trivia and transient chit-chat, and any session where the ShruggieGraph tools are not connected.
icon: icon.png
disable-model-invocation: false
---

# ShruggieGraph Memory

Turn an ordinary working conversation into durable, retrievable memory in ShruggieGraph, a
permission-scoped, source-backed AI memory backend the user owns and carries across AI providers.
When the ShruggieGraph MCP tools are connected, this skill has you write durable knowledge as
notes with the `create_note` tool (each note becomes a cited source with an audit trail) and
recall it with `search_knowledge`, probing the graph on your own rather than waiting to be asked.
The ShruggieGraph backend decides all access; this skill never makes a permission decision of its
own. It only chooses what is worth remembering, writes it to the memory the user's connection is
linked to, and surfaces what the user already knows.

## When to Use

Invoke this skill when:

- The user explicitly asks to remember, save, note, or store something ("remember this", "save
  that to my memory", "note that for later").
- The user asks what is already known ("what do I know about X", "check my memory for", "have I
  noted anything about Y").
- A conversation surfaces a durable, reusable fact even without an explicit save instruction: a
  decision, a stated preference, a commitment or deadline, a stable detail about a person,
  project, organization, or tool, or a recurring theme the user will want again later.
- The conversation opens a new topic, names a person, project, or organization, or asks a
  question the user's own stored memory could answer. Probe the graph for relevant context before
  answering (see Recall).

Do not invoke this skill when:

- The ShruggieGraph MCP tools are not connected in the session. Without them there is nothing to
  write to or read from; do not pretend to save or recall anything.
- The content is transient chit-chat, one-off scratch work, or trivia with no lasting value.
- The user is working inside a different memory or note system; defer to the tool they named.

## Instructions

Treat capture and recall as steady background habits, not features the user has to trigger. The
defaults are: when a durable fact appears, save it; when a topic or question could touch stored
context, search first.

### Knowing where to write

There is nothing to configure and no id to ask for. The user's connection (an `sgmcp_` token or
an OAuth-connected client) is linked to one or more memories when it is created, and `create_note`
writes to the only linked memory by default. Only if the user has linked several memories to one
connection does targeting matter: pass the optional `context` string describing which memory they
mean (for example their personal memory versus a shared group memory). Never invent a `context`
value; if a write fails on targeting, ask the user which memory they intend rather than retrying
blindly.

### What to capture

Write a note when the conversation produces something the user will plausibly want weeks later:

- Decisions and their rationale (for example "we picked Postgres over Mongo because ...").
- Stated preferences and conventions ("I always want commit messages in imperative mood").
- Commitments, deadlines, and plans with dates (convert relative dates to absolute).
- Stable facts about people, projects, organizations, and tools.
- Recurring themes, open questions, and things explicitly flagged as "for later".

Keep each note self-contained: a clear `title` and a `content` body that will still make sense out
of context, with enough detail that a future reader does not need this conversation to understand
it. One idea per note. Prefer the user's own wording for anything where phrasing matters. When a
fact came from someone other than the user, set `attributed_to`, and you may record your provider
and model with `recorded_by_provider` and `recorded_by_model`.

### Attaching context about the material: elicit author, date, and what it is

The person adding a source almost always knows things the raw bytes do not carry — **who wrote it**,
**when it originated**, and **what it is**. That context is cheapest to capture at write time, and
without it the pipeline defaults authorship to the account owner and dates to now, which silently
produces wrong derived knowledge. The canonical failure: a letter written by the user's father,
saved with no context, lets the graph infer the *user* wrote it and has a child by their own name.

So make capturing context a normal part of writing, not an afterthought. Both `create_note` and
`upload_source` take three optional declared-context fields:

- `author` — who authored or originated the material, when that is someone other than the account
  owner. This is distinct from `attributed_to`; set `author` so the derived knowledge attributes to
  that person.
- `date_of_origin` — when the material was written or created, when that differs from now. A fuzzy
  value is fine and encouraged over nothing ("circa 1998", "1998", "spring 2019"); it is stored
  verbatim as a hint. Never invent a precise date the user did not give.
- `about` — a short free-form description of what the material is, to frame the extraction.

Set each field whenever it is knowable — infer it from what the user just told you ("here's a letter
my dad wrote me in 1998" gives you author, date, and about at once), or ask one brief question when
the material clearly has an author or origin date you do not know. All three are optional and none
blocks the write: when the context is genuinely unknown, omit the field rather than guessing, and
capture anyway. The aim is to fill these on nearly every addition where the context is knowable.

### Storing sensitive material: always store, never self-classify

Perceived sensitivity never stops a write. If a durable fact appears, save it, whether or not it
feels personal or sensitive. This is the core promise of a portable memory: it captures what is
going on so the user can rely on it later.

**Never set `sensitive: true` on your own judgment.** Sensitivity is not something you perceive,
infer, or decide — a topic reading as personal, financial, medical, legal, or otherwise delicate
is never on its own a reason to flag it. Default every write to non-sensitive (omit the flag). Set
`sensitive: true` only when the user explicitly and specifically tells you to mark *that* item —
for example "mark this sensitive", "keep this one private", "this is confidential". Their explicit
instruction is the only trigger; absent it, the flag stays off no matter how the content reads, and
you never ask "should this be sensitive?" to prompt one.

The flag restricts the note's sharing and visibility only (sensitive records are excluded from
cross-user and cross-tenant sharing). It never changes whether a note is stored and never affects
which provider may process it — so there is no protective reason to set it yourself. When in doubt,
leave it off; the user can always mark a record sensitive later.

The one and only reason to withhold a write is an explicit user instruction not to save a
specific thing ("don't save this", "keep that out of my memory"). Honor that instruction, and say
plainly that you did not store it so the omission is visible. Do not generalize such an
instruction into a standing rule, and never infer one from the mere feeling that a topic is
sensitive.

### Duplicates are the backend's job — never gate a write on them

Do not search "to check for duplicates," and never withhold a fact because it might already be
stored. ShruggieGraph reconciles and de-duplicates on its own side (entity resolution,
relationship collapse, and claim de-duplication over everything ingested), so a note that restates
something already known is cheap and safe, and a second independent mention only strengthens the
record. The cost of writing a duplicate is near zero; the cost of skipping a real fact is a
permanent gap in the memory. So when a durable fact appears, write it — do not skip it because it
feels familiar, and **never treat having _searched_ as having _captured_**. Searching is for
recall (answering the user); it is not a step in, or a substitute for, a write.

Still write *good* notes: keep one idea per note, and batch several related facts from one exchange
into a small number of clear, self-contained notes rather than many fragments. That is about note
quality, not about holding anything back — when in doubt, write.

### Uploading whole files

When the user hands you a file to remember (a document, image, PDF, or text) rather than a fact to
paraphrase, use `upload_source` so the whole original is stored and enters ingestion. Provide the
bytes base64-encoded in `data_base64`, and optionally a short `note` of what you derived. The same
`sensitive` rule applies: upload it, and mark it sensitive when it genuinely is.

### Recall: probe the graph on your own

Recall is a default reflex, not an on-request action. Whenever the conversation could touch
something the user has stored, check the graph before answering from your own guesswork. Concrete
triggers:

- A new topic opens, or the user shifts to a subject you have not searched this session.
- A specific person, project, organization, tool, or other named entity is mentioned.
- The user asks a question their own stored context could answer ("what did we decide about ...",
  "when is ...", "how do I usually ...").

On a trigger, call `search_knowledge` with a focused query; follow a result to its source with
`get_source` when the span matters, and use `get_project_context` for broader source-backed
context on the connected memory. ShruggieGraph returns permission-filtered, cited context and does
not generate answers itself; you do the reasoning over the returned context and cite the notes you
used. Never imply a fact is stored if the search did not return it.

Stay proportionate. Probe when a trigger fires and relevant context plausibly exists; stay quiet
on small talk, and do not narrate empty searches or fire repeated queries for the same subject in
one session. If nothing relevant comes back, simply answer normally without mentioning the search.

### Confirmation and honesty

After a successful write, tell the user in one short line what you saved and where (for example,
"Saved to ShruggieGraph: 'Q3 launch moved to October'"), and note when you marked it sensitive at
the user's request, so the capture is visible and the audit trail is honest. If a write fails, or you withheld one on the
user's instruction, say so plainly rather than implying the note was stored.

For the tool input shapes, scopes, and setup details, read
[assets/capture-reference.md](assets/capture-reference.md).

## Examples

### Example: proactive capture without an explicit instruction

**User input:**

```
Let's go with Caddy for TLS termination on the alpha VM instead of nginx, mainly because
it does automatic certs and we don't want to manage renewals by hand.
```

**Expected output:**

```
Got it. (calls create_note with title = "TLS termination: Caddy on the alpha VM", content =
"Decision: use Caddy over nginx for TLS on the alpha VM. Reason: automatic certificate
issuance and renewal, so no manual renewal management.")

Saved to ShruggieGraph: "TLS termination: Caddy on the alpha VM".
```

### Example: a personal fact is stored WITHOUT self-marking it sensitive

**User input:**

```
For the record, my mother's health has been rough lately, so I'm going to be taking
Fridays off through the spring to help her with appointments.
```

**Expected output:**

```
(calls create_note with title = "Fridays off through spring for family care", content =
"Commitment: taking Fridays off through spring to help with a family member's medical
appointments." — NO sensitive flag: the user did not ask to mark it, and a personal or medical
topic is never on its own a reason to set it)

Saved to ShruggieGraph: "Fridays off through spring for family care".
```

### Example: sensitive is set ONLY on an explicit user instruction

**User input:**

```
Save my new home address, 42 Elm Street — and keep that one private.
```

**Expected output:**

```
(calls create_note with title = "Home address", content = "Home address: 42 Elm Street.",
sensitive = true — set only because the user explicitly asked to keep it private)

Saved to ShruggieGraph and marked sensitive: "Home address".
```

### Example: eliciting context when uploading material someone else authored

**User input:**

```
Here's a letter my father wrote me back in 1998 — hold onto it for me.
[attaches letter.pdf]
```

**Expected output:**

```
(calls upload_source with filename = "letter.pdf", content_type = "application/pdf",
data_base64 = <bytes>, author = "the user's father", date_of_origin = "1998",
about = "a personal letter the user's father wrote to them")

Saved the letter to ShruggieGraph, attributed to your father and dated 1998 — so it won't be read
as something you wrote.
```

### Example: proactive recall before answering

**User input:**

```
Okay, back to the alpha VM. Remind me how we're handling TLS there?
```

**Expected output:**

```
(recognizes a named subject and a question stored context could answer, calls search_knowledge
with query = "TLS termination decision alpha VM")

From your memory: you chose Caddy over nginx for TLS on the alpha VM, for automatic certificate
issuance and renewal, so there are no manual renewals to manage.
```

### Example: withholding a write on explicit instruction

**User input:**

```
Here's the door code for the office, 4417. Don't save that anywhere.
```

**Expected output:**

```
Understood, I won't store that. (no note written; withheld on the user's explicit instruction,
not because of any sensitivity judgment of my own)
```

## Additional Resources

- [assets/capture-reference.md](assets/capture-reference.md) - the six MCP tools, their input
  shapes, the five scopes, the `sensitive` boolean, and memory targeting.
- `README.md` - human-facing setup: connecting claude.ai or Claude Desktop to ShruggieGraph and
  minting a token.
