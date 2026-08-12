// SPDX-License-Identifier: Apache-2.0

//! The resolution order of specification section 15.3.
//!
//! Directories are built under the scratch directory Cargo provides for
//! integration tests, so no temporary-file dependency is needed and the layout is
//! visible in the test rather than assumed from the environment.
//!
//! Every assertion is on [`ProfileSource`] rather than only on the profile's
//! contents. Two steps can hold identical files, so a test that checked only the
//! contents would pass for the wrong reason.

use std::fs;
use std::path::{Path, PathBuf};

use fragcap_profile::{
    resolve, BundledSet, LoadError, Profile, ProfileSource, ResolveError, SearchPath,
};

/// A valid JSON profile with the given id, and a name that says where it came
/// from.
fn profile_text(id: &str, name: &str) -> String {
    format!(
        r#"{{"schema":1,"kind":"profile","fidelity":"verified","game":{{"id":"{id}","name":"{name}"}},"stage":[{{"role":"client","lifecycle":"session","match":{{"exe":"c.exe"}}}}]}}"#
    )
}

/// A scratch directory unique to one test.
fn scratch(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("resolution")
        .join(test);
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear scratch");
    }
    fs::create_dir_all(&dir).expect("create scratch");
    dir
}

fn write(dir: &Path, file: &str, contents: &str) -> PathBuf {
    fs::create_dir_all(dir).expect("create directory");
    let path = dir.join(file);
    fs::write(&path, contents).expect("write profile");
    path
}

fn bundled(id: &str, name: &str) -> BundledSet {
    let p = Profile::parse(&profile_text(id, name)).expect("valid bundled profile");
    BundledSet::new(vec![p]).expect("no duplicate id")
}

#[test]
fn step_one_takes_an_existing_file_wherever_it_is() {
    let dir = scratch("step_one");
    let path = write(&dir, "anywhere.json", &profile_text("eso", "from path"));

    let got = resolve(
        path.to_str().expect("utf-8 path"),
        &SearchPath::new(),
        &BundledSet::empty(),
    )
    .expect("resolves");

    assert_eq!(got.profile.game().name(), "from path");
    assert_eq!(got.source, ProfileSource::ExplicitPath(path));
}

#[test]
fn step_one_does_not_require_the_reference_to_be_a_slug() {
    // A path carries separators and often an extension, neither of which is a
    // valid id. The operator named a file; that is the whole point of step one.
    let dir = scratch("step_one_not_slug");
    let path = write(&dir, "Not-A-Slug.Profile.json", &profile_text("eso", "ok"));

    let got = resolve(
        path.to_str().expect("utf-8 path"),
        &SearchPath::new(),
        &BundledSet::empty(),
    )
    .expect("resolves");
    assert!(matches!(got.source, ProfileSource::ExplicitPath(_)));
}

#[test]
fn a_directory_does_not_satisfy_step_one() {
    let dir = scratch("directory_reference");
    let sub = dir.join("eso");
    fs::create_dir_all(&sub).expect("create directory");
    // A directory named exactly like the reference, plus a real profile beside
    // it. Step one must not take the directory; step two must find the file.
    write(&dir, "eso.json", &profile_text("eso", "the file"));

    let search = SearchPath {
        command_line: vec![dir.clone()],
        user: None,
    };
    let got = resolve("eso", &search, &BundledSet::empty()).expect("resolves");
    assert_eq!(got.profile.game().name(), "the file");
    assert_eq!(
        got.source,
        ProfileSource::CommandLineDirectory(dir.join("eso.json")),
        "a bare name that is also a directory still means a profile"
    );
}

#[test]
fn a_command_line_directory_shadows_the_user_directory() {
    let root = scratch("shadow_command_line");
    let cli = root.join("cli");
    let user = root.join("user");
    write(&cli, "eso.json", &profile_text("eso", "from cli"));
    write(&user, "eso.json", &profile_text("eso", "from user"));

    let search = SearchPath {
        command_line: vec![cli.clone()],
        user: Some(user),
    };
    let got = resolve("eso", &search, &BundledSet::empty()).expect("resolves");
    assert_eq!(got.profile.game().name(), "from cli");
    assert_eq!(
        got.source,
        ProfileSource::CommandLineDirectory(cli.join("eso.json"))
    );
}

#[test]
fn the_user_directory_shadows_a_bundled_profile() {
    // The reason section 15.3 places bundled profiles last: a bundled profile that
    // has drifted from a game update is corrected locally without a release.
    let root = scratch("shadow_user");
    let user = root.join("user");
    write(&user, "eso.json", &profile_text("eso", "from user"));

    let search = SearchPath {
        command_line: Vec::new(),
        user: Some(user.clone()),
    };
    let got = resolve("eso", &search, &bundled("eso", "from bundle")).expect("resolves");
    assert_eq!(got.profile.game().name(), "from user");
    assert_eq!(
        got.source,
        ProfileSource::UserDirectory(user.join("eso.json"))
    );
}

