// SPDX-License-Identifier: Apache-2.0

//! Facade crate for fragcap.
//!
//! The library is the product; the command line tool is one consumer of it.
//! Anything reachable through the CLI is reachable through this crate.
//!
//! Depends on `fragcap-core` directly, in addition to the mid-level crates.
//! The dependency diagram in specification section 8.3 omits that edge, but a
//! facade that re-exports core types needs core as a direct dependency. The
//! edge violates neither stated rule: it is not a dependency on the binary
//! crate, and it is not a sibling-level dependency. Recorded as decision D-1.
//!
//! Re-exports arrive as the crates below gain surface. Slice S04 is the first
//! to add any: the tier 1 substrate that makes specification section 25.1's
//! claim true, namely that the whole pipeline runs with no capture driver, no
//! elevated privilege, and no game.
//!
//! Slice S05 added [`profile`], which is the first surface an operator writes
//! against rather than a contributor: a profile is a TOML file, and adding
//! support for a game never requires modifying Rust.

/// Types, traits, and the header parser.
pub mod core {
    /// The shared duration and size literal grammars, so the command line
    /// parses `30m` and `4mb` through the same code a profile does.
    pub use fragcap_core::{duration, size};

    pub use fragcap_core::attribution::{Attribution, StageId};
    pub use fragcap_core::duration::DurationError;
    pub use fragcap_core::error::{AttrError, SinkError, SourceError};
    pub use fragcap_core::flow::{AttributionKey, Direction, Endpoint, FlowKey, Proto};
    pub use fragcap_core::interface::{
        select, InterfaceId, InterfaceInventory, InterfaceRecord, SelectedInterface,
        SelectionError, SelectionOutcome, SelectionSettings,
    };
    pub use fragcap_core::link::LinkType;
    pub use fragcap_core::packet::{
        AttributionState, CapturedPacket, Payload, RawPacket, Timestamp,
    };
    pub use fragcap_core::parse::{HeaderParser, InterfaceAddrs, ParseOutcome, ParseReject};
    pub use fragcap_core::pipeline::{
        ConfigError, EndReason, Pipeline, PipelineConfig, PipelineError, PipelineReport,
        SinkFailure, SourceBinding, StopHandle, DEFAULT_CAPACITY, DEFAULT_READ_TIMEOUT,
    };
    pub use fragcap_core::process::tree::NodeId;
    pub use fragcap_core::process::{
        Ancestry, CommandLine, ProcessEvent, ProcessId, ProcessNode, ProcessRecord, ProcessTree,
    };
    pub use fragcap_core::size::SizeError;
    pub use fragcap_core::stats::{CaptureStats, ParseStats, SourceStats};
    pub use fragcap_core::traits::{
        Dissector, FlowAttributor, PacketSource, ProcessWatcher, Sink, WriteGate,
    };
}

/// Steam platform integration (specification section 16).
///
/// Library discovery, profile scaffolding, and managed launch. Carries no
/// capture and no attribution logic; the Windows-only internals are behind
/// `#[cfg(windows)]` in the crate, so the facade re-export builds everywhere.
pub mod steam {
    pub use fragcap_steam::{
        discover, discover_in, install_root_for, install_root_in, launch, launch_request, scaffold,
        InstallLookup, InstalledTitle, LaunchConfigError, LaunchRequest, SteamError,
        SteamInstallation, SteamLibrary, SteamWalkerProvider,
    };
}

/// The targets hint database (issue #78, slice S034), behind the `targets`
/// feature.
///
/// An embedded SQLite store of game launch hints and its schema-conformant
/// `kind: "export"` projection. Off by default so a build that does not want the
/// store compiles no SQLite engine; the `fragcap-cli` binary enables it so the
/// shipped tool carries the `targets` subcommand.
#[cfg(feature = "targets")]
pub mod targets {
    pub use fragcap_targets::{
        export, import, seed_catalog, CatalogBatch, CatalogEntry, CatalogSource, Classification,
        CorpusGate, Engine, EngineConfidence, EngineSource, FixtureCatalog, Game, ImportSummary,
        LaunchEntry, SeedState, SeedSummary, SeedTier, Store, TargetsError, TechCategory,
        Technology, DEFAULT_MIN_REVIEWS,
    };

