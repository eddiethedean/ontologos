//! Clausification golden tests (HermiT ClausificationTest port subset).

use std::path::Path;

use ontologos_alc::Error;
use ontologos_alc::{clausify, Clause};
use ontologos_core::{ClassExpr, Ontology};

#[test]
fn clausify_named_subclass() -> Result<(), Error> {
    let ontology = Ontology::builder()
        .class("http://ex/A")?
        .class("http://ex/B")?
        .subclass_of("http://ex/A", "http://ex/B")?
        .build()
        .map_err(Error::Core)?;
    let mut ont = ontology;
    let clauses = clausify(&mut ont)?;
    assert!(!clauses.is_empty());
    assert!(clauses
        .clauses()
        .iter()
        .any(|c| matches!(c, Clause::Subsumption { .. })));
    Ok(())
}

#[test]
fn clausify_asymmetry_fixture() -> Result<(), Error> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_structural_clausificationtest_testasymmetry.ofn");
    let mut ontology = ontologos_parser::load_ontology(&path).map_err(Error::Parser)?;
    let clauses = clausify(&mut ontology)?;
    assert!(
        clauses
            .clauses()
            .iter()
            .any(|c| matches!(c, Clause::RoleSubsumption { .. })),
        "expected role subsumption clause"
    );
    Ok(())
}

#[test]
fn nnf_complement_in_dl_store() -> Result<(), Error> {
    let mut ontology = Ontology::builder()
        .class("http://ex/A")?
        .class("http://ex/B")?
        .build()
        .map_err(Error::Core)?;
    let a = ontology.lookup_entity("http://ex/A").unwrap();
    let b = ontology.lookup_entity("http://ex/B").unwrap();
    let b_ce = ontology.dl_mut().intern_ce(ClassExpr::Atomic(b));
    let not_b = ontology.dl_mut().intern_ce(ClassExpr::Not(b_ce));
    let sub = ontology.dl_mut().intern_ce(ClassExpr::Atomic(a));
    ontology
        .dl_mut()
        .push_axiom(ontologos_core::DlAxiom::SubClassOf { sub, sup: not_b });
    let clauses = clausify(&mut ontology)?;
    assert!(!clauses.is_empty());
    Ok(())
}
