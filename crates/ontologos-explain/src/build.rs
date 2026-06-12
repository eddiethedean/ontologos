use std::collections::HashMap;

use ontologos_core::{
    Axiom, AxiomId, EntityId, Error as CoreError, InferenceTrace, Ontology, TraceConclusion,
    TracePremise,
};

use crate::{Error, NodeId, ProofGraph, ProofNode, Result};

/// Build a proof graph from an inference trace over `ontology`.
pub fn build_proof_graph(ontology: &Ontology, trace: &InferenceTrace) -> Result<ProofGraph> {
    let mut builder = ProofGraphBuilder::new(ontology);
    builder.ingest(trace)?;
    Ok(builder.finish())
}

struct ProofGraphBuilder<'a> {
    ontology: &'a Ontology,
    nodes: Vec<ProofNode>,
    axiom_nodes: HashMap<AxiomId, NodeId>,
    subsumption_nodes: HashMap<(EntityId, EntityId), NodeId>,
    existential_nodes: HashMap<(EntityId, EntityId, EntityId), NodeId>,
    subproperty_nodes: HashMap<(EntityId, EntityId), NodeId>,
}

impl<'a> ProofGraphBuilder<'a> {
    fn new(ontology: &'a Ontology) -> Self {
        let mut builder = Self {
            ontology,
            nodes: Vec::new(),
            axiom_nodes: HashMap::new(),
            subsumption_nodes: HashMap::new(),
            existential_nodes: HashMap::new(),
            subproperty_nodes: HashMap::new(),
        };
        for (id, axiom) in ontology.axioms().iter() {
            builder.seed_axiom_node(id);
            builder.seed_asserted_structure(axiom);
        }
        builder
    }

    fn seed_asserted_structure(&mut self, axiom: &Axiom) {
        match axiom {
            Axiom::SubClassOf {
                subclass,
                superclass,
            } => {
                self.seed_subsumption_leaf(*subclass, *superclass);
            }
            Axiom::SubClassOfExistential {
                subclass,
                property,
                filler,
            } => {
                self.seed_existential_leaf(*subclass, *property, *filler);
            }
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => {
                self.seed_subproperty_leaf(*sub_property, *super_property);
            }
            _ => {}
        }
    }

    fn seed_subsumption_leaf(&mut self, sub: EntityId, sup: EntityId) {
        if self.subsumption_nodes.contains_key(&(sub, sup)) {
            return;
        }
        let node_id = self.next_id();
        self.nodes.push(ProofNode {
            rule: "asserted".to_string(),
            premises: Vec::new(),
            conclusion_axiom: None,
            conclusion_sub: Some((sub, sup)),
            conclusion_existential: None,
            conclusion_subproperty: None,
        });
        self.subsumption_nodes.insert((sub, sup), node_id);
    }

    fn seed_existential_leaf(&mut self, class: EntityId, property: EntityId, filler: EntityId) {
        if self
            .existential_nodes
            .contains_key(&(class, property, filler))
        {
            return;
        }
        let node_id = self.next_id();
        self.nodes.push(ProofNode {
            rule: "asserted".to_string(),
            premises: Vec::new(),
            conclusion_axiom: None,
            conclusion_sub: None,
            conclusion_existential: Some((class, property, filler)),
            conclusion_subproperty: None,
        });
        self.existential_nodes
            .insert((class, property, filler), node_id);
    }

    fn seed_subproperty_leaf(&mut self, sub: EntityId, sup: EntityId) {
        if self.subproperty_nodes.contains_key(&(sub, sup)) {
            return;
        }
        let node_id = self.next_id();
        self.nodes.push(ProofNode {
            rule: "asserted".to_string(),
            premises: Vec::new(),
            conclusion_axiom: None,
            conclusion_sub: None,
            conclusion_existential: None,
            conclusion_subproperty: Some((sub, sup)),
        });
        self.subproperty_nodes.insert((sub, sup), node_id);
    }

    fn next_id(&self) -> NodeId {
        NodeId(self.nodes.len() as u32)
    }

    fn seed_axiom_node(&mut self, id: AxiomId) {
        if self.axiom_nodes.contains_key(&id) {
            return;
        }
        let node_id = self.next_id();
        self.nodes.push(ProofNode {
            rule: "asserted".to_string(),
            premises: Vec::new(),
            conclusion_axiom: Some(id),
            conclusion_sub: None,
            conclusion_existential: None,
            conclusion_subproperty: None,
        });
        self.axiom_nodes.insert(id, node_id);
    }