#[test]
fn step_four_matches_on_game_id() {
    let got = resolve("eso", &SearchPath::new(), &bundled("eso", "from bundle")).expect("resolves");
    assert_eq!(got.profile.game().name(), "from bundle");
    assert_eq!(got.source, ProfileSource::Bundled);
}

#[test]
fn command_line_directories_are_consulted_in_order() {
    let root = scratch("cli_order");
    let first = root.join("first");
    let second = root.join("second");
    write(&first, "eso.json", &profile_text("eso", "first"));
    write(&second, "eso.json", &profile_text("eso", "second"));

    let search = SearchPath {
        command_line: vec![first.clone(), second],
        user: None,
    };
    let got = resolve("eso", &search, &BundledSet::empty()).expect("resolves");
    assert_eq!(got.profile.game().name(), "first");
    assert_eq!(
        got.source,
        ProfileSource::CommandLineDirectory(first.join("eso.json"))
    );
}

#[test]
fn an_absent_search_directory_is_skipped_rather_than_an_error() {
    // A missing user configuration directory is the ordinary state of a fresh
    // install, not a failure.
    let root = scratch("absent_directory");
    let user = root.join("user");
    write(&user, "eso.json", &profile_text("eso", "from user"));

    let search = SearchPath {
        command_line: vec![root.join("does-not-exist")],
        user: Some(user),
    };
    let got = resolve("eso", &search, &BundledSet::empty()).expect("resolves");
    assert_eq!(got.profile.game().name(), "from user");
}

