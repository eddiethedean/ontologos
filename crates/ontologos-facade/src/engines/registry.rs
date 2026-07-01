//! Central engine registry with enum dispatch.

use std::collections::HashSet;

use ontologos_core::{
    DetectedProfileKind, EngineKind, EntityId, Profile, Reasoner, ResolvedRoute, RoleExpr,
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
    pub(crate) fn is_consistent(route: ResolvedRoute, reasoner: &Reasoner) -> Result<bool> {
        match route.kind {
            EngineKind::El => ElAdapter.is_consistent(reasoner),
            EngineKind::Rdfs => RdfsAdapter.is_consistent(reasoner),
            EngineKind::Rl => RlAdapter.is_consistent(reasoner),
            EngineKind::Alc => AlcAdapter.is_consistent(reasoner),
            EngineKind::Dl => DlAdapter.is_consistent(reasoner),
            EngineKind::Swrl => SwrlAdapter.is_consistent(reasoner),
            EngineKind::Hybrid => Self::is_consistent_hybrid(route, reasoner),
        }
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

    fn is_consistent_hybrid(route: ResolvedRoute, reasoner: &Reasoner) -> Result<bool> {
        match route.detected {
            Some(DetectedProfileKind::Dl) => DlAdapter.is_consistent(reasoner),
            Some(DetectedProfileKind::Rl) => RlAdapter.is_consistent(reasoner),
            Some(DetectedProfileKind::El) | Some(DetectedProfileKind::Ql) => {
                ElAdapter.is_consistent(reasoner)
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
