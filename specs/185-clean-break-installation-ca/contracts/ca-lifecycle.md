# S185 CA lifecycle contract

| Entry | Privilege | Effect | Successful result |
| --- | --- | --- | --- |
| Every CLI start | ordinary | read-only | one health observation; warning on unhealthy; stdout unchanged |
| `ca status` | ordinary | read-only | state, public fingerprint, validity, key-presence truth |
| `ca install` | elevated | exact provision/reuse | trust readback and signing proof |
| Deep Capture preflight | elevated | read-only | exact trust plus signer proof, bound to plan |
| Session teardown/Doctor session recovery | session | session resources only | installed trust unchanged |
| `ca repair` | elevated | explicit exact repair/rotation | new verified identity or retained explicit failure |
| `ca remove` | elevated | exact installation resources | root/key/metadata absent after readback |

Errors from Deep Capture preflight include the stable health state, `LocalMachine/Root`, and `fragcap ca repair`. No route, listener, bundle, launch, or trust effect is allowed first. Package rollback removes only resources created in the failed transaction. Ordinary uninstall leaves the installation identity intact.

The first S185 installation is a separate cutover event: exact identified pre-S185 Fragcap resources are removed and verified before provisioning. A failed or ambiguous removal is visible and blocks Deep Capture. The installed runtime has no legacy compatibility path.
