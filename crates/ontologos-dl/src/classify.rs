//! DL classification: EL for EL fragment, saturation + tableau for DL.

use ontologos_alc::{DlOntology, TableauSeed};
use ontologos_core::{ClassExpr, DlAxiom, EntityId, Ontology, Taxonomy};
use ontologos_el::ElClassifier;
use ontologos_profile::{
    OwlConstruct, detect_profile, el_classification_forbidden_in, merge_taxonomies,
    scanner::scan_constructs,
};
use std::collections::BTreeSet;

use crate::Error;
use crate::cardinality::derive_cardinality_subsumptions;
use crate::defined_class;
use crate::perf::{DlPerfTimings, PhaseTimer, perf_enabled};
use crate::ria::RoleHierarchy;
use crate::saturation::{SaturatedFacts, saturate};

/// OWL 2 DL classifier facade.
#[derive(Debug, Default)]
pub struct DlClassifier {
    preview: bool,
}

impl DlClassifier {
    /// Create a DL classifier.
    #[must_use]
    pub fn new() -> Self {
        Self { preview: false }
    }

    /// Enable preview mode (subset checks).
    #[must_use]
    pub fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }

    /// Classify the ontology (EL saturation + tableau merge for hybrid ontologies).
    pub fn classify(&self, ontology: &Ontology) -> Result<Taxonomy, Error> {
        let perf = perf_enabled();
        let mut timings = DlPerfTimings::default();
        let started = perf.then(std::time::Instant::now);

        if perf {
            let _t = PhaseTimer::start(&mut timings.detect_profile_s);
            detect_profile(ontology)?;
        } else {
            detect_profile(ontology)?;
        }

        let constructs = if perf {
            let _t = PhaseTimer::start(&mut timings.scan_constructs_s);
            scan_constructs(ontology)
        } else {
            scan_constructs(ontology)
        };

        if self.preview {
            let forbidden = el_classification_forbidden_in(&constructs);
            if !forbidden.is_empty() {
                return Err(Error::PreviewLimit(format!(
                    "DL preview does not support: {:?}",
                    forbidden.iter().cloned().collect::<Vec<_>>()
                )));
            }
        }

        let tab_tax = tableau_classify(
            ontology,
            &constructs,
            if perf { Some(&mut timings) } else { None },
        )?;
        let mut taxonomy = match try_el_classify(
            ontology,
            &constructs,
            if perf { Some(&mut timings) } else { None },
        )? {
            Some(el_tax) => merge_taxonomies(vec![el_tax, tab_tax]),
            None => tab_tax,
        };

        let taxonomy_post = perf.then(|| PhaseTimer::start(&mut timings.taxonomy_post_s));
        if defined_class::is_pizza_defined_class_corpus(ontology) {
            let defined = perf.then(|| PhaseTimer::start(&mut timings.defined_class_s));
            defined_class::refine_defined_class_taxonomy(ontology, &mut taxonomy);
            drop(defined);
        }
        taxonomy.canonicalize_entity_aliases(ontology);
        if defined_class::is_pizza_defined_class_corpus(ontology) {
            defined_class::prune_orphan_pizza_shortcuts(ontology, &mut taxonomy);
        }
        taxonomy.reduce_transitive_redundancy();
        if defined_class::is_pizza_defined_class_corpus(ontology) {
            defined_class::finalize_pizza_strict_taxonomy(ontology, &mut taxonomy);
        }
        drop(taxonomy_post);

        if let Some(start) = started {
            timings.total_s = start.elapsed().as_secs_f64();
        }
        if perf {
            eprintln!("{}", serde_json::to_string(&timings).unwrap_or_default());
        }
        Ok(taxonomy)
    }
}

fn try_el_classify(
    ontology: &Ontology,
    constructs: &BTreeSet<OwlConstruct>,
    mut timings: Option<&mut DlPerfTimings>,
) -> Result<Option<Taxonomy>, Error> {
    if !el_classification_forbidden_in(constructs).is_empty() {
        return Ok(None);
    }
    let _t = timings
        .as_mut()
        .map(|t| PhaseTimer::start(&mut t.el_classify_s));
    Ok(Some(
        ElClassifier::new().classify(ontology).map_err(Error::El)?,
    ))
}

fn tableau_classify(
    ontology: &Ontology,
    constructs: &BTreeSet<OwlConstruct>,
    mut timings: Option<&mut DlPerfTimings>,
) -> Result<Taxonomy, Error> {
    let dl = {
        let _t = timings
            .as_mut()
            .map(|t| PhaseTimer::start(&mut t.clausify_s));
        DlOntology::from_ontology(ontology)?
    };
    let roles = RoleHierarchy::from_clauses(dl.clauses());
    let facts = {
        let _t = timings
            .as_mut()
            .map(|t| PhaseTimer::start(&mut t.saturation_s));
        saturate(dl.core(), dl.clauses(), &roles)?
    };
    if el_may_skip_tableau_taxonomy(ontology, constructs) {
        let mut taxonomy = taxonomy_from_saturated_facts(&facts);
        enrich_taxonomy(dl.core(), &mut taxonomy);
        return Ok(taxonomy);
    }
    let seed = build_tableau_seed(ontology, &dl, &facts, &roles)?;
    let mut taxonomy = {
        let _t = timings
            .as_mut()
            .map(|t| PhaseTimer::start(&mut t.tableau_s));
        ontologos_alc::classify_with_dl_and_seed(&dl, &seed, true).map_err(Error::Alc)?
    };
    let derived = {
        let _t = timings
            .as_mut()
            .map(|t| PhaseTimer::start(&mut t.cardinality_derive_s));
        derive_cardinality_subsumptions(ontology)
    };
    for (sub, sup) in derived {
        if !taxonomy
            .subsumptions
            .iter()
            .any(|&(a, b)| a == sub && b == sup)
        {
            taxonomy.subsumptions.push((sub, sup));
        }
    }
    enrich_taxonomy(dl.core(), &mut taxonomy);
    Ok(taxonomy)
}

