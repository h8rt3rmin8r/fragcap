// SPDX-License-Identifier: Apache-2.0

//! Turning the parsed arguments and a resolved profile into the pieces a
//! capture runs on: the effective configuration, the sources, the attributor
//! and its role-stamping decorator, the process events, and the sinks.
//!
//! Two responsibilities live here. The overlay of specification FR-007, command
//! line over the profile's `[capture]` defaults, with the command line winning
//! and an option absent from both staying absent, is [`effective_config`]. The
//! assembly of the seams is the rest: the offline substrate (a replay source, a
//! scripted attributor, and a scripted process timeline) that drives every
//! tier-1 test, and the feature-gated live, socket-table, and ETW paths that
//! run only on a developer machine. The not-yet-supported modes, transport
//! sinks, and managed launch are refused here as configuration errors naming
//! the slice that delivers them (FR-010, FR-011).

use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap::core::SourceBinding;
use fragcap::profile::CaptureMode;
use fragcap::{
    AttributionScript, BindingPublisher, CommandLine, ConsumerReport, FlowAttributor, Format,
    InterfaceAddrs, InterfaceId, InterfaceSpec, LinkType, PacketSource, PayloadMode, ProcessEvent,
    ProcessRecord, ProcessScript, ProcessWatcher, Profile, ReplaySource, RingSink, RingWindow,
    RoleStampingAttributor, RotatingFileSink, RotationPolicy, ScriptedAttributor, ScriptedWatcher,
    SinkFactory, StreamSink, Timestamp,
};
use fragcap::{CaptureScope, SessionConfig, Sink};

use crate::args::{Direction, SinkFormat, SinkSpec, SinkTransport};
use crate::cli::{CaptureArgs, ModeArg, OfflineArgs, ScopeArg};
use crate::exit::CliError;

/// The default snapshot length declared for a capture interface.
const SNAP_LEN: u32 = 65_535;

/// The interface name used when none is given, so a single-interface offline
/// capture has a stable name for its declaration.
const DEFAULT_INTERFACE: &str = "capture";

/// The capture options actually used, formed by overlaying the command line
/// onto a profile's `CaptureDefaults`.
#[derive(Clone, Debug)]
pub struct EffectiveConfig {
    /// The effective capture mode, command line over the profile default.
    pub mode: CaptureMode,
    /// The ring window, when ring mode was configured. Carries the sink-side
    /// [`RingWindow`], mapped from the command-line grammar.
    pub ring: Option<RingWindow>,
    /// The output capture file, when one was given.
    pub out: Option<PathBuf>,
    /// The parsed sink specifications.
    pub sinks: Vec<SinkSpec>,
    /// The capture duration bound, from arm.
    pub duration: Option<Duration>,
    /// The acquisition timeout.
    pub acquisition_timeout: Option<Duration>,
    /// The packet bound.
    pub max_packets: Option<u64>,
    /// The byte bound.
    pub max_bytes: Option<u64>,
    /// The roles to capture, when scoped.
    pub roles: Option<Vec<String>>,
    /// What the capture writes out (slice S064).
    pub scope: CaptureScope,
    /// The direction to scope to.
    pub direction: Direction,
    /// The declared interface names.
    pub interfaces: Vec<String>,
    /// Whether the loopback adapter is included.
    pub loopback: bool,
    /// Whether packet payloads are captured.
    pub payload: bool,
    /// The managed launch to issue once watching, when `--launch` was given and
    /// the profile declares a Steam platform and app_id (specification 16.4).
    #[allow(dead_code)]
    pub launch: Option<fragcap::managed_launch::ManagedLaunch>,
}

impl EffectiveConfig {
    /// The session bounds handed to [`fragcap::CaptureSession`].
    pub fn session_config(&self) -> SessionConfig {
        SessionConfig {
            acquisition_timeout: self.acquisition_timeout,
            duration: self.duration,
            packet_bound: self.max_packets,
            byte_bound: self.max_bytes,
            scope: self.scope,
            // The role set reaches the write gate here, which is what makes the
            // run's `roles ... (enforced)` line true of what the file contains
            // rather than only of which stages trigger acquisition (issue #184).
            allowed_roles: self.roles.clone(),
        }
    }

    /// The interface names to declare, defaulting to one stable name.
    pub fn interface_names(&self) -> Vec<String> {
        if self.interfaces.is_empty() {
            vec![DEFAULT_INTERFACE.to_string()]
        } else {
            self.interfaces.clone()
        }
    }
}

/// Build the effective configuration from `capture` arguments and a resolved or
/// synthesized profile, refusing the options this slice does not yet support.
#[cfg_attr(not(test), allow(dead_code))]
pub fn effective_config(
    args: &CaptureArgs,
    profile: &Profile,
) -> Result<EffectiveConfig, CliError> {
    effective_config_with_target(args, profile, None)
}

/// Build effective Capture configuration while retaining the selected stored
/// target as the sole source for a direct managed launch.
pub fn effective_config_with_target(
    args: &CaptureArgs,
    profile: &Profile,
    stored_target: Option<&fragcap::targets::TargetEntry>,
) -> Result<EffectiveConfig, CliError> {
    reject_unsupported(args, profile)?;

    let defaults = profile.capture();
    let payload = if args.no_payload {
        false
    } else {
        defaults.payload().unwrap_or(true)
    };
    let roles = args
        .roles
        .clone()
        .or_else(|| defaults.roles().map(|r| r.to_vec()));
    let scope = match args.scope {
        ScopeArg::Target => CaptureScope::Target,
        ScopeArg::All => CaptureScope::All,
    };
    let loopback = args.loopback || defaults.loopback().unwrap_or(false);
    let duration = args.duration.or_else(|| defaults.duration());
    let mode = resolve_mode(args, profile);
    let ring = args.ring.map(map_ring_window);
    let launch = build_launch(args, profile, stored_target)?;

    Ok(EffectiveConfig {
        mode,
        ring,
        out: args.out.clone(),
        sinks: args.sink.clone(),
        duration,
        acquisition_timeout: args.wait,
        max_packets: args.max_packets,
        max_bytes: args.max_bytes,
        roles,
        scope,
        direction: args.direction,
        interfaces: args.interface.clone(),
        loopback,
        payload,
        launch,
    })
}

/// Build the managed-launch request from `--launch` and the profile.
///
/// Absent `--launch`, there is none. With it, the profile must declare a Steam
/// platform and an app_id, or the launch is refused as a named configuration
/// error before capture starts (FR-011); on a non-Windows build it is refused as
/// unsupported, since there is no Steam protocol handler to invoke.
fn build_launch(
    args: &CaptureArgs,
    profile: &Profile,
    stored_target: Option<&fragcap::targets::TargetEntry>,
) -> Result<Option<fragcap::managed_launch::ManagedLaunch>, CliError> {
    if !args.launch {
        return Ok(None);
    }
    #[cfg(not(windows))]
    {
        let _ = profile;
        Err(CliError::usage(
            "managed launch (--launch) is only supported on Windows",
        ))
    }
    #[cfg(windows)]
    {
        if profile.game().platform() == Some("steam") {
            return fragcap::steam::launch_request(profile)
                .map(fragcap::managed_launch::ManagedLaunch::Steam)
                .map(Some)
                .map_err(|e| CliError::usage(e.to_string()));
        }
        let target = stored_target
            .ok_or_else(|| CliError::usage("managed direct launch requires a stored target"))?;
        fragcap::managed_launch::prepare_direct_launch(target)
            .map(Some)
            .map_err(|error| CliError::usage(error.to_string()))
    }
}

