// SPDX-License-Identifier: Apache-2.0

//! The target resolution cascade, specification section 15.7.
//!
//! A [`TargetResolver`] holds a set of [`TargetProvider`]s of varying trust,
//! sorts them by a fixed [`Precedence`], and queries them highest first. The
//! first provider to answer wins; its answer is a [`Target`] stamped with the
//! source's fidelity and provenance. A provider that hits a hard error aborts the
//! cascade; if every provider declines, resolution fails with an [`Unresolved`]
//! rather than a silent empty answer (constitution P-4).
//!
//! # The order is imposed, not incidental
//!
//! Providers are sorted by [`Precedence`] at construction, so the result never
//! depends on the order they were registered or iterated in. This mirrors the
//! discipline of the attribution join in `fragcap-attr`: an implementation that
//! took the first hit off an unordered set would pass an ordinary test and
//! produce answers that changed between runs. The permutation test in this module
//! is what proves the order is imposed.
//!
//! # Precedence and fidelity are related but not the same
//!
//! [`Precedence`] is a property of the provider and is the resolver's spine.
//! [`FidelityTier`](crate::schema::FidelityTier) is a stamp carried on each
//! answer. They correlate (a higher-precedence provider carries an equal or
//! higher tier), and the design keeps provider order consistent with the fidelity
//! ceiling of each provider, but they are distinct: the three heuristic providers
//! (hint, engine rule, platform walker) share a tier yet occupy distinct
//! precedence positions, so provider order carries information the tier alone does
//! not.

use std::fmt;
use std::path::Path;

use fragcap_core::process::ProcessTree;

use crate::engine_rule::Engine;
use crate::resolve::{BundledSet, ResolveError, SearchPath};
use crate::schema::MatchPredicates;
use crate::target::Target;

/// The fixed precedence order of the providers, specification section 15.7.
///
/// Declared highest trust first, so the derived [`Ord`] and a plain ascending
/// sort put the highest-precedence provider first. The profile provider covers
/// both of the issue's top two positions (an authored package and a verified
/// profile are both profiles, distinguished by the fidelity the file declares and
/// by the section 15.3 file precedence), so there is one profile position here
/// rather than two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Precedence {
    /// A user-authored package or a curated profile (`resolve()` picks one).
    Profile,
    /// The shipped hint database (issue #78).
    HintDatabase,
    /// A general engine-layout rule (S029).
    EngineRule,
    /// A storefront library walker (S030).
    PlatformWalker,
    /// Runtime observation: the arbiter at the bottom of the cascade.
    RuntimeObservation,
}

/// What a provider may read to answer a resolution.
///
/// Each provider takes only the inputs it needs. A provider whose inputs are
/// absent simply declines, so a request that carries only a profile reference is
/// answered by the profile provider and a request that carries only an identity
/// and a process tree is answered by the observation provider.
pub struct ResolutionRequest<'a> {
    reference: Option<&'a str>,
    search: &'a SearchPath,
    bundled: &'a BundledSet,
    identity: Option<&'a MatchPredicates>,
    tree: Option<&'a ProcessTree>,
    install_root: Option<&'a Path>,
}

impl<'a> ResolutionRequest<'a> {
    /// A request that resolves a profile reference (the command-line case).
    pub fn for_reference(
        reference: &'a str,
        search: &'a SearchPath,
        bundled: &'a BundledSet,
    ) -> ResolutionRequest<'a> {
        ResolutionRequest {
            reference: Some(reference),
            search,
            bundled,
            identity: None,
            tree: None,
            install_root: None,
        }
    }

    /// A request that resolves a live process by identity (the observation case).
    ///
    /// Carries empty search inputs so the profile provider declines; only the
    /// observation provider can answer.
    pub fn for_observation(
        identity: &'a MatchPredicates,
        tree: &'a ProcessTree,
        search: &'a SearchPath,
        bundled: &'a BundledSet,
    ) -> ResolutionRequest<'a> {
        ResolutionRequest {
            reference: None,
            search,
            bundled,
            identity: Some(identity),
            tree: Some(tree),
            install_root: None,
        }
    }

    /// A request that resolves a game's client from its install directory (the
    /// engine-rule case).
    ///
    /// Carries no reference and no process tree, so the profile and observation
    /// providers decline; the engine-rule provider inspects the install root. The
    /// S030 platform walker will populate the same input, so it composes with the
    /// engine-rule provider without changing it.
    pub fn for_install(
        install_root: &'a Path,
        search: &'a SearchPath,
        bundled: &'a BundledSet,
    ) -> ResolutionRequest<'a> {
        ResolutionRequest {
            reference: None,
            search,
            bundled,
            identity: None,
            tree: None,
            install_root: Some(install_root),
        }
    }

    /// The profile reference to resolve, if the request carries one.
    pub fn reference(&self) -> Option<&str> {
        self.reference
    }

    /// The section 15.3 search path.
    pub fn search(&self) -> &SearchPath {
        self.search
    }

    /// The bundled profile set.
    pub fn bundled(&self) -> &BundledSet {
        self.bundled
    }

    /// The identity to observe, if the request carries one.
    pub fn identity(&self) -> Option<&MatchPredicates> {
        self.identity
    }

    /// The observed process tree, if the request carries one.
    pub fn tree(&self) -> Option<&ProcessTree> {
        self.tree
    }

    /// The install directory to inspect for an engine layout, if the request
    /// carries one.
    pub fn install_root(&self) -> Option<&Path> {
        self.install_root
    }

    /// Add an install root to a request that already carries other inputs.
    ///
    /// A real request carries every input available, and the cascade's precedence
    /// decides between the providers that can answer. This is how a request that
    /// resolves a profile reference can also offer the engine-rule provider an
    /// install directory: the higher-precedence profile answer wins when it
    /// resolves, and the engine rule answers only when the profile does not.
    pub fn with_install_root(mut self, install_root: &'a Path) -> ResolutionRequest<'a> {
        self.install_root = Some(install_root);
        self
    }
}