    fn ingest(&mut self, trace: &InferenceTrace) -> Result<()> {
        for step in &trace.steps {
            let premises: Vec<NodeId> = step
                .premises
                .iter()
                .map(|p| self.resolve_premise(p))
                .collect::<Result<Vec<_>>>()?;

            let node_id = self.next_id();
            let mut node = ProofNode {
                rule: step.rule.clone(),
                premises,
                conclusion_axiom: None,
                conclusion_sub: None,
                conclusion_existential: None,
                conclusion_subproperty: None,
            };

            match step.conclusion {
                TraceConclusion::Axiom { id } => {
                    self.ontology.axiom(id).map_err(Error::Core)?;
                    node.conclusion_axiom = Some(id);
                    self.axiom_nodes.insert(id, node_id);
                }
                TraceConclusion::SubClassOf { sub, sup } => {
                    node.conclusion_sub = Some((sub, sup));
                    self.subsumption_nodes.insert((sub, sup), node_id);
                }
                TraceConclusion::Existential {
                    class,
                    property,
                    filler,
                } => {
                    node.conclusion_existential = Some((class, property, filler));
                    self.existential_nodes
                        .insert((class, property, filler), node_id);
                }
                TraceConclusion::SubObjectPropertyOf { sub, sup } => {
                    node.conclusion_subproperty = Some((sub, sup));
                    self.subproperty_nodes.insert((sub, sup), node_id);
                }
            }

            self.nodes.push(node);
        }

        validate_acyclic(&self.nodes)?;
        Ok(())
    }

    fn resolve_premise(&self, premise: &TracePremise) -> Result<NodeId> {
        match premise {
            TracePremise::Axiom { id } => {
                self.ontology.axiom(*id).map_err(Error::Core)?;
                self.axiom_nodes.get(id).copied().ok_or_else(|| {
                    Error::Core(CoreError::Message(format!(
                        "missing justification for axiom {}",
                        id.0
                    )))
                })
            }
            TracePremise::SubClassOf { sub, sup } => self
                .subsumption_nodes
                .get(&(*sub, *sup))
                .copied()
                .ok_or_else(|| {
                    Error::Core(CoreError::Message(format!(
                        "missing justification for subsumption {:?} ⊑ {:?}",
                        sub.0, sup.0
                    )))
                }),
            TracePremise::Existential {
                class,
                property,
                filler,
            } => self
                .existential_nodes
                .get(&(*class, *property, *filler))
                .copied()
                .ok_or_else(|| {
                    Error::Core(CoreError::Message(format!(
                        "missing justification for existential ({:?}, {:?}, {:?})",
                        class.0, property.0, filler.0
                    )))
                }),
            TracePremise::SubObjectPropertyOf { sub, sup } => self
                .subproperty_nodes
                .get(&(*sub, *sup))
                .copied()
                .ok_or_else(|| {
                    Error::Core(CoreError::Message(format!(
                        "missing justification for subproperty {:?} ⊑ {:?}",
                        sub.0, sup.0
                    )))
                }),
        }
    }

    fn finish(self) -> ProofGraph {
        ProofGraph { nodes: self.nodes }
    }
}

fn validate_acyclic(nodes: &[ProofNode]) -> Result<()> {
    let mut visiting = vec![false; nodes.len()];
    let mut done = vec![false; nodes.len()];

    for start in 0..nodes.len() {
        if !done[start] && dfs_acyclic(nodes, start, &mut visiting, &mut done) {
            return Err(Error::Core(CoreError::Message(
                "proof graph contains a cycle".into(),
            )));
        }
    }
    Ok(())
}

fn dfs_acyclic(nodes: &[ProofNode], idx: usize, visiting: &mut [bool], done: &mut [bool]) -> bool {
    if done[idx] {
        return false;
    }
    if visiting[idx] {
        return true;
    }
    visiting[idx] = true;
    for premise in &nodes[idx].premises {
        if dfs_acyclic(nodes, premise.0 as usize, visiting, done) {
            return true;
        }
    }
    visiting[idx] = false;
    done[idx] = true;
    false
}
