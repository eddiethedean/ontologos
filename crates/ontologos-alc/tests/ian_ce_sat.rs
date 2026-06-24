use ontologos_alc::{DlOntology, TableauSeed};
use ontologos_core::DlAxiom;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn ax(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

fn ce_sat(ofn: &str, ce_ofn: &str) -> bool {
    let assertion = format!("ClassAssertion({} :__probe__)", ce_ofn);
    let body = format!(
        "Prefix(:=<file:/c/test.owl#>)\nPrefix(a:=<file:/c/test.owl#>)\nPrefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)\nPrefix(owl:=<http://www.w3.org/2002/07/owl#>)\nPrefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\nOntology(<file:/c/test.owl#>\n{assertion}\n)"
    );
    let temp = std::env::temp_dir().join(format!(
        "ian-ce-probe-{}-{}.ofn",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&temp, &body).unwrap();
    let probe = load_ontology(&temp).unwrap();
    let base = load_ontology(&ax(ofn)).unwrap();
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
    merged
        .dl_mut()
        .import_axioms_from(probe.dl(), |id| entity_map.get(&id).copied().unwrap());
    let dl = DlOntology::from_ontology(&merged).unwrap();
    let ce_id = merged
        .dl()
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::ClassAssertion { class, .. } = axiom else {
                return None;
            };
            Some(*class)
        })
        .last()
        .unwrap();
    ontologos_alc::is_ce_satisfiable_with_seed(&dl, ce_id, &TableauSeed::default()).unwrap()
}

#[test]
fn iant5_ce_is_satisfiable() {
    let ce = "ObjectIntersectionOf(ObjectComplementOf(:a) ObjectSomeValuesFrom(ObjectInverseOf(:f) :a) ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectSomeValuesFrom(ObjectInverseOf(:f) :a)))";
    assert!(
        ce_sat("hermit_reasoner_reasonertest_testiant5.ofn", ce),
        "IanT5 CE should be satisfiable"
    );
}

#[test]
fn iant7b_ce_is_satisfiable() {
    let ce = "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))))";
    assert!(
        ce_sat("hermit_reasoner_reasonertest_testiant7b.ofn", ce),
        "IanT7b CE should be satisfiable"
    );
}
