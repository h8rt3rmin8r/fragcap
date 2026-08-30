// SPDX-License-Identifier: Apache-2.0

//! Managed launch values shared by Capture and Deep Capture.
//!
//! Preparation resolves a stored target before any capture, proxy, trust, or
//! process effect. Execution consumes only the retained program, working
//! directory, argument vector, and child-scoped environment. Direct launch never
//! invokes a command shell and retains no process controller after creation.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::targets::{entry_windows_launch_entries, TargetEntry};

/// One immutable managed launch prepared before capture effects begin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagedLaunch {
    /// Existing Steam protocol managed launch.
    Steam(crate::steam::LaunchRequest),
    /// Exact direct child-process creation.
    Direct(DirectExecutableLaunch),
}

impl ManagedLaunch {
    /// Return a copy with child-scoped environment additions.
    ///
    /// Steam protocol dispatch cannot guarantee environment inheritance through
    /// an already-running client, so only direct launches accept this operation.
    pub fn with_environment<I, K, V>(self, entries: I) -> Result<Self, LaunchConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        match self {
            Self::Steam(_) => Err(LaunchConfigError::EnvironmentUnsupported),
            Self::Direct(mut direct) => {
                for (key, value) in entries {
                    let key = key.into();
                    validate_environment_key(&key)?;
                    direct.environment.insert(key, value.into());
                }
                Ok(Self::Direct(direct))
            }
        }
    }

    /// Issue the exact prepared launch.
    pub fn execute(&self) -> Result<LaunchReceipt, LaunchError> {
        match self {
            Self::Steam(request) => {
                crate::steam::launch(request)
                    .map_err(|error| LaunchError::Steam(error.to_string()))?;
                Ok(LaunchReceipt { process_id: None })
            }
            Self::Direct(direct) => direct.execute(),
        }
    }
}

/// Exact direct child-process configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectExecutableLaunch {
    executable: PathBuf,
    working_directory: PathBuf,
    arguments: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
}

impl DirectExecutableLaunch {
    /// Construct a direct launch from already-validated exact paths and argv.
    pub fn new(
        executable: PathBuf,
        working_directory: PathBuf,
        arguments: Vec<OsString>,
    ) -> Result<Self, LaunchConfigError> {
        if !executable.is_absolute() || !working_directory.is_absolute() {
            return Err(LaunchConfigError::PathsMustBeAbsolute);
        }
        let working_directory =
            working_directory
                .canonicalize()
                .map_err(|source| LaunchConfigError::Path {
                    path: working_directory,
                    source,
                })?;
        let executable = executable
            .canonicalize()
            .map_err(|source| LaunchConfigError::Path {
                path: executable,
                source,
            })?;
        validate_direct_paths(&executable, &working_directory)?;
        Ok(Self {
            executable,
            working_directory,
            arguments,
            environment: BTreeMap::new(),
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }

    pub fn environment(&self) -> &BTreeMap<OsString, OsString> {
        &self.environment
    }

    fn execute(&self) -> Result<LaunchReceipt, LaunchError> {
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.working_directory)
            .args(&self.arguments)
            .envs(&self.environment);
        let child = command.spawn().map_err(|source| LaunchError::Direct {
            executable: self.executable.clone(),
            source,
        })?;
        let process_id = child.id();
        drop(child);
        Ok(LaunchReceipt {
            process_id: Some(process_id),
        })
    }
}

/// Prepare one direct launch from the existing stored target facts.
pub fn prepare_direct_launch(target: &TargetEntry) -> Result<ManagedLaunch, LaunchConfigError> {
    let launches = entry_windows_launch_entries(target);
    let launch = match launches.as_slice() {
        [] => return Err(LaunchConfigError::MissingClient),
        [launch] => launch,
        _ => {
            return Err(LaunchConfigError::AmbiguousClient(
                launches
                    .iter()
                    .map(|launch| launch.executable().to_string())
                    .collect(),
            ));
        }
    };
    let client = launch.executable();
    let stored_client = Path::new(client);
    let root = match target.install_root.as_deref() {
        Some(root) => PathBuf::from(root),
        None if stored_client.is_absolute() => stored_client
            .parent()
            .map(Path::to_path_buf)
            .ok_or(LaunchConfigError::MissingInstallRoot)?,
        None => return Err(LaunchConfigError::MissingInstallRoot),
    };
    let canonical_root = root
        .canonicalize()
        .map_err(|source| LaunchConfigError::Path { path: root, source })?;
    let candidate = if stored_client.is_absolute() {
        stored_client.to_path_buf()
    } else {
        canonical_root.join(stored_client)
    };
    let executable = candidate
        .canonicalize()
        .map_err(|source| LaunchConfigError::Path {
            path: candidate,
            source,
        })?;
    if !executable.starts_with(&canonical_root) {
        return Err(LaunchConfigError::OutsideInstallRoot(executable));
    }
    let arguments = parse_stored_arguments(launch.arguments.as_deref())?;
    DirectExecutableLaunch::new(executable, canonical_root, arguments).map(ManagedLaunch::Direct)
}

#[cfg(windows)]
fn parse_stored_arguments(raw: Option<&str>) -> Result<Vec<OsString>, LaunchConfigError> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::Memory::LocalFree;
    use windows_sys::Win32::UI::Shell::CommandLineToArgvW;

