//! Central engine registry with enum dispatch.

use std::collections::HashSet;

use ontologos_core::{
    ConsistencyResult, DetectedProfileKind, EngineKind, EntityId, Profile, Reasoner, ResolvedRoute,
    RoleExpr,
};
use ontologos_el::ClassifyOutcome;
use ontologos_profile::resolve_route;

use super::alc::AlcAdapter;
use super::dl::DlAdapter;
use super::el::ElAdapter;
use super::hybrid::classify_hybrid_modules;
use super::rdfs::RdfsAdapter;
use super::rl::RlAdapter;
use super::swrl::SwrlAdapter;
use super::{ClassifyEngine, ConsistencyEngine, RoleQueryEngine};
use crate::error::{Error, Result};

pub(crate) struct EngineRegistry;

impl EngineRegistry {
    /// Resolve route from reasoner profile and ontology.
    pub(crate) fn resolve(reasoner: &Reasoner) -> Result<ResolvedRoute> {
        resolve_route(reasoner.profile(), reasoner.ontology()).map_err(|e| Error::El(e.into()))
    }

    /// Classify using the resolved engine route.
    pub(crate) fn classify(
        route: ResolvedRoute,
        reasoner: &mut Reasoner,
    ) -> Result<ClassifyOutcome> {
        if reasoner.profile() == Profile::Auto {
            return match route.kind {
                EngineKind::Dl => DlAdapter.classify(reasoner),
                EngineKind::Hybrid => classify_hybrid_modules(reasoner.ontology()),
                _ => ElAdapter.classify(reasoner),
            };
        }
        match route.kind {
            EngineKind::El => ElAdapter.classify(reasoner),
            EngineKind::Rdfs => RdfsAdapter.classify(reasoner),
            EngineKind::Rl => RlAdapter.classify(reasoner),
            EngineKind::Alc => AlcAdapter.classify(reasoner),
            EngineKind::Dl => DlAdapter.classify(reasoner),
            EngineKind::Swrl => SwrlAdapter.classify(reasoner),
            EngineKind::Hybrid => classify_hybrid_modules(reasoner.ontology()),
        }
    }

    /// Check consistency using the resolved engine route.
    pub(crate) fn check_consistency(
        route: ResolvedRoute,
        reasoner: &Reasoner,
    ) -> Result<ConsistencyResult> {
        match route.kind {
            EngineKind::El => ElAdapter.check_consistency(reasoner),
            EngineKind::Rdfs => RdfsAdapter.check_consistency(reasoner),
            EngineKind::Rl => RlAdapter.check_consistency(reasoner),
            EngineKind::Alc => AlcAdapter.check_consistency(reasoner),
            EngineKind::Dl => DlAdapter.check_consistency(reasoner),
            EngineKind::Swrl => SwrlAdapter.check_consistency(reasoner),
            EngineKind::Hybrid => Self::check_consistency_hybrid(route, reasoner),
        }
    }

    /// Check consistency using the resolved engine route (bool; errors if incomplete).
    pub(crate) fn is_consistent(route: ResolvedRoute, reasoner: &Reasoner) -> Result<bool> {
        Self::check_consistency(route, reasoner)?
            .into_bool()
            .map_err(Error::Core)
    }

    /// Sub-object-property query using the resolved engine route.
    pub(crate) fn sub_object_properties(
        route: ResolvedRoute,
        reasoner: &Reasoner,
        property: EntityId,
        direct: bool,
    ) -> Result<HashSet<RoleExpr>> {
        if route.capabilities.role_query || route.kind == EngineKind::Hybrid {
            let dl_route = if route.kind == EngineKind::Hybrid {
                route_with_dl_role_query(route)
            } else {
                route
            };
            return match dl_route.kind {
                EngineKind::Alc => AlcAdapter.sub_object_properties(reasoner, property, direct),
                EngineKind::Dl | EngineKind::Swrl => {
                    DlAdapter.sub_object_properties(reasoner, property, direct)
                }
                _ => ElAdapter.sub_object_properties(reasoner, property, direct),
            };
        }
        ElAdapter.sub_object_properties(reasoner, property, direct)
    }

    fn check_consistency_hybrid(
        route: ResolvedRoute,
        reasoner: &Reasoner,
    ) -> Result<ConsistencyResult> {
        match route.detected {
            Some(DetectedProfileKind::Dl) => DlAdapter.check_consistency(reasoner),
            Some(DetectedProfileKind::Rl) => RlAdapter.check_consistency(reasoner),
            Some(DetectedProfileKind::El) | Some(DetectedProfileKind::Ql) => {
                ElAdapter.check_consistency(reasoner)
            }
            None => Err(Error::El(ontologos_el::Error::Message(
                "no profile detected".into(),
            ))),
        }
    }
}

fn route_with_dl_role_query(route: ResolvedRoute) -> ResolvedRoute {
    ResolvedRoute {
        kind: EngineKind::Dl,
        capabilities: route.capabilities,
        detected: route.detected,
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::{EngineKind, Ontology, Profile};

    use super::*;

    #[test]
    fn resolve_explicit_profiles() {
        let ontology = Ontology::default();
        for (profile, kind) in [
            (Profile::El, EngineKind::El),
            (Profile::Rdfs, EngineKind::Rdfs),
            (Profile::Rl, EngineKind::Rl),
            (Profile::Alc, EngineKind::Alc),
            (Profile::Dl, EngineKind::Dl),
            (Profile::DlPreview, EngineKind::Dl),
            (Profile::Swrl, EngineKind::Swrl),
        ] {
            let reasoner = Reasoner::builder()
                .profile(profile)
                .build(ontology.clone())
                .unwrap();
            let route = EngineRegistry::resolve(&reasoner).unwrap();
            assert_eq!(route.kind, kind, "profile {profile:?}");
        }
    }

    #[test]
    fn resolve_auto_el_fixture() {
        let ontology = Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .class("http://example.org/B")
            .unwrap()
            .subclass_of("http://example.org/A", "http://example.org/B")
            .unwrap()
            .build()
            .unwrap();
        let reasoner = Reasoner::builder()
            .profile(Profile::Auto)
            .build(ontology)
            .unwrap();
        let route = EngineRegistry::resolve(&reasoner).unwrap();
        assert_eq!(route.kind, EngineKind::El);
        assert!(route.detected.is_some());
    }
}
