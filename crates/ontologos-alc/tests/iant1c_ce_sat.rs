use ontologos_alc::{DlOntology, TableauSeed};
use ontologos_core::DlAxiom;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn ax(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

fn merge_assert(base: &ontologos_core::Ontology, assertion: &str) -> ontologos_core::Ontology {
    let body = format!(
        "Prefix(:=<file:/c/test.owl#>)\nPrefix(a:=<file:/c/test.owl#>)\nPrefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)\nPrefix(owl:=<http://www.w3.org/2002/07/owl#>)\nPrefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\nOntology(<file:/c/test.owl#>\n{assertion}\n)"
    );
    let temp = std::env::temp_dir().join("iant1c-probe.ofn");
    std::fs::write(&temp, &body).unwrap();
    let probe = load_ontology(&temp).unwrap();
    let mut merged = base.clone();
    for (_, axiom) in probe.axioms().iter() {
        let _ = merged.add_axiom(axiom.clone());
    }
    for (_, record) in probe.entities().iter() {
        let Ok(iri) = probe.resolve_iri(record.iri) else {
            continue;
        };
        if merged.lookup_entity(iri).is_none() {
            let _ = merged.entity_id(iri, record.kind);
        }
    }
    let entity_map: std::collections::HashMap<_, _> = probe
        .entities()
        .iter()
        .filter_map(|(id, record)| {
            let iri = probe.resolve_iri(record.iri).ok()?;
            Some((id, merged.lookup_entity(iri)?))
        })
        .collect();
    merged.dl_mut().import_axioms_from(probe.dl(), |id| {
        entity_map.get(&id).copied().unwrap()
    });
    merged
}

#[test]
fn iant1c_ce_is_unsatisfiable() {
    let ce = "ObjectIntersectionOf(:p2 ObjectSomeValuesFrom(ObjectInverseOf(:r) ObjectIntersectionOf(ObjectSomeValuesFrom(:r :p1) ObjectMaxCardinality(1 :r))))";
    let base = load_ontology(&ax("hermit_reasoner_reasonertest_testiant1c.ofn")).unwrap();
    let merged = merge_assert(
        &base,
        &format!("ClassAssertion({ce} :a)"),
    );
    let dl = DlOntology::from_ontology(&merged).unwrap();
    let ce_id = merged
        .dl()
        .axioms()
        .find_map(|a| {
            let DlAxiom::ClassAssertion { class, .. } = a else {
                return None;
            };
            Some(*class)
        })
        .unwrap();
    let sat =
        ontologos_alc::is_ce_satisfiable_with_seed(&dl, ce_id, &TableauSeed::default()).unwrap();
    assert!(
        !sat,
        "IanT1c CE should be unsatisfiable (HermiT expected false)"
    );
}
