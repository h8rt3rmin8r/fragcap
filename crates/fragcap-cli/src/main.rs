// SPDX-License-Identifier: Apache-2.0

//! fragcap command line binary.
//!
//! A shim over [`fragcap_cli::run`]. Every behavior and every test lives in the
//! library, so the whole command surface is driven from `run` without spawning
//! a process; this file only turns the library's [`fragcap_cli::Exit`] into the
//! process exit code.

use std::process::exit;

fn main() {
    exit(fragcap_cli::run(std::env::args_os()).code() as i32);
}
