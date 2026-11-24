# Workflowy record format

## Reusable field names

Use lowercase letters and underscores for reusable field names, such as `title`, `current_state`, and `date_basis`. Keep established section names, including `spec-kit`, and preserve project names, dated or coded titles, and other distinct display labels. Use inline code in prose for exact technical literals readers may need to recognize, search for, or copy. See the [MCP inline code example](workflowy-mcp.md#write-inline-code).

## Dated notes

Use the same contract under project and Personal `notes`.

1. Put every record directly under `notes`, oldest first. The record title is an ISO 8601 date or timestamp by itself. Give a substantive follow-up its own sibling and link the earlier record under `related`. Keep corrections to the same record in place when they do not describe a distinct event.
2. Use the finest precision supported by the source. Preserve minutes, seconds, fractions, and the known UTC offset. Record live work in the operator's local time with the real offset. For historical work preserve the source offset or convert a known instant without losing precision. Preserve wall time if the offset is unknown and state that in `date_basis`. Use a date alone only when the source provides no finer date. Use a verified capture timestamp with `date_basis` when the event date is unknown. Flag a legacy record with neither for review.
3. Compare known instants for order. For day-only entries, place them before timed entries on that local calendar day. Keep stable source order for ties and avoid inferring a time from sibling order. Distinct events at the same instant remain distinct records.
4. Use the first three fields below on every record, then only optional fields that have values. Labels are standalone bullets, even if they have a single value. Indent each value beneath its label. Require the Workflowy bullet format for record titles, field labels, and values. The `details` label remains a bullet and its contents may use paragraphs, code, or other appropriate formats. Use one tag, link, or distinct item per bullet. Use nested bullets for supporting detail and keep the full account in one place.

| Field | Value |
| --- | --- |
| `title` | Short descriptive label. |
| `tags` | Primary project hashtag or `#personal`, with any relevant existing topic tags. One native hashtag per value. Keep tags out of names and other structured fields. |
| `details` | The substantive record, including context, reasoning, questions, and findings. Break it into readable bullets. |
| `outcome` | Supported result or disposition when useful. |
| `evidence` | Exact source, one link or locator per child. Use absolute external URLs. |
| `next_actions` | Follow-up work, one action per child. |
| `related` | Direct Workflowy links to the exact related records. |
| `recorded_at` | Capture or entry timestamp when distinct and relevant. |
| `date_basis` | Meaning, source, or precision limitation when it needs explanation. |

Use [the project outline](templates/project-outline.md) for a paste-ready example. A native Workflowy date can be a standalone value when it preserves the source precision, with the exact ISO timestamp retained as the authoritative date when the widget represents a day only.

## Project info

Use exactly three direct children under `info`: `tags`, `links`, and `details`. Put each project hashtag under `tags`. Put each primary entry point under `links`, such as the repository, tracker, documentation, or canonical notebook record. Put purpose, scope, `current_state`, `next_actions`, `blockers`, `evidence`, and every other project detail under `details`. Include only applicable subfields. Link claims to exact evidence.

## Spec Kit

Keep one direct bullet per formal slice in code order. Use only its formal code and short title, such as `S094 Check source order`. Give each slice its own bullet even when it belongs to a release or epic.

Use label-only fields with nested values. Use `date` for the supported authoritative integration or closeout date and `date_basis` when its meaning needs explanation. Use Pending or Unknown under `date` when applicable and leave it out of date queries. Use `epic` when membership is documented, with the linked epic as its own value. Use `scope`, `outcome`, `evidence`, `tags`, and `related` when useful. Put each hashtag under `tags` on its own bullet. Keep substantive dated narratives in `notes` and link from the slice. Put each evidence link on its own bullet.

The operator owns any future timeline. Routine agent work creates neither timeline fields or tags nor saved searches, synthesized views, or exports.
