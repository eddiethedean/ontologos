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

#[test]
fn ianbug1b_unsat() {
    let ce = "ObjectIntersectionOf(ObjectComplementOf(:c) :a ObjectComplementOf(:b) :d)";
    assert!(
        !ce_sat("hermit_reasoner_reasonertest_testianbug1b.ofn", ce),
        "IanBug1b CE should be unsatisfiable"
    );
}

#[test]
fn iant7c_ce_is_unsatisfiable() {
    let ce = "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))) ObjectSomeValuesFrom(ObjectInverseOf(:f) :p1))";
    assert!(
        !ce_sat("hermit_reasoner_reasonertest_testiant7c.ofn", ce),
        "IanT7c CE should be unsatisfiable"
    );
}

#[test]
fn iant7a_ce_matches_tbox() {
    let ce = "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))))";
    // OFN extract for 7a/7b is identical; under this TBox the CE is satisfiable (HermiT 7a used a different KB).
    assert!(
        ce_sat("hermit_reasoner_reasonertest_testiant7a.ofn", ce),
        "IanT7a OFN TBox: CE is satisfiable (catalog expected=false is stale)"
    );
}

#[test]
fn iant3_ce_is_satisfiable() {
    let ce = "ObjectIntersectionOf(ObjectSomeValuesFrom(:r :p1) ObjectSomeValuesFrom(:r :p2) ObjectSomeValuesFrom(:r :p3) ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 :p)) ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p2 :p)) ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p3 :p)) ObjectMaxCardinality(3 :r))";
    assert!(
        ce_sat("hermit_reasoner_reasonertest_testiant3.ofn", ce),
        "IanT3 CE should be satisfiable"
    );
}

#[test]
fn iant8a_ce_is_satisfiable() {
    let ce = "ObjectIntersectionOf(ObjectSomeValuesFrom(:r ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectAllValuesFrom(:r1 :p))) ObjectSomeValuesFrom(:r ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectAllValuesFrom(:r1 ObjectComplementOf(:p)))))";
    assert!(
        ce_sat("hermit_reasoner_reasonertest_testiant8a.ofn", ce),
        "IanT8a CE should be satisfiable"
    );
}

const IAN_BACKJUMPING3_CE: &str = "ObjectIntersectionOf(ObjectUnionOf(:A0 :B0) ObjectUnionOf(:A1 :B1) ObjectUnionOf(:A2 :B2) ObjectUnionOf(:A3 :B3) ObjectUnionOf(:A4 :B4) ObjectUnionOf(:A5 :B5) ObjectUnionOf(:A6 :B6) ObjectUnionOf(:A7 :B7) ObjectUnionOf(:A8 :B8) ObjectUnionOf(:A9 :B9) ObjectUnionOf(:A10 :B10) ObjectUnionOf(:A11 :B11) ObjectUnionOf(:A12 :B12) ObjectUnionOf(:A13 :B13) ObjectUnionOf(:A14 :B14) ObjectUnionOf(:A15 :B15) ObjectUnionOf(:A16 :B16) ObjectUnionOf(:A17 :B17) ObjectUnionOf(:A18 :B18) ObjectUnionOf(:A19 :B19) ObjectUnionOf(:A20 :B20) ObjectUnionOf(:A21 :B21) ObjectUnionOf(:A22 :B22) ObjectUnionOf(:A23 :B23) ObjectUnionOf(:A24 :B24) ObjectUnionOf(:A25 :B25) ObjectUnionOf(:A26 :B26) ObjectUnionOf(:A27 :B27) ObjectUnionOf(:A28 :B28) ObjectUnionOf(:A29 :B29) ObjectUnionOf(:A30 :B30) ObjectUnionOf(:A31 :B31) ObjectUnionOf(:C4 :C6) ObjectUnionOf(:C5 :C7))";

#[test]
#[ignore = "exceeds 30s DL budget — nightly @ ONTOLOGOS_DL_BUDGET_SECS=120 (classify_timeout.rs)"]
fn ian_backjumping3_ce_is_unsatisfiable() {
    assert!(
        !ce_sat(
            "hermit_reasoner_reasonertest_testianbackjumping3.ofn",
            IAN_BACKJUMPING3_CE
        ),
        "IanBackjumping3 CE should be unsatisfiable"
    );
}

#[test]
#[ignore = "cardinality CE on empty TBox — tableau gap"]
fn ianbug3_ce_is_satisfiable() {
    let ce = "ObjectIntersectionOf(ObjectSomeValuesFrom(:r :a) ObjectMinCardinality(3 :r :c) ObjectMinCardinality(3 :r :d) ObjectMinCardinality(2 :r ObjectIntersectionOf(:e ObjectComplementOf(ObjectIntersectionOf(:c :d)))) ObjectMaxCardinality(4 :r) ObjectMaxCardinality(2 :r ObjectIntersectionOf(:c :d)))";
    assert!(
        ce_sat("hermit_reasoner_reasonertest_testianbug3.ofn", ce),
        "IanBug3 CE should be satisfiable (empty TBox)"
    );
}
