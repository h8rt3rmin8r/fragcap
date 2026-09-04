// SPDX-License-Identifier: Apache-2.0

//! Bounded artifact-reader entry points shared by stable replay and libFuzzer.

use super::ManifestDocument;

pub const MAX_FUZZ_INPUT_BYTES: usize = fragcap_proxy::fuzz_support::MAX_FUZZ_INPUT_BYTES;

fn bounded(data: &[u8]) -> Option<&[u8]> {
    (data.len() <= MAX_FUZZ_INPUT_BYTES).then_some(data)
}

pub fn jsonl(data: &[u8]) {
    let Some(data) = bounded(data) else { return };
    let control = data.first().copied().unwrap_or_default();
    let payload = data.get(1..).unwrap_or_default();
    match control % 4 {
        0 => {
            let _ = super::application::read_application_prefix_bytes(payload);
        }
        1 => {
            let _ = super::lifecycle::read_lifecycle_prefix_bytes(payload);
        }
        2 => {
            let _ = super::journal::read_resource_journal_bytes(payload);
        }
        _ => {
            if let Ok(text) = std::str::from_utf8(payload) {
                let _ = super::process::read_process_trace(text);
            }
        }
    }
}

pub fn manifest(data: &[u8]) {
    let Some(data) = bounded(data) else { return };
    if let Ok(document) = ManifestDocument::parse(data) {
        let serialized = serde_json::to_vec(document.value()).expect("parsed JSON serializes");
        let reparsed =
            ManifestDocument::parse(&serialized).expect("validated manifest round trips");
        assert_eq!(document.version(), reparsed.version());
    }
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = super::manifest::validate_relative_path(text);
    }
}
