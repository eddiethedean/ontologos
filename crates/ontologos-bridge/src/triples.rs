use std::collections::{HashMap, HashSet};

use ontologos_core::{Axiom, AxiomId, DataLiteral, EntityId, EntityKind, Ontology};
use oxrdf::{BlankNode, Literal, NamedNode, Term, Triple};

use crate::{Error, Result};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_SUBCLASS: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const RDFS_SUBPROPERTY: &str = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";
const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
const OWL_EQUIV_CLASS: &str = "http://www.w3.org/2002/07/owl#equivalentClass";
const OWL_EQUIV_PROP: &str = "http://www.w3.org/2002/07/owl#equivalentProperty";
const OWL_INVERSE: &str = "http://www.w3.org/2002/07/owl#inverseOf";
const OWL_SAME_AS: &str = "http://www.w3.org/2002/07/owl#sameAs";
const OWL_TRANSITIVE: &str = "http://www.w3.org/2002/07/owl#TransitiveProperty";
const OWL_SYMMETRIC: &str = "http://www.w3.org/2002/07/owl#SymmetricProperty";
const OWL_REFLEXIVE: &str = "http://www.w3.org/2002/07/owl#ReflexiveProperty";
const OWL_FUNCTIONAL: &str = "http://www.w3.org/2002/07/owl#FunctionalProperty";
const OWL_INVERSE_FUNCTIONAL: &str = "http://www.w3.org/2002/07/owl#InverseFunctionalProperty";
const OWL_IRREFLEXIVE: &str = "http://www.w3.org/2002/07/owl#IrreflexiveProperty";
const OWL_ASYMMETRIC: &str = "http://www.w3.org/2002/07/owl#AsymmetricProperty";
const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
const OWL_OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
const OWL_DATATYPE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DatatypeProperty";
const OWL_NAMED_INDIVIDUAL: &str = "http://www.w3.org/2002/07/owl#NamedIndividual";
const OWL_RESTRICTION: &str = "http://www.w3.org/2002/07/owl#Restriction";
const OWL_ON_PROPERTY: &str = "http://www.w3.org/2002/07/owl#onProperty";
const OWL_SOME_VALUES_FROM: &str = "http://www.w3.org/2002/07/owl#someValuesFrom";
const OWL_DISJOINT_WITH: &str = "http://www.w3.org/2002/07/owl#disjointWith";
const OWL_DIFFERENT_FROM: &str = "http://www.w3.org/2002/07/owl#differentFrom";

fn nn(iri: &str) -> Result<NamedNode> {
    NamedNode::new(iri).map_err(|e| Error::Bridge(format!("invalid IRI {iri}: {e}")))
}

fn entity_iri(ontology: &Ontology, id: EntityId) -> Result<String> {
    let record = ontology.entity(id)?;
    Ok(ontology.resolve_iri(record.iri)?.to_owned())
}

fn entity_iri_cached(
    cache: &mut HashMap<EntityId, String>,
    ontology: &Ontology,
    id: EntityId,
) -> Result<String> {
    if let Some(iri) = cache.get(&id) {
        return Ok(iri.clone());
    }
    let iri = entity_iri(ontology, id)?;
    cache.insert(id, iri.clone());
    Ok(iri)
}

fn triple(s: &str, p: &str, o: &str) -> Result<Triple> {
    Ok(Triple {
        subject: nn(s)?.into(),
        predicate: nn(p)?,
        object: Term::NamedNode(nn(o)?),
    })
}

fn data_property_triple(subject: &str, property: &str, value: &DataLiteral) -> Result<Triple> {
    let datatype = nn(&value.datatype)?;
    Ok(Triple {
        subject: nn(subject)?.into(),
        predicate: nn(property)?,
        object: Term::Literal(Literal::new_typed_literal(value.lexical.clone(), datatype)),
    })
}

fn blank_triple(subject: BlankNode, predicate: &str, object: &str) -> Result<Triple> {
    Ok(Triple {
        subject: subject.into(),
        predicate: nn(predicate)?,
        object: Term::NamedNode(nn(object)?),
    })
}

fn entity_id_pairs(items: &[EntityId]) -> Vec<(EntityId, EntityId)> {
    let mut pairs = Vec::new();
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            pairs.push((items[i], items[j]));
        }
    }
    pairs
}

fn existential_restriction_triples(
    ontology: &Ontology,
    subclass: EntityId,
    property: EntityId,
    filler: EntityId,
) -> Result<Vec<Triple>> {
    let sub = entity_iri(ontology, subclass)?;
    let prop = entity_iri(ontology, property)?;
    let filler_iri = entity_iri(ontology, filler)?;
    let bnode = BlankNode::new(format!("rest_{}_{}_{}", subclass.0, property.0, filler.0))
        .map_err(|e| Error::Bridge(format!("blank node: {e}")))?;
    Ok(vec![
        Triple {
            subject: nn(&sub)?.into(),
            predicate: nn(RDFS_SUBCLASS)?,
            object: Term::BlankNode(bnode.clone()),
        },
        blank_triple(bnode.clone(), RDF_TYPE, OWL_RESTRICTION)?,
        blank_triple(bnode.clone(), OWL_ON_PROPERTY, &prop)?,
        blank_triple(bnode, OWL_SOME_VALUES_FROM, &filler_iri)?,
    ])
}

