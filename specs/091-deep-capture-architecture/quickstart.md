# Quickstart: Verify Deep Capture Architecture

## Audit Required Mode and Trust Contracts

```powershell
rg -n "Capture|Deep Capture|Npcap|mitmdump|steam-protocol-cold|reached-client|confirmed|--trust-ca|current-user Root|packet truth|application.jsonl|tls-keylog.log|manifest.json|cleanup.json" site/content/docs/architecture.mdx
```

Review every match in context. Capture and Deep Capture must have separate diagrams, and `--trust-ca` must be the authorization rather than a promise of a later prompt.

## Audit Forbidden Claims

```powershell
rg -n -i "system-wide.*fallback|silent trust|extract.*target.*key|target.*key.*extract|bypass.*pinn|inject|hook|ReadProcessMemory|WriteProcessMemory|layered service provider|packet interception driver|bundles npcap|redistributes npcap|never downloads" site/content/docs/architecture.mdx
```

Expected: matches appear only where prohibited techniques or obsolete claims are explicitly rejected. The phrase `never downloads` must not remain because optional `net` source builds can fetch the vendor installer after confirmation.

## Audit Diagrams and Links

```powershell
rg -n "```mermaid|flowchart LR|/docs/getting-started|/docs/reference/deep-capture-compatibility|/docs/reference/output-formats|/docs/reference/cli|/docs/glossary/|fragcap-specification" site/content/docs/architecture.mdx
```

Count primary nodes in each Mermaid block. Each diagram must have no more than twelve, concise labels, and a left-to-right flow. Review all internal destinations through the documentation checker.

## Audit Privacy and Text Hygiene

```powershell
rg -n -i "account|token|private endpoint|real title|local host|captured payload" site/content/docs/architecture.mdx
git diff --check
```

Review any match in context. The page must contain no real title, account, endpoint, host, identifier, or captured material and must decode as UTF-8 without BOM or mojibake.

## Run Documentation Gates

```powershell
cargo xtask docs check
cargo xtask docs build
```

## Run the Full Repository Gate

```powershell
cargo fmt --all -- --check
cargo xtask ci
```

Confirm the implementation changes only the architecture page, the changelog fragment, and S091 artifacts.
