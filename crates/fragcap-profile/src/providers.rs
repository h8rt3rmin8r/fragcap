// SPDX-License-Identifier: Apache-2.0

//! The concrete providers of the resolution cascade, specification section 15.7.
//!
//! Two carry data in this slice. [`ProfileProvider`] wraps the section 15.3
//! profile lookup and stamps its answer with the profile's own declared fidelity.
//! [`ObservationProvider`] matches a live process by identity and stamps an
//! [`FidelityTier::Observed`] answer; it is the arbiter at the bottom of the
//! cascade. The remaining three ([`HintProvider`], [`EngineRuleProvider`],
//! [`PlatformWalkerProvider`]) are registered at their precedence positions and
//! decline in this slice; their data arrives in #78, S029, and S030 without
//! touching the resolver engine.
//!
//! No provider here opens a process handle. The observation provider reads only
//! the image name and path already in the process snapshot (constitution P-1).

use fragcap_core::process::ProcessTree;

use crate::engine_rule::{resolve_engine, EngineResolution};
use crate::matching::first_live_match;
use crate::resolve::{resolve, ProfileSource, ResolveError};
use crate::resolver::{
    Precedence, ProviderError, ResolutionNotes, ResolutionRequest, TargetProvider,
};
use crate::schema::{FidelityTier, Provenance};
use crate::target::{ObservedTarget, Target, TargetOrigin};

/// The provider label a profile-backed answer carries when the profile itself
/// declared no provenance.
fn source_label(source: &ProfileSource) -> &'static str {
    match source {
        ProfileSource::ExplicitPath(_) => "profile-path",
        ProfileSource::CommandLineDirectory(_) => "profile-command-line",
        ProfileSource::UserDirectory(_) => "user-profile",
        ProfileSource::Bundled => "bundled-profile",
    }
}

/// Resolves an authored package or a curated profile through the section 15.3
/// lookup. The top of the cascade.
#[derive(Default)]
pub struct ProfileProvider;

impl ProfileProvider {
    /// Build the provider.
    pub fn new() -> ProfileProvider {
        ProfileProvider
    }
}

impl TargetProvider for ProfileProvider {
    fn precedence(&self) -> Precedence {
        Precedence::Profile
    }

    fn provide(
        &self,
        request: &ResolutionRequest,
        notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        let Some(reference) = request.reference() else {
            return Ok(None);
        };
        match resolve(reference, request.search(), request.bundled()) {
            Ok(resolved) => {
                let profile = resolved.profile;
                let fidelity = profile.fidelity();
                // The profile's own provenance when it declared one; otherwise a
                // provenance naming which resolution step supplied it, so the
                // answer is never left unstamped (P-9).
                let provenance = profile.provenance().cloned().unwrap_or_else(|| {
                    Provenance::new(source_label(&resolved.source).to_string(), None)
                });
                Ok(Some(Target::new(
                    fidelity,
                    provenance,
                    TargetOrigin::Profile(profile),
                )))
            }
            // Nothing matched: not an error, the cascade continues. The detail is
            // recorded so a not-resolved outcome prints the same message as today.
            Err(e @ ResolveError::NotFound { .. }) => {
                notes.note_profile_not_found(e);
                Ok(None)
            }
            // A present-but-unusable candidate, or an unusable reference, is a
            // hard error: it must be seen, not silently skipped.
            Err(e) => Err(ProviderError::Profile(e)),
        }
    }
}

/// Matches a live process by identity and stamps an observed answer. The arbiter
/// at the bottom of the cascade; it assumes nothing about origin, so it works for
/// a modded install, a standalone game, and a plain storefront title alike.
#[derive(Default)]
pub struct ObservationProvider;

impl ObservationProvider {
    /// Build the provider.
    pub fn new() -> ObservationProvider {
        ObservationProvider
    }
}

impl TargetProvider for ObservationProvider {
    fn precedence(&self) -> Precedence {
        Precedence::RuntimeObservation
    }

