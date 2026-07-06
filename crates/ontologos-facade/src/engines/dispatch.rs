//! Enum dispatch to profile engines (replaces adapter/registry layer).

use std::collections::HashSet;

use ontologos_bridge::has_bottom_chain_violation;
use ontologos_core::{
    ClassExpr, ConsistencyResult, DlAxiom, EngineKind, EntityId, Ontology, Profile, Reasoner,
    ResolvedRoute, RoleExpr,
};
use ontologos_dl::DlEngine;
use ontologos_el::ElClassifier;
use ontologos_profile::resolve_route;
use ontologos_rl::RlEngine;
use ontologos_rl::rdfs::RdfsEngine;
use ontologos_swrl::SwrlEngine;

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
        EngineKind::Swrl => classify_swrl(reasoner),
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
            match ontologos_el::ElEngine.is_consistent(reasoner.ontology()) {
                Ok(consistent) => Ok(consistency_from_bool(consistent)),
                Err(e) => {
                    tracing::warn!("EL consistency check incomplete: {e}");
                    Ok(ConsistencyResult::incomplete())
                }
            }
        }
        EngineKind::Rdfs => {
            let mut working = reasoner.ontology().clone();
            match RdfsEngine::new().materialize(&mut working) {
                Ok(report) => Ok(consistency_from_bool(
                    !ontologos_rl::clashes_indicate_inconsistency(&report.clashes),
                )),
                Err(e) => {
                    tracing::warn!("RDFS consistency check incomplete: {e}");
                    Ok(ConsistencyResult::incomplete())
                }
            }
        }
        EngineKind::Rl => check_consistency_rl(reasoner),
        EngineKind::Alc | EngineKind::Dl => DlEngine
            .check_consistency(reasoner.ontology(), reasoner.config().budget_secs)
            .map_err(Error::Dl),
        EngineKind::Swrl => match SwrlEngine.is_consistent(reasoner.ontology()) {
            Ok(consistent) => Ok(consistency_from_bool(consistent)),
            Err(e) => {
                tracing::warn!("SWRL consistency check incomplete: {e}");
                Ok(ConsistencyResult::incomplete())
            }
        },
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
    if reasoner.config().incremental || reasoner.profile() == Profile::El {
        Ok(ClassifyOutcome::Taxonomy(
            ontologos_el::classify_routed(reasoner)?.taxonomy,
        ))
    } else {
        let report = ElClassifier::new()
            .classify_with_options(reasoner.ontology(), reasoner.config().explanations)
            .map_err(Error::El)?;
        reasoner.clear_session();
        Ok(ClassifyOutcome::Taxonomy(report.taxonomy))
    }
}

fn classify_swrl(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    let (taxonomy, _report) = SwrlEngine
        .classify_with_swrl(reasoner.ontology())
        .map_err(Error::Swrl)?;
    Ok(ClassifyOutcome::Taxonomy(taxonomy))
}

fn classify_dl(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    if reasoner.config().incremental {
        tracing::warn!(
            "incremental classification is not supported for OWL DL; performing full classify"
        );
    }
    let budget = reasoner.config().budget_secs;
    let preview = reasoner.profile() == Profile::DlPreview;
    let dl_classify = move |ontology: &Ontology| {
        if preview {
            ontologos_dl::DlClassifier::new()
                .preview(true)
                .classify(ontology)
        } else {
            ontologos_dl::classify(ontology)
        }
    };
    let taxonomy_result = if budget.is_some() {
        let ontology = reasoner.ontology().clone();
        ontologos_dl::run_bounded(budget, move || dl_classify(&ontology))?
    } else {
        dl_classify(reasoner.ontology())
    };
    Ok(ClassifyOutcome::Taxonomy(
        taxonomy_result.map_err(Error::Dl)?,
    ))
}

fn check_consistency_rl(reasoner: &Reasoner) -> Result<ConsistencyResult> {
    let mut working = reasoner.ontology().clone();
    let report = RlEngine::new(1).saturate(&mut working).map_err(Error::Rl)?;
    if ontologos_rl::clashes_indicate_inconsistency(&report.clashes)
        || has_bottom_chain_violation(&working)
    {
        return Ok(ConsistencyResult::inconsistent());
    }
    let abox_ok = ontologos_rl::abox::is_abox_consistent(&working).map_err(Error::Rl)?;
    if !abox_ok {
        return Ok(ConsistencyResult::inconsistent());
    }
    let taxonomy = ontologos_el::ElClassifier::new()
        .classify(&working)
        .map_err(Error::El)?;
    if !taxonomy.unsatisfiable.is_empty() {
        return Ok(ConsistencyResult::inconsistent());
    }
    Ok(ConsistencyResult::consistent())
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
            (Profile::Swrl, EngineKind::Swrl),
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
