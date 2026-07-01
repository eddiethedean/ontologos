use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn wg(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

#[test]
fn bottom_object_property_wg_is_inconsistent() {
    let path = wg("wg/New-2DFeature-2DBottomObjectProperty-2D001/premise.rdf");
    let ont = load_ontology(&path).expect("load");
    assert!(
        !is_consistent(&ont).expect("consistency check"),
        "bottom object property feature should be inconsistent"
    );
}

#[test]
#[ignore = "manual debug — dump premise/conclusion DL mapping"]
fn debug_domain_cond() {
    let prem = wg("wg/Rdfbased-2Dsem-2Drdfs-2Ddomain-2Dcond/premise.rdf");
    let conc = wg("wg/Rdfbased-2Dsem-2Drdfs-2Ddomain-2Dcond/conclusion.rdf");
    for (label, path) in [("prem", &prem), ("conc", &conc)] {
        let ont = load_ontology(path).expect("load");
        eprintln!(
            "{label} dl={} core={}",
            ont.dl().axiom_count(),
            ont.axiom_count()
        );
        for (id, ce) in ont.dl().expressions() {
            eprintln!("  CeId({}): {ce:?}", id.0);
        }
    }
}

#[test]
#[ignore = "manual debug — dump boolean intersection WG mapping"]
fn debug_bool_intersection() {
    let prem = wg("wg/Rdfbased-2Dsem-2Dbool-2Dintersection-2Dinst-2Dcomp/premise.rdf");
    let conc = wg("wg/Rdfbased-2Dsem-2Dbool-2Dintersection-2Dinst-2Dcomp/conclusion.rdf");
    for (label, path) in [("prem", &prem), ("conc", &conc)] {
        let ont = load_ontology(path).expect("load");
        eprintln!(
            "{label} dl={} core={}",
            ont.dl().axiom_count(),
            ont.axiom_count()
        );
        for ax in ont.dl().axioms() {
            eprintln!("  dl {ax:?}");
        }
    }
}