    let Some(raw) = raw.filter(|raw| !raw.is_empty()) else {
        return Ok(Vec::new());
    };
    if raw.contains('\0') {
        return Err(LaunchConfigError::InvalidArguments(
            "stored arguments contain a NUL character".into(),
        ));
    }

    // CommandLineToArgvW treats argv[0] specially. Prefix a fixed synthetic
    // program token, parse the stored argument fragment, then discard that token.
    let command_line: Vec<u16> = format!("fragcap-managed-launch {raw}\0")
        .encode_utf16()
        .collect();
    let mut count = 0;
    // SAFETY: command_line is NUL terminated and count is a valid out pointer.
    let argv = unsafe { CommandLineToArgvW(command_line.as_ptr(), &mut count) };
    if argv.is_null() {
        return Err(LaunchConfigError::InvalidArguments(
            std::io::Error::last_os_error().to_string(),
        ));
    }

    let mut arguments = Vec::with_capacity(count.saturating_sub(1) as usize);
    for index in 1..count {
        // SAFETY: CommandLineToArgvW returned count valid NUL-terminated pointers.
        let value = unsafe { *argv.add(index as usize) };
        let mut len = 0;
        // SAFETY: value points to a NUL-terminated UTF-16 string in argv's block.
        while unsafe { *value.add(len) } != 0 {
            len += 1;
        }
        // SAFETY: len was found within the NUL-terminated allocation.
        let units = unsafe { std::slice::from_raw_parts(value, len) };
        arguments.push(OsString::from_wide(units));
    }
    // SAFETY: argv is the allocation returned by CommandLineToArgvW and is freed once.
    unsafe { LocalFree(argv as isize) };
    Ok(arguments)
}

#[cfg(not(windows))]
fn parse_stored_arguments(raw: Option<&str>) -> Result<Vec<OsString>, LaunchConfigError> {
    match raw.filter(|raw| !raw.is_empty()) {
        None => Ok(Vec::new()),
        Some(_) => Err(LaunchConfigError::InvalidArguments(
            "stored Windows arguments can be parsed only on Windows".into(),
        )),
    }
}

fn validate_direct_paths(
    executable: &Path,
    working_directory: &Path,
) -> Result<(), LaunchConfigError> {
    if !executable.is_absolute() || !working_directory.is_absolute() {
        return Err(LaunchConfigError::PathsMustBeAbsolute);
    }
    if !working_directory.is_dir() {
        return Err(LaunchConfigError::WorkingDirectory(
            working_directory.to_path_buf(),
        ));
    }
    if !executable.is_file() {
        return Err(LaunchConfigError::Executable(executable.to_path_buf()));
    }
    if !executable.starts_with(working_directory) {
        return Err(LaunchConfigError::OutsideInstallRoot(
            executable.to_path_buf(),
        ));
    }
    Ok(())
}

fn validate_environment_key(key: &OsStr) -> Result<(), LaunchConfigError> {
    let text = key.to_string_lossy();
    if text.is_empty() || text.contains('=') || text.contains('\0') {
        Err(LaunchConfigError::InvalidEnvironmentKey(key.to_os_string()))
    } else {
        Ok(())
    }
}

