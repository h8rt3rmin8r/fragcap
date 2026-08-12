// SPDX-License-Identifier: Apache-2.0

//! The clap argument grammar for the command surface of specification section
//! 17.
//!
//! The types are derived, so the flags, defaults, subcommands, version, and
//! help text all come from the structure rather than from a hand-maintained
//! parser that could drift from the help the specification prints. The whole
//! seven-command surface is declared here so `--help` foreshadows the tool even
//! where a command is a stub.
//!
//! A handful of flags on `run` and `tap` are hidden. They supply the offline
//! substrate, a recorded capture replayed as the source, a scripted attributor,
//! and a scripted process timeline, so the whole capture path is driven from
//! `run()` in a tier-1 test with no capture driver, no elevation, and no game.
//! They are hidden rather than removed because the same assembly seam serves the
//! feature-gated live path, and a test needs to reach it.

use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::args::{parse_duration, parse_ring, parse_size, Direction, RingWindow, SinkSpec};

/// Passive, process-attributed network capture for Windows.
#[derive(Debug, Parser)]
#[command(name = "fragcap", version, about, long_about = None)]
pub struct Cli {
    /// Suppress progress; keep warnings and errors.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Suppress everything except errors.
    #[arg(long, global = true)]
    pub silent: bool,

    /// Emit machine-readable structured output instead of human text. `run`,
    /// `tap`, `steam`, and `extcap` emit the newline-delimited capture event
    /// stream on standard error; `profile` and `doctor` emit their results as
    /// newline-delimited records on standard output.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// The seven commands of the tool. Four are implemented; three are stubs that
/// name the slice that will deliver them.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Capture a game using a profile.
    Run(Box<RunArgs>),
    /// Capture a running process ad hoc, without an authored profile.
    Tap(TapArgs),
    /// Run a capture file back (not yet implemented).
    Replay(StubArgs),
    /// Manage and validate profiles.
    Profile(ProfileArgs),
    /// Enumerate titles and scaffold profiles from a Steam installation.
    Steam(SteamArgs),
    /// Report environment readiness.
    Doctor(DoctorArgs),
    /// Analyzer integration: enumerate, configure, and capture as an extcap
    /// source (specification section 14.5).
    Extcap(Box<ExtcapArgs>),
}

/// The capture mode, specification section 17.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ModeArg {
    /// Bounded capture written to a file (implemented).
    File,
    /// Live streaming to a transport (not yet implemented).
    Stream,
    /// Rolling in-memory window (not yet implemented).
    Ring,
}

/// Arguments to `run`.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// The profile to capture with: a path, a name, or a game id.
    #[arg(short = 'p', long)]
    pub profile: String,

    /// The output capture file (pcapng). Shorthand for a file sink.
    #[arg(short = 'o', long)]
    pub out: Option<PathBuf>,

    /// The capture mode.
    #[arg(long, value_enum)]
    pub mode: Option<ModeArg>,

    /// An output sink, repeatable. `file:PATH`, `jsonl:PATH`, `pipe:NAME`, or
    /// `tcp://HOST:PORT`.
    #[arg(long, value_parser = crate::args::parse_sink)]
    pub sink: Vec<SinkSpec>,

    /// The capture duration bound, from arm.
    #[arg(short = 'd', long, value_parser = parse_duration)]
    pub duration: Option<Duration>,

    /// How long to wait for a target before giving up (acquisition timeout).
    #[arg(long, value_parser = parse_duration)]
    pub wait: Option<Duration>,

    /// Stop after this many captured packets.
    #[arg(long)]
    pub max_packets: Option<u64>,

    /// Stop after this many captured bytes (integer plus b/kb/mb/gb).
    #[arg(long, value_parser = parse_size)]
    pub max_bytes: Option<u64>,

    /// The roles to capture, comma-separated. Scopes which stages trigger.
    // `value_delimiter` splits one comma-separated value into the role list,
    // matching the `extcap` surface so the command line and the analyzer dialog
    // select capture identically. A custom `value_parser` returning `Vec<String>`
    // cannot be used here: clap derives the element type from the `Vec<String>`
    // field and panics at access time on the type mismatch. This note stays a
    // source comment; it is not user-facing help.
    #[arg(long, value_delimiter = ',')]
    pub roles: Option<Vec<String>>,

    /// The flow direction to scope to.
    #[arg(long, value_enum)]
    pub direction: Option<Direction>,

    /// A capture interface, repeatable.
    #[arg(short = 'i', long)]
    pub interface: Vec<String>,

    /// Include the loopback adapter.
    #[arg(long)]
    pub loopback: bool,

    /// Write metadata only, no packet payloads.
    #[arg(long)]
    pub no_payload: bool,

    /// The ring window (not yet implemented).
    #[arg(long, value_parser = parse_ring)]
    pub ring: Option<RingWindow>,

    /// Launch the game through its platform launcher before capturing, then
    /// capture it (Windows only; requires a Steam app id in the profile).
    #[arg(long)]
    pub launch: bool,

    #[command(flatten)]
    pub offline: OfflineArgs,
}

/// Arguments to `tap`.
#[derive(Debug, Args)]
pub struct TapArgs {
    /// The image name of the process to capture.
    #[arg(short = 'p', long)]
    pub process: String,

