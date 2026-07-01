//! Shared helpers for HermiT-ported conformance tests.
//!
//! **Contributors:** start with [HermiT burndown guide](../../docs/guides/hermit-burndown.md)
//! and `bash benchmarks/scripts/hermit-burndown.sh status`.
//!
//! HermiT fixtures are vendored under `benchmarks/data/hermit/` for CI. A full
//! HermiT source tree at `HermiT/` (gitignored) or `ONTOLOGOS_HERMIT_ROOT` is
//! also supported.

mod catalog;
mod catalog_error;

pub use catalog_error::CatalogError;

pub use catalog::{
    AuditOptions, HermitCase, ParityMetrics, PlannedBacklogAudit, PlannedBacklogSummary,
    PlannedJavaAudit, PlannedJavaCategory, PlannedWgAudit, PlannedWgCategory, WgCase, WgFailure,
    WgFailureBucket, audit_planned_backlog, audit_planned_backlog_with, check_axiom_case,
    check_axiom_case_bounded, check_logical_entailment, check_wg_case,
    ensure_concurrent_scan_defaults, load_catalog, load_wg_catalog, parity_metrics,
    promoted_axiom_ids_path, promoted_wg_ids_path, read_catalog_file, read_promoted_axiom_ids,
    read_promoted_wg_ids, read_wg_catalog_file, refresh_catalog_file_cache, run_hermit_case,
    run_wg_case, scan_all_passing_axiom_cases, scan_all_passing_wg_cases, scan_all_wg_failures,
    scan_planned_dl_failures, scan_planned_engine_failures, scan_planned_passing_wg_cases,
    scan_planned_wg_failures, scan_promotable_axiom_cases, scan_promoted_axiom_failures,
    scan_promoted_wg_failures, scan_unpromoted_passing_axiom_cases,
    scan_unpromoted_passing_wg_cases, scan_wg_failures, stable_promoted_axiom_ids,
    sync_promoted_lists, wg_case_short_id, write_promoted_axiom_ids, write_promoted_wg_ids,
};

use std::path::{Path, PathBuf};

/// Default namespace used by HermiT's `AbstractOntologyTest` (`file:/c/test.owl#`).
pub const HERMIT_DEFAULT_NS: &str = "file:/c/test.owl#";

/// OntoLogos-style namespace for inlined Tier-A ports.
pub const PORT_NS: &str = "http://example.org/hermit-port#";

/// Resolve the HermiT source tree, if present.
#[must_use]
pub fn hermit_root() -> Option<PathBuf> {
    if let Ok(env) = std::env::var("ONTOLOGOS_HERMIT_ROOT") {
        let path = PathBuf::from(env);
        if is_hermit_tree(&path) {
            return path.canonicalize().ok().or(Some(path));
        }
    }

    let default = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../HermiT");
    if is_hermit_tree(&default) {
        return default.canonicalize().ok().or(Some(default));
    }
    None
}

fn is_hermit_tree(path: &Path) -> bool {
    path.join("src/test/java/org/semanticweb/HermiT").is_dir()
        || path.join("project/test/org/semanticweb/HermiT").is_dir()
}

fn hermit_test_subdir() -> &'static str {
    "src/test/java/org/semanticweb/HermiT"
}

fn hermit_resource_subdir() -> &'static str {
    "src/test/resources/org/semanticweb/HermiT"
}

/// Path under the HermiT Java test tree (owlcs/hermit-reasoner layout).
#[must_use]
pub fn hermit_test_path(relative: &str) -> Option<PathBuf> {
    hermit_root().and_then(|root| {
        let modern = root.join(hermit_test_subdir()).join(relative);
        if modern.exists() {
            return Some(modern);
        }
        let legacy = root
            .join("project/test/org/semanticweb/HermiT")
            .join(relative);
        legacy.exists().then_some(legacy)
    })
}

