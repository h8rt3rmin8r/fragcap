# Named project outline template

Copy the shape and fill only the relevant fields. Keep `spec-kit` when the project uses it. The bracketed date is an instruction to insert an activated date in Workflowy.

```text
project-name #project-name
  info
    Purpose: <one or two lines>
    Current state: <latest supported result and direct evidence link>
    Next action: <specific action>
    Blocker: <only when active>
  spec-kit
    S094 Check source order
      Date: [native local date] Integrated #timeline #project-name #slice
      Integrated at: 2026-11-20T10:15:00-05:00
      Epic: <formal epic ID and title, linked>
      Scope: <short description>
      Outcome: <verified result or current disposition>
      Evidence: <permanent spec, PR, commit or tracker record>
  notes
    2026-11-20T10:20:00-05:00 Reviewed source order
      Timeline: [native local date] #timeline #project-name
      Outcome: <what changed>
      Evidence: <exact slice, prior finding or external artifact>
      2026-11-20T10:35:00-05:00 Checked import order
        Result: <new evidence or correction>
```

Use a short code-first title for each direct slice. Keep one `Date` field even when the value is Pending or Unknown. Include `Epic` when membership is documented. For a standalone slice, omit the Epic line.
