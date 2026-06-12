use std::collections::{HashSet, VecDeque};

use ontologos_core::{EntityId, Ontology};

/// In-memory EL completion graph (inference overlay; does not mutate asserted axioms).
#[derive(Debug, Default)]
pub struct CompletionGraph {
    subsumptions: HashSet<(EntityId, EntityId)>,
    existentials: HashSet<(EntityId, EntityId, EntityId)>,
    subproperties: HashSet<(EntityId, EntityId)>,
    todo_sub: VecDeque<(EntityId, EntityId)>,
    todo_ex: VecDeque<(EntityId, EntityId, EntityId)>,
    todo_sp: VecDeque<(EntityId, EntityId)>,
}

impl CompletionGraph {
    /// Seed the graph from ontology axioms (after normal-form preprocessing).
    pub fn seed(ontology: &Ontology) -> Self {
        let mut graph = Self::default();
        for (_id, axiom) in ontology.axioms().iter() {
            match axiom {
                ontologos_core::Axiom::SubClassOf {
                    subclass,
                    superclass,
                } => {
                    graph.add_subsumption(*subclass, *superclass);
                }
                ontologos_core::Axiom::SubClassOfExistential {
                    subclass,
                    property,
                    filler,
                } => {
                    graph.add_existential(*subclass, *property, *filler);
                }
                ontologos_core::Axiom::EquivalentClasses(classes) => {
                    for i in 0..classes.len() {
                        for j in (i + 1)..classes.len() {
                            graph.add_subsumption(classes[i], classes[j]);
                            graph.add_subsumption(classes[j], classes[i]);
                        }
                    }
                }
                ontologos_core::Axiom::SubObjectPropertyOf {
                    sub_property,
                    super_property,
                } => {
                    graph.add_subproperty(*sub_property, *super_property);
                }
                ontologos_core::Axiom::EquivalentObjectProperties(properties) => {
                    for i in 0..properties.len() {
                        for j in (i + 1)..properties.len() {
                            graph.add_subproperty(properties[i], properties[j]);
                            graph.add_subproperty(properties[j], properties[i]);
                        }
                    }
                }
                _ => {}
            }
        }
        graph
    }

    pub fn subsumptions(&self) -> &HashSet<(EntityId, EntityId)> {
        &self.subsumptions
    }

    /// Whether the saturated graph contains `class ⊑ ∃property.filler`.
    #[cfg(test)]
    pub(crate) fn has_existential(
        &self,
        class: EntityId,
        property: EntityId,
        filler: EntityId,
    ) -> bool {
        self.existentials.contains(&(class, property, filler))
    }

    pub fn is_subsumed(&self, sub: EntityId, sup: EntityId) -> bool {
        sub == sup || self.subsumptions.contains(&(sub, sup))
    }

    pub fn add_subsumption(&mut self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup || !self.subsumptions.insert((sub, sup)) {
            return false;
        }
        self.todo_sub.push_back((sub, sup));
        true
    }

    pub fn add_existential(
        &mut self,
        class: EntityId,
        property: EntityId,
        filler: EntityId,
    ) -> bool {
        if !self.existentials.insert((class, property, filler)) {
            return false;
        }
        self.todo_ex.push_back((class, property, filler));
        true
    }

    pub fn add_subproperty(&mut self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup || !self.subproperties.insert((sub, sup)) {
            return false;
        }
        self.todo_sp.push_back((sub, sup));
        true
    }

    pub fn saturate(&mut self) {
        loop {
            let progressed =
                self.drain_sub_queue() || self.drain_ex_queue() || self.drain_sp_queue();
            if !progressed {
                break;
            }
        }
    }

    fn drain_sub_queue(&mut self) -> bool {
        let mut progressed = false;
        while let Some((sub, sup)) = self.todo_sub.pop_front() {
            progressed = true;
            self.apply_subsumption(sub, sup);
        }
        progressed
    }

    fn drain_ex_queue(&mut self) -> bool {
        let mut progressed = false;
        while let Some((c, r, d)) = self.todo_ex.pop_front() {
            progressed = true;
            self.apply_existential(c, r, d);
        }
        progressed
    }

