# fragcap

![fragcap wordmark](brand/logos/svg/fragcap-wordmark-cyan.svg)

[![CI](https://github.com/h8rt3rmin8r/fragcap/actions/workflows/ci.yml/badge.svg)](https://github.com/h8rt3rmin8r/fragcap/actions/workflows/ci.yml) [![Release](https://img.shields.io/github/v/release/h8rt3rmin8r/fragcap)](https://github.com/h8rt3rmin8r/fragcap/releases/latest) [![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**See which Windows game process produced each captured flow.** fragcap joins packets with socket and process-lifecycle evidence, even when a platform or publisher launcher starts the game indirectly. It writes process and role annotations into `.fcapng`, an extended pcapng file that unmodified analyzers such as Wireshark can open.

Capture passively records packets. Deep Capture adds explicit, target-scoped local proxy inspection for compatible stored targets. Both modes avoid injection, hooks, and target-process memory access. [Read the first-run guide](https://fragcap.com/docs/getting-started) for the complete workflow.

## Choose a mode

| Mode | Use it for | Output |
| --- | --- | --- |
| **Capture** | Attribute selected game traffic to launcher, client, and platform-service roles without changing traffic. | Packets and attribution in `.fcapng`, with named loss and exclusion counts. |
| **Deep Capture** | Inspect compatible traffic through an explicitly selected local proxy alongside Capture. | A session bundle with packet, application, correlation, and cleanup evidence. |

Capture can stream to an analyzer or retain a rolling window for later dumping. Encrypted traffic remains ciphertext in Capture. Deep Capture needs current compatibility evidence for the exact target and launch case; traffic that cannot reach or be inspected by the proxy is reported with its limitation. See the [Capture modes](https://fragcap.com/docs/guides/capture-modes) and [Deep Capture compatibility](https://fragcap.com/docs/reference/deep-capture-compatibility) guides for the supported paths and limits.

## Install on Windows

Published baseline: [v0.10.8](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.8).

Download the [latest public release](https://github.com/h8rt3rmin8r/fragcap/releases/latest) and verify the package against its accompanying `.sha256` file. The packages are unsigned, so this checksum checks the downloaded bytes against the published value; it is not a publisher signature.

| Download | When to choose it |
| --- | --- |
| Windows `.msi` | Installs fragcap, adds it to `PATH` for new terminals, and seeds the catalog. |
| Windows `.zip` | Runs from an unpacked directory without an installer. |
| `catalog.db` | Updates the shipped catalog separately. |

Live packet recording also requires a separately installed [Npcap](https://npcap.com) in **WinPcap API-compatible mode** and an elevated terminal. fragcap does not bundle Npcap. [Wireshark](https://www.wireshark.org) is an optional analyzer. The MSI can add a best-effort Windows Defender exclusion for its install directory; review the [packaging and migration guide](https://fragcap.com/docs/guides/packaging-and-migration) for installation, updates, and removal before choosing the installer.

## Make your first capture

Open a new terminal after MSI installation. Check prerequisites, list locally discovered targets, then use a ready target's displayed row number:

```powershell
fragcap doctor
fragcap targets
fragcap capture 1 --duration 5m --out first-capture.fcapng
```

Open `first-capture.fcapng` in a pcapng-aware analyzer. The process and role information appears in packet comments. Row numbers belong to the current `fragcap targets` listing; use a target handle when you need a stable selector. If no target is ready, the [getting-started guide](https://fragcap.com/docs/getting-started) covers registration and setup. The [CLI reference](https://fragcap.com/docs/reference/cli) covers scope, metadata-only capture, streaming, and rolling windows.

For Deep Capture, first use `fragcap calibrate <target>` to establish compatibility for your selected target and launch case. Calibration presents its effects for confirmation, including any current-user certificate trust change needed for TLS inspection. Once applicable evidence exists, `fragcap deep-capture <target> --launch` starts an explicitly scoped session. The [compatibility guide](https://fragcap.com/docs/reference/deep-capture-compatibility) explains routing, trust, and protocol limits.

## Documentation and help

- [Documentation and first-run guide](https://fragcap.com/docs/getting-started)
- [CLI reference](https://fragcap.com/docs/reference/cli), [glossary](https://fragcap.com/docs/glossary), and [library API](https://fragcap.com/docs/reference/library-api)
- [Troubleshooting](https://fragcap.com/docs/guides/doctor-and-troubleshooting) and [release history](CHANGELOG.md)
- [Architecture of record](docs/fragcap-specification.md) and [slice plans](docs/plans/README.md)

For a defect or feature request, use [GitHub Issues](https://github.com/h8rt3rmin8r/fragcap/issues/new/choose). Include the fragcap version and relevant `fragcap doctor` output. Capture files may contain addresses and session data, so review attachments before posting them publicly. Report security problems privately through the [repository security advisory process](CONTRIBUTING.md#security).

## Build and contribute

The repository pins its Rust toolchain. A normal workspace build does not require Npcap or its SDK; the live capture backend is optional. From the repository root:

```powershell
cargo build --workspace
cargo test --workspace --locked
cargo xtask ci
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development workflow, checks, and contribution guidance. The project uses [Spec Kit](https://github.com/github/spec-kit) for numbered work slices.

## License

fragcap's source and original documentation are licensed under [Apache License 2.0](LICENSE); see [NOTICE](NOTICE) for additional notices. Brand assets and bundled fonts have their own terms under [`brand/`](brand/).

## Disclaimer

fragcap is an independent open source project. It is not affiliated with, endorsed by, or sponsored by any game publisher, developer, distribution platform, or software vendor. Product names, game titles, company names, and trademarks appearing in this repository or its documentation are the property of their respective owners and are used solely to identify software that fragcap has been tested against or designed to observe. Their mention implies no relationship between those owners and this project.

fragcap is built and published for demonstration, research, and educational purposes. Capture mode is passive observation: it does not modify, inject, or replay network traffic, and it does not read, write, or attach to the memory of any other process. Deep Capture is explicit local proxy inspection for authorized workflows, not covert process instrumentation. The authors cannot control what third parties choose to do with an open source utility. Use of fragcap may nonetheless violate the terms of service of a given game or platform, and determining that is the user's responsibility rather than the project's.

fragcap is provided "as is", without warranty of any kind, as set out in sections 7 and 8 of the Apache License, Version 2.0.
