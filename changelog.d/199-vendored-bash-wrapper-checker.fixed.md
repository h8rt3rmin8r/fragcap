<!-- spec-impact: 18.4, 24.3 -->
Fixed `cargo xtask wrappers` so Bash script compliance now delegates to the vendored ShruggieTech Bash checker for `fragcap.sh`, `lint-docs.sh`, and `cut-release.sh` instead of a stale Rust reimplementation. The gate now treats missing Bash-runnable ShellCheck as an unable-to-run environment failure.

Applies-To: 0.6.0