/// Serialize mapped core axioms to oxrdf triples for reasonable.
pub fn core_to_triples(ontology: &Ontology) -> Result<Vec<Triple>> {
    let mut out = Vec::new();

    for (id, record) in ontology.entities().iter() {
        let iri = ontology.resolve_iri(record.iri)?;
        match record.kind {
            EntityKind::Class => {
                out.push(triple(iri, RDF_TYPE, OWL_CLASS)?);
            }
            EntityKind::Individual => {
                out.push(triple(iri, RDF_TYPE, OWL_NAMED_INDIVIDUAL)?);
            }
            EntityKind::ObjectProperty => {
                out.push(triple(iri, RDF_TYPE, OWL_OBJECT_PROPERTY)?);
            }
            EntityKind::DataProperty => {
                out.push(triple(iri, RDF_TYPE, OWL_DATATYPE_PROPERTY)?);
            }
            EntityKind::AnnotationProperty | _ => {}
        }
        let _ = id;
    }

    let mut iri_cache = HashMap::new();
    for (_, axiom) in ontology.axioms().iter_asserted() {
        out.extend(axiom_to_triples(ontology, axiom, &mut iri_cache)?);
    }

    Ok(out)
}

/// Serialize all active axioms (asserted and inferred) to oxrdf triples.
pub fn core_to_triples_all(ontology: &Ontology) -> Result<Vec<Triple>> {
    let mut out = Vec::new();

    for (id, record) in ontology.entities().iter() {
        let iri = ontology.resolve_iri(record.iri)?;
        match record.kind {
            EntityKind::Class => {
                out.push(triple(iri, RDF_TYPE, OWL_CLASS)?);
            }
            EntityKind::Individual => {
                out.push(triple(iri, RDF_TYPE, OWL_NAMED_INDIVIDUAL)?);
            }
            EntityKind::ObjectProperty => {
                out.push(triple(iri, RDF_TYPE, OWL_OBJECT_PROPERTY)?);
            }
            EntityKind::DataProperty => {
                out.push(triple(iri, RDF_TYPE, OWL_DATATYPE_PROPERTY)?);
            }
            EntityKind::AnnotationProperty | _ => {}
        }
        let _ = id;
    }

    let mut iri_cache = HashMap::new();
    for (_, axiom) in ontology.axioms().iter() {
        out.extend(axiom_to_triples(ontology, axiom, &mut iri_cache)?);
    }

    Ok(out)
}

/// Serialize triples for a subset of axioms (plus entity type triples for referenced entities).
pub fn core_to_triples_for_axioms(ontology: &Ontology, ids: &[AxiomId]) -> Result<Vec<Triple>> {
    use ontologos_core::axiom_signature;
    use std::collections::HashSet;

    let mut entities: HashSet<EntityId> = HashSet::new();
    let mut out = Vec::new();

    let mut iri_cache = HashMap::new();
    for id in ids {
        let axiom = ontology.axioms().get(*id)?;
        entities.extend(axiom_signature(axiom));
        out.extend(axiom_to_triples(ontology, axiom, &mut iri_cache)?);
    }

    for entity in entities {
        let record = ontology.entity(entity)?;
        let iri = ontology.resolve_iri(record.iri)?;
        match record.kind {
            EntityKind::Class => out.push(triple(iri, RDF_TYPE, OWL_CLASS)?),
            EntityKind::Individual => out.push(triple(iri, RDF_TYPE, OWL_NAMED_INDIVIDUAL)?),
            EntityKind::ObjectProperty => out.push(triple(iri, RDF_TYPE, OWL_OBJECT_PROPERTY)?),
            EntityKind::DataProperty | EntityKind::AnnotationProperty | _ => {}
        }
    }

    Ok(out)
}

