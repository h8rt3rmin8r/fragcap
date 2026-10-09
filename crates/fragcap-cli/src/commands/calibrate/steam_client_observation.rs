// SPDX-License-Identifier: Apache-2.0

//! Read-only Steam client identification from an owned launch and socket table.

#[cfg(any(test, all(feature = "etw", feature = "socket-table", windows)))]
use std::collections::VecDeque;
use std::collections::{BTreeMap, HashSet};
#[cfg(any(test, all(feature = "etw", feature = "socket-table", windows)))]
use std::path::Path;
use std::time::Duration;

use fragcap::targets::TargetEntry;

use crate::emit::Emitter;
use crate::exit::CliError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ClientObservation {
    Unique { executable: String, sockets: u64 },
    None,
    Ambiguous(Vec<(String, u64)>),
    Incomplete(String),
}

fn decide(counts: BTreeMap<String, u64>, complete: bool) -> ClientObservation {
    if !complete {
        return ClientObservation::Incomplete(
            "process or socket observation was incomplete".to_string(),
        );
    }
    let mut candidates: Vec<_> = counts.into_iter().filter(|(_, n)| *n > 0).collect();
    match candidates.len() {
        0 => ClientObservation::None,
        1 => {
            let (executable, sockets) = candidates.remove(0);
            ClientObservation::Unique {
                executable,
                sockets,
            }
        }
        _ => ClientObservation::Ambiguous(candidates),
    }
}

fn observation_profile(app_id: u32) -> Result<fragcap::Profile, CliError> {
    let profile_json = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "heuristic-unverified",
        "game": {"id": "client-observation", "name": "Client observation", "platform": "steam", "app_id": app_id.to_string()},
        "stage": [
            {"role": "candidate", "lifecycle": "session", "match": {"exe": "*", "descends_from": "platform"}},
            {"role": "platform", "lifecycle": "service", "match": {"exe": "steam.exe", "path_contains": "steam.exe"}}
        ]
    });
    fragcap::Profile::parse(&profile_json.to_string()).map_err(|diagnostics| {
        CliError::failure(format!(
            "client observation profile invalid: {diagnostics:?}"
        ))
    })
}

/// A basename-only process event can propose a client only when that executable
/// exists in the selected title's install tree. Full paths must match an exact
/// inventoried file. This prevents ordinary Steam infrastructure descendants
/// from becoming client suggestions on their socket activity alone.
struct GameExecutables {
    names: HashSet<String>,
    paths: HashSet<String>,
}

impl GameExecutables {
    #[cfg(any(test, all(feature = "etw", feature = "socket-table", windows)))]
    fn inventory(root: &Path) -> Result<Self, CliError> {
        let root = std::fs::canonicalize(root).map_err(|error| {
            CliError::usage(format!(
                "client observation cannot read the selected install root: {error}; use --client-executable EXE for an explicit declaration"
            ))
        })?;
        let mut dirs = VecDeque::from([(root, 0usize)]);
        let mut names = HashSet::new();
        let mut paths = HashSet::new();
        let mut visited = 0usize;
        while let Some((dir, depth)) = dirs.pop_front() {
            let entries = std::fs::read_dir(&dir).map_err(|error| {
                CliError::usage(format!(
                    "client observation cannot enumerate the selected install root: {error}; use --client-executable EXE for an explicit declaration"
                ))
            })?;
            for entry in entries {
                visited += 1;
                if visited > 65_536 {
                    return Err(CliError::usage("client observation install inventory exceeded 65536 entries; use --client-executable EXE for an explicit declaration"));
                }
                let entry = entry.map_err(|error| {
                    CliError::usage(format!("client observation install inventory is incomplete: {error}; use --client-executable EXE for an explicit declaration"))
                })?;
                let kind = entry.file_type().map_err(|error| {
                    CliError::usage(format!("client observation install inventory is incomplete: {error}; use --client-executable EXE for an explicit declaration"))
                })?;
                if kind.is_dir() {
                    if depth >= 16 {
                        return Err(CliError::usage("client observation install inventory exceeded 16 directory levels; use --client-executable EXE for an explicit declaration"));
                    }
                    dirs.push_back((entry.path(), depth + 1));
                } else if kind.is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case("exe"))
                {
                    let path = std::fs::canonicalize(entry.path()).map_err(|error| {
                        CliError::usage(format!("client observation install inventory is incomplete: {error}; use --client-executable EXE for an explicit declaration"))
                    })?;
                    names.insert(entry.file_name().to_string_lossy().to_lowercase());
                    paths.insert(path.to_string_lossy().to_lowercase());
                }
            }
        }
        Ok(Self { names, paths })
    }

    fn permits(&self, image: &str) -> bool {
        if image.contains(['\\', '/']) {
            std::fs::canonicalize(image)
                .ok()
                .is_some_and(|path| self.paths.contains(&path.to_string_lossy().to_lowercase()))
        } else {
            self.names.contains(&image.to_lowercase())
        }
    }
}

