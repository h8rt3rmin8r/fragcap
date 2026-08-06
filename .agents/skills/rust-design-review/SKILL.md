---
name: rust-design-review
description: >
  Conduct a thorough Rust design review of code, functions, modules, or entire crates,
  covering idioms, design patterns, and anti-patterns. Use this skill whenever the user
  pastes Rust code and asks for a review, design critique, pattern audit, or says things
  like "is this idiomatic?", "how would a Rust expert write this?", "review my Rust code",
  "what patterns am I missing?", "is this good Rust?", "do a design review", or "what am
  I doing wrong in Rust?". Also trigger for any request to refactor Rust code to be
  more idiomatic or better structured. Produces a structured review grounded in the
  rust-unofficial/patterns book (https://rust-unofficial.github.io/patterns/).
---

# Rust Design Review

A structured review skill grounded in the [Rust Design Patterns book](https://rust-unofficial.github.io/patterns/)
(rust-unofficial/patterns), covering idioms, design patterns, and anti-patterns.

---

## Review Process

Work through these four phases in order. Skip any phase for which there is nothing to report.

### Phase 1 — Understand context

Before reviewing, briefly note:
- **Scope**: Is this a library crate, binary, module, or function snippet?
- **Audience**: Public API (crate users) or internal code?
- **Rust edition** if visible.

Library APIs get stricter scrutiny on ergonomics and trait implementations.
Internal / binary code gets lighter scrutiny on those axes.

### Phase 2 — Idiom scan

Work through the **Idioms Checklist** (see `references/idioms.md`). Flag every
violation with a severity: **🔴 Must Fix**, **🟡 Should Fix**, **🔵 Consider**.

Quick summary of what to scan for:

| Area | Common slip |
|------|-------------|
| Function arguments | `&String` / `&Vec<T>` / `&Box<T>` instead of `&str` / `&[T]` / `&T` |
| String building | `+` operator chains instead of `format!` |
| Constructors | Missing `new` associated fn; `Default` not derived when it could be |
| Ownership in enums | Cloning to satisfy borrow checker instead of `mem::take` / `mem::replace` |
| Options / Results | Manual `if let Some` loops instead of iterator combinators |
| Closures | Capturing whole env with `move` when only some vars are needed |
| Mutability | `mut` persisting beyond the point where mutation is needed |
| Error return | Consuming an arg without returning it inside the error |
| Public API structs | Not using `#[non_exhaustive]` when forward compatibility matters |

### Phase 3 — Pattern & anti-pattern audit

Work through the **Patterns & Anti-patterns Checklist** (see `references/patterns.md`).

Anti-patterns default to **🔴 Must Fix** unless the surrounding context clearly
justifies them (e.g. a deliberate clone in throwaway prototype code). Pattern
opportunities are **🔵 Consider**.

Key things to flag:

| Smell | Likely pattern to suggest |
|-------|--------------------------|
| Long builder-like `new(a, b, c, d, e...)` | Builder pattern |
| Mutex/resource accessed without guard type | RAII with guards |
| `f32`/`u32`/`String` used for domain values without newtype | Newtype pattern |
| `Box<dyn Trait>` list with manual dispatch | Command pattern or strategy via traits |
| Enum variants with complex logic duplicated | Visitor pattern |
| State machine logic scattered across methods | Typestate pattern |
| `Clone` called only to appease the borrow checker | `mem::take` / restructure |
| Inheriting from base structs via `Deref` | Deref polymorphism anti-pattern |
| `#[deny(warnings)]` in library code | Replace with `#[warn]` in CI |
| `unsafe` blocks spanning many lines | Contain unsafety in small modules |

### Phase 4 — Write the review

Use the **Output Format** below. Be specific: quote the offending code (short
snippets), name the pattern from the book, and show the corrected version.

---

## Output Format

```
## Rust Design Review

### Summary
<2–3 sentence overall assessment. Note the biggest wins and the most critical issues.>

### 🔴 Must Fix
<numbered list — each item: what, why, fixed version>

### 🟡 Should Fix
<numbered list — each item: what, why, improved version>

### 🔵 Consider
<numbered list — each item: pattern name, where it applies, brief sketch>

### ✅ What's Done Well
<bullet list — call out idioms and patterns already used correctly>
```

Omit any section that has no entries. Do not pad with generic praise.

---

## Reference Files

- `references/idioms.md` — Full idioms checklist with examples (read when doing the idiom scan)
- `references/patterns.md` — Full patterns & anti-patterns catalogue (read when doing the audit)

The summary tables in Phases 2 and 3 are a quick index only; the reference files
are authoritative. Always read the relevant reference file before writing your
first comment for that phase, including for short snippets — relying on memory
risks inventing rules that aren't in the book.

---

## Tone & Scope

- Be specific, not generic. "Use `&str` instead of `&String` on line 12" beats "prefer slices".
- Quote brief code fragments. Show corrected code for every 🔴 item.
- Prioritise by impact. A single 🔴 anti-pattern matters more than five 🔵 style notes.
- Acknowledge trade-offs. Some patterns add boilerplate; say so.
- Stay grounded in the patterns book; do not invent new rules.