/// Path under the HermiT test resources tree.
#[must_use]
pub fn hermit_resource_path(relative: &str) -> Option<PathBuf> {
    hermit_root().and_then(|root| {
        let modern = root.join(hermit_resource_subdir()).join(relative);
        if modern.exists() {
            return Some(modern);
        }
        let legacy = root
            .join("project/test/org/semanticweb/HermiT")
            .join(relative);
        legacy.exists().then_some(legacy)
    })
}

/// Vendored `ClassificationTest` fixtures under `benchmarks/data/hermit/`.
#[must_use]
pub fn vendored_hermit_test_path(relative: &str) -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(relative);
    path.exists().then_some(path)
}

/// Resolve a HermiT test fixture from vendored benchmarks, resources, or Java tree.
#[must_use]
pub fn classification_fixture_path(relative: &str) -> Option<PathBuf> {
    vendored_hermit_test_path(relative)
        .or_else(|| hermit_resource_path(relative))
        .or_else(|| hermit_test_path(relative))
}

/// Returns true when optional Tier-B HermiT fixture tests should run.
#[must_use]
pub fn hermit_available() -> bool {
    hermit_root().is_some()
}

/// Returns true when `ClassificationTest` fixtures are available (vendored or local HermiT).
#[must_use]
pub fn classification_fixtures_available() -> bool {
    [
        "reasoner/res/pizza.xml",
        "reasoner/res/wine.xml",
        "reasoner/res/galen-ians-full-undoctored.xml",
        "reasoner/res/propreo.xml",
        "reasoner/res/dolce_all.xml",
    ]
    .iter()
    .all(|relative| classification_fixture_path(relative).is_some())
}

/// Assert `subclass` has a direct or indirect super-class `superclass`.
pub fn assert_subsumed(
    ontology: &ontologos_core::Ontology,
    subclass: &str,
    superclass: &str,
) -> bool {
    let Some(sub) = ontology.lookup_entity(subclass) else {
        return false;
    };
    let Some(sup) = ontology.lookup_entity(superclass) else {
        return false;
    };
    if ontology.direct_superclasses(sub).contains(&sup) {
        return true;
    }
    transitive_superclasses(ontology, sub, sup)
}

/// Assert `sub_property` is a direct or indirect sub-property of `super_property`.
pub fn assert_subproperty(
    ontology: &ontologos_core::Ontology,
    sub_property: &str,
    super_property: &str,
) -> bool {
    let Some(sub) = ontology.lookup_entity(sub_property) else {
        return false;
    };
    let Some(sup) = ontology.lookup_entity(super_property) else {
        return false;
    };
    if ontology.direct_superproperties(sub).contains(&sup) {
        return true;
    }
    transitive_superproperties(ontology, sub, sup)
}

/// Assert `sub_property` is a **direct** sub-property of `super_property` (no transitive walk).
pub fn assert_direct_subproperty(
    ontology: &ontologos_core::Ontology,
    sub_property: &str,
    super_property: &str,
) -> bool {
    let Some(sub) = ontology.lookup_entity(sub_property) else {
        return false;
    };
    let Some(sup) = ontology.lookup_entity(super_property) else {
        return false;
    };
    ontology.direct_superproperties(sub).contains(&sup)
}

/// Assert `individual` has an asserted or inferred class type `class` (direct or via `subClassOf`).
pub fn assert_typed(ontology: &ontologos_core::Ontology, individual: &str, class: &str) -> bool {
    let Some(ind) = ontology.lookup_entity(individual) else {
        return false;
    };
    let Some(class_id) = ontology.lookup_entity(class) else {
        return false;
    };
    ontology
        .classes_of(ind)
        .iter()
        .any(|&c| c == class_id || assert_subsumed_by_id(ontology, c, class_id))
}