/// A hard failure inside a provider that aborts the cascade.
///
/// Distinct from "no answer": a present-but-unusable input (a broken profile, an
/// unusable reference) is an error the operator must see, not a silent skip that
/// lets a lower-precedence provider answer a question the operator did not ask.
#[derive(Debug)]
pub enum ProviderError {
    /// The profile provider found a candidate it could not use, or was given a
    /// reference that is neither a file nor a valid slug.
    Profile(ResolveError),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Profile(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProviderError::Profile(e) => Some(e),
        }
    }
}

/// Why the engine-rule provider declined despite recognizing a layout.
///
/// Recorded when a rule matched more than one candidate client under one engine
/// (for example two `*-Win64-Shipping.exe` files). The provider declines rather
/// than pick one arbitrarily (P-9), and this note lets the decline explain itself
/// if nothing lower in the cascade resolves either (P-4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineRuleAmbiguity {
    engine: Engine,
    candidates: usize,
}

impl EngineRuleAmbiguity {
    /// The engine whose layout matched ambiguously.
    pub fn engine(&self) -> Engine {
        self.engine
    }

    /// How many candidate clients the rule matched.
    pub fn candidates(&self) -> usize {
        self.candidates
    }
}

impl fmt::Display for EngineRuleAmbiguity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the {} engine rule matched {} candidate clients, so it declined \
             rather than pick one; runtime observation will disambiguate",
            self.engine.as_str(),
            self.candidates
        )
    }
}

/// Notes a provider records while declining, so a not-resolved outcome can
/// explain itself.
///
/// The profile provider records its [`ResolveError::NotFound`] here when nothing
/// matched a reference, so the command line can print the same "searched ..."
/// message it prints today even though the cascade continued past it. The
/// engine-rule provider records an [`EngineRuleAmbiguity`] when it recognized a
/// layout but could not single out one client.
#[derive(Default)]
pub struct ResolutionNotes {
    profile_not_found: Option<ResolveError>,
    engine_rule_ambiguous: Option<EngineRuleAmbiguity>,
}

impl ResolutionNotes {
    /// Record that the profile provider found nothing for the reference.
    pub fn note_profile_not_found(&mut self, error: ResolveError) {
        self.profile_not_found = Some(error);
    }

    /// Record that the engine-rule provider recognized a layout but matched more
    /// than one candidate client, so it declined.
    pub fn note_engine_rule_ambiguous(&mut self, engine: Engine, candidates: usize) {
        self.engine_rule_ambiguous = Some(EngineRuleAmbiguity { engine, candidates });
    }
}

/// Nothing in the cascade answered.
///
/// A distinct, named outcome rather than an empty success, so a capture is never
/// armed against nothing without saying so (P-4). Carries the profile provider's
/// not-found detail when the profile path was attempted, so the caller can render
/// the same message it renders today.
#[derive(Debug)]
pub struct Unresolved {
    profile_not_found: Option<ResolveError>,
    engine_rule_ambiguous: Option<EngineRuleAmbiguity>,
}

impl Unresolved {
    /// The profile provider's not-found error, if the profile path was attempted
    /// and nothing matched.
    pub fn profile_not_found(&self) -> Option<&ResolveError> {
        self.profile_not_found.as_ref()
    }

    /// The engine-rule provider's ambiguity, if it recognized a layout but
    /// declined because it matched more than one candidate client.
    pub fn engine_rule_ambiguous(&self) -> Option<EngineRuleAmbiguity> {
        self.engine_rule_ambiguous
    }

