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

use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

use crate::args::{parse_duration, parse_ring, parse_size, Direction, RingWindow, SinkSpec};

/// Passive, process-attributed network capture for Windows.
// The four headings are specification section 17.3, purely presentational and
// hiding nothing. clap 4.5.32 (pinned for MSRV) does not group subcommands
// natively, so the grouping is rendered through the custom help template below;
// every command still appears exactly once. Kept as a `//` comment: clap
// publishes `///` verbatim as user-facing help, and a specification section
// number is actionable only to a reader who has the specification open.
#[derive(Debug, Parser)]
#[command(
    name = "fragcap",
    max_term_width = 100,
    version,
    about,
    long_about = None,
    // The `Commands:` block below is a literal, not clap-rendered rows, so
    // `wrap_help` does not reach it and it does not wrap at any width. Its lines
    // are hand-budgeted to 76 columns; keep a new entry inside that or the root
    // page overflows again. `cli_help.rs` asserts the limit, so an over-long
    // entry fails there rather than shipping.
    help_template = "\
{about-with-newline}
{usage-heading} {usage}

Commands:
  Capture:
    capture       Capture a target's traffic (--target or --process)
    replay        Run a capture file back (not yet implemented)

  Targets:
    targets       Manage capture targets in the local store (local.db)
    technologies  Detect engines, anti-cheat, and DRM in an install directory
    steam         Enumerate installed Steam titles

  Environment:
    doctor        Report environment readiness
    extcap        Wireshark extcap integration

  Data:
    catalog       Maintain the shipped catalog store (catalog.db)
    schema        Validate JSON artifacts against the master schema

Options:
{options}"
)]
pub struct Cli {
    /// Suppress progress; keep warnings and errors.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Suppress everything except errors.
    #[arg(long, global = true)]
    pub silent: bool,

    /// Emit machine-readable output instead of human text.
    ///
    /// `capture`, `steam`, and `extcap` emit the newline-delimited capture event
    /// stream on standard error; `doctor` emits its results as newline-delimited
    /// records on standard output.
    #[arg(long, global = true)]
    pub json: bool,

    /// The command to run. With none, `fragcap` lists registered targets and
    /// points at `--help`.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// The command surface of the tool. Most commands are implemented; a couple
/// remain stubs, and their help says so.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Capture a target's traffic.
    ///
    /// Identify the target by a stored-target selector (positionally or with
    /// `--target`), by `--id`, or by a raw process image name (`--process`);
    /// every other option composes freely.
    Capture(Box<CaptureArgs>),
    /// Run a capture file back (not yet implemented).
    Replay(StubArgs),
    /// Register, list, show, and discover capture targets in the user store
    /// (local.db).
    Targets(TargetsArgs),
    /// Detect the technologies present in a game's install directory (engine,
    /// anti-cheat, and DRM), from a signature table.
    Technologies(TechnologiesArgs),
    /// Enumerate installed titles from a Steam installation.
    Steam(SteamArgs),
    /// Report environment readiness.
    Doctor(DoctorArgs),
    /// Analyzer integration: enumerate, configure, and capture as an extcap
    /// source.
    Extcap(Box<ExtcapArgs>),
    /// Maintain the catalog store (catalog.db): import, export, and seed.
    Catalog(CatalogArgs),
    /// Validate JSON artifacts against the master schema, or print it.
    Schema(SchemaArgs),
}

/// The capture mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ModeArg {
    /// Bounded capture written to a file (implemented).
    File,
    /// Live streaming to a transport (not yet implemented).
    Stream,
    /// Rolling in-memory window (not yet implemented).
    Ring,
}

