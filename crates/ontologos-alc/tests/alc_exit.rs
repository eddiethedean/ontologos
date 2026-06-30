//! v1.7 ALC exit criteria — synthetic subsumption + WG unsat + corpus consistency.

use ontologos_alc::{
    DlOntology, Error, TableauSeed, classify, is_ce_satisfiable_with_seed, is_consistent,
};
use ontologos_core::{Axiom, DlAxiom, Ontology};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn wg_premise(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

#[test]
fn alc_domain_existential_subsumption() -> Result<(), Error> {
    let mut ontology = Ontology::builder()
        .class("http://ex/Person")?
        .class("http://ex/Employee")?
        .class("http://ex/Org")?
        .object_property("http://ex/worksFor")?
        .build()
        .map_err(Error::Core)?;
    let person = ontology.lookup_entity("http://ex/Person").unwrap();
    let employee = ontology.lookup_entity("http://ex/Employee").unwrap();
    let org = ontology.lookup_entity("http://ex/Org").unwrap();
    let works_for = ontology.lookup_entity("http://ex/worksFor").unwrap();

    ontology.add_axiom(Axiom::ObjectPropertyDomain {
        property: works_for,
        domain: person,
    })?;
    ontology.add_axiom(Axiom::SubClassOfExistential {
        subclass: employee,
        property: works_for,
        filler: org,
    })?;

    let taxonomy = classify(&ontology)?;
    assert!(
        taxonomy.is_subsumed(employee, person),
        "Employee ⊑ Person from domain(worksFor)"
    );
    Ok(())
}

#[test]
fn dl650_unsat_detected() -> Result<(), Error> {
    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D650/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).map_err(|e| Error::Message(e.to_string()))?;
    let dl = DlOntology::from_ontology(&ont)?;
    let store = ont.dl();
    let unsat = ont
        .lookup_entity("http://oiled.man.example.net/test#Unsatisfiable")
        .expect("unsat");
    let unsat_ce = store
        .expressions()
        .find_map(|(id, expr)| match expr {
            ontologos_core::ClassExpr::Atomic(c) if *c == unsat => Some(id),
            _ => None,
        })
        .expect("unsat ce");
    let equiv_and = store
        .axioms()
        .find_map(|ax| match ax {
            DlAxiom::EquivalentClasses(ids) if ids.contains(&unsat_ce) => {
                ids.iter().copied().find(|&id| id != unsat_ce)
            }
            _ => None,
        })
        .expect("equiv");
    assert!(
        !is_ce_satisfiable_with_seed(&dl, equiv_and, &TableauSeed::default())?,
        "dl-650 Unsatisfiable equiv should be unsat"
    );
    Ok(())
}

#[test]
fn heinsohn_tbox3_classifies() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testheinsohntbox3.ofn",
    );
    let ontology = load_ontology(&path).map_err(|e| Error::Message(e.to_string()))?;
    assert!(is_consistent(&ontology)?);
    let taxonomy = classify(&ontology)?;
    const NS: &str = "file:/c/test.owl#";
    let complex1a = ontology
        .lookup_entity(&format!("{NS}complex1a"))
        .expect("complex1a");
    let complex1b = ontology
        .lookup_entity(&format!("{NS}complex1b"))
        .expect("complex1b");
    assert!(taxonomy.is_subsumed(complex1a, complex1b));
    Ok(())
}

#[test]
fn pizza_is_consistent() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/pizza.owl");
    let ontology = load_ontology(&path).map_err(|e| Error::Message(e.to_string()))?;
    assert!(is_consistent(&ontology)?, "pizza.owl should be consistent");
    Ok(())
}
