//! Triage harness for WG consistency cases listed in phase 4 burndown.

use ontologos_dl::{is_consistent, is_datatype_consistent};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn wg_premise(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

fn check(rel: &str, expected: bool) -> Result<(), String> {
    let path = wg_premise(rel);
    let ont = load_ontology(&path).map_err(|e| format!("{rel}: load: {e}"))?;
    let dt = is_datatype_consistent(&ont);
    let actual = is_consistent(&ont).map_err(|e| format!("{rel}: check: {e}"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{rel}: expected consistent={expected}, got {actual} (datatype={dt})"
        ))
    }
}

#[test]
fn wg_consistency_burndown_triage() {
    let missed_inconsistency = [
        ("wg/New-2DFeature-2DRational-2D002/premise.rdf", false),
        ("wg/One_equals_two/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2DThing-2D005/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D608/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D650/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D910/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2DdisjointWith-2D010/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2Dmiscellaneous-2D204/premise.rdf", false),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D502/premise.rdf", false),
    ];
    let spurious_inconsistency = [
        ("wg/TestCase-3AWebOntology-2D005/premise.rdf", true), // placeholder wrong
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D005/premise.rdf", true),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D018/premise.rdf", true),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D020/premise.rdf", true),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D021/premise.rdf", true),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D024/premise.rdf", true),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D025/premise.rdf", true),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D624/premise.rdf", true),
        ("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D625/premise.rdf", true),
        ("wg/TestCase-3AWebOnt-2Dmiscellaneous-2D002/premise.rdf", true),
    ];

    let mut failures = Vec::new();
    for (rel, expected) in missed_inconsistency
        .iter()
        .chain(spurious_inconsistency.iter().filter(|(r, _)| !r.contains("WebOntology")))
    {
        if let Err(msg) = check(rel, *expected) {
            eprintln!("FAIL: {msg}");
            failures.push(msg);
        } else {
            eprintln!("OK: {rel}");
        }
    }
    assert!(failures.is_empty(), "failures:\n{}", failures.join("\n"));
}