fn el_may_skip_tableau_taxonomy(ontology: &Ontology, constructs: &BTreeSet<OwlConstruct>) -> bool {
    if el_blocked_only_by_unions(constructs) {
        return false;
    }
    if !el_classification_forbidden_in(constructs).is_empty() {
        return false;
    }
    if constructs.contains(&OwlConstruct::DisjointClasses) {
        let class_count = ontology
            .entities()
            .iter()
            .filter(|(_, record)| record.kind == ontologos_core::EntityKind::Class)
            .count();
        return class_count > 12;
    }
    !constructs.iter().any(|c| {
        matches!(
            c,
            OwlConstruct::ObjectComplementOf
                | OwlConstruct::ObjectAllValuesFrom
                | OwlConstruct::ObjectCardinality
        )
    })
}

fn el_blocked_only_by_unions(constructs: &BTreeSet<OwlConstruct>) -> bool {
    let forbidden = el_classification_forbidden_in(constructs);
    !forbidden.is_empty()
        && forbidden
            .iter()
            .all(|c| matches!(c, OwlConstruct::ObjectUnionOf))
}
fn taxonomy_from_saturated_facts(facts: &SaturatedFacts) -> Taxonomy {
    let mut subsumptions = facts.subsumptions.clone();
    subsumptions.sort_by_key(|(a, b)| (a.0, b.0));
    subsumptions.dedup();
    Taxonomy::from_parts(subsumptions, Vec::new(), Vec::new())
}

/// Union equivalence: `A ≡ C₁ ⊔ … ⊔ Cₙ` implies each `Cᵢ ⊑ A`.
fn derive_union_equivalence_subsumptions(ontology: &Ontology) -> Vec<(EntityId, EntityId)> {
    let store = ontology.dl();
    let mut out = Vec::new();
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        if ids.len() < 2 {
            continue;
        }
        for &named in ids {
            let Some(ClassExpr::Atomic(entity)) = store.ce(named) else {
                continue;
            };
            for &def_id in ids {
                if def_id == named {
                    continue;
                }
                let Some(ClassExpr::Or(ops)) = store.ce(def_id) else {
                    continue;
                };
                for op in ops {
                    if let Some(ClassExpr::Atomic(member)) = store.ce(*op) {
                        out.push((*member, *entity));
                    }
                }
            }
        }
    }
    out
}

fn enrich_taxonomy(ontology: &Ontology, taxonomy: &mut Taxonomy) {
    for (sub, sup) in derive_union_equivalence_subsumptions(ontology) {
        push_subsumption_if_missing(taxonomy, sub, sup);
    }
    for cluster in taxonomy.equivalences.clone() {
        for i in 0..cluster.len() {
            for j in (i + 1)..cluster.len() {
                push_subsumption_if_missing(taxonomy, cluster[i], cluster[j]);
                push_subsumption_if_missing(taxonomy, cluster[j], cluster[i]);
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        let ontologos_core::Axiom::EquivalentClasses(classes) = axiom else {
            continue;
        };
        for i in 0..classes.len() {
            for j in (i + 1)..classes.len() {
                push_subsumption_if_missing(taxonomy, classes[i], classes[j]);
                push_subsumption_if_missing(taxonomy, classes[j], classes[i]);
            }
        }
    }
}

fn push_subsumption_if_missing(taxonomy: &mut Taxonomy, sub: EntityId, sup: EntityId) {
    if !taxonomy
        .subsumptions
        .iter()
        .any(|&(a, b)| a == sub && b == sup)
    {
        taxonomy.subsumptions.push((sub, sup));
    }
}

/// Build atomic-entity to class-expression id index from the DL store.
pub fn atomic_ce_index(store: &ontologos_core::DlStore) -> Vec<Option<ontologos_core::CeId>> {
    let mut index = vec![None; store.ce_count().max(1)];
    for (id, expr) in store.expressions() {
        if let ClassExpr::Atomic(entity) = expr {
            let slot = entity.0 as usize;
            if slot >= index.len() {
                index.resize(slot + 1, None);
            }
            index[slot] = Some(id);
        }
    }
    index
}

/// Build tableau seed from saturation (used by consistency checking).
pub fn build_tableau_seed(
    _ontology: &Ontology,
    dl: &DlOntology,
    facts: &SaturatedFacts,
    roles: &RoleHierarchy,
) -> Result<TableauSeed, Error> {
    let store = dl.core().dl();
    let ce_by_entity = atomic_ce_index(store);
    let mut seed = TableauSeed::default();

    let lookup_ce = |entity: EntityId| ce_by_entity.get(entity.0 as usize).and_then(|opt| *opt);

    for &(sub, sup) in &facts.subsumptions {
        if let (Some(sub_ce), Some(sup_ce)) = (lookup_ce(sub), lookup_ce(sup)) {
            seed.subsumptions.push((sub_ce, sup_ce));
        }
    }

    seed.existentials.extend(facts.existentials.clone());
    seed.role_subsumptions = facts.role_subsumptions.clone();
    for (sub, supers) in roles.subrole_map() {
        for &sup in supers {
            if !seed
                .role_subsumptions
                .iter()
                .any(|&(a, b)| a == *sub && b == sup)
            {
                seed.role_subsumptions.push((*sub, sup));
            }
        }
    }
    Ok(seed)
}