fn axiom_to_triples(
    ontology: &Ontology,
    axiom: &Axiom,
    iri_cache: &mut HashMap<EntityId, String>,
) -> Result<Vec<Triple>> {
    let mut out = Vec::new();
    match axiom {
        Axiom::SubClassOf {
            subclass,
            superclass,
        } => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *subclass)?,
                RDFS_SUBCLASS,
                &entity_iri_cached(iri_cache, ontology, *superclass)?,
            )?);
        }
        Axiom::EquivalentClasses(classes) => {
            for (a, b) in entity_id_pairs(classes) {
                out.push(triple(
                    &entity_iri_cached(iri_cache, ontology, a)?,
                    OWL_EQUIV_CLASS,
                    &entity_iri_cached(iri_cache, ontology, b)?,
                )?);
            }
        }
        Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *sub_property)?,
                RDFS_SUBPROPERTY,
                &entity_iri_cached(iri_cache, ontology, *super_property)?,
            )?);
        }
        Axiom::ObjectPropertyDomain { property, domain } => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDFS_DOMAIN,
                &entity_iri_cached(iri_cache, ontology, *domain)?,
            )?);
        }
        Axiom::ObjectPropertyRange { property, range } => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDFS_RANGE,
                &entity_iri_cached(iri_cache, ontology, *range)?,
            )?);
        }
        Axiom::InverseObjectProperties { left, right } => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *left)?,
                OWL_INVERSE,
                &entity_iri_cached(iri_cache, ontology, *right)?,
            )?);
        }
        Axiom::TransitiveObjectProperty(property) => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDF_TYPE,
                OWL_TRANSITIVE,
            )?);
        }
        Axiom::SymmetricObjectProperty(property) => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDF_TYPE,
                OWL_SYMMETRIC,
            )?);
        }
        Axiom::ReflexiveObjectProperty(property) => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDF_TYPE,
                OWL_REFLEXIVE,
            )?);
        }
        Axiom::FunctionalObjectProperty(property) => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDF_TYPE,
                OWL_FUNCTIONAL,
            )?);
        }
        Axiom::InverseFunctionalObjectProperty(property) => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDF_TYPE,
                OWL_INVERSE_FUNCTIONAL,
            )?);
        }
        Axiom::IrreflexiveObjectProperty(property) => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDF_TYPE,
                OWL_IRREFLEXIVE,
            )?);
        }
        Axiom::AsymmetricObjectProperty(property) => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *property)?,
                RDF_TYPE,
                OWL_ASYMMETRIC,
            )?);
        }
        Axiom::EquivalentObjectProperties(properties) => {
            for (a, b) in entity_id_pairs(properties) {
                out.push(triple(
                    &entity_iri_cached(iri_cache, ontology, a)?,
                    OWL_EQUIV_PROP,
                    &entity_iri_cached(iri_cache, ontology, b)?,
                )?);
            }
        }
        Axiom::ClassAssertion { individual, class } => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *individual)?,
                RDF_TYPE,
                &entity_iri_cached(iri_cache, ontology, *class)?,
            )?);
        }
        Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => {
            out.push(triple(
                &entity_iri_cached(iri_cache, ontology, *subject)?,
                &entity_iri_cached(iri_cache, ontology, *property)?,
                &entity_iri_cached(iri_cache, ontology, *object)?,
            )?);
        }
        Axiom::DataPropertyAssertion {
            individual,
            property,
            value,
        } => {
            out.push(data_property_triple(
                &entity_iri_cached(iri_cache, ontology, *individual)?,
                &entity_iri_cached(iri_cache, ontology, *property)?,
                value,
            )?);
        }
        Axiom::NegativeObjectPropertyAssertion { .. } => {
            return Err(Error::Bridge(
                "NegativeObjectPropertyAssertion is not supported in RL triple export".into(),
            ));
        }
        Axiom::NegativeDataPropertyAssertion { .. } => {
            return Err(Error::Bridge(
                "NegativeDataPropertyAssertion is not supported in RL triple export".into(),
            ));
        }
        Axiom::SameIndividual(individuals) => {
            for (a, b) in entity_id_pairs(individuals) {
                out.push(triple(
                    &entity_iri_cached(iri_cache, ontology, a)?,
                    OWL_SAME_AS,
                    &entity_iri_cached(iri_cache, ontology, b)?,
                )?);
            }
        }
        Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } => {
            out.extend(existential_restriction_triples(
                ontology, *subclass, *property, *filler,
            )?);
        }
        Axiom::DisjointClasses(classes) => {
            for (a, b) in entity_id_pairs(classes) {
                out.push(triple(
                    &entity_iri_cached(iri_cache, ontology, a)?,
                    OWL_DISJOINT_WITH,
                    &entity_iri_cached(iri_cache, ontology, b)?,
                )?);
            }
        }
        Axiom::DifferentIndividuals(individuals) => {
            for (a, b) in entity_id_pairs(individuals) {
                out.push(triple(
                    &entity_iri_cached(iri_cache, ontology, a)?,
                    OWL_DIFFERENT_FROM,
                    &entity_iri_cached(iri_cache, ontology, b)?,
                )?);
            }
        }
    }
    Ok(out)
}

/// Limits applied when merging inferred triples back into core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeLimits {
    /// Maximum axioms allowed in the ontology after merge (including asserted).
    pub max_axioms: usize,
    /// Maximum entities allowed in the ontology after merge.
    pub max_entities: usize,
}

impl Default for MergeLimits {
    fn default() -> Self {
        Self {
            max_axioms: 10_000_000,
            max_entities: 1_000_000,
        }
    }
}

/// Summary of merging reasonable output back into core.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MergeReport {
    pub inferred_axioms: usize,
    pub clashes: Vec<String>,
}

/// Merge materialized triples into `ontology`, adding only new axioms.
pub fn merge_triples_into_ontology(
    ontology: &mut Ontology,
    triples: &[Triple],
    diagnostics: &[reasonable::reasoner::ReasoningError],
) -> Result<MergeReport> {
    merge_triples_into_ontology_with_limits(ontology, triples, diagnostics, MergeLimits::default())
}

/// Merge materialized triples with an axiom cap (untrusted reasoning workloads).
pub fn merge_triples_into_ontology_with_limits(
    ontology: &mut Ontology,
    triples: &[Triple],
    diagnostics: &[reasonable::reasoner::ReasoningError],
    limits: MergeLimits,
) -> Result<MergeReport> {
    let before = ontology.axiom_count();
    let prev_limits = ontology.enforce_limits();
    ontology.set_enforce_limits(ontologos_core::Limits {
        max_entities: limits.max_entities,
        max_axioms: limits.max_axioms,
        ..ontologos_core::Limits::default()
    });

    let report = merge_triples_into_ontology_inner(ontology, triples, diagnostics, limits, before);
    ontology.restore_enforce_limits(prev_limits);
    report
}

