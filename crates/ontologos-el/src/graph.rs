use std::collections::{HashMap, HashSet, VecDeque};

use ontologos_core::{Axiom, EntityId, InferenceTrace, Ontology, TracePremise};

use crate::trace::{
    existential_premise, push_existential, push_subproperty, push_subsumption, subproperty_premise,
    subsumption_premise, ElRule,
};

/// In-memory EL completion graph (inference overlay; does not mutate asserted axioms).
#[derive(Debug, Default)]
pub struct CompletionGraph {
    subsumptions: HashSet<(EntityId, EntityId)>,
    existentials: HashSet<(EntityId, EntityId, EntityId)>,
    subproperties: HashSet<(EntityId, EntityId)>,
    domains: HashMap<EntityId, EntityId>,
    todo_sub: VecDeque<(EntityId, EntityId)>,
    todo_ex: VecDeque<(EntityId, EntityId, EntityId)>,
    todo_sp: VecDeque<(EntityId, EntityId)>,
    record_traces: bool,
    trace: InferenceTrace,
}

impl CompletionGraph {
    /// Seed the graph from ontology axioms (after normal-form preprocessing).
    pub fn seed(ontology: &Ontology) -> Self {
        let mut graph = Self::default();
        for (_id, axiom) in ontology.axioms().iter() {
            match axiom {
                Axiom::SubClassOf {
                    subclass,
                    superclass,
                } => {
                    graph.add_subsumption_seed(*subclass, *superclass);
                }
                Axiom::SubClassOfExistential {
                    subclass,
                    property,
                    filler,
                } => {
                    graph.add_existential_seed(*subclass, *property, *filler);
                }
                Axiom::EquivalentClasses(classes) => {
                    for i in 0..classes.len() {
                        for j in (i + 1)..classes.len() {
                            graph.add_subsumption_seed(classes[i], classes[j]);
                            graph.add_subsumption_seed(classes[j], classes[i]);
                        }
                    }
                }
                Axiom::SubObjectPropertyOf {
                    sub_property,
                    super_property,
                } => {
                    graph.add_subproperty_seed(*sub_property, *super_property);
                }
                Axiom::EquivalentObjectProperties(properties) => {
                    for i in 0..properties.len() {
                        for j in (i + 1)..properties.len() {
                            graph.add_subproperty_seed(properties[i], properties[j]);
                            graph.add_subproperty_seed(properties[j], properties[i]);
                        }
                    }
                }
                Axiom::ObjectPropertyDomain { property, domain } => {
                    graph.domains.insert(*property, *domain);
                }
                _ => {}
            }
        }
        graph
    }

    /// Enable inference trace recording.
    #[must_use]
    pub fn with_traces(mut self, enabled: bool) -> Self {
        self.record_traces = enabled;
        self
    }

    /// Take the recorded inference trace.
    #[must_use]
    pub fn into_trace(self) -> InferenceTrace {
        self.trace
    }

    pub fn subsumptions(&self) -> &HashSet<(EntityId, EntityId)> {
        &self.subsumptions
    }

    pub fn is_subsumed(&self, sub: EntityId, sup: EntityId) -> bool {
        sub == sup || self.subsumptions.contains(&(sub, sup))
    }

    fn add_subsumption_seed(&mut self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup || !self.subsumptions.insert((sub, sup)) {
            return false;
        }
        self.todo_sub.push_back((sub, sup));
        true
    }