/// Build the effective configuration for an extcap `--capture`, from the
/// analyzer-supplied options and the resolved profile.
///
/// It overlays the extcap options (roles, direction, loopback) onto the profile's
/// `[capture]` defaults exactly as `capture` does (specification FR-007, and section
/// 14.5's "the same options `capture` honors"), and carries the analyzer FIFO as its
/// single sink. There is no ring,
/// no managed launch, no volume bound, and no `--out` file: an extcap capture is
/// a live stream to the analyzer, stopped by the analyzer closing the FIFO or by
/// a session stop condition.
pub fn effective_config_for_extcap(
    args: &crate::cli::ExtcapArgs,
    profile: &Profile,
) -> EffectiveConfig {
    let defaults = profile.capture();
    let payload = defaults.payload().unwrap_or(true);
    let roles = args
        .roles
        .clone()
        .or_else(|| defaults.roles().map(|r| r.to_vec()));
    let loopback = args.loopback || defaults.loopback().unwrap_or(false);
    let fifo = SinkSpec::for_fifo(
        args.fifo
            .clone()
            .expect("an extcap capture requires --fifo, validated by the command"),
    );
    EffectiveConfig {
        mode: CaptureMode::File,
        ring: None,
        out: None,
        sinks: vec![fifo],
        duration: defaults.duration(),
        acquisition_timeout: None,
        max_packets: None,
        max_bytes: None,
        roles,
        // The extcap path has no `--scope` flag: the analyzer dialog does not
        // offer one, and Wireshark's own display filters are the natural place
        // to widen a view there. It takes the same default as the command line,
        // so what reaches the analyzer is what reaches a file.
        scope: CaptureScope::Target,
        direction: args.direction.unwrap_or(Direction::Both),
        interfaces: Vec::new(),
        loopback,
        payload,
        launch: None,
    }
}

/// The effective capture mode: the command line over the profile's `[capture]`
/// default (specification FR-007). An explicit `--mode` wins; absent one, the
/// profile's declared mode; absent both, `file`.
fn resolve_mode(args: &CaptureArgs, profile: &Profile) -> CaptureMode {
    match args.mode {
        Some(ModeArg::File) => CaptureMode::File,
        Some(ModeArg::Stream) => CaptureMode::Stream,
        Some(ModeArg::Ring) => CaptureMode::Ring,
        None => profile.capture().mode().unwrap_or(CaptureMode::File),
    }
}

/// Map the command-line ring window onto the sink-side [`RingWindow`]. The two
/// enums are deliberately separate: the sink is a library crate that must not
/// depend on the CLI (constitution P-2 dependency direction).
fn map_ring_window(window: crate::args::RingWindow) -> RingWindow {
    match window {
        crate::args::RingWindow::Duration(d) => RingWindow::Duration(d),
        crate::args::RingWindow::Size(bytes) => RingWindow::Size(bytes),
    }
}

/// Validate the mode-specific configuration, refusing every misconfiguration and
/// the options deferred to later slices before capture starts.
///
/// Ring mode (specification section 7.2, this slice) requires both a `--out` dump
/// target and a `--ring` window; a volume stop bound (`--max-bytes`,
/// `--max-packets`) does not apply to a rolling window; and a `--ring` window
/// given outside ring mode would be silently ignored. Each is a configuration
/// error naming the cause.
fn reject_unsupported(args: &CaptureArgs, profile: &Profile) -> Result<(), CliError> {
    let mode = resolve_mode(args, profile);
    if mode == CaptureMode::Ring {
        if args.out.is_none() {
            return Err(CliError::usage(
                "ring mode needs an output file to dump the retained window to; pass --out <PATH>",
            ));
        }
        if args.ring.is_none() {
            return Err(CliError::usage(
                "ring mode needs a ring window; pass --ring <DURATION|SIZE>",
            ));
        }
        if args.max_packets.is_some() || args.max_bytes.is_some() {
            return Err(CliError::usage(
                "--max-packets and --max-bytes do not apply in ring mode; a rolling window does \
                 not stop on accumulated volume",
            ));
        }
    } else if args.ring.is_some() {
        return Err(CliError::usage(
            "--ring applies only in ring mode; add --mode ring",
        ));
    }
    Ok(())
}

/// Where the capture's process events come from.
///
/// Offline, a pre-collected timeline the scripted watcher produced and the
/// orchestrator folds in two phases. Live, a channel the ETW watcher streams
/// onto as processes start and exit. The two reach the same session driver; the
/// difference is that the live stream has no end the driver can read ahead to,
/// so the two are folded differently (the orchestrator's two paths).
pub enum EventStream {
    /// The offline timeline, in timestamp order. Unchanged behavior from before
    /// the live path existed.
    Prerecorded(Vec<ProcessEvent>),
    /// The live process event stream from the ETW watcher. The watcher itself is
    /// kept alive on [`CaptureComponents::watcher`] so this receiver stays
    /// connected for the run.
    #[cfg(all(feature = "etw", windows))]
    Live(std::sync::mpsc::Receiver<ProcessEvent>),
}

/// Everything a capture runs on, assembled from the offline substrate or, on a
/// feature-gated build, from the live capture backends.
pub struct CaptureComponents {
    /// The bound packet sources.
    pub sources: Vec<SourceBinding>,
    /// One `(name, link type)` per selected interface, in selection order, for
    /// declaring interfaces on the sinks. The pipeline assigns each source its
    /// `InterfaceId` from selection position, so this list is in the same order
    /// and of the same length as [`sources`](Self::sources); a sink declares one
    /// interface per source, each with its own link type, so a packet from any
    /// selected interface references a declared one rather than being refused as
    /// undeclared.
    pub interfaces: Vec<(String, LinkType)>,
    /// The role-stamping decorator handed to the pipeline. Slice 015: the
    /// filter-narrowed event reads its profiled-filtered active endpoints, so the
    /// unfiltered inner attributor is no longer kept separately.
    pub stamper: Option<RoleStampingAttributor>,
    /// The handle that publishes bindings to the stamper.
    pub publisher: BindingPublisher,
    /// The process events, offline or live.
    pub events: EventStream,
    /// The startup snapshot of processes already running at arm, from the process
    /// watcher's query-only toolhelp enumeration (live) or the offline script's
    /// declared snapshot. The orchestrator folds it into the session at arm so an
    /// already-running target is attached without a later start event (section
    /// 15.7). Empty when none.
    pub startup_snapshot: Vec<ProcessRecord>,
    /// The instant the startup snapshot reflects, when known. `None` offline,
    /// where the orchestrator applies it at the arm instant.
    pub snapshot_at: Option<Timestamp>,
    /// Optional capture-wide flow registry supplied by Deep Capture so proxy
    /// observations can resolve packet-side flow ids after the run.
    pub flow_registry: Option<Arc<fragcap::FlowRegistry>>,
    /// The live process watcher, kept alive for the duration of the run so the
    /// ETW session it owns is not torn down while its receiver is still being
    /// drained. `None` on the offline path, which owns no watcher.
    #[cfg(all(feature = "etw", windows))]
    pub watcher: Option<fragcap::EtwWatcher>,
}