fn merge_triples_into_ontology_inner(
    ontology: &mut Ontology,
    triples: &[Triple],
    diagnostics: &[reasonable::reasoner::ReasoningError],
    limits: MergeLimits,
    before: usize,
) -> Result<MergeReport> {
    let mut seen: HashSet<(String, String, String)> = HashSet::new();
    let mut seen_entity: HashSet<u64> = HashSet::new();

    for (_, axiom) in ontology.axioms().iter() {
        if let Some(key) = axiom_entity_key(axiom) {
            seen_entity.insert(key);
        }
        if let Some(key) = axiom_triple_key(ontology, axiom)? {
            seen.insert(key);
        }
    }

    if ontology.entity_count() >= limits.max_entities {
        return Err(Error::Bridge(format!(
            "entity limit {} would be exceeded during merge",
            limits.max_entities
        )));
    }

    let mut to_add: Vec<Axiom> = Vec::new();
    let mut merge_clashes: Vec<String> = Vec::new();
    for axiom in collect_existential_axioms(ontology, triples, &limits, &mut merge_clashes)? {
        let entity_key = axiom_entity_key(&axiom);
        let triple_key = axiom_triple_key(ontology, &axiom)?;
        let is_new = entity_key.map(|k| seen_entity.insert(k)).unwrap_or(true)
            && triple_key.map(|k| seen.insert(k)).unwrap_or(true);
        if is_new {
            to_add.push(axiom);
        }
    }

    for t in triples {
        match triple_to_axiom(ontology, t, &limits) {
            Ok(Some(axiom)) => {
                let entity_key = axiom_entity_key(&axiom);
                let triple_key = axiom_triple_key(ontology, &axiom)?;
                let is_new = entity_key.map(|k| seen_entity.insert(k)).unwrap_or(true)
                    && triple_key.map(|k| seen.insert(k)).unwrap_or(true);
                if is_new {
                    to_add.push(axiom);
                }
            }
            Ok(None) => {}
            Err(Error::Core(ontologos_core::Error::EntityKindMismatch {
                iri,
                expected,
                found,
            })) => {
                merge_clashes.push(format!(
                    "entity kind mismatch for {iri}: expected {expected:?}, found {found:?}"
                ));
            }
            Err(e) => return Err(e),
        }
    }

    if ontology.axiom_count().saturating_add(to_add.len()) > limits.max_axioms {
        return Err(Error::Bridge(format!(
            "axiom limit {} would be exceeded during merge ({} new axioms)",
            limits.max_axioms,
            to_add.len()
        )));
    }

    for axiom in to_add {
        ontology.add_inferred_axiom(axiom)?;
    }

    let mut clashes: Vec<String> = diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.rule(), d.message()))
        .collect();
    clashes.extend(merge_clashes);

    Ok(MergeReport {
        inferred_axioms: ontology.axiom_count().saturating_sub(before),
        clashes,
    })
}

fn term_iri(term: &Term) -> Option<String> {
    match term {
        Term::NamedNode(n) => Some(n.as_str().to_string()),
        _ => None,
    }
}

fn blank_node_id(node: &BlankNode) -> String {
    node.as_str().to_string()
}

#[derive(Debug, Default)]
struct RestrictionParts {
    is_restriction: bool,
    property: Option<String>,
    filler: Option<String>,
}

fn collect_existential_axioms(
    ontology: &mut Ontology,
    triples: &[Triple],
    limits: &MergeLimits,
    merge_clashes: &mut Vec<String>,
) -> Result<Vec<Axiom>> {
    let mut restrictions: HashMap<String, RestrictionParts> = HashMap::new();
    let mut subclass_edges: Vec<(String, String)> = Vec::new();

    for triple in triples {
        match &triple.subject {
            oxrdf::NamedOrBlankNode::BlankNode(bnode) => {
                let id = blank_node_id(bnode);
                let entry = restrictions.entry(id).or_default();
                match triple.predicate.as_str() {
                    RDF_TYPE if term_iri(&triple.object).as_deref() == Some(OWL_RESTRICTION) => {
                        entry.is_restriction = true;
                    }
                    OWL_ON_PROPERTY => {
                        entry.property = term_iri(&triple.object);
                    }
                    OWL_SOME_VALUES_FROM => {
                        entry.filler = term_iri(&triple.object);
                    }
                    _ => {}
                }
            }
            oxrdf::NamedOrBlankNode::NamedNode(subject) => {
                if triple.predicate.as_str() == RDFS_SUBCLASS
                    && let Term::BlankNode(bnode) = &triple.object
                {
                    subclass_edges.push((subject.as_str().to_string(), blank_node_id(bnode)));
                }
            }
        }
    }

    let mut axioms = Vec::new();
    for (subclass_iri, bnode_id) in subclass_edges {
        let Some(parts) = restrictions.get(&bnode_id) else {
            continue;
        };
        if !parts.is_restriction {
            continue;
        }
        let (Some(property_iri), Some(filler_iri)) = (&parts.property, &parts.filler) else {
            continue;
        };
        let subclass = match lookup_or_insert(ontology, &subclass_iri, EntityKind::Class, limits) {
            Ok(id) => id,
            Err(Error::Core(ontologos_core::Error::EntityKindMismatch {
                iri,
                expected,
                found,
            })) => {
                merge_clashes.push(format!(
                    "entity kind mismatch for {iri}: expected {expected:?}, found {found:?}"
                ));
                continue;
            }
            Err(e) => return Err(e),
        };
        let property =
            match lookup_or_insert(ontology, property_iri, EntityKind::ObjectProperty, limits) {
                Ok(id) => id,
                Err(Error::Core(ontologos_core::Error::EntityKindMismatch {
                    iri,
                    expected,
                    found,
                })) => {
                    merge_clashes.push(format!(
                        "entity kind mismatch for {iri}: expected {expected:?}, found {found:?}"
                    ));
                    continue;
                }
                Err(e) => return Err(e),
            };
        let filler = match lookup_or_insert(ontology, filler_iri, EntityKind::Class, limits) {
            Ok(id) => id,
            Err(Error::Core(ontologos_core::Error::EntityKindMismatch {
                iri,
                expected,
                found,
            })) => {
                merge_clashes.push(format!(
                    "entity kind mismatch for {iri}: expected {expected:?}, found {found:?}"
                ));
                continue;
            }
            Err(e) => return Err(e),
        };
        axioms.push(Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        });
    }

    Ok(axioms)
}

