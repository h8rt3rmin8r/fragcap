// SPDX-License-Identifier: Apache-2.0

//! Pure policy for an explicit warm-to-cold Deep Capture transition.

use std::fmt;
use std::time::Duration;

use super::LaunchCase;

/// Longest time an operator may be asked to reach a cold launch state.
pub const MAX_WARM_RESTART_WAIT: Duration = Duration::from_secs(120);

/// Immutable authority for one operator-owned close-and-retry wait.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarmRestartPlan {
    warm_case: LaunchCase,
    cold_case: LaunchCase,
    images: Vec<String>,
    deadline: Duration,
}

impl WarmRestartPlan {
    /// Build a bounded plan from an observed warm launch case and its declared images.
    pub fn new<I, S>(
        warm_case: LaunchCase,
        declared_images: I,
        requested_wait: Option<Duration>,
    ) -> Result<Self, WarmRestartPlanError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let cold_case = corresponding_cold_case(warm_case)
            .ok_or(WarmRestartPlanError::UnsupportedCase(warm_case))?;
        let mut images = Vec::<String>::new();
        for image in images_from(declared_images) {
            if image.trim().is_empty() {
                return Err(WarmRestartPlanError::EmptyImage);
            }
            if !images
                .iter()
                .any(|known| known.eq_ignore_ascii_case(&image))
            {
                images.push(image);
            }
        }
        if images.is_empty() {
            return Err(WarmRestartPlanError::MissingImages);
        }
        Ok(Self {
            warm_case,
            cold_case,
            images,
            deadline: requested_wait
                .unwrap_or(MAX_WARM_RESTART_WAIT)
                .min(MAX_WARM_RESTART_WAIT),
        })
    }

    pub fn warm_case(&self) -> LaunchCase {
        self.warm_case
    }

    pub fn cold_case(&self) -> LaunchCase {
        self.cold_case
    }

    pub fn images(&self) -> &[String] {
        &self.images
    }

    pub fn deadline(&self) -> Duration {
        self.deadline
    }

    /// A complete snapshot is cold only when every declared image is absent.
    pub fn snapshot_is_cold(&self, present: &[bool]) -> bool {
        present.len() == self.images.len() && present.iter().all(|value| !value)
    }
}

fn images_from<I, S>(images: I) -> impl Iterator<Item = String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    images.into_iter().map(Into::into)
}

/// Return the exact supported cold counterpart for a warm launch case.
pub fn corresponding_cold_case(warm_case: LaunchCase) -> Option<LaunchCase> {
    match warm_case {
        LaunchCase::SteamProtocolWarm => Some(LaunchCase::SteamProtocolCold),
        LaunchCase::DirectExeWarm => Some(LaunchCase::DirectExeCold),
        LaunchCase::PublisherLauncherWarm | LaunchCase::PublisherLauncherGameStartCleanWarm => {
            Some(LaunchCase::PublisherLauncherCold)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WarmRestartPlanError {
    UnsupportedCase(LaunchCase),
    MissingImages,
    EmptyImage,
}

impl fmt::Display for WarmRestartPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCase(case) => write!(
                formatter,
                "{} is not a supported warm restart case",
                case.as_str()
            ),
            Self::MissingImages => formatter.write_str("warm restart has no declared images"),
            Self::EmptyImage => formatter.write_str("warm restart contains an empty image name"),
        }
    }
}

impl std::error::Error for WarmRestartPlanError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_warm_case_maps_to_its_exact_cold_path() {
        assert_eq!(
            corresponding_cold_case(LaunchCase::DirectExeWarm),
            Some(LaunchCase::DirectExeCold)
        );
        assert_eq!(
            corresponding_cold_case(LaunchCase::SteamProtocolWarm),
            Some(LaunchCase::SteamProtocolCold)
        );
        assert_eq!(
            corresponding_cold_case(LaunchCase::PublisherLauncherWarm),
            Some(LaunchCase::PublisherLauncherCold)
        );
        assert_eq!(corresponding_cold_case(LaunchCase::DirectExeCold), None);
    }

    #[test]
    fn plan_deduplicates_images_and_caps_the_deadline() {
        let plan = WarmRestartPlan::new(
            LaunchCase::PublisherLauncherWarm,
            ["Launcher.exe", "launcher.EXE", "Game.exe"],
            Some(Duration::from_secs(600)),
        )
        .unwrap();
        assert_eq!(plan.images(), ["Launcher.exe", "Game.exe"]);
        assert_eq!(plan.deadline(), MAX_WARM_RESTART_WAIT);
    }

    #[test]
    fn partial_closure_is_never_cold() {
        let plan = WarmRestartPlan::new(
            LaunchCase::PublisherLauncherWarm,
            ["Launcher.exe", "Game.exe"],
            None,
        )
        .unwrap();
        assert!(!plan.snapshot_is_cold(&[false, true]));
        assert!(!plan.snapshot_is_cold(&[false]));
        assert!(plan.snapshot_is_cold(&[false, false]));
    }
}