// Slice 015 removed `RefreshDriver`. `FlowAttributor::refresh` now takes `&self`,
// so the socket-table refresh is driven by the pipeline's own section 8.6
// control thread through the shared `Arc<dyn FlowAttributor>` the capture threads
// resolve against, rather than by a separate control thread here that owned a
// mutable attributor. The read/write split (`PublishedResolver`) is no longer
// needed on this path either.

/// Assemble the capture components.
///
/// An offline replay source, when one is given, drives the tier-1 substrate: a
/// replayed capture, a scripted attributor, and a scripted process timeline.
/// Otherwise, on a `live` build, the real interface, socket-table, and ETW
/// backends are assembled instead. On a build with neither, there is no packet
/// source, which is reported as a usable-interface-absent failure.
pub fn components(
    offline: &OfflineArgs,
    config: &EffectiveConfig,
) -> Result<CaptureComponents, CliError> {
    match &offline.replay_source {
        Some(replay) => offline_components(replay, offline, config),
        None => live_or_absent(config),
    }
}

/// Assemble the offline substrate. Unchanged from before the live path existed,
/// beyond wrapping the events in [`EventStream::Prerecorded`].
fn offline_components(
    replay: &PathBuf,
    offline: &OfflineArgs,
    config: &EffectiveConfig,
) -> Result<CaptureComponents, CliError> {
    let source = ReplaySource::open(replay)?;
    let link = source.link_type();
    let addrs = InterfaceAddrs::new(offline.local_addr.iter().copied());
    let sources = vec![SourceBinding::new(
        InterfaceId::default(),
        Box::new(source),
        addrs,
    )];
    // The single replay interface, declared under every configured name against
    // the one replay link type. With no `-i` this is the single stable
    // "capture" name, so the offline sink bytes are unchanged.
    let interfaces: Vec<(String, LinkType)> = config
        .interface_names()
        .into_iter()
        .map(|name| (name, link))
        .collect();

    let script = match &offline.attr_script {
        Some(path) => AttributionScript::load(path)
            .map_err(|e| CliError::failure(format!("attribution script: {e}")))?,
        None => AttributionScript::parse("").expect("an empty script is valid"),
    };
    let inner: Arc<dyn FlowAttributor> = Arc::new(ScriptedAttributor::new(script));
    let stamper = RoleStampingAttributor::new(Arc::clone(&inner));
    let publisher = stamper.publisher();

    let (events, startup_snapshot) = process_events(&offline.process_script)?;

    Ok(CaptureComponents {
        sources,
        interfaces,
        stamper: Some(stamper),
        publisher,
        events: EventStream::Prerecorded(events),
        startup_snapshot,
        // Offline the snapshot has no separate instant; the orchestrator applies
        // it at the arm instant, so a snapshot process is present from arm.
        snapshot_at: None,
        flow_registry: None,
        #[cfg(all(feature = "etw", windows))]
        watcher: None,
    })
}

/// The elevation precondition for live capture, as a pure decision so the
/// refusal is testable off Windows. `Some(err)` means refuse.
///
/// Live capture opens the npcap driver, which requires an elevated session.
/// This is the detect-instruct-refuse contract: it never auto-relaunches an
/// elevated process (that would spawn a separate console and break `--json`
/// streaming), and the underlying elevation read touches only the current
/// process's own token (constitution P-1). The refusal is an expected
/// environment-precondition failure (exit 1), matching the no-backend refusal.
// Called only from the `live` + windows capture path; on every other build it is
// exercised solely by the platform-neutral test below, so allow it to be unused
// in non-test code there rather than warn.
#[cfg_attr(not(all(feature = "live", windows)), allow(dead_code))]
fn elevation_refusal(elevated: bool) -> Option<CliError> {
    if elevated {
        None
    } else {
        Some(CliError::failure(
            "live capture requires Administrator. Re-launch from an elevated terminal \
             (right-click > Run as administrator), or `Start-Process powershell -Verb RunAs`.",
        ))
    }
}

/// The live path, or the no-backend failure on a build without it.
fn live_or_absent(config: &EffectiveConfig) -> Result<CaptureComponents, CliError> {
    #[cfg(all(feature = "live", windows))]
    {
        // Refuse before touching the driver when the session is not elevated.
        if let Some(refusal) = elevation_refusal(crate::doctor::probe::is_elevated()) {
            return Err(refusal);
        }
        live_components(config)
    }
    #[cfg(not(all(feature = "live", windows)))]
    {
        let _ = config;
        Err(CliError::failure(
            "no capture source: this build has no live capture backend, and no offline replay \
             source was given",
        ))
    }
}

