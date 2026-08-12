// SPDX-License-Identifier: Apache-2.0

//! User-facing `--help` is free of internal roadmap identifiers and
//! argument-parser implementation notes (issues #66, #67).

mod common;

use common::run;

/// Help output must carry no internal slice id and no parser implementation note.
fn assert_help_is_clean(args: &[&str]) {
    let (code, out, _err) = run(args);
    assert_eq!(code, 0, "{args:?} help exits 0");

    for leak in ["S15", "S16", "S17", "slice S"] {
        assert!(
            !out.contains(leak),
            "{args:?} help leaks the internal identifier {leak}:\n{out}"
        );
    }
    for leak in ["value_parser", "value_delimiter", "Vec<String>"] {
        assert!(
            !out.contains(leak),
            "{args:?} help leaks the parser note {leak}:\n{out}"
        );
    }
}

#[test]
fn run_help_is_free_of_internals() {
    assert_help_is_clean(&["run", "--help"]);
}

#[test]
fn extcap_help_is_free_of_internals() {
    assert_help_is_clean(&["extcap", "--help"]);
}

#[test]
fn top_level_help_is_free_of_internals() {
    assert_help_is_clean(&["--help"]);
}

#[test]
fn launch_help_describes_real_behavior_not_a_deferred_slice() {
    let (_code, out, _err) = run(&["run", "--help"]);
    assert!(
        out.contains("launcher"),
        "the --launch help describes the real managed launch: {out}"
    );
    assert!(
        !out.contains("deferred"),
        "the --launch help is not a 'deferred to slice' note: {out}"
    );
}
