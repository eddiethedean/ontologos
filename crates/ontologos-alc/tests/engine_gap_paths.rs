use ontologos_alc::{DlOntology, TableauSeed, is_consistent};
use ontologos_core::ClassExpr;
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
    let temp = std::env::temp_dir().join(format!(
        "engine-gap-probe-{}-{}.ofn",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&temp, &body).unwrap();
    let probe = load_ontology(&temp).unwrap();
    let mut merged = base.clone();
    for (_, a) in probe.axioms().iter() {
        let _ = merged.add_axiom(a.clone());
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
    merged
}

#[test]
fn precompute_a_class_sat_paths() {
    let ont = load_ontology(&ax(
        "hermit_reasoner_reasonertest_testprecomputedisjointclasses.ofn",
    ))
    .unwrap();
    let a = ont.lookup_entity("file:/c/test.owl#A").unwrap();
    let dl = DlOntology::from_ontology(&ont).unwrap();
    let ce = dl
        .core()
        .dl()
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == a => Some(id),
            _ => None,
        })
        .unwrap();
    let alc = ontologos_alc::is_ce_satisfiable_with_seed(&dl, ce, &TableauSeed::default()).unwrap();
    let merged = merge_assert(&ont, "ClassAssertion(:A :a)");
    let kb = is_consistent(&merged).unwrap();
    eprintln!("precompute alc_ce_sat={alc} kb_consistent={kb}");
}

#[test]
fn iant7b_ce_sat_paths() {
    let ce = "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))))";
    let base = load_ontology(&ax("hermit_reasoner_reasonertest_testiant7b.ofn")).unwrap();
    let merged = merge_assert(&base, &format!("ClassAssertion({ce} :a)"));
    let dl = DlOntology::from_ontology(&merged).unwrap();
    let ce_id = merged
        .dl()
        .axioms()
        .find_map(|a| {
            let ontologos_core::DlAxiom::ClassAssertion { class, .. } = a else {
                return None;
            };
            Some(*class)
        })
        .unwrap();
    let alc =
        ontologos_alc::is_ce_satisfiable_with_seed(&dl, ce_id, &TableauSeed::default()).unwrap();
    let kb = is_consistent(&merged).unwrap();
    eprintln!("iant7b alc_ce_sat={alc} kb_consistent={kb}");
}
