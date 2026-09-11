# Quickstart: Explicit Guided Calibration Choice

## 1. Observe target ambiguity

Run a controlled ambiguous discovery without a candidate. Confirm that `calibration.choice_required` contains two content-derived choices and that no registration plan, workflow, bundle, launch, trust, or compatibility fact is created.

## 2. Select and confirm one target

Repeat the exact command with one emitted `--candidate` identifier. Supply the resulting registration plan identifier through `--authorize-stdin`. Confirm that post-confirmation rediscovery reproduces the selected authority and only that target is registered.

## 3. Observe and resolve Steam client ambiguity

Use a stored Steam target without launch entries and controlled conflicting Steam metadata. Confirm that the target-specific choice set appears. Repeat with the selected identifier, confirm the Steam client plan, and verify the exact executable becomes the target's client declaration.

## 4. Persist advanced exact-case intent

Start a controlled workflow with `--launch-case direct-exe-cold --routing-strategy child-environment --proxy-family ipv6`. Pause or interrupt it, then resume only by workflow identifier. Confirm that human guidance, JSON guidance, proposal case, and low-level plan all retain the exact values.

## 5. Exercise refusals

Verify malformed, stale, duplicate, and unused candidate identifiers, unsupported routing, mismatched launch case, and resume-time override arguments. Each must stop before a session authorization plan or effect.

## 6. Run gates

Run focused target-store, event, CLI parse, and guided-calibration tests. Then run `cargo xtask ci`, the specification consistency checks, punctuation and mojibake scans, and a clean-worktree diff review.