/// Side-effect-free managed-launch preparation failure.
#[derive(Debug)]
pub enum LaunchConfigError {
    MissingInstallRoot,
    MissingClient,
    AmbiguousClient(Vec<String>),
    Path {
        path: PathBuf,
        source: std::io::Error,
    },
    PathsMustBeAbsolute,
    WorkingDirectory(PathBuf),
    Executable(PathBuf),
    OutsideInstallRoot(PathBuf),
    EnvironmentUnsupported,
    InvalidEnvironmentKey(OsString),
    InvalidArguments(String),
}

impl fmt::Display for LaunchConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInstallRoot => {
                f.write_str("direct managed launch requires a stored install root")
            }
            Self::MissingClient => {
                f.write_str("direct managed launch requires one resolved Windows client executable")
            }
            Self::AmbiguousClient(clients) => write!(
                f,
                "direct managed launch found multiple Windows client executables: {}",
                clients.join(", ")
            ),
            Self::Path { path, source } => write!(
                f,
                "cannot resolve managed-launch path {}: {source}",
                path.display()
            ),
            Self::PathsMustBeAbsolute => {
                f.write_str("direct managed-launch paths must be absolute")
            }
            Self::WorkingDirectory(path) => write!(
                f,
                "managed-launch working directory is not a directory: {}",
                path.display()
            ),
            Self::Executable(path) => write!(
                f,
                "managed-launch executable is not a file: {}",
                path.display()
            ),
            Self::OutsideInstallRoot(path) => write!(
                f,
                "managed-launch executable resolves outside the stored install root: {}",
                path.display()
            ),
            Self::EnvironmentUnsupported => f.write_str(
                "target-scoped environment is supported only for direct-executable launch",
            ),
            Self::InvalidEnvironmentKey(key) => {
                write!(f, "managed-launch environment key is invalid: {:?}", key)
            }
            Self::InvalidArguments(message) => {
                write!(f, "stored managed-launch arguments are invalid: {message}")
            }
        }
    }
}

impl std::error::Error for LaunchConfigError {}

/// Runtime managed-launch failure.
#[derive(Debug)]
pub enum LaunchError {
    Steam(String),
    Direct {
        executable: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Steam(message) => message.fmt(f),
            Self::Direct { executable, source } => {
                write!(f, "cannot launch {}: {source}", executable.display())
            }
        }
    }
}

impl std::error::Error for LaunchError {}

/// Observable result of issuing a managed launch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaunchReceipt {
    process_id: Option<u32>,
}

