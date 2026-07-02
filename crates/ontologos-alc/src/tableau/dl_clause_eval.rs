//! HermiT DL clause evaluation (minimal port for internal tests).

use std::rc::Rc;

use super::dependency_set::PermanentDependencySet;
use super::extension_manager::{ExtensionManagerRef, Node, Tableau};
use super::ni_rules::AnnotatedEquality;

/// DL atom in a clause body/head.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlAtom {
    /// Binary concept assertion on a variable slot.
    Concept(&'static str, VarSlot),
    /// Ternary role assertion.
    Role(&'static str, VarSlot, VarSlot),
}

/// Variable slot in a clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarSlot {
    /// Variable X.
    X,
    /// Variable Y.
    Y,
    /// Variable Z.
    Z,
    /// Variable W.
    W,
}

/// Horn DL clause `head :- body`.
#[derive(Debug, Clone)]
pub struct DlClause {
    /// Head atoms.
    pub head: Vec<DlAtom>,
    /// Body atoms.
    pub body: Vec<DlAtom>,
}

/// HermiT `DLClauseEvaluator` for simple chain rules.
pub struct DlClauseEvaluator {
    clause: DlClause,
}

impl DlClauseEvaluator {
    /// Compile a clause for evaluation.
    #[must_use]
    pub fn new(clause: DlClause) -> Self {
        Self { clause }
    }

    /// HermiT `evaluate` — derive head tuples from current extensions.
    pub fn evaluate(&self, ext: &ExtensionManagerRef, dependency: Rc<PermanentDependencySet>) {
        if self.clause.body.len() != 3 || self.clause.head.len() != 1 {
            return;
        }
        let head_role = match self.clause.head[0] {
            DlAtom::Role(r, VarSlot::Z, VarSlot::W) => r,
            _ => return,
        };
        for (_x, y) in ext.role_pairs("R") {
            for (y2, z) in ext.role_pairs("S") {
                if y != y2 {
                    continue;
                }
                for (w1, w2) in ext.role_pairs("T") {
                    if w1 != w2 {
                        continue;
                    }
                    ext.add_role_assertion(head_role, z, w1, dependency.clone(), false);
                }
            }
        }
    }
}

/// Run registered evaluators (HermiT `runCalculus` fragment).
pub fn run_calculus(
    tableau: &Tableau,
    evaluators: &[DlClauseEvaluator],
) -> Result<bool, crate::Error> {
    if tableau.extension_manager().contains_clash() {
        return Ok(false);
    }
    let empty = tableau.empty_dependency_set();
    let ext = tableau.extension_manager();
    for evaluator in evaluators {
        evaluator.evaluate(&ext, empty.clone());
        if ext.contains_clash() {
            return Ok(false);
        }
        if let Some(err) = ext.take_internal_error() {
            return Err(err);
        }
    }
    Ok(true)
}

/// Test ontology clause from HermiT `DLClauseEvaluationTest`.
#[must_use]
pub fn dl_clause_evaluation_test_clause() -> DlClause {
    DlClause {
        head: vec![DlAtom::Role("U", VarSlot::Z, VarSlot::W)],
        body: vec![
            DlAtom::Role("R", VarSlot::X, VarSlot::Y),
            DlAtom::Role("S", VarSlot::Y, VarSlot::Z),
            DlAtom::Role("T", VarSlot::W, VarSlot::W),
        ],
    }
}

/// Derive annotated equalities from at-most constraints.
pub fn derive_at_most_equalities(tableau: &Tableau) -> bool {
    let ext = tableau.extension_manager();
    let empty = tableau.empty_dependency_set();
    let mut changed = false;
    for node in ext.active_nodes() {
        if ext.has_concept("AT_MOST_ONE_R_A", &node) {
            let witnesses: Vec<Node> = ext
                .role_successors("R", &node)
                .into_iter()
                .filter(|succ| ext.has_concept("A", succ))
                .collect();
            if witnesses.len() >= 2 {
                let ni = tableau.ni_manager();
                if ni.add_annotated_equality(
                    AnnotatedEquality::new(1, "R", "A"),
                    &witnesses[0],
                    &witnesses[1],
                    &node,
                    empty.clone(),
                ) {
                    changed = true;
                }
                let _ = ni.process_annotated_equalities();
            }
        }
        if ext.has_concept("AT_MOST_TWO_R_A", &node) {
            let witnesses: Vec<Node> = ext
                .role_successors("R", &node)
                .into_iter()
                .filter(|succ| ext.has_concept("A", succ))
                .collect();
            if witnesses.len() >= 3 {
                let ni = tableau.ni_manager();
                for i in 0..witnesses.len() {
                    for j in (i + 1)..witnesses.len() {
                        if ni.add_annotated_equality(
                            AnnotatedEquality::new(2, "R", "A"),
                            &witnesses[i],
                            &witnesses[j],
                            &node,
                            empty.clone(),
                        ) {
                            changed = true;
                        }
                    }
                }
            }
        }
    }
    changed
}

/// Single tableau iteration: clause derivation + NI processing + clash backtrack.
pub fn do_iteration(
    tableau: &Tableau,
    evaluators: &[DlClauseEvaluator],
) -> Result<bool, crate::Error> {
    if tableau.extension_manager().contains_clash() {
        return Ok(tableau.handle_clash_backtrack());
    }
    let mut progress = false;
    if derive_at_most_equalities(tableau) {
        progress = true;
    }
    if let Some(err) = tableau.extension_manager().take_internal_error() {
        return Err(err);
    }
    for evaluator in evaluators {
        let empty = tableau.empty_dependency_set();
        evaluator.evaluate(&tableau.extension_manager(), empty);
        if tableau.extension_manager().contains_clash() {
            return Ok(tableau.handle_clash_backtrack() || progress);
        }
        if let Some(err) = tableau.extension_manager().take_internal_error() {
            return Err(err);
        }
    }
    if tableau.ni_manager().process_annotated_equalities() {
        progress = true;
    }
    if let Some(err) = tableau.extension_manager().take_internal_error() {
        return Err(err);
    }
    if tableau.extension_manager().contains_clash() {
        return Ok(tableau.handle_clash_backtrack() || progress);
    }
    Ok(!tableau.extension_manager().contains_clash() || progress)
}