/// Arguments to `capture`, the single capture verb.
///
/// Exactly one target input is required and they are mutually exclusive: a
/// positional selector or its `--target` equivalent names a stored target by
/// selector resolved against the local store, `--id` names a
/// stored target by its durable identifier, and `--process` names a raw process
/// image name directly. The positional form is the ergonomic one the `fragcap
/// targets` listing hints (`fragcap capture <n>`); `--target` is its explicit-flag
/// equivalent. The clap group enforces the exactly-one rule, so a usage error
/// (exit 2) is reported before any resolution. Every other flag is orthogonal to
/// the target input, so all five section-9.1 captures are expressible.
///
/// The optional path anchors (`--path`, `--path-regex`) disambiguate two
/// processes sharing an image name (the capability the retired `watch` carried).
/// `--launch` requires the resolved `--target` to carry a launchable platform
/// anchor; a `--process` capture, or a target with no anchor, cannot be launched
/// and is refused (exit 2) rather than ignored.
#[derive(Debug, Args)]
#[command(group(ArgGroup::new("target_input").required(true).args(["selector", "target", "id", "process"])))]
pub struct CaptureArgs {
    /// A stored target: an exact handle, an exact name, or a listing row number.
    ///
    /// A bare integer here is always a row index over the current `fragcap
    /// targets` listing, never a handle, a name, or a platform app id. This is
    /// the ergonomic form the listing names in its `fragcap capture <n>` hint;
    /// it is equivalent to `--target` and mutually exclusive with it.
    #[arg(value_name = "SELECTOR")]
    pub selector: Option<String>,

    /// A stored target: an exact handle, an exact name, or a listing row number.
    ///
    /// A bare integer here is always a row index over the current `fragcap
    /// targets` listing, never a handle, a name, or a platform app id. To
    /// capture a Steam title by its app id, register it first with
    /// `targets add --steam <app_id>`. The explicit-flag form of the positional
    /// selector.
    #[arg(long)]
    pub target: Option<String>,

    /// Select a stored target by its durable stable identifier, the machine-facing
    /// form automation addresses (a bare integer given to `--target` is a row index,
    /// not this). Mutually exclusive with `--target` and `--process`.
    #[arg(long)]
    pub id: Option<i64>,

    /// The image name of the process to capture, named directly with no stored
    /// target.
    #[arg(long)]
    pub process: Option<String>,

    /// A case-insensitive substring the target's full image path must contain.
    #[arg(long)]
    pub path: Option<String>,

    /// A regular expression the target's full image path must match.
    #[arg(long)]
    pub path_regex: Option<String>,

    /// The shipped catalog store consulted while resolving a `--target`. Overrides
    /// `FRAGCAP_CATALOG_DB` and the default location; a store named here that does
    /// not exist is not created and is not an error. With neither this flag nor the
    /// environment variable set, `capture` defaults to
    /// `%APPDATA%\fragcap\catalog.db` and creates it on first use (seeding it from
    /// the store shipped beside the executable when present).
    #[arg(long)]
    pub catalog_db: Option<PathBuf>,

    /// The local store, where registered targets and learned launch data live. It
    /// is consulted before the catalog during resolution and is where `--target`
    /// selectors resolve. Overrides `FRAGCAP_LOCAL_DB` and the default location; a
    /// store named here that does not exist is not created and is not an error.
    /// With neither this flag nor the environment variable set, `capture` defaults
    /// to `%APPDATA%\fragcap\local.db` and creates it empty on first use.
    #[arg(long)]
    pub local_db: Option<PathBuf>,

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

