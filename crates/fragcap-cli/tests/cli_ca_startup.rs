// SPDX-License-Identifier: Apache-2.0

#![cfg(windows)]

use std::process::{Command, Output};

fn invoke(args: &[&str]) -> Output {
    use std::os::windows::process::CommandExt;

    Command::new(env!("CARGO_BIN_EXE_fragcap"))
        .args(args)
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .output()
        .expect("hidden fragcap child exits")
}

#[test]
fn real_entry_checks_ca_before_help_version_and_non_deep_commands() {
    let status = invoke(&["--json", "ca", "status"]);
    assert!(status.status.success());
    let value: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    let state = value["state"].as_str().expect("CA state");

    for args in [
        vec!["--help"],
        vec!["--version"],
        vec!["--json", "--version"],
        vec!["--json", "ca", "status"],
    ] {
        let output = invoke(&args);
        assert!(output.status.success(), "{args:?}");
        assert!(!output.stdout.is_empty(), "{args:?}");
        let diagnostic = String::from_utf8(output.stderr).unwrap();
        if state == "ready" {
            assert!(!diagnostic.contains("installation_ca_warning"), "{args:?}");
        } else if args.contains(&"--json") {
            let warning: serde_json::Value =
                serde_json::from_str(diagnostic.trim()).expect("structured startup warning");
            assert_eq!(warning["event"], "fragcap.installation_ca_warning");
            assert_eq!(warning["state"], state);
            assert_eq!(warning["repair"], "fragcap ca repair");
        } else {
            assert!(diagnostic.contains(&format!("installation CA {state}")));
            assert!(diagnostic.contains("fragcap ca repair"));
        }
    }
}
