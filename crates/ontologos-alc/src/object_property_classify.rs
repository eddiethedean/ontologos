//! Object-property taxonomy via concept surrogates (HermiT `classifyObjectProperties`).

use std::collections::{HashMap, HashSet};

use ontologos_core::{
    ClassExpr, DlAxiom, EntityId, EntityKind, Ontology, RoleExpr, Taxonomy,
};

use crate::tableau::{classify, role_expression_subsumes};
use crate::Error;

const FRESH_CLASS: &str = "urn:ontologos:internal:fresh-concept";
const FRESH_INDIVIDUAL: &str = "urn:ontologos:internal:fresh-individual";
const SURROGATE_NS: &str = "urn:ontologos:internal:role-surrogate:";

fn collect_relevant_role_expressions(ontology: &Ontology) -> HashSet<RoleExpr> {
    let mut roles = HashSet::new();
    for (id, record) in ontology.entities().iter() {
        if record.kind == EntityKind::ObjectProperty {
            roles.insert(RoleExpr::Atomic(id));
            roles.insert(RoleExpr::Inverse(id));
        }
    }
    for axiom in ontology.dl().axioms() {
        match axiom {
            DlAxiom::SubObjectPropertyOf { sub, sup } => {
                roles.insert(sub.clone());
                roles.insert(sup.clone());
            }
            DlAxiom::SubObjectPropertyChain {
                chain,
                super_property,
            } => {
                roles.extend(chain.iter().cloned());
                roles.insert(super_property.clone());
            }
            _ => {}
        }
    }
    roles
}

fn role_surrogate_iri(ontology: &Ontology, role: &RoleExpr) -> Result<String, Error> {
    let label = match role {
        RoleExpr::Atomic(id) => ontology.resolve_iri(ontology.entity(*id)?.iri)?.to_string(),
        RoleExpr::Inverse(id) => {
            let iri = ontology.resolve_iri(ontology.entity(*id)?.iri)?;
            format!("inv#{iri}")
        }
    };
    Ok(format!("{SURROGATE_NS}{label}"))
}

fn build_role_classification_ontology(
    base: &Ontology,
) -> Result<(Ontology, HashMap<RoleExpr, EntityId>), Error> {
    let mut ontology = base.clone();
    let fresh_class = ontology
        .entity_id(FRESH_CLASS, EntityKind::Class)
        .map_err(Error::Core)?;
    let fresh_individual = ontology
        .entity_id(FRESH_INDIVIDUAL, EntityKind::Individual)
        .map_err(Error::Core)?;
    let fresh_ce = ontology
        .dl_mut()
        .intern_ce(ClassExpr::Atomic(fresh_class));
    ontology.dl_mut().push_axiom(DlAxiom::ClassAssertion {
        individual: fresh_individual,
        class: fresh_ce,
    });

    let mut role_to_surrogate = HashMap::new();
    for role in collect_relevant_role_expressions(&ontology) {
        let surrogate_iri = role_surrogate_iri(&ontology, &role)?;
        let surrogate = ontology
            .entity_id(&surrogate_iri, EntityKind::Class)
            .map_err(Error::Core)?;
        let surrogate_ce = ontology
            .dl_mut()
            .intern_ce(ClassExpr::Atomic(surrogate));
        let exists_ce = ontology.dl_mut().intern_ce(ClassExpr::Some {
            property: role.clone(),
            filler: fresh_ce,
        });
        ontology.dl_mut().push_axiom(DlAxiom::EquivalentClasses(vec![
            surrogate_ce,
            exists_ce,
        ]));
        role_to_surrogate.insert(role, surrogate);
    }
    Ok((ontology, role_to_surrogate))
}

fn surrogates_equivalent(taxonomy: &Taxonomy, left: EntityId, right: EntityId) -> bool {
    if left == right {
        return true;
    }
    taxonomy.is_subsumed(left, right) && taxonomy.is_subsumed(right, left)
}

fn equivalent_role_expressions_for_surrogate(
    taxonomy: &Taxonomy,
    role_to_surrogate: &HashMap<RoleExpr, EntityId>,
    surrogate: EntityId,
) -> HashSet<RoleExpr> {
    role_to_surrogate
        .iter()
        .filter(|(_, candidate)| surrogates_equivalent(taxonomy, surrogate, **candidate))
        .map(|(role, _)| role.clone())
        .collect()
}