fn triple_to_axiom(
    ontology: &mut Ontology,
    triple: &Triple,
    limits: &MergeLimits,
) -> Result<Option<Axiom>> {
    let sub = match &triple.subject {
        oxrdf::NamedOrBlankNode::NamedNode(n) => n.as_str(),
        _ => return Ok(None),
    };
    let pred = triple.predicate.as_str();
    if let Term::Literal(lit) = &triple.object {
        let individual = lookup_or_insert(ontology, sub, EntityKind::Individual, limits)?;
        let property = lookup_or_insert(ontology, pred, EntityKind::DataProperty, limits)?;
        return Ok(Some(Axiom::DataPropertyAssertion {
            individual,
            property,
            value: DataLiteral {
                lexical: lit.value().to_string(),
                datatype: lit.datatype().as_str().to_string(),
            },
        }));
    }
    let obj = match term_iri(&triple.object) {
        Some(o) => o,
        None => return Ok(None),
    };

    if pred == RDFS_SUBCLASS {
        let subclass = lookup_or_insert(ontology, sub, EntityKind::Class, limits)?;
        let superclass = lookup_or_insert(ontology, &obj, EntityKind::Class, limits)?;
        return Ok(Some(Axiom::SubClassOf {
            subclass,
            superclass,
        }));
    }
    if pred == RDFS_SUBPROPERTY {
        let sub = lookup_or_insert(ontology, sub, EntityKind::ObjectProperty, limits)?;
        let sup = lookup_or_insert(ontology, &obj, EntityKind::ObjectProperty, limits)?;
        return Ok(Some(Axiom::SubObjectPropertyOf {
            sub_property: sub,
            super_property: sup,
        }));
    }
    if pred == RDFS_DOMAIN {
        let property = lookup_or_insert(ontology, sub, EntityKind::ObjectProperty, limits)?;
        let domain = lookup_or_insert(ontology, &obj, EntityKind::Class, limits)?;
        return Ok(Some(Axiom::ObjectPropertyDomain { property, domain }));
    }
    if pred == RDFS_RANGE {
        let property = lookup_or_insert(ontology, sub, EntityKind::ObjectProperty, limits)?;
        let range = lookup_or_insert(ontology, &obj, EntityKind::Class, limits)?;
        return Ok(Some(Axiom::ObjectPropertyRange { property, range }));
    }
    if pred == OWL_EQUIV_CLASS {
        let a = lookup_or_insert(ontology, sub, EntityKind::Class, limits)?;
        let b = lookup_or_insert(ontology, &obj, EntityKind::Class, limits)?;
        return Ok(Some(Axiom::EquivalentClasses(vec![a, b])));
    }
    if pred == OWL_EQUIV_PROP {
        let a = lookup_or_insert(ontology, sub, EntityKind::ObjectProperty, limits)?;
        let b = lookup_or_insert(ontology, &obj, EntityKind::ObjectProperty, limits)?;
        return Ok(Some(Axiom::EquivalentObjectProperties(vec![a, b])));
    }
    if pred == OWL_INVERSE {
        let left = lookup_or_insert(ontology, sub, EntityKind::ObjectProperty, limits)?;
        let right = lookup_or_insert(ontology, &obj, EntityKind::ObjectProperty, limits)?;
        return Ok(Some(Axiom::InverseObjectProperties { left, right }));
    }
    if pred == RDF_TYPE && obj == OWL_TRANSITIVE {
        return Ok(Some(Axiom::TransitiveObjectProperty(lookup_or_insert(
            ontology,
            sub,
            EntityKind::ObjectProperty,
            limits,
        )?)));
    }
    if pred == RDF_TYPE && obj == OWL_SYMMETRIC {
        return Ok(Some(Axiom::SymmetricObjectProperty(lookup_or_insert(
            ontology,
            sub,
            EntityKind::ObjectProperty,
            limits,
        )?)));
    }
    if pred == RDF_TYPE && obj == OWL_REFLEXIVE {
        return Ok(Some(Axiom::ReflexiveObjectProperty(lookup_or_insert(
            ontology,
            sub,
            EntityKind::ObjectProperty,
            limits,
        )?)));
    }
    if pred == RDF_TYPE && obj == OWL_FUNCTIONAL {
        return Ok(Some(Axiom::FunctionalObjectProperty(lookup_or_insert(
            ontology,
            sub,
            EntityKind::ObjectProperty,
            limits,
        )?)));
    }
    if pred == RDF_TYPE && obj == OWL_INVERSE_FUNCTIONAL {
        return Ok(Some(Axiom::InverseFunctionalObjectProperty(
            lookup_or_insert(ontology, sub, EntityKind::ObjectProperty, limits)?,
        )));
    }
    if pred == RDF_TYPE && obj == OWL_IRREFLEXIVE {
        return Ok(Some(Axiom::IrreflexiveObjectProperty(lookup_or_insert(
            ontology,
            sub,
            EntityKind::ObjectProperty,
            limits,
        )?)));
    }
    if pred == RDF_TYPE && obj == OWL_ASYMMETRIC {
        return Ok(Some(Axiom::AsymmetricObjectProperty(lookup_or_insert(
            ontology,
            sub,
            EntityKind::ObjectProperty,
            limits,
        )?)));
    }
    if pred == RDF_TYPE
        && obj != OWL_CLASS
        && obj != OWL_NAMED_INDIVIDUAL
        && obj != OWL_OBJECT_PROPERTY
    {
        let individual = lookup_or_insert(ontology, sub, EntityKind::Individual, limits)?;
        let class = lookup_or_insert(ontology, &obj, EntityKind::Class, limits)?;
        return Ok(Some(Axiom::ClassAssertion { individual, class }));
    }
    if pred == OWL_SAME_AS {
        if sub == obj {
            return Ok(None);
        }
        let left = lookup_or_insert(ontology, sub, EntityKind::Individual, limits)?;
        let right = lookup_or_insert(ontology, &obj, EntityKind::Individual, limits)?;
        return Ok(Some(Axiom::SameIndividual(vec![left, right])));
    }
    if pred == OWL_DISJOINT_WITH {
        if sub == obj {
            return Ok(None);
        }
        let left = lookup_or_insert(ontology, sub, EntityKind::Class, limits)?;
        let right = lookup_or_insert(ontology, &obj, EntityKind::Class, limits)?;
        return Ok(Some(Axiom::DisjointClasses(vec![left, right])));
    }
    if pred == OWL_DIFFERENT_FROM {
        if sub == obj {
            return Ok(None);
        }
        let left = lookup_or_insert(ontology, sub, EntityKind::Individual, limits)?;
        let right = lookup_or_insert(ontology, &obj, EntityKind::Individual, limits)?;
        return Ok(Some(Axiom::DifferentIndividuals(vec![left, right])));
    }
    if pred != RDF_TYPE && pred != RDFS_SUBCLASS && pred != RDFS_SUBPROPERTY {
        let subject = lookup_or_insert(ontology, sub, EntityKind::Individual, limits)?;
        let property = lookup_or_insert(ontology, pred, EntityKind::ObjectProperty, limits)?;
        let object = lookup_or_insert(ontology, &obj, EntityKind::Individual, limits)?;
        return Ok(Some(Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        }));
    }
    Ok(None)
}

