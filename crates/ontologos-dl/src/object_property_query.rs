//! Object-property query API with saturation-derived tableau seed.

use std::collections::HashSet;

use ontologos_alc::{
    augment_for_role_classification, classify_object_property_on_augmented, DlOntology,
    PreparedRoleSurrogateContext, TableauSeed,
};
use ontologos_core::{Ontology, RoleExpr};

use crate::classify::build_tableau_seed;
use crate::ria::RoleHierarchy;
use crate::saturation::saturate;
use crate::Error;

/// Reusable prepared state for object-property queries (avoids repeated augmentation).
pub struct RolePropertyQueryContext {
    ctx: PreparedRoleSurrogateContext,
}

impl RolePropertyQueryContext {
    /// Build augmented surrogate ontology, saturation seed, and query context once.
    pub fn prepare(ontology: &Ontology) -> Result<Self, Error> {
        let (dl, role_map, seed) = build_augmented_role_query(ontology)?;
        let ctx = PreparedRoleSurrogateContext::from_augmented(dl, role_map, &seed)
            .map_err(Error::Alc)?;
        Ok(Self { ctx })
    }

    /// OWL API `getSubObjectProperties`.
    pub fn sub_object_property_expressions(
        &self,
        property: &RoleExpr,
        direct: bool,
    ) -> Result<HashSet<RoleExpr>, Error> {
        self.ctx
            .sub_object_property_expressions(property, direct)
            .map_err(Error::Alc)
    }

    /// OWL API `getEquivalentObjectProperties`.
    pub fn equivalent_object_property_expressions(
        &self,
        property: &RoleExpr,
    ) -> Result<HashSet<RoleExpr>, Error> {
        self.ctx
            .equivalent_object_property_expressions(property)
            .map_err(Error::Alc)
    }
}

fn build_augmented_role_query(
    ontology: &Ontology,
) -> Result<
    (
        DlOntology,
        std::collections::HashMap<RoleExpr, ontologos_core::EntityId>,
        TableauSeed,
    ),
    Error,
> {
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
    RolePropertyQueryContext::prepare(ontology)?.equivalent_object_property_expressions(property)
}

/// OWL API `getSubObjectProperties`.
pub fn sub_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
    direct: bool,
) -> Result<HashSet<RoleExpr>, Error> {
    RolePropertyQueryContext::prepare(ontology)?.sub_object_property_expressions(property, direct)
}

/// OWL API `getInverseObjectProperties`.
pub fn inverse_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
) -> Result<HashSet<RoleExpr>, Error> {
    let inverse = match property {
        RoleExpr::Atomic(id) => RoleExpr::Inverse(*id),
        RoleExpr::Inverse(id) => RoleExpr::Atomic(*id),
    };
    RolePropertyQueryContext::prepare(ontology)?.equivalent_object_property_expressions(&inverse)
}