    fn drain_sp_queue(&mut self) -> bool {
        let mut progressed = false;
        while let Some((sub, sup)) = self.todo_sp.pop_front() {
            progressed = true;
            self.apply_subproperty(sub, sup);
        }
        progressed
    }

    fn apply_subsumption(&mut self, sub: EntityId, sup: EntityId) {
        let forward: Vec<EntityId> = self
            .subsumptions
            .iter()
            .filter_map(|&(s, x)| (s == sup).then_some(x))
            .collect();
        for x in forward {
            self.add_subsumption(sub, x);
        }

        let backward: Vec<EntityId> = self
            .subsumptions
            .iter()
            .filter_map(|&(x, s)| (s == sub).then_some(x))
            .collect();
        for x in backward {
            self.add_subsumption(x, sup);
        }

        let existentials: Vec<(EntityId, EntityId, EntityId)> =
            self.existentials.iter().copied().collect();
        for (c, r, d) in existentials {
            if c == sub && self.is_subsumed(d, sup) {
                self.add_existential(c, r, sup);
            }
        }
    }

    fn apply_existential(&mut self, class: EntityId, property: EntityId, filler: EntityId) {
        let subs: Vec<(EntityId, EntityId)> = self.subsumptions.iter().copied().collect();
        for (sub, sup) in subs {
            if sub == filler {
                self.add_existential(class, property, sup);
            }
        }

        let subprops: Vec<(EntityId, EntityId)> = self.subproperties.iter().copied().collect();
        for (sub, sup) in subprops {
            if sup == property {
                self.add_existential(class, sub, filler);
            }
        }
    }

    fn apply_subproperty(&mut self, sub: EntityId, sup: EntityId) {
        let forward: Vec<EntityId> = self
            .subproperties
            .iter()
            .filter_map(|&(s, x)| (s == sup).then_some(x))
            .collect();
        for x in forward {
            self.add_subproperty(sub, x);
        }

        let backward: Vec<EntityId> = self
            .subproperties
            .iter()
            .filter_map(|&(x, s)| (s == sub).then_some(x))
            .collect();
        for x in backward {
            self.add_subproperty(x, sup);
        }

        let existentials: Vec<(EntityId, EntityId, EntityId)> =
            self.existentials.iter().copied().collect();
        for (c, r, d) in existentials {
            if r == sup {
                self.add_existential(c, sub, d);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};

    use super::CompletionGraph;

    fn class(ontology: &mut Ontology, iri: &str) -> ontologos_core::EntityId {
        ontology.entity_id(iri, EntityKind::Class).unwrap()
    }

    #[test]
    fn equivalent_properties_seed_mutual_subproperties() {
        let mut ontology = Ontology::new();
        let a = class(&mut ontology, "http://ex.org/A");
        let d = class(&mut ontology, "http://ex.org/D");
        let r = ontology
            .entity_id("http://ex.org/r", EntityKind::ObjectProperty)
            .unwrap();
        let s = ontology
            .entity_id("http://ex.org/s", EntityKind::ObjectProperty)
            .unwrap();
        ontology
            .add_axiom(Axiom::EquivalentObjectProperties(vec![r, s]))
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOfExistential {
                subclass: a,
                property: r,
                filler: d,
            })
            .unwrap();

        let mut graph = CompletionGraph::seed(&ontology);
        graph.saturate();
        assert!(graph.has_existential(a, s, d));
    }

    #[test]
    fn existential_propagates_along_filler_subsumption() {
        let mut ontology = Ontology::new();
        let a = class(&mut ontology, "http://ex.org/A");
        let b = class(&mut ontology, "http://ex.org/B");
        let c = class(&mut ontology, "http://ex.org/C");
        let r = ontology
            .entity_id("http://ex.org/r", EntityKind::ObjectProperty)
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOfExistential {
                subclass: a,
                property: r,
                filler: b,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: b,
                superclass: c,
            })
            .unwrap();

        let mut graph = CompletionGraph::seed(&ontology);
        graph.saturate();
        assert!(graph.has_existential(a, r, c));
    }
}