fn record_owned_socket_rows(
    session: &fragcap::CaptureSession,
    rows: &[fragcap::SocketTableEntry],
    observed_at: fragcap::Timestamp,
    game_executables: Option<&GameExecutables>,
    seen: &mut HashSet<(String, fragcap::SocketTableEntry)>,
    counts: &mut BTreeMap<String, u64>,
) {
    for row in rows {
        if row.proto == fragcap::Proto::Tcp && row.remote.is_none() {
            continue;
        }
        let Some(id) = session
            .tree()
            .resolve(fragcap::ProcessId(row.pid), observed_at)
        else {
            continue;
        };
        let Some(node) = session.tree().node(id) else {
            continue;
        };
        if !node.is_live()
            || node
                .stage()
                .is_none_or(|stage| stage.as_str() != "candidate")
        {
            continue;
        }
        if row
            .created
            .is_some_and(|created| node.started().is_some_and(|start| created < start))
        {
            continue;
        }
        let image = node.image_name().to_string();
        if image.eq_ignore_ascii_case("steam.exe") {
            continue;
        }
        if game_executables.is_some_and(|inventory| !inventory.permits(node.image())) {
            continue;
        }
        if seen.insert((image.clone(), *row)) {
            *counts.entry(image).or_default() += 1;
        }
    }
}

/// The reserved synthetic target exercises the same process/socket join through
/// the CLI without starting a real platform or claiming machine evidence.
pub(super) fn observe_controlled(target: &TargetEntry) -> Result<ClientObservation, CliError> {
    use fragcap::{CaptureSession, ProcessEvent, SessionConfig, SocketTableEntry, Timestamp};

    crate::commands::deep_capture::require_controlled_target(target)?;
    let app_id = super::steam_app_id(target)
        .ok_or_else(|| CliError::usage("controlled client observation requires a Steam target"))?;
    let mut session = CaptureSession::new(observation_profile(app_id)?, SessionConfig::default());
    let at = Timestamp::from_nanos;
    session.require_owned_platform_root("C:\\SyntheticSteam\\steam.exe");
    session.observe_until_explicit_stop();
    session.attach(at(0));
    session.arm_owned_platform_root(10, 1, at(1), at(3));
    for (pid, parent, image, when) in [
        (10, 1, "steam.exe", 2),
        (11, 10, "sample_launcher.exe", 4),
        (12, 11, "client.exe", 5),
    ] {
        session.on_process_event(ProcessEvent::started(pid, parent, image, "", at(when)));
    }
    let local = "127.0.0.1:30100".parse().expect("synthetic local address");
    let peer = "192.0.2.5:443".parse().expect("synthetic peer address");
    let rows = [
        SocketTableEntry::tcp_listening(local, 11),
        SocketTableEntry::tcp(local, peer, 12).created_at(at(5)),
    ];
    let mut seen = HashSet::new();
    let mut counts = BTreeMap::new();
    record_owned_socket_rows(&session, &rows, at(7), None, &mut seen, &mut counts);
    Ok(decide(counts, true))
}

