`cargo xtask license` checks that every publishable crate carries the license
text a published package needs, comparing each copy against the repository root
original byte for byte. It runs as part of `cargo xtask ci`, so a copy that
drifts fails the build rather than reaching a published version that can be
yanked but never corrected.