/// Assemble the live capture backends: real interface enumeration and
/// selection, a [`LiveSource`](fragcap::LiveSource) per selected interface, the
/// socket-table attributor, and the ETW process event stream.
///
/// Requires the `etw` feature: without a live process event source no target
/// can ever be acquired, so a live build lacking it fails here naming the
/// missing feature rather than starting a capture that could never bind a stage.
#[cfg(all(feature = "live", windows))]
fn live_components(config: &EffectiveConfig) -> Result<CaptureComponents, CliError> {
    #[cfg(not(feature = "etw"))]
    {
        let _ = config;
        Err(CliError::failure(
            "live capture needs the `etw` feature so a live process event source exists; \
             without it no target can ever be acquired",
        ))
    }

    #[cfg(feature = "etw")]
    {
        use fragcap::capture::{enumerate, LiveOptions, LiveSource};
        use fragcap::{select, PipelineConfig, SelectionSettings};

        // What the machine has, then the section 12.1 precedence over it. Broad
        // capture is requested when the operator named no interface; the
        // default-route rule still applies inside select.
        let inventory =
            enumerate().map_err(|e| CliError::failure(format!("interface enumeration: {e}")))?;
        let settings = SelectionSettings {
            explicit: config.interfaces.clone(),
            loopback: config.loopback,
            broad: config.interfaces.is_empty(),
        };
        let outcome = select(&inventory, &settings)
            .map_err(|e| CliError::failure(format!("interface selection: {e}")))?;

        // A live handle per selected interface. The read timeout matches the
        // pipeline's own so a requested stop is observed promptly. Each
        // interface carries its own name and link type into the declarations, so
        // a capture spanning interfaces of different link types declares each
        // correctly rather than assuming the first one's.
        let read_timeout = PipelineConfig::default().read_timeout;
        let mut sources: Vec<SourceBinding> = Vec::new();
        let mut interfaces: Vec<(String, LinkType)> = Vec::new();
        for sel in &outcome.selected {
            let source = LiveSource::open(&sel.record, LiveOptions::for_pipeline(read_timeout))?;
            interfaces.push((sel.record.name.to_string(), source.link_type()));
            let addrs = InterfaceAddrs::new(sel.record.addresses.iter().copied());
            sources.push(SourceBinding::new(sel.id, Box::new(source), addrs));
        }
        // select already errors on NothingSelected, so at least one source was
        // opened and its declaration recorded.
        if interfaces.is_empty() {
            return Err(CliError::failure("no interface was opened for capture"));
        }

        // The inner attributor the pipeline resolves against. Slice 015: with
        // `refresh(&self)` the pipeline shares and refreshes the real socket-table
        // attributor directly through the shared `Arc<dyn FlowAttributor>` its
        // section 8.6 control thread drives, so there is no separate refresh
        // thread and no read/write split to clone. Without the socket table, an
        // empty scripted attributor: packets are retained unattributed rather
        // than having an owner fabricated (P-4 permits the first, P-9 forbids the
        // second).
        #[cfg(feature = "socket-table")]
        let inner: Arc<dyn FlowAttributor> = {
            use fragcap::attr::{IpHelperTable, ToolhelpNamer};
            use fragcap::{AttributorConfig, Clock, SocketTableAttributor, SystemClock};
            let attributor = SocketTableAttributor::new(
                Box::new(IpHelperTable::new()),
                Box::new(ToolhelpNamer::new()),
                Arc::new(SystemClock) as Arc<dyn Clock>,
                AttributorConfig::default(),
            );
            Arc::new(attributor)
        };
        #[cfg(not(feature = "socket-table"))]
        let inner: Arc<dyn FlowAttributor> = Arc::new(ScriptedAttributor::new(
            AttributionScript::parse("").expect("an empty script is valid"),
        ));

        let stamper = RoleStampingAttributor::new(Arc::clone(&inner));
        let publisher = stamper.publisher();

        // The live process event stream, from an ETW session of its own. Kept
        // alive on the components so the receiver stays connected. The watcher
        // took a query-only startup snapshot at start; it is read here so the
        // orchestrator can attach a target already running at arm (section 15.7).
        let watcher = fragcap::EtwWatcher::start("fragcap-cli")
            .map_err(|e| CliError::failure(format!("process trace session: {e}")))?;
        let rx = watcher.subscribe();
        let startup_snapshot = ProcessWatcher::snapshot(&watcher);
        let snapshot_at = watcher.snapshot_taken_at();

        Ok(CaptureComponents {
            sources,
            interfaces,
            stamper: Some(stamper),
            publisher,
            events: EventStream::Live(rx),
            startup_snapshot,
            snapshot_at,
            flow_registry: None,
            watcher: Some(watcher),
        })
    }
}

/// Read the process events from the offline process script, in timestamp order.
///
/// Uses a [`ScriptedWatcher`] to publish the parsed script, which is the same
/// watcher the process-tree tests drive, so the offline events reach the session
/// through the same path a live watcher's would.
fn process_events(
    path: &Option<PathBuf>,
) -> Result<(Vec<ProcessEvent>, Vec<ProcessRecord>), CliError> {
    let Some(path) = path else {
        return Ok((Vec::new(), Vec::new()));
    };
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::failure(format!("process script {}: {e}", path.display())))?;
    let script = parse_process_script(&text)
        .map_err(|e| CliError::failure(format!("process script {}: {e}", path.display())))?;
    let watcher = ScriptedWatcher::new(script);
    let rx = watcher.subscribe();
    // The declared startup snapshot, read before the streamed events, so the
    // orchestrator can attach a target already running at arm. The trait form is
    // explicit because ScriptedWatcher also has an inherent `snapshot` returning a
    // borrow; the ProcessWatcher method returns the owned records.
    let snapshot = ProcessWatcher::snapshot(&watcher);
    watcher.play();
    let mut events: Vec<ProcessEvent> = rx.try_iter().collect();
    events.sort_by_key(|e| e.at().as_nanos());
    Ok((events, snapshot))
}

/// Parse the offline process-script grammar into a [`ProcessScript`].
///
/// A test seam, not an operator-facing format. Each line is one event, tokens
/// separated by whitespace, so an image path or command line carries no spaces:
///
/// ```text
/// # comment
/// snapshot <pid> <parent> <image>
/// start <pid> <parent> <image> <cmdline> <at>
/// exit <pid> <at>
/// ```
///
/// A `snapshot` line declares a process already running at arm, folded into the
/// session by the orchestrator so an already-running target is attached without a
/// later start event (section 15.7). A `cmdline` of `-` records an unavailable
/// command line, so the `cmdline_contains` predicate's handling of an unobserved
/// command line stays testable.
pub fn parse_process_script(text: &str) -> Result<ProcessScript, String> {
    let mut script = ProcessScript::new();
    let mut snapshot: Vec<ProcessRecord> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let content = raw.trim();
        if content.is_empty() || content.starts_with('#') {
            continue;
        }
        let words: Vec<&str> = content.split_whitespace().collect();
        match words[0] {
            "snapshot" => {
                if words.len() != 4 {
                    return Err(format!(
                        "line {line}: snapshot expects <pid> <parent> <image>"
                    ));
                }
                let pid = parse_u32(line, words[1])?;
                let parent = parse_u32(line, words[2])?;
                snapshot.push(ProcessRecord::new(pid, parent, words[3]));
            }
            "start" => {
                if words.len() != 6 {
                    return Err(format!(
                        "line {line}: start expects <pid> <parent> <image> <cmdline> <at>"
                    ));
                }
                let pid = parse_u32(line, words[1])?;
                let parent = parse_u32(line, words[2])?;
                let at = parse_i64(line, words[5])?;
                script = if words[4] == "-" {
                    push_started_without_cmdline(script, pid, parent, words[3], at)
                } else {
                    script.started(pid, parent, words[3], words[4], at)
                };
            }
            "exit" => {
                if words.len() != 3 {
                    return Err(format!("line {line}: exit expects <pid> <at>"));
                }
                let pid = parse_u32(line, words[1])?;
                let at = parse_i64(line, words[2])?;
                script = script.exited(pid, at);
            }
            other => return Err(format!("line {line}: unknown statement `{other}`")),
        }
    }
    if !snapshot.is_empty() {
        script = script.with_snapshot(snapshot);
    }
    Ok(script)
}

/// A start event whose command line the platform did not supply.
fn push_started_without_cmdline(
    script: ProcessScript,
    pid: u32,
    parent: u32,
    image: &str,
    at: i64,
) -> ProcessScript {
    // The builder's `started_without_cmdline` records `CommandLine::Unavailable`
    // exactly as a snapshot process would. Named here through the import so the
    // intent is legible.
    let _ = CommandLine::Unavailable;
    script.started_without_cmdline(pid, parent, image, at)
}

