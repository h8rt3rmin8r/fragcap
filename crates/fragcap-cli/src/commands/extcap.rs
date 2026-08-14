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

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use fragcap::profile::resolve;

use crate::assemble;
use crate::cli::{ExtcapAction, ExtcapArgs};
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
    // The register/unregister subcommands are handled first; when absent, the
    // analyzer protocol dispatch below runs exactly as before.
    if let Some(action) = &args.action {
        return match action {
            ExtcapAction::Install(a) => install(a.dir.as_deref(), out),
            ExtcapAction::Uninstall(a) => uninstall(a.dir.as_deref(), out),
        };
    }
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

/// The extcap directory to register into: the explicit `--dir`, else the
/// per-user platform location (which honors `FRAGCAP_EXTCAP_DIR`). An error when
/// neither can be determined, so a failed registration is never silent (FR-008).
fn resolve_dir(dir_override: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(dir) = dir_override {
        return Ok(dir.to_path_buf());
    }
    paths::extcap_dir().ok_or_else(|| {
        CliError::failure(
            "could not determine the Wireshark extcap directory on this platform; pass --dir",
        )
    })
}

/// Register fragcap as a Wireshark extcap source: copy the running binary into
/// the extcap directory under the name `doctor` probes for. Idempotent: it
/// overwrites any existing registration so the registered copy matches the
/// running fragcap.
fn install(dir_override: Option<&Path>, out: &mut dyn Write) -> Result<Exit, CliError> {
    let dir = resolve_dir(dir_override)?;
    let source = std::env::current_exe().map_err(|e| {
        CliError::failure(format!(
            "could not determine the running fragcap binary: {e}"
        ))
    })?;
    fs::create_dir_all(&dir).map_err(|e| {
        CliError::failure(format!(
            "could not create the extcap directory {}: {e}",
            dir.display()
        ))
    })?;
    let dest = dir.join(paths::EXTCAP_BINARY);
    // If this invocation IS the already-registered copy, source and dest are the
    // same file. Copying a file onto itself fails on Windows (the running image
    // is locked) and can truncate it on Unix while still returning success, so
    // treat it as an unchanged success rather than rewriting it. This is the
    // idempotent case that the running binary already satisfies.
    if is_same_existing_file(&source, &dest) {
        let _ = writeln!(
            out,
            "fragcap is already registered as a Wireshark extcap source: {}",
            dest.display()
        );
        return Ok(Exit::SUCCESS);
    }
    fs::copy(&source, &dest)
        .map_err(|e| CliError::failure(format!("could not write {}: {e}", dest.display())))?;
    let _ = writeln!(
        out,
        "registered fragcap as a Wireshark extcap source: {}",
        dest.display()
    );
    Ok(Exit::SUCCESS)
}

/// Whether two paths resolve to the same existing file. `false` when either
/// cannot be canonicalized, which includes a destination that does not exist yet
/// (the first-install case), so the caller then proceeds with the copy.
fn is_same_existing_file(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Remove fragcap's Wireshark extcap registration. Idempotent: removing an
/// absent registration is a success reported as a no-op.
fn uninstall(dir_override: Option<&Path>, out: &mut dyn Write) -> Result<Exit, CliError> {
    let dir = resolve_dir(dir_override)?;
    let dest = dir.join(paths::EXTCAP_BINARY);
    // `try_exists` distinguishes "absent" from "could not be checked": a
    // registered binary that is present but inaccessible (ACLs, an I/O error)
    // must not be reported as a clean no-op, which `Path::exists` would do by
    // collapsing the error to `false`.
    let present = dest
        .try_exists()
        .map_err(|e| CliError::failure(format!("could not check {}: {e}", dest.display())))?;
    if present {
        fs::remove_file(&dest)
            .map_err(|e| CliError::failure(format!("could not remove {}: {e}", dest.display())))?;
        let _ = writeln!(
            out,
            "removed the fragcap Wireshark extcap registration: {}",
            dest.display()
        );
    } else {
        let _ = writeln!(
            out,
            "fragcap is not registered as a Wireshark extcap source ({}); nothing to do",
            dest.display()
        );
    }
    Ok(Exit::SUCCESS)
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
        // The analyzer closing its FIFO is the defined clean stop for an extcap
        // capture, so a sink failure ending the run is a success (the summary
        // still carries the loss accounting). The FIFO is opened at assembly, so
        // a mid-capture failure is a consumer disconnect, not a bad path.
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::is_same_existing_file;
    use std::path::Path;

    #[test]
    fn same_existing_file_is_detected_and_distinct_files_are_not() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        std::fs::write(&a, b"x").unwrap();
        std::fs::write(&b, b"x").unwrap();

        // The same path (the self-copy case) is detected as the same file.
        assert!(is_same_existing_file(&a, &a));
        // Two distinct files are not, so a genuine refresh still copies.
        assert!(!is_same_existing_file(&a, &b));
        // A destination that does not exist yet is not "the same file", so the
        // first install proceeds with the copy.
        assert!(!is_same_existing_file(&a, &dir.path().join("missing")));
        assert!(!is_same_existing_file(
            Path::new("nope-a"),
            Path::new("nope-b")
        ));
    }
}
