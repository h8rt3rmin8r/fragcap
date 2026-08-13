// SPDX-License-Identifier: Apache-2.0

//! The Steam platform-walker provider, specification section 15.7 (S030).
//!
//! The walker turns Steam's installed library into cascade answers. Its enumeration
//! (which title, and where it is installed) is done by [`crate::discover`] /
//! [`crate::install_root_in`]; the caller places the resulting install directory on
//! the resolution request as `install_root`, so the higher-precedence engine-rule
//! provider can name a socket holder from layout. This module is the provider that
//! answers below the engine rule: given an install directory, it classifies the
//! executables and resolves a single client, or declines.
//!
//! It declines rather than guess. A client is resolved only when, after dropping
//! installers, redistributables, helpers, and launcher stubs (the same predicates
//! the scaffold uses), exactly one plausible client executable remains. Zero, or
//! several, is a decline, and the cascade falls through to runtime observation.
//! Selecting a client by size among several candidates is the coincidental
//! heuristic the library research found unreliable, so the walker, feeding
//! automatic capture, does not guess where the human-reviewed scaffold does.
//!
//! Every answer is stamped [`FidelityTier::HeuristicUnverified`] with provenance
//! `steam-library`, an honest name for the library walk and install-directory
//! classification it performed; it never claims `steam-appinfo`, a source it does
//! not read (constitution P-9). The walker reads the filesystem only: no process
//! handle, no memory, no network (P-1).

use std::path::{Path, PathBuf};

use fragcap_profile::{
    FidelityTier, MatchPredicates, Precedence, Provenance, ProviderError, ResolutionNotes,
    ResolutionRequest, Target, TargetOrigin, TargetProvider, WalkerTarget,
};

use crate::scaffold::{is_launcher, is_non_game, scan, ExecutableImage};
use crate::SteamError;

/// The result of classifying a Steam install directory. Internal; the provider
/// maps it onto the cascade contract.
enum ClientResolution {
    /// Exactly one plausible client executable remained. Boxed because a
    /// [`WalkerTarget`] is much larger than the other variants.
    Resolved(Box<WalkerTarget>),
    /// No plausible client (only launchers, or nothing).
    NoMatch,
    /// More than one plausible client remained; the walker declines rather than
    /// guess one.
    Ambiguous { candidates: usize },
    /// The install directory could not be read.
    Unreadable { path: PathBuf },
}

/// Classify an install directory into a single client, or a decline.
fn client_for(install_dir: &Path) -> ClientResolution {
    let images = match scan(install_dir) {
        Ok(images) => images,
        // An unreadable directory (or subtree) is not an empty one; surface it
        // rather than report a clean no-match (P-4), mirroring the engine rule.
        Err(SteamError::Io { path, .. }) => return ClientResolution::Unreadable { path },
        Err(_) => {
            return ClientResolution::Unreadable {
                path: install_dir.to_path_buf(),
            }
        }
    };

    // Drop obvious non-game executables, keeping the original set if that would
    // empty it, exactly as the scaffold classifier does, so the two agree on what
    // an installer or helper is.
    let mut candidates: Vec<ExecutableImage> = images
        .iter()
        .filter(|image| !is_non_game(&image.file_name))
        .cloned()
        .collect();
    if candidates.is_empty() {
        candidates = images;
    }

    // The plausible clients are the non-launchers.
    let clients: Vec<ExecutableImage> = candidates
        .into_iter()
        .filter(|image| !is_launcher(image))
        .collect();

    match clients.len() {
        0 => ClientResolution::NoMatch,
        1 => {
            let client = &clients[0];
            match MatchPredicates::with_exe(&client.file_name) {
                Ok(identity) => ClientResolution::Resolved(Box::new(WalkerTarget::new(
                    "steam".to_string(),
                    client.file_name.clone(),
                    client.path.to_string_lossy().into_owned(),
                    identity,
                ))),
                // A client whose name is not a usable match pattern is treated as
                // no match rather than a fabricated target.
                Err(_) => ClientResolution::NoMatch,
            }
        }
        candidates => ClientResolution::Ambiguous { candidates },
    }
}

/// The Steam platform-walker provider. Resolves a client from a Steam install
/// directory at heuristic-unverified fidelity, or declines so the cascade
/// continues to runtime observation.
#[derive(Default)]
pub struct SteamWalkerProvider;

impl SteamWalkerProvider {
    /// Build the provider.
    pub fn new() -> SteamWalkerProvider {
        SteamWalkerProvider
    }
}

impl TargetProvider for SteamWalkerProvider {
    fn precedence(&self) -> Precedence {
        Precedence::PlatformWalker
    }

