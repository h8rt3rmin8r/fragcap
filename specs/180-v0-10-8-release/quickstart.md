# S180 release validation

From `release/0.10.8`, preview the patch preparation, then prepare version, generated evidence, changelog and authored release notes. Run the complete local gate and verify the candidate has not advanced the current published baseline.

After candidate PR review and green exact-head checks, merge the accepted source, verify the squash merge and push one annotated v0.10.8 tag. Follow every tagged release job. Submit only normal protected crates.io approval, then independently reconcile six public files, checksum sidecars, certification authority and ten crate archives. Advance current release records through a separate checked PR and verify the deployed site.