#[test]
fn an_absent_search_directory_is_still_reported_as_searched() {
    // Regression, PR 11 review. Skipping an absent directory is right; leaving it
    // out of the failure report is not. A search consisting only of a missing
    // directory used to fail with an empty list and the message "no profile
    // directories were given", which is false when one was given. The answer an
    // operator needs here is where to put the file.
    let root = scratch("absent_is_reported");
    let missing = root.join("missing");

    let search = SearchPath {
        command_line: vec![missing.clone()],
        user: None,
    };
    match resolve("eso", &search, &BundledSet::empty()) {
        Err(ResolveError::NotFound {
            reference,
            searched,
        }) => {
            assert_eq!(reference, "eso");
            assert_eq!(
                searched,
                vec![missing.join("eso.json")],
                "a directory the caller supplied must appear whether or not it exists"
            );
            let rendered = ResolveError::NotFound {
                reference,
                searched,
            }
            .to_string();
            assert!(
                !rendered.contains("none were given"),
                "the message must not claim nothing was supplied: {rendered}"
            );
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn a_search_with_no_directories_at_all_says_so() {
    // The case the message above is reserved for: nothing was supplied, so there
    // is genuinely nowhere to name.
    match resolve("eso", &SearchPath::new(), &BundledSet::empty()) {
        Err(ResolveError::NotFound { searched, .. }) => {
            assert!(searched.is_empty());
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn a_candidate_that_wins_its_step_and_cannot_be_used_is_an_error_not_a_fallthrough() {
    // The property that matters: the file has already won its step, so falling
    // through would silently hand the operator a profile they did not choose. An
    // unreadable file and an invalid one reach this through the same path, and the
    // invalid one is the case a test can build on any machine.
    let root = scratch("wins_and_fails");
    let user = root.join("user");
    write(&user, "eso.json", "schema = 1\nthis is not toml\n");

    let search = SearchPath {
        command_line: Vec::new(),
        user: Some(user.clone()),
    };
    match resolve("eso", &search, &bundled("eso", "from bundle")) {
        Err(ResolveError::Load { path, source }) => {
            assert_eq!(path, user.join("eso.json"));
            assert!(matches!(source, LoadError::Invalid(_)));
        }
        Ok(got) => panic!(
            "resolution fell through to {:?}, silently substituting a profile the \
             operator did not choose",
            got.source
        ),
        Err(other) => panic!("expected a load error, got {other:?}"),
    }
}

#[test]
fn a_reference_that_is_not_a_slug_is_refused_before_any_path_is_joined() {
    // The rule is a check, not a failed open. A test that asserted NotFound would
    // pass today because nothing is at the traversal target, and stop passing on a
    // machine where something is.
    let root = scratch("traversal");
    let user = root.join("user");
    fs::create_dir_all(&user).expect("create");

    let search = SearchPath {
        command_line: Vec::new(),
        user: Some(user),
    };

    for reference in [
        "../../../windows/system32/drivers/etc/hosts",
        "..",
        "a/b",
        "a\\b",
        "C:\\Windows\\eso",
        "ESO",
        "eso.json",
        "eso profile",
    ] {
        match resolve(reference, &search, &BundledSet::empty()) {
            Err(ResolveError::InvalidReference { reference: got }) => assert_eq!(got, reference),
            other => {
                panic!("reference {reference:?} must be refused before any join, got {other:?}")
            }
        }
    }
}

#[test]
fn a_reference_matching_nothing_names_everywhere_it_looked() {
    let root = scratch("not_found");
    let cli = root.join("cli");
    let user = root.join("user");
    fs::create_dir_all(&cli).expect("create");
    fs::create_dir_all(&user).expect("create");

    let search = SearchPath {
        command_line: vec![cli.clone()],
        user: Some(user.clone()),
    };
    match resolve("absent", &search, &BundledSet::empty()) {
        Err(ResolveError::NotFound {
            reference,
            searched,
        }) => {
            assert_eq!(reference, "absent");
            assert_eq!(
                searched,
                vec![cli.join("absent.json"), user.join("absent.json")],
                "the question an operator asks on this failure is always where you looked"
            );
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn two_bundled_profiles_with_one_id_are_refused_at_construction() {
    let a = Profile::parse(&profile_text("eso", "one")).expect("valid");
    let b = Profile::parse(&profile_text("eso", "two")).expect("valid");
    let err = BundledSet::new(vec![a, b]).expect_err("a duplicate id must be refused");
    assert_eq!(err.0, "eso");
    assert!(
        err.to_string().contains("step four"),
        "the message should say why it matters: {err}"
    );
}

#[test]
fn a_bundled_set_with_distinct_ids_is_accepted() {
    let a = Profile::parse(&profile_text("eso", "one")).expect("valid");
    let b = Profile::parse(&profile_text("div2", "two")).expect("valid");
    let set = BundledSet::new(vec![a, b]).expect("distinct ids");
    assert_eq!(set.len(), 2);
    assert!(!set.is_empty());
    assert_eq!(set.get("eso").map(|p| p.game().name()), Some("one"));
    assert_eq!(set.get("absent"), None);
}

#[test]
fn the_bundled_set_this_slice_ships_is_empty() {
    // Section 15.5 ships profiles for the two focal titles at v0.1.0. A bundled
    // profile is a claim about a game's current process topology, and the slices
    // that can verify such a claim own it. The resolver's ability to consult a set
    // is what ships here.
    assert!(BundledSet::empty().is_empty());
}

#[test]
fn the_source_is_reported_for_every_step() {
    let root = scratch("sources");
    let cli = root.join("cli");
    let user = root.join("user");
    let explicit = write(&root, "explicit.json", &profile_text("eso", "x"));
    write(&cli, "eso.json", &profile_text("eso", "c"));
    write(&user, "div2.json", &profile_text("div2", "u"));

    let search = SearchPath {
        command_line: vec![cli.clone()],
        user: Some(user.clone()),
    };
    let set = bundled("third", "b");

    let by_path = resolve(explicit.to_str().expect("utf-8"), &search, &set).expect("resolves");
    assert_eq!(by_path.source, ProfileSource::ExplicitPath(explicit));

    let by_cli = resolve("eso", &search, &set).expect("resolves");
    assert_eq!(
        by_cli.source,
        ProfileSource::CommandLineDirectory(cli.join("eso.json"))
    );

    let by_user = resolve("div2", &search, &set).expect("resolves");
    assert_eq!(
        by_user.source,
        ProfileSource::UserDirectory(user.join("div2.json"))
    );

    let by_bundle = resolve("third", &search, &set).expect("resolves");
    assert_eq!(by_bundle.source, ProfileSource::Bundled);
}

#[test]
fn a_resolved_profile_is_validated() {
    // Resolution cannot produce an unvalidated profile, because `Profile` has no
    // other constructor. Asserted here so that a future change to the load path
    // cannot quietly bypass validation.
    let root = scratch("validated");
    let user = root.join("user");
    write(
        &user,
        "eso.json",
        // Two unpinned stages that can bind one process: refused by the
        // ambiguity check, not by the parser (valid JSON, semantic fault).
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"eso","name":"T"},"stage":[{"role":"a","lifecycle":"transient","match":{"exe":"x.exe"}},{"role":"b","lifecycle":"session","match":{"exe":"x.exe"}}]}"#,
    );

    let search = SearchPath {
        command_line: Vec::new(),
        user: Some(user),
    };
    assert!(matches!(
        resolve("eso", &search, &BundledSet::empty()),
        Err(ResolveError::Load {
            source: LoadError::Invalid(_),
            ..
        })
    ));
}