    /// Start the target through its platform launcher, then capture it.
    ///
    /// Windows only. The target must be Steam anchored; register one with
    /// `targets add --steam <app_id>`. This describes the stored target, not the
    /// value given to `--target`, which never accepts a platform app id.
    #[arg(long)]
    pub launch: bool,

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

/// Arguments to `steam`.
#[derive(Debug, Args)]
pub struct SteamArgs {
    #[command(subcommand)]
    pub command: SteamCommand,
}

/// The `steam` subcommands. Steam-specific inspection only; registering a title
/// as a capture target is `targets add --steam <app_id>`.
#[derive(Debug, Subcommand)]
pub enum SteamCommand {
    /// List the installed Steam titles this machine can enumerate.
    List,
}

/// Arguments to `schema`.
#[derive(Debug, Args)]
pub struct SchemaArgs {
    #[command(subcommand)]
    pub command: SchemaCommand,
}

/// The `schema` subcommands.
#[derive(Debug, Subcommand)]
pub enum SchemaCommand {
    /// Validate a JSON file against the master schema, reporting every
    /// structural violation in one pass.
    Validate {
        /// The JSON file to validate.
        file: PathBuf,
    },
    /// Print the embedded master schema to standard output.
    Print,
}

/// Arguments to `technologies`.
#[derive(Debug, Args)]
pub struct TechnologiesArgs {
    /// The install directory to scan for technologies.
    #[arg(short = 'p', long)]
    pub path: PathBuf,
    /// The catalog store (`catalog.db`) whose signature table drives detection.
    ///
    /// Seed it with `catalog seed-signatures`.
    #[arg(long)]
    pub catalog_db: Option<PathBuf>,
}

/// Arguments to `targets`.
#[derive(Debug, Args)]
pub struct TargetsArgs {
    /// The `targets` subcommand. With none, `fragcap targets` lists registered
    /// targets from the default local store.
    #[command(subcommand)]
    pub command: Option<TargetsCommand>,
}

/// The `targets` subcommands. Every one operates on the user-owned local store
/// (`local.db`): registering, listing, showing, and discovering capture targets.
/// Operations that write the shipped catalog store live under `catalog`.
#[derive(Debug, Subcommand)]
pub enum TargetsCommand {
    /// Register a target from a name, deriving a unique handle.
    Add(TargetsAddArgs),
    /// List capture targets by row number, handle, readiness, and known
    /// technologies.
    ///
    /// Discovers and registers any newly installed titles first, so this writes
    /// to the store as well as reading it, and it rewrites the row numbering
    /// that `fragcap capture <n>` resolves against. The durable identifier
    /// `--id` consumes is not a column here; `targets show` and `targets export`
    /// carry it.
    List {
        /// The store file (local.db) to read and register into. Defaults to the
        /// local store the bare `fragcap targets` command uses (the
        /// `FRAGCAP_LOCAL_DB` override, else the per-user default) when omitted.
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Show one target resolved by a selector (handle, name, row index, or `--id`).
    Show(TargetsShowArgs),
    /// Discover installed games: walk Steam and the known game-install roots.
    ///
    /// Reads only; a candidate becomes a stored target when acted on
    /// (`targets add`).
    Discover(TargetsDiscoverArgs),
    /// Scan one directory and list it as a single candidate.
    ///
    /// Backs pointing discovery straight at a known game folder. With
    /// `--catalog-db`, detects the engine, anti-cheat, and DRM technologies in
    /// the directory and carries them as evidence.
    Scan {
        /// The directory to treat as a single game location.
        dir: PathBuf,
        /// An optional catalog store (`catalog.db`) whose signature table labels the
        /// technologies in the directory. Without it the candidate carries no
        /// detected evidence.
        #[arg(long)]
        catalog_db: Option<PathBuf>,
        /// The local store (local.db) to register the discovered titles into. Without
        /// it the scan lists what it finds but registers nothing.
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Remove one registered target resolved by a selector.
    ///
    /// The selector is a handle, a name, a row number, or `--id`. An ambiguous
    /// name lists its matches and refuses.
    Remove(TargetsShowArgs),
    /// Export registered targets as a JSON array to standard output.
    ///
    /// All targets, or the one a selector resolves.
    Export(TargetsExportArgs),
    /// Import a target-entry JSON array, merging on the stable identifier.
    ///
    /// A nonconforming file is rejected whole.
    Import {
        /// The JSON file to import.
        file: PathBuf,
        /// The store file (local.db) to merge into, created if absent. Defaults to the
        /// local store the bare `fragcap targets` command uses (the `FRAGCAP_LOCAL_DB`
        /// override, else the per-user default) when omitted.
        #[arg(long)]
        db: Option<PathBuf>,
    },
}

/// Arguments to `targets discover`.
#[derive(Debug, Args)]
pub struct TargetsDiscoverArgs {
    /// The catalog store (`catalog.db`) whose appids classify Steam titles.
    #[arg(long)]
    pub catalog_db: Option<PathBuf>,
    /// The user store (`local.db`) holding the volume eligibility allowlist.
    #[arg(long)]
    pub local_db: Option<PathBuf>,
    /// The Steam installation root. Without it, discovery locates Steam itself and
    /// skips the Steam tier if none is installed.
    #[arg(long)]
    pub steam_root: Option<PathBuf>,
}

/// Arguments to `targets add`.
///
/// `--steam <app_id>` registers an installed Steam title as a target: it resolves
/// the title through the local Steam installation, uses its name, and anchors it to
/// `steam:<app_id>` (replacing the retired `steam profile <app_id>`). It is
/// mutually exclusive with an explicit `--anchor`, and it supplies the name when the
/// positional name is omitted.
#[derive(Debug, Args)]
#[command(group(ArgGroup::new("steam_anchor").args(["steam", "anchor"])))]
pub struct TargetsAddArgs {
    /// The display name to register; its handle is derived automatically. Optional
    /// when `--steam` supplies the name from the installed title.
    pub name: Option<String>,
    /// The store file (local.db) to write, created if absent. Defaults to the local
    /// store the bare `fragcap targets` command uses (the `FRAGCAP_LOCAL_DB` override,
    /// else the per-user default) when omitted.
    #[arg(long)]
    pub db: Option<PathBuf>,
    /// A platform anchor (for example `steam:620`) giving the target a stable,
    /// deterministic identity. Without one the target gets a random identity.
    #[arg(long)]
    pub anchor: Option<String>,
    /// Register an installed Steam title by app id: resolve its name and install
    /// directory, anchor it to `steam:<app_id>`. Mutually exclusive with `--anchor`.
    #[arg(long)]
    pub steam: Option<String>,
    /// A launch executable name, so the target names something capturable.
    #[arg(long)]
    pub exe: Option<String>,
    /// Override the derived handle, subject to the same validity rules
    /// (unique, not purely numeric, normalized shape).
    #[arg(long = "handle")]
    pub handle_override: Option<String>,
    /// Whether the `--exe` executable is the process that holds the sockets:
    /// `yes` records it as the resolved client, `no` records it as a launcher with
    /// the holder unresolved, `unsure` records it observed with no holder claim. The
    /// non-interactive form of the socket-holder question; `unsure` and `no` leave
    /// the chain for a capture to resolve. Requires `--exe`.
    #[arg(long = "socket-holder", value_name = "yes|no|unsure")]
    pub socket_holder: Option<String>,
}

/// Arguments to `catalog`.
#[derive(Debug, Args)]
pub struct CatalogArgs {
    #[command(subcommand)]
    pub command: CatalogCommand,
}

/// The `catalog` subcommands. Every one operates on the shipped, disposable
/// catalog store (`catalog.db`): the maintainer seeds it, and any user refreshes
/// it. `seed` and `seed-engine` read a local document unless a live source flag
/// selects the network, which only a build that can reach the network offers.
#[derive(Debug, Subcommand)]
pub enum CatalogCommand {
    /// Load a local JSON seed document into the catalog store, creating it if needed.
    Import {
        /// The seed document (a `kind: "export"` JSON file).
        seed: PathBuf,
        /// The store file to write.
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Export the catalog store to schema-conformant JSON on standard output.
    Export {
        /// The store file to read.
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Fill the catalog store, one tier or all of them.
    ///
    /// With no `--tier`, fills every tier that has a source available without
    /// one being named, and reports every tier it skipped with the reason.
    Seed(TargetsSeedArgs),
}

/// Arguments to `targets show`. Exactly one of a positional selector or `--id`
/// is required; the clap group enforces it (a usage error, exit 2, otherwise).
#[derive(Debug, Args)]
#[command(group(ArgGroup::new("target_selector").required(true).args(["selector", "id"])))]
pub struct TargetsShowArgs {
    /// A selector: an exact handle, a case-insensitive exact name, or a 1-based
    /// ephemeral row index over the current listing.
    pub selector: Option<String>,
    /// Select by stable identifier: the durable, machine-facing form.
    #[arg(long)]
    pub id: Option<i64>,
    /// The store file (local.db) to read. Defaults to the local store the bare
    /// `fragcap targets` command uses (the `FRAGCAP_LOCAL_DB` override, else the
    /// per-user default) when omitted.
    #[arg(long)]
    pub db: Option<PathBuf>,
}

/// Arguments to `targets export`. The selector is optional: with none, every
/// registered target is exported; with one, only the target it resolves.
#[derive(Debug, Args)]
pub struct TargetsExportArgs {
    /// A selector: an exact handle, a case-insensitive exact name, or a 1-based row
    /// index. Omit to export all registered targets.
    pub selector: Option<String>,
    /// Select by stable identifier: the durable, machine-facing form.
    #[arg(long)]
    pub id: Option<i64>,
    /// The store file (local.db) to read. Defaults to the local store the bare
    /// `fragcap targets` command uses (the `FRAGCAP_LOCAL_DB` override, else the
    /// per-user default) when omitted.
    #[arg(long)]
    pub db: Option<PathBuf>,
}

/// The part of the catalog store a seed run fills.
///
/// One flag instead of one verb per tier. The three verbs this replaces
/// (`seed`, `seed-engine`, `seed-signatures`) were accretion: each arrived with
/// the slice that needed it and none was defended against the others, while the
/// scheme's own next step was a fourth top-level verb for the launch tier, which
/// `SeedTier::Launch` already names and no command reaches (issue #180).
///
/// `Signature` is not a `SeedTier`: the signature table is a separate table with
/// no source, no cursor, and no resume. It is a tier here because to a user it
/// is one more part of the same store, and hiding that distinction behind a
/// separate verb is what made `seed-signatures` read as a step they were
/// expected to perform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SeedTierArg {
    /// Titles: app id, name, and the popularity metrics the corpus gate reads.
    Catalog,
    /// Launch metadata: the launch array, launcher mediation, token requirement.
    Launch,
    /// Engine attribution: engine name, source, confidence.
    Engine,
    /// The detection signature table, from the bundled document. Always offline.
    Signature,
}

/// Arguments to `catalog seed`.
///
/// At most one source may be named. `--from` reads a local document and requires
/// exactly one `--tier`, because the offline documents are bare JSON arrays with
/// no discriminator: sniffing one to guess its tier would silently write the
/// wrong columns, so an ambiguous invocation is a usage error instead (P-9).
#[derive(Debug, Args)]
#[cfg_attr(
    feature = "net",
    command(group(ArgGroup::new("seed_source").args(["from", "steam", "pcgamingwiki"])))
)]
#[cfg_attr(
    not(feature = "net"),
    command(group(ArgGroup::new("seed_source").args(["from"])))
)]
pub struct TargetsSeedArgs {
    /// The tier to fill, repeatable. Omit to fill every tier with a source.
    #[arg(long, value_enum)]
    pub tier: Vec<SeedTierArg>,
    /// Seed from a local document (offline). Requires exactly one `--tier`.
    #[arg(long)]
    pub from: Option<PathBuf>,
    /// Seed the title tier from the live Steam catalog over the network.
    ///
    /// Only present in a build that can reach the network.
    #[cfg(feature = "net")]
    #[arg(long)]
    pub steam: bool,
    /// Seed the engine tier from the live PCGamingWiki query API.
    ///
    /// Only present in a build that can reach the network. The flag names its
    /// actual source rather than `--steam`: the tier is keyed by Steam
    /// application id but the data is PCGamingWiki's.
    #[cfg(feature = "net")]
    #[arg(long)]
    pub pcgamingwiki: bool,
    /// The store file to seed, created if absent. Defaults to the per-user store.
    #[arg(long)]
    pub db: Option<PathBuf>,
    /// The review-count corpus threshold; a title needs at least this many reviews.
    #[arg(long, default_value_t = fragcap::targets::DEFAULT_MIN_REVIEWS)]
    pub min_reviews: u64,
}