fn lookup_or_insert(
    ontology: &mut Ontology,
    iri: &str,
    kind: EntityKind,
    limits: &MergeLimits,
) -> Result<EntityId> {
    if let Some(id) = ontology.lookup_entity(iri) {
        let record = ontology.entity(id)?;
        if record.kind.satisfies(kind) {
            return Ok(id);
        }
        if EntityKind::merge_punning(record.kind, kind).is_none() {
            return Err(Error::Core(ontologos_core::Error::EntityKindMismatch {
                iri: iri.to_owned(),
                expected: kind,
                found: record.kind,
            }));
        }
    } else if ontology.entity_count() >= limits.max_entities {
        return Err(Error::Bridge(format!(
            "entity limit {} would be exceeded during merge",
            limits.max_entities
        )));
    }
    Ok(ontology.entity_id(iri, kind)?)
}

fn axiom_entity_key(axiom: &Axiom) -> Option<u64> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    match axiom {
        Axiom::SubClassOf {
            subclass,
            superclass,
        } => {
            0u8.hash(&mut hasher);
            subclass.hash(&mut hasher);
            superclass.hash(&mut hasher);
        }
        Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => {
            1u8.hash(&mut hasher);
            sub_property.hash(&mut hasher);
            super_property.hash(&mut hasher);
        }
        Axiom::ObjectPropertyDomain { property, domain } => {
            2u8.hash(&mut hasher);
            property.hash(&mut hasher);
            domain.hash(&mut hasher);
        }
        Axiom::ObjectPropertyRange { property, range } => {
            3u8.hash(&mut hasher);
            property.hash(&mut hasher);
            range.hash(&mut hasher);
        }
        Axiom::ClassAssertion { individual, class } => {
            4u8.hash(&mut hasher);
            individual.hash(&mut hasher);
            class.hash(&mut hasher);
        }
        Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => {
            5u8.hash(&mut hasher);
            subject.hash(&mut hasher);
            property.hash(&mut hasher);
            object.hash(&mut hasher);
        }
        _ => return None,
    }
    Some(hasher.finish())
}