    /// The live catalog source, behind the `net` feature (slice S035). Compiled
    /// under `net` but run only by the operator, never in CI.
    #[cfg(feature = "net")]
    pub use fragcap_targets::HttpCatalog;
}

/// Packet acquisition.
pub mod capture {
    pub use fragcap_capture::pcap::{PcapReader, ReplayStats};
    pub use fragcap_capture::replay::ReplaySource;

    /// The live capture backend and interface enumeration, behind the `live`
    /// feature on Windows. The replay source above is the offline stand-in that
    /// drives every tier-1 test.
    #[cfg(all(feature = "live", windows))]
    pub use fragcap_capture::live::{detect_driver, enumerate, LiveOptions, LiveSource};
}

/// Flow attribution.
pub mod attr {
    pub use fragcap_attr::index::{
        AttributionIndex, MatchRank, PublishedIndex, RetainedEntry, RetentionMap,
    };
    pub use fragcap_attr::proc_script::{ProcessScript, ScriptedWatcher};
    pub use fragcap_attr::resolver::PublishedResolver;
    pub use fragcap_attr::schedule::RefreshSchedule;
    pub use fragcap_attr::script::{AttributionScript, ScriptEntry, ScriptError, Window};
    pub use fragcap_attr::scripted::ScriptedAttributor;
    pub use fragcap_attr::seam::{
        Clock, DeclaredNames, DeclaredTable, ProcessNamer, SocketTableSource, SystemClock,
        TestClock,
    };
    pub use fragcap_attr::socket::{AttributorConfig, SocketTableAttributor};
    pub use fragcap_attr::table::{SocketTable, SocketTableEntry};

    /// The Windows socket table backends, behind the `socket-table` feature.
    #[cfg(all(feature = "socket-table", windows))]
    pub use fragcap_attr::platform::{IpHelperTable, ToolhelpNamer};

    /// The ETW process watcher, behind the `etw` feature. The scripted watcher
    /// above is the offline stand-in that drives every tier-1 test.
    #[cfg(all(feature = "etw", windows))]
    pub use fragcap_attr::etw::EtwWatcher;
}

/// Game profiles: schema, validation, resolution, and stage matching.
pub mod profile {
    pub use fragcap_profile::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Position};
    pub use fragcap_profile::engine_rule::Engine;
    pub use fragcap_profile::glob::{ImagePattern, PatternError};
    /// The master JSON Schema and its structural validation surface (issue #75).
    pub use fragcap_profile::jsonschema::{
        schema_document, validate_json, SchemaCode, SchemaDiagnostic, SchemaDiagnostics, Validation,
    };
    pub use fragcap_profile::matching::{bind_stages, first_live_match, stage_for};
    pub use fragcap_profile::parse::{load, LoadError, MAX_PROFILE_BYTES};
    /// The target resolution cascade: providers, precedence, and the resolver
    /// (issue #77, section 15.7).
    pub use fragcap_profile::providers::{
        EngineRuleProvider, HintProvider, ObservationProvider, ProfileProvider,
    };
    pub use fragcap_profile::resolve::{
        resolve, BundledSet, DuplicateGameId, ProfileSource, ResolveError, Resolved, SearchPath,
    };
    pub use fragcap_profile::resolver::{
        DuplicatePrecedence, EngineRuleAmbiguity, Precedence, ProviderError, ResolutionError,
        ResolutionNotes, ResolutionRequest, TargetProvider, TargetResolver, Unresolved,
        WalkerAmbiguity,
    };
    pub use fragcap_profile::schema::{
        CaptureDefaults, CaptureMode, FidelityTier, Game, GameId, Kind, Lifecycle, MatchPredicates,
        PathRegex, Profile, Provenance, Stage, SCHEMA_VERSION,
    };
    pub use fragcap_profile::target::{
        EngineRuleTarget, ObservedTarget, Target, TargetOrigin, WalkerTarget,
    };
    pub use fragcap_profile::technologies::{
        Category, CompiledRuleset, DetectError, ScanOutcome, SkippedPattern, TechnologyFinding,
    };
}

