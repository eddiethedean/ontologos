//! Object-property query API with saturation-derived tableau seed.

use std::collections::HashSet;

use ontologos_alc::{
    augment_for_role_classification, classify_object_property_on_augmented,
    equivalent_object_property_on_augmented, sub_object_property_on_augmented, DlOntology,
    TableauSeed,
};
use ontologos_core::{Ontology, RoleExpr};

use crate::classify::build_tableau_seed;
use crate::ria::RoleHierarchy;
use crate::saturation::saturate;
use crate::Error;

fn build_augmented_role_query(
    ontology: &Ontology,
) -> Result<(DlOntology, std::collections::HashMap<RoleExpr, ontologos_core::EntityId>, TableauSeed), Error>
{
    let (augmented, role_map) = augment_for_role_classification(ontology).map_err(Error::Alc)?;
    let dl = DlOntology::from_ontology(&augmented).map_err(Error::Alc)?;
    let roles = RoleHierarchy::from_clauses(dl.clauses());
    let facts = saturate(&augmented, dl.clauses(), &roles)?;
    let seed = build_tableau_seed(&augmented, &dl, &facts, &roles)?;
    Ok((dl, role_map, seed))
}

/// Classify object-property expressions into equivalence classes.
pub fn classify_object_property_expressions(
    ontology: &Ontology,
) -> Result<Vec<HashSet<RoleExpr>>, Error> {
    let (dl, role_map, seed) = build_augmented_role_query(ontology)?;
    classify_object_property_on_augmented(dl, role_map, &seed).map_err(Error::Alc)
}

/// OWL API `getEquivalentObjectProperties`.
pub fn equivalent_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
) -> Result<HashSet<RoleExpr>, Error> {
    let (dl, role_map, seed) = build_augmented_role_query(ontology)?;
    equivalent_object_property_on_augmented(dl, role_map, property, &seed).map_err(Error::Alc)
}

/// OWL API `getSubObjectProperties`.
pub fn sub_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
    direct: bool,
) -> Result<HashSet<RoleExpr>, Error> {
    let (dl, role_map, seed) = build_augmented_role_query(ontology)?;
    sub_object_property_on_augmented(dl, role_map, property, direct, &seed).map_err(Error::Alc)
}

/// OWL API `getInverseObjectProperties`.
pub fn inverse_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
) -> Result<HashSet<RoleExpr>, Error> {
    let (dl, role_map, seed) = build_augmented_role_query(ontology)?;
    let inverse = match property {
        RoleExpr::Atomic(id) => RoleExpr::Inverse(*id),
        RoleExpr::Inverse(id) => RoleExpr::Atomic(*id),
    };
    equivalent_object_property_on_augmented(dl, role_map, &inverse, &seed).map_err(Error::Alc)
}