/// Classify object-property expressions into equivalence classes (HermiT `m_objectRoleHierarchy` nodes).
pub fn classify_object_property_expressions(
    ontology: &Ontology,
) -> Result<Vec<HashSet<RoleExpr>>, Error> {
    let (augmented, role_to_surrogate) = build_role_classification_ontology(ontology)?;
    let taxonomy = classify(&augmented)?;

    let mut classes: Vec<HashSet<RoleExpr>> = Vec::new();
    let mut assigned = HashSet::new();
    for (role, surrogate) in &role_to_surrogate {
        if !assigned.insert(role.clone()) {
            continue;
        }
        let class = equivalent_role_expressions_for_surrogate(&taxonomy, &role_to_surrogate, *surrogate);
        assigned.extend(class.iter().cloned());
        classes.push(class);
    }
    Ok(classes)
}

/// OWL API `getEquivalentObjectProperties` via surrogate classification.
pub fn equivalent_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
) -> Result<HashSet<RoleExpr>, Error> {
    let (augmented, role_to_surrogate) = build_role_classification_ontology(ontology)?;
    let Some(&surrogate) = role_to_surrogate.get(property) else {
        return Ok(HashSet::from([property.clone()]));
    };
    let taxonomy = classify(&augmented)?;
    Ok(equivalent_role_expressions_for_surrogate(
        &taxonomy,
        &role_to_surrogate,
        surrogate,
    ))
}

/// OWL API `getSubObjectProperties` via saturated role hierarchy (+ surrogate equivalence).
pub fn sub_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
    direct: bool,
) -> Result<HashSet<RoleExpr>, Error> {
    let equiv = equivalent_object_property_expressions(ontology, property)?;
    let roles = collect_relevant_role_expressions(ontology);
    let mut out = HashSet::new();
    for candidate in &roles {
        if equiv.contains(&candidate) {
            continue;
        }
        if !role_expression_subsumes(ontology, property, &candidate)? {
            continue;
        }
        if direct && has_strict_role_intermediate(ontology, property, &candidate, &equiv, &roles)? {
            continue;
        }
        out.insert(candidate.clone());
    }
    Ok(out)
}

fn has_strict_role_intermediate(
    ontology: &Ontology,
    property: &RoleExpr,
    candidate: &RoleExpr,
    equiv: &HashSet<RoleExpr>,
    roles: &HashSet<RoleExpr>,
) -> Result<bool, Error> {
    for mid in roles {
        if mid == candidate || equiv.contains(mid) {
            continue;
        }
        if role_expression_subsumes(ontology, property, mid)?
            && role_expression_subsumes(ontology, mid, candidate)?
            && !role_expression_subsumes(ontology, candidate, mid)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// OWL API `getInverseObjectProperties` (HermiT: `getEquivalentObjectProperties(inverse(pe))`).
pub fn inverse_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
) -> Result<HashSet<RoleExpr>, Error> {
    equivalent_object_property_expressions(ontology, &inverse_role(property))
}

fn inverse_role(role: &RoleExpr) -> RoleExpr {
    match role {
        RoleExpr::Atomic(id) => RoleExpr::Inverse(*id),
        RoleExpr::Inverse(id) => RoleExpr::Atomic(*id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;

    #[test]
    fn hermit_inverse_cycle_equivalence_classes() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_owlreasonertest_testgetinverseobjectpropertyexpressions.ofn",
        );
        let ontology = load_ontology(&path).expect("load inverse OFN");
        const NS: &str = "file:/c/test.owl#";
        let r = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}r")).expect("r"));
        let s = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}s")).expect("s"));
        let t = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}t")).expect("t"));
        let inv_r = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}r")).expect("r"));
        let inv_s = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}s")).expect("s"));
        let inv_t = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}t")).expect("t"));

        let r_inverses = inverse_object_property_expressions(&ontology, &r).expect("inverses of r");
        assert_eq!(r_inverses, HashSet::from([inv_r.clone(), s.clone(), inv_t.clone()]));

        let inv_r_inverses =
            inverse_object_property_expressions(&ontology, &inv_r).expect("inverses of inv(r)");
        assert_eq!(inv_r_inverses, HashSet::from([inv_s, r, t]));
    }
}
