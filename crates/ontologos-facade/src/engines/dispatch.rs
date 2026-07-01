//! Enum dispatch to profile engines (replaces adapter/registry layer).

use std::collections::HashSet;

use ontologos_bridge::has_bottom_chain_violation;
use ontologos_core::{
    ClassExpr, ConsistencyResult, DlAxiom, EngineKind, EntityId, Profile, Reasoner, ResolvedRoute,
    RoleExpr,
};
use ontologos_dl::DlEngine;
use ontologos_el::ElClassifier;
use ontologos_profile::resolve_route;
use ontologos_rl::rdfs::RdfsEngine;
use ontologos_rl::RlEngine;

use crate::error::{Error, Result};
use crate::lookup::index_sub_object_properties;
use crate::outcome::ClassifyOutcome;

use super::hybrid::classify_hybrid_modules;

pub(crate) fn resolve(reasoner: &Reasoner) -> Result<ResolvedRoute> {
    resolve_route(reasoner.profile(), reasoner.ontology()).map_err(|e| Error::El(e.into()))
}

pub(crate) fn classify(route: ResolvedRoute, reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    match route.kind {
        EngineKind::El => classify_el(reasoner),
        EngineKind::Rdfs => Ok(ClassifyOutcome::Rdfs(
            ontologos_rl::rdfs::materialize_routed(reasoner).map_err(Error::Rl)?,
        )),
        EngineKind::Rl => Ok(ClassifyOutcome::Rl(
            ontologos_rl::saturate_routed(reasoner).map_err(Error::Rl)?,
        )),
        EngineKind::Alc | EngineKind::Dl => classify_dl(reasoner),
        EngineKind::Swrl => classify_dl(reasoner),
        EngineKind::Hybrid => classify_hybrid_modules(reasoner.ontology()),
    }
}

pub(crate) fn check_consistency(
    route: ResolvedRoute,
    reasoner: &Reasoner,
) -> Result<ConsistencyResult> {
    match route.kind {
        EngineKind::El => {
            if ontology_needs_dl_abox_consistency(reasoner.ontology()) {
                return DlEngine
                    .check_consistency(reasoner.ontology(), reasoner.config().budget_secs)
                    .map_err(Error::Dl);
            }
            let consistent = ontologos_el::ElEngine
                .is_consistent(reasoner.ontology())
                .map_err(Error::El)?;
            Ok(consistency_from_bool(consistent))
        }
        EngineKind::Rdfs => {
            let consistent = {
                let mut working = reasoner.ontology().clone();
                let report = RdfsEngine::new()
                    .materialize(&mut working)
                    .map_err(Error::Rl)?;
                report.clashes.is_empty()
            };
            Ok(consistency_from_bool(consistent))
        }
        EngineKind::Rl => check_consistency_rl(reasoner),
        EngineKind::Alc | EngineKind::Dl | EngineKind::Swrl => DlEngine
            .check_consistency(reasoner.ontology(), reasoner.config().budget_secs)
            .map_err(Error::Dl),
        EngineKind::Hybrid => check_consistency_hybrid(route, reasoner),
    }
}

pub(crate) fn sub_object_properties(
    route: ResolvedRoute,
    reasoner: &Reasoner,
    property: EntityId,
    direct: bool,
) -> Result<HashSet<RoleExpr>> {
    let kind = if route.kind == EngineKind::Hybrid {
        EngineKind::Dl
    } else {
        route.kind
    };
    match kind {
        EngineKind::Alc | EngineKind::Dl | EngineKind::Swrl => {
            let role = RoleExpr::Atomic(property);
            DlEngine
                .sub_object_properties(reasoner.ontology(), &role, direct)
                .map_err(Error::Dl)
        }
        _ => Ok(index_sub_object_properties(
            reasoner.ontology(),
            property,
            direct,
        )),
    }
}

fn classify_el(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    if reasoner.config().incremental {
        Ok(ClassifyOutcome::Taxonomy(
            ontologos_el::classify_with_report(reasoner)?.taxonomy,
        ))
    } else {
        Ok(ClassifyOutcome::Taxonomy(
            ElClassifier::new()
                .classify(reasoner.ontology())
                .map_err(Error::El)?,
        ))
    }
}

fn classify_dl(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    let taxonomy = if reasoner.profile() == Profile::DlPreview {
        ontologos_dl::DlClassifier::new()
            .preview(true)
            .classify(reasoner.ontology())
            .map_err(Error::Dl)?
    } else {
        DlEngine
            .classify(reasoner.ontology())
            .map_err(Error::Dl)?
    };
    Ok(ClassifyOutcome::Taxonomy(taxonomy))
}

fn check_consistency_rl(reasoner: &Reasoner) -> Result<ConsistencyResult> {
    let mut working = reasoner.ontology().clone();
    let report = RlEngine::new(1)
        .saturate(&mut working)
        .map_err(Error::Rl)?;
    if !report.clashes.is_empty() || has_bottom_chain_violation(&working) {
        return Ok(ConsistencyResult::inconsistent());
    }
    let consistent = ontologos_rl::abox::is_abox_consistent(&working).map_err(Error::Rl)?;
    Ok(consistency_from_bool(consistent))
}

fn check_consistency_hybrid(
    _route: ResolvedRoute,
    reasoner: &Reasoner,
) -> Result<ConsistencyResult> {
    DlEngine
        .check_consistency(reasoner.ontology(), reasoner.config().budget_secs)
        .map_err(Error::Dl)
}

fn ontology_needs_dl_abox_consistency(ontology: &ontologos_core::Ontology) -> bool {
    ontology.dl().axioms().any(|axiom| {
        let DlAxiom::ClassAssertion { class: ce, .. } = axiom else {
            return false;
        };
        ontology
            .dl()
            .ce(*ce)
            .is_some_and(|expr| !matches!(expr, ClassExpr::Atomic(_)))
    })
}

fn consistency_from_bool(consistent: bool) -> ConsistencyResult {
    if consistent {
        ConsistencyResult::consistent()
    } else {
        ConsistencyResult::inconsistent()
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Ontology, Profile};

    use super::*;

    #[test]
    fn resolve_explicit_profiles() {
        let ontology = Ontology::default();
        for (profile, kind) in [
            (Profile::El, EngineKind::El),
            (Profile::Rdfs, EngineKind::Rdfs),
            (Profile::Rl, EngineKind::Rl),
            (Profile::Alc, EngineKind::Dl),
            (Profile::Dl, EngineKind::Dl),
            (Profile::DlPreview, EngineKind::Dl),
            (Profile::Swrl, EngineKind::Dl),
        ] {
            let reasoner = Reasoner::builder()
                .profile(profile)
                .build(ontology.clone())
                .unwrap();
            let route = resolve(&reasoner).unwrap();
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
        let route = resolve(&reasoner).unwrap();
        assert_eq!(route.kind, EngineKind::El);
        assert!(route.detected.is_some());
    }
}
