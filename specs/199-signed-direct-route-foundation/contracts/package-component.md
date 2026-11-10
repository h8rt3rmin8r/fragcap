# Package Component Contract

## Pinned upstream inputs

| Input | Identity |
| --- | --- |
| Binary asset | `https://github.com/basil00/WinDivert/releases/download/v2.2.2/WinDivert-2.2.2-A.zip` |
| Binary SHA-256 | `63cb41763bb4b20f600b6de04e991a9c2be73279e317d4d82f237b150c5f3f15` |
| Source asset | `https://codeload.github.com/basil00/WinDivert/zip/refs/tags/v2.2.2` |
| Source SHA-256 | `65ec79c9e6afa99f648a3f4d1f6db794640b40d0b65bd438770ea503ee14ecb7` |

| Entry | Source member | SHA-256 | Signature |
| --- | --- | --- | --- |
| `WinDivert.dll` | `WinDivert-2.2.2-A/x64/WinDivert.dll` | `c1e060ee19444a259b2162f8af0f3fe8c4428a1c6f694dce20de194ac8d7d9a2` | `NotSigned` |
| `WinDivert64.sys` | `WinDivert-2.2.2-A/x64/WinDivert64.sys` | `8da085332782708d8767bcace5327a6ec7283c17cfb85e40b03cd2323a90ddc2` | `Valid`, signer thumbprint `043589F75FCE2795E7F2CC3E526D46784D5DDAB3` |
| `WinDivert-LICENSE.txt` | `WinDivert-2.2.2-A/LICENSE` | measured during staging and compared to archive | not applicable |
| `WinDivert-source.zip` | complete tag archive | `65ec79c9e6afa99f648a3f4d1f6db794640b40d0b65bd438770ea503ee14ecb7` | not applicable |

The staging step must reject a failed download, archive hash mismatch, absent member, member hash mismatch, wrong PE machine, invalid driver signature or unexpected signer. The final certification repeats identity checks on ZIP and installed MSI bytes. The package notices explain the component's LGPLv3 option, corresponding source and dynamic replacement path while keeping Fragcap's Apache-2.0 identity.

The selected route's future handle open is outside S199. Static readiness must not be described as route success.
