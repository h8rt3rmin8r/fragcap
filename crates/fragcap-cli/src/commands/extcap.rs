// SPDX-License-Identifier: Apache-2.0

//! `extcap`: expose fragcap to an analyzer as an extcap capture source
//! (specification section 14.5).
//!
//! An analyzer drives this command four ways. Three declaration queries print
//! the extcap control grammar to standard output, from which the analyzer renders
//! a native configuration dialog with no graphical code in fragcap:
//! `--extcap-interfaces` lists the one `fragcap` interface, `--extcap-dlts` its
//! link type, and `--extcap-config` the four configurable options. The fourth,
//! `--capture --fifo <path>`, streams pcapng to the analyzer's FIFO.
//!
//! The capture is the same back half `run` reaches: the options the analyzer
//! passes back (profile, roles, direction, loopback) are overlaid on the profile
//! exactly as the `run` flags are, and the FIFO is one sink over the unchanged
//! pcapng writer, so the stream an analyzer reads is byte-identical to a file
//! capture (constitution P-5). No new capture or attribution technique is
//! introduced (P-1, P-3, P-9); extcap is a front half over the existing pipeline.

use std::io::Write;

use fragcap::profile::resolve;

use crate::assemble;
use crate::cli::ExtcapArgs;
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;
use crate::paths;

/// The single logical extcap interface fragcap presents. fragcap's capture
/// subject is the profile and role selection, not a host adapter, so it is one
/// interface keyed by the configurable options rather than one per adapter.
const INTERFACE: &str = "fragcap";

/// Run `extcap`.
///
/// The declaration queries write their grammar to `out` (command results go to
/// standard output, specification FR-019); a capture streams to the FIFO and its
/// diagnostics go through `emitter` to standard error.
pub fn run(
    args: &ExtcapArgs,
    out: &mut dyn Write,
    emitter: &mut Emitter,
) -> Result<Exit, CliError> {
    if args.extcap_interfaces {
        let _ = write!(out, "{}", interfaces_block());
        Ok(Exit::SUCCESS)
    } else if args.extcap_dlts {
        require_selected_interface(args)?;
        let _ = write!(out, "{}", dlts_block());
        Ok(Exit::SUCCESS)
    } else if args.extcap_config {
        require_selected_interface(args)?;
        let _ = write!(out, "{}", config_block());
        Ok(Exit::SUCCESS)
    } else if args.capture {
        capture(args, emitter)
    } else {
        Err(CliError::usage(
            "extcap needs one of --extcap-interfaces, --extcap-dlts, --extcap-config, or --capture",
        ))
    }
}

/// The `--extcap-interfaces` block: the version line and the one interface.
fn interfaces_block() -> String {
    format!(
        "extcap {{version={}}}{{help={}}}\n\
         interface {{value={}}}{{display=fragcap: process-attributed capture}}\n",
        env!("CARGO_PKG_VERSION"),
        "https://github.com/h8rt3rmin8r/fragcap",
        INTERFACE,
    )
}

/// The `--extcap-dlts` block: the link type for the interface.
///
/// Ethernet (DLT 1) is the top-level default; heterogeneous per-packet link
/// types (a loopback conversation) are carried by the stream's own Interface
/// Description Blocks, which the analyzer reads per packet.
fn dlts_block() -> String {
    "dlt {number=1}{name=EN10MB}{display=Ethernet}\n".to_string()
}

/// The `--extcap-config` block: the four configurable options.
///
/// The `call` names are the `run` flag names, so the capture invocation the
/// analyzer builds is parsed by the same grammar and overlaid the same way
/// (specification 14.5, FR-006).
fn config_block() -> String {
    let mut block = String::new();
    block.push_str(
        "arg {number=0}{call=--profile}{display=Profile}\
         {tooltip=The profile to capture with: a path, a name, or a game id}\
         {type=string}{required=true}\n",
    );
    block.push_str(
        "arg {number=1}{call=--roles}{display=Roles}\
         {tooltip=Comma-separated roles to scope which stages are captured}\
         {type=string}\n",
    );
    block.push_str(
        "arg {number=2}{call=--direction}{display=Direction}\
         {tooltip=The flow direction to scope to}{type=selector}\n",
    );
    block.push_str("value {arg=2}{value=both}{display=Both}{default=true}\n");
    block.push_str("value {arg=2}{value=in}{display=Inbound}\n");
    block.push_str("value {arg=2}{value=out}{display=Outbound}\n");
    block.push_str(
        "arg {number=3}{call=--loopback}{display=Include loopback}\
         {tooltip=Include the loopback adapter}{type=boolflag}\n",
    );
    block
}

/// Require a selected interface naming the one fragcap presents. The dlts and
/// config queries are always paired with `--extcap-interface` by the analyzer; a
/// missing or unknown one is a usage error rather than an inert declaration.
fn require_selected_interface(args: &ExtcapArgs) -> Result<(), CliError> {
    match args.extcap_interface.as_deref() {
        None => Err(CliError::usage(
            "this extcap query needs --extcap-interface <name>",
        )),
        Some(INTERFACE) => Ok(()),
        Some(other) => Err(unknown_interface(other)),
    }
}

/// The usage error for an interface fragcap does not present.
fn unknown_interface(name: &str) -> CliError {
    CliError::usage(format!(
        "`{name}` is not a fragcap extcap interface; the only interface is `{INTERFACE}`"
    ))
}

/// Run an extcap `--capture`, streaming pcapng to the analyzer's FIFO.
fn capture(args: &ExtcapArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    if args.fifo.is_none() {
        return Err(CliError::usage(
            "extcap --capture needs a fifo to stream to; pass --fifo <PATH>",
        ));
    }
    // The analyzer passes --extcap-interface at capture; validate it if present.
    if let Some(name) = args.extcap_interface.as_deref() {
        if name != INTERFACE {
            return Err(unknown_interface(name));
        }
    }
    let Some(profile_ref) = &args.profile else {
        return Err(CliError::usage(
            "extcap --capture needs a profile; the analyzer supplies it from the configuration \
             dialog (--profile)",
        ));
    };

    let search = paths::search_path(&[]);
    let bundled = paths::bundled();
    let resolved = resolve(profile_ref, &search, &bundled)?;

    let config = assemble::effective_config_for_extcap(args, &resolved.profile);
    let components = assemble::components(&args.offline, &config)?;

    orchestrator::install_interrupt_handler();
    let allowed_roles = config.roles.clone();
    orchestrator::capture(
        resolved.profile,
        &config,
        components,
        emitter,
        &orchestrator::INTERRUPT,
        args.offline.fire_interrupt,
        allowed_roles,
    )
}
