// SPDX-License-Identifier: Apache-2.0

//! Managed launch (specification section 16.4).
//!
//! [`launch_request`] validates a profile's managed-launch preconditions and
//! builds the request; it is portable and unit-tested. [`launch`] issues the
//! `steam://` protocol handler through `ShellExecuteW` and is Windows-only. The
//! caller (the run path) issues the launch only after the session is watching
//! and the sinks are open, which is what removes the acquisition race (FR-010);
//! the live launch itself is tier-2 and is never asserted as run in CI (D5).

use std::fmt;

use fragcap_profile::Profile;

use crate::SteamError;

/// The platform a managed launch supports.
const STEAM_PLATFORM: &str = "steam";

/// A validated managed-launch request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchRequest {
    /// The Steam application identifier to launch.
    pub app_id: String,
    /// The protocol-handler URL, `steam://run/<app_id>` (S17 D5).
    pub url: String,
}

/// Why a profile cannot drive a managed launch.
///
/// Surfaced by the CLI as a named usage error before any capture starts
/// (FR-011). Kept distinct from [`SteamError`] because these are configuration
/// faults in the profile, not runtime failures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchConfigError {
    /// The profile declares no `game.platform`.
    MissingPlatform,
    /// The profile declares a platform other than `steam`.
    UnsupportedPlatform(String),
    /// The profile declares no `game.app_id`.
    MissingAppId,
}

impl fmt::Display for LaunchConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaunchConfigError::MissingPlatform => write!(
                f,
                "managed launch (--launch) requires `game.platform` in the profile"
            ),
            LaunchConfigError::UnsupportedPlatform(p) => write!(
                f,
                "managed launch (--launch) supports platform `steam`, not `{p}`"
            ),
            LaunchConfigError::MissingAppId => write!(
                f,
                "managed launch (--launch) requires `game.app_id` in the profile"
            ),
        }
    }
}

impl std::error::Error for LaunchConfigError {}

/// Validate managed-launch preconditions and build the request.
///
/// Requires `game.platform == "steam"` and `game.app_id` (FR-011). Portable and
/// unit-tested; it does not touch the operating system.
pub fn launch_request(profile: &Profile) -> Result<LaunchRequest, LaunchConfigError> {
    let game = profile.game();
    match game.platform() {
        None => return Err(LaunchConfigError::MissingPlatform),
        Some(p) if p != STEAM_PLATFORM => {
            return Err(LaunchConfigError::UnsupportedPlatform(p.to_string()))
        }
        Some(_) => {}
    }
    let app_id = game.app_id().ok_or(LaunchConfigError::MissingAppId)?;
    Ok(LaunchRequest {
        app_id: app_id.to_string(),
        url: format!("steam://run/{app_id}"),
    })
}

/// Start the title through Steam's protocol handler.
///
/// Windows-only: issues `ShellExecuteW("open", "steam://run/<app_id>")`. Tier-2
/// and never asserted as run in CI (D5). Opens no process handle (P-1).
#[cfg(windows)]
pub fn launch(request: &LaunchRequest) -> Result<(), SteamError> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;

    // SW_SHOWNORMAL == 1; passed literally to avoid pulling another feature.
    const SW_SHOWNORMAL: i32 = 1;

    let op = crate::to_wide("open");
    let file = crate::to_wide(&request.url);
    let result = unsafe {
        ShellExecuteW(
            0,
            op.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW returns an HINSTANCE; a value greater than 32 is success.
    if (result as isize) > 32 {
        Ok(())
    } else {
        Err(SteamError::LaunchFailed {
            url: request.url.clone(),
            code: result as i64,
        })
    }
}

/// Non-Windows: there is no Steam protocol handler to invoke.
#[cfg(not(windows))]
pub fn launch(_request: &LaunchRequest) -> Result<(), SteamError> {
    Err(SteamError::UnsupportedPlatform)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a test profile, splicing extra `game` fields (each with a leading
    /// comma) into the JSON game object.
    fn profile(game_extra: &str) -> Profile {
        let text = format!(
            r#"{{"schema":1,"kind":"profile","fidelity":"verified","game":{{"id":"g","name":"G"{game_extra}}},"stage":[{{"role":"client","lifecycle":"session","terminal":true,"match":{{"exe":"g.exe"}}}}]}}"#
        );
        Profile::parse(&text).expect("valid test profile")
    }

    #[test]
    fn a_steam_profile_yields_the_run_url() {
        let p = profile(r#","platform":"steam","app_id":"900883""#);
        let req = launch_request(&p).unwrap();
        assert_eq!(req.app_id, "900883");
        assert_eq!(req.url, "steam://run/900883");
    }

    #[test]
    fn a_missing_platform_is_refused() {
        let p = profile(r#","app_id":"900883""#);
        assert_eq!(launch_request(&p), Err(LaunchConfigError::MissingPlatform));
    }

    #[test]
    fn a_non_steam_platform_is_refused() {
        let p = profile(r#","platform":"epic","app_id":"1""#);
        assert_eq!(
            launch_request(&p),
            Err(LaunchConfigError::UnsupportedPlatform("epic".to_string()))
        );
    }

    #[test]
    fn a_missing_app_id_is_refused() {
        let p = profile(r#","platform":"steam""#);
        assert_eq!(launch_request(&p), Err(LaunchConfigError::MissingAppId));
    }
}
