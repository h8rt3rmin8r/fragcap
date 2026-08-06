---
name: token-efficiency
user-invocable: false
description: Minimize token consumption through efficient tool usage patterns
allowed-tools: []
---

# Token Efficiency

## Name
han-core:token-efficiency

## Description

Minimize token consumption without sacrificing quality. Every token spent on overhead is a token not available for thinking.

## File Operations

- **Use Edit for modifications** — sends only the diff (~50-100 tokens), not the whole file
- **Use Write only for new files** — full file content costs ~500-5000 tokens
- **Never re-read a file you just wrote** — you already know its contents
- **Batch related edits** to the same file in one Edit call

## Search Operations

- **Use Glob for file finding** — faster and cheaper than Bash find
- **Use Grep for content search** — cheaper than Bash grep
- **Set head_limit on Grep** — avoid returning thousands of matches

## Context Management

- **Don't repeat what the user said** — they can see their own message
- **Lead with the answer** — skip preamble and filler
- **Use structured output** — tables and lists over prose for data

## Anti-Patterns

| Wasteful | Efficient |
|----------|-----------|
| Read file → Write entire file with 1 line changed | Edit the specific line |
| `cat file.txt` via Bash | Read tool |
| `find . -name "*.ts"` via Bash | Glob `**/*.ts` |
| "Let me start by reading the file to understand..." | Just read it |

## When to Apply

Always. Token efficiency is not premature optimization — it directly extends how much work fits in a session.