#[cfg(all(feature = "etw", feature = "socket-table", windows))]
pub(super) fn observe(
    target: &TargetEntry,
    limit: Duration,
    emitter: &mut Emitter,
) -> Result<ClientObservation, CliError> {
    use std::sync::atomic::Ordering;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use fragcap::attr::SocketTableSource;
    use fragcap::managed_launch::{PlatformLaunchAdapter, SteamPlatformAdapter};
    use fragcap::{
        CaptureSession, EtwWatcher, IpHelperTable, ProcessWatcher, SessionConfig, Timestamp,
    };

    fn now() -> Result<Timestamp, CliError> {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| CliError::failure(format!("clock unavailable: {error}")))?;
        Ok(Timestamp::from_nanos(
            i64::try_from(elapsed.as_nanos()).unwrap_or(i64::MAX),
        ))
    }

    let app_id = super::steam_app_id(target)
        .ok_or_else(|| CliError::usage("client observation requires one exact Steam target"))?;
    match super::early_steam_state(&super::process_snapshot(false), true) {
        super::EarlySteamState::Warm => return Err(CliError::usage("Steam is already running. Close Steam normally, then retry client observation from a cold start")),
        super::EarlySteamState::Unavailable(reason) => return Err(CliError::failure(format!("Steam process observation unavailable: {reason}"))),
        super::EarlySteamState::Cold => {}
    }
    let profile = observation_profile(app_id)?;
    let inventory = GameExecutables::inventory(Path::new(
        target.install_root.as_deref().ok_or_else(|| {
            CliError::usage("client observation requires the selected title's exact install root")
        })?,
    ))?;
    let platform = SteamPlatformAdapter::discover()
        .and_then(|adapter| adapter.prepare(&profile))
        .map_err(|error| {
            CliError::failure(format!(
                "could not prepare a cold Steam observation: {error}"
            ))
        })?;

    let watcher = EtwWatcher::start("fragcap-client-observation").map_err(|error| {
        CliError::failure(format!("client process observation unavailable: {error}"))
    })?;
    let snapshot = ProcessWatcher::snapshot(&watcher);
    if snapshot.iter().any(|record| {
        record
            .image
            .rsplit(['\\', '/'])
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case("steam.exe"))
    }) {
        return Err(CliError::usage("Steam is already running. Close Steam normally, then retry client observation from a cold start"));
    }
    let rx = watcher.subscribe();
    let mut session = CaptureSession::new(profile, SessionConfig::default());
    session.require_owned_platform_root(&platform.root().executable().to_string_lossy());
    session.observe_until_explicit_stop();
    session.attach(now()?);
    session.apply_snapshot(&snapshot, watcher.snapshot_taken_at().unwrap_or(now()?));

    let earliest = now()?;
    let receipt = fragcap::managed_launch::ManagedLaunch::Platform(platform.clone())
        .execute()
        .map_err(|error| {
            CliError::failure(format!("could not start Steam client observation: {error}"))
        })?;
    let latest = now()?;
    let pid = receipt
        .process_id()
        .ok_or_else(|| CliError::failure("Steam launch did not return a process identifier"))?;
    session.arm_owned_platform_root(pid, std::process::id(), earliest, latest);
    emitter.progress("Observing the selected Steam launch and its client connections. Continue normal in-game startup; Ctrl+C cancels setup without changing the target.");

    let mut table = IpHelperTable::new();
    let mut counts = BTreeMap::<String, u64>::new();
    let mut observed_rows = HashSet::new();
    let mut dispatched = false;
    let started = Instant::now();
    let mut complete = true;
    while started.elapsed() < limit {
        if crate::orchestrator::INTERRUPT.load(Ordering::Relaxed) {
            return Ok(ClientObservation::Incomplete(
                "operator interrupted observation".to_string(),
            ));
        }
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => {
                session.on_process_event(event);
                if !dispatched
                    && session.role_bindings().iter().any(|(bound_pid, role, _)| {
                        *bound_pid == pid && role.as_deref() == Some("platform")
                    })
                {
                    platform.dispatch_title().map_err(|error| {
                        CliError::failure(format!(
                            "Steam title dispatch failed during client observation: {error}"
                        ))
                    })?;
                    dispatched = true;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                complete = false;
                break;
            }
        }
        if !dispatched {
            continue;
        }
        let snapshot = match table.read() {
            Ok(value) => value,
            Err(_) => {
                complete = false;
                break;
            }
        };
        let observed_at = now()?;
        record_owned_socket_rows(
            &session,
            snapshot.entries(),
            observed_at,
            Some(&inventory),
            &mut observed_rows,
            &mut counts,
        );
    }
    if !dispatched {
        return Ok(ClientObservation::Incomplete(
            "the owned Steam process did not bind or title dispatch did not occur".to_string(),
        ));
    }
    let report = watcher.report();
    complete &= report.events_lost == 0 && report.buffers_lost == 0;
    Ok(decide(counts, complete))
}

