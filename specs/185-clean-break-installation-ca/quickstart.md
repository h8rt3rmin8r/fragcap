# S185 controlled verification

1. Build the Windows CLI and package under the official features.
2. In an isolated Windows install environment, install the MSI and inspect the logged cutover and provisioning result. Verify one exact root in LocalMachine/Root and one protected signer in ProgramData.
3. Repeat installation, repair, and upgrade. Compare exact DER, SHA-256 fingerprint, and signer proof; expect no new root.
4. Run two controlled Deep Capture sessions, one through HTTP CONNECT and one through QUIC/H3, and compare leaf chains, plans, session manifests, and store inventory.
5. Force a managed owner-release failure while the controlled client remains live. Confirm the installation root remains trusted; proxy and route outcomes remain independent.
6. Remove or alter each health resource in isolated fixtures. Confirm startup warning and pre-effect Deep Capture refusal with named repair guidance, while ordinary Capture/help output remains usable.
7. Inspect MSI/ZIP bytes and staged session artifacts for private key absence, then run `cargo xtask ci` and the applicable package certification gate in the foreground.

Do not run visible GUI or screen-reader acceptance on the owner's active desktop. Real-title and account evidence stays private.
