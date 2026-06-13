//! ALC tableau: expansion, clash detection, blocking, taxonomy extraction.

mod block;
mod cache;
mod clash;
mod expand;

use std::collections::{HashMap, HashSet, VecDeque};

use ontologos_core::{CeId, ClassExpr, EntityId, EntityKind, Ontology, RoleExpr, Taxonomy};

use crate::clause::Clause;
use crate::dl_ontology::DlOntology;
use crate::Error;

/// Skip pairwise entailment inference when the ontology has too many named classes.
const MAX_CLASSES_FOR_ENTAILMENT_INFER: usize = 128;

/// Facts from DL saturation to seed the initial tableau state.
#[derive(Debug, Default, Clone)]
pub struct TableauSeed {
    /// Additional subsumptions `C ⊑ D` (class expression ids).
    pub subsumptions: Vec<(CeId, CeId)>,
    /// Derived `∃r.C ⊑ D` clauses.
    pub existentials: Vec<(RoleExpr, CeId, CeId)>,
    /// Saturated atomic role subsumptions `r ⊑ s`.
    pub role_subsumptions: Vec<(EntityId, EntityId)>,
}

/// ALC tableau classifier entry point.
#[derive(Debug, Default)]
pub struct AlcClassifier;

impl AlcClassifier {
    /// Construct a tableau classifier.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Classify using tableau.
    pub fn classify(&self, ontology: &Ontology) -> Result<Taxonomy, Error> {
        classify(ontology)
    }

    /// Classify with saturation-derived seed facts.
    pub fn classify_with_seed(
        &self,
        ontology: &Ontology,
        seed: &TableauSeed,
    ) -> Result<Taxonomy, Error> {
        classify_with_seed(ontology, seed)
    }
}

/// Classify via tableau on clausified ontology.
pub fn classify(ontology: &Ontology) -> Result<Taxonomy, Error> {
    classify_with_seed(ontology, &TableauSeed::default())
}

/// Classify with optional saturation seed.
pub fn classify_with_seed(ontology: &Ontology, seed: &TableauSeed) -> Result<Taxonomy, Error> {
    let dl = DlOntology::from_ontology(ontology)?;
    run_tableau(&dl, seed)
}

/// Tableau consistency test.
pub fn is_consistent(ontology: &Ontology) -> Result<bool, Error> {
    let dl = DlOntology::from_ontology(ontology)?;
    let top = dl
        .core()
        .dl()
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Top => Some(id),
            _ => None,
        })
        .ok_or_else(|| Error::Message("missing ⊤".into()))?;
    let mut branch = Branch::new(&dl, &TableauSeed::default());
    branch.assert(0, top);
    branch.expand()
}

fn run_tableau(dl: &DlOntology, seed: &TableauSeed) -> Result<Taxonomy, Error> {
    let mut subsumptions = Vec::new();
    for clause in dl.clauses().clauses() {
        if let Clause::Subsumption { sub, sup } = clause {
            if let (Some(a), Some(b)) = (atomic_entity(dl, *sub), atomic_entity(dl, *sup)) {
                subsumptions.push((a, b));
            }
        }
    }
    for &(sub, sup) in &seed.subsumptions {
        if let (Some(a), Some(b)) = (atomic_entity(dl, sub), atomic_entity(dl, sup)) {
            subsumptions.push((a, b));
        }
    }

    let classes: Vec<EntityId> = dl
        .core()
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::Class)
        .map(|(id, _)| id)
        .collect();

    let mut unsatisfiable = Vec::new();
    let class_count = classes.len();
    for class in classes {
        if !is_satisfiable(dl, class, seed)? {
            unsatisfiable.push(class);
        }
    }

    if class_count <= MAX_CLASSES_FOR_ENTAILMENT_INFER {
        subsumptions.extend(infer_named_subsumptions(dl, seed)?);
    }
    subsumptions.sort_unstable_by_key(|(a, b)| (a.0, b.0));
    subsumptions.dedup();

    Ok(Taxonomy {
        subsumptions,
        equivalences: Vec::new(),
        unsatisfiable,
    })
}

fn is_satisfiable(dl: &DlOntology, class: EntityId, seed: &TableauSeed) -> Result<bool, Error> {
    let ce = dl
        .core()
        .dl()
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == class => Some(id),
            _ => None,
        })
        .ok_or_else(|| Error::Message(format!("missing CE for class {:?}", class.0)))?;
    let mut branch = Branch::new(dl, seed);
    branch.assert(0, ce);
    branch.expand()
}

fn infer_named_subsumptions(
    dl: &DlOntology,
    seed: &TableauSeed,
) -> Result<Vec<(EntityId, EntityId)>, Error> {
    let classes: Vec<EntityId> = dl
        .core()
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::Class)
        .map(|(id, _)| id)
        .collect();
    let mut out = Vec::new();
    for &sub in &classes {
        for &sup in &classes {
            if sub != sup && entails(dl, sub, sup, seed)? {
                out.push((sub, sup));
            }
        }
    }
    Ok(out)
}