fn parse_u32(line: usize, word: &str) -> Result<u32, String> {
    word.parse()
        .map_err(|_| format!("line {line}: `{word}` is not a process identifier"))
}

fn parse_i64(line: usize, word: &str) -> Result<i64, String> {
    word.parse()
        .map_err(|_| format!("line {line}: `{word}` is not a timestamp"))
}

/// The image file name from a start event, for the `stage.matched` event.
pub fn image_name(event: &ProcessEvent) -> String {
    match event {
        ProcessEvent::Started { image, .. } => image
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or(image)
            .to_string(),
        _ => String::new(),
    }
}

/// The output sinks plus the per-consumer report handles of any streaming
/// sinks, which the orchestrator reads after the run finishes.
pub struct BuiltSinks {
    pub sinks: Vec<Box<dyn Sink>>,
    pub stream_reports: Vec<StreamReports>,
    /// The ring sink's eviction counter, when the run is in ring mode. The
    /// orchestrator reads it after the run to surface the ring's own retention
    /// accounting (packets evicted from the rolling window) in the summary, the
    /// way `stream_reports` surfaces a streaming sink's per-consumer drops. `None`
    /// outside ring mode.
    pub ring_evicted: Option<Arc<AtomicU64>>,
}

/// One streaming sink's transport description and its report handle.
pub struct StreamReports {
    pub transport: String,
    pub handle: Arc<Mutex<Vec<ConsumerReport>>>,
}

/// Build the output sinks from the effective configuration.
///
/// Format is orthogonal to transport (specification 14.1): a `SinkFactory`
/// builds a fresh encoder per file segment and per streaming consumer. File
/// destinations become rotating file sinks (a single segment when no rotation
/// is asked, byte identical to before); the named pipe, Unix socket, and TCP
/// destinations become streaming sinks over their acceptors. A mismatched
/// option, an unresolvable format, or a platform-unavailable transport is a
/// configuration error naming the cause (FR-014). The tee that feeds the
/// session is prepended by the orchestrator, not here.
pub fn build_sinks(
    config: &EffectiveConfig,
    interfaces: &[(String, LinkType)],
) -> Result<BuiltSinks, CliError> {
    let iface_specs: Vec<InterfaceSpec> = interfaces
        .iter()
        .map(|(name, link)| InterfaceSpec::new(name, *link, SNAP_LEN))
        .collect();

    let mut sinks: Vec<Box<dyn Sink>> = Vec::new();
    let mut stream_reports: Vec<StreamReports> = Vec::new();
    let mut ring_evicted: Option<Arc<AtomicU64>> = None;

    // The primary capture file, from --out, is a pcapng sink. `--out` is the
    // pcapng shorthand and is always pcapng regardless of the filename extension;
    // a JSON Lines file is asked for with an explicit `--sink jsonl:` or
    // `--sink ...,format=jsonl`. In ring mode the --out file is the ring dump
    // target: a RingSink retains a rolling window in memory and materializes it
    // to this file at drain, instead of the continuous file sink.
    if let Some(out) = &config.out {
        let factory = SinkFactory::new(Format::Pcapng, iface_specs.clone());
        if config.mode == CaptureMode::Ring {
            // Ring mode is validated to carry a window (reject_unsupported); the
            // fallback keeps this total rather than panicking if that ever changes.
            let window = config.ring.ok_or_else(|| {
                CliError::usage("ring mode needs a ring window; pass --ring <DURATION|SIZE>")
            })?;
            // The ring dump file is opened here, before capture, so an unwritable
            // --out fails now rather than after the whole window is captured.
            let sink = RingSink::create(out.clone(), window, factory)
                .map_err(|e| CliError::failure(e.to_string()))?;
            // Keep the eviction handle so the orchestrator can surface the count.
            ring_evicted = Some(sink.evicted_handle());
            sinks.push(Box::new(sink));
        } else {
            sinks.push(Box::new(
                RotatingFileSink::create(out, RotationPolicy::None, factory)
                    .map_err(|e| CliError::failure(e.to_string()))?,
            ));
        }
    }

    for spec in &config.sinks {
        build_one_sink(spec, config, &iface_specs, &mut sinks, &mut stream_reports)?;
    }

    Ok(BuiltSinks {
        sinks,
        stream_reports,
        ring_evicted,
    })
}

/// The pcapng-or-JSON-Lines format for a file path, inferred from its extension
/// and the global payload mode. `.jsonl` is JSON Lines; anything else is pcapng.
fn pcapng_or_jsonl(path: &std::path::Path, payload: bool) -> Format {
    let is_jsonl = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("jsonl"))
        .unwrap_or(false);
    if is_jsonl {
        Format::JsonLines(payload_mode(payload))
    } else {
        Format::Pcapng
    }
}

fn payload_mode(payload: bool) -> PayloadMode {
    if payload {
        PayloadMode::WithPayload
    } else {
        PayloadMode::MetadataOnly
    }
}

/// Resolve the format for a sink: the `format=` qualifier wins; otherwise a
/// file path is inferred from its extension. A transport with no extension (a
/// pipe, a socket, TCP) and no explicit `format=` is a configuration error, per
/// specification 14.1, rather than a silent default that might send pcapng when
/// the operator meant JSON Lines.
fn resolve_format(
    spec: &SinkSpec,
    config: &EffectiveConfig,
    path: Option<&std::path::Path>,
) -> Result<Format, CliError> {
    let payload = spec.payload.unwrap_or(config.payload);
    match spec.format {
        Some(SinkFormat::Pcapng) => Ok(Format::Pcapng),
        Some(SinkFormat::JsonLines) => Ok(Format::JsonLines(payload_mode(payload))),
        None => match path {
            Some(path) => Ok(pcapng_or_jsonl(path, payload)),
            None => Err(CliError::usage(
                "this sink has no file extension to infer a format from; add `,format=pcapng` \
                 or `,format=jsonl`",
            )),
        },
    }
}

/// The rotation policy from a file sink's options, refusing both at once.
fn rotation_policy(spec: &SinkSpec) -> Result<RotationPolicy, CliError> {
    match (spec.rotate_size, spec.rotate_duration) {
        (Some(_), Some(_)) => Err(CliError::usage(
            "a sink takes at most one of rotate-size and rotate-duration",
        )),
        (Some(bytes), None) => Ok(RotationPolicy::Size(bytes)),
        (None, Some(window)) => Ok(RotationPolicy::Duration(window)),
        (None, None) => Ok(RotationPolicy::None),
    }
}

/// Reject streaming options on a file sink.
fn no_stream_options(spec: &SinkSpec) -> Result<(), CliError> {
    if spec.queue.is_some() || spec.timeout.is_some() {
        return Err(CliError::usage(
            "queue and timeout apply to streaming sinks, not a file sink",
        ));
    }
    Ok(())
}

/// Reject file rotation options on a streaming sink.
fn no_rotation_options(spec: &SinkSpec) -> Result<(), CliError> {
    if spec.rotate_size.is_some() || spec.rotate_duration.is_some() {
        return Err(CliError::usage(
            "rotate-size and rotate-duration apply to file sinks, not a streaming sink",
        ));
    }
    Ok(())
}