    /// Consume the outcome and return the profile provider's not-found error, so
    /// a caller can map it onto its own error class.
    pub fn into_profile_not_found(self) -> Option<ResolveError> {
        self.profile_not_found
    }
}

/// Why a resolution did not produce a target.
#[derive(Debug)]
pub enum ResolutionError {
    /// A provider hit a hard error and the cascade aborted.
    Provider(ProviderError),
    /// Every provider declined.
    Unresolved(Unresolved),
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolutionError::Provider(e) => write!(f, "{e}"),
            ResolutionError::Unresolved(u) => match &u.profile_not_found {
                Some(e) => write!(f, "{e}"),
                None => write!(f, "no target could be resolved"),
            },
        }
    }
}

impl std::error::Error for ResolutionError {}

/// A source that can answer "what is this game's target identity?"
///
/// Yields either a stamped [`Target`], no answer (the cascade continues to the
/// next provider), or a [`ProviderError`] that aborts the cascade. A provider may
/// record a note while declining so a not-resolved outcome can explain itself.
pub trait TargetProvider {
    /// This provider's fixed position in the precedence order.
    fn precedence(&self) -> Precedence;

    /// Attempt to answer the request.
    ///
    /// # Errors
    ///
    /// [`ProviderError`] on a hard failure that must abort the cascade.
    fn provide(
        &self,
        request: &ResolutionRequest,
        notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError>;
}

/// Two providers reported the same [`Precedence`].
///
/// Refused at construction, because two providers at one position would make the
/// cascade's order depend on which was registered first, which is exactly the
/// registration-order independence the cascade guarantees (section 15.7). Each
/// precedence position holds one provider.
#[derive(Debug)]
pub struct DuplicatePrecedence(pub Precedence);

impl fmt::Display for DuplicatePrecedence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "two providers report precedence {:?}; each position holds one provider, \
             so the resolution order would depend on registration order",
            self.0
        )
    }
}

impl std::error::Error for DuplicatePrecedence {}

/// The cascade: a set of providers queried in precedence order.
pub struct TargetResolver {
    providers: Vec<Box<dyn TargetProvider>>,
}

impl TargetResolver {
    /// Build a resolver, sorting the providers by [`Precedence`] so the query
    /// order is imposed rather than dependent on the order given.
    ///
    /// # Errors
    ///
    /// [`DuplicatePrecedence`] if two providers report the same position. That
    /// is what makes the sort's stability irrelevant to the result: with one
    /// provider per position, the order is total and the query result cannot
    /// depend on the order the providers were passed in.
    pub fn new(
        mut providers: Vec<Box<dyn TargetProvider>>,
    ) -> Result<TargetResolver, DuplicatePrecedence> {
        providers.sort_by_key(|p| p.precedence());
        for pair in providers.windows(2) {
            if pair[0].precedence() == pair[1].precedence() {
                return Err(DuplicatePrecedence(pair[0].precedence()));
            }
        }
        Ok(TargetResolver { providers })
    }

