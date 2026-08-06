---
name: silent-execution
user-invocable: false
description: Reduce token waste by batching tool calls and avoiding narration between sequential operations
allowed-tools: []
---

# Silent Execution

## Name
han-core:silent-execution

## Description

When executing a sequence of related tool calls (running tests, fixing lint, applying edits), batch operations and report results only when complete.

## The Pattern

**Anti-pattern (wasteful):**
> "Now I'll run the tests..."
> [runs tests]
> "Tests passed. Now I'll run the linter..."
> [runs linter]
> "Linting clean. Let me check types..."

**Pattern (efficient):**
> [runs tests, linter, type-check]
> "All quality gates pass: tests (42 passed), lint (clean), types (no errors)."

## Rules

1. **Batch independent operations** — if operations don't depend on each other, run them all before commenting
2. **No narration** — don't describe what you're about to do for each tool call
3. **Report results** — speak only when the batch is complete or something unexpected happens
4. **Exception: interactive debugging** — when the user is watching you debug, narration helps them follow along

## Token Savings

~50-100 tokens per batch cycle. Over a typical session with 20+ build cycles, this saves 1,000-2,000 tokens.
