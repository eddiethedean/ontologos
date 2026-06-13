//! ABox reasoning: individual typing, `sameAs` / `differentFrom` closure, consistency checks.

#![warn(missing_docs)]

mod closure;
mod report;

use ontologos_core::{EntityId, Ontology};
use thiserror::Error;

pub use closure::{same_as_closure, SameAsClosure};
pub use report::AboxReport;

/// Result type for ABox operations.
pub type Result<T> = std::result::Result<T, Error>;

/// ABox reasoning errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Core ontology error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    /// RL saturation error.
    #[error(transparent)]
    Rl(#[from] ontologos_rl::Error),
    /// Inconsistent ABox (clash detected).
    #[error("ABox inconsistent: {0}")]
    Inconsistent(String),
}

/// Materialize ABox consequences (typing + `sameAs` closure) via RL saturation.
pub fn materialize_abox(ontology: &mut Ontology) -> Result<AboxReport> {
    let closure = same_as_closure(ontology);
    let rl_report = ontologos_rl::RlEngine::new(1)
        .saturate(ontology)
        .map_err(Error::Rl)?;
    Ok(AboxReport {
        same_as_clusters: closure.clusters,
        rl_inferences: rl_report.inferred_total(),
    })
}

/// Returns true when no `differentFrom` clash exists among `sameAs` clusters.
pub fn is_abox_consistent(ontology: &Ontology) -> Result<bool> {
    Ok(!detect_clash(ontology))
}

/// Expand `differentFrom` pairs modulo `sameAs` and detect clashes.
#[must_use]
pub fn different_from_clash(ontology: &Ontology) -> bool {
    detect_clash(ontology)
}

fn detect_clash(ontology: &Ontology) -> bool {
    let closure = same_as_closure(ontology);
    let mut rep_pairs: std::collections::HashSet<(EntityId, EntityId)> =
        std::collections::HashSet::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::DifferentIndividuals(ids) = axiom {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let a = closure.representative(ids[i]);
                    let b = closure.representative(ids[j]);
                    if a == b {
                        return true;
                    }
                    let key = if a.0 <= b.0 { (a, b) } else { (b, a) };
                    if !rep_pairs.insert(key) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};

    use super::*;

    #[test]
    fn same_as_clusters_merge() {
        let mut o = Ontology::new();
        let a = o
            .entity_id("http://ex.org/a", EntityKind::Individual)
            .unwrap();
        let b = o
            .entity_id("http://ex.org/b", EntityKind::Individual)
            .unwrap();
        o.add_axiom(Axiom::SameIndividual(vec![a, b])).unwrap();
        let c = same_as_closure(&o);
        assert_eq!(c.representative(a), c.representative(b));
    }
}