#[cfg(not(all(feature = "etw", feature = "socket-table", windows)))]
pub(super) fn observe(
    _target: &TargetEntry,
    _limit: Duration,
    _emitter: &mut Emitter,
) -> Result<ClientObservation, CliError> {
    Ok(ClientObservation::Incomplete(
        "this build has no Windows ETW and socket-table observation backends".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap::{CaptureSession, ProcessEvent, SessionConfig, SocketTableEntry, Timestamp};

    fn at(n: i64) -> Timestamp {
        Timestamp::from_nanos(n)
    }

    #[test]
    fn owned_chain_socket_join_ignores_launcher_listeners_and_foreign_processes() {
        let mut session =
            CaptureSession::new(observation_profile(42).unwrap(), SessionConfig::default());
        session.require_owned_platform_root("C:\\Steam\\steam.exe");
        session.observe_until_explicit_stop();
        session.attach(at(0));
        session.arm_owned_platform_root(10, 1, at(1), at(3));
        for (pid, parent, image, when) in [
            (10, 1, "steam.exe", 2),
            (11, 10, "sample_launcher.exe", 4),
            (12, 11, "sample_client.exe", 5),
            (13, 1, "foreign.exe", 6),
        ] {
            session.on_process_event(ProcessEvent::started(pid, parent, image, "", at(when)));
        }
        let local = "127.0.0.1:30100".parse().unwrap();
        let peer = "192.0.2.5:443".parse().unwrap();
        let rows = [
            SocketTableEntry::tcp_listening(local, 11),
            SocketTableEntry::tcp(local, peer, 12).created_at(at(5)),
            SocketTableEntry::tcp(local, peer, 13),
            SocketTableEntry::tcp(local, peer, 12).created_at(at(3)),
        ];
        let mut seen = HashSet::new();
        let mut counts = BTreeMap::new();
        record_owned_socket_rows(&session, &rows, at(7), None, &mut seen, &mut counts);
        record_owned_socket_rows(&session, &rows, at(8), None, &mut seen, &mut counts);
        assert_eq!(
            decide(counts, true),
            ClientObservation::Unique {
                executable: "sample_client.exe".into(),
                sockets: 1,
            }
        );
    }

    #[test]
    fn only_one_complete_observed_socket_owner_is_selectable() {
        assert!(
            observation_profile(42).is_ok(),
            "{:?}",
            observation_profile(42).err()
        );
        assert_eq!(decide(BTreeMap::new(), true), ClientObservation::None);
        let one = BTreeMap::from([("client.exe".to_string(), 3)]);
        assert_eq!(
            decide(one.clone(), true),
            ClientObservation::Unique {
                executable: "client.exe".into(),
                sockets: 3
            }
        );
        assert!(matches!(
            decide(one, false),
            ClientObservation::Incomplete(_)
        ));
        let two = BTreeMap::from([
            ("launcher.exe".to_string(), 1),
            ("client.exe".to_string(), 3),
        ]);
        assert!(matches!(decide(two, true), ClientObservation::Ambiguous(_)));
    }

    #[test]
    fn steam_helper_socket_does_not_become_a_game_client_candidate() {
        let install = tempfile::tempdir().unwrap();
        std::fs::write(install.path().join("sample_client.exe"), []).unwrap();
        let inventory = GameExecutables::inventory(install.path()).unwrap();
        assert!(inventory.permits("sample_client.exe"));
        assert!(!inventory.permits("steamwebhelper.exe"));
        let outside = tempfile::tempdir().unwrap();
        let foreign_same_name = outside.path().join("sample_client.exe");
        std::fs::write(&foreign_same_name, []).unwrap();
        assert!(!inventory.permits(&foreign_same_name.to_string_lossy()));

        let mut session =
            CaptureSession::new(observation_profile(42).unwrap(), SessionConfig::default());
        session.require_owned_platform_root("C:\\Steam\\steam.exe");
        session.observe_until_explicit_stop();
        session.attach(at(0));
        session.arm_owned_platform_root(10, 1, at(1), at(3));
        for (pid, parent, image, when) in [
            (10, 1, "steam.exe", 2),
            (11, 10, "steamwebhelper.exe", 4),
            (12, 10, "sample_client.exe", 5),
        ] {
            session.on_process_event(ProcessEvent::started(pid, parent, image, "", at(when)));
        }
        let local = "127.0.0.1:30100".parse().unwrap();
        let peer = "192.0.2.5:443".parse().unwrap();
        let rows = [
            SocketTableEntry::tcp(local, peer, 11).created_at(at(4)),
            SocketTableEntry::tcp(local, peer, 12).created_at(at(5)),
        ];
        let mut seen = HashSet::new();
        let mut counts = BTreeMap::new();
        record_owned_socket_rows(
            &session,
            &rows,
            at(7),
            Some(&inventory),
            &mut seen,
            &mut counts,
        );
        assert_eq!(counts, BTreeMap::from([("sample_client.exe".into(), 1)]));
    }
}