fn entails(
    dl: &DlOntology,
    sub: EntityId,
    sup: EntityId,
    seed: &TableauSeed,
) -> Result<bool, Error> {
    let store = dl.core().dl();
    let sub_ce = store
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == sub => Some(id),
            _ => None,
        })
        .ok_or_else(|| Error::Message("missing sub CE".into()))?;
    let sup_ce = store
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == sup => Some(id),
            _ => None,
        })
        .ok_or_else(|| Error::Message("missing sup CE".into()))?;
    let mut branch = Branch::new(dl, seed);
    branch.assert(0, sub_ce);
    branch.assert_negation_of(0, sup_ce);
    Ok(!branch.expand()?)
}

fn saturate_role_hierarchy(role_hierarchy: &mut HashMap<EntityId, HashSet<EntityId>>) {
    let mut changed = true;
    while changed {
        changed = false;
        let pairs: Vec<(EntityId, EntityId)> = role_hierarchy
            .iter()
            .flat_map(|(&a, ss)| ss.iter().map(move |&b| (a, b)))
            .collect();
        for (a, b) in pairs {
            if let Some(bb) = role_hierarchy.get(&b).cloned() {
                for c in bb {
                    if role_hierarchy.entry(a).or_default().insert(c) {
                        changed = true;
                    }
                }
            }
        }
    }
}

fn atomic_entity(dl: &DlOntology, ce: CeId) -> Option<EntityId> {
    match dl.core().dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct World {
    labels: HashSet<CeId>,
    negated: HashSet<CeId>,
    queue: VecDeque<CeId>,
    blocked: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Branch<'a> {
    pub(crate) dl: &'a DlOntology,
    pub(crate) worlds: Vec<World>,
    pub(crate) edges: Vec<(usize, RoleExpr, usize)>,
    pub(crate) clash: bool,
    pub(crate) disjoint: Vec<(CeId, CeId)>,
    pub(crate) existentials: Vec<(RoleExpr, CeId, CeId)>,
    pub(crate) universals: Vec<(CeId, RoleExpr, CeId)>,
    pub(crate) tbox_subsumptions: Vec<(CeId, CeId)>,
    pub(crate) role_hierarchy: HashMap<EntityId, HashSet<EntityId>>,
    pub(crate) cache: cache::UnsatCache,
    pub(crate) expansions: u32,
}

impl<'a> Branch<'a> {
    fn new(dl: &'a DlOntology, seed: &TableauSeed) -> Self {
        let mut disjoint = Vec::new();
        let mut existentials = seed.existentials.clone();
        let mut universals = Vec::new();
        let mut tbox_subsumptions = seed.subsumptions.clone();
        let mut role_hierarchy: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();

        for clause in dl.clauses().clauses() {
            match clause {
                Clause::Subsumption { sub, sup } => {
                    tbox_subsumptions.push((*sub, *sup));
                }
                Clause::Disjoint { left, right } => disjoint.push((*left, *right)),
                Clause::Existential {
                    property,
                    filler,
                    sup,
                } => existentials.push((property.clone(), *filler, *sup)),
                Clause::Universal {
                    sub,
                    property,
                    filler,
                } => universals.push((*sub, property.clone(), *filler)),
                Clause::RoleSubsumption { sub, sup } => {
                    role_hierarchy.entry(*sub).or_default().insert(*sup);
                }
                _ => {}
            }
        }

        for &(sub, sup) in &seed.role_subsumptions {
            role_hierarchy.entry(sub).or_default().insert(sup);
        }

        saturate_role_hierarchy(&mut role_hierarchy);

        Self {
            dl,
            worlds: vec![World::default()],
            edges: Vec::new(),
            clash: false,
            disjoint,
            existentials,
            universals,
            tbox_subsumptions,
            role_hierarchy,
            cache: cache::UnsatCache::new(),
            expansions: 0,
        }
    }

    fn assert(&mut self, world: usize, ce: CeId) {
        clash::assert_label(self, world, ce);
    }

    fn assert_negation_of(&mut self, world: usize, ce: CeId) {
        clash::assert_negation(self, world, ce);
    }

    fn expand(&mut self) -> Result<bool, Error> {
        loop {
            if self.clash {
                return Ok(false);
            }

            let pending = self.next_pending();
            let Some((world, ce)) = pending else {
                return Ok(true);
            };

            if block::is_blocked(self, world) {
                if block::is_budget_exhausted(self) {
                    return Err(Error::ResourceLimit(block::MAX_EXPANSIONS));
                }
                block::mark_blocked(self, world);
                continue;
            }

            if self.cache.is_unsat(&self.worlds[world].labels) {
                return Ok(false);
            }

            expand::process(self, world, ce)?;
        }
    }

    fn next_pending(&mut self) -> Option<(usize, CeId)> {
        for (idx, world) in self.worlds.iter_mut().enumerate() {
            if let Some(ce) = world.queue.pop_front() {
                return Some((idx, ce));
            }
        }
        None
    }
}
