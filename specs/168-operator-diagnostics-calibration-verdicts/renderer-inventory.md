# S168 human report renderer inventory

Every application-authored human multi-column report uses `display::ColumnLayout` or `display::render_fields`. Widths include actual emitted optional/indexed labels, headings, delimiters, and intentional ANSI styling measured as zero cells. Each adjacent anchor follows the widest preceding cell plus exactly four spaces. A narrow terminal retains these anchors, hangs value continuations beneath their anchor, and preserves indivisible exact tokens even when they exceed the terminal width. Human values escape layout controls; machine schemas are unchanged.

| Command/report family | Renderer | Slice owner | Required regression |
| --- | --- | --- | --- |
| Doctor checks, readiness, healthy history | `doctor/mod.rs`, `doctor/checks.rs`, `commands/doctor.rs` | Layout | Actual name/status widths, color/plain parity, narrow identities, many healthy resources plus active/mixed/actionable/limited inventory, exact detail and JSON |
| Capture completion and process breakdown | `output.rs` | Layout | Every accounting field retained, widest process name, capture human goldens |
| Capture live status and process breakdown | `live_status/mod.rs` | Layout | All counters, bound/elapsed/filter state, five-row bounded tally, long exact image/role/PID at narrow width, complete ANSI |
| Steam installed listing | `commands/steam.rs` | Layout | Header and four-column anchors, Unicode/controls, vertical measured-width fallback, unchanged three identity states and JSON |
| Technology detection | `commands/technologies.rs` | Layout | Measured product/fidelity/evidence anchors, every category and coverage warning |
| Sensitive bundle cleanup results | `commands/bundle.rs` | Layout | Human status/path/reason anchors; existing machine result remains unchanged |
| Fresh-start exact-root preview and metadata | `commands/fresh_start.rs` | Layout | Profile/kind/path/state anchors; unchanged reviewed inventory and deletion authority |
| Target listing, detail, discovery, reconciliation, ambiguity | `commands/targets.rs` | Discovery | Source chapters, report-wide field anchors, optional/indexed facts, all identity/accounting/evidence values |
| Capture target-resolution ambiguity | `commands/target_resolve.rs` | Discovery | Human candidate anchors and supported selector parsing |
| Calibration history/prior facts and preflight | `commands/calibrate.rs` | Discovery and root | Exact stored identities, early preflight, case-specific guidance |
| Canonical plan, session diagnosis, artifact and final result | `session_ux.rs`, `commands/deep_capture.rs` | Root | Actual optional/indexed keys, phases, independent authority/populations, exact paths, narrow/color/plain contracts |

Catalog seeding counters, schema validation results, Doctor probe progress, extcap installation outcomes, and ordinary warning/error sentences are prose rather than aligned reports. Extcap declarations are analyzer protocol, build identity is JSON, and event/schema/structured output retain their exact machine grammar. Clap-generated help owns its own option-description formatting; this inventory covers application-authored reports.

The S168 report contract supersedes historical single-space/two-space/fixed-width report padding and the S069 live-status truncation rule. Steam retains its measured-width vertical fallback, now with shared four-space field anchors. Live status retains the five-entry holder bound and explicit overflow count while replacing character truncation with exact-token continuation. `--history-details` expands retained healthy Doctor records; `--json` always preserves every stable resource check, and ordinary Doctor remains read-only.

## Dependency decision (2026-10-09)

Use exact-pinned `unicode-width` 0.2.2 in `fragcap-cli` with default features disabled. The previous handwritten character ranges missed non-Latin combining marks and composed emoji sequences. The library measures complete sanitized strings after CSI/OSC styling removal, preserving emoji and script ligatures across escape boundaries. Normal width treats ambiguous characters as narrow; CJK alternate-width tables are unnecessary. The [official release manifest](https://raw.githubusercontent.com/unicode-rs/unicode-width/v0.2.2/Cargo.toml) declares Rust 1.66, MIT OR Apache-2.0 licensing, and no required runtime dependencies. The [official API documentation](https://docs.rs/unicode-width/0.2.2/unicode_width/) describes complete-string Unicode/emoji width handling. One lockfile package and one CLI runtime dependency are added; no core/platform dependency direction changes.

The width contract uses Unicode terminal-cell conventions, independent of terminal font quirks. Layout is tested against Latin and Hebrew combining marks, East Asian wide characters, flags, composed emoji, CSI/OSC escapes, and repeated whitespace. At a word wrap, a newline replaces one word separator, additional spaces remain in the continuation, and indivisible tokens remain exact. Empty trailing cells emit no trailing padding.
