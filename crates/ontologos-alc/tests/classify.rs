//! ALC/DL tableau classification integration tests.

use ontologos_alc::{classify, Error};
use ontologos_core::{Axiom, Ontology};

#[test]
fn domain_axiom_infers_subsumption() -> Result<(), Error> {
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
        "Employee ⊑ Person expected from domain(worksFor, Person)"
    );
    Ok(())
}

#[test]
fn heinsohn_tbox3_complex1_subsumption() -> Result<(), Error> {
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;

    const NS: &str = "file:/c/test.owl#";

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testheinsohntbox3.ofn");
    let ontology = load_ontology(&path).map_err(|e| Error::Message(e.to_string()))?;
    let taxonomy = classify(&ontology).map_err(|e| Error::Message(e.to_string()))?;

    let complex1a = ontology
        .lookup_entity(&format!("{NS}complex1a"))
        .expect("complex1a");
    let complex1b = ontology
        .lookup_entity(&format!("{NS}complex1b"))
        .expect("complex1b");

    assert!(
        taxonomy.is_subsumed(complex1a, complex1b),
        "complex1a ⊑ complex1b"
    );
    Ok(())
}

#[test]
fn role_hierarchy_does_not_break_classify() -> Result<(), Error> {
    let mut ontology = Ontology::builder()
        .class("http://ex/A")?
        .class("http://ex/B")?
        .object_property("http://ex/r")?
        .object_property("http://ex/s")?
        .build()
        .map_err(Error::Core)?;
    let a = ontology.lookup_entity("http://ex/A").unwrap();
    let b = ontology.lookup_entity("http://ex/B").unwrap();
    let r = ontology.lookup_entity("http://ex/r").unwrap();
    let s = ontology.lookup_entity("http://ex/s").unwrap();

    ontology.add_axiom(Axiom::SubObjectPropertyOf {
        sub_property: r,
        super_property: s,
    })?;
    ontology.add_axiom(Axiom::SubClassOfExistential {
        subclass: a,
        property: r,
        filler: b,
    })?;

    let taxonomy = classify(&ontology)?;
    assert!(
        taxonomy.unsatisfiable.is_empty(),
        "role hierarchy with existential should remain consistent"
    );
    Ok(())
}
