use ontologos_dl::classify;
use ontologos_parser::load_ontology;
use std::path::Path;

fn entailment_holds(premise_path: &Path, conclusion_path: &Path) -> bool {
    let premise = load_ontology(premise_path).expect("premise");
    let conclusion = load_ontology(conclusion_path).expect("conclusion");
    let prem_tax = classify(&premise).expect("prem classify");
    let mut merged = premise.clone();
    for (_, axiom) in conclusion.axioms().iter() {
        let _ = merged.add_axiom(axiom.clone());
    }
    for axiom in conclusion.dl().axioms() {
        merged.dl_mut().push_axiom(axiom.clone());
    }
    let merged_tax = classify(&merged).expect("merged classify");
    for &(sub, sup) in &merged_tax.subsumptions {
        if !prem_tax.is_subsumed(sub, sup) {
            return false;
        }
    }
    for &class in &merged_tax.unsatisfiable {
        if !prem_tax.unsatisfiable.contains(&class) {
            return false;
        }
    }
    true
}

#[test]
fn i46_003_entailment() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/hermit/wg");
    let prem = base.join("TestCase-3AWebOnt-2DI4.6-2D003/premise.rdf");
    let conc = base.join("TestCase-3AWebOnt-2DI4.6-2D003/conclusion.rdf");
    let entailed = entailment_holds(&prem, &conc);
    eprintln!("I4.6-003 entailed={entailed}");
    assert!(entailed);
}
