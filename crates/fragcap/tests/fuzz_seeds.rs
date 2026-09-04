// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn replay(target: &str, data: &[u8]) {
    match target {
        "http1" => {
            fragcap_proxy::fuzz_support::http1(data);
            fragcap_proxy::fuzz_support::proxy_auth(data);
        }
        "socks5" => fragcap_proxy::fuzz_support::socks5(data),
        "streaming" => fragcap_proxy::fuzz_support::streaming(data),
        "identities_quic" => fragcap_proxy::fuzz_support::identities_quic(data),
        "artifact_jsonl" => fragcap::deep_capture::fuzz_support::jsonl(data),
        "manifest" => fragcap::deep_capture::fuzz_support::manifest(data),
        other => panic!("unmapped fuzz target {other}"),
    }
}

fn cases() -> Vec<(String, PathBuf)> {
    let corpus = root().join("fuzz/corpus");
    let mut cases = Vec::new();
    for target in fs::read_dir(corpus).expect("read fuzz corpus") {
        let target = target.expect("read target corpus");
        let target_name = target.file_name().to_string_lossy().into_owned();
        for seed in fs::read_dir(target.path()).expect("read seed corpus") {
            let seed = seed.expect("read seed");
            if seed.file_type().expect("seed type").is_file() {
                cases.push((target_name.clone(), seed.path()));
            }
        }
    }
    cases.sort();
    cases
}

#[test]
fn every_committed_seed_replays_twice_in_stable_order() {
    let cases = cases();
    assert!(!cases.is_empty());
    for _ in 0..2 {
        for (target, path) in &cases {
            let data = fs::read(path).expect("read seed bytes");
            assert!(!data.is_empty(), "empty seed: {}", path.display());
            assert!(
                data.len() <= fragcap_proxy::fuzz_support::MAX_FUZZ_INPUT_BYTES,
                "oversized seed: {}",
                path.display()
            );
            replay(target, &data);
        }
    }
}

#[test]
fn oversized_inputs_are_refused_before_parser_work() {
    let input = vec![0_u8; fragcap_proxy::fuzz_support::MAX_FUZZ_INPUT_BYTES + 1];
    for target in [
        "http1",
        "socks5",
        "streaming",
        "identities_quic",
        "artifact_jsonl",
        "manifest",
    ] {
        replay(target, &input);
    }
}