fn axiom_triple_key(
    ontology: &Ontology,
    axiom: &Axiom,
) -> Result<Option<(String, String, String)>> {
    match axiom {
        Axiom::SubClassOf {
            subclass,
            superclass,
        } => Ok(Some((
            entity_iri(ontology, *subclass)?,
            RDFS_SUBCLASS.to_string(),
            entity_iri(ontology, *superclass)?,
        ))),
        Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => Ok(Some((
            entity_iri(ontology, *sub_property)?,
            RDFS_SUBPROPERTY.to_string(),
            entity_iri(ontology, *super_property)?,
        ))),
        Axiom::ObjectPropertyDomain { property, domain } => Ok(Some((
            entity_iri(ontology, *property)?,
            RDFS_DOMAIN.to_string(),
            entity_iri(ontology, *domain)?,
        ))),
        Axiom::ObjectPropertyRange { property, range } => Ok(Some((
            entity_iri(ontology, *property)?,
            RDFS_RANGE.to_string(),
            entity_iri(ontology, *range)?,
        ))),
        Axiom::ClassAssertion { individual, class } => Ok(Some((
            entity_iri(ontology, *individual)?,
            RDF_TYPE.to_string(),
            entity_iri(ontology, *class)?,
        ))),
        Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => Ok(Some((
            entity_iri(ontology, *subject)?,
            entity_iri(ontology, *property)?,
            entity_iri(ontology, *object)?,
        ))),
        Axiom::DataPropertyAssertion { .. }
        | Axiom::NegativeObjectPropertyAssertion { .. }
        | Axiom::NegativeDataPropertyAssertion { .. } => Ok(None),
        Axiom::EquivalentClasses(classes) => classes
            .windows(2)
            .next()
            .map(|pair| {
                Ok((
                    entity_iri(ontology, pair[0])?,
                    OWL_EQUIV_CLASS.to_string(),
                    entity_iri(ontology, pair[1])?,
                ))
            })
            .transpose(),
        Axiom::EquivalentObjectProperties(properties) => properties
            .windows(2)
            .next()
            .map(|pair| {
                Ok((
                    entity_iri(ontology, pair[0])?,
                    OWL_EQUIV_PROP.to_string(),
                    entity_iri(ontology, pair[1])?,
                ))
            })
            .transpose(),
        Axiom::InverseObjectProperties { left, right } => Ok(Some((
            entity_iri(ontology, *left)?,
            OWL_INVERSE.to_string(),
            entity_iri(ontology, *right)?,
        ))),
        Axiom::SameIndividual(individuals) => individuals
            .windows(2)
            .next()
            .map(|pair| {
                Ok((
                    entity_iri(ontology, pair[0])?,
                    OWL_SAME_AS.to_string(),
                    entity_iri(ontology, pair[1])?,
                ))
            })
            .transpose(),
        Axiom::TransitiveObjectProperty(property) => Ok(Some((
            entity_iri(ontology, *property)?,
            RDF_TYPE.to_string(),
            OWL_TRANSITIVE.to_string(),
        ))),
        Axiom::SymmetricObjectProperty(property) => Ok(Some((
            entity_iri(ontology, *property)?,
            RDF_TYPE.to_string(),
            OWL_SYMMETRIC.to_string(),
        ))),
        Axiom::ReflexiveObjectProperty(property) => Ok(Some((
            entity_iri(ontology, *property)?,
            RDF_TYPE.to_string(),
            OWL_REFLEXIVE.to_string(),
        ))),
        Axiom::FunctionalObjectProperty(property) => Ok(Some((
            entity_iri(ontology, *property)?,
            RDF_TYPE.to_string(),
            OWL_FUNCTIONAL.to_string(),
        ))),
        Axiom::InverseFunctionalObjectProperty(property) => Ok(Some((
            entity_iri(ontology, *property)?,
            RDF_TYPE.to_string(),
            OWL_INVERSE_FUNCTIONAL.to_string(),
        ))),
        Axiom::IrreflexiveObjectProperty(property) => Ok(Some((
            entity_iri(ontology, *property)?,
            RDF_TYPE.to_string(),
            OWL_IRREFLEXIVE.to_string(),
        ))),
        Axiom::AsymmetricObjectProperty(property) => Ok(Some((
            entity_iri(ontology, *property)?,
            RDF_TYPE.to_string(),
            OWL_ASYMMETRIC.to_string(),
        ))),
        Axiom::DisjointClasses(classes) => classes
            .windows(2)
            .next()
            .map(|pair| {
                Ok((
                    entity_iri(ontology, pair[0])?,
                    OWL_DISJOINT_WITH.to_string(),
                    entity_iri(ontology, pair[1])?,
                ))
            })
            .transpose(),
        Axiom::DifferentIndividuals(individuals) => individuals
            .windows(2)
            .next()
            .map(|pair| {
                Ok((
                    entity_iri(ontology, pair[0])?,
                    OWL_DIFFERENT_FROM.to_string(),
                    entity_iri(ontology, pair[1])?,
                ))
            })
            .transpose(),
        Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } => Ok(Some((
            entity_iri(ontology, *subclass)?,
            OWL_ON_PROPERTY.to_string(),
            format!(
                "{}|{}",
                entity_iri(ontology, *property)?,
                entity_iri(ontology, *filler)?
            ),
        ))),
    }
}