/// Output sinks.
pub mod sink {
    pub use fragcap_sink::annotation::{
        AnnotatedDirection, Annotation, AnnotationError, Fidelity, SENTINEL,
    };
    pub use fragcap_sink::error::WriteError;
    pub use fragcap_sink::json::{
        write_json_string, JsonLinesWriter, PayloadMode, VERSION as JSONL_VERSION,
    };
    pub use fragcap_sink::pcapng::interface::InterfaceDeclaration;
    pub use fragcap_sink::pcapng::{PcapngWriter, PROFILE_COMMENT, USER_APPL};
    pub use fragcap_sink::transport::fifo::open_fifo;
    pub use fragcap_sink::transport::file::{RotatingFileSink, RotationPolicy};
    #[cfg(windows)]
    pub use fragcap_sink::transport::pipe::NamedPipeAcceptor;
    pub use fragcap_sink::transport::ring::{RingSink, RingWindow};
    pub use fragcap_sink::transport::stream::{ConsumerReport, DisconnectReason, StreamSink};
    pub use fragcap_sink::transport::tcp::TcpAcceptor;
    #[cfg(unix)]
    pub use fragcap_sink::transport::unix::UnixAcceptor;
    pub use fragcap_sink::transport::{
        Acceptor, ConnShutdown, Connection, Format, InterfaceSpec, SinkFactory,
    };
}

/// The capture session lifecycle: stage matching drives the five-state machine
/// of specification sections 10.5 and 10.6.
pub mod session;

pub use crate::core::*;
pub use crate::core::{EndReason, Pipeline, PipelineConfig, PipelineReport, StopHandle};
pub use crate::session::{
    BindingPublisher, CaptureSession, GateHandle, PacketDisposition, RoleStampingAttributor,
    SessionConfig, SessionGate, SessionState, SessionStats, StopReason,
};
#[cfg(all(feature = "etw", windows))]
pub use attr::EtwWatcher;
pub use attr::{
    AttributionScript, AttributorConfig, Clock, DeclaredNames, DeclaredTable, ProcessScript,
    ScriptedAttributor, ScriptedWatcher, SocketTable, SocketTableAttributor, SocketTableEntry,
    SystemClock, TestClock,
};
#[cfg(all(feature = "socket-table", windows))]
pub use attr::{IpHelperTable, ToolhelpNamer};
#[cfg(all(feature = "live", windows))]
pub use capture::{detect_driver, enumerate, LiveOptions, LiveSource};
pub use capture::{PcapReader, ReplaySource, ReplayStats};
pub use profile::{
    resolve, BundledSet, Diagnostic, DiagnosticCode, Diagnostics, Profile, ProfileSource,
    ResolveError, SearchPath,
};
#[cfg(windows)]
pub use sink::NamedPipeAcceptor;
#[cfg(unix)]
pub use sink::UnixAcceptor;
pub use sink::{
    open_fifo, write_json_string, Acceptor, AnnotatedDirection, Annotation, AnnotationError,
    ConnShutdown, Connection, ConsumerReport, DisconnectReason, Fidelity, Format,
    InterfaceDeclaration, InterfaceSpec, JsonLinesWriter, PayloadMode, PcapngWriter, RingSink,
    RingWindow, RotatingFileSink, RotationPolicy, SinkFactory, StreamSink, TcpAcceptor, WriteError,
};
