use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn wg(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

#[test]
fn debug_keys_002() {
    let ont = load_ontology(&wg("wg/New-2DFeature-2DKeys-2D002/premise.rdf")).unwrap();
    eprintln!("dl={} core={}", ont.dl().axiom_count(), ont.axiom_count());
    for ax in ont.dl().axioms() {
        eprintln!("  {ax:?}");
    }
    let c = is_consistent(&ont).unwrap();
    eprintln!("consistent={c}");
    assert!(!c);
}

#[test]
fn debug_npa_dat() {
    let ont = load_ontology(&wg("wg/Rdfbased-2Dsem-2Dnpa-2Ddat-2Dfw/premise.rdf")).unwrap();
    eprintln!("dl={} core={}", ont.dl().axiom_count(), ont.axiom_count());
    for ax in ont.dl().axioms() {
        eprintln!("  {ax:?}");
    }
    let c = is_consistent(&ont).unwrap();
    eprintln!("consistent={c}");
    assert!(!c);
}

#[test]
fn debug_i55_003() {
    let ont = load_ontology(&wg("wg/TestCase-3AWebOnt-2DI5.5-2D003/premise.rdf")).unwrap();
    eprintln!("dl={} core={}", ont.dl().axiom_count(), ont.axiom_count());
    for ax in ont.dl().axioms() {
        eprintln!("  {ax:?}");
    }
    let c = is_consistent(&ont).unwrap();
    eprintln!("consistent={c}");
    assert!(!c);
}
