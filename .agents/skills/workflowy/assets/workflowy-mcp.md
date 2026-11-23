# Workflowy Desktop MCP

## Verify availability and scope

The desktop app must be running and signed in to the intended account with Local MCP enabled. Configure its Connection Link in a client that supports local Streamable HTTP. Keep the Connection Link in trusted client settings. A node URL points to content. A client that cannot reach the workstation needs a separately configured connection. [Official Desktop MCP setup](https://community.workflowy.com/t/workflowy-desktop-mcp/1695) · [Workflowy REST API](https://workflowy.com/api-reference/)

Discover the active tool schema. For a normal read, use `tree_seek` with `wait_any_match: false`. Use `url_is_any` for a supplied Workflowy node URL. Find named roots with a scoped exact match. Read the parent and ancestry before editing. For an operator-requested watch, use the connector's wait mode and observe its timeout behavior.

```json
{
  "seek_requests": [
    {
      "start_from": "root",
      "direction": "downward-deep",
      "max_match_edge_distance_from_start": 1,
      "filter": { "text_matches_any": [{ "pattern": "^AI Agent Workspace$" }] },
      "limit": 8,
      "truncate_text_at": 160
    }
  ],
  "wait_any_match": false
}
```

## Patch existing records

Use `tree_patch` with the smallest scoped batch. Find the direct child field by label and replace it by handle. For a move, retain the node ID and check parent and sibling order afterward. Use an adjacent anchor for sorted insertion and explicit child nesting for fields. Read back per-request outcomes and the live nodes. Re-read before retrying a failed or uncertain request. The attached note and bullet title are different fields. Use child bullets for structured fields.

Write native hashtags as plain text. Use a supported HTML anchor such as `<a href="https://workflowy.com/#/NODE">Source title</a>` for an exact internal reference. The editor's `[[` picker can also create internal links. Verify the target and, when relevant, its backlink. A generated backlink can appear in a seek as an unlabeled container and excerpts under the target. Treat it as incoming navigation. User-authored empty bullets are a separate case. Use native mirrors for intentionally shared live content and keep one canonical record. [Links and backlinks](https://workflowy.com/help/internal-external-links/) · [Tags and mentions](https://workflowy.com/help/tags-mentions) · [Mirrors](https://workflowy.com/help/mirrors)

## Activate a timeline date

Use the editor's `!!` or `/date` picker, or the active patch schema's supported native `time` HTML. Write the local calendar day and project tag on the same named child. This example indexes a landed slice.

```html
Date: <time startYear="2026" startMonth="11" startDay="20">Nov 20, 2026</time> Integrated #timeline #demo #slice
```

Read back the `time` attributes and exact tags. Workflowy can normalize the displayed date label. Use an ISO timestamp with the local offset for precise order in a session title or an `Integrated at` child. Preserve day precision when an exact time is unavailable. Use Pending or Unknown for undated slices. A text search can find `#timeline`, while the returned markup confirms the native date. A markup regex in a text filter may miss a native date.

Zoom to PROJECTS and use a starred search with `#timeline` and a date range. Add the project hashtag or `#slice` as needed. Calendar Found Dates gives a day view. For an exported chronology, follow each metadata hit to its parent, deduplicate by the canonical parent ID, and order by supported date and time. Add timeline metadata to each indexable event once. Keep catalog reference dates in reference fields unless the source is itself an independently recorded project event. [Date tags](https://community.workflowy.com/t/date-tags/1675) · [Advanced search](https://community.workflowy.com/t/advanced-search/1672) · [Calendar](https://workflowy.com/help/plan-with-calendar/)