/// The default per-consumer queue depth and disconnect timeout, matching the
/// streaming sink's own defaults.
const DEFAULT_QUEUE: usize = 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

fn make_stream_sink(
    factory: SinkFactory,
    acceptor: Box<dyn fragcap::Acceptor>,
    spec: &SinkSpec,
    stream_reports: &mut Vec<StreamReports>,
) -> Box<dyn Sink> {
    let sink = StreamSink::with_settings(
        factory,
        acceptor,
        spec.queue.unwrap_or(DEFAULT_QUEUE),
        spec.timeout.unwrap_or(DEFAULT_TIMEOUT),
    );
    stream_reports.push(StreamReports {
        transport: sink.transport().to_string(),
        handle: sink.reports_handle(),
    });
    Box::new(sink)
}

fn build_one_sink(
    spec: &SinkSpec,
    config: &EffectiveConfig,
    iface_specs: &[InterfaceSpec],
    sinks: &mut Vec<Box<dyn Sink>>,
    stream_reports: &mut Vec<StreamReports>,
) -> Result<(), CliError> {
    match &spec.transport {
        SinkTransport::File(path) => {
            no_stream_options(spec)?;
            let format = resolve_format(spec, config, Some(path))?;
            let policy = rotation_policy(spec)?;
            let factory = SinkFactory::new(format, iface_specs.to_vec());
            sinks.push(Box::new(
                RotatingFileSink::create(path, policy, factory)
                    .map_err(|e| CliError::failure(e.to_string()))?,
            ));
        }
        SinkTransport::JsonLines(path) => {
            no_stream_options(spec)?;
            // The jsonl: scheme fixes the format unless a format= says otherwise.
            let format = match spec.format {
                Some(SinkFormat::Pcapng) => Format::Pcapng,
                _ => Format::JsonLines(payload_mode(spec.payload.unwrap_or(config.payload))),
            };
            let policy = rotation_policy(spec)?;
            let factory = SinkFactory::new(format, iface_specs.to_vec());
            sinks.push(Box::new(
                RotatingFileSink::create(path, policy, factory)
                    .map_err(|e| CliError::failure(e.to_string()))?,
            ));
        }
        SinkTransport::Tcp(authority) => {
            no_rotation_options(spec)?;
            let format = resolve_format(spec, config, None)?;
            let factory = SinkFactory::new(format, iface_specs.to_vec());
            let acceptor = fragcap::TcpAcceptor::bind(authority)
                .map_err(|e| CliError::failure(format!("cannot bind {authority}: {e}")))?;
            sinks.push(make_stream_sink(
                factory,
                Box::new(acceptor),
                spec,
                stream_reports,
            ));
        }
        SinkTransport::Pipe(name) => {
            no_rotation_options(spec)?;
            build_pipe_sink(name, spec, config, iface_specs, sinks, stream_reports)?;
        }
        SinkTransport::Fifo(path) => {
            build_fifo_sink(path, spec, iface_specs, sinks)?;
        }
        SinkTransport::Unix(path) => {
            no_rotation_options(spec)?;
            build_unix_sink(path, spec, config, iface_specs, sinks, stream_reports)?;
        }
    }
    Ok(())
}

/// Build the FIFO sink for the extcap integration (specification section 14.5).
///
/// The analyzer hands fragcap a FIFO or named-pipe path; `open_fifo` opens it for
/// writing (a named-pipe client on Windows, an opened FIFO or file elsewhere) and
/// a pcapng `SinkFactory` encoder writes the same bytes a file capture produces
/// (constitution P-5). A FIFO is a single write-only stream, not a file and not a
/// streaming server, so the file rotation options and the streaming options do
/// not apply, and JSON Lines is not an extcap transport.
fn build_fifo_sink(
    path: &std::path::Path,
    spec: &SinkSpec,
    iface_specs: &[InterfaceSpec],
    sinks: &mut Vec<Box<dyn Sink>>,
) -> Result<(), CliError> {
    no_stream_options(spec)?;
    no_rotation_options(spec)?;
    if matches!(spec.format, Some(SinkFormat::JsonLines)) {
        return Err(CliError::usage(
            "a fifo: sink streams pcapng to an analyzer; format=jsonl does not apply",
        ));
    }
    let factory = SinkFactory::new(Format::Pcapng, iface_specs.to_vec());
    let writer = fragcap::open_fifo(path)
        .map_err(|e| CliError::failure(format!("cannot open fifo {}: {e}", path.display())))?;
    let sink = factory
        .build(writer)
        .map_err(|e| CliError::failure(e.to_string()))?;
    sinks.push(sink);
    Ok(())
}

#[cfg(windows)]
fn build_pipe_sink(
    name: &str,
    spec: &SinkSpec,
    config: &EffectiveConfig,
    iface_specs: &[InterfaceSpec],
    sinks: &mut Vec<Box<dyn Sink>>,
    stream_reports: &mut Vec<StreamReports>,
) -> Result<(), CliError> {
    let format = resolve_format(spec, config, None)?;
    let factory = SinkFactory::new(format, iface_specs.to_vec());
    let acceptor = fragcap::NamedPipeAcceptor::bind(name)
        .map_err(|e| CliError::failure(format!("cannot create pipe {name}: {e}")))?;
    sinks.push(make_stream_sink(
        factory,
        Box::new(acceptor),
        spec,
        stream_reports,
    ));
    Ok(())
}

#[cfg(not(windows))]
fn build_pipe_sink(
    _name: &str,
    _spec: &SinkSpec,
    _config: &EffectiveConfig,
    _iface_specs: &[InterfaceSpec],
    _sinks: &mut Vec<Box<dyn Sink>>,
    _stream_reports: &mut Vec<StreamReports>,
) -> Result<(), CliError> {
    Err(CliError::usage(
        "named-pipe sinks (pipe:) are available on Windows only",
    ))
}

#[cfg(unix)]
fn build_unix_sink(
    path: &std::path::Path,
    spec: &SinkSpec,
    config: &EffectiveConfig,
    iface_specs: &[InterfaceSpec],
    sinks: &mut Vec<Box<dyn Sink>>,
    stream_reports: &mut Vec<StreamReports>,
) -> Result<(), CliError> {
    let format = resolve_format(spec, config, None)?;
    let factory = SinkFactory::new(format, iface_specs.to_vec());
    let acceptor = fragcap::UnixAcceptor::bind(path)
        .map_err(|e| CliError::failure(format!("cannot bind {}: {e}", path.display())))?;
    sinks.push(make_stream_sink(
        factory,
        Box::new(acceptor),
        spec,
        stream_reports,
    ));
    Ok(())
}

#[cfg(not(unix))]
fn build_unix_sink(
    _path: &std::path::Path,
    _spec: &SinkSpec,
    _config: &EffectiveConfig,
    _iface_specs: &[InterfaceSpec],
    _sinks: &mut Vec<Box<dyn Sink>>,
    _stream_reports: &mut Vec<StreamReports>,
) -> Result<(), CliError> {
    Err(CliError::usage(
        "Unix domain socket sinks (unix:) are available on Unix targets only",
    ))
}