    /// Resolve a request to the highest-precedence available target.
    ///
    /// Queries providers highest precedence first: the first to answer wins; a
    /// provider error aborts; if all decline, the result is
    /// [`ResolutionError::Unresolved`].
    ///
    /// # Errors
    ///
    /// [`ResolutionError::Provider`] on a hard provider failure, and
    /// [`ResolutionError::Unresolved`] when no provider answered.
    pub fn resolve(&self, request: &ResolutionRequest) -> Result<Target, ResolutionError> {
        let mut notes = ResolutionNotes::default();
        for provider in &self.providers {
            match provider.provide(request, &mut notes) {
                Ok(Some(target)) => return Ok(target),
                Ok(None) => continue,
                Err(e) => return Err(ResolutionError::Provider(e)),
            }
        }
        Err(ResolutionError::Unresolved(Unresolved {
            profile_not_found: notes.profile_not_found,
            engine_rule_ambiguous: notes.engine_rule_ambiguous,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FidelityTier, MatchPredicates, Provenance};
    use crate::target::{ObservedTarget, TargetOrigin};

    /// A provider that answers at a fixed precedence with a fixed tier, for
    /// exercising the engine without the real providers.
    struct Stub {
        precedence: Precedence,
        tier: Option<FidelityTier>,
        fail: bool,
    }

    impl Stub {
        fn answering(precedence: Precedence, tier: FidelityTier) -> Box<dyn TargetProvider> {
            Box::new(Stub {
                precedence,
                tier: Some(tier),
                fail: false,
            })
        }

        fn silent(precedence: Precedence) -> Box<dyn TargetProvider> {
            Box::new(Stub {
                precedence,
                tier: None,
                fail: false,
            })
        }

        fn failing(precedence: Precedence) -> Box<dyn TargetProvider> {
            Box::new(Stub {
                precedence,
                tier: None,
                fail: true,
            })
        }
    }

    impl TargetProvider for Stub {
        fn precedence(&self) -> Precedence {
            self.precedence
        }

        fn provide(
            &self,
            _request: &ResolutionRequest,
            _notes: &mut ResolutionNotes,
        ) -> Result<Option<Target>, ProviderError> {
            if self.fail {
                // A hard error, using the invalid-reference variant as a stand-in.
                return Err(ProviderError::Profile(ResolveError::InvalidReference {
                    reference: "stub".to_string(),
                }));
            }
            Ok(self.tier.map(|t| {
                Target::new(
                    t,
                    Provenance::new("stub".to_string(), None),
                    TargetOrigin::Observed(ObservedTarget::new(
                        1,
                        "x.exe".to_string(),
                        "C:\\x.exe".to_string(),
                        MatchPredicates::default(),
                    )),
                )
            }))
        }
    }

    fn empty_request<'a>(search: &'a SearchPath, bundled: &'a BundledSet) -> ResolutionRequest<'a> {
        ResolutionRequest::for_reference("t", search, bundled)
    }

    #[test]
    fn the_highest_precedence_provider_that_answers_wins() {
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let resolver = TargetResolver::new(vec![
            Stub::answering(Precedence::RuntimeObservation, FidelityTier::Observed),
            Stub::answering(Precedence::Profile, FidelityTier::Verified),
        ])
        .expect("distinct precedences");
        let target = resolver
            .resolve(&empty_request(&search, &bundled))
            .expect("resolves");
        assert_eq!(target.fidelity(), FidelityTier::Verified);
    }

    #[test]
    fn the_result_is_the_same_for_every_registration_order() {
        // The permutation test. Two providers can both answer; whichever order
        // they are registered in, the higher-precedence one wins.
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let orders: Vec<Vec<Box<dyn TargetProvider>>> = vec![
            vec![
                Stub::answering(Precedence::Profile, FidelityTier::Authored),
                Stub::answering(Precedence::RuntimeObservation, FidelityTier::Observed),
            ],
            vec![
                Stub::answering(Precedence::RuntimeObservation, FidelityTier::Observed),
                Stub::answering(Precedence::Profile, FidelityTier::Authored),
            ],
        ];
        for providers in orders {
            let resolver = TargetResolver::new(providers).expect("distinct precedences");
            let target = resolver
                .resolve(&empty_request(&search, &bundled))
                .expect("resolves");
            assert_eq!(
                target.fidelity(),
                FidelityTier::Authored,
                "the higher-precedence answer wins regardless of registration order"
            );
        }
    }

    #[test]
    fn a_lower_provider_answers_when_higher_ones_are_silent() {
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let resolver = TargetResolver::new(vec![
            Stub::silent(Precedence::Profile),
            Stub::silent(Precedence::HintDatabase),
            Stub::answering(Precedence::RuntimeObservation, FidelityTier::Observed),
        ])
        .expect("distinct precedences");
        let target = resolver
            .resolve(&empty_request(&search, &bundled))
            .expect("resolves");
        assert_eq!(target.fidelity(), FidelityTier::Observed);
    }

    #[test]
    fn a_provider_error_aborts_and_lower_providers_are_not_consulted() {
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let resolver = TargetResolver::new(vec![
            Stub::failing(Precedence::Profile),
            Stub::answering(Precedence::RuntimeObservation, FidelityTier::Observed),
        ])
        .expect("distinct precedences");
        match resolver.resolve(&empty_request(&search, &bundled)) {
            Err(ResolutionError::Provider(ProviderError::Profile(
                ResolveError::InvalidReference { .. },
            ))) => {}
            other => panic!("expected a hard provider error, got {other:?}"),
        }
    }

    #[test]
    fn all_silent_yields_a_named_unresolved_outcome() {
        let search = SearchPath::new();
        let bundled = BundledSet::empty();
        let resolver = TargetResolver::new(vec![
            Stub::silent(Precedence::Profile),
            Stub::silent(Precedence::RuntimeObservation),
        ])
        .expect("distinct precedences");
        match resolver.resolve(&empty_request(&search, &bundled)) {
            Err(ResolutionError::Unresolved(_)) => {}
            other => panic!("expected Unresolved, got {other:?}"),
        }
    }

    #[test]
    fn two_providers_at_one_precedence_are_refused() {
        // Determinism is guaranteed by one provider per position. Two at the same
        // position would make the result depend on registration order, so the
        // constructor refuses them rather than sorting them stably.
        let result = TargetResolver::new(vec![
            Stub::answering(Precedence::Profile, FidelityTier::Verified),
            Stub::answering(Precedence::Profile, FidelityTier::Authored),
        ]);
        match result {
            Err(DuplicatePrecedence(Precedence::Profile)) => {}
            _ => panic!("expected a DuplicatePrecedence error for the Profile position"),
        }
    }
}
