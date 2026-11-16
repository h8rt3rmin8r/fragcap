// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for the CLI tier-1 tests.
//!
//! Every test drives the library entry [`fragcap_cli::run_with`] directly,
//! never a spawned process, capturing the command-result stream and the
//! diagnostics stream separately. Goldens are compared byte for byte and
//! regenerated with `FRAGCAP_UPDATE_GOLDENS=1`, the same discipline the corpus
//! goldens follow.

#![allow(dead_code)]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static CLI_ENV_LOCK: Mutex<()> = Mutex::new(());

struct ExactTestAuthorization;

impl fragcap_cli::DeepCaptureAuthorizationInput for ExactTestAuthorization {
    fn is_terminal(&self) -> bool {
        true
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        Ok(if exact {
            format!("{plan_id}\n").into_bytes()
        } else {
            b"yes\n".to_vec()
        })
    }
}

/// Build an argv, prepending the program name clap expects at position zero.
pub fn argv(list: &[&str]) -> Vec<OsString> {
    std::iter::once("fragcap".to_string())
        .chain(list.iter().map(|s| s.to_string()))
        .map(OsString::from)
        .collect()
}

thread_local! {
    // Each harness thread keeps its default store across CLI calls. Its destructor
    // closes the exact root when the test thread exits, including assertion panic.
    static THREAD_ENV: TestEnvironment = TestEnvironment::new();
}

/// Own a stable CLI environment across several in-process calls in one test.
pub struct TestEnvironment {
    root: Option<tempfile::TempDir>,
    local_db: PathBuf,
    session_dir: PathBuf,
    catalog_db: PathBuf,
}

impl TestEnvironment {
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("fragcap-cli-test-environment-")
            .tempdir()
            .expect("create unique CLI test environment");
        let canonical = root.path().canonicalize().expect("canonical CLI test root");
        fs::write(
            root.path().join("test-owner.txt"),
            format!(
                "owner_pid={}\ncanonical_root={}\n",
                std::process::id(),
                canonical.display()
            ),
        )
        .expect("write exact CLI test owner marker");
        Self {
            local_db: root.path().join("local.db"),
            session_dir: root.path().join("sessions"),
            catalog_db: root.path().join("nonexistent-catalog.db"),
            root: Some(root),
        }
    }

    pub fn run(&self, list: &[&str]) -> (u8, String, String) {
        self.run_with_authorization(list, &mut ExactTestAuthorization)
    }

    fn run_with_authorization(
        &self,
        list: &[&str],
        authorization: &mut dyn fragcap_cli::DeepCaptureAuthorizationInput,
    ) -> (u8, String, String) {
        let _guard = CLI_ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let keys = [
            "FRAGCAP_CATALOG_DB",
            "FRAGCAP_LOCAL_DB",
            "FRAGCAP_SESSION_DIR",
        ];
        let previous: Vec<_> = keys.iter().map(std::env::var_os).collect();
        std::env::set_var(keys[0], &self.catalog_db);
        std::env::set_var(keys[1], &self.local_db);
        std::env::set_var(keys[2], &self.session_dir);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_unlocked(list, authorization)
        }));
        for (key, value) in keys.into_iter().zip(previous) {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
        match result {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    fn close(&mut self) -> std::io::Result<()> {
        let root = self.root.take().expect("CLI root is owned once");
        let path = root.path().to_path_buf();
        root.close()?;
        if path.exists() {
            Err(std::io::Error::other(format!(
                "owned CLI root remains after close: {}",
                path.display()
            )))
        } else {
            Ok(())
        }
    }

    pub fn local_db(&self) -> &Path {
        &self.local_db
    }

    pub fn session_dir(&self) -> &Path {
        &self.session_dir
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if self.root.is_some() {
            if let Err(error) = self.close() {
                eprintln!("CLI test cleanup failed: {error}");
            }
        }
    }
}

pub fn with_test_environment<T>(body: impl FnOnce(&TestEnvironment) -> T) -> T {
    let mut environment = TestEnvironment::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(&environment)));
    let cleanup = environment.close();
    if let Err(error) = cleanup {
        if result.is_err() {
            eprintln!("CLI test cleanup also failed: {error}");
        } else {
            panic!("CLI test cleanup failed: {error}");
        }
    }
    match result {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

/// Run the command surface, returning the exit code, the stdout result stream,
/// and the stderr diagnostics stream.
pub fn run(list: &[&str]) -> (u8, String, String) {
    run_with_authorization(list, &mut ExactTestAuthorization)
}

/// Run with a caller-controlled Deep Capture authorization boundary.
pub fn run_with_authorization(
    list: &[&str],
    authorization: &mut dyn fragcap_cli::DeepCaptureAuthorizationInput,
) -> (u8, String, String) {
    THREAD_ENV.with(|environment| environment.run_with_authorization(list, authorization))
}

fn run_unlocked(
    list: &[&str],
    authorization: &mut dyn fragcap_cli::DeepCaptureAuthorizationInput,
) -> (u8, String, String) {
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let exit = fragcap_cli::run_with_authorization(argv(list), authorization, &mut out, &mut err);
    (
        exit.code(),
        String::from_utf8(out).expect("stdout is UTF-8"),
        String::from_utf8(err).expect("stderr is UTF-8"),
    )
}

/// The committed capture fixtures shared with the rest of the workspace.
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
}

/// The CLI test data (profiles, process scripts) committed beside these tests.
pub fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

/// The CLI golden directory.
pub fn goldens_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("goldens")
}

/// A fixture path as a string, for passing on the command line.
pub fn fixture(name: &str) -> String {
    fixtures_dir().join(name).to_string_lossy().into_owned()
}

/// A data path as a string, for passing on the command line.
pub fn data(name: &str) -> String {
    data_dir().join(name).to_string_lossy().into_owned()
}

/// Compare `produced` against the named golden, or write it when
/// `FRAGCAP_UPDATE_GOLDENS` is set.
pub fn assert_golden(name: &str, produced: &[u8]) {
    let path = goldens_dir().join(name);
    if std::env::var_os("FRAGCAP_UPDATE_GOLDENS").is_some() {
        fs::create_dir_all(goldens_dir()).expect("golden directory");
        fs::write(&path, produced).expect("write golden");
        return;
    }
    let want = fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "golden {} must exist ({e}); regenerate with FRAGCAP_UPDATE_GOLDENS=1",
            path.display()
        )
    });
    if want != produced {
        panic!(
            "{name}: output differs from the committed golden. If the change is intended, \
             regenerate with FRAGCAP_UPDATE_GOLDENS=1 and read the diff.\n\
             golden {} bytes, produced {} bytes",
            want.len(),
            produced.len()
        );
    }
}

/// Replace every RFC3339 timestamp value with a placeholder so an event stream
/// is comparable to a golden without depending on the wall clock.
pub fn normalize_timestamps(stream: &str) -> String {
    let mut out = String::with_capacity(stream.len());
    for line in stream.lines() {
        if let Some(rest) = line.strip_prefix("{\"ts\":\"") {
            if let Some(end) = rest.find('"') {
                out.push_str("{\"ts\":\"<ts>\"");
                out.push_str(&rest[end + 1..]);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