#[test]
fn diagnose_dl005_is_consistent_steps() {
    use ontologos_alc::{DlOntology, TableauSeed};
    use ontologos_core::CeId;

    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D005/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    let dl = DlOntology::from_ontology(&ont).expect("dl");
    let seed = TableauSeed::default();
    eprintln!("is_consistent={:?}", is_consistent(&ont));
    eprintln!(
        "tableau_default={:?}",
        ontologos_alc::tableau_is_consistent_with_seed(&ont, &seed)
    );
    if let Some(id) = ont.lookup_entity("http://oiled.man.example.net/test#Satisfiable") {
        let sat =
            ontologos_alc::is_named_class_satisfiable_with_seed(&dl, id, &seed).expect("sat");
        eprintln!("Satisfiable class sat={sat}");
        let store = ont.dl();
        let def = store.expressions().find_map(|(ce, e)| match e {
            ontologos_core::ClassExpr::Atomic(c) if *c == id => Some(ce),
            _ => None,
        });
        if let Some(def) = def {
            for ax in store.axioms() {
                if let ontologos_core::DlAxiom::EquivalentClasses(ids) = ax {
                    if ids.contains(&def) {
                        for &other in ids {
                            if other != def {
                                eprintln!(
                                    "Satisfiable equiv ce={other:?} expr={:?}",
                                    store.ce(other)
                                );
                                if let Some(ontologos_core::ClassExpr::And(ops)) = store.ce(other) {
                                    for op in ops {
                                        eprintln!("  conjunct {op:?}: {:?}", store.ce(*op));
                                    }
                                }
                                eprintln!("  ce4={:?}", store.ce(CeId(4)));
                                eprintln!("  ce3={:?}", store.ce(CeId(3)));
                                eprintln!("  ce1={:?}", store.ce(CeId(1)));
                                eprintln!("  ce2={:?}", store.ce(CeId(2)));
                                let sat56 = ontologos_alc::is_ce_intersection_satisfiable_with_seed(
                                    &dl, CeId(5), CeId(6), &seed,
                                )
                                .expect("sat");
                                eprintln!("ce5 AND ce6 sat={sat56}");
                                let sat57 = ontologos_alc::is_ce_intersection_satisfiable_with_seed(
                                    &dl, CeId(5), CeId(7), &seed,
                                )
                                .expect("sat");
                                eprintln!("ce5 AND ce7 sat={sat57}");
                                for (label, ce) in [
                                    ("ce5 alone", CeId(5)),
                                    ("ce6 alone", CeId(6)),
                                    ("full and", CeId(8)),
                                ] {
                                    let sat = ontologos_alc::is_ce_satisfiable_with_seed(
                                        &dl, ce, &seed,
                                    )
                                    .expect("sat");
                                    eprintln!("{label} sat={sat}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn diagnose_dl018_tableau() {
    use ontologos_alc::{DlOntology, TableauSeed, is_named_class_satisfiable_with_seed, tableau_is_consistent_with_seed};

    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D018/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    let dl = DlOntology::from_ontology(&ont).expect("dl");
    let seed = TableauSeed::default();
    eprintln!("kb={:?}", tableau_is_consistent_with_seed(&ont, &seed));
    use ontologos_alc::is_ce_satisfiable_with_seed;
    use ontologos_core::CeId;
    eprintln!(
        "and18 sat={:?}",
        is_ce_satisfiable_with_seed(&dl, CeId(18), &seed)
    );
    if let Some(id) = ont.lookup_entity("http://oiled.man.example.net/test#Satisfiable") {
        let store = ont.dl();
        let ce = store.expressions().find_map(|(cid, e)| match e {
            ontologos_core::ClassExpr::Atomic(c) if *c == id => Some(cid),
            _ => None,
        });
        eprintln!("atomic ce={ce:?} equiv18 sat={:?}", is_ce_satisfiable_with_seed(&dl, CeId(18), &seed));
        eprintln!(
            "Satisfiable sat={:?}",
            is_named_class_satisfiable_with_seed(&dl, id, &seed)
        );
        let store = ont.dl();
        let def = store.expressions().find_map(|(ce, e)| match e {
            ontologos_core::ClassExpr::Atomic(c) if *c == id => Some(ce),
            _ => None,
        });
        if let Some(def) = def {
            for ax in store.axioms() {
                if let ontologos_core::DlAxiom::EquivalentClasses(ids) = ax {
                    if ids.contains(&def) {
                        for &other in ids {
                            if other != def {
                                eprintln!("equiv {other:?} => {:?}", store.ce(other));
                                if let Some(ontologos_core::ClassExpr::And(ops)) = store.ce(other) {
                                    for op in ops {
                                        if let Some(ontologos_core::ClassExpr::Some { filler, .. }) =
                                            store.ce(*op)
                                        {
                                            eprintln!("  filler {filler:?}: {:?}", store.ce(*filler));
                                        }
                                    }
                                    eprintln!(
                                        "  equiv And sat={:?}",
                                        is_ce_satisfiable_with_seed(&dl, other, &seed)
                                    );
                                }
                                if let Some(ontologos_core::ClassExpr::And(ops)) = store.ce(other) {
                                    for op in ops {
                                        eprintln!("  conjunct {op:?}: {:?}", store.ce(*op));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn dl601_unsatisfiable_class_sat() {
    use ontologos_alc::{DlOntology, TableauSeed, is_named_class_satisfiable_with_seed};

    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    let dl = DlOntology::from_ontology(&ont).expect("dl");
    let id = ont
        .lookup_entity("http://oiled.man.example.net/test#Unsatisfiable")
        .expect("Unsatisfiable");
    let sat =
        is_named_class_satisfiable_with_seed(&dl, id, &TableauSeed::default()).expect("sat");
    eprintln!("Unsatisfiable sat={sat}");
    assert!(!is_consistent(&ont).expect("check"));
}

#[test]
fn dl018_is_consistent() {
    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D018/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    assert!(is_consistent(&ont).expect("check"));
}

#[test]
fn spot_check_consistency_fixes() {
    let cases = [
        ("disjoint-010", "wg/TestCase-3AWebOnt-2DdisjointWith-2D010/premise.rdf", false),
        ("dl-601", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601/premise.rdf", false),
        ("dl-018", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D018/premise.rdf", true),
        ("dl-020", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D020/premise.rdf", true),
        ("dl-021", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D021/premise.rdf", true),
        ("dl-024", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D024/premise.rdf", true),
        ("dl-025", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D025/premise.rdf", true),
        ("dl-624", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D624/premise.rdf", true),
        ("dl-625", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D625/premise.rdf", true),
    ];
    for (name, rel, expected) in cases {
        let ont = load_ontology(&wg_premise(rel)).expect("load");
        let actual = is_consistent(&ont).expect("check");
        assert_eq!(actual, expected, "{name}");
    }
}

#[test]
fn diagnose_dl608_unsatisfiable() {
    use ontologos_alc::{DlOntology, TableauSeed, is_ce_satisfiable_with_seed, is_named_class_satisfiable_with_seed};

    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D608/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    let dl = DlOntology::from_ontology(&ont).expect("dl");
    let store = ont.dl();
    for ax in store.axioms() {
        eprintln!("dl axiom: {ax:?}");
    }
    for (_, ax) in ont.axioms().iter() {
        if matches!(ax, ontologos_core::Axiom::SubClassOf { .. }) {
            eprintln!("core sub: {ax:?}");
        }
    }
    let id = ont
        .lookup_entity("http://oiled.man.example.net/test#Unsatisfiable")
        .expect("Unsatisfiable");
    let p2 = ont
        .lookup_entity("http://oiled.man.example.net/test#p2")
        .expect("p2");
    let seed = TableauSeed::default();
    eprintln!("p2 named sat={:?}", is_named_class_satisfiable_with_seed(&dl, p2, &seed));
    let taxonomy = ontologos_dl::classify(&ont).expect("classify");
    eprintln!("taxonomy unsat: {:?}", taxonomy.unsatisfiable.len());
    for u in &taxonomy.unsatisfiable {
        if let Ok(iri) = ont.resolve_iri(ont.entity(*u).unwrap().iri) {
            if iri.contains("p2") || iri.contains("Unsatisfiable") {
                eprintln!("  unsat class: {iri}");
            }
        }
    }
    eprintln!(
        "Unsatisfiable named sat={:?}",
        is_named_class_satisfiable_with_seed(&dl, id, &seed)
    );
    if let Some(ce) = store.expressions().find_map(|(cid, e)| match e {
        ontologos_core::ClassExpr::Atomic(c) if *c == id => Some(cid),
        _ => None,
    }) {
        eprintln!("Unsatisfiable ce sat={:?}", is_ce_satisfiable_with_seed(&dl, ce, &seed));
        for ax in store.axioms() {
            if let ontologos_core::DlAxiom::EquivalentClasses(ids) = ax {
                if ids.contains(&ce) {
                    for &other in ids {
                        if other != ce {
                            eprintln!("equiv {other:?}: {:?}", store.ce(other));
                            eprintln!("  and sat={:?}", is_ce_satisfiable_with_seed(&dl, other, &seed));
                            if let Some(ontologos_core::ClassExpr::And(ops)) = store.ce(other) {
                                for op in ops {
                                    eprintln!("    conjunct {op:?}: {:?}", store.ce(*op));
                                    eprintln!("      sat={:?}", is_ce_satisfiable_with_seed(&dl, *op, &seed));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("consistent={:?}", is_consistent(&ont));
}

#[test]
fn diagnose_flower_and_one_equals_two() {
    let cases = [
        ("One_equals_two", "wg/One_equals_two/premise.rdf", false),
        ("dl-601", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601/premise.rdf", false),
        ("dl-608", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D608/premise.rdf", false),
        ("disjointWith-010", "wg/TestCase-3AWebOnt-2DdisjointWith-2D010/premise.rdf", false),
    ];
    for (name, rel, expected) in cases {
        let ont = load_ontology(&wg_premise(rel)).expect("load");
        let actual = is_consistent(&ont).expect("check");
        eprintln!("{name}: expected={expected} actual={actual} datatype={}", is_datatype_consistent(&ont));
    }
}

#[test]
fn diagnose_satisfiable_class_sat() {
    use ontologos_alc::{DlOntology, TableauSeed, is_named_class_satisfiable_with_seed};
    use ontologos_core::EntityKind;

    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D005/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    let dl = DlOntology::from_ontology(&ont).expect("dl");
    let seed = TableauSeed::default();
    for (id, rec) in ont.entities().iter() {
        if rec.kind != EntityKind::Class {
            continue;
        }
        let Ok(iri) = ont.resolve_iri(rec.iri) else {
            continue;
        };
        if iri.contains("Satisfiable") {
            let sat = is_named_class_satisfiable_with_seed(&dl, id, &seed).expect("sat");
            eprintln!("{iri} satisfiable={sat}");
        }
    }
}

#[test]
fn diagnose_priority_cases() {
    let cases = [
        ("Rational-003", "wg/New-2DFeature-2DRational-2D003/premise.rdf"),
        ("Thing-004", "wg/TestCase-3AWebOnt-2DThing-2D004/premise.rdf"),
        ("Thing-005", "wg/TestCase-3AWebOnt-2DThing-2D005/premise.rdf"),
        ("dl-005", "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D005/premise.rdf"),
    ];
    for (name, rel) in cases {
        let ont = load_ontology(&wg_premise(rel)).expect("load");
        eprintln!("\n=== {name} ===");
        eprintln!("dl axioms={}", ont.dl().axiom_count());
        for ax in ont.dl().axioms() {
            eprintln!("  {ax:?}");
        }
        eprintln!(
            "datatype={} consistent={:?}",
            is_datatype_consistent(&ont),
            is_consistent(&ont)
        );
    }
}