    fn provide(
        &self,
        request: &ResolutionRequest,
        _notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        let (Some(identity), Some(tree)) = (request.identity(), request.tree()) else {
            return Ok(None);
        };
        Ok(observe(identity, tree))
    }
}

/// The pure core of observation, over an identity and a tree.
fn observe(identity: &crate::schema::MatchPredicates, tree: &ProcessTree) -> Option<Target> {
    let node_id = first_live_match(identity, tree)?;
    let node = tree.node(node_id)?;
    // Retain the identity that selected this process, not just the current
    // match, so the target carries its match rules (section 15.7).
    let observed = ObservedTarget::new(
        node.pid().0,
        node.image_name().to_string(),
        node.image().to_string(),
        identity.clone(),
    );
    Some(Target::new(
        FidelityTier::Observed,
        Provenance::new("runtime-observation".to_string(), None),
        TargetOrigin::Observed(observed),
    ))
}

/// The shipped hint database (issue #78). Declines in this slice.
#[derive(Default)]
pub struct HintProvider;

impl HintProvider {
    /// Build the provider.
    pub fn new() -> HintProvider {
        HintProvider
    }
}

impl TargetProvider for HintProvider {
    fn precedence(&self) -> Precedence {
        Precedence::HintDatabase
    }

    fn provide(
        &self,
        _request: &ResolutionRequest,
        _notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        Ok(None)
    }
}

/// A general engine-layout rule (S029). Resolves a game's socket-holding client
/// from the engine's documented install layout, at heuristic-unverified fidelity.
///
/// It reads the install directory the request carries and nothing else: no
/// process handle, no memory, no launch, no post-run artifact (P-1). A recognized
/// layout with one client resolves; a recognized layout it cannot single out
/// declines with a note (P-4, P-9); an unrecognized layout, an absent client, or
/// an absent input all decline so the cascade continues.
#[derive(Default)]
pub struct EngineRuleProvider;

impl EngineRuleProvider {
    /// Build the provider.
    pub fn new() -> EngineRuleProvider {
        EngineRuleProvider
    }
}

impl TargetProvider for EngineRuleProvider {
    fn precedence(&self) -> Precedence {
        Precedence::EngineRule
    }

    fn provide(
        &self,
        request: &ResolutionRequest,
        notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        let Some(install_root) = request.install_root() else {
            return Ok(None);
        };
        match resolve_engine(install_root) {
            EngineResolution::Resolved(engine_target) => Ok(Some(Target::new(
                FidelityTier::HeuristicUnverified,
                Provenance::new("engine-rule".to_string(), None),
                TargetOrigin::EngineRule(*engine_target),
            ))),
            // A recognized layout it could not single out: decline, but record the
            // ambiguity so a not-resolved outcome can explain itself. Runtime
            // observation disambiguates at runtime.
            EngineResolution::Ambiguous { engine, candidates } => {
                notes.note_engine_rule_ambiguous(engine, candidates);
                Ok(None)
            }
            // A filesystem error left the scan incomplete: decline, but record the
            // unreadable path so an inaccessible install is distinguishable from an
            // unrecognized engine (P-4, FR-009), never resolved from a partial view.
            EngineResolution::Unreadable { path } => {
                notes.note_engine_rule_unreadable(path);
                Ok(None)
            }
            // No recognized layout, or a recognized layout whose client is absent.
            EngineResolution::NoMatch => Ok(None),
        }
    }
}

/// A storefront library walker (S030). Declines in this slice.
#[derive(Default)]
pub struct PlatformWalkerProvider;

impl PlatformWalkerProvider {
    /// Build the provider.
    pub fn new() -> PlatformWalkerProvider {
        PlatformWalkerProvider
    }
}

impl TargetProvider for PlatformWalkerProvider {
    fn precedence(&self) -> Precedence {
        Precedence::PlatformWalker
    }