#[cfg(test)]
mod adapter_tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};
    use reasonable::reasoner::ReasonerBuilder;

    use super::*;

    #[test]
    fn merge_persists_same_as_disjoint_and_different() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/a", EntityKind::Individual)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/b", EntityKind::Individual)
            .unwrap();
        let c1 = ontology
            .entity_id("http://ex.org/C1", EntityKind::Class)
            .unwrap();
        let c2 = ontology
            .entity_id("http://ex.org/C2", EntityKind::Class)
            .unwrap();

        let triples = vec![
            triple("http://ex.org/a", OWL_SAME_AS, "http://ex.org/b").unwrap(),
            triple("http://ex.org/C1", OWL_DISJOINT_WITH, "http://ex.org/C2").unwrap(),
            triple("http://ex.org/a", OWL_DIFFERENT_FROM, "http://ex.org/b").unwrap(),
        ];
        merge_triples_into_ontology(&mut ontology, &triples, &[]).unwrap();

        assert!(
            ontology.axioms().iter().any(
                |(_, axiom)| matches!(axiom, Axiom::SameIndividual(ids) if ids == &vec![a, b])
            )
        );
        assert!(ontology.axioms().iter().any(
            |(_, axiom)| matches!(axiom, Axiom::DisjointClasses(ids) if ids == &vec![c1, c2])
        ));
        assert!(ontology.axioms().iter().any(
            |(_, axiom)| matches!(axiom, Axiom::DifferentIndividuals(ids) if ids == &vec![a, b])
        ));
        assert!(
            !ontology
                .axioms()
                .iter()
                .any(|(_, axiom)| matches!(axiom, Axiom::ObjectPropertyAssertion { .. }))
        );
    }

    #[test]
    fn existential_blank_nodes_include_filler_in_id() {
        let mut ontology = Ontology::new();
        let sub = ontology
            .entity_id("http://ex.org/C", EntityKind::Class)
            .unwrap();
        let prop = ontology
            .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
            .unwrap();
        let d1 = ontology
            .entity_id("http://ex.org/D1", EntityKind::Class)
            .unwrap();
        let d2 = ontology
            .entity_id("http://ex.org/D2", EntityKind::Class)
            .unwrap();

        let t1 = existential_restriction_triples(&ontology, sub, prop, d1).unwrap();
        let t2 = existential_restriction_triples(&ontology, sub, prop, d2).unwrap();

        let bnode1 = match &t1[0].object {
            Term::BlankNode(b) => b.as_str().to_string(),
            _ => panic!("expected blank node"),
        };
        let bnode2 = match &t2[0].object {
            Term::BlankNode(b) => b.as_str().to_string(),
            _ => panic!("expected blank node"),
        };
        assert_ne!(bnode1, bnode2);
    }

    #[test]
    fn merge_reconstructs_existential_from_blank_node_restriction() {
        let mut ontology = Ontology::new();
        let sub = ontology
            .entity_id("http://ex.org/C", EntityKind::Class)
            .unwrap();
        let prop = ontology
            .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
            .unwrap();
        let filler = ontology
            .entity_id("http://ex.org/D", EntityKind::Class)
            .unwrap();

        let triples = existential_restriction_triples(&ontology, sub, prop, filler).unwrap();
        merge_triples_into_ontology(&mut ontology, &triples, &[]).unwrap();

        assert!(ontology.axioms().iter().any(|(_, axiom)| matches!(
            axiom,
            Axiom::SubClassOfExistential {
                subclass,
                property,
                filler: f
            } if *subclass == sub && *property == prop && *f == filler
        )));
    }

    #[test]
    fn merge_persists_inferred_property_assertions() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/a", EntityKind::Individual)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/b", EntityKind::Individual)
            .unwrap();
        let p = ontology
            .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
            .unwrap();
        ontology
            .add_axiom(Axiom::ObjectPropertyAssertion {
                subject: a,
                property: p,
                object: b,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::SymmetricObjectProperty(p))
            .unwrap();

        let input = core_to_triples(&ontology).unwrap();
        let mut reasoner = ReasonerBuilder::new().with_triples(input).build().unwrap();
        reasoner.reason_full();
        merge_triples_into_ontology(
            &mut ontology,
            reasoner.view_output(),
            reasoner.diagnostics(),
        )
        .unwrap();
        assert!(ontology.axiom_count() > 2);
    }

    #[test]
    fn nary_equivalent_classes_emits_all_pairs() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        let c = ontology
            .entity_id("http://ex.org/C", EntityKind::Class)
            .unwrap();
        ontology
            .add_axiom(Axiom::EquivalentClasses(vec![a, b, c]))
            .unwrap();

        let triples = core_to_triples(&ontology).unwrap();
        let equiv: Vec<_> = triples
            .iter()
            .filter(|t| t.predicate.as_str() == OWL_EQUIV_CLASS)
            .map(|t| {
                let sub = match &t.subject {
                    oxrdf::NamedOrBlankNode::NamedNode(n) => n.as_str().to_string(),
                    _ => panic!("expected named node subject"),
                };
                let obj = match &t.object {
                    Term::NamedNode(n) => n.as_str().to_string(),
                    _ => panic!("expected named node object"),
                };
                (sub, obj)
            })
            .collect();
        assert_eq!(equiv.len(), 3);
        assert!(equiv.contains(&("http://ex.org/A".into(), "http://ex.org/B".into())));
        assert!(equiv.contains(&("http://ex.org/A".into(), "http://ex.org/C".into())));
        assert!(equiv.contains(&("http://ex.org/B".into(), "http://ex.org/C".into())));
    }

    #[test]
    fn negative_assertions_error_on_triple_export() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/a", EntityKind::Individual)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/b", EntityKind::Individual)
            .unwrap();
        let p = ontology
            .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
            .unwrap();
        ontology
            .add_axiom(Axiom::NegativeObjectPropertyAssertion {
                subject: a,
                property: p,
                object: b,
            })
            .unwrap();

        let err = core_to_triples(&ontology).unwrap_err();
        assert!(err.to_string().contains("NegativeObjectPropertyAssertion"));
    }

    #[test]
    fn merge_entity_kind_conflict_records_clash() {
        let mut ontology = Ontology::new();
        ontology
            .entity_id("http://ex.org/a", EntityKind::Individual)
            .unwrap();
        ontology
            .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
            .unwrap();

        let triples = vec![
            data_property_triple(
                "http://ex.org/a",
                "http://ex.org/p",
                &DataLiteral {
                    lexical: "1".into(),
                    datatype: "http://www.w3.org/2001/XMLSchema#integer".into(),
                },
            )
            .unwrap(),
        ];
        let report = merge_triples_into_ontology(&mut ontology, &triples, &[]).unwrap();
        assert!(
            report
                .clashes
                .iter()
                .any(|c| c.contains("entity kind mismatch"))
        );
        assert_eq!(report.inferred_axioms, 0);
    }

    #[test]
    fn merge_entity_limit_checked_before_insert() {
        let mut ontology = Ontology::new();
        ontology
            .entity_id("http://ex.org/existing", EntityKind::Class)
            .unwrap();

        let limits = MergeLimits {
            max_entities: 1,
            max_axioms: 10_000_000,
        };
        let triples =
            vec![triple("http://ex.org/new", RDFS_SUBCLASS, "http://ex.org/existing").unwrap()];
        let err = merge_triples_into_ontology_with_limits(&mut ontology, &triples, &[], limits)
            .unwrap_err();
        assert!(err.to_string().contains("entity limit"));
        assert_eq!(ontology.entity_count(), 1);
    }
}