/// Assert an `ObjectPropertyAssertion` is present after materialization.
pub fn assert_object_property_assertion(
    ontology: &ontologos_core::Ontology,
    subject: &str,
    property: &str,
    object: &str,
) -> bool {
    let Some(subject_id) = ontology.lookup_entity(subject) else {
        return false;
    };
    let Some(property_id) = ontology.lookup_entity(property) else {
        return false;
    };
    let Some(object_id) = ontology.lookup_entity(object) else {
        return false;
    };
    ontology
        .object_assertions_of(subject_id)
        .iter()
        .any(|&(p, o)| p == property_id && o == object_id)
}

/// Assert a property characteristic axiom is present in the ontology.
pub fn has_property_characteristic(
    ontology: &ontologos_core::Ontology,
    property: &str,
    kind: PropertyCharacteristic,
) -> bool {
    let Some(property_id) = lookup_entity_flexible(ontology, property) else {
        return false;
    };
    match kind {
        PropertyCharacteristic::Functional => {
            ontology
                .index()
                .functional_properties()
                .contains(&property_id)
                || ontology.dl().axioms().any(|axiom| {
                    matches!(
                        axiom,
                        ontologos_core::DlAxiom::FunctionalDataProperty(p) if *p == property_id
                    )
                })
        }
        PropertyCharacteristic::InverseFunctional => ontology
            .index()
            .inverse_functional_properties()
            .contains(&property_id),
        PropertyCharacteristic::Symmetric => ontology
            .index()
            .symmetric_properties()
            .contains(&property_id),
        PropertyCharacteristic::Transitive => ontology
            .index()
            .transitive_properties()
            .contains(&property_id),
        PropertyCharacteristic::Reflexive => ontology
            .index()
            .reflexive_properties()
            .contains(&property_id),
        PropertyCharacteristic::Asymmetric => ontology
            .index()
            .asymmetric_properties()
            .contains(&property_id),
        PropertyCharacteristic::Irreflexive => ontology
            .index()
            .irreflexive_properties()
            .contains(&property_id),
    }
}

/// OWL object property characteristics used in HermiT ports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyCharacteristic {
    Functional,
    InverseFunctional,
    Symmetric,
    Transitive,
    Reflexive,
    Asymmetric,
    Irreflexive,
}

fn assert_subsumed_by_id(
    ontology: &ontologos_core::Ontology,
    subclass: ontologos_core::EntityId,
    superclass: ontologos_core::EntityId,
) -> bool {
    if subclass == superclass {
        return true;
    }
    transitive_superclasses(ontology, subclass, superclass)
}

fn transitive_superclasses(
    ontology: &ontologos_core::Ontology,
    subclass: ontologos_core::EntityId,
    target: ontologos_core::EntityId,
) -> bool {
    let mut stack: Vec<ontologos_core::EntityId> = ontology.direct_superclasses(subclass).to_vec();
    let mut seen = std::collections::HashSet::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current) {
            continue;
        }
        if current == target {
            return true;
        }
        stack.extend_from_slice(ontology.direct_superclasses(current));
    }
    false
}

fn transitive_superproperties(
    ontology: &ontologos_core::Ontology,
    sub_property: ontologos_core::EntityId,
    target: ontologos_core::EntityId,
) -> bool {
    let mut stack: Vec<ontologos_core::EntityId> =
        ontology.direct_superproperties(sub_property).to_vec();
    let mut seen = std::collections::HashSet::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current) {
            continue;
        }
        if current == target {
            return true;
        }
        stack.extend_from_slice(ontology.direct_superproperties(current));
    }
    false
}

fn lookup_entity_flexible(
    ontology: &ontologos_core::Ontology,
    iri: &str,
) -> Option<ontologos_core::EntityId> {
    if let Some(id) = ontology.lookup_entity(iri) {
        return Some(id);
    }
    let local = iri.rsplit('#').next().unwrap_or(iri);
    ontology.entities().iter().find_map(|(id, record)| {
        ontology
            .resolve_iri(record.iri)
            .ok()
            .and_then(|entity_iri| entity_iri.rsplit('#').next())
            .filter(|name| name.eq_ignore_ascii_case(local))
            .map(|_| id)
    })
}