    fn provide(
        &self,
        _request: &ResolutionRequest,
        _notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::packet::Timestamp;
    use fragcap_core::process::{ProcessEvent, ProcessTree};

    use crate::engine_rule::Engine;
    use crate::resolve::{BundledSet, SearchPath};
    use crate::resolver::{ResolutionError, TargetResolver};
    use crate::schema::{MatchPredicates, Profile};

    fn at(n: i64) -> Timestamp {
        Timestamp::from_nanos(n)
    }

    fn parse_profile(fidelity: &str) -> Profile {
        let text = format!(
            r#"{{"schema":1,"kind":"profile","fidelity":"{fidelity}","game":{{"id":"eso","name":"ESO"}},"stage":[{{"role":"client","lifecycle":"session","terminal":true,"match":{{"exe":"eso64.exe"}}}}]}}"#
        );
        Profile::parse(&text).expect("valid profile")
    }

    fn identity(match_body: &str) -> MatchPredicates {
        let text = format!(
            r#"{{"schema":1,"kind":"profile","fidelity":"verified","game":{{"id":"t","name":"T"}},"stage":[{{"role":"target","lifecycle":"session","match":{match_body}}}]}}"#
        );
        Profile::parse(&text).expect("valid").stages()[0]
            .predicates()
            .clone()
    }

    #[test]
    fn profile_provider_stamps_with_the_profiles_declared_fidelity() {
        // A bundled profile exercises the "found" path with no filesystem. Its
        // declared fidelity is what the answer carries, not a fixed value.
        let profile = parse_profile("verified");
        let bundled = BundledSet::new(vec![profile]).expect("one profile");
        let search = SearchPath::new();
        let req = ResolutionRequest::for_reference("eso", &search, &bundled);

        let provider = ProfileProvider::new();
        let mut notes = ResolutionNotes::default();
        let target = provider
            .provide(&req, &mut notes)
            .expect("no hard error")
            .expect("an answer");
        assert_eq!(target.fidelity(), FidelityTier::Verified);
        assert!(target.profile().is_some());
        assert_eq!(target.provenance().source(), "bundled-profile");
    }

    #[test]
    fn profile_provider_stamps_authored_when_the_profile_declares_it() {
        let profile = parse_profile("authored");
        let bundled = BundledSet::new(vec![profile]).expect("one profile");
        let search = SearchPath::new();
        let req = ResolutionRequest::for_reference("eso", &search, &bundled);

        let target = ProfileProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error")
            .expect("an answer");
        assert_eq!(target.fidelity(), FidelityTier::Authored);
    }

    #[test]
    fn profile_provider_declines_and_notes_when_nothing_matches() {
        let bundled = BundledSet::empty();
        let search = SearchPath::new();
        let req = ResolutionRequest::for_reference("nosuchgame", &search, &bundled);

        let answer = ProfileProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("not a hard error");
        assert!(
            answer.is_none(),
            "a missing profile is no answer, not an error"
        );
    }

    #[test]
    fn profile_provider_errors_on_an_unusable_reference() {
        let bundled = BundledSet::empty();
        let search = SearchPath::new();
        // Not a file and not a valid slug.
        let req = ResolutionRequest::for_reference("../etc", &search, &bundled);

        match ProfileProvider::new().provide(&req, &mut ResolutionNotes::default()) {
            Err(ProviderError::Profile(ResolveError::InvalidReference { .. })) => {}
            other => panic!("expected an InvalidReference hard error, got {other:?}"),
        }
    }

    #[test]
    fn observation_provider_yields_an_observed_target() {
        let id = identity(r#"{"exe":"eso64.exe"}"#);
        let mut tree = ProcessTree::new();
        tree.apply(ProcessEvent::started(
            42,
            0,
            "C:\\Games\\ESO\\eso64.exe",
            "eso64.exe",
            at(1),
        ));
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = ResolutionRequest::for_observation(&id, &tree, &search, &bundled);

        let target = ObservationProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error")
            .expect("an observed answer");
        assert_eq!(target.fidelity(), FidelityTier::Observed);
        assert_eq!(target.provenance().source(), "runtime-observation");
        match target.origin() {
            TargetOrigin::Observed(o) => {
                assert_eq!(o.pid(), 42);
                assert_eq!(o.image_name(), "eso64.exe");
                assert_eq!(o.image_path(), "C:\\Games\\ESO\\eso64.exe");
                // The identity that selected the process is retained on the
                // target (section 15.7), reusable for a later re-match.
                assert_eq!(o.identity(), &id);
            }
            other => panic!("expected an observed origin, got {other:?}"),
        }
    }

    #[test]
    fn observation_provider_declines_without_a_match() {
        let id = identity(r#"{"exe":"other.exe"}"#);
        let mut tree = ProcessTree::new();
        tree.apply(ProcessEvent::started(
            1,
            0,
            "C:\\G\\eso64.exe",
            "eso64.exe",
            at(1),
        ));
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = ResolutionRequest::for_observation(&id, &tree, &search, &bundled);

        let answer = ObservationProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error");
        assert!(answer.is_none());
    }

    #[test]
    fn the_stub_providers_decline_at_their_precedence() {
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        // A reference request carries no install root, so the engine-rule provider
        // declines here just as the still-empty hint and walker providers do.
        let req = ResolutionRequest::for_reference("eso", &search, &bundled);
        let mut notes = ResolutionNotes::default();

        assert_eq!(HintProvider::new().precedence(), Precedence::HintDatabase);
        assert!(HintProvider::new()
            .provide(&req, &mut notes)
            .unwrap()
            .is_none());
        assert_eq!(
            EngineRuleProvider::new().precedence(),
            Precedence::EngineRule
        );
        assert!(EngineRuleProvider::new()
            .provide(&req, &mut notes)
            .unwrap()
            .is_none());
        assert_eq!(
            PlatformWalkerProvider::new().precedence(),
            Precedence::PlatformWalker
        );
        assert!(PlatformWalkerProvider::new()
            .provide(&req, &mut notes)
            .unwrap()
            .is_none());
    }

    // A minimal Unreal install tree under the system temp dir, removed on drop.
    struct UnrealTree {
        root: std::path::PathBuf,
    }

    impl UnrealTree {
        fn new(game: &str, shipping_exes: &[&str]) -> UnrealTree {
            use std::sync::atomic::{AtomicU32, Ordering};
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "fragcap-provider-unreal-{}-{}-{}",
                std::process::id(),
                game,
                n
            ));
            let win64 = root.join(game).join("Binaries").join("Win64");
            std::fs::create_dir_all(&win64).expect("create win64");
            std::fs::write(root.join(format!("{game}.exe")), b"").expect("stub");
            for exe in shipping_exes {
                std::fs::write(win64.join(exe), b"").expect("shipping");
            }
            UnrealTree { root }
        }

        fn path(&self) -> &std::path::Path {
            &self.root
        }
    }

    impl Drop for UnrealTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn engine_rule_provider_resolves_an_unreal_install() {
        let tree = UnrealTree::new("Providerland", &["Providerland-Win64-Shipping.exe"]);
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = ResolutionRequest::for_install(tree.path(), &search, &bundled);

        let target = EngineRuleProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error")
            .expect("an engine-rule answer");
        assert_eq!(target.fidelity(), FidelityTier::HeuristicUnverified);
        assert_eq!(target.provenance().source(), "engine-rule");
        match target.origin() {
            TargetOrigin::EngineRule(t) => {
                assert_eq!(t.image_name(), "Providerland-Win64-Shipping.exe");
                assert!(t.identity().exe().is_some());
            }
            other => panic!("expected an engine-rule origin, got {other:?}"),
        }
        // An engine rule never yields a profile.
        assert!(target.profile().is_none());
    }

    #[test]
    fn engine_rule_provider_declines_on_an_unrecognized_layout() {
        let dir =
            std::env::temp_dir().join(format!("fragcap-provider-nomatch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create");
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = ResolutionRequest::for_install(&dir, &search, &bundled);
        let answer = EngineRuleProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error");
        assert!(answer.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn engine_rule_provider_declines_and_notes_on_ambiguity() {
        let tree = UnrealTree::new(
            "Twins",
            &["Twins-Win64-Shipping.exe", "Twins2-Win64-Shipping.exe"],
        );
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = ResolutionRequest::for_install(tree.path(), &search, &bundled);
        let mut notes = ResolutionNotes::default();
        let answer = EngineRuleProvider::new()
            .provide(&req, &mut notes)
            .expect("no hard error");
        assert!(
            answer.is_none(),
            "an ambiguous layout is a decline, not a pick"
        );

        // The ambiguity surfaces through Unresolved when nothing lower resolves.
        let resolver =
            TargetResolver::new(vec![Box::new(EngineRuleProvider::new())]).expect("one provider");
        match resolver.resolve(&req) {
            Err(ResolutionError::Unresolved(u)) => {
                let ambiguity = u
                    .engine_rule_ambiguous()
                    .expect("the ambiguity is recorded");
                assert_eq!(ambiguity.engine(), Engine::Unreal);
                assert_eq!(ambiguity.candidates(), 2);
            }
            other => panic!("expected Unresolved with an ambiguity note, got {other:?}"),
        }
    }

    #[test]
    fn engine_rule_provider_surfaces_an_unreadable_install() {
        // An install root that cannot be read declines with the path recorded, so
        // an inaccessible install is distinguishable from an unrecognized engine
        // (FR-009), surfaced through Unresolved rather than swallowed.
        let base = std::env::temp_dir().join(format!(
            "fragcap-provider-unreadable-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let absent = base.join("does-not-exist");
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let req = ResolutionRequest::for_install(&absent, &search, &bundled);

        let answer = EngineRuleProvider::new()
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error");
        assert!(answer.is_none(), "an unreadable install is a decline");

        let resolver =
            TargetResolver::new(vec![Box::new(EngineRuleProvider::new())]).expect("one provider");
        match resolver.resolve(&req) {
            Err(ResolutionError::Unresolved(u)) => {
                assert_eq!(
                    u.engine_rule_unreadable(),
                    Some(absent.as_path()),
                    "the unreadable install path is observable"
                );
            }
            other => panic!("expected Unresolved with the unreadable path, got {other:?}"),
        }
    }

    #[test]
    fn a_profile_outranks_the_engine_rule_for_the_same_install() {
        // A request carries both a profile reference and the install root. When a
        // matching profile exists it wins on precedence; when it does not, the
        // engine rule answers. Registration order does not change either outcome.
        let tree = UnrealTree::new("Ranked", &["Ranked-Win64-Shipping.exe"]);
        let search = SearchPath::new();

        let profile = parse_profile("verified");
        let bundled = BundledSet::new(vec![profile]).expect("one profile");
        let req = ResolutionRequest::for_reference("eso", &search, &bundled)
            .with_install_root(tree.path());

        for order in 0..2 {
            let providers: Vec<Box<dyn TargetProvider>> = if order == 0 {
                vec![
                    Box::new(ProfileProvider::new()),
                    Box::new(EngineRuleProvider::new()),
                ]
            } else {
                vec![
                    Box::new(EngineRuleProvider::new()),
                    Box::new(ProfileProvider::new()),
                ]
            };
            let resolver = TargetResolver::new(providers).expect("distinct precedences");
            let target = resolver.resolve(&req).expect("resolves");
            assert_eq!(
                target.fidelity(),
                FidelityTier::Verified,
                "the profile outranks the engine rule regardless of registration order"
            );
            assert!(target.profile().is_some());
        }

        // With no matching profile, the same request resolves via the engine rule.
        let empty = BundledSet::empty();
        let req = ResolutionRequest::for_reference("nosuchgame", &search, &empty)
            .with_install_root(tree.path());
        let resolver = TargetResolver::new(vec![
            Box::new(ProfileProvider::new()),
            Box::new(EngineRuleProvider::new()),
        ])
        .expect("distinct precedences");
        let target = resolver.resolve(&req).expect("resolves via engine rule");
        assert_eq!(target.fidelity(), FidelityTier::HeuristicUnverified);
        assert_eq!(target.provenance().source(), "engine-rule");
    }
}