/// Arguments to `doctor`.
///
/// `doctor` is read-only by default. `--fix` adds an interactive action layer above
/// the pure classifier: it runs the same checks, prints the same report, then offers
/// to perform the remediations the report named, one at a time, under confirmation.
/// It acts only on remediations `doctor` already printed. `--fix` is refused with
/// `--json` and when stdout (or, without `--yes`, stdin) is not an interactive
/// terminal; `--yes` pre-confirms every offered action for unattended interactive
/// use. `--yes` has no meaning without `--fix`.
#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Offer to perform the remediations the report names, under confirmation.
    #[arg(long)]
    pub fix: bool,

    /// With `--fix`, pre-confirm every offered action (unattended). Still requires an
    /// interactive stdout.
    #[arg(long)]
    pub yes: bool,
}

/// Arguments to `extcap`.
///
/// The extcap protocol drives this command four ways: three declaration queries
/// that print the extcap control grammar to standard output, and a capture that
/// streams pcapng to the analyzer's FIFO. The configurable options are declared
/// by `--extcap-config` and passed back at capture under the same names the
/// `capture` command uses (`--target`, `--roles`, `--direction`, `--loopback`), so
/// the analyzer's native dialog and the command line select a stored target
/// identically.
///
/// The hidden offline flags are flattened in for the same reason `capture` carries
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

    /// Config option: the stored target to capture, named by a selector (an exact
    /// handle, a case-insensitive exact name, or a 1-based row index over the current
    /// listing), resolved against the local store the same way `capture --target`
    /// resolves it.
    #[arg(long)]
    pub target: Option<String>,

    /// The catalog store consulted while resolving a Steam-anchored target. Defaults
    /// like `capture`'s when omitted.
    #[arg(long)]
    pub catalog_db: Option<PathBuf>,

    /// The local store the target selector resolves against. Defaults like
    /// `capture`'s when omitted.
    #[arg(long)]
    pub local_db: Option<PathBuf>,

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

    /// Register or unregister fragcap as a Wireshark extcap source. Absent for
    /// the analyzer protocol invocations above, which are driven by the flags.
    #[command(subcommand)]
    pub action: Option<ExtcapAction>,
}

