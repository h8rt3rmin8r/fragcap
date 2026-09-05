// SPDX-License-Identifier: Apache-2.0

//! Exact compile-time identity consumed by final package certification.

use std::io::Write;

use crate::exit::{CliError, Exit};

pub fn run(out: &mut dyn Write) -> Result<Exit, CliError> {
    let mut features = vec!["native-deep-capture"];
    if cfg!(feature = "etw") {
        features.push("etw");
    }
    if cfg!(feature = "live") {
        features.push("live");
    }
    if cfg!(feature = "socket-table") {
        features.push("socket-table");
    }
    features.sort_unstable();
    let identity = serde_json::json!({
        "schema_version": 1,
        "product": "fragcap",
        "version": env!("CARGO_PKG_VERSION"),
        "source_revision": env!("FRAGCAP_BUILD_SOURCE_REVISION"),
        "target": env!("FRAGCAP_BUILD_TARGET"),
        "architecture": std::env::consts::ARCH,
        "features": features,
        "deep_capture_backend": "fragcap-native",
        "official": env!("FRAGCAP_BUILD_OFFICIAL") == "true"
    });
    serde_json::to_writer(&mut *out, &identity)
        .map_err(|error| CliError::failure(format!("could not render build identity: {error}")))?;
    writeln!(out)
        .map_err(|error| CliError::failure(format!("could not write build identity: {error}")))?;
    Ok(Exit::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_exact_machine_readable_native_truth() {
        let mut out = Vec::new();
        assert_eq!(run(&mut out).unwrap(), Exit::SUCCESS);
        let value: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["product"], "fragcap");
        assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["deep_capture_backend"], "fragcap-native");
        assert!(value["features"]
            .as_array()
            .unwrap()
            .iter()
            .any(|feature| feature == "native-deep-capture"));
    }
}