/// The arm instant used for the offline session clock.
pub const ARMED_AT: Timestamp = Timestamp::from_nanos(0);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::parse_sink;

    fn base_config() -> EffectiveConfig {
        EffectiveConfig {
            mode: CaptureMode::File,
            ring: None,
            out: None,
            sinks: Vec::new(),
            duration: None,
            acquisition_timeout: None,
            max_packets: None,
            max_bytes: None,
            roles: None,
            scope: CaptureScope::All,
            direction: Direction::Both,
            interfaces: vec!["eth0".to_string()],
            loopback: false,
            payload: true,
            launch: None,
        }
    }

    fn one_interface() -> Vec<(String, LinkType)> {
        vec![("eth0".to_string(), LinkType::ETHERNET)]
    }

    // --- Elevation gate (US6) -------------------------------------------

    #[test]
    fn an_unelevated_session_is_refused_at_exit_one() {
        let refusal = elevation_refusal(false).expect("not elevated must refuse");
        assert_eq!(refusal.exit(), crate::exit::Exit::FAILURE);
        assert!(
            refusal.message().contains("Administrator"),
            "the refusal says elevation is required: {}",
            refusal.message()
        );
    }

    #[test]
    fn an_elevated_session_is_allowed_through() {
        assert!(
            elevation_refusal(true).is_none(),
            "an elevated session is not refused"
        );
    }

    fn finish_all(built: BuiltSinks) {
        use fragcap::CaptureStats;
        for sink in built.sinks {
            let _ = sink.finish(&CaptureStats::default());
        }
    }

    // --- Ring mode configuration (slice S16, US2) -----------------------

    use clap::Parser;

    /// Parse a `capture` invocation into its `CaptureArgs`, for the ring and launch
    /// refusal tests. The profile the tests pass to `effective_config` supplies the
    /// capture semantics; the `--process` target input only needs to parse.
    fn run_args(extra: &[&str]) -> Box<crate::cli::CaptureArgs> {
        let mut argv = vec!["fragcap", "capture", "--process", "game.exe"];
        argv.extend_from_slice(extra);
        match crate::cli::Cli::try_parse_from(argv)
            .expect("args parse")
            .command
        {
            Some(crate::cli::Command::Capture(a)) => a,
            _ => unreachable!("capture parses to Command::Capture"),
        }
    }

    /// A minimal valid profile with no `[capture]` mode declared.
    fn plain_profile() -> Profile {
        Profile::parse(
            r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"g","name":"G"},"stage":[{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"game.exe"}}]}"#,
        )
        .expect("the profile parses")
    }

    /// A minimal valid profile declaring ring mode in its `[capture]` table.
    fn ring_mode_profile() -> Profile {
        Profile::parse(
            r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"g","name":"G"},"capture":{"mode":"ring"},"stage":[{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"game.exe"}}]}"#,
        )
        .expect("the profile parses")
    }

    /// A minimal valid profile declaring a Steam platform and app_id.
    fn steam_profile() -> Profile {
        Profile::parse(
            r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"g","name":"G","platform":"steam","app_id":"900883"},"stage":[{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"game.exe"}}]}"#,
        )
        .expect("the profile parses")
    }

    // Managed launch (S17). Without --launch there is no managed launch.
    #[test]
    fn without_launch_there_is_no_managed_launch() {
        let args = run_args(&[]);
        let cfg = effective_config(&args, &steam_profile()).unwrap();
        assert!(cfg.launch.is_none());
    }

    // FR-010. --launch on a Steam profile requests the run URL for its app_id.
    #[cfg(windows)]
    #[test]
    fn launch_on_a_steam_profile_requests_the_run_url() {
        let args = run_args(&["--launch"]);
        let cfg = effective_config(&args, &steam_profile()).unwrap();
        let req = cfg.launch.expect("a launch request");
        let fragcap::managed_launch::ManagedLaunch::Steam(req) = req else {
            panic!("expected Steam launch");
        };
        assert_eq!(req.url, "steam://run/900883");
    }

    // FR-011. --launch without a declared app_id is refused before capture.
    #[cfg(windows)]
    #[test]
    fn launch_without_app_id_is_refused_before_capture() {
        let no_app_id = Profile::parse(
            r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"g","name":"G","platform":"steam"},"stage":[{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"game.exe"}}]}"#,
        )
        .expect("parses");
        let args = run_args(&["--launch"]);
        let err = effective_config(&args, &no_app_id).unwrap_err();
        assert!(err.to_string().contains("app_id"), "{err}");
    }

    // Off Windows, --launch is refused as unsupported before capture.
    #[cfg(not(windows))]
    #[test]
    fn launch_is_refused_as_windows_only_off_windows() {
        let args = run_args(&["--launch"]);
        let err = effective_config(&args, &steam_profile()).unwrap_err();
        assert!(err.to_string().contains("Windows"), "{err}");
    }

    // FR-005. Ring mode without an output file is refused before capture.
    #[test]
    fn ring_mode_without_out_is_refused() {
        let args = run_args(&["--mode", "ring", "--ring", "10m"]);
        let err = effective_config(&args, &plain_profile()).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("output"), "{err}");
    }

    // FR-005. Ring mode without a window is refused before capture.
    #[test]
    fn ring_mode_without_a_window_is_refused() {
        let args = run_args(&["--mode", "ring", "--out", "x.fcapng"]);
        let err = effective_config(&args, &plain_profile()).unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("ring window"),
            "{err}"
        );
    }

    // FR-006. A volume stop bound does not apply to a rolling window.
    #[test]
    fn a_volume_bound_in_ring_mode_is_refused() {
        for bound in [["--max-packets", "1000"], ["--max-bytes", "1mb"]] {
            let args = run_args(&[
                "--mode", "ring", "--ring", "10m", "--out", "x.fcapng", bound[0], bound[1],
            ]);
            let err = effective_config(&args, &plain_profile()).unwrap_err();
            assert!(
                err.to_string().to_lowercase().contains("ring mode"),
                "{bound:?}: {err}"
            );
        }
    }

    // FR-007. A ring window given outside ring mode is refused rather than
    // silently ignored.
    #[test]
    fn a_ring_window_without_ring_mode_is_refused() {
        let args = run_args(&["--ring", "10m", "--out", "x.fcapng"]);
        let err = effective_config(&args, &plain_profile()).unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("ring mode"),
            "{err}"
        );
    }

    // FR-008. A profile declaring ring mode selects it with no `--mode` override,
    // and is validated identically (here, the missing window is refused).
    #[test]
    fn a_profile_declared_ring_mode_is_resolved_and_validated() {
        let args = run_args(&["--out", "x.fcapng"]);
        assert_eq!(resolve_mode(&args, &ring_mode_profile()), CaptureMode::Ring);
        let err = effective_config(&args, &ring_mode_profile()).unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("ring window"),
            "{err}"
        );
    }

    // FR-010. `--duration` remains valid in ring mode; a valid ring configuration
    // is accepted and carries the window.
    #[test]
    fn a_valid_ring_configuration_is_accepted_with_duration() {
        let args = run_args(&[
            "--mode",
            "ring",
            "--ring",
            "5m",
            "--duration",
            "30m",
            "--out",
            "x.fcapng",
        ]);
        let config = effective_config(&args, &plain_profile()).expect("a valid ring config");
        assert_eq!(config.mode, CaptureMode::Ring);
        assert_eq!(
            config.ring,
            Some(RingWindow::Duration(Duration::from_secs(300)))
        );
        assert_eq!(config.duration, Some(Duration::from_secs(1800)));
    }

    // The valid ring configuration builds a sink set (the ring dump over --out).
    #[test]
    fn a_valid_ring_configuration_builds_a_sink() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = base_config();
        config.mode = CaptureMode::Ring;
        config.ring = Some(RingWindow::Size(64 * 1024));
        config.out = Some(dir.path().join("ring.fcapng"));
        let built = build_sinks(&config, &one_interface()).expect("build");
        assert_eq!(built.sinks.len(), 1, "the ring dump is the one sink");
        finish_all(built);
    }

    #[test]
    fn a_tcp_sink_builds_a_streaming_sink_with_a_report_handle() {
        let mut config = base_config();
        config.sinks = vec![parse_sink("tcp://127.0.0.1:0,format=pcapng").unwrap()];
        let built = build_sinks(&config, &one_interface()).expect("build");
        assert_eq!(built.sinks.len(), 1);
        assert_eq!(built.stream_reports.len(), 1);
        assert!(built.stream_reports[0].transport.starts_with("tcp://"));
        finish_all(built);
    }

    #[test]
    fn an_extensionless_sink_without_a_format_is_refused() {
        let mut config = base_config();
        config.sinks = vec![parse_sink("tcp://127.0.0.1:0").unwrap()];
        match build_sinks(&config, &one_interface()) {
            Err(e) => assert!(e.to_string().to_lowercase().contains("format")),
            Ok(_) => panic!("an extensionless sink with no format= must be refused"),
        }
    }

    #[test]
    fn out_is_pcapng_regardless_of_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("capture.jsonl");
        let mut config = base_config();
        config.out = Some(path.clone());
        let built = build_sinks(&config, &one_interface()).expect("build");
        finish_all(built);
        // --out is the pcapng shorthand: a .jsonl name still writes pcapng, so
        // the file begins with the Section Header Block magic, not `{`.
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], &0x0A0D_0D0Au32.to_le_bytes());
    }

    #[test]
    fn a_file_sink_with_rotation_builds() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.fcapng");
        let mut config = base_config();
        config.sinks =
            vec![parse_sink(&format!("file:{},rotate-size=1mb", path.display())).unwrap()];
        let built = build_sinks(&config, &one_interface()).expect("build");
        assert_eq!(built.sinks.len(), 1);
        assert!(built.stream_reports.is_empty());
        finish_all(built);
    }

    #[test]
    fn rotation_options_on_a_streaming_sink_are_refused() {
        let mut config = base_config();
        config.sinks = vec![parse_sink("tcp://127.0.0.1:0,rotate-size=1mb").unwrap()];
        assert!(build_sinks(&config, &one_interface()).is_err());
    }

    #[test]
    fn streaming_options_on_a_file_sink_are_refused() {
        let mut config = base_config();
        config.sinks = vec![parse_sink("file:out.fcapng,queue=8").unwrap()];
        assert!(build_sinks(&config, &one_interface()).is_err());
    }

    #[cfg(not(unix))]
    #[test]
    fn a_unix_sink_off_unix_is_refused_naming_the_platform() {
        let mut config = base_config();
        config.sinks = vec![parse_sink("unix:/tmp/fragcap.sock").unwrap()];
        match build_sinks(&config, &one_interface()) {
            Err(e) => assert!(e.to_string().to_lowercase().contains("unix")),
            Ok(_) => panic!("a unix sink off unix must be refused"),
        }
    }

    #[test]
    fn the_process_script_parses_starts_and_exits() {
        let script = parse_process_script(
            "# a comment\nstart 4242 100 C:\\Games\\game.exe game.exe 0\nexit 4242 9999\n",
        )
        .expect("the script parses");
        assert_eq!(script.events().len(), 2);
        assert_eq!(script.events()[0].pid(), 4242);
        assert_eq!(script.events()[1].pid(), 4242);
    }

    #[test]
    fn a_dash_command_line_is_unavailable() {
        let script = parse_process_script("start 1 0 C:\\a.exe - 5\n").expect("the script parses");
        match &script.events()[0] {
            ProcessEvent::Started { command_line, .. } => {
                assert_eq!(*command_line, CommandLine::Unavailable);
            }
            _ => panic!("expected a start event"),
        }
    }

    #[test]
    fn a_malformed_process_line_reports_its_line() {
        assert!(parse_process_script("start 1 0\n")
            .unwrap_err()
            .contains("line 1"));
        assert!(parse_process_script("wat\n")
            .unwrap_err()
            .contains("unknown statement"));
    }

    // Tier 2 by specification section 25.2. Gated behind the feature and target
    // so the --all-features gate compiles it, and #[ignore]d so a runner with no
    // capture driver, no elevation, and no game does not run it. It mirrors the
    // fragcap-attr tier-2 pattern: the socket-table attributor assembles from the
    // real IP Helper table and toolhelp namer and reads the machine's own table.
    #[cfg(all(feature = "socket-table", windows))]
    #[test]
    #[ignore = "tier 2: reads the machine's real socket table"]
    fn the_socket_table_attributor_assembles_and_reads_the_real_table() {
        use fragcap::attr::{IpHelperTable, ToolhelpNamer};
        use fragcap::core::FlowAttributor;
        use fragcap::{AttributorConfig, Clock, SocketTableAttributor, SystemClock};

        let attributor = SocketTableAttributor::new(
            Box::new(IpHelperTable::new()),
            Box::new(ToolhelpNamer::new()),
            Arc::new(SystemClock) as Arc<dyn Clock>,
            AttributorConfig::default(),
        );
        // The read/write split assembles: a resolver cloned from the attributor
        // is what the pipeline holds while the mutable attributor is refreshed.
        let resolver = attributor.resolver();
        attributor
            .refresh()
            .expect("the machine's socket table reads");
        // The resolver reads the same publication the refresh just wrote.
        let _endpoints = resolver.active_endpoints();
    }

    // Tier 2. The ETW process watcher starts a kernel trace session, which needs
    // elevation, so this is #[ignore]d. It exists to keep the live event-stream
    // construction compiled under the --all-features gate.
    #[cfg(all(feature = "etw", windows))]
    #[test]
    #[ignore = "tier 2: starts an ETW session, which needs elevation"]
    fn the_etw_watcher_starts_and_yields_a_receiver() {
        let watcher =
            fragcap::EtwWatcher::start("fragcap-cli-test").expect("an elevated trace session");
        let _rx = watcher.subscribe();
    }
}