/// The optional `extcap` subcommands that register or unregister fragcap as a
/// Wireshark extcap capture source, distinct from the analyzer protocol flags on
/// [`ExtcapArgs`]. The barewords `install` and `uninstall` cannot collide with
/// those flags (they all start with `--`) or with the top-level extcap routing.
#[derive(Debug, Subcommand)]
pub enum ExtcapAction {
    /// Register fragcap as a Wireshark extcap capture source.
    Install(ExtcapInstallArgs),
    /// Remove fragcap's Wireshark extcap registration.
    Uninstall(ExtcapInstallArgs),
}

/// Options shared by `extcap install` and `extcap uninstall`. The scope selects
/// which Wireshark extcap directory to act on; the three selectors form a
/// mutually exclusive group (at most one), and the default when none is given is
/// the per-user scope, the same location `doctor` probes for the current user.
#[derive(Args, Debug)]
pub struct ExtcapInstallArgs {
    /// Use the per-user Wireshark extcap directory (the default).
    #[arg(long, group = "extcap_scope")]
    pub user: bool,

    /// Use the machine-wide Wireshark extcap directory, so every user on the
    /// machine sees the integration (needs the privilege to write it).
    #[arg(long, group = "extcap_scope")]
    pub system: bool,

    /// An explicit extcap directory, overriding the scope flags. Point this at the
    /// system directory to register machine-wide without `--system`.
    #[arg(long, group = "extcap_scope")]
    pub dir: Option<PathBuf>,
}

/// The catch-all arguments a stub command accepts, so `fragcap replay anything`
/// parses and the stub reports honestly rather than the parser rejecting it.
#[derive(Debug, Args)]
pub struct StubArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    pub rest: Vec<String>,
}