impl LaunchReceipt {
    pub fn process_id(self) -> Option<u32> {
        self.process_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::FidelityTier;
    use crate::targets::{ClassificationSource, TargetClassification};
    use serde_json::json;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    fn target(root: &Path, launch_entries: serde_json::Value) -> TargetEntry {
        TargetEntry {
            id: Some(1),
            stable_id: 1,
            handle: "direct".into(),
            name: "Direct".into(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(launch_entries),
            install_root: Some(root.to_string_lossy().into_owned()),
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        }
    }

    fn unique_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "fragcap-managed-launch-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[cfg(windows)]
    #[test]
    fn stored_target_resolves_one_client_beneath_install_root() {
        let root = unique_dir("prepare");
        let bin = root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("game.exe"), b"fixture").unwrap();
        let entry = target(
            &root,
            json!([{ "executable": "bin/game.exe", "role": "client" }]),
        );
        let mut entries = entry.launch_entries.unwrap();
        entries[0]["arguments"] = json!("one \"two words\" \"\" snowman-☃");
        let entry = target(&root, entries);
        let launch = prepare_direct_launch(&entry).unwrap();
        let ManagedLaunch::Direct(direct) = launch else {
            panic!("expected direct launch");
        };
        assert_eq!(direct.working_directory(), root.canonicalize().unwrap());
        assert_eq!(
            direct.executable(),
            bin.join("game.exe").canonicalize().unwrap()
        );
        assert_eq!(
            direct.arguments(),
            [
                OsString::from("one"),
                OsString::from("two words"),
                OsString::from(""),
                OsString::from("snowman-☃")
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn authored_absolute_client_supplies_its_legacy_install_root() {
        let root = unique_dir("authored-absolute");
        fs::create_dir_all(&root).unwrap();
        let executable = root.join("game.exe");
        fs::write(&executable, b"fixture").unwrap();
        let mut entry = target(
            &root,
            json!([{ "executable": executable, "role": "client" }]),
        );
        entry.install_root = None;

        let launch = prepare_direct_launch(&entry).unwrap();
        let ManagedLaunch::Direct(direct) = launch else {
            panic!("expected direct launch");
        };
        assert_eq!(direct.working_directory(), root.canonicalize().unwrap());
        assert_eq!(direct.executable(), executable.canonicalize().unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preparation_refuses_escape_and_ambiguity() {
        let root = unique_dir("refuse");
        fs::create_dir_all(&root).unwrap();
        let outside = root.parent().unwrap().join(format!(
            "outside-{}.exe",
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&outside, b"fixture").unwrap();
        let escaping_path = format!("../{}", outside.file_name().unwrap().to_string_lossy());
        let escaping = target(
            &root,
            json!([{ "executable": escaping_path, "role": "client" }]),
        );
        assert!(matches!(
            prepare_direct_launch(&escaping),
            Err(LaunchConfigError::OutsideInstallRoot(_))
        ));
        let ambiguous = target(
            &root,
            json!([
                { "executable": "one.exe", "role": "client" },
                { "executable": "two.exe", "role": "client" }
            ]),
        );
        assert!(matches!(
            prepare_direct_launch(&ambiguous),
            Err(LaunchConfigError::AmbiguousClient(_))
        ));
        fs::remove_file(outside).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn environment_overlay_preserves_program_working_directory_and_argv() {
        let executable = std::env::current_exe().unwrap();
        let working_directory = executable.parent().unwrap().to_path_buf();
        let arguments = vec![
            OsString::from(""),
            OsString::from("two words"),
            OsString::from("snowman-☃"),
            OsString::from("\"quoted\" & | < >"),
        ];
        let before = DirectExecutableLaunch::new(
            executable.clone(),
            working_directory.clone(),
            arguments.clone(),
        )
        .unwrap();
        let launch = ManagedLaunch::Direct(before)
            .with_environment([("HTTP_PROXY", "http://127.0.0.1:43123")])
            .unwrap();
        let ManagedLaunch::Direct(after) = launch else {
            panic!("expected direct launch");
        };
        assert_eq!(after.executable(), executable.canonicalize().unwrap());
        assert_eq!(
            after.working_directory(),
            working_directory.canonicalize().unwrap()
        );
        assert_eq!(after.arguments(), arguments);
        assert_eq!(
            after.environment().get(OsStr::new("HTTP_PROXY")),
            Some(&OsString::from("http://127.0.0.1:43123"))
        );
    }

    #[test]
    fn direct_child_inherits_scoped_environment() {
        let executable = std::env::current_exe().unwrap();
        let working_directory = executable.parent().unwrap().to_path_buf();
        let output = unique_dir("child-env.txt");
        let launch = ManagedLaunch::Direct(
            DirectExecutableLaunch::new(
                executable,
                working_directory,
                vec![
                    "--exact".into(),
                    "managed_launch::tests::child_environment_probe".into(),
                    "--ignored".into(),
                ],
            )
            .unwrap(),
        )
        .with_environment([
            (
                "FRAGCAP_LAUNCH_TEST_OUTPUT",
                output.to_string_lossy().as_ref(),
            ),
            ("HTTP_PROXY", "http://127.0.0.1:43124"),
        ])
        .unwrap();
        launch.execute().unwrap();
        let started = Instant::now();
        while !output.is_file() && started.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(
            fs::read_to_string(&output).unwrap(),
            "http://127.0.0.1:43124"
        );
        fs::remove_file(output).unwrap();
    }

    #[test]
    #[ignore = "launched by direct_child_inherits_scoped_environment"]
    fn child_environment_probe() {
        let output = std::env::var_os("FRAGCAP_LAUNCH_TEST_OUTPUT").unwrap();
        let proxy = std::env::var("HTTP_PROXY").unwrap();
        fs::write(output, proxy).unwrap();
    }
}
