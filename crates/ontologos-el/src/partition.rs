use std::collections::{HashMap, HashSet};

use ontologos_core::{axiom_signature, AxiomId, EntityId, Ontology};

/// Maps entities to syntactic partitions for incremental EL updates.
#[derive(Debug, Clone, Default)]
pub struct PartitionIndex {
    entity_partition: HashMap<EntityId, u32>,
    partition_axioms: HashMap<u32, Vec<AxiomId>>,
}

impl PartitionIndex {
    /// Build partitions from all active axioms (root = min entity id in signature).
    pub fn build(ontology: &Ontology) -> Self {
        let mut index = Self::default();
        for (id, axiom) in ontology.axioms().iter() {
            let sig = axiom_signature(axiom);
            let root = sig.iter().map(|e| e.0).min().unwrap_or(id.0);
            let partition = root;
            for entity in sig {
                index.entity_partition.insert(entity, partition);
            }
            index
                .partition_axioms
                .entry(partition)
                .or_default()
                .push(id);
        }
        index
    }

    /// Partitions touched by the given entity signature.
    pub fn partitions_for_signature(&self, sig: &HashSet<EntityId>) -> HashSet<u32> {
        sig.iter()
            .filter_map(|e| self.entity_partition.get(e))
            .copied()
            .collect()
    }

    /// Axiom ids belonging to the given partitions.
    #[allow(dead_code)]
    pub fn axioms_in_partitions(&self, partitions: &HashSet<u32>) -> Vec<AxiomId> {
        let mut ids = Vec::new();
        for pid in partitions {
            if let Some(axioms) = self.partition_axioms.get(pid) {
                ids.extend(axioms.iter().copied());
            }
        }
        ids.sort_by_key(|id| id.0);
        ids.dedup();
        ids
    }

    /// Fraction of partitions affected (0.0..=1.0).
    pub fn affected_fraction(&self, partitions: &HashSet<u32>) -> f64 {
        if self.partition_axioms.is_empty() {
            return 0.0;
        }
        partitions.len() as f64 / self.partition_axioms.len() as f64
    }
}