    fn add_existential_seed(
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

    fn add_subproperty_seed(&mut self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup || !self.subproperties.insert((sub, sup)) {
            return false;
        }
        self.todo_sp.push_back((sub, sup));
        true
    }

    fn infer_subsumption(
        &mut self,
        rule: ElRule,
        premises: Vec<TracePremise>,
        sub: EntityId,
        sup: EntityId,
    ) -> bool {
        if sub == sup || !self.subsumptions.insert((sub, sup)) {
            return false;
        }
        if self.record_traces {
            push_subsumption(&mut self.trace, rule, premises, sub, sup);
        }
        self.todo_sub.push_back((sub, sup));
        true
    }

    fn infer_existential(
        &mut self,
        rule: ElRule,
        premises: Vec<TracePremise>,
        class: EntityId,
        property: EntityId,
        filler: EntityId,
    ) -> bool {
        if !self.existentials.insert((class, property, filler)) {
            return false;
        }
        if self.record_traces {
            push_existential(&mut self.trace, rule, premises, class, property, filler);
        }
        self.todo_ex.push_back((class, property, filler));
        true
    }

    fn infer_subproperty(
        &mut self,
        rule: ElRule,
        premises: Vec<TracePremise>,
        sub: EntityId,
        sup: EntityId,
    ) -> bool {
        if sub == sup || !self.subproperties.insert((sub, sup)) {
            return false;
        }
        if self.record_traces {
            push_subproperty(&mut self.trace, rule, premises, sub, sup);
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
            self.infer_subsumption(
                ElRule::SubTransForward,
                vec![subsumption_premise(sub, sup), subsumption_premise(sup, x)],
                sub,
                x,
            );
        }

        let backward: Vec<EntityId> = self
            .subsumptions
            .iter()
            .filter_map(|&(x, s)| (s == sub).then_some(x))
            .collect();
        for x in backward {
            self.infer_subsumption(
                ElRule::SubTransBackward,
                vec![subsumption_premise(x, sub), subsumption_premise(sub, sup)],
                x,
                sup,
            );
        }

        let existentials: Vec<(EntityId, EntityId, EntityId)> =
            self.existentials.iter().copied().collect();
        for (c, r, d) in existentials {
            if c == sub && self.is_subsumed(d, sup) {
                self.infer_existential(
                    ElRule::ExFillerSub,
                    vec![existential_premise(c, r, d), subsumption_premise(d, sup)],
                    c,
                    r,
                    sup,
                );
            }
        }
    }

    fn apply_existential(&mut self, class: EntityId, property: EntityId, filler: EntityId) {
        if let Some(&domain) = self.domains.get(&property) {
            self.infer_subsumption(
                ElRule::PropertyDomain,
                vec![existential_premise(class, property, filler)],
                class,
                domain,
            );
        }

        let subs: Vec<(EntityId, EntityId)> = self.subsumptions.iter().copied().collect();
        for (sub, sup) in subs {
            if sub == filler {
                self.infer_existential(
                    ElRule::ExFillerSub,
                    vec![
                        existential_premise(class, property, filler),
                        subsumption_premise(sub, sup),
                    ],
                    class,
                    property,
                    sup,
                );
            }
        }

        let subprops: Vec<(EntityId, EntityId)> = self.subproperties.iter().copied().collect();
        for (sub, sup) in subprops {
            if sup == property {
                self.infer_existential(
                    ElRule::ExSubProp,
                    vec![
                        existential_premise(class, property, filler),
                        subproperty_premise(sub, sup),
                    ],
                    class,
                    sub,
                    filler,
                );
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
            self.infer_subproperty(
                ElRule::SubPropTransForward,
                vec![subproperty_premise(sub, sup), subproperty_premise(sup, x)],
                sub,
                x,
            );
        }

        let backward: Vec<EntityId> = self
            .subproperties
            .iter()
            .filter_map(|&(x, s)| (s == sub).then_some(x))
            .collect();
        for x in backward {
            self.infer_subproperty(
                ElRule::SubPropTransBackward,
                vec![subproperty_premise(x, sub), subproperty_premise(sub, sup)],
                x,
                sup,
            );
        }

        let existentials: Vec<(EntityId, EntityId, EntityId)> =
            self.existentials.iter().copied().collect();
        for (c, r, d) in existentials {
            if r == sup {
                self.infer_existential(
                    ElRule::ExSuperProp,
                    vec![existential_premise(c, r, d), subproperty_premise(sub, sup)],
                    c,
                    sub,
                    d,
                );
            }
        }
    }
}