    /// The capture duration bound.
    #[arg(short = 'd', long, value_parser = parse_duration)]
    pub duration: Option<Duration>,

    /// The output capture file (pcapng).
    #[arg(short = 'o', long)]
    pub out: Option<PathBuf>,

    /// An output sink, repeatable.
    #[arg(long, value_parser = crate::args::parse_sink)]
    pub sink: Vec<SinkSpec>,

    /// Write metadata only, no packet payloads.
    #[arg(long)]
    pub no_payload: bool,

    #[command(flatten)]
    pub offline: OfflineArgs,
}

/// The hidden offline substrate flags shared by `run` and `tap`.
///
/// Present so the capture path is driven from `run()` in a tier-1 test with no
/// capture driver. Hidden from help; the live, socket-table, and ETW paths are
/// what an operator reaches.
#[derive(Debug, Args, Default)]
pub struct OfflineArgs {
    /// Replay this capture file as the packet source instead of an interface.
    #[arg(long, hide = true)]
    pub replay_source: Option<PathBuf>,

    /// Resolve attribution from this script instead of the socket table.
    #[arg(long, hide = true)]
    pub attr_script: Option<PathBuf>,

    /// Drive process events from this script instead of Event Tracing.
    #[arg(long, hide = true)]
    pub process_script: Option<PathBuf>,

    /// The capturing interface's local addresses, for direction and the local
    /// endpoint. Repeatable.
    #[arg(long, hide = true)]
    pub local_addr: Vec<IpAddr>,

    /// Fire an operator interrupt at the end of the capture, so the interrupt
    /// path is exercised without a real signal.
    #[arg(long, hide = true)]
    pub fire_interrupt: bool,
}

/// Arguments to `profile`.
#[derive(Debug, Args)]
pub struct ProfileArgs {
    /// A profile directory to search, repeatable.
    #[arg(long = "profile-dir", global = true)]
    pub profile_dir: Vec<PathBuf>,

    #[command(subcommand)]
    pub command: ProfileCommand,
}

/// The `profile` subcommands.
#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    /// Validate a profile, reporting every diagnostic in one pass.
    Validate {
        /// The profile reference.
        reference: String,
    },
    /// List the bundled and user profiles with counts.
    List,
    /// Show how a reference resolves and which source supplied it.
    Show {
        /// The profile reference.
        reference: String,
    },
}

/// Arguments to `steam`.
#[derive(Debug, Args)]
pub struct SteamArgs {
    #[command(subcommand)]
    pub command: SteamCommand,
}

/// The `steam` subcommands.
#[derive(Debug, Subcommand)]
pub enum SteamCommand {
    /// Scaffold a profile skeleton for an installed title.
    Profile {
        /// The Steam application identifier of an installed title.
        app_id: String,
    },
}

/// Arguments to `doctor`.
#[derive(Debug, Args)]
pub struct DoctorArgs {}

/// Arguments to `extcap` (specification section 14.5).
///
/// The extcap protocol drives this command four ways: three declaration queries
/// that print the extcap control grammar to standard output, and a capture that
/// streams pcapng to the analyzer's FIFO. The configurable options are declared
/// by `--extcap-config` and passed back at capture under the same names the `run`
/// command uses (`--profile`, `--roles`, `--direction`, `--loopback`), so the
/// analyzer's native dialog and the command line select capture identically.
///
/// The hidden offline flags are flattened in for the same reason `run` carries
/// them: the whole capture path is driven from a tier-1 test with no capture
/// driver and no analyzer.
#[derive(Debug, Args)]
pub struct ExtcapArgs {
    /// Print the available extcap interfaces and exit.
    #[arg(long)]
    pub extcap_interfaces: bool,

    /// Print the link types for the selected interface and exit.
    #[arg(long)]
    pub extcap_dlts: bool,

    /// Print the configurable options for the selected interface and exit.
    #[arg(long)]
    pub extcap_config: bool,

    /// Start a capture, streaming pcapng to the `--fifo` path.
    #[arg(long)]
    pub capture: bool,

    /// The analyzer FIFO or named-pipe path to stream the capture to.
    #[arg(long)]
    pub fifo: Option<PathBuf>,

    /// The selected extcap interface.
    #[arg(long)]
    pub extcap_interface: Option<String>,

    /// The analyzer protocol version query. Accepted; not acted on.
    #[arg(long)]
    pub extcap_version: Option<String>,

    /// Config option: the profile to capture with.
    #[arg(long)]
    pub profile: Option<String>,

    /// Config option: the roles to scope to, comma-separated.
    // The analyzer sends this as one comma-separated value; `value_delimiter`
    // splits it into the role list the overlay expects. Source comment, not
    // user-facing help.
    #[arg(long, value_delimiter = ',')]
    pub roles: Option<Vec<String>>,

    /// Config option: the flow direction to scope to.
    #[arg(long, value_enum)]
    pub direction: Option<Direction>,

    /// Config option: include the loopback adapter.
    #[arg(long)]
    pub loopback: bool,

    #[command(flatten)]
    pub offline: OfflineArgs,
}

/// The catch-all arguments a stub command accepts, so `fragcap replay anything`
/// parses and the stub reports honestly rather than the parser rejecting it.
#[derive(Debug, Args)]
pub struct StubArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    pub rest: Vec<String>,
}