    fn provide(
        &self,
        request: &ResolutionRequest,
        notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        let Some(install_root) = request.install_root() else {
            return Ok(None);
        };
        match client_for(install_root) {
            ClientResolution::Resolved(target) => Ok(Some(Target::new(
                FidelityTier::HeuristicUnverified,
                Provenance::new("steam-library".to_string(), None),
                TargetOrigin::PlatformWalker(*target),
            ))),
            ClientResolution::NoMatch => Ok(None),
            ClientResolution::Ambiguous { candidates } => {
                notes.note_walker_ambiguous(candidates);
                Ok(None)
            }
            ClientResolution::Unreadable { path } => {
                notes.note_walker_unreadable(path);
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use fragcap_profile::{
        BundledSet, ResolutionError, ResolutionNotes, SearchPath, TargetResolver,
    };

    use crate::test_support::TempTree;

    fn resolved(res: ClientResolution) -> WalkerTarget {
        match res {
            ClientResolution::Resolved(t) => *t,
            ClientResolution::NoMatch => panic!("expected Resolved, got NoMatch"),
            ClientResolution::Ambiguous { candidates } => {
                panic!("expected Resolved, got Ambiguous({candidates})")
            }
            ClientResolution::Unreadable { path } => {
                panic!("expected Resolved, got Unreadable({})", path.display())
            }
        }
    }

    #[test]
    fn a_single_client_among_installers_and_helpers_resolves() {
        let tree = TempTree::new();
        tree.write_exe(&tree.path().join("MyGame.exe"), 4096);
        tree.write_exe(&tree.path().join("vc_redist.x64.exe"), 8192); // dropped: non-game
        tree.write_exe(&tree.path().join("UnityCrashHandler64.exe"), 2048); // dropped: helper
        let target = resolved(client_for(tree.path()));
        assert_eq!(target.platform(), "steam");
        assert_eq!(target.image_name(), "MyGame.exe");
        assert!(target.identity().exe().is_some());
    }

    #[test]
    fn only_launchers_or_nothing_is_a_no_match() {
        let tree = TempTree::new();
        tree.write_exe(&tree.path().join("MyGameLauncher.exe"), 4096); // launcher token
        assert!(matches!(client_for(tree.path()), ClientResolution::NoMatch));

        let empty = TempTree::new();
        assert!(matches!(
            client_for(empty.path()),
            ClientResolution::NoMatch
        ));
    }

    #[test]
    fn several_plausible_clients_are_ambiguous_not_a_guess() {
        let tree = TempTree::new();
        tree.write_exe(&tree.path().join("ClientA.exe"), 9000);
        tree.write_exe(&tree.path().join("ClientB.exe"), 4000);
        match client_for(tree.path()) {
            ClientResolution::Ambiguous { candidates } => assert_eq!(candidates, 2),
            _ => panic!("expected Ambiguous, got a different outcome"),
        }
    }

    #[test]
    fn an_unreadable_install_is_surfaced_not_a_no_match() {
        let tree = TempTree::new();
        let absent = tree.path().join("does-not-exist");
        match client_for(&absent) {
            ClientResolution::Unreadable { path } => assert_eq!(path, absent),
            _ => panic!("expected Unreadable"),
        }
    }

    fn empty_request<'a>(
        install_root: &'a Path,
        search: &'a SearchPath,
        bundled: &'a BundledSet,
    ) -> ResolutionRequest<'a> {
        ResolutionRequest::for_install(install_root, search, bundled)
    }

    #[test]
    fn the_provider_stamps_heuristic_unverified_and_steam_library() {
        let tree = TempTree::new();
        tree.write_exe(&tree.path().join("MyGame.exe"), 4096);
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = empty_request(tree.path(), &search, &bundled);

        let target = SteamWalkerProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error")
            .expect("an answer");
        assert_eq!(target.fidelity(), FidelityTier::HeuristicUnverified);
        assert_eq!(target.provenance().source(), "steam-library");
        match target.origin() {
            TargetOrigin::PlatformWalker(t) => assert_eq!(t.image_name(), "MyGame.exe"),
            _ => panic!("expected a platform-walker origin"),
        }
        assert!(target.profile().is_none());
    }

    #[test]
    fn the_provider_declines_without_an_install_root() {
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        // A reference request carries no install root, so the walker declines.
        let req = ResolutionRequest::for_reference("eso", &search, &bundled);
        let answer = SteamWalkerProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error");
        assert!(answer.is_none());
    }

    #[test]
    fn the_provider_declines_and_notes_on_ambiguity_surfaced_through_unresolved() {
        let tree = TempTree::new();
        tree.write_exe(&tree.path().join("ClientA.exe"), 9000);
        tree.write_exe(&tree.path().join("ClientB.exe"), 4000);
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = empty_request(tree.path(), &search, &bundled);

        let resolver =
            TargetResolver::new(vec![Box::new(SteamWalkerProvider::new())]).expect("one provider");
        match resolver.resolve(&req) {
            Err(ResolutionError::Unresolved(u)) => {
                assert_eq!(u.walker_ambiguous().expect("recorded").candidates(), 2);
            }
            _ => panic!("expected Unresolved with a walker-ambiguity note"),
        }
    }

    #[test]
    fn the_provider_surfaces_an_unreadable_install_through_unresolved() {
        let tree = TempTree::new();
        let absent = tree.path().join("nope");
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = empty_request(&absent, &search, &bundled);

        let resolver =
            TargetResolver::new(vec![Box::new(SteamWalkerProvider::new())]).expect("one provider");
        match resolver.resolve(&req) {
            Err(ResolutionError::Unresolved(u)) => {
                assert_eq!(u.walker_unreadable(), Some(absent.as_path()));
            }
            _ => panic!("expected Unresolved with a walker-unreadable note"),
        }
    }
}
