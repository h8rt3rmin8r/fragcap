// SPDX-License-Identifier: Apache-2.0

//! Exact desktop shell token acquisition, with query-limited process authority.

use super::*;
use windows_sys::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetShellWindow, GetWindowThreadProcessId};

fn owner(window: isize) -> io::Result<u32> {
    let mut pid = 0;
    if window == 0 {
        return Err(bad("desktop shell window is unavailable"));
    }
    if unsafe { GetWindowThreadProcessId(window, &mut pid) } == 0 {
        return Err(error());
    }
    if pid == 0 {
        return Err(bad("desktop shell window owner is unavailable"));
    }
    Ok(pid)
}

pub(super) fn ordinary_token(producer: &str, session: u32) -> io::Result<Handle> {
    let window = unsafe { GetShellWindow() };
    let pid = staged("desktop shell owner query", owner(window))?;
    // The only opened process is the exact desktop shell. These literal rights
    // permit token queries only; no memory, creation or termination rights.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process == 0 {
        return staged("desktop shell process query", Err(error()));
    }
    let process = Handle(process);
    let mut token = 0;
    let ok = unsafe { OpenProcessToken(process.0, TOKEN_QUERY | TOKEN_DUPLICATE, &mut token) };
    if ok == 0 {
        return staged("desktop shell token open", Err(error()));
    }
    let token = Handle(token);
    validate_recipient_token(token.0, producer, session)?;
    let retained = staged("desktop shell token duplication", duplicate(token.0))?;
    validate_recipient_token(retained.0, producer, session)?;
    if unsafe { GetShellWindow() } != window
        || staged("desktop shell owner revalidation", owner(window))? != pid
    {
        return Err(bad(
            "desktop shell identity changed during token acquisition",
        ));
    }
    // Drop both source handles. Only the proven ordinary context survives.
    Ok(retained)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_shell_owner_is_refused() {
        assert!(owner(0).is_err());
    }

    #[test]
    #[ignore = "requires an actual ordinary desktop shell; exercised explicitly in S173 Windows validation"]
    fn exact_shell_token_is_ordinary_and_identity_bound() {
        let producer = user(CURRENT_TOKEN).unwrap();
        let session = super::super::session(CURRENT_TOKEN).unwrap();
        let token = ordinary_token(&producer, session).unwrap();
        ordinary(token.0).unwrap();
        assert!(validate_recipient_token(token.0, "S-1-5-21-0-0-0-99999", session).is_err());
        assert!(validate_recipient_token(token.0, &producer, session.wrapping_add(1)).is_err());
    }
}
